use super::*;

fn operation(steps: u64, user_units: u64, allocation_records: u64) -> Operation {
    Operation { steps: Some(steps), user_units: Some(user_units), allocation_records: Some(allocation_records), ..Operation::default() }
}

fn three_decimals(value: Option<f64>) -> String {
    alpha_text(value)
}

#[test]
fn band_edges_sit_where_the_registration_put_them() {
    assert_eq!(three_decimals(positive_alpha(1.0, 1.778, 1.0, 10.0)), "0.250");
    assert_eq!(band(0.25), Band::DoesNotGrow);
    assert_eq!(three_decimals(positive_alpha(1.0, 3.162, 1.0, 10.0)), "0.500");
    assert_eq!(band(0.5), Band::Grows);
    assert_eq!(band(0.4999), Band::Between);
}

#[test]
fn the_spike_reading_of_e206_gives_0_789() {
    assert_eq!(three_decimals(positive_alpha(2102.0, 168_373.0, 922.0, 238_324.0)), "0.789");
}

#[test]
fn constant_steps_do_not_grow() {
    let reading = two_point_reading((500, 1, 1000.0), (500, 1, 10_000.0));
    assert_eq!(reading.alpha, Some(0.0));
    assert_eq!(band(reading.alpha.expect("读得到")), Band::DoesNotGrow);
}

#[test]
fn steps_proportional_to_the_user_units_read_flat_per_unit_and_one_against_the_units() {
    let (low, high) = ((100.0, 10.0), (100_000.0, 10_000.0));
    assert_eq!(three_decimals(positive_alpha(low.0 / low.1, high.0 / high.1, low.1, high.1)), "0.000");
    assert_eq!(three_decimals(positive_alpha(low.0, high.0, low.1, high.1)), "1.000");
}

#[test]
fn equal_user_units_read_raw_steps_and_different_ones_read_per_unit_and_are_looser() {
    let same = two_point_reading((5, 1, 10.0), (50, 1, 100.0));
    assert!(!same.looser);
    assert_eq!(three_decimals(same.alpha), "1.000");
    let different = two_point_reading((5, 1, 10.0), (500, 10, 100.0));
    assert!(different.looser);
    assert_eq!(three_decimals(different.alpha), "1.000");
    assert_eq!(three_decimals(different.raw_alpha), "2.000");
}

#[test]
fn the_scan_control_on_the_one_unit_construction_grows_while_the_tested_arm_does_not() {
    let (low, high) = (operation(4500, 1, 23_923), operation(5500, 1, 239_228));
    let end = |operation: &Operation, arm: Arm| (arm_steps(operation, arm, None).expect("读得到"), 1, measured_total(operation).expect("读得到"));
    assert_eq!(three_decimals(two_point_reading(end(&low, Arm::ScanEveryPublish), end(&high, Arm::ScanEveryPublish)).alpha), "0.935");
    assert_eq!(three_decimals(two_point_reading(end(&low, Arm::Tested), end(&high, Arm::Tested)).alpha), "0.087");
}

#[test]
fn the_scan_control_on_the_many_units_construction_stays_between_and_four_times_grows() {
    let (low, high) = (operation(178_400, 1000, 23_923), operation(178_400, 1000, 239_228));
    let end = |operation: &Operation, arm: Arm| (arm_steps(operation, arm, None).expect("读得到"), 1000, measured_total(operation).expect("读得到"));
    assert_eq!(three_decimals(two_point_reading(end(&low, Arm::ScanEveryPublish), end(&high, Arm::ScanEveryPublish)).alpha), "0.315");
    assert_eq!(
        three_decimals(two_point_reading(end(&low, Arm::ScanEveryPublishFourTimes), end(&high, Arm::ScanEveryPublishFourTimes)).alpha),
        "0.617"
    );
}

#[test]
fn the_linear_control_replaces_lg_by_the_quantity_itself() {
    let mut low = operation(4500, 1, 23_923);
    let mut high = operation(5500, 1, 239_228);
    low.lg_rows.insert("allocator_release_linear_find", (100 * lg(23_923), 100 * 23_923, 100 * lg(23_923)));
    high.lg_rows.insert("allocator_release_linear_find", (128 * lg(239_228), 128 * 239_228, 128 * lg(239_228)));
    let row = Some("allocator_release_linear_find");
    let end = |operation: &Operation| (arm_steps(operation, Arm::LargestLgRowLinear, row).expect("读得到"), 1, measured_total(operation).expect("读得到"));
    assert_eq!(arm_steps(&high, Arm::LargestLgRowLinear, row), Some(5500 - 128 * 18 + 128 * 239_228));
    assert_eq!(three_decimals(two_point_reading(end(&low), end(&high)).alpha), "1.107");
}

#[test]
fn the_linear_control_takes_the_lg_row_with_the_most_steps() {
    let mut one = operation(10, 1, 1);
    one.lg_rows.insert("allocator_release_linear_find", (5, 50, 5));
    one.lg_rows.insert("mapped_units_whole_table", (2, 20, 40));
    let mut other = operation(10, 1, 1);
    other.lg_rows.insert("allocator_release_linear_find", (5, 50, 5));
    assert_eq!(largest_lg_row(&[&one, &other]), Some("mapped_units_whole_table"));
}

#[test]
fn an_lg_term_is_the_lg_part_of_its_row() {
    let input = ByDeltaInput {
        allocation_records: 1000,
        calls: BTreeMap::from([("allocator_release_linear_find".to_string(), 3)]),
        ..ByDeltaInput::default()
    };
    let rows = by_delta_rows(&input);
    let row = rows.iter().find(|(name, _)| *name == "allocator_release_linear_find").expect("行在表里").1;
    let term = lg_terms(&input).into_iter().find(|(name, _, _)| *name == "allocator_release_linear_find").expect("行在表里");
    assert_eq!(row, 30);
    assert_eq!(term.1 * lg(term.2), row);
}

#[test]
fn the_spike_line_takes_the_floor_and_four_times_the_lower_median() {
    assert_eq!(spike_line(&[7, 7, 7, 433, 7]), 28);
    assert!(433 >= spike_line(&[7, 7, 7, 433, 7]) && 7 < spike_line(&[7, 7, 7, 433, 7]));
    assert_eq!(spike_line(&[3, 3, 3, 3, 20]), 28, "四倍中位数只有 12，旗线仍是下限 28，20 不在旗上");
    let with_an_outlier = [7, 7, 7, 7, 7, 7, 7, 433, 60];
    assert_eq!(lower_median(&with_an_outlier), 7);
    assert!(60 >= spike_line(&with_an_outlier), "中位数 7 ⇒ 旗线 28，60 在旗上；均值 60.2 ⇒ 旗线 240，60 不在");
}

#[test]
fn the_first_pick_is_the_first_in_order_and_later_ones_skip_it() {
    let (one, two, three) = (operation(5, 1, 1), operation(50, 1, 1), operation(7, 1, 1));
    let series = [(&one, 5), (&two, 50), (&three, 7)];
    assert_eq!(pick_from(&series, Pick::First).map(|(_, steps)| steps), Some(5));
    assert_eq!(pick_from(&series, Pick::LaterPeak).map(|(_, steps)| steps), Some(50));
    assert_eq!(pick_from(&series, Pick::LaterMedian).map(|(_, steps)| steps), Some(7));
}

#[test]
fn steady_drops_the_flagged_operations_and_all_keeps_them() {
    let flagged = Operation { flagged: true, ..operation(1, 1, 1) };
    let quiet = operation(1, 1, 1);
    assert!(!kept_for_the_filter(&flagged, true));
    assert!(kept_for_the_filter(&flagged, false));
    assert!(kept_for_the_filter(&quiet, true));
}

#[test]
fn the_release_chain_cause_needs_the_release_site_eighty_percent_pulled_and_old_nodes() {
    assert_eq!(release_chain_cause_holds("release_not_covered", 85, 100, 200), (true, Vec::new()));
    assert_eq!(release_chain_cause_holds("placement_not_covered", 85, 100, 200).1, vec!["site"]);
    assert_eq!(release_chain_cause_holds("release_not_covered", 79, 100, 200).1, vec!["pulled_share"]);
    assert_eq!(release_chain_cause_holds("release_not_covered", 85, 100, 99).1, vec!["age_median"]);
}

#[test]
fn three_points_report_adjacent_slopes_not_the_end_to_end_one() {
    let slopes = local_slopes(&[(1.0, 1.0), (2.0, 10.0), (100.0, 100.0)]);
    assert_eq!(slopes.into_iter().map(three_decimals).collect::<Vec<_>>(), vec!["0.301", "1.699"]);
}

#[test]
fn the_measured_total_is_the_allocation_records_and_the_denominator_is_the_declared_units() {
    let operation = Operation { knob: 10, releases: 2000, mapped_units: Some(500), ..operation(1, 1000, 1000) };
    assert_eq!(measured_total(&operation), Some(1000.0));
    assert_eq!(measured_total_by_mapped_units(&operation), Some(500.0));
    assert_eq!(denominator(&operation), Some(1000));
}

#[test]
fn the_factor_table_reads_zero_everywhere_when_nothing_grows() {
    let flat_per_unit = |units: u64| (7 * units, units);
    for (low, high) in [((1, 1), (1, 1)), ((100, 100), (100, 100)), ((100, 1000), (1000, 1000)), ((1000, 10_000), (10_000, 10_000))] {
        let (low_steps, low_units) = flat_per_unit(low.0);
        let (high_steps, high_units) = flat_per_unit(high.0);
        let reading = two_point_reading((low_steps, low_units, low.1 as f64), (high_steps, high_units, 10.0 * high.1 as f64));
        assert_eq!(three_decimals(reading.alpha), "0.000");
    }
}

