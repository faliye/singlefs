//! 实例表清扫的块设备错两支（C703（清扫的块设备错两支没有用例）；`crates/singlefs-core/src/instance_table_sweep.rs` 的
//! `InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing` 与 `InstanceTableSweepOutcome::SweepPublishFailed`）。
//! D18（块里携带什么信息） 已定项 11「回收」一条只写了清扫发布的落点被拒那一格（这一轮不删行，挂载照样交回可写），块设备错怎么处置条款没写：
//! 这几条钉的是实现现状——这一轮不删行、已抹的头留着、挂载交回可写，之后一次不注入错的清扫照常删行。
//!
//! 盘面同 `instance_table_sweep_at_the_end_of_a_writable_mount.rs`（S = 4 的 12 槽小环、两块 128 MiB 内存盘；实例 1 的孤儿留在盘上、
//! 实例 1 的根全部轮出根环）。注入点从同一份盘面上一次不注入错的清扫挂载（干跑）的录制流里找：抹头的写、抹头之后那道屏障、清扫发布的头一个写，
//! 各在它那一块盘、那个偏移上第几次，再在同一份盘面上起一次挂载、只在那一次上注入。
mod common;
mod common_images_for_the_checker_tier;
mod common_root_ring_holes;
#[path = "common_images_for_the_checker_tier/instance_table_sweep_at_the_end_of_a_writable_mount.rs"]
mod scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount;

use std::collections::BTreeSet;

use common_root_ring_holes::{
    content_of_the_version, fixed_geometry_of_the_ring_hole_pools,
    parameters_of_the_ring_hole_pools, recording_of, RecordedRingHoleDevices,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::{
    head_bytes_at, image_after, instances_with_a_row_on, orphans_of_instance_one_still_on,
    pool_with_a_dead_instance, publish_an_overwrite, sweeping_at_the_end_of_the_mount_on_any_row,
    sweeping_mount_on, PoolWithADeadInstance,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::instance_table_sweep::{
    unit_head_read_by_sweep, InstanceTableSweepAtTheEndOfTheMount, InstanceTableSweepOutcome,
    PlacementOfAUnitHead,
};
use singlefs_core::mount::{
    mount_writable_with_instance_table_sweep_configuration, Mounted, ShadowLedger,
};
use singlefs_core::transaction::{
    publish_overwrite, resend_the_frozen_publish, FirstFile, PoolWriter, PublishError,
};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{
    RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream,
};

/// 清扫挂载的盘：外面一层注入故障，里面一层录制器（注入的那一次写不落盘、录制流里也没有它）。
type FaultyRecordedDevices = Vec<(
    DeviceIdentity,
    FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>,
)>;

/// 同一份盘面上一次不注入错的清扫挂载（有一行就扫）：抹了哪几个头（按抹的次序），与这次挂载的录制流。
struct SweepWithoutAFault {
    heads_erased: Vec<PlacementOfAUnitHead>,
    operations: Vec<RetainedOperation>,
}

fn sweep_without_a_fault(pool: &PoolWithADeadInstance) -> SweepWithoutAFault {
    let (mounted, _devices, stream) =
        sweeping_mount_on(&pool.image, sweeping_at_the_end_of_the_mount_on_any_row());
    let InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
        heads_erased,
        ..
    }) = mounted.output.instance_table_sweep
    else {
        panic!(
            "前提：不注入错的那次清扫删了行：{:?}",
            mounted.output.instance_table_sweep
        );
    };
    assert!(
        heads_erased.len() >= 2,
        "前提：干跑抹了至少两个头（抹头的写报错那一条要在第二个头上注入）：{heads_erased:?}"
    );
    SweepWithoutAFault {
        heads_erased,
        operations: stream.retained_operations(),
    }
}

fn is_a_write(operation: &RetainedOperation) -> bool {
    match operation.operation.kind {
        RecordedOperationKind::Write
        | RecordedOperationKind::WriteForceUnitAccess
        | RecordedOperationKind::WriteZeroes
        | RecordedOperationKind::Discard => true,
        RecordedOperationKind::Barrier => false,
    }
}

/// 录制流里落在 `device` 的 `offset` 上的写，在第 `index` 步（含）之前有几次：注入计划按「这块盘、这个偏移上第几次写」点到同一步。
fn writes_at_the_same_place_up_to(
    operations: &[RetainedOperation],
    index: usize,
) -> (DeviceIdentity, DeviceOffsetInBytes, u64) {
    let target = &operations[index].operation;
    let ordinal = operations[..=index]
        .iter()
        .filter(|operation| {
            is_a_write(operation)
                && operation.operation.device == target.device
                && operation.operation.offset == target.offset
        })
        .count();
    (
        target.device,
        target.offset,
        u64::try_from(ordinal).expect("次数装得进 u64"),
    )
}

