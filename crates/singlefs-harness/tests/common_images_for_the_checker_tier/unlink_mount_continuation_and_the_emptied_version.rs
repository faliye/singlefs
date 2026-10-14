//! harness 测试文件 `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    build_pool, parameters, sequential_write_in_process_as_instance_one_and_advance_the_pool,
    BuiltPool,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{InodeNumber, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish,
    Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_read::{mount_read_only, OpenFileFailure};
use singlefs_core::records::{InodeRecord, TREE_KIND_DELETED_INODES};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, PoolVersion, PoolWriter, TransactionOutput,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};

pub const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);

/// 进程内那几次删除发布都在新池新建文件那个实例里发。
pub const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);

pub fn batches_of_one_data_unit() -> UnlinkBatchSize {
    UnlinkBatchSize::CappedForTests(
        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
    )
}

pub fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: batches_of_one_data_unit(),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container_capacity:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 新池新建文件之后把第一个文件顺序写成三个数据单元（最后一个单元装半个净荷）。
pub fn pool_with_a_three_unit_file(tag: &str) -> BuiltPool {
    let payload_capacity = data_unit_payload_capacity();
    let content: Vec<u8> = (0..2 * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 13 + 5) % 251).expect("小于 256"))
        .collect();
    let mut pool = build_pool(tag);
    let grown =
        sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    assert_eq!(grown.data_pointers.len(), 3, "第一个文件三个数据单元");
    pool
}

/// 第一个文件那条记录现有的写入时间（池的当前版是进程内最后发的那一版）：unlink 的意图照传它，盘上字节与 Q-F 乙之前相同。
pub fn write_time_seconds_of_the_first_file(pool: &BuiltPool) -> u64 {
    pool.output
        .inode_record
        .expect("这一刻第一个文件那条记录在")
        .write_time_seconds
}

/// 在这个进程里发 unlink 的意图，带第一个文件那条记录现有的写入时间。
pub fn publish_the_intent_in_process(pool: &mut BuiltPool) -> TransactionOutput {
    let write_time_seconds = write_time_seconds_of_the_first_file(pool);
    publish_the_intent_in_process_at(pool, write_time_seconds)
}

/// 在这个进程里发 unlink 的意图，带调用方给的写入时间（用户 2026-10-03 定 Q-F 乙：意图带时间、三格同改）。
pub fn publish_the_intent_in_process_at(
    pool: &mut BuiltPool,
    write_time_seconds: u64,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FIRST_FILE,
        write_time_seconds,
        INSTANCE_OF_THE_PROCESS,
    )
    .expect("意图");
    pool.output = output.clone();
    output
}

pub fn publish_the_next_step_in_process(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_unlink_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FIRST_FILE,
        batches_of_one_data_unit(),
        INSTANCE_OF_THE_PROCESS,
    )
    .expect("删除的下一步");
    pool.output = output.clone();
    output
}

/// 重开镜像、按批大小 1 的删除开关可写挂载一次（空间准入照判、影子账开）。
pub fn remount_with_batches_of_one(
    pool: &mut BuiltPool,
) -> (
    Mounted,
    Vec<(
        singlefs_core::address::DeviceIdentity,
        crate::common::Recorded,
    )>,
) {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .expect("重开可写挂载");
    (mounted, devices)
}

/// 这次挂载自己发的最后一次（写行或暖机）那条根的 txg：续做的发布排在它后面。
pub fn txg_of_the_last_row_or_warm_up_publish(mounted: &Mounted) -> u64 {
    mounted
        .output
        .warm_up_publishes
        .last()
        .unwrap_or(&mounted.output.row_publish)
        .root()
        .checkpoint_txg
        .0
}

pub fn member_name(version: &PoolVersion) -> &'static str {
    match version {
        PoolVersion::WithoutFile(_) => "WithoutFile",
        PoolVersion::WithFile(_) => "WithFile",
        PoolVersion::WithTreesAndNoFile(_) => "WithTreesAndNoFile",
    }
}

