//! 追加格与多在途格两路的运行与多在途锚点的输出。

use super::*;

pub(super) fn run_segment_one() {
    let mut emitter = Emitter::new();
    let histories = all_histories();
    let count_of = |family: HistoryFamily| histories.iter().filter(|history| history.family() == family).count();
    emit(
        &mut emitter,
        &format!(
            "name=config experiment=e169 segment=1 q1=serial_publishes,shared_publish q2=jia_whole,jia_interleaved,yi_reserved_whole,ding_control q3=bing0_today,bing1_truncate_before_a,bing2_drop_a_keep_b,bing3_forbid readings=r2_covered,r1_issued histories_h0={} histories_h1={} histories_h2={} in_flight_record_limit={IN_FLIGHT_RECORD_LIMIT}",
            count_of(HistoryFamily::WithoutVoid),
            count_of(HistoryFamily::NarrowVoid),
            count_of(HistoryFamily::WideVoid)
        ),
    );
    // 停机条款 T1：十四条序列逐条核，有一条不同就停。
    let mut fixed_sequence_mismatches = 0u64;
    for sequence in fixed_sequences() {
        let (publishes, records) = bing0_on_fixed_sequence(&sequence);
        let matches = publishes == sequence.expected_publishes_applied && records == sequence.expected_records_applied;
        fixed_sequence_mismatches += u64::from(!matches);
        emit(
            &mut emitter,
            &format!(
                "name=t1_sequence sequence={} deciding_line=lib.rs:{} expected_publishes_applied={} model_publishes_applied={publishes} expected_records_applied={} model_records_applied={records} matches={}",
                sequence.number,
                sequence.deciding_line,
                sequence.expected_publishes_applied,
                sequence.expected_records_applied,
                yes_or_no(matches)
            ),
        );
    }
    emit(&mut emitter, &format!("name=t1_summary sequences=14 mismatches={fixed_sequence_mismatches}"));
    if fixed_sequence_mismatches > 0 {
        emit(&mut emitter, "name=stopped clause=T1 remaining_segment_one=not_run");
        println!("{}", emitter.finish());
        std::process::exit(3);
    }
    emit_anchors(&mut emitter);
    let mut grid: BTreeMap<GridKey, AppendCellResult> = BTreeMap::new();
    for condition in PublishCondition::SEGMENT_ONE {
        for arm in AppendArm::ALL {
            for history in &histories {
                for reading in AppendedReading::ALL {
                    let key_text = cell_key_text(condition, arm, history, reading);
                    match evaluate_cell(&CellRequest::main_grid(condition, arm, history.clone(), reading)) {
                        Some(result) => {
                            emit_cell(&mut emitter, &key_text, &result);
                            grid.insert((condition, arm, history.clone(), reading), result);
                        }
                        None => emit(&mut emitter, &format!("name=cell {key_text} applicable=no")),
                    }
                }
            }
        }
    }
    emit_controls(&mut emitter, &grid);
    emit_verdicts(&mut emitter, &grid, &PublishCondition::SEGMENT_ONE);
    emit_assertions(&mut emitter, &grid, &PublishCondition::SEGMENT_ONE, true);
    emit_geometry(&mut emitter, &grid, &PublishCondition::SEGMENT_ONE, false);
    emit_wide_void_summaries(&mut emitter, &grid);
    println!("{}", emitter.finish());
}

/// 第二段的 B5 锚点：16 格的执行数与去重状态数逐个比；返回不符的条数。
fn emit_segment_two_anchors(emitter: &mut Emitter, grid: &BTreeMap<GridKey, AppendCellResult>) -> u64 {
    let mut checked = 0u64;
    let mut mismatches = 0u64;
    for (counts, condition, arm, executions, states) in MULTI_FLIGHT_ANCHOR {
        let measured = &grid[&(condition, arm, history_without_void(counts), AppendedReading::IssuedAndCovered)];
        for (quantity, expected, value) in [("executions", executions, measured.executions), ("states", states, measured.states)] {
            checked += 1;
            mismatches += u64::from(expected != value);
            emit(
                emitter,
                &format!(
                    "name=anchor family=B5 q1={} q2={} r={} quantity={quantity} source=script expected={expected} measured={value} matches={}",
                    condition.label(),
                    arm.label(),
                    history_without_void(counts).record_counts_label(),
                    yes_or_no(expected == value)
                ),
            );
        }
    }
    emit(emitter, &format!("name=anchor_summary families=B5 checked={checked} mismatches={mismatches}"));
    mismatches
}

