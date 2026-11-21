use super::*;

#[test]
fn the_eleven_tree_kinds_are_the_records_constants_and_fill_the_tree_table() {
    let codes: BTreeSet<u16> = ELEVEN_TREE_KINDS.iter().map(|kind| kind.kind_code()).collect();
    assert_eq!(codes, [1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12].into_iter().collect::<BTreeSet<u16>>());
    assert_eq!(length_as_u64(ELEVEN_TREE_KINDS.len()), TREE_TABLE_ENTRIES);
    assert_eq!(TreeKind::CentralMapping.kind_code(), 5);
}
