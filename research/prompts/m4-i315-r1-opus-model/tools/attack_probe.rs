//! 攻方腿草稿探针（不进仓，只在副本里跑）：故障注入 N 段 × 4 个注入点，CPU 池级 checker 就地判；
//! 每个吞写点打出：被吞的写落在哪、那一段字节之前有没有被写过（复用槽）、实现核出对不上交回的隔离份、
//! 注入之后与插回去两张盘上 I-3.1 / I-3.11 / I-3.15 的判定与 I-3.15 每一处红的记录，以及四种归类器的归类。
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{attack_set_external_quarantine, attack_take_log, check_pool_image};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::transaction::attack_take_quarantine_log;
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
    let _ = attack_take_log();
    let verdicts = check_pool_image(image);
    let log = attack_take_log();
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

/// 一条 I315_RED 日志 → (盘, 槽, 跨度)。
fn red_record(line: &str) -> Option<(u32, u64, u64)> {
    let mut parts = line.split('|');
    if parts.next()? != "I315_RED" {
        return None;
    }
    let _version = parts.next()?;
    let dev = parts.next()?.strip_prefix("dev=")?.parse().ok()?;
    let slot = parts.next()?.strip_prefix("slot=")?.parse().ok()?;
    let span = parts.next()?.strip_prefix("span=")?.parse().ok()?;
    Some((dev, slot, span))
}

fn handle(seed: u64, point: usize, images: &ImagesAfterAFault, quarantine_log: &[(u32, u64, u64, String)], out: &mut Vec<String>) {
    let ops = &images.image_after_the_fault.operations;
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
        out.push(format!("POINT seed={seed} point={point} fault={:?} swallowed=none today={}", images.drawn.fault, form(&classified)));
        return;
    };
    let inserted = landed.operations.iter().zip(ops.iter()).position(|(l, r)| !same(l, r)).unwrap_or(ops.len());
    let swallowed = &landed.operations[inserted].operation;
    let (sw_dev, sw_start, sw_end) = (swallowed.device.0, swallowed.offset.0, swallowed.offset.0 + swallowed.length);
    let reused = landed.operations[..inserted].iter().any(|op| {
        op.operation.device.0 == sw_dev && op.operation.length > 0 && op.operation.offset.0 < sw_end && sw_start < op.operation.offset.0 + op.operation.length
    });
    let overlaps_same_device = |dev: u32, slot: u64, span: u64| dev == sw_dev && sw_start < (slot + span) * SLOT && slot * SLOT < sw_end;
    let overlaps_unit = |slot: u64, span: u64| sw_start < (slot + span) * SLOT && slot * SLOT < sw_end;
    let flagged: BTreeSet<(u32, u64)> = quarantine_log.iter().map(|(d, s, _, _)| (*d, *s)).collect();
    let flagged_units: BTreeSet<(u64, u64)> = quarantine_log.iter().map(|(_, s, span, _)| (*s, *span)).collect();
    attack_set_external_quarantine(BTreeSet::new());
    let replug = judge_image(&image_of(&landed.operations, landed.operations.len()));
    let still = names(&replug.violations);
    let replug_reds: BTreeSet<(u32, u64, u64)> = replug.log.iter().filter_map(|l| red_record(l)).collect();
    let still_jia_device: BTreeSet<&'static str> = if !replug_reds.is_empty()
        && replug_reds.iter().all(|(d, s, span)| overlaps_same_device(*d, *s, *span) && flagged.contains(&(*d, *s)))
    {
        still.iter().copied().filter(|i| *i != "I-3.15").collect()
    } else {
        still.clone()
    };
    let still_jia_unit: BTreeSet<&'static str> = if !replug_reds.is_empty()
        && replug_reds.iter().all(|(_, s, span)| overlaps_unit(*s, *span) && flagged_units.contains(&(*s, *span)))
    {
        still.iter().copied().filter(|i| *i != "I-3.15").collect()
    } else {
        still.clone()
    };
    let external: BTreeSet<(u64, u64)> = flagged_units.iter().copied().filter(|(s, span)| overlaps_unit(*s, *span)).collect();
    attack_set_external_quarantine(external.clone());
    let replug_jia_two = judge_image(&image_of(&landed.operations, landed.operations.len()));
    attack_set_external_quarantine(BTreeSet::new());
    let still_jia_two = names(&replug_jia_two.violations);
    let today = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations.clone(), after.violations.clone(), still.clone());
    let jia_device = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations.clone(), after.violations.clone(), still_jia_device);
    let jia_unit = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations.clone(), after.violations.clone(), still_jia_unit);
    let jia_two = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations, after.violations.clone(), still_jia_two.clone());
    let red_steps: Vec<String> = step_judged
        .iter()
        .enumerate()
        .filter(|(_, j)| j.violations.iter().any(|(i, _)| i.starts_with("I-3.")))
        .map(|(index, j)| format!("{index}:{:?}", names(&j.violations).into_iter().filter(|i| i.starts_with("I-3.")).collect::<Vec<_>>()))
        .collect();
    out.push(format!(
        "POINT seed={seed} point={point} swallowed=dev{sw_dev}:slot{}:len{} reused={reused} flagged={quarantine_log:?} after_space={} after_names={:?} replug_space={} replug_names={:?} replug_reds={replug_reds:?} jia2_external={external:?} jia2_names={still_jia_two:?} steps_with_I3={red_steps:?}",
        sw_start / SLOT, swallowed.length, after.space, names(&after.violations), replug.space, still
    ));
    out.push(format!("CLASS seed={seed} point={point} today={} | jia_device={} | jia_unit={} | jia_two={}", form(&today), form(&jia_device), form(&jia_unit), form(&jia_two)));
    for line in after.log.iter().filter(|l| l.contains("I315_GROUP") || l.contains("I315_RED")) {
        out.push(format!("  AFTERLOG seed={seed} point={point} {line}"));
    }
    for line in replug.log.iter().filter(|l| l.contains("I315_GROUP")) {
        out.push(format!("  REPLUGLOG seed={seed} point={point} {line}"));
    }
}

#[test]
fn attack_probe() {
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let seed_count = env_u64("ATTACK_SEEDS", 24);
    let seed_offset = env_u64("ATTACK_SEED_OFFSET", 0);
    let steps = usize::try_from(env_u64("ATTACK_STEPS", 20)).unwrap();
    let only_seed = std::env::var("ATTACK_ONLY_SEED").ok().and_then(|v| v.parse::<u64>().ok());
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
                let _ = attack_take_quarantine_log();
                let mut handed: Vec<(ImagesAfterAFault, Vec<(u32, u64, u64, String)>)> = Vec::new();
                let _ = inject_faults_into_history_handing_out_the_images(&history, execution, 4, &mut |images| {
                    let log = attack_take_quarantine_log();
                    handed.push((images, log));
                });
                let mut out = Vec::new();
                for (point, (images, log)) in handed.iter().enumerate() {
                    if images.drawn.fault == InjectedFault::WriteIsSwallowed || std::env::var_os("ATTACK_ALL_POINTS").is_some() {
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
