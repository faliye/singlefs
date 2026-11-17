# m4-gpugc-floor-raise-precheck-code-r1：被判的产品代码改动（第 119 批补丁，`crates.patch` 里 `src/` 下、验证包之外的段）

```diff
diff --git a/crates/singlefs-core/src/compaction_plan.rs b/crates/singlefs-core/src/compaction_plan.rs
index 2f692ce91d5b40fb011b58a5d22bb03ca41b14d9..a19fecca2b7ea40d08bee75019b6e3576238a434 100644
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -1545,6 +1545,7 @@ pub trait CallerOfTheBackgroundCompaction {
 /// 产品批大小 16 下一条整段意图（至多 `1 + ⌈64 ÷ 16⌉ = 5` 次发布）走得完；S = 4 时是 4 次，走不完就停在门上报出来。
 /// 下一次可写挂载自己的发布不经门：写行加暖机至多 3 次，续做一条意图至多 5 次，合起来至多 8 次。S = 8 时一次挂载落在余量里；S 更小时一次挂载就可能用完余量。
 /// 连崩几次用完这几槽之后，挂载自己的发布照样挤掉用户状态。
+/// 这笔算术不算 `accept` 被空间拒时当场推的抬 F 那几串（`mounted_session::MountedSessionWithPendingChanges::accept`）：它们不经门，推一串也用掉这几槽。
 #[must_use]
 pub fn root_ring_slots_left_by_a_compaction_publish(
     root_ring_slots_per_region: RootRingSlotsPerRegion,
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index ef6b82d5a3508c49a861028e72e4743e7d874fd5..a805a397e1780058f995d48d23b6ad831ccb0cfd 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -3,7 +3,8 @@
 //! 发布路径报空间准入不够（`PublishError::SpaceAdmissionRefused`）、或准入放行而落点取不到（`PublishError::PlacementRefused`）时，
 //! 先推一串抬 F 的空发布（`mount::push_one_floor_raise_within_the_admission_budget`：F 抬到准入上限；上限卡在挂载时读坏或读回全零的槽上时
 //! 推一次不抬 F 的空发布，D16（发布语义） 已定项 1「抬 F 的上限」那一行；一次准入最多 `mount::PUBLISHES_PER_ADMISSION_AT_MOST` 次发布，
-//! 两种推的都算、再算上这次用户发布），再重判；做满仍不够、或抬不动了，才报空间不够。
+//! 两种推的都算、再算上这次用户发布），再重判；做满仍不够、或抬不动了，才报空间不够。推抬 F 放得出的全放出来也放不下时不推，当场报空间不够
+//! （`UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`）。
 //! 写行那次发布之前不推：会话在可写挂载交回之后才有，写行那次早已发完（可写挂载那一处的推归 `mount::establish_instance`）。
 //!
 //! 为什么要这一层：发布路径（`transaction::publish_overwrite` 这一族）只拿到一个写入口（`PoolWriter`），拿不到抬 F 要的设备表与参数
@@ -26,11 +27,16 @@ use std::time::Duration;
 use singlefs_format::{CLUSTER_SEGMENT_SLOTS, SLOT_BYTES};
 
 use crate::address::{
-    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
-    InstanceGeneration, SlotNumber, TreeIdentifier,
+    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes,
+    InodeNumber, InstanceGeneration, SlotNumber, TreeIdentifier,
+};
+use crate::admission::{
+    AdmissionRefusedOnSomeDevices, AvailableBytesOnOneDevice, BytesOnOneDevice,
+    DeviceShortOfDemand, JudgementThatFoundTheDeviceShort,
+};
+use crate::allocator::{
+    AllocationRecord, DeviceFreeMap, Placement, PlacementOnDevice, PoolAllocator,
 };
-use crate::admission::AdmissionRefusedOnSomeDevices;
-use crate::allocator::{AllocationRecord, Placement, PlacementOnDevice, PoolAllocator};
 use crate::background_compute::{
     decisions_of_one_segment, AlarmOfABackgroundTaskStoppedAfterAComputeError,
     AllocatedBytesAndCapacityOfADevice, BackgroundComputeError, BackgroundComputeOfOneBatch,
@@ -620,6 +626,11 @@ pub enum UserChangeRefused {
     /// 空间不够（报给用户就是 ENOSPC），推过的与不再推的原因在 [`NoSpaceAfterRaisingTheFloor`] 里。
     /// 装箱：不装箱 `UserChangeRefused` 就大过 clippy `result_large_err` 的 128 字节（`MountError` 那几个成员同一个做法）。
     NoSpaceAfterRaisingTheFloor(Box<NoSpaceAfterRaisingTheFloor>),
+    /// 空间不够（报给用户就是 ENOSPC），而且推抬 F 放得出的全放出来也放不下：这一次不再推，[`NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`] 带上界与需求。
+    /// 这一次判之前已经推成的串、发成的版在里面，头一次被拒就判到这里时它们是空的，盘上逐字节不变。装箱同 `NoSpaceAfterRaisingTheFloor`。
+    NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(
+        Box<NoSpaceEvenWithEverythingAFloorRaiseCouldRelease>,
+    ),
     /// 发布被拒是空间不够、为它推一串抬 F 时抬 F 自己报了不是空间不够的错（块设备错、读不出、预演被别的理由拒……）：不是「推满仍不够」，
     /// 不报空间不够（代码审阅第 23 条），错原样在 [`FloorRaiseFailedWhilePushingForSpace`] 里。抬 F 那一串自己也取不到落点、被空间准入拒的
     /// 是推满仍不够，走 `NoSpaceAfterRaisingTheFloor`。装箱同 `NoSpaceAfterRaisingTheFloor`。
@@ -643,6 +654,29 @@ pub struct NoSpaceAfterRaisingTheFloor {
     pub wide_protection_at_the_ceiling: WideProtectionAtTheCeiling,
 }
 
+/// 推抬 F 放得出的全放出来也放不下：`devices_short` 里每块盘的上界都小于需求，所以这一次不推（D16（发布语义） 已定项 1「准入」那一行的射程：
+/// 推了也写不进的写不花根环的槽与写盘）。别的被拒成员一样带着这次调用已经落盘的：`floor_raises` 与 `refusals_that_pushed_the_floor_raises`
+/// 一一对应，`versions_published` 是推的抬 F 之外发成的那几版（[`VersionsPublishedInThisChange`]）。
+#[derive(Debug)]
+pub struct NoSpaceEvenWithEverythingAFloorRaiseCouldRelease {
+    pub devices_short: Vec<DeviceShortEvenWithEverythingAFloorRaiseCouldRelease>,
+    /// 判出这几块盘的那一次被拒（`SpaceAdmissionRefused`）：这一次发布没有推抬 F。
+    pub last_refusal: PublishError,
+    pub floor_raises: Vec<RaisedFloor>,
+    pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
+    pub versions_published: VersionsPublishedInThisChange,
+}
+
+/// 一块放不下的盘：`judged_short_by` 是判它不够的那一判，`most_a_floor_raise_could_make_available` 与 `demand` 都按那一判自己的量
+/// （[`JudgementThatFoundTheDeviceShort`] 各成员的文档）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
+    pub device: DeviceIdentity,
+    pub judged_short_by: JudgementThatFoundTheDeviceShort,
+    pub most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice,
+    pub demand: BytesOnOneDevice,
+}
+
 /// 推满仍不够时，挡着准入的是不是「这个进程写掉那一槽才放」的那一份隔离（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」：
 /// 不自动推环，准入因宽护卡住时报一个点名「被宽护卡住」的错、带上出路；用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」：
 /// 推环那一条出路预演过、推完这次改动放得下才列；同日弹窗定「改：预演延到要出路时才做」：报的时候不预演，调用方要出路时另调
@@ -2851,6 +2885,7 @@ impl MountedSession {
     /// unlink 发成意图之后接着把它的全部批走完。走空里的删除类发布（不判式子那一判、落得下那一判照判，D28（挂载期承诺量） 已定项 1）
     /// 被拒是空间不够时同样推一串抬 F、回收之后接着走（用户 2026-10-02 定案 Q-V「推抬 F 再试」，每一次被拒的删除类发布各算一次准入），
     /// 推满仍不够交回 `NoSpaceAfterRaisingTheFloor`；推的那几串与意图那次推的一起报在 `floor_raises` 里。
+    /// 推抬 F 放得出的全放出来也放不下时不推，交回 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`（`push_one_floor_raise_for_a_refusal_for_space`）。
     /// 删除那几次发布的两个只供测试的开关取分配器上装的（可写挂载的开关链装上去的，`unlink::DeletionSwitches`）。
     /// 每发成一次（走空的每一步、这次改动）`current` 都按 `PoolVersion::of_a_version_of_trees` 重新归成员；现行那一版是有树而文件集合为空的
     /// 一版时只放建 inode（D8（核心索引结构） 已定项 16 第 5 条），别的改动拒成 `NoFileVersionToChange`。
