//! 阳性对照与格的网格、键与输出行：锚点、格、对照、判定、断言与几何输出。

use super::*;

/// B5：`anchors_q1c.py` 原样输出（登记第十三节）：(r, 条件, 臂, 执行数, 去重状态数)，多飞 k = 2、H0、读法 R2。
pub(super) const MULTI_FLIGHT_ANCHOR: [(&[u32], PublishCondition, AppendArm, u64, u64); 16] = [
    (&[1, 1], PublishCondition::MultiFlightRootsOrdered, AppendArm::JiaWhole, 3747, 56),
    (&[1, 1], PublishCondition::MultiFlightRootsOrdered, AppendArm::JiaInterleaved, 3747, 56),
    (&[1, 1], PublishCondition::MultiFlightRootsOrdered, AppendArm::YiReservedWhole, 5049, 59),
    (&[1, 1], PublishCondition::MultiFlightRootsOrdered, AppendArm::DingNoDiscipline, 5049, 80),
    (&[1, 1], PublishCondition::MultiFlightRootsUnordered, AppendArm::JiaWhole, 6435, 62),
    (&[1, 1], PublishCondition::MultiFlightRootsUnordered, AppendArm::JiaInterleaved, 6435, 62),
    (&[1, 1], PublishCondition::MultiFlightRootsUnordered, AppendArm::YiReservedWhole, 12870, 77),
    (&[1, 1], PublishCondition::MultiFlightRootsUnordered, AppendArm::DingNoDiscipline, 12870, 104),
    (&[2, 2], PublishCondition::MultiFlightRootsOrdered, AppendArm::JiaWhole, 10006, 82),
    (&[2, 2], PublishCondition::MultiFlightRootsOrdered, AppendArm::JiaInterleaved, 13366, 146),
    (&[2, 2], PublishCondition::MultiFlightRootsOrdered, AppendArm::YiReservedWhole, 19162, 91),
    (&[2, 2], PublishCondition::MultiFlightRootsOrdered, AppendArm::DingNoDiscipline, 19162, 246),
    (&[2, 2], PublishCondition::MultiFlightRootsUnordered, AppendArm::JiaWhole, 15490, 88),
    (&[2, 2], PublishCondition::MultiFlightRootsUnordered, AppendArm::JiaInterleaved, 24310, 167),
    (&[2, 2], PublishCondition::MultiFlightRootsUnordered, AppendArm::YiReservedWhole, 48620, 115),
    (&[2, 2], PublishCondition::MultiFlightRootsUnordered, AppendArm::DingNoDiscipline, 48620, 306),
];

/// B3：同发 H0 甲交错 / 对照丁的（执行数，去重状态数），`anchors_tongfa.py` 原样输出（登记第十三节）。
#[allow(clippy::type_complexity, reason = "登记那张表逐行抄下来的（回环形状，执行数，去重状态数）三元组，按位取用，命名结构体不会让哪一步更好验")]
pub(super) const SHARED_INTERLEAVED_ANCHOR: [(&[u32], (u64, u64), (u64, u64)); 6] = [
    (&[2, 1], (2, 29), (3, 39)),
    (&[1, 2], (1, 19), (3, 39)),
    (&[2, 2], (3, 59), (6, 104)),
    (&[1, 1, 1], (1, 27), (6, 73)),
    (&[2, 1, 1], (3, 67), (12, 188)),
    (&[2, 2, 2], (15, 695), (90, 3235)),
];

/// B4：`anchors_void.py` 原样输出（登记第十三节）：(r, v, 串落甲, 串落乙, 同发甲, 同发乙)。
#[allow(clippy::type_complexity, reason = "脚本原样输出的一行六列，逐行抄下来按位取用，命名结构体不会让哪一步更好验")]
pub(super) const NARROW_VOID_ANCHOR_SCRIPT: [(&[u32], u32, u64, u64, u64, u64); 10] = [
    (&[1, 1], 1, 9, 11, 9, 11),
    (&[1, 1], 2, 9, 14, 9, 11),
    (&[2, 2], 1, 11, 23, 11, 23),
    (&[2, 2], 2, 11, 18, 11, 23),
    (&[1, 1, 1], 1, 17, 19, 15, 19),
    (&[1, 1, 1], 2, 17, 19, 15, 19),
    (&[1, 1, 1], 3, 17, 22, 15, 19),
    (&[2, 2, 2], 1, 21, 33, 27, 75),
    (&[2, 2, 2], 2, 21, 33, 27, 75),
    (&[2, 2, 2], 3, 21, 28, 27, 75),
];

/// B1、B2 登记里写出数的那五格。
pub(super) const SERIAL_PUBLISH_ANCHOR_LISTED: [(&[u32], u64); 5] = [(&[1, 1], 17), (&[2, 2], 21), (&[1, 2], 19), (&[1, 1, 1], 25), (&[2, 2, 2], 31)];
pub(super) const SHARED_PUBLISH_ANCHOR_LISTED: [(&[u32], u64); 5] = [(&[1, 1], 15), (&[2, 2], 27), (&[1, 2], 19), (&[1, 1, 1], 27), (&[2, 2, 2], 83)];