fn judged(segment: &'static str, tier: &'static str, tested: f64, scan: f64) -> JudgedCell {
    let mut outcome = CellOutcome::default();
    outcome.alpha.insert(Arm::Tested.tag(), Some(tested));
    outcome.alpha.insert(Arm::ScanEveryPublish.tag(), Some(scan));
    outcome.alpha.insert(Arm::LargestLgRowLinear.tag(), Some(1.0));
    JudgedCell { segment, set: "constructed", state: "same_mount", tier, pick: Pick::LaterPeak, steady: true, outcome }
}

#[test]
fn the_void_clause_judges_the_big_points_apart_from_the_new_points() {
    let cells = [judged("new", "user_units_equal_declared", 0.3, 0.32), judged("big", "user_units_equal_declared", 0.3, 0.6)];
    let tallies = control_tallies(&cells);
    let new = tallies.get(&("new", "user_units_equal_declared")).copied().expect("新点那一段有账");
    let big = tallies.get(&("big", "user_units_equal_declared")).copied().expect("大点那一段有自己的账");
    assert!(new.scan_void());
    assert!(!big.scan_void());
    assert_eq!(big, ControlTally { applicable: 1, scan_red: 1, linear_red: 1 });
}

#[test]
fn one_later_steady_cell_that_grows_with_both_controls_flips_row_two() {
    let cell = judged("new", "one_user_unit", 0.6, 0.9);
    let reading = row_reading(&[&cell]);
    assert!(reading.row_two_flips);
    assert!(!reading.row_one_holds);
}

#[test]
fn the_e206_points_take_the_units_from_the_name() {
    assert_eq!(units_by_the_point_name("files_1000_units_100"), 100);
    assert_eq!(units_by_the_point_name("files_100_units_1000_unit_area_2_pow_21"), 1000);
    assert_eq!(units_by_the_point_name("files_1000"), 1);
}

#[test]
fn local_constants_match_the_implementation_lines() {
    assert_eq!(LEAF_SLOTS + INTERNAL_FANOUT + SETTLING_ROUNDS_LIMIT + DISKS, 812 + 169 + 64 + 2);
}

#[test]
fn the_probe_outcomes_sort_into_the_three_hypotheses() {
    let outcome = |cold: (bool, bool), create: (bool, bool), young: (bool, bool)| {
        let wrap = |pair: (bool, bool)| (Some(pair.0), Some(pair.1));
        probe_outcome(&BTreeMap::from([("cold_first", wrap(cold)), ("create_first", wrap(create)), ("young_first", wrap(young))]))
    };
    assert_eq!(outcome((true, false), (false, true), (false, true)), "h1");
    assert_eq!(outcome((true, false), (true, false), (true, false)), "h2");
    assert_eq!(outcome((false, true), (false, true), (false, true)), "h3");
    assert_eq!(outcome((true, true), (false, true), (false, true)), "none_of_the_three");
    assert_eq!(probe_outcome(&BTreeMap::new()), "unread");
}

#[test]
fn the_e206_cold_overwrite_declares_the_built_units_once_and_then_one() {
    let item = PointArmReadings { files: 10, data_units: 1000, units_per_small_file: 1, ..PointArmReadings::default() };
    let mut touched = BTreeSet::new();
    assert_eq!(constructed_user_units(&item, "W2", 1, &mut touched), Some((1000, 1000, 1)));
    assert_eq!(constructed_user_units(&item, "W2", 2, &mut touched), Some((1, 1, 1)));
    assert_eq!(constructed_user_units(&item, "W2", 33, &mut touched), Some((1, 1, 1)), "第 33 次又落回大文件，已覆盖写过，记 1");
    assert_eq!(constructed_user_units(&item, "W3", 1, &mut touched), Some((3, 0, 0)));
    assert_eq!(constructed_user_units(&item, "destroy_snapshot", 1, &mut touched), None);
}

#[test]
fn the_tiers_take_one_unit_and_the_declared_count_from_one_hundred_up() {
    let one = operation(1, 1, 1);
    let large = operation(1, 1000, 1);
    assert!(in_the_tier(&one, "one_user_unit", 1000) && !in_the_tier(&one, "user_units_equal_declared", 1000));
    assert!(in_the_tier(&large, "user_units_equal_declared", 1000) && !in_the_tier(&large, "user_units_equal_declared", 100));
    assert!(!in_the_tier(&operation(1, 10, 1), "user_units_equal_declared", 10), "K 不到 100 的点没有 user_units_equal_declared 档");
}

#[test]
fn only_the_first_growing_is_told_apart_from_a_later_one() {
    let mut first_all = judged("new", "one_user_unit", 0.7, 0.9);
    first_all.pick = Pick::First;
    first_all.steady = false;
    let later = judged("new", "one_user_unit", 0.1, 0.9);
    assert!(row_reading(&[&first_all, &later]).only_the_first_grows);
    let later_grows = judged("new", "one_user_unit", 0.7, 0.9);
    assert!(!row_reading(&[&first_all, &later_grows]).only_the_first_grows);
}

// ---- 报告层：假的 PointArmReadings 与操作喂已知答案 ----
fn item_named(point: &str, arm: &str, units_per_small_file: u64) -> PointArmReadings {
    PointArmReadings {
        name: format!("{point}__{arm}"),
        point: point.to_string(),
        arm: arm.to_string(),
        probe: "standard".to_string(),
        units_per_small_file,
        ..PointArmReadings::default()
    }
}

/// 一次操作：状态、操作、第几次、步数、user_units、操作之前的记录数。
fn operation_at(state: &str, operation_name: &str, operation_index: u64, steps: u64, user_units: u64, records: u64) -> Operation {
    Operation { state: state.to_string(), operation_name: operation_name.to_string(), operation_index, ..operation(steps, user_units, records) }
}

fn points_with<'items>(entries: Vec<(&str, &'items PointArmReadings, Vec<Operation>)>) -> Points<'items> {
    entries.into_iter().map(|(key, item, operations)| (key.to_string(), PointReadings { item, operations })).collect()
}

fn lines_starting<'lines>(out: &'lines [String], start: &str) -> Vec<&'lines str> {
    out.iter().map(String::as_str).filter(|line| line.starts_with(start)).collect()
}

fn settle_at(state: &str, operation_name: &str, operation_index: u64, judgement: u64) -> Settle {
    Settle { state: state.to_string(), operation_name: operation_name.to_string(), operation_index, judgement, ..Settle::default() }
}

/// 一次判的状态读数：B8 式子要的十一样都给（记录数取 `records`）。
fn state_fields(records: u64) -> Fields {
    fields_of(&format!(
        "E207RESULT name=state inode_leaf_containers=1 other_files=0 allocation_records={records} mapped_units=10 deadlist_entries=0 extent_upper_nodes=0 extent_upper_levels=0 accounting_tree_levels=1 deadlist_tree_levels=1 allocation_tree_levels=1 central_mapping_levels=1"
    ))
}

/// 一次判、固定点 0 轮，两道之间那一段 `allocator_release_linear_find` 调 `calls` 次。
fn judgement_calling_the_release_find(calls: u64) -> JudgementReadings {
    let mut judgement = JudgementReadings { fixed_point_rounds: Some(0), ..JudgementReadings::default() };
    judgement.rows.insert(("between".to_string(), "allocator_release_linear_find".to_string()), (calls, 0));
    judgement
}

#[test]
fn one_operation_sums_the_admission_scan_steps_of_its_settles() {
    let mut item = item_named("constructed", "head", 1);
    item.judgements.insert("same_mount/W1/1/".to_string(), JudgementReadings { state: state_fields(1000), ..JudgementReadings::default() });
    item.judgements.insert("same_mount/W1/1/1".to_string(), judgement_calling_the_release_find(1));
    item.judgements.insert("same_mount/W1/1/2".to_string(), judgement_calling_the_release_find(4));
    item.window = vec![settle_at("same_mount", "W1", 1, 1), settle_at("same_mount", "W1", 1, 2)];
    let one = by_delta_of_settle(&item, &item.window[0]).expect("读得到");
    let two = by_delta_of_settle(&item, &item.window[1]).expect("读得到");
    assert_eq!(two, one + 30, "多 3 次 allocator_release_linear_find、记录 1000 条：3 × lg(1000) = 30 步");
    let operations = operations_of(&item);
    assert_eq!(operations.len(), 1, "同一个（状态、操作、第几次）的两次固定点是一次操作");
    assert_eq!(operations[0].steps, Some(one + two));
    assert_eq!(operations[0].allocation_records, Some(1000), "x 取操作之前那一行状态");
}

/// 一次判、固定点 0 轮，两道之间那一段 s37 调 `calls` 次、共 `steps` 步，s29 另有 `per_disk_fallback_steps` 步（都不进 B8 合计）。
fn judgement_calling_the_pooled_fallback(calls: u64, steps: u64, per_disk_fallback_steps: u64) -> JudgementReadings {
    let mut judgement = JudgementReadings { fixed_point_rounds: Some(0), ..JudgementReadings::default() };
    judgement.rows.insert(("between".to_string(), ROW_WITHOUT_A_BY_DELTA_FORMULA.to_string()), (calls, 0));
    judgement.row_steps.insert(("between".to_string(), ROW_WITHOUT_A_BY_DELTA_FORMULA.to_string()), steps);
    judgement.row_steps.insert(("between".to_string(), "fallback_slot_linear_scan".to_string()), per_disk_fallback_steps);
    judgement
}

