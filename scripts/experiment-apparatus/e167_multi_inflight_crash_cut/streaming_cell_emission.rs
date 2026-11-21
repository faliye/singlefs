//! 流式的逐格输出、覆盖行、两盘几何的阳性对照与最大段的搜索。

use e7_index_bench::u32_from_usize;
use super::*;

/// 第二段的一格（两条链的交错，流式）：行的字段与 `emit_cell` 一一对应，K1 一侧按段签名带条数汇总。
pub(super) fn emit_streaming_cell(emitter: &mut Emitter, tally: &mut RunTally, evaluation: &StreamingEvaluation) -> SecondJudgement {
    let world = &evaluation.world;
    let table = &world.table;
    let label = evaluation.label;
    let truth = &evaluation.truth;
    let rows = &evaluation.rows;
    let methods_agree = *truth == evaluation.truth_by_search;
    tally.self_check(methods_agree, &format!("V3:truth_methods:{label}"));
    let streams_in_rows: u128 = rows.iter().map(|row| u128::from(row.count)).sum();
    let streams_agree = u128::from(evaluation.leaves) == evaluation.expected_leaves && streams_in_rows == evaluation.expected_leaves;
    tally.self_check(streams_agree, &format!("V3:interleaving_count:{label}"));
    emit(
        emitter,
        format!(
            "name=cell_truth cell={label} sampling_point={} candidate={} disks={} units={} table_writes={} streams={} truth_by_enumeration={} truth_by_search={} methods_agree={methods_agree} cross_transaction_states={}",
            evaluation.sampling_point,
            world.candidate.name(),
            world.geometry.disk_count,
            world.geometry.units_per_transaction,
            table.len(),
            evaluation.leaves,
            truth.len(),
            evaluation.truth_by_search.len(),
            evaluation.truth_cross,
        ),
    );
    emit(
        emitter,
        format!(
            "name=stream_classes cell={label} interleavings={} interleavings_by_lattice_count={} counts_agree={streams_agree} distinct_segment_signatures={} distinct_order_masks={} state_vector_universe={}",
            evaluation.leaves,
            evaluation.expected_leaves,
            rows.len(),
            evaluation.order_masks,
            evaluation.table_summary.total_vectors,
        ),
    );
    let subset_ok = rows.iter().all(|row| row.cross_states as usize <= evaluation.truth_cross && (row.states - row.cross_states) as usize <= truth.len() - evaluation.truth_cross);
    tally.self_check(subset_ok, &format!("V5:k1_domain_inside_truth:{label}"));
    let truth_non_cross = truth.len() - evaluation.truth_cross;
    let cross_misses_of = |row: &SignatureRow| evaluation.truth_cross - row.cross_states as usize;
    let non_cross_misses_of = |row: &SignatureRow| truth_non_cross - (row.states - row.cross_states) as usize;
    let mut minimum_row = &rows[0];
    for row in rows {
        let (candidate_misses, best_misses) = (cross_misses_of(row), cross_misses_of(minimum_row));
        if candidate_misses < best_misses || (candidate_misses == best_misses && row.representative_path < minimum_row.representative_path) {
            minimum_row = row;
        }
    }
    let minimum_cross_misses = cross_misses_of(minimum_row);
    let maximum_cross_misses = rows.iter().map(&cross_misses_of).max().expect("至少一个段签名");
    let weighted = |select: &dyn Fn(&SignatureRow) -> bool| -> u64 { rows.iter().filter(|row| select(row)).map(|row| row.count).sum() };
    let closed_form_mismatches = weighted(&|row| !row.closed_form_matches);
    tally.self_check(closed_form_mismatches == 0, &format!("V6:closed_form:{label}"));
    let serial_stream = world.serial_stream.as_ref().expect("两条链的世界有串行流");
    let serial_states = closed_form_state_count(table, &write_segments(table, serial_stream));
    emit(
        emitter,
        format!(
            "name=arm_by_root_publish cell={label} streams={} states_minimum={} states_maximum={} states_on_serial_stream={serial_states} states_on_minimum_cross_miss_stream={} closed_form_mismatches={closed_form_mismatches} cross_misses_minimum={minimum_cross_misses} cross_misses_maximum={maximum_cross_misses} streams_with_cross_misses={} minimum_cross_miss_stream={} non_cross_misses_maximum={} streams_with_non_cross_misses={}",
            evaluation.leaves,
            rows.iter().map(|row| row.states).min().expect("至少一个段签名"),
            rows.iter().map(|row| row.states).max().expect("至少一个段签名"),
            minimum_row.states,
            weighted(&|row| cross_misses_of(row) > 0),
            render_path(world, minimum_row.representative_path),
            rows.iter().map(&non_cross_misses_of).max().expect("至少一个段签名"),
            weighted(&|row| non_cross_misses_of(row) > 0),
        ),
    );
    let arm = &evaluation.arm_by_transaction;
    let exact = &evaluation.exact_reachable;
    let cross_misses_of_arm = count_cross_transaction_misses(table, truth, arm);
    let true_cross_misses_of_arm = true_cross_misses(table, truth, exact, arm);
    let extra_states = difference_count(arm, truth);
    let non_cross_misses_of_arm = count_non_cross_misses(table, truth, arm);
    let identity_ok = arm.len() + cross_misses_of_arm + non_cross_misses_of_arm == truth.len() + extra_states;
    tally.self_check(identity_ok, &format!("V5:identity:{label}"));
    emit(
        emitter,
        format!(
            "name=arm_by_transaction cell={label} states={} cross_misses={cross_misses_of_arm} true_cross_misses={true_cross_misses_of_arm} non_cross_misses={non_cross_misses_of_arm} states_not_in_truth={extra_states} identity_ok={identity_ok}",
            arm.len(),
        ),
    );
    let mut recorded_truth_count = 0;
    let mut all_writes_truth_count = 0;
    let mut best_stream_recorded_covered = 0;
    let mut arm_recorded_covered = 0;
    for kind in BOTH_INVALID_PREFIX_KINDS {
        let truth_count = count_invalid_prefixes_in(table, truth, kind);
        let covered_of = |row: &SignatureRow| match kind {
            InvalidPrefixKind::ByRecord => row.invalid_record_states as usize,
            InvalidPrefixKind::ByAllWrites => row.invalid_all_writes_states as usize,
        };
        let covered_minimum = rows.iter().map(&covered_of).min().expect("至少一个段签名");
        let covered_maximum = rows.iter().map(&covered_of).max().expect("至少一个段签名");
        let arm_covered = count_invalid_prefixes_covered(table, truth, arm, kind);
        let arm_unreachable = count_invalid_prefixes_in(table, arm, kind) - arm_covered;
        let truth_count_by_masks = match kind {
            InvalidPrefixKind::ByRecord => evaluation.truth_invalid_record,
            InvalidPrefixKind::ByAllWrites => evaluation.truth_invalid_all_writes,
        };
        tally.self_check(truth_count == truth_count_by_masks, &format!("V5:invalid_prefix_masks:{label}:{}", kind.name()));
        match kind {
            InvalidPrefixKind::ByRecord => {
                recorded_truth_count = truth_count;
                best_stream_recorded_covered = covered_maximum;
                arm_recorded_covered = arm_covered;
            }
            InvalidPrefixKind::ByAllWrites => all_writes_truth_count = truth_count,
        }
        emit(
            emitter,
            format!(
                "name=invalid_prefix cell={label} kind={} truth_count={truth_count} arm_by_root_publish_covered_minimum={covered_minimum} arm_by_root_publish_covered_maximum={covered_maximum} arm_by_transaction_covered={arm_covered} arm_by_transaction_not_in_truth={arm_unreachable} arm_by_root_publish_judgement={} arm_by_transaction_judgement={}",
                kind.name(),
                invalid_prefix_judgement(covered_minimum, truth_count),
                invalid_prefix_judgement(arm_covered, truth_count),
            ),
        );
    }
    emit_cover_lines_of(emitter, tally, label, table, truth, arm);
    let largest_writes = rows.iter().map(|row| usize::from(row.largest_segment_writes)).max().expect("至少一个段签名");
    let largest_expansion = rows.iter().map(|row| u128::from(row.largest_segment_expansion)).max().expect("至少一个段签名");
    let total_states_maximum = rows.iter().map(|row| row.states).max().expect("至少一个段签名");
    let cover = cover_by_merging(table, arm);
    let cover_largest_expansion = cover.segments.iter().map(|segment| segment_expansion(table, segment)).max().unwrap_or(0);
    emit(
        emitter,
        format!(
            "name=capacity cell={label} table_writes={} arm_by_root_publish_largest_segment_writes={largest_writes} arm_by_root_publish_largest_segment_states={largest_expansion} arm_by_root_publish_total_states_maximum={total_states_maximum} arm_by_transaction_cover_largest_segment_states={cover_largest_expansion} arm_by_transaction_cover_total_states={} writes_within_limit={} segments_within_u32={} segments_within_u64={}",
            table.len(),
            cover.cover_state_count,
            table.len() <= MAXIMUM_ENUMERATED_WRITES,
            largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U32 && cover_largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U32,
            largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U64 && cover_largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U64,
        ),
    );
    let exact_not_in_truth = difference_count(exact, truth);
    let truth_not_in_exact = difference_count(truth, exact);
    emit(emitter, format!("name=reference cell={label} exact_states={} exact_not_in_truth={exact_not_in_truth} truth_not_in_exact={truth_not_in_exact}", exact.len()));
    SecondJudgement {
        base: CellJudgement {
            label,
            sampling_point: evaluation.sampling_point,
            candidate: world.candidate,
            disks: world.geometry.disk_count,
            arm_by_root_publish_leak_judgement: leak_judgement(minimum_cross_misses, maximum_cross_misses, 0),
            arm_by_root_publish_minimum_cross_misses: minimum_cross_misses,
            arm_by_root_publish_maximum_cross_misses: maximum_cross_misses,
            arm_by_transaction_cross_misses: cross_misses_of_arm,
            recorded_truth_count,
            all_writes_truth_count,
        },
        l3a: cross_misses_of_arm,
        l3b: true_cross_misses_of_arm,
        exact_not_in_truth,
        truth_not_in_exact,
        best_stream_recorded_covered,
        arm_recorded_covered,
        root_slot_cut_largest_segment_writes: largest_writes,
        root_slot_cut_largest_segment_expansion: largest_expansion,
    }
}

