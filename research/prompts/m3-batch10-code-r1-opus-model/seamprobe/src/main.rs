//! m3-batch10-code-r1 攻方腿（Opus）K4：掉电接缝在随机历史与写死的几段历史里落不落在崩溃段的段首。
//! 只跑历史、切段、数接缝；不枚举崩溃状态、不跑 checker。
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring,
    ContentChoice, ContentLength, GeneratedHistory, GenerationWeights, HistoryExecution, HistoryOperation, HistorySeed,
    HistoryStartingPoint, PerStepChecker, HistoryDeviceWidth,
};
use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes;
use singlefs_harness::SharedStream;

fn judge(label: &str, history: &GeneratedHistory, execution: HistoryExecution) -> (u64, u64) {
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(history, execution, &stream, &mut |_| {});
    let operations = stream.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(&operations, &execution.device_width.fixed_geometry());
    let mut heads = 0u64;
    let mut inside = 0u64;
    for first_stream_index in &run.first_stream_index_after_each_power_loss {
        let write_index = stream_indexes.iter().position(|s| *s >= *first_stream_index).unwrap_or(writes.len());
        let segment = segments.iter().position(|seg| seg.contains(&write_index));
        let is_head = segment.is_none_or(|s| segments[s].first() == Some(&write_index));
        let before: Vec<String> = operations[first_stream_index.saturating_sub(3)..*first_stream_index]
            .iter()
            .map(|o| format!("{:?}", o.operation.kind))
            .collect();
        let earlier_in_segment = segment.map_or(0, |s| segments[s].iter().filter(|w| **w < write_index).count());
        if is_head { heads += 1 } else { inside += 1 }
        println!(
            "SEAM {label} stream_index={first_stream_index} write_index={write_index} segment={segment:?} head={is_head} earlier_writes_in_segment={earlier_in_segment} last_ops_before={before:?} ending={:?}",
            std::mem::discriminant(&run.ending)
        );
    }
    (heads, inside)
}

fn main() {
    let execution = HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: HistoryDeviceWidth::FourGibibytes, space_admission: singlefs_core::admission::SpaceAdmission::JudgedByTheFormula };
    let mut total = (0u64, 0u64);
    let mut add = |t: (u64, u64)| { total.0 += t.0; total.1 += t.1; };
    add(judge("fixed-orphans", &history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring(), execution));
    let lengths = [
        ContentLength::Empty,
        ContentLength::InsideOneDataUnit { selector: 7 },
        ContentLength::ExactlyDataUnitPayloadCapacity,
        ContentLength::OneByteAboveDataUnitPayloadCapacity,
        ContentLength::ExactlyDataUnitBytes,
        ContentLength::SpanningDataUnitsThroughTheSequentialWrite { data_units: 3, selector: 5 },
        ContentLength::SpanningDataUnitsThroughTheSequentialWrite { data_units: 20, selector: 5 },
    ];
    for (li, length) in lengths.iter().enumerate() {
        for before in [0u64, 1, 3] {
            for after in [0u64, 2] {
                let mut operations = Vec::new();
                for k in 0..before {
                    operations.push(HistoryOperation::PublishOverwrite(ContentChoice { length: ContentLength::InsideOneDataUnit { selector: k }, fill_seed: k }));
                }
                operations.push(HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(ContentChoice { length: *length, fill_seed: 9 }));
                for k in 0..after {
                    operations.push(HistoryOperation::PublishOverwrite(ContentChoice { length: ContentLength::InsideOneDataUnit { selector: k + 3 }, fill_seed: k + 3 }));
                }
                let history = GeneratedHistory { seed: HistorySeed(1000 + li as u64), starting_point: HistoryStartingPoint::AfterFirstFile, operations };
                add(judge(&format!("fixed-len{li}-before{before}-after{after}"), &history, execution));
            }
        }
    }
    let seeds: u64 = std::env::var("SEEDS").ok().and_then(|s| s.parse().ok()).unwrap_or(200);
    let steps: usize = std::env::var("STEPS").ok().and_then(|s| s.parse().ok()).unwrap_or(30);
    for seed in 0..seeds {
        let history = generate_history_with_weights(HistorySeed(seed), steps, &GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP);
        add(judge(&format!("orphans-seed{seed}"), &history, execution));
    }
    println!("SEAMTOTAL heads={} inside={}", total.0, total.1);
}
