# m4-gpugc-mount-runtime-code-r3：相对第二轮冻结副本的产品代码改动

含别的会话同一时段的改动，当上下文读、不攻。

## crates/singlefs-mount-runtime/src/lib.rs

```diff
--- a/crates/singlefs-mount-runtime/src/lib.rs
+++ b/crates/singlefs-mount-runtime/src/lib.rs
@@ -7,7 +7,8 @@
 //!
 //! 线程体之外有同步的 [`MountedRuntime::run_one_maintenance_pass`]：确定性测试不起线程，一拍一拍推，录制盘（不能跨线程）也走得通。
 //! 一个维护线程、两个到期时刻：`publish_poll_period` 到期调 `publish_when_due`（D16（发布语义） 丢失窗口里的 P），`compaction_period` 到期推后台整理。
-//! 每一拍（两个到期时刻都算）先重发冻结的发布：写盘报错之后前台的写都被拒，等不到整理到期。
+//! 每一拍（两个到期时刻都算）先重发冻结的发布：写盘报错之后前台的写都被拒，等不到整理到期。前台 fsync（[`MountedRuntime::publish_pending_now`]）持锁时冻结着也先重发一次：
+//! 重发没成就交回错误，不答成功（D16（发布语义） 已定项 7）。
 //!
 //! 卸载等线程、重发冻结的发布、排空待发、走核心层的卸载；重发还失败就交回错误，盘交回，等价于崩溃（[`UnmountRefused::FrozenPublishResendFailed`]）。
 //! 丢弃（Drop）只停线程，不写盘，等价于崩溃（下一次挂载走恢复）。线程或前台在锁里 panic 等价于崩溃：
@@ -32,14 +33,16 @@
 };
 use singlefs_core::block_device::BlockDevice;
 use singlefs_core::compaction_plan::{
-    CallerMonotonicClockReading, CallerOfTheBackgroundCompaction, CompactionMountParameters,
-    CompactionPacing, CompactionRunStop, CompactionTimerFallback, CompactionWriteBandwidthLimit,
-    ForegroundFsync, COMPACTION_WRITE_BANDWIDTH_LIMIT_BY_DEFAULT,
+    AnswerOfTheCallerBeforeTheNextBatch, CallerMonotonicClockReading,
+    CallerOfTheBackgroundCompaction, CompactionMountParameters, CompactionPacing,
+    CompactionRunStop, CompactionTimerFallback, CompactionWriteBandwidthLimit,
+    COMPACTION_WRITE_BANDWIDTH_LIMIT_BY_DEFAULT,
 };
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
 use singlefs_core::mount::{
     mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_parameters,
-    unmount, MountError, MountOutput, Unmounted, WritableMountWithBackgroundComputeRefused,
+    unmount, MountError, MountOutput, RaisedFloor, Unmounted,
+    WritableMountWithBackgroundComputeRefused,
 };
 use singlefs_core::mounted_session::{
     AcceptRefused, BackgroundCompactionAfterThePendingChangesRefused, BackgroundCompactionCall,
@@ -169,10 +172,12 @@
     CompactionCallReturned(&'event BackgroundCompactionCall),
     /// 后台入口这一次调用没跑（盘表不对、现行那一版没有树、排空被拒）。
     CompactionCallRefused(&'event BackgroundCompactionAfterThePendingChangesRefused),
-    /// 一批写盘报错把发布路径冻结了，这一次重发成了：前台的写从此放行。
+    /// 写盘报错把发布路径冻结了，这一次重发成了（前台 fsync 那一次也算）：前台的写不再因冻结被拒；重发的是 unlink 的意图时还要等删除队列走空（后台入口因队列不空起跑）。
     FrozenPublishResent,
     /// 冻结着的那一次这一拍重发又报错：照旧冻结，前台的写照旧被拒（`AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet`），整理这一拍不跑。
     FrozenPublishResendFailed(&'event PublishError),
+    /// 前台 [`MountedRuntime::accept`] 这一次调用为空间推成了这几串抬 F 的空发布（收没收下都报）：调用方看不到发布点的接收方靠它知道根环走了几格。
+    FloorRaisesPushedByAnAccept(&'event [RaisedFloor]),
     /// 维护线程 panic 了：等价于崩溃，运行库从此报 [`RuntimeFailed`]。
     MaintenanceThreadPanicked { description: &'event str },
 }
@@ -195,8 +200,10 @@
 pub struct RuntimeObservations {
     pub maintenance_passes: u64,
     pub compaction_calls: u64,
-    /// 后台入口发成的整理发布次数。
+    /// 后台入口发成的整理发布次数（签发、搬一批、删意图、丢意图），不含删除续做。
     pub compaction_publishes: u64,
+    /// 后台入口发成的删除续做步数。
+    pub deletion_continuation_publishes: u64,
     /// 维护线程发成的排空待发次数。
     pub pending_publishes: u64,
     pub alarms_received: u64,
@@ -205,13 +212,20 @@
     pub last_stop: Option<CompactionRunStop>,
     pub frozen_publishes_resent: u64,
     pub frozen_publish_resends_failed: u64,
+    /// 前台 `accept` 为空间推成的抬 F 串数。
+    pub floor_raises_pushed_by_accepts: u64,
+    /// 那几串一共发了几次空发布（占根环的格数）。
+    pub empty_publishes_pushed_by_accepts: u64,
 }
 
 /// 同步的一拍做成了什么（[`MountedRuntime::run_one_maintenance_pass`]）。
 #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
 pub struct MaintenancePass {
     pub compaction_calls: u32,
+    /// 这一拍的整理发布次数，不含删除续做。
     pub compaction_publishes: u64,
+    /// 这一拍走的删除续做步数。
+    pub deletion_continuation_publishes: u64,
     /// 这一趟被节奏截住、或到了一拍的调用次数上限而没完：下一拍不等一个周期。
     pub is_a_trip_under_way: bool,
 }
@@ -299,7 +313,7 @@
         devices: Vec<(DeviceIdentity, Device)>,
         failure: RuntimeFailed,
     },
-    /// 卸载先重发冻结的那一次，重发又报错：盘交回，内存不接着用，等价于崩溃（冻结的那一次没落盘，下一次挂载走恢复，回到它之前最后发成的那一版）。
+    /// 卸载先重发冻结的那一次，重发又报错：盘交回，内存不接着用，等价于崩溃。那一次可能成了也可能没成（D16（发布语义） 已定项 17）：报错可能在根落盘之后，下一次挂载走恢复，读现行根判。
     FrozenPublishResendFailed {
         devices: Vec<(DeviceIdentity, Device)>,
         refusal: Box<PublishError>,
@@ -372,13 +386,13 @@
         self.clock.reading()
     }
 
-    fn foreground_fsync(&self) -> ForegroundFsync {
+    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
         if self.stopping.load(Ordering::SeqCst) {
-            ForegroundFsync::CallerIsStoppingItsThread
+            AnswerOfTheCallerBeforeTheNextBatch::CallerIsStoppingItsThread
         } else if self.fsync_waiters.load(Ordering::SeqCst) > 0 {
-            ForegroundFsync::Waiting
+            AnswerOfTheCallerBeforeTheNextBatch::AForegroundFsyncIsWaiting
         } else {
-            ForegroundFsync::NoneWaiting
+            AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
         }
     }
 }
@@ -460,6 +474,40 @@
         self.receiver.maintenance_event(event);
     }
 
+    /// 记下一次 `accept` 推成的几串抬 F，一串都没推不记（锁已放开：事件不在锁里送）。
+    fn note_the_floor_raises_pushed_by_an_accept(&self, floor_raises: &[RaisedFloor]) {
+        if floor_raises.is_empty() {
+            return;
+        }
+        let empty_publishes: usize = floor_raises
+            .iter()
+            .map(|raised| raised.publishes.len())
+            .sum();
+        self.update_the_observations(|observations| {
+            observations.floor_raises_pushed_by_accepts +=
+                u64::try_from(floor_raises.len()).expect("一次准入至多推几串");
+            observations.empty_publishes_pushed_by_accepts +=
+                u64::try_from(empty_publishes).expect("一次准入至多推几次空发布");
+        });
+        self.deliver(&MaintenanceEvent::FloorRaisesPushedByAnAccept(floor_raises));
+    }
+
+    /// 记下一次重发成了（锁已放开：事件不在锁里送）。
+    fn note_a_frozen_publish_resent(&self) {
+        self.update_the_observations(|observations| {
+            observations.frozen_publishes_resent += 1;
+        });
+        self.deliver(&MaintenanceEvent::FrozenPublishResent);
+    }
+
+    /// 记下一次重发又报错（锁已放开：事件不在锁里送）。
+    fn note_a_frozen_publish_resend_failed(&self, refusal: &PublishError) {
+        self.update_the_observations(|observations| {
+            observations.frozen_publish_resends_failed += 1;
+        });
+        self.deliver(&MaintenanceEvent::FrozenPublishResendFailed(refusal));
+    }
+
     fn caller(&self) -> CallerOfTheRuntime<'_> {
         CallerOfTheRuntime {
             clock: &*self.clock,
@@ -498,17 +546,11 @@
         drop(locked);
         match outcome {
             Ok(_resent) => {
-                self.update_the_observations(|observations| {
-                    observations.frozen_publishes_resent += 1;
-                });
-                self.deliver(&MaintenanceEvent::FrozenPublishResent);
+                self.note_a_frozen_publish_resent();
                 Ok(false)
             }
             Err(refusal) => {
-                self.update_the_observations(|observations| {
-                    observations.frozen_publish_resends_failed += 1;
-                });
-                self.deliver(&MaintenanceEvent::FrozenPublishResendFailed(&refusal));
+                self.note_a_frozen_publish_resend_failed(&refusal);
                 Ok(true)
             }
         }
@@ -572,11 +614,15 @@
                     false
                 }
                 BackgroundCompactionCall::Ran { run, .. } => {
-                    let publishes =
-                        u64::try_from(run.versions_published.len()).expect("一次调用的发布数");
+                    let deletion_steps = run.deletion_steps_published;
+                    let publishes = u64::try_from(run.versions_published.len())
+                        .expect("一次调用的发布数")
+                        - deletion_steps;
                     pass.compaction_publishes += publishes;
+                    pass.deletion_continuation_publishes += deletion_steps;
                     self.update_the_observations(|observations| {
                         observations.compaction_publishes += publishes;
+                        observations.deletion_continuation_publishes += deletion_steps;
                     });
                     match &run.outcome {
                         CompactionRunOutcome::Stopped(stop) => {
@@ -586,6 +632,7 @@
                             !matches!(
                                 stop,
                                 CompactionRunStop::YieldedToAForegroundFsync
+                                    | CompactionRunStop::YieldedToTheCallerStoppingItsThread
                                     | CompactionRunStop::WriteBandwidthLimitReached { .. }
                             )
                         }
@@ -828,7 +875,10 @@
         }
     }
 
-    /// 前台收下一次用户改动（`MountedSessionWithPendingChanges::accept`）。
+    /// 前台收下一次用户改动（`MountedSessionWithPendingChanges::accept`）。被空间拒时这一次调用当场推抬 F 再判（D16（发布语义） 已定项 1「准入」那一行）：
+    /// 收下之前可能发抬 F 的空发布，一次准入至多 B − 1 次，全程持着会话那把锁（后台批与别的前台入口等到它返回；发布之间不放锁，推的那一串与再判之间不插别的发布），
+    /// 写盘的时长算进这一次调用。推成的报在 [`ChangeAccepted`] 的 `floor_raises_pushed_for_space`（不进 `publishes`：空发布不改用户可见状态），
+    /// 没收下时报在 `AcceptRefused::floor_raises_pushed_by_this_accept`；连同观测 `floor_raises_pushed_by_accepts`、`empty_publishes_pushed_by_accepts` 与事件 [`MaintenanceEvent::FloorRaisesPushedByAnAccept`]。
     ///
     /// # Errors
     /// 核心层拒的原样；运行库坏了报 `Failed`。
@@ -842,15 +892,21 @@
             .lock_the_session()
             .map_err(RuntimeRefused::Failed)?;
         let SessionAndDevices { session, devices } = &mut *locked;
-        session
-            .accept(devices, change, now)
-            .map_err(RuntimeRefused::Refused)
+        let accepted = session.accept(devices, change, now);
+        drop(locked);
+        self.shared
+            .note_the_floor_raises_pushed_by_an_accept(match &accepted {
+                Ok(accepted) => &accepted.floor_raises_pushed_for_space.floor_raises,
+                Err(refused) => refused.floor_raises_pushed_by_this_accept(),
+            });
+        accepted.map_err(RuntimeRefused::Refused)
     }
 
     /// fsync 走这里（`MountedSessionWithPendingChanges::publish_pending_now`）：进门先登记「有 fsync 在等」（取锁之前），整理在下一批之前让路，每次发布之间都放锁。
+    /// 持锁时冻结着一次发布就先重发一次（D16（发布语义） 已定项 7）：重发成了照常排空待发，答成功（待发空着时答 `None`，那一次已落盘）；重发没成交回错误，照旧冻结着。
     ///
     /// # Errors
-    /// 同 [`Self::accept`]。
+    /// 同 [`Self::accept`]；重发又报错时 `Refused` 里是 [`UserChangeRefused::Publish`]，`cause` 是重发报的错：那一次可能成了也可能没成（D16（发布语义） 已定项 17）。
     pub fn publish_pending_now(
         &self,
     ) -> Result<
@@ -864,9 +920,26 @@
             .lock_the_session()
             .map_err(RuntimeRefused::Failed)?;
         let SessionAndDevices { session, devices } = &mut *locked;
-        session
-            .publish_pending_now(devices, now)
-            .map_err(RuntimeRefused::Refused)
+        let was_frozen = session.session().allocator.frozen_publish().is_some();
+        if was_frozen {
+            if let Err(cause) = session.resend_the_frozen_publish(devices) {
+                drop(locked);
+                self.shared.note_a_frozen_publish_resend_failed(&cause);
+                return Err(RuntimeRefused::Refused(PublishOfThePendingChangesRefused {
+                    refusal: Box::new(UserChangeRefused::Publish {
+                        cause,
+                        floor_raises: Vec::new(),
+                        versions_published: Vec::new(),
+                    }),
+                }));
+            }
+        }
+        let published = session.publish_pending_now(devices, now);
+        drop(locked);
+        if was_frozen {
+            self.shared.note_a_frozen_publish_resent();
+        }
+        published.map_err(RuntimeRefused::Refused)
     }
 
     /// 单发一次用户改动（`MountedSessionWithPendingChanges::publish_user_change`）。
@@ -970,7 +1043,7 @@
     /// 活意图不用做完：留在盘上，下次挂载续做。
     ///
     /// # Errors
-    /// [`UnmountRefused`] 的四种。重发冻结的那一次又报错（[`UnmountRefused::FrozenPublishResendFailed`]）等价于崩溃：冻结的那一次没落盘，下一次挂载走恢复。
+    /// [`UnmountRefused`] 的四种。重发冻结的那一次又报错（[`UnmountRefused::FrozenPublishResendFailed`]）等价于崩溃：那一次可能成了也可能没成（D16（发布语义） 已定项 17），下一次挂载走恢复，读现行根判。
     pub fn unmount(mut self) -> Result<UnmountedPool<Device>, Box<UnmountRefused<Device>>> {
         self.stop_and_join_the_maintenance_thread();
         let failure = self.shared.failure_so_far();
```

