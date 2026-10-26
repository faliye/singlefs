//! 删除与整理交错、回退跨过整理意图那条崩溃流的搭建（公共模块 `tests/common_compaction_deletion_prefix/mod.rs`，欠账 C621）在 harness 档能单独核的几样：
//! 录出来的流形状对（两条意图同时活着的那几版、回退之后整理意图还在、最后一批之后 R 按记录全空），观察者另读的逐版期望与每一版落完那一刻的盘上读出的相同，
//! 两张「崩在这一版之后」的盘（unlink 意图之后、回退之后）上可写挂载一次，删除续做走空、整理续做推到意图删了、续做的新落点不进 R。
//! 崩溃状态的枚举在 checker 档的 `crash_enumeration_deletion_interleaved_with_compaction_across_a_rollback_stream.rs`。

mod common;
mod common_compaction_deletion_prefix;
mod common_compaction_prefix;
mod common_snapshot_prefix;

use common_compaction_deletion_prefix::{
    assert_the_mount_continued_the_deletion_then_the_compaction,
    assert_the_version_reads_both_files, content_of_the_other_file,
    content_of_the_other_file_read_by_a_read_only_mount, mount_writable_on_the_crash_disks,
    prepare_the_deletion_compaction_crash_flow, version_of_the_deletion_compaction_flow_landed_on,
    DeletionCompactionFlowStepKind, LandedVersionOfTheDeletionCompactionFlow,
    PreparedDeletionCompactionCrashFlow, PublishOfTheDeletionCompactionFlow,
    INODE_OF_THE_OTHER_FILE,
};
use common_compaction_prefix::{
    image_with_the_relocation_sources_zeroed, region_of_the_crash_flows,
    COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS,
};
use singlefs_core::compaction_plan::region_is_empty_by_the_records;
use singlefs_core::logged_operations::live_compaction_intent_of_the_entries;
use singlefs_harness::memory_pool::closed_form_state_count;

/// 段形（录流按真写表数出来的）：前十二段到 A 同新池新建文件流（取号两轮、暖机、A 24 个单元写，各自记录、根、轮换）；
/// 实例 1 里建 inode 20、顺序写 28 个单元写（两个数据单元、四条记录）；重挂：取号两轮系统配置各 2、写行 18、暖机 16；
/// 签发 18、一批 20；unlink 意图 30、一批 26、最后一批 26；回退 24；回退之后一批 24、最后一批 24 个单元写，各自记录 2、根 1、轮换 2。
const SEGMENT_SIZES: [usize; 62] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2, 28, 4, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2,
    18, 2, 1, 2, 20, 2, 1, 2, 30, 2, 1, 2, 26, 2, 1, 2, 26, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 24,
    2, 1, 2,
];

/// 每次写只取两态时整条流的闭式（`memory_pool::closed_form_state_count`）。
const CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 1_546_190_965;

/// 快档展开的段（写数小于 10）每次写只取两态时的闭式。
const QUICK_TIER_CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 130;

fn the_step_of(
    prepared: &PreparedDeletionCompactionCrashFlow,
    kind: DeletionCompactionFlowStepKind,
) -> &common_compaction_deletion_prefix::DeletionCompactionFlowStep {
    let steps = prepared.recorded.steps_of(kind);
    assert_eq!(steps.len(), 1, "{kind:?} 这一步在流里恰好一次");
    steps[0]
}

