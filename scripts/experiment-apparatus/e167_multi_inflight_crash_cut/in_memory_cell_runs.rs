//! 流存在内存里逐条算的那一路：格的评估、判定、停机检查、锚点表、容量锚点、逐格输出与各组运行。

use e7_index_bench::isize_from_usize;
use super::*;

#[derive(Default)]
pub(super) struct RunTally {
    pub(super) void_conditions: Vec<String>,
    pub(super) halt_conditions: Vec<String>,
    pub(super) self_checks_passed: usize,
}

impl RunTally {
    /// 把另一段的结果并进来：不通过的条件接在后面，通过的自检数相加。
    pub(super) fn absorb(&mut self, other: RunTally) {
        self.void_conditions.extend(other.void_conditions);
        self.halt_conditions.extend(other.halt_conditions);
        self.self_checks_passed += other.self_checks_passed;
    }

    pub(super) fn self_check(&mut self, passed: bool, void_label: &str) {
        if passed {
            self.self_checks_passed += 1;
        } else {
            self.void_conditions.push(void_label.to_string());
        }
    }
}

// ── 停机条款 S1、S2（登记第十一节）──────────────────────────────────────────────────────────

/// `.claude/kb/layout/01-first-txn.md` 第 403 行「普通发布（新池新建文件）」那一行的种类串（整行抄）。
pub(super) const REGISTERED_ORDINARY_PUBLISH_KINDS: &str =
    "[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]";
/// 同一文件第 410 行「新池新建文件那条流」的种类串（整条流，57 次操作）。
pub(super) const REGISTERED_WHOLE_STREAM_KINDS: &str = "[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]";

pub(super) struct RegisteredReading {
    item: &'static str,
    pub(super) segments: String,
    pub(super) kinds: String,
    pub(super) operations: usize,
    pub(super) closed_form_states: u128,
    pub(super) enumerated_states: usize,
}

pub(super) fn registered_ordinary_publish_reading() -> RegisteredReading {
    let geometry = geometry_registered_publish_two_state();
    let mut table = Vec::new();
    let writes = push_transaction_writes(&mut table, &geometry, Owner::OlderTransaction, 0);
    let chain = single_transaction_chain(&writes, geometry.disk_count);
    let segments = write_segments(&table, &chain);
    RegisteredReading {
        item: "ordinary_publish_chain_u12_two_disks",
        segments: segment_sizes_text(&table, &chain),
        kinds: segment_kinds_text(&table, &chain),
        operations: chain.len(),
        closed_form_states: closed_form_state_count(&table, &segments),
        enumerated_states: crash_states_of_segments(&table, &segments).len(),
    }
}

pub(super) fn registered_whole_stream_reading(system_configuration_has_third_state: bool, enumerate: bool) -> RegisteredReading {
    let (table, events) = stream_from_registered_kinds(REGISTERED_WHOLE_STREAM_KINDS, system_configuration_has_third_state);
    let segments = write_segments(&table, &events);
    RegisteredReading {
        item: if system_configuration_has_third_state { "whole_stream_three_state" } else { "whole_stream_two_state" },
        segments: segment_sizes_text(&table, &events),
        kinds: segment_kinds_text(&table, &events),
        operations: events.len(),
        closed_form_states: closed_form_state_count(&table, &segments),
        enumerated_states: if enumerate { crash_states_of_segments(&table, &segments).len() } else { 0 },
    }
}

const REGISTERED_ORDINARY_PUBLISH_SEGMENTS: &str = "24+2+1+2";
pub(super) const REGISTERED_WHOLE_STREAM_SEGMENTS: &str = "2+2+1+2+2+1+2+24+2+1+2";

