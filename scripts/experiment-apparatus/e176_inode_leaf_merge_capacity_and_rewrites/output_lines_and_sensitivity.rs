//! 直方图与停止与取值的文字、一条序列的输出行、灵敏度的各行。

use super::*;

fn histogram_text(histogram: &[u64; 5]) -> String {
    histogram.iter().map(u64::to_string).collect::<Vec<_>>().join("/")
}

fn stop_text(stop: ArmStop) -> String {
    match stop {
        ArmStop::Refused { refused_operation_number } => format!("stop=refused refused_operation={refused_operation_number}"),
        ArmStop::ReachedOperationCap => format!("stop=not_reached operation_cap={OPERATIONS_PER_ARM_AT_MOST}"),
    }
}

pub(super) fn value_text(point: PointValue) -> String {
    point.value.map_or_else(|| "unreadable".to_string(), |value| value.decimal(4))
}

/// 一条序列的全部产物行（不带 `E7RESULT ` 前缀）。
#[must_use]
pub fn sequence_lines(result: &SequenceResult) -> Vec<String> {
    let sequence = result.sequence;
    let mut lines = Vec::new();
    for trajectory in &result.trajectories {
        let full_tree = i128::try_from(LEAF_RECORD_CAPACITY * ROOT_NODE_LEAF_CONTAINER_CAPACITY).expect("31455");
        lines.push(format!(
            "name=arm_run sequence={} family={} main_sampling_point={} arm={} {} operations_accepted={} live_at_stop={} live_over_31455={} \
             containers_peak={} publishes_at_135={} containers_final={} fill_smallest={} fill_final={} merges={} borrows={} \
             i94_third_false_peak={} i94_third_false_positive_publishes={} i94_third_false_final={} consumed_prefix_sha256={}",
            sequence.name,
            sequence.family,
            sequence.is_main_sampling_point,
            trajectory.arm,
            stop_text(trajectory.stop),
            trajectory.operations_accepted,
            trajectory.live_inodes_at_stop,
            Rational::new(i128::from(trajectory.live_inodes_at_stop), full_tree).decimal(4),
            trajectory.containers_peak,
            trajectory.publishes_at_root_capacity,
            trajectory.containers_final,
            Rational::new(i128::from(trajectory.fill_smallest.0), i128::from(trajectory.fill_smallest.1)).decimal(4),
            Rational::new(i128::from(trajectory.fill_final.0), i128::from(trajectory.fill_final.1)).decimal(4),
            trajectory.merges,
            trajectory.borrows,
            trajectory.third_sentence_false_peak,
            trajectory.third_sentence_false_positive_publishes,
            trajectory.third_sentence_false_final,
            trajectory.consumed_prefix_sha256
        ));
    }
    for (window_index, window) in result.windows.iter().enumerate() {
        let arm_names = window.arm_indices.map(|arm_index| result.trajectories[arm_index].arm);
        let prefixes_equal = window.operations_identical && window.prefix_sha256[0] == window.prefix_sha256[1] && window.prefix_sha256[0] == window.prefix_sha256[2];
        lines.push(format!(
            "name=window sequence={} window={} arms={} operations={} unlinks={} ended_by={} prefix_sha256={} prefixes_equal={prefixes_equal}",
            sequence.name,
            window.label,
            arm_names.join(","),
            window.operations,
            window.unlinks,
            window.ended_by_arm.unwrap_or("none"),
            window.prefix_sha256.join(",")
        ));
        for (slot, rewrites) in window.rewrites.iter().enumerate() {
            let unlinks = i128::from(window.unlinks.max(1));
            lines.push(format!(
                "name=window_rewrites sequence={} window={} arm={} unlinks={} content_sum={} position_sum={} content_mean={} position_mean={} \
                 content_largest={} position_largest={} content_histogram={} position_histogram={}",
                sequence.name,
                window.label,
                arm_names[slot],
                window.unlinks,
                rewrites.content_sum,
                rewrites.position_sum,
                Rational::new(i128::from(rewrites.content_sum), unlinks).decimal(4),
                Rational::new(i128::from(rewrites.position_sum), unlinks).decimal(4),
                rewrites.content_largest,
                rewrites.position_largest,
                histogram_text(&rewrites.content_histogram),
                histogram_text(&rewrites.position_histogram)
            ));
        }
        for (pair_index, cumulative) in window.cumulative.iter().enumerate() {
            let notation = if pair_index % 2 == 0 { "position" } else { "content" };
            lines.push(format!(
                "name=cumulative_difference sequence={} window={} pair={}-M1 notation={notation} peak={} positive_single_differences={} final={}",
                sequence.name,
                window.label,
                arm_names[pair_index / 2 + 1],
                cumulative.peak,
                cumulative.positive_single_differences,
                cumulative.running
            ));
        }
        for row in JUDGED_ROWS {
            if row == JudgedRow::CapacityRatio && window_index > 0 {
                continue;
            }
            let point = point_value(result, row, window_index);
            lines.push(format!(
                "name=point_verdict sequence={} window={} row={} value={} note={} verdict={}",
                sequence.name,
                window.label,
                row.name(),
                value_text(point),
                if point.note.is_empty() { "none" } else { point.note },
                point_verdict(row, point, registered_thresholds(row))
            ));
        }
    }
    lines
}

