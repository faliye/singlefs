//! 验证包模块：crash_point_replay_progress、crash_given_states、crash_library、crash_amplification、verdict_store、gpu_unit_checks
//! harness 集成测试真跑实现搭出来的镜像，池级 checker 那一遍搬到这里经给定状态引擎两段判（
//! harness 集成测试真跑实现之后在 CPU 上对镜像跑池级 checker 的那些点，池级 checker 那一遍搬进给定状态引擎、经给定状态表两段判、
//! 配了 GPU 走 GPU；harness 用例只留实现的行为与理想模型、RefFS 的比对）。
//!
//! 每条用例与 harness 里同名的那条是同一个场景：经 `#[path]` 引 `tests/common_images_for_the_given_states_engine/<测试文件名>.rs`
//! 里对应的场景函数 `run_the_scenario_where_<用例名>`（连同它用到的 `tests/common*/mod.rs`），拿收镜像的那一种 `PoolCheckerImages` 调它——场景照样真跑实现、照样核行为，
//! 就地跑池级 checker 的那几处交出那一刻的镜像与池级 checker 该判成什么。每张镜像怎么建表、交引擎判、逐张核场景写下的那几样，
//! 判法只一份，在共用模块 `tests/common_given_states_of_harness_scenario_images/mod.rs`。场景按 harness 测试文件分组，组按文件名排。
#![allow(
    clippy::duplicate_mod,
    reason = "harness 的 common_admission 与 common_unwitnessed_publish 各经 #[path] 装一份 common_admission_pool；两批场景各用其一，类型互不相交"
)]

