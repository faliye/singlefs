//! c381-r2 攻方副本：分配记录树将满的池上，丙与戊各烧掉几个实例代号（收口表第 20a 行那一格 × 失败表）。
#[path = "c381r2_probe_common.rs"]
mod common;
use common::*;
use singlefs_harness::scenario::e142_parameters;

/// 先找出「再挂载一次就被准入挡住」的覆盖写次数。
#[test]
#[ignore = "手动跑"]
fn how_many_overwrites_until_the_mount_is_refused() {
    let base = base_snapshot();
    let parameters = e142_parameters(512, 512);
    for count in [45u64, 46, 47, 48] {
        let snapshot = plus_overwrites(&base, count, "full");
        let outcome = run(&parameters, &snapshot, 0, &Spec { actions: vec![Action::Remount], ..Spec::default() });
        println!("{count} 次覆盖写之后：steps={:?} remount={}", outcome.steps, outcome.remount);
    }
}

/// 池快满时 C 的一次写报错：丙与戊各走到哪，烧掉几个实例代号。
#[test]
#[ignore = "手动跑"]
fn arms_on_a_nearly_full_pool() {
    let base = base_snapshot();
    let parameters = e142_parameters(512, 512);
    for count in [44u64, 45, 46, 47] {
        let snapshot = plus_overwrites(&base, count, "nearly_full");
        println!("--- {count} 次覆盖写：{}", snapshot.note);
        for arm in [0u8, 2, 5, 6] {
            for fault in [16u64, 18, 19] {
                let spec = Spec {
                    faults: vec![(fault, Fault::Transient)],
                    switch_selects_base_root: arm == 6,
                    ..Spec::default()
                };
                let outcome = run(&parameters, &snapshot, arm, &spec);
                println!(
                    "  [{}] C 第{fault}次写瞬时错：分类={} 切换={} 只读={} 实例={} steps={:?} remount={} after红={:?}",
                    arm_name(arm),
                    classify(&outcome),
                    outcome.switches,
                    outcome.read_only,
                    outcome.final_instance,
                    outcome.steps,
                    outcome.remount,
                    outcome.checker_after_remount.len()
                );
            }
        }
    }
}
