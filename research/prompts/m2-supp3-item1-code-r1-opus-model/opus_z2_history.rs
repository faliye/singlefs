//! 攻方腿（Opus）m2-supp3-item1-code-r1 · Z2：手写的一段历史，只放进仓副本的 `crates/singlefs-harness/tests/`。
//! 第一个文件之后可写挂载（实例 2，写行 txg 4、暖机 txg 5），覆盖写三次（txg 6、7、8），抬 F 到 3（选择子 3：3 mod (8 − 0 + 3) = 3）。
//! 一次回退都没有：F = 3 那个 txg 上是第一个文件那条有效根，不在任何回退留下的空档里。
//! 打印收尾（`OPUS-Z2 …`）；在基线上应跑完，在回收门槛差一的变异上看它被分到哪里。

use singlefs_harness::history::{
    execute_history, ContentChoice, ContentLength, FloorTargetChoice, GeneratedHistory, HistoryEnding,
    HistoryOperation, HistorySeed, HistoryStartingPoint,
};

#[test]
#[ignore = "探针：只在副本上跑"]
fn opus_z2_raise_without_any_rollback() {
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed: 1,
    });
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            overwrite,
            overwrite,
            overwrite,
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 3,
            }),
        ],
    };
    let run = execute_history(&history);
    for (index, outcome) in run.outcomes.iter().enumerate() {
        println!("OPUS-Z2 step {index} {:?} -> {outcome:?}", history.operations[index]);
    }
    match &run.ending {
        HistoryEnding::Completed => println!("OPUS-Z2 ending Completed"),
        HistoryEnding::KnownRed { form, observation } => println!(
            "OPUS-Z2 ending KnownRed form {form} at {:?} after {:?} newest_txg {:?} violations {:?}",
            observation.position, observation.operation_kind, observation.newest_ring_root_txg, observation.violations
        ),
        HistoryEnding::NewFinding { signature, observation } => println!(
            "OPUS-Z2 ending NewFinding {signature:?} at {:?} violations {:?} panic {:?}",
            observation.position, observation.violations, observation.panic
        ),
    }
}
