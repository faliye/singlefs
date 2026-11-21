//! 树与文件这一族 harness 场景交出的镜像：记账树与映射树分裂、容器树长一层、inode 号水位、多 inode 与多单元文件的读写、单实例里的覆盖写、树表次序、按身份查找、按 inode 号写、另一个文件上的快照与整理。

use super::*;

// ─── `tests/accounting_and_mapping_tree_split.rs` ───

/// 与 `crates/singlefs-verification/tests/accounting_and_mapping_tree_split.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_accounting_and_mapping_tree_split::run_the_scenario_where_under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts`）。
#[test]
fn under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts",
        scenarios_of_accounting_and_mapping_tree_split::run_the_scenario_where_under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts,
    );
}

/// 与 `crates/singlefs-verification/tests/accounting_and_mapping_tree_split.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_accounting_and_mapping_tree_split::run_the_scenario_where_an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes`）。
#[test]
fn an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes",
        scenarios_of_accounting_and_mapping_tree_split::run_the_scenario_where_an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes,
    );
}

/// 与 `crates/singlefs-verification/tests/accounting_and_mapping_tree_split.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_accounting_and_mapping_tree_split::run_the_scenario_where_pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on`）。
#[test]
fn pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on() {
    judge_the_images_a_harness_scenario_hands_out(
        "pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on",
        scenarios_of_accounting_and_mapping_tree_split::run_the_scenario_where_pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on,
    );
}

/// 与 `crates/singlefs-verification/tests/accounting_and_mapping_tree_split.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_accounting_and_mapping_tree_split::run_the_scenario_where_the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root`）。
#[test]
fn the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root",
        scenarios_of_accounting_and_mapping_tree_split::run_the_scenario_where_the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root,
    );
}

// ─── `tests/container_trees_grow_a_level_when_the_root_is_full.rs` ───

/// 场景、要判的镜像与该判成什么见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry`
/// （`crates/singlefs-verification/tests/container_trees_grow_a_level_when_the_root_is_full.rs` 里没有跑这个场景函数的用例，只有这一条判它）。
#[test]
fn capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry",
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry,
    );
}

/// 与 `crates/singlefs-verification/tests/container_trees_grow_a_level_when_the_root_is_full.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place`）；
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
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place,
    );
}

/// 与同名的 harness 用例同一个场景（见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_the_fifth_tombstone_container_grows_the_tree_to_three_levels_and_rebuild_reads_it`）；
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
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_the_fifth_tombstone_container_grows_the_tree_to_three_levels_and_rebuild_reads_it,
    );
}

/// 与 `crates/singlefs-verification/tests/container_trees_grow_a_level_when_the_root_is_full.rs` 里跑同一个场景函数的 harness 用例同一个场景（见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout`）；
/// 旋钮栏是写这张镜像时那一组（容器容量照旧压着，扇出换回产品）。
#[test]
fn tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout() {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
        knobs_of_the_tombstone_and_the_lineage_container_at,
        TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    };
    judge_the_images_a_harness_scenario_hands_out_under_the_container_capacity_knobs(
        "tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout",
        knobs_of_the_tombstone_and_the_lineage_container_at(
            TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
        ),
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout,
    );
}

/// 与同名的 harness 用例同一个场景（见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_the_third_rollback_grows_the_lineage_tree_and_the_checkpoint_cost_counts_the_internal_level`）；
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
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_the_third_rollback_grows_the_lineage_tree_and_the_checkpoint_cost_counts_the_internal_level,
    );
}

/// 与 `crates/singlefs-verification/tests/container_trees_grow_a_level_when_the_root_is_full.rs` 里跑同一个场景函数的 harness 用例同一个场景（见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant`）：
/// 旋钮栏只填墓碑容器旋钮值 1（`check_pool_image_given_the_tombstone_container_capacity_knob` 那一判）。
#[test]
fn capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant() {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
        knobs_of_the_tombstone_container_only_at, TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    };
    judge_the_images_a_harness_scenario_hands_out_under_the_container_capacity_knobs(
        "capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant",
        knobs_of_the_tombstone_container_only_at(TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB),
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant,
    );
}

