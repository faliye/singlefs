use super::*;

// ── 择根与重放合判一族（登记 r3 5.3 至 5.7、9.2 乙）

#[test]
fn root_choice_local_constants_equal_the_implementation_and_the_first_mount_of_a_new_pool_pushes_two_overhead_publishes(
) {
    cross_check_root_choice_local_constants_against_the_implementation();
}

#[test]
fn the_root_choice_state_counts_of_the_smallest_cells_equal_the_pinned_values_and_the_kw_wrap_cells_read_four_and_one(
) {
    let thread_count = thread_count_from_the_environment();
    for (domain, mounts, user_publishes) in [
        (RootChoiceDomain::BigRing, 1, 0),
        (RootChoiceDomain::BigRing, 1, 1),
        (RootChoiceDomain::BigRing, 1, 2),
        (RootChoiceDomain::BigRing, 2, 0),
        (RootChoiceDomain::BigRing, 2, 1),
        (RootChoiceDomain::SmallRingNine, 1, 0),
        (RootChoiceDomain::SmallRingNine, 2, 1),
        (RootChoiceDomain::WrappingRootRing, 1, 0),
        (RootChoiceDomain::WrappingRootRing, 1, 10),
        (RootChoiceDomain::WrappingRootRing, 1, 11),
    ] {
        let cell = RootChoiceCell {
            mounts,
            user_publishes,
        };
        let tally = root_choice_tally_of_a_cell(
            domain,
            cell,
            RootChoiceImplementation::TodayThreeSteps,
            thread_count,
        );
        assert_the_pinned_root_choice_cell(domain, cell, &tally);
        assert_eq!(
            tally.count_of(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
            0,
            "读槽对拍与真相一致：{domain:?} {cell:?}"
        );
        assert_eq!(tally.count_of(RootChoiceCounter::ChoseNothingStates), 0);
        assert_eq!(tally.count_of(RootChoiceCounter::PanickedStates), 0);
    }
}

/// KL 的形状过登记第七节乙类 B1 的六条钉值（TV4），TV10 只在「实例代号更高而 txg 更低同在」为 0 或「同 txg 不同实例同在」不为 0 时作废
/// （第 7 次重跑登记 5.6 H3、H4）。
#[test]
fn the_landing_slot_adversarial_domain_passes_its_pinned_shape_and_is_void_only_without_its_two_shapes(
) {
    let cell = RootChoiceCell {
        mounts: 0,
        user_publishes: 0,
    };
    let today = root_choice_tally_of_a_cell(
        RootChoiceDomain::AdversarialRootRing,
        cell,
        RootChoiceImplementation::TodayThreeSteps,
        1,
    );
    assert_the_pinned_landing_slot_adversarial_shape(&today);
    let void_conditions_of = |tally: &RootChoiceTally| {
        let mut emitter = Emitter { emitted: 0 };
        let mut findings = RunFindings::default();
        emit_the_verdict_of_a_root_choice_domain(
            &mut emitter,
            &mut findings,
            RootChoiceDomain::AdversarialRootRing,
            &[cell],
            tally,
        );
        findings.void_conditions.len()
    };
    assert_eq!(void_conditions_of(&today), 0, "照落点摆根的 KL 不作废");
    let mut with_two_instances_of_one_txg = today.clone();
    with_two_instances_of_one_txg.add(RootChoiceCounter::SameTxgDifferentInstanceRoots);
    assert_eq!(
        void_conditions_of(&with_two_instances_of_one_txg),
        1,
        "同 txg 不同实例同在不为 0：摆根没照落点，作废"
    );
    assert_eq!(
        void_conditions_of(&RootChoiceTally::new()),
        1,
        "实例代号更高而 txg 更低同在为 0：作废"
    );
}
