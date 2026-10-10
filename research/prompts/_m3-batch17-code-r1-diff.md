# 删除批第 17 批合入的改动原样（2026-10-05）

## 实现员 P：crates.patch（/tmp/claude-1000/m3-batch17/p/patch/crates.patch）

```diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 7fb7c4d..d5dbe5c 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -1737,8 +1737,8 @@ fn unreadable_ring_slots_inside_the_window(
         .collect()
 }
 
-/// 重建分配器：从上一版的分配记录重建，按可再分配谓词的门槛回收（当场可用的只到每块盘上最新有效根那一截，其余扣到这次挂载的暖机
-/// 落满每块盘，甲），再把只被被抛弃根引用的槽隔离（影子账：被抛弃的根按最新根指着的
+/// 重建分配器：从上一版的分配记录重建，按可再分配谓词的门槛回收（当场可用的只到每块盘上最新有效根那一截、某块盘一条有效根都读不出时
+/// 只到一圈窗口里最旧有效根，其余扣到这次挂载的暖机落满每块盘，甲），再把只被被抛弃根引用的槽隔离（影子账：被抛弃的根按最新根指着的
 /// 实例表判，D23（journal 的角色与格式） 已定项 14 射程「影子账与按实例表判抛弃留着，理由是崩溃恢复」）。
 /// 影子账只住内存，所以每次挂载都要重算（步 4 / 步 5 代码三方第一轮云端攻方腿打中：普通重开一次隔离就归零）。
 /// 每块盘的空闲图从择到的系统配置按环长现算的单元区起点起（与偏移 417 那 8 字节相等，择的时候判过；C475（非默认环长下单元区起点取编译期常量））。
@@ -1860,7 +1860,10 @@ fn rebuilt_allocator<Device: BlockDevice>(
     // （判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「形 R」那一行）。于是当场可用的只到
     // reclaim_floor(min(F_生效, m), 一圈窗口里最旧有效根)，m 是每块盘上最新有效根的 txg 取最小（被抛弃的根不算，与卸载那一格同一处定义
     // [`newest_valid_root_txg_on_every_device`]）；其余到原门槛为止的那一截扣住，这次挂载的暖机在每块盘上都落了根之后放开（`establish_instance`）。
-    // 某块盘一条有效根都读不出（m 取不到）时照原门槛当场放开。
+    // 某块盘一条有效根都读不出（m 取不到）时当场只放到一圈窗口里最旧有效根（用户 2026-10-04 选甲，`research/prompts/m3-batch16-code-r1-forks.md`
+    // 第 1 行；判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节「再挂期间某块盘根环整段读坏」那一行）：那块盘上真正最新的根
+    // 这一遍看不到，它引用的单元释放代可以高到 F_生效；窗口里最旧有效根与回收门槛同一个读数（挂载时读不出的槽按它在窗口里能住的最小 txg 并进）。
+    // 这一遍一条有效根都没读出、窗口里最旧有效根也取不到时，「只到窗口里最旧有效根」的那一截是空的：当场一个都不放，整截扣到暖机落满。
     let device_of_txg = |txg: CheckpointTxg| {
         let target = target_for_publish(txg, slots_per_region);
         system_configuration.immutable.region_devices
@@ -1874,13 +1877,15 @@ fn rebuilt_allocator<Device: BlockDevice>(
         &all_devices,
         &device_of_txg,
     ) {
-        Some(newest_valid_root_on_every_device) => reclaim_floor(
+        Some(newest_valid_root_on_every_device) => Some(reclaim_floor(
             effective_floor.min(newest_valid_root_on_every_device),
             oldest_valid_root_in_the_window,
-        ),
-        None => floor_of_the_reclaim,
+        )),
+        None => oldest_valid_root_in_the_window,
     };
-    allocator.reclaim_released_up_to(floor_released_immediately, ReclaimedReuse::Immediately);
+    if let Some(released_immediately_up_to) = floor_released_immediately {
+        allocator.reclaim_released_up_to(released_immediately_up_to, ReclaimedReuse::Immediately);
+    }
     allocator.reclaim_released_up_to(
         floor_of_the_reclaim,
         ReclaimedReuse::HeldUntilFloorTakesEffect,
@@ -2471,7 +2476,10 @@ enum ReuseOfTheSlotsReclaimedByTheSequence {
 ///   一次抽样 9027 个判定择中 0 次）。
 /// - 这一遍读不出、自证不过的槽当没有根（与这一遍回收门槛同一个读数）：恢复同样择不到它；它日后又读得出的话那条根只会更新或更旧——更新的引用的已释放槽更少，
 ///   更旧的轮不到被择。读不出让最小值只会更小，当场放开的只会更少。
-/// - 有一块盘上一条有效根都没读出来：交回 `None`，这一串回收的槽整截扣到生效（不按 txg 0 去比，说清是哪一格）。
+/// - 有一块盘上一条有效根都没读出来：交回 `None`（不按 txg 0 去比，说清是哪一格），两格各自处置：
+///   卸载那一格这一串回收的槽整截扣到生效；重建那一格当场只放到一圈窗口里最旧有效根（挂载时读不出的槽按它在窗口里能住的最小 txg 并进，
+///   与回收门槛同一个读数），其余扣到这次挂载的暖机落满每块盘（`rebuilt_allocator`，用户 2026-10-04 选甲，
+///   `research/prompts/m3-batch16-code-r1-forks.md` 第 1 行）。
 fn newest_valid_root_txg_on_every_device(
     roots_read_from_the_ring: &[RootRecord],
     table: &InstanceTableRecords,
diff --git a/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs b/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
index f37066b..aad7510 100644
--- a/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
+++ b/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
@@ -1,6 +1,8 @@
 //! 卸载半途崩溃（系统配置已带新 F、带新 F 的根没落满每块盘）之后的下一次可写挂载只放开一截（D16（发布语义） 已定项 1「正常卸载」那一行，
 //! 用户 2026-10-04 选甲，`research/prompts/m3-batch14-code-r1-forks.md` 第 1 行）：重建分配器时当场可用的只到
-//! min(F_生效, 每块盘上最新有效根的 txg 取最小)，其余扣到这次挂载的暖机在每块盘上都落了根（`mount::rebuilt_allocator`）。
+//! min(F_生效, 每块盘上最新有效根的 txg 取最小)，某块盘一条有效根都读不出时只到一圈窗口里最旧有效根（用户 2026-10-04 选甲，
+//! `research/prompts/m3-batch16-code-r1-forks.md` 第 1 行），其余扣到这次挂载的暖机在每块盘上都落了根（`mount::rebuilt_allocator`），
+//! 暖机落满之后放开（`mount::establish_instance`）。
 //! 形 R（判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「形 R」那一行，攻方 `research/prompts/m3-batch14-code-r1-opus-output.md`
 //! 第一节 1.2）：卸载那一串只落了第一个系统配置写就崩 → 两块盘都在、再可写挂载 → 这次挂载取号、写行的单元写到一半再崩 → 丢掉盘 0。
 //! 恢复择根不看 F，剩下那块盘择到它自己上面最新的根，低于系统配置里的新 F；判剩下那块盘只读挂载、整读每个活文件，与卸载之前同一块盘上读出的相同。
@@ -16,10 +18,11 @@ use common_admission::{
 };
 use singlefs_core::address::{CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber};
 use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::block_device::BlockDevice;
 use singlefs_core::container_tree::ContainerTreeNodeEntries;
 use singlefs_core::lineage_tree::LineageContainerCapacity;
 use singlefs_core::mount::{
-    mount_writable_with_space_admission,
+    mount_writable_with_space_admission, mount_writable_with_test_only_switches,
     mount_writable_with_test_only_switches_and_deletion_switches, unmount, MountError, Mounted,
     ShadowLedger,
 };
@@ -27,6 +30,7 @@ use singlefs_core::mounted_read::mount_read_only;
 use singlefs_core::mounted_session::MountedSession;
 use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
 use singlefs_core::recovery::{recover, JournalPolicy};
+use singlefs_core::root_ring::RootRingSlot;
 use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
 use singlefs_core::transaction::{
     publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
@@ -35,9 +39,10 @@ use singlefs_core::unit::data_unit_payload_capacity;
 use singlefs_core::unlink::{
     DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
 };
+use singlefs_format::ROOT_RING_REGIONS;
 use singlefs_harness::fault_injection::{
     FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
-    FaultSchedule, InjectedFault, SharedFaultPlan,
+    FaultSchedule, InjectedFault, NamedRootRingSlots, RootRingSlotTarget, SharedFaultPlan,
 };
 use singlefs_harness::history::HistoryDeviceWidth;
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
@@ -98,13 +103,12 @@ fn image_without(image: &MemoryPool, lost: DeviceIdentity) -> MemoryPool {
     }
 }
 
-/// 两块盘包上同一份注错计划：整池从第 `first_failing_write` 次写起每次写都报错（掉电那一形：盘停在之前那几个写之后）。
-fn devices_failing_every_write_from(
-    image: &MemoryPool,
+/// 整池从第 `first_failing_write` 次写起每次写都报错的注错计划（掉电那一形：盘停在之前那几个写之后）。
+fn plan_failing_every_write_from(
     device_width: HistoryDeviceWidth,
     first_failing_write: u64,
-) -> Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> {
-    let plan = SharedFaultPlan::armed(
+) -> SharedFaultPlan {
+    SharedFaultPlan::armed(
         device_width.fixed_geometry(),
         FaultSchedule {
             fault: InjectedFault::WriteFails,
@@ -113,7 +117,16 @@ fn devices_failing_every_write_from(
             counting: FaultCounting::AcrossThePool,
             occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(first_failing_write),
         },
-    );
+    )
+}
+
+/// 两块盘包上同一份注错计划：整池从第 `first_failing_write` 次写起每次写都报错。
+fn devices_failing_every_write_from(
+    image: &MemoryPool,
+    device_width: HistoryDeviceWidth,
+    first_failing_write: u64,
+) -> Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> {
+    let plan = plan_failing_every_write_from(device_width, first_failing_write);
     plain_devices_on(image)
         .into_iter()
         .map(|(identity, device)| {
@@ -139,6 +152,53 @@ fn image_of(
     }
 }
 
+/// 再可写挂载那一次挂载期间根环怎么读（只供测试的开关 `RootRingSlotTarget`）：照常读，或点名那块盘住的几个区域里每个根环槽每次读都报块设备错
+/// （盘上内容不动；挂载交回之后的恢复与整读不经这一层、读得出：再挂这次挂载期间的瞬时读错那一形）。
+#[derive(Clone, Copy, Debug)]
+enum RootRingDuringTheRemount {
+    ReadAsUsual,
+    EverySlotOfTheDeviceFailsEveryRead(DeviceIdentity),
+}
+
+/// 再挂那一次的读注错计划：照常读的不注入、只数调用；点名一块盘的，那块盘住的几个区域里每个根环槽每次读都报错。
+fn read_plan_of_the_remount(
+    pool: &PoolUnderTest<SparseBlockDevice>,
+    device_width: HistoryDeviceWidth,
+    root_ring: RootRingDuringTheRemount,
+) -> SharedFaultPlan {
+    match root_ring {
+        RootRingDuringTheRemount::ReadAsUsual => {
+            SharedFaultPlan::unarmed(device_width.fixed_geometry())
+        }
+        RootRingDuringTheRemount::EverySlotOfTheDeviceFailsEveryRead(device) => {
+            let slots_per_region = pool.parameters.geometry.root_ring_slots_per_region.count();
+            let ring_slots_of_the_device: Vec<RootRingSlot> = (0..ROOT_RING_REGIONS)
+                .filter(|region| {
+                    pool.parameters.region_devices[usize::try_from(*region).expect("区域号")]
+                        == device
+                })
+                .flat_map(|region| {
+                    (0..slots_per_region).map(move |slot| RootRingSlot { region, slot })
+                })
+                .collect();
+            SharedFaultPlan::armed(
+                device_width.fixed_geometry(),
+                FaultSchedule::every_read_of_named_root_ring_slots_fails(RootRingSlotTarget {
+                    named_slots: NamedRootRingSlots::naming(&ring_slots_of_the_device),
+                    region_devices: pool.parameters.region_devices,
+                    fixed_structure_slot_spacing: pool
+                        .parameters
+                        .geometry
+                        .fixed_structure_slot_spacing,
+                }),
+            )
+        }
+    }
+}
+
+/// 再挂那一次的盘：外层按读注错计划注（根环读不出那一形），里层按写注错计划注（掉电那一形）。
+type RemountDevice = FaultInjectingBlockDevice<FaultInjectingBlockDevice<SparseBlockDevice>>;
+
 /// 再可写挂载走哪个入口：窄盘与攻方原型同一种开关（准入判着、影子账开、删除批大小 1），普通盘走 `crash_and_mount_writable` 那一个。
 #[derive(Clone, Copy, Debug)]
 enum RemountEntry {
@@ -146,10 +206,10 @@ enum RemountEntry {
     WithTheSpaceAdmissionJudged,
 }
 
-fn mount_writable_through(
+fn mount_writable_through<Device: BlockDevice>(
     entry: RemountEntry,
     pool: &PoolUnderTest<SparseBlockDevice>,
-    devices: &mut Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)>,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
 ) -> Result<Mounted, MountError> {
     match entry {
         RemountEntry::WithDeletionBatchesOfOne => {
@@ -174,23 +234,22 @@ struct LoneSurvivorAfterACrashedUnmountAndACrashedRemount {
     current_txg_before_the_unmount: CheckpointTxg,
     /// 剩下那块盘上恢复择到的根（看 journal）。
     root_chosen_on_the_survivor: Option<CheckpointTxg>,
+    /// 再挂那一次挂载期间读注错计划拦下的读（照常读的恒 0）。
+    reads_failed_during_the_remount: usize,
+    /// 再挂那一次挂载期间发下去的写调用（含掉电之后报错的那几次）。
+    write_calls_of_the_remount: u64,
     before_the_unmount: Result<WholeReadOfTheLiveFiles, String>,
     after_the_crashes: Result<WholeReadOfTheLiveFiles, String>,
 }
 
-/// 在 `pool` 的会话上卸载，前 `unmount_writes_landed` 个写落下就崩；崩溃镜像上两块盘都在、经 `remount_entry` 再可写挂载，
-/// 前 `remount_writes_landed` 个写落下又崩；丢掉 `lost_device`，卸载之前与两次崩溃之后各在剩下那块盘上整读一遍。
-fn lone_survivor_after_a_crashed_unmount_and_a_crashed_remount(
+/// 在 `pool` 的会话上卸载，前 `unmount_writes_landed` 个写落下就崩；交回崩溃之后两块盘的盘面。
+fn image_after_an_unmount_crashing_after(
     pool: &PoolUnderTest<SparseBlockDevice>,
     device_width: HistoryDeviceWidth,
     unmount_writes_landed: u64,
-    remount_entry: RemountEntry,
-    remount_writes_landed: u64,
-    lost_device: DeviceIdentity,
-) -> LoneSurvivorAfterACrashedUnmountAndACrashedRemount {
+) -> MemoryPool {
     let image_before_the_unmount = pool.image();
     let session = pool.session.as_ref().expect("这一刻有可写会话");
-    let current_txg_before_the_unmount = session.current.root().checkpoint_txg;
     let mut unmount_devices = devices_failing_every_write_from(
         &image_before_the_unmount,
         device_width,
@@ -211,15 +270,49 @@ fn lone_survivor_after_a_crashed_unmount_and_a_crashed_remount(
         unmount_writes_landed + 1,
         unmounted.as_ref().ok().map(|_| "做成了")
     );
-    let after_the_unmount = image_of(
+    image_of(
         &unmount_devices,
         image_before_the_unmount.device_size_in_bytes,
-    );
-    let mut remount_devices = devices_failing_every_write_from(
-        &after_the_unmount,
-        device_width,
-        remount_writes_landed + 1,
-    );
+    )
+}
+
+/// 在 `pool` 的会话上卸载，前 `unmount_writes_landed` 个写落下就崩；崩溃镜像上两块盘都在、经 `remount_entry` 再可写挂载（这次挂载期间根环照
+/// `root_ring_during_the_remount` 读），前 `remount_writes_landed` 个写落下又崩；丢掉 `lost_device`，卸载之前与两次崩溃之后各在剩下那块盘上整读一遍。
+fn lone_survivor_after_a_crashed_unmount_and_a_crashed_remount(
+    pool: &PoolUnderTest<SparseBlockDevice>,
+    device_width: HistoryDeviceWidth,
+    unmount_writes_landed: u64,
+    remount_entry: RemountEntry,
+    root_ring_during_the_remount: RootRingDuringTheRemount,
+    remount_writes_landed: u64,
+    lost_device: DeviceIdentity,
+) -> LoneSurvivorAfterACrashedUnmountAndACrashedRemount {
+    let image_before_the_unmount = pool.image();
+    let current_txg_before_the_unmount = pool
+        .session
+        .as_ref()
+        .expect("这一刻有可写会话")
+        .current
+        .root()
+        .checkpoint_txg;
+    let after_the_unmount =
+        image_after_an_unmount_crashing_after(pool, device_width, unmount_writes_landed);
+    let write_plan = plan_failing_every_write_from(device_width, remount_writes_landed + 1);
+    let read_plan = read_plan_of_the_remount(pool, device_width, root_ring_during_the_remount);
+    let mut remount_devices: Vec<(DeviceIdentity, RemountDevice)> =
+        plain_devices_on(&after_the_unmount)
+            .into_iter()
+            .map(|(identity, device)| {
+                (
+                    identity,
+                    FaultInjectingBlockDevice::new(
+                        identity,
+                        FaultInjectingBlockDevice::new(identity, device, write_plan.clone()),
+                        read_plan.clone(),
+                    ),
+                )
+            })
+            .collect();
     let remounted = mount_writable_through(remount_entry, pool, &mut remount_devices);
     assert!(
         remounted.is_err(),
@@ -228,18 +321,26 @@ fn lone_survivor_after_a_crashed_unmount_and_a_crashed_remount(
         remounted.as_ref().ok().map(|_| "做成了")
     );
     drop(remounted);
-    let survivor_after_the_crashes = image_without(
-        &image_of(
-            &remount_devices,
-            image_before_the_unmount.device_size_in_bytes,
-        ),
-        lost_device,
-    );
+    let after_the_remount = MemoryPool {
+        devices: remount_devices
+            .iter()
+            .map(|(identity, device)| {
+                (
+                    *identity,
+                    device.wrapped_device().wrapped_device().image.clone(),
+                )
+            })
+            .collect(),
+        device_size_in_bytes: image_before_the_unmount.device_size_in_bytes,
+    };
+    let survivor_after_the_crashes = image_without(&after_the_remount, lost_device);
     LoneSurvivorAfterACrashedUnmountAndACrashedRemount {
         current_txg_before_the_unmount,
         root_chosen_on_the_survivor: recover(&survivor_after_the_crashes, JournalPolicy::Consult)
             .effective_root
             .map(|(_, checkpoint_txg)| checkpoint_txg),
+        reads_failed_during_the_remount: read_plan.fired_count(),
+        write_calls_of_the_remount: write_plan.calls().writes,
         before_the_unmount: whole_read_of_the_live_files(&image_without(
             &image_before_the_unmount,
             lost_device,
@@ -342,6 +443,7 @@ fn after_a_crashed_unmount_of_a_narrow_pool_whose_deletion_was_continued_the_nex
             device_width,
             1,
             RemountEntry::WithDeletionBatchesOfOne,
+            RootRingDuringTheRemount::ReadAsUsual,
             6,
             DeviceIdentity(0),
         ),
@@ -369,8 +471,132 @@ fn after_a_crashed_unmount_of_a_four_gibibyte_pool_remounted_once_the_next_writa
             device_width,
             1,
             RemountEntry::WithTheSpaceAdmissionJudged,
+            RootRingDuringTheRemount::ReadAsUsual,
             33,
             DeviceIdentity(0),
         ),
     );
 }
+
+/// 攻方 `research/prompts/m3-batch16-code-r1-opus-output.md` 第 1.3 节那一段（判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节
+/// 「再挂期间某块盘根环整段读坏」那一行）：不补环窗口、准入关着造的窄盘（单元区 264 槽），第一个文件之后覆盖写 6 次、顺序写长到 4 个单元，
+/// 在这个进程里发 unlink 的意图就崩；重开可写（准入判着、影子账开、批大小 1）续做走完。卸载落下 15 个写就崩（两块盘的系统配置都带了新 F、
+/// 带新 F 的根一条都没落）；两块盘都在、再可写挂载，这次挂载期间盘 1 的根环每一槽每次读都报错（挂载之后读得出），落下 6 个写又崩
+/// （写行那次的单元写）；丢盘 0。造法照攻方原型 `research/prompts/m3-batch16-code-r1-opus-model/probe/main.rs` 的 `narrow_start` 与 `HIDE_DEV`
+/// （原型读回全 0，这里报读错；攻方量过两种结局同数）。
+/// 判别力：重建分配器时盘 1 一条有效根都读不出（m 取不到），照原门槛 max(F_生效, 窗口里最旧有效根) 整截当场放开时，写行那次的单元写进
+/// 盘 1 上最新的根（txg 19 < F_生效 20）还引用的映射树节点，盘 1 只读挂载走树读不出（攻方原型上量过
+/// `MappingStillUnreadable { slot: SlotNumber(1410) }`，另一刀 1412），最后一项红；当场只放到窗口里最旧有效根时读得出。
+#[test]
+fn after_a_crashed_unmount_of_a_narrow_pool_the_next_writable_mount_reading_no_root_on_device_one_and_crashing_in_its_row_publish_keeps_what_device_one_reads(
+) {
+    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
+    let overwrites_before_the_file_grows = 6;
+    let data_units_of_the_file = 4;
+    let mut pool =
+        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
+            device_width,
+            264,
+        );
+    for overwrite_index in 0..overwrites_before_the_file_grows {
+        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
+            panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
+        });
+    }
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
+        let current = session
+            .current
+            .version_of_trees()
+            .expect("第一个文件之后有树")
+            .clone();
+        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
+        publish_unlink_intent(
+            &mut writer,
+            &mut session.allocator,
+            &current,
+            InodeNumber(FIRST_INODE_NUMBER),
+            *write_time_seconds,
+            session.instance,
+        )
+        .expect("意图那次");
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
+    let remount_writes_landed = 6;
+    let survivor = lone_survivor_after_a_crashed_unmount_and_a_crashed_remount(
+        &pool,
+        device_width,
+        15,
+        RemountEntry::WithDeletionBatchesOfOne,
+        RootRingDuringTheRemount::EverySlotOfTheDeviceFailsEveryRead(DeviceIdentity(1)),
+        remount_writes_landed,
+        DeviceIdentity(0),
+    );
+    assert!(
+        survivor.reads_failed_during_the_remount > 0,
+        "再挂那一次挂载期间盘 1 的根环读真被拦下过"
+    );
+    assert!(
+        survivor.write_calls_of_the_remount > remount_writes_landed,
+        "再挂那一次挂载走到了第 {} 个写（掉电那一个），不是在任何写之前被拒：发下去的写调用 {}",
+        remount_writes_landed + 1,
+        survivor.write_calls_of_the_remount
+    );
+    assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(&survivor);
+}
+
+/// 暖机落满之后放开重建时扣住的那一截（`mount::establish_instance` 暖机之后那一句 `release_reclaim_holds`，判决
+/// `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节「扣住的那一截放开得对不对」那一行）：4 GiB 盘，第一个文件之后覆盖写 1 次、
+/// 崩了再挂 1 次、再覆盖写 1 次；卸载只落下第一个写就崩（盘 0 的系统配置带新 F = 现行那一版的 txg，盘 1 上最新的根比它低），两块盘都在、
+/// 再可写挂载（影子账关：没有隔离位）做成。挂载交回时分配器每块盘上的空闲槽数与三种位都没置的槽数相同：一个扣住位都没留下。
+/// 判别力：删掉暖机之后那一句放开，重建时释放代高于 m、不高于 F_生效 的那一截（上一次覆盖写换下的槽）留着扣住位交给会话，最后一项红。
+/// 用户看得到的结局没找到（推的，没量过）：扣住的槽准入照算可用、落点取不到时先推抬 F 那一串，那一串做完自己放开全部扣住位，写照样做成。
+#[test]
+fn after_a_crashed_unmount_of_a_four_gibibyte_pool_the_next_writable_mount_hands_over_no_held_slot_once_its_warm_up_landed_on_every_device(
+) {
+    let device_width = HistoryDeviceWidth::FourGibibytes;
+    let mut pool = start_plain(device_width);
+    pool.overwrite(OVERWRITE_BYTES)
+        .unwrap_or_else(|refusal| panic!("第一个文件之后覆盖写 1 次：{refusal:?}"));
+    pool.crash_and_mount_writable()
+        .unwrap_or_else(|error| panic!("崩了再挂：{error:?}"));
+    pool.overwrite(OVERWRITE_BYTES)
+        .unwrap_or_else(|refusal| panic!("再挂之后覆盖写 1 次：{refusal:?}"));
+    let after_the_unmount = image_after_an_unmount_crashing_after(&pool, device_width, 1);
+    let mut remount_devices = plain_devices_on(&after_the_unmount);
+    let mounted = mount_writable_with_test_only_switches(
+        &pool.parameters,
+        &mut remount_devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::Off,
+    )
+    .unwrap_or_else(|error| panic!("卸载半途崩溃之后两块盘都在、再可写挂载：{error:?}"));
+    let (_, session) = MountedSession::of_the_mount(mounted);
+    for device_map in &session.allocator.devices {
+        assert_eq!(
+            (device_map.isolated_slots(), device_map.free_slots()),
+            (0, device_map.unblocked_slots()),
+            "盘 {:?}：影子账关着没有隔离位；空闲的槽每一个都发得出去，重建时扣住的那一截在暖机落满之后全放开了",
+            device_map.device
+        );
+    }
+}
```

## 实现员 P：mutations-append.tsv

```
删除批第 17 批 甲：可写挂载重建分配器时某块盘一条有效根都读不出（m 取不到）照原门槛整截当场放开、不只放到一圈窗口里最旧有效根，再挂期间盘 1 根环每槽读错的窄盘（264 槽）丢盘 0 之后盘 1 读回	crates/singlefs-core/src/mount.rs	        None => oldest_valid_root_in_the_window,	        None => Some(floor_of_the_reclaim),	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads	after_a_crashed_unmount_of_a_narrow_pool_the_next_writable_mount_reading_no_root_on_device_one_and_crashing_in_its_row_publish_keeps_what_device_one_reads
删除批第 17 批 甲：可写挂载暖机在每块盘上落满之后不放开重建分配器时扣住的那一截（删掉 establish_instance 暖机之后那一句 release_reclaim_holds），挂载交回的分配器留着扣住位	crates/singlefs-core/src/mount.rs	    // 扣住的那一截不交给任何会话。\n    allocator.release_reclaim_holds();	    // 扣住的那一截不交给任何会话。	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads	after_a_crashed_unmount_of_a_four_gibibyte_pool_the_next_writable_mount_hands_over_no_held_slot_once_its_warm_up_landed_on_every_device
删除批第 17 批 甲：可写挂载重建分配器时扣住的那一遍回收改成当场放开（HeldUntilFloorTakesEffect 换成 Immediately，等于照原门槛整截当场放开），形 R h2 窄盘剩下盘 1 读回	crates/singlefs-core/src/mount.rs	    allocator.reclaim_released_up_to(\n        floor_of_the_reclaim,\n        ReclaimedReuse::HeldUntilFloorTakesEffect,\n    );	    allocator.reclaim_released_up_to(\n        floor_of_the_reclaim,\n        ReclaimedReuse::Immediately,\n    );	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads	after_a_crashed_unmount_of_a_narrow_pool_whose_deletion_was_continued_the_next_writable_mount_crashing_in_its_row_publish_keeps_what_device_one_reads
删除批第 17 批：第 16 批甲那一行（可写挂载重建分配器时当场放开不取 min(F_生效, m)）补点 4 GiB 那一条用例，形 R 普通盘剩下盘 1 读回	crates/singlefs-core/src/mount.rs	            effective_floor.min(newest_valid_root_on_every_device),	            effective_floor,	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads	after_a_crashed_unmount_of_a_four_gibibyte_pool_remounted_once_the_next_writable_mount_crashing_after_its_thirty_third_write_keeps_what_device_one_reads
```

## 实现员 Q：crates.patch（/tmp/claude-1000/m3-batch17/q/patch/crates.patch）

```diff
diff --git a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
--- a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
+++ b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
@@ -573,16 +573,49 @@
     );
 }
 
