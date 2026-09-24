//! c355-c363-r3 攻方腿（W2）副本探针：只在草稿目录的仓副本里跑，不进入库装置。
//! P1：小盘（单元区 384 槽）上连发覆盖写撞墙，再抬 F；看抬 F 被拒时有没有已经发出写（「报错在任何写之前」只管第一次空发布）。
//! P2：对 P1 里被拒的那几段，把被拒之后由用户决定的一到两步放开扫（再加一次覆盖写收尾），看有没有哪条后缀让覆盖写重新做成。

use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, FloorTargetChoice, GeneratedHistory,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed,
    HistoryStartingPoint, PerStepChecker, RollbackTargetChoice, StepOutcome, StepPosition,
};
use singlefs_harness::SharedStream;

fn overwrite(seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 1999 },
        fill_seed: seed,
    })
}

fn raise(steps: u64) -> HistoryOperation {
    HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
        steps_above_current_floor: steps,
    })
}

struct Step {
    outcome: String,
    writes: usize,
}

struct Run {
    steps: Vec<Step>,
    ending: String,
}

fn run(operations: Vec<HistoryOperation>) -> Run {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    let stream = SharedStream::new();
    let mut last_count = 0usize;
    let mut steps: Vec<Step> = Vec::new();
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::RunContinuingPastTheRingTurnForm,
        device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
    };
    let result = execute_history_with(&history, execution, &stream, &mut |observation| {
        let now = stream.operation_count();
        let writes = now - last_count;
        last_count = now;
        if let StepPosition::Operation(_) = observation.position {
            let outcome = match observation.outcome {
                Some(StepOutcome::Applied(effect)) => format!("Applied({effect:?})"),
                Some(StepOutcome::Refused { member }) => format!("Refused({member})"),
                Some(StepOutcome::NotApplicable(missing)) => format!("NotApplicable({missing:?})"),
                None => "None".to_string(),
            };
            steps.push(Step { outcome, writes });
        }
    });
    let ending = match &result.ending {
        HistoryEnding::Completed => "Completed".to_string(),
        HistoryEnding::KnownRed { form, .. } => format!("KnownRed({form})"),
        HistoryEnding::NewFinding {
            signature,
            observation,
        } => {
            let violations: Vec<String> = observation
                .violations
                .iter()
                .map(|(invariant, detail)| format!("{invariant}: {detail}"))
                .collect();
            format!(
                "NewFinding({signature:?}) at {:?} {:?} newest_txg={:?} violations=[{}]",
                observation.position,
                observation.operation_kind,
                observation.newest_ring_root_txg,
                violations.join(" | ")
            )
        }
    };
    Run { steps, ending }
}

fn short(outcome: &str) -> String {
    if let Some(rest) = outcome.strip_prefix("Applied(RaisedFloor") {
        let publishes = rest
            .split("publishes: ")
            .nth(1)
            .and_then(|tail| tail.split(',').next())
            .unwrap_or("?");
        let floor = rest
            .split("new_floor: CheckpointTxg(")
            .nth(1)
            .and_then(|tail| tail.split(')').next())
            .unwrap_or("?");
        return format!("RaisedFloor(F={floor},publishes={publishes})");
    }
    if outcome.starts_with("Applied(Published") {
        return "Published".to_string();
    }
    if let Some(rest) = outcome.strip_prefix("Applied(Mounted") {
        let publishes = rest
            .split("publishes: ")
            .nth(1)
            .and_then(|tail| tail.split(',').next())
            .unwrap_or("?");
        return format!("Mounted(publishes={publishes})");
    }
    if outcome.starts_with("Applied(Recovered") {
        return "Recovered".to_string();
    }
    outcome.to_string()
}

fn prefix(mounts: usize, noop_raises: usize) -> Vec<HistoryOperation> {
    let mut operations = Vec::new();
    for _ in 0..mounts {
        operations.push(HistoryOperation::CloseAndMountWritable);
    }
    for _ in 0..noop_raises {
        operations.push(raise(0));
    }
    for seed in 0..40u64 {
        operations.push(overwrite(100 + seed));
    }
    operations
}

fn suffix_candidates() -> Vec<(String, HistoryOperation)> {
    let mut candidates = vec![
        ("Overwrite".to_string(), overwrite(7)),
        ("MountWritable".to_string(), HistoryOperation::CloseAndMountWritable),
        ("ColdStartRecover".to_string(), HistoryOperation::ColdStartRecover),
        (
            "Rollback(NewestFloor)".to_string(),
            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRootAtTheNewestFloor),
        ),
    ];
    for index in 0..4u64 {
        candidates.push((
            format!("Rollback(Ring{index})"),
            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                index_from_newest: index,
            }),
        ));
    }
    for steps in [0u64, 1, 2, 3, 5, 8, 13, 21, 34] {
        candidates.push((format!("Raise({steps})"), raise(steps)));
    }
    candidates
}


