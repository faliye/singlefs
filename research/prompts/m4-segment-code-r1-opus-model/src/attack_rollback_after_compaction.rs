//! 攻方腿原型（m4-segment-code-r1，T5 / T7 / T8）：整理搬过单元之后回退。写死前缀：第一个文件 → 可写挂载 → 顺序写（跨几个数据单元）
//! → 整理推到停；之后回退的目标（根环里第几条）与整理的批大小、选段放开扫一遍（只固定前缀与「回退」这一步）。
//! 每一步之后的镜像过池级 checker，记下红的不变量与模型对不对得上；回退之后再接几步覆盖写与可写挂载，看读回的文件。
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_harness::history::{
    execute_history_with, CompactionChoice, CompactionRunEnding, ContentChoice, ContentLength,
    GeneratedHistory, HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed,
    HistoryStartingPoint, RollbackTargetChoice,
};
use singlefs_harness::SharedStream;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn spanning(units: u64, fill_seed: u64) -> ContentChoice {
    ContentChoice {
        length: if units <= 1 {
            ContentLength::InsideOneDataUnit { selector: 777 }
        } else {
            ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                data_units: units,
                selector: 5,
            }
        },
        fill_seed,
    }
}

/// 一段写死的历史跑一遍：交回（结局摘要、每一步红的不变量名与开头那一句）。
fn run(operations: Vec<HistoryOperation>, verbose: bool) -> (String, Vec<String>) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    let mut reds = Vec::new();
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            if verbose {
                eprintln!(
                    "STEP {:?} op={:?} outcome={}",
                    observation.position,
                    observation.operation,
                    observation
                        .outcome
                        .map(|o| format!("{o:?}").chars().take(500).collect::<String>())
                        .unwrap_or_default()
                );
            }
            for (name, verdict) in check_pool_image(observation.image) {
                if let InvariantVerdict::Violated(why) = verdict {
                    if verbose {
                        eprintln!("RED {:?} {name}: {}", observation.position, why.chars().take(2000).collect::<String>());
                    }
                    reds.push(format!("{:?} {name}", observation.position));
                }
            }
        },
    );
    let ending = match &run.ending {
        HistoryEnding::Completed => "Completed".to_string(),
        other => format!("{other:?}").chars().take(700).collect(),
    };
    (ending, reds)
}

#[test]
fn rollback_after_a_compaction_sweep() {
    let verbose = env_u64("ATTACK_VERBOSE", 0) == 1;
    let only = std::env::var("ATTACK_ONLY").ok();
    let mut hits = 0u64;
    let mut cases = 0u64;
    for units in [1_u64, 20] {
        for batch_units in [1_u64, 3] {
            for region_selector in 0..3_u64 {
                for index_from_newest in 0..8_u64 {
                    let name = format!("units={units} batch={batch_units} region={region_selector} rollback_index={index_from_newest}");
                    if only.as_deref().is_some_and(|o| o != name) {
                        continue;
                    }
                    cases += 1;
                    let operations = vec![
                        HistoryOperation::CloseAndMountWritable,
                        HistoryOperation::PublishOverwrite(spanning(units, 3)),
                        HistoryOperation::CompactWhileMounted(CompactionChoice {
                            batch_units,
                            region_selector,
                            ending: CompactionRunEnding::RunsUntilItStops,
                        }),
                        HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                            index_from_newest,
                        }),
                        HistoryOperation::PublishOverwrite(spanning(1, 9)),
                        HistoryOperation::CloseAndMountWritable,
                        HistoryOperation::PublishOverwrite(spanning(units, 11)),
                    ];
                    let (ending, reds) = run(operations, verbose);
                    let hit = ending != "Completed" || !reds.is_empty();
                    if hit {
                        hits += 1;
                    }
                    eprintln!("CASE {name} hit={hit} ending={ending} reds={reds:?}");
                }
            }
        }
    }
    eprintln!("SWEEP cases={cases} hits={hits}");
}

