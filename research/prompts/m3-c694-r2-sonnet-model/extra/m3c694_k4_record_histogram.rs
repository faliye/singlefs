//! m3-c694-r2 辩方腿的额外探针（不在 rerun.sh 里）：E1 那批随机历史（384 槽、种子基起 24 段、每段 24 步、BROAD）里，每个有 journal 记录写的段有几个记录写。
//! 起法：拷进 $WORK/repo/crates/singlefs-harness/tests/ 后 `cargo test --offline --release -p singlefs-harness --test m3c694_k4_record_histogram -- --ignored --nocapture`（经内存包装）。
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryExecution, HistorySeed, PerStepChecker, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes;
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

#[test]
#[ignore = "scratch"]
fn k4_record_writes_per_record_segment() {
    let width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: width,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let mut histogram: std::collections::BTreeMap<usize, u64> = std::collections::BTreeMap::new();
    for seed_offset in 0..24u64 {
        let history = generate_history_with_weights(
            HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + seed_offset),
            24,
            &GenerationWeights::BROAD,
        );
        let stream = SharedStream::retaining_contents();
        execute_history_with(&history, execution, &stream, &mut |_| {});
        let (writes, segments, _) = writes_and_segments_with_stream_indexes(
            &stream.retained_operations(),
            &width.fixed_geometry(),
        );
        for segment in &segments {
            let records = segment
                .iter()
                .filter(|index| writes[**index].kind == StepKind::JournalRecord)
                .count();
            if records > 0 {
                *histogram.entry(records).or_insert(0) += 1;
            }
        }
    }
    println!("K4RECORDSEGMENTS record_writes_per_record_segment_histogram {histogram:?}");
}