#[test]
fn the_pooled_fallback_adds_no_by_delta_steps_and_marks_the_operations_that_call_it() {
    let mut item = item_named("constructed", "head", 1);
    item.judgements.insert("same_mount/W1/1/".to_string(), JudgementReadings { state: state_fields(1000), ..JudgementReadings::default() });
    item.judgements.insert("same_mount/W1/1/1".to_string(), judgement_calling_the_pooled_fallback(0, 0, 0));
    item.judgements.insert("same_mount/W1/2/".to_string(), JudgementReadings { state: state_fields(1000), ..JudgementReadings::default() });
    item.judgements.insert("same_mount/W1/2/1".to_string(), judgement_calling_the_pooled_fallback(3, 500, 7));
    item.window = vec![settle_at("same_mount", "W1", 1, 1), settle_at("same_mount", "W1", 2, 1)];
    let (none, called) = (
        by_delta_of_settle(&item, &item.window[0]).expect("读得到"),
        by_delta_of_settle(&item, &item.window[1]).expect("读得到"),
    );
    assert_eq!(none, called, "s37 没有 B8 式子：调 3 次、500 步也不进 B8 合计");
    let operations = operations_of(&item);
    assert_eq!(operations.len(), 2);
    assert_eq!((operations[0].pooled_fallback_calls, operations[0].pooled_fallback_steps, operations[0].fallback_scan_steps), (0, 0, 0));
    assert_eq!((operations[1].pooled_fallback_calls, operations[1].pooled_fallback_steps, operations[1].fallback_scan_steps), (3, 500, 7));
    let mut out = Lines::new();
    let points: Points<'_> = BTreeMap::from([("constructed".to_string(), PointReadings { item: &item, operations })]);
    report_operations(&points, &mut out);
    report_missing_the_pooled_fallback_row(&points, &mut out);
    assert!(out.contains(&"name=op_pooled_fallback item=constructed__head state=same_mount operation=W1 operation_index=1 pooled_fallback_calls=0 pooled_fallback_steps=0 per_disk_fallback_steps=0 by_delta_arms_total=complete".to_string()), "{out:?}");
    assert!(out.contains(&"name=op_pooled_fallback item=constructed__head state=same_mount operation=W1 operation_index=2 pooled_fallback_calls=3 pooled_fallback_steps=500 per_disk_fallback_steps=7 by_delta_arms_total=missing_pooled_fallback_row_only_for_the_listed_steps".to_string()), "{out:?}");
    assert!(out.contains(&"name=verdict quantity=missing_pooled_fallback operations=2 operations_with_pooled_fallback_calls=1 points_marked=1 marked_points=constructed".to_string()), "{out:?}");
}

#[test]
fn a_point_with_a_frozen_publish_is_voided_whole_and_the_unread_field_is_counted() {
    let state_with = |frozen: Option<u64>| {
        let mut state = state_fields(1000);
        if let Some(value) = frozen {
            state.insert("frozen_publish_set".to_string(), value.to_string());
        }
        state
    };
    let point_with = |name: &str, frozen: [Option<u64>; 2]| {
        let mut item = item_named(name, "head", 1);
        for (index, value) in frozen.iter().enumerate() {
            let operation_index = index as u64 + 1;
            item.judgements.insert(format!("same_mount/W1/{operation_index}/"), JudgementReadings { state: state_with(*value), ..JudgementReadings::default() });
            item.judgements.insert(format!("same_mount/W1/{operation_index}/1"), judgement_calling_the_release_find(1));
        }
        item.window = vec![settle_at("same_mount", "W1", 1, 1), settle_at("same_mount", "W1", 2, 1)];
        item
    };
    let (clean, frozen, old) = (point_with("clean", [Some(0), Some(0)]), point_with("frozen", [Some(0), Some(1)]), point_with("old", [None, None]));
    let items = [clean, frozen, old];
    let mut points = points_of(&items);
    let mut out = Lines::new();
    let voided = points_with_a_frozen_publish(&points, &mut out);
    points.retain(|name, _| !voided.contains(name));
    assert_eq!(out, vec![
        "name=void_frozen_publish point=frozen result=point_voided".to_string(),
        "name=verdict quantity=void_frozen_publish operations=6 operations_without_the_reading=2 points_voided=1 voided_points=frozen".to_string(),
    ]);
    assert_eq!(points.len(), 2, "冻结着的那个点整点拿掉，别的点照常");
}

#[test]
fn the_report_reads_no_voided_point_in_any_quantity() {
    let state_with = |frozen: u64| {
        let mut state = state_fields(1000);
        state.insert("frozen_publish_set".to_string(), frozen.to_string());
        state
    };
    let point_with = |name: &str, frozen: u64| {
        let mut item = item_named(name, "head", 1);
        item.judgements.insert("same_mount/W1/1/".to_string(), JudgementReadings { state: state_with(frozen), ..JudgementReadings::default() });
        item.judgements.insert("same_mount/W1/1/1".to_string(), judgement_calling_the_release_find(1));
        item.window = vec![settle_at("same_mount", "W1", 1, 1)];
        item
    };
    let items = [point_with("clean", 0), point_with("frozen", 1)];
    let mut out = Lines::new();
    report_items("third", &items, &[], &mut out);
    assert!(out.iter().any(|line| line.starts_with("name=verdict quantity=void_frozen_publish") && line.contains("voided_points=frozen")), "{out:?}");
    // 作废的点不进后面各量：缺池化回退行那一量只数没作废的那个点的一次操作（变异：作废的点留在各量里时红）
    assert!(out.iter().any(|line| line.starts_with("name=verdict quantity=missing_pooled_fallback operations=1 ")), "{out:?}");
}

#[test]
fn operations_take_the_declared_units_and_not_the_constructed_ones() {
    let mut item = item_named("constructed", "head", 1000);
    item.window = vec![settle_at("same_mount", "W1", 1, 1)];
    item.declared = vec![(("same_mount".to_string(), "W1".to_string(), 1), (7, 7, 1))];
    let operations = operations_of(&item);
    assert_eq!((operations[0].user_units, operations[0].old_units, operations[0].new_units), (Some(7), 7, 1));
    item.declared.clear();
    assert_eq!(operations_of(&item)[0].user_units, Some(1), "没有声明行（E206 产物）才按构造补：W1 覆盖写第一个文件记 1");
}

#[test]
fn the_comparison_sets_are_the_registered_points() {
    let shape = |sets: &[ComparisonSet]| -> Vec<(&'static str, &'static str, Vec<&'static str>, Vec<&'static str>)> {
        sets.iter().map(|set| (set.segment, set.name, set.points.to_vec(), set.tiers.to_vec())).collect()
    };
    assert_eq!(
        shape(&NEW_POINT_SETS[..]),
        vec![
            ("new", "K100", vec!["files_100_units_100", "files_1000_units_100"], vec!["one_user_unit", "user_units_equal_declared"]),
            ("new", "K1000", vec!["files_10_units_1000", "files_100_units_1000"], vec!["one_user_unit", "user_units_equal_declared"]),
            ("new", "FTB", vec!["files_then_big_1000", "files_then_big_10000"], vec!["one_user_unit"]),
            ("new", "D100", vec!["files_100_units_100", "files_100_units_1000"], vec!["user_units_equal_declared"]),
            ("new", "D10", vec!["files_10_units_1000", "files_10_units_10000"], vec!["user_units_equal_declared"]),
            ("new", "K1000_other_unit_area", vec!["files_10_units_1000_unit_area_2_pow_17", "files_100_units_1000_unit_area_2_pow_20"], vec!["one_user_unit", "user_units_equal_declared"]),
        ]
    );
    assert_eq!(
        shape(&BIG_POINT_SETS[..]),
        vec![(
            "big",
            "K1000L",
            vec!["files_10_units_1000_unit_area_2_pow_21", "files_100_units_1000_unit_area_2_pow_21", "files_1000_units_1000"],
            vec!["one_user_unit", "user_units_equal_declared"]
        )]
    );
    assert_eq!(shape(&RETRO_SETS[..]).iter().map(|set| set.2.len()).collect::<Vec<_>>(), vec![4, 4, 4, 4], "回扫四根轴各四点");
}

#[test]
fn the_quantities_take_the_registered_sets() {
    let table: Vec<(&str, &str, &str, Vec<&str>)> =
        NEW_POINT_QUANTITIES.iter().map(|(reading, quantity, tier, sets)| (*reading, *quantity, *tier, sets.to_vec())).collect();
    assert_eq!(
        table,
        vec![
            ("one_unit_change_against_the_total", "q1", "one_user_unit", vec!["K100", "K1000", "FTB"]),
            ("many_unit_change_against_the_total", "q2", "user_units_equal_declared", vec!["K100", "K1000"]),
            ("per_unit_steps_as_the_change_grows", "q3", "user_units_equal_declared", vec!["D100", "D10"]),
            ("many_unit_change_against_the_total_on_the_other_unit_area", "q2_other_unit_area", "user_units_equal_declared", vec!["K1000_other_unit_area"]),
            ("one_unit_change_against_the_total_on_the_other_unit_area", "q1_other_unit_area", "one_user_unit", vec!["K1000_other_unit_area"]),
        ]
    );
}

#[test]
fn the_controls_read_the_same_operation_as_the_tested_arm() {
    let (low_item, high_item) = (item_named("low", "head", 1), item_named("high", "head", 1));
    let spiked = |operation_index: u64, steps: u64, records: u64, flagged: bool| Operation { flagged, ..operation_at("same_mount", "W1", operation_index, steps, 1, records) };
    let points = points_with(vec![
        ("low", &low_item, vec![spiked(1, 1000, 100, true), spiked(2, 10, 100, false)]),
        ("high", &high_item, vec![spiked(1, 1000, 1000, true), spiked(2, 10, 1000, false)]),
    ]);
    let set = ComparisonSet { segment: "new", name: "constructed", points: &["low", "high"], tiers: &["one_user_unit"] };
    let mut out = Lines::new();
    let outcome = report_cell(&points, &set, "same_mount", "one_user_unit", Pick::First, true, &mut out);
    assert_eq!(lines_starting(&out, "name=cell ").len(), 4, "每条臂一行");
    assert_eq!(three_decimals(outcome.alpha["tested"]), "0.000", "steady 剔掉旗上的 W1#1，两端都取 W1#2：10 步对 10 步");
    assert_eq!(three_decimals(outcome.alpha["c1"]), "0.963", "C1 同样只取 W1#2：(10 + 100) 对 (10 + 1000)");
    assert_eq!(outcome.alpha["c3"], None, "这个点没有含 lg 的行，C3 读不到");
}

