# 附录二：增补 3 第 2 件（理想模型与对拍）代码改动（`git diff 5efaf79 -- crates/singlefs-harness/src/history.rs crates/singlefs-harness/src/lib.rs crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs crates/mutations.tsv` 原样，加三个新文件全文；生成于 2026-09-19 06:21 UTC / 15:21 JST）

## 一、diff（相对基准提交 `5efaf79` 之后的工作区，四个已跟踪文件）

```diff
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index 797dfd2..13541b8 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -143,3 +143,13 @@ Z1-a：实例表准入按每次挂载只写一行算	crates/singlefs-core/src/mo
 增补 3 第 1 件（代码三方第二轮第 1 条，形态 a）：只在挂载写出的写行与暖机里复用改写已回收记录时不改分配代	crates/singlefs-core/src/allocator.rs	            existing.generation = generation;\n            existing.is_released = false;	            if !std::backtrace::Backtrace::force_capture().to_string().contains("mount::establish_instance") {\n                existing.generation = generation;\n            }\n            existing.is_released = false;	-p singlefs-harness --test second_transaction_supplement_three_random_history -- row_and_warm_up_publishes	row_and_warm_up_publishes_that_reuse_reclaimed_records_have_their_allocation_generations_checked
 增补 3 第 1 件（代码三方第二轮第 1 条，形态 b）：挂载写出的写行与暖机里每次分配的分配代记成 txg − 1	crates/singlefs-core/src/allocator.rs	    fn record(&mut self, placement: Placement, generation: CheckpointTxg) {\n	    fn record(&mut self, placement: Placement, generation: CheckpointTxg) {\n        let generation = if std::backtrace::Backtrace::force_capture().to_string().contains("mount::establish_instance") { CheckpointTxg(generation.0 - 1) } else { generation };\n	-p singlefs-harness --test second_transaction_supplement_three_random_history -- row_and_warm_up_publishes	row_and_warm_up_publishes_that_reuse_reclaimed_records_have_their_allocation_generations_checked
 增补 3 第 1 件（代码三方第二轮第 2 条，几何补取样点）：随机历史快档也判出「复用时新记录罩住的已回收记录不删」（第 121、130 行同一处）	crates/singlefs-core/src/allocator.rs	            records.retain(|record| !(record.device == device && record.slot == record_slot));	            // 变异：罩住的已回收记录不删	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
+增补 3 第 2 件（理想模型，r1 攻方变异 B1）：内容正好装满载荷容量也报装不下	crates/singlefs-core/src/transaction.rs	if file.content.len() > data_unit_capacity {	if file.content.len() >= data_unit_capacity {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
+增补 3 第 2 件（理想模型，r1 攻方变异 B2）：回退到 txg = F_生效 的根也拒	crates/singlefs-core/src/mount.rs	if target.checkpoint_txg < effective_floor {	if target.checkpoint_txg <= effective_floor {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_the_root_at_the_effective_floor	rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version
+增补 3 第 2 件（理想模型，r1 攻方变异 B3）：抬 F 上限不取第 4 新的非空根（只取每盘最新有效根）	crates/singlefs-core/src/mount.rs	Some(newest_on_every_device.min(fourth_newest))	Some(newest_on_every_device.max(fourth_newest))	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
+增补 3 第 2 件（理想模型，r1 攻方变异 B5）：非空判定恒真（空发布也算非空）	crates/singlefs-core/src/mount.rs	root_pointers.inode_tree != previous_valid_root_pointers.inode_tree\n        || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree	true || root_pointers.inode_tree != previous_valid_root_pointers.inode_tree\n        || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
+增补 3 第 2 件（理想模型自身）：载荷容量少算一字节	crates/singlefs-harness/src/model.rs	    DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET	    DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET - 1	-p singlefs-harness --lib -- model::tests::content_of_exactly	content_of_exactly_the_payload_capacity_is_accepted_and_one_byte_more_is_refused
+增补 3 第 2 件（理想模型自身）：抬 F 上限取第 3 新的非空根	crates/singlefs-harness/src/model.rs	match non_empty_txgs.get(3) {	match non_empty_txgs.get(2) {	-p singlefs-harness --lib -- model::tests::the_ceiling_is	the_ceiling_is_the_fourth_newest_non_empty_root_capped_by_the_newest_root_on_every_device
+增补 3 第 2 件（理想模型自身）：txg = F_生效 的回退目标判成低于 F	crates/singlefs-harness/src/model.rs	if target.checkpoint_txg < self.effective_rollback_floor() {	if target.checkpoint_txg <= self.effective_rollback_floor() {	-p singlefs-harness --lib -- model::tests::rollback_to_the_effective_floor	rollback_to_the_effective_floor_is_accepted_and_abandons_the_newer_roots_of_that_instance
+增补 3 第 2 件（理想模型自身）：F_生效取各盘最大 F 的最大（只一块盘带新 F 也生效）	crates/singlefs-harness/src/model.rs	        highest_floor_per_device\n            .values()\n            .copied()\n            .min()	        highest_floor_per_device\n            .values()\n            .copied()\n            .max()	-p singlefs-harness --lib -- model::tests::a_raised_floor_carried	a_raised_floor_carried_by_one_device_only_does_not_take_effect
+增补 3 第 2 件（理想模型自身）：分配代只要不小于写它的那次发布就算对	crates/singlefs-harness/src/model.rs	&& record.generation == *written_at	&& record.generation >= *written_at	-p singlefs-harness --lib -- model::tests::a_carried_unit_keeps	a_carried_unit_keeps_the_generation_of_the_publish_that_wrote_it
+增补 3 第 2 件（理想模型自身，D13 已定项 5）：模型模块 use 了 singlefs_core	crates/singlefs-harness/src/model.rs	use std::rc::Rc;	use std::rc::Rc;\nuse singlefs_core as _;	-p singlefs-harness --lib -- model_comparison::tests::the_model_module_uses_only	the_model_module_uses_only_the_standard_library_and_the_format_constants
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index c4bfdb3..f7c8968 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -1,6 +1,8 @@
 //! 随机历史（里程碑「第二个事务」增补 3 第 1 件）：按种子生成一段操作序列，操作只调 `crates/` 今天的公开入口；每一步之后对镜像跑
 //! 池级 checker。入口返回 `Err` 算合法结局；panic 与 checker 违例算失败——撞到「已知红」清单（`KNOWN_RED_FORMS`）里的形态照记、
-//! 这段历史到此为止，清单外的算新发现，收缩到最短复现（`shrink_operations`）。冷启动读回的内容对不对这里不判，那是第 2 件模型的事。
+//! 这段历史到此为止，清单外的算新发现，收缩到最短复现（`shrink_operations`）。
+//! 第 2 件接上了理想模型（`crate::model`）：每一步调入口之前问模型该成、该拒还是区间里都行，调完拿实现的结局与它比（胶水在
+//! `crate::model_comparison`）——冷启动读回的内容、回退与抬 F 该不该被拒、根的身份与 F、单元的分配代都由它判，对不上算失败。
 //!
 //! 生成与执行分开：一步操作只带「做什么 + 选择子」，写多长、回退到哪条根、F 抬到多少，执行时按那一刻的盘面与会话现解——
 //! 删掉前面几步之后，后面的操作照样有意义，收缩靠的就是这一条。
@@ -41,6 +43,16 @@ use singlefs_core::unit::data_unit_payload_capacity;
 use singlefs_format::DATA_UNIT_BYTES;
 
 use crate::crash::{MemoryPool, SparseBlockDevice};
+use crate::model::{
+    IdealModel, ModelAnswer, ModelCheckpointTxg, ModelDeviceIdentity, ModelDisagreement,
+    ModelJudgementCounts, ModelPoolGeometry, ObservedEffect, ObservedOutcome,
+};
+use crate::model_comparison::{
+    model_root_key, observed_mount, observed_read_back, observed_root_of_file_version,
+    observed_root_of_version_without_file, refusal_reason_of_block_device_error,
+    refusal_reason_of_mount_error, refusal_reason_of_publish_error,
+    reported_ceiling_of_mount_error,
+};
 use crate::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
 use crate::{RecordingBlockDevice, SharedStream};
 
@@ -179,6 +191,19 @@ pub enum RollbackTargetChoice {
     RingRoot { index_from_newest: u64 },
     /// 不在根环里的目标：最新那条根的实例、txg = 最新 txg + 1 + `txg_beyond_newest mod 3`。
     BeyondNewestRoot { txg_beyond_newest: u64 },
+    /// 候选集的下沿：根环里 txg 等于最新那条根带的 F 的那条根（同一个 txg 上有几条取实例最大的）；F 那一代已被盖掉时取最新那条。
+    /// 只有 `RollbackTargetDraw::FloorRootHalfTheTime` 抽它（理想模型那一格 B2：回退到 txg = F_生效 的根被拒）。
+    RingRootAtTheNewestFloor,
+}
+
+/// 回退的目标按什么抽。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RollbackTargetDraw {
+    /// 第一版的抽法：五分之二取最近四条、五分之二在整个根环里均匀取、五分之一取根环之外。快档、大档与第 121 行的取样点用它，
+    /// 那几档每个种子生成的历史因此与第一版逐项相同。
+    RecentUniformOrBeyond,
+    /// 先抽一次：一半取候选集的下沿（`RingRootAtTheNewestFloor`），另一半照第一版的抽法。
+    FloorRootHalfTheTime,
 }
 
 /// 抬 F 的目标怎么取。
@@ -280,6 +305,7 @@ pub struct GenerationWeights {
     pub with_session_closed: &'static [(HistoryOperationKind, u64)],
     pub with_session_open_without_file: &'static [(HistoryOperationKind, u64)],
     pub with_session_open_with_file: &'static [(HistoryOperationKind, u64)],
+    pub rollback_targets: RollbackTargetDraw,
 }
 
 impl GenerationWeights {
@@ -314,6 +340,7 @@ impl GenerationWeights {
             (HistoryOperationKind::PublishFirstFile, 4),
             (HistoryOperationKind::PublishWithoutUnits, 2),
         ],
+        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
     };
 
     /// 第 121 行那一类（复用时新记录罩住别的已回收记录）的专门取样点：多从第一个文件起、多覆盖写、多抬 F、多可写挂载，
@@ -348,6 +375,42 @@ impl GenerationWeights {
             (HistoryOperationKind::PublishFirstFile, 1),
             (HistoryOperationKind::PublishWithoutUnits, 1),
         ],
+        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
+    };
+
+    /// 理想模型那一格（B2：回退到 txg = F_生效 的根被拒）的专门取样点：多从第一个文件起、多覆盖写（攒非空根，抬 F 的上限才抬得动）、
+    /// 多抬 F、多回退，少冷启动；回退的目标一半取候选集的下沿（`RollbackTargetDraw::FloorRootHalfTheTime`）。快档那组比重与抽法下
+    /// 回退落到 F 那条根上一次都没有（2026-09-19 数：快档 96 段、第 121 行取样点 48 段都是 0 次；只换比重、不换抽法，192 段里 2 次）。
+    pub const ROLLBACK_AFTER_RAISING_THE_FLOOR: GenerationWeights = GenerationWeights {
+        name: "偏向抬 F 之后回退（B2 那一格的取样点）",
+        starting_points: &[
+            (HistoryStartingPoint::AfterMakeFilesystem, 1),
+            (HistoryStartingPoint::AfterFirstFile, 9),
+        ],
+        with_session_closed: &[
+            (HistoryOperationKind::CloseAndMountWritable, 50),
+            (HistoryOperationKind::CloseAndMountRollback, 45),
+            (HistoryOperationKind::ColdStartRecover, 5),
+        ],
+        with_session_open_without_file: &[
+            (HistoryOperationKind::PublishFirstFile, 85),
+            (HistoryOperationKind::PublishWithoutUnits, 3),
+            (HistoryOperationKind::CloseAndMountWritable, 6),
+            (HistoryOperationKind::CloseAndMountRollback, 2),
+            (HistoryOperationKind::ColdStartRecover, 2),
+            (HistoryOperationKind::PublishOverwrite, 1),
+            (HistoryOperationKind::RaiseRollbackFloor, 1),
+        ],
+        with_session_open_with_file: &[
+            (HistoryOperationKind::PublishOverwrite, 40),
+            (HistoryOperationKind::RaiseRollbackFloor, 25),
+            (HistoryOperationKind::CloseAndMountRollback, 25),
+            (HistoryOperationKind::CloseAndMountWritable, 7),
+            (HistoryOperationKind::ColdStartRecover, 1),
+            (HistoryOperationKind::PublishFirstFile, 1),
+            (HistoryOperationKind::PublishWithoutUnits, 1),
+        ],
+        rollback_targets: RollbackTargetDraw::FloorRootHalfTheTime,
     };
 
     fn for_session(&self, expected: ExpectedSession) -> &'static [(HistoryOperationKind, u64)] {
@@ -393,7 +456,11 @@ fn draw_content(source: &mut SeededRandomSource) -> ContentChoice {
     }
 }
 
-fn draw_operation(source: &mut SeededRandomSource, kind: HistoryOperationKind) -> HistoryOperation {
+fn draw_operation(
+    source: &mut SeededRandomSource,
+    kind: HistoryOperationKind,
+    rollback_targets: RollbackTargetDraw,
+) -> HistoryOperation {
     match kind {
         HistoryOperationKind::PublishFirstFile => {
             HistoryOperation::PublishFirstFile(draw_content(source))
@@ -404,6 +471,15 @@ fn draw_operation(source: &mut SeededRandomSource, kind: HistoryOperationKind) -
         HistoryOperationKind::PublishWithoutUnits => HistoryOperation::PublishWithoutUnits,
         HistoryOperationKind::CloseAndMountWritable => HistoryOperation::CloseAndMountWritable,
         HistoryOperationKind::CloseAndMountRollback => {
+            let takes_the_floor_root = match rollback_targets {
+                RollbackTargetDraw::RecentUniformOrBeyond => false,
+                RollbackTargetDraw::FloorRootHalfTheTime => source.below(2) == 0,
+            };
+            if takes_the_floor_root {
+                return HistoryOperation::CloseAndMountRollback(
+                    RollbackTargetChoice::RingRootAtTheNewestFloor,
+                );
+            }
             // 一半取最近的几条（多半在候选集里），一半在整个根环里均匀取（被抛弃的、F 之下的、暖机那两条树表 0 条的都落得到），
             // 余下的取根环之外。
             let target = match source.below(5) {
@@ -476,7 +552,7 @@ pub fn generate_history_with_weights(
     let mut operations = Vec::with_capacity(operation_count);
     for _ in 0..operation_count {
         let kind = draw_weighted(&mut source, weights.for_session(expected));
-        operations.push(draw_operation(&mut source, kind));
+        operations.push(draw_operation(&mut source, kind, weights.rollback_targets));
         expected = expected_session_after(expected, has_file_expected, kind);
         has_file_expected = has_file_expected || expected == ExpectedSession::OpenWithFile;
     }
@@ -496,16 +572,65 @@ struct WritableSession {
     publishes_in_this_mount: usize,
 }
 
-/// 一段历史跑到哪了：两块盘、这个进程的会话、挂载的次数。
+/// 一段历史跑到哪了：两块盘、这个进程的会话、挂载的次数、理想模型、录制流（判「拒绝之前写没写盘」）。
 struct HistoryPool {
     devices: Vec<(DeviceIdentity, HistoryDevice)>,
     session: Option<WritableSession>,
     successful_mounts: usize,
     mount_attempts: usize,
+    model: IdealModel,
+    stream: SharedStream,
+}
+
+/// 模型对一步的判定：对不上的那一格（没有就是 None）与这一步比了多少格。
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
+pub struct ModelVerdict {
+    pub disagreement: Option<ModelDisagreement>,
+    pub counts: ModelJudgementCounts,
+}
+
+/// 拿实现的结局与模型的答案比，对得上就让模型往前走（模型答不了的，答案本身就是那一格对不上）。
+fn judge_by_model(
+    model: &mut IdealModel,
+    answer: Result<ModelAnswer, ModelDisagreement>,
+    observed: &ObservedOutcome,
+) -> ModelVerdict {
+    match answer.and_then(|answer| model.judge_and_advance(&answer, observed)) {
+        Ok(counts) => ModelVerdict {
+            disagreement: None,
+            counts,
+        },
+        Err(disagreement) => ModelVerdict {
+            disagreement: Some(disagreement),
+            counts: ModelJudgementCounts::default(),
+        },
+    }
+}
+
+/// 入口返回 Err 时交给模型的观测：成员映射成的理由、拒之前做完几次发布、录制流在这一步里有没有多出写或屏障。
+fn observed_refusal(
+    member: String,
+    reason: crate::model::ObservedRefusalReason,
+    publishes_completed: usize,
+    stream_length_before: usize,
+    stream: &SharedStream,
+    reported_ceiling: Option<ModelCheckpointTxg>,
+) -> ObservedOutcome {
+    ObservedOutcome::Refused {
+        member,
+        reason,
+        publishes_completed,
+        wrote_anything: stream.operation_count() != stream_length_before,
+        reported_ceiling,
+    }
 }
 
 impl HistoryPool {
-    fn start(starting_point: HistoryStartingPoint, stream: &SharedStream) -> Self {
+    /// 起点：mkfs（加上同一个进程里的取号、暖机、第一个文件）。模型跟着走一遍，第一个文件那次发布拿实现的输出与模型比，判定一并交回。
+    fn start(
+        starting_point: HistoryStartingPoint,
+        stream: &SharedStream,
+    ) -> (Self, Option<ModelVerdict>) {
         let parameters = history_parameters();
         let mut devices: Vec<(DeviceIdentity, HistoryDevice)> =
             [DeviceIdentity(0), DeviceIdentity(1)]
@@ -526,6 +651,14 @@ impl HistoryPool {
                 .collect();
         let genesis = make_filesystem(&parameters, &mut devices)
             .expect("两块全零的 4 GiB 内存盘上按 E142 参数 mkfs：几何放得下，内存盘的写不报错");
+        let mut model = IdealModel::after_make_filesystem(ModelPoolGeometry {
+            devices: devices
+                .iter()
+                .map(|(identity, _)| ModelDeviceIdentity(identity.0))
+                .collect(),
+            device_size_in_bytes: HISTORY_DEVICE_BYTES,
+        });
+        let mut verdict = None;
         let session = match starting_point {
             HistoryStartingPoint::AfterMakeFilesystem => None,
             HistoryStartingPoint::AfterFirstFile => {
@@ -548,18 +681,29 @@ impl HistoryPool {
                     .expect("刚 mkfs 的池取号：内存盘的写与屏障不报错");
                 let warmed = warm_up(&mut writer, &genesis.root, instance)
                     .expect("刚取号的池暖机：内存盘的写不报错");
+                let content = first_file_content();
                 let output = publish_first_file(
                     &mut writer,
                     &mut allocator,
                     &genesis.root,
                     FirstFile {
-                        content: &first_file_content(),
+                        content: &content,
                         write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                     },
                     instance,
                     &warmed.last_record_bytes,
                 )
                 .expect("暖机之后的第一个文件：与 build_pool 同一条路，各步用例都走过");
+                model.acquire_and_warm_up_in_the_make_filesystem_process();
+                let answer = model.answer_publish_first_file(&content);
+                verdict = Some(judge_by_model(
+                    &mut model,
+                    answer,
+                    &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
+                        roots: vec![observed_root_of_file_version(&output)],
+                        reported_ceiling: None,
+                    }),
+                ));
                 Some(WritableSession {
                     allocator,
                     current: PoolVersion::WithFile(output),
@@ -568,12 +712,17 @@ impl HistoryPool {
                 })
             }
         };
-        Self {
-            devices,
-            session,
-            successful_mounts: 0,
-            mount_attempts: 0,
-        }
+        (
+            Self {
+                devices,
+                session,
+                successful_mounts: 0,
+                mount_attempts: 0,
+                model,
+                stream: stream.clone(),
+            },
+            verdict,
+        )
     }
 
     /// 两块盘此刻的整份镜像（拷一份，checker 与观察者读它）。
@@ -892,6 +1041,8 @@ pub struct FailureObservation {
     pub root_ring_slot_count: Option<u64>,
     /// 执行器自己判出的失败（分配代、冷启动读回）。
     pub harness_judgement: Option<HarnessJudgement>,
+    /// 理想模型与实现对不上的那一格（第 2 件）。
+    pub model_disagreement: Option<ModelDisagreement>,
     /// 抬 F 那一步（入口返回 Ok）之后判红时：抬之前的镜像上，新 F 那个 txg 上的根是不是全属于被抛弃的实例
     /// （`raised_floor_lands_only_on_abandoned_roots`）；别的步、读不出、那个 txg 上没有根，都是 None。
     pub raised_floor_lands_only_on_abandoned_roots: Option<bool>,
@@ -926,10 +1077,11 @@ pub struct KnownRedForm {
     pub matches: fn(&FailureObservation) -> bool,
 }
 
-/// 没有 panic、执行器没判出失败、判红的只有 I-3.1、而且是记账的已分配大于遍历全部有效根得到的（记账多算，不是少算）。
+/// 没有 panic、执行器没判出失败、模型没对不上、判红的只有 I-3.1、而且是记账的已分配大于遍历全部有效根得到的（记账多算，不是少算）。
 fn only_allocated_statistic_above_walked(observation: &FailureObservation) -> bool {
     observation.panic.is_none()
         && observation.harness_judgement.is_none()
+        && observation.model_disagreement.is_none()
         && !observation.violations.is_empty()
         && observation.violations.iter().all(|(invariant, detail)| {
             *invariant == "I-3.1"
@@ -973,20 +1125,29 @@ pub const KNOWN_RED_FORMS: [KnownRedForm; 2] = [
 pub enum FailureSignature {
     Panic { location: String },
     HarnessJudgement { judgement: &'static str },
+    ModelDisagreement { aspect: &'static str },
     CheckerViolations { invariants: Vec<&'static str> },
 }
 
 impl FailureSignature {
-    /// panic 先于执行器判的失败，执行器判的失败先于 checker 的违例（同一步里都有时签名取前一种，违例照样在观察里）。
+    /// panic 先于执行器判的失败，执行器判的失败先于模型对不上，模型对不上先于 checker 的违例（同一步里都有时签名取前一种，
+    /// 别的照样在观察里）。
     fn of(observation: &FailureObservation) -> Self {
-        match (&observation.panic, &observation.harness_judgement) {
-            (Some(panic), Some(_) | None) => FailureSignature::Panic {
+        match (
+            &observation.panic,
+            &observation.harness_judgement,
+            &observation.model_disagreement,
+        ) {
+            (Some(panic), Some(_) | None, Some(_) | None) => FailureSignature::Panic {
                 location: panic.location.clone(),
             },
-            (None, Some(judgement)) => FailureSignature::HarnessJudgement {
+            (None, Some(judgement), Some(_) | None) => FailureSignature::HarnessJudgement {
                 judgement: judgement.name(),
             },
-            (None, None) => FailureSignature::CheckerViolations {
+            (None, None, Some(disagreement)) => FailureSignature::ModelDisagreement {
+                aspect: disagreement.aspect.name(),
+            },
+            (None, None, None) => FailureSignature::CheckerViolations {
                 invariants: observation
                     .violations
                     .iter()
@@ -1058,6 +1219,9 @@ pub struct HistoryTally {
     pub checker_runs_skipped_because_nothing_was_written: u64,
     pub invariant_holds: BTreeMap<&'static str, u64>,
     pub invariant_not_applicable: BTreeMap<&'static str, u64>,
+    /// 理想模型判过的步数（起点那次第一个文件也算一步；前提不满足、入口没调的不算）与各格计数。
+    pub model_judged_steps: u64,
+    pub model_counts: ModelJudgementCounts,
 }
 
 impl HistoryTally {
@@ -1124,6 +1288,13 @@ impl HistoryTally {
         for (invariant, count) in &other.invariant_not_applicable {
             *self.invariant_not_applicable.entry(invariant).or_insert(0) += count;
         }
+        self.model_judged_steps += other.model_judged_steps;
+        self.model_counts.add(&other.model_counts);
+    }
+
+    fn note_model_verdict(&mut self, verdict: &ModelVerdict) {
+        self.model_judged_steps += 1;
+        self.model_counts.add(&verdict.counts);
     }
 
     fn note_reuse(&mut self, reuse: RecordReuse) {
@@ -1283,6 +1454,20 @@ impl HistoryTally {
             "  checker 跑了 {} 次；一个写都没发、沿用上一次结论的 {} 步",
             self.checker_runs, self.checker_runs_skipped_because_nothing_was_written
         );
+        let counts = &self.model_counts;
+        let _ = writeln!(
+            text,
+            "  模型对拍 {} 步：该拒而拒 {}、区间里拒 {}、该成而成 {}；比过根 {} 条、分配记录 {} 条、冷启动内容 {} 次、抬 F 上限 {} 次；回退到 txg = F_生效 > 0 的根做成 {} 次",
+            self.model_judged_steps,
+            counts.required_refusals_matched,
+            counts.permitted_refusals_taken,
+            counts.successes_matched,
+            counts.roots_compared,
+            counts.allocation_records_compared,
+            counts.cold_start_contents_compared,
+            counts.ceilings_compared,
+            counts.rollbacks_accepted_at_the_effective_floor
+        );
         for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
             let _ = writeln!(
                 text,
@@ -1459,19 +1644,42 @@ fn ring_roots_newest_first(image: &MemoryPool) -> (Vec<(u64, u32)>, Option<u64>)
     (roots, Some(geometry.regions * geometry.slots_per_region))
 }
 
-/// 一步操作交回执行器的东西：结局，与执行器从这一步交回的东西里自己判出的失败（没有就是 None）。
+/// 按 checker 的读法：根环里最新那条根（(txg, 实例) 最大）带的 F；读不到时 None。
+fn newest_ring_root_floor(image: &MemoryPool) -> Option<u64> {
+    let geometry = chosen_superblocks(image)
+        .into_iter()
+        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))?;
+    valid_roots(image, &geometry)
+        .into_iter()
+        .map(|(_, _, view)| view)
+        .max_by_key(|view| (view.checkpoint_txg, view.instance))
+        .map(|view| view.rollback_floor)
+}
+
+/// 一步操作交回执行器的东西：结局，执行器从这一步交回的东西里自己判出的失败（没有就是 None），模型的判定（入口没调就是 None）。
 struct AppliedStep {
     outcome: StepOutcome,
     harness_judgement: Option<HarnessJudgement>,
+    model_verdict: Option<ModelVerdict>,
 }
 
 impl AppliedStep {
-    /// 这一步交回的东西里没有执行器要另判的（前提不满足、入口返回 Err、挂载、零单元发布）：只看结局本身（冷启动读回报错）。
-    fn judged_by_outcome_only(outcome: StepOutcome) -> Self {
+    /// 前提不满足、入口没调：只看结局本身，模型不问。
+    fn not_applicable(precondition: MissingPrecondition) -> Self {
+        Self {
+            outcome: StepOutcome::NotApplicable(precondition),
+            harness_judgement: None,
+            model_verdict: None,
+        }
+    }
+
+    /// 这一步交回的东西里没有执行器要另判的（入口返回 Err、零单元发布、冷启动）：执行器只看结局本身（冷启动读回报错），连同模型的判定。
+    fn judged_by_outcome_and_model(outcome: StepOutcome, model_verdict: ModelVerdict) -> Self {
         let harness_judgement = harness_judgement_of_outcome(&outcome);
         Self {
             outcome,
             harness_judgement,
+            model_verdict: Some(model_verdict),
         }
     }
 }
@@ -1503,17 +1711,21 @@ fn apply_publish_first_file(
     write_time_seconds: u64,
 ) -> AppliedStep {
     let HistoryPool {
-        devices, session, ..
+        devices,
+        session,
+        model,
+        stream,
+        ..
     } = pool;
     let Some(session) = session.as_mut() else {
-        return AppliedStep::judged_by_outcome_only(StepOutcome::NotApplicable(
-            MissingPrecondition::NoWritableSession,
-        ));
+        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
     };
     let content = content_choice.bytes();
     let root_to_carry_instance_table_from = *session.current.root();
     let previous_record_bytes = session.current.record_bytes().to_vec();
     let records_before = session.allocator.records().to_vec();
+    let answer = model.answer_publish_first_file(&content);
+    let stream_length_before = stream.operation_count();
     let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
     match publish_first_file(
         &mut writer,
@@ -1526,25 +1738,56 @@ fn apply_publish_first_file(
         session.instance,
         &previous_record_bytes,
     ) {
-        Ok(output) => {
-            let reuse = RecordReuse::between(&records_before, session.allocator.records());
-            let publish_txg = output.root.checkpoint_txg;
-            let harness_judgement = allocation_generation_judgement(
-                &records_before,
-                session.allocator.records(),
-                publish_txg,
-                publish_txg,
+        Ok(output) => settle_file_publish(session, model, answer, &records_before, output),
+        Err(error) => {
+            let member = publish_error_member(&error);
+            let verdict = judge_by_model(
+                model,
+                answer,
+                &observed_refusal(
+                    member.clone(),
+                    refusal_reason_of_publish_error(&error),
+                    0,
+                    stream_length_before,
+                    stream,
+                    None,
+                ),
             );
-            session.current = PoolVersion::WithFile(output);
-            session.publishes_in_this_mount += 1;
-            AppliedStep {
-                outcome: StepOutcome::Applied(AppliedEffect::Published { reuse }),
-                harness_judgement,
-            }
+            AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
         }
-        Err(error) => AppliedStep::judged_by_outcome_only(StepOutcome::Refused {
-            member: publish_error_member(&error),
+    }
+}
+
+/// 第一个文件、覆盖写做成之后：数复用、判分配代（第 1 件）、拿输出与模型比（第 2 件）、把现行版本换成这一版。
+fn settle_file_publish(
+    session: &mut WritableSession,
+    model: &mut IdealModel,
+    answer: Result<ModelAnswer, ModelDisagreement>,
+    records_before: &[AllocationRecord],
+    output: singlefs_core::transaction::TransactionOutput,
+) -> AppliedStep {
+    let reuse = RecordReuse::between(records_before, session.allocator.records());
+    let publish_txg = output.root.checkpoint_txg;
+    let harness_judgement = allocation_generation_judgement(
+        records_before,
+        session.allocator.records(),
+        publish_txg,
+        publish_txg,
+    );
+    let verdict = judge_by_model(
+        model,
+        answer,
+        &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
+            roots: vec![observed_root_of_file_version(&output)],
+            reported_ceiling: None,
         }),
+    );
+    session.current = PoolVersion::WithFile(output);
+    session.publishes_in_this_mount += 1;
+    AppliedStep {
+        outcome: StepOutcome::Applied(AppliedEffect::Published { reuse }),
+        harness_judgement,
+        model_verdict: Some(verdict),
     }
 }
 
@@ -1555,21 +1798,23 @@ fn apply_publish_overwrite(
     write_time_seconds: u64,
 ) -> AppliedStep {
     let HistoryPool {
-        devices, session, ..
+        devices,
+        session,
+        model,
+        stream,
+        ..
     } = pool;
     let Some(session) = session.as_mut() else {
-        return AppliedStep::judged_by_outcome_only(StepOutcome::NotApplicable(
-            MissingPrecondition::NoWritableSession,
-        ));
+        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
     };
     let PoolVersion::WithFile(previous) = &session.current else {
-        return AppliedStep::judged_by_outcome_only(StepOutcome::NotApplicable(
-            MissingPrecondition::CurrentVersionWithoutFile,
-        ));
+        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
     };
     let previous = previous.clone();
     let content = content_choice.bytes();
     let records_before = session.allocator.records().to_vec();
+    let answer = model.answer_publish_overwrite(&content);
+    let stream_length_before = stream.operation_count();
     let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
     match publish_overwrite(
         &mut writer,
@@ -1581,42 +1826,46 @@ fn apply_publish_overwrite(
         },
         session.instance,
     ) {
-        Ok(output) => {
-            let reuse = RecordReuse::between(&records_before, session.allocator.records());
-            let publish_txg = output.root.checkpoint_txg;
-            let harness_judgement = allocation_generation_judgement(
-                &records_before,
-                session.allocator.records(),
-                publish_txg,
-                publish_txg,
+        Ok(output) => settle_file_publish(session, model, answer, &records_before, output),
+        Err(error) => {
+            let member = publish_error_member(&error);
+            let verdict = judge_by_model(
+                model,
+                answer,
+                &observed_refusal(
+                    member.clone(),
+                    refusal_reason_of_publish_error(&error),
+                    0,
+                    stream_length_before,
+                    stream,
+                    None,
+                ),
             );
-            session.current = PoolVersion::WithFile(output);
-            session.publishes_in_this_mount += 1;
-            AppliedStep {
-                outcome: StepOutcome::Applied(AppliedEffect::Published { reuse }),
-                harness_judgement,
-            }
+            AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
         }
-        Err(error) => AppliedStep::judged_by_outcome_only(StepOutcome::Refused {
-            member: publish_error_member(&error),
-        }),
     }
 }
 
 fn apply_publish_without_units(
     pool: &mut HistoryPool,
     parameters: &MakeFilesystemParameters,
-) -> StepOutcome {
+) -> AppliedStep {
     let HistoryPool {
-        devices, session, ..
+        devices,
+        session,
+        model,
+        stream,
+        ..
     } = pool;
     let Some(session) = session.as_mut() else {
-        return StepOutcome::NotApplicable(MissingPrecondition::NoWritableSession);
+        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
     };
     // 前提：只在树表 0 条的一版上调（出处见 `MissingPrecondition::CurrentVersionWithFile` 的注释）。
     let PoolVersion::WithoutFile(previous) = &session.current else {
-        return StepOutcome::NotApplicable(MissingPrecondition::CurrentVersionWithFile);
+        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithFile);
     };
+    let answer = model.answer_publish_without_units();
+    let stream_length_before = stream.operation_count();
     let plan = ZeroUnitPublishPlan {
         txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
         counter: previous.record.counter + 1,
@@ -1628,18 +1877,42 @@ fn apply_publish_without_units(
     let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
     match publish_without_units(&mut writer, &previous_root, plan) {
         Ok(output) => {
+            let verdict = judge_by_model(
+                model,
+                answer,
+                &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
+                    roots: vec![observed_root_of_version_without_file(&output)],
+                    reported_ceiling: None,
+                }),
+            );
             session.current = PoolVersion::WithoutFile(output);
             session.publishes_in_this_mount += 1;
-            StepOutcome::Applied(AppliedEffect::Published {
-                reuse: RecordReuse::default(),
-            })
+            AppliedStep::judged_by_outcome_and_model(
+                StepOutcome::Applied(AppliedEffect::Published {
+                    reuse: RecordReuse::default(),
+                }),
+                verdict,
+            )
         }
-        Err(error) => StepOutcome::Refused {
-            member: format!(
+        Err(error) => {
+            let member = format!(
                 "publish_without_units({})",
                 block_device_error_member(&error)
-            ),
-        },
+            );
+            let verdict = judge_by_model(
+                model,
+                answer,
+                &observed_refusal(
+                    member.clone(),
+                    refusal_reason_of_block_device_error(&error),
+                    0,
+                    stream_length_before,
+                    stream,
+                    None,
+                ),
+            );
+            AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
+        }
     }
 }
 
@@ -1712,15 +1985,23 @@ fn mount_publish_allocation_judgement(
     (comparison, reuse, harness_judgement)
 }
 
+/// 挂载（可写挂载或回退）入口返回之后：做成的判分配代（第 1 件）、与模型比（第 2 件）、开会话；被拒的与模型比理由、写没写盘。
 fn settle_mount(
     pool: &mut HistoryPool,
     mounted: Result<singlefs_core::mount::Mounted, MountError>,
     image_before_mount: &MemoryPool,
+    answer: ModelAnswer,
+    stream_length_before: usize,
 ) -> AppliedStep {
     match mounted {
         Ok(mounted) => {
             let (allocation_records_compared, reuse, harness_judgement) =
                 mount_publish_allocation_judgement(image_before_mount, &mounted);
+            let verdict = judge_by_model(
+                &mut pool.model,
+                Ok(answer),
+                &ObservedOutcome::Succeeded(observed_mount(&mounted)),
+            );
             let publishes = 1 + mounted.output.warm_up_publishes.len();
             let instance = mounted.output.instance;
             pool.successful_mounts += 1;
@@ -1738,12 +2019,26 @@ fn settle_mount(
                     reuse,
                 }),
                 harness_judgement,
+                model_verdict: Some(verdict),
             }
         }
-        // 挂载半路报错（写行之后的发布失败）：这里不比，入口没交回写出去的那几次发布。
-        Err(error) => AppliedStep::judged_by_outcome_only(StepOutcome::Refused {
-            member: mount_error_member(&error),
-        }),
+        // 挂载半路报错（写行之后的发布失败）：第 1 件不比分配代，入口没交回写出去的那几次发布；模型按「拒之前一个字节都不写」判。
+        Err(error) => {
+            let member = mount_error_member(&error);
+            let verdict = judge_by_model(
+                &mut pool.model,
+                Ok(answer),
+                &observed_refusal(
+                    member.clone(),
+                    refusal_reason_of_mount_error(&error),
+                    0,
+                    stream_length_before,
+                    &pool.stream,
+                    None,
+                ),
+            );
+            AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
+        }
     }
 }
 
@@ -1752,10 +2047,19 @@ fn apply_mount_writable(
     parameters: &MakeFilesystemParameters,
 ) -> AppliedStep {
     pool.session = None;
+    pool.model.close_session();
     pool.mount_attempts += 1;
     let image_before_mount = pool.image();
+    let answer = pool.model.answer_mount_writable();
+    let stream_length_before = pool.stream.operation_count();
     let mounted = mount_writable(parameters, &mut pool.devices);
-    settle_mount(pool, mounted, &image_before_mount)
+    settle_mount(
+        pool,
+        mounted,
+        &image_before_mount,
+        answer,
+        stream_length_before,
+    )
 }
 
 fn apply_mount_rollback(
@@ -1764,12 +2068,11 @@ fn apply_mount_rollback(
     choice: RollbackTargetChoice,
 ) -> AppliedStep {
     pool.session = None;
+    pool.model.close_session();
     let image_before_mount = pool.image();
     let (roots, _) = ring_roots_newest_first(&image_before_mount);
     let Some((newest_txg, newest_instance)) = roots.first().copied() else {
-        return AppliedStep::judged_by_outcome_only(StepOutcome::NotApplicable(
-            MissingPrecondition::NoReadableRootInRing,
-        ));
+        return AppliedStep::not_applicable(MissingPrecondition::NoReadableRootInRing);
     };
     let target = match choice {
         RollbackTargetChoice::RingRoot { index_from_newest } => {
@@ -1785,10 +2088,33 @@ fn apply_mount_rollback(
             instance: InstanceGeneration(newest_instance),
             checkpoint_txg: CheckpointTxg(newest_txg + 1 + txg_beyond_newest % 3),
         },
+        RollbackTargetChoice::RingRootAtTheNewestFloor => {
+            let floor = newest_ring_root_floor(&image_before_mount);
+            // 根环按 (txg, 实例) 从新到旧排，同一个 txg 上先碰到的就是实例最大的那条。
+            let (txg, instance) = roots
+                .iter()
+                .copied()
+                .find(|(txg, _)| Some(*txg) == floor)
+                .unwrap_or((newest_txg, newest_instance));
+            RollbackTarget {
+                instance: InstanceGeneration(instance),
+                checkpoint_txg: CheckpointTxg(txg),
+            }
+        }
     };
     pool.mount_attempts += 1;
+    let answer = pool
+        .model
+        .answer_mount_rollback(model_root_key(target.instance, target.checkpoint_txg));
+    let stream_length_before = pool.stream.operation_count();
     let mounted = mount_rollback(parameters, &mut pool.devices, target, ShadowLedger::On);
-    settle_mount(pool, mounted, &image_before_mount)
+    settle_mount(
+        pool,
+        mounted,
+        &image_before_mount,
+        answer,
+        stream_length_before,
+    )
 }
 
 fn apply_raise_rollback_floor(
@@ -1797,12 +2123,14 @@ fn apply_raise_rollback_floor(
     choice: FloorTargetChoice,
 ) -> AppliedStep {
     let HistoryPool {
-        devices, session, ..
+        devices,
+        session,
+        model,
+        stream,
+        ..
     } = pool;
     let Some(session) = session.as_mut() else {
-        return AppliedStep::judged_by_outcome_only(StepOutcome::NotApplicable(
-            MissingPrecondition::NoWritableSession,
-        ));
+        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
     };
     let WritableSession {
         allocator,
@@ -1811,9 +2139,7 @@ fn apply_raise_rollback_floor(
         ..
     } = session;
     let PoolVersion::WithFile(current) = current else {
-        return AppliedStep::judged_by_outcome_only(StepOutcome::NotApplicable(
-            MissingPrecondition::CurrentVersionWithoutFile,
-        ));
+        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
     };
     let current_floor = current.root.rollback_floor.0;
     let txg_before_raise = current.root.checkpoint_txg.0;
@@ -1822,6 +2148,8 @@ fn apply_raise_rollback_floor(
     let new_floor = CheckpointTxg(current_floor + choice.steps_above_current_floor % choices);
     let records_before = allocator.records().to_vec();
     let records_on_disk_before = current.allocation_records.clone();
+    let answer = model.answer_raise_rollback_floor(ModelCheckpointTxg(new_floor.0));
+    let stream_length_before = stream.operation_count();
     let raised = raise_rollback_floor(
         parameters,
         devices,
@@ -1831,10 +2159,23 @@ fn apply_raise_rollback_floor(
         ShadowLedger::On,
     );
     // 抬 F 的空发布逐次把现行版本往前推：半路报错时已经推出去的那几次也算这次挂载写出的根。
-    *publishes_in_this_mount += usize::try_from(current.root.checkpoint_txg.0 - txg_before_raise)
+    let publishes_completed = usize::try_from(current.root.checkpoint_txg.0 - txg_before_raise)
         .expect("一次抬 F 至多推根环区域数那么多次");
+    *publishes_in_this_mount += publishes_completed;
     match raised {
         Ok(raised) => {
+            let verdict = judge_by_model(
+                model,
+                answer,
+                &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
+                    roots: raised
+                        .publishes
+                        .iter()
+                        .map(observed_root_of_file_version)
+                        .collect(),
+                    reported_ceiling: Some(ModelCheckpointTxg(raised.ceiling.0)),
+                }),
+            );
             // 逐次发布比盘上的分配记录：每一次改写或新增的记录，代是那一次的 txg。
             let mut records_of_previous_publish = &records_on_disk_before;
             let mut harness_judgement = None;
@@ -1858,35 +2199,63 @@ fn apply_raise_rollback_floor(
                     reuse: RecordReuse::between(&records_before, allocator.records()),
                 }),
                 harness_judgement,
+                model_verdict: Some(verdict),
             }
         }
         // 半路报错：推出去的那几次没有逐次的输出，现行版本是最后成功的那一次；改写或新增的记录的代要落在这几次的 txg 里。
-        Err(error) => AppliedStep {
-            outcome: StepOutcome::Refused {
-                member: mount_error_member(&error),
-            },
-            harness_judgement: allocation_generation_judgement(
-                &records_on_disk_before,
-                &current.allocation_records,
-                CheckpointTxg(txg_before_raise + 1),
-                current.root.checkpoint_txg,
-            ),
-        },
+        // 模型比理由（容量墙按做完的次数判区间）与上限。
+        Err(error) => {
+            let member = mount_error_member(&error);
+            let verdict = judge_by_model(
+                model,
+                answer,
+                &observed_refusal(
+                    member.clone(),
+                    refusal_reason_of_mount_error(&error),
+                    publishes_completed,
+                    stream_length_before,
+                    stream,
+                    reported_ceiling_of_mount_error(&error),
+                ),
+            );
+            AppliedStep {
+                outcome: StepOutcome::Refused { member },
+                harness_judgement: allocation_generation_judgement(
+                    &records_on_disk_before,
+                    &current.allocation_records,
+                    CheckpointTxg(txg_before_raise + 1),
+                    current.root.checkpoint_txg,
+                ),
+                model_verdict: Some(verdict),
+            }
+        }
     }
 }
 
-fn apply_cold_start_recover(pool: &mut HistoryPool) -> StepOutcome {
+fn apply_cold_start_recover(pool: &mut HistoryPool) -> AppliedStep {
     pool.session = None;
+    pool.model.close_session();
+    let answer = pool.model.answer_cold_start_recover();
     let report = recover(&pool.devices, JournalPolicy::Consult);
+    let verdict = judge_by_model(
+        &mut pool.model,
+        Ok(answer),
+        &ObservedOutcome::Succeeded(ObservedEffect::ColdStart {
+            read_back: observed_read_back(&report.outcome),
+        }),
+    );
     let read_back = match &report.outcome {
         RecoveryOutcome::NoFile { .. } => ColdStartReadBack::NoFile,
         RecoveryOutcome::FileRead { .. } => ColdStartReadBack::FileRead,
         RecoveryOutcome::Failed { .. } => ColdStartReadBack::Failed,
     };
-    StepOutcome::Applied(AppliedEffect::Recovered {
-        outcome: recovery_outcome_member(&report.outcome),
-        read_back,
-    })
+    AppliedStep::judged_by_outcome_and_model(
+        StepOutcome::Applied(AppliedEffect::Recovered {
+            outcome: recovery_outcome_member(&report.outcome),
+            read_back,
+        }),
+        verdict,
+    )
 }
 
 fn apply_operation(
@@ -1906,9 +2275,7 @@ fn apply_operation(
         HistoryOperation::PublishOverwrite(content) => {
             apply_publish_overwrite(pool, &parameters, content, write_time_seconds)
         }
-        HistoryOperation::PublishWithoutUnits => {
-            AppliedStep::judged_by_outcome_only(apply_publish_without_units(pool, &parameters))
-        }
+        HistoryOperation::PublishWithoutUnits => apply_publish_without_units(pool, &parameters),
         HistoryOperation::CloseAndMountWritable => apply_mount_writable(pool, &parameters),
         HistoryOperation::CloseAndMountRollback(choice) => {
             apply_mount_rollback(pool, &parameters, *choice)
@@ -1916,9 +2283,7 @@ fn apply_operation(
         HistoryOperation::RaiseRollbackFloor(choice) => {
             apply_raise_rollback_floor(pool, &parameters, *choice)
         }
-        HistoryOperation::ColdStartRecover => {
-            AppliedStep::judged_by_outcome_only(apply_cold_start_recover(pool))
-        }
+        HistoryOperation::ColdStartRecover => apply_cold_start_recover(pool),
     }
 }
 
@@ -1972,18 +2337,25 @@ pub fn execute_history_observing(
     let position = Cell::new(StepPosition::StartingPoint);
     let mut completed_after_the_root_ring_turned = false;
     let body_result = with_panic_capture(|| -> Option<FailureObservation> {
-        let pool = pool_slot.insert(HistoryPool::start(history.starting_point, stream));
+        let (started_pool, starting_verdict) = HistoryPool::start(history.starting_point, stream);
+        let pool = pool_slot.insert(started_pool);
         let mut image = pool.image();
         let mut checked_stream_length = stream.operation_count();
         let violations_after_the_starting_point = violations_on(&image, &mut tally);
-        if !violations_after_the_starting_point.is_empty() {
-            return Some(failure_observation(
+        if let Some(verdict) = &starting_verdict {
+            tally.note_model_verdict(verdict);
+        }
+        let starting_disagreement = starting_verdict.and_then(|verdict| verdict.disagreement);
+        if !violations_after_the_starting_point.is_empty() || starting_disagreement.is_some() {
+            let mut observation = failure_observation(
                 &image,
                 StepPosition::StartingPoint,
                 None,
                 violations_after_the_starting_point,
                 None,
-            ));
+            );
+            observation.model_disagreement = starting_disagreement;
+            return Some(observation);
         }
         observer(&StepObservation {
             position: StepPosition::StartingPoint,
@@ -1997,8 +2369,13 @@ pub fn execute_history_observing(
             let AppliedStep {
                 outcome,
                 harness_judgement,
+                model_verdict,
             } = apply_operation(pool, operation, step_index);
             tally.note_outcome(operation, &outcome);
+            if let Some(verdict) = &model_verdict {
+                tally.note_model_verdict(verdict);
+            }
+            let model_disagreement = model_verdict.and_then(|verdict| verdict.disagreement);
             if let Some(session) = &pool.session {
                 tally.most_publishes_in_one_mount = tally
                     .most_publishes_in_one_mount
@@ -2035,7 +2412,8 @@ pub fn execute_history_observing(
                     });
                 }
             }
-            if !violations.is_empty() || harness_judgement.is_some() {
+            if !violations.is_empty() || harness_judgement.is_some() || model_disagreement.is_some()
+            {
                 let mut observation = failure_observation(
                     &image,
                     step_position,
@@ -2044,6 +2422,7 @@ pub fn execute_history_observing(
                     None,
                 );
                 observation.harness_judgement = harness_judgement;
+                observation.model_disagreement = model_disagreement;
                 observation.raised_floor_lands_only_on_abandoned_roots =
                     raised_floor_lands_only_on_abandoned;
                 return Some(observation);
@@ -2086,6 +2465,7 @@ pub fn execute_history_observing(
                 newest_ring_root_txg,
                 root_ring_slot_count,
                 harness_judgement: None,
+                model_disagreement: None,
                 raised_floor_lands_only_on_abandoned_roots: None,
             })
         }
@@ -2141,6 +2521,7 @@ fn failure_observation(
         newest_ring_root_txg,
         root_ring_slot_count,
         harness_judgement: None,
+        model_disagreement: None,
         raised_floor_lands_only_on_abandoned_roots: None,
     }
 }
@@ -2259,7 +2640,9 @@ fn simpler_variants(operation: HistoryOperation) -> Vec<HistoryOperation> {
                 })
             })
             .collect(),
-        HistoryOperation::PublishWithoutUnits
+        // 候选集的下沿按那一刻的盘面现解，没有更简单的写法。
+        HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRootAtTheNewestFloor)
+        | HistoryOperation::PublishWithoutUnits
         | HistoryOperation::CloseAndMountWritable
         | HistoryOperation::ColdStartRecover => Vec::new(),
     }
@@ -2439,6 +2822,15 @@ impl NewFindingReport {
         if let Some(judgement) = &self.observation.harness_judgement {
             let _ = writeln!(text, "  执行器判出：{}：{judgement:?}", judgement.name());
         }
+        if let Some(disagreement) = &self.observation.model_disagreement {
+            let _ = writeln!(
+                text,
+                "  模型对不上（{}）：模型答 {}；实现 {}",
+                disagreement.aspect.name(),
+                disagreement.model_answer,
+                disagreement.implementation_answer
+            );
+        }
         let Some(shrunk) = &self.shrunk else {
             let _ = writeln!(
                 text,
@@ -2830,6 +3222,7 @@ mod tests {
                 first_publish_txg: CheckpointTxg(26),
                 last_publish_txg: CheckpointTxg(26),
             }),
+            model_disagreement: None,
             raised_floor_lands_only_on_abandoned_roots: None,
         };
         let HistoryEnding::NewFinding { signature, .. } = classify_failure(observation.clone())
diff --git a/crates/singlefs-harness/src/lib.rs b/crates/singlefs-harness/src/lib.rs
index 46268c4..d0b9307 100644
--- a/crates/singlefs-harness/src/lib.rs
+++ b/crates/singlefs-harness/src/lib.rs
@@ -18,6 +18,8 @@ pub mod device_log;
 pub mod first_transaction_regions;
 pub mod hexadecimal;
 pub mod history;
+pub mod model;
+pub mod model_comparison;
 pub mod scenario;
 pub mod segments;
 pub mod sha256;
diff --git a/crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs b/crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs
index 58a31ec..a52780b 100644
--- a/crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs
+++ b/crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs
@@ -26,6 +26,11 @@ const REUSE_SAMPLING_FIRST_SEED: u64 = 0;
 const REUSE_SAMPLING_SEEDS: u64 = 48;
 const REUSE_SAMPLING_OPERATIONS_PER_HISTORY: usize = 30;
 
+/// 理想模型 B2 那一格的专门取样点：种子区间与每段步数，写死（判出率见那条用例的注释）。
+const ROLLBACK_SAMPLING_FIRST_SEED: u64 = 0;
+const ROLLBACK_SAMPLING_SEEDS: u64 = 48;
+const ROLLBACK_SAMPLING_OPERATIONS_PER_HISTORY: usize = 30;
+
 /// 报告直接写进进程的标准输出，不经 libtest 的捕获：快档通过时计数照样出现在 `check.sh` 的输出里
 /// （`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
 fn print_uncaptured(text: &str) {
@@ -134,6 +139,31 @@ fn assert_every_path_was_exercised(tally: &HistoryTally) {
         tally.histories_that_turned_the_root_ring >= 1,
         "没有一段历史转过根环"
     );
+    assert_the_model_judged_every_kind_of_answer(tally);
+}
+
+/// 理想模型真的判过：该拒而拒、该成而成都有，比过根、分配记录、冷启动读回的内容、抬 F 的上限（第 2 件；阴性结果要能和「模型没跑到」分开）。
+fn assert_the_model_judged_every_kind_of_answer(tally: &HistoryTally) {
+    let counts = &tally.model_counts;
+    assert!(tally.model_judged_steps >= 1, "模型一步都没判");
+    assert!(
+        counts.required_refusals_matched >= 1,
+        "模型一次「该拒而拒」都没判过"
+    );
+    assert!(
+        counts.successes_matched >= 1,
+        "模型一次「该成而成」都没判过"
+    );
+    assert!(counts.roots_compared >= 1, "模型一条根都没比过");
+    assert!(
+        counts.allocation_records_compared >= 1,
+        "模型一条分配记录的分配代都没比过"
+    );
+    assert!(
+        counts.cold_start_contents_compared >= 1,
+        "模型一次冷启动读回的内容都没比过"
+    );
+    assert!(counts.ceilings_compared >= 1, "模型一次抬 F 的上限都没比过");
 }
 
 /// 快档：种子 [0, 96)、每段 30 步。每一步之后跑池级 checker；入口的 `Err` 算合法结局，panic 与违例算失败，撞到「已知红」清单里的形态
