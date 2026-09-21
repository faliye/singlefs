//! c381-r3 云端攻方腿（Opus）K3：候选乙「探针写改成写这次失败的那个落点」的落点安全性与定义域。
//! 三条臂只差探针写的落点：5 = 戊-A（今天的字面，固定落点）、7 = 戊-乙a（失败的那个落点，写探针图样）、
//! 8 = 戊-乙b（失败的那个落点，把那次写的原样字节再写一遍）。切换、N_switch、转只读三者逐字相同。
#[path = "c381r3_probe_common.rs"]
mod common;
use common::*;
use singlefs_harness::scenario::e142_parameters;

const ARMS: [u8; 3] = [5, 7, 8];

fn mode_of(arm: u8) -> ProbeMode {
    match arm {
        7 => ProbeMode::FailedLanding,
        8 => ProbeMode::FailedLandingRetry,
        _ => ProbeMode::Fixed,
    }
}

fn snapshots() -> Vec<(Snapshot, Option<(u32, u64)>)> {
    let base = base_snapshot();
    let wrapped = plus_empty_publishes(&base, 22, "wrapped");
    let after_mount = remounted(&base, "remounted");
    vec![
        (base.clone(), Some((1, 3))),
        (wrapped, Some((1, 3))),
        (after_mount, Some((1, 3))),
    ]
}

fn line(prefix: &str, what: &str, arm: u8, outcome: &Outcome) -> String {
    format!(
        "{prefix}\t{what}\t{}\t{}\t切换={} 只读={} 探针={} 没定义={}\t抹掉={:?}\t落点={:?}\tremount={}\tafter红={:?}\t内容={}\t回退={}",
        arm_name(arm),
        classify(outcome),
        outcome.switches,
        outcome.read_only,
        outcome.probe_writes,
        outcome.probe_undefined,
        outcome.destroyed,
        outcome.probe_notes,
        outcome.remount,
        outcome.checker_after_remount,
        outcome.acknowledged_content_after_remount,
        outcome.rollback_result
    )
}

#[test]
#[ignore = "手动跑"]
fn landing_point_of_the_failed_write() {
    let parameters = e142_parameters(512, 512);
    for (snapshot, rollback) in snapshots() {
        let log = writes_of_c(&parameters, &snapshot);
        println!(
            "=== 前缀 {}：{}（C 共 {} 次写）",
            snapshot.label,
            snapshot.note,
            log.len()
        );
        for (index, (device, offset, length)) in log.iter().enumerate() {
            println!(
                "  C 的第 {index} 次写：盘{device} 偏移{offset} 长{length}（{}）",
                region_name(*offset)
            );
        }
        for (index, _) in log.iter().enumerate() {
            for fault in [Fault::Transient, Fault::Lying] {
                for arm in ARMS {
                    let spec = Spec {
                        faults: vec![(index as u64, fault)],
                        probe_mode: mode_of(arm),
                        rollback_probe: rollback,
                        ..Spec::default()
                    };
                    let outcome = run(&parameters, &snapshot, arm, &spec);
                    println!(
                        "{}",
                        line(
                            snapshot.label,
                            &format!("写#{index}({fault:?},{})", region_name(log[index].1)),
                            arm,
                            &outcome
                        )
                    );
                }
            }
        }
        // 屏障报错：`barrier()` 不带偏移，候选乙在这一格没有落点。
        for barrier in 0u64..2 {
            for arm in ARMS {
                let spec = Spec {
                    barrier_fault: Some(barrier),
                    probe_mode: mode_of(arm),
                    rollback_probe: rollback,
                    ..Spec::default()
                };
                let outcome = run(&parameters, &snapshot, arm, &spec);
                println!(
                    "{}",
                    line(snapshot.label, &format!("屏障#{barrier}"), arm, &outcome)
                );
            }
        }
    }
}

/// 落点持续坏（r2 K1 打中三那一类）：候选乙修不修得掉；修掉的代价是什么。
#[test]
#[ignore = "手动跑"]
fn bad_sector_under_three_probes() {
    let parameters = e142_parameters(512, 512);
    for (snapshot, rollback) in snapshots() {
        let log = writes_of_c(&parameters, &snapshot);
        println!("=== 前缀 {}：{}", snapshot.label, snapshot.note);
        for index in [0usize, 16, 18, 19] {
            let (device, offset, length) = log[index];
            for arm in ARMS {
                let spec = Spec {
                    bad_ranges: vec![(device, offset, offset + length as u64)],
                    probe_mode: mode_of(arm),
                    rollback_probe: rollback,
                    ..Spec::default()
                };
                let outcome = run(&parameters, &snapshot, arm, &spec);
                println!(
                    "{}",
                    line(
                        snapshot.label,
                        &format!("坏扇区@写#{index}({})", region_name(offset)),
                        arm,
                        &outcome
                    )
                );
            }
        }
        // 「写落了盘、设备照样报错」的持续形：同一个落点每次写都落盘、每次都报错。
        for index in [16usize, 18, 19] {
            let (_, offset, _) = log[index];
            for arm in ARMS {
                let spec = Spec {
                    faults: (0..40).map(|step| (step, Fault::Lying)).collect(),
                    probe_mode: mode_of(arm),
                    rollback_probe: rollback,
                    ..Spec::default()
                };
                let outcome = run(&parameters, &snapshot, arm, &spec);
                println!(
                    "{}",
                    line(
                        snapshot.label,
                        &format!("整段 Lying（起点写#{index}，{}）", region_name(offset)),
                        arm,
                        &outcome
                    )
                );
            }
            break;
        }
    }
}

/// 「写落了盘、设备照样报错」的持续形（今天的 `ForceUnitAccess` = write_all_at + sync_data，刷失败就是这一形）：
/// 固定结构的两段——系统配置槽（每盘 8 KiB）与根环区域 0（1 MiB 起 32 KiB）——每次写都落盘、每次都报错。
#[test]
#[ignore = "手动跑"]
fn lying_range_on_fixed_structures() {
    let parameters = e142_parameters(512, 512);
    for (snapshot, rollback) in snapshots() {
        println!("=== 前缀 {}：{}", snapshot.label, snapshot.note);
        for (what, range) in [
            ("系统配置槽两份(盘0)", (0u32, 0u64, 8192u64)),
            ("根环区域0(盘0)", (0u32, 1_048_576u64, 1_048_576u64 + 32768)),
            ("根环区域2(盘0)", (0u32, 7_340_032u64, 7_340_032u64 + 32768)),
        ] {
            for arm in ARMS {
                let spec = Spec {
                    lying_ranges: vec![range],
                    probe_mode: mode_of(arm),
                    rollback_probe: rollback,
                    ..Spec::default()
                };
                let outcome = run(&parameters, &snapshot, arm, &spec);
                println!(
                    "{}",
                    line(snapshot.label, &format!("整段落盘报错@{what}"), arm, &outcome)
                );
            }
        }
    }
}
