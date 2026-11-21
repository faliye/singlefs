//! 被判的实现（今天的 `replay_journal`，或阳性对照的一种假结局，登记 5.4）、理想恢复与真相，与合法历史一族一个状态的判定（不动点、J6Qn、投影之外的字段）。

use super::*;

// ── 被判的实现：今天的 `replay_journal`，或阳性对照的一种假结局（登记 5.4）

/// 阳性对照的假结局。每一种说它把真结局或理想结局改坏在哪里。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ControlKind {
    /// J0、A0：在指定的一态上 panic，别的态照真结局。
    PanicsOnTheDesignatedState,
    /// J1：理想恢复，凡施加过的，返回根的新根段改成时间线之外的值。
    OffTimelineRootWheneverApplied,
    /// J2：理想恢复，下一次发布有记录在盘上但不完整时多落一版。
    LandsOnePublishTooFarOverAnIncompletePublish,
    /// J3：不看上限，落到连续完整的最远一版。
    IgnoresTheInFlightLimit,
    /// J4：永远返回所选根。
    AlwaysReturnsTheChosenRoot,
    /// J5a、J5b：记录全在盘上时返回所选根、施加集为空，其余照理想恢复。
    ReturnsTheChosenRootWhenEveryRecordIsOnDisk,
    /// J5a 与 J5b 分得开：理想恢复，但施加集报成累计的 (0, j\*]。
    IdealWithTheCumulativeAppliedSet,
    /// J6：第一次照真结局，第二次调用的结局换成「再多落一版」（没有下一版时换成时间线之外的根）。
    SecondCallMovesOn,
    /// AS 三条与 A-互：拿 S 的某一种读法当实现。
    SpecificationWithReading(FollowReading),
    /// A-结构：返回根的新根段取表里一条同实例、水位之上、带提交标记而不带末条标志的记录的。
    RootFromARecordWithoutTheLastRecordFlag,
    /// A-限：报告的 `prefix_applied` 改成上限加一。
    ReportsOneRecordOverTheLimit,
    /// IA1：所选根 txg 是最大值时返回「没报错」。
    ReportsNoFailureOnTheLargestTxg,
    /// IA2：关掉点名验证时 `verification_passed` 报成 1。
    ReportsOneVerificationWithoutVerifying,
    /// J6Qn「(i)(ii) 之外都算违例」：第一次照真结局，第二次换成「报 `RootPublishCarriesMoreThanOneLastRecordFlag…`」。
    SecondCallReportsMoreThanOneLastRecordFlag,
    /// J6Qn「(ii) 要核 txg 顶格」：第一次照真结局，第二次换成「报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`」。
    SecondCallReportsTheChosenRootTxgHasNoSuccessor,
    /// J6 与 J6Qn 分得开：第一次取理想恢复（落到 j\*），第二次照真调用。
    IdealFirstCallThenTodaysSecondCall,
    /// J7a：理想恢复，施加过时把投影之外的一个字段换成落到的那一版 R_j 的。
    IdealWithAFieldOfTheLandedVersion(FieldOfTheRebuiltRoot),
    /// J7b：理想恢复，施加过时卸载记号抄所选根的。
    IdealCopyingTheChosenUnmountMarkerWhenApplied,
    /// J7c：理想恢复，一条没施加时卸载记号写成 `NotWritten…`；r5 档的 C8d 也是它（登记 r5 5.4）。
    IdealWritingNotWrittenWhenNothingIsApplied,
    /// C8a（登记 r5 5.4）：理想恢复，施加过时把实例表指针、分配记录树根、卸载记号三项都抄所选根的。
    IdealCopyingTheChosenRecordFieldsWhenApplied,
    /// C8b（登记 r5 5.4）：理想恢复，施加过时卸载记号一律写 [`UNMOUNT_MARKER_THE_SINGLE_MARKER_CONTROL_WRITES_WHEN_APPLIED`]。
    IdealWritingOneUnmountMarkerWhenApplied,
    /// 多盘「任一份在即在」（登记 r2 5.5）：理想恢复，但 j\* 只数 0 号盘那一份。
    IdealCountingOnlyTheFirstDeviceCopies,
    /// T-表（登记 r2 5.5）：扫盘建表之后删掉表里键最大的那条记录，直接建的表不动。
    ScanTableLosesItsLargestKeyRecord,
    /// T-行（登记 r2 5.5）：扫盘建表那一遍的实现结局换成 `Panicked`，表不动。
    ScanPassOutcomeBecomesAPanic,
    /// 对抗一族 J6Qn 按 I-8.12 拆开的四行（登记 r3 5.7 的 KF3）：第一次调用返回所选根，第二次调用报
    /// `RootPublishCarriesMoreThanOneLastRecordFlag…`；第二次既不是返回根、也不是 txg 顶格，每个判到的态都是 J6Qn 违例。
    FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag,
}

/// J7a 判的三个字段（登记 r2 第一节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FieldOfTheRebuiltRoot {
    InstanceTable,
    AllocationRecordTreeRoot,
    FilesystemIdentifier,
}

