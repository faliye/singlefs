//! 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条）。
//!
//! 盘面（S = 4 的 12 槽小环、两块 128 MiB 内存盘）：mkfs → 实例 1 第一个文件、覆盖写两次（txg 3、4、5）→ 一次十六个数据单元的顺序写崩在第一条 journal
//! 记录写之前（单元写落了：实例 1 按它那一行判未发布的孤儿）→ 实例 2 可写挂载（写行 6、暖机 7）→ 实例 2 覆盖写到 txg 19（根环里没有实例 1 的根了）。
//! 接着的可写挂载（实例 3）末尾判一次清扫。
mod common;
mod common_images_for_the_given_states_engine;
mod common_root_ring_holes;
#[path = "common_images_for_the_given_states_engine/instance_table_sweep_at_the_end_of_a_writable_mount.rs"]
mod scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount;

use common::{
    with_unreadable_ranges, without_unreadable_ranges, FailingReadsOfARange,
    SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
};
use common_images_for_the_given_states_engine::PoolCheckerImages;
use common_root_ring_holes::{
    content_of_the_version, devices_of, fixed_geometry_of_the_ring_hole_pools,
    parameters_of_the_ring_hole_pools, pool_of, recording_of, RingHoleDevices,
    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::{
    head_bytes_at, image_after_the_stream, instances_with_a_row_on, pool_with_a_dead_instance,
    publish_an_overwrite, sweeping_at_the_end_of_the_mount_on_any_row, sweeping_mount_on,
};
use singlefs_core::address::{DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::admission::{
    instance_switch_reserve_on_one_device, BytesOnOneDevice, MetadataBlocks, SpaceAdmission,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table_sweep::{
    sweep_the_instance_table_while_mounted, unit_head_read_by_sweep,
    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
    NoRowDeletedThisRound, PlacementOfAUnitHead, SweepPublishPlacements,
};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_instance_table_sweep_configuration, unmount, MountError,
    Mounted, ShadowLedger, StillUnreadableAfterOneReread,
};
use singlefs_core::recovery::instance_table_chain_of_root;
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, RootRingSlot};
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolVersion, PoolWriter};
use singlefs_verification::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
use singlefs_verification::segments::StepKind;
use singlefs_verification::SharedStream;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::run_the_scenario_where_the_row_of_a_dead_instance_is_deleted_after_its_roots_left_the_ring_and_its_unpublished_heads_are_erased`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_row_of_a_dead_instance_is_deleted_after_its_roots_left_the_ring_and_its_unpublished_heads_are_erased(
) {
    scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::run_the_scenario_where_the_row_of_a_dead_instance_is_deleted_after_its_roots_left_the_ring_and_its_unpublished_heads_are_erased(&mut PoolCheckerImages::left_to_the_given_states_engine());
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
        head_bytes_at(&image_after_the_stream(&pool.image, &stream), second_slot),
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
        head_bytes_at(&image_after_the_stream(&pool.image, &stream), second_slot),
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
    let after = image_after_the_stream(&pool.image, &stream);
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::run_the_scenario_where_the_background_sweep_rereads_every_head_before_erasing_and_deletes_no_row_when_a_slot_was_allocated_after_the_judgement`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_background_sweep_rereads_every_head_before_erasing_and_deletes_no_row_when_a_slot_was_allocated_after_the_judgement(
) {
    scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::run_the_scenario_where_the_background_sweep_rereads_every_head_before_erasing_and_deletes_no_row_when_a_slot_was_allocated_after_the_judgement(&mut PoolCheckerImages::left_to_the_given_states_engine());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::run_the_scenario_where_a_device_error_on_the_reread_before_erasing_deletes_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_device_error_on_the_reread_before_erasing_deletes_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance(
) {
    scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::run_the_scenario_where_a_device_error_on_the_reread_before_erasing_deletes_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance(&mut PoolCheckerImages::left_to_the_given_states_engine());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::run_the_scenario_where_zeros_read_back_on_the_reread_before_erasing_delete_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn zeros_read_back_on_the_reread_before_erasing_delete_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance(
) {
    scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::run_the_scenario_where_zeros_read_back_on_the_reread_before_erasing_delete_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance(&mut PoolCheckerImages::left_to_the_given_states_engine());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::run_the_scenario_where_sweeping_a_periodic_schedule_of_unmounted_and_crashed_mounts_keeps_the_instance_table_short`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn sweeping_a_periodic_schedule_of_unmounted_and_crashed_mounts_keeps_the_instance_table_short() {
    scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::run_the_scenario_where_sweeping_a_periodic_schedule_of_unmounted_and_crashed_mounts_keeps_the_instance_table_short(&mut PoolCheckerImages::left_to_the_given_states_engine());
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
        instances_with_a_row_on(&image_after_the_stream(&pool.image, &stream)),
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

/// 场景：产品路径的配置（开关关着、阈值取默认的 365 行）：实例表两行，没超过阈值。
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
                threshold: InstanceTableRowCountThreshold::DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE
            }
        ),
        "没超过阈值不扫：{:?}",
        mounted.output.instance_table_sweep
    );
    let after = image_after_the_stream(&pool.image, &stream);
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
        image_after_the_stream(&image, &swept_stream)
            == image_after_the_stream(&image, &not_swept_stream),
        "清扫那一判一个字节都没写"
    );
    assert_eq!(
        swept_stream.operations().len(),
        not_swept_stream.operations().len(),
        "录制流一样长"
    );
}

/// 产品路径的阈值那一条至多崩了再挂几次（mkfs 那个实例不写行、之后每次挂载写一行：第 365 次崩了再挂之后 365 行）。
const CRASH_REMOUNTS_UNTIL_THE_ROWS_REACH_THE_DEFAULT_THRESHOLD_AT_MOST: u64 = 400;

/// 一次录下来的崩了再挂：挂载交回的、它发的写里每一条根（按写的次序），与它之后的盘面。
struct RecordedCrashRemount {
    mounted: Mounted,
    roots_written: Vec<RootRecord>,
    image_after_the_mount: MemoryPool,
}

/// 在 `image` 上录一次产品路径的可写挂载（之后就当崩了，不卸载）。
fn record_a_crash_remount(image: &MemoryPool) -> RecordedCrashRemount {
    let parameters = parameters_of_the_ring_hole_pools();
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(image, &stream);
    let mounted = mount_writable(&parameters, &mut recording).expect("录下来的那次崩了再挂");
    let (writes, _segments) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    let roots_written = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .map(|write| {
            RootRecord::parse_slot(
                write.bytes().expect("根槽写带字节"),
                &parameters.filesystem_identifier,
            )
            .expect("挂载写的根自证得过")
        })
        .collect();
    let mut image_after_the_mount = image.clone();
    image_after_the_mount.apply_writes(&writes);
    RecordedCrashRemount {
        mounted,
        roots_written,
        image_after_the_mount,
    }
}

/// 一条根指着的实例表有几行（读那一刻的盘面）。
fn rows_of_the_instance_table_of(image: &MemoryPool, root: &RootRecord) -> usize {
    instance_table_chain_of_root(image, root)
        .expect("根指着的实例表读得出")
        .records
        .rows
        .len()
}

/// 这次挂载的实例切换预留（每块盘；只看链重写那一半随行数怎么变，暖机那一半与行数无关、取 0）。
fn switch_reserve_of(mounted: &Mounted) -> BytesOnOneDevice {
    instance_switch_reserve_on_one_device(
        mounted
            .allocator
            .instance_rows_after_this_mounts_row_publish(),
        MetadataBlocks(0),
    )
}

/// 产品路径的阈值（365 行，C690（清扫触发阈值的默认值没定））。
/// 场景：环洞池 mkfs → 实例 1 第一个文件 → 产品路径的配置接连崩了再挂（每次挂载写一行），直到某次挂载之后实例表 364 行；
/// 录下接着的两次崩了再挂（写到 365 行、写到 366 行）。
/// 预期：到 365 行为止的每一次挂载末尾都不扫，结局报行数与默认阈值；365 行那一次录下的写里最后一条根就是现行那一版，实例表 365 行；
/// 366 行那一次挂载末尾清扫、删了行，录下的写比 365 行那一次多一条根：多的是最后那一条清扫发布，它的实例表是 366 行减去删掉的行，
/// 它前面那一条根的实例表是 366 行。整段历史里每一次挂载的实例切换预留都按一片留（行数项 rows0 + 3 不越过一片的 369 行，
/// D28（挂载期承诺量） 已定项 3），与第一次崩了再挂相同。
/// 判别力：阈值判断拿掉、每次都扫时第一次崩了再挂就扫；拿掉、永不扫时 366 行那一次不扫；阈值退回 369 时 366 行那一次不扫。
#[test]
fn the_product_threshold_sweeps_on_the_mount_whose_rows_pass_three_hundred_sixty_five_and_on_no_mount_before(
) {
    let parameters = parameters_of_the_ring_hole_pools();
    let threshold = InstanceTableRowCountThreshold::DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE;
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
    let mut switch_reserve_of_the_first_crash_remount = None;
    // 迭代上界：CRASH_REMOUNTS_UNTIL_THE_ROWS_REACH_THE_DEFAULT_THRESHOLD_AT_MOST 次崩了再挂；跨轮携带的是盘与第一次崩了再挂的切换预留；
    // 提前出口是实例表到了阈值减一那一次。
    for crash_remount_index in 0..CRASH_REMOUNTS_UNTIL_THE_ROWS_REACH_THE_DEFAULT_THRESHOLD_AT_MOST
    {
        let mounted = mount_writable(&parameters, &mut devices)
            .unwrap_or_else(|error| panic!("第 {crash_remount_index} 次崩了再挂做成：{error:?}"));
        let switch_reserve = switch_reserve_of(&mounted);
        let first_switch_reserve =
            *switch_reserve_of_the_first_crash_remount.get_or_insert(switch_reserve);
        assert_eq!(
            switch_reserve, first_switch_reserve,
            "第 {crash_remount_index} 次崩了再挂的切换预留按一片留"
        );
        let InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold {
            rows,
            threshold: seen,
        } = mounted.output.instance_table_sweep
        else {
            panic!(
                "没到阈值的挂载都不扫（第 {crash_remount_index} 次）：{:?}",
                mounted.output.instance_table_sweep
            );
        };
        assert_eq!(seen, threshold, "结局报的是产品路径的默认阈值");
        if rows == threshold.0 - 1 {
            break;
        }
    }
    let first_switch_reserve = switch_reserve_of_the_first_crash_remount.expect("至少崩了再挂一次");

    let at_the_threshold = record_a_crash_remount(&pool_of(&devices));
    assert!(
        matches!(
            at_the_threshold.mounted.output.instance_table_sweep,
            InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { rows: 365, threshold: seen }
                if seen == threshold
        ),
        "365 行那一次挂载没超过阈值、不扫：{:?}",
        at_the_threshold.mounted.output.instance_table_sweep
    );
    let last_root_at_the_threshold = at_the_threshold
        .roots_written
        .last()
        .expect("365 行那一次挂载写了根");
    assert_eq!(
        last_root_at_the_threshold,
        at_the_threshold.mounted.current.root(),
        "365 行那一次录下的最后一条根就是现行那一版（后面没有清扫发布）"
    );
    assert_eq!(
        rows_of_the_instance_table_of(
            &at_the_threshold.image_after_the_mount,
            last_root_at_the_threshold
        ),
        365,
        "365 行那一次最后一条根的实例表一行没删"
    );
    assert_eq!(
        switch_reserve_of(&at_the_threshold.mounted),
        first_switch_reserve,
        "365 行那一次挂载的切换预留按一片留"
    );

    let past_the_threshold = record_a_crash_remount(&at_the_threshold.image_after_the_mount);
    let InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
        rows_deleted,
        ..
    }) = &past_the_threshold.mounted.output.instance_table_sweep
    else {
        panic!(
            "366 行那一次挂载越过阈值、末尾清扫删了行：{:?}",
            past_the_threshold.mounted.output.instance_table_sweep
        );
    };
    assert!(!rows_deleted.is_empty(), "清扫删了行");
    assert_eq!(
        past_the_threshold.roots_written.len(),
        at_the_threshold.roots_written.len() + 1,
        "越过阈值那一次比 365 行那一次多写一条根（清扫发布）"
    );
    let [.., root_before_the_sweep_publish, sweep_publish_root] =
        &past_the_threshold.roots_written[..]
    else {
        panic!("越过阈值那一次至少写了两条根（写行那一次与清扫发布）");
    };
    assert_eq!(
        sweep_publish_root,
        past_the_threshold.mounted.current.root(),
        "录下的最后一条根是清扫发布那一版"
    );
    assert_eq!(
        rows_of_the_instance_table_of(
            &past_the_threshold.image_after_the_mount,
            root_before_the_sweep_publish
        ),
        366,
        "清扫发布前面那一条根的实例表 366 行"
    );
    assert_eq!(
        rows_of_the_instance_table_of(
            &past_the_threshold.image_after_the_mount,
            sweep_publish_root
        ),
        366 - rows_deleted.len(),
        "清扫发布那一版的实例表少了删掉的行"
    );
    assert_eq!(
        switch_reserve_of(&past_the_threshold.mounted),
        first_switch_reserve,
        "366 行那一次挂载的切换预留仍按一片留（rows0 + 3 = 369）"
    );
}

/// 读不出那一形接连挂几次（`research/prompts/rot-decisions-r1-opus-output.md`「要主 agent 跑的命令」一节给的 8 次）。
const MOUNTS_WITH_ONE_ROOT_RING_SLOT_UNREADABLE: usize = 8;

/// 读坏的那一个根环槽：(区域 2, 槽 3)，区域 2 住盘 0（第一版区域归属 0 / 1 / 0）。mkfs 与第一次挂载的那几条根落不到这一槽
/// （第一次挂载之前就读坏它，择根照样择得到）；接连挂的那几次里根轮到它，写照落、读每次都报设备错。
const UNREADABLE_ROOT_RING_SLOT: RootRingSlot = RootRingSlot { region: 2, slot: 3 };

/// 一次挂载之后看到的：挂载末尾清扫的结局，与清扫之后现行那一版的实例表行数。
struct MountWithAWriteAndAnUnmount {
    sweep: InstanceTableSweepAtTheEndOfTheMount,
    rows_after_the_sweep: usize,
}

/// mkfs 之后接连 [`MOUNTS_WITH_ONE_ROOT_RING_SLOT_UNREADABLE`] 次「可写挂载（有一行就扫）→ 写一个文件（树表 0 条就写第一个文件，否则覆盖写）
/// → 正常卸载」，每次盘外面都包上同一份读故障 `ranges`；交回每次挂载看到的。读坏的那一槽这次挂载里写过根时正常卸载被拒（下面那一处注释），
/// 盘面留成崩在写文件之后。
fn mounts_with_a_write_and_an_unmount_each(
    ranges: &SharedUnreadableRanges,
) -> Vec<MountWithAWriteAndAnUnmount> {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices_after_make_filesystem: RingHoleDevices = (0..2u32)
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
    make_filesystem(&parameters, &mut devices_after_make_filesystem).expect("mkfs");
    let mut image = pool_of(&devices_after_make_filesystem);
    let mut seen = Vec::new();
    // 迭代上界：MOUNTS_WITH_ONE_ROOT_RING_SLOT_UNREADABLE 次挂载；跨轮携带的是上一次卸载之后的盘面。
    for mount_index in 0..MOUNTS_WITH_ONE_ROOT_RING_SLOT_UNREADABLE {
        let mut devices = with_unreadable_ranges(devices_of(&image), ranges);
        let mounted = mount_writable_with_instance_table_sweep_configuration(
            &parameters,
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            sweeping_at_the_end_of_the_mount_on_any_row(),
        )
        .unwrap_or_else(|error| panic!("第 {mount_index} 次可写挂载做成：{error:?}"));
        let instance = mounted.output.instance;
        let mut allocator = mounted.allocator;
        let mut current = mounted.current;
        let rows_after_the_sweep = instance_table_chain_of_root(&devices, current.root())
            .expect("现行那一版的实例表读得出")
            .records
            .rows
            .len();
        let written = match &current {
            PoolVersion::WithoutFile(_) => {
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
            }
            PoolVersion::WithFile(previous) | PoolVersion::WithTreesAndNoFile(previous) => {
                let previous = previous.clone();
                publish_an_overwrite(
                    &mut devices,
                    &mut allocator,
                    &previous,
                    &content_of_the_version(previous.root.checkpoint_txg.0 + 1),
                    instance,
                )
            }
        };
        current = PoolVersion::of_a_version_of_trees(written);
        // 正常卸载抬 F 时重读这个进程写过根的槽（D16（发布语义） 已定项 1「根槽这一次读坏」）：读坏的那一槽这次挂载里写过根，卸载就拒、
        // 一个字节都不写，盘面留成崩在写文件之后（下一次挂载照恢复走）；别的拒绝都不该有。
        match unmount(
            &parameters,
            &mut devices,
            &mut allocator,
            &mut current,
            ShadowLedger::On,
        ) {
            Ok(_) => {}
            Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable))
                if matches!(
                    *still_unreadable,
                    StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { ring_slot, .. }
                        if ring_slot == UNREADABLE_ROOT_RING_SLOT
                ) => {}
            Err(error) => panic!("第 {mount_index} 次正常卸载只会因为读坏的那一槽被拒：{error:?}"),
        }
        image = pool_of(&without_unreadable_ranges(devices));
        seen.push(MountWithAWriteAndAnUnmount {
            sweep: mounted.output.instance_table_sweep,
            rows_after_the_sweep,
        });
    }
    seen
}

/// 读坏的那一槽在盘上的那一段：`failing_reads` 给哪几次读坏，读坏时报设备错。
fn unreadable_root_ring_slot(failing_reads: FailingReadsOfARange) -> SharedUnreadableRanges {
    let parameters = parameters_of_the_ring_hole_pools();
    SharedUnreadableRanges::new(
        vec![UnreadableRange {
            device: parameters.region_devices
                [usize::try_from(UNREADABLE_ROOT_RING_SLOT.region).expect("区域号")],
            offset_in_bytes: slot_offset(
                UNREADABLE_ROOT_RING_SLOT,
                parameters.geometry.fixed_structure_slot_spacing,
            )
            .0,
            length_in_bytes: u64::from(parameters.geometry.physical_block_size),
            failing_reads,
        }],
        UnreadableRangeReadBack::DeviceError,
    )
}

/// 场景：根环 (区域 2, 槽 3) 每次读都报设备错（`research/prompts/rot-decisions-r1-opus-output.md`「要主 agent 跑的命令」一节；D18（块里携带什么信息）
/// 已定项 11「回收」一条：根环每个槽都读成功才算「没有」，读不出的槽按「可能有该实例的根」算）。mkfs 之后接连 8 次「可写挂载（有一行就扫）→ 写一个文件
/// → 正常卸载」；那一槽在这次挂载里写过根时正常卸载被拒（`RootRingSlotKnownToHoldARoot`），那一次照崩在写文件之后往下挂。
/// 预期：每次清扫都停在那一槽：结局报 `NoRowDeleted(ARootRingSlotIsUnreadableOrAllZero)`、点名那一槽；清扫之后的实例表行数一次都不降
/// （每次可写挂载写一行，行只增不删——C597（实例表行只增不删，删行的条件没有条款） 盯着的那笔代价）。对照见
/// [`without_the_unreadable_root_ring_slot_the_same_mounts_delete_rows`]。
#[test]
fn a_root_ring_slot_unreadable_on_every_read_keeps_every_sweep_from_deleting_a_row() {
    let ranges = unreadable_root_ring_slot(FailingReadsOfARange::Every);
    let seen = mounts_with_a_write_and_an_unmount_each(&ranges);
    assert!(ranges.failed_reads() > 0, "前提：那一槽真被读过、读坏过");
    // 迭代上界：每次挂载一条；跨轮携带的是上一次清扫之后的行数。
    let mut rows_before = 0;
    for (mount_index, mount) in seen.iter().enumerate() {
        if mount_index == 0 {
            assert!(
                matches!(
                    mount.sweep,
                    InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold {
                        rows: 0,
                        ..
                    }
                ),
                "mkfs 之后第一次挂载（实例 1）不写行、0 行不扫：{:?}",
                mount.sweep
            );
        } else {
            assert!(
                matches!(
                    mount.sweep,
                    InstanceTableSweepAtTheEndOfTheMount::Swept(
                        InstanceTableSweepOutcome::NoRowDeleted(
                            NoRowDeletedThisRound::ARootRingSlotIsUnreadableOrAllZero { .. }
                        )
                    )
                ),
                "第 {mount_index} 次挂载的清扫停在根环一个读不出或全零的槽上：{:?}",
                mount.sweep
            );
        }
        assert!(
            mount.rows_after_the_sweep >= rows_before,
            "第 {mount_index} 次挂载清扫之后 {} 行，比上一次的 {rows_before} 行少",
            mount.rows_after_the_sweep
        );
        rows_before = mount.rows_after_the_sweep;
    }
    assert!(
        matches!(
            seen.last().expect("挂了 8 次").sweep,
            InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::NoRowDeleted(
                NoRowDeletedThisRound::ARootRingSlotIsUnreadableOrAllZero { ring_slot }
            )) if ring_slot == UNREADABLE_ROOT_RING_SLOT
        ),
        "根环早已写满，最后一次清扫点名的是读坏的那一槽：{:?}",
        seen.last().expect("挂了 8 次").sweep
    );
    assert_eq!(
        rows_before,
        MOUNTS_WITH_ONE_ROOT_RING_SLOT_UNREADABLE - 1,
        "8 次挂载里实例 2 起每次写一行、一行都没删"
    );
}

/// 场景：同 [`a_root_ring_slot_unreadable_on_every_read_keeps_every_sweep_from_deleting_a_row`] 那 8 次挂载，那一槽一次都不读坏（只数读了几次）。
/// 预期：至少一次清扫删了行，最后一次清扫之后的行数比 8 次挂载写下的 7 行少。
#[test]
fn without_the_unreadable_root_ring_slot_the_same_mounts_delete_rows() {
    let ranges = unreadable_root_ring_slot(FailingReadsOfARange::Never);
    let seen = mounts_with_a_write_and_an_unmount_each(&ranges);
    assert!(
        ranges.reads_of_range(0) > 0,
        "前提：那一槽被读过（对照与读坏那一条读的是同一槽）"
    );
    assert!(
        seen.iter().any(|mount| matches!(
            mount.sweep,
            InstanceTableSweepAtTheEndOfTheMount::Swept(
                InstanceTableSweepOutcome::RowsDeleted { .. }
            )
        )),
        "至少一次清扫删了行"
    );
    let rows_after_the_last_sweep = seen.last().expect("挂了 8 次").rows_after_the_sweep;
    assert!(
        rows_after_the_last_sweep < MOUNTS_WITH_ONE_ROOT_RING_SLOT_UNREADABLE - 1,
        "最后一次清扫之后 {rows_after_the_last_sweep} 行，比一行都不删的 7 行少"
    );
}
