//! 里程碑「第二个事务」增补 2（调度记录 `records/2026-09-24-里程碑二收尾调度.md` 第三节「fsync 失败掉盘与 C542」
//! 「fsync 失败掉盘的核查」两行）：两块盘的池上，一块盘的 fsync（写屏障、FUA、系统配置轮换写）失败之后，池不许退成只剩一块盘可用。
//!
//! 前半是核查时的用例（原样在 `research/prompts/fsync-drop-investigation/fsync_drop_investigation.rs`）搬进来：
//! - 写路径那一半：36 格 fsync 失败历史（六个注入点 × 三种持续档 × 故障前重开与否），每一格判另发一次被冻结拦住、一次性故障
//!   原样重发做成、故障消失之后可写挂载做成、最后那一版每个单元两块盘各一份、池级 checker 不红、冷重开读回最后确认的一版
//!   （「屏障报错丢脏页」那一档的镜像同判）；
//! - 挂载那一半：盘 1 缺席、盘 1 是空盘、盘 1 停在旧状态三格。可写挂载要么拒、要么挂上之后这一版每个单元两块盘各一份；
//!   盘 1 缺席那一格按 w 的下限拒（`MountError::WritableDeviceCountBelowTheStripeWidthLowerBound`，可写挂载与回退各一条）。
//!   另有读路径一格：盘 1 每次读都报错时可写挂载拒成哪个成员。
//!
//! 后半钉可写挂载准入的逐盘核（D2（RAID 条带策略） 已定项 13「挂载准入」、已定项 6 `w ≥ 2`；D18（块里携带什么信息） 已定项 11
//! 「可写挂载的顺序」里的「可见」与「前提」里的「错过取号的盘整盘作废、只读到重同步完成」）：空盘、停在旧状态的盘拒可写、
//! 报 `MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`、盘上逐字节不变；只读挂载照旧读得回所选那一版。
//!
//! 设备栈：`BarrierFailureLog`（记下每一次屏障报错时录制流有多长）→ `FaultInjectingBlockDevice`（按计划报错）
//! → `RecordingBlockDevice`（只记真落下去的）→ `SparseBlockDevice`（内存盘，没有缓存）。
//! 冷重开的镜像从录制流重建。「fsync 报错丢脏页」那一档（Linux 缓冲写回失败之后页被标干净、内容留在缓存里、盘上没有）
//! 另建一份镜像：每一次屏障在盘 1 上报错，就把盘 1 自上一次成功刷盘（池屏障，或盘 1 上的 FUA 写）以来的普通写从镜像里拿掉。

use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::io::Write as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes,
    InodeNumber, InstanceGeneration, JournalSequenceNumber, SlotNumber,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::journal::record_offset;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_rollback, mount_writable, DeviceCount, DeviceWithoutTheSelectedVersion, MountError,
    RollbackTarget, SelectedVersionLackingOnDevice, ShadowLedger, STRIPE_WIDTH_LOWER_BOUND,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::records::STATISTIC_ALLOCATED_BYTES;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, readable_roots_with_ring_slots, recover,
    verified_system_configuration_slots, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, resend_the_frozen_publish, warm_up,
    FirstFile, PoolVersion, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
    FIRST_INODE_NUMBER,
};
use singlefs_format::{DATA_UNIT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

const IMAGE_BYTES: u64 = 4 << 30;
const DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);
const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);
const RESEND_ATTEMPTS_WHILE_THE_FAULT_PERSISTS: usize = 3;

/// 屏障报错那一刻录制流的长度，按盘记。
type BarrierFailures = Rc<RefCell<Vec<(DeviceIdentity, usize)>>>;

struct BarrierFailureLog<Inner: BlockDevice> {
    inner: Inner,
    device: DeviceIdentity,
    stream: SharedStream,
    failures: BarrierFailures,
}

impl<Inner: BlockDevice> BlockDevice for BarrierFailureLog<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        let result = self.inner.barrier();
        if result.is_err() {
            self.failures
                .borrow_mut()
                .push((self.device, self.stream.operation_count()));
        }
        result
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

type Device = BarrierFailureLog<FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>>;

struct Rig {
    parameters: MakeFilesystemParameters,
    stream: SharedStream,
    plan: SharedFaultPlan,
    failures: BarrierFailures,
    devices: Vec<(DeviceIdentity, Device)>,
}

fn content_of(version: u64) -> Vec<u8> {
    let salt = usize::try_from(version).expect("小");
    (0..4100usize)
        .map(|index| u8::try_from((index * 7 + salt * 13 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 两块录制着的内存盘上做完 mkfs：交回装置与创世根。
fn rig_after_make_filesystem() -> (Rig, singlefs_core::root_record::RootRecord) {
    let parameters = e142_parameters(512, 512);
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let stream = SharedStream::retaining_contents();
    let plan = SharedFaultPlan::unarmed(geometry);
    let failures: BarrierFailures = Rc::new(RefCell::new(Vec::new()));
    let mut devices: Vec<(DeviceIdentity, Device)> = (0..2u32)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                BarrierFailureLog {
                    inner: FaultInjectingBlockDevice::new(
                        identity,
                        RecordingBlockDevice::with_shared_stream(
                            identity,
                            SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                            stream.clone(),
                        ),
                        plan.clone(),
                    ),
                    device: identity,
                    stream: stream.clone(),
                    failures: failures.clone(),
                },
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    (
        Rig {
            parameters,
            stream,
            plan,
            failures,
            devices,
        },
        genesis.root,
    )
}

/// mkfs → 取号 1 → 暖机 → 第一个文件（txg 3）。
fn build_rig() -> (Rig, PoolAllocator, TransactionOutput, InstanceGeneration) {
    let (mut rig, genesis_root) = rig_after_make_filesystem();
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DEVICE_ZERO, IMAGE_BYTES),
        DeviceFreeMap::new(DEVICE_ONE, IMAGE_BYTES),
    ]);
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
    let (first, instance) = {
        let mut writer = PoolWriter::new(&rig.parameters, rig.devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis_root, instance).expect("暖机");
        let first = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &first_file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个事务");
        (first, instance)
    };
    (rig, allocator, first, instance)
}

fn overwrite(
    rig: &mut Rig,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    content: &[u8],
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let mut writer = PoolWriter::new(&rig.parameters, rig.devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        allocator,
        previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + previous.root.checkpoint_txg.0 + 1,
        },
        instance,
    )
}

fn resend(
    rig: &mut Rig,
    allocator: &mut PoolAllocator,
) -> Result<Option<TransactionOutput>, PublishError> {
    let mut writer = PoolWriter::new(&rig.parameters, rig.devices.as_mut_slice());
    resend_the_frozen_publish(&mut writer, allocator).map(|version| {
        version.map(|version| version.into_file_version().expect("冻结的是带文件的一版"))
    })
}

/// 录制流里盘 1 在「屏障报错丢脏页」那一档下丢掉的普通写的下标。
fn writes_lost_by_failed_barriers(rig: &Rig) -> BTreeSet<usize> {
    let operations = rig.stream.operations();
    let failures = rig.failures.borrow().clone();
    // 屏障报错时盘 0 那一道先成功、先记进流里（写入口按盘号次序发屏障）：那一条只罩盘 0。
    let barriers_covering_only_device_zero: BTreeSet<usize> = failures
        .iter()
        .filter_map(|(device, length)| {
            assert_eq!(*device, DEVICE_ONE, "只在盘 1 上注入");
            let index = length.checked_sub(1)?;
            let operation = &operations[index];
            (operation.kind == RecordedOperationKind::Barrier && operation.device == DEVICE_ZERO)
                .then_some(index)
        })
        .collect();
    let mut lost = BTreeSet::new();
    for (_, length) in &failures {
        let end = if barriers_covering_only_device_zero.contains(&(length - 1)) {
            length - 1
        } else {
            *length
        };
        let flush_point = (0..end)
            .rev()
            .find(|index| {
                let operation = &operations[*index];
                (operation.kind == RecordedOperationKind::Barrier
                    && !barriers_covering_only_device_zero.contains(index))
                    || (operation.kind == RecordedOperationKind::WriteForceUnitAccess
                        && operation.device == DEVICE_ONE)
            })
            .map_or(0, |index| index + 1);
        for (index, operation) in operations.iter().enumerate().take(end).skip(flush_point) {
            if operation.device == DEVICE_ONE
                && matches!(
                    operation.kind,
                    RecordedOperationKind::Write | RecordedOperationKind::WriteZeroes
                )
            {
                lost.insert(index);
            }
        }
    }
    lost
}

fn image_of(rig: &Rig, lost: &BTreeSet<usize>) -> MemoryPool {
    let operations: Vec<_> = rig
        .stream
        .retained_operations()
        .into_iter()
        .enumerate()
        .filter(|(index, _)| !lost.contains(index))
        .map(|(_, operation)| operation)
        .collect();
    let mut image = MemoryPool::with_devices(&[DEVICE_ZERO, DEVICE_ONE], IMAGE_BYTES);
    image.apply(&operations);
    image
}