## crates/singlefs-core/src/compaction_plan.rs

```diff
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -520,12 +520,12 @@
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
@@ -535,13 +535,13 @@
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
@@ -550,9 +550,9 @@
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
@@ -617,9 +617,11 @@
     NothingMovableButTheRegionIsNotEmpty,
     /// 用户改动之后起跑的那一趟只跑至多一批，跑完了。
     OneBatchAfterTheUserChangeDone,
-    /// 后台入口：前台有 fsync 在等、或调用方在停这条驱动后台入口的线程，不发下一批（D26（后台整理与放置回收） 已定项 13 第 6 条）。
+    /// 后台入口：前台有 fsync 在等，不发下一批（D26（后台整理与放置回收） 已定项 13 第 6 条）。
     /// 全空段低于底额、前台写不进时这一趟的第一次发布不为前台 fsync 让路（前台已写不进，让路换不来什么）。
     YieldedToAForegroundFsync,
+    /// 后台入口：调用方在停这条驱动后台入口的线程，不发下一批，低于底额的破例也不发（线程很快 join 得上）。
+    YieldedToTheCallerStoppingItsThread,
     /// 后台入口：这一个写带宽窗口里整理写的字节已到写带宽上限，不发下一批（D26 已定项 13 第 6 条，[`CompactionWriteBandwidthLimit`]）。
     WriteBandwidthLimitReached {
         bytes_written_in_the_window: u64,
@@ -652,6 +654,8 @@
     /// 后台入口放锁的那一种（`mounted_session::MountedSessionWithPendingChanges::run_compaction_in_the_background_for_at_most_one_publish`）：
     /// 这一趟这一次调用发成了一次，调用方放锁之后接着调。不是停机，这一趟记在会话上没完。
     OnePublishOfTheTripDone,
+    /// 后台入口因删除队列不空起跑的那一趟（D26（后台整理与放置回收） 已定项 13）：队列走空了，这一趟只走队列、一次整理发布都不发。整理下一次调用照常判起跑。
+    DeletionQueueWalkedWithoutACompactionPublish,
 }
 
 impl CompactionRunStop {
@@ -675,13 +679,15 @@
             | CompactionRunStop::NothingMovableButTheRegionIsNotEmpty
             | CompactionRunStop::OneBatchAfterTheUserChangeDone
             | CompactionRunStop::YieldedToAForegroundFsync
+            | CompactionRunStop::YieldedToTheCallerStoppingItsThread
             | CompactionRunStop::WriteBandwidthLimitReached { .. }
             | CompactionRunStop::BatchAbandonedAfterABackgroundComputeError { .. }
             | CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
             | CompactionRunStop::OnlyRecordsUncertainThisRoundLeftToMove { .. }
             | CompactionRunStop::IssueHeldBackByRecordsUncertainThisRound { .. }
             | CompactionRunStop::RollbackStatesOfTheRingUnreadable
-            | CompactionRunStop::OnePublishOfTheTripDone => false,
+            | CompactionRunStop::OnePublishOfTheTripDone
+            | CompactionRunStop::DeletionQueueWalkedWithoutACompactionPublish => false,
         }
     }
 
@@ -716,6 +722,9 @@
             CompactionRunStop::YieldedToAForegroundFsync => {
                 "compaction_yielded_to_a_foreground_fsync"
             }
+            CompactionRunStop::YieldedToTheCallerStoppingItsThread => {
+                "compaction_yielded_to_the_caller_stopping_its_thread"
+            }
             CompactionRunStop::WriteBandwidthLimitReached { .. } => {
                 "compaction_paced_by_the_write_bandwidth_limit"
             }
@@ -738,6 +747,9 @@
                 "compaction_stopped_the_rollback_states_of_the_ring_are_unreadable"
             }
             CompactionRunStop::OnePublishOfTheTripDone => "compaction_one_publish_of_the_trip_done",
+            CompactionRunStop::DeletionQueueWalkedWithoutACompactionPublish => {
+                "compaction_deletion_queue_walked_without_a_compaction_publish"
+            }
         }
     }
 }
@@ -919,15 +931,15 @@
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
@@ -975,12 +987,12 @@
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
@@ -1099,74 +1111,74 @@
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
 
@@ -1512,20 +1524,20 @@
     };
 }
 
-/// 前台此刻有没有 fsync 在等（D26（后台整理与放置回收） 已定项 13 第 6 条「前台有 fsync 在等时不发下一批」）。封闭集合。
+/// 这一批之前调用方的回答（D26（后台整理与放置回收） 已定项 13 第 6 条「前台有 fsync 在等时不发下一批」）：发不发下一批，要不要让路。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub enum ForegroundFsync {
-    NoneWaiting,
-    Waiting,
+pub enum AnswerOfTheCallerBeforeTheNextBatch {
+    NoForegroundFsyncWaiting,
+    AForegroundFsyncIsWaiting,
     /// 前台不一定有 fsync 在等，是调用方在停这条驱动后台入口的线程：照样不发下一批，低于底额的破例也不发（线程很快 join 得上）。
     CallerIsStoppingItsThread,
 }
 
-/// 后台入口每推一批之前问调用方的两样：此刻的钟、前台有没有 fsync 在等。核心层不起线程，调用方在自己的线程上驱动后台入口，
+/// 后台入口每推一批之前问调用方的两样：此刻的钟、这一批之前它的回答。核心层不起线程，调用方在自己的线程上驱动后台入口，
 /// 钟与前台的 fsync 队列都在调用方手里；测试里由一份假的实现。
 pub trait CallerOfTheBackgroundCompaction {
     fn clock_reading(&self) -> CallerMonotonicClockReading;
-    fn foreground_fsync(&self) -> ForegroundFsync;
+    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch;
 }
 
 /// 回退门给根环留的余量（D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：一次整理发布之后，再发这么多次发布，根环里仍有 4 个不同用户状态。
```

## crates/singlefs-core/src/mounted_session.rs

