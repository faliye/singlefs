//! 新池新建文件的整条路（mkfs → 取号 → 暖机 → 新池新建文件），参数照 E142（新池新建文件的干跑） 装置取：
//! 虚机里的真设备、宿主上的内存盘都跑这一份，同参数同字节（mkfs 与发布都不取系统时钟与随机数）。

use std::ops::Range;

use singlefs_core::address::{DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::BlockDevice;
use singlefs_core::instance_table_sweep::InstanceTableSweepAtTheEndOfTheMount;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, unmount,
    DeletionQueueContinuationOutcome, ShadowLedger, Unmounted,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_sequential_write,
    publish_unlink_intent, publish_unlink_step, warm_up, FirstFile, PoolVersion, PoolWriter,
    TransactionOutput, WarmUpOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_core::write_accounting::WritesByStructureKind;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;

use crate::{RecordedOperation, RecordedOperationKind, RecordedPublishEntry, SharedStream};

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

pub struct NewPoolFileCreationRun {
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
    /// 暖机之后、新池新建文件之前（虚机档在这里给某块盘装上「漏一道屏障」）。
    BeforeNewPoolFileCreation,
}

/// 整条路上的四步，按次序。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewPoolFileCreationPathStep {
    MakeFilesystem,
    InstanceAcquisition,
    WarmUp,
    NewPoolFileCreation,
}

impl NewPoolFileCreationPathStep {
    /// 结果行里的名字，与虚机档分段时间的段名相同。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            NewPoolFileCreationPathStep::MakeFilesystem => "mkfs",
            NewPoolFileCreationPathStep::InstanceAcquisition => "instance_acquisition",
            NewPoolFileCreationPathStep::WarmUp => "warm_up",
            NewPoolFileCreationPathStep::NewPoolFileCreation => "new_pool_file_creation",
        }
    }
}

/// 整条路没走完：停在哪一步、为什么，以及那一步的写入口交得出的账——落盘阶段失败的发布各自已记的写
/// （`PoolWriter::writes_of_failed_publishes`），与同一步里失败之前已经落盘的发布各自的写（暖机两次空发布里第二次失败时，第一次的账由
/// `transaction::WarmUpFailed` 交回）。里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 58 行：失败那次落盘的写不属于任何一次成功发布的账，
/// 同一步里已经落盘的那几次的账又随错一起丢，不交出来，设备一层数到的写就对不上。两样相加就是那一步的写入口记下的全部发布写。
/// mkfs 与取号不是发布，停在这两步时两样恒空；新池新建文件那一步只有一次发布，第二样恒空。
#[derive(Debug)]
pub struct NewPoolFileCreationPathFailure {
    pub failed_step: NewPoolFileCreationPathStep,
    pub cause: String,
    pub writes_of_failed_publishes: Vec<WritesByStructureKind>,
    pub writes_of_persisted_publishes: Vec<WritesByStructureKind>,
}

impl NewPoolFileCreationPathFailure {
    fn before_any_publish(failed_step: NewPoolFileCreationPathStep, cause: String) -> Self {
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
/// 哪一步没做成，连同那一步的写入口交出来的失败账（[`NewPoolFileCreationPathFailure`]）。
pub fn run_new_pool_file_creation<
    Device: BlockDevice,
    AtPoint: FnMut(ScenarioPoint, &mut [(DeviceIdentity, Device)]),
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    stream: &SharedStream,
    at_point: AtPoint,
) -> Result<NewPoolFileCreationRun, NewPoolFileCreationPathFailure> {
    run_new_pool_file_creation_keeping_the_allocator(parameters, devices, stream, at_point)
        .map(|(run, _allocator)| run)
}

/// 同 [`run_new_pool_file_creation`]，连同新池新建文件之后的分配器一起交回：同一个进程接着发布（删除那两段历史）要接着用它。
fn run_new_pool_file_creation_keeping_the_allocator<
    Device: BlockDevice,
    AtPoint: FnMut(ScenarioPoint, &mut [(DeviceIdentity, Device)]),
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    stream: &SharedStream,
    mut at_point: AtPoint,
) -> Result<(NewPoolFileCreationRun, PoolAllocator), NewPoolFileCreationPathFailure> {
    let genesis = make_filesystem(parameters, devices).map_err(|error| {
        NewPoolFileCreationPathFailure::before_any_publish(
            NewPoolFileCreationPathStep::MakeFilesystem,
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
            NewPoolFileCreationPathFailure::before_any_publish(
                NewPoolFileCreationPathStep::InstanceAcquisition,
                format!("{error:?}"),
            )
        })?;
        at_point(ScenarioPoint::AfterInstanceAcquisition, &mut *pool.devices);
        let warm = warm_up(&mut pool, &genesis.root, instance).map_err(|failed| {
            NewPoolFileCreationPathFailure {
                failed_step: NewPoolFileCreationPathStep::WarmUp,
                cause: format!("{:?}", failed.cause),
                writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),
                writes_of_persisted_publishes: failed.writes_of_persisted_publishes,
            }
        })?;
        (instance, warm)
    };
    assert_eq!(instance, InstanceGeneration(1));
    at_point(ScenarioPoint::BeforeNewPoolFileCreation, devices);
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
    .map_err(|error| NewPoolFileCreationPathFailure {
        failed_step: NewPoolFileCreationPathStep::NewPoolFileCreation,
        cause: format!("{error:?}"),
        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),
        writes_of_persisted_publishes: Vec::new(),
    })?;
    Ok((
        NewPoolFileCreationRun {
            mkfs_operation_count,
            warm_up: warm_up_output,
            output,
        },
        allocator,
    ))
}

