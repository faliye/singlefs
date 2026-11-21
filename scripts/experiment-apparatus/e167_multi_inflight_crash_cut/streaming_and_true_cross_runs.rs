//! 流式那一路与跨事务真漏判的运行：阈值平移、几何敏感性、失败条款、跨事务真漏判与规模估计。

use e7_index_bench::isize_from_usize;
use super::*;

pub(super) fn run_second_segment(emitter: &mut Emitter) -> RunTally {
    let mut tally = RunTally::default();
    let threads = worker_thread_count();
    emit(
        emitter,
        "name=config model=independent_count_model candidates=4 sampling_points=G4a,G4a_ordered,G4b,G4b_ordered segment=second quantities=L1,L2,L3a,L3b,L4,L5,L6,L7a,L7b,L8,L10 not_run=L9,G5".to_string(),
    );
    let mut judgements: Vec<SecondJudgement> = Vec::new();
    for (geometry, labels, free_point, ordered_point) in second_segment_geometries() {
        for (label, candidate) in [(labels[0], Candidate::SerialPipeline), (labels[1], Candidate::OnePublishManyTransactions)] {
            let specification = CellSpecification { label, sampling_point: free_point, geometry: geometry.clone(), candidate };
            let evaluation = evaluate_cell_with(&specification, &truth_by_search_fast);
            let base = emit_cell(emitter, &mut tally, &evaluation);
            let miss_maximum = evaluation.streams.iter().map(|metrics| metrics.cross_misses + metrics.non_cross_misses).max().unwrap_or(0);
            let extra_states = isize_from_usize(evaluation.streams.iter().map(|metrics| metrics.state_count).max().unwrap_or(0)) - isize_from_usize(evaluation.truth.len());
            tally.self_check(miss_maximum == 0 && extra_states == 0, &format!("V1:real_baseline:{label}"));
            if candidate == Candidate::OnePublishManyTransactions {
                let stream_states = crash_states_of_stream(&evaluation.world.table, &evaluation.world.streams[0]);
                run_positive_controls_two_disk(
                    emitter,
                    &mut tally,
                    label,
                    &evaluation.world,
                    &evaluation.truth,
                    &evaluation.exact_reachable,
                    &evaluation.arm_by_transaction,
                    &|state| contains(&stream_states, state).then(|| stream_states.clone()),
                );
            }
            judgements.push(second_judgement_of_in_memory(&evaluation, base));
        }
        for (label, candidate, point) in [(labels[2], Candidate::ManyPublishesInFlightFree, free_point), (labels[3], Candidate::ManyPublishesInFlightOrdered, ordered_point)] {
            let specification = CellSpecification { label, sampling_point: point, geometry: geometry.clone(), candidate };
            let evaluation = evaluate_streaming_cell(&specification, threads);
            judgements.push(emit_streaming_cell(emitter, &mut tally, &evaluation));
            let mut by_representative: Vec<&SignatureRow> = evaluation.rows.iter().collect();
            by_representative.sort_unstable_by_key(|row| row.representative_path);
            let table = &evaluation.world.table;
            let find_stream = |state: StateCode| -> Option<StateList> {
                by_representative.iter().find_map(|row| {
                    let segments: Vec<Vec<usize>> = segments_of_signature(row.key, evaluation.table_summary.write_count)
                        .into_iter()
                        .map(|mask| (0..evaluation.table_summary.write_count).filter(|write| mask & (1u32 << write) != 0).collect())
                        .collect();
                    state_is_in_crash_states_of_segments(&segments, state).then(|| crash_states_of_segments(table, &segments))
                })
            };
            run_positive_controls_two_disk(emitter, &mut tally, label, &evaluation.world, &evaluation.truth, &evaluation.exact_reachable, &evaluation.arm_by_transaction, &find_stream);
        }
    }
    run_second_threshold_shifts(emitter, &mut tally, &judgements);
    run_second_geometry_sensitivity(emitter, &judgements);
    let writes_floor_ok = emit_real_shape_capacity(emitter, &mut tally);
    run_two_disk_dynamic_programming_checks(emitter, &mut tally, &judgements);
    run_second_failure_clauses(emitter, &judgements, writes_floor_ok);
    for judgement in &judgements {
        emit(
            emitter,
            format!(
                "name=verdict cell={} sampling_point={} arm_by_root_publish_l2_judgement={} arm_by_root_publish_cross_misses_minimum={} arm_by_root_publish_cross_misses_maximum={} arm_by_transaction_cross_misses={} arm_by_transaction_true_cross_misses={} arm_by_transaction_l3b_judgement={} exact_not_in_truth={} truth_not_in_exact={}",
                judgement.base.label,
                judgement.base.sampling_point,
                judgement.base.arm_by_root_publish_leak_judgement,
                judgement.base.arm_by_root_publish_minimum_cross_misses,
                judgement.base.arm_by_root_publish_maximum_cross_misses,
                judgement.l3a,
                judgement.l3b,
                leak_judgement(judgement.l3b, judgement.l3b, 0),
                judgement.exact_not_in_truth,
                judgement.truth_not_in_exact,
            ),
        );
    }
    tally
}

