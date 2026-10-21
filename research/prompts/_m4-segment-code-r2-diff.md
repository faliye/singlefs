# m4-segment-code-r2：第一轮修复批（两批）对被判产品代码的改动全文（2026-10-16）

相对第一轮冻结副本（`~/.local/share/singlefs/work/m4-segment-code-r1/repo/`）的 `diff -u`；测试代码与池级 checker、checker 档的改动不在这里，见冻结副本。

````diff
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -497,6 +497,23 @@
         });
     }
 
+    /// 签发那次发布照记账换了 `segs0`（`transaction::publish_compaction_step`，判决 `research/prompts/m4-segment-code-r1-main-verification.md`
+    /// 第三节 F3：`segs0` 取那次发布写下的记账第 11 项）：这里记着的活意图是调用方交进去的 `handed_in` 那一条时，换成盘上写下的 `written`，
+    /// R 与排不排他不变；记着别的（或没有）就不动。
+    pub fn take_the_intent_written_by_the_issue(
+        &mut self,
+        handed_in: CompactionIntent,
+        written: CompactionIntent,
+    ) {
+        if let Some(live) = self
+            .live_intent
+            .as_mut()
+            .filter(|live| live.intent == handed_in)
+        {
+            live.intent = written;
+        }
+    }
+
     /// 意图删了（达成或丢掉，删它的那次发布已落盘），或签发那次发布没发成：撤掉活意图连同 R 排他，交回撤掉的那一条。
     pub fn withdraw_the_live_intent(&mut self) -> Option<CompactionIntent> {
         self.live_intent.take().map(|live| live.intent)
@@ -804,7 +821,10 @@
     /// 另一块盘上的记录：选段只数单元区那块盘上的活槽，核的入参里没有它们。
     const ANOTHER_DEVICE: DeviceIdentity = DeviceIdentity(1);
     const MOST_SEGMENTS_IN_THE_UNIT_AREA: usize = 3;
-    const MOST_LIVE_RECORDS_IN_A_SEGMENT: u64 = 2;
+    /// 一段里各条活记录的跨度（槽数），一条一项（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F8：输入域加跨度 2 的记录、
+    /// 段内多于 2 条活记录）。选段数的是活槽、不是条数：跨度 2 的一条与跨度 1 的两条活槽相同、条数不同，跨度 2 的一条与跨度 1 的一条条数相同、
+    /// 活槽不同，打分改成数条数时这两种并排的段排序就变了（`crates/mutations.tsv` 一行）。
+    const LIVE_RECORD_SPANS_OF_A_SEGMENT: [&[u16]; 5] = [&[], &[1], &[2], &[1, 1], &[1, 1, 1]];
     const MOST_ALLOCATIONS_IN_A_REGION: usize = 4;
     const LARGEST_BATCH_UNITS: usize = 3;
     /// 每段里一条已释放的记录落在这个偏移上：选段不数它，核的入参里也没有它。
@@ -838,11 +858,21 @@
 
     #[derive(Clone, Copy, Debug)]
     struct SegmentShape {
-        live_records: u64,
+        live_record_spans: &'static [u16],
         exclusion: SegmentExclusion,
         scan: CompactionKernelSegmentScan,
     }
 
+    impl SegmentShape {
+        /// 这一段的活槽数：各条活记录的跨度之和（核的入参照它直接写出，不经被测的选段算）。
+        fn live_slots(&self) -> u64 {
+            self.live_record_spans
+                .iter()
+                .map(|span_slots| u64::from(*span_slots))
+                .sum()
+        }
+    }
+
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     enum AllocationShape {
         Movable,
@@ -895,11 +925,16 @@
         .expect("默认单元区起点在聚簇段边界上，整段起点都是 64 的倍数")
     }
 
-    fn record(device: DeviceIdentity, slot: u64, is_released: bool) -> AllocationRecord {
+    fn record(
+        device: DeviceIdentity,
+        slot: u64,
+        span_slots: u16,
+        is_released: bool,
+    ) -> AllocationRecord {
         AllocationRecord {
             device,
             slot: SlotNumber(slot),
-            span_slots: 1,
+            span_slots,
             generation: CheckpointTxg(1),
             is_released,
         }
@@ -925,21 +960,25 @@
         let mut segments_refused = SegmentsRefusedByTheRegionScan::default();
         for (segment_index, shape) in shapes.iter().enumerate() {
             let start_slot = segment_start_slot(segment_index);
-            for live_record in 0..shape.live_records {
+            // 第 i 条活记录起在段起点之后 2i：跨度至多 2，各条互不重叠。
+            for (live_record, span_slots) in (0_u64..).zip(shape.live_record_spans) {
                 allocation_records.push(record(
                     DEVICE_OF_THE_UNIT_AREA,
                     start_slot + 2 * live_record,
+                    *span_slots,
                     false,
                 ));
                 allocation_records.push(record(
                     ANOTHER_DEVICE,
                     start_slot + 2 * live_record + 1,
+                    1,
                     false,
                 ));
             }
             allocation_records.push(record(
                 DEVICE_OF_THE_UNIT_AREA,
                 start_slot + RELEASED_RECORD_OFFSET_IN_THE_SEGMENT,
+                1,
                 true,
             ));
             let region = region_of_the_segment(segment_index);
@@ -1010,7 +1049,7 @@
             .iter()
             .enumerate()
             .map(|(segment_index, shape)| CompactionKernelSegment {
-                live_slots: shape.live_records,
+                live_slots: shape.live_slots(),
                 is_opened_in_this_mount: shape.exclusion == SegmentExclusion::OpenedInThisMount,
                 is_the_live_intent_region: live_intent_segment == Some(segment_index),
                 holds_a_pinned_slot: shape.exclusion == SegmentExclusion::HoldsAPinnedSlot,
@@ -1030,15 +1069,16 @@
         }
     }
 
-    /// 一到三段、每段活槽 0 到 2、至多一条不合资格、三种扫描结果、活意图的 R 是哪一段（或没有），全部组合：
-    /// 这里的选段与证过 P1 的核逐项相同；三种答案都至少出现过一次（对拍不空转）。
+    /// 一到三段、每段的活记录取 [`LIVE_RECORD_SPANS_OF_A_SEGMENT`] 那五种（零到三条、跨度 1 或 2）、至多一条不合资格、三种扫描结果、
+    /// 活意图的 R 是哪一段（或没有），全部组合：这里的选段与证过 P1 的核逐项相同；三种答案都至少出现过一次（对拍不空转）。
     #[test]
     fn choose_region_agrees_with_the_verified_segment_choice_on_every_small_unit_area() {
-        let shapes: Vec<SegmentShape> = (0..=MOST_LIVE_RECORDS_IN_A_SEGMENT)
-            .flat_map(|live_records| {
+        let shapes: Vec<SegmentShape> = LIVE_RECORD_SPANS_OF_A_SEGMENT
+            .into_iter()
+            .flat_map(|live_record_spans| {
                 SEGMENT_EXCLUSIONS.into_iter().flat_map(move |exclusion| {
                     SEGMENT_SCANS.into_iter().map(move |scan| SegmentShape {
-                        live_records,
+                        live_record_spans,
                         exclusion,
                         scan,
                     })
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -60,13 +60,14 @@
 use crate::root_record::{RootRecord, UnmountMarker};
 use crate::root_ring::{target_for_publish, RootRingSlot};
 use crate::transaction::{
-    central_mapping_locations_in_the_version, prepare_the_version_publish, publish_compaction_step,
-    publish_create_snapshot, publish_new_inodes, publish_overwrite, publish_sequential_write,
+    central_mapping_locations_in_the_version, prepare_the_version_publish,
+    publish_compaction_step_with_the_region_check_of_the_intent_deletion, publish_create_snapshot,
+    publish_new_inodes, publish_overwrite, publish_sequential_write,
     publish_snapshot_destruction_intent, publish_unlink_intent, publish_version,
     walk_the_deletion_queue_until_it_is_empty, CompactionIntentStep,
     DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile, MappingLookup, PoolVersion,
-    PoolWriter, PublishError, ReleaseChecksumCheck, RelocationBatchRefused, TransactionOutput,
-    TransactionUnit,
+    PoolWriter, PublishError, RegionCheckOfTheIntentDeletion, ReleaseChecksumCheck,
+    RelocationBatchRefused, TransactionOutput, TransactionUnit,
 };
 use crate::unit::{unit_filesystem_identifier, UNIT_CLASS_DATA};
 use crate::unlink::DeletionSwitches;
@@ -107,7 +108,8 @@
     },
     /// 建一个只读快照（`transaction::publish_create_snapshot`，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S2）：
     /// 先把现行那一版的 `deleted_inodes` 走空（同别的改动，D8（核心索引结构） 已定项 16 第 6 条），再发一次独立发布，不与别的改动合在一起。
-    /// 现行那一版有树而文件集合为空时同覆盖写拒成 `NoFileVersionToChange`（第 5 条：删完之后只许建 inode、写行与空发布）。
+    /// 现行那一版有树而文件集合为空时照常发布（用户 2026-10-16 定「放开」，判决 `research/prompts/m4-segment-code-r1-main-verification.md`
+    /// 第三节 F5）：快照抄的是那一刻的两棵空树，读它读到没有文件（`recovery::SnapshotFileReading::NoFile`）。
     CreateSnapshot,
     /// 删一个只读快照（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4 与第七节岔路 2、4）：快照按它码 14 条目的树号点名
     /// （岔路 5）。一次调用 = 先把现行那一版的队列走空 + 意图那次发布（`transaction::publish_snapshot_destruction_intent`）+ 全部批
@@ -1287,6 +1289,27 @@
         &mut self,
         devices: &mut Vec<(DeviceIdentity, Device)>,
     ) -> Result<CompactionRun, CompactionRunRefused> {
+        self.run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region(
+            devices,
+            RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
+        )
+    }
+
+    /// 同 [`Self::run_compaction`]，只差一处：一批照抄意图核出腾空了 R、照同一批删意图再发时，删意图那一支落盘之前的核照 `region_check` 判
+    /// （判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F2）。别处删意图（最后一批、R 已空的只删意图）照记录核，不读它。
+    /// 产品路径只经 [`Self::run_compaction`] 进来（[`RegionCheckOfTheIntentDeletion::JudgedByTheRecords`]）；只供测试的
+    /// [`RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests`] 强制进入「删意图那一支核不过、照这一批不删意图再发」
+    /// 那一支（`.claude/rules/fs-design.md`「五条硬要求」第 2 条）：发出去的那一版 R 按记录全空、意图还在，下一次发布照「R 已按记录全空」只删意图。
+    ///
+    /// # Errors
+    /// 同 [`Self::run_compaction`]。
+    pub fn run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region<
+        Device: BlockDevice,
+    >(
+        &mut self,
+        devices: &mut Vec<(DeviceIdentity, Device)>,
+        region_check: RegionCheckOfTheIntentDeletion,
+    ) -> Result<CompactionRun, CompactionRunRefused> {
         let MountedSession {
             allocator,
             current,
@@ -1308,7 +1331,7 @@
             PoolVersion::WithoutFile(_) => return Err(CompactionRunRefused::NoVersionOfTrees),
         }
         let parameters = parameters_and_device_table.parameters_on_disk();
-        let mut run = run_compaction_on(
+        let mut run = run_compaction_on_with_the_region_check_of_the_intent_deletion(
             parameters,
             devices.as_mut_slice(),
             allocator,
@@ -1316,6 +1339,7 @@
             current,
             *instance,
             CompactionRunReach::UntilAStopOrTheBudget,
+            region_check,
         );
         run.discards = discard_the_segments_reusable_as_a_whole(
             devices.as_mut_slice(),
@@ -1384,10 +1408,11 @@
                 UserChange::NewInodes { .. } | UserChange::DestroySnapshot { .. } => {
                     emptied_version
                 }
+                // 文件集合为空时建快照放行、照常发布，读它读到空（用户 2026-10-16 定「放开」，判决 m4-segment-code-r1 第三节 F5）。
+                UserChange::CreateSnapshot => emptied_version,
                 UserChange::Overwrite(_)
                 | UserChange::SequentialWrite(_)
-                | UserChange::Unlink { .. }
-                | UserChange::CreateSnapshot => {
+                | UserChange::Unlink { .. } => {
                     return Err(no_file_version_to_change(published_in_this_change));
                 }
             },
@@ -1893,6 +1918,34 @@
     instance: InstanceGeneration,
     reach: CompactionRunReach,
 ) -> CompactionRun {
+    run_compaction_on_with_the_region_check_of_the_intent_deletion(
+        parameters,
+        devices,
+        allocator,
+        compaction,
+        current,
+        instance,
+        reach,
+        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
+    )
+}
+
+/// 同 [`run_compaction_on`]，一批腾空了 R、照同一批删意图再发时删意图那一支的核照 `region_check` 判
+/// （[`MountedSession::run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region`]）。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、推到哪为止、删意图那一支的核怎么判"
+)]
+fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut [(DeviceIdentity, Device)],
+    allocator: &mut PoolAllocator,
+    compaction: &mut CompactionOfThisMount,
+    current: &mut PoolVersion,
+    instance: InstanceGeneration,
+    reach: CompactionRunReach,
+    region_check: RegionCheckOfTheIntentDeletion,
+) -> CompactionRun {
     compaction.observations.runs_started += 1;
     let mut versions_published: Vec<TransactionOutput> = Vec::new();
     let finish = |compaction_of_the_run: &mut CompactionOfThisMount,
@@ -2001,14 +2054,19 @@
             batch_units,
             reach,
             &mut last_completed_intent,
+            region_check,
         ) {
             Ok(CompactionStep::Published { version, kind }) => {
                 publishes_of_the_run += 1;
                 compaction.publishes_since_the_last_user_change += 1;
                 compaction.observations.root_ring_slots_used += 1;
                 match kind {
+                    // 一批 = 搬了单元的那一次（判决 m4-segment-code-r1 第三节 F2：没搬单元的发布不记成一批——删意图的那一批计划里的单元
+                    // 都是这次发布本来就重写的节点时，它就是一次只删意图的发布）。
                     CompactionPublishKind::Batch
-                    | CompactionPublishKind::BatchCompletingTheIntent => {
+                    | CompactionPublishKind::BatchCompletingTheIntent
+                        if !version.units_relocated_by_this_publish.is_empty() =>
+                    {
                         batches_published += 1;
                         compaction.observations.batches_published += 1;
                         let relocated =
@@ -2017,7 +2075,9 @@
                         compaction.observations.units_relocated += relocated;
                         compaction.observations.units_relocated_by_the_last_batch = relocated;
                     }
-                    CompactionPublishKind::IntentIssued
+                    CompactionPublishKind::Batch
+                    | CompactionPublishKind::BatchCompletingTheIntent
+                    | CompactionPublishKind::IntentIssued
                     | CompactionPublishKind::IntentCompletedWithoutABatch
                     | CompactionPublishKind::IntentDropped
                     | CompactionPublishKind::IntentDroppedWithNothingMovable => {}
@@ -2138,7 +2198,7 @@
     }
 }
 
-/// 在一个新开的写入口上发整理的一次发布（[`publish_compaction_step`]），接在 `previous` 后面。
+/// 在一个新开的写入口上发整理的一次发布（`transaction::publish_compaction_step`），接在 `previous` 后面；删意图那一支照记录核。
 fn publish_the_compaction_step_once<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut [(DeviceIdentity, Device)],
@@ -2148,15 +2208,46 @@
     intent_step: CompactionIntentStep,
     instance: InstanceGeneration,
 ) -> Result<TransactionOutput, RelocationBatchRefused> {
+    publish_the_compaction_step_once_with_the_region_check_of_the_intent_deletion(
+        parameters,
+        devices,
+        allocator,
+        previous,
+        planned,
+        intent_step,
+        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
+        instance,
+    )
+}
+
+/// 同 [`publish_the_compaction_step_once`]，删意图那一支的核照 `region_check_of_the_intent_deletion` 判
+/// （[`publish_compaction_step_with_the_region_check_of_the_intent_deletion`]）。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：参数、盘表、分配器、上一版、计划、对意图做什么、删意图那一支的核怎么判、实例代号"
+)]
+fn publish_the_compaction_step_once_with_the_region_check_of_the_intent_deletion<
+    Device: BlockDevice,
+>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut [(DeviceIdentity, Device)],
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    planned: &[PlannedRelocation],
+    intent_step: CompactionIntentStep,
+    region_check_of_the_intent_deletion: RegionCheckOfTheIntentDeletion,
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, RelocationBatchRefused> {
     let switches = allocator.deletion_switches();
     let mut writer = PoolWriter::new(parameters, devices);
     writer.set_container_tree_capacities(switches.container_tree_capacities());
-    publish_compaction_step(
+    publish_compaction_step_with_the_region_check_of_the_intent_deletion(
         &mut writer,
         allocator,
         previous,
         planned,
         intent_step,
+        region_check_of_the_intent_deletion,
         instance,
     )
 }
@@ -2173,7 +2264,10 @@
         | RelocationBatchRefused::NoLiveCompactionIntent
         | RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
             ..
-        } => false,
+        }
+        | RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried { .. } => {
+            false
+        }
     }
 }
 
@@ -2181,17 +2275,20 @@
 /// 分配器这一侧的 R 排他与它一致（会话签发、删意图时装上与撤掉，挂载按 logged_ops 装上）。先把发布路径隔离过的槽并进常驻隔离集。
 /// 1. 没有活意图：可写挂载续做停在「没有活意图可续」（只续做、不签新的）；上一条完成的意图全局差值为假就停（攻方 K-h）；选段（强制选的那一段，
 ///    或 greedy 先扫后选），选不出就停；签发前判 R 外非全空段的空洞 ≥ R 里要搬的单元数（攻方 K-g），不够就停；签发：先在分配器上排他 R
-///    （签发那次发布自己的新落点也不进 R），发一次只把意图写进 logged_ops 的发布（`c0` = 现行那一版的 txg，`segs0` = R 那块盘按记录口径的全空段数）；
+///    （签发那次发布自己的新落点也不进 R），发一次只把意图写进 logged_ops 的发布（`c0` = 现行那一版的 txg，`segs0` 先交 R 那块盘按记录口径的全空段数，签发那次发布照它自己写下的记账第 11 项换，
+///    判决 m4-segment-code-r1 第三节 F3）；
 ///    被空间拒就撤排他、停（意图没签成）。
 /// 2. 有活意图：暂停过的重新排他；R 里出现隔离槽（直接查常驻隔离集，钉住与隔离同时成立也算，丁 1 交判第 4 点）、区域扫描拒了 R、扫出隔离类，
 ///    都发一次只删意图的发布丢掉它（攻方 K-c）；对 R 重扫（每批之前都重扫，丁 1 交判第 3 点）、计划一批：要搬就发——这一批搬完 R 里每一条未释放的记录时
-///    同一次发布删意图（D8（核心索引结构） 已定项 5；发布在算好的账上核 R 按记录全空，核不过照这一批不删意图再发）；R 已按记录全空就发一次只删意图的发布；
+///    同一次发布删意图（D8（核心索引结构） 已定项 5；发布在算好的账上核 R 按记录全空，核不过照这一批不删意图再发）；别的批照抄意图、在算好的账上核 R 还有未释放的记录，
+///    这一批腾空了 R（计划里的单元都是这次发布本来就重写的节点）就照同一批删意图再发（判决 m4-segment-code-r1 第三节 F2），一个都没搬的不记成一批；
+///    R 已按记录全空就发一次只删意图的发布；
 ///    一个能搬的都没有而 R 不空就发一次只删意图的发布丢掉它、这一趟停。整理的发布被空间拒就暂停（撤 R 排他、意图留在盘上，攻方 K-k）、停。
 ///
-/// 一次调用至多发两次（意图达成那一支核不过时重发一次），选段那一圈的上界是单元区的整段数。
+/// 一次调用至多算三次账、发一次（照抄那一支核出腾空了 R 换删意图再算，删意图那一支核不过换照抄再算），选段那一圈的上界是单元区的整段数。
 #[allow(
     clippy::too_many_arguments,
-    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、批大小、推到哪为止、上一条完成的意图"
+    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、批大小、推到哪为止、上一条完成的意图、一批腾空了 R 之后删意图那一支的核怎么判"
 )]
 fn advance_compaction_by_one_publish<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
@@ -2203,6 +2300,7 @@
     batch_units: CompactionBatchUnits,
     reach: CompactionRunReach,
     last_completed_intent: &mut Option<CompactionIntent>,
+    region_check_of_the_intent_deletion_after_a_batch_emptying_the_region: RegionCheckOfTheIntentDeletion,
 ) -> Result<CompactionStep, RelocationBatchRefused> {
     compaction.take_the_quarantines_noted_by_the_publishes(allocator.compaction_mut());
     let version = current
@@ -2435,9 +2533,11 @@
                     CompactionPublishKind::BatchCompletingTheIntent,
                 )
             } else {
+                // 计划里的单元都是这次发布本来就重写的节点时，这一批搬不了也腾空 R：照抄意图那一支在算好的账上核 R，
+                // 全空就照同一批删意图再发（判决 m4-segment-code-r1 第三节 F2）。
                 (
                     planned,
-                    CompactionIntentStep::CarryTheIntent,
+                    CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion,
                     CompactionPublishKind::Batch,
                 )
             }
@@ -2487,6 +2587,42 @@
                 Err(refused) => return pause_or_hand_up(allocator, refused),
             }
         }
