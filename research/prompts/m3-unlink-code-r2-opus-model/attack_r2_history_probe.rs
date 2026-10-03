//! 攻方原型（m3-unlink-code-r2，云端攻方）：N1、N3——执行器不再跳过删除中的版本之后，走得到删除中的那一版的历史，
//! 前缀只固定「造出删除中的一版」那几步与覆盖写次数、盘宽，后缀把用户动作放开扫（长度 1、2 穷举，长度 3–6 按种子抽）。
//! 判的是执行器自己的结局：模型对不上、checker 红、panic、执行器判出的失败都算打中。
//! 只放进冻结副本的拷贝里跑（harness 档，测试名不带 layer0）。

use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, FloorTargetChoice, GeneratedHistory,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed,
    HistoryStartingPoint, PerStepChecker, RollbackTargetChoice, StepOutcome,
};
use singlefs_harness::SharedStream;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

fn overwrite(selector: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector },
        fill_seed: selector,
    })
}

fn alphabet() -> Vec<HistoryOperation> {
    vec![
        overwrite(2999),
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::Empty,
            fill_seed: 1,
        }),
        HistoryOperation::Unlink,
        HistoryOperation::CreateInode,
        HistoryOperation::CloseAndMountWritable,
        HistoryOperation::ColdStartRecover,
        HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 0 }),
        HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 1 }),
        HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 2 }),
        HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 3 }),
        HistoryOperation::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: 1 }),
        HistoryOperation::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: 3 }),
        HistoryOperation::PublishWithoutUnits,
        HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
    ]
}

fn prefixes(overwrites: usize) -> Vec<(&'static str, Vec<HistoryOperation>)> {
    let ows: Vec<HistoryOperation> = (0..overwrites).map(|i| overwrite(100 + i as u64)).collect();
    let with = |tail: &[HistoryOperation]| {
        let mut ops = ows.clone();
        ops.extend_from_slice(tail);
        ops
    };
    let back_to_intent =
        HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 2 });
    vec![
        ("PA-rollback-to-intent", with(&[HistoryOperation::Unlink, HistoryOperation::CreateInode, back_to_intent])),
        ("PB-rollback-to-intent-then-mount", with(&[HistoryOperation::Unlink, HistoryOperation::CreateInode, back_to_intent, HistoryOperation::CloseAndMountWritable])),
        ("PC-unlink", with(&[HistoryOperation::Unlink])),
        ("PD-unlink-then-abandon", with(&[HistoryOperation::Unlink, HistoryOperation::CrashRecoveryAbandoningTheNewestRoot])),
    ]
}