/// 新池新建文件那个实例：删除历史里在这个进程里发的每一次（H3 建 size 0 inode 那次、顺序写、意图、H1 与 H3 的各批）都在它里面发。
const INSTANCE_OF_THE_NEW_POOL_FILE_CREATION: InstanceGeneration = InstanceGeneration(1);

/// 删除那两段历史删的是第一个文件。
const FILE_DELETED_IN_THE_DELETION_HISTORIES: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);

/// 删除那两段历史里顺序写比新池新建文件晚几秒（与 harness 档 unlink 那几条用例的 `OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE` 同一个数）。
pub const SECONDS_FROM_THE_FIRST_FILE_TO_THE_SEQUENTIAL_WRITE_IN_THE_DELETION_HISTORIES: u64 = 60;

/// H3 那个 size 0 inode 建出来时的写入时间。E172 跑前登记第五节 5.3 没给这个时间；取新池新建文件那一个，
/// 与顺序写（晚 60 秒）、意图（再晚 60 秒）三个时间各不相同，比对侧读错了哪条记录的时间看得出来。
pub const WRITE_TIME_SECONDS_OF_THE_SIZE_ZERO_INODE_BESIDE_THE_DELETED_FILE: u64 =
    FIXED_WRITE_TIME_SECONDS;

/// 删除那两段历史里第一个文件顺序写成的内容：两个满净荷加半个净荷，三个数据单元（与 harness 档 unlink 那几条用例的
/// `pool_with_a_three_unit_file` 同一份字节）。
#[must_use]
pub fn three_data_unit_file_content() -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    (0..2 * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 13 + 5) % 251).expect("对 251 取余小于 256"))
        .collect()
}

/// 删除那两段历史的批大小：只供测试的批大小旋钮压到每批一个数据单元（`.claude/rules/fs-design.md` 五条硬要求第 2 条），
/// 三个数据单元的文件走「留 2、留 1、最后一批」三批。
#[must_use]
pub fn deletion_history_batch_size() -> UnlinkBatchSize {
    UnlinkBatchSize::CappedForTests(
        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=产品批大小里"),
    )
}

/// H2 那次可写挂载装的删除开关：批大小同 [`deletion_history_batch_size`]，三个容量照产品路径。
#[must_use]
pub fn deletion_history_switches() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: deletion_history_batch_size(),
        tombstone_container_capacity: DeletionSwitches::PRODUCT.tombstone_container_capacity,
        lineage_container_capacity: DeletionSwitches::PRODUCT.lineage_container_capacity,
        retirement_container_capacity: DeletionSwitches::PRODUCT.retirement_container_capacity,
        container_tree_node_entries: DeletionSwitches::PRODUCT.container_tree_node_entries,
    }
}

/// H3 里意图那次的调用方时间比顺序写晚几秒（E172 跑前登记第五节 5.3 ②：与那条记录现有的时间不同，意图写的是不是调用方给的那一个才分辨得出）。
pub const SECONDS_FROM_THE_SEQUENTIAL_WRITE_TO_THE_INTENT_IN_THE_HISTORY_BESIDE_A_SIZE_ZERO_INODE:
    u64 = 60;