/// 路径对拍：k = 2 的每一格再按配置去重算一遍，状态集合、执行数、轨迹、各量逐格比（k = 3 那条路径的校验）。
#[derive(Default)]
struct PathCheckTally {
    cells: u64,
    state_set_mismatches: u64,
    execution_mismatches: u64,
    trajectory_mismatches: u64,
    quantity_mismatches: u64,
    returned_before_last_barrier_zero_mismatches: u64,
    mismatch_lines: Vec<String>,
}

impl PathCheckTally {
    fn compare(&mut self, key_text: &str, request: &CellRequest, layout: &CellLayout, direct: &EnumeratedCrashStates, direct_result: &AppendCellResult) {
        let (searched, searched_result) = multi_flight_cell_result(request, layout, MultiFlightEnumeration::ByConfiguration);
        let state_sets_differ = direct.states != searched.states;
        let executions_differ = direct_result.executions != searched_result.executions;
        let trajectories_differ = direct_result.pending != searched_result.pending;
        let quantities_differ = direct_result.recovery != searched_result.recovery
            || direct_result.incomplete_pair_states != searched_result.incomplete_pair_states
            || direct_result.out_of_order_commit_pattern_states != searched_result.out_of_order_commit_pattern_states;
        let returned_before_last_barrier_zero_differs = (direct_result.returned_before_last_barrier_points == 0) != (searched_result.returned_before_last_barrier_points == 0);
        self.cells += 1;
        self.state_set_mismatches += u64::from(state_sets_differ);
        self.execution_mismatches += u64::from(executions_differ);
        self.trajectory_mismatches += u64::from(trajectories_differ);
        self.quantity_mismatches += u64::from(quantities_differ);
        self.returned_before_last_barrier_zero_mismatches += u64::from(returned_before_last_barrier_zero_differs);
        if state_sets_differ || executions_differ || trajectories_differ || quantities_differ || returned_before_last_barrier_zero_differs {
            self.mismatch_lines.push(format!(
                "name=path_check_mismatch {key_text} direct_states={} searched_states={} direct_executions={} searched_executions={}",
                direct_result.states, searched_result.states, direct_result.executions, searched_result.executions
            ));
        }
    }
    fn emit_into(&self, emitter: &mut Emitter) {
        for line in &self.mismatch_lines {
            emit(emitter, line);
        }
        emit(
            emitter,
            &format!(
                "name=path_check paths=direct,by_configuration cells={} state_set_mismatches={} execution_mismatches={} trajectory_mismatches={} quantity_mismatches={} a7_zero_mismatches={}",
                self.cells, self.state_set_mismatches, self.execution_mismatches, self.trajectory_mismatches, self.quantity_mismatches, self.returned_before_last_barrier_zero_mismatches
            ),
        );
    }
}

