//! 辩方腿草稿探针（m4-i315-r2，不进仓，只在副本里跑）：故障注入 N 段 × 4 个注入点，CPU 池级 checker 就地判。
//! 底是第一轮攻方腿的探针，改了三处：开关换成 I315_ARM（today / exempt / gen，walk.rs 副本里读）；
//! 归类器四列换成 today（只比名）、jia_two（外给清单取与被吞的写重叠的单元，插回去那张盘再判）、
//! jia_all（外给清单取实现交出的全部隔离单元，插回去那张盘再判）、mark（丙的模型：实现交出的隔离单元当盘上记号，
//! 注入之后那张盘与每步镜像上就带着，不用插回去）；每个点另打吞写落点是否复用槽、实现交出的隔离份。
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{check_pool_image, defense_set_external_quarantine, defense_take_log};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::transaction::defense_take_quarantine_log;
use singlefs_harness::fault_injection::{
    classify_the_pool_checker_verdicts_of_the_images_after_a_fault,
    inject_faults_into_history_handing_out_the_images, FailureAfterAFault, ImagesAfterAFault,
    InjectedFault,
};
use singlefs_harness::history::{
    generate_history_with_weights, GenerationWeights, HistoryDeviceWidth, HistoryExecution,
    HistorySeed, PerStepChecker, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::RetainedOperation;

const SLOT: u64 = 16384;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn image_of(operations: &[RetainedOperation], prefix: usize) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        HistoryDeviceWidth::FourGibibytes.device_bytes(),
    );
    pool.apply(&operations[..prefix]);
    pool
}

