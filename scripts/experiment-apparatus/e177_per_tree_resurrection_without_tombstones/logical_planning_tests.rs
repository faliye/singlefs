use super::*;

#[test]
fn fifteen_accounting_rows_three_pool_wide_and_six_per_device() {
    let keys = accounting_keys_of(10);
    assert_eq!(length_as_u64(keys.len()), 15);
    assert!(keys.iter().all(|fields| fields[3] == 10));
}