pub(super) fn run_halt_checks(emitter: &mut Emitter, tally: &mut RunTally) {
    let ordinary = registered_ordinary_publish_reading();
    let ordinary_ok = ordinary.segments == REGISTERED_ORDINARY_PUBLISH_SEGMENTS
        && ordinary.kinds == REGISTERED_ORDINARY_PUBLISH_KINDS
        && ordinary.operations == 35
        && ordinary.closed_form_states == 16_777_223
        && ordinary.enumerated_states == 16_777_223;
    emit(
        emitter,
        format!(
            "name=halt_check condition=S1 item={} segments={} registered_segments={} kinds_match={} operations={} registered_operations=35 closed_form_states={} enumerated_states={} registered_states=16777223 ok={ordinary_ok}",
            ordinary.item,
            ordinary.segments,
            REGISTERED_ORDINARY_PUBLISH_SEGMENTS,
            ordinary.kinds == REGISTERED_ORDINARY_PUBLISH_KINDS,
            ordinary.operations,
            ordinary.closed_form_states,
            ordinary.enumerated_states,
        ),
    );
    if !ordinary_ok {
        tally.halt_conditions.push(format!("S1:{}:kinds={}", ordinary.item, ordinary.kinds));
    }
    let stream_two_state = registered_whole_stream_reading(false, true);
    let stream_ok = stream_two_state.segments == REGISTERED_WHOLE_STREAM_SEGMENTS
        && stream_two_state.kinds == REGISTERED_WHOLE_STREAM_KINDS
        && stream_two_state.operations == 57
        && stream_two_state.closed_form_states == 16_777_240
        && stream_two_state.enumerated_states == 16_777_240;
    emit(
        emitter,
        format!(
            "name=halt_check condition=S1 item={} segments={} registered_segments={} kinds_match={} operations={} registered_operations=57 closed_form_states={} enumerated_states={} registered_states=16777240 ok={stream_ok}",
            stream_two_state.item,
            stream_two_state.segments,
            REGISTERED_WHOLE_STREAM_SEGMENTS,
            stream_two_state.kinds == REGISTERED_WHOLE_STREAM_KINDS,
            stream_two_state.operations,
            stream_two_state.closed_form_states,
            stream_two_state.enumerated_states,
        ),
    );
    if !stream_ok {
        tally.halt_conditions.push(format!("S1:{}:kinds={}", stream_two_state.item, stream_two_state.kinds));
    }
    let stream_three_state = registered_whole_stream_reading(true, false);
    let three_state_ok = stream_three_state.closed_form_states == 16_777_260;
    emit(
        emitter,
        format!(
            "name=halt_check condition=S1 item={} closed_form_states={} registered_states=16777260 ok={three_state_ok}",
            stream_three_state.item, stream_three_state.closed_form_states
        ),
    );
    if !three_state_ok {
        tally.halt_conditions.push("S1:whole_stream_three_state".to_string());
    }
    let attribution_ok = attribution_matches_hand_derivation();
    emit(
        emitter,
        format!(
            "name=halt_check condition=S2 item=publish_attribution_on_serial_stream expected_publishes=2 hand_derived_units_per_publish=2 hand_derived_records_per_publish=1 ok={attribution_ok}"
        ),
    );
    if !attribution_ok {
        tally.halt_conditions.push("S2:publish_attribution".to_string());
    }
}

/// S2：串行流 a1 a2 | rA | RA SA | b1 b2 | rB | RB SB 上，按 `memory_pool.rs:993` `publishes_of_one_recording` 的规则手推：
/// 第一次发布 = 单元写 a1、a2 + 记录写 rA + 根 RA；第二次 = b1、b2、rB、RB；两个系统配置槽写（SA、SB）与屏障不归任何一次发布。
pub(super) fn attribution_matches_hand_derivation() -> bool {
    let world = build_world(Candidate::SerialPipeline, &geometry_one_disk_two_units());
    let index_of = |name: &str| world.table.iter().position(|specification| specification.name == name).expect("写表里有这个名字");
    let hand_derived = vec![
        PublishAttribution { units: vec![index_of("a1"), index_of("a2")], records: vec![index_of("rA")], root: index_of("RA") },
        PublishAttribution { units: vec![index_of("b1"), index_of("b2")], records: vec![index_of("rB")], root: index_of("RB") },
    ];
    let serial = world.serial_stream.as_ref().expect("串行候选有串行流");
    publish_attribution(&world.table, serial) == hand_derived
}

// ── 丙类锚点（登记第七节）──────────────────────────────────────────────────────────────────

pub(super) struct AnchorRow {
    pub(super) geometry: Geometry,
    expected_single_chain: u128,
    expected_transaction_product: u128,
    expected_serial_truth: u128,
    expected_one_publish_truth: u128,
    expected_one_publish_arm_by_transaction: u128,
}

pub(super) fn anchor_rows() -> Vec<AnchorRow> {
    let row = |geometry: Geometry, values: [u128; 5]| AnchorRow {
        geometry,
        expected_single_chain: values[0],
        expected_transaction_product: values[1],
        expected_serial_truth: values[2],
        expected_one_publish_truth: values[3],
        expected_one_publish_arm_by_transaction: values[4],
    };
    vec![
        row(geometry_one_disk_two_units(), [8, 64, 15, 22, 28]),
        row(geometry_one_disk_one_unit(), [6, 36, 11, 10, 12]),
        row(geometry_g4a(), [28, 784, 55, 280, 370]),
        row(geometry_g4b(), [28, 784, 55, 280, 370]),
        row(geometry_one_disk_two_units_without_torn_state(), [7, 49, 13, 21, 27]),
    ]
}

pub(super) struct AnchorReading {
    pub(super) quantity: &'static str,
    pub(super) expected: u128,
    pub(super) actual: u128,
}

