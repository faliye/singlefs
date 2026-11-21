//! 确认丢失那一形的观测表、末段的位、按类分区与恢复读过的范围的单测。

use super::*;
// 经产品取样路径（`sample_crash_states_on_the_cpu`、给定状态引擎）开全仓那一份的用例开头拿它：同一个二进制里
// `pool_checker_holds_and_violations_through_the_given_states_engine` 设一次性库，不拿的在它设着时开到它那一份、它收尾停服之后连不上
use singlefs_verification::crash_library::TheOneLibraryKeptForThisProcess;

/// 等价关系（登记修订 2 第 2.1 节候选二「用例钉住等价关系」）：288 态那一形上全枚举逐态判与按等价类推，每态的恢复报告、两个崩溃子列
/// 与 RefFS 一票逐态相同，类核对 ①–④ 都过；模型照常那一遍与 `model_stopped_at_call` 设成第 3 次调用那一遍各做一遍（V17：键去掉
/// 「C 至少落一个」那一位，第二遍里「C 落了」的态模型崩溃判官不适用、代表上适用，这里红）。各类态数之和等于末段每个写的取法数之积，
/// 主取样点上（不判态，只分类）是 72 × 2²⁴（V19：「C 至少落一个」那一类的态数写成 2 的 C 写数次方，这里红）。
#[test]
fn equivalence_classes_reproduce_every_state_with_and_without_the_model_stopped_at_the_in_flight_call(
) {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let mut stream = equivalence_pairing_stream();
    let pairing = equivalence_pairing_of(&stream);
    assert_eq!(pairing.states, 288, "72 × 2² 态全枚举");
    assert_eq!(
        pairing.class_states, pairing.product_of_the_choices,
        "各类态数之和等于末段每个写的取法数之积（A10）"
    );
    assert_eq!(pairing.classes, 144, "72 种取法 × C 落没落");
    assert_eq!(
        pairing.classification.refinements, 0,
        "C 的写不在代表的读段里"
    );
    assert_eq!(
        pairing.checks,
        EquivalenceCheckCounts {
            input_list_mismatches: 0,
            in_flight_writes_of_a_forbidden_kind: 0,
            recording_report_mismatches: 0,
            derived_class_mismatches: 0,
            class_landing_mismatches: 0,
            in_flight_write_kinds: BTreeMap::from([("unit_write", 2)]),
        },
        "类核对 ①、②、④ 都过"
    );
    assert_eq!(pairing.mismatching_states, 0, "模型照常：逐态相同");
    assert!(!pairing.stops_at_the_class_check());
    stream.model_stopped_at_call = Some(CONFIRMED_LOSS_IN_FLIGHT_CALL);
    let stopped = equivalence_pairing_of(&stream);
    let not_applicable_states = stopped
        .observations
        .iter()
        .filter(|observation| observation.crash_judge == CrashJudgeVerdict::NotApplicable)
        .count();
    assert_eq!(
        not_applicable_states,
        288 - 72,
        "模型停在第 3 次调用：C 落了的态（每种取法 3 态）模型崩溃判官不适用，代表上适用（取样点敏感）"
    );
    assert_eq!(
        stopped.mismatching_states, 0,
        "模型停在第 3 次调用：逐态相同"
    );
    assert!(!stopped.stops_at_the_class_check());
    let (main, _) = confirmed_loss_prepared_stream(
        second_stream_through_the_first_overwrite(RunSwitches::default()),
        ConfirmedLossTruncation::BeforeTheInFlightBarrier,
    );
    let main_tables = ObservationTables::of(&main);
    let main_context = main_tables.context(&main);
    let main_digits = LastSegmentDigits::of(&main, &main_tables.tables);
    let (class_positions, keyed_positions): (Vec<usize>, Vec<usize>) =
        (0..main_digits.write_indexes.len()).partition(|position| {
            main_context.call_of_write[main_digits.write_indexes[*position]]
                == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL)
        });
    let main_partition = ConfirmedLossPartition {
        keyed_positions,
        class_positions,
    };
    assert_eq!(
        main_partition.keyed_assignments(&main_digits)
            * (1 + main_partition.states_with_the_in_flight_writes_landed(&main_digits)),
        LOCAL_CONFIRMED_LOSS_STATES,
        "主取样点各类态数之和是 72 × 2²⁴"
    );
}

