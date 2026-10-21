//! checker 档模块：crash、crash_state_enumeration、writable_mount_phases、crash_amplification
//! 实例表清扫的崩溃点（D18（块里携带什么信息） 已定项 11「回收」一条；判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md`
//! 第三节 F5 与第二节 Q5；C597（实例表行只增不删，删行的条件没有条款） 的判别力自证）。
//!
//! 流：S = 4 的 12 槽小环、两块 128 MiB 内存盘。mkfs → 实例 1 第一个文件、覆盖写两次 → 一次两个数据单元的顺序写崩在第一条 journal 记录写之前
//! （孤儿：按实例 1 那一行判未发布）→ 实例 2 可写挂载、覆盖写到 txg 19（实例 1 的根全部轮出环）→ 实例 3 可写挂载，末尾清扫（有一行就扫）：
//! 抹孤儿的头、每块盘一道屏障、发清扫发布删掉实例 1 那一行。基线是实例 3 那次挂载之前的盘面，枚举的是那次挂载的全部写。
//!
//! 比已有的流多罩的崩溃状态：已有的崩溃点重放流与崩溃注入里没有一次挂载写过清扫的抹头与清扫发布（产品路径的阈值占位 369 行，那几条流到不了）；
//! 这一条的挂载末尾两段（`writable_mount_phases::WritableMountPhase` 的抹头与清扫发布）上每个崩溃状态都枚举。
//! 多跑的一步：清扫两段上的每个崩溃状态再起一次可写挂载（也是有一行就扫），判挂载做成、池级 checker 不红、F5 照判。
//!
//! F5（判决第三节 F5，C597 的验收）：删行那一版生效的每个崩溃态里，被删实例按删前那一行判未发布的单元一个不剩——行一删，盘上再没有东西说它们
//! 没发布过（池级 checker 与 oracle 都看不见），只有看得见删行之前那一行的观察者判得了。判法在这份用例里另写一份（认头只看魔数、类、头校验和，
//! 照认领规则按整单元跳），不调实现的清扫器。
//!
//! C333（删行那次发布被重放）：同一次枚举里每个状态再判「重建态的实例表与分配记录一致」——恢复按记录重建出来的那条根指着的实例表每一片，
//! 在同一条根的分配记录里都有一条仍分配的记录罩住（被引用的片没有记成释放）；重建出来的是清扫发布那一版时，它的实例表里没有被删的行。
//!
//! 跨片流（C597 跨片那一半）：同一个池，产品路径的配置（开关关着、阈值取占位 369）接连崩了再挂到实例表 369 行，下一次挂载写到 370 行、链长出第二片，
//! 挂载末尾越过阈值清扫，链回到一片；基线是那次挂载之前的盘面，枚举的是那次挂载的全部写，每个状态照 C333 判。
#[path = "../../singlefs-harness/tests/common_root_ring_holes/mod.rs"]
mod common_root_ring_holes;

use std::collections::BTreeSet;

use common_root_ring_holes::{
    content_of_the_version, devices_of, fixed_geometry_of_the_ring_hole_pools,
    parameters_of_the_ring_hole_pools, pool_of, recording_of, RingHoleDevices,
    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, JOURNAL_RING_BYTES_OF_THE_RING_HOLE_POOLS,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker_tier::crash::{
    full_expansion, quick_tier_expansion, CrashPointReplayObserverCounts,
    CrashPointReplaySegmentExpansion, CrashPointReplayTally, CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver, ObserverDeduplication,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, enumerate_crash_states_with_expansion,
    CrashPointReplayItem, CrashPointReplayObserver, CrashPointReplayObserverHandingBackAnImage,
};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs";
// 经 `writable_mount_phases` 引、不经 `crash_injection`：这条流是手写的、不拿随机历史当输入，不该把随机历史脚手架带进判法闭包。
use singlefs_checker_tier::writable_mount_phases::{
    materialized_crash_image, publishes_before_the_instance_table_sweep_of_the_mount_output,
    writes_of_the_instance_table_sweep_phases, WritableMountPhase,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::{
    instance_switch_reserve_on_one_device, BytesOnOneDevice, MetadataBlocks, SpaceAdmission,
};
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
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_system_configuration,
    instance_table_chain_of_root, readable_roots, replay_journal, scan_journal, RecoveryReport,
};
use singlefs_core::root_record::RootRecord;
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

/// 清扫那次挂载录下来：基线、写表与段、被判的根槽写（清扫发布那一次）、允许的版本、删前那一行、清扫发布那一版、清扫两段各是哪几个写。
struct RecordedSweepingMount {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    row_deleted: InstanceRow,
    sweep_publish: (InstanceGeneration, CheckpointTxg),
    writes_of_the_sweep_phases: BTreeSet<usize>,
}

fn record_the_sweeping_mount() -> RecordedSweepingMount {
    let base = pool_with_a_dead_instance();
    let stream = SharedStream::retaining_contents();
    let mut devices = recording_of(&base, &stream);
    let mounted = mount_writable_with_instance_table_sweep_configuration(
        &parameters_of_the_ring_hole_pools(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        sweeping_on_any_row(),
    )
    .expect("实例 3 的可写挂载");
    let InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
        rows_deleted,
        heads_erased,
    }) = &mounted.output.instance_table_sweep
    else {
        panic!(
            "前提：没崩过的那次挂载删了实例 1 的行：{:?}",
            mounted.output.instance_table_sweep
        );
    };
    assert!(!heads_erased.is_empty(), "前提：抹了孤儿的头");
    let [row_deleted] = rows_deleted.as_slice() else {
        panic!("前提：只删实例 1 那一行：{rows_deleted:?}");
    };
    let publishes_before_the_sweep =
        publishes_before_the_instance_table_sweep_of_the_mount_output(&mounted.output)
            .expect("清扫写了东西");
    let (writes, segments) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("清扫发布写了根槽");
    let writes_by_phase =
        writes_of_the_instance_table_sweep_phases(&writes, publishes_before_the_sweep);
    let writes_of_the_sweep_phases: BTreeSet<usize> = [
        WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
        WritableMountPhase::InstanceTableSweepPublish,
    ]
    .iter()
    .flat_map(|phase| writes_by_phase.get(phase).cloned().unwrap_or_default())
    .collect();
    let content = content_of_the_version(LAST_TXG_BEFORE_THE_SWEEPING_MOUNT);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(LAST_TXG_BEFORE_THE_SWEEPING_MOUNT),
        content: content.clone(),
    }];
    for write in writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
    {
        let root = RootRecord::parse_slot(
            write.bytes().expect("根槽写带字节"),
            &parameters_of_the_ring_hole_pools().filesystem_identifier,
        )
        .expect("挂载写的根自证得过");
        versions.push(PublishedVersion {
            instance: root.instance,
            checkpoint_txg: root.checkpoint_txg,
            content: content.clone(),
        });
    }
    RecordedSweepingMount {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        row_deleted: *row_deleted,
        sweep_publish: (
            mounted.current.root().instance,
            mounted.current.root().checkpoint_txg,
        ),
        writes_of_the_sweep_phases,
    }
}

