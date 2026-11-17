# m4-close-fixes-code-r3：r2 判决第四节与 P2a 的产品代码改动（补丁 m4-close-fixes-r2-writer/patch/crates.patch 里 crates/singlefs-core 的块原样抄录）

```diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 8489bd64971a59f6256dcb254bc100a9599f27a9..8e933c3a6ef49ac8e36f6d38ba10890f83f82d27 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -26,6 +26,7 @@ use crate::instance_table::InstanceTableRecords;
 use crate::journal::slot_after_the_journal_ring;
 use crate::logged_operations::CompactionRegion;
 use crate::pointer::LocationEntry;
+use crate::recovery::{PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing};
 use crate::root_record::RootRecord;
 use crate::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
 use crate::transaction::FrozenPublish;
@@ -1863,8 +1864,10 @@ impl ReclaimFloorWhenTheRootWasRecorded {
     }
 }
 
-/// 这次可写挂载里建上一版与读—改—写读回之外的读读到只坏一份的账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）：
-/// 经会话读文件、推抬 F 与发 discard 之前沿链读实例表、释放之前读盘核、搬留存单元读源。
+/// 这次可写挂载里建上一版与读—改—写读回之外的读读到只坏一份的账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」），每一处读与不接的依据见
+/// [`PoolAllocator::reading_and_counting_the_other_unit_reads_with_one_copy_failing`]。
+/// 挂载这一串读到的那几组也在这份账上等下一次用户改动（[`PoolAllocator::keep_the_locations_read_with_one_copy_failing_uncounted`]），只是不进这个数：
+/// 挂载读到的一份坏副本，数据单元与映射树叶一样，挂载那个数计一次，之后再读、或另发那一批释放之前读盘核，这个数都不加。
 #[derive(Debug, Default)]
 struct OtherUnitReadsWithOneCopyFailing {
     /// 读到只坏一份的次数：一组位置条目等着下一次用户改动的这一程里只算一次。
@@ -1979,6 +1982,27 @@ impl PoolAllocator {
             .reads
     }
 
+    /// 经数只坏一份的那一层做一次读（`read`），读完（读成与否）把数到的那几组记进账（[`Self::count_the_other_unit_reads_with_one_copy_failing`]）。
+    /// 可写挂载期间经 `reader` 按两条位置条目读单元的读都走它或同形的写法：推抬 F 与发 discard 之前读根环与实例表、整理的回退门、回退读现行那一版与目标那一版、
+    /// 回退的释放之前读盘核、清扫的判与发。账各份分配器拷贝共用（读是真发到盘上的），所以 `&self` 够了。
+    pub(crate) fn reading_and_counting_the_other_unit_reads_with_one_copy_failing<
+        'reader,
+        Reader: PoolReader + ?Sized,
+        ReadOutput,
+    >(
+        &self,
+        reader: &'reader Reader,
+        read: impl FnOnce(&PoolReaderCountingUnitReadsWithOneCopyFailing<'reader, Reader>) -> ReadOutput,
+    ) -> ReadOutput {
+        let counting_reader =
+            PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(reader);
+        let read_output = read(&counting_reader);
+        self.count_the_other_unit_reads_with_one_copy_failing(
+            counting_reader.locations_of_the_units_read_with_one_copy_failing(),
+        );
+        read_output
+    }
+
     /// 记下这一读读到只坏一份的那几组位置条目：已经在等下一次用户改动的不再加次数，别的加一次、接在后面。
     ///
     /// # Panics
@@ -2049,7 +2073,7 @@ impl PoolAllocator {
             .clone()
     }
 
-    /// 那几组位置条目里去掉 `is_handled` 判真的那几组（另发的那一批试过一次、或挂载交回之前认成了映射 key）。次数不动。
+    /// 那几组位置条目里去掉 `is_handled` 判真的那几组（另发的那一批试过一次）。次数不动。
     ///
     /// # Panics
     /// 账的锁中毒（同 [`Self::other_unit_reads_with_one_copy_failing`]）。
diff --git a/crates/singlefs-core/src/compaction_plan.rs b/crates/singlefs-core/src/compaction_plan.rs
index 8fca503341e62703df96d45ac216c91ebc242de7..f68cac3674576f402aa01c0fae8211f2eb03072e 100644
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -519,12 +519,12 @@ pub enum RelocationPublishOrder {
     SourceReleasedWhileTheMappingStillNamesItForTests,
 }
 
-/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支能被测试强制进入）：会话为读回时只坏一份、又被活快照钉着的旧单元另发的那一批搬迁
+/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支能被测试强制进入）：一次用户改动发成之后，会话为读到只坏一份的单元另发的那一批搬迁
 /// （`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`），在发之前当作被空间准入拒。产品路径恒 [`Self::NotForced`]。
 /// 被拒推抬 F 再试那一支在小盘之外的池上造不出「改动放得下、紧接着那一批放不下」，靠它进；
 /// 开关造的拒绝在这一批的写入口建起之前交回，一个写都没发。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub enum RelocationKeptForTheSnapshotsRefusedForSpace {
+pub enum RelocationBatchAfterTheChangeRefusedForSpace {
     NotForced,
     /// 接下来这几次发这一批都当作被拒，用完之后照常发。
     ForcedForTheNextAttemptsForTests {
@@ -534,13 +534,13 @@ pub enum RelocationKeptForTheSnapshotsRefusedForSpace {
     ForcedOnEveryAttemptForTests,
 }
 
-/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：会话另发的那一批搬迁（同 [`RelocationKeptForTheSnapshotsRefusedForSpace`]）在发之前
+/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：会话另发的那一批搬迁（同 [`RelocationBatchAfterTheChangeRefusedForSpace`]）在发之前
 /// 当作被拒、而不是空间不够（计划对不上那一版那一种：`transaction::RelocationBatchRefused::PlannedUnitNotInTheVersion`，点名计划里头一把 key）。
 /// 产品路径恒 [`Self::NotForced`]。被拒不是空间不够那一支（不推抬 F、不再试，`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace`）
 /// 在产品路径上要计划在算出来与发之间失效、或发布路径报空间之外的错才进得去，靠它进；开关造的拒绝在这一批的写入口建起之前交回，一个写都没发。
 /// 被空间拒的那个开关同时开着时先照那一个拒（它在前面判）。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub enum RelocationKeptForTheSnapshotsRefusedNotForSpace {
+pub enum RelocationBatchAfterTheChangeRefusedNotForSpace {
     NotForced,
     /// 每一次发这一批都当作被拒（不是空间不够：不再试，一批只发这一次）。
     ForcedOnEveryAttemptForTests,
@@ -549,9 +549,9 @@ pub enum RelocationKeptForTheSnapshotsRefusedNotForSpace {
 /// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：会话为另发的那一批搬迁被空间拒而推抬 F 时，那一串抬 F 当作自己报了错
 /// （`mount::MountError::InstanceTableUnreadable`：`mount::push_one_floor_raise_within_the_admission_budget` 在任何写之前交得出的那一种），一个写都不发。
 /// 产品路径恒 [`Self::NotForced`]。抬 F 自己报错那一支（`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace`）
-/// 在产品路径上要推抬 F 时读盘或落盘出错才进得去，靠它进；要先被空间拒才推抬 F，与 [`RelocationKeptForTheSnapshotsRefusedForSpace`] 合着开。封闭集合。
+/// 在产品路径上要推抬 F 时读盘或落盘出错才进得去，靠它进；要先被空间拒才推抬 F，与 [`RelocationBatchAfterTheChangeRefusedForSpace`] 合着开。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub enum FloorRaiseForTheRelocationKeptForTheSnapshotsFails {
+pub enum FloorRaiseForTheRelocationBatchAfterTheChangeFails {
     NotForced,
     /// 每一次为这一批推抬 F 都当作抬 F 自己报了错。
     ForcedOnEveryRaiseForTests,
@@ -917,15 +917,15 @@ pub struct CompactionOfThisMount {
     /// 会话下一次用户改动发成之后照快照下另发那批同一条路径把它们搬到新槽、两份新写（
     /// `mounted_session::MountedSession::publish_user_change`），交给那一批就清空。只在内存里，新挂载是空的。
     mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting: Vec<Vec<u8>>,
-    /// 只供测试的开关：会话另发的那一批搬迁发之前当作被空间准入拒（[`RelocationKeptForTheSnapshotsRefusedForSpace`]）。新挂载是产品值。
-    relocation_kept_for_the_snapshots_refused_for_space:
-        RelocationKeptForTheSnapshotsRefusedForSpace,
-    /// 只供测试的开关：会话另发的那一批搬迁发之前当作被拒、不是空间不够（[`RelocationKeptForTheSnapshotsRefusedNotForSpace`]）。新挂载是产品值。
-    relocation_kept_for_the_snapshots_refused_not_for_space:
-        RelocationKeptForTheSnapshotsRefusedNotForSpace,
-    /// 只供测试的开关：为另发的那一批推的抬 F 当作自己报错（[`FloorRaiseForTheRelocationKeptForTheSnapshotsFails`]）。新挂载是产品值。
-    floor_raise_for_the_relocation_kept_for_the_snapshots_fails:
-        FloorRaiseForTheRelocationKeptForTheSnapshotsFails,
+    /// 只供测试的开关：会话另发的那一批搬迁发之前当作被空间准入拒（[`RelocationBatchAfterTheChangeRefusedForSpace`]）。新挂载是产品值。
+    relocation_batch_after_the_change_refused_for_space:
+        RelocationBatchAfterTheChangeRefusedForSpace,
+    /// 只供测试的开关：会话另发的那一批搬迁发之前当作被拒、不是空间不够（[`RelocationBatchAfterTheChangeRefusedNotForSpace`]）。新挂载是产品值。
+    relocation_batch_after_the_change_refused_not_for_space:
+        RelocationBatchAfterTheChangeRefusedNotForSpace,
+    /// 只供测试的开关：为另发的那一批推的抬 F 当作自己报错（[`FloorRaiseForTheRelocationBatchAfterTheChangeFails`]）。新挂载是产品值。
+    floor_raise_for_the_relocation_batch_after_the_change_fails:
+        FloorRaiseForTheRelocationBatchAfterTheChangeFails,
     /// 上一次不发布的停（[`CompactionRunStop::publishes_nothing_until_the_next_publish`]）记在哪一版、哪一组开关上：这两样都没变，定时兜底不重判。
     stop_that_publishes_nothing_until_the_next_publish:
         Option<StopThatPublishesNothingUntilTheNextPublish>,
@@ -970,12 +970,12 @@ impl CompactionOfThisMount {
             old_data_unit_reads_back_with_one_copy_failing: 0,
             unit_reads_with_one_copy_failing_while_mounting: 0,
             mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting: Vec::new(),
-            relocation_kept_for_the_snapshots_refused_for_space:
-                RelocationKeptForTheSnapshotsRefusedForSpace::NotForced,
-            relocation_kept_for_the_snapshots_refused_not_for_space:
-                RelocationKeptForTheSnapshotsRefusedNotForSpace::NotForced,
-            floor_raise_for_the_relocation_kept_for_the_snapshots_fails:
-                FloorRaiseForTheRelocationKeptForTheSnapshotsFails::NotForced,
+            relocation_batch_after_the_change_refused_for_space:
+                RelocationBatchAfterTheChangeRefusedForSpace::NotForced,
+            relocation_batch_after_the_change_refused_not_for_space:
+                RelocationBatchAfterTheChangeRefusedNotForSpace::NotForced,
+            floor_raise_for_the_relocation_batch_after_the_change_fails:
+                FloorRaiseForTheRelocationBatchAfterTheChangeFails::NotForced,
             stop_that_publishes_nothing_until_the_next_publish: None,
             publishes_of_the_mount_are_still_to_be_read_from_the_ring: true,
             background_trip_under_way: None,
@@ -1094,74 +1094,74 @@ impl CompactionOfThisMount {
         )
     }
 
-    /// 装上只供测试的开关：会话另发的那一批搬迁发之前当作被空间准入拒（[`RelocationKeptForTheSnapshotsRefusedForSpace`]）。产品路径一处都不调它。
-    pub fn force_the_relocation_kept_for_the_snapshots_refused_for_space_for_tests(
+    /// 装上只供测试的开关：会话另发的那一批搬迁发之前当作被空间准入拒（[`RelocationBatchAfterTheChangeRefusedForSpace`]）。产品路径一处都不调它。
+    pub fn force_the_relocation_batch_after_the_change_refused_for_space_for_tests(
         &mut self,
-        forcing: RelocationKeptForTheSnapshotsRefusedForSpace,
+        forcing: RelocationBatchAfterTheChangeRefusedForSpace,
     ) {
-        self.relocation_kept_for_the_snapshots_refused_for_space = forcing;
+        self.relocation_batch_after_the_change_refused_for_space = forcing;
     }
 
     /// 会话另发那一批的这一次该不该当作被空间准入拒（开关用掉一次就记下：「接下来这几次」少一次，用完回到产品值）。
-    pub(crate) fn take_a_forced_space_refusal_of_the_relocation_kept_for_the_snapshots(
+    pub(crate) fn take_a_forced_space_refusal_of_the_relocation_batch_after_the_change(
         &mut self,
     ) -> bool {
-        match self.relocation_kept_for_the_snapshots_refused_for_space {
-            RelocationKeptForTheSnapshotsRefusedForSpace::NotForced => false,
-            RelocationKeptForTheSnapshotsRefusedForSpace::ForcedOnEveryAttemptForTests => true,
-            RelocationKeptForTheSnapshotsRefusedForSpace::ForcedForTheNextAttemptsForTests {
+        match self.relocation_batch_after_the_change_refused_for_space {
+            RelocationBatchAfterTheChangeRefusedForSpace::NotForced => false,
+            RelocationBatchAfterTheChangeRefusedForSpace::ForcedOnEveryAttemptForTests => true,
+            RelocationBatchAfterTheChangeRefusedForSpace::ForcedForTheNextAttemptsForTests {
                 attempts,
             } => {
-                self.relocation_kept_for_the_snapshots_refused_for_space =
+                self.relocation_batch_after_the_change_refused_for_space =
                     match NonZeroU64::new(attempts.get() - 1) {
                         Some(attempts_left) => {
-                            RelocationKeptForTheSnapshotsRefusedForSpace::ForcedForTheNextAttemptsForTests {
+                            RelocationBatchAfterTheChangeRefusedForSpace::ForcedForTheNextAttemptsForTests {
                                 attempts: attempts_left,
                             }
                         }
-                        None => RelocationKeptForTheSnapshotsRefusedForSpace::NotForced,
+                        None => RelocationBatchAfterTheChangeRefusedForSpace::NotForced,
                     };
                 true
             }
         }
     }
 
-    /// 装上只供测试的开关：会话另发的那一批搬迁发之前当作被拒、不是空间不够（[`RelocationKeptForTheSnapshotsRefusedNotForSpace`]）。
+    /// 装上只供测试的开关：会话另发的那一批搬迁发之前当作被拒、不是空间不够（[`RelocationBatchAfterTheChangeRefusedNotForSpace`]）。
     /// 产品路径一处都不调它。
-    pub fn force_the_relocation_kept_for_the_snapshots_refused_not_for_space_for_tests(
+    pub fn force_the_relocation_batch_after_the_change_refused_not_for_space_for_tests(
         &mut self,
-        forcing: RelocationKeptForTheSnapshotsRefusedNotForSpace,
+        forcing: RelocationBatchAfterTheChangeRefusedNotForSpace,
     ) {
-        self.relocation_kept_for_the_snapshots_refused_not_for_space = forcing;
+        self.relocation_batch_after_the_change_refused_not_for_space = forcing;
     }
 
     /// 会话另发那一批的这一次该不该当作被拒、不是空间不够。
     #[must_use]
-    pub(crate) fn is_the_relocation_kept_for_the_snapshots_forced_refused_not_for_space(
+    pub(crate) fn is_the_relocation_batch_after_the_change_forced_refused_not_for_space(
         &self,
     ) -> bool {
-        match self.relocation_kept_for_the_snapshots_refused_not_for_space {
-            RelocationKeptForTheSnapshotsRefusedNotForSpace::NotForced => false,
-            RelocationKeptForTheSnapshotsRefusedNotForSpace::ForcedOnEveryAttemptForTests => true,
+        match self.relocation_batch_after_the_change_refused_not_for_space {
+            RelocationBatchAfterTheChangeRefusedNotForSpace::NotForced => false,
+            RelocationBatchAfterTheChangeRefusedNotForSpace::ForcedOnEveryAttemptForTests => true,
         }
     }
 
-    /// 装上只供测试的开关：为另发的那一批推的抬 F 当作自己报错（[`FloorRaiseForTheRelocationKeptForTheSnapshotsFails`]）。产品路径一处都不调它。
-    pub fn force_the_floor_raise_for_the_relocation_kept_for_the_snapshots_to_fail_for_tests(
+    /// 装上只供测试的开关：为另发的那一批推的抬 F 当作自己报错（[`FloorRaiseForTheRelocationBatchAfterTheChangeFails`]）。产品路径一处都不调它。
+    pub fn force_the_floor_raise_for_the_relocation_batch_after_the_change_to_fail_for_tests(
         &mut self,
-        forcing: FloorRaiseForTheRelocationKeptForTheSnapshotsFails,
+        forcing: FloorRaiseForTheRelocationBatchAfterTheChangeFails,
     ) {
-        self.floor_raise_for_the_relocation_kept_for_the_snapshots_fails = forcing;
+        self.floor_raise_for_the_relocation_batch_after_the_change_fails = forcing;
     }
 
     /// 为另发那一批推的这一串抬 F 该不该当作自己报了错。
     #[must_use]
-    pub(crate) fn is_the_floor_raise_for_the_relocation_kept_for_the_snapshots_forced_to_fail(
+    pub(crate) fn is_the_floor_raise_for_the_relocation_batch_after_the_change_forced_to_fail(
         &self,
     ) -> bool {
-        match self.floor_raise_for_the_relocation_kept_for_the_snapshots_fails {
-            FloorRaiseForTheRelocationKeptForTheSnapshotsFails::NotForced => false,
-            FloorRaiseForTheRelocationKeptForTheSnapshotsFails::ForcedOnEveryRaiseForTests => true,
+        match self.floor_raise_for_the_relocation_batch_after_the_change_fails {
+            FloorRaiseForTheRelocationBatchAfterTheChangeFails::NotForced => false,
+            FloorRaiseForTheRelocationBatchAfterTheChangeFails::ForcedOnEveryRaiseForTests => true,
         }
     }
 
diff --git a/crates/singlefs-core/src/instance_table_sweep.rs b/crates/singlefs-core/src/instance_table_sweep.rs
index 75c916c7f2905755d2c37327370fb3ba2a158da7..96d3952ab8475b9a705ec5afb3e706b8e86a0aa5 100644
--- a/crates/singlefs-core/src/instance_table_sweep.rs
+++ b/crates/singlefs-core/src/instance_table_sweep.rs
@@ -738,7 +738,12 @@ pub fn erase_the_heads_and_publish_the_sweep<Device: BlockDevice>(
             };
         }
     }
-    let chain = match instance_table_chain_of_root(&*devices, &current_file_version.root) {
+    // 这一读与下面判的那一读（[`judge_the_instance_table_sweep`] 的调用方）读到只坏一份都进别的读那份账（D4（校验和位置） 已定项 4「可写挂载期间别的读」）。
+    let chain_read = allocator
+        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(&*devices, |reader| {
+            instance_table_chain_of_root(reader, &current_file_version.root)
+        });
+    let chain = match chain_read {
         Ok(chain) => chain,
         Err(failure) => {
             return InstanceTableSweepOutcome::NoRowDeleted(
@@ -912,7 +917,11 @@ pub fn sweep_the_instance_table_at_the_end_of_the_mount<Device: BlockDevice>(
         }
         InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold => {}
     }
-    let rows = match instance_table_chain_of_root(&*devices, current.root()) {
+    let chain_read = allocator
+        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(&*devices, |reader| {
+            instance_table_chain_of_root(reader, current.root())
+        });
+    let rows = match chain_read {
         Ok(chain) => u64::try_from(chain.records.rows.len()).expect("行数装得进 u64"),
         Err(failure) => {
             return InstanceTableSweepAtTheEndOfTheMount::Swept(
@@ -954,7 +963,11 @@ pub fn sweep_the_instance_table_while_mounted<Device: BlockDevice>(
             )
         }
     }
-    let judgement = match judge_the_instance_table_sweep(&*devices, current.root()) {
+    let judgement_read = allocator
+        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(&*devices, |reader| {
+            judge_the_instance_table_sweep(reader, current.root())
+        });
+    let judgement = match judgement_read {
         Ok(judgement) => judgement,
         Err(no_row_deleted) => return InstanceTableSweepOutcome::NoRowDeleted(no_row_deleted),
     };
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 1c435f0685cda9c226dc2b9173ce022e0911bfc2..62abe98f1a5747ac2f081d8767e44dfcbe103568 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -4022,6 +4022,7 @@ pub fn rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount<Device:
     >,
 ) -> Result<CheckpointTxg, MountError> {
     admission_ceilings_not_passing_ring_slots_bad_since_the_mount(
+        devices,
         devices,
         system_configuration,
         current_floor,
@@ -4058,12 +4059,17 @@ impl AdmissionCeilings {
 ///
 /// # Errors
 /// 同 [`rollback_floor_ceiling`]。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "读盘的读者、配置与三份记账都在调用方手里，这个函数只算上限，包一层结构没有别的用处"
+)]
 #[allow(
     clippy::ptr_arg,
     reason = "user_visible_tree_root_pointers 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
 )]
 fn admission_ceilings_not_passing_ring_slots_bad_since_the_mount<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
+    reader_of_the_tree_tables: &dyn PoolReader,
     system_configuration: &crate::system_configuration::SystemConfiguration,
     current_floor: CheckpointTxg,
     table: &InstanceTableRecords,
@@ -4110,7 +4116,10 @@ fn admission_ceilings_not_passing_ring_slots_bad_since_the_mount<Device: BlockDe
         .min()
         .ok_or(RecoveryFailure::NoValidRoot)?;
     // 读不出或解不开就拒绝：它算不算非空、它后面那条跟谁比都判不了，读不出要走修复（D16（发布语义） 已定项 1「非空」从盘上怎么认只说了比树表里的根指针）。
-    let pointers_of = |root: &RootRecord| match user_visible_tree_root_pointers(devices, root) {
+    let pointers_of = |root: &RootRecord| match user_visible_tree_root_pointers(
+        reader_of_the_tree_tables,
+        root,
+    ) {
         Ok(pointers) => Ok(pointers),
         Err(failure) => Err(
             MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable {
@@ -4679,15 +4688,20 @@ fn rollback_floor_ceilings_of_this_process<Device: BlockDevice>(
         current_root.rollback_floor,
         current_root.checkpoint_txg
     );
-    admission_ceilings_not_passing_ring_slots_bad_since_the_mount(
-        devices,
-        system_configuration,
-        current_root.rollback_floor,
-        table,
-        &ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(allocator),
-        &ring_slots_bad_or_reading_back_all_zero_at_mount_by(allocator),
-        &ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator),
-    )
+    allocator.reading_and_counting_the_other_unit_reads_with_one_copy_failing(devices, |reader| {
+        admission_ceilings_not_passing_ring_slots_bad_since_the_mount(
+            devices,
+            reader,
+            system_configuration,
+            current_root.rollback_floor,
+            table,
+            &ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(allocator),
+            &ring_slots_bad_or_reading_back_all_zero_at_mount_by(allocator),
+            &ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(
+                allocator,
+            ),
+        )
+    })
 }
 
 /// 算抬 F 的上限读根环时，这一遍读坏、重读一次仍坏就拒的槽（[`rollback_floor_ceiling`] 的 `ring_slots_whose_reread_still_bad_refuses_the_ceiling`）：
@@ -4784,7 +4798,7 @@ fn raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk<Device
 }
 
 /// 这个进程这一刻的准入抬 F 的两个上限（[`rollback_floor_ceilings_of_this_process`]），判有效用现行那一版的实例表（从它的根指着的第 0 片沿链读）。
-/// 只读盘。
+/// 只读盘。这一读与算上限读树表那一读读到只坏一份都进别的读那份账：F 抬不动、紧接着的 [`raise_the_floor_through`] 不跑时只有它们读到（D4（校验和位置） 已定项 4）。
 ///
 /// # Errors
 /// 现行那一版的实例表读不出或解不开；算上限的错（同 [`rollback_floor_ceiling`]）。
@@ -4798,7 +4812,10 @@ fn admission_ceilings_on_caller_inputs_agreeing_with_the_disk<Device: BlockDevic
     allocator: &PoolAllocator,
     current: &TransactionOutput,
 ) -> Result<AdmissionCeilings, MountError> {
-    let current_instance_table = instance_table_chain_of_root(devices, &current.root)
+    let current_instance_table = allocator
+        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(devices, |reader| {
+            instance_table_chain_of_root(reader, &current.root)
+        })
         .map_err(|failure| instance_table_refusal(&failure))?
         .records;
     rollback_floor_ceilings_of_this_process(
@@ -5333,15 +5350,18 @@ fn raise_the_floor_through<Device: BlockDevice>(
     // 先写系统配置那一步写之前按盘上现状再算一次、取两者的大者（`transaction::RollbackFloorOfASystemConfigurationWrite::RaisedFloor`）。
     // 判「有效根」的那张表（根环里最新那条根的实例表）读不出就重读一次，仍读不出就拒这一次抬，不按「不按表滤」算（C554 乙报告 Q6）：
     // 在动分配器与任何写之前返回，分配器还没动。
-    let effective_floor = effective_rollback_floor_rereading_the_newest_instance_table_once(
-        &**devices,
-        &system_configuration.immutable.region_devices,
-        &system_configuration.immutable.sizes,
-        &system_configuration.immutable.filesystem_identifier,
-    )
-    .map_err(|still_unreadable| {
-        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(still_unreadable.into()))
-    })?;
+    let effective_floor = allocator
+        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(&**devices, |reader| {
+            effective_rollback_floor_rereading_the_newest_instance_table_once(
+                reader,
+                &system_configuration.immutable.region_devices,
+                &system_configuration.immutable.sizes,
+                &system_configuration.immutable.filesystem_identifier,
+            )
+        })
+        .map_err(|still_unreadable| {
+            MountError::NewerStateStillUnreadableAfterOneReread(Box::new(still_unreadable.into()))
+        })?;
     if new_floor < effective_floor {
         return Err(
             MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided {
@@ -7854,7 +7874,8 @@ fn establish_instance<Device: BlockDevice>(
     // 读到只坏一份的那几个单元要搬（用户 2026-11-05 定「可写挂载时搬，冷恢复只计数」）：挂载这一串里不另发——另发一批就是挂载的第七段写，
     // 挂载分段、二次崩溃的摆法、取号之前的续做预演都要跟着改（`crates/singlefs-verification/src/writable_mount_phases.rs` 那六段）；
     // 这里按挂载之后那一版把它们认成映射 key 记进挂载期状态，会话下一次用户改动发成之后照快照下另发那批同一条路径搬
-    // （`mounted_session::MountedSession::publish_user_change`）。认不出 key 的几组留在账上：豁免映射的三类里挂载这一串没换下的（映射树根之下的节点）
+    // （`mounted_session::MountedSession::publish_user_change`）。认没认出 key 的几组都留在账上（不加别的读那个数，等到那一批再去掉）：之后再读、那一批释放之前读盘核，
+    // 同一份坏副本不再计进别的读，数据单元与映射树节点同一个口径。豁免映射的三类里挂载这一串没换下的（映射树根之下的节点）
     // 那一批换落点（`transaction::repair_of_the_exempt_units_at_these_locations`），挂载这一串已经重写掉的（树表、实例表片这类）到那一批认不出，照丢。
     compaction_continuation
         .state_of_this_mount
@@ -7873,11 +7894,6 @@ fn establish_instance<Device: BlockDevice>(
             .keep_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting(
                 units_read_with_one_copy_failing.mapping_keys,
             );
-        allocator.forget_the_locations_read_with_one_copy_failing(|locations| {
-            !units_read_with_one_copy_failing
-                .locations_without_a_mapping_key
-                .contains(locations)
-        });
     }
     // 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条「谁跑、什么时候跑」，用户 2026-10-03 定）：写行、暖机、抬 F、删除续做之后，
     // 交回之前判一次。清扫自己不让挂载失败（落点被拒这一轮不删、挂载照样交回可写，Q4），走到哪儿记进交回值。
@@ -10427,12 +10443,8 @@ fn first_copy_that_does_not_read_back<Reader: PoolReader + ?Sized>(
 /// 只有管理员回退读它（D23（journal 的角色与格式） 已定项 14「水位」一格），交进来的只是按 cur 的实例表没被崩溃恢复抛弃的那几条
 /// （Opt2，[`ring_roots_not_abandoned_by_crash_recovery`]）。`record_standing_for_every_root` 只是给重建顶着的一条记录
 /// （重建出来的版本只拿它的 jsn 与事务号，这里都不用）。
-#[allow(
-    clippy::ptr_arg,
-    reason = "rebuild_version 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
-)]
-fn highest_inode_number_watermark_in_the_ring<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+fn highest_inode_number_watermark_in_the_ring(
+    reader: &dyn PoolReader,
     unit_area_start: UnitAreaStart,
     ring_roots: &[RootRecord],
     record_standing_for_every_root: &crate::journal::JournalRecord,
@@ -10441,7 +10453,7 @@ fn highest_inode_number_watermark_in_the_ring<Device: BlockDevice>(
         .iter()
         .filter_map(|root| {
             match rebuild_version_in_the_unit_area_starting_at(
-                devices,
+                reader,
                 unit_area_start,
                 root,
                 Some(record_standing_for_every_root.clone()),
@@ -10725,25 +10737,38 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
         .ok_or(RollbackError::NextCheckpointTxgPastTheTopOfItsRange {
             current: current.root.checkpoint_txg,
         })?;
-    let current_table = instance_table_chain_of_root(&*devices, &current.root)
+    // 回退读现行那一版与目标那一版按位置条目读单元读到只坏一份都进别的读那份账（D4（校验和位置） 已定项 4「可写挂载期间别的读」）：还住在回退之后那一版里的，
+    // 下一次用户改动发成之后另发的那一批换下，已经不在那一版里的到那一批认不出，照丢。
+    let current_table = allocator
+        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(&*devices, |reader| {
+            instance_table_chain_of_root(reader, &current.root)
+        })
         .map_err(|_unreadable_or_malformed| RollbackError::CurrentInstanceTableMalformed)?
         .records;
-    let target_root = rollback_candidate(
+    let target_root = allocator.reading_and_counting_the_other_unit_reads_with_one_copy_failing(
         &*devices,
-        &system_configuration,
-        &current_table,
-        &ring_slots_known_to_hold_a_root_by(allocator),
-        target,
+        |reader| {
+            rollback_candidate(
+                reader,
+                &system_configuration,
+                &current_table,
+                &ring_slots_known_to_hold_a_root_by(allocator),
+                target,
+            )
+        },
     )?;
     let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
     // R_old 那一版整个从盘上重建（它的账、它的三棵用户可见树、记账树、中央映射树都读回来）：有一个节点读不出就拒。
     // 顶着的记录只给重建填字段，这里只用那一版的用户可见一半与它的账。
-    let target_version = match rebuild_version_in_the_unit_area_starting_at(
-        &*devices,
-        unit_area_start,
-        &target_root,
-        Some(current.record.clone()),
-    ) {
+    let target_version = match allocator
+        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(&*devices, |reader| {
+            rebuild_version_in_the_unit_area_starting_at(
+                reader,
+                unit_area_start,
+                &target_root,
+                Some(current.record.clone()),
+            )
+        }) {
         // 有树而文件集合为空的一版也是候选：候选那一判按树表有没有条目判（带文件读成「那一版写过文件」），回退到它就是回到删空的那一刻。
         Ok(RebuiltVersion::WithFile(version) | RebuiltVersion::WithTreesAndNoFile(version)) => {
             version
@@ -10765,10 +10790,15 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
         }
     };
     // cur 那一版的账有一个节点读不出就拒：从盘上沿它的根读一遍分配记录树（判定与后面的写读的都是内存里那一份账，这一读只判读不读得出）。
-    if let Err(failure) = allocation_records_under_root_in_the_unit_area_starting_at(
+    if let Err(failure) = allocator.reading_and_counting_the_other_unit_reads_with_one_copy_failing(
         &*devices,
-        unit_area_start,
-        &current.root,
+        |reader| {
+            allocation_records_under_root_in_the_unit_area_starting_at(
+                reader,
+                unit_area_start,
+                &current.root,
+            )
+        },
     ) {
         return Err(RollbackError::CurrentAccountUnreadable { failure });
     }
@@ -10843,8 +10873,13 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
             placement_to_release_after_checking_every_device(unit, &locations, allocator)
                 .map_err(not_releasable)?;
         quarantined_user_visible_copies.extend(
-            copies_of_a_released_unit_failing_the_release_checksum_check(
-                unit, &locations, &*devices,
+            allocator.reading_and_counting_the_other_unit_reads_with_one_copy_failing(
+                &*devices,
+                |reader| {
+                    copies_of_a_released_unit_failing_the_release_checksum_check(
+                        unit, &locations, reader,
+                    )
+                },
             ),
         );
         released_user_visible_units.push((unit, placement));
@@ -10867,11 +10902,16 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
         &current_table,
         current.inode_number_watermark(),
         |ring_roots_not_abandoned| {
-            highest_inode_number_watermark_in_the_ring(
-                devices,
-                unit_area_start,
-                ring_roots_not_abandoned,
-                &current.record,
+            allocator.reading_and_counting_the_other_unit_reads_with_one_copy_failing(
+                &*devices,
+                |reader| {
+                    highest_inode_number_watermark_in_the_ring(
+                        reader,
+                        unit_area_start,
+                        ring_roots_not_abandoned,
+                        &current.record,
+                    )
+                },
             )
         },
     );
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 2adffe338d82acfce0a854a9d7b01c3931c3f782..ab533d56ab8576d0e98ec732712af0911a956631 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -79,9 +79,10 @@ use crate::mount::{
 use crate::mounted_read::{
     open_pool_for_read, FileReadFailure, FileReadOutput, OpenFileFailure, OpenPoolForReadFailure,
 };
+use crate::pointer::LocationEntry;
 use crate::recovery::{
     instance_table_chain_of_root, PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing,
-    PoolReaderOfOneReadPass, ReadPassOverThePool,
+    PoolReaderOfOneReadPass, ReadPassOverThePool, RecoveryFailure,
 };
 use crate::region_scan::{
     scan_region_on_the_gpu, scan_region_over_the_identity_index, AllocationInRegion,
@@ -102,7 +103,7 @@ use crate::transaction::{
     publish_snapshot_destruction_step, publish_the_relocations_and_the_repair_of_the_exempt_units,
     publish_truncation_step, publish_unlink_intent, publish_unlink_step, publish_version,
     relocations_of_old_data_units_kept_for_the_snapshots,
-    relocations_of_the_units_read_with_one_copy_failing_while_mounting,
+    relocations_of_the_units_read_with_one_copy_failing,
     repair_of_the_exempt_units_at_these_locations, resend_the_frozen_publish,
     units_at_these_locations, walk_the_deletion_queue_until_it_is_empty,
     CentralMappingOfAVersionReadLeafByLeaf, CompactionIntentStep, CompactionStepRefused,
@@ -222,23 +223,26 @@ pub struct UserChangePublished {
 /// 一次用户改动发成之后，读到只坏一份的单元另发一批换落点（D4（校验和位置） 已定项 4「经已有的搬迁路径把这个单元整个搬到新槽、两份新写，旧落点照释放前核隔离；
 /// 走同一个事务层」；用户 2026-11-04 定有活快照时「另发一批搬迁修好」，另发那批被拒时「推抬 F 再试一遍」）做成了什么。封闭集合。用户改动在每一种结局下都已发成。
 /// 这一批管三处读到的：这次改动读—改—写读回的、改动之后仍被活快照钉着的旧单元；可写挂载读盘建上一版时读到的（挂载记下、挂载之后头一次改动发成时取走）；
-/// 可写挂载期间别的读读到的（[`MountedSession::read_the_file`]、推抬 F 与发 discard 之前沿链读实例表、释放之前读盘核、搬留存单元读源）。
+/// 可写挂载期间别的读读到的（[`MountedSession::read_the_file`]、推抬 F 与发 discard 之前读根环与实例表、整理的回退门、回退、清扫、释放之前读盘核、搬留存单元读源，还有这一批自己认位置时沿链读实例表）。
 /// 豁免映射的三类（映射树节点、树表、实例表片）认不出映射 key，同一批按它们自己的重写路子换落点（`transaction::RepairOfTheExemptUnits`）。
-/// 那一批搬没搬成都只试这一次，结局报在这里，之后的用户改动不再接着搬。
+/// 这一组记下的这一次只试一次，结局报在这里，之后的用户改动不接着搬；同一组之后再被读到就再记、再试一次（账只记等下一次改动的那几组，试过的不留）。
 /// 没有活快照钉着时读回的单元在改动那次发布里就整个重写、旧落点照释放之前读盘核隔离，不另发。
 /// 那一批被空间拒（[`refusal_is_short_of_space`] 那两种）时与会话走空同一套（`publish_the_queue_until_it_is_empty_raising_the_floor_for_space`）：
 /// 推一串抬 F、回收之后再发同一批，预算按 `mount::push_one_floor_raise_within_the_admission_budget` 数，这一批算一次准入，推的数从 0 起。
 #[derive(Debug)]
 pub enum RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing {
     /// 不另发：没有记着读到只坏一份的单元，或记着的那几个在这次改动发成之前已经换了落点（读回的旧单元没有活快照钉着、挂载记下的被这次改动整个重写、
-    /// 树表与映射树根这类每次发布都换下的）。
+    /// 树表与映射树根这类每次发布都换下的），认位置时沿链读实例表也读通、没读到只坏一份。
     NotNeeded,
     /// 另发了一批（`transaction::publish_the_relocations_and_the_repair_of_the_exempt_units`，接在改动发成的那一版后面，被空间拒过的接在推的最后一串后面）：
     /// 计划搬这几把映射 key 的单元，按计划的次序（计划里是这一批本来就重写的节点的，不另搬也在里面）；同一次发布换下这几个豁免映射的单元。
     /// 那一版排在 `versions_published` 末尾，也是会话的 `current`。
+    /// `instance_table_chain_unreadable`：账上还有认不出的组、沿链读实例表读不出时的那个失败，实例表这一批没换（别的照发，D4（校验和位置） 已定项 4 不让一处读不出连带丢掉好 key）；
+    /// 读得出或用不着读是 `None`。
     Relocated {
         mapping_keys: Vec<Vec<u8>>,
         units_exempt_from_the_mapping_replaced: Vec<TransactionUnit>,
+        instance_table_chain_unreadable: Option<RecoveryFailure>,
         floor_raises: FloorRaisesPushedForTheRelocation,
     },
     /// 被空间拒、推满仍放不下（`stop`：预算用完、F 已在上限、抬 F 那一串自己也推不出空间）：会话的 `current` 停在推的最后一串的最后一次空发布上
@@ -249,8 +253,9 @@ pub enum RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing {
         stop: FloorRaiseStop,
         floor_raises: FloorRaisesPushedForTheRelocation,
     },
-    /// 被拒、不是空间不够（计划对不上那一版、发布路径别的错；落盘中途块设备错照发布路径冻结在分配器上，不重试）：会话的 `current` 停在最后一次落盘的
-    /// 那一版上（改动发成的那一版，或推过抬 F 时最后一次空发布）。只供测试的开关 `compaction_plan::RelocationKeptForTheSnapshotsRefusedNotForSpace`
+    /// 被拒、不是空间不够（计划对不上那一版、发布路径别的错；落盘中途块设备错照发布路径冻结在分配器上，不重试；或一样要发的都没有而沿链读实例表读不出，
+    /// `RelocationBatchRefused::InstanceTableOfTheVersionUnreadable`）：会话的 `current` 停在最后一次落盘的
+    /// 那一版上（改动发成的那一版，或推过抬 F 时最后一次空发布）。只供测试的开关 `compaction_plan::RelocationBatchAfterTheChangeRefusedNotForSpace`
     /// 强制进这一支（计划对不上那一形）。
     RefusedNotForSpace {
         refusal: Box<RelocationBatchRefused>,
@@ -258,7 +263,7 @@ pub enum RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing {
     },
     /// 为空间推抬 F 时抬 F 自己报了错（`mount::push_one_floor_raise_within_the_admission_budget` 的 `Err`，原样）：这一串是因为
     /// `refusal_that_pushed_the_failed_raise` 那一次被拒才推的；之前推成的几串在 `floor_raises`。会话的 `current` 停在最后一次落盘的那一版上。
-    /// 只供测试的开关 `compaction_plan::FloorRaiseForTheRelocationKeptForTheSnapshotsFails`（连同被空间拒的那个开关）强制进这一支。
+    /// 只供测试的开关 `compaction_plan::FloorRaiseForTheRelocationBatchAfterTheChangeFails`（连同被空间拒的那个开关）强制进这一支。
     FloorRaiseFailedWhilePushingForSpace {
         cause: Box<MountError>,
         refusal_that_pushed_the_failed_raise: Box<RelocationBatchRefused>,
@@ -2190,14 +2195,16 @@ fn foreground_cannot_write_under_the_switches(allocator: &PoolAllocator) -> bool
 ///
 /// `mapping_keys_read_while_mounting` 是可写挂载读盘建上一版时读到只坏一份的那几个单元（`compaction_plan::CompactionOfThisMount` 记下的映射 key，
 /// 用户 2026-11-05 定「可写挂载时搬，冷恢复只计数」）：其中到 `current` 这一版仍在中央映射里或留存条目里的那几个
-/// （`transaction::relocations_of_the_units_read_with_one_copy_failing_while_mounting`）接在上面那几个后面、进同一批，同一条路径、同一套被拒推抬 F；
+/// （`transaction::relocations_of_the_units_read_with_one_copy_failing`）接在上面那几个后面、进同一批，同一条路径、同一套被拒推抬 F；
 /// 结局报在同一个成员里（[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`]）。挂载那一串里不另发：另发就是挂载的第七段写，挂载分段与二次崩溃、
 /// 取号之前的续做预演都要跟着改，下一次用户改动之后这一处已有搬迁路径与它的崩溃点（判决 `research/prompts/m4-close-code-r3-main-verification.md`
 /// 第三节第 7 条「挑走已有路径改动最小的那一处」）。
 ///
 /// 可写挂载期间别的读读到只坏一份的那几组位置条目（分配器上那份账）按 `current` 这一版认：认出映射 key 的走上面同一条路，
 /// 此刻是豁免映射的单元的同一次发布换下（`transaction::repair_of_the_exempt_units_at_these_locations`），别的（这次改动已经换下的）不管。
