# 删除批第 16 批合入的代码：代码轮 diff（2026-10-04）

两份补丁原样（实现员 P、Q），加两份变异表追加行。补丁的底是派发那一刻的主仓 crates/（根环第 9 批已合入）；冻结副本 /tmp/claude-1000/m3-batch16-code-r1/tree/ 是两份都打上、根环改钉与 9-9 两份也合入之后的主工作区。

## 1、实现员 P（形 R 甲、GA 文档理由、没有有效根那一支、被抛弃根的过滤、GC 活文件不空）

````diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 7bae1a9..13e1d6a 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -1737,7 +1737,8 @@ fn unreadable_ring_slots_inside_the_window(
         .collect()
 }
 
-/// 重建分配器：从上一版的分配记录重建，按可再分配谓词的门槛回收，再把只被被抛弃根引用的槽隔离（影子账：被抛弃的根按最新根指着的
+/// 重建分配器：从上一版的分配记录重建，按可再分配谓词的门槛回收（当场可用的只到每块盘上最新有效根那一截，其余扣到这次挂载的暖机
+/// 落满每块盘，甲），再把只被被抛弃根引用的槽隔离（影子账：被抛弃的根按最新根指着的
 /// 实例表判，D23（journal 的角色与格式） 已定项 14 射程「影子账与按实例表判抛弃留着，理由是崩溃恢复」）。
 /// 影子账只住内存，所以每次挂载都要重算（步 4 / 步 5 代码三方第一轮云端攻方腿打中：普通重开一次隔离就归零）。
 /// 每块盘的空闲图从择到的系统配置按环长现算的单元区起点起（与偏移 417 那 8 字节相等，择的时候判过；C475（非默认环长下单元区起点取编译期常量））。
@@ -1853,9 +1854,36 @@ fn rebuilt_allocator<Device: BlockDevice>(
                 .map(|unreadable| unreadable.lowest_txg_it_can_hold),
         )
         .min();
+    // 卸载半途崩溃之后的下一次可写挂载只放开一截（D16（发布语义） 已定项 1「正常卸载」那一行，用户 2026-10-04 选甲，
+    // `research/prompts/m3-batch14-code-r1-forks.md` 第 1 行）：卸载那一串先写了系统配置里的新 F、带新 F 的根没落满每块盘就崩时，
+    // F_生效 高于某块盘上最新的有效根；恢复择根不看 F，丢掉另一块盘之后择的就是那块盘上最新的根，它引用的单元释放代可以高于它、不高于 F_生效
+    // （判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「形 R」那一行）。于是当场可用的只到
+    // reclaim_floor(min(F_生效, m), 一圈窗口里最旧有效根)，m 是每块盘上最新有效根的 txg 取最小（被抛弃的根不算，与卸载那一格同一处定义
+    // [`newest_valid_root_txg_on_every_device`]）；其余到原门槛为止的那一截扣住，这次挂载的暖机在每块盘上都落了根之后放开（`establish_instance`）。
+    // 某块盘一条有效根都读不出（m 取不到）时照原门槛当场放开。
+    let device_of_txg = |txg: CheckpointTxg| {
+        let target = target_for_publish(txg, slots_per_region);
+        system_configuration.immutable.region_devices
+            [usize::try_from(target.region).expect("区域号")]
+    };
+    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
+    let floor_of_the_reclaim = reclaim_floor(effective_floor, oldest_valid_root_in_the_window);
+    let floor_released_immediately = match newest_valid_root_txg_on_every_device(
+        &roots,
+        &newest_table,
+        &all_devices,
+        &device_of_txg,
+    ) {
+        Some(newest_valid_root_on_every_device) => reclaim_floor(
+            effective_floor.min(newest_valid_root_on_every_device),
+            oldest_valid_root_in_the_window,
+        ),
+        None => floor_of_the_reclaim,
+    };
+    allocator.reclaim_released_up_to(floor_released_immediately, ReclaimedReuse::Immediately);
     allocator.reclaim_released_up_to(
-        reclaim_floor(effective_floor, oldest_valid_root_in_the_window),
-        ReclaimedReuse::Immediately,
+        floor_of_the_reclaim,
+        ReclaimedReuse::HeldUntilFloorTakesEffect,
     );
     let shadow_ledger_computed = match shadow_ledger {
         ShadowLedger::On => isolate_slots_referenced_only_by_abandoned_roots(
@@ -2436,11 +2464,13 @@ enum ReuseOfTheSlotsReclaimedByTheSequence {
     ImmediatelyBelowTheNewestValidRootOnEveryDevice,
 }
 
-/// 每块盘上最新的有效根的 txg 取最小（GA，[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]）：`roots` 是抬 F 那一串这一遍从根环读出来的根，
-/// 一条根落在哪块盘由根环落点公式定（`device_of_txg`）。
-/// - 被抛弃的根（按现行那一版的实例表，`abandoned_by_table`）不算：回退之后新实例的 txg 大于环里每一条根（D23（journal 的角色与格式）
-///   已定项 14 第 3 条），被抛弃的根都早于新实例的第一条根；一个槽若在回退之后才释放，释放代就高于每一条被抛弃的根，放不放开只由它那块盘上
-///   有效的根定；若在回退目标那一版之前就释放了，被抛弃的那条时间线接着那一版走，也不引用它。
+/// 每块盘上最新的有效根的 txg 取最小（GA，[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]；可写挂载重建分配器时当场放开到哪儿也取它，
+/// 甲，`rebuilt_allocator`）：`roots` 是调用方这一遍从根环读出来的根，`table` 是判被抛弃用的那张实例表（卸载那一格是现行那一版的，
+/// 重建那一格是最新那条根指着的），一条根落在哪块盘由根环落点公式定（`device_of_txg`）。
+/// - 被抛弃的根（按 `table`，`abandoned_by_table`）不算。卸载那一格滤不滤结局相同：卸载接在一次可写挂载之后，那次挂载的暖机先在每块盘上
+///   落了新实例的根（D16（发布语义） 已定项 8 戊），新实例的 txg 大于环里每一条根（D23（journal 的角色与格式） 已定项 14 第 3 条），
+///   卸载时各盘最新的根都不是被抛弃的（判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「单盘恢复择中被抛弃的根」那一行，
+///   一次抽样 9027 个判定择中 0 次）。
 /// - 这一遍读不出、自证不过的槽当没有根（与这一遍回收门槛同一个读数）：恢复同样择不到它；它日后又读得出的话那条根只会更新或更旧——更新的引用的已释放槽更少，
 ///   更旧的轮不到被择。读不出让最小值只会更小，当场放开的只会更少。
 /// - 有一块盘上一条有效根都没读出来：交回 `None`，这一串回收的槽整截扣到生效（不按 txg 0 去比，说清是哪一格）。
@@ -4711,6 +4741,13 @@ fn establish_instance<Device: BlockDevice>(
         warm_up_publishes.push(next.clone());
         current = next;
     }
+    // 暖机走完、本实例的根在每块盘上都落了（`warm_up_publish_txgs` 按根环落点公式推到覆盖每块盘为止），重建分配器时扣住的那一截放开
+    // （`rebuilt_allocator`，用户 2026-10-04 选甲）：此后丢哪块盘，恢复择到的都不低于本实例的根，它们不引用这一截。
+    // 放开的只有那一截：扣住位只有两处会置——重建分配器那一处与抬 F 那一串（`raise_rollback_floor`），这次挂载的抬 F 都排在这一句之后
+    // （写行之后的准入 `push_floor_raises_after_the_row_publish`、续做里推的那几串），它们置的扣住位归它们自己放（`raise_rollback_floor`）；
+    // 走到这里分配器上的扣住位全是重建那一处置的。暖机没走完（取号之前被拒、写行或暖机落盘途中报错）时挂载整次报错，分配器随错丢掉，
+    // 扣住的那一截不交给任何会话。
+    allocator.release_reclaim_holds();
     // 取号之前在拷贝上取的落点就是真发起来取的：同一个分配器、同一串分配动作。这一串里有一份换下的单元读盘核校验和对不上时不比——
     // 真发时那一份的分配记录留在已分配，拷贝上没做那一步、照常释放，那一槽若在这一串里被回收（崩溃恢复落到环里最旧的那条根、其余全被抛弃），两边可以不同
     // （`dry_run_of_the_publishes_after_acquisition` 的文档注释）；那一格取号之前判的与真发的不是同一串，是这一版留着的缺口。
@@ -6746,3 +6783,73 @@ mod unreadable_ring_slots_inside_the_window_tests {
         );
     }
 }
