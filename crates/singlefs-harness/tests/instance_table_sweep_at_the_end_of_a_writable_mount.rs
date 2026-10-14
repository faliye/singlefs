//! 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条；判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第二节 S1
//! 与第六节用户定案；攻方原型 `research/prompts/m3-sweep-and-tree-id-r1-opus-model/sweep-proto/src/lib.rs` 的几形）。
//!
//! 盘面（S = 4 的 12 槽小环、两块 128 MiB 内存盘）：mkfs → 实例 1 第一个文件、覆盖写两次（txg 3、4、5）→ 一次十六个数据单元的顺序写崩在第一条 journal
//! 记录写之前（单元写落了：实例 1 按它那一行判未发布的孤儿）→ 实例 2 可写挂载（写行 6、暖机 7）→ 实例 2 覆盖写到 txg 19（根环里没有实例 1 的根了）。
//! 接着的可写挂载（实例 3）末尾判一次清扫。
mod common;
mod common_images_for_the_checker_tier;
mod common_root_ring_holes;
#[path = "common_images_for_the_checker_tier/instance_table_sweep_at_the_end_of_a_writable_mount.rs"]
mod scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount;

use common::{
    with_unreadable_ranges, FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange,
    UnreadableRangeReadBack,
};
use common_images_for_the_checker_tier::PoolCheckerImages;
use common_root_ring_holes::{
    content_of_the_version, parameters_of_the_ring_hole_pools, pool_of, recording_of,
    RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
};
use scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::{
    head_bytes_at, image_after, instances_with_a_row_on, pool_with_a_dead_instance,
    publish_an_overwrite, sweeping_at_the_end_of_the_mount_on_any_row, sweeping_mount_on,
};
use singlefs_core::address::{DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table_sweep::{
    sweep_the_instance_table_while_mounted, unit_head_read_by_sweep,
    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
    NoRowDeletedThisRound, PlacementOfAUnitHead, SweepPublishPlacements,
};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_instance_table_sweep_configuration, ShadowLedger,
};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::SharedStream;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::the_row_of_a_dead_instance_is_deleted_after_its_roots_left_the_ring_and_its_unpublished_heads_are_erased`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_row_of_a_dead_instance_is_deleted_after_its_roots_left_the_ring_and_its_unpublished_heads_are_erased(
) {
    scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::the_row_of_a_dead_instance_is_deleted_after_its_roots_left_the_ring_and_its_unpublished_heads_are_erased(&mut PoolCheckerImages::left_to_the_checker_tier());
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::the_background_sweep_rereads_every_head_before_erasing_and_deletes_no_row_when_a_slot_was_allocated_after_the_judgement`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_background_sweep_rereads_every_head_before_erasing_and_deletes_no_row_when_a_slot_was_allocated_after_the_judgement(
) {
    scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::the_background_sweep_rereads_every_head_before_erasing_and_deletes_no_row_when_a_slot_was_allocated_after_the_judgement(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::a_device_error_on_the_reread_before_erasing_deletes_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_device_error_on_the_reread_before_erasing_deletes_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance(
) {
    scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::a_device_error_on_the_reread_before_erasing_deletes_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::zeros_read_back_on_the_reread_before_erasing_delete_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn zeros_read_back_on_the_reread_before_erasing_delete_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance(
) {
    scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::zeros_read_back_on_the_reread_before_erasing_delete_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::sweeping_a_periodic_schedule_of_unmounted_and_crashed_mounts_keeps_the_instance_table_short`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn sweeping_a_periodic_schedule_of_unmounted_and_crashed_mounts_keeps_the_instance_table_short() {
    scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::sweeping_a_periodic_schedule_of_unmounted_and_crashed_mounts_keeps_the_instance_table_short(&mut PoolCheckerImages::left_to_the_checker_tier());
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
