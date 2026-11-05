# m3-close-b19-code-r1：批 19 产品代码改动全文（HEAD 到 2026-10-29 主工作区）

```diff
diff --git a/crates/singlefs-core/src/admission.rs b/crates/singlefs-core/src/admission.rs
index 7a087edcb..1189dfdec 100644
--- a/crates/singlefs-core/src/admission.rs
+++ b/crates/singlefs-core/src/admission.rs
@@ -1860,7 +1860,7 @@ mod tests {
     /// 请求 1 个单元。按副本之和记：每块盘扣墓碑 2 + 保留池 4，剩 1 槽 < 数据单元 2 槽 ⇒ 拒。
     /// 判别力：换成按一份副本记（记下 2 槽、摊到每块盘只扣 1），那一格翻成放行——而每块盘真要落 2 + 4 + 2 = 8 > 7。
     #[test]
-    fn c375_two_devices_of_seven_slots_with_checkpoint_cost_four_and_one_tombstone_container_refuse_one_data_unit(
+    fn two_devices_of_seven_slots_with_checkpoint_cost_four_and_one_tombstone_container_counted_over_every_replica_refuse_one_data_unit(
     ) {
         let replicas = replicas_of_two_devices();
         let tombstone_container_of_one_copy = slots(2);
@@ -1905,7 +1905,8 @@ mod tests {
     /// 判别力两条：把保留池改回池级标量扣一份（每块盘只扣一半），或让盘 1 的富余补盘 0，那一格都翻成放行——
     /// 而固定点每一块要在盘 0 上落一份，盘 0 真装不下。
     #[test]
-    fn c355_a_device_one_slot_short_is_refused_even_when_the_other_device_has_plenty() {
+    fn device_one_slot_short_of_its_own_checkpoint_reserve_share_is_refused_even_when_the_other_device_has_plenty(
+    ) {
         let replicas = replicas_of_two_devices();
         let reading = AdmissionReading::new(
             vec![empty_device(0, 5), empty_device(1, 100)],
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 1c6a15364..81a712264 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -12,7 +12,8 @@
 //! `DeviceFreeMap::empty_segments` 每次被读都把内存里逐段的已用槽数数一遍（不读盘，代价随段数长）。
 
 use singlefs_format::{
-    ALLOCATION_RECORD_BYTES, CLUSTER_SEGMENT_SLOTS, JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES,
+    ALLOCATION_RECORD_BYTES, CLUSTER_SEGMENT_SLOTS, JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_REGIONS,
+    SLOT_BYTES,
 };
 
 use std::collections::{BTreeMap, BTreeSet};
@@ -1790,8 +1791,27 @@ pub fn oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
 pub struct ReclaimFloorWhenTheRootWasRecorded {
     /// 这一次回收用的门槛：max(挂载时生效的 F, 环里最旧有效根并上钉值)（`mount::reclaim_floor`）。
     pub reclaim_floor: CheckpointTxg,
-    /// 记了这条根之后还钉着的槽（挂载那一刻读坏或读回全零、这个进程还没写掉、重读也没解开的）里钉值最小的那个；一个都没钉着是 `None`。
-    pub lowest_pinned_txg: Option<CheckpointTxg>,
+    /// 记了这条根之后还钉着的槽（挂载那一刻读坏或读回全零、这个进程还没写掉、重读也没解开的）里按（钉值, 槽）最小的那一个；
+    /// 一个都没钉着是 `None`。
+    pub lowest_pin: Option<LowestPinnedRingSlot>,
+}
+
+/// 记一条根之后还钉着的槽里按（钉值, 槽）最小的那一个：钉值与槽一起往下带到写根那一刻（C773（读坏与全零根槽的回收门槛钉只在内存里），
+/// 判决 `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R1 采乙）。三个区域的槽 0 钉值都是第 0 代根的 txg 0
+/// （`mount::oldest_txg_a_ring_slot_bad_at_mount_can_have_held`），钉值不再唯一定槽：只带钉值就还原不出钉的是哪一槽，钉槽 p 存的是槽。
+/// 平局按槽（区域在前、槽号在后）破，同一份根环表每次记根取到的是同一槽。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct LowestPinnedRingSlot {
+    pub pinned_txg: CheckpointTxg,
+    pub ring_slot: RootRingSlot,
+}
+
+impl LowestPinnedRingSlot {
+    /// 钉着的槽在根环里的编码 `3·槽 + 区域`：钉槽 p 存它（0 到 3S 之内与落点公式 `root_ring::target_for_publish` 落到这一槽的最小 txg 相同）。
+    #[must_use]
+    pub fn ring_slot_encoding(self) -> u64 {
+        ROOT_RING_REGIONS * self.ring_slot.slot + self.ring_slot.region
+    }
 }
 
 impl ReclaimFloorWhenTheRootWasRecorded {
@@ -1806,10 +1826,13 @@ impl ReclaimFloorWhenTheRootWasRecorded {
         self.reclaim_floor.max(rollback_floor_of_the_root)
     }
 
-    /// 根记录里的钉槽 p。
+    /// 根记录里的钉槽 p：按（钉值, 槽）最小的那一槽的编码 `3·槽 + 区域`，不是它的钉值（槽号为 0 的槽钉值是 0、编码是区域号）。
     #[must_use]
     pub fn pinned_ring_slot(self) -> crate::root_record::PinnedRingSlot {
-        crate::root_record::PinnedRingSlot::of_the_lowest_pin(self.lowest_pinned_txg)
+        crate::root_record::PinnedRingSlot::of_the_lowest_pin(
+            self.lowest_pin
+                .map(LowestPinnedRingSlot::ring_slot_encoding),
+        )
     }
 }
 
@@ -1873,7 +1896,7 @@ pub struct PoolAllocator {
     /// 发布路径隔离过还没交给会话的槽）。R 排他就是分配器的一条合取（D26（后台整理与放置回收） 已定项 1 第 3 条）；不合格段表、常驻隔离集与观测
     /// 住会话（`compaction_plan::CompactionOfThisMount`）。`new` 与 `rebuild_from_records` 起步是产品值、没有活意图。
     compaction: CompactionSwitchesAndTheExcludedRegion,
-    /// 上一次记根时回收用的门槛与还钉着的槽里最小的钉值（[`ReclaimFloorWhenTheRootWasRecorded`]，C773）。只住内存，`new` 与
+    /// 上一次记根时回收用的门槛与还钉着的槽里按（钉值, 槽）最小的那一槽（[`ReclaimFloorWhenTheRootWasRecorded`]，C773）。只住内存，`new` 与
     /// `rebuild_from_records` 起步是 `None`；随分配器整个换回发布之前（失败的发布）一起退回。
     reclaim_floor_when_the_latest_root_was_recorded: Option<ReclaimFloorWhenTheRootWasRecorded>,
 }