pub(super) fn read_anchor_row(row: &AnchorRow) -> Vec<AnchorReading> {
    let serial = build_world(Candidate::SerialPipeline, &row.geometry);
    let one_publish = build_world(Candidate::OnePublishManyTransactions, &row.geometry);
    let single_chain = crash_states_of_stream(&serial.table, &serial.arm_chains[0]).len() as u128;
    let product = arm_by_transaction_states(&serial).len() as u128;
    let serial_truth = truth_by_enumeration(&serial).len() as u128;
    let one_publish_truth = truth_by_enumeration(&one_publish).len() as u128;
    let one_publish_arm = arm_by_transaction_states(&one_publish).len() as u128;
    vec![
        AnchorReading { quantity: "single_transaction_chain", expected: row.expected_single_chain, actual: single_chain },
        AnchorReading { quantity: "arm_by_transaction_product", expected: row.expected_transaction_product, actual: product },
        AnchorReading { quantity: "serial_pipeline_truth", expected: row.expected_serial_truth, actual: serial_truth },
        AnchorReading { quantity: "one_publish_truth", expected: row.expected_one_publish_truth, actual: one_publish_truth },
        AnchorReading { quantity: "one_publish_arm_by_transaction", expected: row.expected_one_publish_arm_by_transaction, actual: one_publish_arm },
    ]
}

fn run_anchor_table(emitter: &mut Emitter, tally: &mut RunTally) {
    for row in anchor_rows() {
        for reading in read_anchor_row(&row) {
            let ok = reading.expected == reading.actual;
            emit(
                emitter,
                format!("name=anchor table=closed_form geometry={} quantity={} expected={} actual={} ok={ok}", row.geometry.label, reading.quantity, reading.expected, reading.actual),
            );
            tally.self_check(ok, &format!("V5:anchor:{}:{}", row.geometry.label, reading.quantity));
        }
    }
    run_capacity_anchors(emitter, tally);
}

/// 登记第七节末尾「能力那几格的锚点」：罩住段串与真实发布形上的算术。
fn run_capacity_anchors(emitter: &mut Emitter, tally: &mut RunTally) {
    for (geometry, expected_cover_states, expected_cover_writes) in [(geometry_one_disk_two_units(), 2304u128, 10usize), (geometry_g4a(), 1_327_104u128, 18usize)] {
        let world = build_world(Candidate::SerialPipeline, &geometry);
        let product = arm_by_transaction_states(&world);
        let cover = cover_by_merging(&world.table, &product);
        let one_segment_of_everything = cover.segments.len() == 1 && cover.segments[0].len() == expected_cover_writes;
        let cover_largest_expansion = cover.segments.iter().map(|segment| segment_expansion(&world.table, segment)).max().unwrap_or(0);
        let ok = one_segment_of_everything && cover.covers_target && cover_largest_expansion == expected_cover_states;
        emit(
            emitter,
            format!(
                "name=anchor table=capacity geometry={} quantity=arm_by_transaction_cover_one_segment_states expected={expected_cover_states} actual={cover_largest_expansion} cover_segments={} cover_largest_segment_writes={} covers_target={} within_u32={} ok={ok}",
                geometry.label,
                cover.segments.len(),
                cover.segments.iter().map(Vec::len).max().unwrap_or(0),
                cover.covers_target,
                cover_largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U32,
            ),
        );
        tally.self_check(ok, &format!("F6:L7a_anchor:{}", geometry.label));
    }
    let real = geometry_real_publish_shape();
    let world = build_world(Candidate::SerialPipeline, &real);
    let every_write: Vec<usize> = (0..world.table.len()).collect();
    let cover_all_writes_states = segment_expansion(&world.table, &every_write);
    let chain_segments = write_segments(&world.table, &world.arm_chains[0]);
    let single_chain_states = closed_form_state_count(&world.table, &chain_segments);
    let product_states = single_chain_states * single_chain_states;
    let witness_stream: Vec<Event> = {
        let older_units: Vec<Event> = world.arm_chains[0].iter().copied().take_while(|event| matches!(event, Event::Write(index) if world.table[*index].kind == StepKind::UnitWrite)).collect();
        let newer_units: Vec<Event> = world.arm_chains[1].iter().copied().take_while(|event| matches!(event, Event::Write(index) if world.table[*index].kind == StepKind::UnitWrite)).collect();
        let mut stream = older_units.clone();
        stream.extend(newer_units.iter().copied());
        stream.extend(world.arm_chains[0][older_units.len()..].iter().copied());
        stream.extend(world.arm_chains[1][newer_units.len()..].iter().copied());
        stream
    };
    let witness_segments = write_segments(&world.table, &witness_stream);
    let witness_first_segment_writes = witness_segments[0].len();
    let witness_first_segment_states = segment_expansion(&world.table, &witness_segments[0]) - 1;
    let readings: [(&str, u128, u128); 4] = [
        ("real_shape_arm_by_transaction_cover_one_segment_states", 1_459_166_279_268_040_704, cover_all_writes_states),
        ("real_shape_single_chain_states_three_state", 16_777_228, single_chain_states),
        ("real_shape_arm_by_transaction_product_states", 281_475_379_363_984, product_states),
        ("real_shape_k1_witness_first_segment_nonzero_states", 281_474_976_710_655, witness_first_segment_states),
    ];
    for (quantity, expected, actual) in readings {
        let ok = expected == actual;
        emit(
            emitter,
            format!(
                "name=anchor table=capacity geometry={} quantity={quantity} expected={expected} actual={actual} exceeds_u32={} within_u64={} ok={ok}",
                real.label,
                actual > MAXIMUM_SEGMENT_STATES_IN_U32,
                actual <= MAXIMUM_SEGMENT_STATES_IN_U64,
            ),
        );
        tally.self_check(ok, &format!("V5:capacity_anchor:{quantity}"));
    }
    let witness_ok = witness_first_segment_writes == 2 * real.units_per_transaction * real.disk_count;
    emit(
        emitter,
        format!(
            "name=anchor table=capacity geometry={} quantity=real_shape_k1_witness_first_segment_writes expected={} actual={witness_first_segment_writes} within_enumerated_writes_limit={} ok={witness_ok}",
            real.label,
            2 * real.units_per_transaction * real.disk_count,
            world.table.len() <= MAXIMUM_ENUMERATED_WRITES,
        ),
    );
    tally.self_check(witness_ok, "F6:witness_first_segment_writes");
}