pub(super) fn history_without_void(record_counts: &[u32]) -> History {
    History { record_counts: record_counts.to_vec(), voided: None }
}

pub(super) fn history_with_narrow_void(record_counts: &[u32], voided: u32) -> History {
    History { record_counts: record_counts.to_vec(), voided: Some((voided, VoidTiming::NarrowBeforeAnyWrite)) }
}

pub(super) fn grid_cell(condition: PublishCondition, append_arm: AppendArm, history: History, reading: AppendedReading) -> AppendCellResult {
    evaluate_cell(&CellRequest::main_grid(condition, append_arm, history, reading)).expect("这一格适用")
}

// ===== 输出 =====

pub(super) fn emit(emitter: &mut Emitter, body: &str) {
    println!("{}", emitter.emit_raw(body));
}

pub(super) fn yes_or_no(holds: bool) -> &'static str {
    if holds {
        "yes"
    } else {
        "no"
    }
}

fn hit_or_not(value: u64) -> &'static str {
    if value > 0 {
        "hit"
    } else {
        "not_hit"
    }
}

pub(super) fn cell_key_text(condition: PublishCondition, append_arm: AppendArm, history: &History, reading: AppendedReading) -> String {
    format!(
        "q1={} q2={} family={} k={} r={} voided={} reading={}",
        condition.label(),
        append_arm.label(),
        history.family().label(),
        history.transaction_count(),
        history.record_counts_label(),
        history.voided_label(),
        reading.label()
    )
}

pub(super) type GridKey = (PublishCondition, AppendArm, History, AppendedReading);

pub(super) fn emit_anchors(emitter: &mut Emitter) -> u64 {
    let mut checked = 0u64;
    let mut mismatches = 0u64;
    let mut report = |report_emitter: &mut Emitter, family: &str, detail: String, expected: u64, measured: u64| {
        checked += 1;
        if expected != measured {
            mismatches += 1;
        }
        emit(report_emitter, &format!("name=anchor family={family} {detail} expected={expected} measured={measured} matches={}", yes_or_no(expected == measured)));
    };
    let record_counts_without_void: Vec<Vec<u32>> = all_histories().into_iter().filter(|history| history.voided.is_none()).map(|history| history.record_counts).collect();
    for counts in &record_counts_without_void {
        let listed = SERIAL_PUBLISH_ANCHOR_LISTED.iter().find(|(listed_counts, _)| *listed_counts == counts.as_slice()).map(|(_, states)| *states);
        for arm in AppendArm::ALL {
            let measured = grid_cell(PublishCondition::SerialPublishes, arm, history_without_void(counts), AppendedReading::IssuedAndCovered).states;
            report(emitter, "B1", format!("q2={} r={} source=closed_form", arm.label(), history_without_void(counts).record_counts_label()), serial_publish_anchor_closed_form(counts), measured);
            if let Some(listed_states) = listed {
                report(emitter, "B1", format!("q2={} r={} source=listed", arm.label(), history_without_void(counts).record_counts_label()), listed_states, measured);
            }
        }
    }
    for counts in &record_counts_without_void {
        let listed = SHARED_PUBLISH_ANCHOR_LISTED.iter().find(|(listed_counts, _)| *listed_counts == counts.as_slice()).map(|(_, states)| *states);
        for arm in [AppendArm::JiaWhole, AppendArm::YiReservedWhole] {
            let measured = grid_cell(PublishCondition::SharedPublish, arm, history_without_void(counts), AppendedReading::IssuedAndCovered).states;
            report(emitter, "B2", format!("q2={} r={} source=closed_form", arm.label(), history_without_void(counts).record_counts_label()), shared_publish_anchor_closed_form(counts), measured);
            if let Some(listed_states) = listed {
                report(emitter, "B2", format!("q2={} r={} source=listed", arm.label(), history_without_void(counts).record_counts_label()), listed_states, measured);
            }
        }
    }
    for (counts, interleaved, control) in SHARED_INTERLEAVED_ANCHOR {
        for (arm, (executions, states)) in [(AppendArm::JiaInterleaved, interleaved), (AppendArm::DingNoDiscipline, control)] {
            let measured = grid_cell(PublishCondition::SharedPublish, arm, history_without_void(counts), AppendedReading::IssuedAndCovered);
            report(emitter, "B3", format!("q2={} r={} quantity=executions source=script", arm.label(), history_without_void(counts).record_counts_label()), executions, measured.executions);
            report(emitter, "B3", format!("q2={} r={} quantity=states source=script", arm.label(), history_without_void(counts).record_counts_label()), states, measured.states);
        }
    }
    for (counts, voided, serial_jia, serial_yi, shared_jia, shared_yi) in NARROW_VOID_ANCHOR_SCRIPT {
        for (condition, arm, expected) in [
            (PublishCondition::SerialPublishes, AppendArm::JiaWhole, serial_jia),
            (PublishCondition::SerialPublishes, AppendArm::YiReservedWhole, serial_yi),
            (PublishCondition::SharedPublish, AppendArm::JiaWhole, shared_jia),
            (PublishCondition::SharedPublish, AppendArm::YiReservedWhole, shared_yi),
        ] {
            let measured = grid_cell(condition, arm, history_with_narrow_void(counts, voided), AppendedReading::IssuedAndCovered).states;
            report(emitter, "B4", format!("q1={} q2={} r={} voided=T{voided} source=script", condition.label(), arm.label(), history_without_void(counts).record_counts_label()), expected, measured);
        }
    }
    for history in all_histories().into_iter().filter(|candidate| candidate.family() == HistoryFamily::NarrowVoid) {
        let (voided, _) = history.voided.expect("H1 有作废事务");
        for condition in PublishCondition::SEGMENT_ONE {
            for (arm, reserves) in [(AppendArm::JiaWhole, false), (AppendArm::YiReservedWhole, true)] {
                let measured = grid_cell(condition, arm, history.clone(), AppendedReading::IssuedAndCovered).states;
                report(
                    emitter,
                    "B4",
                    format!("q1={} q2={} r={} voided=T{voided} source=closed_form", condition.label(), arm.label(), history.record_counts_label()),
                    narrow_void_anchor_closed_form(condition, reserves, &history.record_counts, voided),
                    measured,
                );
            }
        }
    }
    emit(emitter, &format!("name=anchor_summary families=B1,B2,B3,B4 checked={checked} mismatches={mismatches}"));
    mismatches
}