-/// 经会话发 unlink（`UserChange::Unlink`，会话把调用方给的时间交给意图那次）：给一个与这条记录上一版不同的时间，会话发成意图与三步；
-/// 意图那一版与还带这条记录的两批（留 2、留 1）各自两块盘上第一个文件那条记录的三个时间秒都等于给的时间。
-/// 上面那一条直接调 `publish_unlink_intent`，判不出会话这一层（`research/prompts/m3-batch14-code-r1-main-verification.md` 第二节 N3）。
-/// 判别力：会话那一层丢掉调用方的时间、意图写 0 时（攻方那一刀 MQ1）这一条红。
-#[test]
-fn a_session_unlink_writes_the_given_time_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk(
-) {
-    let mut pool = pool_with_a_three_unit_file("write-time-of-a-session-unlink");
-    let time_before_the_unlink = write_time_seconds_of_the_first_file(&pool);
-    let time_of_the_unlink = time_before_the_unlink + 600;
+/// 调用方给 unlink 的时间与这条记录上一版的写入时间差几秒。
+const SECONDS_BETWEEN_THE_GIVEN_TIME_AND_THE_PREVIOUS_VERSION: u64 = 600;
+
+/// 调用方给 unlink 的时间落在这条记录上一版写入时间的哪一边。Q-F 乙只说意图写调用方给的时间、各批照抄，没说时间只进不退；
+/// 早于上一版那一形判得出「意图取给的时间与上一版里较大的那个」（`research/prompts/m3-batch16-code-r1-main-verification.md` 第二节 N7），
+/// 晚于上一版那一形判不出。
+#[derive(Clone, Copy)]
+enum GivenTimeAgainstThePreviousVersion {
+    Later,
+    Earlier,
+}
+
+impl GivenTimeAgainstThePreviousVersion {
+    fn given_time(self, time_of_the_previous_version: u64) -> u64 {
+        match self {
+            Self::Later => time_of_the_previous_version
+                .checked_add(SECONDS_BETWEEN_THE_GIVEN_TIME_AND_THE_PREVIOUS_VERSION)
+                .expect("上一版的时间是测试池写的固定时间，加 600 秒不溢出"),
+            Self::Earlier => time_of_the_previous_version
+                .checked_sub(SECONDS_BETWEEN_THE_GIVEN_TIME_AND_THE_PREVIOUS_VERSION)
+                .expect("上一版的时间是测试池写的固定时间，大于 600 秒"),
+        }
+    }
+}
+
+/// 一次 unlink 给的时间、这条记录上一版的写入时间，与之后还带这条记录的几版两块盘上读回的三个时间秒（一版一项、一块盘一组）。
+struct TimeFieldsOnDiskAfterAnUnlink {
+    time_of_the_previous_version: u64,
+    given_time: u64,
+    on_disk_in_every_version_with_the_record: Vec<Vec<[u64; 3]>>,
+}
+
+/// 经会话发 unlink（`UserChange::Unlink`，会话把调用方给的时间交给意图那次）：给一个与这条记录上一版不同的时间（在上一版哪一边由
+/// `given_time_against_the_previous_version` 定），会话发成意图与三步；读意图那一版与还带这条记录的两批（留 2、留 1）各自两块盘上
+/// 第一个文件那条记录的三个时间秒。
+fn time_fields_on_disk_after_a_session_unlink(
+    pool_tag: &str,
+    given_time_against_the_previous_version: GivenTimeAgainstThePreviousVersion,
+) -> TimeFieldsOnDiskAfterAnUnlink {
+    let mut pool = pool_with_a_three_unit_file(pool_tag);
+    let time_of_the_previous_version = write_time_seconds_of_the_first_file(&pool);
+    let given_time =
+        given_time_against_the_previous_version.given_time(time_of_the_previous_version);
     let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
     let (_, mut session) = MountedSession::of_the_mount(mounted);
     let published = session
@@ -590,49 +623,122 @@
             &mut devices,
             UserChange::Unlink {
                 inode: FIRST_FILE,
-                write_time_seconds: time_of_the_unlink,
+                write_time_seconds: given_time,
             },
         )
         .expect("会话 unlink");
     drop(devices);
     let image = pool.memory_pool();
-    let on_disk_in_every_version_with_the_record: Vec<Vec<[u64; 3]>> = published
+    let on_disk_in_every_version_with_the_record = published
         .versions_published
         .iter()
         .filter(|version| version.inode_record.is_some())
         .map(|version| time_seconds_of_the_first_file_on_each_device(&image, version))
         .collect();
-    assert_eq!(
+    TimeFieldsOnDiskAfterAnUnlink {
+        time_of_the_previous_version,
+        given_time,
         on_disk_in_every_version_with_the_record,
-        vec![vec![[time_of_the_unlink; 3]; 2]; 3],
-        "意图那一版与留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都是调用方给会话的时间（上一版是 {time_before_the_unlink}）"
+    }
+}
+
+/// 给的时间晚于这条记录上一版：意图那一版与还带这条记录的两批（留 2、留 1）各自两块盘上第一个文件那条记录的三个时间秒都等于给的时间。
+/// 上面那一条直接调 `publish_unlink_intent`，判不出会话这一层（`research/prompts/m3-batch14-code-r1-main-verification.md` 第二节 N3）。
+/// 判别力：会话那一层丢掉调用方的时间、意图写 0 时（攻方那一刀 MQ1）这一条红。
+#[test]
+fn a_session_unlink_writes_the_given_time_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk(
+) {
+    let observed = time_fields_on_disk_after_a_session_unlink(
+        "write-time-of-a-session-unlink",
+        GivenTimeAgainstThePreviousVersion::Later,
+    );
+    assert_eq!(
+        observed.on_disk_in_every_version_with_the_record,
+        vec![vec![[observed.given_time; 3]; 2]; 3],
+        "意图那一版与留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都是调用方给会话的时间（上一版是 {}）",
+        observed.time_of_the_previous_version
     );
 }
 
-/// 意图落盘之后崩溃（进程内发完意图、一批没发就重开镜像），可写挂载续做：续做里还带这条记录的那两批（留 2、留 1）各自两块盘上
-/// 第一个文件那条记录的三个时间秒都等于意图那次给的时间。续做的几批没有调用方给时间，照抄从盘上读回的上一版这条记录的值
-/// （用户 2026-10-03 定 Q-F 乙「各批照抄」；D8（核心索引结构） 已定项 16 第 7 条 ①b）。
-/// 判别力：续做那几批另取时间（取 0、取一个固定时间），或挂载从盘上读回这条记录时丢了时间，这一条红。
+/// 给的时间早于这条记录上一版（上一版减 600 秒）：意图那一版与留 2、留 1 两批各自两块盘上那条记录的三个时间秒都等于给的那个较早的时间，
+/// 不取两者里较大的那个。
+/// 判别力：意图取 `write_time_seconds.max(record.write_time_seconds)`（`research/prompts/m3-batch16-code-r1-main-verification.md`
+/// 第二节 N7 攻方那一刀）时这一条红、晚于上一版那一条照旧绿。
 #[test]
-fn the_batches_of_a_mount_continuation_after_the_intent_copy_the_time_the_intent_wrote_on_disk() {
-    let mut pool = pool_with_a_three_unit_file("write-time-of-the-continuation-batches");
-    let time_before_the_unlink = write_time_seconds_of_the_first_file(&pool);
-    let time_of_the_intent = time_before_the_unlink + 600;
-    publish_the_intent_in_process_at(&mut pool, time_of_the_intent);
+fn a_session_unlink_given_a_time_earlier_than_the_previous_version_writes_that_earlier_time_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk(
+) {
+    let observed = time_fields_on_disk_after_a_session_unlink(
+        "earlier-write-time-of-a-session-unlink",
+        GivenTimeAgainstThePreviousVersion::Earlier,
+    );
+    assert_eq!(
+        observed.on_disk_in_every_version_with_the_record,
+        vec![vec![[observed.given_time; 3]; 2]; 3],
+        "意图那一版与留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都是调用方给会话的那个较早的时间（上一版是 {}）",
+        observed.time_of_the_previous_version
+    );
+}
+
+/// 进程内发意图（给一个与这条记录上一版不同的时间，在上一版哪一边由 `given_time_against_the_previous_version` 定），一批没发就重开镜像
+/// （意图落盘之后崩溃），可写挂载续做；读续做里还带这条记录的那两批（留 2、留 1）各自两块盘上第一个文件那条记录的三个时间秒。
+fn time_fields_on_disk_in_the_continuation_batches_after_an_intent(
+    pool_tag: &str,
+    given_time_against_the_previous_version: GivenTimeAgainstThePreviousVersion,
+) -> TimeFieldsOnDiskAfterAnUnlink {
+    let mut pool = pool_with_a_three_unit_file(pool_tag);
+    let time_of_the_previous_version = write_time_seconds_of_the_first_file(&pool);
+    let given_time =
+        given_time_against_the_previous_version.given_time(time_of_the_previous_version);
+    publish_the_intent_in_process_at(&mut pool, given_time);
     let (mounted, devices) = remount_with_batches_of_one(&mut pool);
     drop(devices);
     let image = pool.memory_pool();
-    let on_disk_in_every_continuation_batch_with_the_record: Vec<Vec<[u64; 3]>> = mounted
+    let on_disk_in_every_version_with_the_record = mounted
         .output
         .deletion_queue_continuation
         .iter()
         .filter(|step| step.inode_record.is_some())
         .map(|step| time_seconds_of_the_first_file_on_each_device(&image, step))
         .collect();
+    TimeFieldsOnDiskAfterAnUnlink {
+        time_of_the_previous_version,
+        given_time,
+        on_disk_in_every_version_with_the_record,
+    }
+}
+
+/// 意图给的时间晚于这条记录上一版：续做里还带这条记录的那两批（留 2、留 1）各自两块盘上第一个文件那条记录的三个时间秒都等于意图那次给的时间。
+/// 续做的几批没有调用方给时间，照抄从盘上读回的上一版这条记录的值（用户 2026-10-03 定 Q-F 乙「各批照抄」；D8（核心索引结构） 已定项 16 第 7 条 ①b）。
+/// 判别力：续做那几批另取时间（取 0、取一个固定时间），或挂载从盘上读回这条记录时丢了时间，这一条红。
+#[test]
+fn the_batches_of_a_mount_continuation_after_the_intent_copy_the_time_the_intent_wrote_on_disk() {
+    let observed = time_fields_on_disk_in_the_continuation_batches_after_an_intent(
+        "write-time-of-the-continuation-batches",
+        GivenTimeAgainstThePreviousVersion::Later,
+    );
+    assert_eq!(
+        observed.on_disk_in_every_version_with_the_record,
+        vec![vec![[observed.given_time; 3]; 2]; 2],
+        "续做的留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都照抄意图那次给的时间（上一版是 {}）",
+        observed.time_of_the_previous_version
+    );
+}
+
+/// 意图给的时间早于这条记录上一版（上一版减 600 秒）：续做的留 2、留 1 两批各自两块盘上那条记录的三个时间秒都照抄意图写下的那个较早的时间。
+/// 判别力：意图取 `write_time_seconds.max(record.write_time_seconds)`（`research/prompts/m3-batch16-code-r1-main-verification.md`
+/// 第二节 N7 攻方那一刀）时意图写下的是上一版的时间、续做照抄它，这一条红、晚于上一版那一条照旧绿。
+#[test]
+fn the_batches_of_a_mount_continuation_after_an_intent_given_a_time_earlier_than_the_previous_version_copy_that_earlier_time_on_disk(
+) {
+    let observed = time_fields_on_disk_in_the_continuation_batches_after_an_intent(
+        "earlier-write-time-of-the-continuation-batches",
+        GivenTimeAgainstThePreviousVersion::Earlier,
+    );
     assert_eq!(
-        on_disk_in_every_continuation_batch_with_the_record,
-        vec![vec![[time_of_the_intent; 3]; 2]; 2],
-        "续做的留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都照抄意图那次给的时间（上一版是 {time_before_the_unlink}）"
+        observed.on_disk_in_every_version_with_the_record,
+        vec![vec![[observed.given_time; 3]; 2]; 2],
+        "续做的留 2、留 1 两批，两块盘上那条记录的 atime、mtime、ctime 秒都照抄意图写下的那个较早的时间（上一版是 {}）",
+        observed.time_of_the_previous_version
     );
 }
 
```

## 实现员 Q：mutations-append.tsv

```
删除批第 17 批 N7：unlink 的意图取给的时间与这条记录上一版里较大的那个（时间只进不退，攻方 n7-clamp），会话那一形给早于上一版的时间	crates/singlefs-core/src/transaction.rs	                change_count: change_count_of_the_intent,\n                write_time_seconds,	                change_count: change_count_of_the_intent,\n                write_time_seconds: write_time_seconds.max(record.write_time_seconds),	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version	a_session_unlink_given_a_time_earlier_than_the_previous_version_writes_that_earlier_time_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk
删除批第 17 批 N7：unlink 的意图取给的时间与这条记录上一版里较大的那个（时间只进不退，攻方 n7-clamp），续做那一形给早于上一版的时间	crates/singlefs-core/src/transaction.rs	                change_count: change_count_of_the_intent,\n                write_time_seconds,	                change_count: change_count_of_the_intent,\n                write_time_seconds: write_time_seconds.max(record.write_time_seconds),	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version	the_batches_of_a_mount_continuation_after_an_intent_given_a_time_earlier_than_the_previous_version_copy_that_earlier_time_on_disk
```