+
+#[cfg(test)]
+mod newest_valid_root_on_every_device_tests {
+    use super::newest_valid_root_txg_on_every_device;
+    use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+    use crate::instance_table::{InstanceRow, InstanceTableRecords};
+    use crate::pointer::NodePointer;
+    use crate::root_record::{RootRecord, UnmountMarker};
+
+    fn root_of(instance: u32, checkpoint_txg: u64) -> RootRecord {
+        RootRecord {
+            filesystem_identifier: [0; 16],
+            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+            instance: InstanceGeneration(instance),
+            checkpoint_txg: CheckpointTxg(checkpoint_txg),
+            tree_table: NodePointer::empty_root(),
+            tree_identifier_watermark: 0,
+            rollback_floor: CheckpointTxg(0),
+            instance_table: NodePointer::empty_root(),
+            mapping_root: NodePointer::empty_root(),
+            allocation_record_tree_root: NodePointer::empty_root(),
+        }
+    }
+
+    /// 这几条用例的落点公式：偶数 txg 落盘 0、奇数落盘 1（只为把每块盘上有哪几条根摆清楚，不是第一版几何）。
+    fn device_of_an_even_or_odd_txg(checkpoint_txg: CheckpointTxg) -> DeviceIdentity {
+        if checkpoint_txg.0.is_multiple_of(2) {
+            DeviceIdentity(0)
+        } else {
+            DeviceIdentity(1)
+        }
+    }
+
+    /// 实例 1 择过 txg 8（实例表那一行），它之后的 txg 9 被抛弃；实例 2 从 txg 12 起。盘 1 上最新的根是被抛弃的 txg 9，
+    /// 最新的有效根是 txg 7：取最小时按 7 算（滤掉被抛弃的根），不按 9 算。只有被抛弃的根的那块盘算「一条有效根都没有」。
+    /// 判别力：不滤被抛弃的根时第一条断言得 9、第二条得 Some(9)。
+    #[test]
+    fn an_abandoned_root_newest_on_a_device_does_not_stand_for_that_devices_newest_valid_root() {
+        let table = InstanceTableRecords {
+            rows: vec![InstanceRow {
+                instance: InstanceGeneration(1),
+                selected_root_txg: CheckpointTxg(8),
+                applied_transaction_high_water: 0,
+            }],
+        };
+        let both_devices = [DeviceIdentity(0), DeviceIdentity(1)];
+        let roots = [root_of(1, 7), root_of(1, 8), root_of(1, 9), root_of(2, 12)];
+        assert_eq!(
+            newest_valid_root_txg_on_every_device(
+                &roots,
+                &table,
+                &both_devices,
+                &device_of_an_even_or_odd_txg
+            ),
+            Some(CheckpointTxg(7)),
+            "盘 0 上最新的有效根 txg 12、盘 1 上 txg 7（txg 9 被抛弃、不算），取最小是 7"
+        );
+        let only_the_abandoned_root_on_device_one = [root_of(1, 8), root_of(1, 9), root_of(2, 12)];
+        assert_eq!(
+            newest_valid_root_txg_on_every_device(
+                &only_the_abandoned_root_on_device_one,
+                &table,
+                &both_devices,
+                &device_of_an_even_or_odd_txg
+            ),
+            None,
+            "盘 1 上只有被抛弃的 txg 9：那块盘一条有效根都没有"
+        );
+    }
+}
diff --git a/crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs b/crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs
index 85ce144..7c96932 100644
--- a/crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs
+++ b/crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs
@@ -1,9 +1,11 @@
 //! 卸载那一串掉电之后只剩一块盘：剩下那块盘只读挂载、整读每个活文件，与卸载之前同一块盘上读出的逐项相同（走到的根 txg 与内容）。
 //! 判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第三节第 2 条（GC）；用户 2026-10-03 定甲 GA
 //! （`mount::RaiseFloorEntry::reuse_of_the_reclaimed_slots`：卸载那一格只把释放代不高于每块盘上最新有效根的那一截当场放开）。
-//! 刀摆在卸载那一串第一次空发布的单元写写完之后（先写系统配置那两个写 + 那几个单元写落下，journal 记录与根都没落），丢掉住着现行那一版的根
-//! 的那块盘。恢复择根不看 F（`recovery::choose_root`），剩下那块盘择到它自己上面最新的根，低于系统配置里的新 F。
-//! 两形照攻方（`research/prompts/m3-unlink-code-r3-opus-output.md` 第一节 1.4）：h2 窄盘续做之后卸载，h4 4 GiB 盘卸载—再挂循环之后第二次卸载。
+//! 刀摆在卸载那一串某一次空发布的单元写写完之后（先写系统配置那两个写、之前那几次空发布的全部写与这一次的单元写落下，这一次的 journal 记录与根都没落），
+//! 丢掉住着现行那一版的根的那块盘。恢复择根不看 F（`recovery::choose_root`），剩下那块盘择到它自己上面最新的根，低于系统配置里的新 F。
+//! 两形照攻方（`research/prompts/m3-unlink-code-r3-opus-output.md` 第一节 1.4）：h2 窄盘续做之后卸载，h4 4 GiB 盘卸载—再挂循环之后第二次卸载，刀都在第一次空发布。
+//! 第 16 批补两形（判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第三节第 3、4 条）：窄盘崩了再挂一次之后卸载、刀在第二次空发布，
+//! 剩下那块盘整读的活文件不为空；同一形上这一遍有一块盘一条有效根都读不出，这一串回收的槽整截扣到生效。
 //! 池级 checker 在单盘镜像上卸载之前就红六七条（判决第二节 N1「第五节跑前判据的字面」那一行），这里不判 checker，判的是读回。
 
 mod common_admission;
