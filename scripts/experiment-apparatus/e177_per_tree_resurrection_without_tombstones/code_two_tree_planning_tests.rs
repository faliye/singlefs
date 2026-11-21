use super::*;

#[test]
fn capacities_follow_the_header_formula() {
    assert_eq!(INODE_LEAF_RECORDS, 233);
    assert_eq!(TOMBSTONE_CONTAINER_RECORDS, 815);
    assert_eq!(ROOT_NODE_CONTAINER_CAPACITY, 135);
    assert_eq!(accounting_tree_capacity(), CodeTwoCapacity { leaf_entries: 477, internal_entries: 150 });
    assert_eq!(mapping_tree_capacity(), CodeTwoCapacity { leaf_entries: 294, internal_entries: 143 });
    assert_eq!(allocation_tree_root_level(), 2);
    assert_eq!(RING_GEOMETRY_UNIT_AREA_SLOTS, 211_968);
}

fn keys_of(values: impl IntoIterator<Item = u64>) -> BTreeSet<KeyFields> {
    values.into_iter().map(single_field_key).collect()
}

fn leaves_of(shape: &CodeTwoShape) -> Vec<Vec<u64>> {
    shape
        .nodes
        .iter()
        .filter_map(|(_, contents)| match contents {
            CodeTwoNodeContents::Leaf { keys } => Some(keys.iter().map(|fields| fields[0]).collect()),
            CodeTwoNodeContents::Internal { .. } => None,
        })
        .collect()
}

#[test]
fn nine_keys_into_a_leaf_of_eight_split_it_in_the_middle_and_grow_a_root() {
    let capacity = CodeTwoCapacity { leaf_entries: 8, internal_entries: 4 };
    let plan = plan_code_two_tree(&CodeTwoShape::default(), &keys_of(1..=9), capacity);
    assert_eq!(leaves_of(&plan.shape), vec![vec![1, 2, 3, 4, 5], vec![6, 7, 8, 9]]);
    assert_eq!(plan.shape.root_position(), Some(CodeTwoNodePosition { level: 1, index_in_level: 0 }));
    assert!(plan.origins.iter().all(|origin| *origin == CodeTwoOrigin::Rewritten));
}

#[test]
fn deleting_every_key_of_a_leaf_drops_it_and_the_only_child_becomes_the_root_unrewritten() {
    let capacity = CodeTwoCapacity { leaf_entries: 8, internal_entries: 4 };
    let first = plan_code_two_tree(&CodeTwoShape::default(), &keys_of(1..=9), capacity);
    let second = plan_code_two_tree(&first.shape, &keys_of(6..=9), capacity);
    assert_eq!(leaves_of(&second.shape), vec![vec![6, 7, 8, 9]]);
    assert_eq!(second.shape.root_position(), Some(CodeTwoNodePosition { level: 0, index_in_level: 0 }));
    assert_eq!(second.origins, vec![CodeTwoOrigin::CarriedFrom(CodeTwoNodePosition { level: 0, index_in_level: 1 })]);
    assert_eq!(second.replaced_previous.len(), 2);
}