@@ -199,6 +229,104 @@ fn reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red
     assert!(tally.raises_that_reclaimed >= 1, "抬 F 一次都没回收到落点");
 }
 
+/// 理想模型那一格 B2（回退到 txg = F_生效 的根被拒，`research/prompts/m2-supp3-item1-code-r1-opus-model/mutants.tsv`）的专门取样点：
+/// 比重取 `GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR`（回退目标一半取候选集的下沿），种子 [0, 48)、每段 30 步。快档那组比重与抽法下
+/// 回退落到 F 那条根上一次都没有。先判清单外的失败（模型对不上也在其内），再核这一格真的跑到了：回退到 txg = F_生效 > 0 的根做成过——
+/// 这个数只在做成时加，变异下被拒、判红的是分类，不是这条计数。
+#[test]
+fn rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms() {
+    let started = std::time::Instant::now();
+    let report = run_history_campaign(
+        ROLLBACK_SAMPLING_FIRST_SEED,
+        ROLLBACK_SAMPLING_SEEDS,
+        ROLLBACK_SAMPLING_OPERATIONS_PER_HISTORY,
+        &GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
+        worker_threads_by_default(),
+        FindingShrinking::ReportSeedsOnly,
+    );
+    let rendered = report.render();
+    print_uncaptured(&format!(
+        "── 随机历史：偏向抬 F 之后回退的取样点 ──\n{rendered}用时 {:.1} 秒\n",
+        started.elapsed().as_secs_f64()
+    ));
+    assert!(
+        report.new_findings.is_empty(),
+        "「已知红」清单外的失败：\n{rendered}"
+    );
+    assert!(
+        report
+            .tally
+            .model_counts
+            .rollbacks_accepted_at_the_effective_floor
+            >= 1,
+        "回退到 txg = F_生效 > 0 的根一次都没做成（B2 那一格没跑到）"
+    );
+}
+
+/// 回退到 txg = F_生效 的根要做成（D16（发布语义） 已定项 1「回退候选集」：txg ≥ F_生效），模型按 B2 那一格判（攻方变异「回退到
+/// txg = F_生效 的根也拒」）。第一个文件（txg 3）之后可写挂载（实例 2：txg 4、5）、覆盖写四次（6–9）、抬 F（选择子 6 ⇒ F = 6：上限是
+/// 第 4 新的非空根 6 与盘 1 上最新的有效根 7 取小，推 txg 10、11 两次）、回退到候选集的下沿 (6, 2)（实例 3：txg 12、13）、冷启动读回。
+/// 今天的代码上这段跑完：回退做成一次、落在 F 上，冷启动读回的是 txg 6 那次覆盖写的内容（模型比过）。取样点里这一格 48 个种子一窗只有
+/// 0–5 段（2026-09-19 在 [0, 960) 上逐窗数），这一条把它钉死。
+#[test]
+fn rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version() {
+    let overwrite = |fill_seed: u64| {
+        HistoryOperation::PublishOverwrite(ContentChoice {
+            length: ContentLength::InsideOneDataUnit { selector: 2999 },
+            fill_seed,
+        })
+    };
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: vec![
+            HistoryOperation::CloseAndMountWritable,
+            overwrite(6),
+            overwrite(7),
+            overwrite(8),
+            overwrite(9),
+            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
+                steps_above_current_floor: 6,
+            }),
+            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRootAtTheNewestFloor),
+            HistoryOperation::ColdStartRecover,
+        ],
+    };
+    let run = execute_history(&history);
+    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
+    assert!(
+        matches!(
+            run.outcomes[5],
+            StepOutcome::Applied(AppliedEffect::RaisedFloor {
+                new_floor: CheckpointTxg(6),
+                publishes: 2,
+                ..
+            })
+        ),
+        "抬到 6、推 txg 10（盘 1）与 11（盘 0）：{:?}",
+        run.outcomes[5]
+    );
+    assert!(
+        matches!(
+            run.outcomes[6],
+            StepOutcome::Applied(AppliedEffect::Mounted { .. })
+        ),
+        "回退到 (6, 2) 要做成：{:?}",
+        run.outcomes[6]
+    );
+    assert_eq!(
+        run.tally
+            .model_counts
+            .rollbacks_accepted_at_the_effective_floor,
+        1,
+        "模型数到一次落在 F_生效 上的回退"
+    );
+    assert_eq!(
+        run.tally.model_counts.cold_start_contents_compared, 1,
+        "冷启动读回的内容模型比过"
+    );
+}
+
 /// 「已知红」清单第 0 条（增补 2 收口表第 ② 行）的复现：第一个文件之后可写挂载一次（写行 txg 4、暖机 txg 5），同一次挂载里连着覆盖写。
 /// 根环 R × S = 24 槽：txg 24、25、26 依次盖掉第 0 代根与两条暖机根，txg 26 那次覆盖写之后再没有一条有效根引用 mkfs 的第 0 版树表单元
 /// （1 槽），它在 txg 3 释放、没回收，记账仍算已分配 ⇒ checker 判 I-3.1 红（记账比遍历多 16384 字节）、别的不变量不红，分类成清单第 0 条；
