//! 攻方腿原型（m4-segment-code-r1）：快照、整理、回退、抬 F、崩溃恢复混在一段随机历史里，每一步之后的镜像过池级 checker。
//! 种子区间取 ATTACK_SEED_OFFSET 起（别的用例没用过的一段）；段数与步数从环境变量取。
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryEnding,
    HistoryExecution, HistoryOperationKind as K, HistorySeed, HistoryStartingPoint,
    NamedGenerationWeights, OverwriteContentDraw, RollbackTargetDraw,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::SharedStream;

include!("attack_common/weights.rs");

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

#[test]
fn attack_mix_every_step_image_passes_the_pool_checker_and_the_model_agrees() {
    let seed_start = env_u64("ATTACK_SEED_START", 0);
    let seeds = env_u64("ATTACK_SEEDS", 2);
    let ops = usize::try_from(env_u64("ATTACK_OPS", 30)).expect("步数");
    let check_every_step = env_u64("ATTACK_CHECK_EVERY_STEP", 1) == 1;
    let verbose = env_u64("ATTACK_VERBOSE", 0) == 1;
    let mut problems: Vec<String> = Vec::new();
    let mut steps_checked = 0u64;
    let mut kinds: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    for index in seed_start..seed_start + seeds {
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + ATTACK_SEED_OFFSET + index;
        let history = generate_history_with_weights(HistorySeed(seed), ops, attack_weights());
        let mut seen: Vec<String> = Vec::new();
        let run = execute_history_with(
            &history,
            HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            &SharedStream::new(),
            &mut |observation| {
                if let Some(outcome) = observation.outcome {
                    let text = format!("{outcome:?}");
                    let head: String = text.chars().take_while(|c| *c != '{' && *c != '(').collect();
                    let inner: String = text.chars().skip(head.len()).take(40).collect();
                    *kinds.entry(format!("{head}{inner}")).or_insert(0) += 1;
                }
                if verbose {
                    eprintln!(
                        "STEP {:?} op={:?} outcome={}",
                        observation.position,
                        observation.operation,
                        observation.outcome.map(|o| format!("{o:?}").chars().take(400).collect::<String>()).unwrap_or_default()
                    );
                }
                if !check_every_step {
                    return;
                }
                steps_checked += 1;
                for (name, verdict) in check_pool_image(observation.image) {
                    if let InvariantVerdict::Violated(why) = verdict {
                        if verbose {
                            eprintln!("RED {:?} {name}: {}", observation.position, why.chars().take(1500).collect::<String>());
                        }
                        seen.push(format!(
                            "种子偏移 {index} 位置 {:?} 操作 {:?}：{name} 红：{}",
                            observation.position,
                            observation.operation,
                            why.chars().take(600).collect::<String>()
                        ));
                    }
                }
            },
        );
        if run.ending != HistoryEnding::Completed {
            problems.push(format!(
                "种子偏移 {index}：结局 {:?}；操作 {:?}",
                run.ending, history.operations
            ));
        }
        if let Some(earliest_red) = seen.first() {
            problems.push(format!("{earliest_red}（这一段共 {} 处红）", seen.len()));
        }
    }
    eprintln!("ATTACK_SUMMARY seeds={seeds} seed_start={seed_start} ops={ops} steps_checked={steps_checked} problems={}", problems.len());
    for problem in &problems {
        eprintln!("ATTACK_PROBLEM {problem}");
    }
    for (kind, count) in &kinds {
        eprintln!("ATTACK_KIND {count} {kind}");
    }
    assert!(problems.is_empty(), "{} 段出问题", problems.len());
}
