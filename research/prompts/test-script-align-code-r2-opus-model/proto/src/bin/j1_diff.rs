//! J1 的分辨：块键（复用键、身份、计划哈希）相同的两个点，逐态现判的判定（red_items）与第 ② 段核对位是不是也逐态相同。
//! 只要有一个状态不同，就是「判器输入不同而块键相同」（同一判法版本下会互相复用、给错结局）。
//! 对：两流用例的 A 与 B；到 D 与到卸载两条固定脚本（共享前 36 个点）；到卸载那条与它的几种「只改点之后」的变体。
//! 不经库：每个点各自现判，按点内序号逐态比。状态数受展开上限约束（R2_EXPAND_UP_TO），每次跑之前打出这一次的状态数与全展开的闭式数。
use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use r2proto::flows::{content_of, rollback_flow, two_flow, unmount_flow, RecordedFlow};
use singlefs_checker_tier::crash::{judge_layer0_state_range, layer0_plan_state_count};
use singlefs_checker_tier::crash_amplification::plan_crash_points;
use singlefs_checker_tier::crash_facts::{extract_flow_facts, verify_states_from_facts};
use singlefs_harness::memory_pool::{closed_form_state_count, WrittenContents};
use singlefs_harness::segments::StepKind;

const STATES_PER_RUN_LIMIT: u64 = 1_000_000;
const CHUNK: u64 = 2048;

fn threads() -> usize {
    std::env::var("R2_THREADS").ok().map_or(4, |text| text.parse().expect("线程数"))
}

/// 一条流里 `ordinals` 这一段逐态现判的 red_items，按序号次序。
fn judge(flow: &RecordedFlow, ordinals: Range<u64>) -> Vec<Vec<String>> {
    let chunks: Vec<Range<u64>> = (ordinals.start..ordinals.end)
        .step_by(CHUNK as usize)
        .map(|start| start..(start + CHUNK).min(ordinals.end))
        .collect();
    let next = AtomicUsize::new(0);
    let results: Mutex<BTreeMap<usize, Vec<Vec<String>>>> = Mutex::new(BTreeMap::new());
    std::thread::scope(|scope| {
        for _ in 0..threads() {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::SeqCst);
                let Some(chunk) = chunks.get(index) else { break };
                let mut items = Vec::new();
                let _ = judge_layer0_state_range(
                    &flow.base, &flow.writes, &flow.segments, flow.judged_root_index, &flow.versions,
                    flow.expansion(), chunk.clone(),
                    &mut |_ordinal, _image, judgement| items.push(judgement.red_items()),
                );
                results.lock().expect("锁").insert(index, items);
            });
        }
    });
    results.into_inner().expect("锁").into_values().flatten().collect()
}

struct Judged {
    by_point: Vec<Option<Vec<Vec<String>>>>,
    verifier_by_point: Vec<Vec<u32>>,
}

fn sizes(flow: &RecordedFlow) -> u64 {
    let states = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, flow.expansion());
    println!(
        "J1DIFF_SIZE flow={} states_this_run={states} closed_form_state_count_fully_expanded={} expand_up_to={}",
        flow.name, closed_form_state_count(&flow.segments), flow.expand_up_to
    );
    assert!(states <= STATES_PER_RUN_LIMIT, "一条流这一次的状态数超过 10^6");
    states
}

fn verifier_bits(flow: &RecordedFlow) -> Vec<Vec<u32>> {
    let facts = extract_flow_facts(&flow.base, &flow.writes, &flow.segments, flow.expansion());
    plan_crash_points(&flow.flow()).points.iter().map(|point| verify_states_from_facts(&facts, point.ordinals.clone())).collect()
}

