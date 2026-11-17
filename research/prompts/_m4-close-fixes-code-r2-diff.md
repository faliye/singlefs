# m4-close-fixes-code-r2：第四节五条的产品代码改动（补丁 m4-close-fixes-writer/patch/crates.patch 里 crates/singlefs-core 的块原样抄录）

```diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 0192391..aacb52b 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -17,6 +17,7 @@ use singlefs_format::{
 };
 
 use std::collections::{BTreeMap, BTreeSet};
+use std::sync::{Arc, Mutex};
 
 use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
 use crate::admission::SpaceAdmission;
@@ -25,6 +26,7 @@ use crate::compaction_plan::CompactionSwitchesAndTheExcludedRegion;
 use crate::instance_table::InstanceTableRecords;
 use crate::journal::slot_after_the_journal_ring;
 use crate::logged_operations::CompactionRegion;
+use crate::pointer::LocationEntry;
 use crate::root_record::RootRecord;
 use crate::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
 use crate::transaction::FrozenPublish;
@@ -1862,6 +1864,16 @@ impl ReclaimFloorWhenTheRootWasRecorded {
     }
 }
 
+/// 这次可写挂载里建上一版与读—改—写读回之外的读读到只坏一份的账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）：
+/// 经会话读文件、推抬 F 与发 discard 之前沿链读实例表、释放之前读盘核、搬留存单元读源。
+#[derive(Debug, Default)]
+struct OtherUnitReadsWithOneCopyFailing {
+    /// 读到只坏一份的次数：一组位置条目等着下一次用户改动的这一程里只算一次。
+    reads: u64,
+    /// 读到只坏一份、还在等下一次用户改动处置的那几组位置条目（一个单元一组）。
+    locations_waiting_for_the_next_user_change: Vec<[LocationEntry; 2]>,
+}
+
 /// 池级分配器：每块盘按同一条规则各自取，各盘取得一致才给号（盘可以不等大，D2（RAID 条带策略） 已定项 2）。
 #[derive(Clone, Debug)]
 pub struct PoolAllocator {
@@ -1925,6 +1937,9 @@ pub struct PoolAllocator {
     /// 上一次记根时回收用的门槛与还钉着的槽里按（钉值, 槽）最小的那一槽（[`ReclaimFloorWhenTheRootWasRecorded`]，C773）。只住内存，`new` 与
     /// `rebuild_from_records` 起步是 `None`；随分配器整个换回发布之前（失败的发布）一起退回。
     reclaim_floor_when_the_latest_root_was_recorded: Option<ReclaimFloorWhenTheRootWasRecorded>,
+    /// 别的读读到只坏一份的账（[`OtherUnitReadsWithOneCopyFailing`]）。一次挂载一份，各份拷贝共用：发布被拒时分配器换回发布之前那一份、
+    /// 预演在拷贝上走，读是真发到盘上的，账不随换回退掉。`new` 与 `rebuild_from_records` 起步是空的。
+    other_unit_reads_with_one_copy_failing: Arc<Mutex<OtherUnitReadsWithOneCopyFailing>>,
 }
 
 impl PoolAllocator {
@@ -1949,9 +1964,107 @@ impl PoolAllocator {
             deletion_switches: DeletionSwitches::PRODUCT,
             compaction: CompactionSwitchesAndTheExcludedRegion::of_a_new_mount(),
             reclaim_floor_when_the_latest_root_was_recorded: None,
+            other_unit_reads_with_one_copy_failing: Arc::default(),
         }
     }
 
+    /// 别的读读到只坏一份的次数（[`OtherUnitReadsWithOneCopyFailing`]）。
+    ///
+    /// # Panics
+    /// 账的锁中毒：持锁的几处只做加法与表操作，不 panic。
+    #[must_use]
+    pub fn other_unit_reads_with_one_copy_failing(&self) -> u64 {
+        self.other_unit_reads_with_one_copy_failing
+            .lock()
+            .expect("账的锁没有中毒：持锁的几处只做加法与表操作")
+            .reads
+    }
+
+    /// 记下这一读读到只坏一份的那几组位置条目：已经在等下一次用户改动的不再加次数，别的加一次、接在后面。
+    ///
+    /// # Panics
+    /// 账的锁中毒（同 [`Self::other_unit_reads_with_one_copy_failing`]）；次数越过 u64：每一次都是一次真读盘，一次挂载里读不到 2⁶⁴ 次。
+    pub(crate) fn count_the_other_unit_reads_with_one_copy_failing(
+        &self,
+        locations: Vec<[LocationEntry; 2]>,
+    ) {
+        let mut account = self
+            .other_unit_reads_with_one_copy_failing
+            .lock()
+            .expect("账的锁没有中毒：持锁的几处只做加法与表操作");
+        // 迭代上界是这一读报的组数；跨轮携带的只有账本身。
+        for read_locations in locations {
+            if !account
+                .locations_waiting_for_the_next_user_change
+                .contains(&read_locations)
+            {
+                account.reads = account
+                    .reads
+                    .checked_add(1)
+                    .expect("一次挂载里只坏一份的读次数装得进 u64");
+                account
+                    .locations_waiting_for_the_next_user_change
+                    .push(read_locations);
+            }
+        }
+    }
+
+    /// 把几组位置条目记成等着下一次用户改动，不加次数：建上一版读到的那几组（次数另有一个数，
+    /// [`crate::compaction_plan::CompactionOfThisMount::unit_reads_with_one_copy_failing_while_mounting`]）。已经在等的不再记。
+    ///
+    /// # Panics
+    /// 账的锁中毒（同 [`Self::other_unit_reads_with_one_copy_failing`]）。
+    pub(crate) fn keep_the_locations_read_with_one_copy_failing_uncounted(
+        &self,
+        locations: &[[LocationEntry; 2]],
+    ) {
+        let mut account = self
+            .other_unit_reads_with_one_copy_failing
+            .lock()
+            .expect("账的锁没有中毒：持锁的几处只做加法与表操作");
+        // 迭代上界是交进来的组数；跨轮携带的只有账本身。
+        for read_locations in locations {
+            if !account
+                .locations_waiting_for_the_next_user_change
+                .contains(read_locations)
+            {
+                account
+                    .locations_waiting_for_the_next_user_change
+                    .push(*read_locations);
+            }
+        }
+    }
+
+    /// 等着下一次用户改动的那几组位置条目，按记下的先后。
+    ///
+    /// # Panics
+    /// 账的锁中毒（同 [`Self::other_unit_reads_with_one_copy_failing`]）。
+    #[must_use]
+    pub(crate) fn locations_read_with_one_copy_failing_waiting_for_the_next_user_change(
+        &self,
+    ) -> Vec<[LocationEntry; 2]> {
+        self.other_unit_reads_with_one_copy_failing
+            .lock()
+            .expect("账的锁没有中毒：持锁的几处只做加法与表操作")
+            .locations_waiting_for_the_next_user_change
+            .clone()
+    }
+
+    /// 那几组位置条目里去掉 `is_handled` 判真的那几组（另发的那一批试过一次、或挂载交回之前认成了映射 key）。次数不动。
+    ///
+    /// # Panics
+    /// 账的锁中毒（同 [`Self::other_unit_reads_with_one_copy_failing`]）。
+    pub(crate) fn forget_the_locations_read_with_one_copy_failing(
+        &self,
+        is_handled: impl Fn(&[LocationEntry; 2]) -> bool,
+    ) {
+        self.other_unit_reads_with_one_copy_failing
+            .lock()
+            .expect("账的锁没有中毒：持锁的几处只做加法与表操作")
+            .locations_waiting_for_the_next_user_change
+            .retain(|locations| !is_handled(locations));
+    }
+
     /// 装上整理的只供测试的开关（`compaction_plan::CompactionSwitches`）。产品路径一处都不调它。
     pub fn set_compaction_switches(
         &mut self,
diff --git a/crates/singlefs-core/src/compaction_plan.rs b/crates/singlefs-core/src/compaction_plan.rs
index b054f08..d57b2db 100644
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -511,7 +511,7 @@ pub enum RelocationPublishOrder {
 }
 
 /// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支能被测试强制进入）：会话为读回时只坏一份、又被活快照钉着的旧单元另发的那一批搬迁
-/// （`mounted_session::RelocationOfTheOldDataUnitsKeptForTheSnapshots`），在发之前当作被空间准入拒。产品路径恒 [`Self::NotForced`]。
+/// （`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`），在发之前当作被空间准入拒。产品路径恒 [`Self::NotForced`]。
 /// 被拒推抬 F 再试那一支在小盘之外的池上造不出「改动放得下、紧接着那一批放不下」，靠它进；
 /// 开关造的拒绝在这一批的写入口建起之前交回，一个写都没发。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -527,7 +527,7 @@ pub enum RelocationKeptForTheSnapshotsRefusedForSpace {
 
 /// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：会话另发的那一批搬迁（同 [`RelocationKeptForTheSnapshotsRefusedForSpace`]）在发之前
 /// 当作被拒、而不是空间不够（计划对不上那一版那一种：`transaction::RelocationBatchRefused::PlannedUnitNotInTheVersion`，点名计划里头一把 key）。
-/// 产品路径恒 [`Self::NotForced`]。被拒不是空间不够那一支（不推抬 F、不再试，`mounted_session::RelocationOfTheOldDataUnitsKeptForTheSnapshots::RefusedNotForSpace`）
+/// 产品路径恒 [`Self::NotForced`]。被拒不是空间不够那一支（不推抬 F、不再试，`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace`）
 /// 在产品路径上要计划在算出来与发之间失效、或发布路径报空间之外的错才进得去，靠它进；开关造的拒绝在这一批的写入口建起之前交回，一个写都没发。
 /// 被空间拒的那个开关同时开着时先照那一个拒（它在前面判）。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -539,7 +539,7 @@ pub enum RelocationKeptForTheSnapshotsRefusedNotForSpace {
 
 /// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：会话为另发的那一批搬迁被空间拒而推抬 F 时，那一串抬 F 当作自己报了错
 /// （`mount::MountError::InstanceTableUnreadable`：`mount::push_one_floor_raise_within_the_admission_budget` 在任何写之前交得出的那一种），一个写都不发。
-/// 产品路径恒 [`Self::NotForced`]。抬 F 自己报错那一支（`mounted_session::RelocationOfTheOldDataUnitsKeptForTheSnapshots::FloorRaiseFailedWhilePushingForSpace`）
+/// 产品路径恒 [`Self::NotForced`]。抬 F 自己报错那一支（`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace`）
 /// 在产品路径上要推抬 F 时读盘或落盘出错才进得去，靠它进；要先被空间拒才推抬 F，与 [`RelocationKeptForTheSnapshotsRefusedForSpace`] 合着开。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum FloorRaiseForTheRelocationKeptForTheSnapshotsFails {
@@ -921,7 +921,7 @@ impl CompactionOfThisMount {
         &self.mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting
     }
 
-    /// 记下可写挂载读盘建上一版时读到只坏一份的那几个单元的映射 key（挂载交回之前调一次，接在已记的后面、同一把只记一次）。
+    /// 记下读到只坏一份的那几个单元的映射 key（挂载交回之前调一次，接在已记的后面、同一把只记一次）。
     pub(crate) fn keep_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting(
         &mut self,
         mapping_keys: Vec<Vec<u8>>,
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 1146261..76817eb 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -89,15 +89,14 @@ use crate::system_configuration::SystemImmutableSizes;
 use crate::transaction::{
     central_mapping_locations_in_the_version,
     copies_of_a_released_unit_failing_the_release_checksum_check,
-    mapping_keys_of_the_units_at_these_locations, next_instance_generation_to_acquire,
-    placement_to_release_after_checking_every_device, prepare_the_publish_without_units,
-    prepare_the_row_publish_on_a_version_without_file, prepare_the_version_publish,
-    publish_instance_table_on_version_without_file, publish_version,
+    next_instance_generation_to_acquire, placement_to_release_after_checking_every_device,
+    prepare_the_publish_without_units, prepare_the_row_publish_on_a_version_without_file,
+    prepare_the_version_publish, publish_instance_table_on_version_without_file, publish_version,
     publish_version_appending_a_lineage_record, publish_version_with_unmount_marker,
     publish_without_units, read_the_witness_before_the_expected_acquisition_writes,
     refuse_locations_that_do_not_name_two_pool_devices,
     rehearse_every_step_of_the_deletion_queue_walk, role_of_allocation_record_tree_node,
-    walk_the_deletion_queue_until_it_is_empty,
+    units_at_these_locations, walk_the_deletion_queue_until_it_is_empty,
     write_the_raised_floor_into_every_system_configuration,
     write_the_witnessed_expected_acquisition, AcquisitionFailed, CodeTwoTreeNodeCapacities,
     ContainerTreeRootUnitPositions, CopyQuarantinedAfterReleaseChecksumMismatch,
@@ -5377,7 +5376,19 @@ fn raise_the_floor_through<Device: BlockDevice>(
     // 候选集按现行那一版的实例表判：从它的根指着的第 0 片沿链真读出来、解出来（C502（抬 F 时现行版本里没有实例表单元）；
     // D18（块里携带什么信息） 已定项 11：一张表可以不止一片）。不看 `TransactionOutput::units`——那是这个进程内存里的角色列表，
     // 第一个文件版本那一版起就不带实例表单元，mkfs 同一个进程里后面发多少次都一样，而表就在根指针后面、读得出也解得开。
-    let table = instance_table_chain_of_root(&*devices, &current.root)
+    // 经数只坏一份的那一层读（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）：下一次用户改动发成之后另发的那一批换下它。
+    // 下面拒了分配器换回去，账不退（读是真发到盘上的，账各份拷贝共用）。
+    let devices_counting_unit_reads_with_one_copy_failing =
+        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*devices);
+    let instance_table_chain_read = instance_table_chain_of_root(
+        &devices_counting_unit_reads_with_one_copy_failing,
+        &current.root,
+    );
+    allocator.count_the_other_unit_reads_with_one_copy_failing(
+        devices_counting_unit_reads_with_one_copy_failing
+            .locations_of_the_units_read_with_one_copy_failing(),
+    );
+    let table = instance_table_chain_read
         .map_err(|failure| instance_table_refusal(&failure))?
         .records;
     // F 的生效值只增不减（「抬 F 那一串」那一行）：新 F 低于盘上现算的生效值，往下「抬」条款没写，在任何写之前拒。
@@ -7705,6 +7716,13 @@ fn establish_instance<Device: BlockDevice>(
     mut allocator: PoolAllocator,
     start: InstanceStart,
 ) -> Result<Mounted, MountError> {
+    // 建上一版读到只坏一份的那几组位置条目先记上（不加次数，次数另有一个数）：这一串的发布换下它们时照释放之前读盘核隔离
+    // （`transaction::copies_of_the_exempt_units_read_with_one_copy_failing`），挂载交回之前按挂载之后那一版认。
+    allocator.keep_the_locations_read_with_one_copy_failing_uncounted(
+        start
+            .previous
+            .units_read_with_one_copy_failing_while_building_on_it(),
+    );
     // 这次挂载往下的读，取号之前的几趟预演与取号之后的真做共用一份（岔路单 `research/prompts/m3-b44-code-r2-forks.md` 岔路一
     // 「读一次两边共用」那一行，用户定）：每一块第一次读时才穿到底下那块盘、读到的字节记下，之后谁再读都取记下的那一份；
     // 这次挂载写成的块读写下的。读阶段、恢复与重建分配器（这之前）读的不经它：它们自己的重读要打到盘上。
@@ -7898,23 +7916,30 @@ fn establish_instance<Device: BlockDevice>(
     // 读到只坏一份的那几个单元要搬（用户 2026-11-05 定「可写挂载时搬，冷恢复只计数」）：挂载这一串里不另发——另发一批就是挂载的第七段写，
     // 挂载分段、二次崩溃的摆法、取号之前的续做预演都要跟着改（`crates/singlefs-verification/src/writable_mount_phases.rs` 那六段）；
     // 这里按挂载之后那一版把它们认成映射 key 记进挂载期状态，会话下一次用户改动发成之后照快照下另发那批同一条路径搬
-    // （`mounted_session::MountedSession::publish_user_change`）。挂载这一串里已经重写掉的（树表、记账树这类每次发布都换下的单元）认不出、不记。
+    // （`mounted_session::MountedSession::publish_user_change`）。认不出 key 的几组留在账上：豁免映射的三类里挂载这一串没换下的（映射树根之下的节点）
+    // 那一批换落点（`transaction::repair_of_the_exempt_units_at_these_locations`），挂载这一串已经重写掉的（树表、实例表片这类）到那一批认不出，照丢。
     compaction_continuation
         .state_of_this_mount
         .count_unit_reads_with_one_copy_failing_while_mounting(
             start.unit_reads_with_one_copy_failing_while_building_on_the_version,
         );
+    let locations_read_with_one_copy_failing =
+        allocator.locations_read_with_one_copy_failing_waiting_for_the_next_user_change();
     if let Some(version_after_the_mount) = current.version_of_trees() {
+        let units_read_with_one_copy_failing = units_at_these_locations(
+            version_after_the_mount,
+            &locations_read_with_one_copy_failing,
+        );
         compaction_continuation
             .state_of_this_mount
             .keep_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting(
-                mapping_keys_of_the_units_at_these_locations(
-                    version_after_the_mount,
-                    start
-                        .previous
-                        .units_read_with_one_copy_failing_while_building_on_it(),
-                ),
+                units_read_with_one_copy_failing.mapping_keys,
             );
+        allocator.forget_the_locations_read_with_one_copy_failing(|locations| {
+            !units_read_with_one_copy_failing
+                .locations_without_a_mapping_key
+                .contains(locations)
+        });
     }
     // 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条「谁跑、什么时候跑」，用户 2026-10-03 定）：写行、暖机、抬 F、删除续做之后，
     // 交回之前判一次。清扫自己不让挂载失败（落点被拒这一轮不删、挂载照样交回可写，Q4），走到哪儿记进交回值。
diff --git a/crates/singlefs-core/src/mounted_read.rs b/crates/singlefs-core/src/mounted_read.rs
index d63934e..6fa24b4 100644
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -51,10 +51,10 @@ use crate::recovery::{
     inode_tree_root_key_range_is_the_subtree_coverage,
     read_mapped_tree_node_via_hint_then_central_mapping, read_mapped_tree_root,
     read_unit_via_locations, replay_journal, scan_journal,
-    tree_table_entries_each_kind_at_most_once, FailureKindForTheFallBack, JournalScanReport,
-    MappedTreeNodeClass, NamedUnitVerification, PoolReader,
-    PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition, RecoveryFailure,
-    TreeTableEntriesRefusal,
+    tree_table_entries_each_kind_at_most_once, ContainerNumberOfAnInodeLeaf,
+    FailureKindForTheFallBack, JournalScanReport, MappedTreeNodeClass, NamedUnitVerification,
+    PoolReader, PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition, RecoveryFailure,
+    SeparatorKeyOfAnInodeLeaf, TreeTableEntriesRefusal,
 };
 use crate::root_record::RootRecord;
 use crate::transaction::MultiLevelCodeTwoTree;
@@ -368,6 +368,8 @@ pub struct MountedPoolForRead {
 }
 
 /// 打开挂载态：把根记录到树表条目到两棵用户可见的树这一段读回内存，之后按偏移读不再碰它们。
+/// 这是只读语义的那一条路：读者读到只坏一份只计数（[`ReadPathObservation::data_unit_reads_with_one_copy_failing`]、报给读者）、不搬，不看同一个池上开没开着可写挂载；
+/// 可写挂载期间用户读文件经 `mounted_session::MountedSession::read_the_file`（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）。
 ///
 /// 读的次序：中央映射树（自举豁免，只按父指针里的位置条目读，D19（块指针的结构与宽度预算） 已定项 8；多层时整棵读进来）→ 树表（同样豁免）
 /// → 树表条目 → extent 树根兼叶 → inode 树根 → 每一片 inode 叶容器。
@@ -570,9 +572,9 @@ pub fn open_pool_for_read(
             // 记录号不小于分隔 key 与容器号（I-9.4 第一句，分隔 key 那一半是 I-9.12 第二句）：与冷走读、可写挂载重建、快照冷读同一个判法，
             // 少了这一判，只读挂载会收下另三个读者都拒的一版。
             inode_record_is_not_below_the_separator_key_and_the_container(
-                record.inode,
-                separator_key,
-                identity_in_the_entry.container,
+                InodeNumber(record.inode),
+                SeparatorKeyOfAnInodeLeaf(separator_key),
+                ContainerNumberOfAnInodeLeaf(identity_in_the_entry.container),
             )?;
             records_of_the_container.push(record);
         }
@@ -994,7 +996,7 @@ impl OpenFileForRead<'_> {
             &mut observation,
         );
         // 读到一半报错（后面一个单元两份都坏、映射里没有、自描述对不上……）的那一次，报错之前已经发出的读、已经读到的只坏一份照样并进打开之后的累计：
-        // 与写入口「改动被拒时那几次读也算」同一种口径——读是真发到盘上的、坏的那一份是真读到的，计不计不看这一次调用成没成
+        // 读是真发到盘上的、坏的那一份是真读到的，计不计不看这一次调用成没成。这个累计不去重：一次调用里一个单元只读一遍，跨调用相加。
         // 这一次读自己的观测随报错丢掉，只留在累计里。
         self.mount.add_to_observation_since_open(observation);
         Ok(FileReadOutput {
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index dc61fb1..2f52709 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -74,7 +74,13 @@ use crate::mount::{
     OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn, ParametersAndDeviceTableOfTheMount,
     RaisedFloor, ShadowLedger,
 };
-use crate::recovery::{instance_table_chain_of_root, PoolReader, ReadPassOverThePool};
+use crate::mounted_read::{
+    open_pool_for_read, FileReadFailure, FileReadOutput, OpenFileFailure, OpenPoolForReadFailure,
+};
+use crate::recovery::{
+    instance_table_chain_of_root, PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing,
+    ReadPassOverThePool,
+};
 use crate::region_scan::{
     scan_region_on_the_gpu, scan_region_over_the_identity_index, AllocationInRegion,
     CentralMappingEntry, CentralMappingIdentityIndex, RegionScan, RegionScanRefusal,
@@ -91,15 +97,17 @@ use crate::transaction::{
     publish_data_changes_published_together, publish_new_inodes, publish_overwrite,
     publish_overwrite_of_an_inode, publish_partial_write, publish_sequential_write,
     publish_sequential_write_of_an_inode, publish_size_change, publish_snapshot_destruction_intent,
-    publish_unlink_intent, publish_version, relocations_of_old_data_units_kept_for_the_snapshots,
+    publish_the_relocations_and_the_repair_of_the_exempt_units, publish_unlink_intent,
+    publish_version, relocations_of_old_data_units_kept_for_the_snapshots,
     relocations_of_the_units_read_with_one_copy_failing_while_mounting,
+    repair_of_the_exempt_units_at_these_locations, units_at_these_locations,
     walk_the_deletion_queue_until_it_is_empty, CentralMappingOfAVersionReadLeafByLeaf,
     CompactionIntentStep, CompactionStepRefused, DataChangesPublishedTogether,
     DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile, MappingLookup,
     NewInodesOfOneChange, PartialWriteBytes, PlannedUnitsRewrittenAnyway, PoolVersion, PoolWriter,
     PublishError, ReadFailedOnTheDisk, ReadFailingSelfVerification, RegionCheckOfTheIntentDeletion,
-    ReleaseChecksumCheck, RelocationBatchRefused, TransactionOutput, TransactionUnit,
-    WholeWriteOfOneInode,
+    ReleaseChecksumCheck, RelocationBatchRefused, RepairOfTheExemptUnits, TransactionOutput,
+    TransactionUnit, WholeWriteOfOneInode,
 };
 use crate::unit::{data_unit_payload_capacity, unit_filesystem_identifier, UNIT_CLASS_DATA};
 use crate::unlink::DeletionSwitches;
@@ -191,37 +199,43 @@ pub enum UserChange<'content> {
 #[derive(Debug)]
 pub struct UserChangePublished {
     /// 这次准入里推的那几串抬 F 的空发布，按先后；一次都没推的是空的。另发的那一批搬迁被空间拒时推的不在这里，在
-    /// [`Self::relocation_of_the_old_data_units_kept_for_the_snapshots`] 里（[`FloorRaisesPushedForTheRelocation`]）。
+    /// [`Self::relocation_after_the_change_of_the_units_read_with_one_copy_failing`] 里（[`FloorRaisesPushedForTheRelocation`]）。
     pub floor_raises: Vec<RaisedFloor>,
     /// 每一串是因为哪一次被拒才推的（与 `floor_raises` 一一对应）：`SpaceAdmissionRefused` 或 `PlacementRefused`。
     pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
     /// 这次调用里推的抬 F 之外发成的每一版，按先后（[`VersionsPublishedInThisChange`]）。整理一批都不在这里：整理挪出了用户改动的调用线程
     /// （用户 2026-10-26 定），改动发成之后起跑判据为真只记成「该跑了」（`compaction_plan::CompactionStartJudgement`），
     /// 由后台入口 [`MountedSession::run_compaction_in_the_background`] 去跑。另发的那一批搬迁（不是整理，是修只坏一份的旧单元，
-    /// [`Self::relocation_of_the_old_data_units_kept_for_the_snapshots`]）发成时排在末尾。
+    /// [`Self::relocation_after_the_change_of_the_units_read_with_one_copy_failing`]）发成时排在末尾。
     pub versions_published: VersionsPublishedInThisChange,
     /// 这次调用的发布都返回之后发的 discard（判决 G5，[`discard_the_segments_reusable_as_a_whole`]）。
     pub discards_after_the_change: DiscardsAfterThePublishes,
-    /// 这次改动里读—改—写读回时只坏一份、改动之后仍被活快照钉着的旧数据单元，另发的那一批搬迁做成了什么
-    /// （[`RelocationOfTheOldDataUnitsKeptForTheSnapshots`]）。
-    pub relocation_of_the_old_data_units_kept_for_the_snapshots:
-        RelocationOfTheOldDataUnitsKeptForTheSnapshots,
+    /// 这次改动发成之后另发的那一批做成了什么（[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`]）：管这次改动里读—改—写读回时只坏一份、改动之后仍被活快照钉着的旧单元，
+    /// 可写挂载读盘建上一版时读到只坏一份的单元，与可写挂载期间别的读读到只坏一份的单元。
+    pub relocation_after_the_change_of_the_units_read_with_one_copy_failing:
+        RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing,
 }
 
