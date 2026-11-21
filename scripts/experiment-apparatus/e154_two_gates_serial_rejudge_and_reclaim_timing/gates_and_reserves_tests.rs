use super::*;

#[test]
fn gate2_formula_matches_hand_computed_sample() {
    let snapshot = GateSnapshot {
        allocatable_free_slots: 50,
        all_free_slots_including_isolated_and_held: 50,
        released: 0,
        live_meta: 4,
        checkpoint_reserve: 4,
        push_reserve: 38,
        residue: 29,
        lag: 0,
        in_flight: 10,
        abandoned_root_exclusive_slots: 0,
        occupied_pub: 0,
        switch_reserve: 0,
        held: 0,
        defined_free_space_formula_slots: 100,
    };
    assert_eq!(compute_gate2(&snapshot), 6);
}

/// A2：`gate_snapshot` 真跑一遍起点状态，推空保留池 = 10 + 7c、残留 = 5 + 6c、
/// checkpoint 保留池 = c，c = 4 时分别是 38、29、4（与登记「七」A2 一致，这里核的是
/// 活代码里的公式，不是像 `push_pool_and_residue_match_registered_constants` 那样单独验算）。
#[test]
fn gate_snapshot_matches_registered_reserve_formula() {
    let geometry = baseline_geometry();
    let pool = Pool::seed(geometry);
    let snapshot = gate_snapshot(&pool, 0);
    assert_eq!(snapshot.push_reserve, 38, "推空保留池 10 + 7 × 4");
    assert_eq!(snapshot.residue, 29, "残留 5 + 6 × 4");
    assert_eq!(snapshot.checkpoint_reserve, 4, "checkpoint 保留池 = c");
}
