use super::*;

const SERIAL: PublishCondition = PublishCondition::SerialPublishes;
const SHARED: PublishCondition = PublishCondition::SharedPublish;
const ISSUED_AND_COVERED: AppendedReading = AppendedReading::IssuedAndCovered;
const ISSUED_ONLY: AppendedReading = AppendedReading::IssuedOnly;

fn legal_histories() -> Vec<History> {
    all_histories().into_iter().filter(|history| history.family().is_legal()).collect()
}

#[test]
fn bing0_gives_the_registered_result_on_each_of_the_fourteen_fixed_sequences() {
    for sequence in fixed_sequences() {
        let (publishes, records) = bing0_on_fixed_sequence(&sequence);
        assert_eq!(publishes, sequence.expected_publishes_applied, "序列 {} 施加的发布数（由 lib.rs:{} 定）", sequence.number, sequence.deciding_line);
        assert_eq!(records, sequence.expected_records_applied, "序列 {} 施加的记录数", sequence.number);
    }
}

#[test]
fn history_families_have_twelve_thirty_two_and_sixteen_members() {
    let histories = all_histories();
    assert_eq!(histories.iter().filter(|history| history.family() == HistoryFamily::WithoutVoid).count(), 12);
    assert_eq!(histories.iter().filter(|history| history.family() == HistoryFamily::NarrowVoid).count(), 32);
    assert_eq!(histories.iter().filter(|history| history.family() == HistoryFamily::WideVoid).count(), 16);
}

#[test]
fn serial_publishes_without_void_state_counts_equal_the_serial_publish_anchor_for_every_append_arm_and_both_readings() {
    for (counts, states) in SERIAL_PUBLISH_ANCHOR_LISTED {
        for arm in AppendArm::ALL {
            for reading in AppendedReading::ALL {
                assert_eq!(grid_cell(SERIAL, arm, history_without_void(counts), reading).states, states, "B1 {arm:?} r={counts:?} {reading:?}");
            }
        }
    }
    for history in all_histories().into_iter().filter(|history| history.family() == HistoryFamily::WithoutVoid) {
        assert_eq!(grid_cell(SERIAL, AppendArm::JiaWhole, history.clone(), ISSUED_AND_COVERED).states, serial_publish_anchor_closed_form(&history.record_counts));
    }
}

#[test]
fn shared_publish_without_void_whole_arm_state_counts_equal_the_shared_publish_anchor() {
    for (counts, states) in SHARED_PUBLISH_ANCHOR_LISTED {
        for arm in [AppendArm::JiaWhole, AppendArm::YiReservedWhole] {
            assert_eq!(grid_cell(SHARED, arm, history_without_void(counts), ISSUED_AND_COVERED).states, states, "B2 {arm:?} r={counts:?}");
            assert_eq!(grid_cell(SHARED, arm, history_without_void(counts), ISSUED_ONLY).states, states, "B2 R1 {arm:?} r={counts:?}");
        }
    }
    for history in all_histories().into_iter().filter(|history| history.family() == HistoryFamily::WithoutVoid) {
        assert_eq!(grid_cell(SHARED, AppendArm::YiReservedWhole, history.clone(), ISSUED_AND_COVERED).states, shared_publish_anchor_closed_form(&history.record_counts));
    }
}

#[test]
fn shared_publish_without_void_interleaved_and_control_executions_and_states_equal_the_shared_interleaved_anchor() {
    for (counts, (interleaved_executions, interleaved_states), (control_executions, control_states)) in SHARED_INTERLEAVED_ANCHOR {
        let interleaved = grid_cell(SHARED, AppendArm::JiaInterleaved, history_without_void(counts), ISSUED_AND_COVERED);
        assert_eq!(interleaved.executions, interleaved_executions, "甲交错执行数 r={counts:?}");
        assert_eq!(interleaved.states, interleaved_states, "甲交错状态数 r={counts:?}");
        let control = grid_cell(SHARED, AppendArm::DingNoDiscipline, history_without_void(counts), ISSUED_AND_COVERED);
        assert_eq!(control.executions, control_executions, "对照丁执行数 r={counts:?}");
        assert_eq!(control.states, control_states, "对照丁状态数 r={counts:?}");
    }
}

#[test]
fn narrow_void_state_counts_equal_the_narrow_void_anchor_script_output() {
    for (counts, voided, serial_jia, serial_yi, shared_jia, shared_yi) in NARROW_VOID_ANCHOR_SCRIPT {
        assert_eq!(grid_cell(SERIAL, AppendArm::JiaWhole, history_with_narrow_void(counts, voided), ISSUED_AND_COVERED).states, serial_jia, "串落甲 r={counts:?} v={voided}");
        assert_eq!(grid_cell(SERIAL, AppendArm::YiReservedWhole, history_with_narrow_void(counts, voided), ISSUED_AND_COVERED).states, serial_yi, "串落乙 r={counts:?} v={voided}");
        assert_eq!(grid_cell(SHARED, AppendArm::JiaWhole, history_with_narrow_void(counts, voided), ISSUED_AND_COVERED).states, shared_jia, "同发甲 r={counts:?} v={voided}");
        assert_eq!(grid_cell(SHARED, AppendArm::YiReservedWhole, history_with_narrow_void(counts, voided), ISSUED_AND_COVERED).states, shared_yi, "同发乙 r={counts:?} v={voided}");
    }
}