// ── 单元格的输出与判决 ────────────────────────────────────────────────────────────────────

pub(super) struct CellJudgement {
    pub(super) label: &'static str,
    pub(super) sampling_point: &'static str,
    pub(super) candidate: Candidate,
    pub(super) disks: usize,
    pub(super) arm_by_root_publish_leak_judgement: &'static str,
    pub(super) arm_by_root_publish_minimum_cross_misses: usize,
    pub(super) arm_by_root_publish_maximum_cross_misses: usize,
    pub(super) arm_by_transaction_cross_misses: usize,
    pub(super) recorded_truth_count: usize,
    pub(super) all_writes_truth_count: usize,
}

pub(super) fn emit_cell(emitter: &mut Emitter, tally: &mut RunTally, evaluation: &CellEvaluation) -> CellJudgement {
    let world = &evaluation.world;
    let table = &world.table;
    let label = evaluation.label;
    let truth = &evaluation.truth;
    let methods_agree = *truth == evaluation.truth_by_search;
    tally.self_check(methods_agree, &format!("V3:truth_methods:{label}"));
    emit(
        emitter,
        format!(
            "name=cell_truth cell={label} sampling_point={} candidate={} disks={} units={} table_writes={} streams={} truth_by_enumeration={} truth_by_search={} methods_agree={methods_agree} cross_transaction_states={}",
            evaluation.sampling_point,
            world.candidate.name(),
            world.geometry.disk_count,
            world.geometry.units_per_transaction,
            table.len(),
            world.streams.len(),
            truth.len(),
            evaluation.truth_by_search.len(),
            truth.iter().filter(|state| is_cross_transaction(table, **state)).count(),
        ),
    );
    let cross_misses: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.cross_misses).collect();
    let non_cross_misses: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.non_cross_misses).collect();
    let state_counts: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.state_count).collect();
    let (minimum_cross_misses, minimum_position) = minimum_with_position(&cross_misses);
    let maximum_cross_misses = maximum_value(&cross_misses);
    let (minimum_states, _) = minimum_with_position(&state_counts);
    let maximum_states = maximum_value(&state_counts);
    let closed_form_mismatches = evaluation.streams.iter().filter(|metrics| metrics.state_count as u128 != metrics.closed_form_state_count).count();
    tally.self_check(closed_form_mismatches == 0, &format!("V6:closed_form:{label}"));
    let serial_states = evaluation.serial_stream_position.map_or_else(|| "not_defined".to_string(), |position| evaluation.streams[position].state_count.to_string());
    emit(
        emitter,
        format!(
            "name=arm_by_root_publish cell={label} streams={} states_minimum={minimum_states} states_maximum={maximum_states} states_on_serial_stream={serial_states} states_on_minimum_cross_miss_stream={} closed_form_mismatches={closed_form_mismatches} cross_misses_minimum={minimum_cross_misses} cross_misses_maximum={maximum_cross_misses} streams_with_cross_misses={} minimum_cross_miss_stream={} non_cross_misses_maximum={} streams_with_non_cross_misses={}",
            evaluation.streams.len(),
            state_counts[minimum_position],
            cross_misses.iter().filter(|misses| **misses > 0).count(),
            render_stream(table, &world.streams[minimum_position]),
            maximum_value(&non_cross_misses),
            non_cross_misses.iter().filter(|misses| **misses > 0).count(),
        ),
    );
    let arm = &evaluation.arm_by_transaction;
    let cross_misses_of_arm = count_cross_transaction_misses(table, truth, arm);
    let true_cross_misses_of_arm: usize = {
        let reachable_truth: Vec<StateCode> = truth.iter().copied().filter(|state| contains(&evaluation.exact_reachable, *state)).collect();
        count_cross_transaction_misses(table, &reachable_truth, arm)
    };
    let extra_states = difference_count(arm, truth);
    let identity_ok = arm.len() + cross_misses_of_arm + count_non_cross_misses(table, truth, arm) == truth.len() + extra_states;
    tally.self_check(identity_ok, &format!("V5:identity:{label}"));
    emit(
        emitter,
        format!(
            "name=arm_by_transaction cell={label} states={} cross_misses={cross_misses_of_arm} true_cross_misses={true_cross_misses_of_arm} non_cross_misses={} states_not_in_truth={extra_states} identity_ok={identity_ok}",
            arm.len(),
            count_non_cross_misses(table, truth, arm),
        ),
    );
    let one_disk = world.geometry.disk_count == 1;
    if one_disk {
        tally.self_check(cross_misses_of_arm == 0, &format!("V4:one_disk_arm_by_transaction_misses:{label}"));
    }
    let mut recorded_truth_count = 0;
    let mut all_writes_truth_count = 0;
    for kind in BOTH_INVALID_PREFIX_KINDS {
        let truth_count = count_invalid_prefixes_in(table, truth, kind);
        let covered: Vec<usize> = evaluation
            .streams
            .iter()
            .map(|metrics| match kind {
                InvalidPrefixKind::ByRecord => metrics.recorded_prefixes_covered,
                InvalidPrefixKind::ByAllWrites => metrics.all_writes_prefixes_covered,
            })
            .collect();
        let arm_covered = count_invalid_prefixes_covered(table, truth, arm, kind);
        let arm_unreachable = count_invalid_prefixes_in(table, arm, kind) - arm_covered;
        match kind {
            InvalidPrefixKind::ByRecord => recorded_truth_count = truth_count,
            InvalidPrefixKind::ByAllWrites => all_writes_truth_count = truth_count,
        }
        emit(
            emitter,
            format!(
                "name=invalid_prefix cell={label} kind={} truth_count={truth_count} arm_by_root_publish_covered_minimum={} arm_by_root_publish_covered_maximum={} arm_by_transaction_covered={arm_covered} arm_by_transaction_not_in_truth={arm_unreachable} arm_by_root_publish_judgement={} arm_by_transaction_judgement={}",
                kind.name(),
                covered.iter().copied().min().expect("至少一条流"),
                maximum_value(&covered),
                invalid_prefix_judgement(covered.iter().copied().min().expect("至少一条流"), truth_count),
                invalid_prefix_judgement(arm_covered, truth_count),
            ),
        );
    }
    emit_cover_lines(emitter, tally, evaluation);
    emit_capacity_line(emitter, evaluation);
    let exact = &evaluation.exact_reachable;
    emit(
        emitter,
        format!(
            "name=reference cell={label} exact_states={} exact_not_in_truth={} truth_not_in_exact={}",
            exact.len(),
            difference_count(exact, truth),
            difference_count(truth, exact),
        ),
    );
    CellJudgement {
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
    }
}