pub fn a_writable_mount_after_the_intent_continues_the_queue_to_empty_after_its_warm_up(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_with_a_three_unit_file("mount-continues-after-the-intent");
    publish_the_intent_in_process(&mut pool);
    let (mounted, devices) = remount_with_batches_of_one(&mut pool);
    let continuation_publishes =
        mounted.current.root().checkpoint_txg.0 - txg_of_the_last_row_or_warm_up_publish(&mounted);
    assert_eq!(
        (
            member_name(&mounted.current),
            continuation_publishes,
            mounted
                .current
                .version_of_trees()
                .expect("有树")
                .deleted_inodes
                .clone(),
        ),
        ("WithTreesAndNoFile", 3, Vec::new()),
        "写行与暖机之后续做：留 2、留 1 两批 + 最后一批，交回时队列空、是删空的一版"
    );
    drop(devices);
    let image = pool.memory_pool();
    images.hand_out(
        "续做之后",
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    let read_only = mount_read_only(&image).expect("只读挂载");
    assert!(
        matches!(
            read_only.mounted.open_file(&image, FIRST_FILE),
            Err(OpenFileFailure::NoSuchInode { inode }) if inode == FIRST_FILE
        ),
        "续做之后只读挂载打开第一个文件答「没有文件」"
    );
}

/// 第一个文件那条 inode 记录在这一版里的样子。
pub fn record_of_the_first_file(version: &TransactionOutput) -> InodeRecord {
    version
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .find(|record| record.inode == FIRST_FILE.0)
        .copied()
        .expect("意图与中间各批之后第一个文件的记录还在（nlink = 0）")
}

/// 意图那次与中间一批都算一次改动（用户 2026-10-03 定 Q-F「算改动，改实现」；D8（核心索引结构） 已定项 6 偏移 88 改动计数 =
/// 最后一次改动所在发布的 checkpoint_txg）：三个单元的文件发意图、再发一批（留 2），两次之后第一个文件那条记录的改动计数各等于那一次发布的 txg；
/// 写入时间另由 [`the_intent_writes_the_given_time_into_the_three_time_fields_and_a_middle_batch_copies_it`] 判（偏移 64 起 atime、mtime、ctime
/// 三个秒字段共写一个 `write_time_seconds`，`records::InodeRecord::to_bytes`；D8（核心索引结构） 已定项 6 字段表）。
/// 两次之后池级 checker 全绿（I-9.26（inode 改动计数落在出生代与容器诞生代号之间））。
/// 判别力：意图那次或那一批照旧带上一版的改动计数时那一项红。
pub fn the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_with_a_three_unit_file("change-count-of-the-deletion-steps");
    let before_the_unlink = record_of_the_first_file(&pool.output);
    let intent = publish_the_intent_in_process(&mut pool);
    let middle_batch = publish_the_next_step_in_process(&mut pool);
    let after_the_intent = record_of_the_first_file(&intent);
    let after_the_middle_batch = record_of_the_first_file(&middle_batch);
    assert!(
        before_the_unlink.change_count < intent.root.checkpoint_txg.0,
        "改动计数本来低于意图那次的 txg：{} 对 {}",
        before_the_unlink.change_count,
        intent.root.checkpoint_txg.0
    );
    assert_eq!(
        (
            after_the_intent.change_count,
            after_the_middle_batch.change_count,
        ),
        (
            intent.root.checkpoint_txg.0,
            middle_batch.root.checkpoint_txg.0,
        ),
        "意图那次与中间那一批各把改动计数设成自己那次发布的 txg"
    );
    images.hand_out(
        "两次之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

pub fn rolling_back_to_the_intent_takes_its_deleted_inodes_root_and_the_next_mount_continues_it_to_empty(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_with_a_three_unit_file("rollback-to-the-intent-then-continue");
    let intent = publish_the_intent_in_process(&mut pool);
    for _ in 0..3 {
        publish_the_next_step_in_process(&mut pool);
    }
    let deleted_inodes_root_of = |version: &TransactionOutput| {
        version
            .tree_table_entries
            .iter()
            .find(|entry| entry.kind == TREE_KIND_DELETED_INODES)
            .expect("树表里 deleted_inodes 一条")
            .root
    };
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: intent.root.instance,
            checkpoint_txg: intent.root.checkpoint_txg,
        },
    )
    .expect("回退到意图那一版");
    assert_eq!(
        (
            pool.output.deleted_inodes.clone(),
            deleted_inodes_root_of(&pool.output),
        ),
        (vec![FIRST_FILE], deleted_inodes_root_of(&intent)),
        "回退新根的 deleted_inodes 取 R_old 的：队列带回那个号、根指针是意图那一版的（D23 已定项 14）"
    );
    images.hand_out(
        "回退之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    let (mounted, remounted_devices) = remount_with_batches_of_one(&mut pool);
    assert_eq!(
        member_name(&mounted.current),
        "WithTreesAndNoFile",
        "下一次可写挂载把带回来的队列续做到空"
    );
    drop(remounted_devices);
    images.hand_out(
        "续做之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
