//! harness 测试文件 `crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    build_pool, crash_state_devices, memory_pool_of_sparse_devices, parameters,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
    FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish,
    Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_read::{mount_read_only, OpenFileFailure};
use singlefs_core::records::{
    TombstoneRecord, STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_INODE_WATERMARK,
    STATISTIC_PENDING_DELETE_BYTES,
};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_new_inodes, publish_unlink_intent, publish_unlink_step, PoolWriter, TransactionOutput,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

pub const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);

/// 进程内那几次发布都在新池新建文件那个实例里发。
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

pub fn publish_the_intent_in_process(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FIRST_FILE,
        previous
            .inode_record
            .expect("意图之前第一个文件那条记录在")
            .write_time_seconds,
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

/// 一份崩溃镜像按批大小 1 的删除开关可写挂载一次（挂载里续做到空），交回挂载与它写到的那两块内存盘。
pub fn mount_a_copy_of_the_image_with_batches_of_one(
    image: &MemoryPool,
    stream: &SharedStream,
) -> (
    Mounted,
    Vec<(
        DeviceIdentity,
        singlefs_harness::RecordingBlockDevice<singlefs_harness::memory_pool::SparseBlockDevice>,
    )>,
) {
    let mut devices = crash_state_devices(image, &[], &[], stream);
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .expect("这一份镜像可写挂载、续做");
    (mounted, devices)
}

/// 这一版那一统计量的现行值：代段最大的那一行（inode 号水位那一统计量还有台阶行，D8（核心索引结构） 已定项 6「读现行值」）。
pub fn accounting_value(version: &TransactionOutput, statistic: u16) -> u64 {
    version
        .accounting_entries
        .iter()
        .filter(|entry| entry.statistic == statistic)
        .max_by_key(|entry| entry.generation)
        .expect("池级三行每一版都写")
        .value
}

/// 第 7 条的「逻辑内容」：inode 每条（号、对象出生代、size、nlink）、数据单元个数、`deleted_inodes` key 集、墓碑记录多重集（含死亡 txg）、
/// 标签 12 / 4 / 6。不含实例号、根与单元头里的 txg、改动计数、落点（第一版只删第一个文件：数据单元的内容随个数一起由那个文件定）。
#[derive(Debug, PartialEq, Eq)]
pub struct LogicalContent {
    pub inode_records: Vec<(u64, CheckpointTxg, u64, u32)>,
    pub data_units: usize,
    pub deleted_inodes: Vec<InodeNumber>,
    pub tombstones: Vec<TombstoneRecord>,
    pub inode_number_watermark: u64,
    pub pending_delete: u64,
    pub committed_reservation: u64,
}

pub fn logical_content_of(version: &TransactionOutput) -> LogicalContent {
    let mut inode_records: Vec<(u64, CheckpointTxg, u64, u32)> = version
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .map(|record| (record.inode, record.object_birth, record.size, record.nlink))
        .collect();
    inode_records.sort_unstable();
    let mut tombstones: Vec<TombstoneRecord> = version
        .tombstone_containers
        .iter()
        .flat_map(|container| container.contents.records.iter().copied())
        .collect();
    tombstones.sort_unstable_by_key(|record| record.to_bytes());
    LogicalContent {
        inode_records,
        data_units: version.data_pointers.len(),
        deleted_inodes: version.deleted_inodes.clone(),
        tombstones,
        inode_number_watermark: accounting_value(version, STATISTIC_INODE_WATERMARK),
        pending_delete: accounting_value(version, STATISTIC_PENDING_DELETE_BYTES),
        committed_reservation: accounting_value(version, STATISTIC_COMMITTED_RESERVATION_BYTES),
    }
}

/// ③（一个写点）：第一批的写发出一半时崩，重开可写挂载续做到底；终态逻辑内容与不崩溃的终态相同，崩溃态与终态池级 checker 全绿，
/// 两份终态的读者都答「没有文件」。逐写点的全量枚举是 checker 档的活。
pub fn a_crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to_the_uninterrupted_end_state(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_with_a_three_unit_file("idempotence-crash-in-the-first-batch");
    publish_the_intent_in_process(&mut pool);
    let operations_before_the_first_batch = pool.retained_operations().len();
    publish_the_next_step_in_process(&mut pool);
    let operations_after_the_first_batch = pool.retained_operations().len();
    publish_the_next_step_in_process(&mut pool);
    let uninterrupted_end = publish_the_next_step_in_process(&mut pool);
    let operations = pool.retained_operations();
    let crash_point = operations_before_the_first_batch
        + (operations_after_the_first_batch - operations_before_the_first_batch) / 2;
    let mut crash_image =
        MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    crash_image.apply(&operations[..crash_point]);
    images.hand_out(
        "崩溃态",
        || crash_image.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    let stream = SharedStream::new();
    let (mounted, devices) = mount_a_copy_of_the_image_with_batches_of_one(&crash_image, &stream);
    let continued_end = mounted
        .current
        .version_of_trees()
        .expect("续做到底之后有树")
        .clone();
    assert_eq!(
        logical_content_of(&continued_end),
        logical_content_of(&uninterrupted_end),
        "终态逻辑内容与不崩溃的终态相同"
    );
    let continued_image = memory_pool_of_sparse_devices(&devices);
    images.hand_out(
        "续做之后",
        || continued_image.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    let read_only = mount_read_only(&continued_image).expect("只读挂载");
    assert!(
        matches!(
            read_only.mounted.open_file(&continued_image, FIRST_FILE),
            Err(OpenFileFailure::NoSuchInode { inode }) if inode == FIRST_FILE
        ),
        "续做之后读者答「没有文件」"
    );
}

/// 回退到删空的一版（Q-K）：删完唯一的文件之后再建一个 size 0 的 inode，回退目标取删空那一版——它写过文件，在候选集里；
/// 回退之后的一版是有树而文件集合为空的那一种，池级 checker 全绿。
pub fn rolling_back_to_the_emptied_version_is_accepted_and_publishes_a_version_with_trees_and_no_file(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_with_a_three_unit_file("rollback-to-the-emptied-version");
    publish_the_intent_in_process(&mut pool);
    publish_the_next_step_in_process(&mut pool);
    publish_the_next_step_in_process(&mut pool);
    let emptied = publish_the_next_step_in_process(&mut pool);
    assert!(
        emptied.is_a_version_with_trees_and_no_file(),
        "最后一批之后是删空的一版"
    );
    {
        let publish_parameters = parameters();
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous,
            1,
            FIXED_WRITE_TIME_SECONDS + 60,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("删空的一版上建一个 inode");
    }
    assert!(
        !pool.output.is_a_version_with_trees_and_no_file(),
        "建了 inode 之后那一版带文件"
    );
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: emptied.root.instance,
            checkpoint_txg: emptied.root.checkpoint_txg,
        },
    )
    .expect("删空的一版写过文件，是回退候选");
    assert!(
        pool.output.is_a_version_with_trees_and_no_file(),
        "回退之后的一版有树而文件集合为空"
    );
    images.hand_out(
        "回退之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