fn judgement_of<'judgements>(judgements: &'judgements [SecondJudgement], label: &str) -> &'judgements SecondJudgement {
    judgements.iter().find(|judgement| judgement.base.label == label).expect("这一格在第二段里跑过")
}

/// 判别力自证（登记第八节末）：L2、L3b、L6 的门槛各挪一次，判定必须翻；L3b 在两个几何上都是 0 时没有可翻的，如实写 not_applicable。
fn run_second_threshold_shifts(emitter: &mut Emitter, tally: &mut RunTally, judgements: &[SecondJudgement]) {
    let free = judgement_of(judgements, "g4a_free");
    let (minimum, maximum) = (free.base.arm_by_root_publish_minimum_cross_misses, free.base.arm_by_root_publish_maximum_cross_misses);
    let before = leak_judgement(minimum, maximum, 0);
    let after = leak_judgement(minimum, maximum, maximum);
    let flips = before != after && after == "never_leaks";
    emit(
        emitter,
        format!("name=threshold_shift quantity=l2_cross_misses cell=g4a_free threshold_before=0 judgement_before={before} threshold_after={maximum} judgement_after={after} flips_as_required={flips}"),
    );
    tally.self_check(flips, "threshold_shift:l2:g4a_free");
    for label in ["g4a_free", "g4b_free"] {
        let cell = judgement_of(judgements, label);
        if cell.l3b > 0 {
            let cell_leak_before = leak_judgement(cell.l3b, cell.l3b, 0);
            let cell_leak_after = leak_judgement(cell.l3b, cell.l3b, cell.l3b);
            let cell_leak_flips = cell_leak_before == "leaks_on_every_stream" && cell_leak_after == "never_leaks";
            emit(
                emitter,
                format!("name=threshold_shift quantity=l3b_true_cross_misses cell={label} threshold_before=0 judgement_before={cell_leak_before} threshold_after={} judgement_after={cell_leak_after} flips_as_required={cell_leak_flips}", cell.l3b),
            );
            tally.self_check(cell_leak_flips, &format!("threshold_shift:l3b:{label}"));
        } else {
            emit(
                emitter,
                format!("name=threshold_shift quantity=l3b_true_cross_misses cell={label} status=not_applicable reason=l3b_is_zero_so_there_is_no_positive_judgement_to_flip"),
            );
        }
    }
    // L6 的门槛是「要求覆盖的个数」：覆盖数 < 要求 记漏。两个方向各挪一次：本来判漏的挪到覆盖数（必须翻成覆盖），本来判覆盖的挪到覆盖数加一（必须翻成漏）。
    let required = free.base.recorded_truth_count;
    for (quantity, covered) in [("l6_recorded_prefixes_best_stream", free.best_stream_recorded_covered), ("l6_recorded_prefixes_arm_by_transaction", free.arm_recorded_covered)] {
        let invalid_prefix_before = invalid_prefix_judgement(covered, required);
        let required_after = if covered < required { covered } else { covered + 1 };
        let invalid_prefix_after = invalid_prefix_judgement(covered, required_after);
        let invalid_prefix_flips = invalid_prefix_before != invalid_prefix_after && (invalid_prefix_before == "misses_invalid_prefixes" || invalid_prefix_before == "covers_every_invalid_prefix");
        emit(
            emitter,
            format!("name=threshold_shift quantity={quantity} cell=g4a_free covered={covered} required_before={required} judgement_before={invalid_prefix_before} required_after={required_after} judgement_after={invalid_prefix_after} flips_as_required={invalid_prefix_flips}"),
        );
        tally.self_check(invalid_prefix_flips, &format!("threshold_shift:{quantity}:g4a_free"));
    }
}

