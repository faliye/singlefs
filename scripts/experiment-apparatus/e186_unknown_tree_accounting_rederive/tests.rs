use super::*;

fn point_at(geometry: Geometry, unknown_writable_head_count: u64, earlier_step_count: u64, initial_publish_changed_the_watermark: bool, tree_dimension_statistic_count: u64) -> SamplePoint {
    SamplePoint {
        geometry,
        retired_kept_generation_count: 4,
        invisible_window_generations: 3,
        unknown_writable_head_count,
        trees_with_registered_bit: 0,
        published_generations: 10,
        earlier_step_count,
        initial_publish_changed_the_watermark,
        tree_dimension_statistic_count,
    }
}

/// 第一次跑的单测钉的是不带参数时的判据（`DEFAULT_JUDGEMENT`）：下面三个助手不写判据时都取它，
/// 第二次跑的单测用 `_under` 结尾的写法点名判据。
fn cell_at(point: SamplePoint, flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy) -> CellOutcome {
    cell_at_under(point, flags_policy, accounting_policy, DEFAULT_JUDGEMENT)
}

fn cell_at_under(point: SamplePoint, flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy, judgement: Judgement) -> CellOutcome {
    evaluate_cell(&point, &watermark_change_history_of(&point), flags_policy, accounting_policy, judgement)
}

fn evaluate_point(point: SamplePoint) -> EvaluatedPoint {
    super::evaluate_point(point, DEFAULT_JUDGEMENT)
}

fn all_evaluated_points() -> &'static [EvaluatedPoint] {
    all_evaluated_points_under(DEFAULT_JUDGEMENT)
}

/// 全部 11520 点按给定判据算一遍，同一个测试进程里每种判据只算一次（并行的几个测试共用）。
fn all_evaluated_points_under(judgement: Judgement) -> &'static [EvaluatedPoint] {
    static VACUOUS: std::sync::OnceLock<Vec<EvaluatedPoint>> = std::sync::OnceLock::new();
    static DANGLING: std::sync::OnceLock<Vec<EvaluatedPoint>> = std::sync::OnceLock::new();
    let slot = match judgement {
        Judgement::VacuousCarriedRowsClause => &VACUOUS,
        Judgement::DanglingCarriedRowsClause => &DANGLING,
    };
    slot.get_or_init(|| all_sample_points().into_iter().map(|point| super::evaluate_point(point, judgement)).collect())
}

#[test]
fn sample_points_number_11520_and_every_point_ranks_all_fifteen_cells() {
    let points = all_sample_points();
    assert_eq!(u64::try_from(points.len()).expect("计数装得进 u64"), EXPECTED_POINT_COUNT);
    assert_eq!(EXPECTED_POINT_COUNT, 5 * 4 * 3 * 2 * 4 * 6 * 2 * 2);
    // 只读挂载那几格（`ReadOnly`、未登记时的 `PerBit`）可写代数为 0，照样进比较。
    let evaluated = evaluate_point(point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 1));
    assert_eq!(u64::try_from(evaluated.cells.len()).expect("计数装得进 u64"), CELLS_PER_POINT);
    assert_eq!(evaluated.outcome_under_the_arms(UnknownFlagsPolicy::ReadOnly, AccountingPolicy::Carry).writable_generations, 0);
}

#[test]
fn ignore_violates_read_only_once_per_generation_and_revives_the_half_deleted_tree() {
    for (published_generations, expected_violations) in [(1u64, 1u64), (4, 4), (5, 5), (10, 10)] {
        let mut point = point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 1);
        point.published_generations = published_generations;
        for accounting_policy in ACCOUNTING_ARMS {
            let cell = cell_at(point, UnknownFlagsPolicy::Ignore, accounting_policy);
            assert_eq!(cell.read_only_violations, expected_violations, "G = {published_generations}");
            assert_eq!(cell.revived_dead_trees, 1);
        }
    }
}

#[test]
fn skip_wrongly_frees_sixty_four_blocks_per_invisible_generation_and_nothing_without_a_window() {
    for (invisible_window_generations, expected_blocks) in [(0u64, 0u64), (1, 64), (3, 192), (8, 512)] {
        let mut point = point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 1);
        point.invisible_window_generations = invisible_window_generations;
        assert_eq!(cell_at(point, UnknownFlagsPolicy::Skip, AccountingPolicy::Carry).blocks_freed_while_referenced, expected_blocks);
        assert_eq!(cell_at(point, UnknownFlagsPolicy::RejectEntry, AccountingPolicy::Carry).blocks_freed_while_referenced, 0);
    }
}

