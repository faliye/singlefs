//! 账本条目与环条目、池的状态与分配回收、谓词与影子账、四条臂、放置策略与几何格。

use e7_index_bench::usize_from_u64;
use super::*;

// ---------------------------------------------------------------------
// 记账账本：一条记录 = (span, 分配代, 已释放, 释放代[, 释放代2 仅 G12])
// ---------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub(super) struct LedgerEntry {
    pub(super) span_in_slots: u64,
    pub(super) allocation_generation: u64,
    pub(super) released: bool,
    pub(super) release_generation: u64,
}

impl LedgerEntry {
    pub(super) fn allocated(span_in_slots: u64, allocation_generation: u64) -> LedgerEntry {
        LedgerEntry { span_in_slots, allocation_generation, released: false, release_generation: 0 }
    }
}

// ---------------------------------------------------------------------
// 池级状态：根环、实例表、F —— 与臂无关（登记第一节各定义均只依赖这三样）
// ---------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(super) struct RingEntry {
    pub(super) txg: u64,
    pub(super) instance: u64,
    rollback_floor: u64,
    pub(super) is_nonempty: bool,
    /// 这个根发布时实例表的长度（供回退取「R_old 指着的那一版实例表」用：truncate 到这个长度）。
    pub(super) instance_table_length_at_publish: usize,
}

#[derive(Clone, Debug)]
pub(super) struct PoolState {
    pub(super) slots_per_region: u64,
    ring: Vec<Option<RingEntry>>,
    /// 实例表：行 (实例, valid_up_to)；无该实例的行 = 该实例的一切根都有效。
    pub(super) instance_table_rows: Vec<(u64, u64)>,
    pub(super) current_txg: u64,
    pub(super) current_instance: u64,
    next_instance_identifier: u64,
    pub(super) current_rollback_floor: u64,
}

impl PoolState {
    pub(super) fn new(slots_per_region: u64) -> PoolState {
        let capacity = usize_from_u64(ROOT_RING_REGIONS * slots_per_region);
        let genesis = RingEntry { txg: 0, instance: 0, rollback_floor: 0, is_nonempty: false, instance_table_length_at_publish: 0 };
        let mut ring = vec![None; capacity];
        ring[usize_from_u64(root_ring_flat_index(0, slots_per_region))] = Some(genesis);
        PoolState { slots_per_region, ring, instance_table_rows: Vec::new(), current_txg: 0, current_instance: 0, next_instance_identifier: 1, current_rollback_floor: 0 }
    }

    pub(super) fn entry_at(&self, txg: u64) -> Option<&RingEntry> {
        self.ring[usize_from_u64(root_ring_flat_index(txg, self.slots_per_region))].as_ref().filter(|entry| entry.txg == txg)
    }

    /// 环里全部自证合法（此刻仍占着该槽）的根，按 txg 升序。
    pub(super) fn live_roots(&self) -> Vec<RingEntry> {
        let mut roots: Vec<RingEntry> = self.ring.iter().flatten().cloned().collect();
        roots.sort_by_key(|entry| entry.txg);
        roots
    }

    /// 一条根按最新持久根指着的实例表判是否被抛弃：有它实例的行 (i,T) 且 txg > T（`mount.rs` 185-192）。
    fn is_abandoned(&self, instance: u64, txg: u64) -> bool {
        self.instance_table_rows.iter().any(|(row_instance, valid_up_to)| *row_instance == instance && txg > *valid_up_to)
    }

    pub(super) fn is_valid(&self, instance: u64, txg: u64) -> bool {
        !self.is_abandoned(instance, txg)
    }

    /// 环里最旧有效根的 txg（D16 已定项 1「环里最旧有效根」；写失败的槽按旧内容算，在飞发布不算，
    /// 因为本模型只在持久之后才把根写进 ring）。
    pub(super) fn oldest_valid_root_txg(&self) -> u64 {
        self.live_roots().iter().filter(|entry| self.is_valid(entry.instance, entry.txg)).map(|entry| entry.txg).min().expect("环里至少有 mkfs 的根")
    }

