// ── 第二段结束标记 ──

use e7_index_bench::{low_byte_of_usize, usize_from_u64};
use super::*;

fn evaluation_of(label: &str) -> CellEvaluation {
    let specification = cell_specifications().into_iter().find(|candidate| candidate.label == label).expect("这个格存在");
    evaluate_cell(&specification)
}

// ── 甲类：每条流上的枚举域等于闭式 ──────────────────────────────────────────────

#[test]
fn every_stream_of_every_one_disk_cell_has_as_many_states_as_the_closed_form() {
    for label in ["g1_serial", "g1_one_publish", "g1_free", "g1_ordered", "g3_serial", "g3_one_publish", "g3_free", "g3_ordered", "g5_serial", "g5_one_publish", "g5_free", "g5_ordered"] {
        let evaluation = evaluation_of(label);
        for metrics in &evaluation.streams {
            assert_eq!(metrics.state_count as u128, metrics.closed_form_state_count, "{label}");
        }
    }
}

#[test]
fn closed_form_starts_from_one_not_zero() {
    let world = build_world(Candidate::SerialPipeline, &geometry_one_disk_two_units());
    let segments = write_segments(&world.table, world.serial_stream.as_ref().expect("串行流"));
    let sum_of_segments: u128 = segments.iter().map(|segment| segment_expansion(&world.table, segment) - 1).sum();
    assert_eq!(closed_form_state_count(&world.table, &segments), sum_of_segments + 1);
}

// ── 丙类：独立算出、命令核过的绝对值 ────────────────────────────────────────────

#[test]
fn closed_form_anchor_table_matches_registered_numbers() {
    for row in anchor_rows() {
        for reading in read_anchor_row(&row) {
            assert_eq!(reading.actual, reading.expected, "{} {}", row.geometry.label, reading.quantity);
        }
    }
}

#[test]
fn serial_pipeline_on_one_disk_with_two_units_has_fifteen_states_on_its_only_stream() {
    let evaluation = evaluation_of("g1_serial");
    assert_eq!(evaluation.truth.len(), 15);
    assert_eq!(evaluation.streams.len(), 1);
    assert_eq!(evaluation.streams[0].state_count, 15);
}

#[test]
fn serial_segments_split_at_every_barrier_and_force_unit_access_write() {
    let world = build_world(Candidate::SerialPipeline, &geometry_one_disk_two_units());
    let serial = world.serial_stream.as_ref().expect("串行流");
    assert_eq!(segment_sizes_text(&world.table, serial), "2+1+1+1+2+1+1+1");
}

// ── 乙类：登记表钉住的（S1 停机条款）────────────────────────────────────────────

#[test]
fn registered_ordinary_publish_chain_has_the_registered_segments_kinds_operations_and_states() {
    let reading = registered_ordinary_publish_reading();
    assert_eq!(reading.segments, "24+2+1+2");
    assert_eq!(reading.kinds, REGISTERED_ORDINARY_PUBLISH_KINDS);
    assert_eq!(reading.operations, 35);
    assert_eq!(reading.closed_form_states, 16_777_223);
    assert_eq!(reading.enumerated_states, 16_777_223);
}

#[test]
fn registered_whole_stream_has_the_registered_segments_kinds_operations_and_states() {
    let two_state = registered_whole_stream_reading(false, true);
    assert_eq!(two_state.segments, REGISTERED_WHOLE_STREAM_SEGMENTS);
    assert_eq!(two_state.kinds, REGISTERED_WHOLE_STREAM_KINDS);
    assert_eq!(two_state.operations, 57);
    assert_eq!(two_state.closed_form_states, 16_777_240);
    assert_eq!(two_state.enumerated_states, 16_777_240);
    let three_state = registered_whole_stream_reading(true, false);
    assert_eq!(three_state.closed_form_states, 16_777_260);
}

// ── S2：发布归属 ───────────────────────────────────────────────────────────────

#[test]
fn publish_attribution_on_the_serial_stream_matches_the_hand_derivation() {
    assert!(attribution_matches_hand_derivation());
}

#[test]
fn unit_write_of_the_next_transaction_issued_before_the_root_is_attributed_to_the_earlier_publish() {
    let world = build_world(Candidate::ManyPublishesInFlightFree, &geometry_one_disk_two_units());
    let index_of = |name: &str| index_of_write(&world.table, name);
    let stream = vec![
        Event::Write(index_of("a1")),
        Event::Write(index_of("b1")),
        Event::Write(index_of("rA")),
        Event::Write(index_of("RA")),
        Event::Write(index_of("rB")),
        Event::Write(index_of("RB")),
    ];
    let attribution = publish_attribution(&world.table, &stream);
    assert_eq!(attribution[0].units, vec![index_of("a1"), index_of("b1")]);
    assert_eq!(attribution[1].records, vec![index_of("rB")]);
}

// ── 逐格独立再算（Python 脚本 independent.py 的读数，定义见登记第一节）──────────────────

struct PinnedCell {
    label: &'static str,
    streams: usize,
    truth: usize,
    states_minimum: usize,
    states_maximum: usize,
    cross_misses_minimum: usize,
    cross_misses_maximum: usize,
    non_cross_misses_maximum: usize,
    cross_states_of_truth: usize,
    arm_by_transaction_not_in_truth: usize,
    recorded_truth: usize,
    recorded_covered_maximum: usize,
    all_writes_truth: usize,
    all_writes_covered_maximum: usize,
}

const PINNED_CELLS: [PinnedCell; 6] = [
    PinnedCell { label: "g1_free", streams: 12870, truth: 64, states_minimum: 15, states_maximum: 29, cross_misses_minimum: 27, cross_misses_maximum: 42, non_cross_misses_maximum: 11, cross_states_of_truth: 49, arm_by_transaction_not_in_truth: 0, recorded_truth: 16, recorded_covered_maximum: 13, all_writes_truth: 6, all_writes_covered_maximum: 6 },
    PinnedCell { label: "g1_ordered", streams: 3747, truth: 46, states_minimum: 15, states_maximum: 29, cross_misses_minimum: 13, cross_misses_maximum: 28, non_cross_misses_maximum: 7, cross_states_of_truth: 35, arm_by_transaction_not_in_truth: 18, recorded_truth: 1, recorded_covered_maximum: 1, all_writes_truth: 2, all_writes_covered_maximum: 2 },
    PinnedCell { label: "g3_free", streams: 3432, truth: 36, states_minimum: 11, states_maximum: 17, cross_misses_minimum: 11, cross_misses_maximum: 20, non_cross_misses_maximum: 9, cross_states_of_truth: 25, arm_by_transaction_not_in_truth: 0, recorded_truth: 8, recorded_covered_maximum: 7, all_writes_truth: 4, all_writes_covered_maximum: 4 },
    PinnedCell { label: "g3_ordered", streams: 973, truth: 26, states_minimum: 11, states_maximum: 17, cross_misses_minimum: 5, cross_misses_maximum: 14, non_cross_misses_maximum: 5, cross_states_of_truth: 19, arm_by_transaction_not_in_truth: 10, recorded_truth: 1, recorded_covered_maximum: 1, all_writes_truth: 2, all_writes_covered_maximum: 2 },
    PinnedCell { label: "g5_free", streams: 12870, truth: 49, states_minimum: 13, states_maximum: 24, cross_misses_minimum: 19, cross_misses_maximum: 30, non_cross_misses_maximum: 9, cross_states_of_truth: 36, arm_by_transaction_not_in_truth: 0, recorded_truth: 12, recorded_covered_maximum: 9, all_writes_truth: 5, all_writes_covered_maximum: 5 },
    PinnedCell { label: "g5_ordered", streams: 3747, truth: 36, states_minimum: 13, states_maximum: 24, cross_misses_minimum: 9, cross_misses_maximum: 20, non_cross_misses_maximum: 6, cross_states_of_truth: 26, arm_by_transaction_not_in_truth: 13, recorded_truth: 1, recorded_covered_maximum: 1, all_writes_truth: 1, all_writes_covered_maximum: 1 },
];