pub(super) fn run_segment_two() {
    let mut emitter = Emitter::new();
    let histories = segment_two_histories();
    let count_of = |family: HistoryFamily| histories.iter().filter(|history| history.family() == family).count();
    emit(
        &mut emitter,
        &format!(
            "name=config experiment=e169 segment=2 q1=multi_flight_roots_ordered,multi_flight_roots_unordered q2=jia_whole,jia_interleaved,yi_reserved_whole,ding_control q3=bing0_today,bing1_truncate_before_a,bing2_drop_a_keep_b,bing3_forbid readings=r2_covered,r1_issued histories_h0={} histories_h1={} histories_h2=not_run_after_decidable k3_record_counts=1,1,1;2,1,1 enumeration_k2=direct enumeration_k3=by_configuration configuration_limit={CONFIGURATION_LIMIT} in_flight_record_limit={IN_FLIGHT_RECORD_LIMIT}",
            count_of(HistoryFamily::WithoutVoid),
            count_of(HistoryFamily::NarrowVoid)
        ),
    );
    let mut grid: BTreeMap<GridKey, AppendCellResult> = BTreeMap::new();
    let mut path_check = PathCheckTally::default();
    for condition in PublishCondition::SEGMENT_TWO {
        for arm in AppendArm::ALL {
            for history in &histories {
                for reading in AppendedReading::ALL {
                    let key_text = cell_key_text(condition, arm, history, reading);
                    let request = CellRequest::main_grid(condition, arm, history.clone(), reading);
                    let layout = plan_cell(condition, arm, history, AppendDiscipline::Kept);
                    let enumeration = MultiFlightEnumeration::registered_for(history);
                    let (enumerated, result) = multi_flight_cell_result(&request, &layout, enumeration);
                    if enumeration == MultiFlightEnumeration::DirectExecutions {
                        path_check.compare(&key_text, &request, &layout, &enumerated, &result);
                    }
                    emit_cell(&mut emitter, &key_text, &result);
                    emit(
                        &mut emitter,
                        &format!(
                            "name=cell_enumeration {key_text} enumeration={} configurations={} configuration_limit_exceeded={}",
                            result.multi_flight_enumeration.map_or("not_applicable", MultiFlightEnumeration::label),
                            result.configurations.map_or("not_applicable".to_string(), |count| count.to_string()),
                            yes_or_no(result.configuration_limit_exceeded)
                        ),
                    );
                    grid.insert((condition, arm, history.clone(), reading), result);
                }
            }
        }
    }
    // 停机条款 T3：k = 3 的配置数。
    let three_transaction_cells: Vec<&AppendCellResult> = grid.iter().filter(|((_, _, history, _), _)| history.transaction_count() == 3).map(|(_, result)| result).collect();
    let largest_configurations = three_transaction_cells.iter().filter_map(|result| result.configurations).max().unwrap_or(0);
    let exceeded_cells = three_transaction_cells.iter().filter(|result| result.configuration_limit_exceeded).count();
    emit(
        &mut emitter,
        &format!(
            "name=t3 k=3 cells={} largest_configurations={largest_configurations} limit={CONFIGURATION_LIMIT} exceeded_cells={exceeded_cells} k3_cells_kept={}",
            three_transaction_cells.len(),
            yes_or_no(exceeded_cells == 0)
        ),
    );
    if exceeded_cells > 0 {
        grid.retain(|(_, _, history, _), _| history.transaction_count() == 2);
        emit(&mut emitter, "name=stopped clause=T3 k3=not_run");
    }
    emit_segment_two_anchors(&mut emitter, &grid);
    path_check.emit_into(&mut emitter);
    emit_verdicts(&mut emitter, &grid, &PublishCondition::SEGMENT_TWO);
    emit_assertions(&mut emitter, &grid, &PublishCondition::SEGMENT_TWO, false);
    emit_geometry(&mut emitter, &grid, &PublishCondition::SEGMENT_TWO, true);
    // H2 放在段末：Q2、Q3 两行在多飞两个条件上的判决格都在 H0 ∪ H1，全部有了值，H2 够判后不跑（登记第六节「附带」、第十一节够判停机）。
    for condition in PublishCondition::SEGMENT_TWO {
        for arm in AppendArm::ALL {
            for reading in AppendedReading::ALL {
                let applicable = all_histories().iter().filter(|history| history.family() == HistoryFamily::WideVoid).any(|history| cell_is_applicable(arm, history));
                let status = if applicable { "applicable=yes run=not_run_after_decidable" } else { "applicable=no" };
                emit(&mut emitter, &format!("name=h2_summary q1={} q2={} reading={} {status}", condition.label(), arm.label(), reading.label()));
            }
        }
    }
    println!("{}", emitter.finish());
}