/// 三·6 干跑导出的删除历史（调查报告第 4.3 节表里的 H1、H2；H3 是 E172 跑前登记第五节 5.3 加的变体）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeletionHistory {
    /// H1：意图与每一批都在新池新建文件那个进程里发，直到队列空。
    IntentAndEveryBatchInProcess,
    /// H2：意图在这个进程里发，正常卸载，同一组盘重开可写挂载，挂载续做在写行与暖机之后把队列走到空。
    IntentThenUnmountThenMountContinuation,
    /// H3：同 H1，只改两处——顺序写之前在同一个实例里建一个 size 0 的 inode（号 2，与 1 号在同一片 inode 叶容器里）；
    /// 意图的调用方时间 = 顺序写的时间 + [`SECONDS_FROM_THE_SEQUENTIAL_WRITE_TO_THE_INTENT_IN_THE_HISTORY_BESIDE_A_SIZE_ZERO_INODE`]。
    LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess,
}

/// 删除历史里被删的第一个文件旁边还有没有别的 inode。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InodesBesideTheDeletedFile {
    /// 只有第一个文件（H1、H2）。
    NoOtherInode,
    /// 顺序写之前在同一个实例里用 `publish_new_inodes` 建一个 size 0 的 inode（H3）。
    OneSizeZeroInodeCreatedBeforeTheSequentialWrite,
}

/// 意图那次发布的调用方时间从哪来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeletionHistoryIntentWriteTime {
    /// 第一个文件那条记录现有的写入时间（顺序写写进去的那一个）：与 harness 档 unlink 那几条用例同一个取法（H1、H2）。
    TheFileRecordsCurrentWriteTime,
    /// 那条记录现有的写入时间再加
    /// [`SECONDS_FROM_THE_SEQUENTIAL_WRITE_TO_THE_INTENT_IN_THE_HISTORY_BESIDE_A_SIZE_ZERO_INODE`]（H3）。
    SecondsAfterTheSequentialWrite,
}

impl DeletionHistory {
    pub const EVERY_HISTORY: [DeletionHistory; 3] = [
        DeletionHistory::IntentAndEveryBatchInProcess,
        DeletionHistory::IntentThenUnmountThenMountContinuation,
        DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess,
    ];

    /// 结果行里的名字，与调查报告那张表的行名相同（H3 与 E172 跑前登记第五节 5.3 的名字相同）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            DeletionHistory::IntentAndEveryBatchInProcess => "h1",
            DeletionHistory::IntentThenUnmountThenMountContinuation => "h2",
            DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess => "h3",
        }
    }

    #[must_use]
    pub const fn inodes_beside_the_deleted_file(self) -> InodesBesideTheDeletedFile {
        match self {
            DeletionHistory::IntentAndEveryBatchInProcess
            | DeletionHistory::IntentThenUnmountThenMountContinuation => {
                InodesBesideTheDeletedFile::NoOtherInode
            }
            DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess => {
                InodesBesideTheDeletedFile::OneSizeZeroInodeCreatedBeforeTheSequentialWrite
            }
        }
    }

    #[must_use]
    pub const fn intent_write_time(self) -> DeletionHistoryIntentWriteTime {
        match self {
            DeletionHistory::IntentAndEveryBatchInProcess
            | DeletionHistory::IntentThenUnmountThenMountContinuation => {
                DeletionHistoryIntentWriteTime::TheFileRecordsCurrentWriteTime
            }
            DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess => {
                DeletionHistoryIntentWriteTime::SecondsAfterTheSequentialWrite
            }
        }
    }
}

/// 删除历史里的一次发布是哪一种。删除类是前两种与最后一种，其余三种是这段历史里别的发布。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeletionHistoryPublishKind {
    /// unlink 的意图（`publish_unlink_intent`）。
    UnlinkIntent,
    /// 这个进程里发的一批（`publish_unlink_step`），最后一批也是它。
    UnlinkBatchInProcess,
    /// 正常卸载那一串带卸载记号的空发布。
    UnmountFloorRaise,
    /// 可写挂载写行那次发布。
    MountInstanceRow,
    /// 可写挂载写行之后的暖机空发布。
    MountWarmUp,
    /// 挂载续做发的一批，最后一批也是它。
    UnlinkBatchOfTheMountContinuation,
}

impl DeletionHistoryPublishKind {
    /// 结果行里的名字。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            DeletionHistoryPublishKind::UnlinkIntent => "unlink_intent",
            DeletionHistoryPublishKind::UnlinkBatchInProcess => "unlink_batch",
            DeletionHistoryPublishKind::UnmountFloorRaise => "unmount_floor_raise",
            DeletionHistoryPublishKind::MountInstanceRow => "mount_instance_row",
            DeletionHistoryPublishKind::MountWarmUp => "mount_warm_up",
            DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation => {
                "mount_continuation_unlink_batch"
            }
        }
    }
}

