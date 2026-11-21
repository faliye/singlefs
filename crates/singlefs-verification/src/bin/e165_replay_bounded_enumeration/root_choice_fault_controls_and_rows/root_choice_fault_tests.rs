use super::*;
use crate::root_choice_fault_tests::cell_of;

#[test]
fn the_promised_rows_subtract_the_states_the_in_flight_limit_cut() {
    let mut cells = [FaultCellTally::default(); FAULT_CELL_COUNT];
    let covered_without_a_judged_fault = fault_cell_index_of(JudgedFault::Nothing, true);
    cells[covered_without_a_judged_fault].violations[KJ1_INDEX] = 3;
    cells[covered_without_a_judged_fault].cut_violations[KJ1_INDEX] = 3;
    let device_gone = fault_cell_index_of(JudgedFault::DeviceZeroGone, true);
    cells[device_gone].violations[KJ1_INDEX] = 5;
    cells[device_gone].cut_violations[KJ1_INDEX] = 1;
    let instance_roots_gone = fault_cell_index_of(JudgedFault::NewestInstanceRoots, true);
    cells[instance_roots_gone].violations[KJ1_INDEX] = 7;
    assert_eq!(
        promised_violations_from_the_split_rows(&cells, KJ1_INDEX),
        4,
        "罩着的格 3 − 3 加 5 − 1；被判时 RI 不罩，不进承诺内"
    );
}

#[test]
fn the_local_choice_rules_pick_as_the_registered_comparisons_on_the_adversarial_rings() {
    for (rule, expected) in PINNED_KL_KJ2_OF_THE_LOCAL_RULES {
        assert_eq!(
            kl_kj2_violations_of_a_local_rule(rule),
            expected,
            "{}",
            rule.label()
        );
    }
}

/// 每条判据在每个被判的格上各有对照读数、都红；读数条数照格数逐条判据数（删掉某条判据的某一行对照，条数对不上）。
#[test]
fn every_fault_criterion_has_a_positive_control_and_every_control_is_red() {
    let cells = [
        (FaultDomain::GateOn, cell_of(1, 0)),
        (FaultDomain::GateOn, cell_of(1, 2)),
        (FaultDomain::GateOff, cell_of(1, 2)),
    ];
    let mut readings = Vec::new();
    for (domain, cell) in cells {
        let tally = fault_tally_of_a_cell(domain, cell, true, 1);
        readings.extend(fault_control_readings_of_a_cell(domain, cell, &tally));
    }
    readings.extend(fault_kl_control_readings());
    for reading in &readings {
        assert!(reading.is_red_as_required, "对照没红：{reading:?}");
    }
    let readings_of = |criterion: FaultCriterion| {
        readings
            .iter()
            .filter(|reading| reading.criterion == criterion)
            .count()
    };
    assert_eq!(readings_of(FaultCriterion::Kj0Defined), 1);
    assert_eq!(
        readings_of(FaultCriterion::Kj1NotEarlierThanTheLastConfirmedPublish),
        3 * cells.len(),
        "KJ1 在每个格上 KF1、KF2、KF4 各一条对照"
    );
    assert_eq!(
        readings_of(FaultCriterion::Kj1tDescendsFromTheLastConfirmedPublish),
        3 * cells.len(),
        "KJ1T 在每个格上 KF1、KF2、KF4 各一条对照"
    );
    assert_eq!(
        readings_of(FaultCriterion::Kj2NotEarlierThanAnyReadableRoot),
        2 * cells.len() + PINNED_KL_KJ2_OF_THE_LOCAL_RULES.len(),
        "KJ2 在每个格上 KF1、KF2 各一条，在 KL 上每种本地择根序一条"
    );
    assert_eq!(
        readings_of(FaultCriterion::Kj3LandsOnTheIdealRecovery),
        cells.len(),
        "KJ3 在每个格上 KF2 一条对照"
    );
}