#[path = "../common/mod.rs"]
mod common;
#[path = "../common_admission/mod.rs"]
mod common_admission;
#[path = "../common_deletion_in_progress/mod.rs"]
mod common_deletion_in_progress;
#[path = "../common_given_states_of_harness_scenario_images/mod.rs"]
mod common_given_states_of_harness_scenario_images;
#[cfg(feature = "gpu")]
#[path = "../common_gpu_cards_of_the_caller/mod.rs"]
mod common_gpu_cards_of_the_caller;
#[path = "../common_images_for_the_given_states_engine/mod.rs"]
mod common_images_for_the_given_states_engine;
#[path = "../common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;
#[path = "../common_root_ring_holes/mod.rs"]
mod common_root_ring_holes;
#[path = "../common_shared_by_harness_scenarios/mod.rs"]
mod common_shared_by_harness_scenarios;
#[path = "../common_tree_split/mod.rs"]
mod common_tree_split;
#[path = "../common_unwitnessed_publish/mod.rs"]
mod common_unwitnessed_publish;
#[path = "../common_images_for_the_given_states_engine/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs"]
mod scenarios_of_a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable;
#[path = "../common_images_for_the_given_states_engine/accounting_and_mapping_tree_split.rs"]
mod scenarios_of_accounting_and_mapping_tree_split;
#[path = "../common_images_for_the_given_states_engine/admission_formula_on_the_mounted_state.rs"]
mod scenarios_of_admission_formula_on_the_mounted_state;
#[path = "../common_images_for_the_given_states_engine/admission_raises_the_floor_before_refusing.rs"]
mod scenarios_of_admission_raises_the_floor_before_refusing;
#[path = "../common_images_for_the_given_states_engine/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs"]
mod scenarios_of_admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments;
#[path = "../common_images_for_the_given_states_engine/an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor.rs"]
mod scenarios_of_an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor;
#[path = "../common_images_for_the_given_states_engine/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs"]
mod scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping;
#[path = "../common_images_for_the_given_states_engine/container_trees_grow_a_level_when_the_root_is_full.rs"]
mod scenarios_of_container_trees_grow_a_level_when_the_root_is_full;
#[path = "../common_images_for_the_given_states_engine/entries_after_mount_refuse_swapped_or_behind_devices.rs"]
mod scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices;
#[path = "../common_images_for_the_given_states_engine/fault_injection_campaign.rs"]
mod scenarios_of_fault_injection_campaign;
#[path = "../common_images_for_the_given_states_engine/fsync_drop_and_devices_without_the_selected_version.rs"]
mod scenarios_of_fsync_drop_and_devices_without_the_selected_version;
#[path = "../common_images_for_the_given_states_engine/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs"]
mod scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path;
#[path = "../common_images_for_the_given_states_engine/inode_number_watermark_pressed_back_reissues_a_number.rs"]
mod scenarios_of_inode_number_watermark_pressed_back_reissues_a_number;
#[path = "../common_images_for_the_given_states_engine/instance_table_chain_read.rs"]
mod scenarios_of_instance_table_chain_read;
#[path = "../common_images_for_the_given_states_engine/instance_table_page_full.rs"]
mod scenarios_of_instance_table_page_full;
#[path = "../common_images_for_the_given_states_engine/instance_table_second_page_write.rs"]
mod scenarios_of_instance_table_second_page_write;
#[path = "../common_images_for_the_given_states_engine/instance_table_sweep_at_the_end_of_a_writable_mount.rs"]
mod scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount;
#[path = "../common_images_for_the_given_states_engine/journal_last_record_flag_of_a_multi_record_publish.rs"]
mod scenarios_of_journal_last_record_flag_of_a_multi_record_publish;
#[path = "../common_images_for_the_given_states_engine/many_inodes_write_and_read.rs"]
mod scenarios_of_many_inodes_write_and_read;
#[path = "../common_images_for_the_given_states_engine/multi_record_publishes_of_transaction_zero.rs"]
mod scenarios_of_multi_record_publishes_of_transaction_zero;
#[path = "../common_images_for_the_given_states_engine/multi_unit_file_write_and_read.rs"]
mod scenarios_of_multi_unit_file_write_and_read;
#[path = "../common_images_for_the_given_states_engine/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs"]
mod scenarios_of_named_unit_verification_stands_in_for_the_barrier_before_the_record;
#[path = "../common_images_for_the_given_states_engine/one_revolution_window_after_a_hole_in_the_root_ring.rs"]
mod scenarios_of_one_revolution_window_after_a_hole_in_the_root_ring;
#[path = "../common_images_for_the_given_states_engine/overwrite_in_one_instance.rs"]
mod scenarios_of_overwrite_in_one_instance;
#[path = "../common_images_for_the_given_states_engine/publish_failure_resent_unchanged.rs"]
mod scenarios_of_publish_failure_resent_unchanged;
#[path = "../common_images_for_the_given_states_engine/quarantine_mark_agrees_with_the_references.rs"]
mod scenarios_of_quarantine_mark_agrees_with_the_references;
#[path = "../common_images_for_the_given_states_engine/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs"]
mod scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table;
#[path = "../common_images_for_the_given_states_engine/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs"]
mod scenarios_of_raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window;
#[path = "../common_images_for_the_given_states_engine/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs"]
mod scenarios_of_raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic;
#[path = "../common_images_for_the_given_states_engine/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs"]
mod scenarios_of_raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount;
#[path = "../common_images_for_the_given_states_engine/random_histories.rs"]
mod scenarios_of_random_histories;
#[path = "../common_images_for_the_given_states_engine/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs"]
mod scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read;
#[path = "../common_images_for_the_given_states_engine/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs"]
mod scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root;
#[path = "../common_images_for_the_given_states_engine/reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root.rs"]
mod scenarios_of_reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root;
#[path = "../common_images_for_the_given_states_engine/release_checksum_quarantine.rs"]
mod scenarios_of_release_checksum_quarantine;
#[path = "../common_images_for_the_given_states_engine/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs"]
mod scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment;
#[path = "../common_images_for_the_given_states_engine/reuse_after_raising_the_floor.rs"]
mod scenarios_of_reuse_after_raising_the_floor;
#[path = "../common_images_for_the_given_states_engine/reused_allocation_record_overlap.rs"]
mod scenarios_of_reused_allocation_record_overlap;
#[path = "../common_images_for_the_given_states_engine/rollback_by_a_forward_publish.rs"]
mod scenarios_of_rollback_by_a_forward_publish;
#[path = "../common_images_for_the_given_states_engine/rollback_lineage_record_and_its_checkpoint_cost_term.rs"]
mod scenarios_of_rollback_lineage_record_and_its_checkpoint_cost_term;
#[path = "../common_images_for_the_given_states_engine/rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate.rs"]
mod scenarios_of_rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate;
#[path = "../common_images_for_the_given_states_engine/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs"]
mod scenarios_of_root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance;
#[path = "../common_images_for_the_given_states_engine/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs"]
mod scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread;
#[path = "../common_images_for_the_given_states_engine/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs"]
mod scenarios_of_root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window;
#[path = "../common_images_for_the_given_states_engine/root_ring_turn_in_one_mount.rs"]
mod scenarios_of_root_ring_turn_in_one_mount;
#[path = "../common_images_for_the_given_states_engine/root_ring_turn_in_the_make_filesystem_session.rs"]
mod scenarios_of_root_ring_turn_in_the_make_filesystem_session;
#[path = "../common_images_for_the_given_states_engine/row_publish_admission.rs"]
mod scenarios_of_row_publish_admission;
#[path = "../common_images_for_the_given_states_engine/row_publish_checks_before_acquisition.rs"]
mod scenarios_of_row_publish_checks_before_acquisition;
#[path = "../common_images_for_the_given_states_engine/row_publish_record_without_its_root.rs"]
mod scenarios_of_row_publish_record_without_its_root;
#[path = "../common_images_for_the_given_states_engine/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs"]
mod scenarios_of_shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring;
#[path = "../common_images_for_the_given_states_engine/system_configuration_fields_core_and_checker_refuse_the_same_table.rs"]
mod scenarios_of_system_configuration_fields_core_and_checker_refuse_the_same_table;
#[path = "../common_images_for_the_given_states_engine/system_configuration_per_device_redundancy.rs"]
mod scenarios_of_system_configuration_per_device_redundancy;
#[path = "../common_images_for_the_given_states_engine/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs"]
mod scenarios_of_system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery;
#[path = "../common_images_for_the_given_states_engine/test_only_switches_force_every_branch.rs"]
mod scenarios_of_test_only_switches_force_every_branch;
#[path = "../common_images_for_the_given_states_engine/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs"]
mod scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker;
#[path = "../common_images_for_the_given_states_engine/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs"]
mod scenarios_of_unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over;
#[path = "../common_images_for_the_given_states_engine/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs"]
mod scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version;
#[path = "../common_images_for_the_given_states_engine/unlink_mount_continuation_and_the_emptied_version.rs"]
mod scenarios_of_unlink_mount_continuation_and_the_emptied_version;
#[path = "../common_images_for_the_given_states_engine/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs"]
mod scenarios_of_unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches;
#[path = "../common_images_for_the_given_states_engine/unlink_single_file_publishes.rs"]
mod scenarios_of_unlink_single_file_publishes;
#[path = "../common_images_for_the_given_states_engine/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs"]
mod scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero;
#[path = "../common_images_for_the_given_states_engine/writable_mount_of_a_formatted_pool.rs"]
mod scenarios_of_writable_mount_of_a_formatted_pool;
#[path = "../common_images_for_the_given_states_engine/writable_mount_takes_a_second_instance.rs"]
mod scenarios_of_writable_mount_takes_a_second_instance;

