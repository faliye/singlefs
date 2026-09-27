//! 代码审阅第 19 条（实审 A1）：两槽轮换（D22（单元原子性怎么合成） 已定项 16：下一次写的槽 = 世代号 mod 2，择槽取校验和过且世代号最大的）
//! 靠的是「覆写较旧那一槽时，另一槽里新写的那一份已经持久」。每次发布末尾的轮换前面有发布自己的屏障，这一条自然成立；
//! 取号、抬 F 先写系统配置、取号失败的回卷这三处系统配置写，前面紧挨着的是上一次写（上一次发布末尾的轮换、刚写的取号写），
//! 中间没有屏障——同一块盘上两个槽的写落在同一段里，崩溃时两槽可以一起撕坏（撕裂并进「没持久」的层 0 枚举看不到这一格）。
//! 改成：这三处第一个系统配置写之前各发一道池屏障。
//!
//! 判据按录制流量：mkfs 之后切段（屏障切段），没有一段里同一块盘上有两次系统配置槽写。mkfs 那一段两槽都写世代号 1（新池，
//! 没有旧的一份要保），不在判的范围里。

mod common;

use common::{
    build_pool, geometry, parameters, publish_overwrite_in_process, BuiltPool, Recorded,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::mount::{mount_writable, raise_rollback_floor, ShadowLedger};
use singlefs_core::transaction::{
    acquire_instance, AcquisitionRollback, InstanceAcquisitionFailed, PoolWriter,
};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::RecordedOperation;

/// 一次违例：录制流里第 `operation_index` 步（从整条流的第 0 步数）是 `device` 在这一段里的第二次系统配置槽写。
#[derive(Debug, PartialEq, Eq)]
struct SecondSystemConfigurationWriteInOneSegment {
    operation_index: usize,
    device: DeviceIdentity,
}

/// mkfs 之后（`from` 起）的录制流里，同一段（两道屏障之间）同一块盘上的第二次系统配置槽写，按流里的次序。
///
/// 循环：按流逐步（轮数 = 步数），跨轮携带「这一段里已经写过系统配置槽的盘」，遇屏障清空；没有提前出口。
fn second_system_configuration_writes_in_one_segment(
    operations: &[RecordedOperation],
    from: usize,
) -> Vec<SecondSystemConfigurationWriteInOneSegment> {
    let geometry = geometry();
    let mut devices_written_in_this_segment: Vec<DeviceIdentity> = Vec::new();
    let mut violations = Vec::new();
    for (operation_index, operation) in operations.iter().enumerate().skip(from) {
        match geometry.classify(operation) {
            StepKind::Barrier => devices_written_in_this_segment.clear(),
            StepKind::SystemConfigurationSlot => {
                if devices_written_in_this_segment.contains(&operation.device) {
                    violations.push(SecondSystemConfigurationWriteInOneSegment {
                        operation_index,
                        device: operation.device,
                    });
                } else {
                    devices_written_in_this_segment.push(operation.device);
                }
            }
            StepKind::UnitWrite
            | StepKind::JournalRecord
            | StepKind::RootRecordFua
            | StepKind::ZeroFill => {}
        }
    }
    violations
}

fn second_content(seed: usize) -> Vec<u8> {
    (0..4100)
        .map(|index| u8::try_from((index * 7 + 3 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 第一个事务（txg 3）之后在同一个进程里覆盖写 `times` 次（txg 4 起）。
fn overwrite_times(pool: &mut BuiltPool, times: usize) {
    for seed in 0..times {
        let previous = pool.output.clone();
        pool.output = publish_overwrite_in_process(
            pool,
            &previous,
            &second_content(seed),
            FIXED_WRITE_TIME_SECONDS + 60,
            InstanceGeneration(1),
        )
        .expect("覆盖写");
    }
}

/// 取号：覆盖写 B（txg 4）之后进程退出、重开可写挂载（取号 2）。B 末尾的轮换写了两块盘各一槽，取号写覆写的是各盘另一槽；
/// 两者之间要有一道屏障。改之前两次写同在一段（第二条流里 B 的轮换与重开取号合成的 4 写一段）。
#[test]
fn acquisition_after_a_publish_of_the_previous_process_overwrites_the_older_slot_only_behind_a_barrier(
) {
    let mut pool = build_pool("a1-barrier-acquisition");
    overwrite_times(&mut pool, 1);
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开之后可写挂载");
    pool.devices = Some(devices);
    assert_eq!(mounted.output.instance, InstanceGeneration(2), "重开取号 2");
    assert_eq!(
        second_system_configuration_writes_in_one_segment(
            &pool.stream.operations(),
            pool.mkfs_operation_count
        ),
        Vec::new(),
        "mkfs 之后没有一段里同一块盘写两次系统配置槽"
    );
}

/// 抬 F 先写系统配置：第一个事务之后覆盖写三次（txg 4–6），抬 F 到 3。txg 6 末尾的轮换与抬 F 先写的那一次系统配置写之间要有一道屏障。
/// 改之前两次写同在一段（第二条流里 txg 14 的轮换与抬 F 先写系统配置合成的 4 写一段）。
#[test]
fn raising_the_floor_overwrites_the_older_system_configuration_slot_only_behind_a_barrier() {
    let mut pool = build_pool("a1-barrier-raise");
    overwrite_times(&mut pool, 3);
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        CheckpointTxg(3),
        ShadowLedger::On,
    )
    .expect("抬到上限 3");
    pool.output = current;
    assert_eq!(
        second_system_configuration_writes_in_one_segment(
            &pool.stream.operations(),
            pool.mkfs_operation_count
        ),
        Vec::new(),
        "mkfs 之后没有一段里同一块盘写两次系统配置槽"
    );
}

/// 取号失败的回卷：盘 1 的取号写报错，盘 0 的取号写已经写出、要回卷成旧代号（D18（块里携带什么信息） 已定项 11）。
/// 回卷写覆写的是盘 0 取号之前最新的那一槽，另一槽是刚写的取号写；两者之间要有一道屏障。回卷照常做成（`RolledBack`）。
#[test]
fn rolling_back_a_failed_acquisition_overwrites_the_older_slot_only_behind_a_barrier() {
    let mut pool = build_pool("a1-barrier-acquisition-rollback");
    let plan = SharedFaultPlan::unarmed(geometry());
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<Recorded>)> = pool
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
    plan.arm(FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(1)),
        placement: FaultPlacement::OffsetBelow(
            singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
                * u64::from(parameters().geometry.fixed_structure_slot_spacing),
        ),
        counting: FaultCounting::AcrossThePool,
        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
    });
    let publish_parameters = parameters();
    let failure = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        match acquire_instance(&mut writer).expect_err("盘 1 的取号写报错，取号必须失败") {
            InstanceAcquisitionFailed::Acquisition(acquisition) => acquisition,
            InstanceAcquisitionFailed::DeviceWithoutASelfVerifiedSystemConfigurationWhenReadingTheWitness {
                device,
            } => panic!("这条用例里每块盘两槽都读得出自证过的系统配置，取号不该拒在见证值那一核：{device:?}"),
        }
    };
    plan.disarm();
    assert!(
        matches!(failure.rollback, AcquisitionRollback::RolledBack),
        "盘 0 那份已写出，回卷做成：{failure:?}"
    );
    assert_eq!(
        second_system_configuration_writes_in_one_segment(
            &pool.stream.operations(),
            pool.mkfs_operation_count
        ),
        Vec::new(),
        "mkfs 之后没有一段里同一块盘写两次系统配置槽"
    );
}