/// `emit_cover_lines` 的不依赖 `CellEvaluation` 的版本（写数 > 8，有序划分穷举不做）。
fn emit_cover_lines_of(emitter: &mut Emitter, tally: &mut RunTally, label: &str, table: &[WriteSpecification], truth: &StateList, arm: &StateList) {
    for (set_name, target) in [("truth", truth), ("arm_by_transaction", arm)] {
        let cover = cover_by_merging(table, target);
        tally.self_check(cover.covers_target, &format!("L7a:cover_covers_target:{label}:{set_name}"));
        emit(
            emitter,
            format!(
                "name=cover cell={label} set={set_name} target_states={} cover_segments={} cover_largest_segment_writes={} cover_states={} covers_target={} cover_equals_target={} exhaustive_agrees=not_run_more_than_8_writes",
                target.len(),
                cover.segments.len(),
                cover.segments.iter().map(Vec::len).max().unwrap_or(0),
                cover.cover_state_count,
                cover.covers_target,
                cover.equals_target,
            ),
        );
    }
}

// ── 第二段的阳性对照 ──────────────────────────────────────────────────────────────────────

pub(super) fn first_write_name(table: &[WriteSpecification], owner: Owner, kind: StepKind) -> String {
    table.iter().find(|specification| specification.owner == owner && specification.kind == kind).map(|specification| specification.name.clone()).expect("写表里有这个所有者与种类的写")
}

