//! m3-close-c409-r1 攻方腿的模型之二：随机历史的段按「种类串」（段内每次写的 StepKind 依次连成的串）去重之后剩多少状态。
//! 只数，不判。种类串当等价类是 D13 已定项 10 射程里「按种类串认节点够不够」那一问的读法，这里只量它能省多少，不论它对不对。
//! 用法：c409_kind_strings <种子基> <种子个数> <步数>
//! 输出：每段历史一行 name=kind_strings …；末行 name=kind_strings_union …（全部历史的段合起来按种类串去重）。

use std::collections::BTreeMap;

use singlefs_harness::history::{
    execute_history_with, generate_history, HistoryExecution, HistorySeed,
};
use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes_and_entries;
use singlefs_harness::SharedStream;

fn log2_sum(lengths: impl Iterator<Item = usize>) -> f64 {
    // log2(1 + Σ(2^k − 1))，k < 1000
    let mut total = 0.0f64; // log2(1)
    for k in lengths {
        let term = if k > 60 { k as f64 } else { ((k as f64).exp2() - 1.0).log2() };
        let (high, low) = if total > term { (total, term) } else { (term, total) };
        total = high + (1.0 + (low - high).exp2()).log2();
    }
    total
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    assert_eq!(arguments.len(), 4, "用法：c409_kind_strings <种子基> <种子个数> <步数>");
    let seed_base: u64 = arguments[1].parse().expect("种子基");
    let seed_count: u64 = arguments[2].parse().expect("种子个数");
    let operation_count: usize = arguments[3].parse().expect("步数");
    let mut union: BTreeMap<String, usize> = BTreeMap::new();
    let mut union_segments = 0usize;
    let mut union_lengths: Vec<usize> = Vec::new();
    for offset in 0..seed_count {
        let seed = seed_base.wrapping_add(offset);
        let history = generate_history(HistorySeed(seed), operation_count);
        let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
        let stream = SharedStream::retaining_contents();
        let run = execute_history_with(&history, execution, &stream, &mut |_| {});
        let operations = stream.retained_operations();
        let mkfs = run.operations_written_by_make_filesystem;
        let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
            &operations,
            &stream.entry_spans(),
            &execution.device_width.fixed_geometry(),
        );
        let mut own: BTreeMap<String, usize> = BTreeMap::new();
        let mut lengths: Vec<usize> = Vec::new();
        for segment in &segments {
            if segment.iter().any(|&write| stream_indexes[write] < mkfs) {
                continue;
            }
            let kinds: String = segment
                .iter()
                .map(|&write| format!("{:?}/{}", writes[write].kind, writes[write].device.0))
                .collect::<Vec<_>>()
                .join(",");
            lengths.push(segment.len());
            own.insert(kinds.clone(), segment.len());
            union.insert(kinds, segment.len());
            union_segments += 1;
            union_lengths.push(segment.len());
        }
        println!(
            "name=kind_strings seed={seed} operations={operation_count} segments={} distinct_kind_strings={} log2_full_two_state={:.2} log2_full_two_state_over_distinct_kind_strings={:.2}",
            lengths.len(),
            own.len(),
            log2_sum(lengths.iter().copied()),
            log2_sum(own.values().copied()),
        );
    }
    println!(
        "name=kind_strings_union histories={seed_count} operations={operation_count} segments={union_segments} distinct_kind_strings={} log2_full_two_state={:.2} log2_full_two_state_over_distinct_kind_strings={:.2}",
        union.len(),
        log2_sum(union_lengths.iter().copied()),
        log2_sum(union.values().copied()),
    );
}
