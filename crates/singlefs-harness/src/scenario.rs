//! 第一个事务的整条路（mkfs → 取号 → 暖机 → 第一个事务），参数照 E142（第一个事务的干跑） 装置取：
//! 虚机里的真设备、宿主上的内存盘都跑这一份，同参数同字节（mkfs 与发布都不取系统时钟与随机数）。

use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
    WarmUpOutput,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;

use crate::SharedStream;

/// E142 装置里的固定 fsid（`FIXED_FSID`）。
pub const E142_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
/// E142 装置里的固定写入时间（`FIXED_WRITE_TIME_SECONDS`）。
pub const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// E142 的第一个文件 3000 字节（`name=config file_bytes=3000`）。
pub const FIRST_FILE_BYTES: usize = 3000;

#[must_use]
pub fn first_file_content() -> Vec<u8> {
    (0..FIRST_FILE_BYTES)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect()
}

/// 几何从设备探来：物理块大小、io_min；槽距 = 4096 向上取整到 io_min 的整数倍（D2（RAID 条带策略） 已定项 19）。
#[must_use]
pub fn e142_parameters(
    physical_block_size: u32,
    minimum_input_output_bytes: u32,
) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E142_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size,
            minimum_input_output_bytes,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(
                minimum_input_output_bytes,
            ),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        },
    }
}

pub struct FirstTransactionRun {
    pub mkfs_operation_count: usize,
    pub warm_up: WarmUpOutput,
    pub output: TransactionOutput,
    pub policy_mismatches: u64,
}

/// 整条路上调用方被叫到的两处。
pub enum ScenarioPoint {
    /// 取号写完、那道屏障做完，暖机还没开始（同一个写入口接着暖机，这里不另发屏障）。
    AfterInstanceAcquisition,
    /// 暖机之后、第一个事务之前（虚机档在这里给某块盘装上「漏一道屏障」）。
    BeforeFirstTransaction,
}

/// 整条路。`at_point` 在 `ScenarioPoint` 的两处各被叫一次：虚机档在两处给设备一层的计数拍快照，暖机与第一个事务各自的写就是快照之差。
pub fn run_first_transaction<
    Device: BlockDevice,
    AtPoint: FnMut(ScenarioPoint, &mut [(DeviceIdentity, Device)]),
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    stream: &SharedStream,
    mut at_point: AtPoint,
) -> Result<FirstTransactionRun, String> {
    let genesis =
        make_filesystem(parameters, devices).map_err(|error| format!("mkfs：{error:?}"))?;
    let mkfs_operation_count = stream.operations().len();
    let mut allocator = PoolAllocator::new(
        devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
    );
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let content = first_file_content();
    let (instance, warm_up_output) = {
        let mut pool = PoolWriter::new(parameters, &mut *devices);
        let instance = acquire_instance(&mut pool).map_err(|error| format!("取号：{error:?}"))?;
        at_point(ScenarioPoint::AfterInstanceAcquisition, &mut *pool.devices);
        let warm = warm_up(&mut pool, &genesis.root, instance)
            .map_err(|error| format!("暖机：{error:?}"))?;
        (instance, warm)
    };
    assert_eq!(instance, InstanceGeneration(1));
    at_point(ScenarioPoint::BeforeFirstTransaction, devices);
    let mut pool = PoolWriter::new(parameters, devices);
    let output = publish_first_file(
        &mut pool,
        &mut allocator,
        &genesis.root,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warm_up_output.last_record_bytes,
    )
    .map_err(|error| format!("第一个事务：{error:?}"))?;
    Ok(FirstTransactionRun {
        mkfs_operation_count,
        warm_up: warm_up_output,
        output,
        policy_mismatches: allocator.policy_mismatches,
    })
}