impl FieldOfTheRebuiltRoot {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::InstanceTable => "instance_table",
            Self::AllocationRecordTreeRoot => "allocation_record_tree_root",
            Self::FilesystemIdentifier => "filesystem_identifier",
        }
    }

    pub(super) const ALL: [Self; 3] = [
        Self::InstanceTable,
        Self::AllocationRecordTreeRoot,
        Self::FilesystemIdentifier,
    ];

    pub(super) const fn violations_counter(self) -> Counter {
        match self {
            Self::InstanceTable => Counter::J7aInstanceTableViolations,
            Self::AllocationRecordTreeRoot => Counter::J7aAllocationRecordTreeRootViolations,
            Self::FilesystemIdentifier => Counter::J7aFilesystemIdentifierViolations,
        }
    }

    /// 把 `root` 的这个字段换成 `source` 的。
    fn copy_from(self, root: &mut RootRecord, source: &RootRecord) {
        match self {
            Self::InstanceTable => root.instance_table = source.instance_table,
            Self::AllocationRecordTreeRoot => {
                root.allocation_record_tree_root = source.allocation_record_tree_root;
            }
            Self::FilesystemIdentifier => root.filesystem_identifier = source.filesystem_identifier,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Implementation {
    TodayReplayJournal,
    Control(ControlKind),
}

/// C8b 的假结局施加过时写的卸载记号（登记 r5 5.4：旧规则的第三项「由记录施加的一律不是卸载写的」）。
const UNMOUNT_MARKER_THE_SINGLE_MARKER_CONTROL_WRITES_WHEN_APPLIED: UnmountMarker =
    UnmountMarker::NotWrittenByTheUnmountSequence;

/// 一次重放施加的发布集合 (low, high]，high 同时是落到的版本。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AppliedInterval {
    pub(super) low: usize,
    pub(super) high: usize,
}

/// J5a（施加集读法）：(inner.low, inner.high] ⊆ (outer.low, outer.high]。空集包含于任何集合。
pub(super) fn applied_set_is_contained(inner: AppliedInterval, outer: AppliedInterval) -> bool {
    inner.low == inner.high || (outer.low <= inner.low && inner.high <= outer.high)
}

/// J5b（落到的版本读法）为假：子状态落到的版本晚于父状态落到的版本。
pub(super) fn landed_version_is_later(inner: AppliedInterval, outer: AppliedInterval) -> bool {
    inner.high > outer.high
}

/// 合法历史一族一个状态上、照条款算出来的量（不读实现的任何输出）。
struct LegalTruth {
    in_flight_limit: usize,
    farthest_version: usize,
    above_water_record_count: usize,
}

fn legal_truth_of(state: &LegalState<'_>, records: &RecordTable) -> LegalTruth {
    let in_flight_limit = state.history.ring.local_in_flight_limit();
    let chosen_fields = state.history.version_fields(state.chosen_version);
    let above_water_record_count = records
        .values()
        .filter(|record| {
            record.instance.0 == chosen_fields.instance
                && record.checkpoint_txg.0 > chosen_fields.checkpoint_txg
        })
        .count();
    LegalTruth {
        in_flight_limit,
        farthest_version: farthest_version_of(state, Some(in_flight_limit)),
        above_water_record_count,
    }
}

/// 落到时间线第 `version` 版的一次返回：没施加时原样返回所选根。
fn returned_at_version(
    state: &LegalState<'_>,
    chosen_root: &RootRecord,
    version: usize,
) -> ReturnedReplay {
    let prefix_applied = record_count_of_versions_after_the_chosen_one(state, version);
    ReturnedReplay {
        root: if version == state.chosen_version {
            *chosen_root
        } else {
            rebuilt_root_of_the_ideal_recovery(state.history, version, chosen_root)
        },
        prefix_applied,
        verification_passed: if state.verify_named_units {
            prefix_applied
        } else {
            0
        },
    }
}

/// 理想恢复落到时间线第 `version` 版时由记录重建的根：r5 档照 D23（journal 的角色与格式） 已定项 15 今天的原句（后三项取那次发布的
/// 记录里的、fsid 抄所选根，登记 r5 5.3），别的档照各自登记时的「投影之外抄所选根」（r1–r4 档一个字不改）。
fn rebuilt_root_of_the_ideal_recovery(
    history: &History,
    version: usize,
    chosen_root: &RootRecord,
) -> RootRecord {
    let landed_fields = history.version_fields(version);
    if history.run_profile.judges_new_root_segment_fields() {
        landed_fields.to_root_rebuilt_from_the_record_over(
            history
                .run_profile
                .record_fields_of_the_timeline_publish(version),
            chosen_root,
        )
    } else {
        landed_fields.to_root_rebuilt_over(chosen_root)
    }
}

/// 一条根，六个字段都不等于时间线上任何一版；投影之外照「由记录重建」抄 `rebuilt_over`。
fn off_timeline_root(
    checkpoint_txg: u64,
    distinguishing_number: u64,
    rebuilt_over: &RootRecord,
) -> RootRecord {
    VersionFields {
        instance: CHOSEN_INSTANCE,
        checkpoint_txg,
        segment: NewRootSegment::off_timeline(distinguishing_number, checkpoint_txg),
    }
    .to_root_rebuilt_over(rebuilt_over)
}

/// 合法历史一族里第一次调用的结局：今天的实现，或阳性对照的假结局。
fn first_outcome_on_a_legal_state(
    implementation: Implementation,
    state: &LegalState<'_>,
    truth: &LegalTruth,
    chosen_root: &RootRecord,
    records: &RecordTable,
    disk: &MemoryDisk<'_>,
    is_the_designated_state: bool,
) -> ReplayOutcome {
    let today = || call_replay_journal(disk, chosen_root, records, state.verify_named_units);
    let ideal = || returned_at_version(state, chosen_root, truth.farthest_version);
    let control = match implementation {
        Implementation::TodayReplayJournal => return today(),
        Implementation::Control(control) => control,
    };
    match control {
        ControlKind::PanicsOnTheDesignatedState => {
            if is_the_designated_state {
                ReplayOutcome::Panicked
            } else {
                today()
            }
        }
        ControlKind::OffTimelineRootWheneverApplied => {
            let mut returned = ideal();
            if truth.farthest_version > state.chosen_version {
                returned.root = off_timeline_root(returned.root.checkpoint_txg.0, 1, chosen_root);
            }
            ReplayOutcome::Returned(returned)
        }
        ControlKind::LandsOnePublishTooFarOverAnIncompletePublish => {
            let next_version = truth.farthest_version + 1;
            let lands_one_too_far = next_version < state.history.chosen_instance_publish_count
                && publish_has_a_record_on_disk(state, next_version)
                && !publish_is_completely_on_disk(state, next_version);
            ReplayOutcome::Returned(returned_at_version(
                state,
                chosen_root,
                if lands_one_too_far {
                    next_version
                } else {
                    truth.farthest_version
                },
            ))
        }
        ControlKind::IgnoresTheInFlightLimit => ReplayOutcome::Returned(returned_at_version(
            state,
            chosen_root,
            farthest_version_of(state, None),
        )),
        ControlKind::AlwaysReturnsTheChosenRoot => ReplayOutcome::Returned(returned_at_version(
            state,
            chosen_root,
            state.chosen_version,
        )),
        ControlKind::ReturnsTheChosenRootWhenEveryRecordIsOnDisk => {
            let every_record_is_on_disk = (0..state.presence.len())
                .all(|record_index| record_has_a_copy_on_disk(state, record_index));
            ReplayOutcome::Returned(returned_at_version(
                state,
                chosen_root,
                if every_record_is_on_disk {
                    state.chosen_version
                } else {
                    truth.farthest_version
                },
            ))
        }
        ControlKind::IdealWithTheCumulativeAppliedSet => ReplayOutcome::Returned(ideal()),
        ControlKind::ReportsNoFailureOnTheLargestTxg => match today() {
            ReplayOutcome::Failed(_kind) => ReplayOutcome::Returned(returned_at_version(
                state,
                chosen_root,
                state.chosen_version,
            )),
            outcome @ (ReplayOutcome::Panicked | ReplayOutcome::Returned(_)) => outcome,
        },
        ControlKind::ReportsOneVerificationWithoutVerifying => match today() {
            ReplayOutcome::Returned(mut returned) => {
                returned.verification_passed = 1;
                ReplayOutcome::Returned(returned)
            }
            outcome @ (ReplayOutcome::Panicked | ReplayOutcome::Failed(_)) => outcome,
        },
        ControlKind::IdealFirstCallThenTodaysSecondCall => ReplayOutcome::Returned(ideal()),
        ControlKind::IdealWithAFieldOfTheLandedVersion(field) => {
            let mut returned = ideal();
            if truth.farthest_version > state.chosen_version {
                field.copy_from(
                    &mut returned.root,
                    &state.history.timeline_root(truth.farthest_version),
                );
            }
            ReplayOutcome::Returned(returned)
        }
        ControlKind::IdealCopyingTheChosenUnmountMarkerWhenApplied => {
            let mut returned = ideal();
            if truth.farthest_version > state.chosen_version {
                returned.root.unmount_marker = chosen_root.unmount_marker;
            }
            ReplayOutcome::Returned(returned)
        }
        ControlKind::IdealWritingNotWrittenWhenNothingIsApplied => {
            let mut returned = ideal();
            if truth.farthest_version == state.chosen_version {
                returned.root.unmount_marker = UnmountMarker::NotWrittenByTheUnmountSequence;
            }
            ReplayOutcome::Returned(returned)
        }
        ControlKind::IdealCopyingTheChosenRecordFieldsWhenApplied => {
            let mut returned = ideal();
            if truth.farthest_version > state.chosen_version {
                returned.root.instance_table = chosen_root.instance_table;
                returned.root.allocation_record_tree_root = chosen_root.allocation_record_tree_root;
                returned.root.unmount_marker = chosen_root.unmount_marker;
            }
            ReplayOutcome::Returned(returned)
        }
        ControlKind::IdealWritingOneUnmountMarkerWhenApplied => {
            let mut returned = ideal();
            if truth.farthest_version > state.chosen_version {
                returned.root.unmount_marker =
                    UNMOUNT_MARKER_THE_SINGLE_MARKER_CONTROL_WRITES_WHEN_APPLIED;
            }
            ReplayOutcome::Returned(returned)
        }
        ControlKind::IdealCountingOnlyTheFirstDeviceCopies => {
            let first_device_view = LegalState {
                history: state.history,
                presence: state.presence,
                mirroring: JournalMirroring::FirstDeviceOnly,
                residue: state.residue,
                chosen_version: state.chosen_version,
                verify_named_units: state.verify_named_units,
            };
            ReplayOutcome::Returned(returned_at_version(
                state,
                chosen_root,
                farthest_version_of(&first_device_view, Some(truth.in_flight_limit)),
            ))
        }
        ControlKind::SecondCallMovesOn
        | ControlKind::SpecificationWithReading(_)
        | ControlKind::RootFromARecordWithoutTheLastRecordFlag
        | ControlKind::ReportsOneRecordOverTheLimit
        | ControlKind::SecondCallReportsMoreThanOneLastRecordFlag
        | ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor
        | ControlKind::FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag
        | ControlKind::ScanTableLosesItsLargestKeyRecord
        | ControlKind::ScanPassOutcomeBecomesAPanic => today(),
    }
}

/// J6 的第二次调用：拿第一次返回的根当所选根、同一张表、同一个读者、同一个开关再调一次。
fn second_outcome(
    implementation: Implementation,
    first_root: &RootRecord,
    next_timeline_version_root: Option<RootRecord>,
    records: &RecordTable,
    disk: &MemoryDisk<'_>,
    verify_named_units: bool,
) -> ReplayOutcome {
    match implementation {
        Implementation::Control(ControlKind::SecondCallMovesOn) => {
            ReplayOutcome::Returned(ReturnedReplay {
                root: next_timeline_version_root.unwrap_or_else(|| {
                    off_timeline_root(first_root.checkpoint_txg.0, 2, first_root)
                }),
                prefix_applied: 1,
                verification_passed: 0,
            })
        }
        Implementation::Control(
            ControlKind::SecondCallReportsMoreThanOneLastRecordFlag
            | ControlKind::FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag,
        ) => ReplayOutcome::Failed(FailureKind::RootPublishCarriesMoreThanOneLastRecordFlag),
        Implementation::Control(ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor) => {
            ReplayOutcome::Failed(FailureKind::ChosenRootTxgHasNoSuccessor)
        }
        Implementation::TodayReplayJournal | Implementation::Control(_) => {
            call_replay_journal(disk, first_root, records, verify_named_units)
        }
    }
}

/// J6（不动点）：判第二次是不是「返回根且与第一次的根全字段相等」，按前提真假分两行记；为假的态上另看第三遍与第二遍相不相等（只报不判）。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数都是这一判要用的一样东西，凑不成一个带不变量的概念"
)]
pub(super) fn judge_the_fixed_point(
    implementation: Implementation,
    run_profile: RunProfile,
    first: &ReturnedReplay,
    next_timeline_version_root: Option<RootRecord>,
    premise_holds: bool,
    table_keeps_the_txg_order: Option<bool>,
    records: &RecordTable,
    disk: &MemoryDisk<'_>,
    verify_named_units: bool,
    describe_witness: &dyn Fn() -> String,
    tally: &mut Tally,
) {
    let second = second_outcome(
        implementation,
        &first.root,
        next_timeline_version_root,
        records,
        disk,
        verify_named_units,
    );
    let fixed_point_holds = matches!(&second, ReplayOutcome::Returned(second_returned) if second_returned.root == first.root);
    tally.add(if premise_holds {
        Counter::FixedPointPremiseHoldsJudged
    } else {
        Counter::FixedPointPremiseFailsJudged
    });
    if run_profile.judges_j6qn() {
        if first.root.checkpoint_txg.0 == u64::MAX {
            tally.add(Counter::FirstReturnedRootTxgIsTheLargest);
        }
        judge_the_fixed_point_under_qn(
            &first.root,
            &second,
            fixed_point_holds,
            premise_holds,
            table_keeps_the_txg_order,
            describe_witness,
            tally,
        );
    }
    if fixed_point_holds {
        return;
    }
    let describe_with_the_second_call = || {
        format!(
            "{};second={}",
            describe_witness(),
            describe_outcome(&second)
        )
    };
    tally.add_violation(
        if premise_holds {
            Counter::FixedPointPremiseHoldsViolations
        } else {
            Counter::FixedPointPremiseFailsViolations
        },
        &describe_with_the_second_call,
    );
    let third_equals_second = match &second {
        ReplayOutcome::Returned(second_returned) => {
            let third =
                call_replay_journal(disk, &second_returned.root, records, verify_named_units);
            matches!(&third, ReplayOutcome::Returned(third_returned) if third_returned.root == second_returned.root)
        }
        ReplayOutcome::Panicked | ReplayOutcome::Failed(_) => false,
    };
    tally.add(if third_equals_second {
        Counter::FixedPointThirdCallEqualsSecond
    } else {
        Counter::FixedPointThirdCallDiffersOrIsMissing
    });
}

