//! 攻方原型（m3-root-ring-code-r3，K1-b，不入库）：清扫那次挂载里第一个孤儿那一槽的第 n 次读报设备错（n = 2 落在抹之前那次重读），
//! 录下那次挂载的写，全量枚举（每一段都展开）每个崩溃状态：恢复、多版本 oracle、池级 checker，另照 F5 数删行那一版生效而按删前那一行
//! 判未发布的孤儿还剩几个。建池与 F5 观察者照 checker 档 `tests/crash_points_of_the_instance_table_sweep.rs` 另写一份（那份不是层 0 名字的用例）。
#![allow(clippy::all)]
#[path = "../../singlefs-harness/tests/common_root_ring_holes/mod.rs"]
mod common_root_ring_holes;

use std::cell::Cell;
use std::collections::BTreeSet;
use std::rc::Rc;

use common_root_ring_holes::{
    content_of_the_version, devices_of, fixed_geometry_of_the_ring_hole_pools,
    parameters_of_the_ring_hole_pools, pool_of, recording_of, RingHoleDevices,
    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, JOURNAL_RING_BYTES_OF_THE_RING_HOLE_POOLS,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker_tier::crash::{
    enumerate_layer0_in_state_slices, full_expansion, quick_tier_expansion, Layer0ObserverCounts,
    Layer0Parallelism, Layer0SegmentExpansion, Layer0Tally,
};
use singlefs_checker_tier::crash_injection::{
    materialized_crash_image, publishes_before_the_instance_table_sweep_of_the_mount_output,
    writes_of_the_instance_table_sweep_phases, WritableMountPhase,
};
use singlefs_checker_tier::layer0_progress::Layer0Resume;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table::InstanceRow;
use singlefs_core::instance_table_sweep::{
    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
    SweepPublishPlacements,
};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_instance_table_sweep_configuration, Mounted, ShadowLedger,
};
use singlefs_core::recovery::{instance_table_chain_of_root, RecoveryReport};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, publish_sequential_write, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_core::unit::{data_unit_payload_capacity, unit_filesystem_identifier};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;
/// 实例 2 覆盖写到的 txg：根环 12 槽里只剩 txg 8–19。
const LAST_TXG_BEFORE_THE_SWEEPING_MOUNT: u64 = 19;
/// 崩掉的那次顺序写写几个数据单元（孤儿少，抹头那一段的状态数小）。
const ORPHAN_DATA_UNITS: usize = 2;
/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
const INVARIANTS_JUDGED_AFTER_THE_REMOUNT: [&str; 5] =
    ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
/// 码 1 / 码 2 / 码 3 的类标签、明文头宽与 key 宽、头校验和的位置（D18 已定项 7 / 16 / 17 / 18；这份用例自己的一份，不取实现的常量）。
const UNIT_MAGIC: &[u8; 4] = b"SFSU";
const DATA_UNIT_CLASS: u8 = 1;
const INDEX_NODE_CLASS: u8 = 2;
const PACKED_UNIT_CLASS: u8 = 3;
const DATA_UNIT_HEADER_END: usize = 105;
const PACKED_UNIT_HEADER_END: usize = 107;
const INDEX_NODE_HEADER_END_WITHOUT_THE_KEY_RANGE: usize = 86;
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
const HEADER_CHECKSUM_OFFSET: usize = 10;
const HEADER_CHECKSUM_FIELD_BYTES: usize = 32;

/// 这份用例认出来的一个单元头：类、写序实例、诞生代号、码 1 / 码 3 的事务号、fsid 低 8 字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UnitHeadSeenByTheObserver {
    unit_class: u8,
    writer_instance: u32,
    birth_txg: u64,
    transaction: u64,
    filesystem_identifier: u64,
}

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

