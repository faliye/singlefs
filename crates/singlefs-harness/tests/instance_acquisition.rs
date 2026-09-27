//! 取号（D23（journal 的角色与格式） 已定项 16、D18（块里携带什么信息） 已定项 11；C322（取号那一步的屏障怎么放没有条款） 2026-09-14 定案）：
//! 第二次取号看得见、世代号逐盘 +1、取号之后那道屏障、取号写或屏障报错时回卷成旧代号。每条都从写完第一个事务的镜像出发。

mod common;

use common::{build_pool, geometry, parameters, BuiltPool, Recorded};
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::recovery::{choose_system_configuration, verified_system_configuration_slots};
use singlefs_core::transaction::{
    acquire_instance, AcquisitionRollback, CommitStep, InstanceAcquisitionFailed, PoolWriter,
};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};

/// 系统配置两槽住在偏移 0 与 4096，都在这个界之下。
const SYSTEM_CONFIGURATION_SLOTS_END_OFFSET: u64 = 8192;

/// 包在录制盘外面的通用故障注入（增补 3 第 4 件，`singlefs_harness::fault_injection`；这里原先手写的 `FaultInjectingDevice`
/// 2026-09-21 并进了它）：按开关让屏障或系统配置槽写报错，并数真正交给设备的屏障（`FaultDeviceCounts::barriers_forwarded`）。
type FaultInjectingDevice = FaultInjectingBlockDevice<Recorded>;

/// 两块盘共用的注入计划连同它们。
struct WrappedDevices {
    plan: SharedFaultPlan,
    devices: Vec<(DeviceIdentity, FaultInjectingDevice)>,
}

/// 每一道屏障都报错。
fn fail_every_barrier() -> FaultSchedule {
    FaultSchedule::every_call_across_the_pool(InjectedFault::BarrierFails)
}

/// 取号写之后那道池屏障在盘 0 上报错，只这一次：取号写之前那道池屏障是全池第 1、2 次屏障调用（盘 0、盘 1），
/// 写之后那道从全池第 3 次（盘 0）起（代码审阅第 19 条加了写之前那一道）。
fn fail_the_barrier_after_the_acquisition_writes_once() -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::BarrierFails,
        device: FaultDeviceSelector::EveryDevice,
        placement: FaultPlacement::AnyOffset,
        counting: FaultCounting::AcrossThePool,
        occurrence: FaultOccurrence::TheNthMatchingCall(3),
    }
}

/// 取号写之后那道池屏障起每一道都报错（全池第 3 次屏障调用起）：回卷写之前那道同样报错。
fn fail_every_barrier_from_the_one_after_the_acquisition_writes() -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::BarrierFails,
        device: FaultDeviceSelector::EveryDevice,
        placement: FaultPlacement::AnyOffset,
        counting: FaultCounting::AcrossThePool,
        occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(3),
    }
}

/// 这块盘上每一次系统配置槽写都报错（两槽住在偏移 0 起的 `SYSTEM_CONFIGURATION_SLOTS_END_OFFSET` 之内）。
fn fail_every_system_configuration_write_on(device: DeviceIdentity) -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::OnlyDevice(device),
        placement: FaultPlacement::OffsetBelow(SYSTEM_CONFIGURATION_SLOTS_END_OFFSET),
        counting: FaultCounting::AcrossThePool,
        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
    }
}

fn wrap(built: &mut BuiltPool) -> WrappedDevices {
    let plan = SharedFaultPlan::unarmed(geometry());
    let devices = built
        .devices
        .take()
        .expect("第一个事务写完，盘还开着")
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, inner, plan.clone()),
            )
        })
        .collect();
    WrappedDevices { plan, devices }
}

