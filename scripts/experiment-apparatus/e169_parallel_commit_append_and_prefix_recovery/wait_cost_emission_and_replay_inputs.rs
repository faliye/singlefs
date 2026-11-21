//! 等待量的锚点、格、判定、成对与几何输出，等待代价那一路的运行与重放核心输入。

use super::*;

/// 登记第七节 B6：k = 2、r = 1、串落、等速、P1：甲整段 8.41、乙整段 8.4（写成整数单位）。
const WAIT_ANCHOR_SERIAL_TWO_TRANSACTIONS: [(AppendArm, u64); 2] = [(AppendArm::JiaWhole, 8410), (AppendArm::YiReservedWhole, 8400)];

fn emit_wait_anchors(emitter: &mut Emitter, grid: &BTreeMap<WaitKey, WaitResult>) -> u64 {
    let mut checked = 0u64;
    let mut mismatches = 0u64;
    let mut check = |check_emitter: &mut Emitter, key: WaitKey, expected: u64| {
        let measured = grid[&key].latest_return_time;
        checked += 1;
        mismatches += u64::from(measured != expected);
        let (condition, arm, profile, point, record_count, reading, transaction_count) = key;
        emit(
            check_emitter,
            &format!(
                "name=wait_anchor family=B6 q1={} q2={} profile={} point={} r={record_count} reading={} k={transaction_count} expected_w_max={} measured_w_max={} matches={}",
                condition.label(),
                arm.label(),
                profile.label(),
                point.label(),
                reading.label(),
                format_wait_units(expected),
                format_wait_units(measured),
                yes_or_no(measured == expected)
            ),
        );
    };
    for condition in WAIT_CONDITIONS {
        for arm in WAIT_APPEND_ARMS {
            for profile in PreparationProfile::ALL {
                for point in CostPoint::ALL {
                    for reading in AppendedReading::ALL {
                        check(emitter, (condition, arm, profile, point, 1, reading, 1), wait_anchor_single_transaction(arm, profile, point));
                    }
                }
            }
        }
    }
    for (arm, expected) in WAIT_ANCHOR_SERIAL_TWO_TRANSACTIONS {
        for reading in AppendedReading::ALL {
            check(emitter, (PublishCondition::SerialPublishes, arm, PreparationProfile::Uniform, CostPoint::DeviceDominated, 1, reading, 2), expected);
        }
    }
    emit(emitter, &format!("name=wait_anchor_summary families=B6 checked={checked} mismatches={mismatches}"));
    mismatches
}

fn wait_verdict_key_text(key: WaitVerdictKey) -> String {
    let (condition, arm, profile, point, record_count, reading) = key;
    format!("q1={} q2={} profile={} point={} r={record_count} reading={}", condition.label(), arm.label(), profile.label(), point.label(), reading.label())
}

fn emit_wait_cell(emitter: &mut Emitter, key: WaitKey, result: &WaitResult) {
    let (condition, arm, profile, point, record_count, reading, transaction_count) = key;
    let lag = match result.reserved_lag {
        Some(lag) => format!("reserved_lag_initial={} reserved_lag_peak_after_drain={} reserved_lag_positive_steps={} reserved_lag_final={}", lag.initial, lag.peak_after_drain, lag.positive_steps, lag.final_value),
        None => "reserved_lag=not_applicable".to_string(),
    };
    emit(
        emitter,
        &format!(
            "name=wait_cell {} k={transaction_count} w_max={} w_mean={} latest_transaction={} n_bar={} n_dev={} n_cpu={} merged_barriers_on_path={} merged_barriers_on_device={} tie_steps={} path_source_time={} pending_initial={} pending_peak_after_drain={} pending_positive_steps={} pending_final={} {lag}",
            wait_verdict_key_text((condition, arm, profile, point, record_count, reading)),
            format_wait_units(result.latest_return_time),
            format_ratio(result.return_time_sum(), u64::from(transaction_count) * WAIT_UNITS_PER_ABSTRACT_UNIT),
            result.latest_transaction,
            result.path.barriers_or_forced_unit_access,
            result.path.device_writes,
            result.path.cpu_steps,
            result.path.merged_barriers,
            result.merged_barriers_on_device,
            result.path.tie_steps,
            format_wait_units(result.path.source_time),
            result.pending.initial,
            result.pending.peak_after_drain,
            result.pending.positive_steps,
            result.pending.final_value
        ),
    );
}