/// 两条流里块键相同的点（身份、复用键、计划哈希都相同），逐态比判定与核对位。`left_cache` 存左边那条已判过的点。
fn compare(left: &RecordedFlow, left_cache: &mut Judged, right: &RecordedFlow, label: &str, judged_states: &mut u64) {
    let plan_left = plan_crash_points(&left.flow());
    let plan_right = plan_crash_points(&right.flow());
    let verifier_right = verifier_bits(right);
    let (mut equal_key_points, mut states, mut differing, mut verifier_differing, mut first) = (0, 0u64, 0u64, 0u64, None);
    let mut equal_key_names = Vec::new();
    for (index_right, point_right) in plan_right.points.iter().enumerate() {
        let Some(index_left) = plan_left.points.iter().position(|point| {
            point.identity == point_right.identity && point.reuse_key == point_right.reuse_key && point.plan_hash == point_right.plan_hash
        }) else { continue };
        let point_left = &plan_left.points[index_left];
        if point_left.ordinals.end == point_left.ordinals.start {
            continue;
        }
        equal_key_points += 1;
        equal_key_names.push(index_right);
        if left_cache.by_point[index_left].is_none() {
            left_cache.by_point[index_left] = Some(judge(left, point_left.ordinals.clone()));
            *judged_states += point_left.ordinals.end - point_left.ordinals.start;
        }
        let fresh_right = judge(right, point_right.ordinals.clone());
        *judged_states += point_right.ordinals.end - point_right.ordinals.start;
        let fresh_left = left_cache.by_point[index_left].as_ref().expect("判过");
        assert_eq!(fresh_left.len(), fresh_right.len(), "块键相同、状态数相同");
        for (offset, (a, b)) in fresh_left.iter().zip(&fresh_right).enumerate() {
            states += 1;
            if a != b {
                differing += 1;
                first.get_or_insert((point_right.name.clone(), offset, a.clone(), b.clone()));
            }
        }
        let bits_left = &left_cache.verifier_by_point[index_left];
        let bits_right = &verifier_right[index_right];
        verifier_differing += bits_left.iter().zip(bits_right).filter(|(a, b)| a != b).count() as u64;
    }
    // 身份与复用键相同、计划哈希不同（只差判器输入）的点：逐态比判定，判定逐态相同就是「判器真读到的相同而块键不同」（前缀共享白丢）
    let (mut split_points, mut split_states, mut split_differing) = (0, 0u64, 0u64);
    let mut split_names = Vec::new();
    for point_right in &plan_right.points {
        let Some(point_left) = plan_left.points.iter().find(|point| {
            point.identity == point_right.identity && point.reuse_key == point_right.reuse_key && point.plan_hash != point_right.plan_hash
        }) else { continue };
        if point_left.ordinals.end - point_left.ordinals.start != point_right.ordinals.end - point_right.ordinals.start {
            split_names.push(format!("{}(states {} vs {})", point_right.name, point_left.ordinals.end - point_left.ordinals.start, point_right.ordinals.end - point_right.ordinals.start));
            continue;
        }
        split_points += 1;
        split_names.push(point_right.name.clone());
        let a = judge(left, point_left.ordinals.clone());
        let b = judge(right, point_right.ordinals.clone());
        *judged_states += (a.len() + b.len()) as u64;
        split_states += a.len() as u64;
        split_differing += a.iter().zip(&b).filter(|(x, y)| x != y).count() as u64;
    }
    println!(
        "J1SPLIT pair={label} same_identity_and_reuse_key_but_different_plan_hash_points={split_points} states_compared={split_states} judge_states_differing={split_differing} points={split_names:?}"
    );
    println!(
        "J1DIFF pair={label} left={} right={} equal_block_key_points_with_states={equal_key_points} states_compared={states} judge_states_differing={differing} verifier_states_differing={verifier_differing} first_difference={first:?}",
        left.name, right.name
    );
    println!("J1DIFF_POINTS pair={label} right_point_indexes={equal_key_names:?}");
}

fn flip_after(mut flow: RecordedFlow, after_write: usize, kind: StepKind, offset: usize, name: &str) -> RecordedFlow {
    let index = (after_write..flow.writes.len()).find(|index| flow.writes[*index].kind == kind).expect("点之后有这一种写");
    match &mut flow.writes[index].contents {
        WrittenContents::Bytes(bytes) => bytes[offset] ^= 0x5a,
        WrittenContents::Zeros { .. } => panic!("这一次写带字节"),
    }
    println!("J1DIFF_VARIANT name={name} write_index={index} kind={} byte={offset}", kind.name());
    flow.renamed(name)
}

