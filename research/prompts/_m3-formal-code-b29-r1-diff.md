# 第 27–29 批代码轮第一轮：改动全文（2026-10-05）

六份补丁按合入次序：27-2、27-2b（第 27 批），28-1、26-5（第 28 批），29-1、29-2（第 29 批），每份照实现员交回的原样；之后是主 agent 自己的两处改动：#9 补钉（`git diff HEAD` 那一份文件，其中 2026-10-04 那一段是第 26 批 26-2 的，已在第 26 批代码轮判过）、只开一个特性编不干净的修法（`/tmp/claude-1000/gpu-only-fix/gpu-only.patch`）。中间夹着别的线的合入（根环线 10c、10d、10e 与第 927 行那份，删除线第 18 批），它们不在这一轮被判，各自的线判。

## m3-formal-27-2

### crates.patch

````diff
diff --git a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
index 356b05d..9cfa2de 100644
--- a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
+++ b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
@@ -43,8 +43,12 @@ use std::time::Instant;
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
 use singlefs_checker_tier::crash::{
-    full_expansion, judge_layer0_state_range, layer0_enumeration_tables, layer0_plan_state_count,
-    quick_tier_expansion, Layer0EnumerationTables, Layer0StateJudgement,
+    check_records, classified_oracle_violation_for_versions, full_expansion,
+    judge_layer0_state_range, layer0_enumeration_tables, layer0_plan_state_count,
+    quick_tier_expansion, Layer0EnumerationTables, Layer0OracleViolation, Layer0StateJudgement,
+    LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE, LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE,
+    LAYER0_RED_PASS_POOL_CHECKER, LAYER0_RED_PASS_RECORD_CHECKER,
+    RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT, RECORD_CHECKER_ROOT_WITHOUT_RECORD,
 };
 use singlefs_checker_tier::crash_facts::{
     extract_flow_facts, persisted_writes_of_state, SegmentFact, TornImageFact,
@@ -78,7 +82,7 @@ use singlefs_core::unlink::UnlinkBatchSize;
 use singlefs_harness::history::StepPosition;
 use singlefs_harness::memory_pool::{
     newest_persisted_root, writes_and_segments_with_stream_indexes_and_entries, CrashImage,
-    MemoryPool, PublishedVersion, RetainedWrite, WrittenContents,
+    MemoryPool, PublishedVersion, RecordCheck, RetainedWrite, WrittenContents,
 };
 use singlefs_harness::model::{
     crash_recovery_disagreement, IdealModel, ModelAnswer, ModelDeviceIdentity, ModelDisagreement,
@@ -272,6 +276,22 @@ const LOCAL_CONFIRMED_LOSS_CLASSES: u64 = 144;
 const LOCAL_CONFIRMED_LOSS_STATES_OF_A_CLASS_WITH_IN_FLIGHT_WRITES_LANDED: u64 = 16_777_215;
 /// 要照原路径判的态数（每种取法的代表加「C 全落」那一态）。
 const LOCAL_CONFIRMED_LOSS_JUDGED_STATES: u64 = 144;
+/// 「等价类对拍」那一形末段里属于第 3 次调用的写数（修订 5 第 5.2 节丙：k = 6）。
+const LOCAL_EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES: usize = 6;
+/// 「等价类对拍」那一形的态数：末段各写取法数之积，K 的 72 种取法 × C 那 6 个整写各 2 种（修订 5 第 5.2 节，72 × 2⁶ = 4 608）。
+const LOCAL_EQUIVALENCE_PAIRING_STATES: u64 =
+    LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2 * 2 * 2 * 2 * 2;
+/// c1…c6 的镜像对数（修订 5 第 5.2 节：两两同偏移、同长度、分属盘 0 与盘 1，三对的偏移互不相同）。
+const LOCAL_EQUIVALENCE_PAIRING_MIRRORED_PAIRS: usize = 3;
+/// 阳性对照「判态路 panic 单列」`pool_checker` 那条注入的态数：c3 落、c4 没落，K 的 72 种取法 × c1、c2、c5、c6 各 2 种（72 × 2⁴ = 1 152）。
+const LOCAL_PIECE_PANIC_CONTROL_POOL_CHECKER_STATES: u64 =
+    LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2 * 2 * 2;
+/// 同上 `ignored_recovery` 那条：c3 没落、c4 落（72 × 2⁴ = 1 152）。
+const LOCAL_PIECE_PANIC_CONTROL_IGNORED_RECOVERY_STATES: u64 =
+    LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2 * 2 * 2;
+/// 同上 `record_checker` 那条：c5 落、c6 没落、(c3,c4) 同落或同不落，72 × c1、c2 各 2 种 × 2（72 × 2² × 2 = 576）。
+const LOCAL_PIECE_PANIC_CONTROL_RECORD_CHECKER_STATES: u64 =
+    LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2 * 2;
 /// `confirmed-loss-form` 一趟判得完的态数上限：每态推 5–50 毫秒，上限这么多约是一天以内。方向相反的取样点全枚举，超过的不跑、
 /// 打一行 `not_run` 退 3 交主 agent；主取样点按等价类判（登记修订 2 第 2.1 节候选二、修订 3），这个上限管的是类核对 ③ 细分之后
 /// 要照原路径判的态数（每种取法的代表加「C 全落」那一态），超过走停机 S13（主取样点全枚举是 1 207 959 552 态，第 1 次重跑第一段跑不了）。
@@ -2609,22 +2629,107 @@ struct ImplementationVerdict {
     ignored_outcome: String,
     red_items: Vec<String>,
     pool_checker_red: Vec<String>,
-    root_without_record: bool,
-    claimed_state_missing_unit: bool,
+    /// 记录核对器的两项（`true` / `false`；逐件重做那一路里那一件 panic 写 `panic`、没跑写 `not_run`）。
+    root_without_record: String,
+    claimed_state_missing_unit: String,
 }
 
 impl ImplementationVerdict {
-    /// 恢复 panic 的那一态（读法 5）：判定那几格都写 `recovery_panic`，池级 checker 那一列不在这里（O2 另判）。
-    fn of_a_panicking_recovery() -> Self {
+    /// 回调链 panic 了的那一态（修订 5 第 5.1 节第 4 条）：各格取各件自己的结果，panic 的件写 `panic`，没跑的写 `not_run`。
+    /// `red_items` 照 [`Layer0StateJudgement::red_items`] 的次序与写法列跑完了的那几件判红的项，panic 与没跑的件记成
+    /// `panic:<项>`、`not_run:<项>`（不以 `<项>:` 起头，不进按项数红的计数）。
+    fn of_the_pieces(
+        consulted: &PieceOutcome<RecoveryReport>,
+        ignored: &PieceOutcome<RecoveryReport>,
+        layer0_oracles: &PieceOutcome<Layer0OraclesOfThePieces>,
+        pool_checker: &PieceOutcome<Vec<(&'static str, InvariantVerdict)>>,
+        record_checker: &PieceOutcome<RecordCheck>,
+    ) -> Self {
+        let mut red_items = Vec::new();
+        let oracles: [(&str, Result<Option<&Layer0OracleViolation>, &str>); 2] =
+            match layer0_oracles {
+                PieceOutcome::Done(Layer0OraclesOfThePieces {
+                    consulted: consulted_oracle,
+                    ignored: ignored_oracle,
+                }) => [
+                    (
+                        LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE,
+                        consulted_oracle
+                            .as_ref()
+                            .map(Option::as_ref)
+                            .ok_or("not_run"),
+                    ),
+                    (
+                        LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE,
+                        ignored_oracle.as_ref().map(Option::as_ref).ok_or("not_run"),
+                    ),
+                ],
+                PieceOutcome::Panicked(_) => [
+                    (LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE, Err("panic")),
+                    (LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE, Err("panic")),
+                ],
+                PieceOutcome::NotRun => [
+                    (LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE, Err("not_run")),
+                    (LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE, Err("not_run")),
+                ],
+            };
+        for (pass, oracle) in oracles {
+            match oracle {
+                Ok(Some(violation)) => {
+                    red_items.push(format!("{pass}:{}", violation.kind.name()));
+                }
+                Ok(None) => {}
+                Err(marker) => red_items.push(format!("{marker}:{pass}")),
+            }
+        }
+        match pool_checker {
+            PieceOutcome::Done(verdicts) => red_items.extend(
+                violated_invariant_names(verdicts)
+                    .into_iter()
+                    .map(|invariant| format!("{LAYER0_RED_PASS_POOL_CHECKER}:{invariant}")),
+            ),
+            PieceOutcome::Panicked(_) => {
+                red_items.push(format!("panic:{LAYER0_RED_PASS_POOL_CHECKER}"));
+            }
+            PieceOutcome::NotRun => {
+                red_items.push(format!("not_run:{LAYER0_RED_PASS_POOL_CHECKER}"));
+            }
+        }
+        match record_checker {
+            PieceOutcome::Done(records) => {
+                if records.root_without_record {
+                    red_items.push(format!(
+                        "{LAYER0_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_ROOT_WITHOUT_RECORD}"
+                    ));
+                }
+                if records.claimed_state_missing_unit {
+                    red_items.push(format!(
+                        "{LAYER0_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT}"
+                    ));
+                }
+            }
+            PieceOutcome::Panicked(_) => {
+                red_items.push(format!("panic:{LAYER0_RED_PASS_RECORD_CHECKER}"));
+            }
+            PieceOutcome::NotRun => {
+                red_items.push(format!("not_run:{LAYER0_RED_PASS_RECORD_CHECKER}"));
+            }
+        }
         Self {
-            consulted_root: "recovery_panic".to_string(),
-            consulted_outcome: "recovery_panic".to_string(),
-            ignored_root: "recovery_panic".to_string(),
-            ignored_outcome: "recovery_panic".to_string(),
-            red_items: vec!["recovery_panic".to_string()],
-            pool_checker_red: Vec::new(),
-            root_without_record: false,
-            claimed_state_missing_unit: false,
+            consulted_root: consulted.cell_text(|report| root_text(report.effective_root)),
+            consulted_outcome: consulted.cell_text(|report| outcome_text(&report.outcome)),
+            ignored_root: ignored.cell_text(|report| root_text(report.effective_root)),
+            ignored_outcome: ignored.cell_text(|report| outcome_text(&report.outcome)),
+            red_items,
+            pool_checker_red: match pool_checker {
+                PieceOutcome::Done(verdicts) => violated_invariant_names(verdicts),
+                PieceOutcome::Panicked(_) => vec!["panic".to_string()],
+                PieceOutcome::NotRun => vec!["not_run".to_string()],
+            },
+            root_without_record: record_checker
+                .cell_text(|records| records.root_without_record.to_string()),
+            claimed_state_missing_unit: record_checker
+                .cell_text(|records| records.claimed_state_missing_unit.to_string()),
         }
     }
     fn of(judgement: &Layer0StateJudgement) -> Self {
@@ -2646,8 +2751,8 @@ impl ImplementationVerdict {
                 })
                 .map(|(invariant, _)| (*invariant).to_string())
                 .collect(),
-            root_without_record: judgement.records.root_without_record,
-            claimed_state_missing_unit: judgement.records.claimed_state_missing_unit,
+            root_without_record: judgement.records.root_without_record.to_string(),
+            claimed_state_missing_unit: judgement.records.claimed_state_missing_unit.to_string(),
         }
     }
 }
@@ -2666,8 +2771,11 @@ enum CrashJudgeVerdict {
     },
     /// 关一条（Q3-核）关掉了 O1：这一列不算。
     Off,
-    /// 这一态上恢复 panic（读法 5「态上的恢复 panic」）：O1 的两个崩溃子列在这一态记不适用。
+    /// 这一态上看 journal 那一遍恢复 panic（读法 5「态上的恢复 panic」；修订 5 第 5.1 节第 3 条收窄成只随这一遍）：
+    /// O1 的两个崩溃子列在这一态记不适用。
     RecoveryPanicked,
+    /// 这一态上 O1-模型崩溃判官自己 panic（修订 5 第 5.1 节第 3 条）：不红不绿，与弃权、不适用分开计。
+    JudgePanicked,
 }
 
 impl CrashJudgeVerdict {
@@ -2678,6 +2786,7 @@ impl CrashJudgeVerdict {
             Self::Red { .. } => "red",
             Self::Off => "off",
             Self::RecoveryPanicked => "recovery_panic",
+            Self::JudgePanicked => "judge_panic",
         }
     }
 }
@@ -2960,6 +3069,8 @@ enum ReferenceFileSystemVoteOfAState {
     },
     /// 这一态上恢复 panic：读回没有，RefFS 不判（同 O1-模型崩溃判官，读法 5）。
     RecoveryPanicked,
+    /// 这一态上 RefFS 自己 panic（修订 5 第 5.1 节第 3 条）：不红不绿，与弃权分开计。
+    JudgePanicked,
     /// 关一条（Q3-核）关掉了 O1：RefFS 子列一起不算。
     Off,
 }
@@ -2971,6 +3082,7 @@ impl ReferenceFileSystemVoteOfAState {
             Self::Red { .. } => "red",
             Self::Abstained { .. } => "abstained",
             Self::RecoveryPanicked => "recovery_panic",
+            Self::JudgePanicked => "judge_panic",
             Self::Off => "off",
         }
     }
@@ -3235,9 +3347,8 @@ fn writable_mount_probe_on(
         CrashJudgeVerdict::Green
         | CrashJudgeVerdict::NotApplicable
         | CrashJudgeVerdict::Off
-        | CrashJudgeVerdict::RecoveryPanicked => {
-            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet
-        }
+        | CrashJudgeVerdict::RecoveryPanicked
+        | CrashJudgeVerdict::JudgePanicked => ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
     };
     let mounted = writable_mount_alone_on(
         crash_image_materialized,
@@ -3375,7 +3486,8 @@ fn raise_the_witnessed_counter_past_the_selected_version(
 // ───── 一个取样态上的 oracle 与它的记录 ─────
 
 /// 一个取样状态上看到的：批号、层 0 序号、段号、落地向量、镜像摘要、`judge_crash_image` 的判定（`impl-verdicts.tsv`）、
-/// O1 三个子列与 O2 的结论与各自耗时、触达的最后一次调用、可写挂载探针（开了才有）。恢复 panic 的态 `recovery_panic` 有值。
+/// O1 三个子列与 O2 的结论与各自耗时、触达的最后一次调用、可写挂载探针（开了才有）。看 journal 那一遍恢复 panic 的态 `recovery_panic` 有值
+/// （修订 5 第 5.1 节第 5 条：含义收窄，回调链上别的件 panic 不算）；判态路上 panic 了的件记在 `piece_panics`。
 struct StateObservation {
     batch: u8,
     ordinal: u64,
@@ -3407,6 +3519,221 @@ struct StateObservation {
     /// 看 journal 那一遍的整份恢复报告：只在 Q5 主取样点按等价类判的那一趟留（类核对 ①、④ 与「等价类对拍」逐字段比它），
     /// 别的趟与恢复 panic 的态是 None。
     consulted_report: Option<RecoveryReport>,
+    /// 回调链 panic 了的态上，逐件重做时 panic 的件（[`JudgingPiece`] 的次序）；回调链没 panic 的态是空的（修订 5 第 5.1 节第 5 条）。
+    piece_panics: Vec<JudgingPiece>,
+}
+
+/// 判一态那条回调链上的一件（修订 5 第 5.1 节第 2 条）：回调链 panic 了，装置按这个次序在同一个镜像上逐件重做、每件各包一层 `catch_unwind`。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+enum JudgingPiece {
+    /// `recover(image, JournalPolicy::Consult)`。
+    ConsultedRecovery,
+    /// `recover(image, JournalPolicy::Ignore)`。
+    IgnoredRecovery,
+    /// 被判那次根槽写落没落（`image.persisted[judged_root_index]`）。
+    RootPersisted,
+    /// 两份 `classified_oracle_violation_for_versions`（连同它们吃的「盘上已持久的最新根」）。
+    Layer0Oracles,
+    /// `check_pool_image`（`judge_crash_image` 里那次与装置直调那次是同一函数、同一镜像，算一件）。
+    PoolChecker,
+    /// `check_records(image, consulted.effective_root)`。
+    RecordChecker,
+    /// O1-模型崩溃判官本身（连同它吃的读回）。
+    CrashJudge,
+    /// RefFS 一票本身（连同它吃的读回）。
+    ReferenceFileSystem,
+    /// 计时那一遍恢复。
+    TimingRecovery,
+    /// 可写挂载探针（开了才有）。
+    Probe,
+}
+
+/// 判态路上的件数（[`JudgingPiece::ALL`] 的长度）。
+const JUDGING_PIECE_COUNT: usize = 10;
+
+impl JudgingPiece {
+    const ALL: [JudgingPiece; JUDGING_PIECE_COUNT] = [
+        JudgingPiece::ConsultedRecovery,
+        JudgingPiece::IgnoredRecovery,
+        JudgingPiece::RootPersisted,
+        JudgingPiece::Layer0Oracles,
+        JudgingPiece::PoolChecker,
+        JudgingPiece::RecordChecker,
+        JudgingPiece::CrashJudge,
+        JudgingPiece::ReferenceFileSystem,
+        JudgingPiece::TimingRecovery,
+        JudgingPiece::Probe,
+    ];
+
+    /// 在 [`JudgingPiece::ALL`] 里的下标。
+    fn position(self) -> usize {
+        JudgingPiece::ALL
+            .iter()
+            .position(|piece| *piece == self)
+            .expect("ALL 列全了每一件")
+    }
+
+    fn name(self) -> &'static str {
+        match self {
+            JudgingPiece::ConsultedRecovery => "consulted_recovery",
+            JudgingPiece::IgnoredRecovery => "ignored_recovery",
+            JudgingPiece::RootPersisted => "root_persisted",
+            JudgingPiece::Layer0Oracles => "layer0_oracles",
+            JudgingPiece::PoolChecker => "pool_checker",
+            JudgingPiece::RecordChecker => "record_checker",
+            JudgingPiece::CrashJudge => "crash_judge",
+            JudgingPiece::ReferenceFileSystem => "reference_filesystem",
+            JudgingPiece::TimingRecovery => "timing_recovery",
+            JudgingPiece::Probe => "probe",
+        }
+    }
+}
+
+/// 逐件重做时一件的结局：做完了、panic 了（panic 的消息）、没跑（它吃的那一遍恢复 panic 了，或开关没开）。
+enum PieceOutcome<Value> {
+    Done(Value),
+    Panicked(String),
+    NotRun,
+}
+
+impl<Value> PieceOutcome<Value> {
+    fn done(&self) -> Option<&Value> {
+        match self {
+            PieceOutcome::Done(value) => Some(value),
+            PieceOutcome::Panicked(_) | PieceOutcome::NotRun => None,
+        }
+    }
+
+    /// `impl-verdicts.tsv` 里这一件的格：做完了照 `cell` 写，panic 写 `panic`，没跑写 `not_run`。
+    fn cell_text(&self, cell: impl FnOnce(&Value) -> String) -> String {
+        match self {
+            PieceOutcome::Done(value) => cell(value),
+            PieceOutcome::Panicked(_) => "panic".to_string(),
+            PieceOutcome::NotRun => "not_run".to_string(),
+        }
+    }
+}
+
+/// 在一个 `catch_unwind` 里做一件；`injected` 是这一件时先 panic（阳性对照「判态路 panic 单列」的注入，[`PiecePanicInjection`]）。
+fn judging_piece_outcome<Value>(
+    piece: JudgingPiece,
+    injected: Option<JudgingPiece>,
+    work: impl FnOnce() -> Value,
+) -> PieceOutcome<Value> {
+    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+        assert!(
+            injected != Some(piece),
+            "修订 5 阳性对照「判态路 panic 单列」注入：{} 这一件 panic",
+            piece.name()
+        );
+        work()
+    })) {
+        Ok(value) => PieceOutcome::Done(value),
+        Err(payload) => PieceOutcome::Panicked(panic_text(&*payload)),
+    }
+}
+
+/// 逐件重做时两份层 0 oracle 的结局：各自吃的那一遍恢复 panic 了是 None（记 `not_run`），跑了是它交回的（没有违例是 `Some(None)`）。
+struct Layer0OraclesOfThePieces {
+    consulted: Option<Option<Layer0OracleViolation>>,
+    ignored: Option<Option<Layer0OracleViolation>>,
+}
+
+/// 一件 panic 了就记下它的名字（[`StateObservation::piece_panics`]）。
+fn note_a_piece_panic<Value>(
+    piece_panics: &mut Vec<JudgingPiece>,
+    piece: JudgingPiece,
+    outcome: &PieceOutcome<Value>,
+) {
+    if let PieceOutcome::Panicked(_) = outcome {
+        piece_panics.push(piece);
+    }
+}
+
+/// 一组态上各件 panic 的态数，按 [`JudgingPiece::ALL`] 的次序每件一格（0 也在）。
+fn piece_panic_states_of<'observation>(
+    observations: impl IntoIterator<Item = &'observation StateObservation>,
+) -> [usize; JUDGING_PIECE_COUNT] {
+    let mut states = [0usize; JUDGING_PIECE_COUNT];
+    for observation in observations {
+        for piece in &observation.piece_panics {
+            states[piece.position()] += 1;
+        }
+    }
+    states
+}
+
+/// `name=batch`、`name=verdict` 两种行的 `piece_panic_states=` 字段：`<件>:<态数>` 逗号连起来，0 的件不写，全 0 写 `-`。
+fn piece_panic_states_text(states: &[usize; JUDGING_PIECE_COUNT]) -> String {
+    let nonzero: Vec<String> = JudgingPiece::ALL
+        .iter()
+        .zip(states)
+        .filter(|(_, count)| **count > 0)
+        .map(|(piece, count)| format!("{}:{count}", piece.name()))
+        .collect();
+    if nonzero.is_empty() {
+        "-".to_string()
+    } else {
+        nonzero.join(",")
+    }
+}
+
+/// `name=q5_piece_panics` 那一行：十件各一个字段（修订 5 第 5.1 节第 5 条）。
+fn q5_piece_panics_line(sampling_point: &str, observations: &[StateObservation]) -> String {
+    let states = piece_panic_states_of(observations);
+    let fields: Vec<String> = JudgingPiece::ALL
+        .iter()
+        .zip(states)
+        .map(|(piece, count)| format!("{}={count}", piece.name()))
+        .collect();
+    format!(
+        "name=q5_piece_panics sampling_point={sampling_point} judged_states={} {}",
+        observations.len(),
+        fields.join(" ")
+    )
+}
+
+/// 阳性对照「判态路 panic 单列」的注入（修订 5 第 5.1 节）：按落地串选态，选中的态正常那一路在回调里先 panic（逼走逐件重做那一路），
+/// 逐件重做时点名的那一件 panic。只经装置内部造（[`piece_panic_control_injection`] 与内联用例），命令行不开放；不改实现。
+struct PiecePanicInjection {
+    rules: Vec<PiecePanicInjectionRule>,
+}
+
+/// 一条注入：落地串满足全部条件的态，`piece` 那一件 panic。各条选中的态互不相交（造的人保证；[`PiecePanicInjection::piece_of`] 取第一条）。
+struct PiecePanicInjectionRule {
+    piece: JudgingPiece,
+    conditions: Vec<InjectedLandingCondition>,
+}
+
+/// 落地串上的一个条件（写表下标；「落」= 那一位不是 `0`）。
+enum InjectedLandingCondition {
+    Landed(usize),
+    NotLanded(usize),
+    BothOrNeitherLanded(usize, usize),
+}
+
+impl PiecePanicInjection {
+    /// 这一态（落地串）要注入哪一件 panic。
+    fn piece_of(&self, landing_vector: &str) -> Option<JudgingPiece> {
+        let landed = |write_index: usize| {
+            landing_vector
+                .as_bytes()
+                .get(write_index)
+                .is_some_and(|landing| *landing != b'0')
+        };
+        self.rules
+            .iter()
+            .find(|rule| {
+                rule.conditions.iter().all(|condition| match condition {
+                    InjectedLandingCondition::Landed(write_index) => landed(*write_index),
+                    InjectedLandingCondition::NotLanded(write_index) => !landed(*write_index),
+                    InjectedLandingCondition::BothOrNeitherLanded(first, second) => {
+                        landed(*first) == landed(*second)
+                    }
+                })
+            })
+            .map(|rule| rule.piece)
+    }
 }
 
 /// 一次观测的状态：阳性对照里「批 1 第一个读回了文件的态」只翻一次（O1-崩溃判官与 RefFS 各一次）、「见证」只改一次。
@@ -3529,6 +3856,8 @@ struct ObservationContext<'stream> {
     reference_filesystem: &'stream ReferenceFileSystemStream,
     /// 写表每一项落在哪一次调用里（[`call_index_of_each_write`]，整条流算一次）。
     call_of_write: Vec<Option<usize>>,
+    /// 阳性对照「判态路 panic 单列」的注入（只有那一遍有，别的都是 None）。
+    piece_panic_injection: Option<&'stream PiecePanicInjection>,
 }
 
 impl ObservationContext<'_> {
@@ -3563,6 +3892,14 @@ fn observation_of_a_judged_state(
         stream.writes.len(),
         context.torn_image_index_by_write,
     );
+    // 阳性对照「判态路 panic 单列」（修订 5 第 5.1 节）：选中的态在回调里先 panic，逼走逐件重做那一路。
+    assert!(
+        context
+            .piece_panic_injection
+            .and_then(|injection| injection.piece_of(&landing_vector))
+            .is_none(),
+        "修订 5 阳性对照「判态路 panic 单列」注入：正常那一路在这一态的回调里先 panic"
+    );
     let touched_call = landing_vector
         .chars()
         .zip(&context.call_of_write)
@@ -3676,27 +4013,32 @@ fn observation_of_a_judged_state(
             ConsultedReports::Dropped => None,
             ConsultedReports::Kept => Some(judgement.consulted.clone()),
         },
+        piece_panics: Vec::new(),
     }
 }
 
-/// 恢复 panic 的那一态（`judge_crash_image` 里 panic，或判这一态的 oracle 里 panic）：镜像按事实表的掩码现造，O2 照判（它也 panic 记红不了，
-/// 写 `pool_checker_panic`），O1 两个崩溃子列与探针不适用（读法 5「态上的恢复 panic」）。
+/// 回调链 panic 了的那一态（`judge_crash_image` 里 panic，或判这一态的 oracle 里 panic）：镜像按事实表的掩码现造，照修订 5 第 5.1 节第 2 条
+/// 的次序逐件重做、每件各包一层 `catch_unwind`（[`JudgingPiece`]），panic 了记下件名、接着做下一件。O1 两个崩溃子列只随看 journal 那遍恢复：
+/// 它 panic 两列都记恢复 panic（读法 5），没 panic 两列照正常那一路的函数从这一遍的恢复报告算，O1-模型崩溃判官或 RefFS 自己 panic 那一列记
+/// `judge_panic`；别的件 panic 不改这两列（第 3 条）。O2 照判，panic 记 `pool_checker_panic`、按红计（第 4 条）。
+/// 阳性对照的翻读回、见证改写与 O2 的坏镜像都只在正常那一路做，这一路不做。
 fn observation_of_a_panicking_state(
     context: &ObservationContext<'_>,
     batch: u8,
     ordinal: u64,
-    panic_message: &str,
     switches: RunSwitches,
+    consulted_reports: ConsultedReports,
 ) -> StateObservation {
+    let stream = context.stream;
     let persisted = context.facts.persisted_writes_of_state(ordinal);
     let image = CrashImage {
-        base: &context.stream.base,
+        base: &stream.base,
         writes: &context.tables.enumerated_writes,
         persisted,
     };
     let landing_vector = landing_vector_of(
         &image.persisted,
-        context.stream.writes.len(),
+        stream.writes.len(),
         context.torn_image_index_by_write,
     );
     let touched_call = landing_vector
@@ -3705,42 +4047,208 @@ fn observation_of_a_panicking_state(
         .filter(|(landing, _)| *landing != '0')
         .filter_map(|(_, call)| *call)
         .max();
+    let segment_index = context.segment_of_ordinal(ordinal);
+    let injected = context
+        .piece_panic_injection
+        .and_then(|injection| injection.piece_of(&landing_vector));
+    let mut piece_panics: Vec<JudgingPiece> = Vec::new();
+    let consulted = judging_piece_outcome(JudgingPiece::ConsultedRecovery, injected, || {
+        singlefs_core::recovery::recover(&image, singlefs_core::recovery::JournalPolicy::Consult)
+    });
+    note_a_piece_panic(
+        &mut piece_panics,
+        JudgingPiece::ConsultedRecovery,
+        &consulted,
+    );
+    let ignored = judging_piece_outcome(JudgingPiece::IgnoredRecovery, injected, || {
+        singlefs_core::recovery::recover(&image, singlefs_core::recovery::JournalPolicy::Ignore)
+    });
+    note_a_piece_panic(&mut piece_panics, JudgingPiece::IgnoredRecovery, &ignored);
+    let root_persisted = judging_piece_outcome(JudgingPiece::RootPersisted, injected, || {
+        image.persisted[stream.judged_root_index]
+    });
+    note_a_piece_panic(
+        &mut piece_panics,
+        JudgingPiece::RootPersisted,
+        &root_persisted,
+    );
+    // 两份层 0 oracle 各吃自己那一遍恢复：那一遍 panic 了，那一份记 `not_run`；两遍都 panic 了这一件整个不跑。
+    let layer0_oracles = if consulted.done().is_none() && ignored.done().is_none() {
+        PieceOutcome::NotRun
+    } else {
+        judging_piece_outcome(JudgingPiece::Layer0Oracles, injected, || {
+            let newest_persisted = newest_persisted_root(image.writes, &image.persisted);
+            let oracle_of = |report: &RecoveryReport| {
+                classified_oracle_violation_for_versions(
+                    &report.outcome,
+                    report.effective_root,
+                    newest_persisted,
+                    &stream.versions,
+                )
+            };
+            Layer0OraclesOfThePieces {
+                consulted: consulted.done().map(oracle_of),
+                ignored: ignored.done().map(oracle_of),
+            }
+        })
+    };
+    note_a_piece_panic(
+        &mut piece_panics,
+        JudgingPiece::Layer0Oracles,
+        &layer0_oracles,
+    );
     let pool_checker_started = Instant::now();
-    let pool_checker_red =
-        (switches.disabled_oracle != Some(DisabledOracle::PoolChecker)).then(|| {
-            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
-                violated_invariant_names(&check_pool_image(&image))
-            }))
-            .unwrap_or_else(|_| vec!["pool_checker_panic".to_string()])
-        });
+    let pool_checker = judging_piece_outcome(JudgingPiece::PoolChecker, injected, || {
+        check_pool_image(&image)
+    });
     let o2_nanoseconds = pool_checker_started.elapsed().as_nanos();
-    let segment_index = context.segment_of_ordinal(ordinal);
+    note_a_piece_panic(&mut piece_panics, JudgingPiece::PoolChecker, &pool_checker);
+    let record_checker = match consulted.done() {
+        Some(report) => judging_piece_outcome(JudgingPiece::RecordChecker, injected, || {
+            check_records(&image, report.effective_root)
+        }),
+        None => PieceOutcome::NotRun,
+    };
+    note_a_piece_panic(
+        &mut piece_panics,
+        JudgingPiece::RecordChecker,
+        &record_checker,
+    );
+    // 两个崩溃子列只随看 journal 那遍恢复 panic（第 3 条）：别的件 panic 不改这两列。
+    let columns_follow_a_recovery_panic = matches!(consulted, PieceOutcome::Panicked(_));
+    let model_is_enabled =
+        switches.disabled_oracle != Some(DisabledOracle::ComparisonAndCrashJudge);
+    let moment = context.reference_filesystem_moment_of(segment_index);
+    let crash_judge_started = Instant::now();
+    let crash_judge = match consulted.done() {
+        _ if columns_follow_a_recovery_panic => CrashJudgeVerdict::RecoveryPanicked,
+        None => CrashJudgeVerdict::RecoveryPanicked,
+        Some(_) if !model_is_enabled => CrashJudgeVerdict::Off,
+        Some(_) if !crash_judge_is_applicable(touched_call, stream.model_stopped_at_call) => {
+            CrashJudgeVerdict::NotApplicable
+        }
+        Some(report) => {
+            let outcome = judging_piece_outcome(JudgingPiece::CrashJudge, injected, || {
+                crash_judge_of_the_state(stream, &image, &observed_read_back_after_a_crash(report))
+            });
+            note_a_piece_panic(&mut piece_panics, JudgingPiece::CrashJudge, &outcome);
+            match outcome {
+                PieceOutcome::Done(verdict) => verdict,
+                PieceOutcome::Panicked(_) | PieceOutcome::NotRun => {
+                    CrashJudgeVerdict::JudgePanicked
+                }
+            }
+        }
+    };
+    let reference_filesystem = match consulted.done() {
+        _ if columns_follow_a_recovery_panic => ReferenceFileSystemVoteOfAState::RecoveryPanicked,
+        None => ReferenceFileSystemVoteOfAState::RecoveryPanicked,
+        Some(_) if !model_is_enabled => ReferenceFileSystemVoteOfAState::Off,
+        Some(report) => {
+            let outcome =
+                judging_piece_outcome(JudgingPiece::ReferenceFileSystem, injected, || {
+                    reference_filesystem_vote_of_a_state(
+                        context.reference_filesystem,
+                        moment,
+                        &observed_read_back_after_a_crash(report),
+                    )
+                });
+            note_a_piece_panic(
+                &mut piece_panics,
+                JudgingPiece::ReferenceFileSystem,
+                &outcome,
+            );
+            match outcome {
+                PieceOutcome::Done(vote) => vote,
+                PieceOutcome::Panicked(_) | PieceOutcome::NotRun => {
+                    ReferenceFileSystemVoteOfAState::JudgePanicked
+                }
+            }
+        }
+    };
+    let o1_nanoseconds = crash_judge_started.elapsed().as_nanos();
+    let recovery_started = Instant::now();
+    let timing_recovery = judging_piece_outcome(JudgingPiece::TimingRecovery, injected, || {
+        let _ = singlefs_core::recovery::recover(
+            &image,
+            singlefs_core::recovery::JournalPolicy::Consult,
+        );
+    });
+    let recovery_nanoseconds = recovery_started.elapsed().as_nanos();
+    note_a_piece_panic(
+        &mut piece_panics,
+        JudgingPiece::TimingRecovery,
+        &timing_recovery,
+    );
+    let probe = match (switches.writable_mount_probe, consulted.done()) {
+        (true, Some(report)) => {
+            let outcome = judging_piece_outcome(JudgingPiece::Probe, injected, || {
+                writable_mount_probe_on(
+                    &materialized_crash_image(&image),
+                    report.effective_root,
+                    &crash_judge,
+                )
+            });
+            note_a_piece_panic(&mut piece_panics, JudgingPiece::Probe, &outcome);
+            match outcome {
+                PieceOutcome::Done(probe) => Some(probe),
+                PieceOutcome::Panicked(_) | PieceOutcome::NotRun => None,
+            }
+        }
+        (true, None) | (false, _) => None,
+    };
+    let pool_checker_red = (switches.disabled_oracle != Some(DisabledOracle::PoolChecker)).then(
+        || match &pool_checker {
+            PieceOutcome::Done(verdicts) => violated_invariant_names(verdicts),
+            PieceOutcome::Panicked(_) | PieceOutcome::NotRun => {
+                vec!["pool_checker_panic".to_string()]
+            }
+        },
+    );
+    let verdict = ImplementationVerdict::of_the_pieces(
+        &consulted,
+        &ignored,
+        &layer0_oracles,
+        &pool_checker,
+        &record_checker,
+    );
     StateObservation {
         batch,
         ordinal,
         segment_index,
         landing_vector,
         image_digest: sha256_text(&crash_image_digest(&image)),
-        verdict: ImplementationVerdict::of_a_panicking_recovery(),
-        judge_nanoseconds: o2_nanoseconds,
+        verdict,
+        judge_nanoseconds: o1_nanoseconds + o2_nanoseconds + recovery_nanoseconds,
         mask_matches_the_facts: true,
         touched_call,
-        crash_judge: CrashJudgeVerdict::RecoveryPanicked,
-        reference_filesystem: ReferenceFileSystemVoteOfAState::RecoveryPanicked,
-        reference_filesystem_crash_moment: crash_moment_text(
-            context.reference_filesystem_moment_of(segment_index),
-        ),
+        crash_judge,
+        reference_filesystem,
+        reference_filesystem_crash_moment: crash_moment_text(moment),
         pool_checker_red,
         pool_checker_matches_the_judgement: true,
-        consulted: RecoverySummary::of_a_panicking_recovery(),
-        ignored: RecoverySummary::of_a_panicking_recovery(),
-        consulted_root: None,
-        probe: None,
-        recovery_panic: Some(panic_message.to_string()),
-        o1_nanoseconds: 0,
+        consulted: consulted.done().map_or_else(
+            RecoverySummary::of_a_panicking_recovery,
+            RecoverySummary::of,
+        ),
+        ignored: ignored.done().map_or_else(
+            RecoverySummary::of_a_panicking_recovery,
+            RecoverySummary::of,
+        ),
+        consulted_root: consulted.done().and_then(|report| report.effective_root),
+        probe,
+        recovery_panic: match &consulted {
+            PieceOutcome::Panicked(message) => Some(message.clone()),
+            PieceOutcome::Done(_) | PieceOutcome::NotRun => None,
+        },
+        o1_nanoseconds,
         o2_nanoseconds,
-        recovery_nanoseconds: 0,
-        consulted_report: None,
+        recovery_nanoseconds,
+        consulted_report: match (consulted_reports, consulted.done()) {
+            (ConsultedReports::Kept, Some(report)) => Some(report.clone()),
+            (ConsultedReports::Kept, None) | (ConsultedReports::Dropped, _) => None,
+        },
+        piece_panics,
     }
 }
 
@@ -3834,13 +4342,14 @@ fn observe_sampled_states(
             .collect::<Vec<u64>>(),
         "panic 的态两处记的一样"
     );
-    for (ordinal, message) in panicking_states {
+    // 正常那一路 panic 的消息不进产物：逐件重做时各件自己的结局才是这一态的判定（修订 5 第 5.1 节第 2 条）。
+    for (ordinal, _message_of_the_judging_path) in panicking_states {
         observations.push(observation_of_a_panicking_state(
             context,
             batch_of_ordinal[&ordinal],
             ordinal,
-            &message,
             switches,
+            consulted_reports,
         ));
     }
     observations.sort_by_key(|observation| (observation.batch, observation.ordinal));
@@ -4105,7 +4614,8 @@ fn stream_status_file_text(stream: &PreparedStream) -> String {
 /// O1-模型崩溃判官（`green` / `red` / `na` / `off` / `recovery_panic`）与红的内容、O2（同）与判红的不变量、O2 直接调与
 /// `judge_crash_image` 内那一遍是否一致、看 journal 那一遍恢复的摘要（根、结局、读回 sha256、施加记录数）、不看 journal 那一遍的摘要、
 /// 层 0 的 `red_items`（附带），加第 1 次重跑的五列：O1-RefFS 的票（`green` / `red` / `abstained` / `recovery_panic` / `off`）、
-/// 弃权种类或红的理由、RefFS 的崩溃时刻、允许集合的下标、恢复 panic 的消息（没有写 `-`）。
+/// 弃权种类或红的理由、RefFS 的崩溃时刻、允许集合的下标、看 journal 那一遍恢复 panic 的消息（没有写 `-`），加修订 5 第 5.1 节第 5 条的
+/// `piece_panics`：回调链 panic 了的态上逐件重做时 panic 的件名，按 [`JudgingPiece`] 的次序逗号连起来（没有写 `-`）。
 fn oracle_verdicts_file_text(observations: &[StateObservation]) -> String {
     observations
         .iter()
@@ -4121,7 +4631,8 @@ fn oracle_verdicts_file_text(observations: &[StateObservation]) -> String {
                 CrashJudgeVerdict::Green
                 | CrashJudgeVerdict::NotApplicable
                 | CrashJudgeVerdict::Off
-                | CrashJudgeVerdict::RecoveryPanicked => "-".to_string(),
+                | CrashJudgeVerdict::RecoveryPanicked
+                | CrashJudgeVerdict::JudgePanicked => "-".to_string(),
             };
             let (reference_filesystem_detail, allowed) = match &observation.reference_filesystem {
                 ReferenceFileSystemVoteOfAState::Green { allowed } => ("-".to_string(), allowed.clone()),
@@ -4132,10 +4643,11 @@ fn oracle_verdicts_file_text(observations: &[StateObservation]) -> String {
                     (field_text(kind), "-".to_string())
                 }
                 ReferenceFileSystemVoteOfAState::RecoveryPanicked
+                | ReferenceFileSystemVoteOfAState::JudgePanicked
                 | ReferenceFileSystemVoteOfAState::Off => ("-".to_string(), "-".to_string()),
             };
             format!(
-                "{state_number}\t{}\t{}\t{}\t{}\t{}\t{o1_detail}\t{o2_word}\t{o2_items}\t{}\t{}\t{}\t{}\t{}\t{reference_filesystem_detail}\t{}\t{allowed}\t{}\n",
+                "{state_number}\t{}\t{}\t{}\t{}\t{}\t{o1_detail}\t{o2_word}\t{o2_items}\t{}\t{}\t{}\t{}\t{}\t{reference_filesystem_detail}\t{}\t{allowed}\t{}\t{}\n",
                 observation.batch,
                 observation.ordinal,
                 observation.segment_index,
@@ -4153,11 +4665,25 @@ fn oracle_verdicts_file_text(observations: &[StateObservation]) -> String {
                     .recovery_panic
                     .as_deref()
                     .map_or_else(|| "-".to_string(), |message| field_text(&cell_text(message))),
+                piece_panics_text(&observation.piece_panics),
             )
         })
         .collect()
 }
 
+/// 一态的 `piece_panics` 列：件名按 [`JudgingPiece`] 的次序逗号连起来，没有写 `-`。
+fn piece_panics_text(piece_panics: &[JudgingPiece]) -> String {
+    if piece_panics.is_empty() {
+        "-".to_string()
+    } else {
+        piece_panics
+            .iter()
+            .map(|piece| piece.name())
+            .collect::<Vec<&str>>()
+            .join(",")
+    }
+}
+
 /// `writable-mount-probe.tsv`（读法 8，登记 5.3 第 6 样）：每个探过的态一行——流名、批号、层 0 序号、状态号、(i) 判不出那一支、
 /// (ii) 计数那一支（为真写是哪一句，假写 `false`）、c_见证、所选版本、挂载结局、拒的成员、单子上的理由、第二截判定。
 fn writable_mount_probe_file_text(stream: StreamName, observations: &[StateObservation]) -> String {
@@ -4590,6 +5116,7 @@ fn export_prepared_stream(
         torn_image_index_by_write: &torn_image_index_by_write,
         reference_filesystem: &reference_filesystem,
         call_of_write: call_index_of_each_write(stream),
+        piece_panic_injection: None,
     };
     let observe_started = Instant::now();
     let observations = observe_sampled_states(&context, &sampled, switches, consulted_reports);
@@ -4696,19 +5223,28 @@ fn state_count_checks_against_the_crates(
 /// `E7RESULT` 行计数（收尾 `name=done emitted=N`，N 把 done 自己也算进去）；`E7TIMING` 行不计数、不进入库产物。
 struct ResultLines {
     emitted: u64,
+    /// 收集模式（只在内联用例里用，修订 5 第 5.7 节 U8）：结果行收进这里、不打到标准输出；None 时照常打。
+    collected: Option<Vec<String>>,
 }
 
 impl ResultLines {
     fn result(&mut self, body: &str) {
         self.emitted += 1;
-        println!("E7RESULT {body}");
+        match &mut self.collected {
+            Some(collected) => collected.push(body.to_string()),
+            None => println!("E7RESULT {body}"),
+        }
     }
     fn timing(body: &str) {
         println!("E7TIMING {body}");
     }
     fn finish(&mut self) {
         self.emitted += 1;
-        println!("E7RESULT name=done emitted={}", self.emitted);
+        let body = format!("name=done emitted={}", self.emitted);
+        match &mut self.collected {
+            Some(collected) => collected.push(body),
+            None => println!("E7RESULT {body}"),
+        }
     }
 }
 
@@ -6330,6 +6866,8 @@ struct StreamTotals {
     reference_filesystem_red_states: usize,
     pool_checker_red_states: usize,
     recovery_panic_states: usize,
+    /// 各件 panic 的态数（修订 5 第 5.1 节第 5 条，按 [`JudgingPiece::ALL`] 的次序）。
+    piece_panic_states: [usize; JUDGING_PIECE_COUNT],
     probed_states: usize,
     /// (ii) 为真的态、其中第二截判合法的、(i) 为真的态（Q4）。
     counter_branch_states: usize,
@@ -6474,12 +7012,16 @@ fn report_batch(
     totals.pool_checker_red_states += o2_red;
     totals.recovery_panic_states +=
         count_where(&|observation| observation.recovery_panic.is_some());
+    let piece_panic_states = piece_panic_states_of(in_batch.iter().copied());
+    for (total, states) in totals.piece_panic_states.iter_mut().zip(piece_panic_states) {
+        *total += states;
+    }
     let segments: BTreeSet<usize> = in_batch
         .iter()
         .map(|observation| observation.segment_index)
         .collect();
     lines.result(&format!(
-        "name=batch stream={label} batch={batch} states={} segments_touched={} states_with_a_torn_write={} o1_crash_judge_red={} o1_crash_judge_green={} o1_crash_judge_not_applicable={} o1_crash_judge_off={} o1_reference_filesystem_red={} o1_reference_filesystem_green={} o1_reference_filesystem_abstained={} o1_reference_filesystem_off={} recovery_panic_states={} o2_red={o2_red} o2_matches_judgement_false={} judge_crash_image_pool_checker_red={} record_checker_red_states={} consulted_oracle_red_states={} ignored_oracle_red_states={} consulted_effective_root_none={} consulted_failed_outcomes={}",
+        "name=batch stream={label} batch={batch} states={} segments_touched={} states_with_a_torn_write={} o1_crash_judge_red={} o1_crash_judge_green={} o1_crash_judge_not_applicable={} o1_crash_judge_off={} o1_reference_filesystem_red={} o1_reference_filesystem_green={} o1_reference_filesystem_abstained={} o1_reference_filesystem_off={} recovery_panic_states={} o2_red={o2_red} o2_matches_judgement_false={} judge_crash_image_pool_checker_red={} record_checker_red_states={} consulted_oracle_red_states={} ignored_oracle_red_states={} consulted_effective_root_none={} consulted_failed_outcomes={} piece_panic_states={}",
         in_batch.len(),
         segments.len(),
         count_where(&|observation| observation.landing_vector.contains('1')),
@@ -6499,6 +7041,7 @@ fn report_batch(
         count_where(&|observation| observation.verdict.red_items.iter().any(|item| item.starts_with("journal_ignored_oracle:"))),
         count_where(&|observation| observation.verdict.consulted_root == "none"),
         count_where(&|observation| observation.verdict.consulted_outcome.starts_with("failed:")),
+        piece_panic_states_text(&piece_panic_states),
     ));
     let abstention_kinds: BTreeMap<&'static str, usize> =
         in_batch
@@ -6864,7 +7407,10 @@ fn stream_independent_checks_of_the_first_rerun() -> Vec<StopCheck> {
 fn run_dump(command_line: &CommandLine) -> i32 {
     let run_kind = command_line.run_kind;
     let switches = command_line.switches;
-    let mut lines = ResultLines { emitted: 0 };
+    let mut lines = ResultLines {
+        emitted: 0,
+        collected: None,
+    };
     lines.result(&format!(
         "name=config segment=s1 mode={} streams={} sector_bytes={LOCAL_SECTOR_BYTES} first_batch_divisor={FIRST_BATCH_DIVISOR} second_batch_divisor={SECOND_BATCH_DIVISOR} third_batch_divisor={THIRD_BATCH_DIVISOR} torn_states_per_segment={TORN_STATES_PER_SEGMENT} torn_scan_ordinals_per_segment={TORN_SCAN_ORDINALS_PER_SEGMENT} third_batch_sampled={} positive_control={} disabled_oracle={} writable_mount_probe={}",
         if run_kind == RunKind::Baseline { "baseline" } else { "row" },
@@ -6928,7 +7474,7 @@ fn run_dump(command_line: &CommandLine) -> i32 {
     }
     failed_checks += totals.failed_checks;
     lines.result(&format!(
-        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={} crash_facts_mask_mismatches={} s3_o2_judgement_mismatches={} s9_failed_checks={s9_failed} streams_not_exported={streams_not_exported} o1_comparison_red_calls={} o1_crash_judge_red_states={} o1_reference_filesystem_red_states={} o2_red_states={} recovery_panic_states={} probed_states={} counter_branch_states={} counter_branch_states_judged_legal={} undecidable_branch_states={} witness_refusals_without_either_branch={} counter_branch_states_that_mounted={}",
+        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={} crash_facts_mask_mismatches={} s3_o2_judgement_mismatches={} s9_failed_checks={s9_failed} streams_not_exported={streams_not_exported} o1_comparison_red_calls={} o1_crash_judge_red_states={} o1_reference_filesystem_red_states={} o2_red_states={} recovery_panic_states={} probed_states={} counter_branch_states={} counter_branch_states_judged_legal={} undecidable_branch_states={} witness_refusals_without_either_branch={} counter_branch_states_that_mounted={} piece_panic_states={}",
         totals.rebuilt_digest_mismatches,
         totals.mask_mismatches,
         totals.s3_mismatches,
@@ -6943,6 +7489,7 @@ fn run_dump(command_line: &CommandLine) -> i32 {
         totals.undecidable_branch_states,
         totals.witness_refusals_without_either_branch,
         totals.counter_branch_states_that_mounted,
+        piece_panic_states_text(&totals.piece_panic_states),
     ));
     lines.finish();
     let s11_hit = totals.witness_refusals_without_either_branch > 0
@@ -6970,6 +7517,13 @@ fn add_stream_totals(totals: &mut StreamTotals, stream: &StreamTotals) {
     totals.reference_filesystem_red_states += stream.reference_filesystem_red_states;
     totals.pool_checker_red_states += stream.pool_checker_red_states;
     totals.recovery_panic_states += stream.recovery_panic_states;
+    for (total, states) in totals
+        .piece_panic_states
+        .iter_mut()
+        .zip(stream.piece_panic_states)
+    {
+        *total += states;
+    }
     totals.probed_states += stream.probed_states;
     totals.counter_branch_states += stream.counter_branch_states;
     totals.counter_branch_states_judged_legal += stream.counter_branch_states_judged_legal;
@@ -7219,7 +7773,36 @@ fn confirmed_loss_form_checks_against_the_crates() -> Vec<StopCheck> {
     );
     let key_choices = partition.keyed_assignments(&digits);
     let states_of_a_landed_class = partition.states_with_the_in_flight_writes_landed(&digits);
+    // 「等价类对拍」那一形的三样（修订 5 第 5.2 节 S1 新加三条）。
+    let (pairing_stream, _) = confirmed_loss_prepared_stream(
+        second_stream_through_the_first_overwrite(RunSwitches::default()),
+        ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
+            EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT,
+        ),
+    );
+    let pairing_tables = layer0_enumeration_tables(
+        &pairing_stream.base,
+        &pairing_stream.writes,
+        &pairing_stream.segments,
+        &full_expansion,
+    );
+    let pairing_facts = equivalence_pairing_shape_facts(&pairing_stream, &pairing_tables);
     vec![
+        StopCheck::new(
+            "equivalence_pairing_in_flight_writes",
+            LOCAL_EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES,
+            pairing_facts.in_flight_writes.len(),
+        ),
+        StopCheck::new(
+            "equivalence_pairing_states",
+            LOCAL_EQUIVALENCE_PAIRING_STATES,
+            pairing_facts.states,
+        ),
+        StopCheck::new(
+            "equivalence_pairing_mirrored_pairs",
+            LOCAL_EQUIVALENCE_PAIRING_MIRRORED_PAIRS,
+            pairing_facts.mirrored_pairs,
+        ),
         StopCheck::new(
             "confirmed_loss_key_choices",
             LOCAL_CONFIRMED_LOSS_KEY_CHOICES,
@@ -7300,9 +7883,13 @@ fn confirmed_loss_form_checks_against_the_crates() -> Vec<StopCheck> {
 
 /// `confirmed-loss-form` 模式：造那一形、每态算 O1 三个子列与附带的 O2，导出中性格式（`<输出>/confirmed_loss_form/<流二目录名>/`），
 /// 打 Q5 的计数（读法 7、第六节 Q5-有效 / Q5-判 / Q5-阴性、第八节轨迹）。方向相反的取样点末段全枚举；主取样点按等价类判
-/// （登记修订 2 第 2.1 节候选二、修订 3）：先过「等价类对拍」，再按类判主取样点，类核对 ①–④ 或对拍不过走停机 S13。
+/// （登记修订 2 第 2.1 节候选二、修订 3）：先过「等价类对拍」与阳性对照「判态路 panic 单列」，再按类判主取样点，类核对 ①–④ 或对拍不过走停机 S13。
+/// 次序照修订 5 第 5.3 节：S1 → 造主取样点（或反向取样点）那一形 → 造对拍那一形 → [`judge_the_confirmed_loss_shapes`]（先看两形的根写）。
 fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
-    let mut lines = ResultLines { emitted: 0 };
+    let mut lines = ResultLines {
+        emitted: 0,
+        collected: None,
+    };
     let truncation = if command_line.truncate_at_the_return_point {
         ConfirmedLossTruncation::AtTheConfirmedReturnPoint
     } else {
@@ -7327,17 +7914,87 @@ fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
             check.holds()
         ));
     }
-    if !command_line.truncate_at_the_return_point {
-        // 装置先过「等价类对拍」再跑主取样点（修订 2 第 2.1 节「定之前与定之后」末句）。
-        let (pairing_stream, _) = confirmed_loss_prepared_stream(
+    let started = Instant::now();
+    let live = second_stream_through_the_first_overwrite(RunSwitches::default());
+    let (prepared, shape) = confirmed_loss_prepared_stream(live, truncation);
+    let drive_nanoseconds = started.elapsed().as_nanos();
+    // 方向相反的取样点没有对拍那一形（修订 5 第 5.3 节：反向取样点只在导出之前先判它自己那一形）。
+    let pairing = (!command_line.truncate_at_the_return_point).then(|| {
+        confirmed_loss_prepared_stream(
             second_stream_through_the_first_overwrite(RunSwitches::default()),
             ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
                 EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT,
             ),
-        );
-        let pairing = equivalence_pairing_of(&pairing_stream);
+        )
+        .0
+    });
+    judge_the_confirmed_loss_shapes(
+        &mut lines,
+        command_line,
+        ConfirmedLossShapes {
+            sampling_point,
+            main: prepared,
+            shape,
+            drive_nanoseconds,
+            pairing,
+        },
+        failed_checks,
+    )
+}
+
+/// Q5 那一次判态要的两形：主取样点（或反向取样点）那一形与它的造法下标、驱动它的耗时，与对拍那一形（反向取样点没有）。
+struct ConfirmedLossShapes {
+    sampling_point: &'static str,
+    main: PreparedStream,
+    shape: ConfirmedLossShape,
+    drive_nanoseconds: u128,
+    pairing: Option<PreparedStream>,
+}
+
+/// 两形里头一个没有可判的根槽写的（`judged_root_index` 是 `usize::MAX`）：`main` 或 `pairing`（修订 5 第 5.3 节）；都有交 `None`。
+fn shape_without_a_root_write(shapes: &ConfirmedLossShapes) -> Option<&'static str> {
+    if shapes.main.judged_root_index == usize::MAX {
+        return Some("main");
+    }
+    if shapes
+        .pairing
+        .as_ref()
+        .is_some_and(|pairing| pairing.judged_root_index == usize::MAX)
+    {
+        return Some("pairing");
+    }
+    None
+}
+
+/// `run_confirmed_loss_form` 判完 S1 之后的那一段：先看两形的根写（修订 5 第 5.3 节），再对拍、阳性对照「判态路 panic 单列」、
+/// 主取样点的分类与判态（反向取样点全枚举）；结果行打进 `lines`，交回退出码（0 或 3）。
+fn judge_the_confirmed_loss_shapes(
+    lines: &mut ResultLines,
+    command_line: &ConfirmedLossCommandLine,
+    shapes: ConfirmedLossShapes,
+    mut failed_checks: usize,
+) -> i32 {
+    let sampling_point = shapes.sampling_point;
+    if let Some(shape_name) = shape_without_a_root_write(&shapes) {
+        // 那一形截出来的写流里没有可判的根槽写：对拍、阳性对照与类核对都不跑，不是 S13；Q5 记「不够判」、不进判定。
+        lines.result(&format!(
+            "name=q5 sampling_point={sampling_point} not_run=no_root_write shape={shape_name}"
+        ));
+        lines.result(&format!(
+            "name=verdict s1_failed_checks={failed_checks} s13=false q5_not_run=no_root_write piece_panic_states=-"
+        ));
+        lines.finish();
+        return 3;
+    }
+    if let Some(pairing_stream) = &shapes.pairing {
+        // 装置先过「等价类对拍」再跑主取样点（修订 2 第 2.1 节「定之前与定之后」末句）。
+        let pairing = equivalence_pairing_of(pairing_stream);
+        let in_flight_writes_kept = call_index_of_each_write(pairing_stream)
+            .iter()
+            .filter(|call| **call == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL))
+            .count();
         lines.result(&format!(
-            "name=q5_equivalence_pairing in_flight_writes_kept={EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT} states={} product_of_the_choices={} class_states={} classes={} mismatching_states={} crash_judge_not_applicable_states={} {} {}",
+            "name=q5_equivalence_pairing in_flight_writes_kept={in_flight_writes_kept} states={} product_of_the_choices={} class_states={} classes={} mismatching_states={} crash_judge_not_applicable_states={} {} s13={}",
             pairing.states,
             pairing.product_of_the_choices,
             pairing.class_states,
@@ -7349,20 +8006,66 @@ fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
                 .filter(|observation| observation.crash_judge == CrashJudgeVerdict::NotApplicable)
                 .count(),
             equivalence_check_fields(&pairing.classification, &pairing.checks),
+            pairing.stops_at_s13()
+        ));
+        lines.result(&format!(
+            "name=q5_partition sampling_point=pairing {}",
             partition_fields(&pairing.classification, &pairing.checks)
         ));
+        lines.result(&q5_piece_panics_line("pairing", &pairing.observations));
+        // S1（修订 5 第 5.2 节）：对拍那一行打的实际截的 k 与分法里的 C 写数相等。
+        let kept_check = StopCheck::new(
+            "equivalence_pairing_in_flight_writes_kept_equal_the_partition",
+            in_flight_writes_kept,
+            pairing.classification.in_flight_writes,
+        );
+        failed_checks += usize::from(!kept_check.holds());
+        lines.result(&format!(
+            "name=s1 stream=confirmed_loss_form check={} local={} crates={} holds={}",
+            kept_check.name,
+            field_text(&kept_check.local),
+            field_text(&kept_check.crates),
+            kept_check.holds()
+        ));
         if pairing.stops_at_s13() {
             lines.result(&format!(
-                "name=verdict s1_failed_checks={failed_checks} s13=equivalence_pairing q5_not_run=equivalence_pairing_failed"
+                "name=verdict s1_failed_checks={failed_checks} s13=equivalence_pairing q5_not_run=equivalence_pairing_failed piece_panic_states=-"
+            ));
+            lines.finish();
+            return 3;
+        }
+        let control = piece_panic_control_of(pairing_stream, &pairing);
+        lines.result(&format!(
+            "name=q5_piece_panic_control states={} pool_checker={} ignored_recovery={} record_checker={} other_pieces={} consulted_recovery={} consulted_recovery_of_the_plain_pairing={} o1_columns_differing_from_the_plain_pairing={} mismatching_states={} s13={} holds={}",
+            control.states,
+            control.piece_panic_states[JudgingPiece::PoolChecker.position()],
+            control.piece_panic_states[JudgingPiece::IgnoredRecovery.position()],
+            control.piece_panic_states[JudgingPiece::RecordChecker.position()],
+            control.other_piece_panic_states(),
+            control.piece_panic_states[JudgingPiece::ConsultedRecovery.position()],
+            control.consulted_recovery_panics_of_the_plain_pairing,
+            control.o1_columns_differing_from_the_plain_pairing,
+            control.mismatching_states,
+            control.stops_at_s13,
+            control.holds()
+        ));
+        lines.result(&q5_piece_panics_line(
+            "piece_panic_control",
+            &control.observations,
+        ));
+        if !control.holds() {
+            // 失败 F9（修订 5 第 5.7 节）：甲那条管子接错了，主取样点不跑、不记成「O1 盲」。
+            lines.result(&format!(
+                "name=verdict s1_failed_checks={failed_checks} s13=false q5_not_run=piece_panic_control_failed piece_panic_states=-"
             ));
             lines.finish();
             return 3;
         }
     }
     let started = Instant::now();
-    let live = second_stream_through_the_first_overwrite(RunSwitches::default());
-    let (prepared, shape) = confirmed_loss_prepared_stream(live, truncation);
-    let drive_nanoseconds = started.elapsed().as_nanos();
+    let prepared = shapes.main;
+    let shape = shapes.shape;
+    let drive_nanoseconds = shapes.drive_nanoseconds;
     let confirmed_root = prepared
         .writes
         .iter()
@@ -7399,7 +8102,7 @@ fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
                 "name=q5 sampling_point={sampling_point} not_run=too_many_states states={product} limit={CONFIRMED_LOSS_STATE_LIMIT}"
             ));
             lines.result(&format!(
-                "name=verdict s1_failed_checks={failed_checks} q5_not_run=too_many_states"
+                "name=verdict s1_failed_checks={failed_checks} q5_not_run=too_many_states piece_panic_states=-"
             ));
             lines.finish();
             return 3;
@@ -7415,14 +8118,15 @@ fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
         );
         let s9 = s9_checks_of_a_stream(&export);
         export.checks.extend(s9);
-        let totals = report_stream(&mut lines, &export);
+        let totals = report_stream(lines, &export);
+        lines.result(&q5_piece_panics_line("reverse", &export.observations));
         let rows: Vec<ConfirmedLossTallyRow<'_>> = export
             .observations
             .iter()
             .map(|observation| ConfirmedLossTallyRow::of_a_judged_state(observation, &last_segment))
             .collect();
         report_the_confirmed_loss(
-            &mut lines,
+            lines,
             sampling_point,
             &rows,
             confirmed_root,
@@ -7441,15 +8145,16 @@ fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
         );
         if classification.stopped_at_the_state_limit {
             lines.result(&format!(
-                "name=q5_equivalence sampling_point={sampling_point} {}",
-                equivalence_check_fields(&classification, &EquivalenceCheckCounts::default())
+                "name=q5_equivalence sampling_point={sampling_point} {} s13={}",
+                equivalence_check_fields(&classification, &EquivalenceCheckCounts::default()),
+                equivalence_stops_at_s13(&classification, &EquivalenceCheckCounts::default())
             ));
             lines.result(&format!(
                 "name=q5_partition sampling_point={sampling_point} {}",
                 partition_fields(&classification, &EquivalenceCheckCounts::default())
             ));
             lines.result(&format!(
-                "name=verdict s1_failed_checks={failed_checks} s13=state_limit q5_not_run=too_many_states_after_refinement"
+                "name=verdict s1_failed_checks={failed_checks} s13=state_limit q5_not_run=too_many_states_after_refinement piece_panic_states=-"
             ));
             lines.finish();
             return 3;
@@ -7467,19 +8172,22 @@ fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
         );
         let s9 = s9_checks_of_a_stream(&export);
         export.checks.extend(s9);
-        let totals = report_stream(&mut lines, &export);
+        let totals = report_stream(lines, &export);
+        lines.result(&q5_piece_panics_line("main", &export.observations));
         if let Some(why) = &export.not_exported {
-            // 这一形截出来的写流里没有可判的根槽写（`export_prepared_stream` 记 `no_root_write_to_judge`）：类核对与对拍都没走到，
-            // 不是 S13；记 Q5 没跑、不进判定（登记「主 agent 认定：25-3a 装置返工交回」第 6 条）。
+            // 第二道（第一道是修订 5 第 5.3 节的 [`shape_without_a_root_write`]）：这一形截出来的写流里没有可判的根槽写
+            // （`export_prepared_stream` 记 `no_root_write_to_judge`）：类核对没走到，不是 S13；记 Q5 没跑、不进判定
+            // （登记「主 agent 认定：25-3a 装置返工交回」第 6 条）。
             lines.result(&format!(
                 "name=q5 sampling_point={sampling_point} not_run=no_root_write not_exported_why={}",
                 field_text(why)
             ));
             lines.result(&format!(
-                "name=verdict s1_failed_checks={} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states=0 s13=false q5_not_run=no_root_write",
+                "name=verdict s1_failed_checks={} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states=0 s13=false q5_not_run=no_root_write piece_panic_states={}",
                 failed_checks + totals.failed_checks,
                 totals.rebuilt_digest_mismatches,
-                totals.s3_mismatches
+                totals.s3_mismatches,
+                piece_panic_states_text(&totals.piece_panic_states)
             ));
             lines.finish();
             return 3;
@@ -7492,8 +8200,9 @@ fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
                 CONFIRMED_LOSS_IN_FLIGHT_CALL,
             );
             lines.result(&format!(
-                "name=q5_equivalence sampling_point={sampling_point} {}",
-                equivalence_check_fields(&classification, &evaluation.checks)
+                "name=q5_equivalence sampling_point={sampling_point} {} s13={}",
+                equivalence_check_fields(&classification, &evaluation.checks),
+                equivalence_stops_at_s13(&classification, &evaluation.checks)
             ));
             lines.result(&format!(
                 "name=q5_partition sampling_point={sampling_point} {}",
@@ -7521,7 +8230,7 @@ fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
                 .map(ConfirmedLossTallyRow::of_a_class)
                 .collect();
             report_the_confirmed_loss(
-                &mut lines,
+                lines,
                 sampling_point,
                 &rows,
                 confirmed_root,
@@ -7534,10 +8243,11 @@ fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
     };
     failed_checks += totals.failed_checks;
     lines.result(&format!(
-        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states={} s13={stops_at_s13}",
+        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states={} s13={stops_at_s13} piece_panic_states={}",
         totals.rebuilt_digest_mismatches,
         totals.s3_mismatches,
-        export.observations.len()
+        export.observations.len(),
+        piece_panic_states_text(&totals.piece_panic_states)
     ));
     lines.finish();
     if failed_checks > 0
@@ -7606,7 +8316,8 @@ impl<'row> ConfirmedLossTallyRow<'row> {
     }
 }
 
-/// Q5 的计数行（第六节 Q5-有效、Q5-判、Q5-阴性；第七节 A10；第八节轨迹：按末段里落了几个写排开）。每一行按它罩的态数计
+/// Q5 的计数行（第六节 Q5-有效、Q5-判、Q5-阴性；第七节 A10；第八节轨迹：按末段里落了几个写排开；修订 5 第 5.1 节第 7 条另加两列各自
+/// `judge_panic` 的态数）。每一行按它罩的态数计
 /// （全枚举的一态计 1，等价类的一类计这一类的态数）；`derived_by` 接在 `name=q5` 那一行末尾（按等价类判时写 `derived_by=…`）。
 fn report_the_confirmed_loss(
     lines: &mut ResultLines,
@@ -7659,13 +8370,15 @@ fn report_the_confirmed_loss(
         .map(|row| row.states)
         .sum();
     lines.result(&format!(
-        "name=q5 sampling_point={sampling_point} states={states} last_segment_writes={last_segment_writes} last_segment_tearable_writes={tearable} product_of_the_choices={product} a10_states_equal_the_product={} lost_the_confirmed_version={} reference_filesystem_red={} model_crash_judge_red={} both_green={} reference_filesystem_abstained={} not_lost_states={} not_lost_o1_red={} states_without_a_root={without_a_root}{derived_by}",
+        "name=q5 sampling_point={sampling_point} states={states} last_segment_writes={last_segment_writes} last_segment_tearable_writes={tearable} product_of_the_choices={product} a10_states_equal_the_product={} lost_the_confirmed_version={} reference_filesystem_red={} model_crash_judge_red={} both_green={} reference_filesystem_abstained={} reference_filesystem_judge_panic={} model_crash_judge_judge_panic={} not_lost_states={} not_lost_o1_red={} states_without_a_root={without_a_root}{derived_by}",
         states == product,
         states_of(&lost, &every_row),
         states_of(&lost, &reference_filesystem_red),
         states_of(&lost, &model_red),
         states_of(&lost, &both_green),
         states_of(&lost, &|row| row.reference_filesystem.word() == "abstained"),
+        states_of(&lost, &|row| row.reference_filesystem.word() == "judge_panic"),
+        states_of(&lost, &|row| row.crash_judge.word() == "judge_panic"),
         states_of(&not_lost, &every_row),
         states_of(&not_lost, &|row| reference_filesystem_red(row) || model_red(row)),
     ));
@@ -7695,8 +8408,9 @@ fn report_the_confirmed_loss(
 // 两态等价 ⇔ 键里那几个写（第 2 次调用的写 K，加上类核对 ③ 细分移进来的第 3 次调用的写）取法相同，且留在类里的第 3 次调用的写（C）
 // 「至少落了一个」这一位相同。代表是「C 一个都没落」那一态；「C 至少落一个」那一类按代表推，另把「C 全落」那一态照原路径判一遍对照（类核对 ④）。
 
-/// 「等价类对拍」那一形截到第 3 次调用发出的前几个写之后（修订 2 第 2.1 节候选二「用例钉住等价关系」：候选三 k = 2，72 × 4 = 288 态）。
-const EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT: usize = 2;
+/// 「等价类对拍」那一形截到第 3 次调用发出的前几个写之后：k = 6 = C 的前三对镜像写，72 × 2⁶ = 4 608 态（修订 5 第 5.2 节丙；
+/// 修订 2 第 2.1 节候选二原来是 k = 2、72 × 4 = 288 态，只罩一个单元的两份镜像）。
+const EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT: usize = 6;
 
 /// [`ObservationContext`] 借的几张表，由一条流现算（与 [`export_prepared_stream`] 里同一套算法；按等价类判与「等价类对拍」用）。
 struct ObservationTables {
@@ -7745,6 +8459,7 @@ impl ObservationTables {
             torn_image_index_by_write: &self.torn_image_index_by_write,
             reference_filesystem: &self.reference_filesystem,
             call_of_write: call_index_of_each_write(stream),
+            piece_panic_injection: None,
         }
     }
 }
@@ -7948,12 +8663,20 @@ impl ConfirmedLossPartition {
     }
 }
 
-/// 恢复读过的一段字节（盘、起点、终点，都按字节；终点不含）。
+/// 恢复读过的一段字节（盘、起点、终点，都按字节；终点不含），与这一段是从哪来的。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct RecordedReadRange {
     device: DeviceIdentity,
     start_in_bytes: u64,
     end_in_bytes: u64,
+    source: RecordedReadSource,
+}
+
+/// 一段记读从哪来：一次 `read`，或 journal 位置提示记的整个环（修订 5 第 5.4 节 U3：类核对 ③ 照旧用全部段，用例分得出提示那一段）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum RecordedReadSource {
+    Read,
+    JournalRecordOffsetsHint,
 }
 
 /// 类核对 ③ 的读者：包在崩溃镜像外面、只转发、只记（做法照 `e161_crash_state_dedup_and_time_split.rs` 的 `RecordingReader`，
@@ -7983,6 +8706,7 @@ impl PoolReader for ReadRecordingPoolReader<'_, '_> {
             device,
             start_in_bytes: offset.0,
             end_in_bytes: offset.0 + u64::try_from(length).expect("读长装得进 u64"),
+            source: RecordedReadSource::Read,
         });
         PoolReader::read(self.image, device, offset, length)
     }
@@ -7997,6 +8721,7 @@ impl PoolReader for ReadRecordingPoolReader<'_, '_> {
             device,
             start_in_bytes: ring_start.0,
             end_in_bytes: ring_start.0 + ring_bytes,
+            source: RecordedReadSource::JournalRecordOffsetsHint,
         });
         PoolReader::journal_record_offsets_hint(self.image, device, ring_start, ring_bytes)
     }
@@ -8187,22 +8912,29 @@ fn crash_columns_of_the_four_inputs(
             ReferenceFileSystemVoteOfAState::RecoveryPanicked,
         ),
         ConsultedRecovery::Reported(report) => {
-            let read_back = observed_read_back_after_a_crash(report);
+            // 判官自己 panic 那一列记 `judge_panic`（修订 5 第 5.1 节第 3 条；与逐件重做那一路同一读法），读回连同判官一起包。
             let crash_judge =
                 if crash_judge_is_applicable(touched_call, context.stream.model_stopped_at_call) {
-                    crash_judge_verdict(
-                        &context.stream.committed_versions,
-                        newest_persisted_root_key,
-                        &read_back,
-                    )
+                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+                        crash_judge_verdict(
+                            &context.stream.committed_versions,
+                            newest_persisted_root_key,
+                            &observed_read_back_after_a_crash(report),
+                        )
+                    }))
+                    .unwrap_or(CrashJudgeVerdict::JudgePanicked)
                 } else {
                     CrashJudgeVerdict::NotApplicable
                 };
-            let reference_filesystem = reference_filesystem_vote_of_a_state(
-                context.reference_filesystem,
-                context.reference_filesystem_moment_of(segment_index),
-                &read_back,
-            );
+            let reference_filesystem =
+                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+                    reference_filesystem_vote_of_a_state(
+                        context.reference_filesystem,
+                        context.reference_filesystem_moment_of(segment_index),
+                        &observed_read_back_after_a_crash(report),
+                    )
+                }))
+                .unwrap_or(ReferenceFileSystemVoteOfAState::JudgePanicked);
             (crash_judge, reference_filesystem)
         }
     }
@@ -8477,29 +9209,27 @@ fn equivalence_stops_at_s13(
         || checks.class_landing_mismatches > 0
 }
 
-/// `name=q5_equivalence` 那一行（与对拍那一行）的类核对字段：①–④ 各自的计数、细分次数与移进键的写、停没停在上限与 S13 的判定。
-/// 停机 S13 看的是前六个计数与 `stopped_at_the_state_limit`（细分次数不是：③ 不相交时细分是照登记做，细分到上限才停）。
+/// `name=q5_equivalence` 那一行（与对拍那一行）的类核对计数：①–④ 各自的计数与 ③ 停下时还相交的写数（修订 5 第 5.5 节第 2 条：
+/// 这两种行只放类核对的计数与判定）。`s13=` 由调用方接在后面：主取样点那一行是 [`equivalence_stops_at_s13`]，对拍那一行是
+/// [`EquivalencePairing::stops_at_s13`]（再并上逐态不同与各类态数之和不等于组合数，第 5.5 节第 3 条）。
 fn equivalence_check_fields(
     classification: &ConfirmedLossClassification,
     checks: &EquivalenceCheckCounts,
 ) -> String {
     format!(
-        "input_list_mismatches={} in_flight_writes_of_a_forbidden_kind={} read_overlaps_left={} recording_report_mismatches={} derived_class_mismatches={} class_landing_mismatches={} refinements={} moved_last_segment_positions={} stopped_at_the_state_limit={} s13={}",
+        "input_list_mismatches={} in_flight_writes_of_a_forbidden_kind={} read_overlaps_left={} recording_report_mismatches={} derived_class_mismatches={} class_landing_mismatches={}",
         checks.input_list_mismatches,
         checks.in_flight_writes_of_a_forbidden_kind,
         classification.overlapping_positions_left,
         checks.recording_report_mismatches,
         checks.derived_class_mismatches,
         checks.class_landing_mismatches,
-        classification.refinements,
-        field_text(&format!("{:?}", classification.moved_positions)),
-        classification.stopped_at_the_state_limit,
-        equivalence_stops_at_s13(classification, checks)
     )
 }
 
-/// `name=q5_partition` 那一行（与对拍那一行）的分法字段：C 的写数与经 `geometry.classify` 逐个列的种类（类核对 ②）、键里与类里的写
-/// （末段里的位置，A10 比对器按它现算各类态数）、要照原路径判的态数与上限。
+/// `name=q5_partition` 那一行（主取样点与对拍各一行）的分法字段：C 的写数与经 `geometry.classify` 逐个列的种类（类核对 ②）、键里与类里的写
+/// （末段里的位置，A10 比对器按它现算各类态数）、要照原路径判的态数与上限，加修订 5 第 5.5 节第 2 条从类核对那一行挪来的细分次数、
+/// 移进键的写与停没停在上限（`stopped_at_the_state_limit` 经 `s13=` 进 S13，这一行的字段不直接当触发观测）。
 fn partition_fields(
     classification: &ConfirmedLossClassification,
     checks: &EquivalenceCheckCounts,
@@ -8510,12 +9240,15 @@ fn partition_fields(
         .map(|(kind, count)| format!("{kind}:{count}"))
         .collect();
     format!(
-        "in_flight_writes={} in_flight_write_kinds={} keyed_last_segment_positions={} class_last_segment_positions={} judged_states_planned={} state_limit={CONFIRMED_LOSS_STATE_LIMIT}",
+        "in_flight_writes={} in_flight_write_kinds={} keyed_last_segment_positions={} class_last_segment_positions={} judged_states_planned={} state_limit={CONFIRMED_LOSS_STATE_LIMIT} refinements={} moved_last_segment_positions={} stopped_at_the_state_limit={}",
         classification.in_flight_writes,
         if kinds.is_empty() { "-".to_string() } else { kinds.join(",") },
         field_text(&format!("{:?}", classification.partition.keyed_positions)),
         field_text(&format!("{:?}", classification.partition.class_positions)),
-        classification.judged_states_planned
+        classification.judged_states_planned,
+        classification.refinements,
+        field_text(&format!("{:?}", classification.moved_positions)),
+        classification.stopped_at_the_state_limit
     )
 }
 
@@ -8544,8 +9277,18 @@ impl EquivalencePairing {
 
 /// 在一条 Q5 那一形的流上做「等价类对拍」。
 fn equivalence_pairing_of(stream: &PreparedStream) -> EquivalencePairing {
+    equivalence_pairing_with_injected_piece_panics(stream, None)
+}
+
+/// 同 [`equivalence_pairing_of`]，`piece_panic_injection` 给了的，判态时照它注入（阳性对照「判态路 panic 单列」，修订 5 第 5.1 节）。
+fn equivalence_pairing_with_injected_piece_panics(
+    stream: &PreparedStream,
+    piece_panic_injection: Option<&PiecePanicInjection>,
+) -> EquivalencePairing {
     let observation_tables = ObservationTables::of(stream);
-    let context = observation_tables.context(stream);
+    let mut context = observation_tables.context(stream);
+    context.piece_panic_injection = piece_panic_injection;
+    let context = context;
     let digits = LastSegmentDigits::of(stream, &observation_tables.tables);
     let product_of_the_choices =
         digits.product_of_the_radixes(&(0..digits.write_indexes.len()).collect::<Vec<usize>>());
@@ -8607,9 +9350,215 @@ fn equivalence_pairing_of(stream: &PreparedStream) -> EquivalencePairing {
     }
 }
 
+/// 「等价类对拍」那一形的三样（修订 5 第 5.2 节 S1 新加三条、U9）：末段里属于第 3 次调用的写（c1…c_k，按发出次序，写表下标）、
+/// 末段各写取法数之积、c1…c_k 按发出次序两两成对（同偏移、同长度、分属两块盘）且各对偏移互不相同的对数。
+struct EquivalencePairingShapeFacts {
+    in_flight_writes: Vec<usize>,
+    states: u64,
+    mirrored_pairs: usize,
+}
+
+fn equivalence_pairing_shape_facts(
+    stream: &PreparedStream,
+    tables: &Layer0EnumerationTables,
+) -> EquivalencePairingShapeFacts {
+    let call_of_write = call_index_of_each_write(stream);
+    let in_flight_writes: Vec<usize> = stream
+        .segments
+        .last()
+        .cloned()
+        .unwrap_or_default()
+        .into_iter()
+        .filter(|write_index| call_of_write[*write_index] == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL))
+        .collect();
+    let (_, _, states) = last_segment_product(stream, tables);
+    let mut pair_offsets: BTreeSet<u64> = BTreeSet::new();
+    let mut mirrored_pairs = 0usize;
+    for pair in in_flight_writes.chunks(2) {
+        if let [first, second] = pair {
+            let first_write = &stream.writes[*first];
+            let second_write = &stream.writes[*second];
+            if first_write.offset == second_write.offset
+                && first_write.length_in_bytes() == second_write.length_in_bytes()
+                && first_write.device != second_write.device
+                && pair_offsets.insert(first_write.offset.0)
+            {
+                mirrored_pairs += 1;
+            }
+        }
+    }
+    EquivalencePairingShapeFacts {
+        in_flight_writes,
+        states,
+        mirrored_pairs,
+    }
+}
+
+/// 阳性对照「判态路 panic 单列」的注入（修订 5 第 5.1 节）：c1…c6 是对拍流末段里第 3 次调用的前 6 个写（三对镜像）；
+/// `pool_checker`：c3 落、c4 没落；`ignored_recovery`：c3 没落、c4 落；`record_checker`：c5 落、c6 没落，且 (c3,c4) 同落或同不落。
+/// 三条选中的态互不相交。C 的写不够 6 个交 None（阳性对照不成立，走 F9）。
+fn piece_panic_control_injection(in_flight_writes: &[usize]) -> Option<PiecePanicInjection> {
+    let [_, _, third, fourth, fifth, sixth] = in_flight_writes.get(..6)? else {
+        return None;
+    };
+    Some(PiecePanicInjection {
+        rules: vec![
+            PiecePanicInjectionRule {
+                piece: JudgingPiece::PoolChecker,
+                conditions: vec![
+                    InjectedLandingCondition::Landed(*third),
+                    InjectedLandingCondition::NotLanded(*fourth),
+                ],
+            },
+            PiecePanicInjectionRule {
+                piece: JudgingPiece::IgnoredRecovery,
+                conditions: vec![
+                    InjectedLandingCondition::NotLanded(*third),
+                    InjectedLandingCondition::Landed(*fourth),
+                ],
+            },
+            PiecePanicInjectionRule {
+                piece: JudgingPiece::RecordChecker,
+                conditions: vec![
+                    InjectedLandingCondition::Landed(*fifth),
+                    InjectedLandingCondition::NotLanded(*sixth),
+                    InjectedLandingCondition::BothOrNeitherLanded(*third, *fourth),
+                ],
+            },
+        ],
+    })
+}
+
+/// 一态上 O1 两个崩溃子列与看 journal 那遍恢复的结局（阳性对照「判态路 panic 单列」逐态比的三样）。
+fn crash_columns_and_consulted_recovery(
+    observation: &StateObservation,
+) -> (
+    &CrashJudgeVerdict,
+    &ReferenceFileSystemVoteOfAState,
+    Option<ConsultedRecovery>,
+) {
+    (
+        &observation.crash_judge,
+        &observation.reference_filesystem,
+        ConsultedRecovery::of_the_observation(observation),
+    )
+}
+
+/// 注入那一遍对不注入那一遍，两个崩溃子列或看 journal 那遍恢复的结局不同的态数（按层 0 序号配对；不注入那一遍里没有的序号也算不同）。
+fn o1_columns_differing(plain: &[StateObservation], injected: &[StateObservation]) -> usize {
+    let plain_of_ordinal: BTreeMap<u64, &StateObservation> = plain
+        .iter()
+        .map(|observation| (observation.ordinal, observation))
+        .collect();
+    injected
+        .iter()
+        .filter(|observation| {
+            plain_of_ordinal
+                .get(&observation.ordinal)
+                .is_none_or(|plain_observation| {
+                    crash_columns_and_consulted_recovery(plain_observation)
+                        != crash_columns_and_consulted_recovery(observation)
+                })
+        })
+        .count()
+}
+
+/// 阳性对照「判态路 panic 单列」那一遍（`name=q5_piece_panic_control` 一行）。
+struct PiecePanicControl {
+    states: u64,
+    piece_panic_states: [usize; JUDGING_PIECE_COUNT],
+    consulted_recovery_panics_of_the_plain_pairing: usize,
+    o1_columns_differing_from_the_plain_pairing: usize,
+    mismatching_states: usize,
+    stops_at_s13: bool,
+    observations: Vec<StateObservation>,
+}
+
+impl PiecePanicControl {
+    /// 注入的三件与看 journal 那遍恢复之外的件 panic 的态数之和。
+    fn other_piece_panic_states(&self) -> usize {
+        JudgingPiece::ALL
+            .iter()
+            .zip(self.piece_panic_states)
+            .filter(|(piece, _)| {
+                !matches!(
+                    piece,
+                    JudgingPiece::ConsultedRecovery
+                        | JudgingPiece::PoolChecker
+                        | JudgingPiece::IgnoredRecovery
+                        | JudgingPiece::RecordChecker
+                )
+            })
+            .map(|(_, states)| states)
+            .sum()
+    }
+
+    /// 恰为修订 5 第 5.1 节的精确数（不符走 F9）。
+    fn holds(&self) -> bool {
+        let states_of = |piece: JudgingPiece| {
+            u64::try_from(self.piece_panic_states[piece.position()]).expect("态数装得进 u64")
+        };
+        self.states == LOCAL_EQUIVALENCE_PAIRING_STATES
+            && states_of(JudgingPiece::PoolChecker) == LOCAL_PIECE_PANIC_CONTROL_POOL_CHECKER_STATES
+            && states_of(JudgingPiece::IgnoredRecovery)
+                == LOCAL_PIECE_PANIC_CONTROL_IGNORED_RECOVERY_STATES
+            && states_of(JudgingPiece::RecordChecker)
+                == LOCAL_PIECE_PANIC_CONTROL_RECORD_CHECKER_STATES
+            && self.other_piece_panic_states() == 0
+            && self.piece_panic_states[JudgingPiece::ConsultedRecovery.position()]
+                == self.consulted_recovery_panics_of_the_plain_pairing
+            && self.o1_columns_differing_from_the_plain_pairing == 0
+            && self.mismatching_states == 0
+            && !self.stops_at_s13
+    }
+}
+
+/// 在对拍那一条流上照 [`piece_panic_control_injection`] 注入、再做一遍对拍，与不注入那一遍（`plain`）逐态比。
+fn piece_panic_control_of(
+    stream: &PreparedStream,
+    plain: &EquivalencePairing,
+) -> PiecePanicControl {
+    let tables = layer0_enumeration_tables(
+        &stream.base,
+        &stream.writes,
+        &stream.segments,
+        &full_expansion,
+    );
+    let facts = equivalence_pairing_shape_facts(stream, &tables);
+    let consulted_recovery_panics_of_the_plain_pairing =
+        piece_panic_states_of(&plain.observations)[JudgingPiece::ConsultedRecovery.position()];
+    let Some(injection) = piece_panic_control_injection(&facts.in_flight_writes) else {
+        return PiecePanicControl {
+            states: 0,
+            piece_panic_states: [0; JUDGING_PIECE_COUNT],
+            consulted_recovery_panics_of_the_plain_pairing,
+            o1_columns_differing_from_the_plain_pairing: 0,
+            mismatching_states: 0,
+            stops_at_s13: false,
+            observations: Vec::new(),
+        };
+    };
+    let injected = equivalence_pairing_with_injected_piece_panics(stream, Some(&injection));
+    PiecePanicControl {
+        states: injected.states,
+        piece_panic_states: piece_panic_states_of(&injected.observations),
+        consulted_recovery_panics_of_the_plain_pairing,
+        o1_columns_differing_from_the_plain_pairing: o1_columns_differing(
+            &plain.observations,
+            &injected.observations,
+        ),
+        mismatching_states: injected.mismatching_states,
+        stops_at_s13: injected.stops_at_s13(),
+        observations: injected.observations,
+    }
+}
+
 /// `stream-constants` 模式：只驱动、只回比（S1 的流相关几条与状态数几条、与流无关的几条、Q5 造法那几样），不写导出、不判态。
 fn run_stream_constants(streams: &[StreamName]) -> i32 {
-    let mut lines = ResultLines { emitted: 0 };
+    let mut lines = ResultLines {
+        emitted: 0,
+        collected: None,
+    };
     lines.result("name=config segment=s1 mode=stream-constants");
     let mut failed_checks = 0usize;
     let mut checks = stream_independent_checks();
@@ -10054,17 +11003,24 @@ mod tests {
         assert_eq!(reverse.len(), 5);
     }
 
-    /// 「等价类对拍」那一形的流（流二同一种造法，截到第 3 次调用发出的前 2 个写之后，72 × 4 = 288 态）。
-    fn equivalence_pairing_stream() -> PreparedStream {
+    /// 内联用例的对拍流截到第 3 次调用发出的前几个写（修订 5 第 5.2 节：U1–U6 与已有的两条留在 k = 2，288 态；装置那一跑是
+    /// [`EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT`]）。
+    const IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS: usize = 2;
+
+    /// 「等价类对拍」那一形的流（流二同一种造法），截到第 3 次调用发出的前 `in_flight_writes_kept` 个写之后。
+    fn equivalence_pairing_stream_keeping(in_flight_writes_kept: usize) -> PreparedStream {
         confirmed_loss_prepared_stream(
             second_stream_through_the_first_overwrite(RunSwitches::default()),
-            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
-                EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT,
-            ),
+            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(in_flight_writes_kept),
         )
         .0
     }
 
+    /// 「等价类对拍」那一形的流，截到第 3 次调用发出的前 2 个写之后（72 × 4 = 288 态）。
+    fn equivalence_pairing_stream() -> PreparedStream {
+        equivalence_pairing_stream_keeping(IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS)
+    }
+
     /// 等价关系（登记修订 2 第 2.1 节候选二「用例钉住等价关系」）：288 态那一形上全枚举逐态判与按等价类推，每态的恢复报告、两个崩溃子列
     /// 与 RefFS 一票逐态相同，类核对 ①–④ 都过；模型照常那一遍与 `model_stopped_at_call` 设成第 3 次调用那一遍各做一遍（V17：键去掉
     /// 「C 至少落一个」那一位，第二遍里「C 落了」的态模型崩溃判官不适用、代表上适用，这里红）。各类态数之和等于末段每个写的取法数之积，
@@ -10200,7 +11156,7 @@ mod tests {
     }
 
     /// 流三照读法 2 驱动：删第一个文件（意图一次、走空一步）、一次建一个 inode 到叶容器第一次变成两片，再多建一次
-    /// （V9：停在 K − 1 次，叶容器还是一片，这里红）。取号、暖机之外的每次调用都对拍过、对得上，变成两片的那一次与之后那一次也是
+    /// （V9：停在叶容器变成两片的那一次（第 K 次），少建之后那一次，这里红）。取号、暖机之外的每次调用都对拍过、对得上，变成两片的那一次与之后那一次也是
     /// （理想模型那件修理之后胶水按片记叶容器的角色，登记修订 2 第 2.2 节）：没有一次「模型接不了这份观测」，模型一次都不停。
     #[test]
     fn the_third_stream_stops_one_creation_after_the_inode_leaf_splits() {
@@ -10341,4 +11297,522 @@ mod tests {
         assert_eq!(stream_names_of_the_numbers("4"), None);
         assert_eq!(stream_names_of_the_numbers(""), None);
     }
+
+    /// 修订 5 第 5.4 节 U1、U4、U5、U6 的共同起点「S0」：k = 2 的对拍流上逐态全判 288 态、分好类（[`equivalence_pairing_of`]）。
+    struct UntamperedPairing {
+        stream: PreparedStream,
+        tables: ObservationTables,
+        pairing: EquivalencePairing,
+    }
+
+    /// 类核对 ①、②、④ 两半、落地串核对五个计数（按这个次序）。
+    fn five_class_check_counts(checks: &EquivalenceCheckCounts) -> [usize; 5] {
+        [
+            checks.input_list_mismatches,
+            checks.in_flight_writes_of_a_forbidden_kind,
+            checks.recording_report_mismatches,
+            checks.derived_class_mismatches,
+            checks.class_landing_mismatches,
+        ]
+    }
+
+    /// 照 [`evaluate_the_classes`] 在 S0 的（可能篡改过的）观测与分法上重做类核对。
+    fn class_checks_of(fixture: &UntamperedPairing) -> EquivalenceCheckCounts {
+        let context = fixture.tables.context(&fixture.stream);
+        let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
+        evaluate_the_classes(
+            &context,
+            &digits,
+            &fixture.pairing.classification,
+            &fixture.pairing.observations,
+            CONFIRMED_LOSS_IN_FLIGHT_CALL,
+        )
+        .checks
+    }
+
+    /// 造 S0，并断言不篡改时类核对五个计数全 0、不停在 S13（证明之后那一处篡改才是让它红的东西）。
+    fn untampered_pairing() -> UntamperedPairing {
+        let stream = equivalence_pairing_stream();
+        let tables = ObservationTables::of(&stream);
+        let pairing = equivalence_pairing_of(&stream);
+        let fixture = UntamperedPairing {
+            stream,
+            tables,
+            pairing,
+        };
+        let checks = class_checks_of(&fixture);
+        assert_eq!(
+            five_class_check_counts(&checks),
+            [0; 5],
+            "不篡改时类核对五个计数全 0"
+        );
+        assert!(
+            !equivalence_stops_at_s13(&fixture.pairing.classification, &checks),
+            "不篡改时不停在 S13"
+        );
+        fixture
+    }
+
+    /// S0 里层 0 序号是 `ordinal` 的那一态在观测里的下标。
+    fn observation_index_of(fixture: &UntamperedPairing, ordinal: u64) -> usize {
+        fixture
+            .pairing
+            .observations
+            .iter()
+            .position(|observation| observation.ordinal == ordinal)
+            .expect("对拍逐态全判：每个序号都有观测")
+    }
+
+    /// 末段里第一个属于第 3 次调用的写（c1）：写表下标与它在末段里的位置。
+    fn first_in_flight_write_of(stream: &PreparedStream) -> (usize, usize) {
+        let call_of_write = call_index_of_each_write(stream);
+        let last_segment = stream.segments.last().cloned().expect("末段");
+        let position = last_segment
+            .iter()
+            .position(|write_index| {
+                call_of_write[*write_index] == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL)
+            })
+            .expect("末段里有第 3 次调用的写");
+        (last_segment[position], position)
+    }
+
+    /// U1（修订 5 第 5.4 节，KM2；V20：类核对 ① 不计数，这里红）：S0，代表 0 的观测里 O1-模型崩溃判官换掉，① 数到这一态、停在 S13。
+    #[test]
+    fn the_input_list_check_counts_a_state_whose_columns_do_not_follow_the_four_inputs() {
+        let mut fixture = untampered_pairing();
+        let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
+        let representative = fixture
+            .pairing
+            .classification
+            .partition
+            .representative_ordinal(&digits, 0);
+        let index = observation_index_of(&fixture, representative);
+        let tampered = match fixture.pairing.observations[index].crash_judge {
+            CrashJudgeVerdict::Green => CrashJudgeVerdict::Red {
+                what: "tampered".to_string(),
+            },
+            CrashJudgeVerdict::NotApplicable
+            | CrashJudgeVerdict::Red { .. }
+            | CrashJudgeVerdict::Off
+            | CrashJudgeVerdict::RecoveryPanicked
+            | CrashJudgeVerdict::JudgePanicked => CrashJudgeVerdict::Green,
+        };
+        fixture.pairing.observations[index].crash_judge = tampered;
+        let checks = class_checks_of(&fixture);
+        assert_eq!(checks.input_list_mismatches, 1, "① 数到代表 0 那一态");
+        assert!(equivalence_stops_at_s13(
+            &fixture.pairing.classification,
+            &checks
+        ));
+    }
+
+    /// U2（修订 5 第 5.4 节，KM1；V21：类核对 ② 不计数，这里红）：c1 那条操作的盘与偏移改成第 2 次调用末段里第一个 journal 记录写的，
+    /// 写表不动（③ 看不见它）：② 数到它、停在 S13。
+    #[test]
+    fn an_in_flight_write_classified_as_a_journal_record_is_a_forbidden_kind() {
+        let mut stream = equivalence_pairing_stream();
+        let call_of_write = call_index_of_each_write(&stream);
+        let last_segment = stream.segments.last().cloned().expect("末段");
+        let journal_record_write = last_segment
+            .iter()
+            .copied()
+            .find(|write_index| {
+                call_of_write[*write_index] == Some(CONFIRMED_LOSS_CONFIRMED_CALL)
+                    && stream.writes[*write_index].kind == StepKind::JournalRecord
+            })
+            .expect("第 2 次调用在末段里写 journal 记录");
+        let (first_in_flight_write, _) = first_in_flight_write_of(&stream);
+        let journal_record_operation = stream.operations
+            [stream.stream_indexes[journal_record_write]]
+            .operation
+            .clone();
+        let in_flight_operation_index = stream.stream_indexes[first_in_flight_write];
+        stream.operations[in_flight_operation_index]
+            .operation
+            .device = journal_record_operation.device;
+        stream.operations[in_flight_operation_index]
+            .operation
+            .offset = journal_record_operation.offset;
+        assert_eq!(
+            common::geometry().classify(&stream.operations[in_flight_operation_index].operation),
+            StepKind::JournalRecord,
+            "改过的那条操作经 geometry.classify 是 journal 记录写"
+        );
+        let pairing = equivalence_pairing_of(&stream);
+        assert_eq!(pairing.checks.in_flight_writes_of_a_forbidden_kind, 1);
+        assert_eq!(
+            pairing.checks.in_flight_write_kinds,
+            BTreeMap::from([("journal_record", 1), ("unit_write", 1)])
+        );
+        assert_eq!(
+            pairing.classification.refinements, 0,
+            "写表没动，③ 看不见它，只有 ② 抓"
+        );
+        assert!(equivalence_stops_at_s13(
+            &pairing.classification,
+            &pairing.checks
+        ));
+    }
+
+    /// 与写对齐的粒度（U3 把 c1 挪到环尾时按它向下对齐）。
+    const WRITE_ALIGNMENT_IN_BYTES: u64 = 4096;
+
+    /// U3（修订 5 第 5.4 节，KM7；V22：journal 提示不记整环，这里红）：c1 挪到盘 0 journal 环尾、只有提示那一段罩得到的位置，
+    /// ③ 经提示那一段把它移进键（细分一次），细分之后逐态相同。
+    #[test]
+    fn an_in_flight_write_in_an_unread_journal_ring_slot_is_moved_into_the_key_through_the_hint() {
+        let mut stream = equivalence_pairing_stream();
+        let (first_in_flight_write, first_in_flight_position) = first_in_flight_write_of(&stream);
+        let length = stream.writes[first_in_flight_write].length_in_bytes();
+        let (device_zero, landing_offset) = {
+            let tables = ObservationTables::of(&stream);
+            let context = tables.context(&stream);
+            let digits = LastSegmentDigits::of(&stream, &tables.tables);
+            let classification = classify_the_last_segment(
+                &context,
+                &digits,
+                CONFIRMED_LOSS_IN_FLIGHT_CALL,
+                CONFIRMED_LOSS_STATE_LIMIT,
+            );
+            let reads_of_the_representative = |keyed_assignment: u64| {
+                let image = CrashImage {
+                    base: &stream.base,
+                    writes: &tables.tables.enumerated_writes,
+                    persisted: tables.facts.persisted_writes_of_state(
+                        classification
+                            .partition
+                            .representative_ordinal(&digits, keyed_assignment),
+                    ),
+                };
+                consulted_recovery_recording_the_reads(&image).1
+            };
+            let ring_hint = reads_of_the_representative(0)
+                .into_iter()
+                .filter(|read| read.source == RecordedReadSource::JournalRecordOffsetsHint)
+                .min_by_key(|read| read.device.0)
+                .expect("代表 0 的恢复经提示记了 journal 环");
+            let landing_offset = (ring_hint.end_in_bytes - length) / WRITE_ALIGNMENT_IN_BYTES
+                * WRITE_ALIGNMENT_IN_BYTES;
+            for keyed_assignment in 0..classification.partition.keyed_assignments(&digits) {
+                assert!(
+                    reads_of_the_representative(keyed_assignment)
+                        .iter()
+                        .filter(|read| read.source == RecordedReadSource::Read
+                            && read.device == ring_hint.device)
+                        .all(|read| read.end_in_bytes <= landing_offset
+                            || landing_offset + length <= read.start_in_bytes),
+                    "前提：第 {keyed_assignment} 种取法的代表上来自 read 的段不罩 [{landing_offset}, {})，只有提示那一段罩得到",
+                    landing_offset + length
+                );
+            }
+            (ring_hint.device, landing_offset)
+        };
+        stream.writes[first_in_flight_write].device = device_zero;
+        stream.writes[first_in_flight_write].offset = DeviceOffsetInBytes(landing_offset);
+        let pairing = equivalence_pairing_of(&stream);
+        assert_eq!(
+            pairing.classification.moved_positions,
+            vec![first_in_flight_position],
+            "环尾那个写经提示那一段移进键"
+        );
+        assert_eq!(pairing.classification.refinements, 1);
+        assert_eq!(pairing.mismatching_states, 0, "细分之后逐态相同");
+    }
+
+    /// U4（修订 5 第 5.4 节，KM4；V23：类核对 ④ 前一半不计数，这里红）：S0，代表 0 的记读那一遍恢复换成 panic，④ 前一半数到它。
+    #[test]
+    fn a_recording_recovery_that_differs_from_the_layer0_report_is_counted() {
+        let mut fixture = untampered_pairing();
+        let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
+        let representative = fixture
+            .pairing
+            .classification
+            .partition
+            .representative_ordinal(&digits, 0);
+        fixture
+            .pairing
+            .classification
+            .representative_recoveries
+            .insert(
+                representative,
+                ConsultedRecovery::Panicked("tampered".to_string()),
+            );
+        let checks = class_checks_of(&fixture);
+        assert_eq!(checks.recording_report_mismatches, 1, "④ 前一半数到代表 0");
+        assert!(equivalence_stops_at_s13(
+            &fixture.pairing.classification,
+            &checks
+        ));
+    }
+
+    /// U5（修订 5 第 5.4 节，KM3；V24：类核对 ④ 后一半不计数，这里红）：S0，全落 0 的观测里 RefFS 一票换掉，④ 后一半数到它（① 同时数到）。
+    #[test]
+    fn a_fully_landed_state_that_differs_from_its_derived_class_is_counted() {
+        let mut fixture = untampered_pairing();
+        let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
+        let fully_landed = fixture
+            .pairing
+            .classification
+            .partition
+            .fully_landed_ordinal(&digits, 0);
+        let index = observation_index_of(&fixture, fully_landed);
+        let tampered = match fixture.pairing.observations[index].reference_filesystem {
+            ReferenceFileSystemVoteOfAState::RecoveryPanicked => {
+                ReferenceFileSystemVoteOfAState::Green {
+                    allowed: String::new(),
+                }
+            }
+            ReferenceFileSystemVoteOfAState::Green { .. }
+            | ReferenceFileSystemVoteOfAState::Red { .. }
+            | ReferenceFileSystemVoteOfAState::Abstained { .. }
+            | ReferenceFileSystemVoteOfAState::JudgePanicked
+            | ReferenceFileSystemVoteOfAState::Off => {
+                ReferenceFileSystemVoteOfAState::RecoveryPanicked
+            }
+        };
+        fixture.pairing.observations[index].reference_filesystem = tampered;
+        let checks = class_checks_of(&fixture);
+        assert_eq!(checks.derived_class_mismatches, 1, "④ 后一半数到全落 0");
+        assert_eq!(checks.input_list_mismatches, 1, "① 同时数到这一态");
+    }
+
+    /// U6（修订 5 第 5.4 节，KM5、KM6；V25、V26：代表、全落那一态的落地串核对恒过，V27：S13 不看落地串核对，这里红）：S0，
+    /// (a) 代表 0 的落地串里 c1 那一位改成 `2`；(b) 还原后全落 0 的落地串里 c1 那一位改成 `0`。各自只有落地串核对数到 1、停在 S13。
+    #[test]
+    fn a_landing_string_off_the_partition_is_counted_and_stops_at_s13() {
+        let mut fixture = untampered_pairing();
+        let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
+        let (first_in_flight_write, _) = first_in_flight_write_of(&fixture.stream);
+        let representative = fixture
+            .pairing
+            .classification
+            .partition
+            .representative_ordinal(&digits, 0);
+        let fully_landed = fixture
+            .pairing
+            .classification
+            .partition
+            .fully_landed_ordinal(&digits, 0);
+        for (ordinal, tampered_landing) in [(representative, "2"), (fully_landed, "0")] {
+            let index = observation_index_of(&fixture, ordinal);
+            let original = fixture.pairing.observations[index].landing_vector.clone();
+            fixture.pairing.observations[index]
+                .landing_vector
+                .replace_range(
+                    first_in_flight_write..first_in_flight_write + 1,
+                    tampered_landing,
+                );
+            let checks = class_checks_of(&fixture);
+            assert_eq!(
+                five_class_check_counts(&checks),
+                [0, 0, 0, 0, 1],
+                "序号 {ordinal} 的 c1 那一位改成 {tampered_landing}：只有落地串核对数到它"
+            );
+            assert!(
+                equivalence_stops_at_s13(&fixture.pairing.classification, &checks),
+                "S13 只能从落地串核对来"
+            );
+            fixture.pairing.observations[index].landing_vector = original;
+        }
+    }
+
+    /// U7（修订 5 第 5.7 节，第 5.1 节甲；V28：一件 panic 两列都记恢复 panic，V29：`pool_checker` 那一件 panic 不记件名，这里红）：
+    /// k = 4 的对拍流（1 152 态，c1…c4 两对镜像），注入 `pool_checker`（c3 落、c4 没落）、`ignored_recovery`（c3 没落、c4 落）、
+    /// `record_checker`（c1 没落、c2 落、(c3,c4) 同落或同不落）：各件的态数恰为 288、288、144，两个崩溃子列与看 journal 那遍恢复逐态不变。
+    #[test]
+    fn a_panic_off_the_consulted_recovery_leaves_the_two_crash_columns_to_the_four_inputs() {
+        let stream = equivalence_pairing_stream_keeping(4);
+        let tables = ObservationTables::of(&stream);
+        let facts = equivalence_pairing_shape_facts(&stream, &tables.tables);
+        let [first, second, third, fourth] = facts.in_flight_writes[..] else {
+            panic!(
+                "k = 4 的对拍流末段里第 3 次调用恰有 4 个写：{:?}",
+                facts.in_flight_writes
+            );
+        };
+        let injection = PiecePanicInjection {
+            rules: vec![
+                PiecePanicInjectionRule {
+                    piece: JudgingPiece::PoolChecker,
+                    conditions: vec![
+                        InjectedLandingCondition::Landed(third),
+                        InjectedLandingCondition::NotLanded(fourth),
+                    ],
+                },
+                PiecePanicInjectionRule {
+                    piece: JudgingPiece::IgnoredRecovery,
+                    conditions: vec![
+                        InjectedLandingCondition::NotLanded(third),
+                        InjectedLandingCondition::Landed(fourth),
+                    ],
+                },
+                PiecePanicInjectionRule {
+                    piece: JudgingPiece::RecordChecker,
+                    conditions: vec![
+                        InjectedLandingCondition::NotLanded(first),
+                        InjectedLandingCondition::Landed(second),
+                        InjectedLandingCondition::BothOrNeitherLanded(third, fourth),
+                    ],
+                },
+            ],
+        };
+        let plain = equivalence_pairing_of(&stream);
+        assert_eq!(
+            piece_panic_states_of(&plain.observations),
+            [0; JUDGING_PIECE_COUNT],
+            "不注入那一遍各件 panic 都是 0（前提）"
+        );
+        let injected = equivalence_pairing_with_injected_piece_panics(&stream, Some(&injection));
+        let two_choices_each_of_two_writes =
+            usize::try_from(LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2).expect("288");
+        let two_choices_of_one_write =
+            usize::try_from(LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2).expect("144");
+        let mut expected = [0usize; JUDGING_PIECE_COUNT];
+        expected[JudgingPiece::PoolChecker.position()] = two_choices_each_of_two_writes;
+        expected[JudgingPiece::IgnoredRecovery.position()] = two_choices_each_of_two_writes;
+        expected[JudgingPiece::RecordChecker.position()] = two_choices_of_one_write;
+        assert_eq!(
+            piece_panic_states_of(&injected.observations),
+            expected,
+            "注入的三件各 288、288、144 态，别的件 0"
+        );
+        assert_eq!(
+            o1_columns_differing(&plain.observations, &injected.observations),
+            0,
+            "逐态两个崩溃子列与看 journal 那遍恢复的结局与不注入那一遍相同"
+        );
+        assert!(injected
+            .observations
+            .iter()
+            .all(|observation| observation.recovery_panic.is_none()));
+        assert_eq!(injected.mismatching_states, 0);
+        assert!(!injected.stops_at_s13());
+        let pool_checker_panicking: Vec<&StateObservation> = injected
+            .observations
+            .iter()
+            .filter(|observation| {
+                observation
+                    .piece_panics
+                    .contains(&JudgingPiece::PoolChecker)
+            })
+            .collect();
+        assert_eq!(pool_checker_panicking.len(), two_choices_each_of_two_writes);
+        assert!(pool_checker_panicking.iter().all(|observation| {
+            observation.pool_checker_red == Some(vec!["pool_checker_panic".to_string()])
+        }));
+    }
+
+    /// U8 用的两形：主取样点那一形与对拍那一形（照 `run_confirmed_loss_form` 造）。
+    fn confirmed_loss_shapes_for_the_test() -> ConfirmedLossShapes {
+        let (main, shape) = confirmed_loss_prepared_stream(
+            second_stream_through_the_first_overwrite(RunSwitches::default()),
+            ConfirmedLossTruncation::BeforeTheInFlightBarrier,
+        );
+        ConfirmedLossShapes {
+            sampling_point: "main_before_the_first_barrier_of_call_3",
+            main,
+            shape,
+            drive_nanoseconds: 0,
+            pairing: Some(equivalence_pairing_stream_keeping(
+                EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT,
+            )),
+        }
+    }
+
+    /// U8（修订 5 第 5.7 节，第 5.3 节；V30：根写的前置判删掉、退回先对拍，这里红）：主取样点那一形、对拍那一形各模拟一次没有根写
+    /// （只把 `judged_root_index` 设成 `usize::MAX`，写流不变），都在对拍之前退 3、记 `q5_not_run=no_root_write`。
+    #[test]
+    fn a_shape_without_a_root_write_stops_before_the_pairing() {
+        let command_line = ConfirmedLossCommandLine {
+            output_directory: std::env::temp_dir().join("e171r1-unit-no-root-write-not-exported"),
+            truncate_at_the_return_point: false,
+        };
+        enum ShapeWithoutARootWrite {
+            Main,
+            Pairing,
+        }
+        for shape_without_a_root in [
+            ShapeWithoutARootWrite::Main,
+            ShapeWithoutARootWrite::Pairing,
+        ] {
+            let mut shapes = confirmed_loss_shapes_for_the_test();
+            let shape_name = match shape_without_a_root {
+                ShapeWithoutARootWrite::Main => {
+                    shapes.main.judged_root_index = usize::MAX;
+                    "main"
+                }
+                ShapeWithoutARootWrite::Pairing => {
+                    shapes
+                        .pairing
+                        .as_mut()
+                        .expect("主取样点那一跑有对拍那一形")
+                        .judged_root_index = usize::MAX;
+                    "pairing"
+                }
+            };
+            let mut lines = ResultLines {
+                emitted: 0,
+                collected: Some(Vec::new()),
+            };
+            let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
+            let collected = lines.collected.expect("收集模式");
+            assert_eq!(exit_code, 3, "{shape_name}：退 3");
+            assert_eq!(
+                collected
+                    .iter()
+                    .filter(|line| line.starts_with("name=q5 ")
+                        && line.contains(" not_run=no_root_write")
+                        && line.contains(&format!(" shape={shape_name}")))
+                    .count(),
+                1,
+                "{shape_name}：一行 name=q5 … not_run=no_root_write shape={shape_name}：{collected:?}"
+            );
+            assert!(
+                collected
+                    .iter()
+                    .any(|line| line.starts_with("name=verdict ")
+                        && line.contains(" s13=false")
+                        && line.contains(" q5_not_run=no_root_write")),
+                "{shape_name}：判决行带 s13=false 与 q5_not_run=no_root_write：{collected:?}"
+            );
+            assert!(
+                !collected.iter().any(|line| {
+                    line.starts_with("name=q5_equivalence_pairing")
+                        || line.starts_with("name=q5_piece_panic_control")
+                        || line.starts_with("name=q5_equivalence")
+                }),
+                "{shape_name}：对拍、阳性对照与类核对一行都没打：{collected:?}"
+            );
+        }
+    }
+
+    /// U9（修订 5 第 5.7 节，第 5.2 节丙；V31：k 由 6 改回 2，这里红）：k = [`EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT`] 的对拍流，
+    /// 末段各写取法数之积 4 608、第 3 次调用的写 6 个、c1…c6 三对镜像（偏移互不相同）。只造末段的进制，不判态。
+    #[test]
+    fn the_pairing_keeps_the_first_three_mirrored_pairs_of_the_in_flight_call() {
+        let stream = equivalence_pairing_stream_keeping(EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT);
+        let tables = layer0_enumeration_tables(
+            &stream.base,
+            &stream.writes,
+            &stream.segments,
+            &full_expansion,
+        );
+        let facts = equivalence_pairing_shape_facts(&stream, &tables);
+        let digits = LastSegmentDigits::of(&stream, &tables);
+        assert_eq!(
+            digits.product_of_the_radixes(&(0..digits.write_indexes.len()).collect::<Vec<usize>>()),
+            LOCAL_EQUIVALENCE_PAIRING_STATES,
+            "末段各写取法数之积是 72 × 2⁶"
+        );
+        assert_eq!(facts.states, LOCAL_EQUIVALENCE_PAIRING_STATES);
+        assert_eq!(
+            facts.in_flight_writes.len(),
+            LOCAL_EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES,
+            "末段里属于第 3 次调用的写 6 个"
+        );
+        assert_eq!(
+            facts.mirrored_pairs, LOCAL_EQUIVALENCE_PAIRING_MIRRORED_PAIRS,
+            "c1…c6 两两同偏移、同长度、分属两块盘，三对的偏移互不相同"
+        );
+    }
 }
diff --git a/crates/singlefs-harness/tests/inode_tree_leaf_containers_appended_one_at_a_time_up_to_what_one_root_node_holds.rs b/crates/singlefs-harness/tests/inode_tree_leaf_containers_appended_one_at_a_time_up_to_what_one_root_node_holds.rs
new file mode 100644
index 0000000..352d0f8
--- /dev/null
+++ b/crates/singlefs-harness/tests/inode_tree_leaf_containers_appended_one_at_a_time_up_to_what_one_root_node_holds.rs
@@ -0,0 +1,91 @@
+//! inode 树叶容器的切法（实现一侧，`singlefs_core::inode_tree::write_records_into_leaf_containers`，纯函数）在 135 片的边界上：
+//! 一次追加一条，第 k 条（从 0 数）重写的恰是左起第 ⌊k ÷ 233⌋ 片，追加到第 233 × 135 条（k = 31 454）为止都收；
+//! 再追加一条要第 136 片，一个码 2 根装不下（D8（核心索引结构） 已定项 6：扇出 135；树长高那一步条款没写），实现在任何落盘动作之前拒。
+//! 理想模型的落点式子（`crates/singlefs-harness/src/model.rs` 的 `inode_leaf_container_position_of_a_new_record`）与它是同一条规则、
+//! 同一个 135（第 25 批代码轮正推 K1 第一行与第四行，判决 `research/prompts/m3-formal-code-b25-r1-main-verification.md`）。
+
+use singlefs_core::address::{CheckpointTxg, TreeIdentifier};
+use singlefs_core::inode_tree::{
+    leaf_containers_one_root_node_holds, write_records_into_leaf_containers,
+    InodeLeafContainerIndexInTree, InodeTreeWriteRefusal,
+};
+use singlefs_core::records::{InodeRecord, NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES};
+use singlefs_format::{INODE_LEAF_RECORDS, TREE_IDENTIFIER_INODE};
+
+/// 一个码 2 根装得下几片叶容器（D8（核心索引结构） 已定项 6：内部条目 120 字节、扇出 135）。
+const LEAF_CONTAINERS_ONE_ROOT_NODE_HOLDS: usize = 135;
+
+/// 第一条追加的记录用的 inode 号：inode 1 是第一个文件，删完之后建的从 2 起。
+const FIRST_APPENDED_INODE_NUMBER: u64 = 2;
+
+/// 第一次追加那次发布的 checkpoint_txg（之后每次加 1）。
+const FIRST_APPEND_CHECKPOINT_TXG: u64 = 4;
+
+fn size_zero_inode_record_appended_as(records_before: u64) -> InodeRecord {
+    InodeRecord {
+        inode: FIRST_APPENDED_INODE_NUMBER + records_before,
+        object_birth: CheckpointTxg(FIRST_APPEND_CHECKPOINT_TXG + records_before),
+        nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
+        size: 0,
+        change_count: 1,
+        write_time_seconds: 1,
+    }
+}
+
+/// 一次追加一条记录，追加到 135 片装满（233 × 135 条），每次只重写新记录落的第 ⌊k ÷ 233⌋ 片；
+/// 再追加一条，交回 `MoreLeafContainersThanOneRootNodeHolds { containers: 136, capacity: 135 }`。
+/// 判别力：容器数的上限放宽一片（第 31 456 条照收），或分裂点提前（一片不装满 233 条就分裂，第 k 条落的片不是 ⌊k ÷ 233⌋）。
+#[test]
+fn appending_inode_records_one_at_a_time_rewrites_container_floor_k_over_233_and_refuses_the_136th_container(
+) {
+    assert_eq!(
+        leaf_containers_one_root_node_holds(),
+        LEAF_CONTAINERS_ONE_ROOT_NODE_HOLDS,
+        "一个码 2 根的扇出是 135（D8 已定项 6）"
+    );
+    let inode_tree = TreeIdentifier(TREE_IDENTIFIER_INODE);
+    let records_one_root_node_holds = INODE_LEAF_RECORDS
+        * u64::try_from(LEAF_CONTAINERS_ONE_ROOT_NODE_HOLDS).expect("135 装得进 u64");
+    let mut containers = Vec::new();
+    for records_before in 0..records_one_root_node_holds {
+        let after = write_records_into_leaf_containers(
+            &containers,
+            &[size_zero_inode_record_appended_as(records_before)],
+            CheckpointTxg(FIRST_APPEND_CHECKPOINT_TXG + records_before),
+            inode_tree,
+        )
+        .unwrap_or_else(|refusal| panic!("第 {records_before} 条（从 0 数）还装得下：{refusal:?}"));
+        let expected_position = usize::try_from(records_before / INODE_LEAF_RECORDS)
+            .expect("不到 135 的片号装得进 usize");
+        assert_eq!(
+            after.rewritten,
+            vec![InodeLeafContainerIndexInTree::of_position(
+                expected_position
+            )],
+            "第 {records_before} 条（从 0 数）只重写左起第 ⌊k ÷ 233⌋ 片"
+        );
+        containers = after.containers;
+    }
+    assert_eq!(
+        containers.len(),
+        LEAF_CONTAINERS_ONE_ROOT_NODE_HOLDS,
+        "233 × 135 条装满 135 片"
+    );
+    let refusal = write_records_into_leaf_containers(
+        &containers,
+        &[size_zero_inode_record_appended_as(
+            records_one_root_node_holds,
+        )],
+        CheckpointTxg(FIRST_APPEND_CHECKPOINT_TXG + records_one_root_node_holds),
+        inode_tree,
+    )
+    .expect_err("第 233 × 135 + 1 条要第 136 片，一个码 2 根装不下");
+    assert_eq!(
+        refusal,
+        InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds {
+            containers: LEAF_CONTAINERS_ONE_ROOT_NODE_HOLDS + 1,
+            capacity: LEAF_CONTAINERS_ONE_ROOT_NODE_HOLDS,
+        },
+        "拒的是根装不下的那一格"
+    );
+}
diff --git a/crates/singlefs-harness/tests/model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers.rs b/crates/singlefs-harness/tests/model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers.rs
new file mode 100644
index 0000000..0421a73
--- /dev/null
+++ b/crates/singlefs-harness/tests/model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers.rs
@@ -0,0 +1,115 @@
+//! O1（理想模型对拍，D13（验证路线） 已定项 6）罩管理员回退跨过 inode 树叶容器分裂的那一支：
+//! 删完第一个文件之后建到 inode 树有三片叶容器（第三片 2 条），回退到根环里第 3 新那条根（第三片还没分出来的那一版），
+//! 再建 3 个 inode（第三片重新分出来）。回退那次发布的新根取目标那一版的叶容器（D23（journal 的角色与格式） 已定项 14），
+//! 模型要把现行那一版有、目标那一版没有的那几片丢掉、把目标那一版的那几片的分配代整组取回来，之后逐片记的账才与实现对得上。
+
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_format::INODE_LEAF_RECORDS;
+use singlefs_harness::history::{
+    execute_history_with, AppliedEffect, GeneratedHistory, HistoryDeviceWidth, HistoryEnding,
+    HistoryExecution, HistoryOperation, HistoryOperationKind, HistorySeed, HistoryStartingPoint,
+    PerStepChecker, RollbackTargetChoice, StepOutcome,
+};
+use singlefs_harness::SharedStream;
+
+/// 回退之前建几个 inode：建满两片（2 × 233 条），再往第三片里建 2 条——根环里最新那条根的第三片 2 条、第 2 新的 1 条、
+/// 第 3 新的那一版还没有第三片。
+const INODE_CREATIONS_BEFORE_THE_ROLLBACK: u64 = 2 * INODE_LEAF_RECORDS + 2;
+
+/// 回退到根环里按 (txg, 实例) 从新到旧第几条（从 0 数）：第 3 新那条，也就是第三片分出来之前建完第 466 个 inode 的那一版。
+const ROLLBACK_TARGET_INDEX_FROM_NEWEST: u64 = 2;
+
+/// 回退之后再建几个 inode：第一个在目标那一版的末尾重新分出第三片，后两个只重写第三片。
+const INODE_CREATIONS_AFTER_THE_ROLLBACK: u64 = 3;
+
+/// mkfs、取号、暖机、第一个文件（起点 `AfterFirstFile`），unlink 第一个文件，一次建一个 inode 建 468 个，
+/// 回退到根环里第 3 新那条根，再建 3 个。模型与实现每一步都对得上：历史跑完、没有新发现，回退做成，回退之后三次建 inode 都做成。
+/// 每一步的池级 checker 不跑（`PerStepChecker::Skipped`）：这条用例判的是 O1 逐片记的账。
+/// 判别力：模型回退时不丢掉现行那一版多出来的第三片（`role_written_at.retain` 那一句没了），或不取回目标那一版的右侧叶容器
+/// （`role_written_at.extend` 那一句没了），回退或回退之后的建 inode 那一步判对不上、历史停在那里
+/// （第 25 批代码轮攻方 T13，判决 `research/prompts/m3-formal-code-b25-r1-main-verification.md` K1 第二行）。
+#[test]
+fn a_rollback_past_an_inode_leaf_container_split_then_creating_inodes_is_judged_by_the_model_container_by_container(
+) {
+    let creations_before =
+        usize::try_from(INODE_CREATIONS_BEFORE_THE_ROLLBACK).expect("几百次装得进 usize");
+    let creations_after =
+        usize::try_from(INODE_CREATIONS_AFTER_THE_ROLLBACK).expect("几次装得进 usize");
+    let mut operations = vec![HistoryOperation::Unlink];
+    operations.extend(std::iter::repeat_n(
+        HistoryOperation::CreateInode,
+        creations_before,
+    ));
+    operations.push(HistoryOperation::RollBackWhileMounted(
+        RollbackTargetChoice::RingRoot {
+            index_from_newest: ROLLBACK_TARGET_INDEX_FROM_NEWEST,
+        },
+    ));
+    operations.extend(std::iter::repeat_n(
+        HistoryOperation::CreateInode,
+        creations_after,
+    ));
+    let rollback_step = 1 + creations_before;
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations,
+    };
+    let run = execute_history_with(
+        &history,
+        HistoryExecution {
+            per_step_checker: PerStepChecker::Skipped,
+            device_width: HistoryDeviceWidth::FourGibibytes,
+            space_admission: SpaceAdmission::JudgedByTheFormula,
+        },
+        &SharedStream::new(),
+        &mut |_| {},
+    );
+    assert_eq!(
+        run.ending,
+        HistoryEnding::Completed,
+        "跑到第 {} 步：{:?}",
+        run.outcomes.len(),
+        run.outcomes.last()
+    );
+    assert_eq!(
+        run.outcomes.len(),
+        history.operations.len(),
+        "每一步操作各有一个结局"
+    );
+    assert!(
+        matches!(
+            run.outcomes[rollback_step],
+            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
+        ),
+        "回退到根环里第 3 新那条根做成：{:?}",
+        run.outcomes[rollback_step]
+    );
+    let not_published: Vec<(usize, &StepOutcome)> = run
+        .outcomes
+        .iter()
+        .enumerate()
+        .filter(|(step, _)| *step != rollback_step)
+        .filter(|(_, outcome)| {
+            !matches!(
+                outcome,
+                StepOutcome::Applied(AppliedEffect::Published { .. })
+            )
+        })
+        .collect();
+    assert!(
+        not_published.is_empty(),
+        "unlink 与回退前后每一次建 inode 都做成：{not_published:?}"
+    );
+    let created = run
+        .tally
+        .operations_by_kind
+        .get(&HistoryOperationKind::CreateInode)
+        .copied()
+        .expect("建过 inode：这一类在计数里");
+    assert_eq!(
+        created.applied,
+        INODE_CREATIONS_BEFORE_THE_ROLLBACK + INODE_CREATIONS_AFTER_THE_ROLLBACK,
+        "回退前后的建 inode 都做成：{created:?}"
+    );
+}
diff --git a/crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs b/crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs
index 0cebffd..5b3ded9 100644
--- a/crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs
+++ b/crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs
@@ -1,6 +1,7 @@
 //! O1（理想模型对拍，D13（验证路线） 已定项 6）罩 inode 树第二片及以后的叶容器（C673 ⑦「历史加跨一片叶」）：
-//! 删完第一个文件之后一次建一个 inode，建满 233 条之后下一条在末尾分裂出第二片（D8（核心索引结构） 已定项 6：左半不动、
-//! 只重写新记录落的那一片与根），模型逐片记各片叶容器的分配代、逐次与实现交回的分配记录比，每一步都对得上。
+//! 删完第一个文件之后一次建一个 inode，建满 233 条之后下一条在末尾分裂出第二片、建满 2 × 233 条之后下一条分裂出第三片
+//! （D8（核心索引结构） 已定项 6：左半不动、只重写新记录落的那一片与根），模型逐片记各片叶容器的分配代、逐次与实现交回的分配记录比，
+//! 每一步都对得上。
 
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_format::INODE_LEAF_RECORDS;
@@ -11,21 +12,24 @@ use singlefs_harness::history::{
 };
 use singlefs_harness::SharedStream;
 
-/// 删完第一个文件之后建几个 inode：建满第一片的 233 条，再往第二片里建 4 条——第 234 条分裂出第二片，
-/// 之后三条只重写第二片、第一片的分配代停在第 233 条那次。
-const INODE_CREATIONS_INTO_THE_SECOND_LEAF_CONTAINER: u64 = INODE_LEAF_RECORDS + 4;
+/// 删完第一个文件之后建几个 inode：建满第一片、第二片各 233 条，再往第三片里建 4 条——第 234 条分裂出第二片、
+/// 第 467 条分裂出第三片，之后三条只重写第三片；第一片的分配代停在第 233 条那次、第二片的停在第 466 条那次。
+/// 建到第三片才分得开「第 k 片」与「第二片及以后都当第二片」（第 25 批代码轮正推 KX1、KX2，
+/// 判决 `research/prompts/m3-formal-code-b25-r1-main-verification.md` K1 第三行）。
+const INODE_CREATIONS_INTO_THE_THIRD_LEAF_CONTAINER: u64 = 2 * INODE_LEAF_RECORDS + 4;
 
 /// mkfs、取号、暖机、第一个文件（起点 `AfterFirstFile`），unlink 第一个文件（意图那次 + 最后一批，队列走空），
-/// 之后一次建一个 inode，建到 inode 树有两片叶容器再多建三个。模型与实现每一步都对得上：历史跑完、没有新发现，
+/// 之后一次建一个 inode，建到 inode 树有三片叶容器再多建三个。模型与实现每一步都对得上：历史跑完、没有新发现，
 /// unlink 与每一次建 inode 都做成。
 /// 每一步的池级 checker 不跑（`PerStepChecker::Skipped`）：这条用例判的是 O1 逐片记的账，跑 checker 时 debug 下要六分多钟。
 /// 判别力：模型把第二片的分配代记成第一片的（建第二片里的 inode 时重写的叶容器记成最左那片），或胶水把第二片当成最左那片，
-/// 建第 234 个 inode 那一步判对不上、历史停在那里。
+/// 建第 234 个 inode 那一步判对不上、历史停在那里；模型把落点封顶在第二片，或胶水把第三片及以后都当第二片，
+/// 建第 467 个 inode 那一步判对不上。
 #[test]
 fn creating_inodes_one_at_a_time_past_the_first_leaf_container_is_judged_by_the_model_container_by_container(
 ) {
-    let creations = usize::try_from(INODE_CREATIONS_INTO_THE_SECOND_LEAF_CONTAINER)
-        .expect("几百次装得进 usize");
+    let creations =
+        usize::try_from(INODE_CREATIONS_INTO_THE_THIRD_LEAF_CONTAINER).expect("几百次装得进 usize");
     let mut operations = vec![HistoryOperation::Unlink];
     operations.extend(std::iter::repeat_n(
         HistoryOperation::CreateInode,
@@ -75,16 +79,16 @@ fn creating_inodes_one_at_a_time_past_the_first_leaf_container_is_judged_by_the_
         .copied()
         .expect("建过 inode：这一类在计数里");
     assert_eq!(
-        created.applied, INODE_CREATIONS_INTO_THE_SECOND_LEAF_CONTAINER,
-        "建过的 inode 多于一片叶容器装的 {INODE_LEAF_RECORDS} 条：{created:?}"
+        created.applied, INODE_CREATIONS_INTO_THE_THIRD_LEAF_CONTAINER,
+        "建过的 inode 多于两片叶容器装的 2 × {INODE_LEAF_RECORDS} 条：{created:?}"
     );
     let counts = &run.tally.model_counts;
     assert!(
-        counts.successes_matched > INODE_CREATIONS_INTO_THE_SECOND_LEAF_CONTAINER,
+        counts.successes_matched > INODE_CREATIONS_INTO_THE_THIRD_LEAF_CONTAINER,
         "模型判过 unlink 与每一次建 inode：{counts:?}"
     );
     assert!(
-        counts.allocation_records_compared > INODE_CREATIONS_INTO_THE_SECOND_LEAF_CONTAINER,
+        counts.allocation_records_compared > INODE_CREATIONS_INTO_THE_THIRD_LEAF_CONTAINER,
         "每一次建 inode 都逐个角色比过分配记录：{counts:?}"
     );
 }
````

### mutations-append.tsv

````text
O1 跨叶容器回退（理想模型自身）：回退时不丢现行那一版有、目标那一版没有的右侧叶容器（攻方 drop-retain）	crates/singlefs-harness/src/model.rs	                role_written_at.retain(|role, _| {\n                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                });	                let _ = &role_written_at;	-p singlefs-harness --test model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers	a_rollback_past_an_inode_leaf_container_split_then_creating_inodes_is_judged_by_the_model_container_by_container
O1 跨叶容器回退（理想模型自身）：回退时不取回目标那一版的右侧叶容器（攻方 drop-extend）	crates/singlefs-harness/src/model.rs	                role_written_at.extend(\n                    target_root\n	                let _ = (&role_written_at, &target_root);\n                Vec::<(ModelUnitRole, ModelCheckpointTxg)>::new().extend(\n                    target_root\n	-p singlefs-harness --test model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers	a_rollback_past_an_inode_leaf_container_split_then_creating_inodes_is_judged_by_the_model_container_by_container
O1 跨叶容器（胶水）：第 3 片及以后都当第 2 片（正推 KX1）	crates/singlefs-harness/src/model_comparison.rs	            ModelUnitRole::inode_leaf_container_at(u64::from(index.0))	            ModelUnitRole::inode_leaf_container_at(u64::from(index.0).min(1))	-p singlefs-harness --test model_compares_inode_creations_across_two_inode_leaf_containers	creating_inodes_one_at_a_time_past_the_first_leaf_container_is_judged_by_the_model_container_by_container
O1 跨叶容器（理想模型自身）：落点封顶在第 2 片（正推 KX2）	crates/singlefs-harness/src/model.rs	    let position_from_the_leftmost = records_before / INODE_LEAF_RECORDS;	    let position_from_the_leftmost = (records_before / INODE_LEAF_RECORDS).min(1);	-p singlefs-harness --test model_compares_inode_creations_across_two_inode_leaf_containers	creating_inodes_one_at_a_time_past_the_first_leaf_container_is_judged_by_the_model_container_by_container
O1 跨叶容器 135 片边界（实现）：叶容器数的上限放宽一片，第 136 片照收	crates/singlefs-core/src/inode_tree.rs	    if containers.len() > capacity {	    if containers.len() > capacity + 1 {	-p singlefs-harness --test inode_tree_leaf_containers_appended_one_at_a_time_up_to_what_one_root_node_holds	appending_inode_records_one_at_a_time_rewrites_container_floor_k_over_233_and_refuses_the_136th_container
O1 跨叶容器 135 片边界（实现）：提前一条分裂，第 k 条落的片不是 ⌊k ÷ 233⌋	crates/singlefs-core/src/inode_tree.rs	    if last.free_record_slots() == 0 {	    if last.free_record_slots() <= 1 {	-p singlefs-harness --test inode_tree_leaf_containers_appended_one_at_a_time_up_to_what_one_root_node_holds	appending_inode_records_one_at_a_time_rewrites_container_floor_k_over_233_and_refuses_the_136th_container
E171R1 装置 V20 Q5 类核对 ① 不计数（登记修订 5 第 5.4 节）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            checks.input_list_mismatches += 1;	            // V20：类核对 ① 不计数	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_input_list_check_counts_a_state_whose_columns_do_not_follow_the_four_inputs
E171R1 装置 V21 Q5 类核对 ② 不计数（登记修订 5 第 5.4 节）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	                checks.in_flight_writes_of_a_forbidden_kind += 1;	                // V21：类核对 ② 不计数	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	an_in_flight_write_classified_as_a_journal_record_is_a_forbidden_kind
E171R1 装置 V22 Q5 类核对 ③ 的 journal 提示不记（登记修订 5 第 5.4 节）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        self.read_ranges.borrow_mut().push(RecordedReadRange {\n            device,\n            start_in_bytes: ring_start.0,\n            end_in_bytes: ring_start.0 + ring_bytes,\n            source: RecordedReadSource::JournalRecordOffsetsHint,\n        });	        // V22：journal 提示那一段不记	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	an_in_flight_write_in_an_unread_journal_ring_slot_is_moved_into_the_key_through_the_hint
E171R1 装置 V23 Q5 类核对 ④ 前一半不计数（登记修订 5 第 5.4 节）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            checks.recording_report_mismatches += 1;	            // V23：类核对 ④ 前一半不计数	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_recording_recovery_that_differs_from_the_layer0_report_is_counted
E171R1 装置 V24 Q5 类核对 ④ 后一半不计数（登记修订 5 第 5.4 节）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            checks.derived_class_mismatches += 1;	            // V24：类核对 ④ 后一半不计数	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_fully_landed_state_that_differs_from_its_derived_class_is_counted
E171R1 装置 V25 Q5 代表的落地串核对恒过（登记修订 5 第 5.4 节）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            || !class_landings_are(representative, '0')	            || false	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_landing_string_off_the_partition_is_counted_and_stops_at_s13
E171R1 装置 V26 Q5 全落那一态的落地串核对恒过（登记修订 5 第 5.4 节）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            || !class_landings_are(fully_landed, highest_digit_landing)	            || false	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_landing_string_off_the_partition_is_counted_and_stops_at_s13
E171R1 装置 V27 Q5 S13 不看落地串核对（登记修订 5 第 5.4 节）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        || checks.class_landing_mismatches > 0	        // V27：S13 不看落地串核对	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_landing_string_off_the_partition_is_counted_and_stops_at_s13
E171R1 装置 V28 判态路任一件 panic 两个崩溃子列都记恢复 panic（登记修订 5 第 5.7 节，退回修订 4 的读法）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    let columns_follow_a_recovery_panic = matches!(consulted, PieceOutcome::Panicked(_));	    let columns_follow_a_recovery_panic = matches!(consulted, PieceOutcome::Panicked(_)) || !piece_panics.is_empty();	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_panic_off_the_consulted_recovery_leaves_the_two_crash_columns_to_the_four_inputs
E171R1 装置 V29 判态路 pool_checker 那一件 panic 不记件名（登记修订 5 第 5.7 节）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    note_a_piece_panic(&mut piece_panics, JudgingPiece::PoolChecker, &pool_checker);	    // V29：pool_checker 那一件 panic 不记件名	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_panic_off_the_consulted_recovery_leaves_the_two_crash_columns_to_the_four_inputs
E171R1 装置 V30 Q5 没有根写的前置判删掉、退回先对拍（登记修订 5 第 5.7 节）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    if let Some(shape_name) = shape_without_a_root_write(&shapes) {	    if let Some(shape_name) = shape_without_a_root_write(&shapes).filter(|_| false) {	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_shape_without_a_root_write_stops_before_the_pairing
E171R1 装置 V31 Q5 对拍的 k 由 6 改回 2（登记修订 5 第 5.7 节）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	const EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT: usize = 6;	const EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT: usize = 2;	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_pairing_keeps_the_first_three_mirrored_pairs_of_the_in_flight_call
````

## m3-formal-27-2b

### crates.patch

````diff
diff --git a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
index 9cfa2de..6b0b5f4 100644
--- a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
+++ b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
@@ -3521,6 +3521,9 @@ struct StateObservation {
     consulted_report: Option<RecoveryReport>,
     /// 回调链 panic 了的态上，逐件重做时 panic 的件（[`JudgingPiece`] 的次序）；回调链没 panic 的态是空的（修订 5 第 5.1 节第 5 条）。
     piece_panics: Vec<JudgingPiece>,
+    /// 正常那一路（`judge_layer0_state_range` 的回调链）panic 了的态是那一次 panic 的消息（不进产物），没 panic 的态是 None。
+    /// 有值而 `piece_panics` 是空的态数进 `unattributed_panic_states=`（[`unattributed_panic_states_of`]）。
+    judging_path_panic: Option<String>,
 }
 
 /// 判一态那条回调链上的一件（修订 5 第 5.1 节第 2 条）：回调链 panic 了，装置按这个次序在同一个镜像上逐件重做、每件各包一层 `catch_unwind`。
@@ -3663,6 +3666,19 @@ fn piece_panic_states_of<'observation>(
     states
 }
 
+/// 正常那一路 `catch_unwind` 接到 panic、逐件重做十件一件都没 panic 的态数（`unattributed_panic_states=`，
+/// 登记「主 agent 认定：27-2 装置交回」第 2 条：甲那条管子漏了一件）。
+fn unattributed_panic_states_of<'observation>(
+    observations: impl IntoIterator<Item = &'observation StateObservation>,
+) -> usize {
+    observations
+        .into_iter()
+        .filter(|observation| {
+            observation.judging_path_panic.is_some() && observation.piece_panics.is_empty()
+        })
+        .count()
+}
+
 /// `name=batch`、`name=verdict` 两种行的 `piece_panic_states=` 字段：`<件>:<态数>` 逗号连起来，0 的件不写，全 0 写 `-`。
 fn piece_panic_states_text(states: &[usize; JUDGING_PIECE_COUNT]) -> String {
     let nonzero: Vec<String> = JudgingPiece::ALL
@@ -3678,7 +3694,7 @@ fn piece_panic_states_text(states: &[usize; JUDGING_PIECE_COUNT]) -> String {
     }
 }
 
-/// `name=q5_piece_panics` 那一行：十件各一个字段（修订 5 第 5.1 节第 5 条）。
+/// `name=q5_piece_panics` 那一行：十件各一个字段（修订 5 第 5.1 节第 5 条），再带 `unattributed_panic_states=`。
 fn q5_piece_panics_line(sampling_point: &str, observations: &[StateObservation]) -> String {
     let states = piece_panic_states_of(observations);
     let fields: Vec<String> = JudgingPiece::ALL
@@ -3687,21 +3703,43 @@ fn q5_piece_panics_line(sampling_point: &str, observations: &[StateObservation])
         .map(|(piece, count)| format!("{}={count}", piece.name()))
         .collect();
     format!(
-        "name=q5_piece_panics sampling_point={sampling_point} judged_states={} {}",
+        "name=q5_piece_panics sampling_point={sampling_point} judged_states={} {} unattributed_panic_states={}",
         observations.len(),
-        fields.join(" ")
+        fields.join(" "),
+        unattributed_panic_states_of(observations)
     )
 }
 
+/// 一组态里有 [`unattributed_panic_states_of`] 数到的态（登记「主 agent 认定：27-2 装置交回」第 2 条）：这一趟的 Q5 不判甲乙、不进 S13，
+/// 打 `q5_not_run=unattributed_panic` 的判决行、收尾，交回退出码 3（交主 agent）；一态都没有交 `None`，调用方接着判。
+fn stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(
+    lines: &mut ResultLines,
+    failed_checks: usize,
+    observations: &[StateObservation],
+) -> Option<i32> {
+    let unattributed_panic_states = unattributed_panic_states_of(observations);
+    if unattributed_panic_states == 0 {
+        return None;
+    }
+    lines.result(&format!(
+        "name=verdict s1_failed_checks={failed_checks} s13=false q5_not_run=unattributed_panic piece_panic_states={} unattributed_panic_states={unattributed_panic_states}",
+        piece_panic_states_text(&piece_panic_states_of(observations))
+    ));
+    lines.finish();
+    Some(3)
+}
+
 /// 阳性对照「判态路 panic 单列」的注入（修订 5 第 5.1 节）：按落地串选态，选中的态正常那一路在回调里先 panic（逼走逐件重做那一路），
 /// 逐件重做时点名的那一件 panic。只经装置内部造（[`piece_panic_control_injection`] 与内联用例），命令行不开放；不改实现。
 struct PiecePanicInjection {
     rules: Vec<PiecePanicInjectionRule>,
 }
 
-/// 一条注入：落地串满足全部条件的态，`piece` 那一件 panic。各条选中的态互不相交（造的人保证；[`PiecePanicInjection::piece_of`] 取第一条）。
+/// 一条注入：落地串满足全部条件的态，正常那一路 panic、逐件重做时 `piece` 那一件 panic。各条选中的态互不相交
+/// （造的人保证；[`PiecePanicInjection::rule_of`] 取第一条）。
 struct PiecePanicInjectionRule {
-    piece: JudgingPiece,
+    /// `None`：逐件重做时一件都不 panic，只有正常那一路 panic（造 `unattributed_panic_states=` 数的那种态，只有内联用例造）。
+    piece: Option<JudgingPiece>,
     conditions: Vec<InjectedLandingCondition>,
 }
 
@@ -3713,26 +3751,23 @@ enum InjectedLandingCondition {
 }
 
 impl PiecePanicInjection {
-    /// 这一态（落地串）要注入哪一件 panic。
-    fn piece_of(&self, landing_vector: &str) -> Option<JudgingPiece> {
+    /// 这一态（落地串）选中的那一条注入；没有一条选中交 `None`。
+    fn rule_of(&self, landing_vector: &str) -> Option<&PiecePanicInjectionRule> {
         let landed = |write_index: usize| {
             landing_vector
                 .as_bytes()
                 .get(write_index)
                 .is_some_and(|landing| *landing != b'0')
         };
-        self.rules
-            .iter()
-            .find(|rule| {
-                rule.conditions.iter().all(|condition| match condition {
-                    InjectedLandingCondition::Landed(write_index) => landed(*write_index),
-                    InjectedLandingCondition::NotLanded(write_index) => !landed(*write_index),
-                    InjectedLandingCondition::BothOrNeitherLanded(first, second) => {
-                        landed(*first) == landed(*second)
-                    }
-                })
+        self.rules.iter().find(|rule| {
+            rule.conditions.iter().all(|condition| match condition {
+                InjectedLandingCondition::Landed(write_index) => landed(*write_index),
+                InjectedLandingCondition::NotLanded(write_index) => !landed(*write_index),
+                InjectedLandingCondition::BothOrNeitherLanded(first, second) => {
+                    landed(*first) == landed(*second)
+                }
             })
-            .map(|rule| rule.piece)
+        })
     }
 }
 
@@ -3896,7 +3931,7 @@ fn observation_of_a_judged_state(
     assert!(
         context
             .piece_panic_injection
-            .and_then(|injection| injection.piece_of(&landing_vector))
+            .and_then(|injection| injection.rule_of(&landing_vector))
             .is_none(),
         "修订 5 阳性对照「判态路 panic 单列」注入：正常那一路在这一态的回调里先 panic"
     );
@@ -4014,6 +4049,7 @@ fn observation_of_a_judged_state(
             ConsultedReports::Kept => Some(judgement.consulted.clone()),
         },
         piece_panics: Vec::new(),
+        judging_path_panic: None,
     }
 }
 
@@ -4021,13 +4057,15 @@ fn observation_of_a_judged_state(
 /// 的次序逐件重做、每件各包一层 `catch_unwind`（[`JudgingPiece`]），panic 了记下件名、接着做下一件。O1 两个崩溃子列只随看 journal 那遍恢复：
 /// 它 panic 两列都记恢复 panic（读法 5），没 panic 两列照正常那一路的函数从这一遍的恢复报告算，O1-模型崩溃判官或 RefFS 自己 panic 那一列记
 /// `judge_panic`；别的件 panic 不改这两列（第 3 条）。O2 照判，panic 记 `pool_checker_panic`、按红计（第 4 条）。
-/// 阳性对照的翻读回、见证改写与 O2 的坏镜像都只在正常那一路做，这一路不做。
+/// 阳性对照的翻读回、见证改写与 O2 的坏镜像都只在正常那一路做，这一路不做。`judging_path_panic` 是正常那一路 panic 的消息，
+/// 记进 [`StateObservation::judging_path_panic`]、不进产物。
 fn observation_of_a_panicking_state(
     context: &ObservationContext<'_>,
     batch: u8,
     ordinal: u64,
     switches: RunSwitches,
     consulted_reports: ConsultedReports,
+    judging_path_panic: String,
 ) -> StateObservation {
     let stream = context.stream;
     let persisted = context.facts.persisted_writes_of_state(ordinal);
@@ -4050,7 +4088,8 @@ fn observation_of_a_panicking_state(
     let segment_index = context.segment_of_ordinal(ordinal);
     let injected = context
         .piece_panic_injection
-        .and_then(|injection| injection.piece_of(&landing_vector));
+        .and_then(|injection| injection.rule_of(&landing_vector))
+        .and_then(|rule| rule.piece);
     let mut piece_panics: Vec<JudgingPiece> = Vec::new();
     let consulted = judging_piece_outcome(JudgingPiece::ConsultedRecovery, injected, || {
         singlefs_core::recovery::recover(&image, singlefs_core::recovery::JournalPolicy::Consult)
@@ -4249,6 +4288,7 @@ fn observation_of_a_panicking_state(
             (ConsultedReports::Kept, None) | (ConsultedReports::Dropped, _) => None,
         },
         piece_panics,
+        judging_path_panic: Some(judging_path_panic),
     }
 }
 
@@ -4343,13 +4383,14 @@ fn observe_sampled_states(
         "panic 的态两处记的一样"
     );
     // 正常那一路 panic 的消息不进产物：逐件重做时各件自己的结局才是这一态的判定（修订 5 第 5.1 节第 2 条）。
-    for (ordinal, _message_of_the_judging_path) in panicking_states {
+    for (ordinal, message_of_the_judging_path) in panicking_states {
         observations.push(observation_of_a_panicking_state(
             context,
             batch_of_ordinal[&ordinal],
             ordinal,
             switches,
             consulted_reports,
+            message_of_the_judging_path,
         ));
     }
     observations.sort_by_key(|observation| (observation.batch, observation.ordinal));
@@ -6868,6 +6909,8 @@ struct StreamTotals {
     recovery_panic_states: usize,
     /// 各件 panic 的态数（修订 5 第 5.1 节第 5 条，按 [`JudgingPiece::ALL`] 的次序）。
     piece_panic_states: [usize; JUDGING_PIECE_COUNT],
+    /// 正常那一路 panic、逐件一件都没 panic 的态数（[`unattributed_panic_states_of`]）。
+    unattributed_panic_states: usize,
     probed_states: usize,
     /// (ii) 为真的态、其中第二截判合法的、(i) 为真的态（Q4）。
     counter_branch_states: usize,
@@ -7016,6 +7059,7 @@ fn report_batch(
     for (total, states) in totals.piece_panic_states.iter_mut().zip(piece_panic_states) {
         *total += states;
     }
+    totals.unattributed_panic_states += unattributed_panic_states_of(in_batch.iter().copied());
     let segments: BTreeSet<usize> = in_batch
         .iter()
         .map(|observation| observation.segment_index)
@@ -7474,7 +7518,7 @@ fn run_dump(command_line: &CommandLine) -> i32 {
     }
     failed_checks += totals.failed_checks;
     lines.result(&format!(
-        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={} crash_facts_mask_mismatches={} s3_o2_judgement_mismatches={} s9_failed_checks={s9_failed} streams_not_exported={streams_not_exported} o1_comparison_red_calls={} o1_crash_judge_red_states={} o1_reference_filesystem_red_states={} o2_red_states={} recovery_panic_states={} probed_states={} counter_branch_states={} counter_branch_states_judged_legal={} undecidable_branch_states={} witness_refusals_without_either_branch={} counter_branch_states_that_mounted={} piece_panic_states={}",
+        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={} crash_facts_mask_mismatches={} s3_o2_judgement_mismatches={} s9_failed_checks={s9_failed} streams_not_exported={streams_not_exported} o1_comparison_red_calls={} o1_crash_judge_red_states={} o1_reference_filesystem_red_states={} o2_red_states={} recovery_panic_states={} probed_states={} counter_branch_states={} counter_branch_states_judged_legal={} undecidable_branch_states={} witness_refusals_without_either_branch={} counter_branch_states_that_mounted={} piece_panic_states={} unattributed_panic_states={}",
         totals.rebuilt_digest_mismatches,
         totals.mask_mismatches,
         totals.s3_mismatches,
@@ -7490,6 +7534,7 @@ fn run_dump(command_line: &CommandLine) -> i32 {
         totals.witness_refusals_without_either_branch,
         totals.counter_branch_states_that_mounted,
         piece_panic_states_text(&totals.piece_panic_states),
+        totals.unattributed_panic_states,
     ));
     lines.finish();
     let s11_hit = totals.witness_refusals_without_either_branch > 0
@@ -7524,6 +7569,7 @@ fn add_stream_totals(totals: &mut StreamTotals, stream: &StreamTotals) {
     {
         *total += states;
     }
+    totals.unattributed_panic_states += stream.unattributed_panic_states;
     totals.probed_states += stream.probed_states;
     totals.counter_branch_states += stream.counter_branch_states;
     totals.counter_branch_states_judged_legal += stream.counter_branch_states_judged_legal;
@@ -7981,7 +8027,7 @@ fn judge_the_confirmed_loss_shapes(
             "name=q5 sampling_point={sampling_point} not_run=no_root_write shape={shape_name}"
         ));
         lines.result(&format!(
-            "name=verdict s1_failed_checks={failed_checks} s13=false q5_not_run=no_root_write piece_panic_states=-"
+            "name=verdict s1_failed_checks={failed_checks} s13=false q5_not_run=no_root_write piece_panic_states=- unattributed_panic_states=0"
         ));
         lines.finish();
         return 3;
@@ -8027,9 +8073,16 @@ fn judge_the_confirmed_loss_shapes(
             field_text(&kept_check.crates),
             kept_check.holds()
         ));
+        if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(
+            lines,
+            failed_checks,
+            &pairing.observations,
+        ) {
+            return exit_code;
+        }
         if pairing.stops_at_s13() {
             lines.result(&format!(
-                "name=verdict s1_failed_checks={failed_checks} s13=equivalence_pairing q5_not_run=equivalence_pairing_failed piece_panic_states=-"
+                "name=verdict s1_failed_checks={failed_checks} s13=equivalence_pairing q5_not_run=equivalence_pairing_failed piece_panic_states=- unattributed_panic_states=0"
             ));
             lines.finish();
             return 3;
@@ -8053,10 +8106,17 @@ fn judge_the_confirmed_loss_shapes(
             "piece_panic_control",
             &control.observations,
         ));
+        if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(
+            lines,
+            failed_checks,
+            &control.observations,
+        ) {
+            return exit_code;
+        }
         if !control.holds() {
             // 失败 F9（修订 5 第 5.7 节）：甲那条管子接错了，主取样点不跑、不记成「O1 盲」。
             lines.result(&format!(
-                "name=verdict s1_failed_checks={failed_checks} s13=false q5_not_run=piece_panic_control_failed piece_panic_states=-"
+                "name=verdict s1_failed_checks={failed_checks} s13=false q5_not_run=piece_panic_control_failed piece_panic_states=- unattributed_panic_states=0"
             ));
             lines.finish();
             return 3;
@@ -8102,7 +8162,7 @@ fn judge_the_confirmed_loss_shapes(
                 "name=q5 sampling_point={sampling_point} not_run=too_many_states states={product} limit={CONFIRMED_LOSS_STATE_LIMIT}"
             ));
             lines.result(&format!(
-                "name=verdict s1_failed_checks={failed_checks} q5_not_run=too_many_states piece_panic_states=-"
+                "name=verdict s1_failed_checks={failed_checks} q5_not_run=too_many_states piece_panic_states=- unattributed_panic_states=0"
             ));
             lines.finish();
             return 3;
@@ -8120,6 +8180,13 @@ fn judge_the_confirmed_loss_shapes(
         export.checks.extend(s9);
         let totals = report_stream(lines, &export);
         lines.result(&q5_piece_panics_line("reverse", &export.observations));
+        if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(
+            lines,
+            failed_checks + totals.failed_checks,
+            &export.observations,
+        ) {
+            return exit_code;
+        }
         let rows: Vec<ConfirmedLossTallyRow<'_>> = export
             .observations
             .iter()
@@ -8154,7 +8221,7 @@ fn judge_the_confirmed_loss_shapes(
                 partition_fields(&classification, &EquivalenceCheckCounts::default())
             ));
             lines.result(&format!(
-                "name=verdict s1_failed_checks={failed_checks} s13=state_limit q5_not_run=too_many_states_after_refinement piece_panic_states=-"
+                "name=verdict s1_failed_checks={failed_checks} s13=state_limit q5_not_run=too_many_states_after_refinement piece_panic_states=- unattributed_panic_states=0"
             ));
             lines.finish();
             return 3;
@@ -8174,6 +8241,13 @@ fn judge_the_confirmed_loss_shapes(
         export.checks.extend(s9);
         let totals = report_stream(lines, &export);
         lines.result(&q5_piece_panics_line("main", &export.observations));
+        if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(
+            lines,
+            failed_checks + totals.failed_checks,
+            &export.observations,
+        ) {
+            return exit_code;
+        }
         if let Some(why) = &export.not_exported {
             // 第二道（第一道是修订 5 第 5.3 节的 [`shape_without_a_root_write`]）：这一形截出来的写流里没有可判的根槽写
             // （`export_prepared_stream` 记 `no_root_write_to_judge`）：类核对没走到，不是 S13；记 Q5 没跑、不进判定
@@ -8183,11 +8257,12 @@ fn judge_the_confirmed_loss_shapes(
                 field_text(why)
             ));
             lines.result(&format!(
-                "name=verdict s1_failed_checks={} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states=0 s13=false q5_not_run=no_root_write piece_panic_states={}",
+                "name=verdict s1_failed_checks={} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states=0 s13=false q5_not_run=no_root_write piece_panic_states={} unattributed_panic_states={}",
                 failed_checks + totals.failed_checks,
                 totals.rebuilt_digest_mismatches,
                 totals.s3_mismatches,
-                piece_panic_states_text(&totals.piece_panic_states)
+                piece_panic_states_text(&totals.piece_panic_states),
+                totals.unattributed_panic_states
             ));
             lines.finish();
             return 3;
@@ -8243,11 +8318,12 @@ fn judge_the_confirmed_loss_shapes(
     };
     failed_checks += totals.failed_checks;
     lines.result(&format!(
-        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states={} s13={stops_at_s13} piece_panic_states={}",
+        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states={} s13={stops_at_s13} piece_panic_states={} unattributed_panic_states={}",
         totals.rebuilt_digest_mismatches,
         totals.s3_mismatches,
         export.observations.len(),
-        piece_panic_states_text(&totals.piece_panic_states)
+        piece_panic_states_text(&totals.piece_panic_states),
+        totals.unattributed_panic_states
     ));
     lines.finish();
     if failed_checks > 0
@@ -9404,21 +9480,21 @@ fn piece_panic_control_injection(in_flight_writes: &[usize]) -> Option<PiecePani
     Some(PiecePanicInjection {
         rules: vec![
             PiecePanicInjectionRule {
-                piece: JudgingPiece::PoolChecker,
+                piece: Some(JudgingPiece::PoolChecker),
                 conditions: vec![
                     InjectedLandingCondition::Landed(*third),
                     InjectedLandingCondition::NotLanded(*fourth),
                 ],
             },
             PiecePanicInjectionRule {
-                piece: JudgingPiece::IgnoredRecovery,
+                piece: Some(JudgingPiece::IgnoredRecovery),
                 conditions: vec![
                     InjectedLandingCondition::NotLanded(*third),
                     InjectedLandingCondition::Landed(*fourth),
                 ],
             },
             PiecePanicInjectionRule {
-                piece: JudgingPiece::RecordChecker,
+                piece: Some(JudgingPiece::RecordChecker),
                 conditions: vec![
                     InjectedLandingCondition::Landed(*fifth),
                     InjectedLandingCondition::NotLanded(*sixth),
@@ -11633,21 +11709,21 @@ mod tests {
         let injection = PiecePanicInjection {
             rules: vec![
                 PiecePanicInjectionRule {
-                    piece: JudgingPiece::PoolChecker,
+                    piece: Some(JudgingPiece::PoolChecker),
                     conditions: vec![
                         InjectedLandingCondition::Landed(third),
                         InjectedLandingCondition::NotLanded(fourth),
                     ],
                 },
                 PiecePanicInjectionRule {
-                    piece: JudgingPiece::IgnoredRecovery,
+                    piece: Some(JudgingPiece::IgnoredRecovery),
                     conditions: vec![
                         InjectedLandingCondition::NotLanded(third),
                         InjectedLandingCondition::Landed(fourth),
                     ],
                 },
                 PiecePanicInjectionRule {
-                    piece: JudgingPiece::RecordChecker,
+                    piece: Some(JudgingPiece::RecordChecker),
                     conditions: vec![
                         InjectedLandingCondition::NotLanded(first),
                         InjectedLandingCondition::Landed(second),
@@ -11815,4 +11891,73 @@ mod tests {
             "c1…c6 两两同偏移、同长度、分属两块盘，三对的偏移互不相同"
         );
     }
+
+    /// 登记「主 agent 认定：27-2 装置交回」第 2 条（变异「unattributed_panic_states 恒 0」在这里红）：对拍流（k = 2）末段第一态上注入一条
+    /// 不点名件的规则，只让正常那一路的回调 panic；逐件重做十件一件都不 panic。`unattributed_panic_states=` 数到 1、
+    /// `name=q5_piece_panics` 行带它，Q5 走 `q5_not_run=unattributed_panic` 那个出口：判决行 `s13=false`、退 3。
+    #[test]
+    fn a_judging_path_panic_that_no_piece_repeats_stops_the_q5_as_unattributed() {
+        let stream = equivalence_pairing_stream();
+        let observation_tables = ObservationTables::of(&stream);
+        let injection = PiecePanicInjection {
+            rules: vec![PiecePanicInjectionRule {
+                piece: None,
+                conditions: Vec::new(),
+            }],
+        };
+        let mut context = observation_tables.context(&stream);
+        context.piece_panic_injection = Some(&injection);
+        let digits = LastSegmentDigits::of(&stream, &observation_tables.tables);
+        let observations = observe_sampled_states(
+            &context,
+            &SampledOrdinals {
+                batches: [
+                    BTreeSet::from([digits.first_ordinal]),
+                    BTreeSet::new(),
+                    BTreeSet::new(),
+                ],
+                torn_scans: Vec::new(),
+            },
+            RunSwitches::default(),
+            ConsultedReports::Dropped,
+        );
+        assert_eq!(observations.len(), 1, "只判末段第一态");
+        assert!(
+            observations[0].judging_path_panic.is_some(),
+            "正常那一路照注入 panic 了"
+        );
+        assert_eq!(
+            piece_panic_states_of(&observations),
+            [0; JUDGING_PIECE_COUNT],
+            "逐件重做十件一件都没 panic"
+        );
+        assert_eq!(unattributed_panic_states_of(&observations), 1);
+        let piece_panics_line = q5_piece_panics_line("pairing", &observations);
+        assert!(
+            piece_panics_line.ends_with(" unattributed_panic_states=1"),
+            "{piece_panics_line}"
+        );
+        let mut lines = ResultLines {
+            emitted: 0,
+            collected: Some(Vec::new()),
+        };
+        let exit_code = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(
+            &mut lines,
+            0,
+            &observations,
+        );
+        let collected = lines.collected.expect("收集模式");
+        assert_eq!(exit_code, Some(3), "退 3：{collected:?}");
+        assert_eq!(
+            collected
+                .iter()
+                .filter(|line| line.starts_with("name=verdict ")
+                    && line.contains(" s13=false")
+                    && line.contains(" q5_not_run=unattributed_panic ")
+                    && line.ends_with(" unattributed_panic_states=1"))
+                .count(),
+            1,
+            "一行判决带 s13=false、q5_not_run=unattributed_panic 与 unattributed_panic_states=1：{collected:?}"
+        );
+    }
 }
diff --git a/crates/singlefs-harness/src/model.rs b/crates/singlefs-harness/src/model.rs
index 70fa7e6..a649d07 100644
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -4253,4 +4253,38 @@ mod tests {
             "映射树至多 5 个节点（根之下 4 个），分配记录树至多 123 个节点：这次占槽上界 722"
         );
     }
+
+    /// 一个码 2 根装 135 片叶容器（D8（核心索引结构） 已定项 6）：删完之后建的第 233 × 135 − 1 条（从 0 数）落在最后那片（第 134 片），
+    /// 第 233 × 135 条要第 136 片、根要长一层，模型的落点式子在那里 panic（判决 `research/prompts/m3-formal-code-b25-r1-main-verification.md`
+    /// K1 第四行：保留 `assert!`、不答拒；core 在同一条拒，`MoreLeafContainersThanOneRootNodeHolds`）。
+    #[test]
+    fn the_inode_leaf_container_position_reaches_the_last_of_135_leaf_containers_and_panics_one_record_beyond(
+    ) {
+        const LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS: u64 = 135;
+        assert_eq!(
+            index_node_entry_capacity(INODE_TREE_KEY_BYTES, INODE_INTERNAL_ENTRY),
+            LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS,
+            "模型按条款算的码 2 根扇出是 135"
+        );
+        let records_one_root_holds = INODE_LEAF_RECORDS * LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS;
+        assert_eq!(
+            inode_leaf_container_position_of_a_new_record(records_one_root_holds - 1),
+            LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS - 1,
+            "第 233 × 135 − 1 条落在第 134 片"
+        );
+        let one_record_beyond = std::panic::catch_unwind(|| {
+            inode_leaf_container_position_of_a_new_record(records_one_root_holds)
+        });
+        let payload =
+            one_record_beyond.expect_err("第 233 × 135 条要第 136 片：模型的落点式子 panic");
+        let message = payload
+            .downcast_ref::<String>()
+            .expect("assert! 带格式参数，panic 的载荷是 String");
+        assert!(
+            message.contains(&format!(
+                "第 {records_one_root_holds} 条落在第 {LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS} 片"
+            )),
+            "panic 的是落点式子那一条 assert!：{message}"
+        );
+    }
 }
diff --git a/crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs b/crates/singlefs-harness/tests/model_compares_inode_creations_into_the_third_inode_leaf_container.rs
similarity index 100%
rename from crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs
rename to crates/singlefs-harness/tests/model_compares_inode_creations_into_the_third_inode_leaf_container.rs
````

### mutations-append.tsv

````text
K1 第四行（理想模型自身）：inode 树落点的 assert! 放宽一片（第 233 × 135 条照样落在第 135 片、不 panic）	crates/singlefs-harness/src/model.rs	        position_from_the_leftmost < leaf_containers_one_root_holds,	        position_from_the_leftmost <= leaf_containers_one_root_holds,	-p singlefs-harness --lib -- model::tests::the_inode_leaf_container_position_reaches	the_inode_leaf_container_position_reaches_the_last_of_135_leaf_containers_and_panics_one_record_beyond
E171R1 装置 unattributed_panic_states 恒 0（登记「主 agent 认定：27-2 装置交回」第 2 条）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            observation.judging_path_panic.is_some() && observation.piece_panics.is_empty()	            false && observation.judging_path_panic.is_some() && observation.piece_panics.is_empty()	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_judging_path_panic_that_no_piece_repeats_stops_the_q5_as_unattributed
````

### mutations-replacements.tsv

````text
O1 跨叶容器（理想模型自身）：第二片晚一条才分出（一片按 234 条记，第 234 个 inode 的新记录与重写记成最左那片）	crates/singlefs-harness/src/model.rs	    let position_from_the_leftmost = records_before / INODE_LEAF_RECORDS;	    let position_from_the_leftmost = records_before / (INODE_LEAF_RECORDS + 1);	-p singlefs-harness --test model_compares_inode_creations_into_the_third_inode_leaf_container	creating_inodes_one_at_a_time_past_the_first_leaf_container_is_judged_by_the_model_container_by_container
O1 跨叶容器（胶水）：实现交回的每片叶容器都当成最左那片	crates/singlefs-harness/src/model_comparison.rs	            ModelUnitRole::inode_leaf_container_at(u64::from(index.0))	            ModelUnitRole::inode_leaf_container_at(u64::from(index.0).min(0))	-p singlefs-harness --test model_compares_inode_creations_into_the_third_inode_leaf_container	creating_inodes_one_at_a_time_past_the_first_leaf_container_is_judged_by_the_model_container_by_container
O1 跨叶容器（理想模型自身）：新记录落在第二片时最左那片也记成这次重写（分配代按第一片记）	crates/singlefs-harness/src/model.rs	            ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => &[\n                ModelUnitRole::InodeRoot,	            ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => &[\n                ModelUnitRole::InodeLeaf,\n                ModelUnitRole::InodeRoot,	-p singlefs-harness --test model_compares_inode_creations_into_the_third_inode_leaf_container	creating_inodes_one_at_a_time_past_the_first_leaf_container_is_judged_by_the_model_container_by_container
O1 跨叶容器（胶水）：第 3 片及以后都当第 2 片（正推 KX1）	crates/singlefs-harness/src/model_comparison.rs	            ModelUnitRole::inode_leaf_container_at(u64::from(index.0))	            ModelUnitRole::inode_leaf_container_at(u64::from(index.0).min(1))	-p singlefs-harness --test model_compares_inode_creations_into_the_third_inode_leaf_container	creating_inodes_one_at_a_time_past_the_first_leaf_container_is_judged_by_the_model_container_by_container
O1 跨叶容器（理想模型自身）：落点封顶在第 2 片（正推 KX2）	crates/singlefs-harness/src/model.rs	    let position_from_the_leftmost = records_before / INODE_LEAF_RECORDS;	    let position_from_the_leftmost = (records_before / INODE_LEAF_RECORDS).min(1);	-p singlefs-harness --test model_compares_inode_creations_into_the_third_inode_leaf_container	creating_inodes_one_at_a_time_past_the_first_leaf_container_is_judged_by_the_model_container_by_container
````

## m3-formal-28-1

### crates.patch

````diff
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index ca434f5..4045711 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -3480,8 +3480,9 @@ impl HeadIdentifierOfATreeTableEntry {
 /// 或哪一种。
 /// ① 盘上相邻两条的树 ID（条目打头 8 字节）严格升序；② 按发号次序（[`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`]），
 /// 树表里有的种类相邻两棵的树 ID 严格升序；③ 同一张树表里每一种种类码（登记过的与没登记的都算）至多一条；④ 树表非空时
-/// 种类 1、2、3、4（extent 树、inode 树、分配记录树、记账树：读者重建一版要的有根的树）各恰好一条
-/// （判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y3「种类 1 不止一条；种类 3 不是恰好一条」那一行）。
+/// 进树表的十一种（[`TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES`]）各恰好一条
+/// （判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y3「种类 1 不止一条；种类 3 不是恰好一条」那一行；
+/// 十一种取自判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-9.16 ④ 放过缺种类的树表（C1）」那一行）。
 /// 0 条的树表四道都成立，1 条的 ① ② ③ 成立。checker 自己按字段表切一份，不与实现共用代码（D13（验证路线） 已定项 5；
 /// 实现那一侧是 `crates/singlefs-core/src/recovery.rs` 的 `tree_table_entries_each_kind_at_most_once` 与重建一版时缺了有根的树报的
 /// 「树表里缺一棵有根的树」）。③ ④ 先于 ② 判：同一张表两道都不成立时第一处违例写 ③ ④ 那一道。
@@ -3544,13 +3545,13 @@ fn judge_tree_table_entries_ordering(
     });
 }
 
-/// I-9.16 读者重建一版要的那四种有根的树（[`judge_tree_table_entry_kind_counts`] 的 ④）。
-const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 4] = [
-    TREE_KIND_EXTENT,
-    TREE_KIND_INODE,
-    TREE_KIND_ALLOCATION,
-    TREE_KIND_ACCOUNTING,
-];
+/// I-9.16 ④（[`judge_tree_table_entry_kind_counts`]）：树表非空时各要恰好一条的种类，就是进树表的那十一种
+/// （[`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`]，同一份，不另抄）。读者重建一版时十二棵树的号逐棵从树表取：
+/// 十一棵按种类找条目、取条目里的树 ID，中央映射树不进树表、取根记录里它那条根指针的出生树；缺哪一种都重建不出这一版
+/// （实现那一侧是 `crates/singlefs-core/src/recovery.rs` 的 `rebuild_version` 逐种取号时报的「树表里没有那棵树」）。
+/// 只读前四种根指针的那几行读法前提读窄了（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1 C1 那一行更正）。
+const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 11] =
+    TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER;
 
 /// I-9.16 的 ③ ④（[`judge_tree_table_entries_ordering`]）：③ 每一种种类码至多一条；④ 树表非空时
 /// [`TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES`] 各恰好一条。条目都读得出种类那 2 字节（调用方判过）。
@@ -3588,7 +3589,7 @@ fn judge_tree_table_entry_kind_counts(
     judgements.judge("I-9.16", first_rooted_kind_missing.is_none(), || {
         let kind = first_rooted_kind_missing.expect("判红时找到过那一种");
         format!(
-            "{version_name}的树表有 {} 条，种类 {kind} 一条都没有（④ 树表非空时种类 1、2、3、4 各恰好一条）",
+            "{version_name}的树表有 {} 条，种类 {kind} 一条都没有（④ 树表非空时进树表的十一种各恰好一条）",
             tree_table_entries.len()
         )
     });
@@ -4881,6 +4882,9 @@ fn judge_release_generation_and_tree_table_birth(
 /// （D22（单元原子性怎么合成） 已定项 7 字段表「分配记录树根指针」那一行；判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`
 /// 第二节 Y1 `allocation_record_tree_root` 那一行）。非零时这一版有两棵分配记录树，「这条根的分配记录树」不止一棵。
 /// 树表读不出的根这一格不判（认不出带不带树）。不变量清单里没有一条专管根记录这一项，暂挂在 I-5.4 下（归属交主 agent 定）。
+/// 这一格成立的那一次评估只在候选集里至少一棵分配记录树走到叶时计进 I-5.4（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md`
+/// 第二节 J1「I-5.4 挂法（C2）」那一行）：一棵都没走到时不计，I-5.4 报部分没判、逐版列出没计的那几版——否则带树一版上这一格成立一次就压过
+/// 不适用，分配记录树两份清零的镜像上 I-5.4 报成立。不成立照报违例（[`Judgements::partially_judged`] 的契约：判出违例照报违例）。
 fn judge_allocation_records_disjoint(
     reader: &dyn ImageReader,
     roots: &[(u64, u64, crate::RootView)],
@@ -4890,6 +4894,9 @@ fn judge_allocation_records_disjoint(
 ) {
     let mut seen_allocation_nodes: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
     let mut judged_allocation_nodes = 0u64;
+    // 带树一版「根记录里树表 0 条那一版的分配记录树根全零」那一格：不成立当场照报违例；成立的那几版按走到的次序记下哪一版，
+    // 走完全部候选根之后才定计不计进评估（至少一棵分配记录树走到叶才计）。
+    let mut versions_where_the_root_record_allocation_tree_root_is_zero: Vec<String> = Vec::new();
     for index in candidate_indexes.iter().copied() {
         let (_, _, root) = &roots[index];
         // 这条根的分配记录树：树表里种类 3 那一条指着的根，以及树表 0 条那一版由根记录直接持有的那一棵
@@ -4903,18 +4910,21 @@ fn judge_allocation_records_disjoint(
                 read_index_node_without_judging(reader, &tree_table_pointer, cache)
             {
                 if !tree_table.entries.is_empty() {
-                    judgements.judge(
-                        "I-5.4",
-                        allocation_roots[0].all_zero,
-                        || {
+                    if allocation_roots[0].all_zero {
+                        versions_where_the_root_record_allocation_tree_root_is_zero.push(format!(
+                            "根（实例 {}、txg {}）",
+                            root.instance, root.checkpoint_txg
+                        ));
+                    } else {
+                        judgements.judge("I-5.4", false, || {
                             format!(
                                 "根（实例 {}、txg {}）的树表有 {} 条（带树的一版），根记录里树表 0 条那一版的分配记录树根却不是全零：这一版有两棵分配记录树",
                                 root.instance,
                                 root.checkpoint_txg,
                                 tree_table.entries.len()
                             )
-                        },
-                    );
+                        });
+                    }
                 }
                 allocation_roots.extend(
                     tree_table
@@ -4949,11 +4959,25 @@ fn judge_allocation_records_disjoint(
             judged_allocation_nodes += 1;
         }
     }
-    if judged_allocation_nodes == 0 {
+    let some_allocation_record_tree_walked_to_its_leaves = judged_allocation_nodes > 0;
+    if some_allocation_record_tree_walked_to_its_leaves {
+        for _version in &versions_where_the_root_record_allocation_tree_root_is_zero {
+            judgements.judge("I-5.4", true, String::new);
+        }
+    } else {
         judgements.not_applicable(
             "I-5.4",
             "候选集里没有一条根的分配记录树走得到叶：第 0 代树表是空的，或树表 / 分配记录树读不出、位置对不上",
         );
+        if !versions_where_the_root_record_allocation_tree_root_is_zero.is_empty() {
+            judgements.partially_judged(
+                "I-5.4",
+                format!(
+                    "候选集里没有一条根的分配记录树走得到叶，记录之间互不相交一格都没判；带树一版根记录里分配记录树根全零那一格成立而没计：{}",
+                    versions_where_the_root_record_allocation_tree_root_is_zero.join("、")
+                ),
+            );
+        }
     }
 }
 
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index 20ecb76..9413449 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -93,11 +93,12 @@ use singlefs_harness::model_comparison::{
     refusal_reason_of_publish_error,
 };
 use singlefs_harness::mount_transparency::{
-    judge_mount_transparency, judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields,
-    ComparedVersion, DiskAfterTheRefusal, FieldOutcome, FloorRaisedByTheMount,
-    FloorRaisesInTheMountWrites, MountTransparency, MountTransparencyAction,
-    ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage, RecoveryFieldsRead,
-    RefusalConditionNotMet, RefusalReasonOnTheD23List,
+    judge_mount_transparency,
+    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount,
+    read_recovery_fields, ComparedVersion, DiskAfterTheRefusal, FieldOutcome,
+    FloorRaisedByTheMount, FloorRaisesInTheMountWrites, MountTransparency, MountTransparencyAction,
+    ReadFaultsInTheCrashState, ReadFaultsUnderTheRefusedMount, ReadOnlyReadBackOfTheCrashImage,
+    RecoveryFieldsRead, RefusalConditionNotMet, RefusalReasonOnTheD23List,
     UserVisibleTreeRootPointersOfTheContinuation, WritableMountRefusalVerdict,
 };
 use singlefs_harness::numeric_environment_variable::{
@@ -2541,6 +2542,12 @@ enum PublishAfterTheMountInTheState {
     NotWrittenYet,
 }
 
+/// 第二截、第三截被拒的那次挂载都起在崩溃镜像（第一次崩溃的、二次崩溃的）上，盘是不带故障层的内存盘：读到的就是镜像上的字节，
+/// 瞬时读错不会发生——崩溃态注入过读故障也一样（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2
+/// C3 / J2-a 那一行，改法 G3：判官恒交「否」）。
+const READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE: ReadFaultsUnderTheRefusedMount =
+    ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults;
+
 /// 判可写挂载被拒合不合法要的：崩溃后镜像（拒的判据在它上面复算，拒之后的池要与它逐字节相同）、它上面只读恢复落到的那一版、
 /// 那一次只读读出在不在模型的允许集合里（问模型的结论）。第二截交第一次崩溃的镜像与第一截的结论；第三截交二次崩溃镜像与那一个二次崩溃上的结论，
 /// 读故障的认定沿用第一次崩溃那个态的（第二截那次挂载起在第一次崩溃的镜像上，不注入读故障）。
@@ -2555,7 +2562,7 @@ struct RefusalOnTheCrashImage<'crash> {
 /// 第二截：挂载或那次发布被拒（单元区墙只计数，模型在单元区墙上只答允许拒绝的区间、而这里没有崩溃点上的模型状态可判区间），
 /// 做没做成都在之后的池上跑池级 checker；`record_check` 是同一个池上记录核对器的结论
 /// （[`HistoryThenWritableMountRecords::record_check_after_the_writable_mount`]）。挂载被拒而理由在 D23（journal 的角色与格式） 已定项 14
-/// 那张单子上的，判拒绝本身（`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state`：判据在崩溃镜像上复算为真、
+/// 那张单子上的，判拒绝本身（`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount`，被拒的那次挂载恒交不带故障层：判据在崩溃镜像上复算为真、
 /// 盘上逐字节不变、只读读出在允许集合里；崩溃态没注入读故障而判据 N-配置 复算为真的另判不合法，C670），合法的只计数，不合法的报红。别的理由被拒按「模型说该成、实现拒了」报：
 /// 模型对可写挂载与之后那一次发布（覆盖写活着的第一个文件、第一个文件、建 inode）从不要求拒，容量墙之外也不许拒（`IdealModel::answer_mount_writable`）。
 #[allow(
@@ -2585,13 +2592,14 @@ fn judge_the_writable_mount_on_the_crash_image(
                 } else {
                     DiskAfterTheRefusal::Changed
                 };
-            match judge_writable_mount_refusal_of_a_crash_state(
+            match judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
                 refusal_on_the_crash_image.crash_image_materialized,
                 refusal_on_the_crash_image.version_landed_on_after_the_crash,
                 refusal_error,
                 disk,
                 refusal_on_the_crash_image.read_only_read_back,
                 refusal_on_the_crash_image.read_faults,
+                READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE,
             ) {
                 WritableMountRefusalVerdict::Legal(_) => {
                     tally.writable_mount_refusals_on_the_d23_list_legal += 1;
@@ -2682,7 +2690,7 @@ const MOUNT_TRANSPARENCY_NOT_JUDGED_THE_PUBLISH_AFTER_THE_MOUNT_WAS_REFUSED: &st
 const MOUNT_TRANSPARENCY_NOT_JUDGED_THE_UNINTERRUPTED_MOUNT_WAS_REFUSED: &str =
     "没崩过的那一次可写挂载被拒";
 /// 第一次崩溃那个态注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：挂载透明的射程只管不带故障的状态
-/// （D13（验证路线） 已定项 1），崩溃镜像上那一次挂载整个不判。
+/// （D13（验证路线） 已定项 1），崩溃镜像上那一次挂载整个不判，它途中的每个二次崩溃也不判（各记一次）。
 const MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE: &str =
     "崩溃态注入过读故障（挂载透明只管不带故障的状态）";
 
@@ -3238,7 +3246,8 @@ struct SecondCrashInputs<'history> {
         Option<&'history RowsDeletedByTheInstanceTableSweepPublish>,
     /// RefFS 在第一次崩溃上的判定：判绿时带着以读出的内容重新起头的 RefFS，二次崩溃接着它判。
     reference_filesystem_at_the_first_crash: &'history ReferenceFileSystemJudgementOfACrash,
-    /// 第一次崩溃那个态注没注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：二次崩溃之后再挂被拒时判 C670 那一条沿用它。
+    /// 第一次崩溃那个态注没注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：二次崩溃之后再挂被拒时判 C670 那一条沿用它；
+    /// 注入过的，二次崩溃镜像上的挂载透明照第二截跳过（改法 G5）。
     read_faults_in_the_first_crash_state: ReadFaultsInTheCrashState,
     /// 二次崩溃怎么摆：挂载每一段按种子各摆一个，或靶向用例写死的那一个。
     second_crash_draw: SecondCrashPointDraw,
@@ -3400,16 +3409,30 @@ fn judge_second_crashes_inside_the_writable_mount(
         tally.second_crash_checker_runs += 1;
         let mut violations = checker_violations_of(&image);
         violations.extend(reference_filesystem_vote.violation());
-        violations.extend(mount_transparency_violations_of_a_second_crash(
-            tally,
-            fields_read_after_the_first_crash,
-            &fields_read_after_the_second_crash,
-            floor_raises_in_the_uninterrupted_mount_writes,
-            continuation_of_the_uninterrupted_mount,
-            rows_deleted_by_the_sweep_of_the_uninterrupted_mount,
-            &persisted_in_the_mount,
-            second.phase,
-        ));
+        // 挂载透明的射程只管不带故障的状态（D13（验证路线） 已定项 1）：第一次崩溃那个态注入过读故障的，它上面那次挂载途中的二次崩溃
+        // 也不判，与第二截同一个理由记一次没判（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2
+        // 「二次崩溃镜像在注入态下照判（J2-d）」那一行，改法 G5）。
+        violations.extend(match read_faults_in_the_first_crash_state {
+            ReadFaultsInTheCrashState::NoneInjected => {
+                mount_transparency_violations_of_a_second_crash(
+                    tally,
+                    fields_read_after_the_first_crash,
+                    &fields_read_after_the_second_crash,
+                    floor_raises_in_the_uninterrupted_mount_writes,
+                    continuation_of_the_uninterrupted_mount,
+                    rows_deleted_by_the_sweep_of_the_uninterrupted_mount,
+                    &persisted_in_the_mount,
+                    second.phase,
+                )
+            }
+            ReadFaultsInTheCrashState::Injected => {
+                *tally
+                    .mount_transparency_not_judged
+                    .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE)
+                    .or_insert(0) += 1;
+                Vec::new()
+            }
+        });
         if violations.is_empty() && disagreement.is_none() && record_check == RecordCheck::default()
         {
             continue;
@@ -3594,14 +3617,16 @@ fn judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion(
     } else {
         DiskAfterTheRefusal::Changed
     };
-    let verdict = judge_writable_mount_refusal_of_a_crash_state(
-        second_crash_image.crash_image_materialized,
-        second_crash_image.version_landed_on_after_the_crash,
-        refusal,
-        disk,
-        second_crash_image.read_only_read_back,
-        second_crash_image.read_faults,
-    );
+    let verdict =
+        judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
+            second_crash_image.crash_image_materialized,
+            second_crash_image.version_landed_on_after_the_crash,
+            refusal,
+            disk,
+            second_crash_image.read_only_read_back,
+            second_crash_image.read_faults,
+            READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE,
+        );
     let Some(conditions_not_met) =
         conditions_not_met_by_a_remount_refusal_with_the_witness_criterion_true(tally, verdict)
     else {
@@ -4994,6 +5019,71 @@ mod tests {
         );
     }
 
+    /// 二次崩溃的种子随历史种子的每一位都变（攻方取样 C3，判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2
+    /// 「F3-e 取样（C4 / J2-f）」那一行与第三节第 1 条、改法 G6；原型 `research/prompts/m3-formal-code-b26-r1-opus-model/probe/src/f3.rs` 的
+    /// `F3PROPOSED`）：段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0 与 2^k（k = 0..=63）共 65 个种子上各算一遍，266175 个种子两两不同。
+    /// 上面 0、1、2^63 那一条只看得见最低位与最高位，历史种子对 3 取余、只留低 8 位与最高位的种子在它上面照绿（攻方量过，`research/prompts/m3-formal-code-b26-r1-opus-model/out/f3.txt`）。
+    /// b24 已是最后一轮，只加宽取样：这一组取样对别的种子形状罩不罩得住没人攻过，剩下的漏形记在 b24 那笔欠账里。
+    /// 判别力：历史种子对 3 取余（`crates/mutations.tsv` 一行）时 2^k 与 0、1、2 撞，这一条红。
+    #[test]
+    fn first_crash_states_under_history_seeds_zero_and_every_power_of_two_seed_their_second_crashes_differently(
+    ) {
+        let history_seeds: Vec<HistorySeed> = std::iter::once(HistorySeed(0))
+            .chain((0..64).map(|bit| HistorySeed(1_u64 << bit)))
+            .collect();
+        let seeds: Vec<SecondCrashPointSeed> = history_seeds
+            .iter()
+            .flat_map(|seed| {
+                seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
+                    *seed, 16, 0,
+                )
+            })
+            .collect();
+        assert_eq!(
+            seeds.len(),
+            65 * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
+            "前提：历史种子 0 与 2^0..=2^63 共 65 个，各 4095 个第一次崩溃态"
+        );
+        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
+        assert_eq!(
+            distinct_seeds.len(),
+            seeds.len(),
+            "段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0 与 2^k（k = 0..=63）上算出的 266175 个二次崩溃种子两两不同"
+        );
+    }
+
+    /// 长 1024 的段里扣下的那一个写落在任何位置都折进种子（攻方取样 B4，同上判决、改法 G6 与原型）：历史种子 0、段号 16、长 1024 的段，
+    /// 扣下其中一个写（1024 种），1024 个第一次崩溃态的种子两两不同。上面长 32 的那一条看不见第 32 个写之后：指纹只折段内前 32 个或末 32 个写的
+    /// 在它上面照绿（攻方量过，`research/prompts/m3-formal-code-b26-r1-opus-model/out/f3.txt`）。
+    /// b24 已是最后一轮，只加宽取样：段长 1024、只扣一个写的取样对更长的段、扣得更多的子集罩不罩得住没人攻过，剩下的漏形记在 b24 那笔欠账里。
+    /// 判别力：指纹只折段内前 32 个写（`crates/mutations.tsv` 一行）时扣下第 32 个写之后哪一个都撞成一个种子，这一条红。
+    #[test]
+    fn first_crash_states_of_a_segment_of_one_thousand_twenty_four_writes_withholding_one_write_anywhere_seed_their_second_crashes_differently(
+    ) {
+        const WRITES_OF_THE_SEGMENT: usize = 1024;
+        let seeds: Vec<SecondCrashPointSeed> = (0..WRITES_OF_THE_SEGMENT)
+            .map(|withheld| {
+                seed_of_the_second_crash_points(
+                    HistorySeed(0),
+                    &CrashPoint {
+                        segment_index: 16,
+                        persisted_within_the_segment: (0..WRITES_OF_THE_SEGMENT)
+                            .map(|write_within_the_segment| write_within_the_segment != withheld)
+                            .collect(),
+                        step: StepPosition::Operation(0),
+                        operation_kind: Some(HistoryOperationKind::PublishOverwrite),
+                    },
+                )
+            })
+            .collect();
+        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
+        assert_eq!(
+            distinct_seeds.len(),
+            WRITES_OF_THE_SEGMENT,
+            "历史种子 0、段号 16、长 1024 的段上扣下一个写的 1024 个第一次崩溃态算出的二次崩溃种子两两不同"
+        );
+    }
+
     /// 实例表清扫两段（D18（块里携带什么信息） 已定项 11「回收」一条）：挂载那一段的写里，第「清扫之前发了几次」个根槽写之后起是清扫抹头
     /// （FUA 写零的单元头、每块盘一道屏障），之后第一个不带 FUA 的单元写起是清扫发布（单元、屏障、记录、根槽、轮换），两段摆二次崩溃时各摆到一个。
     /// 造的写表：取号；写行那次（单元、记录、根槽、轮换）；一次暖机（单元、根槽）；抹两个头、两道屏障；清扫发布——清扫之前发了 2 次。
@@ -6798,18 +6888,10 @@ mod tests {
         );
     }
 
-    /// C696「第三截调用点写死成 NoneInjected」那一种接错的靶向用例（用户定「步表加一条靶向用例」；W 为真的阳性对照，造法同
-    /// [`a_second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site`]）：第一次崩溃摆在
-    /// 「每块盘最新那一槽系统配置读回全 0 的可写挂载」之后那次覆盖写的第一个写上（注入过读故障的态），第二截那次可写挂载的写行那次发布上摆一个
-    /// 二次崩溃——根槽 FUA 没落、紧跟着的两槽系统配置轮换落了。二次崩溃镜像上再挂以判据 N-配置 拒、判据复算为真；崩溃态注入过读故障，
-    /// 第三截判过一次、不判 C670 那一条（不合法 0 次、没有新发现）。
-    /// 判别力：第三截调用点把读故障的认定写死成 NoneInjected（`crates/mutations.tsv` 一行）时不合法变 1、报一条新发现，这一条红。
-    /// 随机历史上那一种接错是等价变异（设计调查报告第三节：注入态与第三截二次崩溃态上判据从没为真），等价性留档在
-    /// `crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs`。
-    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
-    #[test]
-    fn a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount_is_not_judged_by_c670(
-    ) {
+    /// 「每块盘最新那一槽系统配置读回全 0 的可写挂载」之后那次覆盖写的第一个写上摆第一次崩溃（注入过读故障的态），第二截那次可写挂载的
+    /// 写行那次发布上摆一个二次崩溃——根槽 FUA 没落、紧跟着的两槽系统配置轮换落了；交回这一趟崩溃注入。两条靶向用例共用这份搭建，各自断言。
+    fn injection_with_a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount(
+    ) -> HistoryCrashInjection {
         let history = history_of_a_newest_system_configuration_slot_zeros_mount_and_one_overwrite();
         let execution = execution_on_384_slot_devices_without_the_per_step_checker();
         let (writes, segments, first_write_of_the_overwrite) =
@@ -6870,7 +6952,7 @@ mod tests {
             segment_containing_the_write(&mount_segments, root_of_the_row_publish);
         let first_write_of_the_segment_of_the_root = mount_segments[segment_of_the_root][0];
         let last_rotation = rotation_after_the_root[rotation_after_the_root.len() - 1];
-        let injection = inject_crashes_into_history_drawing_second_crashes(
+        inject_crashes_into_history_drawing_second_crashes(
             &history,
             execution,
             CrashPointDraw::ExactlyOneCrashState {
@@ -6888,7 +6970,23 @@ mod tests {
                 .leak(),
             },
             None,
-        );
+        )
+    }
+
+    /// C696「第三截调用点写死成 NoneInjected」那一种接错的靶向用例（用户定「步表加一条靶向用例」；W 为真的阳性对照，造法同
+    /// [`a_second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site`]）：第一次崩溃摆在
+    /// 「每块盘最新那一槽系统配置读回全 0 的可写挂载」之后那次覆盖写的第一个写上（注入过读故障的态），第二截那次可写挂载的写行那次发布上摆一个
+    /// 二次崩溃——根槽 FUA 没落、紧跟着的两槽系统配置轮换落了。二次崩溃镜像上再挂以判据 N-配置 拒、判据复算为真；崩溃态注入过读故障，
+    /// 第三截判过一次、不判 C670 那一条（不合法 0 次、没有新发现）。
+    /// 判别力：第三截调用点把读故障的认定写死成 NoneInjected（`crates/mutations.tsv` 一行）时不合法变 1、报一条新发现，这一条红。
+    /// 随机历史上那一种接错是等价变异（设计调查报告第三节：注入态与第三截二次崩溃态上判据从没为真），等价性留档在
+    /// `crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs`。
+    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
+    #[test]
+    fn a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount_is_not_judged_by_c670(
+    ) {
+        let injection =
+            injection_with_a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount();
         let tally = &injection.tally;
         assert_eq!(
             (
@@ -6927,6 +7025,43 @@ mod tests {
         );
     }
 
+    /// 改法 G5（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2「二次崩溃镜像在注入态下照判（J2-d）」那一行，
+    /// D13（验证路线） 已定项 1「带故障的状态不在射程」）：同上那一份搭建——第一次崩溃那个态注入过读故障，它上面那次可写挂载做成，写行里摆一个
+    /// 二次崩溃。第二截那一次挂载透明不判；那个二次崩溃镜像上的挂载透明也不判，与第二截同一个理由再记一次（共 2 次），按段判过的一次都没有。
+    /// 判别力：二次崩溃那一截不看读故障标记、照判挂载透明（`crates/mutations.tsv` 一行）时按段判过的有写行 1 次、没判的只记 1 次，这一条红。
+    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
+    #[test]
+    fn mount_transparency_is_not_judged_on_a_second_crash_inside_the_mount_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount(
+    ) {
+        let injection =
+            injection_with_a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount();
+        let tally = &injection.tally;
+        assert_eq!(
+            (
+                tally.crash_points_after_a_step_reading_under_injected_faults,
+                tally.writable_mounts_after_the_crash_succeeded,
+                tally.second_crash_points
+            ),
+            (1, 1, 1),
+            "前提：只摆写死的那一个第一次崩溃态（注入过读故障，它上面的可写挂载做成）与那一个二次崩溃：{}",
+            tally.render()
+        );
+        assert_eq!(
+            (
+                tally
+                    .mount_transparency_judged_on_second_crashes_by_phase
+                    .clone(),
+                tally
+                    .mount_transparency_not_judged
+                    .get(MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE)
+                    .copied()
+            ),
+            (BTreeMap::new(), Some(2)),
+            "注入过读故障的崩溃态上，那次挂载与它途中的二次崩溃都不判挂载透明，按理由各记一次：{}",
+            tally.render()
+        );
+    }
+
     /// 起点（取号、暖机、第一个文件）之后一次覆盖写、不重挂的历史，与跑它用的执行（单元区 384 槽的小盘，不逐步跑 checker）。
     fn history_of_the_first_file_and_one_overwrite() -> (GeneratedHistory, HistoryExecution) {
         use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
index d87a6cf..f7878c2 100644
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -35,9 +35,7 @@ use singlefs_core::recovery::{
     JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
 };
 use singlefs_core::root_record::{RootRecord, UnmountMarker};
-use singlefs_core::system_configuration::{
-    SystemConfiguration, SystemImmutableConfiguration, SystemRuntimeConfiguration,
-};
+use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableConfiguration};
 use singlefs_core::transaction::TransactionOutput;
 use singlefs_format::{
     SYSTEM_CONFIGURATION_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
@@ -1352,17 +1350,32 @@ fn byte_ranges_that_do_not_hold<'table>(
         .collect()
 }
 
-/// 系统运行配置那 36 字节的内置默认值：照写槽的同一个次序（T_time 4、T_dirty 8、整理三水位 8 × 3，小端），值取
-/// `SystemRuntimeConfiguration` 交的默认值（今天没有挂载选项通道，写槽总写它们，`system_configuration.rs` 那一档的说明）。
+/// T_time 的内置默认值，单位秒（D16（发布语义） 已定项 5「`T_time` 默认值 5 s」；`.claude/kb/layout/01-first-txn.md` 系统配置字段表
+/// 「可调值 | T_time | 4 | 5 秒」那一行）。照条款写死，不取被测 core 的常量：期望取自被测者时，core 把默认值改错，挂载写槽写的与期望一起错，
+/// 这一行判不出（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「运行配置『R2 = 内置默认』」那一行，改法 G1）。
+const BUILT_IN_TIME_THRESHOLD_SECONDS_OF_THE_CLAUSES: u32 = 5;
+/// T_dirty 的内置默认值，单位字节：2 GiB（D16（发布语义） 已定项 5；字段表「可调值 | T_dirty | 8 | 2 GiB」那一行。挂载时的有效值另按环长夹取，
+/// 不改盘上这 8 字节）。
+const BUILT_IN_DIRTY_THRESHOLD_BYTES_OF_THE_CLAUSES: u64 = 2 * 1024 * 1024 * 1024;
+/// 整理低水位 / 高水位 / 停止线的内置默认值：0 = 内置默认（字段表「可调值 | 整理低水位 / 高水位 / 停止线 | 8 × 3 | 0 = 内置默认」那一行，
+/// D26（后台整理与放置回收） 已定项 1）。
+const BUILT_IN_COMPACTION_WATERMARKS_OF_THE_CLAUSES: [u64; 3] = [0, 0, 0];
+
+/// 系统运行配置那 36 字节的内置默认值：照字段表的次序（T_time 4、T_dirty 8、整理三水位 8 × 3，小端），值照条款写死
+/// （[`BUILT_IN_TIME_THRESHOLD_SECONDS_OF_THE_CLAUSES`] 那三个常量；今天没有挂载选项通道，写槽总写内置默认）。
 #[must_use]
 pub fn built_in_runtime_configuration_bytes() -> Vec<u8> {
-    let defaults = SystemRuntimeConfiguration;
     let mut bytes = Vec::with_capacity(SYSTEM_CONFIGURATION_RUNTIME_CONFIGURATION.len());
-    bytes.extend_from_slice(&defaults.time_threshold_seconds().to_le_bytes());
-    bytes.extend_from_slice(&defaults.dirty_threshold_bytes().to_le_bytes());
-    for watermark in defaults.compaction_watermarks() {
+    bytes.extend_from_slice(&BUILT_IN_TIME_THRESHOLD_SECONDS_OF_THE_CLAUSES.to_le_bytes());
+    bytes.extend_from_slice(&BUILT_IN_DIRTY_THRESHOLD_BYTES_OF_THE_CLAUSES.to_le_bytes());
+    for watermark in BUILT_IN_COMPACTION_WATERMARKS_OF_THE_CLAUSES {
         bytes.extend_from_slice(&watermark.to_le_bytes());
     }
+    assert_eq!(
+        bytes.len(),
+        SYSTEM_CONFIGURATION_RUNTIME_CONFIGURATION.len(),
+        "系统运行配置一档 36 字节：T_time 4 + T_dirty 8 + 整理三水位 8 × 3"
+    );
     bytes
 }
 
@@ -2525,6 +2538,30 @@ pub enum ReadFaultsInTheCrashState {
     Injected,
 }
 
+/// 被拒的这次可写挂载自己在不在读故障下跑（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2
+/// 「注入读故障的态上『新拒因复算为假不算不合法』（C3 / J2-a）」那一行，改法 G3）。与 [`ReadFaultsInTheCrashState`] 是两件事：
+/// 那一样说造出崩溃镜像的历史里有没有读故障步，这一样说被拒的那次挂载读盘时有没有故障层——崩溃注入第二、三截那次挂载起在崩溃镜像上、
+/// 盘是不带故障层的内存盘，历史里注入过读故障也不会在这次挂载里出瞬时读错。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ReadFaultsUnderTheRefusedMount {
+    /// 被拒的那次挂载读的就是崩溃镜像上的字节、不带故障层（崩溃注入第二、三截，E171 装置）：瞬时读错不会发生，
+    /// 拒的判据在崩溃镜像上复算为假就是拒错了。
+    TheRefusedMountReadTheCrashImageWithoutFaults,
+    /// 被拒的那次挂载的读经故障层、读错可以是瞬时的：两遍读之间的瞬时读错在崩溃镜像上复算不出
+    /// （C664（挂载拒绝合法复算把两种拒因报成一个）判别力自证里取号那一刻瞬时读错的态）。
+    TheRefusedMountReadThroughInjectedFaults,
+}
+
+impl ReadFaultsUnderTheRefusedMount {
+    /// 「读阶段判完之后系统配置又读不出」那一拒的判据在崩溃镜像上复算为假，能不能是被拒那次挂载里一次瞬时读错（复算不出）。
+    fn can_hide_a_transient_read_error_from_the_crash_image(self) -> bool {
+        match self {
+            Self::TheRefusedMountReadTheCrashImageWithoutFaults => false,
+            Self::TheRefusedMountReadThroughInjectedFaults => true,
+        }
+    }
+}
+
 /// 单子上的理由，合法还差哪一条。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub enum RefusalConditionNotMet {
@@ -2576,19 +2613,44 @@ pub fn judge_writable_mount_refusal(
     )
 }
 
+/// 同 [`judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount`]，被拒的那次挂载读的就是崩溃镜像、
+/// 不带故障层（[`ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults`]：在崩溃镜像上起一次挂载判它的调用方，
+/// 如 E171 装置）。
+#[must_use]
+pub fn judge_writable_mount_refusal_of_a_crash_state(
+    crash_image: &dyn PoolReader,
+    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
+    refusal: &MountError,
+    disk: DiskAfterTheRefusal,
+    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
+    read_faults: ReadFaultsInTheCrashState,
+) -> WritableMountRefusalVerdict {
+    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
+        crash_image,
+        selected_version,
+        refusal,
+        disk,
+        read_only_read_back,
+        read_faults,
+        ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults,
+    )
+}
+
 /// 判崩溃注入第二截的一次拒可写合不合法：U6 那四条（[`judge_writable_mount_refusal`]），再加 C670 那一条——崩溃态没注入读故障
 /// （`read_faults`）、理由是判据 N-配置，而判据在崩溃镜像上复算、撕裂的系统配置槽跳过之后仍为真（[`WitnessedCounterBranchThatHolds`]），
 /// 拒得不合法（[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]）。注入过读故障的崩溃态不判这一条
-/// （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。理由是「读阶段判完之后系统配置又读不出」而崩溃态注入过读故障的，
-/// 判据在崩溃镜像上复算为假不算不合法（两遍读之间的瞬时读错复算不出，C664（挂载拒绝合法复算把两种拒因报成一个））。
+/// （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。理由是「读阶段判完之后系统配置又读不出」而被拒的那次挂载自己在读故障下跑的
+/// （`read_faults_under_the_refused_mount`），判据在崩溃镜像上复算为假不算不合法（两遍读之间的瞬时读错复算不出，C664（挂载拒绝合法复算把两种拒因
+/// 报成一个））；被拒的那次挂载不带故障层的，崩溃态注没注入过读故障都照 U6 复算（判据为假即不合法，改法 G3）。
 #[must_use]
-pub fn judge_writable_mount_refusal_of_a_crash_state(
+pub fn judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
     crash_image: &dyn PoolReader,
     selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
     refusal: &MountError,
     disk: DiskAfterTheRefusal,
     read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
     read_faults: ReadFaultsInTheCrashState,
+    read_faults_under_the_refused_mount: ReadFaultsUnderTheRefusedMount,
 ) -> WritableMountRefusalVerdict {
     let Some(reason) = refusal_reason_on_the_d23_list(refusal) else {
         return WritableMountRefusalVerdict::ReasonNotOnTheD23List;
@@ -2616,28 +2678,33 @@ pub fn judge_writable_mount_refusal_of_a_crash_state(
                 );
             }
         }
-        // 读阶段之后系统配置又读不出、崩溃态注入过读故障：两遍读之间的瞬时读错在崩溃镜像上复算不出，判据复算为假不算拒得不合法
-        // （C664（挂载拒绝合法复算把两种拒因报成一个）的还清条件：带读故障的档里取号那一刻瞬时读错的态，分开报之后判合法）；
-        // 盘上逐字节不变与只读读出照判。没注入读故障的照 U6 复算（判据为假即不合法）。
-        (
-            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
-            ReadFaultsInTheCrashState::Injected,
-        ) => conditions_not_met.retain(|condition| {
-            *condition != RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
-        }),
         (
             RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
             ReadFaultsInTheCrashState::Injected,
         )
         | (
-            RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
+            RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable
+            | RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
             ReadFaultsInTheCrashState::NoneInjected | ReadFaultsInTheCrashState::Injected,
-        )
-        | (
-            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
-            ReadFaultsInTheCrashState::NoneInjected,
         ) => {}
     }
+    // 读阶段之后系统配置又读不出、被拒的那次挂载自己在读故障下跑：两遍读之间的瞬时读错在崩溃镜像上复算不出，判据复算为假不算拒得不合法
+    // （C664（挂载拒绝合法复算把两种拒因报成一个）的还清条件：带读故障的档里取号那一刻瞬时读错的态，分开报之后判合法）；
+    // 盘上逐字节不变与只读读出照判。被拒的那次挂载不带故障层的照 U6 复算（判据为假即不合法）。
+    match reason {
+        RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage => {
+            if read_faults_under_the_refused_mount
+                .can_hide_a_transient_read_error_from_the_crash_image()
+            {
+                conditions_not_met.retain(|condition| {
+                    *condition
+                        != RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+                });
+            }
+        }
+        RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
+        | RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable => {}
+    }
     verdict_of_a_refusal_on_the_d23_list(reason, conditions_not_met)
 }
 
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 463ce91..caf128f 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -8203,7 +8203,7 @@ fn a_tree_table_whose_allocation_entry_carries_an_unregistered_kind_reddens_only
     assert_only_the_tree_table_ordering_invariant_is_violated(
         &clean,
         &image,
-        "根（实例 1、txg 3）的树表有 11 条，种类 3 一条都没有（④ 树表非空时种类 1、2、3、4 各恰好一条）",
+        "根（实例 1、txg 3）的树表有 11 条，种类 3 一条都没有（④ 树表非空时进树表的十一种各恰好一条）",
     );
 }
 
@@ -8257,7 +8257,85 @@ fn a_tree_table_without_its_allocation_entry_reddens_only_the_tree_table_orderin
     assert_only_the_tree_table_ordering_invariant_is_violated(
         &clean,
         &image,
-        "根（实例 1、txg 3）的树表有 10 条，种类 3 一条都没有（④ 树表非空时种类 1、2、3、4 各恰好一条）",
+        "根（实例 1、txg 3）的树表有 10 条，种类 3 一条都没有（④ 树表非空时进树表的十一种各恰好一条）",
+    );
+}
+
+/// 树表里去掉第 `index` 条，先核那一条原来的种类是 `kind`。
+fn remove_tree_table_entry_of_kind(tree_table: &mut [u8], index: usize, kind: u16) {
+    let kind_offset = tree_table_entry_offset(index) + TREE_TABLE_ENTRY_KIND_OFFSET;
+    assert_eq!(
+        u16::from_le_bytes([tree_table[kind_offset], tree_table[kind_offset + 1]]),
+        kind,
+        "树表第 {index} 条原来的种类"
+    );
+    remove_tree_table_entry(tree_table, index);
+}
+
+/// I-9.16 ④ 十一种（正推 C1，判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-9.16 ④ 放过缺种类的树表（C1）」
+/// 那一行）：整条去掉 deadlist 那一条——树表 10 条、没有种类 8。改之前只要种类 1–4 时整份 checker 一条不红，core 的可写挂载重建这一版时
+/// 逐种取号报「树表里没有那棵树」（正推探针 `research/prompts/m3-formal-code-b26-r1-sonnet-model/probe/known_bad_probe.rs.append`）。
+#[test]
+fn a_tree_table_without_its_deadlist_entry_reddens_only_the_tree_table_ordering_invariant() {
+    let clean = build_pool("known-bad-tree-table-without-the-deadlist-entry").memory_pool();
+    let mut image = clean.clone();
+    mutate_unit(
+        &mut image,
+        TREE_TABLE,
+        |bytes| {
+            remove_tree_table_entry_of_kind(
+                bytes,
+                DEADLIST_TREE_TABLE_ENTRY_INDEX,
+                TREE_KIND_DEADLIST,
+            );
+        },
+        true,
+    );
+    assert_only_the_tree_table_ordering_invariant_is_violated(
+        &clean,
+        &image,
+        "根（实例 1、txg 3）的树表有 10 条，种类 8 一条都没有（④ 树表非空时进树表的十一种各恰好一条）",
+    );
+}
+
+/// I-9.16 ④ 十一种（同上判决与探针）：整条去掉 logged_ops 那一条（树表最末一条）——树表 10 条、没有种类 12。走读另红 I-1.1，照实钉住
+/// （正推探针改之前量到 `violated=["I-1.1"]`、I-9.16 成立：`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/probe-i916-missing-kind.txt`）。
+#[test]
+fn a_tree_table_without_its_logged_operations_entry_reddens_the_tree_table_ordering_invariant_besides_the_walk(
+) {
+    let clean =
+        build_pool("known-bad-tree-table-without-the-logged-operations-entry").memory_pool();
+    assert_eq!(
+        verdict(&clean, "I-9.16"),
+        InvariantVerdict::Holds,
+        "干净镜像上 I-9.16 真被评估过且成立"
+    );
+    let mut image = clean.clone();
+    mutate_unit(
+        &mut image,
+        TREE_TABLE,
+        |bytes| {
+            remove_tree_table_entry_of_kind(
+                bytes,
+                LOGGED_OPERATIONS_TREE_TABLE_ENTRY_INDEX,
+                TREE_KIND_LOGGED_OPERATIONS,
+            );
+        },
+        true,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-1.1", "I-9.16"],
+        "判红的是 I-9.16 加走读带出来的 I-1.1：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict(&image, "I-9.16"),
+        InvariantVerdict::Violated(
+            "根（实例 1、txg 3）的树表有 10 条，种类 12 一条都没有（④ 树表非空时进树表的十一种各恰好一条）"
+                .to_owned()
+        ),
+        "红在 ④ 那一道"
     );
 }
 
@@ -8366,6 +8444,31 @@ fn an_allocation_record_tree_that_does_not_read_back_leaves_the_first_data_unit_
     );
 }
 
+/// I-5.4 挂法（正推 C2，判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-5.4 挂法（C2）」那一行）：
+/// 分配记录树根两份全零——候选集里一棵分配记录树都走不到叶，带树一版「根记录里分配记录树根全零」那一格成立而不计进评估，I-5.4 报部分没判、
+/// 点名没计的那一版（改之前那一格成立一次就压过不适用，报成立：正推探针 `research/prompts/m3-formal-code-b26-r1-sonnet-model/out/probe-i54-new.txt`）。
+/// 判红的照旧是读不出的单元那几条。
+#[test]
+fn an_allocation_record_tree_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
+) {
+    let mut image = build_pool("known-bad-disjointness-allocation-root-zeroed").memory_pool();
+    zero_both_copies_of_the_unit(&mut image, ALLOCATION_RECORD_TREE_ROOT, 16384);
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED,
+        "判红的照改之前那几条：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict(&image, "I-5.4"),
+        InvariantVerdict::PartiallyJudged(
+            "候选集里没有一条根的分配记录树走得到叶，记录之间互不相交一格都没判；带树一版根记录里分配记录树根全零那一格成立而没计：根（实例 1、txg 3）"
+                .to_owned()
+        ),
+        "一棵分配记录树都没走到叶：I-5.4 不报成立，报部分没判"
+    );
+}
+
 /// I-5.5（攻方 d1）：第一个文件的数据指针第二条位置条目从盘 1 改到盘 0——盘 1 上一个数据单元都没有，该盘跳过、不记，
 /// 盘 0 上照判、成立；位置条目不按设备升序，I-2.5 照旧红。
 #[test]
diff --git a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
index e798500..22fce8f 100644
--- a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
+++ b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
@@ -1566,6 +1566,90 @@ fn a_journal_tail_past_every_record_in_the_ring_of_a_second_crash_image_is_red()
     );
 }
 
+/// 那一份择到的系统配置里的 journal tail（择得到系统配置）。
+fn journal_tail_of(fields: &RecoveryFieldsRead) -> u64 {
+    let bytes = fields
+        .chosen_system_configuration_bytes
+        .as_ref()
+        .expect("择得到系统配置");
+    u64::from_le_bytes(
+        bytes[SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET
+            ..SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET + 8]
+            .try_into()
+            .expect("切的是 8 字节"),
+    )
+}
+
+fn set_journal_tail_of(fields: &mut RecoveryFieldsRead, journal_tail: u64) {
+    chosen_system_configuration_bytes_mut(fields)
+        [SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET..SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET + 8]
+        .copy_from_slice(&journal_tail.to_le_bytes());
+}
+
+/// journal_tail 那一行「R1 ≤ R2」那一半（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1
+/// 「journal_tail、journal 实例代号两行」那一行：两半各补一条会红的用例）：取号之后的二次崩溃上（不判取等），R2 的 tail 比 R1 的小一，
+/// 仍不超过 R2 环里自证记录的最大计数器——只有下界那一半不成立。
+/// 判别力：去掉「R1 ≤ R2」那一半（`crates/mutations.tsv` 一行）时这一条红。
+#[test]
+fn a_journal_tail_below_the_one_on_the_crash_image_after_a_second_crash_is_red() {
+    let mut scenario = second_crash_after_the_acquisition(&pool_after_the_first_file());
+    let tail_on_the_crash_image = journal_tail_of(&scenario.before_the_mount);
+    assert!(
+        tail_on_the_crash_image >= 1,
+        "前提：第一个文件之后的崩溃镜像上 tail 至少 1（这一份是 {tail_on_the_crash_image}）"
+    );
+    assert!(
+        tail_on_the_crash_image - 1
+            <= scenario
+                .after_the_action
+                .highest_record_counter_in_the_ring
+                .unwrap_or(0),
+        "前提：R2 的 tail 改小之后仍不超过 R2 环里自证记录的最大计数器（上界那一半照旧成立）"
+    );
+    set_journal_tail_of(&mut scenario.after_the_action, tail_on_the_crash_image - 1);
+    assert_only_this_field_is_violated(
+        &scenario.judge(),
+        MountTransparencyField::SystemConfigurationJournalTail,
+    );
+}
+
+/// journal_tail 那一行「挂载做完取等」那一半（同上判决）：挂载加一次发布之后，R2 的 tail 比 R2 环里自证记录的最大计数器小一，
+/// 仍不小于 R1 的——只有取等那一半不成立。
+/// 判别力：挂载做完时不判取等（`crates/mutations.tsv` 一行）时这一条红。
+#[test]
+fn a_journal_tail_below_the_highest_record_counter_in_the_ring_after_a_mount_and_one_publish_is_red(
+) {
+    let mut scenario = mount_and_one_publish_on(SMALL, &pool_after_the_first_file());
+    let highest_record_counter = scenario
+        .after_the_action
+        .highest_record_counter_in_the_ring
+        .expect("挂载加一次发布之后环里有记录");
+    assert!(
+        journal_tail_of(&scenario.before_the_mount) < highest_record_counter,
+        "前提：R1 的 tail {} 比 R2 环里自证记录的最大计数器 {highest_record_counter} 小（改小一之后下界那一半照旧成立）",
+        journal_tail_of(&scenario.before_the_mount)
+    );
+    set_journal_tail_of(&mut scenario.after_the_action, highest_record_counter - 1);
+    assert_only_this_field_is_violated(
+        &scenario.judge(),
+        MountTransparencyField::SystemConfigurationJournalTail,
+    );
+}
+
+/// 系统运行配置那一行「R2 = 内置默认」（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「运行配置『R2 = 内置默认』」
+/// 那一行，改法 G1）：带文件的一版上只挂载，判官拿 R2 择到的系统配置 433..469 那 36 字节比照条款写死的内置默认（T_time 5 秒、T_dirty 2 GiB、
+/// 整理三水位各 0，D16（发布语义） 已定项 5；`.claude/kb/layout/01-first-txn.md` 系统配置字段表「可调值」那三行），这一样判绿。
+/// 判别力：core 的 T_time 内置默认改成 4（攻方 MUT-A，`crates/mutations.tsv` 一行）时这一条红——判官的期望取自被测 core 时，
+/// 那一改坏下挂载透明 45 条全绿（同一判决同一行）。
+#[test]
+fn the_runtime_configuration_after_a_mount_alone_is_the_built_in_default_written_in_the_clauses() {
+    let scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
+    assert_holds(
+        &scenario.judge(),
+        MountTransparencyField::SystemRuntimeConfiguration,
+    );
+}
+
 #[test]
 fn the_journal_instance_of_the_system_configuration_after_a_mount_alone_is_the_instance_of_its_root(
 ) {
diff --git a/crates/singlefs-harness/tests/writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion.rs b/crates/singlefs-harness/tests/writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion.rs
index 244b3d6..62e039b 100644
--- a/crates/singlefs-harness/tests/writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion.rs
+++ b/crates/singlefs-harness/tests/writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion.rs
@@ -6,9 +6,10 @@
 //! - 合成一个拒因时实现报的那一拒（C664 之前 `mount.rs` 的 `refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage`
 //!   拼的）：判据 N-配置 为真、重读仍为真，前一遍是读阶段往下走用的那一遍读数、后一遍同一个见证值判 R2 判不出。
 //!
-//! 挂载透明的射程只管不带故障的状态（D13（验证路线） 已定项 1）：崩溃态注入过读故障时，两遍读之间的瞬时读错在崩溃镜像上复算不出，
-//! 「读阶段判完之后系统配置又读不出」那一拒的判据复算为假不算拒得不合法；判据 N-配置 那一拒照旧复算（读阶段本来就判过它为假）。
-//! 没注入读故障的崩溃态上同一拒复算为假就是拒错了。
+//! 被拒的那次挂载自己在读故障下跑时（判官入参 `ReadFaultsUnderTheRefusedMount`，判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md`
+//! 第二节 J2 C3 / J2-a 那一行、改法 G3），两遍读之间的瞬时读错在崩溃镜像上复算不出，「读阶段判完之后系统配置又读不出」那一拒的判据复算为假
+//! 不算拒得不合法；判据 N-配置 那一拒照旧复算（读阶段本来就判过它为假）。被拒的那次挂载不带故障层时（崩溃注入第二、三截的调法：
+//! 盘是崩溃镜像本身，历史里注入过读故障也不会在这次挂载里出瞬时读错），同一拒复算为假就是拒错了，崩溃态注没注入过读故障都一样。
 
 use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
 use singlefs_core::admission::SpaceAdmission;
@@ -23,9 +24,10 @@ use singlefs_harness::history::{
 };
 use singlefs_harness::memory_pool::MemoryPool;
 use singlefs_harness::mount_transparency::{
-    judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields, ComparedVersion,
-    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage,
-    RefusalConditionNotMet, RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
+    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount,
+    read_recovery_fields, ComparedVersion, DiskAfterTheRefusal, ReadFaultsInTheCrashState,
+    ReadFaultsUnderTheRefusedMount, ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet,
+    RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
 };
 use singlefs_harness::SharedStream;
 
@@ -72,24 +74,27 @@ fn read_stage_settled_on(
     }
 }
 
-/// 拒的理由、崩溃态注没注入读故障，判一次（所选那一版取只读恢复落到的那一版，盘上逐字节不变、只读读出在允许集合里）。
+/// 拒的理由、崩溃态注没注入读故障、被拒的那次挂载自己在不在读故障下跑，判一次（所选那一版取只读恢复落到的那一版，盘上逐字节不变、
+/// 只读读出在允许集合里）。
 fn verdict_on(
     crash_image: &MemoryPool,
     refusal_on: fn(SelectedVersionAgainstTheWitness) -> MountError,
     read_faults: ReadFaultsInTheCrashState,
+    read_faults_under_the_refused_mount: ReadFaultsUnderTheRefusedMount,
 ) -> WritableMountRefusalVerdict {
     let fields = read_recovery_fields(crash_image, ComparedVersion::TheRecoveredVersion);
     let selected_version = fields
         .report
         .effective_root
         .expect("健康盘面上只读恢复落到第一个文件那一版");
-    judge_writable_mount_refusal_of_a_crash_state(
+    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
         crash_image,
         fields.report.effective_root,
         &refusal_on(read_stage_settled_on(selected_version)),
         DiskAfterTheRefusal::ByteForByteUnchanged,
         ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
         read_faults,
+        read_faults_under_the_refused_mount,
     )
 }
 
@@ -122,10 +127,11 @@ fn refused_as_the_witness_criterion(read_stage: SelectedVersionAgainstTheWitness
     ))
 }
 
-/// 场景：第一个文件之后的健康盘面当崩溃镜像，可写挂载在取号那一刻因瞬时读错拒（盘上看不出来）。
-/// 预期：崩溃态注入过读故障时，分开报的那一拒合法（判据复算为假不算不合法，盘上不变、只读读出在允许集合里）；合成一个拒因时实现报的
-/// 判据 N-配置 那一拒复算为假、判不合法；没注入读故障时分开报的那一拒复算为假、判不合法（拒的理由在盘上不成立）。
-/// 判别力：注入过读故障时不放过那一拒的判据复算（`crates/mutations.tsv` 一行）时第一条红；没注入的也放过（一行）时第三条红；
+/// 场景：第一个文件之后的健康盘面当崩溃镜像，崩溃态注入过读故障，可写挂载在取号那一刻因瞬时读错拒（盘上看不出来）。
+/// 预期：被拒的那次挂载在读故障下跑时，分开报的那一拒合法（判据复算为假不算不合法，盘上不变、只读读出在允许集合里）；合成一个拒因时实现报的
+/// 判据 N-配置 那一拒复算为假、判不合法；被拒的那次挂载不带故障层时（崩溃注入第二、三截的调法），同一拒复算为假、判不合法——
+/// 崩溃态注入过读故障不放过它（攻方 zeros 那组比重上的形状：注入态 + 新拒因复算为假）。
+/// 判别力：入参不看、一律按「挂载不带故障层」（`crates/mutations.tsv` 一行）时第一条红；一律按「挂载在读故障下跑」（一行）时第三条红；
 /// 那一成员映射回判据 N-配置 那一条理由（一行）时第一条红。
 #[test]
 fn a_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_legal_where_the_merged_witness_refusal_is_not(
@@ -135,18 +141,20 @@ fn a_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_legal_where_t
         verdict_on(
             &crash_image,
             refused_after_the_read_stage,
-            ReadFaultsInTheCrashState::Injected
+            ReadFaultsInTheCrashState::Injected,
+            ReadFaultsUnderTheRefusedMount::TheRefusedMountReadThroughInjectedFaults
         ),
         WritableMountRefusalVerdict::Legal(
             RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
         ),
-        "注入过读故障：读阶段之后系统配置又读不出那一拒合法（瞬时读错在崩溃镜像上复算不出）"
+        "被拒的那次挂载在读故障下跑：读阶段之后系统配置又读不出那一拒合法（瞬时读错在崩溃镜像上复算不出）"
     );
     assert_eq!(
         verdict_on(
             &crash_image,
             refused_as_the_witness_criterion,
-            ReadFaultsInTheCrashState::Injected
+            ReadFaultsInTheCrashState::Injected,
+            ReadFaultsUnderTheRefusedMount::TheRefusedMountReadThroughInjectedFaults
         ),
         WritableMountRefusalVerdict::Illegal {
             reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
@@ -160,7 +168,8 @@ fn a_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_legal_where_t
         verdict_on(
             &crash_image,
             refused_after_the_read_stage,
-            ReadFaultsInTheCrashState::NoneInjected
+            ReadFaultsInTheCrashState::Injected,
+            ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults
         ),
         WritableMountRefusalVerdict::Illegal {
             reason: RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
@@ -168,6 +177,6 @@ fn a_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_legal_where_t
                 RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
             ],
         },
-        "没注入读故障：盘上两槽都读得出、都自证过，那一拒的判据复算为假，拒错了"
+        "崩溃态注入过读故障、被拒的那次挂载却不带故障层：盘上两槽都读得出、都自证过，那一拒的判据复算为假，拒错了"
     );
 }
````

### mutations-append.tsv

````text
第28批 I-9.16 ④ 退回只要种类 1–4：缺 deadlist 那一条放过（正推 C1）	crates/singlefs-checker/src/walk.rs	const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 11] =\n    TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER;	const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 4] = [\n    TREE_KIND_EXTENT,\n    TREE_KIND_INODE,\n    TREE_KIND_ALLOCATION,\n    TREE_KIND_ACCOUNTING,\n];	-p singlefs-harness --test checker_known_bad_images -- a_tree_table_without_its_deadlist_entry_reddens_only_the_tree_table_ordering_invariant	a_tree_table_without_its_deadlist_entry_reddens_only_the_tree_table_ordering_invariant
第28批 I-9.16 ④ 退回只要种类 1–4：缺 logged_ops 那一条放过（正推 C1）	crates/singlefs-checker/src/walk.rs	const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 11] =\n    TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER;	const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 4] = [\n    TREE_KIND_EXTENT,\n    TREE_KIND_INODE,\n    TREE_KIND_ALLOCATION,\n    TREE_KIND_ACCOUNTING,\n];	-p singlefs-harness --test checker_known_bad_images -- a_tree_table_without_its_logged_operations_entry_reddens_the_tree_table_ordering_invariant_besides_the_walk	a_tree_table_without_its_logged_operations_entry_reddens_the_tree_table_ordering_invariant_besides_the_walk
第28批 I-5.4 带树一版根记录那一格不等分配记录树走到叶就计进评估（正推 C2）	crates/singlefs-checker/src/walk.rs	    let some_allocation_record_tree_walked_to_its_leaves = judged_allocation_nodes > 0;	    let some_allocation_record_tree_walked_to_its_leaves = true;	-p singlefs-harness --test checker_known_bad_images -- an_allocation_record_tree_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged	an_allocation_record_tree_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged
第28批 G1 攻方 MUT-A：core 的 T_time 内置默认 5 改 4（挂载透明「R2 = 内置默认」的期望照条款写死才抓得到）	crates/singlefs-core/src/system_configuration.rs	const TIME_THRESHOLD_SECONDS: u32 = 5;	const TIME_THRESHOLD_SECONDS: u32 = 4;	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount -- the_runtime_configuration_after_a_mount_alone_is_the_built_in_default_written_in_the_clauses	the_runtime_configuration_after_a_mount_alone_is_the_built_in_default_written_in_the_clauses
第28批 挂载透明 journal_tail 去掉「R1 ≤ R2」那一半	crates/singlefs-harness/src/mount_transparency.rs	        tail_before <= tail_after\n            && tail_after <= highest_record_counter	        tail_before.min(tail_after) <= tail_after\n            && tail_after <= highest_record_counter	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount -- a_journal_tail_below_the_one_on_the_crash_image_after_a_second_crash_is_red	a_journal_tail_below_the_one_on_the_crash_image_after_a_second_crash_is_red
第28批 挂载透明 journal_tail 挂载做完不判取等	crates/singlefs-harness/src/mount_transparency.rs	(!mount_completed || tail_after == highest_record_counter)	(!mount_completed || tail_after <= highest_record_counter)	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount -- a_journal_tail_below_the_highest_record_counter_in_the_ring_after_a_mount_and_one_publish_is_red	a_journal_tail_below_the_highest_record_counter_in_the_ring_after_a_mount_and_one_publish_is_red
第28批 G3 判官入参不看、一律按「被拒的挂载不带故障层」（瞬时读错判成拒错了）	crates/singlefs-harness/src/mount_transparency.rs	            Self::TheRefusedMountReadThroughInjectedFaults => true,	            Self::TheRefusedMountReadThroughInjectedFaults => false,	-p singlefs-harness --test writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion	a_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_legal_where_the_merged_witness_refusal_is_not
第28批 G3 判官入参不看、一律按「被拒的挂载在读故障下跑」（注入态 + 新拒因复算为假放过）	crates/singlefs-harness/src/mount_transparency.rs	            Self::TheRefusedMountReadTheCrashImageWithoutFaults => false,	            Self::TheRefusedMountReadTheCrashImageWithoutFaults => true,	-p singlefs-harness --test writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion	a_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_legal_where_the_merged_witness_refusal_is_not
第28批 G5 二次崩溃那一截不看读故障标记、照判挂载透明	crates/singlefs-checker-tier/src/crash_injection.rs	        violations.extend(match read_faults_in_the_first_crash_state {	        violations.extend(match ReadFaultsInTheCrashState::NoneInjected {	-p singlefs-checker-tier --lib -- crash_injection::tests::mount_transparency_is_not_judged_on_a_second_crash_inside_the_mount_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount	mount_transparency_is_not_judged_on_a_second_crash_inside_the_mount_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount
第28批 G6 B4：指纹只折段内前 32 个写	crates/singlefs-checker-tier/src/crash_injection.rs	    persisted_within_the_segment.iter().fold(	    persisted_within_the_segment.iter().take(32).fold(	-p singlefs-checker-tier --lib -- crash_injection::tests::first_crash_states_of_a_segment_of_one_thousand_twenty_four_writes_withholding_one_write_anywhere_seed_their_second_crashes_differently	first_crash_states_of_a_segment_of_one_thousand_twenty_four_writes_withholding_one_write_anywhere_seed_their_second_crashes_differently
第28批 G6 C3：历史种子对 3 取余	crates/singlefs-checker-tier/src/crash_injection.rs	        seed.0\n            ^ SECOND_CRASH_POINT_SEED_SALT	        (seed.0 % 3)\n            ^ SECOND_CRASH_POINT_SEED_SALT	-p singlefs-checker-tier --lib -- crash_injection::tests::first_crash_states_under_history_seeds_zero_and_every_power_of_two_seed_their_second_crashes_differently	first_crash_states_under_history_seeds_zero_and_every_power_of_two_seed_their_second_crashes_differently
````

### mutations-replacements.tsv

````text
26-3 I-5.4 带树一版的根记录分配记录树根不判全零	crates/singlefs-checker/src/walk.rs	                    if allocation_roots[0].all_zero {	                    if true {	-p singlefs-harness --test checker_known_bad_images -- a_file_version_whose_root_record_carries_a_non_zero_allocation_record_tree_root_reddens_the_disjointness_invariant_besides_the_walk	a_file_version_whose_root_record_carries_a_non_zero_allocation_record_tree_root_reddens_the_disjointness_invariant_besides_the_walk
C670（18-1 P2）接线：第二截换回不判 C670 的旧判官 judge_writable_mount_refusal	crates/singlefs-checker-tier/src/crash_injection.rs	            match judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(\n                refusal_on_the_crash_image.crash_image_materialized,\n                refusal_on_the_crash_image.version_landed_on_after_the_crash,\n                refusal_error,\n                disk,\n                refusal_on_the_crash_image.read_only_read_back,\n                refusal_on_the_crash_image.read_faults,\n                READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE,\n            ) {	            match singlefs_harness::mount_transparency::judge_writable_mount_refusal(\n                refusal_on_the_crash_image.crash_image_materialized,\n                refusal_on_the_crash_image.version_landed_on_after_the_crash,\n                refusal_error,\n                disk,\n                refusal_on_the_crash_image.read_only_read_back,\n            ) {	-p singlefs-checker-tier --lib -- crash_injection::tests::a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash	a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash
C670（18-1 P1）第三截：判再挂被拒时读故障恒交 Injected（不沿用第一次崩溃那个态的认定）	crates/singlefs-checker-tier/src/crash_injection.rs	            second_crash_image.read_only_read_back,\n            second_crash_image.read_faults,\n            READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE,\n        );	            second_crash_image.read_only_read_back,\n            ReadFaultsInTheCrashState::Injected,\n            READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE,\n        );	-p singlefs-checker-tier --lib -- crash_injection::tests::a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal	a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal
C670（18-1 P1）第三截：判再挂被拒换回不判 C670 的旧判官 judge_writable_mount_refusal	crates/singlefs-checker-tier/src/crash_injection.rs	    let verdict =\n        judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(\n            second_crash_image.crash_image_materialized,\n            second_crash_image.version_landed_on_after_the_crash,\n            refusal,\n            disk,\n            second_crash_image.read_only_read_back,\n            second_crash_image.read_faults,\n            READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE,\n        );	    let verdict = singlefs_harness::mount_transparency::judge_writable_mount_refusal(\n        second_crash_image.crash_image_materialized,\n        second_crash_image.version_landed_on_after_the_crash,\n        refusal,\n        disk,\n        second_crash_image.read_only_read_back,\n    );	-p singlefs-checker-tier --lib -- crash_injection::tests::a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal	a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal
````

### mutations-delete.txt

````text
第26批 C664 判法：注入过读故障时读阶段之后系统配置又读不出那一拒照样要判据在镜像上复算为真（瞬时读错判成拒错了）
第26批 C664 判法：没注入读故障时也放过读阶段之后系统配置又读不出那一拒的判据复算
````

## m3-formal-26-5

### crates.patch

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index e809587..8caab55 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -1318,6 +1318,11 @@ fn w_judge_pointer_head(p: Ptr) {
     w_judge(INVARIANT_I_2_4, p.mac_nonce_zero);
 }
 
+// I-2.4 的指针那一半另一格：头部 extent 偏移为 0（image::judge_pointer_extent_offset_is_zero；第一版指针指整个单元，与加密开不开无关）
+fn w_judge_pointer_extent_offset(p: Ptr) {
+    w_judge(INVARIANT_I_2_4, p.extent_offset_zero);
+}
+
 // walk.rs judge_unit_header：按期望的类的字段表判 I-1.6、I-2.4（三格）、I-2.3、I-1.4；交回头能不能用
 fn w_judge_unit_header(s: Slice, expected_class: u32, fsid_low: vec2<u32>) -> bool {
     let unit_class = rd8(s.base, 6u);
@@ -1474,6 +1479,7 @@ fn w_read_index_node(p: Ptr, expected_tree: vec2<u32>, schema: u32, key_range_re
     }
     w_judge_location_order(p);
     w_judge_pointer_head(p);
+    w_judge_pointer_extent_offset(p);
     w_note_pointer_references(p, 1u);
     let s = w_read_referenced_unit(p, NODE_BYTES);
     if (s.version == NONE) {
@@ -1661,7 +1667,8 @@ fn w_walk_instance_table_chain(first: Ptr, judge_rows: bool, fsid_low: vec2<u32>
     }
 }
 
-// 树表条目里 I-9.16 两道（walk.rs judge_tree_table_entries_ordering）
+// 树表条目里 I-9.16 四道（walk.rs judge_tree_table_entries_ordering）：① 盘上相邻两条树 ID 严格升序；条目宽读得出种类时
+// ③ ④（walk.rs judge_tree_table_entry_kind_counts）与 ② 发号次序。只有违例位，几道的先后不影响结论
 fn w_judge_tree_table_entries_ordering(n: Node) {
     for (var e = 1u; e < n.entry_count; e = e + 1u) {
         if (!lt64(rd64(n.base, entry_offset(n, e - 1u)), rd64(n.base, entry_offset(n, e)))) {
@@ -1673,6 +1680,34 @@ fn w_judge_tree_table_entries_ordering(n: Node) {
         w_judge(INVARIANT_I_9_16, true);
         return;
     }
+    // ③ ④：0 条的表不判
+    if (n.entry_count > 0u) {
+        // ③：每一种种类码（登记过的与没登记的都算）至多一条
+        for (var later_entry = 1u; later_entry < n.entry_count; later_entry = later_entry + 1u) {
+            let later_kind = rd16(n.base, entry_offset(n, later_entry) + 10u);
+            for (var earlier_entry = 0u; earlier_entry < later_entry; earlier_entry = earlier_entry + 1u) {
+                if (rd16(n.base, entry_offset(n, earlier_entry) + 10u) == later_kind) {
+                    w_judge(INVARIANT_I_9_16, false);
+                    return;
+                }
+            }
+        }
+        // ④：树表非空时进树表的十一种（发号次序 1,2,3,4,6,7,8,9,10,11,12）各至少一条，与 ③ 合起来各恰好一条
+        // （判决 m3-formal-code-b26-r1 第二节 J1 C1：core 重建一版时十一种各要一条）
+        for (var position = 0u; position < TREE_KINDS_OF_THE_TREE_TABLE; position = position + 1u) {
+            let required_kind = tree_kind_in_the_issuing_order(position);
+            var required_kind_is_present = false;
+            for (var entry_index = 0u; entry_index < n.entry_count; entry_index = entry_index + 1u) {
+                if (rd16(n.base, entry_offset(n, entry_index) + 10u) == required_kind) {
+                    required_kind_is_present = true;
+                }
+            }
+            if (!required_kind_is_present) {
+                w_judge(INVARIANT_I_9_16, false);
+                return;
+            }
+        }
+    }
     // ②：按发号次序 1,2,3,4,6,7,8,9,10,11,12（walk.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER）
     var previous_tree = vec2<u32>(0u, 0u);
     var have_previous = false;
@@ -4079,6 +4114,21 @@ fn judge_root_records_live_in_the_slot_their_txg_names(g: CheckerSlot) {
     }
 }
 
+// 根记录里算法类型 1 + nonce 12 + MAC 16 那 29 字节的起点与字节数（D22（单元原子性怎么合成） 已定项 7 的字段表：偏移 428、429、441，
+// 合计到 457；image::ROOT_RECORD_ALGORITHM_NONCE_AND_MAC 是 428..457）
+const ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_OFFSET: u32 = 428u;
+const ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_BYTES: u32 = 29u;
+
+// I-2.4 根记录那一处：根环里每一条自证过的根（checker_collect_roots，不按候选集滤），根槽 428..457 全 0
+// （image::judge_root_record_algorithm_nonce_and_mac_are_zero；这一版 checker 只收加密类型 0 的系统配置，加密关着时恒 0）。只判不断
+fn judge_root_record_algorithm_nonce_and_mac_are_zero() {
+    for (var root_index = 0u; root_index < root_count; root_index = root_index + 1u) {
+        if (!all_zero(checker_root_of(root_index).base, ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_OFFSET, ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_BYTES)) {
+            violate(INVARIANT_I_2_4);
+        }
+    }
+}
+
 // check_pool_image 从读系统配置槽到走读那一段，交回结论里「走到了第几步」。整池不作保的两支先判：任一盘任一槽带这一版读者不收的值
 // （I-7.13）、任一盘任一可择槽 I-8.1 的三个字段有一格不等（两支都中时走 I-7.13 那一支，image 与 walk 的次序）——中了就只报那一条、
 // 别的不判；再 I-8.5（全池没有一个自证过的槽）、择系统配置、系统配置与根环那几条、逐盘 journal，根环里有根才进走读。
@@ -4188,6 +4238,7 @@ fn judge_the_pool_from_its_system_configuration_slots(device_count: u32) -> u32
     }
     let fsid_low = vec2<u32>(g.fsid.x, g.fsid.y);
     checker_collect_roots(g);
+    judge_root_record_algorithm_nonce_and_mac_are_zero();
     judge_root_records_live_in_the_slot_their_txg_names(g);
     // 自证过、属于本池的槽：I-7.14 本盘设备号；I-7.12 每盘的 F
     for (var d = 0u; d < device_count; d = d + 1u) {
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index df4d8ef..2ef0afc 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -892,7 +892,9 @@ fn judge_allocation_record_ranges() {
     }
 }
 
-// I-5.4：候选集里每条根的分配记录树（树表种类 3 那一条与根记录直接持有的那一棵）
+// I-5.4：候选集里每条根的分配记录树（树表种类 3 那一条与根记录直接持有的那一棵）；同一条下另判一格「一条根只有一棵分配记录树」：
+// 树表读得出、条目非空（带树）的那一版，根记录偏移 342 那条分配记录树根必须全零（walk.rs judge_allocation_records_disjoint，
+// 判决 m3-formal-c655-c687-r1 第二节 Y1 allocation_record_tree_root 那一行）。条目宽不到 200 的表也判，所以放在跳过窄表那一句之前
 fn judge_allocation_records_disjoint() {
     c_seen_node_count = 0u;
     r_noting = false;
@@ -910,7 +912,13 @@ fn judge_allocation_records_disjoint() {
             continue;
         }
         let tt = r_read_index_node(tree_table_pointer);
-        if (!tt.ok || tt.entry_width < 200u) {
+        if (!tt.ok) {
+            continue;
+        }
+        if (tt.entry_count > 0u && !held.all_zero) {
+            violate(INVARIANT_I_5_4);
+        }
+        if (tt.entry_width < 200u) {
             continue;
         }
         for (var e = 0u; e < tt.entry_count; e = e + 1u) {
diff --git a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
index 203d087..7b0f5a9 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -494,10 +494,15 @@ fn device_bytes() -> vec2<u32> {
 
 // ───────────────────────── 指针 ─────────────────────────
 
+// 指针头部 extent 偏移那 2 字节在指针里的偏移（D19（块指针的结构与宽度预算） 已定项 7 的字段表：MAC 16、nonce 12、算法类型 1、压缩算法码 1、
+// 压后长度 2 之后；`singlefs_checker::image` 的 POINTER_EXTENT_OFFSET_FIELD 是 32..34）。第一版指针指整个单元，这 2 字节恒 0
+const POINTER_EXTENT_OFFSET_FIELD_OFFSET: u32 = 32u;
+
 struct Ptr {
     all_zero: bool,
     head_ok: bool,
     mac_nonce_zero: bool,
+    extent_offset_zero: bool,
     birth_tree: vec2<u32>,
     birth_txg: vec2<u32>,
     device0: u32,
@@ -514,6 +519,7 @@ fn parse_pointer_common(base: u32, o: u32) -> Ptr {
     p.all_zero = false;
     p.mac_nonce_zero = all_zero(base, o, 28u);
     p.head_ok = p.mac_nonce_zero && rd8(base, o + 28u) == 0u && rd8(base, o + 29u) == 0u && rd16(base, o + 30u) == 0u;
+    p.extent_offset_zero = rd16(base, o + POINTER_EXTENT_OFFSET_FIELD_OFFSET) == 0u;
     p.birth_tree = rd64(base, o + 34u);
     p.birth_txg = rd64(base, o + 42u);
     p.device0 = rd32(base, o + 50u);
@@ -841,6 +847,7 @@ fn zero_pointer() -> Ptr {
     p.all_zero = true;
     p.head_ok = true;
     p.mac_nonce_zero = true;
+    p.extent_offset_zero = true;
     p.birth_tree = vec2<u32>(0u, 0u);
     p.birth_txg = vec2<u32>(0u, 0u);
     p.device0 = 0u;
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 154ac87..69c9eb5 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -11,6 +11,8 @@
 //! 里程碑三·3 删除线（I-9.5、I-3.4、I-4.7、I-4.6、I-9.6 墓碑那一半）：删除中间态（unlink 意图与第一批）一条流钉四条判过而成立，
 //! 每条各一条只改坏它管的那个字段的流钉 CPU 红、GPU 逐位跟上；走过 nlink = 0 那几版的流只比池级 checker 那一半（恢复那一半今天对不上，见
 //! `compare_the_checker_on_a_deletion_flow`）。
+//! I-9.16 ③ ④（种类重复、缺 deadlist）、根槽 428..457（I-2.4）、带树一版根记录里的分配记录树根（I-5.4）各一条坏流整份比；码 2 节点指针头部的
+//! extent 偏移（I-2.4）一条只比池级 checker 那一半（恢复那一半今天对不上，见 `record_new_pool_with_an_extent_offset_on_the_inode_tree_root_pointer`）。
 //! 要本机一张空闲显存够的 NVIDIA 独显：本机一张 NVIDIA 独显适配器都没有时用卡的用例打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、
 //! 返回成功；有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）。要 `verdict-store,gpu` 特性；
 //! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（没设不定额度）、
@@ -4050,6 +4052,257 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entr
     );
 }
 
+// ───────────── 第 26 批 26-5：I-9.16 ③ ④、根槽 428..457、带树一版的分配记录树根、码 2 节点指针头部的 extent 偏移 ─────────────
+
+/// 树表条目里种类那 2 字节的偏移（D8（核心索引结构） 已定项 8 的字段表：树 ID 8、条目长度 2 之后）。
+const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
+/// 树表条目里树根指针（86 字节的码 2 指针）的偏移（种类 2、flags 2 之后）。
+const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET: usize = 14;
+/// 稀疏旁表与 deadlist 的种类码（D18（块里携带什么信息） 已定项 24 第 6 条那一组）：第一版两棵树都不建根，树表条目的根指针全零，走读与恢复都不读。
+const TREE_KIND_SPARSE_SIDE_TABLE: u16 = 7;
+const TREE_KIND_DEADLIST: u16 = 8;
+/// 一个没登记的种类码：走读与恢复都不认它、不读它的根。
+const UNREGISTERED_TREE_KIND: u16 = 13;
+/// 根记录里树表指针的第 0 条位置条目（盘 4 + 槽 6）的偏移：树表指针在 36，位置条目在指针偏移 50。
+const ROOT_RECORD_TREE_TABLE_POINTER_FIRST_LOCATION_OFFSET: usize = 86;
+/// 根记录里「树表 0 条那一版的分配记录树根」指针的偏移（D22（单元原子性怎么合成） 已定项 7 的字段表）。
+const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET: usize = 342;
+/// 根记录里算法类型 1 + nonce 12 + MAC 16 那 29 字节的最后一个字节（428..457 的末字节）。
+const ROOT_RECORD_LAST_MAC_BYTE_OFFSET: usize = 456;
+
+/// 发布那张树表里种类是 `from_kind` 的那条（恰好一条、根指针全零）的种类改成 `to_kind`、重封到根。树 ID 与条目次序不动。
+fn record_new_pool_with_a_tree_table_entry_kind_rewritten(
+    tag: &str,
+    name: &'static str,
+    from_kind: u16,
+    to_kind: u16,
+) -> RecordedFlow {
+    record_new_pool_with_a_resealed_field(tag, name, &is_a_tree_table_unit, false, &|bytes| {
+        let entry = only_tree_table_entry_of_kind(bytes, from_kind);
+        let root_pointer = entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
+        assert_eq!(
+            bytes[root_pointer..root_pointer + NODE_POINTER_BYTES],
+            [0u8; NODE_POINTER_BYTES],
+            "种类 {from_kind} 那条的根指针本来全零（这棵树第一版不建根，走读与恢复都不读它）"
+        );
+        let kind = entry + TREE_TABLE_ENTRY_KIND_OFFSET;
+        bytes[kind..kind + 2].copy_from_slice(&to_kind.to_le_bytes());
+    })
+}
+
+/// I-9.16 ③（每种至多一条）：deadlist 那条的种类改成稀疏旁表的 7——种类 7 两条、种类 8 一条都没有。① ② 照旧成立（② 按每种第一次出现的那条）。
+/// core 的只读恢复在读任何一棵树之前拒（`tree_table_entries_each_kind_at_most_once`），GPU 恢复内核种类重复那一支同样拒。
+fn record_new_pool_with_two_tree_table_entries_of_the_sparse_side_table_kind(
+    tag: &str,
+) -> RecordedFlow {
+    record_new_pool_with_a_tree_table_entry_kind_rewritten(
+        tag,
+        "new-pool-tree-table-two-entries-of-the-sparse-side-table-kind",
+        TREE_KIND_DEADLIST,
+        TREE_KIND_SPARSE_SIDE_TABLE,
+    )
+}
+
+/// I-9.16 ④（树表非空时十一种各恰好一条）：deadlist 那条的种类改成没登记的 13——没有一种重复，deadlist 一条都没有。
+/// core 的只读恢复照常读到文件（只要种类 1–4 四棵有根的树），可写挂载重建这一版时拒「树表里没有那棵树」；GPU 恢复内核也照常读到。
+fn record_new_pool_with_no_tree_table_entry_of_the_deadlist_kind(tag: &str) -> RecordedFlow {
+    record_new_pool_with_a_tree_table_entry_kind_rewritten(
+        tag,
+        "new-pool-tree-table-without-the-deadlist-kind",
+        TREE_KIND_DEADLIST,
+        UNREGISTERED_TREE_KIND,
+    )
+}
+
+/// I-2.4 根记录那一处：发布的根记录里 MAC 末字节（偏移 456）写 1、重封根记录。core 不读这 29 字节（`root_record.rs` 的 `parse_slot` 跳过），
+/// 两遍恢复照常；CPU 池级 checker 对根环里每条自证过的根判这 29 字节全 0，发布的根落了的状态上红 I-2.4。
+fn record_new_pool_with_a_non_zero_byte_in_the_root_record_mac(tag: &str) -> RecordedFlow {
+    let mut flow = record_new_pool_file_creation(tag, "new-pool-root-record-mac-last-byte-one");
+    let publish_root = flow.judged_root_index;
+    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
+        assert_eq!(
+            bytes[ROOT_RECORD_LAST_MAC_BYTE_OFFSET], 0,
+            "加密关着时根记录的 MAC 恒 0"
+        );
+        bytes[ROOT_RECORD_LAST_MAC_BYTE_OFFSET] = 1;
+    });
+    flow
+}
+
+/// 第 `root_write` 次写（根槽写）指着的树表单元的字节：按根记录里树表指针第 0 条位置条目找根槽写之前落在那个槽上的最后一次单元写。
+fn tree_table_unit_bytes_named_by_the_root_write(
+    flow: &RecordedFlow,
+    root_write: usize,
+) -> Vec<u8> {
+    let root_bytes = flow.writes[root_write].bytes().expect("根槽写带字节");
+    let location = ROOT_RECORD_TREE_TABLE_POINTER_FIRST_LOCATION_OFFSET;
+    let device = u32::from_le_bytes(root_bytes[location..location + 4].try_into().expect("4"));
+    let mut slot = [0u8; 8];
+    slot[..6].copy_from_slice(&root_bytes[location + 4..location + 10]);
+    let slot = u64::from_le_bytes(slot);
+    flow.writes[..root_write]
+        .iter()
+        .rev()
+        .find(|write| {
+            write.kind == StepKind::UnitWrite
+                && write.device.0 == device
+                && write.offset.0 / 16384 == slot
+        })
+        .and_then(RetainedWrite::bytes)
+        .expect("根槽写之前写过它指着的树表单元")
+        .to_vec()
+}
+
+/// I-5.4 那一格「一条根只有一棵分配记录树」：发布的根（带树的一版）根记录偏移 342 那条分配记录树根本来全零，换成它自己树表里种类 3 那条的根指针、
+/// 重封根记录。走读把它当树 0 再走一遍那棵树（另红 I-1.3、I-5.1），发布那条 journal 记录的新根段里这一项仍全零（另红 I-8.10）；
+/// 两棵树指着同一个根，I-5.4 的区间判定只判一次、不红——红 I-5.4 的只有全零那一格。core 的只读恢复在带树的一版上不读这一项，两遍恢复照常。
+fn record_new_pool_with_the_allocation_record_tree_root_of_the_root_record_not_zero(
+    tag: &str,
+) -> RecordedFlow {
+    let mut flow = record_new_pool_file_creation(
+        tag,
+        "new-pool-root-record-allocation-record-tree-root-not-zero",
+    );
+    let publish_root = flow.judged_root_index;
+    let tree_table = tree_table_unit_bytes_named_by_the_root_write(&flow, publish_root);
+    let allocation_entry = only_tree_table_entry_of_kind(&tree_table, 3);
+    let allocation_root_pointer =
+        tree_table[allocation_entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET
+            ..allocation_entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET + NODE_POINTER_BYTES]
+            .to_vec();
+    assert_ne!(
+        allocation_root_pointer,
+        vec![0u8; NODE_POINTER_BYTES],
+        "带树那一版的分配记录树有根"
+    );
+    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
+        let held = ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET;
+        assert_eq!(
+            bytes[held..held + NODE_POINTER_BYTES],
+            [0u8; NODE_POINTER_BYTES],
+            "带树的一版根记录里这一项本来全零"
+        );
+        bytes[held..held + NODE_POINTER_BYTES].copy_from_slice(&allocation_root_pointer);
+    });
+    flow
+}
+
+/// I-2.4 指针那一半的 extent 偏移那一格：发布那张树表里 inode 树那条（种类 2）的根指针头部 extent 偏移写 1、重封到根。
+/// 池级 checker 读码 2 节点（`read_index_node`）时判它，CPU 与 GPU 走读内核都红 I-2.4。core 读树表条目时判这 2 字节、把那一条当损坏拒，
+/// GPU 恢复内核读树时不判（它只在记录新根段四条指针上判 extent 偏移）：恢复那一半今天对不上，这条流只比池级 checker 那一半。
+fn record_new_pool_with_an_extent_offset_on_the_inode_tree_root_pointer(tag: &str) -> RecordedFlow {
+    record_new_pool_with_a_resealed_field(
+        tag,
+        "new-pool-inode-tree-root-pointer-extent-offset-one",
+        &is_a_tree_table_unit,
+        false,
+        &|bytes| {
+            let entry = only_tree_table_entry_of_kind(bytes, 2);
+            let extent_offset =
+                entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET + POINTER_EXTENT_OFFSET_OFFSET;
+            assert_eq!(
+                bytes[extent_offset..extent_offset + 2],
+                [0u8; 2],
+                "第一版指针的 extent 偏移恒 0"
+            );
+            bytes[extent_offset..extent_offset + 2].copy_from_slice(&1u16.to_le_bytes());
+        },
+    )
+}
+
+/// GPU 与 CPU 池级 checker 在这条流的每个状态上违例位逐位相同、一个状态都不许判不了，CPU 池级 checker 在这条流上红 `invariant`；
+/// 恢复那一半不比（调用方写明为什么）。本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
+fn assert_the_gpu_checker_agrees_with_the_cpu_on_a_flow_reddening(
+    test_name: &str,
+    flow: &RecordedFlow,
+    invariant: &'static str,
+) {
+    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, flow, expand_up_to());
+    assert!(
+        comparison.cpu_checker_red_invariants.contains(invariant),
+        "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_red_invariants
+    );
+    assert!(
+        comparison.checker_mismatches.is_empty(),
+        "流 {}：{} 个状态池级 checker 的违例位 GPU 与 CPU 不一致或判不了（前几个）：\n{}",
+        flow.name,
+        comparison.checker_mismatches.len(),
+        comparison
+            .checker_mismatches
+            .iter()
+            .filter(|line| !line.is_empty())
+            .cloned()
+            .collect::<Vec<_>>()
+            .join("\n")
+    );
+}
+
+/// I-9.16 ③：树表里一种种类有两条。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_one_kind() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_one_kind",
+        &record_new_pool_with_two_tree_table_entries_of_the_sparse_side_table_kind(
+            "crash-judge-gpu-i916-kind-twice",
+        ),
+        "I-9.16",
+    );
+}
+
+/// I-9.16 ④：树表非空而十一种里缺 deadlist（种类 1–4 之外的那七种之一，CPU 的 ④ 按十一种判时才红）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_lacks_the_deadlist_kind() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_lacks_the_deadlist_kind",
+        &record_new_pool_with_no_tree_table_entry_of_the_deadlist_kind(
+            "crash-judge-gpu-i916-kind-missing",
+        ),
+        "I-9.16",
+    );
+}
+
+/// I-2.4：根记录里算法类型 / nonce / MAC 那 29 字节有非 0。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_root_record_carries_a_non_zero_mac_byte() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_root_record_carries_a_non_zero_mac_byte",
+        &record_new_pool_with_a_non_zero_byte_in_the_root_record_mac(
+            "crash-judge-gpu-i24-root-mac",
+        ),
+        "I-2.4",
+    );
+}
+
+/// I-5.4：带树的一版根记录里还持有一棵分配记录树。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_version_with_trees_also_holds_an_allocation_record_tree_in_its_root_record(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_version_with_trees_also_holds_an_allocation_record_tree_in_its_root_record",
+        &record_new_pool_with_the_allocation_record_tree_root_of_the_root_record_not_zero(
+            "crash-judge-gpu-i54-held-root",
+        ),
+        "I-5.4",
+    );
+}
+
+/// I-2.4：码 2 节点的指针头部 extent 偏移非 0（池级 checker 那一半）。
+#[test]
+fn the_gpu_checker_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset() {
+    assert_the_gpu_checker_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_checker_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset",
+        &record_new_pool_with_an_extent_offset_on_the_inode_tree_root_pointer(
+            "crash-judge-gpu-i24-node-pointer-extent-offset",
+        ),
+        "I-2.4",
+    );
+}
+
 // ───────────── 里程碑三·3 删除线：I-9.5、I-3.4、I-4.7、I-4.6 与 I-9.6 墓碑那一半（GPU 扫描内核的删除态扫描与 CPU 池级 checker 逐状态逐位对拍）─────────────
 
 /// 码 2 / 码 3 单元头里的声明长度（偏移 8，2 字节）。
````

### mutations-append.tsv

````text
第 26 批 26-5 GPU 判器：走读内核 I-9.16 ④ 只要种类 1–4（缺 deadlist 的树表不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        for (var position = 0u; position < TREE_KINDS_OF_THE_TREE_TABLE; position = position + 1u) {	        for (var position = 0u; position < 4u; position = position + 1u) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_lacks_the_deadlist_kind	the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_lacks_the_deadlist_kind
第 26 批 26-5 GPU 判器：走读内核根槽 428..457 只判前 28 字节（MAC 末字节非 0 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	const ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_BYTES: u32 = 29u;	const ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_BYTES: u32 = 28u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_root_record_carries_a_non_zero_mac_byte	the_gpu_judge_agrees_with_the_cpu_when_a_root_record_carries_a_non_zero_mac_byte
第 26 批 26-5 GPU 判器：扫描内核不判带树一版根记录里的分配记录树根全零	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (tt.entry_count > 0u && !held.all_zero) {	        if (false && tt.entry_count > 0u && !held.all_zero) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_version_with_trees_also_holds_an_allocation_record_tree_in_its_root_record	the_gpu_judge_agrees_with_the_cpu_when_a_version_with_trees_also_holds_an_allocation_record_tree_in_its_root_record
第 26 批 26-5 GPU 判器：走读内核读码 2 节点不判指针头部 extent 偏移	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    w_judge_pointer_head(p);\n    w_judge_pointer_extent_offset(p);\n    w_note_pointer_references(p, 1u);	    w_judge_pointer_head(p);\n    w_note_pointer_references(p, 1u);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_checker_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset	the_gpu_checker_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset
````

## m3-formal-29-1

### crates.patch

````diff
diff --git a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
index 6b0b5f4..6bfee63 100644
--- a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
+++ b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
@@ -3435,6 +3435,10 @@ fn refusal_condition_text(condition: &RefusalConditionNotMet) -> String {
         RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(branch) => {
             format!("witness_criterion_true_without_read_faults:{branch:?}")
         }
+        RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults => {
+            "system_configuration_unreadable_after_the_read_stage_criterion_true_without_read_faults"
+                .to_string()
+        }
     }
 }
 
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index 9413449..e5a0109 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -530,6 +530,11 @@ pub struct CrashInjectionTally {
     /// 崩溃态没注入读故障而判据在二次崩溃镜像上复算仍为真、拒得不合法的（判红）。
     pub remount_refusals_for_a_newer_publish_witnessed_by_system_configuration: u64,
     pub remount_refusals_with_the_witness_criterion_true_without_read_faults: u64,
+    /// 二次崩溃之后再挂被拒、拒因是「读阶段判完之后系统配置又读不出」的（第三截判改法 G4 那一条，与 C670 同形）：判过的、
+    /// 崩溃态没注入读故障而那一拒的判据在二次崩溃镜像上复算为真、拒得不合法的（判红）。
+    pub remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage: u64,
+    pub remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults:
+        u64,
     /// 每一截（崩溃镜像本身、挂载与一次发布之后、二次崩溃）判到已知红与新发现的状态数之和。
     pub crash_states_ending_known_red: BTreeMap<usize, u64>,
     pub crash_states_ending_new_finding: u64,
@@ -545,7 +550,8 @@ pub struct CrashInjectionTally {
     /// 崩溃后镜像上的可写挂载被拒、理由在 D23（journal 的角色与格式） 已定项 14 那张单子上的：拒得合法的、不合法的（判红）。
     pub writable_mount_refusals_on_the_d23_list_legal: u64,
     pub writable_mount_refusals_on_the_d23_list_illegal: u64,
-    /// 崩溃窗口末尾之前历史里有一步在注入的读故障下发过东西的崩溃状态：第二截判拒可写时不判 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）那一条。
+    /// 崩溃窗口末尾之前历史里有一步在注入的读故障下发过东西的崩溃状态：第二截判拒可写时不判 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）
+    /// 与改法 G4 那两条。
     pub crash_points_after_a_step_reading_under_injected_faults: u64,
     /// 第一截与二次崩溃那一截上 RefFS 与 O1 判官两票合起来（G1 用户定案：任一红即红），按哪一边红分开计；
     /// RefFS 弃权的按理由另计（两截并在一起）。
@@ -613,6 +619,8 @@ impl CrashInjectionTally {
             remount_idempotence_not_judged,
             remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
             remount_refusals_with_the_witness_criterion_true_without_read_faults,
+            remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage,
+            remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults,
             crash_states_ending_known_red,
             crash_states_ending_new_finding,
             mount_transparency_judged,
@@ -727,6 +735,10 @@ impl CrashInjectionTally {
             remount_refusals_for_a_newer_publish_witnessed_by_system_configuration;
         self.remount_refusals_with_the_witness_criterion_true_without_read_faults +=
             remount_refusals_with_the_witness_criterion_true_without_read_faults;
+        self.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage +=
+            remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage;
+        self.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults +=
+            remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults;
         for (form, count) in crash_states_ending_known_red {
             *self.crash_states_ending_known_red.entry(*form).or_insert(0) += count;
         }
@@ -932,6 +944,12 @@ impl CrashInjectionTally {
             self.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
             self.remount_refusals_with_the_witness_criterion_true_without_read_faults
         );
+        let _ = writeln!(
+            text,
+            "二次崩溃之后再挂被拒、拒因是读阶段判完之后系统配置又读不出的（第三截判 G4 那一条）：{} 次，崩溃态没注入读故障而判据复算为真、拒得不合法 {}",
+            self.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage,
+            self.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults
+        );
         let _ = writeln!(
             text,
             "挂载透明：第一次崩溃镜像上判了 {:?}，二次崩溃镜像上按段判了 {:?}；没判的 {:?}；判红的字段 {:?}；某一样没判的 {:?}；把清扫删的行放回去再比的 {} 次",
@@ -2555,7 +2573,8 @@ struct RefusalOnTheCrashImage<'crash> {
     crash_image_materialized: &'crash MemoryPool,
     version_landed_on_after_the_crash: Option<(InstanceGeneration, CheckpointTxg)>,
     read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
-    /// 这个崩溃态注没注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：没注入的，判据 N-配置 复算为真即拒得不合法（C670）。
+    /// 这个崩溃态注没注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：没注入的，判据 N-配置 复算为真即拒得不合法（C670），
+    /// 「读阶段判完之后系统配置又读不出」那一拒的判据复算为真也拒得不合法（改法 G4）。
     read_faults: ReadFaultsInTheCrashState,
 }
 
@@ -2563,7 +2582,8 @@ struct RefusalOnTheCrashImage<'crash> {
 /// 做没做成都在之后的池上跑池级 checker；`record_check` 是同一个池上记录核对器的结论
 /// （[`HistoryThenWritableMountRecords::record_check_after_the_writable_mount`]）。挂载被拒而理由在 D23（journal 的角色与格式） 已定项 14
 /// 那张单子上的，判拒绝本身（`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount`，被拒的那次挂载恒交不带故障层：判据在崩溃镜像上复算为真、
-/// 盘上逐字节不变、只读读出在允许集合里；崩溃态没注入读故障而判据 N-配置 复算为真的另判不合法，C670），合法的只计数，不合法的报红。别的理由被拒按「模型说该成、实现拒了」报：
+/// 盘上逐字节不变、只读读出在允许集合里；崩溃态没注入读故障而判据 N-配置 复算为真的另判不合法，C670，「读阶段判完之后系统配置又读不出」
+/// 那一拒的判据复算为真的同样另判不合法，改法 G4），合法的只计数，不合法的报红。别的理由被拒按「模型说该成、实现拒了」报：
 /// 模型对可写挂载与之后那一次发布（覆盖写活着的第一个文件、第一个文件、建 inode）从不要求拒，容量墙之外也不许拒（`IdealModel::answer_mount_writable`）。
 #[allow(
     clippy::too_many_arguments,
@@ -3456,8 +3476,8 @@ fn judge_second_crashes_inside_the_writable_mount(
 
 /// I-8.4 的 R-换：二次崩溃镜像（`second_crash_image.crash_image_materialized`）上再起一次可写挂载，「恢复加写行那一整段」于是做了两次；
 /// 再挂之后只读恢复读出的内容要与没崩过的那一次挂载之后读出的（`once`）相同。不同判红（违例记在 [`REPLAY_IDEMPOTENCE_INVARIANT`] 名下）；
-/// 条款没写的情形（[`RemountIdempotenceNotJudged`]）只计数。再挂被拒时 I-8.4 照旧不判，另判 C670 那一条
-/// （[`judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion`]）；没崩过的那一次挂载被拒（`once` 是没判的理由）时不再挂，两样都不判。
+/// 条款没写的情形（[`RemountIdempotenceNotJudged`]）只计数。再挂被拒时 I-8.4 照旧不判，另判 C670 与改法 G4 那两条
+/// （[`judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_without_read_faults`]）；没崩过的那一次挂载被拒（`once` 是没判的理由）时不再挂，两样都不判。
 #[allow(
     clippy::too_many_arguments,
     reason = "每个参数各是一样东西：两次挂载的输入（连同清扫的配置）、计数与发现、这一个二次崩溃落在哪"
@@ -3503,7 +3523,7 @@ fn judge_the_remount_after_a_second_crash(
                 .remount_idempotence_not_judged
                 .entry(RemountIdempotenceNotJudged::TheRemountWasRefused)
                 .or_insert(0) += 1;
-            judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion(
+            judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_without_read_faults(
                 tally,
                 findings,
                 RemountRefusedAfterASecondCrash {
@@ -3596,8 +3616,11 @@ struct RemountRefusedAfterASecondCrash<'remount> {
 /// 第二节 K2-b）：二次崩溃镜像同样是一个崩溃态，再挂的拒因是判据 N-配置、崩溃态没注入读故障、判据在二次崩溃镜像上（撕裂的系统配置槽跳过）
 /// 复算仍为真 ⇒ 拒得不合法，报红（记在 [`CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount`] 那一截）。判官一并判的 U6 那四条
 /// （r5 判决第二节 U6）条款只管第二截，这一截不拿它们定红绿：只差那几条的不报红、不计不合法，判官列出的没满足的条件只随 C670 那一条的红带出来。
-/// 拒因不是判据 N-配置 的不判。
-fn judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion(
+/// 改法 G4（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2「新拒因少一条 C670 同形的判法（J2-b）」那一行）同一个接法：
+/// 再挂的拒因是「读阶段判完之后系统配置又读不出」、崩溃态没注入读故障、那一拒的判据在二次崩溃镜像上复算为真 ⇒ 拒得不合法，报红
+/// （判官的 [`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]）。
+/// 拒因是实例表读不出的不判。
+fn judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_without_read_faults(
     tally: &mut CrashInjectionTally,
     findings: &mut FindingsOfOneHistory,
     remount: RemountRefusedAfterASecondCrash<'_>,
@@ -3627,19 +3650,20 @@ fn judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion(
             second_crash_image.read_faults,
             READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE,
         );
-    let Some(conditions_not_met) =
-        conditions_not_met_by_a_remount_refusal_with_the_witness_criterion_true(tally, verdict)
+    let Some((reason, conditions_not_met)) =
+        reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
+            tally, verdict,
+        )
     else {
         return;
     };
     let detail = format!(
-        "崩在挂载的 {} 里、二次崩溃镜像上再挂被拒，理由 {:?}；崩溃态没注入读故障，判据在二次崩溃镜像上复算仍为真，拒得不合法（C670；判官列出的没满足的条件 {conditions_not_met:?}）：{member}",
-        phase.name(),
-        RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
+        "崩在挂载的 {} 里、二次崩溃镜像上再挂被拒，理由 {reason:?}；崩溃态没注入读故障，判据在二次崩溃镜像上复算仍为真，拒得不合法（C670 / G4；判官列出的没满足的条件 {conditions_not_met:?}）：{member}",
+        phase.name()
     );
     let model_disagreement = ModelDisagreement::new(
         ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
-        "没注入读故障的崩溃态上，见证判据不许为真：二次崩溃之后恢复到一版上，可写挂载（取号、写行、暖机）都该成"
+        "没注入读故障的崩溃态上，见证判据与「读阶段判完之后系统配置又读不出」那一拒的判据都不许为真：二次崩溃之后恢复到一版上，可写挂载（取号、写行、暖机）都该成"
             .to_string(),
         detail.clone(),
     );
@@ -3669,14 +3693,15 @@ fn judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion(
     );
 }
 
-/// 第三截那一判的计数与取舍：拒因是判据 N-配置 的记一次判过；判官列出的没满足的条件里有 C670 那一条
-/// （[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]）时再记一次不合法，交回全部没满足的条件（报红用）。
-/// 拒因不是它、C670 那一条不成立（注入过读故障、判据复算为假）、只差 U6 那几条的，交回 None。
-fn conditions_not_met_by_a_remount_refusal_with_the_witness_criterion_true(
+/// 第三截那一判的计数与取舍：拒因是判据 N-配置 或「读阶段判完之后系统配置又读不出」的，各记一次判过；判官列出的没满足的条件里有
+/// 没注入读故障时判据复算为真的那一条（N-配置 的 C670：[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]；
+/// 另一拒的改法 G4：[`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]）时
+/// 再各记一次不合法，交回拒因与全部没满足的条件（报红用）。拒因是实例表读不出的、那一条不成立（注入过读故障、判据复算为假）、只差 U6 那几条的，交回 None。
+fn reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
     tally: &mut CrashInjectionTally,
     verdict: WritableMountRefusalVerdict,
-) -> Option<Vec<RefusalConditionNotMet>> {
-    let conditions_not_met = match verdict {
+) -> Option<(RefusalReasonOnTheD23List, Vec<RefusalConditionNotMet>)> {
+    let (reason, conditions_not_met) = match verdict {
         WritableMountRefusalVerdict::ReasonNotOnTheD23List
         | WritableMountRefusalVerdict::Legal(
             RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
@@ -3684,14 +3709,6 @@ fn conditions_not_met_by_a_remount_refusal_with_the_witness_criterion_true(
         | WritableMountRefusalVerdict::Illegal {
             reason: RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
             ..
-        }
-        // 读阶段判完之后系统配置又读不出（C664 之后与判据 N-配置 分开报）：这一截只判 N-配置 那一条（C670），这一拒不判。
-        | WritableMountRefusalVerdict::Legal(
-            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
-        )
-        | WritableMountRefusalVerdict::Illegal {
-            reason: RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
-            ..
         } => return None,
         WritableMountRefusalVerdict::Legal(
             RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
@@ -3704,20 +3721,46 @@ fn conditions_not_met_by_a_remount_refusal_with_the_witness_criterion_true(
             conditions_not_met,
         } => {
             tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration += 1;
-            conditions_not_met
+            (
+                RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
+                conditions_not_met,
+            )
+        }
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
+        ) => {
+            tally.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage += 1;
+            return None;
+        }
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
+            conditions_not_met,
+        } => {
+            tally.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage += 1;
+            (
+                RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
+                conditions_not_met,
+            )
         }
     };
-    let witness_criterion_true_without_read_faults = conditions_not_met.iter().any(|condition| {
-        matches!(
-            condition,
-            RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(_)
-        )
-    });
-    if !witness_criterion_true_without_read_faults {
-        return None;
+    let mut criterion_true_without_read_faults = false;
+    // 迭代上界是判官列出的条件数（至多五条）；跨轮携带的只有上面那一个布尔；没有提前出口。
+    for condition in &conditions_not_met {
+        match condition {
+            RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(_) => {
+                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults += 1;
+                criterion_true_without_read_faults = true;
+            }
+            RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults => {
+                tally.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults += 1;
+                criterion_true_without_read_faults = true;
+            }
+            RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+            | RefusalConditionNotMet::DiskChangedByTheRefusedMount
+            | RefusalConditionNotMet::ReadOnlyReadBackOutsideTheAllowedSet => {}
+        }
     }
-    tally.remount_refusals_with_the_witness_criterion_true_without_read_faults += 1;
-    Some(conditions_not_met)
+    criterion_true_without_read_faults.then_some((reason, conditions_not_met))
 }
 
 /// 这个崩溃状态在段内扣下的写各是什么种类（同一种只记一次）。段是写表里连续的一截
@@ -7343,6 +7386,116 @@ mod tests {
         );
     }
 
+    /// 改法 G4 在第三截的判法（与 C670 那一条同一个接法）：第一个文件之后的盘面上，盘 0 两个系统配置槽各改一个字节（读得出、自证不过，设备不报错），
+    /// 当成一个二次崩溃镜像交给第三截，没崩过的那一次挂载做成过（`once` 有内容）。再挂以「读阶段判完之后系统配置又读不出」拒；崩溃态没注入读故障、
+    /// 那一拒的判据在这份盘面上复算为真（盘 0 一份自证过的系统配置都没有）：I-8.4 照旧记「再挂被拒」不判，G4 那一条判过一次、不合法一次，
+    /// 记一条第三截的发现；判据 N-配置 那两样计数不动。判别力：第三截不认 G4 那一条（`crates/mutations.tsv` 一行），这一条红。
+    #[test]
+    fn a_remount_refused_on_a_second_crash_image_where_device_zero_has_both_system_configuration_slots_torn_is_an_illegal_refusal_after_the_read_stage(
+    ) {
+        /// 撕裂一个系统配置槽时改的那个字节在槽里的位置：槽中间，落在整槽校验和罩着的范围里。
+        const TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT: usize = 2048;
+        let (history, execution) = history_of_the_first_file_and_one_overwrite();
+        let (writes, _segments, first_write_of_the_overwrite) =
+            writes_segments_and_first_write_of_the_overwrite(&history, execution);
+        let mut second_crash_image = MemoryPool::with_devices(
+            &[DeviceIdentity(0), DeviceIdentity(1)],
+            execution.device_width.device_bytes(),
+        );
+        second_crash_image.apply_writes(&writes[..first_write_of_the_overwrite]);
+        let slot_spacing = u64::from(
+            execution
+                .device_width
+                .parameters()
+                .geometry
+                .fixed_structure_slot_spacing,
+        );
+        let slot_bytes = usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES)
+            .expect("系统配置槽 4096 字节装得进 usize");
+        // 迭代上界是一块盘的两个系统配置槽；不跨轮携带状态；没有提前出口。
+        for slot_index in 0..singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
+            let offset = DeviceOffsetInBytes(slot_index * slot_spacing);
+            let mut slot =
+                PoolReader::read(&second_crash_image, DeviceIdentity(0), offset, slot_bytes)
+                    .expect("内存盘上系统配置槽读得出");
+            slot[TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT] ^= 0xFF;
+            second_crash_image
+                .devices
+                .get_mut(&DeviceIdentity(0))
+                .expect("池里有盘 0")
+                .write(offset, &slot);
+        }
+        let recovered =
+            read_recovery_fields(&second_crash_image, ComparedVersion::TheRecoveredVersion);
+        let crash_point = CrashPoint {
+            segment_index: 0,
+            persisted_within_the_segment: vec![false],
+            step: StepPosition::StartingPoint,
+            operation_kind: None,
+        };
+        let mut tally = CrashInjectionTally::default();
+        let mut findings = FindingsOfOneHistory {
+            known_red_hits: Vec::new(),
+            new_findings: Vec::new(),
+        };
+        judge_the_remount_after_a_second_crash(
+            &mut tally,
+            &mut findings,
+            &Ok(content_read_by_the_recovery_report(&recovered.report)),
+            &RefusalOnTheCrashImage {
+                crash_image_materialized: &second_crash_image,
+                version_landed_on_after_the_crash: recovered.report.effective_root,
+                read_only_read_back: ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+                read_faults: ReadFaultsInTheCrashState::NoneInjected,
+            },
+            &execution.device_width.parameters(),
+            execution.space_admission,
+            InstanceTableSweepConfiguration::PRODUCT,
+            WritableMountPhase::RowPublish,
+            history.seed,
+            &crash_point,
+            None,
+        );
+        assert_eq!(
+            tally
+                .remount_idempotence_not_judged
+                .get(&RemountIdempotenceNotJudged::TheRemountWasRefused)
+                .copied(),
+            Some(1),
+            "前提：再挂被拒，I-8.4 不判：{}",
+            tally.render()
+        );
+        assert_eq!(
+            (
+                tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
+                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults
+            ),
+            (0, 0),
+            "前提：拒因不是判据 N-配置：{}",
+            tally.render()
+        );
+        assert_eq!(
+            (
+                tally.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage,
+                tally.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults
+            ),
+            (1, 1),
+            "拒因是读阶段判完之后系统配置又读不出、崩溃态没注入读故障而判据复算为真：判过一次、不合法一次：{}",
+            tally.render()
+        );
+        assert_eq!(
+            findings.known_red_hits.len() + findings.new_findings.len(),
+            1,
+            "第三截记一条：已知红 {:?}，新发现 {:?}",
+            findings.known_red_hits,
+            findings
+                .new_findings
+                .iter()
+                .map(CrashPointFinding::render)
+                .collect::<Vec<_>>()
+        );
+    }
+
     /// C670 在第三截的判别力（攻方改法 P1，`research/prompts/m3-formal-code-b16-r1-main-verification.md` 第二节 K2-b）：起点加一次覆盖写、不重挂，
     /// 短段整段枚举；历史里没有带读故障的步。二次崩溃之后再挂以判据 N-配置 被拒、判据在二次崩溃镜像上复算为真的，一个都没有。
     /// 攻方自造的坏实现 V242u（只在带单元的发布上先轮换后根，`crates/mutations.tsv` 里那一行）在这段历史上第二截看不见（第一次崩溃镜像上 0 个：
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
index f7878c2..391f1d3 100644
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -6,8 +6,9 @@
 //! 或挂载加一次发布——之后再只读恢复一次（R2），逐字段比（[`judge_mount_transparency`]）；挂载途中各相位的二次崩溃镜像上，
 //! 只读恢复读出的与 R1 比（用户 2026-10-02 定 F6「加二次」）；抬 F 的写有持久的那几态 F 不判（[`FloorRaisesInTheMountWrites`]；
 //! 抬 F 的界在挂载写表里对不上位置的那一次挂载，二次崩溃整段 F 不判、按 [`FieldNotJudged`] 计数）。
-//! 可写挂载被拒的状态不判透明，判拒绝本身合不合法（[`judge_writable_mount_refusal`]；崩溃注入第二截再加 C670 那一条：
-//! 没注入读故障的崩溃态上判据 N-配置 复算为真，拒得不合法，[`judge_writable_mount_refusal_of_a_crash_state`]）。
+//! 可写挂载被拒的状态不判透明，判拒绝本身合不合法（[`judge_writable_mount_refusal`]；崩溃注入第二、三截再加两条：
+//! 没注入读故障的崩溃态上判据 N-配置 复算为真（C670），或「读阶段判完之后系统配置又读不出」那一拒的判据复算为真（改法 G4），
+//! 正确实现摆不出这份盘面，拒得不合法，[`judge_writable_mount_refusal_of_a_crash_state`]）。
 //!
 //! 读盘的只有 [`read_recovery_fields`] 与那两个判拒绝的函数里在崩溃镜像上复算拒的判据那一截；[`judge_mount_transparency`]
 //! 只吃读好的字段与挂载交回值。checker 档的崩溃注入调它，harness 档的轻用例也调它（harness 档不依赖 checker 档）。
@@ -2572,6 +2573,10 @@ pub enum RefusalConditionNotMet {
     /// C670：崩溃态没注入读故障，判据 N-配置 在崩溃镜像上（撕裂的系统配置槽跳过）复算仍为真——正确实现的可写挂载在这份盘面上永远拒，
     /// 一次普通断电之后池只剩只读挂载；这样的崩溃态不许出现，拒是对的、造出这份盘面的那一段不对。带的是判据为真的那一支。
     WitnessCriterionTrueOnACrashImageWithoutReadFaults(WitnessedCounterBranchThatHolds),
+    /// 改法 G4（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2「新拒因少一条 C670 同形的判法（J2-b）」那一行）：
+    /// 崩溃态没注入读故障，拒因是「读阶段判完之后系统配置又读不出」，那一拒的判据在崩溃镜像上复算为真（某块盘一份本池自证过的系统配置都没有、
+    /// 一份都择不到，或见证判不出）——没有读故障的历史留不下这份盘面，正确实现在它上面永远拒可写；与 C670 那一条同形，拒是对的、造出这份盘面的那一段不对。
+    SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults,
 }
 
 /// 可写挂载被拒，拒得合不合法（r5 判决第二节 U6：拒绝合法 ⇔ 理由在 D23 已定项 14 那张单子上、那一条判据在崩溃镜像上复算为真、
@@ -2587,7 +2592,7 @@ pub enum WritableMountRefusalVerdict {
     ReasonNotOnTheD23List,
 }
 
-/// 判一次拒可写合不合法，只判 r5 判决 U6 那四条，不判 C670 那一条（崩溃态知道注没注入读故障的调用方用
+/// 判一次拒可写合不合法，只判 r5 判决 U6 那四条，不判 C670 与改法 G4 那两条（崩溃态知道注没注入读故障的调用方用
 /// [`judge_writable_mount_refusal_of_a_crash_state`]）。拒的判据在崩溃镜像（`crash_image`）上自己复算（不经 O1 判官，也不经实现判拒的那一段）；
 /// `selected_version` 是崩溃镜像上只读恢复落到的那一版（R1 恢复报告里的 `effective_root`）。
 #[must_use]
@@ -2636,10 +2641,12 @@ pub fn judge_writable_mount_refusal_of_a_crash_state(
     )
 }
 
-/// 判崩溃注入第二截的一次拒可写合不合法：U6 那四条（[`judge_writable_mount_refusal`]），再加 C670 那一条——崩溃态没注入读故障
+/// 判崩溃注入第二、三截的一次拒可写合不合法：U6 那四条（[`judge_writable_mount_refusal`]），再加 C670 那一条——崩溃态没注入读故障
 /// （`read_faults`）、理由是判据 N-配置，而判据在崩溃镜像上复算、撕裂的系统配置槽跳过之后仍为真（[`WitnessedCounterBranchThatHolds`]），
-/// 拒得不合法（[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]）。注入过读故障的崩溃态不判这一条
-/// （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。理由是「读阶段判完之后系统配置又读不出」而被拒的那次挂载自己在读故障下跑的
+/// 拒得不合法（[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]）；与它同形的改法 G4——崩溃态没注入读故障、
+/// 理由是「读阶段判完之后系统配置又读不出」，而那一拒的判据在崩溃镜像上复算为真，拒得不合法
+/// （[`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]）。注入过读故障的崩溃态
+/// 这两条都不判（D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。理由是「读阶段判完之后系统配置又读不出」而被拒的那次挂载自己在读故障下跑的
 /// （`read_faults_under_the_refused_mount`），判据在崩溃镜像上复算为假不算不合法（两遍读之间的瞬时读错复算不出，C664（挂载拒绝合法复算把两种拒因
 /// 报成一个））；被拒的那次挂载不带故障层的，崩溃态注没注入过读故障都照 U6 复算（判据为假即不合法，改法 G3）。
 #[must_use]
@@ -2678,13 +2685,25 @@ pub fn judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_und
                 );
             }
         }
+        // 改法 G4：没有读故障时设备不报读错，某块盘两槽都自证不过只能是盘面本身如此——正确实现在没有读故障的历史里留不下它。
         (
-            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
+            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
+            ReadFaultsInTheCrashState::NoneInjected,
+        ) => {
+            if system_configuration_unreadable_after_the_read_stage_on_the_crash_image(crash_image)
+            {
+                conditions_not_met.push(
+                    RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults,
+                );
+            }
+        }
+        (
+            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
+            | RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
             ReadFaultsInTheCrashState::Injected,
         )
         | (
-            RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable
-            | RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
+            RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
             ReadFaultsInTheCrashState::NoneInjected | ReadFaultsInTheCrashState::Injected,
         ) => {}
     }
diff --git a/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs b/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
index 4ed6d1c..74dda5f 100644
--- a/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
+++ b/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
@@ -1,12 +1,16 @@
 //! C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）：崩溃注入第二截判拒可写合不合法时，崩溃态没注入读故障、拒的理由是
 //! D23（journal 的角色与格式） 已定项 14 的判据 N-配置，而判据在崩溃镜像上复算（撕裂的系统配置槽跳过）仍为真 ⇒ 拒得不合法
 //! （`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state`）。注入过读故障的崩溃态不判这一条
-//! （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。某块盘两槽都撕裂那一支（一份自证过的系统配置都没有）：C664（挂载拒绝合法复算
-//! 把两种拒因报成一个）之后实现报的是「读阶段判完之后系统配置又读不出」那一拒，不是判据 N-配置，C670 那一条不判；拒照 U6 判——系统配置见证照 R2
-//! 判得出（撕裂不算判不出），那一拒的判据按「某块盘一份自证过的都没有」为真，合法。
+//! （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。
+//! 改法 G4（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2「新拒因少一条 C670 同形的判法（J2-b）」那一行）与它同形：
+//! 某块盘一份自证过的系统配置都没有时，C664（挂载拒绝合法复算把两种拒因报成一个）之后实现报的是「读阶段判完之后系统配置又读不出」那一拒，
+//! 不是判据 N-配置；崩溃态没注入读故障而那一拒的判据在崩溃镜像上复算为真（系统配置见证照 R2 判得出、撕裂不算判不出，判据按「某块盘一份自证过的都没有」为真）
+//! ⇒ 没有读故障的历史留不下这份盘面，拒得不合法。注入过读故障的崩溃态不判这一条，那一拒照 U6 判，判据为真即合法。
 //! 小池是单元区 384 槽的两块内存盘，第一个文件之后的盘面；崩溃镜像是内存盘面本身，读不报错。
 //! 见证判据为真的那份盘面取法照 E171（三oracle合谋测试） 第 242 行（系统配置轮换先于根槽 FUA 落盘）：一次覆盖写那次发布只落了系统配置轮换，
-//! 它的单元、记录与根都没落。
+//! 它的单元、记录与根都没落。盘 0 两槽都自证不过的盘面有两份：一份是手工各改一字节（静默损坏，不是崩溃摆出来的）；
+//! 一份是攻方坏实现 MUT-B1（`research/prompts/m3-formal-code-b26-r1-opus-model/tools/apply_mut_b.py`：盘 0 轮换写完新槽之后把另一槽清成全 0、
+//! 与新槽同在一段里）在 harness 档能摆的最小形状——这次轮换在盘 0 上写到一半（撕裂）、另一槽清零落了。
 
 use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
 use singlefs_core::admission::SpaceAdmission;
@@ -25,10 +29,10 @@ use singlefs_harness::memory_pool::{
     writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
 };
 use singlefs_harness::mount_transparency::{
-    judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields, ComparedVersion,
-    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage,
-    RefusalConditionNotMet, RefusalReasonOnTheD23List, WitnessedCounterBranchThatHolds,
-    WritableMountRefusalVerdict,
+    judge_writable_mount_refusal, judge_writable_mount_refusal_of_a_crash_state,
+    read_recovery_fields, ComparedVersion, DiskAfterTheRefusal, ReadFaultsInTheCrashState,
+    ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet, RefusalReasonOnTheD23List,
+    WitnessedCounterBranchThatHolds, WritableMountRefusalVerdict,
 };
 use singlefs_harness::segments::{FixedGeometry, StepKind};
 use singlefs_harness::{RetainedOperation, SharedStream};
@@ -75,9 +79,8 @@ fn pool_of(operations: &[RetainedOperation]) -> MemoryPool {
     pool
 }
 
-/// 第一个文件之后的盘面上，只落了一次覆盖写那次发布的系统配置轮换（每块盘一槽，journal tail 写的是这次发布末条记录的计数器），
-/// 它的单元、记录与根都没落：系统配置见证过一次比盘上任何一版都新的发布。崩溃镜像读不报错。
-fn crash_image_where_only_the_rotation_of_an_overwrite_landed() -> (MemoryPool, MemoryPool) {
+/// 第一个文件之后的盘面，与在它上面跑一次覆盖写时那次发布的系统配置轮换（每块盘一槽，journal tail 写的是这次发布末条记录的计数器）。
+fn pool_after_the_first_file_and_rotation_of_an_overwrite() -> (MemoryPool, Vec<RetainedWrite>) {
     let first_file = recorded_stream_after_the_first_file_then(Vec::new());
     let overwrite =
         recorded_stream_after_the_first_file_then(vec![HistoryOperation::PublishOverwrite(
@@ -116,14 +119,90 @@ fn crash_image_where_only_the_rotation_of_an_overwrite_landed() -> (MemoryPool,
             && writes_of_the_overwrite
                 .iter()
                 .any(|write| write.kind == StepKind::JournalRecord),
-        "前提：覆盖写那次发布写了记录与根（这份盘面上都不落）"
+        "前提：覆盖写那次发布写了记录与根（摆出来的盘面上都不落）"
     );
-    let healthy = pool_of(&first_file);
+    (pool_of(&first_file), rotation)
+}
+
+/// 第一个文件之后的盘面上，只落了一次覆盖写那次发布的系统配置轮换，它的单元、记录与根都没落：系统配置见证过一次比盘上任何一版都新的发布。
+/// 崩溃镜像读不报错。
+fn crash_image_where_only_the_rotation_of_an_overwrite_landed() -> (MemoryPool, MemoryPool) {
+    let (healthy, rotation) = pool_after_the_first_file_and_rotation_of_an_overwrite();
     let mut crash_image = healthy.clone();
     crash_image.apply_writes(&rotation);
     (healthy, crash_image)
 }
 
+/// 原地覆写写到一半的那份字节：旧字节与新字节不同的那一截，前一半取新、后一半取旧（与层 0 撕裂第三态同一个取法，
+/// checker 档 `crash::torn_image_of_in_place_overwrite`；harness 档不依赖 checker 档，这里留一份）。新旧逐字节相同时交回新字节。
+fn torn_image_of_an_in_place_overwrite(old: &[u8], new: &[u8]) -> Vec<u8> {
+    assert_eq!(old.len(), new.len(), "旧字节与这次写一样长");
+    let differs = |index: &usize| old[*index] != new[*index];
+    let Some(first_differing_byte) = (0..new.len()).find(differs) else {
+        return new.to_vec();
+    };
+    let last_differing_byte = (0..new.len())
+        .rev()
+        .find(differs)
+        .expect("有第一个不同的字节就有最后一个");
+    let differing_span_in_bytes = last_differing_byte - first_differing_byte + 1;
+    let first_old_byte = first_differing_byte + differing_span_in_bytes / 2;
+    let mut torn = new.to_vec();
+    torn[first_old_byte..].copy_from_slice(&old[first_old_byte..]);
+    torn
+}
+
+/// 攻方坏实现 MUT-B1 在 harness 档能摆的最小形状：第一个文件之后的盘面上，覆盖写那次发布在盘 0 的轮换写到一半（撕裂，旧槽内容也没了），
+/// 坏实现紧跟着把盘 0 另一槽清成全 0 的那次写落了（与轮换同在一段里，段内真子集摆得出）；盘 1 不动。盘 0 一份自证过的系统配置都没有，
+/// 设备不报错。
+fn crash_image_where_a_rotation_tore_and_the_other_slot_of_device_zero_was_zeroed(
+) -> (MemoryPool, MemoryPool) {
+    let (healthy, rotation) = pool_after_the_first_file_and_rotation_of_an_overwrite();
+    let rotation_on_device_zero = rotation
+        .iter()
+        .find(|write| write.device == DeviceIdentity(0))
+        .expect("前提：覆盖写那次发布在盘 0 轮换一槽");
+    let slot_spacing = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
+    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
+    assert_eq!(
+        rotation_on_device_zero.length_in_bytes(),
+        SYSTEM_CONFIGURATION_SLOT_BYTES,
+        "前提：轮换那次写正好一个系统配置槽"
+    );
+    let rotated_slot_index = rotation_on_device_zero.offset.0 / slot_spacing;
+    assert!(
+        rotated_slot_index < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
+            && rotation_on_device_zero.offset.0 % slot_spacing == 0,
+        "前提：轮换写在盘 0 的一个系统配置槽上：{:?}",
+        rotation_on_device_zero.offset
+    );
+    let other_slot_offset = DeviceOffsetInBytes(
+        ((rotated_slot_index + 1) % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * slot_spacing,
+    );
+    let old_bytes = healthy
+        .read(
+            DeviceIdentity(0),
+            rotation_on_device_zero.offset,
+            slot_bytes,
+        )
+        .expect("内存盘上系统配置槽读得出");
+    let mut new_bytes = vec![0u8; slot_bytes];
+    rotation_on_device_zero
+        .contents
+        .copy_range_into(0, &mut new_bytes);
+    let mut crash_image = healthy.clone();
+    let device_zero = crash_image
+        .devices
+        .get_mut(&DeviceIdentity(0))
+        .expect("有盘 0");
+    device_zero.write(
+        rotation_on_device_zero.offset,
+        &torn_image_of_an_in_place_overwrite(&old_bytes, &new_bytes),
+    );
+    device_zero.write(other_slot_offset, &vec![0u8; slot_bytes]);
+    (healthy, crash_image)
+}
+
 /// 第一个文件之后的盘面上，盘 0 两个系统配置槽各改一个字节（读得出、自证不过，设备不报错）：撕裂的系统配置，盘 0 一份自证过的都没有。
 fn crash_image_where_both_system_configuration_slots_of_device_zero_are_torn(
 ) -> (MemoryPool, MemoryPool) {
@@ -285,19 +364,11 @@ fn the_same_witness_refusal_in_a_crash_state_with_read_faults_injected_is_legal(
     );
 }
 
-/// 撕裂系统配置那一支：盘 0 两槽都撕裂、没注入读故障，实现在取号之前的逐盘核以「这块盘一份自证过的系统配置都没有」拒
-/// （`MountError::SystemConfigurationUnreadableAfterTheReadStage`，C664（挂载拒绝合法复算把两种拒因报成一个）之后不再报成判据 N-配置 的拒因，
-/// C670 那一条只管 N-配置、不判它）。U6 那四条里那一拒的判据在崩溃镜像上复算：R2 之下撕裂的槽不算判不出（没有槽设备报读错），
-/// 盘 0 一份自证过的系统配置都没有 ⇒ 真：合法。
-#[test]
-fn a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system_configuration_slots_torn_is_legal(
-) {
-    let (healthy, crash_image) =
-        crash_image_where_both_system_configuration_slots_of_device_zero_are_torn();
-    let (error, after) = refused_writable_mount_on(&crash_image);
+/// 前提：盘 0 一份自证过的系统配置都没有，取号之前的逐盘核以读阶段判完之后系统配置又读不出拒（C664 之后不报成判据 N-配置 的拒因）。
+fn assert_refused_after_the_read_stage_for_device_zero(error: &MountError) {
     assert!(
         matches!(
-            &error,
+            error,
             MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable)
                 if unreadable.unreadable
                     == SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
@@ -306,6 +377,18 @@ fn a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system
         ),
         "前提：盘 0 一份自证过的系统配置都没有，取号之前的逐盘核以读阶段之后系统配置又读不出拒：{error:?}"
     );
+}
+
+/// 手工撕裂的系统配置（静默损坏，不是崩溃摆出来的）：盘 0 两槽各改一字节、没注入读故障，实现以「读阶段判完之后系统配置又读不出」拒。
+/// 那一拒的判据在崩溃镜像上复算为真（R2 之下撕裂的槽不算判不出，盘 0 一份自证过的都没有），U6 四条都满足；改法 G4：没注入读故障的崩溃态上
+/// 判据为真，正确实现摆不出这份盘面，拒得不合法。
+#[test]
+fn a_refusal_after_the_read_stage_on_a_crash_image_without_read_faults_where_device_zero_has_both_system_configuration_slots_torn_is_illegal(
+) {
+    let (healthy, crash_image) =
+        crash_image_where_both_system_configuration_slots_of_device_zero_are_torn();
+    let (error, after) = refused_writable_mount_on(&crash_image);
+    assert_refused_after_the_read_stage_for_device_zero(&error);
     let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
     assert_eq!(
         judge_writable_mount_refusal_of_a_crash_state(
@@ -316,8 +399,77 @@ fn a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system
             read_only_read_back_against_the_first_file(&healthy, &crash_image),
             ReadFaultsInTheCrashState::NoneInjected,
         ),
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
+            conditions_not_met: vec![
+                RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults,
+            ],
+        }
+    );
+}
+
+/// 同一份手工撕裂的盘面，调用方说这个崩溃态注入过读故障：G4 那一条不判，那一拒照 U6 判——判据在崩溃镜像上复算为真、盘没变、
+/// 读出在允许集合里，拒本身成立，合法。
+#[test]
+fn the_same_refusal_after_the_read_stage_in_a_crash_state_with_read_faults_injected_is_legal() {
+    let (healthy, crash_image) =
+        crash_image_where_both_system_configuration_slots_of_device_zero_are_torn();
+    let (error, after) = refused_writable_mount_on(&crash_image);
+    assert_refused_after_the_read_stage_for_device_zero(&error);
+    let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state(
+            &crash_image,
+            fields.report.effective_root,
+            &error,
+            disk_after(&crash_image, &after),
+            read_only_read_back_against_the_first_file(&healthy, &crash_image),
+            ReadFaultsInTheCrashState::Injected,
+        ),
         WritableMountRefusalVerdict::Legal(
             RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
         )
     );
 }
+
+/// 攻方坏实现 MUT-B1 的最小形状（盘 0 轮换撕裂、另一槽清零落了），没注入读故障：实现以「读阶段判完之后系统配置又读不出」拒。
+/// 只判 U6 四条的判官（攻方量的那一版第二截）判它合法；改法 G4 之后，崩溃态判官判它拒得不合法、只差 G4 那一条。
+#[test]
+fn a_refusal_after_the_read_stage_without_read_faults_where_a_rotation_tore_and_the_other_slot_of_device_zero_was_zeroed_turns_from_legal_to_illegal(
+) {
+    let (healthy, crash_image) =
+        crash_image_where_a_rotation_tore_and_the_other_slot_of_device_zero_was_zeroed();
+    let (error, after) = refused_writable_mount_on(&crash_image);
+    assert_refused_after_the_read_stage_for_device_zero(&error);
+    let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
+    let read_only_read_back = read_only_read_back_against_the_first_file(&healthy, &crash_image);
+    assert_eq!(
+        judge_writable_mount_refusal(
+            &crash_image,
+            fields.report.effective_root,
+            &error,
+            disk_after(&crash_image, &after),
+            read_only_read_back,
+        ),
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
+        ),
+        "前提：只判 U6 四条时这一拒合法（判据复算为真、盘没变、读出在允许集合里）"
+    );
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state(
+            &crash_image,
+            fields.report.effective_root,
+            &error,
+            disk_after(&crash_image, &after),
+            read_only_read_back,
+            ReadFaultsInTheCrashState::NoneInjected,
+        ),
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
+            conditions_not_met: vec![
+                RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults,
+            ],
+        }
+    );
+}
````

### mutations-append.tsv

````text
第29批 G4 判官整条不判：没注入读故障的崩溃态上「读阶段判完之后系统配置又读不出」那一拒的判据复算为真不报（MUT-B1 最小形状）	crates/singlefs-harness/src/mount_transparency.rs	                conditions_not_met.push(\n                    RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults,\n                );	                let _ = RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults;	-p singlefs-harness --test writable_mount_refusal_on_a_crash_state_without_read_faults	a_refusal_after_the_read_stage_without_read_faults_where_a_rotation_tore_and_the_other_slot_of_device_zero_was_zeroed_turns_from_legal_to_illegal
第29批 G4 判官整条不判：没注入读故障的崩溃态上那一拒的判据复算为真不报（手工两槽撕裂那一形）	crates/singlefs-harness/src/mount_transparency.rs	                conditions_not_met.push(\n                    RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults,\n                );	                let _ = RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults;	-p singlefs-harness --test writable_mount_refusal_on_a_crash_state_without_read_faults	a_refusal_after_the_read_stage_on_a_crash_image_without_read_faults_where_device_zero_has_both_system_configuration_slots_torn_is_illegal
第29批 G4 注入过读故障的崩溃态也判新拒因的判据（越出挂载透明只管不带故障的状态那条射程）	crates/singlefs-harness/src/mount_transparency.rs	            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,\n            ReadFaultsInTheCrashState::NoneInjected,\n        ) => {	            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,\n            ReadFaultsInTheCrashState::NoneInjected | ReadFaultsInTheCrashState::Injected,\n        ) => {	-p singlefs-harness --test writable_mount_refusal_on_a_crash_state_without_read_faults	the_same_refusal_after_the_read_stage_in_a_crash_state_with_read_faults_injected_is_legal
第29批 G4 只判第二截不判第三截：二次崩溃镜像上再挂以新拒因被拒、判据复算为真不报红	crates/singlefs-checker-tier/src/crash_injection.rs	            RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults => {\n                tally.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults += 1;\n                criterion_true_without_read_faults = true;\n            }	            RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults => {}	-p singlefs-checker-tier --lib -- crash_injection::tests::a_remount_refused_on_a_second_crash_image_where_device_zero_has_both_system_configuration_slots_torn_is_an_illegal_refusal_after_the_read_stage	a_remount_refused_on_a_second_crash_image_where_device_zero_has_both_system_configuration_slots_torn_is_an_illegal_refusal_after_the_read_stage
````

### mutations-replacements.tsv

````text
C670（16-3）：撕裂系统配置那一支没分开（按甲把自证不过的槽也当见证判据为真，撕裂的拒一起判不合法）	crates/singlefs-harness/src/mount_transparency.rs	            if let Some(branch) = witnessed_counter_holds_on_a_crash_image_without_read_faults(\n                crash_image,\n                selected_version,\n            ) {	            if let Some(branch) = newer_publish_witnessed_on_the_crash_image(\n                crash_image,\n                selected_version,\n            )\n            .then_some(WitnessedCounterBranchThatHolds::NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable) {	-p singlefs-harness --test writable_mount_refusal_on_a_crash_state_without_read_faults	a_witness_refusal_on_a_crash_image_without_read_faults_where_only_a_rotation_landed_is_illegal
C670（18-1 P1）第三截：再挂被拒那一支只记 I-8.4 没判、不判 C670 那一条	crates/singlefs-checker-tier/src/crash_injection.rs	                .entry(RemountIdempotenceNotJudged::TheRemountWasRefused)\n                .or_insert(0) += 1;\n            judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_without_read_faults(	                .entry(RemountIdempotenceNotJudged::TheRemountWasRefused)\n                .or_insert(0) += 1;\n            if tally.remounts_after_a_second_crash > 0 {\n                return;\n            }\n            judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_without_read_faults(	-p singlefs-checker-tier --lib -- crash_injection::tests::a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal	a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal
````

## m3-formal-29-2

### crates.patch

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index 8caab55..6330e9c 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -1581,6 +1581,7 @@ fn w_walk_instance_table_chain(first: Ptr, judge_rows: bool, fsid_low: vec2<u32>
     for (var page = 0u; page < 256u; page = page + 1u) {
         w_judge_location_order(pointer);
         w_judge_pointer_head(pointer);
+        w_judge_pointer_extent_offset(pointer);
         w_note_pointer_references(pointer, 2u);
         let s = w_read_referenced_unit(pointer, DATA_UNIT_BYTES);
         if (s.version == NONE) {
@@ -1765,6 +1766,7 @@ fn w_walk_extent_data_pointer(base: u32, o: u32, inode: vec2<u32>, offset: vec2<
     let p = parse_data_pointer(base, o);
     w_judge_location_order(p);
     w_judge_pointer_head(p);
+    w_judge_pointer_extent_offset(p);
     w_note_pointer_references(p, 2u);
     let s = w_read_referenced_unit(p, DATA_UNIT_BYTES);
     if (s.version == NONE) {
@@ -2590,6 +2592,7 @@ fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: v
         let child = parse_node_pointer(root.base, o + 34u);
         w_judge_location_order(child);
         w_judge_pointer_head(child);
+        w_judge_pointer_extent_offset(child);
         var length = NODE_BYTES;
         var span = 1u;
         if (record_type == 2u) {
diff --git a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
index 7b0f5a9..5b982ae 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -497,6 +497,10 @@ fn device_bytes() -> vec2<u32> {
 // 指针头部 extent 偏移那 2 字节在指针里的偏移（D19（块指针的结构与宽度预算） 已定项 7 的字段表：MAC 16、nonce 12、算法类型 1、压缩算法码 1、
 // 压后长度 2 之后；`singlefs_checker::image` 的 POINTER_EXTENT_OFFSET_FIELD 是 32..34）。第一版指针指整个单元，这 2 字节恒 0
 const POINTER_EXTENT_OFFSET_FIELD_OFFSET: u32 = 32u;
+// 偏移 o 起那条指针头部的 extent 偏移为 0：恢复内核的读者只要这一判时不解整条指针
+fn pointer_extent_offset_is_zero(base: u32, o: u32) -> bool {
+    return rd16(base, o + POINTER_EXTENT_OFFSET_FIELD_OFFSET) == 0u;
+}
 
 struct Ptr {
     all_zero: bool,
@@ -519,7 +523,7 @@ fn parse_pointer_common(base: u32, o: u32) -> Ptr {
     p.all_zero = false;
     p.mac_nonce_zero = all_zero(base, o, 28u);
     p.head_ok = p.mac_nonce_zero && rd8(base, o + 28u) == 0u && rd8(base, o + 29u) == 0u && rd16(base, o + 30u) == 0u;
-    p.extent_offset_zero = rd16(base, o + POINTER_EXTENT_OFFSET_FIELD_OFFSET) == 0u;
+    p.extent_offset_zero = pointer_extent_offset_is_zero(base, o);
     p.birth_tree = rd64(base, o + 34u);
     p.birth_txg = rd64(base, o + 42u);
     p.device0 = rd32(base, o + 50u);
@@ -902,6 +906,13 @@ fn parse_root(s: Slice, fsid: vec4<u32>) -> RootFields {
     r.allocation_root = parse_node_pointer(base, 342u);
     return r;
 }
+// root_record.rs 的 parse_slot 第五关：树表、实例表、中央映射树、分配记录树四条指针头部的 extent 偏移任一非 0，这条根记录损坏、
+// 与自证不过同一个处置，择根时不可择。只在恢复择根时判：池级 checker 的根环读法（image::valid_roots，走读内核 checker_collect_roots）不判它，
+// 那里红 I-2.4 的是走读跟随这几条指针的那几处
+fn root_record_pointers_carry_no_extent_offset(r: RootFields) -> bool {
+    return r.tree_table.extent_offset_zero && r.instance_table.extent_offset_zero
+        && r.mapping_root.extent_offset_zero && r.allocation_root.extent_offset_zero;
+}
 fn region_device(cfg: SysConfig, region: u32) -> u32 {
     if (region == 0u) {
         return cfg.region0;
@@ -915,7 +926,7 @@ struct RootChoice {
     found: bool,
     root: RootFields,
 };
-// 三个区域全部槽逐个验自证，取 (txg, 实例) 最大的；每条自证过的根记进草稿（版本、区域）
+// 三个区域全部槽逐个验自证（连同四条指针的 extent 偏移），取 (txg, 实例) 最大的；每条自证过的根记进草稿（版本、区域）
 fn choose_root(cfg: SysConfig) -> RootChoice {
     var out: RootChoice;
     out.found = false;
@@ -927,7 +938,7 @@ fn choose_root(cfg: SysConfig) -> RootChoice {
             let offset = u64_of(ROOT_RING_BASE_BYTES + region * ROOT_RING_REGION_STRIDE + slot * cfg.spacing);
             let s = read_at(device, offset, cfg.physical_block_size);
             let candidate = parse_root(s, cfg.fsid);
-            if (!candidate.valid) {
+            if (!candidate.valid || !root_record_pointers_carry_no_extent_offset(candidate)) {
                 continue;
             }
             if (root_count < MAX_ROOTS) {
@@ -1924,7 +1935,8 @@ fn read_code_two_tree(root_pointer: Ptr, root_pointer_base: u32, root_pointer_of
         let e = code2_frames[top].child_index;
         let separator_offset = entry_offset(n, e);
         let child_pointer = parse_node_pointer(n.base, separator_offset + kw);
-        if (!child_pointer.head_ok) {
+        // code_two_tree::parse_internal_entry：子指针头部加密、压缩那几段与 extent 偏移都要是第一版的取值
+        if (!child_pointer.head_ok || !child_pointer.extent_offset_zero) {
             return out;
         }
         if (code2_frames[top].has_previous_separator) {
@@ -2206,6 +2218,10 @@ fn read_allocation_tree(root_pointer: Ptr, root_pointer_base: u32, root_pointer_
         allocation_frames[top].previous_child_device = device;
         allocation_frames[top].previous_child_index = child.index;
         pending_pointer = parse_node_pointer(n.base, o + 10u);
+        // allocation_record_tree 读内部条目的子指针：头部 extent 偏移非 0，这个节点损坏
+        if (!pending_pointer.extent_offset_zero) {
+            return out;
+        }
         pending_pointer_base = n.base;
         pending_pointer_offset = o + 10u;
         pending_node = child;
@@ -2248,6 +2264,16 @@ fn extent_key_offset(base: u32, o: u32) -> vec2<u32> { return rd64(base, o + 16u
 fn extent_key_equals(base: u32, o: u32, inode: vec2<u32>, offset: vec2<u32>) -> bool {
     return is_zero64(rd64(base, o)) && eq64(rd64(base, o + 8u), inode) && eq64(rd64(base, o + 16u), offset);
 }
+// extent_tree 的 internal_entries：一个内部节点的每条子指针（条目 key 24 之后）头部 extent 偏移都为 0；
+// core 在往下读第一个孩子之前就把整个节点的条目解完，有一条非 0 这个节点就损坏
+fn extent_internal_child_pointers_carry_no_extent_offset(n: Node) -> bool {
+    for (var e = 0u; e < n.entry_count; e = e + 1u) {
+        if (!pointer_extent_offset_is_zero(n.base, entry_offset(n, e) + 24u)) {
+            return false;
+        }
+    }
+    return true;
+}
 struct ExtentFrame {
     n: Node,
     level: u32,
@@ -2341,7 +2367,16 @@ fn read_upper_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_of
                         if (!all_zero(n.base, o + 25u + 86u, 2u)) {
                             return out;
                         }
-                    } else if (tag != 2u) {
+                        // ExtentUpperLeafEntry::parse：下段根指针头部 extent 偏移非 0，这条条目损坏
+                        if (!pointer_extent_offset_is_zero(n.base, o + 25u)) {
+                            return out;
+                        }
+                    } else if (tag == 2u) {
+                        // 同上：内联的数据指针
+                        if (!pointer_extent_offset_is_zero(n.base, o + 25u)) {
+                            return out;
+                        }
+                    } else {
                         return out;
                     }
                     if (lt64(inode, first_inode) || lt64(last_inode, inode)) {
@@ -2370,6 +2405,9 @@ fn read_upper_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_of
             if (n.entry_width < 110u) {
                 return out;
             }
+            if (!extent_internal_child_pointers_carry_no_extent_offset(n)) {
+                return out;
+            }
             if (depth >= MAX_FRAMES) {
                 undecidable = UNDECIDABLE_TOO_DEEP;
                 return out;
@@ -2479,6 +2517,10 @@ fn read_lower_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_of
                     if (!is_zero64(extent_key_locality(n.base, o)) || !eq64(extent_key_inode(n.base, o), inode) || !is_zero64(qr.remainder)) {
                         return out;
                     }
+                    // records::parse_extent_record：数据指针头部 extent 偏移非 0，这条叶记录损坏
+                    if (!pointer_extent_offset_is_zero(n.base, o + 24u)) {
+                        return out;
+                    }
                     let unit = qr.quotient;
                     if (lt64(unit, first_unit) || lt64(last_unit, unit)) {
                         return out;
@@ -2511,6 +2553,9 @@ fn read_lower_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_of
             if (n.entry_width < 110u) {
                 return out;
             }
+            if (!extent_internal_child_pointers_carry_no_extent_offset(n)) {
+                return out;
+            }
             if (depth >= MAX_FRAMES) {
                 undecidable = UNDECIDABLE_TOO_DEEP;
                 return out;
@@ -2657,7 +2702,8 @@ fn parse_instance_table_page(p: Packed, page_index: vec2<u32>) -> PageParse {
         }
         out.verdict = 0u;
     } else if (flag == 1u) {
-        if (pointer_zero) {
+        // InstanceTableChainRecord::parse（走读那一种判）：链指针头部 extent 偏移非 0，这一片损坏
+        if (pointer_zero || !pointer_extent_offset_is_zero(p.base, last + 2u)) {
             return out;
         }
         out.verdict = 1u;
@@ -2893,6 +2939,10 @@ fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
         if (rd16(tree_table.base, o + 8u) != 200u || rd16(tree_table.base, o + 12u) != 0u || !all_zero(tree_table.base, o + 124u, 76u)) {
             return walk_failed(23u);
         }
+        // TreeTableEntry::parse：树根指针头部 extent 偏移非 0，这条条目损坏（与条目长度、flags、预留 76 同在读任何一棵树之前拒）
+        if (!pointer_extent_offset_is_zero(tree_table.base, o + 14u)) {
+            return walk_failed(63u);
+        }
         let kind = rd16(tree_table.base, o + 10u);
         if (kind < 32u) {
             if ((kinds_seen & (1u << kind)) != 0u) {
@@ -3130,6 +3180,10 @@ fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
             return walk_failed(46u);
         }
         let child = parse_node_pointer(inode_root.base, o + 34u);
+        // records::parse_inode_internal_entry_or_refusal：子指针头部 extent 偏移非 0，这条内部条目损坏
+        if (!child.extent_offset_zero) {
+            return walk_failed(64u);
+        }
         let m = read_via_hint_then_mapping(child, inode_root.base, o + 34u, 3u, DATA_UNIT_BYTES);
         if (m.failed) {
             return walk_failed(47u);
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 69c9eb5..e94f67e 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -11,8 +11,9 @@
 //! 里程碑三·3 删除线（I-9.5、I-3.4、I-4.7、I-4.6、I-9.6 墓碑那一半）：删除中间态（unlink 意图与第一批）一条流钉四条判过而成立，
 //! 每条各一条只改坏它管的那个字段的流钉 CPU 红、GPU 逐位跟上；走过 nlink = 0 那几版的流只比池级 checker 那一半（恢复那一半今天对不上，见
 //! `compare_the_checker_on_a_deletion_flow`）。
-//! I-9.16 ③ ④（种类重复、缺 deadlist）、根槽 428..457（I-2.4）、带树一版根记录里的分配记录树根（I-5.4）各一条坏流整份比；码 2 节点指针头部的
-//! extent 偏移（I-2.4）一条只比池级 checker 那一半（恢复那一半今天对不上，见 `record_new_pool_with_an_extent_offset_on_the_inode_tree_root_pointer`）。
+//! I-9.16 ③ ④（种类重复、缺 deadlist）、根槽 428..457（I-2.4）、带树一版根记录里的分配记录树根（I-5.4）各一条坏流整份比；指针头部的
+//! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
+//! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）。
 //! 要本机一张空闲显存够的 NVIDIA 独显：本机一张 NVIDIA 独显适配器都没有时用卡的用例打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、
 //! 返回成功；有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）。要 `verdict-store,gpu` 特性；
 //! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（没设不定额度）、
@@ -4188,8 +4189,8 @@ fn record_new_pool_with_the_allocation_record_tree_root_of_the_root_record_not_z
 }
 
 /// I-2.4 指针那一半的 extent 偏移那一格：发布那张树表里 inode 树那条（种类 2）的根指针头部 extent 偏移写 1、重封到根。
-/// 池级 checker 读码 2 节点（`read_index_node`）时判它，CPU 与 GPU 走读内核都红 I-2.4。core 读树表条目时判这 2 字节、把那一条当损坏拒，
-/// GPU 恢复内核读树时不判（它只在记录新根段四条指针上判 extent 偏移）：恢复那一半今天对不上，这条流只比池级 checker 那一半。
+/// 池级 checker 读码 2 节点（`read_index_node`）时判它，CPU 与 GPU 走读内核都红 I-2.4。core 读树表条目时判这 2 字节、把那一条当损坏拒
+/// （`TreeTableEntry::parse`），GPU 恢复内核在 `walk_to_file` 解树表条目那一处同拒：这一版两遍恢复都是读失败。
 fn record_new_pool_with_an_extent_offset_on_the_inode_tree_root_pointer(tag: &str) -> RecordedFlow {
     record_new_pool_with_a_resealed_field(
         tag,
@@ -4210,36 +4211,216 @@ fn record_new_pool_with_an_extent_offset_on_the_inode_tree_root_pointer(tag: &st
     )
 }
 
-/// GPU 与 CPU 池级 checker 在这条流的每个状态上违例位逐位相同、一个状态都不许判不了，CPU 池级 checker 在这条流上红 `invariant`；
-/// 恢复那一半不比（调用方写明为什么）。本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
-fn assert_the_gpu_checker_agrees_with_the_cpu_on_a_flow_reddening(
-    test_name: &str,
-    flow: &RecordedFlow,
-    invariant: &'static str,
-) {
-    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
-        return;
-    };
-    let comparison = compare_on_every_state(&card, flow, expand_up_to());
-    assert!(
-        comparison.cpu_checker_red_invariants.contains(invariant),
-        "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
-        flow.name,
-        comparison.cpu_checker_red_invariants
+// ───────────── 第 29 批 29-2：指针头部 extent 偏移，恢复那一半照 core 判、池级 checker 那一半补三处 ─────────────
+
+/// inode 内部条目里子指针的偏移（D8（核心索引结构） 的字段表：分隔 key 8、身份引用 26 之后）。
+const INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET: usize = 34;
+/// extent 树的 key 宽（locality 8、inode 8、文件偏移 8）。
+const EXTENT_KEY_BYTES: usize = 24;
+/// extent 上段叶条目宽：key 24、标签 1、指针 88。
+const EXTENT_UPPER_LEAF_ENTRY_BYTES: u16 = 113;
+/// extent 下段叶记录宽：key 24、数据指针 88。
+const EXTENT_LOWER_LEAF_RECORD_BYTES: u16 = 112;
+/// extent 上段叶条目的标签：1 指一个下段根、2 内联一个数据指针（标签之后就是那条指针）。
+const EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT: u8 = 1;
+const EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT: u8 = 2;
+/// 分配记录树的 key 宽（盘 4、槽 6）与内部条目宽（key 10、子指针 86）。
+const ALLOCATION_RECORD_TREE_KEY_BYTES: usize = 10;
+const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES: u16 = 96;
+/// inode 树根的 key 宽与内部条目宽（D8（核心索引结构） 的 inode 内部条目 120）。
+const INODE_TREE_KEY_BYTES: usize = 8;
+const INODE_INTERNAL_ENTRY_BYTES: u16 = 120;
+
+/// 偏移 `pointer` 起那条指针头部 extent 偏移那 2 字节（本来 0）写成 1。
+fn set_the_extent_offset_of_the_pointer_at(bytes: &mut [u8], pointer: usize) {
+    let extent_offset = pointer + POINTER_EXTENT_OFFSET_OFFSET;
+    assert_eq!(
+        bytes[extent_offset..extent_offset + 2],
+        [0u8; 2],
+        "第一版指针的 extent 偏移恒 0"
     );
-    assert!(
-        comparison.checker_mismatches.is_empty(),
-        "流 {}：{} 个状态池级 checker 的违例位 GPU 与 CPU 不一致或判不了（前几个）：\n{}",
-        flow.name,
-        comparison.checker_mismatches.len(),
-        comparison
-            .checker_mismatches
-            .iter()
-            .filter(|line| !line.is_empty())
-            .cloned()
-            .collect::<Vec<_>>()
-            .join("\n")
+    bytes[extent_offset..extent_offset + 2].copy_from_slice(&1u16.to_le_bytes());
+}
+
+/// 根记录实例表指针那一处：发布的根记录里实例表指针头部 extent 偏移写 1、重封根记录。池级 checker 走实例表链第 0 片时判它
+/// （`walk_instance_table_chain`），CPU 与 GPU 走读内核都红 I-2.4。core 择根时把这条根当损坏拒（`root_record.rs` 的 `parse_slot` 第五关），
+/// 退回上一条根（看 journal 那一遍再重放回这一版）；GPU 恢复内核择根同拒（`root_record_pointers_carry_no_extent_offset`）。
+fn record_new_pool_with_an_extent_offset_on_the_instance_table_pointer_of_the_root_record(
+    tag: &str,
+) -> RecordedFlow {
+    let mut flow = record_new_pool_file_creation(
+        tag,
+        "new-pool-root-record-instance-table-pointer-extent-offset-one",
     );
+    let publish_root = flow.judged_root_index;
+    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
+        set_the_extent_offset_of_the_pointer_at(bytes, ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET);
+    });
+    flow
+}
+
+/// inode 内部条目子指针那一处：发布那一版 inode 树根（码 2、key 宽 8、条目宽 120）第一条内部条目的子指针头部 extent 偏移写 1、重封到根。
+/// 池级 checker 走 inode 内部条目时判它，CPU 与 GPU 走读内核都红 I-2.4；core 解这条内部条目时拒
+/// （`records::parse_inode_internal_entry_or_refusal`），GPU 恢复内核在 `walk_to_file` 读 inode 树那一处同拒。
+fn record_new_pool_with_an_extent_offset_on_an_inode_internal_entry_child_pointer(
+    tag: &str,
+) -> RecordedFlow {
+    let inode_tree_root = |bytes: &[u8]| {
+        unit_class(bytes) == 2
+            && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
+            && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
+    };
+    record_new_pool_with_a_resealed_field(
+        tag,
+        "new-pool-inode-internal-entry-child-pointer-extent-offset-one",
+        &inode_tree_root,
+        false,
+        &|bytes| {
+            let first_entry = index_node_entries_start(INODE_TREE_KEY_BYTES);
+            set_the_extent_offset_of_the_pointer_at(
+                bytes,
+                first_entry + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET,
+            );
+        },
+    )
+}
+
+/// extent 上段叶条目里的那条指针（标签之后）在单元里的偏移；条目的标签要是 `expected_tag`。
+fn pointer_of_the_first_extent_upper_leaf_entry(bytes: &[u8], expected_tag: u8) -> usize {
+    let first_entry = index_node_entries_start(EXTENT_KEY_BYTES);
+    assert_eq!(
+        bytes[first_entry + EXTENT_KEY_BYTES],
+        expected_tag,
+        "上段叶第一条的标签"
+    );
+    first_entry + EXTENT_KEY_BYTES + 1
+}
+
+fn is_an_extent_upper_leaf(bytes: &[u8]) -> bool {
+    unit_class(bytes) == 2
+        && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
+        && index_node_entry_width(bytes) == EXTENT_UPPER_LEAF_ENTRY_BYTES
+}
+
+/// extent 数据指针那一处（内联在上段叶里的那条）：发布那一版唯一的数据单元由上段叶第一条内联指着（标签 2），这条数据指针头部 extent 偏移写 1、
+/// 重封到根。池级 checker 走数据指针时判它（`walk_extent_data_pointer`），CPU 与 GPU 走读内核都红 I-2.4；core 解上段叶条目时拒
+/// （`ExtentUpperLeafEntry::parse`），GPU 恢复内核读上段叶那一处同拒。
+fn record_new_pool_with_an_extent_offset_on_the_inline_data_pointer(tag: &str) -> RecordedFlow {
+    record_new_pool_with_a_resealed_field(
+        tag,
+        "new-pool-inline-data-pointer-extent-offset-one",
+        &is_an_extent_upper_leaf,
+        false,
+        &|bytes| {
+            let pointer = pointer_of_the_first_extent_upper_leaf_entry(
+                bytes,
+                EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT,
+            );
+            set_the_extent_offset_of_the_pointer_at(bytes, pointer);
+        },
+    )
+}
+
+/// 分配记录树内部条目子指针那一处：发布那一版分配记录树的根（码 2、key 宽 10、条目宽 96；一棵树里最后写出的内部节点）第一条内部条目的子指针
+/// 头部 extent 偏移写 1、重封到根。池级 checker 读分配记录树的码 2 节点时判它（`read_index_node`），CPU 与 GPU 走读内核都红 I-2.4；
+/// core 读这条内部条目时拒（`allocation_record_tree` 读子指针那一处），GPU 恢复内核 `read_allocation_tree` 同拒。
+fn record_new_pool_with_an_extent_offset_on_an_allocation_record_tree_child_pointer(
+    tag: &str,
+) -> RecordedFlow {
+    let allocation_internal_node = |bytes: &[u8]| {
+        unit_class(bytes) == 2
+            && usize::from(index_node_key_width(bytes)) == ALLOCATION_RECORD_TREE_KEY_BYTES
+            && index_node_entry_width(bytes) == ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES
+    };
+    record_new_pool_with_a_resealed_field(
+        tag,
+        "new-pool-allocation-record-tree-child-pointer-extent-offset-one",
+        &allocation_internal_node,
+        false,
+        &|bytes| {
+            let first_entry = index_node_entries_start(ALLOCATION_RECORD_TREE_KEY_BYTES);
+            set_the_extent_offset_of_the_pointer_at(
+                bytes,
+                first_entry + ALLOCATION_RECORD_TREE_KEY_BYTES,
+            );
+        },
+    )
+}
+
+/// 新池新建文件（txg 3）、第一个文件顺序写成三个数据单元（txg 4）：txg 4 那一版 extent 上段叶那一条指一个下段根（标签 1），
+/// 下段叶记三条数据指针。搭建用 `common_deletion_in_progress` 的 `pool_with_a_three_unit_file`，不往下走删除。
+fn record_new_pool_through_a_three_unit_file(tag: &str, name: &'static str) -> RecordedFlow {
+    let pool = pool_with_a_three_unit_file(tag);
+    recorded_flow_of_the_pool(
+        &pool,
+        name,
+        vec![
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(3),
+                content: file_content(),
+            },
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(4),
+                content: content_of_three_data_units(),
+            },
+        ],
+    )
+}
+
+/// 三个数据单元那条流上改坏最后一个 `select` 命中的单元（两份副本）的一处、重封到根。
+fn record_three_unit_file_with_a_resealed_field(
+    tag: &str,
+    name: &'static str,
+    select: &dyn Fn(&[u8]) -> bool,
+    mutate: &dyn Fn(&mut [u8]),
+) -> RecordedFlow {
+    let mut flow = record_new_pool_through_a_three_unit_file(tag, name);
+    let targets = last_unit_copies(&flow, name, select, false);
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
+    flow
+}
+
+/// extent 下段根指针那一处：txg 4 那一版上段叶第一条（标签 1）指着的下段根指针头部 extent 偏移写 1、重封到根。池级 checker 读下段根这个码 2 节点时
+/// 判它（`read_index_node`），CPU 与 GPU 走读内核都红 I-2.4；core 解上段叶条目时拒（`ExtentUpperLeafEntry::parse`），GPU 恢复内核读上段叶那一处同拒。
+fn record_three_unit_file_with_an_extent_offset_on_the_lower_segment_root_pointer(
+    tag: &str,
+) -> RecordedFlow {
+    record_three_unit_file_with_a_resealed_field(
+        tag,
+        "three-unit-file-lower-segment-root-pointer-extent-offset-one",
+        &is_an_extent_upper_leaf,
+        &|bytes| {
+            let pointer = pointer_of_the_first_extent_upper_leaf_entry(
+                bytes,
+                EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT,
+            );
+            set_the_extent_offset_of_the_pointer_at(bytes, pointer);
+        },
+    )
+}
+
+/// extent 数据指针那一处（下段叶记录里的那条）：txg 4 那一版下段叶（码 2、key 宽 24、条目宽 112）第一条记录的数据指针头部 extent 偏移写 1、
+/// 重封到根。池级 checker 走数据指针时判它（`walk_extent_data_pointer`），CPU 与 GPU 走读内核都红 I-2.4；core 解这条叶记录时拒
+/// （`records::parse_extent_record`），GPU 恢复内核 `read_lower_segment` 读叶那一处同拒。
+fn record_three_unit_file_with_an_extent_offset_on_a_lower_leaf_data_pointer(
+    tag: &str,
+) -> RecordedFlow {
+    let extent_lower_leaf = |bytes: &[u8]| {
+        unit_class(bytes) == 2
+            && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
+            && index_node_entry_width(bytes) == EXTENT_LOWER_LEAF_RECORD_BYTES
+    };
+    record_three_unit_file_with_a_resealed_field(
+        tag,
+        "three-unit-file-lower-leaf-data-pointer-extent-offset-one",
+        &extent_lower_leaf,
+        &|bytes| {
+            let first_record = index_node_entries_start(EXTENT_KEY_BYTES);
+            set_the_extent_offset_of_the_pointer_at(bytes, first_record + EXTENT_KEY_BYTES);
+        },
+    )
 }
 
 /// I-9.16 ③：树表里一种种类有两条。
@@ -4291,11 +4472,11 @@ fn the_gpu_judge_agrees_with_the_cpu_when_a_version_with_trees_also_holds_an_all
     );
 }
 
-/// I-2.4：码 2 节点的指针头部 extent 偏移非 0（池级 checker 那一半）。
+/// I-2.4：树表条目里码 2 节点的指针头部 extent 偏移非 0（池级 checker 与恢复两半都比）。
 #[test]
-fn the_gpu_checker_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset() {
-    assert_the_gpu_checker_agrees_with_the_cpu_on_a_flow_reddening(
-        "the_gpu_checker_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset",
+fn the_gpu_judge_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset",
         &record_new_pool_with_an_extent_offset_on_the_inode_tree_root_pointer(
             "crash-judge-gpu-i24-node-pointer-extent-offset",
         ),
@@ -4303,6 +4484,83 @@ fn the_gpu_checker_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_e
     );
 }
 
+/// I-2.4：根记录里实例表指针的头部 extent 偏移非 0（core 择根拒这条根）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_of_a_root_record_carries_an_extent_offset(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_of_a_root_record_carries_an_extent_offset",
+        &record_new_pool_with_an_extent_offset_on_the_instance_table_pointer_of_the_root_record(
+            "crash-judge-gpu-i24-root-instance-table-pointer-extent-offset",
+        ),
+        "I-2.4",
+    );
+}
+
+/// I-2.4：inode 内部条目子指针的头部 extent 偏移非 0。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_an_extent_offset(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_an_extent_offset",
+        &record_new_pool_with_an_extent_offset_on_an_inode_internal_entry_child_pointer(
+            "crash-judge-gpu-i24-inode-child-pointer-extent-offset",
+        ),
+        "I-2.4",
+    );
+}
+
+/// I-2.4：extent 上段叶内联的数据指针的头部 extent 偏移非 0。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_an_extent_offset() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_an_extent_offset",
+        &record_new_pool_with_an_extent_offset_on_the_inline_data_pointer(
+            "crash-judge-gpu-i24-inline-data-pointer-extent-offset",
+        ),
+        "I-2.4",
+    );
+}
+
+/// I-2.4：分配记录树内部条目子指针的头部 extent 偏移非 0。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_an_extent_offset(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_an_extent_offset",
+        &record_new_pool_with_an_extent_offset_on_an_allocation_record_tree_child_pointer(
+            "crash-judge-gpu-i24-allocation-child-pointer-extent-offset",
+        ),
+        "I-2.4",
+    );
+}
+
+/// I-2.4：extent 上段叶指着的下段根指针的头部 extent 偏移非 0。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_an_extent_offset(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_an_extent_offset",
+        &record_three_unit_file_with_an_extent_offset_on_the_lower_segment_root_pointer(
+            "crash-judge-gpu-i24-lower-segment-root-pointer-extent-offset",
+        ),
+        "I-2.4",
+    );
+}
+
+/// I-2.4：extent 下段叶记录的数据指针的头部 extent 偏移非 0。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_an_extent_offset(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_an_extent_offset",
+        &record_three_unit_file_with_an_extent_offset_on_a_lower_leaf_data_pointer(
+            "crash-judge-gpu-i24-lower-leaf-data-pointer-extent-offset",
+        ),
+        "I-2.4",
+    );
+}
+
 // ───────────── 里程碑三·3 删除线：I-9.5、I-3.4、I-4.7、I-4.6 与 I-9.6 墓碑那一半（GPU 扫描内核的删除态扫描与 CPU 池级 checker 逐状态逐位对拍）─────────────
 
 /// 码 2 / 码 3 单元头里的声明长度（偏移 8，2 字节）。
````

### mutations-append.tsv

````text
第 29 批 29-2 GPU 判器：恢复内核择根不判根记录四条指针的 extent 偏移	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	            if (!candidate.valid || !root_record_pointers_carry_no_extent_offset(candidate)) {	            if (!candidate.valid) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_of_a_root_record_carries_an_extent_offset	the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_of_a_root_record_carries_an_extent_offset
第 29 批 29-2 GPU 判器：恢复内核解树表条目不判树根指针的 extent 偏移	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	        if (!pointer_extent_offset_is_zero(tree_table.base, o + 14u)) {	        if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset	the_gpu_judge_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset
第 29 批 29-2 GPU 判器：恢复内核读分配记录树不判内部条目子指针的 extent 偏移	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	        if (!pending_pointer.extent_offset_zero) {	        if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_an_extent_offset	the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_an_extent_offset
第 29 批 29-2 GPU 判器：恢复内核读 extent 上段叶不判下段根指针的 extent 偏移	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	                        // ExtentUpperLeafEntry::parse：下段根指针头部 extent 偏移非 0，这条条目损坏\n                        if (!pointer_extent_offset_is_zero(n.base, o + 25u)) {	                        // ExtentUpperLeafEntry::parse：下段根指针头部 extent 偏移非 0，这条条目损坏\n                        if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_an_extent_offset	the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_an_extent_offset
第 29 批 29-2 GPU 判器：恢复内核读 extent 上段叶不判内联数据指针的 extent 偏移	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	                        // 同上：内联的数据指针\n                        if (!pointer_extent_offset_is_zero(n.base, o + 25u)) {	                        // 同上：内联的数据指针\n                        if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_an_extent_offset	the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_an_extent_offset
第 29 批 29-2 GPU 判器：恢复内核读 extent 下段叶不判数据指针的 extent 偏移	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	                    if (!pointer_extent_offset_is_zero(n.base, o + 24u)) {	                    if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_an_extent_offset	the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_an_extent_offset
第 29 批 29-2 GPU 判器：恢复内核读 inode 内部条目不判子指针的 extent 偏移	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	        if (!child.extent_offset_zero) {	        if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_an_extent_offset	the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_an_extent_offset
第 29 批 29-2 GPU 判器：走读内核读实例表链不判指针头部 extent 偏移	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        w_judge_pointer_head(pointer);\n        w_judge_pointer_extent_offset(pointer);\n        w_note_pointer_references(pointer, 2u);	        w_judge_pointer_head(pointer);\n        w_note_pointer_references(pointer, 2u);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_of_a_root_record_carries_an_extent_offset	the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_of_a_root_record_carries_an_extent_offset
第 29 批 29-2 GPU 判器：走读内核读 extent 数据指针不判指针头部 extent 偏移	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    w_judge_pointer_head(p);\n    w_judge_pointer_extent_offset(p);\n    w_note_pointer_references(p, 2u);	    w_judge_pointer_head(p);\n    w_note_pointer_references(p, 2u);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_an_extent_offset	the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_an_extent_offset
第 29 批 29-2 GPU 判器：走读内核走 inode 内部条目不判子指针头部 extent 偏移	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        w_judge_pointer_head(child);\n        w_judge_pointer_extent_offset(child);\n        var length = NODE_BYTES;	        w_judge_pointer_head(child);\n        var length = NODE_BYTES;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_an_extent_offset	the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_an_extent_offset
````

### mutations-replacements.tsv

````text
第 26 批 26-5 GPU 判器：走读内核读码 2 节点不判指针头部 extent 偏移	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    w_judge_pointer_head(p);\n    w_judge_pointer_extent_offset(p);\n    w_note_pointer_references(p, 1u);	    w_judge_pointer_head(p);\n    w_note_pointer_references(p, 1u);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset	the_gpu_judge_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset
````

## 主 agent：#9 补钉

````diff
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
index 7f1f598e..d9cf6f8a 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
@@ -826,8 +826,7 @@ fn print_the_unsharded_enumerations_for_the_golden_comparison() {
 /// `LAYER0_PARALLEL_START` + 46 行 `LAYER0_PROGRESS` + `LAYER0_FINDINGS` + `LAYER0_PARALLEL_FINISHED` + `GOLDEN_TALLY` 共 51 行。
 /// C577（系统配置没见证到的最新根，乙罩不到）那道发布返回之前的屏障加上之后甲二那一段 54 → 46 个状态（`crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs` 文件头），
 /// 行数与进度文件行数照这个数静态改（合入后验证二，改法 B 报告第四节第 5 条推的，没跑）；池级 checker 多一条 I-7.14 之后
-/// 打印的计数行与进度文件内容也跟着变。**两个 SHA-256 与进度文件名里的计划哈希留原值，待第五步 checker-tier-layer0-replay 跑出来重录**
-/// （checker 档，子 agent 不跑）。
+/// 打印的计数行与进度文件内容也跟着变。打的行与进度文件内容 2026-10-05 在「第 28 批 + 根环线 10e」那棵树上重取（见下）。
 ///
 /// 钉值的来历：头一版取自加分片之前的代码（`crash.rs` 与 `layer0_progress.rs` 加分片之前那一版：38 行、SHA-256 `6deeb48f…`，
 /// 进度文件计划哈希 `a1d502a7…`、内容 `798bef3c…`），证的是加分片没改不分片那条路。实审 B3a-2 按设备记屏障、原地覆写补第三态之后
@@ -837,13 +836,20 @@ fn print_the_unsharded_enumerations_for_the_golden_comparison() {
 /// `LAYER0_FINDINGS signatures=0 red_states=0 states=54`、进度文件格式号 1 → 2 且每行片行多一个 `findings=none`，在那一版上重取
 /// （同一棵树上加发现日志之前是 65 行 `8e1ca03b…`、进度文件 `6ec51601…`，→ 下面的值）：重取之前逐行对过，打的行只多出那两行
 /// `LAYER0_FINDINGS`，进度文件去掉每行的校验和、`findings=none` 与格式号之后逐字节相同，进度文件名（计划哈希）不变。
+/// 2026-10-04 打的行与进度文件名重取自主工作区 checker 档日志（这条用例判红时把 58 行原样打在断言消息里，整段 58 行带换行的 SHA-256
+/// 与断言 `left` 里的 `7e64a883…` 逐字相同）：比 `15867e36…` 那一版不同的只有两趟的 `GOLDEN_TALLY`（池级 checker 多出的不变量各一项，
+/// I-1.12 … I-9.28）与 `LAYER0_RESUME` 里进度文件名的计划哈希（fcc4d340 那一版的 `6118958c…` → `0370ad26…`；HEAD 31e47115 的日志里
+/// 计划哈希也是 `0370ad26…`、打的行只比主工作区少 `I-9.28=0/0/46` 那两项）。
 const GOLDEN_PRINTED_LINE_COUNT: usize = 58;
 const GOLDEN_PRINTED_LINES_SHA256: &str =
-    "15867e366db44d88e7de85fafe40ab6940310fc9cfe1a75787846c39b1024212";
-/// 同一趟留下的进度文件：名字（带计划哈希）与整份内容的 SHA-256（47 行：文件头 + 46 行片行；两个值都待第五步 checker-tier-layer0-replay 重录，见上）。
-const GOLDEN_PROGRESS_FILE_NAME: &str = "layer0-progress-golden_new_pool_file_creation_stream-fingerprint0-5026593803c5654d562ab1858215ddd4a5be477116564e882947660016a1eb5e.txt";
+    "e1ab3f797f369a2e9f3e45195331d252306b41346bef10922c7e04e1336a8cc4";
+/// 同一趟留下的进度文件：名字（带计划哈希）与整份内容的 SHA-256（47 行：文件头 + 46 行片行）。名字取自上面那一趟日志的 `LAYER0_RESUME` 行；
+/// 打的行与内容的 SHA-256 取自 2026-10-05「第 28 批 + 根环线 10e」那棵树（根环线点名跑这一条，树指纹开头结尾都是 `bdf78215…`）：
+/// 打的行是那一趟断言 `left` 的 `e1ab3f79…`；
+/// 内容是在只换了打的行钉值的副本里再跑一次、「不分片时的进度文件与钉值不同」那条断言 `left` 的第二项。池级 checker 的判定集合再变，两个值跟着重取。
+const GOLDEN_PROGRESS_FILE_NAME: &str = "layer0-progress-golden_new_pool_file_creation_stream-fingerprint0-0370ad26cad051266c57d7129a63840319034b488aefec9adebf007a9809b91e.txt";
 const GOLDEN_PROGRESS_FILE_SHA256: &str =
-    "6a153e1e268f829997ed67198ce0d125a450d0b12c4cdfbf5e6c1024ad8d5cd8";
+    "2a77c2d767d5dadc12e55e06860327826fb9c95c04870fe15ab5cb1241bf26bc";
 
 /// 一行打印里会随机器与时刻变的两样换掉：进度目录、`elapsed_seconds=` 的值。
 fn masked_printed_line(line: &str, directory: &Path) -> String {
````

## 主 agent：只开一个特性编不干净

````diff
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -38,10 +38,13 @@
 use crate::crash::{
     judge_layer0_state_range, layer0_state_ranges_by_segment,
     receive_until_the_senders_are_gone_with_wall_clock_lines, Layer0SegmentExpansion,
-    ReceivedOrWallClockLineDue, WallClockProgressLines, LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE,
-    LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE, LAYER0_RED_PASS_POOL_CHECKER,
-    LAYER0_RED_PASS_RECORD_CHECKER, RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT,
-    RECORD_CHECKER_ROOT_WITHOUT_RECORD,
+    ReceivedOrWallClockLineDue, WallClockProgressLines,
+};
+#[cfg(feature = "gpu")]
+use crate::crash::{
+    LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE, LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE,
+    LAYER0_RED_PASS_POOL_CHECKER, LAYER0_RED_PASS_RECORD_CHECKER,
+    RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT, RECORD_CHECKER_ROOT_WITHOUT_RECORD,
 };
 use crate::crash_facts::{extract_flow_facts, verify_states_from_facts, FlowFacts};
 use crate::crash_identity::{
@@ -2381,6 +2384,7 @@
     #[cfg(feature = "gpu")]
     Gpu(Box<crate::crash_verify_gpu::GpuFactsVerifier<'card>>),
     #[cfg(not(feature = "gpu"))]
+    #[expect(dead_code, reason = "没编进 gpu 时只替生命周期参数占位，从不建出")]
     Never(std::marker::PhantomData<&'card ()>),
 }
 
--- a/crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs
+++ b/crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs
@@ -1022,7 +1022,7 @@
 }
 
 /// σ 那次发布的写区间：单元写段起、到它根槽写所在的段末为止；紧跟的一段若只有系统配置槽轮换（没有单元写、没有根槽写），也归它。
-#[cfg(any(feature = "verdict-store", feature = "gpu"))]
+#[cfg(feature = "verdict-store")]
 fn sigma_publish_writes(recorded: &RecordedHistory) -> std::ops::Range<usize> {
     let chain = misaligned_reuse_chain(recorded);
     let sigma_first_write = *recorded.segments[chain.sigma]
@@ -1057,7 +1057,7 @@
 /// （它是今天唯一一条一个落点超过 32 次写的流）。只展开 σ 那次发布的段，其余段不展开；每个状态 GPU 的红项与 CPU 判器的
 /// `red_items` 逐项同名、同序，GPU 一个状态都不许判不了。要一张空闲显存够的 NVIDIA 独显：本机没有 NVIDIA 卡时打一行
 /// `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功，有卡而用不上判红（`gpu_unit_checks::gpu_test_start`）。
-#[cfg(feature = "gpu")]
+#[cfg(all(feature = "gpu", feature = "verdict-store"))]
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_judge_on_sigma_whose_configuration_slot_is_written_more_than_thirty_two_times(
 ) {
````
