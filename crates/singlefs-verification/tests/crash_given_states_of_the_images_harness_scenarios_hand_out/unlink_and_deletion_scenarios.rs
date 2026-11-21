//! unlink 与删除这一族 harness 场景交出的镜像：unlink 路径上 inode 叶的吸收与借、unlink 的幂等判据与回退到删空的版本、挂载续删、跨两片 extent 下层叶的产品批、第一个文件之外的文件与 inode 的删除批。

use super::*;

// ─── `tests/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs` ───

/// 与 `crates/singlefs-verification/tests/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::run_the_scenario_where_the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one`）。
#[test]
fn the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one",
        scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::run_the_scenario_where_the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one,
    );
}

/// 与 `crates/singlefs-verification/tests/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs` 里跑同一个场景函数的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::run_the_scenario_where_a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing`）。
#[test]
fn leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing() {
    judge_the_images_a_harness_scenario_hands_out(
        "leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing",
        scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::run_the_scenario_where_a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing,
    );
}

/// 与 `crates/singlefs-verification/tests/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::run_the_scenario_where_the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it`）。
#[test]
fn the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it",
        scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::run_the_scenario_where_the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it,
    );
}

// ─── `tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` ───

/// 与 `crates/singlefs-verification/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` 里跑同一个场景函数的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version::run_the_scenario_where_a_crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to_the_uninterrupted_end_state`）。
#[test]
fn crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to_the_uninterrupted_end_state(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to_the_uninterrupted_end_state",
        scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version::run_the_scenario_where_a_crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to_the_uninterrupted_end_state,
    );
}

/// 与 `crates/singlefs-verification/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version::run_the_scenario_where_rolling_back_to_the_emptied_version_is_accepted_and_publishes_a_version_with_trees_and_no_file`）。
#[test]
fn rolling_back_to_the_emptied_version_is_accepted_and_publishes_a_version_with_trees_and_no_file()
{
    judge_the_images_a_harness_scenario_hands_out(
        "rolling_back_to_the_emptied_version_is_accepted_and_publishes_a_version_with_trees_and_no_file",
        scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version::run_the_scenario_where_rolling_back_to_the_emptied_version_is_accepted_and_publishes_a_version_with_trees_and_no_file,
    );
}

// ─── `tests/unlink_mount_continuation_and_the_emptied_version.rs` ───

/// 与 `crates/singlefs-verification/tests/unlink_mount_continuation_and_the_emptied_version.rs` 里跑同一个场景函数的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unlink_mount_continuation_and_the_emptied_version::run_the_scenario_where_a_writable_mount_after_the_intent_continues_the_queue_to_empty_after_its_warm_up`）。
#[test]
fn writable_mount_after_the_intent_continues_the_queue_to_empty_after_its_warm_up() {
    judge_the_images_a_harness_scenario_hands_out(
        "writable_mount_after_the_intent_continues_the_queue_to_empty_after_its_warm_up",
        scenarios_of_unlink_mount_continuation_and_the_emptied_version::run_the_scenario_where_a_writable_mount_after_the_intent_continues_the_queue_to_empty_after_its_warm_up,
    );
}

/// 与 `crates/singlefs-verification/tests/unlink_mount_continuation_and_the_emptied_version.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unlink_mount_continuation_and_the_emptied_version::run_the_scenario_where_the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish`）。
#[test]
fn the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish",
        scenarios_of_unlink_mount_continuation_and_the_emptied_version::run_the_scenario_where_the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish,
    );
}

/// 与 `crates/singlefs-verification/tests/unlink_mount_continuation_and_the_emptied_version.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unlink_mount_continuation_and_the_emptied_version::run_the_scenario_where_rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_continues_it_to_empty`）。
#[test]
fn rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_continues_it_to_empty(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_continues_it_to_empty",
        scenarios_of_unlink_mount_continuation_and_the_emptied_version::run_the_scenario_where_rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_continues_it_to_empty,
    );
}

// ─── `tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs` ───

/// 与 `crates/singlefs-verification/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs` 里跑同一个场景函数的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches::run_the_scenario_where_a_file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut`）。
#[test]
fn file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut",
        scenarios_of_unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches::run_the_scenario_where_a_file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut,
    );
}

// ─── `tests/unlink_single_file_publishes.rs` ───

/// 与 `crates/singlefs-verification/tests/unlink_single_file_publishes.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unlink_single_file_publishes::run_the_scenario_where_the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch`）。
#[test]
fn the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch",
        scenarios_of_unlink_single_file_publishes::run_the_scenario_where_the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch,
    );
}

#[test]
fn inode_150_of_three_data_units_in_batches_of_one_takes_two_batches_and_a_final_batch() {
    judge_the_images_a_harness_scenario_hands_out(
        "inode_150_of_three_data_units_in_batches_of_one_takes_two_batches_and_a_final_batch",
        scenarios_of_unlink_batches_of_an_inode_other_than_the_first_follow_its_own_data_units::run_the_scenario_where_inode_150_of_three_data_units_in_batches_of_one_takes_two_batches_and_a_final_batch,
    );
}

#[test]
fn inode_150_of_seven_data_units_in_batches_of_two_takes_three_batches_and_a_final_batch() {
    judge_the_images_a_harness_scenario_hands_out(
        "inode_150_of_seven_data_units_in_batches_of_two_takes_three_batches_and_a_final_batch",
        scenarios_of_unlink_batches_of_an_inode_other_than_the_first_follow_its_own_data_units::run_the_scenario_where_inode_150_of_seven_data_units_in_batches_of_two_takes_three_batches_and_a_final_batch,
    );
}