/// 魔数、类（定头宽）、头校验和（CRC32C 罩 [0, 头末)，字段 10..42 置零后算，存 10..14、其余 28 字节为零）过就认；不看载荷。
fn unit_head_seen_by_the_observer(bytes: &[u8]) -> Option<UnitHeadSeenByTheObserver> {
    if bytes.len() <= INDEX_NODE_KEY_WIDTH_OFFSET || &bytes[..4] != UNIT_MAGIC {
        return None;
    }
    let key_range_bytes = 2 * usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
    let header_end = match bytes[6] {
        DATA_UNIT_CLASS => DATA_UNIT_HEADER_END,
        INDEX_NODE_CLASS => INDEX_NODE_HEADER_END_WITHOUT_THE_KEY_RANGE + key_range_bytes,
        PACKED_UNIT_CLASS => PACKED_UNIT_HEADER_END,
        _ => return None,
    };
    if bytes.len() < header_end {
        return None;
    }
    let mut covered = bytes[..header_end].to_vec();
    covered[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + HEADER_CHECKSUM_FIELD_BYTES].fill(0);
    let expected = singlefs_checker::crc32_castagnoli_bitwise(&covered).to_le_bytes();
    let stored =
        &bytes[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + HEADER_CHECKSUM_FIELD_BYTES];
    if stored[..4] != expected || stored[4..].iter().any(|byte| *byte != 0) {
        return None;
    }
    let six_bytes_at = |offset: usize| {
        let mut widened = [0u8; 8];
        widened[..6].copy_from_slice(&bytes[offset..offset + 6]);
        u64::from_le_bytes(widened)
    };
    Some(match bytes[6] {
        DATA_UNIT_CLASS => UnitHeadSeenByTheObserver {
            unit_class: DATA_UNIT_CLASS,
            birth_txg: u64_at(bytes, 75),
            filesystem_identifier: u64_at(bytes, 83),
            writer_instance: u32::from_le_bytes(bytes[91..95].try_into().expect("4 字节")),
            transaction: six_bytes_at(95),
        },
        PACKED_UNIT_CLASS => UnitHeadSeenByTheObserver {
            unit_class: PACKED_UNIT_CLASS,
            birth_txg: u64_at(bytes, 73),
            filesystem_identifier: u64_at(bytes, 81),
            writer_instance: u32::from_le_bytes(bytes[93..97].try_into().expect("4 字节")),
            transaction: six_bytes_at(97),
        },
        _ => UnitHeadSeenByTheObserver {
            unit_class: INDEX_NODE_CLASS,
            birth_txg: u64_at(bytes, 52 + key_range_bytes),
            filesystem_identifier: u64_at(bytes, 60 + key_range_bytes),
            writer_instance: u32::from_le_bytes(
                bytes[68 + key_range_bytes..72 + key_range_bytes]
                    .try_into()
                    .expect("4 字节"),
            ),
            transaction: 0,
        },
    })
}

/// 已发布谓词有行那一支：码 1 ⇒ b ≤ T ∨ n ≤ W，码 2 / 3 ⇒ b ≤ T；都不成立就是按这一行未发布。
fn is_unpublished_by_the_row(head: &UnitHeadSeenByTheObserver, row: &InstanceRow) -> bool {
    if head.unit_class == DATA_UNIT_CLASS {
        !(head.birth_txg <= row.selected_root_txg.0
            || head.transaction <= row.applied_transaction_high_water)
    } else {
        head.birth_txg > row.selected_root_txg.0
    }
}

/// 盘上按 `row` 判未发布、写序实例是 `row` 那个实例的本池单元有几个：每块盘按槽升序走写过的槽，认出一个头就按整单元跳（码 1、码 3 两槽，码 2 一槽）。
fn unpublished_units_of_the_row_left_on(image: &MemoryPool, row: &InstanceRow) -> u64 {
    let unit_area_start =
        UnitAreaStart::following_the_journal_ring(JOURNAL_RING_BYTES_OF_THE_RING_HOLE_POOLS)
            .expect("1 MiB 环的单元区起点在段边界上")
            .slot()
            .0;
    let filesystem_identifier =
        unit_filesystem_identifier(&parameters_of_the_ring_hole_pools().filesystem_identifier);
    let sectors_per_slot = SLOT_BYTES / 512;
    let mut left = 0;
    for device in image.devices.values() {
        let written_slots: BTreeSet<u64> = device
            .written_sectors()
            .keys()
            .map(|sector| sector / sectors_per_slot)
            .filter(|slot| *slot >= unit_area_start)
            .collect();
        let mut claimed_until = 0;
        for slot in written_slots {
            if slot < claimed_until {
                continue;
            }
            let bytes = device.read(
                singlefs_core::address::DeviceOffsetInBytes(slot * SLOT_BYTES),
                1024,
            );
            let Some(head) = unit_head_seen_by_the_observer(&bytes) else {
                continue;
            };
            claimed_until = slot
                + if head.unit_class == INDEX_NODE_CLASS {
                    1
                } else {
                    2
                };
            if head.filesystem_identifier == filesystem_identifier
                && head.writer_instance == row.instance.0
                && is_unpublished_by_the_row(&head, row)
            {
                left += 1;
            }
        }
    }
    left
}

fn publish_an_overwrite(
    devices: &mut RingHoleDevices,
    allocator: &mut singlefs_core::allocator::PoolAllocator,
    current: &TransactionOutput,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let parameters = parameters_of_the_ring_hole_pools();
    let salt = current.root.checkpoint_txg.0 + 1;
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        allocator,
        current,
        FirstFile {
            content,
            write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
        },
        instance,
    )
    .expect("覆盖写")
}

