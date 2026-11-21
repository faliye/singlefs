use super::claims;
use super::{
    edge_geometries_narrowed_or_registered, narrowed_count_or_registered,
    ALIGNED_PAIR_SHORTAGE_SEEDS, ALIGNED_PAIR_SHORTAGE_STEPS, GEOMETRY_NARROW, GEOMETRY_SMALL,
    SOURCE_PATH_CARRYING_THE_PREFLIGHT_DECLARATIONS,
};

/// `preflight` 交给 `preflight.py check` 的路径要读得到，读出的就是编进这个二进制的根文件，文件头带着准入与运行条件的声明。
/// 要读运行时那条路径上的文件：断言的就是装置跑起来时 `preflight.py` 拿到的那一份在不在、是不是这一份。
#[test]
fn preflight_source_path_reads_the_root_file_compiled_into_this_binary_with_its_declarations() {
    let text = std::fs::read_to_string(SOURCE_PATH_CARRYING_THE_PREFLIGHT_DECLARATIONS)
        .unwrap_or_else(|error| {
            panic!(
                "preflight 要判的源文件 {SOURCE_PATH_CARRYING_THE_PREFLIGHT_DECLARATIONS} 读不到：{error}"
            )
        });
    assert_eq!(
        text,
        include_str!("main.rs"),
        "{SOURCE_PATH_CARRYING_THE_PREFLIGHT_DECLARATIONS} 就是编进这个二进制的根文件"
    );
    assert!(
        text.contains("// admission:") && text.contains("// run-condition:"),
        "根文件的文件头要有准入与运行条件"
    );
}

#[test]
fn the_scale_variables_left_unset_take_the_registered_sixty_four_seeds_three_hundred_steps_and_both_edge_geometries(
) {
    assert_eq!(
        narrowed_count_or_registered(
            "E181_ALIGNED_PAIR_SHORTAGE_SEEDS",
            None,
            ALIGNED_PAIR_SHORTAGE_SEEDS
        ),
        Ok(64),
        "种子数不设取登记的 64"
    );
    assert_eq!(
        narrowed_count_or_registered(
            "E181_ALIGNED_PAIR_SHORTAGE_STEPS",
            None,
            ALIGNED_PAIR_SHORTAGE_STEPS
        ),
        Ok(300),
        "步数不设取登记的 300"
    );
    assert_eq!(
        edge_geometries_narrowed_or_registered(None),
        Ok(format!("{},{}", GEOMETRY_NARROW.name, GEOMETRY_SMALL.name)),
        "跑边的几何不设取登记的两个"
    );
}

#[test]
fn the_scale_variables_narrow_only_and_a_value_above_the_registered_one_is_refused() {
    assert_eq!(
        narrowed_count_or_registered(
            "E181_ALIGNED_PAIR_SHORTAGE_SEEDS",
            Some("4"),
            ALIGNED_PAIR_SHORTAGE_SEEDS
        ),
        Ok(4),
        "往小设照取"
    );
    assert_eq!(
        narrowed_count_or_registered(
            "E181_ALIGNED_PAIR_SHORTAGE_SEEDS",
            Some("64"),
            ALIGNED_PAIR_SHORTAGE_SEEDS
        ),
        Ok(64),
        "设成登记值照取"
    );
    assert!(
        narrowed_count_or_registered(
            "E181_ALIGNED_PAIR_SHORTAGE_SEEDS",
            Some("65"),
            ALIGNED_PAIR_SHORTAGE_SEEDS
        )
        .is_err(),
        "种子数设得比登记的 64 大要拒"
    );
    assert!(
        narrowed_count_or_registered(
            "E181_ALIGNED_PAIR_SHORTAGE_STEPS",
            Some("301"),
            ALIGNED_PAIR_SHORTAGE_STEPS
        )
        .is_err(),
        "步数设得比登记的 300 大要拒"
    );
    assert!(
        narrowed_count_or_registered(
            "E181_ALIGNED_PAIR_SHORTAGE_STEPS",
            Some("abc"),
            ALIGNED_PAIR_SHORTAGE_STEPS
        )
        .is_err(),
        "不是十进制整数要拒"
    );
    assert_eq!(
        edge_geometries_narrowed_or_registered(Some(GEOMETRY_SMALL.name)),
        Ok(GEOMETRY_SMALL.name.to_string()),
        "跑边的几何往小设成登记的一个照取"
    );
    assert!(
        edge_geometries_narrowed_or_registered(Some("wide")).is_err(),
        "跑边的几何写了登记之外的名字要拒"
    );
}

#[test]
fn allocation_record_tree_nodes_of_two_four_gibibyte_devices_are_525_529_and_531_at_root_levels_one_two_and_three(
) {
    claims::check_that_allocation_record_tree_nodes_of_two_four_gibibyte_devices_are_525_529_and_531_at_root_levels_one_two_and_three(
    );
}

#[test]
fn allocation_record_tree_nodes_of_the_small_geometry_are_55_at_root_level_one_and_57_at_root_level_two(
) {
    claims::check_that_allocation_record_tree_nodes_of_the_small_geometry_are_55_at_root_level_one_and_57_at_root_level_two();
}

#[test]
fn central_mapping_nodes_rewritten_at_most_pins_the_hand_computed_examples() {
    claims::check_that_central_mapping_nodes_rewritten_at_most_pins_the_hand_computed_examples();
}

#[test]
fn sparse_layers_share_reads_and_keep_writes_apart() {
    claims::check_that_sparse_layers_share_reads_and_keep_writes_apart();
}