+        // 这一批在算好的账上腾空了 R（计划里的单元都是这次发布本来就重写的节点）：同一批删意图再发（判决 m4-segment-code-r1 第三节 F2）。
+        // 删意图那一支再核 R 按记录全空：两次只差 logged_ops 那一条，取落点不读它，核不过只在只供测试的开关关掉排他时走得到，照这一批不删意图再发
+        // （用例经只供测试的 `RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests` 强制进入）。
+        Err(RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried {
+            ..
+        }) if kind == CompactionPublishKind::Batch => {
+            match publish_the_compaction_step_once_with_the_region_check_of_the_intent_deletion(
+                parameters,
+                devices,
+                allocator,
+                &version,
+                &planned,
+                CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
+                region_check_of_the_intent_deletion_after_a_batch_emptying_the_region,
+                instance,
+            ) {
+                Ok(published) => (published, CompactionPublishKind::BatchCompletingTheIntent),
+                Err(
+                    RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
+                        ..
+                    },
+                ) => match publish_the_compaction_step_once(
+                    parameters,
+                    devices,
+                    allocator,
+                    &version,
+                    &planned,
+                    CompactionIntentStep::CarryTheIntent,
+                    instance,
+                ) {
+                    Ok(published) => (published, CompactionPublishKind::Batch),
+                    Err(refused) => return pause_or_hand_up(allocator, refused),
+                },
+                Err(refused) => return pause_or_hand_up(allocator, refused),
+            }
+        }
         Err(refused) => return pause_or_hand_up(allocator, refused),
     };
     match kind {
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -8108,7 +8108,8 @@
 }
 
 /// 回退那次发布接在后面的「上一版」：用户可见的那一半（数据单元与指针、extent 树、inode 树的叶容器与根、第一个文件那条 inode 记录）