/// 这一版每个单元在两块盘上各有一份逐字节相同的、池级 checker 不红、冷重开读回的是期望的内容：不成立的逐条列出。
fn problems_on(
    image: &MemoryPool,
    current: &TransactionOutput,
    expected: &[u8],
    label: &str,
) -> Vec<String> {
    let mut problems = Vec::new();
    for unit in &current.units {
        let offset = unit.slot.to_device_offset();
        for device in [DEVICE_ZERO, DEVICE_ONE] {
            if image.devices[&device].read(offset, unit.bytes.len()) != unit.bytes {
                problems.push(format!(
                    "{label}：{:?}@{} 在盘{} 上没有这一份",
                    unit.identity, unit.slot.0, device.0
                ));
            }
        }
    }
    for (invariant, verdict) in check_pool_image(image) {
        if let InvariantVerdict::Violated(_) = verdict {
            problems.push(format!("{label}：池级 checker 判 {invariant} 红"));
        }
    }
    let outcome = recover(image, JournalPolicy::Consult).outcome;
    if !matches!(&outcome, RecoveryOutcome::FileRead { content, .. } if content == expected) {
        problems.push(format!(
            "{label}：冷重开读回的不是最后确认的一版：{outcome:?}"
        ));
    }
    problems
}

/// 一个镜像上两块盘的样子，与（给了的话）这一版每个单元、末条记录在两块盘上在不在。
fn describe(
    parameters: &MakeFilesystemParameters,
    image: &MemoryPool,
    current: Option<&TransactionOutput>,
    expected: Option<(&str, &[u8])>,
) -> String {
    let mut text = String::new();
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let fsid = parameters.filesystem_identifier;
    for device in [DEVICE_ZERO, DEVICE_ONE] {
        let mut slots: Vec<(u64, u32, u64)> =
            verified_system_configuration_slots(image, device, spacing, &fsid)
                .iter()
                .map(|slot| {
                    (
                        slot.quantities.slot_generation,
                        slot.quantities.journal_instance.0,
                        slot.quantities.journal_tail,
                    )
                })
                .collect();
        slots.sort_unstable();
        let _ = write!(text, "    系统配置 盘{}：", device.0);
        for (generation, instance, tail) in slots.iter().rev() {
            let _ = write!(text, "[世代 {generation} 实例 {instance} tail {tail}] ");
        }
        text.push('\n');
    }
    let roots = readable_roots_with_ring_slots(
        image,
        &parameters.region_devices,
        &parameters.geometry,
        &fsid,
    );
    for device in [DEVICE_ZERO, DEVICE_ONE] {
        let mut on_device: Vec<(u64, u32)> = roots
            .iter()
            .filter(|(slot, _)| {
                parameters.region_devices[usize::try_from(slot.region).expect("区域")] == device
            })
            .map(|(_, root)| (root.checkpoint_txg.0, root.instance.0))
            .collect();
        on_device.sort_unstable();
        let _ = writeln!(
            text,
            "    根环 盘{}：{} 条，最新 (实例, txg) = {:?}",
            device.0,
            on_device.len(),
            on_device.last().map(|(txg, instance)| (instance, txg))
        );
    }
    let chosen = choose_system_configuration(image)
        .ok()
        .and_then(|system_configuration| choose_root(image, &system_configuration))
        .map(|root| (root.instance.0, root.checkpoint_txg.0));
    let report = recover(image, JournalPolicy::Consult);
    let recovered = match &report.outcome {
        RecoveryOutcome::FileRead { root, content } => {
            let matches = expected.map_or(String::new(), |(name, bytes)| {
                if content.as_slice() == bytes {
                    format!("，读回 = {name}")
                } else {
                    format!("，读回 ≠ {name}")
                }
            });
            format!("读回文件，根 ({}, {}){matches}", root.0 .0, root.1 .0)
        }
        RecoveryOutcome::NoFile { root } => format!("无文件，根 ({}, {})", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { root, failure } => format!("失败 {root:?} {failure:?}"),
    };
    let _ = writeln!(
        text,
        "    择根 (实例, txg) = {chosen:?}；恢复：{recovered}；施加记录 {}",
        report.journal.prefix_applied
    );
    if let Some(version) = current {
        let mut both = 0;
        let mut only_zero = Vec::new();
        let mut only_one = Vec::new();
        let mut neither = Vec::new();
        for unit in &version.units {
            let offset = unit.slot.to_device_offset();
            let length = unit.bytes.len();
            let unit_is_on =
                |device: DeviceIdentity| image.devices[&device].read(offset, length) == unit.bytes;
            match (unit_is_on(DEVICE_ZERO), unit_is_on(DEVICE_ONE)) {
                (true, true) => both += 1,
                (true, false) => only_zero.push(format!("{:?}@{}", unit.identity, unit.slot.0)),
                (false, true) => only_one.push(format!("{:?}@{}", unit.identity, unit.slot.0)),
                (false, false) => neither.push(format!("{:?}@{}", unit.identity, unit.slot.0)),
            }
        }
        let ring = parameters.geometry.journal_ring_bytes;
        let record_at = record_offset(version.record.counter, ring);
        let record_on = |device: DeviceIdentity| {
            image.devices[&device].read(record_at, version.record_bytes.len())
                == version.record_bytes
        };
        let target = target_for_publish(
            version.root.checkpoint_txg,
            parameters.geometry.root_ring_slots_per_region,
        );
        let root_device = parameters.region_devices[usize::try_from(target.region).expect("区域")];
        let root_bytes = image.devices[&root_device].read(
            slot_offset(target, parameters.geometry.fixed_structure_slot_spacing),
            512,
        );
        let root_readable = singlefs_core::root_record::RootRecord::parse_slot(&root_bytes, &fsid)
            .is_some_and(|root| root == version.root);
        let _ = writeln!(
            text,
            "    这一版 (实例 {}, txg {})：{} 个单元两盘都在 {both}，只在盘0 {:?}，只在盘1 {:?}，两盘都不在 {:?}；末条记录 jsn {} 盘0 {} 盘1 {}；根在盘{} 读得出且相同 {root_readable}",
            version.root.instance.0,
            version.root.checkpoint_txg.0,
            version.units.len(),
            only_zero,
            only_one,
            neither,
            version.record.counter,
            record_on(DEVICE_ZERO),
            record_on(DEVICE_ONE),
            root_device.0,
        );
        let allocated = |device: DeviceIdentity| {
            version
                .allocation_records
                .iter()
                .filter(|record| record.device == device && !record.is_released)
                .count()
        };
        let accounted = |device: DeviceIdentity| {
            version
                .accounting_entries
                .iter()
                .find(|entry| {
                    entry.statistic == STATISTIC_ALLOCATED_BYTES && entry.device == device
                })
                .map(|entry| entry.value)
        };
        let _ = writeln!(
            text,
            "    分配记录（已分配条数）盘0 {} 盘1 {}；记账 已分配字节 盘0 {:?} 盘1 {:?}",
            allocated(DEVICE_ZERO),
            allocated(DEVICE_ONE),
            accounted(DEVICE_ZERO),
            accounted(DEVICE_ONE)
        );
    }
    let violated: Vec<&str> = check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(invariant),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect();
    let _ = writeln!(text, "    池级 checker 判红：{violated:?}");
    text
}

#[derive(Clone, Copy, Debug)]
enum FaultCell {
    UnitWrite,
    BarrierAfterUnits,
    JournalRecordWrite,
    BarrierAfterRecords,
    RootForceUnitAccess,
    SystemConfigurationRotation,
}

#[derive(Clone, Copy, Debug)]
enum FaultDuration {
    OneShot,
    PersistsUntilReopen,
    PersistsAcrossReopen,
}

fn schedule(
    cell: FaultCell,
    duration: FaultDuration,
    parameters: &MakeFilesystemParameters,
    txg: CheckpointTxg,
    counter: u64,
) -> FaultSchedule {
    let occurrence_from = |first_matching_call: u64| match duration {
        FaultDuration::OneShot => FaultOccurrence::TheNthMatchingCall(first_matching_call),
        FaultDuration::PersistsUntilReopen | FaultDuration::PersistsAcrossReopen => {
            FaultOccurrence::EveryMatchingCallFromTheNthOnward(first_matching_call)
        }
    };
    let (fault, placement, occurrence) = match cell {
        // 持续那一档：盘 1 的每一次写都报错（整盘写不进）。
        FaultCell::UnitWrite => (
            InjectedFault::WriteFails,
            FaultPlacement::AnyOffset,
            occurrence_from(1),
        ),
        // 持续那一档：盘 1 的每一次刷盘都报错。
        FaultCell::BarrierAfterUnits => (
            InjectedFault::BarrierFails,
            FaultPlacement::AnyOffset,
            occurrence_from(1),
        ),
        // 持续那一档：这条记录那一槽写不进（落点级）。
        FaultCell::JournalRecordWrite => (
            InjectedFault::WriteFails,
            FaultPlacement::OffsetExactly(record_offset(
                counter,
                parameters.geometry.journal_ring_bytes,
            )),
            occurrence_from(1),
        ),
        FaultCell::BarrierAfterRecords => (
            InjectedFault::BarrierFails,
            FaultPlacement::AnyOffset,
            occurrence_from(2),
        ),
        // 持续那一档：这个 txg 的根槽写不进（落点级）。
        FaultCell::RootForceUnitAccess => (
            InjectedFault::WriteFails,
            FaultPlacement::OffsetExactly(slot_offset(
                target_for_publish(txg, parameters.geometry.root_ring_slots_per_region),
                parameters.geometry.fixed_structure_slot_spacing,
            )),
            occurrence_from(1),
        ),
        // 持续那一档：盘 1 两个系统配置槽都写不进。
        FaultCell::SystemConfigurationRotation => (
            InjectedFault::WriteFails,
            FaultPlacement::OffsetBelow(
                SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
                    * u64::from(parameters.geometry.fixed_structure_slot_spacing),
            ),
            occurrence_from(1),
        ),
    };
    FaultSchedule {
        fault,
        device: FaultDeviceSelector::OnlyDevice(DEVICE_ONE),
        placement,
        counting: FaultCounting::PerDevice,
        occurrence,
    }
}

fn print_uncaptured(text: &str) {
    let mut standard_output = std::io::stdout();
    let _ = standard_output.write_all(text.as_bytes());
    let _ = standard_output.flush();
}

/// 一条历史：（可选）先重开一次，推到下一次发布的根落在盘 1 → 那一次在盘 1 上注入 → 再发一次 → 按持续档重发 / 重开 → 再写两次 → 冷重开。
fn run_scenario(
    cell: FaultCell,
    duration: FaultDuration,
    mount_before_the_fault: bool,
) -> (String, Vec<String>) {
    let mut text = String::new();
    let mut problems: Vec<String> = Vec::new();
    let (mut rig, mut allocator, mut current, mut instance) = build_rig();
    let mut acknowledged_content = first_file_content();
    let mut next_version = 2u64;
    if mount_before_the_fault {
        let mounted = mount_writable(&rig.parameters, &mut rig.devices).expect("故障前的可写挂载");
        allocator = mounted.allocator;
        instance = mounted.output.instance;
        current = mounted.current.into_file_version().expect("带文件");
    }
    while (current.root.checkpoint_txg.0 + 1) % 3 != 1 {
        let content = content_of(next_version);
        next_version += 1;
        current = overwrite(&mut rig, &mut allocator, &current, &content, instance)
            .expect("故障前的覆盖写");
        acknowledged_content = content;
    }
    let fault_txg = CheckpointTxg(current.root.checkpoint_txg.0 + 1);
    let fault_counter = current.record.counter + 1;
    let region = target_for_publish(
        fault_txg,
        rig.parameters.geometry.root_ring_slots_per_region,
    )
    .region;
    assert_eq!(
        rig.parameters.region_devices[usize::try_from(region).expect("区域")],
        DEVICE_ONE
    );
    let _ = writeln!(
        text,
        "━━ {cell:?} × {duration:?}（故障前{}重开；注入在实例 {} 的 txg {} 那次发布，根槽在盘 1）",
        if mount_before_the_fault { "" } else { "没" },
        instance.0,
        fault_txg.0
    );
    rig.plan.arm(schedule(
        cell,
        duration,
        &rig.parameters,
        fault_txg,
        fault_counter,
    ));
    let failed_content = content_of(next_version);
    next_version += 1;
    match overwrite(
        &mut rig,
        &mut allocator,
        &current,
        &failed_content,
        instance,
    ) {
        Ok(version) => {
            let _ = writeln!(
                text,
                "  ✗ 注入没打中：txg {} 发布成功，这一格作废",
                version.root.checkpoint_txg.0
            );
            problems.push("注入没打中".to_string());
            return (text, problems);
        }
        Err(error) => {
            let fired: Vec<String> = rig
                .plan
                .fired()
                .iter()
                .map(|fired| fired.render())
                .collect();
            let _ = writeln!(
                text,
                "  1 发布 txg {} 失败：{error:?}；打中 {fired:?}",
                fault_txg.0
            );
        }
    }
    let _ = write!(
        text,
        "  失败之后（镜像 = 已落下的全部写）：\n{}",
        describe(
            &rig.parameters,
            &image_of(&rig, &BTreeSet::new()),
            Some(&current),
            Some(("上一版", &acknowledged_content))
        )
    );
    let blocked_content = content_of(next_version);
    next_version += 1;
    match overwrite(
        &mut rig,
        &mut allocator,
        &current,
        &blocked_content,
        instance,
    ) {
        Ok(version) => {
            let _ = writeln!(
                text,
                "  2 另发一次：成功 txg {}（冻结没拦住）",
                version.root.checkpoint_txg.0
            );
            problems.push("冻结没拦住另一次发布".to_string());
        }
        Err(error) => {
            let _ = writeln!(text, "  2 另发一次：{error:?}");
            if !matches!(
                error,
                PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
            ) {
                problems.push(format!("另发一次报的不是冻结：{error:?}"));
            }
        }
    }
    let resend_attempts = match duration {
        FaultDuration::OneShot => 1,
        FaultDuration::PersistsUntilReopen | FaultDuration::PersistsAcrossReopen => {
            RESEND_ATTEMPTS_WHILE_THE_FAULT_PERSISTS
        }
    };
    let mut resent = None;
    let mut resend_windows: Vec<(usize, usize)> = Vec::new();
    for attempt in 1..=resend_attempts {
        let window_start = rig.stream.operation_count();
        let resend_result = resend(&mut rig, &mut allocator);
        resend_windows.push((window_start, rig.stream.operation_count()));
        match resend_result {
            Ok(Some(version)) => {
                let _ = writeln!(
                    text,
                    "  3.{attempt} 原样重发：成功，txg {}",
                    version.root.checkpoint_txg.0
                );
                resent = Some(version);
                break;
            }
            Ok(None) => {
                let _ = writeln!(text, "  3.{attempt} 原样重发：没有冻结着的发布");
                break;
            }
            Err(error) => {
                let _ = writeln!(text, "  3.{attempt} 原样重发：{error:?}");
                if matches!(duration, FaultDuration::OneShot) {
                    problems.push(format!("一次性故障，重发却报错：{error:?}"));
                }
            }
        }
    }
    let mut mounts_to_try: Vec<bool> = Vec::new();
    let mut resent_version: Option<TransactionOutput> = None;
    match (duration, resent) {
        (FaultDuration::OneShot, Some(version)) => {
            resent_version = Some(version.clone());
            current = version;
            acknowledged_content.clone_from(&failed_content);
            let _ = write!(
                text,
                "  重发之后：\n{}",
                describe(
                    &rig.parameters,
                    &image_of(&rig, &BTreeSet::new()),
                    Some(&current),
                    Some(("重发那一版", &acknowledged_content))
                )
            );
            for _ in 0..2 {
                let content = content_of(next_version);
                next_version += 1;
                current = overwrite(&mut rig, &mut allocator, &current, &content, instance)
                    .expect("重发之后的覆盖写");
                acknowledged_content = content;
            }
            mounts_to_try.push(false);
        }
        (FaultDuration::OneShot, None) => {
            let _ = writeln!(text, "  ✗ 一次性故障，重发却没做成");
            problems.push("一次性故障，重发没做成".to_string());
            return (text, problems);
        }
        (FaultDuration::PersistsUntilReopen, _) => {
            let _ = write!(
                text,
                "  卡住（进程退出前，冻结的那一版随进程丢掉）：\n{}",
                describe(
                    &rig.parameters,
                    &image_of(&rig, &BTreeSet::new()),
                    Some(&current),
                    Some(("最后确认的一版", &acknowledged_content))
                )
            );
            mounts_to_try.push(false);
        }
        (FaultDuration::PersistsAcrossReopen, _) => {
            mounts_to_try.push(true);
            mounts_to_try.push(false);
        }
    }
    // 进程退出：写入口与分配器（连同冻结着的那一版）都丢掉，同一批盘可写挂载。
    drop(allocator);
    let mut mounted = None;
    for still_faulty in mounts_to_try {
        if !still_faulty {
            rig.plan.disarm();
        }
        let label = if still_faulty {
            "故障还在"
        } else {
            "故障已消失"
        };
        match mount_writable(&rig.parameters, &mut rig.devices) {
            Ok(done) => {
                let _ = writeln!(
                    text,
                    "  4 重开可写挂载（{label}）：成功，实例 {}，所选根 ({}, {})，施加后根 ({}, {})，施加记录 {}，写行 {:?}，现行 txg {}",
                    done.output.instance.0,
                    done.output.chosen_root.instance.0,
                    done.output.chosen_root.checkpoint_txg.0,
                    done.output.effective_root.instance.0,
                    done.output.effective_root.checkpoint_txg.0,
                    done.output.journal.prefix_applied,
                    done.output.rows_written,
                    done.current.root().checkpoint_txg.0
                );
                mounted = Some(done);
                break;
            }
            Err(error) => {
                let _ = writeln!(text, "  4 重开可写挂载（{label}）：{error:?}");
                if !still_faulty {
                    problems.push(format!("故障消失之后可写挂载仍报错：{error:?}"));
                }
                let _ = write!(
                    text,
                    "  挂载失败之后：\n{}",
                    describe(
                        &rig.parameters,
                        &image_of(&rig, &BTreeSet::new()),
                        None,
                        Some(("最后确认的一版", &acknowledged_content))
                    )
                );
            }
        }
    }
    let Some(done) = mounted else {
        let _ = writeln!(text, "  ✗ 故障消失之后仍挂不上");
        problems.push("故障消失之后仍挂不上".to_string());
        return (text, problems);
    };
    let mut allocator_after_the_reopen = done.allocator;
    let instance_after_the_reopen = done.output.instance;
    let mut current_after_the_reopen = done.current.into_file_version().expect("带文件");
    let reopened_into = if current_content_is(&rig, &failed_content) {
        "失败那一版（C381 那一格）"
    } else {
        "失败之前的那一版"
    };
    let _ = writeln!(text, "  重开之后现行的是：{reopened_into}");
    for _ in 0..2 {
        let content = content_of(next_version);
        next_version += 1;
        current_after_the_reopen = overwrite(
            &mut rig,
            &mut allocator_after_the_reopen,
            &current_after_the_reopen,
            &content,
            instance_after_the_reopen,
        )
        .expect("重开之后的覆盖写");
        acknowledged_content = content;
    }
    let _ = write!(
        text,
        "  5 重开之后再写两次、冷重开（镜像 = 已落下的全部写）：\n{}",
        describe(
            &rig.parameters,
            &image_of(&rig, &BTreeSet::new()),
            Some(&current_after_the_reopen),
            Some(("最后确认的一版", &acknowledged_content))
        )
    );
    problems.extend(problems_on(
        &image_of(&rig, &BTreeSet::new()),
        &current_after_the_reopen,
        &acknowledged_content,
        "冷重开",
    ));
    let lost = writes_lost_by_failed_barriers(&rig);
    if !rig.failures.borrow().is_empty() {
        let _ = write!(
            text,
            "  6 同一段历史按「屏障报错丢脏页」建镜像（盘 1 屏障报错 {} 次，丢掉盘 1 的普通写 {} 次，下标 {:?}）：\n{}",
            rig.failures.borrow().len(),
            lost.len(),
            lost,
            describe(
                &rig.parameters,
                &image_of(&rig, &lost),
                Some(&current_after_the_reopen),
                Some(("最后确认的一版", &acknowledged_content))
            )
        );
        problems.extend(problems_on(
            &image_of(&rig, &lost),
            &current_after_the_reopen,
            &acknowledged_content,
            "丢脏页档冷重开",
        ));
        // 判别力：假如重发只补一道屏障、不把字节再写一遍（丢掉的那几处在重发窗口里的重写也拿掉），丢脏页档必须红。
        if matches!(duration, FaultDuration::OneShot) {
            let operations = rig.stream.operations();
            let lost_targets: BTreeSet<(u64, u64)> = lost
                .iter()
                .map(|index| (operations[*index].offset.0, operations[*index].length))
                .collect();
            let mut without_the_rewrite = lost.clone();
            for (start, end) in &resend_windows {
                for (index, operation) in operations.iter().enumerate().take(*end).skip(*start) {
                    if operation.device == DEVICE_ONE
                        && operation.kind == RecordedOperationKind::Write
                        && lost_targets.contains(&(operation.offset.0, operation.length))
                    {
                        without_the_rewrite.insert(index);
                    }
                }
            }
            let resent_by_the_one_shot_fault =
                resent_version.as_ref().expect("一次性故障那一格重发做成了");
            let ring = rig.parameters.geometry.journal_ring_bytes;
            let copies_missing_on_device_one = |image: &MemoryPool| -> Vec<String> {
                let mut missing: Vec<String> = resent_by_the_one_shot_fault
                    .units
                    .iter()
                    .filter(|unit| {
                        image.devices[&DEVICE_ONE]
                            .read(unit.slot.to_device_offset(), unit.bytes.len())
                            != unit.bytes
                    })
                    .map(|unit| format!("{:?}@{}", unit.identity, unit.slot.0))
                    .collect();
                if image.devices[&DEVICE_ONE].read(
                    record_offset(resent_by_the_one_shot_fault.record.counter, ring),
                    resent_by_the_one_shot_fault.record_bytes.len(),
                ) != resent_by_the_one_shot_fault.record_bytes
                {
                    missing.push(format!(
                        "记录 jsn {}",
                        resent_by_the_one_shot_fault.record.counter
                    ));
                }
                missing
            };
            let actual = copies_missing_on_device_one(&image_of(&rig, &lost));
            let counterfactual_image = image_of(&rig, &without_the_rewrite);
            let counterfactual = copies_missing_on_device_one(&counterfactual_image);
            let counterfactual_problems = problems_on(
                &counterfactual_image,
                &current_after_the_reopen,
                &acknowledged_content,
                "反事实",
            );
            let _ = writeln!(
                text,
                "  7 判别力（反事实：重发不重写丢掉的那几处，再多拿掉 {} 次写）：重发那一版（txg {}）盘 1 上缺的份，丢脏页档 {:?}，反事实 {:?}；反事实下最后一版的问题 {:?}",
                without_the_rewrite.len() - lost.len(),
                resent_by_the_one_shot_fault.root.checkpoint_txg.0,
                actual,
                counterfactual,
                counterfactual_problems
            );
            if !actual.is_empty() {
                problems.push(format!("丢脏页档下重发那一版盘 1 上缺 {actual:?}"));
            }
            if counterfactual.is_empty() {
                problems
                    .push("判别力自证没红：丢脏页档分不出「重发重写」与「只补屏障」".to_string());
            }
        }
    }
    (text, problems)
}

fn current_content_is(rig: &Rig, content: &[u8]) -> bool {
    let image = image_of(rig, &BTreeSet::new());
    matches!(
        recover(&image, JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead { content: ref read, .. } if read.as_slice() == content
    )
}

const CELLS: [FaultCell; 6] = [
    FaultCell::UnitWrite,
    FaultCell::BarrierAfterUnits,
    FaultCell::JournalRecordWrite,
    FaultCell::BarrierAfterRecords,
    FaultCell::RootForceUnitAccess,
    FaultCell::SystemConfigurationRotation,
];

/// 每一格判：另发一次被冻结拦住；一次性故障重发做成；故障消失之后可写挂载做成；最后那一版每个单元两块盘各一份、
/// checker 不红、冷重开读回最后确认的一版（丢脏页档同判）；一次性屏障那几格的判别力自证要红。
fn run_matrix(duration: FaultDuration) {
    let mut all_problems = Vec::new();
    let mut cells_run = 0;
    for mount_before_the_fault in [false, true] {
        for cell in CELLS {
            let (text, problems) = run_scenario(cell, duration, mount_before_the_fault);
            cells_run += 1;
            let verdict = if problems.is_empty() {
                "  ⇒ 这一格：没有掉盘\n".to_string()
            } else {
                format!("  ⇒ 这一格的问题：{problems:?}\n")
            };
            print_uncaptured(&format!("{text}{verdict}"));
            all_problems.extend(problems.into_iter().map(|problem| {
                format!("{cell:?}×{duration:?}×重开{mount_before_the_fault}：{problem}")
            }));
        }
    }
    print_uncaptured(&format!(
        "== {duration:?}：跑了 {cells_run} 格，问题 {} 条\n",
        all_problems.len()
    ));
    assert_eq!(cells_run, 12, "六个注入点 × 故障前重开与否");
    assert!(all_problems.is_empty(), "{all_problems:#?}");
}

#[test]
fn fsync_drop_one_shot() {
    run_matrix(FaultDuration::OneShot);
}

#[test]
fn fsync_drop_persists_until_reopen() {
    run_matrix(FaultDuration::PersistsUntilReopen);
}

#[test]
fn fsync_drop_persists_across_reopen() {
    run_matrix(FaultDuration::PersistsAcrossReopen);
}

/// 挂载格（不是 fsync 失败的历史）：挂载怎么对待「交进来的盘少一块 / 盘 1 是空盘 / 盘 1 停在旧状态」。
fn sparse_from(image: &MemoryPool, device: DeviceIdentity) -> SparseBlockDevice {
    let mut sparse = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
    sparse.image = image.devices[&device].clone();
    sparse
}

fn blank_device() -> SparseBlockDevice {
    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512))
}

