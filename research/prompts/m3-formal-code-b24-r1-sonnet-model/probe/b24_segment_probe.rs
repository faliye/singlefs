//! 正推腿副本探针（m3-formal-code-b24-r1 sonnet）：只在副本的 crates/singlefs-harness/tests/ 下跑（harness 档，不碰 checker 档包）。
//! 量真历史上候选段的段号与段长：F3-d 的 A 取段号 0..=63、B 取段长 28，真历史走到多大。
//! 「候选段」同 `crash_injection.rs` 的 `draw_crash_points`：整段的写落在 mkfs 之后、最后一个跑完的操作之前。
use singlefs_harness::history::{
    execute_history_with, generate_history, HistoryDeviceWidth, HistoryExecution, HistorySeed,
    PerStepChecker,
};
use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes;
use singlefs_harness::SharedStream;
use singlefs_core::admission::SpaceAdmission;
use std::collections::BTreeMap;

#[test]
fn candidate_segment_indexes_and_lengths_of_generated_histories() {
    let histories: u64 = std::env::var("B24_HISTORIES").ok().and_then(|v| v.parse().ok()).unwrap_or(64);
    let operations: usize = std::env::var("B24_OPERATIONS").ok().and_then(|v| v.parse().ok()).unwrap_or(16);
    for (name, width) in [
        ("384", HistoryDeviceWidth::UnitAreaOf384Slots),
        ("4g", HistoryDeviceWidth::FourGibibytes),
    ] {
        let execution = HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: width,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        };
        let mut max_index = 0usize;
        let mut max_length = 0usize;
        let mut over_63 = 0usize;
        let mut total = 0usize;
        let mut histories_over_63 = 0usize;
        let mut length_histogram: BTreeMap<usize, usize> = BTreeMap::new();
        let mut max_all_segments = 0usize;
        for seed in 0..histories {
            let history = generate_history(HistorySeed(seed), operations);
            let stream = SharedStream::retaining_contents();
            let mut last = 0usize;
            let run = execute_history_with(&history, execution, &stream, &mut |_| {
                last = stream.operation_count();
            });
            let (_writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
                &stream.retained_operations(),
                &width.fixed_geometry(),
            );
            max_all_segments = max_all_segments.max(segments.len());
            let mut this_over = false;
            for (index, segment) in segments.iter().enumerate() {
                let candidate = segment.iter().all(|w| {
                    (run.operations_written_by_make_filesystem..last).contains(&stream_indexes[*w])
                });
                if !candidate {
                    continue;
                }
                total += 1;
                max_index = max_index.max(index);
                max_length = max_length.max(segment.len());
                *length_histogram.entry(segment.len()).or_insert(0) += 1;
                if index > 63 {
                    over_63 += 1;
                    this_over = true;
                }
            }
            if this_over {
                histories_over_63 += 1;
            }
        }
        println!(
            "b24-segments width={name} histories={histories} operations={operations} candidates={total} max_segment_index={max_index} \
             candidates_with_index_over_63={over_63} histories_with_such={histories_over_63} total_segments_max_in_one_history={max_all_segments} \
             longest_candidate={max_length} lengths_over_28={} histogram={length_histogram:?}",
            length_histogram.iter().filter(|(l, _)| **l > 28).map(|(_, c)| *c).sum::<usize>()
        );
    }
}
