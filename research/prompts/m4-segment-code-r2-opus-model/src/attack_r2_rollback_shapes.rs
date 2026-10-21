//! 攻方腿原型（m4-segment-code-r2，R1）：F1 之后的回退在正文 R1 点名的几种形状上，映射、释放集、复活集对不对得上。
//! 写死的前缀照第一轮攻方种子下标 126 那两次顺序写（152 与 290 个数据单元，铺满几段，强制选段落在装着用户数据的段上），
//! 之后的整理、回退目标（根环里第几新）、回退之前可写挂载与否、回退之后接什么，由用户决定的那几步放开扫。
//! 每一步之后的镜像过池级 checker，执行器逐步与理想模型对拍；一格打中 = 结局不是 Completed 或有一步红了一条不变量。
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

fn compaction(batch_units: u64, region_selector: u64, ending: CompactionRunEnding) -> HistoryOperation {
    HistoryOperation::CompactWhileMounted(CompactionChoice {
        batch_units,
        region_selector,
        ending,
    })
}

fn rollback(index_from_newest: u64) -> HistoryOperation {
    HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest })
}

/// 第一轮攻方种子下标 126 的两次顺序写。
fn two_sequential_writes() -> Vec<HistoryOperation> {
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
    ]
}

/// 一段写死的历史跑一遍：交回（结局摘要、每一步红的不变量名）。
fn run(starting_point: HistoryStartingPoint, operations: Vec<HistoryOperation>, verbose: bool) -> (String, Vec<String>) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point,
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
        other => format!("{other:?}").chars().take(900).collect(),
    };
    if verbose {
        eprintln!("OUTCOMES {:?}", run.outcomes.iter().map(|o| format!("{o:?}").chars().take(160).collect::<String>()).collect::<Vec<_>>());
    }
    (ending, reds)
}

/// 回退之后接的几步：覆盖写一个单元、可写挂载、顺序写 290 个、冷启动。
fn after_the_rollback() -> Vec<HistoryOperation> {
    vec![
        HistoryOperation::PublishOverwrite(spanning(1, 9)),
        HistoryOperation::CloseAndMountWritable,
        HistoryOperation::PublishOverwrite(spanning(290, 11)),
        HistoryOperation::ColdStartRecover,
    ]
}

/// 几种形状 × 回退之前挂不挂 × 回退目标第 0–11 新。形状由 ATTACK_SHAPE 选（不给跑全部），ATTACK_VERBOSE=1 配 ATTACK_ONLY 逐步打印一格。
#[test]
fn rollback_shapes_sweep() {
    let verbose = env_u64("ATTACK_VERBOSE", 0) == 1;
    let only_shape = std::env::var("ATTACK_SHAPE").ok();
    let only = std::env::var("ATTACK_ONLY").ok();
    let deepest = env_u64("ATTACK_DEEPEST_INDEX", 11);
    let selector_a = 16_033_315_046_779_662_336_u64;
    let selector_b = 7_u64;
    let shapes: Vec<(&str, Vec<HistoryOperation>, Vec<HistoryOperation>)> = vec![
        // 目标版之后有过多批、两条意图的整理（两趟整理，各推到停）。
        ("two-runs", vec![
            compaction(3, selector_a, CompactionRunEnding::RunsUntilItStops),
            compaction(2, selector_b, CompactionRunEnding::RunsUntilItStops),
        ], after_the_rollback()),
        // 整理搬过的单元又被覆盖写（顺序写 290 个，搬过的 key 在现行那一版里不在了）。
        ("compaction-then-overwrite", vec![
            compaction(3, selector_a, CompactionRunEnding::RunsUntilItStops),
            HistoryOperation::PublishOverwrite(spanning(290, 21)),
        ], after_the_rollback()),
        // 意图半路崩、可写挂载（意图在盘上活着），回退，再接着整理把盘上那一条推完。
        ("crash-mid-intent-then-resume", vec![
            compaction(1, selector_a, CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 2 }),
        ], {
            let mut after = vec![compaction(2, selector_a, CompactionRunEnding::RunsUntilItStops)];
            after.extend(after_the_rollback());
            after
        }),
        // 同上，回退之后先覆盖写、再整理（用户改动之后再续意图）。
        ("crash-mid-intent-overwrite-then-resume", vec![
            compaction(1, selector_a, CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 3 }),
        ], {
            let mut after = vec![
                HistoryOperation::PublishOverwrite(spanning(1, 31)),
                compaction(3, selector_b, CompactionRunEnding::RunsUntilItStops),
            ];
            after.extend(after_the_rollback());
            after
        }),
        // 映射树只有一片叶：起点那个小文件（一个数据单元），不做两次顺序写；整理强制选的段换几种。
        // 只有 152 个数据单元那一次顺序写：映射条目一百多条，看中央映射树是不是一片叶（TRACE 外另由 checker 判），整理一样强制选段。
        ("one-write-batch-3", vec![compaction(3, selector_a, CompactionRunEnding::RunsUntilItStops)], after_the_rollback()),
        ("one-write-crash-mid-intent", vec![compaction(1, selector_a, CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 2 })], {
            let mut after = vec![compaction(2, selector_a, CompactionRunEnding::RunsUntilItStops)];
            after.extend(after_the_rollback());
            after
        }),
        ("single-leaf-selector-0", vec![compaction(1, 0, CompactionRunEnding::RunsUntilItStops)], after_the_rollback()),
        ("single-leaf-selector-1", vec![compaction(1, 1, CompactionRunEnding::RunsUntilItStops)], after_the_rollback()),
        ("single-leaf-selector-2", vec![compaction(2, 2, CompactionRunEnding::RunsUntilItStops)], after_the_rollback()),
        ("single-leaf-selector-3", vec![compaction(3, 3, CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 })], after_the_rollback()),
    ];
    let mut cases = 0u64;
    let mut hits = 0u64;
    for (shape, middle, suffix) in &shapes {
        if only_shape.as_deref().is_some_and(|wanted| wanted != *shape) {
            continue;
        }
        for remount in [false, true] {
            for index_from_newest in 0..=deepest {
                let name = format!("shape={shape} remount={remount} rollback_index={index_from_newest}");
                if only.as_deref().is_some_and(|wanted| wanted != name) {
                    continue;
                }
                let mut operations = if shape.starts_with("single-leaf") {
                    Vec::new()
                } else if shape.starts_with("one-write") {
                    two_sequential_writes().into_iter().take(1).collect()
                } else {
                    two_sequential_writes()
                };
                operations.extend(middle.iter().copied());
                if remount {
                    operations.push(HistoryOperation::CloseAndMountWritable);
                }
                operations.push(rollback(index_from_newest));
                operations.extend(suffix.iter().copied());
                cases += 1;
                let (ending, reds) = run(HistoryStartingPoint::AfterFirstFile, operations, verbose);
                let hit = ending != "Completed" || !reds.is_empty();
                if hit {
                    hits += 1;
                }
                eprintln!("CASE {name} hit={hit} ending={ending} reds={reds:?}");
            }
        }
    }
    eprintln!("SWEEP cases={cases} hits={hits}");
}
