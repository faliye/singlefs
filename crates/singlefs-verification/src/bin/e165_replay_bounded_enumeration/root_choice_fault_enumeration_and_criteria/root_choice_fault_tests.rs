use super::*;

#[test]
fn root_slot_reading_of_a_slot_unreadable_in_the_view_agrees_with_the_truth_only_as_unreadable() {
    let material = mkfs_root_material();
    let unreadable = RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable);
    assert!(fault_slot_reading_agrees_with_the_truth(
        unreadable,
        Some(&material),
        true
    ));
    assert!(!fault_slot_reading_agrees_with_the_truth(
        unreadable,
        Some(&material),
        false
    ));
    let zeros = RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified);
    assert!(!fault_slot_reading_agrees_with_the_truth(zeros, None, true));
    assert!(fault_slot_reading_agrees_with_the_truth(zeros, None, false));
}

/// 登记 9.2 里「一次挂载第一次发布的父取最后一次已确认那一版」与「KJ1T 写成按版本序比」两条变异的取样点：KG 的 M = 2、U = 3 里，挂载 2 开头最新实例的全部根读不出（RI-T1）、
/// 而挂载 1 有已确认发布的那些单元。挂载 2 往下走的版本比最后一次已确认的更旧，它的发布不在已确认那一版的后代上，KJ1T 与 KJ1 在那里分得开。
#[test]
fn the_confirmed_chain_parts_kj1t_from_kj1_when_a_mount_goes_down_from_an_older_version_on_the_gate_off_domain(
) {
    let domain = FaultDomain::GateOff;
    let collecting = FaultEnumeration {
        domain,
        mounts: 2,
        stop_before_mount: Some(2),
    };
    let mut collector = FaultUnitCollector { units: Vec::new() };
    collecting.enumerate_a_mount(
        &FaultWorld::right_after_mkfs(domain.geometry()),
        1,
        3,
        None,
        &mut collector,
    );
    let continuing = FaultEnumeration {
        domain,
        mounts: 2,
        stop_before_mount: None,
    };
    let judgement = FaultJudgement::new(domain, 2, false);
    let first_menu_index_of_ri = MOUNT_FAULT_MENU_LABELS
        .iter()
        .position(|label| *label == "RI-T1")
        .expect("RI-T1 在菜单里");
    let mut tally = FaultTally::new();
    for unit in collector.units.iter().filter(|unit| {
        !unit.world.confirmed_user_publishes.is_empty() && unit.menu_index == first_menu_index_of_ri
    }) {
        let mut judge = FaultStateJudge {
            judgement: &judgement,
            tally: FaultTally::new(),
            histories_seen_in_this_unit: 0,
            is_the_first_unit: false,
        };
        continuing.enumerate_a_mount(
            &unit.world,
            unit.mount_number,
            unit.user_publishes_left,
            Some(unit.menu_index),
            &mut judge,
        );
        tally.merge(&judge.tally);
    }
    assert!(
        tally.confirmed_states > 0,
        "这些单元的最终态都有已确认的发布"
    );
    assert_eq!(
        tally.kf4[1], tally.confirmed_states,
        "KF4 落在不在已确认那条链上的根里：每个有已确认的态 KJ1T 都违例"
    );
    assert!(
        tally.kf4[0] < tally.kf4[1],
        "挂载 2 的根（版本比已确认的那一版新）不在它的后代上：KJ1 不违例而 KJ1T 违例，两条分得开（KJ1 {}、KJ1T {}）",
        tally.kf4[0],
        tally.kf4[1]
    );
}
