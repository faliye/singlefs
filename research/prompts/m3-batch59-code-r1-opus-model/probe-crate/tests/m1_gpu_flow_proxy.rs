//! M1（m3-batch59-code-r1 opus）：被判那几条 GPU 旋钮对拍流（第 58 批 J3 的 [2, 1, 1] 三条、第 59 批 L1 的 [2, 1, 2, 1] 转根环三条，
//! 另加我提的 [2, 1, 2, 2, 1] 转根环三条）的每一个崩溃状态上，CPU 池级 checker 带旋钮 2 的新入口判出的违例位与成立位
//! （`IMPLEMENTED_INVARIANTS` 全部），按状态序号排好、取摘要，另按恢复落到的根（consulted 那一遍的 effective_root 的 txg）分组数。
//! 在正确的 walk.rs 与改坏的 walk.rs 上各跑一次：摘要相同 ⇒ 那一种改坏在这条流的每个状态上与正确判法同判，GPU 扫描内核里
//! 同形的改坏也就在这条对拍上与 CPU 逐位相同（GPU 比的就是这几位、判的就是同一组状态；这一步是推的，GPU 不跑）。
//! 状态域：被判用例自己的展开（段长 ≤ 8 的段取每个真子集，更长的段不展开，`SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO` 缺省 8）；
//! 层 0 全量（每段都展开）只算状态数，超过 10^6 的不跑。
mod flows_b59;

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use flows_b59::{record_knob_flow, KnobFlow};
use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
use singlefs_checker::walk::{
    check_pool_image_given_the_container_capacity_knobs, ContainerCapacityKnobs, LineageContainerCapacityKnob,
    RetirementContainerCapacityKnob, TombstoneContainerCapacityKnob,
};
use singlefs_checker_tier::crash::{
    judge_layer0_state_range_against_the_version_table, layer0_plan_state_count, Layer0SegmentExpansion,
    Layer0VersionTable,
};
use singlefs_harness::memory_pool::closed_form_state_count;

const STATE_LIMIT_PER_RUN: u64 = 1_000_000;

const FLOWS: [KnobFlow; 9] = [
    KnobFlow { name: "tombstone-2-1-1", tree: "tombstone", knobs: &[2, 2, 1, 1], ring_turn: false, expected_counts: &[2, 1, 1] },
    KnobFlow { name: "lineage-2-1-1", tree: "lineage", knobs: &[2, 2, 1, 1], ring_turn: false, expected_counts: &[2, 1, 1] },
    KnobFlow { name: "retirement-2-1-1", tree: "retirement", knobs: &[2, 2, 1, 1], ring_turn: false, expected_counts: &[2, 1, 1] },
    KnobFlow { name: "tombstone-2-1-2-1-ring", tree: "tombstone", knobs: &[2, 2, 1, 1, 2, 1], ring_turn: true, expected_counts: &[2, 1, 2, 1] },
    KnobFlow { name: "lineage-2-1-2-1-ring", tree: "lineage", knobs: &[2, 2, 1, 1, 2, 1], ring_turn: true, expected_counts: &[2, 1, 2, 1] },
    KnobFlow { name: "retirement-2-1-2-1-ring", tree: "retirement", knobs: &[2, 2, 1, 1, 2, 1], ring_turn: true, expected_counts: &[2, 1, 2, 1] },
    KnobFlow { name: "tombstone-2-1-2-2-1-ring", tree: "tombstone", knobs: &[2, 2, 1, 1, 2, 2, 2, 1], ring_turn: true, expected_counts: &[2, 1, 2, 2, 1] },
    KnobFlow { name: "lineage-2-1-2-2-1-ring", tree: "lineage", knobs: &[2, 2, 1, 1, 2, 2, 2, 1], ring_turn: true, expected_counts: &[2, 1, 2, 2, 1] },
    KnobFlow { name: "retirement-2-1-2-2-1-ring", tree: "retirement", knobs: &[2, 2, 1, 1, 2, 2, 2, 1], ring_turn: true, expected_counts: &[2, 1, 2, 2, 1] },
];

fn knobs_of(tree: &str) -> (ContainerCapacityKnobs, &'static str) {
    match tree {
        "tombstone" => (
            ContainerCapacityKnobs { tombstone: Some(TombstoneContainerCapacityKnob::new(2).expect("2")), lineage: None, retirement: None },
            "I-9.22",
        ),
        "lineage" => (
            ContainerCapacityKnobs { tombstone: None, lineage: Some(LineageContainerCapacityKnob::new(2).expect("2")), retirement: None },
            "I-9.37",
        ),
        _ => (
            ContainerCapacityKnobs { tombstone: None, lineage: None, retirement: Some(RetirementContainerCapacityKnob::new(2).expect("2")) },
            "I-9.37",
        ),
    }
}

