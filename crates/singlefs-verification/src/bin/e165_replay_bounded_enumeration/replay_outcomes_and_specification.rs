//! 一次调用 `replay_journal` 的结局（失败种类、返回的根、接住 panic）与照条款写的全函数 S（登记 5.3「对抗一族的判据」）。

use super::*;

// ── 一次调用的结局

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FailureKind {
    /// `UnitMalformed`，`what` 是 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`。
    ChosenRootTxgHasNoSuccessor,
    /// `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`。
    RootPublishCarriesMoreThanOneLastRecordFlag,
    Other,
}

impl FailureKind {
    fn of(failure: &RecoveryFailure) -> Self {
        if matches!(failure, RecoveryFailure::UnitMalformed { what } if *what == CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR)
        {
            Self::ChosenRootTxgHasNoSuccessor
        } else if matches!(
            failure,
            RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided { .. }
        ) {
            Self::RootPublishCarriesMoreThanOneLastRecordFlag
        } else {
            Self::Other
        }
    }

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::ChosenRootTxgHasNoSuccessor => "chosen_root_txg_has_no_successor",
            Self::RootPublishCarriesMoreThanOneLastRecordFlag => "more_than_one_last_record_flag",
            Self::Other => "other_failure",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ReturnedReplay {
    pub(super) root: RootRecord,
    pub(super) prefix_applied: usize,
    pub(super) verification_passed: usize,
}

#[allow(
    clippy::large_enum_variant,
    reason = "每个状态一两个结局、按值比较拷来拷去要 Copy；装箱就丢了 Copy，每个状态还多一次分配"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReplayOutcome {
    Panicked,
    Failed(FailureKind),
    Returned(ReturnedReplay),
}

pub(super) fn describe_outcome(outcome: &ReplayOutcome) -> String {
    match outcome {
        ReplayOutcome::Panicked => "panicked".to_string(),
        ReplayOutcome::Failed(kind) => format!("failed:{}", kind.label()),
        ReplayOutcome::Returned(returned) => format!(
            "returned:txg{}:watermark{}:applied{}",
            returned.root.checkpoint_txg.0,
            returned.root.tree_identifier_watermark,
            returned.prefix_applied
        ),
    }
}

pub(super) fn install_the_panic_hook_that_stays_quiet_inside_the_replay_call() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_information| {
        if !IS_INSIDE_THE_REPLAY_CALL.with(Cell::get) {
            default_hook(panic_information);
        }
    }));
}

/// 被测对象：今天的 `replay_journal`，不改一个字。panic 用 `catch_unwind` 接住（J0、A0）。
pub(super) fn call_replay_journal(
    disk: &MemoryDisk<'_>,
    chosen_root: &RootRecord,
    records: &RecordTable,
    verify_named_units: bool,
) -> ReplayOutcome {
    IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(true));
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        replay_journal(
            disk,
            chosen_root,
            disk.ring.ring_bytes(),
            records,
            if verify_named_units {
                NamedUnitVerification::Verify
            } else {
                NamedUnitVerification::Skip
            },
        )
    }));
    IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(false));
    match result {
        Err(_panic_payload) => ReplayOutcome::Panicked,
        Ok(Err(failure)) => ReplayOutcome::Failed(FailureKind::of(&failure)),
        Ok(Ok((report, root))) => ReplayOutcome::Returned(ReturnedReplay {
            root,
            prefix_applied: report.prefix_applied,
            verification_passed: report.verification_passed,
        }),
    }
}

// ── 照条款写的全函数 S（登记 5.3「对抗一族的判据」）

/// (j) 的读法（岔路单 K9）。`NeverJudges` 只给 A-互 的阳性对照用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FollowReading {
    WholeTable,
    ConsecutiveGapContinues,
    ConsecutiveGapStops,
    NeverJudges,
}

impl FollowReading {
    /// 登记的三种读法，次序是 AS-全、AS-连甲、AS-连乙。
    pub(super) const REGISTERED: [Self; 3] = [
        Self::WholeTable,
        Self::ConsecutiveGapContinues,
        Self::ConsecutiveGapStops,
    ];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::WholeTable => "whole_table",
            Self::ConsecutiveGapContinues => "consecutive_gap_continues",
            Self::ConsecutiveGapStops => "consecutive_gap_stops",
            Self::NeverJudges => "never_judges",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpecificationExpectation {
    /// S 第 1 条：报错、一条都不施加。
    FailsWithoutApplying,
    /// 期望返回的根的六个字段。
    ReturnsRoot(VersionFields),
}

pub(super) struct SpecificationView<'view> {
    pub(super) chosen_root: &'view RootRecord,
    pub(super) records: &'view RecordTable,
    pub(super) in_flight_limit: usize,
    pub(super) verify_named_units: bool,
    pub(super) truths: &'view [RecordTruthOnDisk],
}

pub(super) fn record_carries_the_last_record_flag(record: &JournalRecord) -> bool {
    record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
}

