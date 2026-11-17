# m4-close-batch2-code-r2 被判的改动（2026-11-10）

## 一、代码轮 r1 判决之后的修补（A1、A2、A3 与补的用例），产品代码部分

```diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index ec52bfd..4366295 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -1984,7 +1984,9 @@ impl PoolAllocator {
     }
 
     /// 经数只坏一份的那一层做一次读（`read`），读完（读成与否）把数到的那几组记进账（[`Self::count_the_other_unit_reads_with_one_copy_failing`]）。
-    /// 走它的读：整理的回退门与后台入口读根环、准入算上限、取号之前取树表 0 条那一版的单元清单、会话改动入口的落后支、回退读现行那一版与目标那一版、回退的释放之前读盘核、清扫的判与发。
+    /// 走它的读：整理的回退门与后台入口读根环、准入算上限、取号之前取树表 0 条那一版的单元清单、会话改动入口的落后支、
+    /// 挂着之后收盘表那一核的落后支（`mount::caller_inputs_agreeing_with_the_disk`：抬 F、准入抬 F、一次准入里再推一串、推环、正常卸载、管理员回退共用）、
+    /// 读文件入口的落后支（`mounted_session::refuse_a_read_on_a_device_table_other_than_the_one_of_the_mount`）、回退读现行那一版与目标那一版、回退的释放之前读盘核、清扫的判与发。
     /// 读完另经 [`Self::count_the_other_unit_reads_with_one_copy_failing`] 进账、不走这一层的读：抬 F 读实例表与重算影子账、发 discard 之前读实例表、
     /// 重建分配器读最新那条根的实例表与各根的账（`mount::rebuilt_allocator`）、可写挂载取号之前逐盘核的落后支（`mount::devices_passing_the_check_before_the_acquisition`）、释放之前读盘核、搬留存单元读源、另发那一批认位置沿链读实例表、经会话读文件、写系统配置槽之前算 F 生效值
     /// （经写入口转交，`transaction::PoolWriter::count_the_reads_before_the_system_configuration_writes`）。
diff --git a/crates/singlefs-core/src/compaction_plan.rs b/crates/singlefs-core/src/compaction_plan.rs
index 9409052..bd8db2a 100644
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -637,7 +637,9 @@ pub enum CompactionRunStop {
     /// 整理这个后台任务先前因一批出错停着（D26 已定项 13 第 4 条；挂载那一刻或切换生效的那一批开头，实现照交给它的那一组一张卡都用不上而当场停着的同算），
     /// 而这一刻池子每块盘的已分配占它的容量都不超过补跑阈值：
     /// 停在这一刻：现行那一版里有活意图就单发一次只删意图的发布丢掉它（R 排他跟着撤，同 [`Self::PausedShortOfSpace`]），之后一个写都不发；
-    /// 没有活意图就从这一刻起一个写都不发（删除续做也不走）。一趟之中在 CPU 上补跑过几批之后、或一趟之中切换生效的那一批开头才判出的，
+    /// 没有活意图就从这一刻起一个写都不发（删除续做也不走）。显式入口每一趟开头判这一条；后台入口判出停着而有活意图时起跑一趟只为删它
+    /// （`mounted_session::BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`），没有活意图就不起跑。挂着切到一张卡都用不上的那一组、
+    /// 手里没有活意图时，两个入口都在任何写之前停在这一条。一趟之中在 CPU 上补跑过几批之后、或一趟之中切换生效的那一批开头才判出的，
     /// 前面那几批照常发过。等占用超过阈值（下一趟照常补跑、从头签发），或一次通过校验的切换请求解除停下。
     BackgroundTaskStoppedAfterAnEarlierComputeError,
     /// GPU 路径上 R 里除了这一轮两份核判得不一样的记录，再没有判定了可搬的（D26 已定项 13 第 8 条「这一批不回收、不搬，留到下一轮重算」）：
@@ -758,7 +760,8 @@ impl CompactionRunStop {
 }
 
 /// 一条活意图在这次挂载里的样子：意图本身与它的 R 排没排他（产品值排他；只供测试的开关 [`CompactionRegionExclusion::SwitchedOffForTests`] 不排）。
-/// 整理停下（暂停、一批出错停下）删掉意图连同排他，不留「意图在、R 不排他」那一态。
+/// 整理停下（暂停、一批出错停下）删掉意图连同排他，盘上每一版不留「意图在、R 不排他」那一态；只删意图的那一次发布自己取落点时放开 R
+/// （[`CompactionSwitchesAndTheExcludedRegion::open_the_region_to_the_publish_deleting_the_intent`]），那一次发布落盘的那一版里已没有意图。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct LiveCompactionIntent {
     pub intent: CompactionIntent,
@@ -843,6 +846,28 @@ impl CompactionSwitchesAndTheExcludedRegion {
         self.live_intent.take().map(|live| live.intent)
     }
 
+    /// 只删意图、不搬单元的那一次发布（暂停、出错停下、停着时的删意图，R 已按记录全空的删意图；`transaction::publish_compaction_step_in_the_session`）
+    /// 取落点之前放开 R：那一次发布落盘之后盘上就没有这条意图，它自己的新落点落进 R 不违 I-5.6（「活意图的 R 内没有分配代 > `c0` 的记录」只管意图活着的那几版）。
+    /// 活意图本身不撤（撤的时机照旧在删意图那一版落盘之后，会话撤）；交回放开之前排不排他，没有活意图交回 `None`。发布返回之后（发成或被拒）
+    /// 照交回的那一个经 [`Self::put_back_the_region_exclusion`] 放回，别的发布照旧排除 R。
+    pub(crate) fn open_the_region_to_the_publish_deleting_the_intent(&mut self) -> Option<bool> {
+        self.live_intent
+            .as_mut()
+            .map(|live| std::mem::replace(&mut live.is_the_region_excluded_from_placement, false))
+    }
+
+    /// 放回 [`Self::open_the_region_to_the_publish_deleting_the_intent`] 放开之前的排他。活意图已撤（或从来没有）就什么都不做。
+    pub(crate) fn put_back_the_region_exclusion(
+        &mut self,
+        was_excluded_before_the_publish: Option<bool>,
+    ) {
+        if let (Some(live), Some(was_excluded)) =
+            (self.live_intent.as_mut(), was_excluded_before_the_publish)
+        {
+            live.is_the_region_excluded_from_placement = was_excluded;
+        }
+    }
+
     /// 分配器此刻不许当目的地的 R（活意图且排他时）。
     #[must_use]
     pub fn region_excluded_from_placement(&self) -> Option<CompactionRegion> {
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 5e54d4e..c3e0699 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -4780,7 +4780,7 @@ pub fn raise_rollback_floor_to_the_admission_ceiling<Device: BlockDevice>(
     shadow_ledger: ShadowLedger,
 ) -> Result<RaiseToTheAdmissionCeiling, MountError> {
     let caller_inputs_agreeing_with_the_disk =
-        caller_inputs_agreeing_with_the_disk(parameters, devices, current)?;
+        caller_inputs_agreeing_with_the_disk(parameters, devices, allocator, current)?;
     raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk(
         &caller_inputs_agreeing_with_the_disk,
         devices,
@@ -4994,7 +4994,7 @@ pub fn push_one_floor_raise_within_the_admission_budget<Device: BlockDevice>(
     publishes_pushed: usize,
 ) -> Result<FloorRaisePushedWithinTheAdmissionBudget, MountError> {
     let caller_inputs_agreeing_with_the_disk =
-        caller_inputs_agreeing_with_the_disk(parameters, devices, current)?;
+        caller_inputs_agreeing_with_the_disk(parameters, devices, allocator, current)?;
     let parameters_of_the_pool = &caller_inputs_agreeing_with_the_disk.parameters_of_the_pool;
     let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
     let device_of_txg = |txg: CheckpointTxg| {
@@ -5344,7 +5344,7 @@ fn raise_the_floor_through<Device: BlockDevice>(
     let CallerInputsAgreeingWithTheDisk {
         system_configuration,
         parameters_of_the_pool,
-    } = caller_inputs_agreeing_with_the_disk(parameters, devices, current)?;
+    } = caller_inputs_agreeing_with_the_disk(parameters, devices, allocator, current)?;
     // 候选集按现行那一版的实例表判：从它的根指着的第 0 片沿链真读出来、解出来（C502（抬 F 时现行版本里没有实例表单元）；
     // D18（块里携带什么信息） 已定项 11：一张表可以不止一片）。不看 `TransactionOutput::units`——那是这个进程内存里的角色列表，
     // 第一个文件版本那一版起就不带实例表单元，mkfs 同一个进程里后面发多少次都一样，而表就在根指针后面、读得出也解得开。
@@ -7690,7 +7690,8 @@ struct CallerInputsAgreeingWithTheDisk {
 /// 「可见」之后再核落后支：每块盘最新那份自证系统配置不落后于调用方持着的现行那一版（`current`）、或落后而现行那一版的单元都在、
 /// 它的根住在这块盘上时那个根环槽里也是它（[`devices_behind_the_current_version`]；D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」，用户 2026-09-27 定，
 /// 代码三方 m2-closeout-code-r2「同池旧快照的盘经入口洗白」）。三样里的本盘设备号在前面盘表那一判里已比过（[`device_table_disagreeing_with`]）。
-/// 只读盘，不写盘：每个入口在它的第一个写之前调它。
+/// 只读盘，不写盘：每个入口在它的第一个写之前调它。落后支读到只坏一份的那几组经 `allocator` 的计数层进别的读那一份账
+/// （[`PoolAllocator::reading_and_counting_the_other_unit_reads_with_one_copy_failing`]；D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）。
 ///
 /// # Errors
 /// 交进来的不对（[`CallerInputsDisagreeingWithTheDisk`] 的三种；不「可见」的盘报在第二种的 `disagreeing_device_table` 里，
@@ -7702,6 +7703,7 @@ struct CallerInputsAgreeingWithTheDisk {
 fn caller_inputs_agreeing_with_the_disk<Device: BlockDevice>(
     caller_parameters: &MakeFilesystemParameters,
     devices: &Vec<(DeviceIdentity, Device)>,
+    allocator: &PoolAllocator,
     current: &TransactionOutput,
 ) -> Result<CallerInputsAgreeingWithTheDisk, CallerInputsCheckRefused> {
     refuse_device_identities_handed_in_more_than_once(devices)
@@ -7732,8 +7734,12 @@ fn caller_inputs_agreeing_with_the_disk<Device: BlockDevice>(
             },
         ));
     }
-    let devices_behind =
-        devices_behind_the_current_version(devices, &parameters_of_the_pool, current);
+    // 落后支读到窗口之外只坏一份的那几组经计数层记进别的读那一份账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」），
+    // 下一次用户改动发成之后另发的那一批换下它们；这一核拒了、调用方把分配器换回进来时那一份，账照留（各份分配器拷贝共用）。
+    let devices_behind = allocator
+        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(devices, |reader| {
+            devices_behind_the_current_version(reader, &parameters_of_the_pool, current)
+        });
     if !devices_behind.is_empty() {
         return Err(CallerInputsCheckRefused::Disagreeing(
             CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
@@ -7760,10 +7766,11 @@ fn caller_inputs_agreeing_with_the_disk<Device: BlockDevice>(
 pub(crate) fn parameters_of_the_pool_agreeing_with_the_caller_inputs<Device: BlockDevice>(
     caller_parameters: &MakeFilesystemParameters,
     devices: &Vec<(DeviceIdentity, Device)>,
+    allocator: &PoolAllocator,
     current: &TransactionOutput,
 ) -> Result<MakeFilesystemParameters, MountError> {
     Ok(
-        caller_inputs_agreeing_with_the_disk(caller_parameters, devices, current)?
+        caller_inputs_agreeing_with_the_disk(caller_parameters, devices, allocator, current)?
             .parameters_of_the_pool,
     )
 }
@@ -10873,16 +10880,16 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
     let CallerInputsAgreeingWithTheDisk {
         system_configuration,
         parameters_of_the_pool,
-    } = caller_inputs_agreeing_with_the_disk(parameters, devices, current).map_err(|refused| {
-        match refused {
+    } = caller_inputs_agreeing_with_the_disk(parameters, devices, allocator, current).map_err(
+        |refused| match refused {
             CallerInputsCheckRefused::Disagreeing(disagreeing) => {
                 RollbackError::CallerInputsDisagreeWithTheDisk(disagreeing)
             }
             CallerInputsCheckRefused::SystemConfigurationNotChosen(failure) => {
                 RollbackError::Recovery(failure)
             }
-        }
-    })?;
+        },
+    )?;
     let rollback_txg = current
         .root
         .checkpoint_txg
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 61c4123..7dc7474 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -482,7 +482,8 @@ impl BackgroundCompactionCall {
 /// 后台入口这一次为什么没起跑。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum BackgroundCompactionNotStarted {
-    /// 整理因一批出错停着（D26（后台整理与放置回收） 已定项 13 第 4 条）而池子每块盘的已分配占它的容量都不超过补跑阈值（用户 2026-10-28 定案）：
+    /// 整理因一批出错停着（D26（后台整理与放置回收） 已定项 13 第 4 条）而池子每块盘的已分配占它的容量都不超过补跑阈值（用户 2026-10-28 定案），
+    /// 现行那一版里又没有要删的活意图（有而上一次用户改动给的那一份 `B_gc` 扣完、前台写得进的也在这里，意图与 R 排他留到下一次用户改动之后那一次调用）：
     /// 「该跑了」留着，解除之后（或占用超过阈值时）下一次调用再取。
     BackgroundTaskStoppedAfterAComputeError,
     /// 没有「该跑了」，定时兜底关着。
@@ -510,6 +511,10 @@ pub enum BackgroundCompactionTrigger {
     /// 现行那一版的删除队列不空（D26（后台整理与放置回收） 已定项 13）：只走队列、不发整理，不看水位，也不被那一份 `B_gc` 挡（删除续做不占预算）。
     /// 调用方挂载之后不调任何后台入口，队列没走空时前台的写都被拒（[`AcceptRefused::QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion`]），没有别人走它。
     DeletionQueueNotEmpty,
+    /// 整理因一批出错停着而不补跑（D26（后台整理与放置回收） 已定项 13 第 4 条），现行那一版里还有活意图（补跑那几批签下的、挂载续做的、停下那一刻没删成的）：
+    /// 这一趟只单发一次删意图的发布（同显式入口一趟开头判出停着那一段，[`drop_the_live_intent_on_a_stop`]），停在
+    /// [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]；删完之后的调用照旧不起跑。预算照后台入口别的起跑给：那一份扣完而前台写得进就不起跑。
+    LiveIntentWhileStoppedAfterAComputeError,
 }
 
 /// 后台入口这一次起不起跑（[`MountedSession::run_compaction_in_the_background`] 开头那几判的结论）。封闭集合。
@@ -2581,12 +2586,14 @@ pub enum FileReadInTheSessionRefused {
     Read(FileReadFailure),
 }
 
-/// [`MountedSession::read_the_file`] 读文件之前核盘表（同 [`MountedSession::publish_user_change`] 进循环之前那几样）：只读每块盘的系统配置槽与落后那几块上现行那一版的落点。
+/// [`MountedSession::read_the_file`] 读文件之前核盘表（同 [`MountedSession::publish_user_change`] 进循环之前那几样）：只读每块盘的系统配置槽与落后那几块上现行那一版的落点；
+/// 落后支读到只坏一份的那几组经 `allocator` 的计数层进别的读那一份账（[`crate::allocator::PoolAllocator::reading_and_counting_the_other_unit_reads_with_one_copy_failing`]）。
 ///
 /// # Errors
 /// [`FileReadInTheSessionRefused`] 的盘表四样与 `NoFileVersionToRead`。
 fn refuse_a_read_on_a_device_table_other_than_the_one_of_the_mount<Device: BlockDevice>(
     parameters_and_device_table: &ParametersAndDeviceTableOfTheMount,
+    allocator: &PoolAllocator,
     current: &PoolVersion,
     devices: &[(DeviceIdentity, Device)],
 ) -> Result<(), FileReadInTheSessionRefused> {
@@ -2620,8 +2627,12 @@ fn refuse_a_read_on_a_device_table_other_than_the_one_of_the_mount<Device: Block
             FileReadInTheSessionRefused::OwnDeviceNumbersDifferFromTheDeviceTable { disagreements },
         );
     }
-    let devices_of_the_table_behind =
-        devices_behind_the_current_version(devices, parameters, current_version_of_trees);
+    // 落后的盘上窗口之外坏了的那几份经计数层记进别的读那一份账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」），
+    // 下一次用户改动发成之后另发的那一批换下它们；核不过不读文件，账照留。
+    let devices_of_the_table_behind = allocator
+        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(devices, |reader| {
+            devices_behind_the_current_version(reader, parameters, current_version_of_trees)
+        });
     if !devices_of_the_table_behind.is_empty() {
         return Err(
             FileReadInTheSessionRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
@@ -2853,6 +2864,7 @@ impl MountedSession {
     ) -> Result<FileReadOutput, FileReadInTheSessionRefused> {
         refuse_a_read_on_a_device_table_other_than_the_one_of_the_mount(
             &self.parameters_and_device_table,
+            &self.allocator,
             &self.current,
             devices,
         )?;
@@ -3226,8 +3238,11 @@ impl MountedSession {
 
     /// 整理的后台入口（D26（后台整理与放置回收） 已定项 13 第 5、6 条；用户 2026-10-26 定 GC「挪出调用线程，补齐三样保证」）：调用方在它自己的线程上
     /// 一次次调，核心层不起线程、不读钟（钟与前台的 fsync 由 `caller` 答）。每次调用先看起不起跑，依次：
-    /// 1. 整理因一批出错停着（第 4 条）而池子每块盘的已分配占它的容量都不超过补跑阈值 ⇒ 不起跑，「该跑了」留着，一个写都不发（活意图出错停下那一刻已删；
-    ///    这一判之前才停着而还有活意图的，意图与 R 排他照留，等下一趟显式入口、补跑或解除停下）；
+    /// 1. 整理因一批出错停着（第 4 条）而池子每块盘的已分配占它的容量都不超过补跑阈值：现行那一版里有活意图（补跑那几批签下的、挂载续做的、
+    ///    停下那一刻没删成的）而上一次用户改动给的那一份 `B_gc` 还剩（或全空段低于底额、前台写不进）⇒ 起跑一趟只单发一次删意图的发布
+    ///    （[`BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`]，同显式入口一趟开头判出停着那一段），停在
+    ///    [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]；没有活意图、或那一份扣完而前台写得进 ⇒ 不起跑，「该跑了」留着，
+    ///    一个写都不发（删完意图之后的调用都走这一支）。挂着切到一张卡都用不上的那一组、手里没有活意图时，这一次就在任何写之前报停着；
     ///    超过阈值就照下面几条照常判起不起跑，起跑之后每一批在 CPU 上补跑（用户 2026-10-28 定案）；
     /// 2. 用户改动之后起跑判据记下了「该跑了」⇒ 取走，照用户改动之后那一趟推（判决 G8 丁：至多一批）；
     /// 3. 没有 ⇒ 看定时兜底（`pacing.timer_fallback`）：关着、或离上一次查还不到一个间隔就不起跑；到了就再查一次起跑判据，为真就推一趟到停；
@@ -3241,7 +3256,7 @@ impl MountedSession {
     ///    （[`CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates`]）。
     ///
     /// 整理没停着而现行那一版的删除队列不空：排在第 2、3、4 条之前起跑（[`BackgroundCompactionTrigger::DeletionQueueNotEmpty`]），只走队列、不发整理，不看水位、不被预算挡，「该跑了」与定时兜底都不动；
-    /// 走空之后下一次调用照上面几条判。整理停着时队列也不走（第 1 条：一个写都不发）。
+    /// 走空之后下一次调用照上面几条判。整理停着时队列也不走（第 1 条：至多单发一次删意图，队列照抄）。
     ///
     /// 起跑之后每一批之前问调用方：前台有 fsync 在等就不发下一批（[`CompactionRunStop::YieldedToAForegroundFsync`]；全空段低于底额时这一趟的第一次发布除外），这一个写带宽窗口里
     /// 整理写的字节到了 `pacing.write_bandwidth_limit` 也不发（[`CompactionRunStop::WriteBandwidthLimitReached`]）；用户改动之后起跑的那一趟
@@ -3377,8 +3392,11 @@ impl MountedSession {
         let parameters = parameters_and_device_table.parameters_on_disk();
         let background_compute = parameters_and_device_table.background_compute_of_this_mount();
         if compaction_stays_stopped(allocator, background_compute) {
-            return Ok(BackgroundCompactionStart::NotStarted(
-                BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
+            return Ok(start_while_the_compaction_stays_stopped(
+                allocator,
+                current,
+                parameters.geometry.root_ring_slots_per_region.count(),
+                compaction.publishes_since_the_last_user_change,
             ));
         }
         let is_judged_as_the_product_does = matches!(
@@ -3627,7 +3645,8 @@ impl MountedSession {
                     }
                     BackgroundCompactionTrigger::DueAfterAUserChange
                     | BackgroundCompactionTrigger::TimerFallback
-                    | BackgroundCompactionTrigger::DeletionQueueNotEmpty => {}
+                    | BackgroundCompactionTrigger::DeletionQueueNotEmpty
+                    | BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError => {}
                 }
             }
         }
@@ -3975,6 +3994,7 @@ impl MountedSession {
         let parameters_of_the_pool = parameters_of_the_pool_agreeing_with_the_caller_inputs(
             parameters_on_disk,
             devices,
+            allocator,
             &start,
         )
         .map_err(|refusal| RootRingPushRefused::DevicesHandedIn(Box::new(refusal)))?;
@@ -4273,7 +4293,8 @@ struct CompactionPublishAdvanced {
 }
 
 /// 整理此刻停着而不补跑（D26（后台整理与放置回收） 已定项 13 第 4 条：GC 因一批出错停着、池子每块盘的已分配占它的容量都不超过补跑阈值）：
-/// 交回真；没停着或要在 CPU 上补跑交回假。只判，分配器与盘一样都不动：停着时还有活意图的由推整理的那一步删（[`drop_the_live_intent_on_a_stop`]）。
+/// 交回真；没停着或要在 CPU 上补跑交回假。只判，分配器与盘一样都不动：停着时还有活意图的由一趟开头那一判单发一次删意图（[`drop_the_live_intent_on_a_stop`]）；
+/// 后台入口判出停着而有活意图时起跑那一趟（[`start_while_the_compaction_stays_stopped`]），显式入口每一趟都走到那一判。
 fn compaction_stays_stopped(
     allocator: &PoolAllocator,
     background_compute: &BackgroundComputeOfThisMount,
@@ -4288,6 +4309,42 @@ fn compaction_stays_stopped(
     }
 }
 
+/// 后台入口判出整理停着而不补跑（[`compaction_stays_stopped`]）之后起不起跑：现行那一版里有活意图，而上一次用户改动给的那一份 `B_gc`
+/// 还剩（或全空段低于底额、前台写不进时破例至少 1 次，同后台入口别的起跑）⇒ 起跑一趟
+/// （[`BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`]），那一趟开头判出同一个停着，只单发一次删意图
+/// （[`drop_the_live_intent_on_a_stop`]），之后停在 [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]；
+/// 没有活意图、或那一份扣完而前台写得进 ⇒ 不起跑、一个写都不发，「该跑了」留着。只判，分配器与盘一样都不动。
+fn start_while_the_compaction_stays_stopped(
+    allocator: &PoolAllocator,
+    current: &PoolVersion,
+    root_ring_slots_per_region: u64,
+    publishes_since_the_last_user_change: u64,
+) -> BackgroundCompactionStart {
+    let has_a_live_intent = current.version_of_trees().is_some_and(|version_of_trees| {
+        live_compaction_intent_of_the_entries(&version_of_trees.logged_operations).is_some()
+    });
+    let foreground_cannot_write = foreground_cannot_write_under_the_switches(allocator);
+    let is_the_budget_spent = publishes_left_of_the_budget_given_by_the_last_user_change(
+        root_ring_slots_per_region,
+        publishes_since_the_last_user_change,
+    ) == 0;
+    if !has_a_live_intent || (is_the_budget_spent && !foreground_cannot_write) {
+        return BackgroundCompactionStart::NotStarted(
+            BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
+        );
+    }
+    BackgroundCompactionStart::Starts {
+        trigger: BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError,
+        reach: CompactionRunReach::AtMostOneBatchAfterAUserChange,
+        budget: if foreground_cannot_write {
+            CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite
+        } else {
+            CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange
+        },
+        batches_published_earlier_in_the_trip: 0,
+    }
+}
+
 /// 整理一趟（显式入口、可写挂载续做共用；后台入口另经 [`run_compaction_on_with_the_region_check_of_the_intent_deletion`]）：先走空删除续做（G8 次序），
 /// 再照预算一次一次推（[`advance_compaction_by_one_publish`]）。预算 `B_gc` 数的是整理的每一次发布（签发、搬一批、删意图：每一次都用掉一个根槽）。
 /// 每一批之前不看调用方。discard 由调用方在这一趟之后发。
@@ -5872,14 +5929,16 @@ fn pause_by_dropping_the_live_intent_or_hand_up<Device: BlockDevice>(
 
 /// 整理停下时删掉现行那一版里的活意图（被空间拒的暂停、一批出错放弃、停着而不补跑三处共用）：单发一次只删意图的发布
 /// （[`CompactionIntentStep::Drop`]，不搬单元），发成之后撤分配器这一侧的活意图连同 R 排他、记一次丢意图，现行那一版换成删意图那一版，
-/// 交回那一次发布（种类 [`CompactionPublishKind::IntentDroppedOnAStop`]，这一趟随后停在 `stop`）。R 排他一直留到删意图那一版落盘：
-/// 盘上每一版要么意图活着而 R 排他着，要么意图已不在（I-5.6「活意图的 R 内没有分配代 > `c0` 的记录」），没有「意图在、R 不排他」那一态。
+/// 交回那一次发布（种类 [`CompactionPublishKind::IntentDroppedOnAStop`]，这一趟随后停在 `stop`）。分配器这一侧的活意图一直留到删意图那一版落盘，
+/// R 排他对别的发布照留；删意图那一次发布自己取落点时放开 R（`transaction::publish_compaction_step_in_the_session`，
+/// R 之外没有空槽而 R 里有时照样删得掉）：它落盘的那一版里已没有意图。盘上每一版要么意图活着而 R 里没有分配代 > `c0` 的记录，
+/// 要么意图已不在（I-5.6「活意图的 R 内没有分配代 > `c0` 的记录」）。
 /// 只删意图、不搬单元的整理发布归删除类：式子那一判不判、落点那一判照判（D28（挂载期承诺量） 已定项 1 的不判名单，
 /// `transaction::judge_the_publish_before_any_read_or_write`）。
 /// 现行那一版里没有活意图就什么都不写，交回停在 `stop`。
 ///
 /// # Errors
-/// 删意图那一次没发成（落点取不到，或发布路径的错）：原样交回。取落点被拒在任何写之前，盘上逐字节不变，意图留在盘上、R 照旧排他，
+/// 删意图那一次没发成（R 里外都取不到落点，或发布路径的错）：原样交回。取落点被拒在任何写之前，盘上逐字节不变，意图留在盘上、R 照旧排他，
 /// 下一次整理再走到停下时再删。
 fn drop_the_live_intent_on_a_stop<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
@@ -7449,7 +7508,9 @@ impl MountedSessionWithPendingChanges {
                                 CompactionStartJudgement::DueAfterAUserChange;
                         }
                         BackgroundCompactionTrigger::TimerFallback
-                        | BackgroundCompactionTrigger::DeletionQueueNotEmpty => {}
+                        | BackgroundCompactionTrigger::DeletionQueueNotEmpty
+                        | BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError => {
+                        }
                     },
                 }
                 // 用户 2026-10-30 定每次叫起都报：判过起跑之后排空被拒的这一次也是一次叫起，照数、照报。
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 3cdd82b..11de38b 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -10772,6 +10772,8 @@ pub enum RegionCheckOfTheIntentDeletion {
 /// 整理的一次发布（[`publish_relocation_batch`] 的一般形）：搬 `planned` 里那几个单元（可以一个都没有），同一次发布对意图做 `intent_step`
 /// （判决 G1：意图条目写进 logged_ops 树，签发那次发布写、最后一批同发布删；D8（核心索引结构） 已定项 5 三态表：签发之前崩 = 没有意图，
 /// 签发之后、最后一批之前崩 = 意图还在、挂载续做，最后一批之后 = 意图没了且目标态达成）。别的同 [`publish_relocation_batch`]。
+/// 只删意图、不搬单元的那一次（删意图不带一批、丢意图）取落点时不排除活意图的 R（[`publish_compaction_step_in_the_session`]），
+/// 删意图那一核不数它自己在 R 里的新落点；带一批的照旧排除 R。
 ///
 /// 签发那次写下的 `segs0` 就是调用方交进来的那条意图里的：签发那次发布之前的全空聚簇段数（判决
 /// `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H3，用户 2026-10-16 定「取签发之前」；会话按上一版的分配记录数，
