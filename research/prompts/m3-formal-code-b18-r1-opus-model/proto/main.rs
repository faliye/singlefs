//! m3-formal-code-b18-r1 攻方原型（只在草稿副本里跑）：照 18-1 两条 checker 档内联用例（靶向第二截那一条、第三截判别力那一条）的输入，
//! 经公开入口 `inject_crashes_into_history` 跑，打出它们断言的那几个数；V242u 与接线变体由副本里的环境变量开关切换
//! （B18_V242U、B18_VARIANT，见 scripts/patch_copy.py）。用法：
//!   b18-attack-proto count                 打出这段历史的段与闭式状态数（跑之前估量）
//!   b18-attack-proto t1                    靶向第二截那一条用例的输入
//!   b18-attack-proto t3 <起种子> <止种子>   第三截判别力那一条用例的输入，种子从起到止（含起不含止）逐个跑
use singlefs_checker_tier::crash_injection::{
    inject_crashes_into_history, CrashPointDraw, CrashStateStage, WritableMountPhase,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
    PerStepChecker, StepPosition,
};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments_with_stream_indexes, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn history(seed: u64) -> (GeneratedHistory, HistoryExecution) {
    (
        GeneratedHistory {
            seed: HistorySeed(seed),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 0 },
                fill_seed: 1,
            })],
        },
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
    )
}