/// 全表看：表里任何位置有同实例、同 txg、计数器更大的记录。
fn same_publish_record_follows_anywhere_in_the_table(
    record: &JournalRecord,
    records: &RecordTable,
) -> bool {
    records.values().any(|later| {
        later.instance == record.instance
            && later.checkpoint_txg == record.checkpoint_txg
            && later.counter > record.counter
    })
}

/// 连着看·缺号不停：从下一个计数器起按表里同实例记录的计数器升序看，看到 txg 越过这一条的就停；停之前见到同 txg 的为真。
fn same_publish_record_follows_before_a_later_txg(
    record: &JournalRecord,
    records: &RecordTable,
) -> bool {
    for later in records.values() {
        if later.instance != record.instance || later.counter <= record.counter {
            continue;
        }
        if later.checkpoint_txg > record.checkpoint_txg {
            return false;
        }
        if later.checkpoint_txg == record.checkpoint_txg {
            return true;
        }
    }
    false
}

/// 连着看·缺号即停：只看计数器一个接一个都在表里的那几条，缺号或 txg 越过就停；停之前见到同 txg 的为真。
fn same_publish_record_follows_without_a_gap(
    record: &JournalRecord,
    records: &RecordTable,
) -> bool {
    let largest_counter_in_the_table = records
        .keys()
        .filter(|(instance, _counter)| *instance == record.instance)
        .map(|(_instance, counter)| *counter)
        .max()
        .unwrap_or(record.counter);
    for counter in record.counter + 1..=largest_counter_in_the_table {
        let Some(later) = records.get(&(record.instance, counter)) else {
            return false;
        };
        if later.checkpoint_txg > record.checkpoint_txg {
            return false;
        }
        if later.checkpoint_txg == record.checkpoint_txg {
            return true;
        }
    }
    false
}

pub(super) fn same_publish_record_follows(
    reading: FollowReading,
    record: &JournalRecord,
    records: &RecordTable,
) -> bool {
    match reading {
        FollowReading::WholeTable => {
            same_publish_record_follows_anywhere_in_the_table(record, records)
        }
        FollowReading::ConsecutiveGapContinues => {
            same_publish_record_follows_before_a_later_txg(record, records)
        }
        FollowReading::ConsecutiveGapStops => {
            same_publish_record_follows_without_a_gap(record, records)
        }
        FollowReading::NeverJudges => false,
    }
}

