//! c381-r2 攻方副本：坏扇区（落点持续写不进去、设备整体还活着）——探针写会把它判成「瞬时」。
#[path = "c381r2_probe_common.rs"]
mod common;
use common::*;
use singlefs_harness::scenario::e142_parameters;

fn snapshots() -> Vec<Snapshot> {
    let base = base_snapshot();
    let after_mount = remounted(&base, "remounted");
    let many = plus_overwrites(&after_mount, 5, "many_overwrites");
    vec![
        base.clone(),
        plus_empty_publishes(&base, 1, "txg6"),
        after_mount.clone(),
        rolled_back(&base, "rolled_back"),
        floor_raised(&many, "floor_raised"),
    ]
}

#[test]
#[ignore = "手动跑"]
fn bad_sector_under_each_arm() {
    let parameters = e142_parameters(512, 512);
    for snapshot in snapshots() {
        let log = writes_of_c(&parameters, &snapshot);
        println!("--- 前缀 {}：{}（C 共 {} 次写）", snapshot.label, snapshot.note, log.len());
        // 第 0 次写 = 第一个单元的盘 0 那一份；第 16 次 = 记录盘 0；第 18 次 = 根槽。
        for index in [0usize, 16, 18] {
            let (device, offset, length) = log[index];
            for arm in [0u8, 2, 5, 6] {
                let spec = Spec {
                    bad_ranges: vec![(device, offset, offset + length as u64)],
                    switch_selects_base_root: arm == 6,
                    actions: if arm == 0 || arm == 2 { vec![Action::Remount, Action::Retry] } else { vec![] },
                    ..Spec::default()
                };
                let outcome = run(&parameters, &snapshot, arm, &spec);
                println!(
                    "  [坏扇区在 C 的第{index}次写：盘{device} 偏移{offset} 长{length}] [{}] 分类={} 切换={} 只读={} 实例={} 探针={} 写={} 实例表行数={:?}\n      steps={:?}\n      remount={} after红={:?} 内容={}",
                    arm_name(arm),
                    classify(&outcome),
                    outcome.switches,
                    outcome.read_only,
                    outcome.final_instance,
                    outcome.probe_writes,
                    outcome.writes,
                    outcome.instance_rows,
                    outcome.steps,
                    outcome.remount,
                    outcome.checker_after_remount,
                    outcome.acknowledged_content_after_remount
                );
            }
        }
    }
}