use std::collections::BTreeSet;

use common_given_states_of_harness_scenario_images::{
    is_judged, is_violated, judge_one_image, judge_the_collected_images,
    judge_the_images_a_harness_scenario_hands_out,
    judge_the_images_a_harness_scenario_hands_out_under_the_container_capacity_knobs,
    judge_the_images_a_harness_scenario_hands_out_when_it_hands_out_none, mismatches_of_the_images,
    start_the_engine, start_the_engine_on_the_library_of_the_test,
    start_the_engine_on_the_library_of_the_test_as_a_share, the_run_counts, ConclusionsOfAnImage,
    JudgedAfreshRegardlessOfTheExpectations, LibraryOfTheTest, LibraryOfTheTestAtTheStart,
    WhenTheScenarioHandsOutNoImage,
};
use common_images_for_the_given_states_engine::{
    PoolCheckerExpectation, PoolCheckerImage, PoolCheckerImages,
};
use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
use singlefs_checker::walk::{check_pool_image_naming_the_violation_sites, ContainerCapacityKnobs};
use singlefs_verification::crash_given_states::{
    judge_the_given_states_on_the_cpu, ClaimedVersionsOfTheStates, GivenStatesOfATable,
    GivenStatesOfATableWithItsColumns,
};
use singlefs_verification::fault_injection::{
    classify_the_pool_checker_verdicts_of_the_images_after_a_fault, FailureAfterAFault,
    ImagesAfterAFault, KnownRedFormAtAFault, KnownRedFormOnlyAfterAFault,
    PoolCheckerVerdictsOfTheImagesAfterAFault, ViolationsNamingTheSites,
};
use singlefs_verification::history::FailureSignature;
#[cfg(feature = "gpu")]
use {
    common_given_states_of_harness_scenario_images::ITEM_NAME,
    singlefs_verification::crash_amplification::{
        GivenStatesEngine, GivenStatesItem, GivenStatesRun,
    },
    singlefs_verification::crash_library::DisposableLibrary,
    singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity,
};