/// PC1、PC2、PC3 与 PC2 在 L3b 上的对应（删 K2 里一个「走得到的跨事务状态」，L3b 恰好多 1）。K1 一侧取「枚举域里有这个状态」的字典序最小的那条流。
#[allow(clippy::too_many_arguments, reason = "这些参数是这一次计算彼此独立的输入，合成只用一次的结构体只多一层包装")]
pub(super) fn run_positive_controls_two_disk(
    emitter: &mut Emitter,
    tally: &mut RunTally,
    label: &str,
    world: &World,
    truth: &StateList,
    exact: &StateList,
    arm_by_transaction: &StateList,
    streams_containing: &dyn Fn(StateCode) -> Option<StateList>,
) {
    let table = &world.table;
    if world.candidate == Candidate::SerialPipeline {
        return;
    }
    let older_unit_write = first_write_name(table, Owner::OlderTransaction, StepKind::UnitWrite);
    let newer_unit_write = first_write_name(table, Owner::NewerTransaction, StepKind::UnitWrite);
    if world.candidate == Candidate::ManyPublishesInFlightFree {
        let witness = state_with_only(table, &[newer_unit_write.as_str()]);
        let serial = world.serial_stream.as_ref().expect("两条链的候选有串行流");
        let serial_states = crash_states_of_stream(table, serial);
        let serial_product = serial_concatenation_of_transaction_chains(world);
        for (arm_name, arm_states) in [("arm_by_root_publish", &serial_states), ("arm_by_transaction", &serial_product)] {
            let witness_in_truth = contains(truth, witness);
            let witness_missed = !contains(arm_states, witness);
            let misses = difference_count(truth, arm_states);
            let cross_misses = count_cross_transaction_misses(table, truth, arm_states);
            let ok = witness_in_truth && witness_missed && misses >= 1;
            emit(
                emitter,
                format!(
                    "name=positive_control control=serial_concatenation cell={label} arm={arm_name} witness={} witness_in_truth={witness_in_truth} witness_missed={witness_missed} misses_against_truth={misses} cross_misses={cross_misses} ok={ok}",
                    render_state(table, witness),
                ),
            );
            tally.self_check(ok, &format!("V2:PC1:{label}:{arm_name}"));
        }
    }
    let two_units_state = state_with_only(table, &[older_unit_write.as_str(), newer_unit_write.as_str()]);
    if contains(truth, two_units_state) {
        let arm_states = streams_containing(two_units_state).expect("w2 在真值里就有一条流罩得住它");
        let before = count_cross_transaction_misses(table, truth, &arm_states);
        let after = count_cross_transaction_misses(table, truth, &without_state(&arm_states, two_units_state));
        let per_transaction_cut_before = count_cross_transaction_misses(table, truth, arm_by_transaction);
        let per_transaction_cut_after = count_cross_transaction_misses(table, truth, &without_state(arm_by_transaction, two_units_state));
        let ok = after == before + 1 && per_transaction_cut_after == per_transaction_cut_before + 1;
        emit(
            emitter,
            format!(
                "name=positive_control control=delete_one_state cell={label} deleted={} arm_by_root_publish_misses_before={before} arm_by_root_publish_misses_after={after} arm_by_transaction_misses_before={per_transaction_cut_before} arm_by_transaction_misses_after={per_transaction_cut_after} ok={ok}",
                render_state(table, two_units_state),
            ),
        );
        tally.self_check(ok, &format!("V2:PC2:{label}"));
    }
    let prefix_witness = recorded_prefix_witness(table);
    if contains(truth, prefix_witness) && is_invalid_prefix_by_record(table, prefix_witness) {
        let arm_states = streams_containing(prefix_witness).expect("见证状态在真值里就有一条流罩得住它");
        let before = count_invalid_prefixes_covered(table, truth, &arm_states, InvalidPrefixKind::ByRecord);
        let after = count_invalid_prefixes_covered(table, truth, &without_state(&arm_states, prefix_witness), InvalidPrefixKind::ByRecord);
        let per_transaction_cut_before = count_invalid_prefixes_covered(table, truth, arm_by_transaction, InvalidPrefixKind::ByRecord);
        let per_transaction_cut_after = count_invalid_prefixes_covered(table, truth, &without_state(arm_by_transaction, prefix_witness), InvalidPrefixKind::ByRecord);
        let ok = before == after + 1 && per_transaction_cut_before == per_transaction_cut_after + 1;
        emit(
            emitter,
            format!(
                "name=positive_control control=delete_one_invalid_prefix cell={label} deleted={} arm_by_root_publish_covered_before={before} arm_by_root_publish_covered_after={after} arm_by_transaction_covered_before={per_transaction_cut_before} arm_by_transaction_covered_after={per_transaction_cut_after} ok={ok}",
                render_state(table, prefix_witness),
            ),
        );
        tally.self_check(ok, &format!("V2:PC3:{label}"));
    } else {
        emit(
            emitter,
            format!("name=positive_control control=delete_one_invalid_prefix cell={label} status=not_applicable reason=witness_not_in_truth"),
        );
    }
    let removable = truth
        .iter()
        .copied()
        .find(|state| is_cross_transaction(table, *state) && contains(exact, *state) && contains(arm_by_transaction, *state));
    match removable {
        Some(removed) => {
            let before = true_cross_misses(table, truth, exact, arm_by_transaction);
            let after = true_cross_misses(table, truth, exact, &without_state(arm_by_transaction, removed));
            let ok = after == before + 1;
            emit(
                emitter,
                format!(
                    "name=positive_control control=delete_one_true_cross_state cell={label} deleted={} arm_by_transaction_true_misses_before={before} arm_by_transaction_true_misses_after={after} ok={ok}",
                    render_state(table, removed),
                ),
            );
            tally.self_check(ok, &format!("V2:PC2_true_miss:{label}"));
        }
        None => emit(
            emitter,
            format!("name=positive_control control=delete_one_true_cross_state cell={label} status=not_applicable reason=no_state_in_truth_and_exact_and_cross_and_arm"),
        ),
    }
}