/// F5 那一判在一个崩溃态上：恢复落到清扫发布那一版（删行那一版生效）时，删前那一行判未发布的单元一个都不许剩；交回这个态是不是删行那一版生效的。
fn deletion_took_effect_and_no_unpublished_unit_is_left(
    recorded: &RecordedSweepingMount,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
) -> bool {
    if report.effective_root != Some(recorded.sweep_publish) {
        return false;
    }
    let left = unpublished_units_of_the_row_left_on(
        &materialized_crash_image(image),
        &recorded.row_deleted,
    );
    assert_eq!(
        left, 0,
        "F5：删行那一版生效的崩溃态里，按删前那一行 {:?} 判未发布的单元剩 {left} 个",
        recorded.row_deleted
    );
    true
}

fn assert_the_recovery_and_the_checker_are_clean(tally: &CrashPointReplayTally) {
    assert_eq!(
        (tally.violations, tally.first_violation.clone()),
        (0, None),
        "恢复之后多版本 oracle 0 违例"
    );
    assert_eq!(tally.failed_states, 0, "每个状态都恢复得了");
    assert_eq!(
        tally.checker_violated_states.values().sum::<u64>(),
        0,
        "恢复之后池级 checker 0 违例：{:?}",
        tally.checker_first_violation
    );
}

