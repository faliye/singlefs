use super::*;

fn called(rows: &[(&str, u64)]) -> BTreeMap<String, u64> {
    rows.iter()
        .map(|(row, calls)| ((*row).to_string(), *calls))
        .collect()
}

/// 第七节 ② 第 3 条那两份 E203 产物（`git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-{base,files_10000}-2026-11-07.out`）同一次挂载覆盖写第 1、2 轮喂给 B8 的输入，caller 段全 0。
fn e203_anchor_inputs() -> Vec<(u64, [ByDeltaInput; 2])> {
    vec![
        (
            2102,
            [
                ByDeltaInput {
                    inode_leaf_containers: 1,
                    other_files: 11,
                    touched_files: 0,
                    lower_nodes_of_touched_files: 0,
                    extent_upper_levels: 1,
                    extent_upper_nodes: 1,
                    accounting_rows_of_the_generation: 15,
                    accounting_tree_levels: 1,
                    settling_rounds: 0,
                    changed_leaves_per_round: vec![],
                    allocation_tree_levels: 3,
                    allocation_records: 922,
                    segments_per_disk: 2048,
                    mapped_units: 35,
                    deadlist_entries: 0,
                    calls: called(&[
                        ("files_other_than_the_first_plan", 1),
                        ("inodes_with_an_extent_entry_set", 1),
                        ("extent_upper_segment_plan", 1),
                        ("extent_roles_rewritten_scan", 1),
                        ("root_header_linear_find", 3),
                        ("checkpoint_cost_accounting_tree_plan", 2),
                        ("cluster_segment_sum", 2),
                        ("inode_number_watermark_steps", 1),
                    ]),
                    intersecting_records: 0,
                    dirty_per_call: vec![],
                    code_two_trees_over_all_rounds: vec![],
                    container_rewrites_over_all_rounds: vec![],
                    moved_entries: 0,
                    central_mapping_levels: 1,
                    deadlist_query_levels: 0,
                    deadlist_changed_entries: 0,
                    release_query_levels: 0,
                    changed_slots_and_records_per_round: vec![],
                    mapped_units_changed_over_all_rounds: 0,
                },
                ByDeltaInput {
                    inode_leaf_containers: 1,
                    other_files: 11,
                    touched_files: 0,
                    lower_nodes_of_touched_files: 0,
                    extent_upper_levels: 1,
                    extent_upper_nodes: 1,
                    accounting_rows_of_the_generation: 15,
                    accounting_tree_levels: 1,
                    settling_rounds: 2,
                    changed_leaves_per_round: vec![4, 4],
                    allocation_tree_levels: 3,
                    allocation_records: 922,
                    segments_per_disk: 2048,
                    mapped_units: 35,
                    deadlist_entries: 0,
                    calls: called(&[
                        ("files_other_than_the_first_plan", 2),
                        ("inodes_with_an_extent_entry_set", 2),
                        ("extent_upper_segment_plan", 2),
                        ("extent_roles_rewritten_scan", 2),
                        ("root_header_linear_find", 3),
                        ("checkpoint_cost_accounting_tree_plan", 2),
                        ("inode_number_watermark_steps", 3),
                        ("allocator_release_linear_find", 84),
                        ("user_data_slot_linear_scan", 2),
                        ("record_intersection_filter", 28),
                        ("accounting_entries_of_this_publish", 2),
                    ]),
                    intersecting_records: 2,
                    dirty_per_call: vec![28, 56, 86],
                    code_two_trees_over_all_rounds: vec![
                        (30, 1),
                        (0, 0),
                        (10, 1),
                        (30, 1),
                        (0, 0),
                        (24, 1),
                    ],
                    container_rewrites_over_all_rounds: vec![0, 0, 0, 0, 0, 0],
                    moved_entries: 0,
                    central_mapping_levels: 1,
                    deadlist_query_levels: 48,
                    deadlist_changed_entries: 0,
                    release_query_levels: 114,
                    changed_slots_and_records_per_round: vec![14, 28],
                    mapped_units_changed_over_all_rounds: 42,
                },
            ],
        ),
        (
            2098,
            [
                ByDeltaInput {
                    inode_leaf_containers: 1,
                    other_files: 11,
                    touched_files: 0,
                    lower_nodes_of_touched_files: 0,
                    extent_upper_levels: 1,
                    extent_upper_nodes: 1,
                    accounting_rows_of_the_generation: 15,
                    accounting_tree_levels: 1,
                    settling_rounds: 0,
                    changed_leaves_per_round: vec![],
                    allocation_tree_levels: 3,
                    allocation_records: 948,
                    segments_per_disk: 2048,
                    mapped_units: 35,
                    deadlist_entries: 0,
                    calls: called(&[
                        ("files_other_than_the_first_plan", 1),
                        ("inodes_with_an_extent_entry_set", 1),
                        ("extent_upper_segment_plan", 1),
                        ("extent_roles_rewritten_scan", 1),
                        ("root_header_linear_find", 3),
                        ("checkpoint_cost_accounting_tree_plan", 2),
                        ("cluster_segment_sum", 2),
                        ("inode_number_watermark_steps", 1),
                    ]),
                    intersecting_records: 0,
                    dirty_per_call: vec![],
                    code_two_trees_over_all_rounds: vec![],
                    container_rewrites_over_all_rounds: vec![],
                    moved_entries: 0,
                    central_mapping_levels: 1,
                    deadlist_query_levels: 0,
                    deadlist_changed_entries: 0,
                    release_query_levels: 0,
                    changed_slots_and_records_per_round: vec![],
                    mapped_units_changed_over_all_rounds: 0,
                },
                ByDeltaInput {
                    inode_leaf_containers: 1,
                    other_files: 11,
                    touched_files: 0,
                    lower_nodes_of_touched_files: 0,
                    extent_upper_levels: 1,
                    extent_upper_nodes: 1,
                    accounting_rows_of_the_generation: 15,
                    accounting_tree_levels: 1,
                    settling_rounds: 2,
                    changed_leaves_per_round: vec![4, 4],
                    allocation_tree_levels: 3,
                    allocation_records: 948,
                    segments_per_disk: 2048,
                    mapped_units: 35,
                    deadlist_entries: 0,
                    calls: called(&[
                        ("files_other_than_the_first_plan", 2),
                        ("inodes_with_an_extent_entry_set", 2),
                        ("extent_upper_segment_plan", 2),
                        ("extent_roles_rewritten_scan", 2),
                        ("root_header_linear_find", 3),
                        ("checkpoint_cost_accounting_tree_plan", 2),
                        ("inode_number_watermark_steps", 3),
                        ("allocator_release_linear_find", 84),
                        ("user_data_slot_linear_scan", 2),
                        ("record_intersection_filter", 28),
                        ("accounting_entries_of_this_publish", 2),
                    ]),
                    intersecting_records: 0,
                    dirty_per_call: vec![28, 56, 84],
                    code_two_trees_over_all_rounds: vec![
                        (30, 1),
                        (0, 0),
                        (10, 1),
                        (30, 1),
                        (0, 0),
                        (24, 1),
                    ],
                    container_rewrites_over_all_rounds: vec![0, 0, 0, 0, 0, 0],
                    moved_entries: 0,
                    central_mapping_levels: 1,
                    deadlist_query_levels: 48,
                    deadlist_changed_entries: 0,
                    release_query_levels: 114,
                    changed_slots_and_records_per_round: vec![14, 28],
                    mapped_units_changed_over_all_rounds: 42,
                },
            ],
        ),
        (
            152344,
            [
                ByDeltaInput {
                    inode_leaf_containers: 43,
                    other_files: 10001,
                    touched_files: 0,
                    lower_nodes_of_touched_files: 0,
                    extent_upper_levels: 2,
                    extent_upper_nodes: 71,
                    accounting_rows_of_the_generation: 15,
                    accounting_tree_levels: 1,
                    settling_rounds: 0,
                    changed_leaves_per_round: vec![],
                    allocation_tree_levels: 3,
                    allocation_records: 47624,
                    segments_per_disk: 2048,
                    mapped_units: 10221,
                    deadlist_entries: 0,
                    calls: called(&[
                        ("files_other_than_the_first_plan", 1),
                        ("inodes_with_an_extent_entry_set", 1),
                        ("extent_upper_segment_plan", 1),
                        ("extent_roles_rewritten_scan", 1),
                        ("root_header_linear_find", 3),
                        ("checkpoint_cost_accounting_tree_plan", 2),
                        ("cluster_segment_sum", 2),
                        ("inode_number_watermark_steps", 1),
                    ]),
                    intersecting_records: 0,
                    dirty_per_call: vec![],
                    code_two_trees_over_all_rounds: vec![],
                    container_rewrites_over_all_rounds: vec![],
                    moved_entries: 0,
                    central_mapping_levels: 2,
                    deadlist_query_levels: 0,
                    deadlist_changed_entries: 0,
                    release_query_levels: 0,
                    changed_slots_and_records_per_round: vec![],
                    mapped_units_changed_over_all_rounds: 0,
                },
                ByDeltaInput {
                    inode_leaf_containers: 43,
                    other_files: 10001,
                    touched_files: 0,
                    lower_nodes_of_touched_files: 0,
                    extent_upper_levels: 2,
                    extent_upper_nodes: 71,
                    accounting_rows_of_the_generation: 15,
                    accounting_tree_levels: 1,
                    settling_rounds: 22,
                    changed_leaves_per_round: vec![
                        10, 14, 18, 22, 26, 28, 30, 32, 36, 40, 44, 48, 50, 52, 56, 60, 62, 64,
                        66, 68, 70, 70,
                    ],
                    allocation_tree_levels: 3,
                    allocation_records: 47624,
                    segments_per_disk: 2048,
                    mapped_units: 10221,
                    deadlist_entries: 0,
                    calls: called(&[
                        ("files_other_than_the_first_plan", 22),
                        ("inodes_with_an_extent_entry_set", 22),
                        ("extent_upper_segment_plan", 22),
                        ("extent_roles_rewritten_scan", 22),
                        ("root_header_linear_find", 3),
                        ("checkpoint_cost_accounting_tree_plan", 2),
                        ("inode_number_watermark_steps", 23),
                        ("allocator_release_linear_find", 4804),
                        ("user_data_slot_linear_scan", 2),
                        ("record_intersection_filter", 168),
                        ("empty_segment_linear_scan", 4),
                        ("accounting_entries_of_this_publish", 22),
                    ]),
                    intersecting_records: 152,
                    dirty_per_call: vec![
                        44, 96, 112, 128, 144, 160, 168, 176, 184, 200, 216, 232, 248, 256,
                        264, 280, 296, 304, 312, 320, 328, 336, 656,
                    ],
                    code_two_trees_over_all_rounds: vec![
                        (30, 1),
                        (0, 0),
                        (12, 2),
                        (30, 1),
                        (0, 0),
                        (38, 2),
                        (30, 1),
                        (0, 0),
                        (46, 2),
                        (30, 1),
                        (0, 0),
                        (54, 2),
                        (30, 1),
                        (0, 0),
                        (62, 2),
                        (30, 1),
                        (0, 0),
                        (70, 2),
                        (30, 1),
                        (0, 0),
                        (74, 2),
                        (30, 1),
                        (0, 0),
                        (78, 2),
                        (30, 1),
                        (0, 0),
                        (82, 2),
                        (30, 1),
                        (0, 0),
                        (90, 2),
                        (30, 1),
                        (0, 0),
                        (98, 2),
                        (30, 1),
                        (0, 0),
                        (106, 2),
                        (30, 1),
                        (0, 0),
                        (114, 2),
                        (30, 1),
                        (0, 0),
                        (118, 2),
                        (30, 1),
                        (0, 0),
                        (122, 2),
                        (30, 1),
                        (0, 0),
                        (130, 2),
                        (30, 1),
                        (0, 0),
                        (138, 2),
                        (30, 1),
                        (0, 0),
                        (142, 2),
                        (30, 1),
                        (0, 0),
                        (146, 2),
                        (30, 1),
                        (0, 0),
                        (150, 2),
                        (30, 1),
                        (0, 0),
                        (154, 2),
                        (30, 1),
                        (0, 0),
                        (158, 2),
                    ],
                    container_rewrites_over_all_rounds: vec![
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    ],
                    moved_entries: 0,
                    central_mapping_levels: 2,
                    deadlist_query_levels: 1540,
                    deadlist_changed_entries: 0,
                    release_query_levels: 15890,
                    changed_slots_and_records_per_round: vec![
                        22, 48, 56, 64, 72, 80, 84, 88, 92, 100, 108, 116, 124, 128, 132, 140,
                        148, 152, 156, 160, 164, 168,
                    ],
                    mapped_units_changed_over_all_rounds: 2402,
                },
            ],
        ),
        (
            4226,
            [
                ByDeltaInput {
                    inode_leaf_containers: 43,
                    other_files: 10001,
                    touched_files: 0,
                    lower_nodes_of_touched_files: 0,
                    extent_upper_levels: 2,
                    extent_upper_nodes: 71,
                    accounting_rows_of_the_generation: 15,
                    accounting_tree_levels: 1,
                    settling_rounds: 0,
                    changed_leaves_per_round: vec![],
                    allocation_tree_levels: 3,
                    allocation_records: 47640,
                    segments_per_disk: 2048,
                    mapped_units: 10221,
                    deadlist_entries: 0,
                    calls: called(&[
                        ("files_other_than_the_first_plan", 1),
                        ("inodes_with_an_extent_entry_set", 1),
                        ("extent_upper_segment_plan", 1),
                        ("extent_roles_rewritten_scan", 1),
                        ("root_header_linear_find", 3),
                        ("checkpoint_cost_accounting_tree_plan", 2),
                        ("cluster_segment_sum", 2),
                        ("inode_number_watermark_steps", 1),
                    ]),
                    intersecting_records: 0,
                    dirty_per_call: vec![],
                    code_two_trees_over_all_rounds: vec![],
                    container_rewrites_over_all_rounds: vec![],
                    moved_entries: 0,
                    central_mapping_levels: 2,
                    deadlist_query_levels: 0,
                    deadlist_changed_entries: 0,
                    release_query_levels: 0,
                    changed_slots_and_records_per_round: vec![],
                    mapped_units_changed_over_all_rounds: 0,
                },
                ByDeltaInput {
                    inode_leaf_containers: 43,
                    other_files: 10001,
                    touched_files: 0,
                    lower_nodes_of_touched_files: 0,
                    extent_upper_levels: 2,
                    extent_upper_nodes: 71,
                    accounting_rows_of_the_generation: 15,
                    accounting_tree_levels: 1,
                    settling_rounds: 2,
                    changed_leaves_per_round: vec![4, 4],
                    allocation_tree_levels: 3,
                    allocation_records: 47640,
                    segments_per_disk: 2048,
                    mapped_units: 10221,
                    deadlist_entries: 0,
                    calls: called(&[
                        ("files_other_than_the_first_plan", 2),
                        ("inodes_with_an_extent_entry_set", 2),
                        ("extent_upper_segment_plan", 2),
                        ("extent_roles_rewritten_scan", 2),
                        ("root_header_linear_find", 3),
                        ("checkpoint_cost_accounting_tree_plan", 2),
                        ("inode_number_watermark_steps", 3),
                        ("allocator_release_linear_find", 108),
                        ("user_data_slot_linear_scan", 2),
                        ("record_intersection_filter", 34),
                        ("accounting_entries_of_this_publish", 2),
                    ]),
                    intersecting_records: 32,
                    dirty_per_call: vec![40, 68, 134],
                    code_two_trees_over_all_rounds: vec![
                        (30, 1),
                        (0, 0),
                        (12, 2),
                        (30, 1),
                        (0, 0),
                        (26, 2),
                    ],
                    container_rewrites_over_all_rounds: vec![0, 0, 0, 0, 0, 0],
                    moved_entries: 0,
                    central_mapping_levels: 2,
                    deadlist_query_levels: 140,
                    deadlist_changed_entries: 0,
                    release_query_levels: 322,
                    changed_slots_and_records_per_round: vec![20, 34],
                    mapped_units_changed_over_all_rounds: 54,
                },
            ],
        ),
    ]
}