#[test]
fn many_publishes_in_flight_cells_match_the_independent_python_recount() {
    for pinned in &PINNED_CELLS {
        let evaluation = evaluation_of(pinned.label);
        let table = &evaluation.world.table;
        assert_eq!(evaluation.streams.len(), pinned.streams, "{}", pinned.label);
        assert_eq!(evaluation.truth.len(), pinned.truth, "{}", pinned.label);
        let state_counts: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.state_count).collect();
        assert_eq!(state_counts.iter().copied().min().expect("非空"), pinned.states_minimum, "{}", pinned.label);
        assert_eq!(maximum_value(&state_counts), pinned.states_maximum, "{}", pinned.label);
        let cross_misses: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.cross_misses).collect();
        assert_eq!(cross_misses.iter().copied().min().expect("非空"), pinned.cross_misses_minimum, "{}", pinned.label);
        assert_eq!(maximum_value(&cross_misses), pinned.cross_misses_maximum, "{}", pinned.label);
        assert!(cross_misses.iter().all(|misses| *misses > 0), "{}：每一条流都漏", pinned.label);
        let non_cross: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.non_cross_misses).collect();
        assert_eq!(maximum_value(&non_cross), pinned.non_cross_misses_maximum, "{}", pinned.label);
        assert_eq!(evaluation.truth.iter().filter(|state| is_cross_transaction(table, **state)).count(), pinned.cross_states_of_truth, "{}", pinned.label);
        assert_eq!(difference_count(&evaluation.arm_by_transaction, &evaluation.truth), pinned.arm_by_transaction_not_in_truth, "{}", pinned.label);
        assert_eq!(count_cross_transaction_misses(table, &evaluation.truth, &evaluation.arm_by_transaction), 0, "{}", pinned.label);
        assert_eq!(count_invalid_prefixes_in(table, &evaluation.truth, InvalidPrefixKind::ByRecord), pinned.recorded_truth, "{}", pinned.label);
        assert_eq!(count_invalid_prefixes_in(table, &evaluation.truth, InvalidPrefixKind::ByAllWrites), pinned.all_writes_truth, "{}", pinned.label);
        let recorded_covered: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.recorded_prefixes_covered).collect();
        let all_writes_covered: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.all_writes_prefixes_covered).collect();
        assert_eq!(maximum_value(&recorded_covered), pinned.recorded_covered_maximum, "{}", pinned.label);
        assert_eq!(maximum_value(&all_writes_covered), pinned.all_writes_covered_maximum, "{}", pinned.label);
        assert_eq!(recorded_covered.iter().copied().min().expect("非空"), 0, "{}", pinned.label);
    }
}

#[test]
fn the_serial_stream_is_one_of_the_free_interleavings_and_its_state_count_is_fifteen() {
    let evaluation = evaluation_of("g1_free");
    let position = evaluation.serial_stream_position.expect("串行流是自由交错里的一条");
    assert_eq!(evaluation.streams[position].state_count, 15);
}

// ── L1：两条算法逐状态相同 ─────────────────────────────────────────────────────

#[test]
fn truth_by_enumeration_and_truth_by_search_agree_state_by_state_on_every_one_disk_cell() {
    for label in ["g1_serial", "g1_one_publish", "g1_free", "g1_ordered", "g3_serial", "g3_one_publish", "g3_free", "g3_ordered", "g5_serial", "g5_one_publish", "g5_free", "g5_ordered"] {
        let evaluation = evaluation_of(label);
        assert_eq!(evaluation.truth, evaluation.truth_by_search, "{label}");
    }
}

#[test]
fn the_search_finds_no_stream_for_a_state_that_no_stream_reaches() {
    let evaluation = evaluation_of("g1_serial");
    let table = &evaluation.world.table;
    let unreachable = state_with_only(table, &["b1"]);
    assert!(!contains(&evaluation.truth, unreachable));
    assert!(!some_stream_realizes_target(&evaluation.world, unreachable));
    let free = evaluation_of("g1_free");
    assert!(some_stream_realizes_target(&free.world, unreachable));
}

#[test]
fn the_ordered_candidate_has_the_independently_counted_number_of_streams() {
    let world = build_world(Candidate::ManyPublishesInFlightOrdered, &geometry_one_disk_two_units());
    assert_eq!(world.streams.len(), 3747);
    let free = build_world(Candidate::ManyPublishesInFlightFree, &geometry_one_disk_two_units());
    assert_eq!(free.streams.len(), 12870);
}

// ── 真实基线（V1）──────────────────────────────────────────────────────────────

#[test]
fn real_baseline_serial_pipeline_arm_by_root_publish_equals_truth() {
    let evaluation = evaluation_of("g1_serial");
    assert_eq!(evaluation.streams[0].cross_misses + evaluation.streams[0].non_cross_misses, 0);
    assert_eq!(evaluation.streams[0].state_count, evaluation.truth.len());
    let one_publish = evaluation_of("g1_one_publish");
    assert_eq!(one_publish.streams[0].cross_misses + one_publish.streams[0].non_cross_misses, 0);
    assert_eq!(one_publish.streams[0].state_count, one_publish.truth.len());
}

// ── 按事务切的臂（K2）与恒等式 ───────────────────────────────────────────────────

#[test]
fn arm_by_transaction_on_one_disk_never_misses_a_cross_transaction_state() {
    for label in ["g1_serial", "g1_one_publish", "g1_free", "g1_ordered", "g3_serial", "g3_one_publish", "g3_free", "g3_ordered", "g5_serial", "g5_one_publish", "g5_free", "g5_ordered"] {
        let evaluation = evaluation_of(label);
        assert_eq!(count_cross_transaction_misses(&evaluation.world.table, &evaluation.truth, &evaluation.arm_by_transaction), 0, "{label}");
    }
}

#[test]
fn arm_by_transaction_state_count_is_the_product_of_the_two_chains() {
    let evaluation = evaluation_of("g1_free");
    let older_chain = crash_states_of_stream(&evaluation.world.table, &evaluation.world.arm_chains[0]);
    let newer_chain = crash_states_of_stream(&evaluation.world.table, &evaluation.world.arm_chains[1]);
    assert_eq!(evaluation.arm_by_transaction.len(), older_chain.len() * newer_chain.len());
    assert_eq!(older_chain.len() * newer_chain.len(), 64);
}

#[test]
fn arm_by_transaction_identity_holds_additively() {
    for label in ["g1_ordered", "g3_ordered"] {
        let evaluation = evaluation_of(label);
        let table = &evaluation.world.table;
        let extra = difference_count(&evaluation.arm_by_transaction, &evaluation.truth);
        let cross_misses = count_cross_transaction_misses(table, &evaluation.truth, &evaluation.arm_by_transaction);
        let non_cross_misses = count_non_cross_misses(table, &evaluation.truth, &evaluation.arm_by_transaction);
        assert_eq!(evaluation.arm_by_transaction.len() + cross_misses + non_cross_misses, evaluation.truth.len() + extra, "{label}");
    }
}

// ── L10：精确可达集 ─────────────────────────────────────────────────────────────

#[test]
fn exact_reachable_states_of_the_serial_stream_equal_its_crash_states() {
    let evaluation = evaluation_of("g1_serial");
    assert_eq!(evaluation.exact_reachable.len(), 15);
    assert_eq!(difference_count(&evaluation.exact_reachable, &evaluation.truth), 0);
    assert_eq!(difference_count(&evaluation.truth, &evaluation.exact_reachable), 0);
}

#[test]
fn exact_reachable_states_are_pinned_on_the_two_in_flight_cells() {
    for label in ["g1_free", "g1_ordered", "g3_free", "g3_ordered", "g1_one_publish", "g3_one_publish", "g5_free", "g5_ordered"] {
        let evaluation = evaluation_of(label);
        assert_eq!(difference_count(&evaluation.exact_reachable, &evaluation.truth), 0, "{label}");
        assert_eq!(difference_count(&evaluation.truth, &evaluation.exact_reachable), 0, "{label}");
    }
}

// ── 阳性对照 ───────────────────────────────────────────────────────────────────

#[test]
fn serial_concatenation_misses_the_state_where_only_the_first_unit_of_the_newer_transaction_is_persisted_on_both_arms() {
    let evaluation = evaluation_of("g1_free");
    let table = &evaluation.world.table;
    let witness = state_with_only(table, &["b1"]);
    assert!(contains(&evaluation.truth, witness));
    let serial = evaluation.world.serial_stream.as_ref().expect("串行流");
    assert!(!contains(&crash_states_of_stream(table, serial), witness));
    assert!(!contains(&serial_concatenation_of_transaction_chains(&evaluation.world), witness));
    assert!(difference_count(&evaluation.truth, &crash_states_of_stream(table, serial)) >= 1);
    assert_eq!(difference_count(&evaluation.truth, &crash_states_of_stream(table, serial)) + 15, 64);
}