#[test]
fn one_negative_control_over_four_rounds_fails_the_item_check() {
    let check = |rounds: u64| -> String {
        let mut item = item_named("base", "head", 1);
        item.window = vec![Settle { rounds, ..settle_at("same_mount", "negative_control", 1, 1) }];
        let point = PointReadings { item: &item, operations: Vec::new() };
        let mut out = Lines::new();
        report_item_checks(&point, &mut out);
        out.concat()
    };
    assert!(check(4).contains(" negative_control_rounds=4 negative_control_ok=true"));
    assert!(check(5).contains(" negative_control_rounds=5 negative_control_ok=false"));
}

fn judged_with(tested: f64, scan: f64, linear: f64) -> JudgedCell {
    let mut cell = judged("new", "one_user_unit", tested, scan);
    cell.outcome.alpha.insert(Arm::LargestLgRowLinear.tag(), Some(linear));
    cell
}

#[test]
fn one_growing_later_cell_needs_both_controls_red_to_flip_row_two() {
    assert!(row_reading(&[&judged_with(0.6, 0.9, 1.0)]).row_two_flips);
    assert!(!row_reading(&[&judged_with(0.6, 0.9, 0.1)]).row_two_flips, "C3 没红");
    assert!(!row_reading(&[&judged_with(0.6, 0.1, 1.0)]).row_two_flips, "C1 没红");
    assert!(row_reading(&[&judged_with(0.1, 0.9, 1.0)]).row_one_holds);
    assert!(!row_reading(&[&judged_with(0.1, 0.9, 0.1)]).row_one_holds, "行一同样要两条对照都红");
}

#[test]
fn the_similar_total_set_reads_the_growth_against_the_user_units() {
    let (low, high) = (item_named("files_100_units_1000", "head", 1000), item_named("files_10_units_10000", "head", 10_000));
    let points = points_with(vec![
        ("files_100_units_1000", &low, vec![operation_at("same_mount", "W2", 1, 100_000, 1000, 1000)]),
        ("files_10_units_10000", &high, vec![operation_at("same_mount", "W2", 1, 1_000_000, 10_000, 1100)]),
    ]);
    let mut out = Lines::new();
    report_similar_total_set(&points, &mut out);
    assert_eq!(out.len(), 12, "两种状态 × 三种取法 × 剔不剔旗");
    assert!(out.contains(
        &"name=t1e5 state=same_mount pick=first filter=all alpha_against_the_user_units_per_unit=0.000 band_per_unit=does_not_grow alpha_against_the_user_units_raw=1.000 band_raw=grows".to_string()
    ));
}

#[test]
fn the_retro_first_cold_overwrite_reads_per_unit_and_is_looser() {
    let (base, top) = (item_named("base", "p2a", 1), item_named("data_units_10000", "p2a", 1));
    let retro = points_with(vec![
        ("base", &base, vec![operation_at("same_mount", "W2", 1, 1000, 10, 100)]),
        ("data_units_10000", &top, vec![operation_at("same_mount", "W2", 1, 1_000_000, 10_000, 1000)]),
    ]);
    let mut out = Lines::new();
    report_retro_data_units_first_cold(&retro, &mut out);
    assert_eq!(
        out,
        vec!["name=q4_data_units_first_cold_overwrite ends=1000/10/100,unread,unread,1000000/10000/1000 looser=yes alpha=0.000 band=does_not_grow raw_alpha=3.000 raw_band=grows".to_string()]
    );
}

#[test]
fn one_cell_where_the_tested_arm_grows_is_not_applicable_to_the_controls() {
    let tallies = control_tallies(&[judged("new", "one_user_unit", 0.6, 0.1)]);
    assert_eq!(tallies.get(&("new", "one_user_unit")).copied(), Some(ControlTally::default()));
    assert!(!tallies[&("new", "one_user_unit")].scan_void());
}

#[test]
fn the_spike_flags_line_lists_the_flagged_operations_and_the_peak() {
    let item = item_named("constructed", "head", 1);
    let nodes = |operation_index: u64, steps: u64, final_nodes: u64, flagged: bool| Operation { final_nodes, flagged, ..operation_at("same_mount", "W1", operation_index, steps, 1, 100) };
    let points = points_with(vec![("constructed", &item, vec![nodes(1, 5, 7, false), nodes(2, 900, 433, true), nodes(3, 6, 7, false), nodes(4, 4, 7, false)])]);
    let mut out = Lines::new();
    report_operations(&points, &mut out);
    assert_eq!(lines_starting(&out, "name=op_row ").len(), 4, "每次操作一行");
    assert_eq!(
        lines_starting(&out, "name=q6_spike_flags "),
        vec![
            "name=q6_spike_flags point=constructed state=same_mount operations=4 median_final_nodes=7 line=28 flagged=1 flagged_operations=W1#2:433 peak_steps=900 peak_at=W1#2 final_steps=4",
            "name=q6_spike_flags point=constructed state=remounted operations=0 median_final_nodes=0 line=28 flagged=0 flagged_operations=- peak_steps=unread peak_at=unread final_steps=unread",
        ]
    );
}

#[test]
fn operations_flag_the_spike_against_the_line_of_their_own_state() {
    let mut item = item_named("constructed", "head", 1);
    item.window = vec![
        Settle { final_nodes: 7, ..settle_at("same_mount", "W1", 1, 1) },
        Settle { final_nodes: 7, ..settle_at("same_mount", "W1", 2, 1) },
        Settle { final_nodes: 7, ..settle_at("same_mount", "W1", 3, 1) },
        Settle { final_nodes: 28, ..settle_at("same_mount", "W1", 4, 1) },
        Settle { final_nodes: 7, ..settle_at("same_mount", "W1", 5, 1) },
        Settle { final_nodes: 433, ..settle_at("remounted", "W1", 1, 1) },
    ];
    let flagged: Vec<(String, u64)> = operations_of(&item).into_iter().filter(|operation| operation.flagged).map(|operation| (operation.state, operation.operation_index)).collect();
    assert_eq!(flagged, vec![("same_mount".to_string(), 4)], "28 正在线上算旗上；433 只比它自己那一状态的线（4 × 433）");
}

#[test]
fn the_then_big_ratio_takes_the_largest_ratio_over_the_same_operations() {
    let (then_big, files) = (item_named("files_then_big_10000", "head", 1), item_named("files_10000", "head", 1));
    let points = points_with(vec![
        ("files_then_big_10000", &then_big, vec![operation_at("same_mount", "W1", 1, 300, 1, 1), operation_at("same_mount", "W1", 2, 500, 1, 1)]),
        ("files_10000", &files, vec![operation_at("same_mount", "W1", 1, 200, 1, 1), operation_at("same_mount", "W1", 2, 200, 1, 1), operation_at("same_mount", "W2", 1, 1, 1, 1)]),
    ]);
    let mut out = Lines::new();
    report_then_big_ratio(&points, &mut out);
    assert_eq!(out, vec!["name=verdict quantity=q9 maximum_ratio=2.500 at=same_mount/W1#2 exceeds=true".to_string()]);
}

#[test]
fn the_declared_line_is_read_back_into_the_item() {
    let mut item = PointArmReadings::default();
    take_line(
        &mut item,
        &fields_of("E207RESULT name=declared item=files_10_units_1000__head phase=window state=same_mount operation=W2 operation_index=1 user_units=1000 old_units=1000 new_units=1"),
        E207_OPERATION_KEYS,
    );
    assert_eq!(item.declared, vec![(("same_mount".to_string(), "W2".to_string(), 1), (1000, 1000, 1))]);
}

#[test]
fn each_line_is_read_with_the_operation_keys_of_its_own_experiment() {
    // E206 留存产物逐字的那一对键，钉在这里；E206 那一行照这一对拼。
    assert_eq!(E206_OPERATION_KEYS, OperationKeys { name: "op", index: "op_index" });
    let lines = [
        format!(
            "E206RESULT name=settle item=files_1000__p2a phase=window state=same_mount {}=W1 {}=2 judgement=1 rounds=3",
            E206_OPERATION_KEYS.name, E206_OPERATION_KEYS.index
        ),
        "E207RESULT name=settle item=files_1000__head phase=window state=remounted operation=W2 operation_index=4 judgement=1 rounds=2".to_string(),
    ];
    let mut item = PointArmReadings::default();
    for line in &lines {
        take_line(&mut item, &fields_of(line), operation_keys_of(line));
    }
    let read: Vec<(String, String, u64, u64)> =
        item.window.iter().map(|settle| (settle.state.clone(), settle.operation_name.clone(), settle.operation_index, settle.rounds)).collect();
    assert_eq!(
        read,
        vec![
            ("same_mount".to_string(), "W1".to_string(), 2, 3),
            ("remounted".to_string(), "W2".to_string(), 4, 2),
        ]
    );
}

#[test]
fn the_item_check_compares_the_product_constants_with_the_local_ones() {
    let check = |leaf_slots: u64| -> String {
        let mut item = item_named("base", "head", 1);
        item.constants = fields_of(&format!("E207RESULT name=constants leaf_slots={leaf_slots} internal_fanout=169 settling_rounds_at_most=64 disks=2"));
        let mut out = Lines::new();
        report_item_checks(&PointReadings { item: &item, operations: Vec::new() }, &mut out);
        out.concat()
    };
    assert!(check(812).contains(" constants_match=true "));
    assert!(check(813).contains(" constants_match=false "));
}