-/// 取回退目标那一版的，别的（分配记录树与它的记录、记账树与记账行、中央映射树、树表、实例表各片）取现行那一版的；
+/// 取回退目标那一版的，别的（分配记录树与它的记录、记账树与记账行、中央映射树、树表、实例表各片）取现行那一版的——中央映射树只是节点取现行那一版的，
+/// 用户可见单元的条目由发布路径按目标那一版的位置装、位置不同的那几把 key 所在的路重写（[`mapping_keys_whose_target_locations_differ_from_the_current_mapping`]）；
 /// 根记录、末条记录、事务号取现行那一版的（这次发布接着它发，反向链、计数器、txg 都接着它）。
 /// 发布路径照这一版照抄用户可见单元（inode 树根指针从树表条目取，所以树表条目里 inode 树与 extent 树那两条换成目标那一版的）、
 /// 按分配器现算固定点单元、换下现行那一版被重写的固定点单元——正是「新根的用户可见树取 R_old 的，树表单元、分配记录树、中央映射树与
@@ -8184,6 +8185,9 @@
         extent_tree: target_version.extent_tree.clone(),
         accounting_entries: current.accounting_entries.clone(),
         accounting_tree: current.accounting_tree.clone(),
+        // 映射树的节点是现行那一版的（这次发布照它们 COW、换下的照它们释放）；用户可见单元的条目取目标那一版的位置：发布路径按这一版的
+        // 指针装条目（目标那一版重建时已按它自己的映射覆盖过提示），装着位置不同的那几把 key 的叶连同到根的路重写
+        // （[`mapping_keys_whose_target_locations_differ_from_the_current_mapping`]，判决 m4-segment-code-r1 第三节 F1）。
         central_mapping_tree: current.central_mapping_tree.clone(),
         // 两版都不带快照（回退之前判过，判决 S8）：deadlist 与留存的映射条目照现行那一版（删完全部快照之后是空的）。
         deadlist_entries: current.deadlist_entries.clone(),
@@ -8220,6 +8224,30 @@
     }
 }
 
+/// 回退那次发布要把中央映射里的条目换回目标那一版位置的那几把映射 key（判决 `research/prompts/m4-segment-code-r1-main-verification.md`
+/// 第三节 F1，`research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「映射是唯一权威」）：目标那一版取来的用户可见单元
+/// （[`is_a_unit_taken_from_the_rollback_target`]）里，在现行那一版的中央映射里查到的不是目标那一版映射里那两条位置项的。
+/// 整理在两版之间搬过的单元就是这样：key 不变、现行那一版的条目指着目的落点，而回退释放的正是目的落点、复活的是源落点。
+/// 发布路径只按 key 进出定映射树重写哪些节点，key 相同的叶照抄上一版（现行那一版）的字节，条目就留在被释放的目的落点上；
+/// 交出这几把 key，装着它们的叶连同到根的路这次重写（与整理的一批同一条路，`transaction::publish_version_appending_a_lineage_record`）。
+/// 现行那一版查不到、条目切不动的也交出（重写一条路是合法的 COW）；目标那一版自己查不到的在 [`user_visible_units_through_the_mapping`] 那一判拒过。
+/// 只读两版内存里的映射叶，不读盘。
+fn mapping_keys_whose_target_locations_differ_from_the_current_mapping(
+    current: &TransactionOutput,
+    target_version: &TransactionOutput,
+) -> Vec<Vec<u8>> {
+    target_version
+        .mapped_units
+        .iter()
+        .filter(|(identity, _)| is_a_unit_taken_from_the_rollback_target(*identity))
+        .filter(|(_, key)| {
+            central_mapping_locations_in_the_version(current, key)
+                != central_mapping_locations_in_the_version(target_version, key)
+        })
+        .map(|(_, key)| key.clone())
+        .collect()
+}
+
 /// 管理员回退：挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14「管理员回退是挂着时的一次向前发布」）。
 /// 管理员从回退候选集里选 R_old（`target`）；文件系统在现行那一版（`current`，cur）后面发一次普通发布（D16（发布语义） 已定项 7 的
 /// 持久顺序）：新根的 inode 树、extent 树取 R_old 的；树表单元、分配记录树、中央映射树与记账统计量按这次发布照调整之后的分配器现算；
@@ -8438,6 +8466,11 @@
         },
     );
     let previous = version_to_publish_the_rollback_after(current, &target_version);
+    let mapping_keys_the_rollback_moves_back_to_the_target_locations =
+        mapping_keys_whose_target_locations_differ_from_the_current_mapping(
+            current,
+            &target_version,
+        );
     // 动分配器：先复活、再释放（释放代 = 新根的 txg），之后交给发布路径——它在这份账上照常释放 cur 被换下的固定点单元、取落点、
     // 装记账行。发布在落盘之前被拒时整个换回这一份之前的样子。
     let allocator_before_the_rollback = allocator.clone();
@@ -8495,6 +8528,7 @@
         &previous,
         lineage_record,
         inode_number_watermark,
+        &mapping_keys_the_rollback_moves_back_to_the_target_locations,
     ) {
         Ok(published) => {
             *current = published;
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -343,6 +343,8 @@
     container_tree_capacities: ContainerTreeCapacities,
     /// 这次发布从哪个 inode 号水位起算，只供测试的开关（[`InodeNumberWatermarkSource`]）；新开的写入口按上一版的记账起步。
     inode_number_watermark_source: InodeNumberWatermarkSource,
+    /// 被杀的用户可见单元走释放判定的哪一臂，只供测试的开关（[`SnapshotLifetimeJudgement`]）；新开的写入口照判定。
+    snapshot_lifetime_judgement: SnapshotLifetimeJudgement,
 }
 
 impl<'pool, Device: BlockDevice> PoolWriter<'pool, Device> {
@@ -363,6 +365,7 @@
             container_tree_capacities: ContainerTreeCapacities::FROM_THE_FORMAT,
             inode_number_watermark_source:
                 InodeNumberWatermarkSource::FromThePreviousVersionsAccounting,
+            snapshot_lifetime_judgement: SnapshotLifetimeJudgement::FollowTheJudgement,
         }
     }
 }
@@ -444,6 +447,18 @@
         self.inode_number_watermark_source
     }
 
