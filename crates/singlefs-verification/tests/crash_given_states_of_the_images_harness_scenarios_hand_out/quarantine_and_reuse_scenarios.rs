//! 隔离与复用这一族 harness 场景交出的镜像：隔离标记与引用一致、释放校验和隔离、抬回收下限之后的复用、复用的分配记录重叠、只供测试的开关把复用窗口压到零。

use super::*;

// ─── `tests/checker_quarantine_mark_agrees_with_the_references.rs` ───

/// 与 `crates/singlefs-verification/tests/checker_quarantine_mark_agrees_with_the_references.rs` 里跑同一个场景函数的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_quarantine_mark_agrees_with_the_references::run_the_scenario_where_a_quarantine_carrying_the_mark_on_both_devices_is_exempted_without_reading_its_copies`）。
#[test]
fn quarantine_carrying_the_mark_on_both_devices_is_exempted_without_reading_its_copies() {
    judge_the_images_a_harness_scenario_hands_out(
        "quarantine_carrying_the_mark_on_both_devices_is_exempted_without_reading_its_copies",
        scenarios_of_quarantine_mark_agrees_with_the_references::run_the_scenario_where_a_quarantine_carrying_the_mark_on_both_devices_is_exempted_without_reading_its_copies,
    );
}

/// 与 `crates/singlefs-verification/tests/checker_quarantine_mark_agrees_with_the_references.rs` 里跑同一个场景函数的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_quarantine_mark_agrees_with_the_references::run_the_scenario_where_a_quarantine_carrying_the_mark_on_one_device_only_is_violated_by_the_third_sentence`）。
#[test]
fn quarantine_carrying_the_mark_on_one_device_only_is_violated_by_the_third_sentence() {
    judge_the_images_a_harness_scenario_hands_out(
        "quarantine_carrying_the_mark_on_one_device_only_is_violated_by_the_third_sentence",
        scenarios_of_quarantine_mark_agrees_with_the_references::run_the_scenario_where_a_quarantine_carrying_the_mark_on_one_device_only_is_violated_by_the_third_sentence,
    );
}

/// 与 `crates/singlefs-verification/tests/checker_quarantine_mark_agrees_with_the_references.rs` 里跑同一个场景函数的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_quarantine_mark_agrees_with_the_references::run_the_scenario_where_a_record_carrying_the_mark_under_a_unit_the_newest_root_references_is_violated_by_the_second_sentence`）。
#[test]
fn record_carrying_the_mark_under_a_unit_the_newest_root_references_is_violated_by_the_second_sentence(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "record_carrying_the_mark_under_a_unit_the_newest_root_references_is_violated_by_the_second_sentence",
        scenarios_of_quarantine_mark_agrees_with_the_references::run_the_scenario_where_a_record_carrying_the_mark_under_a_unit_the_newest_root_references_is_violated_by_the_second_sentence,
    );
}

/// 与 `crates/singlefs-verification/tests/checker_quarantine_mark_agrees_with_the_references.rs` 里跑同一个场景函数的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_quarantine_mark_agrees_with_the_references::run_the_scenario_where_a_quarantine_without_the_mark_is_violated_whether_or_not_its_copies_read_back`）。
#[test]
fn quarantine_without_the_mark_is_violated_whether_or_not_its_copies_read_back() {
    judge_the_images_a_harness_scenario_hands_out(
        "quarantine_without_the_mark_is_violated_whether_or_not_its_copies_read_back",
        scenarios_of_quarantine_mark_agrees_with_the_references::run_the_scenario_where_a_quarantine_without_the_mark_is_violated_whether_or_not_its_copies_read_back,
    );
}

/// 与 `crates/singlefs-verification/tests/checker_quarantine_mark_agrees_with_the_references.rs` 里跑同一个场景函数的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_quarantine_mark_agrees_with_the_references::run_the_scenario_where_a_unit_whose_records_fall_into_two_allocation_record_trees_carrying_the_mark_in_one_only_is_violated_by_the_third_sentence`）。
#[test]
fn unit_whose_records_fall_into_two_allocation_record_trees_carrying_the_mark_in_one_only_is_violated_by_the_third_sentence(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "unit_whose_records_fall_into_two_allocation_record_trees_carrying_the_mark_in_one_only_is_violated_by_the_third_sentence",
        scenarios_of_quarantine_mark_agrees_with_the_references::run_the_scenario_where_a_unit_whose_records_fall_into_two_allocation_record_trees_carrying_the_mark_in_one_only_is_violated_by_the_third_sentence,
    );
}

// ─── `tests/release_checksum_quarantine.rs` ───

/// 与 `crates/singlefs-verification/tests/release_checksum_quarantine.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_release_checksum_quarantine::run_the_scenario_where_quarantined_record_carrying_the_mark_is_exempted_from_the_allocated_statistics_whether_or_not_its_copy_reads_back`）。
#[test]
fn quarantined_record_carrying_the_mark_is_exempted_from_the_allocated_statistics_whether_or_not_its_copy_reads_back(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "quarantined_record_carrying_the_mark_is_exempted_from_the_allocated_statistics_whether_or_not_its_copy_reads_back",
        scenarios_of_release_checksum_quarantine::run_the_scenario_where_quarantined_record_carrying_the_mark_is_exempted_from_the_allocated_statistics_whether_or_not_its_copy_reads_back,
    );
}