-/// 一次用户改动发成之后，读—改—写读回时只坏一份、又被活快照钉着的旧数据单元另发一批搬迁（D4（校验和位置） 已定项 4「经已有的搬迁路径把这个单元
-/// 整个搬到新槽、两份新写，旧落点照释放前核隔离；走同一个事务层」；用户 2026-11-04 定有活快照时「另发一批搬迁修好」，另发那批被拒时
-/// 「推抬 F 再试一遍」）做成了什么。封闭集合。用户改动在每一种结局下都已发成。
+/// 一次用户改动发成之后，读到只坏一份的单元另发一批换落点（D4（校验和位置） 已定项 4「经已有的搬迁路径把这个单元整个搬到新槽、两份新写，旧落点照释放前核隔离；
+/// 走同一个事务层」；用户 2026-11-04 定有活快照时「另发一批搬迁修好」，另发那批被拒时「推抬 F 再试一遍」）做成了什么。封闭集合。用户改动在每一种结局下都已发成。
+/// 这一批管三处读到的：这次改动读—改—写读回的、改动之后仍被活快照钉着的旧单元；可写挂载读盘建上一版时读到的（挂载记下、挂载之后头一次改动发成时取走）；
+/// 可写挂载期间别的读读到的（[`MountedSession::read_the_file`]、推抬 F 与发 discard 之前沿链读实例表、释放之前读盘核、搬留存单元读源）。
+/// 豁免映射的三类（映射树节点、树表、实例表片）认不出映射 key，同一批按它们自己的重写路子换落点（`transaction::RepairOfTheExemptUnits`）。
+/// 那一批搬没搬成都只试这一次，结局报在这里，之后的用户改动不再接着搬。
 /// 没有活快照钉着时读回的单元在改动那次发布里就整个重写、旧落点照释放之前读盘核隔离，不另发。
 /// 那一批被空间拒（[`refusal_is_short_of_space`] 那两种）时与会话走空同一套（`publish_the_queue_until_it_is_empty_raising_the_floor_for_space`）：
 /// 推一串抬 F、回收之后再发同一批，预算按 `mount::push_one_floor_raise_within_the_admission_budget` 数，这一批算一次准入，推的数从 0 起。
 #[derive(Debug)]