fn run_second_geometry_sensitivity(emitter: &mut Emitter, judgements: &[SecondJudgement]) {
    let (roots_on_two_disks_free_judgement, roots_on_one_disk_free_judgement) = (judgement_of(judgements, "g4a_free"), judgement_of(judgements, "g4b_free"));
    let same = roots_on_two_disks_free_judgement.base.arm_by_root_publish_leak_judgement == roots_on_one_disk_free_judgement.base.arm_by_root_publish_leak_judgement && (roots_on_two_disks_free_judgement.l3b > 0) == (roots_on_one_disk_free_judgement.l3b > 0);
    emit(
        emitter,
        format!(
            "name=geometry_sensitivity pair=G4a_vs_G4b_free same_judgement={} l2_judgement_g4a={} l2_judgement_g4b={} cross_misses_minimum_g4a={} cross_misses_minimum_g4b={} cross_misses_maximum_g4a={} cross_misses_maximum_g4b={} l3a_g4a={} l3a_g4b={} l3b_g4a={} l3b_g4b={} recorded_truth_g4a={} recorded_truth_g4b={} all_writes_truth_g4a={} all_writes_truth_g4b={}",
            yes_no(same),
            roots_on_two_disks_free_judgement.base.arm_by_root_publish_leak_judgement,
            roots_on_one_disk_free_judgement.base.arm_by_root_publish_leak_judgement,
            roots_on_two_disks_free_judgement.base.arm_by_root_publish_minimum_cross_misses,
            roots_on_one_disk_free_judgement.base.arm_by_root_publish_minimum_cross_misses,
            roots_on_two_disks_free_judgement.base.arm_by_root_publish_maximum_cross_misses,
            roots_on_one_disk_free_judgement.base.arm_by_root_publish_maximum_cross_misses,
            roots_on_two_disks_free_judgement.l3a,
            roots_on_one_disk_free_judgement.l3a,
            roots_on_two_disks_free_judgement.l3b,
            roots_on_one_disk_free_judgement.l3b,
            roots_on_two_disks_free_judgement.base.recorded_truth_count,
            roots_on_one_disk_free_judgement.base.recorded_truth_count,
            roots_on_two_disks_free_judgement.base.all_writes_truth_count,
            roots_on_one_disk_free_judgement.base.all_writes_truth_count,
        ),
    );
    for (free_label, ordered_label, pair_name) in [("g4a_free", "g4a_ordered", "G4a_vs_G4a_ordered"), ("g4b_free", "g4b_ordered", "G4b_vs_G4b_ordered")] {
        let (free, ordered) = (judgement_of(judgements, free_label), judgement_of(judgements, ordered_label));
        let pair_leak_judgement_same = free.base.arm_by_root_publish_leak_judgement == ordered.base.arm_by_root_publish_leak_judgement && (free.l3b > 0) == (ordered.l3b > 0);
        emit(
            emitter,
            format!(
                "name=geometry_sensitivity pair={pair_name} same_judgement={} l2_judgement_free={} l2_judgement_ordered={} cross_misses_minimum_free={} cross_misses_minimum_ordered={} cross_misses_maximum_free={} cross_misses_maximum_ordered={} l3a_free={} l3a_ordered={} l3b_free={} l3b_ordered={} recorded_truth_free={} recorded_truth_ordered={} all_writes_truth_free={} all_writes_truth_ordered={}",
                yes_no(pair_leak_judgement_same),
                free.base.arm_by_root_publish_leak_judgement,
                ordered.base.arm_by_root_publish_leak_judgement,
                free.base.arm_by_root_publish_minimum_cross_misses,
                ordered.base.arm_by_root_publish_minimum_cross_misses,
                free.base.arm_by_root_publish_maximum_cross_misses,
                ordered.base.arm_by_root_publish_maximum_cross_misses,
                free.l3a,
                ordered.l3a,
                free.l3b,
                ordered.l3b,
                free.base.recorded_truth_count,
                ordered.base.recorded_truth_count,
                free.base.all_writes_truth_count,
                ordered.base.all_writes_truth_count,
            ),
        );
    }
}

/// 两盘几何 Σ_free 上 K1 最大一段：动态规划与「从全部段签名里取最大」两条路逐项对拍（L8 的小流版本，核动态规划本身）。
fn run_two_disk_dynamic_programming_checks(emitter: &mut Emitter, tally: &mut RunTally, judgements: &[SecondJudgement]) {
    for (geometry, labels, _, _) in second_segment_geometries() {
        let free = judgement_of(judgements, labels[2]);
        let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry);
        let by_states = largest_segment_of(&world, SegmentObjective::Expansion);
        let by_writes = largest_segment_of(&world, SegmentObjective::Writes);
        let agrees = by_states.value == free.root_slot_cut_largest_segment_expansion && by_writes.value == free.root_slot_cut_largest_segment_writes as u128 && by_states.replay_agrees && by_writes.replay_agrees;
        emit(
            emitter,
            format!(
                "name=capacity_dynamic_programming_check geometry={} dynamic_programming_largest_expansion={} signatures_largest_expansion={} dynamic_programming_most_writes={} signatures_most_writes={} agrees={agrees}",
                geometry.label, by_states.value, free.root_slot_cut_largest_segment_expansion, by_writes.value, free.root_slot_cut_largest_segment_writes,
            ),
        );
        tally.self_check(agrees, &format!("L8:dynamic_programming_vs_signatures:{}", geometry.label));
    }
}

