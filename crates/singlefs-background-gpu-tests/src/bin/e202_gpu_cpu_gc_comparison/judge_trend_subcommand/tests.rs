use super::*;

/// 场景：点名一张卡臂那一组（甲）两小档有 saving_per_batch、顶档在另一组（乙）；另一组（丙）三档都有。
/// 预期：甲、乙三档凑不齐同一组，各打 `trend=not_judged` 与它有几档，不跨组拼三档；丙判出一个走向。
#[test]
fn judge_trend_does_not_join_the_three_tiers_across_hardware_groups() {
    let against = "gpu_one_card".to_string();
    let mut savings: SavingsPerBatchByArmGroupAndTier = BTreeMap::new();
    for (group, tier, value) in [
        ("aaaa", UnitAreaTier::EVERY_TIER[0], 1.0),
        ("aaaa", UnitAreaTier::EVERY_TIER[1], 2.0),
        ("bbbb", UnitAreaTier::EVERY_TIER[2], 3.0),
        ("cccc", UnitAreaTier::EVERY_TIER[0], 1.0),
        ("cccc", UnitAreaTier::EVERY_TIER[1], 2.0),
        ("cccc", UnitAreaTier::EVERY_TIER[2], 3.0),
    ] {
        savings.insert((against.clone(), group.to_string(), tier), vec![value; 5]);
    }
    let lines = trend_lines_by_arm_and_group(&savings);
    assert_eq!(
        lines[..2],
        [
            "against=gpu_one_card group=aaaa tiers_with_a_saving=2 trend=not_judged".to_string(),
            "against=gpu_one_card group=bbbb tiers_with_a_saving=1 trend=not_judged".to_string(),
        ]
    );
    assert!(
        lines.len() == 3
            && lines[2].starts_with("against=gpu_one_card group=cccc tiers_with_a_saving=3 trend=")
            && !lines[2].ends_with("trend=not_judged"),
        "{lines:?}"
    );
}
