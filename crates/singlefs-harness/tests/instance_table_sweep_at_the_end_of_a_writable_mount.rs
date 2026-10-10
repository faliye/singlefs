//! 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条；判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第二节 S1
//! 与第六节用户定案；攻方原型 `research/prompts/m3-sweep-and-tree-id-r1-opus-model/sweep-proto/src/lib.rs` 的几形）。
//!
//! 盘面（S = 4 的 12 槽小环、两块 128 MiB 内存盘）：mkfs → 实例 1 第一个文件、覆盖写两次（txg 3、4、5）→ 一次十六个数据单元的顺序写崩在第一条 journal
//! 记录写之前（单元写落了：实例 1 按它那一行判未发布的孤儿）→ 实例 2 可写挂载（写行 6、暖机 7）→ 实例 2 覆盖写到 txg 19（根环里没有实例 1 的根了）。
//! 接着的可写挂载（实例 3）末尾判一次清扫。
mod common;
mod common_root_ring_holes;

use std::collections::BTreeSet;

use common::{
    with_unreadable_ranges, FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange,
    UnreadableRangeReadBack,
};
use common_root_ring_holes::{
    content_of_the_version, devices_of, fixed_geometry_of_the_ring_hole_pools,
    parameters_of_the_ring_hole_pools, pool_of, recording_of, RingHoleDevices,
    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table::InstanceRow;
use singlefs_core::instance_table_sweep::{
    erase_the_heads_and_publish_the_sweep, judge_the_instance_table_sweep,
    sweep_the_instance_table_while_mounted, unit_head_read_by_sweep, HeadRereadBeforeErasing,
    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
    NoRowDeletedThisRound, PlacementOfAUnitHead, SweepPublishPlacements, UnitHeadReadBySweep,
};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_instance_table_sweep_configuration, unmount, Mounted,
    ShadowLedger,
};
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_chain_of_root, readable_roots, recover,
    JournalPolicy, PoolReader, RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, publish_sequential_write, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{DATA_UNIT_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 实例 2 覆盖写到的 txg：根环 12 槽里只剩 txg 8–19，实例 1 的根全部轮出。
const LAST_TXG_BEFORE_THE_SWEEPING_MOUNT: u64 = 19;
/// 崩掉的那次顺序写写几个数据单元：比实例 2 那十二次覆盖写多，实例 3 挂载时还有孤儿数据单元留着
/// （覆盖写的数据单元取起点最小的空槽对，孤儿那几对在实例 2 的分配器里是空的，先被取走）。
const ORPHAN_DATA_UNITS: usize = 16;
/// 后台那一支判完之后至多覆盖写几次去占单子上的落点。
const OVERWRITES_AFTER_THE_JUDGEMENT_AT_MOST: u64 = 128;
/// 一个码 1 头的字节数（伪头照抄一个孤儿的头）。
const DATA_UNIT_HEAD_BYTES: usize = 105;
/// 带伪头那一版的内容长度：伪头落在数据单元第二个槽开头，内容要够长、又装得进一个数据单元。
const CONTENT_BYTES_CARRYING_A_FORGED_HEAD: usize = 20000;
/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
const INVARIANTS_THE_SWEEP_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];