```diff
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -51,10 +51,10 @@
     plan_batch_skipping_the_units_this_publish_rewrites_anyway, publish_budget_of_a_compaction_run,
     publishes_left_of_the_budget_given_by_the_last_user_change, region_holds_a_quarantined_slot,
     region_is_empty_by_the_records, region_scans_of_the_segments_without_an_unreleased_record,
-    the_completed_intent_yielded, BackgroundTripUnderWay, CallerMonotonicClockReading,
-    CallerOfTheBackgroundCompaction, CompactionBatchSize, CompactionBatchUnits,
-    CompactionOfThisMount, CompactionPacing, CompactionRunForcing, CompactionRunStop,
-    CompactionStartJudgement, CompactionSwitches, CompactionWriteBandwidthLimit, ForegroundFsync,
+    the_completed_intent_yielded, AnswerOfTheCallerBeforeTheNextBatch, BackgroundTripUnderWay,
+    CallerMonotonicClockReading, CallerOfTheBackgroundCompaction, CompactionBatchSize,
+    CompactionBatchUnits, CompactionOfThisMount, CompactionPacing, CompactionRunForcing,
+    CompactionRunStop, CompactionStartJudgement, CompactionSwitches, CompactionWriteBandwidthLimit,
     LiveSlotsOfAClusterSegment, PlannedBatch, PlannedRelocation, RegionChoice,
     RegionChoiceCriterion, TimerFallbackCheck, UnitAreaOfOneDevice,
 };
@@ -62,8 +62,7 @@
 use crate::instance_table_sweep::unit_head_read_by_sweep;
 use crate::journal::JournalBytesToWrite;
 use crate::logged_operations::{
-    live_compaction_intent_of_the_entries, live_truncation_intent_of_the_entries, CompactionIntent,
-    CompactionRegion,
+    live_compaction_intent_of_the_entries, CompactionIntent, CompactionRegion,
 };
 use crate::make_filesystem::MakeFilesystemParameters;
 use crate::mount::{
@@ -79,9 +78,10 @@
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
@@ -99,18 +99,19 @@
     publish_data_changes_published_together, publish_new_inodes, publish_overwrite,
     publish_overwrite_of_an_inode, publish_partial_write, publish_sequential_write,
     publish_sequential_write_of_an_inode, publish_size_change, publish_snapshot_destruction_intent,
-    publish_snapshot_destruction_step, publish_the_relocations_and_the_repair_of_the_exempt_units,
-    publish_truncation_step, publish_unlink_intent, publish_unlink_step, publish_version,
-    relocations_of_old_data_units_kept_for_the_snapshots,
-    relocations_of_the_units_read_with_one_copy_failing_while_mounting,
+    publish_the_next_step_of_the_deletion_queue,
+    publish_the_relocations_and_the_repair_of_the_exempt_units, publish_unlink_intent,
+    publish_version, relocations_of_old_data_units_kept_for_the_snapshots,
+    relocations_of_the_units_read_with_one_copy_failing,
     repair_of_the_exempt_units_at_these_locations, resend_the_frozen_publish,
     units_at_these_locations, walk_the_deletion_queue_until_it_is_empty,
     CentralMappingOfAVersionReadLeafByLeaf, CompactionIntentStep, CompactionStepRefused,
     DataChangesPublishedTogether, DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile,
-    MappingLookup, NewInodesOfOneChange, PartialWriteBytes, PlannedUnitsRewrittenAnyway,
-    PoolVersion, PoolWriter, PublishError, ReadFailedOnTheDisk, ReadFailingSelfVerification,
-    RegionCheckOfTheIntentDeletion, ReleaseChecksumCheck, RelocationBatchRefused,
-    RepairOfTheExemptUnits, TransactionOutput, TransactionUnit, WholeWriteOfOneInode,
+    KindOfTheFrozenPublish, MappingLookup, NewInodesOfOneChange, PartialWriteBytes,
+    PlannedUnitsRewrittenAnyway, PoolVersion, PoolWriter, PublishError, ReadFailedOnTheDisk,
+    ReadFailingSelfVerification, RegionCheckOfTheIntentDeletion, ReleaseChecksumCheck,
+    RelocationBatchRefused, RepairOfTheExemptUnits, TransactionOutput, TransactionUnit,
+    WholeWriteOfOneInode,
 };
 use crate::unit::{data_unit_payload_capacity, unit_filesystem_identifier, UNIT_CLASS_DATA};
 use crate::unlink::DeletionSwitches;
@@ -222,23 +223,26 @@
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
@@ -249,8 +253,9 @@
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
@@ -258,7 +263,7 @@
     },
     /// 为空间推抬 F 时抬 F 自己报了错（`mount::push_one_floor_raise_within_the_admission_budget` 的 `Err`，原样）：这一串是因为
     /// `refusal_that_pushed_the_failed_raise` 那一次被拒才推的；之前推成的几串在 `floor_raises`。会话的 `current` 停在最后一次落盘的那一版上。
-    /// 只供测试的开关 `compaction_plan::FloorRaiseForTheRelocationKeptForTheSnapshotsFails`（连同被空间拒的那个开关）强制进这一支。
+    /// 只供测试的开关 `compaction_plan::FloorRaiseForTheRelocationBatchAfterTheChangeFails`（连同被空间拒的那个开关）强制进这一支。
     FloorRaiseFailedWhilePushingForSpace {
         cause: Box<MountError>,
         refusal_that_pushed_the_failed_raise: Box<RelocationBatchRefused>,
@@ -361,6 +366,8 @@
 pub struct CompactionRun {
     /// 这一趟发成的每一版，按先后：先走空的删除续做、再是整理的各批。最后一个就是会话的 `current`。
     pub versions_published: Vec<TransactionOutput>,
+    /// 其中删除续做的步数（排在最前面）：其余才是整理的发布。
+    pub deletion_steps_published: u64,
     /// 其中整理批的个数：搬了单元的那几次（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F2），签发、
     /// 一个没搬的删意图与丢意图不算。
     pub batches_published: u64,
@@ -490,8 +497,11 @@
     /// 用户改动发成之后起跑判据记下的「该跑了」（判决 G8 丁：至多一批，预算 `B_gc`）。
     DueAfterAUserChange,
     /// 定时兜底这一次查起跑判据为真（D26 已定项 13 第 5 条）。这一趟没有自己的预算，与用户改动共用上一次用户改动给的那一份 `B_gc`
-    /// （岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲，用户 2026-10-26 定）。
+    /// （岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲）。
     TimerFallback,
+    /// 现行那一版的删除队列不空（D26（后台整理与放置回收） 已定项 13）：只走队列、不发整理，不看水位，也不被那一份 `B_gc` 挡（删除续做不占预算）。
+    /// 调用方挂载之后不调任何后台入口，队列没走空时前台的写都被拒（[`AcceptRefused::QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion`]），没有别人走它。
+    DeletionQueueNotEmpty,
 }
 
 /// 后台入口这一次起不起跑（[`MountedSession::run_compaction_in_the_background`] 开头那几判的结论）。封闭集合。
@@ -2099,8 +2109,22 @@
     )
 }
 
-/// 同 [`publish_the_queue_until_it_is_empty`]，只走现行那一版删除队列的下一步（次序同 `transaction::walk_the_deletion_queue_until_it_is_empty`：
-/// `deleted_inodes`、销毁中的快照、截断）：放锁的后台入口在步与步之间放锁，队列多长都不整段持锁。队列空时一步都不发。
+/// 这一次调用把一次发布冻结了（进门时没冻结着）：记上那一次是哪一种，重发成了照它记账（[`MountedSession::resend_the_frozen_publish`]）。发布路径不知道发起它的是谁，种类只有这一层说得出。
+fn note_the_kind_of_a_publish_frozen_by_this_call(
+    allocator: &mut PoolAllocator,
+    was_frozen_before_this_call: bool,
+    kind: KindOfTheFrozenPublish,
+) {
+    if was_frozen_before_this_call {
+        return;
+    }
+    if let Some(frozen) = allocator.take_frozen_publish() {
+        allocator.freeze_publish(frozen.of_the_kind(kind));
+    }
+}
+
+/// 同 [`publish_the_queue_until_it_is_empty`]，只走现行那一版删除队列的下一步（[`publish_the_next_step_of_the_deletion_queue`] 选）：
+/// 放锁的后台入口在步与步之间放锁，队列多长都不整段持锁。队列空时一步都不发。
 fn publish_at_most_one_step_of_the_queue<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut [(DeviceIdentity, Device)],
@@ -2111,40 +2135,17 @@
     let switches: DeletionSwitches = allocator.deletion_switches();
     let mut writer = PoolWriter::new(parameters, devices);
     writer.set_container_tree_capacities(switches.container_tree_capacities());
-    let next_step = match (
-        current.deleted_inodes.first().copied(),
-        current.snapshots_being_destroyed.first().copied(),
-        live_truncation_intent_of_the_entries(&current.logged_operations),
-    ) {
-        (Some(inode), _, _) => publish_unlink_step(
-            &mut writer,
-            allocator,
-            current,
-            inode,
-            switches.batch_size,
-            instance,
-        ),
-        (None, Some(snapshot), _) => publish_snapshot_destruction_step(
-            &mut writer,
-            allocator,
-            current,
-            snapshot,
-            switches.batch_size,
-            instance,
-        ),
-        (None, None, Some(_truncation)) => publish_truncation_step(
-            &mut writer,
-            allocator,
-            current,
-            switches.batch_size,
-            instance,
-        ),
-        (None, None, None) => {
-            return DeletionQueueWalk {
-                steps_published: Vec::new(),
-                stopped_by: Ok(()),
-            }
-        }
+    let Some(next_step) = publish_the_next_step_of_the_deletion_queue(
+        &mut writer,
+        allocator,
+        current,
+        switches.batch_size,
+        instance,
+    ) else {
+        return DeletionQueueWalk {
+            steps_published: Vec::new(),
+            stopped_by: Ok(()),
+        };
     };
     match next_step {
         Ok(next) => {
@@ -2190,14 +2191,16 @@
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
@@ -2237,38 +2240,46 @@
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
@@ -2283,7 +2294,7 @@
         &repair,
     );
     allocator.forget_the_locations_read_with_one_copy_failing(|locations| {
-        locations_read_with_one_copy_failing.contains(locations)
+        locations_tried_by_this_batch.contains(locations)
             || locations.iter().any(|location| {
                 planned
                     .iter()
@@ -2306,6 +2317,7 @@
                     .map(|relocation| relocation.mapping_key)
                     .collect(),
                 units_exempt_from_the_mapping_replaced: repair.units,
+                instance_table_chain_unreadable: repair.instance_table_chain_unreadable,
                 floor_raises,
             }
         }
@@ -2329,11 +2341,11 @@
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
@@ -2359,10 +2371,10 @@
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
@@ -2393,7 +2405,7 @@
             );
         }
         let floor_raise = if compaction
-            .is_the_floor_raise_for_the_relocation_kept_for_the_snapshots_forced_to_fail()
+            .is_the_floor_raise_for_the_relocation_batch_after_the_change_forced_to_fail()
         {
             Err(floor_raise_failure_of_the_relocation_forced_for_tests())
         } else {
@@ -2462,7 +2474,7 @@
     )
 }
 
-/// 只供测试的开关造的那一拒（`compaction_plan::RelocationKeptForTheSnapshotsRefusedForSpace`）：空间准入不够那一种，一块不够的盘都不点名——
+/// 只供测试的开关造的那一拒（`compaction_plan::RelocationBatchAfterTheChangeRefusedForSpace`）：空间准入不够那一种，一块不够的盘都不点名——
 /// 没有真算准入，只为把另发那一批强制进「被空间拒」那一支；在这一批的写入口建起之前交回，一个写都没发。
 fn space_refusal_of_the_relocation_forced_for_tests() -> RelocationBatchRefused {
     RelocationBatchRefused::Publish(PublishError::SpaceAdmissionRefused(
@@ -2472,7 +2484,7 @@
     ))
 }
 
-/// 只供测试的开关造的那一拒（`compaction_plan::RelocationKeptForTheSnapshotsRefusedNotForSpace`）：计划对不上那一版那一种，点名计划里头一把 key——
+/// 只供测试的开关造的那一拒（`compaction_plan::RelocationBatchAfterTheChangeRefusedNotForSpace`）：计划对不上那一版那一种，点名计划里头一把 key——
 /// 没有真核计划，只为把另发那一批强制进「被拒、不是空间不够」那一支；在这一批的写入口建起之前交回，一个写都没发。
 /// 这一批只换豁免映射的单元、计划是空的时点名的 key 是空的。
 fn refusal_not_for_space_of_the_relocation_forced_for_tests(
@@ -2486,7 +2498,7 @@
     }
 }
 
-/// 只供测试的开关造的那一错（`compaction_plan::FloorRaiseForTheRelocationKeptForTheSnapshotsFails`）：现行那一版的实例表读不出——
+/// 只供测试的开关造的那一错（`compaction_plan::FloorRaiseForTheRelocationBatchAfterTheChangeFails`）：现行那一版的实例表读不出——
 /// [`push_one_floor_raise_within_the_admission_budget`] 在任何写之前交得出的一种；没有真读盘，只为把另发那一批强制进「抬 F 自己报错」那一支，一个写都没发。
 fn floor_raise_failure_of_the_relocation_forced_for_tests() -> MountError {
     MountError::InstanceTableUnreadable
@@ -2766,7 +2778,7 @@
     }
 
     /// 这次可写挂载里别的读读到只坏一份的次数（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬，包括……可写挂载期间别的读」）：[`Self::read_the_file`]、
-    /// 推抬 F 与发 discard 之前沿链读实例表、释放之前读盘核、搬留存单元读源。一组位置条目等着下一次用户改动的这一程里只算一次，处置过之后再读到再算。
+    /// 推抬 F 与发 discard 之前读根环与实例表、整理的回退门、回退、清扫、释放之前读盘核、搬留存单元读源、另发那一批认位置沿链读实例表（挂载这一串读到的那几组也在等下一次用户改动，再读不进这个数）。一组位置条目等着下一次用户改动的这一程里只算一次，处置过之后再读到再算。
     /// 与 [`Self::unit_reads_with_one_copy_failing_while_mounting`]、[`Self::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`] 按读的地方分开报。
     /// 只在内存里，不落盘；新挂载从 0 起。
     #[must_use]
@@ -3174,6 +3186,9 @@
     ///    现行那一版与开关都没变就不重判（[`BackgroundCompactionNotStarted::UnchangedSinceAStopThatPublishedNothing`]；全空段低于底额时停在回退门上的那一次不算）；每一次整理发布之前判回退门
     ///    （[`CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates`]）。
     ///
+    /// 整理没停着而现行那一版的删除队列不空：排在第 2、3、4 条之前起跑（[`BackgroundCompactionTrigger::DeletionQueueNotEmpty`]），只走队列、不发整理，不看水位、不被预算挡，「该跑了」与定时兜底都不动；
+    /// 走空之后下一次调用照上面几条判。整理停着时队列也不走（第 1 条：一个写都不发）。
+    ///
     /// 起跑之后每一批之前问调用方：前台有 fsync 在等就不发下一批（[`CompactionRunStop::YieldedToAForegroundFsync`]；全空段低于底额时这一趟的第一次发布除外），这一个写带宽窗口里
     /// 整理写的字节到了 `pacing.write_bandwidth_limit` 也不发（[`CompactionRunStop::WriteBandwidthLimitReached`]）；用户改动之后起跑的那一趟
     /// 停在这两样上时「该跑了」放回去，下一次调用接着推（不发下一批是缓发，不是做完）。签下一条意图那一判
@@ -3215,7 +3230,8 @@
 
     /// 把分配器上冻结着的那一次发布原样重发（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」，[`resend_the_frozen_publish`]）：一批整理或一次用户改动写盘报错之后
     /// 发布路径冻结，前台的写都被拒，靠这一个入口解。重发成了：`current` 换成那一版、分配器换成那次发布成立之后的那一份，活意图的 R 排他按那一版的 logged_ops 重装
-    /// （冻结的那一批可能正是删意图的那一批）。没有冻结着的发布交回 `false`，什么都不写。整理的发布计数不补那一次（差一次，只让这一份预算多发一次）。
+    /// （冻结的那一批可能正是删意图的那一批），再照那一次的种类记账（[`KindOfTheFrozenPublish`]；D26（后台整理与放置回收） 已定项 13 第 6 条）：
+    /// 用户改动那种与一次发成同一份账（重给预算、清上一条完成的意图、起跑判据为真记「该跑了」），整理那种扣预算。没有冻结着的发布交回 `false`，什么都不写。
     ///
     /// # Errors
     /// 重发又有一步报错（`PublishError::BlockDevice`）：照旧冻结着。
@@ -3223,6 +3239,11 @@
         &mut self,
         devices: &mut Vec<(DeviceIdentity, Device)>,
     ) -> Result<bool, PublishError> {
+        let Some(kind_of_the_frozen_publish) =
+            self.allocator.frozen_publish().map(|frozen| frozen.kind())
+        else {
+            return Ok(false);
+        };
         let parameters = self.parameters_and_device_table.parameters_on_disk();
         let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
         let Some(version) = resend_the_frozen_publish(&mut writer, &mut self.allocator)? else {
@@ -3239,6 +3260,23 @@
         if let Some((_, intent)) = live_intent_of_the_version {
             self.allocator.compaction_mut().issue(intent);
         }
+        match kind_of_the_frozen_publish {
+            KindOfTheFrozenPublish::UserChange => {
+                self.compaction.note_a_user_change_published();
+                self.compaction.last_completed_intent = None;
+                let watermark_verdict =
+                    compaction_watermark_verdict_of_the_allocator(&self.allocator);
+                self.compaction.last_watermark_verdict = Some(watermark_verdict);
+                if watermark_verdict.starts() {
+                    self.compaction.start_judgement = CompactionStartJudgement::DueAfterAUserChange;
+                }
+            }
+            KindOfTheFrozenPublish::CompactionPublish => {
+                self.compaction.publishes_since_the_last_user_change += 1;
+                self.compaction.observations.root_ring_slots_used += 1;
+            }
+            KindOfTheFrozenPublish::NeitherOfTheCountedKinds => {}
+        }
         Ok(true)
     }
 
@@ -3298,18 +3336,18 @@
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
@@ -3317,7 +3355,14 @@
         let foreground_cannot_write = foreground_cannot_write_under_the_switches(allocator);
         // 放锁的后台入口上一次调用之后还没完的那一趟接着推：一趟只判一次起跑。整理停着的那一判在上面：停着时这一趟留着，解除之后接着推。
         // 那一份预算扣完就不再推整理，只有两样例外：删除续做不占预算，队列没走空照走；这一趟的破例还没用掉。
-        if let Some(trip) = compaction.take_the_background_trip_under_way() {
+        // 只走队列的那一趟，队列已被前台走空时没什么可推，这一趟到此为止，下面照常判。
+        if let Some(trip) = compaction
+            .take_the_background_trip_under_way()
+            .filter(|trip| {
+                trip.reach != CompactionRunReach::OnlyTheDeletionQueue
+                    || !deletion_queue_of_the_version_is_empty(current)
+            })
+        {
             let is_the_exception_still_open =
                 trip.is_the_exception_of_the_trip_still_open && foreground_cannot_write;
             if publishes_left_of_the_budget_given_by_the_last_user_change(
@@ -3342,6 +3387,15 @@
                 batches_published_earlier_in_the_trip: trip.batches_published,
             });
         }
+        // 队列不空就起跑，只走队列：「该跑了」与定时兜底都不动，队列走空之后照常判。
+        if !deletion_queue_of_the_version_is_empty(current) {
+            return Ok(BackgroundCompactionStart::Starts {
+                trigger: BackgroundCompactionTrigger::DeletionQueueNotEmpty,
+                reach: CompactionRunReach::OnlyTheDeletionQueue,
+                budget: CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange,
+                batches_published_earlier_in_the_trip: 0,
+            });
+        }
         let (trigger, reach) = match compaction.start_judgement {
             CompactionStartJudgement::DueAfterAUserChange => {
                 compaction.start_judgement = CompactionStartJudgement::NotDue;
@@ -3476,6 +3530,7 @@
             run.outcome,
             CompactionRunOutcome::Stopped(
                 CompactionRunStop::YieldedToAForegroundFsync
+                    | CompactionRunStop::YieldedToTheCallerStoppingItsThread
                     | CompactionRunStop::WriteBandwidthLimitReached { .. }
             )
         );
@@ -3486,7 +3541,10 @@
         // 全空段低于底额时为前台 fsync 让路的这一趟不留着：留着的话下一次调用接着让路，前台 fsync 一直有就再也轮不到下一趟的破例。
         let is_yielding_while_the_foreground_cannot_write = matches!(
             run.outcome,
-            CompactionRunOutcome::Stopped(CompactionRunStop::YieldedToAForegroundFsync)
+            CompactionRunOutcome::Stopped(
+                CompactionRunStop::YieldedToAForegroundFsync
+                    | CompactionRunStop::YieldedToTheCallerStoppingItsThread
+            )
         )
             && foreground_cannot_write_under_the_switches(allocator);
         match span {
@@ -3514,7 +3572,8 @@
                         compaction.start_judgement = CompactionStartJudgement::DueAfterAUserChange;
                     }
                     BackgroundCompactionTrigger::DueAfterAUserChange
-                    | BackgroundCompactionTrigger::TimerFallback => {}
+                    | BackgroundCompactionTrigger::TimerFallback
+                    | BackgroundCompactionTrigger::DeletionQueueNotEmpty => {}
                 }
             }
         }
@@ -3547,6 +3606,7 @@
         change: UserChange<'_>,
         old_data_unit_reads_back: &mut OldDataUnitReadsBackWithOneCopyFailingInThisChange,
     ) -> Result<UserChangePublished, UserChangeRefused> {
+        let was_frozen_when_the_change_came_in = allocator.frozen_publish().is_some();
         let current_version_of_trees = match &*current {
             PoolVersion::WithFile(version_of_trees)
             | PoolVersion::WithTreesAndNoFile(version_of_trees) => version_of_trees,
@@ -3643,6 +3703,14 @@
                         RemainderOfTheUserChange::WalkTheQueue,
                         published_in_this_change,
                     );
+                    // 这次改动已经发成，冻结的是它之后的删除步：重发成了这次改动的那份账照记。
+                    if walk.is_err() {
+                        note_the_kind_of_a_publish_frozen_by_this_call(
+                            allocator,
+                            was_frozen_when_the_change_came_in,
+                            KindOfTheFrozenPublish::UserChange,
+                        );
+                    }
                     *current = PoolVersion::of_a_shared_version_of_trees(published);
                     let PublishedInThisChange {
                         floor_raises,
@@ -3659,7 +3727,14 @@
                             RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded,
                     });
                 }
-                Err(refusal) => refusal,
+                Err(refusal) => {
+                    note_the_kind_of_a_publish_frozen_by_this_call(
+                        allocator,
+                        was_frozen_when_the_change_came_in,
+                        KindOfTheFrozenPublish::UserChange,
+                    );
+                    refusal
+                }
             };
             let publishes_of_this_raise;
             (published_in_this_change, publishes_of_this_raise) =
@@ -4102,6 +4177,8 @@
     AtMostOneBatchAfterAUserChange,
     /// 可写挂载续做（D8（核心索引结构） 已定项 5 第 2 条；判决 G1）：只推盘上那一条未完成的意图，推到它删了、停机原因或预算用完，不签新的。
     ContinueTheLiveIntentAtTheMount,
+    /// 后台入口因删除队列不空起跑的那一趟（[`BackgroundCompactionTrigger::DeletionQueueNotEmpty`]）：走空队列就停，不推整理。
+    OnlyTheDeletionQueue,
 }
 
 /// 整理的一次发布做的是哪一种（[`CompactionStep::Published`]）。封闭集合。
@@ -4193,16 +4270,16 @@
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
@@ -4246,6 +4323,8 @@
     in_the_middle_of_each_batch: &mut InTheMiddleOfEachBackgroundBatch<'_>,
 ) -> CompactionRun {
     compaction.observations.runs_started += 1;
+    let was_frozen_when_the_run_started = allocator.frozen_publish().is_some();
+    let deletion_steps_published = std::cell::Cell::new(0_u64);
     let mut versions_published: Vec<TransactionOutput> = Vec::new();
     let mut background_compute_of_each_batch: Vec<BackgroundComputeOfOneBatch> = Vec::new();
     let switches_of_the_run: CompactionSwitches = allocator.compaction().switches;
@@ -4265,6 +4344,7 @@
         }
         CompactionRun {
             versions_published: versions_published_by_the_run,
+            deletion_steps_published: deletion_steps_published.get(),
             batches_published,
             outcome,
             discards: DiscardsAfterThePublishes::default(),
@@ -4339,6 +4419,8 @@
             ),
         };
         *current = PoolVersion::of_a_shared_version_of_trees(walked);
+        deletion_steps_published
+            .set(u64::try_from(walk.steps_published.len()).expect("一趟的删除步数"));
         // 整理这一趟的版本列表按值交回（[`CompactionRun::versions_published`]）：走空的各步只有最后一步还与 `current` 共享，拷那一份。
         versions_published.extend(walk.steps_published.into_iter().map(Arc::unwrap_or_clone));
         if let Err(cause) = walk.stopped_by {
@@ -4351,7 +4433,10 @@
                 CompactionRunOutcome::Failed(CompactionRunFailure::DeletionContinuation(cause)),
             );
         }
-        if span_of_the_call == SpanOfTheBackgroundCall::AtMostOnePublish {
+        if span_of_the_call == SpanOfTheBackgroundCall::AtMostOnePublish
+            && (reach != CompactionRunReach::OnlyTheDeletionQueue
+                || !deletion_queue_of_the_version_is_empty(current))
+        {
             return finish(
                 compaction,
                 current.root().checkpoint_txg,
@@ -4362,6 +4447,18 @@
             );
         }
     }
+    if reach == CompactionRunReach::OnlyTheDeletionQueue {
+        return finish(
+            compaction,
+            current.root().checkpoint_txg,
+            versions_published,
+            background_compute_of_each_batch,
+            0,
+            CompactionRunOutcome::Stopped(
+                CompactionRunStop::DeletionQueueWalkedWithoutACompactionPublish,
+            ),
+        );
+    }
     let budget = match budget_rule {
         CompactionRunBudget::SinceTheLastUserChange => publish_budget_of_a_compaction_run(
             parameters.geometry.root_ring_slots_per_region.count(),
@@ -4438,9 +4535,10 @@
                 caller,
                 ..
             } => {
-                match caller.foreground_fsync() {
-                    ForegroundFsync::Waiting if is_this_publish_the_exception => {}
-                    ForegroundFsync::Waiting | ForegroundFsync::CallerIsStoppingItsThread => {
+                match caller.answer_before_the_next_batch() {
+                    AnswerOfTheCallerBeforeTheNextBatch::AForegroundFsyncIsWaiting
+                        if is_this_publish_the_exception => {}
+                    AnswerOfTheCallerBeforeTheNextBatch::AForegroundFsyncIsWaiting => {
                         return finish(
                             compaction,
                             current.root().checkpoint_txg,
@@ -4452,7 +4550,19 @@
                             ),
                         );
                     }
-                    ForegroundFsync::NoneWaiting => {}
+                    AnswerOfTheCallerBeforeTheNextBatch::CallerIsStoppingItsThread => {
+                        return finish(
+                            compaction,
+                            current.root().checkpoint_txg,
+                            versions_published,
+                            background_compute_of_each_batch,
+                            batches_published,
+                            CompactionRunOutcome::Stopped(
+                                CompactionRunStop::YieldedToTheCallerStoppingItsThread,
+                            ),
+                        );
+                    }
+                    AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting => {}
                 }
                 if let Err(stop) = compaction.admit_the_next_batch_under_the_write_bandwidth_limit(
                     write_bandwidth_limit,
@@ -4471,7 +4581,8 @@
             }
         };
         if is_the_rollback_gate_judged && !is_this_publish_the_exception {
-            if let Some(stop) = stop_of_the_rollback_gate(parameters, &*devices, current) {
+            if let Some(stop) = stop_of_the_rollback_gate(parameters, &*devices, allocator, current)
+            {
                 return finish(
                     compaction,
                     current.root().checkpoint_txg,
@@ -4555,6 +4666,11 @@
                 );
             }
             Err(refused) => {
+                note_the_kind_of_a_publish_frozen_by_this_call(
+                    allocator,
+                    was_frozen_when_the_run_started,
+                    KindOfTheFrozenPublish::CompactionPublish,
+                );
                 return finish(
                     compaction,
                     current.root().checkpoint_txg,
@@ -5870,6 +5986,31 @@
         }
     }
 
+    /// 这一次改动并进待发之后那一批的整份写与按收下次序的建 inode（`DataChangesPublishedTogether` 借的那两样）：整份写后到的盖掉先到的。
+    fn together_with_this_change<'batch>(
+        &'batch self,
+        data_change: DataChangeAccepted<'batch>,
+    ) -> (
+        Option<WholeWriteOfOneInode<'batch>>,
+        Vec<NewInodesOfOneChange>,
+    ) {
+        let mut new_inodes_together = self.new_inodes.clone();
+        let whole_write_together = match data_change {
+            DataChangeAccepted::WholeWrite(whole_write) => Some(whole_write),
+            DataChangeAccepted::NewInodes(created) => {
+                new_inodes_together.push(created);
+                self.whole_write
+                    .as_ref()
+                    .map(|pending_whole_write| WholeWriteOfOneInode {
+                        inode: pending_whole_write.inode,
+                        content: &pending_whole_write.content,
+                        write_time_seconds: pending_whole_write.write_time_seconds,
+                    })
+            }
+        };
+        (whole_write_together, new_inodes_together)
+    }
+
     /// 这一批里建的 inode 一共几个（之后建的号从记账水位加上它起发）。
     fn new_inodes_in_total(&self) -> u64 {
         self.as_published_together().new_inodes_in_total()
@@ -6087,11 +6228,30 @@
     pub published: UserChangePublished,
 }
 
-/// [`MountedSessionWithPendingChanges::accept`] 收下了：怎么收的、这次调用里报的每一次发布（按先后）。
+/// [`MountedSessionWithPendingChanges::accept`] 收下了：怎么收的、这次调用里报的每一次发布（按先后）、收下之前为空间当场推的抬 F。
 #[derive(Debug)]
 pub struct ChangeAccepted {
     pub taken: HowTheChangeWasTaken,
+    /// 这次调用里发成的待发与屏障：改了用户可见状态的发布，不含 `floor_raises_pushed_for_space` 那几串空发布。
     pub publishes: Vec<PublishReportedByTheSession>,
+    pub floor_raises_pushed_for_space: FloorRaisesPushedByTheAccept,
+}
+
+/// [`MountedSessionWithPendingChanges::accept`] 这次调用里，这一次改动被空间拒时当场推成的几串抬 F 的空发布（D16（发布语义） 已定项 1「准入」那一行），已落盘。
+/// 空发布不是用户改动：不进 [`ChangeAccepted::publishes`]、不重给整理预算、不算上次发布的读数（同 [`MountedSession::publish_user_change`] 被拒时推的那几串），
+/// 待发不在里面，收下的改动照旧等下一次排空。
+#[derive(Debug)]
+pub struct FloorRaisesPushedByTheAccept {
+    pub floor_raises: Vec<RaisedFloor>,
+    /// 每一串是因为哪一次被拒才推的（与 `floor_raises` 一一对应）。
+    pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
+}
+
+/// [`MountedSessionWithPendingChanges::accept`] 进门判的是哪一批。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum BatchJudgedAtTheAccept {
+    ThisChangeAlone,
+    PendingChangesTogetherWithThisChange,
 }
 
 /// 一次改动在 [`MountedSessionWithPendingChanges::accept`] 里怎么收的。封闭集合。
@@ -6141,11 +6301,21 @@
     /// 写的号是待发里建 inode 那几次要发的号（还不在现行那一版的树里）：同一次发布里建一个 inode 又写它，条款没写，第一版不支持。在任何读写之前拒。
     WriteToAnInodeCreatedByThePendingChangesNotSupportedInTheFirstVersion { inode: InodeNumber },
     /// 这一次改动单独发布就过不了落盘之前那几判（`transaction::journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write`：
-    /// 号不在树里、删除中、内容装不进覆盖写的一个数据单元、准入、落点、记录条数……），`cause` 原样。在任何读写之前拒，待发不动。
+    /// 号不在树里、删除中、内容装不进覆盖写的一个数据单元、记录条数、合取 7……），`cause` 原样，不是空间不够的那两种（[`refusal_is_short_of_space`]）。
+    /// 在任何读写之前拒，待发不动。
     ChangeRefusedByThePublishJudgement(Box<PublishError>),
-    /// 这一次改动单独过得了，并进待发合起来这一批过不了（判决第二节「一次发布里攒什么」那一格：`accept` 进门先做这一批合起来的准入与条数预判，
-    /// 过不了就在任何读写之前拒，不在返回成功之后拒），`cause` 原样。在任何读写之前拒，待发不动；调用方要做的是先排空待发、再交这一次。
+    /// 这一次改动单独过得了，并进待发合起来这一批过不了（判决第二节「一次发布里攒什么」那一格：`accept` 进门先做这一批合起来的条数预判，
+    /// 过不了就在任何读写之前拒，不在返回成功之后拒），`cause` 原样，不是空间不够的那两种。在任何读写之前拒，待发不动；调用方要做的是先排空待发、再交这一次。
     PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(Box<PublishError>),
+    /// `judgement` 那一批被空间拒，当场推抬 F 再判（D16（发布语义） 已定项 1「准入」那一行）之后还是没收下，`refusal` 原样：
+    /// [`UserChangeRefused::NoSpaceAfterRaisingTheFloor`] 是推到不再推了仍放不下（一次准入的发布数做满、F 已在上限、抬 F 那一串自己放不下，一串都没推成时 `floor_raises` 是空的）；
+    /// [`UserChangeRefused::FloorRaiseFailedWhilePushingForSpace`] 是抬 F 自己报了错（写盘报错时那一次空发布冻结）；
+    /// [`UserChangeRefused::Publish`] 是推过之后再判拒成了别的原因。推成的那几串已落盘，现行那一版在它们后面（[`Self::floor_raises_pushed_by_this_accept`]）。
+    /// 这一次改动没收进待发；待发里已有的原样留着，冻结的那一次空发布不含它们。
+    RefusedAfterRaisingTheFloorForSpace {
+        judgement: BatchJudgedAtTheAccept,
+        refusal: Box<UserChangeRefused>,
+    },
     /// 屏障进门之前排空待发没发成（`refusal` 原样；推的抬 F、发成的走空步在里面）：屏障没发。
     PublishOfThePendingChangesBeforeTheBarrierRefused(Box<UserChangeRefused>),
     /// 待发已排空（`publishes_before_the_refusal`），屏障自己没发成（`refusal` 原样）。
@@ -6153,13 +6323,56 @@
         publishes_before_the_refusal: Vec<PublishReportedByTheSession>,
         refusal: Box<UserChangeRefused>,
     },
-    /// 这次改动已收进待发，收下之后触发的那一次发布没发成（`refusal` 原样）：没冻结时待发原样留着（连同这一次），下一次触发再发；
-    /// 冻结了时待发并进了冻结的那一次（D16（发布语义） 已定项 17：报错之后这次发布可能已成也可能未成），待发清空。
-    AcceptedButTheTriggeredPublishRefused(Box<UserChangeRefused>),
+    /// 这次改动已收进待发，收下之后触发的那一次发布没发成（`refusal` 原样）：冻结的是这一批自己时待发并进了冻结的那一次
+    /// （D16（发布语义） 已定项 17：报错之后这次发布可能已成也可能未成），待发清空；别的情形待发原样留着（连同这一次），下一次触发再发。
+    /// `floor_raises_pushed_for_space` 是收下之前为空间推成的几串。
+    AcceptedButTheTriggeredPublishRefused {
+        floor_raises_pushed_for_space: FloorRaisesPushedByTheAccept,
+        refusal: Box<UserChangeRefused>,
+    },
+}
+
+impl AcceptRefused {
+    /// 这次调用没收下或没发成时，已经为空间推成、落了盘的那几串抬 F（空的是一串都没推）。
+    #[must_use]
+    pub fn floor_raises_pushed_by_this_accept(&self) -> &[RaisedFloor] {
+        match self {
+            AcceptRefused::RefusedAfterRaisingTheFloorForSpace { refusal, .. } => match &**refusal {
+                UserChangeRefused::NoFileVersionToChange { floor_raises, .. }
+                | UserChangeRefused::Publish { floor_raises, .. } => floor_raises,
+                UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) => &no_space.floor_raises,
+                UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(failed) => {
+                    &failed.floor_raises
+                }
+                UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
+                | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
+                | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
+                | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
+                    ..
+                } => &[],
+            },
+            AcceptRefused::AcceptedButTheTriggeredPublishRefused {
+                floor_raises_pushed_for_space,
+                ..
+            } => &floor_raises_pushed_for_space.floor_raises,
+            AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
+            | AcceptRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
+            | AcceptRefused::NoFileVersionToChange
+            | AcceptRefused::QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion
+            | AcceptRefused::WriteAtOffsetIntoThePendingChangesNotSupportedInTheFirstVersion
+            | AcceptRefused::DataChangesPublishedTogetherIntoThePendingChangesNotSupportedInTheFirstVersion
+            | AcceptRefused::WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion { .. }
+            | AcceptRefused::WriteToAnInodeCreatedByThePendingChangesNotSupportedInTheFirstVersion { .. }
+            | AcceptRefused::ChangeRefusedByThePublishJudgement(_)
+            | AcceptRefused::PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(_)
+            | AcceptRefused::PublishOfThePendingChangesBeforeTheBarrierRefused(_)
+            | AcceptRefused::BarrierRefused { .. } => &[],
+        }
+    }
 }
 
 /// 排空待发那一次没发成（[`MountedSessionWithPendingChanges::publish_when_due`]、[`MountedSessionWithPendingChanges::publish_pending_now`]）：
-/// `refusal` 原样。没冻结时待发原样留着；冻结了时待发并进了冻结的那一次、清空（D16（发布语义） 已定项 17）。
+/// `refusal` 原样。冻结的是这一批自己时待发并进了冻结的那一次、清空（D16（发布语义） 已定项 17）；别的情形（没冻结、冻结的是抬 F 的空发布或进门之前就冻结着的那一次）待发原样留着。
 #[derive(Debug)]
 pub struct PublishOfThePendingChangesRefused {
     pub refusal: Box<UserChangeRefused>,
@@ -6293,7 +6506,8 @@
     }
 
     /// 把待发整个并成一次发布交给会话（[`UserChange::DataChangesPublishedTogether`]，先走空队列、被空间拒照旧推抬 F 再判）。待发空就什么都不发、交回 `None`
-    /// （不推空发布）。发成：待发清空、记下这次的读数。没发成：没冻结时待发原样放回；冻结了时待发并进了冻结的那一次，不放回（D16（发布语义） 已定项 17）。
+    /// （不推空发布）。发成：待发清空、记下这次的读数。没发成：冻结的是这一批自己（种类 [`KindOfTheFrozenPublish::UserChange`]）时待发并进了冻结的那一次，
+    /// 不放回（D16（发布语义） 已定项 17）；别的情形待发原样放回：没冻结、冻结的是这一次推的抬 F 空发布、进门之前就冻结着（那几次都不含这一批，丢了就是收下的改动没了）。
     fn publish_the_pending_changes<Device: BlockDevice>(
         &mut self,
         devices: &mut Vec<(DeviceIdentity, Device)>,
@@ -6313,7 +6527,13 @@
                 Ok(Some(PublishReportedByTheSession { reason, published }))
             }
             Err(refusal) => {
-                if self.session.allocator.frozen_publish().is_none() {
+                // 进门之前就冻结着的那一次不含这一批：种类是用户改动的那一次冻结时待发是空的（排空发成之后、或待发并进了它），这里走不到；别的种类不是用户改动。
+                let is_the_batch_in_the_frozen_publish = self
+                    .session
+                    .allocator
+                    .frozen_publish()
+                    .is_some_and(|frozen| frozen.kind() == KindOfTheFrozenPublish::UserChange);
+                if !is_the_batch_in_the_frozen_publish {
                     self.pending = pending;
                 }
                 Err(Box::new(refusal))
@@ -6324,12 +6544,16 @@
     /// 收下一次用户改动（D16（发布语义） 已定项 10，判决第二节「一次发布里攒什么」「串行下不立的」两格）：
     /// 1. 冻结着一次没重发的发布、交进来的盘表与挂载核过的不同：在任何读写之前拒。
     /// 2. 屏障（unlink、截断、建删快照）：先排空待发（一次发布）、再照 [`MountedSession::publish_user_change`] 单发，两次都报。
-    /// 3. 数据写类（整份写、建 inode）：在任何读写之前先判——现行那一版有可改的文件、队列走空了、并得进待发（至多一个文件、不写待发里建的号）、
+    /// 3. 数据写类（整份写、建 inode）：先判——现行那一版有可改的文件、队列走空了、并得进待发（至多一个文件、不写待发里建的号）、
     ///    这一次单独发布过得了落盘之前那几判、并进待发合起来的这一批也过得了（准入与条数预判，`transaction::journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write`）；
-    ///    有一样不过就拒，一个盘都不读、一个写都不发、待发不动。都过了才收进待发（同一 inode 的写折叠、建 inode 追加），脏量加上这一次单独发布要写的 journal 字节；
+    ///    被空间拒（[`refusal_is_short_of_space`] 那两种）时当场推抬 F 的空发布再判（D16（发布语义） 已定项 1「准入」那一行，上限与预算同 [`MountedSession::publish_user_change`]，
+    ///    这一次改动算一次准入），别的不过就拒，一个写都不发、待发不动。都过了才收进待发（同一 inode 的写折叠、建 inode 追加），脏量加上这一次单独发布要写的 journal 字节；
     ///    收下之后判一次触发（[`publish_trigger_cause`]），触发就把待发整个并成一次发布。
     /// 4. 按偏移写、已经并好的一批：条款没写，在任何读写之前拒。
     ///
+    /// 推抬 F 只在进门判过前两样（没冻结、盘表对）与第 3 条前面那几样之后：冻结着、队列没走空时拒的是那几样，不推。推的空发布不含待发、不算用户改动，报在 [`ChangeAccepted::floor_raises_pushed_for_space`]，
+    /// 没收下时在 [`AcceptRefused::floor_raises_pushed_by_this_accept`]。于是这一次调用可能写盘，至多推 B − 1 次空发布（每次一遍单元、记录、根与系统配置，一串多一遍系统配置）。
+    ///
     /// 整理不在这里跑：发成之后起跑判据为真只记「该跑了」（[`MountedSession::publish_user_change`]），由后台入口去跑。
     ///
     /// # Errors
@@ -6375,21 +6599,30 @@
             }
             ChangeAtTheAccept::Data { change, contract } => (change, contract),
         };
-        let journal_bytes_of_this_change =
-            self.judge_a_data_change_before_any_read_or_write(devices, data_change, contract)?;
+        let (journal_bytes_of_this_change, floor_raises_pushed_for_space) = self
+            .judge_a_data_change_raising_the_floor_for_space(
+                devices,
+                change,
+                data_change,
+                contract,
+            )?;
         self.pending
             .take_in(data_change, journal_bytes_of_this_change);
         let mut publishes = Vec::new();
         if let Some(cause) = self.trigger_cause_at(now) {
-            if let Some(published) = self
-                .publish_the_pending_changes(
-                    devices,
-                    now,
-                    ReasonOfThePublish::TriggeredAfterTheAccept(cause),
-                )
-                .map_err(AcceptRefused::AcceptedButTheTriggeredPublishRefused)?
-            {
-                publishes.push(published);
+            match self.publish_the_pending_changes(
+                devices,
+                now,
+                ReasonOfThePublish::TriggeredAfterTheAccept(cause),
+            ) {
+                Ok(Some(published)) => publishes.push(published),
+                Ok(None) => {}
+                Err(refusal) => {
+                    return Err(AcceptRefused::AcceptedButTheTriggeredPublishRefused {
+                        floor_raises_pushed_for_space,
+                        refusal,
+                    })
+                }
             }
         }
         Ok(ChangeAccepted {
@@ -6397,10 +6630,134 @@
                 journal_bytes_of_this_change,
             },
             publishes,
+            floor_raises_pushed_for_space,
         })
     }
 
-    /// [`Self::accept`] 收数据写类改动之前那几判，在任何读写之前：交回这一次单独发布要写的 journal 字节。
+    /// 判 [`Self::judge_a_data_change_before_any_read_or_write`]；被空间拒就为被判的那一批推一串抬 F、再判（与 [`MountedSession::publish_user_change`] 被拒时走同一个
+    /// [`push_one_floor_raise_for_a_refusal_for_space`]，上限与预算一样）。这一次改动算一次准入，推的数从 0 起。判过了交回这一次单独发布要写的 journal 字节与推成的几串。
+    /// 推的那几串就地推进 `current`，待发不动。
+    ///
+    /// 迭代上界：每一轮要么判过交回，要么交回拒绝，要么推成一串（至少一次空发布，已推的数加上去，预算在 B − 1 次之内停），所以至多 8 轮。跨轮携带的是现行那一版与已推的几串。
+    ///
+    /// # Errors
+    /// [`Self::judge_a_data_change_before_any_read_or_write`] 的几种（不是空间不够、又一串没推时原样）；[`AcceptRefused::RefusedAfterRaisingTheFloorForSpace`]。
+    fn judge_a_data_change_raising_the_floor_for_space<Device: BlockDevice>(
+        &mut self,
+        devices: &mut Vec<(DeviceIdentity, Device)>,
+        change: UserChange<'_>,
+        data_change: DataChangeAccepted<'_>,
+        contract: WholeWriteContractAtTheAccept,
+    ) -> Result<(JournalBytesToWrite, FloorRaisesPushedByTheAccept), AcceptRefused> {
+        let mut published_in_this_change = PublishedInThisChange {
+            floor_raises: Vec::new(),
+            refusals_that_pushed_the_floor_raises: Vec::new(),
+            versions_published: Vec::new(),
+        };
+        let mut publishes_pushed_in_this_admission = 0;
+        loop {
+            let (judgement, cause) = match self
+                .judge_a_data_change_before_any_read_or_write(devices, data_change, contract)
+            {
+                Ok(journal_bytes_of_this_change) => {
+                    return Ok((
+                        journal_bytes_of_this_change,
+                        FloorRaisesPushedByTheAccept {
+                            floor_raises: published_in_this_change.floor_raises,
+                            refusals_that_pushed_the_floor_raises: published_in_this_change
+                                .refusals_that_pushed_the_floor_raises,
+                        },
+                    ))
+                }
+                Err(AcceptRefused::ChangeRefusedByThePublishJudgement(cause)) => {
+                    (BatchJudgedAtTheAccept::ThisChangeAlone, *cause)
+                }
+                Err(AcceptRefused::PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(
+                    cause,
+                )) => (
+                    BatchJudgedAtTheAccept::PendingChangesTogetherWithThisChange,
+                    *cause,
+                ),
+                Err(
+                    refused @ (AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
+                    | AcceptRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
+                    | AcceptRefused::NoFileVersionToChange
+                    | AcceptRefused::QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion
+                    | AcceptRefused::WriteAtOffsetIntoThePendingChangesNotSupportedInTheFirstVersion
+                    | AcceptRefused::DataChangesPublishedTogetherIntoThePendingChangesNotSupportedInTheFirstVersion
+                    | AcceptRefused::WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion { .. }
+                    | AcceptRefused::WriteToAnInodeCreatedByThePendingChangesNotSupportedInTheFirstVersion { .. }
+                    | AcceptRefused::RefusedAfterRaisingTheFloorForSpace { .. }
+                    | AcceptRefused::PublishOfThePendingChangesBeforeTheBarrierRefused(_)
+                    | AcceptRefused::BarrierRefused { .. }
+                    | AcceptRefused::AcceptedButTheTriggeredPublishRefused { .. }),
+                ) => return Err(refused),
+            };
+            if !refusal_is_short_of_space(&cause)
+                && published_in_this_change.floor_raises.is_empty()
+            {
+                return Err(match judgement {
+                    BatchJudgedAtTheAccept::ThisChangeAlone => {
+                        AcceptRefused::ChangeRefusedByThePublishJudgement(Box::new(cause))
+                    }
+                    BatchJudgedAtTheAccept::PendingChangesTogetherWithThisChange => {
+                        AcceptRefused::PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(
+                            Box::new(cause),
+                        )
+                    }
+                });
+            }
+            let (whole_write_together, new_inodes_together) =
+                self.pending.together_with_this_change(data_change);
+            let remainder =
+                RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(match judgement {
+                    BatchJudgedAtTheAccept::ThisChangeAlone => change,
+                    BatchJudgedAtTheAccept::PendingChangesTogetherWithThisChange => {
+                        UserChange::DataChangesPublishedTogether(DataChangesPublishedTogether {
+                            whole_write: whole_write_together,
+                            new_inodes: &new_inodes_together,
+                        })
+                    }
+                });
+            let MountedSession {
+                allocator,
+                current,
+                shadow_ledger,
+                parameters_and_device_table,
+                ..
+            } = &mut self.session;
+            let (PoolVersion::WithFile(version_of_trees)
+            | PoolVersion::WithTreesAndNoFile(version_of_trees)) = &*current
+            else {
+                return Err(AcceptRefused::NoFileVersionToChange);
+            };
+            let mut version_pushed = Arc::clone(version_of_trees);
+            let pushed = push_one_floor_raise_for_a_refusal_for_space(
+                parameters_and_device_table.parameters_on_disk(),
+                devices,
+                allocator,
+                &mut version_pushed,
+                *shadow_ledger,
+                remainder,
+                publishes_pushed_in_this_admission,
+                published_in_this_change,
+                cause,
+            );
+            *current = PoolVersion::of_a_shared_version_of_trees(version_pushed);
+            let publishes_of_this_raise;
+            (published_in_this_change, publishes_of_this_raise) =
+                pushed.map_err(
+                    |refusal| AcceptRefused::RefusedAfterRaisingTheFloorForSpace {
+                        judgement,
+                        refusal: Box::new(refusal),
+                    },
+                )?;
+            publishes_pushed_in_this_admission += publishes_of_this_raise;
+        }
+    }
+
+    /// [`Self::accept`] 收数据写类改动之前那几判，一个盘都不读、一个写都不发：交回这一次单独发布要写的 journal 字节。判的是此刻的现行那一版与分配器，被空间拒之后的处置在
+    /// [`Self::judge_a_data_change_raising_the_floor_for_space`]。
     ///
     /// # Errors
     /// [`AcceptRefused::NoFileVersionToChange`]、`QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion`、
@@ -6482,21 +6839,8 @@
                 self.session.instance,
             )
             .map_err(|cause| AcceptRefused::ChangeRefusedByThePublishJudgement(Box::new(cause)))?;
-        let mut new_inodes_together = self.pending.new_inodes.clone();
-        let whole_write_together = match data_change {
-            DataChangeAccepted::WholeWrite(whole_write) => Some(whole_write),
-            DataChangeAccepted::NewInodes(created) => {
-                new_inodes_together.push(created);
-                self.pending
-                    .whole_write
-                    .as_ref()
-                    .map(|pending_whole_write| WholeWriteOfOneInode {
-                        inode: pending_whole_write.inode,
-                        content: &pending_whole_write.content,
-                        write_time_seconds: pending_whole_write.write_time_seconds,
-                    })
-            }
-        };
+        let (whole_write_together, new_inodes_together) =
+            self.pending.together_with_this_change(data_change);
         journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write(
             &writer,
             &self.session.allocator,
@@ -6543,6 +6887,11 @@
                 Ok(ChangeAccepted {
                     taken: HowTheChangeWasTaken::PublishedAloneAsABarrier,
                     publishes,
+                    // 屏障为空间推的抬 F 报在它自己那一次发布里（`UserChangePublished::floor_raises`）。
+                    floor_raises_pushed_for_space: FloorRaisesPushedByTheAccept {
+                        floor_raises: Vec::new(),
+                        refusals_that_pushed_the_floor_raises: Vec::new(),
+                    },
                 })
             }
             Err(refusal) => Err(AcceptRefused::BarrierRefused {
@@ -6571,16 +6920,30 @@
     }
 
     /// 立即排空（D16（发布语义） 已定项 11：只是提前触发一次发布，同一个 journal、同一条发布路径）：待发非空就整个并成一次发布；待发空就什么都不发、交回 `None`
-    /// （不推空发布）。
+    /// （不推空发布）。冻结着一次没重发的发布时在任何读写之前拒（D16（发布语义） 已定项 7「fsync 等系统配置轮换持久之后才返回」）：待发空着也不答 `None`，
+    /// 收下的改动可能正并在冻结的那一次里。调用方要做的是先重发（[`Self::resend_the_frozen_publish`]）、发成了再调。
     ///
     /// # Errors
-    /// 同 [`Self::publish_when_due`]。
+    /// 同 [`Self::publish_when_due`]；冻结着：`refusal` 是 [`UserChangeRefused::Publish`]，`cause` 是 [`PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet`]。
     pub fn publish_pending_now<Device: BlockDevice>(
         &mut self,
         devices: &mut Vec<(DeviceIdentity, Device)>,
         now: CallerMonotonicClockReading,
     ) -> Result<Option<PublishReportedByTheSession>, PublishOfThePendingChangesRefused> {
         self.note_the_reading_seen(now);
+        let frozen_publish = self.session.allocator.frozen_publish();
+        if let Some(frozen) = frozen_publish {
+            return Err(PublishOfThePendingChangesRefused {
+                refusal: Box::new(UserChangeRefused::Publish {
+                    cause: PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
+                        checkpoint_txg: frozen.checkpoint_txg(),
+                        instance: frozen.instance(),
+                    },
+                    floor_raises: Vec::new(),
+                    versions_published: Vec::new(),
+                }),
+            });
+        }
         self.publish_the_pending_changes(
             devices,
             now,
@@ -6790,7 +7153,8 @@
                             self.session.compaction.start_judgement =
                                 CompactionStartJudgement::DueAfterAUserChange;
                         }
-                        BackgroundCompactionTrigger::TimerFallback => {}
+                        BackgroundCompactionTrigger::TimerFallback
+                        | BackgroundCompactionTrigger::DeletionQueueNotEmpty => {}
                     },
                 }
                 // 用户 2026-10-30 定每次叫起都报：判过起跑之后排空被拒的这一次也是一次叫起，照数、照报。
```

