//! 池的状态与它的损坏开关、池上的槽、根环与回滚下限的操作。

use super::*;

// ===== 单次跑的可变状态 =====

pub(super) struct Pool {
    pub(super) capacity_slots: u64,
    pub(super) slots: Vec<SlotState>,
    pub(super) isolated: Vec<bool>,
    pub(super) ring: Vec<Option<RootRecord>>,
    pub(super) slots_per_region: u64,
    pub(super) instance: u64,
    pub(super) instance_table: Vec<InstanceRow>,
    pub(super) fixpoint_cost: u64,
    pub(super) current_fixpoint_slots: Vec<usize>,
    pub(super) current_instance_table_slots: Vec<usize>,
    pub(super) object_slots: BTreeMap<u64, Vec<usize>>,
    pub(super) next_object_identifier: u64,
    pub(super) switch_reserve_shares_per_disk: u64,
    pub(super) pending_hold_floor: Option<u64>,
    pub(super) next_txg: u64,
    // 累计计数
    pub(super) allocation_failure_hit: bool,
    pub(super) unsafe_candidate_reuse_count: u64,
    pub(super) switches_so_far: u64,
    pub(super) pushes_so_far: u64,
    pub(super) publishes_so_far: u64,
    pub(super) filler_objects_written_so_far: u64,
    pub(super) df_reported_enough_but_write_failed_count: u64,
    pub(super) maximum_publish_attempts_seen: u64,
    pub(super) budget_exhausted_hit: bool,
    pub(super) damage: Damage,
}

/// 阳性对照（5.6）：每种改坏各自一个开关，起点状态里全部关闭（不改坏）。
#[derive(Clone, Copy, Default, Debug)]
pub(super) struct Damage {
    /// PC1：推空所带 F 恒取 F_顶（推空照发，F 不抬）。
    pub(super) disable_floor_raise: bool,
    /// PC2：`df` 一律报 U_all + Rel，不扣任何保留、残留、滞后量、预留、在飞。
    pub(super) raw_df: bool,
    /// PC3：一次准入的发布预算改成这个值（默认 `RETRY_BUDGET_PUBLICATIONS`）。
    pub(super) budget_override: Option<u64>,
    /// PC4（G7 一族）：分配器在第一条带新 F 的根持久时就回收，不等两块盘都覆盖。
    pub(super) reclaim_on_first_new_floor_root: bool,
    /// PC4（F-扣 一族）：回收之后不扣住（等价于把 F-扣 当 G7 用，只是不切换回收公式）。
    pub(super) skip_hold: bool,
    /// PC5：checkpoint 保留池、推空保留池、残留、M_sw 全部按 0 算（只影响 gate2 与 df，
    /// 不改变实际分配预算——本模型里固定点分配本身不读这些量，卡死由分配失败触发）。
    pub(super) zero_reserves: bool,
}

impl Pool {
    pub(super) fn ring_index(&self, txg: u64) -> usize {
        let region = region_of_txg(txg);
        let slot = (txg / RING_REGIONS) % self.slots_per_region;
        usize::try_from(region * self.slots_per_region + slot).expect("环索引")
    }

    pub(super) fn seed(geometry: Geometry) -> Self {
        let capacity = geometry.capacity_slots_per_disk;
        let mut slots = vec![SlotState::Free; usize::try_from(capacity).expect("容量")];
        let fixpoint_slots: Vec<usize> = (0..usize::try_from(geometry.fixpoint_cost).expect("c_max")).collect();
        let instance_table_slots: Vec<usize> = vec![fixpoint_slots.len(), fixpoint_slots.len() + 1];
        for &index in &fixpoint_slots {
            slots[index] = SlotState::Allocated;
        }
        for &index in &instance_table_slots {
            slots[index] = SlotState::Allocated;
        }
        let occupied_pub = geometry.fixpoint_cost + 2;
        let ring_length = usize::try_from(RING_REGIONS * geometry.slots_per_region).expect("环长");
        let mut ring: Vec<Option<RootRecord>> = vec![None; ring_length];
        let slots_snapshot = Rc::new(slots.clone());
        let fixpoint_slots_snapshot = Rc::new(fixpoint_slots.clone());
        let instance_table_slots_snapshot = Rc::new(instance_table_slots.clone());
        let root0 = RootRecord {
            txg: 0,
            instance: 0,
            rollback_floor: 0,
            non_empty: false,
            occupied_pub,
            slots_snapshot: slots_snapshot.clone(),
            fixpoint_slots_snapshot: fixpoint_slots_snapshot.clone(),
            instance_table_slots_snapshot: instance_table_slots_snapshot.clone(),
        };
        let root1 = RootRecord {
            txg: 1,
            instance: 1,
            ..root0.clone()
        };
        let root2 = RootRecord {
            txg: 2,
            instance: 1,
            ..root0.clone()
        };
        let index0 = usize::try_from(region_of_txg(0) * geometry.slots_per_region).expect("环索引");
        let index1 = usize::try_from(region_of_txg(1) * geometry.slots_per_region).expect("环索引");
        let index2 = usize::try_from(region_of_txg(2) * geometry.slots_per_region).expect("环索引");
        ring[index0] = Some(root0);
        ring[index1] = Some(root1);
        ring[index2] = Some(root2);
        Pool {
            capacity_slots: capacity,
            slots,
            isolated: vec![false; usize::try_from(capacity).expect("容量")],
            ring,
            slots_per_region: geometry.slots_per_region,
            instance: 1,
            instance_table: Vec::new(),
            fixpoint_cost: geometry.fixpoint_cost,
            current_fixpoint_slots: fixpoint_slots,
            current_instance_table_slots: instance_table_slots,
            object_slots: BTreeMap::new(),
            next_object_identifier: 0,
            switch_reserve_shares_per_disk: INSTANCE_SWITCH_LIMIT,
            pending_hold_floor: None,
            next_txg: 3,
            allocation_failure_hit: false,
            unsafe_candidate_reuse_count: 0,
            switches_so_far: 0,
            pushes_so_far: 0,
            publishes_so_far: 0,
            filler_objects_written_so_far: 0,
            df_reported_enough_but_write_failed_count: 0,
            maximum_publish_attempts_seen: 0,
            budget_exhausted_hit: false,
            damage: Damage::default(),
        }
    }

