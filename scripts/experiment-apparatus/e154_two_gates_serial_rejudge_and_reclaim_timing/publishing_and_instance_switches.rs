//! 一次发布的尝试与撤销、守恒断言、目标下限的推进、空发布与实例切换。

use super::*;

#[derive(Debug)]
pub(super) struct PublishAttemptFailed;

/// 一次发布（可能是空发布、用户发布、写行发布或回退那次发布）。
/// `object_release`／`object_allocate` 只用于用户发布；`rewrite_instance_table` 只在
/// 写行／回退发布为真。`fail` 为真时模拟根槽写失败（R8）：分配整份退回，不持久。
#[allow(clippy::too_many_arguments, reason = "发布是本模型唯一的写路径，参数对应 5.1 明确列出的各个输入维度，拆结构体不会减少要传的信息")]
pub(super) fn attempt_publish(
    pool: &mut Pool,
    reclaim: ReclaimTiming,
    non_empty: bool,
    floor: u64,
    object_release: Option<u64>,
    object_allocate: Option<u64>,
    rewrite_instance_table: bool,
    new_instance_rows: Option<Vec<InstanceRow>>,
    fail: bool,
) -> Result<u64, PublishAttemptFailed> {
    pool.publishes_so_far += 1;
    let txg = pool.next_txg;
    // ① 释放
    let released_object_slots =
        object_release.and_then(|object_identifier| pool.object_slots.get(&object_identifier).cloned());
    let old_fixpoint = pool.current_fixpoint_slots.clone();
    let old_instance_table_slots = pool.current_instance_table_slots.clone();
    release_slots(pool, &old_fixpoint, txg, !non_empty);
    if let Some(slots) = &released_object_slots {
        release_slots(pool, slots, txg, false);
    }
    if rewrite_instance_table {
        release_slots(pool, &old_instance_table_slots, txg, !non_empty);
    }
    // ② 分配。找到的槽当场标记 Allocated——找不全时靠 `undo_publish_attempt`
    // 按同一批索引改回 Free，不靠“没找到就不会被当成已分配”这种隐式假设。
    let new_fixpoint = allocate_lowest_single_slots(pool, pool.fixpoint_cost);
    if let Some(slots) = &new_fixpoint {
        for &index in slots {
            pool.slots[index] = SlotState::Allocated;
        }
    }
    let new_object_slots = object_allocate.map(|units| allocate_lowest_data_units(pool, units));
    if let Some(Some(slots)) = &new_object_slots {
        for &index in slots {
            pool.slots[index] = SlotState::Allocated;
        }
    }
    let new_instance_table_slots = if rewrite_instance_table {
        let found = allocate_lowest_single_slots(pool, 2);
        if let Some(slots) = &found {
            for &index in slots {
                pool.slots[index] = SlotState::Allocated;
            }
        }
        Some(found)
    } else {
        None
    };
    let allocation_ok = new_fixpoint.is_some()
        && new_object_slots.as_ref().is_none_or(Option::is_some)
        && new_instance_table_slots.as_ref().is_none_or(Option::is_some);
    if !allocation_ok {
        pool.allocation_failure_hit = true;
        undo_publish_attempt(
            pool,
            &old_fixpoint,
            txg,
            &released_object_slots,
            object_release,
            rewrite_instance_table,
            &old_instance_table_slots,
            new_fixpoint,
            new_object_slots.flatten(),
            new_instance_table_slots.flatten(),
        );
        return Err(PublishAttemptFailed);
    }
    let new_fixpoint = new_fixpoint.expect("已判空");
    let new_object_slots = new_object_slots.flatten();
    let new_instance_table_slots = new_instance_table_slots.flatten();
    // ③ 写根：注入失败时整份退回，不持久（R8：探针写写得进去，瞬时失败，走实例切换）。
    // txg 仍要推进一格：D23（journal 的角色与格式） 已定项 14 逐字「根槽写失败重发时 checkpoint_txg
    // 推进一格再发（根环区域 = txg mod R）……不推进就是反复重写同一个槽、把 R 个失败域用成一个」。
    // 不推进的话（见登记「十二、修订」），
    // 下一次重试会拿同一个 txg 再写同一个槽——H1 从没注入过失败，只有 H5 的历史会走到这里。
    if fail {
        undo_publish_attempt(
            pool,
            &old_fixpoint,
            txg,
            &released_object_slots,
            object_release,
            rewrite_instance_table,
            &old_instance_table_slots,
            Some(new_fixpoint),
            new_object_slots,
            new_instance_table_slots,
        );
        pool.next_txg += 1;
        return Err(PublishAttemptFailed);
    }
    // 分配确认：登记数据对象、Q6 安全检查（在回收之前，回收之后候选集会变化）。
    let mut all_allocated = new_fixpoint.clone();
    if let Some(units) = object_allocate {
        let slots = new_object_slots.clone().expect("已判空");
        let object_identifier = pool.next_object_identifier;
        pool.next_object_identifier += 1;
        pool.object_slots.insert(object_identifier, slots.clone());
        all_allocated.extend(slots);
        let _ = units;
    }
    if let Some(object_identifier) = object_release {
        pool.object_slots.remove(&object_identifier);
    }
    if let Some(slots) = &new_instance_table_slots {
        all_allocated.extend(slots.iter().copied());
    }
    let unsafe_reuses = count_unsafe_reuses(pool, &all_allocated);
    pool.unsafe_candidate_reuse_count += unsafe_reuses;
    pool.current_fixpoint_slots = new_fixpoint;
    if let Some(slots) = new_instance_table_slots {
        pool.current_instance_table_slots = slots;
    }
    if let Some(rows) = new_instance_rows {
        pool.instance_table = rows;
    }
    // ④ 持久之后：先算这条根的记账行，再登记进环，再回收（回收依赖环里已经有这条根）。
    let ledger_floor = match reclaim {
        ReclaimTiming::AfterEffectiveFloor => pool.effective_rollback_floor(),
        ReclaimTiming::HoldUntilCovered | ReclaimTiming::HoldUntilCoveredDouble => floor,
    };
    let oldest_before_this_root = pool.oldest_valid_root_txg();
    let ledger_threshold = ledger_floor.max(oldest_before_this_root);
    let occupied_pub = pool
        .slots
        .iter()
        .filter(|state| {
            matches!(state, SlotState::Allocated)
                || matches!(state, SlotState::Released { released_at_txg, .. } if *released_at_txg > ledger_threshold)
        })
        .count() as u64;
    let root = RootRecord {
        txg,
        instance: pool.instance,
        rollback_floor: floor,
        non_empty,
        occupied_pub,
        slots_snapshot: Rc::new(pool.slots.clone()),
        fixpoint_slots_snapshot: Rc::new(pool.current_fixpoint_slots.clone()),
        instance_table_slots_snapshot: Rc::new(pool.current_instance_table_slots.clone()),
    };
    let ring_index = pool.ring_index(txg);
    pool.ring[ring_index] = Some(root);
    pool.next_txg += 1;
    reclaim_after_publish(pool, reclaim);
    assert_conservation(pool, "attempt_publish 持久之后（I6）");
    Ok(txg)
}

