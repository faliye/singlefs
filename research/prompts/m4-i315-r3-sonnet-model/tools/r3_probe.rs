//! m4-i315-r3 辩方腿探针（不进仓，只在副本里跑）：故障注入 N 段 × 4 个注入点，CPU 池级 checker 就地判。
//! 同一个注入点上，每个臂（walk.rs 副本里的 defense_set_arm）各判一遍，再按三种归类器各归一次：
//!   name  今天：插回去那张盘不带外给清单，按不变量名比；
//!   j2n   甲二按名比：插回去那张盘把实现交出、与被吞的写有重叠的单元当隔离单元，按不变量名比；
//!   j2s   甲二按（盘, 槽）比：同上，但「仍红」按（不变量, 盘, 槽）认，I-3.1 / I-3.11 / I-3.15 三条的违例点名的槽取自 checker 的日志
//!         （Q31_GROUP 未豁免的单元、I315_RED），别的不变量没有槽信息，退回按名。
//! 环境变量：DEFENSE_SEEDS、DEFENSE_SEED_OFFSET、DEFENSE_STEPS、DEFENSE_ARMS（逗号分隔）、DEFENSE_ONLY_SEED、ATTACK_DEFECT（实现缺陷）。
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{check_pool_image, defense_set_arm, defense_set_external_quarantine, defense_take_log};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::transaction::{defense_take_leak_log, defense_take_quarantine_log};
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
type Sites = BTreeSet<(&'static str, u32, u64)>;

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
    violations: Violations,
    space: String,
    sites: Sites,
    noentry: BTreeSet<String>,
}

fn field(line: &str, key: &str) -> Option<String> {
    line.split('|').find_map(|part| part.strip_prefix(key)).map(|v| v.to_string())
}

fn sites_of(log: &[String]) -> Sites {
    let mut sites = Sites::new();
    for line in log {
        if line.starts_with("Q31_GROUP") && field(line, "exempt=").as_deref() == Some("false") {
            let slot: u64 = field(line, "slot=").and_then(|v| v.parse().ok()).unwrap_or(0);
            let devices = field(line, "devices=").unwrap_or_default();
            for device in devices.trim_matches(|c| c == '[' || c == ']').split(',').filter_map(|d| d.trim().parse::<u32>().ok()) {
                sites.insert(("I-3.1", device, slot));
                sites.insert(("I-3.11", device, slot));
            }
        }
        if line.starts_with("I315_RED") {
            let slot: u64 = field(line, "slot=").and_then(|v| v.parse().ok()).unwrap_or(0);
            let device: u32 = field(line, "dev=").and_then(|v| v.parse().ok()).unwrap_or(0);
            sites.insert(("I-3.15", device, slot));
        }
    }
    sites
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
    let noentry: BTreeSet<String> = log.iter().filter(|l| l.starts_with("NOENTRY")).map(|l| l.trim_start_matches("NOENTRY|").to_string()).collect();
    Judged { violations, space: space.join(","), sites: sites_of(&log), noentry }
}

fn names(violations: &Violations) -> BTreeSet<&'static str> {
    violations.iter().map(|(i, _)| *i).collect()
}

fn same(left: &RetainedOperation, right: &RetainedOperation) -> bool {
    left.operation == right.operation && left.contents == right.contents
}

fn form(failures: &[FailureAfterAFault]) -> String {
    let parts: Vec<String> = failures
        .iter()
        .map(|failure| match failure {
            FailureAfterAFault::KnownRed(_) => "K".to_string(),
            FailureAfterAFault::LeftByTheLyingDevice { .. } => "L".to_string(),
            FailureAfterAFault::NewFinding { signature, .. } => format!("N{signature:?}"),
        })
        .collect();
    if parts.is_empty() { "-".to_string() } else { parts.join("+") }
}

/// 「仍红」按（不变量, 盘, 槽）认：有槽信息的不变量，被观察的那张盘上点名的槽在插回去那张盘上仍红才算仍红；没有槽信息的退回按名。
fn still_red_by_site(observed: &Judged, replug: &Judged) -> BTreeSet<&'static str> {
    let replug_names = names(&replug.violations);
    let mut still = BTreeSet::new();
    for name in names(&observed.violations) {
        let observed_sites: Vec<&(&'static str, u32, u64)> = observed.sites.iter().filter(|s| s.0 == name).collect();
        let red = if observed_sites.is_empty() {
            replug_names.contains(name)
        } else {
            observed_sites.iter().any(|s| replug.sites.contains(*s))
        };
        if red {
            still.insert(name);
        }
    }
    still
}