#[test]
fn reject_entry_counted_semantics_adds_exactly_one_clause_unit_and_skip_gets_none() {
    let evaluated = evaluate_point(point_at(Geometry::ImplementationTwoDevices, 2, 1, false, 1));
    for accounting_policy in ACCOUNTING_ARMS {
        let counted = evaluated.vector(UnknownFlagsPolicy::RejectEntry, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
        let not_counted = evaluated.vector(UnknownFlagsPolicy::RejectEntry, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED);
        assert_eq!(counted[5], not_counted[5] + 1);
        assert_eq!(
            evaluated.vector(UnknownFlagsPolicy::Skip, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED),
            evaluated.vector(UnknownFlagsPolicy::Skip, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED)
        );
    }
    assert_eq!(evaluated.vector(UnknownFlagsPolicy::RejectEntry, AccountingPolicy::Carry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED)[5], 1);
}

#[test]
fn read_only_never_publishes_and_loses_nothing() {
    for accounting_policy in ACCOUNTING_ARMS {
        let cell = cell_at(point_at(Geometry::UnknownTreeFullArms, 2, 0, false, 2), UnknownFlagsPolicy::ReadOnly, accounting_policy);
        assert_eq!(cell.writable_generations, 0);
        assert_eq!(cell.rows_lost, 0);
        assert_eq!(cell.rows_written_all_rows, 0);
        assert_eq!(cell.clause_units_excluding_the_judged_items, 0);
        assert_eq!(cell.clause_units_counting_the_judged_items, 1, "U13 按臂判：只读挂载不是「该条目 EIO」");
    }
}

#[test]
fn per_bit_equals_read_only_when_unregistered_and_skip_when_all_registered() {
    for (trees_with_registered_bit, reduces_to) in [(0u64, UnknownFlagsPolicy::ReadOnly), (UNKNOWN_TREE_COUNT, UnknownFlagsPolicy::Skip)] {
        let mut point = point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 2);
        point.trees_with_registered_bit = trees_with_registered_bit;
        let evaluated = evaluate_point(point);
        for accounting_policy in ACCOUNTING_ARMS {
            for reading in [JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED] {
                assert_eq!(evaluated.vector(UnknownFlagsPolicy::PerBit, accounting_policy, reading), evaluated.vector(reduces_to, accounting_policy, reading));
            }
        }
    }
}

#[test]
fn no_carry_loses_the_second_statistic_tuple_at_the_first_publish_whatever_the_kept_generation_count_says() {
    for retired_kept_generation_count in [1u64, 20] {
        for published_generations in [1u64, 10] {
            let mut point = point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 2);
            point.retired_kept_generation_count = retired_kept_generation_count;
            point.published_generations = published_generations;
            let cell = cell_at(point, UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry);
            assert_eq!(cell.rows_lost, 2, "k_keep = {retired_kept_generation_count}, G = {published_generations}：两个不认识的头各丢第二个统计量那一元组");
            assert_eq!(cell.silent_harm_count, 1, "第二个统计量的元组丢失没有不变量盖");
        }
    }
}

#[test]
fn no_carry_keeps_the_watermark_tuple_through_its_step_rows() {
    let cell = cell_at(point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 1), UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry);
    assert_eq!(cell.rows_lost, 0);
    assert_eq!(cell.clause_units_excluding_the_judged_items, 0);
    assert_eq!(cell.rows_dropped_over_all_generations, 2, "行数口径：R 那一行（不是台阶）在 R_1 里没了，两个头各一行");
}

#[test]
fn keep_with_root_breaks_six_clauses_when_the_initial_row_is_not_a_step_and_none_when_it_is() {
    let not_a_step = cell_at(point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 1), UnknownFlagsPolicy::Skip, AccountingPolicy::KeepWithRoot);
    assert_eq!(not_a_step.clause_units_excluding_the_judged_items, 6, "U1 U2 U3 U4 U6 U7");
    let is_a_step = cell_at(point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 1), UnknownFlagsPolicy::Skip, AccountingPolicy::KeepWithRoot);
    assert_eq!(is_a_step.clause_units_excluding_the_judged_items, 0);
    let second_statistic = cell_at(point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 2), UnknownFlagsPolicy::Skip, AccountingPolicy::KeepWithRoot);
    assert_eq!(second_statistic.clause_units_excluding_the_judged_items, 4, "U1 U3 U6 U7");
    assert_eq!(not_a_step.trajectory_unknown_rows_at_the_initial_txg, Some(Trajectory { peak: 2, positive_versions: 10, final_value: 2 }));
}

#[test]
fn carry_breaks_no_clause_and_keeps_every_step_unchanged() {
    for (earlier_step_count, initial_publish_changed_the_watermark) in STEP_HISTORY_VALUES {
        for statistic_count in [1u64, 2] {
            let cell = cell_at(point_at(Geometry::ImplementationTwoDevices, 2, earlier_step_count, initial_publish_changed_the_watermark, statistic_count), UnknownFlagsPolicy::Skip, AccountingPolicy::Carry);
            assert_eq!(cell.clause_units_excluding_the_judged_items, 0, "s = {earlier_step_count}, lpc = {initial_publish_changed_the_watermark}, r = {statistic_count}");
            assert_eq!(cell.rows_lost, 0);
        }
    }
}

