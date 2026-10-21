//! 攻方腿原型（m4-segment-code-r2）：把随机历史原型打中的一段缩短（逐块删操作，删了仍打中就留下删后的）。
//! 打中 = 结局的 Debug 文字里含 ATTACK_ENDING_CONTAINS（默认「对意图做的事」），或某一步镜像上池级 checker 红了 ATTACK_INVARIANTS 里的一条。
//! ATTACK_SEED_INDEX 选种子（与 attack_r2_random_histories 同一个区间与比重），ATTACK_KEEP 给起点（留哪几步），
//! 挂钟超过 ATTACK_BUDGET_SECONDS 就停下打出当前的 KEEP，下一次接着缩。ATTACK_VERBOSE=1 只跑 KEEP 那一段并逐步打印。
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GeneratedHistory, GenerationWeights,
    HistoryEnding, HistoryExecution, HistoryOperationKind as K, HistorySeed,
    HistoryStartingPoint, NamedGenerationWeights, OverwriteContentDraw, RollbackTargetDraw,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::SharedStream;

include!("attack_common/weights_r2.rs");

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// ATTACK_DEVICE_WIDTH：384 / 256 / 240 取小盘那三档，别的取两块 4 GiB 的盘（同随机历史原型）。
fn execution() -> HistoryExecution {
    use singlefs_harness::history::HistoryDeviceWidth;
    let device_width = match std::env::var("ATTACK_DEVICE_WIDTH").as_deref() {
        Ok("384") => HistoryDeviceWidth::UnitAreaOf384Slots,
        Ok("256") => HistoryDeviceWidth::UnitAreaOf256Slots,
        Ok("240") => HistoryDeviceWidth::UnitAreaOf240Slots,
        _ => HistoryDeviceWidth::FourGibibytes,
    };
    HistoryExecution {
        device_width,
        ..HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES
    }
}

fn hit(history: &GeneratedHistory, invariants: &[String], ending_contains: &str, verbose: bool) -> bool {
    let mut red = false;
    let run = execute_history_with(
        history,
        execution(),
        &SharedStream::new(),
        &mut |observation| {
            if verbose {
                eprintln!(
                    "STEP {:?} op={:?} outcome={}",
                    observation.position,
                    observation.operation,
                    observation
                        .outcome
                        .map(|o| format!("{o:?}").chars().take(700).collect::<String>())
                        .unwrap_or_default()
                );
            }
            if invariants.is_empty() || (red && !verbose) {
                return;
            }
            for (name, verdict) in check_pool_image(observation.image) {
                if let InvariantVerdict::Violated(why) = verdict {
                    if invariants.iter().any(|wanted| wanted == name) {
                        red = true;
                    }
                    if verbose {
                        eprintln!("RED {:?} {name}: {}", observation.position, why.chars().take(2500).collect::<String>());
                    }
                }
            }
        },
    );
    let ending = format!("{:?}", run.ending);
    if verbose {
        eprintln!("ENDING {}", ending.chars().take(4000).collect::<String>());
    }
    red || (run.ending != HistoryEnding::Completed && ending.contains(ending_contains))
}

#[test]
fn minimize_a_hit() {
    let index = env_u64("ATTACK_SEED_INDEX", 297);
    let ops = usize::try_from(env_u64("ATTACK_OPS", 40)).expect("步数");
    let budget = env_u64("ATTACK_BUDGET_SECONDS", 200);
    let verbose = env_u64("ATTACK_VERBOSE", 0) == 1;
    let ending_contains = std::env::var("ATTACK_ENDING_CONTAINS").unwrap_or_else(|_| "对意图做的事".to_string());
    let invariants: Vec<String> = std::env::var("ATTACK_INVARIANTS")
        .unwrap_or_default()
        .split(',')
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect();
    let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + ATTACK_SEED_OFFSET + index;
    let full = generate_history_with_weights(HistorySeed(seed), ops, attack_weights());
    let mut keep: Vec<usize> = match std::env::var("ATTACK_KEEP") {
        Ok(text) if !text.is_empty() => text.split(',').map(|v| v.parse().expect("下标")).collect(),
        _ => (0..full.operations.len()).collect(),
    };
    let history_of = |keep: &[usize]| GeneratedHistory {
        seed: full.seed,
        starting_point: full.starting_point,
        operations: keep.iter().map(|i| full.operations[*i]).collect(),
    };
    eprintln!("START starting_point={:?}", full.starting_point);
    if verbose {
        let h = history_of(&keep);
        eprintln!("OPS {:?}", h.operations);
        let result = hit(&h, &invariants, &ending_contains, true);
        eprintln!("HIT {result}");
        return;
    }
    assert!(hit(&history_of(&keep), &invariants, &ending_contains, false), "起点就没打中");
    let started = Instant::now();
    let mut chunk = (keep.len() / 2).max(1);
    'outer: loop {
        let mut position = 0;
        let mut removed_any = false;
        while position < keep.len() {
            if started.elapsed().as_secs() > budget {
                eprintln!("BUDGET_SPENT");
                break 'outer;
            }
            let end = (position + chunk).min(keep.len());
            let candidate: Vec<usize> = keep[..position].iter().chain(keep[end..].iter()).copied().collect();
            if !candidate.is_empty() && hit(&history_of(&candidate), &invariants, &ending_contains, false) {
                keep = candidate;
                removed_any = true;
                eprintln!("KEEP {}", keep.iter().map(ToString::to_string).collect::<Vec<_>>().join(","));
            } else {
                position = end;
            }
        }
        if !removed_any {
            if chunk == 1 {
                eprintln!("MINIMAL");
                break;
            }
            chunk = (chunk / 2).max(1);
        }
    }
    eprintln!("FINAL_KEEP {}", keep.iter().map(ToString::to_string).collect::<Vec<_>>().join(","));
    eprintln!("FINAL_OPS {:?}", history_of(&keep).operations);
}