/// 干跑的录制流里抹 `head` 那一步：那一槽上第一次写，是一个物理块的全 0 FUA 写（挂载在清扫之前没写过这一槽：写过的话重读认出来的不是判的那个孤儿）。
fn erasing_write_of(sweep: &SweepWithoutAFault, head: PlacementOfAUnitHead) -> usize {
    let offset = head.slot.to_device_offset();
    let index = sweep
        .operations
        .iter()
        .position(|operation| {
            is_a_write(operation)
                && operation.operation.device == head.device
                && operation.operation.offset == offset
        })
        .unwrap_or_else(|| panic!("前提：干跑写过抹的那一槽 {head:?}"));
    let erasing = &sweep.operations[index];
    assert_eq!(
        erasing.operation.kind,
        RecordedOperationKind::WriteForceUnitAccess,
        "前提：那一槽上第一次写是抹头的 FUA 写"
    );
    assert!(
        erasing
            .contents
            .as_ref()
            .is_some_and(|bytes| bytes.iter().all(|byte| *byte == 0)),
        "前提：那一槽上第一次写写的全是 0（抹头）"
    );
    index
}

/// 在 `pool` 的盘面上起清扫挂载（有一行就扫），盘外面包上只在 `schedule` 那一次注入的故障：交回挂载、盘、录制流与注入计划。
fn sweeping_mount_with_a_fault(
    pool: &PoolWithADeadInstance,
    schedule: FaultSchedule,
) -> (
    Mounted,
    FaultyRecordedDevices,
    SharedStream,
    SharedFaultPlan,
) {
    let stream = SharedStream::retaining_contents();
    let plan = SharedFaultPlan::armed(fixed_geometry_of_the_ring_hole_pools(), schedule);
    let recorded: RecordedRingHoleDevices = recording_of(&pool.image, &stream);
    let mut devices: FaultyRecordedDevices = recorded
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
            )
        })
        .collect();
    let mounted = mount_writable_with_instance_table_sweep_configuration(
        &parameters_of_the_ring_hole_pools(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        sweeping_at_the_end_of_the_mount_on_any_row(),
    )
    .expect("块设备错只在清扫那一步：挂载照样做成");
    assert_eq!(plan.fired_count(), 1, "注入恰好发生了一次");
    (mounted, devices, stream, plan)
}

/// 只在 `device` 的 `offset` 上第 `ordinal` 次写注入写报错。
fn the_write_at(
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    ordinal: u64,
) -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::OnlyDevice(device),
        placement: FaultPlacement::OffsetExactly(offset),
        counting: FaultCounting::AcrossThePool,
        occurrence: FaultOccurrence::TheNthMatchingCall(ordinal),
    }
}

/// 挂载交回之后在它的分配器上覆盖写一次做成（挂载交回的是可写的）。
fn an_overwrite_after_the_mount_succeeds(
    mounted: &mut Mounted,
    devices: &mut FaultyRecordedDevices,
) {
    let current = mounted
        .current
        .file_version()
        .expect("这一版带文件")
        .clone();
    publish_an_overwrite(
        devices,
        &mut mounted.allocator,
        &current,
        &content_of_the_version(400),
        mounted.output.instance,
    );
}

/// 在 `image` 上再起一次不注入错的清扫挂载（有一行就扫）：删了行，实例 1 那一行没了，实例 1 的孤儿一个不剩。
fn the_next_sweep_deletes_the_row_of_instance_one(
    pool: &PoolWithADeadInstance,
    image: &MemoryPool,
) {
    let (mounted, _devices, stream) =
        sweeping_mount_on(image, sweeping_at_the_end_of_the_mount_on_any_row());
    assert!(
        matches!(
            mounted.output.instance_table_sweep,
            InstanceTableSweepAtTheEndOfTheMount::Swept(
                InstanceTableSweepOutcome::RowsDeleted { .. }
            )
        ),
        "不注入错的下一次清扫删行：{:?}",
        mounted.output.instance_table_sweep
    );
    let after = image_after(image, &stream);
    assert!(
        !instances_with_a_row_on(&after).contains(&InstanceGeneration(1)),
        "下一次清扫之后实例 1 那一行没了"
    );
    assert_eq!(
        orphans_of_instance_one_still_on(pool, &after),
        BTreeSet::<PlacementOfAUnitHead>::new(),
        "下一次清扫之后实例 1 的孤儿一个不剩"
    );
}