fn main() {
    let expand_up_to: usize = std::env::var("R2_EXPAND_UP_TO").ok().map_or(6, |text| text.parse().expect("数"));
    let only: Option<String> = std::env::var("R2_ONLY").ok();
    let mut judged_states = 0u64;
    let wanted = |label: &str| only.as_deref().is_none_or(|only| only.split(',').any(|item| item == label));

    if wanted("two_flows") {
        let a = two_flow("diff-two-a", &content_of(3, 4100), true, expand_up_to).renamed("two_flows_a");
        let b = two_flow("diff-two-b", &content_of(11, 4100), false, expand_up_to).renamed("two_flows_b");
        sizes(&a);
        sizes(&b);
        let mut cache = Judged { by_point: vec![None; a.crash_points.len()], verifier_by_point: verifier_bits(&a) };
        compare(&a, &mut cache, &b, "two_flows", &mut judged_states);
    }

    let unmount = unmount_flow("diff-unmount", expand_up_to).renamed("unmount");
    sizes(&unmount);
    let mut cache = Judged { by_point: vec![None; unmount.crash_points.len()], verifier_by_point: verifier_bits(&unmount) };
    // D 那次回退的最后一次写之后（到 D 那条流的写数）
    let after_d = rollback_flow("diff-rollback-length", 6).writes.len();
    if wanted("rollback") {
        let rollback = rollback_flow("diff-rollback", expand_up_to).renamed("rollback");
        sizes(&rollback);
        compare(&unmount, &mut cache, &rollback, "rollback_vs_unmount", &mut judged_states);
    }
    let mut variants: Vec<(String, RecordedFlow)> = Vec::new();
    if wanted("later_unit") {
        variants.push(("later_unit".into(), flip_after(unmount.clone(), after_d, StepKind::UnitWrite, 4000, "unmount_later_unit_byte")));
    }
    if wanted("later_journal") {
        variants.push(("later_journal".into(), flip_after(unmount.clone(), after_d, StepKind::JournalRecord, 64, "unmount_later_journal_byte")));
    }
    if wanted("later_root_payload") {
        // 实例表指针（偏移 170 起）里的一个字节；根身份那几个字节（24..36）不动
        variants.push(("later_root_payload".into(), flip_after(unmount.clone(), after_d, StepKind::RootRecordFua, 200, "unmount_later_root_instance_table_pointer_byte")));
    }
    if wanted("later_sysconfig") {
        variants.push(("later_sysconfig".into(), flip_after(unmount.clone(), after_d, StepKind::SystemConfigurationSlot, 100, "unmount_later_system_configuration_byte")));
    }
    if wanted("last_root_txg_jump") {
        let mut variant = unmount.clone().renamed("unmount_last_root_txg_plus_100");
        let index = variant.writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("根槽写");
        if let WrittenContents::Bytes(bytes) = &mut variant.writes[index].contents {
            let txg = u64::from_le_bytes(bytes[28..36].try_into().expect("8")) + 100;
            bytes[28..36].copy_from_slice(&txg.to_le_bytes());
        }
        println!("J1DIFF_VARIANT name=unmount_last_root_txg_plus_100 write_index={index}");
        variants.push(("last_root_txg_jump".into(), variant));
    }
    if wanted("versions_after_d") {
        let mut variant = unmount.clone().renamed("unmount_versions_from_txg_10_miswritten");
        for version in variant.versions.iter_mut().filter(|version| version.checkpoint_txg.0 >= 10) {
            version.content[0] ^= 0x01;
        }
        variants.push(("versions_after_d".into(), variant));
    }
    if wanted("judged_root_moved") {
        let mut variant = unmount.clone().renamed("unmount_judged_root_moved_to_txg_18");
        let roots: Vec<usize> = variant.writes.iter().enumerate().filter(|(_, write)| write.kind == StepKind::RootRecordFua).map(|(index, _)| index).collect();
        variant.judged_root_index = roots[roots.len() - 2];
        variants.push(("judged_root_moved".into(), variant));
    }
    for (label, variant) in &variants {
        compare(&unmount, &mut cache, variant, label, &mut judged_states);
    }
    println!("J1DIFF_TOTAL judged_states={judged_states} threads={}", threads());
}
