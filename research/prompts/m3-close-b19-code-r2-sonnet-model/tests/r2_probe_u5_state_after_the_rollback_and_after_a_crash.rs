//! 辩方腿 m3-close-b19-code-r2-sonnet 的探针（U5）：回卷之后的下一次挂载走哪一支；取号成功后崩溃留下的盘面上，同一块盘有没有自证槽对下一次挂载的差别。
//! 判决 `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R4「回卷那一支用 `WriteFails` 用例罩」）。崩溃点重放录的是不注入故障的真实运行，
//! 盘报写错时的回卷写不在录制流里，这两条用写故障把它强制走到。
//!
//! 都在单元区 384 槽那一档的两块内存小盘上（`tests/common_admission_pool/mod.rs`）：第一个文件之后覆盖写一次（所选那一版 txg 4），
//! 盘 1 两个系统配置槽整槽写 0（两槽全废：读得出、自证不过），过得了补写闸。取号写两轮，每轮逐盘写一次系统配置槽：盘 1 两槽都不自证，
//! 第一轮那一写世代号 1、落槽 1，第二轮世代号 2、落槽 0。
#[path = "common_admission_pool/mod.rs"]
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
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

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


fn zero_both_system_configuration_slots_of_device_one(image: &mut MemoryPool) {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let device_image = image.devices.get_mut(&DEVICE_LOSING_BOTH_SLOTS).expect("盘 1");
    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        device_image.write(DeviceOffsetInBytes(slot_index * slot_spacing_in_bytes()), &vec![0u8; slot_bytes]);
    }
}

fn mount_plain(image: &MemoryPool) -> Result<singlefs_core::mount::Mounted, MountError> {
    let mut devices = plain_devices_on(image);
    mount_writable_with_space_admission(
        &parameters_of_the_384_slot_small_devices(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
}

fn describe(result: &Result<singlefs_core::mount::Mounted, MountError>) -> String {
    match result {
        Ok(mounted) => format!(
            "MountMade instance={:?} devices_rewritten={:?}",
            mounted.output.instance, mounted.output.devices_rewritten_by_the_acquisition
        ),
        Err(error) => format!("{error:?}").chars().take(1400).collect(),
    }
}

/// 取号第二轮盘 1 那一写报错、回卷之后：下一次挂载盘 1 走哪一支（补写盘表为空 = 不再过补写闸、按有自证槽的盘走）。
#[test]
fn u5_a_the_mount_after_the_rollback_does_not_pass_the_rewrite_gate_again() {
    let base = image_with_both_system_configuration_slots_of_device_one_zeroed();
    let failed = mount_with_the_nth_system_configuration_write_on_device_one_failing(&base, 2);
    println!("PROBE name=u5a_slots_of_device_one_after_the_rollback slots={:?}", self_verified_slots_of(&failed.image_after, DEVICE_LOSING_BOTH_SLOTS));
    let next = mount_plain(&failed.image_after);
    println!("PROBE name=u5a_next_mount result={}", describe(&next));
    let mounted = next.expect("回卷之后再挂做成");
    assert!(mounted.output.devices_rewritten_by_the_acquisition.is_empty(), "不再过补写闸：补写盘表为空");
}

/// 补写成功的挂载在第一条 journal 记录之前崩溃：盘 1 带自证槽（号 2）与号 2 写下的单元。同一份单元区内容，盘 1 槽在 / 槽又被写 0 两种下一次挂载各报什么。
#[test]
fn u5_b_the_same_unit_area_is_accepted_with_the_rewritten_slots_and_refused_when_the_slots_are_dead_again() {
    let base = image_with_both_system_configuration_slots_of_device_one_zeroed();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&base)
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect();
    let m2 = mount_writable_with_space_admission(
        &parameters_of_the_384_slot_small_devices(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("补写那次挂载做成");
    println!("PROBE name=u5b_m2 instance={:?} devices_rewritten={:?}", m2.output.instance, m2.output.devices_rewritten_by_the_acquisition);
    drop(m2);
    let (writes, _segments) = writes_and_segments(&stream.retained_operations(), &fixed_geometry_of_the_384_slot_small_devices());
    let first_record = writes.iter().position(|write| write.kind == StepKind::JournalRecord).expect("写行那次发布写了记录");
    println!("PROBE name=u5b_writes_before_the_first_record kinds={:?}", writes[..first_record].iter().map(|write| (write.device, write.kind)).collect::<Vec<_>>());
    let mut crash = base.clone();
    crash.apply_writes(&writes[..first_record]);
    println!("PROBE name=u5b_slots_of_device_one_in_the_crash_image slots={:?}", self_verified_slots_of(&crash, DEVICE_LOSING_BOTH_SLOTS));
    let with_slots = mount_plain(&crash);
    println!("PROBE name=u5b_next_mount_with_the_rewritten_slots result={}", describe(&with_slots));
    let mut slots_dead_again = crash.clone();
    zero_both_system_configuration_slots_of_device_one(&mut slots_dead_again);
    let without_slots = mount_plain(&slots_dead_again);
    println!("PROBE name=u5b_next_mount_with_the_slots_dead_again result={}", describe(&without_slots));
    assert!(with_slots.is_ok(), "A：带自证槽的盘按有槽的盘走，挂载做成");
    assert!(without_slots.is_err(), "同一份单元区、槽又写 0：补写闸四拒");
}


/// A 会错的盘面（拼出来的）：回卷之后盘 1 的两槽（号 2 的一槽、回卷写的号 1 的一槽），配上盘 1 单元区里号 2 写下的单元（取号成功的那次挂载崩在第一条记录之前留下的）。
/// 单元区里有写序实例大于所选那一版的单元，两槽全废时补写闸四拒（u5_b 第二条）；回卷之后的两槽自证得过，下一次挂载不再过闸，看它收不收。
#[test]
fn u5_c_post_rollback_slots_on_a_unit_area_holding_units_of_a_later_instance_are_not_gated_again() {
    let base = image_with_both_system_configuration_slots_of_device_one_zeroed();
    let failed = mount_with_the_nth_system_configuration_write_on_device_one_failing(&base, 2);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&base)
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect();
    let m2 = mount_writable_with_space_admission(
        &parameters_of_the_384_slot_small_devices(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("补写那次挂载做成");
    drop(m2);
    let (writes, _segments) = writes_and_segments(&stream.retained_operations(), &fixed_geometry_of_the_384_slot_small_devices());
    let first_record = writes.iter().position(|write| write.kind == StepKind::JournalRecord).expect("写行那次发布写了记录");
    let mut composed = base.clone();
    composed.apply_writes(&writes[..first_record]);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        let offset = DeviceOffsetInBytes(slot_index * slot_spacing_in_bytes());
        let bytes = failed.image_after.devices[&DEVICE_LOSING_BOTH_SLOTS].read(offset, slot_bytes);
        composed.devices.get_mut(&DEVICE_LOSING_BOTH_SLOTS).expect("盘 1").write(offset, &bytes);
    }
    println!("PROBE name=u5c_slots_of_device_one_in_the_composed_image slots={:?}", self_verified_slots_of(&composed, DEVICE_LOSING_BOTH_SLOTS));
    let next = mount_plain(&composed);
    println!("PROBE name=u5c_next_mount_on_the_composed_image result={}", describe(&next).chars().take(200).collect::<String>());
    assert!(next.is_ok(), "A：回卷之后的两槽自证得过，下一次挂载不再过补写闸，单元区里号 2 的单元不拦");
    assert!(next.expect("做成").output.devices_rewritten_by_the_acquisition.is_empty());
}
