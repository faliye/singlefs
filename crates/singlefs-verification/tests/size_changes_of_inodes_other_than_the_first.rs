//! 第一个文件之外的 inode 缩短与变长（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2、S3、S4、S6、S7；
//! 射程「任意活 inode 都能缩短与变长」）的验收：H1–H5 那五段历史在实现之后都是合法镜像、四个读者都收，
//! 7.1 节行为层的几条（K9 叶边界、批间停下再重挂续做、快照下缩短、带洞时被 unlink、两个 inode 的回退），与「有记录、没有条目、
//! size > 0」读者拒那一形。场景与期望写在 `tests/common_images_for_the_given_states_engine/size_changes_of_inodes_other_than_the_first.rs`。
//! 扫描重建（冷走读同一段核）对删除中（nlink = 0）的记录同样判「有记录、没有条目、size > 0」那一条只在这里：手造的镜像不交给给定状态引擎。

//! 这里只核实现行为；对应场景函数（`run_the_scenario_where_<用例名>`）交出的镜像由给定状态引擎两段判。

mod common;
mod common_deletion_in_progress;
mod common_images_for_the_given_states_engine;
mod common_pointer_head_corruption;
mod common_truncation;
#[path = "common_images_for_the_given_states_engine/size_changes_of_inodes_other_than_the_first.rs"]
mod scenarios;

use singlefs_core::records::{InodeRecord, NLINK_OF_AN_INODE_BEING_DELETED};
use singlefs_core::recovery::{
    rebuild_version, RebuildVersionFailure, RecoveryFailure,
    RECORD_WITHOUT_AN_EXTENT_ENTRY_HAS_A_SIZE,
};

#[test]
fn extending_an_inode_without_an_entry_writes_a_sticky_tag_zero_entry_read_as_zeros_by_every_reader(
) {
    common_images_for_the_given_states_engine::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(scenarios::run_the_scenario_where_extending_an_inode_without_an_entry_writes_a_sticky_tag_zero_entry_read_as_zeros_by_every_reader);
}

#[test]
fn extending_an_inode_inside_its_one_data_unit_rewrites_that_unit_and_every_reader_reads_the_zeros()
{
    common_images_for_the_given_states_engine::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(scenarios::run_the_scenario_where_extending_an_inode_inside_its_one_data_unit_rewrites_that_unit_and_every_reader_reads_the_zeros);
}

#[test]
fn extending_an_inode_across_data_units_declares_the_hole_and_every_reader_reads_zeros_up_to_the_size(
) {
    common_images_for_the_given_states_engine::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(scenarios::run_the_scenario_where_extending_an_inode_across_data_units_declares_the_hole_and_every_reader_reads_zeros_up_to_the_size);
}

#[test]
fn truncating_an_inode_inside_its_one_data_unit_rewrites_it_through_an_intent_and_a_final_batch() {
    common_images_for_the_given_states_engine::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(scenarios::run_the_scenario_where_truncating_an_inode_inside_its_one_data_unit_rewrites_it_through_an_intent_and_a_final_batch);
}

#[test]
fn truncating_a_three_unit_inode_into_unit_zero_writes_its_truncation_field_and_releases_the_units_beyond(
) {
    common_images_for_the_given_states_engine::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(scenarios::run_the_scenario_where_truncating_a_three_unit_inode_into_unit_zero_writes_its_truncation_field_and_releases_the_units_beyond);
}

#[test]
fn truncating_another_inode_onto_the_lower_leaf_boundary_rewrites_the_leaf_holding_the_cut_unit() {
    common_images_for_the_given_states_engine::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(scenarios::run_the_scenario_where_truncating_another_inode_onto_the_lower_leaf_boundary_rewrites_the_leaf_holding_the_cut_unit);
}

#[test]
fn a_truncation_of_another_inode_stopped_between_batches_continues_by_its_number_at_the_writable_remount(
) {
    common_images_for_the_given_states_engine::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(scenarios::run_the_scenario_where_a_truncation_of_another_inode_stopped_between_batches_continues_by_its_number_at_the_writable_remount);
}