/// 种子 126（攻方混合比重）缩出来的前缀：第一个文件 → 顺序写 152 单元 → 顺序写 290 单元 → 整理（一批 3 个、推两批后崩、可写挂载）
/// → 回退到根环里第 3 新的根。之后接的那几步由 ATTACK_SUFFIX 选（用户决定的动作放开扫）。
fn prefix_of_seed_126() -> Vec<HistoryOperation> {
    vec![
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                data_units: 152,
                selector: 9_852_620_462_860_606_767,
            },
            fill_seed: 2_832_512_656_265_544_978,
        }),
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                data_units: 290,
                selector: 15_334_540_613_569_234_429,
            },
            fill_seed: 10_383_977_899_441_184_023,
        }),
        HistoryOperation::CompactWhileMounted(CompactionChoice {
            batch_units: 3,
            region_selector: 16_033_315_046_779_662_336,
            ending: CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 2 },
        }),
        HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
            index_from_newest: 3,
        }),
    ]
}

#[test]
fn seed_126_prefix_then_suffixes() {
    use singlefs_harness::history::FloorTargetChoice;
    let verbose = env_u64("ATTACK_VERBOSE", 0) == 1;
    let only = env_u64("ATTACK_SUFFIX", 99);
    let suffixes: Vec<(&str, Vec<HistoryOperation>)> = vec![
        ("nothing", vec![]),
        ("overwrite-one-unit", vec![HistoryOperation::PublishOverwrite(spanning(1, 9))]),
        ("overwrite-290", vec![HistoryOperation::PublishOverwrite(spanning(290, 9))]),
        ("mount-then-overwrite", vec![HistoryOperation::CloseAndMountWritable, HistoryOperation::PublishOverwrite(spanning(1, 9))]),
        ("unlink", vec![HistoryOperation::Unlink]),
        ("raise-floor-then-rollback", vec![
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: 4_491_436_106_160_728_736 }),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 1 }),
        ]),
        ("compact-again", vec![HistoryOperation::CompactWhileMounted(CompactionChoice {
            batch_units: 3,
            region_selector: 16_033_315_046_779_662_336,
            ending: CompactionRunEnding::RunsUntilItStops,
        })]),
        ("cold-start", vec![HistoryOperation::ColdStartRecover]),
        ("raise-floor-then-compact", vec![
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: u64::MAX }),
            HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 3,
                region_selector: 7,
                ending: CompactionRunEnding::RunsUntilItStops,
            }),
            HistoryOperation::ColdStartRecover,
        ]),
        ("mount-raise-floor-compact-twice", vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: u64::MAX }),
            HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 3,
                region_selector: 1,
                ending: CompactionRunEnding::RunsUntilItStops,
            }),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: u64::MAX }),
            HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 3,
                region_selector: 2,
                ending: CompactionRunEnding::RunsUntilItStops,
            }),
            HistoryOperation::ColdStartRecover,
        ]),
    ];
    for (position, (name, suffix)) in suffixes.into_iter().enumerate() {
        if only != 99 && only != u64::try_from(position).expect("小") {
            continue;
        }
        let mut operations = prefix_of_seed_126();
        operations.extend(suffix);
        let (ending, reds) = run(operations, verbose);
        eprintln!("SUFFIX {name} ending={ending} reds={reds:?}");
    }
}

/// 同种子 126 的前缀，整理那一步换成「推到停、不崩」，之后可写挂载（或不挂载）一次再回退；回退目标在根环里第 0–7 新的根里扫。
#[test]
fn compaction_then_remount_then_rollback_sweep() {
    let verbose = env_u64("ATTACK_VERBOSE", 0) == 1;
    let only_remount = env_u64("ATTACK_REMOUNT", 9);
    let only_index = env_u64("ATTACK_ROLLBACK_INDEX", 99);
    for remount in [false, true] {
        if only_remount != 9 && u64::from(remount) != only_remount {
            continue;
        }
        for index_from_newest in 0..8_u64 {
            if only_index != 99 && index_from_newest != only_index {
                continue;
            }
            let mut operations = prefix_of_seed_126();
            operations.truncate(2);
            operations.push(HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 3,
                region_selector: 16_033_315_046_779_662_336,
                ending: CompactionRunEnding::RunsUntilItStops,
            }));
            if remount {
                operations.push(HistoryOperation::CloseAndMountWritable);
            }
            operations.push(HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest,
            }));
            operations.push(HistoryOperation::PublishOverwrite(spanning(1, 9)));
            let (ending, reds) = run(operations, verbose);
            eprintln!(
                "REMOUNT remount={remount} rollback_index={index_from_newest} ending={} reds={reds:?}",
                ending.chars().take(600).collect::<String>()
            );
        }
    }
}
