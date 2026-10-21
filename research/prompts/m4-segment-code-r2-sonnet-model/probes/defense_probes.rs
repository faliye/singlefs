//! 辩方腿的手写历史探针（只读测试装置，不改被判代码）：每个场景一段历史，每一步之后的镜像过池级 checker，执行器逐步与理想模型比。
//! 环境变量：PROBE_GROUP = t4 | f1 | f5 | f6；PROBE_SHARD = i/N（场景下标取余 N 等于 i 的归这个进程）；PROBE_VERBOSE = 1 打每步结局。
//! 每个场景打一行 `PROBE group=.. name=.. ending=.. steps=.. reds=.. first_reds=..`。

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_harness::history::{
    execute_history_with, CompactionChoice, CompactionRunEnding, ContentChoice, ContentLength,
    GeneratedHistory, HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed,
    HistoryStartingPoint, RollbackTargetChoice, SnapshotChoice, StepOutcome, StepPosition,
};
use singlefs_harness::model::ObservedReadBack;
use singlefs_harness::model_comparison::model_root_key;
use singlefs_harness::reference_filesystem_translation::{
    ReferenceFileSystemCrashMoment, ReferenceFileSystemTranslation, ReferenceFileSystemVote,
};
use singlefs_harness::scenario::first_file_content;
use singlefs_harness::SharedStream;

const SELECTOR_OF_THE_ROUND_ONE_ATTACK: u64 = 16_033_315_046_779_662_336;

struct Scenario {
    name: String,
    operations: Vec<HistoryOperation>,
}

fn sequential(data_units: u64, fill_seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
            data_units,
            selector: 5,
        },
        fill_seed,
    })
}

fn one_unit(fill_seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 777 },
        fill_seed,
    })
}

fn compact(batch_units: u64, region_selector: u64, ending: CompactionRunEnding) -> HistoryOperation {
    HistoryOperation::CompactWhileMounted(CompactionChoice {
        batch_units,
        region_selector,
        ending,
    })
}

fn rollback(index_from_newest: u64) -> HistoryOperation {
    HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest })
}

fn snapshot_at(index_among_the_snapshots: u64) -> SnapshotChoice {
    SnapshotChoice {
        index_among_the_snapshots,
    }
}

fn run_scenario(group: &str, scenario: &Scenario, starting_point: HistoryStartingPoint) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point,
        operations: scenario.operations.clone(),
    };
    let mut reds: Vec<String> = Vec::new();
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            for (name, verdict) in check_pool_image(observation.image) {
                if let InvariantVerdict::Violated(why) = verdict {
                    reds.push(format!(
                        "{:?} {name}：{}",
                        observation.position,
                        why.chars().take(160).collect::<String>()
                    ));
                }
            }
        },
    );
    let ending = match &run.ending {
        HistoryEnding::Completed => "Completed".to_string(),
        other => format!("{other:?}").chars().take(500).collect(),
    };
    let rollback_outcomes: Vec<String> = run
        .outcomes
        .iter()
        .zip(&scenario.operations)
        .filter(|(_, operation)| matches!(operation, HistoryOperation::RollBackWhileMounted(_)))
        .map(|(outcome, _)| match outcome {
            StepOutcome::Applied(_) => "applied".to_string(),
            StepOutcome::Refused { .. } => "refused".to_string(),
            StepOutcome::NotApplicable(_) => "n/a".to_string(),
        })
        .collect();
    println!(
        "PROBE group={group} name={} ending={ending} steps={} rollbacks={:?} reds={} first_reds={}",
        scenario.name,
        run.outcomes.len(),
        rollback_outcomes,
        reds.len(),
        reds.iter().take(2).cloned().collect::<Vec<_>>().join(" | ")
    );
    if std::env::var("PROBE_VERBOSE").is_ok() {
        for (index, outcome) in run.outcomes.iter().enumerate() {
            println!(
                "PROBE_STEP name={} step={index} outcome={}",
                scenario.name,
                format!("{outcome:?}").chars().take(220).collect::<String>()
            );
        }
    }
}

fn shard() -> (usize, usize) {
    let text = std::env::var("PROBE_SHARD").unwrap_or_else(|_| "0/1".to_string());
    let (index, count) = text.split_once('/').expect("PROBE_SHARD=i/N");
    (index.parse().expect("i"), count.parse().expect("N"))
}

fn run_group(group: &str, scenarios: Vec<Scenario>, starting_point: HistoryStartingPoint) {
    let (index, count) = shard();
    for (position, scenario) in scenarios.iter().enumerate() {
        if position % count == index {
            run_scenario(group, scenario, starting_point);
        }
    }
}

