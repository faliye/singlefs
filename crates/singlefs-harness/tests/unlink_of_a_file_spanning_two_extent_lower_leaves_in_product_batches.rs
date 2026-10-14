//! 真发布路径上、按产品批大小（`UnlinkBatchSize::OneExtentLowerLeaf`，一批 144 个数据单元）删一个罩着两片 extent 下段叶的文件
//! （D8（核心索引结构） 已定项 16 第 2 条「从尾部分批」；代码轮 U2：入库的删除用例此前只删 3 个单元的文件、批大小 1，只改尾叶那一支没有真发布路径上的用例）。
//! 287 个数据单元：第一片叶罩单元 0–143、第二片罩 144–286；第一批从尾部删掉 144 个（143–286），留下 143 个——第一片叶的末个单元（143）
//! 正是被删掉的第一个单元，这片叶不能整片照抄、要重写（`unlink::plan_the_extent_tree_after_a_deletion_batch` 的 `is_entirely_kept`）；
//! 第二批删完。逐版判：从盘上重建出来的那一版与发布的那一版（inode 记录、队列、数据指针、extent 树、进映射的单元）逐项相同，
//! 池级 checker 无红；删完之后冷走读答「没有文件」。草稿出处：正推腿探针 `research/prompts/m3-unlink-code-r1-sonnet-model/sonnet_probe_multi_leaf_unlink.rs`。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs"]
mod scenarios_of_unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches::a_file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches::a_file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_leaf_whose_last_unit_is_cut,
    );
}
