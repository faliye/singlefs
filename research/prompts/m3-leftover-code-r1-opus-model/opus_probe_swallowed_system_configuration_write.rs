//! 攻方腿副本探针（m3-leftover-code-r1-opus，K3 / C600）：起点段（mkfs、取号、暖机、第一个文件）里某块盘第 n 次系统配置槽写被设备吞掉，
//! 之后关掉会话、可写挂载、覆盖写、再挂载：每一步实现交回什么。
use singlefs_core::address::DeviceIdentity;
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultOccurrence, FaultPlacement, FaultSchedule,
    InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::{
    execute_history_with_faults, ContentChoice, ContentLength, GeneratedHistory,
    HistoryDeviceWidth, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
};
use singlefs_harness::SharedStream;

fn overwrite(fill_seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 100 + fill_seed },
        fill_seed,
    })
}

#[test]
fn opus_probe_swallowed_system_configuration_slot_write_in_the_starting_segment() {
    let width = HistoryDeviceWidth::FourGibibytes;
    let spacing = u64::from(width.parameters().geometry.fixed_structure_slot_spacing);
    let operations_list: Vec<(&str, Vec<HistoryOperation>)> = vec![
        ("mount", vec![HistoryOperation::CloseAndMountWritable, overwrite(1), HistoryOperation::CloseAndMountWritable]),
        ("overwrite-then-mount", vec![overwrite(2), HistoryOperation::CloseAndMountWritable, overwrite(3), HistoryOperation::CloseAndMountWritable]),
    ];
    for (shape, operations) in &operations_list {
        for device in [0u32, 1] {
            for n in 1..=8u64 {
                let plan = SharedFaultPlan::armed(
                    width.fixed_geometry(),
                    FaultSchedule {
                        fault: InjectedFault::WriteIsSwallowed,
                        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(device)),
                        placement: FaultPlacement::OffsetBelow(2 * spacing),
                        counting: FaultCounting::PerDevice,
                        occurrence: FaultOccurrence::TheNthMatchingCall(n),
                    },
                );
                let history = GeneratedHistory {
                    seed: HistorySeed(n),
                    starting_point: HistoryStartingPoint::AfterFirstFile,
                    operations: operations.clone(),
                };
                let run = execute_history_with_faults(
                    &history,
                    HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
                    &SharedStream::new(),
                    &plan,
                    &mut |_| {},
                );
                let fired: Vec<String> = plan
                    .fired()
                    .iter()
                    .map(|fired| format!("dev{} off{} {:?}", fired.device.0, fired.offset.0, fired.written_structure))
                    .collect();
                let outcomes: Vec<String> = run.outcomes.iter().map(|outcome| format!("{outcome:?}")).collect();
                let ending = format!("{:?}", run.ending);
                eprintln!(
                    "name=opus-c600 shape={shape} device={device} n={n} fired={fired:?} outcomes={outcomes:?} ending={}",
                    &ending[..ending.len().min(900)]
                );
            }
        }
    }
}

#[test]
fn opus_probe_swallowed_make_filesystem_slot_write_then_writable_mount() {
    let width = HistoryDeviceWidth::FourGibibytes;
    let spacing = u64::from(width.parameters().geometry.fixed_structure_slot_spacing);
    for device in [0u32, 1] {
        for n in 1..=4u64 {
            let plan = SharedFaultPlan::armed(
                width.fixed_geometry(),
                FaultSchedule {
                    fault: InjectedFault::WriteIsSwallowed,
                    device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(device)),
                    placement: FaultPlacement::OffsetBelow(2 * spacing),
                    counting: FaultCounting::PerDevice,
                    occurrence: FaultOccurrence::TheNthMatchingCall(n),
                },
            );
            let history = GeneratedHistory {
                seed: HistorySeed(n),
                starting_point: HistoryStartingPoint::AfterMakeFilesystem,
                operations: vec![HistoryOperation::CloseAndMountWritable, overwrite(5), HistoryOperation::CloseAndMountWritable],
            };
            let run = execute_history_with_faults(
                &history,
                HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
                &SharedStream::new(),
                &plan,
                &mut |_| {},
            );
            let fired: Vec<String> = plan
                .fired()
                .iter()
                .map(|fired| format!("dev{} off{} len{} {:?}", fired.device.0, fired.offset.0, fired.length, fired.written_structure))
                .collect();
            let outcomes: Vec<String> = run.outcomes.iter().map(|outcome| format!("{outcome:?}")).collect();
            let ending = format!("{:?}", run.ending);
            eprintln!(
                "name=opus-c600-mkfs device={device} n={n} fired={fired:?} outcomes={outcomes:?} ending={}",
                &ending[..ending.len().min(1500)]
            );
        }
    }
}