/// T4：攻方 T4 的 5 步历史（无快照回退 → 建快照 → 快照下覆盖写），以及攻方说的第二形（1032）所缺的同类前缀变体。
fn t4_scenarios() -> Vec<Scenario> {
    let five_steps = vec![
        HistoryOperation::CloseAndMountWritable,
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit {
                selector: 7_879_751_949_923_199_194,
            },
            fill_seed: 17_840_163_520_990_143_491,
        }),
        rollback(2),
        HistoryOperation::CreateSnapshot,
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                data_units: 291,
                selector: 7_664_500_909_322_504_686,
            },
            fill_seed: 4_676_315_349_287_084_229,
        }),
    ];
    let mut longer = five_steps.clone();
    longer.extend([
        HistoryOperation::DestroySnapshot(snapshot_at(0)),
        HistoryOperation::ColdStartRecover,
    ]);
    let mut two_snapshots = five_steps.clone();
    two_snapshots.extend([
        HistoryOperation::CreateSnapshot,
        one_unit(3),
        HistoryOperation::ReadSnapshot(snapshot_at(0)),
        HistoryOperation::ReadSnapshot(snapshot_at(1)),
    ]);
    vec![
        Scenario {
            name: "t4-attack-five-steps".to_string(),
            operations: five_steps,
        },
        Scenario {
            name: "t4-five-steps-then-destroy-and-cold-start".to_string(),
            operations: longer,
        },
        Scenario {
            name: "t4-five-steps-then-second-snapshot".to_string(),
            operations: two_snapshots,
        },
    ]
}

/// F1：回退之后的映射，几种形状；每种扫回退目标在根环里的位置。
fn f1_scenarios() -> Vec<Scenario> {
    let mut scenarios = Vec::new();
    let big_prefix = || vec![sequential(152, 11), sequential(290, 12)];
    for target in 0..8_u64 {
        // 目标版之后整理两趟（两个不同的段），中间覆盖写把搬过的单元的 key 换掉。
        let mut operations = big_prefix();
        operations.extend([
            compact(3, SELECTOR_OF_THE_ROUND_ONE_ATTACK, CompactionRunEnding::RunsUntilItStops),
            sequential(290, 13),
            compact(3, 2, CompactionRunEnding::RunsUntilItStops),
            rollback(target),
            one_unit(14),
        ]);
        scenarios.push(Scenario {
            name: format!("f1-two-compactions-with-an-overwrite-between-rollback-{target}"),
            operations,
        });
        // 回退目标落在意图半路：整理推一批就崩（意图还活着），回退，再整理推到停。
        let mut operations = big_prefix();
        operations.extend([
            compact(
                3,
                SELECTOR_OF_THE_ROUND_ONE_ATTACK,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 },
            ),
            rollback(target),
            one_unit(15),
            compact(3, SELECTOR_OF_THE_ROUND_ONE_ATTACK, CompactionRunEnding::RunsUntilItStops),
        ]);
        scenarios.push(Scenario {
            name: format!("f1-rollback-with-a-live-intent-then-compact-again-rollback-{target}"),
            operations,
        });
        // 两趟整理之间可写挂载，回退之后冷启动。
        let mut operations = big_prefix();
        operations.extend([
            compact(2, 1, CompactionRunEnding::RunsUntilItStops),
            HistoryOperation::CloseAndMountWritable,
            compact(2, 3, CompactionRunEnding::RunsUntilItStops),
            rollback(target),
            HistoryOperation::ColdStartRecover,
            one_unit(16),
        ]);
        scenarios.push(Scenario {
            name: format!("f1-remount-between-compactions-rollback-{target}-cold-start"),
            operations,
        });
    }
    // 连续两次回退（第二次的目标可以是第一次回退那一版或更早）。
    for first in 0..5_u64 {
        for second in 0..5_u64 {
            let mut operations = big_prefix();
            operations.extend([
                compact(3, SELECTOR_OF_THE_ROUND_ONE_ATTACK, CompactionRunEnding::RunsUntilItStops),
                rollback(first),
                rollback(second),
                one_unit(17),
            ]);
            scenarios.push(Scenario {
                name: format!("f1-two-rollbacks-{first}-then-{second}"),
                operations,
            });
        }
    }
    scenarios
}