pub(super) fn emit_cell(emitter: &mut Emitter, key_text: &str, result: &AppendCellResult) {
    emit(
        emitter,
        &format!(
            "name=cell {key_text} applicable=yes executions={} states={} n2a={} n2b={} strict_segment_length_peak={} in_flight_limit_states={} a7_violation_points={} pending_initial_peak={} pending_peak_after_drain={} pending_positive_closers_peak={} pending_final_peak={} pending_never_drained={} a5_differing_states={}",
            result.executions,
            result.states,
            result.incomplete_pair_states,
            result.out_of_order_commit_pattern_states,
            result.strict_segment_length_peak,
            result.in_flight_limit_states,
            result.returned_before_last_barrier_points,
            result.pending.initial_peak,
            result.pending.peak_after_drain,
            result.pending.positive_closers_peak,
            result.pending.final_peak,
            result.pending.never_drained,
            result.whole_arms_differing_states
        ),
    );
    for arm in RecoveryArm::ALL {
        let counts = &result.recovery[arm.position()];
        emit(
            emitter,
            &format!(
                "name=cell_recovery {key_text} q3={} states={} refused={} v_fsync={} v_fsync_mounted_lost={} v_fsync_refused_with_returned={} v_fsync_branch_all_appended={} v_fsync_branch_void={} v_dep={} v_dangling_returned_empty={} v_dangling_returned_nonempty={} partial_publish={} n_atom={} core_failures={} differs_from_bing0={}",
                arm.label(),
                result.states,
                counts.refused,
                counts.fsync_violations,
                counts.fsync_violations_mounted_lost,
                counts.fsync_violations_refused_with_returned,
                counts.fsync_violations_branch_every_earlier_appended,
                counts.fsync_violations_branch_earlier_passed_as_voided,
                counts.dependency_violations,
                counts.dangling_violations_returned_empty,
                counts.dangling_violations_returned_nonempty,
                counts.partial_publish,
                counts.atomicity_violations,
                counts.core_failures,
                counts.differs_from_bing0
            ),
        );
    }
}