/// 删除历史里不属于任何一次发布的写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeletionHistoryWritesOutsideAnyPublish {
    /// 正常卸载在它第一次发布之前先把新 F 写进每块盘系统配置那一步。
    UnmountSystemConfigurationBeforeTheFirstFloorRaise,
    /// 可写挂载在写行那次发布之前发的写（core 的发布写账不记它们，按「挂载那一截的写数减去各次发布的写账」算出来）。
    MountWritesBeforeItsFirstPublish,
}

impl DeletionHistoryWritesOutsideAnyPublish {
    /// 结果行里的名字。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            DeletionHistoryWritesOutsideAnyPublish::UnmountSystemConfigurationBeforeTheFirstFloorRaise => {
                "unmount_system_configuration_before_the_first_floor_raise"
            }
            DeletionHistoryWritesOutsideAnyPublish::MountWritesBeforeItsFirstPublish => {
                "mount_writes_before_its_first_publish"
            }
        }
    }
}

/// 删除历史里 unlink 之后的一段：`operations` 是录制流下标的半开区间（与 [`SharedStream::operations`] 同一套下标）。
#[derive(Clone, Debug)]
pub enum DeletionHistorySegment {
    Publish {
        kind: DeletionHistoryPublishKind,
        version: Box<TransactionOutput>,
        operations: Range<usize>,
    },
    WritesOutsideAnyPublish {
        kind: DeletionHistoryWritesOutsideAnyPublish,
        operations: Range<usize>,
    },
}

impl DeletionHistorySegment {
    #[must_use]
    pub fn operations(&self) -> Range<usize> {
        match self {
            DeletionHistorySegment::Publish { operations, .. }
            | DeletionHistorySegment::WritesOutsideAnyPublish { operations, .. } => {
                operations.clone()
            }
        }
    }
}

/// 一段删除历史跑完交回的东西。
pub struct DeletionHistoryRun {
    pub mkfs_operation_count: usize,
    /// 意图那次发布之前录制流的长度：导出的窗口从这里起。
    pub operations_before_the_unlink: usize,
    /// 场景交给意图那次发布的调用方时间（[`DeletionHistory::intent_write_time`] 定取法）。是交进去的值，不是 core 写进记录的值：
    /// 两者相不相等正是要判的。
    pub intent_write_time_seconds: u64,
    /// unlink 之后的每一段，按录制流的先后，首尾相接铺满 [`operations_before_the_unlink`](Self::operations_before_the_unlink) 到流尾。
    pub segments: Vec<DeletionHistorySegment>,
    /// 这段历史最后的现行那一版：H1 是这个进程最后一批，H2 是挂载交回的现行那一版。
    pub final_version: PoolVersion,
}

/// 删除历史里的哪一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeletionHistoryStep {
    NewPoolFileCreationPath(NewPoolFileCreationPathStep),
    /// H3 顺序写之前建 size 0 inode 的那次发布。
    SizeZeroInodeCreation,
    SequentialWrite,
    UnlinkIntent,
    UnlinkBatchInProcess,
    Unmount,
    WritableMount,
    /// 卸载与挂载那两截按 core 的写账切开那一步。
    AttributingTheWritesToPublishes,
}

/// 删除历史没走完：停在哪一步、为什么。
#[derive(Debug)]
pub struct DeletionHistoryFailure {
    pub failed_step: DeletionHistoryStep,
    pub cause: String,
}

impl DeletionHistoryFailure {
    fn at(failed_step: DeletionHistoryStep, cause: String) -> Self {
        Self { failed_step, cause }
    }
}

/// 一次调用里先后发出的一段，切开之前：哪一种、core 的写账记了几次写调用。
struct PartOfACall {
    part: PartOfACallKind,
    write_calls: u64,
}

enum PartOfACallKind {
    Publish {
        kind: DeletionHistoryPublishKind,
        version: Box<TransactionOutput>,
    },
    OutsideAnyPublish(DeletionHistoryWritesOutsideAnyPublish),
}

impl PartOfACall {
    fn publish(kind: DeletionHistoryPublishKind, version: TransactionOutput) -> Self {
        Self {
            write_calls: version.writes.total().write_calls,
            part: PartOfACallKind::Publish {
                kind,
                version: Box::new(version),
            },
        }
    }
}

