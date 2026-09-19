//! K2 续（攻方副本）。阶段 D：C 无故障，之后第一步是进程内重开（可写挂载：取号 = S4、写行、暖机），在重开之后的任一次写上
//! 一次瞬时错，再接长 0–2 的动作序列，每个断电点各跑一次。阶段 E：同阶段 A，但 C 那次瞬时错是「写已落盘、设备照样报错」。
#[path = "c381_probe_common.rs"]
mod common;
use common::*;
use singlefs_harness::scenario::e142_parameters;
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
    let overwrite = outcome.remount.contains("UnitUnreadable") || at_crash.iter().chain(after.iter()).any(|id| overwrite_ids.contains(&id.as_str()));
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

#[derive(Default)]
struct Tally {
    runs: u64,
    by_class: BTreeMap<&'static str, u64>,
    examples: BTreeMap<(&'static str, String), (u64, u64, String, String)>,
}

fn print(name: &str, tallies: BTreeMap<(u8, String), Tally>) {
    println!("===== {name}");
    for ((arm, stage), tally) in tallies {
        println!("[{} {stage}] runs={} {:?}", arm_name(arm), tally.runs, tally.by_class);
        for ((class, signature), (k, crash_at, actions, steps)) in &tally.examples {
            if *class != "CLEAN" {
                let crash_text = if *crash_at == u64::MAX { "none".to_string() } else { crash_at.to_string() };
                println!("    {class} {signature}\n        first: fault={k} crash_at={crash_text} {actions} {steps}");
            }
        }
    }
}

fn parallel<T: Send + Sync + Copy>(work: &[T], body: impl Fn(T) -> Vec<((u8, String), &'static str, String, (u64, u64, String, String))> + Sync) -> BTreeMap<(u8, String), Tally> {
    let tallies: Mutex<BTreeMap<(u8, String), Tally>> = Mutex::new(BTreeMap::new());
    let next = Mutex::new(0usize);
    std::thread::scope(|scope| {
        for _ in 0..24 {
            scope.spawn(|| loop {
                let item = {
                    let mut guard = next.lock().expect("锁");
                    let item = work.get(*guard).copied();
                    *guard += 1;
                    item
                };
                let Some(item) = item else { break };
                let rows = body(item);
                let mut global = tallies.lock().expect("锁");
                for (key, class, signature, example) in rows {
                    let tally = global.entry(key).or_default();
                    tally.runs += 1;
                    *tally.by_class.entry(class).or_default() += 1;
                    let keep = tally.examples.get(&(class, signature.clone())).is_none_or(|old| (example.0, example.1) < (old.0, old.1));
                    if keep { tally.examples.insert((class, signature), example); }
                }
            });
        }
    });
    tallies.into_inner().expect("锁")
}

#[test]
fn phase_d_fault_inside_the_remount_then_power_cut() {
    let started = std::time::Instant::now();
    let parameters = e142_parameters(512, 512);
    let snapshot = prefix();
    let tails = sequences(2);
    // C 无故障 21 次写；重开的写从下标 21 起，数一次重开发几次写。
    let probe = run(&parameters, &snapshot, 0, None, &[], &[Action::Remount], None);
    let mount_writes = probe.writes - probe.writes_after_c;
    println!("重开（C 之后第一次可写挂载）共 {mount_writes} 次写，步骤 {:?}", probe.steps);
    let work: Vec<(u8, u64)> = (0..4u8).flat_map(|arm| (0..mount_writes).map(move |j| (arm, probe.writes_after_c + j))).collect();
    let tallies = parallel(&work, |(arm, fault)| {
        let mut rows = Vec::new();
        for tail in &tails {
            let mut actions = vec![Action::Remount];
            actions.extend(tail.iter().copied());
            let base = run(&parameters, &snapshot, arm, None, &[fault], &actions, None);
            for crash_at in std::iter::once(None).chain((fault + 1..base.writes).map(Some)) {
                let outcome = if crash_at.is_none() { base.clone() } else { run(&parameters, &snapshot, arm, None, &[fault], &actions, crash_at) };
                let (class, signature) = classify(&outcome);
                let relative = fault - probe.writes_after_c;
                let segment = if relative < 2 { "S4 取号" } else { "写行 / 暖机" };
                rows.push(((arm, segment.to_string()), class, signature, (relative, crash_at.map_or(u64::MAX, |c| c), format!("actions={actions:?}"), format!("steps={:?} remount={}", outcome.steps, outcome.remount))));
            }
        }
        rows
    });
    print(&format!("阶段 D：重开里第 j 次写瞬时错（j 相对重开起点）+ 序列长 0–2 + 断电；耗时 {:?}", started.elapsed()), tallies);
}

#[test]
fn phase_e_lying_fault_in_c_then_power_cut() {
    let started = std::time::Instant::now();
    let parameters = e142_parameters(512, 512);
    let snapshot = prefix();
    let sequences = sequences(3);
    let work: Vec<(u8, u64)> = (0..4u8).flat_map(|arm| (0..21u64).map(move |k| (arm, k))).collect();
    let tallies = parallel(&work, |(arm, k)| {
        LYING.with(|flag| flag.set(true));
        let mut rows = Vec::new();
        for actions in &sequences {
            let base = run(&parameters, &snapshot, arm, Some(k), &[], actions, None);
            for crash_at in std::iter::once(None).chain((base.writes_after_c..base.writes).map(Some)) {
                let outcome = if crash_at.is_none() { base.clone() } else { run(&parameters, &snapshot, arm, Some(k), &[], actions, crash_at) };
                let (class, signature) = classify(&outcome);
                let stage = if k < 16 { "S1" } else if k < 19 { "S2" } else { "S3" };
                rows.push(((arm, format!("{stage}（报错但已落盘）")), class, signature, (k, crash_at.map_or(u64::MAX, |c| c), format!("actions={actions:?}"), format!("steps={:?} remount={}", outcome.steps, outcome.remount))));
            }
        }
        rows
    });
    print(&format!("阶段 E：C 的第 k 次写「落盘但报错」+ 序列长 0–3 + 断电；耗时 {:?}", started.elapsed()), tallies);
}