// ── 真实发布形上 K1 最大一段（L8）─────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SegmentObjective {
    /// 一段展开出的状态数（2^两态写数 · 3^原地覆写数）。
    Expansion,
    /// 一段里的写数。
    Writes,
}

fn segment_score(objective: SegmentObjective, two_state_writes: u32, in_place_writes: u32) -> u128 {
    match objective {
        SegmentObjective::Expansion => (1u128 << two_state_writes) * 3u128.pow(in_place_writes),
        SegmentObjective::Writes => u128::from(two_state_writes + in_place_writes),
    }
}

/// 在两条链的全部交错上求「一次发出流里最大一段的得分」的最大值：状态 = 两条链各走到哪、未放行的盘、当前段里的两态写数与原地覆写数。
/// 段怎么关与 `operation_segments` 同一条规矩。
pub(super) struct LargestSegmentSearch {
    pub(super) older_steps: Vec<Step>,
    pub(super) newer_steps: Vec<Step>,
    pub(super) in_place: Vec<bool>,
    objective: SegmentObjective,
    memo: FastMap<u64, (u128, u8)>,
}

impl LargestSegmentSearch {
    pub(super) fn new(world: &World, objective: SegmentObjective) -> LargestSegmentSearch {
        LargestSegmentSearch {
            older_steps: steps_of(&world.table, &world.search_chains[0]),
            newer_steps: steps_of(&world.table, &world.search_chains[1]),
            in_place: world.table.iter().map(|specification| specification.is_in_place).collect(),
            objective,
            memo: FastMap::default(),
        }
    }

