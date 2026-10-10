//! K1（m3-batch58-code-r1 opus）：被判那三条 J3 GPU 对拍流（墓碑、谱系、退役各 [2, 1, 1]）的每一个崩溃状态上，CPU 池级 checker
//! 带旋钮 2 的新入口判出的违例位与成立位（`IMPLEMENTED_INVARIANTS` 全部），按状态序号排好、取摘要。在正确的 walk.rs 与改坏的
//! walk.rs（只判倒数第二片、只判最后两片非最右）上各跑一次，摘要相同 ⇒ 那一种改坏在这三条流的每个状态上与正确判法同判，
//! GPU 扫描内核里同形的改坏也就在这三条对拍用例上与 CPU 逐位相同（GPU 判的就是同一组状态、比的就是这几位）。
//! 状态域两种：`gpu-test`（被判用例的展开：段长 ≤ 8 的段取每个真子集，更长的段不展开，即 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO` 的缺省 8）；
//! `full`（每段都展开，层 0 全量），全量超过 10^6 个状态的不跑、只报状态数。
mod flows_b58;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use flows_b58::record_containers_of_two_one_and_one_records_under_changing_knobs;
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

fn judge_the_flow(tree: &str, domain: &str) {
    let flow = record_containers_of_two_one_and_one_records_under_changing_knobs(tree);
    let limit = match domain {
        "gpu-test" => 8usize,
        _ => usize::MAX,
    };
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
        "OPUS_K1P plan flow={} domain={domain} writes={} segments={} longest_segment={longest} closed_form_full={closed_form} states={state_count}",
        flow.name,
        flow.writes.len(),
        flow.segments.len()
    );
    if state_count > STATE_LIMIT_PER_RUN {
        eprintln!("OPUS_K1P skipped flow={} domain={domain} states={state_count} 超过 10^6", flow.name);
        return;
    }
    let (knobs, invariant) = knobs_of(tree);
    let invariant_bit = 1u128 << IMPLEMENTED_INVARIANTS.iter().position(|name| *name == invariant).expect("清单里有");
    let threads: u64 = std::env::var("OPUS_THREADS").ok().and_then(|value| value.parse().ok()).unwrap_or(4);
    let slice = state_count.div_ceil(threads * 8).max(1);
    let next = std::sync::atomic::AtomicU64::new(0);
    let mut judged: Vec<(u64, u128, u128)> = std::thread::scope(|scope| {
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
                            &mut |ordinal, image, _judgement| {
                                let verdicts = check_pool_image_given_the_container_capacity_knobs(image, knobs);
                                let (violated, held) = bits_of(&verdicts);
                                out.push((ordinal, violated, held));
                            },
                        );
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().flat_map(|handle| handle.join().expect("不 panic")).collect()
    });
    judged.sort_by_key(|(ordinal, _, _)| *ordinal);
    assert_eq!(u64::try_from(judged.len()).expect("u64"), state_count);
    let mut hasher = DefaultHasher::new();
    judged.hash(&mut hasher);
    let knob_violated = judged.iter().filter(|(_, violated, _)| violated & invariant_bit != 0).count();
    let knob_held = judged.iter().filter(|(_, _, held)| held & invariant_bit != 0).count();
    let any_violated = judged.iter().filter(|(_, violated, _)| *violated != 0).count();
    eprintln!(
        "OPUS_K1P judged flow={} domain={domain} states={state_count} {invariant}_violated_states={knob_violated} {invariant}_held_states={knob_held} states_with_any_violation={any_violated} digest={:016x}",
        flow.name,
        hasher.finish()
    );
}

#[test]
fn opus_probe_b58_k1_gpu_flows_on_the_cpu() {
    let domains: Vec<String> = std::env::var("OPUS_K1_DOMAINS")
        .unwrap_or_else(|_| "gpu-test".to_string())
        .split(',')
        .map(str::to_string)
        .collect();
    for tree in ["tombstone", "lineage", "retirement"] {
        for domain in &domains {
            judge_the_flow(tree, domain);
        }
    }
}