-/// 这几组与这一批要搬的源落点上记下的，发完（发成或被拒）都从账上去掉：搬没搬成都只试这一次。这一批自己推抬 F 时新读到的不去掉，归下一次改动。
+/// 认的时候沿链读实例表读到只坏一份的那几片也进账、同一批接着换。实例表链读不出时实例表那一半跳过，别的照发，结局报明实例表没换
+/// （[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated`] 的 `instance_table_chain_unreadable`；一样要发的都没有时报在 `RefusedNotForSpace` 里）。
+/// 这几组与这一批要搬的源落点上记下的，发完（发成或被拒）都从账上去掉：这一组记下的这一次搬没搬成都只试这一次。这一批自己推抬 F 时新读到的不去掉，归下一次改动。
 ///
 /// # Panics
 /// `current` 不是有树的一版：调用方只在一次用户改动发成之后调，发成的都是有树的一版（`publish_user_change` 开头拒了没树的那一形）。
@@ -2237,38 +2244,46 @@ fn relocate_the_units_read_with_one_copy_failing_after_the_change<Device: BlockD
             .cloned()
             .collect();
     let planned_for_the_units_read_while_mounting =
-        relocations_of_the_units_read_with_one_copy_failing_while_mounting(
+        relocations_of_the_units_read_with_one_copy_failing(
             version_after_the_change,
             &mapping_keys_read_while_mounting_and_by_the_other_reads,
             &planned,
         );
     planned.extend(planned_for_the_units_read_while_mounting);
-    let repair = match repair_of_the_exempt_units_at_these_locations(
+    let repair = repair_of_the_exempt_units_at_these_locations(
         &*devices,
         version_after_the_change,
         &units_read_with_one_copy_failing.locations_without_a_mapping_key,
-    ) {
-        Ok(repair) => repair,
-        Err(failure) => {
-            allocator.forget_the_locations_read_with_one_copy_failing(|locations| {
-                locations_read_with_one_copy_failing.contains(locations)
-            });
-            return RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace {
-                refusal: Box::new(RelocationBatchRefused::InstanceTableOfTheVersionUnreadable(
-                    failure,
-                )),
-                floor_raises: FloorRaisesPushedForTheRelocation {
-                    floor_raises: Vec::new(),
-                    refusals_that_pushed_the_floor_raises: Vec::new(),
-                },
-            };
-        }
-    };
+    );
+    allocator.count_the_other_unit_reads_with_one_copy_failing(
+        repair
+            .locations_read_with_one_copy_failing_while_reading_the_chain
+            .clone(),
+    );
+    let locations_tried_by_this_batch: Vec<[LocationEntry; 2]> =
+        locations_read_with_one_copy_failing
+            .iter()
+            .chain(&repair.locations_read_with_one_copy_failing_while_reading_the_chain)
+            .copied()
+            .collect();
     if planned.is_empty() && repair.is_empty() {
         allocator.forget_the_locations_read_with_one_copy_failing(|locations| {
-            locations_read_with_one_copy_failing.contains(locations)
+            locations_tried_by_this_batch.contains(locations)
         });
-        return RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded;
+        return match repair.instance_table_chain_unreadable {
+            None => RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded,
+            Some(failure) => {
+                RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace {
+                    refusal: Box::new(RelocationBatchRefused::InstanceTableOfTheVersionUnreadable(
+                        failure,
+                    )),
+                    floor_raises: FloorRaisesPushedForTheRelocation {
+                        floor_raises: Vec::new(),
+                        refusals_that_pushed_the_floor_raises: Vec::new(),
+                    },
+                }
+            }
+        };
     }
     let mut version_to_relocate_on = version_after_the_change.clone();
     let batch = publish_the_relocation_batch_raising_the_floor_for_space(
@@ -2283,7 +2298,7 @@ fn relocate_the_units_read_with_one_copy_failing_after_the_change<Device: BlockD
         &repair,
     );
     allocator.forget_the_locations_read_with_one_copy_failing(|locations| {
-        locations_read_with_one_copy_failing.contains(locations)
+        locations_tried_by_this_batch.contains(locations)
             || locations.iter().any(|location| {
                 planned
                     .iter()
@@ -2306,6 +2321,7 @@ fn relocate_the_units_read_with_one_copy_failing_after_the_change<Device: BlockD
                     .map(|relocation| relocation.mapping_key)
                     .collect(),
                 units_exempt_from_the_mapping_replaced: repair.units,
+                instance_table_chain_unreadable: repair.instance_table_chain_unreadable,
                 floor_raises,
             }
         }
@@ -2329,11 +2345,11 @@ enum RelocationBatchAfterTheFloorRaises {
 /// （`mount::push_one_floor_raise_within_the_admission_budget`，就地推进 `version`）、回收之后再发同一批——与会话走空被空间拒同一套
 /// （[`publish_the_queue_until_it_is_empty_raising_the_floor_for_space`]）：这一批算一次准入，推的数从 0 起，预算照那个函数数（推成的每一串的空发布数相加）。
 /// 推满仍放不下、抬 F 自己报错、被拒不是空间不够，交回会话要报的那一种；被拒不是空间不够的不再试（落盘途中块设备错照发布路径冻结在分配器上）。
-/// 只供测试的开关（`compaction_plan::RelocationKeptForTheSnapshotsRefusedForSpace`）打开时这一次不建写入口，交回一个空间准入不够的拒绝
+/// 只供测试的开关（`compaction_plan::RelocationBatchAfterTheChangeRefusedForSpace`）打开时这一次不建写入口，交回一个空间准入不够的拒绝
 /// （[`space_refusal_of_the_relocation_forced_for_tests`]），照被空间拒那一支往下走；另两个只供测试的开关同一种进法：
-/// `compaction_plan::RelocationKeptForTheSnapshotsRefusedNotForSpace` 打开时交回一个计划对不上的拒绝
+/// `compaction_plan::RelocationBatchAfterTheChangeRefusedNotForSpace` 打开时交回一个计划对不上的拒绝
 /// （[`refusal_not_for_space_of_the_relocation_forced_for_tests`]），照被拒不是空间不够那一支往下走；
-/// `compaction_plan::FloorRaiseForTheRelocationKeptForTheSnapshotsFails` 打开时为空间推抬 F 那一步不推、交回一个抬 F 的错
+/// `compaction_plan::FloorRaiseForTheRelocationBatchAfterTheChangeFails` 打开时为空间推抬 F 那一步不推、交回一个抬 F 的错
 /// （[`floor_raise_failure_of_the_relocation_forced_for_tests`]），照抬 F 自己报错那一支往下走。
 #[allow(
     clippy::too_many_arguments,
@@ -2359,10 +2375,10 @@ fn publish_the_relocation_batch_raising_the_floor_for_space<Device: BlockDevice>
     // 预算在 `mount::PUBLISHES_PER_ADMISSION_AT_MOST` − 1 次空发布之内停）⇒ 至多 8 轮。跨轮携带的是推到的那一版、已推的数与推成的几串。
     loop {
         let refusal_of_the_relocation = if compaction
-            .take_a_forced_space_refusal_of_the_relocation_kept_for_the_snapshots()
+            .take_a_forced_space_refusal_of_the_relocation_batch_after_the_change()
         {
             space_refusal_of_the_relocation_forced_for_tests()
-        } else if compaction.is_the_relocation_kept_for_the_snapshots_forced_refused_not_for_space()
+        } else if compaction.is_the_relocation_batch_after_the_change_forced_refused_not_for_space()
         {
             refusal_not_for_space_of_the_relocation_forced_for_tests(planned)
         } else {
@@ -2393,7 +2409,7 @@ fn publish_the_relocation_batch_raising_the_floor_for_space<Device: BlockDevice>
             );
         }
         let floor_raise = if compaction
-            .is_the_floor_raise_for_the_relocation_kept_for_the_snapshots_forced_to_fail()
+            .is_the_floor_raise_for_the_relocation_batch_after_the_change_forced_to_fail()
         {
             Err(floor_raise_failure_of_the_relocation_forced_for_tests())
         } else {
@@ -2462,7 +2478,7 @@ fn publish_the_relocation_batch_once<Device: BlockDevice>(
     )
 }
 
-/// 只供测试的开关造的那一拒（`compaction_plan::RelocationKeptForTheSnapshotsRefusedForSpace`）：空间准入不够那一种，一块不够的盘都不点名——
+/// 只供测试的开关造的那一拒（`compaction_plan::RelocationBatchAfterTheChangeRefusedForSpace`）：空间准入不够那一种，一块不够的盘都不点名——
 /// 没有真算准入，只为把另发那一批强制进「被空间拒」那一支；在这一批的写入口建起之前交回，一个写都没发。
 fn space_refusal_of_the_relocation_forced_for_tests() -> RelocationBatchRefused {
     RelocationBatchRefused::Publish(PublishError::SpaceAdmissionRefused(
@@ -2472,7 +2488,7 @@ fn space_refusal_of_the_relocation_forced_for_tests() -> RelocationBatchRefused
     ))
 }
 
-/// 只供测试的开关造的那一拒（`compaction_plan::RelocationKeptForTheSnapshotsRefusedNotForSpace`）：计划对不上那一版那一种，点名计划里头一把 key——
+/// 只供测试的开关造的那一拒（`compaction_plan::RelocationBatchAfterTheChangeRefusedNotForSpace`）：计划对不上那一版那一种，点名计划里头一把 key——
 /// 没有真核计划，只为把另发那一批强制进「被拒、不是空间不够」那一支；在这一批的写入口建起之前交回，一个写都没发。
 /// 这一批只换豁免映射的单元、计划是空的时点名的 key 是空的。
 fn refusal_not_for_space_of_the_relocation_forced_for_tests(
@@ -2486,7 +2502,7 @@ fn refusal_not_for_space_of_the_relocation_forced_for_tests(
     }
 }
 
-/// 只供测试的开关造的那一错（`compaction_plan::FloorRaiseForTheRelocationKeptForTheSnapshotsFails`）：现行那一版的实例表读不出——
+/// 只供测试的开关造的那一错（`compaction_plan::FloorRaiseForTheRelocationBatchAfterTheChangeFails`）：现行那一版的实例表读不出——
 /// [`push_one_floor_raise_within_the_admission_budget`] 在任何写之前交得出的一种；没有真读盘，只为把另发那一批强制进「抬 F 自己报错」那一支，一个写都没发。
 fn floor_raise_failure_of_the_relocation_forced_for_tests() -> MountError {
     MountError::InstanceTableUnreadable
@@ -2766,7 +2782,7 @@ impl MountedSession {
     }
 
     /// 这次可写挂载里别的读读到只坏一份的次数（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬，包括……可写挂载期间别的读」）：[`Self::read_the_file`]、
-    /// 推抬 F 与发 discard 之前沿链读实例表、释放之前读盘核、搬留存单元读源。一组位置条目等着下一次用户改动的这一程里只算一次，处置过之后再读到再算。
+    /// 推抬 F 与发 discard 之前读根环与实例表、整理的回退门、回退、清扫、释放之前读盘核、搬留存单元读源、另发那一批认位置沿链读实例表（挂载这一串读到的那几组也在等下一次用户改动，再读不进这个数）。一组位置条目等着下一次用户改动的这一程里只算一次，处置过之后再读到再算。
     /// 与 [`Self::unit_reads_with_one_copy_failing_while_mounting`]、[`Self::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`] 按读的地方分开报。
     /// 只在内存里，不落盘；新挂载从 0 起。
     #[must_use]
@@ -3298,18 +3314,18 @@ impl MountedSession {
         if is_judged_as_the_product_does
             && compaction.take_the_reading_of_the_publishes_of_the_mount_from_the_ring()
         {
-            let publishes_read_from_the_ring = rollback_states_of_the_ring(
-                &PoolReaderOfOneReadPass::reading_for_the_first_time(devices),
-                parameters,
-                current.root(),
-            )
-            .map_or(
-                publish_budget_of_a_compaction_run(
-                    parameters.geometry.root_ring_slots_per_region.count(),
-                    0,
-                ),
-                |states| states.publishes_after_the_newest_state,
-            );
+            let publishes_read_from_the_ring = allocator
+                .reading_and_counting_the_other_unit_reads_with_one_copy_failing(
+                    &PoolReaderOfOneReadPass::reading_for_the_first_time(devices),
+                    |reader| rollback_states_of_the_ring(reader, parameters, current.root()),
+                )
+                .map_or(
+                    publish_budget_of_a_compaction_run(
+                        parameters.geometry.root_ring_slots_per_region.count(),
+                        0,
+                    ),
+                    |states| states.publishes_after_the_newest_state,
+                );
             compaction.publishes_since_the_last_user_change = compaction
                 .publishes_since_the_last_user_change
                 .max(publishes_read_from_the_ring);
@@ -4193,16 +4209,16 @@ pub(crate) fn run_compaction_on<Device: BlockDevice>(
 /// 回退门（D16（发布语义） 已定项 1「回退候选集保留最近 4 个可退到的不同状态」，D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：
 /// 下一次发布写的那一槽与它之后留的余量（[`crate::compaction_plan::root_ring_slots_left_by_a_compaction_publish`]）里的根一走，根环里不同用户状态就少于 4 个而且比此刻少，这次整理发布就不发。
 /// 此刻本来就不到 4 个、走的那几条又都不是某个状态的最后一条的，不是这次发布挤掉的，照发。读不通不发（[`CompactionRunStop::RollbackStatesOfTheRingUnreadable`]）。
-/// 只读，读根环与每条有效根的树表。
+/// 只读，读根环与每条有效根的树表；按位置条目读单元读到只坏一份的进别的读那份账（D4（校验和位置） 已定项 4「可写挂载期间别的读」）。
 fn stop_of_the_rollback_gate<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &[(DeviceIdentity, Device)],
+    allocator: &PoolAllocator,
     current: &PoolVersion,
 ) -> Option<CompactionRunStop> {
-    match rollback_states_of_the_ring(
+    match allocator.reading_and_counting_the_other_unit_reads_with_one_copy_failing(
         &PoolReaderOfOneReadPass::reading_for_the_first_time(devices),
-        parameters,
-        current.root(),
+        |reader| rollback_states_of_the_ring(reader, parameters, current.root()),
     ) {
         Err(_unreadable) => Some(CompactionRunStop::RollbackStatesOfTheRingUnreadable),
         Ok(states) => (states.states_after_the_next_publish_and_the_margin
@@ -4471,7 +4487,8 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
             }
         };
         if is_the_rollback_gate_judged && !is_this_publish_the_exception {
-            if let Some(stop) = stop_of_the_rollback_gate(parameters, &*devices, current) {
+            if let Some(stop) = stop_of_the_rollback_gate(parameters, &*devices, allocator, current)
+            {
                 return finish(
                     compaction,
                     current.root().checkpoint_txg,
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 78e95e2b3b57c1319addd38e96afc7199c3cb55b..0cced883a7a88dd51142a089df35e16b8e0afaf5 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -3634,6 +3634,10 @@ pub fn rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
         mapping_tree,
     };
     let mapped_units = mapped_units_of_the_version(&trees_read);
+    central_mapping_keys_are_those_of_the_mapped_units_and_the_entries_kept_for_the_snapshots(
+        &mapping_entries,
+        &mapped_units,
+    )?;
     let allocation_record_tree = trees_read.allocation_tree_read.version();
     let files_other_than_the_first = files_other_than_the_first_of_the_version(&trees_read);
     let data_pointers = trees_read.first_file_extents.data_pointers.clone();
@@ -6814,6 +6818,11 @@ fn accounting_entry_count_is_three_plus_six_per_device(
     Ok(())
 }
 
+/// [`central_mapping_keys_are_those_of_the_mapped_units_and_the_entries_kept_for_the_snapshots`] 报的那一拒的不变量号与说明。
+pub const CENTRAL_MAPPING_KEY_SET_INVARIANT: &str = "I-1.11";
+pub const CENTRAL_MAPPING_KEY_SET_DISAGREES_WITH_THE_MAPPED_UNITS: &str =
+    "中央映射里的 key 集合不等于这一版进映射的单元的 key 加留存条目的 key（一条映射 key 被换了）";
+
 /// 进映射的单元各有几个（[`mapping_entry_count_is_one_per_mapped_unit`] 的输入）。
 struct MappedUnitCounts {
     extent_tree_nodes: usize,
@@ -6855,6 +6864,41 @@ fn mapping_entry_count_is_one_per_mapped_unit(
     Ok(())
 }
 
+/// 建上一版那一判（I-1.11（映射 key 与单元头相符） 产品一侧；D4（校验和位置） 已定项 4 那一格的用户定案「映射 key 被换的盘挂载时拒」）：中央映射里的 key 集合
+/// 等于这一版进映射的单元的 key 加留存条目的 key。[`mapping_entry_count_is_one_per_mapped_unit`] 只数条数，一条 key 换了、封得对的盘数得过；
+/// 不拒的话这一版照「查不到就留提示」收下，提示在单元搬走之后还指隔离的旧落点，下一次发布把这把 key 盖回那里，搬到的新槽没人引用。只比内存里读过的，不多读盘。
+/// 只读路径（冷走读、`read_at`）不判：换 key 不动位置条目与单元字节，查不到报 [`RecoveryFailure::MappingMiss`]，读不出别的单元的字节。
+///
+/// # Errors
+/// 两个集合不等 ⇒ [`RecoveryFailure::InvariantViolated`]。
+fn central_mapping_keys_are_those_of_the_mapped_units_and_the_entries_kept_for_the_snapshots(
+    mapping_entries: &MappingEntriesOfTheVersion,
+    mapped_units: &[(TransactionUnit, Vec<u8>)],
+) -> Result<(), RecoveryFailure> {
+    let keys_expected: BTreeSet<&[u8]> = mapped_units
+        .iter()
+        .map(|(_, key)| key.as_slice())
+        .chain(
+            mapping_entries
+                .mapping_entries_kept_for_the_snapshots
+                .iter()
+                .map(|(key, _)| key.as_slice()),
+        )
+        .collect();
+    let keys_in_the_mapping: BTreeSet<&[u8]> = mapping_entries
+        .mapping_position_of_key
+        .keys()
+        .map(Vec::as_slice)
+        .collect();
+    if keys_expected != keys_in_the_mapping {
+        return Err(RecoveryFailure::InvariantViolated {
+            invariant: CENTRAL_MAPPING_KEY_SET_INVARIANT,
+            detail: CENTRAL_MAPPING_KEY_SET_DISAGREES_WITH_THE_MAPPED_UNITS,
+        });
+    }
+    Ok(())
+}
+
 /// deadlist 树叶里的条目逐条解开（43 字节，`records::DeadlistEntry`），次序照叶里的（key 升序）。
 ///
 /// # Errors
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index adcb1fd9b80c073e3303b546311774b9fba7e39e..6fb51daf47f5d8152fa997f5508cef8aacb6afba 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -374,6 +374,8 @@ pub struct PoolWriter<'pool, Device: BlockDevice> {
     /// 把这次发布之后仍被活快照钉着（换下来进 deadlist、映射条目留存）的那几个搬到新槽（用户 2026-11-04 定「另发一批搬迁修好」，
     /// [`relocations_of_old_data_units_kept_for_the_snapshots`]）。只在内存里；新开的写入口从空起。
     mapping_keys_of_old_data_units_read_back_with_one_copy_failing: RefCell<Vec<Vec<u8>>>,
+    /// 这个写入口上最近一次分配记录树重写集的固定点走了几轮（[`PoolWriter::rounds_of_the_last_allocation_record_tree_settling`]）。只在内存里、不落盘；新开的写入口从 0 起。
+    rounds_of_the_last_allocation_record_tree_settling: Cell<usize>,
 }
 
 /// 一个写入口上读新槽残留的账（[`PoolWriter::residue_reads`]）：发了几次、读不出几次。
@@ -413,11 +415,20 @@ impl<'pool, Device: BlockDevice> PoolWriter<'pool, Device> {
             residue_reads: Cell::new(ResidueReadsOfTheWriter::NONE),
             old_data_unit_reads_back_with_one_copy_failing: Cell::new(0),
             mapping_keys_of_old_data_units_read_back_with_one_copy_failing: RefCell::new(Vec::new()),
+            rounds_of_the_last_allocation_record_tree_settling: Cell::new(0),
         }
     }
 }
 
 impl<Device: BlockDevice> PoolWriter<'_, Device> {
+    /// 这个写入口上最近一次定下来的分配记录树重写集固定点走了几轮（[`settle_the_allocation_record_tree`]、
+    /// [`settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file`]）；还没有定下来过是 0，轮数用完的那一次走错误成员，不记在这里。
+    #[must_use]
+    pub fn rounds_of_the_last_allocation_record_tree_settling(&self) -> usize {
+        self.rounds_of_the_last_allocation_record_tree_settling
+            .get()
+    }
+
     /// 这个写入口上读新槽残留发了几次、读不出几次（判决 m4-partial-write-r1 第二节 R2「残留读出错记数、不拒写」）。
     #[must_use]
     pub fn residue_reads(&self) -> ResidueReadsOfTheWriter {
@@ -1887,6 +1898,8 @@ struct SettledRowPublishOnAVersionWithoutFile {
     allocation_record_tree: AllocationRecordTreePlan,
     rewritten_roles: Vec<TransactionUnit>,
     release: Vec<Placement>,
+    /// 固定点走了几轮（定下来的那一轮算在内）。
+    rounds_walked: usize,
 }
 
 /// 树表 0 条那一版写行时，分配记录树重写哪几个节点（固定点，同带文件那一路的 [`settle_the_allocation_record_tree`]）：上一版的树是分配器记着的
@@ -1928,10 +1941,11 @@ fn settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(
     let chain = instance_table_chain_to_release(&instance_table.replaced_chain, allocator)?;
     let instance_table_roles =
         instance_table_page_roles_in_bump_order(instance_table.pages_after_this_publish());
+    let previous_version_of_the_tree = previous_tree.map(|tree| &tree.version);
     let mut rewritten: BTreeSet<AllocationRecordTreeNode> = BTreeSet::new();
     let mut nodes_after: BTreeSet<AllocationRecordTreeNode> = previous_nodes.clone();
     // 迭代上界与跨轮携带的同 `settle_the_allocation_record_tree`。
-    for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {
+    for round_index in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {
         let rewritten_among_the_nodes_after: BTreeSet<AllocationRecordTreeNode> =
             rewritten.intersection(&nodes_after).copied().collect();
         let allocation_plan = crate::allocation_record_tree::plan_the_tree_after_this_publish(
@@ -1970,7 +1984,12 @@ fn settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(
         let changed_by_the_release =
             nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
         if !changed_by_the_release.is_subset(&rewritten_by_the_plan) {
-            rewritten.extend(changed_by_the_release);
+            merge_into_the_rewrite_set_walking_the_release_chain(
+                &mut rewritten,
+                changed_by_the_release,
+                previous_version_of_the_tree,
+                &geometry,
+            );
             nodes_after.extend(nodes_holding_records(&geometry, rehearsal.records()));
             continue;
         }
@@ -1985,9 +2004,15 @@ fn settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(
                 allocation_record_tree: allocation_plan,
                 rewritten_roles,
                 release,
+                rounds_walked: round_index + 1,
             });
         }
-        rewritten.extend(changed);
+        merge_into_the_rewrite_set_walking_the_release_chain(
+            &mut rewritten,
+            changed,
+            previous_version_of_the_tree,
+            &geometry,
+        );
         nodes_after = holding_records;
     }
     Err(PublishError::AllocationRecordTreeRewriteSetDidNotSettle {
@@ -2102,6 +2127,8 @@ pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(
         txg,
         allocator,
     )?;
+    pool.rounds_of_the_last_allocation_record_tree_settling
+        .set(settled.rounds_walked);
     // 这次发布切出来的记录条数在动分配器之前判（代码审阅第 26 条）：点名的就是这次重写的角色（下面 `named_roles`）。
     refuse_a_publish_of_more_journal_records_than_the_limit(
         roles_named_by_each_record_of_the_publish(
@@ -11900,15 +11927,19 @@ pub fn units_at_these_locations(
 }
 
 /// 另发的那一批要换落点的豁免映射的单元（D4（校验和位置） 已定项 4 的三类：映射树自己的节点、树表、实例表片；[`repair_of_the_exempt_units_at_these_locations`]）：
-/// 它们认不出映射 key、搬迁入口不收（[`RelocationBatchRefused::PinnedUnitInThePlan`]），换落点照它们自己的重写路子走：那一次发布本来就重写树表与映射树根；
+/// 它们认不出映射 key、搬迁入口不收（[`RelocationBatchRefused::PinnedUnitInThePlan`]），换落点照它们自己的重写路子走：树表与映射树根每次发布都换，走不到这里；
 /// 映射树根之下的节点经 `mapping_keys_routed_through_the_nodes` 把从根到叶那条路整条重写（[`plan_rewriting_the_paths_to_the_moved_entries`]，条目的位置不变）；
 /// 实例表整条链 COW 重写、行不变（D18（块里携带什么信息） 已定项 11「每次写行 COW 重写整条链」，没有不被改写的片）。
 #[derive(Clone, Debug)]
 pub struct RepairOfTheExemptUnits {
-    /// 这一批换下的那几个豁免映射的角色，按映射树节点、树表、实例表片的次序。
+    /// 这一批换下的那几个豁免映射的角色，按映射树节点、实例表片的次序。
     pub units: Vec<TransactionUnit>,
     mapping_keys_routed_through_the_nodes: Vec<Vec<u8>>,
     instance_table_rewrite: Option<InstanceTableRewrite>,
+    /// 认这几组时沿链读实例表读到只坏一份的那几片的位置条目（一片一组）：读到的片同一批接着换，调用方记进别的读那份账。
+    pub locations_read_with_one_copy_failing_while_reading_the_chain: Vec<[LocationEntry; 2]>,
+    /// 认这几组时实例表链读不出、解不开的那个失败：实例表那一半跳过（读不出也写不了行不变的重写），映射树节点与要搬的单元照发，结局报明实例表没换。
+    pub instance_table_chain_unreadable: Option<RecoveryFailure>,
 }
 
 impl RepairOfTheExemptUnits {
@@ -11920,16 +11951,16 @@ impl RepairOfTheExemptUnits {
 }
 
 /// `locations_without_a_mapping_key`（[`UnitsAtTheseLocations`]）里此刻是 `version` 那一版豁免映射的单元的那几组要怎么换落点。
-/// 映射树节点与树表只比内存里这一版的指针；余下的有一组才沿链读实例表（只坏一份的读才有这几组，没有坏副本的发布不多读）。
-/// 没有单元住着的那几组（这次改动已经换下的）不进结果。
-///
-/// # Errors
-/// 还有没认出的那几组而实例表链读不出、解不开：认不出它们是不是实例表片。
+/// 映射树节点只比内存里这一版的指针；余下的有一组才沿链读实例表（只坏一份的读才有这几组，没有坏副本的发布不多读）。这一读经数只坏一份的那一层：
+/// 读到只坏一份的片与账上记着的片一样，整条链同一批重写（D4（校验和位置） 已定项 4「可写挂载期间别的读」）。链读不出、解不开时认不出余下那几组是不是实例表片，
+/// 实例表那一半跳过，失败交在 [`RepairOfTheExemptUnits::instance_table_chain_unreadable`] 里。
+/// 没有单元住着的那几组（这次改动已经换下的，树表与映射树根每次发布都换，在这里头）不进结果。
+#[must_use]
 pub fn repair_of_the_exempt_units_at_these_locations<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     version: &TransactionOutput,
     locations_without_a_mapping_key: &[[LocationEntry; 2]],
-) -> Result<RepairOfTheExemptUnits, RecoveryFailure> {
+) -> RepairOfTheExemptUnits {
     let mut units: Vec<TransactionUnit> = Vec::new();
     let mut mapping_keys_routed_through_the_nodes: Vec<Vec<u8>> = Vec::new();
     let mut locations_left: Vec<[LocationEntry; 2]> = Vec::new();
@@ -11950,24 +11981,40 @@ pub fn repair_of_the_exempt_units_at_these_locations<Reader: PoolReader + ?Sized
             );
             mapping_keys_routed_through_the_nodes
                 .push(a_mapping_key_below_the_node(mapping_tree, node));
-        } else if version.root.tree_table.locations == *locations {
-            units.push(TransactionUnit::TreeTable);
         } else {
             locations_left.push(*locations);
         }
     }
     if locations_left.is_empty() {
-        return Ok(RepairOfTheExemptUnits {
+        return RepairOfTheExemptUnits {
             units,
             mapping_keys_routed_through_the_nodes,
             instance_table_rewrite: None,
-        });
+            locations_read_with_one_copy_failing_while_reading_the_chain: Vec::new(),
+            instance_table_chain_unreadable: None,
+        };
     }
-    let chain = instance_table_chain_of_root(reader, &version.root)?;
-    let any_page_read_with_one_copy_failing = chain
-        .page_pointers
-        .iter()
-        .any(|page_pointer| locations_left.contains(&page_pointer.locations));
+    let counting_reader = PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(reader);
+    let chain_read = instance_table_chain_of_root(&counting_reader, &version.root);
+    let locations_read_with_one_copy_failing_while_reading_the_chain =
+        counting_reader.locations_of_the_units_read_with_one_copy_failing();
+    let chain = match chain_read {
+        Ok(chain) => chain,
+        Err(failure) => {
+            return RepairOfTheExemptUnits {
+                units,
+                mapping_keys_routed_through_the_nodes,
+                instance_table_rewrite: None,
+                locations_read_with_one_copy_failing_while_reading_the_chain,
+                instance_table_chain_unreadable: Some(failure),
+            }
+        }
+    };
+    let any_page_read_with_one_copy_failing = chain.page_pointers.iter().any(|page_pointer| {
+        locations_left.contains(&page_pointer.locations)
+            || locations_read_with_one_copy_failing_while_reading_the_chain
+                .contains(&page_pointer.locations)
+    });
     let instance_table_rewrite = if any_page_read_with_one_copy_failing {
         units.extend((0..chain.page_pointers.len()).map(|position| {
             TransactionUnit::of_instance_table_page(InstanceTablePageIndex(
@@ -11981,11 +12028,13 @@ pub fn repair_of_the_exempt_units_at_these_locations<Reader: PoolReader + ?Sized
     } else {
         None
     };
-    Ok(RepairOfTheExemptUnits {
+    RepairOfTheExemptUnits {
         units,
         mapping_keys_routed_through_the_nodes,
         instance_table_rewrite,
-    })
+        locations_read_with_one_copy_failing_while_reading_the_chain,
+        instance_table_chain_unreadable: None,
+    }
 }
 
 /// `node` 下面的一把映射 key（沿头一个孩子走到叶、取叶里头一把）：把它当「条目换了位置」交给发布，从根到它那片叶的路整条重写，`node` 在路上。
@@ -12113,14 +12162,13 @@ fn copies_of_the_exempt_units_read_with_one_copy_failing<Reader: PoolReader + ?S
     quarantined
 }
 
-/// 可写挂载读盘建上一版时读到只坏一份的那几个单元（`mapping_keys`，[`mapping_keys_of_the_units_at_these_locations`] 认出来的）里，到 `version`
-/// 这一版仍在中央映射里或留存条目里的那几个，照已有的搬迁路径另发一批时的计划（用户 2026-11-05 定「可写挂载时搬」；与
+/// 读到只坏一份的那几个单元（`mapping_keys`：可写挂载读盘建上一版读到的，与可写挂载期间别的读读到的，都是 [`units_at_these_locations`] 认出来的）里，到 `version`
+/// 这一版仍在中央映射里或留存条目里的那几个，照已有的搬迁路径另发一批时的计划（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」；与
 /// [`relocations_of_old_data_units_kept_for_the_snapshots`] 同一批发，`planned_before` 是那一份、已在里面的 key 不重复）：源落点取映射里那两条
 /// 位置条目（或留存条目记的那两条）的槽，跨度按 key 的类标签归到的用户可见角色取（[`user_visible_role_standing_for_a_unit_class`]）。
 /// 到 `version` 这一版已经不在的（这次改动或挂载之后的发布把它整个重写了、旧落点照释放之前读盘核那一路走）不进计划。
-/// 会话读文件、推抬 F、释放之前读盘核、搬留存单元读源记下的那几个（[`units_at_these_locations`] 认出的 key）同一条路。
 #[must_use]
-pub fn relocations_of_the_units_read_with_one_copy_failing_while_mounting(
+pub fn relocations_of_the_units_read_with_one_copy_failing(
     version: &TransactionOutput,
     mapping_keys: &[Vec<u8>],
     planned_before: &[PlannedRelocation],
@@ -13216,6 +13264,8 @@ fn judge_the_publish_before_any_read_or_write<Device: BlockDevice>(
         mapping_keys_whose_entries_change_without_a_relocation,
         pool.snapshot_lifetime_judgement(),
     )?;
+    pool.rounds_of_the_last_allocation_record_tree_settling
+        .set(settled.rounds_walked);
     // 这次发布切出来的记录条数（代码审阅第 26 条）：角色定下来就数得出，在准入、读盘核与动分配器之前判。
     let journal_records_of_the_publish = records_of_a_publish_naming_the_relocated_copies(
         &settled.resolved.rewritten_roles,
@@ -13319,57 +13369,31 @@ struct PublishJudgedBeforeAnyReadOrWrite {
     journal_records_of_the_publish: usize,
 }
 
-/// 同 [`prepare_the_version_publish`]，另带这次要搬的单元（整理的一批，[`publish_relocation_batch`]；别的发布给空的）：
-/// 固定点与真发都按「释放（换下的与搬走的源落点）→ 取目的地 → 取这次重写的角色的落点」走同一串，源落点照释放判定路径逐盘核、
-/// 读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）；这次重写要换下的角色不另搬（它的新落点本来就不在活 R 里）。
-#[allow(
-    clippy::too_many_arguments,
-    reason = "参数各是一样东西：写入口、分配器、计划、上一版、树号、读不读盘核、卸载记号、删除这一步、这次要搬的单元、不搬而换了位置的映射 key、\
-              读回旧单元改好的新净荷"
-)]
-fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
-    pool: &PoolWriter<'_, Device>,
+/// 这次发布释放之前读盘核的全部读（[`prepare_the_version_publish_relocating_units`]）：换下的角色、记着只坏一份的豁免映射的单元、删快照这一批放的条目、搬走的源落点，
+/// 交回要隔离的那几份。经 `reader` 读，调用方在它返回之后（`Err` 也是）取数到的只坏一份。
+///
+/// # Errors
+/// 映射条目的位置项指池外的盘、两条指同一块盘；删快照这一批放的条目认不出角色；[`copies_failing_the_release_checksum_check`] 那几种。都在动分配器的账与任何写之前。
+fn copies_quarantined_by_the_release_checks<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
     allocator: &mut PoolAllocator,
     plan: &PublishPlan<'_>,
     previous: Option<&TransactionOutput>,
-    trees: FileVersionTreeIdentifiers,
+    settled: &SettledPublish,
+    device_identities_of_the_accounting_rows: &[DeviceIdentity],
     release_checksum_check: ReleaseChecksumCheck,
-    unmount_marker: UnmountMarker,
-    deletion_and_lineage: &DeletionAndLineageOfThePublish,
-    relocations: &[RelocationOfThePublish],
-    mapping_keys_whose_entries_change_without_a_relocation: &[Vec<u8>],
-    payloads_of_the_rewritten_data_units: &[RewrittenDataUnitPayload],
-) -> Result<(PublishWrites, TransactionOutput), PublishError> {
-    let PublishJudgedBeforeAnyReadOrWrite {
-        settled,
-        device_identities_of_the_accounting_rows,
-        journal_records_of_the_publish: _,
-    } = judge_the_publish_before_any_read_or_write(
-        pool,
-        allocator,
-        plan,
-        previous,
-        &trees,
-        deletion_and_lineage,
-        relocations,
-        mapping_keys_whose_entries_change_without_a_relocation,
-    )?;
-    // 经映射核到的那几个单元，释放之前再按位置项读盘核一次校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）：只读，
-    // 读不到就在动分配器、发任何一个写之前返回。第一个文件版本换下的 mkfs 那片树表与树表 0 条那一版的分配记录树节点不经映射，不核。
+) -> Result<Vec<CopyQuarantinedAfterReleaseChecksumMismatch>, PublishError> {
     let kept_for_the_snapshots = &settled
         .resolved
         .deadlist
         .user_visible_roles_kept_for_the_snapshots;
-    // 释放之前读盘核读到只坏一份的进这次挂载别的读那一份账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）。
-    let devices_counting_unit_reads_with_one_copy_failing =
-        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*pool.devices);
     let mut quarantine = match (previous, release_checksum_check) {
         (Some(previous_version), ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt) => {
             let mut copies_failing = copies_failing_the_release_checksum_check(
                 previous_version,
                 &settled.previous_roles_replaced,
                 kept_for_the_snapshots,
-                &devices_counting_unit_reads_with_one_copy_failing,
+                reader,
                 FileContentOfThePublish::of_the_plan(plan),
             )?;
             copies_failing.extend(copies_of_the_exempt_units_read_with_one_copy_failing(
@@ -13377,7 +13401,7 @@ fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
                 &settled.previous_roles_replaced,
                 &plan.instance_table,
                 &allocator.locations_read_with_one_copy_failing_waiting_for_the_next_user_change(),
-                &devices_counting_unit_reads_with_one_copy_failing,
+                reader,
             ));
             copies_failing
         }
@@ -13387,7 +13411,7 @@ fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
                 previous_version,
                 &settled.previous_roles_replaced,
                 kept_for_the_snapshots,
-                &device_identities_of_the_accounting_rows,
+                device_identities_of_the_accounting_rows,
                 FileContentOfThePublish::of_the_plan(plan),
             )?;
             Vec::new()
@@ -13408,16 +13432,14 @@ fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
         refuse_locations_that_do_not_name_two_pool_devices(
             role,
             locations,
-            &device_identities_of_the_accounting_rows,
+            device_identities_of_the_accounting_rows,
         )
         .map_err(not_releasable)?;
         match release_checksum_check {
             ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt => {
                 quarantine.extend(
                     copies_of_a_released_unit_failing_the_release_checksum_check(
-                        role,
-                        locations,
-                        &devices_counting_unit_reads_with_one_copy_failing,
+                        role, locations, reader,
                     ),
                 );
             }
@@ -13430,7 +13452,7 @@ fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
         refuse_locations_that_do_not_name_two_pool_devices(
             relocation.role_for_the_checks,
             &relocation.source_locations,
-            &device_identities_of_the_accounting_rows,
+            device_identities_of_the_accounting_rows,
         )?;
         match release_checksum_check {
             ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt => {
@@ -13456,7 +13478,7 @@ fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
                         copies_of_a_released_unit_failing_the_release_checksum_check(
                             relocation.role_for_the_checks,
                             &relocation.source_locations,
-                            &devices_counting_unit_reads_with_one_copy_failing,
+                            reader,
                         ),
                     );
                 }
@@ -13464,10 +13486,64 @@ fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
             ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition => {}
         }
     }
+    Ok(quarantine)
+}
+
+/// 同 [`prepare_the_version_publish`]，另带这次要搬的单元（整理的一批，[`publish_relocation_batch`]；别的发布给空的）：
+/// 固定点与真发都按「释放（换下的与搬走的源落点）→ 取目的地 → 取这次重写的角色的落点」走同一串，源落点照释放判定路径逐盘核、
+/// 读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）；这次重写要换下的角色不另搬（它的新落点本来就不在活 R 里）。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：写入口、分配器、计划、上一版、树号、读不读盘核、卸载记号、删除这一步、这次要搬的单元、不搬而换了位置的映射 key、\
+              读回旧单元改好的新净荷"
+)]
+fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
+    pool: &PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    plan: &PublishPlan<'_>,
+    previous: Option<&TransactionOutput>,
+    trees: FileVersionTreeIdentifiers,
+    release_checksum_check: ReleaseChecksumCheck,
+    unmount_marker: UnmountMarker,
+    deletion_and_lineage: &DeletionAndLineageOfThePublish,
+    relocations: &[RelocationOfThePublish],
+    mapping_keys_whose_entries_change_without_a_relocation: &[Vec<u8>],
+    payloads_of_the_rewritten_data_units: &[RewrittenDataUnitPayload],
+) -> Result<(PublishWrites, TransactionOutput), PublishError> {
+    let PublishJudgedBeforeAnyReadOrWrite {
+        settled,
+        device_identities_of_the_accounting_rows,
+        journal_records_of_the_publish: _,
+    } = judge_the_publish_before_any_read_or_write(
+        pool,
+        allocator,
+        plan,
+        previous,
+        &trees,
+        deletion_and_lineage,
+        relocations,
+        mapping_keys_whose_entries_change_without_a_relocation,
+    )?;
+    // 经映射核到的那几个单元，释放之前再按位置项读盘核一次校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）：只读，
+    // 读不到就在动分配器、发任何一个写之前返回。第一个文件版本换下的 mkfs 那片树表与树表 0 条那一版的分配记录树节点不经映射，不核。
+    // 释放之前读盘核读到只坏一份的进这次挂载别的读那一份账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）。
+    let devices_counting_unit_reads_with_one_copy_failing =
+        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*pool.devices);
+    let quarantine_read = copies_quarantined_by_the_release_checks(
+        &devices_counting_unit_reads_with_one_copy_failing,
+        allocator,
+        plan,
+        previous,
+        &settled,
+        &device_identities_of_the_accounting_rows,
+        release_checksum_check,
+    );
+    // 核不过提前返回的那一次，返回之前读到的只坏一份同样进账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）。
     allocator.count_the_other_unit_reads_with_one_copy_failing(
         devices_counting_unit_reads_with_one_copy_failing
             .locations_of_the_units_read_with_one_copy_failing(),
     );
+    let quarantine = quarantine_read?;
     publish_admitted(
         pool,
         allocator,
@@ -13556,6 +13632,8 @@ struct SettledPublish {
     release: Vec<Placement>,
     /// 这次真搬的那几个（要搬的里面去掉这次重写本来就换下的角色），按计划的次序。
     relocations_in_effect: Vec<RelocationOfThePublish>,
+    /// 固定点走了几轮（定下来的那一轮算在内）。
+    rounds_walked: usize,
 }
 
 /// 这次发布换下的上一版角色与要释放的落点（释放判定路径，只查不改，D19（块指针的结构与宽度预算） 已定项 5 第 1 条）：
@@ -13684,14 +13762,17 @@ fn role_standing_for_the_unit_of_a_deadlist_entry(
 /// 这次写的每个单元、连分配记录树自己的节点，每盘各记一条；换下的每一份把它那条改成已释放（D3（空间分配） 已定项 7；这就是释放链）。
 /// 所以走一个固定点：从「重写集空、节点照上一版」起，按它算这次发布的全部角色，在分配器的**拷贝**上把释放与取落点照发布路径的次序
 /// 走一遍，按走出来的记录算有记录的节点与内容变了的节点（[`nodes_whose_contents_changed`]）；节点对得上、变了的都在重写集里（或本来就是新节点）
-/// 就定下，否则节点换成走出来的那一批、变了的并进重写集再走一遍。多罩的节点内容没变、照样重写一份（合法的 COW）。
+/// 就定下，否则节点换成走出来的那一批、变了的并进重写集、顺着释放链走到闭包（[`merge_into_the_rewrite_set_walking_the_release_chain`]）再走一遍。
+/// 多罩的节点内容没变、照样重写一份（合法的 COW）。
 /// 真发时走的是同一串释放与取落点，得出同一份记录（释放之前读盘核出对不上的那几份记录留在已分配，只会让变了的节点更少）。
 /// 没有上一版（第一个文件版本）时这棵树的树号换成这次发出来的（树表 0 条那一版的分配记录树是树 0），节点全部重写、不照抄。
 ///
 /// **走得完**：产品路径上释放只改写记录、不在这次发布里空出槽（回收要等根记下之后），取落点时分配记录树的节点之后只剩单槽的提交内生块、
 /// 按同一串槽位往后取——多一个角色只把这一串拉长，有记录的节点只多不少，重写集也只增不减，都以池里的节点数为上界。
-/// 只供测试的复用窗口置 0 开关（`ReuseWindow::ForcedToZero`）让释放当场空出槽、回落可能取回它们，那一格的单调性推不出来：
-/// 给 [`ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`] 轮，走不完交回 `AllocationRecordTreeRewriteSetDidNotSettle`（条款没写这一格怎么办）。
+/// 节点数可以比 [`ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`] 大，这个上界不保证轮数够：释放链沿活跃叶连成一长串时，一轮只走一跳就要走满轮数，
+/// 所以每次并进重写集都一次走到闭包，释放链不占轮数，轮数只剩取落点碰到新叶那一半。这一半的上界没有推出来，E206（发布固定点轮数的代价）全部取样点量到至多 5 轮。
+/// 只供测试的复用窗口置 0 开关（`ReuseWindow::ForcedToZero`）让释放当场空出槽、回落可能取回它们，那一格的单调性也推不出来。
+/// 走满 [`ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`] 轮走不完，交回 `AllocationRecordTreeRewriteSetDidNotSettle`（条款没写这一格怎么办）。
 ///
 /// # Errors
 /// 同 [`PublishPlan::resolve`]、[`placements_released_by_this_publish`]，与拷贝上取不到落点（`PlacementRefused`）——
@@ -13733,10 +13814,11 @@ fn settle_the_allocation_record_tree(
         ),
         None => (BTreeSet::new(), &[]),
     };
+    let previous_tree = previous.map(|previous_version| &previous_version.allocation_record_tree);
     let mut rewritten: BTreeSet<AllocationRecordTreeNode> = BTreeSet::new();
     let mut nodes_after: BTreeSet<AllocationRecordTreeNode> = previous_nodes.clone();
-    // 迭代上界是 ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT 轮；跨轮携带的是重写集（只增不减）与这一版的节点（换成上一轮走出来的那一批）。
-    for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {
+    // 迭代上界是 ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT 轮；跨轮携带的是重写集（只增不减，每轮结束时对释放链是闭的）与这一版的节点（换成上一轮走出来的那一批）。
+    for round_index in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {
         let rewritten_among_the_nodes_after: BTreeSet<AllocationRecordTreeNode> =
             rewritten.intersection(&nodes_after).copied().collect();
         let allocation_plan = crate::allocation_record_tree::plan_the_tree_after_this_publish(
@@ -13792,7 +13874,12 @@ fn settle_the_allocation_record_tree(
         let changed_by_the_release =
             nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
         if !changed_by_the_release.is_subset(&rewritten_by_the_plan) {
-            rewritten.extend(changed_by_the_release);
+            merge_into_the_rewrite_set_walking_the_release_chain(
+                &mut rewritten,
+                changed_by_the_release,
+                previous_tree,
+                &geometry,
+            );
             nodes_after.extend(nodes_holding_records(&geometry, rehearsal.records()));
             continue;
         }
@@ -13811,9 +13898,15 @@ fn settle_the_allocation_record_tree(
                 previous_roles_replaced,
                 release,
                 relocations_in_effect,
+                rounds_walked: round_index + 1,
             });
         }
-        rewritten.extend(changed);
+        merge_into_the_rewrite_set_walking_the_release_chain(
+            &mut rewritten,
+            changed,
+            previous_tree,
+            &geometry,
+        );
         nodes_after = holding_records;
     }
     Err(PublishError::AllocationRecordTreeRewriteSetDidNotSettle {
@@ -13821,9 +13914,42 @@ fn settle_the_allocation_record_tree(
     })
 }
 
-/// 分配记录树重写集的固定点最多走几轮（[`settle_the_allocation_record_tree`]）。产品路径上三四轮就定（第一轮重写集是空的、
-/// 释放弄脏的叶在取落点之前先并进来；下一轮罩住这次写的那几片叶与它们的祖先、再一轮罩住释放链弄脏的叶）；
-/// 只供测试的复用窗口置 0 开关下单调性推不出来，这个数给它一个边。
+/// 把一轮弄脏的节点并进分配记录树的重写集，再顺着释放链走到闭包：重写集里的节点这一版要重写，重写就释放它在上一版的两份落点，
+/// 释放改写落点那条记录，记录所在的叶（由槽号定）与叶的祖先内容就变了、也要重写；新并进来的节点接着走同一步，直到不再多。
+/// 走到的节点固定点下一轮的释放一定弄脏，它们本来就在不动点里：提前并进来只省轮数，写的角色与字节不变。
+/// 重写集从空集起、只经这里并节点，进来时对这一步是闭的，所以只从新并进来的节点走。上一版没有的节点没有旧落点，走到它为止；没有上一版的不走。
+fn merge_into_the_rewrite_set_walking_the_release_chain(
+    rewritten: &mut BTreeSet<AllocationRecordTreeNode>,
+    dirty_nodes: BTreeSet<AllocationRecordTreeNode>,
+    previous_tree: Option<&AllocationRecordTreeVersion>,
+    geometry: &AllocationRecordTreeGeometry,
+) {
+    let mut nodes_whose_old_locations_are_not_walked: Vec<AllocationRecordTreeNode> = dirty_nodes
+        .into_iter()
+        .filter(|node| rewritten.insert(*node))
+        .collect();
+    let Some(previous_tree) = previous_tree else {
+        return;
+    };
+    // 迭代上界是树的节点数：一个节点只在头一次并进重写集时进待走的那一叠；祖先那一层循环的上界是树高。没有提前出口。
+    while let Some(node) = nodes_whose_old_locations_are_not_walked.pop() {
+        let Some(pointer) = previous_tree.pointer_of(node) else {
+            continue;
+        };
+        for location in pointer.locations {
+            let mut leaf_or_ancestor = Some(geometry.leaf_of_slot(location.device, location.slot));
+            while let Some(node_on_the_way_to_the_root) = leaf_or_ancestor {
+                if rewritten.insert(node_on_the_way_to_the_root) {
+                    nodes_whose_old_locations_are_not_walked.push(node_on_the_way_to_the_root);
+                }
+                leaf_or_ancestor = geometry.parent_of(node_on_the_way_to_the_root);
+            }
+        }
+    }
+}
+
+/// 分配记录树重写集的固定点最多走几轮（[`settle_the_allocation_record_tree`]）。释放链每次并进重写集都走到闭包，产品路径上轮数只剩取落点碰到新叶那一半，
+/// E206（发布固定点轮数的代价）全部取样点量到至多 5 轮；这一半与只供测试的复用窗口置 0 开关下的单调性都推不出上界，这个数给它们一个边。
 pub const ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT: usize = 64;
 
 /// 一次发布切成几条 journal 记录、每条点名哪几个角色（D23（journal 的角色与格式） 已定项 17，C491（多条记录时共享内生块在哪条点名没定）
```