/// J6Qn (ii)（登记 r2 第一节）：第二次报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`，且判据自己看到 ρ_1 的 txg 是 u64 最大值
/// （「因所选根 txg 顶格」由判据核，不只认报错的字样；不取 Qw）。
fn second_call_failed_because_the_first_root_txg_is_the_largest(
    second: &ReplayOutcome,
    first_root: &RootRecord,
) -> bool {
    let first_root_txg_is_the_largest = first_root.checkpoint_txg.0 == u64::MAX;
    let second_call_reports_no_successor = matches!(
        second,
        ReplayOutcome::Failed(FailureKind::ChosenRootTxgHasNoSuccessor)
    );
    second_call_reports_no_successor && first_root_txg_is_the_largest
}

/// I-8.12（同一实例里 journal 记录的 txg 随计数器不减；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 9 行）
/// 在「这一态交给 `replay_journal` 的那张记录表」上成不成立（登记 r3 第一节「Q3 的拆法」）：表按（实例，计数器）升序摊开，同一实例里
/// 相邻两条记录后一条的 txg 小于前一条即不成立（相邻不减与任两条不减等价）。`parse` 拒收的序号 0 进不了表，判在表上就不看它。
/// 本地写法，不调 `crates/` 里 Verus 前件 `journal_txg_does_not_decrease_with_the_counter_within_an_instance`。
pub(super) fn record_table_keeps_the_txg_order(records: &RecordTable) -> bool {
    let mut previous_record_of_the_same_instance: Option<&JournalRecord> = None;
    for ((instance, _counter), record) in records {
        if let Some(previous_record) = previous_record_of_the_same_instance {
            if previous_record.instance == *instance
                && record.checkpoint_txg < previous_record.checkpoint_txg
            {
                return false;
            }
        }
        previous_record_of_the_same_instance = Some(record);
    }
    true
}

