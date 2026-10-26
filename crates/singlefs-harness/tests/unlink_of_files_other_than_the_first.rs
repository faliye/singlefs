//! 第一个文件之外还有文件带数据单元时的删除各步（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1、M2、M3，
//! 用户 2026-10-19 定的 K-U5 β 时序级读法）：删任一号、删第一个文件而别的文件还在、删到有条目的号集合空，与 extent 上段的规范形
//! （形状只由有条目的 inode 号集合定，写与删共用一个计划）。
//!
//! 每条用例一个场景：攻方量过的两形（只撤拒绝的孤儿 `research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift.txt`、批截第一个文件
//! `opus-lift_count.txt`）改完之后都做成；上段长到常规层 再删第一个文件（攻方 E 段，此前断言 panic）做成、降层；正推模型规则 A 的三种终态
//! （`research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out` 第 11–16 行）在实现上各一条；同一个有条目的号集合经两段历史得到
//! 同一个上段；删之前就有的别的文件删完第一个文件之后照常能写（β）。每一版都交池级 checker 判（连同新立的 I-9.40），一条违例都不许有。

//! 本档只核实现行为；同名场景交出的镜像由 checker 档给定状态引擎两段判。

mod common;
mod common_images_for_the_checker_tier;
mod common_pointer_head_corruption;
#[path = "common_images_for_the_checker_tier/unlink_of_files_other_than_the_first.rs"]
mod scenarios;

#[test]
fn unlinking_another_file_holding_three_data_units_releases_them_and_drops_its_upper_entry() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::unlinking_another_file_holding_three_data_units_releases_them_and_drops_its_upper_entry);
}

#[test]
fn each_deletion_batch_of_another_file_cuts_only_that_file_and_the_first_file_reads_back_unchanged()
{
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::each_deletion_batch_of_another_file_cuts_only_that_file_and_the_first_file_reads_back_unchanged);
}

#[test]
fn unlinking_the_first_file_after_the_upper_segment_grew_to_level_one_keeps_the_leaf_of_the_other_file(
) {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::unlinking_the_first_file_after_the_upper_segment_grew_to_level_one_keeps_the_leaf_of_the_other_file);
}

#[test]
fn deleting_inode_five_hundred_beside_the_first_file_lowers_the_upper_segment_to_the_first_leaf() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::deleting_inode_five_hundred_beside_the_first_file_lowers_the_upper_segment_to_the_first_leaf);
}

#[test]
fn deleting_the_first_file_beside_inode_five_hundred_leaves_its_leaf_under_a_level_one_root() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::deleting_the_first_file_beside_inode_five_hundred_leaves_its_leaf_under_a_level_one_root);
}

#[test]
fn deleting_both_files_in_either_order_ends_in_one_zero_entry_upper_root() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::deleting_both_files_in_either_order_ends_in_one_zero_entry_upper_root);
}

#[test]
fn one_set_of_files_with_data_reaches_one_upper_segment_through_two_histories() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::one_set_of_files_with_data_reaches_one_upper_segment_through_two_histories);
}

#[test]
fn a_file_created_before_the_first_file_was_deleted_is_still_written_after_it() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::a_file_created_before_the_first_file_was_deleted_is_still_written_after_it);
}

#[test]
fn a_writable_remount_after_the_intent_of_another_file_continues_its_deletion_to_the_end() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::a_writable_remount_after_the_intent_of_another_file_continues_its_deletion_to_the_end);
}

#[test]
fn a_root_left_one_level_above_the_only_leaf_with_an_entry_reddens_the_upper_shape_invariant() {
    common_images_for_the_checker_tier::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(scenarios::a_root_left_one_level_above_the_only_leaf_with_an_entry_reddens_the_upper_shape_invariant);
}