+    /// 装上「被杀的用户可见单元走释放判定的哪一臂」这个只供测试的开关（[`SnapshotLifetimeJudgement`]，判决
+    /// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3 末句「只供测试的开关：强制立即释放 / 进 deadlist」）。产品路径一处都不调它。
+    pub fn set_snapshot_lifetime_judgement(&mut self, judgement: SnapshotLifetimeJudgement) {
+        self.snapshot_lifetime_judgement = judgement;
+    }
+
+    /// 这会儿装着的释放判定那一臂：运行时看得出走的是哪一条分支。
+    #[must_use]
+    pub fn snapshot_lifetime_judgement(&self) -> SnapshotLifetimeJudgement {
+        self.snapshot_lifetime_judgement
+    }
+
     pub fn perform_commit_step(&mut self, step: CommitStep<'_>) -> Result<(), BlockDeviceError> {
         if !matches!(step, CommitStep::Barrier) {
             self.has_writes_since_barrier = true;
@@ -4807,7 +4822,7 @@
 /// 这次发布对 deadlist 与留存映射条目做的事（[`DeadlistChangesOfThisPublish`]）。`candidate_roles` 是这次换下的用户可见角色的来源
 /// （这次重写的数据单元、inode 叶容器与根，没有同名后继的那几个，extent 树按计划换下的节点）：按释放判定路径同一个口径展开
 /// （文件变短时尾部的数据单元，`roles_replaced_via_mapping`）、只留六类角色与上一版里有落点的，逐个按映射 key 的出生 txg 与活头的
-/// `previous_snapshot_txg` 判（[`snapshot_lifetime_verdict`]）。进 deadlist 的条目：头 = 活头 inode 树、纪元 = 活头此刻的 `previous_snapshot_txg`
+/// `previous_snapshot_txg` 判（[`snapshot_lifetime_verdict`]；写入口上装着只供测试的强制臂时照 `snapshot_lifetime_judgement`，[`SnapshotLifetimeJudgement`]）。进 deadlist 的条目：头 = 活头 inode 树、纪元 = 活头此刻的 `previous_snapshot_txg`
 /// （D5 已定项 12「纪元 = 块被杀那一刻头的 previous_snapshot_txg」）、映射 key 照上一版；它的映射条目照上一版那两条位置项留存。
 /// 删快照的批放掉的条目按 [`deadlist_entries_the_destruction_of_a_snapshot_releases`] 取（最后一批取全部、别的批取前 `entries_released` 条），
 /// 落点取它留存的那条映射条目。没有上一版（第一个文件版本）时什么都不做。
@@ -4822,6 +4837,7 @@
     candidate_roles: &[TransactionUnit],
     snapshot: SnapshotStepOfThePublish,
     txg: CheckpointTxg,
+    snapshot_lifetime_judgement: SnapshotLifetimeJudgement,
 ) -> Result<DeadlistChangesOfThisPublish, PublishError> {
     let Some(previous_version) = previous else {
         return Ok(DeadlistChangesOfThisPublish::default());
@@ -4839,7 +4855,9 @@
     let mut mapping_entries_kept_this_publish: Vec<(Vec<u8>, [LocationEntry; 2])> = Vec::new();
     for identity in &user_visible_roles_replaced {
         let mapping_key = mapping_key_carried_from(previous_version, *identity);
-        match snapshot_lifetime_verdict(&mapping_key, previous_snapshot_txg_of_the_live_head) {
+        match snapshot_lifetime_judgement
+            .verdict_of(&mapping_key, previous_snapshot_txg_of_the_live_head)
+        {
             SnapshotLifetimeVerdict::ReleasedIntoTheDeferQueue => {}
             SnapshotLifetimeVerdict::KeptOnTheDeadlist => {
                 let locations =
@@ -4996,6 +5014,43 @@
     KeptOnTheDeadlist,
 }
 
+/// 被杀的用户可见单元走释放判定（[`snapshot_lifetime_verdict`]）的哪一臂（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支能被测试强制进入；
+/// 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3 末句、`research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F4）。
+/// 封闭集合。产品路径照判定（写入口新开时的值），另两种只供测试、装在写入口上（[`PoolWriter::set_snapshot_lifetime_judgement`]）：
+/// 强制的那一臂对这次发布换下的每一个问判定的单元都成立，判定本身不算。只管问判定的六类角色（[`asks_the_snapshot_lifetime_judgement`]），
+/// 别的 14 类照旧直接进 defer；删快照的批放哪几条不归它管。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum SnapshotLifetimeJudgement {
+    /// 照 D5（快照 / 空间记账机制） 已定项 19 的判定（[`snapshot_lifetime_verdict`]）。
+    FollowTheJudgement,
+    /// 只供测试：一律立即释放（进 defer），被活快照引用的块也放。
+    ForcedToReleaseIntoTheDeferQueueForTests,
+    /// 只供测试：一律进 deadlist、不释放、映射条目留存，没打过快照、比快照新的块也进。
+    ForcedOntoTheDeadlistForTests,
+}
+
+impl SnapshotLifetimeJudgement {
+    /// 这一臂对一个被杀的单元给出的结论：照判定，或强制的那一种。
+    #[must_use]
+    pub fn verdict_of(
+        self,
+        mapping_key: &[u8],
+        previous_snapshot_txg_of_the_live_head: CheckpointTxg,
+    ) -> SnapshotLifetimeVerdict {
+        match self {
+            SnapshotLifetimeJudgement::FollowTheJudgement => {
+                snapshot_lifetime_verdict(mapping_key, previous_snapshot_txg_of_the_live_head)
+            }
+            SnapshotLifetimeJudgement::ForcedToReleaseIntoTheDeferQueueForTests => {
+                SnapshotLifetimeVerdict::ReleasedIntoTheDeferQueue
+            }
+            SnapshotLifetimeJudgement::ForcedOntoTheDeadlistForTests => {
+                SnapshotLifetimeVerdict::KeptOnTheDeadlist
+            }
+        }
+    }
+}
+
 /// 一个被杀的单元按它映射 key 里的出生 txg 与活头的 `previous_snapshot_txg` 判（[`SnapshotLifetimeVerdict`]）。
 #[must_use]
 pub fn snapshot_lifetime_verdict(
@@ -6277,7 +6332,7 @@
     /// 记账树或中央映射树这次之后的形状算不出来 ⇒ `MultiLevelCodeTwoTreeRefused`。
     #[allow(
         clippy::too_many_arguments,
-        reason = "参数各是一样东西：计划、上一版、树号、记账行的盘、节点容量、分配记录树、删除这一步、这次搬走的映射 key"
+        reason = "参数各是一样东西：计划、上一版、树号、记账行的盘、节点容量、分配记录树、删除这一步、这次搬走的映射 key、释放判定那一臂"
     )]
     pub fn resolve(
         &self,
@@ -6288,6 +6343,7 @@
         allocation_record_tree: AllocationRecordTreePlan,
         deletion_and_lineage: &DeletionAndLineageOfThePublish,
         mapping_keys_moved_by_this_publish: &[Vec<u8>],
+        snapshot_lifetime_judgement: SnapshotLifetimeJudgement,
     ) -> Result<ResolvedPublish, PublishError> {
         let ordinary_part = self.ordinary_allocation_part(previous, trees, deletion_and_lineage)?;
         let deletion_roles_rewritten = ordinary_part.deletion_roles_rewritten();
@@ -6359,6 +6415,7 @@
             &user_visible_roles_rewritten_or_without_a_successor,
             deletion_and_lineage.snapshot,
             self.txg,
+            snapshot_lifetime_judgement,
         )?;
         rewritten_roles.extend(
             allocation_record_tree
@@ -7416,11 +7473,17 @@
     /// 删意图时上一版没有活整理意图。
     NoLiveCompactionIntent,
     /// 意图达成那一支（[`CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords`]）在这次发布算好的账上 R 里还有未释放的记录：
-    /// 停机谓词「删意图那一刻 R 按记录全空」（判决 G4）不成立，不删。只在 R 不排他（只供测试的开关关掉排他）、这次发布的新落点落进 R 时走得到；
+    /// 停机谓词「删意图那一刻 R 按记录全空」（判决 G4）不成立，不删。只在 R 不排他（只供测试的开关关掉排他）、这次发布的新落点落进 R 时走得到，
+    /// 或由只供测试的 [`RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests`] 强制；
     /// 调用方照这一批不删意图再发一次（意图留着，下一批接着搬）。
     RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
         region: CompactionRegion,
     },
+    /// 照抄而核 R 那一支（[`CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion`]）在这次发布算好的账上 R 已按记录全空：
+    /// 这一批就是最后一批，意图要同一次发布删（判决 m4-segment-code-r1 第三节 F2）。调用方照同一批删意图再发一次。
+    RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried {
+        region: CompactionRegion,
+    },
     Publish(PublishError),
 }
 
@@ -7471,6 +7534,10 @@
 pub enum CompactionIntentStep {
     /// 只搬一批，logged_ops 照抄。
     CarryTheIntent,
+    /// 搬一批、logged_ops 照抄，这一批发布之后 R 按记录全空就不发：交回 `RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried`，调用方照同一批
+    /// 删意图再发一次（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F2，G4「目标态到达与最后一批同发布删」：
+    /// 计划里的单元都是这次发布本来就重写的节点时，这一批一个都不搬也腾空了 R）。
+    CarryTheIntentUnlessThisPublishEmptiesTheRegion,
     /// 签发：logged_ops 追加这一条意图（签发 txg = 这次发布的 txg）。上一版不许已有活整理意图（全池至多一条，判决 G1 e1）。
     Issue(CompactionIntent),
     /// 意图达成：删掉上一版那一条活意图。删之前在这次发布算好的账上核 R 按记录全空（停机谓词「删意图那一刻 R 按记录全空」，判决 G4）——
@@ -7480,17 +7547,38 @@
     Drop,
 }
 
+/// 删意图那一支（[`CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords`]）落盘之前核「R 按记录全空」怎么判。封闭集合。
+/// 产品路径恒 [`RegionCheckOfTheIntentDeletion::JudgedByTheRecords`]（[`publish_compaction_step`]）；另一种只供测试
+/// （`.claude/rules/fs-design.md`「五条硬要求」第 2 条：每条分支必须能被测试强制进入），经
+/// [`publish_compaction_step_with_the_region_check_of_the_intent_deletion`] 交进来。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RegionCheckOfTheIntentDeletion {
+    /// 在这次发布算好的账上核 R 按记录全空（停机谓词，判决 G4）。
+    JudgedByTheRecords,
+    /// 只供测试：算好账之后不看记录，照「R 里还有未释放的记录」拒（`RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted`），
+    /// 与真核不过走同一段：任何写之前交回、分配器换回进来时那一份。用例拿它强制进入会话「删意图那一支核不过、照这一批不删意图再发」那一支
+    /// （`mounted_session::MountedSession::run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region`）。
+    RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests,
+}
+
 /// 整理的一次发布（[`publish_relocation_batch`] 的一般形）：搬 `planned` 里那几个单元（可以一个都没有），同一次发布对意图做 `intent_step`
 /// （判决 G1：意图条目写进 logged_ops 树，签发那次发布写、最后一批同发布删；D8（核心索引结构） 已定项 5 三态表：签发之前崩 = 没有意图，
 /// 签发之后、最后一批之前崩 = 意图还在、挂载续做，最后一批之后 = 意图没了且目标态达成）。别的同 [`publish_relocation_batch`]。
 ///
+/// 签发那次写下的 `segs0` 照判决 G1 字段表的定义句取：这次发布写下的记账第 11 项里 R 所在盘的值（判决
+/// `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F3）。调用方交进来的那条意图里的 `segs0` 只是先算的一个数：这次发布
+/// 自己开了新聚簇段时它比记账第 11 项多一。算好账之后两者不同就照记账那个数换掉、再算一次再发（两次只差 logged_ops 条目里那 8 个字节，
+/// 落点与记录不随它变）；分配器上记着交进来那一条活意图的，换成盘上写下的那一条（R 与排不排他不变）。
+///
 /// # Errors
-/// 计划对不上上一版 ⇒ [`RelocationBatchRefused`] 那四种计划错；签发时上一版已有活意图 ⇒ `CompactionIntentAlreadyLive`；删时上一版没有活意图 ⇒
-/// `NoLiveCompactionIntent`；意图达成那一支在这次发布算好的账上 R 还有未释放的记录 ⇒ `RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted`。
-/// 这几种都在任何写之前交回（最后一种在算账之后、落盘之前：分配器换回进来时那一份），盘上逐字节不变；其余同 `publish_version`（`Publish`）。
+/// 计划对不上上一版 ⇒ [`RelocationBatchRefused`] 那四种计划错；签发时上一版已有活意图 ⇒ `CompactionIntentAlreadyLive`；删时、照抄而核 R 时上一版没有活意图 ⇒
+/// `NoLiveCompactionIntent`；意图达成那一支在这次发布算好的账上 R 还有未释放的记录 ⇒ `RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted`；
+/// 照抄而核 R 那一支在算好的账上 R 已按记录全空 ⇒ `RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried`。
+/// 这几种都在任何写之前交回（后两种在算账之后、落盘之前：分配器换回进来时那一份），盘上逐字节不变；其余同 `publish_version`（`Publish`）。
 ///
 /// # Panics
-/// `planned` 是空的而 `intent_step` 是 `CarryTheIntent`：这样的发布什么都不做，调用方不许走到这里。
+/// `planned` 是空的而 `intent_step` 是两种照抄之一：这样的发布什么都不做，调用方不许走到这里。签发那一支照记账换过 `segs0` 再算一次，
+/// 记账第 11 项与头一次不同：两次只差 logged_ops 条目的字节，取落点与记录都不读它。
 pub fn publish_compaction_step<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
@@ -7499,19 +7587,64 @@
     intent_step: CompactionIntentStep,
     instance: InstanceGeneration,
 ) -> Result<TransactionOutput, RelocationBatchRefused> {
+    publish_compaction_step_with_the_region_check_of_the_intent_deletion(
+        pool,
+        allocator,
+        previous,
+        planned,
+        intent_step,
+        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
+        instance,
+    )
+}
+
+/// 同 [`publish_compaction_step`]，删意图那一支落盘之前的核照 `region_check_of_the_intent_deletion` 判（别的 `intent_step` 不读它）。
+/// 产品路径只经 [`publish_compaction_step`] 进来（[`RegionCheckOfTheIntentDeletion::JudgedByTheRecords`]）。
+///
+/// # Errors
+/// 同 [`publish_compaction_step`]；删意图那一支装着只供测试的那一种时，算好账之后照 `RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted` 拒。
+///
+/// # Panics
+/// 同 [`publish_compaction_step`]。
+pub fn publish_compaction_step_with_the_region_check_of_the_intent_deletion<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    planned: &[PlannedRelocation],
+    intent_step: CompactionIntentStep,
+    region_check_of_the_intent_deletion: RegionCheckOfTheIntentDeletion,
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, RelocationBatchRefused> {
     assert!(
-        !(planned.is_empty() && intent_step == CompactionIntentStep::CarryTheIntent),
+        !(planned.is_empty()
+            && matches!(
+                intent_step,
+                CompactionIntentStep::CarryTheIntent
+                    | CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion
+            )),
         "整理的一次发布要么搬单元、要么签发或删意图：调用方不发什么都不做的发布"
     );
     let live_intent = live_compaction_intent_of_the_entries(&previous.logged_operations);
-    let (logged_operations, region_to_be_empty) = match (intent_step, live_intent) {
+    let (logged_operations, check) = match (intent_step, live_intent) {
         (CompactionIntentStep::CarryTheIntent, _) => (
             LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
-            None,
+            CompactionStepCheckBeforePersisting::NothingToCheck,
+        ),
+        (CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion, None) => {
+            return Err(RelocationBatchRefused::NoLiveCompactionIntent)
+        }
+        (
+            CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion,
+            Some((_issued_txg, intent)),
+        ) => (
+            LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
+            CompactionStepCheckBeforePersisting::RegionKeepsAnUnreleasedRecord(intent.region),
         ),
         (CompactionIntentStep::Issue(intent), None) => (
             LoggedOperationsStepOfThePublish::IssueTheCompactionIntent(intent),
-            None,
+            CompactionStepCheckBeforePersisting::EmptyClusterSegmentsAtIssueAsTheAccountingWritesThem(
+                intent,
+            ),
         ),
         (CompactionIntentStep::Issue(_), Some((issued_txg, _))) => {
             return Err(RelocationBatchRefused::CompactionIntentAlreadyLive { issued_txg })
@@ -7526,11 +7659,14 @@
             Some((issued_txg, intent)),
         ) => (
             LoggedOperationsStepOfThePublish::DeleteTheCompactionIntent { issued_txg },
-            Some(intent.region),
+            CompactionStepCheckBeforePersisting::RegionEmptyByTheRecords {
+                region: intent.region,
+                region_check: region_check_of_the_intent_deletion,
+            },
         ),
         (CompactionIntentStep::Drop, Some((issued_txg, _))) => (
             LoggedOperationsStepOfThePublish::DeleteTheCompactionIntent { issued_txg },
-            None,
+            CompactionStepCheckBeforePersisting::NothingToCheck,
         ),
     };
     let mut relocations = Vec::with_capacity(planned.len());
@@ -7541,7 +7677,121 @@
             planned_relocation,
         )?);
     }
