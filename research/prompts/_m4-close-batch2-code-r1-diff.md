# m4-close-batch2-code-r1：这一批产品代码的改动（合并补丁里 crates/singlefs-core/src 的 5 份，2026-11-10）

来源：`~/.local/share/singlefs/work/m4-close-combined/patch/crates.patch`（八份补丁合成一份，「没打这批补丁时 → 此刻主工作区」），只抄产品代码那几份；测试代码不在射程里。

## crates/singlefs-core/src/allocator.rs

```diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 8e933c3..ec52bfd 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -1864,8 +1864,9 @@ impl ReclaimFloorWhenTheRootWasRecorded {
     }
 }
 
-/// 这次可写挂载里建上一版与读—改—写读回之外的读读到只坏一份的账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」），每一处读与不接的依据见
-/// [`PoolAllocator::reading_and_counting_the_other_unit_reads_with_one_copy_failing`]。
+/// 这次可写挂载里建上一版与读—改—写读回之外的读读到只坏一份的账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」），记账的读在
+/// [`PoolAllocator::reading_and_counting_the_other_unit_reads_with_one_copy_failing`] 的文档串里逐类列。读的是现行那一版的单元，下一次用户改动之后那一批换落点；
+/// 读的是别的版本的单元（根环里别的根的账、回退目标、被抛弃根）照样计数，那一批在现行那一版里认不出它们，不搬。
 /// 挂载这一串读到的那几组也在这份账上等下一次用户改动（[`PoolAllocator::keep_the_locations_read_with_one_copy_failing_uncounted`]），只是不进这个数：
 /// 挂载读到的一份坏副本，数据单元与映射树叶一样，挂载那个数计一次，之后再读、或另发那一批释放之前读盘核，这个数都不加。
 #[derive(Debug, Default)]
@@ -1983,8 +1984,12 @@ impl PoolAllocator {
     }
 
     /// 经数只坏一份的那一层做一次读（`read`），读完（读成与否）把数到的那几组记进账（[`Self::count_the_other_unit_reads_with_one_copy_failing`]）。
-    /// 可写挂载期间经 `reader` 按两条位置条目读单元的读都走它或同形的写法：推抬 F 与发 discard 之前读根环与实例表、整理的回退门、回退读现行那一版与目标那一版、
-    /// 回退的释放之前读盘核、清扫的判与发。账各份分配器拷贝共用（读是真发到盘上的），所以 `&self` 够了。
+    /// 走它的读：整理的回退门与后台入口读根环、准入算上限、取号之前取树表 0 条那一版的单元清单、会话改动入口的落后支、回退读现行那一版与目标那一版、回退的释放之前读盘核、清扫的判与发。
+    /// 读完另经 [`Self::count_the_other_unit_reads_with_one_copy_failing`] 进账、不走这一层的读：抬 F 读实例表与重算影子账、发 discard 之前读实例表、
+    /// 重建分配器读最新那条根的实例表与各根的账（`mount::rebuilt_allocator`）、可写挂载取号之前逐盘核的落后支（`mount::devices_passing_the_check_before_the_acquisition`）、释放之前读盘核、搬留存单元读源、另发那一批认位置沿链读实例表、经会话读文件、写系统配置槽之前算 F 生效值
+    /// （经写入口转交，`transaction::PoolWriter::count_the_reads_before_the_system_configuration_writes`）。
+    /// 建上一版那一读与读—改—写读回各有自己的数，不进这份账的次数。没接这份账的是可写挂载之外的读：只读挂载、冷恢复、离线修系统配置。
+    /// 账各份分配器拷贝共用（读是真发到盘上的），所以 `&self` 够了。
     pub(crate) fn reading_and_counting_the_other_unit_reads_with_one_copy_failing<
         'reader,
         Reader: PoolReader + ?Sized,
```

## crates/singlefs-core/src/compaction_plan.rs

```diff
diff --git a/crates/singlefs-core/src/compaction_plan.rs b/crates/singlefs-core/src/compaction_plan.rs
index a19fecc..9409052 100644
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -606,7 +606,8 @@ pub enum CompactionRunStop {
     },
     /// 这次运行的发布预算用完了。
     PublishBudgetSpent { budget: u64 },
-    /// 一批被空间拒（准入或落点）：暂停——撤掉 R 排他、意图留着，不推抬 F。
+    /// 一批被空间拒（准入或落点）：暂停——单独发一次只删意图的发布丢掉意图（R 排他跟着撤），不推抬 F；下一次整理从头签发、重扫。
+    /// 只删意图的那一次归删除类，不判式子、落点照判（`transaction::judge_the_publish_before_any_read_or_write`）。
     PausedShortOfSpace,
     /// 签发那次发布被空间拒（准入或落点）：意图没签成、盘上没有它，R 排他跟着撤；不推抬 F。
     IssueRefusedShortOfSpace,
@@ -627,15 +628,17 @@ pub enum CompactionRunStop {
         bytes_written_in_the_window: u64,
         limit_in_bytes_per_second: u64,
     },
-    /// 这一批算着出了错（D26 已定项 13 第 4 条）：这一批放弃、一个写都没发，意图留在盘上；整理这个后台任务停下或当场解除停下见
-    /// `task_after_the_error`。
+    /// 这一批算着出了错（D26 已定项 13 第 4 条）：这一批放弃、一个单元都没搬；现行那一版里有活意图就单发一次只删意图的发布丢掉它
+    /// （R 排他跟着撤，同 [`Self::PausedShortOfSpace`]），没有就一个写都没发。整理这个后台任务停下或当场解除停下见 `task_after_the_error`；
+    /// 下一次整理从头签发。
     BatchAbandonedAfterABackgroundComputeError {
         task_after_the_error: BackgroundTaskAfterTheComputeError,
     },
     /// 整理这个后台任务先前因一批出错停着（D26 已定项 13 第 4 条；挂载那一刻或切换生效的那一批开头，实现照交给它的那一组一张卡都用不上而当场停着的同算），
     /// 而这一刻池子每块盘的已分配占它的容量都不超过补跑阈值：
-    /// 停在这一刻，从这一刻起一个写都不发、撤掉活意图的 R 排他。一趟开头就判出的，这一趟一个写都没发；一趟之中在 CPU 上补跑过几批之后、
-    /// 或一趟之中切换生效的那一批开头才判出的，前面那几批照常发过。等占用超过阈值（下一趟照常补跑），或一次通过校验的切换请求解除停下。
+    /// 停在这一刻：现行那一版里有活意图就单发一次只删意图的发布丢掉它（R 排他跟着撤，同 [`Self::PausedShortOfSpace`]），之后一个写都不发；
+    /// 没有活意图就从这一刻起一个写都不发（删除续做也不走）。一趟之中在 CPU 上补跑过几批之后、或一趟之中切换生效的那一批开头才判出的，
+    /// 前面那几批照常发过。等占用超过阈值（下一趟照常补跑、从头签发），或一次通过校验的切换请求解除停下。
     BackgroundTaskStoppedAfterAnEarlierComputeError,
     /// GPU 路径上 R 里除了这一轮两份核判得不一样的记录，再没有判定了可搬的（D26 已定项 13 第 8 条「这一批不回收、不搬，留到下一轮重算」）：
     /// 这一批不发，意图照留（不删也不丢），下一批重扫照判。
@@ -754,7 +757,8 @@ impl CompactionRunStop {
     }
 }
 
-/// 一条活意图在这次挂载里的样子：意图本身与它的 R 排没排他（暂停 = 撤掉 R 排他、意图留着）。
+/// 一条活意图在这次挂载里的样子：意图本身与它的 R 排没排他（产品值排他；只供测试的开关 [`CompactionRegionExclusion::SwitchedOffForTests`] 不排）。
+/// 整理停下（暂停、一批出错停下）删掉意图连同排他，不留「意图在、R 不排他」那一态。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct LiveCompactionIntent {
     pub intent: CompactionIntent,
@@ -839,24 +843,6 @@ impl CompactionSwitchesAndTheExcludedRegion {
         self.live_intent.take().map(|live| live.intent)
     }
 
-    /// 暂停：撤掉 R 排他、意图留着。下一次运行接着推它时重新排他。
-    pub fn pause_the_live_intent(&mut self) {
-        if let Some(live) = self.live_intent.as_mut() {
-            live.is_the_region_excluded_from_placement = false;
-        }
-    }
-
-    /// 接着推一条暂停过的意图：照开关重新排他。
-    pub fn resume_the_live_intent(&mut self) {
-        let is_excluded = match self.switches.region_exclusion {
-            CompactionRegionExclusion::ExcludedWhileTheIntentIsLive => true,
-            CompactionRegionExclusion::SwitchedOffForTests => false,
-        };
-        if let Some(live) = self.live_intent.as_mut() {
-            live.is_the_region_excluded_from_placement = is_excluded;
-        }
-    }
-
     /// 分配器此刻不许当目的地的 R（活意图且排他时）。
     #[must_use]
     pub fn region_excluded_from_placement(&self) -> Option<CompactionRegion> {
```

## crates/singlefs-core/src/mount.rs

```diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 026c814..f672fa6 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -784,7 +784,8 @@ pub struct DeviceBehindTheCurrentVersion {
 /// （[`what_a_device_behind_the_version_lacks`]；C585（会话发布放进发布途中取下的旧快照），用户定改法 C）。按次序判：先看单元，单元都在再看根。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum VersionLackingOnADeviceBehindIt {
-    /// 那一版有单元在这块盘上它的落点读不出、或字节与验得过的那一份不同：按角色与落点列出，次序同那一版的单元清单。
+    /// 那一版有单元在这块盘上它的落点读不出、或字节与验得过的那一份不同，落点被 tail 之后的记录点名或那个窗口取不到
+    /// （[`units_of_the_version_missing_on_a_device_behind_it`]）：按角色与落点列出，次序同那一版的单元清单。
     UnitsMissing(Vec<(TransactionUnit, SlotNumber)>),
     /// 单元都在，而那一版的根写在这块盘上、那个根环槽读一次重读一次都不是它：一次发布途中（单元与记录已写、根槽 FUA 还没写）
     /// 取下的这块盘的旧快照就是这个样子。单元都在、根槽里也是它的（根落盘之后、轮换之前崩；只剩一槽自证、那一槽停在上一次轮换）不在这里。
@@ -845,8 +846,8 @@ pub struct DeviceWithoutTheSelectedVersion {
 pub enum SelectedVersionLackingOnDevice {
     /// 停在旧状态：这块盘自证过的系统配置里世代号最大的那一份，(实例代号, journal tail) 落后于所选那一版那次发布的末条记录的 jsn
     /// （每块盘在每次发布之后轮换的系统配置写的就是这一对，`transaction::CommitStep::RotateSystemConfigurationSlots`），
-    /// 而且所选那一版有单元在这块盘上它的落点读不出、或者字节与验得过的那一份不同（`units_missing`，按角色与落点列出，
-    /// 次序同所选那一版的单元清单）。只落后、单元都在的不算：发布的根落盘之后、系统配置轮换之前崩了，
+    /// 而且所选那一版有单元在这块盘上它的落点读不出、或者字节与验得过的那一份不同，落点被 tail 之后的记录点名或那个窗口取不到
+    /// （`units_missing`，按角色与落点列出，次序同所选那一版的单元清单；[`units_of_the_version_missing_on_a_device_behind_it`]）。只落后、单元都在的不算：发布的根落盘之后、系统配置轮换之前崩了，
     /// 或取号失败回卷时写回了取号那一刻的见证值（`transaction::acquire_instance`，C554 乙-配置续），盘都是这个样子。
     BehindTheSelectedVersionAndMissingItsUnits {
         newest_system_configuration_journal_tail: JournalSequenceNumber,
@@ -1311,13 +1312,9 @@ impl PreviousVersion {
     /// # Errors
     /// 树表 0 条那一版的实例表、树表或分配记录树这一次读不出、解不开（重建时读得出，两次读之间的瞬时读错）⇒ `Recovery`：
     /// 核不了就不放行，在任何写之前返回。
-    #[allow(
-        clippy::ptr_arg,
-        reason = "allocation_records_of_version_without_file 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
-    )]
-    fn units<Device: BlockDevice>(
+    fn units<Reader: PoolReader>(
         &self,
-        devices: &Vec<(DeviceIdentity, Device)>,
+        reader: &Reader,
         unit_area_start: UnitAreaStart,
     ) -> Result<Cow<'_, [PublishedUnit]>, MountError> {
         match self {
@@ -1333,7 +1330,7 @@ impl PreviousVersion {
                         slot: root.instance_table.locations[0].slot,
                         identity: TransactionUnit::InstanceTable,
                         bytes: Arc::from(read_unit_via_locations(
-                            devices,
+                            reader,
                             &root.instance_table.locations,
                             data_unit_bytes,
                         )?),
@@ -1342,7 +1339,7 @@ impl PreviousVersion {
                         slot: root.tree_table.locations[0].slot,
                         identity: TransactionUnit::TreeTable,
                         bytes: Arc::from(read_unit_via_locations(
-                            devices,
+                            reader,
                             &root.tree_table.locations,
                             node_bytes,
                         )?),
@@ -1350,7 +1347,7 @@ impl PreviousVersion {
                 ];
                 if let Some(tree) =
                     allocation_records_of_version_without_file_in_the_unit_area_starting_at(
-                        devices,
+                        reader,
                         unit_area_start,
                         root,
                     )?
@@ -1360,7 +1357,7 @@ impl PreviousVersion {
                             slot: pointer.locations[0].slot,
                             identity: role_of_allocation_record_tree_node(*node),
                             bytes: Arc::from(read_unit_via_locations(
-                                devices,
+                                reader,
                                 &pointer.locations,
                                 node_bytes,
                             )?),
@@ -1736,12 +1733,11 @@ impl PlacementsOfAbandonedRoots<'_> {
 /// 与读出来的被抛弃根同一种豁免（候选集里的根引用的、当前账里还分配着的不隔离），不计进 `abandoned_roots_unreadable`（它们的账一个字都没读）；
 /// 可写挂载那一遍给空。
 #[allow(
-    clippy::ptr_arg,
     clippy::too_many_arguments,
-    reason = "ptr_arg：allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、分配器、根表、判被抛弃的谓词、下界 F、当前账、水位之上没施加的记录点名的落点、被抛弃根的落点从哪来、重读前的快照、根槽这一遍读坏的被抛弃根记着的落点"
+    reason = "每个参数各是一样东西：读者、单元区起点、分配器、根表、判被抛弃的谓词、下界 F、当前账、水位之上没施加的记录点名的落点、被抛弃根的落点从哪来、重读前的快照、根槽这一遍读坏的被抛弃根记着的落点"
 )]
-fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+fn isolate_slots_referenced_only_by_abandoned_roots<Reader: PoolReader>(
+    devices: &Reader,
     unit_area_start: UnitAreaStart,
     allocator: &mut PoolAllocator,
     roots: &[RootRecord],
@@ -1946,12 +1942,8 @@ fn slots_of_a_placement_to_isolate_now(
 ///
 /// # Errors
 /// 账与映射重读一次仍都不成、现行账里又没有那道凭据 ⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`（在任何写之前）。
-#[allow(
-    clippy::ptr_arg,
-    reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
-)]
-fn placements_of_an_abandoned_root_whose_account_is_unreadable<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+fn placements_of_an_abandoned_root_whose_account_is_unreadable<Reader: PoolReader>(
+    devices: &Reader,
     unit_area_start: UnitAreaStart,
     root: &RootRecord,
     current_records: &[AllocationRecord],
@@ -1980,14 +1972,10 @@ fn placements_of_an_abandoned_root_whose_account_is_unreadable<Device: BlockDevi
 /// 与可写挂载的 C 读一次没施加的发布（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）共用这一份、交 `None` 时各报各的拒因：
 /// 从中央映射现算；现算也不成就重读一次（重读之前调钩子），账或现算有一样成就用它；仍都不成时，现行账里有那条根之后的记录罩住它的树表槽
 /// （与映射根槽）就当它什么都不引用（[`the_current_account_records_later_units_over_the_account_slots_of`]），没有就交 `None`。
-#[allow(
-    clippy::ptr_arg,
-    reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
-)]
 fn placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread<
-    Device: BlockDevice,
+    Reader: PoolReader,
 >(
-    devices: &Vec<(DeviceIdentity, Device)>,
+    devices: &Reader,
     unit_area_start: UnitAreaStart,
     root: &RootRecord,
     current_records: &[AllocationRecord],
@@ -2054,12 +2042,8 @@ fn the_current_account_records_later_units_over_the_account_slots_of(
 /// （已定项 15），末条读不出时前几条的新根段照样说得出那次发布的树表与映射根在哪。落点不在那块盘的单元区里的（坏盘上的记录）不交。
 /// 只读内存里的记录、不读盘；读盘只问盘的字节数（落点在不在单元区）。那几次发布的账里的落点另由
 /// [`placements_in_the_accounts_of_unapplied_publishes`] 读盘交出，调用方（`rebuilt_allocator`）并进同一份。
-#[allow(
-    clippy::ptr_arg,
-    reason = "placement_lies_in_the_unit_area_of_its_device 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
-)]
-fn placements_named_by_unapplied_journal_records<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+fn placements_named_by_unapplied_journal_records<Reader: PoolReader>(
+    devices: &Reader,
     unit_area_start: UnitAreaStart,
     records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
     selected_version: &RootRecord,
@@ -2135,12 +2119,11 @@ fn placements_named_by_unapplied_journal_records<Device: BlockDevice>(
 /// `NewerStateStillUnreadableAfterOneReread(AccountOfAnUnappliedPublishWithIncompleteReadableRecords)`
 /// （点名那次发布的 (实例代号, checkpoint_txg)），在任何写之前。
 #[allow(
-    clippy::ptr_arg,
     clippy::too_many_arguments,
-    reason = "ptr_arg：placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、读阶段扫出的记录、所选那一版、每区根环槽数、现行账、重读前的钩子"
+    reason = "每个参数各是一样东西：读者、单元区起点、读阶段扫出的记录、所选那一版、每区根环槽数、现行账、重读前的钩子"
 )]
-fn placements_in_the_accounts_of_unapplied_publishes<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+fn placements_in_the_accounts_of_unapplied_publishes<Reader: PoolReader>(
+    devices: &Reader,
     unit_area_start: UnitAreaStart,
     records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
     selected_version: &RootRecord,
@@ -2218,12 +2201,11 @@ fn placements_in_the_accounts_of_unapplied_publishes<Device: BlockDevice>(
 /// # Errors
 /// 同 [`placements_in_the_accounts_of_unapplied_publishes`]（点名那条根的 (实例代号, checkpoint_txg)）。
 #[allow(
-    clippy::ptr_arg,
     clippy::too_many_arguments,
-    reason = "ptr_arg：placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、本池单元头里的 fsid、根环里读出的根、判被抛弃的谓词、读阶段扫出的记录、所选那一版、每区根环槽数、现行账、重读前的钩子"
+    reason = "每个参数各是一样东西：读者、单元区起点、本池单元头里的 fsid、根环里读出的根、判被抛弃的谓词、读阶段扫出的记录、所选那一版、每区根环槽数、现行账、重读前的钩子"
 )]
-fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+fn placements_in_the_accounts_of_newer_readable_roots<Reader: PoolReader>(
+    devices: &Reader,
     unit_area_start: UnitAreaStart,
     filesystem_identifier_in_unit_headers: u64,
     roots: &[RootRecord],
@@ -2351,12 +2333,8 @@ fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
 /// 全零的空指针没有槽可读，不算证据。只读盘、不写盘。
 /// 只要一份不够（攻方 `research/prompts/c834-r3-opus-output.md` 第二节 T2：盖掉一份、另一份这一次读回全零时那条根还走得完）。
 /// 别的池的单元头不算（用户定「比 fsid」，零轮）：它的实例代号是那个池的号，不说明本池有过比 `root` 更新的实例写过这两个槽。
-#[allow(
-    clippy::ptr_arg,
-    reason = "PoolReader 实现在 Vec<(DeviceIdentity, Device)> 上，切片不能转成 &dyn PoolReader"
-)]
-fn one_pointer_has_both_copies_overwritten_by_a_later_instance<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+fn one_pointer_has_both_copies_overwritten_by_a_later_instance<Reader: PoolReader>(
+    devices: &Reader,
     root: &RootRecord,
     filesystem_identifier_in_unit_headers: u64,
 ) -> bool {
@@ -2378,12 +2356,8 @@ fn one_pointer_has_both_copies_overwritten_by_a_later_instance<Device: BlockDevi
 /// `location` 那一槽起读出的自证过的本池单元头（头校验和与载荷校验和都过，头里的 fsid 与 `filesystem_identifier_in_unit_headers` 相同）
 /// 里写序的实例代号：先按索引节点读 16 KiB，解不开再读 32 KiB、依次按数据单元与打包记录单元解；读不出（越过盘尾、没有那块盘）、
 /// 三类都解不开或解得开而 fsid 不是本池的交 `None`。只读盘、不写盘。
-#[allow(
-    clippy::ptr_arg,
-    reason = "PoolReader 实现在 Vec<(DeviceIdentity, Device)> 上，切片不能转成 &dyn PoolReader"
-)]
-fn instance_in_the_self_verified_unit_header_at<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+fn instance_in_the_self_verified_unit_header_at<Reader: PoolReader>(
+    devices: &Reader,
     location: &LocationEntry,
     filesystem_identifier_in_unit_headers: u64,
 ) -> Option<InstanceGeneration> {
@@ -2489,12 +2463,8 @@ fn with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
 ///   宽护那一截与别的 C 落点同一个时机解开（这个进程写掉这条根那一槽时）。
 ///
 /// 只读盘，不改分配器。
-#[allow(
-    clippy::ptr_arg,
-    reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
-)]
-fn placements_a_rollback_publish_revived_released_in_the_current_account<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+fn placements_a_rollback_publish_revived_released_in_the_current_account<Reader: PoolReader>(
+    devices: &Reader,
     unit_area_start: UnitAreaStart,
     newer_root: &RootRecord,
     readable_records_of_its_publish: &[&crate::journal::JournalRecord],
@@ -2567,12 +2537,8 @@ enum PackedUnitNamedByAPublishAsRead {
 }
 
 /// 一次发布读得出的记录点名的每个打包单元（类标签 3），按记录与点名的次序，经位置条目里的校验和读、解。只读盘。
-#[allow(
-    clippy::ptr_arg,
-    reason = "read_unit_via_locations 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
-)]
-fn packed_units_named_by_the_records_of_a_publish_as_read<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+fn packed_units_named_by_the_records_of_a_publish_as_read<Reader: PoolReader>(
+    devices: &Reader,
     readable_records_of_the_publish: &[&crate::journal::JournalRecord],
 ) -> Vec<PackedUnitNamedByAPublishAsRead> {
     let packed_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
@@ -2668,12 +2634,8 @@ fn placements_still_allocated_in_the_account(
 /// 映射现算不成、重读一次仍不成、凭据也没有 ⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnUnappliedPublishWithIncompleteReadableRecords)`，
 /// `unapplied_publish` 点名 `root_of_the_publish` 的 (实例代号, checkpoint_txg)；与 C393 那一拒分开报（用户 2026-10-08 定「乙：core 另起错误成员」），
 /// 在任何写之前。
-#[allow(
-    clippy::ptr_arg,
-    reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
-)]
-fn placements_of_an_unapplied_publish_whose_account_is_unreadable<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+fn placements_of_an_unapplied_publish_whose_account_is_unreadable<Reader: PoolReader>(
+    devices: &Reader,
     unit_area_start: UnitAreaStart,
     root_of_the_publish: &RootRecord,
     current_records: &[AllocationRecord],
@@ -3127,10 +3089,14 @@ fn rebuilt_allocator<Device: BlockDevice>(
 ) -> Result<RebuiltAllocator, MountError> {
     let unit_area_start = unit_area_start_of_the_chosen_system_configuration(system_configuration);
     let mut allocator = allocator_of_the_previous_version(devices, unit_area_start, previous)?;
+    // 读最新那条根的实例表与读各根的账（候选根、被抛弃根、没施加的发布、较新可读根、回退目标）读到只坏一份的进这次挂载别的读那一份账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）：
+    // 同一个单元建上一版已经读到的归挂载那个数，不再算；读的是别的版本的，下一次用户改动之后那一批认不出、只留下计数。
+    let devices_counting_unit_reads_with_one_copy_failing =
+        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(devices);
     let current_records = allocator.records().to_vec();
     let (newest_table, instance_table_of_the_newest_root) =
         instance_table_of_the_newest_root_read_at_most_twice(
-            devices,
+            &devices_counting_unit_reads_with_one_copy_failing,
             system_configuration,
             before_the_one_reread,
         )?;
@@ -3178,7 +3144,7 @@ fn rebuilt_allocator<Device: BlockDevice>(
         );
     let placements_named_by_unapplied_journal_records = match shadow_ledger {
         ShadowLedger::On => placements_of_unapplied_publishes_and_newer_readable_roots(
-            devices,
+            &devices_counting_unit_reads_with_one_copy_failing,
             system_configuration,
             unit_area_start,
             records,
@@ -3192,7 +3158,7 @@ fn rebuilt_allocator<Device: BlockDevice>(
     };
     let shadow_ledger_computed = match shadow_ledger {
         ShadowLedger::On => isolate_slots_referenced_only_by_abandoned_roots(
-            devices,
+            &devices_counting_unit_reads_with_one_copy_failing,
             unit_area_start,
             &mut allocator,
             &sorted.roots,
@@ -3213,6 +3179,15 @@ fn rebuilt_allocator<Device: BlockDevice>(
             placements_referenced_by_abandoned_root: BTreeMap::new(),
         },
     };
+    let units_read_while_building_the_previous_version =
+        previous.units_read_with_one_copy_failing_while_building_on_it();
+    allocator.count_the_other_unit_reads_with_one_copy_failing(
+        devices_counting_unit_reads_with_one_copy_failing
+            .locations_of_the_units_read_with_one_copy_failing()
+            .into_iter()
+            .filter(|locations| !units_read_while_building_the_previous_version.contains(locations))
+            .collect(),
+    );
     let ShadowLedgerComputed {
         abandoned_roots_unreadable,
         placements_referenced_by_abandoned_root,
@@ -3587,16 +3562,12 @@ fn roots_of_the_rebuild_pass_with_the_root_chosen_by_the_read_stage(
 ///
 /// # Errors
 /// 那两份账读不出、从中央映射也现算不成、重读一次仍不成（同那两个函数）。
-#[allow(
-    clippy::ptr_arg,
-    reason = "那几份落点走 &Vec 那一份读者：Vec 实现了 PoolReader、切片不能转成 dyn"
-)]
 #[allow(
     clippy::too_many_arguments,
-    reason = "参数各是一样东西：设备表、系统配置、单元区起点、读阶段扫出的记录、所选那一版、读出来的根、判被抛弃、现行账、重读前的钩子"
+    reason = "参数各是一样东西：读者、系统配置、单元区起点、读阶段扫出的记录、所选那一版、读出来的根、判被抛弃、现行账、重读前的钩子"
 )]
-fn placements_of_unapplied_publishes_and_newer_readable_roots<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+fn placements_of_unapplied_publishes_and_newer_readable_roots<Reader: PoolReader>(
+    devices: &Reader,
     system_configuration: &crate::system_configuration::SystemConfiguration,
     unit_area_start: UnitAreaStart,
     records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
@@ -5520,9 +5491,13 @@ fn raise_the_floor_through<Device: BlockDevice>(
                 .collect()
         })
         .unwrap_or_default();
+    // 重算影子账读每条候选根与表里没记着的被抛弃根的账，现行那一版也在候选里（读的是它的分配记录树、树表）：读到只坏一份的进别的读那一份账
+    // （D4（校验和位置） 已定项 4「可写挂载期间别的读」）。下面拒了分配器换回去，账不退（账各份拷贝共用）。
+    let devices_counting_the_reads_of_the_accounts =
+        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*devices);
     let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {
         match isolate_slots_referenced_only_by_abandoned_roots(
-            devices,
+            &devices_counting_the_reads_of_the_accounts,
             unit_area_start_of_the_chosen_system_configuration(&system_configuration),
             allocator,
             &roots,
@@ -5539,12 +5514,20 @@ fn raise_the_floor_through<Device: BlockDevice>(
             Ok(computed) => computed.abandoned_roots_unreadable,
             Err(error) => {
                 *allocator = allocator_before_the_raise;
+                allocator.count_the_other_unit_reads_with_one_copy_failing(
+                    devices_counting_the_reads_of_the_accounts
+                        .locations_of_the_units_read_with_one_copy_failing(),
+                );
                 return Err(error);
             }
         }
     } else {
         0
     };
+    allocator.count_the_other_unit_reads_with_one_copy_failing(
+        devices_counting_the_reads_of_the_accounts
+            .locations_of_the_units_read_with_one_copy_failing(),
+    );
     // 卸载那一格当场放开的那一截（`RaiseFloorEntry::reuse_of_the_reclaimed_slots`，GA）：下面的整串预演就在分配器的拷贝上算上它们；
     // 其余那一截与准入那一格一样扣到生效。分配器记着回收过的槽不重复回收，第二次只回收第一次没回收的那几个。
     let floor_of_the_reclaim = reclaim_floor(new_floor, oldest_valid_root_in_the_ring);
@@ -5624,9 +5607,11 @@ fn raise_the_floor_through<Device: BlockDevice>(
             current.record.counter,
             current.record.instance,
         ) {
+            // 写成了：这一读留在写入口上，这一串第一次空发布落盘时一并转交。
             Ok(writes) => writes,
             Err(failed) => {
                 *allocator = allocator_before_the_raise;
+                pool.count_the_reads_before_the_system_configuration_writes(allocator);
                 return Err(
                     MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(Box::new(
                         failed,
@@ -7080,12 +7065,16 @@ struct VersionJudgedBehindOrNot<'version> {
 /// 落后支缺单元那一半（[`what_a_device_behind_the_version_lacks`] 先判它）：`newest_self_verified` 是这块盘两槽里本池自证过、世代号最大的那一份系统配置。
 /// 这一半算不算落后（乙'，三方判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U3 与第四节「落地」，C593（落后且缺一份单元的盘被永久拒可写））：
 /// 它的 (实例代号, journal tail) 不落后于那一版那次发布末条记录的 jsn，或这块盘自己的 journal 环槽里有那一版的末条记录
-/// （[`last_record_of_the_version_in_its_ring_slot_on_the_device`]），就不算落后、交回 `None`（不读它的单元）——单元写在记录之前那道屏障之前，
-/// 末条记录在本盘是「落盘之后坏了一份」与「从没落盘」唯一分得开的盘上证据。算落后的再逐个读 `units` 在这块盘上的落点，读不出或字节与验得过的那一份不同就记下，
-/// 一个都没记下也交回 `None`（只落后、单元都在：根落盘之后、轮换之前崩，或只剩一槽自证、那一槽停在上一次轮换）；记下了至少一个才交回 (那份的 tail, 缺的单元)。
+/// （[`last_record_of_the_version_in_its_ring_slot_on_the_device`]），就不算落后、交回 `None`（不读它的单元）。
+/// 算落后的再逐个读 `units` 在这块盘上的落点，读不出或字节与验得过的那一份不同的，只有落在 (tail, 那一版] 那个窗口里记录点名的落点上才算缺
+/// （D18（块里携带什么信息） 已定项 11）：单元写在记录之前那道屏障之前、轮换写在屏障之后，
+/// 这块盘收过 tail 及之前每次发布的单元，窗口之外缺的是落盘之后坏了的一份。窗口取不到（[`slots_named_by_the_records_after_the_tail`] 交回 `None`）时全部算缺。
+/// 算缺的一个都没有也交回 `None`（只落后、单元都在：根落盘之后、轮换之前崩，或只剩一槽自证、那一槽停在上一次轮换；或只有窗口之外坏了的那几份）；有才交回 (那份的 tail, 缺的单元)。
+/// 窗口之外坏了的那几份各按读到只坏一份报给读者（[`PoolReader::note_that_a_unit_was_read_with_one_copy_failing`]，D4（校验和位置） 已定项 4）：
+/// 第二条位置条目上那一份只有这里看得见，读单元读到第一条就停。挂载与会话改动的入口经数只坏一份的那一层交读者，记进别的读那一份账，下一次用户改动发成之后另发的那一批换下它。
 /// 字节是空的单元（提示与映射都读不出的数据单元，D19（块指针的结构与宽度预算） 已定项 5）没有验得过的一份可比，跳过。只读盘。
 ///
-/// 循环：按 `units`（轮数 = 这一版的单元数），跨轮只往缺的清单里追加，没有提前出口。
+/// 循环：按 `units`（轮数 = 这一版的单元数）筛一遍，再按筛出来的单元报（轮数不超过这一版的单元数），跨轮只往清单里追加，没有提前出口。
 fn units_of_the_version_missing_on_a_device_behind_it<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     parameters_of_the_pool: &MakeFilesystemParameters,
@@ -7110,7 +7099,7 @@ fn units_of_the_version_missing_on_a_device_behind_it<Reader: PoolReader + ?Size
     ) {
         return None;
     }
-    let units_missing: Vec<(TransactionUnit, SlotNumber)> = version
+    let units_differing_on_this_device: Vec<&PublishedUnit> = version
         .units
         .iter()
         .filter(|unit| !unit.bytes.is_empty())
@@ -7119,11 +7108,93 @@ fn units_of_the_version_missing_on_a_device_behind_it<Reader: PoolReader + ?Size
                 .read(device, unit.slot.to_device_offset(), unit.bytes.len())
                 .is_some_and(|bytes_on_this_device| bytes_on_this_device[..] == unit.bytes[..])
         })
+        .collect();
+    let slots_named_after_the_tail = slots_named_by_the_records_after_the_tail(
+        reader,
+        parameters_of_the_pool,
+        newest_system_configuration_journal_tail,
+        version_journal_position,
+    );
+    let (units_counted_as_missing, units_with_only_this_copy_damaged): (
+        Vec<&PublishedUnit>,
+        Vec<&PublishedUnit>,
+    ) = units_differing_on_this_device
+        .into_iter()
+        .partition(|unit| {
+            slots_named_after_the_tail
+                .as_ref()
+                .is_none_or(|slots_named| slots_named.contains(&unit.slot))
+        });
+    let identities_of_the_devices = reader.device_identities();
+    for unit in units_with_only_this_copy_damaged {
+        reader.note_that_a_unit_was_read_with_one_copy_failing(
+            &crate::make_filesystem::location_entries(
+                &identities_of_the_devices,
+                unit.slot,
+                &unit.bytes,
+            ),
+        );
+    }
+    let units_missing: Vec<(TransactionUnit, SlotNumber)> = units_counted_as_missing
+        .into_iter()
         .map(|unit| (unit.identity, unit.slot))
         .collect();
     (!units_missing.is_empty()).then_some((newest_system_configuration_journal_tail, units_missing))
 }
 
+/// (这块盘的 tail, 那一版那次发布末条记录] 那个窗口里每条记录点名的槽（第一版两盘同槽，D2（RAID 条带策略） 已定项 10，点名的槽就是这块盘上的落点）。
+/// 交回 `None` 的两种（调用方全部算缺）：tail 的实例代号不是那一版的（跨实例的窗口不放行）；窗口里有一个计数器在哪块盘的环槽里都解不出这个计数器的记录
+/// （读不出、自证不过，或环转过一圈已被别的计数器盖掉：窗口比环长时第一个计数器就走这一支，读的槽数因此不超过环的槽数乘盘数）。
+/// 窗口里的记录按计数器认、实例代号不限：tail 与那一版同实例时，夹在中间的计数器可以住着上一个实例没来得及轮换的那条记录（取号写的 tail 是取号那一刻的见证值，新实例的计数器接在环里最大的计数器后面）。
+/// 同一个计数器几块盘上各有一条（分叉的盘）时取并集，多点名只多算缺、不多放行。调用方保证 tail 小于那一版的 jsn。只读盘。
+///
+/// 循环：外层按窗口里的计数器（轮数 = 末条计数器 − tail 的计数器），内层按盘（轮数 = 盘数）；跨轮只往槽的清单里追加；提前出口一个：某个计数器哪块盘上都解不出，整个交回 `None`。
+fn slots_named_by_the_records_after_the_tail<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    parameters_of_the_pool: &MakeFilesystemParameters,
+    tail: JournalSequenceNumber,
+    version_journal_position: JournalSequenceNumber,
+) -> Option<Vec<SlotNumber>> {
+    if tail.instance_generation != version_journal_position.instance_generation {
+        return None;
+    }
+    let record_bytes =
+        usize::try_from(singlefs_format::JOURNAL_RECORD_BYTES).expect("一条记录 4096 字节");
+    let filesystem_identifier_in_records =
+        crate::unit::unit_filesystem_identifier(&parameters_of_the_pool.filesystem_identifier);
+    let identities_of_the_devices = reader.device_identities();
+    let mut slots_named = Vec::new();
+    for counter in (tail.counter + 1)..=version_journal_position.counter {
+        let offset = crate::journal::record_offset(
+            counter,
+            parameters_of_the_pool.geometry.journal_ring_bytes,
+        );
+        let mut is_the_counter_found = false;
+        for holding_device in &identities_of_the_devices {
+            let record_of_the_counter = reader
+                .read(*holding_device, offset, record_bytes)
+                .and_then(|bytes| {
+                    crate::journal::JournalRecord::parse(&bytes, filesystem_identifier_in_records)
+                })
+                .filter(|record| record.counter == counter);
+            if let Some(record) = record_of_the_counter {
+                is_the_counter_found = true;
+                slots_named.extend(
+                    record
+                        .named
+                        .iter()
+                        .flat_map(|named| named.locations)
+                        .map(|location| location.slot),
+                );
+            }
+        }
+        if !is_the_counter_found {
+            return None;
+        }
+    }
+    Some(slots_named)
+}
+
 /// 这块盘自己的 journal 环里、那一版末条记录该在的那一槽（`journal::record_offset` 按计数器算）读一次，解得开、带末条标志、
 /// (实例代号, 计数器) 是 `version_journal_position`、(实例代号, checkpoint_txg) 是 `version` 的，交回 true（乙'）。
 /// 计数器是 0（那一版没有记录：环里一条记录都没有）的不读、交回 false。只读盘：至多读一槽，不重读。
@@ -7761,10 +7832,15 @@ fn establish_instance<Device: BlockDevice>(
         .map(|device_map| (device_map.device, device_map.isolated_slots()))
         .collect();
     // 所选那一版的单元清单在开写入口之前取好（树表 0 条那一版要按位置条目读盘）：取号之前逐盘核拿它比。
-    let units_of_the_selected_version = start.previous.units(
-        devices,
-        unit_area_start_of_the_chosen_system_configuration(&start.system_configuration),
-    )?;
+    // 这一读是这次挂载共用的那一份第一次读这几个单元，之后写系统配置槽之前读实例表取记下的那一份，读不到盘；
+    // 读到只坏一份进这次挂载别的读那一份账（D4（校验和位置） 已定项 4「可写挂载期间别的读」），建上一版已读到的那几组上面记过，不再加次数。
+    let units_of_the_selected_version = allocator
+        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(&*devices, |reader| {
+            start.previous.units(
+                reader,
+                unit_area_start_of_the_chosen_system_configuration(&start.system_configuration),
+            )
+        })?;
     let pool = PoolWriter::new(parameters, devices.as_mut_slice());
     let all_devices: Vec<DeviceIdentity> =
         pool.devices.iter().map(|(identity, _)| *identity).collect();
@@ -7834,6 +7910,7 @@ fn establish_instance<Device: BlockDevice>(
         system_configuration_slots_as_checked,
     } = devices_passing_the_check_before_the_acquisition(
         &pool,
+        &allocator,
         &start,
         &units_of_the_selected_version,
         &read_route,
@@ -8096,12 +8173,14 @@ fn publishes_after_acquisition_rehearsed<Device: BlockDevice>(
 /// 落后支缺根那一半只核写进了根环的所选那一版：择到的那条根上没施加 journal 记录才是它（施加过的那一版是崩在根槽 FUA 之前的那次发布，
 /// 哪块盘的根环里都没有它，那一形是正常的崩溃状态）。交回过了补写闸、两槽全废的盘（丙'，取号写两轮顺带补写它们），
 /// 连同每块盘每个系统配置槽读的那一刻的结论（改法丙，取号之前预演续做判两本账时用）。
+/// 落后的盘上窗口之外坏了的那几份（[`units_of_the_version_missing_on_a_device_behind_it`]）放行时记进 `allocator` 上别的读那一份账；拒了分配器随错丢掉。
 ///
 /// # Errors
 /// 一块盘的系统配置槽在读阶段之后读不出 ⇒ [`refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage`] 那一种；
 /// 有盘不带所选那一版 ⇒ `WritableMountRefusedByDevicesWithoutTheSelectedVersion`。
 fn devices_passing_the_check_before_the_acquisition<Device: BlockDevice>(
     pool: &PoolWriter<'_, Device>,
+    allocator: &PoolAllocator,
     start: &InstanceStart,
     units_of_the_selected_version: &[PublishedUnit],
     read_route: &Cell<ReadRouteOfThisMount>,
@@ -8117,13 +8196,15 @@ fn devices_passing_the_check_before_the_acquisition<Device: BlockDevice>(
     } else {
         RootOfTheVersionInTheRing::NotWritten
     };
+    let reader_of_the_check =
+        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*pool.devices);
     let DevicesCheckedBeforeTheAcquisition {
         devices_without,
         devices_rewritten_by_the_acquisition,
         system_configuration_slots_as_checked,
     } = with_reads_straight_to_the_disk(read_route, || {
         devices_without_the_selected_version(
-            &*pool.devices,
+            &reader_of_the_check,
             &start.system_configuration,
             SelectedVersionCheckedOnEachDevice {
                 root: &start.effective_root,
@@ -8154,6 +8235,9 @@ fn devices_passing_the_check_before_the_acquisition<Device: BlockDevice>(
             },
         );
     }
+    allocator.count_the_other_unit_reads_with_one_copy_failing(
+        reader_of_the_check.locations_of_the_units_read_with_one_copy_failing(),
+    );
     Ok(DevicesPassingTheCheckBeforeTheAcquisition {
         devices_rewritten_by_the_acquisition,
         system_configuration_slots_as_checked,
@@ -8711,6 +8795,9 @@ fn acquire_then_publish_until_the_continuation<Device: BlockDevice>(
         );
     }
 
+    // 写入口上写系统配置槽之前读到只坏一份的那一份账到这里没有要记的：取号写读到的由接着那次经分配器的发布转交（`transaction::persist_the_publish_or_freeze_it`）；
+    // 接着是零单元发布只有要写的行为空这一种，即盘上没有实例代号大于 0 的系统配置与根（`transaction::next_instance_generation_to_acquire`），最新那条根指着 mkfs 那张表，
+    // 这一串读的就是取号之前取单元清单读过的那一片，读到只坏一份也是账上那一组；写过行之后的暖机读这次挂载自己写的那张表，读的是写下的那一份、不到盘。
     let space_admission = match admission_before_acquisition {
         WritableMountAdmissionBeforeAcquisition::Admitted => {
             MountSpaceAdmission::AdmittedBeforeAcquisition
```

## crates/singlefs-core/src/mounted_session.rs

```diff
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index d5f92c9..ea0b134 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -229,7 +229,8 @@ pub struct UserChangePublished {
 /// 一次用户改动发成之后，读到只坏一份的单元另发一批换落点（D4（校验和位置） 已定项 4「经已有的搬迁路径把这个单元整个搬到新槽、两份新写，旧落点照释放前核隔离；
 /// 走同一个事务层」；用户 2026-11-04 定有活快照时「另发一批搬迁修好」，另发那批被拒时「推抬 F 再试一遍」）做成了什么。封闭集合。用户改动在每一种结局下都已发成。
 /// 这一批管三处读到的：这次改动读—改—写读回的、改动之后仍被活快照钉着的旧单元；可写挂载读盘建上一版时读到的（挂载记下、挂载之后头一次改动发成时取走）；
-/// 可写挂载期间别的读读到的（[`MountedSession::read_the_file`]、推抬 F 与发 discard 之前读根环与实例表、整理的回退门、回退、清扫、释放之前读盘核、搬留存单元读源，还有这一批自己认位置时沿链读实例表）。
+/// 可写挂载期间别的读读到的（[`crate::allocator::PoolAllocator::reading_and_counting_the_other_unit_reads_with_one_copy_failing`] 文档串里那几类，其中写系统配置槽之前算 F 生效值每次发布都读最新那条根的实例表，
+/// 这次改动那次发布读到的片就由这一批换）。读的是别的版本、这一批在现行那一版里认不出的只留下计数，不搬。
 /// 豁免映射的三类（映射树节点、树表、实例表片）认不出映射 key，同一批按它们自己的重写路子换落点（`transaction::RepairOfTheExemptUnits`）。
 /// 这一组记下的这一次只试一次，结局报在这里，之后的用户改动不接着搬；同一组之后再被读到就再记、再试一次（账只记等下一次改动的那几组，试过的不留）。
 /// 没有活快照钉着时读回的单元在改动那次发布里就整个重写、旧落点照释放之前读盘核隔离，不另发。
@@ -400,7 +401,8 @@ pub enum CompactionRunOutcome {
     Failed(CompactionRunFailure),
 }
 
-/// 整理运行里一次发布没发成（被空间拒不在这里：那是暂停，[`CompactionRunStop::PausedShortOfSpace`]）。已发成的那几版留在盘上、`current` 停在最后那一版。
+/// 整理运行里一次发布没发成（一批被空间拒不在这里：那是暂停，[`CompactionRunStop::PausedShortOfSpace`]；暂停或出错停下时只删意图的那一次
+/// 没发成在这里，意图留在盘上、R 照旧排他）。已发成的那几版留在盘上、`current` 停在最后那一版。
 #[derive(Debug)]
 pub enum CompactionRunFailure {
     /// 先走空的删除续做有一步没发成。
@@ -559,6 +561,19 @@ enum PacingOfTheRun<'caller> {
     },
 }
 
+/// 这一趟整理的写带宽上限：后台入口照调用方给的那一个，显式入口与可写挂载续做不限。
+fn write_bandwidth_limit_of_the_pacing(
+    pacing: PacingOfTheRun<'_>,
+) -> CompactionWriteBandwidthLimit {
+    match pacing {
+        PacingOfTheRun::NotPaced => CompactionWriteBandwidthLimit::Unlimited,
+        PacingOfTheRun::PacedByTheCaller {
+            write_bandwidth_limit,
+            ..
+        } => write_bandwidth_limit,
+    }
+}
+
 /// 一次用户改动里推的抬 F 之外发成的每一版，按先后：先走空的删除步（D8（核心索引结构） 已定项 16 第 6 条）、这次改动本身
 /// （unlink 是意图那次）、unlink 之后走空的删除步；与推的抬 F 的空发布按 txg 交错。最后一个是会话的 `current`
 /// （之后没再推抬 F 时）。交给调用方逐次比（模型对拍）：会话的 `current` 只是最后那一版。
@@ -2812,7 +2827,7 @@ impl MountedSession {
     }
 
     /// 这次可写挂载里别的读读到只坏一份的次数（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬，包括……可写挂载期间别的读」）：[`Self::read_the_file`]、
-    /// 推抬 F 与发 discard 之前读根环与实例表、整理的回退门、回退、清扫、释放之前读盘核、搬留存单元读源、另发那一批认位置沿链读实例表（挂载这一串读到的那几组也在等下一次用户改动，再读不进这个数）。一组位置条目等着下一次用户改动的这一程里只算一次，处置过之后再读到再算。
+    /// [`crate::allocator::PoolAllocator::reading_and_counting_the_other_unit_reads_with_one_copy_failing`] 文档串里那几类（建上一版读到的那几组也在等下一次用户改动，再读不进这个数）。一组位置条目等着下一次用户改动的这一程里只算一次，处置过之后再读到再算。
     /// 与 [`Self::unit_reads_with_one_copy_failing_while_mounting`]、[`Self::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`] 按读的地方分开报。
     /// 只在内存里，不落盘；新挂载从 0 起。
     #[must_use]
@@ -2950,11 +2965,14 @@ impl MountedSession {
                 },
             );
         }
-        let devices_behind = devices_behind_the_current_version(
-            devices.as_slice(),
-            parameters,
-            current_version_of_trees,
-        );
+        // 落后的盘上窗口之外坏了的那几份经这一层记进别的读那一份账（`mount::devices_behind_the_current_version`），这次改动发成之后另发的那一批换下它们。
+        let devices_behind = allocator
+            .reading_and_counting_the_other_unit_reads_with_one_copy_failing(
+                devices.as_slice(),
+                |reader| {
+                    devices_behind_the_current_version(reader, parameters, current_version_of_trees)
+                },
+            );
         if !devices_behind.is_empty() {
             return Err(
                 UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
@@ -3026,7 +3044,7 @@ impl MountedSession {
     /// 没有活意图就选段、签发前判、发一次把意图写进 logged_ops 的发布（判决 G1）；有就对 R 重扫（不复用上一批的扫描，丁 1 交判第 3 点）、
     /// R 里有隔离槽就发一次删意图的发布丢掉它（攻方 K-c）、计划一批、发这一批，最后一批同一次发布删意图（D8（核心索引结构） 已定项 5）；
     /// 全局差值为假就不签下一条。这一趟至多发 `B_gc` 次整理发布（[`publish_budget_of_a_compaction_run`]，签发、搬一批、删意图各算一次）；
-    /// 一次整理发布被空间拒就暂停（撤 R 排他、意图留在盘上）、不推抬 F。发布都返回之后发 discard（判决 G5）。
+    /// 一次整理发布被空间拒就暂停：发一次只删意图的发布丢掉意图、连同 R 排他一起撤，不推抬 F，下一趟从头签发。发布都返回之后发 discard（判决 G5）。
     ///
     /// # Errors
     /// 盘表与挂载核过的不同、现行那一版树表 0 条 ⇒ [`CompactionRunRefused`]，在任何读写之前。
@@ -3208,7 +3226,8 @@ impl MountedSession {
 
     /// 整理的后台入口（D26（后台整理与放置回收） 已定项 13 第 5、6 条；用户 2026-10-26 定 GC「挪出调用线程，补齐三样保证」）：调用方在它自己的线程上
     /// 一次次调，核心层不起线程、不读钟（钟与前台的 fsync 由 `caller` 答）。每次调用先看起不起跑，依次：
-    /// 1. 整理因一批出错停着（第 4 条）而池子每块盘的已分配占它的容量都不超过补跑阈值 ⇒ 不起跑，「该跑了」留着，活意图的 R 排他撤掉（同出错停下那一刻）；
+    /// 1. 整理因一批出错停着（第 4 条）而池子每块盘的已分配占它的容量都不超过补跑阈值 ⇒ 不起跑，「该跑了」留着，一个写都不发（活意图出错停下那一刻已删；
+    ///    这一判之前才停着而还有活意图的，意图与 R 排他照留，等下一趟显式入口、补跑或解除停下）；
     ///    超过阈值就照下面几条照常判起不起跑，起跑之后每一批在 CPU 上补跑（用户 2026-10-28 定案）；
     /// 2. 用户改动之后起跑判据记下了「该跑了」⇒ 取走，照用户改动之后那一趟推（判决 G8 丁：至多一批）；
     /// 3. 没有 ⇒ 看定时兜底（`pacing.timer_fallback`）：关着、或离上一次查还不到一个间隔就不起跑；到了就再查一次起跑判据，为真就推一趟到停；
@@ -3357,7 +3376,7 @@ impl MountedSession {
             .put_the_switch_requested_from_the_next_batch_in_effect();
         let parameters = parameters_and_device_table.parameters_on_disk();
         let background_compute = parameters_and_device_table.background_compute_of_this_mount();
-        if compaction_stays_stopped_pausing_the_live_intent(allocator, background_compute) {
+        if compaction_stays_stopped(allocator, background_compute) {
             return Ok(BackgroundCompactionStart::NotStarted(
                 BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
             ));
@@ -4231,6 +4250,10 @@ enum CompactionPublishKind {
     IntentDropped,
     /// 丢意图：R 里一个能搬的都没有、却还有未释放的记录；丢完这一趟停（[`CompactionRunStop::NothingMovableButTheRegionIsNotEmpty`]）。
     IntentDroppedWithNothingMovable,
+    /// 丢意图：整理停下时单独一次发布删它（[`drop_the_live_intent_on_a_stop`]），丢完这一趟停在 `stop`：一批被空间拒的暂停
+    /// （[`CompactionRunStop::PausedShortOfSpace`]）、一批出错放弃（[`CompactionRunStop::BatchAbandonedAfterABackgroundComputeError`]）、
+    /// 停着而不补跑（[`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]）。
+    IntentDroppedOnAStop { stop: CompactionRunStop },
 }
 
 /// 推一步整理的结局：发成一次整理发布，或照判据停。
@@ -4249,21 +4272,17 @@ struct CompactionPublishAdvanced {
     background_compute_of_the_batch: Option<BackgroundComputeOfOneBatch>,
 }
 
-/// 整理此刻停着而不补跑（用户 2026-10-28 定案：GC 因一批出错停着、池子每块盘的已分配占它的容量都不超过阈值）：是就撤掉活意图的 R 排他、意图留着
-/// （同一批出错停下那一刻的暂停，停着的时候 R 不压着分配），交回真；没停着或要在 CPU 上补跑交回假，什么都不动。判的是这一刻分配器的计数，
-/// 不读写盘。
-fn compaction_stays_stopped_pausing_the_live_intent(
-    allocator: &mut PoolAllocator,
+/// 整理此刻停着而不补跑（D26（后台整理与放置回收） 已定项 13 第 4 条：GC 因一批出错停着、池子每块盘的已分配占它的容量都不超过补跑阈值）：
+/// 交回真；没停着或要在 CPU 上补跑交回假。只判，分配器与盘一样都不动：停着时还有活意图的由推整理的那一步删（[`drop_the_live_intent_on_a_stop`]）。
+fn compaction_stays_stopped(
+    allocator: &PoolAllocator,
     background_compute: &BackgroundComputeOfThisMount,
 ) -> bool {
     match background_compute.whether_the_task_runs_the_next_batch(
         BackgroundTask::CompactionRelocation,
         AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator(allocator),
     ) {
-        WhetherTheTaskRunsTheNextBatch::StaysStopped(_) => {
-            allocator.compaction_mut().pause_the_live_intent();
-            true
-        }
+        WhetherTheTaskRunsTheNextBatch::StaysStopped(_) => true,
         WhetherTheTaskRunsTheNextBatch::AsTheSettingsSay
         | WhetherTheTaskRunsTheNextBatch::OnTheCpuToCompensateWhileStopped(_) => false,
     }
@@ -4338,7 +4357,7 @@ const ROLLBACK_STATES_KEPT_BY_THE_COMPACTION: u64 = 4;
 /// （[`MountedSession::run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region`]），
 /// 每一批读完后台计算设置之后照 `in_the_middle_of_each_batch` 做（产品路径什么都不做，
 /// [`MountedSession::run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch`]）。整理因一批出错停着而每块盘都不超过补跑阈值时，
-/// 从判出的那一刻起一个写都不发（[`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]）。
+/// 从判出的那一刻起只发一次删活意图的发布（有活意图时），之后一个写都不发（[`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]）。
 #[allow(
     clippy::too_many_arguments,
     reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、推到哪为止、预算按哪一条算、每批之前看不看调用方、删意图那一支的核怎么判、这次挂载的后台计算状态、一批读完设置之后做什么"
@@ -4403,19 +4422,72 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
         }
     }
     // D26（后台整理与放置回收） 已定项 13 第 4 条：整理因一批出错停着，等一次通过校验的切换请求解除；停着而池子每块盘的已分配占它的容量都不超过补跑阈值时
-    // 一个写都不发（删除续做也不走），超过就照常推、每一批在 CPU 上补跑（用户 2026-10-28 定案）。挂着切到一张卡都用不上的那一组时，
-    // 先在这里换上它（一趟开头不在一批之内），与挂载那一刻一样当场判出停着。
+    // 删除续做不走：还有活意图（补跑那几批推着的、挂载续做的、停下那一刻没删成的）而这一趟的预算没用完，就单发一次只删意图的发布
+    // （[`drop_the_live_intent_on_a_stop`]），之后一个写都不发、当场停；没有活意图或预算用完就一个写都不发。超过阈值就照常推、每一批在 CPU 上补跑。
+    // 挂着切到一张卡都用不上的那一组时，先在这里换上它（一趟开头不在一批之内），与挂载那一刻一样当场判出停着。
     background_compute.put_the_switch_requested_from_the_next_batch_in_effect();
-    if compaction_stays_stopped_pausing_the_live_intent(allocator, background_compute) {
+    let budget = match budget_rule {
+        CompactionRunBudget::SinceTheLastUserChange => publish_budget_of_a_compaction_run(
+            parameters.geometry.root_ring_slots_per_region.count(),
+            compaction.publishes_since_the_last_user_change,
+        ),
+        CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange => {
+            publishes_left_of_the_budget_given_by_the_last_user_change(
+                parameters.geometry.root_ring_slots_per_region.count(),
+                compaction.publishes_since_the_last_user_change,
+            )
+        }
+        CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite => {
+            publishes_left_of_the_budget_given_by_the_last_user_change(
+                parameters.geometry.root_ring_slots_per_region.count(),
+                compaction.publishes_since_the_last_user_change,
+            )
+            .max(1)
+        }
+    };
+    if compaction_stays_stopped(allocator, background_compute) {
+        let stop = CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError;
+        let dropped_on_the_stop = if budget == 0 {
+            Ok(CompactionStep::Stopped(stop))
+        } else {
+            drop_the_live_intent_on_a_stop(
+                parameters, devices, allocator, compaction, current, instance, stop,
+            )
+        };
+        match dropped_on_the_stop {
+            Ok(CompactionStep::Published { version, kind: _ }) => {
+                compaction.publishes_since_the_last_user_change += 1;
+                compaction.observations.root_ring_slots_used += 1;
+                compaction.note_the_bytes_written_by_a_compaction_publish(
+                    write_bandwidth_limit_of_the_pacing(pacing),
+                    bytes_written_by_a_compaction_publish(&version, devices.len()),
+                );
+                versions_published.push(*version);
+            }
+            Ok(CompactionStep::Stopped(_nothing_to_drop)) => {}
+            Err(refused) => {
+                note_the_kind_of_a_publish_frozen_by_this_call(
+                    allocator,
+                    was_frozen_when_the_run_started,
+                    KindOfTheFrozenPublish::CompactionPublish,
+                );
+                return finish(
+                    compaction,
+                    current.root().checkpoint_txg,
+                    versions_published,
+                    background_compute_of_each_batch,
+                    0,
+                    CompactionRunOutcome::Failed(CompactionRunFailure::Batch(refused)),
+                );
+            }
+        }
         return finish(
             compaction,
             current.root().checkpoint_txg,
             versions_published,
             background_compute_of_each_batch,
             0,
-            CompactionRunOutcome::Stopped(
-                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError,
-            ),
+            CompactionRunOutcome::Stopped(stop),
         );
     }
     let batch_units = match switches.batch_size {
@@ -4494,25 +4566,6 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
             ),
         );
     }
-    let budget = match budget_rule {
-        CompactionRunBudget::SinceTheLastUserChange => publish_budget_of_a_compaction_run(
-            parameters.geometry.root_ring_slots_per_region.count(),
-            compaction.publishes_since_the_last_user_change,
-        ),
-        CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange => {
-            publishes_left_of_the_budget_given_by_the_last_user_change(
-                parameters.geometry.root_ring_slots_per_region.count(),
-                compaction.publishes_since_the_last_user_change,
-            )
-        }
-        CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite => {
-            publishes_left_of_the_budget_given_by_the_last_user_change(
-                parameters.geometry.root_ring_slots_per_region.count(),
-                compaction.publishes_since_the_last_user_change,
-            )
-            .max(1)
-        }
-    };
     // 回退门只在后台入口、开关照水位判时判（显式入口没有产品调用方，可写挂载续做只推盘上那一条，发布数有上界）。
     let is_the_rollback_gate_judged = matches!(pacing, PacingOfTheRun::PacedByTheCaller { .. })
         && matches!(
@@ -4674,19 +4727,29 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
                     | CompactionPublishKind::IntentIssued
                     | CompactionPublishKind::IntentCompletedWithoutABatch
                     | CompactionPublishKind::IntentDropped
-                    | CompactionPublishKind::IntentDroppedWithNothingMovable => {}
+                    | CompactionPublishKind::IntentDroppedWithNothingMovable
+                    | CompactionPublishKind::IntentDroppedOnAStop { .. } => {}
                 }
                 versions_published.push(*version);
-                if kind == CompactionPublishKind::IntentDroppedWithNothingMovable {
+                let stop_after_the_drop = match kind {
+                    CompactionPublishKind::IntentDroppedWithNothingMovable => {
+                        Some(CompactionRunStop::NothingMovableButTheRegionIsNotEmpty)
+                    }
+                    CompactionPublishKind::IntentDroppedOnAStop { stop } => Some(stop),
+                    CompactionPublishKind::IntentIssued
+                    | CompactionPublishKind::Batch
+                    | CompactionPublishKind::BatchCompletingTheIntent
+                    | CompactionPublishKind::IntentCompletedWithoutABatch
+                    | CompactionPublishKind::IntentDropped => None,
+                };
+                if let Some(stop) = stop_after_the_drop {
                     return finish(
                         compaction,
                         current.root().checkpoint_txg,
                         versions_published,
                         background_compute_of_each_batch,
                         batches_published,
-                        CompactionRunOutcome::Stopped(
-                            CompactionRunStop::NothingMovableButTheRegionIsNotEmpty,
-                        ),
+                        CompactionRunOutcome::Stopped(stop),
                     );
                 }
             }
@@ -4926,11 +4989,13 @@ fn compaction_publish_refused_short_of_space(refused: &RelocationBatchRefused) -
 /// 断言没换）。选 `gpu` 时这一批的甲、乙、丙三段经设置带着的那一份 GPU 实现算（[`CompactionComputeOfTheBatch::OnTheGpu`]），
 /// 三段各自确认、不确定、交 CPU 判的条目数记进这一批的记账；发布那一半照旧在 CPU 上，照发布路径逐条核计划与读盘核（第 8 条）。
 /// 读完设置之后、开算之前照 `in_the_middle_of_each_batch` 做（只供测试的钩子在这一刻请求切换、或注入一次出错）。
-/// 这一批算着出了错（第 4 条：GPU 实现交回出错，或只供测试的钩子注入）：这一批放弃，一个写都不发、意图留在盘上（撤 R 排他，同暂停），
-/// 不换回 CPU；整理这个后台任务停下，出错之前就挂着待生效的切换时当场解除（`background_compute::BackgroundComputeOfThisMount::stop_the_task_after_a_compute_error`）。
-/// 整理停着时（用户 2026-10-28 定案）：进门那一刻池子最满的那一块盘的已分配占它的容量超过补跑阈值，这一批走 CPU（记账的实际走法是
-/// `ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError`，带那一判的读数）；不超过就不开这一批、一个写都不发，撤 R 排他，
-/// 停在 [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]。
+/// 这一批算着出了错（第 4 条：GPU 实现交回出错，或只供测试的钩子注入）：这一批放弃、一个单元都不搬，不换回 CPU；整理这个后台任务停下，
+/// 出错之前就挂着待生效的切换时当场解除（`background_compute::BackgroundComputeOfThisMount::stop_the_task_after_a_compute_error`）；
+/// 现行那一版里有活意图就单发一次只删意图的发布丢掉它（[`drop_the_live_intent_on_a_stop`]，同被空间拒的暂停），
+/// 停在 [`CompactionRunStop::BatchAbandonedAfterABackgroundComputeError`]。
+/// 整理停着时（第 4 条的补跑阈值）：进门那一刻池子最满的那一块盘的已分配占它的容量超过补跑阈值，这一批走 CPU（记账的实际走法是
+/// `ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError`，带那一判的读数）；不超过就不开这一批、一个单元都不搬，
+/// 有活意图就同样单发一次只删意图的发布，停在 [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]。
 #[allow(
     clippy::too_many_arguments,
     reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、批大小、推到哪为止、一批腾空了 R 之后删意图那一支的核怎么判、这次挂载的后台计算状态、一批读完设置之后做什么"
@@ -4952,12 +5017,17 @@ fn advance_compaction_by_one_publish<Device: BlockDevice>(
         BackgroundTask::CompactionRelocation,
         AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator(allocator),
     ) else {
-        // 停着而这一批开头占用不超过补跑阈值（上一批补跑腾出了空间）：这一批不开，同出错停下那一刻撤 R 排他、意图留在盘上。
-        allocator.compaction_mut().pause_the_live_intent();
+        // 停着而这一批开头占用不超过补跑阈值（上一批补跑腾出了空间、或一趟开头就停着而还有活意图）：这一批不开，有活意图就删。
         return CompactionPublishAdvanced {
-            step: Ok(CompactionStep::Stopped(
+            step: drop_the_live_intent_on_a_stop(
+                parameters,
+                devices,
+                allocator,
+                compaction,
+                current,
+                instance,
                 CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError,
-            )),
+            ),
             background_compute_of_the_batch: None,
         };
     };
@@ -5008,17 +5078,22 @@ fn advance_compaction_by_one_publish<Device: BlockDevice>(
         Ok(step) => Ok(step),
         Err(CompactionBatchNotCompleted::Refused(refused)) => Err(refused),
         Err(CompactionBatchNotCompleted::ComputeFailed(error)) => {
-            allocator.compaction_mut().pause_the_live_intent();
             let task_after_the_error = background_compute.stop_the_task_after_a_compute_error(
                 BackgroundTask::CompactionRelocation,
                 error.clone(),
             );
             compute_error_that_abandoned_the_batch = Some(error);
-            Ok(CompactionStep::Stopped(
+            drop_the_live_intent_on_a_stop(
+                parameters,
+                devices,
+                allocator,
+                compaction,
+                current,
+                instance,
                 CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                     task_after_the_error,
                 },
-            ))
+            )
         }
     };
     CompactionPublishAdvanced {
@@ -5147,17 +5222,18 @@ fn live_slots_per_cluster_segment_computing_on(
 ///    （签发那次发布自己的新落点也不进 R），发一次只把意图写进 logged_ops 的发布（`c0` = 现行那一版的 txg，`segs0` = 签发那次发布之前、
 ///    现行那一版 R 那块盘按记录口径的全空段数，判决 m4-segment-code-r2 第三节 H3，用户 2026-10-16 定「取签发之前」）；
 ///    被空间拒就撤排他、停（意图没签成）。
-/// 2. 有活意图：暂停过的重新排他；R 里出现隔离槽（直接查常驻隔离集，钉住与隔离同时成立也算，丁 1 交判第 4 点）、区域扫描拒了 R、扫出隔离类，
+/// 2. 有活意图：R 里出现隔离槽（直接查常驻隔离集，钉住与隔离同时成立也算，丁 1 交判第 4 点）、区域扫描拒了 R、扫出隔离类，
 ///    都发一次只删意图的发布丢掉它（攻方 K-c）；对 R 重扫（每批之前都重扫，丁 1 交判第 3 点）、计划一批（跳过这次发布本来就重写的节点、补上 R 里
 ///    真要搬的单元，判决 m4-segment-code-r2 第三节 H4）：要搬就发——这一批罩住 R 里每一条未释放的记录时同一次发布删意图（D8（核心索引结构）
 ///    已定项 5；R 里只剩这次发布本来就重写的节点时也是这一支；发布在算好的账上核 R 按记录全空，核不过照这一批不删意图再发）；别的批照抄意图、
 ///    在算好的账上核 R 还有未释放的记录，这一批腾空了 R（计划外的、这次发布本来就重写的节点也离开 R）就照同一批删意图再发（判决 m4-segment-code-r1
 ///    第三节 F2），R 还不空而计划里有本来就重写、不另搬的单元就跳过它们再计划，所以照抄意图的一批至少真搬一个；一个都没搬的不记成一批；
 ///    R 已按记录全空就发一次只删意图的发布；
-///    一个能搬的都没有而 R 不空就发一次只删意图的发布丢掉它、这一趟停。整理的发布被空间拒就暂停（撤 R 排他、意图留在盘上，攻方 K-k）、停。
+///    一个能搬的都没有而 R 不空就发一次只删意图的发布丢掉它、这一趟停。整理的发布被空间拒就暂停：发一次只删意图的发布丢掉它、连同 R 排他
+///    一起撤、停，下一次整理从头签发（[`pause_by_dropping_the_live_intent_or_hand_up`]）。整理停下的路上没有「意图留在盘上、R 不排他」那一态。
 ///
 /// 一次调用至多算 R 里能搬的单元数加三次账、发一次（每跳过一次本来就重写的单元再算一次；照抄那一支核出腾空了 R 换删意图再算，删意图那一支核不过
-/// 换照抄再算），选段那一圈的上界是单元区的整段数。
+/// 换照抄再算；被空间拒暂停时再算一次丢意图的账），选段那一圈的上界是单元区的整段数。
 ///
 /// 区域扫描与选段打分照 `compute` 在哪算（[`CompactionComputeOfTheBatch`]）：走 GPU 时三段交实现，判定都确认（或交 CPU 判）时每一步与 CPU 那一条
 /// 逐项相同；这一轮不确定的条目这一批不回收、不搬（D26（后台整理与放置回收） 已定项 13 第 8 条）——不确定的段不进选段候选，不确定的记录不当可搬；
@@ -5442,13 +5518,6 @@ fn advance_compaction_by_one_publish_computing_on<Device: BlockDevice>(
             }
         };
     };
-    let is_the_region_excluded = allocator
-        .compaction()
-        .live_intent()
-        .is_some_and(|live| live.is_the_region_excluded_from_placement);
-    if !is_the_region_excluded {
-        allocator.compaction_mut().resume_the_live_intent();
-    }
     let region = intent.region;
     // 对 R 重扫的结果与这一轮不确定的记录（CPU 那一条恒空）。
     let scan: Option<(RegionScan, Vec<Placement>)> =
@@ -5526,8 +5595,10 @@ fn advance_compaction_by_one_publish_computing_on<Device: BlockDevice>(
                 );
             }
             Err(CompactionStepRefused::Refused(refused)) => {
-                return pause_or_hand_up(allocator, refused)
-                    .map_err(CompactionBatchNotCompleted::from)
+                return pause_by_dropping_the_live_intent_or_hand_up(
+                    parameters, devices, allocator, compaction, current, instance, refused,
+                )
+                .map_err(CompactionBatchNotCompleted::from)
             }
         }
     };
@@ -5539,7 +5610,8 @@ fn advance_compaction_by_one_publish_computing_on<Device: BlockDevice>(
             compaction.last_completed_intent = Some(intent);
         }
         CompactionPublishKind::IntentDropped
-        | CompactionPublishKind::IntentDroppedWithNothingMovable => {
+        | CompactionPublishKind::IntentDroppedWithNothingMovable
+        | CompactionPublishKind::IntentDroppedOnAStop { .. } => {
             allocator.compaction_mut().withdraw_the_live_intent();
             compaction.observations.intents_dropped += 1;
         }
@@ -5772,19 +5844,77 @@ fn publish_the_planned_compaction_step_falling_back_on_the_region_check<Device:
     }
 }
 
-/// 整理的一次发布没发成（活意图在盘上）：空间不够就暂停——撤 R 排他、意图留着（攻方 K-k），不推抬 F（判决 G8 g1）；别的原样交回。
-fn pause_or_hand_up(
+/// 整理的一次发布没发成（活意图在盘上，现行那一版就是没发成的那一次的上一版）：空间不够就暂停——在现行那一版上单发一次丢意图的发布
+/// （[`drop_the_live_intent_on_a_stop`]，停在 [`CompactionRunStop::PausedShortOfSpace`]），下一次整理照常从头签发、重扫，不续做这一条。
+/// 不推抬 F（判决 G8 g1）。别的拒绝原样交回，什么都不写。
+fn pause_by_dropping_the_live_intent_or_hand_up<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut [(DeviceIdentity, Device)],
     allocator: &mut PoolAllocator,
+    compaction: &mut CompactionOfThisMount,
+    current: &mut PoolVersion,
+    instance: InstanceGeneration,
     refused: RelocationBatchRefused,
 ) -> Result<CompactionStep, RelocationBatchRefused> {
-    if compaction_publish_refused_short_of_space(&refused) {
-        allocator.compaction_mut().pause_the_live_intent();
-        Ok(CompactionStep::Stopped(
-            CompactionRunStop::PausedShortOfSpace,
-        ))
-    } else {
-        Err(refused)
+    if !compaction_publish_refused_short_of_space(&refused) {
+        return Err(refused);
     }
+    drop_the_live_intent_on_a_stop(
+        parameters,
+        devices,
+        allocator,
+        compaction,
+        current,
+        instance,
+        CompactionRunStop::PausedShortOfSpace,
+    )
+}
+
+/// 整理停下时删掉现行那一版里的活意图（被空间拒的暂停、一批出错放弃、停着而不补跑三处共用）：单发一次只删意图的发布
+/// （[`CompactionIntentStep::Drop`]，不搬单元），发成之后撤分配器这一侧的活意图连同 R 排他、记一次丢意图，现行那一版换成删意图那一版，
+/// 交回那一次发布（种类 [`CompactionPublishKind::IntentDroppedOnAStop`]，这一趟随后停在 `stop`）。R 排他一直留到删意图那一版落盘：
+/// 盘上每一版要么意图活着而 R 排他着，要么意图已不在（I-5.6「活意图的 R 内没有分配代 > `c0` 的记录」），没有「意图在、R 不排他」那一态。
+/// 只删意图、不搬单元的整理发布归删除类：式子那一判不判、落点那一判照判（D28（挂载期承诺量） 已定项 1 的不判名单，
+/// `transaction::judge_the_publish_before_any_read_or_write`）。
+/// 现行那一版里没有活意图就什么都不写，交回停在 `stop`。
+///
+/// # Errors
+/// 删意图那一次没发成（落点取不到，或发布路径的错）：原样交回。取落点被拒在任何写之前，盘上逐字节不变，意图留在盘上、R 照旧排他，
+/// 下一次整理再走到停下时再删。
+fn drop_the_live_intent_on_a_stop<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut [(DeviceIdentity, Device)],
+    allocator: &mut PoolAllocator,
+    compaction: &mut CompactionOfThisMount,
+    current: &mut PoolVersion,
+    instance: InstanceGeneration,
+    stop: CompactionRunStop,
+) -> Result<CompactionStep, RelocationBatchRefused> {
+    let Some(version_with_the_live_intent) = current
+        .version_of_trees()
+        .filter(|version| {
+            live_compaction_intent_of_the_entries(&version.logged_operations).is_some()
+        })
+        .cloned()
+    else {
+        return Ok(CompactionStep::Stopped(stop));
+    };
+    let published = publish_the_compaction_step_once(
+        parameters,
+        devices,
+        allocator,
+        &version_with_the_live_intent,
+        &[],
+        CompactionIntentStep::Drop,
+        instance,
+    )?;
+    allocator.compaction_mut().withdraw_the_live_intent();
+    compaction.observations.intents_dropped += 1;
+    *current = PoolVersion::of_a_version_of_trees(published.clone());
+    Ok(CompactionStep::Published {
+        version: Box::new(published),
+        kind: CompactionPublishKind::IntentDroppedOnAStop { stop },
+    })
 }
 
 /// 现行那一版没有可改的文件：拒成 [`UserChangeRefused::NoFileVersionToChange`]，带上这次改动里已经落盘的（走空的那几步与推的抬 F）。
```

## crates/singlefs-core/src/transaction.rs

```diff
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 0a4ff61..3cdd82b 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -374,6 +374,11 @@ pub struct PoolWriter<'pool, Device: BlockDevice> {
     /// 把这次发布之后仍被活快照钉着（换下来进 deadlist、映射条目留存）的那几个搬到新槽（用户 2026-11-04 定「另发一批搬迁修好」，
     /// [`relocations_of_old_data_units_kept_for_the_snapshots`]）。只在内存里；新开的写入口从空起。
     mapping_keys_of_old_data_units_read_back_with_one_copy_failing: RefCell<Vec<Vec<u8>>>,
+    /// 这个写入口上写系统配置槽之前算 F 生效值（[`Self::write_system_configuration_slot`]）读到只坏一份的那几组位置条目，按读的先后；
+    /// 每次发布都走这一读，读的是最新那条根的实例表（D4（校验和位置） 已定项 4「可写挂载期间别的读」）。写入口手里没有分配器：
+    /// 有分配器的调用方接着把它们记进这次挂载别的读那一份账（[`Self::count_the_reads_before_the_system_configuration_writes`]）。只在内存里；新开的写入口从空起。
+    locations_read_with_one_copy_failing_before_the_system_configuration_writes:
+        RefCell<Vec<[LocationEntry; 2]>>,
     /// 这个写入口上最近一次分配记录树重写集的固定点走了几轮（[`PoolWriter::rounds_of_the_last_allocation_record_tree_settling`]）。只在内存里、不落盘；新开的写入口从 0 起。
     rounds_of_the_last_allocation_record_tree_settling: Cell<usize>,
 }
@@ -415,6 +420,8 @@ impl<'pool, Device: BlockDevice> PoolWriter<'pool, Device> {
             residue_reads: Cell::new(ResidueReadsOfTheWriter::NONE),
             old_data_unit_reads_back_with_one_copy_failing: Cell::new(0),
             mapping_keys_of_old_data_units_read_back_with_one_copy_failing: RefCell::new(Vec::new()),
+            locations_read_with_one_copy_failing_before_the_system_configuration_writes:
+                RefCell::new(Vec::new()),
             rounds_of_the_last_allocation_record_tree_settling: Cell::new(0),
         }
     }
@@ -451,6 +458,18 @@ impl<Device: BlockDevice> PoolWriter<'_, Device> {
         self.old_data_unit_reads_back_with_one_copy_failing.get()
     }
 
+    /// 把写系统配置槽之前算 F 生效值读到只坏一份的那几组位置条目记进这次挂载别的读那一份账（[`PoolAllocator::count_the_other_unit_reads_with_one_copy_failing`]），
+    /// 写入口上的清空。下一次用户改动发成之后另发的那一批按它们换落点，发布本身已发成的那次改动的那一批就算下一批。
+    pub(crate) fn count_the_reads_before_the_system_configuration_writes(
+        &self,
+        allocator: &PoolAllocator,
+    ) {
+        allocator.count_the_other_unit_reads_with_one_copy_failing(
+            self.locations_read_with_one_copy_failing_before_the_system_configuration_writes
+                .take(),
+        );
+    }
+
     /// 同 [`Self::old_data_unit_reads_back_with_one_copy_failing`] 那几次读回的旧单元在上一版里的映射 key，按读的先后，一次一把。
     /// 有活快照时读回的旧单元换下来进 deadlist、不释放（判决 m4-partial-write-r1 R7），坏的那一份留在旧落点上；会话按这几把 key
     /// 另发一批搬迁（[`relocations_of_old_data_units_kept_for_the_snapshots`]）。
@@ -676,12 +695,20 @@ impl<Device: BlockDevice> PoolWriter<'_, Device> {
         // 要按它分两类），而这一写是发布末尾的轮换、取号或先写 F，报错的去处只有块设备错。退路不让 F 回落：生效值取系统配置里读得出的
         // F 与根上带的 F 的大者，要被覆写的那一槽在这里先读进来，根上的 F 不高于它那块盘系统配置里的（I-7.12（系统配置 F 不低于同盘根上的 F）），
         // 只剩「那块盘两槽系统配置与带最高 F 的根同时读不出」这一种叠加故障算得低（推的，没造过）。
+        let devices_counting_unit_reads_with_one_copy_failing =
+            PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*self.devices);
         let pool_effective_floor = effective_rollback_floor_rereading_unreadable_reads_once(
-            &*self.devices,
+            &devices_counting_unit_reads_with_one_copy_failing,
             &self.parameters.region_devices,
             &self.parameters.geometry,
             &self.parameters.filesystem_identifier,
         );
+        self.locations_read_with_one_copy_failing_before_the_system_configuration_writes
+            .borrow_mut()
+            .extend(
+                devices_counting_unit_reads_with_one_copy_failing
+                    .locations_of_the_units_read_with_one_copy_failing(),
+            );
         let rollback_floor = match rollback_floor_to_write {
             RollbackFloorOfASystemConfigurationWrite::PoolEffectiveFloor => pool_effective_floor,
             RollbackFloorOfASystemConfigurationWrite::RaisedFloor(raised_floor) => {
@@ -1626,7 +1653,10 @@ fn persist_the_publish_or_freeze_it<Device: BlockDevice>(
     version: PoolVersion,
 ) -> Result<PoolVersion, PublishError> {
     let writes_before_this_publish = pool.writes_by_structure_kind.clone();
-    if let Err(cause) = persist_publish_writes(pool, &writes) {
+    let persisted = persist_publish_writes(pool, &writes);
+    // 账各份分配器拷贝共用：落盘中途失败、分配器换回发布之前那一份，读到的照样在账上（读是真发到盘上的）。
+    pool.count_the_reads_before_the_system_configuration_writes(allocator);
+    if let Err(cause) = persisted {
         pool.count_failed_publish(&writes_before_this_publish);
         let allocator_after_the_publish =
             std::mem::replace(allocator, allocator_before_this_publish);
@@ -1658,7 +1688,9 @@ pub fn resend_the_frozen_publish<Device: BlockDevice>(
         return Ok(None);
     };
     let writes_before_the_resend = pool.writes_by_structure_kind.clone();
-    if let Err(cause) = persist_publish_writes(pool, &frozen.writes) {
+    let persisted = persist_publish_writes(pool, &frozen.writes);
+    pool.count_the_reads_before_the_system_configuration_writes(allocator);
+    if let Err(cause) = persisted {
         pool.count_failed_publish(&writes_before_the_resend);
         allocator.freeze_publish(frozen);
         return Err(PublishError::BlockDevice(cause));
@@ -10669,10 +10701,12 @@ impl From<PublishError> for RelocationBatchRefused {
 /// 用户 2026-10-26 定 C906「照条款：整理批走常规准入」）：式子那一判照判，需求是这次重写的角色（这一个入口照抄 logged_ops，重写的只有记账树、
 /// 分配记录树、中央映射树与树表这几样固定点），批次最坏占用（计划里每一项在目的地各占它的跨度、每份副本各一份，
 /// `admission::worst_case_occupancy_of_a_compaction_batch`）加进这一次读数的已承诺预留；可用压到需求之下就拒成 `SpaceAdmissionRefused`，
-/// 在任何写之前，会话照暂停办（意图留在盘上，下次再试）。落点那一判只在重写的角色里有普通分配时判（`admission::publish_has_an_ordinary_allocation`），
-/// 搬迁副本的落点由取目的地那一步在任何写之前判（取不到交回 `PlacementRefused`）。合取 7 免（带搬迁的发布不是前台写入，
-/// `reach_of_the_empty_cluster_segments_conjunct`）。同一发布路径上签发、删意图、丢意图那几次重写 logged_ops 根（普通分配），两判照判
-/// （[`publish_compaction_step`]）。
+/// 在任何写之前，会话照暂停办（单发一次只删意图的发布丢掉意图，下一次整理从头签发）。落点那一判只在重写的角色里有普通分配时判
+/// （`admission::publish_has_an_ordinary_allocation`），搬迁副本的落点由取目的地那一步在任何写之前判（取不到交回 `PlacementRefused`）。
+/// 合取 7 免（带搬迁的发布不是前台写入，`reach_of_the_empty_cluster_segments_conjunct`）。同一发布路径上（[`publish_compaction_step`]）
+/// 签发那一次照判：logged_ops 根重写是普通分配，两判照判。只删意图、不搬单元的那几次（意图达成而 R 已按记录全空、丢意图、整理停下时删意图）
+/// 归删除类（D28（挂载期承诺量） 已定项 1 的不判名单）：式子那一判不判；落点那一判照判——logged_ops 删空时这一版没有 `LoggedOperationsRoot`、
+/// 重写的只有固定点，没有普通分配、落点那一判也不判，固定点照常取落点；logged_ops 里另有截断意图时它重写 logged_ops 根，落点那一判照判。
 ///
 /// # Errors
 /// 计划对不上上一版 ⇒ [`RelocationBatchRefused`] 那四种，在任何读写之前；其余同 `publish_version`（`Publish`）。
@@ -13357,15 +13391,18 @@ fn judge_the_publish_before_any_read_or_write<Device: BlockDevice>(
     );
     // 删除类发布（意图、各批、最后一批）不判式子那一判、落点那一判照判（D28（挂载期承诺量） 已定项 1：它们的空间罩在环窗口里）。
     // 删快照的三种发布与 unlink 同准入（判决第七节岔路 4「删快照照 unlink 的过程做、同等准入」）：式子那一判同样不判。
-    // 整理的各次发布（`publish_compaction_step`）不是删除类，走 `NotADeletion` 那一臂。整理不是释放路径、走常规准入（D26（后台整理与放置回收）
-    // 已定项 3 第 2 句；用户 2026-10-26 定 C906「照条款：整理批走常规准入」）：搬了单元的一批照判式子那一判，哪怕重写的角色只有固定点，
-    // 批次最坏占用（计划里每一项都真搬，[`worst_case_occupancy_of_a_compaction_batch`]）加进这一次读数的已承诺预留；签发、删意图、丢意图
-    // 重写 logged_ops 根（普通分配），照判。可用压到需求之下就拒（`SpaceAdmissionRefused`），会话照暂停办：意图留在盘上，下次再试。
+    // 整理的各次发布（`publish_compaction_step`）走 `NotADeletion` 那一臂。整理不是释放路径、走常规准入（D26（后台整理与放置回收）
+    // 已定项 3 第 2 句；C906「照条款：整理批走常规准入」）：搬了单元的一批照判式子那一判，哪怕重写的角色只有固定点，
+    // 批次最坏占用（计划里每一项都真搬，[`worst_case_occupancy_of_a_compaction_batch`]）加进这一次读数的已承诺预留；签发重写 logged_ops 根
+    // （普通分配），照判。可用压到需求之下就拒（`SpaceAdmissionRefused`），会话照暂停办：单发一次只删意图的发布丢掉意图。
+    // 只删整理意图、不搬单元的发布（意图达成而 R 已空、丢意图、暂停与出错停下时删意图）归删除类（D28（挂载期承诺量） 已定项 1 的不判名单）：
+    // 式子那一判不判，落点那一判在上面照判（logged_ops 里另有截断意图、重写 logged_ops 根时）。
     // 截断的意图、各批、最后一批同样当删除类（判决 m4-trunc-r1 K10）；变长与只改记录当普通写，照判。
     let demand_is_judged = match (allocator.space_admission(), deletion_and_lineage.step) {
         (SpaceAdmission::JudgedByTheFormula, step) => {
             !step.is_a_deletion_class_publish()
                 && !deletion_and_lineage.snapshot.is_a_snapshot_destruction()
+                && !deletes_only_the_compaction_intent(deletion_and_lineage, relocations)
                 && (publish_has_an_ordinary_allocation(&settled.resolved.rewritten_roles)
                     || !relocations.is_empty())
         }
@@ -13403,6 +13440,24 @@ fn judge_the_publish_before_any_read_or_write<Device: BlockDevice>(
     })
 }
 
+/// 这次发布只删整理意图、不搬单元（[`CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords`] 不带一批、[`CompactionIntentStep::Drop`]）：
+/// 归删除类，式子那一判不判（D28（挂载期承诺量） 已定项 1 的不判名单，[`judge_the_publish_before_any_read_or_write`]）。
+/// logged_ops 那一步之外别的都照抄（整理的发布不写文件、不建 inode、不建也不删快照），只看 logged_ops 那一步与搬不搬单元。
+fn deletes_only_the_compaction_intent(
+    deletion_and_lineage: &DeletionAndLineageOfThePublish,
+    relocations: &[RelocationOfThePublish],
+) -> bool {
+    let deletes_the_compaction_intent = match deletion_and_lineage.logged_operations {
+        LoggedOperationsStepOfThePublish::DeleteTheCompactionIntent { .. } => true,
+        LoggedOperationsStepOfThePublish::CarryTheLoggedOperations
+        | LoggedOperationsStepOfThePublish::IssueTheCompactionIntent(_)
+        | LoggedOperationsStepOfThePublish::IssueTheTruncationIntent(_)
+        | LoggedOperationsStepOfThePublish::DeleteTheTruncationIntent { .. }
+        | LoggedOperationsStepOfThePublish::TakeTheTruncationEntryOfTheRollbackTarget(_) => false,
+    };
+    deletes_the_compaction_intent && relocations.is_empty()
+}
+
 /// [`judge_the_publish_before_any_read_or_write`] 判过的：走到固定点的这一次发布、记账行的盘、这次切成几条 journal 记录。
 struct PublishJudgedBeforeAnyReadOrWrite {
     settled: SettledPublish,
@@ -13811,7 +13866,7 @@ fn role_standing_for_the_unit_of_a_deadlist_entry(
 /// **走得完**：产品路径上释放只改写记录、不在这次发布里空出槽（回收要等根记下之后），取落点时分配记录树的节点之后只剩单槽的提交内生块、
 /// 按同一串槽位往后取——多一个角色只把这一串拉长，有记录的节点只多不少，重写集也只增不减，都以池里的节点数为上界。
 /// 节点数可以比 [`ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`] 大，这个上界不保证轮数够：释放链沿活跃叶连成一长串时，一轮只走一跳就要走满轮数，
-/// 所以每次并进重写集都一次走到闭包，释放链不占轮数，轮数只剩取落点碰到新叶那一半。这一半的上界没有推出来，E206（发布固定点轮数的代价）全部取样点量到至多 5 轮。
+/// 所以每次并进重写集都一次走到闭包，释放链不占轮数，轮数只剩取落点碰到新叶那一半。这一半的上界没有推出来，E206（发布固定点轮数的代价）第一段 16 个取样点量到至多 5 轮，这几个点都有全空段、走不到回落；回落到最低空槽那一格没有取样点。
 /// 只供测试的复用窗口置 0 开关（`ReuseWindow::ForcedToZero`）让释放当场空出槽、回落可能取回它们，那一格的单调性也推不出来。
 /// 走满 [`ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`] 轮走不完，交回 `AllocationRecordTreeRewriteSetDidNotSettle`（条款没写这一格怎么办）。
 ///
```

