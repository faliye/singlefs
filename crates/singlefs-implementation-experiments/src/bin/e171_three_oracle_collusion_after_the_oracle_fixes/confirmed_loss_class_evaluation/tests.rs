//! 确认丢失那一形的末段分类、四个输入的崩溃列与各类的判的单测。

use super::*;
// 经产品取样路径（`sample_crash_states_on_the_cpu`、给定状态引擎）开全仓那一份的用例开头拿它：同一个二进制里
// `pool_checker_holds_and_violations_through_the_given_states_engine` 设一次性库，不拿的在它设着时开到它那一份、它收尾停服之后连不上
use singlefs_verification::crash_library::TheOneLibraryKeptForThisProcess;

/// S0 里崩溃状态序号是 `ordinal` 的那一态在观测里的下标。
fn observation_index_of(fixture: &UntamperedPairing, ordinal: u64) -> usize {
    fixture
        .pairing
        .observations
        .iter()
        .position(|observation| observation.ordinal == ordinal)
        .expect("对拍逐态全判：每个序号都有观测")
}

/// U1（修订 5 第 5.4 节，KM2；V20：类核对 ① 不计数，这里红）：S0，代表 0 的观测里 O1-模型崩溃判官换掉，① 数到这一态、停在 S13。
#[test]
fn the_input_list_check_counts_a_state_whose_columns_do_not_follow_the_four_inputs() {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let mut fixture = untampered_pairing();
    let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
    let representative = fixture
        .pairing
        .classification
        .partition
        .representative_ordinal(&digits, 0);
    let index = observation_index_of(&fixture, representative);
    let tampered = match fixture.pairing.observations[index].crash_judge {
        CrashJudgeVerdict::Green => CrashJudgeVerdict::Red {
            what: "tampered".to_string(),
        },
        CrashJudgeVerdict::NotApplicable
        | CrashJudgeVerdict::Red { .. }
        | CrashJudgeVerdict::Off
        | CrashJudgeVerdict::RecoveryPanicked
        | CrashJudgeVerdict::JudgePanicked => CrashJudgeVerdict::Green,
    };
    fixture.pairing.observations[index].crash_judge = tampered;
    let checks = class_checks_of(&fixture);
    assert_eq!(checks.input_list_mismatches, 1, "① 数到代表 0 那一态");
    assert!(equivalence_stops_at_the_class_check(
        &fixture.pairing.classification,
        &checks
    ));
}

/// U2（修订 5 第 5.4 节，KM1；V21：类核对 ② 不计数，这里红）：c1 那条操作的盘与偏移改成第 2 次调用末段里第一个 journal 记录写的，
/// 写表不动（③ 看不见它）：② 数到它、停在 S13。
#[test]
fn an_in_flight_write_classified_as_a_journal_record_is_a_forbidden_kind() {
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
    let (first_in_flight_write, _) = first_in_flight_write_of(&stream);
    let journal_record_operation = stream.operations[stream.stream_indexes[journal_record_write]]
        .operation
        .clone();
    let in_flight_operation_index = stream.stream_indexes[first_in_flight_write];
    stream.operations[in_flight_operation_index]
        .operation
        .device = journal_record_operation.device;
    stream.operations[in_flight_operation_index]
        .operation
        .offset = journal_record_operation.offset;
    assert_eq!(
        experiment_pool::geometry()
            .classify(&stream.operations[in_flight_operation_index].operation),
        StepKind::JournalRecord,
        "改过的那条操作经 geometry.classify 是 journal 记录写"
    );
    let pairing = equivalence_pairing_of(&stream);
    assert_eq!(pairing.checks.in_flight_writes_of_a_forbidden_kind, 1);
    assert_eq!(
        pairing.checks.in_flight_write_kinds,
        BTreeMap::from([("journal_record", 1), ("unit_write", 1)])
    );
    assert_eq!(
        pairing.classification.refinements, 0,
        "写表没动，③ 看不见它，只有 ② 抓"
    );
    assert!(equivalence_stops_at_the_class_check(
        &pairing.classification,
        &pairing.checks
    ));
}

/// U5（修订 5 第 5.4 节，KM3；V24：类核对 ④ 后一半不计数，这里红）：S0，全落 0 的观测里 RefFS 一票换掉，④ 后一半数到它（① 同时数到）。
#[test]
fn fully_landed_state_that_differs_from_its_derived_class_is_counted() {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let mut fixture = untampered_pairing();
    let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
    let fully_landed = fixture
        .pairing
        .classification
        .partition
        .fully_landed_ordinal(&digits, 0);
    let index = observation_index_of(&fixture, fully_landed);
    let tampered = match fixture.pairing.observations[index].reference_filesystem {
        ReferenceFileSystemVoteOfAState::RecoveryPanicked => {
            ReferenceFileSystemVoteOfAState::Green {
                allowed: String::new(),
            }
        }
        ReferenceFileSystemVoteOfAState::Green { .. }
        | ReferenceFileSystemVoteOfAState::Red { .. }
        | ReferenceFileSystemVoteOfAState::Abstained { .. }
        | ReferenceFileSystemVoteOfAState::JudgePanicked
        | ReferenceFileSystemVoteOfAState::Off => ReferenceFileSystemVoteOfAState::RecoveryPanicked,
    };
    fixture.pairing.observations[index].reference_filesystem = tampered;
    let checks = class_checks_of(&fixture);
    assert_eq!(checks.derived_class_mismatches, 1, "④ 后一半数到全落 0");
    assert_eq!(checks.input_list_mismatches, 1, "① 同时数到这一态");
}

/// U6（修订 5 第 5.4 节，KM5、KM6；V25、V26：代表、全落那一态的落地串核对恒过，V27：S13 不看落地串核对，这里红）：S0，
/// (a) 代表 0 的落地串里 c1 那一位改成 `2`；(b) 还原后全落 0 的落地串里 c1 那一位改成 `0`。各自只有落地串核对数到 1、停在 S13。
#[test]
fn landing_string_off_the_partition_is_counted_and_stops_at_the_class_check() {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let mut fixture = untampered_pairing();
    let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
    let (first_in_flight_write, _) = first_in_flight_write_of(&fixture.stream);
    let representative = fixture
        .pairing
        .classification
        .partition
        .representative_ordinal(&digits, 0);
    let fully_landed = fixture
        .pairing
        .classification
        .partition
        .fully_landed_ordinal(&digits, 0);
    for (ordinal, tampered_landing) in [(representative, "2"), (fully_landed, "0")] {
        let index = observation_index_of(&fixture, ordinal);
        let original = fixture.pairing.observations[index].landing_vector.clone();
        fixture.pairing.observations[index]
            .landing_vector
            .replace_range(
                first_in_flight_write..first_in_flight_write + 1,
                tampered_landing,
            );
        let checks = class_checks_of(&fixture);
        assert_eq!(
            five_class_check_counts(&checks),
            [0, 0, 0, 0, 1],
            "序号 {ordinal} 的 c1 那一位改成 {tampered_landing}：只有落地串核对数到它"
        );
        assert!(
            equivalence_stops_at_the_class_check(&fixture.pairing.classification, &checks),
            "S13 只能从落地串核对来"
        );
        fixture.pairing.observations[index].landing_vector = original;
    }
}