-    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
+    match publish_compaction_step_after_its_check(
+        pool,
+        allocator,
+        previous,
+        &relocations,
+        logged_operations,
+        check,
+        instance,
+    ) {
+        Ok(published) => Ok(published),
+        Err(CompactionStepRefusedBeforePersisting::Refused(refused)) => Err(refused),
+        Err(
+            CompactionStepRefusedBeforePersisting::EmptyClusterSegmentsAtIssueDifferFromTheAccounting {
+                handed_in,
+                written_by_the_accounting,
+            },
+        ) => {
+            let written = CompactionIntent {
+                region: handed_in.region,
+                allocation_generation_at_issue: handed_in.allocation_generation_at_issue,
+                empty_cluster_segments_at_issue: written_by_the_accounting,
+            };
+            let published = match publish_compaction_step_after_its_check(
+                pool,
+                allocator,
+                previous,
+                &relocations,
+                LoggedOperationsStepOfThePublish::IssueTheCompactionIntent(written),
+                CompactionStepCheckBeforePersisting::EmptyClusterSegmentsAtIssueAsTheAccountingWritesThem(
+                    written,
+                ),
+                instance,
+            ) {
+                Ok(published) => published,
+                Err(CompactionStepRefusedBeforePersisting::Refused(refused)) => return Err(refused),
+                Err(
+                    CompactionStepRefusedBeforePersisting::EmptyClusterSegmentsAtIssueDifferFromTheAccounting {
+                        written_by_the_accounting: written_the_second_time,
+                        ..
+                    },
+                ) => panic!(
+                    "签发那次照记账换过 segs0（{written_by_the_accounting}）再算，记账第 11 项成了 {written_the_second_time}：\
+                     两次只差 logged_ops 条目的字节，取落点与记录都不读它"
+                ),
+            };
+            allocator
+                .compaction_mut()
+                .take_the_intent_written_by_the_issue(handed_in, written);
+            Ok(published)
+        }
+    }
+}
+
+/// 整理的一次发布落盘之前在算好的账上核什么（[`publish_compaction_step`]）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum CompactionStepCheckBeforePersisting {
+    NothingToCheck,
+    /// 删意图：R 按记录全空（停机谓词，判决 G4），照 `region_check` 判（产品路径恒照记录）。
+    RegionEmptyByTheRecords {
+        region: CompactionRegion,
+        region_check: RegionCheckOfTheIntentDeletion,
+    },
+    /// 照抄意图：R 还有一条未释放的记录（判决 m4-segment-code-r1 第三节 F2：这一批腾空了 R 就同一次发布删意图）。
+    RegionKeepsAnUnreleasedRecord(CompactionRegion),
+    /// 签发：写进意图的 `segs0` 等于这次发布写下的记账第 11 项里 R 所在盘的值（判决 G1 字段表定义句，m4-segment-code-r1 第三节 F3）。
+    EmptyClusterSegmentsAtIssueAsTheAccountingWritesThem(CompactionIntent),
+}
+
+/// 整理的一次发布在落盘之前被拒（[`publish_compaction_step_after_its_check`]）：交给调用方的拒绝，或签发那一支要照记账换 `segs0` 再算一次。
+enum CompactionStepRefusedBeforePersisting {
+    Refused(RelocationBatchRefused),
+    EmptyClusterSegmentsAtIssueDifferFromTheAccounting {
+        handed_in: CompactionIntent,
+        written_by_the_accounting: u64,
+    },
+}
+
+impl From<PublishError> for CompactionStepRefusedBeforePersisting {
+    fn from(cause: PublishError) -> Self {
+        CompactionStepRefusedBeforePersisting::Refused(RelocationBatchRefused::Publish(cause))
+    }
+}
+
+/// 一版写下的记账第 11 项（全空聚簇段数）里 `device` 那一行的值。
+///
+/// # Panics
+/// 这一版没有这一行：记账树每次发布整批重写，池里每块盘一行第 11 项、代 = 这次的 txg（`accounting_entries_of_this_publish`）。
+fn empty_cluster_segments_written_by_the_version(
+    version: &TransactionOutput,
+    device: DeviceIdentity,
+) -> u64 {
+    version
+        .accounting_entries
+        .iter()
+        .find(|entry| {
+            entry.statistic == STATISTIC_EMPTY_CLUSTER_SEGMENTS
+                && entry.device == device
+                && entry.generation == version.root.checkpoint_txg
+        })
+        .expect("记账树每次发布整批重写，池里每块盘一行第 11 项、代 = 这次的 txg")
+        .value
+}
+
+/// [`publish_compaction_step`] 核过计划之后那一半：按 `logged_operations` 算这一版、落盘之前照 `check` 核、核过才落盘。
+fn publish_compaction_step_after_its_check<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    relocations: &[RelocationOfThePublish],
+    logged_operations: LoggedOperationsStepOfThePublish,
+    check: CompactionStepCheckBeforePersisting,
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, CompactionStepRefusedBeforePersisting> {
+    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)
+        .map_err(|refused| CompactionStepRefusedBeforePersisting::Refused(refused.into()))?;
     // 整理的每一次发布（搬一批、签发、删意图）都是一个事务，事务号接着这个实例（同建快照、删除的意图那一次：不写数据单元也取下一个号）。
     let transaction_of_this_publish = previous.highest_transaction_number_in_this_instance + 1;
     let deletion_and_lineage = DeletionAndLineageOfThePublish {
@@ -7573,21 +7823,67 @@
         previous.tree_identifiers,
         UnmountMarker::NotWrittenByTheUnmountSequence,
         &deletion_and_lineage,
-        &relocations,
-        |version_after_this_publish| match region_to_be_empty {
-            Some(region)
-                if !region_is_empty_by_the_records(
+        relocations,
+        &[],
+        |version_after_this_publish| {
+            match check {
+            CompactionStepCheckBeforePersisting::NothingToCheck => Ok(()),
+            CompactionStepCheckBeforePersisting::RegionEmptyByTheRecords {
+                region,
+                region_check,
+            } => {
+                let is_the_region_empty_by_the_records = match region_check {
+                    RegionCheckOfTheIntentDeletion::JudgedByTheRecords => {
+                        region_is_empty_by_the_records(
+                            &version_after_this_publish.allocation_records,
+                            region,
+                        )
+                    }
+                    RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests => false,
+                };
+                if is_the_region_empty_by_the_records {
+                    Ok(())
+                } else {
+                    Err(CompactionStepRefusedBeforePersisting::Refused(
+                        RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
+                            region,
+                        },
+                    ))
+                }
+            }
+            CompactionStepCheckBeforePersisting::RegionKeepsAnUnreleasedRecord(region) => {
+                if region_is_empty_by_the_records(
                     &version_after_this_publish.allocation_records,
                     region,
-                ) =>
-            {
-                Err(
-                    RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
-                        region,
-                    },
-                )
+                ) {
+                    Err(CompactionStepRefusedBeforePersisting::Refused(
+                        RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried {
+                            region,
+                        },
+                    ))
+                } else {
+                    Ok(())
+                }
             }
-            Some(_) | None => Ok(()),
+            CompactionStepCheckBeforePersisting::EmptyClusterSegmentsAtIssueAsTheAccountingWritesThem(
+                handed_in,
+            ) => {
+                let written_by_the_accounting = empty_cluster_segments_written_by_the_version(
+                    version_after_this_publish,
+                    handed_in.region.device(),
+                );
+                if handed_in.empty_cluster_segments_at_issue == written_by_the_accounting {
+                    Ok(())
+                } else {
+                    Err(
+                        CompactionStepRefusedBeforePersisting::EmptyClusterSegmentsAtIssueDifferFromTheAccounting {
+                            handed_in,
+                            written_by_the_accounting,
+                        },
+                    )
+                }
+            }
+        }
         },
     )
 }
@@ -8110,6 +8406,9 @@
 /// （D18（块里携带什么信息） 已定项 24 第 5 条、D23（journal 的角色与格式） 已定项 14「回退那次发布追加一条谱系记录」）。
 /// 谱系容器与谱系树根归固定点，这次发布照旧没有普通分配、不判式子（D28（挂载期承诺量） 已定项 1 接线）。
 /// 这次的 inode 号水位写 `inode_number_watermark_after_the_rollback`（回退那一判算好的 max），记账里的台阶行照抄 `previous`（它的记账行原样是 cur 的）。
+/// `mapping_keys_the_rollback_moves_back_to_the_target_locations` 是条目要换回目标那一版位置的映射 key（`previous` 的指针已是目标那一版的位置，
+/// 映射树的节点却是现行那一版的）：装着它们的叶连同到根的路这次重写，同整理的一批换位置的 key（判决
+/// `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F1）。
 ///
 /// # Errors
 /// 同 [`publish_version`]（谱系树根满时长高，`crate::container_tree`）。
@@ -8123,8 +8422,9 @@
     previous: &TransactionOutput,
     lineage_record: LineageRecord,
     inode_number_watermark_after_the_rollback: u64,
+    mapping_keys_the_rollback_moves_back_to_the_target_locations: &[Vec<u8>],
 ) -> Result<TransactionOutput, PublishError> {
-    publish_version_of_trees(
+    publish_version_of_trees_relocating_units_after_a_check(
         pool,
         allocator,
         plan,
@@ -8141,6 +8441,9 @@
             snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
             logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
         },
+        &[],
+        mapping_keys_the_rollback_moves_back_to_the_target_locations,
+        |_version_after_this_publish| Ok::<(), PublishError>(()),
     )
 }
 
@@ -8174,7 +8477,7 @@
 /// `check_before_persisting` 的拒绝；其余同 [`publish_version_of_trees_relocating_units`]（经 `From<PublishError>` 换成 `Refusal`）。
 #[allow(
     clippy::too_many_arguments,
