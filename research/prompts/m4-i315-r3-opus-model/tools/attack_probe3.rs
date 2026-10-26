//! m4-i315-r3 攻方腿草稿探针（不进仓，只在副本里跑）：故障注入 N 段 × 4 个注入点，CPU 池级 checker 就地判。
//! 「没有位置项可比」那一格四种判法（0 自证 = 今天、1 自证加代号核对、2 丁 两份互比、3 两者叠用）× 四种归类器
//! （今天按名比；甲二按名比；甲二按（盘, 槽）比，插回去那张盘取历史末尾那一张；甲二按（盘, 槽）比、插回去那张盘取第一张红的每步镜像那一刻），
//! 每个注入点各归一次。另记这一格被走到时自证、头里诞生代号、记录分配代、另一份同不同（两个推的盲区就在这几栏里认）。
//! 环境变量：ATTACK_SEEDS、ATTACK_SEED_OFFSET、ATTACK_STEPS、ATTACK_ONLY_SEED、ATTACK_ALL_POINTS、ATTACK_WEIGHTS
//! （broad / orphans / snapshot / compaction_rollback / broad_compaction）、SINGLEFS_THREAD_CAP；实现那一侧的 ATTACK_DEFECT、
//! ATTACK_SWALLOW_PARTNER 由 apply_r3_implementation.py 打进副本。
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{
    attack_set_arm, attack_set_external_quarantine, attack_take_culprits, attack_take_noloc_hits,
    check_pool_image,
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
const ARMS: [(&str, u8); 4] = [("selfcheck", 0), ("gen", 1), ("cmp", 2), ("both", 3)];
const SPACE: [&str; 3] = ["I-3.1", "I-3.11", "I-3.15"];
type Violations = Vec<(&'static str, String)>;
type Culprits = BTreeSet<(&'static str, u32, u64)>;

#[derive(Clone)]
struct Judged {
    violations: Violations,
    space: String,
    hits: Vec<String>,
    culprits: Culprits,
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn weights() -> &'static GenerationWeights {
    match std::env::var("ATTACK_WEIGHTS").as_deref() {
        Ok("orphans") => &GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP,
        Ok("snapshot") => &GenerationWeights::SNAPSHOT_OPERATIONS,
        Ok("compaction_rollback") => &GenerationWeights::COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK,
        Ok("broad_compaction") => &GenerationWeights::BROAD_WITH_COMPACTION_WHILE_MOUNTED,
        _ => &GenerationWeights::BROAD,
    }
}

fn image_of(operations: &[RetainedOperation], prefix: usize) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        HistoryDeviceWidth::FourGibibytes.device_bytes(),
    );
    pool.apply(&operations[..prefix]);
    pool
}

fn judge_once(image: &MemoryPool, arm: u8, external: &BTreeSet<(u64, u64)>) -> Judged {
    attack_set_arm(arm);
    attack_set_external_quarantine(external.clone());
    let _ = attack_take_noloc_hits();
    let _ = attack_take_culprits();
    let verdicts = check_pool_image(image);
    let hits = attack_take_noloc_hits();
    let culprits: Culprits = attack_take_culprits().into_iter().collect();
    attack_set_arm(0);
    attack_set_external_quarantine(BTreeSet::new());
    let mut space = Vec::new();
    let mut violations = Vec::new();
    for (invariant, verdict) in verdicts {
        let shape = match &verdict {
            InvariantVerdict::Holds => "成立",
            InvariantVerdict::Violated(_) => "违例",
            InvariantVerdict::NotApplicable(_) => "不适用",
            InvariantVerdict::PartiallyJudged(_) => "部分判",
        };
        if SPACE.contains(&invariant) {
            space.push(format!("{invariant}={shape}"));
        }
        if let InvariantVerdict::Violated(detail) = verdict {
            violations.push((invariant, detail));
        }
    }
    Judged { violations, space: space.join(","), hits, culprits }
}

/// 四种判法各判一次；这张盘上「没有位置项可比」那一格没有一份自证过时四种判法同判，只判一次（三个收严只在自证过之后多要求一样）。
fn judge_every_arm(image: &MemoryPool, external: &BTreeSet<(u64, u64)>) -> Vec<Judged> {
    let today = judge_once(image, 0, external);
    if !today.hits.iter().any(|hit| hit.contains("|self=true|")) {
        return vec![today.clone(), today.clone(), today.clone(), today];
    }
    let mut out = vec![today];
    for (_, arm) in &ARMS[1..] {
        out.push(judge_once(image, *arm, external));
    }
    out
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

/// 插回去那张盘的录制流里，哪几步是插进去的被吞的写（下标），各插在重开那张盘录制流的第几步之前。
fn inserted_writes(landed: &[RetainedOperation], ops: &[RetainedOperation]) -> Vec<(usize, usize)> {
    let mut inserted = Vec::new();
    let mut j = 0;
    for (i, operation) in landed.iter().enumerate() {
        if j < ops.len() && same(operation, &ops[j]) {
            j += 1;
        } else {
            inserted.push((i, j));
        }
    }
    inserted
}

/// 一条失败（违例连同违例点名的槽）在一张对照盘上「仍红」的不变量：I-3.1 / I-3.11 / I-3.15 按（盘, 槽）比——这一条点名的槽里有一个
/// 在对照盘上同一条不变量下仍被点名（点名的槽为空时退回按名比）；别的不变量按名比。
fn still_red_by_slot(observation: &Judged, comparison: &Judged) -> BTreeSet<&'static str> {
    let red_in_comparison = names(&comparison.violations);
    names(&observation.violations)
        .into_iter()
        .filter(|invariant| {
            if !red_in_comparison.contains(invariant) {
                return false;
            }
            if !SPACE.contains(invariant) {
                return true;
            }
            let named: Vec<(u32, u64)> = observation
                .culprits
                .iter()
                .filter(|(tag, _, _)| tag == invariant)
                .map(|(_, device, slot)| (*device, *slot))
                .collect();
            named.is_empty()
                || named.iter().any(|(device, slot)| comparison.culprits.contains(&(*invariant, *device, *slot)))
        })
        .collect()
}

/// 这一格被走到的记录里自证过的那几份按两个推的盲区分：代号相等而两份不同（自证加代号核对认、丁不认）、两份相同而代号不等
/// （丁认、代号核对不认）、两份相同且代号相等（四种判法都认）。
fn hit_summary(hits: &[String]) -> String {
    let unique: BTreeSet<&String> = hits.iter().filter(|h| !h.starts_with("LOC|")).collect();
    let passes: Vec<&&String> = unique.iter().filter(|h| h.contains("|self=true|")).collect();
    let generation_equal = |h: &str| {
        let birth = h.split("|birth=").nth(1).and_then(|r| r.split('|').next()).unwrap_or("");
        let generation = h.split("|gen=").nth(1).and_then(|r| r.split('|').next()).unwrap_or("x");
        birth.starts_with("Some") && birth == generation
    };
    let other_same = |h: &str| h.contains("|other_same=true");
    // 按单元（哪一处、槽、跨度）归组：一个单元每一份都自证过且代号相等时代号核对判它对得上，每一份都自证过且与另一份相同时丁判它对得上。
    let unit_of = |h: &str| {
        let fields: Vec<&str> = h.split('|').collect();
        format!("{}|{}|{}", fields[0], fields.get(2).unwrap_or(&""), fields.get(3).unwrap_or(&""))
    };
    let mut units: BTreeMap<String, (bool, bool)> = BTreeMap::new();
    for h in &unique {
        let entry = units.entry(unit_of(h)).or_insert((true, true));
        let passes_self = h.contains("|self=true|");
        entry.0 &= passes_self && generation_equal(h);
        entry.1 &= passes_self && other_same(h);
    }
    let gen_only = units.values().filter(|(g, c)| *g && !*c).count();
    let cmp_only = units.values().filter(|(g, c)| !*g && *c).count();
    let both = units.values().filter(|(g, c)| *g && *c).count();
    format!("{}/{} genEq_otherDiff={gen_only} genNe_otherSame={cmp_only} genEq_otherSame={both}", passes.len(), unique.len())
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
    let mut swallowed_text = Vec::new();
    let mut external: BTreeSet<(u64, u64)> = BTreeSet::new();
    let mut inserted = Vec::new();
    if let Some(landed) = landed {
        inserted = inserted_writes(&landed.operations, ops);
        for (index, _) in &inserted {
            let swallowed = &landed.operations[*index].operation;
            let (dev, start, end) = (swallowed.device.0, swallowed.offset.0, swallowed.offset.0 + swallowed.length);
            let reused = landed.operations[..*index].iter().any(|op| {
                op.operation.device.0 == dev && op.operation.length > 0 && op.operation.offset.0 < end && start < op.operation.offset.0 + op.operation.length
            });
            external.extend(
                quarantine_log
                    .iter()
                    .filter(|(d, s, span, _)| *d == dev && start < (s + span) * SLOT && s * SLOT < end)
                    .map(|(_, s, span, _)| (*s, *span)),
            );
            swallowed_text.push(format!("dev{dev}:slot{}:len{}:reused={reused}", start / SLOT, swallowed.length));
        }
    }
    out.push(format!(
        "POINT seed={seed} point={point} fault={} swallowed={swallowed_text:?} flagged={quarantine_log:?} leaks={leak_log:?} external={external:?}",
        images.drawn.fault.name()
    ));
    let none = BTreeSet::new();
    let steps: Vec<Vec<Judged>> = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .map(|step| judge_every_arm(&image_of(ops, step.operations_recorded), &none))
        .collect();
    let after = judge_every_arm(&image_of(ops, ops.len()), &none);
    out.push(format!("HITS seed={seed} point={point} after={}", hit_summary(&after[0].hits)));
    // 位置项还在、这一份对不上的隔离单元：对不上的那一份自证过而头里诞生代号恰等于记录分配代（代号核对推的盲区），每步镜像与重开那张盘一起数。
    let located: BTreeSet<String> = steps
        .iter()
        .map(|s| &s[0])
        .chain(std::iter::once(&after[0]))
        .flat_map(|j| j.hits.iter())
        .filter(|h| h.starts_with("LOC|") && h.contains("|loc_self=true|"))
        .cloned()
        .collect();
    let equal_birth: Vec<&String> = located
        .iter()
        .filter(|h| {
            let birth = h.split("|birth=").nth(1).and_then(|r| r.split('|').next()).unwrap_or("");
            let generation = h.split("|gen=").nth(1).unwrap_or("x");
            birth.starts_with("Some") && birth == generation
        })
        .collect();
    out.push(format!("LOCS seed={seed} point={point} loc_self_pass={} equal_birth={}", located.len(), equal_birth.len()));
    for hit in equal_birth.iter().take(4) {
        out.push(format!("LOCEQ seed={seed} point={point} {hit}"));
    }
    let passing: BTreeSet<&String> = after[0].hits.iter().filter(|h| h.contains("|self=true|")).collect();
    for hit in passing.iter().take(6) {
        out.push(format!("HIT seed={seed} point={point} {hit}"));
    }
    let landed_judged = landed.map(|landed| {
        let image = image_of(&landed.operations, landed.operations.len());
        (judge_every_arm(&image, &none), judge_every_arm(&image, &external))
    });
    for (arm_index, (arm, _)) in ARMS.iter().enumerate() {
        let step_violations: Vec<Violations> = steps.iter().map(|s| s[arm_index].violations.clone()).collect();
        let after_violations = after[arm_index].violations.clone();
        let red_steps: Vec<String> = steps
            .iter()
            .enumerate()
            .filter(|(_, s)| s[arm_index].violations.iter().any(|(i, _)| i.starts_with("I-3.")))
            .map(|(index, s)| format!("{index}:{:?}", names(&s[arm_index].violations).into_iter().filter(|i| i.starts_with("I-3.")).collect::<Vec<_>>()))
            .collect();
        let Some((replug, replug_external)) = &landed_judged else {
            let classified = form(&classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations, after_violations.clone(), BTreeSet::new()));
            out.push(format!(
                "CLASS seed={seed} point={point} arm={arm} after_space={} after_names={:?} steps_with_I3={red_steps:?} today={classified} | jia2n={classified} | jia2s={classified} | jia2k={classified}",
                after[arm_index].space, names(&after_violations)
            ));
            continue;
        };
        let landed = landed.expect("有插回去的判定就有插回去那张盘");
        let today = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations.clone(), after_violations.clone(), names(&replug[arm_index].violations));
        let jia2n = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations.clone(), after_violations.clone(), names(&replug_external[arm_index].violations));
        let first_red = step_violations.iter().position(|v| !v.is_empty());
        let empty_steps: Vec<Violations> = vec![Vec::new(); step_violations.len()];
        let mut jia2s = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(
            images,
            step_violations.clone(),
            Vec::new(),
            first_red.map(|k| still_red_by_slot(&steps[k][arm_index], &replug_external[arm_index])).unwrap_or_default(),
        );
        jia2s.extend(classify_the_pool_checker_verdicts_of_the_images_after_a_fault(
            images,
            empty_steps.clone(),
            after_violations.clone(),
            still_red_by_slot(&after[arm_index], &replug_external[arm_index]),
        ));
        let step_still_red = first_red.map(|k| {
            let recorded = images.step_images_of_the_run_with_the_fault[k].operations_recorded;
            let prefix = recorded + inserted.iter().filter(|(_, before)| *before <= recorded).count();
            let replug_of_the_step = judge_every_arm(&image_of(&landed.operations, prefix), &external);
            still_red_by_slot(&steps[k][arm_index], &replug_of_the_step[arm_index])
        });
        let mut jia2k = classify_the_pool_checker_verdicts_of_the_images_after_a_fault(images, step_violations.clone(), Vec::new(), step_still_red.unwrap_or_default());
        jia2k.extend(classify_the_pool_checker_verdicts_of_the_images_after_a_fault(
            images,
            empty_steps,
            after_violations.clone(),
            still_red_by_slot(&after[arm_index], &replug_external[arm_index]),
        ));
        out.push(format!(
            "CLASS seed={seed} point={point} arm={arm} after_space={} after_names={:?} replug_space={} replugx_space={} steps_with_I3={red_steps:?} after_culprits={:?} today={} | jia2n={} | jia2s={} | jia2k={}",
            after[arm_index].space,
            names(&after_violations),
            replug[arm_index].space,
            replug_external[arm_index].space,
            after[arm_index].culprits,
            form(&today),
            form(&jia2n),
            form(&jia2s),
            form(&jia2k)
        ));
    }
}

#[test]
fn attack_probe3() {
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
    let lines: Mutex<BTreeMap<u64, Vec<String>>> = Mutex::new(BTreeMap::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::SeqCst);
                let Some(seed) = seeds.get(index) else { break };
                let history = generate_history_with_weights(HistorySeed(*seed), steps, weights());
                let _ = attack_take_quarantine_log();
                let _ = attack_take_leak_log();
                // 每个注入点交出来就地判完再丢，不把四个注入点的镜像攒在一起（120 步的历史攒四份在 31 个线程上撞 8G 上限）。
                let mut out = Vec::new();
                let mut point = 0usize;
                let _ = inject_faults_into_history_handing_out_the_images(&history, execution, 4, &mut |images| {
                    let log = attack_take_quarantine_log();
                    let leaks = attack_take_leak_log();
                    if images.drawn.fault == InjectedFault::WriteIsSwallowed || all_points {
                        handle(*seed, point, &images, &log, &leaks, &mut out);
                    }
                    point += 1;
                });
                out.push(format!("DONE seed={seed}"));
                lines.lock().unwrap().insert(*seed, out);
            });
        }
    });
    for (_, out) in lines.into_inner().unwrap() {
        for line in out {
            println!("{line}");
        }
    }
}
