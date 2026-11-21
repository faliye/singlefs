//! 判的几块（judging piece）：各块的结局、块内 panic 的记账与注入。

use super::*;

/// 判一态那条回调链上的一件（修订 5 第 5.1 节第 2 条）：回调链 panic 了，装置按这个次序在同一个镜像上逐件重做、每件各包一层 `catch_unwind`。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum JudgingPiece {
    /// `recover(image, JournalPolicy::Consult)`。
    ConsultedRecovery,
    /// `recover(image, JournalPolicy::Ignore)`。
    IgnoredRecovery,
    /// 被判那次根槽写落没落（`image.persisted[judged_root_index]`）。
    RootPersisted,
    /// 两份 `classified_oracle_violation_for_versions`（连同它们吃的「盘上已持久的最新根」）。
    CrashPointReplayOracles,
    /// `check_pool_image`（`judge_crash_image` 里那次与装置直调那次是同一函数、同一镜像，算一件）。
    PoolChecker,
    /// `check_records(image, consulted.effective_root)`。
    RecordChecker,
    /// O1-模型崩溃判官本身（连同它吃的读回）。
    CrashJudge,
    /// RefFS 一票本身（连同它吃的读回）。
    ReferenceFileSystem,
    /// 计时那一遍恢复。
    TimingRecovery,
    /// 可写挂载探针（开了才有）。
    Probe,
}

/// 判态路上的件数（[`JudgingPiece::ALL`] 的长度）。
pub(super) const JUDGING_PIECE_COUNT: usize = 10;

impl JudgingPiece {
    pub(super) const ALL: [JudgingPiece; JUDGING_PIECE_COUNT] = [
        JudgingPiece::ConsultedRecovery,
        JudgingPiece::IgnoredRecovery,
        JudgingPiece::RootPersisted,
        JudgingPiece::CrashPointReplayOracles,
        JudgingPiece::PoolChecker,
        JudgingPiece::RecordChecker,
        JudgingPiece::CrashJudge,
        JudgingPiece::ReferenceFileSystem,
        JudgingPiece::TimingRecovery,
        JudgingPiece::Probe,
    ];

    /// 在 [`JudgingPiece::ALL`] 里的下标。
    pub(super) fn position(self) -> usize {
        JudgingPiece::ALL
            .iter()
            .position(|piece| *piece == self)
            .expect("ALL 列全了每一件")
    }

    pub(super) fn name(self) -> &'static str {
        match self {
            JudgingPiece::ConsultedRecovery => "consulted_recovery",
            JudgingPiece::IgnoredRecovery => "ignored_recovery",
            JudgingPiece::RootPersisted => "root_persisted",
            JudgingPiece::CrashPointReplayOracles => "layer0_oracles",
            JudgingPiece::PoolChecker => "pool_checker",
            JudgingPiece::RecordChecker => "record_checker",
            JudgingPiece::CrashJudge => "crash_judge",
            JudgingPiece::ReferenceFileSystem => "reference_filesystem",
            JudgingPiece::TimingRecovery => "timing_recovery",
            JudgingPiece::Probe => "probe",
        }
    }
}

/// 逐件重做时一件的结局：做完了、panic 了（panic 的消息）、没跑（它吃的那一遍恢复 panic 了，或开关没开）。
pub(super) enum PieceOutcome<Value> {
    Done(Value),
    Panicked(String),
    NotRun,
}

impl<Value> PieceOutcome<Value> {
    pub(super) fn done(&self) -> Option<&Value> {
        match self {
            PieceOutcome::Done(value) => Some(value),
            PieceOutcome::Panicked(_) | PieceOutcome::NotRun => None,
        }
    }

    /// `impl-verdicts.tsv` 里这一件的格：做完了照 `cell` 写，panic 写 `panic`，没跑写 `not_run`。
    pub(super) fn cell_text(&self, cell: impl FnOnce(&Value) -> String) -> String {
        match self {
            PieceOutcome::Done(value) => cell(value),
            PieceOutcome::Panicked(_) => "panic".to_string(),
            PieceOutcome::NotRun => "not_run".to_string(),
        }
    }
}

/// 在一个 `catch_unwind` 里做一件；`injected` 是这一件时先 panic（阳性对照「判态路 panic 单列」的注入，[`PiecePanicInjection`]）。
pub(super) fn judging_piece_outcome<Value>(
    piece: JudgingPiece,
    injected: Option<JudgingPiece>,
    work: impl FnOnce() -> Value,
) -> PieceOutcome<Value> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert!(
            injected != Some(piece),
            "修订 5 阳性对照「判态路 panic 单列」注入：{} 这一件 panic",
            piece.name()
        );
        work()
    })) {
        Ok(value) => PieceOutcome::Done(value),
        Err(payload) => PieceOutcome::Panicked(panic_text(&*payload)),
    }
}

/// 逐件重做时两份崩溃点重放 oracle 的结局：各自吃的那一遍恢复 panic 了是 None（记 `not_run`），跑了是它交回的（没有违例是 `Some(None)`）。
pub(super) struct CrashPointReplayOraclesOfThePieces {
    pub(super) consulted: Option<Option<CrashPointReplayOracleViolation>>,
    pub(super) ignored: Option<Option<CrashPointReplayOracleViolation>>,
}

/// 一件 panic 了就记下它的名字（[`StateObservation::piece_panics`]）。
pub(super) fn note_a_piece_panic<Value>(
    piece_panics: &mut Vec<JudgingPiece>,
    piece: JudgingPiece,
    outcome: &PieceOutcome<Value>,
) {
    if let PieceOutcome::Panicked(_) = outcome {
        piece_panics.push(piece);
    }
}

