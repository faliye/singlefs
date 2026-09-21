//! c381-r2 攻方副本：前缀与臂的自检——每段前缀在无故障下跑一次 C，三条臂都该干净。
#[path = "c381r2_probe_common.rs"]
mod common;
use common::*;
use singlefs_harness::scenario::e142_parameters;

#[test]
fn smoke_prefixes_and_arms() {
    let parameters = e142_parameters(512, 512);
    let base = base_snapshot();
    let txg6 = plus_empty_publishes(&base, 1, "txg6");
    let txg7 = plus_empty_publishes(&base, 2, "txg7");
    let after_mount = remounted(&base, "remounted");
    let after_rollback = rolled_back(&base, "rolled_back");
    let many = plus_overwrites(&after_mount, 5, "many_overwrites");
    let raised = floor_raised(&many, "floor_raised");
    for snapshot in [&base, &txg6, &txg7, &after_mount, &after_rollback, &many, &raised] {
        println!("[前缀 {}] {}", snapshot.label, snapshot.note);
        for arm in [0u8, 2, 5, 6] {
            let outcome = run(&parameters, snapshot, arm, &Spec::default());
            println!(
                "  [{}] 无故障：C 共 {} 次写，steps={:?}，remount={}，分类={}，crash红={:?}，after红={:?}",
                arm_name(arm),
                outcome.writes_after_c,
                outcome.steps,
                outcome.remount,
                classify(&outcome),
                outcome.checker_at_crash,
                outcome.checker_after_remount
            );
        }
    }
}