fn round_with(rewrite_set_size: u64, exit: &str, release_dirty_new: u64, holding_differs: bool) -> Round {
    Round { rewrite_set_size, exit: exit.to_string(), release_dirty_new, holding_differs, ..Round::default() }
}

#[test]
fn the_fixed_point_premises_are_checked_from_the_round_readings() {
    let clean = Settle { detail: vec![round_with(3, "release_not_covered", 0, false), round_with(5, "settled", 0, false)], ..Settle::default() };
    assert_eq!(fixed_point_premise_breaks(&[clean]), (0, 0));
    let shrinking = Settle {
        detail: vec![round_with(3, "release_not_covered", 0, false), round_with(5, "placement_not_covered", 0, false), round_with(4, "settled", 0, false)],
        ..Settle::default()
    };
    assert_eq!(fixed_point_premise_breaks(&[shrinking]), (1, 0), "5 → 4 变小了一次");
    let dirty = Settle { detail: vec![round_with(3, "settled", 1, false)], ..Settle::default() };
    let holding = Settle { detail: vec![round_with(3, "settled", 0, true)], ..Settle::default() };
    assert_eq!(fixed_point_premise_breaks(&[dirty, holding]), (0, 2), "定下那一轮还有新弄脏、或有记录的节点对不上");
}

#[test]
fn the_round_work_is_held_against_the_sum_of_the_rewritten_roles() {
    let rounds = |roles: &[u64]| -> Vec<Round> { roles.iter().map(|roles_rewritten| Round { roles_rewritten: *roles_rewritten, ..Round::default() }).collect() };
    let settled = Settle { round_work: 8, detail: rounds(&[3, 5][..]), ..Settle::default() };
    let off = Settle { round_work: 9, detail: rounds(&[3, 5][..]), ..Settle::default() };
    let history = Settle { round_work: 9, ..Settle::default() };
    assert_eq!(round_work_mismatches(&[settled, off, history]), 1, "W = 3 + 5 = 8 的对得上；9 的对不上；历史那一次没有逐轮读数，不比");
}

#[test]
fn the_old_criterion_line_is_written_like_the_e206_report() {
    let first = old_criterion_line("q8_first_publish", "files", &[Some(2102.0), Some(2331.0), Some(4598.0), Some(15_440.0)]);
    assert_eq!(first, OLD_CRITERION_FIRST_PUBLISH_FILES);
    let peak = old_criterion_line("q8_peak", "data_units", &[Some(3763.0), Some(17_200.0), Some(179_690.0), Some(3_231_038.0)]);
    assert_eq!(peak, OLD_CRITERION_PEAK_DATA_UNITS);
    assert_eq!(
        old_criterion_line("q8_peak", "data_units", &[Some(3763.0), None, Some(179_690.0), None]),
        "name=q8_peak arm=p2a axis=data_units low_end=3763 high_end=unread alpha=unread band=unread local_slopes=unread,unread,unread"
    );
    let mut out = Lines::new();
    assert!(!old_criterion_pins(&[], &mut out), "没有回扫产物两行都读不到，不当相同");
    assert_eq!(out[2], "name=verdict quantity=old_criterion_pins first_publish_files_matches=false peak_data_units_matches=false");
}

#[test]
fn the_reconciliation_compares_every_window_settle_with_e206() {
    let window = |hash: &str| -> Vec<Settle> {
        vec![Settle { rounds: 2, final_roles: 20, final_nodes: 7, round_work: 32, final_set_hash: hash.to_string(), ..settle_at("same_mount", "W1", 1, 1) }]
    };
    let (mut today_base, mut before_base) = (item_named("base", "head", 1), item_named("base", "p2a", 1));
    today_base.window = window("aa");
    before_base.window = window("bb");
    let (mut today_files, mut before_files) = (item_named("files_1000", "head", 1), item_named("files_1000", "p2a", 1));
    today_files.window = window("cc");
    before_files.window = window("cc");
    let today = points_with(vec![("base", &today_base, Vec::new()), ("files_1000", &today_files, Vec::new())]);
    let before = points_with(vec![("base", &before_base, Vec::new()), ("files_1000", &before_files, Vec::new())]);
    let mut out = Lines::new();
    report_reconciliation(&today, &before, &mut out);
    assert_eq!(
        out,
        vec![
            "name=verdict quantity=reconciliation_with_e206 point=base settles_today=1 settles_e206=1 differing=1 matches=false first_differing=same_mount/W1/1/1",
            "name=verdict quantity=reconciliation_with_e206 point=files_1000 settles_today=1 settles_e206=1 differing=0 matches=true first_differing=-",
            "name=verdict quantity=reconciliation_with_e206 point=files_10000 matches=not_run",
        ]
    );
}

fn spike_point(final_nodes: u64) -> PointArmReadings {
    let mut item = item_named("files_1000_units_100", "head", 100);
    item.window = vec![
        Settle { rounds: 3, final_roles: 452, final_nodes, round_work: 917, closure_nodes_added: 410, ..settle_at("same_mount", "W1", 1, 1) },
        Settle { rounds: 2, final_roles: 20, final_nodes: 7, round_work: 32, ..settle_at("same_mount", "W1", 2, 1) },
    ];
    let state = fields_of("E207RESULT name=state mapped_units=101581 allocation_records=238324");
    item.judgements.insert("same_mount/W1/1/".to_string(), JudgementReadings { state, ..JudgementReadings::default() });
    item
}

#[test]
fn the_spike_pin_holds_the_registered_numbers() {
    for (final_nodes, expected) in [
        (433, "name=verdict quantity=spike_pin matches=true mismatches=-"),
        (432, "name=verdict quantity=spike_pin matches=false mismatches=W1#1.final_nodes=432≠433"),
    ] {
        let item = spike_point(final_nodes);
        let points = points_with(vec![("files_1000_units_100", &item, Vec::new())]);
        let mut out = Lines::new();
        report_spike_pin(&points, &mut out);
        assert_eq!(out, vec![expected]);
    }
    let mut out = Lines::new();
    report_spike_pin(&Points::new(), &mut out);
    assert_eq!(out, vec!["name=verdict quantity=spike_pin matches=not_run"]);
}

#[test]
fn the_declared_units_are_held_against_the_mapped_units_on_first_touches() {
    let item = item_named("files_10_units_1000", "head", 1000);
    let touch = |operation_index: u64, mapped: u64| Operation { old_units: 1000, new_units: 1, mapped_units: Some(mapped), ..operation_at("same_mount", "W2", operation_index, 1, 1000, 1) };
    let mut operations = vec![touch(1, 10_000), touch(2, 9001), touch(3, 8002), touch(4, 7003), touch(5, 6004)];
    operations.push(Operation { mapped_units: Some(5005), ..operation_at("same_mount", "W3", 1, 1, 3, 1) });
    let points = points_with(vec![("files_10_units_1000", &item, operations.clone())]);
    let mut out = Lines::new();
    report_declared_against_mapped(&points, &mut out);
    assert_eq!(
        out,
        vec![
            "name=verdict quantity=declared_against_mapped point=files_10_units_1000 state=same_mount first_touches=5 enough_first_touches=true ratios_outside=0 outside=-",
            "name=verdict quantity=declared_against_mapped point=files_10_units_1000 state=remounted first_touches=0 enough_first_touches=false ratios_outside=0 outside=-",
        ],
        "每次换下 999 个单元、mapped_units 也少 999：比值 1，落在 [0.5, 2]"
    );
    let four = points_with(vec![("files_10_units_1000", &item, operations[1..].to_vec())]);
    let mut lines_for_four = Lines::new();
    report_declared_against_mapped(&four, &mut lines_for_four);
    assert!(lines_for_four[0].contains(" first_touches=4 enough_first_touches=false "), "F8：第一次碰的少于 5 个，这一状态的 user_units_equal_declared 档样本不足");
}

#[test]
fn the_plain_build_is_held_against_the_counted_build_on_the_image() {
    let with_image = |point: &str, arm: &str, sha: &str| PointArmReadings { image_sha256_at_the_history_end: Some(sha.to_string()), ..item_named(point, arm, 1) };
    let (base, base_plain) = (with_image("base", "head", "aa"), with_image("base", "plain", "aa"));
    let (files, files_plain) = (with_image("files_1000", "head", "aa"), with_image("files_1000", "plain", "bb"));
    let points = points_with(vec![
        ("base", &base, Vec::new()),
        ("base__plain", &base_plain, Vec::new()),
        ("files_1000", &files, Vec::new()),
        ("files_1000__plain", &files_plain, Vec::new()),
    ]);
    let mut out = Lines::new();
    report_image_pins(&points, &mut out);
    assert_eq!(out, vec!["name=verdict quantity=plain_image point=base same_image=true", "name=verdict quantity=plain_image point=files_1000 same_image=false"]);
    let mut lines_without_operations = Lines::new();
    report_image_pins(&points_with(vec![("base", &base, Vec::new())]), &mut lines_without_operations);
    assert_eq!(lines_without_operations[0], "name=verdict quantity=plain_image point=base same_image=not_run");
}