fn bits_of(verdicts: &[(&'static str, InvariantVerdict)]) -> (u128, u128) {
    verdicts.iter().fold((0u128, 0u128), |(violated, held), (invariant, verdict)| {
        let bit = 1u128 << IMPLEMENTED_INVARIANTS.iter().position(|name| name == invariant).expect("清单里有");
        match verdict {
            InvariantVerdict::Violated(_) => (violated | bit, held),
            InvariantVerdict::Holds => (violated, held | bit),
            InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => (violated, held),
        }
    })
}

/// 把 [a, a, a, b, b] 这类逐步形状压成「a×3 b×2」。
fn runs_of(shapes: &[Vec<usize>]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut index = 0;
    while index < shapes.len() {
        let mut end = index;
        while end < shapes.len() && shapes[end] == shapes[index] {
            end += 1;
        }
        out.push(format!("{:?}x{}", shapes[index], end - index));
        index = end;
    }
    out.join(" ")
}

fn judge_the_flow(flow_spec: &KnobFlow) {
    let (flow, written, turned) = record_knob_flow(flow_spec);
    let turned_shapes: Vec<Vec<usize>> = turned.iter().map(|(_, shape)| shape.clone()).collect();
    eprintln!(
        "OPUS_M1 shape flow={} written={} ring_turn_txgs={} ring_turn_shapes={} last_txg={}",
        flow.name,
        runs_of(&written),
        turned.first().map_or("-".to_string(), |(first, _)| format!("{first}..={}", turned.last().expect("非空").0)),
        if turned.is_empty() { "-".to_string() } else { runs_of(&turned_shapes) },
        flow.versions.last().expect("有版本").checkpoint_txg.0
    );
    let limit = 8usize;
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let longest = flow.segments.iter().map(Vec::len).max().unwrap_or(0);
    let closed_form = if longest < 60 { format!("{}", closed_form_state_count(&flow.segments)) } else { format!(">2^{longest}") };
    let state_count = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
    eprintln!(
        "OPUS_M1 plan flow={} domain=gpu-test writes={} segments={} longest_segment={longest} closed_form_full={closed_form} states={state_count}",
        flow.name,
        flow.writes.len(),
        flow.segments.len()
    );
    if state_count > STATE_LIMIT_PER_RUN {
        eprintln!("OPUS_M1 skipped flow={} states={state_count} 超过 10^6", flow.name);
        return;
    }
    let (knobs, invariant) = knobs_of(flow_spec.tree);
    let invariant_bit = 1u128 << IMPLEMENTED_INVARIANTS.iter().position(|name| *name == invariant).expect("清单里有");
    let threads: u64 = std::env::var("OPUS_THREADS").ok().and_then(|value| value.parse().ok()).unwrap_or(4);
    let slice = state_count.div_ceil(threads * 8).max(1);
    let next = std::sync::atomic::AtomicU64::new(0);
    let mut judged: Vec<(u64, u64, u128, u128)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let (flow, next, expansion) = (&flow, &next, &expansion);
                scope.spawn(move || {
                    let mut out = Vec::new();
                    loop {
                        let index = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let start = index * slice;
                        if start >= state_count {
                            break;
                        }
                        let range = start..(start + slice).min(state_count);
                        let _ = judge_layer0_state_range_against_the_version_table(
                            &flow.base,
                            &flow.writes,
                            &flow.segments,
                            flow.judged_root_index,
                            Layer0VersionTable::new(&flow.versions, &[]),
                            expansion,
                            range,
                            &mut |ordinal, image, judgement| {
                                let verdicts = check_pool_image_given_the_container_capacity_knobs(image, knobs);
                                let (violated, held) = bits_of(&verdicts);
                                let root_txg = judgement.consulted.effective_root.map_or(0, |(_, txg)| txg.0);
                                out.push((ordinal, root_txg, violated, held));
                            },
                        );
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().flat_map(|handle| handle.join().expect("不 panic")).collect()
    });
    judged.sort_by_key(|(ordinal, _, _, _)| *ordinal);
    assert_eq!(u64::try_from(judged.len()).expect("u64"), state_count);
    let mut hasher = DefaultHasher::new();
    judged.hash(&mut hasher);
    let knob_violated = judged.iter().filter(|(_, _, violated, _)| violated & invariant_bit != 0).count();
    let knob_held = judged.iter().filter(|(_, _, _, held)| held & invariant_bit != 0).count();
    let any_violated = judged.iter().filter(|(_, _, violated, _)| *violated != 0).count();
    let other_violated = judged.iter().filter(|(_, _, violated, _)| violated & !invariant_bit != 0).count();
    eprintln!(
        "OPUS_M1 judged flow={} states={state_count} {invariant}_violated_states={knob_violated} {invariant}_held_states={knob_held} states_with_any_violation={any_violated} states_with_another_violation={other_violated} digest={:016x}",
        flow.name,
        hasher.finish()
    );
    let mut by_root: BTreeMap<u64, (usize, usize, usize)> = BTreeMap::new();
    for (_, root_txg, violated, held) in &judged {
        let entry = by_root.entry(*root_txg).or_insert((0, 0, 0));
        entry.0 += 1;
        entry.1 += usize::from(violated & invariant_bit != 0);
        entry.2 += usize::from(held & invariant_bit != 0);
    }
    let listing: Vec<String> = by_root
        .iter()
        .map(|(txg, (states, violated, held))| format!("{txg}:{states}/{violated}/{held}"))
        .collect();
    eprintln!("OPUS_M1 by-root flow={} root_txg:states/{invariant}_violated/{invariant}_held {}", flow.name, listing.join(" "));
}

#[test]
fn opus_probe_b59_m1_gpu_flows_on_the_cpu() {
    let wanted: Option<Vec<String>> =
        std::env::var("OPUS_M1_FLOWS").ok().map(|value| value.split(',').map(str::to_string).collect());
    for flow in &FLOWS {
        if wanted.as_ref().is_none_or(|names| names.iter().any(|name| name == flow.name)) {
            judge_the_flow(flow);
        }
    }
}