    /// F_生效 = 各盘上根所带 F 最大值的最小值（一块盘没有根就不算它）。
    pub(super) fn effective_floor(&self) -> u64 {
        let mut per_device: Vec<u64> = Vec::new();
        for device in [Device::Disk0, Device::Disk1] {
            let maximum_floor = self.live_roots().iter().filter(|entry| root_ring_device(entry.txg) == device).map(|entry| entry.rollback_floor).max();
            if let Some(value) = maximum_floor {
                per_device.push(value);
            }
        }
        per_device.into_iter().min().unwrap_or(0)
    }

    /// 甲-T1 与基线共用的门槛：max(F_生效, 环里最旧有效根)（D16 已定项 1「可再分配」）。
    /// 单一来源——回收步与记账公式都调这一处，别在调用点各写一份（U1 的变异锚点）。
    pub(super) fn reclaim_threshold(&self) -> u64 {
        self.oldest_valid_root_txg().max(self.effective_floor())
    }

    /// 回退候选集：有效 ∧ txg ≥ F_生效。
    pub(super) fn candidate_roots(&self) -> Vec<RingEntry> {
        let floor = self.effective_floor();
        self.live_roots().into_iter().filter(|entry| self.is_valid(entry.instance, entry.txg) && entry.txg >= floor).collect()
    }

    /// 被抛弃根：按最新持久根指着的实例表判不有效的根（不含回退重建那一刻额外加的那一条，另用
    /// `abandoned_roots_including_rollback_extension`）。
    pub(super) fn abandoned_roots(&self) -> Vec<RingEntry> {
        self.live_roots().into_iter().filter(|entry| self.is_abandoned(entry.instance, entry.txg)).collect()
    }

    /// 回退重建那一刻的「被抛弃」：按最新持久根判被抛弃 ∪ (txg,实例) 大于 R_old 的根。
    pub(super) fn abandoned_roots_including_rollback_extension(&self, rollback_target_txg: u64, rollback_target_instance: u64) -> Vec<RingEntry> {
        let mut set: BTreeMap<u64, RingEntry> = BTreeMap::new();
        for entry in self.abandoned_roots() {
            set.insert(entry.txg, entry);
        }
        for entry in self.live_roots() {
            if (entry.txg, entry.instance) > (rollback_target_txg, rollback_target_instance) {
                set.insert(entry.txg, entry);
            }
        }
        set.into_values().collect()
    }

    /// 「blocking」txg 集合，供 G12 的区间谓词用：候选集 ∪ 被抛弃根的 txg（登记第五节主谓词 G12）。
    pub(super) fn interval_blocking_generations(&self) -> BTreeSet<u64> {
        let mut set: BTreeSet<u64> = BTreeSet::new();
        for entry in self.candidate_roots() {
            set.insert(entry.txg);
        }
        for entry in self.abandoned_roots() {
            set.insert(entry.txg);
        }
        set
    }

    /// 写一条根：把它放进环里对应的坐标（region/slot 由 txg 决定），推进 current_*。写失败时不调用本方法
    /// （ring 保留旧内容），只推进 current_txg 之外的调用方状态。
    pub(super) fn commit_root(&mut self, txg: u64, instance: u64, rollback_floor: u64, is_nonempty: bool) {
        let index = usize_from_u64(root_ring_flat_index(txg, self.slots_per_region));
        let instance_table_length_at_publish = self.instance_table_rows.len();
        self.ring[index] = Some(RingEntry { txg, instance, rollback_floor, is_nonempty, instance_table_length_at_publish });
        self.current_txg = txg;
        self.current_instance = instance;
        self.current_rollback_floor = rollback_floor;
    }

    pub(super) fn take_next_instance(&mut self) -> u64 {
        let instance = self.next_instance_identifier;
        self.next_instance_identifier += 1;
        instance
    }
}

// ---------------------------------------------------------------------
// 账户查询：I-3.1 的候选集去重求和（登记 walk.rs 46-72 的去重键 (设备, 起点槽, 跨度)）
// ---------------------------------------------------------------------