/// 一块盘两槽里自证过的系统配置：(世代号, 实例代号)，按世代号排好。
fn slots_of(
    devices: &[(DeviceIdentity, FaultInjectingDevice)],
    device: DeviceIdentity,
) -> Vec<(u64, InstanceGeneration)> {
    let parameters = parameters();
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let mut slots: Vec<(u64, InstanceGeneration)> = verified_system_configuration_slots(
        devices,
        device,
        spacing,
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

fn acquire(
    devices: &mut [(DeviceIdentity, FaultInjectingDevice)],
) -> Result<InstanceGeneration, singlefs_core::transaction::AcquisitionFailed> {
    let parameters = parameters();
    let mut pool = PoolWriter::new(&parameters, devices);
    acquire_instance(&mut pool).map_err(|failure| match failure {
        InstanceAcquisitionFailed::Acquisition(acquisition) => acquisition,
        InstanceAcquisitionFailed::DeviceWithoutASelfVerifiedSystemConfigurationWhenReadingTheWitness {
            device,
        } => panic!("这条用例里每块盘两槽都读得出自证过的系统配置，取号不该拒在见证值那一核：{device:?}"),
    })
}

const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

#[test]
fn second_acquisition_writes_generation_six_on_both_disks_and_the_next_acquisition_gets_three() {
    let mut built = build_pool("acquire-second");
    let WrappedDevices {
        plan: _,
        mut devices,
    } = wrap(&mut built);
    assert_eq!(
        acquire(&mut devices).expect("第二次取号"),
        InstanceGeneration(2)
    );
    for disk in DISKS {
        assert_eq!(
            slots_of(&devices, disk),
            vec![(5, InstanceGeneration(1)), (6, InstanceGeneration(2))],
            "{disk:?}：第一个事务留下世代 5，第二次取号按那块盘自证过的最大世代号 + 1 写世代 6（D22（单元原子性怎么合成） 已定项 16 逐盘计）"
        );
    }
    assert_eq!(
        choose_system_configuration(&devices)
            .expect("择系统配置")
            .quantities
            .journal_instance,
        InstanceGeneration(2),
        "择到的那一份带新号：取号的世代号若恒为 2，会被世代 5 的旧槽藏住、这里读回 1"
    );
    assert_eq!(
        acquire(&mut devices).expect("第三次取号"),
        InstanceGeneration(3)
    );
}

#[test]
fn failed_barrier_after_acquisition_rolls_both_disks_back_and_the_skipped_number_is_never_handed_out_again(
) {
    let mut built = build_pool("acquire-barrier-error");
    let WrappedDevices { plan, mut devices } = wrap(&mut built);
    plan.arm(fail_the_barrier_after_the_acquisition_writes_once());
    let failure =
        acquire(&mut devices).expect_err("取号之后那道屏障报错，取号必须失败、不交出新号");
    assert!(
        matches!(failure.rollback, AcquisitionRollback::RolledBack),
        "两份取号写都报过 Ok，都要回卷（回卷写之前那道屏障这一次不报错）：{failure:?}"
    );
    for disk in DISKS {
        assert_eq!(
            slots_of(&devices, disk),
            vec![(6, InstanceGeneration(2)), (7, InstanceGeneration(1))],
            "{disk:?}：取号写世代 6 带新号 2，回卷写世代 7 带取号之前全部自证过的槽中最大的号 1"
        );
    }
    plan.disarm();
    assert_eq!(
        acquire(&mut devices).expect("屏障好了再取号"),
        InstanceGeneration(3),
        "全部自证过的槽里还躺着号 2：跳过它；只读择到的那一份（世代 7、号 1）就会再发一次 2"
    );
}

/// 取号写之前那道屏障报错（代码审阅第 19 条加的那一道：取号写覆写较旧那一槽之前，另一槽里上一次轮换写的先持久）：
/// 一个取号写都没发，取号失败、没有要回卷的（`NothingWritten`），两块盘两槽照旧是第一个事务留下的世代 4、5；屏障好了再取号取到 2。
#[test]
fn failed_barrier_before_the_acquisition_writes_writes_nothing_and_the_number_is_handed_out_later()
{
    let mut built = build_pool("acquire-barrier-before-error");
    let WrappedDevices { plan, mut devices } = wrap(&mut built);
    plan.arm(fail_every_barrier());
    let failure = acquire(&mut devices).expect_err("取号写之前那道屏障报错，取号必须失败");
    assert!(
        matches!(failure.rollback, AcquisitionRollback::NothingWritten),
        "一个取号写都没发：{failure:?}"
    );
    for disk in DISKS {
        assert_eq!(
            slots_of(&devices, disk),
            vec![(4, InstanceGeneration(1)), (5, InstanceGeneration(1))],
            "{disk:?}：两槽不动"
        );
    }
    plan.disarm();
    assert_eq!(
        acquire(&mut devices).expect("屏障好了再取号"),
        InstanceGeneration(2),
        "上一次一个字节没写，号 2 没被烧掉"
    );
}

/// 取号写之后那道屏障起每一道都报错：回卷写之前那道屏障（代码审阅第 19 条加的：回卷写覆写取号之前最新的那一槽之前，刚写的取号写先持久）
/// 同样报错，回卷不写（`RollbackFailed`，D18（块里携带什么信息） 已定项 11「回卷不成才只读挂载」）。两块盘上留着带新号 2 的世代 6，
/// 世代 5 那一槽没被覆写；屏障好了再取号跳过 2、取 3。
#[test]
fn barrier_failing_again_before_the_rollback_writes_leaves_the_new_number_and_it_is_never_handed_out_again(
) {
    let mut built = build_pool("acquire-barrier-rollback-error");
    let WrappedDevices { plan, mut devices } = wrap(&mut built);
    plan.arm(fail_every_barrier_from_the_one_after_the_acquisition_writes());
    let failure = acquire(&mut devices).expect_err("取号之后那道屏障报错，取号必须失败");
    assert!(
        matches!(failure.rollback, AcquisitionRollback::RollbackFailed(_)),
        "回卷写之前那道屏障也报错，回卷不写：{failure:?}"
    );
    for disk in DISKS {
        assert_eq!(
            slots_of(&devices, disk),
            vec![(5, InstanceGeneration(1)), (6, InstanceGeneration(2))],
            "{disk:?}：取号写世代 6 带新号 2 留着，世代 5 那一槽没被回卷写覆写"
        );
    }
    plan.disarm();
    assert_eq!(
        acquire(&mut devices).expect("屏障好了再取号"),
        InstanceGeneration(3),
        "两槽里躺着号 2：跳过它"
    );
}

#[test]
fn failed_system_configuration_write_on_the_second_disk_rolls_the_first_disk_back_and_leaves_the_second_untouched(
) {
    let mut built = build_pool("acquire-write-error");
    let WrappedDevices { plan, mut devices } = wrap(&mut built);
    plan.arm(fail_every_system_configuration_write_on(DeviceIdentity(1)));
    let failure = acquire(&mut devices).expect_err("盘 1 的取号写报错，取号必须失败");
    assert!(
        matches!(failure.rollback, AcquisitionRollback::RolledBack),
        "盘 0 那份已写出，要回卷：{failure:?}"
    );
    assert_eq!(
        slots_of(&devices, DeviceIdentity(0)),
        vec![(6, InstanceGeneration(2)), (7, InstanceGeneration(1))]
    );
    assert_eq!(
        slots_of(&devices, DeviceIdentity(1)),
        vec![(4, InstanceGeneration(1)), (5, InstanceGeneration(1))],
        "盘 1 一个字节都没写成"
    );
}

#[test]
fn barrier_right_after_the_acquisition_barrier_is_not_sent_to_the_devices() {
    let mut built = build_pool("acquire-barrier-skip");
    let WrappedDevices { plan, mut devices } = wrap(&mut built);
    {
        let parameters = parameters();
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        acquire_instance(&mut pool).expect("取号");
        pool.perform_commit_step(CommitStep::Barrier)
            .expect("紧跟着的一道屏障");
    }
    for (identity, _) in &devices {
        assert_eq!(
            plan.counts_of_device(*identity).barriers_forwarded,
            2,
            "{identity:?}：取号写之前一道（新开的写入口按「发过写」起步，代码审阅第 19 条）、取号写之后一道；紧跟着的那道前面没有写，不再发——\
             首次挂载路径上暖机开场那道就这样并掉"
        );
    }
}