#[test]
fn truncating_another_inode_under_a_live_snapshot_keeps_its_replaced_data_units_on_the_deadlist() {
    common_images_for_the_given_states_engine::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(scenarios::run_the_scenario_where_truncating_another_inode_under_a_live_snapshot_keeps_its_replaced_data_units_on_the_deadlist);
}

#[test]
fn unlinking_another_inode_with_a_hole_switches_its_tag_back_and_deletes_it() {
    common_images_for_the_given_states_engine::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(scenarios::run_the_scenario_where_unlinking_another_inode_with_a_hole_switches_its_tag_back_and_deletes_it);
}

#[test]
fn a_rollback_between_two_truncations_of_different_inodes_keeps_one_entry_naming_the_target_inode()
{
    common_images_for_the_given_states_engine::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(scenarios::run_the_scenario_where_a_rollback_between_two_truncations_of_different_inodes_keeps_one_entry_naming_the_target_inode);
}

#[test]
fn a_record_with_a_size_but_no_upper_leaf_entry_is_refused_by_the_readers_and_reddens_only_the_unit_count(
) {
    common_images_for_the_given_states_engine::PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(scenarios::run_the_scenario_where_a_record_with_a_size_but_no_upper_leaf_entry_is_refused_by_the_readers_and_reddens_only_the_unit_count);
}

/// 扫描重建判「有记录、没有条目、size > 0」不给删除中的记录开例外（判决 m4-close-code-r1 T1 缺口三；这一判在 `recovery.rs` 的
/// `data_units_judged_against_the_inode_records`，冷走读同一段；合法历史里删除中的记录不会有这一形，豁免与不豁免分不出，要手造）：
/// 新池新建文件之后建一个 inode（size 0、上段没有条目），发它的 unlink 意图那一次（nlink 0、进 `deleted_inodes`），
/// 再把最新那一版里它的记录 size 改成 5000（不经写者，重封叶、校验和补到根）。按改过之后那条最新的根重建，拒成 `RECORD_WITHOUT_AN_EXTENT_ENTRY_HAS_A_SIZE`。
#[test]
fn scan_rebuild_refuses_a_record_being_deleted_with_a_size_but_no_upper_leaf_entry() {
    let mut pool = common::build_pool("scan-rebuild-record-being-deleted-without-an-entry");
    let inode = pool.output.inode_number_watermark();
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let intent = common_deletion_in_progress::publish_the_intent_of(&mut pool, inode);
    assert_eq!(
        intent.deleted_inodes,
        vec![singlefs_core::address::InodeNumber(inode)],
        "意图那一次之后 inode {inode} 在删除队列里"
    );
    let mut image = pool.memory_pool();
    common_deletion_in_progress::change_the_inode_record(&mut image, inode, |bytes| {
        let mut record = InodeRecord::parse(bytes).expect("记录解得开");
        assert_eq!(
            (record.nlink, record.size),
            (NLINK_OF_AN_INODE_BEING_DELETED, 0),
            "改之前：删除中、size 0"
        );
        record.size = 5000;
        bytes.copy_from_slice(&record.to_bytes());
    });
    // 改了记录、校验和补到了根槽：按改过之后那条最新的根重建（记录照写出那一版的那一条顶着）。
    let rebuilt = rebuild_version(
        &image,
        &common_pointer_head_corruption::newest_root(&image),
        Some(intent.record.clone()),
    );
    assert!(
        matches!(
            &rebuilt,
            Err(RebuildVersionFailure::Walk(RecoveryFailure::InvariantViolated { detail, .. }))
                if *detail == RECORD_WITHOUT_AN_EXTENT_ENTRY_HAS_A_SIZE
        ),
        "扫描重建拒成「有记录、没有条目、size > 0」：{rebuilt:?}"
    );
}