#[test]
fn the_big_point_units_are_held_inside_the_registered_range() {
    let item = item_named("files_1000_units_1000", "head", 1000);
    let check = |mapped: u64| -> Lines {
        let points = points_with(vec![("files_1000_units_1000", &item, vec![Operation { mapped_units: Some(mapped), ..operation_at("same_mount", "W1", 1, 1, 1, 1) }])]);
        let mut out = Lines::new();
        report_unit_counts(&points, &mut out);
        out
    };
    assert_eq!(
        check(1_020_000),
        vec![
            "name=occupancy point=files_1000_units_1000 units_per_small_file=1000 mapped_units_at_the_history_end=1020000 change_share=0.0010 far_below_the_total=true",
            "name=verdict quantity=big_point_units point=files_1000_units_1000 mapped_units=1020000 inside=true",
        ]
    );
    assert_eq!(check(999_999)[1], "name=verdict quantity=big_point_units point=files_1000_units_1000 mapped_units=999999 inside=false");
}

#[test]
fn looser_instances_count_the_cells_where_only_the_per_unit_reading_stays_flat() {
    let cell = |raw: f64, per_unit: f64| {
        let mut cell = judged("new", "user_units_equal_declared", per_unit, 0.9);
        cell.outcome.raw_alpha.insert(Arm::Tested.tag(), Some(raw));
        cell
    };
    let mut out = Lines::new();
    report_looser_instances(&[cell(0.6, 0.2), cell(0.4, 0.2)], &mut out);
    assert_eq!(
        out,
        vec![
            "name=f3_looser_instance set=constructed state=same_mount tier=user_units_equal_declared pick=later_peak filter=steady raw_alpha=0.600 per_unit_alpha=0.200",
            "name=f3_looser_instances count=1",
        ]
    );
}

#[test]
fn item_checks_count_the_fixed_points_that_did_not_settle_or_hit_the_limit() {
    let mut item = item_named("base", "head", 1);
    item.window = vec![
        Settle { outcome: "did_not_settle".to_string(), rounds: 3, ..settle_at("same_mount", "W1", 1, 1) },
        Settle { outcome: "settled".to_string(), rounds: 64, ..settle_at("same_mount", "W1", 2, 1) },
        Settle { outcome: "settled".to_string(), rounds: 63, ..settle_at("same_mount", "W1", 3, 1) },
    ];
    let mut out = Lines::new();
    report_item_checks(&PointReadings { item: &item, operations: Vec::new() }, &mut out);
    assert!(out[0].contains(" did_not_settle=1 rounds_at_the_limit=1 "), "{}", out[0]);
}

#[test]
fn an_item_with_missing_declared_lines_or_without_done_is_reported() {
    let mut item = item_named("files_10_units_1000", "head", 1000);
    item.declared = vec![(("same_mount".to_string(), "W1".to_string(), 1), (1, 1, 1))];
    let operations = vec![operation_at("same_mount", "W1", 1, 1, 1, 1), operation_at("same_mount", "W1", 2, 1, 1, 1)];
    let mut out = Lines::new();
    report_item_checks(&PointReadings { item: &item, operations: operations.clone() }, &mut out);
    assert!(out[0].contains(" item_done=false "), "{}", out[0]);
    assert!(out[0].contains(" declared_lines=1 window_operations=2 declared_lines_match=false "), "{}", out[0]);
    let complete = PointArmReadings { done: true, declared: vec![item.declared[0].clone(), (("same_mount".to_string(), "W1".to_string(), 2), (1, 1, 1))], ..item.clone() };
    let mut lines_for_a_complete_item = Lines::new();
    report_item_checks(&PointReadings { item: &complete, operations }, &mut lines_for_a_complete_item);
    assert!(lines_for_a_complete_item[0].contains(" item_done=true ") && lines_for_a_complete_item[0].contains(" declared_lines_match=true "), "{}", lines_for_a_complete_item[0]);
}

#[test]
fn the_three_point_set_without_its_high_end_is_not_read_as_three_points() {
    let (low, middle) = (item_named("low", "head", 1), item_named("middle", "head", 1));
    let points = points_with(vec![
        ("low", &low, vec![operation_at("same_mount", "W1", 1, 10, 1, 100)]),
        ("middle", &middle, vec![operation_at("same_mount", "W1", 1, 100, 1, 1000)]),
    ]);
    let set = ComparisonSet { segment: "big", name: "constructed", points: &["low", "middle", "high"], tiers: &["one_user_unit"] };
    let mut out = Lines::new();
    let outcome = report_cell(&points, &set, "same_mount", "one_user_unit", Pick::First, false, &mut out);
    assert_eq!(outcome.alpha["tested"], None, "高端缺了，α 不拿低端与中点冒充");
    let tested = lines_starting(&out, "name=cell ")[0];
    assert!(tested.contains(" alpha=unread ") && tested.contains(" local_slopes=high_end_missing lower_pair_alpha=1.000 "), "{tested}");
}

fn window_of(state: &str, operations: &[(&str, u64)]) -> Vec<Operation> {
    operations.iter().flat_map(|(operation_name, count)| (1..=*count).map(move |operation_index| operation_at(state, operation_name, operation_index, 1, 1, 1))).collect()
}

#[test]
fn the_window_shape_check_names_a_missing_probe_prefix_and_extended_window() {
    let item = PointArmReadings { probe: "cold_first".to_string(), ..item_named("files_1000_units_100", "head", 100) };
    let mut operations = window_of("same_mount", &WINDOW_OPERATIONS);
    operations.extend(window_of("remounted", &WINDOW_OPERATIONS));
    assert_eq!(
        window_shape_mismatches(&PointReadings { item: &item, operations: operations.clone() }),
        vec!["same_mount/P1:0≠1", "same_mount/W5:0≠200", "same_mount/W6:0≠110", "same_mount/W7:0≠11"]
    );
    operations.extend(window_of("same_mount", &EXTENDED_WINDOW_OPERATIONS));
    operations.push(operation_at("same_mount", "P1", 1, 1, 1, 1));
    assert!(window_shape_mismatches(&PointReadings { item: &item, operations }).is_empty());
    let base = item_named("base", "head", 1);
    let mut base_operations = window_of("same_mount", &WINDOW_OPERATIONS);
    base_operations.extend(window_of("remounted", &WINDOW_OPERATIONS));
    base_operations.push(operation_at("remounted", "negative_control", 1, 1, 1, 1));
    assert_eq!(window_shape_mismatches(&PointReadings { item: &base, operations: base_operations }), vec!["same_mount/negative_control:0≠1"]);
}

// ---- 复量段 hundred_units_per_file_repeat（修订 2）----
/// 六点 x 每步乘 10 的 0.25 次幂，相邻五段的指数 `exponents`：Q = 上一点的 Q × (x 之比)^指数。返回 (Q, x)。
fn series_from_exponents(exponents: [f64; 5]) -> Vec<(f64, f64)> {
    let mut points = vec![(1.0, 1.0)];
    for (index, exponent) in exponents.iter().enumerate() {
        let position = 10_f64.powf(0.25 * (index as f64 + 1.0));
        let (previous_quantity, previous_position) = *points.last().expect("非空");
        points.push((previous_quantity * (position / previous_position).powf(*exponent), position));
    }
    points
}

#[test]
fn the_series_verdicts_match_the_five_registered_constructions() {
    let read = |exponents: [f64; 5]| series_reading(&series_from_exponents(exponents));
    let all_grow = read([0.6; 5]);
    assert_eq!((alpha_text(all_grow.beta), all_grow.grows, all_grow.verdict), ("0.600".to_string(), 5, "stable_grows"));
    assert_eq!(alphas_text(&all_grow), "0.600,0.600,0.600,0.600,0.600");
    let between = read([0.4; 5]);
    assert_eq!((alpha_text(between.beta), between.between, between.verdict), ("0.400".to_string(), 5, "stable_between"));
    let flat = read([0.1; 5]);
    assert_eq!((alpha_text(flat.beta), flat.does_not_grow, flat.verdict), ("0.100".to_string(), 5, "stable_does_not_grow"));
    let falling = read([0.6, 0.6, 0.6, 0.1, 0.1]);
    assert_eq!((alpha_text(falling.beta), falling.grows, falling.does_not_grow, falling.verdict), ("0.414".to_string(), 3, 2, "unstable"));
    let late_rise = read([0.1, 0.1, 0.1, 0.1, 0.9]);
    assert_eq!((alpha_text(late_rise.beta), late_rise.grows, late_rise.does_not_grow, late_rise.verdict), ("0.214".to_string(), 1, 4, "unstable"));
    assert_eq!(band_text(late_rise.beta), "does_not_grow", "只在最高端翘起：β 读不随；首尾两点的 α 是 0.260（介于），不是 β");
}

#[test]
fn one_growing_segment_among_between_ones_is_unstable_and_two_points_are_not_enough() {
    let one_grows = series_reading(&series_from_exponents([0.5, 0.4, 0.4, 0.4, 0.4]));
    assert_eq!((alpha_text(one_grows.beta), one_grows.grows, one_grows.verdict), ("0.414".to_string(), 1, "unstable"));
    assert_eq!(series_reading(&series_from_exponents([0.6; 5])[..2]).verdict, "not_enough_points", "读到的点少于三个不判");
    assert_eq!(series_reading(&series_from_exponents([0.6; 5])[..3]).verdict, "unstable", "三个点两段：g = 2 < 3，不满足稳");
}

#[test]
fn the_controls_must_both_read_grows_or_the_primary_cell_is_void() {
    assert!(!controls_are_red(Some(0.3), Some(0.9)), "C1 的 β 是 0.3：不红");
    assert!(!controls_are_red(Some(0.9), Some(0.49)), "C3 的 β 不到 0.5：不红");
    assert!(!controls_are_red(Some(0.9), None), "读不到不算红");
    assert!(controls_are_red(Some(0.5), Some(0.9)));
    assert_eq!(series_route("stable_does_not_grow", "-"), "does_not_support_the_flip");
    assert_eq!(series_route("stable_grows", "control_not_red"), "series_not_made");
    assert_eq!(series_route("unstable", "-"), "unstable_the_cell_stays_open");
    assert_eq!(series_route("not_enough_points", "-"), "series_not_made");
}