/// 一组态上各件 panic 的态数，按 [`JudgingPiece::ALL`] 的次序每件一格（0 也在）。
pub(super) fn piece_panic_states_of<'observation>(
    observations: impl IntoIterator<Item = &'observation StateObservation>,
) -> [usize; JUDGING_PIECE_COUNT] {
    let mut states = [0usize; JUDGING_PIECE_COUNT];
    for observation in observations {
        for piece in &observation.piece_panics {
            states[piece.position()] += 1;
        }
    }
    states
}

/// 正常那一路 `catch_unwind` 接到 panic、逐件重做十件一件都没 panic 的态数（`unattributed_panic_states=`，
/// 登记「主 agent 认定：27-2 装置交回」第 2 条：甲那条管子漏了一件）。
pub(super) fn unattributed_panic_states_of<'observation>(
    observations: impl IntoIterator<Item = &'observation StateObservation>,
) -> usize {
    observations
        .into_iter()
        .filter(|observation| {
            observation.judging_path_panic.is_some() && observation.piece_panics.is_empty()
        })
        .count()
}

/// `name=batch`、`name=verdict` 两种行的 `piece_panic_states=` 字段：`<件>:<态数>` 逗号连起来，0 的件不写，全 0 写 `-`。
pub(super) fn piece_panic_states_text(states: &[usize; JUDGING_PIECE_COUNT]) -> String {
    let nonzero: Vec<String> = JudgingPiece::ALL
        .iter()
        .zip(states)
        .filter(|(_, count)| **count > 0)
        .map(|(piece, count)| format!("{}:{count}", piece.name()))
        .collect();
    if nonzero.is_empty() {
        "-".to_string()
    } else {
        nonzero.join(",")
    }
}

/// `name=q5_piece_panics` 那一行：十件各一个字段（修订 5 第 5.1 节第 5 条），再带 `unattributed_panic_states=`。
pub(super) fn confirmed_loss_form_piece_panics_line(
    sampling_point: &str,
    observations: &[StateObservation],
) -> String {
    let states = piece_panic_states_of(observations);
    let fields: Vec<String> = JudgingPiece::ALL
        .iter()
        .zip(states)
        .map(|(piece, count)| format!("{}={count}", piece.name()))
        .collect();
    format!(
        "name=q5_piece_panics sampling_point={sampling_point} judged_states={} {} unattributed_panic_states={}",
        observations.len(),
        fields.join(" "),
        unattributed_panic_states_of(observations)
    )
}

/// 一组态里有 [`unattributed_panic_states_of`] 数到的态（登记「主 agent 认定：27-2 装置交回」第 2 条）：这一趟的 Q5 不判甲乙、不进 S13，
/// 打 `q5_not_run=unattributed_panic` 的判决行、收尾，交回退出码 3（交主 agent）；一态都没有交 `None`，调用方接着判。
pub(super) fn stop_the_confirmed_loss_form_and_write_the_verdict_when_a_panic_is_unattributed(
    lines: &mut ResultLines,
    failed_checks: usize,
    observations: &[StateObservation],
) -> Option<i32> {
    let unattributed_panic_states = unattributed_panic_states_of(observations);
    if unattributed_panic_states == 0 {
        return None;
    }
    lines.emit_result_line(&format!(
        "name=verdict s1_failed_checks={failed_checks} s13=false q5_not_run=unattributed_panic piece_panic_states={} unattributed_panic_states={unattributed_panic_states}",
        piece_panic_states_text(&piece_panic_states_of(observations))
    ));
    lines.finish();
    Some(3)
}

/// 阳性对照「判态路 panic 单列」的注入（修订 5 第 5.1 节）：按落地串选态，选中的态正常那一路在回调里先 panic（逼走逐件重做那一路），
/// 逐件重做时点名的那一件 panic。只经装置内部造（[`piece_panic_control_injection`] 与内联用例），命令行不开放；不改实现。
pub(super) struct PiecePanicInjection {
    pub(super) rules: Vec<PiecePanicInjectionRule>,
}

/// 一条注入：落地串满足全部条件的态，正常那一路 panic、逐件重做时 `piece` 那一件 panic。各条选中的态互不相交
/// （造的人保证；[`PiecePanicInjection::rule_of`] 取第一条）。
pub(super) struct PiecePanicInjectionRule {
    /// `None`：逐件重做时一件都不 panic，只有正常那一路 panic（造 `unattributed_panic_states=` 数的那种态，只有内联用例造）。
    pub(super) piece: Option<JudgingPiece>,
    pub(super) conditions: Vec<InjectedLandingCondition>,
}

/// 落地串上的一个条件（写表下标；「落」= 那一位不是 `0`）。
pub(super) enum InjectedLandingCondition {
    Landed(usize),
    NotLanded(usize),
    BothOrNeitherLanded(usize, usize),
}

impl PiecePanicInjection {
    /// 这一态（落地串）选中的那一条注入；没有一条选中交 `None`。
    pub(super) fn rule_of(&self, landing_vector: &str) -> Option<&PiecePanicInjectionRule> {
        let landed = |write_index: usize| {
            landing_vector
                .as_bytes()
                .get(write_index)
                .is_some_and(|landing| *landing != b'0')
        };
        self.rules.iter().find(|rule| {
            rule.conditions.iter().all(|condition| match condition {
                InjectedLandingCondition::Landed(write_index) => landed(*write_index),
                InjectedLandingCondition::NotLanded(write_index) => !landed(*write_index),
                InjectedLandingCondition::BothOrNeitherLanded(first, second) => {
                    landed(*first) == landed(*second)
                }
            })
        })
    }
}

#[cfg(test)]
mod tests;