fn violations_of(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter(|(invariant, _)| INVARIANTS_THE_SWEEP_KEEPS_GREEN.contains(invariant))
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn sweeping_at_the_end_of_the_mount_on_any_row() -> InstanceTableSweepConfiguration {
    InstanceTableSweepConfiguration {
        switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
        row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
        sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
    }
}

/// 一个槽里的单元头字节（读清扫认头那么长）。
fn head_bytes_at(image: &MemoryPool, placement: PlacementOfAUnitHead) -> Vec<u8> {
    PoolReader::read(
        image,
        placement.device,
        placement.slot.to_device_offset(),
        1024,
    )
    .expect("内存盘上读得出")
}

/// 实例 1 的那几个孤儿：一次顺序写崩在第一条 journal 记录写之前，单元写落了的那几个（落点与头）。
#[derive(Clone, Debug)]
struct OrphansOfInstanceOne {
    heads: Vec<(PlacementOfAUnitHead, UnitHeadReadBySweep)>,
    /// 第一个孤儿数据单元的头字节（伪头照抄它）。
    first_data_unit_head_bytes: Vec<u8>,
}

/// 带伪头的那一版：数据单元在哪（第一个槽），第二个槽开头的字节（伪头）。
#[derive(Clone, Debug)]
struct VersionCarryingAForgedHead {
    data_unit: PlacementOfAUnitHead,
    second_slot_bytes: Vec<u8>,
}

/// 清扫那次挂载之前的盘面。
struct PoolWithADeadInstance {
    image: MemoryPool,
    /// 盘上还留着的孤儿（顺序写落下的、实例 2 覆盖写没盖掉的）。
    orphans_left_on_disk: Vec<(PlacementOfAUnitHead, UnitHeadReadBySweep)>,
    forged: Option<VersionCarryingAForgedHead>,
}

fn publish_an_overwrite<Device: singlefs_core::block_device::BlockDevice>(
    devices: &mut Vec<(DeviceIdentity, Device)>,
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

/// mkfs → 实例 1 写到 txg 5 → 顺序写崩在第一条记录写之前；交回崩溃之后的盘面与孤儿。
fn pool_with_orphans_of_instance_one() -> (MemoryPool, OrphansOfInstanceOne) {
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
    let heads: Vec<(PlacementOfAUnitHead, UnitHeadReadBySweep)> = writes[..first_record_write]
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .filter_map(|write| {
            let head = unit_head_read_by_sweep(write.bytes()?)?;
            Some((
                PlacementOfAUnitHead {
                    device: write.device,
                    slot: SlotNumber(write.offset.0 / SLOT_BYTES),
                },
                head,
            ))
        })
        .collect();
    let first_data_unit_head_bytes = writes[..first_record_write]
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .filter_map(|write| write.bytes())
        .find(|bytes| unit_head_read_by_sweep(bytes).is_some_and(|head| head.unit_class == 1))
        .expect("顺序写落下了数据单元")[..DATA_UNIT_HEAD_BYTES]
        .to_vec();
    assert!(
        heads
            .iter()
            .all(|(_, head)| head.writer_instance == instance),
        "孤儿都是实例 1 写的"
    );
    (
        image,
        OrphansOfInstanceOne {
            heads,
            first_data_unit_head_bytes,
        },
    )
}

/// 孤儿之后：实例 2 可写挂载、覆盖写到 txg 19；`forged_head_in_the_content_of_txg` 给出时，那一版的内容在数据单元第二个槽开头带一个照抄孤儿的伪头。
fn pool_with_a_dead_instance(
    forged_head_in_the_content_of_txg: Option<u64>,
) -> PoolWithADeadInstance {
    let parameters = parameters_of_the_ring_hole_pools();
    let (image, orphans) = pool_with_orphans_of_instance_one();
    let mut devices = devices_of(&image);
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 2 的可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(2));
    let mut allocator = mounted.allocator;
    let mut current = mounted.current.into_file_version().expect("这一版带文件");
    let payload_offset_of_the_second_slot = usize::try_from(SLOT_BYTES).expect("16384")
        - (usize::try_from(DATA_UNIT_BYTES).expect("32768") - data_unit_payload_capacity());
    let mut forged = None;
    // 迭代上界：覆盖写到 txg 19；跨轮携带的是现行那一版与带伪头那一版在哪。
    while current.root.checkpoint_txg.0 < LAST_TXG_BEFORE_THE_SWEEPING_MOUNT {
        let txg = current.root.checkpoint_txg.0 + 1;
        let content = if forged_head_in_the_content_of_txg == Some(txg) {
            let mut content = vec![0x5Au8; CONTENT_BYTES_CARRYING_A_FORGED_HEAD];
            content[payload_offset_of_the_second_slot
                ..payload_offset_of_the_second_slot + DATA_UNIT_HEAD_BYTES]
                .copy_from_slice(&orphans.first_data_unit_head_bytes);
            content
        } else {
            content_of_the_version(txg)
        };
        current = publish_an_overwrite(
            &mut devices,
            &mut allocator,
            &current,
            &content,
            InstanceGeneration(2),
        );
        if forged_head_in_the_content_of_txg == Some(txg) {
            let location = current.data_pointers[0].locations[0];
            forged = Some(PlacementOfAUnitHead {
                device: location.device,
                slot: location.slot,
            });
        }
    }
    let image_before_the_sweeping_mount = pool_of(&devices);
    let orphans_left_on_disk = orphans
        .heads
        .iter()
        .copied()
        .filter(|(placement, head)| {
            unit_head_read_by_sweep(&head_bytes_at(&image_before_the_sweeping_mount, *placement))
                == Some(*head)
        })
        .collect();
    let forged = forged.map(|data_unit| {
        let second_slot = PlacementOfAUnitHead {
            device: data_unit.device,
            slot: SlotNumber(data_unit.slot.0 + 1),
        };
        VersionCarryingAForgedHead {
            data_unit,
            second_slot_bytes: head_bytes_at(&image_before_the_sweeping_mount, second_slot),
        }
    });
    PoolWithADeadInstance {
        image: image_before_the_sweeping_mount,
        orphans_left_on_disk,
        forged,
    }
}

/// 在 `image` 上按 `configuration` 起可写挂载（实例 3），盘外面包录制器：交回挂载、盘与录下的流。
fn sweeping_mount_on(
    image: &MemoryPool,
    configuration: InstanceTableSweepConfiguration,
) -> (
    Mounted,
    Vec<(
        DeviceIdentity,
        singlefs_harness::RecordingBlockDevice<SparseBlockDevice>,
    )>,
    SharedStream,
) {
    let stream = SharedStream::retaining_contents();
    let mut devices = recording_of(image, &stream);
    let mounted = mount_writable_with_instance_table_sweep_configuration(
        &parameters_of_the_ring_hole_pools(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        configuration,
    )
    .expect("实例 3 的可写挂载");
    (mounted, devices, stream)
}

fn image_after(image: &MemoryPool, stream: &SharedStream) -> MemoryPool {
    let mut after = image.clone();
    after.apply(&stream.retained_operations());
    after
}

/// 盘上最新那条根指着的实例表里有哪几个实例的行。
fn instances_with_a_row_on(image: &MemoryPool) -> Vec<InstanceGeneration> {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let immutable = &system_configuration.immutable;
    let newest: RootRecord = readable_roots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .max_by_key(|root| (root.checkpoint_txg, root.instance))
    .expect("根环里有根");
    instance_table_chain_of_root(image, &newest)
        .expect("实例表读得出")
        .records
        .rows
        .iter()
        .map(|row: &InstanceRow| row.instance)
        .collect()
}

fn placements_of(
    heads: &[(PlacementOfAUnitHead, UnitHeadReadBySweep)],
) -> BTreeSet<PlacementOfAUnitHead> {
    heads.iter().map(|(placement, _)| *placement).collect()
}

/// 场景：清扫那次挂载开关关着、阈值取「有一行就扫」。
/// 预期：根环里没有实例 1 的根，实例 1 那一行是候选行：盘上留着的实例 1 的孤儿头全被抹掉（读不出单元头），清扫发布删掉那一行
/// （最新那条根的实例表里只剩实例 2 的行）；挂载之后那几条不变量不红。
#[test]
fn the_row_of_a_dead_instance_is_deleted_after_its_roots_left_the_ring_and_its_unpublished_heads_are_erased(
) {
    let pool = pool_with_a_dead_instance(None);
    assert!(
        !pool.orphans_left_on_disk.is_empty(),
        "前提：盘上还留着实例 1 的孤儿"
    );
    let (mounted, _devices, stream) =
        sweeping_mount_on(&pool.image, sweeping_at_the_end_of_the_mount_on_any_row());
    let InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
        rows_deleted,
        heads_erased,
    }) = mounted.output.instance_table_sweep
    else {
        panic!("删了实例 1 的行：{:?}", mounted.output.instance_table_sweep);
    };
    assert_eq!(
        rows_deleted
            .iter()
            .map(|row| row.instance)
            .collect::<Vec<_>>(),
        vec![InstanceGeneration(1)],
        "删的是实例 1 那一行"
    );
    assert_eq!(
        heads_erased.iter().copied().collect::<BTreeSet<_>>(),
        placements_of(&pool.orphans_left_on_disk),
        "抹的正好是盘上留着的孤儿"
    );
    let after = image_after(&pool.image, &stream);
    for (placement, _) in &pool.orphans_left_on_disk {
        assert_eq!(
            unit_head_read_by_sweep(&head_bytes_at(&after, *placement)),
            None,
            "孤儿 {placement:?} 的头抹掉了"
        );
    }
    assert_eq!(
        instances_with_a_row_on(&after),
        vec![InstanceGeneration(2)],
        "清扫发布那一版的实例表里只剩实例 2 的行"
    );
    assert_eq!(violations_of(&after), Vec::<String>::new(), "清扫之后");
}