/// 与 `crates/singlefs-verification/tests/container_trees_grow_a_level_when_the_root_is_full.rs` 里跑同一个场景函数的 harness 用例同一个场景（见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant`）：
/// 同一张镜像带三组旋钮值各判一趟（`KnobsTheTombstoneContainerOfTwoRecordsIsJudgedUnder` 每一种一趟，场景名接上那一组的名字）。
#[test]
fn tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant()
{
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::KnobsTheTombstoneContainerOfTwoRecordsIsJudgedUnder;
    for judged_under in KnobsTheTombstoneContainerOfTwoRecordsIsJudgedUnder::ALL {
        judge_the_images_a_harness_scenario_hands_out_under_the_container_capacity_knobs(
            &format!(
                "tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant::{}",
                judged_under.name()
            ),
            judged_under.knobs(),
            |images| {
                scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant(
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
    let mut handed_out = PoolCheckerImages::collected_for_the_given_states_engine();
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

/// 与 `crates/singlefs-verification/tests/container_trees_grow_a_level_when_the_root_is_full.rs` 里跑同一个场景函数的 harness 用例同一个场景（见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step`）：
/// 每一步的镜像旋钮栏填三棵树都是一片 1 条，一条都不红；I-9.22、I-9.37 在某一步上判成立。
#[test]
fn random_history_under_every_container_knob_reddens_no_invariant_at_any_step() {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
        knobs_of_every_container_tree_at, KNOB_INVARIANTS_HELD_AT_SOME_STEP,
    };
    judge_every_step_image_of_a_random_history_under_the_knobs_and_find_each_knob_invariant_held_at_some_step(
        "random_history_under_every_container_knob_reddens_no_invariant_at_any_step",
        knobs_of_every_container_tree_at(1),
        &KNOB_INVARIANTS_HELD_AT_SOME_STEP,
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step,
    );
}

/// 与 `crates/singlefs-verification/tests/container_trees_grow_a_level_when_the_root_is_full.rs` 里跑同一个场景函数的 harness 用例同一个场景（见
/// `scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step`）：
/// 每一步的镜像旋钮栏填三棵树都是一片 2 条，一条都不红；I-9.22、I-9.37 在某一步上判成立。
#[test]
fn random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step() {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
        knobs_of_every_container_tree_at, KNOB_INVARIANTS_HELD_AT_SOME_STEP,
    };
    judge_every_step_image_of_a_random_history_under_the_knobs_and_find_each_knob_invariant_held_at_some_step(
        "random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step",
        knobs_of_every_container_tree_at(2),
        &KNOB_INVARIANTS_HELD_AT_SOME_STEP,
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step,
    );
}

// ─── `tests/inode_number_watermark_pressed_back_reissues_a_number.rs` ───

/// 与 `crates/singlefs-verification/tests/inode_number_watermark_pressed_back_reissues_a_number.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::run_the_scenario_where_creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant`）。
#[test]
fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant",
        scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::run_the_scenario_where_creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant,
    );
}

/// 与 `crates/singlefs-verification/tests/inode_number_watermark_pressed_back_reissues_a_number.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::run_the_scenario_where_creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant`）。
#[test]
fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant",
        scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::run_the_scenario_where_creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant,
    );
}

/// 与 `crates/singlefs-verification/tests/inode_number_watermark_pressed_back_reissues_a_number.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::run_the_scenario_where_pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants`）。
#[test]
fn pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants",
        scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::run_the_scenario_where_pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants,
    );
}

// ─── `tests/many_inodes_write_and_read.rs` ───

/// 与 `crates/singlefs-verification/tests/many_inodes_write_and_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_many_inodes_write_and_read::run_the_scenario_where_creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold`）。
#[test]
fn creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold",
        scenarios_of_many_inodes_write_and_read::run_the_scenario_where_creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold,
    );
}

/// 与 `crates/singlefs-verification/tests/many_inodes_write_and_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_many_inodes_write_and_read::run_the_scenario_where_creating_one_inode_below_the_threshold_keeps_one_container_and_bumps_the_watermark_by_one`）。
#[test]
fn creating_one_inode_below_the_threshold_keeps_one_container_and_bumps_the_watermark_by_one() {
    judge_the_images_a_harness_scenario_hands_out(
        "creating_one_inode_below_the_threshold_keeps_one_container_and_bumps_the_watermark_by_one",
        scenarios_of_many_inodes_write_and_read::run_the_scenario_where_creating_one_inode_below_the_threshold_keeps_one_container_and_bumps_the_watermark_by_one,
    );
}

/// 与 `crates/singlefs-verification/tests/many_inodes_write_and_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_many_inodes_write_and_read::run_the_scenario_where_publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish`）。
#[test]
fn publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish",
        scenarios_of_many_inodes_write_and_read::run_the_scenario_where_publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish,
    );
}

// ─── `tests/multi_unit_file_write_and_read.rs` ───

