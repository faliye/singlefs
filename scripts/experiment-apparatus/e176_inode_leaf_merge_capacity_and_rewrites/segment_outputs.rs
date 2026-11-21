//! 按线程并行跑序列、判定与汇总行、两段的整个输出。

use super::*;

/// 按序列分线程跑，每条序列喂给同一组臂变体、开同一组窗口；结果按登记次序排，与单线程逐字相同（单测盯着）。工作线程 panic（丙类断言红、作废断言红）整段停下。
#[must_use]
pub fn run_sequences(
    sequences: &[RegisteredSequence],
    arms: &[ArmVariant],
    window_triples: &[(&'static str, [usize; 3])],
    threads: usize,
    operations_per_arm_at_most: u64,
) -> Vec<SequenceResult> {
    let next_sequence = std::sync::atomic::AtomicUsize::new(0);
    let mut collected: Vec<(usize, SequenceResult)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads.max(1))
            .map(|_| {
                scope.spawn(|| {
                    let mut finished = Vec::new();
                    loop {
                        let sequence_index = next_sequence.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let Some(sequence) = sequences.get(sequence_index) else { break };
                        finished.push((sequence_index, run_sequence(*sequence, arms, window_triples, operations_per_arm_at_most)));
                    }
                    finished
                })
            })
            .collect();
        workers.into_iter().flat_map(|worker| worker.join().expect("工作线程 panic：丙类断言或作废断言红，这一段不出产物")).collect()
    });
    collected.sort_by_key(|(sequence_index, _)| *sequence_index);
    assert_eq!(collected.len(), sequences.len(), "派出去几条序列就收回几条");
    collected.into_iter().map(|(_, result)| result).collect()
}

/// 窗口指纹那一行（第十一节作废条款的读数）与 F6 那一行，两段共用。
fn window_prefix_and_drop_empty_only_reached_verdict_lines(results: &[SequenceResult]) -> Vec<String> {
    let windows: Vec<&ComparisonWindow> = results.iter().flat_map(|result| result.windows.iter()).collect();
    let equal_windows = windows
        .iter()
        .filter(|window| window.operations_identical && window.prefix_sha256[0] == window.prefix_sha256[1] && window.prefix_sha256[0] == window.prefix_sha256[2])
        .count();
    let drop_empty_only_not_reached = results.iter().filter(|result| result.trajectories[0].stop == ArmStop::ReachedOperationCap).count();
    vec![
        format!(
            "name=verdict scope=window_prefixes windows={} windows_with_equal_prefixes={equal_windows} all_window_prefixes_equal={}",
            windows.len(),
            equal_windows == windows.len()
        ),
        format!("name=verdict scope=f6_m1_reached points={} m1_not_reached_points={drop_empty_only_not_reached} m1_reached_on_every_point={}", results.len(), drop_empty_only_not_reached == 0),
    ]
}

/// 每个窗口每一行的总判定（Q1r 只在第 0 个窗口判：它比的是 M1 与 M2 主，与窗口里第三条臂无关），两段共用。
fn row_verdict_lines(results: &[SequenceResult], window_labels: &[&str]) -> Vec<String> {
    let mut lines = Vec::new();
    for (window_index, window_label) in window_labels.iter().enumerate() {
        for row in JUDGED_ROWS {
            if row == JudgedRow::CapacityRatio && window_index > 0 {
                continue;
            }
            let judged_points = results.iter().filter(|result| !point_value(result, row, window_index).blocks_judgement()).count();
            lines.push(format!(
                "name=verdict scope=row row={} window={window_label} points={} judged_points={judged_points} verdict={}",
                row.name(),
                results.len(),
                overall_verdict(results, row, window_index)
            ));
        }
    }
    lines
}

/// 敏感性行之后那一行挪门槛自证的计数：两点同判的对数、判定不同的对数、挪门槛之后转成不同的对数、两点值逐位相等挪不开的对数（单列，不并进别的计数）。
fn sensitivity_self_proof_line(sensitivity: &[String]) -> String {
    let same_pairs = sensitivity.iter().filter(|line| line.contains(" result=two_points_same")).count();
    let moved_to_unstable = sensitivity.iter().filter(|line| line.contains(" moved_result=unstable")).count();
    let equal_values = sensitivity.iter().filter(|line| line.contains(" moved_result=equal_values_cannot_be_separated")).count();
    let unstable_pairs = sensitivity.iter().filter(|line| line.contains(" result=unstable")).count();
    format!(
        "name=verdict scope=sensitivity_self_proof same_pairs={same_pairs} unstable_pairs={unstable_pairs} moved_to_unstable={moved_to_unstable} equal_values_not_separable={equal_values} every_separable_same_pair_moved_to_unstable={}",
        moved_to_unstable + equal_values == same_pairs
    )
}

/// 第二段序列跑完之后的总判定行（不带 `E7RESULT ` 前缀）：窗口指纹、F6、三个窗口每行总判定、两个旋钮的敏感性与它的挪门槛自证。
#[must_use]
pub fn segment_two_summary_lines(results: &[SequenceResult]) -> Vec<String> {
    let mut lines = window_prefix_and_drop_empty_only_reached_verdict_lines(results);
    let window_labels: Vec<&str> = SEGMENT_TWO_WINDOWS.iter().map(|(label, _)| *label).collect();
    lines.extend(row_verdict_lines(results, &window_labels));
    let sensitivity = segment_two_sensitivity_lines(results);
    let self_proof = sensitivity_self_proof_line(&sensitivity);
    lines.extend(sensitivity);
    lines.push(self_proof);
    lines
}

