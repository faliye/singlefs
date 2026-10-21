//! 取号写两轮（甲'，三方判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U1 与第四节「落地」；用户 2026-10-19 定 S5「罩，取号写两轮」）：
//! 每轮逐盘写一次系统配置、每轮之后一道池屏障，两轮都完成才交出新号；任一轮报错，每块盘只回卷一次。
//! 两条都在单元区 384 槽那一档的两块内存小盘上（`tests/common_admission_pool/mod.rs`）。
#[path = "common_admission_pool/mod.rs"]
mod common_admission_pool;

use common_admission_pool::{
    device_bytes_of_the_384_slot_small_devices, fixed_geometry_of_the_384_slot_small_devices,
    parameters_of_the_384_slot_small_devices, plain_device, plain_devices_on, PoolUnderTest,
    OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
use singlefs_core::recovery::verified_system_configuration_slots;
use singlefs_core::transaction::AcquisitionRollback;
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::memory_pool::{
    writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice, WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 系统配置两槽住在偏移 0 与固定结构槽距处（这一档小盘的槽距 4096），都在这个界之下。
const SYSTEM_CONFIGURATION_SLOTS_END_OFFSET: u64 = 8192;

fn slot_spacing_in_bytes(parameters: &MakeFilesystemParameters) -> u64 {
    u64::from(parameters.geometry.fixed_structure_slot_spacing)
}

/// 第一个文件之后（单元区 384 槽那一档小盘，mkfs 同一个进程里取号 1、暖机、第一个文件）的盘面。
fn image_after_the_first_file() -> MemoryPool {
    PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        plain_device,
    )
    .image()
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
        slot_spacing_in_bytes(&parameters),
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
    slots.sort();
    slots
}

/// 原地覆写撕在半中间的镜像：新旧两份不同的那一段，前一半是新字节、后一半是旧字节（与崩溃点重放的撕裂判据同一个取法：新旧都读不出）。
fn torn_image_of_an_in_place_overwrite(old: &[u8], new: &[u8]) -> Vec<u8> {
    assert_eq!(old.len(), new.len(), "旧字节与这次写一样长");
    let differs = |index: &usize| old[*index] != new[*index];
    let Some(first_differing_byte) = (0..new.len()).find(differs) else {
        return new.to_vec();
    };
    let last_differing_byte = (0..new.len())
        .rev()
        .find(differs)
        .expect("有第一个不同的字节就有最后一个");
    let differing_span_in_bytes = last_differing_byte - first_differing_byte + 1;
    let first_old_byte = first_differing_byte + differing_span_in_bytes / 2;
    let mut torn = new.to_vec();
    torn[first_old_byte..].copy_from_slice(&old[first_old_byte..]);
    torn
}

/// 一段里一次写在一个崩溃状态里落成什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteInTheCrashState {
    NotPersisted,
    Persisted,
    /// 原地覆写撕在半中间（只给原地覆写：落点上原来有字节）。
    Torn,
}

fn image_with(
    base: &MemoryPool,
    write: &RetainedWrite,
    outcome: WriteInTheCrashState,
) -> MemoryPool {
    let mut image = base.clone();
    match outcome {
        WriteInTheCrashState::NotPersisted => {}
        WriteInTheCrashState::Persisted => image.apply_writes(std::slice::from_ref(write)),
        WriteInTheCrashState::Torn => {
            let WrittenContents::Bytes(new) = &write.contents else {
                panic!("原地覆写的是系统配置槽，带字节：{write:?}");
            };
            let device_image = image
                .devices
                .get_mut(&write.device)
                .expect("写落在池里的盘上");
            let old = device_image.read(write.offset, new.len());
            device_image.write(
                write.offset,
                &torn_image_of_an_in_place_overwrite(&old, new),
            );
        }
    }
    image
}

/// 一次写在崩溃状态里可能落成的样子：落点上原来有字节的（原地覆写）三态，别的两态。
fn outcomes_of(base: &MemoryPool, write: &RetainedWrite) -> Vec<WriteInTheCrashState> {
    let WrittenContents::Bytes(new) = &write.contents else {
        return vec![
            WriteInTheCrashState::NotPersisted,
            WriteInTheCrashState::Persisted,
        ];
    };
    let old = base
        .devices
        .get(&write.device)
        .expect("写落在池里的盘上")
        .read(write.offset, new.len());
    if old.iter().all(|byte| *byte == 0) {
        vec![
            WriteInTheCrashState::NotPersisted,
            WriteInTheCrashState::Persisted,
        ]
    } else {
        vec![
            WriteInTheCrashState::NotPersisted,
            WriteInTheCrashState::Persisted,
            WriteInTheCrashState::Torn,
        ]
    }
}

