//! 攻方原型（m3-batch58-code-r1 opus）共用的流：照抄被判冻结副本里
//! `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 那几条流的造法（名字不带 layer0），
//! 盘换成内存稀疏盘（`SparseBlockDevice`，外面包录制器；不建镜像文件），搭建照 `crates/singlefs-harness/tests/common/mod.rs` 的 `build_pool`
//! （同一组 mkfs 参数、取号、暖机、新池新建文件、分配器的两步装载）与 `common_deletion_in_progress` 的建号、回退。
#![allow(dead_code, reason = "几份原型各用其中一部分")]

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::container_tree::{ContainerTreeCapacities, ContainerTreeNodeEntries};
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, MakeFilesystemParameters,
    INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::tombstone_tree::{TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_unlink_intent,
    publish_unlink_steps_until_the_queue_is_empty, warm_up, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

pub const IMAGE_BYTES: u64 = 4 << 30;
pub const FILE_BYTES: usize = 3000;
pub const E142_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
pub const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
pub const WRITE_TIME_SECONDS_OF_THE_NEW_INODES: u64 = 1_788_000_100;
pub const INSTANCE_OF_THE_PUBLISHES: InstanceGeneration = InstanceGeneration(1);
pub const FIRST_FILE: u64 = 1;
pub const INODE_RECORDS_PER_LEAF: u64 = 233;
pub const KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER: [u64; 4] = [2, 2, 1, 1];

pub fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E142_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

pub fn geometry() -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        root_ring_slots_per_region: parameters().geometry.root_ring_slots_per_region,
    }
}

pub fn file_content() -> Vec<u8> {
    (0..FILE_BYTES).map(|index| u8::try_from(index % 251).expect("小于 256")).collect()
}

pub type Recorded = RecordingBlockDevice<SparseBlockDevice>;

/// `build_pool` 的内存盘版：录制流开内容保留。
pub struct SparsePool {
    pub devices: Vec<(DeviceIdentity, Recorded)>,
    pub stream: SharedStream,
    pub output: TransactionOutput,
    pub allocator: PoolAllocator,
    pub mkfs_operation_count: usize,
}

impl SparsePool {
    pub fn build() -> Self {
        let parameters = parameters();
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<(DeviceIdentity, Recorded)> = (0..2u32)
            .map(|number| {
                let identity = DeviceIdentity(number);
                let device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
                (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()))
            })
            .collect();
        let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mkfs_operation_count = stream.operations().len();
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
            DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
        ]);
        allocator.mark_format_time_units(
            Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
            Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
        );
        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(&parameters, &genesis));
        let content = file_content();
        let output = {
            let mut pool = PoolWriter::new(&parameters, &mut devices);
            let instance = acquire_instance(&mut pool).expect("取号");
            assert_eq!(instance, InstanceGeneration(1));
            let warmed = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
            publish_first_file(
                &mut pool,
                &mut allocator,
                warmed.roots.last().expect("暖机两代根"),
                FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("新池新建文件")
        };
        Self { devices, stream, output, allocator, mkfs_operation_count }
    }

    pub fn retained_operations(&self) -> Vec<RetainedOperation> {
        self.stream.retained_operations()
    }

    pub fn memory_pool_of(operations: &[RetainedOperation]) -> MemoryPool {
        let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
        pool.apply(operations);
        pool
    }

    pub fn memory_pool(&self) -> MemoryPool {
        Self::memory_pool_of(&self.retained_operations())
    }

    pub fn memory_pool_after_mkfs(&self) -> MemoryPool {
        Self::memory_pool_of(&self.retained_operations()[..self.mkfs_operation_count])
    }

    pub fn publish_new_inodes(&mut self, count: u64) {
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        self.output = publish_new_inodes(
            &mut writer,
            &mut self.allocator,
            &self.output,
            count,
            WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
            INSTANCE_OF_THE_PUBLISHES,
        )
        .expect("建空 inode");
    }

    pub fn with_new_inodes(count: u64) -> Self {
        let mut pool = Self::build();
        pool.publish_new_inodes(count);
        pool
    }

    pub fn unlink_to_the_end_under(&mut self, inode: u64, capacities: ContainerTreeCapacities) {
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        writer.set_container_tree_capacities(capacities);
        let mut current = publish_unlink_intent(
            &mut writer,
            &mut self.allocator,
            &self.output,
            InodeNumber(inode),
            WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
            INSTANCE_OF_THE_PUBLISHES,
        )
        .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
        publish_unlink_steps_until_the_queue_is_empty(
            &mut writer,
            &mut self.allocator,
            &mut current,
            UnlinkBatchSize::OneExtentLowerLeaf,
            INSTANCE_OF_THE_PUBLISHES,
        )
        .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的各批：{refusal:?}"));
        self.output = current;
    }

    pub fn roll_back_to(&mut self, target: &TransactionOutput) {
        let parameters = parameters();
        roll_back_by_a_forward_publish(
            &parameters,
            &mut self.devices,
            &mut self.allocator,
            &mut self.output,
            RollbackTarget { instance: target.root.instance, checkpoint_txg: target.root.checkpoint_txg },
        )
        .unwrap_or_else(|refusal| panic!("回退：{refusal:?}"));
    }

    pub fn versions_with_the_first_file_through_the_last_publish(&self) -> Vec<PublishedVersion> {
        (3..=self.output.root.checkpoint_txg.0)
            .map(|txg| PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(txg),
                content: file_content(),
            })
            .collect()
    }
}