-    reason = "参数各是一样东西：写入口、分配器、计划、上一版、树号、卸载记号、删除这一步、这次要搬的单元、落盘之前的核"
+    reason = "参数各是一样东西：写入口、分配器、计划、上一版、树号、卸载记号、删除这一步、这次要搬的单元、不搬而换了位置的映射 key、落盘之前的核"
 )]
 fn publish_version_of_trees_relocating_units_after_a_check<Device: BlockDevice, Refusal, Check>(
     pool: &mut PoolWriter<'_, Device>,
@@ -8185,6 +8488,7 @@
     unmount_marker: UnmountMarker,
     deletion_and_lineage: &DeletionAndLineageOfThePublish,
     relocations: &[RelocationOfThePublish],
+    mapping_keys_whose_entries_change_without_a_relocation: &[Vec<u8>],
     check_before_persisting: Check,
 ) -> Result<TransactionOutput, Refusal>
 where
@@ -8205,6 +8509,7 @@
         unmount_marker,
         deletion_and_lineage,
         relocations,
+        mapping_keys_whose_entries_change_without_a_relocation,
     );
     let (writes, output) = match built {
         Ok(built) => built,
@@ -8257,6 +8562,7 @@
         unmount_marker,
         deletion_and_lineage,
         relocations,
+        &[],
         |_version_after_this_publish| Ok(()),
     )
 }
@@ -8305,6 +8611,7 @@
         unmount_marker,
         deletion_and_lineage,
         &[],
+        &[],
     )
 }
 
@@ -8341,7 +8648,7 @@
 /// 读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）；这次重写要换下的角色不另搬（它的新落点本来就不在活 R 里）。
 #[allow(
     clippy::too_many_arguments,
-    reason = "参数各是一样东西：写入口、分配器、计划、上一版、树号、读不读盘核、卸载记号、删除这一步、这次要搬的单元"
+    reason = "参数各是一样东西：写入口、分配器、计划、上一版、树号、读不读盘核、卸载记号、删除这一步、这次要搬的单元、不搬而换了位置的映射 key"
 )]
 fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
     pool: &PoolWriter<'_, Device>,
@@ -8353,6 +8660,7 @@
     unmount_marker: UnmountMarker,
     deletion_and_lineage: &DeletionAndLineageOfThePublish,
     relocations: &[RelocationOfThePublish],
+    mapping_keys_whose_entries_change_without_a_relocation: &[Vec<u8>],
 ) -> Result<(PublishWrites, TransactionOutput), PublishError> {
     // 先算这次之后每棵树是什么样、这次重写哪些角色、换下哪些：只读（固定点在分配器的拷贝上走），条款没写的那几格与多层码 2 树的拒绝、
     // 释放判定的错、取不到落点都在这里交回，盘上逐字节不变。记账行按分配器里的那几块盘展开（装记账行时读的是同一批）。
@@ -8400,6 +8708,8 @@
         allocator,
         deletion_and_lineage,
         relocations,
+        mapping_keys_whose_entries_change_without_a_relocation,
+        pool.snapshot_lifetime_judgement(),
     )?;
     // 这次发布切出来的记录条数（代码审阅第 26 条）：角色定下来就数得出，在准入、读盘核与动分配器之前判。
     refuse_a_publish_of_more_journal_records_than_the_limit(
@@ -8799,7 +9109,7 @@
 /// 真发时同一个落点同样取不到；轮数用完 ⇒ `AllocationRecordTreeRewriteSetDidNotSettle`。都在任何落盘动作之前交回。
 #[allow(
     clippy::too_many_arguments,
-    reason = "参数各是一样东西：计划、上一版、树号、记账行的盘、节点容量、分配器、删除这一步、这次要搬的单元"
+    reason = "参数各是一样东西：计划、上一版、树号、记账行的盘、节点容量、分配器、删除这一步、这次要搬的单元、不搬而换了位置的映射 key、释放判定那一臂"
 )]
 fn settle_the_allocation_record_tree(
     plan: &PublishPlan<'_>,
@@ -8810,11 +9120,19 @@
     allocator: &PoolAllocator,
     deletion_and_lineage: &DeletionAndLineageOfThePublish,
     relocations: &[RelocationOfThePublish],
+    mapping_keys_whose_entries_change_without_a_relocation: &[Vec<u8>],
+    snapshot_lifetime_judgement: SnapshotLifetimeJudgement,
 ) -> Result<SettledPublish, PublishError> {
     let geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
+    // 条目换了位置、key 不变的：整理这一批搬走的，与管理员回退把条目换回目标那一版位置的（判决 m4-segment-code-r1 第三节 F1）。
     let mapping_keys_moved_by_this_publish: Vec<Vec<u8>> = relocations
         .iter()
         .map(|relocation| relocation.mapping_key.clone())
+        .chain(
+            mapping_keys_whose_entries_change_without_a_relocation
+                .iter()
+                .cloned(),
+        )
         .collect();
     let (previous_nodes, previous_records): (
         BTreeSet<AllocationRecordTreeNode>,
@@ -8847,6 +9165,7 @@
             allocation_plan,
             deletion_and_lineage,
             &mapping_keys_moved_by_this_publish,
+            snapshot_lifetime_judgement,
         )?;
         let (previous_roles_replaced, mut release) =
             placements_released_by_this_publish(plan, previous, &resolved, allocator)?;
--- a/crates/singlefs-reference-filesystem/src/lib.rs
+++ b/crates/singlefs-reference-filesystem/src/lib.rs
@@ -108,16 +108,23 @@
         requested: VersionNumber,
         versions_so_far: usize,
     },
-    /// 还没有文件时建快照：实现在没有活文件的一版上拒建快照（会话拒成 `NoFileVersionToChange`），条款没写它在 POSIX 面上是什么。
-    CreatingASnapshotWithoutAFileIsNotInTheFirstVersion,
-    /// 建快照用了一个已经活着的快照名。
+    /// 建快照用了一个已经活着的快照名（允许的每一个快照集合里它都活着）。
     SnapshotNameAlreadyTaken { name: SnapshotName },
-    /// 删一个不活着的快照。
+    /// 删一个不活着的快照（允许的每一个快照集合里它都不在）。
     NoSnapshotWithThisName { name: SnapshotName },
-    /// 有活快照时回退：第一版有快照就拒回退（用户 2026-10-13 定，判决第三节 S8 与第七节岔路 3）。
+    /// 有活快照时回退：第一版有快照就拒回退（用户 2026-10-13 定，判决第三节 S8 与第七节岔路 3）；允许的每一个快照集合都不空。
     RollBackWithSnapshotsIsNotInTheFirstVersion,
 }
 
+/// 读快照读出的内容不是允许的任何一个快照集合里那个快照的内容：判红；返回它时模型一个字段都没动。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct SnapshotReadOutsideTheAllowedReads {
+    pub name: SnapshotName,
+    pub read_out: VisibleState,
+    /// 允许的每一个快照集合里这个快照读出什么（`None` = 那个集合里没有它），去重。
+    pub allowed_reads: Vec<Option<VisibleState>>,
+}
+
 /// 允许集合里的一个状态与它的下标。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct AllowedState {
@@ -194,13 +201,83 @@
     }
 }
 
+/// 一个操作怎么改快照集合。不按「之后的快照集合」记：恢复之后允许的快照集合可以不止一个（判决
+/// `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6），同一个操作在每一个上各施加一次。
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum SnapshotChangeOfTheOperation {
+    Unchanged,
+    Created {
+        name: SnapshotName,
+        captured: VisibleState,
+    },
+    Destroyed {
+        name: SnapshotName,
+    },
+}
+
+impl SnapshotChangeOfTheOperation {
+    fn apply_to(&self, snapshots: &mut Snapshots) {
+        match self {
+            SnapshotChangeOfTheOperation::Unchanged => {}
+            SnapshotChangeOfTheOperation::Created { name, captured } => {
+                snapshots.insert(*name, captured.clone());
+            }
+            SnapshotChangeOfTheOperation::Destroyed { name } => {
+                snapshots.remove(name);
+            }
+        }
+    }
+}
+
+/// 实现做了这个操作（进了序列）说明它那一刻的快照集合满足什么：实现在任何写之前拒掉不满足的，拒掉的不进序列。
+/// 允许的快照集合里不满足的那几个由此排除（同一段历史里实现只有一个快照集合）。不看快照集合的操作没有这一样。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum SnapshotPreconditionOfTheOperation {
+    NameFree(SnapshotName),
+    SnapshotPresent(SnapshotName),
+    NoSnapshotAtAll,
+}
+
+impl SnapshotPreconditionOfTheOperation {
+    fn holds_in(self, snapshots: &Snapshots) -> bool {
+        match self {
+            SnapshotPreconditionOfTheOperation::NameFree(name) => !snapshots.contains_key(&name),
+            SnapshotPreconditionOfTheOperation::SnapshotPresent(name) => {
+                snapshots.contains_key(&name)
+            }
+            SnapshotPreconditionOfTheOperation::NoSnapshotAtAll => snapshots.is_empty(),
+        }
+    }
+
+    /// 允许的快照集合一个都不满足时不收这个操作的那一条。
+    fn refusal_when_no_allowed_snapshot_set_meets_it(self) -> OperationNotAccepted {
+        match self {
+            SnapshotPreconditionOfTheOperation::NameFree(name) => {
+                OperationNotAccepted::SnapshotNameAlreadyTaken { name }
+            }
+            SnapshotPreconditionOfTheOperation::SnapshotPresent(name) => {
+                OperationNotAccepted::NoSnapshotWithThisName { name }
+            }
+            SnapshotPreconditionOfTheOperation::NoSnapshotAtAll => {
+                OperationNotAccepted::RollBackWithSnapshotsIsNotInTheFirstVersion
+            }
+        }
+    }
+}
+
 #[derive(Clone, Debug, PartialEq, Eq)]
 struct EnteredOperation {
     kind: EnteredKind,
     outcome: EnteredOutcome,
     state_after: VisibleState,
-    /// 这个操作施加之后活着的快照（不建、不删快照的操作照抄前一个的）。
-    snapshots_after: Snapshots,
+    snapshot_change: SnapshotChangeOfTheOperation,
+}
+
+/// 往允许的快照集合里加一个，已经有相同的就不加（集合按内容去重）。
+fn push_distinct_snapshot_set(snapshot_sets: &mut Vec<Snapshots>, snapshots: Snapshots) {
+    if !snapshot_sets.contains(&snapshots) {
+        snapshot_sets.push(snapshots);
+    }
 }
 
 /// 序列里已确认的发布（返回成功的发布调用、fsync 与回退，原样重发成功的也算）各自发布之后的内容，按历史次序。
@@ -290,8 +367,10 @@
     operations_since_the_last_recovery: Vec<EnteredOperation>,
     /// 上一次恢复为止的版本，按历史次序：起点、之前各轮里已确认的发布之后的内容、各次恢复读出的那一份；第 m 版在位置 m − 1。
     versions_up_to_the_last_recovery: Vec<VisibleState>,
