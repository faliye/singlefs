//! harness 测试文件 `crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, JournalSequenceNumber,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::record_offset;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::records::STATISTIC_ALLOCATED_BYTES;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, readable_roots_with_ring_slots, recover,
    verified_system_configuration_slots, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, resend_the_frozen_publish, warm_up,
    FirstFile, PoolWriter, PublishError, TransactionOutput,
};
use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::io::Write as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

pub const IMAGE_BYTES: u64 = 4 << 30;

pub const DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);

pub const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);

pub const RESEND_ATTEMPTS_WHILE_THE_FAULT_PERSISTS: usize = 3;

/// 屏障报错那一刻录制流的长度，按盘记。
pub type BarrierFailures = Rc<RefCell<Vec<(DeviceIdentity, usize)>>>;

pub struct BarrierFailureLog<Inner: BlockDevice> {
    pub inner: Inner,
    pub device: DeviceIdentity,
    pub stream: SharedStream,
    pub failures: BarrierFailures,
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

pub type Device =
    BarrierFailureLog<FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>>;

pub struct Rig {
    pub parameters: MakeFilesystemParameters,
    pub stream: SharedStream,
    pub plan: SharedFaultPlan,
    pub failures: BarrierFailures,
    pub devices: Vec<(DeviceIdentity, Device)>,
}

pub fn content_of(version: u64) -> Vec<u8> {
    let salt = usize::try_from(version).expect("小");
    (0..4100usize)
        .map(|index| u8::try_from((index * 7 + salt * 13 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 两块录制着的内存盘上做完 mkfs：交回装置与 mkfs 的输出。
pub fn rig_after_make_filesystem() -> (Rig, singlefs_core::make_filesystem::MakeFilesystemOutput) {
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
        genesis,
    )
}

/// mkfs → 取号 1 → 暖机 → 第一个文件（txg 3）。
pub fn build_rig() -> (Rig, PoolAllocator, TransactionOutput, InstanceGeneration) {
    let (mut rig, genesis) = rig_after_make_filesystem();
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
    // mkfs 同一个进程里接着写的会话装上 mkfs 刚写下的根环（没装的分配器记根就断言，C551）。
    allocator.install_root_ring_occupancy(
        singlefs_core::make_filesystem::root_ring_occupancy_after_make_filesystem(
            &rig.parameters,
            &genesis,
        ),
    );
    let (first, instance) = {
        let mut writer = PoolWriter::new(&rig.parameters, rig.devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
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
        .expect("新池新建文件");
        (first, instance)
    };
    (rig, allocator, first, instance)
}

pub fn overwrite(
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

pub fn resend(
    rig: &mut Rig,
    allocator: &mut PoolAllocator,
) -> Result<Option<TransactionOutput>, PublishError> {
    let mut writer = PoolWriter::new(&rig.parameters, rig.devices.as_mut_slice());
    resend_the_frozen_publish(&mut writer, allocator).map(|version| {
        version.map(|version| version.into_file_version().expect("冻结的是带文件的一版"))
    })
}

/// 录制流里盘 1 在「屏障报错丢脏页」那一档下丢掉的普通写的下标。
pub fn writes_lost_by_failed_barriers(rig: &Rig) -> BTreeSet<usize> {
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

pub fn image_of(rig: &Rig, lost: &BTreeSet<usize>) -> MemoryPool {
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
pub fn problems_on(
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
    let outcome = recover(image, JournalPolicy::Consult).outcome;
    if !matches!(&outcome, RecoveryOutcome::FileRead { content, .. } if content == expected) {
        problems.push(format!(
            "{label}：冷重开读回的不是最后确认的一版：{outcome:?}"
        ));
    }
    problems
}

/// 一个镜像上两块盘的样子，与（给了的话）这一版每个单元、末条记录在两块盘上在不在。
pub fn describe(
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
    text
}

#[derive(Clone, Copy, Debug)]
pub enum FaultCell {
    UnitWrite,
    BarrierAfterUnits,
    JournalRecordWrite,
    BarrierAfterRecords,
    RootForceUnitAccess,
    SystemConfigurationRotation,
}

#[derive(Clone, Copy, Debug)]
pub enum FaultDuration {
    OneShot,
    PersistsUntilReopen,
    PersistsAcrossReopen,
}

pub fn schedule(
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

pub fn print_uncaptured(text: &str) {
    let mut standard_output = std::io::stdout();
    let _ = standard_output.write_all(text.as_bytes());
    let _ = standard_output.flush();
}

/// 一条历史：（可选）先重开一次，推到下一次发布的根落在盘 1 → 那一次在盘 1 上注入 → 再发一次 → 按持续档重发 / 重开 → 再写两次 → 冷重开。
pub fn run_scenario(
    cell: FaultCell,
    duration: FaultDuration,
    mount_before_the_fault: bool,
    images: &mut PoolCheckerImages,
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
    // 冷重开那一张：池级 checker 一条不红
    images.hand_out(
        &format!("{cell:?}×{duration:?}×重开{mount_before_the_fault}：冷重开"),
        || image_of(&rig, &BTreeSet::new()),
        PoolCheckerExpectation::no_invariant_violated(),
    );
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
        // 丢脏页档冷重开那一张：池级 checker 一条不红
        images.hand_out(
            &format!("{cell:?}×{duration:?}×重开{mount_before_the_fault}：丢脏页档冷重开"),
            || image_of(&rig, &lost),
            PoolCheckerExpectation::no_invariant_violated(),
        );
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

pub fn current_content_is(rig: &Rig, content: &[u8]) -> bool {
    let image = image_of(rig, &BTreeSet::new());
    matches!(
        recover(&image, JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead { content: ref read, .. } if read.as_slice() == content
    )
}

pub const CELLS: [FaultCell; 6] = [
    FaultCell::UnitWrite,
    FaultCell::BarrierAfterUnits,
    FaultCell::JournalRecordWrite,
    FaultCell::BarrierAfterRecords,
    FaultCell::RootForceUnitAccess,
    FaultCell::SystemConfigurationRotation,
];

/// 每一格判：另发一次被冻结拦住；一次性故障重发做成；故障消失之后可写挂载做成；最后那一版每个单元两块盘各一份、
/// checker 不红、冷重开读回最后确认的一版（丢脏页档同判）；一次性屏障那几格的判别力自证要红。
pub fn run_matrix(duration: FaultDuration, images: &mut PoolCheckerImages) {
    let mut all_problems = Vec::new();
    let mut cells_run = 0;
    for mount_before_the_fault in [false, true] {
        for cell in CELLS {
            let (text, problems) = run_scenario(cell, duration, mount_before_the_fault, images);
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

pub fn fsync_drop_one_shot(images: &mut PoolCheckerImages) {
    run_matrix(FaultDuration::OneShot, images);
}

pub fn fsync_drop_persists_until_reopen(images: &mut PoolCheckerImages) {
    run_matrix(FaultDuration::PersistsUntilReopen, images);
}

pub fn fsync_drop_persists_across_reopen(images: &mut PoolCheckerImages) {
    run_matrix(FaultDuration::PersistsAcrossReopen, images);
}

/// 挂载格（不是 fsync 失败的历史）：挂载怎么对待「交进来的盘少一块 / 盘 1 是空盘 / 盘 1 停在旧状态」。
pub fn sparse_from(image: &MemoryPool, device: DeviceIdentity) -> SparseBlockDevice {
    let mut sparse = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
    sparse.image = image.devices[&device].clone();
    sparse
}

pub fn blank_device() -> SparseBlockDevice {
    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512))
}

pub fn memory_pool_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    let mut image = MemoryPool::with_devices(&[DEVICE_ZERO, DEVICE_ONE], IMAGE_BYTES);
    for (identity, device) in devices {
        image.devices.insert(*identity, device.image.clone());
    }
    image
}

pub fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
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
pub enum AuxiliaryMount {
    Refused(MountError),
    Panicked(String),
    /// 可写挂载做成；带挂载之后这一版里没有两份的单元、写完两次之后的镜像（池级 checker 那一遍交给 checker 档）。
    MountedWritable {
        units_on_one_device_after_the_mount: Vec<String>,
        image_after_writing: MemoryPool,
    },
}

pub fn mount_and_write_on(
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
                (
                    mount_text,
                    AuxiliaryMount::MountedWritable {
                        units_on_one_device_after_the_mount,
                        image_after_writing: after_writing,
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
pub struct PoolAtTxgFiveWithDeviceOneStaleAtTxgThree {
    pub parameters: MakeFilesystemParameters,
    pub full: MemoryPool,
    pub stale: MemoryPool,
    /// txg 3、4、5 三版，按先后。
    pub versions: [TransactionOutput; 3],
    /// 整段历史的录制流（每一次写的内容都留着）。
    pub stream: SharedStream,
}

pub fn pool_at_txg_five_with_device_one_stale_at_txg_three(
) -> PoolAtTxgFiveWithDeviceOneStaleAtTxgThree {
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

pub fn assert_refused_or_both_copies(
    verdict: &AuxiliaryMount,
    what: &str,
    images: &mut PoolCheckerImages,
) {
    match verdict {
        AuxiliaryMount::Refused(error) => {
            print_uncaptured(&format!("  {what}：可写挂载拒了（{error:?}）\n"));
        }
        AuxiliaryMount::Panicked(message) => panic!("{what}：可写挂载 panic 了：{message}"),
        AuxiliaryMount::MountedWritable {
            units_on_one_device_after_the_mount,
            image_after_writing,
        } => {
            assert!(
                units_on_one_device_after_the_mount.is_empty(),
                "{what}：可写挂载做成了，而这一版有单元只剩一份 {units_on_one_device_after_the_mount:?}——盘 1 被当成冗余的一半接着写，它缺的那几份没人补"
            );
            // 写两次之后池级 checker 一条不红
            images.hand_out(
                &format!("{what}：可写挂载做成、写两次之后"),
                || image_after_writing.clone(),
                PoolCheckerExpectation::no_invariant_violated(),
            );
        }
    }
}

/// 盘 1 换成一块空盘：可写挂载要么拒，要么挂上之后这一版每个单元两块盘各一份（改之前：挂上、四个单元只在盘 0）。
pub fn mount_writable_with_a_blank_device_one_refuses_or_keeps_two_copies(
    images: &mut PoolCheckerImages,
) {
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
    assert_refused_or_both_copies(&verdict, "盘 1 是空盘", images);
}

/// 盘 1 停在 txg 3 之后、盘 0 到了 txg 5：同判（改之前同样挂上、四个单元只在盘 0）。
pub fn mount_writable_with_a_stale_device_one_refuses_or_keeps_two_copies(
    images: &mut PoolCheckerImages,
) {
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
    assert_refused_or_both_copies(&verdict, "盘 1 停在旧状态", images);
}

pub fn journal_position(instance: u32, counter: u64) -> JournalSequenceNumber {
    JournalSequenceNumber {
        instance_generation: InstanceGeneration(instance),
        counter,
    }
}

pub fn newest_system_configuration_journal_tail_on(
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

/// 整池到了 txg 5，只把盘 1 上最后那一次系统配置写（txg 5 那次发布的轮换）拿掉：盘 1 的系统配置落后一格、单元一个不缺——
/// 发布的根落盘之后、轮换之前崩了的样子。
pub fn full_image_without_the_last_system_configuration_write_on_device_one(
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
pub fn device_one_behind_only_by_the_rotation_it_missed_still_mounts_writable_with_two_copies(
    images: &mut PoolCheckerImages,
) {
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
        image_after_writing,
    } = verdict
    else {
        panic!("只落后、单元都在的盘不许拒可写：{text}")
    };
    assert_eq!(
        units_on_one_device_after_the_mount,
        Vec::<String>::new(),
        "挂上之后这一版每个单元两块盘各一份"
    );
    images.hand_out(
        "写两次之后",
        || image_after_writing.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