/// 与 `crates/singlefs-verification/tests/multi_unit_file_write_and_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_multi_unit_file_write_and_read::run_the_scenario_where_units_written_across_the_sixty_four_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte`）。
#[test]
fn units_written_across_the_sixty_four_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "units_written_across_the_sixty_four_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte",
        scenarios_of_multi_unit_file_write_and_read::run_the_scenario_where_units_written_across_the_sixty_four_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte,
    );
}

/// 与 `crates/singlefs-verification/tests/multi_unit_file_write_and_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_multi_unit_file_write_and_read::run_the_scenario_where_the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline`）。
#[test]
fn the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline",
        scenarios_of_multi_unit_file_write_and_read::run_the_scenario_where_the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline,
    );
}

/// 与 `crates/singlefs-verification/tests/multi_unit_file_write_and_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_multi_unit_file_write_and_read::run_the_scenario_where_shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has`）。
#[test]
fn shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has() {
    judge_the_images_a_harness_scenario_hands_out(
        "shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has",
        scenarios_of_multi_unit_file_write_and_read::run_the_scenario_where_shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has,
    );
}

/// 与 `crates/singlefs-verification/tests/multi_unit_file_write_and_read.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_multi_unit_file_write_and_read::run_the_scenario_where_reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping`）。
#[test]
fn reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping",
        scenarios_of_multi_unit_file_write_and_read::run_the_scenario_where_reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping,
    );
}

// ─── `tests/overwrite_in_one_instance.rs` ───

/// 与 `crates/singlefs-verification/tests/overwrite_in_one_instance.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_overwrite_in_one_instance::run_the_scenario_where_cold_start_reads_the_second_content_and_the_pool_checker_stays_green`）。
#[test]
fn cold_start_reads_the_second_content_and_the_pool_checker_stays_green() {
    judge_the_images_a_harness_scenario_hands_out(
        "cold_start_reads_the_second_content_and_the_pool_checker_stays_green",
        scenarios_of_overwrite_in_one_instance::run_the_scenario_where_cold_start_reads_the_second_content_and_the_pool_checker_stays_green,
    );
}

/// 与 `crates/singlefs-verification/tests/overwrite_in_one_instance.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_overwrite_in_one_instance::run_the_scenario_where_repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree`）。
#[test]
fn repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree",
        scenarios_of_overwrite_in_one_instance::run_the_scenario_where_repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree,
    );
}

/// 与 `crates/singlefs-verification/tests/overwrite_in_one_instance.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_overwrite_in_one_instance::run_the_scenario_where_allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records`）。
#[test]
fn allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records",
        scenarios_of_overwrite_in_one_instance::run_the_scenario_where_allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records,
    );
}

// ─── `tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` ───

/// 与 `crates/singlefs-verification/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::run_the_scenario_where_the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker`）。
#[test]
fn the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker",
        scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::run_the_scenario_where_the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker,
    );
}

/// 与 `crates/singlefs-verification/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::run_the_scenario_where_the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds`）。
#[test]
fn the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds",
        scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::run_the_scenario_where_the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds,
    );
}

/// 与 `crates/singlefs-verification/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::run_the_scenario_where_the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places`）。
#[test]
fn the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places",
        scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::run_the_scenario_where_the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places,
    );
}

#[test]
fn every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units",
        scenarios_of_publish_lookup_by_identity::run_the_scenario_where_every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units,
    );
}

#[test]
fn every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_forty_eight_files_of_eight_data_units(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_forty_eight_files_of_eight_data_units",
        scenarios_of_publish_lookup_by_identity::run_the_scenario_where_every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_forty_eight_files_of_eight_data_units,
    );
}

#[test]
fn overwriting_another_file_under_a_live_snapshot_puts_its_replaced_units_on_the_deadlist() {
    judge_the_images_a_harness_scenario_hands_out(
        "overwriting_another_file_under_a_live_snapshot_puts_its_replaced_units_on_the_deadlist",
        scenarios_of_snapshot_and_compaction_over_a_file_other_than_the_first::run_the_scenario_where_overwriting_another_file_under_a_live_snapshot_puts_its_replaced_units_on_the_deadlist,
    );
}

#[test]
fn relocation_batch_moving_the_data_unit_of_another_file_takes_its_mapping_position_over_the_hint()
{
    judge_the_images_a_harness_scenario_hands_out(
        "relocation_batch_moving_the_data_unit_of_another_file_takes_its_mapping_position_over_the_hint",
        scenarios_of_snapshot_and_compaction_over_a_file_other_than_the_first::run_the_scenario_where_a_relocation_batch_moving_the_data_unit_of_another_file_takes_its_mapping_position_over_the_hint,
    );
}