/// 录出来的流：步的次序；unlink 意图与不是最后一批的那几批两条意图同时活着，最后一批删除意图删了、整理意图还在；回退落到签发之前那一版、
/// 整理意图跨回退留着、那个文件回来了、删除队列空；最后一批同一次发布删意图、R 按记录全空；段形与闭式照钉的数。
#[test]
fn the_recorded_flow_interleaves_the_unlink_with_the_compaction_and_the_rollback_keeps_the_compaction_intent(
) {
    let prepared = prepare_the_deletion_compaction_crash_flow("compaction-deletion-flow-shape");
    let recorded = &prepared.recorded;
    let kinds: Vec<DeletionCompactionFlowStepKind> =
        recorded.steps.iter().map(|step| step.kind).collect();
    println!(
        "COMPACTION_DELETION_FLOW steps={kinds:?} segments={:?} closed_form={} quick_closed_form={} relocated_units={:?}",
        prepared.segment_sizes(),
        closed_form_state_count(&prepared.segments),
        closed_form_state_count(&prepared.segments_shorter_than(10)),
        recorded
            .steps
            .iter()
            .map(|step| step
                .versions
                .iter()
                .map(|(_publish, output)| output.units_relocated_by_this_publish.len())
                .collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        kinds[..7],
        [
            DeletionCompactionFlowStepKind::NewInodeInProcess,
            DeletionCompactionFlowStepKind::SequentialWriteOfTheOtherFileInProcess,
            DeletionCompactionFlowStepKind::MountWritable,
            DeletionCompactionFlowStepKind::CompactionIssue,
            DeletionCompactionFlowStepKind::CompactionBatch,
            DeletionCompactionFlowStepKind::UnlinkOfTheOtherFileThroughTheSession,
            DeletionCompactionFlowStepKind::RollbackToTheVersionBeforeTheIssue,
        ],
        "建 inode → 顺序写 → 重挂 → 签发 → 一批 → unlink → 回退"
    );
    assert_eq!(
        kinds.last(),
        Some(&DeletionCompactionFlowStepKind::CompactionFinalBatch),
        "回退之后整理推到最后一批"
    );
    let unlink = the_step_of(
        &prepared,
        DeletionCompactionFlowStepKind::UnlinkOfTheOtherFileThroughTheSession,
    );
    let (final_unlink_publish, final_unlink_version) =
        unlink.versions.last().expect("unlink 至少发意图与最后一批");
    assert_eq!(
        *final_unlink_publish,
        PublishOfTheDeletionCompactionFlow::UnlinkFinalBatch
    );
    assert!(
        unlink.versions.len() >= 3,
        "两个数据单元、删除批大小 1：意图、一批、最后一批"
    );
    for (publish, output) in &unlink.versions[..unlink.versions.len() - 1] {
        assert!(
            output.deleted_inodes.contains(&INODE_OF_THE_OTHER_FILE)
                && live_compaction_intent_of_the_entries(&output.logged_operations).is_some(),
            "{publish:?}：删除意图与整理意图同时活着"
        );
    }
    assert!(
        final_unlink_version.deleted_inodes.is_empty()
            && live_compaction_intent_of_the_entries(&final_unlink_version.logged_operations)
                .is_some(),
        "unlink 的最后一批：删除意图删了，整理意图还在"
    );
    let issue = the_step_of(&prepared, DeletionCompactionFlowStepKind::CompactionIssue);
    let issued_intent =
        live_compaction_intent_of_the_entries(&issue.versions[0].1.logged_operations)
            .expect("签发那一版有活意图");
    assert_eq!(
        issued_intent.1.region,
        region_of_the_crash_flows(),
        "签的是强制选的那一段"
    );
    assert!(
        recorded.rollback_target.checkpoint_txg < issue.versions[0].1.root.checkpoint_txg,
        "回退目标在签发之前"
    );
    let rollback = the_step_of(
        &prepared,
        DeletionCompactionFlowStepKind::RollbackToTheVersionBeforeTheIssue,
    );
    let rolled_back = &rollback.versions[0].1;
    assert_eq!(
        live_compaction_intent_of_the_entries(&rolled_back.logged_operations),
        Some(issued_intent),
        "回退那一版：整理意图随现行、跨回退留着（判决 m4-trunc-r1 K1）"
    );
    assert!(
        rolled_back.deleted_inodes.is_empty()
            && rolled_back
                .files_other_than_the_first
                .iter()
                .any(|file| file.inode == INODE_OF_THE_OTHER_FILE),
        "回退那一版：删除队列取签发之前那一版的（空），那个文件取它的（回来了）"
    );
    let batch_before_the_rollback = &recorded
        .steps
        .iter()
        .take_while(|step| {
            step.kind != DeletionCompactionFlowStepKind::RollbackToTheVersionBeforeTheIssue
        })
        .find(|step| step.kind == DeletionCompactionFlowStepKind::CompactionBatch)
        .expect("回退之前有整理的一批")
        .versions[0]
        .1;
    assert_eq!(
        batch_before_the_rollback
            .units_relocated_by_this_publish
            .len(),
        COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS,
        "回退之前那一批搬一整批"
    );
    let version_of_the_rollback = prepared
        .versions_of_the_flow
        .iter()
        .find(|version| version.publish == PublishOfTheDeletionCompactionFlow::Rollback)
        .expect("回退那一版在流里");
    for relocated in &batch_before_the_rollback.units_relocated_by_this_publish {
        for location in &relocated.source {
            assert!(
                region_of_the_crash_flows().contains_slot(location.slot)
                    && !version_of_the_rollback
                        .relocation_sources_unused_by_this_version
                        .contains(&(location.device, location.slot)),
                "回退把回退之前那一批搬走的单元复活在 R 里的源落点上（目标那一版引用的是源落点）：{location:?}"
            );
        }
    }
    let relocated_after_the_rollback: usize = recorded
        .steps
        .iter()
        .skip_while(|step| {
            step.kind != DeletionCompactionFlowStepKind::RollbackToTheVersionBeforeTheIssue
        })
        .flat_map(|step| step.versions.iter())
        .map(|(_publish, output)| output.units_relocated_by_this_publish.len())
        .sum();
    assert!(
        relocated_after_the_rollback > batch_before_the_rollback.units_relocated_by_this_publish.len(),
        "回退之后整理重扫 R，复活的那几个单元连同原来剩下的一起搬：搬了 {relocated_after_the_rollback} 个"
    );
    let (_publish, last_version) = recorded
        .steps
        .last()
        .and_then(|step| step.versions.last())
        .expect("最后一批那一版");
    assert!(
        last_version.logged_operations.is_empty()
            && region_is_empty_by_the_records(
                &last_version.allocation_records,
                region_of_the_crash_flows()
            ),
        "最后一批同一次发布删意图、R 按记录全空"
    );
    assert_eq!(prepared.discards_in_the_stream(), 0, "这条流上没有 discard");
    assert_eq!(prepared.segment_sizes(), SEGMENT_SIZES, "段形照钉的数");
    assert_eq!(
        closed_form_state_count(&prepared.segments),
        CLOSED_FORM_WITH_TWO_STATES_PER_WRITE,
        "每次写只取两态时的闭式：1 + Σ(2^|段| − 1)"
    );
    assert_eq!(
        closed_form_state_count(&prepared.segments_shorter_than(10)),
        QUICK_TIER_CLOSED_FORM_WITH_TWO_STATES_PER_WRITE,
        "快档展开的段的闭式"
    );
}

/// 观察者另读的逐版期望与盘上对得上：A 之后每一版落完的那一刻（根槽写与系统配置轮换都落了盘）的盘，只读挂载落到这一版、
/// 那个文件读出这一版期望里写的（还没建、删除中的答没有文件；回退之后回来了）；这一版没人用的源落点整槽抹成 0 之后，沿这一版的根
/// 两个文件照样读出期望里写的（读的那个观察者在每个崩溃状态上做的同一件事，这里只在每一版落完那一刻的盘上做）。
#[test]
fn every_version_of_the_flow_reads_the_other_file_as_its_expectation_right_after_its_publish() {
    let prepared = prepare_the_deletion_compaction_crash_flow("compaction-deletion-flow-reads");
    let mut versions_read = 0usize;
    for step in &prepared.recorded.steps {
        for (version_index, (publish, output)) in step.versions.iter().enumerate() {
            let image = prepared.image_right_after_the_version_of_the_step(step, version_index);
            let LandedVersionOfTheDeletionCompactionFlow::Version(expected) =
                version_of_the_deletion_compaction_flow_landed_on(
                    &prepared.versions_of_the_flow,
                    (output.root.instance, output.root.checkpoint_txg),
                )
            else {
                panic!("{:?}：每一版都在流里", step.kind);
            };
            let what = format!("{:?}（{publish:?}）那一版落完", step.kind);
            let (landed_root, content) =
                content_of_the_other_file_read_by_a_read_only_mount(&image, &what);
            assert_eq!(landed_root, expected.version, "{what}：只读挂载落到这一版");
            assert!(
                content == expected.content_of_the_other_file_read_back,
                "{what}：那个文件读出期望里写的（{:?} 字节），实际 {:?} 字节",
                expected
                    .content_of_the_other_file_read_back
                    .as_ref()
                    .map(Vec::len),
                content.as_ref().map(Vec::len)
            );
            let zeroed = image_with_the_relocation_sources_zeroed(
                &image,
                &expected.relocation_sources_unused_by_this_version,
            );
            assert_the_version_reads_both_files(
                &zeroed,
                expected,
                "这一版没人用的源落点抹成 0 之后",
            );
            versions_read += 1;
        }
    }
    assert!(
        prepared
            .versions_of_the_flow
            .iter()
            .any(|version| version.content_of_the_other_file_read_back
                == Some(content_of_the_other_file())),
        "流里有读出那个文件内容的一版"
    );
    assert_eq!(
        versions_read + 1,
        prepared.versions_of_the_flow.len(),
        "A 之后的每一版都核过"
    );
}

/// 崩在 unlink 意图那次发布之后（删除意图与整理意图同时活着）：可写挂载先把删除续做走空、再把整理意图推到删了，续做的新落点不进 R，
/// 挂完 logged_ops 与删除队列都空、那个文件删完。
#[test]
fn a_writable_mount_right_after_the_unlink_intent_walks_the_deletion_then_continues_the_compaction()
{
    let prepared =
        prepare_the_deletion_compaction_crash_flow("compaction-deletion-mount-after-unlink-intent");
    let unlink = the_step_of(
        &prepared,
        DeletionCompactionFlowStepKind::UnlinkOfTheOtherFileThroughTheSession,
    );
    let image = prepared.image_right_after_the_version_of_the_step(unlink, 0);
    let output = &unlink.versions[0].1;
    let landed = version_of_the_deletion_compaction_flow_landed_on(
        &prepared.versions_of_the_flow,
        (output.root.instance, output.root.checkpoint_txg),
    );
    let mounted = mount_writable_on_the_crash_disks(&image);
    let compaction_publishes =
        assert_the_mount_continued_the_deletion_then_the_compaction(landed, &mounted);
    assert!(compaction_publishes > 0, "整理意图活着：续做真发过");
}

/// 崩在回退那次发布之后（整理意图跨回退留着、回退把那个文件连同它在 R 里的单元带回来）：可写挂载把整理意图推到删了，续做的新落点不进 R，
/// 挂完 logged_ops 空、那个文件读出它的内容。
#[test]
fn a_writable_mount_right_after_the_rollback_continues_the_compaction_intent_carried_across_it() {
    let prepared =
        prepare_the_deletion_compaction_crash_flow("compaction-deletion-mount-after-rollback");
    let rollback = the_step_of(
        &prepared,
        DeletionCompactionFlowStepKind::RollbackToTheVersionBeforeTheIssue,
    );
    let image = prepared.image_right_after_the_version_of_the_step(rollback, 0);
    let output = &rollback.versions[0].1;
    let landed = version_of_the_deletion_compaction_flow_landed_on(
        &prepared.versions_of_the_flow,
        (output.root.instance, output.root.checkpoint_txg),
    );
    let mounted = mount_writable_on_the_crash_disks(&image);
    let compaction_publishes =
        assert_the_mount_continued_the_deletion_then_the_compaction(landed, &mounted);
    assert!(compaction_publishes > 0, "整理意图活着：续做真发过");
}