@@ -2726,7 +2749,7 @@ impl PoolAllocator {
     ///   `install_root_ring_occupancy(make_filesystem::root_ring_occupancy_after_make_filesystem(..))`；挂载建的分配器（`mount::rebuilt_allocator`）装着。
     /// - 这条根的 txg 不是上一条记到的加一：有一条根没经分配器写出去，这张表说的环已经不是盘上的环，再按它回收会把还被环里的根引用的槽发出去。
     ///
-    /// 记完、回收完之后把这一次回收用的门槛与还钉着的槽里最小的钉值记下（[`Self::reclaim_floor_when_the_latest_root_was_recorded`]，C773）：
+    /// 记完、回收完之后把这一次回收用的门槛与还钉着的槽里按（钉值, 槽）最小的那一槽记下（[`Self::reclaim_floor_when_the_latest_root_was_recorded`]，C773）：
     /// 带单元的发布拿它写根记录的 t 与 p；零单元发布在根落盘之后才调，它的 t、p 照抄上一版，用不到这一对。
     pub fn record_root_written_by_this_process(&mut self, checkpoint_txg: CheckpointTxg) {
         let occupancy = self.root_ring.as_mut().expect(
@@ -2759,10 +2782,15 @@ impl PoolAllocator {
             .unwrap_or_default();
         let still_protected_by_the_isolation = occupancy.placements_the_isolation_still_protects();
         // 钉槽 p 的采样与回收门槛同一刻：这条根已经盖掉它那一槽（盖掉的若是钉着的槽，钉随之解开），之后到装根之前不再记根。
-        let lowest_pinned_txg = occupancy
+        // 钉值与槽一起取，按（钉值, 槽）取最小：三个区域的槽 0 钉值都是 0，只比钉值就分不出取哪一槽（C773 判决 R1 乙）。
+        let lowest_pin = occupancy
             .ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held()
-            .into_values()
-            .min();
+            .into_iter()
+            .map(|(pinned_ring_slot, pinned_txg)| LowestPinnedRingSlot {
+                pinned_txg,
+                ring_slot: pinned_ring_slot,
+            })
+            .min_by_key(|pin| (pin.pinned_txg, pin.ring_slot));
         match displaced {
             Some(
                 RootRingOccupant::AbandonedRoot {
@@ -2805,11 +2833,11 @@ impl PoolAllocator {
         self.reclaim_floor_when_the_latest_root_was_recorded =
             Some(ReclaimFloorWhenTheRootWasRecorded {
                 reclaim_floor,
-                lowest_pinned_txg,
+                lowest_pin,
             });
     }
 
-    /// 这个分配器上一次记根（[`Self::record_root_written_by_this_process`]）回收用的门槛与记完之后还钉着的槽里最小的钉值；
+    /// 这个分配器上一次记根（[`Self::record_root_written_by_this_process`]）回收用的门槛与记完之后还钉着的槽里按（钉值, 槽）最小的那一槽；
     /// 这个分配器还没记过根时 `None`。带单元的发布在记这次的根之后、装根之前读它（C773 的采样点）。
     #[must_use]
     pub fn reclaim_floor_when_the_latest_root_was_recorded(
diff --git a/crates/singlefs-core/src/instance_table_sweep.rs b/crates/singlefs-core/src/instance_table_sweep.rs
index 64c74704e..b1b9494f8 100644
--- a/crates/singlefs-core/src/instance_table_sweep.rs
+++ b/crates/singlefs-core/src/instance_table_sweep.rs
@@ -24,7 +24,7 @@ use std::collections::{BTreeMap, BTreeSet};
 
 use singlefs_format::{
     DATA_UNIT_HEADER_BYTES, INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE,
-    INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT, INSTANCE_TABLE_PAGE_RECORDS, PACKED_UNIT_HEADER_BYTES,
+    INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT, PACKED_UNIT_HEADER_BYTES,
 };
 
 use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
@@ -66,9 +66,10 @@ pub enum InstanceTableSweepSwitch {
 pub struct InstanceTableRowCountThreshold(pub u64);
 
 impl InstanceTableRowCountThreshold {
-    /// 默认值没定（C690（清扫触发阈值的默认值没定））：这里的占位取一片实例表装得下的数据行数 369（370 条记录减去链指针那一条），
-    /// 行数越过它链就长出第二片。不是定案，定了照定案换。
-    pub const PLACEHOLDER_UNTIL_C690: Self = Self(INSTANCE_TABLE_PAGE_RECORDS - 1);
+    /// 产品路径的默认阈值 365 行：用户 2026-10-29 弹窗定（C690（清扫触发阈值的默认值没定））。行数越过它的那一次挂载末尾就扫，
+    /// 链在一片之内（一片装 369 行，370 条记录减去链指针那一条）收回，切换预留不按两片留。
+    // 出处：E183 第三段（H5）的读数，θ = 365 不越过一片的上界、θ = 369 越过（`.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md`「H5 的读数」那一条）。
+    pub const DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE: Self = Self(365);
     /// 有一行就清扫（用例要在少量行上强制进入清扫那一支，`.claude/rules/fs-design.md` 五条硬要求第 2 条）。
     pub const ANY_ROW: Self = Self(0);
 }
@@ -92,10 +93,11 @@ pub struct InstanceTableSweepConfiguration {
 }
 
 impl InstanceTableSweepConfiguration {
-    /// 产品路径（`mount::mount_writable`）：开关关着、阈值取占位（[`InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690`]）。
+    /// 产品路径（`mount::mount_writable`）：开关关着、阈值取默认的 365 行（[`InstanceTableRowCountThreshold::DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE`]）。
     pub const PRODUCT: Self = Self {
         switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
-        row_count_threshold: InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690,
+        row_count_threshold:
+            InstanceTableRowCountThreshold::DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE,
         sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
     };
 }
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index a5a4d2b80..f7b756303 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -23,9 +23,10 @@ use crate::allocator::{
     TxgsARingSlotBadAtMountCanHold, UnitAreaStart,
 };
 use crate::background_compute::{
-    admit_background_compute_settings_with_the_gpu_compute, AdmittedBackgroundComputeSettings,
-    BackgroundComputeOfThisMount, BackgroundComputeSettings, BackgroundComputeSettingsRefused,
-    BackgroundGpuComputeHandle,
+    admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold,
+    AdmittedBackgroundComputeSettings, BackgroundComputeOfThisMount, BackgroundComputeSettings,
+    BackgroundComputeSettingsRefused, BackgroundGcCpuCompensationThreshold,
+    BackgroundGpuComputeHandle, BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
 };
 use crate::block_device::BlockDevice;
 use crate::checksum::crc32_castagnoli;
@@ -369,8 +370,14 @@ pub struct SystemConfigurationUnreadableAfterTheReadStage {
 /// 读阶段判完之后在哪一处、读到系统配置的哪一种读不出。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum SystemConfigurationUnreadableWhen {
-    /// 取号之前的逐盘核（落后支要读每块盘自证过、世代号最大的那一份）读到 `device` 两槽里一份本池自证过的都没有。
-    CheckingEachDeviceBeforeTheAcquisition { device: DeviceIdentity },
+    /// 取号之前的逐盘核（落后支要读每块盘自证过、世代号最大的那一份）读到 `device` 两槽里一份本池自证过的都没有，又没被取号补写：
+    /// 为什么没补写在 `not_rewritten_because`（[`DeviceNotRewrittenByTheAcquisition`]，C874（补写那一支进不了崩溃测试、运行时看不出））。
+    /// 只报盘表次序第一块这样的盘、它第一件没过的（逐盘那一圈遇到第一块就整个返回，补写闸按闸一、闸五、闸三、闸四、闸二的次序判、
+    /// 第一件不过就返回）：管理员按报的那一件修好再挂，可能撞上同一块盘的下一件或下一块盘。
+    CheckingEachDeviceBeforeTheAcquisition {
+        device: DeviceIdentity,
+        not_rewritten_because: DeviceNotRewrittenByTheAcquisition,
+    },
     /// 取号那一刻读见证值时判不出（R2：有槽设备报读错，且没有一块盘两槽都自证过）；`first_device_reporting_a_read_error` 是盘表次序第一块报读错的盘。
     AcquisitionReadsAnUndecidableWitness {
         first_device_reporting_a_read_error: DeviceIdentity,
@@ -1066,6 +1073,11 @@ pub struct MountOutput {
     /// 盘上那一条未完成的整理意图的续做（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1；D8（核心索引结构） 已定项 5
     /// 第 2 条），连同这次挂载会话这一侧的整理状态（会话接着用，`mounted_session::MountedSession::of_the_mount`）。
     pub compaction_continuation: CompactionContinuationAtTheMount,
+    /// 取号写两轮顺带补写的盘（丙'：两槽一份本池自证过的系统配置都没有、过了补写闸的那几块），按盘表次序；没有补写是空的。
+    /// 补写这一支走没走从这里看得出（`.claude/rules/fs-design.md`「五条硬要求」第 4 条；C874（补写那一支进不了崩溃测试、运行时看不出），
+    /// 判决 `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R4）；没补写的原因在拒因的载荷里
+    /// （[`SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition`]）。
+    pub devices_rewritten_by_the_acquisition: Vec<DeviceIdentity>,
 }
 
 /// 可写挂载续做整理意图（[`MountOutput::compaction_continuation`]）：次序在删除续做之后（判决 G8「删除续做 → 为用户请求推的抬 F → GC 批」，
@@ -2908,7 +2920,8 @@ fn ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes(
 
 /// 挂载那一刻读坏（设备报读错，或读得出、不是全零而自证不过）的根环槽各能住哪几个 txg（区域内槽位只由 txg 定，
 /// D22（单元原子性怎么合成） 已定项 2）：
-/// - 可能住过的最旧 txg：落到这个槽、小于 `first_txg` 的最小 txg。回收门槛钉在这里（用户 2026-10-04 定：读不出、自证不过的根槽在回收门槛里
+/// - 可能住过的最旧 txg：落到这个槽、小于 `first_txg` 的最小 txg；mkfs 种第 0 代根的那几个槽取第 0 代根的 txg
+///   （[`oldest_txg_a_ring_slot_bad_at_mount_can_have_held`]）。回收门槛钉在这里（用户 2026-10-04 定：读不出、自证不过的根槽在回收门槛里
 ///   钉在它可能住过的最旧 txg，不再只在一圈窗口里取——窗口之外、崩在记录之后根槽之前留在洞里的旧根也可能就住在这一槽里）。
 /// - 能住的最大 txg：(N − 3S, first_txg − 1] 里落到这个槽的最大 txg（D16（发布语义） 已定项 1「根环表记最大的」，
 ///   `research/prompts/m3-ring-r3-main-verification.md` 第二节第 4 行）：准入抬 F 的上限照 Q3 按它并。上沿取新实例第一次发布之前的那个 txg：
@@ -2953,14 +2966,17 @@ fn txgs_ring_slots_bad_at_mount_can_hold(
     }
     oldest_txg_of_each_bad_slot
         .into_iter()
-        .map(|(ring_slot, oldest_txg_it_can_have_held)| {
+        .map(|(ring_slot, oldest_txg_by_the_formula)| {
             let highest_txg_it_can_hold = *highest_txg_of_each_bad_slot.get(&ring_slot).expect(
                 "这个槽在 [0, first_txg) 里住得下一个 txg，区间 (N − 3S, first_txg − 1] 要么从 0 起、要么长过一圈，它在区间里也住得下一个",
             );
             (
                 ring_slot,
                 TxgsARingSlotBadAtMountCanHold::new(
-                    oldest_txg_it_can_have_held,
+                    oldest_txg_a_ring_slot_bad_at_mount_can_have_held(
+                        ring_slot,
+                        oldest_txg_by_the_formula,
+                    ),
                     highest_txg_it_can_hold,
                 ),
             )
@@ -2968,6 +2984,25 @@ fn txgs_ring_slots_bad_at_mount_can_hold(
         .collect()
 }
 
+/// 挂载那一刻读坏或读回全零、公式在 [0, first_txg) 里落得到的一个槽可能住过的最旧 txg（回收门槛钉在这里）：mkfs 种第 0 代根的那几个槽
+/// （三个区域的槽 0，[`crate::make_filesystem::genesis_root_ring_slots`]）取第 0 代根的 txg——区域 1、2 的槽 0 照公式要等 txg 1、2 才被盖掉，
+/// 那两次的根槽写崩在记录之后、根槽之前时第 0 代根就留在那里（C773（读坏与全零根槽的回收门槛钉只在内存里），判决
+/// `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R1 采乙「第 0 代根所在槽的钉值取 0」）；别的槽取公式落到它的最小 txg
+/// `oldest_txg_by_the_formula`。公式在 [0, first_txg) 里一个 txg 都落不到的槽（年轻的环里还没轮到的）照旧不交回、不钉：
+/// 那时第 0 代根还在区域 0 槽 0，那一槽读得出门槛就是 0、读坏就钉在 0。
+fn oldest_txg_a_ring_slot_bad_at_mount_can_have_held(
+    ring_slot: RootRingSlot,
+    oldest_txg_by_the_formula: CheckpointTxg,
+) -> CheckpointTxg {
+    if crate::make_filesystem::genesis_root_ring_slots()
+        .any(|genesis_slot| genesis_slot == ring_slot)
+    {
+        crate::make_filesystem::GENESIS_CHECKPOINT_TXG
+    } else {
+        oldest_txg_by_the_formula
+    }
+}
+
 /// 重建分配器：从上一版的分配记录重建，按可再分配谓词的门槛回收（当场可用的只到每块盘上最新有效根那一截、某块盘一条有效根都读不出时
 /// 只到环里最旧有效根（RGall），其余扣到这次挂载的暖机落满每块盘，甲），再把只被被抛弃根引用的槽隔离（影子账：被抛弃的根按最新根指着的
 /// 实例表判，D23（journal 的角色与格式） 已定项 14 射程「影子账与按实例表判抛弃留着，理由是崩溃恢复」）。
@@ -3978,12 +4013,16 @@ pub fn rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount<Device:
             });
         }
     }
+    // C724（准入抬 F 上限里同名拒因不在单子上）：交 `None` 只有「第 4 新的不同状态没有、最旧有效根也没有」一条路，而最旧有效根取自 `valid`；
+    // `valid` 为空时 `newest_per_device` 也为空，上面取每块盘最新那一判已经交回 `NoValidRoot`。这里与盘面、故障都无关，走不到。
     let ceiling_from_the_roots_read = ceiling_from_newest_and_non_empty_roots(
         newest_on_every_device,
         newest_first_distinct_states(non_empty),
         valid.iter().map(|root| root.checkpoint_txg).min(),
     )
-    .ok_or(RecoveryFailure::NoValidRoot)?;
+    .expect(
+        "valid 不空（每块盘最新的有效根取得出最小值，空的 valid 在那一判已交回 NoValidRoot），最旧有效根就有：上限一定算得出",
+    );
     let lowest_txg_a_ring_slot_bad_since_the_mount_can_hold =
         ring_slots_bad_on_this_read_taken_as_holding_no_root
             .iter()
@@ -4339,8 +4378,25 @@ pub fn ring_slots_recorded_as_holding_an_abandoned_root_by(
 /// 知道住着根的槽里记成被抛弃根的那几槽不算「重读仍坏就拒」的槽（用户 2026-10-08 定 H1「被抛弃根的槽不拒」）：这一遍读坏就当没有根，
 /// 上限只在有效根上取、本来不看它们；它们也不是挂载那一刻读坏的槽，不并进上限。
 ///
+/// 今天的 F 取现行那一版的根 `current_root` 带的 F。
+///
+/// 共享函数里「一条有效根都没有」那一错（`Recovery(NoValidRoot)`）在这里走不到（C724（准入抬 F 上限里同名拒因不在单子上），判决
+/// `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R2「写明走不到」）：现行那一版的根是这个进程写的，它在有效根里，
+/// 要两条前提，都不是这个函数自己保证的：
+/// 1. 这个进程写出的每一条现行根都经记根（`PoolAllocator::record_root_written_by_this_process`）进了根环表：它的槽是「知道住着根的槽」，
+///    这一遍读坏就重读、仍坏就拒成 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，读得出就进有效根（这个实例在现行那一张实例表里没有行，
+///    不会被判抛弃）；
+/// 2. 根带的 F 不高于它自己的 txg（`valid` 滤的是 txg ≥ F）。
+///
+/// 第 2 条在这里断言；第 1 条破了时共享函数交回 `NoValidRoot`，这里当场断言失败、不往上交（不在 D23（journal 的角色与格式） 已定项 14 的单子上，
+/// 这一处在写行之后跑，判据在崩溃镜像上复算不了，加进单子是类别错）。共享函数 [`rollback_floor_ceiling`] 留着这个错：
+/// 不带这个进程根环表的调用方（实验装置、用例手拼的分配器）在根环全读坏时合法地走到它。
+///
 /// # Errors
-/// 同 [`rollback_floor_ceiling`]。
+/// 同 [`rollback_floor_ceiling`]，除 `Recovery(NoValidRoot)` 之外。
+///
+/// # Panics
+/// `current_root` 带的 F 高于它自己的 txg；共享函数交回 `Recovery(NoValidRoot)`（第 1 条前提破了：有一条现行根没经记根）。
 #[allow(
     clippy::ptr_arg,
     reason = "rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount 走 &Vec：要一个有大小的读者"
@@ -4348,19 +4404,36 @@ pub fn ring_slots_recorded_as_holding_an_abandoned_root_by(
 fn rollback_floor_ceiling_of_this_process<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
     system_configuration: &crate::system_configuration::SystemConfiguration,
-    current_floor: CheckpointTxg,
+    current_root: &RootRecord,
     table: &InstanceTableRecords,
     allocator: &PoolAllocator,
 ) -> Result<CheckpointTxg, MountError> {
-    rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
+    assert!(
+        current_root.rollback_floor <= current_root.checkpoint_txg,
+        "现行那一版的根带的 F {:?} 高于它自己的 txg {:?}：写者给新根带的 F 取盘上已有的根与系统配置里的，新实例第一次发布的 txg 高于环里一切\
+         （D23（journal 的角色与格式） 已定项 14 射程两条前提之二，C724）",
+        current_root.rollback_floor,
+        current_root.checkpoint_txg
+    );
+    let ceiling = rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
         devices,
         system_configuration,
-        current_floor,
+        current_root.rollback_floor,
         table,
         &ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(allocator),
         &ring_slots_bad_at_mount_by(allocator),
         &ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator),
-    )
+    );
+    assert!(
+        !matches!(
+            ceiling,
+            Err(MountError::Recovery(RecoveryFailure::NoValidRoot))
+        ),
+        "这个进程的准入算抬 F 上限读不出一条有效根（现行那一版 txg {:?}）：这个进程写出的现行根没经记根进根环表，它的槽读坏没有重读即拒\
+         （D23（journal 的角色与格式） 已定项 14 射程两条前提之一，C724）",
+        current_root.checkpoint_txg
+    );
+    ceiling
 }
 
 /// 算抬 F 的上限读根环时，这一遍读坏、重读一次仍坏就拒的槽（[`rollback_floor_ceiling`] 的 `ring_slots_whose_reread_still_bad_refuses_the_ceiling`）：
@@ -4442,7 +4515,7 @@ fn raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk<Device
     let ceiling = rollback_floor_ceiling_of_this_process(
         devices,
         &caller_inputs_agreeing_with_the_disk.system_configuration,
-        current.root.rollback_floor,
+        &current.root,
         &current_instance_table,
         allocator,
     )?;
@@ -4820,7 +4893,7 @@ fn raise_the_floor_through<Device: BlockDevice>(
             let ceiling = rollback_floor_ceiling_of_this_process(
                 devices,
                 &system_configuration,
-                current.root.rollback_floor,
+                &current.root,
                 &table,
                 allocator,
             )?;
@@ -5949,11 +6022,23 @@ struct DevicesCheckedBeforeTheAcquisition {
     devices_rewritten_by_the_acquisition: Vec<DeviceIdentity>,
 }
 
+/// 两槽一份本池自证过的系统配置都没有的盘为什么没被取号补写（丙'，C874（补写那一支进不了崩溃测试、运行时看不出），判决
+/// `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R4「补写闸没过的原因进挂载错误成员的载荷」）：闸前那一判（两槽里有一槽设备报读错）
+/// 或补写闸五件里第一件没过的（[`RewriteGateNotMet`]）。拒因的成员表不加成员，原因装在
+/// [`SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition`] 的载荷里。封闭集合，`match` 不写通配臂。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum DeviceNotRewrittenByTheAcquisition {
+    /// 闸前：两槽里有一槽设备报读错（读错的槽里装的是什么看不见、可能是一份更新的见证，不补写，S-辨同一个口径）。
+    SystemConfigurationSlotReportsAReadError,
+    /// 两槽都读得出、都自证不过，补写闸有一件没过。
+    RewriteGateNotMet(RewriteGateNotMet),
+}
+
 /// 两槽全废的盘过不了丙' 补写闸的是哪一件（[`rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration`]）。
-/// 只在核里分辨：调用方照旧拒成 [`SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition`]，不另起错误成员
-/// （拒因的成员表 harness 档与 checker 档按它穷举匹配，加成员要连着改那两边，这一批不碰）。封闭集合，`match` 不写通配臂。
+/// 调用方拒成 [`SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition`]、装在它的载荷里报出来
+/// （[`DeviceNotRewrittenByTheAcquisition::RewriteGateNotMet`]）。封闭集合，`match` 不写通配臂。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
-enum RewriteGateNotMet {
+pub enum RewriteGateNotMet {
     /// 闸一：所选那一版的单元清单是空的，或有一个在这块盘上读不出、字节与验得过的那一份不同，或没有验得过的一份可比。
     UnitsOfTheSelectedVersionNotAllOnTheDevice,
     /// 闸五：这块盘的根环或 journal 环里有一槽设备报读错（读错的槽里可能藏着更新的根或记录，看不见不当没有）。
@@ -5971,7 +6056,7 @@ enum RewriteGateNotMet {
 ///
 /// 每块盘先看它两槽里本池自证过的系统配置。一份都没有：照丙'（用户 2026-10-19 定 S4「开例外，四件闸」，2026-10-20 再加单元区扫描与环槽读错两件；
 /// 三方判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U4 与第三节）过补写闸
-/// （[`rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration`]），都过就记进补写的盘、交给取号补写；有一件不过交 `Err(那块盘)`，
+/// （[`rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration`]），都过就记进补写的盘、交给取号补写；有一件不过交 `Err((那块盘, 那一件))`，
 /// 由调用方按 C331（择根倒挂压过已确认的写） 取甲的拒因报（用户 2026-09-28 定：这一支并进甲的拒因）。
 /// 有自证过的：世代号最大的那一份按落后支那一判（[`what_a_device_behind_the_version_lacks`]，与挂着之后收盘表的入口同一判）。
 ///
@@ -5979,12 +6064,13 @@ enum RewriteGateNotMet {
 /// 外层的提前出口两个：两槽全废又过不了补写闸（整个核交 `Err`）、不落后或落后而不缺（这块盘判完了、看下一块，不记）。
 ///
 /// # Errors
-/// 某块盘两槽里一份本池自证过的系统配置都读不出、又过不了补写闸 ⇒ `Err(那块盘)`。
+/// 某块盘两槽里一份本池自证过的系统配置都读不出、又没被补写 ⇒ `Err((那块盘, 为什么没补写))`：两槽里有一槽设备报读错，或补写闸第一件没过的。
 fn devices_without_the_selected_version<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     system_configuration: &crate::system_configuration::SystemConfiguration,
     selected_version: SelectedVersionCheckedOnEachDevice<'_>,
-) -> Result<DevicesCheckedBeforeTheAcquisition, DeviceIdentity> {
+) -> Result<DevicesCheckedBeforeTheAcquisition, (DeviceIdentity, DeviceNotRewrittenByTheAcquisition)>
+{
     let parameters_of_the_pool = parameters_of_the_system_configuration(system_configuration);
     let mut devices_without = Vec::new();
     let mut devices_rewritten_by_the_acquisition = Vec::new();
@@ -6006,7 +6092,10 @@ fn devices_without_the_selected_version<Reader: PoolReader + ?Sized>(
             // 两槽「全废」只认这一次读得出、自证不过的（撕裂、清零、别的池的）：设备报读错的槽里装的是什么看不见、可能是一份更新的见证，
             // 与 S-辨（一槽撕裂与设备读不出分开）同一个口径，不补写，照今天拒（读阶段判完之后系统配置又读不出）。
             if slots_reporting_a_read_error > 0 {
-                return Err(device);
+                return Err((
+                    device,
+                    DeviceNotRewrittenByTheAcquisition::SystemConfigurationSlotReportsAReadError,
+                ));
             }
             match rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration(
                 reader,
@@ -6019,7 +6108,12 @@ fn devices_without_the_selected_version<Reader: PoolReader + ?Sized>(
                     devices_rewritten_by_the_acquisition.push(device);
                     continue;
                 }
-                Some(_gate_not_met) => return Err(device),
+                Some(gate_not_met) => {
+                    return Err((
+                        device,
+                        DeviceNotRewrittenByTheAcquisition::RewriteGateNotMet(gate_not_met),
+                    ));
+                }
             }
         };
         let Some((newest_system_configuration_journal_tail, lacking)) =
@@ -7035,6 +7129,9 @@ fn establish_instance<Device: BlockDevice>(
             &units_of_the_selected_version,
             &read_route,
         )?;
+    let devices_rewritten_by_the_acquisition = publishes_planned
+        .devices_rewritten_by_the_acquisition
+        .clone();
     let PublishedBetweenAcquisitionAndTheContinuation {
         instance,
         row_publish,
@@ -7124,6 +7221,7 @@ fn establish_instance<Device: BlockDevice>(
             deletion_queue_continuation_outcome,
             instance_table_sweep,
             compaction_continuation,
+            devices_rewritten_by_the_acquisition,
         },
         allocator,
         current,
@@ -7277,10 +7375,13 @@ fn devices_rewritten_by_the_acquisition_after_checking_each_device<Device: Block
             },
         )
     })
-    .map_err(|device| {
+    .map_err(|(device, not_rewritten_because)| {
         refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
             start.rereads.read_stage,
-            SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition { device },
+            SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
+                device,
+                not_rewritten_because,
+            },
         )
     })?;
     if !devices_without.is_empty() {
@@ -8585,7 +8686,7 @@ pub fn mount_writable_with_compaction_switches<Device: BlockDevice>(
 }
 
 /// 同 [`mount_writable_with_test_only_switches`]，交回之前那一判实例表清扫的开关与阈值也由调用方给
-/// （D18（块里携带什么信息） 已定项 11「回收」一条：开关与阈值是配置项，阈值默认值没定，C690；
+/// （D18（块里携带什么信息） 已定项 11「回收」一条：开关与阈值是配置项，阈值默认 365 行，用户 2026-10-29 定，C690；
 /// [`crate::instance_table_sweep::InstanceTableSweepConfiguration`]）。产品路径走 [`mount_writable`]，取
 /// `InstanceTableSweepConfiguration::PRODUCT`。
 ///
@@ -8628,6 +8729,7 @@ pub enum WritableMountWithBackgroundComputeRefused {
 /// 不过就在读任何一块盘之前拒绝挂载，不退回 `cpu`；过了就照 [`mount_writable`] 挂，设置连同实现装进交回的那份参数与盘表
 /// （[`ParametersAndDeviceTableOfTheMount::background_compute_of_this_mount`]），挂载续做与之后的会话照它。
 /// [`mount_writable`] 与别的可写挂载入口都等价于在这里给 `BackgroundComputeSettings::PRODUCT`（`cpu`）、不给实现。
+/// GC 停着时在 CPU 上补跑的阈值取默认（[`mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`]）。
 ///
 /// # Errors
 /// 设置被拒 ⇒ [`WritableMountWithBackgroundComputeRefused::BackgroundComputeSettings`]；
@@ -8638,9 +8740,34 @@ pub fn mount_writable_with_background_compute_settings<Device: BlockDevice>(
     background_compute_settings: BackgroundComputeSettings,
     gpu_compute: Option<BackgroundGpuComputeHandle>,
 ) -> Result<Mounted, WritableMountWithBackgroundComputeRefused> {
-    let background_compute = match admit_background_compute_settings_with_the_gpu_compute(
+    mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold(
+        parameters,
+        devices,
         background_compute_settings,
         gpu_compute,
+        BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
+    )
+}
+
+/// 同 [`mount_writable_with_background_compute_settings`]，另给挂载参数 `background_gc_cpu_compensation_threshold`
+/// （GC 因一批计算出错停着时，池子已分配占总容量超过它就在 CPU 上补跑；用户 2026-10-28 定案，默认 85%）。它与后台计算设置同一套校验：
+/// 超过 100% 就在读任何一块盘之前拒绝挂载（`BackgroundComputeSettingsRefused::GcCpuCompensationThresholdAboveTheWholeCapacity`），盘上逐字节不变。
+///
+/// # Errors
+/// 同 [`mount_writable_with_background_compute_settings`]，设置被拒多一种阈值越界。
+pub fn mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold<
+    Device: BlockDevice,
+>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    background_compute_settings: BackgroundComputeSettings,
+    gpu_compute: Option<BackgroundGpuComputeHandle>,
+    gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
+) -> Result<Mounted, WritableMountWithBackgroundComputeRefused> {
+    let background_compute = match admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold(
+        background_compute_settings,
+        gpu_compute,
+        gc_cpu_compensation_threshold,
     ) {
         Ok(admitted) => admitted,
         Err(refused) => {
@@ -10195,7 +10322,9 @@ mod txgs_ring_slots_bad_at_mount_can_hold_tests {
             .collect()
     }
 
-    /// 参照：[0, first_txg) 与区间 (N − 3S, first_txg − 1] 里的 txg 一个一个走（只给区间装得下的组用）。
+    /// 参照：[0, first_txg) 与区间 (N − 3S, first_txg − 1] 里的 txg 一个一个走（只给区间装得下的组用）。可能住过的最旧 txg：
+    /// 槽号为 0 的槽（mkfs 在每个区域的槽 0 种一条第 0 代根，txg 0）是 0，别的槽是走到的第一个 txg（C773 判决 R1 乙；这里按槽号另写一份，
+    /// 不调实现认第 0 代根槽的那一处）。
     fn walk_every_txg_below_the_first(
         bad_ring_slots: &BTreeSet<RootRingSlot>,
         window_anchor: CheckpointTxg,
@@ -10216,7 +10345,11 @@ mod txgs_ring_slots_bad_at_mount_can_hold_tests {
             }
             oldest_of_each
                 .entry(ring_slot)
-                .or_insert(CheckpointTxg(txg));
+                .or_insert(if ring_slot.slot == 0 {
+                    CheckpointTxg(0)
+                } else {
+                    CheckpointTxg(txg)
+                });
             if txg >= lowest_txg_inside_the_window {
                 highest_of_each.insert(ring_slot, CheckpointTxg(txg));
             }
@@ -10258,6 +10391,41 @@ mod txgs_ring_slots_bad_at_mount_can_hold_tests {
         );
     }
 
+    /// 第 0 代根留过的槽（C773（读坏与全零根槽的回收门槛钉只在内存里），判决 `research/prompts/m3-close-b19-core-r1-main-verification.md`
+    /// 第二节 R1 乙）：first_txg 14、S = 4，区域 1 槽 0 与区域 2 槽 0 落点公式最早住 txg 1、2，mkfs 却在那里种过第 0 代根（txg 0）：
+    /// 可能住过的最旧 txg 取 0；能住的最大 txg 照公式（13、2——区间 [2, 13] 里落到区域 2 槽 0 的只有 2）。区域 0 槽 1 照公式 3。
+    #[test]
+    fn bad_ring_slot_where_make_filesystem_planted_a_genesis_root_is_pinned_at_txg_zero() {
+        let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(4)
+            .expect("4 是 S 区间 [4, 16] 的下界");
+        assert_eq!(
+            txgs_ring_slots_bad_at_mount_can_hold(
+                &BTreeSet::from([
+                    RootRingSlot { region: 0, slot: 1 },
+                    RootRingSlot { region: 1, slot: 0 },
+                    RootRingSlot { region: 2, slot: 0 },
+                ]),
+                CheckpointTxg(13),
+                CheckpointTxg(14),
+                slots_per_region,
+            ),
+            vec![
+                (
+                    RootRingSlot { region: 0, slot: 1 },
+                    TxgsARingSlotBadAtMountCanHold::new(CheckpointTxg(3), CheckpointTxg(3))
+                ),
+                (
+                    RootRingSlot { region: 1, slot: 0 },
+                    TxgsARingSlotBadAtMountCanHold::new(CheckpointTxg(0), CheckpointTxg(13))
+                ),
+                (
+                    RootRingSlot { region: 2, slot: 0 },
+                    TxgsARingSlotBadAtMountCanHold::new(CheckpointTxg(0), CheckpointTxg(2))
+                ),
+            ]
+        );
+    }
+
     /// 年轻的环（first_txg 5，S = 4）：txg 5 起才轮到的槽一个 txg 都住不了，不交回。
     #[test]
     fn a_bad_slot_the_young_ring_has_not_reached_holds_no_txg() {
@@ -10331,7 +10499,7 @@ mod txgs_ring_slots_bad_at_mount_can_hold_tests {
         }
 
         // S = 4，3S = 12：txg 模 12 的余数 r 落在区域 r mod 3 的槽 r div 3（D22 已定项 2 的槽位公式在一个周期里展开），
-        // 可能住过的最旧 txg 就是 r。2^64 模 12 是 4，所以 u64::MAX 模 12 是 3：区间尾 u64::MAX − 12 ..= u64::MAX − 1 里余数是 r 的那个
+        // 可能住过的最旧 txg 就是 r（槽 0 那三个余数 0、1、2 是第 0 代根的 0）。2^64 模 12 是 4，所以 u64::MAX 模 12 是 3：区间尾 u64::MAX − 12 ..= u64::MAX − 1 里余数是 r 的那个
         // txg 是 u64::MAX − k，k 取 1..=12 里 (3 − k) 模 12 等于 r 的那一个。
         let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(4)
             .expect("4 是 S 区间 [4, 16] 的下界");
@@ -10354,7 +10522,7 @@ mod txgs_ring_slots_bad_at_mount_can_hold_tests {
                         slot: residue / 3,
                     },
                     TxgsARingSlotBadAtMountCanHold::new(
-                        CheckpointTxg(residue),
+                        CheckpointTxg(if residue / 3 == 0 { 0 } else { residue }),
                         CheckpointTxg(u64::MAX - distance_below_u64_maximum),
                     ),
                 )
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index a9ad218cc..e56a86461 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -29,10 +29,12 @@ use crate::address::{
 };
 use crate::allocator::{AllocationRecord, Placement, PlacementOnDevice, PoolAllocator};
 use crate::background_compute::{
-    decisions_of_one_segment, BackgroundComputeError, BackgroundComputeOfOneBatch,
-    BackgroundComputeOfThisMount, BackgroundComputeSettings, BackgroundComputeSettingsRefused,
-    BackgroundGpuCompute, BackgroundGpuComputeHandle, BackgroundTask, GpuComputedSegment,
+    decisions_of_one_segment, AllocatedBytesAndCapacityOfADevice, BackgroundComputeError,
+    BackgroundComputeOfOneBatch, BackgroundComputeOfThisMount, BackgroundComputeSettings,
+    BackgroundComputeSettingsRefused, BackgroundGcCpuCompensationThreshold, BackgroundGpuCompute,
+    BackgroundGpuComputeHandle, BackgroundTask, GcCpuCompensationReading, GpuComputedSegment,
     GpuEntryDecision, GpuWorkOfTheBatch, InTheMiddleOfEachBackgroundBatch,
+    WhetherTheTaskRunsTheNextBatch,
 };
 use crate::block_device::{
     check_aligned_and_in_range, BlockDevice, BlockDeviceError, DiscardOutcome,
@@ -208,6 +210,10 @@ pub struct DiscardsAfterThePublishes {
     pub withheld_holding_unswept_orphan_heads: Vec<(DeviceIdentity, SlotNumber)>,
     /// 判孤儿头时读不出实例表或段里的一个槽、没发的段（读不到不等于没有，按装着算）。
     pub withheld_after_a_failed_read: Vec<(DeviceIdentity, SlotNumber)>,
+    /// 分配器上冻结着一次没重发的发布时，一段都不发、待 discard 表原样留着（C928（冻结着时整理照发 discard））：冻结那次的 txg；没冻结是 `None`。
+    /// 冻结时分配器退回了发布之前那一份，而冻结那次的单元、记录、根槽可能已经落盘：退回的那一份里整段可再分配的段可能正装着它的单元，
+    /// 重发之前断电，恢复择到它的根，那一段收过 discard 就读不回它的单元。
+    pub txg_of_the_frozen_publish_withholding_every_discard: Option<CheckpointTxg>,
 }
 
 /// 一次整理运行（显式入口 [`MountedSession::run_compaction`]，或后台入口 [`MountedSession::run_compaction_in_the_background`] 起跑的那一趟）做成了什么。
@@ -223,7 +229,8 @@ pub struct CompactionRun {
     pub discards: DiscardsAfterThePublishes,
     /// 整理每推一次发布（[`advance_compaction_by_one_publish`] 每进一次门：签发、搬一批、删意图，停在判据上的那一次也算）是后台任务的一批，
     /// 按先后一批一项：请求的后端（这一批开头读到的设置）与实际走的路（D26（后台整理与放置回收） 已定项 13 第 1 条；
-    /// `.claude/rules/fs-design.md` 五条硬要求第 4 条）。删除续做那几步不是整理的批，不在这里。
+    /// `.claude/rules/fs-design.md` 五条硬要求第 4 条）。删除续做那几步不是整理的批，不在这里；整理停着、占用不超过补跑阈值而没开成的那一批也不在
+    /// （一个都没算，用户 2026-10-28 定案）。
     pub background_compute_of_each_batch: Vec<BackgroundComputeOfOneBatch>,
 }
 
@@ -270,7 +277,8 @@ pub enum BackgroundCompactionCall {
 /// 后台入口这一次为什么没起跑。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum BackgroundCompactionNotStarted {
-    /// 整理因一批出错停着（D26（后台整理与放置回收） 已定项 13 第 4 条）：「该跑了」留着，解除之后下一次调用再取。
+    /// 整理因一批出错停着（D26（后台整理与放置回收） 已定项 13 第 4 条）而池子每块盘的已分配占它的容量都不超过补跑阈值（用户 2026-10-28 定案）：
+    /// 「该跑了」留着，解除之后（或占用超过阈值时）下一次调用再取。
     BackgroundTaskStoppedAfterAComputeError,
     /// 没有「该跑了」，定时兜底关着。
     NothingDueAndTheTimerFallbackOff,
@@ -2203,6 +2211,17 @@ impl MountedSession {
             .background_compute_of_this_mount()
     }
 
+    /// GC 停着时在 CPU 上补跑那一判此刻的读数（用户 2026-10-28 定案，同日弹窗定「任一块盘超过就补跑」）：池子此刻最满的那一块盘、它的已分配字节与容量
+    /// （分配器增量维护的计数，[`AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator`]），与下一批读的那一组的补跑阈值。只读。
+    #[must_use]
+    pub fn gc_cpu_compensation_reading(&self) -> GcCpuCompensationReading {
+        self.background_compute().gc_cpu_compensation_reading(
+            AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator(
+                &self.allocator,
+            ),
+        )
+    }
+
     /// 挂着切换后台计算的设置（D26（后台整理与放置回收） 已定项 13 第 1 条：一次切全部后台任务，从下一批开始生效、一批之内不换）：
     /// 与挂载同一套校验（第 4 条，`background_compute::admit_background_compute_settings_with_the_gpu_compute`；`gpu_compute` 是这一次递进来的 GPU 实现，
     /// `None` 是沿用这次挂载已有的那一份，都没有而切到 `gpu` 就是 GPU 不在）；
@@ -2220,6 +2239,26 @@ impl MountedSession {
             .request_switch_from_the_next_batch(background_compute_settings, gpu_compute)
     }
 
+    /// 同 [`Self::switch_background_compute_from_the_next_batch`]，连同 GC 停着时在 CPU 上补跑的阈值一起换（挂载参数
+    /// `background_gc_cpu_compensation_threshold`，用户 2026-10-28 定案；不给阈值的那一个切换沿用原来的阈值）。阈值超过 100% 也拒，停着的照旧停着。
+    ///
+    /// # Errors
+    /// 校验不过：拒绝切换，生效的与待生效的设置都留着原样。
+    pub fn switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
+        &mut self,
+        background_compute_settings: BackgroundComputeSettings,
+        gpu_compute: Option<BackgroundGpuComputeHandle>,
+        gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
+    ) -> Result<(), BackgroundComputeSettingsRefused> {
+        self.parameters_and_device_table
+            .background_compute_of_this_mount_mut()
+            .request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
+                background_compute_settings,
+                gpu_compute,
+                gc_cpu_compensation_threshold,
+            )
+    }
+
     /// 显式入口的三种起法共用的那一段：盘表核过、现行那一版有树，就推一趟、发 discard。
     fn run_compaction_with_the_region_check_and_a_hook_in_the_middle_of_each_batch<
         Device: BlockDevice,
@@ -2276,7 +2315,8 @@ impl MountedSession {
 
     /// 整理的后台入口（D26（后台整理与放置回收） 已定项 13 第 5、6 条；用户 2026-10-26 定 GC「挪出调用线程，补齐三样保证」）：调用方在它自己的线程上
     /// 一次次调，核心层不起线程、不读钟（钟与前台的 fsync 由 `caller` 答）。每次调用先看起不起跑，依次：
-    /// 1. 整理因一批出错停着（第 4 条）⇒ 不起跑，「该跑了」留着；
+    /// 1. 整理因一批出错停着（第 4 条）而池子每块盘的已分配占它的容量都不超过补跑阈值 ⇒ 不起跑，「该跑了」留着，活意图的 R 排他撤掉（同出错停下那一刻）；
+    ///    超过阈值就照下面几条照常判起不起跑，起跑之后每一批在 CPU 上补跑（用户 2026-10-28 定案）；
     /// 2. 用户改动之后起跑判据记下了「该跑了」⇒ 取走，照用户改动之后那一趟推（判决 G8 丁：至多一批）；
     /// 3. 没有 ⇒ 看定时兜底（`pacing.timer_fallback`）：关着、或离上一次查还不到一个间隔就不起跑；到了就再查一次起跑判据，为真就推一趟到停；
     /// 4. 用户改动之后那一趟与定时兜底那一趟都从上一次用户改动给的那一份 `B_gc` 里扣（岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲，
@@ -2345,8 +2385,7 @@ impl MountedSession {
         }
         let parameters = parameters_and_device_table.parameters_on_disk();
         let background_compute = parameters_and_device_table.background_compute_of_this_mount();
-        if background_compute.is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation)
-        {
+        if compaction_stays_stopped_pausing_the_live_intent(allocator, background_compute) {
             return Ok(BackgroundCompactionStart::NotStarted(
                 BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
             ));
@@ -2877,6 +2916,11 @@ impl MountedSession {
 /// 抹孤儿头归实例表清扫（`instance_table_sweep`），清扫判过、抹过之前段里的孤儿头要原样留着。孤儿头 = 段里某个槽开头认得出本池的单元头
 /// （清扫认头的那一判，`instance_table_sweep::unit_head_read_by_sweep`），它的写序实例在现行那一版的实例表里还有一行、按那一行判未发布
 /// （`UnitHeadReadBySweep::is_unpublished_by_the_row`）。判之前读不出实例表、读不出段里的一个槽，这一段同样不发（读不到不等于没有）。
+///
+/// 分配器上冻结着一次没重发的发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）时第一道就交回、一段都不发，待 discard 表不取、原样留着
+/// （C928（冻结着时整理照发 discard），判决 `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R3 K3-1 两个候选里取「discard 开头判冻结」）：
+/// 冻结时分配器退回发布之前那一份，退回的那一份里整段可再分配的段可能正装着冻结那次已落盘的单元。判在这里而不在整理入口：发 discard 的三处
+/// （用户改动发成之后、整理显式入口、后台入口）一处罩住，整理入口的错误成员表不动；整理那一趟自己的发布本来就在发布路径第一道被冻结拒。
 pub fn discard_the_segments_reusable_as_a_whole<Device: BlockDevice>(
     devices: &mut [(DeviceIdentity, Device)],
     allocator: &mut PoolAllocator,
@@ -2884,6 +2928,11 @@ pub fn discard_the_segments_reusable_as_a_whole<Device: BlockDevice>(
     current_root: &RootRecord,
 ) -> DiscardsAfterThePublishes {
     let mut discards = DiscardsAfterThePublishes::default();
+    if let Some(frozen) = allocator.frozen_publish() {
+        discards.txg_of_the_frozen_publish_withholding_every_discard =
+            Some(frozen.checkpoint_txg());
+        return discards;
+    }
     let segments_to_discard = allocator.take_the_segments_to_discard_on_every_device();
     if segments_to_discard
         .iter()
@@ -3037,9 +3086,30 @@ enum CompactionStep {
 }
 
 /// 整理推一次发布（后台任务的一批）交回的两样：这一步的结局，与这一批的后台计算记账（请求的后端、实际走的路）。
+/// 整理停着、占用不超过补跑阈值而这一批没开成时没有记账（`None`），结局是 [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]。
 struct CompactionPublishAdvanced {
     step: Result<CompactionStep, RelocationBatchRefused>,
-    background_compute_of_the_batch: BackgroundComputeOfOneBatch,
+    background_compute_of_the_batch: Option<BackgroundComputeOfOneBatch>,
+}
+
+/// 整理此刻停着而不补跑（用户 2026-10-28 定案：GC 因一批出错停着、池子每块盘的已分配占它的容量都不超过阈值）：是就撤掉活意图的 R 排他、意图留着
+/// （同一批出错停下那一刻的暂停，停着的时候 R 不压着分配），交回真；没停着或要在 CPU 上补跑交回假，什么都不动。判的是这一刻分配器的计数，
+/// 不读写盘。
+fn compaction_stays_stopped_pausing_the_live_intent(
+    allocator: &mut PoolAllocator,
+    background_compute: &BackgroundComputeOfThisMount,
+) -> bool {
+    match background_compute.whether_the_task_runs_the_next_batch(
+        BackgroundTask::CompactionRelocation,
+        AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator(allocator),
+    ) {
+        WhetherTheTaskRunsTheNextBatch::StaysStopped(_) => {
+            allocator.compaction_mut().pause_the_live_intent();
+            true
+        }
+        WhetherTheTaskRunsTheNextBatch::AsTheSettingsSay
+        | WhetherTheTaskRunsTheNextBatch::OnTheCpuToCompensateWhileStopped(_) => false,
+    }
 }
 
 /// 整理一趟（显式入口、可写挂载续做共用；后台入口另经 [`run_compaction_on_with_the_region_check_of_the_intent_deletion`]）：先走空删除续做（G8 次序），
@@ -3079,8 +3149,8 @@ pub(crate) fn run_compaction_on<Device: BlockDevice>(
 /// 写带宽到了上限就不发下一批，D26（后台整理与放置回收） 已定项 13 第 6 条）；一批腾空了 R、照同一批删意图再发时删意图那一支的核照 `region_check` 判
 /// （[`MountedSession::run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region`]），
 /// 每一批读完后台计算设置之后照 `in_the_middle_of_each_batch` 做（产品路径什么都不做，
-/// [`MountedSession::run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch`]）。整理因一批出错停着时一个写都不发
-/// （[`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]）。
+/// [`MountedSession::run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch`]）。整理因一批出错停着而每块盘都不超过补跑阈值时，
+/// 从判出的那一刻起一个写都不发（[`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]）。
 #[allow(
     clippy::too_many_arguments,
     reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、推到哪为止、预算按哪一条算、每批之前看不看调用方、删意图那一支的核怎么判、这次挂载的后台计算状态、一批读完设置之后做什么"
@@ -3132,8 +3202,9 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
         CompactionRunForcing::FollowTheWatermarks | CompactionRunForcing::ForcedToStartForTests => {
         }
     }
-    // D26（后台整理与放置回收） 已定项 13 第 4 条：整理因一批出错停着，等一次通过校验的切换请求解除；停着时一个写都不发（删除续做也不走）。
-    if background_compute.is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation) {
+    // D26（后台整理与放置回收） 已定项 13 第 4 条：整理因一批出错停着，等一次通过校验的切换请求解除；停着而池子每块盘的已分配占它的容量都不超过补跑阈值时
+    // 一个写都不发（删除续做也不走），超过就照常推、每一批在 CPU 上补跑（用户 2026-10-28 定案）。
+    if compaction_stays_stopped_pausing_the_live_intent(allocator, background_compute) {
         return finish(
             compaction,
             versions_published,
@@ -3275,7 +3346,7 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
             background_compute,
             in_the_middle_of_each_batch,
         );
-        background_compute_of_each_batch.push(background_compute_of_the_batch);
+        background_compute_of_each_batch.extend(background_compute_of_the_batch);
         match step {
             Ok(CompactionStep::Published { version, kind }) => {
                 publishes_of_the_run += 1;
@@ -3550,6 +3621,9 @@ fn compaction_publish_refused_short_of_space(refused: &RelocationBatchRefused) -
 /// 读完设置之后、开算之前照 `in_the_middle_of_each_batch` 做（只供测试的钩子在这一刻请求切换、或注入一次出错）。
 /// 这一批算着出了错（第 4 条：GPU 实现交回出错，或只供测试的钩子注入）：这一批放弃，一个写都不发、意图留在盘上（撤 R 排他，同暂停），
 /// 不换回 CPU；整理这个后台任务停下，出错之前就挂着待生效的切换时当场解除（`background_compute::BackgroundComputeOfThisMount::stop_the_task_after_a_compute_error`）。
+/// 整理停着时（用户 2026-10-28 定案）：进门那一刻池子最满的那一块盘的已分配占它的容量超过补跑阈值，这一批走 CPU（记账的实际走法是
+/// `ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError`，带那一判的读数）；不超过就不开这一批、一个写都不发，撤 R 排他，
+/// 停在 [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]。
 #[allow(
     clippy::too_many_arguments,
     reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、批大小、推到哪为止、一批腾空了 R 之后删意图那一支的核怎么判、这次挂载的后台计算状态、一批读完设置之后做什么"
@@ -3567,7 +3641,19 @@ fn advance_compaction_by_one_publish<Device: BlockDevice>(
     background_compute: &mut BackgroundComputeOfThisMount,
     in_the_middle_of_each_batch: &mut InTheMiddleOfEachBackgroundBatch<'_>,
 ) -> CompactionPublishAdvanced {
-    let batch_under_way = background_compute.begin_a_batch(BackgroundTask::CompactionRelocation);
+    let Ok(batch_under_way) = background_compute.begin_a_batch(
+        BackgroundTask::CompactionRelocation,
+        AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator(allocator),
+    ) else {
+        // 停着而这一批开头占用不超过补跑阈值（上一批补跑腾出了空间）：这一批不开，同出错停下那一刻撤 R 排他、意图留在盘上。
+        allocator.compaction_mut().pause_the_live_intent();
+        return CompactionPublishAdvanced {
+            step: Ok(CompactionStep::Stopped(
+                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError,
+            )),
+            background_compute_of_the_batch: None,
+        };
+    };
     in_the_middle_of_each_batch.in_the_middle_of_the_batch(background_compute);
     let gpu_compute = batch_under_way.gpu_compute_of_the_route().cloned();
     let mut gpu_work = GpuWorkOfTheBatch::default();
@@ -3626,11 +3712,11 @@ fn advance_compaction_by_one_publish<Device: BlockDevice>(
     };
     CompactionPublishAdvanced {
         step,
-        background_compute_of_the_batch: background_compute.end_the_batch(
+        background_compute_of_the_batch: Some(background_compute.end_the_batch(
             batch_under_way,
             gpu_compute.map(|_| gpu_work),
             compute_error_that_abandoned_the_batch,
-        ),
+        )),
     }
 }
 
diff --git a/crates/singlefs-core/src/root_record.rs b/crates/singlefs-core/src/root_record.rs
index 687730635..f8f0432ee 100644
--- a/crates/singlefs-core/src/root_record.rs
+++ b/crates/singlefs-core/src/root_record.rs
@@ -1,22 +1,41 @@
 //! 根记录（D22（单元原子性怎么合成） 已定项 7）：466 字节住一个判定宽度（physical_block_size）的根槽，
 //! 自证校验和罩整槽含补齐、自身按 0 参与。行序即盘上顺序，偏移按行累加。
+//!
+//! 整份在 `verus!` 里，由 Verus 直接验（里程碑四出口 5「core 直验」编解码与记录格式这一簇）：[`RootRecord::to_slot`] 交出的就是
+//! [`RootRecord::spec_slot_bytes`]，[`RootRecord::parse_slot`] 交出的就是 [`RootRecord::spec_parse_slot`]，往返引理
+//! [`RootRecord::lemma_parse_slot_round_trip`] 证写出来的槽读回同一条根记录。
 
 use singlefs_format::{
     NODE_POINTER_BYTES, ROOT_RECORD_BYTES, ROOT_RECORD_NO_RING_SLOT_PINNED, WIDE_CHECKSUM_BYTES,
 };
+use vstd::prelude::*;
 
 use crate::address::{CheckpointTxg, InstanceGeneration};
-use crate::bytes::{ByteReader, ByteWriter};
-use crate::checksum::{wide_checksum_field_holds, wide_checksum_with_field_zeroed};
-use crate::pointer::NodePointer;
+// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
+use crate::bytes::*;
+// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
+use crate::checksum::*;
+// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
+use crate::pointer::*;
 
-pub const ROOT_MAGIC: [u8; 4] = *b"SFSR";
+verus! {
+
+broadcast use group_little_endian_round_trips;
+
+// ===== 这一份文件用到的 singlefs-format 常量的规格（写法同 unit.rs 头上那一段）=====
+#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
+pub assume_specification[ singlefs_format::ROOT_RECORD_BYTES ] -> (root_record_bytes: u64) returns 466u64;
+#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
+pub assume_specification[ singlefs_format::ROOT_RECORD_NO_RING_SLOT_PINNED ] -> (no_slot_pinned: u8) returns 255u8;
+
+/// 根记录的 magic「SFSR」（写成四个字节值：Verus 不认字节串字面量）。
+pub const ROOT_MAGIC: [u8; 4] = [0x53, 0x46, 0x53, 0x52];
 /// magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8 + 树表指针 86 + 树 ID 水位 8 + 回退下界 F 8 = 138。
 pub const ROOT_CHECKSUM_OFFSET: usize = 4 + 16 + 4 + 4 + 8 + 86 + 8 + 8;
 /// flags 位 0 = 卸载记号：正常卸载那一串写的根置 1，其余根写 0（D22（单元原子性怎么合成） 已定项 7，用户 2026-09-26 定）；
 /// 位 n 住第 n div 8 个字节的第 n mod 8 低位（已定项 13）。其余位第一版恒 0、非 0 拒收（已定项 17）。
 /// 内存里是 [`RootRecord::unmount_marker`]（[`UnmountMarker`]），只有 `mount::unmount` 那一串写 1。
-pub const ROOT_RECORD_FLAG_UNMOUNT_MARKER: u32 = 1 << 0;
+pub const ROOT_RECORD_FLAG_UNMOUNT_MARKER: u32 = 1;
 
 /// 根记录 flags 位 0 的卸载记号（D22（单元原子性怎么合成） 已定项 7）：这条根是不是正常卸载那一串空发布写的
 /// （D16（发布语义） 已定项 1「正常卸载」）。池级 checker 按它给 I-7.9（回退下界 F 不高于抬 F 的上限） 分两支判；
@@ -33,9 +52,47 @@ pub enum UnmountMarker {
 }
 
 impl UnmountMarker {
+    /// 写进 flags 那 4 字节的值（[`Self::flags_bits`] 的规格）。
+    pub open spec fn spec_flags_bits(self) -> u32 {
+        match self {
+            UnmountMarker::WrittenByTheUnmountSequence => ROOT_RECORD_FLAG_UNMOUNT_MARKER,
+            UnmountMarker::NotWrittenByTheUnmountSequence => 0,
+        }
+    }
+
+    /// 只带记号那一位的 flags 读回的记号（[`Self::of_flags_bits_carrying_only_the_marker`] 的规格）。
+    pub open spec fn spec_of_flags_bits_carrying_only_the_marker(flags: u32) -> Self {
+        if flags & ROOT_RECORD_FLAG_UNMOUNT_MARKER == 0 {
+            UnmountMarker::NotWrittenByTheUnmountSequence
+        } else {
+            UnmountMarker::WrittenByTheUnmountSequence
+        }
+    }
+
+    /// 记号那 4 字节读回的记号：位 0 之外有位为 1 ⇒ `None`（[`Self::of_unmount_marker_field_of_a_new_root_segment`] 的规格）。
+    pub open spec fn spec_of_unmount_marker_field(field: u32) -> Option<Self> {
+        if field & !ROOT_RECORD_FLAG_UNMOUNT_MARKER != 0 {
+            None
+        } else {
+            Some(Self::spec_of_flags_bits_carrying_only_the_marker(field))
+        }
+    }
+
+    /// 往返：写进 flags 的值读回同一个记号。
+    pub proof fn lemma_flags_bits_round_trip(self)
+        ensures
+            Self::spec_of_unmount_marker_field(self.spec_flags_bits()) == Some(self),
+    {
+        assert(1u32 & !1u32 == 0u32 && 1u32 & 1u32 != 0u32 && 0u32 & !1u32 == 0u32 && 0u32 & 1u32 == 0u32)
+            by (bit_vector);
+    }
+
     /// 写进 flags 那 4 字节的值：记号那一位，其余位恒 0。
     #[must_use]
-    pub const fn flags_bits(self) -> u32 {
+    pub const fn flags_bits(self) -> (bits: u32)
+        ensures
+            bits == self.spec_flags_bits(),
+    {
         match self {
             UnmountMarker::WrittenByTheUnmountSequence => ROOT_RECORD_FLAG_UNMOUNT_MARKER,
             UnmountMarker::NotWrittenByTheUnmountSequence => 0,
@@ -45,7 +102,10 @@ impl UnmountMarker {
     /// journal 记录新根段里的卸载记号那 4 字节（D23（journal 的角色与格式） 已定项 15）：与根记录 flags 同一个编码，位 0 之外有位为 1 ⇒ `None`
     /// （读者把那条记录当损坏，与根记录 flags、记录标志「其余位读到非 0」同一个处置）。
     #[must_use]
-    pub const fn of_unmount_marker_field_of_a_new_root_segment(field: u32) -> Option<Self> {
+    pub const fn of_unmount_marker_field_of_a_new_root_segment(field: u32) -> (marker: Option<Self>)
+        ensures
+            marker == Self::spec_of_unmount_marker_field(field),
+    {
         if field & !ROOT_RECORD_FLAG_UNMOUNT_MARKER != 0 {
             return None;
         }
@@ -54,7 +114,10 @@ impl UnmountMarker {
 
     /// 从已经判过「记号之外的位全是 0」的 flags 读回记号。
     #[must_use]
-    const fn of_flags_bits_carrying_only_the_marker(flags: u32) -> Self {
+    const fn of_flags_bits_carrying_only_the_marker(flags: u32) -> (marker: Self)
+        ensures
+            marker == Self::spec_of_flags_bits_carrying_only_the_marker(flags),
+    {
         if flags & ROOT_RECORD_FLAG_UNMOUNT_MARKER == 0 {
             UnmountMarker::NotWrittenByTheUnmountSequence
         } else {
@@ -64,8 +127,10 @@ impl UnmountMarker {
 }
 
 /// 根记录与 journal 新根段里的钉槽 p（D16（发布语义） 已定项 1「根槽这一次读坏」那一行「钉也落盘」，C773（读坏与全零根槽的回收门槛钉只在内存里），
-/// 用户 2026-10-20 定）：建这一版的账时还钉着的根环槽里钉值最小的那个的钉值。钉值是那一槽可能住过的最旧 txg，也就是 `3·槽 + 区域`
-/// （0 到 3S 之内 `root_ring::target_for_publish` 是 (txg mod 3, txg div 3)，一槽一个钉值），所以这一个字节同时说了哪一槽、钉在哪个 txg；
+/// 用户 2026-10-20 定）：建这一版的账时还钉着的根环槽里按（钉值, 槽）最小的那一槽，存那一槽的编码 `3·槽 + 区域`
+/// （0 到 3S 之内与 `root_ring::target_for_publish` 落到它的最小 txg 相同，一槽一个编码）。钉值是那一槽可能住过的最旧 txg：槽号为 0 的槽
+/// （mkfs 种第 0 代根的那三槽）是第 0 代根的 txg 0，别的槽就是编码本身（判决 `research/prompts/m3-close-b19-core-r1-main-verification.md`
+/// 第二节 R1 采乙：p 照存公式槽，钉值由槽推）。三个槽 0 的钉值都是 0，p 存钉值就还原不出钉的是哪一槽，所以存槽不存钉值。
 /// 没有钉着的槽写 [`ROOT_RECORD_NO_RING_SLOT_PINNED`]。
 ///
 /// 盘上那 1 字节原样：core 的读者不判它的取值（core 只在施加记录拼根时把它原样带进拼出的根，不拿它建状态），
@@ -77,20 +142,33 @@ impl PinnedRingSlot {
     /// 建账时一个槽都没钉着；mkfs 的第 0 代也写它。
     pub const NO_SLOT_PINNED: Self = Self(ROOT_RECORD_NO_RING_SLOT_PINNED);
 
-    /// 钉着的槽里钉值最小的那个（`None` = 一个都没钉着）写成的字节。
+    /// 钉着的槽里按（钉值, 槽）最小的那一槽的编码 `3·槽 + 区域`（`None` = 一个都没钉着）写成的字节（`allocator::LowestPinnedRingSlot`）。
     ///
     /// # Panics
-    /// 钉值装不进一个字节、或撞上「没有钉着」那个值：钉值是一槽可能住过的最旧 txg，小于环长 3S（S ≤ 16，
-    /// `singlefs_format::ROOT_RING_SLOTS_PER_REGION_MAXIMUM`），装得进也撞不上；装不进说明钉值不是按那一槽可能住过的最旧 txg 算的。
+    /// 编码装不进一个字节、或撞上「没有钉着」那个值：编码小于环长 3S（S ≤ 16，`singlefs_format::ROOT_RING_SLOTS_PER_REGION_MAXIMUM`），
+    /// 装得进也撞不上；装不进说明传进来的不是槽的编码。
     #[must_use]
-    pub fn of_the_lowest_pin(lowest_pinned_txg: Option<CheckpointTxg>) -> Self {
-        match lowest_pinned_txg {
+    ///
+    /// `requires` 就是这两种 panic 的反面。
+    pub fn of_the_lowest_pin(lowest_pinned_ring_slot_encoding: Option<u64>) -> (pin: Self)
+        requires
+            lowest_pinned_ring_slot_encoding matches Some(encoding) ==> encoding < ROOT_RECORD_NO_RING_SLOT_PINNED,
+        ensures
+            match lowest_pinned_ring_slot_encoding {
+                None => pin.0 == ROOT_RECORD_NO_RING_SLOT_PINNED,
+                Some(encoding) => pin.0 == encoding,
+            },
+    {
+        match lowest_pinned_ring_slot_encoding {
             None => Self::NO_SLOT_PINNED,
-            Some(pinned_txg) => {
-                let pin = u8::try_from(pinned_txg.0)
-                    .ok()
-                    .filter(|pin| *pin != ROOT_RECORD_NO_RING_SLOT_PINNED)
-                    .expect("钉值是那一槽可能住过的最旧 txg，小于环长 3S ≤ 48：装得进一个字节、撞不上「没有钉着」的 255");
+            Some(encoding) => {
+                // Option::filter 带闭包没有 Verus 规格：装得进一个字节与撞不上 255 拆成两步判，panic 的那一句不变。
+                let pin = u8::try_from(encoding)
+                    .expect("钉着的槽的编码 3·槽 + 区域小于环长 3S ≤ 48：装得进一个字节、撞不上「没有钉着」的 255");
+                assert!(
+                    pin != ROOT_RECORD_NO_RING_SLOT_PINNED,
+                    "钉着的槽的编码 3·槽 + 区域小于环长 3S ≤ 48：装得进一个字节、撞不上「没有钉着」的 255"
+                );
                 Self(pin)
             }
         }
@@ -127,9 +205,161 @@ pub struct RootRecord {
 }
 
 impl RootRecord {
+    /// 四条指针的槽号都装得进 6 字节、位置条目的次序写者肯写：[`Self::to_slot`] 写得出它。
+    pub open spec fn is_encodable(self) -> bool {
+        &&& self.tree_table.is_encodable() && self.tree_table.has_location_entries_the_writer_accepts()
+        &&& self.instance_table.is_encodable() && self.instance_table.has_location_entries_the_writer_accepts()
+        &&& self.mapping_root.is_encodable() && self.mapping_root.has_location_entries_the_writer_accepts()
+        &&& self.allocation_record_tree_root.is_encodable()
+            && self.allocation_record_tree_root.has_location_entries_the_writer_accepts()
+    }
+
+    /// 自证校验和之前那 138 字节：magic、fsid、flags、实例代号、checkpoint_txg、树表指针、树 ID 水位、回退下界 F。
+    pub open spec fn spec_bytes_before_the_checksum(self) -> Seq<u8> {
+        ROOT_MAGIC@ + self.filesystem_identifier@ + little_endian_bytes(self.unmount_marker.spec_flags_bits() as nat, 4)
+            + little_endian_bytes(self.instance.0 as nat, 4) + little_endian_bytes(self.checkpoint_txg.0 as nat, 8)
+            + self.tree_table.spec_bytes() + little_endian_bytes(self.tree_identifier_watermark as nat, 8)
+            + little_endian_bytes(self.rollback_floor.0 as nat, 8)
+    }
+
+    /// 自证校验和之后那 296 字节：实例表、中央映射树、分配记录树三条指针，回收门槛 t、钉槽 p、算法类型 0、nonce 与 MAC 28 字节 0。
+    pub open spec fn spec_bytes_after_the_checksum(self) -> Seq<u8> {
+        self.instance_table.spec_bytes() + self.mapping_root.spec_bytes() + self.allocation_record_tree_root.spec_bytes()
+            + little_endian_bytes(self.reclaim_threshold.0 as nat, 8) + little_endian_bytes(self.pinned_ring_slot.0 as nat, 1)
+            + little_endian_bytes(0, 1) + zero_bytes(28)
+    }
+
+    /// 校验和字段还是 0 的整槽：记录 466 字节、之后补 0 到槽宽。
+    pub open spec fn spec_unsealed_slot_bytes(self, slot_bytes: nat) -> Seq<u8> {
+        self.spec_bytes_before_the_checksum() + zero_bytes(WIDE_CHECKSUM_BYTES as nat) + self.spec_bytes_after_the_checksum()
+            + zero_bytes((slot_bytes - ROOT_RECORD_BYTES) as nat)
+    }
+
+    /// [`Self::to_slot`] 写出的整槽：校验和字段换成罩整槽、自身按 0 算出来的那 32 字节。
+    pub open spec fn spec_slot_bytes(self, slot_bytes: nat) -> Seq<u8> {
+        let unsealed = self.spec_unsealed_slot_bytes(slot_bytes);
+        unsealed.subrange(0, ROOT_CHECKSUM_OFFSET as int) + spec_wide_checksum_field(
+            unsealed,
+            slot_bytes as int,
+            ROOT_CHECKSUM_OFFSET as int,
+        ) + unsealed.subrange(ROOT_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES, slot_bytes as int)
+    }
+
+    /// [`Self::parse_slot`] 的判法：magic、整槽校验和、fsid、flags、四条指针头部五关，过了才按字段表读出来。
+    pub open spec fn spec_parse_slot(bytes: Seq<u8>, expected_filesystem_identifier: [u8; 16]) -> Option<Self> {
+        let flags = little_endian_value(bytes.subrange(20, 24)) as u32;
+        if bytes.len() < ROOT_RECORD_BYTES || bytes.subrange(0, 4) != ROOT_MAGIC@ {
+            None
+        } else if bytes.subrange(ROOT_CHECKSUM_OFFSET as int, ROOT_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES)
+            != spec_wide_checksum_field(bytes, bytes.len() as int, ROOT_CHECKSUM_OFFSET as int) {
+            None
+        } else if bytes.subrange(4, 20) != expected_filesystem_identifier@ {
+            None
+        } else if flags & !ROOT_RECORD_FLAG_UNMOUNT_MARKER != 0 {
+            None
+        } else if NodePointer::spec_read_judging_at(bytes, 36) is Err
+            || NodePointer::spec_read_judging_at(bytes, 170) is Err
+            || NodePointer::spec_read_judging_at(bytes, 256) is Err
+            || NodePointer::spec_read_judging_at(bytes, 342) is Err {
+            None
+        } else {
+            Some(RootRecord {
+                filesystem_identifier: expected_filesystem_identifier,
+                unmount_marker: UnmountMarker::spec_of_flags_bits_carrying_only_the_marker(flags),
+                instance: InstanceGeneration(little_endian_value(bytes.subrange(24, 28)) as u32),
+                checkpoint_txg: CheckpointTxg(little_endian_value(bytes.subrange(28, 36)) as u64),
+                tree_table: NodePointer::spec_decode_at(bytes, 36),
+                tree_identifier_watermark: little_endian_value(bytes.subrange(122, 130)) as u64,
+                rollback_floor: CheckpointTxg(little_endian_value(bytes.subrange(130, 138)) as u64),
+                instance_table: NodePointer::spec_decode_at(bytes, 170),
+                mapping_root: NodePointer::spec_decode_at(bytes, 256),
+                allocation_record_tree_root: NodePointer::spec_decode_at(bytes, 342),
+                reclaim_threshold: CheckpointTxg(little_endian_value(bytes.subrange(428, 436)) as u64),
+                pinned_ring_slot: PinnedRingSlot(little_endian_value(bytes.subrange(436, 437)) as u8),
+            })
+        }
+    }
+
+    /// 往返：写出的槽带着这条根记录的 fsid 读回同一条根记录。
+    pub proof fn lemma_parse_slot_round_trip(self, slot_bytes: nat)
+        requires
+            self.is_encodable(),
+            slot_bytes >= ROOT_RECORD_BYTES,
+        ensures
+            Self::spec_parse_slot(self.spec_slot_bytes(slot_bytes), self.filesystem_identifier) == Some(self),
+    {
+        lemma_little_endian_capacities();
+        self.unmount_marker.lemma_flags_bits_round_trip();
+        let unsealed = self.spec_unsealed_slot_bytes(slot_bytes);
+        let sealed = self.spec_slot_bytes(slot_bytes);
+        let before = self.spec_bytes_before_the_checksum();
+        let after = self.spec_bytes_after_the_checksum();
+        assert(before.len() == 138);
+        assert(after.len() == 296);
+        assert(unsealed.len() == slot_bytes);
+        let digest = spec_wide_checksum_field(unsealed, slot_bytes as int, 138);
+        assert(digest.len() == 32);
+        assert(sealed.len() == slot_bytes);
+        // 校验和字段之外两截与没封的那一份逐字节相同，所以罩住的那一截（字段按 0）两份相同、算出来的校验和相同。
+        assert(sealed.subrange(0, 138) =~= before);
+        assert(sealed.subrange(138, 170) =~= digest);
+        assert(sealed.subrange(170, 466) =~= after);
+        assert(spec_covered_with_the_field_zeroed(sealed, slot_bytes as int, 138)
+            =~= spec_covered_with_the_field_zeroed(unsealed, slot_bytes as int, 138));
+        // 前 138 字节逐段剥开。
+        let magic = ROOT_MAGIC@;
+        let fsid = self.filesystem_identifier@;
+        let flags = little_endian_bytes(self.unmount_marker.spec_flags_bits() as nat, 4);
+        let instance = little_endian_bytes(self.instance.0 as nat, 4);
+        let checkpoint = little_endian_bytes(self.checkpoint_txg.0 as nat, 8);
+        let tree_table = self.tree_table.spec_bytes();
+        let watermark = little_endian_bytes(self.tree_identifier_watermark as nat, 8);
+        let rollback = little_endian_bytes(self.rollback_floor.0 as nat, 8);
+        lemma_split_a_concatenated_range(sealed, 0, magic + fsid + flags + instance + checkpoint + tree_table + watermark, rollback);
+        lemma_split_a_concatenated_range(sealed, 0, magic + fsid + flags + instance + checkpoint + tree_table, watermark);
+        lemma_split_a_concatenated_range(sealed, 0, magic + fsid + flags + instance + checkpoint, tree_table);
+        lemma_split_a_concatenated_range(sealed, 0, magic + fsid + flags + instance, checkpoint);
+        lemma_split_a_concatenated_range(sealed, 0, magic + fsid + flags, instance);
+        lemma_split_a_concatenated_range(sealed, 0, magic + fsid, flags);
+        lemma_split_a_concatenated_range(sealed, 0, magic, fsid);
+        // 后 296 字节逐段剥开。
+        let instance_table = self.instance_table.spec_bytes();
+        let mapping_root = self.mapping_root.spec_bytes();
+        let allocation_root = self.allocation_record_tree_root.spec_bytes();
+        let reclaim = little_endian_bytes(self.reclaim_threshold.0 as nat, 8);
+        let pinned = little_endian_bytes(self.pinned_ring_slot.0 as nat, 1);
+        let algorithm_type = little_endian_bytes(0, 1);
+        lemma_split_a_concatenated_range(
+            sealed,
+            170,
+            instance_table + mapping_root + allocation_root + reclaim + pinned + algorithm_type,
+            zero_bytes(28),
+        );
+        lemma_split_a_concatenated_range(sealed, 170, instance_table + mapping_root + allocation_root + reclaim + pinned, algorithm_type);
+        lemma_split_a_concatenated_range(sealed, 170, instance_table + mapping_root + allocation_root + reclaim, pinned);
+        lemma_split_a_concatenated_range(sealed, 170, instance_table + mapping_root + allocation_root, reclaim);
+        lemma_split_a_concatenated_range(sealed, 170, instance_table + mapping_root, allocation_root);
+        lemma_split_a_concatenated_range(sealed, 170, instance_table, mapping_root);
+        self.tree_table.lemma_decode_at_round_trip(sealed, 36);
+        self.instance_table.lemma_decode_at_round_trip(sealed, 170);
+        self.mapping_root.lemma_decode_at_round_trip(sealed, 256);
+        self.allocation_record_tree_root.lemma_decode_at_round_trip(sealed, 342);
+        assert(Self::spec_parse_slot(sealed, self.filesystem_identifier).unwrap().filesystem_identifier
+            == self.filesystem_identifier);
+    }
+
     /// 写成一个 `slot_bytes` 宽的槽；记录 466 字节，其余补 0。
+    ///
+    /// # Panics
+    /// 槽宽装不下 466 字节，或四条指针有一条写者不肯写（`requires` 排除的两种）。
     #[must_use]
-    pub fn to_slot(&self, slot_bytes: usize) -> Vec<u8> {
+    pub fn to_slot(&self, slot_bytes: usize) -> (bytes: Vec<u8>)
+        requires
+            slot_bytes >= ROOT_RECORD_BYTES,
+            self.is_encodable(),
+        ensures
+            bytes@ == self.spec_slot_bytes(slot_bytes as nat),
+    {
         assert!(
             slot_bytes >= usize::try_from(ROOT_RECORD_BYTES).expect("466"),
             "根槽装不下根记录"
@@ -158,8 +388,10 @@ impl RootRecord {
         writer.skip(12 + 16); // nonce、MAC：第一版留位全 0
         writer.assert_position(ROOT_RECORD_BYTES, "根记录");
         let mut bytes = writer.into_bytes();
+        assert(bytes@ =~= self.spec_unsealed_slot_bytes(slot_bytes as nat));
         let digest = wide_checksum_with_field_zeroed(&bytes, slot_bytes, ROOT_CHECKSUM_OFFSET);
-        bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
+        copy_bytes_into(&mut bytes, ROOT_CHECKSUM_OFFSET, &digest);
+        assert(bytes@ =~= self.spec_slot_bytes(slot_bytes as nat));
         bytes
     }
 
@@ -171,20 +403,33 @@ impl RootRecord {
     /// 读者遇到非 0 一律判该结构损坏」，D19（块指针的结构与宽度预算） 已定项 3，C581（extent 偏移非 0 没有判据），
     /// `pointer::PointerHeadFieldOutsideTheFirstVersion`）：拒收的根与自证不过的根同一个处置，择根时不可择。
     #[must_use]
-    pub fn parse_slot(bytes: &[u8], expected_filesystem_identifier: &[u8; 16]) -> Option<Self> {
+    pub fn parse_slot(bytes: &[u8], expected_filesystem_identifier: &[u8; 16]) -> (root: Option<Self>)
+        ensures
+            root == Self::spec_parse_slot(bytes@, *expected_filesystem_identifier),
+    {
         if bytes.len() < usize::try_from(ROOT_RECORD_BYTES).expect("466")
             || bytes[..4] != ROOT_MAGIC
         {
+            proof {
+                if bytes@.len() >= 466 {
+                    assert(bytes@.subrange(0, 4) != ROOT_MAGIC@);
+                }
+            }
             return None;
         }
+        assert(bytes@.subrange(0, 4) =~= ROOT_MAGIC@);
         if !wide_checksum_field_holds(bytes, bytes.len(), ROOT_CHECKSUM_OFFSET) {
             return None;
         }
         let mut reader = ByteReader::at(bytes, 4);
-        let filesystem_identifier: [u8; 16] = reader.take(16).try_into().expect("切了 16 字节");
-        if &filesystem_identifier != expected_filesystem_identifier {
+        let filesystem_identifier: [u8; 16] = copied_into_an_array(reader.take(16));
+        if filesystem_identifier != *expected_filesystem_identifier {
+            proof {
+                assert(bytes@.subrange(4, 20) != expected_filesystem_identifier@);
+            }
             return None;
         }
+        assert(bytes@.subrange(4, 20) =~= expected_filesystem_identifier@);
         let flags = reader.get_u32();
         if flags & !ROOT_RECORD_FLAG_UNMOUNT_MARKER != 0 {
             return None;
@@ -192,37 +437,47 @@ impl RootRecord {
         let unmount_marker = UnmountMarker::of_flags_bits_carrying_only_the_marker(flags);
         let instance = InstanceGeneration(reader.get_u32());
         let checkpoint_txg = CheckpointTxg(reader.get_u64());
+        // Result::ok 接 ? 写成 match：拒收的那一种与校验和不过同一个处置（交回 None）。
         let tree_table =
-            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+            match NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                 &mut reader,
-            )
-            .ok()?;
+            ) {
+                Ok(pointer) => pointer,
+                Err(_refusal) => return None,
+            };
         let tree_identifier_watermark = reader.get_u64();
         let rollback_floor = CheckpointTxg(reader.get_u64());
         reader.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
         let instance_table =
-            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+            match NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                 &mut reader,
-            )
-            .ok()?;
+            ) {
+                Ok(pointer) => pointer,
+                Err(_refusal) => return None,
+            };
         let mapping_root =
-            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+            match NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                 &mut reader,
-            )
-            .ok()?;
+            ) {
+                Ok(pointer) => pointer,
+                Err(_refusal) => return None,
+            };
         let allocation_record_tree_root =
-            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+            match NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                 &mut reader,
-            )
-            .ok()?;
-        assert_eq!(
-            reader.position(),
-            ROOT_CHECKSUM_OFFSET + 32 + 3 * usize::try_from(NODE_POINTER_BYTES).expect("86")
+            ) {
+                Ok(pointer) => pointer,
+                Err(_refusal) => return None,
+            };
+        assert!(
+            reader.position()
+                == ROOT_CHECKSUM_OFFSET + 32 + 3 * usize::try_from(NODE_POINTER_BYTES).expect("86"),
+            "根记录读到回收门槛 t 之前停在偏移 428"
         );
         let reclaim_threshold = CheckpointTxg(reader.get_u64());
         let pinned_ring_slot = PinnedRingSlot(reader.get_u8());
         Some(Self {
-            filesystem_identifier,
+            filesystem_identifier: *expected_filesystem_identifier,
             unmount_marker,
             instance,
             checkpoint_txg,
@@ -238,6 +493,8 @@ impl RootRecord {
     }
 }
 
+} // verus!
+
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -306,14 +563,19 @@ mod tests {
         assert_eq!(sample().to_slot(512)[436], 255, "没有钉着的槽写 255");
     }
 
-    /// 钉值写成钉槽字节：没有钉着写 255，钉在 txg 6 的（区域 0 槽 2）写 6。
+    /// 钉着的槽的编码写成钉槽字节：没有钉着写 255，区域 0 槽 2 写 6，区域 1 槽 0（钉值是第 0 代根的 0）写 1。
     #[test]
-    fn the_pinned_ring_slot_byte_is_the_lowest_pinned_txg_or_255_when_nothing_is_pinned() {
+    fn the_pinned_ring_slot_byte_is_the_encoding_of_the_pinned_slot_or_255_when_nothing_is_pinned()
+    {
         assert_eq!(PinnedRingSlot::of_the_lowest_pin(None), PinnedRingSlot(255));
         assert_eq!(
-            PinnedRingSlot::of_the_lowest_pin(Some(CheckpointTxg(6))),
+            PinnedRingSlot::of_the_lowest_pin(Some(6)),
             PinnedRingSlot(6)
         );
+        assert_eq!(
+            PinnedRingSlot::of_the_lowest_pin(Some(1)),
+            PinnedRingSlot(1)
+        );
     }
 
     /// 把槽里 flags 那 4 字节（偏移 20）改成 `flags`，自证校验和重封：magic、校验和、fsid 三关照样过，拒不拒只看 flags 判。
```
