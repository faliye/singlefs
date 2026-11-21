use super::*;

#[test]
fn after_admission_parts_at_the_focused_and_hundred_million_points() {
    let focused = jia_after_patch_copied_after_admission(&main_with(|knobs| knobs.touched_leaves = TouchedLeaves::Focused));
    assert_eq!(focused.mapping_carried_nodes_each_copy, 65728);
    let large = main_with(|knobs| knobs.tombstone_records = 100_000_000);
    let parts = jia_after_patch_copied_after_admission(&large);
    assert_eq!((parts.inode_leaves_again, parts.mapping_carried_nodes_each_copy, parts.multi_level_shapes), (64280, 3368560, 14357051));
    assert_eq!(jia_after_patch_copied_before_admission(&large).outside_the_rounds, 19616);
    assert_eq!((large.mapping_levels_after.clone(), large.mapping_carried_nodes), (vec![423, 3, 1], 205));
    assert_eq!((large.allocation_levels_after.clone(), large.allocation_carried_nodes), (vec![608, 4, 1], 524));
}

#[test]
fn publish_that_appends_no_tombstone() {
    let geometry = main_with(|knobs| knobs.tombstones_added = 0);
    let rounds = jia_after_patch_copied_before_admission(&geometry);
    assert_eq!((rounds.ordinary_allocation_pass, rounds.settle_round), (40277664, 585220));
    assert_eq!((jia_before(&geometry), jia_after(&geometry)), (203729488, 81266111));
}
