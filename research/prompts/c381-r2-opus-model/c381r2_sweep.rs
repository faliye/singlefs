//! c381-r2 攻方副本：四条臂（甲对照、丙、戊-A、戊-B）× 七段前缀 × 第一轮没取过的故障型。
#[path = "c381r2_probe_common.rs"]
mod common;
use common::*;
use singlefs_harness::scenario::e142_parameters;
use std::collections::BTreeMap;

const ARMS: [u8; 4] = [0, 2, 5, 6];

fn prefixes() -> Vec<Snapshot> {
    let base = base_snapshot();
    let after_mount = remounted(&base, "remounted");
    let many = plus_overwrites(&after_mount, 5, "many_overwrites");
    vec![
        plus_empty_publishes(&base, 1, "txg6"),
        plus_empty_publishes(&base, 2, "txg7"),
        after_mount.clone(),
        rolled_back(&base, "rolled_back"),
        many.clone(),
        floor_raised(&many, "floor_raised"),
        base,
    ]
}

/// 一批工件的汇总：按（前缀, 臂, 分类）数段数，每一类留第一例的原样。
#[derive(Default)]
struct Tally {
    counts: BTreeMap<(String, &'static str, String), usize>,
    first: BTreeMap<(String, &'static str, String), String>,
}

impl Tally {
    fn add(&mut self, prefix: &str, arm: u8, label: &str, outcome: &Outcome) {
        let class = classify(outcome);
        let key = (prefix.to_string(), arm_name(arm), class.clone());
        *self.counts.entry(key.clone()).or_default() += 1;
        self.first.entry(key).or_insert_with(|| {
            format!(
                "{label} steps={:?} remount={} crash红={:?} after红={:?} 内容={} 切换={} 只读={} 实例={} 探针={}",
                outcome.steps,
                outcome.remount,
                outcome.checker_at_crash,
                outcome.checker_after_remount,
                outcome.acknowledged_content_after_remount,
                outcome.switches,
                outcome.read_only,
                outcome.final_instance,
                outcome.probe_writes
            )
        });
    }
    fn report(&self, phase: &str) {
        println!("=== {phase} 汇总 ===");
        for ((prefix, arm, class), count) in &self.counts {
            println!("[{prefix} {arm} {class}] runs={count}");
            if class != "干净" {
                println!("    first: {}", self.first[&(prefix.clone(), *arm, class.clone())]);
            }
        }
    }
}

fn merge(into: &mut Tally, from: Tally) {
    for (key, count) in from.counts {
        *into.counts.entry(key).or_default() += count;
    }
    for (key, value) in from.first {
        into.first.entry(key).or_insert(value);
    }
}

/// 一：四种故障型 × C 的 21 个写下标 × 用户动作长 ≤ 1，不断电。
#[test]
#[ignore = "手动跑"]
fn phase_f_fault_kinds() {
    let snapshots = prefixes();
    let kinds = [
        (Fault::Transient, "瞬时"),
        (Fault::Lying, "已落盘但报错"),
        (Fault::TornError, "撕裂+报错"),
        (Fault::TornSilent, "撕裂而不报错"),
    ];
    let actions: Vec<Vec<Action>> = vec![vec![], vec![Action::Retry], vec![Action::Remount]];
    let work: Vec<(usize, u8, u64, usize, usize)> = (0..snapshots.len())
        .flat_map(|snapshot| {
            ARMS.iter().flat_map(move |arm| {
                (0..21u64).flat_map(move |k| {
                    (0..kinds.len()).flat_map(move |kind| {
                        (0..3usize).map(move |action| (snapshot, *arm, k, kind, action))
                    })
                })
            })
        })
        .collect();
    let results = sweep(work.len(), 24, |parameters, index| {
        let (snapshot, arm, k, kind, action) = work[index];
        let spec = Spec {
            faults: vec![(k, kinds[kind].0)],
            actions: actions[action].clone(),
            switch_selects_base_root: arm == 6,
            ..Spec::default()
        };
        let outcome = run(parameters, &snapshots[snapshot], arm, &spec);
        (snapshot, arm, format!("k={k} {} actions={:?}", kinds[kind].1, actions[action]), outcome)
    });
    let mut tally = Tally::default();
    for (_, (snapshot, arm, label, outcome)) in &results {
        tally.add(snapshots[*snapshot].label, *arm, label, outcome);
    }
    tally.report("一（故障型 × 下标，不断电）");
}

/// 二：故障型 × C 的后六个写下标 × 断电点 0..60。
#[test]
#[ignore = "手动跑"]
fn phase_c_crash_points() {
    let snapshots = prefixes();
    let kinds = [
        (Fault::Transient, "瞬时"),
        (Fault::Lying, "已落盘但报错"),
        (Fault::TornError, "撕裂+报错"),
    ];
    let indices = [15u64, 16, 17, 18, 19, 20];
    let work: Vec<(usize, u8, usize, usize, u64)> = (0..snapshots.len())
        .flat_map(|snapshot| {
            ARMS.iter().flat_map(move |arm| {
                (0..indices.len()).flat_map(move |index| {
                    (0..kinds.len()).flat_map(move |kind| {
                        (0..61u64).map(move |crash| (snapshot, *arm, index, kind, crash))
                    })
                })
            })
        })
        .collect();
    let results = sweep(work.len(), 24, |parameters, item| {
        let (snapshot, arm, index, kind, crash) = work[item];
        let spec = Spec {
            faults: vec![(indices[index], kinds[kind].0)],
            crash_at: Some(crash),
            switch_selects_base_root: arm == 6,
            ..Spec::default()
        };
        let outcome = run(parameters, &snapshots[snapshot], arm, &spec);
        (
            snapshot,
            arm,
            format!("k={} {} crash={crash}", indices[index], kinds[kind].1),
            outcome,
        )
    });
    let mut tally = Tally::default();
    for (_, (snapshot, arm, label, outcome)) in &results {
        tally.add(snapshots[*snapshot].label, *arm, label, outcome);
    }
    tally.report("二（故障型 × 断电点）");
}

/// 三：屏障报错（C 自己两道，切换里还有几道）× 用户动作 × 断电点。
#[test]
#[ignore = "手动跑"]
fn phase_b_barrier() {
    let snapshots = prefixes();
    let actions: Vec<Vec<Action>> = vec![vec![], vec![Action::Retry], vec![Action::Remount]];
    let work: Vec<(usize, u8, u64, usize, Option<u64>)> = (0..snapshots.len())
        .flat_map(|snapshot| {
            ARMS.iter().flat_map(move |arm| {
                (0..13u64).flat_map(move |barrier| {
                    (0..3usize).flat_map(move |action| {
                        (0..9usize).map(move |crash| {
                            (
                                snapshot,
                                *arm,
                                barrier,
                                action,
                                if crash == 0 { None } else { Some((crash as u64 - 1) * 8) },
                            )
                        })
                    })
                })
            })
        })
        .collect();
    let results = sweep(work.len(), 24, |parameters, item| {
        let (snapshot, arm, barrier, action, crash) = work[item];
        let spec = Spec {
            barrier_fault: Some(barrier),
            actions: actions[action].clone(),
            crash_at: crash,
            switch_selects_base_root: arm == 6,
            ..Spec::default()
        };
        let outcome = run(parameters, &snapshots[snapshot], arm, &spec);
        (
            snapshot,
            arm,
            format!("屏障{barrier} actions={:?} crash={crash:?}", actions[action]),
            outcome,
        )
    });
    let mut tally = Tally::default();
    for (_, (snapshot, arm, label, outcome)) in &results {
        tally.add(snapshots[*snapshot].label, *arm, label, outcome);
    }
    tally.report("三（屏障报错）");
}

/// 四：整块盘从某个写下标起持续失败（戊的探针写该判「持续」）× 探针写落点政策。
#[test]
#[ignore = "手动跑"]
fn phase_d_dead_device() {
    let snapshots = prefixes();
    let work: Vec<(usize, u8, u32, u64, bool, bool)> = (0..snapshots.len())
        .flat_map(|snapshot| {
            ARMS.iter().flat_map(move |arm| {
                (0..2u32).flat_map(move |device| {
                    (0..21u64).flat_map(move |from| {
                        [false, true].into_iter().flat_map(move |probe_zero| {
                            [false, true]
                                .into_iter()
                                .map(move |remount| (snapshot, *arm, device, from, probe_zero, remount))
                        })
                    })
                })
            })
        })
        .collect();
    let results = sweep(work.len(), 24, |parameters, item| {
        let (snapshot, arm, device, from, probe_zero, remount) = work[item];
        let spec = Spec {
            dead_device: Some((device, from)),
            probe_to_device_zero: probe_zero,
            switch_selects_base_root: arm == 6,
            actions: if remount { vec![Action::Remount] } else { vec![] },
            ..Spec::default()
        };
        let outcome = run(parameters, &snapshots[snapshot], arm, &spec);
        (
            snapshot,
            arm,
            format!("盘{device}从{from}起坏 探针打盘0={probe_zero} 之后重开={remount}"),
            outcome,
        )
    });
    let mut tally = Tally::default();
    for (_, (snapshot, arm, label, outcome)) in &results {
        tally.add(snapshots[*snapshot].label, *arm, label, outcome);
    }
    tally.report("四（整块盘持续失败）");
}

/// 五：戊自己那几步（探针写、取号、写行、暖机、重发）上再失败：C 上一个故障 + 之后某个下标再一个。
#[test]
#[ignore = "手动跑"]
fn phase_e_switch_steps() {
    let snapshots = prefixes();
    let first = [16u64, 18, 19];
    let kinds = [
        (Fault::Transient, "瞬时"),
        (Fault::TornError, "撕裂+报错"),
    ];
    let work: Vec<(usize, u8, usize, u64, usize)> = (0..snapshots.len())
        .flat_map(|snapshot| {
            [5u8, 6].iter().flat_map(move |arm| {
                (0..first.len()).flat_map(move |index| {
                    (21..70u64).flat_map(move |second| {
                        (0..kinds.len()).map(move |kind| (snapshot, *arm, index, second, kind))
                    })
                })
            })
        })
        .collect();
    let results = sweep(work.len(), 24, |parameters, item| {
        let (snapshot, arm, index, second, kind) = work[item];
        let spec = Spec {
            faults: vec![(first[index], Fault::Transient), (second, kinds[kind].0)],
            switch_selects_base_root: arm == 6,
            ..Spec::default()
        };
        let outcome = run(parameters, &snapshots[snapshot], arm, &spec);
        (
            snapshot,
            arm,
            format!("C 第{}次写瞬时错 + 第{second}次写{}", first[index], kinds[kind].1),
            outcome,
        )
    });
    let mut tally = Tally::default();
    for (_, (snapshot, arm, label, outcome)) in &results {
        tally.add(snapshots[*snapshot].label, *arm, label, outcome);
    }
    tally.report("五（戊自己那几步上再失败）");
}

/// 六：断电撕裂——断电那一次写把前半落进盘（一条记录或一个单元只落一半）。
#[test]
#[ignore = "手动跑"]
fn phase_t_torn_crash() {
    let snapshots = prefixes();
    let work: Vec<(usize, u8, u64, bool)> = (0..snapshots.len())
        .flat_map(|snapshot| {
            ARMS.iter().flat_map(move |arm| {
                (0..45u64).flat_map(move |crash| {
                    [false, true]
                        .into_iter()
                        .map(move |with_fault| (snapshot, *arm, crash, with_fault))
                })
            })
        })
        .collect();
    let results = sweep(work.len(), 24, |parameters, item| {
        let (snapshot, arm, crash, with_fault) = work[item];
        let spec = Spec {
            faults: if with_fault {
                vec![(16, Fault::Transient)]
            } else {
                vec![]
            },
            crash_at: Some(crash),
            crash_torn: true,
            switch_selects_base_root: arm == 6,
            ..Spec::default()
        };
        let outcome = run(parameters, &snapshots[snapshot], arm, &spec);
        (
            snapshot,
            arm,
            format!("断电撕裂 crash={crash} C第16次写瞬时错={with_fault}"),
            outcome,
        )
    });
    let mut tally = Tally::default();
    for (_, (snapshot, arm, label, outcome)) in &results {
        tally.add(snapshots[*snapshot].label, *arm, label, outcome);
    }
    let mut merged = Tally::default();
    merge(&mut merged, tally);
    merged.report("六（断电撕裂）");
}