/// 录制流 `ops` 与被吞的写插回去之后的 `landed`：前者前 `n` 步对应后者的前几步（插在第 `n` 步之前的不算进前缀）。
fn landed_prefix_length(ops: &[RetainedOperation], landed: &[RetainedOperation], n: usize) -> usize {
    let (mut p, mut q, mut extras_before_n) = (0usize, 0usize, 0usize);
    while q < landed.len() {
        if p < ops.len() && same(&landed[q], &ops[p]) {
            p += 1;
            q += 1;
        } else {
            if p < n {
                extras_before_n += 1;
            }
            q += 1;
        }
    }
    n + extras_before_n
}

fn classify_split(
    images: &ImagesAfterAFault,
    first_red: &Option<(usize, Judged)>,
    after: &Judged,
    still_step: BTreeSet<&'static str>,
    still_after: BTreeSet<&'static str>,
) -> Vec<FailureAfterAFault> {
    let step_count = images.step_images_of_the_run_with_the_fault.len();
    let mut result = Vec::new();
    if let Some((index, judged)) = first_red {
        let mut steps: Vec<Violations> = vec![Vec::new(); step_count];
        steps[*index] = judged.violations.clone();
        result.extend(classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, steps, Vec::new(), still_step));
    }
    if !after.violations.is_empty() {
        let steps: Vec<Violations> = vec![Vec::new(); step_count];
        result.extend(classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, steps, after.violations.clone(), still_after));
    }
    result
}

fn handle(seed: u64, point: usize, images: &ImagesAfterAFault, quarantine_log: &[(u32, u64, u64, String)], leaks: &[(u64, u64, u64)], arms: &[String], out: &mut Vec<String>) {
    let ops = &images.image_after_the_fault.operations;
    let landed = images.image_with_the_swallowed_writes_landed.as_ref();
    let mut external_overlapping: BTreeSet<(u64, u64)> = BTreeSet::new();
    let mut reused = false;
    let mut swallowed_text = "none".to_string();
    if let Some(landed) = landed {
        let inserted = landed.operations.iter().zip(ops.iter()).position(|(l, r)| !same(l, r)).unwrap_or(ops.len());
        let swallowed = &landed.operations[inserted].operation;
        let (sw_dev, sw_start, sw_end) = (swallowed.device.0, swallowed.offset.0, swallowed.offset.0 + swallowed.length);
        reused = landed.operations[..inserted].iter().any(|op| {
            op.operation.device.0 == sw_dev && op.operation.length > 0 && op.operation.offset.0 < sw_end && sw_start < op.operation.offset.0 + op.operation.length
        });
        let overlaps_unit = |slot: u64, span: u64| sw_start < (slot + span) * SLOT && slot * SLOT < sw_end;
        external_overlapping = quarantine_log.iter().map(|(_, s, span, _)| (*s, *span)).filter(|(s, span)| overlaps_unit(*s, *span)).collect();
        swallowed_text = format!("dev{sw_dev}:slot{}:len{}", sw_start / SLOT, swallowed.length);
    }
    let flagged: Vec<(u32, u64, u64)> = quarantine_log.iter().map(|(d, s, span, _)| (*d, *s, *span)).collect();
    out.push(format!(
        "POINT seed={seed} point={point} fault={:?} swallowed={swallowed_text} reused={reused} flagged={flagged:?} overlapping={external_overlapping:?} leaks={leaks:?}",
        images.drawn.fault
    ));
    for arm in arms {
        defense_set_arm(arm);
        defense_set_external_quarantine(BTreeSet::new());
        let mut first_red: Option<(usize, Judged)> = None;
        for (index, step) in images.step_images_of_the_run_with_the_fault.iter().enumerate() {
            let judged = judge_image(&image_of(ops, step.operations_recorded));
            if !judged.violations.is_empty() {
                first_red = Some((index, judged));
                break;
            }
        }
        let after = judge_image(&image_of(ops, ops.len()));
        let (name, j2n, j2s, j2t, replug_space, j2_space);
        if let Some(landed) = landed {
            let plain = judge_image(&image_of(&landed.operations, landed.operations.len()));
            defense_set_external_quarantine(external_overlapping.clone());
            let j2 = judge_image(&image_of(&landed.operations, landed.operations.len()));
            defense_set_external_quarantine(BTreeSet::new());
            name = form(&classify_split(images, &first_red, &after, names(&plain.violations), names(&plain.violations)));
            j2n = form(&classify_split(images, &first_red, &after, names(&j2.violations), names(&j2.violations)));
            let still_step = first_red.as_ref().map(|(_, judged)| still_red_by_site(judged, &j2)).unwrap_or_default();
            j2s = form(&classify_split(images, &first_red, &after, still_step, still_red_by_site(&after, &j2)));
            let still_step_t = match &first_red {
                Some((index, judged)) => {
                    let n = images.step_images_of_the_run_with_the_fault[*index].operations_recorded;
                    let prefix = landed_prefix_length(ops, &landed.operations, n);
                    defense_set_external_quarantine(external_overlapping.clone());
                    let replug_step = judge_image(&image_of(&landed.operations, prefix));
                    defense_set_external_quarantine(BTreeSet::new());
                    still_red_by_site(judged, &replug_step)
                }
                None => BTreeSet::new(),
            };
            j2t = form(&classify_split(images, &first_red, &after, still_step_t, still_red_by_site(&after, &j2)));
            replug_space = plain.space.clone();
            j2_space = j2.space.clone();
        } else {
            name = form(&classify_split(images, &first_red, &after, BTreeSet::new(), BTreeSet::new()));
            j2n = name.clone();
            j2s = name.clone();
            j2t = name.clone();
            replug_space = "none".to_string();
            j2_space = "none".to_string();
        }
        if Some(arm) == arms.first() {
            for text in &after.noentry {
                out.push(format!("NOENT seed={seed} point={point} where=after {text}"));
            }
            if let Some((_, judged)) = &first_red {
                for text in &judged.noentry {
                    out.push(format!("NOENT seed={seed} point={point} where=first_red {text}"));
                }
            }
        }
        let first_red_names = first_red.as_ref().map(|(_, judged)| format!("{:?}", names(&judged.violations))).unwrap_or_else(|| "none".into());
        out.push(format!(
            "CLASS seed={seed} point={point} arm={arm} name={name} | j2n={j2n} | j2s={j2s} | j2t={j2t} | after_space={} replug_space={replug_space} j2_space={j2_space} first_red={first_red_names} after_names={:?}",
            after.space, names(&after.violations)
        ));
    }
    defense_set_arm("today");
}

