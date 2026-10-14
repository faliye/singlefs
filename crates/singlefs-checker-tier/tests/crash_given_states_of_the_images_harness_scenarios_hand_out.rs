//! checker 档模块：crash_point_replay_progress、crash_given_states、crash_library、crash_amplification、gpu_unit_checks
//! harness 档集成测试真跑实现搭出来的镜像，池级 checker 那一遍搬到这里经给定状态引擎两段判（用户 2026-10-14 弹窗定「搬进 checker 档两段」：
//! harness 档集成测试真跑实现之后在 CPU 上对镜像跑池级 checker 的那些点，池级 checker 那一遍搬进 checker 档、经给定状态表两段判、
//! 配了 GPU 走 GPU；harness 用例只留实现的行为与理想模型、RefFS 的比对）。
//!
//! 每条用例与 harness 档里同名的那条是同一个场景：经 `#[path]` 引 harness 档 `tests/common_images_for_the_checker_tier/<测试文件名>.rs`
//! 里同名的场景函数（连同它用到的 `tests/common*/mod.rs`），拿收镜像的那一种 `PoolCheckerImages` 调它——场景照样真跑实现、照样核行为，
//! 原先就地跑池级 checker 的那几处交出那一刻的镜像与池级 checker 该判成什么。每张镜像怎么建表、交引擎判、逐张核场景写下的那几样，
//! 判法只一份，在共用模块 `tests/common_given_states_of_harness_scenario_images/mod.rs`。场景按 harness 测试文件分组，组按文件名排。
#![cfg(feature = "verdict-store")]
#![allow(
    clippy::duplicate_mod,
    reason = "harness 档的 common_admission 与 common_unwitnessed_publish 各经 #[path] 装一份 common_admission_pool；两批场景各用其一，类型互不相交"
)]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;
#[path = "../../singlefs-harness/tests/common_corrupted_allocation_record/mod.rs"]
mod common_corrupted_allocation_record;
#[path = "../../singlefs-harness/tests/common_deletion_in_progress/mod.rs"]
mod common_deletion_in_progress;
mod common_given_states_of_harness_scenario_images;
#[cfg(feature = "gpu")]
mod common_gpu_cards_of_the_caller;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/mod.rs"]
mod common_images_for_the_checker_tier;
#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;
#[path = "../../singlefs-harness/tests/common_root_ring_holes/mod.rs"]
mod common_root_ring_holes;
#[path = "../../singlefs-harness/tests/common_tree_split/mod.rs"]
mod common_tree_split;
#[path = "../../singlefs-harness/tests/common_unwitnessed_publish/mod.rs"]
mod common_unwitnessed_publish;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs"]
mod scenarios_of_a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/accounting_and_mapping_tree_split.rs"]
mod scenarios_of_accounting_and_mapping_tree_split;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/admission_formula_on_the_mounted_state.rs"]
mod scenarios_of_admission_formula_on_the_mounted_state;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/admission_raises_the_floor_before_refusing.rs"]
mod scenarios_of_admission_raises_the_floor_before_refusing;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs"]
mod scenarios_of_admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor.rs"]
mod scenarios_of_an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs"]
mod scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/container_trees_grow_a_level_when_the_root_is_full.rs"]
mod scenarios_of_container_trees_grow_a_level_when_the_root_is_full;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/entries_after_mount_refuse_swapped_or_behind_devices.rs"]
mod scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/fault_injection_fast_tier.rs"]
mod scenarios_of_fault_injection_fast_tier;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/fsync_drop_and_devices_without_the_selected_version.rs"]
mod scenarios_of_fsync_drop_and_devices_without_the_selected_version;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs"]
mod scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/inode_number_watermark_pressed_back_reissues_a_number.rs"]
mod scenarios_of_inode_number_watermark_pressed_back_reissues_a_number;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/instance_table_chain_read.rs"]
mod scenarios_of_instance_table_chain_read;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/instance_table_page_full.rs"]
mod scenarios_of_instance_table_page_full;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/instance_table_second_page_write.rs"]
mod scenarios_of_instance_table_second_page_write;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/instance_table_sweep_at_the_end_of_a_writable_mount.rs"]
mod scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/journal_last_record_flag_of_a_multi_record_publish.rs"]
mod scenarios_of_journal_last_record_flag_of_a_multi_record_publish;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/many_inodes_write_and_read.rs"]
mod scenarios_of_many_inodes_write_and_read;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/multi_record_publishes_of_transaction_zero.rs"]
mod scenarios_of_multi_record_publishes_of_transaction_zero;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/multi_unit_file_write_and_read.rs"]
mod scenarios_of_multi_unit_file_write_and_read;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs"]
mod scenarios_of_named_unit_verification_stands_in_for_the_barrier_before_the_record;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/one_revolution_window_after_a_hole_in_the_root_ring.rs"]
mod scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/overwrite_in_one_instance.rs"]
mod scenarios_of_overwrite_in_one_instance;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/publish_failure_resent_unchanged.rs"]
mod scenarios_of_publish_failure_resent_unchanged;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs"]
mod scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs"]
mod scenarios_of_raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs"]
mod scenarios_of_raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs"]
mod scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/random_histories.rs"]
mod scenarios_of_random_histories;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs"]
mod scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs"]
mod scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root.rs"]
mod scenarios_of_reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/release_checksum_quarantine.rs"]
mod scenarios_of_release_checksum_quarantine;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs"]
mod scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/reuse_after_raising_the_floor.rs"]
mod scenarios_of_reuse_after_raising_the_floor;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/reused_allocation_record_overlap.rs"]
mod scenarios_of_reused_allocation_record_overlap;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/rollback_by_a_forward_publish.rs"]
mod scenarios_of_rollback_by_a_forward_publish;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/rollback_lineage_record_and_its_checkpoint_cost_term.rs"]
mod scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate.rs"]
mod scenarios_of_rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs"]
mod scenarios_of_root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs"]
mod scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs"]
mod scenarios_of_root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/root_ring_turn_in_one_mount.rs"]
mod scenarios_of_root_ring_turn_in_one_mount;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/root_ring_turn_in_the_make_filesystem_session.rs"]
mod scenarios_of_root_ring_turn_in_the_make_filesystem_session;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/row_publish_admission.rs"]
mod scenarios_of_row_publish_admission;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/row_publish_checks_before_acquisition.rs"]
mod scenarios_of_row_publish_checks_before_acquisition;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/row_publish_record_without_its_root.rs"]
mod scenarios_of_row_publish_record_without_its_root;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs"]
mod scenarios_of_shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/system_configuration_fields_core_and_checker_refuse_the_same_table.rs"]
mod scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/system_configuration_per_device_redundancy.rs"]
mod scenarios_of_system_configuration_per_device_redundancy;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs"]
mod scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/test_only_switches_force_every_branch.rs"]
mod scenarios_of_test_only_switches_force_every_branch;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs"]
mod scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs"]
mod scenarios_of_unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs"]
mod scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/unlink_mount_continuation_and_the_emptied_version.rs"]
mod scenarios_of_unlink_mount_continuation_and_the_emptied_version;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs"]
mod scenarios_of_unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/unlink_single_file_publishes.rs"]
mod scenarios_of_unlink_single_file_publishes;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs"]
mod scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/writable_mount_of_a_formatted_pool.rs"]
mod scenarios_of_writable_mount_of_a_formatted_pool;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/writable_mount_takes_a_second_instance.rs"]
mod scenarios_of_writable_mount_takes_a_second_instance;

use std::collections::BTreeSet;

use common_given_states_of_harness_scenario_images::{
    is_judged, is_violated, judge_one_image, judge_the_collected_images,
    judge_the_images_a_harness_scenario_hands_out,
    judge_the_images_a_harness_scenario_hands_out_under_the_container_capacity_knobs,
    judge_the_images_a_harness_scenario_hands_out_when_it_hands_out_none, mismatches_of_the_images,
    start_the_engine, start_the_engine_on_the_library_of_the_test, the_run_counts,
    ConclusionsOfAnImage, JudgedAfreshRegardlessOfTheExpectations, LibraryOfTheTestAtTheStart,
    WhenTheScenarioHandsOutNoImage,
};
use common_images_for_the_checker_tier::{
    PoolCheckerExpectation, PoolCheckerImage, PoolCheckerImages,
};
use singlefs_checker::image::IMPLEMENTED_INVARIANTS;
use singlefs_checker::walk::ContainerCapacityKnobs;
use singlefs_checker_tier::crash_given_states::GivenStatesOfATable;
use singlefs_harness::fault_injection::{
    classify_the_pool_checker_verdicts_of_the_images_after_a_fault, FailureAfterAFault,
    ImagesAfterAFault, KnownRedFormAtAFault, KnownRedFormOnlyAfterAFault,
};
#[cfg(feature = "gpu")]
use {
    common_given_states_of_harness_scenario_images::ITEM_NAME,
    singlefs_checker_tier::crash_amplification::{
        GivenStatesEngine, GivenStatesItem, GivenStatesRun,
    },
    singlefs_checker_tier::crash_library::{
        repository_root_of_this_crate, CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
    },
    singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity,
};

/// 这份测试文件的仓内路径：给定状态表那一路的设点文件（CPU 结论的版本按它出发的闭包取；共用模块经 `crate::THIS_FILE` 读它）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs";

// ============================== 场景：每份 harness 测试文件一组，与 harness 档的用例同名 ==============================

// ─── harness 档 `tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable::a_witnessed_newest_root_that_reads_back_an_error_until_just_before_the_one_reread_is_chosen_and_nothing_is_abandoned`）。
#[test]
fn a_witnessed_newest_root_that_reads_back_an_error_until_just_before_the_one_reread_is_chosen_and_nothing_is_abandoned(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_witnessed_newest_root_that_reads_back_an_error_until_just_before_the_one_reread_is_chosen_and_nothing_is_abandoned",
        scenarios_of_a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable::a_witnessed_newest_root_that_reads_back_an_error_until_just_before_the_one_reread_is_chosen_and_nothing_is_abandoned,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable::a_witnessed_newest_root_that_reads_back_zeros_until_just_before_the_one_reread_is_chosen_because_zeros_stay_out_of_the_read_cache`）。
#[test]
fn a_witnessed_newest_root_that_reads_back_zeros_until_just_before_the_one_reread_is_chosen_because_zeros_stay_out_of_the_read_cache(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_witnessed_newest_root_that_reads_back_zeros_until_just_before_the_one_reread_is_chosen_because_zeros_stay_out_of_the_read_cache",
        scenarios_of_a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable::a_witnessed_newest_root_that_reads_back_zeros_until_just_before_the_one_reread_is_chosen_because_zeros_stay_out_of_the_read_cache,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable::a_witnessed_newest_root_whose_reads_fail_only_once_is_chosen_by_the_immediate_reread_of_the_product_path`）。
#[test]
fn a_witnessed_newest_root_whose_reads_fail_only_once_is_chosen_by_the_immediate_reread_of_the_product_path(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_witnessed_newest_root_whose_reads_fail_only_once_is_chosen_by_the_immediate_reread_of_the_product_path",
        scenarios_of_a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable::a_witnessed_newest_root_whose_reads_fail_only_once_is_chosen_by_the_immediate_reread_of_the_product_path,
    );
}

// ─── harness 档 `tests/accounting_and_mapping_tree_split.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/accounting_and_mapping_tree_split.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_accounting_and_mapping_tree_split::under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts`）。
#[test]
fn under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts",
        scenarios_of_accounting_and_mapping_tree_split::under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/accounting_and_mapping_tree_split.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_accounting_and_mapping_tree_split::an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes`）。
#[test]
fn an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes",
        scenarios_of_accounting_and_mapping_tree_split::an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/accounting_and_mapping_tree_split.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_accounting_and_mapping_tree_split::pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on`）。
#[test]
fn pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on() {
    judge_the_images_a_harness_scenario_hands_out(
        "pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on",
        scenarios_of_accounting_and_mapping_tree_split::pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/accounting_and_mapping_tree_split.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_accounting_and_mapping_tree_split::the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root`）。
#[test]
fn the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root",
        scenarios_of_accounting_and_mapping_tree_split::the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root,
    );
}

// ─── harness 档 `tests/admission_formula_on_the_mounted_state.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_admission_formula_on_the_mounted_state::writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_the_row_publish_and_is_admitted`）。
#[test]
fn writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_the_row_publish_and_is_admitted(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_the_row_publish_and_is_admitted",
        scenarios_of_admission_formula_on_the_mounted_state::writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_the_row_publish_and_is_admitted,
    );
}

// ─── harness 档 `tests/admission_raises_the_floor_before_refusing.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_admission_raises_the_floor_before_refusing::a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_and_continues_to_the_emptied_version`）。
#[test]
fn a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_and_continues_to_the_emptied_version(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_and_continues_to_the_emptied_version",
        scenarios_of_admission_raises_the_floor_before_refusing::a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_and_continues_to_the_emptied_version,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_admission_raises_the_floor_before_refusing::a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue`）。
#[test]
fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue",
        scenarios_of_admission_raises_the_floor_before_refusing::a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_admission_raises_the_floor_before_refusing::a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims`）。
#[test]
fn a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims",
        scenarios_of_admission_raises_the_floor_before_refusing::a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_admission_raises_the_floor_before_refusing::an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version`）。
#[test]
fn an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version",
        scenarios_of_admission_raises_the_floor_before_refusing::an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_admission_raises_the_floor_before_refusing::after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor`）。
#[test]
fn after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor",
        scenarios_of_admission_raises_the_floor_before_refusing::after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_admission_raises_the_floor_before_refusing::after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes`）。
#[test]
fn after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes() {
    judge_the_images_a_harness_scenario_hands_out(
        "after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes",
        scenarios_of_admission_raises_the_floor_before_refusing::after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_admission_raises_the_floor_before_refusing::crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck`）。
#[test]
fn crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck",
        scenarios_of_admission_raises_the_floor_before_refusing::crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_admission_raises_the_floor_before_refusing::mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount`）。
#[test]
fn mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount",
        scenarios_of_admission_raises_the_floor_before_refusing::mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount,
    );
}

// ─── harness 档 `tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments::with_the_space_admission_switched_off_the_session_raises_the_floor_after_a_placement_refusal_and_publishes_the_write`）。
#[test]
fn with_the_space_admission_switched_off_the_session_raises_the_floor_after_a_placement_refusal_and_publishes_the_write(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "with_the_space_admission_switched_off_the_session_raises_the_floor_after_a_placement_refusal_and_publishes_the_write",
        scenarios_of_admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments::with_the_space_admission_switched_off_the_session_raises_the_floor_after_a_placement_refusal_and_publishes_the_write,
    );
}

// ─── harness 档 `tests/an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor::the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries`）。
#[test]
fn the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries",
        scenarios_of_an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor::the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries,
    );
}

// ─── harness 档 `tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_readable_allocation_record_tree_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31`）。
#[test]
fn c393_readable_allocation_record_tree_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "c393_readable_allocation_record_tree_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31",
        scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_readable_allocation_record_tree_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_readable_tree_table_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31`）。
#[test]
fn c393_readable_tree_table_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31() {
    judge_the_images_a_harness_scenario_hands_out(
        "c393_readable_tree_table_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31",
        scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_readable_tree_table_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_allocation_record_tree_unreadable_on_every_read`）。