@@ -10947,6 +10949,11 @@ pub(crate) fn publish_compaction_step_in_the_session<Device: BlockDevice>(
             CompactionStepCheckBeforePersisting::RegionEmptyByTheRecords {
                 region: intent.region,
                 region_check: region_check_of_the_intent_deletion,
+                placements_of_this_publish: if planned.is_empty() {
+                    PlacementsOfThisPublishInTheRegion::NotCountedTheRegionIsOpenToThisPublish
+                } else {
+                    PlacementsOfThisPublishInTheRegion::CountedAsRecordsInTheRegion
+                },
             },
         ),
         (CompactionIntentStep::Drop, Some((issued_txg, _))) => (
@@ -10955,7 +10962,25 @@ pub(crate) fn publish_compaction_step_in_the_session<Device: BlockDevice>(
         ),
     };
     let relocations = relocations_checked_against_the_version(pool, allocator, previous, planned)?;
-    publish_compaction_step_after_its_check(
+    // 只删意图、不搬单元的那一次（删意图那一支不带一批、丢意图）取落点时放开 R：它落盘的那一版里已没有意图，排他只挡得住它自己，
+    // R 之外取不到落点时意图就删不掉、R 一直排他（D26（后台整理与放置回收） 已定项 2；D28（挂载期承诺量） 已定项 1 的结构判据「只删整理意图、不搬单元」）。
+    // 带一批的照旧排除 R：搬出 R 的单元不许落回 R。撤分配器侧的活意图照旧由会话在这一版落盘之后做；发成、被拒都在返回之前放回排他。
+    let deletes_only_the_intent = planned.is_empty()
+        && match intent_step {
+            CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords
+            | CompactionIntentStep::Drop => true,
+            CompactionIntentStep::CarryTheIntent
+            | CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion
+            | CompactionIntentStep::Issue(_) => false,
+        };
+    let exclusion_before_the_publish = if deletes_only_the_intent {
+        allocator
+            .compaction_mut()
+            .open_the_region_to_the_publish_deleting_the_intent()
+    } else {
+        None
+    };
+    let published = publish_compaction_step_after_its_check(
         pool,
         allocator,
         previous,
@@ -10963,7 +10988,11 @@ pub(crate) fn publish_compaction_step_in_the_session<Device: BlockDevice>(
         logged_operations,
         check,
         instance,
-    )
+    );
+    allocator
+        .compaction_mut()
+        .put_back_the_region_exclusion(exclusion_before_the_publish);
+    published
 }
 
 /// 计划里每一项对着上一版核过、带上搬它要的东西（[`relocation_checked_against_the_version`]）。按留存条目读源读到只坏一份的进这次挂载别的读那一份账