// ---------------------------------------------------------------------
// 主谓词
// ---------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Predicate {
    /// 甲-T1：已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)。
    ThresholdFloor,
    /// G12：已释放 ∧ 环里没有根 r：txg(r) ∈ [分配代, 释放代) ∧ (r 被抛弃 ∨ txg(r) ≥ F_生效)。
    IntervalBlocking,
}

/// 影子账形态。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ShadowLedger {
    /// 保守读法：环里每条被抛弃根的账里未释放的槽全部隔离，含当前账 / 候选账也引用的。
    Conservative,
    /// G28：只隔离「被抛弃根引用 − 候选根引用」的差集。
    Narrow,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Arm {
    ThresholdConservative,
    ThresholdNarrow,
    IntervalConservative,
    IntervalNarrow,
    /// 真实基线「今」：分配记录同甲-T1（20 字节一个代），回收只在挂载/切换/回退重建/抬 F 时发生，
    /// 记账三项直接反映分配器物理状态；影子账取豁免-候选集形态（`mount.rs` 201-258）。
    Baseline,
}

impl Arm {
    pub(super) fn tag(self) -> &'static str {
        match self {
            Arm::ThresholdConservative => "threshold_conservative",
            Arm::ThresholdNarrow => "threshold_narrow",
            Arm::IntervalConservative => "interval_conservative",
            Arm::IntervalNarrow => "interval_narrow",
            Arm::Baseline => "baseline_today",
        }
    }

    pub(super) fn predicate(self) -> Option<Predicate> {
        match self {
            Arm::ThresholdConservative | Arm::ThresholdNarrow | Arm::Baseline => Some(Predicate::ThresholdFloor),
            Arm::IntervalConservative | Arm::IntervalNarrow => Some(Predicate::IntervalBlocking),
        }
    }

    pub(super) fn record_width_bytes(self) -> u64 {
        match self {
            Arm::ThresholdConservative | Arm::ThresholdNarrow | Arm::Baseline => RECORD_WIDTH_BYTES_SINGLE_GENERATION,
            Arm::IntervalConservative | Arm::IntervalNarrow => RECORD_WIDTH_BYTES_DUAL_GENERATION,
        }
    }

    pub(super) fn records_per_node(self) -> u64 {
        match self {
            Arm::ThresholdConservative | Arm::ThresholdNarrow | Arm::Baseline => RECORDS_PER_NODE_SINGLE_GENERATION,
            Arm::IntervalConservative | Arm::IntervalNarrow => RECORDS_PER_NODE_DUAL_GENERATION,
        }
    }

    pub(super) fn is_measured(self) -> bool {
        !matches!(self, Arm::Baseline)
    }
}

// ---------------------------------------------------------------------
// 落点政策
// ---------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum PlacementPolicy {
    /// 每个单元取单元区内最低的、跨度内每槽都可发、跨度 2 时起点偶数的位置；没有段（M0.17）。
    Low,
    /// 用户数据取最低偶数对；提交内生块从开放段 bump，段 = 最低的、64 槽对齐、段内无占用的段（M0.16）。
    Impl,
}

// ---------------------------------------------------------------------
// 一个几何格
// ---------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct GeometryCell {
    pub(super) slots_per_region: u64,
    pub(super) placement: PlacementPolicy,
    pub(super) workload_period: u64, // ρ = 1/period；period=1 时 ρ=1，period=4 时 ρ=1/4
}

impl GeometryCell {
    pub(super) fn ring_capacity(self) -> u64 {
        ROOT_RING_REGIONS * self.slots_per_region
    }
}

pub(super) const BASE_GEOMETRY: GeometryCell = GeometryCell { slots_per_region: 8, placement: PlacementPolicy::Impl, workload_period: 1 };

pub(super) fn all_geometry_cells() -> Vec<GeometryCell> {
    let mut cells = Vec::new();
    for slots_per_region in [4u64, 8, 16] {
        for placement in [PlacementPolicy::Impl, PlacementPolicy::Low] {
            for workload_period in [1u64, 4] {
                cells.push(GeometryCell { slots_per_region, placement, workload_period });
            }
        }
    }
    cells
}

#[cfg(test)]
#[path = "ledger_pool_state_and_arms_tests.rs"]
mod tests;