/// 观察者「清扫两段之后再挂」（`remount_after_the_sweep_phases`）：这个崩溃状态当交回镜像的观察者再起一次可写挂载（有一行就扫）：挂不上、
/// 行还在、删前那一行判未发布的单元还剩，都当场 panic（记成红项、用例红），挂成的记一个数；挂完的盘交回、那几条不变量在判段判。
/// 观察者的身份是这个 fn 的闭包键（入口登记在 `singlefs_checker_tier::judging_units_entries::OBSERVER_ENTRIES`）。
// gpu-exempt: writable-mount
fn remount_after_the_sweep_phases_without_the_deleted_row(
    recorded: &RecordedSweepingMount,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    deletion_took_effect_and_no_unpublished_unit_is_left(recorded, image, report);
    let mut materialized = recorded.base.clone();
    let landed: Vec<RetainedWrite> = image
        .writes
        .iter()
        .zip(&image.persisted)
        .filter(|(_, is_persisted)| **is_persisted)
        .map(|(write, _)| write.clone())
        .collect();
    materialized.apply_writes(&landed);
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices = recording_of(&materialized, &stream);
    let mounted = mount_writable_with_instance_table_sweep_configuration(
        &parameters_of_the_ring_hole_pools(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        sweeping_on_any_row(),
    )
    .unwrap_or_else(|error| panic!("再挂被拒：{error:?}"));
    let mut after = materialized.clone();
    after.apply(&stream.retained_operations());
    let rows_after: Vec<InstanceGeneration> =
        instance_table_chain_of_root(&after, mounted.current.root())
            .map(|chain| chain.records.rows.iter().map(|row| row.instance).collect())
            .unwrap_or_default();
    assert!(
        !rows_after.contains(&recorded.row_deleted.instance),
        "再挂之后实例 1 那一行还在：{rows_after:?}"
    );
    let left = unpublished_units_of_the_row_left_on(&after, &recorded.row_deleted);
    assert_eq!(left, 0, "再挂之后删前那一行判未发布的单元剩 {left} 个");
    counts.add("remounted_without_the_deleted_row", 1);
    let (writes_on_the_crash_image, _segments_of_the_mount) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    ImageAfterTheObserver {
        writes_on_the_crash_image,
        disks_after_the_observer: after,
        ranges_read_on_the_crash_image:
            singlefs_checker_tier::crash_amplification::ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(
                &stream.read_ranges(),
            ),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::Only(
            INVARIANTS_JUDGED_AFTER_THE_REMOUNT.to_vec(),
        ),
        counter_when_the_pool_checker_is_clean: "checker_clean_after_the_remount".to_string(),
    }
}

/// 观察者「删行之后 F5」（`f5_after_the_deletion`）：这个状态上删行那一版生效、删前那一行判未发布的单元一个不剩的，记一次
/// `deletion_took_effect`。观察者的身份是这个 fn 的闭包键（入口登记在 `singlefs_checker_tier::judging_units_entries::OBSERVER_ENTRIES`）。
fn count_the_states_where_the_deletion_took_effect(
    recorded: &RecordedSweepingMount,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    if deletion_took_effect_and_no_unpublished_unit_is_left(recorded, image, report) {
        counts.add("deletion_took_effect", 1);
    }
}

/// 删行那次发布（清扫发布）在 C333（删行那次发布被重放） 那一判里要对着的两样：删掉的是哪几个实例的行，清扫发布那一版的 (实例, txg)。
#[derive(Clone, Debug)]
struct DeletionPublishToJudgeTheRebuiltStateAgainst {
    deleted_instances: BTreeSet<InstanceGeneration>,
    sweep_publish: (InstanceGeneration, CheckpointTxg),
}

impl DeletionPublishToJudgeTheRebuiltStateAgainst {
    /// 观察者另读的输入的摘要（进它的版本）。
    fn digest(&self) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
        singlefs_harness::sha256::sha256_digest(
            format!(
                "deleted_instances={:?}\nsweep_publish={:?}\n",
                self.deleted_instances, self.sweep_publish
            )
            .as_bytes(),
        )
    }
}

/// 恢复在这个崩溃态上重建出来的那条根：择系统配置、择根、扫 journal、照记录施加（与 `recovery::recover` 那几步同一串，点名单元逐个验）。
/// 走读失败之后的退回不在这里做：这里重建出来的 (实例, txg) 与恢复报告里实际走的那条对不上，当场 panic（这条流里没有走读失败的态）。
fn root_rebuilt_by_the_recovery(image: &CrashImage<'_>, report: &RecoveryReport) -> RootRecord {
    let system_configuration = choose_system_configuration(image)
        .unwrap_or_else(|failure| panic!("恢复走到了重建根那一步、系统配置择得出：{failure:?}"));
    let chosen =
        choose_root(image, &system_configuration).expect("恢复走到了重建根那一步、择得到根");
    let records = scan_journal(image, &system_configuration);
    let (_journal, rebuilt) = replay_journal(
        image,
        &chosen,
        system_configuration.immutable.sizes.journal_ring_bytes,
        &records,
        true,
    )
    .unwrap_or_else(|failure| panic!("恢复走到了重建根那一步、施加得了记录：{failure:?}"));
    assert_eq!(
        Some((rebuilt.instance, rebuilt.checkpoint_txg)),
        report.effective_root,
        "这里重建出来的根就是恢复报告里实际走的那条（没有走读失败之后的退回）"
    );
    rebuilt
}