/// 序列跑完之后的总判定行（不带 `E7RESULT ` 前缀）：窗口指纹、F5、F6、每行总判定、敏感性与它的挪门槛自证。
#[must_use]
pub fn summary_lines(results: &[SequenceResult]) -> Vec<String> {
    let mut lines = window_prefix_and_drop_empty_only_reached_verdict_lines(results);
    let main_point_borrows: u64 = results.iter().filter(|result| result.sequence.is_main_sampling_point).map(|result| result.trajectories[2].borrows).sum();
    lines.push(format!("name=verdict scope=f5_m3_borrows_on_main_points main_point_m3_borrows={main_point_borrows} m3_borrowed_on_some_main_point={}", main_point_borrows > 0));
    let window_labels: Vec<&str> = SEGMENT_ONE_WINDOWS.iter().map(|(label, _)| *label).collect();
    lines.extend(row_verdict_lines(results, &window_labels));
    let sensitivity = sensitivity_lines(results);
    let self_proof = sensitivity_self_proof_line(&sensitivity);
    lines.extend(sensitivity);
    lines.push(self_proof);
    lines
}

/// 第一段整段：config、锚点与阳性对照（不全对就到此为止）、22 条序列、总判定。交回产物行（不带前缀）与阳性对照过没过。
#[must_use]
pub fn segment_one_output(threads: usize) -> (Vec<String>, bool) {
    let mut lines = vec![format!(
        "name=config experiment=E176 segment=one sequences={} arms={} leaf_record_capacity={LEAF_RECORD_CAPACITY} root_node_leaf_container_capacity={ROOT_NODE_LEAF_CONTAINER_CAPACITY} operations_per_arm_at_most={OPERATIONS_PER_ARM_AT_MOST} unlinks_for_a_judgement_at_least={UNLINKS_IN_THE_WINDOW_FOR_A_JUDGEMENT_AT_LEAST}",
        registered_sequences().len(),
        SEGMENT_ONE_ARMS.map(|arm| arm.name).join(",")
    )];
    let anchors_ok = push_anchor_lines(&mut lines, &anchor_readings());
    if !anchors_ok {
        return (lines, false);
    }
    let results = run_sequences(&registered_sequences(), &SEGMENT_ONE_ARMS, &SEGMENT_ONE_WINDOWS, threads, OPERATIONS_PER_ARM_AT_MOST);
    for result in &results {
        assert_window_prefixes_agree(result);
        lines.extend(sequence_lines(result));
    }
    lines.extend(summary_lines(&results));
    (lines, true)
}

/// 锚点行与阳性对照那一行判决，两段共用；交回登记了答案的格是不是全对。
fn push_anchor_lines(lines: &mut Vec<String>, readings: &[AnchorReading]) -> bool {
    for reading in readings {
        let expected = match reading.expected {
            Some(expected) if expected == reading.observed => "matches=true".to_string(),
            Some(expected) => format!("matches=false expected={}", expected.replace(' ', ",")),
            None => "matches=unregistered".to_string(),
        };
        lines.push(format!("name=anchor cell={} arm={} {} {expected}", reading.cell, reading.arm, reading.observed));
    }
    let registered = readings.iter().filter(|reading| reading.expected.is_some()).count();
    let matching = readings.iter().filter(|reading| reading.expected.is_some() && reading.matches()).count();
    let anchors_ok = registered == matching;
    lines.push(format!("name=verdict scope=positive_controls anchors_registered={registered} anchors_matching={matching} all_registered_anchors_match={anchors_ok}"));
    anchors_ok
}

/// 第二段整段（跑前登记第五节第二段那一行、修订 3）：config、锚点与阳性对照（不全对就到此为止）、22 条序列 × 六个臂变体三个窗口、总判定。
#[must_use]
pub fn segment_two_output(threads: usize) -> (Vec<String>, bool) {
    let mut lines = vec![format!(
        "name=config experiment=E176 segment=two sequences={} arms={} windows={} leaf_record_capacity={LEAF_RECORD_CAPACITY} root_node_leaf_container_capacity={ROOT_NODE_LEAF_CONTAINER_CAPACITY} operations_per_arm_at_most={OPERATIONS_PER_ARM_AT_MOST} unlinks_for_a_judgement_at_least={UNLINKS_IN_THE_WINDOW_FOR_A_JUDGEMENT_AT_LEAST}",
        registered_sequences().len(),
        SEGMENT_TWO_ARMS.map(|arm| arm.name).join(","),
        SEGMENT_TWO_WINDOWS.map(|(label, _)| label).join(",")
    )];
    let anchors_ok = push_anchor_lines(&mut lines, &segment_two_anchor_readings());
    if !anchors_ok {
        return (lines, false);
    }
    let results = run_sequences(&registered_sequences(), &SEGMENT_TWO_ARMS, &SEGMENT_TWO_WINDOWS, threads, OPERATIONS_PER_ARM_AT_MOST);
    for result in &results {
        assert_window_prefixes_agree(result);
        lines.extend(sequence_lines(result));
    }
    lines.extend(segment_two_summary_lines(&results));
    (lines, true)
}