/// I6：空闲 + 仍分配 + 已释放 + 扣住 = C，每个事件之后断言（登记「七」）。
pub(super) fn assert_conservation(pool: &Pool, where_: &str) {
    let mut free = 0u64;
    let mut allocated = 0u64;
    let mut released = 0u64;
    let mut held = 0u64;
    for state in &pool.slots {
        match state {
            SlotState::Free => free += 1,
            SlotState::Allocated => allocated += 1,
            SlotState::Released { .. } => released += 1,
            SlotState::Held { .. } => held += 1,
        }
    }
    let total = free + allocated + released + held;
    assert_eq!(
        total, pool.capacity_slots,
        "{where_}：空闲{free} + 仍分配{allocated} + 已释放{released} + 扣住{held} = {total}，应等于容量 {}",
        pool.capacity_slots
    );
}

#[allow(clippy::too_many_arguments, reason = "退回要精确对称地撤销分配与释放的每一步，参数与 attempt_publish 一一对应")]
fn undo_publish_attempt(
    pool: &mut Pool,
    old_fixpoint: &[usize],
    txg: u64,
    released_object_slots: &Option<Vec<usize>>,
    object_release: Option<u64>,
    rewrite_instance_table: bool,
    old_instance_table_slots: &[usize],
    tentative_fixpoint: Option<Vec<usize>>,
    tentative_object_slots: Option<Vec<usize>>,
    tentative_instance_table_slots: Option<Vec<usize>>,
) {
    let _ = txg;
    for &index in old_fixpoint {
        pool.slots[index] = SlotState::Allocated;
    }
    if let Some(slots) = released_object_slots {
        for &index in slots {
            pool.slots[index] = SlotState::Allocated;
        }
    }
    let _ = object_release;
    if rewrite_instance_table {
        for &index in old_instance_table_slots {
            pool.slots[index] = SlotState::Allocated;
        }
    }
    if let Some(slots) = tentative_fixpoint {
        for index in slots {
            pool.slots[index] = SlotState::Free;
        }
    }
    if let Some(slots) = tentative_object_slots {
        for index in slots {
            pool.slots[index] = SlotState::Free;
        }
    }
    if let Some(slots) = tentative_instance_table_slots {
        for index in slots {
            pool.slots[index] = SlotState::Free;
        }
    }
}