    /// 走一步之后的（未放行盘，两态写数，原地覆写数，这一步关掉的那一段的得分）。
    fn advance(&self, step: Step, unreleased: u8, two_state: u32, in_place: u32) -> (u8, u32, u32, Option<u128>) {
        let (mut unreleased, mut two_state, mut in_place) = (unreleased, two_state, in_place);
        let may_close;
        match step {
            Step::Write { index, disk, fua } => {
                if self.in_place[usize::from(index)] {
                    in_place += 1;
                } else {
                    two_state += 1;
                }
                if fua {
                    unreleased &= !(1u8 << disk);
                    may_close = true;
                } else {
                    unreleased |= 1u8 << disk;
                    may_close = false;
                }
            }
            Step::Barrier { disk } => {
                unreleased &= !(1u8 << disk);
                may_close = true;
            }
        }
        if may_close && two_state + in_place > 0 && unreleased == 0 {
            (unreleased, 0, 0, Some(segment_score(self.objective, two_state, in_place)))
        } else {
            (unreleased, two_state, in_place, None)
        }
    }

    fn best(&mut self, older_position: usize, newer_position: usize, unreleased: u8, two_state: u32, in_place: u32) -> u128 {
        let key = (((older_position as u64 * 64 + newer_position as u64) * 4 + u64::from(unreleased)) * 64 + u64::from(two_state)) * 8 + u64::from(in_place);
        if let Some(&(value, _)) = self.memo.get(&key) {
            return value;
        }
        let (older_length, newer_length) = (self.older_steps.len(), self.newer_steps.len());
        let (value, choice) = if older_position == older_length && newer_position == newer_length {
            (if two_state + in_place > 0 { segment_score(self.objective, two_state, in_place) } else { 0 }, 2u8)
        } else {
            let mut best_value = 0u128;
            let mut best_choice = 2u8;
            for (choice, may_take) in [(0u8, older_position < older_length), (1u8, newer_position < newer_length)] {
                if !may_take {
                    continue;
                }
                let step = if choice == 0 { self.older_steps[older_position] } else { self.newer_steps[newer_position] };
                let (next_older_position, next_newer_position) = if choice == 0 { (older_position + 1, newer_position) } else { (older_position, newer_position + 1) };
                let (unreleased_after, two_state_after, in_place_after, closed_score) = self.advance(step, unreleased, two_state, in_place);
                let continued = self.best(next_older_position, next_newer_position, unreleased_after, two_state_after, in_place_after);
                let value = closed_score.map_or(continued, |score| score.max(continued));
                if best_choice == 2 || value > best_value {
                    best_value = value;
                    best_choice = choice;
                }
            }
            (best_value, best_choice)
        };
        self.memo.insert(key, (value, choice));
        value
    }