@@ -15,6 +17,7 @@ use common_admission::{
 };
 use singlefs_core::address::{CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber};
 use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::allocator::{PoolAllocator, RootRingOccupancy, RootRingOccupant};
 use singlefs_core::container_tree::ContainerTreeNodeEntries;
 use singlefs_core::lineage_tree::LineageContainerCapacity;
 use singlefs_core::mount::{
@@ -24,8 +27,11 @@ use singlefs_core::mount::{
 use singlefs_core::mounted_read::mount_read_only;
 use singlefs_core::mounted_session::MountedSession;
 use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
-use singlefs_core::recovery::{recover, JournalPolicy};
-use singlefs_core::root_ring::target_for_publish;
+use singlefs_core::recovery::{
+    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy,
+    RootRingSlotReading,
+};
+use singlefs_core::root_ring::{target_for_publish, RootRingSlot};
 use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
 use singlefs_core::transaction::{
     publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
@@ -35,16 +41,14 @@ use singlefs_core::unlink::{
     DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
 };
 use singlefs_core::write_accounting::WrittenStructureKind;
+use singlefs_format::ROOT_RING_REGIONS;
 use singlefs_harness::fault_injection::{
     FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
-    FaultSchedule, InjectedFault, SharedFaultPlan,
+    FaultSchedule, InjectedFault, NamedRootRingSlots, RootRingSlotTarget, SharedFaultPlan,
 };
 use singlefs_harness::history::HistoryDeviceWidth;
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
 
-/// 抬 F 那一串第一次发布之前先写系统配置那一步的写：每块盘一次，两块盘。
-const SYSTEM_CONFIGURATION_WRITES_OF_THE_SEQUENCE: u64 = 2;
-
 /// 删除批大小 1 个数据单元（只供测试的开关，挂载的开关链装上去，续做照它）。
 fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
     DeletionSwitches {
@@ -102,32 +106,76 @@ fn image_without(image: &MemoryPool, lost: DeviceIdentity) -> MemoryPool {
 }
 
 /// 卸载那一串掉电、丢掉一块盘之后要比的几样。
-struct LoneSurvivorAfterTheFirstUnmountPublish {
+struct LoneSurvivorAfterAnUnmountPublish {
     /// 丢掉的盘：住着卸载之前现行那一版的根。
     lost_device: DeviceIdentity,
     current_txg_before_the_unmount: CheckpointTxg,
-    /// 卸载那一串第一次空发布写了几个单元（不注错地在一份拷贝上卸载一次数出来的）。
-    unit_writes_of_the_first_publish: u64,
+    /// 掉电在那一串第几次空发布（从 0 数）的单元写写完之后，那一次写了几个单元（不注错地在一份拷贝上卸载一次数出来的）。
+    unit_writes_of_the_publish: u64,
     /// 剩下那块盘上恢复择到的根（看 journal）。
     root_chosen_on_the_survivor: Option<CheckpointTxg>,
     before_the_unmount: Result<WholeReadOfTheLiveFiles, String>,
     after_the_crash: Result<WholeReadOfTheLiveFiles, String>,
 }
 
-/// 在 `pool` 的会话上卸载：先在一份拷贝上不注错地卸载一次，数清那一串第一次空发布的单元写；再在另一份上从「先写系统配置那两个写 + 那几个单元写」
-/// 之后的第一个写起每个写都报错（掉电那一形：盘停在那几个单元写之后，journal 记录与根都没落）；丢掉住着现行那一版的根的那块盘，
-/// 卸载之前与掉电之后各在剩下那块盘上整读一遍。
-fn lone_survivor_after_the_first_unmount_publish(
+/// 卸载那一串第几次空发布（从 0 数）的单元写写完之后掉电。
+#[derive(Clone, Copy, Debug)]
+struct PublishOfTheUnmountSequence(usize);
+
+/// 卸载时每次读都失败的根环槽（只供测试的开关 `RootRingSlotTarget`，`NamedRootRingSlots::NONE` 是哪个都不点名）。
+fn ring_slots_failing_every_read_during_the_unmount(
+    pool: &PoolUnderTest<SparseBlockDevice>,
+    device_width: HistoryDeviceWidth,
+    named_slots: NamedRootRingSlots,
+) -> SharedFaultPlan {
+    SharedFaultPlan::armed(
+        device_width.fixed_geometry(),
+        FaultSchedule {
+            fault: InjectedFault::ReadFails,
+            device: FaultDeviceSelector::EveryDevice,
+            placement: FaultPlacement::WithinNamedRootRingSlots(RootRingSlotTarget {
+                named_slots,
+                region_devices: pool.parameters.region_devices,
+                fixed_structure_slot_spacing: pool.parameters.geometry.fixed_structure_slot_spacing,
+            }),
+            counting: FaultCounting::AcrossThePool,
+            occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
+        },
+    )
+}
+
+/// 在 `pool` 的盘面上拿 `allocator_of_the_unmount`（会话的分配器或它的改写本）与会话的现行那一版卸载：先在一份拷贝上不注错地卸载一次，
+/// 数清那一串第 `publish` 次空发布之前落下的写与它自己的单元写；再在另一份上从「那几个写 + 这一次的单元写」之后的第一个写起每个写都报错
+/// （掉电那一形：盘停在这一次的单元写之后，它的 journal 记录与根都没落）；丢掉住着现行那一版的根的那块盘，卸载之前与掉电之后各在剩下那块盘上整读一遍。
+/// `unreadable_ring_slots` 点名的根环槽在两次卸载里每次读都失败（盘上内容不动，整读不经这一层）。
+fn lone_survivor_after_the_unit_writes_of_an_unmount_publish(
     pool: &PoolUnderTest<SparseBlockDevice>,
     device_width: HistoryDeviceWidth,
-) -> LoneSurvivorAfterTheFirstUnmountPublish {
+    allocator_of_the_unmount: &PoolAllocator,
+    publish: PublishOfTheUnmountSequence,
+    unreadable_ring_slots: NamedRootRingSlots,
+) -> LoneSurvivorAfterAnUnmountPublish {
     let image_before_the_unmount = pool.image();
     let session = pool.session.as_ref().expect("这一刻有可写会话");
     let current_txg_before_the_unmount = session.current.root().checkpoint_txg;
-    let unit_writes_of_the_first_publish = {
-        let mut allocator = session.allocator.clone();
+    let (writes_before_the_publish, unit_writes_of_the_publish) = {
+        let mut allocator = allocator_of_the_unmount.clone();
         let mut current = session.current.clone();
-        let mut devices = plain_devices_on(&image_before_the_unmount);
+        let read_plan = ring_slots_failing_every_read_during_the_unmount(
+            pool,
+            device_width,
+            unreadable_ring_slots,
+        );
+        let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
+            plain_devices_on(&image_before_the_unmount)
+                .into_iter()
+                .map(|(identity, device)| {
+                    (
+                        identity,
+                        FaultInjectingBlockDevice::new(identity, device, read_plan.clone()),
+                    )
+                })
+                .collect();
         let unmounted = unmount(
             &pool.parameters,
             &mut devices,
@@ -138,14 +186,25 @@ fn lone_survivor_after_the_first_unmount_publish(
         let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted else {
             panic!("拷贝上卸载做成：{:?}", unmounted.as_ref().err());
         };
-        let first = &raised.publishes[0];
-        let structure_writes = |kind: WrittenStructureKind| first.writes.of(kind).write_calls;
-        first.writes.total().write_calls
-            - structure_writes(WrittenStructureKind::JournalRecord)
-            - structure_writes(WrittenStructureKind::RootSlot)
-            - structure_writes(WrittenStructureKind::SystemConfigurationSlot)
+        let the_publish = &raised.publishes[publish.0];
+        let structure_writes = |kind: WrittenStructureKind| the_publish.writes.of(kind).write_calls;
+        let writes_before_the_publish = raised
+            .system_configuration_writes_before_the_first_publish
+            .total()
+            .write_calls
+            + raised.publishes[..publish.0]
+                .iter()
+                .map(|earlier| earlier.writes.total().write_calls)
+                .sum::<u64>();
+        (
+            writes_before_the_publish,
+            the_publish.writes.total().write_calls
+                - structure_writes(WrittenStructureKind::JournalRecord)
+                - structure_writes(WrittenStructureKind::RootSlot)
+                - structure_writes(WrittenStructureKind::SystemConfigurationSlot),
+        )
     };
-    let plan = SharedFaultPlan::armed(
+    let write_plan = SharedFaultPlan::armed(
         device_width.fixed_geometry(),
         FaultSchedule {
             fault: InjectedFault::WriteFails,
@@ -153,21 +212,29 @@ fn lone_survivor_after_the_first_unmount_publish(
             placement: FaultPlacement::AnyOffset,
             counting: FaultCounting::AcrossThePool,
             occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(
-                SYSTEM_CONFIGURATION_WRITES_OF_THE_SEQUENCE + unit_writes_of_the_first_publish + 1,
+                writes_before_the_publish + unit_writes_of_the_publish + 1,
             ),
         },
     );
-    let mut failing_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
-        plain_devices_on(&image_before_the_unmount)
-            .into_iter()
-            .map(|(identity, device)| {
-                (
+    let read_plan =
+        ring_slots_failing_every_read_during_the_unmount(pool, device_width, unreadable_ring_slots);
+    let mut failing_devices: Vec<(
+        DeviceIdentity,
+        FaultInjectingBlockDevice<FaultInjectingBlockDevice<SparseBlockDevice>>,
+    )> = plain_devices_on(&image_before_the_unmount)
+        .into_iter()
+        .map(|(identity, device)| {
+            (
+                identity,
+                FaultInjectingBlockDevice::new(
                     identity,
-                    FaultInjectingBlockDevice::new(identity, device, plan.clone()),
-                )
-            })
-            .collect();
-    let mut allocator = session.allocator.clone();
+                    FaultInjectingBlockDevice::new(identity, device, write_plan.clone()),
+                    read_plan.clone(),
+                ),
+            )
+        })
+        .collect();
+    let mut allocator = allocator_of_the_unmount.clone();
     let mut current = session.current.clone();
     let unmounted = unmount(
         &pool.parameters,
@@ -181,13 +248,19 @@ fn lone_survivor_after_the_first_unmount_publish(
             unmounted,
             Err(MountError::RaiseFloorSequencePublishFailed(_))
         ),
-        "掉电之后那一串第一次空发布写 journal 记录时报错：{:?}",
+        "掉电之后那一串第 {} 次空发布写 journal 记录时报错：{:?}",
+        publish.0,
         unmounted.as_ref().err()
     );
     let crashed = MemoryPool {
         devices: failing_devices
             .iter()
-            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
+            .map(|(identity, device)| {
+                (
+                    *identity,
+                    device.wrapped_device().wrapped_device().image.clone(),
+                )
+            })
             .collect(),
         device_size_in_bytes: image_before_the_unmount.device_size_in_bytes,
     };
@@ -198,10 +271,10 @@ fn lone_survivor_after_the_first_unmount_publish(
     let lost_device = pool.parameters.region_devices
         [usize::try_from(ring_slot_of_the_current_root.region).expect("区域号")];
     let survivor_after_the_crash = image_without(&crashed, lost_device);
-    LoneSurvivorAfterTheFirstUnmountPublish {
+    LoneSurvivorAfterAnUnmountPublish {
         lost_device,
         current_txg_before_the_unmount,
-        unit_writes_of_the_first_publish,
+        unit_writes_of_the_publish,
         root_chosen_on_the_survivor: recover(&survivor_after_the_crash, JournalPolicy::Consult)
             .effective_root
             .map(|(_, checkpoint_txg)| checkpoint_txg),
@@ -213,14 +286,14 @@ fn lone_survivor_after_the_first_unmount_publish(
     }
 }
 
-/// 判一份 [`LoneSurvivorAfterTheFirstUnmountPublish`]：第一次空发布真写了单元；剩下那块盘择到的是它自己上面低于现行那一版的根；
+/// 判一份 [`LoneSurvivorAfterAnUnmountPublish`]：那一次空发布真写了单元；剩下那块盘择到的是它自己上面低于现行那一版的根；
 /// 卸载之前那块盘读得出；掉电之后读出来的与卸载之前逐项相同。
 fn assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
-    survivor: &LoneSurvivorAfterTheFirstUnmountPublish,
+    survivor: &LoneSurvivorAfterAnUnmountPublish,
 ) {
     assert!(
-        survivor.unit_writes_of_the_first_publish > 0,
-        "卸载那一串第一次空发布写了单元（这一刀要判的就是它们先于带新 F 的根写下去）"
+        survivor.unit_writes_of_the_publish > 0,
+        "卸载那一串那次空发布写了单元（这一刀要判的就是它们先于带新 F 的根写下去）"
     );
     assert!(
         survivor
@@ -242,6 +315,87 @@ fn assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
     );
 }
 
+/// 窄盘（不补环窗口、准入关着造，单元区 176 槽）：第一个文件之后不覆盖写，崩了再挂可写一次（准入判着、影子账开、批大小 1），在这次挂载的会话上卸载。
+/// 盘 1 上最新的根是写行那次（txg 4），盘 0 上是暖机那次（txg 5，现行那一版）；盘 1 上最新的根引用的槽有一个由暖机那次释放（释放代 5）。
+fn narrow_pool_remounted_once_after_its_first_file() -> PoolUnderTest<SparseBlockDevice> {
+    let mut pool =
+        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
+            HistoryDeviceWidth::UnitAreaOf384Slots,
+            176,
+        );
+    pool.session = None;
+    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
+        &pool.parameters,
+        &mut pool.devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        deletion_switches_with_batches_of_one(),
+    )
+    .unwrap_or_else(|error| panic!("第一个文件之后崩了再挂可写：{error:?}"));
+    let (_, session) = MountedSession::of_the_mount(mounted);
+    pool.session = Some(session);
+    pool
+}
+
+/// 会话分配器的一份改写本：根环表里不记 `device` 上的根环槽（「挂载那一刻读不出、这个进程之后没写过」，这个进程不知道那里住着根），
+/// 别的槽照盘上读出来的记成有效根（这个池里没有被抛弃的根）。经可写挂载造不出这一份：暖机在每块盘上都写了这个进程的根。
+fn allocator_not_knowing_the_root_ring_slots_of(
+    pool: &PoolUnderTest<SparseBlockDevice>,
+    device: DeviceIdentity,
+) -> PoolAllocator {
+    let image = pool.image();
+    let system_configuration =
+        choose_system_configuration(&image).expect("两块盘都在的盘面上系统配置择得出");
+    let immutable = &system_configuration.immutable;
+    let occupants: Vec<(RootRingSlot, RootRingOccupant)> = every_root_ring_slot(&immutable.sizes)
+        .into_iter()
+        .filter(|ring_slot| {
+            immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")] != device
+        })
+        .filter_map(|ring_slot| {
+            match read_root_ring_slot(
+                &image,
+                &immutable.region_devices,
+                &immutable.sizes,
+                &immutable.filesystem_identifier,
+                ring_slot,
+            ) {
+                RootRingSlotReading::SelfVerified(root) => Some((
+                    ring_slot,
+                    RootRingOccupant::ValidRoot {
+                        checkpoint_txg: root.checkpoint_txg,
+                    },
+                )),
+                RootRingSlotReading::Bad(_) => None,
+            }
+        })
+        .collect();
+    let session = pool.session();
+    let mut allocator = session.allocator.clone();
+    allocator.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
+        immutable.sizes.root_ring_slots_per_region,
+        occupants,
+        session.current.root().rollback_floor,
+        session.current.root().checkpoint_txg,
+    ));
+    allocator
+}
+
+/// `device` 那几个区域的每个根环槽。
+fn every_root_ring_slot_of(
+    pool: &PoolUnderTest<SparseBlockDevice>,
+    device: DeviceIdentity,
+) -> NamedRootRingSlots {
+    let slots_per_region = pool.parameters.geometry.root_ring_slots_per_region.count();
+    let ring_slots: Vec<RootRingSlot> = (0..ROOT_RING_REGIONS)
+        .filter(|region| {
+            pool.parameters.region_devices[usize::try_from(*region).expect("区域号")] == device
+        })
+        .flat_map(|region| (0..slots_per_region).map(move |slot| RootRingSlot { region, slot }))
+        .collect();
+    NamedRootRingSlots::naming(&ring_slots)
+}
+
 /// h2（攻方第一节 1.4）：不补环窗口、准入关着造的窄盘（单元区 176 槽），不先覆盖写，第一个文件顺序写长到 4 个单元，
 /// 在这个进程里发 unlink 的意图与 3 批（批大小 1）就崩；重开可写（准入判着、影子账开、批大小 1）续做走完，在这次挂载的会话上卸载。
 /// 判别力：卸载那一格回收的槽整截当场放开（丙1，`ReclaimedReuse::Immediately`）时，第一次空发布把单元写进剩下那块盘上最新的根还引用的槽，
@@ -307,7 +461,13 @@ fn after_the_first_unmount_publish_of_a_narrow_pool_whose_deletion_was_continued
     let (_, session) = MountedSession::of_the_mount(mounted);
     pool.session = Some(session);
     assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
-        &lone_survivor_after_the_first_unmount_publish(&pool, device_width),
+        &lone_survivor_after_the_unit_writes_of_an_unmount_publish(
+            &pool,
+            device_width,
+            &pool.session().allocator,
+            PublishOfTheUnmountSequence(0),
+            NamedRootRingSlots::NONE,
+        ),
     );
 }
 
@@ -348,6 +508,64 @@ fn after_the_first_publish_of_a_second_unmount_of_a_four_gibibyte_pool_a_lone_su
             .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写做成：{refusal:?}"));
     }
     assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
-        &lone_survivor_after_the_first_unmount_publish(&pool, device_width),
+        &lone_survivor_after_the_unit_writes_of_an_unmount_publish(
+            &pool,
+            device_width,
+            &pool.session().allocator,
+            PublishOfTheUnmountSequence(0),
+            NamedRootRingSlots::NONE,
+        ),
+    );
+}
+
+/// 活文件不为空的一形（判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第三节第 4 条：h2 整读的活文件集合为空）：
+/// 窄盘第一个文件之后崩了再挂一次（[`narrow_pool_remounted_once_after_its_first_file`]），在这次挂载的会话上卸载；掉电在那一串第二次空发布的单元写写完之后
+/// （第一次空发布连根落在盘 0，盘 1 上最新的根还是写行那次 txg 4），丢掉住着现行那一版（txg 5）的根的盘 0。剩下的盘 1 整读第一个文件（3000 字节）。
+/// 判别力：卸载那一格回收的槽整截当场放开（丙1）时，第二次空发布把单元写进盘 1 上最新的根还引用、由暖机那次释放的槽，
+/// 盘 1 只读挂载走树读不出（改回丙1 的拷贝上量过 `UnitUnreadable { slot: SlotNumber(1419) }`），最后一项红。
+#[test]
+fn after_the_second_unmount_publish_of_a_narrow_pool_remounted_once_a_lone_surviving_device_reads_back_its_live_file(
+) {
+    let pool = narrow_pool_remounted_once_after_its_first_file();
+    let survivor = lone_survivor_after_the_unit_writes_of_an_unmount_publish(
+        &pool,
+        HistoryDeviceWidth::UnitAreaOf384Slots,
+        &pool.session().allocator,
+        PublishOfTheUnmountSequence(1),
+        NamedRootRingSlots::NONE,
+    );
+    assert!(
+        survivor
+            .before_the_unmount
+            .as_ref()
+            .is_ok_and(|whole_read| !whole_read.live_files.is_empty()),
+        "卸载之前剩下那块盘读得出活文件（这一形要判的就是活文件不为空）：{:?}",
+        survivor.before_the_unmount
+    );
+    assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(&survivor);
+}
+
+/// 卸载这一遍有一块盘一条有效根都读不出时，这一串回收的槽整截扣到生效（D16（发布语义） 已定项 1「正常卸载」那一行「有一块盘一条有效根都读不出时这一截为空」）。
+/// 同上一形的池与刀；卸载用的分配器是改写本（[`allocator_not_knowing_the_root_ring_slots_of`]：这个进程不知道盘 1 的根环槽住着根），
+/// 两次卸载里盘 1 那几个根环槽每次读都失败，于是这一遍盘 1 上一条有效根都读不出。盘上的内容不动：掉电之后剩下的盘 1 照旧择到 txg 4。
+/// 经可写挂载走不到这一格（暖机在每块盘上都写了这个进程的根，那几个槽读坏就重读、仍坏就拒），这里用改写本强制进入（`.claude/rules/fs-design.md`
+/// 五条硬要求第 2 条）。
+/// 判别力：这一格改成照原门槛当场放开时，第二次空发布把单元写进盘 1 上最新的根还引用的槽，盘 1 只读挂载读不出，最后一项红。
+#[test]
+fn when_no_valid_root_on_device_one_is_read_during_the_unmount_every_reclaimed_slot_is_held_and_device_one_reads_back_its_live_file(
+) {
+    let pool = narrow_pool_remounted_once_after_its_first_file();
+    let survivor = lone_survivor_after_the_unit_writes_of_an_unmount_publish(
+        &pool,
+        HistoryDeviceWidth::UnitAreaOf384Slots,
+        &allocator_not_knowing_the_root_ring_slots_of(&pool, DeviceIdentity(1)),
+        PublishOfTheUnmountSequence(1),
+        every_root_ring_slot_of(&pool, DeviceIdentity(1)),
+    );
+    assert_eq!(
+        survivor.lost_device,
+        DeviceIdentity(0),
+        "丢掉的是住着现行那一版的根的盘 0，剩下的是这一遍读不出根的盘 1"
     );
+    assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(&survivor);
 }
diff --git a/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs b/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
new file mode 100644
index 0000000..f37066b
--- /dev/null
+++ b/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
@@ -0,0 +1,376 @@
+//! 卸载半途崩溃（系统配置已带新 F、带新 F 的根没落满每块盘）之后的下一次可写挂载只放开一截（D16（发布语义） 已定项 1「正常卸载」那一行，
+//! 用户 2026-10-04 选甲，`research/prompts/m3-batch14-code-r1-forks.md` 第 1 行）：重建分配器时当场可用的只到
+//! min(F_生效, 每块盘上最新有效根的 txg 取最小)，其余扣到这次挂载的暖机在每块盘上都落了根（`mount::rebuilt_allocator`）。
+//! 形 R（判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「形 R」那一行，攻方 `research/prompts/m3-batch14-code-r1-opus-output.md`
+//! 第一节 1.2）：卸载那一串只落了第一个系统配置写就崩 → 两块盘都在、再可写挂载 → 这次挂载取号、写行的单元写到一半再崩 → 丢掉盘 0。
+//! 恢复择根不看 F，剩下那块盘择到它自己上面最新的根，低于系统配置里的新 F；判剩下那块盘只读挂载、整读每个活文件，与卸载之前同一块盘上读出的相同。
+//! 造法照攻方原型 `research/prompts/m3-batch14-code-r1-opus-model/probe/main.rs` 的 `narrow_start`、`plain_start`、`run_remount_on`（只抄造法）：
+//! 原型按录制流取前缀刀，这里用故障注入让某一次写起每次写都报错，盘停在前 n 个写之后。池级 checker 在单盘镜像上判不了（判决同一行），这里判读回。
+
+mod common_admission;
+
+use common_admission::{
+    plain_devices_on,
+    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
+    start_plain, PoolUnderTest, OVERWRITE_BYTES,
+};
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::container_tree::ContainerTreeNodeEntries;
+use singlefs_core::lineage_tree::LineageContainerCapacity;
+use singlefs_core::mount::{
+    mount_writable_with_space_admission,
+    mount_writable_with_test_only_switches_and_deletion_switches, unmount, MountError, Mounted,
+    ShadowLedger,
+};
+use singlefs_core::mounted_read::mount_read_only;
+use singlefs_core::mounted_session::MountedSession;
+use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
+use singlefs_core::recovery::{recover, JournalPolicy};
+use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
+use singlefs_core::transaction::{
+    publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
+};
+use singlefs_core::unit::data_unit_payload_capacity;
+use singlefs_core::unlink::{
+    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
+};
+use singlefs_harness::fault_injection::{
+    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
+    FaultSchedule, InjectedFault, SharedFaultPlan,
+};
+use singlefs_harness::history::HistoryDeviceWidth;
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+
+/// 删除批大小 1 个数据单元（只供测试的开关，挂载的开关链装上去，续做照它）。
+fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
+    DeletionSwitches {
+        batch_size: UnlinkBatchSize::CappedForTests(
+            UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
+        ),
+        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
+        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
+    }
+}
+
+/// 只读挂载、把挂载态里每个活着的文件从头读到尾：走到的根 txg 与每个活文件（inode 号、全部字节）。删除中的 inode（nlink = 0）不读。
+#[derive(Debug, PartialEq, Eq)]
+struct WholeReadOfTheLiveFiles {
+    root_txg: CheckpointTxg,
+    live_files: Vec<(u64, Vec<u8>)>,
+}
+
+/// 只读挂载与整读每个活文件；哪一步读不出就交回那一步的错（只读挂载、打开、读）。
+fn whole_read_of_the_live_files(image: &MemoryPool) -> Result<WholeReadOfTheLiveFiles, String> {
+    let read_only = mount_read_only(image).map_err(|failure| format!("只读挂载：{failure:?}"))?;
+    let mut live_files = Vec::new();
+    // 迭代上界是这一版 inode 树的记录数；提前出口只有读不出的那三种错。
+    for record in read_only.mounted.inode_records() {
+        if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
+            continue;
+        }
+        let file = read_only
+            .mounted
+            .open_file(image, InodeNumber(record.inode))
+            .map_err(|failure| format!("打开 inode {}：{failure:?}", record.inode))?;
+        let read = file
+            .read_at(image, FileOffsetInBytes(0), record.size)
+            .map_err(|failure| format!("读 inode {}：{failure:?}", record.inode))?;
+        live_files.push((record.inode, read.bytes));
+    }
+    Ok(WholeReadOfTheLiveFiles {
+        root_txg: read_only.effective_root.checkpoint_txg,
+        live_files,
+    })
+}
+
+/// 一份镜像去掉一块盘。
+fn image_without(image: &MemoryPool, lost: DeviceIdentity) -> MemoryPool {
+    MemoryPool {
+        devices: image
+            .devices
+            .iter()
+            .filter(|(identity, _)| **identity != lost)
+            .map(|(identity, device)| (*identity, device.clone()))
+            .collect(),
+        device_size_in_bytes: image.device_size_in_bytes,
+    }
+}
+
+/// 两块盘包上同一份注错计划：整池从第 `first_failing_write` 次写起每次写都报错（掉电那一形：盘停在之前那几个写之后）。
+fn devices_failing_every_write_from(
+    image: &MemoryPool,
+    device_width: HistoryDeviceWidth,
+    first_failing_write: u64,
+) -> Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> {
+    let plan = SharedFaultPlan::armed(
+        device_width.fixed_geometry(),
+        FaultSchedule {
+            fault: InjectedFault::WriteFails,
+            device: FaultDeviceSelector::EveryDevice,
+            placement: FaultPlacement::AnyOffset,
+            counting: FaultCounting::AcrossThePool,
+            occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(first_failing_write),
+        },
+    );
+    plain_devices_on(image)
+        .into_iter()
+        .map(|(identity, device)| {
+            (
+                identity,
+                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
+            )
+        })
+        .collect()
+}
+
+/// 注错的那几块盘上这会儿的盘面。
+fn image_of(
+    devices: &[(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)],
+    device_size_in_bytes: u64,
+) -> MemoryPool {
+    MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
+            .collect(),
+        device_size_in_bytes,
+    }
+}
+
+/// 再可写挂载走哪个入口：窄盘与攻方原型同一种开关（准入判着、影子账开、删除批大小 1），普通盘走 `crash_and_mount_writable` 那一个。
+#[derive(Clone, Copy, Debug)]
+enum RemountEntry {
+    WithDeletionBatchesOfOne,
+    WithTheSpaceAdmissionJudged,
+}
+
+fn mount_writable_through(
+    entry: RemountEntry,
+    pool: &PoolUnderTest<SparseBlockDevice>,
+    devices: &mut Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)>,
+) -> Result<Mounted, MountError> {
+    match entry {
+        RemountEntry::WithDeletionBatchesOfOne => {
+            mount_writable_with_test_only_switches_and_deletion_switches(
+                &pool.parameters,
+                devices,
+                SpaceAdmission::JudgedByTheFormula,
+                ShadowLedger::On,
+                deletion_switches_with_batches_of_one(),
+            )
+        }
+        RemountEntry::WithTheSpaceAdmissionJudged => mount_writable_with_space_admission(
+            &pool.parameters,
+            devices,
+            SpaceAdmission::JudgedByTheFormula,
+        ),
+    }
+}
+
+/// 形 R 两次崩溃、丢掉一块盘之后要比的几样。
+struct LoneSurvivorAfterACrashedUnmountAndACrashedRemount {
+    current_txg_before_the_unmount: CheckpointTxg,
+    /// 剩下那块盘上恢复择到的根（看 journal）。
+    root_chosen_on_the_survivor: Option<CheckpointTxg>,
+    before_the_unmount: Result<WholeReadOfTheLiveFiles, String>,
+    after_the_crashes: Result<WholeReadOfTheLiveFiles, String>,
+}
+
+/// 在 `pool` 的会话上卸载，前 `unmount_writes_landed` 个写落下就崩；崩溃镜像上两块盘都在、经 `remount_entry` 再可写挂载，
+/// 前 `remount_writes_landed` 个写落下又崩；丢掉 `lost_device`，卸载之前与两次崩溃之后各在剩下那块盘上整读一遍。
+fn lone_survivor_after_a_crashed_unmount_and_a_crashed_remount(
+    pool: &PoolUnderTest<SparseBlockDevice>,
+    device_width: HistoryDeviceWidth,
+    unmount_writes_landed: u64,
+    remount_entry: RemountEntry,
+    remount_writes_landed: u64,
+    lost_device: DeviceIdentity,
+) -> LoneSurvivorAfterACrashedUnmountAndACrashedRemount {
+    let image_before_the_unmount = pool.image();
+    let session = pool.session.as_ref().expect("这一刻有可写会话");
+    let current_txg_before_the_unmount = session.current.root().checkpoint_txg;
+    let mut unmount_devices = devices_failing_every_write_from(
+        &image_before_the_unmount,
+        device_width,
+        unmount_writes_landed + 1,
+    );
+    let mut allocator = session.allocator.clone();
+    let mut current = session.current.clone();
+    let unmounted = unmount(
+        &pool.parameters,
+        &mut unmount_devices,
+        &mut allocator,
+        &mut current,
+        session.shadow_ledger,
+    );
+    assert!(
+        unmounted.is_err(),
+        "卸载在第 {} 个写上掉电、报错：{:?}",
+        unmount_writes_landed + 1,
+        unmounted.as_ref().ok().map(|_| "做成了")
+    );
+    let after_the_unmount = image_of(
+        &unmount_devices,
+        image_before_the_unmount.device_size_in_bytes,
+    );
+    let mut remount_devices = devices_failing_every_write_from(
+        &after_the_unmount,
+        device_width,
+        remount_writes_landed + 1,
+    );
+    let remounted = mount_writable_through(remount_entry, pool, &mut remount_devices);
+    assert!(
+        remounted.is_err(),
+        "再可写挂载在第 {} 个写上掉电、报错：{:?}",
+        remount_writes_landed + 1,
+        remounted.as_ref().ok().map(|_| "做成了")
+    );
+    drop(remounted);
+    let survivor_after_the_crashes = image_without(
+        &image_of(
+            &remount_devices,
+            image_before_the_unmount.device_size_in_bytes,
+        ),
+        lost_device,
+    );
+    LoneSurvivorAfterACrashedUnmountAndACrashedRemount {
+        current_txg_before_the_unmount,
+        root_chosen_on_the_survivor: recover(&survivor_after_the_crashes, JournalPolicy::Consult)
+            .effective_root
+            .map(|(_, checkpoint_txg)| checkpoint_txg),
+        before_the_unmount: whole_read_of_the_live_files(&image_without(
+            &image_before_the_unmount,
+            lost_device,
+        )),
+        after_the_crashes: whole_read_of_the_live_files(&survivor_after_the_crashes),
+    }
+}
+
+/// 剩下那块盘择到的是它自己上面低于卸载之前现行那一版的根；卸载之前那块盘读得出；两次崩溃之后读出来的与卸载之前逐项相同。
+fn assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
+    survivor: &LoneSurvivorAfterACrashedUnmountAndACrashedRemount,
+) {
+    assert!(
+        survivor
+            .root_chosen_on_the_survivor
+            .is_some_and(|chosen| chosen < survivor.current_txg_before_the_unmount),
+        "剩下那块盘择到它自己上面早于卸载之前现行那一版（txg {:?}）的根：{:?}",
+        survivor.current_txg_before_the_unmount,
+        survivor.root_chosen_on_the_survivor
+    );
+    assert!(
+        survivor.before_the_unmount.is_ok(),
+        "卸载之前剩下那块盘只读挂载、整读每个活文件做得成：{:?}",
+        survivor.before_the_unmount
+    );
+    assert_eq!(
+        survivor.after_the_crashes, survivor.before_the_unmount,
+        "剩下那块盘走到的根 txg 与每个活文件的内容与卸载之前同一块盘上读出的相同：再挂那一次当场放开的槽没有一个被它最新的根引用"
+    );
+}
+
+/// h2（攻方第一节 1.2）：不补环窗口、准入关着造的窄盘（单元区 176 槽），不先覆盖写，第一个文件顺序写长到 4 个单元，在这个进程里发 unlink 的意图与
+/// 3 批（批大小 1）就崩；重开可写（准入判着、影子账开、批大小 1）续做走完。卸载只落下第一个写（盘 0 的系统配置带新 F）就崩；再可写挂载落下
+/// 6 个写（取号两个系统配置写、写行那次的前 4 个单元写）又崩；丢盘 0。
+/// 判别力：重建分配器照原门槛 max(F_生效, 窗口里最旧有效根) 整截当场放开时，写行那次的单元写进盘 1 上最新的根（txg 13 < F_生效 14）还引用的槽，
+/// 盘 1 只读挂载走树读不出（攻方原型上量过 `MappingStillUnreadable { slot: SlotNumber(1411) }`），最后一项红。
+#[test]
+fn after_a_crashed_unmount_of_a_narrow_pool_whose_deletion_was_continued_the_next_writable_mount_crashing_in_its_row_publish_keeps_what_device_one_reads(
+) {
+    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
+    let data_units_of_the_file = 4;
+    let batches_published_after_the_intent = 3;
+    let mut pool =
+        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
+            device_width,
+            176,
+        );
+    pool.sequential_write((data_units_of_the_file - 1) * data_unit_payload_capacity() + 100)
+        .unwrap_or_else(|refusal| panic!("顺序写把文件长到 4 个单元：{refusal:?}"));
+    {
+        let PoolUnderTest {
+            session,
+            devices,
+            parameters,
+            write_time_seconds,
+            ..
+        } = &mut pool;
+        let session = session.as_mut().expect("这一刻有可写会话");
+        let mut current = session
+            .current
+            .version_of_trees()
+            .expect("第一个文件之后有树")
+            .clone();
+        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
+        current = publish_unlink_intent(
+            &mut writer,
+            &mut session.allocator,
+            &current,
+            InodeNumber(FIRST_INODE_NUMBER),
+            *write_time_seconds,
+            session.instance,
+        )
+        .expect("意图那次");
+        for batch_index in 0..batches_published_after_the_intent {
+            current = publish_unlink_step(
+                &mut writer,
+                &mut session.allocator,
+                &current,
+                InodeNumber(FIRST_INODE_NUMBER),
+                deletion_switches_with_batches_of_one().batch_size,
+                session.instance,
+            )
+            .unwrap_or_else(|refusal| panic!("第 {batch_index} 批：{refusal:?}"));
+        }
+    }
+    pool.session = None;
+    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
+        &pool.parameters,
+        &mut pool.devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        deletion_switches_with_batches_of_one(),
+    )
+    .unwrap_or_else(|error| panic!("崩了之后重开可写、续做：{error:?}"));
+    let (_, session) = MountedSession::of_the_mount(mounted);
+    pool.session = Some(session);
+    assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
+        &lone_survivor_after_a_crashed_unmount_and_a_crashed_remount(
+            &pool,
+            device_width,
+            1,
+            RemountEntry::WithDeletionBatchesOfOne,
+            6,
+            DeviceIdentity(0),
+        ),
+    );
+}
+
+/// 普通盘（攻方第一节 1.2 普通盘族 `plain W=4 k=1 m=1 k2=1`）：4 GiB 盘（随机历史默认那一档），第一个文件之后覆盖写 1 次、崩了再挂 1 次、
+/// 再覆盖写 1 次。卸载只落下第一个写就崩；再可写挂载（准入判着）落下 33 个写又崩；丢盘 0。
+/// 判别力：重建分配器整截当场放开时，再挂那一次的单元写进盘 1 上最新的根还引用的槽，盘 1 只读挂载做得成、打开活文件时 extent 树走不下去
+/// （攻方原型上量过 `ExtentTreeWalk(MappingStillUnreadable { slot: SlotNumber(50253) })`），最后一项红。
+#[test]
+fn after_a_crashed_unmount_of_a_four_gibibyte_pool_remounted_once_the_next_writable_mount_crashing_after_its_thirty_third_write_keeps_what_device_one_reads(
+) {
+    let device_width = HistoryDeviceWidth::FourGibibytes;
+    let mut pool = start_plain(device_width);
+    pool.overwrite(OVERWRITE_BYTES)
+        .unwrap_or_else(|refusal| panic!("第一个文件之后覆盖写 1 次：{refusal:?}"));
+    pool.crash_and_mount_writable()
+        .unwrap_or_else(|error| panic!("崩了再挂：{error:?}"));
+    pool.overwrite(OVERWRITE_BYTES)
+        .unwrap_or_else(|refusal| panic!("再挂之后覆盖写 1 次：{refusal:?}"));
+    assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
+        &lone_survivor_after_a_crashed_unmount_and_a_crashed_remount(
+            &pool,
+            device_width,
+            1,
+            RemountEntry::WithTheSpaceAdmissionJudged,
+            33,
+            DeviceIdentity(0),
+        ),
+    );
+}
````

