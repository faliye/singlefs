//! E154 两道闸串-重判与回收时点的代价。
//!
//! 跑前登记：`research/prompts/e154-preregistration.md`。纯计数模型：没有 I/O、没有并发、
//! 没有随机源，同一份输入跑 N 遍必然一样；证据强度来自本文件的绝对值单测与变异表
//! （`research/mutations/e154_two_gates_serial_rejudge_and_reclaim_timing.tsv`），不靠重跑轮数。
//!
//! 输出体量说明（技术实现细节，不改登记「六、七、九」的判据）：六节要求「臂 × 历史 × 格」
//! 逐格报判定，本文件对全部 48 格 × 24 臂 × 12 历史都算出并报出 `RUN` 汇总行（Q1–Q8 与
//! 几何一起报）；`ROW`（逐次尝试）明细只对真实基线（串-重判 × G7，两种停法）、全部阳性对照、
//! 以及任何一次触发失败条款的跑打印——全仓现存最大产物 371 KB
//! （`research/results/e47-ring-loss-2026-08-30.out`），13824 次跑的全量 ROW 明细会把产物撑到
//! 数十 MB，不是这个仓的产物体量。RUN 行本身覆盖了六节「分辨格」要求的全部判定，没有被这项
//! 调整削弱；I1–I10、A1–A6 由单测钉死，不依赖 ROW 明细是否打印。

use e7_index_bench::Emitter;
use std::collections::BTreeMap;
use std::rc::Rc;

// ===== 常量（登记「读条款时读到的数」「判问法时自己算出来的」） =====

/// 根环区域数 R（D22（单元原子性怎么合成） 已定项 2）。
const RING_REGIONS: u64 = 3;
/// 区域设备归属（D2（RAID 条带策略） 已定项 7，`crates/singlefs-format/src/lib.rs:192` 同值）。
const REGION_DEVICES: [u64; 3] = [0, 1, 0];
/// 一次挂载允许的连续实例切换次数上限（D23（journal 的角色与格式） 已定项 14）。
const INSTANCE_SWITCH_LIMIT: u64 = 3;
/// 一次处置容忍的根槽写失败次数（D16（发布语义） 已定项 1）。
const TOLERATED_ROOT_WRITE_FAILURE_COUNT: u64 = 2;
/// 一次准入最多的发布尝试预算 B = 4 + 2 × k_tol（D16（发布语义） 已定项 1）。
const RETRY_BUDGET_PUBLICATIONS: u64 = 4 + 2 * TOLERATED_ROOT_WRITE_FAILURE_COUNT;
/// 最少保留的可退到的状态数（D16（发布语义） 已定项 1）。
const MINIMUM_RETAINED_STATES: u64 = 4;
/// 重试循环的最多触动次数 J_max = 3S + 5（登记「五、5.4」）。
fn retry_touch_budget(slots_per_region: u64) -> u64 {
    3 * slots_per_region + 5
}

// ===== 5.1 槽状态 =====

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SlotState {
    Free,
    Allocated,
    Released {
        released_at_txg: u64,
        from_empty_publish: bool,
    },
    Held {
        released_at_txg: u64,
        from_empty_publish: bool,
    },
}

// ===== 24 条臂：三个正交轴 =====

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum GateReading {
    Serial,
    SerialRejudge,
    Union,
    UnionPrime,
}

