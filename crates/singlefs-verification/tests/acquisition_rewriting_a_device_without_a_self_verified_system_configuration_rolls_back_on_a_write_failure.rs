//! 取号补写两槽全废的盘（丙'）时取号写报块设备错：补写闸第六件「补写失败按甲' 每块盘回卷一次」那一支（C874（补写那一支进不了崩溃测试、运行时看不出），
//! 判决 `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R4「回卷那一支用 `WriteFails` 用例罩」）。崩溃点重放录的是不注入故障的真实运行，
//! 盘报写错时的回卷写不在录制流里，这两条用写故障把它强制走到。
//!
//! 都在单元区 384 槽那一档的两块内存小盘上（`tests/common_admission_pool/mod.rs`）：第一个文件之后覆盖写一次（所选那一版 txg 4），
//! 盘 1 两个系统配置槽整槽写 0（两槽全废：读得出、自证不过），过得了补写闸。取号写两轮，每轮逐盘写一次系统配置槽：盘 1 两槽都不自证，
//! 前一轮那一写世代号 1、落槽 1，后一轮世代号 2、落槽 0。
mod common_admission_pool;

use common_admission_pool::{
    device_bytes_of_the_384_slot_small_devices, fixed_geometry_of_the_384_slot_small_devices,
    parameters_of_the_384_slot_small_devices, plain_device, plain_devices_on, PoolUnderTest,
    OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
use singlefs_core::recovery::verified_system_configuration_slots;
use singlefs_core::transaction::AcquisitionRollback;
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_verification::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_verification::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
use singlefs_verification::segments::StepKind;
use singlefs_verification::{RecordingBlockDevice, SharedStream};

const DEVICE_LOSING_BOTH_SLOTS: DeviceIdentity = DeviceIdentity(1);
const OTHER_DEVICE: DeviceIdentity = DeviceIdentity(0);
/// 补写之前池里的实例：第一个文件与覆盖写都是 mkfs 同一个进程里的实例 1。
const INSTANCE_BEFORE_THE_ACQUISITION: InstanceGeneration = InstanceGeneration(1);
/// 报错的那次挂载要取的号。
const INSTANCE_OF_THE_FAILED_ACQUISITION: InstanceGeneration = InstanceGeneration(2);

fn slot_spacing_in_bytes() -> u64 {
    u64::from(
        parameters_of_the_384_slot_small_devices()
            .geometry
            .fixed_structure_slot_spacing,
    )
}

/// 第一个文件之后覆盖写一次的池，盘 1 两个系统配置槽整槽写 0。
fn image_with_both_system_configuration_slots_of_device_one_zeroed() -> MemoryPool {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        plain_device,
    );
    pool.overwrite(OVERWRITE_BYTES)
        .expect("384 槽小盘上第一个文件之后覆盖写一次落得下");
    let mut image = pool.image();
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let device_image = image
        .devices
        .get_mut(&DEVICE_LOSING_BOTH_SLOTS)
        .expect("池里有盘 1");
    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        device_image.write(
            DeviceOffsetInBytes(slot_index * slot_spacing_in_bytes()),
            &vec![0u8; slot_bytes],
        );
    }
    image
}

/// 一块盘两槽里本池自证过的系统配置：(世代号, 实例代号)，按世代号排好。
fn self_verified_slots_of(
    image: &MemoryPool,
    device: DeviceIdentity,
) -> Vec<(u64, InstanceGeneration)> {
    let parameters = parameters_of_the_384_slot_small_devices();
    let mut slots: Vec<(u64, InstanceGeneration)> = verified_system_configuration_slots(
        image,
        device,
        slot_spacing_in_bytes(),
        &parameters.filesystem_identifier,
    )
    .iter()
    .map(|system_configuration| {
        (
            system_configuration.quantities.slot_generation,
            system_configuration.quantities.journal_instance,
        )
    })
    .collect();
    slots.sort_unstable();
    slots
}