/// F1 的小池：文件只有一个到几十个数据单元，中央映射树只有一片叶；整理强制选不同的段，回退到根环里不同的根，再覆盖写。
fn f1_small_scenarios() -> Vec<Scenario> {
    let mut scenarios = Vec::new();
    for units in [0_u64, 20, 60] {
        for selector in 0..4_u64 {
            for target in 0..5_u64 {
                let mut operations = Vec::new();
                if units >= 2 {
                    operations.push(sequential(units, 21));
                }
                operations.extend([
                    HistoryOperation::CloseAndMountWritable,
                    compact(2, selector, CompactionRunEnding::RunsUntilItStops),
                    rollback(target),
                    one_unit(22),
                    HistoryOperation::ColdStartRecover,
                ]);
                scenarios.push(Scenario {
                    name: format!("f1small-units-{units}-selector-{selector}-rollback-{target}"),
                    operations,
                });
            }
        }
    }
    scenarios
}

/// F5：文件集合为空（unlink 删完）时建快照、读快照、重挂、冷启动、建 inode、回退、再建快照、删快照；以及抛弃最新根的恢复。
fn f5_scenarios() -> Vec<Scenario> {
    let mut scenarios = Vec::new();
    scenarios.push(Scenario {
        name: "f5-snapshot-of-the-emptied-file-set-then-read-remount-cold-start".to_string(),
        operations: vec![
            HistoryOperation::Unlink,
            HistoryOperation::CreateSnapshot,
            HistoryOperation::ReadSnapshot(snapshot_at(0)),
            HistoryOperation::ColdStartRecover,
            HistoryOperation::ReadSnapshot(snapshot_at(0)),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::ReadSnapshot(snapshot_at(0)),
            HistoryOperation::CreateInode,
            HistoryOperation::ReadSnapshot(snapshot_at(0)),
        ],
    });
    scenarios.push(Scenario {
        name: "f5-rollback-refused-with-a-snapshot-of-the-emptied-set-then-destroy".to_string(),
        operations: vec![
            HistoryOperation::Unlink,
            HistoryOperation::CreateSnapshot,
            rollback(1),
            rollback(2),
            HistoryOperation::DestroySnapshot(snapshot_at(0)),
            rollback(1),
            HistoryOperation::ColdStartRecover,
        ],
    });
    scenarios.push(Scenario {
        name: "f5-destroy-the-snapshot-of-the-emptied-set-in-a-fresh-mount".to_string(),
        operations: vec![
            HistoryOperation::Unlink,
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::DestroySnapshot(snapshot_at(0)),
            HistoryOperation::ColdStartRecover,
            HistoryOperation::CreateSnapshot,
            HistoryOperation::ReadSnapshot(snapshot_at(0)),
        ],
    });
    scenarios.push(Scenario {
        name: "f5-crash-abandoning-the-snapshot-creation-root".to_string(),
        operations: vec![
            HistoryOperation::Unlink,
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
            HistoryOperation::ReadSnapshot(snapshot_at(0)),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::ReadSnapshot(snapshot_at(0)),
            HistoryOperation::ColdStartRecover,
        ],
    });
    // 快照上限 10 数不数它：在空文件集合上建到上限再多建一个。
    let mut at_the_cap = vec![HistoryOperation::Unlink];
    for _ in 0..12 {
        at_the_cap.push(HistoryOperation::CreateSnapshot);
    }
    at_the_cap.push(HistoryOperation::ColdStartRecover);
    scenarios.push(Scenario {
        name: "f5-twelve-snapshots-of-the-emptied-set-the-cap-of-ten".to_string(),
        operations: at_the_cap,
    });
    // 对照：有文件时建快照、之后 unlink 删空，快照读出建它那一刻的内容。
    scenarios.push(Scenario {
        name: "f5-control-snapshot-with-a-file-then-unlink".to_string(),
        operations: vec![
            HistoryOperation::CreateSnapshot,
            HistoryOperation::Unlink,
            HistoryOperation::ReadSnapshot(snapshot_at(0)),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::ReadSnapshot(snapshot_at(0)),
            HistoryOperation::ReadSnapshot(snapshot_at(1)),
            HistoryOperation::ColdStartRecover,
        ],
    });
    scenarios
}

#[test]
#[ignore = "辩方腿的探针，按环境变量跑"]
fn defense_probe_group() {
    let group = std::env::var("PROBE_GROUP").expect("PROBE_GROUP");
    match group.as_str() {
        "t4" => run_group("t4", t4_scenarios(), HistoryStartingPoint::AfterFirstFile),
        "f1" => run_group("f1", f1_scenarios(), HistoryStartingPoint::AfterFirstFile),
        "f5" => run_group("f5", f5_scenarios(), HistoryStartingPoint::AfterFirstFile),
        "f1small" => run_group("f1small", f1_small_scenarios(), HistoryStartingPoint::AfterFirstFile),
        other => panic!("认不出的组 {other}"),
    }
}