fn memory_pool_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    let mut image = MemoryPool::with_devices(&[DEVICE_ZERO, DEVICE_ONE], IMAGE_BYTES);
    for (identity, device) in devices {
        image.devices.insert(*identity, device.image.clone());
    }
    image
}

fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_string())
        })
        .unwrap_or_else(|| "（非字符串 panic）".to_string())
}

/// 挂载那一步的结局。
enum AuxiliaryMount {
    Refused(MountError),
    Panicked(String),
    /// 可写挂载做成；带挂载之后这一版里没有两份的单元、写完两次之后池级 checker 判红的不变量。
    MountedWritable {
        units_on_one_device_after_the_mount: Vec<String>,
        violated_after_writing: Vec<&'static str>,
    },
}

fn mount_and_write_on(
    parameters: &MakeFilesystemParameters,
    mut devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    label: &str,
) -> (String, AuxiliaryMount) {
    let mut text = String::new();
    let _ = writeln!(text, "━━ 挂载格：{label}");
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let mut mount_text = String::new();
        match mount_writable(parameters, &mut devices) {
            Err(error) => {
                let _ = writeln!(mount_text, "  可写挂载：{error:?}");
                (mount_text, AuxiliaryMount::Refused(error))
            }
            Ok(done) => {
                let _ = writeln!(
                    mount_text,
                    "  可写挂载：成功，实例 {}，所选根 ({}, {})，现行 txg {}",
                    done.output.instance.0,
                    done.output.chosen_root.instance.0,
                    done.output.chosen_root.checkpoint_txg.0,
                    done.current.root().checkpoint_txg.0
                );
                let mut allocator = done.allocator;
                let mut current = done.current.into_file_version().expect("带文件");
                let after_the_mount = memory_pool_of(&devices);
                let units_on_one_device_after_the_mount: Vec<String> = current
                    .units
                    .iter()
                    .filter(|unit| {
                        [DEVICE_ZERO, DEVICE_ONE].iter().any(|device| {
                            after_the_mount.devices[device]
                                .read(unit.slot.to_device_offset(), unit.bytes.len())
                                != unit.bytes
                        })
                    })
                    .map(|unit| format!("{:?}@{}", unit.identity, unit.slot.0))
                    .collect();
                let _ = write!(
                    mount_text,
                    "  挂载之后：\n{}",
                    describe(parameters, &after_the_mount, Some(&current), None)
                );
                for version in 0..2u64 {
                    let content = content_of(100 + version);
                    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
                    match publish_overwrite(
                        &mut writer,
                        &mut allocator,
                        &current,
                        FirstFile {
                            content: &content,
                            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 999 + version,
                        },
                        done.output.instance,
                    ) {
                        Ok(next) => {
                            let _ = writeln!(
                                mount_text,
                                "  覆盖写：成功 txg {}",
                                next.root.checkpoint_txg.0
                            );
                            current = next;
                        }
                        Err(error) => {
                            let _ = writeln!(mount_text, "  覆盖写：{error:?}");
                            break;
                        }
                    }
                }
                let after_writing = memory_pool_of(&devices);
                let _ = write!(
                    mount_text,
                    "  写完之后：\n{}",
                    describe(parameters, &after_writing, Some(&current), None)
                );
                let violated_after_writing: Vec<&'static str> = check_pool_image(&after_writing)
                    .into_iter()
                    .filter_map(|(invariant, verdict)| match verdict {
                        InvariantVerdict::Violated(_) => Some(invariant),
                        InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
                    })
                    .collect();
                (
                    mount_text,
                    AuxiliaryMount::MountedWritable {
                        units_on_one_device_after_the_mount,
                        violated_after_writing,
                    },
                )
            }
        }
    }));
    match outcome {
        Ok((inner, verdict)) => {
            text.push_str(&inner);
            (text, verdict)
        }
        Err(payload) => {
            let message = panic_text(&*payload);
            let _ = writeln!(text, "  panic：{message}");
            (text, AuxiliaryMount::Panicked(message))
        }
    }
}