pub(super) fn emit_controls(emitter: &mut Emitter, grid: &BTreeMap<GridKey, AppendCellResult>) -> u64 {
    let mut controls_not_positive = 0u64;
    let mut report = |report_emitter: &mut Emitter, body: String, value: u64| {
        if value == 0 {
            controls_not_positive += 1;
        }
        emit(report_emitter, &format!("name=control {body} value={value} positive={}", yes_or_no(value > 0)));
    };
    for reading in AppendedReading::ALL {
        // 5.2：每条 Q2 臂拿掉自己的纪律；对照丁本身也跑同一格。
        for (arm, discipline) in [
            (AppendArm::JiaWhole, AppendDiscipline::RemovedForPositiveControl),
            (AppendArm::JiaInterleaved, AppendDiscipline::RemovedForPositiveControl),
            (AppendArm::YiReservedWhole, AppendDiscipline::RemovedForPositiveControl),
            (AppendArm::DingNoDiscipline, AppendDiscipline::Kept),
        ] {
            let mut request = CellRequest::main_grid(PublishCondition::SharedPublish, arm, history_without_void(&[2, 2]), reading);
            request.discipline = discipline;
            let result = evaluate_cell(&request).expect("H0 适用");
            let sample = "q1=shared_publish family=H0 k=2 r=2,2";
            report(emitter, format!("quantity=Q2-1_n2a control=discipline_removed q2={} {sample} reading={}", arm.label(), reading.label()), result.incomplete_pair_states);
            report(emitter, format!("quantity=Q2-2_n2b control=discipline_removed q2={} {sample} reading={}", arm.label(), reading.label()), result.out_of_order_commit_pattern_states);
        }
        // 5.3 V_fsync：早返回，四条 Q2 臂都套，每条 Q3 臂各报。
        for arm in AppendArm::ALL {
            let mut request = CellRequest::main_grid(PublishCondition::SharedPublish, arm, history_without_void(&[1, 1]), reading);
            request.return_rule = ReturnRule::EarlyAfterOwnRecordsIssued;
            let result = evaluate_cell(&request).expect("H0 适用");
            for recovery_arm in RecoveryArm::ALL {
                report(
                    emitter,
                    format!("quantity=Q3-1_v_fsync control=early_return q2={} q3={} q1=shared_publish family=H0 k=2 r=1,1 reading={}", arm.label(), recovery_arm.label(), reading.label()),
                    result.recovery[recovery_arm.position()].fsync_violations,
                );
            }
        }
        // 5.3 V_dep：每条臂关掉「不保后面」的那一处、按记录施加。
        let mut dependency_request = CellRequest::main_grid(PublishCondition::SharedPublish, AppendArm::DingNoDiscipline, history_without_void(&[2, 2]), reading);
        dependency_request.recovery_control = RecoveryControl::KeepLaterRecordsRemoved;
        let dependency_result = evaluate_cell(&dependency_request).expect("H0 适用");
        // 5.3 N_atom：每条臂去掉提交标记的处理并去掉整次发布施加。
        let mut atomicity_request = CellRequest::main_grid(PublishCondition::SharedPublish, AppendArm::JiaWhole, history_without_void(&[2, 2]), reading);
        atomicity_request.recovery_control = RecoveryControl::CommitMarkerHandlingRemoved;
        let atomicity_result = evaluate_cell(&atomicity_request).expect("H0 适用");
        // 5.3 V_dangling：拿掉 B1、B2。
        let mut dangling_request = CellRequest::main_grid(PublishCondition::SerialPublishes, AppendArm::JiaWhole, history_without_void(&[1, 1]), reading);
        dangling_request.barriers = PublishBarriers::WithoutBarriersAfterUnitsAndRecords;
        let dangling_result = evaluate_cell(&dangling_request).expect("H0 适用");
        for recovery_arm in RecoveryArm::ALL {
            report(
                emitter,
                format!("quantity=Q3-2_v_dep control=keep_later_records_removed q2=ding_control q3={} q1=shared_publish family=H0 k=2 r=2,2 reading={}", recovery_arm.label(), reading.label()),
                dependency_result.recovery[recovery_arm.position()].dependency_violations,
            );
            report(
                emitter,
                format!("quantity=A3_n_atom control=commit_marker_handling_removed q2=jia_whole q3={} q1=shared_publish family=H0 k=2 r=2,2 reading={}", recovery_arm.label(), reading.label()),
                atomicity_result.recovery[recovery_arm.position()].atomicity_violations,
            );
            report(
                emitter,
                format!("quantity=Q3-3_v_dangling control=barriers_after_units_and_records_removed q2=jia_whole q3={} q1=serial_publishes family=H0 k=2 r=1,1 reading={}", recovery_arm.label(), reading.label()),
                dangling_result.recovery[recovery_arm.position()].dangling_violations(),
            );
        }
        // 5.3 判坏（只丙3）与部分施加（只丙2）：对照丁当 Q2 臂，主网格里那两格。
        let judged_bad = &grid[&(PublishCondition::SharedPublish, AppendArm::DingNoDiscipline, history_without_void(&[2, 1]), reading)];
        report(
            emitter,
            format!("quantity=bing3_refused control=ding_as_q2 q2=ding_control q3=bing3_forbid q1=shared_publish family=H0 k=2 r=2,1 reading={}", reading.label()),
            judged_bad.recovery[RecoveryArm::Bing3ForbidPattern.position()].refused,
        );
        let partial = &grid[&(PublishCondition::SharedPublish, AppendArm::DingNoDiscipline, history_without_void(&[2, 2]), reading)];
        report(
            emitter,
            format!("quantity=Q3-4_partial_publish control=ding_as_q2 q2=ding_control q3=bing2_drop_a_keep_b q1=shared_publish family=H0 k=2 r=2,2 reading={}", reading.label()),
            partial.recovery[RecoveryArm::Bing2DropTheIncompleteKeepTheComplete.position()].partial_publish,
        );
    }
    emit(emitter, &format!("name=control_summary controls_not_positive={controls_not_positive}"));
    controls_not_positive
}

/// 合法历史（H0 ∪ H1）上某个量的合计与为正的格数。
fn legal_sum(grid: &BTreeMap<GridKey, AppendCellResult>, condition: PublishCondition, arm: AppendArm, reading: AppendedReading, quantity: &dyn Fn(&AppendCellResult) -> u64) -> (u64, u64, u64) {
    let mut cells = 0u64;
    let mut positive_cells = 0u64;
    let mut total = 0u64;
    for ((cell_condition, cell_arm, history, cell_reading), result) in grid {
        if *cell_condition == condition && *cell_arm == arm && *cell_reading == reading && history.family().is_legal() {
            let value = quantity(result);
            cells += 1;
            positive_cells += u64::from(value > 0);
            total += value;
        }
    }
    (cells, positive_cells, total)
}

