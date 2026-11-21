//! 写死的几段历史（段内真子集全枚举）：每个崩溃状态都恢复进提交过的版本，抬回收下限进回退空档的历史在往前回退下干净，崩溃状态落进零单元发布与第一个文件、落不进冷启动与被拒的崩溃恢复。

use super::*;

/// 写死的一段历史：mkfs → 可写挂载 → 第一个文件 → 覆盖写 → 可写挂载 → 覆盖写；写数不超过 4 的段整段枚举（全部真子集），
/// 更长的段各抽 8 个。今天的代码上每个崩溃状态都恢复到模型提交过的某一版、池级 checker 与记录核对器一条都不红。
///
/// 判别力靠的就是这一条：段内真子集全枚举，所以「记录写持久了而根槽没持久」与「根槽持久了而它那次发布的记录一条都不在」
/// 两类状态都摆得出来——少一道屏障（`crates/mutations.tsv`「步 3：零单元发布在记录与根之间少一道屏障」）就把两段并成一段，
/// 后一类立刻摆得出，记录核对器判红。种子写死不随机：这一条判的是枚举域罩不罩得住，不是抽样运气。
// crash-case-check:not-a-crash-case 写死的一段短历史，写数不超过 4 的段由崩溃注入库自己整段枚举真子集、更长的段各抽 8 个，不调枚举一族函数、不经块判定库；随机崩溃注入的默认批登记成 crash-case:crash-injection-default-batch
#[test]
fn every_crash_state_of_a_written_out_history_recovers_into_a_committed_version() {
    let content = |selector: u64| {
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector },
            fill_seed: selector,
        })
    };
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 3 },
                fill_seed: 1,
            }),
            content(5),
            HistoryOperation::PublishWithoutUnits,
            HistoryOperation::CloseAndMountWritable,
            content(7),
        ],
    };
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes: 4,
            sampled_in_longer_segments: 8,
        },
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    print_uncaptured(&format!(
        "── 崩溃注入：写死的一段历史，段内真子集全枚举 ──\n{}",
        injection.tally.render()
    ));
    if printed_not_run_because_the_engine_gave_placeholder_conclusions(
        PlaceholderTablesOfTheRun::of_the_tally(&injection.tally),
        "every_crash_state_of_a_written_out_history_recovers_into_a_committed_version",
    ) {
        return;
    }
    assert!(
        injection.new_findings.is_empty(),
        "写死的这段历史上崩溃状态判红：{:#?}",
        injection.new_findings
    );
    // 代码审阅第 19 条之后取号写之前多一道屏障：第二次可写挂载那一处，上一次发布末尾的轮换与取号那 4 写一段拆成 2、2，
    // 真子集 15 → 3 + 3，这段历史摆得出的崩溃状态 104 → 95（屏障只拆段、不添状态）。
    assert!(
        injection.tally.crash_points >= 95,
        "这段历史摆得出的崩溃状态不少于 95 个：{}",
        injection.tally.crash_points
    );
    assert_eq!(
        injection.tally.recoveries_failed, 0,
        "每个崩溃状态都恢复得出来"
    );
    assert!(
        injection.tally.crash_points_inside_an_unfinished_publish >= 4,
        "每次发布的单元写与记录写都截在它的根落盘之前：{}",
        injection.tally.crash_points_inside_an_unfinished_publish
    );
    assert!(
        injection
            .tally
            .crash_points_withholding_write_kind
            .get(StepKind::JournalRecord.name())
            .copied()
            .unwrap_or(0)
            >= 1,
        "一次都没把 journal 记录写扣下（少一道屏障那条变异就判不出来）：{:?}",
        injection.tally.crash_points_withholding_write_kind
    );
    assert_eq!(
        injection.tally.record_root_without_record, 0,
        "今天的代码上没有「根在而它那次发布的记录一条都不在」的状态"
    );
    assert_eq!(
        injection.tally.crash_states_ending_new_finding, 0,
        "一个崩溃状态都不许是新发现"
    );
}

/// 「已知红」清单那一条（增补 2 收口表第 43 行）在崩溃状态上的复现那一段历史（与随机历史那边
/// `the_history_that_raised_the_floor_into_a_rollback_gap_completes_under_the_forward_rollback` 逐项相同），两次回退换成挂着时的向前回退：
/// 没有回退留下的空档，抬 F 那几步之后的崩溃状态上一条清单外的失败都不许有；回退与抬 F 的写上都摆得到崩溃状态。
/// 清单那一形（F 落在被抛弃实例留下的空档里）今天只剩崩溃恢复抛弃的时间线造得出，随机历史与崩溃注入都没有这一种操作，
/// 它在崩溃状态上认不认得出来（代码三方 m2-supp3-item3-code-r1 判决 K6 的假阳那一半）这一段不再罩，交主 agent（实三报告）。
#[test]
fn crash_states_of_the_history_that_raised_the_floor_into_a_rollback_gap_are_clean_under_the_forward_rollback(
) {
    let empty = ContentChoice {
        length: ContentLength::Empty,
        fill_seed: 0,
    };
    let history = GeneratedHistory {
        seed: HistorySeed(80),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 0,
            }),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 3,
            }),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 8,
            }),
        ],
    };
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes: 2,
            sampled_in_longer_segments: 1,
        },
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    print_uncaptured(&format!(
        "── 崩溃注入：原先抬 F 落进回退空档那一段历史（向前回退） ──\n{}",
        injection.tally.render()
    ));
    if printed_not_run_because_the_engine_gave_placeholder_conclusions(
        PlaceholderTablesOfTheRun::of_the_tally(&injection.tally),
        "crash_states_of_the_history_that_raised_the_floor_into_a_rollback_gap_are_clean_under_the_forward_rollback",
    ) {
        return;
    }
    assert!(
        injection.new_findings.is_empty(),
        "一条清单外的失败都不许有：{:#?}",
        injection.new_findings
    );
    // 「回退与抬 F 的写上也摆得到崩溃状态」这一条钉在这里，不钉在随机种子那一段上：
    // 这段历史是写死的，回退两次、抬 F 一次都在里面，段内真子集又是全枚举的，所以这个结论不靠抽样运气。
    for kind in [
        HistoryOperationKind::RollBackWhileMounted,
        HistoryOperationKind::RaiseRollbackFloor,
    ] {
        assert!(
            injection
                .tally
                .crash_points_by_operation_kind
                .get(&kind)
                .copied()
                .unwrap_or(0)
                >= 1,
            "一个崩溃状态都没落在 {kind:?} 里：{:?}",
            injection.tally.crash_points_by_operation_kind
        );
    }
}