#[test]
fn r3_probe() {
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let seed_count = env_u64("DEFENSE_SEEDS", 24);
    let seed_offset = env_u64("DEFENSE_SEED_OFFSET", 0);
    let steps = usize::try_from(env_u64("DEFENSE_STEPS", 20)).unwrap();
    let only_seed = std::env::var("DEFENSE_ONLY_SEED").ok().and_then(|v| v.parse::<u64>().ok());
    let arms: Vec<String> = std::env::var("DEFENSE_ARMS")
        .unwrap_or_else(|_| "today,exempt,gen,genplus,cmp,both,bothplus,jrn,jrngen".into())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
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
                let weights = match std::env::var("DEFENSE_WEIGHTS").as_deref() {
                    Ok("reuse") => GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
                    Ok("rollback") => GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
                    Ok("compaction") => GenerationWeights::COMPACTION_WHILE_MOUNTED,
                    Ok("compaction_rollback") => GenerationWeights::COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK,
                    Ok("broad_compaction") => GenerationWeights::BROAD_WITH_COMPACTION_WHILE_MOUNTED,
                    Ok("snapshot") => GenerationWeights::SNAPSHOT_OPERATIONS,
                    Ok("writes_by_inode") => GenerationWeights::WRITES_BY_INODE_NUMBER,
                    Ok("truncation") => GenerationWeights::TRUNCATION,
                    _ => GenerationWeights::BROAD,
                };
                let history = generate_history_with_weights(HistorySeed(*seed), steps, &weights);
                let _ = defense_take_quarantine_log();
                let _ = defense_take_leak_log();
                let mut handed: Vec<(ImagesAfterAFault, Vec<(u32, u64, u64, String)>, Vec<(u64, u64, u64)>)> = Vec::new();
                let _ = inject_faults_into_history_handing_out_the_images(&history, execution, 4, &mut |images| {
                    handed.push((images, defense_take_quarantine_log(), defense_take_leak_log()));
                });
                let mut out = Vec::new();
                for (point, (images, log, leaks)) in handed.iter().enumerate() {
                    if images.drawn.fault == InjectedFault::WriteIsSwallowed || std::env::var_os("DEFENSE_ALL_POINTS").is_some() {
                        handle(*seed, point, images, log, leaks, &arms, &mut out);
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
    let _ = BTreeMap::<u8, u8>::new();
}
