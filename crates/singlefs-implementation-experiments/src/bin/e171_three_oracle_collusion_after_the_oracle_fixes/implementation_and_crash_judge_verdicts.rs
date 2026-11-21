//! 一个取样态上实现读回的那一版、崩溃判官（O1-崩溃判官）的判与恢复摘要。

use super::*;

/// 一个状态上实现的判定（登记第一节读法 6：「实现都放行」= 池级 checker 一条 Violated 都没有，且记录核对器两项都为假）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ImplementationVerdict {
    pub(super) consulted_root: String,
    pub(super) consulted_outcome: String,
    pub(super) ignored_root: String,
    pub(super) ignored_outcome: String,
    pub(super) red_items: Vec<String>,
    pub(super) pool_checker_red: Vec<String>,
    /// 记录核对器的两项（`true` / `false`；逐件重做那一路里那一件 panic 写 `panic`、没跑写 `not_run`）。
    pub(super) root_without_record: String,
    pub(super) claimed_state_missing_unit: String,
}

impl ImplementationVerdict {
    /// 回调链 panic 了的那一态（修订 5 第 5.1 节第 4 条）：各格取各件自己的结果，panic 的件写 `panic`，没跑的写 `not_run`。
    /// `red_items` 照 [`CrashStateJudgement::red_items`] 的次序与写法列跑完了的那几件判红的项，panic 与没跑的件记成
    /// `panic:<项>`、`not_run:<项>`（不以 `<项>:` 起头，不进按项数红的计数）。
    pub(super) fn of_the_pieces(
        consulted: &PieceOutcome<RecoveryReport>,
        ignored: &PieceOutcome<RecoveryReport>,
        crash_point_replay_oracles: &PieceOutcome<CrashPointReplayOraclesOfThePieces>,
        pool_checker: &PieceOutcome<Vec<(&'static str, InvariantVerdict)>>,
        record_checker: &PieceOutcome<RecordCheck>,
    ) -> Self {
        let mut red_items = Vec::new();
        let oracles: [(&str, Result<Option<&CrashPointReplayOracleViolation>, &str>); 2] =
            match crash_point_replay_oracles {
                PieceOutcome::Done(CrashPointReplayOraclesOfThePieces {
                    consulted: consulted_oracle,
                    ignored: ignored_oracle,
                }) => [
                    (
                        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE,
                        consulted_oracle
                            .as_ref()
                            .map(Option::as_ref)
                            .ok_or("not_run"),
                    ),
                    (
                        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE,
                        ignored_oracle.as_ref().map(Option::as_ref).ok_or("not_run"),
                    ),
                ],
                PieceOutcome::Panicked(_) => [
                    (
                        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE,
                        Err("panic"),
                    ),
                    (
                        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE,
                        Err("panic"),
                    ),
                ],
                PieceOutcome::NotRun => [
                    (
                        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE,
                        Err("not_run"),
                    ),
                    (
                        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE,
                        Err("not_run"),
                    ),
                ],
            };
        for (pass, oracle) in oracles {
            match oracle {
                Ok(Some(violation)) => {
                    red_items.push(format!("{pass}:{}", violation.kind.name()));
                }
                Ok(None) => {}
                Err(marker) => red_items.push(format!("{marker}:{pass}")),
            }
        }
        match pool_checker {
            PieceOutcome::Done(verdicts) => red_items.extend(
                violated_invariant_names(verdicts)
                    .into_iter()
                    .map(|invariant| {
                        format!("{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}:{invariant}")
                    }),
            ),
            PieceOutcome::Panicked(_) => {
                red_items.push(format!("panic:{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}"));
            }
            PieceOutcome::NotRun => {
                red_items.push(format!(
                    "not_run:{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}"
                ));
            }
        }
        match record_checker {
            PieceOutcome::Done(records) => {
                if records.root_without_record {
                    red_items.push(format!(
                        "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_ROOT_WITHOUT_RECORD}"
                    ));
                }
                if records.claimed_state_missing_unit {
                    red_items.push(format!(
                        "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT}"
                    ));
                }
            }
            PieceOutcome::Panicked(_) => {
                red_items.push(format!(
                    "panic:{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}"
                ));
            }
            PieceOutcome::NotRun => {
                red_items.push(format!(
                    "not_run:{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}"
                ));
            }
        }
        Self {
            consulted_root: consulted.cell_text(|report| root_text(report.effective_root)),
            consulted_outcome: consulted.cell_text(|report| outcome_text(&report.outcome)),
            ignored_root: ignored.cell_text(|report| root_text(report.effective_root)),
            ignored_outcome: ignored.cell_text(|report| outcome_text(&report.outcome)),
            red_items,
            pool_checker_red: match pool_checker {
                PieceOutcome::Done(verdicts) => violated_invariant_names(verdicts),
                PieceOutcome::Panicked(_) => vec!["panic".to_string()],
                PieceOutcome::NotRun => vec!["not_run".to_string()],
            },
            root_without_record: record_checker
                .cell_text(|records| records.root_without_record.to_string()),
            claimed_state_missing_unit: record_checker
                .cell_text(|records| records.claimed_state_missing_unit.to_string()),
        }
    }
    pub(super) fn of(judgement: &CrashStateJudgement) -> Self {
        let report_root = |report: &RecoveryReport| root_text(report.effective_root);
        Self {
            consulted_root: report_root(&judgement.consulted),
            consulted_outcome: outcome_text(&judgement.consulted.outcome),
            ignored_root: report_root(&judgement.ignored),
            ignored_outcome: outcome_text(&judgement.ignored.outcome),
            red_items: judgement.red_items(),
            pool_checker_red: judgement
                .pool_checker
                .iter()
                .filter(|(_, verdict)| match verdict {
                    InvariantVerdict::Violated(_) => true,
                    InvariantVerdict::Holds
                    | InvariantVerdict::NotApplicable(_)
                    | InvariantVerdict::PartiallyJudged(_) => false,
                })
                .map(|(invariant, _)| (*invariant).to_string())
                .collect(),
            root_without_record: judgement.records.root_without_record.to_string(),
            claimed_state_missing_unit: judgement.records.claimed_state_missing_unit.to_string(),
        }
    }
}

// ───────────────────────────── 每个取样状态上的 oracle（O1 三个子列、O2；O3 在 Python 一侧） ─────────────────────────────

/// O1-模型崩溃判官在一个状态上的结论（登记第一节读法 1、6）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CrashJudgeVerdict {
    /// 这一态有写落在模型停下的那次调用（对拍判出不同、交回错误、panic，读法 6 的三处）或之后的调用里。
    NotApplicable,
    Green,
    /// 红：模型那一侧说的话（`ModelDisagreement::implementation_answer`，空白换成下划线）。
    Red {
        what: String,
    },
    /// 关一条（Q3-核）关掉了 O1：这一列不算。
    Off,
    /// 这一态上看 journal 那一遍恢复 panic（读法 5「态上的恢复 panic」；修订 5 第 5.1 节第 3 条收窄成只随这一遍）：
    /// O1 的两个崩溃子列在这一态记不适用。
    RecoveryPanicked,
    /// 这一态上 O1-模型崩溃判官自己 panic（修订 5 第 5.1 节第 3 条）：不红不绿，与弃权、不适用分开计。
    JudgePanicked,
}