#[test]
fn no_carry_without_any_step_loses_the_unknown_heads_watermark_rows_under_the_watermark_row_invariant() {
    let cell = cell_at(point_at(Geometry::UnknownTreeFullArms, 2, 0, false, 1), UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry);
    assert_eq!(cell.clause_units_excluding_the_judged_items, 1, "U8");
    assert_eq!(cell.rows_lost, 2);
    assert_eq!(cell.silent_harm_count, 0, "水位元组丢失有 I-9.11 盖");
}

#[test]
fn rows_written_match_the_registered_closed_forms() {
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    let expectations: [(Geometry, u64, u64, bool, u64, [u64; 3], [u64; 3]); 3] = [
        (Geometry::UnknownTreeFullArms, 2, 1, true, 1, [23, 21, 21], [19, 17, 17]),
        (Geometry::ImplementationTwoDevices, 4, 3, false, 2, [37, 29, 37], [24, 16, 16]),
        (Geometry::ImplementationTwoDevices, 2, 0, false, 1, [18, 16, 18], [17, 15, 15]),
    ];
    for (geometry, head_count, earlier_step_count, initial_publish_changed_the_watermark, statistic_count, all_rows, this_generation_rows) in expectations {
        let point = point_at(geometry, head_count, earlier_step_count, initial_publish_changed_the_watermark, statistic_count);
        for ((accounting_policy, expected_all_rows), expected_this_generation_rows) in ACCOUNTING_ARMS.into_iter().zip(all_rows).zip(this_generation_rows) {
            let cell = cell_at(point, UnknownFlagsPolicy::Skip, accounting_policy);
            assert_eq!(cell.rows_written_all_rows, expected_all_rows, "{point:?} {accounting_policy:?} W2");
            assert_eq!(cell.rows_written_this_generation, expected_this_generation_rows, "{point:?} {accounting_policy:?} W1");
        }
    }
    // 甲类：实现两盘几何、没有不认识的头时，W1 = 15（D5 已定项 8 的 15 行），W2 再加认识的头那一级台阶。
    let implementation = cell_at(point_at(Geometry::ImplementationTwoDevices, 0, 1, true, 1), UnknownFlagsPolicy::Skip, AccountingPolicy::Carry);
    assert_eq!(implementation.rows_written_this_generation, CRATES_POOL_WIDE_ROWS + CRATES_PER_DEVICE_ROWS * CRATES_DEVICE_COUNT);
    assert_eq!(implementation.rows_written_this_generation, 15);
    assert_eq!(implementation.rows_written_all_rows, 15 + CRATES_KNOWN_HEAD_STEP_ROWS);
}

#[test]
fn this_generation_rows_ranking_puts_carry_ahead_of_keep_with_root_on_clause_units_despite_more_rows() {
    let evaluated = evaluate_point(point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 1));
    let this_generation_rows_reading = INCIDENTAL_READINGS[2];
    assert_eq!(this_generation_rows_reading.rows_written, RowsWrittenReading::ThisGenerationRowsOnly);
    let carry = evaluated.vector(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry, this_generation_rows_reading);
    let keep_with_root = evaluated.vector(UnknownFlagsPolicy::Skip, AccountingPolicy::KeepWithRoot, this_generation_rows_reading);
    assert_eq!(carry, [192, 0, 0, 0, 0, 0, 19]);
    assert_eq!(keep_with_root, [192, 0, 0, 0, 0, 6, 17]);
    assert!(carry < keep_with_root);
}

#[test]
fn all_rows_ranking_puts_carry_ahead_of_keep_with_root_when_the_second_statistic_is_not_a_step() {
    let evaluated = evaluate_point(point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 2));
    let carry = evaluated.vector(UnknownFlagsPolicy::RejectEntry, AccountingPolicy::Carry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED);
    let keep_with_root = evaluated.vector(UnknownFlagsPolicy::RejectEntry, AccountingPolicy::KeepWithRoot, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED);
    // 自己那些行 11 + 6 × 2 = 23；Carry 每头 s + lpc + r = 4 行，随根留着每头 s + r = 3 行。
    assert_eq!(carry, [0, 0, 0, 0, 0, 0, 23 + 2 * 4]);
    assert_eq!(keep_with_root, [0, 0, 0, 0, 0, 4, 23 + 2 * 3]);
    assert!(carry < keep_with_root);
}

