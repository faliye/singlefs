//! 攻方副本探针（m3-formal-code-b24-r1 opus）：只在副本里，不进仓。挂在 crash_injection 下面当子模块，用它的私有项。
//! 照 `inject_crashes_into_history_drawing_second_crashes` 的前半段跑一段真历史、切段、用同一个 `draw_crash_points` 摆第一次崩溃，
//! 不恢复、不挂载、不跑 checker：只交回段号、段长与摆出的第一次崩溃态（二次崩溃的种子只由这几样算）。
use super::*;
use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};

#[must_use]
pub fn weights_named(name: &str) -> GenerationWeights {
    match name {
        "broad" => GenerationWeights::BROAD,
        "rollback" => GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        other => panic!("不认的比重 {other}"),
    }
}

#[must_use]
pub fn width_named(name: &str) -> HistoryDeviceWidth {
    match name {
        "384" => HistoryDeviceWidth::UnitAreaOf384Slots,
        "4g" => HistoryDeviceWidth::FourGibibytes,
        other => panic!("不认的宽度 {other}"),
    }
}

/// 一段真历史：`H` 一行（段数、候选段数、候选段的 段号:段长），之后每个摆出的第一次崩溃态一行 `P <历史种子> <段号> <段内 0/1 串>`。
/// `points` = 0 时照 `EveryProperSubsetOfShortSegments { segment_writes: 12, sampled_in_longer_segments: 64 }` 摆（稠密，不是战役的摆法）。
#[must_use]
pub fn real_history(seed: u64, operations: usize, weights: &str, width: &str, points: usize) -> String {
    let history = generate_history_with_weights(HistorySeed(seed), operations, &weights_named(weights));
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: width_named(width),
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let stream = SharedStream::retaining_contents();
    let mut marks = StreamMarks {
        after_make_filesystem: 0,
        after_the_last_finished_step: 0,
        after_each_step: Vec::new(),
    };
    let run = execute_history_with(&history, execution, &stream, &mut |observation| {
        let operations_so_far = stream.operation_count();
        marks.after_the_last_finished_step = operations_so_far;
        marks.after_each_step.push((observation.position, operations_so_far));
    });
    marks.after_make_filesystem = run.operations_written_by_make_filesystem;
    let operations_recorded = stream.retained_operations();
    let parameters = execution.device_width.parameters();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let (_writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations_recorded, &geometry);
    let step_kinds: Vec<HistoryOperationKind> =
        history.operations.iter().map(HistoryOperation::kind).collect();
    let candidates: Vec<(usize, usize)> = (0..segments.len())
        .filter(|segment_index| {
            segments[*segment_index].iter().all(|write| {
                (marks.after_make_filesystem..marks.after_the_last_finished_step)
                    .contains(&stream_indexes[*write])
            })
        })
        .map(|segment_index| (segment_index, segments[segment_index].len()))
        .collect();
    let draw = if points == 0 {
        CrashPointDraw::EveryProperSubsetOfShortSegments { segment_writes: 12, sampled_in_longer_segments: 64 }
    } else {
        CrashPointDraw::Sampled { crash_points_per_history: points }
    };
    let crash_points = draw_crash_points(history.seed, &segments, &stream_indexes, &marks, &step_kinds, draw);
    let mut out = String::new();
    let _ = writeln!(
        out,
        "H seed={seed} operations={operations} weights={weights} width={width} points={points} segments={} candidates={} cand={}",
        segments.len(),
        candidates.len(),
        candidates.iter().map(|(i, l)| format!("{i}:{l}")).collect::<Vec<_>>().join(",")
    );
    for crash_point in &crash_points {
        let bits: String = crash_point
            .persisted_within_the_segment
            .iter()
            .map(|p| if *p { '1' } else { '0' })
            .collect();
        let _ = writeln!(out, "P {seed} {} {bits}", crash_point.segment_index);
    }
    out
}
