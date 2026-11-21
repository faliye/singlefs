//! 场景镜像直接交 CPU 判器判的孪生用例。

use super::*;

// ============================== 场景镜像直接交 CPU 判器判的孪生用例（变异表里改坏 CPU 判器那几行点名它们） ==============================
//
// 同名的引擎用例照本机配置判：配了 GPU 只由 GPU 判器判，CPU 判器（`crash_given_states::judge_a_given_state_on_the_cpu`：两遍恢复、
// 池级 checker、记录核对器）一行都不跑，`crates/mutations.tsv` 里改坏它的那几行在开 GPU 的门禁上空转。
// 下面每条照同一个场景、同一组该判成什么，镜像不经引擎、不经判定库，直接交 CPU 判器判。

/// 跑一个 harness 场景（收镜像的那一种），每张镜像建成一张给定状态表（基线是它、写表与持久集合照场景交的，旋钮栏填
/// `container_capacity_knobs`），直接交 CPU 判器（[`judge_the_given_states_on_the_cpu`]：不经给定状态引擎与判定库，不看本机配不配 GPU），
/// 再照 [`mismatches_of_the_images`] 逐张核场景写下的池级 checker 该判成什么。结论都是这一趟现判的，判到位与违例细节都核。
/// 场景一张镜像都不交、或有一处对不上，都判红。
// gpu-exempt: cpu-judge-mutation-rows
fn judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
    scenario_name: &str,
    container_capacity_knobs: ContainerCapacityKnobs,
    scenario: impl FnOnce(&mut PoolCheckerImages),
) {
    let mut handed_out = PoolCheckerImages::collected_for_the_given_states_engine();
    scenario(&mut handed_out);
    let images = handed_out.into_collected();
    assert!(
        !images.is_empty(),
        "场景 {scenario_name} 一张镜像都没交给池级 checker"
    );
    let conclusions: Vec<ConclusionsOfAnImage> = images
        .iter()
        .map(|image| {
            let table = GivenStatesOfATable {
                table_name: image.label.clone(),
                base: &image.image,
                writes: &image.writes,
                segments: &image.segments,
                seams: &[],
                persisted_sets: image.persisted_sets.clone(),
            };
            let judged = judge_the_given_states_on_the_cpu(GivenStatesOfATableWithItsColumns {
                table: &table,
                container_capacity_knobs,
                claimed_versions:
                    ClaimedVersionsOfTheStates::EveryStateLandedOnByTheConsultedRecovery,
            });
            assert_eq!(
                judged.states.len(),
                image.persisted_sets.len(),
                "{}：CPU 判器逐状态交回结论，与表里的状态同序",
                image.label
            );
            ConclusionsOfAnImage {
                states: judged.states,
                source: judged.source,
            }
        })
        .collect();
    let mismatches = mismatches_of_the_images(&images, &conclusions, true);
    assert!(
        mismatches.is_empty(),
        "场景 {scenario_name} 交出的镜像上 CPU 判器判得与场景写下的对不上：\n{}",
        mismatches.join("\n")
    );
}

/// [`checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「步 3：I-3.8 判定恒真」）：池级 checker 的 I-3.8 恒判成立，场景写下的 I-3.8 红没红，这条红。
#[test]
fn on_the_cpu_judge_checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_writable_mount_takes_a_second_instance::run_the_scenario_where_checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root,
    );
}

/// [`the_pool_checker_walks_both_pages_of_a_chain_and_reddens_only_the_second_page_left_out_of_the_allocation_records`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「实例表第二片：checker 拿第 0 片的身份判链上每一片」）：第二片照第 0 片的身份判，I-1.1 在实例表一层合法的链上多红，这条红。
#[test]
fn on_the_cpu_judge_the_pool_checker_walks_both_pages_of_a_chain_and_reddens_only_the_second_page_left_out_of_the_allocation_records(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "the_pool_checker_walks_both_pages_of_a_chain_and_reddens_only_the_second_page_left_out_of_the_allocation_records",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_instance_table_chain_read::run_the_scenario_where_the_pool_checker_walks_both_pages_of_a_chain_and_reddens_only_the_second_page_left_out_of_the_allocation_records,
    );
}