#[test]
fn c393_allocation_record_tree_unreadable_on_every_read() {
    judge_the_images_a_harness_scenario_hands_out(
        "c393_allocation_record_tree_unreadable_on_every_read",
        scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_allocation_record_tree_unreadable_on_every_read,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_allocation_record_tree_unreadable_until_the_reread_hook`）。
#[test]
fn c393_allocation_record_tree_unreadable_until_the_reread_hook() {
    judge_the_images_a_harness_scenario_hands_out(
        "c393_allocation_record_tree_unreadable_until_the_reread_hook",
        scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_allocation_record_tree_unreadable_until_the_reread_hook,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_allocation_record_tree_unreadable_only_on_the_first_read_of_each_copy`）。
#[test]
fn c393_allocation_record_tree_unreadable_only_on_the_first_read_of_each_copy() {
    judge_the_images_a_harness_scenario_hands_out(
        "c393_allocation_record_tree_unreadable_only_on_the_first_read_of_each_copy",
        scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_allocation_record_tree_unreadable_only_on_the_first_read_of_each_copy,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_tree_table_unreadable_on_every_read`）。
#[test]
fn c393_tree_table_unreadable_on_every_read() {
    judge_the_images_a_harness_scenario_hands_out(
        "c393_tree_table_unreadable_on_every_read",
        scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_tree_table_unreadable_on_every_read,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_tree_table_unreadable_until_the_reread_hook`）。
#[test]
fn c393_tree_table_unreadable_until_the_reread_hook() {
    judge_the_images_a_harness_scenario_hands_out(
        "c393_tree_table_unreadable_until_the_reread_hook",
        scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_tree_table_unreadable_until_the_reread_hook,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_tree_table_unreadable_only_on_the_first_read_of_each_copy`）。
#[test]
fn c393_tree_table_unreadable_only_on_the_first_read_of_each_copy() {
    judge_the_images_a_harness_scenario_hands_out(
        "c393_tree_table_unreadable_only_on_the_first_read_of_each_copy",
        scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_tree_table_unreadable_only_on_the_first_read_of_each_copy,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_checker_on_a_persistently_damaged_allocation_record_tree_without_reuse`）。
#[test]
fn c393_checker_on_a_persistently_damaged_allocation_record_tree_without_reuse() {
    judge_the_images_a_harness_scenario_hands_out(
        "c393_checker_on_a_persistently_damaged_allocation_record_tree_without_reuse",
        scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_checker_on_a_persistently_damaged_allocation_record_tree_without_reuse,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_checker_on_a_persistently_damaged_tree_table_without_reuse`）。
#[test]
fn c393_checker_on_a_persistently_damaged_tree_table_without_reuse() {
    judge_the_images_a_harness_scenario_hands_out(
        "c393_checker_on_a_persistently_damaged_tree_table_without_reuse",
        scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_checker_on_a_persistently_damaged_tree_table_without_reuse,
    );
}

// ─── harness 档 `tests/container_trees_grow_a_level_when_the_root_is_full.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry`）。
#[test]
fn a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry",
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place`）；
/// 旋钮栏是写这张镜像时那一组（墓碑与谱系各一片 1 条）。
#[test]
fn the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place() {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
        knobs_of_the_tombstone_and_the_lineage_container_at,
        TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    };
    judge_the_images_a_harness_scenario_hands_out_under_the_container_capacity_knobs(
        "the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place",
        knobs_of_the_tombstone_and_the_lineage_container_at(
            TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
        ),
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place,
    );
}

/// 与 harness 档同名的用例同一个场景（见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::the_fifth_tombstone_container_grows_the_tree_to_three_levels_and_rebuild_reads_it`）；
/// 旋钮栏是写这张镜像时那一组。
#[test]
fn the_fifth_tombstone_container_grows_the_tree_to_three_levels_and_rebuild_reads_it() {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
        knobs_of_the_tombstone_and_the_lineage_container_at,
        TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    };
    judge_the_images_a_harness_scenario_hands_out_under_the_container_capacity_knobs(
        "the_fifth_tombstone_container_grows_the_tree_to_three_levels_and_rebuild_reads_it",
        knobs_of_the_tombstone_and_the_lineage_container_at(
            TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
        ),
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::the_fifth_tombstone_container_grows_the_tree_to_three_levels_and_rebuild_reads_it,
    );
}

/// 与 harness 档同名的用例同一个场景（见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout`）；
/// 旋钮栏是写这张镜像时那一组（容器容量照旧压着，扇出换回产品）。
#[test]
fn a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout() {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
        knobs_of_the_tombstone_and_the_lineage_container_at,
        TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    };
    judge_the_images_a_harness_scenario_hands_out_under_the_container_capacity_knobs(
        "a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout",
        knobs_of_the_tombstone_and_the_lineage_container_at(
            TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
        ),
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout,
    );
}

/// 与 harness 档同名的用例同一个场景（见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::the_third_rollback_grows_the_lineage_tree_and_the_checkpoint_cost_counts_the_internal_level`）；
/// 旋钮栏是写这张镜像时那一组。
#[test]
fn the_third_rollback_grows_the_lineage_tree_and_the_checkpoint_cost_counts_the_internal_level() {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
        knobs_of_the_tombstone_and_the_lineage_container_at,
        TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    };
    judge_the_images_a_harness_scenario_hands_out_under_the_container_capacity_knobs(
        "the_third_rollback_grows_the_lineage_tree_and_the_checkpoint_cost_counts_the_internal_level",
        knobs_of_the_tombstone_and_the_lineage_container_at(
            TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
        ),
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::the_third_rollback_grows_the_lineage_tree_and_the_checkpoint_cost_counts_the_internal_level,
    );
}

/// 与 harness 档同名的用例同一个场景（见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant`）：
/// 旋钮栏只填墓碑容器旋钮值 1（原先 `check_pool_image_given_the_tombstone_container_capacity_knob` 那一判）。
#[test]
fn a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant() {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
        knobs_of_the_tombstone_container_only_at, TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    };
    judge_the_images_a_harness_scenario_hands_out_under_the_container_capacity_knobs(
        "a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant",
        knobs_of_the_tombstone_container_only_at(TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB),
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant,
    );
}

/// 与 harness 档同名的用例同一个场景（见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant`）：
/// 同一张镜像带三组旋钮值各判一趟（`KnobsTheTombstoneContainerOfTwoRecordsIsJudgedUnder` 每一种一趟，场景名接上那一组的名字）。
#[test]
fn a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant(
) {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::KnobsTheTombstoneContainerOfTwoRecordsIsJudgedUnder;
    for judged_under in KnobsTheTombstoneContainerOfTwoRecordsIsJudgedUnder::ALL {
        judge_the_images_a_harness_scenario_hands_out_under_the_container_capacity_knobs(
            &format!(
                "a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant::{}",
                judged_under.name()
            ),
            judged_under.knobs(),
            |images| {
                scenarios_of_container_trees_grow_a_level_when_the_root_is_full::a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant(
                    images,
                    judged_under,
                );
            },
        );
    }
}

/// 随机历史每一步的镜像带同一组旋钮值判（[`judge_the_images_a_harness_scenario_hands_out_under_the_container_capacity_knobs`] 那几样逐张核），
/// 再在全部镜像合起来的结论上核 `held_at_some_step` 每一条在某一步上判成立（判到且不红：这一趟现判，判到位作数）。
fn judge_every_step_image_of_a_random_history_under_the_knobs_and_find_each_knob_invariant_held_at_some_step(
    scenario_name: &str,
    container_capacity_knobs: ContainerCapacityKnobs,
    held_at_some_step: &[&'static str],
    scenario: impl FnOnce(&mut PoolCheckerImages),
) {
    let mut handed_out = PoolCheckerImages::collected_for_the_checker_tier();
    scenario(&mut handed_out);
    let images = handed_out.into_collected();
    assert!(
        !images.is_empty(),
        "场景 {scenario_name} 一张镜像都没交给池级 checker"
    );
    let Some((conclusions, run, judged_afresh)) = judge_the_collected_images(
        scenario_name,
        &images,
        container_capacity_knobs,
        JudgedAfreshRegardlessOfTheExpectations::Yes,
    ) else {
        return;
    };
    let mismatches = mismatches_of_the_images(&images, &conclusions, judged_afresh);
    assert!(
        mismatches.is_empty(),
        "场景 {scenario_name} 交出的镜像上池级 checker 判得与场景写下的对不上（引擎 {:?}）：\n{}",
        run.tally,
        mismatches.join("\n")
    );
    for invariant in held_at_some_step {
        assert!(
            conclusions
                .iter()
                .any(|judged| judged.states.iter().any(|conclusion| {
                    is_judged(conclusion, invariant) && !is_violated(conclusion, invariant)
                })),
            "场景 {scenario_name}：{invariant} 在某一步上真被评估过且成立（带旋钮值判）"
        );
    }
}

/// 与 harness 档同名的用例同一个场景（见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step`）：
/// 每一步的镜像旋钮栏填三棵树都是一片 1 条，一条都不红；I-9.22、I-9.37 在某一步上判成立。
#[test]
fn a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step() {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
        knobs_of_every_container_tree_at, KNOB_INVARIANTS_HELD_AT_SOME_STEP,
    };
    judge_every_step_image_of_a_random_history_under_the_knobs_and_find_each_knob_invariant_held_at_some_step(
        "a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step",
        knobs_of_every_container_tree_at(1),
        &KNOB_INVARIANTS_HELD_AT_SOME_STEP,
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step,
    );
}

/// 与 harness 档同名的用例同一个场景（见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::a_random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step`）：
/// 每一步的镜像旋钮栏填三棵树都是一片 2 条，一条都不红；I-9.22、I-9.37 在某一步上判成立。
#[test]
fn a_random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step() {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
        knobs_of_every_container_tree_at, KNOB_INVARIANTS_HELD_AT_SOME_STEP,
    };
    judge_every_step_image_of_a_random_history_under_the_knobs_and_find_each_knob_invariant_held_at_some_step(
        "a_random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step",
        knobs_of_every_container_tree_at(2),
        &KNOB_INVARIANTS_HELD_AT_SOME_STEP,
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::a_random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step,
    );
}

// ─── harness 档 `tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_device_left_with_one_self_verified_system_configuration_slot_is_not_refused_by_any_entry`）。
#[test]
fn a_device_left_with_one_self_verified_system_configuration_slot_is_not_refused_by_any_entry() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_device_left_with_one_self_verified_system_configuration_slot_is_not_refused_by_any_entry",
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_device_left_with_one_self_verified_system_configuration_slot_is_not_refused_by_any_entry,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused`）。
#[test]
fn a_session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused",
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot`）。
#[test]
fn a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot",
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot`）。
#[test]
fn a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot",
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot`）。
#[test]
fn an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot",
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot`）。
#[test]
fn a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot",
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot`）。
#[test]
fn a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot",
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot`）。
#[test]
fn an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot",
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot,
    );
}

// ─── harness 档 `tests/fault_injection_fast_tier.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/fault_injection_fast_tier.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_fault_injection_fast_tier::publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root`）。
#[test]
fn publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root",
        scenarios_of_fault_injection_fast_tier::publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root,
    );
}

// ─── harness 档 `tests/fsync_drop_and_devices_without_the_selected_version.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_fsync_drop_and_devices_without_the_selected_version::fsync_drop_one_shot`）。
#[test]
fn fsync_drop_one_shot() {
    judge_the_images_a_harness_scenario_hands_out(
        "fsync_drop_one_shot",
        scenarios_of_fsync_drop_and_devices_without_the_selected_version::fsync_drop_one_shot,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_fsync_drop_and_devices_without_the_selected_version::fsync_drop_persists_until_reopen`）。
#[test]
fn fsync_drop_persists_until_reopen() {
    judge_the_images_a_harness_scenario_hands_out(
        "fsync_drop_persists_until_reopen",
        scenarios_of_fsync_drop_and_devices_without_the_selected_version::fsync_drop_persists_until_reopen,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_fsync_drop_and_devices_without_the_selected_version::fsync_drop_persists_across_reopen`）。
#[test]
fn fsync_drop_persists_across_reopen() {
    judge_the_images_a_harness_scenario_hands_out(
        "fsync_drop_persists_across_reopen",
        scenarios_of_fsync_drop_and_devices_without_the_selected_version::fsync_drop_persists_across_reopen,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_fsync_drop_and_devices_without_the_selected_version::mount_writable_with_a_blank_device_one_refuses_or_keeps_two_copies`）。
#[test]
fn mount_writable_with_a_blank_device_one_refuses_or_keeps_two_copies() {
    judge_the_images_a_harness_scenario_hands_out_when_it_hands_out_none(
        "mount_writable_with_a_blank_device_one_refuses_or_keeps_two_copies",
        scenarios_of_fsync_drop_and_devices_without_the_selected_version::mount_writable_with_a_blank_device_one_refuses_or_keeps_two_copies,
        WhenTheScenarioHandsOutNoImage::IsExpectedOnTheRefusingPath,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_fsync_drop_and_devices_without_the_selected_version::mount_writable_with_a_stale_device_one_refuses_or_keeps_two_copies`）。
#[test]
fn mount_writable_with_a_stale_device_one_refuses_or_keeps_two_copies() {
    judge_the_images_a_harness_scenario_hands_out_when_it_hands_out_none(
        "mount_writable_with_a_stale_device_one_refuses_or_keeps_two_copies",
        scenarios_of_fsync_drop_and_devices_without_the_selected_version::mount_writable_with_a_stale_device_one_refuses_or_keeps_two_copies,
        WhenTheScenarioHandsOutNoImage::IsExpectedOnTheRefusingPath,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_fsync_drop_and_devices_without_the_selected_version::device_one_behind_only_by_the_rotation_it_missed_still_mounts_writable_with_two_copies`）。
#[test]
fn device_one_behind_only_by_the_rotation_it_missed_still_mounts_writable_with_two_copies() {
    judge_the_images_a_harness_scenario_hands_out(
        "device_one_behind_only_by_the_rotation_it_missed_still_mounts_writable_with_two_copies",
        scenarios_of_fsync_drop_and_devices_without_the_selected_version::device_one_behind_only_by_the_rotation_it_missed_still_mounts_writable_with_two_copies,
    );
}

// ─── harness 档 `tests/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one`）。
#[test]
fn the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one",
        scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing`）。
#[test]
fn a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing()
{
    judge_the_images_a_harness_scenario_hands_out(
        "a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing",
        scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it`）。
#[test]
fn the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it",
        scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it,
    );
}

// ─── harness 档 `tests/inode_number_watermark_pressed_back_reissues_a_number.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant`）。
#[test]
fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant",
        scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant`）。
#[test]
fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant",
        scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants`）。
#[test]
fn pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants",
        scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants,
    );
}

// ─── harness 档 `tests/instance_table_chain_read.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/instance_table_chain_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_instance_table_chain_read::the_pool_checker_walks_both_pages_of_a_valid_chain_and_every_invariant_holds`）。
#[test]
fn the_pool_checker_walks_both_pages_of_a_valid_chain_and_every_invariant_holds() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_pool_checker_walks_both_pages_of_a_valid_chain_and_every_invariant_holds",
        scenarios_of_instance_table_chain_read::the_pool_checker_walks_both_pages_of_a_valid_chain_and_every_invariant_holds,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/instance_table_chain_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_instance_table_chain_read::chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant`）。
#[test]
fn chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant() {
    judge_the_images_a_harness_scenario_hands_out(
        "chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant",
        scenarios_of_instance_table_chain_read::chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/instance_table_chain_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_instance_table_chain_read::chain_pointer_whose_checksum_does_not_match_the_second_page_reddens_the_checksum_invariant`）。
#[test]
fn chain_pointer_whose_checksum_does_not_match_the_second_page_reddens_the_checksum_invariant() {
    judge_the_images_a_harness_scenario_hands_out(
        "chain_pointer_whose_checksum_does_not_match_the_second_page_reddens_the_checksum_invariant",
        scenarios_of_instance_table_chain_read::chain_pointer_whose_checksum_does_not_match_the_second_page_reddens_the_checksum_invariant,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/instance_table_chain_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_instance_table_chain_read::the_same_instance_on_both_pages_reddens_only_the_instance_table_invariant`）。
#[test]
fn the_same_instance_on_both_pages_reddens_only_the_instance_table_invariant() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_same_instance_on_both_pages_reddens_only_the_instance_table_invariant",
        scenarios_of_instance_table_chain_read::the_same_instance_on_both_pages_reddens_only_the_instance_table_invariant,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/instance_table_chain_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_instance_table_chain_read::contradictory_chain_record_reddens_only_the_instance_table_invariant`）。
#[test]
fn contradictory_chain_record_reddens_only_the_instance_table_invariant() {
    judge_the_images_a_harness_scenario_hands_out(
        "contradictory_chain_record_reddens_only_the_instance_table_invariant",
        scenarios_of_instance_table_chain_read::contradictory_chain_record_reddens_only_the_instance_table_invariant,
    );
}

// ─── harness 档 `tests/instance_table_page_full.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/instance_table_page_full.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_instance_table_page_full::writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page`）。
#[test]
fn writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page",
        scenarios_of_instance_table_page_full::writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page,
    );
}

// ─── harness 档 `tests/instance_table_second_page_write.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/instance_table_second_page_write.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_instance_table_second_page_write::cold_start_reads_back_every_row_the_writer_put_on_two_pages_and_the_pool_checker_stays_green`）。
#[test]
fn cold_start_reads_back_every_row_the_writer_put_on_two_pages_and_the_pool_checker_stays_green() {
    judge_the_images_a_harness_scenario_hands_out(
        "cold_start_reads_back_every_row_the_writer_put_on_two_pages_and_the_pool_checker_stays_green",
        scenarios_of_instance_table_second_page_write::cold_start_reads_back_every_row_the_writer_put_on_two_pages_and_the_pool_checker_stays_green,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/instance_table_second_page_write.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_instance_table_second_page_write::the_next_mount_rewrites_the_whole_two_page_chain_and_releases_both_old_pages`）。
#[test]
fn the_next_mount_rewrites_the_whole_two_page_chain_and_releases_both_old_pages() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_next_mount_rewrites_the_whole_two_page_chain_and_releases_both_old_pages",
        scenarios_of_instance_table_second_page_write::the_next_mount_rewrites_the_whole_two_page_chain_and_releases_both_old_pages,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/instance_table_second_page_write.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_instance_table_second_page_write::version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both`）。
#[test]
fn version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both() {
    judge_the_images_a_harness_scenario_hands_out(
        "version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both",
        scenarios_of_instance_table_second_page_write::version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both,
    );
}

// ─── harness 档 `tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::the_row_of_a_dead_instance_is_deleted_after_its_roots_left_the_ring_and_its_unpublished_heads_are_erased`）。
#[test]
fn the_row_of_a_dead_instance_is_deleted_after_its_roots_left_the_ring_and_its_unpublished_heads_are_erased(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_row_of_a_dead_instance_is_deleted_after_its_roots_left_the_ring_and_its_unpublished_heads_are_erased",
        scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::the_row_of_a_dead_instance_is_deleted_after_its_roots_left_the_ring_and_its_unpublished_heads_are_erased,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::the_background_sweep_rereads_every_head_before_erasing_and_deletes_no_row_when_a_slot_was_allocated_after_the_judgement`）。
#[test]
fn the_background_sweep_rereads_every_head_before_erasing_and_deletes_no_row_when_a_slot_was_allocated_after_the_judgement(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_background_sweep_rereads_every_head_before_erasing_and_deletes_no_row_when_a_slot_was_allocated_after_the_judgement",
        scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::the_background_sweep_rereads_every_head_before_erasing_and_deletes_no_row_when_a_slot_was_allocated_after_the_judgement,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::a_device_error_on_the_reread_before_erasing_deletes_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance`）。
#[test]
fn a_device_error_on_the_reread_before_erasing_deletes_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_device_error_on_the_reread_before_erasing_deletes_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance",
        scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::a_device_error_on_the_reread_before_erasing_deletes_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::zeros_read_back_on_the_reread_before_erasing_delete_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance`）。
#[test]
fn zeros_read_back_on_the_reread_before_erasing_delete_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "zeros_read_back_on_the_reread_before_erasing_delete_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance",
        scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::zeros_read_back_on_the_reread_before_erasing_delete_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::sweeping_a_periodic_schedule_of_unmounted_and_crashed_mounts_keeps_the_instance_table_short`）。
#[test]
fn sweeping_a_periodic_schedule_of_unmounted_and_crashed_mounts_keeps_the_instance_table_short() {
    judge_the_images_a_harness_scenario_hands_out(
        "sweeping_a_periodic_schedule_of_unmounted_and_crashed_mounts_keeps_the_instance_table_short",
        scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::sweeping_a_periodic_schedule_of_unmounted_and_crashed_mounts_keeps_the_instance_table_short,
    );
}

// ─── harness 档 `tests/journal_last_record_flag_of_a_multi_record_publish.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/journal_last_record_flag_of_a_multi_record_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_journal_last_record_flag_of_a_multi_record_publish::with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens`）。
#[test]
fn with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens",
        scenarios_of_journal_last_record_flag_of_a_multi_record_publish::with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/journal_last_record_flag_of_a_multi_record_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_journal_last_record_flag_of_a_multi_record_publish::last_record_flag_followed_by_the_record_of_the_same_publish_breaks_the_chain_at_the_flag_and_the_checker_reddens`）。
#[test]
fn last_record_flag_followed_by_the_record_of_the_same_publish_breaks_the_chain_at_the_flag_and_the_checker_reddens(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "last_record_flag_followed_by_the_record_of_the_same_publish_breaks_the_chain_at_the_flag_and_the_checker_reddens",
        scenarios_of_journal_last_record_flag_of_a_multi_record_publish::last_record_flag_followed_by_the_record_of_the_same_publish_breaks_the_chain_at_the_flag_and_the_checker_reddens,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/journal_last_record_flag_of_a_multi_record_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_journal_last_record_flag_of_a_multi_record_publish::last_record_flag_followed_only_by_the_very_next_record_of_the_same_publish_breaks_the_chain_at_the_flag`）。
#[test]
fn last_record_flag_followed_only_by_the_very_next_record_of_the_same_publish_breaks_the_chain_at_the_flag(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "last_record_flag_followed_only_by_the_very_next_record_of_the_same_publish_breaks_the_chain_at_the_flag",
        scenarios_of_journal_last_record_flag_of_a_multi_record_publish::last_record_flag_followed_only_by_the_very_next_record_of_the_same_publish_breaks_the_chain_at_the_flag,
    );
}

// ─── harness 档 `tests/many_inodes_write_and_read.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/many_inodes_write_and_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_many_inodes_write_and_read::creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold`）。
#[test]
fn creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold",
        scenarios_of_many_inodes_write_and_read::creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/many_inodes_write_and_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_many_inodes_write_and_read::creating_one_inode_below_the_threshold_keeps_one_container_and_bumps_the_watermark_by_one`）。
#[test]
fn creating_one_inode_below_the_threshold_keeps_one_container_and_bumps_the_watermark_by_one() {
    judge_the_images_a_harness_scenario_hands_out(
        "creating_one_inode_below_the_threshold_keeps_one_container_and_bumps_the_watermark_by_one",
        scenarios_of_many_inodes_write_and_read::creating_one_inode_below_the_threshold_keeps_one_container_and_bumps_the_watermark_by_one,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/many_inodes_write_and_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_many_inodes_write_and_read::publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish`）。
#[test]
fn publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish",
        scenarios_of_many_inodes_write_and_read::publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish,
    );
}

// ─── harness 档 `tests/multi_record_publishes_of_transaction_zero.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/multi_record_publishes_of_transaction_zero.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_multi_record_publishes_of_transaction_zero::row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it`）。
#[test]
fn row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it",
        scenarios_of_multi_record_publishes_of_transaction_zero::row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/multi_record_publishes_of_transaction_zero.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_multi_record_publishes_of_transaction_zero::commit_marker_other_than_zero_or_one_counts_as_torn_and_breaks_the_chain_and_the_checker_reddens`）。
#[test]
fn commit_marker_other_than_zero_or_one_counts_as_torn_and_breaks_the_chain_and_the_checker_reddens(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "commit_marker_other_than_zero_or_one_counts_as_torn_and_breaks_the_chain_and_the_checker_reddens",
        scenarios_of_multi_record_publishes_of_transaction_zero::commit_marker_other_than_zero_or_one_counts_as_torn_and_breaks_the_chain_and_the_checker_reddens,
    );
}

// ─── harness 档 `tests/multi_unit_file_write_and_read.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/multi_unit_file_write_and_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_multi_unit_file_write_and_read::units_written_across_the_sixty_four_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte`）。
#[test]
fn units_written_across_the_sixty_four_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "units_written_across_the_sixty_four_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte",
        scenarios_of_multi_unit_file_write_and_read::units_written_across_the_sixty_four_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/multi_unit_file_write_and_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_multi_unit_file_write_and_read::the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline`）。
#[test]
fn the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline",
        scenarios_of_multi_unit_file_write_and_read::the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/multi_unit_file_write_and_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_multi_unit_file_write_and_read::shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has`）。
#[test]
fn shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has() {
    judge_the_images_a_harness_scenario_hands_out(
        "shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has",
        scenarios_of_multi_unit_file_write_and_read::shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/multi_unit_file_write_and_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_multi_unit_file_write_and_read::reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping`）。
#[test]
fn reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping",
        scenarios_of_multi_unit_file_write_and_read::reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping,
    );
}

