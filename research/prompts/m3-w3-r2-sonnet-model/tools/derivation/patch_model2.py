#!/usr/bin/env python3
"""patch_model.py 的第二刀：加「自上次回退起挂载过没有」的标志（把 C1b 的漏分成会话内与挂载之后）、第三个阳性对照臂 V3（恢复时把水位少算 1）、辩方组配置。
用法：patch_model2.py <输入 main.rs> <输出 main.rs>"""
import sys
src, out = sys.argv[1], sys.argv[2]
t = open(src, encoding="utf-8").read()
def once(old, new):
    global t
    n = t.count(old)
    if n != 1:
        sys.stderr.write(f"旧串命中 {n} 次：{old[:90]!r}\n"); sys.exit(3)
    t = t.replace(old, new)

once("""    PublishDoesNotAddTheNewInodes,
}""", """    PublishDoesNotAddTheNewInodes,
    /// 辩方腿加的第三个阳性对照臂 V3：恢复（挂载、崩溃恢复、实例切换）时新实例的水位比所选根的少 1（不低于 2），即「恢复时水位取错」，与回退无关。
    RecoveryLowersTheWatermark,
}""")
once("""    fixed_erased_by_the_sweep: Vec<FixedEntry>,
}""", """    fixed_erased_by_the_sweep: Vec<FixedEntry>,
    /// 自最近一次管理员回退起，有没有发生过恢复类的步（挂载、崩溃恢复、实例切换）。回退时清零。
    mounted_since_rollback: bool,
}""")
once("""            fixed_erased_by_the_sweep: vec![FixedEntry { instance: 1, txg: 3, inode: 1, birth: 3 }],
        };""", """            fixed_erased_by_the_sweep: vec![FixedEntry { instance: 1, txg: 3, inode: 1, birth: 3 }],
            mounted_since_rollback: false,
        };""")
# V3：recover_onto 与切换里新实例的水位
once("""        let version = self.next_version(
            config, &selected, new_instance, txg, selected.watermark, selected.tree.clone(), selected.floor, table.clone(),""",
"""        let recovered_watermark = match config.arm {
            Arm::RecoveryLowersTheWatermark => selected.watermark.saturating_sub(1).max(2),
            _ => selected.watermark,
        };
        let version = self.next_version(
            config, &selected, new_instance, txg, recovered_watermark, selected.tree.clone(), selected.floor, table.clone(),""")
# create_in 对 V3：与 P 同（inode+1）
once("""            Arm::Product | Arm::RollbackTakesTheTargetsWatermark => inode + 1,
        };
        let issued = union_sorted(&base.issued, &[(inode, txg)]);""", """            Arm::Product | Arm::RollbackTakesTheTargetsWatermark | Arm::RecoveryLowersTheWatermark => inode + 1,
        };
        let issued = union_sorted(&base.issued, &[(inode, txg)]);""")
once("""                    Arm::Product | Arm::PublishDoesNotAddTheNewInodes => current.watermark.max(ring_highest),""",
     """                    Arm::Product | Arm::PublishDoesNotAddTheNewInodes | Arm::RecoveryLowersTheWatermark => current.watermark.max(ring_highest),""")
once("""                    Arm::Product | Arm::RollbackTakesTheTargetsWatermark => current.watermark + 1,""",
     """                    Arm::Product | Arm::RollbackTakesTheTargetsWatermark | Arm::RecoveryLowersTheWatermark => current.watermark + 1,""")
# 标志：回退清零，恢复类置位
once("""                version.lineage_tree.push((next_txg, target.txg));""", """                version.lineage_tree.push((next_txg, target.txg));
                self.mounted_since_rollback = false;""")
once("""            Op::Mount => {
                let selected = self.newest_readable().clone();""", """            Op::Mount => {
                self.mounted_since_rollback = true;
                let selected = self.newest_readable().clone();""")
once("""                let slot = usize::try_from(newest.txg).expect("小") % self.ring.len();
                self.ring[slot].as_mut().expect("最新那条根在环里").readable = false;""", """                self.mounted_since_rollback = true;
                let slot = usize::try_from(newest.txg).expect("小") % self.ring.len();
                self.ring[slot].as_mut().expect("最新那条根在环里").readable = false;""")
