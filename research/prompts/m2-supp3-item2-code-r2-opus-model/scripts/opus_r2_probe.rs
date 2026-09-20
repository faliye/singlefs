//! Opus 攻方腿 m2-supp3-item2-code-r2 的探针（只在仓副本里）：按环境变量取比重、种子、步数，执行器照 `PerStepChecker::Skipped` 跑
//! （与门禁第四段同一个执行器），另外在观察者里每一步对镜像跑池级 checker，只记「不是已知红第 0 条那一形（只有 I-3.1 且已分配 > 遍历）」的违例。
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_harness::history::{
    allocated_and_walked_bytes, execute_history_with, generate_history_with_weights,
    GenerationWeights, HistoryEnding, HistoryOperationKind as K, HistorySeed, HistoryStartingPoint,
    PerStepChecker, RollbackTargetDraw, StepOutcome,
};
use singlefs_harness::SharedStream;

/// 放开用户动作：墙取样点的比重，但回退与抬 F 多（让回退、抬 F 落在逼近 812 条的时候）。
const WALL_ROLLBACK_HEAVY: GenerationWeights = GenerationWeights {
    name: "opus-r2：逼近墙、回退与抬 F 多",
    starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
    with_session_closed: &[(K::CloseAndMountWritable, 60), (K::CloseAndMountRollback, 40)],
    with_session_open_without_file: &[(K::PublishFirstFile, 90), (K::CloseAndMountWritable, 10)],
    with_session_open_with_file: &[
        (K::PublishOverwrite, 70),
        (K::CloseAndMountWritable, 8),
        (K::RaiseRollbackFloor, 10),
        (K::CloseAndMountRollback, 10),
        (K::ColdStartRecover, 2),
    ],
    rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
};

/// 放开用户动作：只覆盖写与可写挂载（最快逼近墙），不抬 F、不回退。
const WALL_PLAIN: GenerationWeights = GenerationWeights {
    name: "opus-r2：逼近墙、只覆盖写与可写挂载",
    starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
    with_session_closed: &[(K::CloseAndMountWritable, 100)],
    with_session_open_without_file: &[(K::PublishFirstFile, 100)],
    with_session_open_with_file: &[(K::PublishOverwrite, 85), (K::CloseAndMountWritable, 15)],
    rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
};

fn weights() -> &'static GenerationWeights {
    match std::env::var("PROBE_WEIGHTS").as_deref().unwrap_or("wall") {
        "broad" => &GenerationWeights::BROAD,
        "reuse" => &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        "rollback" => &GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        "wall" => &GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
        "wall_rb" => &WALL_ROLLBACK_HEAVY,
        "wall_plain" => &WALL_PLAIN,
        other => panic!("PROBE_WEIGHTS {other}"),
    }
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().map_or(default, |v| v.parse().expect("数"))
}

fn only_known_red_zero_shape(violations: &[(&'static str, String)]) -> bool {
    violations.iter().all(|(invariant, detail)| {
        *invariant == "I-3.1"
            && allocated_and_walked_bytes(detail).is_some_and(|(allocated, walked)| allocated > walked)
    })
}

#[test]
#[ignore]
fn opus_r2_probe() {
    let weights = weights();
    let first = env_u64("PROBE_FIRST", 0);
    let seeds = env_u64("PROBE_SEEDS", 32);
    let steps = usize::try_from(env_u64("PROBE_STEPS", 150)).expect("步");
    let observe = std::env::var("PROBE_CHECKER").as_deref() == Ok("observe");
    let threads = env_u64("PROBE_THREADS", 16);
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
                let history = generate_history_with_weights(HistorySeed(seed), steps, weights);
                let mut first_checker_red: Option<String> = None;
                let mut checker_runs = 0u64;
                let mut wall_refusals = 0u64;
                let mut placement_refusals = 0u64;
                let run = execute_history_with(
                    &history,
                    PerStepChecker::Skipped,
                    &SharedStream::new(),
                    &mut |observation| {
                        if let Some(StepOutcome::Refused { member, .. }) = observation.outcome {
                            if member.contains("AllocationRecordsExceedOneNode") {
                                wall_refusals += 1;
                            }
                            if member.contains("PlacementRefused") {
                                placement_refusals += 1;
                            }
                        }
                        if !observe || first_checker_red.is_some() {
                            return;
                        }
                        checker_runs += 1;
                        let violations: Vec<(&'static str, String)> = check_pool_image(observation.image)
                            .into_iter()
                            .filter_map(|(invariant, verdict)| match verdict {
                                InvariantVerdict::Violated(detail) => Some((invariant, detail)),
                                _ => None,
                            })
                            .collect();
                        if !violations.is_empty() && !only_known_red_zero_shape(&violations) {
                            let names: Vec<String> = violations
                                .iter()
                                .map(|(i, d)| format!("{i}: {}", d.chars().take(160).collect::<String>()))
                                .collect();
                            first_checker_red = Some(format!(
                                "{:?} after {:?}: {}",
                                observation.position,
                                observation.operation.map(|o| o.kind()),
                                names.join(" | ")
                            ));
                        }
                    },
                );
                let ending = match &run.ending {
                    HistoryEnding::Completed => "completed".to_string(),
                    HistoryEnding::KnownRed { form, observation } => {
                        format!("known-red {form} at {:?}", observation.position)
                    }
                    HistoryEnding::NewFinding { signature, observation } => format!(
                        "NEW {signature:?} at {:?} after {:?}; model={:?}; harness={:?}; panic={:?}",
                        observation.position,
                        observation.operation_kind,
                        observation.model_disagreement,
                        observation.harness_judgement,
                        observation.panic.as_ref().map(|p| p.location.clone())
                    ),
                };
                let line = format!(
                    "seed {seed}: ending={ending}; wall_refusals_seen_by_observer={wall_refusals}; placement_refusals_seen_by_observer={placement_refusals}; checker_runs={checker_runs}; probe_checker_first_non_known_red={}",
                    first_checker_red.as_deref().unwrap_or("none")
                );
                lines.lock().expect("锁").push((seed, line));
            });
        }
    });
    let mut lines = lines.into_inner().expect("锁");
    lines.sort_by_key(|(seed, _)| *seed);
    let new_findings = lines.iter().filter(|(_, l)| l.contains("ending=NEW")).count();
    let probe_red = lines.iter().filter(|(_, l)| !l.ends_with("probe_checker_first_non_known_red=none")).count();
    for (_, line) in &lines {
        println!("{line}");
    }
    println!(
        "PROBE SUMMARY weights={} first={first} seeds={seeds} steps={steps} observe={observe}: new_findings={new_findings} probe_checker_red={probe_red}",
        weights.name
    );
}
