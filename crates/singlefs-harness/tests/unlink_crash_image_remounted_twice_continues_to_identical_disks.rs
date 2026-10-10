//! unlink 的崩溃镜像各自重挂载续做，两份盘逐字节相同（D8（核心索引结构） 已定项 16 第 7 条 ①b）：删除意图刚发完那一刻的镜像、
//! 第一批写到中间（单元与记录落了、根槽没落）的崩溃镜像，各自拷两份、各挂一次可写挂载续做到队列空，两份盘逐字节相同。
//! 两批之间那一份干净镜像在 `unlink_mount_continuation_and_the_emptied_version.rs` 的
//! `two_copies_of_the_image_between_two_batches_continue_to_byte_identical_disks`（第 34 批 U 交回：①b 只钉了那一形，判决
//! `research/prompts/m3-batch33r-code-r1-main-verification.md` 第三节第 4 条，这两形补在这里）。三个单元的第一个文件、只供测试的批大小 1。

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{crash_state_devices, geometry, memory_pool_of_sparse_devices, parameters};
use common_deletion_in_progress::{
    batches_of_one_data_unit, pool_with_a_three_unit_file, publish_the_intent,
    publish_the_next_step_in_batches_of_one,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, ShadowLedger,
};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::PoolVersion;
use singlefs_core::unlink::DeletionSwitches;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: batches_of_one_data_unit(),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container_capacity:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 一份崩溃镜像（`base` 叠上 `writes` 里 `persisted` 的那几条）在自己的两块内存盘上可写挂载一次、按批大小 1 续做到队列空，交回挂完之后的整份盘。
///
/// # Panics
/// 挂不上、或挂完那一版不是删空的一版（续做没走到队列空）。
fn remount_a_copy_of_the_crash_image_and_continue_to_the_empty_queue(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: &[bool],
) -> MemoryPool {
    let stream = SharedStream::new();
    let mut devices = crash_state_devices(base, writes, persisted, &stream);
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .expect("这一份崩溃镜像可写挂载、续做");
    assert!(
        matches!(mounted.current, PoolVersion::WithTreesAndNoFile(_)),
        "续做走到队列空：挂完那一版是删空的一版"
    );
    memory_pool_of_sparse_devices(&devices)
}

/// 删除意图刚发完（一批都没删）那一刻的镜像：两份各自挂一次、续做三批（留 2、留 1、最后一批），两份盘逐字节相同。
#[test]
fn a_crash_image_right_after_the_unlink_intent_remounted_twice_continues_to_byte_identical_disks() {
    let mut pool = pool_with_a_three_unit_file("unlink-intent-image-remounted-twice");
    publish_the_intent(&mut pool);
    let image_right_after_the_intent = pool.memory_pool();
    let first_copy = remount_a_copy_of_the_crash_image_and_continue_to_the_empty_queue(
        &image_right_after_the_intent,
        &[],
        &[],
    );
    let second_copy = remount_a_copy_of_the_crash_image_and_continue_to_the_empty_queue(
        &image_right_after_the_intent,
        &[],
        &[],
    );
    assert!(
        first_copy == second_copy,
        "意图刚发完的镜像各自重挂载续做，两份盘逐字节相同（D8 已定项 16 第 7 条 ①b）"
    );
    assert!(first_copy != image_right_after_the_intent, "续做真的写了盘");
}

/// 第一批写到中间的崩溃镜像：意图之后那一批的单元写、journal 记录都落了，根槽与系统配置轮换没落（发布按单元、记录、根槽、系统配置的次序落盘，
/// `transaction::persist_publish_writes`）。两份各自挂一次：恢复由那一批的记录施加出它（D23（journal 的角色与格式） 已定项 15），再续做剩下的两批，
/// 两份盘逐字节相同。
#[test]
fn a_crash_image_with_the_first_batch_written_up_to_its_root_remounted_twice_continues_to_byte_identical_disks(
) {
    let mut pool = pool_with_a_three_unit_file("unlink-first-batch-crash-image-remounted-twice");
    publish_the_intent(&mut pool);
    let image_after_the_intent = pool.memory_pool();
    let operations_before_the_first_batch = pool.retained_operations().len();
    publish_the_next_step_in_batches_of_one(&mut pool);
    let operations = pool.retained_operations();
    let (writes_of_the_first_batch, _segments, _stream_indexes) =
        writes_and_segments_with_stream_indexes(
            &operations[operations_before_the_first_batch..],
            &geometry(),
        );
    let root_write = writes_of_the_first_batch
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("第一批有一条根槽写");
    assert!(
        writes_of_the_first_batch[..root_write]
            .iter()
            .any(|write| write.kind == StepKind::JournalRecord)
            && writes_of_the_first_batch[..root_write]
                .iter()
                .any(|write| write.kind == StepKind::UnitWrite),
        "根槽写之前那一批的单元与记录都写了"
    );
    let persisted_up_to_the_root: Vec<bool> = (0..writes_of_the_first_batch.len())
        .map(|write_index| write_index < root_write)
        .collect();
    let first_copy = remount_a_copy_of_the_crash_image_and_continue_to_the_empty_queue(
        &image_after_the_intent,
        &writes_of_the_first_batch,
        &persisted_up_to_the_root,
    );
    let second_copy = remount_a_copy_of_the_crash_image_and_continue_to_the_empty_queue(
        &image_after_the_intent,
        &writes_of_the_first_batch,
        &persisted_up_to_the_root,
    );
    assert!(
        first_copy == second_copy,
        "第一批写到中间的崩溃镜像各自重挂载续做，两份盘逐字节相同（D8 已定项 16 第 7 条 ①b）"
    );
    assert!(first_copy != image_after_the_intent, "续做真的写了盘");
}
