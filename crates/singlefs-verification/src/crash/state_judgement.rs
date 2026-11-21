//! 判一个崩溃状态：恢复、oracle、池级 checker 与记录核对器各跑一遍，结论收成一份判定；读回那一路按序号区间逐个判。

use super::*;

/// 单版本形态：被判的那次根槽写出的那一代就是唯一带文件的版本。
pub fn evaluate_state(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: Vec<bool>,
    root_index: usize,
    expected_content: &[u8],
    tally: &mut CrashPointReplayTally,
) -> RecoveryReport {
    let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_index]);
    let versions = [PublishedVersion {
        instance,
        checkpoint_txg,
        content: expected_content.to_vec(),
    }];
    evaluate_state_for_versions(base, writes, persisted, root_index, &versions, tally)
}

/// 评一个状态：跑两种 journal 政策的恢复，记进计数。`judged_root_index` 是被判的那次根槽 FUA 写（计「根槽已持久」的状态数用），
/// oracle 按 `versions` 判实际走的根该读出哪一版。手摆的状态不在任何一段里，发现表（[`CrashPointReplayTally::findings`]）不记。
pub fn evaluate_state_for_versions(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: Vec<bool>,
    judged_root_index: usize,
    versions: &[PublishedVersion],
    tally: &mut CrashPointReplayTally,
) -> RecoveryReport {
    let image = CrashImage {
        base,
        writes,
        persisted,
    };
    let judgement = judge_crash_image_against_the_version_table(
        &image,
        judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(versions),
    );
    let passes_differ = recovery_passes_differ(&judgement.consulted, &judgement.ignored);
    let CrashStateJudgement {
        consulted,
        ignored: _,
        root_persisted,
        consulted_oracle,
        ignored_oracle,
        pool_checker,
        records,
    } = judgement;
    tally.states += 1;
    if root_persisted {
        tally.root_persisted_states += 1;
    }
    if passes_differ {
        tally.journal_differing_states += 1;
    }
    match &consulted.outcome {
        RecoveryOutcome::NoFile { .. } => tally.no_file_states += 1,
        RecoveryOutcome::FileRead { .. } => tally.file_read_states += 1,
        RecoveryOutcome::Failed { .. } => tally.failed_states += 1,
    }
    if let Some(violation) = consulted_oracle {
        tally.violations += 1;
        if tally.first_violation.is_none() {
            tally.first_violation = Some(violation_with_the_persisted_write_kinds(
                &violation.reason,
                &image,
            ));
        }
    }
    if let Some(violation) = ignored_oracle {
        tally.ignored_violations += 1;
        if tally.first_ignored_violation.is_none() {
            tally.first_ignored_violation = Some(violation.reason);
        }
    }
    for (invariant, verdict) in pool_checker {
        match verdict {
            InvariantVerdict::Holds => {
                *tally.checker_evaluated_states.entry(invariant).or_insert(0) += 1
            }
            InvariantVerdict::Violated(detail) => {
                *tally.checker_evaluated_states.entry(invariant).or_insert(0) += 1;
                *tally.checker_violated_states.entry(invariant).or_insert(0) += 1;
                tally
                    .checker_first_violation
                    .entry(invariant)
                    .or_insert(detail);
            }
            // 部分没判（`InvariantVerdict::PartiallyJudged`）不红、也不算评估过（判到的那一部分成立不等于整条成立）：计进不适用这一格，
            // 「评估过 + 不适用 = 状态数」照旧成立。
            InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {
                *tally
                    .checker_not_applicable_states
                    .entry(invariant)
                    .or_insert(0) += 1;
            }
        }
    }
    if records.root_without_record {
        tally.record_root_without_record += 1;
    }
    if records.claimed_state_missing_unit {
        tally.record_claimed_state_missing_unit += 1;
    }
    consulted
}

/// 看 journal 那一遍 oracle 的违例原文：原因后面带这一状态里持久了的写的种类（[`CrashPointReplayTally::first_violation`] 与发现表的样本同一个写法）。
fn violation_with_the_persisted_write_kinds(reason: &str, image: &CrashImage<'_>) -> String {
    let persisted_kinds: Vec<&str> = image
        .persisted
        .iter()
        .zip(image.writes)
        .filter(|(is_persisted, _)| **is_persisted)
        .map(|(_, write)| write.kind.name())
        .collect();
    format!("{reason}（持久的写：{}）", persisted_kinds.join("|"))
}