#[test]
fn crates_step_reading_matches_the_change_history_on_a_single_head_history() {
    let head = TreeOwner::KnownHead(0);
    let watermark_row = |generation_txg: u64, value: u64| AccountingRow { statistic: Statistic::InodeNumberWatermark, tree: head, generation_txg, value };
    let history = WatermarkChangeHistory { changed_at: [(head, 990), (head, 995), (head, 1001)].into_iter().collect() };
    let changed_this_publish = AccountingVersion { txg: 1001, rows: vec![watermark_row(990, 5), watermark_row(995, 6), watermark_row(1001, 7)], rows_written_for_unknown_trees: Vec::new() };
    assert_eq!(steps_by_the_crates_reading(&changed_this_publish), steps_by_the_watermark_change_history(&changed_this_publish, &history));
    assert_eq!(steps_by_the_crates_reading(&changed_this_publish).len(), 3);
    let unchanged_this_publish = AccountingVersion { txg: 1002, rows: vec![watermark_row(990, 5), watermark_row(995, 6), watermark_row(1001, 7), watermark_row(1002, 7)], rows_written_for_unknown_trees: Vec::new() };
    assert_eq!(steps_by_the_crates_reading(&unchanged_this_publish), steps_by_the_watermark_change_history(&unchanged_this_publish, &history));
    assert_eq!(steps_by_the_crates_reading(&unchanged_this_publish).len(), 3);
}

#[test]
fn checker_syntax_counts_the_second_statistic_but_skips_the_kept_watermark_row() {
    let cell = cell_at(point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 2), UnknownFlagsPolicy::Skip, AccountingPolicy::KeepWithRoot);
    assert_eq!(cell.trajectory_row_generation_invariant_syntactic_violations, Some(Trajectory { peak: 2, positive_versions: 10, final_value: 2 }));
}

#[test]
fn positive_and_negative_controls_hold_on_every_point() {
    let evaluated_points = all_evaluated_points();
    for control in POSITIVE_CONTROLS {
        let (checked, failures) = evaluated_points.iter().map(|evaluated| positive_control_failures(evaluated, control)).fold((0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1));
        assert!(checked > 0, "{control:?} 一格都没查");
        assert_eq!(failures, 0, "{control:?}");
    }
    for negative_control in [negative_control_no_window_failures, negative_control_no_unknown_heads_failures] {
        let (checked, failures) = evaluated_points.iter().map(negative_control).fold((0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1));
        assert!(checked > 0);
        assert_eq!(failures, 0);
    }
}

#[test]
fn device_predicates_and_the_hand_ledger_agree_on_every_cell_under_both_readings() {
    for judgement in [Judgement::VacuousCarriedRowsClause, Judgement::DanglingCarriedRowsClause] {
        let mut compared = 0u64;
        for evaluated in all_evaluated_points_under(judgement) {
            for cell in &evaluated.cells {
                assert_eq!(cell.clause_units_excluding_the_judged_items, clause_units_from_the_hand_ledger(&evaluated.point, cell.flags_policy, cell.accounting_policy, ClauseReading::ExcludesTheJudgedItems, judgement), "{judgement:?} {:?} {cell:?}", evaluated.point);
                assert_eq!(cell.clause_units_counting_the_judged_items, clause_units_from_the_hand_ledger(&evaluated.point, cell.flags_policy, cell.accounting_policy, ClauseReading::CountsTheJudgedItems, judgement), "{judgement:?} {:?} {cell:?}", evaluated.point);
                compared += 1;
            }
        }
        assert_eq!(compared, EXPECTED_POINT_COUNT * CELLS_PER_POINT);
    }
}

#[test]
fn registered_anchors_hold_on_every_selected_point() {
    for (name, checked, mismatches) in anchor_failures(all_evaluated_points()) {
        assert!(checked > 0, "{name} 一格都没查");
        assert_eq!(mismatches, 0, "{name}");
    }
}

#[test]
fn kept_generation_count_is_not_read_by_the_model() {
    for point in all_sample_points().into_iter().filter(|point| point.retired_kept_generation_count == 20 && point.invisible_window_generations == 3) {
        let with_largest = evaluate_point(point);
        let with_smallest = evaluate_point(point_with(&point, PointChange::RetiredKeptGenerationCount(1)));
        for cell in &with_largest.cells {
            for reading in [JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED] {
                assert_eq!(vector_under(cell, &point, reading), with_smallest.vector(cell.flags_policy, cell.accounting_policy, reading), "{point:?}");
            }
        }
    }
}