/// J6Qn 的前提真 / 假与 I-8.12 成不成立，四个格各自的「判了几态」与「违例几态」两个量（登记 r3 5.6）。
pub(super) const fn j6qn_cell_counters(
    premise_holds: bool,
    txg_order_holds: bool,
) -> (Counter, Counter) {
    match (premise_holds, txg_order_holds) {
        (true, true) => (
            Counter::FixedPointQnPremiseHoldsTxgOrderHoldsJudged,
            Counter::FixedPointQnPremiseHoldsTxgOrderHoldsViolations,
        ),
        (true, false) => (
            Counter::FixedPointQnPremiseHoldsTxgOrderFailsJudged,
            Counter::FixedPointQnPremiseHoldsTxgOrderFailsViolations,
        ),
        (false, true) => (
            Counter::FixedPointQnPremiseFailsTxgOrderHoldsJudged,
            Counter::FixedPointQnPremiseFailsTxgOrderHoldsViolations,
        ),
        (false, false) => (
            Counter::FixedPointQnPremiseFailsTxgOrderFailsJudged,
            Counter::FixedPointQnPremiseFailsTxgOrderFailsViolations,
        ),
    }
}

/// J6Qn（登记 r2 第一节，判决 S6 取 Qn）：第二次 (i) 返回根且与 ρ_1 全字段相等，或 (ii) 因 ρ_1 的 txg 顶格报
/// `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`，记相等；别的报错与 panic 一律违例。前提真 / 假两行照原 J6 分。
/// `table_keeps_the_txg_order` 给了（r3 档的对抗一族）时，同一次判再按这张表满不满足 I-8.12 记进四格里的一格（登记 r3 5.6），
/// 两行原有的计数一个不动。
fn judge_the_fixed_point_under_qn(
    first_root: &RootRecord,
    second: &ReplayOutcome,
    fixed_point_holds: bool,
    premise_holds: bool,
    table_keeps_the_txg_order: Option<bool>,
    describe_witness: &dyn Fn() -> String,
    tally: &mut Tally,
) {
    tally.add(if premise_holds {
        Counter::FixedPointQnPremiseHoldsJudged
    } else {
        Counter::FixedPointQnPremiseFailsJudged
    });
    let split_cell = table_keeps_the_txg_order
        .map(|txg_order_holds| j6qn_cell_counters(premise_holds, txg_order_holds));
    if let Some((judged_counter, _violations_counter)) = split_cell {
        tally.add(judged_counter);
    }
    let counts_as_equal_under_qn = fixed_point_holds
        || second_call_failed_because_the_first_root_txg_is_the_largest(second, first_root);
    if counts_as_equal_under_qn {
        return;
    }
    let describe_with_the_second_call = || {
        format!(
            "{};first_root_txg={};second={}",
            describe_witness(),
            first_root.checkpoint_txg.0,
            describe_outcome(second)
        )
    };
    tally.add_violation(
        if premise_holds {
            Counter::FixedPointQnPremiseHoldsViolations
        } else {
            Counter::FixedPointQnPremiseFailsViolations
        },
        &describe_with_the_second_call,
    );
    if let Some((_judged_counter, violations_counter)) = split_cell {
        tally.add_violation(violations_counter, &describe_with_the_second_call);
    }
}