/// 场景：实例 2 覆盖写 txg 8 那一版的内容在数据单元第二个槽开头带一个照抄实例 1 孤儿的伪头（头校验和过、按实例 1 那一行判未发布）；
/// 那一版到清扫时已经出了根环、那个数据单元已释放、没被别的根的账引用，第一个槽里它自己的头还在。
/// 预期：认领规则按整单元跳——第一个槽认出码 1 单元，第二个槽被认领、不当探针：伪头不被抹（字节不变，不在抹了的落点里）。
#[test]
fn a_forged_head_inside_the_content_of_a_data_unit_is_claimed_by_the_unit_and_not_erased() {
    let pool = pool_with_a_dead_instance(Some(8));
    let forged = pool.forged.clone().expect("带伪头那一版写了");
    assert!(
        unit_head_read_by_sweep(&head_bytes_at(&pool.image, forged.data_unit))
            .is_some_and(|head| head.writer_instance == InstanceGeneration(2)),
        "前提：那个数据单元第一个槽里还是它自己的头（没被复用）"
    );
    assert!(
        unit_head_read_by_sweep(&forged.second_slot_bytes)
            .is_some_and(|head| head.writer_instance == InstanceGeneration(1)),
        "前提：第二个槽开头是一个认得出的、实例 1 的伪头"
    );
    let second_slot = PlacementOfAUnitHead {
        device: forged.data_unit.device,
        slot: SlotNumber(forged.data_unit.slot.0 + 1),
    };
    let (mounted, _devices, stream) =
        sweeping_mount_on(&pool.image, sweeping_at_the_end_of_the_mount_on_any_row());
    let InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
        heads_erased,
        ..
    }) = &mounted.output.instance_table_sweep
    else {
        panic!("这一轮删了行：{:?}", mounted.output.instance_table_sweep);
    };
    assert!(
        !heads_erased.contains(&second_slot),
        "伪头所在的槽不在抹了的落点里"
    );
    assert_eq!(
        head_bytes_at(&image_after(&pool.image, &stream), second_slot),
        forged.second_slot_bytes,
        "伪头那一槽的字节不变"
    );
}