/// 第一个文件（txg 3）→ 覆盖写 txg 4、txg 5 之后的池：整池的镜像、「盘 1 停在 txg 3 之后」的镜像，与三版各自写出的东西。
struct PoolAtTxgFiveWithDeviceOneStaleAtTxgThree {
    parameters: MakeFilesystemParameters,
    full: MemoryPool,
    stale: MemoryPool,
    /// txg 3、4、5 三版，按先后。
    versions: [TransactionOutput; 3],
    /// 整段历史的录制流（每一次写的内容都留着）。
    stream: SharedStream,
}

fn pool_at_txg_five_with_device_one_stale_at_txg_three() -> PoolAtTxgFiveWithDeviceOneStaleAtTxgThree
{
    let (mut rig, mut allocator, first, instance) = build_rig();
    let stale_point = rig.stream.operation_count();
    let second =
        overwrite(&mut rig, &mut allocator, &first, &content_of(2), instance).expect("txg 4");
    let third =
        overwrite(&mut rig, &mut allocator, &second, &content_of(3), instance).expect("txg 5");
    assert_eq!(third.root.checkpoint_txg, CheckpointTxg(5));
    let full = image_of(&rig, &BTreeSet::new());
    let mut stale = MemoryPool::with_devices(&[DEVICE_ZERO, DEVICE_ONE], IMAGE_BYTES);
    stale.apply(&rig.stream.retained_operations()[..stale_point]);
    PoolAtTxgFiveWithDeviceOneStaleAtTxgThree {
        parameters: rig.parameters,
        full,
        stale,
        versions: [first, second, third],
        stream: rig.stream,
    }
}