/// 类核对 ③ 细分（登记修订 2 第 2.1 节；V18：③ 恒判不相交，这里红）：288 态那一形上把第 3 次调用的第 1 个写挪到第 2 次调用第一个
/// journal 记录写的同一个落点（手造写流），代表上的恢复读到它的字节段，那个写移进键（细分一次）；那个写落没落，恢复有不同的态
/// （取样点敏感）；细分之后按等价类推的与全枚举逐态相同。
#[test]
fn an_in_flight_write_over_a_journal_record_of_the_confirmed_call_is_moved_into_the_key() {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let mut stream = equivalence_pairing_stream();
    let call_of_write = call_index_of_each_write(&stream);
    let last_segment = stream.segments.last().cloned().expect("末段");
    let journal_record_write = last_segment
        .iter()
        .copied()
        .find(|write_index| {
            call_of_write[*write_index] == Some(CONFIRMED_LOSS_CONFIRMED_CALL)
                && stream.writes[*write_index].kind == StepKind::JournalRecord
        })
        .expect("第 2 次调用在末段里写 journal 记录");
    let first_in_flight_position = last_segment
        .iter()
        .position(|write_index| call_of_write[*write_index] == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL))
        .expect("末段里有第 3 次调用的写");
    let first_in_flight_write = last_segment[first_in_flight_position];
    stream.writes[first_in_flight_write].device = stream.writes[journal_record_write].device;
    stream.writes[first_in_flight_write].offset = stream.writes[journal_record_write].offset;
    let pairing = equivalence_pairing_of(&stream);
    assert_eq!(pairing.states, 288);
    assert_eq!(
        pairing.classification.moved_positions,
        vec![first_in_flight_position],
        "压在 journal 记录上的那个写移进键"
    );
    assert_eq!(pairing.classification.refinements, 1);
    let tables = ObservationTables::of(&stream);
    let digits = LastSegmentDigits::of(&stream, &tables.tables);
    let weight = digits.weights[first_in_flight_position];
    let observation_of_ordinal: BTreeMap<u64, &StateObservation> = pairing
        .observations
        .iter()
        .map(|observation| (observation.ordinal, observation))
        .collect();
    let differing_when_that_write_lands = pairing
        .observations
        .iter()
        .filter(|observation| digits.digit_of(observation.ordinal, first_in_flight_position) == 1)
        .filter(|observation| {
            observation_of_ordinal[&(observation.ordinal - weight)].consulted_report
                != observation.consulted_report
        })
        .count();
    assert!(
        differing_when_that_write_lands > 0,
        "那个写落没落，有恢复不同的态"
    );
    assert_eq!(pairing.mismatching_states, 0, "细分之后逐态相同");
    assert!(!pairing.stops_at_the_class_check());
}

/// 与写对齐的粒度（U3 把 c1 挪到环尾时按它向下对齐）。
const WRITE_ALIGNMENT_IN_BYTES: u64 = 4096;

/// U3（修订 5 第 5.4 节，KM7；V22：journal 提示不记整环，这里红）：c1 挪到盘 0 journal 环尾、只有提示那一段罩得到的位置，
/// ③ 经提示那一段把它移进键（细分一次），细分之后逐态相同。
#[test]
fn an_in_flight_write_in_an_unread_journal_ring_slot_is_moved_into_the_key_through_the_hint() {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let mut stream = equivalence_pairing_stream();
    let (first_in_flight_write, first_in_flight_position) = first_in_flight_write_of(&stream);
    let length = stream.writes[first_in_flight_write].length_in_bytes();
    let (device_zero, landing_offset) = {
        let tables = ObservationTables::of(&stream);
        let context = tables.context(&stream);
        let digits = LastSegmentDigits::of(&stream, &tables.tables);
        let classification = classify_the_last_segment(
            &context,
            &digits,
            CONFIRMED_LOSS_IN_FLIGHT_CALL,
            CONFIRMED_LOSS_STATE_LIMIT,
        );
        let reads_of_the_representative = |keyed_assignment: u64| {
            let image = CrashImage {
                base: &stream.base,
                writes: &tables.tables.enumerated_writes,
                persisted: tables.facts.persisted_writes_of_state(
                    classification
                        .partition
                        .representative_ordinal(&digits, keyed_assignment),
                ),
            };
            consulted_recovery_recording_the_reads(&image).1
        };
        let ring_hint = reads_of_the_representative(0)
            .into_iter()
            .filter(|read| read.source == RecordedReadSource::JournalRecordOffsetsHint)
            .min_by_key(|read| read.device.0)
            .expect("代表 0 的恢复经提示记了 journal 环");
        let landing_offset =
            (ring_hint.end_in_bytes - length) / WRITE_ALIGNMENT_IN_BYTES * WRITE_ALIGNMENT_IN_BYTES;
        for keyed_assignment in 0..classification.partition.keyed_assignments(&digits) {
            assert!(
                reads_of_the_representative(keyed_assignment)
                    .iter()
                    .filter(|read| read.source == RecordedReadSource::Read
                        && read.device == ring_hint.device)
                    .all(|read| read.end_in_bytes <= landing_offset
                        || landing_offset + length <= read.start_in_bytes),
                "前提：第 {keyed_assignment} 种取法的代表上来自 read 的段不罩 [{landing_offset}, {})，只有提示那一段罩得到",
                landing_offset + length
            );
        }
        (ring_hint.device, landing_offset)
    };
    stream.writes[first_in_flight_write].device = device_zero;
    stream.writes[first_in_flight_write].offset = DeviceOffsetInBytes(landing_offset);
    let pairing = equivalence_pairing_of(&stream);
    assert_eq!(
        pairing.classification.moved_positions,
        vec![first_in_flight_position],
        "环尾那个写经提示那一段移进键"
    );
    assert_eq!(pairing.classification.refinements, 1);
    assert_eq!(pairing.mismatching_states, 0, "细分之后逐态相同");
}

/// U4（修订 5 第 5.4 节，KM4；V23：类核对 ④ 前一半不计数，这里红）：S0，代表 0 的记读那一遍恢复换成 panic，④ 前一半数到它。
#[test]
fn recording_recovery_that_differs_from_the_crash_point_replay_report_is_counted() {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let mut fixture = untampered_pairing();
    let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
    let representative = fixture
        .pairing
        .classification
        .partition
        .representative_ordinal(&digits, 0);
    fixture
        .pairing
        .classification
        .representative_recoveries
        .insert(
            representative,
            ConsultedRecovery::Panicked("tampered".to_string()),
        );
    let checks = class_checks_of(&fixture);
    assert_eq!(checks.recording_report_mismatches, 1, "④ 前一半数到代表 0");
    assert!(equivalence_stops_at_the_class_check(
        &fixture.pairing.classification,
        &checks
    ));
}