## 2、实现员 P 的变异追加

````text
删除批第 16 批 甲：可写挂载重建分配器时当场放开不取 min(F_生效, m)（照原门槛整截当场放开），形 R 剩下盘 1 读回（h2 窄盘与 4 GiB 盘两形同一个测试二进制）	crates/singlefs-core/src/mount.rs	            effective_floor.min(newest_valid_root_on_every_device),	            effective_floor,	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads	after_a_crashed_unmount_of_a_narrow_pool_whose_deletion_was_continued_the_next_writable_mount_crashing_in_its_row_publish_keeps_what_device_one_reads
删除批第 16 批 GA：卸载那一格某块盘一条有效根都读不出时照原门槛当场放开（不整截扣到生效），盘 1 这一遍读不出根时单盘幸存读回	crates/singlefs-core/src/mount.rs	                None => Vec::new(),	                None => allocator.reclaim_released_up_to(floor_of_the_reclaim, ReclaimedReuse::Immediately),	-p singlefs-harness --test a_lone_surviving_device_reads_back_what_it_read_before_the_unmount	when_no_valid_root_on_device_one_is_read_during_the_unmount_every_reclaimed_slot_is_held_and_device_one_reads_back_its_live_file
删除批第 16 批 GA：每块盘上最新有效根取最小时不滤被抛弃的根（卸载那一格与重建分配器那一格共用的那一处定义）	crates/singlefs-core/src/mount.rs	                    !abandoned_by_table(root, table)\n                        && device_of_txg(root.checkpoint_txg) == *device	                    device_of_txg(root.checkpoint_txg) == *device	-p singlefs-core --lib	an_abandoned_root_newest_on_a_device_does_not_stand_for_that_devices_newest_valid_root
删除批第 16 批 GC：卸载那一格回收的槽整截当场放开（改回丙1），窄盘崩了再挂一次之后第二次空发布、活文件不为空的单盘幸存读回	crates/singlefs-core/src/mount.rs	                    floor_of_the_reclaim.min(newest_valid_root_on_every_device),	                    floor_of_the_reclaim,	-p singlefs-harness --test a_lone_surviving_device_reads_back_what_it_read_before_the_unmount	after_the_second_unmount_publish_of_a_narrow_pool_remounted_once_a_lone_surviving_device_reads_back_its_live_file
````