/// Q2-4：k = 64 的分量大于 k = 8 的分量 ⇒ 这个分量随 k 长。返回（n_bar，n_dev，n_cpu）各自长没长。
fn growing_components(small: &WaitResult, large: &WaitResult) -> [(&'static str, u64, u64, bool); 3] {
    [
        ("n_bar", small.path.barriers_or_forced_unit_access, large.path.barriers_or_forced_unit_access),
        ("n_dev", small.path.device_writes, large.path.device_writes),
        ("n_cpu", small.path.cpu_steps, large.path.cpu_steps),
    ]
    .map(|(name, at_small, at_large)| (name, at_small, at_large, at_large > at_small))
}

fn growing_component_names(components: &[(&'static str, u64, u64, bool); 3]) -> Vec<&'static str> {
    components.iter().filter(|(_, _, _, grows)| *grows).map(|(name, ..)| *name).collect()
}

fn names_or_none(names: &[&'static str]) -> String {
    if names.is_empty() {
        "none".to_string()
    } else {
        names.join(",")
    }
}

fn wait_pair_of_grid(grid: &BTreeMap<WaitKey, WaitResult>, key: WaitVerdictKey) -> (&WaitResult, &WaitResult) {
    let (condition, arm, profile, point, record_count, reading) = key;
    (
        &grid[&(condition, arm, profile, point, record_count, reading, GROWTH_RATIO_SMALL_TRANSACTION_COUNT)],
        &grid[&(condition, arm, profile, point, record_count, reading, GROWTH_RATIO_LARGE_TRANSACTION_COUNT)],
    )
}

fn registered_growth_judgement(grid: &BTreeMap<WaitKey, WaitResult>, key: WaitVerdictKey) -> GrowthJudgement {
    let (small, large) = wait_pair_of_grid(grid, key);
    growth_judgement(small.latest_return_time, large.latest_return_time, REGISTERED_GROWS_THRESHOLD, REGISTERED_BOUNDED_THRESHOLD)
}

fn every_wait_verdict_key() -> Vec<WaitVerdictKey> {
    let mut keys = Vec::new();
    for condition in WAIT_CONDITIONS {
        for arm in WAIT_APPEND_ARMS {
            for profile in PreparationProfile::ALL {
                for point in CostPoint::ALL {
                    for record_count in WAIT_RECORD_COUNTS {
                        for reading in AppendedReading::ALL {
                            keys.push((condition, arm, profile, point, record_count, reading));
                        }
                    }
                }
            }
        }
    }
    keys
}

/// 第六节 Q2-3（ρ 与判定）与 Q2-4（三个分量各自长没长），每个（Q1 条件，Q2 臂，剖面，取样点，r，读法）各一行。
fn emit_wait_verdicts(emitter: &mut Emitter, grid: &BTreeMap<WaitKey, WaitResult>) {
    for key in every_wait_verdict_key() {
        let (small, large) = wait_pair_of_grid(grid, key);
        emit(
            emitter,
            &format!(
                "name=wait_verdict quantity=Q2-3 {} w_max_k8={} w_max_k64={} rho={} judgement={}",
                wait_verdict_key_text(key),
                format_wait_units(small.latest_return_time),
                format_wait_units(large.latest_return_time),
                format_ratio(large.latest_return_time, small.latest_return_time),
                registered_growth_judgement(grid, key).label()
            ),
        );
        let components = growing_components(small, large);
        let component_text: Vec<String> = components.iter().map(|(name, at_small, at_large, grows)| format!("{name}_k8={at_small} {name}_k64={at_large} {name}_grows={}", yes_or_no(*grows))).collect();
        emit(
            emitter,
            &format!("name=wait_verdict quantity=Q2-4 {} {} growing={}", wait_verdict_key_text(key), component_text.join(" "), names_or_none(&growing_component_names(&components))),
        );
    }
}

/// 第十节 F9、F10：同一（Q1 条件，剖面，取样点）下两条 Q2 臂的 Q2-3 判定配对（单一追加者的两种读法各与乙整段配一次）。
fn emit_wait_pairs(emitter: &mut Emitter, grid: &BTreeMap<WaitKey, WaitResult>) {
    let pairs = [(AppendArm::JiaWhole, AppendArm::YiReservedWhole), (AppendArm::JiaInterleaved, AppendArm::YiReservedWhole)];
    for condition in WAIT_CONDITIONS {
        for profile in PreparationProfile::ALL {
            for point in CostPoint::ALL {
                for record_count in WAIT_RECORD_COUNTS {
                    for reading in AppendedReading::ALL {
                        for (first, second) in pairs {
                            let first_key = (condition, first, profile, point, record_count, reading);
                            let second_key = (condition, second, profile, point, record_count, reading);
                            let first_judgement = registered_growth_judgement(grid, first_key);
                            let second_judgement = registered_growth_judgement(grid, second_key);
                            let (first_small, first_large) = wait_pair_of_grid(grid, first_key);
                            let (second_small, second_large) = wait_pair_of_grid(grid, second_key);
                            let first_growing = growing_component_names(&growing_components(first_small, first_large));
                            let second_growing = growing_component_names(&growing_components(second_small, second_large));
                            let shared_growing: Vec<&'static str> = first_growing.iter().filter(|name| second_growing.contains(*name)).copied().collect();
                            let clause = match (first_judgement, second_judgement) {
                                (GrowthJudgement::Grows, GrowthJudgement::Bounded) => "f10_first_arm_flips",
                                (GrowthJudgement::Bounded, GrowthJudgement::Grows) => "f10_second_arm_flips",
                                (GrowthJudgement::Grows, GrowthJudgement::Grows) => "f9_both_grow",
                                (
                                    GrowthJudgement::Grows | GrowthJudgement::Bounded | GrowthJudgement::Between,
                                    GrowthJudgement::Grows | GrowthJudgement::Bounded | GrowthJudgement::Between,
                                ) => "neither_f9_nor_f10",
                            };
                            emit(
                                emitter,
                                &format!(
                                    "name=wait_pair q1={} profile={} point={} r={record_count} reading={} first={} second={} first_judgement={} second_judgement={} first_growing={} second_growing={} shared_growing={} clause={clause}",
                                    condition.label(),
                                    profile.label(),
                                    point.label(),
                                    reading.label(),
                                    first.label(),
                                    second.label(),
                                    first_judgement.label(),
                                    second_judgement.label(),
                                    names_or_none(&first_growing),
                                    names_or_none(&second_growing),
                                    names_or_none(&shared_growing)
                                ),
                            );
                        }
                    }
                }
            }
        }
    }
}

/// 登记 8.2 的 G3：主取样点 P1、等速、r = 1；方向相反的取样点 P2、慢头、r = 2（一次换一个旋钮）。
fn emit_wait_geometry(emitter: &mut Emitter, grid: &BTreeMap<WaitKey, WaitResult>) -> u64 {
    let mut unstable_lines = 0u64;
    let opposite_points: [(&str, PreparationProfile, CostPoint, u32); 3] = [
        ("p2_cpu_dominated", PreparationProfile::Uniform, CostPoint::CpuDominated, 1),
        ("slow_head", PreparationProfile::SlowHead, CostPoint::DeviceDominated, 1),
        ("r2", PreparationProfile::Uniform, CostPoint::DeviceDominated, 2),
    ];
    for condition in WAIT_CONDITIONS {
        for arm in WAIT_APPEND_ARMS {
            for reading in AppendedReading::ALL {
                let main_key = (condition, arm, PreparationProfile::Uniform, CostPoint::DeviceDominated, 1, reading);
                let main_judgement = registered_growth_judgement(grid, main_key);
                let (main_small, main_large) = wait_pair_of_grid(grid, main_key);
                let main_growing = growing_component_names(&growing_components(main_small, main_large));
                for (opposite_label, profile, point, record_count) in opposite_points {
                    let opposite_key = (condition, arm, profile, point, record_count, reading);
                    let opposite_judgement = registered_growth_judgement(grid, opposite_key);
                    let (opposite_small, opposite_large) = wait_pair_of_grid(grid, opposite_key);
                    let opposite_growing = growing_component_names(&growing_components(opposite_small, opposite_large));
                    let stable = main_judgement == opposite_judgement && main_growing == opposite_growing;
                    unstable_lines += u64::from(!stable);
                    emit(
                        emitter,
                        &format!(
                            "name=wait_geometry id=G3 q1={} q2={} reading={} main=p1_uniform_r1 opposite={opposite_label} main_q2_3={} opposite_q2_3={} main_q2_4_growing={} opposite_q2_4_growing={} stability={}",
                            condition.label(),
                            arm.label(),
                            reading.label(),
                            main_judgement.label(),
                            opposite_judgement.label(),
                            names_or_none(&main_growing),
                            names_or_none(&opposite_growing),
                            if stable { "stable" } else { "unstable" }
                        ),
                    );
                }
            }
        }
    }
    unstable_lines
}

pub(super) fn run_segment_three() {
    let mut emitter = Emitter::new();
    let cost_text = |point: CostPoint| {
        let costs = point.costs();
        format!(
            "{}=w{},b{},h_or_g{},delta{}",
            point.label(),
            format_wait_units(costs.write),
            format_wait_units(costs.barrier_or_forced_unit_access),
            format_wait_units(costs.cpu_step),
            format_wait_units(costs.slow_head_delay)
        )
    };
    emit(
        &mut emitter,
        &format!(
            "name=config experiment=e169 segment=3 model=fsync_wait q1=serial_publishes,shared_publish,multi_flight_roots_ordered,multi_flight_roots_unordered q2=jia_whole,jia_interleaved,yi_reserved_whole ding_control=no_wait_rule_registered profiles=uniform,slow_head readings=r2_covered,r1_issued k=1,2,4,8,16,32,64 r=1,2 histories=H0 {} {} rho=w_max_k64_over_w_max_k8 grows_threshold=2 bounded_threshold=1.25",
            cost_text(CostPoint::DeviceDominated),
            cost_text(CostPoint::CpuDominated)
        ),
    );
    let grid = wait_grid();
    emit_wait_anchors(&mut emitter, &grid);
    for (key, result) in &grid {
        emit_wait_cell(&mut emitter, *key, result);
    }
    emit_wait_verdicts(&mut emitter, &grid);
    emit_wait_pairs(&mut emitter, &grid);
    let unstable_lines = emit_wait_geometry(&mut emitter, &grid);
    emit(&mut emitter, &format!("name=wait_geometry_summary id=G3 lines={} unstable_lines={unstable_lines}", WAIT_CONDITIONS.len() * WAIT_APPEND_ARMS.len() * AppendedReading::ALL.len() * 3));
    // 第八节 8.1：期末值必须是 0，不为 0 判装置错；关键路径之和在 `evaluate_wait` 里逐格断言过。
    let cells = grid.len();
    let pending_final_nonzero = grid.values().filter(|result| result.pending.final_value > 0).count();
    let pending_rebound = grid.values().filter(|result| result.pending.peak_after_drain > 0).count();
    let lag_cells = grid.values().filter(|result| result.reserved_lag.is_some()).count();
    let lag_final_nonzero = grid.values().filter(|result| result.reserved_lag.is_some_and(|lag| lag.final_value > 0)).count();
    let lag_peak = grid.values().filter_map(|result| result.reserved_lag.map(|lag| lag.peak_after_drain.max(lag.initial))).max().unwrap_or(0);
    emit(&mut emitter, &format!("name=wait_assertion check=critical_path_sum cells={cells} mismatches=0"));
    emit(&mut emitter, &format!("name=wait_assertion check=pending_trajectory cells={cells} pending_final_nonzero={pending_final_nonzero} pending_peak_after_drain_nonzero={pending_rebound}"));
    emit(&mut emitter, &format!("name=wait_assertion check=reserved_lag_trajectory cells={lag_cells} reserved_lag_final_nonzero={lag_final_nonzero} reserved_lag_largest_initial_or_rebound={lag_peak}"));
    for condition in WAIT_CONDITIONS {
        let judgements: Vec<GrowthJudgement> = every_wait_verdict_key().into_iter().filter(|key| key.0 == condition).map(|key| registered_growth_judgement(&grid, key)).collect();
        let count_of = |wanted: GrowthJudgement| judgements.iter().filter(|judgement| **judgement == wanted).count();
        emit(
            &mut emitter,
            &format!(
                "name=wait_summary quantity=Q2-3 q1={} verdict_lines={} grows={} bounded={} between_not_judged={}",
                condition.label(),
                judgements.len(),
                count_of(GrowthJudgement::Grows),
                count_of(GrowthJudgement::Bounded),
                count_of(GrowthJudgement::Between)
            ),
        );
    }
    println!("{}", emitter.finish());
}

// ===== 路径 3 对路径 4 的直接对拍：把第一、二段每个崩溃状态的恢复输入与模型丙0 的结果打出来，交给按路径依赖 `replay_core` 的临时 crate 逐份比 =====

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum ReplayCoreInputKind {
    /// 这个崩溃状态全部持久的记录（模型的丙0 在实验里吃的就是这一份）。
    AllDurableRecords,
    /// 这个崩溃状态的严格连续段（登记第一节 1.2；派发提示点名的那一份）。
    StrictSegment,
}

impl ReplayCoreInputKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            ReplayCoreInputKind::AllDurableRecords => "all_durable",
            ReplayCoreInputKind::StrictSegment => "strict_segment",
        }
    }
}

/// (jsn, txg, 事务号, 提交标记, 末条标志, 本次发布内序号, 点名单元验得过)
type ReplayCoreRecordKey = (u64, u64, u32, bool, bool, u32, bool);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Bing0Result {
    pub(super) core_failure: bool,
    applied_records: usize,
    last_applied_jsn: Option<u64>,
    maximum_applied_transaction: u32,
}

fn cell_crash_states(request: &CellRequest) -> Option<HashSet<CrashState>> {
    if !cell_is_applicable(request.append_arm, &request.history) {
        return None;
    }
    let layout = plan_cell(request.condition, request.append_arm, &request.history, request.discipline);
    if request.condition.publishes_fly_together() {
        return Some(enumerate_multi_flight(request, &layout, MultiFlightEnumeration::registered_for(&request.history)).states);
    }
    let mut states = HashSet::new();
    for execution in build_executions(&layout, request.append_arm, request.discipline, request.barriers) {
        collect_crash_states(&execution, request.reading, request.return_rule, &mut states);
    }
    Some(states)
}

pub(super) fn run_replay_core_inputs() {
    let mut emitter = Emitter::new();
    let mut inputs: BTreeMap<(ReplayCoreInputKind, u64, Vec<ReplayCoreRecordKey>), (u64, Bing0Result)> = BTreeMap::new();
    let segments: [(&str, [PublishCondition; 2], Vec<History>); 2] =
        [("one", PublishCondition::SEGMENT_ONE, all_histories()), ("two", PublishCondition::SEGMENT_TWO, segment_two_histories())];
    let mut state_counts: Vec<(&str, u64, u64)> = Vec::new();
    for (segment_label, conditions, histories) in &segments {
        let mut cells = 0u64;
        let mut states_in_segment = 0u64;
        for condition in conditions {
            for arm in AppendArm::ALL {
                for history in histories {
                    for reading in AppendedReading::ALL {
                        let Some(states) = cell_crash_states(&CellRequest::main_grid(*condition, arm, history.clone(), reading)) else {
                            continue;
                        };
                        cells += 1;
                        states_in_segment += u64::try_from(states.len()).expect("状态数有限");
                        for state in &states {
                            let view = StateView::of(state);
                            let root_txg = view.chosen_root_txg();
                            let verify = |record: &RecordContent| named_units_are_verified(record, &view.durable_new);
                            let strict_segment = strict_continuous_segment(&view.records_by_jsn, root_txg);
                            for (kind, records) in [(ReplayCoreInputKind::AllDurableRecords, &view.records_by_jsn), (ReplayCoreInputKind::StrictSegment, &strict_segment)] {
                                let walk = walk_prefix(records, root_txg, &verify, PrefixRules::TODAY);
                                let result = Bing0Result {
                                    core_failure: walk.core_failure,
                                    applied_records: walk.applied.len(),
                                    last_applied_jsn: walk.applied.last().map(|record| record.jsn),
                                    maximum_applied_transaction: walk.applied.iter().map(|record| record.transaction).max().unwrap_or(0),
                                };
                                let key_records: Vec<ReplayCoreRecordKey> = records
                                    .iter()
                                    .map(|record| (record.jsn, record.txg, record.transaction, record.is_commit, record.ends_its_publish, record.ordinal_within_publish, verify(record)))
                                    .collect();
                                let entry = inputs.entry((kind, root_txg, key_records)).or_insert((0, result));
                                assert_eq!(entry.1, result, "同一份恢复输入，模型的丙0 给同一个结果");
                                entry.0 += 1;
                            }
                        }
                    }
                }
            }
        }
        state_counts.push((*segment_label, cells, states_in_segment));
    }
    let flag = |holds: bool| if holds { "1" } else { "0" };
    for ((kind, root_txg, records), (states, result)) in &inputs {
        let records_text = if records.is_empty() {
            "none".to_string()
        } else {
            records
                .iter()
                .map(|(jsn, txg, transaction, is_commit, ends, ordinal, verified)| format!("{jsn}:{txg}:{transaction}:{}:{}:{ordinal}:{}", flag(*is_commit), flag(*ends), flag(*verified)))
                .collect::<Vec<_>>()
                .join(",")
        };
        emit(
            &mut emitter,
            &format!(
                "name=replay_core_input input={} root_txg={root_txg} states={states} records={records_text} bing0_failure={} bing0_applied_records={} bing0_last_applied_jsn={} bing0_maximum_applied_transaction={}",
                kind.label(),
                yes_or_no(result.core_failure),
                result.applied_records,
                result.last_applied_jsn.map_or("none".to_string(), |jsn| jsn.to_string()),
                result.maximum_applied_transaction
            ),
        );
    }
    for (segment_label, cells, states) in &state_counts {
        emit(&mut emitter, &format!("name=replay_core_input_states segment={segment_label} cells={cells} states={states}"));
    }
    for kind in [ReplayCoreInputKind::AllDurableRecords, ReplayCoreInputKind::StrictSegment] {
        let of_kind: Vec<&(u64, Bing0Result)> = inputs.iter().filter(|((candidate, ..), _)| *candidate == kind).map(|(_, value)| value).collect();
        emit(
            &mut emitter,
            &format!(
                "name=replay_core_input_summary input={} distinct_inputs={} states={}",
                kind.label(),
                of_kind.len(),
                of_kind.iter().map(|(states, _)| *states).sum::<u64>()
            ),
        );
    }
    println!("{}", emitter.finish());
}