@@ -5773,8 +5808,125 @@ struct PublishedInThisChange {
     versions_published: VersionsPublishedInThisChange,
 }
 
+/// 一块盘上抬 F 至多放得开的槽，都是分配器增量维护的数（`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」：准入路径上不扫单元区）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct SlotsAFloorRaiseCouldRelease {
+    /// defer 窗口里全部已释放的槽：只算槽数、不问这一串的上限够不够得到它们。
+    deferred: u64,
+    /// 隔离着的槽（式子第九项）：被抛弃根离开根环、C 那一圈解开时回来，推环这条出路也靠它们（[`StuckBehindTheWideProtection`]）。
+    isolated: u64,
+    /// 计数上空闲而位上挡着的槽：隔离着没分配的与抬 F 扣住的（`DeviceFreeMap::hold_until_floor_takes_effect`）。
+    /// 式子本来不扣扣住的槽，落得下那一判里它们挡着槽对。
+    free_but_blocked: u64,
+}
+
+impl SlotsAFloorRaiseCouldRelease {
+    fn of_the_device(device_map: &DeviceFreeMap) -> Self {
+        Self {
+            deferred: device_map.deferred_slots(),
+            isolated: device_map.isolated_slots(),
+            free_but_blocked: device_map
+                .free_slots()
+                .checked_sub(device_map.unblocked_slots())
+                .expect("没挡的槽都没置分配位，分配位没置的都算空闲：没挡的槽数不大于空闲槽数"),
+        }
+    }
+}
+
+/// 一块被判短的盘推抬 F 至多凑得出多少，按判它短的那一判自己的量（宁大勿小：上界算小了会误拒一个推了本来写得进的写）：
+/// 式子那一判是可用加 defer 与隔离的字节；段外槽对那一判是现有的槽对加每个放得开的槽各补成一对（另一半本来就空着时放开一个槽就多一对，所以按一个数据单元的槽数换算）。
+/// 提交内生块那一判交 `None`：这一判数的槽抬 F 那一串自己也要占，不在这里算。
+fn most_a_floor_raise_could_make_available(
+    short: &DeviceShortOfDemand,
+    slots: SlotsAFloorRaiseCouldRelease,
+) -> Option<AvailableBytesOnOneDevice> {
+    let bytes_of_slots = |count: u64| i128::from(BytesOnOneDevice::of_slots(count).0);
+    match short.judged_short_by {
+        JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand => {
+            Some(AvailableBytesOnOneDevice(
+                short.available.0 + bytes_of_slots(slots.deferred) + bytes_of_slots(slots.isolated),
+            ))
+        }
+        JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits => {
+            let slots_that_could_complete_a_pair =
+                slots.deferred + slots.isolated + slots.free_but_blocked;
+            let slots_of_a_data_unit =
+                TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots();
+            Some(AvailableBytesOnOneDevice(
+                short.available.0
+                    + bytes_of_slots(slots_that_could_complete_a_pair * slots_of_a_data_unit),
+            ))
+        }
+        JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits => None,
+    }
+}
+
+/// 这块被判短的盘推抬 F 全放出来也放不下时交回它的上界与需求；上界恰好等于需求算放得下，判不准的那一判（[`most_a_floor_raise_could_make_available`] 交 `None`）也算放得下。
+fn device_short_even_with_everything_a_floor_raise_could_release(
+    short: &DeviceShortOfDemand,
+    slots: SlotsAFloorRaiseCouldRelease,
+) -> Option<DeviceShortEvenWithEverythingAFloorRaiseCouldRelease> {
+    let upper_bound = most_a_floor_raise_could_make_available(short, slots)?;
+    (upper_bound.0 < i128::from(short.demand.0)).then_some(
+        DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
+            device: short.device,
+            judged_short_by: short.judged_short_by,
+            most_a_floor_raise_could_make_available: upper_bound,
+            demand: short.demand,
+        },
+    )
+}
+
+/// 这一次被拒里全放出来也放不下的盘，按拒绝里的次序。只读分配器（内存）。只有 `SpaceAdmissionRefused` 判得出需求，`PlacementRefused` 交空的、照旧推。
+fn devices_short_even_with_everything_a_floor_raise_could_release(
+    allocator: &PoolAllocator,
+    refusal: &PublishError,
+) -> Vec<DeviceShortEvenWithEverythingAFloorRaiseCouldRelease> {
+    let PublishError::SpaceAdmissionRefused(refused) = refusal else {
+        return Vec::new();
+    };
+    refused
+        .short_devices
+        .iter()
+        .filter_map(|short| {
+            let device_map = allocator
+                .devices
+                .iter()
+                .find(|device_map| device_map.device == short.device)
+                .expect("拒绝里的盘来自这个分配器：准入的两份读数都按它的盘建");
+            device_short_even_with_everything_a_floor_raise_could_release(
+                short,
+                SlotsAFloorRaiseCouldRelease::of_the_device(device_map),
+            )
+        })
+        .collect()
+}
+
+/// 全放出来也放不下的那个成员，把这次调用已经落盘的都带上。
+fn no_space_even_with_everything_a_floor_raise_could_release(
+    devices_short: Vec<DeviceShortEvenWithEverythingAFloorRaiseCouldRelease>,
+    last_refusal: PublishError,
+    published_in_this_change: PublishedInThisChange,
+) -> UserChangeRefused {
+    let PublishedInThisChange {
+        floor_raises,
+        refusals_that_pushed_the_floor_raises,
+        versions_published,
+    } = published_in_this_change;
+    UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(Box::new(
+        NoSpaceEvenWithEverythingAFloorRaiseCouldRelease {
+            devices_short,
+            last_refusal,
+            floor_raises,
+            refusals_that_pushed_the_floor_raises,
+            versions_published,
+        },
+    ))
+}
+
 /// 一次发布被拒之后会话怎么接（D16（发布语义） 已定项 1「准入」那一行）：不是空间不够的那两种（[`refusal_is_short_of_space`]）原样交回 `Publish`；