/// 场景：清扫挂载抹第二个孤儿头的那一个 FUA 写报块设备错。
/// 预期：结局 `StoppedByABlockDeviceErrorWhileErasing`，已抹的是第一个头（留着、读不出单元头），第二个起的头都还在；这一轮不删行（实例 1 那一行还在）；
/// 挂载交回可写（覆盖写一次做成）；之后一次不注入错的清扫删掉实例 1 那一行、孤儿一个不剩。
#[test]
fn a_write_error_while_erasing_a_head_deletes_no_row_keeps_the_heads_erased_before_it_and_the_next_sweep_deletes_the_row(
) {
    let pool = pool_with_a_dead_instance(None);
    let dry = sweep_without_a_fault(&pool);
    let erasing_the_second_head = erasing_write_of(&dry, dry.heads_erased[1]);
    let (device, offset, ordinal) =
        writes_at_the_same_place_up_to(&dry.operations, erasing_the_second_head);
    let (mut mounted, mut devices, stream, _plan) =
        sweeping_mount_with_a_fault(&pool, the_write_at(device, offset, ordinal));
    assert!(
        matches!(
            &mounted.output.instance_table_sweep,
            InstanceTableSweepAtTheEndOfTheMount::Swept(
                InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing {
                    heads_erased,
                    ..
                }
            ) if *heads_erased == dry.heads_erased[..1]
        ),
        "抹第二个头的写报错：停在抹头那一步、已抹的只有第一个：{:?}",
        mounted.output.instance_table_sweep
    );
    let after_the_faulty_sweep = image_after(&pool.image, &stream);
    assert!(
        instances_with_a_row_on(&after_the_faulty_sweep).contains(&InstanceGeneration(1)),
        "这一轮不删行：实例 1 那一行还在"
    );
    assert_eq!(
        unit_head_read_by_sweep(&head_bytes_at(&after_the_faulty_sweep, dry.heads_erased[0])),
        None,
        "已抹的第一个头留着（抹掉了）"
    );
    let heads_of_the_orphans: Vec<_> = pool
        .orphans_left_on_disk
        .iter()
        .filter(|(placement, _)| dry.heads_erased[1..].contains(placement))
        .collect();
    assert_eq!(
        heads_of_the_orphans.len(),
        dry.heads_erased.len() - 1,
        "前提：干跑抹的头都是盘上留着的孤儿"
    );
    for (placement, head) in heads_of_the_orphans {
        assert_eq!(
            unit_head_read_by_sweep(&head_bytes_at(&after_the_faulty_sweep, *placement)),
            Some(*head),
            "没抹到的孤儿头 {placement:?} 还在"
        );
    }
    an_overwrite_after_the_mount_succeeds(&mut mounted, &mut devices);
    the_next_sweep_deletes_the_row_of_instance_one(&pool, &image_after(&pool.image, &stream));
}

/// 场景：清扫挂载抹完全部孤儿头之后，那一块盘的那道屏障报块设备错。
/// 预期：结局 `StoppedByABlockDeviceErrorWhileErasing`，已抹的是全部头（留着）；这一轮不删行；挂载交回可写；之后一次不注入错的清扫删掉实例 1 那一行。
#[test]
fn a_barrier_error_after_erasing_the_heads_deletes_no_row_keeps_the_heads_erased_and_the_next_sweep_deletes_the_row(
) {
    let pool = pool_with_a_dead_instance(None);
    let dry = sweep_without_a_fault(&pool);
    let erasing_the_last_head =
        erasing_write_of(&dry, *dry.heads_erased.last().expect("干跑抹了头"));
    let barrier_after_erasing = erasing_the_last_head
        + 1
        + dry.operations[erasing_the_last_head + 1..]
            .iter()
            .position(|operation| operation.operation.kind == RecordedOperationKind::Barrier)
            .expect("前提：抹头之后有屏障");
    let device = dry.operations[barrier_after_erasing].operation.device;
    let barriers_on_the_device = dry.operations[..=barrier_after_erasing]
        .iter()
        .filter(|operation| {
            operation.operation.kind == RecordedOperationKind::Barrier
                && operation.operation.device == device
        })
        .count();
    let (mut mounted, mut devices, stream, _plan) = sweeping_mount_with_a_fault(
        &pool,
        FaultSchedule {
            fault: InjectedFault::BarrierFails,
            device: FaultDeviceSelector::OnlyDevice(device),
            placement: FaultPlacement::AnyOffset,
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::TheNthMatchingCall(
                u64::try_from(barriers_on_the_device).expect("次数装得进 u64"),
            ),
        },
    );
    assert!(
        matches!(
            &mounted.output.instance_table_sweep,
            InstanceTableSweepAtTheEndOfTheMount::Swept(
                InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing {
                    heads_erased,
                    ..
                }
            ) if *heads_erased == dry.heads_erased
        ),
        "抹头之后的屏障报错：停在抹头那一步、全部头都抹了：{:?}",
        mounted.output.instance_table_sweep
    );
    let after_the_faulty_sweep = image_after(&pool.image, &stream);
    assert!(
        instances_with_a_row_on(&after_the_faulty_sweep).contains(&InstanceGeneration(1)),
        "这一轮不删行：实例 1 那一行还在"
    );
    for placement in &dry.heads_erased {
        assert_eq!(
            unit_head_read_by_sweep(&head_bytes_at(&after_the_faulty_sweep, *placement)),
            None,
            "已抹的头 {placement:?} 留着（抹掉了）"
        );
    }
    an_overwrite_after_the_mount_succeeds(&mut mounted, &mut devices);
    the_next_sweep_deletes_the_row_of_instance_one(&pool, &image_after(&pool.image, &stream));
}