/// 数自己被读了几次的盘：读写原样转给内存盘。钉「交进来的盘数不够就在读任何一块盘之前拒」。
struct ReadCountingDevice {
    inner: SparseBlockDevice,
    reads: Cell<u64>,
}

impl BlockDevice for ReadCountingDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.reads.set(self.reads.get() + 1);
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 只把盘 0（txg 5 那一版的整池镜像上的那一块）交进来，外面包一层数读的。
fn only_device_zero_counting_reads(
    pool: &PoolAtTxgFiveWithDeviceOneStaleAtTxgThree,
) -> Vec<(DeviceIdentity, ReadCountingDevice)> {
    vec![(
        DEVICE_ZERO,
        ReadCountingDevice {
            inner: sparse_from(&pool.full, DEVICE_ZERO),
            reads: Cell::new(0),
        },
    )]
}

/// 交进来的盘少于 w 的下限、在读任何一块盘之前被拒：成员是「可写设备数低于 w 的下限」，带交进来的 1 块与下限 2；
/// 那块盘一次都没被读、逐字节不变。
fn assert_refused_below_the_stripe_width_lower_bound_before_any_read(
    error: &MountError,
    only_device_zero: &[(DeviceIdentity, ReadCountingDevice)],
    before: &SparseDevice,
    what: &str,
) {
    let MountError::WritableDeviceCountBelowTheStripeWidthLowerBound {
        devices_handed_in,
        stripe_width_lower_bound,
    } = error
    else {
        panic!("{what}：该报 WritableDeviceCountBelowTheStripeWidthLowerBound，实际 {error:?}")
    };
    assert_eq!(
        (*devices_handed_in, *stripe_width_lower_bound),
        (DeviceCount(1), DeviceCount(2)),
        "{what}：交进来 1 块盘，w 的下限是 2（D2（RAID 条带策略） 已定项 6「`w ≥ 2` 是硬下界」）"
    );
    assert_eq!(
        STRIPE_WIDTH_LOWER_BOUND,
        DeviceCount(2),
        "第一版两块盘镜像，w 的下限是 2"
    );
    let [(device, counting)] = only_device_zero else {
        panic!("{what}：只交了盘 0")
    };
    assert_eq!(*device, DEVICE_ZERO);
    assert_eq!(
        counting.reads.get(),
        0,
        "{what}：拒在选系统配置、恢复、重建与读分配记录树之前，盘 0 一次都没被读"
    );
    assert_eq!(
        &counting.inner.image, before,
        "{what}：拒在任何写之前，盘 0 逐字节不变"
    );
}

/// 盘 1 缺席（只把盘 0 交进来）：可写挂载在读任何一块盘之前拒掉，报「可写设备数低于 w 的下限」
/// （D2（RAID 条带策略） 已定项 13 挂载准入的第一条合取、已定项 6 的 `w ≥ 2`；D18（块里携带什么信息） 已定项 11「可写挂载的顺序」先判这一条），
/// 不靠分配记录树按一块盘的几何重建碰巧报 I-1.1。对照：同一个池两块盘都交，可写挂载照旧做成、接在 txg 5 那一版后面。
#[test]
fn mount_writable_with_only_device_zero_is_refused() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let mut only_device_zero = only_device_zero_counting_reads(&pool);
    let before = pool.full.devices[&DEVICE_ZERO].clone();
    let refusal = mount_writable(&pool.parameters, &mut only_device_zero)
        .expect_err("盘 1 缺席时可写挂载必须拒绝，不许单盘可写");
    assert_refused_below_the_stripe_width_lower_bound_before_any_read(
        &refusal,
        &only_device_zero,
        &before,
        "只把盘 0 交给可写挂载",
    );

    let mut both_devices = vec![
        (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
        (DEVICE_ONE, sparse_from(&pool.full, DEVICE_ONE)),
    ];
    let mounted = mount_writable(&pool.parameters, &mut both_devices)
        .unwrap_or_else(|error| panic!("两块盘都交的健康池，可写挂载照旧做成，实际 {error:?}"));
    assert_eq!(
        mounted.output.effective_root.checkpoint_txg,
        CheckpointTxg(5),
        "两块盘都交：接在 txg 5 那一版后面"
    );
}

/// 回退同一道判：只把盘 0 交给回退（目标 (1, 3)），在读任何一块盘之前拒掉、报同一个成员，盘 0 一次都没被读、逐字节不变。
/// 对照：同一个池两块盘都交，回退到 (1, 3) 照旧做成。
#[test]
fn mount_rollback_with_only_device_zero_is_refused() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let mut only_device_zero = only_device_zero_counting_reads(&pool);
    let before = pool.full.devices[&DEVICE_ZERO].clone();
    let refusal = mount_rollback(
        &pool.parameters,
        &mut only_device_zero,
        version_key(1, 3),
        ShadowLedger::On,
    )
    .expect_err("盘 1 缺席时回退必须拒绝，不许单盘可写");
    assert_refused_below_the_stripe_width_lower_bound_before_any_read(
        &refusal,
        &only_device_zero,
        &before,
        "只把盘 0 交给回退",
    );

    let mut both_devices = vec![
        (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
        (DEVICE_ONE, sparse_from(&pool.full, DEVICE_ONE)),
    ];
    let rolled_back = mount_rollback(
        &pool.parameters,
        &mut both_devices,
        version_key(1, 3),
        ShadowLedger::On,
    )
    .unwrap_or_else(|error| panic!("两块盘都交的健康池，回退到 (1, 3) 照旧做成，实际 {error:?}"));
    assert_eq!(
        version_key(
            rolled_back.output.effective_root.instance.0,
            rolled_back.output.effective_root.checkpoint_txg.0
        ),
        version_key(1, 3),
        "两块盘都交：新实例接在 R_old = (1, 3) 那一版后面"
    );
}

fn assert_refused_or_both_copies(verdict: &AuxiliaryMount, what: &str) {
    match verdict {
        AuxiliaryMount::Refused(error) => {
            print_uncaptured(&format!("  {what}：可写挂载拒了（{error:?}）\n"));
        }
        AuxiliaryMount::Panicked(message) => panic!("{what}：可写挂载 panic 了：{message}"),
        AuxiliaryMount::MountedWritable {
            units_on_one_device_after_the_mount,
            violated_after_writing,
        } => {
            assert!(
                units_on_one_device_after_the_mount.is_empty() && violated_after_writing.is_empty(),
                "{what}：可写挂载做成了，而这一版有单元只剩一份 {units_on_one_device_after_the_mount:?}，写两次之后池级 checker 判红 {violated_after_writing:?}——盘 1 被当成冗余的一半接着写，它缺的那几份没人补"
            );
        }
    }
}