fn emit_cover_lines(emitter: &mut Emitter, tally: &mut RunTally, evaluation: &CellEvaluation) {
    let table = &evaluation.world.table;
    for (set_name, target) in [("truth", &evaluation.truth), ("arm_by_transaction", &evaluation.arm_by_transaction)] {
        let cover = cover_by_merging(table, target);
        let exhaustive_agrees = if table.len() <= 8 {
            let exhaustive = cover_by_exhaustive_ordered_partitions(table, target);
            let agrees = exhaustive.minimal_state_count == cover.cover_state_count && exhaustive.optimal_covers.contains(&cover.segments);
            tally.self_check(agrees, &format!("L7a:merging_vs_exhaustive:{}:{set_name}", evaluation.label));
            yes_no(agrees).to_string()
        } else {
            "not_run_more_than_8_writes".to_string()
        };
        emit(
            emitter,
            format!(
                "name=cover cell={} set={set_name} target_states={} cover_segments={} cover_largest_segment_writes={} cover_states={} covers_target={} cover_equals_target={} exhaustive_agrees={exhaustive_agrees}",
                evaluation.label,
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

fn emit_capacity_line(emitter: &mut Emitter, evaluation: &CellEvaluation) {
    let table = &evaluation.world.table;
    let largest_writes = evaluation.streams.iter().map(|metrics| metrics.largest_segment_writes).max().unwrap_or(0);
    let largest_expansion = evaluation.streams.iter().map(|metrics| metrics.largest_segment_expansion).max().unwrap_or(0);
    let total_states = evaluation.streams.iter().map(|metrics| metrics.state_count).max().unwrap_or(0);
    let cover = cover_by_merging(table, &evaluation.arm_by_transaction);
    let cover_largest_expansion = cover.segments.iter().map(|segment| segment_expansion(table, segment)).max().unwrap_or(0);
    emit(
        emitter,
        format!(
            "name=capacity cell={} table_writes={} arm_by_root_publish_largest_segment_writes={largest_writes} arm_by_root_publish_largest_segment_states={largest_expansion} arm_by_root_publish_total_states_maximum={total_states} arm_by_transaction_cover_largest_segment_states={cover_largest_expansion} arm_by_transaction_cover_total_states={} writes_within_limit={} segments_within_u32={} segments_within_u64={}",
            evaluation.label,
            table.len(),
            cover.cover_state_count,
            table.len() <= MAXIMUM_ENUMERATED_WRITES,
            largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U32 && cover_largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U32,
            largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U64 && cover_largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U64,
        ),
    );
}

// ── 阳性对照 ───────────────────────────────────────────────────────────────────────────

pub(super) fn index_of_write(table: &[WriteSpecification], name: &str) -> usize {
    table.iter().position(|specification| specification.name == name).unwrap_or_else(|| panic!("写表里没有 {name}"))
}

pub(super) fn state_with_only(table: &[WriteSpecification], names: &[&str]) -> StateCode {
    names.iter().fold(0, |code, name| with_value(code, index_of_write(table, name), PERSISTED))
}

pub(super) fn without_state(states: &[StateCode], removed: StateCode) -> StateList {
    states.iter().copied().filter(|state| *state != removed).collect()
}

/// PC3 要的见证状态：两个事务的单元写都持久、rB 持久、rA 不持久（登记第五节 PC3 的例子）。
pub(super) fn recorded_prefix_witness(table: &[WriteSpecification]) -> StateCode {
    let mut code = 0;
    for (write_index, specification) in table.iter().enumerate() {
        let is_unit = specification.kind == StepKind::UnitWrite;
        let is_record_of_the_newer_transaction = specification.kind == StepKind::JournalRecord && specification.owner == Owner::NewerTransaction;
        if is_unit || is_record_of_the_newer_transaction {
            code = with_value(code, write_index, PERSISTED);
        }
    }
    code
}

fn run_positive_controls(emitter: &mut Emitter, tally: &mut RunTally, evaluations: &[CellEvaluation]) {
    for evaluation in evaluations {
        let world = &evaluation.world;
        let table = &world.table;
        let truth = &evaluation.truth;
        let label = evaluation.label;
        if world.candidate == Candidate::ManyPublishesInFlightFree {
            let witness = state_with_only(table, &["b1"]);
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
        if world.candidate == Candidate::SerialPipeline {
            continue;
        }
        let two_units_state = state_with_only(table, &["a1", "b1"]);
        if contains(truth, two_units_state) {
            let stream_of_witness = world.streams.iter().find(|stream| contains(&crash_states_of_stream(table, stream), two_units_state)).expect("w2 在真值里就有一条流罩得住它");
            let arm_states = crash_states_of_stream(table, stream_of_witness);
            let before = count_cross_transaction_misses(table, truth, &arm_states);
            let after = count_cross_transaction_misses(table, truth, &without_state(&arm_states, two_units_state));
            let per_transaction_cut_before = count_cross_transaction_misses(table, truth, &evaluation.arm_by_transaction);
            let per_transaction_cut_after = count_cross_transaction_misses(table, truth, &without_state(&evaluation.arm_by_transaction, two_units_state));
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
            let stream_of_witness = world.streams.iter().find(|stream| contains(&crash_states_of_stream(table, stream), prefix_witness)).expect("见证状态在真值里就有一条流罩得住它");
            let arm_states = crash_states_of_stream(table, stream_of_witness);
            let before = count_invalid_prefixes_covered(table, truth, &arm_states, InvalidPrefixKind::ByRecord);
            let after = count_invalid_prefixes_covered(table, truth, &without_state(&arm_states, prefix_witness), InvalidPrefixKind::ByRecord);
            let per_transaction_cut_before = count_invalid_prefixes_covered(table, truth, &evaluation.arm_by_transaction, InvalidPrefixKind::ByRecord);
            let per_transaction_cut_after = count_invalid_prefixes_covered(table, truth, &without_state(&evaluation.arm_by_transaction, prefix_witness), InvalidPrefixKind::ByRecord);
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
    }
}

// ── 判别力自证与几何敏感性 ───────────────────────────────────────────────────────────────

fn run_threshold_shifts(emitter: &mut Emitter, tally: &mut RunTally, evaluations: &[CellEvaluation], judgements: &[CellJudgement]) {
    let free = judgements.iter().find(|judgement| judgement.label == "g1_free").expect("G1 的 Σ_free 格");
    let before = leak_judgement(free.arm_by_root_publish_minimum_cross_misses, free.arm_by_root_publish_maximum_cross_misses, 0);
    let after = leak_judgement(free.arm_by_root_publish_minimum_cross_misses, free.arm_by_root_publish_maximum_cross_misses, free.arm_by_root_publish_maximum_cross_misses);
    let flips = before != after && after == "never_leaks";
    emit(
        emitter,
        format!(
            "name=threshold_shift quantity=l2_cross_misses cell=g1_free threshold_before=0 judgement_before={before} threshold_after={} judgement_after={after} flips_as_required={flips}",
            free.arm_by_root_publish_maximum_cross_misses,
        ),
    );
    tally.self_check(flips, "threshold_shift:l2");
    let evaluation = evaluations.iter().find(|candidate| candidate.label == "g1_free").expect("G1 的 Σ_free 格");
    let required = count_invalid_prefixes_in(&evaluation.world.table, &evaluation.truth, InvalidPrefixKind::ByRecord);
    let best_stream_covered = evaluation.streams.iter().map(|metrics| metrics.recorded_prefixes_covered).max().expect("至少一条流");
    let invalid_prefix_before = invalid_prefix_judgement(best_stream_covered, required);
    let invalid_prefix_after = invalid_prefix_judgement(best_stream_covered, best_stream_covered);
    let invalid_prefix_flips = invalid_prefix_before == "misses_invalid_prefixes" && invalid_prefix_after == "covers_every_invalid_prefix";
    emit(
        emitter,
        format!(
            "name=threshold_shift quantity=l6_recorded_prefixes_best_stream cell=g1_free covered={best_stream_covered} required_before={required} judgement_before={invalid_prefix_before} required_after={best_stream_covered} judgement_after={invalid_prefix_after} flips_as_required={invalid_prefix_flips}"
        ),
    );
    tally.self_check(invalid_prefix_flips, "threshold_shift:l6");
    emit(
        emitter,
        "name=threshold_shift quantity=l3b_true_cross_misses status=not_applicable reason=one_disk_cells_have_zero_by_derivation_and_l3b_is_judged_on_two_disk_geometries_in_the_second_segment".to_string(),
    );
}

fn run_geometry_sensitivity(emitter: &mut Emitter, judgements: &[CellJudgement]) {
    for (free_label, ordered_label, geometry_name) in [
        ("g1_free", "g1_ordered", "G1_vs_G2"),
        ("g3_free", "g3_ordered", "G3_vs_G3_ordered"),
        ("g5_free", "g5_ordered", "G5_vs_G5_ordered"),
    ] {
        let free = judgements.iter().find(|judgement| judgement.label == free_label).expect("自由交错格");
        let ordered = judgements.iter().find(|judgement| judgement.label == ordered_label).expect("带约束格");
        let same = free.arm_by_root_publish_leak_judgement == ordered.arm_by_root_publish_leak_judgement;
        emit(
            emitter,
            format!(
                "name=geometry_sensitivity pair={geometry_name} l2_judgement_free={} l2_judgement_ordered={} same_judgement={} cross_misses_minimum_free={} cross_misses_minimum_ordered={} cross_misses_maximum_free={} cross_misses_maximum_ordered={} invalid_prefix_recorded_truth_free={} invalid_prefix_recorded_truth_ordered={} invalid_prefix_all_writes_truth_free={} invalid_prefix_all_writes_truth_ordered={}",
                free.arm_by_root_publish_leak_judgement,
                ordered.arm_by_root_publish_leak_judgement,
                yes_no(same),
                free.arm_by_root_publish_minimum_cross_misses,
                ordered.arm_by_root_publish_minimum_cross_misses,
                free.arm_by_root_publish_maximum_cross_misses,
                ordered.arm_by_root_publish_maximum_cross_misses,
                free.recorded_truth_count,
                ordered.recorded_truth_count,
                free.all_writes_truth_count,
                ordered.all_writes_truth_count,
            ),
        );
    }
    for (three_state_label, two_state_label, pair_name) in [("g1_free", "g5_free", "G1_vs_G5_free"), ("g1_ordered", "g5_ordered", "G2_vs_G5_ordered")] {
        let three_state = judgements.iter().find(|judgement| judgement.label == three_state_label).expect("三态格");
        let two_state = judgements.iter().find(|judgement| judgement.label == two_state_label).expect("两态格");
        let same = three_state.arm_by_root_publish_leak_judgement == two_state.arm_by_root_publish_leak_judgement
            && three_state.arm_by_transaction_cross_misses == two_state.arm_by_transaction_cross_misses
            && invalid_prefix_judgement(0, three_state.recorded_truth_count) == invalid_prefix_judgement(0, two_state.recorded_truth_count);
        emit(
            emitter,
            format!(
                "name=geometry_sensitivity pair={pair_name} system_configuration_third_state_changes_judgement={} l2_judgement_three_state={} l2_judgement_two_state={} cross_misses_minimum_three_state={} cross_misses_minimum_two_state={} cross_misses_maximum_three_state={} cross_misses_maximum_two_state={} recorded_truth_three_state={} recorded_truth_two_state={}",
                yes_no(!same),
                three_state.arm_by_root_publish_leak_judgement,
                two_state.arm_by_root_publish_leak_judgement,
                three_state.arm_by_root_publish_minimum_cross_misses,
                two_state.arm_by_root_publish_minimum_cross_misses,
                three_state.arm_by_root_publish_maximum_cross_misses,
                two_state.arm_by_root_publish_maximum_cross_misses,
                three_state.recorded_truth_count,
                two_state.recorded_truth_count,
            ),
        );
    }
}

fn run_failure_clauses(emitter: &mut Emitter, evaluations: &[CellEvaluation], judgements: &[CellJudgement]) {
    let reference_triggered = evaluations.iter().any(|evaluation| difference_count(&evaluation.exact_reachable, &evaluation.truth) > 0);
    emit(emitter, format!("name=failure_clause clause=F1 triggered={} observation=exact_not_in_truth_over_all_cells_or_closed_form_mismatch", yes_no(reference_triggered)));
    let both_lose = judgements.iter().any(|judgement| {
        judgement.candidate == Candidate::ManyPublishesInFlightFree && judgement.disks == 1 && judgement.arm_by_root_publish_leak_judgement == "leaks_on_every_stream" && judgement.arm_by_transaction_cross_misses > 0
    });
    emit(emitter, format!("name=failure_clause clause=F2 triggered={} observation=l2_minimum_positive_and_l3b_positive_on_the_same_geometry_one_disk_only", yes_no(both_lose)));
    for (free_label, ordered_label) in [("g1_free", "g1_ordered"), ("g3_free", "g3_ordered"), ("g5_free", "g5_ordered")] {
        let free = judgements.iter().find(|judgement| judgement.label == free_label).expect("自由交错格");
        let ordered = judgements.iter().find(|judgement| judgement.label == ordered_label).expect("带约束格");
        emit(
            emitter,
            format!(
                "name=failure_clause clause=F3 pair={free_label}_vs_{ordered_label} triggered={}",
                yes_no(free.arm_by_root_publish_leak_judgement != ordered.arm_by_root_publish_leak_judgement)
            ),
        );
    }
    for judgement in judgements.iter().filter(|judgement| judgement.candidate == Candidate::ManyPublishesInFlightFree) {
        emit(
            emitter,
            format!(
                "name=failure_clause clause=F4 cell={} recorded_truth_count={} all_writes_truth_count={} triggered_recorded={} triggered_all_writes={}",
                judgement.label,
                judgement.recorded_truth_count,
                judgement.all_writes_truth_count,
                yes_no(judgement.recorded_truth_count == 0),
                yes_no(judgement.all_writes_truth_count == 0),
            ),
        );
    }
    emit(emitter, "name=failure_clause clause=F5 status=not_applicable reason=judged_on_two_disk_geometries_in_the_second_segment".to_string());
}

// ── 主流程 ─────────────────────────────────────────────────────────────────────────────

pub(super) fn run_everything(emitter: &mut Emitter) -> RunTally {
    let mut tally = RunTally::default();
    emit(
        emitter,
        "name=config model=independent_count_model candidates=4 sampling_points=G1,G2,G3,G3_ordered,G5,G5_ordered anchor_geometries=G1,G3,G4a,G4b,G5 segment=first not_run=L8,L9,G4a_cells,G4b_cells".to_string(),
    );
    run_halt_checks(emitter, &mut tally);
    if !tally.halt_conditions.is_empty() {
        return tally;
    }
    run_anchor_table(emitter, &mut tally);
    let evaluations: Vec<CellEvaluation> = cell_specifications().iter().map(evaluate_cell).collect();
    let mut judgements = Vec::new();
    for evaluation in &evaluations {
        judgements.push(emit_cell(emitter, &mut tally, evaluation));
    }
    for evaluation in &evaluations {
        if matches!(evaluation.world.candidate, Candidate::SerialPipeline | Candidate::OnePublishManyTransactions) {
            let miss_maximum = evaluation.streams.iter().map(|metrics| metrics.cross_misses + metrics.non_cross_misses).max().unwrap_or(0);
            let extra_states = isize_from_usize(evaluation.streams.iter().map(|metrics| metrics.state_count).max().unwrap_or(0)) - isize_from_usize(evaluation.truth.len());
            tally.self_check(miss_maximum == 0 && extra_states == 0, &format!("V1:real_baseline:{}", evaluation.label));
        }
    }
    run_positive_controls(emitter, &mut tally, &evaluations);
    run_threshold_shifts(emitter, &mut tally, &evaluations, &judgements);
    run_geometry_sensitivity(emitter, &judgements);
    run_failure_clauses(emitter, &evaluations, &judgements);
    for judgement in &judgements {
        emit(
            emitter,
            format!(
                "name=verdict cell={} sampling_point={} arm_by_root_publish_l2_judgement={} arm_by_root_publish_cross_misses_minimum={} arm_by_root_publish_cross_misses_maximum={} arm_by_transaction_cross_misses={}",
                judgement.label,
                judgement.sampling_point,
                judgement.arm_by_root_publish_leak_judgement,
                judgement.arm_by_root_publish_minimum_cross_misses,
                judgement.arm_by_root_publish_maximum_cross_misses,
                judgement.arm_by_transaction_cross_misses,
            ),
        );
    }
    tally
}