-/// 是就为被拒的那一次发布推一串抬 F 的空发布（这次准入已推 `publishes_pushed_in_this_admission` 次，预算照
+/// 推抬 F 放得出的全放出来也放不下（[`devices_short_even_with_everything_a_floor_raise_could_release`]）交回 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，一个写都不发；
+/// 别的就为被拒的那一次发布推一串抬 F 的空发布（这次准入已推 `publishes_pushed_in_this_admission` 次，预算照
 /// `mount::push_one_floor_raise_within_the_admission_budget`），推成交回记上这一串之后的 `published_in_this_change` 与这一串推了几次空发布，
 /// 不再推交回 `NoSpaceAfterRaisingTheFloor`（报不报「被宽护卡住」，报的话记下这次改动剩下的 `remainder`，要出路时照它预演，
 /// [`wide_protection_at_the_ceiling`]），抬 F 自己报错交回 `FloorRaiseFailedWhilePushingForSpace`。
@@ -5795,6 +5947,15 @@ fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
     published_in_this_change: PublishedInThisChange,
     refusal: PublishError,
 ) -> Result<(PublishedInThisChange, usize), UserChangeRefused> {
+    let devices_short =
+        devices_short_even_with_everything_a_floor_raise_could_release(allocator, &refusal);
+    if !devices_short.is_empty() {
+        return Err(no_space_even_with_everything_a_floor_raise_could_release(
+            devices_short,
+            refusal,
+            published_in_this_change,
+        ));
+    }
     let PublishedInThisChange {
         mut floor_raises,
         mut refusals_that_pushed_the_floor_raises,
@@ -6309,6 +6470,7 @@ pub enum AcceptRefused {
     PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(Box<PublishError>),
     /// `judgement` 那一批被空间拒，当场推抬 F 再判（D16（发布语义） 已定项 1「准入」那一行）之后还是没收下，`refusal` 原样：
     /// [`UserChangeRefused::NoSpaceAfterRaisingTheFloor`] 是推到不再推了仍放不下（一次准入的发布数做满、F 已在上限、抬 F 那一串自己放不下，一串都没推成时 `floor_raises` 是空的）；
+    /// [`UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`] 是推抬 F 放得出的全放出来也放不下，这一次不再推（头一次被拒就判到这里时 `floor_raises` 是空的）；
     /// [`UserChangeRefused::FloorRaiseFailedWhilePushingForSpace`] 是抬 F 自己报了错（写盘报错时那一次空发布冻结）；
     /// [`UserChangeRefused::Publish`] 是推过之后再判拒成了别的原因。推成的那几串已落盘，现行那一版在它们后面（[`Self::floor_raises_pushed_by_this_accept`]）。
     /// 这一次改动没收进待发；待发里已有的原样留着，冻结的那一次空发布不含它们。
@@ -6341,6 +6503,9 @@ impl AcceptRefused {
                 UserChangeRefused::NoFileVersionToChange { floor_raises, .. }
                 | UserChangeRefused::Publish { floor_raises, .. } => floor_raises,
                 UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) => &no_space.floor_raises,
+                UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) => {
+                    &no_space.floor_raises
+                }
                 UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(failed) => {
                     &failed.floor_raises
                 }
@@ -7886,6 +8051,252 @@ mod tests {
     }
 }
 
+#[cfg(test)]
+mod everything_a_floor_raise_could_release_tests {
+    use super::*;
+    use crate::mount::FloorOfThePushedPublishes;
+    use crate::write_accounting::WritesByStructureKind;
+    use singlefs_format::SLOT_BYTES;
+
+    const DEVICE_OF_THE_TEST: DeviceIdentity = DeviceIdentity(0);
+
+    fn no_slots() -> SlotsAFloorRaiseCouldRelease {
+        SlotsAFloorRaiseCouldRelease {
+            deferred: 0,
+            isolated: 0,
+            free_but_blocked: 0,
+        }
+    }
+
+    fn short_by(
+        judged_short_by: JudgementThatFoundTheDeviceShort,
+        available_in_bytes: i128,
+        demand_in_bytes: u64,
+    ) -> DeviceShortOfDemand {
+        DeviceShortOfDemand {
+            device: DEVICE_OF_THE_TEST,
+            judged_short_by,
+            available: AvailableBytesOnOneDevice(available_in_bytes),
+            demand: BytesOnOneDevice(demand_in_bytes),
+        }
+    }
+
+    /// 式子那一判：可用是负的、defer 两槽、隔离一槽。需求恰好等于三样之和。
+    const FORMULA_AVAILABLE_IN_BYTES: i128 = -100;
+
+    fn formula_demand_equal_to_the_bound() -> u64 {
+        3 * SLOT_BYTES - 100
+    }
+
+    fn formula_slots() -> SlotsAFloorRaiseCouldRelease {
+        SlotsAFloorRaiseCouldRelease {
+            deferred: 2,
+            isolated: 1,
+            free_but_blocked: 0,
+        }
+    }
+
+    #[test]
+    fn a_formula_demand_exactly_covered_by_available_deferred_and_isolated_bytes_is_still_pushed_for(
+    ) {
+        assert_eq!(
+            device_short_even_with_everything_a_floor_raise_could_release(
+                &short_by(
+                    JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand,
+                    FORMULA_AVAILABLE_IN_BYTES,
+                    formula_demand_equal_to_the_bound(),
+                ),
+                formula_slots(),
+            ),
+            None,
+            "上界恰好等于需求：放得下，照推"
+        );
+    }
+
+    #[test]
+    fn a_formula_demand_one_byte_above_the_bound_is_named_with_the_bound_and_the_demand() {
+        assert_eq!(
+            device_short_even_with_everything_a_floor_raise_could_release(
+                &short_by(
+                    JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand,
+                    FORMULA_AVAILABLE_IN_BYTES,
+                    formula_demand_equal_to_the_bound() + 1,
+                ),
+                formula_slots(),
+            ),
+            Some(DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
+                device: DEVICE_OF_THE_TEST,
+                judged_short_by: JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand,
+                most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice(i128::from(
+                    formula_demand_equal_to_the_bound()
+                )),
+                demand: BytesOnOneDevice(formula_demand_equal_to_the_bound() + 1),
+            }),
+            "差一个字节：点名这块盘，带上界与需求"
+        );
+    }
+
+    #[test]
+    fn the_formula_bound_does_not_count_slots_the_formula_never_deducted() {
+        assert!(
+            device_short_even_with_everything_a_floor_raise_could_release(
+                &short_by(
+                    JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand,
+                    0,
+                    1,
+                ),
+                SlotsAFloorRaiseCouldRelease {
+                    deferred: 0,
+                    isolated: 0,
+                    free_but_blocked: 8,
+                },
+            )
+            .is_some(),
+            "扣住的槽式子本来不扣：加进上界就多算"
+        );
+    }
+
+    /// 段外槽对那一判：一个槽对都没有，需求一个数据单元，这一样放得开的只有一个槽：放开它就可能多一对。
+    fn one_data_unit_short_on_the_slot_pairs_is_pushed_for_with(
+        slots: SlotsAFloorRaiseCouldRelease,
+    ) -> bool {
+        let bytes_of_a_data_unit = BytesOnOneDevice::of_slots(
+            TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots(),
+        )
+        .0;
+        device_short_even_with_everything_a_floor_raise_could_release(
+            &short_by(
+                JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits,
+                0,
+                bytes_of_a_data_unit,
+            ),
+            slots,
+        )
+        .is_none()
+    }
+
+    #[test]
+    fn one_deferred_slot_can_complete_a_slot_pair_for_one_data_unit() {
+        assert!(one_data_unit_short_on_the_slot_pairs_is_pushed_for_with(
+            SlotsAFloorRaiseCouldRelease {
+                deferred: 1,
+                isolated: 0,
+                free_but_blocked: 0,
+            }
+        ));
+    }
+
+    #[test]
+    fn one_isolated_slot_can_complete_a_slot_pair_for_one_data_unit() {
+        assert!(one_data_unit_short_on_the_slot_pairs_is_pushed_for_with(
+            SlotsAFloorRaiseCouldRelease {
+                deferred: 0,
+                isolated: 1,
+                free_but_blocked: 0,
+            }
+        ));
+    }
+
+    #[test]
+    fn one_free_but_blocked_slot_can_complete_a_slot_pair_for_one_data_unit() {
+        assert!(one_data_unit_short_on_the_slot_pairs_is_pushed_for_with(
+            SlotsAFloorRaiseCouldRelease {
+                deferred: 0,
+                isolated: 0,
+                free_but_blocked: 1,
+            }
+        ));
+    }
+
+    #[test]
+    fn no_slot_to_release_leaves_a_device_short_on_the_slot_pairs_named() {
+        assert!(!one_data_unit_short_on_the_slot_pairs_is_pushed_for_with(
+            no_slots()
+        ));
+    }
+
+    #[test]
+    fn a_device_short_on_the_slots_for_commit_generated_units_is_always_pushed_for() {
+        assert_eq!(
+            device_short_even_with_everything_a_floor_raise_could_release(
+                &short_by(
+                    JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits,
+                    0,
+                    SLOT_BYTES,
+                ),
+                no_slots(),
+            ),
+            None,
+            "这一判不算上界：照推"
+        );
+    }
+
+    fn raised_floor_without_publishes() -> RaisedFloor {
+        RaisedFloor {
+            ceiling: CheckpointTxg(7),
+            system_configuration_writes_before_the_first_publish:
+                WritesByStructureKind::NOTHING_WRITTEN,
+            publishes: Vec::new(),
+            floor_of_the_publishes: FloorOfThePushedPublishes::RaisedToTheCeiling,
+            reclaimed: Vec::new(),
+            abandoned_roots_unreadable: 0,
+        }
+    }
+
+    fn refusal_of_the_test() -> PublishError {
+        PublishError::SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices {
+            short_devices: vec![short_by(
+                JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand,
+                0,
+                SLOT_BYTES,
+            )],
+        })
+    }
+
+    fn refusal_after_one_series_was_pushed() -> UserChangeRefused {
+        no_space_even_with_everything_a_floor_raise_could_release(
+            Vec::new(),
+            refusal_of_the_test(),
+            PublishedInThisChange {
+                floor_raises: vec![raised_floor_without_publishes()],
+                refusals_that_pushed_the_floor_raises: vec![refusal_of_the_test()],
+                versions_published: Vec::new(),
+            },
+        )
+    }
+
+    /// 第二轮才判到放不下时前面已推过串：丢了它们，模型对拍与调用方都少一串已落盘的。
+    #[test]
+    fn the_refusal_keeps_the_series_already_pushed_and_the_refusals_that_pushed_them() {
+        let UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) =
+            refusal_after_one_series_was_pushed()
+        else {
+            panic!("交回的就是全放出来也放不下那个成员");
+        };
+        assert_eq!(
+            (
+                no_space.floor_raises.len(),
+                no_space.refusals_that_pushed_the_floor_raises.len(),
+            ),
+            (1, 1),
+            "前面推成的那一串与推它的那一次被拒都带着"
+        );
+    }
+
+    #[test]
+    fn the_accept_entry_reports_the_series_kept_by_the_refusal() {
+        let refused = AcceptRefused::RefusedAfterRaisingTheFloorForSpace {
+            judgement: BatchJudgedAtTheAccept::ThisChangeAlone,
+            refusal: Box::new(refusal_after_one_series_was_pushed()),
+        };
+        assert_eq!(
+            refused.floor_raises_pushed_by_this_accept().len(),
+            1,
+            "`accept` 这个入口报拒绝里带的那一串"
+        );
+    }
+}
+
 #[cfg(test)]
 mod old_data_unit_reads_back_with_one_copy_failing_in_this_change_tests {
     use super::OldDataUnitReadsBackWithOneCopyFailingInThisChange;
diff --git a/crates/singlefs-mount-runtime/src/lib.rs b/crates/singlefs-mount-runtime/src/lib.rs
index 9193461986a35b771a8efcb2d4ca69ac428883ef..ef60936a7ecbf14737ac88c1c84a739538a5092d 100644
--- a/crates/singlefs-mount-runtime/src/lib.rs
+++ b/crates/singlefs-mount-runtime/src/lib.rs
@@ -177,6 +177,8 @@ pub enum MaintenanceEvent<'event> {
     /// 冻结着的那一次这一拍重发又报错：照旧冻结，前台的写照旧被拒（`AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet`），整理这一拍不跑。
     FrozenPublishResendFailed(&'event PublishError),
     /// 前台 [`MountedRuntime::accept`] 这一次调用为空间推成了这几串抬 F 的空发布（收没收下都报）：调用方看不到发布点的接收方靠它知道根环走了几格。
+    /// 这几种串不报，根环照样走了：屏障进门排空待发与屏障自己发布时推的串（在 `AcceptRefused` 的屏障那两个成员或那几次发布的报里）、
+    /// 收下之后触发的那一次发布自己推的串、抬 F 半路报错那一串里已落盘的那几次（`FloorRaiseFailedWhilePushingForSpace` 的 `floor_raises` 不含它们）。
     FloorRaisesPushedByAnAccept(&'event [RaisedFloor]),
     /// 维护线程 panic 了：等价于崩溃，运行库从此报 [`RuntimeFailed`]。
     MaintenanceThreadPanicked { description: &'event str },
```