fn run_second_failure_clauses(emitter: &mut Emitter, judgements: &[SecondJudgement], writes_floor_ok: bool) {
    let exact_not_in_truth_triggered = judgements.iter().any(|judgement| judgement.exact_not_in_truth > 0);
    emit(emitter, format!("name=failure_clause clause=F1 triggered={} observation=exact_not_in_truth_over_the_eight_two_disk_cells", yes_no(exact_not_in_truth_triggered)));
    for (free_label, ordered_label) in [("g4a_free", "g4a_ordered"), ("g4b_free", "g4b_ordered")] {
        for label in [free_label, ordered_label] {
            let cell = judgement_of(judgements, label);
            emit(
                emitter,
                format!(
                    "name=failure_clause clause=F2 cell={label} triggered={} l2_minimum={} l3b={}",
                    yes_no(cell.base.arm_by_root_publish_minimum_cross_misses > 0 && cell.l3b > 0),
                    cell.base.arm_by_root_publish_minimum_cross_misses,
                    cell.l3b,
                ),
            );
            emit(
                emitter,
                format!("name=failure_clause clause=F5 cell={label} triggered={} l3a={} l3b={}", yes_no(cell.l3a > 0 && cell.l3b == 0), cell.l3a, cell.l3b),
            );
        }
        let (free, ordered) = (judgement_of(judgements, free_label), judgement_of(judgements, ordered_label));
        emit(
            emitter,
            format!(
                "name=failure_clause clause=F3 pair={free_label}_vs_{ordered_label} triggered={}",
                yes_no(free.base.arm_by_root_publish_leak_judgement != ordered.base.arm_by_root_publish_leak_judgement)
            ),
        );
    }
    for label in ["g4a_free", "g4b_free", "g4a_ordered", "g4b_ordered"] {
        let cell = judgement_of(judgements, label);
        emit(
            emitter,
            format!(
                "name=failure_clause clause=F4 cell={label} recorded_truth_count={} all_writes_truth_count={} triggered_recorded={} triggered_all_writes={}",
                cell.base.recorded_truth_count,
                cell.base.all_writes_truth_count,
                yes_no(cell.base.recorded_truth_count == 0),
                yes_no(cell.base.all_writes_truth_count == 0),
            ),
        );
    }
    emit(
        emitter,
        format!("name=failure_clause clause=F6 triggered={} observation=most_writes_in_one_segment_on_the_real_publish_shape_below_48", yes_no(!writes_floor_ok)),
    );
}

// ── 第三段补一格：L11 K1 对 T ∩ T_exact ∩ C 的跨事务漏 ───────────────────────────────────
//
// 登记第十二节修订 10：K1 在一条录制流 σ 上的 L11(σ) = |(T ∩ T_exact ∩ C) \ Dom4(σ)|，与 L3b 同口径（参照取 T ∩ T_exact，只数跨事务状态）。
// 每个段签名的枚举域装置里已有；这里不再存枚举域，换两条做法：(A) 目标集里每个状态逐个按「存在一个当前段」的定义判在不在这串段里；
// (B) 把目标集的混合进制序号放进位集，对每个签名枚举 Dom4(σ) 数命中。两条逐签名相等才算数。

/// 每段的（两位一写的写位掩码，各写取 1 的掩码）。
pub(super) fn segment_digit_masks(segments: &[u32]) -> Vec<(u64, u64)> {
    segments
        .iter()
        .map(|segment| {
            let (mut any, mut ones) = (0u64, 0u64);
            for write in 0..MAXIMUM_STREAMING_WRITES {
                if segment & (1u32 << write) != 0 {
                    any |= 0b11u64 << (2 * write);
                    ones |= 0b01u64 << (2 * write);
                }
            }
            (any, ones)
        })
        .collect()
}

/// 方法 A：一个状态在不在一串段的枚举域里（存在一个当前段，前面各段全持久、后面各段全没持久）；全零状态总在。
pub(super) fn code_in_digit_masks(masks: &[(u64, u64)], code: u64) -> bool {
    code == 0
        || (0..masks.len()).any(|current| {
            masks.iter().enumerate().all(|(position, &(any, ones))| {
                if position < current {
                    code & any == ones
                } else if position > current {
                    code & any == 0
                } else {
                    true
                }
            })
        })
}

/// 目标集里不在这串段枚举域里的个数；`removed` 是阳性对照里「从枚举域里删掉」的那个状态。
pub(super) fn true_cross_misses_of_masks(masks: &[(u64, u64)], target: &[u64], removed: Option<u64>) -> usize {
    target.iter().filter(|&&code| Some(code) == removed || !code_in_digit_masks(masks, code)).count()
}