## 3、实现员 Q（Q-F 经会话与挂载续做两条用例、recovery.rs 那条 TODO）

````diff
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -4262,7 +4262,8 @@
 /// 每盘落点数的下限：mkfs 写在单元区里的 2 个落点，加新池新建文件只写 8 个单元时的 8 个落点。
 /// 新池新建文件在默认几何下写 12 个落点（字节表五：28 条记录，每盘 14 条；提交内生块个数随几何变），之后每次发布只多不少，
 /// 所以这条下限比盘上实有的松。
-/// TODO：下限没随分配记录树按位置寻址（D8（核心索引结构） 已定项 14）抬上去：内生块个数随几何变，取各几何的最小值还是按几何现算，没有条款。
+/// TODO：下限没随分配记录树按位置寻址（D8（核心索引结构） 已定项 14）抬上去：内生块个数随几何变，取各几何的最小值还是按几何现算，没有条款；
+/// 欠账在 `.claude/kb/checks-owed.md` C692（新池新建文件每盘落点下限偏松），定下取法之后照那一行改常量与它的单测。
 const NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE: usize = 10;
 
 /// 冷走读里的中央映射树（自举豁免，只按父指针里的位置条目读）：到第一次有树根的提示读不出、要经映射回退时，
diff --git a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
--- a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
+++ b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
@@ -571,6 +571,69 @@
     );
 }
 
