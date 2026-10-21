//! 攻方原型（m4-batch2-code-r1）：随机历史每一步之后对镜像跑池级 checker，按 I-1.13 的判定（成立 / 违例 / 不适用理由）计数，
//! 违例逐条打出种子、步号与说明。只读镜像、不改产品代码。环境变量：ATTACK_FIRST_SEED、ATTACK_SEEDS、ATTACK_OPS、
//! ATTACK_WEIGHTS（broad / reuse / rollback / delete / orphans / wall）、ATTACK_DEVICES（384 / 256 / 4g）、ATTACK_THREADS。

use std::collections::BTreeMap;
use std::sync::Mutex;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryExecution, HistorySeed, PerStepChecker, StepPosition,
};
use singlefs_harness::SharedStream;

fn env_u64(name: &str, default: u64) -> u64 {
    match std::env::var(name) {
        Ok(value) => value.parse().expect("整数"),
        Err(_) => default,
    }
}

#[test]
#[ignore = "攻方原型：手动跑"]
fn tally_i113_verdicts_over_random_histories() {
    let first_seed = env_u64("ATTACK_FIRST_SEED", 1000);
    let seeds = env_u64("ATTACK_SEEDS", 8);
    let operations = usize::try_from(env_u64("ATTACK_OPS", 30)).expect("步数");
    let threads = env_u64("ATTACK_THREADS", 8);
    let weights_name = std::env::var("ATTACK_WEIGHTS").unwrap_or_else(|_| "broad".to_string());
    let weights = match weights_name.as_str() {
        "broad" => GenerationWeights::BROAD,
        "reuse" => GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        "rollback" => GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        "delete" => GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES,
        "orphans" => GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP,
        "wall" => GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        other => panic!("未知比重 {other}"),
    };
    let device_width = match std::env::var("ATTACK_DEVICES").as_deref() {
        Ok("256") => HistoryDeviceWidth::UnitAreaOf256Slots,
        Ok("4g") => HistoryDeviceWidth::FourGibibytes,
        _ => HistoryDeviceWidth::UnitAreaOf384Slots,
    };
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let tally: Mutex<BTreeMap<String, u64>> = Mutex::new(BTreeMap::new());
    let violations: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let other_reds: Mutex<BTreeMap<String, u64>> = Mutex::new(BTreeMap::new());
    let next = Mutex::new(0u64);
    let started = std::time::Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = {
                    let mut next = next.lock().expect("锁");
                    if *next >= seeds {
                        break;
                    }
                    *next += 1;
                    *next - 1
                };
                let seed = HistorySeed(first_seed + index);
                let history = generate_history_with_weights(seed, operations, &weights);
                let _run = execute_history_with(
                    &history,
                    execution,
                    &SharedStream::new(),
                    &mut |observation| {
                        let step = match observation.position {
                            StepPosition::StartingPoint => "起点".to_string(),
                            StepPosition::Operation(at) => format!("第 {at} 步"),
                        };
                        let operation = match observation.operation {
                            Some(operation) => format!("{:?}", operation.kind()),
                            None => "起点".to_string(),
                        };
                        for (invariant, verdict) in check_pool_image(observation.image) {
                            if invariant == "I-1.13" {
                                let key = match &verdict {
                                    InvariantVerdict::Holds => "成立".to_string(),
                                    InvariantVerdict::Violated(_) => "违例".to_string(),
                                    InvariantVerdict::NotApplicable(reason) => {
                                        format!("不适用：{reason}")
                                    }
                                    InvariantVerdict::PartiallyJudged(_) => "部分没判".to_string(),
                                };
                                *tally
                                    .lock()
                                    .expect("锁")
                                    .entry(format!("{key}｜上一步 {operation}"))
                                    .or_insert(0) += 1;
                                if let InvariantVerdict::Violated(detail) = &verdict {
                                    violations.lock().expect("锁").push(format!(
                                        "种子 {} {step}（{operation}）：{detail}",
                                        seed.0
                                    ));
                                }
                            } else if let InvariantVerdict::Violated(_) = verdict {
                                *other_reds
                                    .lock()
                                    .expect("锁")
                                    .entry(invariant.to_string())
                                    .or_insert(0) += 1;
                            }
                        }
                    },
                );
            });
        }
    });
    println!(
        "ATTACK_SUMMARY weights={weights_name} seeds={seeds} first_seed={first_seed} ops={operations} elapsed_s={:.1}",
        started.elapsed().as_secs_f64()
    );
    for (key, count) in tally.lock().expect("锁").iter() {
        println!("ATTACK_I113 {count} {key}");
    }
    for (invariant, count) in other_reds.lock().expect("锁").iter() {
        println!("ATTACK_OTHER_RED {count} {invariant}");
    }
    let violations = violations.lock().expect("锁");
    println!("ATTACK_VIOLATIONS {}", violations.len());
    for violation in violations.iter().take(20) {
        println!("ATTACK_VIOLATION {violation}");
    }
}
