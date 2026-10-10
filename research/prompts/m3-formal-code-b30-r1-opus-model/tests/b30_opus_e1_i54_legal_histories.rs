//! b30 攻方腿 M2 探针：合法历史（随机生成的几组比重、每步之后）上池级 checker 的 I-2.4 与 I-5.4 报什么。
//! 找两样：I-2.4 违例（E1 在合法历史上误红）；I-5.4 报部分没判而原因是「别的版本的树走到了叶不替它作保」（按版本那一层在合法历史上多出来的）。
//! 环境变量：B30_SEEDS（每组几个种子，默认 8）、B30_OPS（每段几步，默认 30）、B30_SEED_BASE（种子起点，默认 0）、B30_WEIGHTS（逗号分隔的组名，默认全部四组）。

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, AppliedEffect, GenerationWeights,
    HistoryExecution, HistoryOperationKind, HistorySeed, PerStepChecker, StepOutcome,
};
use singlefs_harness::SharedStream;
use std::collections::BTreeMap;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn e1_and_per_version_i54_on_legal_histories(chunk: u64) {
    let seeds = env_u64("B30_SEEDS", 8);
    let operations = usize::try_from(env_u64("B30_OPS", 30)).expect("usize");
    let seed_base = env_u64("B30_SEED_BASE", 0) + chunk * seeds;
    let wanted = std::env::var("B30_WEIGHTS").unwrap_or_default();
    let groups: [(&str, &GenerationWeights); 4] = [
        ("broad", &GenerationWeights::BROAD),
        ("delete", &GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES),
        ("rollback_floor", &GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR),
        ("reuse_floor", &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR),
    ];
    let mut total_images = 0u64;
    let mut i24_violated = 0u64;
    let mut i54_counts: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut i54_new_partial = 0u64;
    let mut i54_other_partial = 0u64;
    for (name, weights) in groups {
        if !wanted.is_empty() && !wanted.split(',').any(|w| w == name) {
            continue;
        }
        let mut images = 0u64;
        let mut applied: BTreeMap<String, u64> = BTreeMap::new();
        let mut group_i24 = 0u64;
        let mut group_new_partial = 0u64;
        let mut group_i54: BTreeMap<&'static str, u64> = BTreeMap::new();
        for offset in 0..seeds {
            let seed = HistorySeed(seed_base + offset);
            let history = generate_history_with_weights(seed, operations, weights);
            let run = execute_history_with(
                &history,
                HistoryExecution {
                    per_step_checker: PerStepChecker::Skipped,
                    ..HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES
                },
                &SharedStream::new(),
                &mut |observation| {
                    images += 1;
                    let verdicts = check_pool_image(observation.image);
                    for (invariant, verdict) in &verdicts {
                        if *invariant == "I-2.4" {
                            if let InvariantVerdict::Violated(text) = verdict {
                                group_i24 += 1;
                                println!("B30E1 I-2.4 violated group={name} seed={} position={:?} op={:?} text={}", seed.0, observation.position, observation.operation.map(|o| o.kind()), text.chars().take(200).collect::<String>());
                            }
                        }
                        if *invariant == "I-5.4" {
                            let label = match verdict {
                                InvariantVerdict::Holds => "Holds",
                                InvariantVerdict::Violated(_) => "Violated",
                                InvariantVerdict::NotApplicable(_) => "NotApplicable",
                                InvariantVerdict::PartiallyJudged(text) => {
                                    if text.contains("别的版本的树走到了叶不替它作保") {
                                        group_new_partial += 1;
                                        println!("B30I54 new-partial group={name} seed={} position={:?} op={:?} text={}", seed.0, observation.position, observation.operation.map(|o| o.kind()), text.chars().take(200).collect::<String>());
                                    } else {
                                        println!("B30I54 other-partial group={name} seed={} position={:?} text={}", seed.0, observation.position, text.chars().take(200).collect::<String>());
                                    }
                                    "PartiallyJudged"
                                }
                            };
                            *group_i54.entry(label).or_default() += 1;
                            if label == "Violated" {
                                println!("B30I54 violated group={name} seed={} position={:?} verdict={verdict:?}", seed.0, observation.position);
                            }
                        }
                    }
                    if let (Some(operation), Some(StepOutcome::Applied(effect))) = (observation.operation, observation.outcome) {
                        let tag = match effect {
                            AppliedEffect::RolledBack { .. } => "rolled_back",
                            AppliedEffect::Published { .. } => "published",
                            _ => "other_applied",
                        };
                        *applied.entry(format!("{:?}/{tag}", operation.kind())).or_default() += 1;
                    }
                },
            );
            let _ = run;
        }
        let unlinks = applied.iter().filter(|(k, _)| k.starts_with(&format!("{:?}", HistoryOperationKind::Unlink))).map(|(_, v)| v).sum::<u64>();
        let rollbacks = applied.iter().filter(|(k, _)| k.ends_with("rolled_back")).map(|(_, v)| v).sum::<u64>();
        let mounts = applied.iter().filter(|(k, _)| k.starts_with(&format!("{:?}", HistoryOperationKind::CloseAndMountWritable))).map(|(_, v)| v).sum::<u64>();
        println!("B30GROUP {name} seeds={seeds} ops={operations} seed_base={seed_base} images={images} i24_violated={group_i24} i54={group_i54:?} i54_new_partial={group_new_partial} unlink_applied={unlinks} rollback_applied={rollbacks} mount_applied={mounts}");
        total_images += images;
        i24_violated += group_i24;
        i54_new_partial += group_new_partial;
        for (k, v) in group_i54 {
            if k == "PartiallyJudged" { i54_other_partial += v; }
            *i54_counts.entry(k).or_default() += v;
        }
    }
    i54_other_partial -= i54_new_partial.min(i54_other_partial);
    println!("B30TOTAL images={total_images} i24_violated={i24_violated} i54={i54_counts:?} i54_new_partial={i54_new_partial} i54_other_partial={i54_other_partial}");
}

// 每个用例跑一段种子（第 chunk 段：种子 [B30_SEED_BASE + chunk × B30_SEEDS, 再加 B30_SEEDS)），libtest 按线程上限并行跑这几段。
macro_rules! chunks {
    ($($name:ident: $chunk:expr;)*) => { $(#[test] fn $name() { e1_and_per_version_i54_on_legal_histories($chunk); })* };
}
chunks! {
    chunk_0: 0; chunk_1: 1; chunk_2: 2; chunk_3: 3; chunk_4: 4; chunk_5: 5; chunk_6: 6;
}