once("""                self.append_fixed(FixedEntry { instance: current.instance, txg: next_txg, inode: current.watermark, birth: next_txg });
                let selected = self.newest_readable().clone();
                self.recover_onto(config, selected.clone(), selected.watermark, next_txg);
                self.sweep(config);
                Some(StepResult { issued_now: Vec::new() })
            }
            Op::SwitchRedoingTheCreate""", """                self.mounted_since_rollback = true;
                self.append_fixed(FixedEntry { instance: current.instance, txg: next_txg, inode: current.watermark, birth: next_txg });
                let selected = self.newest_readable().clone();
                self.recover_onto(config, selected.clone(), selected.watermark, next_txg);
                self.sweep(config);
                Some(StepResult { issued_now: Vec::new() })
            }
            Op::SwitchRedoingTheCreate""")
once("""                self.append_fixed(FixedEntry { instance: current.instance, txg: next_txg, inode: current.watermark, birth: next_txg });
                let memory_watermark = match config.arm {""", """                self.mounted_since_rollback = true;
                self.append_fixed(FixedEntry { instance: current.instance, txg: next_txg, inode: current.watermark, birth: next_txg });
                let memory_watermark = match config.arm {""")
# 切换里 error 版本的水位
once("""config, &selected, new_instance, next_txg, selected.watermark, selected.tree.clone(), selected.floor, table,""",
     """config, &selected, new_instance, next_txg, match config.arm { Arm::RecoveryLowersTheWatermark => selected.watermark.saturating_sub(1).max(2), _ => selected.watermark }, selected.tree.clone(), selected.floor, table,""")
# 计数：kind 8 / 9：existing 静默的事件里，自上次回退起挂载过 / 没挂载过
once("""const KINDS: [&str; 8] = [""", """const KINDS: [&str; 10] = [""")
once("""    "residual_missed_where_existing_checks_silent",
];""", """    "residual_missed_where_existing_checks_silent",
    "event_missed_where_existing_silent_and_mounted_since_the_last_rollback",
    "event_missed_where_existing_silent_and_not_mounted_since_the_last_rollback",
];""")
once("""    counts: [[u64; 8]; N],""", """    counts: [[u64; 10]; N],
    events_existing_silent_mounted: u64,
    events_existing_silent_not_mounted: u64,""")
once("""            if existing_silent {
                self.events_where_existing_silent += 1;
            }
        }""", """            if existing_silent {
                self.events_where_existing_silent += 1;
                if state.mounted_since_rollback {
                    self.events_existing_silent_mounted += 1;
                } else {
                    self.events_existing_silent_not_mounted += 1;
                }
            }
        }""")
once("""                if existing_silent {
                    self.hit(candidate, 6, sequence);
                }
            }""", """                if existing_silent {
                    self.hit(candidate, 6, sequence);
                    self.hit(candidate, if state.mounted_since_rollback { 8 } else { 9 }, sequence);
                }
            }""")
once("""events_where_existing_checks_silent={} residual_images""", """events_where_existing_checks_silent={} events_existing_silent_mounted_since_the_last_rollback={} events_existing_silent_not_mounted_since_the_last_rollback={} residual_images""")
once("""tally.events_where_existing_silent,
        tally.residual_images""", """tally.events_where_existing_silent,
        tally.events_existing_silent_mounted, tally.events_existing_silent_not_mounted, tally.residual_images""")
# 配置：辩方组
once("""    let mut index_in_group: BTreeMap<&str, usize> = BTreeMap::new();""", """    // 辩方组（defense）：V1 在环 4 / 6 / 8 / 12 上（看 C1b 挂载之后的漏随环长怎么变）、V3、P 在环 8 上（误报）。
    for ring_slots in [4_usize, 6, 8, 12] {
        configs.push(("defense", Config { arm: Arm::RollbackTakesTheTargetsWatermark, ring_slots, sweep_threshold: u64::MAX, ..base }));
    }
    configs.push(("defense", Config { arm: Arm::RecoveryLowersTheWatermark, ring_slots: 4, sweep_threshold: u64::MAX, ..base }));
    configs.push(("defense", Config { arm: Arm::Product, ring_slots: 8, sweep_threshold: 0, ..base }));
    let mut index_in_group: BTreeMap<&str, usize> = BTreeMap::new();""")
open(out, "w", encoding="utf-8").write(t)