/// T ∩ T_exact ∩ C（用 `is_cross_transaction` 与排好序的 T、T_exact 查，不用掩码版谓词）。
pub(super) fn true_cross_target(table: &[WriteSpecification], truth: &[StateCode], exact: &[StateCode]) -> Vec<u64> {
    truth
        .iter()
        .copied()
        .filter(|state| is_cross_transaction(table, *state) && contains(exact, *state))
        .map(|state| u64::try_from(state).expect("两盘几何的状态编码装得进 u64"))
        .collect()
}

/// 方法 B：对每个段签名枚举 Dom4(σ)，数落在目标集里的个数，返回目标集减它。
fn true_cross_misses_by_enumeration(table_summary: &WriteTableSummary, rows: &[SignatureRow], target: &[u64], threads: usize) -> Vec<u32> {
    let mut target_bits = Bitset::new(table_summary.total_vectors);
    for &code in target {
        target_bits.insert_index(table_summary.index_of_code(code));
    }
    const CHUNK: usize = 64;
    let next_start = AtomicUsize::new(0);
    let parts: Vec<Vec<(usize, u32)>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut found = Vec::new();
                    loop {
                        let start = next_start.fetch_add(CHUNK, AtomicOrdering::Relaxed);
                        if start >= rows.len() {
                            break;
                        }
                        for (position, row) in rows.iter().enumerate().take((start + CHUNK).min(rows.len())).skip(start) {
                            let segments = segments_of_signature(row.key, table_summary.write_count);
                            let mut hits = 0usize;
                            for_each_crash_state(table_summary, &segments, |_, index| {
                                if target_bits.has(index) {
                                    hits += 1;
                                }
                            });
                            found.push((position, u32::try_from(target.len() - hits).expect("漏数装得进 u32")));
                        }
                    }
                    found
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("线程不 panic")).collect()
    });
    let mut misses = vec![0u32; rows.len()];
    for (position, value) in parts.into_iter().flatten() {
        misses[position] = value;
    }
    misses
}

/// 一格上 K1 的 L11 读数。
pub(super) struct TrueCrossRead {
    pub(super) target: Vec<u64>,
    /// 方法 A 的逐签名漏数（与 `rows` 同序）。
    pub(super) misses: Vec<u32>,
    pub(super) methods_agree: bool,
    pub(super) minimum: u32,
    pub(super) minimum_row: usize,
    pub(super) maximum: u32,
    pub(super) maximum_row: usize,
    pub(super) streams_with_misses: u64,
    signatures_with_misses: usize,
    pub(super) serial_misses: usize,
    pub(super) serial_agrees_across_methods: bool,
}

pub(super) fn read_root_slot_cut_true_cross(evaluation: &StreamingEvaluation, threads: usize) -> TrueCrossRead {
    let table = &evaluation.world.table;
    let rows = &evaluation.rows;
    let table_summary = &evaluation.table_summary;
    let target = true_cross_target(table, &evaluation.truth, &evaluation.exact_reachable);
    let misses: Vec<u32> = rows
        .iter()
        .map(|row| {
            let masks = segment_digit_masks(&segments_of_signature(row.key, table_summary.write_count));
            u32::try_from(true_cross_misses_of_masks(&masks, &target, None)).expect("漏数装得进 u32")
        })
        .collect();
    let methods_agree = misses == true_cross_misses_by_enumeration(table_summary, rows, &target, threads);
    // 取最小与最大时并列的，取字典序最小的那条流（与 `emit_streaming_cell` 取最小值那条流同一规矩）。
    let better = |candidate: usize, current: usize, prefer_larger: bool| {
        let (candidate_misses, current_misses) = (misses[candidate], misses[current]);
        let strictly = if prefer_larger { candidate_misses > current_misses } else { candidate_misses < current_misses };
        strictly || (candidate_misses == current_misses && rows[candidate].representative_path < rows[current].representative_path)
    };
    let (mut minimum_row, mut maximum_row) = (0usize, 0usize);
    for position in 0..rows.len() {
        if better(position, minimum_row, false) {
            minimum_row = position;
        }
        if better(position, maximum_row, true) {
            maximum_row = position;
        }
    }
    let serial = evaluation.world.serial_stream.as_ref().expect("两条链的世界有串行流");
    let serial_segments = write_segments(table, serial);
    let serial_masks: Vec<u32> = serial_segments.iter().map(|segment| segment.iter().fold(0u32, |mask, &write| mask | (1u32 << write))).collect();
    let by_masks = true_cross_misses_of_masks(&segment_digit_masks(&serial_masks), &target, None);
    let by_table = target.iter().filter(|&&code| !state_is_in_crash_states_of_segments(&serial_segments, StateCode::from(code))).count();
    TrueCrossRead {
        minimum: misses[minimum_row],
        maximum: misses[maximum_row],
        streams_with_misses: rows.iter().zip(&misses).filter(|(_, &miss)| miss > 0).map(|(row, _)| row.count).sum(),
        signatures_with_misses: misses.iter().filter(|&&miss| miss > 0).count(),
        serial_misses: by_masks,
        serial_agrees_across_methods: by_masks == by_table,
        minimum_row,
        maximum_row,
        methods_agree,
        misses,
        target,
    }
}