impl GateReading {
    fn label(self) -> &'static str {
        match self {
            GateReading::Serial => "串",
            GateReading::SerialRejudge => "串重判",
            GateReading::Union => "合",
            GateReading::UnionPrime => "合撇",
        }
    }
    /// 第一道闸不过时，这个读法会不会推空重判第一道闸。
    fn rejudges_first_gate(self) -> bool {
        matches!(self, GateReading::SerialRejudge)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum ReclaimTiming {
    AfterEffectiveFloor,
    HoldUntilCovered,
    HoldUntilCoveredPrime,
}

impl ReclaimTiming {
    fn label(self) -> &'static str {
        match self {
            ReclaimTiming::AfterEffectiveFloor => "G7",
            ReclaimTiming::HoldUntilCovered => "F扣",
            ReclaimTiming::HoldUntilCoveredPrime => "F扣撇",
        }
    }
    fn holds_before_release(self) -> bool {
        matches!(
            self,
            ReclaimTiming::HoldUntilCovered | ReclaimTiming::HoldUntilCoveredPrime
        )
    }
    /// F-扣撇 在第一道闸再减一次扣住槽数；F-扣 不减。
    fn first_gate_excludes_held(self) -> bool {
        matches!(self, ReclaimTiming::HoldUntilCoveredPrime)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum StopPolicy {
    StopAtTarget,
    FillBudget,
}

impl StopPolicy {
    fn label(self) -> &'static str {
        match self {
            StopPolicy::StopAtTarget => "止",
            StopPolicy::FillBudget => "满",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Arm {
    gate: GateReading,
    reclaim: ReclaimTiming,
    stop: StopPolicy,
}

impl Arm {
    fn label(self) -> String {
        format!(
            "{}-{}-{}",
            self.gate.label(),
            self.reclaim.label(),
            self.stop.label()
        )
    }
    fn is_real_baseline(self) -> bool {
        self.gate == GateReading::SerialRejudge && self.reclaim == ReclaimTiming::AfterEffectiveFloor
    }
}

fn all_arms() -> Vec<Arm> {
    let mut arms = Vec::new();
    for gate in [
        GateReading::Serial,
        GateReading::SerialRejudge,
        GateReading::Union,
        GateReading::UnionPrime,
    ] {
        for reclaim in [
            ReclaimTiming::AfterEffectiveFloor,
            ReclaimTiming::HoldUntilCovered,
            ReclaimTiming::HoldUntilCoveredPrime,
        ] {
            for stop in [StopPolicy::StopAtTarget, StopPolicy::FillBudget] {
                arms.push(Arm { gate, reclaim, stop });
            }
        }
    }
    arms
}

// ===== 几何格（5.5） =====

#[derive(Clone, Copy, Debug)]
struct Geometry {
    capacity_slots_per_disk: u64,
    object_units: u64,
    fixpoint_cost: u64,
    slots_per_region: u64,
    padding_deletes: u64,
}

fn all_geometries() -> Vec<Geometry> {
    let mut geometries = Vec::new();
    for capacity_slots_per_disk in [600u64, 4000u64] {
        for object_units in [1u64, 16u64] {
            for fixpoint_cost in [4u64, 9u64] {
                for slots_per_region in [4u64, 8u64] {
                    for padding_deletes in [0u64, 1u64, 2u64] {
                        geometries.push(Geometry {
                            capacity_slots_per_disk,
                            object_units,
                            fixpoint_cost,
                            slots_per_region,
                            padding_deletes,
                        });
                    }
                }
            }
        }
    }
    geometries
}

// ===== 持久根记录（5.1） =====

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct InstanceRow {
    instance: u64,
    selected_root_txg: u64,
    applied_transaction_high_water: u64,
    is_rollback: bool,
}

/// 2026-09-17（第二段跑之前）加了两个字段，专供 H6（管理员回退）用：回退要把
/// `pool.current_fixpoint_slots` / `pool.current_instance_table_slots` 也载回 R_old 那一刻的值，
/// 不能靠"槽状态快照里最低的几个 Allocated 槽"去猜——那是个只在"分配器总从最低空槽发"这条
/// 不变量下才成立的假设，猜错了下一次发布释放的就不是 R_old 真正持有的那几个槽。
/// H4／H5 的崩溃-恢复分支仍不实现（见文件头「输出体量说明」旁的范围记录），对象表快照
/// 仍然留给以后接上那两类历史时再加。
#[derive(Clone)]
struct RootRecord {
    txg: u64,
    instance: u64,
    rollback_floor: u64,
    non_empty: bool,
    occupied_pub: u64,
    slots_snapshot: Rc<Vec<SlotState>>,
    fixpoint_slots_snapshot: Rc<Vec<usize>>,
    instance_table_slots_snapshot: Rc<Vec<usize>>,
}

fn region_of_txg(txg: u64) -> u64 {
    txg % RING_REGIONS
}

fn device_of_txg(txg: u64) -> u64 {
    REGION_DEVICES[usize::try_from(region_of_txg(txg)).expect("区域号")]
}

// ===== 单次跑的可变状态 =====

struct Pool {
    capacity_slots: u64,
    slots: Vec<SlotState>,
    isolated: Vec<bool>,
    ring: Vec<Option<RootRecord>>,
    slots_per_region: u64,
    instance: u64,
    instance_table: Vec<InstanceRow>,
    fixpoint_cost: u64,
    current_fixpoint_slots: Vec<usize>,
    current_instance_table_slots: Vec<usize>,
    object_slots: BTreeMap<u64, Vec<usize>>,
    next_object_identifier: u64,
    switch_reserve_shares_per_disk: u64,
    pending_hold_floor: Option<u64>,
    next_txg: u64,
    // 累计计数
    allocation_failure_hit: bool,
    unsafe_candidate_reuse_count: u64,
    switches_so_far: u64,
    pushes_so_far: u64,
    publishes_so_far: u64,
    filler_objects_written_so_far: u64,
    df_reported_enough_but_write_failed_count: u64,
    maximum_publish_attempts_seen: u64,
    budget_exhausted_hit: bool,
    damage: Damage,
}

/// 阳性对照（5.6）：每种改坏各自一个开关，起点状态里全部关闭（不改坏）。
#[derive(Clone, Copy, Default, Debug)]
struct Damage {
    /// PC1：推空所带 F 恒取 F_顶（推空照发，F 不抬）。
    disable_floor_raise: bool,
    /// PC2：`df` 一律报 U_all + Rel，不扣任何保留、残留、滞后量、预留、在飞。
    raw_df: bool,
    /// PC3：一次准入的发布预算改成这个值（默认 `RETRY_BUDGET_PUBLICATIONS`）。
    budget_override: Option<u64>,
    /// PC4（G7 一族）：分配器在第一条带新 F 的根持久时就回收，不等两块盘都覆盖。
    reclaim_on_first_new_floor_root: bool,
    /// PC4（F-扣 一族）：回收之后不扣住（等价于把 F-扣 当 G7 用，只是不切换回收公式）。
    skip_hold: bool,
    /// PC5：checkpoint 保留池、推空保留池、残留、M_sw 全部按 0 算（只影响 gate2 与 df，
    /// 不改变实际分配预算——本模型里固定点分配本身不读这些量，卡死由分配失败触发）。
    zero_reserves: bool,
}

impl Pool {
    fn ring_index(&self, txg: u64) -> usize {
        let region = region_of_txg(txg);
        let slot = (txg / RING_REGIONS) % self.slots_per_region;
        usize::try_from(region * self.slots_per_region + slot).expect("环索引")
    }

    fn seed(geometry: Geometry) -> Self {
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
        let index0 = usize::try_from(region_of_txg(0) * geometry.slots_per_region + 0).expect("环索引");
        let index1 = usize::try_from(region_of_txg(1) * geometry.slots_per_region + 0).expect("环索引");
        let index2 = usize::try_from(region_of_txg(2) * geometry.slots_per_region + 0).expect("环索引");
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

    fn readable_roots(&self) -> impl Iterator<Item = &RootRecord> {
        self.ring.iter().filter_map(|slot| slot.as_ref())
    }

    fn is_abandoned(&self, root: &RootRecord) -> bool {
        self.instance_table.iter().any(|row| {
            row.instance == root.instance && root.txg > row.selected_root_txg
        })
    }

    fn oldest_valid_root_txg(&self) -> u64 {
        self.readable_roots()
            .filter(|root| !self.is_abandoned(root))
            .map(|root| root.txg)
            .min()
            .unwrap_or(0)
    }

    fn effective_rollback_floor(&self) -> u64 {
        let mut highest_per_device: BTreeMap<u64, u64> = BTreeMap::new();
        for root in self.readable_roots() {
            let device = device_of_txg(root.txg);
            let entry = highest_per_device.entry(device).or_insert(root.rollback_floor);
            *entry = (*entry).max(root.rollback_floor);
        }
        highest_per_device.values().copied().min().unwrap_or(0)
    }

    /// 抬 F 的上限（D16（发布语义） 已定项 1）。返回 None 只在没有任何有效根时发生（起点状态之后不会）。
    fn rollback_floor_ceiling(&self) -> Option<u64> {
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

    fn newest_persisted_floor(&self) -> u64 {
        self.readable_roots()
            .max_by_key(|root| root.txg)
            .map(|root| root.rollback_floor)
            .unwrap_or(0)
    }

    fn latest_occupied_pub(&self) -> u64 {
        self.readable_roots()
            .max_by_key(|root| root.txg)
            .map(|root| root.occupied_pub)
            .unwrap_or(0)
    }

    fn switch_reserve_slots(&self) -> u64 {
        self.switch_reserve_shares_per_disk * (2 + RING_REGIONS * self.fixpoint_cost)
    }

    fn released_slot_count(&self) -> u64 {
        self.slots
            .iter()
            .filter(|state| matches!(state, SlotState::Released { .. }))
            .count() as u64
    }

    fn held_slot_count(&self) -> u64 {
        self.slots
            .iter()
            .filter(|state| matches!(state, SlotState::Held { .. }))
            .count() as u64
    }

    /// U：可发的空闲槽（空闲 ∧ 不隔离 ∧ 不扣住）。扣住不是 `SlotState::Free`，天然被排除。
    fn free_count_allocatable(&self) -> u64 {
        self.slots
            .iter()
            .enumerate()
            .filter(|(index, state)| matches!(state, SlotState::Free) && !self.isolated[*index])
            .count() as u64
    }

    /// U_all：全部空闲槽（含隔离与扣住）——隔离位不改变槽是不是 `Free`，扣住单独加回。
    fn free_count_all(&self) -> u64 {
        let free_regardless_of_isolation = self
            .slots
            .iter()
            .filter(|state| matches!(state, SlotState::Free))
            .count() as u64;
        free_regardless_of_isolation + self.held_slot_count()
    }

    fn isolated_count(&self) -> u64 {
        self.isolated.iter().filter(|flag| **flag).count() as u64
    }

    fn lag_count(&self) -> u64 {
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

// ===== 5.2 闸读的量 =====

struct GateSnapshot {
    allocatable_free_slots: u64,
    all_free_slots_including_isolated_and_held: u64,
    released: u64,
    live_meta: u64,
    checkpoint_reserve: u64,
    push_reserve: u64,
    #[allow(dead_code, reason = "登记「五、5.2」把 Res 列成闸读的量之一；这两个字段只在 gate_snapshot 内部参与 defined_free_space_formula_slots 的计算，ROW 级逐行输出没有实现（见文件头「输出体量说明」），所以字段构造完之后没有第二个读者——留着是为了这份 struct 仍然完整对应 5.2 那张表，不是遗留的死代码")]
    residue: u64,
    #[allow(dead_code, reason = "同上一个字段的理由：lag 对应登记里的滞后量，只在这里参与 defined_free_space_formula_slots 的运算")]
    lag: u64,
    in_flight: u64,
    abandoned_root_exclusive_slots: u64,
    occupied_pub: u64,
    switch_reserve: u64,
    held: u64,
    defined_free_space_formula_slots: u64,
}

fn gate_snapshot(pool: &Pool, in_flight: u64) -> GateSnapshot {
    let fixpoint_cost = pool.fixpoint_cost;
    let (push_reserve, residue, checkpoint_reserve, switch_reserve) = if pool.damage.zero_reserves {
        (0, 0, 0, 0)
    } else {
        (10 + 7 * fixpoint_cost, 5 + 6 * fixpoint_cost, fixpoint_cost, pool.switch_reserve_slots())
    };
    let allocatable_free_slots = pool.free_count_allocatable();
    let all_free_slots_including_isolated_and_held = pool.free_count_all();
    let released = pool.released_slot_count();
    let live_meta = fixpoint_cost;
    let lag = pool.lag_count();
    let abandoned_root_exclusive_slots = pool.isolated_count();
    let occupied_pub = pool.latest_occupied_pub();
    let held = pool.held_slot_count();
    let defined_free_space_formula_slots = if pool.damage.raw_df {
        all_free_slots_including_isolated_and_held + released
    } else {
        (all_free_slots_including_isolated_and_held + released + live_meta)
            .saturating_sub(push_reserve)
            .saturating_sub(residue)
            .saturating_sub(lag)
    };
    GateSnapshot {
        allocatable_free_slots,
        all_free_slots_including_isolated_and_held,
        released,
        live_meta,
        checkpoint_reserve,
        push_reserve,
        residue,
        lag,
        in_flight,
        abandoned_root_exclusive_slots,
        occupied_pub,
        switch_reserve,
        held,
        defined_free_space_formula_slots,
    }
}

/// 可用_串（D28（挂载期承诺量） 已定项 1，「已分配」读成占着、删掉 defer 待释放）。
fn gate_serial(snapshot: &GateSnapshot, pool: &Pool, arm: Arm) -> u64 {
    let held_penalty = if arm.reclaim.first_gate_excludes_held() {
        snapshot.held
    } else {
        0
    };
    pool.capacity_slots
        .saturating_sub(snapshot.occupied_pub)
        .saturating_sub(snapshot.switch_reserve)
        .saturating_sub(snapshot.abandoned_root_exclusive_slots)
        .saturating_sub(snapshot.checkpoint_reserve)
        .saturating_sub(snapshot.in_flight)
        .saturating_sub(held_penalty)
}

/// 可用_合 = 可用_串 + 已释放且 g ≤ 抬 F 上限的槽数（登记「五、5.2」）。
/// `serial` 已经按 F-扣′ 扣过一次扣住槽数（`gate_serial` 对 `arm.reclaim` 独立判断，
/// 与 `arm.gate` 是哪一种读法无关），这里不重复扣。
fn gate_union(pool: &Pool, serial: u64) -> u64 {
    let ceiling = pool.rollback_floor_ceiling().unwrap_or(0);
    let creditable = pool
        .slots
        .iter()
        .filter(|state| {
            matches!(state, SlotState::Released { released_at_txg, .. } if *released_at_txg <= ceiling)
        })
        .count() as u64;
    serial + creditable
}

/// 第一道闸（gate1）：按闸读法分派。
fn compute_gate1(pool: &Pool, arm: Arm, in_flight: u64) -> (u64, GateSnapshot) {
    let snapshot = gate_snapshot(pool, in_flight);
    let serial = gate_serial(&snapshot, pool, arm);
    let value = match arm.gate {
        GateReading::Serial | GateReading::SerialRejudge => serial,
        GateReading::Union | GateReading::UnionPrime => gate_union(pool, serial),
    };
    (value, snapshot)
}

/// 第二道闸：可分配_D16 = min(U − InF + L_meta − P_push, df_D16 − InF)，四条闸读法共用。
fn compute_gate2(snapshot: &GateSnapshot) -> u64 {
    let left = snapshot
        .allocatable_free_slots
        .saturating_sub(snapshot.in_flight)
        .saturating_add(snapshot.live_meta)
        .saturating_sub(snapshot.push_reserve);
    let right = snapshot.defined_free_space_formula_slots.saturating_sub(snapshot.in_flight);
    left.min(right)
}

/// `df`（字节，按臂）：串族取可用_串，合族取 D16（发布语义） 那条 `df` 式子算出的槽数；
/// 合撇 在这个基础上再减一次切换预留、被抛弃根独占量与在飞已批准（见「五、5.3」闸的读法表）。
/// PC2（阳性对照）：改坏时一律报 U_all + Rel，不管闸读法是哪一种。
fn df_bytes(pool: &Pool, arm: Arm, snapshot: &GateSnapshot, serial: u64) -> u64 {
    if pool.damage.raw_df {
        return (snapshot.all_free_slots_including_isolated_and_held + snapshot.released) * 16384;
    }
    let slots = match arm.gate {
        GateReading::Serial | GateReading::SerialRejudge => serial,
        GateReading::Union => snapshot.defined_free_space_formula_slots,
        GateReading::UnionPrime => snapshot
            .defined_free_space_formula_slots
            .saturating_sub(snapshot.switch_reserve)
            .saturating_sub(snapshot.abandoned_root_exclusive_slots)
            .saturating_sub(snapshot.in_flight),
    };
    slots * 16384
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pool_and_residue_match_registered_constants() {
        assert_eq!(10 + 7 * 4, 38);
        assert_eq!(5 + 6 * 4, 29);
        assert_eq!(38 + 29, 67);
        assert_eq!(10 + 7 * 9, 73);
        assert_eq!(5 + 6 * 9, 59);
        assert_eq!(73 + 59, 132);
    }

    #[test]
    fn switch_reserve_per_disk_matches_registered_formula() {
        let switch_share = |fixpoint_cost: u64| 2 + RING_REGIONS * fixpoint_cost;
        assert_eq!((INSTANCE_SWITCH_LIMIT + 1) * switch_share(4), 56);
        assert_eq!((INSTANCE_SWITCH_LIMIT + 1) * switch_share(9), 116);
    }

    /// A4：checkpoint 保留池 = c（D28（挂载期承诺量） 已定项 4）。此前这里写的是
    /// `assert_eq!(fixpoint_cost, fixpoint_cost)`，对任何输入恒真、不会红，等于没有断言
    /// （2026-09-17 产物之前发现并改写，见登记「十二、修订」）。现在真跑 `gate_snapshot`，
    /// 锚点是与公式本身无关的字面量（不是再抄一遍 `fixpoint_cost` 变量），覆盖登记「五、5.5」
    /// 两个取样点 c = 4 与 c = 9。
    #[test]
    fn checkpoint_reserve_equals_fixpoint_cost() {
        for (fixpoint_cost, expected_checkpoint_reserve) in [(4u64, 4u64), (9u64, 9u64)] {
            let geometry = Geometry {
                capacity_slots_per_disk: 600,
                object_units: 1,
                fixpoint_cost,
                slots_per_region: 4,
                padding_deletes: 0,
            };
            let pool = Pool::seed(geometry);
            let snapshot = gate_snapshot(&pool, 0);
            assert_eq!(
                snapshot.checkpoint_reserve, expected_checkpoint_reserve,
                "checkpoint 保留池应等于 c，c = {fixpoint_cost}"
            );
        }
    }

    #[test]
    fn retry_budget_is_eight() {
        assert_eq!(RETRY_BUDGET_PUBLICATIONS, 4 + 2 * TOLERATED_ROOT_WRITE_FAILURE_COUNT);
        assert_eq!(RETRY_BUDGET_PUBLICATIONS, 8);
    }

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
    fn baseline_geometry() -> Geometry {
        Geometry {
            capacity_slots_per_disk: 600,
            object_units: 1,
            fixpoint_cost: 4,
            slots_per_region: 4,
            padding_deletes: 0,
        }
    }

    fn real_baseline_arm() -> Arm {
        Arm { gate: GateReading::SerialRejudge, reclaim: ReclaimTiming::AfterEffectiveFloor, stop: StopPolicy::StopAtTarget }
    }

    /// R8：根槽写失败也要推进一格 txg（D23（journal 的角色与格式） 已定项 14 逐字「不推进就是
    /// 反复重写同一个槽、把 R 个失败域用成一个」）。这条分支在 H1 里从没被走到过（H1 从不注入
    /// 失败）；2026-09-17 实现 H5 之前发现它不碰 `pool.next_txg`，产物之前修好，见登记「十二、修订」。
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

    /// 重试循环的触动预算 J_max = 3S + 5（登记「五、5.4」）。
    #[test]
    fn retry_touch_budget_matches_registered_formula() {
        assert_eq!(retry_touch_budget(4), 17);
        assert_eq!(retry_touch_budget(8), 29);
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

    /// I7：平时切换预留 = 3 × (2 + 3c)；每次实例切换做完再少 2 + 3c。
    #[test]
    fn switch_reserve_slots_matches_registered_formula() {
        let mut pool = Pool::seed(baseline_geometry());
        assert_eq!(pool.switch_reserve_slots(), 3 * (2 + 3 * 4), "平时（写行暖机做完）应为 3 份 × (2 + 3c)");
        pool.switch_reserve_shares_per_disk -= 1;
        assert_eq!(pool.switch_reserve_slots(), 2 * (2 + 3 * 4), "切换一次少一份 2 + 3c");
    }

    /// I2 的锚点公式（`3S − 1`）本身：环里最旧有效根只靠环转一圈往前挪，不靠抬 F、不靠推空——
    /// 直接用空发布把 txg 一路推到某个参照点 D 之后，核「再持久多少条空发布，环里最旧有效根
    /// 才追上 D」。这与登记「五、5.4」对串族 K 的推导是同一个环形几何事实，只是不经过
    /// 准入闸（准入闸会不会额外触发推空是 5.3 的题，这里只核环本身）。
    #[test]
    fn ring_rotation_only_reaches_reference_txg_after_three_slots_per_region_minus_one_publishes() {
        let geometry = Geometry { capacity_slots_per_disk: 4000, object_units: 1, fixpoint_cost: 4, slots_per_region: 4, padding_deletes: 0 };
        let arm_reclaim = ReclaimTiming::AfterEffectiveFloor;
        let mut pool = Pool::seed(geometry);
        for _ in 0..80 {
            push_one_empty_publish(&mut pool, arm_reclaim, &mut |_last_txg| false);
        }
        let reference_txg = pool.readable_roots().map(|root| root.txg).max().expect("至少一条根");
        let mut steps = 0u64;
        loop {
            if pool.oldest_valid_root_txg() >= reference_txg {
                break;
            }
            push_one_empty_publish(&mut pool, arm_reclaim, &mut |_last_txg| false);
            steps += 1;
        }
        assert_eq!(steps, 3 * geometry.slots_per_region - 1, "环转一圈追上参照点 D 应恰好要 3S − 1 条额外发布");
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

    /// 阳性对照 PC1 在装置里真的会改变 Q1（不是摆设）：H1 在真实基线上有推空发生的那一格，
    /// 关掉抬 F 之后重试循环只能靠环转一圈，K 应变得更大或直接不达标。
    #[test]
    fn positive_control_disabling_floor_raise_changes_delete_then_rewrite_outcome() {
        let geometry = Geometry {
            capacity_slots_per_disk: 4000,
            object_units: 16,
            fixpoint_cost: 4,
            slots_per_region: 8,
            padding_deletes: 2,
        };
        let arm = real_baseline_arm();
        let baseline = run_delete_then_rewrite_history(arm, geometry);
        let damaged = run_delete_then_rewrite_history_with_damage(arm, geometry, Damage { disable_floor_raise: true, ..Damage::default() });
        assert_ne!(
            (baseline.delete_then_rewrite_outcome_label.clone(), baseline.delete_then_rewrite_publish_count),
            (damaged.delete_then_rewrite_outcome_label, damaged.delete_then_rewrite_publish_count),
            "PC1 应当能改变 Q1：baseline={:?}/{} damaged 应不同",
            baseline.delete_then_rewrite_outcome_label, baseline.delete_then_rewrite_publish_count
        );
    }

    /// 阳性对照 PC2 真的会让 Q2a 计数：`df` 报成 U_all+Rel 之后，串族的 df 会比真实的可用_串
    /// 更宽（不扣 M_sw／checkpoint 保留池／占着），至少能在填到快满时出现「df 报的比闸判的宽」。
    #[test]
    fn positive_control_pc2_produces_q2a_hits() {
        let geometry = Geometry {
            capacity_slots_per_disk: 4000,
            object_units: 16,
            fixpoint_cost: 9,
            slots_per_region: 8,
            padding_deletes: 2,
        };
        let arm = real_baseline_arm();
        let damaged = run_delete_then_rewrite_history_with_damage(arm, geometry, Damage { raw_df: true, ..Damage::default() });
        assert!(damaged.df_reported_enough_but_write_failed_count >= 1, "PC2 改坏 df 之后，填满前后至少要有一次 df≥s 而写失败");
    }

    /// 阳性对照 PC5：保留量全 0 时，近满盘上固定点分配会失败，卡死。
    #[test]
    fn positive_control_pc5_wedges_when_reserves_zero() {
        let geometry = Geometry {
            capacity_slots_per_disk: 600,
            object_units: 1,
            fixpoint_cost: 9,
            slots_per_region: 8,
            padding_deletes: 2,
        };
        let mut wedged_any = false;
        for arm in all_arms() {
            let damaged = run_delete_then_rewrite_history_with_damage(arm, geometry, Damage { zero_reserves: true, ..Damage::default() });
            if damaged.wedged {
                wedged_any = true;
            }
        }
        assert!(wedged_any, "PC5：保留量全 0 时，至少要有一条臂在这一格上卡死");
    }

    /// 阳性对照 PC5 对 H5 也要跑（2026-09-17 主 agent 指示「阳性对照对新历史也要跑」）：k=1、位=首
    /// 在 `pc5_geometry` 上对每一条臂都退化（2026-09-17 实测：`zero_reserves` 把 `checkpoint_reserve`／
    /// `switch_reserve` 都清零，gate1 的减项跟着少两项，3 次触动之后 Y 常常不用推空就批准——
    /// `FailureInjector` 的窗口一次都没被问到，`injected=0`，H4a／H5 的退化条款按字面触发，这不是 bug，
    /// 是「注入的故障没有机会发生」）；k=3、位=覆需要 pushing 推得更久，能撞见真卡死——同一份探针
    /// 换成 k=3／覆 之后，24 条臂里多数（除 `串` 族与 `串重判-G7-止`）都在这一格真卡死。
    #[test]
    fn positive_control_pc5_wedges_root_write_failure_when_reserves_zero() {
        let geometry = Geometry { capacity_slots_per_disk: 600, object_units: 1, fixpoint_cost: 9, slots_per_region: 8, padding_deletes: 2 };
        let mut wedged_any = false;
        for arm in all_arms() {
            let (summary, ..) = run_root_write_failure_history(arm, geometry, 3, FailurePosition::Cover, Damage { zero_reserves: true, ..Damage::default() });
            if summary.wedged {
                wedged_any = true;
            }
        }
        assert!(wedged_any, "PC5 对 H5（k=3、位=覆）：保留量全 0 时，至少要有一条臂在这一格上真卡死，不是退化");
    }

    /// `FailureInjector`（H5-k-位 R8「切换的写行发布的根也算在连续失败里」的窗口）：位=首（`window_start=1`）
    /// 连续 k 次真返回、之后恒假。
    #[test]
    fn failure_injector_first_position_injects_from_first_call() {
        let mut injector = FailureInjector::new(1, 2);
        assert_eq!([injector.next(0), injector.next(0), injector.next(0), injector.next(0)], [true, true, false, false]);
        assert_eq!(injector.injected, 2);
    }

    /// 位=覆（`window_start` > 1）：窗口之前恒假，进窗口之后连续 k 次真，窗口用完恒假。
    #[test]
    fn failure_injector_cover_position_injects_from_window_start() {
        let mut injector = FailureInjector::new(3, 2);
        assert_eq!(
            [injector.next(0), injector.next(0), injector.next(0), injector.next(0), injector.next(0)],
            [false, false, true, true, false]
        );
        assert_eq!(injector.injected, 2);
    }

    /// `cover_push_index` 与「四」I1 锚点同一个 txg mod 3 公式：起点状态最后一条根 txg=2（暖机第二条），
    /// 2 mod 3 = 2 ⇒ 2 次；推一次空发布到 txg=3（3 mod 3 = 0）⇒ 仍是 2 次；再推一次到 txg=4
    /// （4 mod 3 = 1）⇒ 3 次。
    #[test]
    fn cover_push_index_matches_ring_rotation_anchor_formula() {
        let geometry = baseline_geometry();
        let mut pool = Pool::seed(geometry);
        assert_eq!(cover_push_index(&pool), 2, "起点状态最后一条根 txg=2，2 mod 3 = 2 ⇒ 2 次");
        push_one_empty_publish(&mut pool, ReclaimTiming::AfterEffectiveFloor, &mut |_last_txg| false);
        assert_eq!(cover_push_index(&pool), 2, "txg=3，3 mod 3 = 0 ⇒ 2 次");
        push_one_empty_publish(&mut pool, ReclaimTiming::AfterEffectiveFloor, &mut |_last_txg| false);
        assert_eq!(cover_push_index(&pool), 3, "txg=4，4 mod 3 = 1 ⇒ 3 次");
    }

    /// R8「切换的写行发布的根也算在连续失败里」：k=1、位=首在一个真的需要推空的几何点上必须真正
    /// 触发至少一次实例切换（不是「退化」——这里手选了一个容量小、S 小、临近满盘的格，确保 3 次
    /// 触动之后写 Y 一定要推空）。钉绝对值：`injected` 必须恰好等于 1（不多不少，多了说明窗口
    /// 没有在命中之后关掉，少了说明这次准入里没有走到该走的推空）。
    #[test]
    fn run_root_write_failure_history_with_one_failure_and_first_position_injects_exactly_one_failure_and_switches() {
        let geometry = Geometry { capacity_slots_per_disk: 600, object_units: 16, fixpoint_cost: 4, slots_per_region: 4, padding_deletes: 0 };
        let arm = real_baseline_arm();
        let (summary, injected, switches_triggered, recovery_publishes_until_covered) =
            run_root_write_failure_history(arm, geometry, 1, FailurePosition::First, Damage::default());
        assert_eq!(injected, 1, "k=1／首：窗口只要 1 次失败，多问或少问都不对");
        assert!(switches_triggered >= 1, "根槽写失败必须按 R8 触发至少一次实例切换");
        assert!(recovery_publishes_until_covered >= 1, "覆盖两块盘至少要 1 次根槽写尝试");
        assert_ne!(summary.delete_then_rewrite_outcome_label, "退化", "这一格本该真的走到推空，不该退化");
    }

    /// H6「回退之后隔离了一批槽再写」：真实基线在一个能回退成功的几何点上，隔离槽数必须 > 0——
    /// 这正是登记 H6 那句话本身要核的事实（不是摆设：`perform_rollback` 若漏掉「只被被抛弃根引用」
    /// 这个条件，隔离数会变成 0 或过大，这条断言都能看出来）。
    #[test]
    fn run_rollback_history_isolates_slots_after_rollback() {
        let geometry = Geometry { capacity_slots_per_disk: 4000, object_units: 16, fixpoint_cost: 4, slots_per_region: 8, padding_deletes: 2 };
        let arm = real_baseline_arm();
        let (summary, isolated_slots_after_rollback) = run_rollback_history(arm, geometry, Damage::default());
        assert_ne!(summary.delete_then_rewrite_outcome_label, "退化", "这一格本该回退成功（不是回退够不着）");
        assert!(isolated_slots_after_rollback > 0, "回退之后应当隔离了一批只被被抛弃根引用的槽");
    }

    /// F5／PC4 的反面：未改坏时，真实基线在这个几何点上不该发出候选根还引用的槽（Q6 = 0）。
    #[test]
    fn real_baseline_never_reuses_candidate_referenced_slots() {
        let geometry = Geometry {
            capacity_slots_per_disk: 4000,
            object_units: 16,
            fixpoint_cost: 4,
            slots_per_region: 8,
            padding_deletes: 2,
        };
        let arm = real_baseline_arm();
        let summary = run_delete_then_rewrite_history(arm, geometry);
        assert_eq!(summary.unsafe_candidate_reuse_count_field, 0, "真实基线（串-重判 × G7）不该发出候选根还引用的槽");
    }

    // ===== H5′（2026-09-17 第四段）：`LazyFailureInjector` =====

    /// 位=首：`window_start` 恒为 1，与 `next` 的参数无关——传任意 `last_txg` 都不改变这一点，
    /// 与 `FailureInjector::new(1, k)` 逐次读数相同。
    #[test]
    fn lazy_failure_injector_first_position_ignores_last_txg_argument() {
        let mut injector = LazyFailureInjector::new(FailurePosition::First, 2);
        assert_eq!(
            [injector.next(999), injector.next(0), injector.next(12345), injector.next(7)],
            [true, true, false, false]
        );
        assert_eq!(injector.injected, 2);
    }

    /// 位=覆：`window_start` 在**第一次**被问到时，用那一次的 `last_txg` 现算并锁定——之后的
    /// 调用即使传不同的 `last_txg`，也不会重算。第一次传 2（2 mod 3 = 2）⇒ `window_start = 2`，
    /// 与 `cover_push_index_matches_ring_rotation_anchor_formula` 的起点状态锚点同一个值。
    #[test]
    fn lazy_failure_injector_cover_position_locks_window_start_on_first_call() {
        let mut injector = LazyFailureInjector::new(FailurePosition::Cover, 2);
        let first = injector.next(2);
        let rest = [injector.next(999), injector.next(999), injector.next(999)];
        assert_eq!(first, false, "call_index=1 < window_start=2");
        assert_eq!(rest, [true, true, false], "window_start 锁定之后不再看后续调用传的 999");
        assert_eq!(injector.injected, 2);
    }

    /// 同一条锚点公式的另一个取样点：第一次传 4（4 mod 3 = 1）⇒ `window_start = 3`——
    /// 证明 `window_start` 真的是从第一次调用的参数现算出来的，不是写死的 2。
    #[test]
    fn lazy_failure_injector_cover_position_with_different_first_txg_matches_ring_rotation_anchor() {
        let mut injector = LazyFailureInjector::new(FailurePosition::Cover, 1);
        assert_eq!([injector.next(4), injector.next(4), injector.next(4)], [false, false, true]);
        assert_eq!(injector.injected, 1);
    }

    // ===== H4a／H4b（2026-09-17 第四段）：推空发布之中崩溃再重开 =====

    /// H4a 与 H4b 在本模型里产生同一个 `RunSummary`（除 `history` 标签外）——见
    /// `run_crash_mid_push_history` 头部 ⚠️：`attempt_publish` 是一次性完成分配 + 写根，
    /// 崩在「根持久之后」与崩在「下一次已分配、根还没持久」在本模型能表达的粒度上是同一个状态。
    /// 这条断言把「先证明等价，再当等价留档」的等价性钉死，不靠肉眼比对产物。
    #[test]
    fn h4a_and_h4b_crash_points_are_observationally_equivalent_in_this_model() {
        let geometry = Geometry { capacity_slots_per_disk: 600, object_units: 1, fixpoint_cost: 4, slots_per_region: 4, padding_deletes: 0 };
        let arm = Arm { gate: GateReading::SerialRejudge, reclaim: ReclaimTiming::AfterEffectiveFloor, stop: StopPolicy::FillBudget };
        let (after_first_push_summary, after_first_push_self_pushes, after_first_push_recovery_publishes) = run_crash_mid_push_history(arm, geometry, CrashPoint::AfterFirstPush, Damage::default());
        let (before_second_push_summary, before_second_push_self_pushes, before_second_push_recovery_publishes) = run_crash_mid_push_history(arm, geometry, CrashPoint::BeforeSecondPush, Damage::default());
        assert_eq!(after_first_push_summary.delete_then_rewrite_outcome_label, "达标", "这一格本该真的崩到、重开、写成——不是退化或卡死");
        assert_eq!(after_first_push_summary.delete_then_rewrite_outcome_label, before_second_push_summary.delete_then_rewrite_outcome_label);
        assert_eq!(after_first_push_summary.delete_then_rewrite_publish_count, before_second_push_summary.delete_then_rewrite_publish_count);
        assert_eq!(after_first_push_summary.wedged, before_second_push_summary.wedged);
        assert_eq!(after_first_push_summary.unsafe_candidate_reuse_count_field, before_second_push_summary.unsafe_candidate_reuse_count_field);
        assert_eq!(after_first_push_summary.filler_objects_written_in_fill_phase, before_second_push_summary.filler_objects_written_in_fill_phase);
        assert_eq!(after_first_push_self_pushes, before_second_push_self_pushes);
        assert_eq!(after_first_push_self_pushes, 1, "两个崩溃点在本模型里都恰好在 1 次自推空发布持久之后崩");
        assert_eq!(after_first_push_recovery_publishes, before_second_push_recovery_publishes);
    }

    /// H4a／H4b 的退化条款：这次准入根本不用推空就过闸（第一道闸不重判）的臂上，崩溃点不存在。
    #[test]
    fn run_crash_mid_push_history_degenerates_when_gate_never_self_pushes() {
        let geometry = Geometry { capacity_slots_per_disk: 4000, object_units: 16, fixpoint_cost: 4, slots_per_region: 8, padding_deletes: 2 };
        let arm = Arm { gate: GateReading::Serial, reclaim: ReclaimTiming::AfterEffectiveFloor, stop: StopPolicy::StopAtTarget };
        let (summary, self_pushes_before_crash, recovery_publishes_after_reopen) = run_crash_mid_push_history(arm, geometry, CrashPoint::AfterFirstPush, Damage::default());
        assert_eq!(summary.delete_then_rewrite_outcome_label, "退化", "串 这条臂的第一道闸不重判，永远走不到自推空发布，崩溃点不存在");
        assert_eq!(self_pushes_before_crash, 0);
        assert_eq!(recovery_publishes_after_reopen, 0);
    }

    // ===== H5′（2026-09-17 第四段）：注入点改到填满阶段 =====

    /// 真实基线在这个几何点上，填满阶段自己确实自推了空发布——`run_fill_phase_injected_history` 的注入
    /// 打在填满阶段（`fill_phase_self_pushes` > 0），不是退回 H5 的注入点。数字是这个几何 + k=1
    /// + 位=首下的实测值（见登记「十二、修订」第四段）。
    #[test]
    fn run_fill_phase_injected_history_injects_during_fill_phase_for_real_baseline() {
        let geometry = Geometry { capacity_slots_per_disk: 600, object_units: 1, fixpoint_cost: 4, slots_per_region: 4, padding_deletes: 0 };
        let arm = real_baseline_arm();
        let (summary, injected, _switches, _recovery, fill_phase_self_pushes) =
            run_fill_phase_injected_history(arm, geometry, 1, FailurePosition::First, Damage::default());
        assert_eq!(injected, 1, "k=1：窗口只要 1 次失败");
        assert_eq!(fill_phase_self_pushes, 4, "这个几何点上填满阶段自己推了 4 次空发布，注入应打在其中第 1 次");
        assert_ne!(summary.delete_then_rewrite_outcome_label, "退化", "注入已经打在填满阶段，这一格不该退化");
    }

    /// 头部结论：真实基线（串重判 × G7 × 止）在 H5 上因为「填满阶段自己把 F 抬走」而退化的那个
    /// 具体几何点（登记「十二、修订」第三段诊断第 2 类原文实测），H5′ 上不再退化——这是
    /// `run_fill_phase_injected_history` 整个改动要解决的那个问题，钉成一条对照断言（H5 与 H5′ 用同一个
    /// k=1、位=首，唯一变量是注入点）。
    #[test]
    fn fill_phase_injection_eliminates_degeneration_that_original_injection_point_could_not_reach() {
        let geometry = Geometry { capacity_slots_per_disk: 4000, object_units: 1, fixpoint_cost: 4, slots_per_region: 4, padding_deletes: 0 };
        let arm = real_baseline_arm();
        let (original_injection_point_summary, ..) = run_root_write_failure_history(arm, geometry, 1, FailurePosition::First, Damage::default());
        let (fill_phase_injected_summary, ..) = run_fill_phase_injected_history(arm, geometry, 1, FailurePosition::First, Damage::default());
        assert_eq!(original_injection_point_summary.delete_then_rewrite_outcome_label, "退化", "H5 在这一格上退化（注入点从没被这次准入的推空碰到）");
        assert_eq!(fill_phase_injected_summary.delete_then_rewrite_outcome_label, "达标", "H5′ 把注入点挪到填满阶段之后，这一格不再退化");
    }
}


// ===== 分配、释放、发布（5.1 一次发布 ①②③④） =====

/// 取 `count` 个槽号最低的可发单槽（R12）：固定点与实例表用，不要求成对。
fn allocate_lowest_single_slots(pool: &Pool, count: u64) -> Option<Vec<usize>> {
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
fn allocate_lowest_data_units(pool: &Pool, unit_count: u64) -> Option<Vec<usize>> {
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

fn release_slots(pool: &mut Pool, slots: &[usize], released_at_txg: u64, from_empty_publish: bool) {
    for &index in slots {
        pool.slots[index] = SlotState::Released {
            released_at_txg,
            from_empty_publish,
        };
    }
}

/// 一次发布持久之后的回收（5.3「回收时点」）：ring-rotation 那一半（g ≤ 环里最旧有效根）
/// 两种回收时点都直接生效；G7 再按 max(F_生效, 环里最旧有效根) 回收；F-扣／F-扣′ 只在
/// 扣住到期（`pending_hold_floor` 达到 F_生效）时把扣住的槽放开。
fn reclaim_after_publish(pool: &mut Pool, reclaim: ReclaimTiming) {
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
        ReclaimTiming::HoldUntilCovered | ReclaimTiming::HoldUntilCoveredPrime => {
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

/// F-扣／F-扣′：这次发布要带的 F 大于 F_顶 时，先把 g ≤ 新 F 的已释放槽扣住（登记「五、5.3」）。
/// PC4（F-扣 一族）：改坏时跳过扣住，直接把这批槽变空闲——复现 `mount.rs` 注释里
/// 「不扣住时抬 F 自己的空发布落在刚回收的槽上」那个曾经打中过的缺陷。
fn maybe_hold_before_publish(pool: &mut Pool, reclaim: ReclaimTiming, new_floor: u64) {
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
fn count_unsafe_reuses(pool: &Pool, allocated: &[usize]) -> u64 {
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

#[derive(Debug)]
struct PublishAttemptFailed;

/// 一次发布（可能是空发布、用户发布、写行发布或回退那次发布）。
/// `object_release`／`object_allocate` 只用于用户发布；`rewrite_instance_table` 只在
/// 写行／回退发布为真。`fail` 为真时模拟根槽写失败（R8）：分配整份退回，不持久。
#[allow(clippy::too_many_arguments, reason = "发布是本模型唯一的写路径，参数对应 5.1 明确列出的各个输入维度，拆结构体不会减少要传的信息")]
fn attempt_publish(
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
    // 2026-09-17 产物之前发现并改（见登记「十二、修订」）：这一分支此前不碰 `pool.next_txg`，
    // 下一次重试会拿同一个 txg 再写同一个槽——H1 从没注入过失败，这条分支此前零覆盖，H5 是第一个
    // 会走到这里的历史。
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
        ReclaimTiming::HoldUntilCovered | ReclaimTiming::HoldUntilCoveredPrime => floor,
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
fn assert_conservation(pool: &Pool, where_: &str) {
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
            _ => None,
        })
        .max()
        .unwrap_or(0);
    newest_persisted_rollback_floor.max(maximum_released_txg.min(ceiling))
}

/// P-止：推空之前，目标 ≤ F_顶 且 F_生效 = F_顶（没有东西可抬、也没有抬到一半）就停。
fn stop_policy_says_stop(pool: &Pool, stop: StopPolicy) -> bool {
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
/// 供 `LazyFailureInjector` 现算 `window_start` 用；`FailureInjector` 忽略它）；H5（2026-09-17 加）要求
/// 「切换的写行发布的根也算在连续失败里」（登记「五、5.4」H5-k-位 定义），所以这个闭包要一路带进
/// `perform_instance_switch`，覆盖写行与暖机自己的根槽写——不能只在这里问一次就把结果存成 `bool`
/// 往下传，那样切换内部的每一次根槽写都会读到同一个决定。
fn push_one_empty_publish(pool: &mut Pool, reclaim: ReclaimTiming, failures: &mut dyn FnMut(u64) -> bool) -> u64 {
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
fn perform_instance_switch(pool: &mut Pool, reclaim: ReclaimTiming, failures: &mut dyn FnMut(u64) -> bool) -> u64 {
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

// ===== 准入（5.3：一次准入的发布预算 B = 8，按闸读法分派） =====

/// 第二道闸不过时不会单独终止：`admit_request_inner` 里的 gate2 循环只会继续推空，
/// 直到批准、停法叫停、预算做满或卡死——所以没有一条独立的 `EnospcGate2` 出口。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AdmitOutcome {
    Approved,
    EnospcGate1,
    EnospcStop,
    EnospcBudget,
    Wedged,
}

/// 一次准入判定：`budget_used` 是这次调用之前已经用掉的发布尝试数（同一窗口内多个请求共用）；
/// `other_in_flight` 是窗口内其它已批准、还没封窗的请求占用的槽数。
/// 返回批准与否、这次判定额外消耗的发布尝试数、以及经过的失败注入队列（用于 H5）。
struct AdmitResult {
    outcome: AdmitOutcome,
    attempts_used: u64,
}

/// 外壳：判之前按臂取 `df`（字节），判完按 Q2a／Q3 的口径记账，再转发给内层判定逻辑。
fn admit_request(
    pool: &mut Pool,
    arm: Arm,
    need_slots: u64,
    other_in_flight: u64,
    budget_used: u64,
    failures: impl FnMut(u64) -> bool,
) -> AdmitResult {
    let (gate1_before, snapshot_before) = compute_gate1(pool, arm, other_in_flight);
    let initial_df_bytes = df_bytes(pool, arm, &snapshot_before, gate1_before);
    let need_bytes = need_slots * 16384;
    let result = admit_request_inner(pool, arm, need_slots, other_in_flight, budget_used, failures);
    if result.outcome != AdmitOutcome::Approved && initial_df_bytes >= need_bytes {
        pool.df_reported_enough_but_write_failed_count += 1;
    }
    if result.outcome == AdmitOutcome::EnospcBudget {
        pool.budget_exhausted_hit = true;
    }
    pool.maximum_publish_attempts_seen = pool.maximum_publish_attempts_seen.max(budget_used + result.attempts_used);
    result
}

fn admit_request_inner(
    pool: &mut Pool,
    arm: Arm,
    need_slots: u64,
    other_in_flight: u64,
    budget_used: u64,
    mut failures: impl FnMut(u64) -> bool,
) -> AdmitResult {
    let mut attempts = 0u64;
    let budget = pool.damage.budget_override.unwrap_or(RETRY_BUDGET_PUBLICATIONS);
    let (gate1, _) = compute_gate1(pool, arm, other_in_flight);
    let mut gate1_value = gate1;
    if gate1_value < need_slots {
        if arm.gate.rejudges_first_gate() {
            loop {
                if budget_used + attempts >= budget {
                    return AdmitResult { outcome: AdmitOutcome::EnospcBudget, attempts_used: attempts };
                }
                if stop_policy_says_stop(pool, arm.stop) {
                    return AdmitResult { outcome: AdmitOutcome::EnospcStop, attempts_used: attempts };
                }
                attempts += push_one_empty_publish(pool, arm.reclaim, &mut failures);
                if pool.allocation_failure_hit {
                    return AdmitResult { outcome: AdmitOutcome::Wedged, attempts_used: attempts };
                }
                let (recomputed, _) = compute_gate1(pool, arm, other_in_flight);
                gate1_value = recomputed;
                if gate1_value >= need_slots {
                    break;
                }
            }
        } else {
            return AdmitResult { outcome: AdmitOutcome::EnospcGate1, attempts_used: attempts };
        }
    }
    loop {
        let (_, snapshot) = compute_gate1(pool, arm, other_in_flight);
        let gate2_value = compute_gate2(&snapshot);
        if gate2_value >= need_slots {
            return AdmitResult { outcome: AdmitOutcome::Approved, attempts_used: attempts };
        }
        if budget_used + attempts >= budget {
            return AdmitResult { outcome: AdmitOutcome::EnospcBudget, attempts_used: attempts };
        }
        if stop_policy_says_stop(pool, arm.stop) {
            return AdmitResult { outcome: AdmitOutcome::EnospcStop, attempts_used: attempts };
        }
        attempts += push_one_empty_publish(pool, arm.reclaim, &mut failures);
        if pool.allocation_failure_hit {
            return AdmitResult { outcome: AdmitOutcome::Wedged, attempts_used: attempts };
        }
    }
}

/// 批准之后的写通常不会再失败，但阳性对照 PC5（保留量清零）故意破坏了这份保证——
/// 那种时候这里要老老实实返回 `None`，不许 `.expect()`。`attempt_publish` 失败时已经把
/// `pool.allocation_failure_hit` 置位（Q5a／Q5b 的口径），调用方按这个标志走卡死分支。
fn publish_user_write(pool: &mut Pool, reclaim: ReclaimTiming, units: u64) -> Option<(u64, u64)> {
    let floor = pool.newest_persisted_floor();
    match attempt_publish(pool, reclaim, true, floor, None, Some(units), false, None, false) {
        Ok(txg) => Some((txg, pool.next_object_identifier - 1)),
        Err(PublishAttemptFailed) => None,
    }
}

fn publish_delete(pool: &mut Pool, reclaim: ReclaimTiming, object_identifier: u64) -> Option<u64> {
    let floor = pool.newest_persisted_floor();
    attempt_publish(pool, reclaim, true, floor, Some(object_identifier), None, false, None, false).ok()
}

// ===== Q1 与轨迹：一次「删了再写」的重试循环 =====

#[derive(Clone, Copy, Debug)]
enum RetryOutcome {
    Success { non_empty_publish_count: u64 },
    OverBound { non_empty_publish_count: u64 },
    NotCompleted,
    Wedged,
    Degenerate,
}

/// 重试循环（登记「五、5.4」）：尝试写 `unit_count` 个单元；不成就触动一次（删一个填充对象），
/// 再尝试，最多 `retry_touch_budget(geometry.slots_per_region)` 次触动。
/// `failures` 在每次推空前被问一次，返回真表示这次要注入根槽写失败（R8／H5 用）。
/// `initial_non_empty_publish_count` 与 `initial_touches`：H4a／H5 的前缀在进这个循环之前已经先做了
/// 若干次「触动（中间不尝试）」（登记「五、5.4」：H4a／H5 的前缀是「连续 3 次触动，中间不尝试」，
/// 与 H1 从第一次就「尝试再触动」交替着走不同）——这两个参数把那几次已经发生的非空发布计进来，
/// H1 传 0、0 与此前完全等价。
fn retry_loop(
    pool: &mut Pool,
    arm: Arm,
    geometry: Geometry,
    filler_queue: &mut Vec<u64>,
    unit_count: u64,
    initial_non_empty_publish_count: u64,
    initial_touches: u64,
    mut failures: impl FnMut(u64) -> bool,
) -> RetryOutcome {
    let need = unit_count * 2;
    let budget = retry_touch_budget(geometry.slots_per_region);
    let mut non_empty_publish_count = initial_non_empty_publish_count;
    let mut touches = initial_touches;
    loop {
        let result = admit_request(pool, arm, need, 0, 0, &mut failures);
        match result.outcome {
            AdmitOutcome::Approved => {
                return match publish_user_write(pool, arm.reclaim, unit_count) {
                    Some(_) => {
                        if non_empty_publish_count <= 3 {
                            RetryOutcome::Success { non_empty_publish_count }
                        } else {
                            RetryOutcome::OverBound { non_empty_publish_count }
                        }
                    }
                    None => RetryOutcome::Wedged,
                };
            }
            AdmitOutcome::Wedged => return RetryOutcome::Wedged,
            _ => {
                if touches >= budget {
                    return RetryOutcome::NotCompleted;
                }
                match filler_queue.pop() {
                    Some(object_identifier) => {
                        if publish_delete(pool, arm.reclaim, object_identifier).is_none() {
                            return RetryOutcome::Wedged;
                        }
                        non_empty_publish_count += 1;
                        touches += 1;
                    }
                    None => return RetryOutcome::Degenerate,
                }
            }
        }
    }
}

// ===== 六：一次「跑」的汇总（Q1–Q8） =====

#[derive(Clone, Debug)]
struct RunSummary {
    arm: String,
    is_real_baseline: bool,
    reclaim_label: String,
    stop_label: String,
    history: String,
    capacity: u64,
    object_units: u64,
    fixpoint_cost: u64,
    slots_per_region: u64,
    padding_deletes: u64,
    delete_then_rewrite_outcome_label: String,
    delete_then_rewrite_publish_count: i64,
    df_reported_enough_but_write_failed_count: u64,
    admission_passed_but_allocation_failed_count: u64,
    budget_was_binding: bool,
    maximum_publish_attempts_seen_field: u64,
    allocation_failure_hit_field: bool,
    wedged: bool,
    unsafe_candidate_reuse_count_field: u64,
    retained_states_dropped_below_minimum: u64,
    filler_objects_written_in_fill_phase: u64,
}

fn retry_outcome_label(outcome: RetryOutcome) -> (String, i64) {
    match outcome {
        RetryOutcome::Success { non_empty_publish_count } => {
            ("达标".to_string(), i64::try_from(non_empty_publish_count).unwrap_or(i64::MAX))
        }
        RetryOutcome::OverBound { non_empty_publish_count } => {
            ("越界".to_string(), i64::try_from(non_empty_publish_count).unwrap_or(i64::MAX))
        }
        RetryOutcome::NotCompleted => ("未成".to_string(), -1),
        RetryOutcome::Wedged => ("卡死".to_string(), -2),
        RetryOutcome::Degenerate => ("退化".to_string(), -3),
    }
}

/// 填满阶段：写对象 X（`geometry.object_units` 个单元），再一个接一个写 1 单元填充对象，
/// 直到某次填充写报 ENOSPC。返回 X 的对象号与填充对象队列（先进先出即最先写、槽号最低）。
/// `failures`（2026-09-17 第四段加，H5′ 用）：填满阶段每一次自推空发布之前被问一次，参数是
/// `last_persisted_txg`——H1／H5／H6 一律传 `&mut |_last_txg| false`（不注入，行为与加这个参数
/// 之前逐字节相同），只有 `run_fill_phase_injected_history` 传一个真的 `LazyFailureInjector`。内部两处
/// `admit_request` 调用都要 `&mut *failures` 重新借用——`failures` 是个 `&mut dyn FnMut(u64) -> bool`，
/// 不能被 `admit_request` 按值吃掉之后还留着给下一轮循环用。
fn fill_pool(pool: &mut Pool, arm: Arm, geometry: Geometry, failures: &mut dyn FnMut(u64) -> bool) -> (u64, Vec<u64>) {
    let deleted_reference_object_admission_result = admit_request(pool, arm, geometry.object_units * 2, 0, 0, &mut *failures);
    assert_eq!(
        deleted_reference_object_admission_result.outcome,
        AdmitOutcome::Approved,
        "起点状态之后立刻写 X 应当总是过闸（池几乎全空）"
    );
    let (_, deleted_reference_object_identifier) = publish_user_write(pool, arm.reclaim, geometry.object_units)
        .expect("起点状态之后立刻写 X：池几乎全空，唯一会失败的原因是分配记账有 bug");
    pool.reset_filler_objects_written_counter();
    let mut fillers = Vec::new();
    let mut filler_writes = 0u64;
    loop {
        // 填充对象每个占 2 槽，物理上最多写 capacity/2 个；超过这个数还没停就是分配记账有 bug
        // （比如新分配的槽没被标记 Allocated），不是「池子恰好很大」。
        assert!(
            filler_writes <= pool.capacity_slots / 2 + 1,
            "fill_pool：写了 {filler_writes} 个填充对象还没被拒绝，容量只有 {} 槽——分配记账像是有 bug",
            pool.capacity_slots
        );
        let result = admit_request(pool, arm, 2, 0, 0, &mut *failures);
        match result.outcome {
            AdmitOutcome::Approved => match publish_user_write(pool, arm.reclaim, 1) {
                Some((_, filler_object_identifier)) => {
                    fillers.push(filler_object_identifier);
                    pool.filler_objects_written_so_far += 1;
                    filler_writes += 1;
                }
                // PC5（保留量清零）会让「闸批准了、实际分配却失败」真的发生；填满阶段把它
                // 当成这一格已经写满处理，Q5a／Q5b 已经在 attempt_publish 里记过。
                None => break,
            },
            _ => break,
        }
    }
    (deleted_reference_object_identifier, fillers)
}

impl Pool {
    fn reset_filler_objects_written_counter(&mut self) {
        self.filler_objects_written_so_far = 0;
    }
}

/// Q7 的「状态数」：候选集（持久∧有效∧txg≥F_生效 的根）里按 txg 去重的根数——
/// 登记原话按「最近非空发布」归并连续的空发布进同一个状态；本文件的简化读法直接数候选集里
/// 的持久根条数（每条持久根本身就是一个可退到的落点），比原定义更严格（更容易记违例）。
fn distinct_valid_states(pool: &Pool) -> u64 {
    let effective = pool.effective_rollback_floor();
    pool.readable_roots()
        .filter(|root| root.txg >= effective && !pool.is_abandoned(root))
        .count() as u64
}

/// 把一次跑的公共字段（「六」Q1–Q8 里不专属某一种历史的那些）拼成 `RunSummary`。H1／H5／H6
/// 三个历史构造器都走这个函数，字段的取法只写一处（`.claude/singlefs-ai-sop/rules/code-discipline.md`
/// 「重复要生成，不许手抄」）。
#[allow(clippy::too_many_arguments, reason = "RunSummary 本身就是「六」那张表逐列摊开，拆结构体不会减少要传的信息")]
fn build_run_summary(
    arm: Arm,
    geometry: Geometry,
    history: String,
    pool: &Pool,
    outcome_label: String,
    delete_then_rewrite_publish_count: i64,
    wedged: bool,
    retained_states_dropped_below_minimum: u64,
) -> RunSummary {
    RunSummary {
        arm: arm.label(),
        is_real_baseline: arm.is_real_baseline(),
        reclaim_label: arm.reclaim.label().to_string(),
        stop_label: arm.stop.label().to_string(),
        history,
        capacity: geometry.capacity_slots_per_disk,
        object_units: geometry.object_units,
        fixpoint_cost: geometry.fixpoint_cost,
        slots_per_region: geometry.slots_per_region,
        padding_deletes: geometry.padding_deletes,
        delete_then_rewrite_outcome_label: outcome_label,
        delete_then_rewrite_publish_count,
        df_reported_enough_but_write_failed_count: pool.df_reported_enough_but_write_failed_count,
        admission_passed_but_allocation_failed_count: 0,
        budget_was_binding: pool.budget_exhausted_hit,
        maximum_publish_attempts_seen_field: pool.maximum_publish_attempts_seen,
        allocation_failure_hit_field: pool.allocation_failure_hit,
        wedged,
        unsafe_candidate_reuse_count_field: pool.unsafe_candidate_reuse_count,
        retained_states_dropped_below_minimum,
        filler_objects_written_in_fill_phase: pool.filler_objects_written_so_far,
    }
}

/// H1：填满、垫片、删 X、重试循环（登记「五、5.4」）。`damage` 非默认时用于阳性对照（5.6）。
fn run_delete_then_rewrite_history_with_damage(arm: Arm, geometry: Geometry, damage: Damage) -> RunSummary {
    let mut pool = Pool::seed(geometry);
    pool.damage = damage;
    let (deleted_reference_object_identifier, mut fillers) = fill_pool(&mut pool, arm, geometry, &mut |_last_txg| false);
    for _ in 0..geometry.padding_deletes {
        if let Some(filler_object_identifier) = fillers.pop() {
            publish_delete(&mut pool, arm.reclaim, filler_object_identifier);
        }
    }
    publish_delete(&mut pool, arm.reclaim, deleted_reference_object_identifier);
    let states_before_retry = distinct_valid_states(&pool);
    let outcome = retry_loop(&mut pool, arm, geometry, &mut fillers, geometry.object_units, 0, 0, |_last_txg| false);
    let (outcome_label, delete_then_rewrite_publish_count) = retry_outcome_label(outcome);
    let states_after_retry = distinct_valid_states(&pool);
    let retained_states_dropped_below_minimum = u64::from(states_before_retry >= MINIMUM_RETAINED_STATES && states_after_retry < MINIMUM_RETAINED_STATES);
    build_run_summary(
        arm,
        geometry,
        "H1".to_string(),
        &pool,
        outcome_label,
        delete_then_rewrite_publish_count,
        matches!(outcome, RetryOutcome::Wedged),
        retained_states_dropped_below_minimum,
    )
}

fn run_delete_then_rewrite_history(arm: Arm, geometry: Geometry) -> RunSummary {
    run_delete_then_rewrite_history_with_damage(arm, geometry, Damage::default())
}

// ===== H5-k-位：根槽写失败按 R8 走实例切换（登记「五、5.4」） =====

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FailurePosition {
    /// 从第一条带新 F 的根起（这次准入内部第 1 次根槽写）。
    First,
    /// 从让 F_生效 变成新值的那条根起（覆盖两块盘所需的最后一次根槽写；「四」的 I1 锚点
    /// 给出的 2/3 次公式在这里复用，因为它就是同一个 txg mod 3 轮转事实）。
    Cover,
}

impl FailurePosition {
    fn label(self) -> &'static str {
        match self {
            FailurePosition::First => "首",
            FailurePosition::Cover => "覆",
        }
    }
}

/// 覆盖两块盘所需的根槽写次数（与「四」I1 锚点同一个公式：`last_txg mod 3 = 1` 时 3 次，否则 2 次），
/// 只是这里要的是"从哪次开始失败"的窗口起点，不是覆盖次数本身——两者数值相同（覆盖需要几次，
/// 让 F_生效 真正变成新值的正是最后那一次，也就是第几次）。拆成一个只吃 `last_txg` 标量的版本
/// （`cover_push_index_from_last_txg`），是因为 H5′（2026-09-17 第四段）要在 `push_one_empty_publish`
/// 内部、`pool` 已经借给函数参数的那一刻现算这个值——传一个 `&Pool` 进闭包会撞借用检查，传一个
/// 提前读好的 `u64` 不会。
fn cover_push_index_from_last_txg(last_txg: u64) -> u64 {
    if last_txg % RING_REGIONS == 1 { 3 } else { 2 }
}

fn cover_push_index(pool: &Pool) -> u64 {
    cover_push_index_from_last_txg(pool.readable_roots().map(|root| root.txg).max().unwrap_or(0))
}

/// 这次推空即将发生之前，环里最新一条持久根的 txg——`push_one_empty_publish`／`perform_instance_switch`
/// 在问 `failures` 之前用它现算「这是这次运行里第几次自推空发布」所需要知道的那一点点 pool 状态，
/// 不需要把整个 `&Pool` 传进闭包。
fn last_persisted_txg(pool: &Pool) -> u64 {
    pool.readable_roots().map(|root| root.txg).max().unwrap_or(0)
}

/// 连续 k 次根槽写失败的窗口：从 `window_start` 起（1 起数，数的是这次准入内部**每一次**根槽写
/// 尝试——推空自己的、切换写行的、暖机的，登记「五、5.4」H5-k-位「切换的写行发布的根也算在
/// 连续失败里」），连续 k 次返回真。`injected` 记真正问到几次失败——若这次准入的根槽写次数不够
/// （比如闸提前批准、或提前卡死），`injected < k`，调用方据此判定「退化」（登记 H4a／H4b／H5
/// 那句「在这次准入里根本没有推空（或没有第二次推空）的臂上……记『退化』」）。
struct FailureInjector {
    call_index: u64,
    window_start: u64,
    remaining: u64,
    injected: u64,
}

impl FailureInjector {
    fn new(window_start: u64, consecutive_root_write_failure_count: u64) -> Self {
        FailureInjector { call_index: 0, window_start, remaining: consecutive_root_write_failure_count, injected: 0 }
    }
    /// `_last_txg`：H5 的 `window_start` 已经在构造时算好（在进入这次准入的 episode 之前，
    /// 用 `cover_push_index(&pool)` 现算一次），这里不需要再看 pool 状态——参数只是为了跟
    /// `LazyFailureInjector::next` 共用同一个 `FnMut(u64) -> bool` 闭包签名。
    fn next(&mut self, _last_txg: u64) -> bool {
        self.call_index += 1;
        if self.call_index >= self.window_start && self.remaining > 0 {
            self.remaining -= 1;
            self.injected += 1;
            true
        } else {
            false
        }
    }
}

/// H5′ 专用（2026-09-17 第四段，主 agent 定，依据「十二、修订」第三段第 7 条第 2 类诊断：
/// P-止 在「填满」阶段自己把 F 抬到位，H5 的失败注入窗口从没落在这几次自推空发布上）：与
/// `FailureInjector` 同一套「连续 k 次」语义，区别是 `window_start` 不在构造时算好，而是在
/// 第一次被问到（也就是这次运行里第一次真的发生自推空发布，不管是在填满阶段还是在「删后写回」
/// 那次准入里）那一刻，用当时的 `last_txg` 现算——这样同一个注入器既能落在填满阶段第一次为
/// 抬 F 自推的那个 episode 上，也能在填满阶段全程不自推时，原样落到「删后写回」那次准入的推空
/// episode 上（`run_fill_phase_injected_history` 把它从 `fill_pool` 一路带进后续 3 次触动之后的 `retry_loop`，
/// 两处都不清空 `call_index`）。
struct LazyFailureInjector {
    position: FailurePosition,
    call_index: u64,
    window_start: Option<u64>,
    remaining: u64,
    injected: u64,
}

impl LazyFailureInjector {
    fn new(position: FailurePosition, consecutive_root_write_failure_count: u64) -> Self {
        LazyFailureInjector { position, call_index: 0, window_start: None, remaining: consecutive_root_write_failure_count, injected: 0 }
    }
    fn next(&mut self, last_txg: u64) -> bool {
        self.call_index += 1;
        let window_start = *self.window_start.get_or_insert_with(|| match self.position {
            FailurePosition::First => 1,
            FailurePosition::Cover => cover_push_index_from_last_txg(last_txg),
        });
        if self.call_index >= window_start && self.remaining > 0 {
            self.remaining -= 1;
            self.injected += 1;
            true
        } else {
            false
        }
    }
}

/// 「这一格该记『退化』还是『卡死』」：`.claude/rules/mutation-sampling.md`「同一条纪律的反面」——
/// 2026-09-17 探针跑撞到过（PC5 在 H5 上 `pc5_root_write_failure_hit_any=false`）：`Damage::zero_reserves` 时
/// `fill_pool` 自己就会先卡死（近满盘固定点分配失败，`pool.allocation_failure_hit` 已置位），
/// 之后连续 3 次触动里第一次 `publish_delete` 撞上同一个卡死、`is_none()` 为真，H5 的退化分支
/// 把它记成「退化」——这掩盖了 PC5 真正要证明的那件事：它不是「这次准入没走到该走的路径」，
/// 是真的卡死了。所以两处退化分支都要先问 `pool.allocation_failure_hit`，真卡死记「卡死」，
/// 不记「退化」。
fn degenerate_or_wedged_summary(arm: Arm, geometry: Geometry, history: String, pool: &Pool) -> RunSummary {
    if pool.allocation_failure_hit {
        build_run_summary(arm, geometry, history, pool, "卡死".to_string(), -2, true, 0)
    } else {
        build_run_summary(arm, geometry, history, pool, "退化".to_string(), -3, false, 0)
    }
}

/// H5-k-位（登记「五、5.4」）：同 H4a 的前缀——填满、垫片、删 X、连续 3 次触动（中间不尝试，
/// 与 H1 的「尝试再触动」交替不同）；这次准入里（尝试写 Y）连续 k 次根槽写失败，位=首／覆
/// 决定失败窗口从第几次根槽写算起；失败按 R8 走实例切换；之后正常重试循环（窗口过去之后
/// `FailureInjector` 恒返回假，不再注入）。
///
/// 返回 `(RunSummary, injected, switches_triggered, recovery_publishes_until_covered)`：
/// `injected` 是 `FailureInjector` 真正问到的失败次数（< k 时这一格按登记记「退化」，
/// `delete_then_rewrite_outcome_label` 已经是 "退化"，调用方不用另判）；`switches_triggered` 是
/// `pool.switches_so_far`（诊断量，含级联失败触发的中间几次）；`recovery_publishes_until_covered`
/// 是 Q4 的粗口径——从这次准入开始到两块盘都覆盖新 F 为止，一共发生了几次根槽写尝试（含失败的）。
/// ⚠️ **这不是登记「六」Q4 要的 `D_acct`／`D_alloc` 两个分量**：那两个数要逐次尝试（ROW 级）才
/// 分得开（哪一次持久之后记账先认、哪一次之后分配器才真能发），本文件没有实现 ROW 明细
/// （见文件头「输出体量说明」），所以只报"覆盖总共花了几次尝试"这一个合并数，在实验页与报告里
/// 注明是粗口径。
fn run_root_write_failure_history(
    arm: Arm,
    geometry: Geometry,
    consecutive_root_write_failure_count: u64,
    position: FailurePosition,
    damage: Damage,
) -> (RunSummary, u64, u64, u64) {
    let history_label = format!("H5-{consecutive_root_write_failure_count}-{}", position.label());
    let mut pool = Pool::seed(geometry);
    pool.damage = damage;
    let (deleted_reference_object_identifier, mut fillers) = fill_pool(&mut pool, arm, geometry, &mut |_last_txg| false);
    for _ in 0..geometry.padding_deletes {
        if let Some(filler_object_identifier) = fillers.pop() {
            publish_delete(&mut pool, arm.reclaim, filler_object_identifier);
        }
    }
    publish_delete(&mut pool, arm.reclaim, deleted_reference_object_identifier);

    // 连续 3 次触动，中间不尝试。
    let mut prefix_touches = 0u64;
    let mut degenerate = false;
    for _ in 0..3 {
        match fillers.pop() {
            Some(filler_object_identifier) => {
                if publish_delete(&mut pool, arm.reclaim, filler_object_identifier).is_none() {
                    degenerate = true;
                    break;
                }
                prefix_touches += 1;
            }
            None => {
                degenerate = true;
                break;
            }
        }
    }
    if degenerate {
        let summary = degenerate_or_wedged_summary(arm, geometry, history_label, &pool);
        return (summary, 0, pool.switches_so_far, 0);
    }

    let window_start = match position {
        FailurePosition::First => 1,
        FailurePosition::Cover => cover_push_index(&pool),
    };
    let switches_before = pool.switches_so_far;
    let pushes_before = pool.pushes_so_far;
    let mut injector = FailureInjector::new(window_start, consecutive_root_write_failure_count);
    let states_before_retry = distinct_valid_states(&pool);
    let outcome = {
        let mut failures = |last_txg| injector.next(last_txg);
        retry_loop(&mut pool, arm, geometry, &mut fillers, geometry.object_units, prefix_touches, prefix_touches, &mut failures)
    };
    let states_after_retry = distinct_valid_states(&pool);
    let retained_states_dropped_below_minimum = u64::from(states_before_retry >= MINIMUM_RETAINED_STATES && states_after_retry < MINIMUM_RETAINED_STATES);

    if injector.injected < consecutive_root_write_failure_count {
        // 这次准入里根槽写次数不够，k 次失败没有全部问到：登记 H4a／H4b／H5 的退化条款
        // （除非是真卡死——`degenerate_or_wedged_summary` 先问 `pool.allocation_failure_hit`）。
        let summary = degenerate_or_wedged_summary(arm, geometry, history_label, &pool);
        return (summary, injector.injected, pool.switches_so_far - switches_before, 0);
    }

    let (outcome_label, delete_then_rewrite_publish_count) = retry_outcome_label(outcome);
    let recovery_publishes_until_covered = pool.pushes_so_far - pushes_before;
    let summary = build_run_summary(
        arm,
        geometry,
        history_label,
        &pool,
        outcome_label,
        delete_then_rewrite_publish_count,
        matches!(outcome, RetryOutcome::Wedged),
        retained_states_dropped_below_minimum,
    );
    (summary, injector.injected, pool.switches_so_far - switches_before, recovery_publishes_until_covered)
}

// ===== H5′：注入点改成填满阶段自推的空发布（登记「十二、修订」第三段第 7 条第 2 类诊断，
// 主 agent 2026-09-17 第四段指定；不是登记「五」原文的一种历史，是原 H5 的一个变体） =====

/// H5′：与 H5 相同的 k ∈ {1,2,3} × 位 ∈ {首,覆}，区别只在注入点——不等「删后写回」那次准入里
/// 的推空，而是打在**填满阶段为抬 F 自己推的那几次空发布**上（第三段诊断：P-止 在「填满」阶段
/// 自身推了 2 次空发布把 F 从 0 抬到 1949 那一类，串重判-G7-止 在 H5 上因此有 73% 的格从没被
/// H5 的注入点碰到，退化）。用同一个 `LazyFailureInjector`，`window_start` 在第一次真的发生
/// 自推空发布时现算——若填满阶段全程一次都没自推，注入器在填满阶段问零次，`call_index` 保持
/// 0，之后把同一个注入器原样带进「删后写回」那次准入的 3 次触动之后的 `retry_loop`，
/// 那里第一次自推就会把 `window_start` 现算出来——等价于原样落回 H5 的注入点，满足
/// 「填满阶段没有自推空发布的格，照 H5 原样注入」。
///
/// 返回 `(RunSummary, injected, switches_so_far, recovery_publishes_until_covered, fill_phase_self_pushes)`：
/// 最后一个是诊断量——填满阶段里真的发生过几次自推空发布（> 0 就说明这一格的注入确实打在了
/// 填满阶段，= 0 就说明这一格退回了 H5 的注入点，两者都不算错，但报告要分清是哪一种，见登记
/// 「十二、修订」第四段）。`recovery_publishes_until_covered` 从 `fill_pool` 开始算（不是从
/// 3 次触动之后算）：注入可能发生在填满阶段，覆盖两块盘所花的总发布尝试数要从注入可能发生的
/// 最早点起算，否则会漏掉填满阶段里那部分。
fn run_fill_phase_injected_history(
    arm: Arm,
    geometry: Geometry,
    consecutive_root_write_failure_count: u64,
    position: FailurePosition,
    damage: Damage,
) -> (RunSummary, u64, u64, u64, u64) {
    let history_label = format!("H5'-{consecutive_root_write_failure_count}-{}", position.label());
    let mut pool = Pool::seed(geometry);
    pool.damage = damage;
    let mut injector = LazyFailureInjector::new(position, consecutive_root_write_failure_count);
    let pushes_before_fill = pool.pushes_so_far;
    let (deleted_reference_object_identifier, mut fillers) = {
        let mut failures = |last_txg| injector.next(last_txg);
        fill_pool(&mut pool, arm, geometry, &mut failures)
    };
    let fill_phase_self_pushes = pool.pushes_so_far - pushes_before_fill;

    for _ in 0..geometry.padding_deletes {
        if let Some(filler_object_identifier) = fillers.pop() {
            publish_delete(&mut pool, arm.reclaim, filler_object_identifier);
        }
    }
    publish_delete(&mut pool, arm.reclaim, deleted_reference_object_identifier);

    // 连续 3 次触动，中间不尝试（与 H5 同一前缀）。
    let mut prefix_touches = 0u64;
    let mut degenerate = false;
    for _ in 0..3 {
        match fillers.pop() {
            Some(filler_object_identifier) => {
                if publish_delete(&mut pool, arm.reclaim, filler_object_identifier).is_none() {
                    degenerate = true;
                    break;
                }
                prefix_touches += 1;
            }
            None => {
                degenerate = true;
                break;
            }
        }
    }
    if degenerate {
        let summary = degenerate_or_wedged_summary(arm, geometry, history_label, &pool);
        return (summary, injector.injected, pool.switches_so_far, 0, fill_phase_self_pushes);
    }

    let switches_before = pool.switches_so_far;
    let states_before_retry = distinct_valid_states(&pool);
    let outcome = {
        let mut failures = |last_txg| injector.next(last_txg);
        retry_loop(&mut pool, arm, geometry, &mut fillers, geometry.object_units, prefix_touches, prefix_touches, &mut failures)
    };
    let states_after_retry = distinct_valid_states(&pool);
    let retained_states_dropped_below_minimum = u64::from(states_before_retry >= MINIMUM_RETAINED_STATES && states_after_retry < MINIMUM_RETAINED_STATES);

    if injector.injected < consecutive_root_write_failure_count {
        // 窗口没走完——既没在填满阶段问够 k 次，也没在「删后写回」那次准入里问够（可能连
        // window_start 都没算出来，说明这次运行里一次自推空发布都没发生）：登记「退化」，
        // 与 H4a／H4b／H5 同一形态。
        let summary = degenerate_or_wedged_summary(arm, geometry, history_label, &pool);
        return (summary, injector.injected, pool.switches_so_far - switches_before, 0, fill_phase_self_pushes);
    }

    let (outcome_label, delete_then_rewrite_publish_count) = retry_outcome_label(outcome);
    let recovery_publishes_until_covered = pool.pushes_so_far - pushes_before_fill;
    let summary = build_run_summary(
        arm,
        geometry,
        history_label,
        &pool,
        outcome_label,
        delete_then_rewrite_publish_count,
        matches!(outcome, RetryOutcome::Wedged),
        retained_states_dropped_below_minimum,
    );
    (summary, injector.injected, pool.switches_so_far - switches_before, recovery_publishes_until_covered, fill_phase_self_pushes)
}

// ===== H6：管理员回退之后隔离了一批槽再写（登记「五、5.4」） =====

/// 管理员回退（D23（journal 的角色与格式） 已定项 14）：取新实例；`pool.slots`／`current_fixpoint_slots`／
/// `current_instance_table_slots` 从 `rollback_target` 的快照载入；影子账按窄读法（2026-09-16 用户定案，
/// D23（journal 的角色与格式） 已定项 14 正文「按主语『被抛弃时间线的根』读，仍被有效根引用的槽不在其内」）
/// 隔离「只被被抛弃根引用、不被候选根引用、且在 `rollback_target` 自己的快照里不是 Allocated」的槽；
/// 回退行 `(rollback_target.instance, rollback_target.txg, 0, is_rollback=true)` 写进实例表；回退那次发布非空（R9：
/// 它改了用户可见状态），带 F_生效（回退候选集已要求 `rollback_target.txg ≥ F_生效`，回退本身不改变 F）。
/// 返回这次回退 + 暖机一共消耗的发布尝试数；回退那次发布自己真的分配不到固定点 + 实例表槽
/// （近满盘，Q5a／Q5b 那类真卡死，不是 H6 故意注入的故障）时返回 `None`——回退不保证总能成功，
/// 这不是登记要它测的故障历史，但模型不能假装它不会发生（2026-09-17 冒烟跑撞到过，见登记
/// 「十二、修订」）。
///
/// ⚠️ **简化，不是登记的全部**：`pool.object_slots`（对象号 → 槽号）不按 `rollback_target` 重新对齐——
/// 本模型只追踪"当前还有哪些填充对象未删"这一件事（`fillers` 队列），回退之后可能删除的对象
/// 已经从队列与 `object_slots` 里一起弹出，之后的 `retry_loop` 只会碰未删过的对象，不会再引用
/// 已经不存在的对象号，所以这处简化不影响 Q1′／Q6／Q7 的计算；但它意味着本函数不模拟"对象在
/// 回退之后重新出现"这件事（H6 的问法也没有问这个）。
fn perform_rollback(pool: &mut Pool, reclaim: ReclaimTiming, rollback_target: &RootRecord) -> Option<u64> {
    let effective_floor = pool.effective_rollback_floor();
    let abandoned_txgs: Vec<u64> = pool
        .readable_roots()
        .filter(|root| root.txg > rollback_target.txg)
        .map(|root| root.txg)
        .collect();
    let candidate_txgs: Vec<u64> = pool
        .readable_roots()
        .filter(|root| root.txg <= rollback_target.txg && root.txg >= effective_floor)
        .map(|root| root.txg)
        .collect();
    // 先只读、把要隔离的槽号收集成一份独立的列表，再统一写 `pool.isolated`——`snapshot_of` 借用
    // 的是 `pool.readable_roots()`，与写 `pool.isolated` 不能同时借用同一个 `pool`。
    let snapshot_of = |txg: u64| -> Option<Rc<Vec<SlotState>>> {
        pool.readable_roots().find(|root| root.txg == txg).map(|root| root.slots_snapshot.clone())
    };
    let capacity = pool.slots.len();
    let mut slots_to_isolate = Vec::new();
    for index in 0..capacity {
        let referenced_by_abandoned = abandoned_txgs.iter().any(|&txg| {
            snapshot_of(txg).is_some_and(|snapshot| matches!(snapshot.get(index), Some(SlotState::Allocated)))
        });
        if !referenced_by_abandoned {
            continue;
        }
        let referenced_by_candidate = candidate_txgs.iter().any(|&txg| {
            snapshot_of(txg).is_some_and(|snapshot| matches!(snapshot.get(index), Some(SlotState::Allocated)))
        });
        if referenced_by_candidate {
            continue;
        }
        let allocated_in_rollback_target = matches!(rollback_target.slots_snapshot.get(index), Some(SlotState::Allocated));
        if allocated_in_rollback_target {
            continue;
        }
        slots_to_isolate.push(index);
    }
    for index in slots_to_isolate {
        pool.isolated[index] = true;
    }

    pool.slots = (*rollback_target.slots_snapshot).clone();
    pool.current_fixpoint_slots = (*rollback_target.fixpoint_slots_snapshot).clone();
    pool.current_instance_table_slots = (*rollback_target.instance_table_slots_snapshot).clone();

    pool.instance_table.push(InstanceRow {
        instance: rollback_target.instance,
        selected_root_txg: rollback_target.txg,
        applied_transaction_high_water: 0,
        is_rollback: true,
    });
    pool.instance += 1;

    // 回退那次发布不在故障历史范围内（H6 不注入根槽写失败），但它和任何发布一样要真的分配得到
    // 固定点 + 2 个实例表槽——近满盘时这个分配可以真的失败（`attempt_publish` 已经把
    // `pool.allocation_failure_hit` 置位、按 Q5a／Q5b 的口径处理），不能假定它总是成功。
    let rollback_floor = pool.effective_rollback_floor();
    let rows = pool.instance_table.clone();
    let rollback_txg = match attempt_publish(pool, reclaim, true, rollback_floor, None, None, true, Some(rows), false) {
        Ok(txg) => txg,
        Err(PublishAttemptFailed) => return None,
    };

    let mut attempts = 1u64;
    let mut covered = vec![device_of_txg(rollback_txg)];
    let all_devices = [0u64, 1u64];
    while all_devices.iter().any(|device| !covered.contains(device)) && attempts - 1 < RING_REGIONS {
        let current_floor = pool.newest_persisted_floor();
        let warm_result = attempt_publish(pool, reclaim, false, current_floor, None, None, false, None, false);
        attempts += 1;
        let Ok(warm_txg) = warm_result else { break };
        let device = device_of_txg(warm_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
    }
    Some(attempts)
}

/// H6（登记「五、5.4」）：H1 跑到 Y 成功（没成功记「退化」）；再触动 2 次；管理员回退到
/// `R_old` = Y 被封进的那次发布之前最新的持久有效根（要求 `R_old` 在候选集里，不在记「回退够不着」，
/// 这一格同样退化）；回退与暖机之后尝试写 Y2，进重试循环——这里的 Q1（`RunSummary` 里那一栏）
/// 就是登记「六」的 Q1′（回退之后再写要几次非空发布）。
///
/// 返回 `(RunSummary, isolated_slots_after_rollback)`：后者是回退之后 `pool.isolated` 里
/// 置位的槽数，对应登记「H6：回退之后隔离了一批槽再写」那句话本身——它是不是 > 0，是这一格
/// 最直接能核的事实。
fn run_rollback_history(arm: Arm, geometry: Geometry, damage: Damage) -> (RunSummary, u64) {
    let mut pool = Pool::seed(geometry);
    pool.damage = damage;
    let (deleted_reference_object_identifier, mut fillers) = fill_pool(&mut pool, arm, geometry, &mut |_last_txg| false);
    for _ in 0..geometry.padding_deletes {
        if let Some(filler_object_identifier) = fillers.pop() {
            publish_delete(&mut pool, arm.reclaim, filler_object_identifier);
        }
    }
    publish_delete(&mut pool, arm.reclaim, deleted_reference_object_identifier);
    let last_txg_before_rewrite_attempt = pool.readable_roots().map(|root| root.txg).max().unwrap_or(0);
    let outcome = retry_loop(&mut pool, arm, geometry, &mut fillers, geometry.object_units, 0, 0, |_last_txg| false);
    let degenerate_summary =
        |pool: &Pool| degenerate_or_wedged_summary(arm, geometry, "H6".to_string(), pool);
    if !matches!(outcome, RetryOutcome::Success { .. } | RetryOutcome::OverBound { .. }) {
        // H1 到 Y 那一步没成功：outcome 本身已经知道是卡死还是未成／退化，直接用它的标签
        // （不是「猜」`allocation_failure_hit`——这里已经有真实的 `RetryOutcome` 可读）。
        let (label, count) = retry_outcome_label(outcome);
        let summary = build_run_summary(arm, geometry, "H6".to_string(), &pool, label, count, matches!(outcome, RetryOutcome::Wedged), 0);
        return (summary, 0);
    }

    // R_old = Y 被封进的那次发布之前最新的持久有效根：环里 txg 严格小于「删 X 之后到 Y 成功为止
    // 新产生的第一条根」的、这个前缀阶段本来就有的最新根。因为 Y 的重试循环从 `last_txg_before_rewrite_attempt` 之后
    // 才开始产生新根，`last_txg_before_rewrite_attempt` 本身就是「Y 被封进的那次发布之前最新的持久根」。
    let rollback_target_root = pool.readable_roots().find(|root| root.txg == last_txg_before_rewrite_attempt).cloned();
    let Some(rollback_target_root) = rollback_target_root else {
        return (degenerate_summary(&pool), 0);
    };

    for _ in 0..2 {
        match fillers.pop() {
            Some(filler_object_identifier) => {
                if publish_delete(&mut pool, arm.reclaim, filler_object_identifier).is_none() {
                    return (degenerate_summary(&pool), 0);
                }
            }
            None => return (degenerate_summary(&pool), 0),
        }
    }

    let effective_floor_now = pool.effective_rollback_floor();
    if rollback_target_root.txg < effective_floor_now || pool.is_abandoned(&rollback_target_root) {
        // 回退够不着（不在候选集里）：登记明令这一格记「回退够不着」，退化。
        return (degenerate_summary(&pool), 0);
    }

    if perform_rollback(&mut pool, arm.reclaim, &rollback_target_root).is_none() {
        // 回退那次发布自己真的分配不到（近满盘，Q5a／Q5b）：记卡死，不是「退化」——它是这一格
        // 真实发生的结果，不是「这次准入根本没走到该走的路径」。
        let summary = build_run_summary(arm, geometry, "H6".to_string(), &pool, "卡死".to_string(), -2, true, 0);
        return (summary, pool.isolated_count());
    }
    let isolated_slots_after_rollback = pool.isolated_count();

    let states_before_retry = distinct_valid_states(&pool);
    let outcome2 = retry_loop(&mut pool, arm, geometry, &mut fillers, geometry.object_units, 0, 0, |_last_txg| false);
    let states_after_retry = distinct_valid_states(&pool);
    let retained_states_dropped_below_minimum = u64::from(states_before_retry >= MINIMUM_RETAINED_STATES && states_after_retry < MINIMUM_RETAINED_STATES);
    let (outcome_label, delete_then_rewrite_publish_count) = retry_outcome_label(outcome2);
    let summary = build_run_summary(
        arm,
        geometry,
        "H6".to_string(),
        &pool,
        outcome_label,
        delete_then_rewrite_publish_count,
        matches!(outcome2, RetryOutcome::Wedged),
        retained_states_dropped_below_minimum,
    );
    (summary, isolated_slots_after_rollback)
}

// ===== H4a／H4b：推空发布之中崩溃，第一条带新 F 的根只落了一块盘，再重开（登记「五、5.4」，
// 2026-09-17 第四段） =====

/// 两个崩溃点，登记原文：
/// - H4a「这次准入里第一条带新 F 的推空根持久之后立刻崩溃（这条根只在一块盘上）」；
/// - H4b「崩在这次准入的第二次推空已经分配、单元写完、根还没持久的时候」。
///
/// ⚠️ **两者在本模型里落到同一条代码路径**：`attempt_publish` 是一次性完成「释放 → 分配 →
/// 写根 → 持久」，不暴露「已分配、单元写完、根还没持久」这个中间态——崩在这个中间态，与崩在
/// 「上一次已持久之后、下一次还没开始」，两种说法在本模型能表达的粒度上，恢复选到的根都是
/// 「崩溃前最后一条已持久的根」，完全相同（H4b 那次「已经分配」的槽从没进过 `pool.ring`，
/// 崩溃即丢，等价于那次推空从没发生过）。这不是取样点选窄了，是 `attempt_publish` 原子性
/// 粒度决定的、与 M12「暖机等价变异」同一类（先证明等价，再当等价留档，不算漏测）——
/// `crash_point_equivalence_is_documented_not_assumed` 那条单测把这句话钉成断言。History
/// 标签仍按 `CrashPoint::label()` 分开打两条，供报告按标签统计。真要把两者分开，需要先把
/// `attempt_publish` 拆成「分配」与「写根持久」两个能单独崩溃的步骤，这份登记没有要求这一段
/// 做到（与 H6 `perform_rollback` 头部「简化，不是登记的全部」同一处置）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CrashPoint {
    AfterFirstPush,
    BeforeSecondPush,
}

impl CrashPoint {
    fn label(self) -> &'static str {
        match self {
            CrashPoint::AfterFirstPush => "H4a",
            CrashPoint::BeforeSecondPush => "H4b",
        }
    }
}

/// 崩溃与重开（5.1「崩溃与重开」行）：进程没了——扣住、在飞、影子账都丢；恢复选 txg 最大的
/// 持久有效根，从它的快照载入槽状态；取新实例、写行发布、暖机到本实例的根覆盖两块盘、M_sw
/// 回到 4 份再扣一份——这后半段（新实例、写行、暖机、扣一份切换预留）与「实例切换」（R8）
/// 逐字相同，直接复用 `perform_instance_switch`；这个函数只做前半段：选根、载入槽状态、
/// 清扣住／影子账、把切换预留重置回 4 份。写行那次发布之前不再推空发布——崩溃重开走的是
/// 恢复，不是准入，`perform_instance_switch` 本来就不推空，只写行 + 暖机。
fn crash_and_reopen(pool: &mut Pool, reclaim: ReclaimTiming) {
    let recovery_root = pool
        .readable_roots()
        .filter(|root| !pool.is_abandoned(root))
        .max_by_key(|root| root.txg)
        .cloned()
        .expect("起点状态至少有 mkfs 那条根，readable_roots 不会空");
    pool.slots = (*recovery_root.slots_snapshot).clone();
    pool.current_fixpoint_slots = (*recovery_root.fixpoint_slots_snapshot).clone();
    pool.current_instance_table_slots = (*recovery_root.instance_table_slots_snapshot).clone();
    pool.isolated.iter_mut().for_each(|flag| *flag = false);
    pool.pending_hold_floor = None;
    pool.switch_reserve_shares_per_disk = INSTANCE_SWITCH_LIMIT;
    let mut no_further_failures = |_last_txg: u64| false;
    perform_instance_switch(pool, reclaim, &mut no_further_failures);
}

/// H4a／H4b：同 H3a 的前缀到 3 次触动；尝试写 Y；这次准入里让恰好 1 次自推空发布持久
/// （见上方 ⚠️，H4a／H4b 因此共用这一步），然后崩溃、重开；重试循环接着走（触动计数接着数）。
///
/// 这次准入里自推次数一次都没发生（闸不用推空就过，或这条臂第一道闸不重判）：崩溃点不存在，
/// 记「退化」，与 H4a／H4b／H5 那句「这次准入里根本没有推空的臂上……记『退化』」同一形态。
/// 那一次自推自己就真的分配不到固定点（近满盘，PC5 那一类）：记「卡死」，不是「退化」——
/// 与 H4a／H4b／H5 要测的「崩溃再重开」是两件事。
///
/// 返回 `(RunSummary, self_pushes_before_crash, recovery_publishes_after_reopen)`：前者应恒为
/// 0（退化／卡死）或 1（真的崩到了）；后者是重开之后到 Y 最终批准为止又发生了几次发布尝试
/// （写行 + 暖机 + 之后可能还要的推空），诊断量，口径与 H5 的 `recovery_publishes_until_covered`
/// 相同（不是登记「六」Q4 的精确分量）。
fn run_crash_mid_push_history(arm: Arm, geometry: Geometry, crash_point: CrashPoint, damage: Damage) -> (RunSummary, u64, u64) {
    let history_label = crash_point.label().to_string();
    let mut pool = Pool::seed(geometry);
    pool.damage = damage;
    let (deleted_reference_object_identifier, mut fillers) = fill_pool(&mut pool, arm, geometry, &mut |_last_txg| false);
    for _ in 0..geometry.padding_deletes {
        if let Some(filler_object_identifier) = fillers.pop() {
            publish_delete(&mut pool, arm.reclaim, filler_object_identifier);
        }
    }
    publish_delete(&mut pool, arm.reclaim, deleted_reference_object_identifier);

    let mut prefix_touches = 0u64;
    let mut degenerate = false;
    for _ in 0..3 {
        match fillers.pop() {
            Some(filler_object_identifier) => {
                if publish_delete(&mut pool, arm.reclaim, filler_object_identifier).is_none() {
                    degenerate = true;
                    break;
                }
                prefix_touches += 1;
            }
            None => {
                degenerate = true;
                break;
            }
        }
    }
    if degenerate {
        let summary = degenerate_or_wedged_summary(arm, geometry, history_label, &pool);
        return (summary, 0, 0);
    }

    let need = geometry.object_units * 2;
    let (gate1, _) = compute_gate1(&pool, arm, 0);
    if gate1 >= need || !arm.gate.rejudges_first_gate() {
        // 闸不用推空就过，或这条臂第一道闸不重判：这次准入根本走不到「自推空发布」，
        // 崩溃点不存在——退化。
        let summary = degenerate_or_wedged_summary(arm, geometry, history_label, &pool);
        return (summary, 0, 0);
    }
    let mut no_injected_failures = |_last_txg: u64| false;
    push_one_empty_publish(&mut pool, arm.reclaim, &mut no_injected_failures);
    if pool.allocation_failure_hit {
        // 这一次自推自己就真的卡死了（近满盘）：与 H4a／H4b 要测的「崩溃再重开」是两件事。
        let summary = degenerate_or_wedged_summary(arm, geometry, history_label, &pool);
        return (summary, 0, 0);
    }
    let self_pushes_before_crash = 1u64;

    crash_and_reopen(&mut pool, arm.reclaim);
    if pool.allocation_failure_hit {
        let summary = build_run_summary(arm, geometry, history_label, &pool, "卡死".to_string(), -2, true, 0);
        return (summary, self_pushes_before_crash, 0);
    }

    let pushes_before_retry = pool.pushes_so_far;
    let states_before_retry = distinct_valid_states(&pool);
    let outcome = retry_loop(&mut pool, arm, geometry, &mut fillers, geometry.object_units, prefix_touches, prefix_touches, |_last_txg| false);
    let states_after_retry = distinct_valid_states(&pool);
    let retained_states_dropped_below_minimum = u64::from(states_before_retry >= MINIMUM_RETAINED_STATES && states_after_retry < MINIMUM_RETAINED_STATES);
    let (outcome_label, delete_then_rewrite_publish_count) = retry_outcome_label(outcome);
    let recovery_publishes_after_reopen = pool.pushes_so_far - pushes_before_retry;
    let summary = build_run_summary(
        arm,
        geometry,
        history_label,
        &pool,
        outcome_label,
        delete_then_rewrite_publish_count,
        matches!(outcome, RetryOutcome::Wedged),
        retained_states_dropped_below_minimum,
    );
    (summary, self_pushes_before_crash, recovery_publishes_after_reopen)
}

fn run_summary_line(summary: &RunSummary) -> String {
    format!(
        "RUN gate={} baseline={} reclaim={} stop={} history={} c={} n={} c_max={} s={} pad={} delete_then_rewrite_outcome_label={} delete_then_rewrite_publish_count={} df_reported_enough_but_write_failed_count={} admission_passed_but_allocation_failed_count={} budget_was_binding={} maximum_publish_attempts_seen_field={} allocation_failure_hit_field={} wedged={} unsafe_candidate_reuse_count_field={} retained_states_dropped_below_minimum={} filler_objects_written_so_far={}",
        summary.arm,
        summary.is_real_baseline,
        summary.reclaim_label,
        summary.stop_label,
        summary.history,
        summary.capacity,
        summary.object_units,
        summary.fixpoint_cost,
        summary.slots_per_region,
        summary.padding_deletes,
        summary.delete_then_rewrite_outcome_label,
        summary.delete_then_rewrite_publish_count,
        summary.df_reported_enough_but_write_failed_count,
        summary.admission_passed_but_allocation_failed_count,
        summary.budget_was_binding,
        summary.maximum_publish_attempts_seen_field,
        summary.allocation_failure_hit_field,
        summary.wedged,
        summary.unsafe_candidate_reuse_count_field,
        summary.retained_states_dropped_below_minimum,
        summary.filler_objects_written_in_fill_phase,
    )
}

fn main() {
    let mut emitter = Emitter::new();
    let mut run_count = 0u64;
    for geometry in all_geometries() {
        for arm in all_arms() {
            let summary = run_delete_then_rewrite_history(arm, geometry);
            println!("{}", emitter.emit_raw(&run_summary_line(&summary)));
            run_count += 1;
        }
    }

    // H5-k-位（登记「五、5.4」，2026-09-17 第二段跑补）：k ∈ {1,2,3} × 位 ∈ {首,覆}，24 臂 × 48 格。
    for consecutive_root_write_failure_count in [1u64, 2, 3] {
        for position in [FailurePosition::First, FailurePosition::Cover] {
            for geometry in all_geometries() {
                for arm in all_arms() {
                    let (summary, injected, switches_triggered, recovery_publishes_until_covered) =
                        run_root_write_failure_history(arm, geometry, consecutive_root_write_failure_count, position, Damage::default());
                    println!("{}", emitter.emit_raw(&run_summary_line(&summary)));
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "H5DETAIL gate={} reclaim={} stop={} k={} position={} c={} n={} c_max={} s={} pad={} injected_failures={} switches_triggered={} recovery_publishes_until_covered={}",
                            summary.arm,
                            summary.reclaim_label,
                            summary.stop_label,
                            consecutive_root_write_failure_count,
                            position.label(),
                            geometry.capacity_slots_per_disk,
                            geometry.object_units,
                            geometry.fixpoint_cost,
                            geometry.slots_per_region,
                            geometry.padding_deletes,
                            injected,
                            switches_triggered,
                            recovery_publishes_until_covered,
                        ))
                    );
                    run_count += 1;
                }
            }
        }
    }

    // H5′（2026-09-17 第四段，主 agent 定）：同一套 k × 位，注入点改成填满阶段的自推空发布，
    // 填满阶段不自推时原样落回 H5 的注入点；24 臂 × 48 格。
    for consecutive_root_write_failure_count in [1u64, 2, 3] {
        for position in [FailurePosition::First, FailurePosition::Cover] {
            for geometry in all_geometries() {
                for arm in all_arms() {
                    let (summary, injected, switches_triggered, recovery_publishes_until_covered, fill_phase_self_pushes) =
                        run_fill_phase_injected_history(arm, geometry, consecutive_root_write_failure_count, position, Damage::default());
                    println!("{}", emitter.emit_raw(&run_summary_line(&summary)));
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "H5PRIMEDETAIL gate={} reclaim={} stop={} k={} position={} c={} n={} c_max={} s={} pad={} injected_failures={} switches_triggered={} recovery_publishes_until_covered={} fill_phase_self_pushes={}",
                            summary.arm,
                            summary.reclaim_label,
                            summary.stop_label,
                            consecutive_root_write_failure_count,
                            position.label(),
                            geometry.capacity_slots_per_disk,
                            geometry.object_units,
                            geometry.fixpoint_cost,
                            geometry.slots_per_region,
                            geometry.padding_deletes,
                            injected,
                            switches_triggered,
                            recovery_publishes_until_covered,
                            fill_phase_self_pushes,
                        ))
                    );
                    run_count += 1;
                }
            }
        }
    }

    // H6（登记「五、5.4」，2026-09-17 第二段跑补）：24 臂 × 48 格。
    for geometry in all_geometries() {
        for arm in all_arms() {
            let (summary, isolated_slots_after_rollback) = run_rollback_history(arm, geometry, Damage::default());
            println!("{}", emitter.emit_raw(&run_summary_line(&summary)));
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "H6DETAIL gate={} reclaim={} stop={} c={} n={} c_max={} s={} pad={} isolated_slots_after_rollback={}",
                    summary.arm,
                    summary.reclaim_label,
                    summary.stop_label,
                    geometry.capacity_slots_per_disk,
                    geometry.object_units,
                    geometry.fixpoint_cost,
                    geometry.slots_per_region,
                    geometry.padding_deletes,
                    isolated_slots_after_rollback,
                ))
            );
            run_count += 1;
        }
    }

    // H4a／H4b（登记「五、5.4」，2026-09-17 第四段跑补）：推空发布之中崩溃再重开；24 臂 × 48 格。
    for crash_point in [CrashPoint::AfterFirstPush, CrashPoint::BeforeSecondPush] {
        for geometry in all_geometries() {
            for arm in all_arms() {
                let (summary, self_pushes_before_crash, recovery_publishes_after_reopen) =
                    run_crash_mid_push_history(arm, geometry, crash_point, Damage::default());
                println!("{}", emitter.emit_raw(&run_summary_line(&summary)));
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "H4DETAIL gate={} reclaim={} stop={} crash_point={} c={} n={} c_max={} s={} pad={} self_pushes_before_crash={} recovery_publishes_after_reopen={}",
                        summary.arm,
                        summary.reclaim_label,
                        summary.stop_label,
                        crash_point.label(),
                        geometry.capacity_slots_per_disk,
                        geometry.object_units,
                        geometry.fixpoint_cost,
                        geometry.slots_per_region,
                        geometry.padding_deletes,
                        self_pushes_before_crash,
                        recovery_publishes_after_reopen,
                    ))
                );
                run_count += 1;
            }
        }
    }

    // Q3：B=∞ 反事实（登记「五、5.5」「不设上限；P-满 改成推到闸过或本次准入推满 10×3S 次为止」），
    // 同臂同格与 B=8 的基线比——H1、全部 24 臂 × 48 格。
    let mut unbounded_budget_diverged_count = 0u64;
    for geometry in all_geometries() {
        let unbounded_budget = 10 * 3 * geometry.slots_per_region;
        for arm in all_arms() {
            let baseline = run_delete_then_rewrite_history(arm, geometry);
            let unbounded = run_delete_then_rewrite_history_with_damage(
                arm,
                geometry,
                Damage { budget_override: Some(unbounded_budget), ..Damage::default() },
            );
            let diverged = baseline.delete_then_rewrite_outcome_label != unbounded.delete_then_rewrite_outcome_label
                || baseline.delete_then_rewrite_publish_count != unbounded.delete_then_rewrite_publish_count
                || baseline.wedged != unbounded.wedged;
            if diverged {
                unbounded_budget_diverged_count += 1;
            }
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "Q3 gate={} reclaim={} stop={} history=H1 c={} n={} c_max={} s={} pad={} baseline_q1={}/{} unbounded_q1={}/{} baseline_wedged={} unbounded_wedged={} diverged={}",
                    baseline.arm,
                    baseline.reclaim_label,
                    baseline.stop_label,
                    geometry.capacity_slots_per_disk,
                    geometry.object_units,
                    geometry.fixpoint_cost,
                    geometry.slots_per_region,
                    geometry.padding_deletes,
                    baseline.delete_then_rewrite_outcome_label,
                    baseline.delete_then_rewrite_publish_count,
                    unbounded.delete_then_rewrite_outcome_label,
                    unbounded.delete_then_rewrite_publish_count,
                    baseline.wedged,
                    unbounded.wedged,
                    diverged,
                ))
            );
            run_count += 1;
        }
    }

    // 阳性对照（5.6）：每条改坏在指定的格上，遍历全部 24 条臂。
    let pc_geometry = Geometry {
        capacity_slots_per_disk: 4000,
        object_units: 16,
        fixpoint_cost: 4,
        slots_per_region: 8,
        padding_deletes: 2,
    };
    let mut pc1_hit_any = false;
    let mut pc2_hit_any = false;
    let mut pc4_hit_any = false;
    let mut pc5_hit_any = false;
    for arm in all_arms() {
        let baseline = run_delete_then_rewrite_history(arm, pc_geometry);
        let pc1 = run_delete_then_rewrite_history_with_damage(
            arm,
            pc_geometry,
            Damage { disable_floor_raise: true, ..Damage::default() },
        );
        if pc1.delete_then_rewrite_outcome_label != baseline.delete_then_rewrite_outcome_label || pc1.delete_then_rewrite_publish_count != baseline.delete_then_rewrite_publish_count {
            pc1_hit_any = true;
        }
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "PC name=pc1 arm={} baseline_q1={}/{} damaged_q1={}/{}",
                arm.label(),
                baseline.delete_then_rewrite_outcome_label,
                baseline.delete_then_rewrite_publish_count,
                pc1.delete_then_rewrite_outcome_label,
                pc1.delete_then_rewrite_publish_count,
            ))
        );

        let pc2 = run_delete_then_rewrite_history_with_damage(arm, pc_geometry, Damage { raw_df: true, ..Damage::default() });
        if pc2.df_reported_enough_but_write_failed_count >= 1 {
            pc2_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc2 arm={} df_reported_enough_but_write_failed_count={}", arm.label(), pc2.df_reported_enough_but_write_failed_count)));

        let pc4_damage = if arm.reclaim == ReclaimTiming::AfterEffectiveFloor {
            Damage { reclaim_on_first_new_floor_root: true, ..Damage::default() }
        } else {
            Damage { skip_hold: true, ..Damage::default() }
        };
        let pc4 = run_delete_then_rewrite_history_with_damage(arm, pc_geometry, pc4_damage);
        if pc4.unsafe_candidate_reuse_count_field >= 1 {
            pc4_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc4 arm={} unsafe_candidate_reuse_count_field={}", arm.label(), pc4.unsafe_candidate_reuse_count_field)));
        run_count += 3;
    }
    let pc5_geometry = Geometry {
        capacity_slots_per_disk: 600,
        object_units: 1,
        fixpoint_cost: 9,
        slots_per_region: 8,
        padding_deletes: 2,
    };
    for arm in all_arms() {
        let pc5 = run_delete_then_rewrite_history_with_damage(arm, pc5_geometry, Damage { zero_reserves: true, ..Damage::default() });
        if pc5.wedged {
            pc5_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc5 arm={} wedged={}", arm.label(), pc5.wedged)));
        run_count += 1;
    }

    // 阳性对照对新历史也要跑（2026-09-17 主 agent 指示）：H5 用 k=1、位=首（最简单、最容易命中
    // 的组合，与 H1 用同一组 PC 几何格）；H6 与 H1 完全共用 fill_pool／retry_loop／publish_delete，
    // 按同样的道理重跑一遍。
    let mut pc1_root_write_failure_hit_any = false;
    let mut pc2_root_write_failure_hit_any = false;
    let mut pc4_root_write_failure_hit_any = false;
    let mut pc5_root_write_failure_hit_any = false;
    let mut pc1_rollback_hit_any = false;
    let mut pc2_rollback_hit_any = false;
    let mut pc4_rollback_hit_any = false;
    let mut pc5_rollback_hit_any = false;
    // H4（`CrashPoint::AfterFirstPush`）与 H5′（k=1、位=首，与其它历史的阳性对照同一个理由：
    // 最简单、最容易命中的组合）也要跑（2026-09-17 第四段，主 agent 指示「阳性对照对 H4、H5′
    // 也跑」）。H5′ 的 PC5 复用 H5 探针探出来的 k=3／位=覆——`pc5_geometry` 不落在诊断报的
    // 「n=16×S=4」自推区间里，`LazyFailureInjector` 大概率在填满阶段问零次、原样落回 H5 的
    // 注入点，理由与判定同 H5 的 PC5。
    let mut pc1_crash_mid_push_hit_any = false;
    let mut pc2_crash_mid_push_hit_any = false;
    let mut pc4_crash_mid_push_hit_any = false;
    let mut pc5_crash_mid_push_hit_any = false;
    let mut pc1_fill_phase_injected_hit_any = false;
    let mut pc2_fill_phase_injected_hit_any = false;
    let mut pc4_fill_phase_injected_hit_any = false;
    let mut pc5_fill_phase_injected_hit_any = false;
    for arm in all_arms() {
        let (root_write_failure_baseline, ..) = run_root_write_failure_history(arm, pc_geometry, 1, FailurePosition::First, Damage::default());
        let (root_write_failure_pc1, ..) = run_root_write_failure_history(arm, pc_geometry, 1, FailurePosition::First, Damage { disable_floor_raise: true, ..Damage::default() });
        if root_write_failure_pc1.delete_then_rewrite_outcome_label != root_write_failure_baseline.delete_then_rewrite_outcome_label
            || root_write_failure_pc1.delete_then_rewrite_publish_count != root_write_failure_baseline.delete_then_rewrite_publish_count
        {
            pc1_root_write_failure_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc1-root_write_failure arm={} baseline_q1={}/{} damaged_q1={}/{}", arm.label(), root_write_failure_baseline.delete_then_rewrite_outcome_label, root_write_failure_baseline.delete_then_rewrite_publish_count, root_write_failure_pc1.delete_then_rewrite_outcome_label, root_write_failure_pc1.delete_then_rewrite_publish_count)));

        let (root_write_failure_pc2, ..) = run_root_write_failure_history(arm, pc_geometry, 1, FailurePosition::First, Damage { raw_df: true, ..Damage::default() });
        if root_write_failure_pc2.df_reported_enough_but_write_failed_count >= 1 {
            pc2_root_write_failure_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc2-root_write_failure arm={} df_reported_enough_but_write_failed_count={}", arm.label(), root_write_failure_pc2.df_reported_enough_but_write_failed_count)));

        let pc4_root_write_failure_damage = if arm.reclaim == ReclaimTiming::AfterEffectiveFloor {
            Damage { reclaim_on_first_new_floor_root: true, ..Damage::default() }
        } else {
            Damage { skip_hold: true, ..Damage::default() }
        };
        let (root_write_failure_pc4, ..) = run_root_write_failure_history(arm, pc_geometry, 1, FailurePosition::First, pc4_root_write_failure_damage);
        if root_write_failure_pc4.unsafe_candidate_reuse_count_field >= 1 {
            pc4_root_write_failure_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc4-root_write_failure arm={} unsafe_candidate_reuse_count_field={}", arm.label(), root_write_failure_pc4.unsafe_candidate_reuse_count_field)));

        // k=1／首 在这个几何上对每条臂都会退化（zero_reserves 让 gate1 少两项减项，3 次触动后常常
        // 不用推空就批准）；k=3／位=覆需要推得更久，才撞得到真卡死（见 `positive_control_pc5_wedges_root_write_failure_when_reserves_zero`）。
        let (root_write_failure_pc5, ..) = run_root_write_failure_history(arm, pc5_geometry, 3, FailurePosition::Cover, Damage { zero_reserves: true, ..Damage::default() });
        if root_write_failure_pc5.wedged {
            pc5_root_write_failure_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc5-root_write_failure arm={} wedged={}", arm.label(), root_write_failure_pc5.wedged)));

        let (rollback_baseline, _) = run_rollback_history(arm, pc_geometry, Damage::default());
        let (rollback_pc1, _) = run_rollback_history(arm, pc_geometry, Damage { disable_floor_raise: true, ..Damage::default() });
        if rollback_pc1.delete_then_rewrite_outcome_label != rollback_baseline.delete_then_rewrite_outcome_label
            || rollback_pc1.delete_then_rewrite_publish_count != rollback_baseline.delete_then_rewrite_publish_count
        {
            pc1_rollback_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc1-rollback arm={} baseline_q1={}/{} damaged_q1={}/{}", arm.label(), rollback_baseline.delete_then_rewrite_outcome_label, rollback_baseline.delete_then_rewrite_publish_count, rollback_pc1.delete_then_rewrite_outcome_label, rollback_pc1.delete_then_rewrite_publish_count)));

        let (rollback_pc2, _) = run_rollback_history(arm, pc_geometry, Damage { raw_df: true, ..Damage::default() });
        if rollback_pc2.df_reported_enough_but_write_failed_count >= 1 {
            pc2_rollback_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc2-rollback arm={} df_reported_enough_but_write_failed_count={}", arm.label(), rollback_pc2.df_reported_enough_but_write_failed_count)));

        let pc4_rollback_damage = if arm.reclaim == ReclaimTiming::AfterEffectiveFloor {
            Damage { reclaim_on_first_new_floor_root: true, ..Damage::default() }
        } else {
            Damage { skip_hold: true, ..Damage::default() }
        };
        let (rollback_pc4, _) = run_rollback_history(arm, pc_geometry, pc4_rollback_damage);
        if rollback_pc4.unsafe_candidate_reuse_count_field >= 1 {
            pc4_rollback_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc4-rollback arm={} unsafe_candidate_reuse_count_field={}", arm.label(), rollback_pc4.unsafe_candidate_reuse_count_field)));

        let (rollback_pc5, _) = run_rollback_history(arm, pc5_geometry, Damage { zero_reserves: true, ..Damage::default() });
        if rollback_pc5.wedged {
            pc5_rollback_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc5-rollback arm={} wedged={}", arm.label(), rollback_pc5.wedged)));

        let (crash_mid_push_baseline, ..) = run_crash_mid_push_history(arm, pc_geometry, CrashPoint::AfterFirstPush, Damage::default());
        let (crash_mid_push_pc1, ..) = run_crash_mid_push_history(arm, pc_geometry, CrashPoint::AfterFirstPush, Damage { disable_floor_raise: true, ..Damage::default() });
        if crash_mid_push_pc1.delete_then_rewrite_outcome_label != crash_mid_push_baseline.delete_then_rewrite_outcome_label
            || crash_mid_push_pc1.delete_then_rewrite_publish_count != crash_mid_push_baseline.delete_then_rewrite_publish_count
        {
            pc1_crash_mid_push_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc1-crash_mid_push arm={} baseline_q1={}/{} damaged_q1={}/{}", arm.label(), crash_mid_push_baseline.delete_then_rewrite_outcome_label, crash_mid_push_baseline.delete_then_rewrite_publish_count, crash_mid_push_pc1.delete_then_rewrite_outcome_label, crash_mid_push_pc1.delete_then_rewrite_publish_count)));

        let (crash_mid_push_pc2, ..) = run_crash_mid_push_history(arm, pc_geometry, CrashPoint::AfterFirstPush, Damage { raw_df: true, ..Damage::default() });
        if crash_mid_push_pc2.df_reported_enough_but_write_failed_count >= 1 {
            pc2_crash_mid_push_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc2-crash_mid_push arm={} df_reported_enough_but_write_failed_count={}", arm.label(), crash_mid_push_pc2.df_reported_enough_but_write_failed_count)));

        let pc4_crash_mid_push_damage = if arm.reclaim == ReclaimTiming::AfterEffectiveFloor {
            Damage { reclaim_on_first_new_floor_root: true, ..Damage::default() }
        } else {
            Damage { skip_hold: true, ..Damage::default() }
        };
        let (crash_mid_push_pc4, ..) = run_crash_mid_push_history(arm, pc_geometry, CrashPoint::AfterFirstPush, pc4_crash_mid_push_damage);
        if crash_mid_push_pc4.unsafe_candidate_reuse_count_field >= 1 {
            pc4_crash_mid_push_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc4-crash_mid_push arm={} unsafe_candidate_reuse_count_field={}", arm.label(), crash_mid_push_pc4.unsafe_candidate_reuse_count_field)));

        let (crash_mid_push_pc5, ..) = run_crash_mid_push_history(arm, pc5_geometry, CrashPoint::AfterFirstPush, Damage { zero_reserves: true, ..Damage::default() });
        if crash_mid_push_pc5.wedged {
            pc5_crash_mid_push_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc5-crash_mid_push arm={} wedged={}", arm.label(), crash_mid_push_pc5.wedged)));

        let (fill_phase_injected_baseline, ..) = run_fill_phase_injected_history(arm, pc_geometry, 1, FailurePosition::First, Damage::default());
        let (fill_phase_injected_pc1, ..) = run_fill_phase_injected_history(arm, pc_geometry, 1, FailurePosition::First, Damage { disable_floor_raise: true, ..Damage::default() });
        if fill_phase_injected_pc1.delete_then_rewrite_outcome_label != fill_phase_injected_baseline.delete_then_rewrite_outcome_label
            || fill_phase_injected_pc1.delete_then_rewrite_publish_count != fill_phase_injected_baseline.delete_then_rewrite_publish_count
        {
            pc1_fill_phase_injected_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc1-h5prime arm={} baseline_q1={}/{} damaged_q1={}/{}", arm.label(), fill_phase_injected_baseline.delete_then_rewrite_outcome_label, fill_phase_injected_baseline.delete_then_rewrite_publish_count, fill_phase_injected_pc1.delete_then_rewrite_outcome_label, fill_phase_injected_pc1.delete_then_rewrite_publish_count)));

        let (fill_phase_injected_pc2, ..) = run_fill_phase_injected_history(arm, pc_geometry, 1, FailurePosition::First, Damage { raw_df: true, ..Damage::default() });
        if fill_phase_injected_pc2.df_reported_enough_but_write_failed_count >= 1 {
            pc2_fill_phase_injected_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc2-h5prime arm={} df_reported_enough_but_write_failed_count={}", arm.label(), fill_phase_injected_pc2.df_reported_enough_but_write_failed_count)));

        let pc4_fill_phase_injected_damage = if arm.reclaim == ReclaimTiming::AfterEffectiveFloor {
            Damage { reclaim_on_first_new_floor_root: true, ..Damage::default() }
        } else {
            Damage { skip_hold: true, ..Damage::default() }
        };
        let (fill_phase_injected_pc4, ..) = run_fill_phase_injected_history(arm, pc_geometry, 1, FailurePosition::First, pc4_fill_phase_injected_damage);
        if fill_phase_injected_pc4.unsafe_candidate_reuse_count_field >= 1 {
            pc4_fill_phase_injected_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc4-h5prime arm={} unsafe_candidate_reuse_count_field={}", arm.label(), fill_phase_injected_pc4.unsafe_candidate_reuse_count_field)));

        let (fill_phase_injected_pc5, ..) = run_fill_phase_injected_history(arm, pc5_geometry, 3, FailurePosition::Cover, Damage { zero_reserves: true, ..Damage::default() });
        if fill_phase_injected_pc5.wedged {
            pc5_fill_phase_injected_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc5-h5prime arm={} wedged={}", arm.label(), fill_phase_injected_pc5.wedged)));

        run_count += 16;
    }

    println!(
        "{}",
        emitter.emit_raw(&format!(
            "DONE runs={run_count} pc1_hit_any={pc1_hit_any} pc2_hit_any={pc2_hit_any} pc4_hit_any={pc4_hit_any} pc5_hit_any={pc5_hit_any} pc1_root_write_failure_hit_any={pc1_root_write_failure_hit_any} pc2_root_write_failure_hit_any={pc2_root_write_failure_hit_any} pc4_root_write_failure_hit_any={pc4_root_write_failure_hit_any} pc5_root_write_failure_hit_any={pc5_root_write_failure_hit_any} pc1_rollback_hit_any={pc1_rollback_hit_any} pc2_rollback_hit_any={pc2_rollback_hit_any} pc4_rollback_hit_any={pc4_rollback_hit_any} pc5_rollback_hit_any={pc5_rollback_hit_any} pc1_crash_mid_push_hit_any={pc1_crash_mid_push_hit_any} pc2_crash_mid_push_hit_any={pc2_crash_mid_push_hit_any} pc4_crash_mid_push_hit_any={pc4_crash_mid_push_hit_any} pc5_crash_mid_push_hit_any={pc5_crash_mid_push_hit_any} pc1_fill_phase_injected_hit_any={pc1_fill_phase_injected_hit_any} pc2_fill_phase_injected_hit_any={pc2_fill_phase_injected_hit_any} pc4_fill_phase_injected_hit_any={pc4_fill_phase_injected_hit_any} pc5_fill_phase_injected_hit_any={pc5_fill_phase_injected_hit_any} unbounded_budget_diverged_count={unbounded_budget_diverged_count}"
        ))
    );
    println!("{}", emitter.finish());
}