    pub(super) fn readable_roots(&self) -> impl Iterator<Item = &RootRecord> {
        self.ring.iter().filter_map(|slot| slot.as_ref())
    }

    pub(super) fn is_abandoned(&self, root: &RootRecord) -> bool {
        self.instance_table.iter().any(|row| {
            row.instance == root.instance && root.txg > row.selected_root_txg
        })
    }

    pub(super) fn oldest_valid_root_txg(&self) -> u64 {
        self.readable_roots()
            .filter(|root| !self.is_abandoned(root))
            .map(|root| root.txg)
            .min()
            .unwrap_or(0)
    }

    pub(super) fn effective_rollback_floor(&self) -> u64 {
        let mut highest_per_device: BTreeMap<u64, u64> = BTreeMap::new();
        for root in self.readable_roots() {
            let device = device_of_txg(root.txg);
            let entry = highest_per_device.entry(device).or_insert(root.rollback_floor);
            *entry = (*entry).max(root.rollback_floor);
        }
        highest_per_device.values().copied().min().unwrap_or(0)
    }

    /// 抬 F 的上限（D16（发布语义） 已定项 1）。返回 None 只在没有任何有效根时发生（起点状态之后不会）。
    pub(super) fn rollback_floor_ceiling(&self) -> Option<u64> {
        let current_floor = self
            .readable_roots()
            .filter(|root| !self.is_abandoned(root))
            .map(|root| root.rollback_floor)
            .max()
            .unwrap_or(0);
        let valid: Vec<&RootRecord> = self
            .readable_roots()
            .filter(|root| root.txg >= current_floor)
            .filter(|root| !self.is_abandoned(root))
            .collect();
        if valid.is_empty() {
            return None;
        }
        let mut newest_per_device: BTreeMap<u64, u64> = BTreeMap::new();
        for root in &valid {
            let device = device_of_txg(root.txg);
            let entry = newest_per_device.entry(device).or_insert(root.txg);
            *entry = (*entry).max(root.txg);
        }
        let newest_on_every_device = *newest_per_device.values().min()?;
        let mut non_empty: Vec<u64> = valid
            .iter()
            .filter(|root| root.non_empty)
            .map(|root| root.txg)
            .collect();
        non_empty.sort_unstable_by(|left, right| right.cmp(left));
        let fourth_newest = non_empty
            .get(3)
            .copied()
            .unwrap_or_else(|| valid.iter().map(|root| root.txg).min().expect("valid 非空"));
        Some(newest_on_every_device.min(fourth_newest))
    }

    pub(super) fn newest_persisted_floor(&self) -> u64 {
        self.readable_roots()
            .max_by_key(|root| root.txg)
            .map(|root| root.rollback_floor)
            .unwrap_or(0)
    }

    pub(super) fn latest_occupied_pub(&self) -> u64 {
        self.readable_roots()
            .max_by_key(|root| root.txg)
            .map(|root| root.occupied_pub)
            .unwrap_or(0)
    }

    pub(super) fn switch_reserve_slots(&self) -> u64 {
        self.switch_reserve_shares_per_disk * (2 + RING_REGIONS * self.fixpoint_cost)
    }

    pub(super) fn released_slot_count(&self) -> u64 {
        self.slots
            .iter()
            .filter(|state| matches!(state, SlotState::Released { .. }))
            .count() as u64
    }

    pub(super) fn held_slot_count(&self) -> u64 {
        self.slots
            .iter()
            .filter(|state| matches!(state, SlotState::Held { .. }))
            .count() as u64
    }

    /// U：可发的空闲槽（空闲 ∧ 不隔离 ∧ 不扣住）。扣住不是 `SlotState::Free`，天然被排除。
    pub(super) fn free_count_allocatable(&self) -> u64 {
        self.slots
            .iter()
            .enumerate()
            .filter(|(index, state)| matches!(state, SlotState::Free) && !self.isolated[*index])
            .count() as u64
    }

    /// U_all：全部空闲槽（含隔离与扣住）——隔离位不改变槽是不是 `Free`，扣住单独加回。
    pub(super) fn free_count_all(&self) -> u64 {
        let free_regardless_of_isolation = self
            .slots
            .iter()
            .filter(|state| matches!(state, SlotState::Free))
            .count() as u64;
        free_regardless_of_isolation + self.held_slot_count()
    }

    pub(super) fn isolated_count(&self) -> u64 {
        self.isolated.iter().filter(|flag| **flag).count() as u64
    }

    pub(super) fn lag_count(&self) -> u64 {
        let ceiling = self.rollback_floor_ceiling().unwrap_or(0);
        let oldest = self.oldest_valid_root_txg();
        let threshold = ceiling.max(oldest);
        self.slots
            .iter()
            .filter(|state| {
                matches!(
                    state,
                    SlotState::Released { released_at_txg, from_empty_publish }
                        if *released_at_txg > threshold && !from_empty_publish
                )
            })
            .count() as u64
    }
}

#[cfg(test)]
#[path = "pool_state_and_damage_tests.rs"]
mod tests;
