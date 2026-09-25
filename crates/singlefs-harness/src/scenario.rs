//! 第一个事务的整条路（mkfs → 取号 → 暖机 → 第一个事务），参数照 E142（第一个事务的干跑） 装置取：
//! 虚机里的真设备、宿主上的内存盘都跑这一份，同参数同字节（mkfs 与发布都不取系统时钟与随机数）。

use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
    WarmUpOutput,
};
use singlefs_core::write_accounting::WritesByStructureKind;
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
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

pub struct FirstTransactionRun {
    pub mkfs_operation_count: usize,
    pub warm_up: WarmUpOutput,
    pub output: TransactionOutput,
}

/// 整条路上调用方被叫到的三处。
pub enum ScenarioPoint {
    /// mkfs 写完（含末尾那道屏障），取号还没开始。虚机档在这里切出 mkfs 那一段的挂钟。
    AfterMakeFilesystem,
    /// 取号写完、那道屏障做完，暖机还没开始（同一个写入口接着暖机，这里不另发屏障）。
    AfterInstanceAcquisition,
    /// 暖机之后、第一个事务之前（虚机档在这里给某块盘装上「漏一道屏障」）。
    BeforeFirstTransaction,
}

/// 整条路上的四步，按次序。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirstTransactionPathStep {
    MakeFilesystem,
    InstanceAcquisition,
    WarmUp,
    FirstTransaction,
}

impl FirstTransactionPathStep {
    /// 结果行里的名字，与虚机档分段时间的段名相同。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            FirstTransactionPathStep::MakeFilesystem => "mkfs",
            FirstTransactionPathStep::InstanceAcquisition => "instance_acquisition",
            FirstTransactionPathStep::WarmUp => "warm_up",
            FirstTransactionPathStep::FirstTransaction => "first_transaction",
        }
    }
}

/// 整条路没走完：停在哪一步、为什么，以及那一步的写入口交得出的账——落盘阶段失败的发布各自已记的写
/// （`PoolWriter::writes_of_failed_publishes`），与同一步里失败之前已经落盘的发布各自的写（暖机两次空发布里第二次失败时，第一次的账由
/// `transaction::WarmUpFailed` 交回）。里程碑「第二个事务」增补 2 收口表第 58 行：失败那次落盘的写不属于任何一次成功发布的账，
/// 同一步里已经落盘的那几次的账又随错一起丢，不交出来，设备一层数到的写就对不上。两样相加就是那一步的写入口记下的全部发布写。
/// mkfs 与取号不是发布，停在这两步时两样恒空；第一个事务那一步只有一次发布，第二样恒空。
#[derive(Debug)]
pub struct FirstTransactionPathFailure {
    pub failed_step: FirstTransactionPathStep,
    pub cause: String,
    pub writes_of_failed_publishes: Vec<WritesByStructureKind>,
    pub writes_of_persisted_publishes: Vec<WritesByStructureKind>,
}

impl FirstTransactionPathFailure {
    fn before_any_publish(failed_step: FirstTransactionPathStep, cause: String) -> Self {
        Self {
            failed_step,
            cause,
            writes_of_failed_publishes: Vec::new(),
            writes_of_persisted_publishes: Vec::new(),
        }
    }

    /// 给人看的一句：哪一步、什么错。
    #[must_use]
    pub fn render(&self) -> String {
        format!("{}：{}", self.failed_step.name(), self.cause)
    }
}

/// 整条路。`at_point` 在 `ScenarioPoint` 的三处各被叫一次：虚机档在这三处给设备一层的计数与单调时钟拍快照，
/// 每一段的写与挂钟就是相邻两处之差。
///
/// # Errors
/// 哪一步没做成，连同那一步的写入口交出来的失败账（[`FirstTransactionPathFailure`]）。
pub fn run_first_transaction<
    Device: BlockDevice,
    AtPoint: FnMut(ScenarioPoint, &mut [(DeviceIdentity, Device)]),
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    stream: &SharedStream,
    mut at_point: AtPoint,
) -> Result<FirstTransactionRun, FirstTransactionPathFailure> {
    let genesis = make_filesystem(parameters, devices).map_err(|error| {
        FirstTransactionPathFailure::before_any_publish(
            FirstTransactionPathStep::MakeFilesystem,
            format!("{error:?}"),
        )
    })?;
    let mkfs_operation_count = stream.operations().len();
    at_point(ScenarioPoint::AfterMakeFilesystem, devices);
    let mut allocator = allocator_after_make_filesystem(parameters, devices, &genesis);
    let content = first_file_content();
    let (instance, warm_up_output) = {
        let mut pool = PoolWriter::new(parameters, &mut *devices);
        let instance = acquire_instance(&mut pool).map_err(|error| {
            FirstTransactionPathFailure::before_any_publish(
                FirstTransactionPathStep::InstanceAcquisition,
                format!("{error:?}"),
            )
        })?;
        at_point(ScenarioPoint::AfterInstanceAcquisition, &mut *pool.devices);
        let warm = warm_up(&mut pool, &genesis.root, instance).map_err(|failed| {
            FirstTransactionPathFailure {
                failed_step: FirstTransactionPathStep::WarmUp,
                cause: format!("{:?}", failed.cause),
                writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),
                writes_of_persisted_publishes: failed.writes_of_persisted_publishes,
            }
        })?;
        (instance, warm)
    };
    assert_eq!(instance, InstanceGeneration(1));
    at_point(ScenarioPoint::BeforeFirstTransaction, devices);
    let mut pool = PoolWriter::new(parameters, devices);
    let output = publish_first_file(
        &mut pool,
        &mut allocator,
        warm_up_output.roots.last().expect("暖机两代根"),
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warm_up_output.last_record_bytes,
    )
    .map_err(|error| FirstTransactionPathFailure {
        failed_step: FirstTransactionPathStep::FirstTransaction,
        cause: format!("{error:?}"),
        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),
        writes_of_persisted_publishes: Vec::new(),
    })?;
    Ok(FirstTransactionRun {
        mkfs_operation_count,
        warm_up: warm_up_output,
        output,
    })
}
