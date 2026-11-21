//! 一格的结果行（G1–G5、S5）的单测。

use super::*;

#[test]
fn constant_keys_separate_recovery_outcomes_and_verdicts_on_the_first_stream() {
    let (_stream, mut totals, small) =
        evaluate_the_first_stream_small_domain(2, TimingControl::WALL_CLOCK_WITHOUT_BUSY_WAIT);
    let stream = prepare_first_stream();
    let table = TornWriteTable::of(&stream);
    let plan = LocalPlan::new(
        &stream,
        &table,
        domain_expansions(&stream, LocalExpansion::EveryCombination),
    );
    let intervals = TrajectoryIntervals::of(&plan, &small, &BTreeSet::new());
    let stores = std::mem::take(&mut totals.arm_stores);
    let mut output = ResultLines::new();
    let summary = report_cell(
        &mut output,
        "test-first-small",
        &totals,
        stores,
        &intervals,
        None,
    );
    assert!(
        summary.arm_constant_key_inconsistent[Arm::RecoveryConsult.slot()] >= 1,
        "PC2b：键换成常数时看 journal 那一遍不一致数 ≥ 1"
    );
    assert!(
        summary.arm_constant_key_inconsistent[Arm::FullReadSet.slot()] >= 1,
        "PC4b：键换成常数时判定不一致数 ≥ 1"
    );
    assert_eq!(
        summary.arm_inconsistent[Arm::RecoveryConsult.slot()],
        0,
        "按读集复用在这 53 个状态上没有不一致"
    );
}