/// 一个序列点的产物：状态里有 5 个第一次碰 K = 100 个单元的 W2、3 个 W1，步数固定、记录数照 `records`；`with_lg_row` 给 `C3` 一行含 lg 的行；扩展窗口的一个 W7 操作 `late_spike` 步。
fn series_point(name: &str, records: u64, with_lg_row: bool, late_spike: Option<u64>) -> (PointArmReadings, Vec<Operation>) {
    let mut item = PointArmReadings { done: true, ..item_named(name, "head", 100) };
    for file in ["crates/singlefs-core/src/transaction.rs", "crates/singlefs-core/src/allocation_record_tree.rs", "crates/singlefs-core/src/allocator.rs"] {
        item.source_sha256.insert(file.to_string(), "00".to_string());
    }
    item.window = vec![Settle { rounds: 2, final_roles: 3, final_nodes: 4, round_work: 5, final_set_hash: "ab".to_string(), previous_records: records, ..settle_at("same_mount", "W1", 1, 0) }];
    let mut operations = Vec::new();
    for state in ["same_mount", "remounted"] {
        for index in 1..=3 {
            operations.push(operation_at(state, "W1", index, 100, 1, records));
        }
        for index in 1..=5 {
            operations.push(Operation { old_units: 100, ..operation_at(state, "W2", index, 100, 100, records) });
        }
    }
    if let Some(steps) = late_spike {
        operations.push(operation_at("same_mount", "W7", 1, steps, 1, records));
    }
    if with_lg_row {
        for operation in &mut operations {
            operation.lg_rows.insert("allocator_release_linear_find", (10, 10 * records, 10));
        }
    }
    (item, operations)
}

fn series_report_tweaked(with_lg_row: bool, late_spike: Option<u64>, tweak: impl FnOnce(&mut Vec<(PointArmReadings, Vec<Operation>)>, &mut PointArmReadings)) -> Lines {
    let totals = [14_000, 25_000, 45_000, 80_000, 140_000, 250_000];
    let mut built: Vec<(PointArmReadings, Vec<Operation>)> = HUNDRED_UNITS_PER_FILE_SERIES
        .iter()
        .zip(totals)
        .map(|(name, records)| series_point(name, records, with_lg_row, if *name == "files_1000_units_100" { late_spike } else { None }))
        .collect();
    let mut pin = built[1].0.clone();
    tweak(&mut built, &mut pin);
    let points = points_with(HUNDRED_UNITS_PER_FILE_SERIES.iter().zip(&built).map(|(name, (item, operations))| (*name, item, operations.clone())).collect());
    let mut out = Lines::new();
    report_hundred_units_per_file_series(&points, &[pin], &mut out);
    out
}

fn series_report(with_lg_row: bool, late_spike: Option<u64>) -> Lines {
    series_report_tweaked(with_lg_row, late_spike, |_, _| {})
}

#[test]
fn the_hundred_units_per_file_series_reports_the_primary_cell_with_the_controls_red() {
    let out = series_report(true, None);
    let primary = lines_starting(&out, "name=series_cell ").into_iter().filter(|line| line.contains(" primary=yes ")).collect::<Vec<_>>();
    assert_eq!(primary.len(), 1, "主格只有一格：{primary:?}");
    assert!(primary[0].contains("state=same_mount tier=one_user_unit pick=later_peak filter=steady window=common_window arm=tested "), "{}", primary[0]);
    assert!(primary[0].contains(" points=6 ") && primary[0].contains(" beta=0.000 ") && primary[0].contains(" verdict=stable_does_not_grow"), "{}", primary[0]);
    assert_eq!(out.iter().filter(|line| line.starts_with("name=series_cell ")).count(), 192, "2 状态 × 2 档 × 3 取法 × 2 剔旗 × 2 窗 × 4 臂");
    assert!(out.iter().any(|line| line == "name=verdict quantity=hundred_units_per_file_pin settles=1 differing=0 matches=true first_differing=-"), "{out:?}");
    assert!(out.iter().any(|line| line == "name=verdict quantity=hundred_units_per_file_series outcome=stable_does_not_grow series_void=- points=6 route=does_not_support_the_flip"), "{out:?}");
}

#[test]
fn the_hundred_units_per_file_series_is_void_when_the_linear_control_cannot_be_read_and_the_common_window_hides_the_extended_peak() {
    let out = series_report(false, Some(9_999_999));
    assert!(out.iter().any(|line| line.starts_with("name=verdict quantity=hundred_units_per_file_series ") && line.contains(" series_void=control_not_red ") && line.ends_with(" route=series_not_made")), "{out:?}");
    // 公共窗只看 W1–W4：高端的 W7 大操作不进公共窗的主格，进自己的窗
    let cell = |window: &str| {
        out.iter().find(|line| line.starts_with("name=series_cell state=same_mount tier=one_user_unit pick=later_peak filter=steady ") && line.contains(&format!(" window={window} arm=tested "))).expect("有这一格").clone()
    };
    assert!(cell("common_window").contains(" beta=0.000 "), "{}", cell("common_window"));
    assert!(!cell("own_window").contains(" beta=0.000 "), "自己的窗里高端有 W7 的大操作，峰与 β 要变：{}", cell("own_window"));
}

#[test]
fn a_differing_pin_or_disagreeing_sources_void_the_series_and_an_unfinished_point_drops_out() {
    let pin_differs = series_report_tweaked(true, None, |_, pin| pin.window[0].final_set_hash = "zz".to_string());
    assert!(pin_differs.iter().any(|line| line == "name=verdict quantity=hundred_units_per_file_pin settles=1 differing=1 matches=false first_differing=same_mount/W1/1/0"), "{pin_differs:?}");
    assert!(pin_differs.iter().any(|line| line.starts_with("name=verdict quantity=hundred_units_per_file_series ") && line.contains(" series_void=pin_differs ")), "{pin_differs:?}");
    let sources_differ = series_report_tweaked(true, None, |built, _| {
        built[0].0.source_sha256.insert("crates/singlefs-core/src/allocator.rs".to_string(), "11".to_string());
    });
    assert!(sources_differ.iter().any(|line| line.starts_with("name=verdict quantity=sources_agree ") && line.ends_with(" files_with_more_than_one_value=1 agree=false")), "{sources_differ:?}");
    assert!(sources_differ.iter().any(|line| line.starts_with("name=verdict quantity=hundred_units_per_file_series ") && line.contains(" series_void=sources_differ ")), "{sources_differ:?}");
    let unfinished = series_report_tweaked(true, None, |built, _| built[2].0.done = false);
    assert!(unfinished.iter().any(|line| line.starts_with("name=verdict quantity=hundred_units_per_file_series ") && line.contains(" points=5 ")), "没做完的点不进序列：{unfinished:?}");
}

// ---- 对账分辨段 reconciliation_split（修订 3）----
fn operation_key(state: &str, operation_name: &str, operation_index: u64) -> (String, String, u64) {
    (state.to_string(), operation_name.to_string(), operation_index)
}

/// 一份带镜像哈希与对账行的产物：同样的三个操作（同一次挂载 W1 第 1、2 次与重挂载 W1 第 1 次），`hash` 给每个操作的哈希、`rows` 给对账行（盘上变了, 计数记）。
fn split_product(point: &str, arm: &str, hashes: [&str; 3], rows: [(u64, u64); 3], remount: (&str, &str)) -> PointArmReadings {
    let keys = [operation_key("same_mount", "W1", 1), operation_key("same_mount", "W1", 2), operation_key("remounted", "W1", 1)];
    let mut item = PointArmReadings { done: true, ..item_named(point, arm, 1) };
    item.images_after_the_operations = keys.iter().cloned().zip(hashes.iter().map(|hash| hash.to_string())).collect();
    if arm == "head" {
        item.reconciliation_rows = keys.iter().cloned().zip(rows).collect();
    }
    item.images_at_the_remount = BTreeMap::from([("before_the_crash".to_string(), remount.0.to_string()), ("after_the_mount".to_string(), remount.1.to_string())]);
    item
}

#[test]
fn head_against_plain_names_the_first_differing_operation_and_the_remount_images() {
    let plain = split_product("files_1000", "plain", ["a", "b", "c"], [(0, 0); 3], ("image_before_the_crash", "image_after_the_mount"));
    let same = split_product("files_1000", "head", ["a", "b", "c"], [(0, 0); 3], ("image_before_the_crash", "image_after_the_mount"));
    let reading = head_against_plain(&same, &plain).expect("读得到");
    assert_eq!((reading.operations, reading.differing, reading.first_differing.as_str(), reading.before_the_crash_same, reading.after_the_mount_same), (3, 0, "-", true, true));
    let different = split_product("files_1000", "head", ["a", "b", "x"], [(0, 0); 3], ("image_before_the_crash", "other"));
    let differing_reading = head_against_plain(&different, &plain).expect("读得到");
    assert_eq!((differing_reading.differing, differing_reading.first_differing.as_str(), differing_reading.before_the_crash_same, differing_reading.after_the_mount_same), (1, "remounted/W1/1", true, false));
    let without_images = PointArmReadings { done: true, ..item_named("files_1000", "plain", 1) };
    assert_eq!(head_against_plain(&same, &without_images), None, "缺新行记读不到");
}

#[test]
fn the_placement_gap_counts_the_three_signs_and_names_the_first_undercount() {
    let item = split_product("files_1000", "head", ["a", "b", "c"], [(32, 32), (30, 32), (32, 31)], ("same_image", "same_image"));
    let reading = placement_gap(&item).expect("读得到");
    assert_eq!((reading.operations, reading.negative, reading.zero, reading.positive, reading.first_negative.as_str()), (3, 1, 1, 1, "remounted/W1/1"));
    let clean = split_product("base", "head", ["a", "b", "c"], [(32, 32), (32, 32), (4, 5)], ("same_image", "same_image"));
    let clean_reading = placement_gap(&clean).expect("读得到");
    assert_eq!((clean_reading.negative, clean_reading.positive, clean_reading.first_negative.as_str()), (0, 1, "-"), "放下去、盘上内容没变（正小）不算少记");
}

