use super::*;

#[test]
fn switch_reserve_per_disk_matches_registered_formula() {
    let switch_share = |fixpoint_cost: u64| 2 + RING_REGIONS * fixpoint_cost;
    assert_eq!((INSTANCE_SWITCH_LIMIT + 1) * switch_share(4), 56);
    assert_eq!((INSTANCE_SWITCH_LIMIT + 1) * switch_share(9), 116);
}

#[test]
fn retry_budget_is_eight() {
    assert_eq!(RETRY_BUDGET_PUBLICATIONS, 4 + 2 * TOLERATED_ROOT_WRITE_FAILURE_COUNT);
    assert_eq!(RETRY_BUDGET_PUBLICATIONS, 8);
}

/// 重试循环的触动预算 J_max = 3S + 5（登记「五、5.4」）。
#[test]
fn retry_touch_budget_matches_registered_formula() {
    assert_eq!(retry_touch_budget(4), 17);
    assert_eq!(retry_touch_budget(8), 29);
}