/// 一条写流的每个崩溃状态：之前的段全持久、这一段里每次写各取它可能落成的样子（全组合），再加整条流全持久那一态。
/// 段只有几次写（这条用例的流一共五次写），全组合几十个。
fn every_crash_image_of(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
) -> Vec<MemoryPool> {
    let mut images = Vec::new();
    let mut before_the_segment = base.clone();
    for segment in segments {
        let mut images_of_the_segment = vec![before_the_segment.clone()];
        for write_index in segment {
            let write = &writes[*write_index];
            images_of_the_segment = images_of_the_segment
                .iter()
                .flat_map(|image| {
                    outcomes_of(image, write)
                        .into_iter()
                        .map(|outcome| image_with(image, write, outcome))
                        .collect::<Vec<_>>()
                })
                .collect();
        }
        images.extend(images_of_the_segment);
        let segment_writes: Vec<RetainedWrite> = segment
            .iter()
            .map(|write_index| writes[*write_index].clone())
            .collect();
        before_the_segment.apply_writes(&segment_writes);
    }
    images.push(before_the_segment);
    images
}

/// 盘 1 的系统配置槽写只有第二次报错：取号写第二轮在盘 1 上那一写（第一轮那一写是第一次，之后的回卷写照常）。
fn fail_the_second_round_write_on_device_one() -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(1)),
        placement: FaultPlacement::OffsetBelow(SYSTEM_CONFIGURATION_SLOTS_END_OFFSET),
        counting: FaultCounting::AcrossThePool,
        occurrence: FaultOccurrence::TheNthMatchingCall(2),
    }
}