@@ -365,6 +493,7 @@ fn an_allocated_statistic_over_count_after_raising_the_floor_without_a_rollback_
         newest_ring_root_txg: Some(10),
         root_ring_slot_count: Some(24),
         harness_judgement: None,
+        model_disagreement: None,
         raised_floor_lands_only_on_abandoned_roots: lands_only_on_abandoned,
     };
     let ending = classify_failure(observation);
@@ -483,7 +612,10 @@ fn random_histories_large_tier_seeds_and_length_from_the_environment() {
     let weights = match std::env::var("SINGLEFS_RANDOM_HISTORY_WEIGHTS").as_deref() {
         Err(std::env::VarError::NotPresent) | Ok("broad") => GenerationWeights::BROAD,
         Ok("reuse") => GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
-        Ok(other) => panic!("SINGLEFS_RANDOM_HISTORY_WEIGHTS={other}：只认 broad 与 reuse"),
+        Ok("rollback") => GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
+        Ok(other) => {
+            panic!("SINGLEFS_RANDOM_HISTORY_WEIGHTS={other}：只认 broad、reuse 与 rollback")
+        }
         Err(std::env::VarError::NotUnicode(raw)) => {
             panic!("SINGLEFS_RANDOM_HISTORY_WEIGHTS 不是 UTF-8：{raw:?}")
         }
```

## 二、新文件 `crates/singlefs-harness/src/model.rs` 全文

```rust
//! 理想模型（里程碑「第二个事务」增补 3 第 2 件）：一个只住内存的模型，记每一版提交的内容、每条根属于哪个实例、实例表的行、回退下界 F，
//! 回答「冷启动该读回哪一版」「回退到这条根该不该被拒」「这个错误该不该出现」；第 1 件的每一步拿实现的结局与它比（比的那层胶水在
//! `model_comparison.rs`，那里可以用 `singlefs_core` 的类型）。
//!
//! 只 `use singlefs_format`，不 `use singlefs_core` / `singlefs_checker`（D13（验证路线） 已定项 5：只共享一份从 kb 生成的常量）：代号、实例、
//! 角色、根环落点、F 的生效值、抬 F 的上限、候选集、内容装不装得下都照条款另写一份，不照抄实现的算法。
//!
//! 不建崩溃：第 1 件的历史里没有断电、没有设备错，每一步的写都落完，所以环里最新的那条根就是最后写出的那一条、journal 里没有要施加的记录
//! （可写挂载写的上一个实例那一行 W 恒 0）。第 3 件（崩溃注入）接上时要加「这个崩溃状态恢复到的版本在不在允许集合里」。
//!
//! 条款把答案留给实现取上界的地方（容量墙），模型答「允许拒绝的区间」，不照抄实现的上界算法；打回重议那几处（增补 2 收口表第 ①、② 行）
//! 照代码今天的读法写，每一处标「预想，跟收口表第 X 行」。

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use singlefs_format::{
    index_node_header_bytes, ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, ALLOCATION_RECORD_BYTES,
    ALLOCATION_RECORD_KEY_BYTES, CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES, DATA_UNIT_PAYLOAD_OFFSET,
    FIRST_TRANSACTION_TXG, INSTANCE_TABLE_PAGE_RECORDS, NODE_BYTES, ROOT_RING_REGIONS,
    ROOT_RING_REGION_DEVICES, ROOT_RING_SLOTS_PER_REGION, SLOT_BYTES, UNIT_AREA_START_SLOT,
};

/// 模型里的 checkpoint 号（D16（发布语义） 已定项 6：每次发布 + 1）。与实现的 `CheckpointTxg` 各自声明（D13（验证路线） 已定项 5）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelCheckpointTxg(pub u64);

/// 模型里的实例代号（D23（journal 的角色与格式） 已定项 16：取号 = 见过的最大 + 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelInstanceGeneration(pub u64);

/// 模型里的 jsn 计数器（D23（journal 的角色与格式） 已定项 14 第 3 条：全池接着走、换实例不归零）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelJournalCounter(pub u64);

/// 模型里的盘。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelDeviceIdentity(pub u32);

/// 一条根的身份。择根按 txg 为主、实例代号破平局（D22（单元原子性怎么合成） 已定项 7）：字段次序就是比较次序。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelRootKey {
    pub checkpoint_txg: ModelCheckpointTxg,
    pub instance: ModelInstanceGeneration,
}

/// mkfs 的实例代号与第 0 代根的 txg（D23（journal 的角色与格式） 已定项 16：mkfs 写 0，第一次取号取 1）。
const MAKE_FILESYSTEM_INSTANCE: ModelInstanceGeneration = ModelInstanceGeneration(0);
const MAKE_FILESYSTEM_TXG: ModelCheckpointTxg = ModelCheckpointTxg(0);

/// 记账树里不带设备维的行与每块盘各一组的行（D5（快照 / 空间记账机制） 已定项 8：inode 号水位、待删占用、已承诺预留；每盘已分配、空闲、
/// 不可回收、defer 待释放、碎片段数、全空聚簇段数）。
const POOL_WIDE_ACCOUNTING_ROWS: u64 = 3;
const ACCOUNTING_ROWS_PER_DEVICE: u64 = 6;

/// 一版里的一个角色：第一个文件版本的八个单元（D3（空间分配） 已定项 10 ⑤ 的次序）与实例表单元（D18（块里携带什么信息） 已定项 11）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelUnitRole {
    Data,
    ExtentRoot,
    InodeLeaf,
    InodeRoot,
    AllocationTree,
    AccountingTree,
    MappingTree,
    TreeTable,
    InstanceTable,
}