#[test]
fn zeroing_masks_the_sealed_layers_and_a_later_write_shows_through_the_mask() {
    claims::check_that_zeroing_masks_the_sealed_layers_and_a_later_write_shows_through_the_mask();
}

#[test]
fn the_first_file_lands_at_50180_and_its_records_end_at_50253() {
    claims::check_that_the_first_file_lands_at_50180_and_its_records_end_at_50253();
}

#[test]
fn the_first_file_publish_has_six_ordinary_and_eight_fixed_point_slots_by_both_paths() {
    claims::check_that_the_first_file_publish_has_six_ordinary_and_eight_fixed_point_slots_by_both_paths();
}

#[test]
fn the_local_checkpoint_cost_equals_the_core_one_on_every_version_measured() {
    claims::check_that_the_local_checkpoint_cost_equals_the_core_one_on_every_version_measured();
}

#[test]
fn growth_write_deletes_the_four_first_file_units_and_inserts_every_rewritten_mapped_ordinary_role()
{
    claims::check_that_a_growth_write_deletes_the_four_first_file_units_and_inserts_every_rewritten_mapped_ordinary_role();
}

#[test]
fn sixty_four_unit_write_on_the_small_capacity_trees_makes_the_checkpoint_cost_with_ordinary_mapping_changes_exceed_the_checkpoint_cost(
) {
    claims::check_that_a_sixty_four_unit_write_on_the_small_capacity_trees_makes_the_checkpoint_cost_with_ordinary_mapping_changes_exceed_the_checkpoint_cost();
}

#[test]
fn extra_deletes_and_inserts_each_raise_the_local_checkpoint_cost_on_a_wide_tree() {
    claims::check_that_extra_deletes_and_inserts_each_raise_the_local_checkpoint_cost_on_a_wide_tree();
}

#[test]
fn the_judges_flip_when_the_threshold_moves_between_two_readings() {
    claims::check_that_the_judges_flip_when_the_threshold_moves_between_two_readings();
}

#[test]
fn slots_enough_and_aligned_pairs_short_holds_only_when_the_slots_are_enough_everywhere_and_the_aligned_pairs_are_short_somewhere(
) {
    claims::check_that_slots_enough_and_aligned_pairs_short_holds_only_when_the_slots_are_enough_everywhere_and_the_aligned_pairs_are_short_somewhere();
}

#[test]
fn marking_every_pair_outside_the_clusters_leaves_no_lowest_slot_and_marking_none_leaves_50182() {
    claims::check_that_marking_every_pair_outside_the_clusters_leaves_no_lowest_slot_and_marking_none_leaves_50182();
}

#[test]
fn aligned_free_pairs_count_the_open_segment_too() {
    claims::check_that_aligned_free_pairs_count_the_open_segment_too();
}

#[test]
fn the_seeded_stream_repeats_for_a_seed_and_differs_between_seeds() {
    claims::check_that_the_seeded_stream_repeats_for_a_seed_and_differs_between_seeds();
}

#[test]
fn refusals_are_named_by_the_gate_that_refused() {
    claims::check_that_refusals_are_named_by_the_gate_that_refused();
}

#[test]
fn the_units_to_try_after_a_refusal_follow_the_surplus_the_shortfall_or_half() {
    claims::check_that_the_units_to_try_after_a_refusal_follow_the_surplus_the_shortfall_or_half();
}

#[test]
fn capped_fill_of_the_small_geometry_publishes_several_times_and_ends_with_nothing_left_or_a_refused_single_unit(
) {
    claims::check_that_a_capped_fill_of_the_small_geometry_publishes_several_times_and_ends_with_nothing_left_or_a_refused_single_unit();
}

#[test]
fn the_lower_bound_is_tried_on_the_filled_base_first_and_follows_the_refusals_down_to_a_passing_count(
) {
    claims::check_that_the_lower_bound_is_tried_on_the_filled_base_first_and_follows_the_refusals_down_to_a_passing_count();
}

#[test]
fn the_edge_search_predicts_the_edge_from_the_shortfall_and_closes_it_with_observed_points() {
    claims::check_that_the_edge_search_predicts_the_edge_from_the_shortfall_and_closes_it_with_observed_points();
}

#[test]
fn bisection_round_tries_the_predicted_edge_and_the_next_count_before_splitting_evenly() {
    claims::check_that_a_bisection_round_tries_the_predicted_edge_and_the_next_count_before_splitting_evenly();
}

#[test]
fn an_edge_that_no_observation_brackets_is_reported_as_not_found() {
    claims::check_that_an_edge_that_no_observation_brackets_is_reported_as_not_found();
}

#[test]
fn small_base_filled_to_six_tenths_gets_an_edge_below_the_filled_units_that_independent_probes_confirm(
) {
    claims::check_that_a_small_base_filled_to_six_tenths_gets_an_edge_below_the_filled_units_that_independent_probes_confirm();
}

#[test]
fn the_refusal_note_names_the_gate_and_the_gap_and_the_shortfall_rounds_up_to_whole_units() {
    claims::check_that_the_refusal_note_names_the_gate_and_the_gap_and_the_shortfall_rounds_up_to_whole_units();
}

#[test]
fn geometry_whose_edge_was_not_found_stops_the_fixed_point_upper_bound_section() {
    claims::check_that_a_geometry_whose_edge_was_not_found_stops_the_fixed_point_upper_bound_section();
}

#[test]
fn the_preflight_verdict_line_gives_a_summary_only_when_forced() {
    claims::check_that_the_preflight_verdict_line_gives_a_summary_only_when_forced();
}

#[test]
fn local_constants_agree_with_the_crates() {
    claims::check_that_local_constants_agree_with_the_crates();
}