/// 观察者「重建态的实例表与分配记录」（`rebuilt_instance_table_against_the_allocation_records`，C333（删行那次发布被重放））：恢复交回了实际走的那条根的
/// 每个状态上，重建出来的那条根指着的实例表每一片的每一份副本，在同一条根的分配记录里都有一条罩住它、仍分配的记录（被引用的片没有记成释放、
/// 也不是账外的槽），记一次 `rebuilt_instance_table_allocated`；重建出来的就是清扫发布那一版时，它的实例表里一行被删的实例都没有，
/// 再记一次 `rebuilt_sweep_publish_without_the_deleted_rows`。不满足的当场 panic（记成红项、用例红）。
/// 判法只读盘、按恢复的读法重建（`root_rebuilt_by_the_recovery`），不调实现的清扫器。
/// 观察者的身份是这个 fn 的闭包键（入口要登记在 `singlefs_checker_tier::judging_units_entries::OBSERVER_ENTRIES`；没登记时取键拒算、照判）。
fn judge_the_rebuilt_instance_table_against_the_allocation_records(
    deletion_publish: &DeletionPublishToJudgeTheRebuiltStateAgainst,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    if report.effective_root.is_none() {
        return;
    }
    let rebuilt = root_rebuilt_by_the_recovery(image, report);
    let chain = instance_table_chain_of_root(image, &rebuilt).unwrap_or_else(|failure| {
        panic!(
            "重建出来的根 {:?} 指着的实例表读得出：{failure:?}",
            report.effective_root
        )
    });
    let records = allocation_records_under_root(image, &rebuilt).unwrap_or_else(|failure| {
        panic!(
            "重建出来的根 {:?} 的分配记录读得出：{failure:?}",
            report.effective_root
        )
    });
    for (page_index, page) in chain.page_pointers.iter().enumerate() {
        for location in &page.locations {
            let covering = records.iter().find(|record| {
                record.device == location.device
                    && record.slot.0 <= location.slot.0
                    && location.slot.0 < record.slot.0 + u64::from(record.span_slots)
            });
            assert!(
                covering.is_some_and(|record| !record.is_released),
                "重建出来的根 {:?} 的实例表第 {page_index} 片在盘 {} 槽 {} 的那一份，同一条根的分配记录里是 {covering:?}（要有一条仍分配的记录罩住它）",
                report.effective_root,
                location.device.0,
                location.slot.0
            );
        }
    }
    counts.add("rebuilt_instance_table_allocated", 1);
    if report.effective_root == Some(deletion_publish.sweep_publish) {
        let instances_left: Vec<InstanceGeneration> = chain
            .records
            .rows
            .iter()
            .map(|row| row.instance)
            .filter(|instance| deletion_publish.deleted_instances.contains(instance))
            .collect();
        assert!(
            instances_left.is_empty(),
            "重建出来的是清扫发布那一版 {:?}，它的实例表里还有被删的实例 {instances_left:?} 的行",
            deletion_publish.sweep_publish
        );
        counts.add("rebuilt_sweep_publish_without_the_deleted_rows", 1);
    }
}

/// C333 那个观察者记的两个数（`rebuilt_instance_table_allocated`、`rebuilt_sweep_publish_without_the_deleted_rows`）都不为 0：
/// 判到了重建态，也判到了清扫发布那一版生效的态（施加清扫发布的记录、或根槽已落盘的那几种）。
fn assert_the_rebuilt_state_was_judged(tally: &CrashPointReplayTally) {
    assert!(
        tally
            .observer_counts
            .count_named("rebuilt_instance_table_allocated")
            > 0,
        "C333 那一判判到了重建态（{} 个状态里 0 个）",
        tally.states
    );
    assert!(
        tally
            .observer_counts
            .count_named("rebuilt_sweep_publish_without_the_deleted_rows")
            > 0,
        "C333 那一判判到了清扫发布那一版生效的态（{} 个状态里 0 个）",
        tally.states
    );
}

/// 枚举清扫那次挂载的写，每个状态照 F5 判、再照 C333 判重建态的实例表与分配记录（观察者挂进崩溃判定库那一路，判到的数经观察者计数带出来）；
/// 交回 tally 与删行那一版生效的状态数。
fn enumerate_the_sweeping_mount_judging_f5(
    recorded: &RecordedSweepingMount,
    item_label: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<(CrashPointReplayTally, u64)> {
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        count_the_states_where_the_deletion_took_effect(recorded, image, report, counts);
    };
    let deletion_publish = DeletionPublishToJudgeTheRebuiltStateAgainst {
        deleted_instances: BTreeSet::from([recorded.row_deleted.instance]),
        sweep_publish: recorded.sweep_publish,
    };
    let judge_the_rebuilt_state =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            judge_the_rebuilt_instance_table_against_the_allocation_records(
                &deletion_publish,
                image,
                report,
                counts,
            );
        };
    let item_name = format!("instance_table_sweep::mount::{item_label}");
    let tally = enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&recorded.versions),
        expansion,
        // 观察者另读的输入：删前那一行（F5 对着它判未发布的单元剩几个）；删掉的实例与清扫发布那一版（C333 那一判对着它们）
        &[
            CrashPointReplayObserver {
                name: "f5_after_the_deletion",
                extra_inputs_digest: singlefs_harness::sha256::sha256_digest(
                    format!("row_deleted={:?}\n", recorded.row_deleted).as_bytes(),
                ),
                observe: &observe,
            },
            CrashPointReplayObserver {
                name: "rebuilt_instance_table_against_the_allocation_records",
                extra_inputs_digest: deletion_publish.digest(),
                observe: &judge_the_rebuilt_state,
            },
        ],
    )
    .judged()?;
    let states_where_the_deletion_took_effect =
        tally.observer_counts.count_named("deletion_took_effect");
    Some((tally, states_where_the_deletion_took_effect))
}

