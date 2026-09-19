//! 三方正推腿（sonnet, m2-supp3-item1-code-r1）：独立复现里程碑「第二个事务」增补 2 收口表第 43 行——
//! 同一段 11 步历史只换抬 F 的目标（steps_above_current_floor），F = 7..12 分别是红是不红。
//! 这份文件只在草稿副本上使用，不进主工作区。

use singlefs_harness::history::{
    execute_history, AppliedEffect, ContentChoice, ContentLength, FloorTargetChoice,
    GeneratedHistory, HistoryEnding, HistoryOperation, HistorySeed, HistoryStartingPoint,
    RollbackTargetChoice, StepOutcome,
};

fn history_with_floor_target(steps_above_current_floor: u64) -> GeneratedHistory {
    let empty = ContentChoice {
        length: ContentLength::Empty,
        fill_seed: 0,
    };
    GeneratedHistory {
        seed: HistorySeed(80),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                index_from_newest: 0,
            }),
            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                index_from_newest: 3,
            }),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor,
            }),
        ],
    }
}

#[test]
fn sonnet_r1_probe_row43_floor_targets_seven_through_twelve() {
    for target in 7u64..=12u64 {
        let history = history_with_floor_target(target);
        let run = execute_history(&history);
        let ending = match &run.ending {
            HistoryEnding::Completed => "Completed".to_string(),
            HistoryEnding::KnownRed { form, .. } => format!("KnownRed(form={form})"),
            HistoryEnding::NewFinding { signature, .. } => format!("NewFinding({signature:?})"),
        };
        let new_floor = if let StepOutcome::Applied(AppliedEffect::RaisedFloor {
            new_floor, ..
        }) = &run.outcomes[10]
        {
            Some(*new_floor)
        } else {
            None
        };
        println!(
            "steps_above_current_floor={target} -> new_floor={new_floor:?} ending={ending} outcome_at_10={:?}",
            run.outcomes.get(10)
        );
    }
}
