use super::*;

// ── 第七节「独立算出」：G0 两棵树与甲 ──

#[test]
fn main_geometry_mapping_and_allocation_record_trees() {
    let geometry = main_with(|_| {});
    assert_eq!((geometry.mapping_entries, geometry.mapping_levels.clone()), (1272, vec![5, 1]));
    assert_eq!((geometry.slots_per_device, geometry.allocation_levels.clone()), (2501, vec![8, 1]));
    assert_eq!(geometry.mapping_changed_entries, 56);
    assert_eq!((geometry.mapping_rewritten_nodes, geometry.mapping_carried_nodes), (6, 0));
    assert_eq!((geometry.allocation_rewritten_nodes, geometry.allocation_carried_nodes), (9, 0));
    assert_eq!((geometry.units, geometry.rewritten_units, geometry.new_units), (1280, 35, 35));
}

#[test]
fn main_geometry_jia_after_patch_copies_before_admission() {
    let geometry = main_with(|_| {});
    let rounds = jia_after_patch_copied_before_admission(&geometry);
    assert_eq!((rounds.ordinary_allocation_pass, rounds.settle_round, rounds.outside_the_rounds), (80346376, 585019, 288));
    assert_eq!(jia_before(&geometry), 404072244);
}

#[test]
fn main_geometry_jia_after_patch_copies_after_admission() {
    assert_eq!(jia_after(&main_with(|_| {})), 81200231);
}

#[test]
fn main_geometry_jia_today_copies_1808_more_before_admission() {
    let geometry = main_with(|_| {});
    assert_eq!(jia_today_copied_before_admission(&geometry), 404074052);
    assert_eq!(jia_before(&geometry) + 1808, jia_today_copied_before_admission(&geometry));
}

#[test]
fn jia_today_is_the_same_under_the_second_and_third_registration_formulas() {
    for (_, knobs) in sampling_points() {
        let geometry = Geometry::new(knobs);
        let added = items_added_by_the_third_registration(&geometry);
        let settle_rounds = knobs.settle_rounds;
        let second_registration_jia = jia_before(&geometry) - added.tombstone_shape_when_not_appending - added.lineage_shape_when_not_appending;
        let tombstone_and_retirement: u64 =
            geometry.tombstone_and_retirement_trees().iter().map(|(tree, _)| (2 + tree.new_containers) * tree.shape_bytes()).sum();
        let second_registration_today = second_registration_jia
            + (settle_rounds + 1) * tombstone_and_retirement
            + settle_rounds * (2 + geometry.lineage.new_containers) * geometry.lineage.shape_bytes();
        assert_eq!(second_registration_today, jia_today_copied_before_admission(&geometry), "{knobs:?}");
    }
}

#[test]
fn after_admission_parts_at_the_main_geometry() {
    let parts = jia_after_patch_copied_after_admission(&main_with(|_| {}));
    assert_eq!(parts.inode_leaves_again, 64280);
    assert_eq!(parts.mapping_carried_nodes_each_copy, 0);
    assert_eq!(parts.multi_level_shapes, 1664 + 147087);
}

#[test]
fn after_admission_parts_at_the_small_geometry() {
    let small = Geometry::new(Knobs::small_geometry());
    let parts = jia_after_patch_copied_after_admission(&small);
    assert_eq!((parts.inode_leaves_again, parts.mapping_carried_nodes_each_copy, parts.multi_level_shapes), (120, 0, 4111));
    assert_eq!(jia_after_patch_copied_before_admission(&small).outside_the_rounds, 96);
}

#[test]
fn main_geometry_yi2_bing_and_ding() {
    let geometry = main_with(|_| {});
    assert_eq!(yi2_copied(&geometry), (2014312, 421160));
    assert_eq!(recomputed_residual(&geometry), (64160, 132912));
    assert_eq!(arm_bytes(&geometry, Arm::Bing), ArmBytes { copied_before: 0, copied_after: 4648, recomputed_before: 64160, recomputed_after: 132912 });
    let compaction = arm_bytes(&geometry, Arm::DingWholeNewBaseCompaction);
    assert_eq!((compaction.copied_after, compaction.recomputed_after), (81793919, 925712));
    assert_eq!(arm_bytes(&geometry, Arm::DingWithoutCompaction).copied_after, 0);
}

#[test]
fn small_geometry_every_arm() {
    let small = Geometry::new(Knobs::small_geometry());
    assert_eq!((jia_before(&small), jia_after(&small)), (14398, 39575));
    assert_eq!(yi2_copied(&small), (19166, 83239));
    assert_eq!(bing_copied_after_admission(&small), 3080);
    assert_eq!(installed_bytes(&small), (3119, 412584));
    assert_eq!(recomputed_in_full(&small), (8223, 16094));
    assert_eq!(recomputed_residual(&small), (8175, 15998));
}