#[test]
fn inode_150_of_144_data_units_with_the_product_batch_goes_straight_to_the_final_batch() {
    judge_the_images_a_harness_scenario_hands_out(
        "inode_150_of_144_data_units_with_the_product_batch_goes_straight_to_the_final_batch",
        scenarios_of_unlink_batches_of_an_inode_other_than_the_first_follow_its_own_data_units::run_the_scenario_where_inode_150_of_144_data_units_with_the_product_batch_goes_straight_to_the_final_batch,
    );
}

#[test]
fn inode_150_of_145_data_units_with_the_product_batch_cuts_144_and_then_releases_the_last_one() {
    judge_the_images_a_harness_scenario_hands_out(
        "inode_150_of_145_data_units_with_the_product_batch_cuts_144_and_then_releases_the_last_one",
        scenarios_of_unlink_batches_of_an_inode_other_than_the_first_follow_its_own_data_units::run_the_scenario_where_inode_150_of_145_data_units_with_the_product_batch_cuts_144_and_then_releases_the_last_one,
    );
}

#[test]
fn unlinking_another_file_holding_three_data_units_releases_them_and_drops_its_upper_entry() {
    judge_the_images_a_harness_scenario_hands_out(
        "unlinking_another_file_holding_three_data_units_releases_them_and_drops_its_upper_entry",
        scenarios_of_unlink_of_files_other_than_the_first::run_the_scenario_where_unlinking_another_file_holding_three_data_units_releases_them_and_drops_its_upper_entry,
    );
}

#[test]
fn each_deletion_batch_of_another_file_cuts_only_that_file_and_the_first_file_reads_back_unchanged()
{
    judge_the_images_a_harness_scenario_hands_out(
        "each_deletion_batch_of_another_file_cuts_only_that_file_and_the_first_file_reads_back_unchanged",
        scenarios_of_unlink_of_files_other_than_the_first::run_the_scenario_where_each_deletion_batch_of_another_file_cuts_only_that_file_and_the_first_file_reads_back_unchanged,
    );
}

#[test]
fn unlinking_the_first_file_after_the_upper_segment_grew_to_level_one_keeps_the_leaf_of_the_other_file(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "unlinking_the_first_file_after_the_upper_segment_grew_to_level_one_keeps_the_leaf_of_the_other_file",
        scenarios_of_unlink_of_files_other_than_the_first::run_the_scenario_where_unlinking_the_first_file_after_the_upper_segment_grew_to_level_one_keeps_the_leaf_of_the_other_file,
    );
}

#[test]
fn deleting_inode_five_hundred_beside_the_first_file_lowers_the_upper_segment_to_the_first_leaf() {
    judge_the_images_a_harness_scenario_hands_out(
        "deleting_inode_five_hundred_beside_the_first_file_lowers_the_upper_segment_to_the_first_leaf",
        scenarios_of_unlink_of_files_other_than_the_first::run_the_scenario_where_deleting_inode_five_hundred_beside_the_first_file_lowers_the_upper_segment_to_the_first_leaf,
    );
}

#[test]
fn deleting_the_first_file_beside_inode_five_hundred_leaves_its_leaf_under_a_level_one_root() {
    judge_the_images_a_harness_scenario_hands_out(
        "deleting_the_first_file_beside_inode_five_hundred_leaves_its_leaf_under_a_level_one_root",
        scenarios_of_unlink_of_files_other_than_the_first::run_the_scenario_where_deleting_the_first_file_beside_inode_five_hundred_leaves_its_leaf_under_a_level_one_root,
    );
}

#[test]
fn deleting_both_files_in_either_order_ends_in_one_zero_entry_upper_root() {
    judge_the_images_a_harness_scenario_hands_out(
        "deleting_both_files_in_either_order_ends_in_one_zero_entry_upper_root",
        scenarios_of_unlink_of_files_other_than_the_first::run_the_scenario_where_deleting_both_files_in_either_order_ends_in_one_zero_entry_upper_root,
    );
}

#[test]
fn one_set_of_files_with_data_reaches_one_upper_segment_through_two_histories() {
    judge_the_images_a_harness_scenario_hands_out(
        "one_set_of_files_with_data_reaches_one_upper_segment_through_two_histories",
        scenarios_of_unlink_of_files_other_than_the_first::run_the_scenario_where_one_set_of_files_with_data_reaches_one_upper_segment_through_two_histories,
    );
}

#[test]
fn file_created_before_the_first_file_was_deleted_is_still_written_after_it() {
    judge_the_images_a_harness_scenario_hands_out(
        "file_created_before_the_first_file_was_deleted_is_still_written_after_it",
        scenarios_of_unlink_of_files_other_than_the_first::run_the_scenario_where_a_file_created_before_the_first_file_was_deleted_is_still_written_after_it,
    );
}

#[test]
fn writable_remount_after_the_intent_of_another_file_continues_its_deletion_to_the_end() {
    judge_the_images_a_harness_scenario_hands_out(
        "writable_remount_after_the_intent_of_another_file_continues_its_deletion_to_the_end",
        scenarios_of_unlink_of_files_other_than_the_first::run_the_scenario_where_a_writable_remount_after_the_intent_of_another_file_continues_its_deletion_to_the_end,
    );
}

#[test]
fn root_left_one_level_above_the_only_leaf_with_an_entry_reddens_the_upper_shape_invariant() {
    judge_the_images_a_harness_scenario_hands_out(
        "root_left_one_level_above_the_only_leaf_with_an_entry_reddens_the_upper_shape_invariant",
        scenarios_of_unlink_of_files_other_than_the_first::run_the_scenario_where_a_root_left_one_level_above_the_only_leaf_with_an_entry_reddens_the_upper_shape_invariant,
    );
}
