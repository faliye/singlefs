//! 正推腿探针（第 23 批代码轮第一轮 K1，F3-c）：随机历史录下来的写表按 `writes_and_segments_with_stream_indexes` 切段，
//! 段长的分布——内联用例只试到长 12 的段，看真历史里有没有更长的段（更长的段上「只折前 12 个写」这类弱指纹才分得出来）。
use std::collections::BTreeMap;

use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, generate_history, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistorySeed, PerStepChecker,
};
use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes;
use singlefs_harness::SharedStream;

#[test]
fn segment_length_histogram_over_random_histories() {
    let mut histogram: BTreeMap<usize, u64> = BTreeMap::new();
    let mut longest = (0usize, 0u64, "");
    for (width_name, width) in [
        ("384", HistoryDeviceWidth::UnitAreaOf384Slots),
        ("4GiB", HistoryDeviceWidth::FourGibibytes),
    ] {
        for seed in 0..10u64 {
            let history = generate_history(HistorySeed(seed), 16);
            let execution = HistoryExecution {
                per_step_checker: PerStepChecker::Skipped,
                device_width: width,
                space_admission: SpaceAdmission::JudgedByTheFormula,
            };
            let stream = SharedStream::retaining_contents();
            let run = execute_history_with(&history, execution, &stream, &mut |_| {});
            if !matches!(run.ending, HistoryEnding::Completed) {
                continue;
            }
            let (_writes, segments, _) = writes_and_segments_with_stream_indexes(
                &stream.retained_operations(),
                &width.fixed_geometry(),
            );
            for segment in &segments {
                *histogram.entry(segment.len()).or_insert(0) += 1;
                if segment.len() > longest.0 {
                    longest = (segment.len(), seed, width_name);
                }
            }
        }
    }
    println!("segment-length histogram (len: count) = {histogram:?}");
    println!("longest segment = {} writes (seed {}, width {})", longest.0, longest.1, longest.2);
    println!(
        "segments longer than 12 = {}",
        histogram.iter().filter(|(len, _)| **len > 12).map(|(_, c)| *c).sum::<u64>()
    );
}
