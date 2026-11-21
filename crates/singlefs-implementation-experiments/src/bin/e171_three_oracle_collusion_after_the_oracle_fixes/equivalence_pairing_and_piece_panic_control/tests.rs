//! 等价配对与块内 panic 的对照的单测。

use super::*;
// 经产品取样路径（`sample_crash_states_on_the_cpu`、给定状态引擎）开全仓那一份的用例开头拿它：同一个二进制里
// `pool_checker_holds_and_violations_through_the_given_states_engine` 设一次性库，不拿的在它设着时开到它那一份、它收尾停服之后连不上
use singlefs_verification::crash_library::TheOneLibraryKeptForThisProcess;

/// U7（修订 5 第 5.7 节，第 5.1 节甲；V28：一件 panic 两列都记恢复 panic，V29：`pool_checker` 那一件 panic 不记件名，这里红）：
/// k = 4 的对拍流（1 152 态，c1…c4 两对镜像），注入 `pool_checker`（c3 落、c4 没落）、`ignored_recovery`（c3 没落、c4 落）、
/// `record_checker`（c1 没落、c2 落、(c3,c4) 同落或同不落）：各件的态数恰为 288、288、144，两个崩溃子列与看 journal 那遍恢复逐态不变。
#[test]
fn panic_off_the_consulted_recovery_leaves_the_two_crash_columns_to_the_four_inputs() {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let stream = equivalence_pairing_stream_keeping(4);
    let tables = ObservationTables::of(&stream);
    let facts = equivalence_pairing_shape_facts(&stream, &tables.tables);
    let [first, second, third, fourth] = facts.in_flight_writes[..] else {
        panic!(
            "k = 4 的对拍流末段里第 3 次调用恰有 4 个写：{:?}",
            facts.in_flight_writes
        );
    };
    let injection = PiecePanicInjection {
        rules: vec![
            PiecePanicInjectionRule {
                piece: Some(JudgingPiece::PoolChecker),
                conditions: vec![
                    InjectedLandingCondition::Landed(third),
                    InjectedLandingCondition::NotLanded(fourth),
                ],
            },
            PiecePanicInjectionRule {
                piece: Some(JudgingPiece::IgnoredRecovery),
                conditions: vec![
                    InjectedLandingCondition::NotLanded(third),
                    InjectedLandingCondition::Landed(fourth),
                ],
            },
            PiecePanicInjectionRule {
                piece: Some(JudgingPiece::RecordChecker),
                conditions: vec![
                    InjectedLandingCondition::NotLanded(first),
                    InjectedLandingCondition::Landed(second),
                    InjectedLandingCondition::BothOrNeitherLanded(third, fourth),
                ],
            },
        ],
    };
    let plain = equivalence_pairing_of(&stream);
    assert_eq!(
        piece_panic_states_of(&plain.observations),
        [0; JUDGING_PIECE_COUNT],
        "不注入那一遍各件 panic 都是 0（前提）"
    );
    let injected = equivalence_pairing_with_injected_piece_panics(&stream, Some(&injection));
    let two_choices_each_of_two_writes =
        usize::try_from(LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2).expect("288");
    let two_choices_of_one_write =
        usize::try_from(LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2).expect("144");
    let mut expected = [0usize; JUDGING_PIECE_COUNT];
    expected[JudgingPiece::PoolChecker.position()] = two_choices_each_of_two_writes;
    expected[JudgingPiece::IgnoredRecovery.position()] = two_choices_each_of_two_writes;
    expected[JudgingPiece::RecordChecker.position()] = two_choices_of_one_write;
    assert_eq!(
        piece_panic_states_of(&injected.observations),
        expected,
        "注入的三件各 288、288、144 态，别的件 0"
    );
    assert_eq!(
        crash_judge_columns_differing(&plain.observations, &injected.observations),
        0,
        "逐态两个崩溃子列与看 journal 那遍恢复的结局与不注入那一遍相同"
    );
    assert!(injected
        .observations
        .iter()
        .all(|observation| observation.recovery_panic.is_none()));
    assert_eq!(injected.mismatching_states, 0);
    assert!(!injected.stops_at_the_class_check());
    let pool_checker_panicking: Vec<&StateObservation> = injected
        .observations
        .iter()
        .filter(|observation| {
            observation
                .piece_panics
                .contains(&JudgingPiece::PoolChecker)
        })
        .collect();
    assert_eq!(pool_checker_panicking.len(), two_choices_each_of_two_writes);
    assert!(pool_checker_panicking.iter().all(|observation| {
        observation.pool_checker_red == Some(vec!["pool_checker_panic".to_string()])
    }));
}

/// U9（修订 5 第 5.7 节，第 5.2 节丙；V31：k 由 6 改回 2，这里红）：k = [`EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT`] 的对拍流，
/// 末段各写取法数之积 4 608、第 3 次调用的写 6 个、c1…c6 三对镜像（偏移互不相同）。只造末段的进制，不判态。
#[test]
fn the_pairing_keeps_the_first_three_mirrored_pairs_of_the_in_flight_call() {
    let stream = equivalence_pairing_stream_keeping(EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT);
    let tables = crash_state_enumeration_tables(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let facts = equivalence_pairing_shape_facts(&stream, &tables);
    let digits = LastSegmentDigits::of(&stream, &tables);
    assert_eq!(
        digits.product_of_the_radixes(&(0..digits.write_indexes.len()).collect::<Vec<usize>>()),
        LOCAL_EQUIVALENCE_PAIRING_STATES,
        "末段各写取法数之积是 72 × 2⁶"
    );
    assert_eq!(facts.states, LOCAL_EQUIVALENCE_PAIRING_STATES);
    assert_eq!(
        facts.in_flight_writes.len(),
        LOCAL_EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES,
        "末段里属于第 3 次调用的写 6 个"
    );
    assert_eq!(
        facts.mirrored_pairs, LOCAL_EQUIVALENCE_PAIRING_MIRRORED_PAIRS,
        "c1…c6 两两同偏移、同长度、分属两块盘，三对的偏移互不相同"
    );
}