## crates/singlefs-core/src/transaction.rs

```diff
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -374,6 +374,8 @@
     /// 把这次发布之后仍被活快照钉着（换下来进 deadlist、映射条目留存）的那几个搬到新槽（用户 2026-11-04 定「另发一批搬迁修好」，
     /// [`relocations_of_old_data_units_kept_for_the_snapshots`]）。只在内存里；新开的写入口从空起。
     mapping_keys_of_old_data_units_read_back_with_one_copy_failing: RefCell<Vec<Vec<u8>>>,
+    /// 这个写入口上最近一次分配记录树重写集的固定点走了几轮（[`PoolWriter::rounds_of_the_last_allocation_record_tree_settling`]）。只在内存里、不落盘；新开的写入口从 0 起。
+    rounds_of_the_last_allocation_record_tree_settling: Cell<usize>,
 }
 
 /// 一个写入口上读新槽残留的账（[`PoolWriter::residue_reads`]）：发了几次、读不出几次。
@@ -413,11 +415,20 @@
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
@@ -1535,6 +1546,8 @@
 #[derive(Clone, Debug)]
 pub struct FrozenPublish {
     writes: PublishWrites,
+    /// 这次发布是哪一种（[`KindOfTheFrozenPublish`]）：冻结在发布路径里定不了，由发起它的会话记上。重发成了，会话照它记账。
+    kind: KindOfTheFrozenPublish,
     /// 重发成功之后交出去的那一版；它的写账在重发那一刻现记（`writes` 字段），这里是空账。
     version: PoolVersion,
     /// 这次发布成立之后的分配器（释放、取落点、记根都做完的那一份）。冻结期间调用方手里的分配器回到发布之前的样子，
@@ -1558,6 +1571,28 @@
     pub fn writes(&self) -> &PublishWrites {
         &self.writes
     }
+
+    #[must_use]
+    pub fn kind(&self) -> KindOfTheFrozenPublish {
+        self.kind
+    }
+
+    /// 同一次发布，种类换成发起它的会话说的那一种。
+    #[must_use]
+    pub(crate) fn of_the_kind(self, kind: KindOfTheFrozenPublish) -> Self {
+        Self { kind, ..self }
+    }
+}
+
+/// 冻结的那一次发布重发成了，会话要怎么记账（D26（后台整理与放置回收） 已定项 13 第 6 条）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum KindOfTheFrozenPublish {
+    /// 用户改动的发布，或这次用户改动之后接着走的删除步：这次改动已经算发成，重给一份预算、清上一条完成的意图。
+    UserChange,
+    /// 整理的一次发布：扣预算、记一个根槽。
+    CompactionPublish,
+    /// 两样都不记：删除续做的一步、推抬 F 的一次发布，与发布路径刚冻结、还没有会话说种类的。
+    NeitherOfTheCountedKinds,
 }
 
 /// 发布路径的第一道：分配器上冻结着一次没重发的发布，就在任何读写之前拒绝（D23（journal 的角色与格式） 已定项 14
@@ -1597,6 +1632,7 @@
             std::mem::replace(allocator, allocator_before_this_publish);
         allocator.freeze_publish(FrozenPublish {
             writes,
+            kind: KindOfTheFrozenPublish::NeitherOfTheCountedKinds,
             version,
             allocator_after_the_publish,
         });
@@ -1629,6 +1665,7 @@
     }
     let FrozenPublish {
         writes: _,
+        kind: _,
         version,
         allocator_after_the_publish,
     } = frozen;
@@ -1887,6 +1924,8 @@
     allocation_record_tree: AllocationRecordTreePlan,
     rewritten_roles: Vec<TransactionUnit>,
     release: Vec<Placement>,
+    /// 固定点走了几轮（定下来的那一轮算在内）。
+    rounds_walked: usize,
 }
 
 /// 树表 0 条那一版写行时，分配记录树重写哪几个节点（固定点，同带文件那一路的 [`settle_the_allocation_record_tree`]）：上一版的树是分配器记着的
@@ -1928,10 +1967,11 @@
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
@@ -1970,7 +2010,12 @@
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
@@ -1985,9 +2030,15 @@
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
@@ -2102,6 +2153,8 @@
         txg,
         allocator,
     )?;
+    pool.rounds_of_the_last_allocation_record_tree_settling
+        .set(settled.rounds_walked);
     // 这次发布切出来的记录条数在动分配器之前判（代码审阅第 26 条）：点名的就是这次重写的角色（下面 `named_roles`）。
     refuse_a_publish_of_more_journal_records_than_the_limit(
         roles_named_by_each_record_of_the_publish(
@@ -10323,7 +10376,8 @@
 /// 一个盘都不读、一个写都不发；`pool` 只读它的参数与开关。过了交回这一次要写的 journal 字节（记录条数 × 4096）。
 ///
 /// 判的是「此刻接在 `previous` 后面」：会话里收下与排空之间不再有别的发布（收下与排空都拿着 `&mut` 会话，屏障类先排空），排空那一次与这里判的
-/// 是同一份上一版与分配器；被空间拒时这里不推抬 F（推是写），会话排空那一次照旧推抬 F 再判。
+/// 是同一份上一版与分配器，除非又一次 `accept` 为空间推了抬 F（那一次在推完之后把待发并上它重判过）；这里只判、不推抬 F（推是写）：被空间拒时
+/// `mounted_session::MountedSessionWithPendingChanges::accept` 推抬 F 再调这里，会话排空那一次照旧推抬 F 再判。
 ///
 /// # Errors
 /// `PublishFrozenAfterAWriteFailureIsNotResentYet`；计划那几种；`SpaceAdmissionRefused`、`PlacementRefused`、`JournalRecordsOfThePublishExceedTheLimit`、
@@ -11900,15 +11954,19 @@
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
@@ -11920,16 +11978,16 @@
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
@@ -11950,24 +12008,40 @@
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
@@ -11981,11 +12055,13 @@
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
@@ -12113,14 +12189,13 @@
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
@@ -12625,6 +12700,34 @@
     pub stopped_by: Result<(), PublishError>,
 }
 
+/// 选删除队列的下一步、发它：次序只在这里：`deleted_inodes` 先走空，再走销毁中的快照（树号升序），再走截断；整理是提示类，不在这里走。
+/// 下一步只由 `current` 这一版定：走空一趟（[`walk_the_deletion_queue_until_it_is_empty`]）与放锁的后台入口一次只走一步都调它，步与步之间插进别的发布，走到的还是同一串步。
+/// 队列空着交回 `None`，一个写都不发。
+pub fn publish_the_next_step_of_the_deletion_queue<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    current: &TransactionOutput,
+    batch_size: UnlinkBatchSize,
+    instance: InstanceGeneration,
+) -> Option<Result<TransactionOutput, PublishError>> {
+    match (
+        current.deleted_inodes.first().copied(),
+        current.snapshots_being_destroyed.first().copied(),
+        live_truncation_intent_of_the_entries(&current.logged_operations),
+    ) {
+        (Some(inode), _, _) => Some(publish_unlink_step(
+            pool, allocator, current, inode, batch_size, instance,
+        )),
+        (None, Some(snapshot), _) => Some(publish_snapshot_destruction_step(
+            pool, allocator, current, snapshot, batch_size, instance,
+        )),
+        (None, None, Some(_truncation)) => Some(publish_truncation_step(
+            pool, allocator, current, batch_size, instance,
+        )),
+        (None, None, None) => None,
+    }
+}
+
 /// 同 [`publish_unlink_steps_until_the_queue_is_empty`]，连同发成的每一步交回（会话与挂载续做要把它们交给调用方，
 /// 模型对拍逐次比它们）。每发成一步，`current` 与交回的那一步指着同一份（不拷一整版）。
 pub fn walk_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
@@ -12637,28 +12740,14 @@
     let mut steps_published = Vec::new();
     // 迭代上界：每一步要么让队首那个文件少至少一个单元、要么删掉队首那个号（最后一批），要么放掉销毁中那个快照至少一条 deadlist 条目、
     // 要么删掉它那一对条目（最后一批），要么让截断的文件少至少一个单元、要么删掉截断条目（最后一批）：队列、单元数与条目数都有限。
-    // 次序（判决 m4-trunc-r1 K1）：`deleted_inodes` 先走空，再走销毁中的快照（树号升序），再走截断；整理是提示类，不在这里走。
     loop {
-        let next_step = match (
-            current.deleted_inodes.first().copied(),
-            current.snapshots_being_destroyed.first().copied(),
-            live_truncation_intent_of_the_entries(&current.logged_operations),
-        ) {
-            (Some(inode), _, _) => {
-                publish_unlink_step(pool, allocator, current, inode, batch_size, instance)
-            }
-            (None, Some(snapshot), _) => publish_snapshot_destruction_step(
-                pool, allocator, current, snapshot, batch_size, instance,
-            ),
-            (None, None, Some(_truncation)) => {
-                publish_truncation_step(pool, allocator, current, batch_size, instance)
-            }
-            (None, None, None) => {
-                return DeletionQueueWalk {
-                    steps_published,
-                    stopped_by: Ok(()),
-                }
-            }
+        let Some(next_step) = publish_the_next_step_of_the_deletion_queue(
+            pool, allocator, current, batch_size, instance,
+        ) else {
+            return DeletionQueueWalk {
+                steps_published,
+                stopped_by: Ok(()),
+            };
         };
         match next_step {
             Ok(next) => {
@@ -13216,6 +13305,8 @@
         mapping_keys_whose_entries_change_without_a_relocation,
         pool.snapshot_lifetime_judgement(),
     )?;
+    pool.rounds_of_the_last_allocation_record_tree_settling
+        .set(settled.rounds_walked);
     // 这次发布切出来的记录条数（代码审阅第 26 条）：角色定下来就数得出，在准入、读盘核与动分配器之前判。
     let journal_records_of_the_publish = records_of_a_publish_naming_the_relocated_copies(
         &settled.resolved.rewritten_roles,
@@ -13319,57 +13410,31 @@
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
@@ -13377,7 +13442,7 @@
                 &settled.previous_roles_replaced,
                 &plan.instance_table,
                 &allocator.locations_read_with_one_copy_failing_waiting_for_the_next_user_change(),
-                &devices_counting_unit_reads_with_one_copy_failing,
+                reader,
             ));
             copies_failing
         }
@@ -13387,7 +13452,7 @@
                 previous_version,
                 &settled.previous_roles_replaced,
                 kept_for_the_snapshots,
-                &device_identities_of_the_accounting_rows,
+                device_identities_of_the_accounting_rows,
                 FileContentOfThePublish::of_the_plan(plan),
             )?;
             Vec::new()
@@ -13408,16 +13473,14 @@
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
@@ -13430,7 +13493,7 @@
         refuse_locations_that_do_not_name_two_pool_devices(
             relocation.role_for_the_checks,
             &relocation.source_locations,
-            &device_identities_of_the_accounting_rows,
+            device_identities_of_the_accounting_rows,
         )?;
         match release_checksum_check {
             ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt => {
@@ -13456,7 +13519,7 @@
                         copies_of_a_released_unit_failing_the_release_checksum_check(
                             relocation.role_for_the_checks,
                             &relocation.source_locations,
-                            &devices_counting_unit_reads_with_one_copy_failing,
+                            reader,
                         ),
                     );
                 }
@@ -13464,10 +13527,64 @@
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
@@ -13556,6 +13673,8 @@
     release: Vec<Placement>,
     /// 这次真搬的那几个（要搬的里面去掉这次重写本来就换下的角色），按计划的次序。
     relocations_in_effect: Vec<RelocationOfThePublish>,
+    /// 固定点走了几轮（定下来的那一轮算在内）。
+    rounds_walked: usize,
 }
 
 /// 这次发布换下的上一版角色与要释放的落点（释放判定路径，只查不改，D19（块指针的结构与宽度预算） 已定项 5 第 1 条）：
@@ -13684,14 +13803,17 @@
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
@@ -13733,10 +13855,11 @@
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
@@ -13792,7 +13915,12 @@
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
@@ -13811,9 +13939,15 @@
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
@@ -13821,9 +13955,42 @@
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

## crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs

```diff
--- a/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
+++ b/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
@@ -11,6 +11,8 @@
 //! 同一进程里新建到一定数量之后 Vulkan 驱动不再认得出独显（前面的实例丢掉也不还额度），所以新建的次数有上限（[`VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST`]），到了就拒，出路是重启进程。
 //!
 //! 只点一张卡也被没点名的卡拖：开卡按卡串行、没参与的卡也开，最坏是卡数乘一张卡的上限（默认期限 [`OPENING_DEADLINE_BY_DEFAULT`]）。第一版接受。