/// 第八节「几何 / 旋钮敏感性」里第一段跑的负载旋钮：(旋钮, 主取样点或参照点, 方向相反的取样点)。W-R 按种子逐个配对。
pub const WORKLOAD_KNOB_PAIRS: [(&str, &str, &str); 17] = [
    ("insert_probability_0.95", "wr_p055_seed1", "wr_p095_seed1"),
    ("insert_probability_0.95", "wr_p055_seed2", "wr_p095_seed2"),
    ("insert_probability_0.95", "wr_p055_seed3", "wr_p095_seed3"),
    ("insert_probability_0.95", "wr_p055_seed4", "wr_p095_seed4"),
    ("insert_probability_0.95", "wr_p055_seed5", "wr_p095_seed5"),
    ("insert_probability_0.75", "wr_p055_seed1", "wr_p075_seed1"),
    ("insert_probability_0.75", "wr_p055_seed2", "wr_p075_seed2"),
    ("insert_probability_0.75", "wr_p055_seed3", "wr_p075_seed3"),
    ("insert_probability_0.75", "wr_p055_seed4", "wr_p075_seed4"),
    ("insert_probability_0.75", "wr_p055_seed5", "wr_p075_seed5"),
    ("deleted_fraction_0.95", "ws_a1000_d500_smallest_first", "ws_a1000_d950_smallest_first"),
    ("deletion_order_largest_first", "ws_a1000_d500_smallest_first", "ws_a1000_d500_largest_first"),
    ("kept_fraction_3of4", "wa_keep1of2_a1000", "wa_keep3of4_a1000"),
    ("kept_fraction_1of233", "wa_keep1of2_a2330", "wa_keep1of233_a2330"),
    ("batch_size_2330", "wa_keep1of2_a1000", "wa_keep1of2_a2330"),
    ("borrow_floor_59", "", ""),
    ("rewrite_notation_content", "", ""),
];

/// 两点各按自己那一行的规则判（记法旋钮那一对是 Q2p 对 Q2c，规则只差 Δ2c 不算「显著」一侧）；挪门槛自证按方向相反那一点的行做。
fn sensitivity_line(knob: &str, main_label: &str, opposite_label: &str, rows: (JudgedRow, JudgedRow), main: PointValue, opposite: PointValue) -> String {
    let (main_row, row) = rows;
    let registered = registered_thresholds(row);
    let main_verdict = point_verdict(main_row, main, registered);
    let opposite_verdict = point_verdict(row, opposite, registered);
    let comparable = !main.blocks_judgement() && !opposite.blocks_judgement() && main.value.is_some() && opposite.value.is_some();
    let result = if !comparable {
        "not_comparable"
    } else if main_verdict == opposite_verdict {
        "two_points_same"
    } else {
        "unstable"
    };
    let mut line = format!(
        "name=sensitivity knob={knob} row={} main={main_label} main_value={} main_verdict={main_verdict} opposite={opposite_label} opposite_value={} opposite_verdict={opposite_verdict} result={result}",
        if main_row == row { row.name().to_string() } else { format!("{}/{}", main_row.name(), row.name()) },
        value_text(main),
        value_text(opposite)
    );
    if result == "two_points_same" {
        match moved_threshold_verdicts(row, main, opposite) {
            Some((moved, moved_main, moved_opposite)) => {
                let moved_result = if moved_main == moved_opposite { "two_points_same" } else { "unstable" };
                write!(
                    line,
                    " moved_lower={} moved_upper={} moved_main_verdict={moved_main} moved_opposite_verdict={moved_opposite} moved_result={moved_result}",
                    moved.lower.decimal(6),
                    moved.upper.decimal(6)
                )
                .expect("写进 String 不会失败");
            }
            None => line.push_str(" moved_result=equal_values_cannot_be_separated"),
        }
    }
    line
}