impl ModelUnitRole {
    /// 占几个槽：数据单元与打包记录单元（inode 叶、实例表）恒 32 KiB（D4（校验和位置） 已定项 5），索引节点恒 16 KiB
    /// （D8（核心索引结构） 已定项 2），落点粒度 16 KiB（D3（空间分配） 已定项 7）。
    #[must_use]
    pub fn span_in_slots(self) -> u64 {
        match self {
            ModelUnitRole::Data | ModelUnitRole::InodeLeaf | ModelUnitRole::InstanceTable => {
                DATA_UNIT_BYTES / SLOT_BYTES
            }
            ModelUnitRole::ExtentRoot
            | ModelUnitRole::InodeRoot
            | ModelUnitRole::AllocationTree
            | ModelUnitRole::AccountingTree
            | ModelUnitRole::MappingTree
            | ModelUnitRole::TreeTable => NODE_BYTES / SLOT_BYTES,
        }
    }
}

/// 一次发布重写哪些角色。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModelPublishKind {
    /// 第一个文件版本、覆盖写：四个文件角色，加四个固定点单元（D16（发布语义） 已定项 9：记账行每发布重写，连带记账树节点、分配记录、
    /// 映射条目与树表单元）。
    FileVersion,
    /// 写行：实例表单元（D18（块里携带什么信息） 已定项 11：每次可写挂载都写行）加四个固定点单元（D16（发布语义） 已定项 9）。
    RowsOnFileVersion,
    /// 树表不是 0 条时的空发布（暖机、抬 F）：四个固定点单元（D16（发布语义） 已定项 9）。
    EmptyOnFileVersion,
    /// 树表 0 条：零单元（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）。
    ZeroUnit,
}

impl ModelPublishKind {
    fn rewritten_roles(self) -> &'static [ModelUnitRole] {
        match self {
            ModelPublishKind::FileVersion => &[
                ModelUnitRole::Data,
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::RowsOnFileVersion => &[
                ModelUnitRole::InstanceTable,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::EmptyOnFileVersion => &[
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::ZeroUnit => &[],
        }
    }
}

/// 一版的文件：写出它那次发布的身份与内容。树表里 inode 树、extent 树的根指针由写出它的那次发布唯一定（指针带诞生代号），
/// 所以「两条根的这两棵树的根指针相同」⟺「两条根的文件是同一次发布写的」（D16（发布语义） 已定项 1「非空」从盘上怎么认）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelFileVersion {
    pub written_by: ModelRootKey,
    pub content: Rc<[u8]>,
}

/// 实例表的一行 (i, T, W) 与回退标志（D18（块里携带什么信息） 已定项 11；D23（journal 的角色与格式） 已定项 14）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelInstanceRow {
    pub instance: ModelInstanceGeneration,
    pub selected_root_txg: ModelCheckpointTxg,
    pub applied_transaction_high_water: u64,
    pub is_rollback: bool,
}

/// 一条已提交的根与它指着的那一版。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRoot {
    pub key: ModelRootKey,
    pub journal_counter: ModelJournalCounter,
    pub rollback_floor: ModelCheckpointTxg,
    /// 树表 0 条（第 0 代根、暖机根、零单元发布）时没有文件。
    pub file: Option<ModelFileVersion>,
    pub instance_table_rows: Rc<Vec<ModelInstanceRow>>,
    /// 这一版每个角色的单元是哪次发布写的：它的分配记录的分配代就是这个 txg（D3（空间分配） 已定项 3 / 7：value = 分配代；
    /// COW：单元只在分配它的那次发布里写）。mkfs 写的实例表与第 0 版树表记 txg 0。
    pub role_written_at: BTreeMap<ModelUnitRole, ModelCheckpointTxg>,
    /// 这一版之后分配记录条数的上界：mkfs 两个单元每盘各一条，沿这一版的来路每次发布每个重写的角色每盘至多加一条
    /// （D3（空间分配） 已定项 7：一条记一个单元、释放只改写不删）。回收、复用只会让真数比它小。
    pub allocation_records_upper_bound: u64,
    /// 同样口径的每盘已占槽数上界：沿来路每次发布写出的单元的槽数之和，释放与回收一概不扣。
    pub occupied_slots_upper_bound_per_device: u64,
}

impl ModelRoot {
    /// 这条根的用户可见树（inode 树、extent 树）的根指针是谁写的：树表 0 条时 None。
    fn user_visible_trees_written_by(&self) -> Option<ModelRootKey> {
        self.file.as_ref().map(|file| file.written_by)
    }
}

/// 池的几何：盘的身份与每块盘的字节数（与实现的 mkfs 参数由调用方各给一份）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelPoolGeometry {
    pub devices: Vec<ModelDeviceIdentity>,
    pub device_size_in_bytes: u64,
}

/// 根环里的一个槽：第 n 次发布写区域 `n mod R` 的槽 `(n div R) mod S`（D22（单元原子性怎么合成） 已定项 2 / 已定项 16）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ModelRingPosition {
    region: u64,
    slot_in_region: u64,
}

fn ring_position_of(checkpoint_txg: ModelCheckpointTxg) -> ModelRingPosition {
    ModelRingPosition {
        region: checkpoint_txg.0 % ROOT_RING_REGIONS,
        slot_in_region: (checkpoint_txg.0 / ROOT_RING_REGIONS) % ROOT_RING_SLOTS_PER_REGION,
    }
}

/// 一条根落在哪块盘：区域的设备归属第一版写死 0 / 1 / 0（D2（RAID 条带策略） 已定项 7；D16（发布语义） 已定项 8）。
fn device_holding_the_root_of(checkpoint_txg: ModelCheckpointTxg) -> ModelDeviceIdentity {
    let region = usize::try_from(checkpoint_txg.0 % ROOT_RING_REGIONS).expect("区域号小于 3");
    ModelDeviceIdentity(ROOT_RING_REGION_DEVICES[region])
}

/// 一个索引节点装几条定宽条目：(节点 − 头) ÷ 条目宽（D8（核心索引结构） 已定项 2 / 已定项 11）。
fn index_node_entry_capacity(key_width_in_bytes: u64, entry_width_in_bytes: u64) -> u64 {
    (NODE_BYTES - index_node_header_bytes(key_width_in_bytes)) / entry_width_in_bytes
}

/// 一个数据单元装得下多少字节的用户内容：32768 含头与预留位（D4（校验和位置） 已定项 5；D18（块里携带什么信息） 已定项 16），
/// 第一版没有扩展点声明值。
#[must_use]
pub fn data_unit_payload_capacity_in_bytes() -> u64 {
    DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET
}

/// 模型对拒绝理由的分法：按条款说的「为什么不能做」分，不按实现的错误成员分（胶水把成员映射过来）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelRefusalReason {
    /// 内容装不进一个数据单元（D4（校验和位置） 已定项 5）。
    ContentExceedsDataUnitPayload,
    /// 第一个文件版本写死 txg 3（`FIRST_TRANSACTION_TXG`，D16（发布语义） 已定项 8 的格式常量），只接得上 txg 2 那一版。
    FirstFileNotRightAfterTheWarmUp,
    /// 分配记录树第一版只有一个节点（容量墙，收口表第 39 行：模型答允许拒绝的区间）。
    AllocationRecordNodeWall,
    /// 记账树第一版只有一个节点。
    AccountingNodeWall,
    /// 单元区装不下（D28（挂载期承诺量） 已定项 1 的准入；模型答允许拒绝的区间）。
    UnitAreaWall,
    /// 实例表第一版只有一片（D18（块里携带什么信息） 已定项 11：一片 370 条含链指针；第二片第一版不做）。
    InstanceTableOnePageWall,
    /// 树表 0 条的一版上要写行：落点记在哪没有条款，第一版不支持（条款没写，照代码今天的读法）。
    RowsOnVersionWithoutFileUnsupported,
    /// 空池挂载的形状接不上写死 txg 3 的第一个文件版本（第一版不支持，照代码今天的读法）。
    FormattedPoolMountNotShapedLikeTheFirstTransaction,
    /// 回退目标不在根环里（D23（journal 的角色与格式） 已定项 14：候选集是根环里的根）。
    RollbackTargetNotInRing,
    /// 回退目标的 txg 低于 F_生效（D16（发布语义） 已定项 1「回退候选集」）。
    RollbackTargetBelowEffectiveFloor,
    /// 回退目标在被抛弃的时间线上（D23（journal 的角色与格式） 已定项 14：有行 (i, Ti, Wi) 且 T > Ti）。
    RollbackTargetOnAbandonedTimeline,
    /// 回退到树表 0 条的根：回退行要重写实例表，落点记在哪没有条款，第一版不支持（照代码今天的读法）。
    RollbackToVersionWithoutFileUnsupported,
    /// 要抬的 F 超过上限（D16（发布语义） 已定项 1「抬 F 的上限」）。
    FloorAboveCeiling,
    /// 现行版本的实例表还是 mkfs 写的那一片时抬 F：条款没写这个拒绝（设计问题，交主 agent）；照代码今天的读法当作第一版不支持的区间。
    RaiseWithFormatTimeInstanceTableUnsupported,
}

impl ModelRefusalReason {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            ModelRefusalReason::ContentExceedsDataUnitPayload => "内容装不进一个数据单元",
            ModelRefusalReason::FirstFileNotRightAfterTheWarmUp => {
                "第一个文件版本只接得上第二次暖机那一版"
            }
            ModelRefusalReason::AllocationRecordNodeWall => "分配记录树一个节点装不下",
            ModelRefusalReason::AccountingNodeWall => "记账树一个节点装不下",
            ModelRefusalReason::UnitAreaWall => "单元区装不下",
            ModelRefusalReason::InstanceTableOnePageWall => "实例表一片装不下",
            ModelRefusalReason::RowsOnVersionWithoutFileUnsupported => {
                "树表 0 条的一版上要写行（第一版不支持）"
            }
            ModelRefusalReason::FormattedPoolMountNotShapedLikeTheFirstTransaction => {
                "空池挂载的形状接不上第一个文件版本（第一版不支持）"
            }
            ModelRefusalReason::RollbackTargetNotInRing => "回退目标不在根环里",
            ModelRefusalReason::RollbackTargetBelowEffectiveFloor => "回退目标低于 F_生效",
            ModelRefusalReason::RollbackTargetOnAbandonedTimeline => "回退目标在被抛弃的时间线上",
            ModelRefusalReason::RollbackToVersionWithoutFileUnsupported => {
                "回退到树表 0 条的根（第一版不支持）"
            }
            ModelRefusalReason::FloorAboveCeiling => "要抬的 F 超过上限",
            ModelRefusalReason::RaiseWithFormatTimeInstanceTableUnsupported => {
                "实例表还是 mkfs 那一片时抬 F（条款没写，照代码今天的读法）"
            }
        }
    }

    /// 容量墙：模型只答允许拒绝的区间（按这一步计划的每次发布之后的上界判），不答必须拒。
    fn is_capacity_wall_with_an_interval(self) -> bool {
        match self {
            ModelRefusalReason::AllocationRecordNodeWall | ModelRefusalReason::UnitAreaWall => true,
            ModelRefusalReason::ContentExceedsDataUnitPayload
            | ModelRefusalReason::FirstFileNotRightAfterTheWarmUp
            | ModelRefusalReason::AccountingNodeWall
            | ModelRefusalReason::InstanceTableOnePageWall
            | ModelRefusalReason::RowsOnVersionWithoutFileUnsupported
            | ModelRefusalReason::FormattedPoolMountNotShapedLikeTheFirstTransaction
            | ModelRefusalReason::RollbackTargetNotInRing
            | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
            | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
            | ModelRefusalReason::RollbackToVersionWithoutFileUnsupported
            | ModelRefusalReason::FloorAboveCeiling
            | ModelRefusalReason::RaiseWithFormatTimeInstanceTableUnsupported => false,
        }
    }
}