#[test]
fn self_proofs_turn_red_when_the_discriminating_position_is_broken() {
    let mut no_window = point_at(Geometry::ImplementationTwoDevices, 2, 1, false, 1);
    no_window.invisible_window_generations = 0;
    let evaluated = evaluate_point(no_window);
    for accounting_policy in ACCOUNTING_ARMS {
        assert!(flags_question_self_proof_turns_red(&evaluated, accounting_policy));
    }
    assert!(accounting_question_position_six_self_proof_turns_red(&evaluated, UnknownFlagsPolicy::Skip));
    let initial_is_a_step = evaluate_point(point_at(Geometry::ImplementationTwoDevices, 2, 1, true, 1));
    assert!(accounting_question_position_seven_self_proof_turns_red(&initial_is_a_step, UnknownFlagsPolicy::RejectEntry));
    // 记账臂没有作用点的格（只读挂载）七位全平，弄坏也不变：自证在那里不成立，只在记账臂起作用的格上查。
    assert!(!accounting_question_position_six_self_proof_turns_red(&evaluated, UnknownFlagsPolicy::ReadOnly));
}

#[test]
fn first_separating_position_and_winners_follow_the_lexicographic_order() {
    let vectors = [[0, 0, 0, 0, 0, 6, 17], [0, 0, 0, 0, 0, 0, 19], [0, 0, 0, 0, 0, 0, 17]];
    assert_eq!(first_separating_position(&vectors), Some(6));
    assert_eq!(winner_indices(&vectors), vec![2]);
    assert_eq!(first_separating_position(&[[1, 2, 3, 4, 5, 6, 7], [1, 2, 3, 4, 5, 6, 7]]), None);
    assert_eq!(winner_indices(&[[1, 2, 3, 4, 5, 6, 7], [1, 2, 3, 4, 5, 6, 7]]), vec![0, 1]);
}

// ---- 第二次跑（`dangling-u5`）的单测：登记第二次跑第七、八、九、十一节 ----

fn carried_rows_clause_false_units(cell: &CellOutcome) -> u64 {
    u64::from(carried_rows_clause_false_trajectory_of(cell).is_some_and(|trajectory| trajectory.peak > 0))
}

fn arguments_of(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

#[test]
fn the_default_judgement_is_the_first_runs_and_the_parameter_picks_either() {
    assert_eq!(DEFAULT_JUDGEMENT, Judgement::VacuousCarriedRowsClause);
    assert_eq!(Judgement::from_arguments(&[]), Ok(Judgement::VacuousCarriedRowsClause));
    assert_eq!(Judgement::from_arguments(&arguments_of(&["--judgement", "vacuous-u5"])), Ok(Judgement::VacuousCarriedRowsClause));
    assert_eq!(Judgement::from_arguments(&arguments_of(&["--judgement", "dangling-u5"])), Ok(Judgement::DanglingCarriedRowsClause));
    assert!(Judgement::from_arguments(&arguments_of(&["--judgement", "other"])).is_err());
    assert!(Judgement::from_arguments(&arguments_of(&["--judgement"])).is_err());
    assert!(Judgement::from_arguments(&arguments_of(&["dangling-u5"])).is_err());
    assert_eq!(Judgement::VacuousCarriedRowsClause.label(), "vacuous-u5");
    assert_eq!(Judgement::DanglingCarriedRowsClause.label(), "dangling-u5");
}

#[test]
fn dangling_carried_rows_clause_on_the_registered_point_counts_zero_one_one_across_the_accounting_arms() {
    let point = point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 1);
    let never_false = Trajectory { peak: 0, positive_versions: 0, final_value: 0 };
    let false_in_every_version = Trajectory { peak: 1, positive_versions: 10, final_value: 1 };
    // (记账臂, 新 U5 记几, 新 U5 为假的轨迹, 新判据下 B1 第 6 位)：登记第七节「单测的期望」第一张表。
    let expectations = [
        (AccountingPolicy::Carry, 0u64, never_false, 0u64),
        (AccountingPolicy::NoCarry, 1, false_in_every_version, 1),
        (AccountingPolicy::KeepWithRoot, 1, false_in_every_version, 7),
    ];
    let dangling = super::evaluate_point(point, Judgement::DanglingCarriedRowsClause);
    for (accounting_policy, expected_carried_rows_clause_units, expected_trajectory, expected_position_six) in expectations {
        let cell = cell_at_under(point, UnknownFlagsPolicy::Skip, accounting_policy, Judgement::DanglingCarriedRowsClause);
        assert_eq!(carried_rows_clause_false_units(&cell), expected_carried_rows_clause_units, "{accounting_policy:?}");
        assert_eq!(carried_rows_clause_false_trajectory_of(&cell), Some(expected_trajectory), "{accounting_policy:?}");
        assert_eq!(dangling.vector(UnknownFlagsPolicy::Skip, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED)[5], expected_position_six, "{accounting_policy:?}");
        // 第一次跑的判据下三条臂的 U5 都是 0；新判据比它多的正是 U5（写加法）。
        let vacuous_cell = cell_at_under(point, UnknownFlagsPolicy::Skip, accounting_policy, Judgement::VacuousCarriedRowsClause);
        assert_eq!(carried_rows_clause_false_units(&vacuous_cell), 0, "{accounting_policy:?}");
        assert_eq!(cell.clause_units_excluding_the_judged_items, vacuous_cell.clause_units_excluding_the_judged_items + expected_carried_rows_clause_units, "{accounting_policy:?}");
    }
}