/// 场景：实例 2 覆盖写 txg 14 那一版的内容带同样的伪头；清扫时那一版的根还在根环里（回退候选），它的数据单元在它的账里仍分配。
/// 再把那个数据单元第一个槽里的头翻坏一个字节（认领规则在那一槽认不出单元，第二个槽成了探针）。
/// 预期：跳过根环里可读根的账里仍分配的槽——伪头所在的槽在那条根的账里，不被抹（字节不变，不在抹了的落点里）。
#[test]
fn a_head_inside_a_unit_still_allocated_in_the_account_of_a_ring_root_is_skipped_and_not_erased() {
    let mut pool = pool_with_a_dead_instance(Some(14));
    let forged = pool.forged.clone().expect("带伪头那一版写了");
    pool.image.flip_byte(
        forged.data_unit.device,
        forged.data_unit.slot.to_device_offset(),
        20,
    );
    assert_eq!(
        unit_head_read_by_sweep(&head_bytes_at(&pool.image, forged.data_unit)),
        None,
        "前提：第一个槽认不出单元头了"
    );
    let second_slot = PlacementOfAUnitHead {
        device: forged.data_unit.device,
        slot: SlotNumber(forged.data_unit.slot.0 + 1),
    };
    let (mounted, _devices, stream) =
        sweeping_mount_on(&pool.image, sweeping_at_the_end_of_the_mount_on_any_row());
    let InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
        heads_erased,
        ..
    }) = &mounted.output.instance_table_sweep
    else {
        panic!("这一轮删了行：{:?}", mounted.output.instance_table_sweep);
    };
    assert!(
        !heads_erased.contains(&second_slot),
        "伪头所在的槽在环里那条根的账里，不在抹了的落点里"
    );
    assert_eq!(
        head_bytes_at(&image_after(&pool.image, &stream), second_slot),
        forged.second_slot_bytes,
        "伪头那一槽的字节不变"
    );
}

/// 场景：清扫那次挂载里，第一个孤儿所在的槽每次读都报设备错（挂载自己不读它）。
/// 预期：这一次清扫有一次读失败：一行都不删、一个头都不抹（结局报那个落点）；挂载照样做成，实例 1 那一行还在，孤儿头都还在。
#[test]
fn a_read_failure_on_one_placement_deletes_no_row_and_erases_no_head() {
    let pool = pool_with_a_dead_instance(None);
    let (first_orphan, _) = *pool
        .orphans_left_on_disk
        .first()
        .expect("前提：盘上还留着实例 1 的孤儿");
    let unreadable = SharedUnreadableRanges::new(
        vec![UnreadableRange {
            device: first_orphan.device,
            offset_in_bytes: first_orphan.slot.to_device_offset().0,
            length_in_bytes: 512,
            failing_reads: FailingReadsOfARange::Every,
        }],
        UnreadableRangeReadBack::DeviceError,
    );
    let stream = SharedStream::retaining_contents();
    let mut devices = with_unreadable_ranges(recording_of(&pool.image, &stream), &unreadable);
    let mounted = mount_writable_with_instance_table_sweep_configuration(
        &parameters_of_the_ring_hole_pools(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        sweeping_at_the_end_of_the_mount_on_any_row(),
    )
    .expect("读故障只在清扫读的那一槽：挂载照样做成");
    assert!(
        matches!(
            mounted.output.instance_table_sweep,
            InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::NoRowDeleted(
                NoRowDeletedThisRound::AReadOfAPlacementFailed { placement }
            )) if placement == first_orphan
        ),
        "读失败这一轮不删：{:?}",
        mounted.output.instance_table_sweep
    );
    let after = image_after(&pool.image, &stream);
    assert_eq!(
        instances_with_a_row_on(&after),
        vec![InstanceGeneration(1), InstanceGeneration(2)],
        "实例 1 那一行还在"
    );
    for (placement, head) in &pool.orphans_left_on_disk {
        assert_eq!(
            unit_head_read_by_sweep(&head_bytes_at(&after, *placement)),
            Some(*head),
            "孤儿 {placement:?} 的头还在"
        );
    }
}

/// 场景：清扫交给后台（开关开着），挂载之后后台那一轮只拿到池里一块盘。
/// 预期：挂载末尾不扫；后台那一轮设备不全，一行都不删（结局报盘数）。
#[test]
fn the_background_sweep_with_a_device_missing_deletes_no_row() {
    let pool = pool_with_a_dead_instance(None);
    let (mut mounted, devices, _stream) = sweeping_mount_on(
        &pool.image,
        InstanceTableSweepConfiguration {
            switch: InstanceTableSweepSwitch::HandedToTheBackgroundSweepWhileMounted,
            row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
            sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
        },
    );
    assert!(
        matches!(
            mounted.output.instance_table_sweep,
            InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted
        ),
        "开关开着：挂载末尾不扫"
    );
    let mut only_the_first_device: Vec<_> = devices.into_iter().take(1).collect();
    let outcome = sweep_the_instance_table_while_mounted(
        &parameters_of_the_ring_hole_pools(),
        &mut only_the_first_device,
        &mut mounted.allocator,
        &mut mounted.current,
        SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
    );
    assert!(
        matches!(
            outcome,
            InstanceTableSweepOutcome::NoRowDeleted(
                NoRowDeletedThisRound::NotEveryDeviceOfThePoolIsPresent {
                    devices_present: 1,
                    devices_of_the_pool: 2
                }
            )
        ),
        "设备不全不删：{outcome:?}"
    );
}

