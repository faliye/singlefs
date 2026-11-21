use super::*;
use crate::helpers_shared_by_the_tests::{parsed, verdict_line};

fn threshold_summary(
    threshold_rows: u64,
    writable_mounts: u64,
    total_write_bytes: u64,
) -> ThresholdCellSummary {
    ThresholdCellSummary {
        cell: sweep_history_cells()[0],
        threshold_rows,
        writable_mounts,
        total_write_bytes,
        peak_chain_pages: 1,
        differential_control_holds: true,
    }
}

#[test]
fn write_amplification_divides_total_writes_by_writable_mounts_and_accepts_exactly_1_10() {
    let smallest = threshold_summary(64, 100, 1_000_000);
    let exactly = threshold_summary(365, 200, 2_200_000);
    let above = threshold_summary(369, 200, 2_200_200);
    assert_eq!(
        write_amplification_ratio_permille(&exactly, &smallest),
        1100
    );
    assert!(write_amplification_acceptable(
        1100,
        WRITE_AMPLIFICATION_ACCEPTABLE_PERMILLE
    ));
    assert_eq!(
        write_amplification_ratio_permille(&above, &smallest),
        1100,
        "千分比向下取整"
    );
    assert!(!write_amplification_acceptable(
        1101,
        WRITE_AMPLIFICATION_ACCEPTABLE_PERMILLE
    ));
    let picked = smallest_write_per_mount(&[exactly, smallest, above]).expect("有一格");
    assert_eq!(picked.threshold_rows, 64, "每次挂载 10000 字节最小");
    let mut void = smallest;
    void.differential_control_holds = false;
    assert_eq!(
        smallest_write_per_mount(&[void, exactly]).map(|cell| cell.threshold_rows),
        Some(365),
        "PC-差分 没过的格不参加"
    );
}

#[test]
fn moving_the_acceptance_boundary_between_two_real_ratios_flips_a_verdict() {
    let (doubled_boundary, flipped) =
        acceptance_boundary_moved_between_real_ratios(&[1000, 1050, 1300]).expect("挪得出");
    assert_eq!(doubled_boundary, 2050, "1000 与 1050 的中点 1025（两倍记）");
    assert_eq!(flipped, 1, "1050 从可接受转成不可接受");
    assert_eq!(
        acceptance_boundary_moved_between_real_ratios(&[1000, 1000]),
        None
    );
    assert!(chain_pages_stay_on_one_page(1));
    assert!(!chain_pages_stay_on_one_page(2));
}

/// 合成的 26 片 H5：可写挂载 θ = 64 / 365 / 369 / 738 为 128 / 730 / 738 / 1476 次（与锚点同量级、各不相同，W 除错了分母就看得出），
/// 每次挂载写字节 12000 / 10500 / 10000 / 10000，片数峰值 1 / 1 / 2 / 3；
/// 答案：只有 365 两关都过，越过上界的是 369、738，各点判定相同。
fn synthetic_history_shards(repeat_rows0: u64) -> Vec<HistoryShard> {
    synthetic_history_shards_with_the_two_differential_arms(repeat_rows0, true, true)
}

/// 同上，原臂（写、读字节之差都 0、越过那次写字节之差 > 0）与新臂（读字节之差等于读链字节数）各给各的答案，两臂互不相干。
fn synthetic_history_shards_with_the_two_differential_arms(
    repeat_rows0: u64,
    original_arm_holds: bool,
    chain_read_arm_holds: bool,
) -> Vec<HistoryShard> {
    sweep_shards()
        .into_iter()
        .map(|(shard, cell)| {
            let summary = match cell.arm {
                SweepArm::AtTheEndOfTheMountAboveTheThreshold { threshold_rows } => {
                    let (mounts, per_mount, pages) = match threshold_rows {
                        64 => (128, 12_000, 1),
                        365 => (730, 10_500, 1),
                        369 => (738, 10_000, 2),
                        _ => (1476, 10_000, 3),
                    };
                    format!("E7RESULT name=h5_cell_summary shard={shard} writable_mounts={mounts} sweeps_done=2 total_write_bytes={} peak_chain_pages={pages} positive_control_differential_below_the_threshold={original_arm_holds} positive_control_differential_at_the_first_sweep=true sweep_mount_nanoseconds=7", mounts * per_mount)
                }
                SweepArm::HandedToTheBackground => format!("E7RESULT name=h5_cell_summary shard={shard} writable_mounts=400 total_write_bytes=1 peak_chain_pages=2 positive_control_no_sweep_publish=true positive_control_rows0_plus_one_each_mount=true final_rows0=400 final_rows0_equals_the_registered_value=true"),
            };
            let rows0 = if shard.starts_with(SWEEP_REPEAT_SHARD_PREFIX) { repeat_rows0 } else { 1 };
            let mut lines = vec![
                format!("E7RESULT name=h5_config shard={shard} binary_sha256=same"),
                format!("E7RESULT name=h5_mount shard={shard} mount=1 rows0={rows0} mount_nanoseconds=3"),
            ];
            if cell.arm.threshold_rows().is_some() {
                lines.push(format!("E7RESULT name=h5_chain_read_differential shard={shard} mount=1 read_bytes_difference=32768 model_rows0=1 expected_chain_pages=1 expected_chain_read_bytes=32768 read_bytes_difference_equals_the_chain_read={chain_read_arm_holds}"));
            }
            lines.push(summary);
            (shard, cell, parsed(&lines))
        })
        .collect()
}