-    /// S_0 那一刻活着的快照：上一次恢复以它重新起头的那一份（还没崩溃过时没有快照）。
-    snapshots_at_the_last_recovery: Snapshots,
+    /// S_0 那一刻允许活着的快照集合，去重、至少一个：还没崩溃过时只有「没有快照」；恢复之后是允许集合里与读出的内容相同的每一个状态那一刻的
+    /// 快照集合（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6），再排除之后实现做了的操作说不通的那几个。
+    /// 序列里每个操作在每一个上各施加一次（[`SnapshotChangeOfTheOperation`]），第 k 个下标与 S_0 那一刻的第 k 个一一对应。
+    allowed_snapshot_sets_at_the_last_recovery: Vec<Snapshots>,
 }
 
 impl ReferenceFileSystem {
@@ -301,22 +380,77 @@
             operations_since_the_last_recovery: Vec::new(),
             // 起点是第 1 版：mkfs 写的根与 mkfs 之后可写挂载写的根映到它（r6 用户定案 H1-收严「m 的编号覆盖每一个写根的入口」）
             versions_up_to_the_last_recovery: vec![VisibleState::NoFile],
-            snapshots_at_the_last_recovery: Snapshots::new(),
+            allowed_snapshot_sets_at_the_last_recovery: vec![Snapshots::new()],
         }
     }
 
