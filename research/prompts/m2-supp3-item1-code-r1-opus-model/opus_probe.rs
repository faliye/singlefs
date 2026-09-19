//! 攻方腿（Opus）m2-supp3-item1-code-r1 的探针：只放进仓副本的 `crates/singlefs-harness/tests/`，不进主工作区。
//! 按环境变量跑一批种子（与快档同一个生成器与执行器），每段历史一行：种子、收尾、在哪一步、签名；末尾一行汇总。
//! OPUS_FIRST（默认 0）、OPUS_SEEDS（默认 96）、OPUS_OPS（默认 30）、OPUS_THREADS（默认 16）。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use singlefs_harness::history::{
    execute_history, generate_history, HistoryEnding, HistoryOperation, HistorySeed, StepOutcome,
};

fn number(name: &str, default: u64) -> u64 {
    std::env::var(name).map_or(default, |text| text.parse().expect("非负整数"))
}

#[test]
#[ignore = "探针：只在副本上按环境变量跑"]
fn opus_probe_per_seed_endings() {
    let first = number("OPUS_FIRST", 0);
    let seeds = number("OPUS_SEEDS", 96);
    let operations = usize::try_from(number("OPUS_OPS", 30)).expect("步数");
    let threads = number("OPUS_THREADS", 16);
    let next = AtomicU64::new(0);
    let lines: Mutex<Vec<(u64, String)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let offset = next.fetch_add(1, Ordering::Relaxed);
                if offset >= seeds {
                    break;
                }
                let seed = first + offset;
                let history = generate_history(HistorySeed(seed), operations);
                let run = execute_history(&history);
                let rollbacks_ok = run
                    .outcomes
                    .iter()
                    .zip(&history.operations)
                    .filter(|(outcome, operation)| {
                        matches!(operation, HistoryOperation::CloseAndMountRollback(_))
                            && matches!(outcome, StepOutcome::Applied(_))
                    })
                    .count();
                let raises_ok = run
                    .outcomes
                    .iter()
                    .zip(&history.operations)
                    .filter(|(outcome, operation)| {
                        matches!(operation, HistoryOperation::RaiseRollbackFloor(_))
                            && matches!(outcome, StepOutcome::Applied(_))
                    })
                    .count();
                let text = match &run.ending {
                    HistoryEnding::Completed => "Completed".to_string(),
                    HistoryEnding::KnownRed { form, observation } => format!(
                        "KnownRed{form} at {:?} after {:?} newest_txg {:?} violations {:?}",
                        observation.position,
                        observation.operation_kind,
                        observation.newest_ring_root_txg,
                        observation.violations
                    ),
                    HistoryEnding::NewFinding {
                        signature,
                        observation,
                    } => format!(
                        "NewFinding {signature:?} at {:?} after {:?} newest_txg {:?} violations {:?} panic {:?}",
                        observation.position,
                        observation.operation_kind,
                        observation.newest_ring_root_txg,
                        observation.violations,
                        observation.panic
                    ),
                };
                lines.lock().expect("锁").push((
                    seed,
                    format!("seed {seed} rollbacks_ok {rollbacks_ok} raises_ok {raises_ok} :: {text}"),
                ));
            });
        }
    });
    let mut lines = lines.into_inner().expect("锁");
    lines.sort_by_key(|(seed, _)| *seed);
    let mut completed = 0;
    let mut known = [0u64; 2];
    let mut new_findings = 0;
    for (_, line) in &lines {
        println!("OPUS {line}");
        if line.contains(":: Completed") {
            completed += 1;
        } else if line.contains(":: KnownRed0") {
            known[0] += 1;
        } else if line.contains(":: KnownRed1") {
            known[1] += 1;
        } else {
            new_findings += 1;
        }
    }
    println!(
        "OPUS-SUMMARY first {first} seeds {seeds} ops {operations}: completed {completed} known0 {} known1 {} new {new_findings}",
        known[0], known[1]
    );
}
