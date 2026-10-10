//! 层 0：起点之后的写全量枚举（`enumerate_layer0_selecting_versions_observing_each_state`，每一段都展开），每个状态都跑 checker；
//! 观察者记下每个状态的持久集合，枚举完之后按 `stage2_every` 取（1 = 每个状态都取）交给第二截。
use std::collections::BTreeMap;
use std::sync::Mutex;

use singlefs_checker_tier::crash::{
    enumerate_layer0_selecting_versions_observing_each_state, full_expansion,
    layer0_state_count_with_torn_in_place_overwrites,
};
use singlefs_harness::memory_pool::RetainedWrite;
use singlefs_harness::segments::StepKind;

use crate::common::Recorded;
use crate::stage2::{judge_state, StateRecord, Tally};

pub struct Options {
    pub max_states: u64,
    pub stage2_every: u64,
    pub second_crash_cap: usize,
    pub threads: usize,
}

pub const TARGETS: [&str; 4] = ["I-9.16", "I-2.4", "I-5.4", "I-5.5"];

/// 全量枚举的状态数（每一段都展开、原地覆写取三态）；起点之后没有根槽写时 None。
pub fn state_count(recorded: &Recorded) -> Option<u64> {
    recorded
        .writes
        .iter()
        .any(|write| write.kind == StepKind::RootRecordFua)
        .then(|| {
            layer0_state_count_with_torn_in_place_overwrites(
                &recorded.base,
                &recorded.writes,
                &recorded.segments,
                &full_expansion,
            )
        })
}

/// 一段历史跑完层 0（与第二截）交回：状态数、checker 违例（按不变量）、目标几条的 评估/违例/不适用、第二截计数。
pub fn run_one(recorded: &Recorded, seed: u64, options: &Options, label: &str) -> Option<(u64, Tally)> {
    let Some(judged_root_index) = recorded
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
    else {
        println!("L0SKIP {label} seed={seed} 起点之后没有根槽写 ending={}", recorded.ending);
        return None;
    };
    let states = layer0_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &full_expansion,
    );
    if states > options.max_states {
        println!("L0SKIP {label} seed={seed} 状态数 {states} 超过 {} ending={}", options.max_states, recorded.ending);
        return None;
    }
    let mut segment_of_write = vec![0usize; recorded.writes.len()];
    for (index, segment) in recorded.segments.iter().enumerate() {
        for write in segment {
            segment_of_write[*write] = index;
        }
    }
    let mut enumerated: Option<Vec<RetainedWrite>> = None;
    let mut collected: Vec<StateRecord> = Vec::new();
    let mut ordinal = 0u64;
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        judged_root_index,
        &recorded.versions,
        &|_, _| true,
        &mut |image, _report| {
            if enumerated.is_none() {
                enumerated = Some(image.writes.to_vec());
            }
            ordinal += 1;
            if options.stage2_every == 0 || ordinal % options.stage2_every != 0 {
                return;
            }
            let first_withheld = image.persisted[..recorded.writes.len()]
                .iter()
                .position(|persisted| !*persisted);
            let segment = first_withheld.map_or(recorded.segments.len() - 1, |write| segment_of_write[write]);
            let window_end = recorded.stream_indexes[*recorded.segments[segment].last().expect("段非空")];
            let injected = recorded
                .faulted_steps
                .iter()
                .any(|(_, before)| *before <= window_end);
            collected.push(StateRecord { persisted: image.persisted.clone(), segment, injected });
        },
    );
    let mut violated: BTreeMap<&str, u64> = BTreeMap::new();
    for (invariant, count) in &tally.checker_violated_states {
        if *count > 0 {
            violated.insert(invariant, *count);
        }
    }
    let targets: Vec<String> = TARGETS
        .iter()
        .map(|invariant| {
            format!(
                "{invariant}={}/{}/{}",
                tally.checker_evaluated_states.get(invariant).copied().unwrap_or(0),
                tally.checker_violated_states.get(invariant).copied().unwrap_or(0),
                tally.checker_not_applicable_states.get(invariant).copied().unwrap_or(0)
            )
        })
        .collect();
    println!(
        "L0 {label} seed={seed} states={} closed_form={states} segments={} faulted_steps={:?} oracle_violations={} root_without_record={} checker_violated={violated:?} targets={} ending={}",
        tally.states,
        recorded.segments.len(),
        recorded.faulted_steps,
        tally.violations,
        tally.record_root_without_record,
        targets.join(","),
        recorded.ending
    );
    for (invariant, first) in &tally.checker_first_violation {
        println!("L0FIRST {label} seed={seed} {invariant}: {}", first.chars().take(400).collect::<String>());
    }
    let mut stage2 = Tally::default();
    if let Some(enumerated) = enumerated {
        let queue = Mutex::new(collected.into_iter());
        let merged = Mutex::new(Tally::default());
        std::thread::scope(|scope| {
            for _ in 0..options.threads.max(1) {
                scope.spawn(|| {
                    let mut local = Tally::default();
                    loop {
                        let next = queue.lock().expect("队列").next();
                        let Some(state) = next else { break };
                        judge_state(recorded, &enumerated, &state, seed, options.second_crash_cap, &mut local);
                    }
                    merged.lock().expect("合并").absorb(local);
                });
            }
        });
        stage2 = merged.into_inner().expect("合并");
    }
    Some((tally.states, stage2))
}