/// 冷启动该读回什么（D23（journal 的角色与格式） 已定项 14：所选根 = 环里 (txg, 实例) 最大的那条；不建崩溃，没有要施加的记录）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelReadBack {
    NoFile {
        root: ModelRootKey,
    },
    FileRead {
        root: ModelRootKey,
        content: Rc<[u8]>,
    },
}

/// 模型问的是哪一种操作（报告用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelOperationKind {
    PublishFirstFile,
    PublishOverwrite,
    PublishWithoutUnits,
    MountWritable,
    MountRollback,
    RaiseRollbackFloor,
    ColdStartRecover,
}

/// 一步里计划的一次发布之后的两个上界（容量墙的区间按它判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PlannedPublishUpperBounds {
    allocation_records: u64,
    occupied_slots_per_device: u64,
}

/// 模型对一步操作的答案。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelAnswer {
    pub operation: ModelOperationKind,
    /// 条款要求拒绝的理由：非空就必须拒，理由是这里的任一条或区间允许的任一条墙。
    pub required_refusals: BTreeSet<ModelRefusalReason>,
    /// 不是容量墙、也允许拒绝的理由（照代码今天的读法划的「第一版不支持」区，条款没写）。
    pub permitted_refusals: BTreeSet<ModelRefusalReason>,
    /// 做成时写出的根，按次序。
    pub expected_roots: Vec<ModelRoot>,
    /// 做成时这次挂载取到的实例代号与写的行。
    pub expected_mount: Option<(ModelInstanceGeneration, Vec<ModelInstanceRow>)>,
    /// 冷启动该读回什么。
    pub expected_read_back: Option<ModelReadBack>,
    /// 抬 F 时模型算的上限（D16（发布语义） 已定项 1），拒与成都要与实现报的相等。
    pub rollback_floor_ceiling: Option<ModelCheckpointTxg>,
    planned_publish_upper_bounds: Vec<PlannedPublishUpperBounds>,
    /// 容量墙在第一次写之前一串判完（可写挂载、回退）还是逐次发布判（发布、抬 F）。
    walls_are_judged_before_the_first_write: bool,
    session_after_success: ModelSessionAfterSuccess,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModelSessionAfterSuccess {
    /// 会话照旧开着，现行版本换成最后写出的那条根。
    StaysOpen,
    /// 这次挂载开了一个新会话，实例是这个。
    OpenedAs(ModelInstanceGeneration),
    /// 会话关着（冷启动）。
    Closed,
}

/// 实现写出的一条根，胶水从实现交回的东西里取。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedRoot {
    pub key: ModelRootKey,
    pub journal_counter: ModelJournalCounter,
    pub rollback_floor: ModelCheckpointTxg,
    pub has_file: bool,
    /// 这一版每个单元（实现交回的那几个角色）的分配记录，每块盘各一条。树表 0 条的一版为空。
    pub unit_allocation_records: Vec<(ModelUnitRole, Vec<ObservedAllocationRecord>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedAllocationRecord {
    pub device: ModelDeviceIdentity,
    pub generation: ModelCheckpointTxg,
    pub is_released: bool,
}

/// 实现冷启动读回了什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedReadBack {
    NoFile {
        root: ModelRootKey,
    },
    FileRead {
        root: ModelRootKey,
        content: Vec<u8>,
    },
    Failed {
        what: String,
    },
}

/// 实现做成了什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedEffect {
    /// 一次或几次发布（发布、抬 F）；抬 F 连同实现报的上限。
    Publishes {
        roots: Vec<ObservedRoot>,
        reported_ceiling: Option<ModelCheckpointTxg>,
    },
    Mount {
        instance: ModelInstanceGeneration,
        rows_written: Vec<ModelInstanceRow>,
        roots: Vec<ObservedRoot>,
    },
    ColdStart {
        read_back: ObservedReadBack,
    },
}

/// 实现的错误成员映射到模型的理由：`Explained` 里是「这个成员说的是这几条之一」，`Unexplained` 是模型没有对应理由的成员（I/O、盘坏……）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedRefusalReason {
    Explained(Vec<ModelRefusalReason>),
    Unexplained,
}

/// 实现这一步的结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedOutcome {
    Succeeded(ObservedEffect),
    Refused {
        member: String,
        reason: ObservedRefusalReason,
        /// 拒之前已经做完的发布次数（抬 F 半路被拒时才可能非 0）。
        publishes_completed: usize,
        /// 录制流在这一步里有没有多出写或屏障。
        wrote_anything: bool,
        /// 实现报的上限（抬 F 被上限拒时）。
        reported_ceiling: Option<ModelCheckpointTxg>,
    },
}

/// 模型与实现对不上的是哪一格。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelDisagreementAspect {
    SessionState,
    RefusedWhenModelRequiresSuccess,
    SucceededWhenModelRequiresRefusal,
    RefusalReason,
    WroteBeforeRefusing,
    PublishCount,
    RootIdentity,
    JournalCounter,
    RollbackFloor,
    FilePresence,
    AllocationGeneration,
    InstanceGeneration,
    InstanceRows,
    RollbackFloorCeiling,
    ColdStartReadBack,
    NotModeled,
}

impl ModelDisagreementAspect {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            ModelDisagreementAspect::SessionState => "会话状态",
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess => "模型说该成、实现拒了",
            ModelDisagreementAspect::SucceededWhenModelRequiresRefusal => "模型说该拒、实现做成了",
            ModelDisagreementAspect::RefusalReason => "拒绝的理由",
            ModelDisagreementAspect::WroteBeforeRefusing => "拒绝之前写了盘",
            ModelDisagreementAspect::PublishCount => "写出的根的条数",
            ModelDisagreementAspect::RootIdentity => "根的 (txg, 实例)",
            ModelDisagreementAspect::JournalCounter => "jsn",
            ModelDisagreementAspect::RollbackFloor => "根带的 F",
            ModelDisagreementAspect::FilePresence => "这一版有没有文件",
            ModelDisagreementAspect::AllocationGeneration => "单元的分配代",
            ModelDisagreementAspect::InstanceGeneration => "取到的实例代号",
            ModelDisagreementAspect::InstanceRows => "写的实例表行",
            ModelDisagreementAspect::RollbackFloorCeiling => "抬 F 的上限",
            ModelDisagreementAspect::ColdStartReadBack => "冷启动读回",
            ModelDisagreementAspect::NotModeled => "模型答不了",
        }
    }
}

/// 模型与实现对不上：哪一格、模型答了什么、实现做了什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelDisagreement {
    pub aspect: ModelDisagreementAspect,
    pub model_answer: String,
    pub implementation_answer: String,
}

impl ModelDisagreement {
    fn new(
        aspect: ModelDisagreementAspect,
        model_answer: String,
        implementation_answer: String,
    ) -> Self {
        Self {
            aspect,
            model_answer,
            implementation_answer,
        }
    }
}

/// 一步判完之后的计数：模型怎么答的、比了多少格。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelJudgementCounts {
    pub required_refusals_matched: u64,
    pub permitted_refusals_taken: u64,
    pub successes_matched: u64,
    pub roots_compared: u64,
    pub allocation_records_compared: u64,
    pub cold_start_contents_compared: u64,
    pub ceilings_compared: u64,
    /// 回退做成、目标的 txg 正好等于 F_生效且 F_生效 > 0（B2 那一格跑到了）。
    pub rollbacks_accepted_at_the_effective_floor: u64,
}

impl ModelJudgementCounts {
    /// 把另一份逐项加进来。
    pub fn add(&mut self, other: &ModelJudgementCounts) {
        self.required_refusals_matched += other.required_refusals_matched;
        self.permitted_refusals_taken += other.permitted_refusals_taken;
        self.successes_matched += other.successes_matched;
        self.roots_compared += other.roots_compared;
        self.allocation_records_compared += other.allocation_records_compared;
        self.cold_start_contents_compared += other.cold_start_contents_compared;
        self.ceilings_compared += other.ceilings_compared;
        self.rollbacks_accepted_at_the_effective_floor +=
            other.rollbacks_accepted_at_the_effective_floor;
    }
}

/// 一个可写会话：实例与接下来的发布要接在后面的那一版。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ModelSession {
    instance: ModelInstanceGeneration,
    current: ModelRoot,
}

/// 理想模型的状态。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdealModel {
    geometry: ModelPoolGeometry,
    ring: BTreeMap<ModelRingPosition, ModelRoot>,
    /// 取过的最大实例代号（超级块里的号；取号先于任何带新号的写，D23（journal 的角色与格式） 已定项 16）。
    highest_acquired_instance: ModelInstanceGeneration,
    /// 发布过的最大 txg 与 jsn：不建崩溃，环里全部根与全部记录里的最大值就是它们（D23（journal 的角色与格式） 已定项 14 第 3 条）。
    highest_published_txg: ModelCheckpointTxg,
    highest_journal_counter: ModelJournalCounter,
    session: Option<ModelSession>,
}

impl IdealModel {
    /// mkfs 之后：根环里只有第 0 代根（实例 0、txg 0、F 0、树表 0 条、实例表没有行），mkfs 写了实例表与第 0 版树表两个单元，
    /// 没有可写会话。
    #[must_use]
    pub fn after_make_filesystem(geometry: ModelPoolGeometry) -> Self {
        let device_count = u64::try_from(geometry.devices.len()).expect("盘数");
        let genesis = ModelRoot {
            key: ModelRootKey {
                checkpoint_txg: MAKE_FILESYSTEM_TXG,
                instance: MAKE_FILESYSTEM_INSTANCE,
            },
            journal_counter: ModelJournalCounter(0),
            rollback_floor: ModelCheckpointTxg(0),
            file: None,
            instance_table_rows: Rc::new(Vec::new()),
            role_written_at: BTreeMap::from([
                (ModelUnitRole::InstanceTable, MAKE_FILESYSTEM_TXG),
                (ModelUnitRole::TreeTable, MAKE_FILESYSTEM_TXG),
            ]),
            allocation_records_upper_bound: 2 * device_count,
            occupied_slots_upper_bound_per_device: ModelUnitRole::InstanceTable.span_in_slots()
                + ModelUnitRole::TreeTable.span_in_slots(),
        };
        let mut ring = BTreeMap::new();
        ring.insert(ring_position_of(MAKE_FILESYSTEM_TXG), genesis);
        Self {
            geometry,
            ring,
            highest_acquired_instance: MAKE_FILESYSTEM_INSTANCE,
            highest_published_txg: MAKE_FILESYSTEM_TXG,
            highest_journal_counter: ModelJournalCounter(0),
            session: None,
        }
    }

    /// mkfs 同一个进程里取号、暖机（D16（发布语义） 已定项 8：连推空发布到本实例的根覆盖每块盘；树表 0 条 ⇒ 零单元），会话开着、
    /// 接下来发第一个文件版本。与可写挂载只做过 mkfs 的池同一个形状，只是不经恢复。
    ///
    /// # Panics
    /// 模型不在 mkfs 刚做完的状态（这一步只在起点调）。
    pub fn acquire_and_warm_up_in_the_make_filesystem_process(&mut self) {
        assert!(
            self.session.is_none() && self.highest_acquired_instance == MAKE_FILESYSTEM_INSTANCE,
            "只在 mkfs 刚做完、还没取过号时调"
        );
        let answer = self.answer_establishing_an_instance(
            ModelOperationKind::MountWritable,
            &self.newest_root().clone(),
            None,
        );
        assert!(
            answer.required_refusals.is_empty(),
            "刚 mkfs 的池取号暖机：条款不要求拒（{:?}）",
            answer.required_refusals
        );
        self.advance_by_success(&answer);
    }

    /// 关掉这个进程的可写会话（可写挂载、回退、冷启动之前实现都先关）。
    pub fn close_session(&mut self) {
        self.session = None;
    }

    #[must_use]
    pub fn has_writable_session(&self) -> bool {
        self.session.is_some()
    }

    fn newest_root(&self) -> &ModelRoot {
        self.ring
            .values()
            .max_by_key(|root| root.key)
            .expect("根环里至少有 mkfs 的第 0 代根：不建崩溃，没有读不出的根槽")
    }

    fn root_with_key(&self, key: ModelRootKey) -> Option<&ModelRoot> {
        self.ring.values().find(|root| root.key == key)
    }

    /// F_生效 = 各幸存盘所带 F 最大值的最小值；一块盘上一条根都没有就不算它，一条根都没有时 0。
    /// 预想，跟收口表第 ② 行（「生效」那一行 2026-09-17 打回重议，定案之前照字面；被抛弃时间线上的根带的 F 也算，照字面「持久根」）。
    #[must_use]
    pub fn effective_rollback_floor(&self) -> ModelCheckpointTxg {
        let mut highest_floor_per_device: BTreeMap<ModelDeviceIdentity, ModelCheckpointTxg> =
            BTreeMap::new();
        for root in self.ring.values() {
            let highest = highest_floor_per_device
                .entry(device_holding_the_root_of(root.key.checkpoint_txg))
                .or_insert(root.rollback_floor);
            *highest = (*highest).max(root.rollback_floor);
        }
        highest_floor_per_device
            .values()
            .copied()
            .min()
            .unwrap_or(ModelCheckpointTxg(0))
    }

    /// 按实例表判是不是被抛弃的：表里有它那个实例的行 (i, Ti, Wi) 且 txg > Ti（D23（journal 的角色与格式） 已定项 14）。
    /// 用哪一张表：最新那条根指着的（恢复与回退择的就是它；不建崩溃时它也是会话的现行版本）。
    fn is_abandoned_by(root: &ModelRoot, table: &[ModelInstanceRow]) -> bool {
        table.iter().any(|row| {
            row.instance == root.key.instance && root.key.checkpoint_txg > row.selected_root_txg
        })
    }

    /// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的有效根, 第 4 新的非空有效根)；非空有效根不足 4 个时取最旧的有效根。
    /// 有效 = 按最新根的实例表不被抛弃 ∧ txg ≥ 现行的 F（「非空」从盘上怎么认，2026-09-17 用户定案）；现行的 F 取 F_生效（不建崩溃时与
    /// 现行根带的相等）。非空 = inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排）的不同，最旧的有效根与「没有这两棵树」比。
    /// 一块盘上没有有效根时不算它（条款没写这一格；不建崩溃时抬 F 与暖机都推到每块盘都有，走不到）。预想，跟收口表第 ② 行
    /// （候选集下界用哪个 F、被抛弃根带的 F 算不算都在那一行里重议）。
    #[must_use]
    pub fn rollback_floor_ceiling(&self) -> Option<ModelCheckpointTxg> {
        let current_floor = self.effective_rollback_floor();
        let newest_table = Rc::clone(&self.newest_root().instance_table_rows);
        let mut valid: Vec<&ModelRoot> = self
            .ring
            .values()
            .filter(|root| {
                !Self::is_abandoned_by(root, &newest_table)
                    && root.key.checkpoint_txg >= current_floor
            })
            .collect();
        valid.sort_by_key(|root| root.key);
        let mut newest_valid_per_device: BTreeMap<ModelDeviceIdentity, ModelCheckpointTxg> =
            BTreeMap::new();
        for root in &valid {
            let newest = newest_valid_per_device
                .entry(device_holding_the_root_of(root.key.checkpoint_txg))
                .or_insert(root.key.checkpoint_txg);
            *newest = (*newest).max(root.key.checkpoint_txg);
        }
        let newest_on_every_device = newest_valid_per_device.values().copied().min()?;
        let mut non_empty_txgs: Vec<ModelCheckpointTxg> = Vec::new();
        let mut previous_trees_written_by: Option<ModelRootKey> = None;
        for root in &valid {
            let trees_written_by = root.user_visible_trees_written_by();
            if trees_written_by != previous_trees_written_by {
                non_empty_txgs.push(root.key.checkpoint_txg);
            }
            previous_trees_written_by = trees_written_by;
        }
        non_empty_txgs.sort_unstable_by(|left, right| right.cmp(left));
        let fourth_newest_non_empty = match non_empty_txgs.get(3) {
            Some(fourth) => *fourth,
            None => valid.first()?.key.checkpoint_txg,
        };
        Some(newest_on_every_device.min(fourth_newest_non_empty))
    }

    /// 分配记录树一个节点装几条（D3（空间分配） 已定项 11：key 10、条目 20）。
    fn allocation_record_node_capacity() -> u64 {
        index_node_entry_capacity(ALLOCATION_RECORD_KEY_BYTES, ALLOCATION_RECORD_BYTES)
    }

