use super::*;

#[test]
fn local_constants_equal_the_implementation_constants_and_the_in_flight_limits_are_21_3_2() {
    cross_check_local_constants_against_the_implementation();
}

#[test]
fn every_generated_record_carries_the_back_chain_of_the_record_before_it() {
    check_the_back_chain_of_every_generated_record();
}

#[test]
fn every_record_of_a_publish_carries_the_same_new_root_segment() {
    check_the_new_root_segment_of_every_generated_record();
}

#[test]
fn the_state_counts_of_the_smallest_sizes_equal_the_pinned_values_and_the_empty_subset_is_counted()
{
    check_the_state_counts_of_the_smallest_sizes();
}

#[test]
fn the_three_readings_part_on_a_later_txg_and_on_a_missing_counter() {
    check_the_three_readings_on_hand_made_tables();
}

#[test]
fn root_plus_disk_is_violated_when_a_named_unit_copy_is_bad() {
    check_root_plus_disk_on_a_bad_unit_copy();
}

#[test]
fn the_in_flight_limit_control_is_red_on_the_small_ring_and_green_on_the_big_ring() {
    check_the_in_flight_limit_control_on_both_rings();
}

#[test]
fn the_farthest_version_stops_at_the_in_flight_limit_on_the_small_ring() {
    check_the_farthest_version_on_the_small_ring();
}

#[test]
fn the_fixed_point_control_is_red() {
    check_the_fixed_point_control();
}

#[test]
fn ideal_recovery_on_one_later_record_breaks_the_applied_set_reading_once_and_the_landed_version_reading_never(
) {
    check_the_two_monotone_readings_on_one_later_record();
}

#[test]
fn the_structure_control_is_red() {
    check_the_structure_control();
}

#[test]
fn residue_on_the_second_device_does_not_complete_a_publish() {
    check_a_residue_does_not_complete_a_publish();
}

#[test]
fn parse_rejects_a_record_with_ordinal_zero_whose_checksums_hold() {
    check_parse_rejects_ordinal_zero_whose_checksums_hold();
}

#[test]
fn the_varying_fields_generator_varies_the_four_fields_beyond_the_projection_with_the_version() {
    check_the_fields_beyond_the_projection_of_the_varying_fields_generator();
}

#[test]
fn without_profile_the_three_constant_fields_product_commands_stay_on_the_constant_fields_profile_and_the_marker_parity_needs_varying_fields(
) {
    check_the_profile_options();
}

#[test]
fn the_j6qn_controls_read_3110_on_the_big_ring_and_70_against_0_on_e1() {
    check_the_j6qn_control_readings();
}

#[test]
fn the_copied_fields_controls_read_762_and_2348_on_the_varying_fields_generator_and_0_on_the_constant_fields_generator(
) {
    check_the_copied_fields_control_readings();
}

#[test]
fn txg_order_holds_when_the_txg_does_not_fall_with_the_counter_inside_one_instance_and_equal_txgs_count_as_holding(
) {
    check_the_txg_order_judgement_on_hand_made_tables();
}

#[test]
fn the_txg_order_state_counts_of_the_adversarial_domains_equal_the_pinned_values() {
    check_the_txg_order_state_counts_of_the_smallest_sizes();
}

#[test]
fn the_txg_order_split_controls_read_the_pinned_cells_of_the_adversarial_domains() {
    check_the_txg_order_split_control_readings();
}

#[test]
fn every_txg_order_cell_has_a_row_on_the_txg_order_split_profile_and_the_other_profiles_have_none()
{
    check_the_rows_of_the_txg_order_split();
}

#[test]
fn the_odd_marker_parity_turns_the_unmount_marker_controls_to_0_on_the_legal_big_ring_and_above_0_on_the_monotone_big_ring(
) {
    check_the_odd_marker_parity_turns_the_unmount_marker_controls();
}

#[test]
fn the_multi_disk_state_counts_lagging_classes_and_witnesses_of_the_smallest_sizes_equal_the_pinned_values(
) {
    check_the_multi_disk_state_counts_of_the_smallest_sizes();
}

#[test]
fn state_whose_first_device_holds_nothing_and_whose_second_holds_the_first_record_lags_on_the_first_device(
) {
    check_the_lagging_direction_on_hand_made_masks();
}

#[test]
fn the_multi_disk_controls_read_100_996_0_on_the_first_device_alone_and_228_2052_45_on_the_returned_root_and_farthest_version_controls(
) {
    check_the_multi_disk_control_readings();
}

#[test]
fn residue_enters_the_direct_table_only_when_the_first_device_copy_is_not_on_disk() {
    check_the_direct_table_reads_the_first_device_before_the_second();
}

#[test]
fn the_scan_configuration_carries_the_generators_filesystem_identifier_and_ring_length() {
    check_the_scan_system_configuration_matches_the_generator();
}

#[test]
fn the_table_and_disk_order_options_default_to_direct_and_normal_and_the_constant_fields_profile_refuses_them(
) {
    check_the_table_and_disk_order_options();
}

#[test]
fn reversed_memory_disk_reports_the_second_device_before_the_first() {
    check_a_reversed_memory_disk_reports_the_second_device_first();
}
