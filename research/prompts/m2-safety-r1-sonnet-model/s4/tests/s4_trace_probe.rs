//! S4 探针（m2-safety-r1 云端正推腿，副本专用，不入库）：在 admission.rs / mount.rs / transaction.rs 打了
//! SONNET_S4_TRACE 开关的 eprintln 之后，跑一段与 Z19-B 同形的历史（小盘、反复覆盖写、下一次可写挂载），
//! 把每一次准入检查读到的九项各自打出来，找出发布准入与挂载准入在同一块盘上算出不同结论的那一项。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports)]

use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
};
use singlefs_harness::SharedStream;

fn overwrite(selector: u64, fill_seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector },
        fill_seed,
    })
}

#[test]
fn s4_trace_next_mount_after_admitted_overwrites() {
    std::env::set_var("SONNET_S4_TRACE", "1");
    let k = 6u64;
    let mut operations = vec![HistoryOperation::CloseAndMountWritable];
    operations.extend((0..k).map(|i| overwrite(2999, i + 1)));
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf240Slots,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &SharedStream::new(),
        &mut |_| {},
    );
    for (index, outcome) in run.outcomes.iter().enumerate() {
        eprintln!("S4TRACE step={index} outcome={outcome:?}");
    }
}