@@ -11071,10 +11100,11 @@ pub(crate) fn publish_the_relocations_and_the_repair_of_the_exempt_units<Device:
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum CompactionStepCheckBeforePersisting {
     NothingToCheck,
-    /// 删意图：R 按记录全空（停机谓词，判决 G4），照 `region_check` 判（产品路径恒照记录）。
+    /// 删意图：R 按记录全空（停机谓词，判决 G4），照 `region_check` 判（产品路径恒照记录）；这次发布自己在 R 里的新落点算不算照 `placements_of_this_publish`。
     RegionEmptyByTheRecords {
         region: CompactionRegion,
         region_check: RegionCheckOfTheIntentDeletion,
+        placements_of_this_publish: PlacementsOfThisPublishInTheRegion,
     },
     /// 照抄意图，依次核：R 还有一条未释放的记录（判决 m4-segment-code-r1 第三节 F2：这一批腾空了 R 就同一次发布删意图）；
     /// 再照 `planned_units_rewritten_anyway` 核计划里每个单元都真搬了（判决 m4-segment-code-r2 第三节 H4）。
@@ -11139,6 +11169,16 @@ fn publish_compaction_step_after_its_check<Device: BlockDevice>(
     )
 }
 
+/// 删意图那一核（[`CompactionStepCheckBeforePersisting::RegionEmptyByTheRecords`]）数不数这次发布自己在 R 里的新落点。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum PlacementsOfThisPublishInTheRegion {
+    /// 带一批的删意图：取落点照旧排除 R，这次发布在 R 里有一条分配代是它的记录也是 R 不空（只供测试的开关关掉排他时走得到）。
+    CountedAsRecordsInTheRegion,
+    /// 不带一批的删意图：取落点放开了 R（[`publish_compaction_step_in_the_session`]），分配代是这次发布的记录是它落盘那一版自己的节点，
+    /// 那一版里已没有意图，不算 R 不空；上一版就在 R 里的记录照算。
+    NotCountedTheRegionIsOpenToThisPublish,
+}
+
 /// 整理的一次发布落盘之前那一核（[`CompactionStepCheckBeforePersisting`]）：在这次发布算好的这一版上判，过了交 `Ok`，不过交给调用方的拒绝。
 /// 只读这一版与计划，不读盘：判定与随后落盘的读的是同一份算好的账。
 fn compaction_step_check_on_the_version_after_this_publish(
@@ -11151,13 +11191,35 @@ fn compaction_step_check_on_the_version_after_this_publish(
         CompactionStepCheckBeforePersisting::RegionEmptyByTheRecords {
             region,
             region_check,
+            placements_of_this_publish,
         } => {
             let is_the_region_empty_by_the_records = match region_check {
                 RegionCheckOfTheIntentDeletion::JudgedByTheRecords => {
-                    region_is_empty_by_the_records(
-                        &version_after_this_publish.allocation_records,
-                        region,
-                    )
+                    match placements_of_this_publish {
+                        PlacementsOfThisPublishInTheRegion::CountedAsRecordsInTheRegion => {
+                            region_is_empty_by_the_records(
+                                &version_after_this_publish.allocation_records,
+                                region,
+                            )
+                        }
+                        PlacementsOfThisPublishInTheRegion::NotCountedTheRegionIsOpenToThisPublish => {
+                            let txg_of_this_publish = version_after_this_publish.root.checkpoint_txg;
+                            let records_from_before_this_publish: Vec<AllocationRecord> =
+                                version_after_this_publish
+                                    .allocation_records
+                                    .iter()
+                                    .filter(|record| {
+                                        record.is_released
+                                            || record.generation != txg_of_this_publish
+                                    })
+                                    .copied()
+                                    .collect();
+                            region_is_empty_by_the_records(
+                                &records_from_before_this_publish,
+                                region,
+                            )
+                        }
+                    }
                 }
                 RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests => {
                     false
```

## 二、mounted_read.rs 相对 HEAD 的全部改动（早先几批合入，代码轮 m4-close-fixes-code r1/r2 正文提到而点名一节没列）

```diff
diff --git a/crates/singlefs-core/src/mounted_read.rs b/crates/singlefs-core/src/mounted_read.rs
index d63934e42..6c3bb7981 100644
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -45,16 +45,17 @@ use crate::records::{
 };
 use crate::recovery::{
     choose_root, choose_system_configuration_naming_devices_without_a_mountable_slot,
+    choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot,
     fall_back_judged_on_the_three_readers,
     inode_record_is_not_below_the_separator_key_and_the_container,
     inode_records_of_the_leaf_ascend_after_the_leaves_before,
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
@@ -368,6 +369,8 @@ pub struct MountedPoolForRead {
 }
 
 /// 打开挂载态：把根记录到树表条目到两棵用户可见的树这一段读回内存，之后按偏移读不再碰它们。
+/// 这是只读语义的那一条路：读者读到只坏一份只计数（[`ReadPathObservation::data_unit_reads_with_one_copy_failing`]、报给读者）、不搬，不看同一个池上开没开着可写挂载；
+/// 可写挂载期间用户读文件经 `mounted_session::MountedSession::read_the_file`（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）。
 ///
 /// 读的次序：中央映射树（自举豁免，只按父指针里的位置条目读，D19（块指针的结构与宽度预算） 已定项 8；多层时整棵读进来）→ 树表（同样豁免）
 /// → 树表条目 → extent 树根兼叶 → inode 树根 → 每一片 inode 叶容器。
@@ -570,9 +573,9 @@ pub fn open_pool_for_read(
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
@@ -641,14 +644,43 @@ pub struct MountedReadOnly {
 ///
 /// **扫环只在这一步发生一次**：之后每一次 [`OpenFileForRead::read_at`] 都不碰 journal 环。
 /// 崩溃点重放的每个恢复结果上也能跑这一条——`reader` 换成那个崩溃镜像即可（并行线二验收第 3 条；
-/// 枚举那一半归验证包，由主 agent 经门禁 crash-point-replay 跑）。
+/// 枚举那一半归验证包，由主 agent 经门禁 crash-point-replay 跑）。系统配置不筛 fsid；管理员知道本池 fsid 的用 [`mount_read_only_of_the_pool`]。
 ///
 /// # Errors
 /// 见 [`MountReadOnlyFailure`]。
 pub fn mount_read_only(reader: &dyn PoolReader) -> Result<MountedReadOnly, MountReadOnlyFailure> {
-    let system_configuration_choice =
+    mount_read_only_from_the_system_configuration_choice(
+        reader,
         choose_system_configuration_naming_devices_without_a_mountable_slot(reader)
-            .map_err(MountReadOnlyFailure::Recovery)?;
+            .map_err(MountReadOnlyFailure::Recovery)?,
+    )
+}
+
+/// 带本池 fsid 的只读挂载：与 [`mount_read_only`] 同一条路径，系统配置只从自述 fsid 是 `filesystem_identifier_of_the_pool` 的槽里择
+/// （`recovery::choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot`，与可写挂载同一个筛法）：
+/// 别的池的槽世代号再大也择不到，一块盘两槽都是别的池的就跳过这块盘。一个字节都不写。
+///
+/// # Errors
+/// 见 [`MountReadOnlyFailure`]；盘上没有一槽是这个 fsid 的池的报 `Recovery(RecoveryFailure::NoValidSystemConfiguration)`，不读别的池。
+pub fn mount_read_only_of_the_pool(
+    reader: &dyn PoolReader,
+    filesystem_identifier_of_the_pool: &[u8; 16],
+) -> Result<MountedReadOnly, MountReadOnlyFailure> {
+    mount_read_only_from_the_system_configuration_choice(
+        reader,
+        choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot(
+            reader,
+            filesystem_identifier_of_the_pool,
+        )
+        .map_err(MountReadOnlyFailure::Recovery)?,
+    )
+}
+
+/// [`mount_read_only`] 与 [`mount_read_only_of_the_pool`] 共用的路径：系统配置已经择过。
+fn mount_read_only_from_the_system_configuration_choice(
+    reader: &dyn PoolReader,
+    system_configuration_choice: crate::recovery::SystemConfigurationChoice,
+) -> Result<MountedReadOnly, MountReadOnlyFailure> {
     let system_configuration = system_configuration_choice.system_configuration;
     let chosen_root = choose_root(reader, &system_configuration)
         .ok_or(MountReadOnlyFailure::Recovery(RecoveryFailure::NoValidRoot))?;
@@ -994,7 +1026,7 @@ impl OpenFileForRead<'_> {
             &mut observation,
         );
         // 读到一半报错（后面一个单元两份都坏、映射里没有、自描述对不上……）的那一次，报错之前已经发出的读、已经读到的只坏一份照样并进打开之后的累计：
-        // 与写入口「改动被拒时那几次读也算」同一种口径——读是真发到盘上的、坏的那一份是真读到的，计不计不看这一次调用成没成
+        // 读是真发到盘上的、坏的那一份是真读到的，计不计不看这一次调用成没成。这个累计不去重：一次调用里一个单元只读一遍，跨调用相加。
         // 这一次读自己的观测随报错丢掉，只留在累计里。
         self.mount.add_to_observation_since_open(observation);
         Ok(FileReadOutput {
```