    /// 一次写记账行的发布要几行、一个节点装几行（D5（快照 / 空间记账机制） 已定项 8）。
    fn accounting_rows_exceed_one_node(&self) -> bool {
        let device_count = u64::try_from(self.geometry.devices.len()).expect("盘数");
        POOL_WIDE_ACCOUNTING_ROWS + ACCOUNTING_ROWS_PER_DEVICE * device_count
            > index_node_entry_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_ENTRY_BYTES)
    }

    /// 单元区每块盘的槽数。
    fn unit_area_slots_per_device(&self) -> u64 {
        (self.geometry.device_size_in_bytes / SLOT_BYTES).saturating_sub(UNIT_AREA_START_SLOT)
    }

    /// 接在 `previous` 后面的一次发布写出的根（D16（发布语义） 已定项 6：txg + 1；jsn 接着全池计数器）。
    #[allow(
        clippy::too_many_arguments,
        reason = "一条根的每个身份字段各是一个参数，收成结构体只会多一层没人验的名字"
    )]
    fn next_root(
        &self,
        previous: &ModelRoot,
        instance: ModelInstanceGeneration,
        checkpoint_txg: ModelCheckpointTxg,
        journal_counter: ModelJournalCounter,
        kind: ModelPublishKind,
        rollback_floor: ModelCheckpointTxg,
        new_file_content: Option<&[u8]>,
        instance_table_rows: Rc<Vec<ModelInstanceRow>>,
    ) -> ModelRoot {
        let key = ModelRootKey {
            checkpoint_txg,
            instance,
        };
        let device_count = u64::try_from(self.geometry.devices.len()).expect("盘数");
        let mut role_written_at = previous.role_written_at.clone();
        let mut slots_written = 0;
        let rewritten = kind.rewritten_roles();
        for role in rewritten {
            role_written_at.insert(*role, checkpoint_txg);
            slots_written += role.span_in_slots();
        }
        let file = match new_file_content {
            Some(content) => Some(ModelFileVersion {
                written_by: key,
                content: Rc::from(content),
            }),
            None => previous.file.clone(),
        };
        ModelRoot {
            key,
            journal_counter,
            rollback_floor,
            file,
            instance_table_rows,
            role_written_at,
            allocation_records_upper_bound: previous.allocation_records_upper_bound
                + u64::try_from(rewritten.len()).expect("至多九个角色") * device_count,
            occupied_slots_upper_bound_per_device: previous.occupied_slots_upper_bound_per_device
                + slots_written,
        }
    }

    fn upper_bounds_of(roots: &[ModelRoot]) -> Vec<PlannedPublishUpperBounds> {
        roots
            .iter()
            .map(|root| PlannedPublishUpperBounds {
                allocation_records: root.allocation_records_upper_bound,
                occupied_slots_per_device: root.occupied_slots_upper_bound_per_device,
            })
            .collect()
    }

    fn open_session(&self) -> Result<&ModelSession, ModelDisagreement> {
        self.session.as_ref().ok_or_else(|| {
            ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                "模型里这个进程没有可写会话".to_string(),
                "实现的会话开着（执行器调了要会话的入口）".to_string(),
            )
        })
    }

    /// 发布第一个文件版本（`transaction::publish_first_file`）。
    ///
    /// # Errors
    /// 模型里没有可写会话：会话状态与实现对不上。
    pub fn answer_publish_first_file(
        &self,
        content: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let mut required_refusals = BTreeSet::new();
        // 第一个文件版本写死 txg 3 与 jsn 3（FIRST_TRANSACTION_TXG；第一个事务 jsn 与 txg 同号），只接得上 txg 2、jsn 2 的那一版。
        let follows_the_warm_up = current.key.checkpoint_txg.0 + 1 == FIRST_TRANSACTION_TXG
            && current.journal_counter.0 + 1 == FIRST_TRANSACTION_TXG;
        if !follows_the_warm_up {
            required_refusals.insert(ModelRefusalReason::FirstFileNotRightAfterTheWarmUp);
        }
        if u64::try_from(content.len()).expect("内容长度") > data_unit_payload_capacity_in_bytes()
        {
            required_refusals.insert(ModelRefusalReason::ContentExceedsDataUnitPayload);
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(FIRST_TRANSACTION_TXG),
            ModelJournalCounter(FIRST_TRANSACTION_TXG),
            ModelPublishKind::FileVersion,
            current.rollback_floor,
            Some(content),
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishFirstFile,
            required_refusals,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 覆盖写（`transaction::publish_overwrite`）：接在现行的带文件的一版后面。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本没有文件（执行器只在带文件时调）：会话状态与实现对不上。
    pub fn answer_publish_overwrite(
        &self,
        content: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if current.file.is_none() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!("模型里现行版本 {:?} 没有文件", current.key),
                "实现的现行版本带文件（执行器调了覆盖写）".to_string(),
            ));
        }
        let mut required_refusals = BTreeSet::new();
        if u64::try_from(content.len()).expect("内容长度") > data_unit_payload_capacity_in_bytes()
        {
            required_refusals.insert(ModelRefusalReason::ContentExceedsDataUnitPayload);
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::FileVersion,
            current.rollback_floor,
            Some(content),
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishOverwrite,
            required_refusals,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 零单元发布（`transaction::publish_without_units`）：接在现行的树表 0 条的一版后面。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本带文件（执行器只在树表 0 条时调）：会话状态与实现对不上。
    pub fn answer_publish_without_units(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if current.file.is_some() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!("模型里现行版本 {:?} 带文件", current.key),
                "实现的现行版本树表 0 条（执行器调了零单元发布）".to_string(),
            ));
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::ZeroUnit,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishWithoutUnits,
            BTreeSet::new(),
            BTreeSet::new(),
            vec![root],
        ))
    }

    fn answer_for_publishes(
        &self,
        operation: ModelOperationKind,
        mut required_refusals: BTreeSet<ModelRefusalReason>,
        permitted_refusals: BTreeSet<ModelRefusalReason>,
        roots: Vec<ModelRoot>,
    ) -> ModelAnswer {
        // 这一步里有一次发布重写了记账树（零单元发布不写记账行，别的都写，D16（发布语义） 已定项 9）。
        let writes_accounting_rows = roots.iter().any(|root| {
            root.role_written_at.get(&ModelUnitRole::AccountingTree)
                == Some(&root.key.checkpoint_txg)
        });
        if writes_accounting_rows && self.accounting_rows_exceed_one_node() {
            required_refusals.insert(ModelRefusalReason::AccountingNodeWall);
        }
        ModelAnswer {
            operation,
            required_refusals,
            permitted_refusals,
            planned_publish_upper_bounds: Self::upper_bounds_of(&roots),
            expected_roots: roots,
            expected_mount: None,
            expected_read_back: None,
            rollback_floor_ceiling: None,
            walls_are_judged_before_the_first_write: false,
            session_after_success: ModelSessionAfterSuccess::StaysOpen,
        }
    }

    /// 可写挂载（`mount::mount_writable`）：所选根 = 环里最新的那条（不建崩溃，没有要施加的记录）。调之前先 `close_session`。
    #[must_use]
    pub fn answer_mount_writable(&self) -> ModelAnswer {
        let chosen = self.newest_root().clone();
        let previous_row = ModelInstanceRow {
            instance: chosen.key.instance,
            selected_root_txg: chosen.key.checkpoint_txg,
            // W = 这次恢复施加的记录里最大的事务号（D23（journal 的角色与格式） 已定项 14 第 4 条）：不建崩溃，一条都不施加。
            applied_transaction_high_water: 0,
            is_rollback: false,
        };
        self.answer_establishing_an_instance(
            ModelOperationKind::MountWritable,
            &chosen,
            Some(previous_row),
        )
    }

    /// 管理员回退（`mount::mount_rollback`，D23（journal 的角色与格式） 已定项 14 的显式例外）。调之前先 `close_session`。
    #[must_use]
    pub fn answer_mount_rollback(&self, target: ModelRootKey) -> ModelAnswer {
        let refused = |reasons: BTreeSet<ModelRefusalReason>| ModelAnswer {
            operation: ModelOperationKind::MountRollback,
            required_refusals: reasons,
            permitted_refusals: BTreeSet::new(),
            expected_roots: Vec::new(),
            expected_mount: None,
            expected_read_back: None,
            rollback_floor_ceiling: None,
            planned_publish_upper_bounds: Vec::new(),
            walls_are_judged_before_the_first_write: true,
            session_after_success: ModelSessionAfterSuccess::Closed,
        };
        let Some(target_root) = self.root_with_key(target) else {
            return refused(BTreeSet::from([
                ModelRefusalReason::RollbackTargetNotInRing,
            ]));
        };
        let mut reasons = BTreeSet::new();
        if target.checkpoint_txg < self.effective_rollback_floor() {
            reasons.insert(ModelRefusalReason::RollbackTargetBelowEffectiveFloor);
        }
        if Self::is_abandoned_by(target_root, &self.newest_root().instance_table_rows) {
            reasons.insert(ModelRefusalReason::RollbackTargetOnAbandonedTimeline);
        }
        if target_root.file.is_none() {
            reasons.insert(ModelRefusalReason::RollbackToVersionWithoutFileUnsupported);
        }
        if !reasons.is_empty() {
            return refused(reasons);
        }
        let previous_row = ModelInstanceRow {
            instance: target.instance,
            selected_root_txg: target.checkpoint_txg,
            applied_transaction_high_water: 0,
            is_rollback: true,
        };
        self.answer_establishing_an_instance(
            ModelOperationKind::MountRollback,
            &target_root.clone(),
            Some(previous_row),
        )
    }

    /// 可写挂载与回退共用的后半段：取号、写行那次发布、暖机到本实例的根覆盖每块盘（D16（发布语义） 已定项 8 甲′，至多 R 次）。
    /// `previous_row` 为 None 只在 mkfs 同一个进程里（不写行）。
    fn answer_establishing_an_instance(
        &self,
        operation: ModelOperationKind,
        base: &ModelRoot,
        previous_row: Option<ModelInstanceRow>,
    ) -> ModelAnswer {
        let instance = ModelInstanceGeneration(self.highest_acquired_instance.0 + 1);
        // 行：给 [max(上一个实例, 1), 新实例) 里每个实例一行——上一个实例（或被退回的实例）那一行，中间实例 (i, 0, 0)；实例 0 不写行
        // （D18（块里携带什么信息） 已定项 11；D23（journal 的角色与格式） 已定项 14）。
        let rows_to_write: Vec<ModelInstanceRow> = match previous_row {
            None => Vec::new(),
            Some(previous_row) => (previous_row.instance.0.max(1)..instance.0)
                .map(|row_instance| {
                    if row_instance == previous_row.instance.0 {
                        previous_row
                    } else {
                        ModelInstanceRow {
                            instance: ModelInstanceGeneration(row_instance),
                            selected_root_txg: ModelCheckpointTxg(0),
                            applied_transaction_high_water: 0,
                            is_rollback: false,
                        }
                    }
                })
                .collect(),
        };
        // 新实例的第一次发布的 txg 与 jsn：环里全部根与全部记录里的最大值 + 1（D23（journal 的角色与格式） 已定项 14 第 3 条）。
        // 回退的 jsn 接在环里最大的 jsn 之后（C340 取 P2）：预想，跟收口表第 ① 行。
        let first_txg = ModelCheckpointTxg(self.highest_published_txg.0 + 1);
        let first_journal_counter = ModelJournalCounter(self.highest_journal_counter.0 + 1);
        // 新实例的根带的 F = 恢复后生效的 F（D16（发布语义） 已定项 1「生效」）：预想，跟收口表第 ② 行。
        let rollback_floor = self.effective_rollback_floor();
        let mut required_refusals = BTreeSet::new();
        let has_file = base.file.is_some();
        if !has_file && !rows_to_write.is_empty() {
            required_refusals.insert(ModelRefusalReason::RowsOnVersionWithoutFileUnsupported);
        }
        if !has_file && rows_to_write.is_empty() {
            let shaped_like_the_first_transaction = first_txg.0 == 1
                && first_journal_counter.0 == 1
                && device_holding_the_root_of(ModelCheckpointTxg(1))
                    != device_holding_the_root_of(ModelCheckpointTxg(2));
            if !shaped_like_the_first_transaction {
                required_refusals
                    .insert(ModelRefusalReason::FormattedPoolMountNotShapedLikeTheFirstTransaction);
            }
        }
        if has_file {
            let rows_after =
                u64::try_from(base.instance_table_rows.len() + rows_to_write.len()).expect("行数");
            // 一片 370 条，链指针记录恒为一片的最后一条（D18（块里携带什么信息） 已定项 11）；第二片第一版不做。
            if rows_after + 1 > INSTANCE_TABLE_PAGE_RECORDS {
                required_refusals.insert(ModelRefusalReason::InstanceTableOnePageWall);
            }
        }
        let table_after: Rc<Vec<ModelInstanceRow>> = if has_file {
            let mut rows = base.instance_table_rows.as_ref().clone();
            rows.extend_from_slice(&rows_to_write);
            Rc::new(rows)
        } else {
            Rc::clone(&base.instance_table_rows)
        };
        let (row_kind, warm_up_kind) = if has_file {
            (
                ModelPublishKind::RowsOnFileVersion,
                ModelPublishKind::EmptyOnFileVersion,
            )
        } else {
            (ModelPublishKind::ZeroUnit, ModelPublishKind::ZeroUnit)
        };
        let mut roots = vec![self.next_root(
            base,
            instance,
            first_txg,
            first_journal_counter,
            row_kind,
            rollback_floor,
            None,
            Rc::clone(&table_after),
        )];
        let mut covered: BTreeSet<ModelDeviceIdentity> =
            BTreeSet::from([device_holding_the_root_of(first_txg)]);
        let mut warm_up_publishes: u64 = 0;
        while self
            .geometry
            .devices
            .iter()
            .any(|device| !covered.contains(device))
            && warm_up_publishes < ROOT_RING_REGIONS
        {
            let previous = roots.last().expect("至少有写行那一次").clone();
            let next_txg = ModelCheckpointTxg(previous.key.checkpoint_txg.0 + 1);
            covered.insert(device_holding_the_root_of(next_txg));
            roots.push(self.next_root(
                &previous,
                instance,
                next_txg,
                ModelJournalCounter(previous.journal_counter.0 + 1),
                warm_up_kind,
                rollback_floor,
                None,
                Rc::clone(&table_after),
            ));
            warm_up_publishes += 1;
        }
        let mut answer =
            self.answer_for_publishes(operation, required_refusals, BTreeSet::new(), roots);
        answer.walls_are_judged_before_the_first_write = true;
        answer.expected_mount = Some((instance, rows_to_write));
        answer.session_after_success = ModelSessionAfterSuccess::OpenedAs(instance);
        answer
    }

    /// 抬 F（`mount::raise_rollback_floor`）到 `new_floor`：推空发布直到每块盘上都有带新 F 的根（D16（发布语义） 已定项 1「生效」），
    /// 至多 R 次。
    ///
    /// # Errors
    /// 模型里没有可写会话或现行版本没有文件（执行器只在带文件时调）；`new_floor` 低于现行的 F（条款没写往下抬，生成器只取现行 F 及以上，
    /// 走到这里说明模型与实现的 F 已经对不上）。
    pub fn answer_raise_rollback_floor(
        &self,
        new_floor: ModelCheckpointTxg,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if current.file.is_none() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!("模型里现行版本 {:?} 没有文件", current.key),
                "实现的现行版本带文件（执行器调了抬 F）".to_string(),
            ));
        }
        if new_floor < current.rollback_floor {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!(
                    "往下抬 F（{} < 现行 {}）条款没写，模型不答",
                    new_floor.0, current.rollback_floor.0
                ),
                "执行器按实现的现行 F 取了目标".to_string(),
            ));
        }
        let ceiling = self.rollback_floor_ceiling();
        let mut required_refusals = BTreeSet::new();
        let mut permitted_refusals = BTreeSet::new();
        match ceiling {
            Some(ceiling) if new_floor > ceiling => {
                required_refusals.insert(ModelRefusalReason::FloorAboveCeiling);
            }
            Some(_) => {}
            None => {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::NotModeled,
                    "模型里一条有效根都没有，上限算不出".to_string(),
                    "实现调了抬 F".to_string(),
                ))
            }
        }
        // 现行版本的实例表还是 mkfs 那一片（这一路来路上还没有写行过）：实现拒（它从现行版本的单元里读表，没有这一单元），条款没写
        // 这个拒绝——mkfs 的表没有行、候选集照样判得出。设计问题交主 agent；照代码今天的读法划进允许拒绝。
        if current.role_written_at.get(&ModelUnitRole::InstanceTable) == Some(&MAKE_FILESYSTEM_TXG)
        {
            permitted_refusals
                .insert(ModelRefusalReason::RaiseWithFormatTimeInstanceTableUnsupported);
        }
        let mut roots: Vec<ModelRoot> = Vec::new();
        let mut covered: BTreeSet<ModelDeviceIdentity> = BTreeSet::new();
        while self
            .geometry
            .devices
            .iter()
            .any(|device| !covered.contains(device))
            && u64::try_from(roots.len()).expect("次数") < ROOT_RING_REGIONS
        {
            let previous = roots.last().unwrap_or(current).clone();
            let next_txg = ModelCheckpointTxg(previous.key.checkpoint_txg.0 + 1);
            covered.insert(device_holding_the_root_of(next_txg));
            let journal_counter = if roots.is_empty() {
                ModelJournalCounter(self.highest_journal_counter.0 + 1)
            } else {
                ModelJournalCounter(previous.journal_counter.0 + 1)
            };
            roots.push(self.next_root(
                &previous,
                session.instance,
                next_txg,
                journal_counter,
                ModelPublishKind::EmptyOnFileVersion,
                new_floor,
                None,
                Rc::clone(&previous.instance_table_rows),
            ));
        }
        let mut answer = self.answer_for_publishes(
            ModelOperationKind::RaiseRollbackFloor,
            required_refusals,
            permitted_refusals,
            roots,
        );
        answer.rollback_floor_ceiling = ceiling;
        Ok(answer)
    }

    /// 冷启动读回（`recovery::recover`，看 journal）：所选根 = 环里 (txg, 实例) 最大的那条，读回它那一版的文件。调之前先 `close_session`。
    #[must_use]
    pub fn answer_cold_start_recover(&self) -> ModelAnswer {
        let newest = self.newest_root();
        let read_back = match &newest.file {
            Some(file) => ModelReadBack::FileRead {
                root: newest.key,
                content: Rc::clone(&file.content),
            },
            None => ModelReadBack::NoFile { root: newest.key },
        };
        ModelAnswer {
            operation: ModelOperationKind::ColdStartRecover,
            required_refusals: BTreeSet::new(),
            permitted_refusals: BTreeSet::new(),
            expected_roots: Vec::new(),
            expected_mount: None,
            expected_read_back: Some(read_back),
            rollback_floor_ceiling: None,
            planned_publish_upper_bounds: Vec::new(),
            walls_are_judged_before_the_first_write: true,
            session_after_success: ModelSessionAfterSuccess::Closed,
        }
    }

    /// 容量墙的区间（收口表第 39 行那种：条款把答案留给实现取上界，模型答允许拒绝的区间）：
    /// - 分配记录树（预想，跟收口表第 39 行）：下端（必须拒）是真条数装不下一个节点——模型不知道真条数，这一端不判（装不下还去写会 panic，
    ///   由第 1 件判）；上端（允许拒）是这一步计划的那次发布之后、沿来路每次发布每个角色每盘都新加一条的上界超过一个节点：一条记一个单元、
    ///   释放只改写不删（D3（空间分配） 已定项 7），条目 20 字节、key 10（D3（空间分配） 已定项 11），节点 16 KiB 减头（D8（核心索引结构）
    ///   已定项 11）⇒ 812 条；「每个角色每盘新增一条」是 2026-09-18 用户定保留的上界准入。模型的上界沿来路累加、不看分配器此刻的条数，
    ///   与实现「此刻条数 + 这次新增」不是同一个算法，只会更宽。
    /// - 单元区：下端同样不判；上端是占槽上界 × 一个聚簇段的槽数超过单元区（每个落点最坏独占一段：已分配 + defer ≤ 2 × 上界，
    ///   保留池 10 + 7 c_max 与切换预留（D16（发布语义） 已定项 1；D28（挂载期承诺量） 已定项 3）在 64 倍里）。预想：D28 已定项 1 的准入式子
    ///   第一版没实现，各项没有现值。
    fn capacity_wall_is_permitted(
        &self,
        reason: ModelRefusalReason,
        answer: &ModelAnswer,
        publishes_completed: usize,
    ) -> bool {
        let exceeds = |bounds: &PlannedPublishUpperBounds| match reason {
            ModelRefusalReason::AllocationRecordNodeWall => {
                bounds.allocation_records > Self::allocation_record_node_capacity()
            }
            ModelRefusalReason::UnitAreaWall => {
                bounds.occupied_slots_per_device * CLUSTER_SEGMENT_SLOTS
                    > self.unit_area_slots_per_device()
            }
            ModelRefusalReason::ContentExceedsDataUnitPayload
            | ModelRefusalReason::FirstFileNotRightAfterTheWarmUp
            | ModelRefusalReason::AccountingNodeWall
            | ModelRefusalReason::InstanceTableOnePageWall
            | ModelRefusalReason::RowsOnVersionWithoutFileUnsupported
            | ModelRefusalReason::FormattedPoolMountNotShapedLikeTheFirstTransaction
            | ModelRefusalReason::RollbackTargetNotInRing
            | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
            | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
            | ModelRefusalReason::RollbackToVersionWithoutFileUnsupported
            | ModelRefusalReason::FloorAboveCeiling
            | ModelRefusalReason::RaiseWithFormatTimeInstanceTableUnsupported => false,
        };
        if answer.walls_are_judged_before_the_first_write {
            answer.planned_publish_upper_bounds.iter().any(exceeds)
        } else {
            answer
                .planned_publish_upper_bounds
                .get(publishes_completed)
                .is_some_and(exceeds)
        }
    }

    /// 拿实现的结局与模型的答案比；对得上就按模型自己算的结果往前走（不按实现交回的），对不上交回哪一格。
    ///
    /// # Errors
    /// 对不上的第一格。
    pub fn judge_and_advance(
        &mut self,
        answer: &ModelAnswer,
        observed: &ObservedOutcome,
    ) -> Result<ModelJudgementCounts, ModelDisagreement> {
        let mut counts = ModelJudgementCounts::default();
        match observed {
            ObservedOutcome::Refused {
                member,
                reason,
                publishes_completed,
                wrote_anything,
                reported_ceiling,
            } => {
                self.judge_reported_ceiling(answer, *reported_ceiling, &mut counts)?;
                let explained_reasons: &[ModelRefusalReason] = match reason {
                    ObservedRefusalReason::Explained(reasons) => reasons,
                    ObservedRefusalReason::Unexplained => &[],
                };
                let accepted = explained_reasons.iter().copied().find(|candidate| {
                    answer.required_refusals.contains(candidate)
                        || answer.permitted_refusals.contains(candidate)
                        || (candidate.is_capacity_wall_with_an_interval()
                            && self.capacity_wall_is_permitted(
                                *candidate,
                                answer,
                                *publishes_completed,
                            ))
                });
                let Some(accepted_reason) = accepted else {
                    let aspect = if answer.required_refusals.is_empty() {
                        ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
                    } else {
                        ModelDisagreementAspect::RefusalReason
                    };
                    return Err(ModelDisagreement::new(
                        aspect,
                        describe_answer(answer),
                        format!("拒了：{member}"),
                    ));
                };
                let partial_publishes_allowed = !answer.walls_are_judged_before_the_first_write
                    && accepted_reason.is_capacity_wall_with_an_interval();
                if *publishes_completed > 0 && !partial_publishes_allowed {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::PublishCount,
                        format!("{}：拒之前一次发布都不做", accepted_reason.name()),
                        format!("拒之前做了 {publishes_completed} 次发布（{member}）"),
                    ));
                }
                if *wrote_anything && *publishes_completed == 0 {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::WroteBeforeRefusing,
                        format!("{}：拒绝之前盘上一个字节都不动", accepted_reason.name()),
                        format!("拒了（{member}），录制流里多了写或屏障"),
                    ));
                }
                let Some(completed) = answer.expected_roots.get(..*publishes_completed) else {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::PublishCount,
                        format!("至多 {} 次发布", answer.expected_roots.len()),
                        format!("拒之前做了 {publishes_completed} 次发布（{member}）"),
                    ));
                };
                if answer.required_refusals.contains(&accepted_reason) {
                    counts.required_refusals_matched += 1;
                } else {
                    counts.permitted_refusals_taken += 1;
                }
                for root in completed {
                    self.write_root(root.clone());
                }
                if let (Some(session), Some(last)) = (self.session.as_mut(), completed.last()) {
                    session.current = last.clone();
                }
                Ok(counts)
            }
            ObservedOutcome::Succeeded(effect) => {
                if !answer.required_refusals.is_empty() {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::SucceededWhenModelRequiresRefusal,
                        describe_answer(answer),
                        "做成了".to_string(),
                    ));
                }
                self.judge_effect(answer, effect, &mut counts)?;
                counts.successes_matched += 1;
                self.advance_by_success(answer);
                Ok(counts)
            }
        }
    }

    fn judge_reported_ceiling(
        &self,
        answer: &ModelAnswer,
        reported_ceiling: Option<ModelCheckpointTxg>,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        let Some(reported) = reported_ceiling else {
            return Ok(());
        };
        counts.ceilings_compared += 1;
        if answer.rollback_floor_ceiling == Some(reported) {
            Ok(())
        } else {
            Err(ModelDisagreement::new(
                ModelDisagreementAspect::RollbackFloorCeiling,
                format!(
                    "上限 {:?}（D16（发布语义） 已定项 1）",
                    answer.rollback_floor_ceiling.map(|ceiling| ceiling.0)
                ),
                format!("实现报上限 {}", reported.0),
            ))
        }
    }

    fn judge_effect(
        &self,
        answer: &ModelAnswer,
        effect: &ObservedEffect,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        match effect {
            ObservedEffect::Publishes {
                roots,
                reported_ceiling,
            } => {
                self.judge_reported_ceiling(answer, *reported_ceiling, counts)?;
                if answer.operation == ModelOperationKind::RaiseRollbackFloor
                    && reported_ceiling.is_none()
                {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::RollbackFloorCeiling,
                        "抬 F 做成要报上限".to_string(),
                        "胶水没交上限".to_string(),
                    ));
                }
                self.judge_roots(&answer.expected_roots, roots, counts)
            }
            ObservedEffect::Mount {
                instance,
                rows_written,
                roots,
            } => {
                let Some((expected_instance, expected_rows)) = &answer.expected_mount else {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::NotModeled,
                        describe_answer(answer),
                        "实现交回的是挂载".to_string(),
                    ));
                };
                if instance != expected_instance {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::InstanceGeneration,
                        format!("取号 {}", expected_instance.0),
                        format!("取号 {}", instance.0),
                    ));
                }
                if rows_written != expected_rows {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::InstanceRows,
                        format!("写行 {expected_rows:?}"),
                        format!("写行 {rows_written:?}"),
                    ));
                }
                if answer.operation == ModelOperationKind::MountRollback {
                    let base_txg = expected_rows
                        .iter()
                        .find(|row| row.is_rollback)
                        .map(|row| row.selected_root_txg);
                    let floor = self.effective_rollback_floor();
                    if floor.0 > 0 && base_txg == Some(floor) {
                        counts.rollbacks_accepted_at_the_effective_floor += 1;
                    }
                }
                self.judge_roots(&answer.expected_roots, roots, counts)
            }
            ObservedEffect::ColdStart { read_back } => {
                let Some(expected) = &answer.expected_read_back else {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::NotModeled,
                        describe_answer(answer),
                        "实现交回的是冷启动读回".to_string(),
                    ));
                };
                let matches = match (expected, read_back) {
                    (
                        ModelReadBack::NoFile {
                            root: expected_root,
                        },
                        ObservedReadBack::NoFile { root },
                    ) => expected_root == root,
                    (
                        ModelReadBack::FileRead {
                            root: expected_root,
                            content: expected_content,
                        },
                        ObservedReadBack::FileRead { root, content },
                    ) => {
                        counts.cold_start_contents_compared += 1;
                        expected_root == root && expected_content.as_ref() == content.as_slice()
                    }
                    (
                        ModelReadBack::NoFile { .. } | ModelReadBack::FileRead { .. },
                        ObservedReadBack::NoFile { .. }
                        | ObservedReadBack::FileRead { .. }
                        | ObservedReadBack::Failed { .. },
                    ) => false,
                };
                if matches {
                    Ok(())
                } else {
                    Err(ModelDisagreement::new(
                        ModelDisagreementAspect::ColdStartReadBack,
                        describe_read_back(expected),
                        describe_observed_read_back(read_back),
                    ))
                }
            }
        }
    }

    fn judge_roots(
        &self,
        expected_roots: &[ModelRoot],
        observed_roots: &[ObservedRoot],
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        if expected_roots.len() != observed_roots.len() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::PublishCount,
                format!(
                    "写出 {} 条根：{:?}",
                    expected_roots.len(),
                    expected_roots
                        .iter()
                        .map(|root| root.key)
                        .collect::<Vec<_>>()
                ),
                format!(
                    "写出 {} 条根：{:?}",
                    observed_roots.len(),
                    observed_roots
                        .iter()
                        .map(|root| root.key)
                        .collect::<Vec<_>>()
                ),
            ));
        }
        for (expected, observed) in expected_roots.iter().zip(observed_roots) {
            counts.roots_compared += 1;
            if expected.key != observed.key {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::RootIdentity,
                    format!("{:?}", expected.key),
                    format!("{:?}", observed.key),
                ));
            }
            if expected.journal_counter != observed.journal_counter {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::JournalCounter,
                    format!("{:?} 的 jsn {}", expected.key, expected.journal_counter.0),
                    format!("jsn {}", observed.journal_counter.0),
                ));
            }
            if expected.rollback_floor != observed.rollback_floor {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::RollbackFloor,
                    format!("{:?} 带 F {}", expected.key, expected.rollback_floor.0),
                    format!("带 F {}", observed.rollback_floor.0),
                ));
            }
            if expected.file.is_some() != observed.has_file {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::FilePresence,
                    format!("{:?} 有文件：{}", expected.key, expected.file.is_some()),
                    format!("有文件：{}", observed.has_file),
                ));
            }
            self.judge_allocation_generations(expected, observed, counts)?;
        }
        Ok(())
    }

    /// 这一版每个单元的分配记录：每块盘各一条、仍分配着、分配代等于写它的那次发布的 txg（D3（空间分配） 已定项 3 / 7）。
    fn judge_allocation_generations(
        &self,
        expected: &ModelRoot,
        observed: &ObservedRoot,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        for (role, records) in &observed.unit_allocation_records {
            let Some(written_at) = expected.role_written_at.get(role) else {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::AllocationGeneration,
                    format!("{:?} 这一版没有 {role:?} 这个角色", expected.key),
                    format!("实现交回了 {role:?} 的单元"),
                ));
            };
            let devices_seen: BTreeSet<ModelDeviceIdentity> =
                records.iter().map(|record| record.device).collect();
            let devices_expected: BTreeSet<ModelDeviceIdentity> =
                self.geometry.devices.iter().copied().collect();
            let well_formed = records.len() == self.geometry.devices.len()
                && devices_seen == devices_expected
                && records
                    .iter()
                    .all(|record| !record.is_released && record.generation == *written_at);
            counts.allocation_records_compared += u64::try_from(records.len()).expect("条数");
            if !well_formed {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::AllocationGeneration,
                    format!(
                        "{:?} 的 {role:?}：每块盘一条、仍分配、分配代 {}（写它的那次发布）",
                        expected.key, written_at.0
                    ),
                    format!("{records:?}"),
                ));
            }
        }
        Ok(())
    }

    fn write_root(&mut self, root: ModelRoot) {
        self.highest_published_txg = self.highest_published_txg.max(root.key.checkpoint_txg);
        self.highest_journal_counter = self.highest_journal_counter.max(root.journal_counter);
        self.ring
            .insert(ring_position_of(root.key.checkpoint_txg), root);
    }

    fn advance_by_success(&mut self, answer: &ModelAnswer) {
        for root in &answer.expected_roots {
            self.write_root(root.clone());
        }
        match answer.session_after_success {
            ModelSessionAfterSuccess::StaysOpen => {
                if let (Some(session), Some(last)) =
                    (self.session.as_mut(), answer.expected_roots.last())
                {
                    session.current = last.clone();
                }
            }
            ModelSessionAfterSuccess::OpenedAs(instance) => {
                self.highest_acquired_instance = instance;
                let current = answer
                    .expected_roots
                    .last()
                    .expect("挂载至少写出写行那一次")
                    .clone();
                self.session = Some(ModelSession { instance, current });
            }
            ModelSessionAfterSuccess::Closed => self.session = None,
        }
    }
}