fn parallel<T: Send, R: Send>(items: Vec<T>, work: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let threads = 24usize;
    let items: Vec<(usize, T)> = items.into_iter().enumerate().collect();
    let mut buckets: Vec<Vec<(usize, T)>> = (0..threads).map(|_| Vec::new()).collect();
    for (index, item) in items {
        buckets[index % threads].push((index, item));
    }
    let mut results: Vec<(usize, R)> = std::thread::scope(|scope| {
        let handles: Vec<_> = buckets
            .into_iter()
            .map(|bucket| {
                let work = &work;
                scope.spawn(move || {
                    bucket
                        .into_iter()
                        .map(|(index, item)| (index, work(&item)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("线程"))
            .collect()
    });
    results.sort_by_key(|(index, _)| *index);
    results.into_iter().map(|(_, result)| result).collect()
}

fn mounts_max() -> usize {
    std::env::var("W2_MOUNTS_MAX").ok().and_then(|v| v.parse().ok()).unwrap_or(3)
}
fn noop_max() -> usize {
    std::env::var("W2_NOOP_MAX").ok().and_then(|v| v.parse().ok()).unwrap_or(4)
}

#[test]
fn w2_probe() {
    println!("# P1\tmounts\tnoop_raises\traise_steps\tapplied_overwrites\tfirst_refused_overwrite_index\traise_outcome\traise_writes\tending");
    let mut cases: Vec<(usize, usize, u64)> = Vec::new();
    for mounts in 1..=mounts_max() {
        for noop_raises in 0..=noop_max() {
            for raise_steps in [0u64, 3, 8, 20, 36] {
                cases.push((mounts, noop_raises, raise_steps));
            }
        }
    }
    let lines = parallel(cases, |(mounts, noop_raises, raise_steps)| {
        let (mounts, noop_raises, raise_steps) = (*mounts, *noop_raises, *raise_steps);
        let mut operations = prefix(mounts, noop_raises);
        let raise_index = operations.len();
        operations.push(raise(raise_steps));
        let result = run(operations);
        if result.steps.len() <= raise_index {
            return (
                format!("P1\t{mounts}\t{noop_raises}\t{raise_steps}\t-\t-\tSTOPPED_BEFORE_RAISE\t-\t{}", result.ending),
                None,
            );
        }
        let first_overwrite = mounts + noop_raises;
        let applied = result.steps[first_overwrite..raise_index]
            .iter()
            .filter(|step| step.outcome.starts_with("Applied"))
            .count();
        let first_refused = result.steps[first_overwrite..raise_index]
            .iter()
            .position(|step| !step.outcome.starts_with("Applied"))
            .map_or("-".to_string(), |index| index.to_string());
        let raise_step = &result.steps[raise_index];
        let noop: Vec<String> = result.steps[mounts..first_overwrite]
            .iter()
            .map(|step| format!("{}({})", short(&step.outcome), step.writes))
            .collect();
        let line = format!(
            "P1\t{mounts}\t{noop_raises}\t{raise_steps}\t{applied}\t{first_refused}\t{}\t{}\t{}\tnoop=[{}]",
            short(&raise_step.outcome),
            raise_step.writes,
            result.ending,
            noop.join(" ")
        );
        let refused = if raise_step.outcome.contains("NoFreeSlotOnAnyDevice") {
            Some((mounts, noop_raises, raise_steps, raise_step.writes > 0))
        } else {
            None
        };
        (line, refused)
    });
    let mut refused_histories: Vec<(usize, usize, u64, bool)> = Vec::new();
    for (line, refused) in lines {
        println!("{line}");
        if let Some((mounts, noop_raises, raise_steps, wrote)) = refused {
            if !refused_histories
                .iter()
                .any(|(m, n, _, w)| *m == mounts && *n == noop_raises && *w == wrote)
            {
                refused_histories.push((mounts, noop_raises, raise_steps, wrote));
            }
        }
    }
    println!("# P2 refused prefixes: {}", refused_histories.len());
    for entry in &refused_histories {
        println!("# P2 prefix {entry:?}");
    }
    if std::env::var("W2_P2").is_err() {
        return;
    }
    println!("# P2\tmounts\tnoop_raises\traise_steps\tsuffix\tsuffix_outcomes(writes)\tfinal_overwrite\tescaped\tending");
    let candidates = suffix_candidates();
    let mut jobs: Vec<(usize, usize, u64, Vec<usize>)> = Vec::new();
    for (mounts, noop_raises, raise_steps, _) in &refused_histories {
        for first in 0..candidates.len() {
            jobs.push((*mounts, *noop_raises, *raise_steps, vec![first]));
            for second in 0..candidates.len() {
                jobs.push((*mounts, *noop_raises, *raise_steps, vec![first, second]));
            }
        }
    }
    let results = parallel(jobs, |(mounts, noop_raises, raise_steps, suffix)| {
        let mut operations = prefix(*mounts, *noop_raises);
        operations.push(raise(*raise_steps));
        let suffix_start = operations.len();
        for index in suffix {
            operations.push(candidates[*index].1.clone());
        }
        operations.push(overwrite(9));
        let result = run(operations);
        let tail: Vec<String> = result
            .steps
            .iter()
            .skip(suffix_start)
            .map(|step| format!("{}({})", short(&step.outcome), step.writes))
            .collect();
        let escaped = result
            .steps
            .iter()
            .skip(suffix_start)
            .any(|step| step.outcome.starts_with("Applied(Published"));
        let names: Vec<&str> = suffix.iter().map(|index| candidates[*index].0.as_str()).collect();
        (
            format!(
                "P2\t{mounts}\t{noop_raises}\t{raise_steps}\t{}\t{}\t{}\t{escaped}\t{}",
                names.join("+"),
                tail[..tail.len().saturating_sub(1)].join(" "),
                tail.last().cloned().unwrap_or_default(),
                result.ending
            ),
            escaped,
        )
    });
    let total = results.len();
    let mut escapes = 0usize;
    for (line, escaped) in results {
        if escaped {
            escapes += 1;
        }
        println!("{line}");
    }
    println!("# P2 summary: histories={total} escaped={escapes}");
}

/// V1 对拍（「失败路径放开扣住」）：同一批历史在入库实现的副本与变异副本上各跑一遍。前缀照 P1（抬 F 目标 20 / 36，
/// 让扣住的槽里有释放代高于旧回收门槛的），后缀是被拒之后由用户决定的几步：连着覆盖写 1–3 次，再挂一次（可写 / 回退到
/// 根环里第 k 新的根 / 候选集下沿）或冷启动，最后一次覆盖写。只看结局：哪几段以新发现收尾、签名是什么。
#[test]
fn w2_v1_probe() {
    if std::env::var("W2_V1").is_err() {
        return;
    }
    let arm = std::env::var("W2_V1").unwrap();
    let mut jobs: Vec<(usize, usize, u64, usize, usize)> = Vec::new();
    let mut tails: Vec<(String, HistoryOperation)> = vec![
        ("MountWritable".to_string(), HistoryOperation::CloseAndMountWritable),
        ("ColdStartRecover".to_string(), HistoryOperation::ColdStartRecover),
        (
            "Rollback(NewestFloor)".to_string(),
            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRootAtTheNewestFloor),
        ),
    ];
    for index in 0..8u64 {
        tails.push((
            format!("Rollback(Ring{index})"),
            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                index_from_newest: index,
            }),
        ));
    }
    for mounts in 1..=3usize {
        for noop_raises in 0..=4usize {
            for raise_steps in [20u64, 36] {
                for overwrites in 1..=3usize {
                    for tail in 0..tails.len() {
                        jobs.push((mounts, noop_raises, raise_steps, overwrites, tail));
                    }
                }
            }
        }
    }
    let results = parallel(jobs, |(mounts, noop_raises, raise_steps, overwrites, tail)| {
        let mut operations = prefix(*mounts, *noop_raises);
        let raise_index = operations.len();
        operations.push(raise(*raise_steps));
        for seed in 0..*overwrites {
            operations.push(overwrite(500 + u64::try_from(seed).expect("小")));
        }
        operations.push(tails[*tail].1.clone());
        operations.push(overwrite(9));
        let result = run(operations);
        let outcomes: Vec<String> = result
            .steps
            .iter()
            .skip(raise_index)
            .map(|step| format!("{}({})", short(&step.outcome), step.writes))
            .collect();
        format!(
            "V1[{arm}]\t{mounts}\t{noop_raises}\t{raise_steps}\t{overwrites}\t{}\t{}\t{}",
            tails[*tail].0,
            outcomes.join(" "),
            result.ending
        )
    });
    for line in results {
        println!("{line}");
    }
}
