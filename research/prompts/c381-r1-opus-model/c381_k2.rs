//! K2：四条臂 × C 在它 21 次写的每一次上报错（S1 = 下标 0–15 单元，S2 = 16–18 记录两盘 + 根槽，S3 = 19–20 超级块槽两盘），
//! 之后用户动作序列全放开（长度 0–3，五种动作），每个断电点各跑一次（攻方副本）。
//! 阶段 A：一次瞬时错（C）+ 断电。阶段 B：C 一次 + C 之后任一次写再一次瞬时错、不断电（序列长 0–3）。
//! 阶段 C：两次瞬时错 + 断电（序列长 0–1）。按「OVERWRITE = 重开读不出单元，或 I-2.1 / I-4.8 / I-7.2 / I-7.4 任一红」分类。
#[path = "c381_probe_common.rs"]
mod common;
use common::*;
use std::collections::BTreeMap;
use std::sync::Mutex;

fn sequences(max_len: usize) -> Vec<Vec<Action>> {
    let mut all = vec![vec![]];
    let mut frontier = vec![vec![]];
    for _ in 0..max_len {
        let mut next = Vec::new();
        for prefix in &frontier {
            for action in ACTIONS {
                let mut sequence: Vec<Action> = prefix.clone();
                sequence.push(action);
                next.push(sequence);
            }
        }
        all.extend(next.iter().cloned());
        frontier = next;
    }
    all
}

fn ids(list: &[String]) -> Vec<String> {
    list.iter().map(|line| line.split(':').next().unwrap_or("?").to_string()).collect()
}

fn classify(outcome: &Outcome) -> (&'static str, String) {
    let at_crash = ids(&outcome.checker_at_crash);
    let after = ids(&outcome.checker_after_remount);
    let overwrite_ids = ["I-2.1", "I-4.8", "I-7.2", "I-7.4"];
    let remount_kind = if outcome.remount.starts_with("ok") { "ok".to_string() } else { outcome.remount.chars().take(60).collect() };
    let signature = format!("remount={remount_kind} crash={at_crash:?} after={after:?}");
    let overwrite = outcome.remount.contains("UnitUnreadable")
        || at_crash.iter().chain(after.iter()).any(|id| overwrite_ids.contains(&id.as_str()));
    let class = if outcome.steps.iter().any(|step| step.contains("PANIC")) || outcome.remount == "PANIC" {
        "PANIC"
    } else if overwrite {
        "OVERWRITE"
    } else if !at_crash.is_empty() || !after.is_empty() {
        "OTHER_RED"
    } else if !outcome.remount.starts_with("ok") {
        "REFUSED_OTHER"
    } else {
        "CLEAN"
    };
    (class, signature)
}

fn stage(k: u64) -> &'static str {
    if k < 16 { "S1" } else if k < 19 { "S2" } else { "S3" }
}

#[derive(Default)]
struct Tally {
    runs: u64,
    by_class: BTreeMap<&'static str, u64>,
    examples: BTreeMap<(&'static str, String), (u64, u64, String, String)>,
}

fn phase(name: &str, max_len: usize, later_fault: bool, crash: bool) {
    let started = std::time::Instant::now();
    let sequences = sequences(max_len);
    let tallies: Mutex<BTreeMap<(u8, &'static str), Tally>> = Mutex::new(BTreeMap::new());
    let _ = sweep(&[0, 1, 2, 3], 21, |parameters, snapshot, arm, k| {
        let mut local: BTreeMap<(u8, &'static str), Tally> = BTreeMap::new();
        for actions in &sequences {
            let base = run(parameters, snapshot, arm, Some(k), &[], actions, None);
            let later_faults: Vec<Option<u64>> = if later_fault { (base.writes_after_c..base.writes).map(Some).collect() } else { vec![None] };
            for fault in later_faults {
                let faults: Vec<u64> = fault.into_iter().collect();
                let with_fault = if faults.is_empty() { base.clone() } else { run(parameters, snapshot, arm, Some(k), &faults, actions, None) };
                let crash_points: Vec<Option<u64>> = if crash { std::iter::once(None).chain((with_fault.writes_after_c..with_fault.writes).map(Some)).collect() } else { vec![None] };
                for crash_at in crash_points {
                    let outcome = if crash_at.is_none() { with_fault.clone() } else { run(parameters, snapshot, arm, Some(k), &faults, actions, crash_at) };
                    let (class, signature) = classify(&outcome);
                    let tally = local.entry((arm, stage(k))).or_default();
                    tally.runs += 1;
                    *tally.by_class.entry(class).or_default() += 1;
                    tally.examples.entry((class, signature)).or_insert_with(|| (k, crash_at.map_or(u64::MAX, |c| c), format!("actions={actions:?} later_fault={fault:?}"), format!("steps={:?} remount={}", outcome.steps, outcome.remount)));
                }
            }
        }
        let mut global = tallies.lock().expect("锁");
        for (key, tally) in local {
            let entry = global.entry(key).or_default();
            entry.runs += tally.runs;
            for (class, count) in tally.by_class {
                *entry.by_class.entry(class).or_default() += count;
            }
            for (key, example) in tally.examples {
                let keep = entry.examples.get(&key).is_none_or(|old| (example.0, example.1) < (old.0, old.1));
                if keep { entry.examples.insert(key, example); }
            }
        }
        Vec::new()
    });
    println!("===== {name}：序列长 0–{max_len}（{} 条），C 之后再一次瞬时错={later_fault}，断电={crash}；耗时 {:?}", sequences.len(), started.elapsed());
    for ((arm, stage), tally) in tallies.into_inner().expect("锁") {
        println!("[{} {stage}] runs={} {:?}", arm_name(arm), tally.runs, tally.by_class);
        for ((class, signature), (k, crash_at, actions, steps)) in &tally.examples {
            if *class != "CLEAN" {
                let crash_text = if *crash_at == u64::MAX { "none".to_string() } else { crash_at.to_string() };
                println!("    {class} {signature}\n        first: k={k} crash_at={crash_text} {actions} {steps}");
            }
        }
    }
}

#[test]
fn phase_a_one_fault_in_c_then_power_cut() {
    phase("阶段 A", 3, false, true);
}

#[test]
fn phase_b_two_faults_no_power_cut() {
    phase("阶段 B", 3, true, false);
}

#[test]
fn phase_c_two_faults_then_power_cut() {
    phase("阶段 C", 1, true, true);
}