/// 场景：清扫挂载抹完头、过了屏障，清扫发布的头一个写报块设备错。
/// 预期：结局 `SweepPublishFailed`（原因是块设备错），已抹的是全部头；这一轮不删行（盘上最新的根指着的实例表里实例 1 那一行还在）；
/// 落盘途中报错的那次清扫发布照发布路径冻结在交回的分配器上：冻结着时覆盖写被拒，原样重发做成之后覆盖写做成（挂载交回可写）；
/// 在清扫挂载刚交回时的盘面上起一次不注入错的清扫，删掉实例 1 那一行、孤儿一个不剩。
#[test]
fn a_write_error_while_landing_the_sweep_publish_deletes_no_row_freezes_the_publish_and_the_next_sweep_deletes_the_row(
) {
    let pool = pool_with_a_dead_instance(None);
    let dry = sweep_without_a_fault(&pool);
    let erasing_the_last_head =
        erasing_write_of(&dry, *dry.heads_erased.last().expect("干跑抹了头"));
    let first_write_of_the_sweep_publish = erasing_the_last_head
        + 1
        + dry.operations[erasing_the_last_head + 1..]
            .iter()
            .position(is_a_write)
            .expect("前提：抹头之后清扫发布有写");
    let (device, offset, ordinal) =
        writes_at_the_same_place_up_to(&dry.operations, first_write_of_the_sweep_publish);
    let (mut mounted, mut devices, stream, _plan) =
        sweeping_mount_with_a_fault(&pool, the_write_at(device, offset, ordinal));
    assert!(
        matches!(
            &mounted.output.instance_table_sweep,
            InstanceTableSweepAtTheEndOfTheMount::Swept(
                InstanceTableSweepOutcome::SweepPublishFailed {
                    heads_erased,
                    cause,
                }
            ) if *heads_erased == dry.heads_erased
                && matches!(**cause, PublishError::BlockDevice(_))
        ),
        "清扫发布落盘途中报错：清扫发布失败、全部头都抹了：{:?}",
        mounted.output.instance_table_sweep
    );
    let after_the_faulty_sweep = image_after(&pool.image, &stream);
    assert!(
        instances_with_a_row_on(&after_the_faulty_sweep).contains(&InstanceGeneration(1)),
        "这一轮不删行：实例 1 那一行还在"
    );
    assert!(
        mounted.allocator.frozen_publish().is_some(),
        "落盘途中报错的那次清扫发布冻结在交回的分配器上"
    );
    let parameters = parameters_of_the_ring_hole_pools();
    let current = mounted
        .current
        .file_version()
        .expect("这一版带文件")
        .clone();
    let refused = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted.allocator,
            &current,
            FirstFile {
                content: &content_of_the_version(400),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + 400,
            },
            mounted.output.instance,
        )
    };
    assert!(
        matches!(
            refused,
            Err(PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. })
        ),
        "冻结着时覆盖写被拒：{refused:?}"
    );
    let resent = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        resend_the_frozen_publish(&mut writer, &mut mounted.allocator)
            .expect("原样重发做成")
            .expect("有一次冻结着的发布")
    };
    let resent_version = resent.into_file_version().expect("重发的那一版带文件");
    publish_an_overwrite(
        &mut devices,
        &mut mounted.allocator,
        &resent_version,
        &content_of_the_version(401),
        mounted.output.instance,
    );
    the_next_sweep_deletes_the_row_of_instance_one(&pool, &after_the_faulty_sweep);
}