fn describe_answer(answer: &ModelAnswer) -> String {
    if answer.required_refusals.is_empty() {
        let permitted: Vec<&'static str> = answer
            .permitted_refusals
            .iter()
            .map(|reason| reason.name())
            .collect();
        format!(
            "{:?} 该成（写出 {:?}；允许拒的只有容量墙区间与 {permitted:?}）",
            answer.operation,
            answer
                .expected_roots
                .iter()
                .map(|root| (root.key.checkpoint_txg.0, root.key.instance.0))
                .collect::<Vec<_>>()
        )
    } else {
        let required: Vec<&'static str> = answer
            .required_refusals
            .iter()
            .map(|reason| reason.name())
            .collect();
        format!("{:?} 该拒：{required:?}", answer.operation)
    }
}

fn describe_read_back(read_back: &ModelReadBack) -> String {
    match read_back {
        ModelReadBack::NoFile { root } => format!("所选根 {root:?}，没有文件"),
        ModelReadBack::FileRead { root, content } => format!(
            "所选根 {root:?}，读回 {} 字节（末字节 {:?}）",
            content.len(),
            content.last()
        ),
    }
}

fn describe_observed_read_back(read_back: &ObservedReadBack) -> String {
    match read_back {
        ObservedReadBack::NoFile { root } => format!("所选根 {root:?}，没有文件"),
        ObservedReadBack::FileRead { root, content } => format!(
            "所选根 {root:?}，读回 {} 字节（末字节 {:?}）",
            content.len(),
            content.last()
        ),
        ObservedReadBack::Failed { what } => format!("读回失败：{what}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_device_model() -> IdealModel {
        IdealModel::after_make_filesystem(ModelPoolGeometry {
            devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
            device_size_in_bytes: 4 << 30,
        })
    }

    fn succeed(model: &mut IdealModel, answer: &ModelAnswer) {
        assert!(
            answer.required_refusals.is_empty(),
            "这一步模型要求拒：{:?}",
            answer.required_refusals
        );
        model.advance_by_success(answer);
    }

    fn key(checkpoint_txg: u64, instance: u64) -> ModelRootKey {
        ModelRootKey {
            checkpoint_txg: ModelCheckpointTxg(checkpoint_txg),
            instance: ModelInstanceGeneration(instance),
        }
    }

    /// mkfs、同一进程取号暖机、第一个文件（txg 3）、可写挂载（实例 2：写行 txg 4、暖机 txg 5）、覆盖写 txg 6–9，内容是 txg 号那一个字节。
    fn model_after_four_overwrites_in_a_second_instance() -> IdealModel {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        succeed(&mut model, &first);
        model.close_session();
        let mount = model.answer_mount_writable();
        succeed(&mut model, &mount);
        for content_byte in 6_u8..=9 {
            let overwrite = model
                .answer_publish_overwrite(&[content_byte])
                .expect("会话开着、现行版本带文件");
            succeed(&mut model, &overwrite);
        }
        model
    }

    /// 内容装不装得下按 32768 含头与预留位算（D4（校验和位置） 已定项 5）：正好装满不拒，多一个字节才拒。
    #[test]
    fn content_of_exactly_the_payload_capacity_is_accepted_and_one_byte_more_is_refused() {
        assert_eq!(data_unit_payload_capacity_in_bytes(), 32768 - 105 - 29);
        let model = model_after_four_overwrites_in_a_second_instance();
        let capacity = usize::try_from(data_unit_payload_capacity_in_bytes()).expect("三万多");
        let fits = model
            .answer_publish_overwrite(&vec![7; capacity])
            .expect("会话开着");
        assert!(fits.required_refusals.is_empty(), "{fits:?}");
        let exceeds = model
            .answer_publish_overwrite(&vec![7; capacity + 1])
            .expect("会话开着");
        assert_eq!(
            exceeds.required_refusals,
            BTreeSet::from([ModelRefusalReason::ContentExceedsDataUnitPayload])
        );
    }

    /// 抬 F 的上限（D16（发布语义） 已定项 1）：有效根 txg 0–9，非空的是 3（有了文件）与 6–9（覆盖写），4、5（写行、暖机）照抄 3 的文件、
    /// 0–2 没有文件，都不算。第 4 新的非空根是 6；每块盘上最新的有效根：盘 0 是 9（区域 0），盘 1 是 7（区域 1）⇒ 上限 min(7, 6) = 6。
    /// 抬到 6 不拒，抬到 7 必须拒。
    #[test]
    fn the_ceiling_is_the_fourth_newest_non_empty_root_capped_by_the_newest_root_on_every_device() {
        let model = model_after_four_overwrites_in_a_second_instance();
        assert_eq!(model.rollback_floor_ceiling(), Some(ModelCheckpointTxg(6)));
        let at_the_ceiling = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(6))
            .expect("会话开着");
        assert!(
            at_the_ceiling.required_refusals.is_empty(),
            "{at_the_ceiling:?}"
        );
        assert_eq!(
            at_the_ceiling
                .expected_roots
                .iter()
                .map(|root| root.key)
                .collect::<Vec<_>>(),
            vec![key(10, 2), key(11, 2)],
            "txg 10 落盘 1、txg 11 落盘 0：两块盘都有带新 F 的根就停"
        );
        let above = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(7))
            .expect("会话开着");
        assert_eq!(
            above.required_refusals,
            BTreeSet::from([ModelRefusalReason::FloorAboveCeiling])
        );
    }

    /// 抬 F 到 6 之后 F_生效 = 6：回退到 txg 6 的根是候选（txg ≥ F_生效，D16（发布语义） 已定项 1「回退候选集」），txg 5 的必须拒；
    /// 回退到 (6, 2) 之后实例 2 的 7–11 被抛弃（写了回退行 (2, 6, 0)），再回退到 (9, 2) 必须拒；冷启动读回回退那一版（txg 6 写的内容）。
    #[test]
    fn rollback_to_the_effective_floor_is_accepted_and_abandons_the_newer_roots_of_that_instance() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let raise = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(6))
            .expect("会话开着");
        succeed(&mut model, &raise);
        assert_eq!(model.effective_rollback_floor(), ModelCheckpointTxg(6));
        model.close_session();
        assert_eq!(
            model.answer_mount_rollback(key(5, 2)).required_refusals,
            BTreeSet::from([ModelRefusalReason::RollbackTargetBelowEffectiveFloor])
        );
        let rollback = model.answer_mount_rollback(key(6, 2));
        assert_eq!(
            rollback.expected_mount,
            Some((
                ModelInstanceGeneration(3),
                vec![ModelInstanceRow {
                    instance: ModelInstanceGeneration(2),
                    selected_root_txg: ModelCheckpointTxg(6),
                    applied_transaction_high_water: 0,
                    is_rollback: true,
                }]
            ))
        );
        succeed(&mut model, &rollback);
        model.close_session();
        assert_eq!(
            model.answer_mount_rollback(key(9, 2)).required_refusals,
            BTreeSet::from([ModelRefusalReason::RollbackTargetOnAbandonedTimeline])
        );
        let cold_start = model.answer_cold_start_recover();
        let Some(ModelReadBack::FileRead { root, content }) = cold_start.expected_read_back else {
            panic!("回退那一版带文件：{cold_start:?}");
        };
        assert_eq!(
            root,
            key(13, 3),
            "回退写行 txg 12（盘 0）、暖机 txg 13（盘 1）"
        );
        assert_eq!(content.as_ref(), &[6]);
    }

    /// F_生效 = 各盘所带 F 最大值的最小值：只有一块盘上有带新 F 的根时不生效（D16（发布语义） 已定项 1「生效」）。抬 F 半路停在第一次
    /// （txg 10 落盘 1），盘 0 上最大的 F 仍是 0。
    #[test]
    fn a_raised_floor_carried_by_one_device_only_does_not_take_effect() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let raise = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(6))
            .expect("会话开着");
        let first_publish_only = raise.expected_roots[..1].to_vec();
        for root in first_publish_only {
            model.write_root(root);
        }
        assert_eq!(model.effective_rollback_floor(), ModelCheckpointTxg(0));
    }

    /// 单元的分配代是写它的那次发布的 txg（D3（空间分配） 已定项 3 / 7）：写行那次（txg 4）照抄的数据单元分配代 3，写成 4 就对不上；
    /// 这次重写的实例表分配代 4。
    #[test]
    fn a_carried_unit_keeps_the_generation_of_the_publish_that_wrote_it() {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model.answer_publish_first_file(&[3]).expect("会话开着");
        succeed(&mut model, &first);
        model.close_session();
        let mount = model.answer_mount_writable();
        let row_publish = &mount.expected_roots[0];
        let records = |generation: u64| {
            [ModelDeviceIdentity(0), ModelDeviceIdentity(1)]
                .into_iter()
                .map(|device| ObservedAllocationRecord {
                    device,
                    generation: ModelCheckpointTxg(generation),
                    is_released: false,
                })
                .collect::<Vec<_>>()
        };
        let observed = |data_generation: u64| ObservedRoot {
            key: row_publish.key,
            journal_counter: row_publish.journal_counter,
            rollback_floor: row_publish.rollback_floor,
            has_file: true,
            unit_allocation_records: vec![
                (ModelUnitRole::Data, records(data_generation)),
                (ModelUnitRole::InstanceTable, records(4)),
            ],
        };
        let mut counts = ModelJudgementCounts::default();
        assert_eq!(
            model.judge_allocation_generations(row_publish, &observed(3), &mut counts),
            Ok(())
        );
        assert_eq!(counts.allocation_records_compared, 4);
        let wrong = model
            .judge_allocation_generations(row_publish, &observed(4), &mut counts)
            .expect_err("照抄的数据单元分配代写成了这次的 txg");
        assert_eq!(wrong.aspect, ModelDisagreementAspect::AllocationGeneration);
    }
}
```

## 三、新文件 `crates/singlefs-harness/src/model_comparison.rs` 全文

```rust
//! 理想模型与实现之间那层胶水（里程碑「第二个事务」增补 3 第 2 件）：把实现交回的东西（`singlefs_core` 的类型）换成模型的观测，
//! 把实现的错误成员映射到模型的拒绝理由。D13（验证路线） 已定项 5 管的是模型本身（`model.rs` 只 `use singlefs_format`）；
//! 拿实现结局与模型比的这一层可以用 core 的类型。
//!
//! 映射只做「这个成员说的是哪条理由」，不做判断：成员说得出条款理由的映射过去，I/O、盘坏、走读失败这类模型里没有的成员一律
//! `Unexplained`（不建崩溃与设备错的历史里它们都不该出现）。