#[test]
fn focused_touched_leaves() {
    let geometry = main_with(|knobs| knobs.touched_leaves = TouchedLeaves::Focused);
    assert_eq!((geometry.mapping_rewritten_nodes, geometry.mapping_carried_nodes), (2, 4));
    assert_eq!((geometry.allocation_rewritten_nodes, geometry.allocation_carried_nodes, geometry.rewritten_units), (3, 6, 25));
    assert_eq!((jia_before(&geometry), jia_after(&geometry)), (404073852, 81431079));
    assert_eq!(recomputed_in_full(&geometry).1, 848450);
    assert_eq!(recomputed_residual(&geometry), (58640, 50766));
}

#[test]
fn small_geometry_turns_jia_over_yi2_before_admission_below_two() {
    let small = Geometry::new(Knobs::small_geometry());
    let yi2_before = JudgedCell { compared_arm: Arm::Yi2, after_admission: false, recomputed: false };
    assert_eq!(yi2_before.verdict(&small, JUDGED_THRESHOLD), "leans_keep_jia");
    assert_eq!(ratio_text(yi2_before.forms(&small)[0].0, yi2_before.forms(&small)[0].1), "0.7512");
}

#[test]
fn read_hops_at_the_main_geometry() {
    assert_eq!(yi2_extra_hops(&main_with(|_| {})), (5, 3));
}

#[test]
fn resident_peaks_of_jia_and_bing_at_the_main_geometry() {
    let geometry = main_with(|_| {});
    assert_eq!(jia_resident_peak(&geometry), 245898497);
    assert_eq!(bing_resident_peak(&geometry), 82623711);
    assert_eq!(resident_peak_side(82623711, 245898497), "less_than_jia");
}

#[test]
fn yi2_sharing_turned_off_copies_the_untouched_retirement_tree_and_every_record() {
    // a_X = 1 的点上退役树碰到了，第四份第九节「漏掉没碰到的树」那一条在这里不变；G-small 一片、fill = R。
    assert_eq!(yi2_sharing_off_parts(&main_with(|knobs| knobs.retirements_added = 1)).3, 202620136);
    assert_eq!(yi2_sharing_off_parts(&Geometry::new(Knobs::small_geometry())).3, 10558);
    let retirements = &main_with(|_| {}).retirements;
    assert_eq!(whole_container_tree_bytes(retirements, RETIREMENT_RECORD_MEMORY_BYTES), 40000 + 112 + 32);
}

#[test]
fn yi2_positive_control_reported_only_reading_copies_only_the_touched_collections() {
    let parts = |geometry: &Geometry| {
        let rounds = yi2_touched_collections_whole_copied_before_admission(geometry);
        (rounds.ordinary_allocation_pass, rounds.settle_round, rounds.outside_the_rounds, rounds.total(geometry.knobs.settle_rounds))
    };
    assert_eq!(parts(&main_with(|_| {})), (116440, 437936, 144, 2334088));
    assert_eq!(parts(&main_with(|knobs| knobs.tombstone_records = 100_000)), (44912, 54096, 48, 440992));
    let (lines, fields) = positive_control_lines(&sampling_points());
    assert!(lines.iter().any(|line| line.starts_with(
        "name=positive_control_reported_only arm=yi2_persistent_touched_collections_whole tombstone_records_1e5=440992 tombstone_records_1e6=2334088 ratio=5.2928 judged=no"
    )));
    assert!(fields.iter().all(|(name, _)| !name.contains("touched_collections_whole")));
}

#[test]
fn yi1_copied_bytes_at_the_registered_points() {
    let yi1 = |geometry: &Geometry| {
        let rounds = yi1_copied_before_admission(geometry, ReferenceSharing::Shared);
        (rounds.total(geometry.knobs.settle_rounds), yi1_copied_after_admission(geometry, ReferenceSharing::Shared))
    };
    let main = main_with(|_| {});
    let rounds = yi1_copied_before_admission(&main, ReferenceSharing::Shared);
    assert_eq!((rounds.ordinary_allocation_pass, rounds.settle_round, rounds.outside_the_rounds), (131464, 584587, 288));
    assert_eq!(yi1(&main), (2995956, 435031));
    let small = Geometry::new(Knobs::small_geometry());
    let small_rounds = yi1_copied_before_admission(&small, ReferenceSharing::Shared);
    assert_eq!((small_rounds.ordinary_allocation_pass, small_rounds.settle_round, small_rounds.outside_the_rounds), (640, 11934, 96));
    assert_eq!(yi1(&small), (13310, 6279));
    assert_eq!(yi1(&main_with(|knobs| knobs.tombstone_records = 100_000)), (807940, 108576));
    assert_eq!(yi1(&main_with(|knobs| knobs.tombstones_added = 0)), (2785680, 435295));
    assert_eq!(yi1(&main_with(|knobs| knobs.lineage_records_added = 1)), (2996988, 434855));
    assert_eq!(yi1(&main_with(|knobs| knobs.file_data_units = 32768)), (63297364, 12919367));
    assert_eq!(yi1(&main_with(|knobs| knobs.touched_leaves = TouchedLeaves::Focused)), (2997564, 435943));
    assert_eq!(yi1(&main_with(|knobs| knobs.tombstone_records = 1_000_005)).0, 2834224);
}