/// 与 `crates/singlefs-verification/tests/release_checksum_quarantine.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_release_checksum_quarantine::run_the_scenario_where_unit_with_one_failing_copy_carries_the_mark_on_every_device_and_stays_exempted_once_both_copies_read_back`）。
#[test]
fn unit_with_one_failing_copy_carries_the_mark_on_every_device_and_stays_exempted_once_both_copies_read_back(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "unit_with_one_failing_copy_carries_the_mark_on_every_device_and_stays_exempted_once_both_copies_read_back",
        scenarios_of_release_checksum_quarantine::run_the_scenario_where_unit_with_one_failing_copy_carries_the_mark_on_every_device_and_stays_exempted_once_both_copies_read_back,
    );
}

/// 与 `crates/singlefs-verification/tests/release_checksum_quarantine.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_release_checksum_quarantine::run_the_scenario_where_quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_by_the_mark`）。
#[test]
fn quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_by_the_mark() {
    judge_the_images_a_harness_scenario_hands_out(
        "quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_by_the_mark",
        scenarios_of_release_checksum_quarantine::run_the_scenario_where_quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_by_the_mark,
    );
}

/// 与 `crates/singlefs-verification/tests/checker_compaction_intent_and_relocation_must_red_worlds.rs` 里跑同一个场景函数的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_release_checksum_quarantine::run_the_scenario_where_a_quarantined_unit_whose_copy_on_one_device_has_an_unusable_header_holds_the_reverse_lookup`）。
#[test]
fn quarantined_unit_whose_copy_on_one_device_has_an_unusable_header_holds_the_reverse_lookup() {
    judge_the_images_a_harness_scenario_hands_out(
        "quarantined_unit_whose_copy_on_one_device_has_an_unusable_header_holds_the_reverse_lookup",
        scenarios_of_release_checksum_quarantine::run_the_scenario_where_a_quarantined_unit_whose_copy_on_one_device_has_an_unusable_header_holds_the_reverse_lookup,
    );
}

// ─── `tests/reuse_after_raising_the_floor.rs` ───

/// 与 `crates/singlefs-verification/tests/reuse_after_raising_the_floor.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reuse_after_raising_the_floor::run_the_scenario_where_raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it`）。
#[test]
fn raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it",
        scenarios_of_reuse_after_raising_the_floor::run_the_scenario_where_raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it,
    );
}

/// 与 `crates/singlefs-verification/tests/reuse_after_raising_the_floor.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reuse_after_raising_the_floor::run_the_scenario_where_the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker`）。
#[test]
fn the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker",
        scenarios_of_reuse_after_raising_the_floor::run_the_scenario_where_the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker,
    );
}

/// 与 `crates/singlefs-verification/tests/reuse_after_raising_the_floor.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reuse_after_raising_the_floor::run_the_scenario_where_reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red`）。
#[test]
fn reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red",
        scenarios_of_reuse_after_raising_the_floor::run_the_scenario_where_reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red,
    );
}

/// 与 `crates/singlefs-verification/tests/reuse_after_raising_the_floor.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reuse_after_raising_the_floor::run_the_scenario_where_raising_the_floor_in_the_make_filesystem_process_after_three_overwrites_reads_the_instance_table_through_the_root`）。
#[test]
fn raising_the_floor_in_the_make_filesystem_process_after_three_overwrites_reads_the_instance_table_through_the_root(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "raising_the_floor_in_the_make_filesystem_process_after_three_overwrites_reads_the_instance_table_through_the_root",
        scenarios_of_reuse_after_raising_the_floor::run_the_scenario_where_raising_the_floor_in_the_make_filesystem_process_after_three_overwrites_reads_the_instance_table_through_the_root,
    );
}

/// 与 `crates/singlefs-verification/tests/reuse_after_raising_the_floor.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reuse_after_raising_the_floor::run_the_scenario_where_four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units`）。
#[test]
fn four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units",
        scenarios_of_reuse_after_raising_the_floor::run_the_scenario_where_four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units,
    );
}

/// 与 `crates/singlefs-verification/tests/reuse_after_raising_the_floor.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reuse_after_raising_the_floor::run_the_scenario_where_the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor`）。
#[test]
fn the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor",
        scenarios_of_reuse_after_raising_the_floor::run_the_scenario_where_the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor,
    );
}

// ─── `tests/reused_allocation_record_overlap.rs` ───

/// 与 `crates/singlefs-verification/tests/reused_allocation_record_overlap.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reused_allocation_record_overlap::run_the_scenario_where_reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds`）。
#[test]
fn reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds",
        scenarios_of_reused_allocation_record_overlap::run_the_scenario_where_reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds,
    );
}

// ─── `tests/test_only_switches_force_every_branch.rs` ───

/// 与 `crates/singlefs-verification/tests/test_only_switches_force_every_branch.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_test_only_switches_force_every_branch::run_the_scenario_where_reuse_window_forced_to_zero_lets_a_publish_overwrite_a_unit_a_candidate_root_still_references_and_the_checker_goes_red`）。
#[test]
fn reuse_window_forced_to_zero_lets_a_publish_overwrite_a_unit_a_candidate_root_still_references_and_the_checker_goes_red(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "reuse_window_forced_to_zero_lets_a_publish_overwrite_a_unit_a_candidate_root_still_references_and_the_checker_goes_red",
        scenarios_of_test_only_switches_force_every_branch::run_the_scenario_where_reuse_window_forced_to_zero_lets_a_publish_overwrite_a_unit_a_candidate_root_still_references_and_the_checker_goes_red,
    );
}
