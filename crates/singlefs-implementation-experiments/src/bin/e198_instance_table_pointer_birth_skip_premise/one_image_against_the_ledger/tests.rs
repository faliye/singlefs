use super::*;
use crate::helpers_shared_by_the_tests::{
    assert_the_history_is_consistent, key, ring_version, sweep_specification,
};

#[test]
fn each_arm_counts_e1_and_e2_exactly_on_the_four_kinds_of_pairs() {
    let pair = |bridged_today: bool, bridged_with_the_pointer_birth: bool| HolePair {
        previous_txg: 6,
        previous_group: vec![key(1, 6)],
        version: ring_version(2, 8, Some(7)),
        bridged_today,
        bridged_with_the_pointer_birth,
    };
    let counts = |today_e1, today_e2, with_pointer_birth_e1, with_pointer_birth_e2| ArmCounts {
        today_e1,
        today_e2,
        with_pointer_birth_e1,
        with_pointer_birth_e2,
    };
    assert_eq!(
        arm_counts_of_the_pair(&pair(true, true), Truth::ExistedVersionsInBetween),
        counts(1, 0, 1, 0),
        "两臂都把存在过的认成取号跳过：E1"
    );
    assert_eq!(
        arm_counts_of_the_pair(&pair(false, false), Truth::DirectlyBridged),
        counts(0, 1, 0, 1),
        "两臂都把取号跳过认成存在过：E2"
    );
    assert_eq!(
        arm_counts_of_the_pair(&pair(true, false), Truth::PreviousNotOnTheChain),
        counts(1, 0, 0, 0),
        "只有今天的臂认错"
    );
    assert_eq!(
        arm_counts_of_the_pair(&pair(true, true), Truth::DirectlyBridged),
        counts(0, 0, 0, 0),
        "两臂都认对取号跳过"
    );
    assert_eq!(
        arm_counts_of_the_pair(&pair(false, false), Truth::ExistedVersionsInBetween),
        counts(0, 0, 0, 0),
        "两臂都认对存在过"
    );
    assert_eq!(
        arm_counts_of_the_pair(&pair(true, true), Truth::VersionMissingFromTheLedger),
        counts(0, 0, 0, 0),
        "R 不在账本里的不计"
    );
}

/// 登记修订 3：sw-crash 里实例 4 的挂载盖掉实例 3 写行那一版起的几个根槽，那几版的记录还读得出、b 那一槽还住着实例 2 的 b，
/// 装置按 ①–④ 认它们由记录施加出来；账本收被盖掉的版本那一支收下它们，S5 两边相同。
/// 登记修订 4：实例 3 崩掉的那一版（根槽没落、被实例 4 的挂载施加）在环里最旧的有效根还比它老的那几张上装置认、账本收；
/// 环转过去、最旧的有效根比它新之后（k = 9 起）装置按 ④ 不认，账本那一支也要过 ④、不收，S5 两边仍相同。
#[test]
fn records_applied_comparison_takes_in_the_covered_versions_whose_records_are_still_readable() {
    let result = run_the_history(&sweep_specification(
        SweepVariant::CrashBeforeTheNextInstance,
    ));
    assert_the_history_is_consistent(&result, 13);
    let mut recognised_with_a_root_that_landed = 0;
    let crashed_versions: Vec<VersionKey> = result
        .ledger
        .versions
        .iter()
        .filter(|(_, version)| version.applied_by_a_mount && !version.root_landed)
        .map(|(key, _)| *key)
        .collect();
    assert!(
        !crashed_versions.is_empty(),
        "sw-crash 里有崩掉、被下一个实例的挂载施加的版本"
    );
    let mut images_recognising_a_crashed_version = 0;
    let mut images_not_recognising_any_crashed_version = 0;
    for image in &result.images {
        assert_eq!(
            image.records_applied.mismatch_count(),
            0,
            "S5：{} 装置认的与账本推的相同：{:?}",
            image.label,
            image.records_applied
        );
        let reading = image.reading.as_ref().expect("读得出");
        if reading
            .applied_only_by_records
            .iter()
            .any(|version| crashed_versions.contains(&version.key))
        {
            images_recognising_a_crashed_version += 1;
        } else {
            images_not_recognising_any_crashed_version += 1;
        }
        recognised_with_a_root_that_landed += reading
            .applied_only_by_records
            .iter()
            .filter(|version| {
                result
                    .ledger
                    .versions
                    .get(&version.key)
                    .is_some_and(|ledger_version| ledger_version.root_landed)
            })
            .count();
    }
    assert!(
        recognised_with_a_root_that_landed >= 1,
        "被环盖掉、根槽落过的版本里有装置按 ①–④ 认作由记录施加出来的"
    );
    assert!(
        images_recognising_a_crashed_version >= 1
            && images_not_recognising_any_crashed_version >= 1,
        "崩掉的那一版：环里最旧的有效根还比它老的张上认（{images_recognising_a_crashed_version} 张），环转过去之后不认（{images_not_recognising_any_crashed_version} 张）"
    );
}
