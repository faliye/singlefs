//! 攻方腿原型（m4-segment-code-r2，R5）：文件集合为空时建快照（F5）。写死前缀：起点第一个文件 → unlink（一次调用走完全部批，之后有树而文件集合为空）
//! → 建 n 个快照（n 放开扫 1–11，第 11 个要被上限拒）→ 之后接一步（放开扫：读快照、删快照、回退第 k 新、建 inode、可写挂载、冷启动、整理）
//! → 再建一个快照 → 读快照 → 可写挂载 → 冷启动。每一步过池级 checker、执行器逐步与理想模型对拍。
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_harness::history::{
    execute_history_with, CompactionChoice, CompactionRunEnding, GeneratedHistory, HistoryEnding,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, RollbackTargetChoice,
    SnapshotChoice,
};
use singlefs_harness::SharedStream;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

#[test]
fn snapshots_on_an_empty_file_set_sweep() {
    let verbose = env_u64("ATTACK_VERBOSE", 0) == 1;
    let only = std::env::var("ATTACK_ONLY").ok();
    let followers: Vec<(&str, HistoryOperation)> = vec![
        ("read-newest", HistoryOperation::ReadSnapshot(SnapshotChoice { index_among_the_snapshots: u64::MAX })),
        ("read-oldest", HistoryOperation::ReadSnapshot(SnapshotChoice { index_among_the_snapshots: 0 })),
        ("destroy-oldest", HistoryOperation::DestroySnapshot(SnapshotChoice { index_among_the_snapshots: 0 })),
        ("rollback-1", HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 1 })),
        ("rollback-6", HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 6 })),
        ("create-inode", HistoryOperation::CreateInode),
        ("mount", HistoryOperation::CloseAndMountWritable),
        ("cold-start", HistoryOperation::ColdStartRecover),
        ("compact", HistoryOperation::CompactWhileMounted(CompactionChoice { batch_units: 2, region_selector: 1, ending: CompactionRunEnding::RunsUntilItStops })),
    ];
    let mut cases = 0u64;
    let mut hits = 0u64;
    for snapshots in 1..=11_u64 {
        for (name, follower) in &followers {
            let case = format!("snapshots={snapshots} then={name}");
            if only.as_deref().is_some_and(|wanted| wanted != case) {
                continue;
            }
            let mut operations = vec![HistoryOperation::Unlink];
            operations.extend((0..snapshots).map(|_| HistoryOperation::CreateSnapshot));
            operations.push(*follower);
            operations.push(HistoryOperation::CreateSnapshot);
            operations.push(HistoryOperation::ReadSnapshot(SnapshotChoice { index_among_the_snapshots: 3 }));
            operations.push(HistoryOperation::CloseAndMountWritable);
            operations.push(HistoryOperation::ColdStartRecover);
            let history = GeneratedHistory {
                seed: HistorySeed(0),
                starting_point: HistoryStartingPoint::AfterFirstFile,
                operations,
            };
            let mut reds = Vec::new();
            let mut outcomes = Vec::new();
            let run = execute_history_with(
                &history,
                HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                &SharedStream::new(),
                &mut |observation| {
                    if let Some(outcome) = observation.outcome {
                        let text = format!("{outcome:?}");
                        outcomes.push(text.chars().take(90).collect::<String>());
                    }
                    for (invariant, verdict) in check_pool_image(observation.image) {
                        if let InvariantVerdict::Violated(why) = verdict {
                            if verbose {
                                eprintln!("RED {:?} {invariant}: {}", observation.position, why.chars().take(1500).collect::<String>());
                            }
                            reds.push(format!("{:?} {invariant}", observation.position));
                        }
                    }
                },
            );
            let ending = match &run.ending {
                HistoryEnding::Completed => "Completed".to_string(),
                other => format!("{other:?}").chars().take(900).collect(),
            };
            let hit = ending != "Completed" || !reds.is_empty();
            cases += 1;
            if hit {
                hits += 1;
            }
            eprintln!("CASE {case} hit={hit} ending={ending} reds={reds:?}");
            if verbose || hit {
                eprintln!("OUTCOMES {outcomes:?}");
            }
        }
    }
    eprintln!("SWEEP cases={cases} hits={hits}");
}

/// 回退拒：执行器只在现行那一版带文件或建过 inode 时调回退，所以先建一个 inode。删完 → 建快照（或建了再删）→ 建 inode → 回退第 k 新。
#[test]
fn rollback_after_a_snapshot_on_an_empty_file_set_sweep() {
    let mut cases = 0u64;
    let mut hits = 0u64;
    for destroy_first in [false, true] {
        for index_from_newest in 0..8_u64 {
            let mut operations = vec![HistoryOperation::Unlink, HistoryOperation::CreateSnapshot];
            if destroy_first {
                operations.push(HistoryOperation::DestroySnapshot(SnapshotChoice { index_among_the_snapshots: 0 }));
            }
            operations.push(HistoryOperation::CreateInode);
            operations.push(HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest }));
            operations.push(HistoryOperation::CloseAndMountWritable);
            let history = GeneratedHistory {
                seed: HistorySeed(0),
                starting_point: HistoryStartingPoint::AfterFirstFile,
                operations,
            };
            let mut reds = Vec::new();
            let mut rollback_outcome = String::new();
            let run = execute_history_with(
                &history,
                HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                &SharedStream::new(),
                &mut |observation| {
                    if matches!(observation.operation, Some(HistoryOperation::RollBackWhileMounted(_))) {
                        rollback_outcome = format!("{:?}", observation.outcome).chars().take(200).collect();
                    }
                    for (invariant, verdict) in check_pool_image(observation.image) {
                        if let InvariantVerdict::Violated(_) = verdict {
                            reds.push(format!("{:?} {invariant}", observation.position));
                        }
                    }
                },
            );
            let ending = match &run.ending {
                HistoryEnding::Completed => "Completed".to_string(),
                other => format!("{other:?}").chars().take(900).collect(),
            };
            let hit = ending != "Completed" || !reds.is_empty();
            cases += 1;
            if hit {
                hits += 1;
            }
            eprintln!("CASE destroy_first={destroy_first} rollback_index={index_from_newest} hit={hit} rollback={rollback_outcome} ending={ending} reds={reds:?}");
        }
    }
    eprintln!("SWEEP cases={cases} hits={hits}");
}
