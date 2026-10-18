# m3-b44-code-r3 被判的改动（2026-10-18）

`crates/singlefs-core/src/` 下 `block_device.rs`、`deleted_inodes.rs`、`mounted_session.rs`、`mount.rs`、`recovery.rs`、`transaction.rs`、`unit.rs` 七份：W13 之前（在草稿里把 W16、W15、W14、W13 四份补丁里这七份的那一段依次倒着撤掉还原，与暂存区逐份 sha256 相同）与此刻工作树的 diff，7 个文件 +1178/−183。

```diff
diff --git a/crates/singlefs-core/src/block_device.rs b/crates/singlefs-core/src/block_device.rs
index fcd461cf..4735c22b 100644
--- a/crates/singlefs-core/src/block_device.rs
+++ b/crates/singlefs-core/src/block_device.rs
@@ -156,6 +156,19 @@ pub trait BlockDevice {
     ) -> Option<Vec<DeviceOffsetInBytes>> {
         None
     }
+    /// 读坏（读不出或读出来自证不过）之后当场重读的那一读（条款要它再读一次盘：D19（块指针的结构与宽度预算） 已定项 5、
+    /// D16（发布语义） 已定项 1「根槽这一次读坏」那一行）。默认就是 [`BlockDevice::read_at`]：盘自己不记读过什么。
+    /// 记着这次挂载读过什么的那一层（`mounted_session::DeviceReadOnceInThisMount`）在这里绕过记下的那一份、直接读底下那块盘。
+    fn reread_at_after_a_bad_read(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        self.read_at(offset, buffer)
+    }
+    /// 当场重读那一读读到、调用方核过自证（根槽自证、单元校验和与位置项相符）的字节：记着这次挂载读过什么的那一层拿它换掉记下的那一份，
+    /// 之后再读这几块取的是它。默认什么都不做：盘自己不记读过什么。`offset` 与 `bytes` 是那一次重读的请求，对齐与越界照 [`BlockDevice::read_at`]。
+    fn keep_the_self_verified_reread(&self, _offset: DeviceOffsetInBytes, _bytes: &[u8]) {}
 }
 
 /// 文件后端的写零怎么落到镜像文件上。
diff --git a/crates/singlefs-core/src/deleted_inodes.rs b/crates/singlefs-core/src/deleted_inodes.rs
index bd1a0bbe..6fd4cff2 100644
--- a/crates/singlefs-core/src/deleted_inodes.rs
+++ b/crates/singlefs-core/src/deleted_inodes.rs
@@ -58,7 +58,8 @@ pub fn deleted_inodes_root_entries(
         .collect())
 }
 
-/// 从一个 `deleted_inodes` 根的条目读回号（条目宽在这之前由读者核过等于 8）。
+/// 从一个 `deleted_inodes` 根的条目读回号。每条条目宽恰好 8（无 value）：重建上一版读这个根的那一判
+/// （`recovery::rebuild_version_in_the_unit_area_starting_at`）宽不等于 8 就报 `UnitMalformed`、不走到这里；宽于 8 的不截前 8 字节收下。
 #[must_use]
 pub fn inode_numbers_of_the_deleted_inodes_root_entries(entries: &[Vec<u8>]) -> Vec<InodeNumber> {
     entries
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index a75238d2..af1043c4 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -30,7 +30,10 @@ use crate::instance_table_sweep::{
 };
 use crate::journal::back_chain_of;
 use crate::make_filesystem::MakeFilesystemParameters;
-use crate::mounted_session::{refusal_is_short_of_space, DeviceKeepingWritesInMemory};
+use crate::mounted_session::{
+    refusal_is_short_of_space, DeviceKeepingWritesInMemory, DeviceReadOnceInThisMount,
+    ReadRouteOfThisMount,
+};
 use crate::pointer::LocationEntry;
 use crate::pointer::{slot_shared_by_both_location_entries, LocationEntriesOnDifferentSlots};
 use crate::records::{
@@ -65,32 +68,37 @@ use crate::recovery::{
     RootRingSlotKnownToHoldARootStillBadAfterOneReread, SystemConfigurationSlotsOfOneDevice,
     UserVisibleTreeRootPointers,
 };
-use crate::recovery::{every_root_ring_slot, read_root_ring_slot, RootRingSlotReading};
+use crate::recovery::{
+    every_root_ring_slot, read_root_ring_slot, PoolReaderOfOneReadPass, RootRingSlotReading,
+};
 use crate::root_record::{RootRecord, UnmountMarker};
 use crate::root_ring::{target_for_publish, RootRingSlot};
 use crate::system_configuration::SystemImmutableSizes;
 use crate::transaction::{
-    acquire_expected_instance, central_mapping_locations_in_the_version,
+    central_mapping_locations_in_the_version,
     copies_of_a_released_unit_failing_the_release_checksum_check,
     next_instance_generation_to_acquire, placement_to_release_after_checking_every_device,
     prepare_the_publish_without_units, prepare_the_row_publish_on_a_version_without_file,
     prepare_the_version_publish, publish_instance_table_on_version_without_file, publish_version,
     publish_version_appending_a_lineage_record, publish_version_with_unmount_marker,
-    publish_without_units, refuse_locations_that_do_not_name_two_pool_devices,
+    publish_without_units, read_the_witness_before_the_expected_acquisition_writes,
+    refuse_locations_that_do_not_name_two_pool_devices,
     rehearse_every_step_of_the_deletion_queue_walk, role_of_allocation_record_tree_node,
     walk_the_deletion_queue_until_it_is_empty,
-    write_the_raised_floor_into_every_system_configuration, AcquisitionFailed,
-    CodeTwoTreeNodeCapacities, CopyQuarantinedAfterReleaseChecksumMismatch,
-    DeletionAndLineageOfThePublish, DeletionQueueWalk, ExpectedInstanceAcquisitionFailed,
-    InstanceGenerationPastTheTopOfItsRange, InstanceTableOnlyPublishPlan, InstanceTablePlan,
-    InstanceTableRewrite, MappingLookup, PoolVersion, PoolWriter, PublishError, PublishPlan,
-    PublishedUnit, RaisedFloorSystemConfigurationWriteFailed, ReleaseChecksumCheck,
+    write_the_raised_floor_into_every_system_configuration,
+    write_the_witnessed_expected_acquisition, AcquisitionFailed, CodeTwoTreeNodeCapacities,
+    CopyQuarantinedAfterReleaseChecksumMismatch, DeletionAndLineageOfThePublish, DeletionQueueWalk,
+    ExpectedInstanceAcquisitionFailed, InstanceGenerationPastTheTopOfItsRange,
+    InstanceTableOnlyPublishPlan, InstanceTablePlan, InstanceTableRewrite, MappingLookup,
+    PoolVersion, PoolWriter, PublishError, PublishPlan, PublishedUnit,
+    RaisedFloorSystemConfigurationWriteFailed, ReadFailedOnTheDisk, ReleaseChecksumCheck,
     TransactionOutput, TransactionUnit, VersionWithoutFilePublishOutput, ZeroUnitPublishPlan,
 };
 use crate::unlink::DeletionSwitches;
 use crate::write_accounting::{WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind};
 use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES, ROOT_RING_REGIONS, SLOT_BYTES};
 use std::borrow::Cow;
+use std::cell::{Cell, RefCell};
 use std::collections::{BTreeMap, BTreeSet};
 
 pub use crate::instance_table::{InstanceRow, InstanceTableRecords};
@@ -115,7 +123,9 @@ pub enum MountError {
     Acquisition(AcquisitionFailed),
     /// 取号之后那一串发布（写行一次、暖机至多 R 次）里有一次没做成，或挂载续做里的一步没发成、不是空间不够：错，连同这次挂载交得出的写账
     /// （[`PublishSequenceFailed`]：续做那一格的已落盘各次是写行、暖机、推的抬 F 与续做里已落盘的每一次，按先后）。
-    /// 续做里有一步要开的容器号到顶（C841）在取号之前预演出来时同样交这个成员，那时一个写都没发，三份写账都是空的
+    /// 续做里有一步要开的容器号到顶（C841）在取号之前预演出来时同样交这个成员；取号之前预演续做没走到结局、读盘时又有过读错
+    /// （`cause` 是 `PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition`）也交这个成员。这两种都在取号之前，
+    /// 一个写都没发，三份写账都是空的
     /// （`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`）。
     /// 装箱（与 `RaiseFloorSequencePublishFailed` 同）：写账带上抬 F 先写系统配置那一步之后，不装箱 `MountError` 就大过 clippy
     /// `result_large_err` 的 128 字节。
@@ -401,8 +411,9 @@ pub enum StillUnreadableAfterOneReread {
     /// 被退掉的那条）——的账（树表或分配记录树）读不出、那次发布读得出的记录又不齐（本次发布内序号缺一条，或没有带末条标志的那条），
     /// 照被抛弃根那一份从中央映射现算也不成，重读一次仍不成，现行账里又没有 C694 的 B 那道凭据（`C_MAPPING` 收窄，判决
     /// `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K2 W1 / W2 H1、K2 W2 H3 两行与第三节第 1 条）。
-    /// 较新可读根那次发布一条记录都读不出时，只在那条根的树表与中央映射树根两个指针都没有「两份都是更新实例的单元」时走到这一拒
-    /// （C834，用户 2026-10-15 定「同一指针两份都更新」，`research/prompts/c834-r3-main-verification.md`「四、用户定案」）。
+    /// 较新可读根那次发布一条记录都读不出时，只在那条根的树表与中央映射树根两个指针都没有「两份都是本池更新实例的单元」时走到这一拒
+    /// （C834，用户 2026-10-15 定「同一指针两份都更新」，`research/prompts/c834-r3-main-verification.md`「四、用户定案」；
+    /// 单元头里的 fsid 与本池的相同才算本池的，用户定「比 fsid」，零轮）。
     /// `unapplied_publish` 点名那次发布（那条较新可读根就是它自己那次发布）。与 C393 那一拒（[`Self::AccountOfAnAbandonedRoot`]）分开报：
     /// 这一拒列在 D23（journal 的角色与格式） 已定项 14 那张单子上，C393 那一拒不在（用户 2026-10-08 定「乙：core 另起错误成员」）。
     /// 在取号之前拒，盘上逐字节不变，只读挂载照常。
@@ -807,7 +818,8 @@ pub enum SelectedVersionLackingOnDevice {
 }
 
 /// 自己开写入口、接连推的一串发布里有一次没做成：可写挂载取号之后那一串——写行一次、暖机推到本实例的根覆盖每块盘
-/// （D16（发布语义） 已定项 8 戊）；抬 F 那一串空发布——推到每块盘上都有一条带新 F 的根（D16（发布语义） 已定项 1）。
+/// （D16（发布语义） 已定项 8 戊）——连同之后的挂载续做；抬 F 那一串空发布——推到每块盘上都有一条带新 F 的根（D16（发布语义） 已定项 1）。
+/// 可写挂载取号之前预演续做拒（C841，或预演没走到结局而读盘时有过读错）也交它：那时这一串还没开写入口、一个写都没发，三份写账都是空的。
 /// 写入口是这一串自己开的、随错一起丢掉，所以它交得出的写账都在这里：已经落盘的那几次发布各自的写，
 /// 与失败那一次落盘阶段已记的写（增补 2 收口表第 58 行：不交出来，设备一层数到的写与程序交得出的账对不上）。
 /// 不是发布的写不在其内：可写挂载取号那两次系统配置槽写（取号的回卷另由 `AcquisitionFailed` 报）。
@@ -1816,9 +1828,15 @@ fn placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_afte
         return Some(computed);
     }
     before_the_one_reread.before_rereading();
+    // 重读那一遍绕过这次挂载记下的那一份、再读一次盘（`recovery::ReadPassOverThePool::RereadAfterABadRead`）。
+    let rereading_reader = PoolReaderOfOneReadPass::rereading_after_a_bad_read(devices);
     if let Some(placements) =