-pub enum RelocationOfTheOldDataUnitsKeptForTheSnapshots {
-    /// 不另发：这次改动读回旧单元时没有只坏一份的，或只坏一份的那几个在改动里已经释放（没有活快照钉着）。
+pub enum RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing {
+    /// 不另发：没有记着读到只坏一份的单元，或记着的那几个在这次改动发成之前已经换了落点（读回的旧单元没有活快照钉着、挂载记下的被这次改动整个重写、
+    /// 树表与映射树根这类每次发布都换下的）。
     NotNeeded,
-    /// 另发了一批（`transaction::publish_relocation_batch`，接在改动发成的那一版后面，被空间拒过的接在推的最后一串后面）：搬了这几把映射 key 的单元，
-    /// 按计划的次序。那一版排在 `versions_published` 末尾，也是会话的 `current`。
+    /// 另发了一批（`transaction::publish_the_relocations_and_the_repair_of_the_exempt_units`，接在改动发成的那一版后面，被空间拒过的接在推的最后一串后面）：
+    /// 计划搬这几把映射 key 的单元，按计划的次序（计划里是这一批本来就重写的节点的，不另搬也在里面）；同一次发布换下这几个豁免映射的单元。
+    /// 那一版排在 `versions_published` 末尾，也是会话的 `current`。
     Relocated {
         mapping_keys: Vec<Vec<u8>>,
+        units_exempt_from_the_mapping_replaced: Vec<TransactionUnit>,
         floor_raises: FloorRaisesPushedForTheRelocation,
     },
     /// 被空间拒、推满仍放不下（`stop`：预算用完、F 已在上限、抬 F 那一串自己也推不出空间）：会话的 `current` 停在推的最后一串的最后一次空发布上
@@ -258,9 +272,9 @@ pub struct FloorRaisesPushedForTheRelocation {
 
 /// 一次用户改动里各次发布的写入口上读—改—写读回旧数据单元时只坏一份的那几次（`transaction::PoolWriter` 的两份账），按发布的先后并起来：
 /// 次数并进这次挂载的累计（[`MountedSession::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`]），映射 key 交给另发的那一批搬迁。
-/// 被拒的那几次发布读回的也算：读是真发到盘上的。同一个旧单元一次改动里只算一次：被空间拒、推一串抬 F 之后重发同一个改动，
-/// 每一次发布都重读同几个旧单元（同一把映射 key），读到的是同一份坏副本，算一次（判决 `research/prompts/m4-close-code-r3-main-verification.md`
-/// 第三节第 6 条「每次用户改动只计一次」；与可写挂载建上一版那一读「同一份坏副本一次挂载只计一次」同一种口径）。
+/// 被拒的那几次发布读回的也算：坏的那一份是真读到的。这个数按「一次用户改动」去重：被空间拒、推一串抬 F 之后重发同一个改动，每一次发布都重读同几个旧单元
+/// （同一把映射 key），算一次（判决 `research/prompts/m4-close-code-r3-main-verification.md` 第三节第 6 条「每次用户改动只计一次」）；跨改动相加，
+/// 同一份坏副本下一次改动再读回再算。去重的范围与挂载那个数（一次挂载、同一组位置条目）、只读挂载的 `read_at` 累计（不去重）各是各的，三个数分开报。
 struct OldDataUnitReadsBackWithOneCopyFailingInThisChange {
     reads: u64,
     mapping_keys: Vec<Vec<u8>>,
@@ -2063,7 +2077,7 @@ fn publish_the_queue_until_it_is_empty<Device: BlockDevice>(
     )
 }
 
-/// 用户改动发成之后另发的那一批搬迁（[`RelocationOfTheOldDataUnitsKeptForTheSnapshots`]）：`mapping_keys` 是这次改动里读回时只坏一份的旧单元，
+/// 用户改动发成之后另发的那一批（[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`]）：`mapping_keys` 是这次改动里读回时只坏一份的旧单元，
 /// 其中到 `current` 这一版仍被活快照钉着的（`transaction::relocations_of_old_data_units_kept_for_the_snapshots`）一个都没有就不发；有就在新开的
 /// 写入口上照整理的搬迁入口发一批（[`publish_the_relocation_batch_raising_the_floor_for_space`]，意图照抄：`transaction::publish_relocation_batch` 那一支），
 /// 同一个事务层、同一条发布路径：原样字节写到新槽两份、留存的映射条目换到新槽、源落点照释放之前读盘核（只坏一份 ⇒ 两份一起隔离）。
@@ -2074,17 +2088,21 @@ fn publish_the_queue_until_it_is_empty<Device: BlockDevice>(
 /// `mapping_keys_read_while_mounting` 是可写挂载读盘建上一版时读到只坏一份的那几个单元（`compaction_plan::CompactionOfThisMount` 记下的映射 key，
 /// 用户 2026-11-05 定「可写挂载时搬，冷恢复只计数」）：其中到 `current` 这一版仍在中央映射里或留存条目里的那几个
 /// （`transaction::relocations_of_the_units_read_with_one_copy_failing_while_mounting`）接在上面那几个后面、进同一批，同一条路径、同一套被拒推抬 F；
-/// 结局报在同一个成员里（[`RelocationOfTheOldDataUnitsKeptForTheSnapshots`]）。挂载那一串里不另发：另发就是挂载的第七段写，挂载分段与二次崩溃、
+/// 结局报在同一个成员里（[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`]）。挂载那一串里不另发：另发就是挂载的第七段写，挂载分段与二次崩溃、
 /// 取号之前的续做预演都要跟着改，下一次用户改动之后这一处已有搬迁路径与它的崩溃点（判决 `research/prompts/m4-close-code-r3-main-verification.md`
 /// 第三节第 7 条「挑走已有路径改动最小的那一处」）。
 ///
+/// 可写挂载期间别的读读到只坏一份的那几组位置条目（分配器上那份账）按 `current` 这一版认：认出映射 key 的走上面同一条路，
+/// 此刻是豁免映射的单元的同一次发布换下（`transaction::repair_of_the_exempt_units_at_these_locations`），别的（这次改动已经换下的）不管。
+/// 这几组与这一批要搬的源落点上记下的，发完（发成或被拒）都从账上去掉：搬没搬成都只试这一次。这一批自己推抬 F 时新读到的不去掉，归下一次改动。
+///
 /// # Panics
 /// `current` 不是有树的一版：调用方只在一次用户改动发成之后调，发成的都是有树的一版（`publish_user_change` 开头拒了没树的那一形）。
 #[allow(
     clippy::too_many_arguments,
     reason = "参数各是一样东西：参数、盘表、分配器、现行那一版、实例代号、影子账那一臂、会话这一侧的整理状态、读回只坏一份的 key、挂载时读到只坏一份的 key、这次调用发成的各版"
 )]
-fn relocate_the_old_data_units_kept_for_the_snapshots<Device: BlockDevice>(
+fn relocate_the_units_read_with_one_copy_failing_after_the_change<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
     allocator: &mut PoolAllocator,
@@ -2095,7 +2113,7 @@ fn relocate_the_old_data_units_kept_for_the_snapshots<Device: BlockDevice>(
     mapping_keys: &[Vec<u8>],
     mapping_keys_read_while_mounting: &[Vec<u8>],
     versions_published: &mut VersionsPublishedInThisChange,
-) -> RelocationOfTheOldDataUnitsKeptForTheSnapshots {
+) -> RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing {
     let version_after_the_change = current
         .version_of_trees()
         .expect("一次用户改动发成之后现行那一版是有树的一版");
@@ -2103,15 +2121,51 @@ fn relocate_the_old_data_units_kept_for_the_snapshots<Device: BlockDevice>(
         version_after_the_change,
         mapping_keys,
     );
+    let locations_read_with_one_copy_failing =
+        allocator.locations_read_with_one_copy_failing_waiting_for_the_next_user_change();
+    let units_read_with_one_copy_failing = units_at_these_locations(
+        version_after_the_change,
+        &locations_read_with_one_copy_failing,
+    );
+    let mapping_keys_read_while_mounting_and_by_the_other_reads: Vec<Vec<u8>> =
+        mapping_keys_read_while_mounting
+            .iter()
+            .chain(&units_read_with_one_copy_failing.mapping_keys)
+            .cloned()
+            .collect();
     let planned_for_the_units_read_while_mounting =
         relocations_of_the_units_read_with_one_copy_failing_while_mounting(
             version_after_the_change,
-            mapping_keys_read_while_mounting,
+            &mapping_keys_read_while_mounting_and_by_the_other_reads,
             &planned,
         );
     planned.extend(planned_for_the_units_read_while_mounting);
-    if planned.is_empty() {
-        return RelocationOfTheOldDataUnitsKeptForTheSnapshots::NotNeeded;
+    let repair = match repair_of_the_exempt_units_at_these_locations(
+        &*devices,
+        version_after_the_change,
+        &units_read_with_one_copy_failing.locations_without_a_mapping_key,
+    ) {
+        Ok(repair) => repair,
+        Err(failure) => {
+            allocator.forget_the_locations_read_with_one_copy_failing(|locations| {
+                locations_read_with_one_copy_failing.contains(locations)
+            });
+            return RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace {
+                refusal: Box::new(RelocationBatchRefused::InstanceTableOfTheVersionUnreadable(
+                    failure,
+                )),
+                floor_raises: FloorRaisesPushedForTheRelocation {
+                    floor_raises: Vec::new(),
+                    refusals_that_pushed_the_floor_raises: Vec::new(),
+                },
+            };
+        }
+    };
+    if planned.is_empty() && repair.is_empty() {
+        allocator.forget_the_locations_read_with_one_copy_failing(|locations| {
+            locations_read_with_one_copy_failing.contains(locations)
+        });
+        return RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded;
     }
     let mut version_to_relocate_on = version_after_the_change.clone();
     let batch = publish_the_relocation_batch_raising_the_floor_for_space(
@@ -2123,7 +2177,16 @@ fn relocate_the_old_data_units_kept_for_the_snapshots<Device: BlockDevice>(
         shadow_ledger,
         compaction,
         &planned,
+        &repair,
     );
+    allocator.forget_the_locations_read_with_one_copy_failing(|locations| {
+        locations_read_with_one_copy_failing.contains(locations)
+            || locations.iter().any(|location| {
+                planned
+                    .iter()
+                    .any(|relocation| relocation.source.slot == location.slot)
+            })
+    });
     // 推成的抬 F 已落盘（被拒的结局也是）：现行那一版至少是推到的那一版。
     *current = PoolVersion::of_a_version_of_trees(version_to_relocate_on);
     match batch {
@@ -2134,11 +2197,12 @@ fn relocate_the_old_data_units_kept_for_the_snapshots<Device: BlockDevice>(
             let relocated = Arc::new(*relocated);
             versions_published.push(Arc::clone(&relocated));
             *current = PoolVersion::of_a_shared_version_of_trees(relocated);
-            RelocationOfTheOldDataUnitsKeptForTheSnapshots::Relocated {
+            RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated {
                 mapping_keys: planned
                     .into_iter()
                     .map(|relocation| relocation.mapping_key)
                     .collect(),
+                units_exempt_from_the_mapping_replaced: repair.units,
                 floor_raises,
             }
         }
@@ -2147,17 +2211,17 @@ fn relocate_the_old_data_units_kept_for_the_snapshots<Device: BlockDevice>(
 }
 
 /// 另发那一批走完推抬 F 再试之后：发成（发出来的那一版与推的几串），或会话要报的那一种没发成
-/// （[`RelocationOfTheOldDataUnitsKeptForTheSnapshots`] 的三种被拒）。
+/// （[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`] 的三种被拒）。
 enum RelocationBatchAfterTheFloorRaises {
     /// 装箱：一版的输出远大过另一个成员（clippy `large_enum_variant`）。
     Published {
         relocated: Box<TransactionOutput>,
         floor_raises: FloorRaisesPushedForTheRelocation,
     },
-    NotPublished(RelocationOfTheOldDataUnitsKeptForTheSnapshots),
+    NotPublished(RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing),
 }
 
-/// 在新开的写入口上照 `planned` 发一批搬迁（[`publish_the_compaction_step_once`]，意图照抄），接在 `version` 后面；被空间拒
+/// 在新开的写入口上照 `planned` 与 `repair` 发一批（[`publish_the_relocation_batch_once`]，意图照抄），接在 `version` 后面；被空间拒
 /// （[`compaction_publish_refused_short_of_space`]：发布路径的空间准入不够、或准入放行而落点取不到）就推一串抬 F
 /// （`mount::push_one_floor_raise_within_the_admission_budget`，就地推进 `version`）、回收之后再发同一批——与会话走空被空间拒同一套
 /// （[`publish_the_queue_until_it_is_empty_raising_the_floor_for_space`]）：这一批算一次准入，推的数从 0 起，预算照那个函数数（推成的每一串的空发布数相加）。
@@ -2170,7 +2234,7 @@ enum RelocationBatchAfterTheFloorRaises {
 /// （[`floor_raise_failure_of_the_relocation_forced_for_tests`]），照抬 F 自己报错那一支往下走。
 #[allow(
     clippy::too_many_arguments,
-    reason = "参数各是一样东西：参数、盘表、分配器、接在后面的那一版、实例代号、影子账那一臂、会话这一侧的整理状态、这一批的计划"
+    reason = "参数各是一样东西：参数、盘表、分配器、接在后面的那一版、实例代号、影子账那一臂、会话这一侧的整理状态、这一批的计划、这一批换下的豁免映射的单元"
 )]
 fn publish_the_relocation_batch_raising_the_floor_for_space<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
@@ -2181,6 +2245,7 @@ fn publish_the_relocation_batch_raising_the_floor_for_space<Device: BlockDevice>
     shadow_ledger: ShadowLedger,
     compaction: &mut CompactionOfThisMount,
     planned: &[PlannedRelocation],
+    repair: &RepairOfTheExemptUnits,
 ) -> RelocationBatchAfterTheFloorRaises {
     let mut floor_raises = FloorRaisesPushedForTheRelocation {
         floor_raises: Vec::new(),
@@ -2198,13 +2263,13 @@ fn publish_the_relocation_batch_raising_the_floor_for_space<Device: BlockDevice>
         {
             refusal_not_for_space_of_the_relocation_forced_for_tests(planned)
         } else {
-            match publish_the_compaction_step_once(
+            match publish_the_relocation_batch_once(
                 parameters,
                 devices.as_mut_slice(),
                 allocator,
                 version,
                 planned,
-                CompactionIntentStep::CarryTheIntent,
+                repair,
                 instance,
             ) {
                 Ok(relocated) => {
@@ -2218,7 +2283,7 @@ fn publish_the_relocation_batch_raising_the_floor_for_space<Device: BlockDevice>
         };
         if !compaction_publish_refused_short_of_space(&refusal_of_the_relocation) {
             return RelocationBatchAfterTheFloorRaises::NotPublished(
-                RelocationOfTheOldDataUnitsKeptForTheSnapshots::RefusedNotForSpace {
+                RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace {
                     refusal: Box::new(refusal_of_the_relocation),
                     floor_raises,
                 },
@@ -2248,7 +2313,7 @@ fn publish_the_relocation_batch_raising_the_floor_for_space<Device: BlockDevice>
             }
             Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => {
                 return RelocationBatchAfterTheFloorRaises::NotPublished(
-                    RelocationOfTheOldDataUnitsKeptForTheSnapshots::NoSpaceAfterRaisingTheFloor {
+                    RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NoSpaceAfterRaisingTheFloor {
                         last_refusal: Box::new(refusal_of_the_relocation),
                         stop,
                         floor_raises,
@@ -2257,7 +2322,7 @@ fn publish_the_relocation_batch_raising_the_floor_for_space<Device: BlockDevice>
             }
             Err(cause) => {
                 return RelocationBatchAfterTheFloorRaises::NotPublished(
-                    RelocationOfTheOldDataUnitsKeptForTheSnapshots::FloorRaiseFailedWhilePushingForSpace {
+                    RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace {
                         cause: Box::new(cause),
                         refusal_that_pushed_the_failed_raise: Box::new(refusal_of_the_relocation),
                         floor_raises,
@@ -2268,6 +2333,32 @@ fn publish_the_relocation_batch_raising_the_floor_for_space<Device: BlockDevice>
     }
 }
 
+/// 在新开的写入口上发一次另发的那一批（写入口的容器容量取分配器上装的开关，同 [`publish_the_compaction_step_once`]）。
+///
+/// # Errors
+/// 同 `transaction::publish_the_relocations_and_the_repair_of_the_exempt_units`。
+fn publish_the_relocation_batch_once<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut [(DeviceIdentity, Device)],
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    planned: &[PlannedRelocation],
+    repair: &RepairOfTheExemptUnits,
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, RelocationBatchRefused> {
+    let switches = allocator.deletion_switches();
+    let mut writer = PoolWriter::new(parameters, devices);
+    writer.set_container_tree_capacities(switches.container_tree_capacities());
+    publish_the_relocations_and_the_repair_of_the_exempt_units(
+        &mut writer,
+        allocator,
+        previous,
+        planned,
+        repair,
+        instance,
+    )
+}
+
 /// 只供测试的开关造的那一拒（`compaction_plan::RelocationKeptForTheSnapshotsRefusedForSpace`）：空间准入不够那一种，一块不够的盘都不点名——
 /// 没有真算准入，只为把另发那一批强制进「被空间拒」那一支；在这一批的写入口建起之前交回，一个写都没发。
 fn space_refusal_of_the_relocation_forced_for_tests() -> RelocationBatchRefused {
@@ -2280,18 +2371,15 @@ fn space_refusal_of_the_relocation_forced_for_tests() -> RelocationBatchRefused
 
 /// 只供测试的开关造的那一拒（`compaction_plan::RelocationKeptForTheSnapshotsRefusedNotForSpace`）：计划对不上那一版那一种，点名计划里头一把 key——
 /// 没有真核计划，只为把另发那一批强制进「被拒、不是空间不够」那一支；在这一批的写入口建起之前交回，一个写都没发。
-///
-/// # Panics
-/// `planned` 是空的：调用方（[`relocate_the_old_data_units_kept_for_the_snapshots`]）计划为空就交回 `NotNeeded`、不发这一批。
+/// 这一批只换豁免映射的单元、计划是空的时点名的 key 是空的。
 fn refusal_not_for_space_of_the_relocation_forced_for_tests(
     planned: &[PlannedRelocation],
 ) -> RelocationBatchRefused {
     RelocationBatchRefused::PlannedUnitNotInTheVersion {
         mapping_key: planned
             .first()
-            .expect("另发那一批的计划不是空的：计划为空时调用方不发")
-            .mapping_key
-            .clone(),
+            .map(|relocation| relocation.mapping_key.clone())
+            .unwrap_or_default(),
     }
 }
 
@@ -2301,6 +2389,105 @@ fn floor_raise_failure_of_the_relocation_forced_for_tests() -> MountError {
     MountError::InstanceTableUnreadable
 }
 
+/// 经会话读文件没读成（[`MountedSession::read_the_file`]）。盘表那四样在读文件之前，盘上逐字节不变（读本身从不写盘）；后三样是读者报的。
+#[derive(Debug)]
+pub enum FileReadInTheSessionRefused {
+    /// 同 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`]。
+    DeviceTableOtherThanTheOneOfTheMount {
+        device_table_of_the_mount: Vec<DeviceIdentity>,
+        device_table_handed_in: Vec<DeviceIdentity>,
+    },
+    /// 同 [`UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration`]。
+    DevicesWithoutASelfVerifiedSystemConfiguration { devices: Vec<DeviceIdentity> },
+    /// 同 [`UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable`]。
+    OwnDeviceNumbersDifferFromTheDeviceTable {
+        disagreements: Vec<OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn>,
+    },
+    /// 同 [`UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`]。
+    DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
+        devices: Vec<DeviceBehindTheCurrentVersion>,
+    },
+    /// 现行那一版树表 0 条：还没有文件版本。
+    NoFileVersionToRead,
+    /// 打开挂载态没做成。装箱：读者的错远大过别的成员（clippy `result_large_err`）。
+    OpenPool(Box<OpenPoolForReadFailure>),
+    /// 打开文件没做成。装箱同上。
+    OpenFile(Box<OpenFileFailure>),
+    /// 读字节没做成。
+    Read(FileReadFailure),
+}
+
+/// [`MountedSession::read_the_file`] 读文件之前核盘表（同 [`MountedSession::publish_user_change`] 进循环之前那几样）：只读每块盘的系统配置槽与落后那几块上现行那一版的落点。
+///
+/// # Errors
+/// [`FileReadInTheSessionRefused`] 的盘表四样与 `NoFileVersionToRead`。
+fn refuse_a_read_on_a_device_table_other_than_the_one_of_the_mount<Device: BlockDevice>(
+    parameters_and_device_table: &ParametersAndDeviceTableOfTheMount,
+    current: &PoolVersion,
+    devices: &[(DeviceIdentity, Device)],
+) -> Result<(), FileReadInTheSessionRefused> {
+    if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {
+        return Err(
+            FileReadInTheSessionRefused::DeviceTableOtherThanTheOneOfTheMount {
+                device_table_of_the_mount: parameters_and_device_table
+                    .device_identities_in_table_order()
+                    .to_vec(),
+                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
+            },
+        );
+    }
+    let parameters = parameters_and_device_table.parameters_on_disk();
+    let Some(current_version_of_trees) = current.version_of_trees() else {
+        return Err(FileReadInTheSessionRefused::NoFileVersionToRead);
+    };
+    let devices_without_a_system_configuration =
+        devices_without_a_self_verified_system_configuration(devices, parameters);
+    if !devices_without_a_system_configuration.is_empty() {
+        return Err(
+            FileReadInTheSessionRefused::DevicesWithoutASelfVerifiedSystemConfiguration {
+                devices: devices_without_a_system_configuration,
+            },
+        );
+    }
+    let disagreements =
+        own_device_numbers_on_disk_differing_from_the_identities_handed_in(devices, parameters);
+    if !disagreements.is_empty() {
+        return Err(
+            FileReadInTheSessionRefused::OwnDeviceNumbersDifferFromTheDeviceTable { disagreements },
+        );
+    }
+    let devices_of_the_table_behind =
+        devices_behind_the_current_version(devices, parameters, current_version_of_trees);
+    if !devices_of_the_table_behind.is_empty() {
+        return Err(
+            FileReadInTheSessionRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
+                devices: devices_of_the_table_behind,
+            },
+        );
+    }
+    Ok(())
+}
+
+/// 沿 `root` 打开挂载态、打开 `inode` 那个文件、读那一段（[`MountedSession::read_the_file`] 经数只坏一份的那一层交 `reader`）。
+///
+/// # Errors
+/// [`FileReadInTheSessionRefused`] 的读者那三种。
+fn read_the_file_of_the_root(
+    reader: &dyn PoolReader,
+    root: &RootRecord,
+    inode: InodeNumber,
+    offset: FileOffsetInBytes,
+    length_in_bytes: u64,
+) -> Result<FileReadOutput, FileReadInTheSessionRefused> {
+    let mounted = open_pool_for_read(reader, root)
+        .map_err(|failure| FileReadInTheSessionRefused::OpenPool(Box::new(failure)))?;
+    let file = mounted
+        .open_file(reader, inode)
+        .map_err(|failure| FileReadInTheSessionRefused::OpenFile(Box::new(failure)))?;
+    file.read_at(reader, offset, length_in_bytes)
+        .map_err(FileReadInTheSessionRefused::Read)
+}
+
 /// 在一个新开的写入口上发一次这个改动，接在 `previous` 后面。unlink 这里只发意图那一次，全部批由调用方接着走空。
 /// 发完（发成或被拒）把这个写入口上读—改—写读回时只坏一份的那两份账并进 `old_data_unit_reads_back`：会话里只有这一处的发布读回旧数据单元
 /// （部分写、截断与变长，`transaction::publish_partial_write`、`transaction::publish_size_change`），别的几处写入口上这两份账恒空。
@@ -2475,8 +2662,55 @@ impl MountedSession {
             .unit_reads_with_one_copy_failing_while_mounting()
     }
 
+    /// 这次可写挂载里别的读读到只坏一份的次数（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬，包括……可写挂载期间别的读」）：[`Self::read_the_file`]、
+    /// 推抬 F 与发 discard 之前沿链读实例表、释放之前读盘核、搬留存单元读源。一组位置条目等着下一次用户改动的这一程里只算一次，处置过之后再读到再算。
+    /// 与 [`Self::unit_reads_with_one_copy_failing_while_mounting`]、[`Self::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`] 按读的地方分开报。
+    /// 只在内存里，不落盘；新挂载从 0 起。
+    #[must_use]
+    pub fn other_unit_reads_with_one_copy_failing_in_this_mount(&self) -> u64 {
+        self.allocator.other_unit_reads_with_one_copy_failing()
+    }
+
+    /// 可写挂载期间的用户读文件：读现行那一版（`current`）里 `inode` 那个文件 `[offset, offset + length_in_bytes)` 的字节（读者同只读挂载，
+    /// `mounted_read::open_pool_for_read`、`open_file`、`read_at`，每次调用从根读起），经数只坏一份的那一层读。读到只坏一份的单元进
+    /// [`Self::other_unit_reads_with_one_copy_failing_in_this_mount`]，下一次用户改动发成之后另发的那一批把它换到新槽、两份新写
+    /// （[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`]，D4（校验和位置） 已定项 4）。读本身不写盘。报错的那一次报错之前读到的照样进账。
+    /// 盘表照 [`Self::publish_user_change`] 核三样（D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」），核不过不读文件。
+    /// `open_pool_for_read` 那条不挂在会话上的路是只读语义，只计数、不搬。
+    ///
+    /// # Errors
+    /// [`FileReadInTheSessionRefused`]。
+    pub fn read_the_file<Device: BlockDevice>(
+        &self,
+        devices: &[(DeviceIdentity, Device)],
+        inode: InodeNumber,
+        offset: FileOffsetInBytes,
+        length_in_bytes: u64,
+    ) -> Result<FileReadOutput, FileReadInTheSessionRefused> {
+        refuse_a_read_on_a_device_table_other_than_the_one_of_the_mount(
+            &self.parameters_and_device_table,
+            &self.current,
+            devices,
+        )?;
+        let devices_counting_unit_reads_with_one_copy_failing =
+            PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(devices);
+        let read = read_the_file_of_the_root(
+            &devices_counting_unit_reads_with_one_copy_failing,
+            self.current.root(),
+            inode,
+            offset,
+            length_in_bytes,
+        );
+        self.allocator
+            .count_the_other_unit_reads_with_one_copy_failing(
+                devices_counting_unit_reads_with_one_copy_failing
+                    .locations_of_the_units_read_with_one_copy_failing(),
+            );
+        read
+    }
+
     /// 这次可写挂载读盘建上一版时读到只坏一份、还没交给另发那一批的那几个单元的映射 key（挂载交回之前按挂载之后那一版认的，
-    /// `transaction::mapping_keys_of_the_units_at_these_locations`）：下一次用户改动发成之后另发的那一批取走它们、照快照下另发那批同一条路径搬
+    /// `transaction::units_at_these_locations`）：下一次用户改动发成之后另发的那一批取走它们、照快照下另发那批同一条路径搬
     /// （[`Self::publish_user_change`]，用户 2026-11-05 定「可写挂载时搬」），取走之后是空的。
     #[must_use]
     pub fn mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting_not_yet_relocated(
@@ -2508,7 +2742,7 @@ impl MountedSession {
     /// 发成之后查一次整理的起跑判据，为真只记成「该跑了」（`compaction_plan::CompactionStartJudgement::DueAfterAUserChange`），
     /// 返回之前一批整理都不发（整理挪出了调用线程，用户 2026-10-26 定；后台入口 [`Self::run_compaction_in_the_background`]）。
     /// 这次改动读—改—写读回旧数据单元时只坏一份的次数并进这次挂载的累计（[`Self::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`]），
-    /// 发成之后其中仍被活快照钉着的那几个另发一批搬迁（[`RelocationOfTheOldDataUnitsKeptForTheSnapshots`]），都在 discard 之前。
+    /// 发成之后其中仍被活快照钉着的那几个、挂载记下的、别的读记下的另发一批换落点（[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`]），都在 discard 之前。
     ///
     /// # Errors
     /// [`UserChangeRefused`] 的八种。
@@ -2601,8 +2835,8 @@ impl MountedSession {
         // 挂载之后的头一次用户改动发成时取走，搬没搬成都只试这一次；这次改动已经把它整个重写了的不进计划。
         let mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting = compaction
             .take_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting();
-        published.relocation_of_the_old_data_units_kept_for_the_snapshots =
-            relocate_the_old_data_units_kept_for_the_snapshots(
+        published.relocation_after_the_change_of_the_units_read_with_one_copy_failing =
+            relocate_the_units_read_with_one_copy_failing_after_the_change(
                 parameters,
                 devices,
                 allocator,
@@ -3152,8 +3386,8 @@ impl MountedSession {
                         versions_published,
                         discards_after_the_change: DiscardsAfterThePublishes::default(),
                         // 另发的那一批由 `publish_user_change` 在这一段返回之后发（预演不发它），发了再换。
-                        relocation_of_the_old_data_units_kept_for_the_snapshots:
-                            RelocationOfTheOldDataUnitsKeptForTheSnapshots::NotNeeded,
+                        relocation_after_the_change_of_the_units_read_with_one_copy_failing:
+                            RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded,
                     });
                 }
                 Err(refusal) => refusal,
@@ -3472,17 +3706,27 @@ pub fn discard_the_segments_reusable_as_a_whole<Device: BlockDevice>(
     {
         return discards;
     }
+    // 经数只坏一份的那一层读（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）：下一次用户改动发成之后另发的那一批换下它。
+    let devices_counting_unit_reads_with_one_copy_failing =
+        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*devices);
     let rows_of_the_instance_table: Option<BTreeMap<InstanceGeneration, InstanceRow>> =
-        instance_table_chain_of_root(&*devices, current_root)
-            .ok()
-            .map(|chain| {
-                chain
-                    .records
-                    .rows
-                    .iter()
-                    .map(|row| (row.instance, *row))
-                    .collect()
-            });
+        instance_table_chain_of_root(
+            &devices_counting_unit_reads_with_one_copy_failing,
+            current_root,
+        )
+        .ok()
+        .map(|chain| {
+            chain
+                .records
+                .rows
+                .iter()
+                .map(|row| (row.instance, *row))
+                .collect()
+        });
+    allocator.count_the_other_unit_reads_with_one_copy_failing(
+        devices_counting_unit_reads_with_one_copy_failing
+            .locations_of_the_units_read_with_one_copy_failing(),
+    );
     let filesystem_identifier = unit_filesystem_identifier(&parameters.filesystem_identifier);
     for (device_identity, segment_starts) in segments_to_discard {
         for segment_start in segment_starts {
@@ -4138,6 +4382,7 @@ fn compaction_publish_refused_short_of_space(refused: &RelocationBatchRefused) -
         RelocationBatchRefused::PlannedUnitNotInTheVersion { .. }
         | RelocationBatchRefused::PinnedUnitInThePlan { .. }
         | RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition { .. }
+        | RelocationBatchRefused::InstanceTableOfTheVersionUnreadable(_)
         | RelocationBatchRefused::UnitBytesNotAvailable { .. }
         | RelocationBatchRefused::CompactionIntentAlreadyLive { .. }
         | RelocationBatchRefused::NoLiveCompactionIntent
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index eec63d9..78e95e2 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -188,7 +188,7 @@ pub trait PoolReader {
     /// 按一组位置条目读一个单元时只坏一份（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：同一组里前一条读不回字节或整单元校验和
     /// 对不上，后一条对上、交回了它那一份。`locations` 是那一组两条位置条目（哪个单元由它们认）。默认什么都不做；要取数的读者
     /// 套 [`PoolReaderCountingUnitReadsWithOneCopyFailing`]。两条位置提示都没对上、转去查中央映射的那一回不算（提示过期与两份都坏分不开），
-    /// 映射那一组里只坏一份的照算；按提示读到只坏一份而提示不是这个单元此刻在映射里的落点也不报
+    /// 映射那一组里只坏一份的照算；按提示读到只坏一份而映射里查到的落点不是提示也不报
     /// （`PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition`）。
     fn note_that_a_unit_was_read_with_one_copy_failing(&self, _locations: &[LocationEntry; 2]) {}
 }
@@ -318,7 +318,8 @@ impl<Reader: PoolReader + ?Sized> PoolReader
 /// 只在读的那一组位置条目就是这个单元此刻的权威落点——中央映射里它那两条位置条目（D19（块指针的结构与宽度预算） 已定项 8）——时往里转；
 /// 别的读与报告原样交给里面那一份。提示是盘上 extent 叶、父条目里记的，单元被整理搬走之后提示还指旧槽，旧槽上一份被盖掉、另一份还是旧字节，
 /// 按提示读就是「只坏一份」，而映射里的权威落点两份都好：旧槽上坏的那一份不归这个单元计（判决 `research/prompts/m4-close-code-r3-main-verification.md`
-/// 第二节 T2 ②、第三节第 4 条）。映射里查不出（没有这把 key、查映射本身判红）同样不转：这一读已经读成，判红留给走到映射那一步的读者照报。
+/// 第二节 T2 ②、第三节第 4 条）。只有查到了、落点不是这一组才判得出提示过期；映射里查不出（没有这把 key、查映射本身判红）判不出，这一份是真读到的坏副本，
+/// 照转（D4（校验和位置） 已定项 4「读到只坏一份时计数」，只有造坏的镜像走得到，判红留给走到映射那一步的读者照报）。
 /// 经映射回退的那一读读的就是映射里那两条，照转。没转的那一报记下（[`Self::dropped_a_one_copy_failing_note_off_the_mapping_position`]）。
 pub(crate) struct PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition<'reader> {
     reader: &'reader dyn PoolReader,
@@ -411,10 +412,12 @@ impl PoolReader for PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition<'_> {
     }
     fn note_that_a_unit_was_read_with_one_copy_failing(&self, locations: &[LocationEntry; 2]) {
         match (self.central_mapping_locations_of_key)(self.mapping_key) {
-            Ok(Some(mapped_locations)) if mapped_locations == *locations => self
+            Ok(Some(mapped_locations)) if mapped_locations != *locations => {
+                self.one_copy_failing_note_dropped.set(true);
+            }
+            Ok(Some(_) | None) | Err(_) => self
                 .reader
                 .note_that_a_unit_was_read_with_one_copy_failing(locations),
-            Ok(Some(_) | None) | Err(_) => self.one_copy_failing_note_dropped.set(true),
         }
     }
 }
@@ -6163,6 +6166,14 @@ pub const INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT: &str =
 pub const INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER: &str =
     "inode 记录号小于分隔 key 或容器号";
 
+/// 指着一片 inode 叶的那条内部条目的分隔 key（I-9.12）：与记录号、容器号底下都是 u64，各是一个量。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub(crate) struct SeparatorKeyOfAnInodeLeaf(pub(crate) u64);
+
+/// 一片 inode 叶容器头里的容器号（I-9.4）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub(crate) struct ContainerNumberOfAnInodeLeaf(pub(crate) u64);
+
 /// 一片 inode 叶容器里的一条记录号不小于指着这片叶的那条内部条目的分隔 key、也不小于这片叶的容器号（I-9.4 第一句；分隔 key 那一半是
 /// I-9.12 第二句「第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界」，逐条记录判就判到了下界）。读 inode 树的四个读者——可写挂载重建、冷走读、
 /// 快照第一个文件的冷读与只读挂载打开池——逐条记录调这一个，同一张镜像一个结局（判决 `research/prompts/m4-close-code-r3-main-verification.md`
@@ -6171,11 +6182,11 @@ pub const INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER: &str =
 /// # Errors
 /// 记录号小于分隔 key 或容器号 ⇒ [`RecoveryFailure::InvariantViolated`]（I-9.4，[`INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER`]）。
 pub(crate) fn inode_record_is_not_below_the_separator_key_and_the_container(
-    record_inode: u64,
-    separator_key: u64,
-    container: u64,
+    record_inode: InodeNumber,
+    separator_key: SeparatorKeyOfAnInodeLeaf,
+    container: ContainerNumberOfAnInodeLeaf,
 ) -> Result<(), RecoveryFailure> {
-    if record_inode < separator_key || record_inode < container {
+    if record_inode.0 < separator_key.0 || record_inode.0 < container.0 {
         return Err(RecoveryFailure::InvariantViolated {
             invariant: "I-9.4",
             detail: INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER,
@@ -7008,9 +7019,9 @@ fn inode_leaf_container_judged_against_its_entry(
             what: "inode 记录",
         })?;
         inode_record_is_not_below_the_separator_key_and_the_container(
-            record.inode,
-            separator_key,
-            identity.container,
+            InodeNumber(record.inode),
+            SeparatorKeyOfAnInodeLeaf(separator_key),
+            ContainerNumberOfAnInodeLeaf(identity.container),
         )?;
         records.push(record);
     }
@@ -9048,13 +9059,13 @@ mod back_chain_judgement_tests {
 mod inode_leaf_records_ascending_across_the_leaves_tests {
     use super::{
         inode_record_is_not_below_the_separator_key_and_the_container,
-        inode_records_of_the_leaf_ascend_after_the_leaves_before, RecoveryFailure,
-        INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM,
+        inode_records_of_the_leaf_ascend_after_the_leaves_before, ContainerNumberOfAnInodeLeaf,
+        RecoveryFailure, SeparatorKeyOfAnInodeLeaf, INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM,
         INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND,
         INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER,
         INODE_SEPARATOR_KEY_NOT_ABOVE_THE_LEAVES_BEFORE_IT,
     };
-    use crate::address::CheckpointTxg;
+    use crate::address::{CheckpointTxg, InodeNumber};
     use crate::records::{
         InodeRecord, InodeRecordTruncationField, NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
     };
@@ -9124,15 +9135,27 @@ mod inode_leaf_records_ascending_across_the_leaves_tests {
             detail: INODE_RECORD_BELOW_THE_SEPARATOR_KEY_OR_THE_CONTAINER,
         });
         assert_eq!(
-            inode_record_is_not_below_the_separator_key_and_the_container(234, 235, 234),
+            inode_record_is_not_below_the_separator_key_and_the_container(
+                InodeNumber(234),
+                SeparatorKeyOfAnInodeLeaf(235),
+                ContainerNumberOfAnInodeLeaf(234)
+            ),
             below
         );
         assert_eq!(
-            inode_record_is_not_below_the_separator_key_and_the_container(234, 234, 235),
+            inode_record_is_not_below_the_separator_key_and_the_container(
+                InodeNumber(234),
+                SeparatorKeyOfAnInodeLeaf(234),
+                ContainerNumberOfAnInodeLeaf(235)
+            ),
             below
         );
         assert_eq!(
-            inode_record_is_not_below_the_separator_key_and_the_container(234, 234, 234),
+            inode_record_is_not_below_the_separator_key_and_the_container(
+                InodeNumber(234),
+                SeparatorKeyOfAnInodeLeaf(234),
+                ContainerNumberOfAnInodeLeaf(234)
+            ),
             Ok(())
         );
     }
@@ -9169,7 +9192,10 @@ mod inode_leaf_records_ascending_across_the_leaves_tests {
 
 #[cfg(test)]
 mod counting_unit_reads_with_one_copy_failing_tests {
-    use super::{PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing};
+    use super::{
+        PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing,
+        PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition, RecoveryFailure,
+    };
     use crate::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
     use crate::pointer::LocationEntry;
 
@@ -9210,6 +9236,49 @@ mod counting_unit_reads_with_one_copy_failing_tests {
         })
     }
 
+    /// 按提示读到只坏一份：映射里查不到这把 key、查映射本身判红时判不出提示过期，照转、照计（只有造坏的镜像走得到）；
+    /// 查到了而落点不是这一组是提示过期，不转、记下没转。
+    #[test]
+    fn a_note_is_dropped_only_when_the_mapping_names_another_position() {
+        let key = [1u8; 27];
+        let noted_when_the_mapping_answers =
+            |answer: Result<Option<[LocationEntry; 2]>, RecoveryFailure>| {
+                let counting = PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(
+                    &PoolReaderWithoutDevices,
+                );
+                let lookup = |_key: &[u8]| answer.clone();
+                let noting = PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition::over(
+                    &counting, &key, &lookup,
+                );
+                noting
+                    .note_that_a_unit_was_read_with_one_copy_failing(&locations_of_a_unit_at(100));
+                (
+                    counting.unit_reads_with_one_copy_failing(),
+                    noting.dropped_a_one_copy_failing_note_off_the_mapping_position(),
+                )
+            };
+        assert_eq!(
+            noted_when_the_mapping_answers(Ok(None)),
+            (1, false),
+            "映射里没有这把 key：照计"
+        );
+        assert_eq!(
+            noted_when_the_mapping_answers(Err(RecoveryFailure::NoValidRoot)),
+            (1, false),
+            "查映射本身判红：照计"
+        );
+        assert_eq!(
+            noted_when_the_mapping_answers(Ok(Some(locations_of_a_unit_at(100)))),
+            (1, false),
+            "提示就是映射里的落点：照计"
+        );
+        assert_eq!(
+            noted_when_the_mapping_answers(Ok(Some(locations_of_a_unit_at(102)))),
+            (0, true),
+            "映射里的落点是别处：提示过期，不计"
+        );
+    }
+
     /// 同一个单元的同一份坏副本报两次（实例表链经两处读两遍那一形，判决 m4-close-code-r3 第二节 T2 ①）只计一次；另一个单元再报一次计第二次；
     /// 交出的位置条目按第一次报的先后、一个单元一组。
     #[test]
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index c1bfcbb..adcb1fd 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -59,7 +59,7 @@ use crate::code_two_tree::{
     plan_the_tree_after_this_publish, plan_the_tree_after_this_publish_down_to_a_zero_root,
     CodeTwoKeyFieldWidths, CodeTwoTreeKey, CodeTwoTreeNodeCapacity, CodeTwoTreeNodeContents,
     CodeTwoTreeNodeOrigin, CodeTwoTreeNodePosition, CodeTwoTreePlan, CodeTwoTreeReadExpectation,
-    CodeTwoTreeRefusal, CodeTwoTreeShape, CodeTwoTreeVersion,
+    CodeTwoTreeRefusal, CodeTwoTreeShape, CodeTwoTreeShapeNode, CodeTwoTreeVersion,
 };
 use crate::compaction_plan::{
     region_is_empty_by_the_records, PlannedRelocation, RelocationDestinationPolicy,
@@ -131,12 +131,12 @@ use crate::records::{
 };
 use crate::recovery::{
     effective_rollback_floor_rereading_unreadable_reads_once, highest_root_instance,
-    snapshot_tree_table_entries_paired_one_to_one,
+    instance_table_chain_of_root, snapshot_tree_table_entries_paired_one_to_one,
     system_configuration_slots_of_one_device_read_once,
     system_configuration_witness_is_undecidable, tree_table_entry_count,
-    verified_system_configuration_slots, PoolReader, PoolReaderOfOneReadPass,
-    PoolReaderStraightFromTheDisk, RecoveryFailure, SnapshotTreeTableEntries,
-    SystemConfigurationSlotsOfOneDevice,
+    verified_system_configuration_slots, PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing,
+    PoolReaderOfOneReadPass, PoolReaderStraightFromTheDisk, RecoveryFailure,
+    SnapshotTreeTableEntries, SystemConfigurationSlotsOfOneDevice,
 };
 use crate::retirement_tree::{
     append_retirement_records_to_the_open_container, retirement_records_of_this_publish,
@@ -5407,6 +5407,17 @@ pub(crate) fn copies_of_a_released_unit_failing_the_release_checksum_check<
     if !some_copy_failed {
         return Vec::new();
     }
+    // 一份没对上、另一份对得上（重读也没对上的不算读到）：这一读读到只坏一份（D4（校验和位置） 已定项 4「读到只坏一份时计数」）。
+    if reading_of_each_copy
+        .iter()
+        .any(|(_, reading)| match reading {
+            QuarantinedCopyReading::IntactButAnotherCopyFailed => true,
+            QuarantinedCopyReading::ChecksumMismatch
+            | QuarantinedCopyReading::UnreadableAfterOneReread => false,
+        })
+    {
+        reader.note_that_a_unit_was_read_with_one_copy_failing(locations);
+    }
     // 两条位置项各指池里一块不同的盘（调用方判过；第一版池里恰好两块盘，mkfs 断言过），两份都留：按位置项的次序（设备身份升序）各交一份。
     reading_of_each_copy
         .into_iter()
@@ -10555,6 +10566,9 @@ pub enum RelocationBatchRefused {
         planned_source: Placement,
         mapping_locations: Option<[LocationEntry; 2]>,
     },
+    /// 另发的那一批（[`publish_the_relocations_and_the_repair_of_the_exempt_units`] 之前认位置）沿链读实例表读不出、解不开：认不出记下的那几组位置条目是不是实例表片。
+    /// 在这一批任何写之前。
+    InstanceTableOfTheVersionUnreadable(RecoveryFailure),
     /// 这个单元的字节不在手里：从盘上重建时提示与映射都读不出的数据单元（D19 已定项 5：照抄位置项、不读内容），
     /// 或留存条目指的单元两份都读不出、核不上位置项里的校验和。原样字节写目的地（判决 G3）做不到，不搬。
     UnitBytesNotAvailable {
@@ -10852,22 +10866,7 @@ pub(crate) fn publish_compaction_step_in_the_session<Device: BlockDevice>(
             CompactionStepCheckBeforePersisting::NothingToCheck,
         ),
     };
-    let mut relocations = Vec::with_capacity(planned.len());
-    {
-        let _inside_the_relocation_sources = lookup_work::entering(LookupSite::RelocationSources);
-        // 计划里每一项按映射 key 查一次上一版：按 key 的索引与批量读者在第一项要查时建一次（七·6，用户 2026-10-21 定并进），
-        // 不逐项在 `mapped_units` 上找、逐 key 重解叶。
-        let mut previous_read_by_key = None;
-        for planned_relocation in planned {
-            relocations.push(relocation_checked_against_the_version(
-                &*pool.devices,
-                previous_read_by_key.get_or_insert_with(|| {
-                    MappedUnitsOfAVersionReadByKey::of_the_version(previous)
-                }),
-                planned_relocation,
-            )?);
-        }
-    }
+    let relocations = relocations_checked_against_the_version(pool, allocator, previous, planned)?;
     publish_compaction_step_after_its_check(
         pool,
         allocator,
@@ -10879,6 +10878,107 @@ pub(crate) fn publish_compaction_step_in_the_session<Device: BlockDevice>(
     )
 }
 
+/// 计划里每一项对着上一版核过、带上搬它要的东西（[`relocation_checked_against_the_version`]）。按留存条目读源读到只坏一份的进这次挂载别的读那一份账
+/// （D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）：有一项核不过时前面几项读到的也进，读是真发到盘上的。
+///
+/// # Errors
+/// [`relocation_checked_against_the_version`] 的那几种，在任何写之前。
+fn relocations_checked_against_the_version<Device: BlockDevice>(
+    pool: &PoolWriter<'_, Device>,
+    allocator: &PoolAllocator,
+    previous: &TransactionOutput,
+    planned: &[PlannedRelocation],
+) -> Result<Vec<RelocationOfThePublish>, RelocationBatchRefused> {
+    let _inside_the_relocation_sources = lookup_work::entering(LookupSite::RelocationSources);
+    let devices_counting_unit_reads_with_one_copy_failing =
+        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*pool.devices);
+    // 计划里每一项按映射 key 查一次上一版：按 key 的索引与批量读者在第一项要查时建一次（七·6，用户 2026-10-21 定并进），
+    // 不逐项在 `mapped_units` 上找、逐 key 重解叶。
+    let mut previous_read_by_key = None;
+    let relocations: Result<Vec<RelocationOfThePublish>, RelocationBatchRefused> = planned
+        .iter()
+        .map(|planned_relocation| {
+            relocation_checked_against_the_version(
+                &devices_counting_unit_reads_with_one_copy_failing,
+                previous_read_by_key.get_or_insert_with(|| {
+                    MappedUnitsOfAVersionReadByKey::of_the_version(previous)
+                }),
+                planned_relocation,
+            )
+        })
+        .collect();
+    allocator.count_the_other_unit_reads_with_one_copy_failing(
+        devices_counting_unit_reads_with_one_copy_failing
+            .locations_of_the_units_read_with_one_copy_failing(),
+    );
+    relocations
+}
+
+/// 用户改动发成之后另发的那一批（`mounted_session` 的 `relocate_the_units_read_with_one_copy_failing_after_the_change`）：搬 `planned` 那几个单元（可以一个都没有）、
+/// 同一次发布换下 `repair` 点名的豁免映射的单元（[`RepairOfTheExemptUnits`]，可以一个都没有），两样不能都空。同一个事务层、同一条发布路径：
+/// 意图照抄（照 [`publish_relocation_batch`]，意图的签发与删不归这一批），计划核法、准入、落盘同 [`publish_compaction_step`]，实例表要换就照写行整条链重写。
+/// 不新加提交步骤、不改可写挂载的分段（这一批在用户改动之后，不在挂载那一串里）。
+///
+/// # Errors
+/// 同 [`publish_relocation_batch`]。
+///
+/// # Panics
+/// `planned` 与 `repair` 都是空的：调用方两样都空就不发。
+pub(crate) fn publish_the_relocations_and_the_repair_of_the_exempt_units<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    planned: &[PlannedRelocation],
+    repair: &RepairOfTheExemptUnits,
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, RelocationBatchRefused> {
+    assert!(
+        !(planned.is_empty() && repair.is_empty()),
+        "另发的那一批要么搬单元、要么换豁免映射的单元：调用方两样都空就不发"
+    );
+    let relocations = relocations_checked_against_the_version(pool, allocator, previous, planned)?;
+    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
+    // 事务号接着这个实例（同整理的每一次发布，[`publish_compaction_step_after_its_check`]）。
+    let transaction_of_this_publish = previous.highest_transaction_number_in_this_instance + 1;
+    let deletion_and_lineage = DeletionAndLineageOfThePublish {
+        step: DeletionStepOfThePublish::NotADeletion,
+        container_tree_capacities: pool.container_tree_capacities(),
+        lineage: LineageStepOfThePublish::NoLineageRecord,
+        snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
+        logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
+    };
+    publish_version_of_trees_relocating_units_after_a_check(
+        pool,
+        allocator,
+        PublishPlan {
+            txg,
+            counter: previous.record.counter + 1,
+            transaction: transaction_of_this_publish,
+            highest_transaction_number_before_this_publish: previous
+                .highest_transaction_number_in_this_instance,
+            instance,
+            back_chain: back_chain_of(&previous.record_bytes),
+            file: None,
+            new_inode_records: &[],
+            instance_table: match &repair.instance_table_rewrite {
+                Some(rewrite) => InstanceTablePlan::Rewrite(rewrite.clone()),
+                None => InstanceTablePlan::Carry(previous.root.instance_table),
+            },
+            tree_birth_txg: previous.tree_birth_txg(),
+            tree_identifier_watermark: previous.root.tree_identifier_watermark,
+            rollback_floor: previous.root.rollback_floor,
+        },
+        Some(previous),
+        previous.tree_identifiers,
+        UnmountMarker::NotWrittenByTheUnmountSequence,
+        &deletion_and_lineage,
+        &relocations,
+        &repair.mapping_keys_routed_through_the_nodes,
+        &[],
+        |_version_after_this_publish| Ok::<(), RelocationBatchRefused>(()),
+    )
+}
+
 /// 整理的一次发布落盘之前在算好的账上核什么（[`publish_compaction_step`]）。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum CompactionStepCheckBeforePersisting {
@@ -11177,16 +11277,24 @@ fn relocation_checked_against_the_version<Reader: PoolReader + ?Sized>(
         RelocatedUnitHolder::KeptForTheSnapshots => {
             let unit_bytes = usize::try_from(planned.source.span * SLOT_BYTES)
                 .expect("一个单元两槽以内，字节数装得进 usize");
+            let mut a_copy_before_this_one_failed = false;
             source_locations
                 .iter()
                 .find_map(|location| {
-                    reader
+                    let copy = reader
                         .read(
                             location.device,
                             location.slot.to_device_offset(),
                             unit_bytes,
                         )
-                        .filter(|copy| crc32_castagnoli(copy) == location.unit_checksum)
+                        .filter(|copy| crc32_castagnoli(copy) == location.unit_checksum);
+                    match (&copy, a_copy_before_this_one_failed) {
+                        (Some(_), true) => reader
+                            .note_that_a_unit_was_read_with_one_copy_failing(&source_locations),
+                        (Some(_), false) => {}
+                        (None, _) => a_copy_before_this_one_failed = true,
+                    }
+                    copy
                 })
                 .unwrap_or_default()
         }
@@ -11731,23 +11839,36 @@ pub fn relocations_of_old_data_units_kept_for_the_snapshots(
     planned
 }
 
-/// 一版里此刻住在 `locations` 这几组位置条目上的那几个单元的映射 key：进中央映射的单元按映射里那两条位置条目认，留存的映射条目
-/// （`version.mapping_entries_kept_for_the_snapshots`）按它记的那两条认；一把 key 只出一次，按 `mapped_units` 的次序、留存的排在后面。
-/// 可写挂载读盘建上一版时读到只坏一份的那几个单元（`recovery::PoolReaderCountingUnitReadsWithOneCopyFailing::locations_of_the_units_read_with_one_copy_failing`）
-/// 在挂载交回之前经它认成 key，交给会话下一次用户改动之后另发的那一批（用户 2026-11-05 定「可写挂载时搬」）。
-/// 豁免映射的三类（映射树自己的节点、树表、实例表）不在映射里、认不出；挂载这一串的发布里已经换了落点的单元（树表、记账树这类每次发布都换下的）
-/// 也认不出——它们的旧落点照释放之前读盘核那一路走。
+/// 一版里 `locations` 这几组位置条目认出来的结果（[`units_at_these_locations`]）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct UnitsAtTheseLocations {
+    /// 住在这几组位置条目上的、进中央映射或留存着映射条目的那几个单元的映射 key。一把 key 只出一次，按 `mapped_units` 的次序、留存的排在后面。
+    pub mapping_keys: Vec<Vec<u8>>,
+    /// 认不出映射 key 的那几组，按交进来的次序：豁免映射的三类（映射树自己的节点、树表、实例表片）的，与这一版里已经没有单元住着的。
+    pub locations_without_a_mapping_key: Vec<[LocationEntry; 2]>,
+}
+
+/// 按一版认 `locations` 这几组位置条目（[`UnitsAtTheseLocations`]）：进中央映射的单元按映射里那两条位置条目认，留存的映射条目
+/// （`version.mapping_entries_kept_for_the_snapshots`）按它记的那两条认。可写挂载交回之前按挂载之后那一版认建上一版读到只坏一份的那几组
+/// （`recovery::PoolReaderCountingUnitReadsWithOneCopyFailing::locations_of_the_units_read_with_one_copy_failing`），会话下一次用户改动发成之后
+/// 按那一版认别的读记下的那几组，认出的 key 进另发的那一批（用户 2026-11-05 定「可写挂载时搬」）。
+/// 挂载这一串的发布、或这次改动已经换了落点的单元认不出：它们的旧落点只在读到只坏一份之后记着那组位置条目时走释放之前读盘核
+/// （[`copies_of_the_exempt_units_read_with_one_copy_failing`] 管豁免映射的三类）。
 #[must_use]
-pub fn mapping_keys_of_the_units_at_these_locations(
+pub fn units_at_these_locations(
     version: &TransactionOutput,
     locations: &[[LocationEntry; 2]],
-) -> Vec<Vec<u8>> {
+) -> UnitsAtTheseLocations {
     let mut mapping_keys: Vec<Vec<u8>> = Vec::new();
     if locations.is_empty() {
-        return mapping_keys;
+        return UnitsAtTheseLocations {
+            mapping_keys,
+            locations_without_a_mapping_key: Vec::new(),
+        };
     }
+    let mut locations_with_a_mapping_key: Vec<[LocationEntry; 2]> = Vec::new();
     let mut central_mapping = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
-    // 迭代上界是这一版进映射的单元数加留存条目数；跨轮携带的只有已认出的那几把 key。
+    // 迭代上界是这一版进映射的单元数加留存条目数；跨轮携带的只有已认出的那几把 key 与那几组位置条目。
     for (_, mapping_key) in &version.mapped_units {
         if mapping_keys.contains(mapping_key) {
             continue;
@@ -11755,6 +11876,7 @@ pub fn mapping_keys_of_the_units_at_these_locations(
         match central_mapping.locations_of_key(mapping_key) {
             MappingLookup::Found(mapped_locations) if locations.contains(&mapped_locations) => {
                 mapping_keys.push(mapping_key.clone());
+                locations_with_a_mapping_key.push(mapped_locations);
             }
             MappingLookup::Found(_)
             | MappingLookup::NodeMalformedOrNoEntryWithThisKey
@@ -11764,9 +11886,231 @@ pub fn mapping_keys_of_the_units_at_these_locations(
     for (kept_key, kept_locations) in &version.mapping_entries_kept_for_the_snapshots {
         if !mapping_keys.contains(kept_key) && locations.contains(kept_locations) {
             mapping_keys.push(kept_key.clone());
+            locations_with_a_mapping_key.push(*kept_locations);
         }
     }
-    mapping_keys
+    UnitsAtTheseLocations {
+        mapping_keys,
+        locations_without_a_mapping_key: locations
+            .iter()
+            .filter(|read_locations| !locations_with_a_mapping_key.contains(read_locations))
+            .copied()
+            .collect(),
+    }
+}
+
+/// 另发的那一批要换落点的豁免映射的单元（D4（校验和位置） 已定项 4 的三类：映射树自己的节点、树表、实例表片；[`repair_of_the_exempt_units_at_these_locations`]）：
+/// 它们认不出映射 key、搬迁入口不收（[`RelocationBatchRefused::PinnedUnitInThePlan`]），换落点照它们自己的重写路子走：那一次发布本来就重写树表与映射树根；
+/// 映射树根之下的节点经 `mapping_keys_routed_through_the_nodes` 把从根到叶那条路整条重写（[`plan_rewriting_the_paths_to_the_moved_entries`]，条目的位置不变）；
+/// 实例表整条链 COW 重写、行不变（D18（块里携带什么信息） 已定项 11「每次写行 COW 重写整条链」，没有不被改写的片）。
+#[derive(Clone, Debug)]
+pub struct RepairOfTheExemptUnits {
+    /// 这一批换下的那几个豁免映射的角色，按映射树节点、树表、实例表片的次序。
+    pub units: Vec<TransactionUnit>,
+    mapping_keys_routed_through_the_nodes: Vec<Vec<u8>>,
+    instance_table_rewrite: Option<InstanceTableRewrite>,
+}
+
+impl RepairOfTheExemptUnits {
+    /// 这一批要不要为豁免映射的单元发。
+    #[must_use]
+    pub fn is_empty(&self) -> bool {
+        self.units.is_empty()
+    }
+}
+
+/// `locations_without_a_mapping_key`（[`UnitsAtTheseLocations`]）里此刻是 `version` 那一版豁免映射的单元的那几组要怎么换落点。
+/// 映射树节点与树表只比内存里这一版的指针；余下的有一组才沿链读实例表（只坏一份的读才有这几组，没有坏副本的发布不多读）。
+/// 没有单元住着的那几组（这次改动已经换下的）不进结果。
+///
+/// # Errors
+/// 还有没认出的那几组而实例表链读不出、解不开：认不出它们是不是实例表片。
+pub fn repair_of_the_exempt_units_at_these_locations<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    version: &TransactionOutput,
+    locations_without_a_mapping_key: &[[LocationEntry; 2]],
+) -> Result<RepairOfTheExemptUnits, RecoveryFailure> {
+    let mut units: Vec<TransactionUnit> = Vec::new();
+    let mut mapping_keys_routed_through_the_nodes: Vec<Vec<u8>> = Vec::new();
+    let mut locations_left: Vec<[LocationEntry; 2]> = Vec::new();
+    let mapping_tree = &version.central_mapping_tree;
+    // 迭代上界是交进来的组数乘映射树的节点数；跨轮携带的只有认出的那几样。
+    for locations in locations_without_a_mapping_key {
+        let mapping_tree_node = mapping_tree
+            .shape
+            .nodes()
+            .iter()
+            .zip(&mapping_tree.pointers)
+            .find(|(_, pointer)| pointer.locations == *locations)
+            .map(|(node, _)| node);
+        if let Some(node) = mapping_tree_node {
+            units.push(
+                MultiLevelCodeTwoTree::CentralMapping
+                    .role_of_node(node.position, &mapping_tree.shape),
+            );
+            mapping_keys_routed_through_the_nodes
+                .push(a_mapping_key_below_the_node(mapping_tree, node));
+        } else if version.root.tree_table.locations == *locations {
+            units.push(TransactionUnit::TreeTable);
+        } else {
+            locations_left.push(*locations);
+        }
+    }
+    if locations_left.is_empty() {
+        return Ok(RepairOfTheExemptUnits {
+            units,
+            mapping_keys_routed_through_the_nodes,
+            instance_table_rewrite: None,
+        });
+    }
+    let chain = instance_table_chain_of_root(reader, &version.root)?;
+    let any_page_read_with_one_copy_failing = chain
+        .page_pointers
+        .iter()
+        .any(|page_pointer| locations_left.contains(&page_pointer.locations));
+    let instance_table_rewrite = if any_page_read_with_one_copy_failing {
+        units.extend((0..chain.page_pointers.len()).map(|position| {
+            TransactionUnit::of_instance_table_page(InstanceTablePageIndex(
+                u64::try_from(position).expect("片序号"),
+            ))
+        }));
+        Some(InstanceTableRewrite {
+            rows: chain.records.rows,
+            replaced_chain: chain.page_pointers,
+        })
+    } else {
+        None
+    };
+    Ok(RepairOfTheExemptUnits {
+        units,
+        mapping_keys_routed_through_the_nodes,
+        instance_table_rewrite,
+    })
+}
+
+/// `node` 下面的一把映射 key（沿头一个孩子走到叶、取叶里头一把）：把它当「条目换了位置」交给发布，从根到它那片叶的路整条重写，`node` 在路上。
+///
+/// # Panics
+/// 节点的孩子不在同一份形状里、或叶或内部节点是空的：形状由规划或读树造出来，节点都不空、孩子都在（`code_two_tree` 读树那一步判过）。
+fn a_mapping_key_below_the_node(
+    mapping_tree: &CodeTwoTreeVersion,
+    node: &CodeTwoTreeShapeNode,
+) -> Vec<u8> {
+    let mut contents = &node.contents;
+    // 迭代上界：每一轮下降一层，至多树高那么多轮；走到叶就停。
+    loop {
+        match contents {
+            CodeTwoTreeNodeContents::Leaf { keys } => {
+                return keys
+                    .first()
+                    .expect("映射树的叶不空：规划与读树都不留空叶")
+                    .bytes()
+                    .to_vec();
+            }
+            CodeTwoTreeNodeContents::Internal { children } => {
+                let child = children
+                    .first()
+                    .expect("映射树的内部节点至少一个孩子：规划与读树都不留空节点")
+                    .child;
+                contents = &mapping_tree
+                    .shape
+                    .node(child)
+                    .expect("孩子的位置取自同一份形状")
+                    .contents;
+            }
+        }
+    }
+}
+
+/// 这次发布换下的豁免映射的单元里，读到只坏一份之后记着位置条目的那几个（`locations_read_with_one_copy_failing`，
+/// [`PoolAllocator::locations_read_with_one_copy_failing_waiting_for_the_next_user_change`]）照释放之前读盘核那一路读：一份没对上就两份都隔离
+/// （D4（校验和位置） 已定项 4「旧落点照释放前核隔离」，读法同 [`copies_of_a_released_unit_failing_the_release_checksum_check`]）。
+/// 没有记着的不读：豁免映射的单元不在硬规则 1 的射程里（D19（块指针的结构与宽度预算） 已定项 5），这一读只跟着读到只坏一份的那几组。
+fn copies_of_the_exempt_units_read_with_one_copy_failing<Reader: PoolReader + ?Sized>(
+    previous: &TransactionOutput,
+    previous_roles_replaced: &[TransactionUnit],
+    instance_table: &InstanceTablePlan,
+    locations_read_with_one_copy_failing: &[[LocationEntry; 2]],
+    reader: &Reader,
+) -> Vec<CopyQuarantinedAfterReleaseChecksumMismatch> {
+    let mut quarantined = Vec::new();
+    if locations_read_with_one_copy_failing.is_empty() {
+        return quarantined;
+    }
+    let mut check_when_it_was_read_with_one_copy_failing =
+        |identity: TransactionUnit, locations: &[LocationEntry; 2]| {
+            if locations_read_with_one_copy_failing.contains(locations) {
+                quarantined.extend(
+                    copies_of_a_released_unit_failing_the_release_checksum_check(
+                        identity, locations, reader,
+                    ),
+                );
+            }
+        };
+    // 迭代上界是这次换下的角色数；不是豁免映射的那几个走 `copies_failing_the_release_checksum_check`。
+    for identity in previous_roles_replaced {
+        match identity {
+            TransactionUnit::MappingTreeNodeBelowTheRoot(position) => {
+                check_when_it_was_read_with_one_copy_failing(
+                    *identity,
+                    &previous
+                        .central_mapping_tree
+                        .pointer_of(*position)
+                        .locations,
+                );
+            }
+            TransactionUnit::MappingTree => check_when_it_was_read_with_one_copy_failing(
+                *identity,
+                &previous.root.mapping_root.locations,
+            ),
+            TransactionUnit::TreeTable => check_when_it_was_read_with_one_copy_failing(
+                *identity,
+                &previous.root.tree_table.locations,
+            ),
+            TransactionUnit::InstanceTable
+            | TransactionUnit::InstanceTablePageAfterTheFirst(_)
+            | TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
+            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
+            | TransactionUnit::Data(_)
+            | TransactionUnit::ExtentLowerNode(_)
+            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
+            | TransactionUnit::ExtentRoot
+            | TransactionUnit::InodeLeafContainer(_)
+            | TransactionUnit::InodeRoot
+            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
+            | TransactionUnit::AllocationTree
+            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+            | TransactionUnit::AccountingTree
+            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+            | TransactionUnit::DeadlistTree
+            | TransactionUnit::DeletedInodesRoot
+            | TransactionUnit::LoggedOperationsRoot
+            | TransactionUnit::TombstoneLeafContainer(_)
+            | TransactionUnit::TombstoneRoot
+            | TransactionUnit::TombstoneNodeBelowTheRoot(_)
+            | TransactionUnit::LineageLeafContainer(_)
+            | TransactionUnit::LineageRoot
+            | TransactionUnit::LineageNodeBelowTheRoot(_)
+            | TransactionUnit::RetirementLeafContainer(_)
+            | TransactionUnit::RetirementRoot
+            | TransactionUnit::RetirementNodeBelowTheRoot(_) => {}
+        }
+    }
+    // 整条实例表旧链由计划带着（第 1 片起的指针不在上一版的内存态里）。
+    match instance_table {
+        InstanceTablePlan::Rewrite(rewrite) => {
+            for (position, page_pointer) in rewrite.replaced_chain.iter().enumerate() {
+                check_when_it_was_read_with_one_copy_failing(
+                    TransactionUnit::of_instance_table_page(InstanceTablePageIndex(
+                        u64::try_from(position).expect("片序号"),
+                    )),
+                    &page_pointer.locations,
+                );
+            }
+        }
+        InstanceTablePlan::Carry(_) => {}
+    }
+    quarantined
 }
 
 /// 可写挂载读盘建上一版时读到只坏一份的那几个单元（`mapping_keys`，[`mapping_keys_of_the_units_at_these_locations`] 认出来的）里，到 `version`
@@ -11774,6 +12118,7 @@ pub fn mapping_keys_of_the_units_at_these_locations(
 /// [`relocations_of_old_data_units_kept_for_the_snapshots`] 同一批发，`planned_before` 是那一份、已在里面的 key 不重复）：源落点取映射里那两条
 /// 位置条目（或留存条目记的那两条）的槽，跨度按 key 的类标签归到的用户可见角色取（[`user_visible_role_standing_for_a_unit_class`]）。
 /// 到 `version` 这一版已经不在的（这次改动或挂载之后的发布把它整个重写了、旧落点照释放之前读盘核那一路走）不进计划。
+/// 会话读文件、推抬 F、释放之前读盘核、搬留存单元读源记下的那几个（[`units_at_these_locations`] 认出的 key）同一条路。
 #[must_use]
 pub fn relocations_of_the_units_read_with_one_copy_failing_while_mounting(
     version: &TransactionOutput,
@@ -11824,7 +12169,7 @@ pub fn relocations_of_the_units_read_with_one_copy_failing_while_mounting(
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum CopiesOfAUnitRead {
     /// 头一条就对上，或两条位置提示都没对上、映射那一组头一条对上（提示过期与两份都坏分不开，不算只坏一份），
-    /// 或按提示读到只坏一份而提示不是这个单元此刻在映射里的落点（提示过期：旧槽上坏的那一份不归这个单元计）。
+    /// 或按提示读到只坏一份而映射里查到这个单元此刻的落点不是提示（提示过期：旧槽上坏的那一份不归这个单元计）。
     WithoutAFailingCopy,
     /// 同一组里前一条读不回字节或整单元 CRC-32C 对不上，后一条对上：只坏一份。
     FromTheOtherCopyAfterOneFailed,
@@ -11903,8 +12248,9 @@ pub fn read_back_the_old_data_unit_telling_whether_a_copy_failed<Reader: PoolRea
                         MappingLookup::Found(mapped_locations) => {
                             mapped_locations == pointer.locations
                         }
+                        // 查不出落点就判不出提示过期：这一份是真读到的坏副本，照计（查不到的判红留给照映射释放那一步）。
                         MappingLookup::NodeMalformedOrNoEntryWithThisKey
-                        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => false,
+                        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => true,
                     };
                 if !hints_are_the_mapping_position {
                     copies_read.set(CopiesOfAUnitRead::WithoutAFailingCopy);
@@ -13014,15 +13360,26 @@ fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
         .resolved
         .deadlist
         .user_visible_roles_kept_for_the_snapshots;
+    // 释放之前读盘核读到只坏一份的进这次挂载别的读那一份账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）。
+    let devices_counting_unit_reads_with_one_copy_failing =
+        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*pool.devices);
     let mut quarantine = match (previous, release_checksum_check) {
         (Some(previous_version), ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt) => {
-            copies_failing_the_release_checksum_check(
+            let mut copies_failing = copies_failing_the_release_checksum_check(
                 previous_version,
                 &settled.previous_roles_replaced,
                 kept_for_the_snapshots,
-                &*pool.devices,
+                &devices_counting_unit_reads_with_one_copy_failing,
                 FileContentOfThePublish::of_the_plan(plan),
-            )?
+            )?;
+            copies_failing.extend(copies_of_the_exempt_units_read_with_one_copy_failing(
+                previous_version,
+                &settled.previous_roles_replaced,
+                &plan.instance_table,
+                &allocator.locations_read_with_one_copy_failing_waiting_for_the_next_user_change(),
+                &devices_counting_unit_reads_with_one_copy_failing,
+            ));
+            copies_failing
         }
         (Some(previous_version), ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition) => {
             // 预演不读盘，但映射条目的位置项指池外的盘、两条指同一块盘那一判不读盘，照样判（发布路径在读盘核之前判的同一件事）。
@@ -13060,7 +13417,7 @@ fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
                     copies_of_a_released_unit_failing_the_release_checksum_check(
                         role,
                         locations,
-                        &*pool.devices,
+                        &devices_counting_unit_reads_with_one_copy_failing,
                     ),
                 );
             }
@@ -13099,7 +13456,7 @@ fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
                         copies_of_a_released_unit_failing_the_release_checksum_check(
                             relocation.role_for_the_checks,
                             &relocation.source_locations,
-                            &*pool.devices,
+                            &devices_counting_unit_reads_with_one_copy_failing,
                         ),
                     );
                 }
@@ -13107,6 +13464,10 @@ fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
             ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition => {}
         }
     }
+    allocator.count_the_other_unit_reads_with_one_copy_failing(
+        devices_counting_unit_reads_with_one_copy_failing
+            .locations_of_the_units_read_with_one_copy_failing(),
+    );
     publish_admitted(
         pool,
         allocator,
```
