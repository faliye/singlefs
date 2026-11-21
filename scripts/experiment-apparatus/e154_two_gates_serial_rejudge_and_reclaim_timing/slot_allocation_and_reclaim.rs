//! 最低槽的分配、数据单元的分配与释放、发布之后的回收、发布前的保留与不安全复用的计数。

use super::*;

// ===== 分配、释放、发布（5.1 一次发布 ①②③④） =====

/// 取 `count` 个槽号最低的可发单槽（R12）：固定点与实例表用，不要求成对。
pub(super) fn allocate_lowest_single_slots(pool: &Pool, count: u64) -> Option<Vec<usize>> {
    let mut found = Vec::new();
    for (index, state) in pool.slots.iter().enumerate() {
        if matches!(state, SlotState::Free) && !pool.isolated[index] {
            found.push(index);
            if found.len() as u64 == count {
                return Some(found);
            }
        }
    }
    None
}

/// 取 `unit_count` 个数据单元，每个占槽号最低的一对相邻偶数起点空槽（R12）。
pub(super) fn allocate_lowest_data_units(pool: &Pool, unit_count: u64) -> Option<Vec<usize>> {
    let capacity = usize::try_from(pool.capacity_slots).expect("容量");
    let mut found = Vec::new();
    let mut start = 0usize;
    while start + 1 < capacity && (found.len() as u64) < unit_count * 2 {
        let pair_free = matches!(pool.slots[start], SlotState::Free)
            && !pool.isolated[start]
            && matches!(pool.slots[start + 1], SlotState::Free)
            && !pool.isolated[start + 1];
        if pair_free {
            found.push(start);
            found.push(start + 1);
        }
        start += 2;
    }
    if found.len() as u64 == unit_count * 2 {
        Some(found)
    } else {
        None
    }
}

pub(super) fn release_slots(pool: &mut Pool, slots: &[usize], released_at_txg: u64, from_empty_publish: bool) {
    for &index in slots {
        pool.slots[index] = SlotState::Released {
            released_at_txg,
            from_empty_publish,
        };
    }
}

/// 一次发布持久之后的回收（5.3「回收时点」）：ring-rotation 那一半（g ≤ 环里最旧有效根）
/// 两种回收时点都直接生效；G7 再按 max(F_生效, 环里最旧有效根) 回收；F-扣／F-双扣 只在
/// 扣住到期（`pending_hold_floor` 达到 F_生效）时把扣住的槽放开。
pub(super) fn reclaim_after_publish(pool: &mut Pool, reclaim: ReclaimTiming) {
    let oldest = pool.oldest_valid_root_txg();
    match reclaim {
        ReclaimTiming::AfterEffectiveFloor => {
            // PC4（G7 一族）：改坏时用 F_顶（最新持久根自己带的 F）代替 F_生效，
            // 等于在第一条带新 F 的根持久时就回收，不等两块盘都覆盖。
            let effective = if pool.damage.reclaim_on_first_new_floor_root {
                pool.newest_persisted_floor()
            } else {
                pool.effective_rollback_floor()
            };
            let threshold = effective.max(oldest);
            for state in pool.slots.iter_mut() {
                if let SlotState::Released { released_at_txg, .. } = state {
                    if *released_at_txg <= threshold {
                        *state = SlotState::Free;
                    }
                }
            }
        }
        ReclaimTiming::HoldUntilCovered | ReclaimTiming::HoldUntilCoveredDouble => {
            for state in pool.slots.iter_mut() {
                if let SlotState::Released { released_at_txg, .. } = state {
                    if *released_at_txg <= oldest {
                        *state = SlotState::Free;
                    }
                }
            }
            if let Some(hold_floor) = pool.pending_hold_floor {
                if pool.effective_rollback_floor() >= hold_floor {
                    for state in pool.slots.iter_mut() {
                        if let SlotState::Held { .. } = state {
                            *state = SlotState::Free;
                        }
                    }
                    pool.pending_hold_floor = None;
                }
            }
        }
    }
}

/// F-扣／F-双扣：这次发布要带的 F 大于 F_顶 时，先把 g ≤ 新 F 的已释放槽扣住（登记「五、5.3」）。
/// PC4（F-扣 一族）：改坏时跳过扣住，直接把这批槽变空闲——复现 `mount.rs` 注释里
/// 「不扣住时抬 F 自己的空发布落在刚回收的槽上」那个缺陷。
pub(super) fn maybe_hold_before_publish(pool: &mut Pool, reclaim: ReclaimTiming, new_floor: u64) {
    if !reclaim.holds_before_release() {
        return;
    }
    let newest_persisted_rollback_floor = pool.newest_persisted_floor();
    if new_floor <= newest_persisted_rollback_floor {
        return;
    }
    if pool.damage.skip_hold {
        for state in pool.slots.iter_mut() {
            if let SlotState::Released { released_at_txg, .. } = *state {
                if released_at_txg <= new_floor {
                    *state = SlotState::Free;
                }
            }
        }
        return;
    }
    for state in pool.slots.iter_mut() {
        if let SlotState::Released {
            released_at_txg,
            from_empty_publish,
        } = *state
        {
            if released_at_txg <= new_floor {
                *state = SlotState::Held {
                    released_at_txg,
                    from_empty_publish,
                };
            }
        }
    }
    pool.pending_hold_floor = Some(new_floor);
}

/// 一次发布持久之后，Q6 安全检查：这次分配的槽有没有落在候选根还引用的槽上
/// （候选集 = 持久∧有效∧txg≥F_生效 的根，加上环里还在的被抛弃根，D23（journal 的角色与格式） 已定项 14）。
pub(super) fn count_unsafe_reuses(pool: &Pool, allocated: &[usize]) -> u64 {
    let effective = pool.effective_rollback_floor();
    let candidates: Vec<&RootRecord> = pool
        .readable_roots()
        .filter(|root| {
            let abandoned = pool.is_abandoned(root);
            (root.txg >= effective && !abandoned) || abandoned
        })
        .collect();
    let mut hits = 0u64;
    for &index in allocated {
        for root in &candidates {
            if let Some(state) = root.slots_snapshot.get(index) {
                if matches!(state, SlotState::Allocated) {
                    hits += 1;
                    break;
                }
            }
        }
    }
    hits
}