#[test]
fn cross_transaction_miss_counter_counts_only_the_cross_states_of_the_serial_concatenation() {
    let evaluation = evaluation_of("g1_free");
    let table = &evaluation.world.table;
    let serial = evaluation.world.serial_stream.as_ref().expect("串行流");
    let serial_states = crash_states_of_stream(table, serial);
    let cross_misses = count_cross_transaction_misses(table, &evaluation.truth, &serial_states);
    let non_cross_misses = count_non_cross_misses(table, &evaluation.truth, &serial_states);
    assert_eq!(cross_misses + non_cross_misses, difference_count(&evaluation.truth, &serial_states));
    assert!(non_cross_misses > 0, "单独 B 有写持久的状态不在 C 里，计数器拿 T 去减会多算");
    assert!(cross_misses < difference_count(&evaluation.truth, &serial_states));
}

#[test]
fn deleting_one_state_from_either_arm_raises_its_miss_count_by_exactly_one() {
    let evaluation = evaluation_of("g1_free");
    let table = &evaluation.world.table;
    let two_units = state_with_only(table, &["a1", "b1"]);
    let stream = evaluation.world.streams.iter().find(|stream| contains(&crash_states_of_stream(table, stream), two_units)).expect("有一条流让 a1、b1 同在第一段");
    let states = crash_states_of_stream(table, stream);
    assert_eq!(
        count_cross_transaction_misses(table, &evaluation.truth, &without_state(&states, two_units)),
        count_cross_transaction_misses(table, &evaluation.truth, &states) + 1
    );
    assert_eq!(count_cross_transaction_misses(table, &evaluation.truth, &without_state(&evaluation.arm_by_transaction, two_units)), 1);
}

#[test]
fn deleting_one_invalid_prefix_lowers_the_covered_count_by_exactly_one_on_the_ordered_cell() {
    let evaluation = evaluation_of("g1_ordered");
    let table = &evaluation.world.table;
    let witness = recorded_prefix_witness(table);
    assert!(contains(&evaluation.truth, witness));
    assert!(is_invalid_prefix_by_record(table, witness));
    let covered = count_invalid_prefixes_covered(table, &evaluation.truth, &evaluation.arm_by_transaction, InvalidPrefixKind::ByRecord);
    let covered_after = count_invalid_prefixes_covered(table, &evaluation.truth, &without_state(&evaluation.arm_by_transaction, witness), InvalidPrefixKind::ByRecord);
    assert_eq!(covered, covered_after + 1);
    assert_eq!(covered, 1);
}

#[test]
fn invalid_prefix_by_record_is_directional_between_the_older_and_newer_transactions() {
    let evaluation = evaluation_of("g1_ordered");
    let table = &evaluation.world.table;
    let newer_record_only = state_with_only(table, &["rB"]);
    let older_record_only = state_with_only(table, &["rA"]);
    assert!(is_invalid_prefix_by_record(table, newer_record_only));
    assert!(!is_invalid_prefix_by_record(table, older_record_only));
    assert_eq!(count_invalid_prefixes_in(table, &evaluation.truth, InvalidPrefixKind::ByRecord), 1);
}

// ── L7a：罩住段串 ───────────────────────────────────────────────────────────────

#[test]
fn the_cover_of_the_arm_by_transaction_on_one_disk_is_all_writes_in_one_segment() {
    let evaluation = evaluation_of("g1_free");
    let table = &evaluation.world.table;
    let cover = cover_by_merging(table, &evaluation.arm_by_transaction);
    assert_eq!(cover.segments.len(), 1);
    assert_eq!(cover.segments[0].len(), 10);
    assert_eq!(cover.cover_state_count, 2304);
    assert!(cover.covers_target);
    assert!(!cover.equals_target);
}

#[test]
fn merging_agrees_with_exhaustive_ordered_partitions_on_the_one_unit_cells() {
    for label in ["g3_serial", "g3_one_publish", "g3_free", "g3_ordered"] {
        let evaluation = evaluation_of(label);
        let table = &evaluation.world.table;
        for target in [&evaluation.truth, &evaluation.arm_by_transaction] {
            let merged = cover_by_merging(table, target);
            let exhaustive = cover_by_exhaustive_ordered_partitions(table, target);
            assert_eq!(merged.cover_state_count, exhaustive.minimal_state_count, "{label}");
            assert!(exhaustive.optimal_covers.contains(&merged.segments), "{label}");
            assert!(merged.covers_target, "{label}");
        }
    }
}

#[test]
fn state_set_that_is_exactly_one_stream_has_an_exactly_equal_cover() {
    let evaluation = evaluation_of("g1_serial");
    let cover = cover_by_merging(&evaluation.world.table, &evaluation.truth);
    assert!(cover.equals_target);
    assert_eq!(cover.cover_state_count, 15);
    assert_eq!(cover.segments.len(), 8);
}

#[test]
fn membership_test_agrees_with_generated_crash_states_on_every_vector_of_a_small_stream() {
    let evaluation = evaluation_of("g3_free");
    let table = &evaluation.world.table;
    let stream = &evaluation.world.streams[0];
    let segments = write_segments(table, stream);
    let generated = crash_states_of_segments(table, &segments);
    for vector in all_state_vectors(table) {
        assert_eq!(state_is_in_crash_states_of_segments(&segments, vector), contains(&generated, vector));
    }
}

// ── 枚举器自身 ─────────────────────────────────────────────────────────────────

#[test]
fn doubled_barriers_on_one_disk_do_not_change_the_crash_state_set() {
    let world = build_world(Candidate::SerialPipeline, &geometry_one_disk_two_units());
    let serial = world.serial_stream.as_ref().expect("串行流");
    let mut doubled = Vec::new();
    for event in serial {
        doubled.push(*event);
        if matches!(event, Event::Barrier(_)) {
            doubled.push(*event);
        }
    }
    assert_ne!(doubled.len(), serial.len());
    assert_eq!(crash_states_of_stream(&world.table, &doubled), crash_states_of_stream(&world.table, serial));
}

#[test]
fn torn_write_appears_only_in_the_current_segment_never_before_it() {
    let world = build_world(Candidate::SerialPipeline, &geometry_one_disk_two_units());
    let serial = world.serial_stream.as_ref().expect("串行流");
    let states = crash_states_of_stream(&world.table, serial);
    let older_system_configuration_write = index_of_write(&world.table, "SA");
    let newer_first_write = index_of_write(&world.table, "b1");
    for state in states {
        if value_of(state, older_system_configuration_write) == TORN {
            assert_eq!(value_of(state, newer_first_write), NOT_PERSISTED);
        }
    }
}

#[test]
fn force_unit_access_closes_its_own_segment_and_does_not_make_earlier_writes_durable() {
    let table = vec![
        WriteSpecification { name: "plain".to_string(), disk: 0, kind: StepKind::UnitWrite, owner: Owner::Shared, is_in_place: false },
        WriteSpecification { name: "fua".to_string(), disk: 0, kind: StepKind::RootRecordFua, owner: Owner::Shared, is_in_place: false },
        WriteSpecification { name: "after".to_string(), disk: 0, kind: StepKind::UnitWrite, owner: Owner::Shared, is_in_place: false },
    ];
    let stream = vec![Event::Write(0), Event::Write(1), Event::Write(2), Event::Barrier(0)];
    assert_eq!(write_segments(&table, &stream), vec![vec![0, 1], vec![2]]);
}

#[test]
fn barrier_on_one_disk_does_not_close_a_segment_with_unreleased_writes_on_the_other() {
    let table = vec![
        WriteSpecification { name: "d0".to_string(), disk: 0, kind: StepKind::UnitWrite, owner: Owner::Shared, is_in_place: false },
        WriteSpecification { name: "d1".to_string(), disk: 1, kind: StepKind::UnitWrite, owner: Owner::Shared, is_in_place: false },
        WriteSpecification { name: "later".to_string(), disk: 0, kind: StepKind::UnitWrite, owner: Owner::Shared, is_in_place: false },
    ];
    let stream = vec![Event::Write(0), Event::Write(1), Event::Barrier(0), Event::Write(2), Event::Barrier(1), Event::Barrier(0)];
    assert_eq!(write_segments(&table, &stream), vec![vec![0, 1, 2]]);
}

// ── 两盘几何的区域归属（M10）──────────────────────────────────────────────────────