fn writes_segments_first_overwrite_write(
    history: &GeneratedHistory,
    execution: HistoryExecution,
) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, usize) {
    let stream = SharedStream::retaining_contents();
    let mut after_starting_point = None;
    let run = execute_history_with(history, execution, &stream, &mut |observation| {
        if observation.position == StepPosition::StartingPoint {
            after_starting_point = Some(stream.operation_count());
        }
    });
    assert_eq!(run.ending, HistoryEnding::Completed);
    let after_starting_point = after_starting_point.expect("起点登记过");
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let first = stream_indexes
        .iter()
        .position(|index| *index >= after_starting_point)
        .expect("覆盖写写过东西");
    (writes, segments, first)
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    match arguments.get(1).map(String::as_str) {
        Some("count") => {
            let (history, execution) = history(0);
            let (writes, segments, first) = writes_segments_first_overwrite_write(&history, execution);
            println!(
                "writes={} segments={} first_overwrite_write={} closed_form_state_count={}",
                writes.len(),
                segments.iter().map(|s| s.len().to_string()).collect::<Vec<_>>().join("+"),
                first,
                closed_form_state_count(&segments)
            );
            for (index, segment) in segments.iter().enumerate() {
                let kinds: Vec<&str> = segment.iter().map(|w| writes[*w].kind.name()).collect();
                println!("segment {index}: {kinds:?}");
            }
        }
        Some("t1") => {
            let (history, execution) = history(0);
            let (writes, segments, first) = writes_segments_first_overwrite_write(&history, execution);
            let of_the_overwrite = &writes[first..];
            let segment_index = segments
                .iter()
                .position(|segment| segment[0] == first)
                .expect("覆盖写那次发布从一段的第一个写起");
            let withheld: Vec<usize> = of_the_overwrite
                .iter()
                .enumerate()
                .filter(|(_, write)| write.kind != StepKind::SystemConfigurationSlot)
                .map(|(index, _)| index)
                .collect();
            let injection = inject_crashes_into_history(
                &history,
                execution,
                CrashPointDraw::ExactlyOneCrashState {
                    segment_index,
                    writes_from_the_segment_on: of_the_overwrite.len(),
                    withheld_among_them: withheld.leak(),
                },
                None,
            );
            let tally = &injection.tally;
            println!(
                "t1 crash_points={} injected={} model_contents_compared={} legal={} illegal={} new_findings={}",
                injection.crash_points.len(),
                tally.crash_points_after_a_step_reading_under_injected_faults,
                tally.model_contents_compared,
                tally.writable_mount_refusals_on_the_d23_list_legal,
                tally.writable_mount_refusals_on_the_d23_list_illegal,
                injection.new_findings.len()
            );
            for finding in &injection.new_findings {
                println!("t1 finding stage={:?} {}", finding.stage, finding.render().lines().next().unwrap_or(""));
                let text = finding.render();
                println!(
                    "t1 conditions c670={} criterion_false={} read_back_outside={}",
                    text.contains("WitnessCriterionTrueOnACrashImageWithoutReadFaults"),
                    text.contains("CriterionFalseWhenRecomputedOnTheCrashImage"),
                    text.contains("ReadOnlyReadBackOutsideTheAllowedSet")
                );
            }
        }
        Some("t3") => {
            let from: u64 = arguments[2].parse().expect("起种子");
            let to: u64 = arguments[3].parse().expect("止种子");
            // 第 4 个参数是步长倍数：跑的种子是 k × 步长（k 从起到止）。不给时步长 1。
            let stride: u64 = arguments.get(4).map_or(1, |s| s.parse().expect("步长"));
            let mut seeds_with_a_hit = 0u64;
            for seed in (from..to).map(|k| k * stride) {
                let (history, execution) = history(seed);
                let injection = inject_crashes_into_history(
                    &history,
                    execution,
                    CrashPointDraw::EveryProperSubsetOfShortSegments {
                        segment_writes: 4,
                        sampled_in_longer_segments: 2,
                    },
                    None,
                );
                let tally = &injection.tally;
                let illegal = tally.remount_refusals_with_the_witness_criterion_true_without_read_faults;
                if illegal > 0 {
                    seeds_with_a_hit += 1;
                }
                let stage3_findings = injection
                    .new_findings
                    .iter()
                    .filter(|f| matches!(f.stage, CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(_)))
                    .count();
                println!(
                    "t3 seed={seed} crash_points={} injected={} overwrite_points={} second_crash_points={} row_publish_second={} remounts={} c670_judged={} c670_illegal={} stage3_new_findings={} stage2_illegal={} test3_red={}",
                    injection.crash_points.len(),
                    tally.crash_points_after_a_step_reading_under_injected_faults,
                    injection.crash_points.iter().filter(|p| p.operation_kind.is_some()).count(),
                    tally.second_crash_points,
                    tally.second_crash_points_by_phase.get(&WritableMountPhase::RowPublish).copied().unwrap_or(0),
                    tally.remounts_after_a_second_crash,
                    tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
                    illegal,
                    stage3_findings,
                    tally.writable_mount_refusals_on_the_d23_list_illegal,
                    illegal != 0
                );
            }
            println!("t3 summary seeds={} seeds_with_a_hit={seeds_with_a_hit}", to - from);
        }
        Some("rand") => {
            // b18-attack-proto rand <起 k> <止 k> <步长> <每段历史的步数> <每段历史摆的崩溃态数>：BROAD 比重的随机历史（种子 k × 步长），
            // 按种子抽崩溃态，打出第二截、第三截拒可写的计数（基线上找第三截判官误红，变体上找接线错的痕迹）。
            let from: u64 = arguments[2].parse().expect("起");
            let to: u64 = arguments[3].parse().expect("止");
            let stride: u64 = arguments[4].parse().expect("步长");
            let steps: usize = arguments[5].parse().expect("步数");
            let points: usize = arguments[6].parse().expect("崩溃态数");
            for seed in (from..to).map(|k| k * stride) {
                let history = singlefs_harness::history::generate_history(HistorySeed(seed), steps);
                let execution = HistoryExecution {
                    per_step_checker: PerStepChecker::Skipped,
                    device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
                    space_admission: SpaceAdmission::JudgedByTheFormula,
                };
                let injection = inject_crashes_into_history(
                    &history,
                    execution,
                    CrashPointDraw::Sampled { crash_points_per_history: points },
                    None,
                );
                let tally = &injection.tally;
                let kinds: Vec<String> = history.operations.iter().map(|o| format!("{:?}", o.kind())).collect();
                println!(
                    "rand seed={seed} ops={} crash_points={} injected={} second={} remounts={} not_judged={:?} s2_legal={} s2_illegal={} s3_judged={} s3_illegal={} new_findings={} known_red={}",
                    kinds.join(","),
                    injection.crash_points.len(),
                    tally.crash_points_after_a_step_reading_under_injected_faults,
                    tally.second_crash_points,
                    tally.remounts_after_a_second_crash,
                    tally.remount_idempotence_not_judged,
                    tally.writable_mount_refusals_on_the_d23_list_legal,
                    tally.writable_mount_refusals_on_the_d23_list_illegal,
                    tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
                    tally.remount_refusals_with_the_witness_criterion_true_without_read_faults,
                    injection.new_findings.len(),
                    injection.known_red_hits.len()
                );
                for finding in &injection.new_findings {
                    println!("rand finding seed={seed} stage={:?} {}", finding.stage, finding.render().lines().next().unwrap_or(""));
                    if std::env::var_os("B18_FULL").is_some() {
                        println!("{}", finding.render());
                    }
                }
            }
        }
        _ => {
            eprintln!("用法见文件头");
            std::process::exit(2);
        }
    }
}