+//!
+//! 同一时刻只有一次开卡：多个挂载同时开卡时后到的排队，期限从轮到它才算，从调用起算的总等待可以超过期限（排在 n 个前面的之后，至多 (n + 1) 个期限）。
 
 use std::sync::atomic::{AtomicBool, Ordering};
 use std::sync::mpsc::{channel, RecvTimeoutError};
@@ -55,6 +57,23 @@
     instances_forgotten: u64,
 }
 
+/// 建实例途中占着的那一个额度：建实例 panic 了（展开走到这里）没占住实例，额度退回；建完了就不退。建到一半挂住时它留在那条线程上，额度照旧占着。
+struct LimitGivenBackIfTheMakingPanics<'opener, Instance> {
+    instance_of_this_process: &'opener Mutex<InstanceOfThisProcess<Instance>>,
+    is_the_instance_made: bool,
+}
+
+impl<Instance> Drop for LimitGivenBackIfTheMakingPanics<'_, Instance> {
+    fn drop(&mut self) {
+        if !self.is_the_instance_made {
+            self.instance_of_this_process
+                .lock()
+                .unwrap_or_else(PoisonError::into_inner)
+                .instances_created -= 1;
+        }
+    }
+}
+
 /// 工作线程结束时放开闸（走到末尾与 panic 展开都走到）。
 struct GateOpenedWhenTheOpeningEnds {
     an_opening_is_hung: Arc<AtomicBool>,
@@ -69,7 +88,7 @@
 /// 见模块说明。泛型 `Instance` 只为让不要卡的单测换上不碰 Vulkan 的替身；产品是 [`VulkanInstanceOfThisProcess`]（[`Self::of_this_process`]）。
 pub struct GpuOpenerOfThisProcess<Instance> {
     opening_deadline: Duration,
-    /// 同一时刻只有一次开卡：两次挂载同时开卡时后到的那一次在这里等。
+    /// 同一时刻只有一次开卡：两次挂载同时开卡时后到的那一次在这里等（这段等待不算进 `opening_deadline`）。
     one_opening_at_a_time: Mutex<()>,
     /// 上一次开卡的工作线程还没回来（调用线程已经到点不等了）。
     an_opening_is_hung: Arc<AtomicBool>,
@@ -171,7 +190,7 @@
         held.instances_forgotten += 1;
     }
 
-    /// 这一次开卡用的实例：有就用留着的，没有就新建、计数加一（额度在建之前就记，建到一半挂住也算用了一个）；计数到上限就拒。
+    /// 这一次开卡用的实例：有就用留着的，没有就新建、计数加一（额度在建之前就记，建到一半挂住也算用了一个；建实例 panic 了退回）；计数到上限就拒。
     /// 在工作线程上调：建实例装 Vulkan 加载器，驱动也可能挂在这里，所以建的时候不持那把锁。同一时刻至多一条工作线程走到这里（闸）。
     fn the_instance_or_a_new_one_under_the_limit(
         instance_of_this_process: &Mutex<InstanceOfThisProcess<Instance>>,
@@ -191,7 +210,13 @@
         held.instances_created += 1;
         let instances_forgotten_before_the_making = held.instances_forgotten;
         drop(held);
+        let mut limit_given_back_if_the_making_panics = LimitGivenBackIfTheMakingPanics {
+            instance_of_this_process,
+            is_the_instance_made: false,
+        };
         let instance = make_an_instance();
+        limit_given_back_if_the_making_panics.is_the_instance_made = true;
+        drop(limit_given_back_if_the_making_panics);
         let mut held_after_the_making = instance_of_this_process
             .lock()
             .unwrap_or_else(PoisonError::into_inner);
```

