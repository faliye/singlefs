use super::*;

#[test]
fn main_geometry_installed_bytes_version_sizes_and_recomputation() {
    let geometry = main_with(|_| {});
    assert_eq!(installed_bytes(&geometry), (14232, 610728));
    assert_eq!((version_bytes(&geometry), version_bytes_after(&geometry)), (81793847, 81793919));
    assert_eq!(recomputed_in_full(&geometry), (1814496, 925712));
}

#[test]
fn publish_that_appends_815000_tombstones() {
    let geometry = main_with(|knobs| knobs.tombstones_added = 815_000);
    assert_eq!((jia_before(&geometry), jia_after(&geometry)), (404071172, 113972417));
    assert_eq!(installed_bytes(&geometry), (163286052, 66338416));
}

#[test]
fn carried_file_data_units_and_pointers_at_32768_data_units() {
    let geometry = main_with(|knobs| knobs.file_data_units = 32768);
    let added = items_added_by_the_third_registration(&geometry);
    assert_eq!((added.carried_file_data_units, added.carried_file_data_pointers), (1074528256, 2883584));
    assert_eq!(added.carried_file_data_units + added.carried_file_data_pointers, 1077411840);
    let parts = jia_after_patch_copied_after_admission(&geometry);
    assert_eq!((parts.carried_file_data_units, parts.carried_file_data_pointers), (1074528256, 2883584));
    let trees = geometry.trees_with_record_widths().map(|(tree, width)| (tree.records, width, tree.containers, tree.nodes, tree.height));
    let (without_pointers, pointers) =
        version_size_parts(trees, &geometry, geometry.mapping_entries, geometry.mapping_nodes, geometry.allocation_nodes, geometry.allocation_records);
    assert_eq!(without_pointers + 2883584, version_bytes(&geometry));
    assert_eq!(pointers, 2883584);
    assert_eq!((jia_before(&geometry), jia_after(&geometry), version_bytes(&geometry)), (464373652, 2244448535, 1173117639));
    let main = items_added_by_the_third_registration(&main_with(|_| {}));
    assert_eq!((main.carried_file_data_units, main.carried_file_data_pointers), (0, 0));
}

#[test]
fn ding_resident_peak_without_compaction_is_the_base_every_layer_and_the_residual() {
    // 第四份第五节：丁加残余式；p = 1024 在 256 次里不压实，两种压实方式峰值相同（互比旁边钉绝对值）。
    let geometries = ding_history_geometries();
    let whole = ding_history(&geometries, 1024, Compaction::WholeNewBase);
    assert_eq!((whole.resident_peak, whole.resident_final), (238289246, 238289246));
    let fold = ding_history(&geometries, 1024, Compaction::FoldInPlace);
    assert_eq!((fold.resident_peak, fold.resident_final), (238289246, 238289246));
}

#[test]
fn ding_resident_peak_at_period_sixty_four_for_both_compactions() {
    let geometries = ding_history_geometries();
    let whole = ding_history(&geometries, 64, Compaction::WholeNewBase);
    assert_eq!((whole.resident_peak, whole.resident_final), (203701016, 203701016));
    let fold = ding_history(&geometries, 64, Compaction::FoldInPlace);
    assert_eq!((fold.resident_peak, fold.resident_final), (121863763, 121863763));
}

#[test]
fn ding_resident_peak_at_period_one_for_both_compactions() {
    let geometries = ding_history_geometries();
    let whole = ding_history(&geometries, 1, Compaction::WholeNewBase);
    assert_eq!((whole.resident_peak, whole.resident_final), (165227672, 165227672));
    let fold = ding_history(&geometries, 1, Compaction::FoldInPlace);
    assert_eq!((fold.resident_peak, fold.resident_final), (83390419, 83390419));
}

#[test]
fn bing_and_ding_positive_controls_at_the_two_registered_points() {
    let main = main_with(|_| {});
    let lower = main_with(|knobs| knobs.tombstone_records = 100_000);
    assert_eq!(bing_copied_after_admission(&main) + version_bytes(&main), 4648 + 81793847);
    assert_eq!(bing_copied_after_admission(&lower) + version_bytes(&lower), 3096 + 8863272);
    assert_eq!(ding_copied_after_admission(&main, true, Compaction::WholeNewBase), 81793919);
    assert_eq!(ding_copied_after_admission(&lower, true, Compaction::WholeNewBase), 8863344);
    let (lines, _) = positive_control_lines(&sampling_points());
    assert!(lines.iter().any(|line| line
        == "name=positive_control arm=bing_cloned_before_handing_in tombstone_records_1e5=8866368 tombstone_records_1e6=81798495 ratio=9.2257 threshold=9 holds=true"));
    assert!(lines.iter().any(|line| line
        == "name=positive_control arm=ding_period_1_whole_new_base tombstone_records_1e5=8863344 tombstone_records_1e6=81793919 ratio=9.2283 threshold=9 holds=true"));
}