impl CrashJudgeVerdict {
    pub(super) fn word(&self) -> &'static str {
        match self {
            Self::NotApplicable => "na",
            Self::Green => "green",
            Self::Red { .. } => "red",
            Self::Off => "off",
            Self::RecoveryPanicked => "recovery_panic",
            Self::JudgePanicked => "judge_panic",
        }
    }
}

/// 一遍恢复报告的摘要（触达判法 ③，登记第一节读法 4）：择到的根、结局种类、读回内容的 sha256、施加的记录条数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RecoverySummary {
    pub(super) root: String,
    pub(super) outcome: String,
    read_back_sha256: String,
    records_applied: usize,
}

impl RecoverySummary {
    pub(super) fn of(report: &RecoveryReport) -> Self {
        Self {
            root: root_text(report.effective_root),
            outcome: outcome_text(&report.outcome),
            read_back_sha256: match &report.outcome {
                RecoveryOutcome::FileRead { content, .. } => sha256_hexadecimal(content),
                RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. } => "-".to_string(),
            },
            records_applied: report.journal.prefix_applied,
        }
    }
    /// 恢复 panic 的那一态：四格都写 `recovery_panic`。
    pub(super) fn of_a_panicking_recovery() -> Self {
        Self {
            root: "recovery_panic".to_string(),
            outcome: "recovery_panic".to_string(),
            read_back_sha256: "-".to_string(),
            records_applied: 0,
        }
    }
    pub(super) fn text(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.root, self.outcome, self.read_back_sha256, self.records_applied
        )
    }
}

/// 池级 checker 判红的不变量名（`Violated` 才算；`NotApplicable` 不算红）。
pub(super) fn violated_invariant_names(
    verdicts: &[(&'static str, InvariantVerdict)],
) -> Vec<String> {
    verdicts
        .iter()
        .filter(|(_, verdict)| match verdict {
            InvariantVerdict::Violated(_) => true,
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => false,
        })
        .map(|(invariant, _)| (*invariant).to_string())
        .collect()
}

#[cfg(test)]
mod tests;