/// J7（登记 r2 第一节，判决 S1 后半）：凡第一次调用返回根 ρ 的态都判。J7a：实例表指针、分配记录树根、fsid 各等于所选根的；
/// 投影不等于所选根的（由记录重建过）J7b：卸载记号是 `NotWritten…`；投影等于所选根的（一条没施加）J7c：卸载记号等于所选根的。
/// 「重建过没有」由判据比投影定，不读报告的 `prefix_applied`。
pub(super) fn judge_the_fields_beyond_the_projection(
    chosen_root: &RootRecord,
    returned_root: &RootRecord,
    describe_witness: &dyn Fn() -> String,
    tally: &mut Tally,
) {
    let projection_equals_the_chosen_one =
        VersionFields::of_root(returned_root) == VersionFields::of_root(chosen_root);
    let describe_with_both_roots = || {
        format!(
            "{};chosen_root={};returned_root={};projection_equals_the_chosen_one={projection_equals_the_chosen_one}",
            describe_witness(),
            FieldsBeyondTheProjection::of_root(chosen_root).describe(),
            FieldsBeyondTheProjection::of_root(returned_root).describe()
        )
    };
    tally.add(Counter::CopiedFieldsJudged);
    if returned_root.instance_table != chosen_root.instance_table {
        tally.add_violation(
            Counter::J7aInstanceTableViolations,
            &describe_with_both_roots,
        );
    }
    if returned_root.allocation_record_tree_root != chosen_root.allocation_record_tree_root {
        tally.add_violation(
            Counter::J7aAllocationRecordTreeRootViolations,
            &describe_with_both_roots,
        );
    }
    if returned_root.filesystem_identifier != chosen_root.filesystem_identifier {
        tally.add_violation(
            Counter::J7aFilesystemIdentifierViolations,
            &describe_with_both_roots,
        );
    }
    let expected_unmount_marker = if projection_equals_the_chosen_one {
        chosen_root.unmount_marker
    } else {
        UnmountMarker::NotWrittenByTheUnmountSequence
    };
    let (judged_counter, violations_counter) = if projection_equals_the_chosen_one {
        (Counter::J7cJudged, Counter::J7cViolations)
    } else {
        (Counter::J7bJudged, Counter::J7bViolations)
    };
    tally.add(judged_counter);
    if returned_root.unmount_marker != expected_unmount_marker {
        tally.add_violation(violations_counter, &describe_with_both_roots);
    }
}