/// 一次调用在录制流里发出的那一截（`operations_of_the_call`），按先后几段各自的写数（`write_calls_of_each_part`，每个都至少 1）切开：
/// 第 k 段是第 k 个写数那么多次写，加上紧跟其后、下一段第一次写之前的屏障；这一截开头、第一次写之前的屏障归第一段。
/// 交回各段在这一截里的半开区间，首尾相接铺满整截。只看操作是不是屏障，不看偏移、长度与内容。
///
/// # Errors
/// 有一段写数是 0，或这一截的写数与各段之和对不上（多了或少了）。
pub fn spans_of_consecutive_parts_by_their_write_calls(
    operations_of_the_call: &[RecordedOperation],
    write_calls_of_each_part: &[u64],
) -> Result<Vec<Range<usize>>, String> {
    if write_calls_of_each_part.contains(&0) {
        return Err(format!(
            "每段至少一次写才切得开：{write_calls_of_each_part:?}"
        ));
    }
    let writes_in_the_call = operations_of_the_call
        .iter()
        .filter(|operation| operation.kind != RecordedOperationKind::Barrier)
        .count();
    let writes_of_the_parts: u64 = write_calls_of_each_part.iter().sum();
    if u64::try_from(writes_in_the_call).expect("录制流的长度装得进 u64") != writes_of_the_parts
    {
        return Err(format!(
            "这一截有 {writes_in_the_call} 次写，各段写账加起来 {writes_of_the_parts}：{write_calls_of_each_part:?}"
        ));
    }
    let mut spans = Vec::with_capacity(write_calls_of_each_part.len());
    let mut part_start = 0;
    let mut part_index = 0;
    let mut writes_in_this_part = 0;
    // 跨轮携带的状态：`part_index` 是正在装的那一段，`writes_in_this_part` 是它已装的写数，不超过它的写账
    // （上面核过总数相等，装满一段才遇到下一次写时换段，`part_index` 不越界）。迭代次数的上界是这一截的长度，没有提前出口。
    for (index, operation) in operations_of_the_call.iter().enumerate() {
        if operation.kind == RecordedOperationKind::Barrier {
            continue;
        }
        if writes_in_this_part == write_calls_of_each_part[part_index] {
            spans.push(part_start..index);
            part_index += 1;
            part_start = index;
            writes_in_this_part = 0;
        }
        writes_in_this_part += 1;
        assert!(
            writes_in_this_part <= write_calls_of_each_part[part_index],
            "装满一段才换段：第 {part_index} 段装了 {writes_in_this_part} 次写"
        );
    }
    spans.push(part_start..operations_of_the_call.len());
    assert_eq!(
        spans.len(),
        write_calls_of_each_part.len(),
        "总数相等时每段都装满了、最后一段收在这一截的尾上"
    );
    Ok(spans)
}

/// 把一次调用在录制流里发出的那一截按 `parts` 的写账切成几段；写数是 0 的「不属于任何一次发布的写」不成段。
fn segments_of_one_call(
    stream: &SharedStream,
    operations_of_the_call: Range<usize>,
    parts: Vec<PartOfACall>,
) -> Result<Vec<DeletionHistorySegment>, DeletionHistoryFailure> {
    let parts: Vec<PartOfACall> = parts
        .into_iter()
        .filter(|part| match part.part {
            PartOfACallKind::Publish { .. } => true,
            PartOfACallKind::OutsideAnyPublish(_) => part.write_calls > 0,
        })
        .collect();
    let write_calls: Vec<u64> = parts.iter().map(|part| part.write_calls).collect();
    let recorded_operations = stream.operations();
    let spans = spans_of_consecutive_parts_by_their_write_calls(
        &recorded_operations[operations_of_the_call.clone()],
        &write_calls,
    )
    .map_err(|cause| {
        DeletionHistoryFailure::at(DeletionHistoryStep::AttributingTheWritesToPublishes, cause)
    })?;
    Ok(parts
        .into_iter()
        .zip(spans)
        .map(|(part, span)| {
            let operations =
                operations_of_the_call.start + span.start..operations_of_the_call.start + span.end;
            match part.part {
                PartOfACallKind::Publish { kind, version } => DeletionHistorySegment::Publish {
                    kind,
                    version,
                    operations,
                },
                PartOfACallKind::OutsideAnyPublish(kind) => {
                    DeletionHistorySegment::WritesOutsideAnyPublish { kind, operations }
                }
            }
        })
        .collect())
}