pub(super) fn emit_verdicts(emitter: &mut Emitter, grid: &BTreeMap<GridKey, AppendCellResult>, conditions: &[PublishCondition]) {
    for condition in conditions.iter().copied() {
        for arm in AppendArm::ALL {
            for reading in AppendedReading::ALL {
                let head = format!("q1={} q2={} reading={}", condition.label(), arm.label(), reading.label());
                let (cells, positive_cells, total) = legal_sum(grid, condition, arm, reading, &|result| result.incomplete_pair_states);
                emit(emitter, &format!("name=verdict quantity=Q2-1 {head} legal_cells={cells} legal_cells_positive={positive_cells} legal_states={total} judgement={}", hit_or_not(total)));
                let (out_of_order_cells, out_of_order_positive_cells, out_of_order_total) = legal_sum(grid, condition, arm, reading, &|result| result.out_of_order_commit_pattern_states);
                let pairing = if out_of_order_total > 0 { "bing3_pairing_out" } else { "bing3_pairing_stands" };
                emit(emitter, &format!("name=verdict quantity=Q2-2 {head} legal_cells={out_of_order_cells} legal_cells_positive={out_of_order_positive_cells} legal_states={out_of_order_total} judgement={pairing}"));
                for recovery_arm in RecoveryArm::ALL {
                    let position = recovery_arm.position();
                    let (fsync_violation_cells, fsync_violation_positive_cells, fsync_violation_total) = legal_sum(grid, condition, arm, reading, &|result| result.recovery[position].fsync_violations);
                    let (_, _, mounted_lost) = legal_sum(grid, condition, arm, reading, &|result| result.recovery[position].fsync_violations_mounted_lost);
                    let (_, _, refused_with_returned) = legal_sum(grid, condition, arm, reading, &|result| result.recovery[position].fsync_violations_refused_with_returned);
                    let (_, _, branch_appended) = legal_sum(grid, condition, arm, reading, &|result| result.recovery[position].fsync_violations_branch_every_earlier_appended);
                    let (_, _, branch_void) = legal_sum(grid, condition, arm, reading, &|result| result.recovery[position].fsync_violations_branch_earlier_passed_as_voided);
                    emit(
                        emitter,
                        &format!(
                            "name=verdict quantity=Q3-1 {head} q3={} legal_cells={fsync_violation_cells} legal_cells_positive={fsync_violation_positive_cells} legal_states={fsync_violation_total} mounted_lost={mounted_lost} refused_with_returned={refused_with_returned} branch_all_appended={branch_appended} branch_void={branch_void} judgement={}",
                            recovery_arm.label(),
                            hit_or_not(fsync_violation_total)
                        ),
                    );
                    let (dependency_violation_cells, dependency_violation_positive_cells, dependency_violation_total) = legal_sum(grid, condition, arm, reading, &|result| result.recovery[position].dependency_violations);
                    emit(emitter, &format!("name=verdict quantity=Q3-2 {head} q3={} legal_cells={dependency_violation_cells} legal_cells_positive={dependency_violation_positive_cells} legal_states={dependency_violation_total} judgement={}", recovery_arm.label(), hit_or_not(dependency_violation_total)));
                }
                // Q3-3：四条 Q3 臂同值 ⇒ 打中不分辨臂（F8）；不同值 ⇒ 值大的那几条臂。
                let per_arm: Vec<u64> = RecoveryArm::ALL
                    .iter()
                    .map(|recovery_arm| legal_sum(grid, condition, arm, reading, &|result| result.recovery[recovery_arm.position()].dangling_violations()).2)
                    .collect();
                let (_, cells_where_arms_differ, _) = legal_sum(grid, condition, arm, reading, &|result| {
                    let first = result.recovery[0].dangling_violations();
                    u64::from(result.recovery.iter().any(|counts| counts.dangling_violations() != first))
                });
                let largest = per_arm.iter().copied().max().unwrap_or(0);
                let judgement = if largest == 0 {
                    "all_zero".to_string()
                } else if per_arm.iter().all(|value| *value == largest) {
                    "same_nonzero_across_arms_f8".to_string()
                } else {
                    let arms: Vec<&str> = RecoveryArm::ALL.iter().filter(|recovery_arm| per_arm[recovery_arm.position()] == largest).map(|recovery_arm| recovery_arm.label()).collect();
                    format!("differs_largest={}", arms.join(","))
                };
                emit(
                    emitter,
                    &format!(
                        "name=verdict quantity=Q3-3 {head} bing0={} bing1={} bing2={} bing3={} legal_cells_where_arms_differ={cells_where_arms_differ} judgement={judgement}",
                        per_arm[0], per_arm[1], per_arm[2], per_arm[3]
                    ),
                );
                let bing2 = RecoveryArm::Bing2DropTheIncompleteKeepTheComplete.position();
                let (partial_publish_cells, partial_publish_positive_cells, partial_publish_total) = legal_sum(grid, condition, arm, reading, &|result| result.recovery[bing2].partial_publish);
                emit(emitter, &format!("name=verdict quantity=Q3-4 {head} q3=bing2_drop_a_keep_b legal_cells={partial_publish_cells} legal_cells_positive={partial_publish_positive_cells} legal_states={partial_publish_total} judgement={}", hit_or_not(partial_publish_total)));
            }
        }
    }
}

