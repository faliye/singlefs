//! 乙′（攻方提的改法，只在攻方副本上量过、被攻过零轮）：阶段 A 的 S2 三格（C 的第 16–18 次写报错）+ 序列长 0–3 + 断电，
//! 与乙并排；另跑「报错但已落盘」那一版（阶段 E 的同一片）。
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

#[test]
fn s2_arms_b_and_b_prime() {
    let parameters = e142_parameters(512, 512);
    let snapshot = prefix();
    let sequences = sequences(3);
    let work: Vec<(u8, u64, bool)> = [1u8, 4].iter().flat_map(|arm| (16..19u64).flat_map(move |k| [false, true].map(|lying| (*arm, k, lying)))).collect();
    let tallies: Mutex<BTreeMap<(u8, u64, bool), BTreeMap<String, (u64, String)>>> = Mutex::new(BTreeMap::new());
    let next = Mutex::new(0usize);
    let started = std::time::Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..12 {
            scope.spawn(|| loop {
                let item = { let mut guard = next.lock().expect("锁"); let item = work.get(*guard).copied(); *guard += 1; item };
                let Some((arm, k, lying)) = item else { break };
                LYING.with(|flag| flag.set(lying));
                let mut local: BTreeMap<String, (u64, String)> = BTreeMap::new();
                for actions in &sequences {
                    let base = run(&parameters, &snapshot, arm, Some(k), &[], actions, None);
                    for crash_at in std::iter::once(None).chain((base.writes_after_c..base.writes).map(Some)) {
                        let outcome = if crash_at.is_none() { base.clone() } else { run(&parameters, &snapshot, arm, Some(k), &[], actions, crash_at) };
                        let ids = |list: &[String]| list.iter().map(|line| line.split(':').next().unwrap_or("?").to_string()).collect::<Vec<_>>();
                        let key = format!("remount={} crash={:?} after={:?}", if outcome.remount.starts_with("ok") { "ok".to_string() } else { outcome.remount.chars().take(60).collect() }, ids(&outcome.checker_at_crash), ids(&outcome.checker_after_remount));
                        let entry = local.entry(key).or_insert((0, format!("crash_at={crash_at:?} actions={actions:?} steps={:?}", outcome.steps)));
                        entry.0 += 1;
                    }
                }
                eprintln!("progress arm {} k {k} lying {lying} at {:?}", arm_name(arm), started.elapsed());
                tallies.lock().expect("锁").insert((arm, k, lying), local);
            });
        }
    });
    for ((arm, k, lying), rows) in tallies.into_inner().expect("锁") {
        let total: u64 = rows.values().map(|(count, _)| count).sum();
        println!("[{} k={k} 报错但已落盘={lying}] runs={total}", arm_name(arm));
        for (key, (count, example)) in rows {
            println!("    {count:>6} {key}\n           first: {example}");
        }
    }
}
