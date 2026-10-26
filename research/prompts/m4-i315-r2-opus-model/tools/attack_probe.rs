//! m4-i315-r2 攻方腿草稿探针（不进仓，只在副本里跑）：故障注入 N 段 × 4 个注入点，CPU 池级 checker 就地判。
//! 「没有位置项可比」那一格两种 checker 判法（退回自证 = 今天、当隔离豁免）× 两种归类器（今天只比名、甲二插回去那张盘照实现隔离清单
//! 里与被吞的写重叠的单元再判一遍），每个注入点各归一次；另记这一格被走到几次、自证过没过，与缺陷开关留下的泄漏。
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{
    attack_set_external_quarantine, attack_set_noloc_exempt, attack_take_noloc_hits, check_pool_image,
};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::transaction::{attack_take_leak_log, attack_take_quarantine_log};
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
type Violations = Vec<(&'static str, String)>;

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

/// 判一张镜像：违例、三条空间记账不变量的判定一行、这一格被走到的记录。
fn judge_image(image: &MemoryPool) -> (Violations, String, Vec<String>) {
    let _ = attack_take_noloc_hits();
    let verdicts = check_pool_image(image);
    let hits = attack_take_noloc_hits();
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
    (violations, space.join(","), hits)
}

fn names(violations: &Violations) -> BTreeSet<&'static str> {
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

fn hit_summary(hits: &[String]) -> String {
    let passes = hits.iter().filter(|h| h.ends_with("self_check_passes=true")).count();
    format!("{}/{}", passes, hits.len())
}

#[allow(clippy::too_many_lines)]
fn handle(
    seed: u64,
    point: usize,
    images: &ImagesAfterAFault,
    quarantine_log: &[(u32, u64, u64, String)],
    leak_log: &[(u64, u64, u64)],
    out: &mut Vec<String>,
) {
    let ops = &images.image_after_the_fault.operations;
    let landed = images.image_with_the_swallowed_writes_landed.as_ref();
    let mut swallowed_text = "none".to_string();
    let mut external: BTreeSet<(u64, u64)> = BTreeSet::new();
    if let Some(landed) = landed {
        let inserted = landed.operations.iter().zip(ops.iter()).position(|(l, r)| !same(l, r)).unwrap_or(ops.len());
        let swallowed = &landed.operations[inserted].operation;
        let (sw_dev, sw_start, sw_end) = (swallowed.device.0, swallowed.offset.0, swallowed.offset.0 + swallowed.length);
        let reused = landed.operations[..inserted].iter().any(|op| {
            op.operation.device.0 == sw_dev && op.operation.length > 0 && op.operation.offset.0 < sw_end && sw_start < op.operation.offset.0 + op.operation.length
        });
        let overlaps_unit = |slot: u64, span: u64| sw_start < (slot + span) * SLOT && slot * SLOT < sw_end;
        external = quarantine_log.iter().map(|(_, s, span, _)| (*s, *span)).filter(|(s, span)| overlaps_unit(*s, *span)).collect();
        swallowed_text = format!("dev{sw_dev}:slot{}:len{} reused={reused}", sw_start / SLOT, swallowed.length);
    }
    out.push(format!(
        "POINT seed={seed} point={point} fault={} swallowed={swallowed_text} flagged={quarantine_log:?} leaks={leak_log:?} jia2_external={external:?}",
        images.drawn.fault.name()
    ));
    for exempt in [false, true] {
        attack_set_noloc_exempt(exempt);
        attack_set_external_quarantine(BTreeSet::new());
        let arm = if exempt { "exempt" } else { "selfcheck" };
        let step_judged: Vec<(Violations, String, Vec<String>)> = images
            .step_images_of_the_run_with_the_fault
            .iter()
            .map(|step| judge_image(&image_of(ops, step.operations_recorded)))
            .collect();
        let step_violations: Vec<Violations> = step_judged.iter().map(|j| j.0.clone()).collect();
        let step_hits: Vec<String> = step_judged.iter().flat_map(|j| j.2.clone()).collect();
        let (after_violations, after_space, after_hits) = judge_image(&image_of(ops, ops.len()));
        let red_steps: Vec<String> = step_judged
            .iter()
            .enumerate()
            .filter(|(_, j)| j.0.iter().any(|(i, _)| i.starts_with("I-3.")))
            .map(|(index, j)| format!("{index}:{:?}", names(&j.0).into_iter().filter(|i| i.starts_with("I-3.")).collect::<Vec<_>>()))
            .collect();
        let Some(landed) = landed else {
            let classified = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations, after_violations.clone(), BTreeSet::new());
            out.push(format!(
                "CLASS seed={seed} point={point} arm={arm} after_space={after_space} after_names={:?} steps_with_I3={red_steps:?} noloc_steps={} noloc_after={} today={} | jia_two={}",
                names(&after_violations), hit_summary(&step_hits), hit_summary(&after_hits), form(&classified), form(&classified)
            ));
            continue;
        };
        let landed_image = image_of(&landed.operations, landed.operations.len());
        let (replug, replug_space, replug_hits) = judge_image(&landed_image);
        attack_set_external_quarantine(external.clone());
        let (replug_jia_two, replug_jia_two_space, _) = judge_image(&landed_image);
        attack_set_external_quarantine(BTreeSet::new());
        let today = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations.clone(), after_violations.clone(), names(&replug));
        let jia_two = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations, after_violations.clone(), names(&replug_jia_two));
        out.push(format!(
            "CLASS seed={seed} point={point} arm={arm} after_space={after_space} after_names={:?} replug_space={replug_space} replug_names={:?} replug_jia2_space={replug_jia_two_space} replug_jia2_names={:?} steps_with_I3={red_steps:?} noloc_steps={} noloc_after={} noloc_replug={} today={} | jia_two={}",
            names(&after_violations), names(&replug), names(&replug_jia_two), hit_summary(&step_hits), hit_summary(&after_hits), hit_summary(&replug_hits), form(&today), form(&jia_two)
        ));
    }
    attack_set_noloc_exempt(false);
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
    let all_points = std::env::var_os("ATTACK_ALL_POINTS").is_some();
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
                let _ = attack_take_leak_log();
                let mut handed: Vec<(ImagesAfterAFault, Vec<(u32, u64, u64, String)>, Vec<(u64, u64, u64)>)> = Vec::new();
                let _ = inject_faults_into_history_handing_out_the_images(&history, execution, 4, &mut |images| {
                    handed.push((images, attack_take_quarantine_log(), attack_take_leak_log()));
                });
                let mut out = Vec::new();
                for (point, (images, log, leaks)) in handed.iter().enumerate() {
                    if images.drawn.fault == InjectedFault::WriteIsSwallowed || all_points {
                        handle(*seed, point, images, log, leaks, &mut out);
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