/// 清扫那次挂载之前的盘面：实例 1 的孤儿留在盘上、实例 1 的根全部轮出环。
fn pool_with_a_dead_instance() -> MemoryPool {
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
    } = mount_writable(&parameters, &mut devices).expect("实例 1 的可写挂载");
    let instance = output.instance;
    let mut current = {
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
    // 迭代上界：两次覆盖写。
    for _ in 0..2 {
        current = publish_an_overwrite(
            &mut devices,
            &mut allocator,
            &current,
            &content_of_the_version(current.root.checkpoint_txg.0 + 1),
            instance,
        );
    }
    let before = pool_of(&devices);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&before, &stream);
    {
        let mut writer = PoolWriter::new(&parameters, recording.as_mut_slice());
        publish_sequential_write(
            &mut writer,
            &mut allocator.clone(),
            &current,
            FirstFile {
                content: &vec![7u8; ORPHAN_DATA_UNITS * data_unit_payload_capacity()],
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + 100,
            },
            instance,
        )
        .expect("顺序写（录下来、之后截掉）");
    }
    let (writes, _segments) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    let first_record_write = writes
        .iter()
        .position(|write| write.kind == StepKind::JournalRecord)
        .expect("顺序写有 journal 记录写");
    let mut image = before;
    image.apply_writes(&writes[..first_record_write]);
    let mut devices_of_instance_two = devices_of(&image);
    let mounted =
        mount_writable(&parameters, &mut devices_of_instance_two).expect("实例 2 的可写挂载");
    let mut allocator_of_instance_two = mounted.allocator;
    let mut current_of_instance_two = mounted.current.into_file_version().expect("这一版带文件");
    // 迭代上界：覆盖写到 txg 19。
    while current_of_instance_two.root.checkpoint_txg.0 < LAST_TXG_BEFORE_THE_SWEEPING_MOUNT {
        current_of_instance_two = publish_an_overwrite(
            &mut devices_of_instance_two,
            &mut allocator_of_instance_two,
            &current_of_instance_two,
            &content_of_the_version(current_of_instance_two.root.checkpoint_txg.0 + 1),
            InstanceGeneration(2),
        );
    }
    pool_of(&devices_of_instance_two)
}

fn sweeping_on_any_row() -> InstanceTableSweepConfiguration {
    InstanceTableSweepConfiguration {
        switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
        row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
        sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
    }
}

/// 读故障：落在 [offset, offset + length) 的读，按这一段被读的次数从 1 数，第 nth 次报设备错（只那一次）。
struct FailNthRead<Inner: singlefs_core::block_device::BlockDevice> {
    identity: DeviceIdentity,
    inner: Inner,
    target: Option<(DeviceIdentity, u64, u64)>,
    nth: u64,
    reads: Rc<Cell<u64>>,
}

