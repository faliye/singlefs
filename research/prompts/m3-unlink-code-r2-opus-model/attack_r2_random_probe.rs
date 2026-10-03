//! 攻方原型（m3-unlink-code-r2，云端攻方）：N1、N3——按种子生成的随机历史，比重往「unlink、回退到意图那一版、删除中的一版上做用户改动、
//! 窄盘逼近单元区墙」那边偏；执行器副本里加了两行 eprintln（ATTACK-WALK-FROM-DELETING、ATTACK-MOUNT-CONTINUATION）数走到了哪几支。
//! 只放进冻结副本的拷贝里跑（harness 档，测试名不带 layer0）。

use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperationKind, HistorySeed, HistoryStartingPoint,
    NamedGenerationWeights, PerStepChecker, RollbackTargetDraw,
};
use singlefs_harness::SharedStream;
use std::sync::atomic::{AtomicU64, Ordering};

const UNLINK_NEAR_THE_WALL: GenerationWeights = GenerationWeights {
    name: "攻方：删除中的一版上的用户改动（逼近单元区墙）",
    named: NamedGenerationWeights::TowardTheUnitAreaWall,
    starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
    with_session_closed: &[
        (HistoryOperationKind::CloseAndMountWritable, 90),
        (HistoryOperationKind::ColdStartRecover, 10),
    ],
    with_session_open_without_file: &[
        (HistoryOperationKind::PublishFirstFile, 80),
        (HistoryOperationKind::CloseAndMountWritable, 20),
    ],
    with_session_open_with_file: &[
        (HistoryOperationKind::PublishOverwrite, 62),
        (HistoryOperationKind::Unlink, 8),
        (HistoryOperationKind::CreateInode, 6),
        (HistoryOperationKind::RollBackWhileMounted, 10),
        (HistoryOperationKind::CloseAndMountWritable, 6),
        (HistoryOperationKind::RaiseRollbackFloor, 5),
        (HistoryOperationKind::ColdStartRecover, 2),
        (HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot, 1),
    ],
    with_session_open_after_the_file_was_deleted: Some(&[
        (HistoryOperationKind::CreateInode, 25),
        (HistoryOperationKind::RollBackWhileMounted, 25),
        (HistoryOperationKind::PublishOverwrite, 20),
        (HistoryOperationKind::Unlink, 10),
        (HistoryOperationKind::CloseAndMountWritable, 15),
        (HistoryOperationKind::RaiseRollbackFloor, 5),
    ]),
    rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
};

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

#[test]
fn attack_r2_random_probe() {
    let seed_base = env_u64("ATTACK_SEED_BASE", 0x5EED_0000);
    let seeds = env_u64("ATTACK_SEEDS", 60);
    let operations = usize::try_from(env_u64("ATTACK_OPERATIONS", 40)).expect("步数");
    let execution_name = std::env::var("ATTACK_EXECUTION").unwrap_or_else(|_| "256-skip".to_string());
    let weights_name = std::env::var("ATTACK_WEIGHTS").unwrap_or_else(|_| "attack".to_string());
    let weights = match weights_name.as_str() {
        "attack" => UNLINK_NEAR_THE_WALL,
        "wall" => GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        "broad" => GenerationWeights::BROAD,
        other => panic!("比重 {other}"),
    };
    let (device_width, space_admission) = match execution_name.as_str() {
        "4G" => (HistoryDeviceWidth::FourGibibytes, SpaceAdmission::JudgedByTheFormula),
        "384" => (HistoryDeviceWidth::UnitAreaOf384Slots, SpaceAdmission::JudgedByTheFormula),
        "256" => (HistoryDeviceWidth::UnitAreaOf256Slots, SpaceAdmission::JudgedByTheFormula),
        "256-skip" => (HistoryDeviceWidth::UnitAreaOf256Slots, SpaceAdmission::SkippedByTheTestOnlySwitch),
        "240" => (HistoryDeviceWidth::UnitAreaOf240Slots, SpaceAdmission::JudgedByTheFormula),
        "240-skip" => (HistoryDeviceWidth::UnitAreaOf240Slots, SpaceAdmission::SkippedByTheTestOnlySwitch),
        other => panic!("盘宽 {other}"),
    };
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Run,
        device_width,
        space_admission,
    };
    let threads = usize::try_from(env_u64("SINGLEFS_THREAD_CAP", 4)).expect("线程").max(1);
    let cursor = AtomicU64::new(0);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let offset = cursor.fetch_add(1, Ordering::Relaxed);
                if offset >= seeds {
                    break;
                }
                let seed = HistorySeed(seed_base.wrapping_add(offset));
                let history = generate_history_with_weights(seed, operations, &weights);
                let run = execute_history_with(&history, execution, &SharedStream::new(), &mut |_| {});
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
                let mut refused: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
                for outcome in &run.outcomes {
                    if let singlefs_harness::history::StepOutcome::Refused { member } = outcome {
                        *refused.entry(member.chars().take(70).collect()).or_insert(0) += 1;
                    }
                }
                println!("R exec={execution_name} weights={weights_name} seed={:#x} ops={operations} refused={refused:?} ending={ending}", seed.0);
            });
        }
    });
    println!("DONE seeds={seeds}");
}