#[test]
fn dangling_carried_rows_clause_counts_nothing_where_the_accounting_arm_does_not_act_or_there_is_no_unknown_head() {
    let point = point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 1);
    for flags_policy in [UnknownFlagsPolicy::Ignore, UnknownFlagsPolicy::ReadOnly] {
        for accounting_policy in ACCOUNTING_ARMS {
            let cell = cell_at_under(point, flags_policy, accounting_policy, Judgement::DanglingCarriedRowsClause);
            assert_eq!(carried_rows_clause_false_units(&cell), 0, "{flags_policy:?} {accounting_policy:?}");
        }
    }
    let without_unknown_head = point_at(Geometry::UnknownTreeFullArms, 0, 1, false, 1);
    for accounting_policy in ACCOUNTING_ARMS {
        let cell = cell_at_under(without_unknown_head, UnknownFlagsPolicy::Skip, accounting_policy, Judgement::DanglingCarriedRowsClause);
        assert_eq!(carried_rows_clause_false_units(&cell), 0, "{accounting_policy:?}");
        assert_eq!(carried_rows_clause_false_trajectory_of(&cell), Some(Trajectory { peak: 0, positive_versions: 0, final_value: 0 }), "{accounting_policy:?}");
    }
}

#[test]
fn dangling_carried_rows_clause_ledger_columns_follow_the_registered_second_run_table() {
    // (几何点: h_new, s, lpc, r_tree_dim)，三条记账臂的 B1 第 6 位（Skip 下）：登记第七节「另三格钉账本别的列」。
    let columns = [((2u64, 1u64, true, 1u64), [0u64, 1, 1]), ((2, 0, false, 1), [0, 2, 7]), ((2, 1, true, 2), [0, 1, 5])];
    for ((head_count, earlier_step_count, lpc, statistic_count), expected) in columns {
        let evaluated = super::evaluate_point(point_at(Geometry::UnknownTreeFullArms, head_count, earlier_step_count, lpc, statistic_count), Judgement::DanglingCarriedRowsClause);
        for (accounting_policy, expected_position_six) in ACCOUNTING_ARMS.into_iter().zip(expected) {
            assert_eq!(evaluated.vector(UnknownFlagsPolicy::Skip, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED)[5], expected_position_six, "{head_count} {earlier_step_count} {lpc} {statistic_count} {accounting_policy:?}");
        }
    }
}

#[test]
fn dangling_carried_rows_clause_self_proof_turns_red_where_the_vacuous_reading_cannot_tell_carry_from_no_carry() {
    let point = point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 1);
    assert!(is_a_point_of_the_dangling_carried_rows_self_proof(&point));
    let dangling = super::evaluate_point(point, Judgement::DanglingCarriedRowsClause);
    for flags_policy in [UnknownFlagsPolicy::Skip, UnknownFlagsPolicy::RejectEntry] {
        assert!(accounting_question_dangling_carried_rows_self_proof_turns_red(&dangling, flags_policy), "{flags_policy:?}");
    }
    // 记账臂没有作用点的格（只读挂载）七位全平，弄坏也不变：自证只在记账臂起作用的格上查。
    assert!(!accounting_question_dangling_carried_rows_self_proof_turns_red(&dangling, UnknownFlagsPolicy::ReadOnly));
    // 第一次跑的判据下 Carry 与 NoCarry 在这一点第 6 位相等（第一次跑判据错在这里），这条自证在那里本来就红不起来。
    let vacuous = super::evaluate_point(point, Judgement::VacuousCarriedRowsClause);
    assert_eq!(
        vacuous.vector(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED)[5],
        vacuous.vector(UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED)[5]
    );
    assert!(!accounting_question_dangling_carried_rows_self_proof_turns_red(&vacuous, UnknownFlagsPolicy::Skip));
    assert!(!is_a_point_of_the_dangling_carried_rows_self_proof(&point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 1)));
    assert!(!is_a_point_of_the_dangling_carried_rows_self_proof(&point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 2)));
}