/// F6：RefFS 翻译层上的快照允许集合：每一步里崩、读回的内容相同（A）或不同（B），看判票与重启之后 RefFS 允许的快照集合。
#[test]
#[ignore = "辩方腿的探针"]
fn defense_probe_reference_filesystem_snapshot_sets() {
    let overwrite = ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 200 },
        fill_seed: 7,
    };
    let histories: Vec<(&str, Vec<HistoryOperation>)> = vec![
        (
            "create-overwrite-read-destroy",
            vec![
                HistoryOperation::CreateSnapshot,
                HistoryOperation::PublishOverwrite(overwrite),
                HistoryOperation::ReadSnapshot(snapshot_at(0)),
                HistoryOperation::DestroySnapshot(snapshot_at(0)),
            ],
        ),
        (
            "two-snapshots-then-destroy-the-first",
            vec![
                HistoryOperation::CreateSnapshot,
                HistoryOperation::CreateSnapshot,
                HistoryOperation::DestroySnapshot(snapshot_at(0)),
            ],
        ),
    ];
    for (name, operations) in histories {
        let history = GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations,
        };
        let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
        let stream = SharedStream::retaining_contents();
        let mut translation =
            ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
        let run = execute_history_with(&history, execution, &stream, &mut |observation| {
            translation.note_an_observed_step(observation, stream.operation_count());
        });
        println!("PROBE group=f6 name={name} ending={:?}", run.ending);
        let translated = translation.translate(
            &stream.retained_operations(),
            &execution.device_width.fixed_geometry(),
        );
        let contents: Vec<(&str, Vec<u8>)> = vec![
            ("first-file", first_file_content()),
            ("overwrite", overwrite.bytes()),
        ];
        let mut moments: Vec<(String, ReferenceFileSystemCrashMoment)> = (0..history.operations.len())
            .map(|index| {
                (
                    format!("inside-step-{index}"),
                    ReferenceFileSystemCrashMoment::InsideStep(StepPosition::Operation(index)),
                )
            })
            .collect();
        moments.push((
            "after-every-step".to_string(),
            ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
        ));
        for (moment_name, moment) in moments {
            for (content_name, content) in &contents {
                let judgement = translated.judge_a_crash(
                    moment,
                    &ObservedReadBack::FileRead {
                        root: model_root_key(InstanceGeneration(1), CheckpointTxg(0)),
                        content: content.clone(),
                    },
                );
                let sets = judgement
                    .restarted
                    .as_ref()
                    .map(|restarted| {
                        let described: Vec<String> = restarted
                            .allowed_snapshot_sets()
                            .iter()
                            .map(|snapshots| {
                                let members: Vec<String> = snapshots
                                    .iter()
                                    .map(|(name, state)| match state {
                                        singlefs_reference_filesystem::VisibleState::NoFile => {
                                            format!("{}:NoFile", name.0)
                                        }
                                        singlefs_reference_filesystem::VisibleState::OneFile(
                                            singlefs_reference_filesystem::FileContent(bytes),
                                        ) => format!("{}:file{}B", name.0, bytes.len()),
                                    })
                                    .collect();
                                format!("{{{}}}", members.join(","))
                            })
                            .collect();
                        format!("[{}]", described.join(" ; "))
                    })
                    .unwrap_or_else(|| "-".to_string());
                let green = judgement.vote == ReferenceFileSystemVote::Green;
                println!(
                    "PROBE group=f6 name={name} moment={moment_name} readback={content_name} green={green} vote={:?} sets_after={}",
                    judgement.vote,
                    sets.chars().take(300).collect::<String>()
                );
            }
        }
    }
}

/// F3 的停机谓词边界与回退重写的算术：只调公开函数，不跑历史。
#[test]
#[ignore = "辩方腿的探针"]
fn defense_probe_arithmetic() {
    use singlefs_core::compaction_plan::the_completed_intent_yielded;
    use singlefs_core::transaction::{CodeTwoTreeNodeCapacities, MultiLevelCodeTwoTree};
    // 签发那次发布开了一个新聚簇段：签发之前按记录数得 3310，记账第 11 项写下 3309。意图完成后净产出为 0 时现在的全空段数回到 3310。
    println!(
        "PROBE group=f3 name=k-h-boundary net_zero_cycle: yielded_with_the_count_before_the_issue(3310 vs 3310)={} yielded_with_the_accounting_value(3310 vs 3309)={}",
        the_completed_intent_yielded(3310, 3310),
        the_completed_intent_yielded(3310, 3309)
    );
    let capacity = CodeTwoTreeNodeCapacities::FromTheNodeFormat.of_tree(MultiLevelCodeTwoTree::CentralMapping);
    println!(
        "PROBE group=f1 name=central-mapping-capacity leaf_entries={} internal_entries={}",
        capacity.leaf_entries, capacity.internal_entries
    );
}