    /// 取到最大值的那条发出流（每个状态取字典序最小的选择：有并列时 A 先）。
    fn witness_stream(&mut self, world: &World) -> Vec<Event> {
        let (mut older_position, mut newer_position, mut unreleased, mut two_state, mut in_place) = (0usize, 0usize, 0u8, 0u32, 0u32);
        let mut events = Vec::new();
        loop {
            self.best(older_position, newer_position, unreleased, two_state, in_place);
            let key = (((older_position as u64 * 64 + newer_position as u64) * 4 + u64::from(unreleased)) * 64 + u64::from(two_state)) * 8 + u64::from(in_place);
            let (_, choice) = self.memo[&key];
            let step = match choice {
                0 => {
                    events.push(world.search_chains[0][older_position]);
                    let step = self.older_steps[older_position];
                    older_position += 1;
                    step
                }
                1 => {
                    events.push(world.search_chains[1][newer_position]);
                    let step = self.newer_steps[newer_position];
                    newer_position += 1;
                    step
                }
                _ => return events,
            };
            let (unreleased_after, two_state_after, in_place_after, _) = self.advance(step, unreleased, two_state, in_place);
            unreleased = unreleased_after;
            two_state = two_state_after;
            in_place = in_place_after;
        }
    }
}

pub(super) struct LargestSegment {
    pub(super) value: u128,
    stream: Vec<Event>,
    pub(super) writes_in_best_segment: usize,
    in_place_in_best_segment: usize,
    pub(super) states_in_best_segment: u128,
    pub(super) replay_agrees: bool,
}