/// 三·6 干跑那两段删除历史（D8（核心索引结构） 已定项 16）：E142 那套参数上 mkfs → 取号 → 暖机 → 新池新建文件
/// （与 [`run_new_pool_file_creation`] 同一条路）→ 同一个实例里把第一个文件顺序写成三个数据单元（[`three_data_unit_file_content`]）
/// → unlink，批大小 [`deletion_history_batch_size`]。H1 意图与每一批都在这个进程里发，直到队列空；H2 意图在这个进程里发，
/// 正常卸载（[`unmount`]，录制流把那一截记成卸载入口），丢掉这个进程的分配器，同一组盘重开可写挂载（[`deletion_history_switches`]）：
/// 挂载续做在写行与暖机之后把队列走到空。H3 照 H1，只多两处（[`DeletionHistory::inodes_beside_the_deleted_file`]、
/// [`DeletionHistory::intent_write_time`]）：顺序写之前先建一个 size 0 的 inode，意图的调用方时间晚顺序写 60 秒。
///
/// 交回 unlink 之前录制流的长度与它之后的每一段：直接调的发布各一段；卸载与挂载里的几次发布按 core 交回的每次发布写账
/// （`TransactionOutput::writes`，一次写调用 = 交给一块盘的一次写）在流里切开（[`spans_of_consecutive_parts_by_their_write_calls`]）。
///
/// # Errors
/// 哪一步没做成、为什么。挂载推了抬 F、做了清扫发布、续做没走到空、某一版没有树，或写账与流里那一截的写数对不上，
/// 都报在 [`DeletionHistoryStep::AttributingTheWritesToPublishes`]：这几样这段历史不该出现，出现了就切不准。
pub fn run_deletion_history<Device: BlockDevice>(
    history: DeletionHistory,
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    stream: &SharedStream,
) -> Result<DeletionHistoryRun, DeletionHistoryFailure> {
    let (new_pool_file_creation, mut allocator) = run_new_pool_file_creation_keeping_the_allocator(
        parameters,
        devices.as_mut_slice(),
        stream,
        |_point, _devices| {},
    )
    .map_err(|failure| {
        DeletionHistoryFailure::at(
            DeletionHistoryStep::NewPoolFileCreationPath(failure.failed_step),
            failure.cause,
        )
    })?;
    let version_before_the_sequential_write = match history.inodes_beside_the_deleted_file() {
        InodesBesideTheDeletedFile::NoOtherInode => new_pool_file_creation.output,
        InodesBesideTheDeletedFile::OneSizeZeroInodeCreatedBeforeTheSequentialWrite => {
            publish_new_inodes(
                &mut PoolWriter::new(parameters, devices.as_mut_slice()),
                &mut allocator,
                &new_pool_file_creation.output,
                1,
                WRITE_TIME_SECONDS_OF_THE_SIZE_ZERO_INODE_BESIDE_THE_DELETED_FILE,
                INSTANCE_OF_THE_NEW_POOL_FILE_CREATION,
            )
            .map_err(|error| {
                DeletionHistoryFailure::at(
                    DeletionHistoryStep::SizeZeroInodeCreation,
                    format!("{error:?}"),
                )
            })?
        }
    };
    let content = three_data_unit_file_content();
    let three_units = publish_sequential_write(
        &mut PoolWriter::new(parameters, devices.as_mut_slice()),
        &mut allocator,
        &version_before_the_sequential_write,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS
                + SECONDS_FROM_THE_FIRST_FILE_TO_THE_SEQUENTIAL_WRITE_IN_THE_DELETION_HISTORIES,
        },
        INSTANCE_OF_THE_NEW_POOL_FILE_CREATION,
    )
    .map_err(|error| {
        DeletionHistoryFailure::at(DeletionHistoryStep::SequentialWrite, format!("{error:?}"))
    })?;
    let operations_before_the_unlink = stream.operation_count();
    let write_time_seconds = three_units
        .inode_record
        .expect("顺序写之后第一个文件那条记录在树里")
        .write_time_seconds;
    let intent_write_time_seconds = match history.intent_write_time() {
        DeletionHistoryIntentWriteTime::TheFileRecordsCurrentWriteTime => write_time_seconds,
        DeletionHistoryIntentWriteTime::SecondsAfterTheSequentialWrite => {
            write_time_seconds
                + SECONDS_FROM_THE_SEQUENTIAL_WRITE_TO_THE_INTENT_IN_THE_HISTORY_BESIDE_A_SIZE_ZERO_INODE
        }
    };
    let intent = publish_unlink_intent(
        &mut PoolWriter::new(parameters, devices.as_mut_slice()),
        &mut allocator,
        &three_units,
        FILE_DELETED_IN_THE_DELETION_HISTORIES,
        intent_write_time_seconds,
        INSTANCE_OF_THE_NEW_POOL_FILE_CREATION,
    )
    .map_err(|error| {
        DeletionHistoryFailure::at(DeletionHistoryStep::UnlinkIntent, format!("{error:?}"))
    })?;
    let intent_segment = DeletionHistorySegment::Publish {
        kind: DeletionHistoryPublishKind::UnlinkIntent,
        version: Box::new(intent.clone()),
        operations: operations_before_the_unlink..stream.operation_count(),
    };
    let (later_segments, final_version) = match history {
        DeletionHistory::IntentAndEveryBatchInProcess
        | DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess => {
            every_batch_in_process(parameters, devices, stream, &mut allocator, intent)?
        }
        DeletionHistory::IntentThenUnmountThenMountContinuation => {
            unmount_then_mount_continuation(parameters, devices, stream, allocator, intent)?
        }
    };
    Ok(DeletionHistoryRun {
        mkfs_operation_count: new_pool_file_creation.mkfs_operation_count,
        operations_before_the_unlink,
        intent_write_time_seconds,
        segments: std::iter::once(intent_segment)
            .chain(later_segments)
            .collect(),
        final_version,
    })
}