/// 场景：清扫交给后台。后台那一轮先判（单子上是孤儿），判完到抹之间实例 3 接着覆盖写，直到单子上有一个落点被现行那一版的数据单元占了，再抹、发。
/// 预期：抹之前那一遍重读到那一槽的头已经不是判的那个单元：这一轮不删、一个头都不抹（C，用户 2026-10-04 定：读回的不是那个头这一轮都不删；
/// 原来 F1 是那一槽不抹、单子上别的照抹、照删行）；实例 1 那一行还在；现行那一版的内容读得回来、那几条不变量不红。
#[test]
fn the_background_sweep_rereads_every_head_before_erasing_and_deletes_no_row_when_a_slot_was_allocated_after_the_judgement(
) {
    let pool = pool_with_a_dead_instance(None);
    let (mut mounted, mut devices, stream) = sweeping_mount_on(
        &pool.image,
        InstanceTableSweepConfiguration {
            switch: InstanceTableSweepSwitch::HandedToTheBackgroundSweepWhileMounted,
            row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
            sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
        },
    );
    let judgement =
        judge_the_instance_table_sweep(&devices, mounted.current.root()).expect("后台那一轮判得出");
    let judged: BTreeSet<PlacementOfAUnitHead> = judgement
        .heads_to_erase
        .iter()
        .map(|to_erase| to_erase.placement)
        .collect();
    assert_eq!(
        judged,
        placements_of(&pool.orphans_left_on_disk),
        "单子上是盘上留着的孤儿"
    );
    let mut current = mounted
        .current
        .file_version()
        .expect("这一版带文件")
        .clone();
    let mut reused = None;
    let mut last_content = Vec::new();
    // 迭代上界：至多 OVERWRITES_AFTER_THE_JUDGEMENT_AT_MOST 次覆盖写；跨轮携带的是现行那一版；提前出口是单子上有一个落点的头被盖掉了。
    for step in 0..OVERWRITES_AFTER_THE_JUDGEMENT_AT_MOST {
        last_content = content_of_the_version(300 + step);
        current = publish_an_overwrite(
            &mut devices,
            &mut mounted.allocator,
            &current,
            &last_content,
            InstanceGeneration(3),
        );
        reused = judgement
            .heads_to_erase
            .iter()
            .find(|to_erase| {
                let bytes = PoolReader::read(
                    &devices,
                    to_erase.placement.device,
                    to_erase.placement.slot.to_device_offset(),
                    1024,
                )
                .expect("内存盘上读得出");
                unit_head_read_by_sweep(&bytes) != Some(to_erase.head)
            })
            .map(|to_erase| to_erase.placement);
        if reused.is_some() {
            break;
        }
    }
    let reused = reused.expect("前提：判完之后的覆盖写盖掉了单子上一个落点的头");
    let mut current_version = PoolVersion::of_a_version_of_trees(current);
    let outcome = erase_the_heads_and_publish_the_sweep(
        &parameters_of_the_ring_hole_pools(),
        &mut devices,
        &mut mounted.allocator,
        &mut current_version,
        &judgement,
        SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
    );
    assert!(
        matches!(
            outcome,
            InstanceTableSweepOutcome::NoRowDeleted(
                NoRowDeletedThisRound::AHeadRereadBeforeErasingIsNotTheJudgedUnit {
                    placement,
                    reread: HeadRereadBeforeErasing::NotTheJudgedUnit,
                }
            ) if placement == reused
        ),
        "抹之前那一遍重读到那一槽已不是判的那个头：这一轮不删（C）：{outcome:?}"
    );
    let after = image_after(&pool.image, &stream);
    assert_eq!(
        instances_with_a_row_on(&after),
        vec![InstanceGeneration(1), InstanceGeneration(2)],
        "实例 1 那一行还在"
    );
    // 单子上的落点要么还是判的那个孤儿头、要么已被实例 3 后来的写占了（认得出头），没有一个被抹成认不出头。
    for to_erase in &judgement.heads_to_erase {
        assert!(
            unit_head_read_by_sweep(&head_bytes_at(&after, to_erase.placement)).is_some(),
            "单子上的 {:?} 一个都没抹",
            to_erase.placement
        );
    }
    assert!(
        matches!(
            &recover(&after, JournalPolicy::Consult).outcome,
            RecoveryOutcome::FileRead { content, .. } if *content == last_content
        ),
        "现行那一版的内容读得回来"
    );
    assert_eq!(violations_of(&after), Vec::<String>::new(), "清扫之后");
}