#[test]
fn the_rerun_compares_the_hash_too_and_names_the_first_differing_publish() {
    let settle_with = |hash: &str, previous_records: u64| Settle { rounds: 2, final_roles: 20, final_nodes: 7, round_work: 32, final_set_hash: hash.to_string(), previous_records, ..settle_at("same_mount", "W1", 1, 0) };
    let mut today = item_named("base", "head", 1);
    let mut before = item_named("base", "head", 1);
    today.window = vec![settle_with("aa", 10), Settle { operation_index: 2, ..settle_with("bb", 10) }];
    before.window = vec![settle_with("aa", 10), Settle { operation_index: 2, ..settle_with("bb", 10) }];
    assert_eq!(settles_against(&today, &before), (2, vec![]));
    before.window[1].final_set_hash = "bc".to_string();
    assert_eq!(settles_against(&today, &before), (2, vec!["same_mount/W1/2/0".to_string()]), "只差哈希的一对也算不同");
    before.window[1].final_set_hash = "bb".to_string();
    before.window[0].previous_records = 11;
    assert_eq!(settles_against(&today, &before).1, vec!["same_mount/W1/1/0".to_string()], "previous_records 也比");
}

fn state_row_item(point: &str, arm: &str, rows: &[(&str, u64, u64)]) -> PointArmReadings {
    let mut item = item_named(point, arm, 1);
    for (state, index, cluster_segments) in rows {
        let fields = fields_of(&format!("E207RESULT name=state allocation_records=100 cluster_segments={cluster_segments} mapped_units=10 units=12"));
        item.judgements.insert(format!("{state}/W1/{index}/"), JudgementReadings { state: fields, ..JudgementReadings::default() });
    }
    item
}

fn state_row_points<'items>(today: &'items PointArmReadings, before: &'items PointArmReadings) -> (Points<'items>, Points<'items>) {
    let operations = |state_rows: &[(&str, u64)]| -> Vec<Operation> { state_rows.iter().map(|(state, index)| operation_at(state, "W1", *index, 1, 1, 1)).collect() };
    let keys = [("same_mount", 1), ("remounted", 1), ("remounted", 2)];
    (points_with(vec![("files_1000", today, operations(&keys))]), points_with(vec![("files_1000", before, operations(&keys))]))
}

#[test]
fn the_state_rows_count_the_fields_and_the_same_mount_ones() {
    let before = state_row_item("files_1000", "p2a", &[("same_mount", 1, 2), ("remounted", 1, 2), ("remounted", 2, 2)]);
    let today = state_row_item("files_1000", "head", &[("same_mount", 1, 2), ("remounted", 1, 1), ("remounted", 2, 2)]);
    let (today_points, before_points) = state_row_points(&today, &before);
    let rows = state_rows_against_e206(&today_points["files_1000"], &before_points["files_1000"]).expect("读得到");
    assert_eq!((rows.operations, rows.differing_operations, rows.by_field, rows.same_mount_differing_operations, rows.first_differing.as_str()), (3, 1, [0, 1, 0, 0], 0, "remounted/W1/1"));
    let same_mount_differs = state_row_item("files_1000", "head", &[("same_mount", 1, 1), ("remounted", 1, 2), ("remounted", 2, 2)]);
    let (same_mount_today_points, same_mount_before_points) = state_row_points(&same_mount_differs, &before);
    assert_eq!(state_rows_against_e206(&same_mount_today_points["files_1000"], &same_mount_before_points["files_1000"]).expect("读得到").same_mount_differing_operations, 1);
    let mut without_the_field = before.clone();
    for judgement in without_the_field.judgements.values_mut() {
        judgement.state.remove("cluster_segments");
    }
    let (without_field_today_points, without_field_before_points) = state_row_points(&today, &without_the_field);
    assert_eq!(state_rows_against_e206(&without_field_today_points["files_1000"], &without_field_before_points["files_1000"]), None, "E206 的状态行缺 cluster_segments：记读不到，不拿别的字段顶替");
}

#[test]
fn the_split_outcomes_follow_the_registered_order() {
    let facts = |head_differs: bool, gap_negative: bool, rerun_differs: bool, signature_of_a_changed_tree: bool, five_items_differ: bool| {
        Some(SplitFacts { head_differs, gap_negative, rerun_differs, signature_of_a_changed_tree, five_items_differ })
    };
    assert_eq!(reconciliation_split_outcome(facts(true, false, false, false, true)), "counting_wrong");
    assert_eq!(reconciliation_split_outcome(facts(false, true, false, false, true)), "counting_wrong");
    assert_eq!(reconciliation_split_outcome(facts(false, true, false, true, true)), "both", "c_gap 且 t_sign 是 both：次序上 both 不能被 counting_wrong 吃掉");
    assert_eq!(reconciliation_split_outcome(facts(false, false, false, true, true)), "tree_changed");
    assert_eq!(reconciliation_split_outcome(facts(false, false, false, false, true)), "neither_reading_differs");
    assert_eq!(reconciliation_split_outcome(facts(false, false, true, false, true)), "neither_not_reproducible");
    assert_eq!(reconciliation_split_outcome(facts(false, false, true, true, true)), "neither_not_reproducible", "没有 c_head、c_gap 而 c_rerun：树变了的记号也压不过");
    assert_eq!(reconciliation_split_outcome(facts(false, false, false, false, false)), "nothing_left_to_split");
    assert_eq!(reconciliation_split_outcome(None), "not_run");
}

/// 对账分辨段的整段读数：base 一处不差，files_1000 的 `same_mount_state_differs` 为真时同一次挂载里也有一处状态行不同，`remount_state_differs` 为真时重挂载里有一处不同。
fn split_report(same_mount_state_differs: bool, remount_state_differs: bool) -> Lines {
    let hashes = ["a", "b", "c"];
    let build = |point: &str, arm: &str, rows: [(u64, u64); 3]| split_product(point, arm, hashes, rows, ("image_before_the_crash", "image_after_the_mount"));
    let mut items = vec![build("base", "head", [(32, 32); 3]), build("base", "plain", [(0, 0); 3]), build("files_1000", "head", [(32, 32); 3]), build("files_1000", "plain", [(0, 0); 3])];
    let window = |hash: &str| vec![Settle { rounds: 2, final_roles: 3, final_nodes: 4, round_work: 5, final_set_hash: hash.to_string(), ..settle_at("same_mount", "W1", 1, 0) }];
    for item in &mut items {
        item.window = window("h");
    }
    let previous: Vec<PointArmReadings> = items.iter().filter(|item| item.arm == "head").cloned().collect();
    let state_rows = |differs_in_same_mount: bool, differs_in_remount: bool| {
        [("same_mount", 1, if differs_in_same_mount { 1 } else { 2 }), ("remounted", 1, if differs_in_remount { 1 } else { 2 }), ("remounted", 2, 2)]
    };
    let today_state = |point: &str, same_mount: bool, remount: bool| state_row_item(point, "head", &state_rows(same_mount, remount));
    let retro_state = |point: &str| {
        let mut item = state_row_item(point, "p2a", &state_rows(false, false));
        item.window = window("h");
        item
    };
    for item in &mut items {
        if item.arm == "head" {
            let (same_mount, remount) = if item.point == "files_1000" { (same_mount_state_differs, remount_state_differs) } else { (false, false) };
            item.judgements = today_state(&item.point, same_mount, remount).judgements;
        }
    }
    let retro_items = [retro_state("base"), retro_state("files_1000")];
    let operations = |_item: &PointArmReadings| -> Vec<Operation> { [("same_mount", 1), ("remounted", 1), ("remounted", 2)].iter().map(|(state, index)| operation_at(state, "W1", *index, 1, 1, 1)).collect() };
    let points = points_with(items.iter().map(|item| (if item.arm == "plain" { if item.point == "base" { "base__plain" } else { "files_1000__plain" } } else { item.point.as_str() }, item, operations(item))).collect());
    let previous_points = points_with(previous.iter().map(|item| (item.point.as_str(), item, operations(item))).collect());
    let retro_points = points_with(retro_items.iter().map(|item| (item.point.as_str(), item, operations(item))).collect());
    let mut out = Lines::new();
    report_reconciliation_split(&points, &previous_points, &retro_points, &mut out);
    out
}

fn split_outcome_line(out: &[String], point: &str) -> String {
    out.iter().find(|line| line.starts_with(&format!("name=verdict quantity=reconciliation_split point={point} "))).expect("有总判行").clone()
}

#[test]
fn the_split_report_reads_a_changed_tree_only_when_the_same_mount_stays_clean() {
    let out = split_report(false, true);
    assert!(split_outcome_line(&out, "base").contains(" outcome=negative_control_clean reason=-"), "{out:?}");
    assert!(split_outcome_line(&out, "files_1000").contains(" outcome=tree_changed "), "{}", split_outcome_line(&out, "files_1000"));
    assert!(out.iter().any(|line| line.contains("quantity=state_rows_against_e206 point=files_1000 operations=3 differing_operations=1 ")), "{out:?}");
    let with_the_control_differing = split_report(true, true);
    assert!(
        split_outcome_line(&with_the_control_differing, "files_1000").contains(" outcome=nothing_left_to_split "),
        "同一次挂载里状态行也不同：不是「只在重挂载之后」的树变了记号：{}",
        split_outcome_line(&with_the_control_differing, "files_1000")
    );
}
