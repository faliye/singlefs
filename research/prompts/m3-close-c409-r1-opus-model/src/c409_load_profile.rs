//! m3-close-c409-r1 攻方腿的模型：随机历史当「常规层负载」时，写请求数与崩溃状态数是什么关系。
//! 只数，不判：按 crates/ 今天的录制器录下随机历史的写流，按 harness 的切段（writes_and_segments_with_stream_indexes_and_entries，
//! 与崩溃点重放同一条切段规则）切段，报每段写数，算全量两态闭式（1 + Σ(2^|段| − 1)，以 log2 报）与甲二的下界
//! （段内原地写任意子集 × COW 写全落或全不落，原地写只按两态算，第三态不加，所以是下界）。
//! 另报同一种子跑两次写流摘要是否相同（N 轮同种子是不是同一份判定）。
//! 用法：c409_load_profile <种子基> <种子个数> <步数,步数,…>
//! 输出：每段历史一行 name=load_profile …，最后一行 name=summary …。

use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_harness::history::{
    execute_history_with, generate_history, HistoryEnding, HistoryExecution, HistorySeed,
};
use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes_and_entries;
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::sha256_digest;
use singlefs_harness::SharedStream;

/// log2(2^a + 2^b)，两数都按 log2 给。
fn log2_add(a: f64, b: f64) -> f64 {
    if a == f64::NEG_INFINITY {
        return b;
    }
    if b == f64::NEG_INFINITY {
        return a;
    }
    let (high, low) = if a > b { (a, b) } else { (b, a) };
    high + (1.0 + (low - high).exp2()).log2()
}

/// log2(2^k − 1)，k ≥ 1。
fn log2_power_minus_one(k: f64) -> f64 {
    if k > 60.0 {
        k
    } else {
        (k.exp2() - 1.0).log2()
    }
}

struct Profile {
    ending: &'static str,
    mkfs_operations: usize,
    writes_after_mkfs: usize,
    segments_after_mkfs: usize,
    longest_segment: usize,
    segments_at_most_6: usize,
    segments_7_to_20: usize,
    segments_21_to_40: usize,
    segments_41_to_63: usize,
    segments_64_or_more: usize,
    log2_full_two_state: f64,
    log2_quick_tier_lower_bound: f64,
    exact_full_two_state: Option<u128>,
    stream_digest: String,
}

