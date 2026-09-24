//! 攻方腿 K1 的可跑构造（只住副本，不是被判代码的一部分）：随机注入那一路只摆
//! `FaultSchedule::the_nth_call_across_the_pool`（只在第 n 次命中的调用上注入一次），
//! 「同一块盘上接连两次读都失败」在任何规模下概率为 0。
//! 这条用例把那两次读摆出来：盘 1 的两个系统配置槽都读不出来时，
//! `choose_system_configuration` 当场 `return Err`，不看已经从盘 0 读到的那一份。

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::recovery::{
    choose_system_configuration, recover, JournalPolicy, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter,
};
use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const IMAGE_BYTES: u64 = 4 << 30;

type Devices = Vec<(
    DeviceIdentity,
    FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>,
)>;

/// mkfs、取号、暖机、第一个文件：一个健康的池。
fn healthy_pool() -> (Devices, SharedFaultPlan, u64) {
    let parameters = e142_parameters(512, 512);
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
    };
    let stream = SharedStream::retaining_contents();
    let plan = SharedFaultPlan::unarmed(geometry);
    let mut devices: Devices = (0..2u32)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                FaultInjectingBlockDevice::new(
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                    plan.clone(),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
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
    {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut writer,
            &mut allocator,
            &genesis.root,
            FirstFile {
                content: &first_file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个事务");
    }
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    (devices, plan, spacing)
}

/// 对照：盘 1 的系统配置槽只坏一个（第 1 次命中的读报错），另一个槽还在 ⇒ 恢复照常。
/// 随机注入今天摆得出的就是这一格。
#[test]
fn one_failing_read_on_one_system_configuration_slot_still_recovers() {
    let (devices, plan, spacing) = healthy_pool();
    plan.arm(FaultSchedule {
        fault: InjectedFault::ReadFails,
        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(1)),
        placement: FaultPlacement::OffsetBelow(SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE * spacing),
        counting: FaultCounting::PerDevice,
        occurrence: FaultOccurrence::TheNthMatchingCall(1),
    });
    let chosen = choose_system_configuration(&devices);
    assert!(
        chosen.is_ok(),
        "盘 1 只坏一个系统配置槽时还选得出系统配置：{chosen:?}"
    );
    plan.disarm();
    plan.arm(FaultSchedule {
        fault: InjectedFault::ReadFails,
        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(1)),
        placement: FaultPlacement::OffsetBelow(SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE * spacing),
        counting: FaultCounting::PerDevice,
        occurrence: FaultOccurrence::TheNthMatchingCall(1),
    });
    let recovery = recover(&devices, JournalPolicy::Consult);
    assert!(
        matches!(recovery.outcome, RecoveryOutcome::FileRead { .. }),
        "盘 1 只坏一个系统配置槽时还读得回文件：{:?}",
        recovery.outcome
    );
}

/// 打中：盘 1 的两个系统配置槽都读不出来（那块盘死了、或者前两个固定槽坏了），
/// 盘 0 的两个槽完好，而 `choose_system_configuration` 当场 `return Err`，整个池挂不上。
#[test]
fn both_system_configuration_slots_dead_on_one_device_makes_the_whole_pool_unrecoverable() {
    let (devices, plan, spacing) = healthy_pool();
    let schedule = FaultSchedule {
        fault: InjectedFault::ReadFails,
        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(1)),
        placement: FaultPlacement::OffsetBelow(SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE * spacing),
        counting: FaultCounting::PerDevice,
        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
    };
    plan.arm(schedule);
    let chosen = choose_system_configuration(&devices);
    assert_eq!(
        chosen.err(),
        Some(RecoveryFailure::NoValidSystemConfiguration {
            device: DeviceIdentity(1)
        }),
        "盘 0 的两个系统配置槽完好，盘 1 的两个都读不出来"
    );
    plan.disarm();
    plan.arm(schedule);
    let recovery = recover(&devices, JournalPolicy::Consult);
    assert!(
        matches!(
            recovery.outcome,
            RecoveryOutcome::Failed {
                failure: RecoveryFailure::NoValidSystemConfiguration { .. },
                ..
            }
        ),
        "整个池恢复不了：{:?}",
        recovery.outcome
    );
    // 盘 0 单独拿出来读得出系统配置：坏的只有盘 1，信息一点没少。
    let only_device_zero: Vec<_> = devices
        .into_iter()
        .filter(|(identity, _)| *identity == DeviceIdentity(0))
        .collect();
    assert!(
        choose_system_configuration(&only_device_zero).is_ok(),
        "盘 0 自己的两个系统配置槽是完好的"
    );
    let _ = DeviceOffsetInBytes(0);
}
