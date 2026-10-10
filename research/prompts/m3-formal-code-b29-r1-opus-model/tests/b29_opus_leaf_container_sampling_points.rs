//! 攻方原型（m3-formal-code-b29-r1-opus，L1）：27-2 的 O1 跨叶容器用例换取样点——建到第 4 片；回退目标那一版有第 3 片而现行那一版重写过它；
//! 现行那一版有目标那一版没有的第 4 片。只在攻方副本上跑；每一步 checker 不跑（同 27-2 那两条）。

use singlefs_core::admission::SpaceAdmission;
use singlefs_format::INODE_LEAF_RECORDS;
use singlefs_harness::history::{
    execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker, RollbackTargetChoice,
};
use singlefs_harness::SharedStream;

fn run(creations_before: u64, rollback: Option<u64>, creations_after: u64) {
    let mut operations = vec![HistoryOperation::Unlink];
    operations.extend(std::iter::repeat_n(HistoryOperation::CreateInode, usize::try_from(creations_before).unwrap()));
    if let Some(index_from_newest) = rollback {
        operations.push(HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest }));
    }
    operations.extend(std::iter::repeat_n(HistoryOperation::CreateInode, usize::try_from(creations_after).unwrap()));
    let history = GeneratedHistory { seed: HistorySeed(0), starting_point: HistoryStartingPoint::AfterFirstFile, operations };
    let result = execute_history_with(
        &history,
        HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: HistoryDeviceWidth::FourGibibytes, space_admission: SpaceAdmission::JudgedByTheFormula },
        &SharedStream::new(),
        &mut |_| {},
    );
    println!("steps={} outcomes={} ending={:?}", history.operations.len(), result.outcomes.len(), result.ending);
    println!("last_outcome={:?}", result.outcomes.last());
    assert_eq!(result.ending, HistoryEnding::Completed);
    assert_eq!(result.outcomes.len(), history.operations.len());
}

/// 建到第 4 片（3 × 233 + 4 条）。
#[test]
fn creations_into_the_fourth_leaf_container() {
    run(3 * INODE_LEAF_RECORDS + 4, None, 0);
}

/// 第 3 片 5 条时回退到第 3 新那条根（第 3 片 3 条），再建 3 个：回退要把第 3 片的分配代整组取回目标那一版的。
#[test]
fn rollback_to_a_version_whose_third_leaf_container_was_rewritten_since() {
    run(2 * INODE_LEAF_RECORDS + 5, Some(2), 3);
}

/// 第 4 片 2 条时回退到第 3 新那条根（3 片、第 3 片满），再建 3 个：回退要丢掉第 4 片。
#[test]
fn rollback_to_a_version_without_the_fourth_leaf_container() {
    run(3 * INODE_LEAF_RECORDS + 2, Some(2), 3);
}