#[test]
fn root_slots_of_the_two_disk_geometries_follow_the_three_region_assignment() {
    assert_eq!(geometry_g4a().root_disk_of_the_older_transaction, 0);
    assert_eq!(geometry_g4a().root_disk_of_the_newer_transaction, 1);
    assert_eq!(geometry_g4b().root_disk_of_the_older_transaction, 0);
    assert_eq!(geometry_g4b().root_disk_of_the_newer_transaction, 0);
    let world = build_world(Candidate::SerialPipeline, &geometry_g4a());
    let root_disk_of = |prefix: &str| {
        let roots: Vec<&WriteSpecification> = world.table.iter().filter(|specification| specification.kind == StepKind::RootRecordFua && specification.name.starts_with(prefix)).collect();
        assert_eq!(roots.len(), 1);
        roots[0].disk
    };
    assert_eq!(root_disk_of("RB"), 1);
    assert_eq!(root_disk_of("RA"), 0);
}

// ── 判别力自证：挪门槛判定要翻 ────────────────────────────────────────────────────

#[test]
fn moving_the_threshold_to_the_maximum_turns_the_leak_judgement_from_leaking_to_never_leaking() {
    assert_eq!(leak_judgement(27, 42, 0), "leaks_on_every_stream");
    assert_eq!(leak_judgement(0, 42, 0), "leaks_on_some_streams");
    assert_eq!(leak_judgement(27, 42, 42), "never_leaks");
    assert_eq!(leak_judgement(0, 0, 0), "never_leaks");
    assert_eq!(invalid_prefix_judgement(13, 16), "misses_invalid_prefixes");
    assert_eq!(invalid_prefix_judgement(13, 13), "covers_every_invalid_prefix");
    assert_eq!(invalid_prefix_judgement(0, 0), "no_invalid_prefix_in_truth");
}

// ── 能力锚点与结构常量 ───────────────────────────────────────────────────────────

#[test]
fn capacity_anchors_of_the_real_publish_shape_match_the_registered_arithmetic() {
    let world = build_world(Candidate::SerialPipeline, &geometry_real_publish_shape());
    let every_write: Vec<usize> = (0..world.table.len()).collect();
    assert_eq!(world.table.len(), 2 * (24 + 2 + 1 + 2));
    assert_eq!(segment_expansion(&world.table, &every_write), 1_459_166_279_268_040_704);
    let chain_segments = write_segments(&world.table, &world.arm_chains[0]);
    assert_eq!(closed_form_state_count(&world.table, &chain_segments), 16_777_228);
    assert!(std::hint::black_box(1_459_166_279_268_040_704u128 > MAXIMUM_SEGMENT_STATES_IN_U32));
    assert!(std::hint::black_box(1_459_166_279_268_040_704u128 <= MAXIMUM_SEGMENT_STATES_IN_U64));
    assert_eq!(MAXIMUM_ENUMERATED_WRITES, 2048);
}

#[test]
fn two_disk_cover_of_the_arm_by_transaction_is_all_eighteen_writes_in_one_segment() {
    let world = build_world(Candidate::SerialPipeline, &geometry_g4a());
    let product = arm_by_transaction_states(&world);
    let cover = cover_by_merging(&world.table, &product);
    assert_eq!(cover.segments.len(), 1);
    assert_eq!(cover.segments[0].len(), 18);
    assert_eq!(cover.cover_state_count, 1_327_104);
}

// ═══ 第二段：流式路径与两盘几何 ═══════════════════════════════════════════════════

struct CollectingSink {
    leaves: Vec<(u128, u64, [u32; MAXIMUM_STREAMING_WRITES])>,
}

impl LeafSink for CollectingSink {
    fn leaf(&mut self, walk: &Walk, masks: &[u32; MAXIMUM_STREAMING_WRITES]) {
        self.leaves.push((walk.key, walk.path, *masks));
    }
}

/// 数叶子，并数其中满足三条先后约束的：每条约束是（A 的第几步，B 的第几步），要求 A 那一步先发出。
struct CountingSink {
    leaves: u64,
    obeying: u64,
    pairs_of_chain_positions: Vec<(usize, usize)>,
    total_steps: usize,
}

fn order_pairs_of(table: &[WriteSpecification]) -> Vec<(usize, usize)> {
    let find = |prefix: &str, disk_suffix: &str| index_of_write(table, &format!("{prefix}{disk_suffix}"));
    let mut pairs = Vec::new();
    let disks = table.iter().map(|specification| specification.disk).max().expect("写表非空") + 1;
    for disk in 0..disks {
        let suffix = if disks == 1 { String::new() } else { format!("@{disk}") };
        pairs.push((find("rA", &suffix), find("rB", &suffix)));
        pairs.push((find("SA", &suffix), find("SB", &suffix)));
    }
    let root_of = |tag: &str| table.iter().position(|specification| specification.kind == StepKind::RootRecordFua && specification.name.starts_with(&format!("R{tag}"))).expect("每个事务一次根槽写");
    pairs.push((root_of("A"), root_of("B")));
    pairs
}

fn obeys_ordering(pairs: &[(usize, usize)], events: &[Event]) -> bool {
    let position_of = |write: usize| events.iter().position(|event| *event == Event::Write(write)).expect("流里有这次写");
    pairs.iter().all(|&(first, second)| position_of(first) < position_of(second))
}

impl LeafSink for CountingSink {
    fn leaf(&mut self, walk: &Walk, _masks: &[u32; MAXIMUM_STREAMING_WRITES]) {
        self.leaves += 1;
        let (mut older_issue_step, mut newer_issue_step) = ([0u8; 32], [0u8; 32]);
        let (mut older_position, mut newer_position) = (0usize, 0usize);
        for step in 0..self.total_steps {
            if (walk.path >> (self.total_steps - 1 - step)) & 1 == 0 {
                older_issue_step[older_position] = low_byte_of_usize(step);
                older_position += 1;
            } else {
                newer_issue_step[newer_position] = low_byte_of_usize(step);
                newer_position += 1;
            }
        }
        if self.pairs_of_chain_positions.iter().all(|&(older_chain_position, newer_chain_position)| older_issue_step[older_chain_position] < newer_issue_step[newer_chain_position]) {
            self.obeying += 1;
        }
    }
}

/// 深搜前 `steps` 步之后的任务里每隔几个取一个，把这些任务下面的全部叶子收下来：两盘几何的抽查用（不进任何判据，只拿来逐条对拍流函数）。
fn sampled_leaves(world: &World, tasks_wanted: usize) -> (WriteTableSummary, Vec<(u128, u64, [u32; MAXIMUM_STREAMING_WRITES])>) {
    let table_summary = WriteTableSummary::new(&world.table);
    let context = WalkContext::new(world, &table_summary);
    let tasks = split_tasks(&context, 18);
    let stride = (tasks.len() / tasks_wanted).max(1);
    let mut sink = CollectingSink { leaves: Vec::new() };
    for task in tasks.iter().step_by(stride) {
        let mut masks = task.masks;
        walk_interleavings(&context, task.walk, &mut masks, &mut sink);
    }
    (table_summary, sink.leaves)
}

fn two_disk_one_unit_geometry() -> Geometry {
    geometry_two_disks("two_disk_u1", 1, 3)
}

// ── 交错数：登记 S3 的 C(30, 15)，与另一条数法逐格相同 ──

#[test]
fn interleaving_counts_equal_the_independent_python_counts() {
    // 数字来自 /tmp 里那份只按登记第一节读法写的 Python 小脚本（命令与输出在第二段报告里）。
    for (geometry, candidate, expected) in [
        (geometry_one_disk_two_units(), Candidate::ManyPublishesInFlightFree, 12_870u128),
        (geometry_one_disk_two_units(), Candidate::ManyPublishesInFlightOrdered, 3_747),
        (geometry_one_disk_one_unit(), Candidate::ManyPublishesInFlightOrdered, 973),
        (geometry_g4a(), Candidate::ManyPublishesInFlightFree, 155_117_520),
        (geometry_g4a(), Candidate::ManyPublishesInFlightOrdered, 37_834_712),
        (geometry_g4b(), Candidate::ManyPublishesInFlightOrdered, 37_834_712),
        (two_disk_one_unit_geometry(), Candidate::ManyPublishesInFlightFree, 10_400_600),
        (two_disk_one_unit_geometry(), Candidate::ManyPublishesInFlightOrdered, 2_431_135),
    ] {
        let world = build_two_chain_world_without_streams(candidate, &geometry);
        let table_summary = WriteTableSummary::new(&world.table);
        assert_eq!(count_interleavings(&WalkContext::new(&world, &table_summary)), expected, "{} {}", geometry.label, candidate.name());
    }
    assert_eq!(155_117_520u128, (1..=15u128).fold(1u128, |sum, step| sum * (15 + step) / step));
}