/// 取号第二轮盘 1 那一写报错（盘 0 已写过两轮、盘 1 写过一轮），回卷每块盘只写一次；再断电在那次挂载写流的每一个崩溃状态上
/// （之前的段全持久、这一段每次写各取没落 / 落了 / 撕裂），每块盘都至少有一槽自证、不加故障再可写挂载一次都挂得上。
/// 判别力：回卷改回逐份传（盘 0 在盘表里出现两次，变异「甲' 取号写两轮 回卷改回逐份传（盘 0 连写两槽、同段撕坏两槽）」）时，
/// 盘 0 两槽同在回卷那一段里被覆写，那一段两写都撕裂的状态里盘 0 一份自证的都没有，「每块盘至少一槽自证」那一条红；
/// 写流里盘 0 的系统配置写数从 3 变 4，写数那一条也红。
// crash-case-check:not-a-crash-case 取号失败那次挂载的写流只有五次系统配置写、二十来个崩溃状态，debug 下单跑几秒
#[test]
fn a_second_round_write_failing_on_device_one_rolls_each_device_back_once_and_every_crash_state_of_that_mount_mounts_writable(
) {
    let parameters = parameters_of_the_384_slot_small_devices();
    let base = image_after_the_first_file();
    let stream = SharedStream::retaining_contents();
    let plan = SharedFaultPlan::unarmed(fixed_geometry_of_the_384_slot_small_devices());
    plan.arm(fail_the_second_round_write_on_device_one());
    let mut devices: Vec<(
        DeviceIdentity,
        FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>,
    )> = plain_devices_on(&base)
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
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect_err("取号第二轮盘 1 那一写报错，可写挂载必须失败");
    let MountError::Acquisition(acquisition) = refusal else {
        panic!("该报取号失败，实际 {refusal:?}");
    };
    assert!(
        matches!(acquisition.rollback, AcquisitionRollback::RolledBack),
        "两块盘都写出过取号写，都要回卷，回卷写之前那道屏障与回卷写都不报错：{acquisition:?}"
    );
    let (writes, segments) = writes_and_segments(
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
    assert_eq!(
        (writes_on(DeviceIdentity(0)), writes_on(DeviceIdentity(1))),
        (3, 2),
        "盘 0：两轮取号写加一次回卷；盘 1：第一轮取号写加一次回卷（第二轮那一写报错、没落盘）——每块盘只回卷一次"
    );
    let crash_images = every_crash_image_of(&base, &writes, &segments);
    assert!(
        crash_images.len() >= 20,
        "崩溃状态够数（三段：第一轮两写、第二轮盘 0 一写、回卷两写，原地覆写各三态）：{}",
        crash_images.len()
    );
    for (crash_state_index, crash_image) in crash_images.iter().enumerate() {
        for device in DEVICES {
            assert!(
                !self_verified_slots_of(crash_image, device).is_empty(),
                "崩溃状态 {crash_state_index}：{device:?} 两槽都解不出（回卷连写同一块盘两槽的那一形）"
            );
        }
        let mut crash_devices = plain_devices_on(crash_image);
        mount_writable_with_space_admission(
            &parameters,
            &mut crash_devices,
            SpaceAdmission::JudgedByTheFormula,
        )
        .unwrap_or_else(|error| {
            panic!("崩溃状态 {crash_state_index}：不加故障再可写挂载一次应当挂得上，实际 {error:?}")
        });
    }
}

/// C555（两盘最新系统配置槽都坏时取号撞号）那段历史：第一个文件之后覆盖写一次（确认过），可写挂载取号 2、写行那次发布的单元落了、
/// 记录与根都没落就崩；两块盘最新的那一槽（取号第二轮写的）各翻坏一个字节，再可写挂载。取号写两轮之下另一槽（第一轮写的）也带号 2，
/// 这一次取 3，不撞号。判别力：只写一轮（变异「甲' 取号写两轮 只写一轮（C555 那段历史撞号）」）时最新那一槽就是唯一带号 2 的一槽，
/// 翻坏之后两槽、根环里最大都是 1，这一次又取 2、与留在单元区的号 2 的单元撞号，这一条红。
#[test]
fn both_devices_losing_the_newest_system_configuration_slot_after_an_acquisition_do_not_hand_out_its_instance_again(
) {
    let parameters = parameters_of_the_384_slot_small_devices();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters.clone(),
        device_bytes_of_the_384_slot_small_devices(),
        plain_device,
    );
    pool.overwrite(OVERWRITE_BYTES)
        .expect("384 槽小盘上第一个文件之后覆盖写一次落得下");
    let base = pool.image();
    let stream = SharedStream::retaining_contents();
    let mut recorded_devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        plain_devices_on(&base)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
                )
            })
            .collect();
    let first_mount = mount_writable_with_space_admission(
        &parameters,
        &mut recorded_devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("覆盖写之后的盘面可写挂载做得成");
    assert_eq!(
        first_mount.output.instance,
        InstanceGeneration(2),
        "这次挂载取号 2"
    );
    let (writes, _segments) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let first_record = writes
        .iter()
        .position(|write| write.kind == StepKind::JournalRecord)
        .expect("写行那次发布写了记录");
    assert!(
        writes[..first_record]
            .iter()
            .any(|write| write.kind == StepKind::UnitWrite),
        "崩在写行那次发布的单元落了、记录还没落的那一刻：前缀里有号 2 写下的单元"
    );
    let mut crash_image = base.clone();
    crash_image.apply_writes(&writes[..first_record]);
    for device in DEVICES {
        let slots = self_verified_slots_of(&crash_image, device);
        assert_eq!(
            slots
                .iter()
                .map(|(_, instance)| *instance)
                .collect::<Vec<_>>(),
            vec![InstanceGeneration(2), InstanceGeneration(2)],
            "{device:?}：取号写两轮之后两槽都带号 2"
        );
        let (newest_generation, _) = *slots.last().expect("两槽都自证");
        let newest_slot_offset =
            DeviceOffsetInBytes((newest_generation % 2) * slot_spacing_in_bytes(&parameters));
        crash_image.flip_byte(device, newest_slot_offset, 200);
    }
    let mut devices = plain_devices_on(&crash_image);
    let second_mount = mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("两块盘最新那一槽翻坏、另一槽自证：判得出（不是读错），可写挂载做得成");
    assert_eq!(
        second_mount.output.instance,
        InstanceGeneration(3),
        "号 2 留在两块盘取号第一轮写的那一槽里，这一次取 3，不与单元区里号 2 写下的单元撞号"
    );
}