// ─── harness 档 `tests/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_named_unit_verification_stands_in_for_the_barrier_before_the_record::with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations`）。
#[test]
fn with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations",
        scenarios_of_named_unit_verification_stands_in_for_the_barrier_before_the_record::with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations,
    );
}

// ─── harness 档 `tests/one_revolution_window_after_a_hole_in_the_root_ring.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated`）。
#[test]
fn reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated",
        scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::a_remount_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated`）。
#[test]
fn a_remount_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_remount_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated",
        scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::a_remount_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::raising_the_floor_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated`）。
#[test]
fn raising_the_floor_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated()
{
    judge_the_images_a_harness_scenario_hands_out(
        "raising_the_floor_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated",
        scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::raising_the_floor_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::a_remount_reading_the_slot_of_the_stale_root_left_in_the_hole_bad_keeps_every_unit_of_it_allocated`）。
#[test]
fn a_remount_reading_the_slot_of_the_stale_root_left_in_the_hole_bad_keeps_every_unit_of_it_allocated(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_remount_reading_the_slot_of_the_stale_root_left_in_the_hole_bad_keeps_every_unit_of_it_allocated",
        scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::a_remount_reading_the_slot_of_the_stale_root_left_in_the_hole_bad_keeps_every_unit_of_it_allocated,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_stays_unreadable_keeps_every_unit_of_it_allocated`）。
#[test]
fn raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_stays_unreadable_keeps_every_unit_of_it_allocated(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_stays_unreadable_keeps_every_unit_of_it_allocated",
        scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_stays_unreadable_keeps_every_unit_of_it_allocated,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated`）。
#[test]
fn raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated",
        scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated,
    );
}

// ─── harness 档 `tests/overwrite_in_one_instance.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/overwrite_in_one_instance.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_overwrite_in_one_instance::cold_start_reads_the_second_content_and_the_pool_checker_stays_green`）。
#[test]
fn cold_start_reads_the_second_content_and_the_pool_checker_stays_green() {
    judge_the_images_a_harness_scenario_hands_out(
        "cold_start_reads_the_second_content_and_the_pool_checker_stays_green",
        scenarios_of_overwrite_in_one_instance::cold_start_reads_the_second_content_and_the_pool_checker_stays_green,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/overwrite_in_one_instance.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_overwrite_in_one_instance::repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree`）。
#[test]
fn repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree",
        scenarios_of_overwrite_in_one_instance::repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/overwrite_in_one_instance.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_overwrite_in_one_instance::allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records`）。
#[test]
fn allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records",
        scenarios_of_overwrite_in_one_instance::allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records,
    );
}

// ─── harness 档 `tests/publish_failure_resent_unchanged.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/publish_failure_resent_unchanged.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_publish_failure_resent_unchanged::publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish`）。
#[test]
fn publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish() {
    judge_the_images_a_harness_scenario_hands_out(
        "publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish",
        scenarios_of_publish_failure_resent_unchanged::publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish,
    );
}

// ─── harness 档 `tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds`）。
#[test]
fn unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements`）。
#[test]
fn session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds`）。
#[test]
fn unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements`）。
#[test]
fn session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot`）。
#[test]
fn unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again`）。
#[test]
fn units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_while_its_account_still_reads_back_zero`）。
#[test]
fn units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_while_its_account_still_reads_back_zero(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_while_its_account_still_reads_back_zero",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_while_its_account_still_reads_back_zero,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written`）。
#[test]
fn units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount`）。
#[test]
fn units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount`）。
#[test]
fn units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount`）。
#[test]
fn units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount`）。
#[test]
fn units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount`）。
#[test]
fn units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices`）。
#[test]
fn wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero`）。
#[test]
fn revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through`）。
#[test]
fn a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through`）。
#[test]
fn an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through`）。
#[test]
fn a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through`）。
#[test]
fn unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write`）。
#[test]
fn pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write() {
    judge_the_images_a_harness_scenario_hands_out(
        "pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way`）。
#[test]
fn ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed`）。
#[test]
fn defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push`）。
#[test]
fn a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through`）。
#[test]
fn a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_write_refused_even_without_the_wide_protection_lists_only_unmounting_which_lets_it_through`）。
#[test]
fn a_write_refused_even_without_the_wide_protection_lists_only_unmounting_which_lets_it_through() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_write_refused_even_without_the_wide_protection_lists_only_unmounting_which_lets_it_through",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_write_refused_even_without_the_wide_protection_lists_only_unmounting_which_lets_it_through,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting`）。
#[test]
fn a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_eighty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting`）。
#[test]
fn an_eighty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting() {
    judge_the_images_a_harness_scenario_hands_out(
        "an_eighty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_eighty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_sixty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting`）。
#[test]
fn a_sixty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_sixty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_sixty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_eighty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting`）。
#[test]
fn an_eighty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "an_eighty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_eighty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_sixty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting`）。
#[test]
fn a_sixty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_sixty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_sixty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_eighty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting`）。
#[test]
fn an_eighty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting() {
    judge_the_images_a_harness_scenario_hands_out(
        "an_eighty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_eighty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refused_as_frozen_before_any_write`）。
#[test]
fn pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refused_as_frozen_before_any_write(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refused_as_frozen_before_any_write",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refused_as_frozen_before_any_write,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_the_two_device_bodies_swapped_is_refused_before_any_write`）。
#[test]
fn pushing_the_root_ring_with_the_two_device_bodies_swapped_is_refused_before_any_write() {
    judge_the_images_a_harness_scenario_hands_out(
        "pushing_the_root_ring_with_the_two_device_bodies_swapped_is_refused_before_any_write",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_the_two_device_bodies_swapped_is_refused_before_any_write,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::no_space_while_the_ring_slots_hold_placements_none_of_which_is_isolated_is_not_reported_as_the_wide_protection`）。
#[test]
fn no_space_while_the_ring_slots_hold_placements_none_of_which_is_isolated_is_not_reported_as_the_wide_protection(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "no_space_while_the_ring_slots_hold_placements_none_of_which_is_isolated_is_not_reported_as_the_wide_protection",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::no_space_while_the_ring_slots_hold_placements_none_of_which_is_isolated_is_not_reported_as_the_wide_protection,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_while_the_ring_slots_hold_placements_none_of_which_is_isolated_writes_nothing`）。
#[test]
fn pushing_the_root_ring_while_the_ring_slots_hold_placements_none_of_which_is_isolated_writes_nothing(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "pushing_the_root_ring_while_the_ring_slots_hold_placements_none_of_which_is_isolated_writes_nothing",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_while_the_ring_slots_hold_placements_none_of_which_is_isolated_writes_nothing,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to`）。
#[test]
fn a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both`）。
#[test]
fn pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_after_the_session_moved_past_the_version_refused_on_is_refused_without_reading_a_device`）。
#[test]
fn asking_for_the_ways_out_after_the_session_moved_past_the_version_refused_on_is_refused_without_reading_a_device(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "asking_for_the_ways_out_after_the_session_moved_past_the_version_refused_on_is_refused_without_reading_a_device",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_after_the_session_moved_past_the_version_refused_on_is_refused_without_reading_a_device,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_with_the_two_devices_in_reversed_order_is_refused_without_reading_a_device`）。
#[test]
fn asking_for_the_ways_out_with_the_two_devices_in_reversed_order_is_refused_without_reading_a_device(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "asking_for_the_ways_out_with_the_two_devices_in_reversed_order_is_refused_without_reading_a_device",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_with_the_two_devices_in_reversed_order_is_refused_without_reading_a_device,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_with_one_device_more_than_the_mount_is_refused_without_reading_a_device`）。
#[test]
fn asking_for_the_ways_out_with_one_device_more_than_the_mount_is_refused_without_reading_a_device()
{
    judge_the_images_a_harness_scenario_hands_out(
        "asking_for_the_ways_out_with_one_device_more_than_the_mount_is_refused_without_reading_a_device",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_with_one_device_more_than_the_mount_is_refused_without_reading_a_device,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_with_one_device_fewer_than_the_mount_is_refused_without_reading_a_device`）。
#[test]
fn asking_for_the_ways_out_with_one_device_fewer_than_the_mount_is_refused_without_reading_a_device(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "asking_for_the_ways_out_with_one_device_fewer_than_the_mount_is_refused_without_reading_a_device",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_with_one_device_fewer_than_the_mount_is_refused_without_reading_a_device,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_a_device_table_other_than_the_mount_in_reversed_order_is_refused_without_reading_a_device`）。
#[test]
fn pushing_the_root_ring_with_a_device_table_other_than_the_mount_in_reversed_order_is_refused_without_reading_a_device(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "pushing_the_root_ring_with_a_device_table_other_than_the_mount_in_reversed_order_is_refused_without_reading_a_device",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_a_device_table_other_than_the_mount_in_reversed_order_is_refused_without_reading_a_device,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_more_is_refused_without_reading_a_device`）。
#[test]
fn pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_more_is_refused_without_reading_a_device(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_more_is_refused_without_reading_a_device",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_more_is_refused_without_reading_a_device,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_fewer_is_refused_without_reading_a_device`）。
#[test]
fn pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_fewer_is_refused_without_reading_a_device(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_fewer_is_refused_without_reading_a_device",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_fewer_is_refused_without_reading_a_device,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_after_the_session_moved_with_the_two_devices_in_reversed_order_is_refused_as_the_session_moved`）。
#[test]
fn asking_for_the_ways_out_after_the_session_moved_with_the_two_devices_in_reversed_order_is_refused_as_the_session_moved(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "asking_for_the_ways_out_after_the_session_moved_with_the_two_devices_in_reversed_order_is_refused_as_the_session_moved",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_after_the_session_moved_with_the_two_devices_in_reversed_order_is_refused_as_the_session_moved,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_after_unmounting_and_mounting_again_with_the_report_from_before_is_refused_as_the_session_moved_naming_the_new_instance`）。
#[test]
fn asking_for_the_ways_out_after_unmounting_and_mounting_again_with_the_report_from_before_is_refused_as_the_session_moved_naming_the_new_instance(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "asking_for_the_ways_out_after_unmounting_and_mounting_again_with_the_report_from_before_is_refused_as_the_session_moved_naming_the_new_instance",
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_after_unmounting_and_mounting_again_with_the_report_from_before_is_refused_as_the_session_moved_naming_the_new_instance,
    );
}

// ─── harness 档 `tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window::admission_raising_the_floor_above_holes_at_the_oldest_end_of_the_window_takes_its_ceiling_only_from_roots_inside_the_window`）。
#[test]
fn admission_raising_the_floor_above_holes_at_the_oldest_end_of_the_window_takes_its_ceiling_only_from_roots_inside_the_window(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "admission_raising_the_floor_above_holes_at_the_oldest_end_of_the_window_takes_its_ceiling_only_from_roots_inside_the_window",
        scenarios_of_raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window::admission_raising_the_floor_above_holes_at_the_oldest_end_of_the_window_takes_its_ceiling_only_from_roots_inside_the_window,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window::admission_raising_the_floor_keeps_the_root_ring_slot_unreadable_since_the_mount_pinned_at_the_oldest_txg_it_can_have_held`）。
#[test]
fn admission_raising_the_floor_keeps_the_root_ring_slot_unreadable_since_the_mount_pinned_at_the_oldest_txg_it_can_have_held(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "admission_raising_the_floor_keeps_the_root_ring_slot_unreadable_since_the_mount_pinned_at_the_oldest_txg_it_can_have_held",
        scenarios_of_raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window::admission_raising_the_floor_keeps_the_root_ring_slot_unreadable_since_the_mount_pinned_at_the_oldest_txg_it_can_have_held,
    );
}

// ─── harness 档 `tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic::raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_every_invariant_green`）。
#[test]
fn raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_every_invariant_green() {
    judge_the_images_a_harness_scenario_hands_out(
        "raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_every_invariant_green",
        scenarios_of_raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic::raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_every_invariant_green,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic::raising_the_floor_past_the_gap_keeps_every_invariant_green`）。
#[test]
fn raising_the_floor_past_the_gap_keeps_every_invariant_green() {
    judge_the_images_a_harness_scenario_hands_out(
        "raising_the_floor_past_the_gap_keeps_every_invariant_green",
        scenarios_of_raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic::raising_the_floor_past_the_gap_keeps_every_invariant_green,
    );
}

// ─── harness 档 `tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::admission_raising_the_floor_to_the_ceiling_stops_at_a_slot_unreadable_since_the_mount`）。
#[test]
fn admission_raising_the_floor_to_the_ceiling_stops_at_a_slot_unreadable_since_the_mount() {
    judge_the_images_a_harness_scenario_hands_out(
        "admission_raising_the_floor_to_the_ceiling_stops_at_a_slot_unreadable_since_the_mount",
        scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::admission_raising_the_floor_to_the_ceiling_stops_at_a_slot_unreadable_since_the_mount,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::admission_raising_the_floor_to_the_ceiling_with_x_readable_reaches_x`）。
#[test]
fn admission_raising_the_floor_to_the_ceiling_with_x_readable_reaches_x() {
    judge_the_images_a_harness_scenario_hands_out(
        "admission_raising_the_floor_to_the_ceiling_with_x_readable_reaches_x",
        scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::admission_raising_the_floor_to_the_ceiling_with_x_readable_reaches_x,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::admission_raising_the_floor_over_the_holes_with_a_slot_reading_back_zeros_since_the_mount_stops_at_the_roots_left_in_the_holes`）。
#[test]
fn admission_raising_the_floor_over_the_holes_with_a_slot_reading_back_zeros_since_the_mount_stops_at_the_roots_left_in_the_holes(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "admission_raising_the_floor_over_the_holes_with_a_slot_reading_back_zeros_since_the_mount_stops_at_the_roots_left_in_the_holes",
        scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::admission_raising_the_floor_over_the_holes_with_a_slot_reading_back_zeros_since_the_mount_stops_at_the_roots_left_in_the_holes,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::admission_raising_the_floor_without_holes_stops_at_a_slot_unreadable_since_the_mount`）。
#[test]
fn admission_raising_the_floor_without_holes_stops_at_a_slot_unreadable_since_the_mount() {
    judge_the_images_a_harness_scenario_hands_out(
        "admission_raising_the_floor_without_holes_stops_at_a_slot_unreadable_since_the_mount",
        scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::admission_raising_the_floor_without_holes_stops_at_a_slot_unreadable_since_the_mount,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::admission_raising_the_floor_without_holes_passes_a_slot_reading_back_zeros_since_the_mount_as_the_known_cost`）。
#[test]
fn admission_raising_the_floor_without_holes_passes_a_slot_reading_back_zeros_since_the_mount_as_the_known_cost(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "admission_raising_the_floor_without_holes_passes_a_slot_reading_back_zeros_since_the_mount_as_the_known_cost",
        scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount::admission_raising_the_floor_without_holes_passes_a_slot_reading_back_zeros_since_the_mount_as_the_known_cost,
    );
}

// ─── harness 档 `tests/random_histories.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/random_histories.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_random_histories::crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units`）。
#[test]
fn crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units",
        scenarios_of_random_histories::crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units,
    );
}

// ─── harness 档 `tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read::only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over`）。
#[test]
fn only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read::only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read::a_remount_without_faults_does_not_reread_the_root_ring_and_reads_each_slot_as_often_as_before`）。
#[test]
fn a_remount_without_faults_does_not_reread_the_root_ring_and_reads_each_slot_as_often_as_before() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_remount_without_faults_does_not_reread_the_root_ring_and_reads_each_slot_as_often_as_before",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read::a_remount_without_faults_does_not_reread_the_root_ring_and_reads_each_slot_as_often_as_before,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read::a_reread_still_failing_on_device_one_hands_over_what_both_passes_failing_only_on_device_one_hand_over`）。
#[test]
fn a_reread_still_failing_on_device_one_hands_over_what_both_passes_failing_only_on_device_one_hand_over(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_reread_still_failing_on_device_one_hands_over_what_both_passes_failing_only_on_device_one_hand_over",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read::a_reread_still_failing_on_device_one_hands_over_what_both_passes_failing_only_on_device_one_hand_over,
    );
}

// ─── harness 档 `tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_192_slot_pool_after_a_crashed_unmount_device_zero_failing_the_rebuild_pass_rereads_and_mounts`）。
#[test]
fn on_the_192_slot_pool_after_a_crashed_unmount_device_zero_failing_the_rebuild_pass_rereads_and_mounts(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "on_the_192_slot_pool_after_a_crashed_unmount_device_zero_failing_the_rebuild_pass_rereads_and_mounts",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_192_slot_pool_after_a_crashed_unmount_device_zero_failing_the_rebuild_pass_rereads_and_mounts,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_192_slot_pool_after_a_crashed_unmount_device_one_failing_the_rebuild_pass_rereads_and_mounts`）。
#[test]
fn on_the_192_slot_pool_after_a_crashed_unmount_device_one_failing_the_rebuild_pass_rereads_and_mounts(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "on_the_192_slot_pool_after_a_crashed_unmount_device_one_failing_the_rebuild_pass_rereads_and_mounts",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_192_slot_pool_after_a_crashed_unmount_device_one_failing_the_rebuild_pass_rereads_and_mounts,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_192_slot_pool_after_a_crashed_unmount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts`）。
#[test]
fn on_the_192_slot_pool_after_a_crashed_unmount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "on_the_192_slot_pool_after_a_crashed_unmount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_192_slot_pool_after_a_crashed_unmount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_192_slot_pool_after_a_crashed_unmount_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts`）。
#[test]
fn on_the_192_slot_pool_after_a_crashed_unmount_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "on_the_192_slot_pool_after_a_crashed_unmount_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_192_slot_pool_after_a_crashed_unmount_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_216_slot_pool_crashed_after_its_continued_mount_device_zero_failing_the_rebuild_pass_rereads_and_mounts`）。
#[test]
fn on_the_216_slot_pool_crashed_after_its_continued_mount_device_zero_failing_the_rebuild_pass_rereads_and_mounts(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "on_the_216_slot_pool_crashed_after_its_continued_mount_device_zero_failing_the_rebuild_pass_rereads_and_mounts",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_216_slot_pool_crashed_after_its_continued_mount_device_zero_failing_the_rebuild_pass_rereads_and_mounts,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_failing_the_rebuild_pass_rereads_and_mounts`）。
#[test]
fn on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_failing_the_rebuild_pass_rereads_and_mounts(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_failing_the_rebuild_pass_rereads_and_mounts",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_failing_the_rebuild_pass_rereads_and_mounts,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts`）。
#[test]
fn on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_216_slot_pool_crashed_after_its_continued_mount_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts`）。
#[test]
fn on_the_216_slot_pool_crashed_after_its_continued_mount_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "on_the_216_slot_pool_crashed_after_its_continued_mount_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_216_slot_pool_crashed_after_its_continued_mount_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_264_slot_pool_with_a_live_file_device_zero_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before`）。
#[test]
fn on_the_264_slot_pool_with_a_live_file_device_zero_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "on_the_264_slot_pool_with_a_live_file_device_zero_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_264_slot_pool_with_a_live_file_device_zero_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_264_slot_pool_with_a_live_file_device_one_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before`）。
#[test]
fn on_the_264_slot_pool_with_a_live_file_device_one_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "on_the_264_slot_pool_with_a_live_file_device_one_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_264_slot_pool_with_a_live_file_device_one_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_264_slot_pool_with_a_live_file_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before`）。
#[test]
fn on_the_264_slot_pool_with_a_live_file_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "on_the_264_slot_pool_with_a_live_file_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_264_slot_pool_with_a_live_file_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_264_slot_pool_with_a_live_file_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before`）。
#[test]
fn on_the_264_slot_pool_with_a_live_file_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "on_the_264_slot_pool_with_a_live_file_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::on_the_264_slot_pool_with_a_live_file_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::a_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_does_not_refuse_the_floor_raise_of_the_narrow_mount`）。
#[test]
fn a_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_does_not_refuse_the_floor_raise_of_the_narrow_mount(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_does_not_refuse_the_floor_raise_of_the_narrow_mount",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::a_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_does_not_refuse_the_floor_raise_of_the_narrow_mount,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::a_root_self_verified_only_on_the_rebuild_pass_read_bad_once_more_after_the_reread_is_taken_as_read_by_the_floor_raise`）。
#[test]
fn a_root_self_verified_only_on_the_rebuild_pass_read_bad_once_more_after_the_reread_is_taken_as_read_by_the_floor_raise(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_root_self_verified_only_on_the_rebuild_pass_read_bad_once_more_after_the_reread_is_taken_as_read_by_the_floor_raise",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::a_root_self_verified_only_on_the_rebuild_pass_read_bad_once_more_after_the_reread_is_taken_as_read_by_the_floor_raise,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_fourth_newest_non_empty_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_still_sets_the_ceiling_of_the_narrow_mounts_floor_raise`）。
#[test]
fn the_fourth_newest_non_empty_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_still_sets_the_ceiling_of_the_narrow_mounts_floor_raise(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_fourth_newest_non_empty_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_still_sets_the_ceiling_of_the_narrow_mounts_floor_raise",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_fourth_newest_non_empty_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_still_sets_the_ceiling_of_the_narrow_mounts_floor_raise,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`）。
#[test]
fn the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`）。
#[test]
fn the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`）。
#[test]
fn the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`）。
#[test]
fn the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites`）。
#[test]
fn the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites`）。
#[test]
fn the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites`）。
#[test]
fn the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites`）。
#[test]
fn the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite`）。
#[test]
fn the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite`）。
#[test]
fn the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite`）。
#[test]
fn the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite`）。
#[test]
fn the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites`）。
#[test]
fn the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites`）。
#[test]
fn the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites`）。
#[test]
fn the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites`）。
#[test]
fn the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::a_reread_failing_on_the_whole_ring_after_a_first_pass_that_read_device_zero_mounts_on_the_first_pass_roots`）。
#[test]
fn a_reread_failing_on_the_whole_ring_after_a_first_pass_that_read_device_zero_mounts_on_the_first_pass_roots(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_reread_failing_on_the_whole_ring_after_a_first_pass_that_read_device_zero_mounts_on_the_first_pass_roots",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::a_reread_failing_on_the_whole_ring_after_a_first_pass_that_read_device_zero_mounts_on_the_first_pass_roots,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::an_abandoned_root_self_verified_only_on_the_rebuild_pass_read_back_as_zeros_by_the_floor_raise_keeps_its_units_isolated`）。
#[test]
fn an_abandoned_root_self_verified_only_on_the_rebuild_pass_read_back_as_zeros_by_the_floor_raise_keeps_its_units_isolated(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "an_abandoned_root_self_verified_only_on_the_rebuild_pass_read_back_as_zeros_by_the_floor_raise_keeps_its_units_isolated",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::an_abandoned_root_self_verified_only_on_the_rebuild_pass_read_back_as_zeros_by_the_floor_raise_keeps_its_units_isolated,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::an_abandoned_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling`）。
#[test]
fn an_abandoned_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "an_abandoned_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::an_abandoned_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::a_valid_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling`）。
#[test]
fn a_valid_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_valid_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::a_valid_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::unmounting_after_a_remount_with_an_abandoned_root_self_verified_only_on_the_rebuild_pass_keeps_its_units_isolated`）。
#[test]
fn unmounting_after_a_remount_with_an_abandoned_root_self_verified_only_on_the_rebuild_pass_keeps_its_units_isolated(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "unmounting_after_a_remount_with_an_abandoned_root_self_verified_only_on_the_rebuild_pass_keeps_its_units_isolated",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::unmounting_after_a_remount_with_an_abandoned_root_self_verified_only_on_the_rebuild_pass_keeps_its_units_isolated,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::an_abandoned_root_self_verified_only_on_the_rebuild_pass_holds_its_units_in_the_allocator_until_it_leaves_the_ring`）。
#[test]
fn an_abandoned_root_self_verified_only_on_the_rebuild_pass_holds_its_units_in_the_allocator_until_it_leaves_the_ring(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "an_abandoned_root_self_verified_only_on_the_rebuild_pass_holds_its_units_in_the_allocator_until_it_leaves_the_ring",
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::an_abandoned_root_self_verified_only_on_the_rebuild_pass_holds_its_units_in_the_allocator_until_it_leaves_the_ring,
    );
}

// ─── harness 档 `tests/reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root::eleven_crashes_before_the_row_root_then_a_mount_leave_every_unit_of_the_roots_outside_the_one_revolution_window_allocated`）。
#[test]
fn eleven_crashes_before_the_row_root_then_a_mount_leave_every_unit_of_the_roots_outside_the_one_revolution_window_allocated(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "eleven_crashes_before_the_row_root_then_a_mount_leave_every_unit_of_the_roots_outside_the_one_revolution_window_allocated",
        scenarios_of_reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root::eleven_crashes_before_the_row_root_then_a_mount_leave_every_unit_of_the_roots_outside_the_one_revolution_window_allocated,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root::one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring`）。
#[test]
fn one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring",
        scenarios_of_reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root::one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring,
    );
}

// ─── harness 档 `tests/release_checksum_quarantine.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/release_checksum_quarantine.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read`）。
#[test]
fn quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read",
        scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/release_checksum_quarantine.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_release_checksum_quarantine::unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not`）。
#[test]
fn unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not() {
    judge_the_images_a_harness_scenario_hands_out(
        "unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not",
        scenarios_of_release_checksum_quarantine::unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/release_checksum_quarantine.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_release_checksum_quarantine::quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry`）。
#[test]
fn quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry",
        scenarios_of_release_checksum_quarantine::quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry,
    );
}

// ─── harness 档 `tests/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::the_replayed_version_invariant_holds_over_the_untouched_record_without_its_root`）。
#[test]
fn the_replayed_version_invariant_holds_over_the_untouched_record_without_its_root() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_replayed_version_invariant_holds_over_the_untouched_record_without_its_root",
        scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::the_replayed_version_invariant_holds_over_the_untouched_record_without_its_root,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment`）。
#[test]
fn the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment",
        scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment`）。
#[test]
fn the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment",
        scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment,
    );
}

// ─── harness 档 `tests/reuse_after_raising_the_floor.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reuse_after_raising_the_floor::raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it`）。
#[test]
fn raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it",
        scenarios_of_reuse_after_raising_the_floor::raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reuse_after_raising_the_floor::the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker`）。
#[test]
fn the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker",
        scenarios_of_reuse_after_raising_the_floor::the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reuse_after_raising_the_floor::reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red`）。
#[test]
fn reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red",
        scenarios_of_reuse_after_raising_the_floor::reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reuse_after_raising_the_floor::raising_the_floor_in_the_make_filesystem_process_after_three_overwrites_reads_the_instance_table_through_the_root`）。
#[test]
fn raising_the_floor_in_the_make_filesystem_process_after_three_overwrites_reads_the_instance_table_through_the_root(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "raising_the_floor_in_the_make_filesystem_process_after_three_overwrites_reads_the_instance_table_through_the_root",
        scenarios_of_reuse_after_raising_the_floor::raising_the_floor_in_the_make_filesystem_process_after_three_overwrites_reads_the_instance_table_through_the_root,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reuse_after_raising_the_floor::four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units`）。
#[test]
fn four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units",
        scenarios_of_reuse_after_raising_the_floor::four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reuse_after_raising_the_floor::the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor`）。
#[test]
fn the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor",
        scenarios_of_reuse_after_raising_the_floor::the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor,
    );
}

// ─── harness 档 `tests/reused_allocation_record_overlap.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/reused_allocation_record_overlap.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_reused_allocation_record_overlap::reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds`）。
#[test]
fn reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds",
        scenarios_of_reused_allocation_record_overlap::reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds,
    );
}

// ─── harness 档 `tests/rollback_by_a_forward_publish.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_by_a_forward_publish::rolling_back_to_the_first_version_while_mounted_publishes_one_root_carrying_its_state_and_cold_start_reads_the_first_content`）。
#[test]
fn rolling_back_to_the_first_version_while_mounted_publishes_one_root_carrying_its_state_and_cold_start_reads_the_first_content(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "rolling_back_to_the_first_version_while_mounted_publishes_one_root_carrying_its_state_and_cold_start_reads_the_first_content",
        scenarios_of_rollback_by_a_forward_publish::rolling_back_to_the_first_version_while_mounted_publishes_one_root_carrying_its_state_and_cold_start_reads_the_first_content,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_by_a_forward_publish::after_rolling_back_to_the_first_version_the_next_overwrite_releases_the_resurrected_units_and_every_invariant_holds`）。
#[test]
fn after_rolling_back_to_the_first_version_the_next_overwrite_releases_the_resurrected_units_and_every_invariant_holds(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "after_rolling_back_to_the_first_version_the_next_overwrite_releases_the_resurrected_units_and_every_invariant_holds",
        scenarios_of_rollback_by_a_forward_publish::after_rolling_back_to_the_first_version_the_next_overwrite_releases_the_resurrected_units_and_every_invariant_holds,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_by_a_forward_publish::after_rolling_back_to_the_first_version_the_third_versions_root_is_still_a_candidate_and_rolling_back_to_it_reads_the_third_content`）。
#[test]
fn after_rolling_back_to_the_first_version_the_third_versions_root_is_still_a_candidate_and_rolling_back_to_it_reads_the_third_content(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "after_rolling_back_to_the_first_version_the_third_versions_root_is_still_a_candidate_and_rolling_back_to_it_reads_the_third_content",
        scenarios_of_rollback_by_a_forward_publish::after_rolling_back_to_the_first_version_the_third_versions_root_is_still_a_candidate_and_rolling_back_to_it_reads_the_third_content,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_by_a_forward_publish::rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write`）。
#[test]
fn rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write",
        scenarios_of_rollback_by_a_forward_publish::rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_by_a_forward_publish::rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write`）。
#[test]
fn rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write",
        scenarios_of_rollback_by_a_forward_publish::rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_refused_before_writing_is_resent_in_process_or_replayed_after_a_crash`）。
#[test]
fn the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_refused_before_writing_is_resent_in_process_or_replayed_after_a_crash(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_refused_before_writing_is_resent_in_process_or_replayed_after_a_crash",
        scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_refused_before_writing_is_resent_in_process_or_replayed_after_a_crash,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash`）。
#[test]
fn the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash",
        scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash`）。
#[test]
fn the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash",
        scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_by_a_forward_publish::writable_mount_on_the_newest_root_cannot_reclaim_what_its_own_chain_released_so_the_rehearsal_without_the_release_read_cannot_diverge`）。
#[test]
fn writable_mount_on_the_newest_root_cannot_reclaim_what_its_own_chain_released_so_the_rehearsal_without_the_release_read_cannot_diverge(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "writable_mount_on_the_newest_root_cannot_reclaim_what_its_own_chain_released_so_the_rehearsal_without_the_release_read_cannot_diverge",
        scenarios_of_rollback_by_a_forward_publish::writable_mount_on_the_newest_root_cannot_reclaim_what_its_own_chain_released_so_the_rehearsal_without_the_release_read_cannot_diverge,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_by_a_forward_publish::plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced`）。
#[test]
fn plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced",
        scenarios_of_rollback_by_a_forward_publish::plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_by_a_forward_publish::a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root`）。
#[test]
fn a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root",
        scenarios_of_rollback_by_a_forward_publish::a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_by_a_forward_publish::inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback`）。
#[test]
fn inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback",
        scenarios_of_rollback_by_a_forward_publish::inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback,
    );
}

// ─── harness 档 `tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term::a_rollback_appends_one_lineage_record_that_reads_back_field_by_field`）。
#[test]
fn a_rollback_appends_one_lineage_record_that_reads_back_field_by_field() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_rollback_appends_one_lineage_record_that_reads_back_field_by_field",
        scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term::a_rollback_appends_one_lineage_record_that_reads_back_field_by_field,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term::a_second_rollback_appends_to_the_same_open_lineage_container`）。
#[test]
fn a_second_rollback_appends_to_the_same_open_lineage_container() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_second_rollback_appends_to_the_same_open_lineage_container",
        scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term::a_second_rollback_appends_to_the_same_open_lineage_container,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term::a_rollback_records_the_inode_number_watermark_of_the_version_it_rolls_back_from_not_of_its_target`）。
#[test]
fn a_rollback_records_the_inode_number_watermark_of_the_version_it_rolls_back_from_not_of_its_target(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_rollback_records_the_inode_number_watermark_of_the_version_it_rolls_back_from_not_of_its_target",
        scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term::a_rollback_records_the_inode_number_watermark_of_the_version_it_rolls_back_from_not_of_its_target,
    );
}

// ─── harness 档 `tests/rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate::rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_reads_back_that_version`）。
#[test]
fn rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_reads_back_that_version()
{
    judge_the_images_a_harness_scenario_hands_out(
        "rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_reads_back_that_version",
        scenarios_of_rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate::rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_reads_back_that_version,
    );
}

// ─── harness 档 `tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance::of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does`）。
#[test]
fn of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does",
        scenarios_of_root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance::of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does,
    );
}

// ─── harness 档 `tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::a_valid_root_read_bad_once_on_the_reread_keeps_the_slot_pinned_at_the_oldest_txg_it_can_have_held`）。
#[test]
fn a_valid_root_read_bad_once_on_the_reread_keeps_the_slot_pinned_at_the_oldest_txg_it_can_have_held(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_valid_root_read_bad_once_on_the_reread_keeps_the_slot_pinned_at_the_oldest_txg_it_can_have_held",
        scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::a_valid_root_read_bad_once_on_the_reread_keeps_the_slot_pinned_at_the_oldest_txg_it_can_have_held,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::a_valid_root_read_on_the_reread_holds_the_reclaim_threshold_at_its_txg`）。
#[test]
fn a_valid_root_read_on_the_reread_holds_the_reclaim_threshold_at_its_txg() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_valid_root_read_on_the_reread_holds_the_reclaim_threshold_at_its_txg",
        scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::a_valid_root_read_on_the_reread_holds_the_reclaim_threshold_at_its_txg,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::a_front_hole_unreadable_at_mount_reread_as_all_zero_stays_pinned_until_the_process_overwrites_it`）。
#[test]
fn a_front_hole_unreadable_at_mount_reread_as_all_zero_stays_pinned_until_the_process_overwrites_it(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_front_hole_unreadable_at_mount_reread_as_all_zero_stays_pinned_until_the_process_overwrites_it",
        scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::a_front_hole_unreadable_at_mount_reread_as_all_zero_stays_pinned_until_the_process_overwrites_it,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::an_abandoned_root_read_on_the_reread_is_taken_as_no_root`）。
#[test]
fn an_abandoned_root_read_on_the_reread_is_taken_as_no_root() {
    judge_the_images_a_harness_scenario_hands_out(
        "an_abandoned_root_read_on_the_reread_is_taken_as_no_root",
        scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::an_abandoned_root_read_on_the_reread_is_taken_as_no_root,
    );
}

// ─── harness 档 `tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window::a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants`）。
#[test]
fn a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants",
        scenarios_of_root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window::a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window::a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it`）。
#[test]
fn a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it",
        scenarios_of_root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window::a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it,
    );
}

// ─── harness 档 `tests/root_ring_turn_in_one_mount.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/root_ring_turn_in_one_mount.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_root_ring_turn_in_one_mount::turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it`）。
#[test]
fn turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it",
        scenarios_of_root_ring_turn_in_one_mount::turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it,
    );
}

// ─── harness 档 `tests/root_ring_turn_in_the_make_filesystem_session.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/root_ring_turn_in_the_make_filesystem_session.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_root_ring_turn_in_the_make_filesystem_session::overwrites_in_the_make_filesystem_session_past_one_turn_of_the_root_ring_reclaim_and_keep_every_invariant`）。
#[test]
fn overwrites_in_the_make_filesystem_session_past_one_turn_of_the_root_ring_reclaim_and_keep_every_invariant(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "overwrites_in_the_make_filesystem_session_past_one_turn_of_the_root_ring_reclaim_and_keep_every_invariant",
        scenarios_of_root_ring_turn_in_the_make_filesystem_session::overwrites_in_the_make_filesystem_session_past_one_turn_of_the_root_ring_reclaim_and_keep_every_invariant,
    );
}

// ─── harness 档 `tests/row_publish_admission.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/row_publish_admission.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_row_publish_admission::writable_mount_on_the_pool_whose_row_publish_used_to_be_refused_before_acquisition_succeeds`）。
#[test]
fn writable_mount_on_the_pool_whose_row_publish_used_to_be_refused_before_acquisition_succeeds() {
    judge_the_images_a_harness_scenario_hands_out(
        "writable_mount_on_the_pool_whose_row_publish_used_to_be_refused_before_acquisition_succeeds",
        scenarios_of_row_publish_admission::writable_mount_on_the_pool_whose_row_publish_used_to_be_refused_before_acquisition_succeeds,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/row_publish_admission.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_row_publish_admission::writable_mount_on_the_pool_whose_second_warm_up_used_to_be_refused_before_acquisition_succeeds`）。
#[test]
fn writable_mount_on_the_pool_whose_second_warm_up_used_to_be_refused_before_acquisition_succeeds()
{
    judge_the_images_a_harness_scenario_hands_out(
        "writable_mount_on_the_pool_whose_second_warm_up_used_to_be_refused_before_acquisition_succeeds",
        scenarios_of_row_publish_admission::writable_mount_on_the_pool_whose_second_warm_up_used_to_be_refused_before_acquisition_succeeds,
    );
}

// ─── harness 档 `tests/row_publish_checks_before_acquisition.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/row_publish_checks_before_acquisition.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_row_publish_checks_before_acquisition::row_publishes_on_a_version_without_file_with_a_sixty_six_page_instance_table_keep_mounting_writable_past_812_allocation_records`）。
#[test]
fn row_publishes_on_a_version_without_file_with_a_sixty_six_page_instance_table_keep_mounting_writable_past_812_allocation_records(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "row_publishes_on_a_version_without_file_with_a_sixty_six_page_instance_table_keep_mounting_writable_past_812_allocation_records",
        scenarios_of_row_publish_checks_before_acquisition::row_publishes_on_a_version_without_file_with_a_sixty_six_page_instance_table_keep_mounting_writable_past_812_allocation_records,
    );
}

// ─── harness 档 `tests/row_publish_record_without_its_root.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/row_publish_record_without_its_root.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_row_publish_record_without_its_root::c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_stay_isolated_from_the_next_mount`）。
#[test]
fn c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_stay_isolated_from_the_next_mount(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_stay_isolated_from_the_next_mount",
        scenarios_of_row_publish_record_without_its_root::c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_stay_isolated_from_the_next_mount,
    );
}

// ─── harness 档 `tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring::raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring`）。
#[test]
fn raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring",
        scenarios_of_shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring::raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring,
    );
}

// ─── harness 档 `tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_unit_area_start_one_segment_past_the_ring_is_refused_by_core_and_reddens_the_checker_table`）。
#[test]
fn a_unit_area_start_one_segment_past_the_ring_is_refused_by_core_and_reddens_the_checker_table() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_unit_area_start_one_segment_past_the_ring_is_refused_by_core_and_reddens_the_checker_table",
        scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_unit_area_start_one_segment_past_the_ring_is_refused_by_core_and_reddens_the_checker_table,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_journal_ring_start_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table`）。
#[test]
fn a_journal_ring_start_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_journal_ring_start_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table",
        scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_journal_ring_start_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_root_ring_base_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table`）。
#[test]
fn a_root_ring_base_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_root_ring_base_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table",
        scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_root_ring_base_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_unit_area_start_off_the_cluster_segment_boundary_is_refused_by_core_and_reddens_the_checker_table`）。
#[test]
fn a_unit_area_start_off_the_cluster_segment_boundary_is_refused_by_core_and_reddens_the_checker_table(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_unit_area_start_off_the_cluster_segment_boundary_is_refused_by_core_and_reddens_the_checker_table",
        scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_unit_area_start_off_the_cluster_segment_boundary_is_refused_by_core_and_reddens_the_checker_table,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_root_ring_base_of_zero_is_refused_by_core_for_the_slot_spacing_and_reddens_the_checker_table_for_the_base`）。
#[test]
fn a_root_ring_base_of_zero_is_refused_by_core_for_the_slot_spacing_and_reddens_the_checker_table_for_the_base(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_root_ring_base_of_zero_is_refused_by_core_for_the_slot_spacing_and_reddens_the_checker_table_for_the_base",
        scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table::a_root_ring_base_of_zero_is_refused_by_core_for_the_slot_spacing_and_reddens_the_checker_table_for_the_base,
    );
}

// ─── harness 档 `tests/system_configuration_per_device_redundancy.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/system_configuration_per_device_redundancy.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_system_configuration_per_device_redundancy::the_pool_checker_still_judges_every_invariant_when_one_device_has_no_valid_system_configuration`）。
#[test]
fn the_pool_checker_still_judges_every_invariant_when_one_device_has_no_valid_system_configuration()
{
    judge_the_images_a_harness_scenario_hands_out(
        "the_pool_checker_still_judges_every_invariant_when_one_device_has_no_valid_system_configuration",
        scenarios_of_system_configuration_per_device_redundancy::the_pool_checker_still_judges_every_invariant_when_one_device_has_no_valid_system_configuration,
    );
}

// ─── harness 档 `tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::an_io_min_8192_pool_whose_slot_zeros_are_wiped_has_the_checker_and_recovery_both_choose_slot_one_at_8192`）。
#[test]
fn an_io_min_8192_pool_whose_slot_zeros_are_wiped_has_the_checker_and_recovery_both_choose_slot_one_at_8192(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "an_io_min_8192_pool_whose_slot_zeros_are_wiped_has_the_checker_and_recovery_both_choose_slot_one_at_8192",
        scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::an_io_min_8192_pool_whose_slot_zeros_are_wiped_has_the_checker_and_recovery_both_choose_slot_one_at_8192,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::a_slot_carrying_a_value_the_reader_refuses_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery`）。
#[test]
fn a_slot_carrying_a_value_the_reader_refuses_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_slot_carrying_a_value_the_reader_refuses_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery",
        scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::a_slot_carrying_a_value_the_reader_refuses_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::a_slot_whose_in_flight_limit_is_off_by_one_and_whose_spacing_is_not_its_offset_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery`）。
#[test]
fn a_slot_whose_in_flight_limit_is_off_by_one_and_whose_spacing_is_not_its_offset_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_slot_whose_in_flight_limit_is_off_by_one_and_whose_spacing_is_not_its_offset_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery",
        scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::a_slot_whose_in_flight_limit_is_off_by_one_and_whose_spacing_is_not_its_offset_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::a_slot_whose_in_flight_limit_is_off_by_one_and_whose_generation_is_lower_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery`）。
#[test]
fn a_slot_whose_in_flight_limit_is_off_by_one_and_whose_generation_is_lower_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_slot_whose_in_flight_limit_is_off_by_one_and_whose_generation_is_lower_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery",
        scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::a_slot_whose_in_flight_limit_is_off_by_one_and_whose_generation_is_lower_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery`）。
#[test]
fn only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery",
        scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery::only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery,
    );
}

// ─── harness 档 `tests/test_only_switches_force_every_branch.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/test_only_switches_force_every_branch.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_test_only_switches_force_every_branch::reuse_window_forced_to_zero_lets_a_publish_overwrite_a_unit_a_candidate_root_still_references_and_the_checker_goes_red`）。
#[test]
fn reuse_window_forced_to_zero_lets_a_publish_overwrite_a_unit_a_candidate_root_still_references_and_the_checker_goes_red(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "reuse_window_forced_to_zero_lets_a_publish_overwrite_a_unit_a_candidate_root_still_references_and_the_checker_goes_red",
        scenarios_of_test_only_switches_force_every_branch::reuse_window_forced_to_zero_lets_a_publish_overwrite_a_unit_a_candidate_root_still_references_and_the_checker_goes_red,
    );
}

// ─── harness 档 `tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker`）。
#[test]
fn the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker",
        scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds`）。
#[test]
fn the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds",
        scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places`）。
#[test]
fn the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places",
        scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places,
    );
}

// ─── harness 档 `tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over::a_one_gibibyte_journal_ring_puts_the_unit_area_right_after_the_ring_through_mount_and_checker`）。
#[test]
fn a_one_gibibyte_journal_ring_puts_the_unit_area_right_after_the_ring_through_mount_and_checker() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_one_gibibyte_journal_ring_puts_the_unit_area_right_after_the_ring_through_mount_and_checker",
        scenarios_of_unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over::a_one_gibibyte_journal_ring_puts_the_unit_area_right_after_the_ring_through_mount_and_checker,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over::a_128_mebibyte_journal_ring_puts_the_unit_area_right_after_the_ring_through_mount_and_checker`）。
#[test]
fn a_128_mebibyte_journal_ring_puts_the_unit_area_right_after_the_ring_through_mount_and_checker() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_128_mebibyte_journal_ring_puts_the_unit_area_right_after_the_ring_through_mount_and_checker",
        scenarios_of_unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over::a_128_mebibyte_journal_ring_puts_the_unit_area_right_after_the_ring_through_mount_and_checker,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over::the_rollback_takes_the_tree_identifier_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_every_root_carrying_it`）。
#[test]
fn the_rollback_takes_the_tree_identifier_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_every_root_carrying_it(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_rollback_takes_the_tree_identifier_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_every_root_carrying_it",
        scenarios_of_unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over::the_rollback_takes_the_tree_identifier_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_every_root_carrying_it,
    );
}

// ─── harness 档 `tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version::a_crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to_the_uninterrupted_end_state`）。
#[test]
fn a_crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to_the_uninterrupted_end_state(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to_the_uninterrupted_end_state",
        scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version::a_crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to_the_uninterrupted_end_state,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version::rolling_back_to_the_emptied_version_is_accepted_and_publishes_a_version_with_trees_and_no_file`）。
#[test]
fn rolling_back_to_the_emptied_version_is_accepted_and_publishes_a_version_with_trees_and_no_file()
{
    judge_the_images_a_harness_scenario_hands_out(
        "rolling_back_to_the_emptied_version_is_accepted_and_publishes_a_version_with_trees_and_no_file",
        scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version::rolling_back_to_the_emptied_version_is_accepted_and_publishes_a_version_with_trees_and_no_file,
    );
}

// ─── harness 档 `tests/unlink_mount_continuation_and_the_emptied_version.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unlink_mount_continuation_and_the_emptied_version::a_writable_mount_after_the_intent_continues_the_queue_to_empty_after_its_warm_up`）。
#[test]
fn a_writable_mount_after_the_intent_continues_the_queue_to_empty_after_its_warm_up() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_writable_mount_after_the_intent_continues_the_queue_to_empty_after_its_warm_up",
        scenarios_of_unlink_mount_continuation_and_the_emptied_version::a_writable_mount_after_the_intent_continues_the_queue_to_empty_after_its_warm_up,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unlink_mount_continuation_and_the_emptied_version::the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish`）。
#[test]
fn the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish",
        scenarios_of_unlink_mount_continuation_and_the_emptied_version::the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unlink_mount_continuation_and_the_emptied_version::rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_continues_it_to_empty`）。
#[test]
fn rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_continues_it_to_empty(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_continues_it_to_empty",
        scenarios_of_unlink_mount_continuation_and_the_emptied_version::rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_continues_it_to_empty,
    );
}

// ─── harness 档 `tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches::a_file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut`）。
#[test]
fn a_file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut",
        scenarios_of_unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches::a_file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut,
    );
}

// ─── harness 档 `tests/unlink_single_file_publishes.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/unlink_single_file_publishes.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_unlink_single_file_publishes::the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch`）。
#[test]
fn the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch",
        scenarios_of_unlink_single_file_publishes::the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch,
    );
}

// ─── harness 档 `tests/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero::a_publish_whose_root_slot_and_last_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount`）。
#[test]
fn a_publish_whose_root_slot_and_last_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_publish_whose_root_slot_and_last_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount",
        scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero::a_publish_whose_root_slot_and_last_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero::a_publish_whose_root_slot_and_middle_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount`）。
#[test]
fn a_publish_whose_root_slot_and_middle_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_publish_whose_root_slot_and_middle_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount",
        scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero::a_publish_whose_root_slot_and_middle_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero::a_publish_whose_root_slot_and_first_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount`）。
#[test]
fn a_publish_whose_root_slot_and_first_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_publish_whose_root_slot_and_first_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount",
        scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero::a_publish_whose_root_slot_and_first_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount,
    );
}

// ─── harness 档 `tests/writable_mount_of_a_formatted_pool.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_writable_mount_of_a_formatted_pool::the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool`）。
#[test]
fn the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool",
        scenarios_of_writable_mount_of_a_formatted_pool::the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_writable_mount_of_a_formatted_pool::writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold`）。
#[test]
fn writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold",
        scenarios_of_writable_mount_of_a_formatted_pool::writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_writable_mount_of_a_formatted_pool::a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold`）。
#[test]
fn a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold() {
    judge_the_images_a_harness_scenario_hands_out(
        "a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold",
        scenarios_of_writable_mount_of_a_formatted_pool::a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_writable_mount_of_a_formatted_pool::crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file_chooses_it_when_the_read_fault_is_lifted_before_the_one_reread`）。
#[test]
fn crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file_chooses_it_when_the_read_fault_is_lifted_before_the_one_reread(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file_chooses_it_when_the_read_fault_is_lifted_before_the_one_reread",
        scenarios_of_writable_mount_of_a_formatted_pool::crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file_chooses_it_when_the_read_fault_is_lifted_before_the_one_reread,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_writable_mount_of_a_formatted_pool::crash_recovery_abandoning_the_unwitnessed_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them`）。
#[test]
fn crash_recovery_abandoning_the_unwitnessed_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "crash_recovery_abandoning_the_unwitnessed_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them",
        scenarios_of_writable_mount_of_a_formatted_pool::crash_recovery_abandoning_the_unwitnessed_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them,
    );
}

// ─── harness 档 `tests/writable_mount_takes_a_second_instance.rs` ───

/// 与 harness 档 `crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_writable_mount_takes_a_second_instance::remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version`）。
#[test]
fn remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version()
{
    judge_the_images_a_harness_scenario_hands_out(
        "remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version",
        scenarios_of_writable_mount_takes_a_second_instance::remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_writable_mount_takes_a_second_instance::checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root`）。
#[test]
fn checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root() {
    judge_the_images_a_harness_scenario_hands_out(
        "checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root",
        scenarios_of_writable_mount_takes_a_second_instance::checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root,
    );
}

// ─── harness 档 `tests/fault_injection_fast_tier.rs` 写死的那两条注入点：池级 checker 的结论交 harness 归类 ───

/// mkfs 在写到系统配置之前就报错时重开的那张盘：盘上没有一份自证过的系统配置，GPU 判器建不出判表，引擎照 H6 记一条「GPU 判不了」、
/// 交 CPU 判器的结论（`crates/singlefs-checker-tier/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs`
/// 那几张 mkfs 没做完的表同一种）。
const MAKE_FILESYSTEM_UNFINISHED: &str =
    "mkfs 在写到系统配置之前就报错：盘上没有一份自证过的系统配置，GPU 判器建不出判表";

/// 一个注入点交出去的盘（`fault_injection::ImagesAfterAFault`）重建成镜像（每步镜像、重开的那张、被吞的写插回去那张各一张表），
/// 经给定状态引擎判池级 checker，结论交 harness 的归类（`fault_injection::classify_the_pool_checker_verdicts_of_the_images_after_a_fault`）。
/// 交回归好类的失败；这一趟不作数交 `None`。`the_gpu_may_refuse_the_reopened_image_because` 给了就是许 GPU 建不出重开那一张的判表。
fn failures_after_a_fault_judged_through_the_library(
    scenario_name: &str,
    handed_out: &ImagesAfterAFault,
    the_gpu_may_refuse_the_reopened_image_because: Option<&'static str>,
) -> Option<Vec<FailureAfterAFault>> {
    let pool_images = scenarios_of_fault_injection_fast_tier::pool_images_after_a_fault(handed_out);
    let mut collected = PoolCheckerImages::collected_for_the_checker_tier();
    for (index, step_image) in pool_images.step_images.iter().enumerate() {
        collected.hand_out(
            &format!("带着注入跑的那一遍的每步镜像（下标 {index}）"),
            || step_image.clone(),
            PoolCheckerExpectation::only_the_named_invariants(),
        );
    }
    collected.hand_out_allowing_the_gpu_to_refuse_its_tables(
        "注入之后重开的那张盘",
        || pool_images.after_the_fault.clone(),
        PoolCheckerExpectation::only_the_named_invariants(),
        the_gpu_may_refuse_the_reopened_image_because,
    );
    if let Some(landed) = &pool_images.with_the_swallowed_writes_landed {
        collected.hand_out(
            "被吞的写插回去之后的那张盘",
            || landed.clone(),
            PoolCheckerExpectation::only_the_named_invariants(),
        );
    }
    let images = collected.into_collected();
    let (conclusions, _run, _judged_afresh) = judge_the_collected_images(
        scenario_name,
        &images,
        ContainerCapacityKnobs::PRODUCT,
        JudgedAfreshRegardlessOfTheExpectations::OnlyIfAnImageExpectsIt,
    )?;
    let violations_of = |judged: &ConclusionsOfAnImage| -> Vec<(&'static str, String)> {
        let [state] = judged.states.as_slice() else {
            panic!("一张镜像一个状态，交回 {} 个", judged.states.len());
        };
        state.pool_checker_violations()
    };
    let step_images = pool_images.step_images.len();
    let violations_of_the_step_images: Vec<Vec<(&'static str, String)>> = conclusions
        [..step_images]
        .iter()
        .map(violations_of)
        .collect();
    let violations_after_the_fault = violations_of(&conclusions[step_images]);
    let invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet<&'static str> =
        conclusions
            .get(step_images + 1)
            .map_or_else(BTreeSet::new, |landed| {
                violations_of(landed)
                    .into_iter()
                    .map(|(invariant, _)| invariant)
                    .collect()
            });
    Some(
        classify_the_pool_checker_verdicts_of_the_images_after_a_fault(
            handed_out,
            violations_of_the_step_images,
            violations_after_the_fault,
            invariants_still_violated_if_the_swallowed_writes_had_landed,
        ),
    )
}

/// 与 harness 档 `tests/fault_injection_fast_tier.rs` 里同名的用例同一个注入点（第一个文件之后覆盖写五次，txg 7 那次的根槽 FUA 写被吞；
/// 场景见 `scenarios_of_fault_injection_fast_tier::the_fixed_history_with_its_root_slot_write_swallowed`）：交出去的盘经引擎判池级 checker、
/// 交归类——txg 8 那次覆盖写之后 checker 只判红 I-3.1（被吞的根槽留下的，被吞的写插回去就消失），按注入点认成说谎的设备留下的，
/// 不算新发现：带着注入跑的那一遍里第一张判红的每步镜像、重开的那张盘，各一条。
#[test]
fn swallowed_write_after_which_the_checker_flags_only_the_allocated_statistic_invariant_is_excused_as_what_the_lying_device_may_leave(
) {
    const TEST_NAME: &str = "swallowed_write_after_which_the_checker_flags_only_the_allocated_statistic_invariant_is_excused_as_what_the_lying_device_may_leave";
    let (_injection, handed_out) =
        scenarios_of_fault_injection_fast_tier::the_fixed_history_with_its_root_slot_write_swallowed();
    let [images] = handed_out.as_slice() else {
        panic!("一个注入点交出去一份盘：交了 {} 份", handed_out.len());
    };
    let Some(failures) = failures_after_a_fault_judged_through_the_library(TEST_NAME, images, None)
    else {
        return;
    };
    let expected_signature = "CheckerViolations { invariants: [\"I-3.1\"] }";
    assert_eq!(
        scenarios_of_fault_injection_fast_tier::rendered_failures_after_a_fault(&failures),
        vec![format!("说谎 {expected_signature}"), format!("说谎 {expected_signature}")],
        "被吞掉的那一次写之后只判红 I-3.1、被吞的写插回去就消失，按注入点认成说谎的设备留下的——带着注入跑的那一遍里第一张判红的每步镜像、\
         重开的那张盘，各一条：{failures:?}"
    );
}

/// 与 harness 档 `tests/fault_injection_fast_tier.rs` 里同名的用例同一个注入点（快档种子基 + 7 那一段在整池第 1 次 barrier 上报错，
/// 落在 mkfs 第一道屏障上；场景见 `scenarios_of_fault_injection_fast_tier::the_history_whose_make_filesystem_fails_at_its_first_barrier`）：
/// 重开的盘上只红 I-8.5，归类记成故障注入的「已知红」那一条（mkfs 在系统配置槽落盘之前报错），不记新发现。
/// 判别力自证：判法改回不认这一类（`known_red_form_only_after_a_fault` 恒交回 None，`crates/mutations.tsv` 同名那一条），
/// 这一次落成新发现 `CheckerViolations { invariants: ["I-8.5"] }`，这里红。
#[test]
fn make_filesystem_failing_at_its_first_barrier_before_any_system_configuration_slot_lands_ends_known_red_not_a_new_finding(
) {
    const TEST_NAME: &str = "make_filesystem_failing_at_its_first_barrier_before_any_system_configuration_slot_lands_ends_known_red_not_a_new_finding";
    let (_injection, handed_out) =
        scenarios_of_fault_injection_fast_tier::the_history_whose_make_filesystem_fails_at_its_first_barrier();
    let [images] = handed_out.as_slice() else {
        panic!("一个注入点交出去一份盘：交了 {} 份", handed_out.len());
    };
    assert!(
        images.step_images_of_the_run_with_the_fault.is_empty(),
        "mkfs 自己报错，一张每步镜像都不交"
    );
    let Some(failures) = failures_after_a_fault_judged_through_the_library(
        TEST_NAME,
        images,
        Some(MAKE_FILESYSTEM_UNFINISHED),
    ) else {
        return;
    };
    assert_eq!(
        failures,
        vec![FailureAfterAFault::KnownRed(KnownRedFormAtAFault::OnlyAfterAFault(
            KnownRedFormOnlyAfterAFault::MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded,
        ))],
        "mkfs 在系统配置槽落盘之前报错、重开的盘上只红 I-8.5 是认下的「已知红」那一条，不是新发现"
    );
}

// ─── GPU 判器与 CPU 判器在这几类镜像上对拍（编进 `gpu` 特性才有） ───

/// 一个交镜像的 harness 场景函数。
#[cfg(feature = "gpu")]
type ScenarioHandingOutImages = fn(&mut PoolCheckerImages);

/// 对拍取的场景：干净的镜像、改坏记录 / 根槽 / 链指针 / 树根的镜像、隔离豁免、不适用、产品旋钮下判红、一批单扣崩溃态，各取一两个；
/// 再加 unlink 路径上 inode 叶借、借完再吸收那两个（七百来个 inode：GPU 走读内核 inode → 对象出生代表那张表要装得下，
/// 这两个场景在对拍里由 GPU 判，那张表收窄时变异副本里也抓得到）。
#[cfg(feature = "gpu")]
const SCENARIOS_PAIRED_ON_THE_GPU_AND_THE_CPU: [(&str, ScenarioHandingOutImages); 13] = [
    (
        "writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page",
        scenarios_of_instance_table_page_full::writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page,
    ),
    (
        "the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment",
        scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment,
    ),
    (
        "chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant",
        scenarios_of_instance_table_chain_read::chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant,
    ),
    (
        "row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it",
        scenarios_of_multi_record_publishes_of_transaction_zero::row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it,
    ),
    (
        "commit_marker_other_than_zero_or_one_counts_as_torn_and_breaks_the_chain_and_the_checker_reddens",
        scenarios_of_multi_record_publishes_of_transaction_zero::commit_marker_other_than_zero_or_one_counts_as_torn_and_breaks_the_chain_and_the_checker_reddens,
    ),
    (
        "with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens",
        scenarios_of_journal_last_record_flag_of_a_multi_record_publish::with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens,
    ),
    (
        "the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root",
        scenarios_of_accounting_and_mapping_tree_split::the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root,
    ),
    (
        "with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations",
        scenarios_of_named_unit_verification_stands_in_for_the_barrier_before_the_record::with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations,
    ),
    (
        "quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read",
        scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read,
    ),
    (
        "the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries",
        scenarios_of_an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor::the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries,
    ),
    (
        "a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry",
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry,
    ),
    (
        "a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing",
        scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing,
    ),
    (
        "the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it",
        scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it,
    ),
];

/// 对拍那几个场景交出来的镜像，名字前面接上场景名（几个场景合进一趟，名字不撞）。
#[cfg(feature = "gpu")]
fn images_of_the_paired_scenarios() -> Vec<PoolCheckerImage> {
    let mut images = Vec::new();
    for (scenario_name, scenario) in SCENARIOS_PAIRED_ON_THE_GPU_AND_THE_CPU {
        let mut handed_out = PoolCheckerImages::collected_for_the_checker_tier();
        scenario(&mut handed_out);
        for mut image in handed_out.into_collected() {
            image.label = format!("{scenario_name}/{}", image.label);
            images.push(image);
        }
    }
    images
}

/// 一趟判下来的逐张镜像逐状态的对拍字，与引擎那一趟。
#[cfg(feature = "gpu")]
struct WordsOfARun {
    words: Vec<Vec<singlefs_checker_tier::crash_judge_conclusions::CrossCheckWords>>,
    run: GivenStatesRun,
}

/// 同一批镜像的两趟：配了 GPU 的那一趟与没开 GPU 的那一趟。
#[cfg(feature = "gpu")]
struct PairedRuns {
    images: Vec<PoolCheckerImage>,
    on_the_gpu: WordsOfARun,
    on_the_cpu: WordsOfARun,
}

/// 同一批镜像在环境守卫里判一趟：多机配置指到 `configuration`、库在 `library`，开 GPU 时这台机器的角色与清场之后的显卡先后照调用方给这个进程的那一份
/// （显卡先后经环境变量 `GPU_CARDS_BY_PRIORITY` 交，`common_gpu_cards_of_the_caller` 文件头写了为什么不抄进配置；守卫把一趟读的变量连同机器角色
/// 一起清掉；写法同 `crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs`）。
/// 交回逐张镜像逐状态的对拍字与引擎那一趟。
#[cfg(feature = "gpu")]
fn words_of_the_paired_images_judged_under(
    images: &[PoolCheckerImage],
    configuration: &std::path::Path,
    library: &std::path::Path,
    gpu_enabled: bool,
    fault: singlefs_checker_tier::crash_amplification::FaultPutOnTheGpuSide,
) -> WordsOfARun {
    use singlefs_checker_tier::crash_amplification::CrashAmplificationEnvironment;
    use singlefs_checker_tier::gpu_card_configuration::{
        GpuConfigurationOfTheCaller, CARDS_BY_PRIORITY_KEY, HOST_ROLE_ENVIRONMENT_VARIABLE,
        MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
    };
    let host = GpuConfigurationOfTheCaller::of_this_process()
        .host()
        .expect("调用方给的机器角色是 local 或 peer");
    let cards = if gpu_enabled {
        common_gpu_cards_of_the_caller::card_priority_value_of_the_caller(
            &repository_root_of_this_crate(),
        )
    } else {
        String::new()
    };
    let mut settings: Vec<(&str, &std::ffi::OsStr)> = vec![
        (
            MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            configuration.as_os_str(),
        ),
        (
            CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
            library.as_os_str(),
        ),
    ];
    if gpu_enabled {
        settings.push((
            HOST_ROLE_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new(host.name()),
        ));
        settings.push((CARDS_BY_PRIORITY_KEY, std::ffi::OsStr::new(&cards)));
    }
    let environment = CrashAmplificationEnvironment::cleared_then_set(&settings);
    let engine = GivenStatesEngine::start_from_environment_putting_a_fault_on_the_gpu_side(
        GivenStatesItem {
            item_name: ITEM_NAME.to_string(),
            setting_file: THIS_FILE.to_string(),
            seeds_and_scale: "gpu-and-cpu-pairing".to_string(),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        &repository_root_of_this_crate(),
        fault,
    )
    .expect("引擎照这一趟的配置起得来");
    let words = images
        .iter()
        .map(|image| {
            judge_one_image(&engine, image, ContainerCapacityKnobs::PRODUCT)
                .states
                .iter()
                .map(|state| state.words)
                .collect()
        })
        .collect();
    let run = engine.finish();
    drop(environment);
    WordsOfARun { words, run }
}

/// 同一批镜像一趟配了 GPU、一趟没开 GPU，各写一份自己的库（家目录下流水线单元测试那一类里以用例标签命名、起跑先清空、跑完留着）。
#[cfg(feature = "gpu")]
fn paired_images_judged_on_the_gpu_and_on_the_cpu(
    tag: &str,
    fault: singlefs_checker_tier::crash_amplification::FaultPutOnTheGpuSide,
) -> PairedRuns {
    let directory_of = |run: &str| {
        singlefs_checker_tier::crash_library::amplification_unit_tests_library_directory_emptied_at_the_start(
            &format!("harness-scenario-images-gpu-pairing-{tag}-{run}"),
        )
        .expect("流水线单元测试那一类里这条用例的目录")
    };
    let images = images_of_the_paired_scenarios();
    let gpu_directory = directory_of("on-the-gpu");
    let gpu_configuration = gpu_directory.join("multi-host-with-gpu.env");
    // 配置只写开关：显卡先后照调用方那一份经环境变量交（`words_of_the_paired_images_judged_under`）
    std::fs::write(&gpu_configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
    let on_the_gpu = words_of_the_paired_images_judged_under(
        &images,
        &gpu_configuration,
        &gpu_directory.join("library"),
        true,
        fault,
    );
    let cpu_directory = directory_of("on-the-cpu");
    let cpu_configuration = cpu_directory.join("multi-host-on-the-cpu.env");
    std::fs::write(&cpu_configuration, "ENABLE_GPU=0\n").expect("写得了配置");
    let on_the_cpu = words_of_the_paired_images_judged_under(
        &images,
        &cpu_configuration,
        &cpu_directory.join("library"),
        false,
        singlefs_checker_tier::crash_amplification::FaultPutOnTheGpuSide::NoFault,
    );
    PairedRuns {
        images,
        on_the_gpu,
        on_the_cpu,
    }
}

/// GPU 与 CPU 对拍（两趟独立、都从库里读）：对拍那几个场景交出来的同一批镜像一趟由 GPU 判器判、一趟由 CPU 判器判，逐张逐状态的对拍字相同；
/// GPU 那一趟判不了的只许是场景许它建不出判表的那几张（这一批里没有）。要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
#[cfg(feature = "gpu")]
#[test]
fn the_images_of_the_paired_harness_scenarios_judged_on_the_gpu_and_on_the_cpu_give_the_same_words()
{
    use singlefs_checker_tier::crash_amplification::{FaultPutOnTheGpuSide, JudgedBy};
    use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    const TEST_NAME: &str =
        "the_images_of_the_paired_harness_scenarios_judged_on_the_gpu_and_on_the_cpu_give_the_same_words";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let PairedRuns {
        images,
        on_the_gpu: WordsOfARun {
            words: gpu_words,
            run: gpu_run,
        },
        on_the_cpu: WordsOfARun {
            words: cpu_words,
            run: cpu_run,
        },
    } = paired_images_judged_on_the_gpu_and_on_the_cpu("agrees", FaultPutOnTheGpuSide::NoFault);
    assert_eq!(
        gpu_run.judged_by,
        JudgedBy::GpuJudge,
        "配了 GPU 的那一趟：GPU 判器判"
    );
    assert_eq!(
        cpu_run.judged_by,
        JudgedBy::CpuJudge,
        "没开 GPU 的那一趟：CPU 判器判"
    );
    assert!(
        gpu_run.tally.states_judged_by_the_gpu_here > 0 && gpu_run.tally.states_reused == 0,
        "GPU 那一趟从空库起、状态都是这一趟在卡上判的：{:?}",
        gpu_run.tally
    );
    assert_eq!(
        gpu_run.tally.states_the_gpu_could_not_judge,
        Vec::<String>::new(),
        "GPU 判器判得了这一批每一张（写表为空的表、改坏的镜像都在里面）"
    );
    let mismatches: Vec<String> = images
        .iter()
        .zip(gpu_words.iter().zip(&cpu_words))
        .filter(|(_, (on_the_gpu, on_the_cpu))| on_the_gpu != on_the_cpu)
        .map(|(image, (on_the_gpu, on_the_cpu))| {
            format!("{}：GPU {on_the_gpu:?}；CPU {on_the_cpu:?}", image.label)
        })
        .collect();
    println!(
        "HARNESS_SCENARIO_IMAGES_GPU_PAIRING images={} states={} mismatches={}",
        images.len(),
        gpu_words.iter().map(Vec::len).sum::<usize>(),
        mismatches.len()
    );
    assert!(
        mismatches.is_empty(),
        "{} 张镜像上 GPU 一趟与 CPU 一趟的对拍字不同：\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

/// 上一条的判别力：往 GPU 那一侧塞一个已知故障（每张表第一个状态的第一条不变量违例位翻过来），GPU 那一趟的对拍字与 CPU 那一趟不同。
#[cfg(feature = "gpu")]
#[test]
fn a_fault_put_on_the_gpu_side_makes_the_gpu_run_on_the_paired_harness_scenario_images_differ_from_the_cpu_run(
) {
    use singlefs_checker_tier::crash_amplification::FaultPutOnTheGpuSide;
    use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    const TEST_NAME: &str =
        "a_fault_put_on_the_gpu_side_makes_the_gpu_run_on_the_paired_harness_scenario_images_differ_from_the_cpu_run";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let paired = paired_images_judged_on_the_gpu_and_on_the_cpu(
        "fault",
        FaultPutOnTheGpuSide::FirstInvariantViolatedBitFlippedOnTheFirstStateOfEachTable,
    );
    assert_ne!(
        paired.on_the_gpu.words, paired.on_the_cpu.words,
        "GPU 那一侧塞了故障，两趟的对拍字却相同"
    );
}

// ============================== I-7.4 点名的被抛弃根（用户 2026-10-14 弹窗定「GPU 判器加细节输出」）与旋钮栏进块键 ==============================

/// 被抛弃根那一半判 I-7.4 红的一张镜像与它点名的那一条根：建池（第一个文件 A）→ 覆盖写 B、C → 崩溃恢复抛弃 C（实例 1、txg 5；
/// `scenarios_of_random_histories::pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version`，挂载时影子账隔离 C 的单元，这张镜像池级 checker 一条不红），
/// 再把 C 写的第一个数据单元的第一份里头之后的一段字节改掉：现行时间线（实例 2）上的根都不引用它，只有被抛弃根那一半走 C 时读到它校验和对不上，
/// I-7.4 的第一处违例就是这一条，违例细节点名 C。
fn image_whose_abandoned_root_walks_into_an_overwritten_unit(
    tag: &str,
) -> (
    singlefs_harness::memory_pool::MemoryPool,
    singlefs_checker_tier::crash_given_states::AbandonedRootNamedByThePoolChecker,
) {
    use singlefs_checker_tier::crash_given_states::AbandonedRootNamedByThePoolChecker;
    use singlefs_core::address::DeviceOffsetInBytes;
    /// 改掉的那一段在数据单元里的偏移与长度：头之后、载荷里的一段（改哪一段不要紧，只要那一份的位置项校验和对不上）。
    /// 盘按整扇区读写：读写一个扇区，只翻其中头 OVERWRITTEN_BYTES 个字节。
    const OVERWRITTEN_BYTES_OFFSET_IN_THE_UNIT: u64 = 4096;
    const OVERWRITTEN_BYTES: usize = 64;
    let sector_bytes =
        usize::try_from(singlefs_harness::memory_pool::SECTOR_BYTES).expect("扇区字节数装进 usize");
    let (pool, abandoned) =
        scenarios_of_random_histories::pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version(tag);
    let mut image = pool.memory_pool();
    let location = abandoned
        .data_pointers
        .first()
        .expect("C 那一版的文件至少一个数据单元")
        .locations[0];
    let offset = DeviceOffsetInBytes(
        location.slot.to_device_offset().0 + OVERWRITTEN_BYTES_OFFSET_IN_THE_UNIT,
    );
    let device = image
        .devices
        .get_mut(&location.device)
        .expect("数据单元那一份所在的盘在池里");
    let mut sector = device.read(offset, sector_bytes);
    for byte in &mut sector[..OVERWRITTEN_BYTES] {
        *byte = !*byte;
    }
    device.write(offset, &sector);
    (
        image,
        AbandonedRootNamedByThePoolChecker {
            instance: abandoned.root.instance,
            checkpoint_txg: abandoned.root.checkpoint_txg,
        },
    )
}

/// 一张镜像建成的给定状态表（基线是它、写表为空、一个空的持久集合）。
fn table_of_one_image<'image>(
    table_name: &str,
    image: &'image singlefs_harness::memory_pool::MemoryPool,
) -> GivenStatesOfATable<'image> {
    GivenStatesOfATable {
        table_name: table_name.to_string(),
        base: image,
        writes: &[],
        segments: &[],
        seams: &[],
        persisted_sets: vec![Vec::new()],
    }
}

/// 从库里复用回来的结论也带 I-7.4 点名的被抛弃根：被抛弃根那一半判红的镜像（[`image_whose_abandoned_root_walks_into_an_overwritten_unit`]）
/// 在这条用例自己那一份库里判两趟（照本机配置：配了 GPU 只由 GPU 判器判）。第一趟现判、存进库，第二趟同一把版本下整张复用；
/// 两趟的结论都红 I-7.4、都点名 C 那一条根。判别力（`crates/mutations.tsv`）：存进库时点名的根丢掉，第二趟读回的是空的，这条红。
#[test]
fn the_abandoned_root_the_pool_checker_names_comes_back_with_the_conclusions_reused_from_the_library(
) {
    const TEST_NAME: &str =
        "the_abandoned_root_the_pool_checker_names_comes_back_with_the_conclusions_reused_from_the_library";
    let (image, expected_root) = image_whose_abandoned_root_walks_into_an_overwritten_unit(
        "given-states-named-abandoned-root-reused",
    );
    let table = table_of_one_image("被抛弃根那一半判红的镜像", &image);
    let mut runs = Vec::new();
    for library_at_the_start in [
        LibraryOfTheTestAtTheStart::EmptiedWhenTheRecordStageRuns,
        LibraryOfTheTestAtTheStart::KeptFromTheEarlierRunOfTheTest,
    ] {
        let engine = start_the_engine_on_the_library_of_the_test(
            TEST_NAME,
            Some(TEST_NAME),
            library_at_the_start,
        );
        let judged = judge_one_image(
            &engine,
            &PoolCheckerImage {
                label: table.table_name.clone(),
                image: image.clone(),
                writes: Vec::new(),
                segments: Vec::new(),
                persisted_sets: vec![Vec::new()],
                expected: PoolCheckerExpectation::only_the_named_invariants(),
                the_gpu_may_refuse_its_tables_because: None,
            },
            ContainerCapacityKnobs::PRODUCT,
        );
        let run = engine.finish();
        if !the_run_counts(&run, std::slice::from_ref(&judged), TEST_NAME) {
            return;
        }
        runs.push((judged, run));
    }
    let [(first, first_run), (second, second_run)] = runs.as_slice() else {
        panic!("两趟各交回一份结论");
    };
    assert_eq!(
        (
            first_run.tally.tables_judged_here,
            first_run.tally.tables_reused
        ),
        (1, 0),
        "第一趟从清空的库起、现判：{:?}",
        first_run.tally
    );
    assert_eq!(
        (
            second_run.tally.tables_judged_here,
            second_run.tally.tables_reused
        ),
        (0, 1),
        "第二趟同一把版本下整张复用：{:?}",
        second_run.tally
    );
    for (which_run, judged) in [("现判的那一趟", first), ("复用的那一趟", second)] {
        let [state] = judged.states.as_slice() else {
            panic!("一张镜像一个状态");
        };
        assert!(
            is_violated(state, singlefs_checker_tier::crash_given_states::INVARIANT_WHOSE_DETAIL_NAMES_ABANDONED_ROOTS),
            "{which_run}：被抛弃根那一半判 I-7.4 红（违例 {:?}）",
            state.pool_checker_violations()
        );
        assert_eq!(
            state.abandoned_roots_named_by_the_pool_checker,
            [expected_root],
            "{which_run}：I-7.4 的违例细节点名被抛弃的 C 那一条根"
        );
    }
}

/// 一张给定状态表在环境守卫里判一趟（用例自己的一份多机配置与一份库，起跑先清空）：开 GPU 那一趟显卡先后照调用方那一份
/// （`common_gpu_cards_of_the_caller::card_priority_value_of_the_caller`）、机器角色照调用方那一份。交回这张表的结论与引擎那一趟。
#[cfg(feature = "gpu")]
fn conclusions_of_a_table_judged_with_the_gpu(
    table: &GivenStatesOfATable<'_>,
    tag: &str,
    gpu_enabled: bool,
) -> (
    singlefs_checker_tier::crash_given_states::GivenStatesConclusionsOfATable,
    GivenStatesRun,
) {
    use singlefs_checker_tier::crash_amplification::CrashAmplificationEnvironment;
    use singlefs_checker_tier::crash_given_states::GivenStatesJudge;
    use singlefs_checker_tier::gpu_card_configuration::{
        GpuConfigurationOfTheCaller, CARDS_BY_PRIORITY_KEY, HOST_ROLE_ENVIRONMENT_VARIABLE,
        MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
    };
    let directory =
        singlefs_checker_tier::crash_library::amplification_unit_tests_library_directory_emptied_at_the_start(
            tag,
        )
        .expect("流水线单元测试那一类里这条用例的目录");
    let configuration = directory.join("multi-host.env");
    std::fs::write(
        &configuration,
        if gpu_enabled {
            "ENABLE_GPU=1\n"
        } else {
            "ENABLE_GPU=0\n"
        },
    )
    .expect("写得了配置");
    let library = directory.join("library");
    let host = GpuConfigurationOfTheCaller::of_this_process()
        .host()
        .expect("调用方给的机器角色是 local 或 peer");
    let cards = if gpu_enabled {
        common_gpu_cards_of_the_caller::card_priority_value_of_the_caller(
            &repository_root_of_this_crate(),
        )
    } else {
        String::new()
    };
    let mut settings: Vec<(&str, &std::ffi::OsStr)> = vec![
        (
            MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            configuration.as_os_str(),
        ),
        (
            CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
            library.as_os_str(),
        ),
    ];
    if gpu_enabled {
        settings.push((
            HOST_ROLE_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new(host.name()),
        ));
        settings.push((CARDS_BY_PRIORITY_KEY, std::ffi::OsStr::new(&cards)));
    }
    let environment = CrashAmplificationEnvironment::cleared_then_set(&settings);
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: ITEM_NAME.to_string(),
            setting_file: THIS_FILE.to_string(),
            seeds_and_scale: format!("abandoned-root-named-on-the-gpu-and-the-cpu {tag}"),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        &repository_root_of_this_crate(),
    )
    .expect("引擎照这一趟的配置起得来");
    let judged = engine
        .judge_the_given_states(table)
        .unwrap_or_else(|reason| panic!("判不了（不退回别处判）：{reason}"));
    let run = engine.finish();
    drop(environment);
    (judged, run)
}

/// GPU 与 CPU 对拍 I-7.4 点名的被抛弃根（两趟独立、各从自己那一份库读回）：被抛弃根那一半判红的镜像
/// （[`image_whose_abandoned_root_walks_into_an_overwritten_unit`]）一趟由 GPU 判器判（走读内核记下点名的根）、一趟由 CPU 判器判
/// （从违例细节里读出），两趟的对拍字相同、点名的根逐个相同，都是 C 那一条。要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
/// 判别力（`crates/mutations.tsv`）：走读内核不记点名的根，GPU 那一趟点名的是空的，这条红。
#[cfg(feature = "gpu")]
#[test]
fn the_gpu_judge_names_the_same_abandoned_root_as_the_cpu_judge_on_an_image_whose_abandoned_root_walks_into_an_overwritten_unit(
) {
    use singlefs_checker_tier::crash_amplification::JudgedBy;
    use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    const TEST_NAME: &str =
        "the_gpu_judge_names_the_same_abandoned_root_as_the_cpu_judge_on_an_image_whose_abandoned_root_walks_into_an_overwritten_unit";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let (image, expected_root) = image_whose_abandoned_root_walks_into_an_overwritten_unit(
        "given-states-named-abandoned-root-paired",
    );
    let table = table_of_one_image("被抛弃根那一半判红的镜像", &image);
    let (on_the_gpu, gpu_run) = conclusions_of_a_table_judged_with_the_gpu(
        &table,
        "given-states-named-abandoned-root-on-the-gpu",
        true,
    );
    let (on_the_cpu, cpu_run) = conclusions_of_a_table_judged_with_the_gpu(
        &table,
        "given-states-named-abandoned-root-on-the-cpu",
        false,
    );
    assert_eq!(
        (gpu_run.judged_by, cpu_run.judged_by),
        (JudgedBy::GpuJudge, JudgedBy::CpuJudge),
        "一趟由 GPU 判器判、一趟由 CPU 判器判"
    );
    assert_eq!(
        (
            gpu_run.tally.states_judged_by_the_gpu_here,
            gpu_run.tally.states_the_gpu_could_not_judge.clone()
        ),
        (1, Vec::<String>::new()),
        "GPU 那一趟从空库起、这一个状态在卡上判得了"
    );
    let [gpu_state] = on_the_gpu.states.as_slice() else {
        panic!("一张镜像一个状态");
    };
    let [cpu_state] = on_the_cpu.states.as_slice() else {
        panic!("一张镜像一个状态");
    };
    assert_eq!(
        gpu_state.words, cpu_state.words,
        "两趟的对拍字相同（违例位、判到位、两遍恢复落到的根）"
    );
    assert!(
        is_violated(
            cpu_state,
            singlefs_checker_tier::crash_given_states::INVARIANT_WHOSE_DETAIL_NAMES_ABANDONED_ROOTS
        ),
        "被抛弃根那一半判 I-7.4 红：{:?}",
        cpu_state.pool_checker_violations()
    );
    assert_eq!(
        cpu_state.abandoned_roots_named_by_the_pool_checker,
        [expected_root],
        "CPU 判器从违例细节里读出的是被抛弃的 C 那一条根"
    );
    assert_eq!(
        gpu_state.abandoned_roots_named_by_the_pool_checker,
        cpu_state.abandoned_roots_named_by_the_pool_checker,
        "GPU 判器点名的根与 CPU 判器的逐个相同"
    );
}

/// 旋钮栏进块键（用户 2026-10-14 弹窗定「给定状态表加旋钮栏」）：同一张镜像（墓碑容器压到一片 1 条写出来的那一张）、同一个表名，
/// 旋钮栏一张不填（产品容量：I-9.21 红）、一张填墓碑容器旋钮值 1（一条不红），在同一趟、同一份清空过的库里先后判：两张各判各的、
/// 一张都不复用，判出的违例各照各的旋钮栏。判别力（`crates/mutations.tsv`）：块键不收旋钮栏，后判的那张复用先判那张的结论、判出 I-9.21 红，这条红。
#[test]
fn two_tables_of_one_name_and_one_image_under_different_container_capacity_knobs_are_each_judged_under_their_own_knobs(
) {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
        knobs_of_the_tombstone_container_only_at,
        pool_with_a_capped_tombstone_tree_handing_its_product_image_out,
        TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    };
    use singlefs_checker::image::NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE;
    const TEST_NAME: &str =
        "two_tables_of_one_name_and_one_image_under_different_container_capacity_knobs_are_each_judged_under_their_own_knobs";
    let pool = pool_with_a_capped_tombstone_tree_handing_its_product_image_out(
        &mut PoolCheckerImages::left_to_the_checker_tier(),
    );
    let image = PoolCheckerImage {
        label: "墓碑容器压到一片 1 条".to_string(),
        image: pool.memory_pool(),
        writes: Vec::new(),
        segments: Vec::new(),
        persisted_sets: vec![Vec::new()],
        expected: PoolCheckerExpectation::only_the_named_invariants(),
        the_gpu_may_refuse_its_tables_because: None,
    };
    let engine = start_the_engine(TEST_NAME, Some(TEST_NAME));
    let under_the_product_capacity =
        judge_one_image(&engine, &image, ContainerCapacityKnobs::PRODUCT);
    let under_the_tombstone_knob = judge_one_image(
        &engine,
        &image,
        knobs_of_the_tombstone_container_only_at(TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB),
    );
    let run = engine.finish();
    let conclusions = [under_the_product_capacity, under_the_tombstone_knob];
    if !the_run_counts(&run, &conclusions, TEST_NAME) {
        return;
    }
    assert_eq!(
        (run.tally.tables_judged_here, run.tally.tables_reused),
        (2, 0),
        "同名、同一张镜像、旋钮栏不同的两张表各判各的，一张都不复用：{:?}",
        run.tally
    );
    let violated_of = |judged: &ConclusionsOfAnImage| -> Vec<&'static str> {
        let [state] = judged.states.as_slice() else {
            panic!("一张镜像一个状态");
        };
        IMPLEMENTED_INVARIANTS
            .iter()
            .copied()
            .filter(|invariant| is_violated(state, invariant))
            .collect()
    };
    assert_eq!(
        violated_of(&conclusions[0]),
        [NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE],
        "旋钮栏不填（产品容量）：只红 I-9.21"
    );
    assert_eq!(
        violated_of(&conclusions[1]),
        Vec::<&str>::new(),
        "旋钮栏填墓碑容器旋钮值 1：一条不红"
    );
}

// ============================== 叶条目比字段表宽的记账树（GPU 收 inode 号水位行那一处的叶条目宽要等于 34） ==============================

/// 记账树一片叶的条目比字段表宽一个字节（34 → 35，每一条末尾补一个 0）的镜像：两棵树压小写出第一个文件版本
/// （`scenarios_of_accounting_and_mapping_tree_split` 的 [`SMALL_ACCOUNTING`]，记账树两层四片叶），改记账树按 bump 次序第一片叶、
/// 补好引用链上的校验和（`common_tree_split::propagate_the_new_checksum`），镜像上只剩这一处说话。CPU 主走读照 I-1.10 判条目宽（等于字段表宽），
/// 不等就不往下走、这棵树不算走全，I-9.30 收不到水位行、报不适用；GPU 扫描内核收水位行那一处照同一条判据（叶 34、内部 108 都要等于）。
///
/// [`SMALL_ACCOUNTING`]: scenarios_of_accounting_and_mapping_tree_split::SMALL_ACCOUNTING
#[cfg(feature = "gpu")]
fn image_whose_accounting_leaf_entries_are_wider_than_the_field_table(
) -> singlefs_harness::memory_pool::MemoryPool {
    use common_tree_split::{
        capacities, propagate_the_new_checksum, write_unit_to_the_same_slot_on_both_devices,
        TreeSplitPool,
    };
    use scenarios_of_accounting_and_mapping_tree_split::{
        node_units, SMALL_ACCOUNTING, SMALL_CENTRAL_MAPPING,
    };
    use singlefs_core::checksum::crc32_castagnoli;
    use singlefs_core::transaction::MultiLevelCodeTwoTree;
    use singlefs_core::unit::{build_index_node, parse_index_node};
    let pool = TreeSplitPool::with_the_first_file_version_under(capacities(
        SMALL_ACCOUNTING,
        SMALL_CENTRAL_MAPPING,
    ));
    let output = &pool.output;
    let leaf_unit = node_units(output, MultiLevelCodeTwoTree::Accounting)
        .into_iter()
        .find(|unit| {
            parse_index_node(&unit.bytes)
                .expect("记账树的节点解得开")
                .level
                == 0
        })
        .expect("压小的记账树长成两层，有叶");
    let leaf = parse_index_node(&leaf_unit.bytes).expect("记账树的叶解得开");
    let wider_entries: Vec<Vec<u8>> = leaf
        .entries
        .iter()
        .map(|entry| {
            let mut wider = entry.clone();
            wider.push(0);
            wider
        })
        .collect();
    let wider_leaf = build_index_node(
        leaf.tree,
        leaf.level,
        leaf.key_width,
        &leaf.smallest_key,
        &leaf.largest_key,
        leaf.birth_txg,
        &common::parameters().filesystem_identifier,
        leaf.instance,
        leaf.birth_sequence,
        u16::try_from(leaf.entry_width + 1).expect("条目宽装得进 u16"),
        &wider_entries,
    );
    let mut image = pool.memory_pool();
    write_unit_to_the_same_slot_on_both_devices(&mut image, leaf_unit.slot.0, &wider_leaf);
    let units: Vec<(u64, usize)> = output
        .units
        .iter()
        .map(|unit| (unit.slot.0, unit.bytes.len()))
        .collect();
    propagate_the_new_checksum(
        &mut image,
        &units,
        (
            leaf_unit.slot.0,
            crc32_castagnoli(&leaf_unit.bytes),
            crc32_castagnoli(&wider_leaf),
        ),
    );
    image
}

/// GPU 与 CPU 对拍叶条目比字段表宽的记账树（两趟独立、各从自己那一份库读回）：[`image_whose_accounting_leaf_entries_are_wider_than_the_field_table`]
/// 一趟由 GPU 判器判、一趟由 CPU 判器判，对拍字相同；CPU 那一趟 I-1.10 红、I-9.30 没判到（这棵树不算走全、一行水位行都不收）。
/// 要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
/// 判别力（`crates/mutations.tsv`）：GPU 扫描内核收水位行时叶条目宽改回只要求不小于 34，GPU 那一趟读过这片叶、判到 I-9.30，这条红。
#[cfg(feature = "gpu")]
#[test]
fn an_accounting_leaf_wider_than_its_field_table_is_judged_alike_on_the_gpu_and_the_cpu_and_its_watermark_rows_stay_unread(
) {
    use singlefs_checker_tier::crash_amplification::JudgedBy;
    use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    /// 条目宽要等于字段表宽的那一条（池级 checker 主走读照它判、不等就不往下走）。
    const ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE: &str = "I-1.10";
    /// 记账树 inode 号水位行那一条（只取走全了的记账树的行）。
    const INODE_NUMBER_WATERMARK_STEPS: &str = "I-9.30";
    const TEST_NAME: &str =
        "an_accounting_leaf_wider_than_its_field_table_is_judged_alike_on_the_gpu_and_the_cpu_and_its_watermark_rows_stay_unread";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let image = image_whose_accounting_leaf_entries_are_wider_than_the_field_table();
    let table = table_of_one_image("记账树一片叶的条目宽 35", &image);
    let (on_the_gpu, gpu_run) = conclusions_of_a_table_judged_with_the_gpu(
        &table,
        "given-states-accounting-leaf-wider-than-its-field-table-on-the-gpu",
        true,
    );
    let (on_the_cpu, cpu_run) = conclusions_of_a_table_judged_with_the_gpu(
        &table,
        "given-states-accounting-leaf-wider-than-its-field-table-on-the-cpu",
        false,
    );
    assert_eq!(
        (gpu_run.judged_by, cpu_run.judged_by),
        (JudgedBy::GpuJudge, JudgedBy::CpuJudge),
        "一趟由 GPU 判器判、一趟由 CPU 判器判"
    );
    assert_eq!(
        (
            gpu_run.tally.states_judged_by_the_gpu_here,
            gpu_run.tally.states_the_gpu_could_not_judge.clone()
        ),
        (1, Vec::<String>::new()),
        "GPU 那一趟从空库起、这一个状态在卡上判得了"
    );
    let [gpu_state] = on_the_gpu.states.as_slice() else {
        panic!("一张镜像一个状态");
    };
    let [cpu_state] = on_the_cpu.states.as_slice() else {
        panic!("一张镜像一个状态");
    };
    assert!(
        is_violated(cpu_state, ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE),
        "前提：CPU 主走读判条目宽比字段表宽（{ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE}）红：{:?}",
        cpu_state.pool_checker_violations()
    );
    assert!(
        !is_judged(cpu_state, INODE_NUMBER_WATERMARK_STEPS),
        "前提：这棵记账树不算走全，CPU 判 {INODE_NUMBER_WATERMARK_STEPS} 不适用"
    );
    assert_eq!(
        gpu_state.words, cpu_state.words,
        "两趟的对拍字相同（违例位、判到位、两遍恢复落到的根）"
    );
}