#[test]
fn the_depth_first_walk_visits_exactly_the_counted_interleavings_and_the_ordered_ones_are_exactly_those_obeying_the_three_orders() {
    for (candidate, expected) in [(Candidate::ManyPublishesInFlightFree, 10_400_600u64), (Candidate::ManyPublishesInFlightOrdered, 2_431_135)] {
        let world = build_two_chain_world_without_streams(candidate, &two_disk_one_unit_geometry());
        let table_summary = WriteTableSummary::new(&world.table);
        let context = WalkContext::new(&world, &table_summary);
        let pairs_of_chain_positions: Vec<(usize, usize)> = order_pairs_of(&world.table)
            .into_iter()
            .map(|(older_chain_position, newer_chain_position)| {
                (
                    world.search_chains[0].iter().position(|event| *event == Event::Write(older_chain_position)).expect("A 的链里有这次写"),
                    world.search_chains[1].iter().position(|event| *event == Event::Write(newer_chain_position)).expect("B 的链里有这次写"),
                )
            })
            .collect();
        let total_steps = world.search_chains[0].len() + world.search_chains[1].len();
        let mut sink = CountingSink { leaves: 0, obeying: 0, pairs_of_chain_positions, total_steps };
        walk_interleavings(&context, Walk::start(), &mut [0; MAXIMUM_STREAMING_WRITES], &mut sink);
        assert_eq!(sink.leaves, expected, "{}", candidate.name());
        assert_eq!(sink.obeying, if candidate == Candidate::ManyPublishesInFlightFree { 2_431_135 } else { expected }, "{}", candidate.name());
    }
}

#[test]
fn ordered_streams_on_two_disks_put_the_older_root_before_the_newer_root_whichever_disks_they_land_on() {
    let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightOrdered, &geometry_g4a());
    let (_, leaves) = sampled_leaves(&world, 100);
    assert!(leaves.len() > 500);
    let pairs = order_pairs_of(&world.table);
    assert_eq!(pairs.len(), 5);
    let older_root_write = world.table.iter().position(|specification| specification.name == "RA@0").expect("A 的根落盘 0");
    let newer_root_write = world.table.iter().position(|specification| specification.name == "RB@1").expect("B 的根落盘 1");
    assert!(pairs.contains(&(older_root_write, newer_root_write)));
    for (_, path, _) in leaves {
        assert!(obeys_ordering(&pairs, &events_of_path(&world, path)));
    }
}

// ── 流式路径与第一段的内存路径逐项相同 ──

#[test]
fn streaming_evaluation_equals_the_in_memory_evaluation_on_the_one_disk_cells() {
    for label in ["g1_serial", "g1_free", "g1_ordered", "g3_free", "g3_ordered", "g5_free", "g5_ordered"] {
        let specification = cell_specifications().into_iter().find(|candidate| candidate.label == label).expect("这个格存在");
        let memory = evaluate_cell(&specification);
        let streaming = evaluate_streaming_cell(&specification, 3);
        assert_eq!(streaming.truth, memory.truth, "{label}");
        assert_eq!(streaming.truth_by_search, memory.truth_by_search, "{label}");
        assert_eq!(streaming.exact_reachable, memory.exact_reachable, "{label}");
        assert_eq!(streaming.arm_by_transaction, memory.arm_by_transaction, "{label}");
        assert_eq!(usize_from_u64(streaming.leaves), memory.streams.len(), "{label}");
        assert_eq!(streaming.expected_leaves as usize, memory.streams.len(), "{label}");
        let table = &memory.world.table;
        assert_eq!(streaming.truth_cross, memory.truth.iter().filter(|state| is_cross_transaction(table, **state)).count(), "{label}");
        for kind in BOTH_INVALID_PREFIX_KINDS {
            let by_masks = match kind {
                InvalidPrefixKind::ByRecord => streaming.truth_invalid_record,
                InvalidPrefixKind::ByAllWrites => streaming.truth_invalid_all_writes,
            };
            assert_eq!(by_masks, count_invalid_prefixes_in(table, &memory.truth, kind), "{label}");
        }
        let weighted = |value_of_row: &dyn Fn(&SignatureRow) -> usize| -> Vec<(usize, u64)> {
            let mut by_value: std::collections::BTreeMap<usize, u64> = std::collections::BTreeMap::new();
            for row in &streaming.rows {
                *by_value.entry(value_of_row(row)).or_insert(0) += row.count;
            }
            by_value.into_iter().collect()
        };
        let memory_distribution = |value_of_stream: &dyn Fn(&StreamMetrics) -> usize| -> Vec<(usize, u64)> {
            let mut by_value: std::collections::BTreeMap<usize, u64> = std::collections::BTreeMap::new();
            for metrics in &memory.streams {
                *by_value.entry(value_of_stream(metrics)).or_insert(0) += 1;
            }
            by_value.into_iter().collect()
        };
        assert_eq!(weighted(&|row| streaming.truth_cross - row.cross_states as usize), memory_distribution(&|metrics| metrics.cross_misses), "{label} 跨事务漏数的分布");
        assert_eq!(
            weighted(&|row| (streaming.truth.len() - streaming.truth_cross) - (row.states - row.cross_states) as usize),
            memory_distribution(&|metrics| metrics.non_cross_misses),
            "{label} 跨事务之外漏数的分布"
        );
        assert_eq!(weighted(&|row| row.states as usize), memory_distribution(&|metrics| metrics.state_count), "{label} 状态数的分布");
        assert_eq!(weighted(&|row| row.invalid_record_states as usize), memory_distribution(&|metrics| metrics.recorded_prefixes_covered), "{label} 按记录覆盖数的分布");
        assert_eq!(weighted(&|row| row.invalid_all_writes_states as usize), memory_distribution(&|metrics| metrics.all_writes_prefixes_covered), "{label} 按全部写覆盖数的分布");
        assert_eq!(weighted(&|row| usize::from(row.largest_segment_writes)), memory_distribution(&|metrics| metrics.largest_segment_writes), "{label} 最大一段写数的分布");
        assert!(streaming.rows.iter().all(|row| row.closed_form_matches), "{label}");
    }
}

#[test]
fn the_streaming_judgement_and_the_minimum_cross_miss_stream_equal_the_in_memory_ones() {
    for label in ["g1_free", "g1_ordered", "g3_free", "g5_ordered"] {
        let specification = cell_specifications().into_iter().find(|candidate| candidate.label == label).expect("这个格存在");
        let memory = evaluate_cell(&specification);
        let streaming = evaluate_streaming_cell(&specification, 2);
        let mut memory_tally = RunTally::default();
        let mut streaming_tally = RunTally::default();
        let memory_base = emit_cell(&mut Emitter::new(), &mut memory_tally, &memory);
        let memory_judgement = second_judgement_of_in_memory(&memory, memory_base);
        let streaming_judgement = emit_streaming_cell(&mut Emitter::new(), &mut streaming_tally, &streaming);
        assert!(memory_tally.void_conditions.is_empty() && streaming_tally.void_conditions.is_empty(), "{label}");
        let (memory_side, streaming_side) = (&memory_judgement, &streaming_judgement);
        assert_eq!(memory_side.base.arm_by_root_publish_leak_judgement, streaming_side.base.arm_by_root_publish_leak_judgement, "{label}");
        assert_eq!(memory_side.base.arm_by_root_publish_minimum_cross_misses, streaming_side.base.arm_by_root_publish_minimum_cross_misses, "{label}");
        assert_eq!(memory_side.base.arm_by_root_publish_maximum_cross_misses, streaming_side.base.arm_by_root_publish_maximum_cross_misses, "{label}");
        assert_eq!((memory_side.l3a, memory_side.l3b, memory_side.exact_not_in_truth, memory_side.truth_not_in_exact), (streaming_side.l3a, streaming_side.l3b, streaming_side.exact_not_in_truth, streaming_side.truth_not_in_exact), "{label}");
        assert_eq!((memory_side.base.recorded_truth_count, memory_side.base.all_writes_truth_count), (streaming_side.base.recorded_truth_count, streaming_side.base.all_writes_truth_count), "{label}");
        assert_eq!(memory_side.best_stream_recorded_covered, streaming_side.best_stream_recorded_covered, "{label}");
        assert_eq!(memory_side.arm_recorded_covered, streaming_side.arm_recorded_covered, "{label}");
        assert_eq!((memory_side.root_slot_cut_largest_segment_writes, memory_side.root_slot_cut_largest_segment_expansion), (streaming_side.root_slot_cut_largest_segment_writes, streaming_side.root_slot_cut_largest_segment_expansion), "{label}");
        let cross_misses: Vec<usize> = memory.streams.iter().map(|metrics| metrics.cross_misses).collect();
        let (_, position) = minimum_with_position(&cross_misses);
        let minimum_row = streaming
            .rows
            .iter()
            .filter(|row| streaming.truth_cross - row.cross_states as usize == memory_side.base.arm_by_root_publish_minimum_cross_misses)
            .min_by_key(|row| row.representative_path)
            .expect("有取到最小值的签名");
        assert_eq!(render_path(&streaming.world, minimum_row.representative_path), render_stream(&memory.world.table, &memory.world.streams[position]), "{label}");
    }
}

