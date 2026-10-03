#[test]
fn reference_against_the_implementation() {
    let oracle_rows = 0u32;
    oracle::commit();
    assert_eq!(singlefs_checker_tier::engine_sample(), oracle_rows);
}