/// [`the_same_instance_on_both_pages_reddens_the_instance_table_invariant_and_the_second_page_left_out_of_the_allocation_records`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「实例表第二片：checker 按片各判行唯一，不按整张表」）：两片上同一个实例不再判重，该红的那一条没红，这条红。
#[test]
fn on_the_cpu_judge_the_same_instance_on_both_pages_reddens_the_instance_table_invariant_and_the_second_page_left_out_of_the_allocation_records(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "the_same_instance_on_both_pages_reddens_the_instance_table_invariant_and_the_second_page_left_out_of_the_allocation_records",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_instance_table_chain_read::run_the_scenario_where_the_same_instance_on_both_pages_reddens_the_instance_table_invariant_and_the_second_page_left_out_of_the_allocation_records,
    );
}

/// [`the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「树分裂 checker：孩子层级不是父层级减一也判成立」「树分裂 checker：多层码 2 树的条目宽不判 I-1.10」）：
/// 两处判定恒成立，场景里对应那张弄坏的镜像该红的没红，这条红。
#[test]
fn on_the_cpu_judge_the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root() {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_accounting_and_mapping_tree_split::run_the_scenario_where_the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root,
    );
}

/// [`the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「K2 × checker：extent 下段节点按位置规定罩的那一段上端少罩最后一个单元的字节」）：多单元文件上 I-1.1 判红，这条红。
#[test]
fn on_the_cpu_judge_the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_multi_unit_file_write_and_read::run_the_scenario_where_the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline,
    );
}

/// [`the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「checker 的「4 个不同状态」不去重」）：I-7.9 的上限算宽，场景写下的判法对不上，这条红。
#[test]
fn on_the_cpu_judge_the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_reuse_after_raising_the_floor::run_the_scenario_where_the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker,
    );
}

/// [`the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「I-9.16 树表排序：checker 读树表时不判」）：I-9.16 一次都判不到，场景写下的「判成立」对不上，这条红。
#[test]
fn on_the_cpu_judge_the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::run_the_scenario_where_the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker,
    );
}

/// [`pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants`]
/// 那个场景，镜像直接交 CPU 判器判。判别力（`crates/mutations.tsv`「C I-9.29：同号几个出生代也放过」）：重发的号两个出生代不红，这条红。
#[test]
fn on_the_cpu_judge_pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_inode_number_watermark_pressed_back_reissues_a_number::run_the_scenario_where_pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants,
    );
}

/// [`raising_the_floor_past_the_gap_keeps_every_invariant_green`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「两段课题 4-3：池级 checker 把每条判定都报成违例」）：每条都报违例，场景写下的「一条不红」对不上，这条红。
#[test]
fn on_the_cpu_judge_raising_the_floor_past_the_gap_keeps_every_invariant_green() {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "raising_the_floor_past_the_gap_keeps_every_invariant_green",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic::run_the_scenario_where_raising_the_floor_past_the_gap_keeps_every_invariant_green,
    );
}

/// [`the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place`] 那个场景，旋钮栏照写这张镜像时那一组（墓碑与谱系各一片 1 条），
/// 镜像直接交 CPU 判器判。判别力（`crates/mutations.tsv`「给定状态表旋钮栏 CPU 判器不照旋钮栏判」）：池级 checker 按产品容量判、容量那几条红，这条红。
#[test]
fn on_the_cpu_judge_the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place() {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
        knobs_of_the_tombstone_and_the_lineage_container_at,
        TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    };
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place",
        knobs_of_the_tombstone_and_the_lineage_container_at(
            TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
        ),
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place,
    );
}