// ===== 推空一次 / 实例切换（5.3 推空一次；D23（journal 的角色与格式） 已定项 14 R8） =====

/// 推空一次：所带 F = max(F_顶, 目标)；目标 = min(已释放槽里 g 的最大值, 抬 F 上限)。
/// PC1（阳性对照）：改坏时目标恒取 F_顶，推空照发但 F 不抬。
fn push_target_floor(pool: &Pool) -> u64 {
    let newest_persisted_rollback_floor = pool.newest_persisted_floor();
    if pool.damage.disable_floor_raise {
        return newest_persisted_rollback_floor;
    }
    let ceiling = pool.rollback_floor_ceiling().unwrap_or(newest_persisted_rollback_floor);
    let maximum_released_txg = pool
        .slots
        .iter()
        .filter_map(|state| match state {
            SlotState::Released { released_at_txg, .. } => Some(*released_at_txg),
            SlotState::Free | SlotState::Allocated | SlotState::Held { .. } => None,
        })
        .max()
        .unwrap_or(0);
    newest_persisted_rollback_floor.max(maximum_released_txg.min(ceiling))
}

/// P-止：推空之前，目标 ≤ F_顶 且 F_生效 = F_顶（没有东西可抬、也没有抬到一半）就停。
pub(super) fn stop_policy_says_stop(pool: &Pool, stop: StopPolicy) -> bool {
    match stop {
        StopPolicy::FillBudget => false,
        StopPolicy::StopAtTarget => {
            let newest_persisted_rollback_floor = pool.newest_persisted_floor();
            let target = push_target_floor(pool);
            target <= newest_persisted_rollback_floor && pool.effective_rollback_floor() == newest_persisted_rollback_floor
        }
    }
}

/// 一次推空发布，可能因为注入的根槽写失败而走实例切换（R8）。返回本次消耗的发布尝试数。
/// 正常情况下写行／暖机只会因为注入的根槽写失败而失败（R8 的「探针写写得进去」前提）；
/// 阳性对照 PC5（保留量清零）会让固定点分配本身也真的失败——那种时候不许 `.expect()`，
/// 只能把 `pool.allocation_failure_hit`（`attempt_publish` 已经置位）当卡死信号交回给调用方。
///
/// `failures` 在这次推空自己的根槽写之前被问一次（参数是这次推空即将发生之前的 `last_persisted_txg`，
/// 供 `LazyFailureInjector` 现算 `window_start` 用；`FailureInjector` 忽略它）；H5 要求
/// 「切换的写行发布的根也算在连续失败里」（登记「五、5.4」H5-k-位 定义），所以这个闭包要一路带进
/// `perform_instance_switch`，覆盖写行与暖机自己的根槽写——不能只在这里问一次就把结果存成 `bool`
/// 往下传，那样切换内部的每一次根槽写都会读到同一个决定。
pub(super) fn push_one_empty_publish(pool: &mut Pool, reclaim: ReclaimTiming, failures: &mut dyn FnMut(u64) -> bool) -> u64 {
    pool.pushes_so_far += 1;
    let new_floor = push_target_floor(pool);
    maybe_hold_before_publish(pool, reclaim, new_floor);
    let inject_failure = failures(last_persisted_txg(pool));
    let result = attempt_publish(pool, reclaim, false, new_floor, None, None, false, None, inject_failure);
    match result {
        Ok(_) => 1,
        Err(PublishAttemptFailed) if pool.allocation_failure_hit => 1,
        Err(PublishAttemptFailed) => 1 + perform_instance_switch(pool, reclaim, failures),
    }
}