/// 清扫那次挂载里，第一个孤儿所在的槽判的那一次读照读、第 2 次起（抹之前那一遍重读）按 `read_back` 读坏；
/// 交回清扫的结局与挂载之后的盘面。
fn sweep_with_the_reread_of_the_first_orphan_read_bad(
    pool: &PoolWithADeadInstance,
    read_back: UnreadableRangeReadBack,
) -> (InstanceTableSweepAtTheEndOfTheMount, MemoryPool) {
    let (first_orphan, _) = *pool
        .orphans_left_on_disk
        .first()
        .expect("前提：盘上还留着实例 1 的孤儿");
    let unreadable = SharedUnreadableRanges::new(
        vec![UnreadableRange {
            device: first_orphan.device,
            offset_in_bytes: first_orphan.slot.to_device_offset().0,
            length_in_bytes: 512,
            failing_reads: FailingReadsOfARange::FromTheNthOnward(2),
        }],
        read_back,
    );
    let stream = SharedStream::retaining_contents();
    let mut devices = with_unreadable_ranges(recording_of(&pool.image, &stream), &unreadable);
    let mounted = mount_writable_with_instance_table_sweep_configuration(
        &parameters_of_the_ring_hole_pools(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        sweeping_at_the_end_of_the_mount_on_any_row(),
    )
    .expect("读故障只在清扫读的那一槽：挂载照样做成");
    (
        mounted.output.instance_table_sweep,
        image_after(&pool.image, &stream),
    )
}

/// 实例 1 的孤儿里头还在盘上的那几个。
fn orphans_of_instance_one_still_on(
    pool: &PoolWithADeadInstance,
    image: &MemoryPool,
) -> BTreeSet<PlacementOfAUnitHead> {
    pool.orphans_left_on_disk
        .iter()
        .filter(|(placement, head)| {
            unit_head_read_by_sweep(&head_bytes_at(image, *placement)) == Some(*head)
        })
        .map(|(placement, _)| *placement)
        .collect()
}

/// 抹之前那一遍重读读坏（C，用户 2026-10-04 定，`research/prompts/m3-root-ring-code-r3-main-verification.md` 第二节 K1-b 与第六节 Q2；
/// 攻方 `k1-reread` 那两形）：这一轮不删、一个头都不抹；同一份盘面上不注入读错再扫一次，删行生效、实例 1 的孤儿一个不剩。
/// 改之前那一槽当「已不是那个单元」不抹、照删行，删行之后实例 1 按删前那一行判未发布的孤儿剩 1 个（攻方 `results/reread.log` 的 nth=2 两行）。
fn the_reread_read_bad_deletes_no_row_and_the_next_sweep_leaves_no_orphan(
    read_back: UnreadableRangeReadBack,
    reread_seen: HeadRereadBeforeErasing,
) {
    let pool = pool_with_a_dead_instance(None);
    let (first_orphan, _) = *pool
        .orphans_left_on_disk
        .first()
        .expect("前提：盘上还留着实例 1 的孤儿");
    let (sweep, after_the_faulty_sweep) =
        sweep_with_the_reread_of_the_first_orphan_read_bad(&pool, read_back);
    assert!(
        matches!(
            sweep,
            InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::NoRowDeleted(
                NoRowDeletedThisRound::AHeadRereadBeforeErasingIsNotTheJudgedUnit {
                    placement,
                    reread,
                }
            )) if placement == first_orphan && reread == reread_seen
        ),
        "抹之前那一遍重读读坏：这一轮不删：{sweep:?}"
    );
    assert!(
        instances_with_a_row_on(&after_the_faulty_sweep).contains(&InstanceGeneration(1)),
        "实例 1 那一行还在"
    );
    assert_eq!(
        orphans_of_instance_one_still_on(&pool, &after_the_faulty_sweep),
        placements_of(&pool.orphans_left_on_disk),
        "一个头都没抹"
    );
    let (mounted, _devices, stream) = sweeping_mount_on(
        &after_the_faulty_sweep,
        sweeping_at_the_end_of_the_mount_on_any_row(),
    );
    let next_sweep = mounted.output.instance_table_sweep;
    assert!(
        matches!(
            &next_sweep,
            InstanceTableSweepAtTheEndOfTheMount::Swept(
                InstanceTableSweepOutcome::RowsDeleted { .. }
            )
        ),
        "不注入读错的下一次清扫删行：{next_sweep:?}"
    );
    let after_the_next_sweep = image_after(&after_the_faulty_sweep, &stream);
    assert!(
        !instances_with_a_row_on(&after_the_next_sweep).contains(&InstanceGeneration(1)),
        "实例 1 那一行删了"
    );
    assert_eq!(
        orphans_of_instance_one_still_on(&pool, &after_the_next_sweep),
        BTreeSet::<PlacementOfAUnitHead>::new(),
        "删行生效之后实例 1 的孤儿一个不剩"
    );
    assert_eq!(
        violations_of(&after_the_next_sweep),
        Vec::<String>::new(),
        "清扫之后"
    );
}

