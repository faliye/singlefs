//! m2-final-code-r4 云端攻方 Z22：改过的用例还测不测原来那条性质。只在冻结副本的拷贝上跑。
//! formatted_pool 那一条（注入点从第 2 读改到第 7 读）：把注入点 n 从 1 扫到 12，看每个 n 上挂载交回什么；
//! 副本上的变异开关 `R4_OPUS_MUTATION=skip-recompute`（`acquire_expected_instance` 不重算、照判定的号写）下再扫一遍，
//! 看第 7 读那一格还分不分得出「取号写之前重算」这一判。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports)]

mod common;

use common::{disk_snapshot, format_pool, geometry, parameters};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::transaction::{acquire_instance, PoolWriter};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};

fn fail_the_nth_read_of_system_configuration_slot_zero_on_each_device(ordinal: u64) -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::ReadFails,
        device: FaultDeviceSelector::EveryDevice,
        placement: FaultPlacement::OffsetExactly(DeviceOffsetInBytes(0)),
        counting: FaultCounting::PerDevice,
        occurrence: FaultOccurrence::TheNthMatchingCall(ordinal),
    }
}

/// 用例原样的历史（mkfs 之后取号 1 就停），注入点取 `n`。交回挂载的结局、盘面变没变。
fn run_at(n: u64) -> (String, bool) {
    let mut formatted = format_pool(&format!("r4-z22-formatted-{n}"));
    {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        assert_eq!(acquire_instance(&mut writer).expect("取号"), InstanceGeneration(1));
    }
    let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    let plan = SharedFaultPlan::armed(
        geometry(),
        fail_the_nth_read_of_system_configuration_slot_zero_on_each_device(n),
    );
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<_>)> = formatted
        .reopen_recorded()
        .into_iter()
        .map(|(identity, recorded)| {
            (identity, FaultInjectingBlockDevice::new(identity, recorded, plan.clone()))
        })
        .collect();
    let result = mount_writable(&parameters(), &mut devices);
    formatted.devices = Some(
        devices
            .into_iter()
            .map(|(identity, device)| (identity, device.into_inner()))
            .collect(),
    );
    let unchanged = disk_snapshot(&formatted.memory_pool(), &formatted.stream) == before;
    let described = match &result {
        Ok(mounted) => format!("Ok(instance={})", mounted.output.instance.0),
        Err(MountError::InstanceGenerationChangedBeforeAcquisition { expected, recomputed }) => format!(
            "InstanceGenerationChangedBeforeAcquisition{{expected:{},recomputed:{}}}",
            expected.0, recomputed.0
        ),
        Err(other) => format!("Err({other:?})").chars().take(160).collect(),
    };
    (described, unchanged)
}

#[test]
fn z22_formatted_pool_injection_point_sweep() {
    let mutation = std::env::var("R4_OPUS_MUTATION").unwrap_or_default();
    for n in 1..=12u64 {
        let (described, unchanged) = run_at(n);
        let the_original_assertions_hold =
            described == "InstanceGenerationChangedBeforeAcquisition{expected:1,recomputed:2}" && unchanged;
        println!(
            "Z22 mutation={mutation:?} n={n} -> {described} disk_snapshot_unchanged={unchanged} original_test_assertions_hold={the_original_assertions_hold}"
        );
    }
}