/// 实例切换（D23（journal 的角色与格式） 已定项 14）：取新实例、写行发布（带恢复后生效的 F）、
/// 暖机到本实例的根覆盖两块盘。返回消耗的发布尝试数（写行 + 暖机次数，遇到 `pool.allocation_failure_hit`
/// 提前停，把已经用掉的尝试数如实报回去）。
///
/// 写行与暖机自己的根槽写也会问 `failures`：一条根写失败按 R8「一律当瞬时失败 ⇒ 实例切换」，
/// 不分是推空、写行还是暖机——所以写行或暖机失败时递归再走一次切换，不是直接放弃。
/// `switch_reserve_shares_per_disk` 只在一条切换链**真正落地**（走到函数末尾、不是中途递归出去）
/// 时才扣一份：连续失败触发的中间那几次尝试算在 `switches_so_far`（诊断计数，无条件 +1）里，
/// 不重复扣预留——预留对应的是「挂载期间发生过几次可用的实例切换」，不是「尝试了几次根槽写」。
pub(super) fn perform_instance_switch(pool: &mut Pool, reclaim: ReclaimTiming, failures: &mut dyn FnMut(u64) -> bool) -> u64 {
    pool.switches_so_far += 1;
    let previous_instance = pool.instance;
    let new_instance = previous_instance + 1;
    let effective_floor = pool.effective_rollback_floor();
    let first_row_instance = 1u64.max(previous_instance);
    let mut rows = pool.instance_table.clone();
    for row_instance in first_row_instance..new_instance {
        rows.push(InstanceRow {
            instance: row_instance,
            selected_root_txg: 0,
            applied_transaction_high_water: 0,
            is_rollback: false,
        });
    }
    pool.instance = new_instance;
    let inject_row_failure = failures(last_persisted_txg(pool));
    let row_result = attempt_publish(pool, reclaim, false, effective_floor, None, None, true, Some(rows), inject_row_failure);
    let row_txg = match row_result {
        Ok(txg) => txg,
        Err(PublishAttemptFailed) if pool.allocation_failure_hit => return 1,
        Err(PublishAttemptFailed) => return 1 + perform_instance_switch(pool, reclaim, failures),
    };
    let mut attempts = 1u64;
    let mut covered = vec![device_of_txg(row_txg)];
    let all_devices = [0u64, 1u64];
    while all_devices.iter().any(|device| !covered.contains(device)) && attempts - 1 < RING_REGIONS {
        let current_floor = pool.newest_persisted_floor();
        let inject_warm_failure = failures(last_persisted_txg(pool));
        let warm_result = attempt_publish(pool, reclaim, false, current_floor, None, None, false, None, inject_warm_failure);
        attempts += 1;
        let warm_txg = match warm_result {
            Ok(txg) => txg,
            Err(PublishAttemptFailed) if pool.allocation_failure_hit => break,
            Err(PublishAttemptFailed) => {
                attempts += perform_instance_switch(pool, reclaim, failures);
                break;
            }
        };
        let device = device_of_txg(warm_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
    }
    pool.switch_reserve_shares_per_disk = pool.switch_reserve_shares_per_disk.saturating_sub(1);
    attempts
}