/// J8（登记 r5 5.3，照 D23（journal 的角色与格式） 已定项 15 今天的原句重新登记）：凡第一次调用返回根 ρ 的态都判。
/// J8b：ρ 的 fsid 等于所选根的。投影等于所选根的（一条没施加）J8c：fsid、实例表指针、分配记录树根、卸载记号四项都等于所选根的。
/// 投影不等于所选根的（由记录重建过）J8a 三格：实例表指针、分配记录树根、卸载记号各等于生成器真相里新根段投影等于 ρ 的投影的那条记录的；
/// 生成器真相里没有这样一条记录的不判 J8a，记「无出处」（只报）。「重建过没有」由判据比投影定，不读报告的 `prefix_applied`。
/// `landed_version_root` 是 ρ 落到的那一版时间线根（对抗一族与落点不在时间线上的给 `None`），只进见证。
pub(super) fn judge_the_fields_beyond_the_projection_against_the_generated_records(
    chosen_root: &RootRecord,
    returned_root: &RootRecord,
    record_truth: &GeneratedRecordTruth,
    landed_version_root: Option<RootRecord>,
    describe_witness: &dyn Fn() -> String,
    tally: &mut Tally,
) {
    let returned_projection = VersionFields::of_root(returned_root);
    let projection_equals_the_chosen_one =
        returned_projection == VersionFields::of_root(chosen_root);
    let source_of_the_returned_projection = record_truth.fields_of(returned_projection);
    let describe_with_both_roots = || {
        format!(
            "{};chosen_root={};returned_root={};record_of_the_returned_projection={};landed_version_root={};projection_equals_the_chosen_one={projection_equals_the_chosen_one}",
            describe_witness(),
            FieldsBeyondTheProjection::of_root(chosen_root).describe(),
            FieldsBeyondTheProjection::of_root(returned_root).describe(),
            source_of_the_returned_projection
                .map_or_else(|| "none".to_string(), RecordFieldsBeyondTheProjection::describe),
            landed_version_root.map_or_else(
                || "none".to_string(),
                |landed_root| FieldsBeyondTheProjection::of_root(&landed_root).describe()
            )
        )
    };
    tally.add(Counter::NewRootSegmentFieldsJudged);
    let expected_filesystem_identifier = chosen_root.filesystem_identifier;
    if returned_root.filesystem_identifier != expected_filesystem_identifier {
        tally.add_violation(Counter::J8bViolations, &describe_with_both_roots);
    }
    if projection_equals_the_chosen_one {
        tally.add(Counter::J8cJudged);
        let returned_equals_the_chosen_root_beyond_the_projection =
            FieldsBeyondTheProjection::of_root(returned_root)
                == FieldsBeyondTheProjection::of_root(chosen_root);
        if !returned_equals_the_chosen_root_beyond_the_projection {
            tally.add_violation(Counter::J8cViolations, &describe_with_both_roots);
        }
        return;
    }
    let Some(fields_from_the_new_root_segment) = source_of_the_returned_projection else {
        tally.add_violation(
            Counter::NewRootSegmentFieldsWithoutASource,
            &describe_with_both_roots,
        );
        return;
    };
    tally.add(Counter::J8aJudged);
    let expected_fields_of_the_rebuilt_root = fields_from_the_new_root_segment;
    let returned_fields = RecordFieldsBeyondTheProjection::of_root(returned_root);
    for (field_differs, violations_counter) in [
        (
            returned_fields.instance_table != expected_fields_of_the_rebuilt_root.instance_table,
            Counter::J8aInstanceTableViolations,
        ),
        (
            returned_fields.allocation_record_tree_root
                != expected_fields_of_the_rebuilt_root.allocation_record_tree_root,
            Counter::J8aAllocationRecordTreeRootViolations,
        ),
        (
            returned_fields.unmount_marker != expected_fields_of_the_rebuilt_root.unmount_marker,
            Counter::J8aUnmountMarkerViolations,
        ),
    ] {
        if field_differs {
            tally.add_violation(violations_counter, &describe_with_both_roots);
        }
    }
}

/// 这个域的状态进不进「S 自己的对照」（登记 5.4：L、L 小环、U 开着验证、X、E1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpecificationSelfCheck {
    Judged,
    NotJudged,
}

/// 一个合法历史状态的写法（逐态行的 `state=` 与每条违例的见证）：形状、每条记录在盘上与否、1 号盘那几份、残留、所选版本、j\*、第一次调用的结局。
fn describe_a_legal_state_with_its_first_outcome(
    state: &LegalState<'_>,
    farthest_version: usize,
    first: &ReplayOutcome,
) -> String {
    let presence: Vec<String> = state
        .presence
        .iter()
        .map(|record_presence| match record_presence {
            RecordPresence::NotOnDisk => "-".to_string(),
            RecordPresence::OnDisk(unit_state) => unit_state.label().to_string(),
        })
        .collect();
    let second_device_presence = match state.mirroring {
        JournalMirroring::FirstDeviceOnly => String::new(),
        JournalMirroring::BothDevices {
            second_device_copy_is_on_disk,
        } => format!(
            ";on_second_device={}",
            second_device_copy_is_on_disk
                .iter()
                .map(|is_on_disk| if *is_on_disk { "good" } else { "-" })
                .collect::<Vec<&str>>()
                .join(",")
        ),
    };
    format!(
        "shape={};on_disk={}{second_device_presence};residue_at={};chosen={};farthest={};verify={};first={}",
        state.history.shape_description,
        presence.join(","),
        state.residue.map_or_else(
            || "none".to_string(),
            |residue| residue.record_index.to_string()
        ),
        state.chosen_version,
        farthest_version,
        state.verify_named_units,
        describe_outcome(first)
    )
}

