# m4-gpugc-mount-runtime-code-r2：第 111 批产品代码改动全文（2026-11-09）

````diff
diff --git a/crates/singlefs-core/src/compaction_plan.rs b/crates/singlefs-core/src/compaction_plan.rs
index 8fca503..d6336b2 100644
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -17,12 +17,13 @@ use singlefs_replay_prefix::compaction_plan_proofs::{
 };
 
 use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
-use crate::admission::UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED;
+use crate::admission::{BytesOnOneDevice, UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED};
 use crate::allocator::{AllocationRecord, DeviceFreeMap, Placement, PoolAllocator, UnitAreaStart};
 use crate::background_compute::{BackgroundTaskAfterTheComputeError, ClusterSegmentEntryKey};
 use crate::logged_operations::{CompactionIntent, CompactionRegion};
 use crate::mounted_session::{BackgroundCompactionTrigger, CompactionRunReach};
 use crate::region_scan::{AllocationInRegion, RegionScan, RegionScanRefusal, ScannedAllocation};
+use crate::root_ring::RootRingSlotsPerRegion;
 
 /// 选段判据。封闭集合：加一种判据就加一个成员。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -616,7 +617,8 @@ pub enum CompactionRunStop {
     NothingMovableButTheRegionIsNotEmpty,
     /// 用户改动之后起跑的那一趟只跑至多一批，跑完了。
     OneBatchAfterTheUserChangeDone,