#[test]
fn sweep_summary_judges_the_page_reading_the_write_amplification_and_the_candidate_clause_on_answer_known_synthetic_shards(
) {
    let mut emitter = Emitter::collecting_lines();
    let end = emit_sweep_summary(&mut emitter, &synthetic_history_shards(1));
    assert_eq!(end, MergeEnd::Complete);
    let lines = emitter.into_collected_lines();
    let verdict = verdict_line(&lines, "h5_summary");
    let expected = [
        ("positive_control_no_sweep_publish", "true"),
        ("positive_control_rows0_plus_one_each_mount", "true"),
        ("control_final_rows0", "400"),
        (
            "control_final_rows0_equals_the_revised_registered_value",
            "true",
        ),
        ("differential_control_failed_cells", "0"),
        ("chain_read_differential_failed_cells", "0"),
        ("r5b_crossing_thresholds", "369,738"),
        ("points_without_a_candidate", "0"),
        ("geometry_unstable_thresholds", "none"),
        ("discrimination_self_proof_holds", "true"),
        ("repeat_of_the_first_shard_equal_outside_timing", "true"),
    ];
    for (key, value) in expected {
        assert_eq!(verdict.field(key), Some(value), "{key}");
    }
    let points: Vec<ResultLine> = parsed(&lines)
        .into_iter()
        .filter(|line| line.name() == Some("h5_point"))
        .collect();
    assert_eq!(points.len(), 6);
    assert!(points
        .iter()
        .all(|line| line.field("thresholds_passing_both") == Some("365")));
    let sixty_four = parsed(&lines)
        .into_iter()
        .find(|line| {
            line.name() == Some("h5_threshold_verdicts")
                && line.field("threshold_rows") == Some("64")
        })
        .expect("有 θ = 64 的判定行");
    assert_eq!(
        sixty_four.field("ratio_to_the_smallest_write_per_mount_permille"),
        Some("1200")
    );
    assert_eq!(sixty_four.field("r5c"), Some("not_acceptable"));
}

#[test]
fn the_chain_read_arm_and_the_original_differential_arm_are_counted_apart() {
    let verdict_of = |original_arm_holds, chain_read_arm_holds| {
        let mut emitter = Emitter::collecting_lines();
        emit_sweep_summary(
            &mut emitter,
            &synthetic_history_shards_with_the_two_differential_arms(
                1,
                original_arm_holds,
                chain_read_arm_holds,
            ),
        );
        verdict_line(&emitter.into_collected_lines(), "h5_summary")
    };
    let only_the_new_arm_fails = verdict_of(true, false);
    assert_eq!(
        only_the_new_arm_fails.field("differential_control_failed_cells"),
        Some("0")
    );
    assert_eq!(
        only_the_new_arm_fails.field("chain_read_differential_failed_cells"),
        Some("24")
    );
    let only_the_original_arm_fails = verdict_of(false, true);
    assert_eq!(
        only_the_original_arm_fails.field("differential_control_failed_cells"),
        Some("24")
    );
    assert_eq!(
        only_the_original_arm_fails.field("chain_read_differential_failed_cells"),
        Some("0")
    );
}

#[test]
fn threshold_cell_without_a_chain_read_line_counts_as_failed_for_the_new_arm() {
    let mut shards = synthetic_history_shards(1);
    let (_, _, lines) = &mut shards[0];
    lines.retain(|line| line.name() != Some("h5_chain_read_differential"));
    assert_eq!(chain_read_differential_failed_cells(&shards), 1);
}

#[test]
fn repeat_of_the_first_shard_that_differs_outside_timing_stops_the_sweep_merge() {
    let mut emitter = Emitter::collecting_lines();
    let end = emit_sweep_summary(&mut emitter, &synthetic_history_shards(2));
    assert!(matches!(end, MergeEnd::Stopped { .. }), "{end:?}");
    let verdict = verdict_line(&emitter.into_collected_lines(), "h5_summary");
    assert_eq!(verdict.field("repeat_first_differing_line"), Some("2"));
}