impl<Inner: singlefs_core::block_device::BlockDevice> singlefs_core::block_device::BlockDevice for FailNthRead<Inner> {
    fn read_at(&self, offset: singlefs_core::address::DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        if let Some((device, start, length)) = self.target {
            let len = buffer.len() as u64;
            if device == self.identity && offset.0 < start + length && start < offset.0 + len {
                let n = self.reads.get() + 1;
                self.reads.set(n);
                if n == self.nth {
                    return Err(singlefs_core::block_device::BlockDeviceError::InputOutput(std::io::Error::other("注入的读错")));
                }
            }
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: singlefs_core::address::DeviceOffsetInBytes, bytes: &[u8], durability: singlefs_core::block_device::WriteDurability) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: singlefs_core::address::DeviceOffsetInBytes, length: u64) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 盘上按 `row` 判未发布、写序实例是 row 那个实例的单元落点（与 `unpublished_units_of_the_row_left_on` 同一个走法）。
fn unpublished_placements_of_the_row(image: &MemoryPool, row: &InstanceRow) -> Vec<(DeviceIdentity, u64)> {
    let unit_area_start = UnitAreaStart::following_the_journal_ring(JOURNAL_RING_BYTES_OF_THE_RING_HOLE_POOLS).expect("起点").slot().0;
    let filesystem_identifier = unit_filesystem_identifier(&parameters_of_the_ring_hole_pools().filesystem_identifier);
    let sectors_per_slot = SLOT_BYTES / 512;
    let mut out = Vec::new();
    for (identity, device) in &image.devices {
        let written_slots: BTreeSet<u64> = device.written_sectors().keys().map(|s| s / sectors_per_slot).filter(|s| *s >= unit_area_start).collect();
        let mut claimed_until = 0;
        for slot in written_slots {
            if slot < claimed_until { continue; }
            let bytes = device.read(singlefs_core::address::DeviceOffsetInBytes(slot * SLOT_BYTES), 1024);
            let Some(head) = unit_head_seen_by_the_observer(&bytes) else { continue; };
            claimed_until = slot + if head.unit_class == INDEX_NODE_CLASS { 1 } else { 2 };
            if head.filesystem_identifier == filesystem_identifier && head.writer_instance == row.instance.0 && is_unpublished_by_the_row(&head, row) {
                out.push((*identity, slot));
            }
        }
    }
    out
}

fn main() {
    let nth: u64 = std::env::var("PROBE_NTH").ok().and_then(|s| s.parse().ok()).unwrap_or(2);
    let cap: u64 = std::env::var("PROBE_STATE_CAP").ok().and_then(|s| s.parse().ok()).unwrap_or(1_000_000);
    let base = pool_with_a_dead_instance();
    let row_before = InstanceRow { instance: InstanceGeneration(1), selected_root_txg: CheckpointTxg(5), applied_transaction_high_water: 0 };
    let orphans = unpublished_placements_of_the_row(&base, &row_before);
    println!("ENUM orphans_on_base={orphans:?}");
    let target = if nth == 0 { None } else { orphans.first().map(|(device, slot)| (*device, slot * SLOT_BYTES, 512u64)) };
    let stream = SharedStream::retaining_contents();
    let reads = Rc::new(Cell::new(0u64));
    let mut devices: Vec<(DeviceIdentity, FailNthRead<_>)> = recording_of(&base, &stream)
        .into_iter()
        .map(|(identity, inner)| (identity, FailNthRead { identity, inner, target, nth, reads: reads.clone() }))
        .collect();
    let mounted = mount_writable_with_instance_table_sweep_configuration(
        &parameters_of_the_ring_hole_pools(), &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, sweeping_on_any_row(),
    ).expect("实例 3 的可写挂载");
    println!("ENUM nth={nth} reads_of_the_target={} sweep={}", reads.get(), format!("{:?}", mounted.output.instance_table_sweep).chars().take(300).collect::<String>());
    let (writes, segments) = writes_and_segments(&stream.retained_operations(), &fixed_geometry_of_the_ring_hole_pools());
    let state_count = singlefs_harness::memory_pool::closed_form_state_count(&segments);
    println!("ENUM writes={} segments={} closed_form_state_count={state_count}", writes.len(), segments.len());
    if state_count > cap {
        println!("ENUM skipped: state count {state_count} > cap {cap}");
        return;
    }
    let judged_root_index = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).expect("根槽写");
    let content = content_of_the_version(LAST_TXG_BEFORE_THE_SWEEPING_MOUNT);
    let mut versions = vec![PublishedVersion { instance: InstanceGeneration(2), checkpoint_txg: CheckpointTxg(LAST_TXG_BEFORE_THE_SWEEPING_MOUNT), content: content.clone() }];
    for write in writes.iter().filter(|w| w.kind == StepKind::RootRecordFua) {
        let root = singlefs_core::root_record::RootRecord::parse_slot(write.bytes().expect("字节"), &parameters_of_the_ring_hole_pools().filesystem_identifier).expect("根");
        versions.push(PublishedVersion { instance: root.instance, checkpoint_txg: root.checkpoint_txg, content: content.clone() });
    }
    let sweep_publish = (mounted.current.root().instance, mounted.current.root().checkpoint_txg);
    let deleted = matches!(mounted.output.instance_table_sweep, InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted { .. }));
    let mut effective_states = 0u64;
    let mut f5_left_states = 0u64;
    let mut f5_left_max = 0u64;
    let mut observe = |image: &CrashImage<'_>, report: &RecoveryReport, _counts: &mut Layer0ObserverCounts| {
        if deleted && report.effective_root == Some(sweep_publish) {
            effective_states += 1;
            let left = unpublished_units_of_the_row_left_on(&materialized_crash_image(image), &row_before);
            if left > 0 { f5_left_states += 1; f5_left_max = f5_left_max.max(left); }
        }
    };
    let started = std::time::Instant::now();
    let tally: Layer0Tally = enumerate_layer0_in_state_slices(
        &base, &writes, &segments, judged_root_index, &versions, &full_expansion,
        Layer0Parallelism::from_environment(), Some(&mut observe), &Layer0Resume::NoProgressFile,
    );
    let checker_red: u64 = tally.checker_violated_states.values().sum();
    println!(
        "ENUM-TALLY nth={nth} states={} violations={} failed_states={} checker_violated_states={checker_red} checker_first={:?} deletion_effective_states={effective_states} f5_left_states={f5_left_states} f5_left_max={f5_left_max} i928_not_applicable_or_partial={:?} seconds={}",
        tally.states, tally.violations, tally.failed_states, tally.checker_first_violation, tally.checker_not_applicable_states.get("I-9.28"), started.elapsed().as_secs()
    );
}