/// 判合法历史一族的一个状态：J0–J4、J6、IA1、IA2、域非空见证、在飞上限的分布、S 自己的对照。记录表怎么建由档的 `table_source` 定
/// （登记 r2 5.4）：`Both` 时两张表各判一遍、逐态比，汇总的计数取直接建表那一遍。
/// 返回这一次重放施加的发布区间（C 域的状态对要它）；panic 或落点不在时间线上时是 `None`。
pub(super) fn judge_a_legal_state(
    state: &LegalState<'_>,
    implementation: Implementation,
    specification_self_check: SpecificationSelfCheck,
    is_the_designated_state: bool,
    tally: &mut Tally,
) -> Option<AppliedInterval> {
    let probe_decision = begin_probing_a_state(tally);
    if matches!(probe_decision, ProbeDecision::SkipThisState) {
        return None;
    }
    let run_profile = state.history.run_profile;
    let (direct_table, disk, truths) = record_table_and_disk_of(state);
    let judge_on = |table: &RecordTable,
                    implementation_of_the_pass: Implementation,
                    designated_in_the_pass: bool,
                    tally_of_the_pass: &mut Tally| {
        judge_a_legal_state_on_a_table(
            state,
            implementation_of_the_pass,
            specification_self_check,
            designated_in_the_pass,
            table,
            &disk,
            &truths,
            tally_of_the_pass,
        )
    };
    let judgement = match run_profile.table_source {
        RecordTableSource::Direct => judge_on(
            &direct_table,
            implementation,
            is_the_designated_state,
            tally,
        ),
        RecordTableSource::Scanned => {
            let scanned_table = scanned_record_table_of(&disk, implementation);
            assert_the_scan_table_of_a_fully_landed_state(
                state,
                &scanned_table,
                implementation,
                tally,
            );
            judge_on(
                &scanned_table,
                implementation,
                is_the_designated_state,
                tally,
            )
        }
        RecordTableSource::Both => {
            let scanned_table = scanned_record_table_of(&disk, implementation);
            let counts_before_the_state = tally.counts.clone();
            let (scan_pass_implementation, scan_pass_is_designated) =
                implementation_and_designation_of_the_scan_pass(
                    implementation,
                    is_the_designated_state,
                );
            let mut scanned_tally = Tally::new();
            let scanned_judgement = judge_on(
                &scanned_table,
                scan_pass_implementation,
                scan_pass_is_designated,
                &mut scanned_tally,
            );
            let scanned_summary = JudgedRowSummary {
                first_outcome: scanned_judgement.first_outcome,
                counter_labels: counter_labels_a_state_added_to(
                    &scanned_tally,
                    &Tally::new().counts,
                ),
            };
            let direct_judgement = judge_on(
                &direct_table,
                implementation,
                is_the_designated_state,
                tally,
            );
            let direct_summary = JudgedRowSummary {
                first_outcome: direct_judgement.first_outcome,
                counter_labels: counter_labels_a_state_added_to(tally, &counts_before_the_state),
            };
            let describe_state = || {
                describe_a_legal_state_with_its_first_outcome(
                    state,
                    direct_judgement.farthest_version,
                    &direct_judgement.first_outcome,
                )
            };
            add_the_comparison_of_two_judged_tables(
                tally,
                (Counter::TablesDiffer, Counter::RowsDiffer),
                (&direct_table, &direct_summary),
                (&scanned_table, &scanned_summary),
                &describe_state,
            );
            if !direct_table.is_empty() {
                tally.add(Counter::DirectTableHoldsARecord);
            }
            assert_the_scan_table_of_a_fully_landed_state(
                state,
                &scanned_table,
                implementation,
                tally,
            );
            if run_profile.reported_disk_order == ReportedDiskOrder::SecondDeviceBeforeFirst {
                let (reported_order_table, _reported_order_disk, reported_order_truths) =
                    record_table_and_disk_of_in_device_order(
                        state,
                        run_profile.reported_disk_order.device_numbers(),
                    );
                let mut reported_order_tally = Tally::new();
                let reported_order_judgement = judge_a_legal_state_on_a_table(
                    state,
                    implementation,
                    specification_self_check,
                    is_the_designated_state,
                    &reported_order_table,
                    &disk,
                    &reported_order_truths,
                    &mut reported_order_tally,
                );
                let reported_order_summary = JudgedRowSummary {
                    first_outcome: reported_order_judgement.first_outcome,
                    counter_labels: counter_labels_a_state_added_to(
                        &reported_order_tally,
                        &Tally::new().counts,
                    ),
                };
                add_the_comparison_of_two_judged_tables(
                    tally,
                    (
                        Counter::TablesDifferInReportedOrder,
                        Counter::RowsDifferInReportedOrder,
                    ),
                    (&reported_order_table, &reported_order_summary),
                    (&scanned_table, &scanned_summary),
                    &describe_state,
                );
            }
            direct_judgement
        }
    };
    finish_probing_a_state(probe_decision, tally, &|| {
        describe_a_legal_state_with_its_first_outcome(
            state,
            judgement.farthest_version,
            &judgement.first_outcome,
        )
    });
    judgement.interval
}