/// 快档：清扫那次挂载的写按甲二展开，每个状态跑恢复、oracle、池级 checker、F5 与 C333（重建态的实例表与分配记录）。
/// 判别力：清扫发布照发、头不抹（「不抹就删」）时，删行那一版生效的态里孤儿头都还在，F5 红；
/// 由记录重建根时实例表指针与分配记录树根照抄所选根（立账时的施加规则）时，施加了记录的那几个态重建出来的根，实例表的片在同一条根的
/// 分配记录里记成释放（副本里同一判法的 harness 探针在这一条流与跨片流上都红在这一处），C333 那一判红。
#[test]
fn quick_tier_of_the_sweeping_mount_leaves_no_unpublished_unit_of_the_deleted_row() {
    let recorded = record_the_sweeping_mount();
    assert!(
        !recorded.writes_of_the_sweep_phases.is_empty(),
        "前提：挂载写里有清扫两段"
    );
    let Some((tally, states_where_the_deletion_took_effect)) =
        enumerate_the_sweeping_mount_judging_f5(&recorded, "quick_tier", &quick_tier_expansion)
    else {
        return;
    };
    assert_the_recovery_and_the_checker_are_clean(&tally);
    assert_the_rebuilt_state_was_judged(&tally);
    assert!(
        states_where_the_deletion_took_effect > 0,
        "F5 判到了删行那一版生效的状态（{} 个状态里 0 个，说明枚举没走到清扫发布落盘之后）",
        tally.states
    );
}

/// 全量：清扫那次挂载的每一段都展开（D13（验证路线） 已定项 4 的全量），每个状态跑恢复、oracle、池级 checker、F5 与 C333（删行那次发布的全部崩溃点）。
#[test]
#[ignore = "checker 档全量崩溃枚举：登记在 .claude/gate.d/stage-inputs.tsv 的 crash-case:instance-table-sweep-mount，由 checker-tier-crash-point-replay --full 跑"]
fn every_crash_state_of_the_sweeping_mount_leaves_no_unpublished_unit_of_the_deleted_row() {
    let recorded = record_the_sweeping_mount();
    let Some((tally, states_where_the_deletion_took_effect)) =
        enumerate_the_sweeping_mount_judging_f5(&recorded, "full", &full_expansion)
    else {
        return;
    };
    assert_the_recovery_and_the_checker_are_clean(&tally);
    assert_the_rebuilt_state_was_judged(&tally);
    assert!(
        states_where_the_deletion_took_effect > 0,
        "F5 判到了删行那一版生效的状态"
    );
}

/// 清扫两段（抹头、清扫发布）上的每个崩溃状态再起一次可写挂载（有一行就扫）：挂载做成，之后的盘面上那几条不变量不红，
/// 删行那一版在那一刻生效的照 F5 判一个不剩；挂载之后实例 1 那一行要么已删、要么这一次挂载又删了（清扫这一轮整轮重扫重抹，判决第二节 Q5）。
#[test]
#[ignore = "checker 档全量崩溃枚举：登记在 .claude/gate.d/stage-inputs.tsv 的 crash-case:instance-table-sweep-phases-remount，由 checker-tier-crash-point-replay --full 跑"]
fn every_crash_state_inside_the_two_sweep_phases_remounts_writable_and_ends_without_the_deleted_row(
) {
    let recorded = record_the_sweeping_mount();
    let sweep_phase_writes = recorded.writes_of_the_sweep_phases.clone();
    let expand_the_sweep_phases_only = |_segment_index: usize, segment: &[usize]| {
        if segment
            .iter()
            .any(|write_index| sweep_phase_writes.contains(write_index))
        {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    // 清扫两段上的每个崩溃状态当交回镜像的观察者再起一次可写挂载（有一行就扫；两段，用户 2026-10-13 定：录入段在 CPU 上挂，
    // 豁免表的 writable-mount 那一行，按恢复镜像去重）：挂不上、行还在、删前那一行判未发布的单元还剩，都当场 panic（记成红项、用例红），
    // 挂成的记一个数；挂完的盘上那几条不变量在判段判（配了 GPU 上卡），判干净时 `checker_clean_after_the_remount` 加一
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts|
     -> ImageAfterTheObserver {
        remount_after_the_sweep_phases_without_the_deleted_row(&recorded, image, report, counts)
    };
    let Some(tally) = enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: "instance_table_sweep::phases_remount",
            setting_file: THIS_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&recorded.versions),
        &expand_the_sweep_phases_only,
        &[],
        // 观察者另读的输入：删前那一行（判行在不在、未发布的单元剩几个都对着它）、挂载的参数与清扫的配置
        &[CrashPointReplayObserverHandingBackAnImage {
            name: "remount_after_the_sweep_phases",
            extra_inputs_digest: singlefs_harness::sha256::sha256_digest(
                format!(
                    "row_deleted={:?}\nparameters={:?}\nsweep={:?}\n",
                    recorded.row_deleted,
                    parameters_of_the_ring_hole_pools(),
                    sweeping_on_any_row()
                )
                .as_bytes(),
            ),
            observe: &observe,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        }],
    )
    .judged() else {
        return;
    };
    assert_the_recovery_and_the_checker_are_clean(&tally);
    assert_eq!(
        tally
            .observer_counts
            .count_named("remounted_without_the_deleted_row"),
        tally.states,
        "清扫两段上的每个崩溃态都再挂成、行已删、单元不剩"
    );
    assert_eq!(
        tally
            .observer_counts
            .count_named("checker_clean_after_the_remount"),
        tally.states,
        "清扫两段上的每个崩溃态再挂之后的盘，那几条不变量判段一条都不判违例"
    );
}

