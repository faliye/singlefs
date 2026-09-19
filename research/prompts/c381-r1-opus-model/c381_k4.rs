//! K4：Z2-c 原历史在四条臂下（攻方副本）。
#[path = "c381_probe_common.rs"]
mod common;
use common::*;
use singlefs_harness::scenario::e142_parameters;

#[test]
fn k4_original_history_per_arm() {
    let parameters = e142_parameters(512, 512);
    let snapshot = prefix();
    let started = std::time::Instant::now();
    let base = run(&parameters, &snapshot, 0, None, &[], &[], None);
    println!("C 无故障：C 共 {} 次写；步骤 {:?}；耗时 {:?}", base.writes_after_c, base.steps, started.elapsed());
    // 原历史：C 的第 21 次写（下标 20，盘 1 的超级块槽）报错；E 从 B 发空发布，E 的记录在盘 0 那一次（下标 29）起断电。
    let histories: Vec<(&str, Vec<Action>, Option<u64>)> = vec![
        ("原历史：E 从 B", vec![Action::EmptyFromB], Some(29)),
        ("E 从调用方手里的现行版", vec![Action::Empty], Some(29)),
        ("对照：原样重发 C", vec![Action::Retry], None),
    ];
    for arm in 0..4u8 {
        for (label, actions, crash) in &histories {
            let outcome = run(&parameters, &snapshot, arm, Some(20), &[], actions, *crash);
            println!("[{}] {label}: steps={:?} writes={} crash_at={crash:?}\n    checker@crash={:?}\n    remount={}\n    checker@after={:?}", arm_name(arm), outcome.steps, outcome.writes, outcome.checker_at_crash, outcome.remount, outcome.checker_after_remount);
        }
    }
}