/// 被抛弃根那一半判 I-7.4 红的镜像（[`image_whose_abandoned_root_walks_into_an_overwritten_unit`]）直接交 CPU 判器判：判 I-7.4 红，
/// 从违例细节里读出的被抛弃根恰好是 C 那一条。同一张镜像经引擎判、别的机器照用的那一条是
/// [`the_abandoned_root_the_pool_checker_names_comes_back_with_the_red_conclusions_another_host_judged`]（配了 GPU 只由 GPU 判器判）。
/// 判别力（`crates/mutations.tsv`「CPU 判器 I-7.4 违例细节里点名的被抛弃根读出来丢掉一个」）：点名的根读出来是空的，这条红。
// gpu-exempt: cpu-judge-mutation-rows
#[test]
fn the_cpu_judge_names_the_abandoned_root_of_an_image_whose_abandoned_root_walks_into_an_overwritten_unit(
) {
    let (image, expected_root) = image_whose_abandoned_root_walks_into_an_overwritten_unit(
        "given-states-named-abandoned-root-on-the-cpu-judge",
    );
    let table = table_of_one_image("被抛弃根那一半判红的镜像", &image);
    let judged = judge_the_given_states_on_the_cpu(&table);
    let [state] = judged.states.as_slice() else {
        panic!("一张镜像一个状态，交回 {} 个", judged.states.len());
    };
    assert!(
        is_violated(
            state,
            singlefs_verification::crash_given_states::INVARIANT_WHOSE_DETAIL_NAMES_ABANDONED_ROOTS
        ),
        "被抛弃根那一半判 I-7.4 红（违例 {:?}）",
        state.pool_checker_violations()
    );
    assert_eq!(
        state.abandoned_roots_named_by_the_pool_checker,
        [expected_root],
        "CPU 判器从 I-7.4 的违例细节里读出的是被抛弃的 C 那一条根"
    );
}

/// [`the_pool_checker_still_judges_every_invariant_when_one_device_has_no_valid_system_configuration`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「C461：一块盘的系统配置全废时 checker 整片报不适用，而恢复挂得上」）：盘 0 两个系统配置槽都自证不过的那张上
/// 整片报不适用，场景写下的「比干净镜像至多少判一条、不许整片不适用」对不上，这条红。
#[test]
fn on_the_cpu_judge_the_pool_checker_still_judges_every_invariant_when_one_device_has_no_valid_system_configuration(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "the_pool_checker_still_judges_every_invariant_when_one_device_has_no_valid_system_configuration",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_system_configuration_per_device_redundancy::run_the_scenario_where_the_pool_checker_still_judges_every_invariant_when_one_device_has_no_valid_system_configuration,
    );
}

/// [`rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根」与 C511 那一行「候选集不再把被回退行判出局的根剔掉」）：
/// 崩溃恢复抛弃的 C 又进了遍历与并集，场景写下的「一条不红」对不上，这条红。
#[test]
fn on_the_cpu_judge_rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_rollback_by_a_forward_publish::run_the_scenario_where_rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write,
    );
}

/// [`raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「步 5：checker 的候选集不按 F 收（F 之下的根照走）」与「I-8.7 不按实例分组：跨实例的事务号也拿来比」）：
/// F 之下的根照走、或两个实例的事务号拿来互比，合法的镜像上判红，场景写下的「一条不红」对不上，这条红。
#[test]
fn on_the_cpu_judge_raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_reuse_after_raising_the_floor::run_the_scenario_where_raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it,
    );
}

/// [`the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「I-9.16 树表排序：checker 不判 ① 盘上次序」）：两条条目互换位置只剩 I-1.1 红、I-9.16 不红，这条红。
#[test]
fn on_the_cpu_judge_the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::run_the_scenario_where_the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places,
    );
}

/// [`the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「池级 checker 判 I-9.16 ② 的发号次序不认四棵新树（退回七棵）」）：墓碑与谱系两条条目换了种类而 I-9.16 不红，这条红。
#[test]
fn on_the_cpu_judge_the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::run_the_scenario_where_the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds,
    );
}

/// [`one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring`]
/// 那个场景，镜像直接交 CPU 判器判。判别力（`crates/mutations.tsv` 里「池级 checker 候选过滤改回一圈窗口」那一行）：窗口外、洞里留着的实例 1 的根
/// 不当候选走，它们引用的单元照算已分配，I-3.1 红，场景写下的「一条不红」对不上，这条红。
#[test]
fn on_the_cpu_judge_one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root::run_the_scenario_where_one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring,
    );
}

/// [`reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv` 里「池级 checker「由记录施加出来的那一版」第 ④ 条改回一圈窗口」那一行）：洞里那条根之后只被更新的根引用的单元
/// 不并进遍历，I-3.1 红，场景写下的「一条不红」对不上，这条红。
#[test]
fn on_the_cpu_judge_reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring::run_the_scenario_where_reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated,
    );
}