+/// 经会话发 unlink（`UserChange::Unlink`，会话把调用方给的时间交给意图那次）：给一个与这条记录上一版不同的时间，会话发成意图与三步；
+/// 意图那一版与还带这条记录的两批（留 2、留 1）各自两块盘上第一个文件那条记录的三个时间秒都等于给的时间。
+/// 上面那一条直接调 `publish_unlink_intent`，判不出会话这一层（`research/prompts/m3-batch14-code-r1-main-verification.md` 第二节 N3）。
+/// 判别力：会话那一层丢掉调用方的时间、意图写 0 时（攻方那一刀 MQ1）这一条红。
+#[test]
+fn a_session_unlink_writes_the_given_time_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk(
+) {
+    let mut pool = pool_with_a_three_unit_file("write-time-of-a-session-unlink");
+    let time_before_the_unlink = write_time_seconds_of_the_first_file(&pool);
+    let time_of_the_unlink = time_before_the_unlink + 600;
+    let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
+    let (_, mut session) = MountedSession::of_the_mount(mounted);
+    let published = session
+        .publish_user_change(
+            &mut devices,
+            UserChange::Unlink {
+                inode: FIRST_FILE,
+                write_time_seconds: time_of_the_unlink,
+            },
+        )
+        .expect("会话 unlink");
+    drop(devices);
+    let image = pool.memory_pool();
+    let on_disk_in_every_version_with_the_record: Vec<Vec<[u64; 3]>> = published
+        .versions_published
+        .iter()
+        .filter(|version| version.inode_record.is_some())
+        .map(|version| time_seconds_of_the_first_file_on_each_device(&image, version))
+        .collect();
+    assert_eq!(
+        on_disk_in_every_version_with_the_record,
+        vec![vec![[time_of_the_unlink; 3]; 2]; 3],
+        "意图那一版与留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都是调用方给会话的时间（上一版是 {time_before_the_unlink}）"
+    );
+}
+
+/// 意图落盘之后崩溃（进程内发完意图、一批没发就重开镜像），可写挂载续做：续做里还带这条记录的那两批（留 2、留 1）各自两块盘上
+/// 第一个文件那条记录的三个时间秒都等于意图那次给的时间。续做的几批没有调用方给时间，照抄从盘上读回的上一版这条记录的值
+/// （用户 2026-10-03 定 Q-F 乙「各批照抄」；D8（核心索引结构） 已定项 16 第 7 条 ①b）。
+/// 判别力：续做那几批另取时间（取 0、取一个固定时间），或挂载从盘上读回这条记录时丢了时间，这一条红。
+#[test]
+fn the_batches_of_a_mount_continuation_after_the_intent_copy_the_time_the_intent_wrote_on_disk() {
+    let mut pool = pool_with_a_three_unit_file("write-time-of-the-continuation-batches");
+    let time_before_the_unlink = write_time_seconds_of_the_first_file(&pool);
+    let time_of_the_intent = time_before_the_unlink + 600;
+    publish_the_intent_in_process_at(&mut pool, time_of_the_intent);
+    let (mounted, devices) = remount_with_batches_of_one(&mut pool);
+    drop(devices);
+    let image = pool.memory_pool();
+    let on_disk_in_every_continuation_batch_with_the_record: Vec<Vec<[u64; 3]>> = mounted
+        .output
+        .deletion_queue_continuation
+        .iter()
+        .filter(|step| step.inode_record.is_some())
+        .map(|step| time_seconds_of_the_first_file_on_each_device(&image, step))
+        .collect();
+    assert_eq!(
+        on_disk_in_every_continuation_batch_with_the_record,
+        vec![vec![[time_of_the_intent; 3]; 2]; 2],
+        "续做的留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都照抄意图那次给的时间（上一版是 {time_before_the_unlink}）"
+    );
+}
+
 /// 两块盘的池：标签 6 是全部副本之和，每份一个副本。
 const REPLICAS_OF_THE_TWO_DEVICE_POOL: u64 = 2;
 
