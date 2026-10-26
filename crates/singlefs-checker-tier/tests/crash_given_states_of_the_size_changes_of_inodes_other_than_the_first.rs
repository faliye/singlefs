//! checker 档模块：无（判法全在共用模块 `tests/common_given_states_of_harness_scenario_images/mod.rs` 里，本文件只引它与 harness 档的场景文件）
//! harness 档 `tests/size_changes_of_inodes_other_than_the_first.rs` 那几个场景（第一个文件之外的 inode 缩短与变长，判决
//! `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2、S3、S7）交出来的镜像经给定状态引擎两段判
//! （用户 2026-10-14 弹窗定「搬进 checker 档两段」）：每条用例与 harness 档里同名的那条是同一个场景，经 `#[path]` 引同一份场景文件，
//! 判法在共用模块 `tests/common_given_states_of_harness_scenario_images/mod.rs`。配了 GPU 只由 GPU 判器判：「有记录、没有条目、size > 0」那一张
//! GPU 检查内核要与 CPU 同红 I-4.6，合法的那几张两边都一条不红（判决第二节 S2「GPU 检查内核与恢复内核同判」）。
#![cfg(feature = "verdict-store")]
#![allow(
    dead_code,
    reason = "共用模块 common_given_states_of_harness_scenario_images 里另几样判法（压小容器树、只在拒绝那一路交镜像、续用上一趟的库）这一份不用"
)]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_deletion_in_progress/mod.rs"]
mod common_deletion_in_progress;
mod common_given_states_of_harness_scenario_images;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/mod.rs"]
mod common_images_for_the_checker_tier;
#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;
#[path = "../../singlefs-harness/tests/common_truncation/mod.rs"]
mod common_truncation;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/size_changes_of_inodes_other_than_the_first.rs"]
mod scenarios_of_size_changes_of_inodes_other_than_the_first;

use common_given_states_of_harness_scenario_images::judge_the_images_a_harness_scenario_hands_out;

