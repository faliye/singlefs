//! 攻方腿 m4-exit5-relocation-proof-r1 的原型（副本上跑，不入库）：同一段历史跑两份——一份带挂着时整理，一份去掉整理那一步——
//! 整理之后接一步由用户定的操作（回退到根环里从新往旧第 i 条根、抬 F 若干步），把这一步的结局类与之后冷读出的文件逐份打出来。
//! 用户定的那一步放开扫（i 取 0..8，抬 F 取 0..6），只固定前缀（两种：一次或两次覆盖写，中间各重挂一次）与整理那一步的参数（批大小 1、2，选段 0、1；
//! 选段 0 在这两种前缀上一批都不发，是对照）。草稿上另跑过选段 2、3（2 与 0 同为不发，3 与 1 逐行同类），正式取样缩成 0、1。
//! 只用 singlefs-harness 的公开入口；不碰 checker 档。

use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_harness::history::{
    execute_history_with, AppliedEffect, CompactionChoice, CompactionRunEnding, ContentChoice,
    ContentLength, FloorTargetChoice, GeneratedHistory, HistoryExecution, HistoryOperation,
    HistoryOperationKind, HistorySeed, HistoryStartingPoint, RollbackTargetChoice, StepOutcome,
};
use singlefs_harness::SharedStream;

fn overwrite(fill_seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 300 },
        fill_seed,
    })
}

fn fnv(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn class(outcome: &StepOutcome) -> String {
    match outcome {
        StepOutcome::Applied(AppliedEffect::Compacted { batches_published, units_relocated, stop, .. }) => {
            format!("Applied(Compacted batches={batches_published}) units={units_relocated} stop={stop}")
        }
        StepOutcome::Applied(AppliedEffect::RolledBack { target, .. }) => {
            format!("Applied(RolledBack target={target:?})")
        }
        StepOutcome::Applied(AppliedEffect::RaisedFloor { new_floor, .. }) => format!("Applied(RaisedFloor) new_floor={}", new_floor.0),
        StepOutcome::Applied(other) => format!("Applied({})", format!("{other:?}").split([' ', '{', '(']).next().unwrap_or("")),
        StepOutcome::Refused { member } => format!("Refused({member})"),
        StepOutcome::NotApplicable(precondition) => format!("NotApplicable({})", precondition.name()),
    }
}

/// 跑一段历史，交回每一步的（结局类、之后冷读出的文件摘要）。
fn run(history: &GeneratedHistory) -> Vec<(String, String)> {
    let mut seen = Vec::new();
    let _ = execute_history_with(
        history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            let file = match recover(observation.image, JournalPolicy::Consult).outcome {
                RecoveryOutcome::FileRead { content, .. } => format!("file:{:016x}", fnv(&content)),
                RecoveryOutcome::NoFile { .. } => "nofile".to_string(),
                RecoveryOutcome::Failed { .. } => "failed".to_string(),
            };
            seen.push((observation.outcome.map(class).unwrap_or_else(|| "start".to_string()), file));
        },
    );
    seen
}

fn twin(name: &str, prefix: &[HistoryOperation], compaction: HistoryOperation, last: HistoryOperation) -> (bool, u64) {
    let mut with_ops = prefix.to_vec();
    with_ops.push(compaction);
    with_ops.push(last);
    let without_ops: Vec<HistoryOperation> = with_ops
        .iter()
        .filter(|operation| operation.kind() != HistoryOperationKind::CompactWhileMounted)
        .copied()
        .collect();
    let history_of = |operations: Vec<HistoryOperation>| GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    let with = run(&history_of(with_ops));
    let without = run(&history_of(without_ops));
    let compaction_step = with.get(prefix.len() + 1).map(|step| step.0.clone()).unwrap_or_default();
    let with_last = with.last().cloned().unwrap_or_default();
    let without_last = without.last().cloned().unwrap_or_default();
    let same_class = with_last.0 == without_last.0;
    let same_file = with_last.1 == without_last.1;
    println!(
        "name=twin case={name} compaction=\"{compaction_step}\" with=\"{} {}\" without=\"{} {}\" same_outcome_class={same_class} same_file={same_file}",
        with_last.0, with_last.1, without_last.0, without_last.1
    );
    let batches = compaction_step
        .strip_prefix("Applied(Compacted batches=")
        .and_then(|rest| rest.split(')').next())
        .and_then(|number| number.parse::<u64>().ok())
        .unwrap_or(0);
    (same_class && same_file, batches)
}