/// 盘 1 换成一块空盘：可写挂载要么拒，要么挂上之后这一版每个单元两块盘各一份（改之前：挂上、四个单元只在盘 0）。
#[test]
fn mount_writable_with_a_blank_device_one_refuses_or_keeps_two_copies() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let (text, verdict) = mount_and_write_on(
        &pool.parameters,
        vec![
            (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
            (DEVICE_ONE, blank_device()),
        ],
        "盘 1 换成一块空盘（全 0）",
    );
    print_uncaptured(&text);
    assert_refused_or_both_copies(&verdict, "盘 1 是空盘");
}

/// 盘 1 停在 txg 3 之后、盘 0 到了 txg 5：同判（改之前同样挂上、四个单元只在盘 0）。
#[test]
fn mount_writable_with_a_stale_device_one_refuses_or_keeps_two_copies() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let (text, verdict) = mount_and_write_on(
        &pool.parameters,
        vec![
            (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
            (DEVICE_ONE, sparse_from(&pool.stale, DEVICE_ONE)),
        ],
        "盘 1 停在第一个文件（txg 3）之后，盘 0 到了 txg 5",
    );
    print_uncaptured(&text);
    assert_refused_or_both_copies(&verdict, "盘 1 停在旧状态");
}

// ─── 可写挂载准入的逐盘核：拒的时候报什么、只读挂载读什么、哪些盘不拒 ───

/// 可写挂载被「有盘不带所选那一版」拒掉时交回的三样（`MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`）。
struct RefusalByDevicesWithoutTheSelectedVersion {
    selected_version: RollbackTarget,
    selected_version_journal_position: JournalSequenceNumber,
    devices: Vec<DeviceWithoutTheSelectedVersion>,
}

fn refusal_by_devices_without_the_selected_version(
    error: &MountError,
    what: &str,
) -> RefusalByDevicesWithoutTheSelectedVersion {
    let MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion {
        selected_version,
        selected_version_journal_position,
        devices,
    } = error
    else {
        panic!(
            "{what}：该报 WritableMountRefusedByDevicesWithoutTheSelectedVersion，实际 {error:?}"
        )
    };
    RefusalByDevicesWithoutTheSelectedVersion {
        selected_version: *selected_version,
        selected_version_journal_position: *selected_version_journal_position,
        devices: devices.clone(),
    }
}

fn version_key(instance: u32, txg: u64) -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(txg),
    }
}

fn journal_position(instance: u32, counter: u64) -> JournalSequenceNumber {
    JournalSequenceNumber {
        instance_generation: InstanceGeneration(instance),
        counter,
    }
}

/// txg 5 那一版里 txg 3 那一版没有的单元（txg 4、txg 5 写出的；停在 txg 3 之后的盘 1 一个都没收到），按角色与落点排好。
/// 从进程里两次发布交回的 `units` 算，不读盘。
fn units_written_after_txg_three(
    pool: &PoolAtTxgFiveWithDeviceOneStaleAtTxgThree,
) -> Vec<(TransactionUnit, SlotNumber)> {
    let [first, _, third] = &pool.versions;
    let mut written_after: Vec<(TransactionUnit, SlotNumber)> = third
        .units
        .iter()
        .filter(|unit| !first.units.contains(unit))
        .map(|unit| (unit.identity, unit.slot))
        .collect();
    written_after.sort_unstable();
    let roles: BTreeSet<TransactionUnit> = written_after.iter().map(|(role, _)| *role).collect();
    for role in [
        TransactionUnit::Data(DataUnitIndexInFile::FIRST),
        TransactionUnit::ExtentRoot,
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(0)),
        TransactionUnit::InodeRoot,
        TransactionUnit::TreeTable,
    ] {
        assert!(
            roles.contains(&role),
            "覆盖写每次都重写 {role:?}：txg 5 那一版的这一份不是 txg 3 那一版的"
        );
    }
    written_after
}

fn newest_system_configuration_journal_tail_on(
    parameters: &MakeFilesystemParameters,
    image: &MemoryPool,
    device: DeviceIdentity,
) -> Option<JournalSequenceNumber> {
    verified_system_configuration_slots(
        image,
        device,
        u64::from(parameters.geometry.fixed_structure_slot_spacing),
        &parameters.filesystem_identifier,
    )
    .into_iter()
    .max_by_key(|slot| slot.quantities.slot_generation)
    .map(|slot| {
        journal_position(
            slot.quantities.journal_instance.0,
            slot.quantities.journal_tail,
        )
    })
}

/// 空盘在盘 1（盘 0 带着 txg 5 那一版）或在盘 0（盘 1 带着的最新一版是 txg 4：txg 5 的根在盘 0 上，它那条记录的点名单元
/// 盘 0 那一份验不过、施加不了）：可写挂载在取号之前拒掉，报新成员、点名那块空盘「没有自证过的系统配置」，
/// 带出所选那一版与它那次发布末条记录的 jsn；两块盘逐字节不变。
#[test]
fn blank_device_refuses_the_writable_mount_by_name_before_any_write() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let [_, second, third] = &pool.versions;
    assert_eq!(
        (second.record.counter, third.record.counter),
        (4, 5),
        "txg 4、txg 5 各一条记录"
    );
    let cases = [
        (
            "盘 1 是空盘",
            DEVICE_ONE,
            vec![
                (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
                (DEVICE_ONE, blank_device()),
            ],
            version_key(1, 5),
            journal_position(1, 5),
        ),
        (
            "盘 0 是空盘",
            DEVICE_ZERO,
            vec![
                (DEVICE_ZERO, blank_device()),
                (DEVICE_ONE, sparse_from(&pool.full, DEVICE_ONE)),
            ],
            version_key(1, 4),
            journal_position(1, 4),
        ),
    ];
    for (what, blank, mut devices, expected_version, expected_position) in cases {
        let before = memory_pool_of(&devices);
        let error =
            mount_writable(&pool.parameters, &mut devices).expect_err("有一块空盘，可写挂载要拒");
        let refusal = refusal_by_devices_without_the_selected_version(&error, what);
        assert_eq!(
            refusal.selected_version, expected_version,
            "{what}：所选那一版"
        );
        assert_eq!(
            refusal.selected_version_journal_position, expected_position,
            "{what}：所选那一版那次发布的末条记录"
        );
        assert_eq!(
            refusal.devices,
            vec![DeviceWithoutTheSelectedVersion {
                device: blank,
                lacking: SelectedVersionLackingOnDevice::NoSelfVerifiedSystemConfiguration,
            }],
            "{what}：点名那块空盘，缺的是自证过的系统配置"
        );
        assert_eq!(
            memory_pool_of(&devices),
            before,
            "{what}：拒在取号之前，两块盘逐字节不变"
        );
    }
}

/// 盘 1 停在 txg 3 之后、盘 0 到了 txg 5：可写挂载在取号之前拒掉，点名盘 1——它最新的系统配置停在 txg 3 那次发布的末条记录
/// (1, 3)，落后于所选那一版 (1, 5)；缺的单元正是 txg 4、txg 5 写出的那几个（从进程里的两次发布算，不读盘）。两块盘逐字节不变。
#[test]
fn stale_device_one_refuses_the_writable_mount_naming_every_unit_it_misses_before_any_write() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let expected_missing = units_written_after_txg_three(&pool);
    let mut devices = vec![
        (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
        (DEVICE_ONE, sparse_from(&pool.stale, DEVICE_ONE)),
    ];
    let before = memory_pool_of(&devices);
    let error =
        mount_writable(&pool.parameters, &mut devices).expect_err("盘 1 停在旧状态，可写挂载要拒");
    let refusal = refusal_by_devices_without_the_selected_version(&error, "盘 1 停在旧状态");
    assert_eq!(refusal.selected_version, version_key(1, 5));
    assert_eq!(
        refusal.selected_version_journal_position,
        journal_position(1, 5)
    );
    let [DeviceWithoutTheSelectedVersion {
        device,
        lacking:
            SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits {
                newest_system_configuration_journal_tail,
                units_missing,
            },
    }] = refusal.devices.as_slice()
    else {
        panic!("只点名盘 1、缺的是 txg 4 之后的单元：{:?}", refusal.devices)
    };
    assert_eq!(*device, DEVICE_ONE);
    assert_eq!(
        *newest_system_configuration_journal_tail,
        journal_position(1, 3),
        "盘 1 最新的系统配置是 txg 3 那次发布轮换写的"
    );
    let mut missing_sorted = units_missing.clone();
    missing_sorted.sort_unstable();
    assert_eq!(
        missing_sorted, expected_missing,
        "缺的就是 txg 4、txg 5 写出、txg 5 那一版还引用着的那几个单元"
    );
    assert_eq!(
        memory_pool_of(&devices),
        before,
        "拒在取号之前，两块盘逐字节不变"
    );
}