fn executions() -> Vec<(&'static str, HistoryExecution)> {
    let make = |device_width, space_admission| HistoryExecution {
        per_step_checker: PerStepChecker::Run,
        device_width,
        space_admission,
    };
    let all = vec![
        ("4G", make(HistoryDeviceWidth::FourGibibytes, SpaceAdmission::JudgedByTheFormula)),
        ("384", make(HistoryDeviceWidth::UnitAreaOf384Slots, SpaceAdmission::JudgedByTheFormula)),
        ("256", make(HistoryDeviceWidth::UnitAreaOf256Slots, SpaceAdmission::JudgedByTheFormula)),
        ("240", make(HistoryDeviceWidth::UnitAreaOf240Slots, SpaceAdmission::JudgedByTheFormula)),
        ("256-skip", make(HistoryDeviceWidth::UnitAreaOf256Slots, SpaceAdmission::SkippedByTheTestOnlySwitch)),
        ("240-skip", make(HistoryDeviceWidth::UnitAreaOf240Slots, SpaceAdmission::SkippedByTheTestOnlySwitch)),
    ];
    let wanted = std::env::var("ATTACK_EXECUTIONS").unwrap_or_default();
    all.into_iter()
        .filter(|(name, _)| wanted.is_empty() || wanted.split(',').any(|w| w == *name))
        .collect()
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// 简单的确定性抽样（xorshift），只给后缀抽长度与操作。
fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[test]
fn attack_r2_history_probe() {
    let alphabet = alphabet();
    let overwrites: Vec<usize> = std::env::var("ATTACK_OVERWRITES")
        .unwrap_or_else(|_| "0,2,5".to_string())
        .split(',')
        .map(|v| v.parse().expect("数"))
        .collect();
    let exhaustive_length = env_usize("ATTACK_EXHAUSTIVE_LENGTH", 2);
    let random_suffixes = env_usize("ATTACK_RANDOM_SUFFIXES", 0);
    let random_max_length = env_usize("ATTACK_RANDOM_MAX_LENGTH", 6);
    let prefix_filter = std::env::var("ATTACK_PREFIXES").unwrap_or_default();
    let mut work: Vec<(String, GeneratedHistory, HistoryExecution, usize)> = Vec::new();
    for (execution_name, execution) in executions() {
        for &k in &overwrites {
            for (prefix_name, prefix) in prefixes(k) {
                if !prefix_filter.is_empty() && !prefix_filter.split(',').any(|p| prefix_name.starts_with(p)) {
                    continue;
                }
                let mut suffixes: Vec<Vec<HistoryOperation>> = vec![Vec::new()];
                let mut layer: Vec<Vec<HistoryOperation>> = vec![Vec::new()];
                for _ in 0..exhaustive_length {
                    let mut next_layer = Vec::new();
                    for base in &layer {
                        for op in &alphabet {
                            let mut s = base.clone();
                            s.push(*op);
                            next_layer.push(s);
                        }
                    }
                    suffixes.extend(next_layer.iter().cloned());
                    layer = next_layer;
                }
                let mut state: u64 = 0x9E37_79B9_7F4A_7C15 ^ (k as u64) ^ ((prefix_name.len() as u64) << 32);
                for _ in 0..random_suffixes {
                    let length = 3 + (next(&mut state) as usize) % (random_max_length - 2);
                    suffixes.push((0..length).map(|_| alphabet[(next(&mut state) as usize) % alphabet.len()]).collect());
                }
                for suffix in suffixes {
                    let mut operations = prefix.clone();
                    operations.extend(suffix);
                    let label = format!("{execution_name} k={k} {prefix_name}");
                    work.push((
                        label,
                        GeneratedHistory {
                            seed: HistorySeed(0),
                            starting_point: HistoryStartingPoint::AfterFirstFile,
                            operations,
                        },
                        execution,
                        prefix.len(),
                    ));
                }
            }
        }
    }
    let threads = env_usize("SINGLEFS_THREAD_CAP", 4).max(1);
    println!("WORK histories={} threads={threads}", work.len());
    let cursor = AtomicUsize::new(0);
    let out = Mutex::new(std::io::stdout());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = cursor.fetch_add(1, Ordering::Relaxed);
                let Some((label, history, execution, prefix_len)) = work.get(index) else {
                    break;
                };
                let run = execute_history_with(history, *execution, &SharedStream::new(), &mut |_| {});
                let outcomes: Vec<String> = run
                    .outcomes
                    .iter()
                    .map(|o| match o {
                        StepOutcome::Applied(effect) => format!("A:{}", format!("{effect:?}").split(|c: char| c == ' ' || c == '{' || c == '(').next().unwrap_or("")),
                        StepOutcome::Refused { member } => format!("R:{member}"),
                        StepOutcome::NotApplicable(missing) => format!("N:{missing:?}"),
                    })
                    .collect();
                let ending = match &run.ending {
                    HistoryEnding::Completed => "Completed".to_string(),
                    HistoryEnding::KnownRed { form, .. } => format!("KnownRed{form}"),
                    HistoryEnding::NewFinding { signature, observation } => format!(
                        "NEWFINDING {signature:?} at={:?} op={:?} model={:?} harness={:?} panic={:?} violations={:?}",
                        observation.position,
                        observation.operation_kind,
                        observation.model_disagreement,
                        observation.harness_judgement,
                        observation.panic,
                        observation.violations
                    ),
                };
                let line = format!(
                    "H {label} prefix_len={prefix_len} ops={:?} | outcomes={outcomes:?} | ending={ending}",
                    history.operations
                );
                use std::io::Write;
                let mut guard = out.lock().expect("stdout");
                writeln!(guard, "{line}").expect("写");
            });
        }
    });
    println!("DONE");
}