#[test]
fn compaction_then_a_user_chosen_rollback_or_floor_raise_against_the_same_history_without_it() {
    let prefixes: Vec<(&str, Vec<HistoryOperation>)> = vec![
        ("p1", vec![HistoryOperation::CloseAndMountWritable, overwrite(21), HistoryOperation::CloseAndMountWritable]),
        ("p2", vec![
            HistoryOperation::CloseAndMountWritable,
            overwrite(21),
            HistoryOperation::CloseAndMountWritable,
            overwrite(22),
            HistoryOperation::CloseAndMountWritable,
        ]),
    ];
    let mut cases = 0_u64;
    let mut differing = 0_u64;
    let mut with_batches = 0_u64;
    for (prefix_name, prefix) in &prefixes {
    for batch_units in [1_u64, 2] {
        for region_selector in [0_u64, 1] {
            let compaction = HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units,
                region_selector,
                ending: CompactionRunEnding::RunsUntilItStops,
            });
            let mut lasts: Vec<(String, HistoryOperation)> = Vec::new();
            for index_from_newest in 0_u64..8 {
                lasts.push((
                    format!("{prefix_name}-k{batch_units}-r{region_selector}-rollback-ring-root-{index_from_newest}"),
                    HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest }),
                ));
            }
            for steps_above_current_floor in 0_u64..6 {
                lasts.push((
                    format!("{prefix_name}-k{batch_units}-r{region_selector}-raise-floor-{steps_above_current_floor}"),
                    HistoryOperation::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor }),
                ));
            }
            for (name, last) in lasts {
                cases += 1;
                let (same, batches) = twin(&name, prefix, compaction, last);
                if batches > 0 {
                    with_batches += 1;
                }
                if !same {
                    differing += 1;
                }
            }
        }
    }
    }
    println!("name=twin-summary cases={cases} with_batches={with_batches} differing={differing}");
}

/// 不靠根环下标、不靠 txg 编号的比法：抬 F 若干步之后，回退能读回的文件内容集合（对根环里从新往旧第 0..8 条根各回退一次、做成的那几次冷读出的文件）。
/// 两份历史里 txg 与根环下标都不同，比的只是「用户能退回到哪几种内容」。
fn contents_reachable_by_rollback(prefix: &[HistoryOperation], compaction: Option<HistoryOperation>, floor_steps: u64) -> (std::collections::BTreeSet<String>, String) {
    let mut reachable = std::collections::BTreeSet::new();
    let mut floor_outcome = String::new();
    for index_from_newest in 0_u64..8 {
        let mut operations = prefix.to_vec();
        if let Some(compaction) = compaction {
            operations.push(compaction);
        }
        operations.push(HistoryOperation::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: floor_steps }));
        operations.push(HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest }));
        let seen = run(&GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations,
        });
        floor_outcome = seen[seen.len() - 2].0.clone();
        let last = seen.last().cloned().unwrap_or_default();
        if last.0.starts_with("Applied(RolledBack") {
            reachable.insert(last.1);
        }
    }
    (reachable, floor_outcome)
}

#[test]
fn contents_reachable_by_rollback_after_a_floor_raise_with_and_without_the_compaction() {
    let prefixes: Vec<(&str, Vec<HistoryOperation>)> = vec![
        ("p1", vec![HistoryOperation::CloseAndMountWritable, overwrite(21), HistoryOperation::CloseAndMountWritable]),
        ("p2", vec![
            HistoryOperation::CloseAndMountWritable,
            overwrite(21),
            HistoryOperation::CloseAndMountWritable,
            overwrite(22),
            HistoryOperation::CloseAndMountWritable,
        ]),
    ];
    let mut cases = 0_u64;
    let mut differing = 0_u64;
    for (prefix_name, prefix) in &prefixes {
        for region_selector in [1_u64] {
            let compaction = HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 1,
                region_selector,
                ending: CompactionRunEnding::RunsUntilItStops,
            });
            for floor_steps in 0_u64..6 {
                let (with, with_floor) = contents_reachable_by_rollback(prefix, Some(compaction), floor_steps);
                let (without, without_floor) = contents_reachable_by_rollback(prefix, None, floor_steps);
                cases += 1;
                let same = with == without;
                if !same {
                    differing += 1;
                }
                println!(
                    "name=reachable case={prefix_name}-k1-r{region_selector}-floor-{floor_steps} with_floor=\"{with_floor}\" without_floor=\"{without_floor}\" with={with:?} without={without:?} same_set={same}"
                );
            }
        }
    }
    println!("name=reachable-summary cases={cases} differing={differing}");
}