pub struct RecordedFlow {
    pub name: &'static str,
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub judged_root_index: usize,
    pub versions: Vec<PublishedVersion>,
}

/// 被判那一份的 `recorded_flow_of_the_pool`：从 mkfs 之后录下的写切段，被判的是最后一次根槽写。
pub fn recorded_flow_of_the_pool(pool: &SparsePool, name: &'static str) -> RecordedFlow {
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations[pool.mkfs_operation_count..], &geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布有根槽写");
    RecordedFlow {
        name,
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        judged_root_index,
        versions: pool.versions_with_the_first_file_through_the_last_publish(),
    }
}

pub fn capacities_of_the_tombstone_container_knob(records_per_container: u64) -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(records_per_container).expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container: singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

pub fn capacities_of_the_retirement_container_knob(records_per_container: u64) -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container: singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
            singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(records_per_container)
                .expect("1..=959"),
        ),
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 被判那一份的 `pool_rolled_back_under_these_lineage_knobs`。
pub fn pool_rolled_back_under_these_lineage_knobs(knobs: &[u64]) -> SparsePool {
    let mut pool = SparsePool::build();
    let first_file_version = pool.output.clone();
    pool.publish_new_inodes(1);
    for lineage_records_per_container in knobs {
        pool.allocator.set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
            batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
            tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
            lineage_container_capacity: LineageContainerCapacity::CappedForTests(
                singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(*lineage_records_per_container)
                    .expect("1..=1359"),
            ),
            retirement_container_capacity:
                singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
            container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
        });
        pool.roll_back_to(&first_file_version);
    }
    pool
}

/// 被判那一份的三条 J3 GPU 流（[2, 1, 1]），`tree` 是 tombstone / lineage / retirement。
pub fn record_containers_of_two_one_and_one_records_under_changing_knobs(tree: &str) -> RecordedFlow {
    let knobs = KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER;
    let (pool, counts, name): (SparsePool, Vec<usize>, &'static str) = match tree {
        "tombstone" => {
            let mut pool = SparsePool::with_new_inodes(u64::try_from(knobs.len()).expect("u64"));
            for (inode, records_per_container) in (FIRST_FILE + 1..).zip(knobs) {
                pool.unlink_to_the_end_under(inode, capacities_of_the_tombstone_container_knob(records_per_container));
            }
            let counts = pool.output.tombstone_containers.iter().map(|c| c.contents.records.len()).collect();
            (pool, counts, "tombstone-2-1-1")
        }
        "lineage" => {
            let pool = pool_rolled_back_under_these_lineage_knobs(&knobs);
            let counts = pool.output.lineage_containers.iter().map(|c| c.contents.records.len()).collect();
            (pool, counts, "lineage-2-1-1")
        }
        "retirement" => {
            let mut pool = SparsePool::with_new_inodes(INODE_RECORDS_PER_LEAF);
            for (retirement_index, records_per_container) in knobs.into_iter().enumerate() {
                if retirement_index > 0 {
                    pool.publish_new_inodes(1);
                }
                let inode = pool.output.inode_number_watermark() - 1;
                pool.unlink_to_the_end_under(inode, capacities_of_the_retirement_container_knob(records_per_container));
            }
            let counts = pool.output.retirement_containers.iter().map(|c| c.contents.records.len()).collect();
            (pool, counts, "retirement-2-1-1")
        }
        other => panic!("不认识的树 {other}"),
    };
    assert_eq!(counts, [2, 1, 1], "{tree} 按叶序每片的记录数");
    recorded_flow_of_the_pool(&pool, name)
}