/// 盘 1 的系统配置槽写只有第 `ordinal` 次报块设备错。
fn the_nth_system_configuration_write_on_device_one_fails(ordinal: u64) -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::OnlyDevice(DEVICE_LOSING_BOTH_SLOTS),
        placement: FaultPlacement::OffsetBelow(
            SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE * slot_spacing_in_bytes(),
        ),
        counting: FaultCounting::PerDevice,
        occurrence: FaultOccurrence::TheNthMatchingCall(ordinal),
    }
}

/// 报错的那次可写挂载：取号失败、回卷成没成，挂载之后的盘面，与每块盘落盘的系统配置槽写数。
struct FailedAcquisition {
    rollback: AcquisitionRollback,
    image_after: MemoryPool,
    writes_on_the_device_losing_both_slots: usize,
    writes_on_the_other_device: usize,
}

/// 在 `base` 上可写挂载一次，盘 1 的系统配置槽写第 `ordinal` 次报块设备错：可写挂载必须以取号失败返回。
fn mount_with_the_nth_system_configuration_write_on_device_one_failing(
    base: &MemoryPool,
    ordinal: u64,
) -> FailedAcquisition {
    let stream = SharedStream::retaining_contents();
    let plan = SharedFaultPlan::unarmed(fixed_geometry_of_the_384_slot_small_devices());
    plan.arm(the_nth_system_configuration_write_on_device_one_fails(
        ordinal,
    ));
    let mut devices: Vec<(
        DeviceIdentity,
        FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>,
    )> = plain_devices_on(base)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(
                    identity,
                    RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
                    plan.clone(),
                ),
            )
        })
        .collect();
    let refusal = mount_writable_with_space_admission(
        &parameters_of_the_384_slot_small_devices(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect_err("取号写报块设备错：可写挂载必须失败");
    let MountError::Acquisition(acquisition) = refusal else {
        panic!("该报取号失败，实际 {refusal:?}");
    };
    let (writes, _segments) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    assert!(
        writes
            .iter()
            .all(|write| write.kind == StepKind::SystemConfigurationSlot),
        "取号失败那次挂载只写了系统配置槽：{:?}",
        writes.iter().map(|write| write.kind).collect::<Vec<_>>()
    );
    let writes_on =
        |device: DeviceIdentity| writes.iter().filter(|write| write.device == device).count();
    let mut image_after = base.clone();
    image_after.apply_writes(&writes);
    FailedAcquisition {
        rollback: acquisition.rollback,
        image_after,
        writes_on_the_device_losing_both_slots: writes_on(DEVICE_LOSING_BOTH_SLOTS),
        writes_on_the_other_device: writes_on(OTHER_DEVICE),
    }
}

/// 在 `image` 的拷贝上不注入故障可写挂载一次：交回取到的号与这次补写过的盘。
fn mount_without_faults(image: &MemoryPool) -> (InstanceGeneration, Vec<DeviceIdentity>) {
    let mut devices = plain_devices_on(image);
    let mounted = mount_writable_with_space_admission(
        &parameters_of_the_384_slot_small_devices(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .unwrap_or_else(|error| panic!("回卷之后不注入故障再可写挂载应当做成，实际 {error:?}"));
    (
        mounted.output.instance,
        mounted.output.devices_rewritten_by_the_acquisition,
    )
}

/// 取号后一轮盘 1 那一写报错（前一轮补写的那一写落了槽 1、号 2）：盘 0 写过两轮、盘 1 写过一轮，回卷每块盘只写一次（盘 0 三写、盘 1 两写）；
/// 盘 1 的回卷写世代号 2、落槽 0、带回卷之前的号 1，槽 1 留着补写那一写的号 2——盘 1 从此两槽都自证得过，下一次取号跳过 2、取 3。
/// 回卷之后不注入故障再挂：盘 1 按有自证槽的盘走（落后支），不再过补写闸，补写盘表为空（C929（补写盘回卷之后不再过补写闸），代码轮
/// `research/prompts/m3-close-b19-code-r2-main-verification.md` 第二节 U5 采候选一「照今天不判」）。
/// 判别力：回卷只回卷盘表第一块盘（变异「C874 取号补写失败：回卷只回卷盘表第一块盘」）时盘 1 一写都不回卷，写数那一条红；
/// 有自证槽的盘也照两槽全废的盘再过补写闸、过了就交给取号补写（变异「C929 回卷之后下一次挂载仍把那块盘交给补写」）时，补写盘表为空那一条红。
#[test]
fn second_round_write_failing_on_the_device_being_rewritten_rolls_each_device_back_once() {
    let base = image_with_both_system_configuration_slots_of_device_one_zeroed();
    assert!(
        self_verified_slots_of(&base, DEVICE_LOSING_BOTH_SLOTS).is_empty(),
        "前提：盘 1 两槽全废"
    );
    let failed = mount_with_the_nth_system_configuration_write_on_device_one_failing(&base, 2);
    assert!(
        matches!(failed.rollback, AcquisitionRollback::RolledBack),
        "两块盘都写出过取号写，都回卷，回卷写之前那道屏障与回卷写都不报错：{:?}",
        failed.rollback
    );
    assert_eq!(
        (
            failed.writes_on_the_other_device,
            failed.writes_on_the_device_losing_both_slots
        ),
        (3, 2),
        "盘 0：两轮取号写加一次回卷；盘 1：第一轮补写那一写加一次回卷（第二轮那一写报错、没落盘）——每块盘只回卷一次"
    );
    assert_eq!(
        self_verified_slots_of(&failed.image_after, DEVICE_LOSING_BOTH_SLOTS),
        vec![
            (1, INSTANCE_OF_THE_FAILED_ACQUISITION),
            (2, INSTANCE_BEFORE_THE_ACQUISITION)
        ],
        "盘 1：第一轮补写那一写（世代号 1、号 2）留着，回卷写世代号 2、带回卷之前的号 1"
    );
    let (instance_of_the_next_mount, devices_rewritten_by_the_next_mount) =
        mount_without_faults(&failed.image_after);
    assert_eq!(
        instance_of_the_next_mount,
        InstanceGeneration(3),
        "盘 1 槽 1 留着号 2：下一次取号跳过它"
    );
    assert!(
        devices_rewritten_by_the_next_mount.is_empty(),
        "回卷之后盘 1 两槽都自证得过：下一次挂载按有自证槽的盘走、不再过补写闸、没有盘交给取号补写，实际补写了 {devices_rewritten_by_the_next_mount:?}"
    );
}

/// 取号头一轮盘 1 那一写（补写它的第一写）就报错：只有盘 0 写出过取号写，回卷只回卷盘 0（盘 0 两写、盘 1 一写都没落），
/// 盘 1 照旧两槽全废；不注入故障再挂，盘 1 照旧过补写闸、被这一次的取号补写（挂载输出点名它），号取 3（盘 0 一槽留着号 2）。
#[test]
fn first_round_write_failing_on_the_device_being_rewritten_rolls_back_the_other_device_and_the_next_mount_rewrites_it(
) {
    let base = image_with_both_system_configuration_slots_of_device_one_zeroed();
    let failed = mount_with_the_nth_system_configuration_write_on_device_one_failing(&base, 1);
    assert!(
        matches!(failed.rollback, AcquisitionRollback::RolledBack),
        "盘 0 写出过取号写，回卷它：{:?}",
        failed.rollback
    );
    assert_eq!(
        (
            failed.writes_on_the_other_device,
            failed.writes_on_the_device_losing_both_slots
        ),
        (2, 0),
        "盘 0：第一轮取号写加一次回卷；盘 1：补写那一写报错、没落盘，没写出过就不回卷"
    );
    assert!(
        self_verified_slots_of(&failed.image_after, DEVICE_LOSING_BOTH_SLOTS).is_empty(),
        "盘 1 照旧两槽全废"
    );
    assert_eq!(
        mount_without_faults(&failed.image_after),
        (InstanceGeneration(3), vec![DEVICE_LOSING_BOTH_SLOTS]),
        "不注入故障再挂：盘 1 照旧过补写闸、被补写（挂载输出点名它），盘 0 一槽留着号 2，取 3"
    );
}