fn profile_of(seed: u64, operation_count: usize) -> Profile {
    let history = generate_history(HistorySeed(seed), operation_count);
    let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(&history, execution, &stream, &mut |_| {});
    let ending = match run.ending {
        HistoryEnding::Completed => "completed",
        HistoryEnding::KnownRed { .. } => "known_red",
        HistoryEnding::NewFinding { .. } => "new_finding",
    };
    let operations = stream.retained_operations();
    let mut digest_input: Vec<u8> = Vec::new();
    for retained in &operations {
        digest_input.extend_from_slice(retained.operation.to_stream_line().as_bytes());
        if let Some(bytes) = &retained.contents {
            digest_input.extend_from_slice(&sha256_digest(bytes));
        }
    }
    let stream_digest: String = sha256_digest(&digest_input)
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let mkfs = run.operations_written_by_make_filesystem;
    let geometry = execution.device_width.fixed_geometry();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &operations,
        &stream.entry_spans(),
        &geometry,
    );
    let mut profile = Profile {
        ending,
        mkfs_operations: mkfs,
        writes_after_mkfs: 0,
        segments_after_mkfs: 0,
        longest_segment: 0,
        segments_at_most_6: 0,
        segments_7_to_20: 0,
        segments_21_to_40: 0,
        segments_41_to_63: 0,
        segments_64_or_more: 0,
        log2_full_two_state: 0.0, // 全部持久那一个状态
        log2_quick_tier_lower_bound: 0.0,
        exact_full_two_state: Some(1),
        stream_digest,
    };
    for segment in &segments {
        if segment.iter().any(|&write| stream_indexes[write] < mkfs) {
            continue; // mkfs 那几段：崩溃点重放从 mkfs 之后起枚举
        }
        let length = segment.len();
        profile.segments_after_mkfs += 1;
        profile.writes_after_mkfs += length;
        profile.longest_segment = profile.longest_segment.max(length);
        match length {
            0..=6 => profile.segments_at_most_6 += 1,
            7..=20 => profile.segments_7_to_20 += 1,
            21..=40 => profile.segments_21_to_40 += 1,
            41..=63 => profile.segments_41_to_63 += 1,
            _ => profile.segments_64_or_more += 1,
        }
        let length_f = length as f64;
        profile.log2_full_two_state =
            log2_add(profile.log2_full_two_state, log2_power_minus_one(length_f));
        let in_place = segment
            .iter()
            .filter(|&&write| writes[write].kind != StepKind::UnitWrite)
            .count();
        let copy_on_write_choices = usize::from(in_place < length);
        let quick_exponent = (in_place + copy_on_write_choices) as f64;
        if quick_exponent >= 1.0 {
            profile.log2_quick_tier_lower_bound = log2_add(
                profile.log2_quick_tier_lower_bound,
                log2_power_minus_one(quick_exponent),
            );
        }
        profile.exact_full_two_state = profile.exact_full_two_state.and_then(|sum| {
            if length >= 127 {
                None
            } else {
                sum.checked_add((1u128 << length) - 1)
            }
        });
    }
    profile
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    assert_eq!(arguments.len(), 4, "用法：c409_load_profile <种子基> <种子个数> <步数,步数,…>");
    let seed_base: u64 = arguments[1].parse().expect("种子基");
    let seed_count: u64 = arguments[2].parse().expect("种子个数");
    let operation_counts: Vec<usize> = arguments[3]
        .split(',')
        .map(|text| text.parse().expect("步数"))
        .collect();
    let mut histories = 0u64;
    let mut panicked = 0u64;
    let mut over_a_thousand_writes = 0u64;
    let mut over_a_thousand_and_full_fits_in_u64 = 0u64;
    let mut determinism_checked = 0u64;
    let mut determinism_differs = 0u64;
    for &operation_count in &operation_counts {
        for offset in 0..seed_count {
            let seed = seed_base.wrapping_add(offset);
            histories += 1;
            let outcome = catch_unwind(AssertUnwindSafe(|| profile_of(seed, operation_count)));
            let profile = match outcome {
                Ok(profile) => profile,
                Err(_) => {
                    panicked += 1;
                    println!("name=load_profile seed={seed} operations={operation_count} outcome=panicked");
                    continue;
                }
            };
            // 同一种子再跑一次：写流摘要相同，就是 N 轮同种子判的是同一份状态
            if offset == 0 {
                determinism_checked += 1;
                if let Ok(again) = catch_unwind(AssertUnwindSafe(|| profile_of(seed, operation_count))) {
                    if again.stream_digest != profile.stream_digest {
                        determinism_differs += 1;
                    }
                } else {
                    determinism_differs += 1;
                }
            }
            if profile.writes_after_mkfs >= 1000 {
                over_a_thousand_writes += 1;
                if profile.log2_full_two_state < 64.0 {
                    over_a_thousand_and_full_fits_in_u64 += 1;
                }
            }
            let exact = profile
                .exact_full_two_state
                .map_or_else(|| "beyond_u128".to_string(), |count| count.to_string());
            println!(
                "name=load_profile seed={seed} operations={operation_count} outcome={} mkfs_operations={} writes_after_mkfs={} segments_after_mkfs={} longest_segment={} segments_by_length=le6:{},7-20:{},21-40:{},41-63:{},ge64:{} log2_full_two_state={:.2} full_two_state_exact={exact} log2_quick_tier_lower_bound={:.2} stream_digest={}",
                profile.ending,
                profile.mkfs_operations,
                profile.writes_after_mkfs,
                profile.segments_after_mkfs,
                profile.longest_segment,
                profile.segments_at_most_6,
                profile.segments_7_to_20,
                profile.segments_21_to_40,
                profile.segments_41_to_63,
                profile.segments_64_or_more,
                profile.log2_full_two_state,
                profile.log2_quick_tier_lower_bound,
                profile.stream_digest,
            );
        }
    }
    println!(
        "name=summary histories={histories} panicked={panicked} over_a_thousand_writes={over_a_thousand_writes} over_a_thousand_and_full_fits_in_u64={over_a_thousand_and_full_fits_in_u64} determinism_checked={determinism_checked} determinism_differs={determinism_differs}"
    );
}