pub(super) fn largest_segment_of(world: &World, objective: SegmentObjective) -> LargestSegment {
    let mut search = LargestSegmentSearch::new(world, objective);
    let value = search.best(0, 0, 0, 0, 0);
    let stream = search.witness_stream(world);
    let segments = write_segments(&world.table, &stream);
    let score_of = |segment: &Vec<usize>| {
        let in_place = u32_from_usize(segment.iter().filter(|&&write| world.table[write].is_in_place).count());
        segment_score(objective, u32_from_usize(segment.len()) - in_place, in_place)
    };
    let best_segment = segments.iter().max_by_key(|segment| score_of(segment)).expect("流里至少有一段");
    let replay_agrees = score_of(best_segment) == value;
    LargestSegment {
        value,
        writes_in_best_segment: best_segment.len(),
        in_place_in_best_segment: best_segment.iter().filter(|&&write| world.table[write].is_in_place).count(),
        states_in_best_segment: segment_expansion(&world.table, best_segment) - 1,
        replay_agrees,
        stream,
    }
}

pub(super) fn emit_real_shape_capacity(emitter: &mut Emitter, tally: &mut RunTally) -> bool {
    let geometry = geometry_real_publish_shape();
    let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry);
    let mut writes_floor_ok = true;
    for (objective, name) in [(SegmentObjective::Expansion, "largest_segment_states"), (SegmentObjective::Writes, "most_writes_in_one_segment")] {
        let largest = largest_segment_of(&world, objective);
        tally.self_check(largest.replay_agrees, &format!("L8:replay_agrees:{name}"));
        if objective == SegmentObjective::Writes {
            writes_floor_ok = largest.writes_in_best_segment >= 2 * geometry.units_per_transaction * geometry.disk_count;
        }
        emit(
            emitter,
            format!(
                "name=capacity_real_shape shape={} objective={name} interleavings=seventy_choose_thirty_five table_writes={} segment_writes={} segment_in_place_writes={} segment_states={} exceeds_u32={} within_u64={} within_enumerated_writes_limit={} dynamic_programming_agrees_with_replay={} stream={}",
                geometry.label,
                world.table.len(),
                largest.writes_in_best_segment,
                largest.in_place_in_best_segment,
                largest.states_in_best_segment,
                largest.states_in_best_segment > MAXIMUM_SEGMENT_STATES_IN_U32,
                largest.states_in_best_segment <= MAXIMUM_SEGMENT_STATES_IN_U64,
                world.table.len() <= MAXIMUM_ENUMERATED_WRITES,
                largest.replay_agrees,
                render_stream(&world.table, &largest.stream),
            ),
        );
    }
    writes_floor_ok
}

// ── 第二段的主流程 ────────────────────────────────────────────────────────────────────────

pub(super) fn second_segment_geometries() -> [(Geometry, [&'static str; 4], &'static str, &'static str); 2] {
    [
        (geometry_g4a(), ["g4a_serial", "g4a_one_publish", "g4a_free", "g4a_ordered"], "G4a", "G4a_ordered"),
        (geometry_g4b(), ["g4b_serial", "g4b_one_publish", "g4b_free", "g4b_ordered"], "G4b", "G4b_ordered"),
    ]
}