// ── 流式路径的零件逐个对拍 ──

#[test]
fn the_fixed_array_search_equals_the_set_based_search_on_every_one_disk_cell_and_on_the_two_disk_one_publish_cell() {
    for specification in cell_specifications() {
        let world = build_world(specification.candidate, &specification.geometry);
        assert_eq!(truth_by_search_fast(&world), truth_by_search(&world), "{}", specification.label);
    }
    let world = build_world(Candidate::OnePublishManyTransactions, &geometry_g4a());
    assert_eq!(truth_by_search_fast(&world), truth_by_search(&world), "g4a_one_publish");
    let serial_pipeline_world = build_world(Candidate::SerialPipeline, &two_disk_one_unit_geometry());
    assert_eq!(truth_by_search_fast(&serial_pipeline_world), truth_by_search(&serial_pipeline_world), "two_disk_u1_serial");
}

#[test]
fn index_and_code_agree_and_the_category_masks_equal_the_per_state_predicates() {
    for geometry in [geometry_one_disk_two_units(), geometry_g4a(), geometry_g4b()] {
        let world = build_world(Candidate::SerialPipeline, &geometry);
        let table = &world.table;
        let table_summary = WriteTableSummary::new(table);
        let mut seen: Vec<StateCode> = (0..table_summary.total_vectors).map(|index| StateCode::from(table_summary.code_of_index(index))).collect();
        seen.sort_unstable();
        assert_eq!(seen, all_state_vectors(table), "{}", geometry.label);
        for index in (0..table_summary.total_vectors).step_by(7) {
            let code = table_summary.code_of_index(index);
            let state = StateCode::from(code);
            assert_eq!(table_summary.is_cross(code), is_cross_transaction(table, state), "{} {index}", geometry.label);
            assert_eq!(table_summary.is_invalid_by_record(code), is_invalid_prefix_by_record(table, state), "{} {index}", geometry.label);
            assert_eq!(table_summary.is_invalid_by_all_writes(code), is_invalid_prefix_by_all_writes(table, state), "{} {index}", geometry.label);
        }
    }
}

#[test]
fn walk_signatures_and_order_masks_equal_the_stream_functions_on_sampled_two_disk_streams() {
    for (candidate, geometry) in [
        (Candidate::ManyPublishesInFlightFree, geometry_g4a()),
        (Candidate::ManyPublishesInFlightFree, geometry_g4b()),
        (Candidate::ManyPublishesInFlightOrdered, geometry_g4a()),
    ] {
        let world = build_two_chain_world_without_streams(candidate, &geometry);
        let (table_summary, leaves) = sampled_leaves(&world, 200);
        assert!(leaves.len() > 800, "{}", geometry.label);
        for (key, path, direct) in leaves {
            let events = events_of_path(&world, path);
            let expected_segments: Vec<u32> = write_segments(&world.table, &events).iter().map(|segment| segment.iter().fold(0u32, |mask, &write| mask | (1u32 << write))).collect();
            assert_eq!(segments_of_signature(key, table_summary.write_count), expected_segments, "{} {path}", geometry.label);
            let closed = close_masks(&direct, table_summary.write_count);
            assert_eq!(&closed[..table_summary.write_count], &must_persist_before_masks(&world.table, &events)[..], "{} {path}", geometry.label);
        }
    }
}

#[test]
fn enumeration_by_signature_equals_the_stream_enumeration_state_by_state_with_consistent_indexes() {
    let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry_g4a());
    let (table_summary, leaves) = sampled_leaves(&world, 20);
    for (key, path, _) in leaves.into_iter().step_by(11).take(60) {
        let segments = segments_of_signature(key, table_summary.write_count);
        let mut states: StateList = Vec::new();
        for_each_crash_state(&table_summary, &segments, |code, index| {
            assert_eq!(table_summary.code_of_index(index), code);
            states.push(StateCode::from(code));
        });
        assert_eq!(states.len() as u128, closed_form_of_segments(&table_summary, &segments));
        states.sort_unstable();
        assert_eq!(states, crash_states_of_stream(&world.table, &events_of_path(&world, path)));
    }
}

#[test]
fn the_recursive_down_set_enumeration_equals_the_subset_scan_on_two_disk_orders() {
    let free = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry_g4a());
    let (table_summary, leaves) = sampled_leaves(&free, 30);
    let mut checked = 0;
    for (_, path, _) in leaves.into_iter().step_by(37).take(25) {
        let events = events_of_path(&free, path);
        let masks = must_persist_before_masks(&free.table, &events);
        let mut direct = [0u32; MAXIMUM_STREAMING_WRITES];
        direct[..masks.len()].copy_from_slice(&masks);
        let mut states: StateList = Vec::new();
        for_each_exact_state(&table_summary, &direct, &mut |index| states.push(StateCode::from(table_summary.code_of_index(index))));
        states.sort_unstable();
        assert_eq!(states, sorted_deduplicated(reachable_states_of_partial_order(&free.table, &masks)), "{path}");
        checked += 1;
    }
    assert_eq!(checked, 25);
}

#[test]
fn true_cross_misses_count_only_the_cross_states_that_are_both_in_the_truth_and_exactly_reachable() {
    let world = build_world(Candidate::SerialPipeline, &geometry_one_disk_two_units());
    let table = &world.table;
    let reachable = state_with_only(table, &["a1", "b1"]);
    let model_only = state_with_only(table, &["a1", "a2", "b1"]);
    let truth = sorted_deduplicated(vec![reachable, model_only]);
    let exact = vec![reachable];
    assert_eq!(count_cross_transaction_misses(table, &truth, &[]), 2);
    assert_eq!(true_cross_misses(table, &truth, &exact, &[]), 1);
    assert_eq!(true_cross_misses(table, &truth, &exact, &[reachable]), 0);
}

#[test]
fn pass_one_keeps_the_smallest_path_and_the_full_count_of_every_signature_whatever_the_thread_split() {
    for geometry in [geometry_one_disk_two_units(), geometry_one_disk_one_unit()] {
        let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry);
        let table_summary = WriteTableSummary::new(&world.table);
        let context = WalkContext::new(&world, &table_summary);
        let mut sink = CollectingSink { leaves: Vec::new() };
        walk_interleavings(&context, Walk::start(), &mut [0; MAXIMUM_STREAMING_WRITES], &mut sink);
        let mut expected: std::collections::BTreeMap<u128, (u64, u64)> = std::collections::BTreeMap::new();
        for (key, path, _) in &sink.leaves {
            let entry = expected.entry(*key).or_insert((0, u64::MAX));
            entry.0 += 1;
            entry.1 = entry.1.min(*path);
        }
        let expected: Vec<(u128, u64, u64)> = expected.into_iter().map(|(key, (count, path))| (key, count, path)).collect();
        for threads in [1, 4] {
            assert_eq!(pass_one(&context, threads).signatures, expected, "{} {threads}", geometry.label);
        }
    }
}

// ── L8：最大一段的动态规划 ──