/// 池级 checker 在一个状态上判违例的那几条拼成一条原文：`I-3.1：细节；I-7.1：细节`。
fn pool_checker_violation_text(violated_invariants: &[(&'static str, String)]) -> String {
    violated_invariants
        .iter()
        .map(|(invariant, detail)| format!("{invariant}：{detail}"))
        .collect::<Vec<String>>()
        .join("；")
}

/// 记录核对器在一个状态上成立的判据拼成一条原文。
fn record_checker_violation_text(record_check: RecordCheck) -> String {
    let mut criteria = Vec::new();
    if record_check.root_without_record {
        criteria.push("根槽写在盘上而那次发布的 journal 记录一份都不在");
    }
    if record_check.claimed_state_missing_unit {
        criteria.push("恢复自称的那一版该有的单元两份都缺席");
    }
    criteria.join("；")
}

/// 一个崩溃状态的全部判定：两种 journal 政策的恢复、两份 oracle、池级 checker 逐条、记录核对器。
/// 崩溃点重放的计数（[`evaluate_state_for_versions`] 一族）与崩溃放量流水线（`crash_amplification`）都从这一份取，不各判一遍。
#[derive(Clone, Debug)]
pub struct CrashStateJudgement {
    pub consulted: RecoveryReport,
    pub ignored: RecoveryReport,
    /// 被判的那次根槽写落没落（计「根槽已持久」的状态数用）。
    pub root_persisted: bool,
    pub consulted_oracle: Option<CrashPointReplayOracleViolation>,
    pub ignored_oracle: Option<CrashPointReplayOracleViolation>,
    pub pool_checker: Vec<(&'static str, InvariantVerdict)>,
    pub records: RecordCheck,
}

impl CrashStateJudgement {
    /// 判红的各项，按固定次序：两份 oracle 各记它的违例种类，池级 checker 记违反的不变量名，记录核对器记它的两种结论；
    /// 一项都没有就是绿。崩溃放量流水线按这串存「逐项原始判定」（读方现算红绿与计数）。
    #[must_use]
    pub fn red_items(&self) -> Vec<String> {
        let mut items = Vec::new();
        if let Some(violation) = &self.consulted_oracle {
            items.push(format!(
                "{CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE}:{}",
                violation.kind.name()
            ));
        }
        if let Some(violation) = &self.ignored_oracle {
            items.push(format!(
                "{CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE}:{}",
                violation.kind.name()
            ));
        }
        for (invariant, verdict) in &self.pool_checker {
            match verdict {
                InvariantVerdict::Violated(_) => items.push(format!(
                    "{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}:{invariant}"
                )),
                InvariantVerdict::Holds
                | InvariantVerdict::NotApplicable(_)
                | InvariantVerdict::PartiallyJudged(_) => {}
            }
        }
        if self.records.root_without_record {
            items.push(format!(
                "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_ROOT_WITHOUT_RECORD}"
            ));
        }
        if self.records.claimed_state_missing_unit {
            items.push(format!(
                "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT}"
            ));
        }
        items
    }

    /// 违例正文：这个状态自己现算的（「正文逐状态」），不从类的代表取。绿的状态是空串。
    #[must_use]
    pub fn violation_text(&self, image: &CrashImage<'_>) -> String {
        let mut text = String::new();
        if let Some(violation) = &self.consulted_oracle {
            text.push_str(&violation_with_the_persisted_write_kinds(
                &violation.reason,
                image,
            ));
            text.push('\n');
        }
        if let Some(violation) = &self.ignored_oracle {
            text.push_str(&violation.reason);
            text.push('\n');
        }
        let violated: Vec<(&'static str, String)> = self
            .pool_checker
            .iter()
            .filter_map(|(invariant, verdict)| match verdict {
                InvariantVerdict::Violated(detail) => Some((*invariant, detail.clone())),
                InvariantVerdict::Holds
                | InvariantVerdict::NotApplicable(_)
                | InvariantVerdict::PartiallyJudged(_) => None,
            })
            .collect();
        if !violated.is_empty() {
            text.push_str(&pool_checker_violation_text(&violated));
            text.push('\n');
        }
        if self.records.root_without_record || self.records.claimed_state_missing_unit {
            text.push_str(&record_checker_violation_text(self.records));
            text.push('\n');
        }
        text
    }
}

/// 同 [`judge_crash_image`]，oracle 对一张版本表判（[`classified_oracle_violation_for_the_version_table`]）。
#[must_use]
pub(super) fn judge_crash_image_against_the_version_table(
    image: &CrashImage<'_>,
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
) -> CrashStateJudgement {
    let root_persisted = image.persisted[judged_root_index];
    let newest_persisted = newest_persisted_root(image.writes, &image.persisted);
    let consulted = recover(image, JournalPolicy::Consult);
    let ignored = recover(image, JournalPolicy::Ignore);
    let consulted_oracle = classified_oracle_violation_for_the_version_table(
        &consulted.outcome,
        consulted.effective_root,
        newest_persisted,
        versions,
    );
    let ignored_oracle = classified_oracle_violation_for_the_version_table(
        &ignored.outcome,
        ignored.effective_root,
        newest_persisted,
        versions,
    );
    let pool_checker = check_pool_image(image);
    let records = check_records(image, consulted.effective_root);
    CrashStateJudgement {
        consulted,
        ignored,
        root_persisted,
        consulted_oracle,
        ignored_oracle,
        pool_checker,
        records,
    }
}

/// [`judge_crash_state_range_against_the_version_table_naming_the_publish`] 的本体：只给已经从判定库读回的流用（崩溃放量判段、取样）。
/// 每调一次现建一份枚举的表（[`TornImageTablesOfTheReadBackFlow`]）：几条线程判同一条流的，由调用方建一份、经
/// [`CrashStatesOfTheReadBackFlow`] 共用，不逐段调这一个。
///
/// # Panics
/// `ordinals` 越过整条流的状态数。
#[allow(
    clippy::too_many_arguments,
    reason = "与 judge_crash_state_range_against_the_version_table_naming_the_publish 同一组参数"
)]
#[must_use]
pub(crate) fn judge_crash_state_range_of_the_read_back_flow(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    ordinals: Range<u64>,
    judge_state: &mut dyn FnMut(
        u64,
        CrashPointReplayPublishOfState,
        &CrashImage<'_>,
        &CrashStateJudgement,
    ),
) -> u64 {
    let tables = TornImageTablesOfTheReadBackFlow::of_the_read_back_flow(base, writes, segments);
    judge_crash_state_range_of_the_read_back_flow_with_slices(
        &tables.crash_states_planned_by(
            base,
            writes,
            segments,
            expansion,
            &no_segment_slices(segments.len()),
        ),
        judged_root_index,
        versions,
        ordinals,
        judge_state,
    )
}

/// 判 `crash_states_of_the_flow`（读回的流排好次序的全部状态，分片的段照录入段定的片只排那一截）里 `ordinals` 那几个状态
/// （[`judge_crash_image_against_the_version_table`]），连同镜像交 `judge_state`，交回整条流的状态数。崩溃放量判段的 CPU 那一路：
/// 几条核对线程各判一段时借同一份 `crash_states_of_the_flow`，枚举的表只建一份。
///
/// # Panics
/// `ordinals` 越过整条流的状态数。
#[must_use]
pub(crate) fn judge_crash_state_range_of_the_read_back_flow_with_slices(
    crash_states_of_the_flow: &CrashStatesOfTheReadBackFlow<'_>,
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    ordinals: Range<u64>,
    judge_state: &mut dyn FnMut(
        u64,
        CrashPointReplayPublishOfState,
        &CrashImage<'_>,
        &CrashStateJudgement,
    ),
) -> u64 {
    crash_states_of_the_flow.visit_the_states_in_the_ordinals(
        ordinals,
        &mut |ordinal, publish, image| {
            let judgement =
                judge_crash_image_against_the_version_table(image, judged_root_index, versions);
            judge_state(ordinal, publish, image, &judgement);
        },
    )
}
