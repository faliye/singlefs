//! 攻方腿（Opus）m2-supp3-item1-code-r2 · Y1 / Y2 的探针：只放进仓副本的 `crates/singlefs-harness/tests/`，主工作区不放。
//! 对每段历史：
//!   - 用 checker 的读法（`singlefs_checker::image`，与 `singlefs-core` 不共用代码）记每一步之后的根环；
//!   - 每次成功的回退，按执行器同一条选法算出目标 (实例, T)，空档 = (T, 回退之前环里最新根的 txg + 1)，只从历史与 checker 读法得出，
//!     不经 `singlefs-core` 的实例表解析（与 `raised_floor_lands_only_on_abandoned_roots` 独立）；
//!   - 以已知红第 1 条收尾的，报新 F 在不在任何一个空档里（`GAP ... in_gap=`）；
//!   - 以新发现收尾而发生在抬 F 那一步、只有 I-3.1 多算的，也报在不在空档里（`NEWRAISE ...`）；
//!   - 执行器判出（HarnessJudgement）的逐条报（`HJ ...`）。
//! 环境变量：OPUS_FIRST、OPUS_SEEDS、OPUS_OPS、OPUS_THREADS、OPUS_WEIGHTS（broad / reuse）。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use singlefs_checker::image::{chosen_superblocks, valid_roots};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::history::{
    allocated_and_walked_bytes, execute_history_observing, generate_history_with_weights,
    AppliedEffect, GenerationWeights, HistoryEnding, HistoryOperation, HistorySeed,
    RollbackTargetChoice, StepOutcome, StepPosition,
};
use singlefs_harness::SharedStream;

fn number(name: &str, default: u64) -> u64 {
    std::env::var(name).map_or(default, |text| text.parse().expect("非负整数"))
}

/// checker 读法下的根环：(txg, 实例)，新的在前，去重。
fn ring(image: &MemoryPool) -> Vec<(u64, u32)> {
    let Some(geometry) = chosen_superblocks(image)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
    else {
        return Vec::new();
    };
    let mut roots: Vec<(u64, u32)> = valid_roots(image, &geometry)
        .into_iter()
        .map(|(_, _, view)| (view.checkpoint_txg, view.instance))
        .collect();
    roots.sort_unstable_by(|left, right| right.cmp(left));
    roots.dedup();
    roots
}

#[test]
#[ignore = "攻方探针，副本上手动跑"]
fn opus_r2_gap_probe() {
    let first = number("OPUS_FIRST", 0);
    let seeds = number("OPUS_SEEDS", 96);
    let operations = usize::try_from(number("OPUS_OPS", 30)).expect("步数");
    let threads = number("OPUS_THREADS", 16);
    let weights = match std::env::var("OPUS_WEIGHTS").as_deref() {
        Ok("reuse") => GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        _ => GenerationWeights::BROAD,
    };
    let next = AtomicU64::new(0);
    let lines: Mutex<Vec<(u64, String)>> = Mutex::new(Vec::new());
    let counts: Mutex<std::collections::BTreeMap<String, u64>> = Mutex::new(Default::default());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let offset = next.fetch_add(1, Ordering::Relaxed);
                if offset >= seeds {
                    break;
                }
                let seed = first + offset;
                let history = generate_history_with_weights(HistorySeed(seed), operations, &weights);
                // rings[k] = 第 k 步之前（起点之后算第 0 份）的根环。
                let mut rings: Vec<Vec<(u64, u32)>> = Vec::new();
                let run = execute_history_observing(&history, &SharedStream::new(), &mut |observation| {
                    rings.push(ring(observation.image));
                });
                // 回退的空档
                let mut gaps: Vec<(u64, u64, u32, usize)> = Vec::new();
                for (step, operation) in history.operations.iter().enumerate() {
                    let HistoryOperation::CloseAndMountRollback(choice) = operation else { continue };
                    let Some(StepOutcome::Applied(AppliedEffect::Mounted { .. })) = run.outcomes.get(step) else { continue };
                    let Some(before) = rings.get(step) else { continue };
                    let Some(&(newest, _)) = before.first() else { continue };
                    let (target_txg, target_instance) = match *choice {
                        RollbackTargetChoice::RingRoot { index_from_newest } => {
                            before[usize::try_from(index_from_newest % u64::try_from(before.len()).unwrap()).unwrap()]
                        }
                        RollbackTargetChoice::BeyondNewestRoot { .. } => continue,
                    };
                    gaps.push((target_txg, newest + 1, target_instance, step));
                }
                let gap_text = gaps
                    .iter()
                    .map(|(t, n, i, s)| format!("step{s}:({i},{t})->({t},{n})"))
                    .collect::<Vec<_>>()
                    .join(",");
                let in_gap = |floor: u64| gaps.iter().any(|(t, n, _, _)| *t < floor && floor < *n);
                let key;
                let mut line = None;
                match &run.ending {
                    HistoryEnding::Completed => key = "completed".to_string(),
                    HistoryEnding::KnownRed { form, observation } => {
                        key = format!("known_red_{form}");
                        if *form == 1 {
                            let StepPosition::Operation(step) = observation.position else { unreachable!() };
                            let floor = match &run.outcomes[step] {
                                StepOutcome::Applied(AppliedEffect::RaisedFloor { new_floor, .. }) => new_floor.0,
                                other => panic!("{other:?}"),
                            };
                            let detail = observation.violations.first().map(|(_, d)| d.clone()).unwrap_or_default();
                            let diff = allocated_and_walked_bytes(&detail).map(|(a, w)| a - w);
                            line = Some(format!(
                                "GAP seed={seed} step={step} F={floor} in_gap={} gaps=[{gap_text}] ring_before={:?} overcount_bytes={diff:?}",
                                in_gap(floor),
                                rings.get(step).map(|r| r.iter().take(16).collect::<Vec<_>>())
                            ));
                            *counts.lock().unwrap().entry(format!("form1_in_gap={}", in_gap(floor))).or_insert(0) += 1;
                        }
                    }
                    HistoryEnding::NewFinding { signature, observation } => {
                        key = format!("new:{signature:?}");
                        if let Some(judgement) = &observation.harness_judgement {
                            line = Some(format!("HJ seed={seed} pos={:?} {judgement:?}", observation.position));
                        } else if let StepPosition::Operation(step) = observation.position {
                            if let StepOutcome::Applied(AppliedEffect::RaisedFloor { new_floor, .. }) = &run.outcomes[step] {
                                line = Some(format!(
                                    "NEWRAISE seed={seed} step={step} F={} in_gap={} gaps=[{gap_text}] lands={:?} turned={} violations={:?}",
                                    new_floor.0,
                                    in_gap(new_floor.0),
                                    observation.raised_floor_lands_only_on_abandoned_roots,
                                    observation.root_ring_has_turned(),
                                    observation.violations
                                ));
                            } else {
                                line = Some(format!("NEW seed={seed} pos={:?} op={:?} violations={:?} panic={:?}", observation.position, observation.operation_kind, observation.violations, observation.panic));
                            }
                        }
                    }
                }
                *counts.lock().unwrap().entry(key).or_insert(0) += 1;
                if let Some(line) = line {
                    lines.lock().unwrap().push((seed, line));
                }
            });
        }
    });
    let mut lines = lines.into_inner().unwrap();
    lines.sort();
    for (_, line) in &lines {
        println!("{line}");
    }
    println!("SUMMARY first={first} seeds={seeds} ops={operations} weights={} {:?}", weights.name, counts.into_inner().unwrap());
}