/// H1 意图之后：这个进程里一批一批发，直到队列空。
fn every_batch_in_process<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    stream: &SharedStream,
    allocator: &mut PoolAllocator,
    intent: TransactionOutput,
) -> Result<(Vec<DeletionHistorySegment>, PoolVersion), DeletionHistoryFailure> {
    // 迭代次数的上界：每批至少截掉一个数据单元，意图那一版有几个数据单元就至多几批；跨轮携带的是现行那一版，
    // 出口是队列空。上界用完队列还不空就是批没截短，报错、不再发。
    let batch_limit = intent.data_pointers.len();
    let mut segments = Vec::with_capacity(batch_limit);
    let mut current = intent;
    for _ in 0..batch_limit {
        if current.deleted_inodes.is_empty() {
            break;
        }
        let start = stream.operation_count();
        current = publish_unlink_step(
            &mut PoolWriter::new(parameters, devices.as_mut_slice()),
            allocator,
            &current,
            FILE_DELETED_IN_THE_DELETION_HISTORIES,
            deletion_history_batch_size(),
            INSTANCE_OF_THE_NEW_POOL_FILE_CREATION,
        )
        .map_err(|error| {
            DeletionHistoryFailure::at(
                DeletionHistoryStep::UnlinkBatchInProcess,
                format!("{error:?}"),
            )
        })?;
        segments.push(DeletionHistorySegment::Publish {
            kind: DeletionHistoryPublishKind::UnlinkBatchInProcess,
            version: Box::new(current.clone()),
            operations: start..stream.operation_count(),
        });
    }
    if !current.deleted_inodes.is_empty() {
        return Err(DeletionHistoryFailure::at(
            DeletionHistoryStep::UnlinkBatchInProcess,
            format!(
                "发了 {batch_limit} 批队列还不空：{:?}",
                current.deleted_inodes
            ),
        ));
    }
    Ok((segments, PoolVersion::of_a_version_of_trees(current)))
}

/// 一版要有树才切得开：写行与暖机发在有树的一版上，交回的就是有树的一版。
fn version_of_trees_of_a_mount_publish(
    version: &PoolVersion,
) -> Result<TransactionOutput, DeletionHistoryFailure> {
    version.version_of_trees().cloned().ok_or_else(|| {
        DeletionHistoryFailure::at(
            DeletionHistoryStep::AttributingTheWritesToPublishes,
            format!(
                "挂载里 txg {:?} 那次发布交回的一版没有树",
                version.root().checkpoint_txg
            ),
        )
    })
}