/// [`the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「三·2 池级 checker：墓碑记录宽登记成 41」）：墓碑容器照错的记录宽判，删除各批之后那几张判红，场景写下的「一条不红」对不上，这条红。
#[test]
fn on_the_cpu_judge_the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch() {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_unlink_single_file_publishes::run_the_scenario_where_the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch,
    );
}

/// [`formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「增补 2 收口第 44 行：I-3.10 不读树表 0 条那一版根记录直接持有的那片分配记录树」）：那一版被换下的单元
/// 没人罩着，场景写下的判决对不上，这条红。
#[test]
fn on_the_cpu_judge_a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_writable_mount_of_a_formatted_pool::run_the_scenario_where_a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold,
    );
}

/// [`contradictory_chain_record_reddens_only_the_instance_table_invariant`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「实例表第二片：checker 不判链指针记录合不合法」）：自相矛盾的链记录判成合法，场景写下的 I-3.8 红没红，这条红。
#[test]
fn on_the_cpu_judge_contradictory_chain_record_reddens_only_the_instance_table_invariant() {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "contradictory_chain_record_reddens_only_the_instance_table_invariant",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_instance_table_chain_read::run_the_scenario_where_contradictory_chain_record_reddens_only_the_instance_table_invariant,
    );
}

/// [`chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「实例表第二片：checker 不沿链走（链指针指到别的单元不红）」）：链指针指到别的单元而不走过去，身份那一条不红，这条红。
#[test]
fn on_the_cpu_judge_chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant() {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_instance_table_chain_read::run_the_scenario_where_chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant,
    );
}

/// [`the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「checker 的 F_生效 不算被抛弃根带的 F」）：F_生效 少算被抛弃根带的 F，场景写下的判决对不上，这条红。
#[test]
fn on_the_cpu_judge_the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries() {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor::run_the_scenario_where_the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries,
    );
}

/// [`the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「池级 checker 跟上（…）：重放那一版不走新根段里的实例表指针」）：由记录施加出来的那一版不走新根段里的实例表，
/// 场景写下的判决对不上，这条红。
#[test]
fn on_the_cpu_judge_the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::run_the_scenario_where_the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment,
    );
}

/// [`the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「池级 checker 跟上（…）：重放那一版不走新根段里的分配记录树根」）：由记录施加出来的那一版不走新根段里的
/// 分配记录树根，场景写下的判决对不上，这条红。
#[test]
fn on_the_cpu_judge_the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::run_the_scenario_where_the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment,
    );
}

/// [`raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring`]
/// 那个场景，镜像直接交 CPU 判器判。判别力（`crates/mutations.tsv` 里 defer 门槛改回一圈窗口那一行：只取锚在镜像最新根上的窗口里的有效根）：
/// 洞里 F 之下的根引用、释放代落在那一截里的槽不并，I-3.1 红，场景写下的「一条不红」对不上，这条红。
#[test]
fn on_the_cpu_judge_raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring::run_the_scenario_where_raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring,
    );
}

/// [`capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv` 里 I-9.21 那一行「非最右墓碑容器只要求不超 815」）：压小了的墓碑容器照产品镜像判也不红，场景写下的 I-9.21 红没红，这条红。
#[test]
fn on_the_cpu_judge_a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry,
    );
}