pub(super) fn emit_assertions(emitter: &mut Emitter, grid: &BTreeMap<GridKey, AppendCellResult>, conditions: &[PublishCondition], includes_real_baseline: bool) {
    // A1 真实基线：串落 × 甲整段 × 丙0，H0 与 H1（只在串落那一段的格里有）。
    let real_baseline_readings: &[AppendedReading] = if includes_real_baseline { &AppendedReading::ALL } else { &[] };
    for reading in real_baseline_readings.iter().copied() {
        let bing0 = RecoveryArm::Bing0Today.position();
        #[allow(clippy::type_complexity, reason = "每一项就是（量名，取这个量的函数），只在这个循环里遍历，命名别名不会让哪一步更好验")]
        let quantities: [(&str, &dyn Fn(&AppendCellResult) -> u64); 7] = [
            ("n2a", &|result: &AppendCellResult| result.incomplete_pair_states),
            ("n2b", &|result: &AppendCellResult| result.out_of_order_commit_pattern_states),
            ("v_fsync", &|result: &AppendCellResult| result.recovery[bing0].fsync_violations),
            ("v_dep", &|result: &AppendCellResult| result.recovery[bing0].dependency_violations),
            ("v_dangling", &|result: &AppendCellResult| result.recovery[bing0].dangling_violations()),
            ("partial_publish", &|result: &AppendCellResult| result.recovery[bing0].partial_publish),
            ("n_atom", &|result: &AppendCellResult| result.recovery[bing0].atomicity_violations),
        ];
        let mut fields = Vec::new();
        let mut nonzero_quantities = 0u64;
        let mut cells = 0u64;
        for (name, quantity) in quantities {
            let (cell_count, _, total) = legal_sum(grid, PublishCondition::SerialPublishes, AppendArm::JiaWhole, reading, quantity);
            cells = cell_count;
            nonzero_quantities += u64::from(total > 0);
            fields.push(format!("{name}={total}"));
        }
        emit(emitter, &format!("name=assertion check=A1 q1=serial_publishes q2=jia_whole q3=bing0_today reading={} legal_cells={cells} {} nonzero_quantities={nonzero_quantities}", reading.label(), fields.join(" ")));
    }
    // A3：每个格、每条 Q3 臂 N_atom = 0（合法与 H2 分开报）。
    for legal in [true, false] {
        let mut per_arm = [0u64; 4];
        let mut cells = 0u64;
        for ((_, _, history, _), result) in grid {
            if history.family().is_legal() == legal {
                cells += 1;
                for recovery_arm in RecoveryArm::ALL {
                    per_arm[recovery_arm.position()] += result.recovery[recovery_arm.position()].atomicity_violations;
                }
            }
        }
        emit(
            emitter,
            &format!(
                "name=assertion check=A3 histories={} cells={cells} bing0_n_atom={} bing1_n_atom={} bing2_n_atom={} bing3_n_atom={}",
                if legal { "legal" } else { "h2" },
                per_arm[0], per_arm[1], per_arm[2], per_arm[3]
            ),
        );
    }
    // A4：甲整段、乙整段在 H0、H1 的每一格 N2a = N2b = 0。
    for condition in conditions.iter().copied() {
        for arm in [AppendArm::JiaWhole, AppendArm::YiReservedWhole] {
            for reading in AppendedReading::ALL {
                let (cells, _, n2a) = legal_sum(grid, condition, arm, reading, &|result| result.incomplete_pair_states);
                let (_, _, n2b) = legal_sum(grid, condition, arm, reading, &|result| result.out_of_order_commit_pattern_states);
                emit(emitter, &format!("name=assertion check=A4 q1={} q2={} reading={} legal_cells={cells} n2a={n2a} n2b={n2b}", condition.label(), arm.label(), reading.label()));
            }
        }
    }
    // A5：甲整段、乙整段在 H0、H1 的每一格四条 Q3 臂恢复出的版本逐状态相同；丙0、丙1、丙3 的 V_dep 在每一格 = 0。
    for condition in conditions.iter().copied() {
        for arm in [AppendArm::JiaWhole, AppendArm::YiReservedWhole] {
            for reading in AppendedReading::ALL {
                let (cells, positive_cells, differing) = legal_sum(grid, condition, arm, reading, &|result| result.whole_arms_differing_states);
                let differing_by_arm: Vec<String> = RecoveryArm::ALL
                    .iter()
                    .map(|recovery_arm| format!("{}_differs={}", recovery_arm.label(), legal_sum(grid, condition, arm, reading, &|result| result.recovery[recovery_arm.position()].differs_from_bing0).2))
                    .collect();
                emit(
                    emitter,
                    &format!(
                        "name=assertion check=A5_versions q1={} q2={} reading={} legal_cells={cells} legal_cells_with_differing_states={positive_cells} differing_states={differing} {}",
                        condition.label(),
                        arm.label(),
                        reading.label(),
                        differing_by_arm.join(" ")
                    ),
                );
            }
        }
    }
    for condition in conditions.iter().copied() {
        for reading in AppendedReading::ALL {
            let mut fields = Vec::new();
            for recovery_arm in [RecoveryArm::Bing0Today, RecoveryArm::Bing1TruncateBeforeTheIncompleteTransaction, RecoveryArm::Bing3ForbidPattern] {
                let total: u64 = AppendArm::ALL
                    .iter()
                    .map(|arm| legal_sum(grid, condition, *arm, reading, &|result| result.recovery[recovery_arm.position()].dependency_violations).2)
                    .sum();
                fields.push(format!("{}_v_dep={total}", recovery_arm.label()));
            }
            emit(emitter, &format!("name=assertion check=A5_v_dep q1={} q2=all reading={} {}", condition.label(), reading.label(), fields.join(" ")));
        }
    }
    // A6、A7 与轨迹期末值：全部格。
    let mut longest = 0usize;
    let mut limit_states = 0u64;
    let mut returned_before_last_barrier_point_total = 0u64;
    let mut final_peak = 0u64;
    let mut never_drained = 0u64;
    let mut core_failures = 0u64;
    for result in grid.values() {
        longest = longest.max(result.strict_segment_length_peak);
        limit_states += result.in_flight_limit_states;
        returned_before_last_barrier_point_total += result.returned_before_last_barrier_points;
        final_peak = final_peak.max(result.pending.final_peak);
        never_drained += result.pending.never_drained;
        core_failures += result.recovery.iter().map(|counts| counts.core_failures).sum::<u64>();
    }
    emit(emitter, &format!("name=assertion check=A6 cells={} strict_segment_length_peak={longest} bound={STRICT_SEGMENT_LENGTH_BOUND} in_flight_limit={IN_FLIGHT_RECORD_LIMIT} in_flight_limit_states={limit_states}", grid.len()));
    emit(emitter, &format!("name=assertion check=A7 cells={} a7_violation_points={returned_before_last_barrier_point_total}", grid.len()));
    emit(emitter, &format!("name=assertion check=pending_trajectory cells={} pending_final_peak={final_peak} pending_never_drained={never_drained}", grid.len()));
    emit(emitter, &format!("name=assertion check=core_failures cells={} core_failures={core_failures}", grid.len()));
}