/// 只读挂载照旧：空盘在盘 1、盘 1 停在旧状态、空盘在盘 0 三种池上，只读挂载做成、读回所选那一版的文件。
/// 盘 0 是空盘那一格，每个单元的第一条位置条目在盘 0 上验不过，读到的是盘 1 上那一份——带着这一版的那块盘。
#[test]
fn read_only_mount_reads_the_selected_version_from_the_device_that_carries_it() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let mut device_one_blank = pool.full.clone();
    device_one_blank
        .devices
        .insert(DEVICE_ONE, SparseDevice::default());
    let mut device_one_stale = pool.full.clone();
    device_one_stale
        .devices
        .insert(DEVICE_ONE, pool.stale.devices[&DEVICE_ONE].clone());
    let mut device_zero_blank = pool.full.clone();
    device_zero_blank
        .devices
        .insert(DEVICE_ZERO, SparseDevice::default());
    let cases = [
        (
            "盘 1 是空盘",
            device_one_blank,
            version_key(1, 5),
            content_of(3),
        ),
        (
            "盘 1 停在旧状态",
            device_one_stale,
            version_key(1, 5),
            content_of(3),
        ),
        (
            "盘 0 是空盘",
            device_zero_blank,
            version_key(1, 4),
            content_of(2),
        ),
    ];
    for (what, image, expected_version, expected_content) in cases {
        let mounted = mount_read_only(&image)
            .unwrap_or_else(|failure| panic!("{what}：只读挂载要做成，实际 {failure:?}"));
        assert_eq!(
            version_key(
                mounted.effective_root.instance.0,
                mounted.effective_root.checkpoint_txg.0
            ),
            expected_version,
            "{what}：只读挂载沿所选那一版打开"
        );
        let file = mounted
            .mounted
            .open_file(&image, InodeNumber(FIRST_INODE_NUMBER))
            .unwrap_or_else(|failure| panic!("{what}：文件要打得开，实际 {failure:?}"));
        let read = file
            .read_at(
                &image,
                FileOffsetInBytes(0),
                u64::try_from(expected_content.len()).expect("4100"),
            )
            .unwrap_or_else(|failure| panic!("{what}：文件要读得出，实际 {failure:?}"));
        assert_eq!(read.bytes, expected_content, "{what}：读回所选那一版的内容");
    }
}

/// 整池到了 txg 5，只把盘 1 上最后那一次系统配置写（txg 5 那次发布的轮换）拿掉：盘 1 的系统配置落后一格、单元一个不缺——
/// 发布的根落盘之后、轮换之前崩了的样子。
fn full_image_without_the_last_system_configuration_write_on_device_one(
    pool: &PoolAtTxgFiveWithDeviceOneStaleAtTxgThree,
) -> MemoryPool {
    let slots_end = SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
        * u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
    let operations = pool.stream.retained_operations();
    let last_system_configuration_write_on_device_one = operations
        .iter()
        .rposition(|retained| {
            retained.operation.device == DEVICE_ONE
                && retained.operation.kind == RecordedOperationKind::Write
                && retained.operation.offset.0 < slots_end
        })
        .expect("盘 1 上写过系统配置槽");
    let kept: Vec<_> = operations
        .into_iter()
        .enumerate()
        .filter(|(index, _)| *index != last_system_configuration_write_on_device_one)
        .map(|(_, retained)| retained)
        .collect();
    let mut image = MemoryPool::with_devices(&[DEVICE_ZERO, DEVICE_ONE], IMAGE_BYTES);
    image.apply(&kept);
    image
}

/// 盘 1 只是系统配置落后一格（(1, 4) 对所选那一版的 (1, 5)）、单元一个不缺：不拒，可写挂载做成，挂上之后这一版每个单元两块盘各一份、
/// 写两次之后池级 checker 不红。钉住「只落后不算不带」。
#[test]
fn device_one_behind_only_by_the_rotation_it_missed_still_mounts_writable_with_two_copies() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let image = full_image_without_the_last_system_configuration_write_on_device_one(&pool);
    assert_eq!(
        newest_system_configuration_journal_tail_on(&pool.parameters, &image, DEVICE_ONE),
        Some(journal_position(1, 4)),
        "盘 1 的系统配置落后于所选那一版的 (1, 5)：逐盘核会去读它的单元"
    );
    let (text, verdict) = mount_and_write_on(
        &pool.parameters,
        vec![
            (DEVICE_ZERO, sparse_from(&image, DEVICE_ZERO)),
            (DEVICE_ONE, sparse_from(&image, DEVICE_ONE)),
        ],
        "盘 1 少了 txg 5 那次系统配置轮换，单元都在",
    );
    print_uncaptured(&text);
    let AuxiliaryMount::MountedWritable {
        units_on_one_device_after_the_mount,
        violated_after_writing,
    } = verdict
    else {
        panic!("只落后、单元都在的盘不许拒可写：{text}")
    };
    assert_eq!(
        (units_on_one_device_after_the_mount, violated_after_writing),
        (Vec::<String>::new(), Vec::<&str>::new()),
        "挂上之后这一版每个单元两块盘各一份，写两次之后池级 checker 不红"
    );
}

/// 盘 1 的系统配置跟得上所选那一版（(1, 5)），只是 txg 5 那一版的数据单元在盘 1 上那一份坏了一个字节：不拒，可写挂载做成。
/// 钉住「单份坏不算不带」：不是空盘、也没有停在旧状态，条款不让它作废（这一格交主 agent 定，见报告）。
#[test]
fn current_device_one_with_one_corrupted_copy_still_mounts_writable() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let [_, _, third] = &pool.versions;
    let data_location_on_device_one = third.data_pointers[0]
        .locations
        .iter()
        .find(|location| location.device == DEVICE_ONE)
        .expect("数据单元两条位置条目，一条在盘 1");
    let offset = data_location_on_device_one.slot.to_device_offset();
    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let mut image = pool.full.clone();
    let mut corrupted = image.devices[&DEVICE_ONE].read(offset, data_unit_bytes);
    corrupted[100] ^= 0xFF;
    image
        .devices
        .get_mut(&DEVICE_ONE)
        .expect("池里有盘 1")
        .write(offset, &corrupted);
    assert_eq!(
        newest_system_configuration_journal_tail_on(&pool.parameters, &image, DEVICE_ONE),
        Some(journal_position(1, 5)),
        "盘 1 的系统配置跟得上所选那一版"
    );
    let mut devices = vec![
        (DEVICE_ZERO, sparse_from(&image, DEVICE_ZERO)),
        (DEVICE_ONE, sparse_from(&image, DEVICE_ONE)),
    ];
    let mounted = mount_writable(&pool.parameters, &mut devices).unwrap_or_else(|error| {
        panic!("跟得上这一版、只坏了一份的盘，可写挂载照样做成，实际 {error:?}")
    });
    assert_eq!(
        mounted.output.effective_root.checkpoint_txg,
        CheckpointTxg(5),
        "接在 txg 5 那一版后面"
    );
}

/// 不接受零长度读的盘（有的块层把长度 0 当成不是物理块整数倍的请求拒掉）：别的读写原样转给内存盘。
struct ZeroLengthReadRefusingDevice {
    inner: SparseBlockDevice,
}