/// 第三段每格要留到末尾判决用的读数。
pub(super) struct TrueCrossJudgement {
    pub(super) label: &'static str,
    pub(super) sampling_point: &'static str,
    pub(super) minimum: usize,
    pub(super) maximum: usize,
    pub(super) streams_with_misses: u64,
    pub(super) arm_by_transaction_true_cross_misses: usize,
}

/// 第三段的一格：L11 的两条算法、与第二段 L2 的逐签名比、阳性对照 PC1（串行拼接）与 PC2（删一态）。
pub(super) fn emit_true_cross_cell(emitter: &mut Emitter, tally: &mut RunTally, evaluation: &StreamingEvaluation, threads: usize) -> TrueCrossJudgement {
    let (label, table, rows, table_summary) = (evaluation.label, &evaluation.world.table, &evaluation.rows, &evaluation.table_summary);
    tally.self_check(evaluation.truth == evaluation.truth_by_search, &format!("V3:truth_methods:{label}"));
    let read = read_root_slot_cut_true_cross(evaluation, threads);
    let target_size = read.target.len();
    emit(
        emitter,
        format!(
            "name=true_cross_reference cell={label} sampling_point={} candidate={} truth_states={} exact_states={} truth_and_exact_states={} truth_cross_states={} truth_and_exact_cross_states={target_size}",
            evaluation.sampling_point,
            evaluation.world.candidate.name(),
            evaluation.truth.len(),
            evaluation.exact_reachable.len(),
            evaluation.truth.len() - difference_count(&evaluation.truth, &evaluation.exact_reachable),
            evaluation.truth_cross,
        ),
    );
    tally.self_check(read.methods_agree, &format!("V5:l11_two_methods:{label}"));
    tally.self_check(read.serial_agrees_across_methods, &format!("V5:l11_serial_two_membership_tests:{label}"));
    let misses_not_above_the_cross_bound = rows.iter().zip(&read.misses).all(|(row, &miss)| miss as usize <= evaluation.truth_cross - row.cross_states as usize);
    tally.self_check(misses_not_above_the_cross_bound && target_size <= evaluation.truth_cross, &format!("V5:l11_not_above_l2:{label}"));
    let (arm, exact) = (&evaluation.arm_by_transaction, &evaluation.exact_reachable);
    let l3b = true_cross_misses(table, &evaluation.truth, exact, arm);
    let streams_in_rows: u64 = rows.iter().map(|row| row.count).sum();
    tally.self_check(u128::from(streams_in_rows) == evaluation.expected_leaves, &format!("V3:interleaving_count:{label}"));
    emit(
        emitter,
        format!(
            "name=arm_by_root_publish_true_cross cell={label} streams={streams_in_rows} signatures={} true_cross_states={target_size} true_cross_misses_minimum={} true_cross_misses_maximum={} streams_with_true_cross_misses={} streams_with_zero_true_cross_misses={} signatures_with_true_cross_misses={} true_cross_misses_on_serial_stream={} minimum_true_cross_miss_stream={} maximum_true_cross_miss_stream={} two_methods_agree={} judgement={} arm_by_transaction_true_cross_misses={l3b}",
            rows.len(),
            read.minimum,
            read.maximum,
            read.streams_with_misses,
            streams_in_rows - read.streams_with_misses,
            read.signatures_with_misses,
            read.serial_misses,
            render_path(&evaluation.world, rows[read.minimum_row].representative_path),
            render_path(&evaluation.world, rows[read.maximum_row].representative_path),
            yes_no(read.methods_agree),
            leak_judgement(read.minimum as usize, read.maximum as usize, 0),
        ),
    );
    // PC1：串行拼接那一条流在 K1 上必须漏掉见证状态 {a1, b1}（A、B 各只有第一个单元写持久：跨事务，T、T_exact 里都有；单独的 b1 不是跨事务状态，不当见证）。
    let older_unit_write = first_write_name(table, Owner::OlderTransaction, StepKind::UnitWrite);
    let newer_unit_write = first_write_name(table, Owner::NewerTransaction, StepKind::UnitWrite);
    let witness = state_with_only(table, &[older_unit_write.as_str(), newer_unit_write.as_str()]);
    let witness_code = u64::try_from(witness).expect("两盘几何的状态编码装得进 u64");
    let serial = evaluation.world.serial_stream.as_ref().expect("两条链的世界有串行流");
    let witness_in_target = read.target.contains(&witness_code);
    let witness_missed = !state_is_in_crash_states_of_segments(&write_segments(table, serial), witness);
    let pc1_ok = witness_in_target && witness_missed && read.serial_misses >= 1;
    emit(
        emitter,
        format!(
            "name=positive_control control=serial_concatenation_l11 cell={label} witness={} witness_in_true_cross_target={witness_in_target} witness_missed={witness_missed} true_cross_misses_on_serial_stream={} ok={pc1_ok}",
            render_state(table, witness),
            read.serial_misses,
        ),
    );
    tally.self_check(pc1_ok, &format!("V2:PC1_l11:{label}"));
    // PC2：从取到最小漏数的那条流的枚举域里删一个目标集里的状态，L11 恰好多 1。
    let minimum_masks = segment_digit_masks(&segments_of_signature(rows[read.minimum_row].key, table_summary.write_count));
    match read.target.iter().copied().find(|&code| code_in_digit_masks(&minimum_masks, code)) {
        Some(removed) => {
            let before = true_cross_misses_of_masks(&minimum_masks, &read.target, None);
            let after = true_cross_misses_of_masks(&minimum_masks, &read.target, Some(removed));
            let pc2_ok = after == before + 1 && before == read.minimum as usize;
            emit(
                emitter,
                format!("name=positive_control control=delete_one_true_cross_state_from_k1 cell={label} deleted={} true_cross_misses_before={before} true_cross_misses_after={after} ok={pc2_ok}", render_state(table, StateCode::from(removed))),
            );
            tally.self_check(pc2_ok, &format!("V2:PC2_l11:{label}"));
        }
        None => emit(
            emitter,
            format!("name=positive_control control=delete_one_true_cross_state_from_k1 cell={label} status=not_applicable reason=minimum_stream_covers_none_of_the_target"),
        ),
    }
    TrueCrossJudgement {
        label,
        sampling_point: evaluation.sampling_point,
        minimum: read.minimum as usize,
        maximum: read.maximum as usize,
        streams_with_misses: read.streams_with_misses,
        arm_by_transaction_true_cross_misses: l3b,
    }
}