-    /// 后台入口：前台有 fsync 在等，不发下一批（D26（后台整理与放置回收） 已定项 13 第 6 条）。
+    /// 后台入口：前台有 fsync 在等、或调用方在停这条驱动后台入口的线程，不发下一批（D26（后台整理与放置回收） 已定项 13 第 6 条）。
+    /// 全空段低于底额、前台写不进时这一趟的第一次发布不为前台 fsync 让路（前台已写不进，让路换不来什么）。
     YieldedToAForegroundFsync,
     /// 后台入口：这一个写带宽窗口里整理写的字节已到写带宽上限，不发下一批（D26 已定项 13 第 6 条，[`CompactionWriteBandwidthLimit`]）。
     WriteBandwidthLimitReached {
@@ -639,7 +641,7 @@ pub enum CompactionRunStop {
     /// GPU 路径上选中要签的那一段里有这一轮两份核判得不一样的记录（D26 已定项 13 第 8 条）：R 里要搬几个数不准，签发前那一判判不了，
     /// 这一批不签，下一批重扫照判。
     IssueHeldBackByRecordsUncertainThisRound { records_uncertain_this_round: u64 },
-    /// 后台入口：这次整理发布要写的根槽与它之后 [`ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH`] 槽里的根一走，根环里不同用户状态就少于 4 个
+    /// 后台入口：这次整理发布要写的根槽与它之后余量（[`root_ring_slots_left_by_a_compaction_publish`]）那几槽里的根一走，根环里不同用户状态就少于 4 个
     /// （D16（发布语义） 已定项 1 的回退候选集；D26 已定项 13 第 5 条的加门）：不发，一个写都没发。全空段低于底额、前台写不进时每趟第一次发布破例不判。
     CompactionPublishWouldLeaveFewerThanFourRollbackStates {
         states_now: u64,
@@ -951,6 +953,9 @@ pub(crate) struct BackgroundTripUnderWay {
     pub(crate) reach: CompactionRunReach,
     /// 这一趟前几次调用已发的批数（用户改动之后那一趟至多一批，跨调用数）。
     pub(crate) batches_published: u64,
+    /// 这一趟是全空段低于底额、前台写不进时起跑的，前几次调用还没发过一次整理发布（只走了删除续做、被写带宽截住或排空被拒）：
+    /// 下一次调用接着推时，整理的第一次发布照样破例。
+    pub(crate) is_the_exception_of_the_trip_still_open: bool,
 }
 
 impl CompactionOfThisMount {
@@ -1512,6 +1517,8 @@ impl CompactionPacing {
 pub enum ForegroundFsync {
     NoneWaiting,
     Waiting,
+    /// 前台不一定有 fsync 在等，是调用方在停这条驱动后台入口的线程：照样不发下一批，低于底额的破例也不发（线程很快 join 得上）。
+    CallerIsStoppingItsThread,
 }
 
 /// 后台入口每推一批之前问调用方的两样：此刻的钟、前台有没有 fsync 在等。核心层不起线程，调用方在自己的线程上驱动后台入口，
@@ -1522,9 +1529,16 @@ pub trait CallerOfTheBackgroundCompaction {
 }
 
 /// 回退门给根环留的余量（D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：一次整理发布之后，再发这么多次发布，根环里仍有 4 个不同用户状态。
-/// 下一次可写挂载自己的发布不经门：写行加暖机至多 3 次，续做一条意图在产品批大小 16 下至多 `1 + ⌈64 ÷ 16⌉ = 5` 次，合起来是 8。
+/// 取每区根槽数 S（根环 3S 槽的三分之一）：最新的 4 个状态之后门放 `3S − 4 − S = 2S − 4` 次发布，S = 5 时是 6 次，
+/// 产品批大小 16 下一条整段意图（至多 `1 + ⌈64 ÷ 16⌉ = 5` 次发布）走得完；S = 4 时是 4 次，走不完就停在门上报出来。
+/// 下一次可写挂载自己的发布不经门：写行加暖机至多 3 次，续做一条意图至多 5 次，合起来至多 8 次。S = 8 时一次挂载落在余量里；S 更小时一次挂载就可能用完余量。
 /// 连崩几次用完这几槽之后，挂载自己的发布照样挤掉用户状态。
-pub const ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH: u64 = 8;
+#[must_use]
+pub fn root_ring_slots_left_by_a_compaction_publish(
+    root_ring_slots_per_region: RootRingSlotsPerRegion,
+) -> u64 {
+    root_ring_slots_per_region.count()
+}
 
 /// 一次整理运行的发布预算：
 /// `3S − 4 − 已经过去的发布数`，至少 1。`S` 是每区根槽数。
@@ -1689,6 +1703,32 @@ pub enum CompactionWatermarkCrossingForcing {
     ForcedForTests(WatermarkQuantity),
 }
 
+/// 一块盘上全空聚簇段的段数：全空聚簇段那条水位的读数与线。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct EmptyClusterSegmentCount(pub u64);
+
+/// 一块盘上空闲段（连续空闲槽的一段）的段数：碎片度那条水位的读数，线换算到这块盘此刻的已分配槽数上也是它。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct FreeRunCount(pub u64);
+
+/// 头一个越线的量，连同它的读数与线。三个量的单位各不相同，各带各的类型。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum WatermarkCrossing {
+    EmptyClusterSegments {
+        reading: EmptyClusterSegmentCount,
+        line: EmptyClusterSegmentCount,
+    },
+    /// `line` 是把线换算到这块盘的已分配槽数上的空闲段数。
+    Fragmentation {
+        reading: FreeRunCount,
+        line: FreeRunCount,
+    },
+    UnreclaimableBytes {
+        reading: BytesOnOneDevice,
+        line: BytesOnOneDevice,
+    },
+}
+
 /// 起跑判据的结论，连同走了哪一支（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum WatermarkVerdict {
@@ -1700,12 +1740,10 @@ pub enum WatermarkVerdict {
     CrossingForcedByTheTestOnlySwitch(WatermarkQuantity),
     /// 每块盘的三个量都没越线。
     NoLineCrossed,
-    /// 按盘的次序、每块盘按全空聚簇段、碎片度、不可回收字节的次序头一个越线的。碎片度的 `reading` 是空闲段数、`line` 是把线换算到这块盘的已分配槽数上的空闲段数。
+    /// 按盘的次序、每块盘按全空聚簇段、碎片度、不可回收字节的次序头一个越线的。
     Crossed {
         device: DeviceIdentity,
-        quantity: WatermarkQuantity,
-        reading: u64,
-        line: u64,
+        crossing: WatermarkCrossing,
     },
 }
 
@@ -1728,17 +1766,16 @@ impl WatermarkVerdict {
 fn watermark_crossed_on_the_device(
     watermarks: &CompactionWatermarks,
     reading: &WatermarkReadingOfADevice,
-) -> Option<(WatermarkQuantity, u64, u64)> {
+) -> Option<WatermarkCrossing> {
     if let Some(line) = watermarks
         .empty_cluster_segments
         .line_in_segments(reading.cluster_segments_in_the_unit_area)
     {
         if reading.empty_cluster_segments < line {
-            return Some((
-                WatermarkQuantity::EmptyClusterSegments,
-                reading.empty_cluster_segments,
-                line,
-            ));
+            return Some(WatermarkCrossing::EmptyClusterSegments {
+                reading: EmptyClusterSegmentCount(reading.empty_cluster_segments),
+                line: EmptyClusterSegmentCount(line),
+            });
         }
     }
     match watermarks.fragmentation {
@@ -1758,22 +1795,22 @@ fn watermark_crossed_on_the_device(
                 && is_above_the_line
                 && reading.free_runs >= at_least_free_runs.get()
             {
-                return Some((
-                    WatermarkQuantity::Fragmentation,
-                    reading.free_runs,
-                    u64::try_from(line_on_this_device).expect("低于空闲段数读数，装得进 u64"),
-                ));
+                return Some(WatermarkCrossing::Fragmentation {
+                    reading: FreeRunCount(reading.free_runs),
+                    line: FreeRunCount(
+                        u64::try_from(line_on_this_device).expect("低于空闲段数读数，装得进 u64"),
+                    ),
+                });
             }
         }
     }
     match watermarks.unreclaimable_bytes {
         UnreclaimableBytesWatermark::Off => None,
         UnreclaimableBytesWatermark::AtLeast(line) => (reading.unreclaimable_bytes >= line.get())
-            .then_some((
-                WatermarkQuantity::UnreclaimableBytes,
-                reading.unreclaimable_bytes,
-                line.get(),
-            )),
+            .then_some(WatermarkCrossing::UnreclaimableBytes {
+                reading: BytesOnOneDevice(reading.unreclaimable_bytes),
+                line: BytesOnOneDevice(line.get()),
+            }),
     }
 }
 
@@ -1796,13 +1833,10 @@ pub fn compaction_starts_by_the_watermarks(
         CompactionRunForcing::FollowTheWatermarks => {}
     }
     for reading in readings {
-        if let Some((quantity, value, line)) = watermark_crossed_on_the_device(watermarks, reading)
-        {
+        if let Some(crossing) = watermark_crossed_on_the_device(watermarks, reading) {
             return WatermarkVerdict::Crossed {
                 device: reading.device,
-                quantity,
-                reading: value,
-                line,
+                crossing,
             };
         }
     }
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 0d38b2c..2adffe3 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -62,7 +62,8 @@ use crate::instance_table::InstanceRow;
 use crate::instance_table_sweep::unit_head_read_by_sweep;
 use crate::journal::JournalBytesToWrite;
 use crate::logged_operations::{
-    live_compaction_intent_of_the_entries, CompactionIntent, CompactionRegion,
+    live_compaction_intent_of_the_entries, live_truncation_intent_of_the_entries, CompactionIntent,
+    CompactionRegion,
 };
 use crate::make_filesystem::MakeFilesystemParameters;
 use crate::mount::{
@@ -98,8 +99,9 @@ use crate::transaction::{
     publish_data_changes_published_together, publish_new_inodes, publish_overwrite,
     publish_overwrite_of_an_inode, publish_partial_write, publish_sequential_write,
     publish_sequential_write_of_an_inode, publish_size_change, publish_snapshot_destruction_intent,
-    publish_the_relocations_and_the_repair_of_the_exempt_units, publish_unlink_intent,
-    publish_version, relocations_of_old_data_units_kept_for_the_snapshots,
+    publish_snapshot_destruction_step, publish_the_relocations_and_the_repair_of_the_exempt_units,
+    publish_truncation_step, publish_unlink_intent, publish_unlink_step, publish_version,
+    relocations_of_old_data_units_kept_for_the_snapshots,
     relocations_of_the_units_read_with_one_copy_failing_while_mounting,
     repair_of_the_exempt_units_at_these_locations, resend_the_frozen_publish,
     units_at_these_locations, walk_the_deletion_queue_until_it_is_empty,
@@ -514,7 +516,7 @@ enum CompactionRunBudget {
     /// 后台入口的一趟（用户改动之后那一趟与定时兜底那几趟）：上一次用户改动给的那一份 `B_gc` 还剩的，没有至少 1，扣完是 0
     /// （[`publishes_left_of_the_budget_given_by_the_last_user_change`]；岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲）。
     LeftOfTheBudgetGivenByTheLastUserChange,
-    /// 同上，只差全空段低于底额、前台写不进时破例（D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：这一趟至少发 1 次，第一次发布不判回退门。
+    /// 同上，只差全空段低于底额、前台写不进时破例（D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：这一趟至少发 1 次，第一次发布不判回退门，也不为前台 fsync 让路。
     /// 前台等整理腾段，整理不能再等前台的用户改动给预算。
     AtLeastOnePublishWhileTheForegroundCannotWrite,
 }
@@ -525,7 +527,7 @@ enum SpanOfTheBackgroundCall {
     /// 一次调用把一趟推到停（[`MountedSession::run_compaction_in_the_background`]）。
     WholeTrip,
     /// 至多一次发布就返回（[`MountedSessionWithPendingChanges::run_compaction_in_the_background_for_at_most_one_publish`]）：
-    /// 调用方放锁之后接着调，这一趟记在会话上，不重判起跑。先走空的删除续做与一趟之后的 discard 切不开。
+    /// 调用方放锁之后接着调，这一趟记在会话上，不重判起跑。整理之前的删除续做也一次调用至多走一步。一趟之后的 discard 切不开。
     AtMostOnePublish,
 }
 
@@ -2097,6 +2099,87 @@ fn publish_the_queue_until_it_is_empty<Device: BlockDevice>(
     )
 }
 
+/// 同 [`publish_the_queue_until_it_is_empty`]，只走现行那一版删除队列的下一步（次序同 `transaction::walk_the_deletion_queue_until_it_is_empty`：
+/// `deleted_inodes`、销毁中的快照、截断）：放锁的后台入口在步与步之间放锁，队列多长都不整段持锁。队列空时一步都不发。
+fn publish_at_most_one_step_of_the_queue<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut [(DeviceIdentity, Device)],
+    allocator: &mut PoolAllocator,
+    current: &mut Arc<TransactionOutput>,
+    instance: InstanceGeneration,
+) -> DeletionQueueWalk {
+    let switches: DeletionSwitches = allocator.deletion_switches();
+    let mut writer = PoolWriter::new(parameters, devices);
+    writer.set_container_tree_capacities(switches.container_tree_capacities());
+    let next_step = match (
+        current.deleted_inodes.first().copied(),
+        current.snapshots_being_destroyed.first().copied(),
+        live_truncation_intent_of_the_entries(&current.logged_operations),
+    ) {
+        (Some(inode), _, _) => publish_unlink_step(
+            &mut writer,
+            allocator,
+            current,
+            inode,
+            switches.batch_size,
+            instance,
+        ),
+        (None, Some(snapshot), _) => publish_snapshot_destruction_step(
+            &mut writer,
+            allocator,
+            current,
+            snapshot,
+            switches.batch_size,
+            instance,
+        ),
+        (None, None, Some(_truncation)) => publish_truncation_step(
+            &mut writer,
+            allocator,
+            current,
+            switches.batch_size,
+            instance,
+        ),
+        (None, None, None) => {
+            return DeletionQueueWalk {
+                steps_published: Vec::new(),
+                stopped_by: Ok(()),
+            }
+        }
+    };
+    match next_step {
+        Ok(next) => {
+            *current = Arc::new(next);
+            DeletionQueueWalk {
+                steps_published: vec![Arc::clone(current)],
+                stopped_by: Ok(()),
+            }
+        }
+        Err(refusal) => DeletionQueueWalk {
+            steps_published: Vec::new(),
+            stopped_by: Err(refusal),
+        },
+    }
+}
+
+/// 这一版的删除队列（`deleted_inodes`、销毁中的快照、截断）空着。没有树的一版没有队列。
+fn deletion_queue_of_the_version_is_empty(version: &PoolVersion) -> bool {
+    match version {
+        PoolVersion::WithFile(version_of_trees)
+        | PoolVersion::WithTreesAndNoFile(version_of_trees) => {
+            version_of_trees.queue_of_committed_intents_is_empty()
+        }
+        PoolVersion::WithoutFile(_) => true,
+    }
+}
+
+/// 开关照水位判、任一块盘全空段低于底额：前台写入类发布已写不进（后台入口的破例、不再为前台 fsync 留着没完的一趟都判它）。
+fn foreground_cannot_write_under_the_switches(allocator: &PoolAllocator) -> bool {
+    matches!(
+        allocator.compaction().switches.run_forcing,
+        CompactionRunForcing::FollowTheWatermarks
+    ) && foreground_writes_are_below_the_empty_cluster_segments_floor(allocator)
+}
+
 /// 用户改动发成之后另发的那一批（[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`]）：`mapping_keys` 是这次改动里读回时只坏一份的旧单元，
 /// 其中到 `current` 这一版仍被活快照钉着的（`transaction::relocations_of_old_data_units_kept_for_the_snapshots`）一个都没有就不发；有就在新开的
 /// 写入口上照整理的搬迁入口发一批（[`publish_the_relocation_batch_raising_the_floor_for_space`]，意图照抄：`transaction::publish_relocation_batch` 那一支），
@@ -3091,7 +3174,7 @@ impl MountedSession {
     ///    现行那一版与开关都没变就不重判（[`BackgroundCompactionNotStarted::UnchangedSinceAStopThatPublishedNothing`]；全空段低于底额时停在回退门上的那一次不算）；每一次整理发布之前判回退门
     ///    （[`CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates`]）。
     ///
-    /// 起跑之后每一批之前问调用方：前台有 fsync 在等就不发下一批（[`CompactionRunStop::YieldedToAForegroundFsync`]），这一个写带宽窗口里
+    /// 起跑之后每一批之前问调用方：前台有 fsync 在等就不发下一批（[`CompactionRunStop::YieldedToAForegroundFsync`]；全空段低于底额时这一趟的第一次发布除外），这一个写带宽窗口里
     /// 整理写的字节到了 `pacing.write_bandwidth_limit` 也不发（[`CompactionRunStop::WriteBandwidthLimitReached`]）；用户改动之后起跑的那一趟
     /// 停在这两样上时「该跑了」放回去，下一次调用接着推（不发下一批是缓发，不是做完）。签下一条意图那一判
     /// 读会话上跨趟留着的上一条完成的意图（`compaction_plan::CompactionOfThisMount::last_completed_intent`）。发布都返回之后发 discard（判决 G5）。
@@ -3231,14 +3314,18 @@ impl MountedSession {
                 .publishes_since_the_last_user_change
                 .max(publishes_read_from_the_ring);
         }
-        let foreground_cannot_write = is_judged_as_the_product_does
-            && foreground_writes_are_below_the_empty_cluster_segments_floor(allocator);
+        let foreground_cannot_write = foreground_cannot_write_under_the_switches(allocator);
         // 放锁的后台入口上一次调用之后还没完的那一趟接着推：一趟只判一次起跑。整理停着的那一判在上面：停着时这一趟留着，解除之后接着推。
+        // 那一份预算扣完就不再推整理，只有两样例外：删除续做不占预算，队列没走空照走；这一趟的破例还没用掉。
         if let Some(trip) = compaction.take_the_background_trip_under_way() {
+            let is_the_exception_still_open =
+                trip.is_the_exception_of_the_trip_still_open && foreground_cannot_write;
             if publishes_left_of_the_budget_given_by_the_last_user_change(
                 parameters.geometry.root_ring_slots_per_region.count(),
                 compaction.publishes_since_the_last_user_change,
             ) == 0
+                && deletion_queue_of_the_version_is_empty(current)
+                && !is_the_exception_still_open
             {
                 return Ok(BackgroundCompactionStart::NotStarted(
                     BackgroundCompactionNotStarted::PublishBudgetOfTheLastUserChangeSpent,
@@ -3247,7 +3334,11 @@ impl MountedSession {
             return Ok(BackgroundCompactionStart::Starts {
                 trigger: trip.trigger,
                 reach: trip.reach,
-                budget: CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange,
+                budget: if is_the_exception_still_open {
+                    CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite
+                } else {
+                    CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange
+                },
                 batches_published_earlier_in_the_trip: trip.batches_published,
             });
         }
@@ -3360,6 +3451,7 @@ impl MountedSession {
         } = self;
         let (parameters, background_compute) = parameters_and_device_table
             .parameters_on_disk_and_the_background_compute_of_this_mount_mut();
+        let compaction_publishes_before_this_call = compaction.publishes_since_the_last_user_change;
         let mut run = run_compaction_on_with_the_region_check_of_the_intent_deletion(
             parameters,
             devices.as_mut_slice(),
@@ -3391,15 +3483,27 @@ impl MountedSession {
             run.outcome,
             CompactionRunOutcome::Stopped(CompactionRunStop::OnePublishOfTheTripDone)
         );
+        // 全空段低于底额时为前台 fsync 让路的这一趟不留着：留着的话下一次调用接着让路，前台 fsync 一直有就再也轮不到下一趟的破例。
+        let is_yielding_while_the_foreground_cannot_write = matches!(
+            run.outcome,
+            CompactionRunOutcome::Stopped(CompactionRunStop::YieldedToAForegroundFsync)
+        )
+            && foreground_cannot_write_under_the_switches(allocator);
         match span {
             SpanOfTheBackgroundCall::AtMostOnePublish
-                if is_held_back_by_the_pacing || is_done_with_this_call_only =>
+                if (is_held_back_by_the_pacing
+                    && !is_yielding_while_the_foreground_cannot_write)
+                    || is_done_with_this_call_only =>
             {
                 compaction.keep_the_background_trip_under_way(BackgroundTripUnderWay {
                     trigger,
                     reach,
                     batches_published: batches_published_earlier_in_the_trip
                         + run.batches_published,
+                    is_the_exception_of_the_trip_still_open: budget
+                        == CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite
+                        && compaction.publishes_since_the_last_user_change
+                            == compaction_publishes_before_this_call,
                 });
             }
             SpanOfTheBackgroundCall::AtMostOnePublish | SpanOfTheBackgroundCall::WholeTrip => {
@@ -4087,7 +4191,7 @@ pub(crate) fn run_compaction_on<Device: BlockDevice>(
 }
 
 /// 回退门（D16（发布语义） 已定项 1「回退候选集保留最近 4 个可退到的不同状态」，D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：
-/// 下一次发布写的那一槽与它之后留的余量（[`crate::compaction_plan::ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH`]）里的根一走，根环里不同用户状态就少于 4 个而且比此刻少，这次整理发布就不发。
+/// 下一次发布写的那一槽与它之后留的余量（[`crate::compaction_plan::root_ring_slots_left_by_a_compaction_publish`]）里的根一走，根环里不同用户状态就少于 4 个而且比此刻少，这次整理发布就不发。
 /// 此刻本来就不到 4 个、走的那几条又都不是某个状态的最后一条的，不是这次发布挤掉的，照发。读不通不发（[`CompactionRunStop::RollbackStatesOfTheRingUnreadable`]）。
 /// 只读，读根环与每条有效根的树表。
 fn stop_of_the_rollback_gate<Device: BlockDevice>(
@@ -4203,27 +4307,37 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
         CompactionBatchSize::OfTheMount(batch_units)
         | CompactionBatchSize::ForcedForTests(batch_units) => batch_units,
     };
-    // 次序（判决 G8）：删除续做在整理批之前。
-    let queue_is_empty = match &*current {
-        PoolVersion::WithFile(version_of_trees)
-        | PoolVersion::WithTreesAndNoFile(version_of_trees) => {
-            version_of_trees.queue_of_committed_intents_is_empty()
-        }
-        PoolVersion::WithoutFile(_) => true,
+    let (span_of_the_call, batches_published_earlier_in_the_trip) = match pacing {
+        PacingOfTheRun::NotPaced => (SpanOfTheBackgroundCall::WholeTrip, 0),
+        PacingOfTheRun::PacedByTheCaller {
+            span,
+            batches_published_earlier_in_the_trip,
+            ..
+        } => (span, batches_published_earlier_in_the_trip),
     };
-    if !queue_is_empty {
+    // 次序（判决 G8）：删除续做在整理批之前。放锁的后台入口一次调用至多走一步、发了就返回，队列长的时候前台在步与步之间等得到锁。
+    if !deletion_queue_of_the_version_is_empty(current) {
         let mut walked = Arc::clone(
             current
                 .shared_version_of_trees()
                 .expect("队列不空的一版是有树的一版"),
         );
-        let walk = publish_the_queue_until_it_is_empty(
-            parameters,
-            devices,
-            allocator,
-            &mut walked,
-            instance,
-        );
+        let walk = match span_of_the_call {
+            SpanOfTheBackgroundCall::WholeTrip => publish_the_queue_until_it_is_empty(
+                parameters,
+                devices,
+                allocator,
+                &mut walked,
+                instance,
+            ),
+            SpanOfTheBackgroundCall::AtMostOnePublish => publish_at_most_one_step_of_the_queue(
+                parameters,
+                devices,
+                allocator,
+                &mut walked,
+                instance,
+            ),
+        };
         *current = PoolVersion::of_a_shared_version_of_trees(walked);
         // 整理这一趟的版本列表按值交回（[`CompactionRun::versions_published`]）：走空的各步只有最后一步还与 `current` 共享，拷那一份。
         versions_published.extend(walk.steps_published.into_iter().map(Arc::unwrap_or_clone));
@@ -4237,6 +4351,16 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
                 CompactionRunOutcome::Failed(CompactionRunFailure::DeletionContinuation(cause)),
             );
         }
+        if span_of_the_call == SpanOfTheBackgroundCall::AtMostOnePublish {
+            return finish(
+                compaction,
+                current.root().checkpoint_txg,
+                versions_published,
+                background_compute_of_each_batch,
+                0,
+                CompactionRunOutcome::Stopped(CompactionRunStop::OnePublishOfTheTripDone),
+            );
+        }
     }
     let budget = match budget_rule {
         CompactionRunBudget::SinceTheLastUserChange => publish_budget_of_a_compaction_run(
@@ -4257,14 +4381,6 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
             .max(1)
         }
     };
-    let (span_of_the_call, batches_published_earlier_in_the_trip) = match pacing {
-        PacingOfTheRun::NotPaced => (SpanOfTheBackgroundCall::WholeTrip, 0),
-        PacingOfTheRun::PacedByTheCaller {
-            span,
-            batches_published_earlier_in_the_trip,
-            ..
-        } => (span, batches_published_earlier_in_the_trip),
-    };
     // 回退门只在后台入口、开关照水位判时判（显式入口没有产品调用方，可写挂载续做只推盘上那一条，发布数有上界）。
     let is_the_rollback_gate_judged = matches!(pacing, PacingOfTheRun::PacedByTheCaller { .. })
         && matches!(
@@ -4310,6 +4426,10 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
                 CompactionRunOutcome::Stopped(CompactionRunStop::OnePublishOfTheTripDone),
             );
         }
+        // 全空段低于底额、前台写不进时这一趟的第一次发布破例：不判回退门（前台等它腾段），也不为前台 fsync 让路（前台已写不进，让路换不来什么）。
+        let is_this_publish_the_exception = budget_rule
+            == CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite
+            && publishes_of_the_run == 0;
         // D26（后台整理与放置回收） 已定项 13 第 6 条：后台入口每一批之前问调用方，前台有 fsync 在等、写带宽到了上限就不发下一批。
         let write_bandwidth_limit = match pacing {
             PacingOfTheRun::NotPaced => CompactionWriteBandwidthLimit::Unlimited,
@@ -4319,7 +4439,8 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
                 ..
             } => {
                 match caller.foreground_fsync() {
-                    ForegroundFsync::Waiting => {
+                    ForegroundFsync::Waiting if is_this_publish_the_exception => {}
+                    ForegroundFsync::Waiting | ForegroundFsync::CallerIsStoppingItsThread => {
                         return finish(
                             compaction,
                             current.root().checkpoint_txg,
@@ -4349,10 +4470,6 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
                 write_bandwidth_limit
             }
         };
-        // 全空段低于底额、前台写不进时这一趟的第一次发布破例不判（前台等它腾段）；别的整理发布判过才发。
-        let is_this_publish_the_exception = budget_rule
-            == CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite
-            && publishes_of_the_run == 0;
         if is_the_rollback_gate_judged && !is_this_publish_the_exception {
             if let Some(stop) = stop_of_the_rollback_gate(parameters, &*devices, current) {
                 return finish(
@@ -6538,7 +6655,8 @@ impl MountedSessionWithPendingChanges {
     }
 
     /// 同 [`Self::run_compaction_in_the_background`]，只差至多一次发布就返回（D26（后台整理与放置回收） 已定项 13 第 5 条：前台与后台经一把锁，每次发布之间放锁）：
-    /// 这一趟没完时停因是 [`CompactionRunStop::OnePublishOfTheTripDone`]（或节奏截住），记在会话上，下一次调用不重判起跑、接着推。
+    /// 这一趟没完时停因是 [`CompactionRunStop::OnePublishOfTheTripDone`]（或节奏截住），记在会话上，下一次调用不重判起跑、接着推。整理之前的删除续做同样一次调用至多走一步。
+    /// 全空段低于底额时为前台 fsync 让路的那一趟不记（下一趟照样破例）。
     /// 两次调用之间前台收下的待发在下一次调用先排空。挂着的会话这一层不起线程、不持锁，放锁是调用方的事（运行库 `singlefs_mount_runtime`）。
     ///
     /// # Errors
@@ -6662,6 +6780,8 @@ impl MountedSessionWithPendingChanges {
                                 trigger,
                                 reach,
                                 batches_published: batches_published_earlier_in_the_trip,
+                                is_the_exception_of_the_trip_still_open: budget
+                                    == CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite,
                             },
                         );
                     }
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 1c435f0..cd58f49 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -32,8 +32,8 @@ use crate::background_compute::{
 use crate::block_device::BlockDevice;
 use crate::checksum::crc32_castagnoli;
 use crate::compaction_plan::{
-    CompactionBatchSize, CompactionMountParameters, CompactionOfThisMount, CompactionSwitches,
-    ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH,
+    root_ring_slots_left_by_a_compaction_publish, CompactionBatchSize, CompactionMountParameters,
+    CompactionOfThisMount, CompactionSwitches,
 };
 use crate::instance_table_sweep::{
     sweep_the_instance_table_at_the_end_of_the_mount, InstanceTableSweepAtTheEndOfTheMount,
@@ -4214,7 +4214,7 @@ pub fn newest_first_distinct_states(mut non_empty: Vec<NonEmptyValidRoot>) -> Ve
 pub(crate) struct RollbackStatesOfTheRing {
     /// 根环里此刻不同用户状态的个数。
     pub(crate) states_now: u64,
-    /// 下一次发布写的那一槽与它之后 [`ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH`] 槽里的根都走了之后还剩几个（走的那一条是某个状态在环里最后一条时才少一个）。
+    /// 下一次发布写的那一槽与它之后余量（[`root_ring_slots_left_by_a_compaction_publish`]）那几槽里的根都走了之后还剩几个（走的那一条是某个状态在环里最后一条时才少一个）。
     pub(crate) states_after_the_next_publish_and_the_margin: u64,
     /// 比最新那个用户状态的根新的有效根条数（环里一个状态都没有时是有效根的条数）：挂载那一份预算当它是已经过去的发布数。
     pub(crate) publishes_after_the_newest_state: u64,
@@ -4255,7 +4255,9 @@ pub(crate) fn rollback_states_of_the_ring(
 ) -> Result<RollbackStatesOfTheRing, RecoveryFailure> {
     let table = instance_table_chain_of_root(reader, current_root)?.records;
     let ring_slots_of_the_next_publish_and_the_margin: Vec<RootRingSlot> = (1..=1
-        + ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH)
+        + root_ring_slots_left_by_a_compaction_publish(
+            parameters.geometry.root_ring_slots_per_region,
+        ))
         .map(|publishes_ahead| {
             target_for_publish(
                 CheckpointTxg(current_root.checkpoint_txg.0 + publishes_ahead),
diff --git a/crates/singlefs-mount-runtime/src/lib.rs b/crates/singlefs-mount-runtime/src/lib.rs
index e8d85e5..40627d5 100644
--- a/crates/singlefs-mount-runtime/src/lib.rs
+++ b/crates/singlefs-mount-runtime/src/lib.rs
@@ -7,8 +7,10 @@
 //!
 //! 线程体之外有同步的 [`MountedRuntime::run_one_maintenance_pass`]：确定性测试不起线程，一拍一拍推，录制盘（不能跨线程）也走得通。
 //! 一个维护线程、两个到期时刻：`publish_poll_period` 到期调 `publish_when_due`（D16（发布语义） 丢失窗口里的 P），`compaction_period` 到期推后台整理。
+//! 每一拍（两个到期时刻都算）先重发冻结的发布：写盘报错之后前台的写都被拒，等不到整理到期。
 //!
-//! 卸载等线程、排空待发、走核心层的卸载；丢弃（Drop）只停线程，不写盘，等价于崩溃（下一次挂载走恢复）。线程或前台在锁里 panic 等价于崩溃：
+//! 卸载等线程、重发冻结的发布、排空待发、走核心层的卸载；重发还失败就交回错误，盘交回，等价于崩溃（[`UnmountRefused::FrozenPublishResendFailed`]）。
+//! 丢弃（Drop）只停线程，不写盘，等价于崩溃（下一次挂载走恢复）。线程或前台在锁里 panic 等价于崩溃：
 //! 会话可能停在半个更新里，不接着用，之后每个入口报 [`RuntimeFailed`]，只能丢掉内存重挂。
 #![deny(unsafe_code)]
 
@@ -16,6 +18,7 @@ use std::any::Any;
 use std::fmt;
 use std::panic::{catch_unwind, AssertUnwindSafe};
 use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
+use std::sync::mpsc::channel;
 use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
 use std::thread::JoinHandle;
 use std::time::{Duration, Instant};
@@ -296,7 +299,12 @@ pub enum UnmountRefused<Device> {
         devices: Vec<(DeviceIdentity, Device)>,
         failure: RuntimeFailed,
     },
-    /// 排空待发没发成：运行库原样交回（维护线程已经停了），待发没冻结时还在里面。可以重发冻结的那一次或腾出空间之后再卸载；丢掉它就是崩溃。
+    /// 卸载先重发冻结的那一次，重发又报错：盘交回，内存不接着用，等价于崩溃（冻结的那一次没落盘，下一次挂载走恢复，回到它之前最后发成的那一版）。
+    FrozenPublishResendFailed {
+        devices: Vec<(DeviceIdentity, Device)>,
+        refusal: Box<PublishError>,
+    },
+    /// 排空待发没发成：运行库原样交回，待发没冻结时还在里面。维护线程已经停了，它不再自己发布与整理：腾出空间之后再调一次卸载（卸载先重发冻结的那一次）；丢掉它就是崩溃。
     PendingChangesNotPublished {
         runtime: MountedRuntime<Device>,
         refusal: Box<UserChangeRefused>,
@@ -315,6 +323,10 @@ impl<Device> fmt::Debug for UnmountRefused<Device> {
                 .debug_struct("Failed")
                 .field("failure", failure)
                 .finish_non_exhaustive(),
+            UnmountRefused::FrozenPublishResendFailed { refusal, .. } => formatter
+                .debug_struct("FrozenPublishResendFailed")
+                .field("refusal", refusal)
+                .finish_non_exhaustive(),
             UnmountRefused::PendingChangesNotPublished { refusal, .. } => formatter
                 .debug_struct("PendingChangesNotPublished")
                 .field("refusal", refusal)
@@ -348,7 +360,7 @@ struct Shared<Device> {
     receiver: Arc<dyn ReceiverOfTheMaintenanceEvents>,
 }
 
-/// 后台入口每一批之前问的两样（核心层不起线程，钟与前台 fsync 都在这一层）。停线程时也答「在等」：整理在下一批之前就返回，线程很快 join 得上。
+/// 后台入口每一批之前问的两样（核心层不起线程，钟与前台 fsync 都在这一层）。停线程时答「在停线程」：整理在下一批之前就返回，线程很快 join 得上。
 struct CallerOfTheRuntime<'shared> {
     clock: &'shared dyn ClockOfTheRuntime,
     fsync_waiters: &'shared AtomicU32,
@@ -361,7 +373,9 @@ impl CallerOfTheBackgroundCompaction for CallerOfTheRuntime<'_> {
     }
 
     fn foreground_fsync(&self) -> ForegroundFsync {
-        if self.fsync_waiters.load(Ordering::SeqCst) > 0 || self.stopping.load(Ordering::SeqCst) {
+        if self.stopping.load(Ordering::SeqCst) {
+            ForegroundFsync::CallerIsStoppingItsThread
+        } else if self.fsync_waiters.load(Ordering::SeqCst) > 0 {
             ForegroundFsync::Waiting
         } else {
             ForegroundFsync::NoneWaiting
@@ -473,7 +487,7 @@ impl<Device: BlockDevice> Shared<Device> {
         Ok(())
     }
 
-    /// 上一批写盘报错冻结了发布就重发一次。交回这一拍还冻不冻着：冻着时整理不跑（每次都是签发一次、被拒一次）。
+    /// 上一次写盘报错冻结了发布就重发一次。交回这一拍还冻不冻着：冻着时 `publish_when_due` 与整理都不跑（每次都是被拒一次）。
     fn resend_the_frozen_publish_if_any(&self) -> Result<bool, RuntimeFailed> {
         let mut locked = self.lock_the_session()?;
         let SessionAndDevices { session, devices } = &mut *locked;
@@ -500,15 +514,15 @@ impl<Device: BlockDevice> Shared<Device> {
         }
     }
 
-    /// 一拍：`publish_when_due`、重发冻结的那一次，再一次次调放锁的后台入口，每次之间放锁。没起跑、被节奏截住、出错或到了调用次数上限就停。
+    /// 一拍：重发冻结的那一次、`publish_when_due`，再一次次调放锁的后台入口，每次之间放锁。没起跑、被节奏截住、出错或到了调用次数上限就停。
     /// 用户改动之后那一趟（至多一批）停了接着调：定时兜底那一支在同一拍里轮得到，用户改动不断时整理不会只剩每拍一批。
     fn run_one_maintenance_pass(&self) -> Result<MaintenancePass, RuntimeFailed> {
         self.update_the_observations(|observations| observations.maintenance_passes += 1);
-        self.publish_when_due_once()?;
         let mut pass = MaintenancePass::default();
         if self.resend_the_frozen_publish_if_any()? {
             return Ok(pass);
         }
+        self.publish_when_due_once()?;
         let pacing = self.runtime_parameters.compaction_pacing();
         let caller = self.caller();
         while pass.compaction_calls < COMPACTION_CALLS_PER_MAINTENANCE_PASS_AT_MOST
@@ -602,7 +616,7 @@ impl<Device: BlockDevice> Shared<Device> {
                     } else {
                         compaction_period
                     };
-            } else {
+            } else if !self.resend_the_frozen_publish_if_any()? {
                 self.publish_when_due_once()?;
             }
             let until_the_next_compaction =
@@ -667,10 +681,14 @@ impl<Device: BlockDevice + Send + 'static> MountedRuntime<Device> {
             clock,
             receiver,
         )?;
-        let shared_of_the_thread = Arc::clone(&runtime.shared);
+        // 线程起来之后才把这一份交给它：起不来时没有第二个持有者，盘交得回去。
+        let (hand_over_the_shared, shared_handed_over) = channel::<Arc<Shared<Device>>>();
         let started = std::thread::Builder::new()
             .name("singlefs-maintenance".to_string())
             .spawn(move || {
+                let Ok(shared_of_the_thread) = shared_handed_over.recv() else {
+                    return;
+                };
                 let body = catch_unwind(AssertUnwindSafe(|| {
                     shared_of_the_thread.maintenance_thread_body()
                 }));
@@ -689,6 +707,9 @@ impl<Device: BlockDevice + Send + 'static> MountedRuntime<Device> {
             });
         match started {
             Ok(thread) => {
+                // 送不出去只有一种：线程在收之前就结束了，这一份留在这里随运行库走。
+                let _thread_gone_before_taking_it =
+                    hand_over_the_shared.send(Arc::clone(&runtime.shared));
                 runtime.maintenance_thread = Some(thread);
                 Ok(RuntimeMounted { output, runtime })
             }
@@ -697,7 +718,7 @@ impl<Device: BlockDevice + Send + 'static> MountedRuntime<Device> {
                 drop(runtime);
                 let devices_handed_back = Arc::try_unwrap(shared)
                     .unwrap_or_else(|_still_shared| {
-                        panic!("维护线程没起来：运行库是这一份的唯一持有者")
+                        panic!("维护线程没起来，从没拿到过这一份；运行库已丢：这一份是唯一持有者")
                     })
                     .locked
                     .into_inner()
@@ -911,7 +932,7 @@ impl<Device: BlockDevice> MountedRuntime<Device> {
         Ok(work(&mut *locked))
     }
 
-    /// 同步的一拍：线程体循环里那一次（`publish_when_due`、重发冻结的那一次、放锁的后台入口推到没得推）。不起线程的确定性测试直接调。
+    /// 同步的一拍：线程体循环里那一次（重发冻结的那一次、`publish_when_due`、放锁的后台入口推到没得推）。不起线程的确定性测试直接调。
     ///
     /// # Errors
     /// 运行库坏了。
@@ -945,11 +966,11 @@ impl<Device: BlockDevice> MountedRuntime<Device> {
         }
     }
 
-    /// 卸载：停线程 → 排空待发 → 走核心层的卸载 → 丢会话（GPU 把手随它丢，有没交完的提交的那张卡整张泄漏、记进开卡器那份账）→ 盘交回。
+    /// 卸载：停线程 → 重发冻结的那一次（没有冻结着的发布就什么都不写）→ 排空待发 → 走核心层的卸载 → 丢会话（GPU 把手随它丢，有没交完的提交的那张卡整张泄漏、记进开卡器那份账）→ 盘交回。
     /// 活意图不用做完：留在盘上，下次挂载续做。
     ///
     /// # Errors
-    /// [`UnmountRefused`] 的三种。
+    /// [`UnmountRefused`] 的四种。重发冻结的那一次又报错（[`UnmountRefused::FrozenPublishResendFailed`]）等价于崩溃：冻结的那一次没落盘，下一次挂载走恢复。
     pub fn unmount(mut self) -> Result<UnmountedPool<Device>, Box<UnmountRefused<Device>>> {
         self.stop_and_join_the_maintenance_thread();
         let failure = self.shared.failure_so_far();
@@ -968,7 +989,7 @@ impl<Device: BlockDevice> MountedRuntime<Device> {
         let locked_or_poisoned = locked.into_inner();
         let is_poisoned = locked_or_poisoned.is_err();
         let SessionAndDevices {
-            session,
+            mut session,
             mut devices,
         } = locked_or_poisoned.unwrap_or_else(PoisonError::into_inner);
         if failure.is_some() || is_poisoned {
@@ -979,6 +1000,12 @@ impl<Device: BlockDevice> MountedRuntime<Device> {
                 }),
             }));
         }
+        if let Err(refusal) = session.resend_the_frozen_publish(&mut devices) {
+            return Err(Box::new(UnmountRefused::FrozenPublishResendFailed {
+                devices,
+                refusal: Box::new(refusal),
+            }));
+        }
         let now = clock.reading();
         let (pending_changes_published, mut session) =
             match session.into_session_after_publishing_the_pending_changes(&mut devices, now) {
diff --git a/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs b/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
index ef8f463..7d72610 100644
--- a/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
+++ b/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
@@ -4,7 +4,7 @@
 //!
 //! 开卡（枚举适配器在内：wgpu 在造 future 那一步同步做完枚举，等待上限管不到它）放工作线程，调用线程至多等 `opening_deadline`。
 //! 到点挂载拒绝，工作线程留在驱动里不杀（没有安全的杀法）；它回来之前闸住新开卡，不再多起一条枚举线程
-//! （[`BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung`]；同一个驱动加载器里再起一条会不会也挂没量过）。它回来之后开出来的东西没人要，随丢弃记进泄漏账，闸放开。
+//! （[`BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung`]；同一个驱动加载器里再起一条会不会也挂没量过）。它回来之后开出来的东西没人要，随丢弃记进泄漏账，闸放开；它 panic 了闸同样放开。
 //! 实例对象在超时之后的状态读不到：造不出真挂住的驱动，这一格核不了。
 //!
 //! 实例枚举到的驱动清单定在建它的那一刻：修好驱动要新建。[`GpuOpenerOfThisProcess::forget_the_instance_so_the_next_opening_creates_one`] 显式重建；
@@ -47,10 +47,23 @@ type OpeningOnTheWorkerThread<Instance> = Arc<
 /// 建这个进程的一个实例。
 type InstanceMaker<Instance> = Arc<dyn Fn() -> Instance + Send + Sync>;
 
-/// 进程级实例与它的计数。
+/// 进程级实例与它的计数。建实例不持这把锁（驱动可能挂在那里）：读计数、丢实例都不被挂住。
 struct InstanceOfThisProcess<Instance> {
     instance: Option<Instance>,
     instances_created: u32,
+    /// 丢过几次实例：建实例途中丢过的，建好的那一个这一次开卡照用、不留给下一次开卡（它是丢之前的驱动清单建的）。
+    instances_forgotten: u64,
+}
+
+/// 工作线程结束时放开闸（走到末尾与 panic 展开都走到）。
+struct GateOpenedWhenTheOpeningEnds {
+    an_opening_is_hung: Arc<AtomicBool>,
+}
+
+impl Drop for GateOpenedWhenTheOpeningEnds {
+    fn drop(&mut self) {
+        self.an_opening_is_hung.store(false, Ordering::SeqCst);
+    }
 }
 
 /// 见模块说明。泛型 `Instance` 只为让不要卡的单测换上不碰 Vulkan 的替身；产品是 [`VulkanInstanceOfThisProcess`]（[`Self::of_this_process`]）。
@@ -125,6 +138,7 @@ impl<Instance: Clone + Send + 'static> GpuOpenerOfThisProcess<Instance> {
             instance_of_this_process: Arc::new(Mutex::new(InstanceOfThisProcess {
                 instance: None,
                 instances_created: 0,
+                instances_forgotten: 0,
             })),
             memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
             make_an_instance,
@@ -149,13 +163,16 @@ impl<Instance: Clone + Send + 'static> GpuOpenerOfThisProcess<Instance> {
 
     /// 修好驱动之后用：丢掉这个进程留着的实例，下一次开卡新建一个（已开着的把手持有自己的设备，不动）。只改内存，新建的计数不还。
     pub fn forget_the_instance_so_the_next_opening_creates_one(&self) {
-        self.instance_of_this_process
+        let mut held = self
+            .instance_of_this_process
             .lock()
-            .unwrap_or_else(PoisonError::into_inner)
-            .instance = None;
+            .unwrap_or_else(PoisonError::into_inner);
+        held.instance = None;
+        held.instances_forgotten += 1;
     }
 
-    /// 这一次开卡用的实例：有就用留着的，没有就新建、计数加一；计数到上限就拒。在工作线程上调：建实例装 Vulkan 加载器，驱动也可能挂在这里。
+    /// 这一次开卡用的实例：有就用留着的，没有就新建、计数加一（额度在建之前就记，建到一半挂住也算用了一个）；计数到上限就拒。
+    /// 在工作线程上调：建实例装 Vulkan 加载器，驱动也可能挂在这里，所以建的时候不持那把锁。同一时刻至多一条工作线程走到这里（闸）。
     fn the_instance_or_a_new_one_under_the_limit(
         instance_of_this_process: &Mutex<InstanceOfThisProcess<Instance>>,
         make_an_instance: &(dyn Fn() -> Instance + Send + Sync),
@@ -171,9 +188,16 @@ impl<Instance: Clone + Send + 'static> GpuOpenerOfThisProcess<Instance> {
                 instances_created: held.instances_created,
             });
         }
-        let instance = make_an_instance();
         held.instances_created += 1;
-        held.instance = Some(instance.clone());
+        let instances_forgotten_before_the_making = held.instances_forgotten;
+        drop(held);
+        let instance = make_an_instance();
+        let mut held_after_the_making = instance_of_this_process
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner);
+        if held_after_the_making.instances_forgotten == instances_forgotten_before_the_making {
+            held_after_the_making.instance = Some(instance.clone());
+        }
         Ok(instance)
     }
 }
@@ -201,6 +225,8 @@ impl<Instance: Clone + Send + 'static> OpensTheBackgroundGpu for GpuOpenerOfThis
         let started = std::thread::Builder::new()
             .name("singlefs-gpu-opener".to_string())
             .spawn(move || {
+                let gate_opened_when_the_opening_ends =
+                    GateOpenedWhenTheOpeningEnds { an_opening_is_hung };
                 let outcome = Self::the_instance_or_a_new_one_under_the_limit(
                     &instance_of_this_process,
                     &*make_an_instance,
@@ -212,7 +238,7 @@ impl<Instance: Clone + Send + 'static> OpensTheBackgroundGpu for GpuOpenerOfThis
                         })
                 });
                 // 闸先放开再送：调用线程收到结局时下一次开卡不会被这一次闸住。
-                an_opening_is_hung.store(false, Ordering::SeqCst);
+                drop(gate_opened_when_the_opening_ends);
                 // 送不出去只有一种：调用线程到点不等了、这次的结局没人要，开成的把手随丢弃记进泄漏账。
                 let _nobody_waits_for_this_opening_any_more = outcome_sender.send(outcome);
             });
@@ -228,7 +254,6 @@ impl<Instance: Clone + Send + 'static> OpensTheBackgroundGpu for GpuOpenerOfThis
                 deadline: self.opening_deadline,
             }),
             Err(RecvTimeoutError::Disconnected) => {
-                self.an_opening_is_hung.store(false, Ordering::SeqCst);
                 Err(BackgroundGpuNotOpened::RefusedByTheOpening {
                     description: "开卡的工作线程 panic 了".to_string(),
                 })
````
