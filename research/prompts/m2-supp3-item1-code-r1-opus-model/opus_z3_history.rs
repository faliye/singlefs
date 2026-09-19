//! 攻方腿（Opus）m2-supp3-item1-code-r1 · Z3：手写的几段历史，只放进仓副本的 `crates/singlefs-harness/tests/`。每一步的结局与收尾打成 `OPUS-Z3 …` 行。
//! ceiling：第一个文件之后可写挂载（txg 4、5），覆盖写三次（6、7、8），抬 F 到 7（选择子 7：7 mod 11），再回退到 txg 6 那条根（根环新到旧第 2 条）。
//!   非空有效根是 3、6、7、8，第 4 新的是 3 ⇒ 上限 3（D16（发布语义） 已定项 1）：抬到 7 该被拒。
//! cold：第一个文件之后冷启动（E142 的 3000 字节，末字节非零）。

use singlefs_harness::history::{
    execute_history, ContentChoice, ContentLength, FloorTargetChoice, GeneratedHistory, HistoryEnding,
    HistoryOperation, HistorySeed, HistoryStartingPoint, RollbackTargetChoice,
};

fn run_and_print(label: &str, history: &GeneratedHistory) {
    let run = execute_history(history);
    for (index, outcome) in run.outcomes.iter().enumerate() {
        println!("OPUS-Z3 {label} step {index} {:?} -> {outcome:?}", history.operations[index]);
    }
    match &run.ending {
        HistoryEnding::Completed => println!("OPUS-Z3 {label} ending Completed"),
        HistoryEnding::KnownRed { form, observation } => println!(
            "OPUS-Z3 {label} ending KnownRed form {form} at {:?} violations {:?}",
            observation.position, observation.violations
        ),
        HistoryEnding::NewFinding { signature, observation } => println!(
            "OPUS-Z3 {label} ending NewFinding {signature:?} at {:?} violations {:?} panic {:?}",
            observation.position, observation.violations, observation.panic
        ),
    }
}

#[test]
#[ignore = "探针：只在副本上跑"]
fn opus_z3_histories() {
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed: 1,
    });
    run_and_print(
        "ceiling",
        &GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![
                HistoryOperation::CloseAndMountWritable,
                overwrite,
                overwrite,
                overwrite,
                HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                    steps_above_current_floor: 7,
                }),
                HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                    index_from_newest: 4,
                }),
            ],
        },
    );
    run_and_print(
        "cold",
        &GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![HistoryOperation::ColdStartRecover],
        },
    );
}
