use super::*;

/// 这一版的树表别的版本先走过：条目不重走，「走到没有条款的形状」照先走那一版记下的条目认；开始走下一版时清掉。
#[test]
fn version_sharing_a_tree_table_walked_earlier_takes_its_entry_without_a_clause_until_the_next_version(
) {
    let image = ImageHoldingOneUnitAtSlotOne::of(&index_node_of_tree(7));
    let mut walk = Walk::starting_with(&image, Judgements::default(), 0, 0, 10);
    let tree_table_placement = (0u32, 1u64);
    walk.tree_table_entries_of_a_walked_tree_table.insert(
        tree_table_placement,
        vec![TreeTableEntryOfTheWalk {
            tree: 65_541,
            kind: TREE_KIND_NOT_REGISTERED,
            previous_snapshot_txg: 0,
            birth_txg: 3,
            user_visible_root: None,
            root_pointer_is_all_zero: true,
            is_a_snapshot_being_destroyed: false,
        }],
    );
    walk.judge_the_tree_table_entries_an_earlier_version_walked(tree_table_placement);
    assert_eq!(
        walk.kind_of_a_tree_table_entry_without_a_clause_in_this_version,
        Some(TREE_KIND_NOT_REGISTERED),
        "共享这片树表的版本同样引用没收全"
    );
    walk.begin_walking_a_version();
    assert_eq!(
        walk.kind_of_a_tree_table_entry_without_a_clause_in_this_version, None,
        "下一版从没有记起"
    );
}