/// S：照 D23（journal 的角色与格式） 已定项 14 五条与注 1、已定项 4 读者规则、已定项 7、I-8.3、I-8.6 的字面，对任意一张表给出期望的结局。
/// 分四条（登记 5.3）：1. 所选根那次发布带末条标志的多于一条 ⇒ 报错；2. 链首；3. 前缀逐条取、遇 (a)–(k) 任一情形停；
/// 4. 施加前缀里走到了末条的发布，期望的根 = 所选根把六个字段换成最后一次被施加的发布里带末条标志那一条的。
pub(super) fn specification_expectation(
    view: &SpecificationView<'_>,
    reading: FollowReading,
) -> SpecificationExpectation {
    let instance = view.chosen_root.instance;
    let chosen_txg = view.chosen_root.checkpoint_txg;
    let is_above_water = |record: &JournalRecord| record.checkpoint_txg > chosen_txg;
    // 第 1 条与第 2 条的锚点。
    let counters_of_the_root_publish_carrying_the_last_record_flag: Vec<u64> = view
        .records
        .values()
        .filter(|record| {
            record.instance == instance
                && record.checkpoint_txg == chosen_txg
                && record_carries_the_last_record_flag(record)
        })
        .map(|record| record.counter)
        .collect();
    if counters_of_the_root_publish_carrying_the_last_record_flag.len() > 1 {
        return SpecificationExpectation::FailsWithoutApplying;
    }
    let nothing_applied =
        SpecificationExpectation::ReturnsRoot(VersionFields::of_root(view.chosen_root));
    let chain_head_counter =
        match counters_of_the_root_publish_carrying_the_last_record_flag.first() {
            Some(anchor_counter) => anchor_counter + 1,
            None => {
                // 注 1：锚点读不出时链首接水位之上第一条可读记录，前提是它的序号为 1、txg = 所选根的 txg + 1。
                let Some(first_record_above_water) = view
                    .records
                    .values()
                    .filter(|record| record.instance == instance && is_above_water(record))
                    .min_by_key(|record| record.counter)
                else {
                    return nothing_applied;
                };
                let follows_the_chosen_root_directly = chosen_txg.0.checked_add(1)
                    == Some(first_record_above_water.checkpoint_txg.0)
                    && first_record_above_water.ordinal_within_publish.0 == 1;
                if !follows_the_chosen_root_directly {
                    return nothing_applied;
                }
                first_record_above_water.counter
            }
        };
    // 第 3 条：前缀。迭代次数的上界是在飞上限（(c)）；跨轮携带的是已进前缀的记录，每一轮至多加一条。
    let mut prefix: Vec<&JournalRecord> = Vec::new();
    let mut counter = chain_head_counter;
    while let Some(record) = view.records.get(&(instance, counter)) {
        // (a) 计数器缺号：`get` 取不到就出循环。(b) 该条不在水位之上：取「停」（条款没逐字写，推的）。
        if !is_above_water(record) {
            break;
        }
        // (c) 已取满在飞上限条。
        if prefix.len() == view.in_flight_limit {
            break;
        }
        // (d) 反向链不等于盘上前一条的头算出的链值；前一条不在盘上不判（I-8.6）。
        let previous_on_disk = view.truths.iter().find(|truth| {
            truth.instance == instance.0 && Some(truth.counter) == counter.checked_sub(1)
        });
        if previous_on_disk.is_some_and(|truth| truth.chain_value_of_header != record.back_chain) {
            break;
        }
        // (e) 序号 0。
        if record.ordinal_within_publish.0 == 0 {
            break;
        }
        let previous_record_of_the_open_publish = prefix
            .last()
            .filter(|previous| !record_carries_the_last_record_flag(previous));
        match previous_record_of_the_open_publish {
            // (f) 一次发布的首条（链首，或紧跟在带末条标志的那条之后）序号不是 1。
            None => {
                if record.ordinal_within_publish.0 != 1 {
                    break;
                }
            }
            Some(previous) => {
                // (g) 不是首条，而 txg 与本次发布前一条不同、或序号不是前一条加一。
                if record.checkpoint_txg != previous.checkpoint_txg
                    || u64::from(record.ordinal_within_publish.0)
                        != u64::from(previous.ordinal_within_publish.0) + 1
                {
                    break;
                }
                // (h) 本次发布前一条不带提交标记而该条换了事务号。
                if !previous.is_commit && record.transaction != previous.transaction {
                    break;
                }
            }
        }
        if record_carries_the_last_record_flag(record) {
            // (i) 带末条标志而不带提交标记。
            if !record.is_commit {
                break;
            }
            // (j) 带末条标志，且按所取读法之后还有同 (实例, txg) 的记录。
            if same_publish_record_follows(reading, record, view.records) {
                break;
            }
        }
        // (k) 验点名单元开着而该条有一份点名单元读不出或校验和不对。
        if view.verify_named_units {
            let named_units_all_verified = view
                .truths
                .iter()
                .find(|truth| truth.instance == instance.0 && truth.counter == counter)
                .is_some_and(|truth| truth.named_units_all_verified);
            if !named_units_all_verified {
                break;
            }
        }
        prefix.push(record);
        let Some(next_counter) = counter.checked_add(1) else {
            break;
        };
        counter = next_counter;
    }
    // 第 4 条：末尾没走到末条的那一段不施加。
    match prefix
        .iter()
        .rev()
        .find(|record| record_carries_the_last_record_flag(record))
    {
        Some(last_applied_record) => {
            SpecificationExpectation::ReturnsRoot(VersionFields::of_record(last_applied_record))
        }
        None => nothing_applied,
    }
}

/// 实现的结局与 S 的期望同不同（AS 三条）：比「报错 / 返回」的种类与返回根的六个字段。
pub(super) fn outcome_matches_the_expectation(
    outcome: &ReplayOutcome,
    expectation: SpecificationExpectation,
) -> bool {
    match (outcome, expectation) {
        (ReplayOutcome::Failed(_kind), SpecificationExpectation::FailsWithoutApplying) => true,
        (ReplayOutcome::Returned(returned), SpecificationExpectation::ReturnsRoot(fields)) => {
            VersionFields::of_root(&returned.root) == fields
        }
        (ReplayOutcome::Panicked, _)
        | (ReplayOutcome::Failed(_), SpecificationExpectation::ReturnsRoot(_))
        | (ReplayOutcome::Returned(_), SpecificationExpectation::FailsWithoutApplying) => false,
    }
}

/// 把 S 的期望当成一次调用的结局（AS 三条与 A-互 的阳性对照：拿另一种读法的 S 当实现）。
pub(super) fn outcome_of_the_expectation(
    chosen_root: &RootRecord,
    expectation: SpecificationExpectation,
) -> ReplayOutcome {
    match expectation {
        SpecificationExpectation::FailsWithoutApplying => {
            ReplayOutcome::Failed(FailureKind::RootPublishCarriesMoreThanOneLastRecordFlag)
        }
        SpecificationExpectation::ReturnsRoot(fields) => {
            let root = if fields == VersionFields::of_root(chosen_root) {
                *chosen_root
            } else {
                fields.to_root_rebuilt_over(chosen_root)
            };
            ReplayOutcome::Returned(ReturnedReplay {
                root,
                prefix_applied: 0,
                verification_passed: 0,
            })
        }
    }
}