-    /// 序列里的操作全都做成时活着的快照。
-    pub fn snapshots(&self) -> &Snapshots {
-        self.operations_since_the_last_recovery
-            .last()
-            .map_or(&self.snapshots_at_the_last_recovery, |operation| {
-                &operation.snapshots_after
+    /// S_j 那一刻的快照集合，与 S_0 那一刻允许的快照集合逐个对应（不去重）：每一个施加序列里前 j 个操作。
+    fn snapshot_set_of_each_allowed_start_at(&self, index: StateIndex) -> Vec<Snapshots> {
+        let operations = &self.operations_since_the_last_recovery[..index.0];
+        self.allowed_snapshot_sets_at_the_last_recovery
+            .iter()
+            .map(|snapshots_at_the_last_recovery| {
+                let mut snapshots = snapshots_at_the_last_recovery.clone();
+                for operation in operations {
+                    operation.snapshot_change.apply_to(&mut snapshots);
+                }
+                snapshots
             })
+            .collect()
+    }
+
+    /// 序列里的操作全都做成时允许活着的快照集合，去重：恢复之后读出的文件定不了落在哪一个发布末尾时不止一个（判决第三节 F6）。
+    pub fn allowed_snapshot_sets(&self) -> Vec<Snapshots> {
+        let mut allowed = Vec::new();
+        for snapshots in self.snapshot_set_of_each_allowed_start_at(StateIndex(
+            self.operations_since_the_last_recovery.len(),
+        )) {
+            push_distinct_snapshot_set(&mut allowed, snapshots);
+        }
+        allowed
     }
 
-    /// 读快照 `name`：它活着时交回建它那一刻的状态，不活着交回 `None`（序列里的操作全都做成时）。
-    pub fn read_snapshot(&self, name: SnapshotName) -> Option<&VisibleState> {
-        self.snapshots().get(&name)
+    /// 读快照 `name` 允许读出什么（序列里的操作全都做成时）：允许的每一个快照集合里它活着时是建它那一刻的状态，不活着是 `None`；去重。
+    pub fn allowed_reads_of_the_snapshot(&self, name: SnapshotName) -> Vec<Option<VisibleState>> {
+        let mut allowed_reads = Vec::new();
+        for snapshots in self.allowed_snapshot_sets() {
+            let read = snapshots.get(&name).cloned();
+            if !allowed_reads.contains(&read) {
+                allowed_reads.push(read);
+            }
+        }
+        allowed_reads
+    }
+
+    /// 实现读快照 `name` 读出 `read_out`（序列里的操作全都做成时）：它要是允许的某一个快照集合里那个快照的内容；
+    /// 是的话只留下读得出它的那几个快照集合（实现只有一个快照集合，读出的内容说明是哪几个）。
+    ///
+    /// # Errors
+    /// 允许的快照集合里没有一个读得出它：判红，模型不动。
+    pub fn judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
+        &mut self,
+        name: SnapshotName,
+        read_out: &VisibleState,
+    ) -> Result<(), SnapshotReadOutsideTheAllowedReads> {
+        let current = self.snapshot_set_of_each_allowed_start_at(StateIndex(
+            self.operations_since_the_last_recovery.len(),
+        ));
+        let kept: Vec<Snapshots> = self
+            .allowed_snapshot_sets_at_the_last_recovery
+            .iter()
+            .zip(&current)
+            .filter(|(_, snapshots)| snapshots.get(&name) == Some(read_out))
+            .map(|(snapshots_at_the_last_recovery, _)| snapshots_at_the_last_recovery.clone())
+            .collect();
+        if kept.is_empty() {
+            return Err(SnapshotReadOutsideTheAllowedReads {
+                name,
+                read_out: read_out.clone(),
+                allowed_reads: self.allowed_reads_of_the_snapshot(name),
+            });
+        }
+        self.allowed_snapshot_sets_at_the_last_recovery = kept;
+        Ok(())
     }
 
     /// 到这一刻为止的版本，按历史次序（第 m 版在位置 m − 1）。
@@ -391,14 +525,52 @@
             // 写了之后报错＝两可：D16（发布语义） .claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
             OperationOutcome::FailedAfterWriting => EnteredOutcome::FailedAfterWriting,
         };
+        // 实现做了这个操作说明它那一刻的快照集合满足什么（[`SnapshotPreconditionOfTheOperation`]）：允许的快照集合里不满足的排除，
+        // 一个都不满足就不收。回退先判快照、再判目标版本（两样都不收时报快照那一条）。
+        let snapshot_precondition = match &operation {
+            Operation::CreateTheFirstFile(_)
+            | Operation::OverwriteTheFile(_)
+            | Operation::ExplicitPublish
+            | Operation::Fsync => None,
+            // 第一版有快照就拒回退（判决 S8）：实现在任何写之前拒，调用方交的是「写之前拒」，走不到这里；允许的快照集合都不空时走到了就是输入不合法。
+            Operation::RollBackToVersion(_) => {
+                Some(SnapshotPreconditionOfTheOperation::NoSnapshotAtAll)
+            }
+            Operation::CreateSnapshot(name) => {
+                Some(SnapshotPreconditionOfTheOperation::NameFree(*name))
+            }
+            Operation::DestroySnapshot(name) => {
+                Some(SnapshotPreconditionOfTheOperation::SnapshotPresent(*name))
+            }
+        };
+        let allowed_snapshot_sets_at_the_last_recovery = match snapshot_precondition {
+            None => self.allowed_snapshot_sets_at_the_last_recovery.clone(),
+            Some(precondition) => {
+                let current = self.snapshot_set_of_each_allowed_start_at(StateIndex(
+                    self.operations_since_the_last_recovery.len(),
+                ));
+                let kept: Vec<Snapshots> = self
+                    .allowed_snapshot_sets_at_the_last_recovery
+                    .iter()
+                    .zip(&current)
+                    .filter(|(_, snapshots)| precondition.holds_in(snapshots))
+                    .map(|(snapshots_at_the_last_recovery, _)| {
+                        snapshots_at_the_last_recovery.clone()
+                    })
+                    .collect();
+                if kept.is_empty() {
+                    return Err(precondition.refusal_when_no_allowed_snapshot_set_meets_it());
+                }
+                kept
+            }
+        };
         let state_so_far = self.state_after_every_operation_so_far();
-        let snapshots_so_far = self.snapshots();
-        let (kind, state_after, snapshots_after) = match operation {
+        let (kind, state_after, snapshot_change) = match operation {
             Operation::CreateTheFirstFile(content) => match state_so_far {
                 VisibleState::NoFile => (
                     EnteredKind::ContentChange,
                     VisibleState::OneFile(content),
-                    snapshots_so_far.clone(),
+                    SnapshotChangeOfTheOperation::Unchanged,
                 ),
                 VisibleState::OneFile(_) => return Err(OperationNotAccepted::CreatingTheFirstFileWhileAFileExistsIsNotInTheFirstVersion),
             },
@@ -407,69 +579,48 @@
                 VisibleState::OneFile(_) => (
                     EnteredKind::ContentChange,
                     VisibleState::OneFile(content),
-                    snapshots_so_far.clone(),
+                    SnapshotChangeOfTheOperation::Unchanged,
                 ),
             },
             Operation::ExplicitPublish => (
                 EnteredKind::ExplicitPublish,
                 state_so_far.clone(),
-                snapshots_so_far.clone(),
+                SnapshotChangeOfTheOperation::Unchanged,
             ),
             Operation::Fsync => (
                 EnteredKind::Fsync,
                 state_so_far.clone(),
-                snapshots_so_far.clone(),
+                SnapshotChangeOfTheOperation::Unchanged,
+            ),
+            Operation::RollBackToVersion(number) => (
+                EnteredKind::RollBack,
+                self.content_of_version(number)?,
+                SnapshotChangeOfTheOperation::Unchanged,
+            ),
+            // 文件集合为空时建快照照收，快照读出「没有文件」（用户 2026-10-16 定「放开」，判决
+            // `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F5）。
+            Operation::CreateSnapshot(name) => (
+                EnteredKind::CreateSnapshot,
+                state_so_far.clone(),
+                SnapshotChangeOfTheOperation::Created {
+                    name,
+                    captured: state_so_far.clone(),
+                },
+            ),
+            Operation::DestroySnapshot(name) => (
+                EnteredKind::DestroySnapshot,
+                state_so_far.clone(),
+                SnapshotChangeOfTheOperation::Destroyed { name },
             ),
-            Operation::RollBackToVersion(number) => {
-                // 第一版有快照就拒回退（判决 S8）：实现在任何写之前拒，调用方交的是「写之前拒」，走不到这里；走到了就是输入不合法。
-                if !snapshots_so_far.is_empty() {
-                    return Err(OperationNotAccepted::RollBackWithSnapshotsIsNotInTheFirstVersion);
-                }
-                (
-                    EnteredKind::RollBack,
-                    self.content_of_version(number)?,
-                    snapshots_so_far.clone(),
-                )
-            }
-            Operation::CreateSnapshot(name) => {
-                if snapshots_so_far.contains_key(&name) {
-                    return Err(OperationNotAccepted::SnapshotNameAlreadyTaken { name });
-                }
-                match state_so_far {
-                    VisibleState::NoFile => {
-                        return Err(
-                            OperationNotAccepted::CreatingASnapshotWithoutAFileIsNotInTheFirstVersion,
-                        )
-                    }
-                    VisibleState::OneFile(_) => {}
-                }
-                let mut snapshots_after = snapshots_so_far.clone();
-                snapshots_after.insert(name, state_so_far.clone());
-                (
-                    EnteredKind::CreateSnapshot,
-                    state_so_far.clone(),
-                    snapshots_after,
-                )
-            }
-            Operation::DestroySnapshot(name) => {
-                if !snapshots_so_far.contains_key(&name) {
-                    return Err(OperationNotAccepted::NoSnapshotWithThisName { name });
-                }
-                let mut snapshots_after = snapshots_so_far.clone();
-                snapshots_after.remove(&name);
-                (
-                    EnteredKind::DestroySnapshot,
-                    state_so_far.clone(),
-                    snapshots_after,
-                )
-            }
         };
+        self.allowed_snapshot_sets_at_the_last_recovery =
+            allowed_snapshot_sets_at_the_last_recovery;
         self.operations_since_the_last_recovery
             .push(EnteredOperation {
                 kind,
                 outcome: entered_outcome,
                 state_after,
-                snapshots_after,
+                snapshot_change,
             });
         Ok(SubmittedOperation::Entered(OperationNumber(
             self.operations_since_the_last_recovery.len(),
@@ -552,19 +703,6 @@
         Ok(())
     }
 
-    /// S_j 那一刻活着的快照（S_0 是上一次恢复以它起头的那一份）。
-    fn snapshots_at(&self, index: StateIndex) -> Snapshots {
-        match index.0.checked_sub(1) {
-            None => self.snapshots_at_the_last_recovery.clone(),
-            Some(position) => self
-                .operations_since_the_last_recovery
-                .get(position)
-                .map_or_else(Snapshots::new, |operation| {
-                    operation.snapshots_after.clone()
-                }),
-        }
-    }
-
     fn restart_from_the_recovered_state(
         &mut self,
         crash_falls_in: OperationNumber,
@@ -580,24 +718,41 @@
         // 恢复读出、以它重新起头的那一份也是一个版本：之后可写挂载写的根映到它（r6 用户定案 H1-收严）
         self.versions_up_to_the_last_recovery
             .push(recovered.clone());
-        // 恢复之后活着的快照：允许集合里与读出的内容相同、下标最大的那个状态那一刻的快照（这一版的崩溃判定只比文件，
-        // 快照集合不比——建快照与删快照都是发布末尾，读出的文件定不了它落在哪一个末尾时取最新的那个；第一版的随机历史与崩溃流
-        // 还不交快照操作，各状态的快照集合相同）。
-        let snapshots_after_the_recovery = self
+        // 恢复之后允许活着的快照集合：允许集合里与读出的内容相同的每一个状态那一刻的快照集合（判决
+        // `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6）。崩溃判定只比文件；建快照与删快照都是发布末尾，
+        // 写了之后报错或崩在里面的那一次落没落盘读出的文件定不了，两种结局都收，不取下标最大的那一个。
+        // 读出的内容一个都对不上（抛弃最新根的那次恢复不判读出的内容）时照取 S_0 那一刻的。
+        let indices_whose_state_is_the_recovered_one: Vec<StateIndex> = self
             .allowed_set_for_a_crash_in(crash_falls_in)
-            .ok()
-            .and_then(|allowed| {
+            .map(|allowed| {
                 allowed
                     .allowed
                     .iter()
-                    .rev()
-                    .find(|allowed_state| allowed_state.state == recovered)
-                    .map(|allowed_state| self.snapshots_at(allowed_state.index))
+                    .filter(|allowed_state| allowed_state.state == recovered)
+                    .map(|allowed_state| allowed_state.index)
+                    .collect()
             })
-            .unwrap_or_else(|| self.snapshots_at(StateIndex(0)));
+            .expect(
+                "两个调用方都先核过 hi 判得了（`allowed_set_for_a_crash_in` 返回 Ok）才走到这里",
+            );
+        let indices_the_snapshot_sets_are_taken_at =
+            if indices_whose_state_is_the_recovered_one.is_empty() {
+                vec![StateIndex(0)]
+            } else {
+                indices_whose_state_is_the_recovered_one
+            };
+        let mut allowed_snapshot_sets_after_the_recovery = Vec::new();
+        for index in indices_the_snapshot_sets_are_taken_at {
+            for snapshots in self.snapshot_set_of_each_allowed_start_at(index) {
+                push_distinct_snapshot_set(
+                    &mut allowed_snapshot_sets_after_the_recovery,
+                    snapshots,
+                );
+            }
+        }
         // 每次恢复之后以读出的内容为起点接着走（r5 判决第二节 U2 ②）
         self.state_at_the_last_recovery = recovered;
-        self.snapshots_at_the_last_recovery = snapshots_after_the_recovery;
+        self.allowed_snapshot_sets_at_the_last_recovery = allowed_snapshot_sets_after_the_recovery;
         self.operations_since_the_last_recovery.clear();
     }
 }
@@ -650,17 +805,21 @@
         succeed(&mut model, Operation::CreateSnapshot(SnapshotName(65_549)));
         succeed(&mut model, Operation::OverwriteTheFile(content("B")));
         assert_eq!(
-            model.read_snapshot(SnapshotName(65_549)),
-            Some(&file_with("A")),
+            model.allowed_reads_of_the_snapshot(SnapshotName(65_549)),
+            vec![Some(file_with("A"))],
             "快照读出建它那一刻的 A"
         );
         succeed(&mut model, Operation::DestroySnapshot(SnapshotName(65_549)));
         assert_eq!(
-            model.read_snapshot(SnapshotName(65_549)),
-            None,
+            model.allowed_reads_of_the_snapshot(SnapshotName(65_549)),
+            vec![None],
             "删掉之后读不到"
         );
-        assert!(model.snapshots().is_empty(), "快照集合空了");
+        assert_eq!(
+            model.allowed_snapshot_sets(),
+            vec![Snapshots::new()],
+            "快照集合空了"
+        );
         assert_eq!(
             model.state_after_every_operation_so_far(),
             &file_with("B"),
@@ -685,25 +844,154 @@
             "建快照返回成功：下界是它的末尾"
         );
         assert_eq!(
-            model.read_snapshot(SnapshotName(7)),
-            Some(&file_with("A")),
+            model.allowed_reads_of_the_snapshot(SnapshotName(7)),
+            vec![Some(file_with("A"))],
             "恢复之后快照接着活着"
         );
     }
 
-    /// 不收的四种：没有文件时建快照、快照名已经活着、删一个不活着的快照、有活快照时回退；返回时模型不动。
+    /// 文件集合为空时建快照照收（用户 2026-10-16 定「放开」，判决 `research/prompts/m4-segment-code-r1-main-verification.md`
+    /// 第三节 F5）：空池上建快照，之后写第一个文件，读快照读出「没有文件」，活头读出写的那份。
     #[test]
-    fn snapshot_operations_the_first_version_does_not_take_are_not_accepted_and_leave_the_model_unchanged(
+    fn a_snapshot_created_while_there_is_no_file_reads_no_file_after_the_first_file_is_written() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(3)));
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        assert_eq!(
+            model.allowed_reads_of_the_snapshot(SnapshotName(3)),
+            vec![Some(VisibleState::NoFile)],
+            "快照读出建它那一刻的「没有文件」"
+        );
+        assert_eq!(
+            model.state_after_every_operation_so_far(),
+            &file_with("A"),
+            "活头读出写的 A"
+        );
+    }
+
+    /// 建快照写了之后报错（做没做成两可），之后崩溃、恢复读出 A：建快照之前与之后两个发布末尾都是 A，读出的文件定不了那次发布落没落盘，
+    /// 快照有与没有两种结局都收（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6）——之后删这个快照照收（落了盘），
+    /// 回退也照收（没落盘：第一版有快照就拒回退，只有快照集合可以是空的才收）。
+    #[test]
+    fn a_crash_after_a_snapshot_creation_that_failed_after_writing_allows_the_snapshot_to_be_there_or_not(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::Fsync);
+        enter(
+            &mut model,
+            Operation::CreateSnapshot(SnapshotName(7)),
+            OperationOutcome::FailedAfterWriting,
+        );
+        let crash = model.crash_after_every_operation_returned();
+        model
+            .crash_and_recover(crash, file_with("A"))
+            .expect("A 在允许集合里");
+        assert_eq!(
+            model.allowed_snapshot_sets(),
+            vec![
+                Snapshots::new(),
+                Snapshots::from([(SnapshotName(7), file_with("A"))])
+            ],
+            "快照没有与有两种快照集合都允许"
+        );
+        let mut the_creation_landed = model.clone();
+        succeed(
+            &mut the_creation_landed,
+            Operation::DestroySnapshot(SnapshotName(7)),
+        );
+        let mut the_creation_did_not_land = model.clone();
+        let rollback_target = the_creation_did_not_land.newest_version();
+        succeed(
+            &mut the_creation_did_not_land,
+            Operation::RollBackToVersion(rollback_target),
+        );
+        assert_eq!(
+            the_creation_did_not_land.allowed_snapshot_sets(),
+            vec![Snapshots::new()],
+            "回退做成说明快照没落盘：只剩空的那个快照集合"
+        );
+    }
+
+    /// 删快照写了之后报错（意图落了、批没落完，两可），之后崩溃、恢复读出 A：快照还在与已删两种结局都收（判决第三节 F6）——
+    /// 读快照读出建它那一刻的 A 照收、读出别的内容判红且模型不动；读出 A 之后只剩快照还在的那个集合，回退照拒；
+    /// 没读快照时回退照收（快照已删）。
+    #[test]
+    fn a_crash_after_a_snapshot_destruction_that_failed_after_writing_allows_the_snapshot_to_be_there_or_not(
     ) {
         let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
+        enter(
+            &mut model,
+            Operation::DestroySnapshot(SnapshotName(7)),
+            OperationOutcome::FailedAfterWriting,
+        );
+        let crash = model.crash_after_every_operation_returned();
+        model
+            .crash_and_recover(crash, file_with("A"))
+            .expect("A 在允许集合里");
+        assert_eq!(
+            model.allowed_snapshot_sets(),
+            vec![
+                Snapshots::from([(SnapshotName(7), file_with("A"))]),
+                Snapshots::new()
+            ],
+            "快照还在与已删两种快照集合都允许"
+        );
+        let mut the_destruction_did_not_land = model.clone();
+        let before_the_wrong_read = the_destruction_did_not_land.clone();
         assert_eq!(
-            model.submit(
-                Operation::CreateSnapshot(SnapshotName(1)),
+            the_destruction_did_not_land
+                .judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
+                    SnapshotName(7),
+                    &file_with("B")
+                ),
+            Err(SnapshotReadOutsideTheAllowedReads {
+                name: SnapshotName(7),
+                read_out: file_with("B"),
+                allowed_reads: vec![Some(file_with("A")), None],
+            }),
+            "读出建它那一刻之外的内容判红"
+        );
+        assert_eq!(
+            the_destruction_did_not_land, before_the_wrong_read,
+            "判红的读不动模型"
+        );
+        the_destruction_did_not_land
+            .judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
+                SnapshotName(7),
+                &file_with("A"),
+            )
+            .expect("快照还在：读出建它那一刻的 A");
+        assert_eq!(
+            the_destruction_did_not_land.allowed_snapshot_sets(),
+            vec![Snapshots::from([(SnapshotName(7), file_with("A"))])],
+            "读出 A 之后只剩快照还在的那个集合"
+        );
+        let rollback_target_while_the_snapshot_is_kept =
+            the_destruction_did_not_land.newest_version();
+        assert_eq!(
+            the_destruction_did_not_land.submit(
+                Operation::RollBackToVersion(rollback_target_while_the_snapshot_is_kept),
                 OperationOutcome::Succeeded
             ),
-            Err(OperationNotAccepted::CreatingASnapshotWithoutAFileIsNotInTheFirstVersion),
-            "没有文件时建快照"
+            Err(OperationNotAccepted::RollBackWithSnapshotsIsNotInTheFirstVersion),
+            "快照还在：回退照拒"
+        );
+        let mut the_destruction_landed = model.clone();
+        let rollback_target_after_the_destruction_landed = the_destruction_landed.newest_version();
+        succeed(
+            &mut the_destruction_landed,
+            Operation::RollBackToVersion(rollback_target_after_the_destruction_landed),
         );
+    }
+
+    /// 不收的三种：快照名已经活着、删一个不活着的快照、有活快照时回退；返回时模型不动。
+    #[test]
+    fn snapshot_operations_the_first_version_does_not_take_are_not_accepted_and_leave_the_model_unchanged(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
         succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
         succeed(&mut model, Operation::CreateSnapshot(SnapshotName(1)));
         let before = model.clone();
````
