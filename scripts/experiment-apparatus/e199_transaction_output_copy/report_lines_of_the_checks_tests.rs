use super::*;

#[test]
fn the_artifact_constant_checks_all_hold() {
    let (_, holds) = constant_check_lines();
    assert_eq!(holds, [true, true, true]);
}

#[test]
fn positive_control_of_jia_compares_a_million_with_a_hundred_thousand_records() {
    let lower = main_with(|knobs| knobs.tombstone_records = 100_000);
    assert_eq!((jia_before(&lower), jia_after(&lower)), (41402308, 8452936));
    let (lines, fields) = positive_control_lines(&sampling_points());
    assert!(lines.iter().any(|line| line.starts_with("name=positive_control arm=jia_after_patch_copied_before tombstone_records_1e5=41402308 tombstone_records_1e6=404072244 ratio=9.7597")));
    let holding: Vec<&str> = fields.iter().filter(|(_, holds)| holds == "true").map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        holding,
        [
            "positive_control_jia_after_patch_copied_before",
            "positive_control_yi2_persistent_sharing_turned_off",
            "positive_control_yi1_reference_counted_sharing_turned_off",
            "positive_control_bing_cloned_before_handing_in",
            "positive_control_ding_period_1_whole_new_base",
            "recompute_positive_control_yi2_persistent",
            "recompute_positive_control_bing_by_value",
            "recompute_positive_control_ding_without_compaction",
        ]
    );
}

#[test]
fn discriminative_power_examples_flip_between_the_main_and_small_geometries() {
    let (lines, every_example_flips) = judgement_lines(&sampling_points());
    assert!(every_example_flips);
    assert_eq!(lines.iter().filter(|line| line.starts_with("name=discriminative_power ")).count(), 3);
}

#[test]
fn jia_growth_order_before_admission() {
    let lines = growth_order_lines(&sampling_points());
    assert!(lines.iter().any(|line| line.contains("quantity=jia_after_patch_copied_before from_tombstone_records=1e5 to_tombstone_records=1e6 lower=41402308 upper=404072244 slope=0.98943")));
    assert!(lines.iter().any(|line| line.contains("from_tombstone_records=1e6 to_tombstone_records=1e7 lower=404072244 upper=4030722004 slope=0.99892")));
}

#[test]
fn ding_smallest_period_with_a_ratio_of_two() {
    let lines = ding_history_lines();
    assert!(lines.iter().any(|line| line.contains("compaction=whole_new_base") && line.ends_with("smallest_period_with_copied_after_ratio_at_least_two=3")));
    assert!(lines.iter().any(|line| line.contains("compaction=fold_in_place") && line.ends_with("smallest_period_with_copied_after_ratio_at_least_two=at_most_1")));
}
