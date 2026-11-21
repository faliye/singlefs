use super::*;

/// R8：根槽写失败也要推进一格 txg（D23（journal 的角色与格式） 已定项 14 逐字「不推进就是
/// 反复重写同一个槽、把 R 个失败域用成一个」）。这条分支在 H1 里从没被走到过（H1 从不注入
/// 失败），只有 H5 走到它（见登记「十二、修订」）。
#[test]
fn failed_publish_still_advances_next_txg() {
    let geometry = baseline_geometry();
    let mut pool = Pool::seed(geometry);
    let before = pool.next_txg;
    let result = attempt_publish(
        &mut pool,
        ReclaimTiming::AfterEffectiveFloor,
        false,
        0,
        None,
        None,
        false,
        None,
        true,
    );
    assert!(result.is_err(), "注入 fail=true 的发布应当返回 Err");
    assert_eq!(
        pool.next_txg,
        before + 1,
        "写失败也要推进一格 txg，不然重发会撞回同一个环槽"
    );
}

/// A5：起点状态之后，seed 出的下一个 txg 是 3（mkfs=0、暖机=1、2；D16（发布语义） 已定项 8）。
#[test]
fn seed_next_txg_is_three() {
    let pool = Pool::seed(baseline_geometry());
    assert_eq!(pool.next_txg, 3, "起点状态之后第一次用户发布的 txg 应为 3");
    assert_eq!(pool.switch_reserve_shares_per_disk, INSTANCE_SWITCH_LIMIT, "起点状态『平时 3 份』（写行与暖机已做完）");
}

/// I6（自证）：seed 出的状态本身就满足守恒——c_max 个固定点槽 + 2 个实例表槽仍分配，其余空闲。
#[test]
fn seed_state_satisfies_conservation() {
    let geometry = baseline_geometry();
    let pool = Pool::seed(geometry);
    assert_conservation(&pool, "seed");
    let allocated = pool.slots.iter().filter(|state| matches!(state, SlotState::Allocated)).count() as u64;
    assert_eq!(allocated, geometry.fixpoint_cost + 2, "起点状态『c 个固定点槽与 2 个实例表槽仍分配』");
}

/// I7：平时切换预留 = 3 × (2 + 3c)；每次实例切换做完再少 2 + 3c。
#[test]
fn switch_reserve_slots_matches_registered_formula() {
    let mut pool = Pool::seed(baseline_geometry());
    assert_eq!(pool.switch_reserve_slots(), 3 * (2 + 3 * 4), "平时（写行暖机做完）应为 3 份 × (2 + 3c)");
    pool.switch_reserve_shares_per_disk -= 1;
    assert_eq!(pool.switch_reserve_slots(), 2 * (2 + 3 * 4), "切换一次少一份 2 + 3c");
}

/// I10：填满 + 写 X + 全部填充对象写完之后，仍分配槽数 = X 的槽数 + 全部填充对象槽数
/// + 固定点槽数 + 实例表槽数（起点状态守恒，逐事件断言已经盯着，这里只核填满阶段末尾那一格）。
#[test]
fn fill_phase_allocation_matches_objects_written() {
    let geometry = baseline_geometry();
    let arm = real_baseline_arm();
    let mut pool = Pool::seed(geometry);
    let (_, fillers) = fill_pool(&mut pool, arm, geometry, &mut |_last_txg| false);
    let allocated = pool.slots.iter().filter(|state| matches!(state, SlotState::Allocated)).count() as u64;
    let expected = geometry.object_units * 2 // X
        + fillers.len() as u64 * 2 // 填充对象各 2 槽
        + geometry.fixpoint_cost // 固定点
        + 2; // 实例表
    assert_eq!(allocated, expected, "填满阶段末尾：仍分配槽数应等于全部写出的对象与元数据之和");
}