/// 登记 8.2：在合法格里只取某一 (k, r) 的 H0 与 H1 各格判一次。
fn point_judgement(grid: &BTreeMap<GridKey, AppendCellResult>, condition: PublishCondition, arm: AppendArm, reading: AppendedReading, record_counts: &[u32], quantity: &dyn Fn(&AppendCellResult) -> u64) -> u64 {
    grid.iter()
        .filter(|((cell_condition, cell_arm, history, cell_reading), _)| {
            *cell_condition == condition && *cell_arm == arm && *cell_reading == reading && history.family().is_legal() && history.record_counts == record_counts
        })
        .map(|(_, result)| quantity(result))
        .sum()
}

/// 第八节几何敏感性判的各量（第六节 Q2-1、Q2-2、Q3-1 至 Q3-4）。
#[allow(clippy::type_complexity, reason = "每一项就是（量名，取这个量的闭包），只在几何敏感性判里遍历，命名别名不会让哪一步更好验")]
fn geometry_quantities() -> Vec<(String, Box<dyn Fn(&AppendCellResult) -> u64>)> {
    let mut quantities: Vec<(String, Box<dyn Fn(&AppendCellResult) -> u64>)> = vec![
        ("Q2-1".to_string(), Box::new(|result: &AppendCellResult| result.incomplete_pair_states)),
        ("Q2-2".to_string(), Box::new(|result: &AppendCellResult| result.out_of_order_commit_pattern_states)),
    ];
    for recovery_arm in RecoveryArm::ALL {
        let position = recovery_arm.position();
        quantities.push((format!("Q3-1 q3={}", recovery_arm.label()), Box::new(move |result: &AppendCellResult| result.recovery[position].fsync_violations)));
        quantities.push((format!("Q3-2 q3={}", recovery_arm.label()), Box::new(move |result: &AppendCellResult| result.recovery[position].dependency_violations)));
        quantities.push((format!("Q3-3 q3={}", recovery_arm.label()), Box::new(move |result: &AppendCellResult| result.recovery[position].dangling_violations())));
    }
    let bing2 = RecoveryArm::Bing2DropTheIncompleteKeepTheComplete.position();
    quantities.push(("Q3-4 q3=bing2_drop_a_keep_b".to_string(), Box::new(move |result: &AppendCellResult| result.recovery[bing2].partial_publish)));
    quantities
}

