use super::*;

/// A6：全部臂、全部 12 格，txg 3 那一行 Disk0：第 1 项 13、第 5 项 1、第 2 项 211968-13。
#[test]
fn txg_three_accounting_row_matches_every_arm_and_cell() {
    for cell in all_geometry_cells() {
        let world = new_pool_file_creation_world(cell);
        for arm in &world.arms {
            let row = arm.accounting_row();
            assert_eq!(row.item1_occupied_slots, 13, "{:?} cell={:?}", arm.arm, cell);
            assert_eq!(row.item5_deferred_slots, 1, "{:?} cell={:?}", arm.arm, cell);
            assert_eq!(row.item2_free_slots, UNIT_AREA_CAPACITY_SLOTS - 13, "{:?} cell={:?}", arm.arm, cell);
        }
    }
}

/// U3（M5 的锚点）：G12 的 blocking 集合必须含被抛弃根，不能只看候选集。
/// 记录 [5,20)、blocking={7,25}（7 是被抛弃根）：原式不可再分配。
#[test]
fn blocking_set_must_include_abandoned_roots() {
    let blocking_with_abandoned: BTreeSet<u64> = [7u64, 25].into_iter().collect();
    let record = pending_record(0, 5, 20);
    assert!(!is_record_reclaimable(Predicate::IntervalBlocking, 0, Some(&blocking_with_abandoned), &record), "7 在 [5,20) 里且是被抛弃根，原式不可再分配");
    // M5 的变异：丢「被抛弃」子句，blocking 只剩候选集 {25}——25 不在 [5,20) 里，变异应当变成可再分配。
    let blocking_without_abandoned: BTreeSet<u64> = [25u64].into_iter().collect();
    assert!(is_record_reclaimable(Predicate::IntervalBlocking, 0, Some(&blocking_without_abandoned), &record), "丢了被抛弃根之后 [5,20) 里没有 blocking 成员");
}

/// U4（M6 的锚点）：G12 的 blocking 集合里，有效根要 txg ≥ F_生效 才算 blocking，不是「有效根一律保护」。
/// 记录 [5,20)、环 {12 有效, 25}、F_生效 15：12 < 15 不该 blocking，原式可再分配。
#[test]
fn valid_root_below_floor_does_not_block() {
    let blocking_correct: BTreeSet<u64> = [25u64].into_iter().collect(); // 12 valid 但 12 < F_生效=15，不进 blocking
    let record = pending_record(0, 5, 20);
    assert!(is_record_reclaimable(Predicate::IntervalBlocking, 0, Some(&blocking_correct), &record), "12 未过 F_生效 门槛，不 blocking，原式可再分配");
    // M6 的变异：丢「txg ≥ F_生效」子句，有效根一律保护——12 也进 blocking，12 在 [5,20) 里，变异应当不可再分配。
    let blocking_mutated: BTreeSet<u64> = [12u64, 25].into_iter().collect();
    assert!(!is_record_reclaimable(Predicate::IntervalBlocking, 0, Some(&blocking_mutated), &record));
}
