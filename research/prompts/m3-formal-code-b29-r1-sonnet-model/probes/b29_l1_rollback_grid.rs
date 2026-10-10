//! b29 正推腿 L1 探针：T13 形状的回退用例换取样点（建几个、回退到根环第几新、回退之后再建几个），
//! 每个格各开一个线程跑一段历史，打 `CELL` 行（不断言）。基线上全是 Completed；施加变异之后看哪几格不再 Completed。
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker, RollbackTargetChoice,
};
use singlefs_harness::SharedStream;

fn cell(before: usize, index_from_newest: u64, after: usize, unlink_first: bool) -> String {
    let mut operations = Vec::new();
    if unlink_first { operations.push(HistoryOperation::Unlink); }
    operations.extend(std::iter::repeat_n(HistoryOperation::CreateInode, before));
    operations.push(HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest }));
    operations.extend(std::iter::repeat_n(HistoryOperation::CreateInode, after));
    let history = GeneratedHistory { seed: HistorySeed(0), starting_point: HistoryStartingPoint::AfterFirstFile, operations };
    let run = execute_history_with(
        &history,
        HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: HistoryDeviceWidth::FourGibibytes, space_admission: SpaceAdmission::JudgedByTheFormula },
        &SharedStream::new(),
        &mut |_| {},
    );
    let ending = match &run.ending {
        HistoryEnding::Completed => "Completed".to_string(),
        other => format!("{other:?}").chars().take(160).collect(),
    };
    format!("CELL before={before} idx={index_from_newest} after={after} steps_done={} ending={ending}", run.outcomes.len())
}

#[test]
fn b29_l1_rollback_grid() {
    let spec = std::env::var("GRID").unwrap_or_else(|_| "468:2:3".to_string());
    let cells: Vec<(usize, u64, usize)> = spec.split(',').map(|c| { let p: Vec<&str> = c.split(':').collect(); (p[0].parse().unwrap(), p[1].parse().unwrap(), p[2].parse().unwrap()) }).collect();
    let handles: Vec<_> = cells.into_iter().map(|(b, i, a)| std::thread::spawn(move || cell(b, i, a, true))).collect();
    for h in handles { println!("{}", h.join().unwrap()); }
}