/// 各类操作的写上摆到几个崩溃状态（共八类操作各钉一条；
/// 含崩溃恢复抛弃根）。覆盖写与可写挂载由默认批钉、回退与抬 F 由上一条写死历史钉，剩下四类钉在这一段写死的历史上：
/// mkfs → 可写挂载 → 零单元发布（树表 0 条的一版上才调得成）→ 冷启动 → 可写挂载 → 第一个文件 → 崩溃恢复抛弃根（C554 乙拒）→ 可写挂载。
/// 写数不超过 2 的段整段枚举（2^n − 1 个真子集），更长的段各抽 1 个，所以每一类摆到几个是定数，不靠抽样运气：
/// 零单元发布的段是 2、1、2 ⇒ 3 + 1 + 3 = 7；第一个文件的段是 24、2、1、2 ⇒ 1 + 3 + 1 + 3 = 8；
/// 冷启动只读盘、一个写都不发，崩溃恢复抛弃根在取号之前被拒、也不发写 ⇒ 两类都是 0（崩溃状态按第一个被扣下的写落在哪一步归类，
/// 不发写的那一步归不到）。段序列是 harness 侧按同一段历史、同一个执行（`execute_history_with` 加
/// `writes_and_segments_with_stream_indexes`）现量的。
/// 比已有的两段写死历史多罩的：树表 0 条的一版上的零单元发布、冷启动之后与崩溃恢复被拒之后的可写挂载那几段的崩溃状态。
#[test]
fn crash_states_land_in_zero_unit_publishes_and_first_files_but_never_in_cold_starts_or_refused_crash_recoveries(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishWithoutUnits,
            HistoryOperation::ColdStartRecover,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 3 },
                fill_seed: 1,
            }),
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
            HistoryOperation::CloseAndMountWritable,
        ],
    };
    let crash_recovery_outcomes = outcomes_of_the_crash_recovery_steps(&history);
    assert_eq!(
        crash_recovery_outcomes,
        vec![StepOutcome::Refused {
            member: CRASH_RECOVERY_REFUSED_BY_THE_WITNESS.to_string()
        }],
        "崩溃恢复抛弃根那一步被 C554 乙拒：它归到 0 个崩溃状态说的是「被拒的那一步不发写」"
    );
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes: 2,
            sampled_in_longer_segments: 1,
        },
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    print_uncaptured(&format!(
        "── 崩溃注入：零单元发布、第一个文件、冷启动与崩溃恢复抛弃根各摆几个崩溃状态 ──\n{}",
        injection.tally.render()
    ));
    if printed_not_run_because_the_engine_gave_placeholder_conclusions(
        PlaceholderTablesOfTheRun::of_the_tally(&injection.tally),
        "crash_states_land_in_zero_unit_publishes_and_first_files_but_never_in_cold_starts_or_refused_crash_recoveries",
    ) {
        return;
    }
    let crash_points_in = |kind: HistoryOperationKind| {
        injection
            .tally
            .crash_points_by_operation_kind
            .get(&kind)
            .copied()
            .unwrap_or(0)
    };
    assert_eq!(
        crash_points_in(HistoryOperationKind::ColdStartRecover),
        0,
        "冷启动一个写都不发，崩溃状态归不到它：{:?}",
        injection.tally.crash_points_by_operation_kind
    );
    assert_eq!(
        crash_points_in(HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot),
        0,
        "崩溃恢复抛弃根在取号之前被拒、一个写都不发，崩溃状态归不到它：{:?}",
        injection.tally.crash_points_by_operation_kind
    );
    assert_eq!(
        crash_points_in(HistoryOperationKind::PublishWithoutUnits),
        7,
        "零单元发布的段 2、1、2：3 + 1 + 3：{:?}",
        injection.tally.crash_points_by_operation_kind
    );
    assert_eq!(
        crash_points_in(HistoryOperationKind::PublishFirstFile),
        8,
        "第一个文件的段 24、2、1、2：1 + 3 + 1 + 3：{:?}",
        injection.tally.crash_points_by_operation_kind
    );
    // 计数先钉、判定后钉：计数只看段序列与归步，少一道屏障那类变异先红在上面的计数上（`crates/mutations.tsv`）。
    assert!(
        injection.new_findings.is_empty(),
        "一条清单外的失败都不许有：{:#?}",
        injection.new_findings
    );
}