#[test]
fn positive_and_negative_controls_hold_on_every_point_under_the_dangling_carried_rows_clause() {
    let evaluated_points = all_evaluated_points_under(Judgement::DanglingCarriedRowsClause);
    let controls = positive_controls_of(Judgement::DanglingCarriedRowsClause);
    assert_eq!(controls.len(), POSITIVE_CONTROLS.len() + 3);
    assert_eq!(positive_controls_of(Judgement::VacuousCarriedRowsClause), POSITIVE_CONTROLS.to_vec());
    for control in controls {
        let (checked, failures) = evaluated_points.iter().map(|evaluated| positive_control_failures(evaluated, control)).fold((0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1));
        assert!(checked > 0, "{control:?} 一格都没查");
        assert_eq!(failures, 0, "{control:?}");
    }
    for negative_control in [negative_control_no_window_failures, negative_control_no_unknown_heads_failures] {
        let (checked, failures) = evaluated_points.iter().map(negative_control).fold((0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1));
        assert!(checked > 0);
        assert_eq!(failures, 0);
    }
}

#[test]
fn the_new_carried_rows_clause_controls_fail_on_the_first_runs_cells() {
    // 判别力：新 U5 的阳性对照在第一次跑的判据算出的格上必须红，否则这三条对照分不出两种判据。
    let vacuous_points = all_evaluated_points_under(Judgement::VacuousCarriedRowsClause);
    for control in DANGLING_CARRIED_ROWS_POSITIVE_CONTROLS {
        let failures: u64 = vacuous_points.iter().map(|evaluated| positive_control_failures(evaluated, control).1).sum();
        let expected_to_fail = control != PositiveControl::DanglingCarriedRowsCarryHolds;
        assert_eq!(failures > 0, expected_to_fail, "{control:?}");
    }
}

fn pt(has_unknown_writable_heads: bool, winners: &[AccountingPolicy]) -> (bool, Vec<AccountingPolicy>) {
    (has_unknown_writable_heads, winners.to_vec())
}

#[test]
fn every_winner_form_gets_its_registered_consequence() {
    use AccountingPolicy::{Carry, KeepWithRoot, NoCarry};
    let form_and_consequence = |points: &[(bool, Vec<AccountingPolicy>)]| {
        let tally = carry_acceptance_of(points);
        (tally.form, tally.consequence)
    };
    assert_eq!(form_and_consequence(&[pt(true, &[Carry]), pt(true, &[Carry])]), (AcceptanceForm::UniqueCarry, Consequence::CarryStands));
    assert_eq!(form_and_consequence(&[pt(true, &[NoCarry]), pt(true, &[NoCarry])]), (AcceptanceForm::UniqueOtherArm, Consequence::FlipsTo(NoCarry)));
    assert_eq!(form_and_consequence(&[pt(true, &[KeepWithRoot])]), (AcceptanceForm::UniqueOtherArm, Consequence::FlipsTo(KeepWithRoot)));
    assert_eq!(form_and_consequence(&[pt(true, &[Carry, NoCarry]), pt(true, &[Carry, NoCarry])]), (AcceptanceForm::TieWithCarry, Consequence::CarryStandsTieKnownToUser));
    assert_eq!(form_and_consequence(&[pt(true, &[NoCarry, KeepWithRoot]), pt(true, &[NoCarry, KeepWithRoot])]), (AcceptanceForm::TieWithoutCarry, Consequence::FlipChoiceToUser));
    assert_eq!(form_and_consequence(&[pt(true, &[Carry]), pt(true, &[Carry, NoCarry])]), (AcceptanceForm::DependsOnPointCarryEverywhere, Consequence::CarryStands));
    assert_eq!(form_and_consequence(&[pt(true, &[NoCarry]), pt(true, &[NoCarry, KeepWithRoot])]), (AcceptanceForm::DependsOnPointCarryNowhere, Consequence::FlipChoiceToUser));
    assert_eq!(form_and_consequence(&[pt(true, &[Carry]), pt(true, &[NoCarry])]), (AcceptanceForm::DependsOnPointMixed, Consequence::LeftToUser));
    assert_eq!(form_and_consequence(&[]), (AcceptanceForm::NoJudgedPoints, Consequence::NoJudgedPoints));
    // 全是 h_new = 0 的点：没有被判点，同样读不到。
    assert_eq!(form_and_consequence(&[pt(false, &[Carry, NoCarry, KeepWithRoot])]), (AcceptanceForm::NoJudgedPoints, Consequence::NoJudgedPoints));
}

#[test]
fn points_with_no_unknown_writable_head_are_reported_but_not_judged() {
    use AccountingPolicy::{Carry, KeepWithRoot, NoCarry};
    let tally = carry_acceptance_of(&[pt(true, &[Carry]), pt(false, &[Carry, NoCarry, KeepWithRoot]), pt(true, &[Carry]), pt(false, &[Carry, NoCarry, KeepWithRoot])]);
    assert_eq!((tally.judged_points, tally.excluded_points_without_unknown_writable_heads), (2, 2));
    assert_eq!((tally.form, tally.consequence), (AcceptanceForm::UniqueCarry, Consequence::CarryStands));
    assert_eq!(tally.winner_sets.len(), 1);
    assert_eq!(render_acceptance_line(Judgement::DanglingCarriedRowsClause, "legal_pooled", true, &tally),
        "name=verdict check=q1_carry_acceptance_form judgement=dangling-u5 reading=b1_w2_q2_counted group=legal_pooled judged=yes judged_points=2 excluded_points_h_new_0=2 form=unique_carry consequence=carry_stands winner_sets=Carry:2");
}