impl BlockDevice for ZeroLengthReadRefusingDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        if buffer.is_empty() {
            return Err(BlockDeviceError::Unaligned {
                offset,
                length: 0,
                physical_block_size: self.inner.probe_physical_block_size().0,
            });
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 盘 1 只是系统配置落后一格，而 txg 5 那一版的数据单元两块盘上都读不出（D19（块指针的结构与宽度预算） 已定项 5 的 N2：
/// 重建照抄它的位置项、不读内容，挂载照常）：那一项没有验得过的一份可比，逐盘核跳过它，可写挂载做成。
/// 两块盘都不接受零长度读：逐盘核要是拿那一项空的字节去读盘比，读就报错、盘 1 被记成缺它。
#[test]
fn device_one_behind_by_the_rotation_with_the_data_unit_unreadable_everywhere_still_mounts_writable(
) {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let [_, _, third] = &pool.versions;
    let mut image = full_image_without_the_last_system_configuration_write_on_device_one(&pool);
    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    for location in &third.data_pointers[0].locations {
        image
            .devices
            .get_mut(&location.device)
            .expect("池里有这块盘")
            .write(
                location.slot.to_device_offset(),
                &vec![0u8; data_unit_bytes],
            );
    }
    assert_eq!(
        newest_system_configuration_journal_tail_on(&pool.parameters, &image, DEVICE_ONE),
        Some(journal_position(1, 4)),
        "盘 1 的系统配置落后：逐盘核会去读它的单元"
    );
    let mut devices: Vec<(DeviceIdentity, ZeroLengthReadRefusingDevice)> =
        [DEVICE_ZERO, DEVICE_ONE]
            .into_iter()
            .map(|device| {
                (
                    device,
                    ZeroLengthReadRefusingDevice {
                        inner: sparse_from(&image, device),
                    },
                )
            })
            .collect();
    let mounted = mount_writable(&pool.parameters, &mut devices).unwrap_or_else(|error| {
        panic!("数据单元哪块盘上都读不出，不是哪一块盘缺它：可写挂载照常做成，实际 {error:?}")
    });
    let current = mounted.current.file_version().expect("带文件");
    assert!(
        current
            .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
            .bytes
            .is_empty(),
        "那个数据单元照抄位置项、不读内容"
    );
}

/// 树表 0 条的一版：mkfs → 第一次可写挂载（取号 1、零单元写行、暖机）→ 盘 1 停在这里 → 第二次可写挂载（取号 2、给实例 1 写行：
/// 重写实例表、写出分配记录树）。盘 1 停在旧状态时第三次可写挂载拒掉，点名盘 1，缺的正是第二次挂载写行那次重写的那几个单元。
#[test]
fn device_one_stale_on_the_version_without_file_refuses_the_writable_mount_naming_what_the_row_publish_rewrote(
) {
    let (mut rig, _genesis_root) = rig_after_make_filesystem();
    mount_writable(&rig.parameters, &mut rig.devices).expect("第一次可写挂载");
    let stale_point = rig.stream.operation_count();
    let second_mount = mount_writable(&rig.parameters, &mut rig.devices).expect("第二次可写挂载");
    assert!(
        second_mount.current.file_version().is_none(),
        "两次挂载之后仍是树表 0 条的一版"
    );
    let PoolVersion::WithoutFile(row_publish) = &second_mount.output.row_publish else {
        panic!("树表 0 条的一版上写行，走只重写实例表的那条发布路径")
    };
    let mut rewritten_by_the_row_publish = row_publish.rewritten.clone();
    rewritten_by_the_row_publish.sort_unstable();
    assert!(
        rewritten_by_the_row_publish.contains(&TransactionUnit::InstanceTable)
            && rewritten_by_the_row_publish.contains(&TransactionUnit::AllocationTree),
        "给实例 1 写行：重写实例表、写出这一版的分配记录树（C512（树表 0 条的一版上被换下的单元记在哪））：{rewritten_by_the_row_publish:?}"
    );
    let full = image_of(&rig, &BTreeSet::new());
    let mut stale = MemoryPool::with_devices(&[DEVICE_ZERO, DEVICE_ONE], IMAGE_BYTES);
    stale.apply(&rig.stream.retained_operations()[..stale_point]);
    let mut devices = vec![
        (DEVICE_ZERO, sparse_from(&full, DEVICE_ZERO)),
        (DEVICE_ONE, sparse_from(&stale, DEVICE_ONE)),
    ];
    let before = memory_pool_of(&devices);
    let error = mount_writable(&rig.parameters, &mut devices)
        .expect_err("盘 1 停在第一次挂载之后，可写挂载要拒");
    let refusal =
        refusal_by_devices_without_the_selected_version(&error, "树表 0 条的一版上盘 1 停在旧状态");
    assert_eq!(
        refusal.selected_version,
        RollbackTarget {
            instance: second_mount.output.instance,
            checkpoint_txg: second_mount.current.root().checkpoint_txg,
        },
        "所选那一版是第二次挂载最后那次暖机的根"
    );
    let [DeviceWithoutTheSelectedVersion {
        device,
        lacking:
            SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits {
                newest_system_configuration_journal_tail,
                units_missing,
            },
    }] = refusal.devices.as_slice()
    else {
        panic!("只点名盘 1：{:?}", refusal.devices)
    };
    assert_eq!(*device, DEVICE_ONE);
    assert_eq!(
        newest_system_configuration_journal_tail.instance_generation,
        InstanceGeneration(1),
        "盘 1 最新的系统配置停在实例 1"
    );
    let mut missing_roles: Vec<TransactionUnit> =
        units_missing.iter().map(|(role, _)| *role).collect();
    missing_roles.sort_unstable();
    assert_eq!(
        missing_roles, rewritten_by_the_row_publish,
        "缺的就是第二次挂载写行那次重写的那几个单元（树表是 mkfs 那一份，盘 1 上有）"
    );
    assert_eq!(
        memory_pool_of(&devices),
        before,
        "拒在取号之前，两块盘逐字节不变"
    );
}

/// 回退也是可写挂载（取号、写行、暖机），同一道逐盘核，所选那一版是 R_old 那一版：
/// 盘 1 是空盘时回退到 (1, 3)（根在盘 0 的区域 0）拒掉、点名盘 1；盘 1 停在 txg 3 之后时回退到最新的 (1, 5)
/// （盘 1 上没有 txg 4 的根，(1, 4) 不在根环里）拒掉、点名盘 1 缺 txg 4 之后的单元。两格都在取号之前，两块盘逐字节不变。
#[test]
fn rollback_onto_the_blank_or_stale_device_one_is_refused_the_same_way_before_any_write() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let expected_missing = units_written_after_txg_three(&pool);

    let mut onto_a_blank_device = vec![
        (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
        (DEVICE_ONE, blank_device()),
    ];
    let before_the_rollback_onto_a_blank_device = memory_pool_of(&onto_a_blank_device);
    let blank_device_error = mount_rollback(
        &pool.parameters,
        &mut onto_a_blank_device,
        version_key(1, 3),
        ShadowLedger::On,
    )
    .expect_err("盘 1 是空盘，回退要拒");
    let blank_device_refusal =
        refusal_by_devices_without_the_selected_version(&blank_device_error, "回退到空盘上");
    assert_eq!(
        blank_device_refusal.selected_version,
        version_key(1, 3),
        "所选那一版是 R_old"
    );
    assert_eq!(
        blank_device_refusal.selected_version_journal_position,
        journal_position(1, 3)
    );
    assert_eq!(
        blank_device_refusal.devices,
        vec![DeviceWithoutTheSelectedVersion {
            device: DEVICE_ONE,
            lacking: SelectedVersionLackingOnDevice::NoSelfVerifiedSystemConfiguration,
        }]
    );
    assert_eq!(
        memory_pool_of(&onto_a_blank_device),
        before_the_rollback_onto_a_blank_device,
        "两块盘逐字节不变"
    );

    let mut onto_a_stale_device = vec![
        (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
        (DEVICE_ONE, sparse_from(&pool.stale, DEVICE_ONE)),
    ];
    let before_the_rollback_onto_a_stale_device = memory_pool_of(&onto_a_stale_device);
    let stale_device_error = mount_rollback(
        &pool.parameters,
        &mut onto_a_stale_device,
        version_key(1, 5),
        ShadowLedger::On,
    )
    .expect_err("盘 1 停在旧状态，回退到它没有的那一版要拒");
    let stale_device_refusal = refusal_by_devices_without_the_selected_version(
        &stale_device_error,
        "回退到停在旧状态的盘上",
    );
    assert_eq!(stale_device_refusal.selected_version, version_key(1, 5));
    assert_eq!(
        stale_device_refusal.selected_version_journal_position,
        journal_position(1, 5)
    );
    let [DeviceWithoutTheSelectedVersion {
        device,
        lacking:
            SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits {
                newest_system_configuration_journal_tail,
                units_missing,
            },
    }] = stale_device_refusal.devices.as_slice()
    else {
        panic!("只点名盘 1：{:?}", stale_device_refusal.devices)
    };
    assert_eq!(
        (*device, *newest_system_configuration_journal_tail),
        (DEVICE_ONE, journal_position(1, 3))
    );
    let mut missing_sorted = units_missing.clone();
    missing_sorted.sort_unstable();
    assert_eq!(missing_sorted, expected_missing);
    assert_eq!(
        memory_pool_of(&onto_a_stale_device),
        before_the_rollback_onto_a_stale_device,
        "两块盘逐字节不变"
    );
}

/// 读路径那一格（不是 fsync 失败；调查文件那条 `mount_writable_while_every_read_of_device_one_fails_is_recorded` 只记不判）：
/// 第一个文件（txg 3）→ 覆盖写 txg 4 之后，盘 1 的每一次读都报错、写照常。交进来的是两块盘，w 的下限那一判不拦；
/// 可写挂载在取号之前拒掉，报 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`，点名盘 1「没有自证过的系统配置」——
/// 两个系统配置槽都读不出，这块盘不「可见」（D18（块里携带什么信息） 已定项 11「可写挂载的顺序」）。一个写、一道屏障都没落下。
/// 所选那一版是 (1, 3)、不是 (1, 4)：txg 4 的根只落在区域 1（盘 1，读不出），txg 4 那条记录点名的单元在盘 1 上那一份验不了、
/// 那次发布不施加（同「盘 0 是空盘」那一格所选的是 (1, 4) 而不是 (1, 5) 的道理）。
#[test]
fn mount_writable_while_every_read_of_device_one_fails_is_refused_naming_device_one() {
    let (mut rig, mut allocator, first, instance) = build_rig();
    let second =
        overwrite(&mut rig, &mut allocator, &first, &content_of(2), instance).expect("txg 4");
    assert_eq!(
        (instance, second.root.checkpoint_txg, second.record.counter),
        (InstanceGeneration(1), CheckpointTxg(4), 4),
        "实例 1 的 txg 4 那一版，末条记录计数器 4"
    );
    drop(allocator);
    rig.plan.arm(FaultSchedule {
        fault: InjectedFault::ReadFails,
        device: FaultDeviceSelector::OnlyDevice(DEVICE_ONE),
        placement: FaultPlacement::AnyOffset,
        counting: FaultCounting::PerDevice,
        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
    });
    let operations_before_the_mount = rig.stream.operation_count();
    let error = mount_writable(&rig.parameters, &mut rig.devices)
        .expect_err("盘 1 每次读都报错，可写挂载要拒");
    rig.plan.disarm();
    let refusal = refusal_by_devices_without_the_selected_version(&error, "盘 1 每次读都报错");
    assert_eq!(
        refusal.selected_version,
        version_key(1, 3),
        "所选那一版是 txg 3 那一版：txg 4 的根只在盘 1 上，它那条记录施加不了"
    );
    assert_eq!(
        refusal.selected_version_journal_position,
        journal_position(1, 3),
        "所选那一版那次发布的末条记录"
    );
    assert_eq!(
        refusal.devices,
        vec![DeviceWithoutTheSelectedVersion {
            device: DEVICE_ONE,
            lacking: SelectedVersionLackingOnDevice::NoSelfVerifiedSystemConfiguration,
        }],
        "点名盘 1：两个系统配置槽都读不出，没有自证过的一份"
    );
    assert_eq!(
        rig.stream.operation_count(),
        operations_before_the_mount,
        "拒在取号之前：录制流里一个写、一道屏障都没多"
    );
}
