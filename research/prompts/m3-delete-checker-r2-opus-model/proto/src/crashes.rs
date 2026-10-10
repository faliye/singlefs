//! 崩溃状态：每段历史每一步（前缀那次挂载除外）写出的那一段，照层 0 的枚举域（`singlefs_checker_tier::crash` 的
//! `enumerate_layer0_selecting_versions_observing_each_state`，每一段都展开：屏障与 FUA 切段、段内整写子集逐个）逐个崩溃状态判，
//! 每个状态另跑一次池级 checker 与各候选。一段历史全量（各步闭式之和）超过上限的不跑、报出来。
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state;
use singlefs_harness::memory_pool::{closed_form_state_count, root_identity_of_write, writes_and_segments, PublishedVersion};
use singlefs_harness::segments::StepKind;

use crate::common::geometry;
use crate::driver::{Op, Pool};
use crate::{judge, plans, threads, Tally};

pub fn crash_plan(name: &str) -> Vec<(String, Vec<Op>, u64)> {
    let mut out = Vec::new();
    let texts: Vec<(&str, u64)> = match name {
        "c1" => vec![
            ("U1", 144),
            ("U1,R2", 144),
            ("C1,U2,R2", 144),
            ("U1,R2,R1", 144),
            ("U1,R2,U1", 144),
            ("C1,X,C1", 144),
            ("C2,U3,X,C1", 144),
            ("U1,C1,R3", 144),
        ],
        "c2" => vec![("W2,U1", 1), ("W2,U1,R3", 1), ("C1,U2,R2,X,C1", 144), ("C1,U1,R3,U2", 144)],
        other => other.split(';').map(|text| (text, 144)).collect(),
    };
    for (text, batch) in texts {
        out.push((format!("b{batch}:{text}"), plans::parse(text), batch));
    }
    out
}

fn steps_of(label: &str, ops: &[Op], batch: u64) -> (Pool, Vec<(usize, u64, Vec<usize>)>) {
    let mut pool = Pool::build(4 << 30, batch);
    for op in ops {
        pool.run(op);
    }
    let geometry = geometry();
    let mut sizes = Vec::new();
    for (index, step) in pool.steps.iter().enumerate().skip(1) {
        let operations = step.stream.retained_operations();
        let (writes, segments) = writes_and_segments(&operations[step.start..step.end], &geometry);
        let lengths: Vec<usize> = segments.iter().map(Vec::len).collect();
        let count = if writes.is_empty() { 0 } else { closed_form_state_count(&segments) };
        let _ = label;
        sizes.push((index, count, lengths));
    }
    (pool, sizes)
}

pub fn run(args: &[String]) {
    let mode = args.first().map(String::as_str).unwrap_or("count");
    let plan_name = args.get(1).cloned().unwrap_or_else(|| "c1".into());
    let cap: u64 = std::env::var("M3DCR2_CRASH_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(1_000_000);
    let plan = crash_plan(&plan_name);
    if mode == "count" {
        for (label, ops, batch) in &plan {
            let (pool, sizes) = steps_of(label, ops, *batch);
            let total: u64 = sizes.iter().map(|(_, count, _)| count).sum();
            println!("CRASHCOUNT {label} total={total} cap={cap} run={}", total <= cap);
            for (index, count, lengths) in &sizes {
                println!("  step #{index} {} [{}] closed_form={count} segments={lengths:?}", pool.steps[*index].label, pool.steps[*index].outcome);
            }
        }
        return;
    }
    let next = AtomicUsize::new(0);
    let total = Mutex::new(Tally::default());
    let started = std::time::Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..threads() {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::SeqCst);
                let Some((label, ops, batch)) = plan.get(index) else { break };
                let (pool, sizes) = steps_of(label, ops, *batch);
                let history_total: u64 = sizes.iter().map(|(_, count, _)| count).sum();
                if history_total > cap {
                    println!("CRASHSKIP {label} total={history_total} cap={cap}（一段历史超过上限，不跑）");
                    continue;
                }
                let mut tally = Tally::default();
                let geometry = geometry();
                let mut enumerated = 0u64;
                let mut tier_checker_reds: std::collections::BTreeMap<&'static str, u64> = Default::default();
                for (step_index, count, _) in &sizes {
                    if *count == 0 {
                        continue;
                    }
                    let step = &pool.steps[*step_index];
                    let operations = step.stream.retained_operations();
                    let (writes, segments) = writes_and_segments(&operations[step.start..step.end], &geometry);
                    let Some(root_index) = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua) else {
                        println!("CRASHNOROOT {label} #{step_index}（这一步没有根槽写，不枚举）");
                        continue;
                    };
                    let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_index]);
                    let versions = [PublishedVersion { instance, checkpoint_txg, content: Vec::new() }];
                    let context = format!("{label} #{step_index}:{}", step.label);
                    let mut ordinal = 0u64;
                    let layer0 = enumerate_layer0_selecting_versions_observing_each_state(
                        &step.base,
                        &writes,
                        &segments,
                        root_index,
                        &versions,
                        &|_segment_index, _segment| true,
                        &mut |image, _report| {
                            let judged = judge(image);
                            tally.note("crash", &judged, &format!("{context} state{ordinal}"));
                            ordinal += 1;
                        },
                    );
                    assert_eq!(layer0.states, ordinal, "观察者看过每一个状态");
                    enumerated += layer0.states;
                    for (name, count) in &layer0.checker_violated_states {
                        *tier_checker_reds.entry(name).or_default() += count;
                    }
                }
                println!(
                    "CRASHRUN {label} closed_form_total={history_total} enumerated={enumerated} tier_checker_violated={tier_checker_reds:?} elapsed={:.1}s",
                    started.elapsed().as_secs_f64()
                );
                total.lock().expect("锁").absorb(tally);
            });
        }
    });
    let total = total.into_inner().expect("锁");
    total.print(&format!("crash-{plan_name}"), false);
}