/// H2 意图之后：正常卸载，进程退出，同一组盘重开可写挂载，挂载续做走到空。
fn unmount_then_mount_continuation<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    stream: &SharedStream,
    mut allocator: PoolAllocator,
    intent: TransactionOutput,
) -> Result<(Vec<DeletionHistorySegment>, PoolVersion), DeletionHistoryFailure> {
    let mut current = PoolVersion::WithFile(intent);
    let unmount_start = stream.operation_count();
    let unmounted = stream
        .record_entry(RecordedPublishEntry::Unmount, || {
            unmount(
                parameters,
                devices,
                &mut allocator,
                &mut current,
                ShadowLedger::On,
            )
        })
        .map_err(|error| {
            DeletionHistoryFailure::at(DeletionHistoryStep::Unmount, format!("{error:?}"))
        })?;
    let unmount_operations = unmount_start..stream.operation_count();
    let raised = match unmounted {
        Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised,
        Unmounted::NothingWrittenOnAVersionWithoutFile {
            current: version_the_unmount_saw,
        } => {
            return Err(DeletionHistoryFailure::at(
                DeletionHistoryStep::Unmount,
                format!("意图那一版带文件，卸载却一个字节都没写：{version_the_unmount_saw:?}"),
            ));
        }
    };
    // 进程退出：这个进程的分配器随它丢掉；盘留着，重开就是同一组盘接着用。
    drop(allocator);
    let mut unmount_parts = vec![PartOfACall {
        write_calls: raised
            .system_configuration_writes_before_the_first_publish
            .total()
            .write_calls,
        part: PartOfACallKind::OutsideAnyPublish(
            DeletionHistoryWritesOutsideAnyPublish::UnmountSystemConfigurationBeforeTheFirstFloorRaise,
        ),
    }];
    unmount_parts.extend(raised.publishes.into_iter().map(|publish| {
        PartOfACall::publish(DeletionHistoryPublishKind::UnmountFloorRaise, publish)
    }));
    let unmount_segments = segments_of_one_call(stream, unmount_operations, unmount_parts)?;

    let mount_start = stream.operation_count();
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        parameters,
        devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_history_switches(),
    )
    .map_err(|error| {
        DeletionHistoryFailure::at(DeletionHistoryStep::WritableMount, format!("{error:?}"))
    })?;
    let mount_operations = mount_start..stream.operation_count();
    let output = &mounted.output;
    let publishes_this_history_does_not_attribute = [
        (
            !output.space_admission.floor_raises().is_empty(),
            "挂载在写行之后推了抬 F",
        ),
        (
            !output.floor_raises_pushed_by_the_continuation.is_empty(),
            "挂载续做推了抬 F",
        ),
        (
            matches!(
                output.instance_table_sweep,
                InstanceTableSweepAtTheEndOfTheMount::Swept(_)
            ),
            "挂载末尾判了实例表清扫",
        ),
        (
            !matches!(
                output.deletion_queue_continuation_outcome,
                DeletionQueueContinuationOutcome::WalkedToEmpty
            ),
            "挂载续做没走到空",
        ),
    ];
    if let Some((_, what)) = publishes_this_history_does_not_attribute
        .iter()
        .find(|(happened, _)| *happened)
    {
        return Err(DeletionHistoryFailure::at(
            DeletionHistoryStep::AttributingTheWritesToPublishes,
            format!("{what}：这段历史只切写行、暖机与续做各批"),
        ));
    }
    let mut publishes = vec![PartOfACall::publish(
        DeletionHistoryPublishKind::MountInstanceRow,
        version_of_trees_of_a_mount_publish(&output.row_publish)?,
    )];
    for warm_up_publish in &output.warm_up_publishes {
        publishes.push(PartOfACall::publish(
            DeletionHistoryPublishKind::MountWarmUp,
            version_of_trees_of_a_mount_publish(warm_up_publish)?,
        ));
    }
    publishes.extend(output.deletion_queue_continuation.iter().map(|step| {
        PartOfACall::publish(
            DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation,
            step.clone(),
        )
    }));
    let writes_of_the_publishes: u64 = publishes.iter().map(|part| part.write_calls).sum();
    let writes_in_the_mount = u64::try_from(
        stream.operations()[mount_operations.clone()]
            .iter()
            .filter(|operation| operation.kind != RecordedOperationKind::Barrier)
            .count(),
    )
    .expect("录制流的长度装得进 u64");
    let writes_before_the_first_publish = writes_in_the_mount
        .checked_sub(writes_of_the_publishes)
        .ok_or_else(|| {
            DeletionHistoryFailure::at(
                DeletionHistoryStep::AttributingTheWritesToPublishes,
                format!(
                    "挂载那一截 {writes_in_the_mount} 次写，少于各次发布写账之和 {writes_of_the_publishes}"
                ),
            )
        })?;
    let mount_parts = std::iter::once(PartOfACall {
        write_calls: writes_before_the_first_publish,
        part: PartOfACallKind::OutsideAnyPublish(
            DeletionHistoryWritesOutsideAnyPublish::MountWritesBeforeItsFirstPublish,
        ),
    })
    .chain(publishes)
    .collect();
    let mount_segments = segments_of_one_call(stream, mount_operations, mount_parts)?;
    Ok((
        unmount_segments.into_iter().chain(mount_segments).collect(),
        mounted.current,
    ))
}
