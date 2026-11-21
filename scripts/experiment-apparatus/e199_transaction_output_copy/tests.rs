use super::*;

// ── 第七节「独立算出」：容器树形状 ──

#[test]
fn tombstone_tree_shape_at_a_million_records() {
    let tree = &main_with(|_| {}).tombstones;
    assert_eq!((tree.containers, tree.levels.clone(), tree.nodes, tree.fill), (1227, vec![10, 1], 11, 810));
}

#[test]
fn tombstone_tree_shape_at_ten_and_a_hundred_million_records() {
    let ten_million = main_with(|knobs| knobs.tombstone_records = 10_000_000);
    assert_eq!((ten_million.tombstones.containers, ten_million.tombstones.levels.clone()), (12270, vec![91, 1]));
    let hundred_million = main_with(|knobs| knobs.tombstone_records = 100_000_000);
    assert_eq!((hundred_million.tombstones.containers, hundred_million.tombstones.levels.clone()), (122700, vec![909, 7, 1]));
}

#[test]
fn tombstone_tree_grows_a_level_when_the_container_count_passes_the_fanout() {
    let tree = &main_with(|knobs| knobs.tombstone_records = 110_025).tombstones;
    assert_eq!((tree.containers, tree.levels.clone()), (135, vec![1]));
    assert_eq!((tree.containers_after, tree.levels_after.clone()), (136, vec![2, 1]));
}

#[test]
fn jia_before_admission_along_the_tombstone_records_sweep() {
    let expected = [(0, 1107132), (10, 1114972), (1000, 1513360), (10_000, 5139628), (100_000, 41402308), (10_000_000, 4030722004), (100_000_000, 40297407128)];
    for (tombstone_records, copied) in expected {
        assert_eq!(jia_before(&main_with(|knobs| knobs.tombstone_records = tombstone_records)), copied, "u = {tombstone_records}");
    }
    assert_eq!(jia_after(&main_with(|knobs| knobs.tombstone_records = 10_000_000)), 808615927);
}

#[test]
fn jia_before_admission_at_one_and_sixty_four_settle_rounds() {
    assert_eq!(jia_before(&main_with(|knobs| knobs.settle_rounds = 1)), 161278059);
    assert_eq!(jia_before(&main_with(|knobs| knobs.settle_rounds = 64)), 5259955944);
}

#[test]
fn publish_that_opens_a_new_tombstone_container() {
    let geometry = main_with(|knobs| knobs.tombstone_records = 1_000_005);
    assert_eq!((geometry.tombstones.open_container_rewritten, geometry.tombstones.new_containers), (0, 1));
    assert_eq!((jia_before(&geometry), jia_after(&geometry)), (404074512, 81233498));
}

#[test]
fn half_full_mapping_leaves() {
    let geometry = main_with(|knobs| knobs.mapping_leaves_half_full = true);
    assert_eq!(geometry.mapping_levels.clone(), vec![9, 1]);
    assert_eq!((jia_before(&geometry), jia_after(&geometry)), (404076644, 81201171));
}
