//! 攻方腿 m2-supp3-item3-code-r2（K3）的第二件量具：写死的那段历史上，把段内真子集枚举得更深，
//! 看记录核对器第二条判据（claimed_state_missing_unit）在什么样的崩溃状态上才判得红。只在副本上跑。

use singlefs_harness::crash::writes_and_segments_with_stream_indexes;
use singlefs_harness::crash_injection::{inject_crashes_into_history, CrashPointDraw};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::SharedStream;

const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
};

fn written_out_history() -> GeneratedHistory {
    let content = |selector: u64| {
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector },
            fill_seed: selector,
        })
    };
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 3 },
                fill_seed: 1,
            }),
            content(5),
            HistoryOperation::PublishWithoutUnits,
            HistoryOperation::CloseAndMountWritable,
            content(7),
        ],
    }
}

#[test]
fn probe_segments_and_deeper_enumeration_of_the_written_out_history() {
    let history = written_out_history();
    let stream = SharedStream::retaining_contents();
    let mut after_the_last_finished_step = 0usize;
    let run = execute_history_with(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        &stream,
        &mut |_observation| {
            after_the_last_finished_step = stream.operation_count();
        },
    );
    let operations = stream.retained_operations();
    let parameters = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES.device_width.parameters();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
    };
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations, &geometry);
    println!("mkfs 之后第 {} 步起；最后一个跑完的操作在第 {after_the_last_finished_step} 步", run.operations_written_by_make_filesystem);
    for (index, segment) in segments.iter().enumerate() {
        let is_candidate = segment.iter().all(|write| {
            (run.operations_written_by_make_filesystem..after_the_last_finished_step)
                .contains(&stream_indexes[*write])
        });
        let kinds: Vec<&'static str> = segment.iter().map(|w| writes[*w].kind.name()).collect();
        println!(
            "段 {index}：{} 个写，候选 {is_candidate}，种类 {kinds:?}",
            segment.len()
        );
    }

    for segment_writes in [4usize, 8, 12, 14] {
        let injection = inject_crashes_into_history(
            &history,
            UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            CrashPointDraw::EveryProperSubsetOfShortSegments {
                segment_writes,
                sampled_in_longer_segments: 8,
            },
            None,
        );
        println!(
            "枚举到段长 {segment_writes}：崩溃状态 {} 个；记录核对器 root_without_record {} 次、claimed_state_missing_unit {} 次；新发现 {} 条",
            injection.tally.crash_points,
            injection.tally.record_root_without_record,
            injection.tally.record_claimed_state_missing_unit,
            injection.new_findings.len()
        );
        for finding in &injection.new_findings {
            println!("  新发现：{:?}，{}", finding.signature, finding.crash_point.render());
        }
    }
}
