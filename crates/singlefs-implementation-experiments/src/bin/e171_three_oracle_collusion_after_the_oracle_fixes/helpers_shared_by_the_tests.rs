//! 几份单测共用的帮手。

use super::*;

pub(super) fn unit_text(_: &()) -> String {
    "()".to_string()
}

/// 单测里造一个说得出成员名的错误：`error_member_of` 取 `Debug` 文本的第一个标识符，`&str` 的 `Debug` 文本以引号起头、取不到。
#[derive(Debug)]
pub(super) enum RefusedForTheTest {
    DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness,
}

/// 内联用例的对拍流截到第 3 次调用发出的前几个写（修订 5 第 5.2 节：U1–U6 与已有的两条留在 k = 2，288 态；装置那一跑是
/// [`EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT`]）。
pub(super) const IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS: usize = 2;

/// 「等价类对拍」那一形的流（流二同一种造法），截到第 3 次调用发出的前 `in_flight_writes_kept` 个写之后。
pub(super) fn equivalence_pairing_stream_keeping(in_flight_writes_kept: usize) -> PreparedStream {
    confirmed_loss_prepared_stream(
        second_stream_through_the_first_overwrite(RunSwitches::default()),
        ConfirmedLossTruncation::AfterTheFirstInFlightWrites(in_flight_writes_kept),
    )
    .0
}

/// 「等价类对拍」那一形的流，截到第 3 次调用发出的前 2 个写之后（72 × 4 = 288 态）。
pub(super) fn equivalence_pairing_stream() -> PreparedStream {
    equivalence_pairing_stream_keeping(IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS)
}

/// 修订 5 第 5.4 节 U1、U4、U5、U6 的共同起点「S0」：k = 2 的对拍流上逐态全判 288 态、分好类（[`equivalence_pairing_of`]）。
pub(super) struct UntamperedPairing {
    pub(super) stream: PreparedStream,
    pub(super) tables: ObservationTables,
    pub(super) pairing: EquivalencePairing,
}

/// 类核对 ①、②、④ 两半、落地串核对五个计数（按这个次序）。
pub(super) fn five_class_check_counts(checks: &EquivalenceCheckCounts) -> [usize; 5] {
    [
        checks.input_list_mismatches,
        checks.in_flight_writes_of_a_forbidden_kind,
        checks.recording_report_mismatches,
        checks.derived_class_mismatches,
        checks.class_landing_mismatches,
    ]
}

/// 照 [`evaluate_the_classes`] 在 S0 的（可能篡改过的）观测与分法上重做类核对。
pub(super) fn class_checks_of(fixture: &UntamperedPairing) -> EquivalenceCheckCounts {
    let context = fixture.tables.context(&fixture.stream);
    let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
    evaluate_the_classes(
        &context,
        &digits,
        &fixture.pairing.classification,
        &fixture.pairing.observations,
        CONFIRMED_LOSS_IN_FLIGHT_CALL,
    )
    .checks
}

/// 造 S0，并断言不篡改时类核对五个计数全 0、不停在 S13（证明之后那一处篡改才是让它红的东西）。
pub(super) fn untampered_pairing() -> UntamperedPairing {
    let stream = equivalence_pairing_stream();
    let tables = ObservationTables::of(&stream);
    let pairing = equivalence_pairing_of(&stream);
    let fixture = UntamperedPairing {
        stream,
        tables,
        pairing,
    };
    let checks = class_checks_of(&fixture);
    assert_eq!(
        five_class_check_counts(&checks),
        [0; 5],
        "不篡改时类核对五个计数全 0"
    );
    assert!(
        !equivalence_stops_at_the_class_check(&fixture.pairing.classification, &checks),
        "不篡改时不停在 S13"
    );
    fixture
}

/// 末段里第一个属于第 3 次调用的写（c1）：写表下标与它在末段里的位置。
pub(super) fn first_in_flight_write_of(stream: &PreparedStream) -> (usize, usize) {
    let call_of_write = call_index_of_each_write(stream);
    let last_segment = stream.segments.last().cloned().expect("末段");
    let position = last_segment
        .iter()
        .position(|write_index| call_of_write[*write_index] == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL))
        .expect("末段里有第 3 次调用的写");
    (last_segment[position], position)
}