/// [`tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant`] 那个场景里只报墓碑容器旋钮值 1 的那一组
/// （`KnobsTheTombstoneContainerOfTwoRecordsIsJudgedUnder::OneTombstoneRecordOnly`：装了 2 条的那片容器多于旋钮值，只红 I-9.22），
/// 镜像带这一组旋钮值直接交 CPU 判器判。同名的引擎用例三组各判一趟；判别力只落在旋钮值小于装的条数的这一组上，所以只取这一组。
/// 判别力（`crates/mutations.tsv` 里 I-9.22 那一行「墓碑容器按产品容量 815 比、不按旋钮值」）：装 2 条的容器照 815 比不红，场景写下的 I-9.22 红没红，这条红。
#[test]
fn on_the_cpu_judge_a_tombstone_container_holding_more_records_than_a_tombstone_knob_of_one_record_reddens_only_the_knob_capacity_invariant(
) {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::KnobsTheTombstoneContainerOfTwoRecordsIsJudgedUnder;
    let judged_under = KnobsTheTombstoneContainerOfTwoRecordsIsJudgedUnder::OneTombstoneRecordOnly;
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
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

/// [`of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv` 里「checker 择最新根键（txg, 实例）相同时取读根环次序里后一条」那一行）：池级 checker 走了后一个槽里那条根，
/// 场景写下的判决对不上，这条红。
#[test]
fn on_the_cpu_judge_of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance::run_the_scenario_where_of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does,
    );
}

/// [`plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_the_reclaim_threshold`]
/// 那个场景，镜像直接交 CPU 判器判。判别力（`crates/mutations.tsv`「C560 认成已知红的判别力自证：池级 checker 的 I-7.18 求等放宽成不大于」）：
/// 恢复之前那一张上根记录的回收门槛 4 不大于现算的 5 被放过，场景写下的「恰好 I-7.18 红」对不上，这条红。
#[test]
fn on_the_cpu_judge_plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_the_reclaim_threshold(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_the_reclaim_threshold",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_rollback_by_a_forward_publish::run_the_scenario_where_plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_the_reclaim_threshold,
    );
}

/// [`root_left_one_level_above_the_only_leaf_with_an_entry_reddens_the_upper_shape_invariant`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「上段规范形 I-9.40：池级 checker 判形状那一句恒判成立」）：上段根留高一层那张必红镜像上 I-9.40 不再红，
/// 场景写下的「I-9.40 红、说明里写出根层级与多出的 (1, 0)」对不上，这条红。
#[test]
fn on_the_cpu_judge_a_root_left_one_level_above_the_only_leaf_with_an_entry_reddens_the_upper_shape_invariant(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "root_left_one_level_above_the_only_leaf_with_an_entry_reddens_the_upper_shape_invariant",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_unlink_of_files_other_than_the_first::run_the_scenario_where_a_root_left_one_level_above_the_only_leaf_with_an_entry_reddens_the_upper_shape_invariant,
    );
}

/// [`reclaim_threshold_written_as_the_floor_while_a_slot_stays_pinned_reddens_the_reclaim_threshold_invariant`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「C773：池级 checker 的 I-7.18 不判」）：根记录的回收门槛写成 F（0）而现算的是 6 也放过，
/// 场景写下的「I-7.18 红」对不上，这条红。
#[test]
fn on_the_cpu_judge_a_reclaim_threshold_written_as_the_floor_while_a_slot_stays_pinned_reddens_the_reclaim_threshold_invariant(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "reclaim_threshold_written_as_the_floor_while_a_slot_stays_pinned_reddens_the_reclaim_threshold_invariant",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot::run_the_scenario_where_a_reclaim_threshold_written_as_the_floor_while_a_slot_stays_pinned_reddens_the_reclaim_threshold_invariant,
    );
}

/// [`pinned_ring_slot_left_on_a_slot_this_instance_wrote_reddens_the_pinned_slot_invariant`] 那个场景，镜像直接交 CPU 判器判。
/// 判别力（`crates/mutations.tsv`「C773：池级 checker 的 I-7.19 不判」）：钉槽指着本实例自己写的 txg 20 也放过，
/// 场景写下的「I-7.19 红、I-7.18 成立」对不上，这条红。
#[test]
fn on_the_cpu_judge_a_pinned_ring_slot_left_on_a_slot_this_instance_wrote_reddens_the_pinned_slot_invariant(
) {
    judge_the_images_a_harness_scenario_hands_out_on_the_cpu_judge(
        "pinned_ring_slot_left_on_a_slot_this_instance_wrote_reddens_the_pinned_slot_invariant",
        ContainerCapacityKnobs::PRODUCT,
        scenarios_of_root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot::run_the_scenario_where_a_pinned_ring_slot_left_on_a_slot_this_instance_wrote_reddens_the_pinned_slot_invariant,
    );
}