/// 与 harness 档 `crates/singlefs-harness/tests/size_changes_of_inodes_other_than_the_first.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_size_changes_of_inodes_other_than_the_first::extending_an_inode_without_an_entry_writes_a_sticky_tag_zero_entry_read_as_zeros_by_every_reader`）。
#[test]
fn extending_an_inode_without_an_entry_writes_a_sticky_tag_zero_entry_read_as_zeros_by_every_reader(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "extending_an_inode_without_an_entry_writes_a_sticky_tag_zero_entry_read_as_zeros_by_every_reader",
        scenarios_of_size_changes_of_inodes_other_than_the_first::extending_an_inode_without_an_entry_writes_a_sticky_tag_zero_entry_read_as_zeros_by_every_reader,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/size_changes_of_inodes_other_than_the_first.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_size_changes_of_inodes_other_than_the_first::extending_an_inode_inside_its_one_data_unit_rewrites_that_unit_and_every_reader_reads_the_zeros`）。
#[test]
fn extending_an_inode_inside_its_one_data_unit_rewrites_that_unit_and_every_reader_reads_the_zeros()
{
    judge_the_images_a_harness_scenario_hands_out(
        "extending_an_inode_inside_its_one_data_unit_rewrites_that_unit_and_every_reader_reads_the_zeros",
        scenarios_of_size_changes_of_inodes_other_than_the_first::extending_an_inode_inside_its_one_data_unit_rewrites_that_unit_and_every_reader_reads_the_zeros,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/size_changes_of_inodes_other_than_the_first.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_size_changes_of_inodes_other_than_the_first::extending_an_inode_across_data_units_declares_the_hole_and_every_reader_reads_zeros_up_to_the_size`）。
#[test]
fn extending_an_inode_across_data_units_declares_the_hole_and_every_reader_reads_zeros_up_to_the_size(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "extending_an_inode_across_data_units_declares_the_hole_and_every_reader_reads_zeros_up_to_the_size",
        scenarios_of_size_changes_of_inodes_other_than_the_first::extending_an_inode_across_data_units_declares_the_hole_and_every_reader_reads_zeros_up_to_the_size,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/size_changes_of_inodes_other_than_the_first.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_size_changes_of_inodes_other_than_the_first::truncating_an_inode_inside_its_one_data_unit_rewrites_it_through_an_intent_and_a_final_batch`）。
#[test]
fn truncating_an_inode_inside_its_one_data_unit_rewrites_it_through_an_intent_and_a_final_batch() {
    judge_the_images_a_harness_scenario_hands_out(
        "truncating_an_inode_inside_its_one_data_unit_rewrites_it_through_an_intent_and_a_final_batch",
        scenarios_of_size_changes_of_inodes_other_than_the_first::truncating_an_inode_inside_its_one_data_unit_rewrites_it_through_an_intent_and_a_final_batch,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/size_changes_of_inodes_other_than_the_first.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_size_changes_of_inodes_other_than_the_first::truncating_a_three_unit_inode_into_unit_zero_writes_its_truncation_field_and_releases_the_units_beyond`）。
#[test]
fn truncating_a_three_unit_inode_into_unit_zero_writes_its_truncation_field_and_releases_the_units_beyond(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "truncating_a_three_unit_inode_into_unit_zero_writes_its_truncation_field_and_releases_the_units_beyond",
        scenarios_of_size_changes_of_inodes_other_than_the_first::truncating_a_three_unit_inode_into_unit_zero_writes_its_truncation_field_and_releases_the_units_beyond,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/size_changes_of_inodes_other_than_the_first.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_size_changes_of_inodes_other_than_the_first::truncating_another_inode_onto_the_lower_leaf_boundary_rewrites_the_leaf_holding_the_cut_unit`）。
#[test]
fn truncating_another_inode_onto_the_lower_leaf_boundary_rewrites_the_leaf_holding_the_cut_unit() {
    judge_the_images_a_harness_scenario_hands_out(
        "truncating_another_inode_onto_the_lower_leaf_boundary_rewrites_the_leaf_holding_the_cut_unit",
        scenarios_of_size_changes_of_inodes_other_than_the_first::truncating_another_inode_onto_the_lower_leaf_boundary_rewrites_the_leaf_holding_the_cut_unit,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/size_changes_of_inodes_other_than_the_first.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_size_changes_of_inodes_other_than_the_first::a_truncation_of_another_inode_stopped_between_batches_continues_by_its_number_at_the_writable_remount`）。
#[test]
fn a_truncation_of_another_inode_stopped_between_batches_continues_by_its_number_at_the_writable_remount(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_truncation_of_another_inode_stopped_between_batches_continues_by_its_number_at_the_writable_remount",
        scenarios_of_size_changes_of_inodes_other_than_the_first::a_truncation_of_another_inode_stopped_between_batches_continues_by_its_number_at_the_writable_remount,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/size_changes_of_inodes_other_than_the_first.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_size_changes_of_inodes_other_than_the_first::truncating_another_inode_under_a_live_snapshot_keeps_its_replaced_data_units_on_the_deadlist`）。
#[test]
fn truncating_another_inode_under_a_live_snapshot_keeps_its_replaced_data_units_on_the_deadlist() {
    judge_the_images_a_harness_scenario_hands_out(
        "truncating_another_inode_under_a_live_snapshot_keeps_its_replaced_data_units_on_the_deadlist",
        scenarios_of_size_changes_of_inodes_other_than_the_first::truncating_another_inode_under_a_live_snapshot_keeps_its_replaced_data_units_on_the_deadlist,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/size_changes_of_inodes_other_than_the_first.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_size_changes_of_inodes_other_than_the_first::unlinking_another_inode_with_a_hole_switches_its_tag_back_and_deletes_it`）。
#[test]
fn unlinking_another_inode_with_a_hole_switches_its_tag_back_and_deletes_it() {
    judge_the_images_a_harness_scenario_hands_out(
        "unlinking_another_inode_with_a_hole_switches_its_tag_back_and_deletes_it",
        scenarios_of_size_changes_of_inodes_other_than_the_first::unlinking_another_inode_with_a_hole_switches_its_tag_back_and_deletes_it,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/size_changes_of_inodes_other_than_the_first.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_size_changes_of_inodes_other_than_the_first::a_rollback_between_two_truncations_of_different_inodes_keeps_one_entry_naming_the_target_inode`）。
#[test]
fn a_rollback_between_two_truncations_of_different_inodes_keeps_one_entry_naming_the_target_inode()
{
    judge_the_images_a_harness_scenario_hands_out(
        "a_rollback_between_two_truncations_of_different_inodes_keeps_one_entry_naming_the_target_inode",
        scenarios_of_size_changes_of_inodes_other_than_the_first::a_rollback_between_two_truncations_of_different_inodes_keeps_one_entry_naming_the_target_inode,
    );
}

/// 与 harness 档 `crates/singlefs-harness/tests/size_changes_of_inodes_other_than_the_first.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_size_changes_of_inodes_other_than_the_first::a_record_with_a_size_but_no_upper_leaf_entry_is_refused_by_the_readers_and_reddens_only_the_unit_count`）。
#[test]
fn a_record_with_a_size_but_no_upper_leaf_entry_is_refused_by_the_readers_and_reddens_only_the_unit_count(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "a_record_with_a_size_but_no_upper_leaf_entry_is_refused_by_the_readers_and_reddens_only_the_unit_count",
        scenarios_of_size_changes_of_inodes_other_than_the_first::a_record_with_a_size_but_no_upper_leaf_entry_is_refused_by_the_readers_and_reddens_only_the_unit_count,
    );
}
