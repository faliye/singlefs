use super::*;

#[test]
fn the_two_equal_height_points_give_yi2_bing_and_the_residual_the_same_bytes() {
    let lower = main_with(|knobs| (knobs.tombstone_records, knobs.touched_leaves) = (5_216_400, TouchedLeaves::Focused));
    let upper = main_with(|knobs| (knobs.tombstone_records, knobs.touched_leaves) = (13_040_400, TouchedLeaves::Focused));
    assert_eq!((lower.tombstones.containers, upper.tombstones.containers), (6401, 16001));
    for geometry in [&lower, &upper] {
        assert_eq!(yi2_copied(geometry), (4267192, 703976));
        assert_eq!(bing_copied_after_admission(geometry), 3368);
        assert_eq!(recomputed_residual(geometry), (58640, 50766));
        assert_eq!((geometry.rewritten_units, geometry.mapping_changed_entries), (25, 44));
    }
    assert_eq!((jia_before(&lower), jia_before(&upper)), (2103124788, 5255907796));
}

// ── 第三份补进的四项 ──

#[test]
fn items_added_by_the_third_registration_at_the_registered_points() {
    let pairs = |geometry: &Geometry| {
        let added = items_added_by_the_third_registration(geometry);
        (added.tombstone_shape_when_not_appending, added.lineage_shape_when_not_appending)
    };
    assert_eq!(pairs(&main_with(|_| {})), (0, 128));
    assert_eq!(pairs(&main_with(|knobs| knobs.tombstones_added = 0)), (680, 128));
    assert_eq!(pairs(&main_with(|knobs| knobs.settle_rounds = 64)), (0, 2048));
    assert_eq!(pairs(&main_with(|knobs| knobs.lineage_records_added = 1)), (0, 0));
    assert_eq!(pairs(&Geometry::new(Knobs::small_geometry())), (0, 0));
    assert_eq!(pairs(&main_with(|knobs| (knobs.tombstone_records, knobs.tombstones_added) = (100_000_000, 0))), (37040, 128));
}

#[test]
fn yi1_positive_control_with_sharing_turned_off_is_jia_after_patch() {
    let total = |geometry: &Geometry| yi1_copied_before_admission(geometry, ReferenceSharing::TurnedOff).total(geometry.knobs.settle_rounds);
    let main = main_with(|_| {});
    let lower = main_with(|knobs| knobs.tombstone_records = 100_000);
    assert_eq!((total(&main), total(&lower)), (404072244, 41402308));
    assert_eq!(yi1_copied_after_admission(&main, ReferenceSharing::TurnedOff), 81200231);
    for (label, knobs) in sampling_points() {
        let geometry = Geometry::new(knobs);
        assert_eq!(total(&geometry), jia_before(&geometry), "{label}");
        assert_eq!(yi1_copied_after_admission(&geometry, ReferenceSharing::TurnedOff), jia_after(&geometry), "{label}");
    }
    let (lines, _) = positive_control_lines(&sampling_points());
    assert!(lines.iter().any(|line| line
        == "name=positive_control arm=yi1_reference_counted_sharing_turned_off tombstone_records_1e5=41402308 tombstone_records_1e6=404072244 ratio=9.7597 threshold=9 holds=true"));
}
