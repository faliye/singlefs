use super::*;

// ── 取样点覆盖实验会跑到的全部取值；判定那几格 ──

#[test]
fn every_sampling_point_yields_its_lines() {
    let points = sampling_points();
    assert_eq!(points.len(), 40);
    for (label, knobs) in &points {
        let lines = point_lines(label, &Geometry::new(*knobs));
        assert_eq!(lines.len(), 1 + Arm::EVERY_ARM.len() + 5 + judged_cells().len(), "{label}");
    }
}

#[test]
fn main_geometry_judged_cells_and_their_ratios() {
    let geometry = main_with(|_| {});
    let yi2_before = JudgedCell { compared_arm: Arm::Yi2, after_admission: false, recomputed: false };
    let forms: Vec<String> = yi2_before.forms(&geometry).iter().map(|&(numerator, denominator)| ratio_text(numerator, denominator)).collect();
    assert_eq!(forms, ["200.6006", "199.2003"]);
    let bing_before = JudgedCell { compared_arm: Arm::Bing, ..yi2_before };
    assert_eq!(ratio_text(bing_before.forms(&geometry)[0].0, bing_before.forms(&geometry)[0].1), "zero_denominator");
    for cell in judged_cells() {
        assert_eq!(cell.verdict(&geometry, JUDGED_THRESHOLD), "does_not_lean_keep_jia", "{}", cell.label());
    }
    assert_eq!(judged_cells().len(), 12);
}

#[test]
fn verdicts_name_a_side_only_from_the_threshold() {
    assert_eq!(side_of(399, 200, JUDGED_THRESHOLD), Side::BelowThreshold);
    assert_eq!(side_of(400, 200, JUDGED_THRESHOLD), Side::AtOrAboveThreshold);
    assert_eq!(side_of(1, 0, JUDGED_THRESHOLD), Side::AtOrAboveThreshold);
    assert_eq!(verdict_of_sides(&[Side::BelowThreshold, Side::AtOrAboveThreshold]), "unstable");
    assert_eq!(verdict_of_sides(&[Side::BelowThreshold, Side::BelowThreshold]), "leans_keep_jia");
}

#[test]
fn geometry_sensitivity_names_the_point_that_flips_a_judged_cell() {
    let (lines, _) = judgement_lines(&sampling_points());
    let yi2_before = lines
        .iter()
        .find(|line| line.starts_with("name=judgement point=main_geometry cell=jia_after_patch_over_yi2_persistent_copied_before_admission "))
        .expect("甲 ÷ 乙2 准入前照抄那一格有一行");
    assert!(yi2_before.contains("verdict=does_not_lean_keep_jia sensitivity=unstable_flipped_at_small_geometry "), "{yi2_before}");
}

#[test]
fn yi2_positive_control_copies_every_collection_whole_with_sharing_turned_off() {
    assert_eq!(yi2_sharing_off_parts(&main_with(|_| {})), (40173272, 438408, 144, 202620136));
    assert_eq!(yi2_sharing_off_parts(&main_with(|knobs| knobs.tombstone_records = 100_000)), (4111344, 54568, 48, 20775040));
    assert_eq!(side_of(202620136, 20775040, (9, 1)), Side::AtOrAboveThreshold);
    let (lines, _) = positive_control_lines(&sampling_points());
    assert!(lines.iter().any(|line| line
        == "name=positive_control arm=yi2_persistent_sharing_turned_off tombstone_records_1e5=20775040 tombstone_records_1e6=202620136 ratio=9.7531 threshold=9 holds=true"));
}