/// [`judge_a_legal_state`] 在一张给定的记录表上判一个状态：不管探针、不管两种建表法怎么比，只数这一遍判出来的量。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数都是这一判要用的一样东西，凑不成一个带不变量的概念"
)]
fn judge_a_legal_state_on_a_table(
    state: &LegalState<'_>,
    implementation: Implementation,
    specification_self_check: SpecificationSelfCheck,
    is_the_designated_state: bool,
    records: &RecordTable,
    disk: &MemoryDisk<'_>,
    truths: &[RecordTruthOnDisk],
    tally: &mut Tally,
) -> LegalJudgement {
    let truth = legal_truth_of(state, records);
    let history = state.history;
    let chosen_version = state.chosen_version;
    let chosen_root = history.timeline_root(chosen_version);
    let first = first_outcome_on_a_legal_state(
        implementation,
        state,
        &truth,
        &chosen_root,
        records,
        disk,
        is_the_designated_state,
    );
    let describe_witness =
        || describe_a_legal_state_with_its_first_outcome(state, truth.farthest_version, &first);

    tally.add(Counter::States);
    if chosen_version > 0 {
        tally.add(Counter::ChosenRootIsALaterVersion);
    }
    tally.above_water_maximum = tally
        .above_water_maximum
        .max(truth.above_water_record_count);
    if truth.above_water_record_count > truth.in_flight_limit {
        tally.add(Counter::AboveWaterOverLimit);
    }
    if truth.above_water_record_count == truth.in_flight_limit {
        tally.add(Counter::AboveWaterAtLimit);
    }
    if truth.farthest_version > chosen_version {
        tally.add(Counter::FarthestAtLeastOne);
    }
    if truth.farthest_version > chosen_version + 1 {
        tally.add(Counter::FarthestAtLeastTwo);
    }

    if specification_self_check == SpecificationSelfCheck::Judged {
        let view = SpecificationView {
            chosen_root: &chosen_root,
            records,
            in_flight_limit: truth.in_flight_limit,
            verify_named_units: state.verify_named_units,
            truths,
        };
        let expected_by_the_farthest_version =
            SpecificationExpectation::ReturnsRoot(history.version_fields(truth.farthest_version));
        let mismatch_counters = [
            Counter::SpecificationWholeTableMismatch,
            Counter::SpecificationGapContinuesMismatch,
            Counter::SpecificationGapStopsMismatch,
        ];
        for (reading, mismatch_counter) in
            FollowReading::REGISTERED.into_iter().zip(mismatch_counters)
        {
            if specification_expectation(&view, reading) != expected_by_the_farthest_version {
                tally.add_violation(mismatch_counter, &describe_witness);
            }
        }
    }

    // IA1、IA2 是实现锚点，不是条款判据（登记 5.3 末段）。
    if chosen_root.checkpoint_txg.0 == u64::MAX {
        tally.add(Counter::Ia1Judged);
        if first != ReplayOutcome::Failed(FailureKind::ChosenRootTxgHasNoSuccessor) {
            tally.add_violation(Counter::Ia1Violations, &describe_witness);
        }
    }
    if !state.verify_named_units {
        tally.add(Counter::Ia2Judged);
        if matches!(&first, ReplayOutcome::Returned(returned) if returned.verification_passed != 0)
        {
            tally.add_violation(Counter::Ia2Violations, &describe_witness);
        }
    }

    // 落点 j：报错记 j = c；返回的根六个字段等于 R_j 的那个 j；没有这样的 j 是「不在时间线上」。
    let landing: Option<usize> = match &first {
        ReplayOutcome::Panicked => {
            tally.add_violation(Counter::Panicked, &describe_witness);
            None
        }
        ReplayOutcome::Failed(_kind) => {
            tally.add_violation(Counter::Failed, &describe_witness);
            tally.add(Counter::FixedPointFirstCallFailed);
            Some(chosen_version)
        }
        ReplayOutcome::Returned(returned) => {
            let landing =
                history.timeline_version_with_fields(VersionFields::of_root(&returned.root));
            if landing.is_none() {
                tally.add(Counter::OffTimelineLandings);
            }
            let next_timeline_version_root = landing
                .map(|version| version + 1)
                .filter(|next_version| *next_version < history.chosen_instance_publish_count)
                .map(|next_version| {
                    history
                        .version_fields(next_version)
                        .to_root_rebuilt_over(&returned.root)
                });
            judge_the_fixed_point(
                implementation,
                history.run_profile,
                returned,
                next_timeline_version_root,
                truth.above_water_record_count <= truth.in_flight_limit,
                None,
                records,
                disk,
                state.verify_named_units,
                &describe_witness,
                tally,
            );
            if history.run_profile.judges_copied_fields() {
                judge_the_fields_beyond_the_projection(
                    &chosen_root,
                    &returned.root,
                    &describe_witness,
                    tally,
                );
            }
            if history.run_profile.judges_new_root_segment_fields() {
                judge_the_fields_beyond_the_projection_against_the_generated_records(
                    &chosen_root,
                    &returned.root,
                    &history.record_truth,
                    landing.map(|version| history.timeline_root(version)),
                    &describe_witness,
                    tally,
                );
            }
            landing
        }
    };

    // J1 返回的根：落点存在且不早于所选根。
    let returned_root_holds = landing.is_some_and(|version| version >= chosen_version);
    if !returned_root_holds {
        tally.add_violation(Counter::ReturnedRootViolations, &describe_witness);
    }
    // J2 根加盘：J1 为真，且 (c, j] 里每次发布都完整在盘上。
    let root_plus_disk_holds = returned_root_holds
        && landing.is_some_and(|version| {
            (chosen_version + 1..=version)
                .all(|applied_version| publish_is_completely_on_disk(state, applied_version))
        });
    if !root_plus_disk_holds {
        tally.add_violation(Counter::RootPlusDiskViolations, &describe_witness);
    }
    // J3 在飞上限：J1 为真，且 (c, j] 里各次发布的记录条数之和不超过在飞上限。
    let within_the_in_flight_limit = returned_root_holds
        && landing.is_some_and(|version| {
            let applied_record_count =
                record_count_of_versions_after_the_chosen_one(state, version);
            applied_record_count <= truth.in_flight_limit
        });
    if !within_the_in_flight_limit {
        tally.add_violation(Counter::InFlightLimitViolations, &describe_witness);
    }
    // J4 最远一版：j = j*。
    if landing != Some(truth.farthest_version) {
        tally.add_violation(Counter::FarthestVersionViolations, &describe_witness);
    }
    // 域非空见证：实现施加过至少一次、两次以上发布。
    if landing.is_some_and(|version| version > chosen_version) {
        tally.add(Counter::AppliedAtLeastOne);
    }
    if landing.is_some_and(|version| version > chosen_version + 1) {
        tally.add(Counter::AppliedAtLeastTwo);
    }

    let interval = landing
        .filter(|version| *version >= chosen_version)
        .map(|version| AppliedInterval {
            low: if implementation
                == Implementation::Control(ControlKind::IdealWithTheCumulativeAppliedSet)
            {
                0
            } else {
                chosen_version
            },
            high: version,
        });
    LegalJudgement {
        interval,
        first_outcome: first,
        farthest_version: truth.farthest_version,
    }
}

#[cfg(test)]
mod root_choice_fault_tests;