/// 跨片流至多挂几次（mkfs 那个实例不写行、之后每次挂载写一行：第 370 次挂载之后 369 行，下一次挂载越过阈值）。
const MOUNTS_UNTIL_THE_ROWS_REACH_THE_THRESHOLD_AT_MOST: usize = 400;

/// 实例切换预留按这一个 ckpt_cost 算（只看链重写那一半随行数怎么变；暖机那一半与行数无关，取 0 不碍比较）。
const CHECKPOINT_COST_COMPARED_ACROSS_THE_MOUNTS: MetadataBlocks = MetadataBlocks(0);

/// 跨片流录下来的那次越过阈值的可写挂载（C597（实例表行只增不删，删行的条件没有条款） 跨片那一半）：基线、写表与段、被判的根槽写、允许的版本、
/// 删掉的行、清扫发布那一版；连同这条流上看到的几样——越过之前那一次挂载的清扫结局，清扫判的那一刻的实例表与根环里有根的实例，
/// 清扫之后的实例表，三次挂载的实例切换预留。
struct RecordedCrossingMount {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    rows_deleted: Vec<InstanceRow>,
    sweep_publish: (InstanceGeneration, CheckpointTxg),
    sweep_of_the_mount_before: InstanceTableSweepAtTheEndOfTheMount,
    rows_at_the_judgement: Vec<InstanceRow>,
    pages_at_the_judgement: usize,
    instances_with_a_root_in_the_ring_at_the_judgement: BTreeSet<InstanceGeneration>,
    rows_after_the_sweep: Vec<InstanceRow>,
    pages_after_the_sweep: usize,
    switch_reserve_of_the_first_mount_writing_a_row: BytesOnOneDevice,
    switch_reserve_of_the_crossing_mount: BytesOnOneDevice,
    switch_reserve_of_the_mount_after: BytesOnOneDevice,
}

fn switch_reserve_of(mounted: &Mounted) -> BytesOnOneDevice {
    instance_switch_reserve_on_one_device(
        mounted
            .allocator
            .instance_rows_after_this_mounts_row_publish(),
        CHECKPOINT_COST_COMPARED_ACROSS_THE_MOUNTS,
    )
}

/// 盘面上最新那条可读根，与根环里全部可读根的实例。
fn newest_root_and_instances_in_the_ring(
    image: &MemoryPool,
) -> (RootRecord, BTreeSet<InstanceGeneration>) {
    let system_configuration = choose_system_configuration(image).expect("系统配置择得出");
    let immutable = &system_configuration.immutable;
    let roots = readable_roots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    );
    let newest = *roots
        .iter()
        .max_by_key(|root| (root.checkpoint_txg, root.instance))
        .expect("根环里有根");
    (newest, roots.iter().map(|root| root.instance).collect())
}