/// 登记 8.2：G1、G2 在给定的条件里判；`includes_g4` 时另报 G4（多飞·根序对多飞·根乱，两个子形态互为方向相反的点）。
pub(super) fn emit_geometry(emitter: &mut Emitter, grid: &BTreeMap<GridKey, AppendCellResult>, conditions: &[PublishCondition], includes_multi_flight_root_order_geometry: bool) {
    #[allow(clippy::type_complexity, reason = "每一项就是（几何名，基准回环形状，对照回环形状的名单），只在这个函数里遍历，命名结构体不会让哪一步更好验")]
    let checks: [(&str, &[u32], Vec<(&str, &[u32])>); 2] = [
        ("G1", &[2, 2], vec![("r1,1", &[1, 1][..]), ("r1,1,1", &[1, 1, 1][..])]),
        ("G2", &[2, 1], vec![("r1,2", &[1, 2][..])]),
    ];
    let mut unstable = 0u64;
    let mut lines = 0u64;
    for (check, main_point, opposite_points) in &checks {
        for condition in conditions.iter().copied() {
            for arm in AppendArm::ALL {
                for reading in AppendedReading::ALL {
                    let quantities = geometry_quantities();
                    for (quantity_name, quantity) in &quantities {
                        let main_value = point_judgement(grid, condition, arm, reading, main_point, quantity.as_ref());
                        let mut fields = vec![format!("main={}", hit_or_not(main_value))];
                        let mut flips = false;
                        for (point_name, point) in opposite_points {
                            let value = point_judgement(grid, condition, arm, reading, point, quantity.as_ref());
                            flips |= (value > 0) != (main_value > 0);
                            fields.push(format!("{point_name}={}", hit_or_not(value)));
                        }
                        unstable += u64::from(flips);
                        lines += 1;
                        emit(
                            emitter,
                            &format!(
                                "name=geometry check={check} quantity={quantity_name} q1={} q2={} reading={} {} stability={}",
                                condition.label(),
                                arm.label(),
                                reading.label(),
                                fields.join(" "),
                                if flips { "unstable" } else { "two_points_agree" }
                            ),
                        );
                    }
                }
            }
        }
    }
    if includes_multi_flight_root_order_geometry {
        for arm in AppendArm::ALL {
            for reading in AppendedReading::ALL {
                for (quantity_name, quantity) in &geometry_quantities() {
                    let ordered = legal_sum(grid, PublishCondition::MultiFlightRootsOrdered, arm, reading, quantity.as_ref()).2;
                    let unordered = legal_sum(grid, PublishCondition::MultiFlightRootsUnordered, arm, reading, quantity.as_ref()).2;
                    let flips = (ordered > 0) != (unordered > 0);
                    unstable += u64::from(flips);
                    lines += 1;
                    emit(
                        emitter,
                        &format!(
                            "name=geometry check=G4 quantity={quantity_name} q2={} reading={} multi_flight_roots_ordered={} multi_flight_roots_unordered={} stability={}",
                            arm.label(),
                            reading.label(),
                            hit_or_not(ordered),
                            hit_or_not(unordered),
                            if flips { "unstable" } else { "two_points_agree" }
                        ),
                    );
                }
            }
        }
    }
    emit(emitter, &format!("name=geometry_summary lines={lines} unstable_lines={unstable}"));
}

pub(super) fn emit_wide_void_summaries(emitter: &mut Emitter, grid: &BTreeMap<GridKey, AppendCellResult>) {
    for condition in PublishCondition::SEGMENT_ONE {
        for arm in AppendArm::ALL {
            for reading in AppendedReading::ALL {
                let cells: Vec<&AppendCellResult> = grid
                    .iter()
                    .filter(|((cell_condition, cell_arm, history, cell_reading), _)| *cell_condition == condition && *cell_arm == arm && *cell_reading == reading && history.family() == HistoryFamily::WideVoid)
                    .map(|(_, result)| result)
                    .collect();
                if cells.is_empty() {
                    emit(emitter, &format!("name=h2_summary q1={} q2={} reading={} applicable=no", condition.label(), arm.label(), reading.label()));
                    continue;
                }
                let n2a: u64 = cells.iter().map(|result| result.incomplete_pair_states).sum();
                let n2b: u64 = cells.iter().map(|result| result.out_of_order_commit_pattern_states).sum();
                let mut fields = vec![format!("cells={} n2a={n2a} n2b={n2b}", cells.len())];
                for recovery_arm in RecoveryArm::ALL {
                    let mut totals = RecoveryCounts::default();
                    for result in &cells {
                        totals.absorb(&result.recovery[recovery_arm.position()]);
                    }
                    fields.push(format!(
                        "{}=v_fsync:{},v_dep:{},v_dangling:{},partial_publish:{},n_atom:{},refused:{}",
                        recovery_arm.label(),
                        totals.fsync_violations,
                        totals.dependency_violations,
                        totals.dangling_violations(),
                        totals.partial_publish,
                        totals.atomicity_violations,
                        totals.refused
                    ));
                }
                emit(emitter, &format!("name=h2_summary q1={} q2={} reading={} applicable=yes {}", condition.label(), arm.label(), reading.label(), fields.join(" ")));
            }
        }
    }
}
