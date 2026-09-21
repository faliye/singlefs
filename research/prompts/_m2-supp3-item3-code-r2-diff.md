# 这一轮被判的改动（相对第一轮判过的那一版）

第一轮判过的 `crash_injection.rs` 取自 `git show 3cff909^:research/prompts/_m2-supp3-item3-code-r1-diff.md` 里「新增整份」那一段（1040 行）；下面是它到今天工作区那一版（1415 行）的 diff。两个用例文件与 `crates/mutations.tsv` 是这次改动里新增或改过的，整份在工作区，腿自己读。

```diff
--- /tmp/claude-1000/-home-fy5090-code-singlefs/d16a74c5-453c-44d5-8a19-7e71d116de72/scratchpad/crash_injection-r1.rs	2026-09-21 08:08:43.967624537 +0000
+++ crates/singlefs-harness/src/crash_injection.rs	2026-09-21 00:41:22.698204818 +0000
@@ -1,16 +1,20 @@
-//! 崩溃注入（里程碑「第二个事务」增补 3 第 3 件）：第 1 件的每段历史跑的过程中开着内容保留的录制流，按种子抽若干次写、
-//! 在那次写之后把流截断，对每个崩溃点重建镜像、跑恢复、跑池级 checker，并问理想模型「恢复到的这一版允不允许」
-//! （`crate::model::crash_recovery_disagreement`）。
+//! 崩溃注入（里程碑「第二个事务」增补 3 第 3 件）：第 1 件的每段历史跑的过程中开着内容保留的录制流，把流按屏障与 FUA 写切段，
+//! 按种子摆若干个崩溃状态（一段 + 那一段里持久了哪几个写），对每个崩溃状态重建镜像、跑恢复、跑池级 checker、跑记录核对器，
+//! 并问理想模型「恢复到的这一版允不允许」（`crate::model::crash_recovery_disagreement`）。
 //!
-//! 抽样，不全枚举：全量枚举仍归门禁 54 号的两条固定流（`crate::crash` 的层 0 枚举）。两者分工不同——
-//! 层 0 在两条写死的流上把每一段的任意写子集都摆一遍（含乱序持久与撕裂），这里只摆「前缀持久」这一种，
-//! 换来的是任意一段随机历史上的崩溃点：关闭重开、可写挂载、回退、抬 F、零单元发布、冷启动都在流里。
+//! 枚举域与层 0（`crate::crash`）同一个：更早的段整段持久、当前段任意真子集、更晚的段一个都没持久，撕裂并进「没持久」
+//! （D13（验证路线） 已定项 4）。差别只在**怎么取**：层 0 在两条写死的流上全枚举，这里在任意一段随机历史上抽样——
+//! 关闭重开、可写挂载、回退、抬 F、零单元发布、冷启动都在流里，而那样的流全枚举不起。
+//! 屏障因此进了枚举域：少一道屏障就把两段并成一段，段内子集立刻多出「后发的写先持久」那一类状态
+//! （`crates/mutations.tsv` 里「步 3：零单元发布在记录与根之间少一道屏障」那一条靠这个判得出）。
 //!
-//! 每段历史自己怎么跑由 `HistoryExecution` 定；崩溃点只抽在起点之后、最后一个跑完的操作为止的那段流上——
-//! 起点（mkfs 与起点那次发布）整个持久，失败那一步的写不抽（模型还没判过它写出的根，目录里没有那一版）。
+//! 每段历史自己怎么跑由 `HistoryExecution` 定；崩溃状态摆在起点那一段起、最后一个跑完的操作为止的那几段上——
+//! 起点（mkfs 与起点那次发布）也算（代码三方 m2-supp3-item3-code-r1 判决 K1-d，用户 2026-09-20 定案第 4 条），
+//! 失败那一步的写不摆（模型还没判过它写出的根，目录里没有那一版）。
 
 use std::collections::{BTreeMap, BTreeSet};
 use std::fmt::Write as _;
+use std::path::{Path, PathBuf};
 use std::rc::Rc;
 use std::sync::atomic::{AtomicUsize, Ordering};
 use std::sync::mpsc;
@@ -18,13 +22,20 @@
 
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
-use singlefs_core::address::DeviceIdentity;
-use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
+use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
+use singlefs_core::recovery::{
+    choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome,
+};
 
-use crate::crash::{root_identity_written_by, MemoryPool};
+use crate::crash::{
+    check_records_against, newest_persisted_root, some_publish_persisted_without_its_root,
+    writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, RecordCheck, RetainedWrite,
+    SECTOR_BYTES,
+};
 use crate::history::{
     classify_failure, execute_history_with, generate_history_with_weights,
-    newest_ring_root_and_slot_count, FailureObservation, FailureSignature, GeneratedHistory,
+    newest_ring_root_and_slot_count_of, raised_floor_lands_only_on_abandoned_roots,
+    record_check_aspects, FailureObservation, FailureSignature, GeneratedHistory,
     GenerationWeights, HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind,
     HistorySeed, SeededRandomSource, StepPosition, KNOWN_RED_FORMS,
 };
@@ -33,18 +44,40 @@
 };
 use crate::model_comparison::{model_root_key, observed_read_back_after_a_crash};
 use crate::segments::{FixedGeometry, StepKind};
-use crate::{RecordedOperationKind, RetainedOperation, SharedStream};
+use crate::SharedStream;
 
 /// 崩溃注入的工作线程数从这个环境变量取（十进制正整数）；没设就取 [`std::thread::available_parallelism`]。
 pub const CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =
     "SINGLEFS_CRASH_INJECTION_THREADS";
 
-/// 抽崩溃点的随机源与生成历史那一路岔开：同一个种子，历史怎么生成不受抽崩溃点影响，反过来也一样。
+/// 这一个测试周期用的种子基：随机历史那五段（增补 3 第 1 件）与崩溃注入这三档（第 3 件）的种子区间都从它起。
+///
+/// **一个测试周期**从两件事里先到的那一件开始：开一个新里程碑，或者用户显式说从零开始测试 / 重建测试。
+/// 周期因此可能比里程碑短——里程碑跑到一半被显式重建，那就是新的一个周期。
+///
+/// 当前这个周期（里程碑「第二个事务」）的基是 2026-09-20 抽的，抽法：
+/// `python3 -c "import secrets; print(secrets.randbits(63))"`。
+///
+/// **开一个新周期要做三样**：
+/// 1. 照上面那条命令重抽一次，把下面这个数换掉；
+/// 2. 旧的数据盘与虚拟盘镜像全删掉、全部重建（上一个周期的盘面是在旧种子基上攒出来的）；
+/// 3. `crates/mutations.tsv` 整表在新种子基上重验一遍——点名随机历史与崩溃注入那些行的判红，都是在旧基上量的。
+///
+/// 周期之内写死不动：`crates/mutations.tsv` 的「这条变异必须红」与里程碑那几条验收说的都是**这一批**历史上的判红，
+/// 种子基一动它们就成了掷骰子，判别力本身没了。周期之间重抽，才不会永远只测同一批历史（用户 2026-09-20 定案第 7 条，
+/// 同日定的范围：随机说的是多次实验之间的随机，不是每次跑重抽）。它接着用户已定的另一条：
+/// 历史实验数据只在它自己那个周期下有效。
+pub const SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE: u64 = 7_463_871_032_432_355_113;
+
+/// 摆崩溃状态的随机源与生成历史那一路岔开：同一个种子，历史怎么生成不受它影响，反过来也一样。
 const CRASH_POINT_SEED_SALT: u64 = 0x63_72_61_73_68_70_74_00;
 
 /// 每个工作线程摊到的片数：各段历史的长短差得远（一步就被拒的与连发四十次的），片切得比线程多，先跑完的线程接着领下一片。
 const SLICES_PER_WORKER_THREAD: usize = 4;
 
+/// 段内写数不超过这个数时，子集掩码直接用一个 u64 抽；更长的段逐写各抽一次。
+const WRITES_A_SUBSET_MASK_HOLDS: usize = 63;
+
 /// 工作线程数是从哪来的：与实际起的线程数一起打进进度行，「机器多于 1 核却只用了 1 个线程」看得出来。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum CrashInjectionWorkerThreads {
@@ -125,20 +158,80 @@
     }
 }
 
-/// 一个崩溃点：截在录制流的第几步（那一步是一次写）、那次写落在盘上的哪一区（种类）、它是历史哪一步里发出的。
+/// 一段历史上摆几个崩溃状态、怎么挑。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CrashPointDraw {
+    /// 按种子抽这么多个（每个是「一段 + 那一段的一个真子集」，同一段同一个子集只摆一次）。
+    Sampled { crash_points_per_history: usize },
+    /// 写数不超过 `segment_writes` 的段整段枚举（全部真子集），更长的段按种子各抽 `sampled_in_longer_segments` 个：
+    /// 写死的那几段历史靠它把小段摆全，判别力不靠运气。
+    EveryProperSubsetOfShortSegments {
+        segment_writes: usize,
+        sampled_in_longer_segments: usize,
+    },
+}
+
+/// 失败时那份崩溃镜像留不留。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum FailureImageRetention {
+    /// 留着，路径打进报告（探索档／大档：现场要能重看）。
+    KeepTheImageFiles,
+    /// 整批跑完就把这一批的镜像目录删掉（门禁档不在临时目录里留垃圾）。
+    DeleteTheImageFilesWhenTheRunFinishes,
+}
+
+/// 一个崩溃状态：截在录制流的第几段、那一段里哪几个写持久了。
+/// 更早的段整段持久、更晚的段一个都没持久；这一段里持久的是一个真子集（全持久等于「没崩在这一段」，那个状态由下一段的空子集摆）。
+#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
 pub struct CrashPoint {
-    /// 录制流里的下标：`[0, stream_index]` 这一段持久，之后的都没落盘。
-    pub stream_index: usize,
-    pub truncated_write_kind: StepKind,
+    pub segment_index: usize,
+    /// 这一段里第 k 个写持久了没有（长度就是这一段的写数）。
+    pub persisted_within_the_segment: Vec<bool>,
+    /// 这一段里第一个没持久的写落在历史的哪一步里。
     pub step: StepPosition,
-    /// 崩溃点落在哪一类操作里；落在起点里的是 None（起点整个持久，抽不到）。
+    /// 那一步是哪一类操作；落在起点里的是 None。
     pub operation_kind: Option<HistoryOperationKind>,
-    /// 这一步里、崩溃点之后还有根槽写：这次截断把一次发布截在它的根落盘之前。
-    pub truncates_a_publish_before_its_root: bool,
 }
 
-/// 一段历史上一类崩溃点失败。
+impl CrashPoint {
+    /// 段内持久的那几个写不是这一段的前缀：有一个写被扣下了，它后面却有写持久了
+    /// （「后发的写先持久」那一类状态，只截前缀摆不出来）。
+    #[must_use]
+    pub fn withholds_a_write_before_a_persisted_one(&self) -> bool {
+        self.persisted_within_the_segment
+            .iter()
+            .position(|persisted| !persisted)
+            .is_some_and(|withheld| {
+                self.persisted_within_the_segment[withheld..]
+                    .iter()
+                    .any(|persisted| *persisted)
+            })
+    }
+
+    /// 给人看：第几段、段内持久了几个写、扣下的写在段内的下标。
+    #[must_use]
+    pub fn render(&self) -> String {
+        let withheld: Vec<usize> = self
+            .persisted_within_the_segment
+            .iter()
+            .enumerate()
+            .filter(|(_, persisted)| !**persisted)
+            .map(|(index, _)| index)
+            .collect();
+        format!(
+            "第 {} 段（{} 个写，持久 {} 个，扣下段内第 {:?} 个）",
+            self.segment_index,
+            self.persisted_within_the_segment.len(),
+            self.persisted_within_the_segment
+                .iter()
+                .filter(|persisted| **persisted)
+                .count(),
+            withheld
+        )
+    }
+}
+
+/// 一段历史上一类崩溃状态失败。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct CrashPointFinding {
     pub signature: FailureSignature,
@@ -147,6 +240,8 @@
     pub observation: FailureObservation,
     /// 恢复读回了什么（给人看）。
     pub read_back: String,
+    /// 这份崩溃镜像落成文件之后放在哪；没落盘（不留现场，或写文件没成）时是 None。
+    pub image_files: Option<PathBuf>,
 }
 
 impl CrashPointFinding {
@@ -155,11 +250,10 @@
         let mut text = String::new();
         let _ = writeln!(
             text,
-            "崩溃点上的新发现 {:?}：种子 {}，截在录制流第 {} 步（{}，{:?}／{:?}）",
+            "崩溃状态上的新发现 {:?}：种子 {}，{}，落在 {:?}／{:?}",
             self.signature,
             self.seed.0,
-            self.crash_point.stream_index,
-            self.crash_point.truncated_write_kind.name(),
+            self.crash_point.render(),
             self.crash_point.step,
             self.crash_point.operation_kind
         );
@@ -167,6 +261,9 @@
         for (invariant, detail) in &self.observation.violations {
             let _ = writeln!(text, "  {invariant}：{detail}");
         }
+        for aspect in record_check_aspects(&self.observation.record_check) {
+            let _ = writeln!(text, "  记录核对器判红：{aspect}");
+        }
         if let Some(panic) = &self.observation.panic {
             let _ = writeln!(text, "  panic 在 {}：{}", panic.location, panic.message);
         }
@@ -179,6 +276,9 @@
                 disagreement.implementation_answer
             );
         }
+        if let Some(directory) = &self.image_files {
+            let _ = writeln!(text, "  崩溃镜像留在：{}", directory.display());
+        }
         let _ = writeln!(
             text,
             "  复现：SINGLEFS_CRASH_INJECTION_FIRST_SEED={} SINGLEFS_CRASH_INJECTION_SEEDS=1 跑大档那条 #[ignore] 用例",
@@ -188,42 +288,50 @@
     }
 }
 
-/// 崩溃点上以「已知红」收尾的一次：种子、清单第几条、哪个崩溃点。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+/// 崩溃状态上以「已知红」收尾的一次：种子、清单第几条、哪个崩溃状态。
+#[derive(Clone, Debug, PartialEq, Eq)]
 pub struct KnownRedAtACrashPoint {
     pub seed: HistorySeed,
     pub form: usize,
     pub crash_point: CrashPoint,
 }
 
-/// 跑过的崩溃点的计数：绝对数，证明各条路径真的跑到了（`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
+/// 跑过的崩溃状态的计数：绝对数，证明各条路径真的跑到了（`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
 #[derive(Clone, Debug, Default, PartialEq, Eq)]
 pub struct CrashInjectionTally {
     pub histories: u64,
-    /// 这段历史自己怎么收尾（崩溃点之外）。
+    /// 这段历史自己怎么收尾（崩溃状态之外）。
     pub histories_completed: u64,
     pub histories_ended_known_red: BTreeMap<usize, u64>,
     pub histories_ended_new_finding: u64,
-    /// 起点之后一次写都没有、一个崩溃点都抽不到的历史段数。
+    /// 一个崩溃状态都摆不出的历史段数（流里一个候选段都没有）。
     pub histories_without_any_crash_point: u64,
     pub crash_points: u64,
-    /// 按截断的那次写的种类分（四种落点都抽到过才算罩全）。
-    pub crash_points_by_write_kind: BTreeMap<&'static str, u64>,
-    /// 按崩溃点落在哪一类操作里分（固定脚本只有一条路，这里证明挂载、回退、抬 F、冷启动的写上都截过）。
+    /// 按段内被扣下的写的种类分（四种落点都扣下过才算罩全）；一个崩溃状态扣下几种就各记一次。
+    pub crash_points_withholding_write_kind: BTreeMap<&'static str, u64>,
+    /// 按崩溃状态落在哪一类操作里分（固定脚本只有一条路，这里证明挂载、回退、抬 F、冷启动的写上都摆过）。
     pub crash_points_by_operation_kind: BTreeMap<HistoryOperationKind, u64>,
+    /// 崩溃状态落在起点那一段里（起点段 2026-09-20 才放开，这个数证明真的摆到了）。
+    pub crash_points_inside_the_starting_point: u64,
+    /// 段内持久的不是前缀：「后发的写先持久」那一类状态摆出来了几个。
+    pub crash_points_withholding_a_write_before_a_persisted_one: u64,
     /// 截断之后盘上已持久的最新根槽，比这段历史最后提交的那一版旧：真的把状态截回了过去。
     pub crash_points_that_land_before_the_last_committed_version: u64,
-    /// 截断落在一次发布的中间：这次写之后、这一步里还有根槽写没落盘。
+    /// 一次发布已经写过东西而它的根槽还没持久。
     pub crash_points_inside_an_unfinished_publish: u64,
     pub recoveries_reading_a_file: u64,
     pub recoveries_without_a_file: u64,
     pub recoveries_failed: u64,
-    /// 恢复真的施加过 journal 记录前缀的崩溃点数。
+    /// 恢复真的施加过 journal 记录前缀的崩溃状态数。
     pub recoveries_that_applied_journal_records: u64,
     pub checker_runs: u64,
     pub invariant_holds: BTreeMap<&'static str, u64>,
     pub invariant_not_applicable: BTreeMap<&'static str, u64>,
-    /// 问过模型「恢复到的这一版允不允许」的次数（每个崩溃点一次）。
+    /// 记录核对器跑过几次、两类各判红几次（与层 0 那一路同一份判据，`crash::check_records_against`）。
+    pub record_checks: u64,
+    pub record_root_without_record: u64,
+    pub record_claimed_state_missing_unit: u64,
+    /// 问过模型「恢复到的这一版允不允许」的次数（每个崩溃状态一次）。
     pub model_judgements: u64,
     /// 模型认得择到的那条根、并真的逐字节比过内容的次数。
     pub model_contents_compared: u64,
@@ -245,8 +353,10 @@
             histories_ended_new_finding,
             histories_without_any_crash_point,
             crash_points,
-            crash_points_by_write_kind,
+            crash_points_withholding_write_kind,
             crash_points_by_operation_kind,
+            crash_points_inside_the_starting_point,
+            crash_points_withholding_a_write_before_a_persisted_one,
             crash_points_that_land_before_the_last_committed_version,
             crash_points_inside_an_unfinished_publish,
             recoveries_reading_a_file,
@@ -256,6 +366,9 @@
             checker_runs,
             invariant_holds,
             invariant_not_applicable,
+            record_checks,
+            record_root_without_record,
+            record_claimed_state_missing_unit,
             model_judgements,
             model_contents_compared,
             model_versions_without_a_file_matched,
@@ -271,8 +384,11 @@
         self.histories_ended_new_finding += histories_ended_new_finding;
         self.histories_without_any_crash_point += histories_without_any_crash_point;
         self.crash_points += crash_points;
-        for (kind, count) in crash_points_by_write_kind {
-            *self.crash_points_by_write_kind.entry(kind).or_insert(0) += count;
+        for (kind, count) in crash_points_withholding_write_kind {
+            *self
+                .crash_points_withholding_write_kind
+                .entry(kind)
+                .or_insert(0) += count;
         }
         for (kind, count) in crash_points_by_operation_kind {
             *self
@@ -280,6 +396,9 @@
                 .entry(*kind)
                 .or_insert(0) += count;
         }
+        self.crash_points_inside_the_starting_point += crash_points_inside_the_starting_point;
+        self.crash_points_withholding_a_write_before_a_persisted_one +=
+            crash_points_withholding_a_write_before_a_persisted_one;
         self.crash_points_that_land_before_the_last_committed_version +=
             crash_points_that_land_before_the_last_committed_version;
         self.crash_points_inside_an_unfinished_publish += crash_points_inside_an_unfinished_publish;
@@ -294,6 +413,9 @@
         for (invariant, count) in invariant_not_applicable {
             *self.invariant_not_applicable.entry(invariant).or_insert(0) += count;
         }
+        self.record_checks += record_checks;
+        self.record_root_without_record += record_root_without_record;
+        self.record_claimed_state_missing_unit += record_claimed_state_missing_unit;
         self.model_judgements += model_judgements;
         self.model_contents_compared += model_contents_compared;
         self.model_versions_without_a_file_matched += model_versions_without_a_file_matched;
@@ -306,13 +428,13 @@
         self.crash_states_ending_new_finding += crash_states_ending_new_finding;
     }
 
-    /// 给人看的一整块：绝对数——抽了几个崩溃点、恢复成了几次、checker 跑了几次、模型判了几次、新发现几条。
+    /// 给人看的一整块：绝对数——摆了几个崩溃状态、恢复成了几次、checker 与记录核对器跑了几次、模型判了几次、新发现几条。
     #[must_use]
     pub fn render(&self) -> String {
         let mut text = String::new();
         let _ = writeln!(
             text,
-            "历史 {} 段：跑完 {}、以已知红收尾 {:?}、新发现 {}；一个崩溃点都抽不到的 {} 段；模型目录最多 {} 版",
+            "历史 {} 段：跑完 {}、以已知红收尾 {:?}、新发现 {}；一个崩溃状态都摆不出的 {} 段；模型目录最多 {} 版",
             self.histories,
             self.histories_completed,
             self.histories_ended_known_red,
@@ -322,13 +444,15 @@
         );
         let _ = writeln!(
             text,
-            "崩溃点 {} 个：截在发布中间（根还没落盘）的 {} 个、截回到最后一版之前的 {} 个",
+            "崩溃状态 {} 个：截在发布中间（根还没落盘）的 {} 个、截回到最后一版之前的 {} 个、段内有洞（后发的写先持久）的 {} 个、落在起点那一段里的 {} 个",
             self.crash_points,
             self.crash_points_inside_an_unfinished_publish,
-            self.crash_points_that_land_before_the_last_committed_version
+            self.crash_points_that_land_before_the_last_committed_version,
+            self.crash_points_withholding_a_write_before_a_persisted_one,
+            self.crash_points_inside_the_starting_point
         );
-        for (kind, count) in &self.crash_points_by_write_kind {
-            let _ = writeln!(text, "  截断的那次写是 {kind}：{count} 个");
+        for (kind, count) in &self.crash_points_withholding_write_kind {
+            let _ = writeln!(text, "  段内扣下过 {kind}：{count} 个");
         }
         for kind in HistoryOperationKind::ALL {
             let _ = writeln!(
@@ -350,8 +474,15 @@
         );
         let _ = writeln!(
             text,
-            "崩溃点上 checker 跑了 {} 次；问模型 {} 次：比过内容 {} 次、树表 0 条对上 {} 次",
+            "崩溃状态上 checker 跑了 {} 次；记录核对器跑了 {} 次：根在而记录一条都不在 {} 次、恢复自称新态而单元缺席 {} 次",
             self.checker_runs,
+            self.record_checks,
+            self.record_root_without_record,
+            self.record_claimed_state_missing_unit
+        );
+        let _ = writeln!(
+            text,
+            "问模型 {} 次：比过内容 {} 次、树表 0 条对上 {} 次",
             self.model_judgements,
             self.model_contents_compared,
             self.model_versions_without_a_file_matched
@@ -380,38 +511,51 @@
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct HistoryCrashInjection {
     pub seed: HistorySeed,
-    /// 抽到的崩溃点，按录制流里的次序。
+    /// 摆出来的崩溃状态，按段号从小到大。
     pub crash_points: Vec<CrashPoint>,
     pub tally: CrashInjectionTally,
     pub known_red_hits: Vec<KnownRedAtACrashPoint>,
-    /// 按崩溃点从小到大排；同一段历史里同一个签名只留第一个。
+    /// 按段号从小到大排；同一段历史里同一个签名只留第一个。
     pub new_findings: Vec<CrashPointFinding>,
 }
 
-/// 一段历史跑到哪一步、录制流到那时有多长：崩溃点只抽在这段区间里。
+/// 一段历史跑到哪一步、录制流到那时有多长：崩溃状态只摆在这段区间里。
 struct StreamMarks {
-    /// 起点跑完时流里有几步（崩溃点从它起）。
-    after_the_starting_point: usize,
-    /// 最后一个跑完的操作之后流里有几步（崩溃点到它为止）。
+    /// mkfs 占了流开头的几步（崩溃状态从它之后起）。
+    after_make_filesystem: usize,
+    /// 最后一个跑完的操作之后流里有几步（崩溃状态到它为止；失败那一步的写不摆）。
     after_the_last_finished_step: usize,
-    /// 每一步跑完时流里有几步，按次序（崩溃点落在哪一步靠它定）。
+    /// 每一步跑完时流里有几步，按次序（崩溃状态落在哪一步靠它定）。
     after_each_step: Vec<(StepPosition, usize)>,
 }
 
-/// 跑一段历史，按种子在它的录制流上抽 `crash_points_per_history` 个崩溃点，逐个重建镜像、跑恢复与池级 checker、问模型。
+impl StreamMarks {
+    /// 录制流里第 `stream_index` 步落在哪一步操作里：第一个「跑完时流里的步数」大于它的那一条登记。
+    /// 登记里找不到（在最后一个跑完的操作之后）时是 None。
+    fn step_containing(&self, stream_index: usize) -> Option<StepPosition> {
+        self.after_each_step
+            .iter()
+            .find(|(_, operations_so_far)| *operations_so_far > stream_index)
+            .map(|(position, _)| *position)
+    }
+}
+
+/// 跑一段历史，按 `draw` 在它的录制流上摆崩溃状态，逐个重建镜像、跑恢复、池级 checker 与记录核对器、问模型。
+/// `image_directory` 给了就把判红的那几个崩溃状态落成镜像文件放进去（留现场）。
 ///
 /// # Panics
-/// 录制流没开内容保留（`MemoryPool::apply` 要字节）——这里自己建流，走不到。
+/// 录制流没开内容保留（重建镜像要字节）——这里自己建流，走不到。
 #[must_use]
 pub fn inject_crashes_into_history(
     history: &GeneratedHistory,
     execution: HistoryExecution,
-    crash_points_per_history: usize,
+    draw: CrashPointDraw,
+    image_directory: Option<&Path>,
 ) -> HistoryCrashInjection {
     let stream = SharedStream::retaining_contents();
     let mut committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
     let mut marks = StreamMarks {
-        after_the_starting_point: 0,
+        after_make_filesystem: 0,
         after_the_last_finished_step: 0,
         after_each_step: Vec::new(),
     };
@@ -420,15 +564,12 @@
             committed_versions.insert(key, file);
         }
         let operations_so_far = stream.operation_count();
-        match observation.position {
-            StepPosition::StartingPoint => marks.after_the_starting_point = operations_so_far,
-            StepPosition::Operation(_) => {}
-        }
         marks.after_the_last_finished_step = operations_so_far;
         marks
             .after_each_step
             .push((observation.position, operations_so_far));
     });
+    marks.after_make_filesystem = run.operations_written_by_make_filesystem;
 
     let mut tally = CrashInjectionTally {
         histories: 1,
@@ -449,53 +590,85 @@
         fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
         journal_ring_bytes: parameters.geometry.journal_ring_bytes,
     };
+    let (writes, segments, stream_indexes) =
+        writes_and_segments_with_stream_indexes(&operations, &geometry);
+    let step_kinds: Vec<HistoryOperationKind> = history
+        .operations
+        .iter()
+        .map(HistoryOperation::kind)
+        .collect();
     let crash_points = draw_crash_points(
-        history,
-        &operations,
+        history.seed,
+        &segments,
+        &stream_indexes,
         &marks,
-        &geometry,
-        crash_points_per_history,
+        &step_kinds,
+        draw,
     );
     if crash_points.is_empty() {
         tally.histories_without_any_crash_point = 1;
     }
     let last_committed_version = committed_versions.keys().next_back().copied();
 
-    let mut image = MemoryPool::with_devices(
+    let mut base = MemoryPool::with_devices(
         &[DeviceIdentity(0), DeviceIdentity(1)],
         execution.device_width.device_bytes(),
     );
-    let mut applied_operations = 0usize;
-    let mut newest_persisted_root: Option<ModelRootKey> = None;
+    let mut writes_applied_to_the_base = 0usize;
     let mut known_red_hits = Vec::new();
     let mut new_findings: Vec<CrashPointFinding> = Vec::new();
-    for crash_point in crash_points.iter().copied() {
-        // 崩溃点从小到大，镜像只往前叠：整条流重放一遍就够，不为每个崩溃点从头重建。
-        let newly_persisted = &operations[applied_operations..=crash_point.stream_index];
-        image.apply(newly_persisted);
-        for retained in newly_persisted {
-            if let Some(root) = root_written_by(retained, &geometry) {
-                newest_persisted_root = newest_persisted_root.max(Some(root));
-            }
-        }
-        applied_operations = crash_point.stream_index + 1;
+    for crash_point in &crash_points {
+        let segment = &segments[crash_point.segment_index];
+        let first_write_of_the_segment = segment[0];
+        let writes_in_the_segment = crash_point.persisted_within_the_segment.len();
+        // 段号从小到大，基线只往前叠：更早的段整段持久，按写表整段施加，不为每个崩溃状态从头重建。
+        base.apply_writes(&writes[writes_applied_to_the_base..first_write_of_the_segment]);
+        writes_applied_to_the_base = first_write_of_the_segment;
+        let writes_up_to_this_segment =
+            &writes[..first_write_of_the_segment + writes_in_the_segment];
+        let mut persisted = vec![false; writes.len()];
+        persisted[..first_write_of_the_segment].fill(true);
+        for (within_the_segment, is_persisted) in
+            crash_point.persisted_within_the_segment.iter().enumerate()
+        {
+            persisted[first_write_of_the_segment + within_the_segment] = *is_persisted;
+        }
+        let image = CrashImage {
+            base: &base,
+            writes: &writes
+                [first_write_of_the_segment..first_write_of_the_segment + writes_in_the_segment],
+            persisted: crash_point.persisted_within_the_segment.clone(),
+        };
+
         tally.crash_points += 1;
-        *tally
-            .crash_points_by_write_kind
-            .entry(crash_point.truncated_write_kind.name())
-            .or_insert(0) += 1;
+        for kind in withheld_write_kinds(&writes, segment, crash_point) {
+            *tally
+                .crash_points_withholding_write_kind
+                .entry(kind.name())
+                .or_insert(0) += 1;
+        }
+        match crash_point.step {
+            StepPosition::StartingPoint => tally.crash_points_inside_the_starting_point += 1,
+            StepPosition::Operation(_) => {}
+        }
         if let Some(kind) = crash_point.operation_kind {
             *tally
                 .crash_points_by_operation_kind
                 .entry(kind)
                 .or_insert(0) += 1;
         }
-        if newest_persisted_root < last_committed_version {
+        if crash_point.withholds_a_write_before_a_persisted_one() {
+            tally.crash_points_withholding_a_write_before_a_persisted_one += 1;
+        }
+        let newest_persisted = newest_persisted_root(&writes, &persisted)
+            .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
+        if newest_persisted < last_committed_version {
             tally.crash_points_that_land_before_the_last_committed_version += 1;
         }
-        if crash_point.truncates_a_publish_before_its_root {
+        if some_publish_persisted_without_its_root(&writes, &persisted) {
             tally.crash_points_inside_an_unfinished_publish += 1;
         }
+
         let report = recover(&image, JournalPolicy::Consult);
         match &report.outcome {
             RecoveryOutcome::FileRead { .. } => tally.recoveries_reading_a_file += 1,
@@ -508,7 +681,7 @@
         let read_back = observed_read_back_after_a_crash(&report);
         tally.model_judgements += 1;
         let disagreement =
-            crash_recovery_disagreement(&committed_versions, newest_persisted_root, &read_back);
+            crash_recovery_disagreement(&committed_versions, newest_persisted, &read_back);
         if disagreement.is_none() {
             match &read_back {
                 ObservedReadBack::FileRead { .. } => tally.model_contents_compared += 1,
@@ -518,12 +691,29 @@
                 ObservedReadBack::Failed { .. } => {}
             }
         }
+        // 记录核对器（层 0 那一路同一份判据）：读的是这份崩溃镜像，核的是到这一段为止的整条记录流——
+        // 更早的段里发出的单元写与 journal 记录写也要算进一次发布里，只给当前段就核不出跨段的那几次发布。
+        let record_check =
+            check_records_against(&image, writes_up_to_this_segment, report.effective_root);
+        tally.record_checks += 1;
+        if record_check.root_without_record {
+            tally.record_root_without_record += 1;
+        }
+        if record_check.claimed_state_missing_unit {
+            tally.record_claimed_state_missing_unit += 1;
+        }
         let violations = checker_violations_on(&image, &mut tally);
-        if violations.is_empty() && disagreement.is_none() {
+        if violations.is_empty() && disagreement.is_none() && record_check == RecordCheck::default()
+        {
             continue;
         }
-        let observation =
-            crash_state_observation(&image, crash_point, violations, disagreement.clone());
+        let observation = crash_state_observation(
+            &image,
+            crash_point,
+            violations,
+            disagreement.clone(),
+            record_check,
+        );
         match classify_failure(observation) {
             HistoryEnding::Completed => {}
             HistoryEnding::KnownRed { form, .. } => {
@@ -531,7 +721,7 @@
                 known_red_hits.push(KnownRedAtACrashPoint {
                     seed: history.seed,
                     form,
-                    crash_point,
+                    crash_point: crash_point.clone(),
                 });
             }
             HistoryEnding::NewFinding {
@@ -543,12 +733,16 @@
                     .iter()
                     .any(|finding| finding.signature == signature)
                 {
+                    let image_files = image_directory.and_then(|directory| {
+                        write_crash_image_files(&image, directory, history.seed, crash_point)
+                    });
                     new_findings.push(CrashPointFinding {
                         signature,
                         seed: history.seed,
-                        crash_point,
+                        crash_point: crash_point.clone(),
                         observation,
                         read_back: format!("{read_back:?}"),
+                        image_files,
                     });
                 }
             }
@@ -563,81 +757,133 @@
     }
 }
 
-/// 一次录制的写写下了哪条根（落在根环里的就是根槽写）；不是根槽写的是 None。
-fn root_written_by(retained: &RetainedOperation, geometry: &FixedGeometry) -> Option<ModelRootKey> {
-    if geometry.classify(&retained.operation) != StepKind::RootRecordFua {
-        return None;
-    }
-    let bytes = retained
-        .contents
-        .as_ref()
-        .expect("崩溃注入的录制流开了内容保留");
-    let (instance, checkpoint_txg) = root_identity_written_by(bytes);
-    Some(model_root_key(instance, checkpoint_txg))
-}
-
-/// 录制流里第 `stream_index` 步落在哪一步操作里、那一步跑完时流里有几步：第一个「跑完时流里的步数」大于它的那一条登记。
-/// 登记里找不到（崩溃点在最后一个跑完的操作之后，抽不到）时是 None。
-fn step_containing(stream_index: usize, marks: &StreamMarks) -> Option<(StepPosition, usize)> {
-    marks
-        .after_each_step
-        .iter()
-        .find(|(_, operations_so_far)| *operations_so_far > stream_index)
-        .copied()
+/// 这个崩溃状态在段内扣下的写各是什么种类（同一种只记一次）。
+fn withheld_write_kinds(
+    writes: &[RetainedWrite],
+    segment: &[usize],
+    crash_point: &CrashPoint,
+) -> Vec<StepKind> {
+    let mut kinds: BTreeSet<StepKind> = BTreeSet::new();
+    for (within_the_segment, is_persisted) in
+        crash_point.persisted_within_the_segment.iter().enumerate()
+    {
+        if !is_persisted {
+            kinds.insert(writes[segment[within_the_segment]].kind);
+        }
+    }
+    kinds.into_iter().collect()
 }
 
-/// 按种子抽崩溃点：候选是起点之后、最后一个跑完的操作为止那段流里的写（屏障不是写，截在屏障上与截在它前一个写之后同一个状态）。
-/// 候选不够就全要。抽法是部分 Fisher–Yates 洗牌（同一个种子逐项复现），抽完按流里的次序排好——镜像只往前叠。
+/// 按 `draw` 摆崩溃状态：候选段是「整段的写都落在 mkfs 之后、最后一个跑完的操作之前」的那几段，段内取真子集。
+///
+/// 起点那一段（取号、暖机、第一个文件）在内——2026-09-20 才放开；mkfs 那几段不在内，与层 0 同一条界：
+/// mkfs 不是事务，它写到一半的盘面上还没有池，恢复报不出根是对的，拿事务的 oracle 去判只会判出假红
+/// （层 0 也从 `mkfs_operation_count` 之后起枚举）。失败那一步的写也不在内（模型还没判过它写出的根）。
+///
+/// 抽的那一路同一段同一个子集只摆一次；摆完按段号排好——基线只往前叠。
 fn draw_crash_points(
-    history: &GeneratedHistory,
-    operations: &[RetainedOperation],
+    seed: HistorySeed,
+    segments: &[Vec<usize>],
+    stream_indexes: &[usize],
     marks: &StreamMarks,
-    geometry: &FixedGeometry,
-    crash_points_per_history: usize,
+    step_kinds: &[HistoryOperationKind],
+    draw: CrashPointDraw,
 ) -> Vec<CrashPoint> {
-    let mut candidates: Vec<usize> = (marks.after_the_starting_point
-        ..marks.after_the_last_finished_step.min(operations.len()))
-        .filter(|index| operations[*index].operation.kind != RecordedOperationKind::Barrier)
+    let candidate_segments: Vec<usize> = (0..segments.len())
+        .filter(|segment_index| {
+            segments[*segment_index].iter().all(|write| {
+                (marks.after_make_filesystem..marks.after_the_last_finished_step)
+                    .contains(&stream_indexes[*write])
+            })
+        })
         .collect();
-    let drawn = crash_points_per_history.min(candidates.len());
-    let mut source = SeededRandomSource::from_seed(history.seed.0 ^ CRASH_POINT_SEED_SALT);
-    for position in 0..drawn {
-        let remaining = u64::try_from(candidates.len() - position).expect("候选数装得进 u64");
-        let picked = position + usize::try_from(source.below(remaining)).expect("下标装得进 usize");
-        candidates.swap(position, picked);
+    if candidate_segments.is_empty() {
+        return Vec::new();
+    }
+    let mut source = SeededRandomSource::from_seed(seed.0 ^ CRASH_POINT_SEED_SALT);
+    let mut chosen: BTreeSet<(usize, Vec<bool>)> = BTreeSet::new();
+    match draw {
+        CrashPointDraw::Sampled {
+            crash_points_per_history,
+        } => {
+            for _ in 0..crash_points_per_history {
+                let picked = usize::try_from(
+                    source.below(u64::try_from(candidate_segments.len()).expect("候选段数")),
+                )
+                .expect("下标装得进 usize");
+                let segment_index = candidate_segments[picked];
+                let subset = draw_a_proper_subset(&mut source, segments[segment_index].len());
+                chosen.insert((segment_index, subset));
+            }
+        }
+        CrashPointDraw::EveryProperSubsetOfShortSegments {
+            segment_writes,
+            sampled_in_longer_segments,
+        } => {
+            for segment_index in candidate_segments {
+                let length = segments[segment_index].len();
+                if length <= segment_writes {
+                    for mask in 0..(1u64 << length) - 1 {
+                        chosen.insert((segment_index, bits_of(mask, length)));
+                    }
+                } else {
+                    for _ in 0..sampled_in_longer_segments {
+                        let subset = draw_a_proper_subset(&mut source, length);
+                        chosen.insert((segment_index, subset));
+                    }
+                }
+            }
+        }
     }
-    let mut chosen: Vec<usize> = candidates[..drawn].to_vec();
-    chosen.sort_unstable();
     chosen
         .into_iter()
-        .map(|stream_index| {
-            let (step, step_ends_at) = step_containing(stream_index, marks)
-                .expect("崩溃点抽在最后一个跑完的操作之前，登记里一定找得到它那一步");
+        .map(|(segment_index, persisted_within_the_segment)| {
+            let first_withheld = persisted_within_the_segment
+                .iter()
+                .position(|persisted| !persisted)
+                .expect("段内持久的是真子集，至少有一个写被扣下");
+            let step = marks
+                .step_containing(stream_indexes[segments[segment_index][first_withheld]])
+                .expect("崩溃状态摆在最后一个跑完的操作之前，登记里一定找得到它那一步");
             let operation_kind = match step {
                 StepPosition::StartingPoint => None,
-                StepPosition::Operation(step_index) => history
-                    .operations
-                    .get(step_index)
-                    .map(HistoryOperation::kind),
+                StepPosition::Operation(step_index) => step_kinds.get(step_index).copied(),
             };
             CrashPoint {
-                stream_index,
-                truncated_write_kind: geometry.classify(&operations[stream_index].operation),
+                segment_index,
+                persisted_within_the_segment,
                 step,
                 operation_kind,
-                truncates_a_publish_before_its_root: operations
-                    [stream_index + 1..step_ends_at.min(operations.len())]
-                    .iter()
-                    .any(|retained| root_written_by(retained, geometry).is_some()),
             }
         })
         .collect()
 }
 
+/// 掩码的低 `length` 位摊成逐写的持久与否。
+fn bits_of(mask: u64, length: usize) -> Vec<bool> {
+    (0..length).map(|bit| mask & (1 << bit) != 0).collect()
+}
+
+/// 抽一个真子集：段短时直接抽一个掩码（[0, 2^n − 1) 不含全持久），段长时逐写各抽一次、全中了再去掉一个。
+/// 真子集这一条要守住：全持久等于「没崩在这一段」，那个状态由下一段的空子集摆。
+fn draw_a_proper_subset(source: &mut SeededRandomSource, segment_length: usize) -> Vec<bool> {
+    if segment_length <= WRITES_A_SUBSET_MASK_HOLDS {
+        let mask = source.below((1u64 << segment_length) - 1);
+        return bits_of(mask, segment_length);
+    }
+    let mut persisted: Vec<bool> = (0..segment_length).map(|_| source.below(2) == 1).collect();
+    if persisted.iter().all(|is_persisted| *is_persisted) {
+        let withheld = usize::try_from(source.below(u64::try_from(segment_length).expect("段长")))
+            .expect("下标装得进 usize");
+        persisted[withheld] = false;
+    }
+    persisted
+}
+
 /// 对一份崩溃镜像跑池级 checker，记每条不变量判绿、不适用各几次，交回判红的那几条（与 `history.rs` 的 `violations_on` 同一条口径，
 /// 计数进的是崩溃注入自己的那一份）。
 fn checker_violations_on(
-    image: &MemoryPool,
+    image: &CrashImage<'_>,
     tally: &mut CrashInjectionTally,
 ) -> Vec<(&'static str, String)> {
     tally.checker_runs += 1;
@@ -654,16 +900,27 @@
     violations
 }
 
+/// 这个崩溃镜像上，最新那条根带的回退下界 F 落不落在回退留下的空档里（F 那个 txg 上的根全属于被抛弃的实例）。
+/// 与活盘面那一路（抬 F 之前的镜像 + 新 F）是同一个谓词、同一份实现，读的盘面不同：这里读崩溃镜像自己，
+/// F 从镜像里最新那条根上取——抬 F 之后的任何一个崩溃状态都摆得出同一机理的盘面，不限于抬 F 那一步
+/// （代码三方 m2-supp3-item3-code-r1 判决 K6 的假阳那一半：此前这里写死成 None，收口表第 43 行那一形在崩溃状态上恒不匹配）。
+fn raised_floor_of_the_newest_root_lands_only_on_abandoned_roots(
+    image: &CrashImage<'_>,
+) -> Option<bool> {
+    let system_configuration = choose_system_configuration(image).ok()?;
+    let newest_root = choose_root(image, &system_configuration)?;
+    raised_floor_lands_only_on_abandoned_roots(image, newest_root.rollback_floor)
+}
+
 /// 一个崩溃状态的失败观察：拿去对「已知红」清单（`KNOWN_RED_FORMS`，与历史那一路同一张清单）。
-/// 抬 F 那一格（第 1 条）要的「新 F 落在回退留下的空档里」在崩溃状态上不算：那一条判的是抬 F 这一步之后的活盘面，
-/// 这里留 None，崩溃状态上的 I-3.1 只由第 0 条（根环转过一圈）接得走。
 fn crash_state_observation(
-    image: &MemoryPool,
-    crash_point: CrashPoint,
+    image: &CrashImage<'_>,
+    crash_point: &CrashPoint,
     violations: Vec<(&'static str, String)>,
     model_disagreement: Option<ModelDisagreement>,
+    record_check: RecordCheck,
 ) -> FailureObservation {
-    let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count(image);
+    let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count_of(image);
     FailureObservation {
         position: crash_point.step,
         operation_kind: crash_point.operation_kind,
@@ -673,8 +930,53 @@
         root_ring_slot_count,
         harness_judgement: None,
         model_disagreement,
-        raised_floor_lands_only_on_abandoned_roots: None,
+        raised_floor_lands_only_on_abandoned_roots:
+            raised_floor_of_the_newest_root_lands_only_on_abandoned_roots(image),
+        record_check,
+    }
+}
+
+/// 把一个崩溃状态落成镜像文件（一块盘一个稀疏文件）：只写这个状态里有内容的那些扇区，没写过的留成洞、读出全 0，
+/// 与内存里那份逐字节相同。写不出来（目录建不出来、盘满）时交回 None：留现场是为了好查，不是判据，不因为它红。
+fn write_crash_image_files(
+    image: &CrashImage<'_>,
+    directory: &Path,
+    seed: HistorySeed,
+    crash_point: &CrashPoint,
+) -> Option<PathBuf> {
+    use std::os::unix::fs::FileExt as _;
+    let here = directory.join(format!(
+        "seed-{}-segment-{}",
+        seed.0, crash_point.segment_index
+    ));
+    std::fs::create_dir_all(&here).ok()?;
+    let sector_length = usize::try_from(SECTOR_BYTES).expect("512");
+    for device in image.base.devices.keys().copied() {
+        let file = std::fs::File::create(here.join(format!("device-{}.img", device.0))).ok()?;
+        file.set_len(image.base.device_size_in_bytes).ok()?;
+        let mut sectors: BTreeSet<u64> = image
+            .base
+            .devices
+            .get(&device)
+            .expect("这块盘在基线里")
+            .written_sectors_in(DeviceOffsetInBytes(0), image.base.device_size_in_bytes)
+            .into_iter()
+            .collect();
+        for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
+            if !is_persisted || write.device != device {
+                continue;
+            }
+            let first_sector = write.offset.0 / SECTOR_BYTES;
+            let sector_count = u64::try_from(write.bytes.len()).expect("写长") / SECTOR_BYTES;
+            sectors.extend(first_sector..first_sector + sector_count);
+        }
+        for sector in sectors {
+            let offset = DeviceOffsetInBytes(sector * SECTOR_BYTES);
+            let bytes = PoolReader::read(image, device, offset, sector_length)?;
+            file.write_all_at(&bytes, offset.0).ok()?;
+        }
     }
+    Some(here)
 }
 
 /// 一批种子跑下来的崩溃注入报告。
@@ -683,40 +985,60 @@
     pub first_seed: u64,
     pub seed_count: u64,
     pub operations_per_history: usize,
-    pub crash_points_per_history: usize,
+    pub draw: CrashPointDraw,
     pub weights: GenerationWeights,
     pub execution: HistoryExecution,
     pub tally: CrashInjectionTally,
     pub known_red_hits: Vec<KnownRedAtACrashPoint>,
     /// 按种子从小到大排；同一个签名只留第一个种子。
     pub new_findings: Vec<CrashPointFinding>,
+    /// 这一批判红的崩溃镜像留在哪；跑完删掉的那一档是 None。
+    pub image_directory: Option<PathBuf>,
+    pub image_retention: FailureImageRetention,
 }
 
 impl CrashInjectionReport {
+    /// 第一行就是这次用的种子基与规模：种子基是这个测试周期写死的那一个（[`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`]），
+    /// 报告与失败信息里不带它就重放不了，下一个周期重抽之后这一批数也就换了（用户 2026-09-20 定案第 7 条）。
     #[must_use]
     pub fn render(&self) -> String {
         let mut text = String::new();
         let _ = writeln!(
             text,
-            "种子 [{}, {})，每段 {} 步、每段抽 {} 个崩溃点，比重：{}；{}",
+            "种子基 {}（这个测试周期写死的；同一个构建加同一个种子基重放得出来），种子 [{}, {})，每段 {} 步，崩溃状态怎么摆 {:?}，比重：{}；{}",
+            self.first_seed,
             self.first_seed,
             self.first_seed + self.seed_count,
             self.operations_per_history,
-            self.crash_points_per_history,
+            self.draw,
             self.weights.name,
             self.execution.name()
         );
+        match (&self.image_directory, self.image_retention) {
+            (Some(directory), FailureImageRetention::KeepTheImageFiles) => {
+                let _ = writeln!(text, "判红的崩溃镜像留在 {}", directory.display());
+            }
+            (Some(directory), FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes) => {
+                let _ = writeln!(
+                    text,
+                    "判红的崩溃镜像曾写在 {}，跑完已删",
+                    directory.display()
+                );
+            }
+            (None, FailureImageRetention::KeepTheImageFiles)
+            | (None, FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes) => {}
+        }
         text.push_str(&self.tally.render());
         for (form_index, form) in KNOWN_RED_FORMS.iter().enumerate() {
             let hits: Vec<(u64, usize)> = self
                 .known_red_hits
                 .iter()
                 .filter(|hit| hit.form == form_index)
-                .map(|hit| (hit.seed.0, hit.crash_point.stream_index))
+                .map(|hit| (hit.seed.0, hit.crash_point.segment_index))
                 .collect();
             let _ = writeln!(
                 text,
-                "崩溃状态上的已知红第 {form_index} 条（{}）：{} 个崩溃点；前几个 (种子, 流下标) {:?}",
+                "崩溃状态上的已知红第 {form_index} 条（{}）：{} 个崩溃状态；前几个 (种子, 段号) {:?}",
                 form.closeout_table_row,
                 hits.len(),
                 hits.iter().take(8).collect::<Vec<_>>()
@@ -729,33 +1051,52 @@
     }
 }
 
-/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，每段 `operations_per_history` 步、每段按种子抽 `crash_points_per_history`
-/// 个崩溃点，分给 `worker_threads.count()` 个工作线程：种子区间切成首尾相接的片，线程按片号从小到大领片，
-/// 调用线程收到一片就打一行 `CRASH_INJECTION_PROGRESS`（片号、种子区间、已跑完的片数与段数），再按片号从小到大并计数。
-/// 计数按片的次序相加、新发现按种子从小到大留第一个，所以 [`CrashInjectionReport::render`] 与线程数、调度次序无关
-/// （进度行是按到达次序打的，只报跑到哪了，不带判定）。
+/// 一次崩溃注入跑什么：种子区间、每段几步、怎么摆崩溃状态、比重、历史怎么跑、线程数、判红时镜像留不留。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CrashInjectionCampaign {
+    pub first_seed: u64,
+    pub seed_count: u64,
+    pub operations_per_history: usize,
+    pub draw: CrashPointDraw,
+    pub weights: GenerationWeights,
+    pub execution: HistoryExecution,
+    pub worker_threads: CrashInjectionWorkerThreads,
+    pub image_retention: FailureImageRetention,
+}
+
+/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，分给 `worker_threads.count()` 个工作线程：种子区间切成首尾相接的片，
+/// 线程按片号从小到大领片，调用线程收到一片就打一行 `CRASH_INJECTION_PROGRESS`（片号、种子区间、已跑完的片数与段数），
+/// 再按片号从小到大并计数。计数按片的次序相加、新发现按种子从小到大留第一个，所以
+/// [`CrashInjectionReport::render`] 与线程数、调度次序无关（进度行是按到达次序打的，只报跑到哪了，不带判定）。
 ///
 /// # Panics
-/// 某个工作线程 panic（历史里的 panic 在执行器里接住，走到这里的是崩溃点上恢复或 checker 的断言）；有一片领了却没交回。
+/// 某个工作线程 panic（历史里的 panic 在执行器里接住，走到这里的是崩溃状态上恢复或 checker 的断言）；有一片领了却没交回。
 #[must_use]
-pub fn run_crash_injection_campaign(
-    first_seed: u64,
-    seed_count: u64,
-    operations_per_history: usize,
-    crash_points_per_history: usize,
-    weights: &GenerationWeights,
-    execution: HistoryExecution,
-    worker_threads: CrashInjectionWorkerThreads,
-) -> CrashInjectionReport {
+pub fn run_crash_injection_campaign(campaign: &CrashInjectionCampaign) -> CrashInjectionReport {
+    let CrashInjectionCampaign {
+        first_seed,
+        seed_count,
+        operations_per_history,
+        draw,
+        weights,
+        execution,
+        worker_threads,
+        image_retention,
+    } = *campaign;
     let slices = seed_slices(seed_count, worker_threads.count());
     let spawned_worker_threads = worker_threads.count().min(slices.len().max(1));
     let started = Instant::now();
+    let image_directory = std::env::temp_dir().join(format!(
+        "singlefs-crash-injection-{}-seedbase-{first_seed}",
+        std::process::id()
+    ));
     println!(
-        "CRASH_INJECTION_START seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={}",
+        "CRASH_INJECTION_START seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={} images={}",
         first_seed + seed_count,
         slices.len(),
         worker_threads.count(),
-        worker_threads.source_name()
+        worker_threads.source_name(),
+        image_directory.display()
     );
     let next_slice_index = AtomicUsize::new(0);
     let (finished_slices, merged_slice_count) = std::thread::scope(|scope| {
@@ -764,6 +1105,7 @@
             let sender = sender.clone();
             let slices = &slices;
             let next_slice_index = &next_slice_index;
+            let image_directory = image_directory.as_path();
             scope.spawn(move || loop {
                 let slice_index = next_slice_index.fetch_add(1, Ordering::Relaxed);
                 let Some(slice) = slices.get(slice_index) else {
@@ -773,11 +1115,16 @@
                     .clone()
                     .map(|offset| {
                         let history = generate_history_with_weights(
-                            HistorySeed(first_seed + offset),
+                            HistorySeed(first_seed.wrapping_add(offset)),
                             operations_per_history,
-                            weights,
+                            &weights,
                         );
-                        inject_crashes_into_history(&history, execution, crash_points_per_history)
+                        inject_crashes_into_history(
+                            &history,
+                            execution,
+                            draw,
+                            Some(image_directory),
+                        )
                     })
                     .collect();
                 if sender.send((slice_index, injections)).is_err() {
@@ -798,8 +1145,8 @@
                 "CRASH_INJECTION_PROGRESS slice={}/{} seeds=[{},{}) finished_slices={}/{} finished_histories={finished_histories}/{seed_count} elapsed_seconds={:.1}",
                 slice_index + 1,
                 slices.len(),
-                first_seed + slice.start,
-                first_seed + slice.end,
+                first_seed.wrapping_add(slice.start),
+                first_seed.wrapping_add(slice.end),
                 finished_slice_count + 1,
                 slices.len(),
                 started.elapsed().as_secs_f64()
@@ -829,23 +1176,33 @@
     let mut signatures_seen: BTreeSet<FailureSignature> = BTreeSet::new();
     for injection in &finished_slices {
         tally.absorb(&injection.tally);
-        known_red_hits.extend(injection.known_red_hits.iter().copied());
+        known_red_hits.extend(injection.known_red_hits.iter().cloned());
         for finding in &injection.new_findings {
             if signatures_seen.insert(finding.signature.clone()) {
                 new_findings.push(finding.clone());
             }
         }
     }
+    let image_directory_left_behind = match image_retention {
+        FailureImageRetention::KeepTheImageFiles => image_directory.exists(),
+        FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes => {
+            let existed = image_directory.exists();
+            let _ = std::fs::remove_dir_all(&image_directory);
+            existed
+        }
+    };
     CrashInjectionReport {
         first_seed,
         seed_count,
         operations_per_history,
-        crash_points_per_history,
-        weights: *weights,
+        draw,
+        weights,
         execution,
         tally,
         known_red_hits,
         new_findings,
+        image_directory: image_directory_left_behind.then_some(image_directory),
+        image_retention,
     }
 }
 
@@ -870,6 +1227,10 @@
     use super::*;
     use crate::history::{generate_history, HistoryStartingPoint};
 
+    const FOUR_CRASH_POINTS: CrashPointDraw = CrashPointDraw::Sampled {
+        crash_points_per_history: 4,
+    };
+
     /// 线程数从环境变量取，没设取 `available_parallelism`；设了不是正整数就停，不悄悄退回单线程。
     #[test]
     fn worker_thread_count_comes_from_the_environment_variable_or_available_parallelism() {
@@ -920,46 +1281,70 @@
         }
     }
 
-    /// 同一个种子抽到的崩溃点逐项相同；崩溃点都落在起点之后、都是写、按流里的次序排好。
+    /// 同一个种子摆出来的崩溃状态逐项相同；段内持久的是真子集、按段号排好。
     #[test]
-    fn crash_points_are_reproducible_and_sorted_writes_after_the_starting_point() {
+    fn crash_points_are_reproducible_proper_subsets_sorted_by_segment() {
         let history = generate_history(HistorySeed(11), 12);
         let first = inject_crashes_into_history(
             &history,
             HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
-            4,
+            FOUR_CRASH_POINTS,
+            None,
         );
         let second = inject_crashes_into_history(
             &history,
             HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
-            4,
+            FOUR_CRASH_POINTS,
+            None,
         );
         assert_eq!(first, second, "同一个种子、同一个跑法，逐项相同");
         assert!(
             first.tally.crash_points >= 1,
-            "种子 11 的 12 步里抽得到崩溃点：{:?}",
+            "种子 11 的 12 步里摆得出崩溃状态：{:?}",
             first.tally
         );
         assert_eq!(
             first.tally.model_judgements, first.tally.crash_points,
-            "每个崩溃点都问过模型"
+            "每个崩溃状态都问过模型"
         );
         assert_eq!(
             first.tally.checker_runs, first.tally.crash_points,
-            "每个崩溃点都跑过 checker"
+            "每个崩溃状态都跑过 checker"
+        );
+        assert_eq!(
+            first.tally.record_checks, first.tally.crash_points,
+            "每个崩溃状态都跑过记录核对器"
+        );
+        assert!(
+            first.crash_points.iter().all(|crash_point| crash_point
+                .persisted_within_the_segment
+                .iter()
+                .any(|persisted| !persisted)),
+            "段内持久的是真子集：{:?}",
+            first.crash_points
+        );
+        assert!(
+            first
+                .crash_points
+                .windows(2)
+                .all(|pair| pair[0].segment_index <= pair[1].segment_index),
+            "崩溃状态按段号排好"
         );
     }
 
-    /// 崩溃点抽在写上：种子越多，四种落点（单元、journal 记录、根槽、超级块槽）都抽得到。
+    /// 段内扣下的写四种落点都见得到，段内有洞（后发的写先持久）的状态真的摆出来了，起点那一段也摆得到。
     #[test]
-    fn crash_points_cover_every_kind_of_write() {
+    fn crash_points_withhold_every_kind_of_write_and_leave_holes_inside_a_segment() {
         let mut tally = CrashInjectionTally::default();
         for seed in 0..8u64 {
             let history = generate_history(HistorySeed(seed), 16);
             let injection = inject_crashes_into_history(
                 &history,
                 HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
-                6,
+                CrashPointDraw::Sampled {
+                    crash_points_per_history: 6,
+                },
+                None,
             );
             tally.absorb(&injection.tally);
         }
@@ -967,74 +1352,64 @@
             StepKind::UnitWrite,
             StepKind::JournalRecord,
             StepKind::RootRecordFua,
-            StepKind::SuperblockSlot,
+            StepKind::SystemConfigurationSlot,
         ] {
             assert!(
                 tally
-                    .crash_points_by_write_kind
+                    .crash_points_withholding_write_kind
                     .get(kind.name())
                     .copied()
                     .unwrap_or(0)
                     >= 1,
-                "八个种子里一次都没截在 {} 上：{:?}",
+                "八个种子里一次都没扣下 {}：{:?}",
                 kind.name(),
-                tally.crash_points_by_write_kind
+                tally.crash_points_withholding_write_kind
             );
         }
         assert!(
             tally.crash_points_that_land_before_the_last_committed_version >= 1,
-            "一个崩溃点都没截回到最后一版之前"
+            "一个崩溃状态都没截回到最后一版之前"
+        );
+        assert!(
+            tally.crash_points_withholding_a_write_before_a_persisted_one >= 1,
+            "一个段内有洞的状态都没摆出来（等于还是只截前缀）：{tally:?}"
+        );
+        assert!(
+            tally.crash_points_inside_the_starting_point >= 1,
+            "起点那一段里一个崩溃状态都没摆到（起点段 2026-09-20 才放开）：{tally:?}"
         );
     }
 
-    /// 起点那一段整个持久：从 mkfs 起的历史里，崩溃点的流下标都在起点之后。
+    /// 起点那一段也在候选里：从第一个文件起的历史，崩溃状态落得到起点那一步
+    /// （代码三方 m2-supp3-item3-code-r1 判决 K1-d：快档 7219 次写里 627 次此前永远抽不到）。
     #[test]
-    fn no_crash_point_falls_inside_the_starting_point() {
+    fn crash_points_fall_inside_the_starting_point_too() {
         let history = GeneratedHistory {
             seed: HistorySeed(3),
             starting_point: HistoryStartingPoint::AfterFirstFile,
             operations: generate_history(HistorySeed(3), 10).operations,
         };
-        let stream = SharedStream::retaining_contents();
-        let mut operations_after_the_starting_point = 0;
-        let _ = execute_history_with(
-            &history,
-            HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
-            &stream,
-            &mut |observation| {
-                if observation.position == StepPosition::StartingPoint {
-                    operations_after_the_starting_point = stream.operation_count();
-                }
-            },
-        );
-        assert!(
-            operations_after_the_starting_point > 0,
-            "起点写过盘（mkfs 加第一个文件）"
-        );
         let injection = inject_crashes_into_history(
             &history,
             HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
-            6,
+            CrashPointDraw::EveryProperSubsetOfShortSegments {
+                segment_writes: 2,
+                sampled_in_longer_segments: 2,
+            },
+            None,
         );
         assert!(injection.tally.crash_points >= 1);
         assert!(
             injection
                 .crash_points
                 .iter()
-                .all(|crash_point| crash_point.stream_index >= operations_after_the_starting_point),
-            "崩溃点都在起点之后（起点整个持久）：{:?}，起点占了 {operations_after_the_starting_point} 步",
+                .any(|crash_point| crash_point.step == StepPosition::StartingPoint),
+            "起点那一段里一个崩溃状态都没摆到：{:?}",
             injection
                 .crash_points
                 .iter()
-                .map(|crash_point| crash_point.stream_index)
-                .collect::<Vec<usize>>()
-        );
-        assert!(
-            injection
-                .crash_points
-                .windows(2)
-                .all(|pair| pair[0].stream_index < pair[1].stream_index),
-            "崩溃点按流里的次序排好、互不相同"
+                .map(|crash_point| crash_point.step)
+                .collect::<Vec<StepPosition>>()
         );
     }
 }
```

## `crates/mutations.tsv` 这一轮新增的 37 行

```
增补 3 第 1 件：随机历史快档判出「复用时追加记录而不改写」（第 41 行同一处）	crates/singlefs-core/src/allocator.rs	            existing.span_slots = u16::try_from(placement.span).expect("跨度 2 字节");\n            existing.generation = generation;\n            existing.is_released = false;	            records.push(AllocationRecord {\n                device,\n                slot: placement.slot,\n                span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),\n                generation,\n                is_released: false,\n            });	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
增补 3 第 1 件：随机历史偏向抬 F 之后复用的取样点判出「复用时新记录罩住的已回收记录不删」（第 121 行同一处）	crates/singlefs-core/src/allocator.rs	            records.retain(|record| !(record.device == device && record.slot == record_slot));	            // 变异：罩住的已回收记录不删	-p singlefs-harness --test second_transaction_supplement_three_random_history -- reuse_heavy_random_histories	reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms
增补 3 第 1 件（代码三方第一轮第 1 条）：已知红第 1 条不看 F 是否落在回退留下的空档里	crates/singlefs-harness/src/history.rs	    observation.raised_floor_lands_only_on_abandoned_roots == Some(true)\n        && only_allocated_statistic_above_walked(observation)	    only_allocated_statistic_above_walked(observation)	-p singlefs-harness --test second_transaction_supplement_three_random_history -- an_allocated_statistic_over_count	an_allocated_statistic_over_count_after_raising_the_floor_without_a_rollback_is_a_new_finding
增补 3 第 1 件（代码三方第一轮第 3 条）：随机历史快档判出「复用改写已回收记录时不改分配代」（N2）	crates/singlefs-core/src/allocator.rs	            existing.generation = generation;\n            existing.is_released = false;	            existing.is_released = false;	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
增补 3 第 1 件（代码三方第一轮第 4 条）：随机历史快档判出「checker 判绿的镜像上冷启动读回报错」（B6：补齐字节从最后一个载荷字节算起）	crates/singlefs-core/src/unit.rs	    if bytes[payload_end..].iter().any(|byte| *byte != 0) {	    if bytes[payload_end - 1..].iter().any(|byte| *byte != 0) {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
增补 3 第 1 件（代码三方第二轮第 1 条，形态 a）：只在挂载写出的写行与暖机里复用改写已回收记录时不改分配代	crates/singlefs-core/src/allocator.rs	            existing.generation = generation;\n            existing.is_released = false;	            if !std::backtrace::Backtrace::force_capture().to_string().contains("mount::establish_instance") {\n                existing.generation = generation;\n            }\n            existing.is_released = false;	-p singlefs-harness --test second_transaction_supplement_three_random_history -- row_and_warm_up_publishes	row_and_warm_up_publishes_that_reuse_reclaimed_records_have_their_allocation_generations_checked
增补 3 第 1 件（代码三方第二轮第 1 条，形态 b）：挂载写出的写行与暖机里每次分配的分配代记成 txg − 1	crates/singlefs-core/src/allocator.rs	    fn record(&mut self, placement: Placement, generation: CheckpointTxg) {\n	    fn record(&mut self, placement: Placement, generation: CheckpointTxg) {\n        let generation = if std::backtrace::Backtrace::force_capture().to_string().contains("mount::establish_instance") { CheckpointTxg(generation.0 - 1) } else { generation };\n	-p singlefs-harness --test second_transaction_supplement_three_random_history -- row_and_warm_up_publishes	row_and_warm_up_publishes_that_reuse_reclaimed_records_have_their_allocation_generations_checked
增补 3 第 1 件（代码三方第二轮第 2 条，几何补取样点）：随机历史快档也判出「复用时新记录罩住的已回收记录不删」（第 121、130 行同一处）	crates/singlefs-core/src/allocator.rs	            records.retain(|record| !(record.device == device && record.slot == record_slot));	            // 变异：罩住的已回收记录不删	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
增补 3 第 2 件（理想模型，r1 攻方变异 B1）：内容正好装满载荷容量也报装不下	crates/singlefs-core/src/transaction.rs	if file.content.len() > data_unit_capacity {	if file.content.len() >= data_unit_capacity {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
增补 3 第 2 件（理想模型，r1 攻方变异 B2）：回退到 txg = F_生效 的根也拒	crates/singlefs-core/src/mount.rs	if target.checkpoint_txg < effective_floor {	if target.checkpoint_txg <= effective_floor {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_the_root_at_the_effective_floor	rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version
增补 3 第 2 件（理想模型，r1 攻方变异 B3）：抬 F 上限不取第 4 新的非空根（只取每盘最新有效根）	crates/singlefs-core/src/mount.rs	Some(newest_on_every_device.min(fourth_newest))	Some(newest_on_every_device.max(fourth_newest))	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
增补 3 第 2 件（理想模型，r1 攻方变异 B5）：非空判定恒真（空发布也算非空）	crates/singlefs-core/src/mount.rs	root_pointers.inode_tree != previous_valid_root_pointers.inode_tree\n        || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree	true || root_pointers.inode_tree != previous_valid_root_pointers.inode_tree\n        || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
增补 3 第 2 件（代码三方第一轮判决第三节第 1 条，攻方变异 W1）：分配记录墙差一，812 条正好装满也拒（> 写成 >=）；逼近墙的取样点判出	crates/singlefs-core/src/transaction.rs	    if records_after_this_publish > allocation_node_capacity {	    if records_after_this_publish >= allocation_node_capacity {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_wall_sampling	allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count
增补 3 第 2 件（代码三方第一轮判决第三节第 1 条，攻方变异 W1 同一处）：分配记录墙差一；812 条那一格的边沿用例判出	crates/singlefs-core/src/transaction.rs	    if records_after_this_publish > allocation_node_capacity {	    if records_after_this_publish >= allocation_node_capacity {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- an_overwrite_that_fills_the_allocation_node	an_overwrite_that_fills_the_allocation_node_to_exactly_812_records_succeeds_and_the_next_is_refused
增补 3 第 2 件（代码三方第一轮判决第三节第 1 条的改法）：checker 数一条根下的分配记录只数一半	crates/singlefs-checker/src/walk.rs	        records += node.entries.len();	        records += node.entries.len() / 2;	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_records_counted_on_the_image	allocation_records_counted_on_the_image_are_one_per_unit_per_device_and_zero_without_a_file
增补 3 第 2 件（代码三方第一轮判决第三节第 2 条，攻方变异 R1）：被抛弃时间线的根报成低于 F（只换判别字段）	crates/singlefs-core/src/mount.rs	            exclusion: RollbackCandidateExclusion::OnAbandonedTimeline,	            exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_onto_an_abandoned_root_above_the_floor	rolling_back_onto_an_abandoned_root_above_the_floor_is_refused_for_being_abandoned
增补 3 第 2 件（代码三方第一轮判决第三节第 2 条的改法）：胶水把回退目标「被抛弃」映射成「低于 F」	crates/singlefs-harness/src/model_comparison.rs	        RollbackCandidateExclusion::OnAbandonedTimeline => {\n            ModelRefusalReason::RollbackTargetOnAbandonedTimeline\n        }	        RollbackCandidateExclusion::OnAbandonedTimeline => {\n            ModelRefusalReason::RollbackTargetBelowEffectiveFloor\n        }	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_onto_an_abandoned_root_above_the_floor	rolling_back_onto_an_abandoned_root_above_the_floor_is_refused_for_being_abandoned
增补 3 第 2 件（代码三方第二轮判决第三节第 1 条）：回退到树表 0 条的根（在候选集里、第一版不支持）报成候选排除「低于 F」	crates/singlefs-core/src/mount.rs	        return Err(MountError::RollbackToVersionWithoutFileUnsupported(target));	        return Err(MountError::RollbackTargetNotACandidate {\n            target,\n            exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,\n        });	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
增补 3 第 2 件（代码三方第二轮判决第三节第 1 条）：候选排除「低于 F」报成「树表 0 条、第一版不支持」；偏向回退那一档抽样判出（这一档的必红是抽样断言，随测试周期的种子基重验，见 d13-item4-fua-r1 判决）	crates/singlefs-core/src/mount.rs	        return Err(MountError::RollbackTargetNotACandidate {\n            target,\n            exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,\n        });	        return Err(MountError::RollbackToVersionWithoutFileUnsupported(target));	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rollback_heavy_random_histories	rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms
增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2h）：分配器用户数据那一处把「每块盘上都没有」报成「小盘写满」；小盘段判出	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::Agreed(slot) => slot,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);	            DeviceAgreement::Agreed(slot) => slot,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec::new() });	-p singlefs-harness --test second_transaction_supplement_three_random_history -- unit_area_wall_sampling	unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device
增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2j）：分配器提交内生块那一处把「每块盘上都没有」报成「小盘写满」；小盘段判出	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::Agreed(answer) => answer,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);	            DeviceAgreement::Agreed(answer) => answer,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec::new() });	-p singlefs-harness --test second_transaction_supplement_three_random_history -- unit_area_wall_sampling	unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device
增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4a）：根环转过之后回收门槛取「环里最旧有效根 + 1」；逼近分配记录墙那一段照跑 checker 判出	crates/singlefs-core/src/mount.rs	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(if oldest.0 > 0 { CheckpointTxg(oldest.0 + 1) } else { oldest }))	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_wall_sampling	allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count
增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4b）：分配记录过 600 条之后记账「已分配」少记一槽；逼近分配记录墙那一段照跑 checker 判出	crates/singlefs-core/src/transaction.rs	            device_map.allocated_slots() * SLOT_BYTES,	            (device_map.allocated_slots() - u64::from(allocator.records().len() > 600)) * SLOT_BYTES,	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_wall_sampling	allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count
增补 3 第 2 件（代码三方第二轮判决第三节第 3 条的改法）：「已知红第 0 条那一形只记不停」退回成判红就停；逼近分配记录墙那一段走不到墙	crates/singlefs-harness/src/history.rs	                if !continues_past_the_ring_turn_form {\n                    return Some(observation);\n                }	                if true || !continues_past_the_ring_turn_form {\n                    return Some(observation);\n                }	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_wall_sampling	allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count
增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1c；要带调试符号的构建，函数改名会悄悄失效）：只在抬 F 路径的准入里多算一个角色；抬 F 逼近墙的写死用例判出	crates/singlefs-core/src/transaction.rs	        records_before_this_publish + rewritten.len() * allocator.devices.len();	        records_before_this_publish + (rewritten.len() + usize::from(std::backtrace::Backtrace::force_capture().to_string().contains("raise_rollback_floor"))) * allocator.devices.len();	-p singlefs-harness --test second_transaction_supplement_three_random_history -- raising_the_floor_with_a_second_empty_publish	raising_the_floor_with_a_second_empty_publish_that_fills_the_allocation_node_to_exactly_812_records_succeeds
增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1d；要带调试符号的构建，函数改名会悄悄失效）：只在回退路径的准入里多算一个角色；回退逼近墙的写死用例判出	crates/singlefs-core/src/transaction.rs	        records_before_this_publish + rewritten.len() * allocator.devices.len();	        records_before_this_publish + (rewritten.len() + usize::from(std::backtrace::Backtrace::force_capture().to_string().contains("mount_rollback"))) * allocator.devices.len();	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_a_root_whose_warm_up	rolling_back_to_a_root_whose_warm_up_fills_the_allocation_node_to_exactly_812_records_succeeds
增补 3 第 2 件（代码三方第二轮判决第三节第 1 条）：候选排除「低于 F」报成「树表 0 条、第一版不支持」；回退到 F 之下的写死用例判出（门禁五段各只红 1 段）	crates/singlefs-core/src/mount.rs	        return Err(MountError::RollbackTargetNotACandidate {\n            target,\n            exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,\n        });	        return Err(MountError::RollbackToVersionWithoutFileUnsupported(target));	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_below_the_effective_floor	rolling_back_below_the_effective_floor_is_refused_for_being_below_the_floor
增补 3 第 3 件：抽 0 个崩溃点时照样抽一个（抽崩溃点会改变这段历史怎么跑，判别力那一半失效）	crates/singlefs-harness/src/crash_injection.rs	            for _ in 0..crash_points_per_history {	            for _ in 0..crash_points_per_history.max(1) {	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- drawing_crash_points	drawing_crash_points_does_not_change_the_generated_history
增补 3 第 3 件：判崩溃点时取择根而不是施加记录前缀之后实际走的那条根（记录已写、根槽未写那一格内容与根身份配不上）	crates/singlefs-harness/src/crash_injection.rs	        let read_back = observed_read_back_after_a_crash(&report);	        let read_back = crate::model_comparison::observed_read_back(&report.outcome);	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- crash_injection_fast_tier	crash_injection_fast_tier_recovers_only_into_versions_the_model_committed
增补 3 第 3 件（用户 2026-09-20 定案第 1 条）：屏障不再切段（屏障进了枚举域，少一道屏障就把两段并成一段）	crates/singlefs-harness/src/crash.rs	            RecordedOperationKind::Barrier => {\n                if !current.is_empty() {\n                    segments.push(std::mem::take(&mut current));\n                }\n            }	            RecordedOperationKind::Barrier => {}	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- every_crash_state_of_a_written_out_history	every_crash_state_of_a_written_out_history_recovers_into_a_committed_version
增补 3 第 3 件（用户 2026-09-20 定案第 2 条）：段内只截前缀，不摆任意真子集（「后发的写先持久」整类零覆盖）	crates/singlefs-harness/src/crash_injection.rs	    if segment_length <= WRITES_A_SUBSET_MASK_HOLDS {\n        let mask = source.below((1u64 << segment_length) - 1);\n        return bits_of(mask, segment_length);\n    }\n    let mut persisted: Vec<bool> = (0..segment_length).map(|_| source.below(2) == 1).collect();\n    if persisted.iter().all(|is_persisted| *is_persisted) {\n        let withheld = usize::try_from(source.below(u64::try_from(segment_length).expect("段长")))\n            .expect("下标装得进 usize");\n        persisted[withheld] = false;\n    }\n    persisted	    let persisted_prefix = source.below(u64::try_from(segment_length).expect("段长"));\n    (0..segment_length)\n        .map(|write| u64::try_from(write).expect("下标") < persisted_prefix)\n        .collect()	-p singlefs-harness --lib -- crash_injection::tests::crash_points_withhold_every_kind_of_write	crash_points_withhold_every_kind_of_write_and_leave_holes_inside_a_segment
增补 3 第 3 件（用户 2026-09-20 定案第 3 条）：崩溃状态上不跑记录核对器	crates/singlefs-harness/src/crash_injection.rs	        let record_check =\n            check_records_against(&image, writes_up_to_this_segment, report.effective_root);\n        tally.record_checks += 1;	        let record_check = RecordCheck::default();	-p singlefs-harness --lib -- crash_injection::tests::crash_points_are_reproducible	crash_points_are_reproducible_proper_subsets_sorted_by_segment
增补 3 第 3 件（用户 2026-09-20 定案第 4 条）：崩溃状态又只摆在起点跑完之后（起点那一段的写永远抽不到）	crates/singlefs-harness/src/crash_injection.rs	                (marks.after_make_filesystem..marks.after_the_last_finished_step)\n                    .contains(&stream_indexes[*write])	                (marks\n                    .after_each_step\n                    .first()\n                    .map_or(0, |(_, after)| *after)\n                    ..marks.after_the_last_finished_step)\n                    .contains(&stream_indexes[*write])	-p singlefs-harness --lib -- crash_injection::tests::crash_points_fall_inside_the_starting_point	crash_points_fall_inside_the_starting_point_too
增补 3 第 3 件（用户 2026-09-20 定案第 6 条）：checker 在 I-3.1 的说明文字里少写一项机理标识（判读的一方就认不出机理）	crates/singlefs-checker/src/walk.rs	、回退下界 F {newest_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"	、低于 F 的根槽 {root_slots_dropped_below_floor} 个"	-p singlefs-harness --test second_transaction_supplement_three_random_history -- turning_the_root_ring_with_overwrites	turning_the_root_ring_with_overwrites_in_one_mount_ends_in_the_first_known_red_form
增补 3 第 3 件（用户 2026-09-20 定案第 6 条）：崩溃状态上不算「F 落在回退留下的空档里」，写死成 None（收口表第 43 行那一形恒不匹配）	crates/singlefs-harness/src/crash_injection.rs	        raised_floor_lands_only_on_abandoned_roots:\n            raised_floor_of_the_newest_root_lands_only_on_abandoned_roots(image),	        raised_floor_lands_only_on_abandoned_roots: None,	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- crash_states_after_raising_the_floor_into_the_gap	crash_states_after_raising_the_floor_into_the_gap_match_the_second_known_red_form
增补 3 第 3 件（用户 2026-09-20 定案第 7 条，同日定的范围）：这个测试周期的种子基换成别的数（这一周期量过的判红整批作废，两个二进制也不再同基）	crates/singlefs-harness/src/crash_injection.rs	pub const SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE: u64 = 7_463_871_032_432_355_113;	pub const SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE: u64 = 12345;	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- the_test_cycle_seed_base	the_test_cycle_seed_base_is_the_number_drawn_for_this_cycle
增补 3 第 3 件（用户 2026-09-20 定案第 7 条，同日定的范围）：随机历史快档的种子基改回写死的别的数（与崩溃注入那个二进制不再同基，跑的是另一批历史）	crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs	const FAST_TIER_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;	const FAST_TIER_FIRST_SEED: u64 = 0;	-p singlefs-harness --test second_transaction_supplement_three_random_history -- the_five_sampling_tiers	the_five_sampling_tiers_start_from_the_test_cycle_seed_base
```