struct Judged {
    violations: Vec<(&'static str, String)>,
    space: String,
    log: Vec<String>,
}

fn judge_image(image: &MemoryPool) -> Judged {
    let _ = defense_take_log();
    let verdicts = check_pool_image(image);
    let log = defense_take_log();
    let mut space = Vec::new();
    let mut violations = Vec::new();
    for (invariant, verdict) in verdicts {
        let shape = match &verdict {
            InvariantVerdict::Holds => "成立",
            InvariantVerdict::Violated(_) => "违例",
            InvariantVerdict::NotApplicable(_) => "不适用",
            InvariantVerdict::PartiallyJudged(_) => "部分判",
        };
        if ["I-3.1", "I-3.11", "I-3.15"].contains(&invariant) {
            space.push(format!("{invariant}={shape}"));
        }
        if let InvariantVerdict::Violated(detail) = verdict {
            violations.push((invariant, detail));
        }
    }
    Judged { violations, space: space.join(","), log }
}

fn names(violations: &[(&'static str, String)]) -> BTreeSet<&'static str> {
    violations.iter().map(|(i, _)| *i).collect()
}

fn same(left: &RetainedOperation, right: &RetainedOperation) -> bool {
    left.operation == right.operation && left.contents == right.contents
}

fn form(failures: &[FailureAfterAFault]) -> String {
    failures
        .iter()
        .map(|failure| match failure {
            FailureAfterAFault::KnownRed(_) => "known-red".to_string(),
            FailureAfterAFault::LeftByTheLyingDevice { signature } => format!("lying{signature:?}"),
            FailureAfterAFault::NewFinding { signature, .. } => format!("NEW{signature:?}"),
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

fn handle(seed: u64, point: usize, images: &ImagesAfterAFault, quarantine_log: &[(u32, u64, u64, String)], out: &mut Vec<String>) {
    let ops = &images.image_after_the_fault.operations;
    defense_set_external_quarantine(BTreeSet::new());
    let step_judged: Vec<Judged> = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .map(|step| judge_image(&image_of(ops, step.operations_recorded)))
        .collect();
    let after = judge_image(&image_of(ops, ops.len()));
    let step_violations: Vec<Vec<(&'static str, String)>> =
        step_judged.iter().map(|j| j.violations.clone()).collect();
    let Some(landed) = images.image_with_the_swallowed_writes_landed.as_ref() else {
        let classified = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations, after.violations.clone(), BTreeSet::new());
        out.push(format!("POINT seed={seed} point={point} fault={:?} swallowed=none flagged={quarantine_log:?} today={}", images.drawn.fault, form(&classified)));
        return;
    };
    let inserted = landed.operations.iter().zip(ops.iter()).position(|(l, r)| !same(l, r)).unwrap_or(ops.len());
    let swallowed = &landed.operations[inserted].operation;
    let (sw_dev, sw_start, sw_end) = (swallowed.device.0, swallowed.offset.0, swallowed.offset.0 + swallowed.length);
    let reused = landed.operations[..inserted].iter().any(|op| {
        op.operation.device.0 == sw_dev && op.operation.length > 0 && op.operation.offset.0 < sw_end && sw_start < op.operation.offset.0 + op.operation.length
    });
    let overlaps_unit = |slot: u64, span: u64| sw_start < (slot + span) * SLOT && slot * SLOT < sw_end;
    let flagged_units: BTreeSet<(u64, u64)> = quarantine_log.iter().map(|(_, s, span, _)| (*s, *span)).collect();
    let external_overlapping: BTreeSet<(u64, u64)> = flagged_units.iter().copied().filter(|(s, span)| overlaps_unit(*s, *span)).collect();
    // today：插回去那张盘不带外给清单
    let replug = judge_image(&image_of(&landed.operations, landed.operations.len()));
    let still = names(&replug.violations);
    let replug_reds: Vec<String> = replug.log.iter().filter(|l| l.starts_with("I315_RED")).cloned().collect();
    // 甲二：外给清单取与被吞的写重叠的单元
    defense_set_external_quarantine(external_overlapping.clone());
    let still_jia_two = names(&judge_image(&image_of(&landed.operations, landed.operations.len())).violations);
    // 甲二（不滤重叠）：外给清单取实现交出的全部隔离单元
    defense_set_external_quarantine(flagged_units.clone());
    let still_jia_all = names(&judge_image(&image_of(&landed.operations, landed.operations.len())).violations);
    // 丙的模型：注入之后那张盘与每步镜像上就带着记号（实现交出的隔离单元）；记号随隔离那一刻才有，这里整份给每步镜像，
    // 比真记号宽：只会多遮，不会少遮（模型的方向，报告里写明）
    defense_set_external_quarantine(flagged_units.clone());
    let mark_after = judge_image(&image_of(ops, ops.len()));
    let mark_steps: Vec<Vec<(&'static str, String)>> = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .map(|step| judge_image(&image_of(ops, step.operations_recorded)).violations)
        .collect();
    let mark_still = names(&judge_image(&image_of(&landed.operations, landed.operations.len())).violations);
    defense_set_external_quarantine(BTreeSet::new());
    let today = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations.clone(), after.violations.clone(), still.clone());
    let jia_two = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations.clone(), after.violations.clone(), still_jia_two.clone());
    let jia_all = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations, after.violations.clone(), still_jia_all.clone());
    let mark = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, mark_steps, mark_after.violations.clone(), mark_still.clone());
    out.push(format!(
        "POINT seed={seed} point={point} swallowed=dev{sw_dev}:slot{}:len{} reused={reused} flagged={quarantine_log:?} overlapping={external_overlapping:?} after_space={} after_names={:?} replug_space={} replug_names={:?} replug_reds={replug_reds:?} jia2_names={still_jia_two:?}",
        sw_start / SLOT, swallowed.length, after.space, names(&after.violations), replug.space, still
    ));
    out.push(format!("CLASS seed={seed} point={point} today={} | jia_two={} | jia_all={} | mark={}", form(&today), form(&jia_two), form(&jia_all), form(&mark)));
    for line in after.log.iter().filter(|l| l.contains("I315_GROUP") || l.contains("I315_RED") || l.contains("Q31_GROUP")) {
        out.push(format!("  AFTERLOG seed={seed} point={point} {line}"));
    }
}

#[test]
fn defense_probe() {
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let seed_count = env_u64("DEFENSE_SEEDS", 24);
    let seed_offset = env_u64("DEFENSE_SEED_OFFSET", 0);
    let steps = usize::try_from(env_u64("DEFENSE_STEPS", 20)).unwrap();
    let only_seed = std::env::var("DEFENSE_ONLY_SEED").ok().and_then(|v| v.parse::<u64>().ok());
    let seeds: Vec<u64> = (0..seed_count)
        .map(|offset| SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + seed_offset + offset)
        .filter(|seed| only_seed.is_none_or(|only| only == *seed))
        .collect();
    let threads = usize::try_from(env_u64("SINGLEFS_THREAD_CAP", 1)).unwrap();
    let next = AtomicUsize::new(0);
    let lines: Mutex<Vec<(u64, Vec<String>)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::SeqCst);
                let Some(seed) = seeds.get(index) else { break };
                let history = generate_history_with_weights(HistorySeed(*seed), steps, &GenerationWeights::BROAD);
                let _ = defense_take_quarantine_log();
                let mut handed: Vec<(ImagesAfterAFault, Vec<(u32, u64, u64, String)>)> = Vec::new();
                let _ = inject_faults_into_history_handing_out_the_images(&history, execution, 4, &mut |images| {
                    let log = defense_take_quarantine_log();
                    handed.push((images, log));
                });
                let mut out = Vec::new();
                for (point, (images, log)) in handed.iter().enumerate() {
                    if images.drawn.fault == InjectedFault::WriteIsSwallowed || std::env::var_os("DEFENSE_ALL_POINTS").is_some() {
                        handle(*seed, point, images, log, &mut out);
                    }
                }
                lines.lock().unwrap().push((*seed, out));
            });
        }
    });
    let mut lines = lines.into_inner().unwrap();
    lines.sort_by_key(|(seed, _)| *seed);
    for (_, out) in lines {
        for line in out {
            println!("{line}");
        }
    }
}