#[test]
fn judged_and_excluded_point_counts_match_the_registered_anchors() {
    let tallies = acceptance_tallies(all_evaluated_points_under(Judgement::DanglingCarriedRowsClause));
    for (group, judged_points, excluded_points) in EXPECTED_ACCEPTANCE_POINT_COUNTS {
        assert_eq!((tallies[group].judged_points, tallies[group].excluded_points_without_unknown_writable_heads), (judged_points, excluded_points), "{group}");
    }
    assert_eq!(EXPECTED_ACCEPTANCE_POINT_COUNTS.map(|(_, judged_points, _)| judged_points), [3200, 3200, 6400]);
    // 命令独立数出来的 E113 口径点数 1920 与全部点数（登记第七节乙类）。
    let e113_style = all_sample_points().iter().filter(|point| !point.is_legal_under_todays_step_rows()).count();
    assert_eq!(e113_style, 1920);
    assert_eq!(tallies["e113_style_r_tree_dim_1"].judged_points + tallies["e113_style_r_tree_dim_1"].excluded_points_without_unknown_writable_heads, 960);
    assert_eq!(tallies["e113_style_r_tree_dim_2"].judged_points + tallies["e113_style_r_tree_dim_2"].excluded_points_without_unknown_writable_heads, 960);
}

#[test]
fn the_two_judgements_differ_only_in_the_carried_rows_clause_on_every_point_and_cell() {
    let vacuous = all_evaluated_points_under(Judgement::VacuousCarriedRowsClause);
    let dangling = all_evaluated_points_under(Judgement::DanglingCarriedRowsClause);
    assert_eq!(vacuous.len(), dangling.len());
    let mut cells_compared = 0u64;
    let mut cells_where_the_carried_rows_clause_differs = 0u64;
    for (before, after) in vacuous.iter().zip(dangling) {
        assert_eq!(before.point, after.point);
        for (vacuous_cell, dangling_cell) in before.cells.iter().zip(&after.cells) {
            cells_compared += 1;
            cells_where_the_carried_rows_clause_differs += u64::from(carried_rows_clause_false_units(vacuous_cell) != carried_rows_clause_false_units(dangling_cell));
            assert!(cells_differ_only_in_the_carried_rows_clause(vacuous_cell, dangling_cell, &after.point), "{:?} {:?} {:?}", after.point, dangling_cell.flags_policy, dangling_cell.accounting_policy);
        }
        assert!(flags_question_judgements_agree(before, after), "{:?}", after.point);
    }
    assert_eq!(cells_compared, EXPECTED_POINT_COUNT * CELLS_PER_POINT);
    assert!(cells_where_the_carried_rows_clause_differs > 0, "两种判据真的不同，这条对拍才有分辨力");
}

#[test]
fn the_judgement_comparison_turns_red_when_something_other_than_the_carried_rows_clause_moves() {
    let point = point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 1);
    let vacuous = cell_at_under(point, UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry, Judgement::VacuousCarriedRowsClause);
    let dangling = cell_at_under(point, UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry, Judgement::DanglingCarriedRowsClause);
    assert!(cells_differ_only_in_the_carried_rows_clause(&vacuous, &dangling, &point));
    let mut wrongly_freed = dangling.clone();
    wrongly_freed.blocks_freed_while_referenced += 1;
    assert!(!cells_differ_only_in_the_carried_rows_clause(&vacuous, &wrongly_freed, &point));
    let mut extra_clause_unit = dangling.clone();
    extra_clause_unit.clause_units_excluding_the_judged_items += 1;
    assert!(!cells_differ_only_in_the_carried_rows_clause(&vacuous, &extra_clause_unit, &point));
    let mut other_clause_moved = dangling.clone();
    for (clause, trajectory) in &mut other_clause_moved.clause_false_trajectories {
        if *clause == ClauseUnit::DecisionSixteenItemNineRewriteEachPublish {
            *trajectory = Some(Trajectory { peak: 1, positive_versions: 1, final_value: 1 });
        }
    }
    assert!(!cells_differ_only_in_the_carried_rows_clause(&vacuous, &other_clause_moved, &point));
}

#[test]
fn carried_rows_object_count_equals_unknown_writable_heads_positive_on_every_point() {
    let mut checked = 0u64;
    for point in all_sample_points() {
        assert!(carried_rows_object_count_matches_the_unknown_writable_heads(&point), "{point:?}");
        checked += 1;
    }
    assert_eq!(checked, EXPECTED_POINT_COUNT);
}
