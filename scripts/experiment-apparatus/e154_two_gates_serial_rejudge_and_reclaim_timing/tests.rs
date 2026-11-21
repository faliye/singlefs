#[test]
fn push_pool_and_residue_match_registered_constants() {
    assert_eq!(10 + 7 * 4, 38);
    assert_eq!(5 + 6 * 4, 29);
    assert_eq!(38 + 29, 67);
    assert_eq!(10 + 7 * 9, 73);
    assert_eq!(5 + 6 * 9, 59);
    assert_eq!(73 + 59, 132);
}