#[test]
fn after_two_rollbacks_beta_refuses_a_write_to_an_inode_born_after_the_deletion_on_the_parent_chain(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "after_two_rollbacks_beta_refuses_a_write_to_an_inode_born_after_the_deletion_on_the_parent_chain",
        scenarios_of_writes_by_inode_number_after_rollbacks_past_the_deletion_of_the_first_file::run_the_scenario_where_after_two_rollbacks_beta_refuses_a_write_to_an_inode_born_after_the_deletion_on_the_parent_chain,
    );
}

#[test]
fn write_after_rolling_back_past_the_deletion_of_the_first_file_is_not_judged_by_beta() {
    judge_the_images_a_harness_scenario_hands_out(
        "write_after_rolling_back_past_the_deletion_of_the_first_file_is_not_judged_by_beta",
        scenarios_of_writes_by_inode_number_after_rollbacks_past_the_deletion_of_the_first_file::run_the_scenario_where_a_write_after_rolling_back_past_the_deletion_of_the_first_file_is_not_judged_by_beta,
    );
}

#[test]
fn the_interleaved_history_of_the_attack_seed_two_hundred_agrees_with_beta_on_its_parent_chain() {
    judge_the_images_a_harness_scenario_hands_out(
        "the_interleaved_history_of_the_attack_seed_two_hundred_agrees_with_beta_on_its_parent_chain",
        scenarios_of_writes_by_inode_number_in_the_interleaved_histories_of_the_attack::run_the_scenario_where_the_interleaved_history_of_the_attack_seed_two_hundred_agrees_with_beta_on_its_parent_chain,
    );
}

#[test]
fn the_interleaved_history_of_the_attack_seed_one_thousand_and_fifty_one_agrees_with_beta_on_its_parent_chain(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "the_interleaved_history_of_the_attack_seed_one_thousand_and_fifty_one_agrees_with_beta_on_its_parent_chain",
        scenarios_of_writes_by_inode_number_in_the_interleaved_histories_of_the_attack::run_the_scenario_where_the_interleaved_history_of_the_attack_seed_one_thousand_and_fifty_one_agrees_with_beta_on_its_parent_chain,
    );
}

#[test]
fn two_inodes_overwritten_in_turn_across_two_writable_remounts_each_read_back_what_was_last_written_to_it(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "two_inodes_overwritten_in_turn_across_two_writable_remounts_each_read_back_what_was_last_written_to_it",
        scenarios_of_writes_by_inode_number_of_several_files::run_the_scenario_where_two_inodes_overwritten_in_turn_across_two_writable_remounts_each_read_back_what_was_last_written_to_it,
    );
}

#[test]
fn writing_one_inode_leaves_the_data_units_and_extent_lower_nodes_of_the_other_files_untouched_on_disk(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "writing_one_inode_leaves_the_data_units_and_extent_lower_nodes_of_the_other_files_untouched_on_disk",
        scenarios_of_writes_by_inode_number_of_several_files::run_the_scenario_where_writing_one_inode_leaves_the_data_units_and_extent_lower_nodes_of_the_other_files_untouched_on_disk,
    );
}

#[test]
fn write_to_an_inode_past_the_first_upper_leaf_grows_the_upper_segment_and_every_file_reads_back_after_a_remount(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "write_to_an_inode_past_the_first_upper_leaf_grows_the_upper_segment_and_every_file_reads_back_after_a_remount",
        scenarios_of_writes_by_inode_number_of_several_files::run_the_scenario_where_a_write_to_an_inode_past_the_first_upper_leaf_grows_the_upper_segment_and_every_file_reads_back_after_a_remount,
    );
}

#[test]
fn unlink_while_a_file_other_than_the_first_holds_data_units_deletes_it_and_then_the_first_file() {
    judge_the_images_a_harness_scenario_hands_out(
        "unlink_while_a_file_other_than_the_first_holds_data_units_deletes_it_and_then_the_first_file",
        scenarios_of_writes_by_inode_number_of_several_files::run_the_scenario_where_unlink_while_a_file_other_than_the_first_holds_data_units_deletes_it_and_then_the_first_file,
    );
}

#[test]
fn writing_a_file_after_the_first_file_emptied_the_upper_segment_replaces_the_zero_entry_root() {
    judge_the_images_a_harness_scenario_hands_out(
        "writing_a_file_after_the_first_file_emptied_the_upper_segment_replaces_the_zero_entry_root",
        scenarios_of_writes_by_inode_number_of_several_files::run_the_scenario_where_writing_a_file_after_the_first_file_emptied_the_upper_segment_replaces_the_zero_entry_root,
    );
}
