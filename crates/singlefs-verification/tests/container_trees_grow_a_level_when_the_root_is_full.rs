//! 墓碑树与谱系树根满长高（D18（块里携带什么信息） 已定项 24 第 1、5 条「根满时长高，照 D8（核心索引结构） 已定项 6」，K-R3-3）：
//! 码 2 节点装满时末尾分裂，根满时长一层——新根与分出的右半各一个码 2，旧根一个字节不动、留在原位置当根之下的节点；
//! 只供测试的旋钮把容器与节点压小（一片 1 条、一个节点 2 条），旋钮不落盘；重建读层级 > 1 的根。池是两块 4 GiB 盘。
//! 池级 checker 从镜像外拿墓碑容器的旋钮值（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T5 取 K1）：
//! 这些镜像带旋钮值判，I-9.22（测试镜像里墓碑容器不超旋钮值） 按旋钮值判；同一张镜像当没装旋钮判 I-9.21 红。
//! 谱系容器同样压到一片 1 条，旋钮值一并给谱系的（I-9.37；不给的话回退两次写出的两片谱系容器按产品容量判 I-9.36 红）。
//! G2：三棵容器树都压到一片 1 条的一段固定种子随机历史（建号、unlink、回退），每一步带旋钮值判一条不红；
//! H1：同一段历史压到一片 2 条，旋钮下「不是最右的那片装满旋钮值」真判到多片。
//! 池级 checker 那一遍都搬进给定状态引擎两段（给定状态表加旋钮栏）：每条用例的场景在
//! `common_images_for_the_given_states_engine/container_trees_grow_a_level_when_the_root_is_full.rs` 里对应的场景函数 `run_the_scenario_where_<用例名>`，这里只核实现的行为；
//! 镜像连同该判成什么交给 `crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`
//! 的同名用例，那一侧照场景配的旋钮值建给定状态表的旋钮栏、经给定状态引擎判。
mod common;
mod common_deletion_in_progress;
mod common_images_for_the_given_states_engine;
mod common_pointer_head_corruption;
mod common_shared_by_harness_scenarios;
#[path = "common_images_for_the_given_states_engine/container_trees_grow_a_level_when_the_root_is_full.rs"]
mod scenarios_of_container_trees_grow_a_level_when_the_root_is_full;

use common_images_for_the_given_states_engine::PoolCheckerImages;
use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::KnobsTheTombstoneContainerOfTwoRecordsIsJudgedUnder;

/// 第 3 片墓碑容器长一层（场景、要判的镜像与该判成什么见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place`）。
#[test]
fn the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place() {
    scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place(
        &mut PoolCheckerImages::left_to_the_given_states_engine(),
    );
}

/// 第 5 片墓碑容器长到三层（见对应的场景函数）。
#[test]
fn the_fifth_tombstone_container_grows_the_tree_to_three_levels_and_rebuild_reads_it() {
    scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_the_fifth_tombstone_container_grows_the_tree_to_three_levels_and_rebuild_reads_it(
        &mut PoolCheckerImages::left_to_the_given_states_engine(),
    );
}

/// 压小扇出写出来的两层树换回产品扇出接着写（见对应的场景函数）。
#[test]
fn a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout() {
    scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout(
        &mut PoolCheckerImages::left_to_the_given_states_engine(),
    );
}

/// 第三次回退谱系树长一层（见对应的场景函数）。
#[test]
fn the_third_rollback_grows_the_lineage_tree_and_the_checkpoint_cost_counts_the_internal_level() {
    scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_the_third_rollback_grows_the_lineage_tree_and_the_checkpoint_cost_counts_the_internal_level(
        &mut PoolCheckerImages::left_to_the_given_states_engine(),
    );
}

/// 墓碑容器压到一片 1 条写出来的镜像（见对应的场景函数）：当没装旋钮判与带墓碑容器旋钮值 1 判都在给定状态用例。
#[test]
fn a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant() {
    scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant(
        &mut PoolCheckerImages::left_to_the_given_states_engine(),
    );
}

/// 墓碑容器压到一片 2 条写出来的镜像（见对应的场景函数）：三组旋钮值各判一趟在给定状态用例；这里搭一次场景（三组搭出来的池相同）。
#[test]
fn a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant(
) {
    scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant(
        &mut PoolCheckerImages::left_to_the_given_states_engine(),
        KnobsTheTombstoneContainerOfTwoRecordsIsJudgedUnder::TwoTombstoneRecordsAndTheLineageKnob,
    );
}

/// 三棵容器树都压到一片 1 条的随机历史（见对应的场景函数）：每一步带旋钮值判一条不红在给定状态引擎。
#[test]
fn a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step() {
    scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step(
        &mut PoolCheckerImages::left_to_the_given_states_engine(),
    );
}

/// 同一段随机历史压到一片 2 条（见对应的场景函数）。
#[test]
fn a_random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step() {
    scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step(
        &mut PoolCheckerImages::left_to_the_given_states_engine(),
    );
}