/// 第八节的敏感性：负载旋钮按点配对；f 旋钮在每条序列上比主窗口与 f = 59 窗口（只比涉及 M3 的四行）；记法旋钮在每个主取样点上比位置记法与内容记法的同一对臂。
#[must_use]
pub fn sensitivity_lines(results: &[SequenceResult]) -> Vec<String> {
    let by_name = |name: &str| results.iter().find(|result| result.sequence.name == name).expect("旋钮配对点名的序列都在登记里");
    let mut lines = Vec::new();
    for (knob, main_name, opposite_name) in WORKLOAD_KNOB_PAIRS.iter().filter(|(_, main_name, _)| !main_name.is_empty()) {
        let (main_result, opposite_result) = (by_name(main_name), by_name(opposite_name));
        for row in JUDGED_ROWS {
            lines.push(sensitivity_line(knob, main_name, opposite_name, (row, row), point_value(main_result, row, 0), point_value(opposite_result, row, 0)));
        }
    }
    for result in results {
        for row in JUDGED_ROWS.iter().filter(|row| matches!(row, JudgedRow::WriteCost { minuend_slot: 2, .. })) {
            let main_label = format!("{}@M3", result.sequence.name);
            let opposite_label = format!("{}@M3_floor59", result.sequence.name);
            lines.push(sensitivity_line("borrow_floor_59", &main_label, &opposite_label, (*row, *row), point_value(result, *row, 0), point_value(result, *row, 1)));
        }
    }
    let notation_pairs = [(JUDGED_ROWS[1], JUDGED_ROWS[3]), (JUDGED_ROWS[2], JUDGED_ROWS[4]), (JUDGED_ROWS[5], JUDGED_ROWS[6])];
    for result in results.iter().filter(|result| result.sequence.is_main_sampling_point) {
        for (position_row, content_row) in notation_pairs {
            let main_label = format!("{}@{}", result.sequence.name, position_row.name());
            let opposite_label = format!("{}@{}", result.sequence.name, content_row.name());
            let line = sensitivity_line(
                "rewrite_notation_content",
                &main_label,
                &opposite_label,
                (position_row, content_row),
                point_value(result, position_row, 0),
                point_value(result, content_row, 0),
            );
            lines.push(line);
        }
    }
    lines
}

/// 第二段两个旋钮（第八节末两行）的敏感性：「借多少」在每条序列上比主窗口与借到刚好够下限那个窗口里涉及 M3 的四行；
/// 「先查哪一对」在每条序列上比主窗口与先查左边那个窗口的六行（Q2p、Q2c、Q3 两种记法；M2、M3 都换了先查的那一对）。
#[must_use]
pub fn segment_two_sensitivity_lines(results: &[SequenceResult]) -> Vec<String> {
    let window_index_of = |result: &SequenceResult, label: &str| -> usize {
        result.windows.iter().position(|window| window.label == label).expect("第二段的三个窗口每条序列都开")
    };
    let mut lines = Vec::new();
    for result in results {
        let (main_window, just_enough_window) = (window_index_of(result, "main"), window_index_of(result, "just_enough"));
        for row in JUDGED_ROWS.iter().filter(|row| matches!(row, JudgedRow::WriteCost { minuend_slot: 2, .. })) {
            let main_label = format!("{}@M3", result.sequence.name);
            let opposite_label = format!("{}@M3_just_enough", result.sequence.name);
            let points = (point_value(result, *row, main_window), point_value(result, *row, just_enough_window));
            lines.push(sensitivity_line("borrow_amount_just_enough", &main_label, &opposite_label, (*row, *row), points.0, points.1));
        }
    }
    for result in results {
        let (main_window, left_first_window) = (window_index_of(result, "main"), window_index_of(result, "left_first"));
        for row in JUDGED_ROWS.iter().filter(|row| matches!(row, JudgedRow::WriteCost { .. })) {
            let main_label = format!("{}@main", result.sequence.name);
            let opposite_label = format!("{}@left_first", result.sequence.name);
            let points = (point_value(result, *row, main_window), point_value(result, *row, left_first_window));
            lines.push(sensitivity_line("check_order_left_first", &main_label, &opposite_label, (*row, *row), points.0, points.1));
        }
    }
    lines
}