/// 判别力自证：L11 的门槛从 0 挪到最大值，「某些流上漏」必须翻成「不漏」；最大值是 0 时没有可翻的，如实写 not_applicable。
pub(super) fn emit_true_cross_threshold_shifts(emitter: &mut Emitter, tally: &mut RunTally, judgements: &[TrueCrossJudgement]) {
    for judgement in judgements {
        let label = judgement.label;
        if judgement.maximum > 0 {
            let before = leak_judgement(judgement.minimum, judgement.maximum, 0);
            let after = leak_judgement(judgement.minimum, judgement.maximum, judgement.maximum);
            let flips = before != "never_leaks" && after == "never_leaks";
            emit(
                emitter,
                format!("name=threshold_shift quantity=l11_true_cross_misses cell={label} threshold_before=0 judgement_before={before} threshold_after={} judgement_after={after} flips_as_required={flips}", judgement.maximum),
            );
            tally.self_check(flips, &format!("threshold_shift:l11:{label}"));
        } else {
            emit(
                emitter,
                format!("name=threshold_shift quantity=l11_true_cross_misses cell={label} status=not_applicable reason=l11_maximum_is_zero_so_there_is_no_positive_judgement_to_flip"),
            );
        }
    }
}

pub(super) fn run_third_segment(emitter: &mut Emitter) -> RunTally {
    let mut tally = RunTally::default();
    let threads = worker_thread_count();
    emit(
        emitter,
        "name=config model=independent_count_model candidates=2 sampling_points=G4a,G4a_ordered,G4b,G4b_ordered segment=third quantities=L11 reuses=L1,T_exact,L3b not_run=L2,L3a,L4,L5,L6,L7,L8,L9,L10,G5".to_string(),
    );
    let mut judgements: Vec<TrueCrossJudgement> = Vec::new();
    for (geometry, labels, free_point, ordered_point) in second_segment_geometries() {
        for (label, candidate, point) in [(labels[2], Candidate::ManyPublishesInFlightFree, free_point), (labels[3], Candidate::ManyPublishesInFlightOrdered, ordered_point)] {
            let specification = CellSpecification { label, sampling_point: point, geometry: geometry.clone(), candidate };
            let evaluation = evaluate_streaming_cell(&specification, threads);
            judgements.push(emit_true_cross_cell(emitter, &mut tally, &evaluation, threads));
        }
    }
    emit_true_cross_threshold_shifts(emitter, &mut tally, &judgements);
    let by_label = |label: &str| judgements.iter().find(|judgement| judgement.label == label).expect("这一格在第三段里跑过");
    for (left, right, pair) in [("g4a_free", "g4b_free", "G4a_vs_G4b_free"), ("g4a_free", "g4a_ordered", "G4a_vs_G4a_ordered"), ("g4b_free", "g4b_ordered", "G4b_vs_G4b_ordered")] {
        let (left_judgement, right_judgement) = (by_label(left), by_label(right));
        let (left_cell_judgement, right_cell_judgement) = (leak_judgement(left_judgement.minimum, left_judgement.maximum, 0), leak_judgement(right_judgement.minimum, right_judgement.maximum, 0));
        emit(
            emitter,
            format!(
                "name=geometry_sensitivity quantity=l11 pair={pair} same_judgement={} l11_judgement_left={left_cell_judgement} l11_judgement_right={right_cell_judgement} l11_minimum_left={} l11_minimum_right={} l11_maximum_left={} l11_maximum_right={} l11_streams_with_misses_left={} l11_streams_with_misses_right={}",
                yes_no(left_cell_judgement == right_cell_judgement),
                left_judgement.minimum,
                right_judgement.minimum,
                left_judgement.maximum,
                right_judgement.maximum,
                left_judgement.streams_with_misses,
                right_judgement.streams_with_misses,
            ),
        );
    }
    for judgement in &judgements {
        emit(
            emitter,
            format!(
                "name=verdict cell={} sampling_point={} arm_by_root_publish_l11_judgement={} arm_by_root_publish_true_cross_misses_minimum={} arm_by_root_publish_true_cross_misses_maximum={} arm_by_root_publish_streams_with_true_cross_misses={} arm_by_transaction_true_cross_misses={} arm_by_transaction_l3b_judgement={}",
                judgement.label,
                judgement.sampling_point,
                leak_judgement(judgement.minimum, judgement.maximum, 0),
                judgement.minimum,
                judgement.maximum,
                judgement.streams_with_misses,
                judgement.arm_by_transaction_true_cross_misses,
                leak_judgement(judgement.arm_by_transaction_true_cross_misses, judgement.arm_by_transaction_true_cross_misses, 0),
            ),
        );
    }
    tally
}