````

## 4、实现员 Q 的变异追加

````text
删除批第 16 批 N3：会话那一层丢掉调用方给 unlink 的时间、意图写 0（攻方 MQ1）	crates/singlefs-core/src/mounted_session.rs	            inode,\n            write_time_seconds,\n            instance,\n        ),	            inode,\n            { let _ = write_time_seconds; 0 },\n            instance,\n        ),	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version	a_session_unlink_writes_the_given_time_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk
删除批第 16 批 N3：挂载续做的几批不照抄这条记录上一版的写入时间、取 0	crates/singlefs-core/src/transaction.rs	                write_time_seconds: record.write_time_seconds,	                write_time_seconds: 0,	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version	the_batches_of_a_mount_continuation_after_the_intent_copy_the_time_the_intent_wrote_on_disk
删除批第 16 批 N3：挂载续做的几批不照抄这条记录上一版的写入时间、取一个固定时间	crates/singlefs-core/src/transaction.rs	                write_time_seconds: record.write_time_seconds,	                write_time_seconds: 1_788_000_000,	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version	the_batches_of_a_mount_continuation_after_the_intent_copy_the_time_the_intent_wrote_on_disk
删除批第 16 批 N3：挂载从盘上读回 inode 记录时丢掉三个时间秒、续做的几批照抄到 0	crates/singlefs-core/src/records.rs	        let write_time_seconds = reader.get_u64();	        let write_time_seconds = { let _ = reader.get_u64(); 0 };	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version	the_batches_of_a_mount_continuation_after_the_intent_copy_the_time_the_intent_wrote_on_disk
````