-        placements_referenced_by_root(devices, unit_area_start, root).or_else(|| {
-            placements_referenced_by_root_computed_from_the_mapping(devices, unit_area_start, root)
+        placements_referenced_by_root(&rereading_reader, unit_area_start, root).or_else(|| {
+            placements_referenced_by_root_computed_from_the_mapping(
+                &rereading_reader,
+                unit_area_start,
+                root,
+            )
         })
     {
         return Some(placements);
@@ -2020,20 +2038,23 @@ fn placements_in_the_accounts_of_unapplied_publishes<Device: BlockDevice>(
 /// 账里的落点按它的 txg 在根环里对应的槽（[`target_for_publish`]）归组交回，调用方并进 C 的集合、同一个时机解开。账读不出时照没施加的发布那一份办：
 /// 那次发布读得出的记录不齐就从中央映射现算（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]，仍不成拒），记录齐就不交账（交 F1 乙）、
 /// 改交挂载那一刻现行账（`current_records`）里仍分配着的落点（丙，用户 2026-10-08 定，函数体里那一支的注释）；
-/// 一条记录都读不出的，映射现算得出就交；现算不成时，那条根的树表与中央映射树根两个指针里有一个两份都已被更新实例的单元盖掉
-/// （[`one_pointer_has_both_copies_overwritten_by_a_later_instance`]）就不交也不拒，否则照记录不齐那一支办（仍不成拒；函数体里那一支的注释）。
-/// `roots` 是重建分配器读根环那一遍读出的根（影子账、根环表用的同一份）。
+/// 一条记录都读不出的，映射现算得出就交；现算不成时，那条根的树表与中央映射树根两个指针里有一个两份都已被本池更新实例的单元盖掉
+/// （[`one_pointer_has_both_copies_overwritten_by_a_later_instance`]，单元头里的 fsid 与 `filesystem_identifier_in_unit_headers` 相同才算）
+/// 就不交也不拒，否则照记录不齐那一支办（仍不成拒；函数体里那一支的注释）。
+/// `roots` 是重建分配器读根环那一遍读出的根（影子账、根环表用的同一份）；`filesystem_identifier_in_unit_headers` 是择到的那份系统配置里
+/// fsid 的低 8 字节（单元头里记的那一截，[`crate::unit::unit_filesystem_identifier`]），不另读盘。
 ///
 /// # Errors
 /// 同 [`placements_in_the_accounts_of_unapplied_publishes`]（点名那条根的 (实例代号, checkpoint_txg)）。
 #[allow(
     clippy::ptr_arg,
     clippy::too_many_arguments,
-    reason = "ptr_arg：placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、根环里读出的根、判被抛弃的谓词、读阶段扫出的记录、所选那一版、每区根环槽数、现行账、重读前的钩子"
+    reason = "ptr_arg：placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、本池单元头里的 fsid、根环里读出的根、判被抛弃的谓词、读阶段扫出的记录、所选那一版、每区根环槽数、现行账、重读前的钩子"
 )]
 fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
     unit_area_start: UnitAreaStart,
+    filesystem_identifier_in_unit_headers: u64,
     roots: &[RootRecord],
     is_abandoned: &dyn Fn(&RootRecord) -> bool,
     records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
@@ -2068,7 +2089,8 @@ fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
                         // 那次发布一条记录都读不出：映射现算得出就并进来（攻方 H1 里账读不出的那几格要它）。现算不成时只看那条根自己的
                         // 树表与中央映射树根两个指针（C834，用户 2026-10-15 定「同一指针两份都更新」，`research/prompts/c834-r3-main-verification.md`
                         // 「四、用户定案」，被攻过零轮；替换用户 2026-10-08 定的 Y2 收严那一判，不再读系统配置里取过号的实例代号——那个号随盘序变）：
-                        // 有一个指针两份都已被更新实例的单元盖掉，那条根那个单元两份都没了、本来就走不完，映射也就现算不成，拒了就再也做不成
+                        // 有一个指针两份都已被本池更新实例的单元盖掉（单元头里的 fsid 与本池的相同才算，用户定「比 fsid」，零轮：
+                        // 别的池的单元头不说明本池有过更新的实例），那条根那个单元两份都没了、本来就走不完，映射也就现算不成，拒了就再也做不成
                         // 可写挂载（入库用例 `urJ` 那一形），不拒、不重读；否则照「记录不齐」那一支办（重读一次、看 C694 的 B 凭据，仍不成拒）——
                         // 它的单元可能还在（只是这一次读不出，或只被盖掉一份），不拒就会在写行那次发布里写掉它们。
                         match placements_referenced_by_root_computed_from_the_mapping(
@@ -2079,7 +2101,9 @@ fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
                             Some(computed) => computed,
                             None
                                 if !one_pointer_has_both_copies_overwritten_by_a_later_instance(
-                                    devices, root,
+                                    devices,
+                                    root,
+                                    filesystem_identifier_in_unit_headers,
                                 ) =>
                             {
                                 placements_of_an_unapplied_publish_whose_account_is_unreadable(
@@ -2151,9 +2175,11 @@ fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
 }
 
 /// C834 那一判的凭据（用户 2026-10-15 定「同一指针两份都更新」，`research/prompts/c834-r3-main-verification.md`「四、用户定案」，被攻过零轮）：
-/// `root` 的根记录里树表与中央映射树根两个指针中有一个，它两条位置条目指着的两个槽都读出自证过的单元头、写序里的实例代号比 `root` 的实例代号大
-/// （[`instance_in_the_self_verified_unit_header_at`]）。全零的空指针没有槽可读，不算证据。只读盘、不写盘。
+/// `root` 的根记录里树表与中央映射树根两个指针中有一个，它两条位置条目指着的两个槽都读出自证过的本池单元头（头里的 fsid 与
+/// `filesystem_identifier_in_unit_headers` 相同）、写序里的实例代号比 `root` 的实例代号大（[`instance_in_the_self_verified_unit_header_at`]）。
+/// 全零的空指针没有槽可读，不算证据。只读盘、不写盘。
 /// 只要一份不够（攻方 `research/prompts/c834-r3-opus-output.md` 第二节 T2：盖掉一份、另一份这一次读回全零时那条根还走得完）。
+/// 别的池的单元头不算（用户定「比 fsid」，零轮）：它的实例代号是那个池的号，不说明本池有过比 `root` 更新的实例写过这两个槽。
 #[allow(
     clippy::ptr_arg,
     reason = "PoolReader 实现在 Vec<(DeviceIdentity, Device)> 上，切片不能转成 &dyn PoolReader"
@@ -2161,20 +2187,26 @@ fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
 fn one_pointer_has_both_copies_overwritten_by_a_later_instance<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
     root: &RootRecord,
+    filesystem_identifier_in_unit_headers: u64,
 ) -> bool {
     [&root.tree_table, &root.mapping_root]
         .into_iter()
         .filter(|pointer| **pointer != crate::pointer::NodePointer::empty_root())
         .any(|pointer| {
             pointer.locations.iter().all(|location| {
-                instance_in_the_self_verified_unit_header_at(devices, location)
-                    .is_some_and(|instance| instance > root.instance)
+                instance_in_the_self_verified_unit_header_at(
+                    devices,
+                    location,
+                    filesystem_identifier_in_unit_headers,
+                )
+                .is_some_and(|instance| instance > root.instance)
             })
         })
 }
 
-/// `location` 那一槽起读出的自证过的单元头（头校验和与载荷校验和都过）里写序的实例代号：先按索引节点读 16 KiB，解不开再读 32 KiB、
-/// 依次按数据单元与打包记录单元解；读不出（越过盘尾、没有那块盘）或三类都解不开交 `None`。只读盘、不写盘。
+/// `location` 那一槽起读出的自证过的本池单元头（头校验和与载荷校验和都过，头里的 fsid 与 `filesystem_identifier_in_unit_headers` 相同）
+/// 里写序的实例代号：先按索引节点读 16 KiB，解不开再读 32 KiB、依次按数据单元与打包记录单元解；读不出（越过盘尾、没有那块盘）、
+/// 三类都解不开或解得开而 fsid 不是本池的交 `None`。只读盘、不写盘。
 #[allow(
     clippy::ptr_arg,
     reason = "PoolReader 实现在 Vec<(DeviceIdentity, Device)> 上，切片不能转成 &dyn PoolReader"
@@ -2182,31 +2214,36 @@ fn one_pointer_has_both_copies_overwritten_by_a_later_instance<Device: BlockDevi
 fn instance_in_the_self_verified_unit_header_at<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
     location: &LocationEntry,
+    filesystem_identifier_in_unit_headers: u64,
 ) -> Option<InstanceGeneration> {
     let offset = location.slot.to_device_offset();
-    let index_node_instance = PoolReader::read(
+    let index_node_header = PoolReader::read(
         devices,
         location.device,
         offset,
         usize::try_from(NODE_BYTES).expect("16384"),
     )
-    .and_then(|bytes| crate::unit::parse_index_node(&bytes).ok())
-    .map(|header| header.instance);
-    if index_node_instance.is_some() {
-        return index_node_instance;
-    }
-    let unit_bytes = PoolReader::read(
-        devices,
-        location.device,
-        offset,
-        usize::try_from(DATA_UNIT_BYTES).expect("32768"),
-    )?;
-    crate::unit::parse_data_unit(&unit_bytes)
-        .map(|header| header.write_order.instance)
-        .or_else(|_not_a_data_unit| {
-            crate::unit::parse_packed_unit(&unit_bytes).map(|header| header.write_order.instance)
-        })
-        .ok()
+    .and_then(|bytes| crate::unit::parse_index_node(&bytes).ok());
+    let (filesystem_identifier_in_the_header, instance) = match index_node_header {
+        Some(header) => (header.filesystem_identifier, header.instance),
+        None => {
+            let unit_bytes = PoolReader::read(
+                devices,
+                location.device,
+                offset,
+                usize::try_from(DATA_UNIT_BYTES).expect("32768"),
+            )?;
+            crate::unit::parse_data_unit(&unit_bytes)
+                .map(|header| (header.filesystem_identifier, header.write_order.instance))
+                .or_else(|_not_a_data_unit| {
+                    crate::unit::parse_packed_unit(&unit_bytes)
+                        .map(|header| (header.filesystem_identifier, header.write_order.instance))
+                })
+                .ok()?
+        }
+    };
+    (filesystem_identifier_in_the_header == filesystem_identifier_in_unit_headers)
+        .then_some(instance)
 }
 
 /// [`placements_in_the_accounts_of_newer_readable_roots`] 交回的两样。
@@ -2495,12 +2532,8 @@ fn placements_of_an_unapplied_publish_whose_account_is_unreadable<Device: BlockD
 /// 树表、中央映射树的每个节点）加中央映射树里每一条映射条目的落点（数据单元、打包单元、四棵树的节点，含分配记录树自己的节点）。
 /// 树表自己的落点取根记录里那条指针。只在内存里用，不写盘。中央映射树读不出、解不开（映射根是空指针的 mkfs 第 0 代、树表 0 条的一版也在内），
 /// 映射条目解不开、落点不同槽或不在单元区：交 `None`。
-#[allow(
-    clippy::ptr_arg,
-    reason = "read_unit_via_locations 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
-)]
-fn placements_referenced_by_root_computed_from_the_mapping<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+fn placements_referenced_by_root_computed_from_the_mapping<Reader: PoolReader>(
+    devices: &Reader,
     unit_area_start: UnitAreaStart,
     root: &RootRecord,
 ) -> Option<Vec<(DeviceIdentity, crate::address::SlotNumber, u64)>> {
@@ -2538,9 +2571,10 @@ fn placements_referenced_by_root_computed_from_the_mapping<Device: BlockDevice>(
         located.push((locations, span));
     }
     let mut placements = Vec::new();
+    let identities = devices.device_identities();
     for (locations, span) in located {
         let slot = slot_shared_by_both_location_entries(&locations).ok()?;
-        for (identity, _) in devices {
+        for identity in &identities {
             placement_lies_in_the_unit_area_of_its_device(
                 devices,
                 unit_area_start,
@@ -2567,12 +2601,8 @@ fn placements_referenced_by_root_computed_from_the_mapping<Device: BlockDevice>(
 /// 少了这一臂它在影子账里就成了「账读不出」的被抛弃根，那片实例表不被隔离、也不在重建出来的账里，
 /// 挂载之后当空闲槽发出去，把根环里还引用着它的那条根读坏（2026-09-23 崩溃注入打中：那时的挂载时回退里
 /// I-2.1 / I-4.8 / I-7.4 三条红在「最新根引用的单元已被复用」；崩溃恢复落到树表 0 条的根时是同一格）。
-#[allow(
-    clippy::ptr_arg,
-    reason = "allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
-)]
-fn placements_referenced_by_root<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+fn placements_referenced_by_root<Reader: PoolReader>(
+    devices: &Reader,
     unit_area_start: UnitAreaStart,
     root: &RootRecord,
 ) -> Option<Vec<(DeviceIdentity, crate::address::SlotNumber, u64)>> {
@@ -2611,7 +2641,7 @@ fn placements_referenced_by_root<Device: BlockDevice>(
                 )
             {
                 let slot = slot_shared_by_both_location_entries(&pointer.locations).ok()?;
-                for (identity, _) in devices {
+                for identity in &devices.device_identities() {
                     // 槽号是盘上读来的 6 字节：进分配器（`PoolAllocator::isolate_abandoned` 与根环表里清隔离那一步）之前判它在这块盘的单元区里，
                     // 不在就与两条位置条目不同槽同一个结局——这条根的账解不开，调用方只计数（代码审阅第 32 条；改之前是
                     // `DeviceFreeMap::index` 的 expect 与 `isolate` 的跨度断言 panic）。
@@ -2863,7 +2893,7 @@ fn txgs_ring_slots_bad_at_mount_can_hold(
 /// 读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，两遍逐槽并起来整个环仍一条都没有 ⇒
 /// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（用户 2026-10-05 定「重读」、Q3 另起拒因、Q4 rr2、R1 按槽并两遍）；
 /// C 读一次没施加的发布或一条较新可读根的账读不出、那次发布读得出的记录不齐（较新可读根那次发布一条记录都读不出、树表与中央映射树根
-/// 两个指针都没有两份都是更新实例的单元的，也算这一支）、从中央映射也现算不成、重读一次仍不成
+/// 两个指针都没有两份都是本池更新实例的单元的，也算这一支；别的池的单元头不算）、从中央映射也现算不成、重读一次仍不成
 /// （[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）⇒
 /// `NewerStateStillUnreadableAfterOneReread(AccountOfAnUnappliedPublishWithIncompleteReadableRecords)`。
 /// 都在取号之前，盘上逐字节不变。
@@ -3178,6 +3208,9 @@ fn rebuilt_allocator<Device: BlockDevice>(
             } = placements_in_the_accounts_of_newer_readable_roots(
                 devices,
                 unit_area_start,
+                crate::unit::unit_filesystem_identifier(
+                    &system_configuration.immutable.filesystem_identifier,
+                ),
                 &roots,
                 &is_abandoned,
                 records,
@@ -3482,6 +3515,18 @@ fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
             ring_slot,
         )
     };
+    // 读坏之后当场重读那一读绕过这次挂载记下的那一份、再读一次盘，读到自证过的换掉记下的那一份
+    // （`recovery::ReadPassOverThePool::RereadAfterABadRead`）。
+    let rereading_reader = PoolReaderOfOneReadPass::rereading_after_a_bad_read(reader);
+    let reread_once_after_a_bad_read = |ring_slot: RootRingSlot| {
+        read_root_ring_slot(
+            &rereading_reader,
+            &system_configuration.immutable.region_devices,
+            &system_configuration.immutable.sizes,
+            &system_configuration.immutable.filesystem_identifier,
+            ring_slot,
+        )
+    };
     let mut roots = Vec::new();
     let mut ring_slots_bad_on_this_read_taken_as_holding_no_root = BTreeSet::new();
     // 迭代上界是根环的槽数（3 × S）；跨轮携带的只有交回的根与当没有根的槽；提前出口只有「知道住着根的槽重读仍坏」那一条。
@@ -3496,7 +3541,7 @@ fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
         if let Some(root_record_self_verified_on_one_rebuild_pass) =
             root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass.get(&ring_slot)
         {
-            match read_once(ring_slot) {
+            match reread_once_after_a_bad_read(ring_slot) {
                 RootRingSlotReading::SelfVerified(root) => roots.push(root),
                 RootRingSlotReading::Bad(_still_bad) => {
                     roots.push(*root_record_self_verified_on_one_rebuild_pass);
@@ -3508,7 +3553,7 @@ fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
             ring_slots_bad_on_this_read_taken_as_holding_no_root.insert(ring_slot);
             continue;
         }
-        match read_once(ring_slot) {
+        match reread_once_after_a_bad_read(ring_slot) {
             RootRingSlotReading::SelfVerified(root) => roots.push(root),
             RootRingSlotReading::Bad(reread) => {
                 return Err(
@@ -4363,6 +4408,8 @@ fn readable_roots_for_raising_the_floor<Reader: PoolReader + ?Sized>(
         .iter()
         .map(|(ring_slot, _)| *ring_slot)
         .collect();
+    // 重读那一读绕过这次挂载记下的那一份、再读一次盘（`recovery::ReadPassOverThePool::RereadAfterABadRead`）。
+    let rereading_reader = PoolReaderOfOneReadPass::rereading_after_a_bad_read(reader);
     // 迭代上界是两遍里只有一遍自证过的槽数（至多根环的槽数）；跨轮只往交回的根里加；没有提前出口。上面那一遍读坏、当没有根的才在这里重读一次。
     for (ring_slot, root_record_self_verified_on_one_rebuild_pass) in
         &ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass
@@ -4371,7 +4418,7 @@ fn readable_roots_for_raising_the_floor<Reader: PoolReader + ?Sized>(
             continue;
         }
         match read_root_ring_slot(
-            reader,
+            &rereading_reader,
             &system_configuration.immutable.region_devices,
             &system_configuration.immutable.sizes,
             &system_configuration.immutable.filesystem_identifier,
@@ -5078,7 +5125,8 @@ struct DryRunRefusal {
 }
 
 /// 取号之前把这次挂载取号之后要发的那一串——写行一次、暖机 `warm_up_publish_txgs` 那几次——在分配器的一份拷贝上整串预演一遍，
-/// 交回每一次取到的落点。只动拷贝、不碰盘；**走的就是发布路径落盘之前那一段**（带文件的一版 `transaction::prepare_the_version_publish`，
+/// 交回每一次取到的落点、预演到最后的那一版与那一份分配器拷贝（[`PublishesRehearsedBeforeAcquisition`]：取号之前预演续做接着它们走）。
+/// 只动拷贝、不写盘（读盘读的是这次挂载共用的那一份，`mounted_session::DeviceReadOnceInThisMount`）；**走的就是发布路径落盘之前那一段**（带文件的一版 `transaction::prepare_the_version_publish`，
 /// 树表 0 条的一版上写行 `transaction::prepare_the_row_publish_on_a_version_without_file`，零单元发布
 /// `transaction::prepare_the_publish_without_units`），计划也是取号之后那几次发布用的同一张（`row_publish_plan_on_file_version`、
 /// `empty_publish_plan_after_file_version`、`empty_publish_plan_after_version_without_file`），不另推一份：
@@ -5238,16 +5286,31 @@ struct PublishesRehearsedBeforeAcquisition {
 /// 那一行；代码轮 `research/prompts/m3-b44-code-r1-main-verification.md` 第二节 K6）：在取号之前那一串预演到最后的那一版与那一份分配器拷贝上，
 /// 照续做取的批大小与容器树容量（分配器上装的删除开关，续做读同一份）把删除队列走到空的每一步都只算不写
 /// （`transaction::rehearse_every_step_of_the_deletion_queue_walk`），真分配器不动。
+/// 这份拷贝是取号之前那一串（写行、暖机）预演到最后的那一份，重建分配器时扣住的那一截没放开（真做在暖机走完之后放开，
+/// [`acquire_then_publish_until_the_continuation`]），不读盘核换下的每一份的校验和（`ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition`）。
+/// 扣住位没放开那一差让这一趟的空间判得比真做紧：紧到有一步空间不够时转去下面只进内存那一趟，那一趟走真做那一段、放开扣住位，
+/// 所以这一差只多花一趟、不多拒。
 /// 那一趟有一步被拒是空间不够的那两种（[`refusal_is_short_of_space`]：续做真发时推一串抬 F 再试的就是这两种）时，抬 F 也预演进去：
 /// 在只进内存的盘与 `allocator` 的一份拷贝上把取号之后那一串连同续做照真发走一遍
-/// （[`walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`]），取续做在那里报的错。
+/// （[`walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`]），看它走到哪儿。
 /// 有一步报 C841（容器树要开新容器而最右那片的号已到顶）就在取号之前拒：不拒的话这一错在续做里才交回，那时写行那次发布与续做里
 /// 排在它前面的各步（连同推的抬 F）已落盘，之后每次可写挂载再取号、写行、再拒（攻方 `research/prompts/m3-b44-code-r1-opus-output.md` 第五节）。
-/// 预演报的别的错这里不拒、照旧留给续做（[`rehearsed_continuation_refusal_is_returned_before_acquisition`]）；分配器拷贝上那一趟报了
-/// 别的错（不是空间不够）的那一步之后接不上一版，不再预演。
+/// 两趟读的都是这次挂载共用的那一份（`mounted_session::DeviceReadOnceInThisMount`，真做之后读的也是它）。预演没走到续做的结局——
+/// 续做里一步报了 C841 之外的错，或只进内存那一趟在续做之前（取号、写行、暖机、写行之后推的抬 F）报了错、续做里推的抬 F 自己报了错——
+/// 而这两趟读盘时有过读错（`reads_failed_on_the_disk` 里这两趟记下的）：在取号之前拒，交最先的那一次读错；读错的那一块没记下，
+/// 交给真做的话真做再读一次盘、看到的可能与预演不同。没有读错的照旧留给续做、真做时报它自己的（岔路单那一行只定了 C841，
+/// [`rehearsed_continuation_refusal_is_returned_before_acquisition`]）：只进内存那一趟与真做读同一份字节、走同一段，它报的错真做走到那一步照样报；
+/// 分配器拷贝上那一趟与真做差扣住位与校验和核验两样，它报的错真做不一定报。
+/// 分配器拷贝上那一趟报了别的错（不是空间不够）的那一步之后接不上一版，不再预演。
 ///
 /// # Errors
-/// C841 ⇒ `MountError::Publish`：`cause` 是那个错，三份写账都是空的（取号之前，一个写都没发到盘上）。
+/// C841 ⇒ `MountError::Publish`：`cause` 是那个错；预演没走到续做的结局而读盘时有过读错 ⇒ `MountError::Publish`：`cause` 是
+/// `PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition`（最先的那一次读错）。两种都是三份写账全空
+/// （取号之前，一个写都没发到盘上）。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "取号之前预演续做要的：写入口、预演到最后的那一版与分配器拷贝、真分配器、起点、要写的行、取号之后的计划、读错账，各自独立"
+)]
 fn refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range<
     Device: BlockDevice,
 >(
@@ -5258,13 +5321,16 @@ fn refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_p
     start: &InstanceStart,
     rows_written: &[InstanceRow],
     publishes_planned: &PublishesPlannedAfterAcquisition,
+    reads_failed_on_the_disk: &RefCell<Vec<ReadFailedOnTheDisk>>,
 ) -> Result<(), MountError> {
     let version_of_trees = match version_after_the_publishes_rehearsed {
         PoolVersion::WithoutFile(_) => return Ok(()),
         PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => version,
     };
+    // 读错账只看这两趟预演记下的：之前读的（所选那一版的单元清单、写行与暖机那一串的预演）有读错也已各自判过。
+    reads_failed_on_the_disk.borrow_mut().clear();
     let deletion_switches_the_continuation_reads = allocator.deletion_switches();
-    let refusal = match rehearse_every_step_of_the_deletion_queue_walk(
+    let ending = match rehearse_every_step_of_the_deletion_queue_walk(
         pool,
         &mut allocator_after_the_publishes_rehearsed,
         version_of_trees,
@@ -5274,23 +5340,37 @@ fn refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_p
     ) {
         Ok(()) => return Ok(()),
         Err(refusal) if refusal_is_short_of_space(&refusal) => {
-            match walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory(
+            walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory(
                 pool.parameters,
                 &*pool.devices,
                 allocator,
                 start,
                 rows_written,
                 publishes_planned,
-            ) {
-                Some(refusal_of_the_continuation) => refusal_of_the_continuation,
-                None => return Ok(()),
-            }
+            )
+        }
+        Err(refusal) => ContinuationRehearsedBeforeAcquisition::StepRefused(refusal),
+    };
+    let refusal = match ending {
+        ContinuationRehearsedBeforeAcquisition::WalkedToItsEnd => return Ok(()),
+        ContinuationRehearsedBeforeAcquisition::StepRefused(refusal)
+            if rehearsed_continuation_refusal_is_returned_before_acquisition(&refusal) =>
+        {
+            refusal
+        }
+        ContinuationRehearsedBeforeAcquisition::StepRefused(_)
+        | ContinuationRehearsedBeforeAcquisition::StoppedBeforeTheEndOfTheContinuation => {
+            let first_read_failed_on_the_disk =
+                reads_failed_on_the_disk.borrow_mut().drain(..).next();
+            // 这两趟读盘没出过读错：照旧留给续做（岔路单那一行只定了 C841）。
+            let Some(read_failed_on_the_disk) = first_read_failed_on_the_disk else {
+                return Ok(());
+            };
+            PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(Box::new(
+                read_failed_on_the_disk,
+            ))
         }
-        Err(refusal) => refusal,
     };
-    if !rehearsed_continuation_refusal_is_returned_before_acquisition(&refusal) {
-        return Ok(());
-    }
     Err(MountError::Publish(Box::new(PublishSequenceFailed {
         cause: refusal,
         writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,
@@ -5299,11 +5379,24 @@ fn refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_p
     })))
 }
 
-/// 取号之前预演整个续做报的错里，哪几种在取号之前拒（[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）：
-/// 只有 C841 那一种（岔路单那一行只定了这一错）；别的照旧留给续做，续做里怎么处置见 [`continue_the_deletion_queue_until_it_is_empty`]。
+/// 取号之前预演续做走到哪儿（[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）。
+enum ContinuationRehearsedBeforeAcquisition {
+    /// 续做走到空，或推满仍放不下、停在那里（真做时挂载照样做成可写、队列留着）。
+    WalkedToItsEnd,
+    /// 续做里一步报了错：分配器拷贝上那一趟报的不是空间不够的错，或只进内存那一趟里续做交的 `MountError::Publish` 的 `cause`。
+    StepRefused(PublishError),
+    /// 只进内存那一趟在续做之前（取号、写行、暖机、写行之后推的抬 F）报了错，或续做里推的抬 F 自己报了错：没走到续做的结局。
+    StoppedBeforeTheEndOfTheContinuation,
+}
+
+/// 取号之前预演整个续做报的错里，哪几种不看读错账就在取号之前拒（[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）：
+/// 只有 C841 那一种（岔路单那一行只定了这一错）；别的预演那一趟读盘没出过读错时照旧留给续做，续做里怎么处置见
+/// [`continue_the_deletion_queue_until_it_is_empty`]，出过读错时由读错账那一判拒。
 fn rehearsed_continuation_refusal_is_returned_before_acquisition(refusal: &PublishError) -> bool {
     match refusal {
         PublishError::NextContainerNumberPastTheTopOfItsRange(_) => true,
+        // 预演读错那一拒是调用方按读错账自己造的，续做里的一步不报它；走到这里也是取号之前拒的那一形。
+        PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(_) => true,
         PublishError::SpaceAdmissionRefused(_)
         | PublishError::PlacementRefused { .. }
         | PublishError::MultiLevelCodeTwoTreeRefused { .. }
@@ -5324,8 +5417,8 @@ fn rehearsed_continuation_refusal_is_returned_before_acquisition(refusal: &Publi
         | PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_)
         | PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
         | PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }
-        // txg 到顶不拒（岔路单那一行只定了 C841，别的错不扩大）：分配器拷贝上那一趟不算写行之后为空间推的抬 F，
-        // 续做各步的 txg 在那里算的不是真发的那一个。
+        // txg 到顶不拒：岔路单那一行只定了 C841，别的错不扩大，两趟预演都照这一条。分配器拷贝上那一趟另有一层：它不算写行之后为空间推的抬 F，
+        // 续做各步的 txg 在那里算的不是真发的那一个；只进内存那一趟走真做那一段，txg 与真发相同。
         | PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. }
         | PublishError::WriteToAnInodeBeingDeleted { .. }
         | PublishError::WriteToAnInodeThatIsNotInTheTree { .. }
@@ -6173,10 +6266,36 @@ fn admit_the_writable_device_count(devices_handed_in: DeviceCount) -> Result<(),
 /// 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘 → 取号之前准入不够的，推抬 F 的空发布再判：可写挂载的后半段。
 fn establish_instance<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
-    devices: &mut Vec<(DeviceIdentity, Device)>,
+    devices_handed_in: &mut [(DeviceIdentity, Device)],
     mut allocator: PoolAllocator,
     start: InstanceStart,
 ) -> Result<Mounted, MountError> {
+    // 这次挂载往下的读，取号之前的几趟预演与取号之后的真做共用一份（岔路单 `research/prompts/m3-b44-code-r2-forks.md` 岔路一
+    // 「读一次两边共用」那一行，用户定）：每一块第一次读时才穿到底下那块盘、读到的字节记下，之后谁再读都取记下的那一份；
+    // 这次挂载写成的块读写下的。读阶段、恢复与重建分配器（这之前）读的不经它：它们自己的重读要打到盘上。
+    // 逐盘核、真做取号与实例表清扫三段照旧读盘（`ReadRouteOfThisMount::StraightToTheDisk`）：条款要它们在那一刻再读一次盘。
+    // 读坏之后当场重读的那一读（抬 F 算上限与重算影子账读根环、算 F 生效值重读根环与实例表、释放之前读盘核）也穿到盘上，
+    // 读到自证过的换掉记下的那一份（`recovery::PoolReaderOfOneReadPass`，`ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies`）。
+    let reads_failed_on_the_disk: RefCell<Vec<ReadFailedOnTheDisk>> = RefCell::new(Vec::new());
+    let read_route = Cell::new(ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun);
+    let mut devices_read_once_in_this_mount: Vec<(
+        DeviceIdentity,
+        DeviceReadOnceInThisMount<'_, '_, Device>,
+    )> = devices_handed_in
+        .iter_mut()
+        .map(|(identity, device)| {
+            (
+                *identity,
+                DeviceReadOnceInThisMount::over_the_disk(
+                    *identity,
+                    device,
+                    &reads_failed_on_the_disk,
+                    &read_route,
+                ),
+            )
+        })
+        .collect();
+    let devices = &mut devices_read_once_in_this_mount;
     let isolated_slots_per_device: Vec<(DeviceIdentity, u64)> = allocator
         .devices
         .iter()
@@ -6317,6 +6436,7 @@ fn establish_instance<Device: BlockDevice>(
     };
     // 挂载续做接着写行与暖机预演到最后的那一版整个预演一遍（续做要推抬 F 时连同取号之后那一串在只进内存的盘上走）：
     // 有一步要开的容器号到顶（C841）就在取号之前拒，盘上逐字节不变；不预演的话这一错在写行那次发布落盘之后才在续做里交回。
+    // 预演没走到续做的结局而读盘时有过读错，同样在取号之前拒（读错的那一块没记进共用的那一份，真做再读可能走出别的结局）。
     refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range(
         &pool,
         &version_after_the_last_publish,
@@ -6325,6 +6445,7 @@ fn establish_instance<Device: BlockDevice>(
         &start,
         &rows_written,
         &publishes_planned,
+        &reads_failed_on_the_disk,
     )?;
     // 可写挂载准入的逐盘核（D2（RAID 条带策略） 已定项 13、D18（块里携带什么信息） 已定项 11）：取号是这次挂载的第一个写，
     // 核在它之前、读的是紧接着要写的这批盘；有一块不带所选那一版就不取号，盘上逐字节不变。排在别的准入之后，
@@ -6341,13 +6462,15 @@ fn establish_instance<Device: BlockDevice>(
     } else {
         RootOfTheVersionInTheRing::NotWritten
     };
-    let devices_without = devices_without_the_selected_version(
-        &*pool.devices,
-        &start.system_configuration,
-        start.selected_version_journal_position,
-        &units_of_the_selected_version,
-        root_of_the_selected_version,
-    )
+    let devices_without = with_reads_straight_to_the_disk(&read_route, || {
+        devices_without_the_selected_version(
+            &*pool.devices,
+            &start.system_configuration,
+            start.selected_version_journal_position,
+            &units_of_the_selected_version,
+            root_of_the_selected_version,
+        )
+    })
     .map_err(|device| {
         refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
             start.rereads.read_stage,
@@ -6380,6 +6503,7 @@ fn establish_instance<Device: BlockDevice>(
         &start,
         &rows_written,
         publishes_planned,
+        ReadsOfTheAcquisition::StraightToTheDiskOfTheRealRun(&read_route),
     )?;
 
     // 续做之前这次挂载已落盘的发布（写行那次、暖机每次、写行之后推的抬 F 每一次空发布）各自的写，按先后：续做里的一步没发成、
@@ -6413,13 +6537,15 @@ fn establish_instance<Device: BlockDevice>(
     )?;
     // 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条「谁跑、什么时候跑」，用户 2026-10-03 定）：写行、暖机、抬 F、删除续做之后，
     // 交回之前判一次。清扫自己不让挂载失败（落点被拒这一轮不删、挂载照样交回可写，Q4），走到哪儿记进交回值。
-    let instance_table_sweep = sweep_the_instance_table_at_the_end_of_the_mount(
-        parameters,
-        devices,
-        &mut allocator,
-        &mut current,
-        start.instance_table_sweep,
-    );
+    let instance_table_sweep = with_reads_straight_to_the_disk(&read_route, || {
+        sweep_the_instance_table_at_the_end_of_the_mount(
+            parameters,
+            devices,
+            &mut allocator,
+            &mut current,
+            start.instance_table_sweep,
+        )
+    });
 
     Ok(Mounted {
         output: MountOutput {
@@ -6694,6 +6820,26 @@ struct PublishedBetweenAcquisitionAndTheContinuation {
     allocator: PoolAllocator,
 }
 
+/// `body` 里的读穿到盘上（[`ReadRouteOfThisMount::StraightToTheDisk`]），做完切回进来之前那一路。
+fn with_reads_straight_to_the_disk<Outcome>(
+    read_route: &Cell<ReadRouteOfThisMount>,
+    body: impl FnOnce() -> Outcome,
+) -> Outcome {
+    let route_before = read_route.replace(ReadRouteOfThisMount::StraightToTheDisk);
+    let outcome = body();
+    read_route.set(route_before);
+    outcome
+}
+
+/// 取号那几次读（[`acquire_then_publish_until_the_continuation`]）走哪一路。
+enum ReadsOfTheAcquisition<'route> {
+    /// 真做：取号写之前那几次读（重算要取的号、读上一个号、屏障前后各读一次见证值）穿到盘上；取号写起切回共用的那一份
+    /// （写系统配置槽时算生效 F 读的根环，预演那一趟读过、记下了）。
+    StraightToTheDiskOfTheRealRun(&'route Cell<ReadRouteOfThisMount>),
+    /// 取号之前只进内存那一趟：读没写过的块照它底下那一份的路读（共用的那一份）。
+    ThroughTheDevicesOfTheRehearsal,
+}
+
 /// 取号、写行、暖机，取号之前不够的再推抬 F（`push_floor_raises_after_the_row_publish`）：可写挂载取号之后、续做之前那一串。
 /// 可写挂载真发走它（`establish_instance`）；取号之前预演续做要推抬 F 时，在只进内存的盘与分配器的一份拷贝上也走它
 /// （[`walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`]），两边是同一段代码。
@@ -6709,6 +6855,7 @@ fn acquire_then_publish_until_the_continuation<Device: BlockDevice>(
     start: &InstanceStart,
     rows_written: &[InstanceRow],
     publishes_planned: PublishesPlannedAfterAcquisition,
+    reads_of_the_acquisition: ReadsOfTheAcquisition<'_>,
 ) -> Result<PublishedBetweenAcquisitionAndTheContinuation, MountError> {
     let PublishesPlannedAfterAcquisition {
         instance_to_acquire,
@@ -6718,7 +6865,22 @@ fn acquire_then_publish_until_the_continuation<Device: BlockDevice>(
         admission_before_acquisition,
     } = publishes_planned;
     let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
-    let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(
+    let witnessed = match reads_of_the_acquisition {
+        ReadsOfTheAcquisition::StraightToTheDiskOfTheRealRun(read_route) => {
+            with_reads_straight_to_the_disk(read_route, || {
+                read_the_witness_before_the_expected_acquisition_writes(
+                    &mut pool,
+                    instance_to_acquire,
+                )
+            })
+        }
+        ReadsOfTheAcquisition::ThroughTheDevicesOfTheRehearsal => {
+            read_the_witness_before_the_expected_acquisition_writes(&mut pool, instance_to_acquire)
+        }
+    };
+    let acquired = witnessed
+        .and_then(|witnessed| write_the_witnessed_expected_acquisition(&mut pool, witnessed));
+    let instance = acquired.map_err(
         |failure| match failure {
             ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {
                 expected,
@@ -6936,13 +7098,15 @@ fn acquire_then_publish_until_the_continuation<Device: BlockDevice>(
 /// 取号之前预演续做、续做要推抬 F 的那一格（[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）：
 /// 分配器拷贝上那一趟（`transaction::rehearse_every_step_of_the_deletion_queue_walk`）不读不写盘，推不了抬 F——抬 F 要读根环、实例表与
 /// 被抛弃根的账，还要先写系统配置、再发空发布——空间不够的那一步之后就接不上一版。这里改在只进内存的盘上
-/// （`mounted_session::DeviceKeepingWritesInMemory`：读穿到底下那块盘，写只记进内存）与 `allocator` 的一份拷贝上，把取号之后那一串
+/// （`mounted_session::DeviceKeepingWritesInMemory`：写只记进内存，读没写过的块穿到 `devices`）与 `allocator` 的一份拷贝上，把取号之后那一串
 /// （[`acquire_then_publish_until_the_continuation`]）连同续做（[`continue_the_deletion_queue_until_it_is_empty`]：被拒是空间不够就推一串抬 F、
-/// 再试，推满仍放不下就停）照真发走一遍：同一份计划、同一个推抬 F 的判法与同一段抬 F，续做报的错就是真发时会报的那一个。
-/// 底下那几块盘一个写都不发，真分配器不动。
+/// 再试，推满仍放不下就停）照真发走一遍：同一份计划、同一个推抬 F 的判法与同一段抬 F。`devices` 是这次挂载共用的那一份读
+/// （`mounted_session::DeviceReadOnceInThisMount`）：这一趟读成的块记在那里，真做再读同一块取的是同一份字节，所以读都读成时，
+/// 续做里报的错就是真发走到那一步时会报的那一个；这一趟有一次读穿到盘上读错了的，真做那一块要再读一次盘，可能读得成、走出别的结局，
+/// 这一格由调用方按读错账判（读错的那一块没记下）。底下那几块盘一个写都不发，真分配器不动。
 ///
-/// 交回续做里那一步报的错（`MountError::Publish` 的 `cause`：不是空间不够的那一种）；续做走到空、推满仍放不下、抬 F 自己报错，
-/// 或取号之后那一串在续做之前就报错，交回 `None`：那些照旧留给真发，真发时报它自己的。
+/// 交回走到哪儿：续做走到空或推满仍放不下（`WalkedToItsEnd`）；续做里一步报的错（`MountError::Publish` 的 `cause`，`StepRefused`）；
+/// 取号之后那一串在续做之前报错、续做里推的抬 F 自己报错（`StoppedBeforeTheEndOfTheContinuation`）。
 fn walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory<
     Device: BlockDevice,
 >(
@@ -6952,7 +7116,7 @@ fn walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory<
     start: &InstanceStart,
     rows_written: &[InstanceRow],
     publishes_planned: &PublishesPlannedAfterAcquisition,
-) -> Option<PublishError> {
+) -> ContinuationRehearsedBeforeAcquisition {
     let mut devices_keeping_writes_in_memory: Vec<(
         DeviceIdentity,
         DeviceKeepingWritesInMemory<'_>,
@@ -6963,21 +7127,24 @@ fn walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory<
             (*identity, DeviceKeepingWritesInMemory::over_the_disk(disk))
         })
         .collect();
-    let PublishedBetweenAcquisitionAndTheContinuation {
+    let Ok(PublishedBetweenAcquisitionAndTheContinuation {
         instance,
         space_admission,
         mut current,
         mut allocator,
         ..
-    } = acquire_then_publish_until_the_continuation(
+    }) = acquire_then_publish_until_the_continuation(
         parameters,
         &mut devices_keeping_writes_in_memory,
         allocator.clone(),
         start,
         rows_written,
         publishes_planned.clone(),
+        ReadsOfTheAcquisition::ThroughTheDevicesOfTheRehearsal,
     )
-    .ok()?;
+    else {
+        return ContinuationRehearsedBeforeAcquisition::StoppedBeforeTheEndOfTheContinuation;
+    };
     let continuation = continue_the_deletion_queue_until_it_is_empty(
         parameters,
         &mut devices_keeping_writes_in_memory,
@@ -6989,10 +7156,13 @@ fn walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory<
         Vec::new(),
         space_admission.floor_raises(),
     );
-    if let Err(MountError::Publish(failed)) = continuation {
-        Some(failed.cause)
+    let Err(error_of_the_continuation) = continuation else {
+        return ContinuationRehearsedBeforeAcquisition::WalkedToItsEnd;
+    };
+    if let MountError::Publish(failed) = error_of_the_continuation {
+        ContinuationRehearsedBeforeAcquisition::StepRefused(failed.cause)
     } else {
-        None
+        ContinuationRehearsedBeforeAcquisition::StoppedBeforeTheEndOfTheContinuation
     }
 }
 
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 2b8cbb46..000f2911 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -17,6 +17,7 @@
 //! 缺根那一半是 C585（会话发布放进发布途中取下的旧快照） 用户定的改法 C）。
 //! 发布路径与抬 F 各自开写入口的做法不变。
 
+use std::cell::{Cell, RefCell};
 use std::collections::BTreeMap;
 
 use crate::address::{
@@ -38,13 +39,14 @@ use crate::mount::{
     OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn, ParametersAndDeviceTableOfTheMount,
     RaisedFloor, ShadowLedger,
 };
+use crate::recovery::ReadPassOverThePool;
 use crate::root_record::UnmountMarker;
 use crate::root_ring::{target_for_publish, RootRingSlot};
 use crate::transaction::{
     prepare_the_version_publish, publish_new_inodes, publish_overwrite, publish_sequential_write,
     publish_unlink_intent, publish_version, walk_the_deletion_queue_until_it_is_empty,
     DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile, PoolVersion, PoolWriter,
-    PublishError, ReleaseChecksumCheck, TransactionOutput,
+    PublishError, ReadFailedOnTheDisk, ReleaseChecksumCheck, TransactionOutput,
 };
 use crate::unlink::DeletionSwitches;
 use singlefs_format::ROOT_RING_REGIONS;
@@ -355,7 +357,8 @@ impl StuckBehindTheWideProtection {
 
 /// 预演用的盘：读穿到底下那块盘（这一刻盘上的字节），写与写零只记进内存、盖在之后的读上，屏障什么都不做——一个写都不发到底下那块盘。
 /// 要出路时预演推环与推完之后这次改动的准入用（[`rehearse_the_push_then_the_remainder_of_the_change`]）；可写挂载取号之前预演续做、
-/// 续做要推抬 F 时，取号之后那一串连同续做也在它上面走一遍（`mount` 的 `walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`）。
+/// 续做要推抬 F 时，取号之后那一串连同续做也在它上面走一遍（`mount` 的 `walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`）：
+/// 那一处底下那块盘是这次挂载共用的那一份读（[`DeviceReadOnceInThisMount`]），这里读成的块在那里记下，真做再读取的是同一份。
 /// 底下那块盘只读借成 `&dyn BlockDevice`：改盘的三个动作要 `&mut`，类型上调不到。请求先过两个后端那一份判据
 /// （`block_device::check_aligned_and_in_range`），真盘会拒的这里照样拒。
 pub(crate) struct DeviceKeepingWritesInMemory<'disk> {
@@ -393,13 +396,15 @@ impl<'disk> DeviceKeepingWritesInMemory<'disk> {
     }
 
     /// 块号 `first_unwritten_block` 起、到 `end_block` 之前那一段没写过的块从底下那块盘读进 `buffer` 里对应的那一截
-    /// （`buffer` 从块号 `first_block_of_the_buffer` 起）。
+    /// （`buffer` 从块号 `first_block_of_the_buffer` 起）；`pass` 是当场重读那一遍时走底下那块盘的
+    /// [`BlockDevice::reread_at_after_a_bad_read`]。
     fn read_unwritten_blocks_from_the_disk(
         &self,
         first_unwritten_block: u64,
         end_block: u64,
         first_block_of_the_buffer: u64,
         buffer: &mut [u8],
+        pass: ReadPassOverThePool,
     ) -> Result<(), BlockDeviceError> {
         let block_bytes = self.block_bytes();
         let start_in_buffer =
@@ -407,29 +412,22 @@ impl<'disk> DeviceKeepingWritesInMemory<'disk> {
                 .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
         let end_in_buffer = usize::try_from((end_block - first_block_of_the_buffer) * block_bytes)
             .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
-        self.disk.read_at(
-            DeviceOffsetInBytes(first_unwritten_block * block_bytes),
-            &mut buffer[start_in_buffer..end_in_buffer],
-        )
-    }
-
-    fn keep_in_memory(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
-        let block_bytes = self.block_bytes();
-        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
-        let first_block = offset.0 / block_bytes;
-        // 迭代上界是这次写的块数；不跨轮带状态；没有提前出口。
-        for (block_offset, block) in (0u64..).zip(bytes.chunks(block_length)) {
-            self.written_blocks
-                .insert(first_block + block_offset, block.to_vec());
+        let offset = DeviceOffsetInBytes(first_unwritten_block * block_bytes);
+        let range_of_the_buffer = &mut buffer[start_in_buffer..end_in_buffer];
+        match pass {
+            ReadPassOverThePool::FirstRead => self.disk.read_at(offset, range_of_the_buffer),
+            ReadPassOverThePool::RereadAfterABadRead => self
+                .disk
+                .reread_at_after_a_bad_read(offset, range_of_the_buffer),
         }
     }
-}
 
-impl BlockDevice for DeviceKeepingWritesInMemory<'_> {
-    fn read_at(
+    /// 读一段：写过的块取内存里那一份，没写过的那几段从底下那块盘读（`pass` 定走 [`BlockDevice::read_at`] 还是当场重读那一读）。
+    fn read_over_the_blocks_kept_in_memory(
         &self,
         offset: DeviceOffsetInBytes,
         buffer: &mut [u8],
+        pass: ReadPassOverThePool,
     ) -> Result<(), BlockDeviceError> {
         let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
         self.check_request(offset, length)?;
@@ -451,6 +449,7 @@ impl BlockDevice for DeviceKeepingWritesInMemory<'_> {
                             block_index,
                             first_block,
                             buffer,
+                            pass,
                         )?;
                     }
                     let start_in_buffer = usize::try_from(block_index - first_block)
@@ -462,11 +461,67 @@ impl BlockDevice for DeviceKeepingWritesInMemory<'_> {
             }
         }
         if let Some(run_start) = unwritten_run_start {
-            self.read_unwritten_blocks_from_the_disk(run_start, end_block, first_block, buffer)?;
+            self.read_unwritten_blocks_from_the_disk(
+                run_start,
+                end_block,
+                first_block,
+                buffer,
+                pass,
+            )?;
         }
         Ok(())
     }
 
+    fn keep_in_memory(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
+        let block_bytes = self.block_bytes();
+        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
+        let first_block = offset.0 / block_bytes;
+        // 迭代上界是这次写的块数；不跨轮带状态；没有提前出口。
+        for (block_offset, block) in (0u64..).zip(bytes.chunks(block_length)) {
+            self.written_blocks
+                .insert(first_block + block_offset, block.to_vec());
+        }
+    }
+}
+
+impl BlockDevice for DeviceKeepingWritesInMemory<'_> {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        self.read_over_the_blocks_kept_in_memory(offset, buffer, ReadPassOverThePool::FirstRead)
+    }
+
+    /// 写过的块照旧取内存里那一份（这一趟写的，盘上没有）；没写过的走底下那块盘的当场重读那一读。
+    fn reread_at_after_a_bad_read(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        self.read_over_the_blocks_kept_in_memory(
+            offset,
+            buffer,
+            ReadPassOverThePool::RereadAfterABadRead,
+        )
+    }
+
+    /// 没写过的那几段交给底下那块盘（它们是从那里重读来的）；写过的块是这一趟写在内存里的，不交。
+    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
+        let block_bytes = self.block_bytes();
+        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
+        let first_block = offset.0 / block_bytes;
+        // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
+        for (block_index, block) in (first_block..).zip(bytes.chunks(block_length)) {
+            if !self.written_blocks.contains_key(&block_index) {
+                self.disk.keep_the_self_verified_reread(
+                    DeviceOffsetInBytes(block_index * block_bytes),
+                    block,
+                );
+            }
+        }
+    }
+
     fn write_at(
         &mut self,
         offset: DeviceOffsetInBytes,
@@ -502,6 +557,371 @@ impl BlockDevice for DeviceKeepingWritesInMemory<'_> {
     }
 }
 
+/// 可写挂载这次挂载里的读，取号之前的预演与取号之后的真做共用一份（岔路单 `research/prompts/m3-b44-code-r2-forks.md` 岔路一
+/// 「读一次两边共用」那一行，用户定）：每一块在这次挂载里第一次读时才穿到底下那块盘，读到的字节记下；之后再读这一块——预演
+/// （分配器拷贝上那一趟、只进内存那一趟，[`DeviceKeepingWritesInMemory`] 读没写过的块穿到它这里）还是真做（取号、写行、暖机、抬 F、续做）——
+/// 都取记下的那一份，不再读盘。这次挂载写成的块记成写下的字节，之后读它的读写下的。
+/// 写报错的那几块原来记着的丢掉（盘上那几块现在是什么不知道），之后读它们穿到盘上；读错的那一块不记，下一次读照样穿到盘上，
+/// 读错记进 `reads_failed_on_the_disk`（几块盘共用一份）：取号之前预演没走到续做的结局时拿它判（`mount::establish_instance`）。
+/// 记下的那一份活到这次挂载交回为止，读过的、写过的块都在里面，没有上限（岔路二「先量再定」归计数实验）。
+/// 读走哪一路看几块盘共用的 `read_route`（[`ReadRouteOfThisMount`]，`mount::establish_instance` 按段切）。
+/// 读坏之后当场重读那一读（[`BlockDevice::reread_at_after_a_bad_read`]）不取记下的那一份、穿到盘上读；调用方核过自证交回来的那几字节
+/// （[`BlockDevice::keep_the_self_verified_reread`]）换掉记下的那一份（[`ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies`]）。
+/// 请求先过两个后端那一份判据（`block_device::check_aligned_and_in_range`），真盘会拒的这里照样拒、不发到盘上。
+pub(crate) struct DeviceReadOnceInThisMount<'disk, 'log, Device> {
+    identity: DeviceIdentity,
+    disk: &'disk mut Device,
+    physical_block_size: PhysicalBlockSizeInBytes,
+    /// 这次挂载读成或写成的物理块：块号（按 `physical_block_size`）→ 那一块的字节。
+    blocks_read_or_written_in_this_mount: RefCell<BTreeMap<u64, Vec<u8>>>,
+    reads_failed_on_the_disk: &'log RefCell<Vec<ReadFailedOnTheDisk>>,
+    read_route: &'log Cell<ReadRouteOfThisMount>,
+}
+
+/// [`DeviceReadOnceInThisMount`] 的读此刻走哪一路。写不分路：写成的块一律记成写下的字节。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub(crate) enum ReadRouteOfThisMount {
+    /// 取记下的那一份，没记下的穿到盘上读、读成记下：预演与真做共用的那一份（岔路一「读一次两边共用」那一行）。
+    SharedBetweenTheRehearsalAndTheRealRun,
+    /// 穿到盘上读，不取也不改记下的那一份：条款要这一刻再读一次盘的那几段——取号之前逐盘核（读阶段之后的瞬时读错在这一遍报，
+    /// D18（块里携带什么信息） 已定项 11）、真做取号（屏障前后各读一次见证值，C554 乙-配置续 Q1）、实例表清扫（抹之前重读每个头，
+    /// 用户 2026-10-04 定 K1-b）。它们不在岔路单那一行说的「抬 F 与续做要读的东西」里，预演也不替它们读。
+    StraightToTheDisk,
+    /// 共用的那一段里读坏（读不出或自证不过）之后当场重读的那一读（[`BlockDevice::reread_at_after_a_bad_read`]；D19（块指针的结构与宽度预算）
+    /// 已定项 5、D16（发布语义） 已定项 1「根槽这一次读坏」那一行要它再读一次盘）：绕过记下的那一份、穿到盘上读，读错照样记进读错账；
+    /// 读到的字节调用方核过自证才交回来（[`BlockDevice::keep_the_self_verified_reread`]），换掉记下的那一份——预演里换了，真做再读取的就是它。
+    /// 第一次读回来的坏字节记下了也挡不住这一读（主 agent 判定：抬 F 与续做里「读坏当场重读」那一读不许被共用的那一份吃掉）。
+    /// 这一路只按每一次重读走，不装进几块盘共用的那一格。
+    RereadStraightToTheDiskKeepingWhatSelfVerifies,
+}
+
+impl ReadRouteOfThisMount {
+    /// 这一段里当场重读那一读走哪一路：共用的那一段改走穿到盘上、核过自证再换掉记下的那一份；照旧读盘的那几段本来就穿到盘上，
+    /// 也不改记下的那一份。
+    fn for_the_reread_after_a_bad_read(self) -> ReadRouteOfThisMount {
+        match self {
+            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun
+            | ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
+                ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies
+            }
+            ReadRouteOfThisMount::StraightToTheDisk => ReadRouteOfThisMount::StraightToTheDisk,
+        }
+    }
+}
+
+impl<'disk, 'log, Device: BlockDevice> DeviceReadOnceInThisMount<'disk, 'log, Device> {
+    pub(crate) fn over_the_disk(
+        identity: DeviceIdentity,
+        disk: &'disk mut Device,
+        reads_failed_on_the_disk: &'log RefCell<Vec<ReadFailedOnTheDisk>>,
+        read_route: &'log Cell<ReadRouteOfThisMount>,
+    ) -> Self {
+        let physical_block_size = disk.probe_physical_block_size();
+        DeviceReadOnceInThisMount {
+            identity,
+            disk,
+            physical_block_size,
+            blocks_read_or_written_in_this_mount: RefCell::new(BTreeMap::new()),
+            reads_failed_on_the_disk,
+            read_route,
+        }
+    }
+
+    fn block_bytes(&self) -> u64 {
+        u64::from(self.physical_block_size.0)
+    }
+
+    /// 两个后端那一份判据（`block_device::check_aligned_and_in_range`），设备大小取底下那块盘的。
+    fn check_request(
+        &self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        let device_size = self.disk.size_in_bytes();
+        check_aligned_and_in_range(offset, length, self.physical_block_size, device_size)
+    }
+
+    /// 块号 `first_unrecorded_block` 起、到 `end_block` 之前那一段没记下的块从底下那块盘读进 `buffer` 里对应的那一截
+    /// （`buffer` 从块号 `first_block_of_the_buffer` 起）：读成就逐块记下；读错记进读错账、原样交回。
+    fn read_unrecorded_blocks_from_the_disk(
+        &self,
+        first_unrecorded_block: u64,
+        end_block: u64,
+        first_block_of_the_buffer: u64,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        let block_bytes = self.block_bytes();
+        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
+        let start_in_buffer =
+            usize::try_from((first_unrecorded_block - first_block_of_the_buffer) * block_bytes)
+                .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
+        let end_in_buffer = usize::try_from((end_block - first_block_of_the_buffer) * block_bytes)
+            .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
+        let offset = DeviceOffsetInBytes(first_unrecorded_block * block_bytes);
+        if let Err(error) = self
+            .disk
+            .read_at(offset, &mut buffer[start_in_buffer..end_in_buffer])
+        {
+            self.reads_failed_on_the_disk
+                .borrow_mut()
+                .push(ReadFailedOnTheDisk {
+                    device: self.identity,
+                    offset,
+                    length_in_bytes: (end_block - first_unrecorded_block) * block_bytes,
+                    error: block_device_error_with_the_same_content(&error),
+                });
+            return Err(error);
+        }
+        let mut recorded = self.blocks_read_or_written_in_this_mount.borrow_mut();
+        // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
+        for (block_index, block) in (first_unrecorded_block..end_block)
+            .zip(buffer[start_in_buffer..end_in_buffer].chunks(block_length))
+        {
+            recorded.insert(block_index, block.to_vec());
+        }
+        Ok(())
+    }
+
+    /// 穿到盘上读、不取也不改记下的那一份（当场重读那一读）：读错记进读错账、原样交回。
+    fn reread_straight_from_the_disk(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
+        if let Err(error) = self.disk.read_at(offset, buffer) {
+            self.reads_failed_on_the_disk
+                .borrow_mut()
+                .push(ReadFailedOnTheDisk {
+                    device: self.identity,
+                    offset,
+                    length_in_bytes,
+                    error: block_device_error_with_the_same_content(&error),
+                });
+            return Err(error);
+        }
+        Ok(())
+    }
+
+    /// 读一段，照 `route` 走（[`ReadRouteOfThisMount`]）。
+    fn read_by_the_route(
+        &self,
+        route: ReadRouteOfThisMount,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
+        self.check_request(offset, length)?;
+        match route {
+            ReadRouteOfThisMount::StraightToTheDisk => return self.disk.read_at(offset, buffer),
+            ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
+                return self.reread_straight_from_the_disk(offset, buffer)
+            }
+            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun => {}
+        }
+        let block_bytes = self.block_bytes();
+        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
+        let first_block = offset.0 / block_bytes;
+        let end_block = first_block + length / block_bytes;
+        // 迭代上界是这次读的块数；跨轮带着还没读的那一段没记下的块从哪一块起；出口只有底下那块盘读错（原样交回）。
+        let mut unrecorded_run_start: Option<u64> = None;
+        for block_index in first_block..end_block {
+            let is_recorded_in_this_mount = self
+                .blocks_read_or_written_in_this_mount
+                .borrow()
+                .contains_key(&block_index);
+            if !is_recorded_in_this_mount {
+                unrecorded_run_start.get_or_insert(block_index);
+                continue;
+            }
+            if let Some(run_start) = unrecorded_run_start.take() {
+                self.read_unrecorded_blocks_from_the_disk(
+                    run_start,
+                    block_index,
+                    first_block,
+                    buffer,
+                )?;
+            }
+            let start_in_buffer = usize::try_from(block_index - first_block)
+                .expect("块数装得进 usize")
+                * block_length;
+            let recorded = self.blocks_read_or_written_in_this_mount.borrow();
+            buffer[start_in_buffer..start_in_buffer + block_length].copy_from_slice(
+                recorded
+                    .get(&block_index)
+                    .expect("这一块刚判过记着：判与取之间没有写"),
+            );
+        }
+        if let Some(run_start) = unrecorded_run_start {
+            self.read_unrecorded_blocks_from_the_disk(run_start, end_block, first_block, buffer)?;
+        }
+        Ok(())
+    }
+
+    /// 写成的那几块记成写下的字节。
+    fn record_the_written_blocks(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
+        let block_bytes = self.block_bytes();
+        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
+        let first_block = offset.0 / block_bytes;
+        let recorded = self.blocks_read_or_written_in_this_mount.get_mut();
+        // 迭代上界是这次写的块数；不跨轮带状态；没有提前出口。
+        for (block_offset, block) in (0u64..).zip(bytes.chunks(block_length)) {
+            recorded.insert(first_block + block_offset, block.to_vec());
+        }
+    }
+
+    /// 写报错的那几块：原来记着的丢掉，之后读它们穿到盘上。
+    fn forget_the_blocks_of_a_failed_write(&mut self, offset: DeviceOffsetInBytes, length: u64) {
+        let block_bytes = self.block_bytes();
+        let first_block = offset.0 / block_bytes;
+        let end_block = first_block + length / block_bytes;
+        let recorded = self.blocks_read_or_written_in_this_mount.get_mut();
+        // 迭代上界是这次写的块数；不跨轮带状态；没有提前出口。
+        for block_index in first_block..end_block {
+            recorded.remove(&block_index);
+        }
+    }
+}
+
+/// 读错记进读错账要一份自己的：盘报的错交回给读的那一方，账里另存一份内容相同的（`BlockDeviceError` 带着 `io::Error`，不能 `Clone`）。
+fn block_device_error_with_the_same_content(error: &BlockDeviceError) -> BlockDeviceError {
+    match error {
+        BlockDeviceError::OutOfRange {
+            offset,
+            length,
+            device_size,
+        } => BlockDeviceError::OutOfRange {
+            offset: *offset,
+            length: *length,
+            device_size: *device_size,
+        },
+        BlockDeviceError::Unaligned {
+            offset,
+            length,
+            physical_block_size,
+        } => BlockDeviceError::Unaligned {
+            offset: *offset,
+            length: *length,
+            physical_block_size: *physical_block_size,
+        },
+        BlockDeviceError::InputOutput(input_output_error) => BlockDeviceError::InputOutput(
+            std::io::Error::new(input_output_error.kind(), input_output_error.to_string()),
+        ),
+        BlockDeviceError::ProbeFailed { attribute } => BlockDeviceError::ProbeFailed { attribute },
+        BlockDeviceError::ImageFileAlreadyExists { path } => {
+            BlockDeviceError::ImageFileAlreadyExists { path: path.clone() }
+        }
+        BlockDeviceError::ImageFileMissing { path } => {
+            BlockDeviceError::ImageFileMissing { path: path.clone() }
+        }
+    }
+}
+
+impl<Device: BlockDevice> BlockDevice for DeviceReadOnceInThisMount<'_, '_, Device> {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        self.read_by_the_route(self.read_route.get(), offset, buffer)
+    }
+
+    fn reread_at_after_a_bad_read(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        self.read_by_the_route(
+            self.read_route.get().for_the_reread_after_a_bad_read(),
+            offset,
+            buffer,
+        )
+    }
+
+    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
+        match self.read_route.get().for_the_reread_after_a_bad_read() {
+            ReadRouteOfThisMount::StraightToTheDisk => {}
+            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun
+            | ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
+                let length = u64::try_from(bytes.len()).expect("重读的长度装得进 u64");
+                assert!(
+                    self.check_request(offset, length).is_ok(),
+                    "交回来的是一次读成的重读：偏移与长度过了同一份对齐与越界判据"
+                );
+                let block_bytes = self.block_bytes();
+                let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
+                let first_block = offset.0 / block_bytes;
+                let mut recorded = self.blocks_read_or_written_in_this_mount.borrow_mut();
+                // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
+                for (block_index, block) in (first_block..).zip(bytes.chunks(block_length)) {
+                    recorded.insert(block_index, block.to_vec());
+                }
+            }
+        }
+    }
+
+    fn write_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+        durability: WriteDurability,
+    ) -> Result<(), BlockDeviceError> {
+        let length = u64::try_from(bytes.len()).expect("写长装得进 u64");
+        self.check_request(offset, length)?;
+        match self.disk.write_at(offset, bytes, durability) {
+            Ok(()) => {
+                self.record_the_written_blocks(offset, bytes);
+                Ok(())
+            }
+            Err(error) => {
+                self.forget_the_blocks_of_a_failed_write(offset, length);
+                Err(error)
+            }
+        }
+    }
+
+    fn write_zeroes_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        self.check_request(offset, length)?;
+        match self.disk.write_zeroes_at(offset, length) {
+            Ok(()) => {
+                let zeros =
+                    vec![0u8; usize::try_from(length).expect("写零的长度装得进 usize")];
+                self.record_the_written_blocks(offset, &zeros);
+                Ok(())
+            }
+            Err(error) => {
+                self.forget_the_blocks_of_a_failed_write(offset, length);
+                Err(error)
+            }
+        }
+    }
+
+    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+        self.disk.barrier()
+    }
+
+    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
+        self.physical_block_size
+    }
+
+    fn size_in_bytes(&self) -> u64 {
+        self.disk.size_in_bytes()
+    }
+
+    fn journal_record_offsets_hint(
+        &self,
+        ring_start: DeviceOffsetInBytes,
+        ring_bytes: u64,
+    ) -> Option<Vec<DeviceOffsetInBytes>> {
+        self.disk
+            .journal_record_offsets_hint(ring_start, ring_bytes)
+    }
+}
+
 /// 被拒的那一次发布之后，这次用户改动还剩什么要发（「被宽护卡住」那一报记一份拷贝，要出路时推完环之后照它再预演一次，
 /// [`rehearse_the_push_then_the_remainder_of_the_change`]）。
 #[derive(Clone, Copy, Debug)]
@@ -939,7 +1359,9 @@ pub fn refusal_is_short_of_space(refusal: &PublishError) -> bool {
         | PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. }
         // `deleted_inodes` 装不下：类型化错误（D8（核心索引结构） 已定项 16 第 6 条），与空间无关。
         | PublishError::DeletedInodesQueueRefused(_)
-        | PublishError::BlockDevice(_) => false,
+        | PublishError::BlockDevice(_)
+        // 取号之前预演读错那一拒只在可写挂载取号之前那一判里造，发布路径不报它。
+        | PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(_) => false,
     }
 }
 
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index c8196195..527bb738 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -125,6 +125,138 @@ pub trait PoolReader {
         ring_start: DeviceOffsetInBytes,
         ring_bytes: u64,
     ) -> Option<Vec<DeviceOffsetInBytes>>;
+    /// 读坏之后当场重读的那一读（[`BlockDevice::reread_at_after_a_bad_read`]）。默认就是 [`PoolReader::read`]。
+    /// 调用方不直接调它：当场重读的那几处经 [`PoolReaderOfOneReadPass::rereading_after_a_bad_read`] 读。
+    fn reread_after_a_bad_read(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length: usize,
+    ) -> Option<Vec<u8>> {
+        self.read(device, offset, length)
+    }
+    /// 当场重读那一读读到、核过自证的字节交给盘那一层（[`BlockDevice::keep_the_self_verified_reread`]）。默认什么都不做。
+    /// 调用方不直接调它：经 [`PoolReader::note_that_the_bytes_read_self_verify`] 由重读那一遍的读者转过来。
+    fn keep_the_self_verified_reread(
+        &self,
+        _device: DeviceIdentity,
+        _offset: DeviceOffsetInBytes,
+        _bytes: &[u8],
+    ) {
+    }
+    /// 读的一方核过自证（根槽自证、单元校验和与位置项相符）之后报一声：这几字节是从 `device` 的 `offset` 起读到的、自证过。
+    /// 默认什么都不做；当场重读那一遍的读者（[`PoolReaderOfOneReadPass`] 的重读那一种）把它转成
+    /// [`PoolReader::keep_the_self_verified_reread`]：重读读到自证过的就换掉这次挂载记下的那一份。
+    fn note_that_the_bytes_read_self_verify(
+        &self,
+        _device: DeviceIdentity,
+        _offset: DeviceOffsetInBytes,
+        _bytes: &[u8],
+    ) {
+    }
+}
+
+/// 一遍读是第一次读，还是读坏之后当场重读的那一遍（D19（块指针的结构与宽度预算） 已定项 5、D16（发布语义） 已定项 1「根槽这一次读坏」那一行
+/// 与「重读一次」：只重读一次，重读要再读一次盘）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ReadPassOverThePool {
+    FirstRead,
+    /// 读走 [`PoolReader::reread_after_a_bad_read`]：可写挂载里绕过这次挂载记下的那一份、直接读底下那块盘
+    /// （主 agent 判定：抬 F 与续做里「读坏当场重读」那一读不许被预演与真做共用的那一份吃掉）；读到自证过的换掉记下的那一份。
+    RereadAfterABadRead,
+}
+
+/// 一遍读的读者：第一次读照原样读；读坏之后当场重读的那一遍，读走 [`PoolReader::reread_after_a_bad_read`]，
+/// 读的一方核过自证报上来的字节（[`PoolReader::note_that_the_bytes_read_self_verify`]）转成 [`PoolReader::keep_the_self_verified_reread`]。
+pub struct PoolReaderOfOneReadPass<'reader, Reader: PoolReader + ?Sized> {
+    reader: &'reader Reader,
+    pass: ReadPassOverThePool,
+}
+
+impl<'reader, Reader: PoolReader + ?Sized> PoolReaderOfOneReadPass<'reader, Reader> {
+    #[must_use]
+    pub fn reading_for_the_first_time(reader: &'reader Reader) -> Self {
+        PoolReaderOfOneReadPass {
+            reader,
+            pass: ReadPassOverThePool::FirstRead,
+        }
+    }
+
+    #[must_use]
+    pub fn rereading_after_a_bad_read(reader: &'reader Reader) -> Self {
+        PoolReaderOfOneReadPass {
+            reader,
+            pass: ReadPassOverThePool::RereadAfterABadRead,
+        }
+    }
+
+    #[must_use]
+    pub fn in_the_pass(reader: &'reader Reader, pass: ReadPassOverThePool) -> Self {
+        PoolReaderOfOneReadPass { reader, pass }
+    }
+}
+
+impl<Reader: PoolReader + ?Sized> PoolReader for PoolReaderOfOneReadPass<'_, Reader> {
+    fn device_identities(&self) -> Vec<DeviceIdentity> {
+        self.reader.device_identities()
+    }
+    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
+        self.reader.device_size_in_bytes(device)
+    }
+    fn read(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length: usize,
+    ) -> Option<Vec<u8>> {
+        match self.pass {
+            ReadPassOverThePool::FirstRead => self.reader.read(device, offset, length),
+            ReadPassOverThePool::RereadAfterABadRead => {
+                self.reader.reread_after_a_bad_read(device, offset, length)
+            }
+        }
+    }
+    fn journal_record_offsets_hint(
+        &self,
+        device: DeviceIdentity,
+        ring_start: DeviceOffsetInBytes,
+        ring_bytes: u64,
+    ) -> Option<Vec<DeviceOffsetInBytes>> {
+        self.reader
+            .journal_record_offsets_hint(device, ring_start, ring_bytes)
+    }
+    fn reread_after_a_bad_read(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length: usize,
+    ) -> Option<Vec<u8>> {
+        self.reader.reread_after_a_bad_read(device, offset, length)
+    }
+    fn keep_the_self_verified_reread(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+    ) {
+        self.reader
+            .keep_the_self_verified_reread(device, offset, bytes);
+    }
+    fn note_that_the_bytes_read_self_verify(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+    ) {
+        match self.pass {
+            ReadPassOverThePool::FirstRead => self
+                .reader
+                .note_that_the_bytes_read_self_verify(device, offset, bytes),
+            ReadPassOverThePool::RereadAfterABadRead => self
+                .reader
+                .keep_the_self_verified_reread(device, offset, bytes),
+        }
+    }
 }
 
 impl<Device: BlockDevice> PoolReader for [(DeviceIdentity, Device)] {
@@ -156,6 +288,29 @@ impl<Device: BlockDevice> PoolReader for [(DeviceIdentity, Device)] {
         let (_, block_device) = self.iter().find(|(identity, _)| *identity == device)?;
         block_device.journal_record_offsets_hint(ring_start, ring_bytes)
     }
+    fn reread_after_a_bad_read(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length: usize,
+    ) -> Option<Vec<u8>> {
+        let (_, block_device) = self.iter().find(|(identity, _)| *identity == device)?;
+        let mut buffer = vec![0u8; length];
+        block_device
+            .reread_at_after_a_bad_read(offset, &mut buffer)
+            .ok()?;
+        Some(buffer)
+    }
+    fn keep_the_self_verified_reread(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+    ) {
+        if let Some((_, block_device)) = self.iter().find(|(identity, _)| *identity == device) {
+            block_device.keep_the_self_verified_reread(offset, bytes);
+        }
+    }
 }
 
 /// 文件后端的池就是一串盘：读法与切片那一份相同。
@@ -182,6 +337,22 @@ impl<Device: BlockDevice> PoolReader for Vec<(DeviceIdentity, Device)> {
     ) -> Option<Vec<DeviceOffsetInBytes>> {
         PoolReader::journal_record_offsets_hint(self.as_slice(), device, ring_start, ring_bytes)
     }
+    fn reread_after_a_bad_read(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length: usize,
+    ) -> Option<Vec<u8>> {
+        PoolReader::reread_after_a_bad_read(self.as_slice(), device, offset, length)
+    }
+    fn keep_the_self_verified_reread(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+    ) {
+        PoolReader::keep_the_self_verified_reread(self.as_slice(), device, offset, bytes);
+    }
 }
 
 /// 恢复走不下去的原因，按调用方能据以行动的粒度分。
@@ -399,6 +570,11 @@ pub(crate) fn read_unit_via_locations<Reader: PoolReader + ?Sized>(
             continue;
         };
         if crc32_castagnoli(&bytes) == location.unit_checksum {
+            reader.note_that_the_bytes_read_self_verify(
+                location.device,
+                location.slot.to_device_offset(),
+                &bytes,
+            );
             return Ok(bytes);
         }
     }
@@ -1259,6 +1435,7 @@ pub enum RootRingSlotReading {
 }
 
 /// 读一个根环槽（区域归属表给出它在哪块盘、槽距给出偏移），验自证校验和与 fsid。只读一次，不重试。
+/// 自证过的字节报给读者（[`PoolReader::note_that_the_bytes_read_self_verify`]：当场重读那一遍的读者拿它换掉这次挂载记下的那一份）。
 #[must_use]
 pub fn read_root_ring_slot<Reader: PoolReader + ?Sized>(
     reader: &Reader,
@@ -1274,7 +1451,10 @@ pub fn read_root_ring_slot<Reader: PoolReader + ?Sized>(
         return RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable);
     };
     match RootRecord::parse_slot(&bytes, filesystem_identifier) {
-        Some(root) => RootRingSlotReading::SelfVerified(root),
+        Some(root) => {
+            reader.note_that_the_bytes_read_self_verify(device, offset, &bytes);
+            RootRingSlotReading::SelfVerified(root)
+        }
         None => RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified),
     }
 }
@@ -1430,6 +1610,17 @@ pub fn readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_
             ring_slot,
         )
     };
+    // 重读那一读绕过这次挂载记下的那一份、再读一次盘（[`ReadPassOverThePool::RereadAfterABadRead`]）。
+    let rereading_reader = PoolReaderOfOneReadPass::rereading_after_a_bad_read(reader);
+    let reread_the_slot = |ring_slot| {
+        read_root_ring_slot(
+            &rereading_reader,
+            region_devices,
+            immutable_sizes,
+            filesystem_identifier,
+            ring_slot,
+        )
+    };
     let mut roots = Vec::new();
     // 迭代次数的上界是根环槽数 R × S；跨轮只带已认出的根。提前出口只有「知道住着根的槽重读仍坏」一个。
     for ring_slot in every_root_ring_slot(immutable_sizes) {
@@ -1443,7 +1634,7 @@ pub fn readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_
         if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {
             continue;
         }
-        match read_the_slot(ring_slot) {
+        match reread_the_slot(ring_slot) {
             RootRingSlotReading::SelfVerified(root) => roots.push((ring_slot, root)),
             RootRingSlotReading::Bad(reread) => {
                 return Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {
@@ -1654,9 +1845,9 @@ pub fn effective_rollback_floor_rereading_the_newest_instance_table_once<
     immutable_sizes: &SystemImmutableSizes,
     filesystem_identifier: &[u8; 16],
 ) -> Result<CheckpointTxg, InstanceTableOfTheNewestRootStillUnreadableAfterOneReread> {
-    let read_the_ring = || {
+    let read_the_ring = |pass| {
         Ok::<_, std::convert::Infallible>(readable_roots_with_ring_slots(
-            reader,
+            &PoolReaderOfOneReadPass::in_the_pass(reader, pass),
             region_devices,
             immutable_sizes,
             filesystem_identifier,
@@ -1699,9 +1890,9 @@ pub fn effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instan
     filesystem_identifier: &[u8; 16],
     ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
 ) -> Result<CheckpointTxg, EffectiveFloorReadingStillUnreadableAfterOneReread> {
-    let read_the_ring = || {
+    let read_the_ring = |pass| {
         readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once(
-            reader,
+            &PoolReaderOfOneReadPass::in_the_pass(reader, pass),
             region_devices,
             immutable_sizes,
             filesystem_identifier,
@@ -1743,10 +1934,10 @@ pub fn effective_rollback_floor_rereading_unreadable_reads_once<Reader: PoolRead
     immutable_sizes: &SystemImmutableSizes,
     filesystem_identifier: &[u8; 16],
 ) -> CheckpointTxg {
-    let read_the_ring = || {
+    let read_the_ring = |pass| {
         Ok::<_, std::convert::Infallible>(
             readable_roots_with_ring_slots_rereading_unreadable_slots_once(
-                reader,
+                &PoolReaderOfOneReadPass::in_the_pass(reader, pass),
                 region_devices,
                 immutable_sizes,
                 filesystem_identifier,
@@ -1792,7 +1983,7 @@ enum EffectiveFloorAfterAtMostOneReread {
 
 /// 读一遍根环（`read_the_ring`：怎么读、读坏的槽怎么办由调用方定）、择最新那条根、沿链读它的实例表；表读不出、解不开或一条根都择不到时
 /// 立即把根环与表一起重读一次（C554 乙的形态，R = 1；D16（发布语义） 已定项 1「重读一次」），读出表就按它算 F 生效值。
-/// 第一次读与重读一次走同一段。
+/// 第一次读与重读一次走同一段；重读那一遍根环与表都按 [`ReadPassOverThePool::RereadAfterABadRead`] 读（`read_the_ring` 收到哪一遍）。
 ///
 /// # Errors
 /// `read_the_ring` 的错原样交回（那一遍之后不再读）。
@@ -1804,13 +1995,21 @@ fn effective_rollback_floor_rereading_the_newest_instance_table_once_over<
     region_devices: &[DeviceIdentity; 3],
     immutable_sizes: &SystemImmutableSizes,
     filesystem_identifier: &[u8; 16],
-    read_the_ring: impl Fn() -> Result<Vec<(RootRingSlot, RootRecord)>, RingReadingFailure>,
+    read_the_ring: impl Fn(
+        ReadPassOverThePool,
+    ) -> Result<Vec<(RootRingSlot, RootRecord)>, RingReadingFailure>,
 ) -> Result<EffectiveFloorAfterAtMostOneReread, RingReadingFailure> {
-    let read_the_ring_and_the_newest_roots_table = || {
-        let roots_with_ring_slots = read_the_ring()?;
+    let read_the_ring_and_the_newest_roots_table = |pass: ReadPassOverThePool| {
+        let roots_with_ring_slots = read_the_ring(pass)?;
         let newest = newest_root_among(&roots_with_ring_slots);
         let newest_roots_table = newest
-            .and_then(|newest| instance_table_chain_of_root(reader, &newest).ok())
+            .and_then(|newest| {
+                instance_table_chain_of_root(
+                    &PoolReaderOfOneReadPass::in_the_pass(reader, pass),
+                    &newest,
+                )
+                .ok()
+            })
             .map(|chain| chain.records);
         Ok((roots_with_ring_slots, newest, newest_roots_table))
     };
@@ -1827,19 +2026,23 @@ fn effective_rollback_floor_rereading_the_newest_instance_table_once_over<
             ),
         )
     };
-    if let (roots_with_ring_slots, _, Some(table)) = read_the_ring_and_the_newest_roots_table()? {
+    if let (roots_with_ring_slots, _, Some(table)) =
+        read_the_ring_and_the_newest_roots_table(ReadPassOverThePool::FirstRead)?
+    {
         return Ok(floor_under(&roots_with_ring_slots, &table));
     }
-    Ok(match read_the_ring_and_the_newest_roots_table()? {
-        (roots_with_ring_slots, _, Some(table)) => floor_under(&roots_with_ring_slots, &table),
-        (roots_with_ring_slots_on_the_reread, newest_on_the_reread, None) => {
-            EffectiveFloorAfterAtMostOneReread::NewestRootsTableStillUnreadable {
-                roots_with_ring_slots_on_the_reread,
-                newest_root_on_the_reread: newest_on_the_reread
-                    .map(|root| (root.instance, root.checkpoint_txg)),
+    Ok(
+        match read_the_ring_and_the_newest_roots_table(ReadPassOverThePool::RereadAfterABadRead)? {
+            (roots_with_ring_slots, _, Some(table)) => floor_under(&roots_with_ring_slots, &table),
+            (roots_with_ring_slots_on_the_reread, newest_on_the_reread, None) => {
+                EffectiveFloorAfterAtMostOneReread::NewestRootsTableStillUnreadable {
+                    roots_with_ring_slots_on_the_reread,
+                    newest_root_on_the_reread: newest_on_the_reread
+                        .map(|root| (root.instance, root.checkpoint_txg)),
+                }
             }
-        }
-    })
+        },
+    )
 }
 
 /// 读出来的根里 (checkpoint_txg, 实例代号) 最大的那一条（与 [`choose_root`] 同一个择法）；一条都没有时 `None`。
@@ -2546,6 +2749,7 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
     }
     let record =
         record_standing_for_root.ok_or(RebuildVersionFailure::NoRecordStandingForFileVersion)?;
+    new_root_segment_pointers_of_the_record_ascend_by_device(&record)?;
     let record_bytes = record.to_bytes();
     let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)
         .map_err(RecoveryFailure::from)?;
@@ -2788,7 +2992,7 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
                 || node
                     .entries
                     .iter()
-                    .any(|entry| entry.len() < DELETED_INODES_KEY_WIDTH_IN_BYTES)
+                    .any(|entry| entry.len() != DELETED_INODES_KEY_WIDTH_IN_BYTES)
             {
                 return Err(RecoveryFailure::UnitMalformed {
                     what: "deleted_inodes 根不是层级 0、条目宽 8 的非空节点",
@@ -5115,6 +5319,27 @@ fn root_record_pointers_ascend_by_device(root: &RootRecord) -> Result<(), Recove
     Ok(())
 }
 
+/// 顶着这一版的那条 journal 记录新根段里的四条指针（树表、映射树根、实例表、分配记录树根）按 I-2.5 判（全零的豁免），
+/// 判法与根记录那四条（[`root_record_pointers_ascend_by_device`]）、重建出来的这一版（[`pointers_of_the_rebuilt_version_ascend_by_device`]）
+/// 同一个：逐条 `NodePointer::location_entries_ascend_by_device`、坏的报 [`location_entries_not_ascending`]。
+/// 重建一拿到这条记录就重新编码它（`JournalRecord::to_bytes`），写者 `NodePointer::write_to` 在那里断言升序：
+/// 那一步之前不判，坏镜像就在可写挂载的那条断言上 panic（I-2.5 罩「任一块指针」，新根段这四条也是盘上读来的块指针）。
+fn new_root_segment_pointers_of_the_record_ascend_by_device(
+    record: &JournalRecord,
+) -> Result<(), RecoveryFailure> {
+    for pointer in [
+        &record.new_tree_table,
+        &record.new_mapping_root,
+        &record.new_instance_table,
+        &record.new_allocation_record_tree_root,
+    ] {
+        pointer
+            .location_entries_ascend_by_device()
+            .map_err(|_not_ascending| location_entries_not_ascending())?;
+    }
+    Ok(())
+}
+
 /// 一棵容器树（墓碑树、谱系树、退役树）这一版里盘上读来的指针：每片容器的指针与根之下每个码 2 节点的指针（根的指针住树表条目，
 /// 随树表条目判）。下一次重写这棵树的某个节点时，没换的条目连同它们的指针照抄写回。
 fn container_tree_pointers_read_from_disk<'version>(
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index fe8c2ef5..14587217 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -112,7 +112,7 @@ use crate::recovery::{
     effective_rollback_floor_rereading_unreadable_reads_once, highest_root_instance,
     system_configuration_slots_of_one_device_read_once,
     system_configuration_witness_is_undecidable, tree_table_entry_count,
-    verified_system_configuration_slots, PoolReader, RecoveryFailure,
+    verified_system_configuration_slots, PoolReader, PoolReaderOfOneReadPass, RecoveryFailure,
     SystemConfigurationSlotsOfOneDevice,
 };
 use crate::retirement_tree::{
@@ -995,6 +995,20 @@ pub fn acquire_expected_instance<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     expected: InstanceGeneration,
 ) -> Result<InstanceGeneration, ExpectedInstanceAcquisitionFailed> {
+    let witnessed = read_the_witness_before_the_expected_acquisition_writes(pool, expected)?;
+    write_the_witnessed_expected_acquisition(pool, witnessed)
+}
+
+/// [`acquire_expected_instance`] 的头一半：写之前重算要取的号、与 `expected` 比，再照 [`read_the_witness_before_the_acquisition_writes`]
+/// 读上一个号、核见证值、发一道屏障、再读见证值。只读盘与发那一道屏障，一个字节都不写：可写挂载在这一半里让读穿到盘上
+/// （`mount::acquire_then_publish_until_the_continuation`，C554 乙-配置续 Q1 的两次读要读盘）。
+///
+/// # Errors
+/// 同 [`acquire_expected_instance`] 写之前的那几种（号变了、号到顶、判不出、某块盘一份自证过的都没有、那道屏障报错）。
+pub(crate) fn read_the_witness_before_the_expected_acquisition_writes<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    expected: InstanceGeneration,
+) -> Result<AcquisitionWitnessedBeforeItsWrites, ExpectedInstanceAcquisitionFailed> {
     let recomputed = next_instance_generation_to_acquire(pool)
         .map_err(ExpectedInstanceAcquisitionFailed::InstanceGenerationPastTheTopOfItsRange)?;
     if recomputed != expected {
@@ -1005,7 +1019,25 @@ pub fn acquire_expected_instance<Device: BlockDevice>(
             },
         );
     }
-    write_acquired_instance(pool, recomputed).map_err(|failure| match failure {
+    read_the_witness_before_the_acquisition_writes(pool, recomputed)
+        .map_err(expected_acquisition_failure_of)
+}
+
+/// [`acquire_expected_instance`] 的后一半：照头一半读到的写（[`write_the_witnessed_acquisition`]）。
+///
+/// # Errors
+/// 取号写或写之后那道屏障报错 ⇒ `Acquisition`（回卷的结局在里面）。
+pub(crate) fn write_the_witnessed_expected_acquisition<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    witnessed: AcquisitionWitnessedBeforeItsWrites,
+) -> Result<InstanceGeneration, ExpectedInstanceAcquisitionFailed> {
+    write_the_witnessed_acquisition(pool, witnessed).map_err(expected_acquisition_failure_of)
+}
+
+fn expected_acquisition_failure_of(
+    failure: InstanceAcquisitionFailed,
+) -> ExpectedInstanceAcquisitionFailed {
+    match failure {
         InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
             first_device_reporting_a_read_error,
         } => ExpectedInstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
@@ -1019,19 +1051,35 @@ pub fn acquire_expected_instance<Device: BlockDevice>(
         InstanceAcquisitionFailed::Acquisition(acquisition) => {
             ExpectedInstanceAcquisitionFailed::Acquisition(acquisition)
         }
-    })
+    }
 }
 
-/// 取号的写那一半：逐盘核一遍每块盘两槽读得出自证过的系统配置，一道屏障，再读一次每块盘两槽、取见证值
-/// （[`self_verified_system_configurations_of_every_device`]、[`highest_journal_tail_of`]），逐盘写一次系统配置槽（实例代号写新号、
-/// tail 写见证值），两写之后一道屏障。两次读里有一次判不出（R2）或有一块盘一份自证过的都没有就一个字节不写、报它；写之前那道屏障报错就一个字节不写、报失败，
-/// 之后任一报错就把已写出的回卷成旧代号（tail 仍写见证值）。
+/// 取号写之前读到的（[`read_the_witness_before_the_acquisition_writes`] 造，[`write_the_witnessed_acquisition`] 照它写）：
+/// 要取的号、写之前盘上最大的实例代号（回卷写用）、屏障之后读到的见证值。字段私有：写那一半只收读那一半交出来的这一份。
+#[derive(Debug)]
+pub(crate) struct AcquisitionWitnessedBeforeItsWrites {
+    instance: InstanceGeneration,
+    previous_instance: InstanceGeneration,
+    witnessed_journal_tail: u64,
+}
+
+/// 取号的写那一半：照 [`read_the_witness_before_the_acquisition_writes`] 读、再照 [`write_the_witnessed_acquisition`] 写。
 fn write_acquired_instance<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     instance: InstanceGeneration,
 ) -> Result<InstanceGeneration, InstanceAcquisitionFailed> {
+    let witnessed = read_the_witness_before_the_acquisition_writes(pool, instance)?;
+    write_the_witnessed_acquisition(pool, witnessed)
+}
+
+/// 取号写之前那几次读：读上一个号，逐盘核一遍每块盘两槽读得出自证过的系统配置，一道屏障，再读一次每块盘两槽、取见证值
+/// （[`self_verified_system_configurations_of_every_device`]、[`highest_journal_tail_of`]）。两次读里有一次判不出（R2）或有一块盘
+/// 一份自证过的都没有就报它；那道屏障报错就报失败。都是一个字节没写。
+fn read_the_witness_before_the_acquisition_writes<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    instance: InstanceGeneration,
+) -> Result<AcquisitionWitnessedBeforeItsWrites, InstanceAcquisitionFailed> {
     let previous_instance = highest_system_configuration_instance(pool);
-    let mut written = Vec::new();
     // 读见证值时判不出（R2）或有一块盘一份自证过的都没有就拒（C554 乙-配置续 Q1 第 1 条）：这一核放在第一道屏障之前，拒的时候连屏障都没发，录制流一步不多。
     self_verified_system_configurations_of_every_device(pool)?;
     // 第一个取号写之前一道池屏障（代码审阅第 19 条）：取号写覆写的是这块盘上较旧的那一槽，另一槽可能是上一次发布末尾轮换刚写的
@@ -1051,6 +1099,24 @@ fn write_acquired_instance<Device: BlockDevice>(
     // mkfs 写的 tail 是 0，mkfs 之后第一次取号读到的就是 0：新池新建文件的字节不变。
     let witnessed_journal_tail =
         highest_journal_tail_of(&self_verified_system_configurations_of_every_device(pool)?);
+    Ok(AcquisitionWitnessedBeforeItsWrites {
+        instance,
+        previous_instance,
+        witnessed_journal_tail,
+    })
+}
+
+/// 取号写：逐盘写一次系统配置槽（实例代号写新号、tail 写见证值），两写之后一道屏障。任一报错就把已写出的回卷成旧代号（tail 仍写见证值）。
+fn write_the_witnessed_acquisition<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    witnessed: AcquisitionWitnessedBeforeItsWrites,
+) -> Result<InstanceGeneration, InstanceAcquisitionFailed> {
+    let AcquisitionWitnessedBeforeItsWrites {
+        instance,
+        previous_instance,
+        witnessed_journal_tail,
+    } = witnessed;
+    let mut written = Vec::new();
     for index in 0..pool.devices.len() {
         if let Err(cause) = pool.write_system_configuration_slot(
             index,
@@ -3684,30 +3750,24 @@ pub(crate) fn copies_of_a_released_unit_failing_the_release_checksum_check<
 ) -> Vec<CopyQuarantinedAfterReleaseChecksumMismatch> {
     let unit_bytes = usize::try_from(identity.span_slots() * SLOT_BYTES)
         .expect("一个单元两槽以内，字节数装得进 usize");
+    // 重读那一读绕过这次挂载记下的那一份、再读一次盘（`recovery::ReadPassOverThePool::RereadAfterABadRead`）。
+    let rereading_reader = PoolReaderOfOneReadPass::rereading_after_a_bad_read(reader);
     let reading_of_each_copy: Vec<(LocationEntry, QuarantinedCopyReading)> = locations
         .iter()
         .map(|location| {
-            let read_the_copy = || {
-                reader.read(
-                    location.device,
-                    location.slot.to_device_offset(),
-                    unit_bytes,
-                )
-            };
-            let check_the_copy_against_its_location_entry = |copy: Option<Vec<u8>>| match copy {
-                None => CopyCheck::Unreadable,
-                Some(copy) if crc32_castagnoli(&copy) != location.unit_checksum => {
-                    CopyCheck::ChecksumMismatch
-                }
-                Some(_) => CopyCheck::Intact,
-            };
             // 第一次读不出、或读出来核出对不上，都先重读一次（只一次，D19（块指针的结构与宽度预算） 已定项 5
             // 「硬规则 1 的读盘核读不出、核出对不上时怎么办」）：一次瞬时坏读不许让单元永久隔离。
             // 重读那一次对得上就算这一份对得上；两次都没对上，按重读那一次的样子报。
-            let reading = match check_the_copy_against_its_location_entry(read_the_copy()) {
+            let reading = match read_one_copy_and_check_it_against_its_location_entry(
+                reader, location, unit_bytes,
+            ) {
                 CopyCheck::Intact => QuarantinedCopyReading::IntactButAnotherCopyFailed,
                 CopyCheck::Unreadable | CopyCheck::ChecksumMismatch => {
-                    match check_the_copy_against_its_location_entry(read_the_copy()) {
+                    match read_one_copy_and_check_it_against_its_location_entry(
+                        &rereading_reader,
+                        location,
+                        unit_bytes,
+                    ) {
                         CopyCheck::Intact => QuarantinedCopyReading::IntactButAnotherCopyFailed,
                         CopyCheck::Unreadable => QuarantinedCopyReading::UnreadableAfterOneReread,
                         CopyCheck::ChecksumMismatch => QuarantinedCopyReading::ChecksumMismatch,
@@ -3744,6 +3804,26 @@ pub(crate) fn copies_of_a_released_unit_failing_the_release_checksum_check<
         .collect()
 }
 
+/// 按一条位置项读那一份整单元、算 CRC-32C 与位置项里的比（[`copies_of_a_released_unit_failing_the_release_checksum_check`] 读一次、重读一次各走它）：
+/// 对得上的那几字节报给读者（[`PoolReader::note_that_the_bytes_read_self_verify`]：重读那一遍的读者拿它换掉这次挂载记下的那一份）。只读盘。
+fn read_one_copy_and_check_it_against_its_location_entry<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    location: &LocationEntry,
+    unit_bytes: usize,
+) -> CopyCheck {
+    let offset = location.slot.to_device_offset();
+    match reader.read(location.device, offset, unit_bytes) {
+        None => CopyCheck::Unreadable,
+        Some(copy) if crc32_castagnoli(&copy) != location.unit_checksum => {
+            CopyCheck::ChecksumMismatch
+        }
+        Some(copy) => {
+            reader.note_that_the_bytes_read_self_verify(location.device, offset, &copy);
+            CopyCheck::Intact
+        }
+    }
+}
+
 /// 释放之前读盘核一份的一次结果（[`copies_failing_the_release_checksum_check`]）：读不出、读出来整单元 CRC-32C 与位置项里的对不上、对得上。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum CopyCheck {
@@ -4011,6 +4091,24 @@ pub enum PublishError {
     DeletedInodesQueueRefused(DeletedInodesQueueRefusal),
     /// 块设备报的错。落盘阶段报的（带文件的一版、树表 0 条的一版上写行）：这次发布已冻结在分配器上等原样重发。
     BlockDevice(BlockDeviceError),
+    /// 可写挂载取号之前预演续做（分配器拷贝上走删除队列那一趟，连同空间不够时只进内存的盘上走取号之后那一串与续做那一趟）报了错、
+    /// 没走到续做的结局，而这两趟读盘时有一次读穿到底下那块盘读错了：交最先的那一次读错，在取号之前拒
+    /// （`MountError::Publish`，三份写账都是空的，两块盘逐字节不变；`mount` 的
+    /// `refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`）。
+    /// 这次挂载读过的块预演与真做共用一份（`mounted_session::DeviceReadOnceInThisMount`），读错的那一块没记下：交给真做，真做会再读一次
+    /// 底下那块盘，看到的可能与预演不同（读错是瞬时的），预演判的就不是真做走的那一串（岔路单 `research/prompts/m3-b44-code-r2-forks.md`
+    /// 岔路一「读一次两边共用」那一行，用户定）。只在取号之前那一判里造，发布路径不报它。
+    ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(Box<ReadFailedOnTheDisk>),
+}
+
+/// 一次读穿到底下那块盘时读错了：哪块盘、从哪个偏移起读多长、盘报的错（可写挂载这次挂载共用的那一份读，
+/// `mounted_session::DeviceReadOnceInThisMount`，记它）。
+#[derive(Debug)]
+pub struct ReadFailedOnTheDisk {
+    pub device: DeviceIdentity,
+    pub offset: DeviceOffsetInBytes,
+    pub length_in_bytes: u64,
+    pub error: BlockDeviceError,
 }
 
 /// 接在 `version_to_build_on` 后面那一次发布的 checkpoint_txg：它加一（D16（发布语义） 已定项 6）。
diff --git a/crates/singlefs-core/src/unit.rs b/crates/singlefs-core/src/unit.rs
index 1b230d4b..e9b219e3 100644
--- a/crates/singlefs-core/src/unit.rs
+++ b/crates/singlefs-core/src/unit.rs
@@ -565,7 +565,7 @@ pub fn parse_data_unit(bytes: &[u8]) -> Result<DataUnitHeader, UnitError> {
     };
     let payload_start = header_end + reserved_bytes();
     if payload_start + usize::from(declared_length) > bytes.len() {
-        return Err(UnitError::Structure("声明长度越过单元末尾"));
+        return Err(UnitError::Structure(DECLARED_LENGTH_PAST_THE_UNIT_END));
     }
     Ok(DataUnitHeader {
         identity,
@@ -576,10 +576,22 @@ pub fn parse_data_unit(bytes: &[u8]) -> Result<DataUnitHeader, UnitError> {
     })
 }
 
+/// 码 1 数据单元的声明长度越过单元末尾时 [`parse_data_unit`] 与 [`data_unit_payload`] 报 [`UnitError::Structure`]，带这一句。
+pub const DECLARED_LENGTH_PAST_THE_UNIT_END: &str = "声明长度越过单元末尾";
+
 /// 载荷从 134 起、长声明长度；之后到单元末尾必须全 0（I-2.3），不是就报结构错。
+///
+/// # Errors
+/// 载荷末尾（134 + 声明长度）越过 `bytes` 末尾 ⇒ [`UnitError::Structure`]（[`DECLARED_LENGTH_PAST_THE_UNIT_END`]）：声明长度是盘上读来的，
+/// 两字节最大 65535、比一个单元的载荷容量大，这里自己判、不越界切片——今天的调用方都先经 [`parse_data_unit`] 核过这一格，
+/// 这一判让本函数不靠调用方的那一步也不 panic（坏的是盘上数据，按「能恢复的写进类型」报错，不断言）。
+/// 补齐字节非零 ⇒ [`UnitError::Structure`]。
 pub fn data_unit_payload(bytes: &[u8], declared_length: u16) -> Result<&[u8], UnitError> {
     let payload_start = usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105") + reserved_bytes();
     let payload_end = payload_start + usize::from(declared_length);
+    if payload_end > bytes.len() {
+        return Err(UnitError::Structure(DECLARED_LENGTH_PAST_THE_UNIT_END));
+    }
     if bytes[payload_end..].iter().any(|byte| *byte != 0) {
         return Err(UnitError::Structure("补齐字节非零"));
     }
@@ -807,4 +819,58 @@ mod tests {
             Err(UnitError::PayloadChecksumMismatch)
         );
     }
+
+    /// 外部审核 ⑥：声明长度越过单元末尾（134 + 40000 > 32768）时 `data_unit_payload` 自己报结构错，不在切片上越界 panic；
+    /// 不经 `parse_data_unit` 直接调它。
+    #[test]
+    fn data_unit_payload_with_a_declared_length_past_the_unit_end_is_a_structure_error() {
+        let unit = build_data_unit(
+            DataUnitIdentity {
+                tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_EXTENT),
+                object: 1,
+                object_birth: CheckpointTxg(3),
+                anchor_offset: 0,
+            },
+            CheckpointTxg(3),
+            &[9u8; 16],
+            WriteOrder {
+                instance: InstanceGeneration(1),
+                transaction: 1,
+            },
+            &[5u8; 100],
+        );
+        assert_eq!(
+            data_unit_payload(&unit, 40_000),
+            Err(UnitError::Structure(DECLARED_LENGTH_PAST_THE_UNIT_END)),
+            "声明长度 40000：载荷末尾越过 32768 字节的单元"
+        );
+    }
+
+    /// 声明长度恰好等于载荷容量（32634，载荷末尾就是单元末尾）：越界那一判不拒，补齐区是空的，交回整段载荷。
+    #[test]
+    fn data_unit_payload_with_a_declared_length_reaching_the_unit_end_exactly_is_the_whole_payload()
+    {
+        let capacity = data_unit_payload_capacity();
+        let unit = build_data_unit(
+            DataUnitIdentity {
+                tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_EXTENT),
+                object: 1,
+                object_birth: CheckpointTxg(3),
+                anchor_offset: 0,
+            },
+            CheckpointTxg(3),
+            &[9u8; 16],
+            WriteOrder {
+                instance: InstanceGeneration(1),
+                transaction: 1,
+            },
+            &vec![6u8; capacity],
+        );
+        assert_eq!(
+            data_unit_payload(&unit, u16::try_from(capacity).expect("32634 装得进两字节"))
+                .map(<[u8]>::len),
+            Ok(capacity),
+            "载荷末尾等于单元末尾"
+        );
+    }
 }
```
