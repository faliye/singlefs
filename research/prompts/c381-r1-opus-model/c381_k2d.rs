//! 核对：乙′ 与甲在 C 的第 16、17 次写上报错时，动作序列里**不带** `Remount` 的段，断电那一刻（或序列走完）的镜像上 I-3.1 红几段（攻方副本）。
#[path = "c381_probe_common.rs"]
mod common;
use common::*;
use singlefs_harness::scenario::e142_parameters;

#[test]
fn b_prime_without_remount() {
    let parameters = e142_parameters(512, 512);
    let snapshot = prefix();
    let mut sequences: Vec<Vec<Action>> = vec![vec![]];
    for _ in 0..3 {
        let longer: Vec<Vec<Action>> = sequences.iter().filter(|s| s.len() == sequences.last().map_or(0, Vec::len)).flat_map(|s| ACTIONS.iter().filter(|a| **a != Action::Remount).map(move |a| { let mut t = s.clone(); t.push(*a); t })).collect();
        sequences.extend(longer);
    }
    println!("不带 Remount 的序列 {} 条", sequences.len());
    for arm in [0u8, 1, 4] {
        for k in [16u64, 17, 18] {
            let (mut runs, mut red_at_crash) = (0u64, 0u64);
            let mut example = String::new();
            for actions in &sequences {
                let base = run(&parameters, &snapshot, arm, Some(k), &[], actions, None);
                for crash_at in std::iter::once(None).chain((base.writes_after_c..base.writes).map(Some)) {
                    let outcome = if crash_at.is_none() { base.clone() } else { run(&parameters, &snapshot, arm, Some(k), &[], actions, crash_at) };
                    runs += 1;
                    if !outcome.checker_at_crash.is_empty() {
                        red_at_crash += 1;
                        if example.is_empty() { example = format!("crash_at={crash_at:?} actions={actions:?} steps={:?} {:?}", outcome.steps, outcome.checker_at_crash); }
                    }
                }
            }
            eprintln!("progress arm {} k {k}", arm_name(arm));
            println!("[{} k={k}] 不带重开的段 {runs}，断电那一刻已红 {red_at_crash}；第一例 {example}", arm_name(arm));
        }
    }
}