/// mkfs → 实例 1 第一个文件 → 接连崩了再挂（产品路径的配置：开关关着、阈值取占位 369），直到某次挂载之后实例表恰好 369 行；
/// 录下下一次挂载（实例表写到 370 行、链长出第二片，挂载末尾越过阈值清扫），再在它之后的盘面上崩了再挂一次。
fn record_the_mount_whose_rows_cross_the_threshold() -> RecordedCrossingMount {
    let parameters = parameters_of_the_ring_hole_pools();
    let threshold = InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690;
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
    {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content_of_the_version(0),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
            },
            output.instance,
            current.record_bytes(),
        )
        .expect("第一个文件");
    }
    let mut switch_reserve_of_the_first_mount_writing_a_row = None;
    let mut sweep_of_the_mount_before = None;
    // 迭代上界：MOUNTS_UNTIL_THE_ROWS_REACH_THE_THRESHOLD_AT_MOST 次崩了再挂；跨轮携带的是盘与第一次写行那次挂载的切换预留；
    // 提前出口是实例表到了阈值那一次。
    for mount_index in 0..MOUNTS_UNTIL_THE_ROWS_REACH_THE_THRESHOLD_AT_MOST {
        let mounted = mount_writable(&parameters, &mut devices)
            .unwrap_or_else(|error| panic!("第 {mount_index} 次崩了再挂做成：{error:?}"));
        if switch_reserve_of_the_first_mount_writing_a_row.is_none() {
            switch_reserve_of_the_first_mount_writing_a_row = Some(switch_reserve_of(&mounted));
        }
        let InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { rows, .. } =
            mounted.output.instance_table_sweep
        else {
            panic!(
                "前提：越过阈值之前的挂载都不扫（第 {mount_index} 次）：{:?}",
                mounted.output.instance_table_sweep
            );
        };
        if rows == threshold.0 {
            sweep_of_the_mount_before = Some(mounted.output.instance_table_sweep);
            break;
        }
    }
    let sweep_of_the_mount_before =
        sweep_of_the_mount_before.expect("前提：接连崩了再挂，实例表到了阈值");
    let base = pool_of(&devices);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&base, &stream);
    let mounted = mount_writable(&parameters, &mut recording).expect("越过阈值的那次可写挂载");
    let InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
        rows_deleted,
        ..
    }) = &mounted.output.instance_table_sweep
    else {
        panic!(
            "越过阈值的那次挂载末尾清扫删了行：{:?}",
            mounted.output.instance_table_sweep
        );
    };
    let (writes, segments) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    let root_writes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let [.., root_write_before_the_sweep_publish, judged_root_index] = root_writes[..] else {
        panic!("前提：这次挂载至少写了两条根（写行那一次与清扫发布）");
    };
    // 清扫判的那一刻：清扫发布之前那一次发布的根槽落了盘（清扫是挂载末尾最后一次发布，抹头的写不碰根环与实例表）。
    let mut at_the_judgement = base.clone();
    at_the_judgement.apply_writes(&writes[..=root_write_before_the_sweep_publish]);
    let (root_at_the_judgement, instances_with_a_root_in_the_ring_at_the_judgement) =
        newest_root_and_instances_in_the_ring(&at_the_judgement);
    let chain_at_the_judgement =
        instance_table_chain_of_root(&at_the_judgement, &root_at_the_judgement)
            .expect("清扫判的那一刻的实例表读得出");
    let mut after = base.clone();
    after.apply(&stream.retained_operations());
    let chain_after_the_sweep = instance_table_chain_of_root(&after, mounted.current.root())
        .expect("清扫发布那一版的实例表读得出");
    let content = content_of_the_version(0);
    let (newest_root_of_the_base, _) = newest_root_and_instances_in_the_ring(&base);
    let mut versions = vec![PublishedVersion {
        instance: newest_root_of_the_base.instance,
        checkpoint_txg: newest_root_of_the_base.checkpoint_txg,
        content: content.clone(),
    }];
    for write in writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
    {
        let root = RootRecord::parse_slot(
            write.bytes().expect("根槽写带字节"),
            &parameters.filesystem_identifier,
        )
        .expect("挂载写的根自证得过");
        versions.push(PublishedVersion {
            instance: root.instance,
            checkpoint_txg: root.checkpoint_txg,
            content: content.clone(),
        });
    }
    let mut devices_after = devices_of(&after);
    let mounted_after =
        mount_writable(&parameters, &mut devices_after).expect("越过阈值之后再崩了再挂一次");
    RecordedCrossingMount {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        rows_deleted: rows_deleted.clone(),
        sweep_publish: (
            mounted.current.root().instance,
            mounted.current.root().checkpoint_txg,
        ),
        sweep_of_the_mount_before,
        rows_at_the_judgement: chain_at_the_judgement.records.rows,
        pages_at_the_judgement: chain_at_the_judgement.page_pointers.len(),
        instances_with_a_root_in_the_ring_at_the_judgement,
        rows_after_the_sweep: chain_after_the_sweep.records.rows,
        pages_after_the_sweep: chain_after_the_sweep.page_pointers.len(),
        switch_reserve_of_the_first_mount_writing_a_row:
            switch_reserve_of_the_first_mount_writing_a_row.expect("至少挂了一次"),
        switch_reserve_of_the_crossing_mount: switch_reserve_of(&mounted),
        switch_reserve_of_the_mount_after: switch_reserve_of(&mounted_after),
    }
}

/// 跨片流上不靠崩溃枚举就判得了的几样（D18（块里携带什么信息） 已定项 11「回收」一条；C597 跨片那一半）：
/// 越过之前那一次挂载 369 行、没超过阈值不扫；越过的那一次清扫判的那一刻实例表 370 行、两片；清扫发布删掉的恰好是根环里没有根的那几个实例的行，
/// 留下的恰好是有根的那几个、一片；实例切换预留在越过的那一次比第一次写行那次挂载多（链按两片留），之后那一次挂载回到一片的量。
fn assert_the_crossing_mount_sweeps_back_to_one_page(recorded: &RecordedCrossingMount) {
    let threshold = InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690;
    assert!(
        matches!(
            recorded.sweep_of_the_mount_before,
            InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { rows, threshold: seen }
                if rows == threshold.0 && seen == threshold
        ),
        "越过之前那一次挂载 369 行、没超过阈值不扫：{:?}",
        recorded.sweep_of_the_mount_before
    );
    assert_eq!(
        u64::try_from(recorded.rows_at_the_judgement.len()).expect("行数装得进 u64"),
        threshold.0 + 1,
        "越过的那一次挂载写行之后实例表 370 行"
    );
    assert_eq!(recorded.pages_at_the_judgement, 2, "370 行的链长出了第二片");
    let kept_by_the_clause: Vec<InstanceRow> = recorded
        .rows_at_the_judgement
        .iter()
        .copied()
        .filter(|row| {
            recorded
                .instances_with_a_root_in_the_ring_at_the_judgement
                .contains(&row.instance)
        })
        .collect();
    let deleted_by_the_clause: Vec<InstanceRow> = recorded
        .rows_at_the_judgement
        .iter()
        .copied()
        .filter(|row| {
            !recorded
                .instances_with_a_root_in_the_ring_at_the_judgement
                .contains(&row.instance)
        })
        .collect();
    assert_eq!(
        recorded.rows_after_the_sweep, kept_by_the_clause,
        "清扫之后留下的恰好是根环里还有根的那几个实例的行"
    );
    assert_eq!(
        recorded.rows_deleted, deleted_by_the_clause,
        "清扫删掉的恰好是根环里没有根的那几个实例的行"
    );
    assert_eq!(recorded.pages_after_the_sweep, 1, "清扫之后链回到一片");
    assert!(
        recorded.switch_reserve_of_the_crossing_mount
            > recorded.switch_reserve_of_the_first_mount_writing_a_row,
        "越过的那一次挂载的切换预留按两片留：{:?} 对 {:?}",
        recorded.switch_reserve_of_the_crossing_mount,
        recorded.switch_reserve_of_the_first_mount_writing_a_row
    );
    assert_eq!(
        recorded.switch_reserve_of_the_mount_after,
        recorded.switch_reserve_of_the_first_mount_writing_a_row,
        "清扫之后那一次挂载的切换预留回到一片的量（不单调涨）"
    );
}