mod admission_scenarios;
mod fault_injection_points_classified_by_the_harness;
mod judges_and_libraries_of_the_given_states_engine;
mod mount_instance_table_and_system_configuration_scenarios;
mod publish_journal_record_and_rollback_scenarios;
mod quarantine_and_reuse_scenarios;
mod root_ring_and_reclaim_floor_scenarios;
mod scenario_images_judged_directly_on_the_cpu_judge;
mod tree_and_file_scenarios;
mod unlink_and_deletion_scenarios;

/// 被抛弃根那一半判 I-7.4 红的一张镜像与它点名的那一条根：建池（第一个文件 A）→ 覆盖写 B、C → 崩溃恢复抛弃 C（实例 1、txg 5；
/// `scenarios_of_random_histories::pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version`，挂载时影子账隔离 C 的单元，这张镜像池级 checker 一条不红），
/// 再把 C 写的第一个数据单元的第一份里头之后的一段字节改掉：现行时间线（实例 2）上的根都不引用它，只有被抛弃根那一半走 C 时读到它校验和对不上，
/// I-7.4 的第一处违例就是这一条，违例细节点名 C。
fn image_whose_abandoned_root_walks_into_an_overwritten_unit(
    tag: &str,
) -> (
    singlefs_verification::memory_pool::MemoryPool,
    singlefs_verification::crash_given_states::AbandonedRootNamedByThePoolChecker,
) {
    use singlefs_core::address::DeviceOffsetInBytes;
    use singlefs_verification::crash_given_states::AbandonedRootNamedByThePoolChecker;
    /// 改掉的那一段在数据单元里的偏移与长度：头之后、载荷里的一段（改哪一段不要紧，只要那一份的位置项校验和对不上）。
    /// 盘按整扇区读写：读写一个扇区，只翻其中头 OVERWRITTEN_BYTES 个字节。
    const OVERWRITTEN_BYTES_OFFSET_IN_THE_UNIT: u64 = 4096;
    const OVERWRITTEN_BYTES: usize = 64;
    let sector_bytes = usize::try_from(singlefs_verification::memory_pool::SECTOR_BYTES)
        .expect("扇区字节数装进 usize");
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
    image: &'image singlefs_verification::memory_pool::MemoryPool,
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

// 原 CPU 就地判调用点迁入同一套给定状态两段流程。

#[path = "../common_images_for_the_given_states_engine/acquisition_rewrites_a_device_that_lost_both_system_configuration_slots_only_through_the_gates.rs"]
mod scenarios_of_acquisition_rewrites_a_device_that_lost_both_system_configuration_slots_only_through_the_gates;

#[path = "../common_images_for_the_given_states_engine/publish_lookup_by_identity.rs"]
mod scenarios_of_publish_lookup_by_identity;

#[path = "../common_images_for_the_given_states_engine/snapshot_and_compaction_over_a_file_other_than_the_first.rs"]
mod scenarios_of_snapshot_and_compaction_over_a_file_other_than_the_first;

#[path = "../common_images_for_the_given_states_engine/unlink_batches_of_an_inode_other_than_the_first_follow_its_own_data_units.rs"]
mod scenarios_of_unlink_batches_of_an_inode_other_than_the_first_follow_its_own_data_units;

#[path = "../common_images_for_the_given_states_engine/unlink_of_files_other_than_the_first.rs"]
mod scenarios_of_unlink_of_files_other_than_the_first;

#[path = "../common_images_for_the_given_states_engine/writes_by_inode_number_after_rollbacks_past_the_deletion_of_the_first_file.rs"]
mod scenarios_of_writes_by_inode_number_after_rollbacks_past_the_deletion_of_the_first_file;

#[path = "../common_images_for_the_given_states_engine/writes_by_inode_number_in_the_interleaved_histories_of_the_attack.rs"]
mod scenarios_of_writes_by_inode_number_in_the_interleaved_histories_of_the_attack;

#[path = "../common_images_for_the_given_states_engine/writes_by_inode_number_of_several_files.rs"]
mod scenarios_of_writes_by_inode_number_of_several_files;

#[path = "../common_images_for_the_given_states_engine/root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot.rs"]
mod scenarios_of_root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot;