#[test]
fn the_largest_segment_dynamic_programming_equals_the_maximum_over_every_signature_on_the_cells_with_a_brute_force_count() {
    for geometry in [geometry_one_disk_two_units(), geometry_one_disk_one_unit(), two_disk_one_unit_geometry()] {
        let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry);
        let table_summary = WriteTableSummary::new(&world.table);
        let context = WalkContext::new(&world, &table_summary);
        let pass = pass_one(&context, 4);
        let mut most_expansion = 0u128;
        let mut most_writes = 0u32;
        for (key, _, _) in &pass.signatures {
            for segment in segments_of_signature(*key, table_summary.write_count) {
                most_expansion = most_expansion.max(expansion_of_segment(&table_summary, segment));
                most_writes = most_writes.max(segment.count_ones());
            }
        }
        let by_states = largest_segment_of(&world, SegmentObjective::Expansion);
        let by_writes = largest_segment_of(&world, SegmentObjective::Writes);
        assert_eq!(by_states.value, most_expansion, "{}", geometry.label);
        assert_eq!(by_writes.value, u128::from(most_writes), "{}", geometry.label);
        assert!(by_states.replay_agrees && by_writes.replay_agrees, "{}", geometry.label);
    }
}

#[test]
fn the_largest_segment_witness_stream_on_one_disk_two_units_is_pinned() {
    // 一盘：任何屏障与 FUA 都放行唯一的盘，一段里的写只能是两条链在同一对屏障之间的写：手推最大是两个事务的两个单元写共四个
    // （a1 a2 b1 b2 都发在 A 的第一道屏障之前），别的段（记录、根与系统配置槽）至多两个。
    let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry_one_disk_two_units());
    let by_writes = largest_segment_of(&world, SegmentObjective::Writes);
    assert_eq!(by_writes.writes_in_best_segment, 4);
    assert_eq!(by_writes.value, 4);
    let by_states = largest_segment_of(&world, SegmentObjective::Expansion);
    assert_eq!(by_states.states_in_best_segment, (1 << 4) - 1);
}

#[test]
fn real_publish_shape_largest_root_slot_cut_segment_is_at_least_the_registered_forty_eight_writes_and_the_witness_replays() {
    let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry_real_publish_shape());
    let by_writes = largest_segment_of(&world, SegmentObjective::Writes);
    let by_states = largest_segment_of(&world, SegmentObjective::Expansion);
    assert!(by_writes.replay_agrees && by_states.replay_agrees);
    assert!(by_writes.writes_in_best_segment >= 48);
    assert!(by_states.states_in_best_segment >= 281_474_976_710_655);
    assert_eq!(by_states.states_in_best_segment + 1, by_states.value);
    // 登记第四节第 7 条算过的「全部写一段」：2^54 · 3^4 个组合、少去全零一个。
    assert_eq!(by_writes.writes_in_best_segment, 58);
    assert_eq!(by_states.states_in_best_segment, 1_459_166_279_268_040_703);
}

// ── 第三段补格：L11 ──

#[test]
fn the_true_cross_target_keeps_only_the_cross_states_in_both_the_truth_and_the_exact_set_by_hand() {
    let world = build_world(Candidate::SerialPipeline, &geometry_one_disk_one_unit());
    let table = &world.table;
    let in_both = state_with_only(table, &["a1", "b1"]);
    let in_truth_only = state_with_only(table, &["a1", "rA", "b1"]);
    let in_exact_only = state_with_only(table, &["a1", "rA", "RA", "b1"]);
    let not_cross = state_with_only(table, &["a1"]);
    let truth = sorted_deduplicated(vec![in_both, in_truth_only, not_cross]);
    let exact = sorted_deduplicated(vec![in_both, in_exact_only, not_cross]);
    assert_eq!(true_cross_target(table, &truth, &exact), vec![u64::try_from(in_both).expect("装得进 u64")]);
}

#[test]
fn membership_in_segment_masks_matches_the_hand_derived_cases_and_the_miss_counter_counts_deletions() {
    // 三次写，段串 [写 0 与写 1] [写 2]；两位一写：写 0 取 1 = 0b01，写 1 取 1 = 0b0100，写 2 取 1 = 0b010000。
    let masks = segment_digit_masks(&[0b011, 0b100]);
    let (persisted_write_zero, persisted_write_one, persisted_write_two) = (0b01u64, 0b01u64 << 2, 0b01u64 << 4);
    for code in [0, persisted_write_zero, persisted_write_one, persisted_write_zero | persisted_write_one, persisted_write_zero | persisted_write_one | persisted_write_two, 0b10] {
        assert!(code_in_digit_masks(&masks, code), "{code:#b}");
    }
    for code in [persisted_write_two, persisted_write_zero | persisted_write_two, persisted_write_one | persisted_write_two, 0b10 | persisted_write_two] {
        assert!(!code_in_digit_masks(&masks, code), "{code:#b}");
    }
    let target = [persisted_write_zero, persisted_write_two, persisted_write_zero | persisted_write_one | persisted_write_two];
    // 手算：w0 在（当前段 0）、w2 不在（前一段没全持久）、全持久在（当前段 1）→ 漏 1；再从枚举域里删 w0 → 漏 2。
    assert_eq!(true_cross_misses_of_masks(&masks, &target, None), 1);
    assert_eq!(true_cross_misses_of_masks(&masks, &target, Some(persisted_write_zero)), 2);
}

#[test]
fn membership_by_masks_equals_membership_by_table_and_by_enumeration_on_every_vector_of_sampled_one_disk_streams() {
    let specification = cell_specifications().into_iter().find(|candidate| candidate.label == "g3_free").expect("这个格存在");
    let memory = evaluate_cell(&specification);
    let table = &memory.world.table;
    let table_summary = WriteTableSummary::new(table);
    for index in 0..table_summary.total_vectors {
        assert_eq!(table_summary.index_of_code(table_summary.code_of_index(index)), index);
    }
    for stream in memory.world.streams.iter().step_by(53) {
        let segments = write_segments(table, stream);
        let masks: Vec<u32> = segments.iter().map(|segment| segment.iter().fold(0u32, |mask, &write| mask | (1u32 << write))).collect();
        let digit_masks = segment_digit_masks(&masks);
        let enumerated = crash_states_of_segments(table, &segments);
        for state in all_state_vectors(table) {
            let by_masks = code_in_digit_masks(&digit_masks, u64::try_from(state).expect("装得进 u64"));
            assert_eq!(by_masks, state_is_in_crash_states_of_segments(&segments, state));
            assert_eq!(by_masks, contains(&enumerated, state));
        }
    }
}

/// 两盘一单元（每个事务 7 次写、两条链各 13 个事件）的格：T_exact（184）远小于 T（82944），T_exact 过滤那一半在这里才敏感。
/// 数与 `research/results/e167-multi-inflight-crash-cut-python-l11-two-disk-recount-2026-10-01.py`（独立 Python，只按定义写，原样输出是同名 `-a-ordered.out`、`-b-ordered.out`、`-a-free.out`）逐个相等；
/// Python 的「段签名数」把段内写序也算进签名，与装置的签名（段内按集合）不同口径，所以不钉。
fn check_two_disk_one_unit_cell(candidate: Candidate, txg_of_the_older_transaction: usize, expected: (u128, usize, usize, usize, usize, u32, u32, u64, usize)) {
    let geometry = geometry_two_disks("two_disk_u1", 1, txg_of_the_older_transaction);
    let specification = CellSpecification { label: "two_disk_u1", sampling_point: "two_disk_u1", geometry, candidate };
    let streaming = evaluate_streaming_cell(&specification, 3);
    let read = read_root_slot_cut_true_cross(&streaming, 3);
    assert!(read.methods_agree && read.serial_agrees_across_methods);
    let cross_bound_maximum = streaming.rows.iter().map(|row| streaming.truth_cross - row.cross_states as usize).max().expect("至少一个段签名");
    let observed = (
        streaming.expected_leaves,
        streaming.truth.len(),
        streaming.exact_reachable.len(),
        streaming.truth_cross,
        read.target.len(),
        read.minimum,
        read.maximum,
        read.streams_with_misses,
        read.serial_misses,
    );
    assert_eq!(observed, expected);
    assert!(read.maximum as usize <= cross_bound_maximum);
    let table = &streaming.world.table;
    for (row, &miss) in streaming.rows.iter().zip(&read.misses).step_by(97) {
        let segments: Vec<Vec<usize>> = segments_of_signature(row.key, streaming.table_summary.write_count)
            .into_iter()
            .map(|mask| (0..streaming.table_summary.write_count).filter(|write| mask & (1u32 << write) != 0).collect())
            .collect();
        let states = crash_states_of_segments(table, &segments);
        assert_eq!(miss as usize, read.target.iter().filter(|&&code| !contains(&states, StateCode::from(code))).count());
    }
}