/// 枚举越过阈值的那次挂载的写，每个状态照 C333 判重建态的实例表与分配记录；交回 tally。
fn enumerate_the_crossing_mount_judging_the_rebuilt_state(
    recorded: &RecordedCrossingMount,
    item_label: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    let deletion_publish = DeletionPublishToJudgeTheRebuiltStateAgainst {
        deleted_instances: recorded
            .rows_deleted
            .iter()
            .map(|row| row.instance)
            .collect(),
        sweep_publish: recorded.sweep_publish,
    };
    let judge_the_rebuilt_state =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            judge_the_rebuilt_instance_table_against_the_allocation_records(
                &deletion_publish,
                image,
                report,
                counts,
            );
        };
    let item_name = format!("instance_table_sweep::crossing_mount::{item_label}");
    enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&recorded.versions),
        expansion,
        // 观察者另读的输入：删掉的实例与清扫发布那一版（C333 那一判对着它们）
        &[CrashPointReplayObserver {
            name: "rebuilt_instance_table_against_the_allocation_records",
            extra_inputs_digest: deletion_publish.digest(),
            observe: &judge_the_rebuilt_state,
        }],
    )
    .judged()
}

/// 快档：跨片流（C597 跨片那一半）。产品路径的配置接连崩了再挂到实例表 369 行，下一次挂载写到 370 行、链长出第二片，挂载末尾越过阈值清扫：
/// 清扫发布照 D18（块里携带什么信息） 已定项 11「回收」一条删掉根环里没有根的实例的行、链回到一片，之后那一次挂载的切换预留回到一片的量；
/// 那次挂载的写按甲二展开，每个状态跑恢复、oracle、池级 checker 与 C333（重建态的实例表与分配记录）。
/// 比已有的流多罩的：已有的清扫流实例表都是一片（有一行就扫的配置），这一条是产品路径的阈值、写行那次发布重写两片的链、清扫发布换下两片换上一片。
/// 判别力：清扫发布不删行（实例表整链照原样重写）时，清扫之后还是 370 行、两片，切换预留不回落，这一条红。
#[test]
fn quick_tier_of_the_mount_whose_instance_table_rows_cross_the_threshold_sweeps_the_chain_back_to_one_page(
) {
    let recorded = record_the_mount_whose_rows_cross_the_threshold();
    assert_the_crossing_mount_sweeps_back_to_one_page(&recorded);
    let Some(tally) = enumerate_the_crossing_mount_judging_the_rebuilt_state(
        &recorded,
        "quick_tier",
        &quick_tier_expansion,
    ) else {
        return;
    };
    assert_the_recovery_and_the_checker_are_clean(&tally);
    assert_the_rebuilt_state_was_judged(&tally);
}

/// 全量：跨片流越过阈值的那次挂载每一段都展开（D13（验证路线） 已定项 4 的全量），每个状态跑恢复、oracle、池级 checker 与 C333。
#[test]
#[ignore = "checker 档全量崩溃枚举：登记在 .claude/gate.d/stage-inputs.tsv 的 crash-case:instance-table-sweep-crossing-mount，由 checker-tier-crash-point-replay --full 跑"]
fn every_crash_state_of_the_mount_whose_instance_table_rows_cross_the_threshold_keeps_the_rebuilt_instance_table_allocated(
) {
    let recorded = record_the_mount_whose_rows_cross_the_threshold();
    assert_the_crossing_mount_sweeps_back_to_one_page(&recorded);
    let Some(tally) =
        enumerate_the_crossing_mount_judging_the_rebuilt_state(&recorded, "full", &full_expansion)
    else {
        return;
    };
    assert_the_recovery_and_the_checker_are_clean(&tally);
    assert_the_rebuilt_state_was_judged(&tally);
}