/// 场景：抹之前那一遍重读第一个孤儿那一槽报设备错。
/// 预期：这一轮不删（报 `ReadFailed`）、一个头都不抹；下一次不注入读错的清扫删行、实例 1 的孤儿一个不剩。
#[test]
fn a_device_error_on_the_reread_before_erasing_deletes_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance(
) {
    the_reread_read_bad_deletes_no_row_and_the_next_sweep_leaves_no_orphan(
        UnreadableRangeReadBack::DeviceError,
        HeadRereadBeforeErasing::ReadFailed,
    );
}

/// 场景：抹之前那一遍重读第一个孤儿那一槽报成功、字节全 0（认不出头）。
/// 预期：这一轮不删（报 `NotTheJudgedUnit`）、一个头都不抹；下一次不注入读错的清扫删行、实例 1 的孤儿一个不剩。
#[test]
fn zeros_read_back_on_the_reread_before_erasing_delete_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance(
) {
    the_reread_read_bad_deletes_no_row_and_the_next_sweep_leaves_no_orphan(
        UnreadableRangeReadBack::Zeros,
        HeadRereadBeforeErasing::NotTheJudgedUnit,
    );
}

/// 挂载一次（开关关着、有一行就扫），之后覆盖写 `overwrites` 次；`unmounted` 为真时正常卸载，为假时不卸载就退出（崩在最后一次发布之后）。
/// 交回清扫之后现行那一版的实例表行数与挂载之后的盘面。
fn one_mount_of_the_periodic_schedule(
    image: &MemoryPool,
    overwrites: u64,
    unmounted: bool,
) -> (usize, MemoryPool) {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = devices_of(image);
    let mounted = mount_writable_with_instance_table_sweep_configuration(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        sweeping_at_the_end_of_the_mount_on_any_row(),
    )
    .expect("可写挂载");
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator;
    let mut current = mounted.current;
    let rows_after_the_sweep = instance_table_chain_of_root(&pool_of(&devices), current.root())
        .expect("现行那一版的实例表")
        .records
        .rows
        .len();
    if matches!(current, PoolVersion::WithoutFile(_)) {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let first_file = publish_first_file(
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
        .expect("第一个文件");
        current = PoolVersion::of_a_version_of_trees(first_file);
    }
    // 迭代上界：`overwrites` 次覆盖写；跨轮携带的是现行那一版。
    for _ in 0..overwrites {
        let previous = current.into_file_version().expect("带文件");
        let next = publish_an_overwrite(
            &mut devices,
            &mut allocator,
            &previous,
            &content_of_the_version(previous.root.checkpoint_txg.0 + 1),
            instance,
        );
        current = PoolVersion::of_a_version_of_trees(next);
    }
    if unmounted {
        unmount(
            &parameters,
            &mut devices,
            &mut allocator,
            &mut current,
            ShadowLedger::On,
        )
        .expect("正常卸载");
    }
    (rows_after_the_sweep, pool_of(&devices))
}

/// 一次挂载之后实例表至多几行：根环 12 槽里住着最近几次挂载的根（一次正常卸载的挂载推 5 次左右的发布、一次不卸载的推 3 次左右），
/// 根还在环里的实例与这一次挂载自己的那一行加起来不过这个数。
const INSTANCE_ROWS_AT_MOST_ON_THE_PERIODIC_SCHEDULE: usize = 4;
/// 周期调度挂几次（攻方 `k1_periodic` 的默认值）。
const MOUNTS_OF_THE_PERIODIC_SCHEDULE: usize = 60;
/// 每隔几次挂载跑一次池级 checker（攻方 `PROBE_CHECK_EVERY = 5`）。
const MOUNTS_BETWEEN_CHECKS_OF_THE_PERIODIC_SCHEDULE: usize = 5;

/// 场景：攻方 `k1_periodic` 那一段无故障合法历史（A2，用户 2026-10-04 定，`research/prompts/m3-root-ring-code-r3-main-verification.md` 第二节 K1-a
/// 与第六节 Q1）：mkfs 之后第一次挂载写第一个文件、正常卸载；之后每次挂载覆盖写 1 次，正常卸载与不卸载交替，共 60 次挂载，每次挂载末尾有一行就扫。
/// 正常卸载把 F 抬到现行那一版，F 之下的旧根引用的分配记录树节点被合法复用、那条根的账读不出。
/// 预期：清扫算账时 txg 低于现行 F 的根跳过它的账，每次挂载之后实例表的行数不超过
/// [`INSTANCE_ROWS_AT_MOST_ON_THE_PERIODIC_SCHEDULE`]；每 5 次挂载与最后一次挂载之后那几条不变量不红。
/// 改之前每一次清扫都停在 `AccountOfARingRootUnreadable`，60 次挂载之后 59 行、一行没删（攻方 `results/long-60.log`）。
#[test]
fn sweeping_a_periodic_schedule_of_unmounted_and_crashed_mounts_keeps_the_instance_table_short() {
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
    let mut image = pool_of(&devices);
    // 迭代上界：60 次挂载；跨轮携带的是上一次挂载之后的盘面。
    for mount_index in 0..MOUNTS_OF_THE_PERIODIC_SCHEDULE {
        let (overwrites, unmounted) = if mount_index == 0 {
            (0, true)
        } else {
            (1, mount_index % 2 == 1)
        };
        let (rows_after_the_sweep, after) =
            one_mount_of_the_periodic_schedule(&image, overwrites, unmounted);
        assert!(
            rows_after_the_sweep <= INSTANCE_ROWS_AT_MOST_ON_THE_PERIODIC_SCHEDULE,
            "第 {mount_index} 次挂载清扫之后实例表 {rows_after_the_sweep} 行"
        );
        if mount_index % MOUNTS_BETWEEN_CHECKS_OF_THE_PERIODIC_SCHEDULE == 0
            || mount_index + 1 == MOUNTS_OF_THE_PERIODIC_SCHEDULE
        {
            assert_eq!(
                violations_of(&after),
                Vec::<String>::new(),
                "第 {mount_index} 次挂载之后"
            );
        }
        image = after;
    }
}

/// 场景：清扫那次挂载用只供测试的落点开关：清扫发布在一份每个空闲槽都被占掉的分配器拷贝上取落点（真的取落点那一段，被拒）。
/// 预期：清扫发布被落点拒，这一轮不删行（实例 1 那一行还在）；挂载照样交回可写——接着在交回的分配器上覆盖写一次做成。
#[test]
fn a_sweep_publish_refused_for_its_placement_deletes_no_row_and_the_mount_stays_writable() {
    let pool = pool_with_a_dead_instance(None);
    let (mut mounted, mut devices, stream) = sweeping_mount_on(
        &pool.image,
        InstanceTableSweepConfiguration {
            switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
            row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
            sweep_publish_placements:
                SweepPublishPlacements::EveryFreeSlotTakenAwayByTheTestOnlySwitch,
        },
    );
    assert!(
        matches!(
            mounted.output.instance_table_sweep,
            InstanceTableSweepAtTheEndOfTheMount::Swept(
                InstanceTableSweepOutcome::SweepPublishRefusedForSpace { .. }
            )
        ),
        "清扫发布被落点拒：{:?}",
        mounted.output.instance_table_sweep
    );
    assert_eq!(
        instances_with_a_row_on(&image_after(&pool.image, &stream)),
        vec![InstanceGeneration(1), InstanceGeneration(2)],
        "这一轮不删行"
    );
    let current = mounted
        .current
        .file_version()
        .expect("这一版带文件")
        .clone();
    publish_an_overwrite(
        &mut devices,
        &mut mounted.allocator,
        &current,
        &content_of_the_version(400),
        InstanceGeneration(3),
    );
}

/// 场景：产品路径的配置（开关关着、阈值取占位 369）：实例表两行，没超过阈值。
/// 预期：挂载末尾不扫（结局报行数与阈值），孤儿头都还在。
#[test]
fn the_product_configuration_does_not_sweep_a_table_at_or_below_the_threshold() {
    let pool = pool_with_a_dead_instance(None);
    let (mounted, _devices, stream) =
        sweeping_mount_on(&pool.image, InstanceTableSweepConfiguration::PRODUCT);
    assert!(
        matches!(
            mounted.output.instance_table_sweep,
            InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold {
                rows: 2,
                threshold: InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690
            }
        ),
        "没超过阈值不扫：{:?}",
        mounted.output.instance_table_sweep
    );
    let after = image_after(&pool.image, &stream);
    for (placement, head) in &pool.orphans_left_on_disk {
        assert_eq!(
            unit_head_read_by_sweep(&head_bytes_at(&after, *placement)),
            Some(*head),
            "孤儿 {placement:?} 的头还在"
        );
    }
}

/// 场景：mkfs 之后一直不写文件、接连可写挂载七次（实例 1 的根轮出了 12 槽的环），第八次挂载开关关着、有一行就扫。
/// 预期：现行那一版树表 0 条，第一版不清扫：结局报这个成员；这次挂载写下的盘面与不扫的同一次挂载逐字节相同（清扫一个字节都没写）。
#[test]
fn a_sweep_on_a_version_without_file_is_not_supported_and_writes_nothing() {
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
    // 迭代上界：七次可写挂载，每次写行与暖机两三条根。
    for _ in 0..7 {
        mount_writable(&parameters, &mut devices).expect("树表 0 条的池可写挂载");
    }
    let image = pool_of(&devices);
    let (swept, _devices_of_the_swept_mount, swept_stream) =
        sweeping_mount_on(&image, sweeping_at_the_end_of_the_mount_on_any_row());
    assert!(
        matches!(
            swept.output.instance_table_sweep,
            InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::NoRowDeleted(
                NoRowDeletedThisRound::SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion
            ))
        ),
        "树表 0 条的一版上不清扫：{:?}",
        swept.output.instance_table_sweep
    );
    let (_not_swept, _devices_of_the_mount_not_swept, not_swept_stream) =
        sweeping_mount_on(&image, InstanceTableSweepConfiguration::PRODUCT);
    assert!(
        image_after(&image, &swept_stream) == image_after(&image, &not_swept_stream),
        "清扫那一判一个字节都没写"
    );
    assert_eq!(
        swept_stream.operations().len(),
        not_swept_stream.operations().len(),
        "录制流一样长"
    );
}
