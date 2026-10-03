//! 根环上有洞的池的搭建（S = 4 的 12 槽小环、1 MiB journal 环、两块 128 MiB 内存稀疏盘）：攻方腿原型
//! `research/prompts/m3-root-ring-code-r1-opus-model/rr-attack-proto/src/lib.rs` 里 `build_plain`、`crash_a_mount_before_its_row_root`、
//! `publish_empty` 那几样照同一个做法另写一份（D13（验证路线） 已定项 5：用例不与原型共用代码）。
#![allow(dead_code, reason = "每个测试文件各自只用到其中一部分")]

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::recovery::{choose_system_configuration, readable_roots_with_ring_slots};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, publish_version, FirstFile, InstanceTablePlan,
    PoolWriter, PublishError, PublishPlan, TransactionOutput,
};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 这几份用例池的 fsid。
pub const FILESYSTEM_IDENTIFIER_OF_THE_RING_HOLE_POOLS: [u8; 16] = *b"RING-HOLES-POOL1";
/// 每块盘 128 MiB（攻方 `rf-*`、`rfo3-*` 那几组的 `PROTO_IMAGE_BYTES`）。
pub const DEVICE_BYTES_OF_THE_RING_HOLE_POOLS: u64 = 128 << 20;
/// journal 环 1 MiB（同那几组的 `PROTO_JOURNAL_BYTES`）。
pub const JOURNAL_RING_BYTES_OF_THE_RING_HOLE_POOLS: u64 = 1 << 20;
/// 每区槽数 S = 4：环长 12。
pub const SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS: u64 = 4;
/// 写入时间的起点。
pub const WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS: u64 = 1_788_000_000;
/// 一次覆盖写的内容长度：一个数据单元装得下。
const CONTENT_BYTES: usize = 3000;

pub type RingHoleDevices = Vec<(DeviceIdentity, SparseBlockDevice)>;
pub type RecordedRingHoleDevices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

#[must_use]
pub fn parameters_of_the_ring_hole_pools() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER_OF_THE_RING_HOLE_POOLS,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_BYTES_OF_THE_RING_HOLE_POOLS,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS,
            )
            .expect("S = 4 在区间里"),
        },
    }
}

#[must_use]
pub fn fixed_geometry_of_the_ring_hole_pools() -> FixedGeometry {
    let parameters = parameters_of_the_ring_hole_pools();
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

/// 第 `salt` 版的文件内容（3000 字节，各版不同）。
#[must_use]
pub fn content_of_the_version(salt: u64) -> Vec<u8> {
    let salt = usize::try_from(salt % 251).expect("小于 251");
    (0..CONTENT_BYTES)
        .map(|index| u8::try_from((index + salt * 7 + 3) % 251).expect("小于 256"))
        .collect()
}

#[must_use]
pub fn pool_of(devices: &RingHoleDevices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
    }
}

#[must_use]
pub fn devices_of(pool: &MemoryPool) -> RingHoleDevices {
    pool.devices
        .iter()
        .map(|(identity, image)| {
            let mut device = SparseBlockDevice::new(
                DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
                PhysicalBlockSizeInBytes(512),
            );
            device.image = image.clone();
            (*identity, device)
        })
        .collect()
}

/// 这份盘面上的两块盘，外面包录制器、录进 `stream`。
#[must_use]
pub fn recording_of(pool: &MemoryPool, stream: &SharedStream) -> RecordedRingHoleDevices {
    devices_of(pool)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect()
}

/// 带文件的一版上的空发布：文件不变，只换根与记账（准入、抬 F 那几串推的空发布同一个计划）。
///
/// # Errors
/// 发布路径的错原样交回。
pub fn publish_empty<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    publish_version(
        pool,
        allocator,
        PublishPlan {
            txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
            counter: previous.record.counter + 1,
            transaction: 0,
            highest_transaction_number_before_this_publish: previous
                .highest_transaction_number_in_this_instance,
            instance,
            back_chain: singlefs_core::journal::back_chain_of(&previous.record_bytes),
            file: None,
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
    )
}

/// mkfs → 可写挂载（实例 1）→ 第一个文件 → 覆盖写到 txg `last`，每次都完整落盘。
#[must_use]
pub fn pool_overwritten_through(last: u64) -> MemoryPool {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices: RingHoleDevices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(
                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
                    PhysicalBlockSizeInBytes(512),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(&parameters, &mut devices).expect("第一次可写挂载");
    let instance = output.instance;
    let mut current: TransactionOutput = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content_of_the_version(0),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
            },
            instance,
            current.record_bytes(),
        )
        .expect("第一个文件")
    };
    // 迭代上界：txg 逐次加一，到 `last` 为止。
    while current.root.checkpoint_txg.0 < last {
        let salt = current.root.checkpoint_txg.0 + 1;
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile {
                content: &content_of_the_version(salt),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
            },
            instance,
        )
        .expect("覆盖写");
    }
    pool_of(&devices)
}

/// 在 `image` 上做一次可写挂载，崩在写行那次根槽写之前（写行那次的单元与记录已持久、根槽没持久，之后的写都没落）。
/// 交回崩溃后的盘面与写行那次的 txg：那一槽里留着上一圈的根，环上多一个洞。
#[must_use]
pub fn crash_a_mount_before_its_row_root(image: &MemoryPool) -> (MemoryPool, u64) {
    let parameters = parameters_of_the_ring_hole_pools();
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(image, &stream);
    let mounted = mount_writable(&parameters, &mut recording).expect("被截的那次挂载");
    let row_txg = mounted.output.row_publish.root().checkpoint_txg.0;
    let (writes, _segments) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    let cut = writes
        .iter()
        .position(|write| {
            write.kind == StepKind::RootRecordFua
                && write
                    .bytes()
                    .and_then(|bytes| {
                        RootRecord::parse_slot(bytes, &FILESYSTEM_IDENTIFIER_OF_THE_RING_HOLE_POOLS)
                    })
                    .is_some_and(|root| root.checkpoint_txg.0 == row_txg)
        })
        .expect("写行那次的根槽写");
    let mut crashed = image.clone();
    crashed.apply_writes(&writes[..cut]);
    (crashed, row_txg)
}

/// 环里读得出的、txg 为 `checkpoint_txg` 的那条根。
#[must_use]
pub fn readable_root_at_txg(image: &MemoryPool, checkpoint_txg: u64) -> Option<RootRecord> {
    let system_configuration = choose_system_configuration(image).ok()?;
    let immutable = &system_configuration.immutable;
    readable_roots_with_ring_slots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|(_, root)| root)
    .find(|root| root.checkpoint_txg.0 == checkpoint_txg)
}