fn rounded(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

#[test]
fn chain_of_one_settles_in_two_rounds_on_both_arms() {
    for arm in [ModelArm::Today, ModelArm::Closure] {
        let outcome = chain_outcome(1, 3, arm);
        assert_eq!(outcome.rounds, Some(2));
        assert_eq!(outcome.round_work(), 7);
        assert_eq!(outcome.per_round, vec![3, 4]);
    }
    assert_eq!(chain_outcome(1, 3, ModelArm::Today).sum_of_increments(), 4);
}

#[test]
fn chain_of_five_takes_six_rounds_today_and_two_with_the_closure() {
    let today = chain_outcome(5, 3, ModelArm::Today);
    assert_eq!(today.rounds, Some(6));
    assert_eq!(today.round_work(), 33);
    assert_eq!(today.per_round, vec![3, 4, 5, 6, 7, 8]);
    assert_eq!(today.sum_of_increments(), 8);
    let closure = chain_outcome(5, 3, ModelArm::Closure);
    assert_eq!(closure.rounds, Some(2));
    assert_eq!(closure.round_work(), 11);
}

#[test]
fn chain_of_twenty_takes_twenty_one_rounds_today_and_two_with_the_closure() {
    let today = chain_outcome(20, 3, ModelArm::Today);
    assert_eq!(today.rounds, Some(21));
    assert_eq!(today.round_work(), 273);
    assert_eq!(today.sum_of_increments(), 23);
    assert_eq!(rounded(today.effective_rounds()), rounded(273.0 / 23.0));
    let closure = chain_outcome(20, 3, ModelArm::Closure);
    assert_eq!(closure.rounds, Some(2));
    assert_eq!(closure.round_work(), 26);
    assert_eq!(closure.per_round, vec![3, 23]);
}

#[test]
fn chain_of_seventy_runs_out_of_rounds_today_and_settles_with_the_closure() {
    let today = chain_outcome(70, 3, ModelArm::Today);
    assert_eq!(today.rounds, None);
    assert_eq!(today.per_round.len(), 64);
    assert_eq!(today.round_work(), 2208);
    let closure = chain_outcome(70, 3, ModelArm::Closure);
    assert_eq!(closure.rounds, Some(2));
    assert_eq!(closure.round_work(), 76);
    assert_eq!(closure.per_round, vec![3, 73]);
}

#[test]
fn crossing_leaves_keep_one_more_round_with_the_closure() {
    let today = crossing_outcome(ModelArm::Today);
    assert_eq!(today.rounds, Some(6));
    assert_eq!(today.round_work(), 27);
    assert_eq!(today.per_round, vec![2, 3, 4, 5, 6, 7]);
    let closure = crossing_outcome(ModelArm::Closure);
    assert_eq!(closure.rounds, Some(3));
    assert_eq!(closure.round_work(), 14);
    assert_eq!(closure.per_round, vec![2, 5, 7]);
}

#[test]
fn geometry_table_matches_the_registered_anchors() {
    let expected = [
        (14, 22, 45),
        (17, 164, 331),
        (18, 325, 655),
        (19, 648, 1305),
        (20, 1294, 2605),
        (24, 20664, 41577),
    ];
    for (exponent, leaves, nodes) in expected {
        let (got_leaves, _, got_nodes) = tree_geometry(1_u64 << exponent);
        assert_eq!((got_leaves, got_nodes), (leaves, nodes), "U = 2^{exponent}");
    }
}

#[test]
fn local_constants_match_the_registered_values() {
    assert_eq!(LEAF_SLOTS, 812);
    assert_eq!(INTERNAL_FANOUT, 169);
    assert_eq!(SETTLING_ROUNDS_LIMIT, 64);
    assert_eq!(DISKS, 2);
}

#[test]
fn growth_orders_match_the_registered_anchors() {
    assert_eq!(rounded(alpha_on_the_axis(2.0, 22.0)), 0.347);
    assert_eq!(rounded(alpha_on_the_axis(2.0, 64.0)), 0.502);
    assert_eq!(rounded(alpha_on_the_axis(21.0, 1201.0)), 0.586);
    assert_eq!(rounded(alpha_on_the_axis(14.0, 84.0)), 0.259);
}

#[test]
fn bands_split_at_a_quarter_and_a_half() {
    assert_eq!(band(0.25), Band::DoesNotGrow);
    assert_eq!(band(0.26), Band::Between);
    assert_eq!(band(0.5), Band::Grows);
    assert_eq!(band(alpha_on_the_axis(21.0, 200.0)), Band::Between);
    assert_eq!(band(alpha_on_the_axis(21.0, 118.0)), Band::DoesNotGrow);
    assert_eq!(band(alpha_on_the_axis(21.0, 664.1)), Band::Grows);
}

#[test]
fn rounds_verdict_has_three_levels() {
    assert_eq!(rounds_verdict(8, false), "not_many");
    assert_eq!(rounds_verdict(9, false), "many");
    assert_eq!(rounds_verdict(63, false), "many");
    assert_eq!(rounds_verdict(64, false), "hits_the_limit");
    assert_eq!(rounds_verdict(5, true), "hits_the_limit");
}

/// 第八节判别力自证：门槛落在两点之间，判定翻面。
#[test]
fn the_verdicts_flip_between_the_chain_of_five_and_the_chain_of_twenty() {
    let (short, long) = (
        chain_outcome(5, 3, ModelArm::Today),
        chain_outcome(20, 3, ModelArm::Today),
    );
    assert_eq!(
        rounds_verdict(short.rounds.expect("定下"), false),
        "not_many"
    );
    assert_eq!(rounds_verdict(long.rounds.expect("定下"), false), "many");
    assert_eq!(
        rounds_held_down_verdict(
            short.rounds.expect("定下"),
            short.effective_rounds().min(4.0)
        ),
        "held_down"
    );
    assert_eq!(
        rounds_held_down_verdict(long.rounds.expect("定下"), long.effective_rounds()),
        "not_held_down"
    );
    assert_eq!(
        band(alpha_on_the_axis(
            short.round_work() as f64,
            long.round_work() as f64
        )),
        Band::Between
    );
    assert_eq!(
        band(alpha_on_the_axis(short.round_work() as f64, 1000.0)),
        Band::Between
    );
    assert_eq!(
        band(alpha_on_the_axis(
            short.round_work() as f64,
            short.round_work() as f64 * 2.0
        )),
        Band::DoesNotGrow
    );
}

#[test]
fn the_many_rounds_threshold_is_not_the_arm_definition() {
    assert_eq!(rounds_held_down_verdict(8, 4.0), "held_down");
    assert_eq!(rounds_held_down_verdict(9, 1.0), "not_held_down");
    assert_eq!(rounds_held_down_verdict(2, 4.01), "not_held_down");
}

#[test]
fn local_by_delta_rows_reproduce_the_four_e203_totals() {
    for (expected, [landing, between]) in e203_anchor_inputs() {
        assert_eq!(
            by_delta_total(&landing) + by_delta_total(&between),
            expected
        );
    }
}

#[test]
fn between_rounds_pricing_on_the_cumulative_inputs_equals_the_by_delta_rows() {
    for (_, [_, between]) in e203_anchor_inputs() {
        let steps = between_rounds_pricing_round_steps(&between_rounds_pricing_cumulative_input(&between), between_rounds_unit_prices(&between));
        assert_eq!(steps, by_delta_rows_that_between_rounds_pricing_changes(&between));
    }
}

fn fabricated_round(exit: &str, roles: u64, records: u64) -> Round {
    Round {
        round: 1,
        roles_rewritten: roles,
        exit: exit.to_string(),
        roles_changed: roles,
        records_changed: records,
        ..Round::default()
    }
}

#[test]
fn the_premise_ratio_counts_the_release_rounds_too() {
    let settle = Settle {
        round_work: 10,
        detail: vec![
            fabricated_round("release_not_covered", 4, 4),
            fabricated_round("release_not_covered", 3, 2),
            fabricated_round("settled", 3, 2),
        ],
        ..Settle::default()
    };
    assert_eq!(rounded(premise_ratio(&settle)), 0.8);
}

#[test]
fn between_rounds_pricing_reading_sums_the_rounds_and_keeps_the_largest() {
    let prices = ByDeltaInput {
        allocation_tree_levels: 2,
        allocation_records: 7,
        mapped_units: 3,
        segments_per_disk: 64,
        code_two_trees_over_all_rounds: vec![(1, 2), (1, 3), (1, 4)],
        ..ByDeltaInput::default()
    };
    let settle = Settle {
        detail: vec![
            Round {
                tree_keys_changed: vec![1, 0, 0],
                rewrite_set_size: 2,
                ..fabricated_round("release_not_covered", 2, 1)
            },
            Round {
                tree_keys_changed: vec![0, 0, 2],
                rewrite_set_size: 5,
                ..fabricated_round("settled", 3, 2)
            },
        ],
        ..Settle::default()
    };
    // 第 1 轮：树 2 × 1；mapped 4 + 4 × 2；view 1 + find 3 + 交集 3 + dirty 1 = 22
    // 第 2 轮：树 4 × 2；mapped 6 + 6 × 2；view 2 + find 6 + 交集 6 + dirty 2 = 42
    let reading = between_rounds_pricing_reading(&settle, &prices, 100);
    assert_eq!(reading.sum_of_rounds, 22 + 42);
    assert_eq!(reading.largest_round, 42);
    assert_eq!(reading.memory_bytes, 3 * 108 + 5 * 24 + 3 * 24);
    assert_eq!(reading.maintenance_steps, 3 * 3 + 5 * 2);
}

#[test]
fn fallback_and_recurrence_read_the_sequence() {
    assert_eq!(fallback_and_recurrence(&[2, 22, 2, 2]), (true, false));
    assert_eq!(
        fallback_and_recurrence(&[2, 22, 9, 9, 9, 9, 9, 9, 9, 9, 9]),
        (false, false)
    );
    assert_eq!(
        fallback_and_recurrence(&[22, 2, 2, 2, 2, 2, 2, 2, 2, 9]),
        (true, true)
    );
}

#[test]
fn chain_explained_ratio_only_reads_the_long_fixed_points() {
    let settle = |rounds, release_chain_depth| Settle {
        rounds,
        release_chain_depth,
        ..Settle::default()
    };
    assert_eq!(chain_explained_ratio(&[settle(2, 0)]), None);
    assert_eq!(
        chain_explained_ratio(&[settle(12, 10), settle(12, 7)]),
        Some(0.5)
    );
}

fn sequence_settle(hash: &str, rounds: u64, final_roles: u64, final_nodes: u64) -> Settle {
    Settle {
        rounds,
        final_roles,
        final_nodes,
        round_work: rounds * final_roles,
        final_set_hash: hash.to_string(),
        outcome: "settled".to_string(),
        ..Settle::default()
    }
}

#[test]
fn same_final_sets_report_the_first_difference() {
    let (one, two, other) = (
        sequence_settle("a", 2, 3, 1),
        sequence_settle("b", 2, 3, 1),
        sequence_settle("c", 2, 3, 1),
    );
    assert_eq!(same_final_sets(&[&one, &two], &[&one, &two]), (2, 2, None));
    assert_eq!(
        same_final_sets(&[&one, &two], &[&one, &other]),
        (1, 2, Some(1))
    );
    assert_eq!(same_final_sets(&[&one], &[&one, &other]), (1, 2, Some(1)));
}

#[test]
fn same_readings_needs_every_field_equal() {
    let today = sequence_settle("a", 2, 3, 1);
    let more_rounds = sequence_settle("a", 3, 3, 1);
    assert!(same_readings(&[&today], &[&today]));
    assert!(!same_readings(&[&today], &[&more_rounds]));
    assert!(!same_readings(&[&today], &[&today, &today]));
}

#[test]
fn single_step_control_needs_the_same_set_enough_rounds_and_more_rounds() {
    let today = sequence_settle("a", 2, 5, 2);
    let single = sequence_settle("a", 4, 5, 2);
    let window_today = vec![today.clone()];
    let window_single = vec![single.clone()];
    assert!(single_step_control_holds(
        &[&single],
        &[&today],
        &window_single,
        &window_today
    ));
    assert!(
        !single_step_control_holds(
            &[&sequence_settle("b", 4, 5, 2)],
            &[&today],
            &window_single,
            &window_today
        ),
        "不动点变了"
    );
    assert!(
        !single_step_control_holds(
            &[&sequence_settle("a", 1, 5, 2)],
            &[&today],
            &window_single,
            &window_today
        ),
        "轮数少于节点数"
    );
    assert!(
        !single_step_control_holds(&[&single], &[&today], &window_today, &window_today),
        "量窗一次都没多轮"
    );
}

fn lines_of(text: &str) -> PointArmReadings {
    let mut item = PointArmReadings::default();
    for line in text.lines() {
        take_line(&mut item, &fields_of(line));
    }
    item
}

const COMPLETE_PRODUCT: &str = "E206RESULT name=point item=base__p1 point=base arm=p1 unit_area_slots=131072 unlinks=10 data_units=10 files=10\n\
E206RESULT name=constants item=base__p1 leaf_slots=812 internal_fanout=169 settling_rounds_at_most=64 disks=2\n\
E206RESULT name=history_histogram item=base__p1 bucket=0 calls=2 rounds_one=0 rounds_two=1 rounds_three_or_more=1 sequence=2:7:2:21:00000000000000aa:40:s;3:7:2:21:00000000000000ab:40:s\n\
E206RESULT name=history_settle item=base__p1 phase=history rounds=3\n\
E206RESULT name=history item=base__p1 settle_calls=2\n\
E206RESULT name=settle item=base__p1 phase=window state=same_mount op=W1 op_index=1 rounds=2 outcome=settled final_roles=14 final_nodes=4 round_work=21 final_set_hash=00000000000000ac judgement=0\n\
E206RESULT name=round item=base__p1 state=same_mount op=W1 op_index=1 round=1 roles_rewritten=7 nodes_rewritten=0 round_exit=release_not_covered\n\
E206RESULT name=round_row item=base__p1 round=1 row=records_of_each_leaf calls=2 steps=30 query_levels=1\n\
E206RESULT name=round item=base__p1 state=same_mount op=W1 op_index=1 round=2 roles_rewritten=14 nodes_rewritten=4 round_exit=settled\n\
E206RESULT name=item_done item=base__p1\n";

#[test]
fn complete_product_reads_back_into_every_group() {
    let item = lines_of(COMPLETE_PRODUCT);
    assert_eq!(item_is_complete(&item), Ok(()));
    assert_eq!(item.history.len(), 2);
    assert_eq!(item.history[1].rounds, 3);
    assert_eq!(item.history[1].final_set_hash, "00000000000000ab");
    assert_eq!(item.window.len(), 1);
    assert_eq!(item.window[0].publish_key(), "same_mount/W1/1/0");
    assert_eq!(
        item.window[0].detail[0].rows["records_of_each_leaf"],
        (2, 30, 1)
    );
    assert_eq!(item.window[0].fixed_point_steps(), 30);
    assert_eq!(peaks_of(all_settles(&item)).rounds, 3);
    assert!(model_mismatches(&item).is_empty());
}

#[test]
fn product_without_item_done_is_void_and_not_zero() {
    let item =
        lines_of(&COMPLETE_PRODUCT.replace("E206RESULT name=item_done item=base__p1\n", ""));
    assert_eq!(item_is_complete(&item), Err("no_item_done".to_string()));
}

#[test]
fn histogram_short_of_the_fixed_point_calls_is_void() {
    let item = lines_of(&COMPLETE_PRODUCT.replace("settle_calls=2", "settle_calls=3"));
    assert!(item_is_complete(&item).is_err());
}

#[test]
fn single_history_line_missing_is_void() {
    let item = lines_of(&COMPLETE_PRODUCT.replace(
        "E206RESULT name=history_settle item=base__p1 phase=history rounds=3\n",
        "",
    ));
    assert!(item_is_complete(&item).is_err());
}

#[test]
fn round_missing_from_the_window_is_void() {
    let item = lines_of(
        &COMPLETE_PRODUCT
            .replace(
                "round=2 roles_rewritten=14 nodes_rewritten=4 round_exit=settled",
                "round=2",
            )
            .replace("rounds=2 outcome", "rounds=3 outcome"),
    );
    assert!(item_is_complete(&item).is_err());
}

#[test]
fn round_work_off_the_sum_of_the_rounds_stops_the_run() {
    let item = lines_of(
        &COMPLETE_PRODUCT.replace("round_work=21 final_set", "round_work=22 final_set"),
    );
    assert_eq!(
        model_mismatches(&item),
        vec!["round_work=22_sum=21".to_string()]
    );
}

#[test]
fn constants_off_the_local_ones_stop_the_run() {
    let item = lines_of(&COMPLETE_PRODUCT.replace("leaf_slots=812", "leaf_slots=813"));
    assert_eq!(model_mismatches(&item).len(), 1);
}

#[test]
fn refused_history_keeps_its_cause_and_the_did_not_settle_outcome_counts_as_the_limit() {
    let item = lines_of(
        "E206RESULT name=point item=x__p1 point=x arm=p1\n\
E206RESULT name=history_histogram item=x__p1 bucket=0 calls=1 sequence=64:0:0:2208:0000000000000000:0:d\n\
E206RESULT name=history_refused item=x__p1 cause=AllocationRecordTreeRewriteSetDidNotSettle\n",
    );
    assert_eq!(
        item_is_complete(&item),
        Err("refused:AllocationRecordTreeRewriteSetDidNotSettle".to_string())
    );
    let peaks = peaks_of(all_settles(&item));
    assert_eq!(
        rounds_verdict(peaks.rounds, peaks.did_not_settle),
        "hits_the_limit"
    );
}

#[test]
fn write_amplification_splits_at_one_and_a_quarter() {
    assert_eq!(write_amplification_verdict(1.25), "acceptable");
    assert_eq!(write_amplification_verdict(1.26), "expensive");
}

#[test]
fn net_steps_add_the_closure_to_the_arm_and_take_off_the_base() {
    let with_steps = |steps: u64| Settle {
        detail: vec![Round {
            rows: BTreeMap::from([("records_of_each_leaf".to_string(), (1, steps, 0))]),
            ..Round::default()
        }],
        ..Settle::default()
    };
    let arm = Settle {
        closure_lookup_steps: 7,
        closure_hops: 3,
        previous_nodes: 15,
        ..with_steps(100)
    };
    let base = with_steps(120);
    assert_eq!(net_steps_linear(&arm, &base), 100 + 7 - 120);
    assert_eq!(net_steps_indexed(&arm, &base), 100 + 3 * 4 - 120);
}

#[test]
fn negative_control_needs_flat_roles_and_few_rounds() {
    let settle = |final_roles, rounds| Settle {
        final_roles,
        rounds,
        ..Settle::default()
    };
    assert!(negative_control_holds(&settle(14, 2), &settle(14, 4)));
    assert!(!negative_control_holds(&settle(14, 2), &settle(28, 2)));
    assert!(!negative_control_holds(&settle(14, 2), &settle(14, 5)));
}

#[test]
fn the_pooled_fallback_adds_no_by_delta_steps_and_is_counted_by_the_judgements_that_call_it() {
    let judgement = |calls: u64, steps: u64| {
        let mut judgement = JudgementReadings {
            fixed_point_rounds: Some(0),
            ..JudgementReadings::default()
        };
        judgement.rows.insert(("between".to_string(), ROW_WITHOUT_A_BY_DELTA_FORMULA.to_string()), (calls, 0));
        judgement.row_steps.insert(("between".to_string(), ROW_WITHOUT_A_BY_DELTA_FORMULA.to_string()), steps);
        judgement.row_steps.insert(("caller".to_string(), "fallback_slot_linear_scan".to_string()), 4);
        judgement
    };
    assert_eq!(pooled_fallback_of_the_judgement(&judgement(0, 0)), (0, 0, 4));
    assert_eq!(pooled_fallback_of_the_judgement(&judgement(3, 500)), (3, 500, 4));
    // B8 的输入里读得到 pooled_fallback_calls，式子里没有它：合计一步不多
    let input = |calls: u64| {
        let mut read = judgement(calls, 500);
        read.state = fields_of("E206RESULT name=state inode_leaf_containers=1 other_files=0 allocation_records=1000 mapped_units=10 deadlist_entries=0 extent_upper_nodes=0 extent_upper_levels=0 accounting_tree_levels=1 deadlist_tree_levels=1 allocation_tree_levels=1 central_mapping_levels=1");
        by_delta_of_the_judgement(&read, 1 << 17)
    };
    assert_eq!(input(0), input(3), "s37 没有 B8 式子");
    let mut item = PointArmReadings::default();
    item.judgements.insert("same_mount/W1/1/".to_string(), JudgementReadings::default());
    item.judgements.insert("same_mount/W1/2/".to_string(), JudgementReadings::default());
    item.judgements.insert("same_mount/W1/1/1".to_string(), judgement(0, 0));
    item.judgements.insert("same_mount/W1/2/1".to_string(), judgement(3, 500));
    item.window = vec![
        Settle { state: "same_mount".to_string(), operation: "W1".to_string(), operation_index: 1, judgement: 1, ..Settle::default() },
        Settle { state: "same_mount".to_string(), operation: "W1".to_string(), operation_index: 2, judgement: 1, ..Settle::default() },
    ];
    assert_eq!(pooled_fallback_of_the_window(&item), (2, 1, 3, 500, 8));
}

#[test]
fn an_unread_state_is_a_gap_and_not_zero() {
    let judgement = JudgementReadings {
        fixed_point_rounds: Some(0),
        ..JudgementReadings::default()
    };
    let gaps = by_delta_input(&judgement, "between", 1 << 17).expect_err("规模读数读不到");
    assert!(gaps.contains(&"state.allocation_records".to_string()));
}

#[test]
fn an_unread_round_is_a_gap_and_not_zero() {
    let judgement = JudgementReadings {
        fixed_point_rounds: Some(1),
        ..JudgementReadings::default()
    };
    let gaps = by_delta_input(&judgement, "between", 1 << 17).expect_err("逐轮读数读不到");
    assert!(gaps.contains(&"changed_leaves@round1".to_string()));
}

#[test]
fn the_state_line_is_merged_into_the_judgement_of_a_settle() {
    let item = lines_of(
        "E206RESULT name=state phase=window state=same_mount op=W1 op_index=1 units=37\n\
E206RESULT name=publish publish=same_mount/W1/1/0 fixed_point_rounds=2\n\
E206RESULT name=settle state=same_mount op=W1 op_index=1 judgement=0 rounds=2\n",
    );
    let judgement = judgement_of(&item, &item.window[0]).expect("那一次判读得到");
    assert_eq!(number(&judgement.state, "units"), Some(37));
    assert_eq!(judgement.fixed_point_rounds, Some(2));
}

#[test]
fn the_e203_reader_takes_only_the_asked_round_of_the_same_mount_overwrite() {
    let path =
        std::env::temp_dir().join(format!("e206-e203-reader-{}.out", std::process::id()));
    std::fs::write(
        &path,
        "E203RESULT name=point point=base unit_area_slots=131072\n\
E203RESULT name=publish state=same_mount kind=overwrite round=1 fixed_point_rounds=2\n\
E203RESULT name=publish state=same_mount kind=overwrite round=2 fixed_point_rounds=9\n\
E203RESULT name=publish state=remounted kind=overwrite round=1 fixed_point_rounds=4\n",
    )
    .expect("临时文件写得下");
    let (judgement, unit_area_slots) =
        e203_judgement(path.to_str().expect("路径是 UTF-8"), 2).expect("读得到");
    std::fs::remove_file(&path).expect("临时文件删得掉");
    assert_eq!(judgement.fixed_point_rounds, Some(9));
    assert_eq!(unit_area_slots, 131_072);
}

#[test]
fn verdicts_agree_flips_when_one_point_changes_level() {
    let readings =
        |text: &[&str]| -> Vec<String> { text.iter().map(|one| (*one).to_string()).collect() };
    assert!(verdicts_agree(&readings(&["a:2:not_many", "b:3:not_many"])));
    assert!(!verdicts_agree(&readings(&["a:2:not_many", "b:22:many"])));
}

#[test]
fn the_measured_anchor_needs_rounds_work_and_every_round() {
    let settle = Settle {
        rounds: 2,
        round_work: 21,
        detail: vec![
            fabricated_round("release_not_covered", 7, 0),
            fabricated_round("settled", 14, 0),
        ],
        ..Settle::default()
    };
    assert!(anchor_matches(&settle, 2, 21, &[7, 14]));
    assert!(!anchor_matches(&settle, 3, 21, &[7, 14]));
    assert!(!anchor_matches(&settle, 2, 22, &[7, 14]));
    assert!(!anchor_matches(&settle, 2, 21, &[7, 15]));
}

#[test]
fn reconciliation_needs_the_two_counts_equal() {
    assert!(reconciliation_holds(14, 14));
    assert!(!reconciliation_holds(14, 15));
}

#[test]
fn rounds_at_least_reads_the_history_and_the_window() {
    let item = lines_of(COMPLETE_PRODUCT);
    assert_eq!(rounds_at_least(&item, 3), 1);
    assert_eq!(rounds_at_least(&item, 2), 3);
}

#[test]
fn fork_rows_need_every_point_of_the_first_segment() {
    let on = |arm: &str| -> BTreeSet<(String, String)> {
        FIRST_SEGMENT_POINTS
            .iter()
            .map(|point| ((*point).to_string(), arm.to_string()))
            .collect()
    };
    assert_eq!(
        fork_rows_with_enough_readings(&on("p1")),
        [("P1", true), ("P2", false), ("P3", true)]
    );
    let mut short = on("p2a");
    short.remove(&("base".to_string(), "p2a".to_string()));
    assert_eq!(
        fork_rows_with_enough_readings(&short),
        [("P1", false), ("P2", false), ("P3", false)]
    );
    assert_eq!(fork_rows_with_enough_readings(&on("p2a"))[1], ("P2", true));
}

#[test]
fn exit_share_only_counts_the_long_fixed_points() {
    let settles = [
        Settle {
            rounds: 9,
            detail: vec![
                fabricated_round("release_not_covered", 1, 1),
                fabricated_round("placement_not_covered", 1, 1),
                fabricated_round("placement_not_covered", 1, 1),
                fabricated_round("settled", 1, 1),
            ],
            ..Settle::default()
        },
        Settle {
            rounds: 2,
            detail: vec![fabricated_round("release_not_covered", 1, 1)],
            ..Settle::default()
        },
    ];
    assert_eq!(exit_share(&settles, "placement_not_covered"), 0.5);
    assert_eq!(exit_share(&settles, "release_not_covered"), 0.25);
}