#[test]
fn narrow_void_state_counts_equal_the_narrow_void_closed_forms_for_all_thirty_two_histories() {
    for history in all_histories().into_iter().filter(|history| history.family() == HistoryFamily::NarrowVoid) {
        let (voided, _) = history.voided.expect("H1");
        for condition in PublishCondition::SEGMENT_ONE {
            assert_eq!(grid_cell(condition, AppendArm::JiaWhole, history.clone(), ISSUED_AND_COVERED).states, narrow_void_anchor_closed_form(condition, false, &history.record_counts, voided), "{condition:?} 甲 {history:?}");
            assert_eq!(grid_cell(condition, AppendArm::YiReservedWhole, history.clone(), ISSUED_AND_COVERED).states, narrow_void_anchor_closed_form(condition, true, &history.record_counts, voided), "{condition:?} 乙 {history:?}");
        }
    }
}

#[test]
fn real_baseline_serial_jia_whole_bing0_is_zero_on_every_quantity_of_every_legal_history() {
    for history in legal_histories() {
        for reading in AppendedReading::ALL {
            let result = grid_cell(SERIAL, AppendArm::JiaWhole, history.clone(), reading);
            let bing0 = &result.recovery[RecoveryArm::Bing0Today.position()];
            assert_eq!(result.incomplete_pair_states, 0, "A1 n2a {history:?}");
            assert_eq!(result.out_of_order_commit_pattern_states, 0, "A1 n2b {history:?}");
            assert_eq!(bing0.fsync_violations, 0, "A1 v_fsync {history:?}");
            assert_eq!(bing0.dependency_violations, 0, "A1 v_dep {history:?}");
            assert_eq!(bing0.dangling_violations(), 0, "A1 v_dangling {history:?}");
            assert_eq!(bing0.partial_publish, 0, "A1 部分施加 {history:?}");
            assert_eq!(bing0.atomicity_violations, 0, "A1 n_atom {history:?}");
        }
    }
}

#[test]
fn whole_append_arms_leave_no_incomplete_pair_and_no_out_of_order_commit_pattern_on_legal_histories() {
    for condition in PublishCondition::SEGMENT_ONE {
        for arm in [AppendArm::JiaWhole, AppendArm::YiReservedWhole] {
            for history in legal_histories() {
                let result = grid_cell(condition, arm, history.clone(), ISSUED_AND_COVERED);
                assert_eq!(result.incomplete_pair_states, 0, "A4 n2a {condition:?} {arm:?} {history:?}");
                assert_eq!(result.out_of_order_commit_pattern_states, 0, "A4 n2b {condition:?} {arm:?} {history:?}");
            }
        }
    }
}