// ── 登记 S3 要的规模估计 ──────────────────────────────────────────────────────────────────

/// 子命令 `s3`：G4a、G4b 上 Σ_free 与 Σ_ord 的交错数（两条算法数）、去掉同一段里写序之后的不同段签名数、
/// 不同的偏序数、逐状态深搜的状态全集与每隔 1009 个向量抽一个跑搜索的访问量（只用来估规模，不进任何判据）。
pub(super) fn run_scale_estimate(emitter: &mut Emitter) {
    for (geometry, labels, _, _) in second_segment_geometries() {
        for (label, candidate) in [(labels[2], Candidate::ManyPublishesInFlightFree), (labels[3], Candidate::ManyPublishesInFlightOrdered)] {
            let started = std::time::Instant::now();
            let world = build_two_chain_world_without_streams(candidate, &geometry);
            let table_summary = WriteTableSummary::new(&world.table);
            let context = WalkContext::new(&world, &table_summary);
            let expected = count_interleavings(&context);
            let pass = pass_one(&context, worker_thread_count());
            eprintln!("s3 {label}: pass one {:?}", started.elapsed());
            let search_started = std::time::Instant::now();
            let stride = 1009u64;
            let (vectors, visited_total, visited_maximum, realized) = search_sample(&context, &table_summary, stride);
            eprintln!("s3 {label}: search sample {:?}", search_started.elapsed());
            let space = (context.older_steps.len() + 1) * (context.newer_steps.len() + 1) * (1usize << table_summary.disk_count) * 4 * 3;
            emit(
                emitter,
                format!(
                    "name=s3_estimate cell={label} interleavings={} interleavings_by_lattice_count={expected} counts_agree={} distinct_segment_signatures={} distinct_order_masks={} state_vector_universe={} search_state_space={space} sample_stride={stride} sampled_vectors={vectors} sampled_visited_search_states={visited_total} sampled_visited_search_states_maximum={visited_maximum} sampled_realized={realized}",
                    pass.leaves,
                    yes_no(u128::from(pass.leaves) == expected),
                    pass.signatures.len(),
                    pass.masks.len(),
                    table_summary.total_vectors,
                ),
            );
        }
    }
}