#[test]
fn missed_states_on_the_two_disk_one_unit_ordered_cell_equal_the_independent_python_recount() {
    // 根落同一块盘以外的一面（A 的 txg 取余 0：RA 落盘 0、RB 落盘 1）。
    // （交错数，|T|，|T_exact|，|T ∩ C|，|T ∩ T_exact ∩ C|，L11 最小，最大，有漏流数，σ_ser 上的值）
    check_two_disk_one_unit_cell(Candidate::ManyPublishesInFlightOrdered, 3, (2_431_135, 82_944, 184, 82_369, 165, 0, 150, 2_431_067, 150));
    // 两个根都落盘 0（A 的 txg 取余 2：RA、RB 都落盘 0）：有漏的流多一些（只有 34 条流一个都不漏）。
    check_two_disk_one_unit_cell(Candidate::ManyPublishesInFlightOrdered, 2, (2_431_135, 82_944, 184, 82_369, 165, 0, 150, 2_431_101, 150));
}

#[test]
fn missed_states_on_the_two_disk_one_unit_free_cell_equal_the_independent_python_recount() {
    // 自由交错 10400600 条流，T_exact 256、T ∩ T_exact ∩ C 为 225，只有 103 条流一个都不漏。
    check_two_disk_one_unit_cell(Candidate::ManyPublishesInFlightFree, 3, (10_400_600, 82_944, 256, 82_369, 225, 0, 210, 10_400_497, 210));
}

/// 一盘四格的 L11：与 `research/results/e167-multi-inflight-crash-cut-python-l11-recount-2026-10-01.py` 的输出逐个相等（Python 只按定义算，不读这份装置）。
/// 一盘上 T_exact = T，所以这几个数同时等于第一段 L2 的最小、最大（27 / 42、13 / 28、11 / 20、5 / 14）。
#[test]
fn missed_states_on_the_one_disk_cells_equal_the_independent_python_recount_and_the_in_memory_brute_force() {
    // （格，目标集大小，最小，最大，有漏的流数，串行流上的漏数）
    for (label, target_size, minimum, maximum, streams_with_misses, serial_misses) in [
        ("g1_free", 49usize, 27u32, 42u32, 12_870u64, 42usize),
        ("g1_ordered", 35, 13, 28, 3_747, 28),
        ("g3_free", 25, 11, 20, 3_432, 20),
        ("g3_ordered", 19, 5, 14, 973, 14),
    ] {
        let specification = cell_specifications().into_iter().find(|candidate| candidate.label == label).expect("这个格存在");
        let streaming = evaluate_streaming_cell(&specification, 3);
        let read = read_root_slot_cut_true_cross(&streaming, 3);
        assert!(read.methods_agree && read.serial_agrees_across_methods, "{label}");
        assert_eq!((read.target.len(), read.minimum, read.maximum, read.streams_with_misses, read.serial_misses), (target_size, minimum, maximum, streams_with_misses, serial_misses), "{label}");
        // 内存路径：逐条发出流枚举，数目标集里没落进它的状态。
        let memory = evaluate_cell(&specification);
        let table = &memory.world.table;
        let memory_target = true_cross_target(table, &memory.truth, &memory.exact_reachable);
        assert_eq!(memory_target, read.target, "{label}");
        let mut by_value: std::collections::BTreeMap<usize, u64> = std::collections::BTreeMap::new();
        let mut per_stream: Vec<usize> = Vec::new();
        for stream in &memory.world.streams {
            let states = crash_states_of_stream(table, stream);
            let misses = memory_target.iter().filter(|&&code| !contains(&states, StateCode::from(code))).count();
            *by_value.entry(misses).or_insert(0) += 1;
            per_stream.push(misses);
        }
        // 取到最小、最大的那条流：内存路径取字典序最先的，流式取代表路径最小的，两边渲染出来的流必须是同一条。
        let first_with = |value: usize| per_stream.iter().position(|&misses| misses == value).expect("有取到这个值的流");
        assert_eq!(render_path(&streaming.world, streaming.rows[read.minimum_row].representative_path), render_stream(table, &memory.world.streams[first_with(minimum as usize)]), "{label} 最小");
        assert_eq!(render_path(&streaming.world, streaming.rows[read.maximum_row].representative_path), render_stream(table, &memory.world.streams[first_with(maximum as usize)]), "{label} 最大");
        let mut streaming_by_value: std::collections::BTreeMap<usize, u64> = std::collections::BTreeMap::new();
        for (row, &miss) in streaming.rows.iter().zip(&read.misses) {
            *streaming_by_value.entry(miss as usize).or_insert(0) += row.count;
        }
        assert_eq!(by_value, streaming_by_value, "{label}");
    }
}

#[test]
fn the_third_segment_cell_passes_its_own_self_checks_and_reports_the_pinned_judgement_on_the_one_disk_cells() {
    let mut judgements = Vec::new();
    let mut tally = RunTally::default();
    for label in ["g1_free", "g1_ordered", "g3_free", "g3_ordered"] {
        let specification = cell_specifications().into_iter().find(|candidate| candidate.label == label).expect("这个格存在");
        let streaming = evaluate_streaming_cell(&specification, 2);
        judgements.push(emit_true_cross_cell(&mut Emitter::new(), &mut tally, &streaming, 2));
    }
    // 两盘一单元有序格：T_exact 比 T 小得多，L11 低于 L2（不高于 L2 那条自检在这里才不是同值）。
    let two_disk = CellSpecification { label: "two_disk_u1", sampling_point: "two_disk_u1", geometry: two_disk_one_unit_geometry(), candidate: Candidate::ManyPublishesInFlightOrdered };
    judgements.push(emit_true_cross_cell(&mut Emitter::new(), &mut tally, &evaluate_streaming_cell(&two_disk, 2), 2));
    emit_true_cross_threshold_shifts(&mut Emitter::new(), &mut tally, &judgements);
    assert!(tally.void_conditions.is_empty(), "{:?}", tally.void_conditions);
    // 每格：两个阳性对照各 1、两条做法一致 1、串行流两种判法一致 1、不高于 L2 1、真值两算法 1、交错数 1、L11 门槛挪动 1。
    assert_eq!(tally.self_checks_passed, 5 * 8);
    let observed: Vec<(usize, usize, u64)> = judgements.iter().map(|judgement| (judgement.minimum, judgement.maximum, judgement.streams_with_misses)).collect();
    assert_eq!(observed, vec![(27, 42, 12_870), (13, 28, 3_747), (11, 20, 3_432), (5, 14, 973), (0, 150, 2_431_067)]);
    // 一盘上 K2 的积包住 T（第四节第 6 条），所以一盘四格的 L3b 全是 0。
    assert!(judgements[..4].iter().all(|judgement| judgement.arm_by_transaction_true_cross_misses == 0));
}

/// 不带参数三段都跑（第一、二、三段依次）；段名只跑那一段；认不出的参数不选任何一段。
#[test]
fn no_argument_selects_all_three_segments_and_a_segment_name_selects_one() {
    assert_eq!(segments_for_argument(None), Some(vec![SegmentChoice::First, SegmentChoice::Second, SegmentChoice::Third]));
    assert_eq!(segments_for_argument(Some("first")), Some(vec![SegmentChoice::First]));
    assert_eq!(segments_for_argument(Some("second")), Some(vec![SegmentChoice::Second]));
    assert_eq!(segments_for_argument(Some("third")), Some(vec![SegmentChoice::Third]));
    assert_eq!(segments_for_argument(Some("halt")), Some(vec![SegmentChoice::HaltChecksOnly]));
    assert_eq!(segments_for_argument(Some("s3")), Some(vec![SegmentChoice::ScaleEstimateOnly]));
    assert_eq!(segments_for_argument(Some("never-used-segment-name")), None);
}

/// 并进来的段把不通过的条件与通过的自检数都带上。
#[test]
fn absorbing_a_segment_keeps_its_conditions_and_self_check_count() {
    let mut first = RunTally::default();
    first.self_check(true, "first-void");
    first.self_check(false, "first-void");
    let mut second = RunTally::default();
    second.self_check(true, "second-void");
    second.halt_conditions.push("second-halt".to_string());
    first.absorb(second);
    assert_eq!(first.self_checks_passed, 2);
    assert_eq!(first.void_conditions, vec!["first-void".to_string()]);
    assert_eq!(first.halt_conditions, vec!["second-halt".to_string()]);
}