#[test]
fn no_recovery_arm_applies_part_of_a_transaction_in_any_cell() {
    for condition in PublishCondition::SEGMENT_ONE {
        for arm in AppendArm::ALL {
            for history in all_histories() {
                if let Some(result) = evaluate_cell(&CellRequest::main_grid(condition, arm, history.clone(), ISSUED_AND_COVERED)) {
                    for recovery_arm in RecoveryArm::ALL {
                        assert_eq!(result.recovery[recovery_arm.position()].atomicity_violations, 0, "A3 {condition:?} {arm:?} {history:?} {recovery_arm:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn strict_segment_never_exceeds_six_and_the_in_flight_limit_never_bites() {
    for condition in PublishCondition::SEGMENT_ONE {
        for arm in AppendArm::ALL {
            for history in all_histories() {
                if let Some(result) = evaluate_cell(&CellRequest::main_grid(condition, arm, history.clone(), ISSUED_AND_COVERED)) {
                    assert!(result.strict_segment_length_peak <= STRICT_SEGMENT_LENGTH_BOUND, "A6 {condition:?} {arm:?} {history:?}");
                    assert_eq!(result.in_flight_limit_states, 0, "A6 {condition:?} {arm:?} {history:?}");
                    assert_eq!(result.returned_before_last_barrier_points, 0, "A7 {condition:?} {arm:?} {history:?}");
                    assert_eq!(result.pending.final_peak, 0, "轨迹期末值 {condition:?} {arm:?} {history:?}");
                }
            }
        }
    }
    let longest = grid_cell(SHARED, AppendArm::DingNoDiscipline, history_without_void(&[2, 2, 2]), ISSUED_AND_COVERED).strict_segment_length_peak;
    assert_eq!(longest, 6, "k = 3、r = 2 时严格连续段取得到 6 条");
}

/// 手推：执行 [T1.1, T2.1, T1.2] 里持久集合 {U1, U2, S, jsn1, jsn2} 是唯一满足模式 P 的状态，丙3 判坏它。
#[test]
fn shared_interleaved_two_one_has_exactly_one_out_of_order_commit_pattern_state_which_bing3_refuses() {
    let result = grid_cell(SHARED, AppendArm::JiaInterleaved, history_without_void(&[2, 1]), ISSUED_AND_COVERED);
    assert_eq!(result.out_of_order_commit_pattern_states, 1, "N2b");
    assert_eq!(result.incomplete_pair_states, 0, "N2a");
    assert_eq!(result.recovery[RecoveryArm::Bing3ForbidPattern.position()].refused, 1, "丙3 判坏数");
    let layout = plan_cell(SHARED, AppendArm::JiaInterleaved, &history_without_void(&[2, 1]), AppendDiscipline::Kept);
    let executions = build_executions(&layout, AppendArm::JiaInterleaved, AppendDiscipline::Kept, PublishBarriers::AllThree);
    let mut states = HashSet::new();
    for execution in &executions {
        collect_crash_states(execution, ISSUED_AND_COVERED, ReturnRule::AfterOwnPublishLastBarrier, &mut states);
    }
    let target = states
        .iter()
        .find(|state| {
            let view = StateView::of(state);
            view.records_by_jsn.len() == 2
                && view.records_by_jsn[0].jsn == 1
                && view.records_by_jsn[0].transaction == 1
                && view.records_by_jsn[1].jsn == 2
                && view.records_by_jsn[1].transaction == 2
                && view.records_by_jsn[1].is_commit
        })
        .expect("登记第九节 M13 那个状态在");
    let view = StateView::of(target);
    assert!(!recover(&view, RecoveryArm::Bing3ForbidPattern, RecoveryControl::NoControl).mounted, "丙3 判坏");
}

/// 手推：甲交错 r = (2, 2) 的三个执行里前两条属于不同事务的是 1212、1221；段 = {jsn1, jsn2} 时持久集合是
/// {1, 2}（两个执行内容相同）或 {1, 2, 4}（jsn4 在 1212 里是 T2.2、在 1221 里是 T1.2，内容不同）⇒ 3 个；
/// 满足模式 P 的是两个执行各自的 {jsn1, jsn2, jsn3}（jsn4 不持久），内容不同，共 2 个。
#[test]
fn shared_interleaved_two_two_has_three_incomplete_pair_states_and_two_out_of_order_commit_pattern_states() {
    let result = grid_cell(SHARED, AppendArm::JiaInterleaved, history_without_void(&[2, 2]), ISSUED_AND_COVERED);
    assert_eq!(result.incomplete_pair_states, 3);
    assert_eq!(result.out_of_order_commit_pattern_states, 2);
}

/// 手推：对照丁 r = (2, 2) 里两个半截事务同在段里，只能是段 = {jsn1, jsn2} 且两条属于不同事务（前两条是 12 或 21）；
/// 每种前两条下持久集合 {1, 2} 一个、{1, 2, 4} 按 jsn4 是谁的第二条两个 ⇒ 2 × 3 = 6。
#[test]
fn shared_control_two_two_has_six_incomplete_pair_states() {
    assert_eq!(grid_cell(SHARED, AppendArm::DingNoDiscipline, history_without_void(&[2, 2]), ISSUED_AND_COVERED).incomplete_pair_states, 6);
}

#[test]
fn append_discipline_positive_controls_are_positive_for_every_append_arm() {
    for arm in [AppendArm::JiaWhole, AppendArm::JiaInterleaved, AppendArm::YiReservedWhole] {
        let mut request = CellRequest::main_grid(SHARED, arm, history_without_void(&[2, 2]), ISSUED_AND_COVERED);
        request.discipline = AppendDiscipline::RemovedForPositiveControl;
        let result = evaluate_cell(&request).expect("H0");
        assert!(result.incomplete_pair_states > 0, "{arm:?} 拿掉纪律后 N2a");
        assert!(result.out_of_order_commit_pattern_states > 0, "{arm:?} 拿掉纪律后 N2b");
    }
}

/// 手推（登记 5.3 V_fsync 早返回，同发甲整段 r = (1, 1)）：R2 下 T1 在 jsn1 发出后就返回、T2 要等 B2，
/// 记录段里返回集合 {T1} 的 4 个持久子集里 3 个丢 T1；R1 下另有返回集合 {T1, T2} 的 3 个丢失状态与 {T1} 的 2 个，共 5。
#[test]
fn early_return_control_loses_returned_transactions_in_three_states_under_the_issued_and_covered_reading_and_five_under_the_issued_reading() {
    for (reading, expected) in [(ISSUED_AND_COVERED, 3u64), (ISSUED_ONLY, 5u64)] {
        let mut request = CellRequest::main_grid(SHARED, AppendArm::JiaWhole, history_without_void(&[1, 1]), reading);
        request.return_rule = ReturnRule::EarlyAfterOwnRecordsIssued;
        let result = evaluate_cell(&request).expect("H0");
        assert_eq!(result.recovery[RecoveryArm::Bing0Today.position()].fsync_violations, expected, "{reading:?}");
    }
}

#[test]
fn early_return_control_is_positive_for_every_append_arm_recovery_arm_and_reading() {
    for arm in AppendArm::ALL {
        for reading in AppendedReading::ALL {
            let mut request = CellRequest::main_grid(SHARED, arm, history_without_void(&[1, 1]), reading);
            request.return_rule = ReturnRule::EarlyAfterOwnRecordsIssued;
            let result = evaluate_cell(&request).expect("H0");
            for recovery_arm in RecoveryArm::ALL {
                assert!(result.recovery[recovery_arm.position()].fsync_violations > 0, "{arm:?} {reading:?} {recovery_arm:?}");
            }
        }
    }
}

/// 手推（拿掉 B1、B2，串落甲整段 r = (1, 1)）：第一次发布那一段 F1 持久而 U1、S1 缺一的 6 个，
/// 第二次发布那一段 F2 持久而 U2、S2 缺一的 6 个（返回集合 {T1}）。
#[test]
fn dropping_the_unit_and_record_barriers_dangles_six_plus_six_states_under_bing0() {
    let mut request = CellRequest::main_grid(SERIAL, AppendArm::JiaWhole, history_without_void(&[1, 1]), ISSUED_AND_COVERED);
    request.barriers = PublishBarriers::WithoutBarriersAfterUnitsAndRecords;
    let result = evaluate_cell(&request).expect("H0");
    let bing0 = &result.recovery[RecoveryArm::Bing0Today.position()];
    assert_eq!(bing0.dangling_violations_returned_empty, 6);
    assert_eq!(bing0.dangling_violations_returned_nonempty, 6);
    for recovery_arm in RecoveryArm::ALL {
        assert!(result.recovery[recovery_arm.position()].dangling_violations() > 0, "{recovery_arm:?}");
    }
}

#[test]
fn dependency_and_atomicity_controls_are_positive_for_every_recovery_arm() {
    let mut dependency = CellRequest::main_grid(SHARED, AppendArm::DingNoDiscipline, history_without_void(&[2, 2]), ISSUED_AND_COVERED);
    dependency.recovery_control = RecoveryControl::KeepLaterRecordsRemoved;
    let dependency_result = evaluate_cell(&dependency).expect("H0");
    let mut atomicity = CellRequest::main_grid(SHARED, AppendArm::JiaWhole, history_without_void(&[2, 2]), ISSUED_AND_COVERED);
    atomicity.recovery_control = RecoveryControl::CommitMarkerHandlingRemoved;
    let atomicity_result = evaluate_cell(&atomicity).expect("H0");
    for recovery_arm in RecoveryArm::ALL {
        assert!(dependency_result.recovery[recovery_arm.position()].dependency_violations > 0, "V_dep 对照 {recovery_arm:?}");
        assert!(atomicity_result.recovery[recovery_arm.position()].atomicity_violations > 0, "N_atom 对照 {recovery_arm:?}");
    }
}

/// 手推：同发甲整段 r = (2, 2) 里 T1 两条持久、T2 不全的 3 个状态，丙2 施加 T1 的两条（一次发布的一部分），
/// 丙0 一条不施加 ⇒ 部分施加 3、与丙0 不同的状态 3；r = (1, 1) 时只有 {jsn1} 那 1 个。
#[test]
fn bing2_applies_part_of_a_shared_publish_in_three_states_at_two_two_and_one_at_one_one() {
    let two_two = grid_cell(SHARED, AppendArm::JiaWhole, history_without_void(&[2, 2]), ISSUED_AND_COVERED);
    assert_eq!(two_two.recovery[RecoveryArm::Bing2DropTheIncompleteKeepTheComplete.position()].partial_publish, 3);
    assert_eq!(two_two.recovery[RecoveryArm::Bing2DropTheIncompleteKeepTheComplete.position()].differs_from_bing0, 3);
    assert_eq!(two_two.whole_arms_differing_states, 3);
    let one_one = grid_cell(SHARED, AppendArm::JiaWhole, history_without_void(&[1, 1]), ISSUED_AND_COVERED);
    assert_eq!(one_one.recovery[RecoveryArm::Bing2DropTheIncompleteKeepTheComplete.position()].partial_publish, 1);
}

#[test]
fn bing0_bing1_and_bing3_never_apply_part_of_a_publish() {
    for condition in PublishCondition::SEGMENT_ONE {
        for arm in AppendArm::ALL {
            for history in all_histories() {
                if let Some(result) = evaluate_cell(&CellRequest::main_grid(condition, arm, history.clone(), ISSUED_AND_COVERED)) {
                    for recovery_arm in [RecoveryArm::Bing0Today, RecoveryArm::Bing1TruncateBeforeTheIncompleteTransaction, RecoveryArm::Bing3ForbidPattern] {
                        assert_eq!(result.recovery[recovery_arm.position()].partial_publish, 0, "{condition:?} {arm:?} {history:?} {recovery_arm:?}");
                    }
                }
            }
        }
    }
}

/// 登记 8.2 判别力自证：把 Q2-1 的门槛从「> 0」挪到主取样点上的实测值，那一格由「打中」转「没打中」。
#[test]
fn moving_the_incomplete_pair_threshold_to_the_main_point_reading_flips_hit_to_not_hit() {
    let measured = grid_cell(SHARED, AppendArm::JiaInterleaved, history_without_void(&[2, 2]), ISSUED_AND_COVERED).incomplete_pair_states;
    let judge = |value: u64, threshold: u64| value > threshold;
    assert!(judge(measured, 0), "门槛 > 0 时打中");
    assert!(!judge(measured, measured), "门槛挪到实测值时没打中");
}

/// 等价变异留档（登记第九节末段）：单实例、没有残留记录，`lib.rs:452` 那一判在模型里恒不断，删它的变异在所有输入上同值。
#[test]
fn back_chain_is_never_broken_in_a_single_instance_without_residue() {
    let layout = plan_cell(SHARED, AppendArm::DingNoDiscipline, &history_without_void(&[2, 2, 2]), AppendDiscipline::Kept);
    for execution in build_executions(&layout, AppendArm::DingNoDiscipline, AppendDiscipline::Kept, PublishBarriers::AllThree) {
        for operation in &execution.operations {
            if let Some(WriteContent::JournalRecord(record)) = operation.written_content() {
                assert!(!back_chain_is_broken(&record));
            }
        }
    }
}

#[test]
fn issued_and_issued_and_covered_readings_give_the_same_counts_on_serial_and_shared_publishes() {
    for condition in PublishCondition::SEGMENT_ONE {
        for arm in AppendArm::ALL {
            for history in all_histories() {
                let under_the_issued_and_covered_reading = evaluate_cell(&CellRequest::main_grid(condition, arm, history.clone(), ISSUED_AND_COVERED));
                let under_the_issued_reading = evaluate_cell(&CellRequest::main_grid(condition, arm, history.clone(), ISSUED_ONLY));
                if let (Some(issued_and_covered_result), Some(issued_result)) = (under_the_issued_and_covered_reading, under_the_issued_reading) {
                    assert_eq!(issued_and_covered_result.states, issued_result.states, "{condition:?} {arm:?} {history:?}");
                    assert_eq!(issued_and_covered_result.recovery, issued_result.recovery, "{condition:?} {arm:?} {history:?}");
                }
            }
        }
    }
}

// ===== 第二段（多飞） =====

const ROOTS_ORDERED: PublishCondition = PublishCondition::MultiFlightRootsOrdered;
const ROOTS_UNORDERED: PublishCondition = PublishCondition::MultiFlightRootsUnordered;

fn multi_flight(condition: PublishCondition, arm: AppendArm, history: History, reading: AppendedReading, enumeration: MultiFlightEnumeration) -> (EnumeratedCrashStates, AppendCellResult) {
    let request = CellRequest::main_grid(condition, arm, history.clone(), reading);
    let layout = plan_cell(condition, arm, &history, AppendDiscipline::Kept);
    multi_flight_cell_result(&request, &layout, enumeration)
}

#[test]
fn segment_two_runs_six_histories_without_void_and_fourteen_with_narrow_void() {
    let histories = segment_two_histories();
    assert_eq!(histories.iter().filter(|history| history.family() == HistoryFamily::WithoutVoid).count(), 6);
    assert_eq!(histories.iter().filter(|history| history.family() == HistoryFamily::NarrowVoid).count(), 14);
    assert_eq!(histories.len(), 20);
}

/// B5（`anchors_q1c.py`）：16 格的执行数与去重状态数，直接枚举与按配置去重两条路径各对一遍。
/// 取样点覆盖 M14（根序 甲整段 r = (1, 1)：3747 → 6435）与 M15（根乱 乙整段 r = (1, 1)：77 → 81）。
#[test]
fn multi_flight_without_void_executions_and_states_equal_the_multi_flight_anchor_on_both_enumerations() {
    for (counts, condition, arm, executions, states) in MULTI_FLIGHT_ANCHOR {
        for enumeration in [MultiFlightEnumeration::DirectExecutions, MultiFlightEnumeration::ByConfiguration] {
            let (_, result) = multi_flight(condition, arm, history_without_void(counts), ISSUED_AND_COVERED, enumeration);
            assert_eq!(result.executions, executions, "B5 执行数 {condition:?} {arm:?} r={counts:?} {enumeration:?}");
            assert_eq!(result.states, states, "B5 状态数 {condition:?} {arm:?} r={counts:?} {enumeration:?}");
        }
    }
}

/// 按配置去重这条路径的校验：k = 2 的合法历史上与直接枚举逐格比状态集合、执行数、轨迹与各量（H0 r = (2, 2) 已由 B5 两条路径各对锚点）。
#[test]
fn configuration_search_matches_direct_enumeration_on_two_transaction_multi_flight_histories() {
    let mut compared = 0u32;
    for condition in PublishCondition::SEGMENT_TWO {
        for arm in AppendArm::ALL {
            for history in segment_two_histories().into_iter().filter(|history| history.transaction_count() == 2) {
                let readings: &[AppendedReading] = if history.family() == HistoryFamily::NarrowVoid || history.record_counts == [2, 1] { &AppendedReading::ALL } else { &[ISSUED_AND_COVERED] };
                if history.family() == HistoryFamily::WithoutVoid && history.record_counts == [2, 2] {
                    continue;
                }
                for reading in readings.iter().copied() {
                    let (direct, direct_result) = multi_flight(condition, arm, history.clone(), reading, MultiFlightEnumeration::DirectExecutions);
                    let (searched, searched_result) = multi_flight(condition, arm, history.clone(), reading, MultiFlightEnumeration::ByConfiguration);
                    let label = format!("{condition:?} {arm:?} {history:?} {reading:?}");
                    assert!(direct.states == searched.states, "状态集合 {label}");
                    assert_eq!(direct_result.executions, searched_result.executions, "执行数 {label}");
                    assert_eq!(direct_result.pending, searched_result.pending, "轨迹 {label}");
                    assert_eq!(direct_result.recovery, searched_result.recovery, "各量 {label}");
                    assert_eq!(direct_result.returned_before_last_barrier_points, 0, "A7 直接枚举 {label}");
                    assert_eq!(searched_result.returned_before_last_barrier_points, 0, "A7 按配置 {label}");
                    compared += 1;
                }
            }
        }
    }
    // 2 个条件 × 4 条臂 ×（H0 r = (1, 1)、(1, 2) 各 1 读 + H0 r = (2, 1) 2 读 + H1 8 种 × 2 读）= 8 × 20
    assert_eq!(compared, 160);
}

/// 根乱下乙整段与对照丁没有任何跨发布的次序约束：执行数 = 各发布操作序列的多项式系数（k = 3：24! / 8!³、25! / (9! 8! 8!)）。
#[test]
fn configuration_search_counts_every_merge_when_no_cross_publish_order_applies() {
    for arm in [AppendArm::YiReservedWhole, AppendArm::DingNoDiscipline] {
        let (_, one_one_one) = multi_flight(ROOTS_UNORDERED, arm, history_without_void(&[1, 1, 1]), ISSUED_AND_COVERED, MultiFlightEnumeration::ByConfiguration);
        assert_eq!(one_one_one.executions, 9_465_511_770, "{arm:?} r=(1,1,1)");
        let (_, two_one_one) = multi_flight(ROOTS_UNORDERED, arm, history_without_void(&[2, 1, 1]), ISSUED_AND_COVERED, MultiFlightEnumeration::ByConfiguration);
        assert_eq!(two_one_one.executions, 26_293_088_250, "{arm:?} r=(2,1,1)");
        assert!(two_one_one.configurations.expect("按配置去重报配置数") < CONFIGURATION_LIMIT);
    }
}

/// 多飞下也量得出 V_fsync：早返回（记录发出就返回）在根序、甲整段、r = (1, 1) 上每条 Q3 臂都 > 0。
#[test]
fn early_return_control_is_positive_under_multi_flight_for_every_recovery_arm() {
    for condition in PublishCondition::SEGMENT_TWO {
        let mut request = CellRequest::main_grid(condition, AppendArm::JiaWhole, history_without_void(&[1, 1]), ISSUED_AND_COVERED);
        request.return_rule = ReturnRule::EarlyAfterOwnRecordsIssued;
        let result = evaluate_cell(&request).expect("H0 适用");
        for recovery_arm in RecoveryArm::ALL {
            assert!(result.recovery[recovery_arm.position()].fsync_violations > 0, "{condition:?} {recovery_arm:?}");
        }
    }
}

/// 根序对根乱：去掉 F、C 的次序约束，执行只多不少，而且根序的每个崩溃状态在根乱里都有。
#[test]
fn roots_unordered_states_contain_every_roots_ordered_state() {
    for arm in AppendArm::ALL {
        for counts in [&[1u32, 1][..], &[2, 1][..]] {
            let (ordered, ordered_result) = multi_flight(ROOTS_ORDERED, arm, history_without_void(counts), ISSUED_AND_COVERED, MultiFlightEnumeration::DirectExecutions);
            let (unordered, unordered_result) = multi_flight(ROOTS_UNORDERED, arm, history_without_void(counts), ISSUED_AND_COVERED, MultiFlightEnumeration::DirectExecutions);
            assert!(ordered_result.executions < unordered_result.executions, "{arm:?} r={counts:?}");
            assert!(ordered.states.is_subset(&unordered.states), "{arm:?} r={counts:?}");
        }
    }
}

// ===== 第三段：等待模型 =====

fn wait(condition: PublishCondition, arm: AppendArm, profile: PreparationProfile, point: CostPoint, record_count: u32, transaction_count: u32, reading: AppendedReading) -> WaitResult {
    evaluate_wait(&WaitScenario { condition, append_arm: arm, profile, point, record_count, transaction_count }, reading)
}

const DEVICE_DOMINATED_POINT: CostPoint = CostPoint::DeviceDominated;
const CPU_DOMINATED_POINT: CostPoint = CostPoint::CpuDominated;
const UNIFORM: PreparationProfile = PreparationProfile::Uniform;
const SLOW_HEAD: PreparationProfile = PreparationProfile::SlowHead;

/// 登记第七节 B6 的原数：k = 1、r = 1：P1 下 4.205 / 4.2、P2 下 4.7 / 4.2（甲 / 乙），慢头各加 Δ = 5；四个条件、两读都一样。
#[test]
fn wait_single_transaction_equals_the_registered_wait_anchor_values_under_every_condition_and_reading() {
    let expected: [(AppendArm, PreparationProfile, CostPoint, u64); 12] = [
        (AppendArm::JiaWhole, UNIFORM, DEVICE_DOMINATED_POINT, 4205),
        (AppendArm::JiaInterleaved, UNIFORM, DEVICE_DOMINATED_POINT, 4205),
        (AppendArm::YiReservedWhole, UNIFORM, DEVICE_DOMINATED_POINT, 4200),
        (AppendArm::JiaWhole, UNIFORM, CPU_DOMINATED_POINT, 4700),
        (AppendArm::JiaInterleaved, UNIFORM, CPU_DOMINATED_POINT, 4700),
        (AppendArm::YiReservedWhole, UNIFORM, CPU_DOMINATED_POINT, 4200),
        (AppendArm::JiaWhole, SLOW_HEAD, DEVICE_DOMINATED_POINT, 9205),
        (AppendArm::JiaInterleaved, SLOW_HEAD, DEVICE_DOMINATED_POINT, 9205),
        (AppendArm::YiReservedWhole, SLOW_HEAD, DEVICE_DOMINATED_POINT, 9200),
        (AppendArm::JiaWhole, SLOW_HEAD, CPU_DOMINATED_POINT, 9700),
        (AppendArm::JiaInterleaved, SLOW_HEAD, CPU_DOMINATED_POINT, 9700),
        (AppendArm::YiReservedWhole, SLOW_HEAD, CPU_DOMINATED_POINT, 9200),
    ];
    for condition in WAIT_CONDITIONS {
        for reading in AppendedReading::ALL {
            for (arm, profile, point, latest_return_time) in expected {
                assert_eq!(wait(condition, arm, profile, point, 1, 1, reading).latest_return_time, latest_return_time, "{condition:?} {arm:?} {profile:?} {point:?} {reading:?}");
                assert_eq!(wait_anchor_single_transaction(arm, profile, point), latest_return_time, "闭式与原数相同");
            }
        }
    }
}

#[test]
fn wait_serial_two_transactions_equal_the_registered_wait_anchor_values() {
    for reading in AppendedReading::ALL {
        assert_eq!(wait(SERIAL, AppendArm::JiaWhole, UNIFORM, DEVICE_DOMINATED_POINT, 1, 2, reading).latest_return_time, 8410);
        assert_eq!(wait(SERIAL, AppendArm::YiReservedWhole, UNIFORM, DEVICE_DOMINATED_POINT, 1, 2, reading).latest_return_time, 8400);
    }
}

/// 手推：多飞·根乱、甲整段、k = 2、r = 1、P1、等速：B1、B2、B3 的第二道都并进前一道（读法 ⑤ 跨发布），两个 F 各占 b。
#[test]
fn wait_roots_unordered_two_transactions_merge_the_second_barriers_and_follow_the_hand_computed_path() {
    let result = wait(ROOTS_UNORDERED, AppendArm::JiaWhole, UNIFORM, DEVICE_DOMINATED_POINT, 1, 2, ISSUED_AND_COVERED);
    assert_eq!(result.return_times, vec![5405, 5405]);
    assert_eq!(result.latest_transaction, 2);
    assert_eq!(result.merged_barriers_on_device, 3);
    let expected_path = CriticalPath { barriers_or_forced_unit_access: 5, device_writes: 8, cpu_steps: 1, merged_barriers: 1, tie_steps: 1, source_time: 0, duration_sum: 5405 };
    assert_eq!(result.path, expected_path);
}

/// 手推：同发、甲整段、r = 1、P2、等速：追加者每条 0.5，k = 8 时 8.55，k = 64 时 39.35；关键路径 64 步 CPU、4 道屏障或 FUA、67 次写。
#[test]
fn wait_shared_publish_jia_whole_under_cpu_dominated_costs_follows_the_appender() {
    assert_eq!(wait(SHARED, AppendArm::JiaWhole, UNIFORM, CPU_DOMINATED_POINT, 1, 8, ISSUED_AND_COVERED).latest_return_time, 8550);
    let result = wait(SHARED, AppendArm::JiaWhole, UNIFORM, CPU_DOMINATED_POINT, 1, 64, ISSUED_AND_COVERED);
    assert_eq!(result.latest_return_time, 39350);
    assert_eq!((result.path.cpu_steps, result.path.barriers_or_forced_unit_access, result.path.device_writes, result.path.tie_steps), (64, 4, 67, 0));
}

/// 手推：同发、乙整段、r = 1、P2、等速：预留每个 0.5，T64 的预留在 32 才完成，k = 8 时 7.1，k = 64 时 35.1；关键路径 64 步预留、2 次写、3 道屏障或 FUA。
#[test]
fn wait_shared_publish_yi_under_cpu_dominated_costs_follows_the_reservations() {
    assert_eq!(wait(SHARED, AppendArm::YiReservedWhole, UNIFORM, CPU_DOMINATED_POINT, 1, 8, ISSUED_AND_COVERED).latest_return_time, 7100);
    let result = wait(SHARED, AppendArm::YiReservedWhole, UNIFORM, CPU_DOMINATED_POINT, 1, 64, ISSUED_AND_COVERED);
    assert_eq!(result.latest_return_time, 35100);
    assert_eq!((result.path.cpu_steps, result.path.device_writes, result.path.barriers_or_forced_unit_access), (64, 2, 3));
}

/// 手推：多飞、乙整段、k = 2、r = 1、P1、慢头：根乱下 T2 先做完、等 T1 已追加（R2 等 T1 的 B2 罩住 7.15，R1 等 T1 的记录发出 6.1）；
/// 根序下 F2 要等 F1，T2 落到 11.25。
#[test]
fn wait_multi_flight_yi_with_a_slow_head_waits_for_the_head_by_each_reading() {
    assert_eq!(wait(ROOTS_UNORDERED, AppendArm::YiReservedWhole, SLOW_HEAD, DEVICE_DOMINATED_POINT, 1, 2, ISSUED_AND_COVERED).return_times, vec![9200, 7150]);
    assert_eq!(wait(ROOTS_UNORDERED, AppendArm::YiReservedWhole, SLOW_HEAD, DEVICE_DOMINATED_POINT, 1, 2, ISSUED_ONLY).return_times, vec![9200, 6100]);
    assert_eq!(wait(ROOTS_ORDERED, AppendArm::YiReservedWhole, SLOW_HEAD, DEVICE_DOMINATED_POINT, 1, 2, ISSUED_AND_COVERED).return_times, vec![10200, 11250]);
}

/// 登记 8.2 判别力自证：把 Q2-3 的门槛 2 挪到两个取样点实测 ρ 之间，那一格的判定必须翻。
#[test]
fn moving_the_wait_growth_threshold_between_the_two_cost_points_flips_one_judgement() {
    let at = |point: CostPoint, transaction_count: u32| wait(SHARED, AppendArm::JiaWhole, UNIFORM, point, 1, transaction_count, ISSUED_AND_COVERED).latest_return_time;
    let (device_dominated_small, device_dominated_large, cpu_dominated_small, cpu_dominated_large) = (at(DEVICE_DOMINATED_POINT, 8), at(DEVICE_DOMINATED_POINT, 64), at(CPU_DOMINATED_POINT, 8), at(CPU_DOMINATED_POINT, 64));
    assert_ne!(device_dominated_large * cpu_dominated_small, cpu_dominated_large * device_dominated_small, "两个取样点的 ρ 不同");
    // 门槛取两个 ρ 的中点：(ρ1 + ρ2) / 2 = (l1·s2 + l2·s1) / (2·s1·s2)。
    let midpoint = (device_dominated_large * cpu_dominated_small + cpu_dominated_large * device_dominated_small, 2 * device_dominated_small * cpu_dominated_small);
    let registered = (growth_judgement(device_dominated_small, device_dominated_large, REGISTERED_GROWS_THRESHOLD, REGISTERED_BOUNDED_THRESHOLD), growth_judgement(cpu_dominated_small, cpu_dominated_large, REGISTERED_GROWS_THRESHOLD, REGISTERED_BOUNDED_THRESHOLD));
    let moved = (growth_judgement(device_dominated_small, device_dominated_large, midpoint, (0, 1)), growth_judgement(cpu_dominated_small, cpu_dominated_large, midpoint, (0, 1)));
    assert_eq!(registered.1, GrowthJudgement::Grows);
    assert_ne!(moved.0, moved.1, "门槛挪到两点之间，两点的判定一个是长、一个不是");
}

#[test]
fn wait_grid_pending_and_reserved_lag_trajectories_end_at_zero() {
    let grid = wait_grid();
    assert_eq!(grid.len(), 1344);
    assert!(grid.values().all(|result| result.pending.final_value == 0 && result.pending.peak_after_drain == 0));
    assert!(grid.values().all(|result| result.reserved_lag.is_none_or(|lag| lag.final_value == 0)));
    assert!(grid.iter().all(|(key, result)| result.return_times.len() == usize::try_from(key.6).expect("k ≤ 64")));
}

/// 不带参数三段依次都跑；段名只跑那一段；认不出的参数不选任何一段。
#[test]
fn no_argument_selects_all_three_segments_and_a_segment_name_selects_one() {
    let arguments = |texts: &[&str]| texts.iter().map(|text| text.to_string()).collect::<Vec<String>>();
    assert_eq!(segments_for_arguments(&arguments(&[])), Some(vec![SegmentChoice::One, SegmentChoice::Two, SegmentChoice::Three]));
    assert_eq!(segments_for_arguments(&arguments(&["segment-one"])), Some(vec![SegmentChoice::One]));
    assert_eq!(segments_for_arguments(&arguments(&["segment-two"])), Some(vec![SegmentChoice::Two]));
    assert_eq!(segments_for_arguments(&arguments(&["segment-three"])), Some(vec![SegmentChoice::Three]));
    assert_eq!(segments_for_arguments(&arguments(&["replay-core-inputs"])), Some(vec![SegmentChoice::ReplayCoreInputs]));
    assert_eq!(segments_for_arguments(&arguments(&["never-used-segment-name"])), None);
    assert_eq!(segments_for_arguments(&arguments(&["segment-one", "segment-two"])), None);
}