use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::block_device::BlockDeviceError;
use singlefs_core::mount::{InstanceRow, MountError, Mounted};
use singlefs_core::recovery::RecoveryOutcome;
use singlefs_core::transaction::{
    PoolVersion, PublishError, TransactionOutput, TransactionUnit, ZeroUnitPublishOutput,
};

use crate::model::{
    ModelCheckpointTxg, ModelDeviceIdentity, ModelInstanceGeneration, ModelInstanceRow,
    ModelJournalCounter, ModelRefusalReason, ModelRootKey, ModelUnitRole, ObservedAllocationRecord,
    ObservedEffect, ObservedReadBack, ObservedRefusalReason, ObservedRoot,
};

#[must_use]
pub fn model_instance(instance: InstanceGeneration) -> ModelInstanceGeneration {
    ModelInstanceGeneration(u64::from(instance.0))
}

#[must_use]
pub fn model_txg(checkpoint_txg: CheckpointTxg) -> ModelCheckpointTxg {
    ModelCheckpointTxg(checkpoint_txg.0)
}

#[must_use]
pub fn model_root_key(instance: InstanceGeneration, checkpoint_txg: CheckpointTxg) -> ModelRootKey {
    ModelRootKey {
        checkpoint_txg: model_txg(checkpoint_txg),
        instance: model_instance(instance),
    }
}

#[must_use]
pub fn model_unit_role(unit: TransactionUnit) -> ModelUnitRole {
    match unit {
        TransactionUnit::Data => ModelUnitRole::Data,
        TransactionUnit::ExtentRoot => ModelUnitRole::ExtentRoot,
        TransactionUnit::InodeLeaf => ModelUnitRole::InodeLeaf,
        TransactionUnit::InodeRoot => ModelUnitRole::InodeRoot,
        TransactionUnit::AllocationTree => ModelUnitRole::AllocationTree,
        TransactionUnit::AccountingTree => ModelUnitRole::AccountingTree,
        TransactionUnit::MappingTree => ModelUnitRole::MappingTree,
        TransactionUnit::TreeTable => ModelUnitRole::TreeTable,
        TransactionUnit::InstanceTable => ModelUnitRole::InstanceTable,
    }
}

#[must_use]
pub fn model_instance_row(row: &InstanceRow) -> ModelInstanceRow {
    ModelInstanceRow {
        instance: model_instance(row.instance),
        selected_root_txg: model_txg(row.selected_root_txg),
        applied_transaction_high_water: row.applied_transaction_high_water,
        is_rollback: row.is_rollback,
    }
}

/// 带文件的一版：根的身份、jsn、F，和这一版每个单元（`units` 里的角色）在这一版分配记录里的那几条（按槽号找，每块盘一条）。
#[must_use]
pub fn observed_root_of_file_version(output: &TransactionOutput) -> ObservedRoot {
    let unit_allocation_records = output
        .units
        .iter()
        .map(|unit| {
            let records = output
                .allocation_records
                .iter()
                .filter(|record| record.slot == unit.slot)
                .map(|record| ObservedAllocationRecord {
                    device: ModelDeviceIdentity(record.device.0),
                    generation: model_txg(record.generation),
                    is_released: record.is_released,
                })
                .collect();
            (model_unit_role(unit.identity), records)
        })
        .collect();
    ObservedRoot {
        key: model_root_key(output.root.instance, output.root.checkpoint_txg),
        journal_counter: ModelJournalCounter(output.record.counter),
        rollback_floor: model_txg(output.root.rollback_floor),
        has_file: true,
        unit_allocation_records,
    }
}

/// 树表 0 条的一版（零单元发布）：没有单元、没有分配记录。
#[must_use]
pub fn observed_root_of_version_without_file(output: &ZeroUnitPublishOutput) -> ObservedRoot {
    ObservedRoot {
        key: model_root_key(output.root.instance, output.root.checkpoint_txg),
        journal_counter: ModelJournalCounter(output.record.counter),
        rollback_floor: model_txg(output.root.rollback_floor),
        has_file: false,
        unit_allocation_records: Vec::new(),
    }
}

#[must_use]
pub fn observed_root_of_pool_version(version: &PoolVersion) -> ObservedRoot {
    match version {
        PoolVersion::WithFile(output) => observed_root_of_file_version(output),
        PoolVersion::WithoutFile(output) => observed_root_of_version_without_file(output),
    }
}

/// 一次挂载做成了什么：取到的号、写的行、写行与暖机那几条根。
#[must_use]
pub fn observed_mount(mounted: &Mounted) -> ObservedEffect {
    ObservedEffect::Mount {
        instance: model_instance(mounted.output.instance),
        rows_written: mounted
            .output
            .rows_written
            .iter()
            .map(model_instance_row)
            .collect(),
        roots: std::iter::once(&mounted.output.row_publish)
            .chain(mounted.output.warm_up_publishes.iter())
            .map(observed_root_of_pool_version)
            .collect(),
    }
}

fn explained(reason: ModelRefusalReason) -> ObservedRefusalReason {
    ObservedRefusalReason::Explained(vec![reason])
}

/// 发布的错误成员说的是哪条理由。
#[must_use]
pub fn refusal_reason_of_publish_error(error: &PublishError) -> ObservedRefusalReason {
    match error {
        PublishError::NoSpaceFor { .. } => explained(ModelRefusalReason::UnitAreaWall),
        PublishError::AllocationRecordsExceedOneNode { .. } => {
            explained(ModelRefusalReason::AllocationRecordNodeWall)
        }
        PublishError::AccountingEntriesExceedOneNode { .. } => {
            explained(ModelRefusalReason::AccountingNodeWall)
        }
        PublishError::ContentExceedsDataUnit { .. } => {
            explained(ModelRefusalReason::ContentExceedsDataUnitPayload)
        }
        PublishError::FirstFileVersionNotRightAfterTheSecondWarmUp { .. } => {
            explained(ModelRefusalReason::FirstFileNotRightAfterTheWarmUp)
        }
        // 释放判定路径的四种：上一版的映射或分配记录与上一版对不上，健康的历史里不该出现。
        PublishError::ReleaseNotInMapping { .. }
        | PublishError::ReleaseTargetNotAllocated { .. }
        | PublishError::ReleaseTargetAlreadyReleased { .. }
        | PublishError::ReleaseSpanMismatch { .. }
        | PublishError::BlockDevice(_) => ObservedRefusalReason::Unexplained,
    }
}

/// 零单元发布只会报块设备错：内存盘不报错，模型里没有它的理由。
#[must_use]
pub fn refusal_reason_of_block_device_error(_error: &BlockDeviceError) -> ObservedRefusalReason {
    ObservedRefusalReason::Unexplained
}

/// 挂载、回退、抬 F 的错误成员说的是哪条理由。
#[must_use]
pub fn refusal_reason_of_mount_error(error: &MountError) -> ObservedRefusalReason {
    match error {
        MountError::Publish(cause)
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { cause, .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { cause, .. } => {
            refusal_reason_of_publish_error(cause)
        }
        MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion => {
            explained(ModelRefusalReason::RaiseWithFormatTimeInstanceTableUnsupported)
        }
        MountError::RollbackTargetNotInRing(_) => {
            explained(ModelRefusalReason::RollbackTargetNotInRing)
        }
        // 成员只说「不在候选集里」，两条理由之一（理由文字是给人看的，不拿来分流）。
        MountError::RollbackTargetNotACandidate { .. } => ObservedRefusalReason::Explained(vec![
            ModelRefusalReason::RollbackTargetBelowEffectiveFloor,
            ModelRefusalReason::RollbackTargetOnAbandonedTimeline,
        ]),
        MountError::RollbackFloorAboveCeiling { .. } => {
            explained(ModelRefusalReason::FloorAboveCeiling)
        }
        MountError::InstanceRowsOnVersionWithoutFileUnsupported { .. } => {
            explained(ModelRefusalReason::RowsOnVersionWithoutFileUnsupported)
        }
        MountError::RollbackToVersionWithoutFileUnsupported(_) => {
            explained(ModelRefusalReason::RollbackToVersionWithoutFileUnsupported)
        }
        MountError::InstanceTableRowsExceedOnePageSecondPageUnsupported { .. } => {
            explained(ModelRefusalReason::InstanceTableOnePageWall)
        }
        MountError::FormattedPoolMountNotShapedLikeTheFirstTransaction { .. } => {
            explained(ModelRefusalReason::FormattedPoolMountNotShapedLikeTheFirstTransaction)
        }
        // 恢复失败、记录读不出、表解不开、取号失败、坏盘上才有的根、判定与取号之间号变了：健康的内存盘上都不该出现。
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::Acquisition(_)
        | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. } => {
            ObservedRefusalReason::Unexplained
        }
    }
}

/// 抬 F 被上限拒时实现报的上限。
#[must_use]
pub fn reported_ceiling_of_mount_error(error: &MountError) -> Option<ModelCheckpointTxg> {
    match error {
        MountError::RollbackFloorAboveCeiling { ceiling, .. } => Some(model_txg(*ceiling)),
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion
        | MountError::Acquisition(_)
        | MountError::Publish(_)
        | MountError::RollbackTargetNotInRing(_)
        | MountError::RollbackTargetNotACandidate { .. }
        | MountError::InstanceRowsOnVersionWithoutFileUnsupported { .. }
        | MountError::RollbackToVersionWithoutFileUnsupported(_)
        | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
        | MountError::InstanceTableRowsExceedOnePageSecondPageUnsupported { .. }
        | MountError::FormattedPoolMountNotShapedLikeTheFirstTransaction { .. } => None,
    }
}

/// 冷启动读回的结局：所选根、读回的内容或失败的成员。
#[must_use]
pub fn observed_read_back(outcome: &RecoveryOutcome) -> ObservedReadBack {
    match outcome {
        RecoveryOutcome::NoFile { root } => ObservedReadBack::NoFile {
            root: model_root_key(root.0, root.1),
        },
        RecoveryOutcome::FileRead { root, content } => ObservedReadBack::FileRead {
            root: model_root_key(root.0, root.1),
            content: content.clone(),
        },
        RecoveryOutcome::Failed { failure, root } => ObservedReadBack::Failed {
            what: format!("{failure:?}（所选根 {root:?}）"),
        },
    }
}

#[cfg(test)]
mod tests {
    /// 模型模块只用格式常量那一个 crate（D13（验证路线） 已定项 5）：`model.rs` 里注释之外的每一行都不提 `singlefs_core`、`singlefs_checker`，
    /// `use` 只有 `std` 与 `singlefs_format`（测试模块的 `use super::*` 除外）。胶水（这个文件）可以用 core。
    #[test]
    fn the_model_module_uses_only_the_standard_library_and_the_format_constants() {
        let source = include_str!("model.rs");
        let code_lines: Vec<&str> = source
            .lines()
            .map(str::trim_start)
            .filter(|line| !line.starts_with("//"))
            .collect();
        assert!(code_lines.len() > 100, "读到的是 model.rs 本身");
        for line in &code_lines {
            assert!(
                !line.contains("singlefs_core") && !line.contains("singlefs_checker"),
                "模型模块里有一行提到了实现或 checker：{line}"
            );
            if line.starts_with("use ") {
                assert!(
                    line.starts_with("use std::")
                        || line.starts_with("use singlefs_format::")
                        || *line == "use super::*;",
                    "模型模块的 use 只许 std 与 singlefs_format：{line}"
                );
            }
        }
    }
}
```

## 四、新文件 `.claude/gate.d/74-model-differential.sh` 全文

```bash
#!/usr/bin/env bash
# gate-stage: 模型对拍（里程碑「第二个事务」增补 3 第 2 件：随机历史快档与两个取样点，每一步拿实现的结局与只住内存的理想模型比）
# gate-covers: 模型对拍
#
# 被测的是 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs` 那三段：快档、偏向抬 F 之后复用的取样点、
# 偏向抬 F 之后回退的取样点。每段的报告里有一行「模型对拍 N 步：…」（`crates/singlefs-harness/src/history.rs` 的报告渲染），
# 模型模块 `crates/singlefs-harness/src/model.rs` 只用 `singlefs_format` 的常量（D13（验证路线） 已定项 5）。
# 这里在 release 下单跑那一个测试二进制，要求三段都打出「模型对拍」那一行、步数都大于 0——测试绿而模型一步没判，等于没对拍。
# 判别力：模型对拍自己会不会红，由 `crates/mutations.tsv` 第 146–155 行（门禁 59 号）证明；这个阶段只判「跑了、判过、没报对不上」。
# 样本：被判目录里没有 Cargo.toml 而有 `model-differential-cargo-output.log` 时，不跑 cargo，只判那份录好的输出
# （`.claude/gate.d/fixtures/74-model-differential.sh/` 的红绿样本走这一支）。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2

TEST_BINARY="second_transaction_supplement_three_random_history"
SECTIONS=("随机历史快档" "随机历史：偏向抬 F 之后复用的取样点" "随机历史：偏向抬 F 之后回退的取样点")

log="$(mktemp)"
if [[ -f Cargo.toml && -d crates/singlefs-harness ]]; then
  if ! cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --nocapture >"$log" 2>&1; then
    tail -40 "$log"
    echo "  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）"
    echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test $TEST_BINARY -- --nocapture"
    echo "                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。"
    rm -f "$log"
    exit 1
  fi
elif [[ -f model-differential-cargo-output.log ]]; then
  cp model-differential-cargo-output.log "$log"
else
  rm -f "$log"
  echo "  ! 没有 crates/singlefs-harness，本阶段跳过（增补 3 第 2 件之前没有模型）"
  exit 77
fi

missing=()
zero=()
reported=()
for section in "${SECTIONS[@]}"; do
  # 一段的报告从它的标题行起、到下一个「── … ──」标题行为止；不按固定行数取，报告长了照样找得到
  line="$(awk -v header="── ${section} ──" '$0 == header { inside = 1; next } inside && /^── .* ──$/ { exit } inside' "$log" | grep -m1 '模型对拍 [0-9]* 步' || true)"
  if [[ -z "$line" ]]; then
    missing+=("$section")
    continue
  fi
  steps="$(sed -n 's/.*模型对拍 \([0-9]*\) 步.*/\1/p' <<<"$line")"
  if [[ -z "$steps" || "$steps" == 0 ]]; then
    zero+=("$section")
    continue
  fi
  reported+=("${section}：$(sed 's/^[[:space:]]*//' <<<"$line")")
done
rm -f "$log"

if (( ${#missing[@]} > 0 )); then
  echo "  ✗ 测试跑过了，这几段却没有「模型对拍 N 步」那一行：$(printf '「%s」' "${missing[@]}")"
  echo "     → 怎么办：history.rs 的报告渲染要在每段报告里打「模型对拍 N 步：…」；测试文件里那三段要经 print_uncaptured 把报告写进标准输出。"
  echo "                行没了多半是执行器绕开了 judge_by_model，那一段等于没对拍。"
  exit 1
fi
if (( ${#zero[@]} > 0 )); then
  echo "  ✗ 这几段模型一步都没判：$(printf '「%s」' "${zero[@]}")"
  echo "     → 怎么办：测试绿而模型没判过任何一步，等于没对拍（show-me-test.md「扫到 0 项也不是通过」）。"
  echo "                看 history.rs 的执行器每一步调入口之前有没有先问模型（judge_by_model），前提不满足的步不算。"
  exit 1
fi
echo "  ✓ 模型对拍三段都判过、实现与模型没有对不上的（查了 ${#SECTIONS[@]} 段）："
for entry in "${reported[@]}"; do
  echo "      $entry"
done
```
