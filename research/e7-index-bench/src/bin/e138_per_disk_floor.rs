//! E138：按盘生效的动态回退下界与整环 + 推空 —— D16（发布语义） 已定项 1 第二轮判决之后的重跑。
//!
//! 预注册 `research/prompts/e138-preregistration.md`。装置从 E135（动态回退下界） 的源码拷出来改：
//! ① 准入失败时先把正在攒的窗口发布出去，再按臂处置（D3（空间分配） 已定项 9 允许先做有界的内部工作）；
//! ② 两种 df 都报，判据只用扣掉保留池与推空最坏残留的那一种；
//! ③ 新增 activation_faults：把根槽写失败与丢盘注入到抬 F 到生效那段区间里（E135 从没注入过）。

use e7_index_bench::Emitter;
use std::collections::{BTreeSet, HashSet, VecDeque};
use std::rc::Rc;

/// R = 3（D22（单元原子性怎么合成） 已定项 2）。
const REGION_COUNT: u64 = 3;
const DISK_COUNT: usize = 2;
/// 臂 C、臂 D 的 K_min：C222 按轮转算术给的 3。
const TXG_ARITHMETIC_GENERATIONS: u64 = 3;
/// 用户定案（2026-09-11）：盘紧时最少保留 4 个持久根。
const MINIMUM_RETAINED_ROOTS: usize = 4;
/// 对照臂 G_count：带新 F 的第一个根之后再数几次持久发布。
const COUNT_ACTIVATION_PUBLICATIONS: u64 = 3;
/// 对照臂 G_txgcap：抬 F 的上限按 txg 数几代。
const TXG_UPPER_LIMIT_GENERATIONS: u64 = 4;
const USER_BLOCKS_PER_OBJECT: usize = 8;
const USER_BLOCKS: u64 = USER_BLOCKS_PER_OBJECT as u64;
const METADATA_BLOCKS_PER_PUBLICATION: u64 = 5;
const BLOCKS_PER_PUBLICATION: u64 = USER_BLOCKS + METADATA_BLOCKS_PER_PUBLICATION;
/// 臂 C、臂 D 沿用 E135（动态回退下界） 的保留池。
const CONTROL_ARM_RESERVE_BLOCKS: u64 = BLOCKS_PER_PUBLICATION;
/// 预注册的 B_G：发布正在攒的窗口 + 至多 6 次空发布。
const FLOOR_PER_DISK_PUBLICATION_BOUND: u64 = 7;
const CAPACITY_BLOCKS: usize = 4000;
const STEADY_FILL: f64 = 0.70;
const WINDOWS_PER_WORLD: u64 = 400;
const SEED_COUNT: u64 = 24;
const ACTIVATION_FAULT_SEED_COUNT: u64 = 8;
const RESERVE_SWEEP_SEED_COUNT: u64 = 8;
const SLOTS_PER_REGION_SWEEP: [u64; 3] = [1, 4, 16];
const ACTIVATION_FAULT_SLOTS_SWEEP: [u64; 2] = [4, 16];
const EMPTY_PUBLICATION_COST_SWEEP: [u64; 2] = [1, 5];
/// 第 k 次根槽写失败，k ∈ 0..5。
const ACTIVATION_FAULT_ATTEMPTS: u64 = 5;
/// 第 j 个窗口分配完、根槽写之前崩溃，j ∈ 1..=8。
const ACTIVATION_FAULT_CRASH_WINDOWS: u64 = 8;
const RESERVE_SWEEP_BELOW: u64 = 6;
const RESERVE_SWEEP_ABOVE: u64 = 8;
const RESERVE_SWEEP_SLOTS_PER_REGION: u64 = 16;
/// 快照留多少份：要大于最大的根环深度 48。
const SNAPSHOTS_KEPT: usize = 64;
const BLOCK_BYTES: u64 = 16384;
const DETAIL_LINES_MAXIMUM: u64 = 20;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    RingPersisted,
    ProtectedGenerationsAtMinimum,
    FloorTxg,
    FloorPerDisk,
    FloorPerDiskCountActivation,
    FloorPerDiskTxgUpperLimit,
}

const ARMS: [Arm; 6] = [
    Arm::RingPersisted,
    Arm::ProtectedGenerationsAtMinimum,
    Arm::FloorTxg,
    Arm::FloorPerDisk,
    Arm::FloorPerDiskCountActivation,
    Arm::FloorPerDiskTxgUpperLimit,
];

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::RingPersisted => "A_ring_persisted",
            Arm::ProtectedGenerationsAtMinimum => "C_k_runtime",
            Arm::FloorTxg => "D_floor_txg",
            Arm::FloorPerDisk => "G_floor_per_disk",
            Arm::FloorPerDiskCountActivation => "G_count_activation",
            Arm::FloorPerDiskTxgUpperLimit => "G_txg_cap",
        }
    }
    fn uses_floor(self) -> bool {
        match self {
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum => false,
            Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskCountActivation | Arm::FloorPerDiskTxgUpperLimit => true,
        }
    }
    fn is_candidate(self) -> bool {
        match self {
            Arm::RingPersisted | Arm::FloorPerDisk => true,
            Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDiskCountActivation | Arm::FloorPerDiskTxgUpperLimit => false,
        }
    }
    /// 预注册的界 B：一次准入最多用几次发布（含发布正在攒的窗口）。只对候选臂判。
    fn publication_bound(self, ring_depth: u64) -> u64 {
        match self {
            Arm::RingPersisted => ring_depth,
            Arm::FloorPerDisk | Arm::FloorPerDiskCountActivation | Arm::FloorPerDiskTxgUpperLimit => FLOOR_PER_DISK_PUBLICATION_BOUND,
            Arm::ProtectedGenerationsAtMinimum => 1,
            Arm::FloorTxg => 1 + TXG_ARITHMETIC_GENERATIONS + 2,
        }
    }
    /// 预注册：保留池 = 上一个窗口的元数据 + 这次处置要拿的块。
    fn reserve_blocks(self, ring_depth: u64, empty_publication_cost: u64) -> u64 {
        match self {
            Arm::RingPersisted => 2 * METADATA_BLOCKS_PER_PUBLICATION + (ring_depth - 1) * empty_publication_cost,
            Arm::FloorPerDisk | Arm::FloorPerDiskCountActivation | Arm::FloorPerDiskTxgUpperLimit => {
                2 * METADATA_BLOCKS_PER_PUBLICATION + (FLOOR_PER_DISK_PUBLICATION_BOUND - 1) * empty_publication_cost
            }
            Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg => CONTROL_ARM_RESERVE_BLOCKS,
        }
    }
    /// 预注册：一次完整处置之后还扣着的最坏块数。
    fn push_residue_blocks(self, ring_depth: u64, empty_publication_cost: u64) -> u64 {
        match self {
            Arm::RingPersisted => METADATA_BLOCKS_PER_PUBLICATION + (ring_depth - 2) * empty_publication_cost,
            Arm::FloorPerDisk | Arm::FloorPerDiskCountActivation | Arm::FloorPerDiskTxgUpperLimit => {
                METADATA_BLOCKS_PER_PUBLICATION + (FLOOR_PER_DISK_PUBLICATION_BOUND - 2) * empty_publication_cost
            }
            Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg => 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum World {
    Steady,
    TornNewest,
    DiskLoss,
    SlotWriteFail,
    AdminRollback,
    RollbackFirstPublish,
}

const WORLDS: [World; 6] = [World::Steady, World::TornNewest, World::DiskLoss, World::SlotWriteFail, World::AdminRollback, World::RollbackFirstPublish];

impl World {
    fn name(self) -> &'static str {
        match self {
            World::Steady => "steady",
            World::TornNewest => "torn_newest",
            World::DiskLoss => "disk_loss",
            World::SlotWriteFail => "slot_write_fail",
            World::AdminRollback => "admin_rollback",
            World::RollbackFirstPublish => "rollback_first_publish",
        }
    }
    /// E135 跑前补登第 1 条：这四个世界给 F 臂在故障之前安排一次强制盘紧。
    fn forces_pressure_before_event(self) -> bool {
        match self {
            World::TornNewest | World::DiskLoss | World::SlotWriteFail | World::AdminRollback => true,
            World::Steady | World::RollbackFirstPublish => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BlockState {
    Unused,
    Live { allocated_at: u64 },
    Freed { allocated_at: u64, freed_at: u64 },
}

#[derive(Clone)]
struct PoolState {
    blocks: Vec<BlockState>,
    objects: Vec<[usize; USER_BLOCKS_PER_OBJECT]>,
    metadata: Vec<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct RootRecord {
    txg: u64,
    instance: u64,
    floor: u64,
    is_self_verified: bool,
}

#[derive(Clone, Copy)]
struct PendingFloor {
    value: u64,
    carrier_txg: Option<u64>,
    durable_publications_since_carrier: u64,
}

#[derive(Clone, Copy, Debug)]
struct Metrics {
    fake_candidates: u64,
    unrecoverable: u64,
    retention_violations: u64,
    checks: u64,
    minimum_candidates: u64,
    candidate_sum: u64,
    nonempty_candidate_sum: u64,
    stalls: u64,
    forced_publications: u64,
    user_windows: u64,
    admissions_under_pressure: u64,
    maximum_publications_per_admission: u64,
    false_enospc_raw: u64,
    false_enospc_guaranteed: u64,
    true_enospc: u64,
    write_after_delete_failed: u64,
    halted_before_counting: u64,
}

impl Metrics {
    fn fresh() -> Self {
        Metrics {
            fake_candidates: 0,
            unrecoverable: 0,
            retention_violations: 0,
            checks: 0,
            minimum_candidates: u64::MAX,
            candidate_sum: 0,
            nonempty_candidate_sum: 0,
            stalls: 0,
            forced_publications: 0,
            user_windows: 0,
            admissions_under_pressure: 0,
            maximum_publications_per_admission: 0,
            false_enospc_raw: 0,
            false_enospc_guaranteed: 0,
            true_enospc: 0,
            write_after_delete_failed: 0,
            halted_before_counting: 0,
        }
    }
    fn add(&mut self, other: &Metrics) {
        self.fake_candidates += other.fake_candidates;
        self.unrecoverable += other.unrecoverable;
        self.retention_violations += other.retention_violations;
        self.checks += other.checks;
        self.minimum_candidates = self.minimum_candidates.min(other.minimum_candidates);
        self.candidate_sum += other.candidate_sum;
        self.nonempty_candidate_sum += other.nonempty_candidate_sum;
        self.stalls += other.stalls;
        self.forced_publications += other.forced_publications;
        self.user_windows += other.user_windows;
        self.admissions_under_pressure += other.admissions_under_pressure;
        self.maximum_publications_per_admission = self.maximum_publications_per_admission.max(other.maximum_publications_per_admission);
        self.false_enospc_raw += other.false_enospc_raw;
        self.false_enospc_guaranteed += other.false_enospc_guaranteed;
        self.true_enospc += other.true_enospc;
        self.write_after_delete_failed += other.write_after_delete_failed;
        self.halted_before_counting += other.halted_before_counting;
    }
    fn minimum_candidates_or_zero(&self) -> u64 {
        if self.minimum_candidates == u64::MAX { 0 } else { self.minimum_candidates }
    }
    fn average_candidates_in_hundredths(&self) -> u64 {
        if self.checks == 0 { 0 } else { self.candidate_sum * 100 / self.checks }
    }
    fn average_nonempty_candidates_in_hundredths(&self) -> u64 {
        if self.checks == 0 { 0 } else { self.nonempty_candidate_sum * 100 / self.checks }
    }
    fn forced_publications_per_window_in_hundredths(&self) -> u64 {
        if self.user_windows == 0 { 0 } else { self.forced_publications * 100 / self.user_windows }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WindowOutcome {
    Created,
    AllocatedWithoutPublish,
    NoSpace,
    Halted,
}

#[derive(Clone)]
struct Simulation {
    arm: Arm,
    slots_per_region: u64,
    ring_depth: u64,
    empty_publication_cost: u64,
    reserve_blocks: u64,
    push_residue_blocks: u64,
    random_state: u64,
    pool: PoolState,
    damaged_txgs: HashSet<u64>,
    ring: Vec<Option<RootRecord>>,
    disk_lost: [bool; DISK_COUNT],
    instance: u64,
    instance_rows: Vec<(u64, u64)>,
    newest_durable_txg: u64,
    maximum_published_txg: u64,
    next_txg: u64,
    floor_carried: u64,
    floor_active: u64,
    pending_floor: Option<PendingFloor>,
    snapshots: VecDeque<((u64, u64), Rc<PoolState>)>,
    keep_snapshots: bool,
    tear_next_publication: bool,
    fail_next_slot_write: bool,
    /// activation_faults 用：第几次根槽写要失败（按根槽写的绝对序号，重发也算一次）。
    planned_root_write_failures: BTreeSet<u64>,
    root_write_attempts: u64,
    current_window_changes_user_state: bool,
    /// 非空发布的 txg：空发布与推空的根不改用户可见状态（第二轮反推腿指出「可退代数」该数这个）。
    nonempty_txgs: HashSet<u64>,
    admin_rollback_happened: bool,
    halted: bool,
    metrics: Metrics,
    /// 管理员回退之前盘上那一份分配状态，被抛弃时间线的根引用的是它（E135 的 abandoned_view 原样）。
    abandoned_view: Option<(Rc<PoolState>, u64)>,
}

impl Simulation {
    fn new(arm: Arm, slots_per_region: u64, empty_publication_cost: u64, seed: u64, keep_snapshots: bool, reserve_override: Option<u64>) -> Self {
        let ring_depth = REGION_COUNT * slots_per_region;
        let seed_root = RootRecord { txg: 0, instance: 1, floor: 0, is_self_verified: true };
        Simulation {
            arm,
            slots_per_region,
            ring_depth,
            empty_publication_cost,
            reserve_blocks: reserve_override.unwrap_or_else(|| arm.reserve_blocks(ring_depth, empty_publication_cost)),
            push_residue_blocks: arm.push_residue_blocks(ring_depth, empty_publication_cost),
            random_state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
            pool: PoolState { blocks: vec![BlockState::Unused; CAPACITY_BLOCKS], objects: Vec::new(), metadata: Vec::new() },
            damaged_txgs: HashSet::new(),
            // mkfs 把第 0 代种进全部槽（D22（单元原子性怎么合成） 已定项 8）
            ring: vec![Some(seed_root); usize::try_from(ring_depth).expect("环深装得进 usize")],
            disk_lost: [false; DISK_COUNT],
            instance: 1,
            instance_rows: Vec::new(),
            newest_durable_txg: 0,
            maximum_published_txg: 0,
            next_txg: 1,
            floor_carried: 0,
            floor_active: 0,
            pending_floor: None,
            snapshots: VecDeque::new(),
            keep_snapshots,
            tear_next_publication: false,
            fail_next_slot_write: false,
            planned_root_write_failures: BTreeSet::new(),
            root_write_attempts: 0,
            current_window_changes_user_state: false,
            nonempty_txgs: HashSet::new(),
            admin_rollback_happened: false,
            halted: false,
            metrics: Metrics::fresh(),
            abandoned_view: None,
        }
    }

    fn next_random(&mut self) -> u64 {
        self.random_state ^= self.random_state >> 12;
        self.random_state ^= self.random_state << 25;
        self.random_state ^= self.random_state >> 27;
        self.random_state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// 区域 = txg mod R，区内槽 = ⌊txg / R⌋ mod S（区内公式是 first-txn-layout.md 的预想）。
    fn slot_index(&self, txg: u64) -> usize {
        let region = txg % REGION_COUNT;
        let slot_within_region = (txg / REGION_COUNT) % self.slots_per_region;
        usize::try_from(region * self.slots_per_region + slot_within_region).expect("槽号装得进 usize")
    }

    /// 区域 0、2 在盘 0，区域 1 在盘 1（first-txn-layout.md 的预想归属）。
    fn disk_of_slot(&self, slot: usize) -> usize {
        let region = slot as u64 / self.slots_per_region;
        if region == 1 { 1 } else { 0 }
    }

    fn slot_survives(&self, slot: usize) -> bool {
        !self.disk_lost[self.disk_of_slot(slot)]
    }

    fn surviving_disks(&self) -> Vec<usize> {
        (0..DISK_COUNT).filter(|disk| !self.disk_lost[*disk]).collect()
    }

    fn is_any_disk_lost(&self) -> bool {
        self.disk_lost.iter().any(|is_lost| *is_lost)
    }

    fn is_instance_valid(&self, root: &RootRecord) -> bool {
        match self.instance_rows.iter().filter(|(row_instance, _)| *row_instance == root.instance).map(|(_, limit)| *limit).min() {
            None => true,
            Some(limit) => root.txg <= limit,
        }
    }

    /// 幸存盘上还读得到、自证合法的根（不管实例表）。
    fn visible_roots(&self) -> Vec<RootRecord> {
        (0..self.ring.len())
            .filter(|slot| self.slot_survives(*slot))
            .filter_map(|slot| self.ring[slot])
            .filter(|root| root.is_self_verified)
            .collect()
    }

    fn valid_roots_with_disk(&self) -> Vec<(usize, RootRecord)> {
        (0..self.ring.len())
            .filter(|slot| self.slot_survives(*slot))
            .filter_map(|slot| self.ring[slot].map(|root| (self.disk_of_slot(slot), root)))
            .filter(|(_, root)| root.is_self_verified && self.is_instance_valid(root))
            .collect()
    }

    fn valid_roots(&self) -> Vec<RootRecord> {
        self.valid_roots_with_disk().into_iter().map(|(_, root)| root).collect()
    }

    fn oldest_persisted_valid_txg(&self) -> u64 {
        self.valid_roots().iter().map(|root| root.txg).min().unwrap_or(0)
    }

    fn newest_valid_root(&self) -> Option<RootRecord> {
        self.valid_roots().into_iter().max_by_key(|root| (root.instance, root.txg))
    }

    fn newest_valid_txg_on_disk(&self, disk: usize) -> Option<u64> {
        self.valid_roots_with_disk().iter().filter(|(root_disk, _)| *root_disk == disk).map(|(_, root)| root.txg).max()
    }

    /// 用户定案的上限：min(每块幸存盘上最新的持久有效根, 第 4 新的持久有效根)；有效根不足 4 个时取最旧的那个。
    fn per_disk_retention_upper_limit(&self) -> u64 {
        let distinct_newest_first: Vec<u64> = self.valid_roots().iter().map(|root| root.txg).collect::<BTreeSet<u64>>().into_iter().rev().collect();
        let fourth_newest = if distinct_newest_first.len() >= MINIMUM_RETAINED_ROOTS {
            distinct_newest_first[MINIMUM_RETAINED_ROOTS - 1]
        } else {
            distinct_newest_first.last().copied().unwrap_or(0)
        };
        let newest_on_every_disk = self.surviving_disks().iter().map(|disk| self.newest_valid_txg_on_disk(*disk).unwrap_or(0)).min().unwrap_or(0);
        fourth_newest.min(newest_on_every_disk)
    }

    /// 这条臂抬 F 的上限。
    fn floor_upper_limit(&self) -> u64 {
        match self.arm {
            Arm::FloorTxg => (self.newest_durable_txg + 1).saturating_sub(TXG_ARITHMETIC_GENERATIONS),
            Arm::FloorPerDiskTxgUpperLimit => (self.newest_durable_txg + 1).saturating_sub(TXG_UPPER_LIMIT_GENERATIONS),
            Arm::FloorPerDisk | Arm::FloorPerDiskCountActivation => self.per_disk_retention_upper_limit(),
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum => 0,
        }
    }

    /// 候选集的下界：幸存盘上全部自证合法的根（含被抛弃时间线的根）所带 F 的最大值。
    fn candidacy_floor(&self) -> u64 {
        if self.arm.uses_floor() {
            self.visible_roots().iter().map(|root| root.floor).max().unwrap_or(0)
        } else {
            0
        }
    }

    /// 可再分配的谓词：已释放 ∧ 释放代 ≤ 这个界。
    fn reuse_bound(&self) -> u64 {
        match self.arm {
            Arm::RingPersisted => self.oldest_persisted_valid_txg(),
            Arm::ProtectedGenerationsAtMinimum => (self.newest_durable_txg + 1).saturating_sub(TXG_ARITHMETIC_GENERATIONS),
            Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskCountActivation | Arm::FloorPerDiskTxgUpperLimit => {
                self.floor_active.max(self.oldest_persisted_valid_txg())
            }
        }
    }

    fn reusable_count(&self) -> u64 {
        let bound = self.reuse_bound();
        self.pool.blocks.iter().filter(|state| match state {
            BlockState::Unused => true,
            BlockState::Live { .. } => false,
            BlockState::Freed { freed_at, .. } => *freed_at <= bound,
        }).count() as u64
    }

    /// 已释放而还不可再分配的块数。
    fn pinned_count(&self) -> u64 {
        let bound = self.reuse_bound();
        self.pool.blocks.iter().filter(|state| match state {
            BlockState::Unused | BlockState::Live { .. } => false,
            BlockState::Freed { freed_at, .. } => *freed_at > bound,
        }).count() as u64
    }

    /// E135 的 df：没用过的 + 已释放的，扣住的块与保留池都算空闲。
    fn df_raw(&self) -> u64 {
        self.pool.blocks.iter().filter(|state| match state {
            BlockState::Unused | BlockState::Freed { .. } => true,
            BlockState::Live { .. } => false,
        }).count() as u64
    }

    /// 预注册的 df：扣掉这条臂的保留池与推空最坏残留。
    fn df_guaranteed(&self) -> u64 {
        self.df_raw().saturating_sub(self.reserve_blocks + self.push_residue_blocks)
    }

    /// 普通分配看得见的：扣掉保留池。
    fn user_allocatable(&self) -> u64 {
        self.reusable_count().saturating_sub(self.reserve_blocks)
    }

    /// 取 count 块：先取可再分配的已释放块，再取没用过的。复用一块就把它上一段区间记成损坏。
    fn allocate(&mut self, count: u64, txg: u64) -> Option<Vec<usize>> {
        let bound = self.reuse_bound();
        let wanted = usize::try_from(count).expect("块数装得进 usize");
        let mut chosen: Vec<usize> = self.pool.blocks.iter().enumerate()
            .filter(|(_, state)| matches!(state, BlockState::Freed { freed_at, .. } if *freed_at <= bound))
            .map(|(index, _)| index).take(wanted).collect();
        if chosen.len() < wanted {
            let missing = wanted - chosen.len();
            chosen.extend(self.pool.blocks.iter().enumerate()
                .filter(|(_, state)| matches!(state, BlockState::Unused))
                .map(|(index, _)| index).take(missing));
        }
        if chosen.len() < wanted {
            return None;
        }
        let oldest_visible_txg = self.ring.iter().flatten().map(|root| root.txg).min().unwrap_or(0);
        for &index in &chosen {
            if let BlockState::Freed { allocated_at, freed_at } = self.pool.blocks[index] {
                for damaged_txg in allocated_at.max(oldest_visible_txg)..freed_at {
                    self.damaged_txgs.insert(damaged_txg);
                }
            }
            if let Some((abandoned_pool, abandoned_newest_txg)) = &self.abandoned_view {
                let abandoned_range = match abandoned_pool.blocks[index] {
                    BlockState::Unused => None,
                    BlockState::Live { allocated_at } => Some((allocated_at, abandoned_newest_txg + 1)),
                    BlockState::Freed { allocated_at, freed_at } => Some((allocated_at, freed_at)),
                };
                if let Some((start, end)) = abandoned_range {
                    for damaged_txg in start.max(oldest_visible_txg)..end {
                        self.damaged_txgs.insert(damaged_txg);
                    }
                }
            }
            self.pool.blocks[index] = BlockState::Live { allocated_at: txg };
        }
        Some(chosen)
    }

    fn free_blocks(&mut self, blocks: &[usize], txg: u64) {
        for &index in blocks {
            if let BlockState::Live { allocated_at } = self.pool.blocks[index] {
                self.pool.blocks[index] = BlockState::Freed { allocated_at, freed_at: txg };
            }
        }
    }

    /// (假候选数, 候选数, 其中非空发布的根数)
    fn count_fake_candidates(&self) -> (u64, u64, u64) {
        let floor = self.candidacy_floor();
        let candidates: Vec<RootRecord> = self.valid_roots().into_iter().filter(|root| root.txg >= floor).collect();
        let fake = candidates.iter().filter(|root| self.damaged_txgs.contains(&root.txg)).count() as u64;
        let nonempty = candidates.iter().filter(|root| self.nonempty_txgs.contains(&root.txg)).count() as u64;
        (fake, candidates.len() as u64, nonempty)
    }

    /// 用户定案的最少保留：没丢盘时 F_候选 ≤ 上限；丢盘之后查幸存盘上最新的有效根在候选集里；管理员回退之后不查。
    fn is_retention_violated(&self) -> bool {
        if !self.arm.uses_floor() || self.admin_rollback_happened {
            return false;
        }
        let floor = self.candidacy_floor();
        if self.is_any_disk_lost() {
            self.surviving_disks().iter().any(|disk| self.newest_valid_txg_on_disk(*disk).is_some_and(|newest| newest < floor))
        } else {
            floor > self.per_disk_retention_upper_limit()
        }
    }

    fn record_check(&mut self) {
        let (fake, candidates, nonempty) = self.count_fake_candidates();
        self.metrics.fake_candidates += fake;
        self.metrics.checks += 1;
        self.metrics.minimum_candidates = self.metrics.minimum_candidates.min(candidates);
        self.metrics.candidate_sum += candidates;
        self.metrics.nonempty_candidate_sum += nonempty;
        if self.is_retention_violated() {
            self.metrics.retention_violations += 1;
        }
    }

    fn prune_damage(&mut self) {
        let oldest_visible_txg = self.ring.iter().flatten().map(|root| root.txg).min().unwrap_or(0);
        self.damaged_txgs.retain(|txg| *txg >= oldest_visible_txg);
    }

    fn prefill(&mut self, fill: f64) {
        let target_live = (CAPACITY_BLOCKS as f64 * fill) as usize;
        let mut next_block = 0usize;
        while next_block + USER_BLOCKS_PER_OBJECT <= target_live {
            let mut object = [0usize; USER_BLOCKS_PER_OBJECT];
            for slot in object.iter_mut() {
                *slot = next_block;
                self.pool.blocks[next_block] = BlockState::Live { allocated_at: 0 };
                next_block += 1;
            }
            self.pool.objects.push(object);
        }
        for _ in 0..METADATA_BLOCKS_PER_PUBLICATION {
            self.pool.blocks[next_block] = BlockState::Live { allocated_at: 0 };
            self.pool.metadata.push(next_block);
            next_block += 1;
        }
        self.save_snapshot();
    }

    fn save_snapshot(&mut self) {
        if !self.keep_snapshots {
            return;
        }
        self.snapshots.push_back(((self.instance, self.newest_durable_txg), Rc::new(self.pool.clone())));
        while self.snapshots.len() > SNAPSHOTS_KEPT {
            self.snapshots.pop_front();
        }
    }

    fn restore_snapshot(&mut self, instance: u64, txg: u64) -> bool {
        match self.snapshots.iter().rev().find(|(key, _)| *key == (instance, txg)) {
            Some((_, state)) => {
                self.pool = PoolState::clone(state);
                true
            }
            None => false,
        }
    }

    fn relabel_generation(&mut self, from: u64, to: u64) {
        for state in self.pool.blocks.iter_mut() {
            *state = match *state {
                BlockState::Unused => BlockState::Unused,
                BlockState::Live { allocated_at } => BlockState::Live { allocated_at: if allocated_at == from { to } else { allocated_at } },
                BlockState::Freed { allocated_at, freed_at } => BlockState::Freed {
                    allocated_at: if allocated_at == from { to } else { allocated_at },
                    freed_at: if freed_at == from { to } else { freed_at },
                },
            };
        }
    }

    /// 每块幸存盘上都至少有一个持久、自证合法、有效、所带 F ≥ value 的根。
    fn floor_written_on_every_surviving_disk(&self, value: u64) -> bool {
        let roots = self.valid_roots_with_disk();
        self.surviving_disks().iter().all(|disk| roots.iter().any(|(root_disk, root)| root_disk == disk && root.floor >= value))
    }

    /// 发布 txg 为 t 的根。根槽写失败时 txg 推进一格再发（D23（journal 的角色与格式） 已定项 14 的索引行），写失败的槽留着旧内容。
    fn publish(&mut self, txg: u64) {
        let slot = self.slot_index(txg);
        let attempt = self.root_write_attempts;
        self.root_write_attempts += 1;
        let is_planned_failure = self.planned_root_write_failures.remove(&attempt);
        if self.fail_next_slot_write || is_planned_failure {
            self.fail_next_slot_write = false;
            self.relabel_generation(txg, txg + 1);
            self.next_txg = txg + 1;
            self.publish(txg + 1);
            return;
        }
        if self.tear_next_publication {
            self.tear_next_publication = false;
            self.ring[slot] = Some(RootRecord { txg, instance: self.instance, floor: self.floor_carried, is_self_verified: false });
            self.crash_and_recover();
            return;
        }
        self.ring[slot] = Some(RootRecord { txg, instance: self.instance, floor: self.floor_carried, is_self_verified: true });
        if self.current_window_changes_user_state {
            self.nonempty_txgs.insert(txg);
            self.current_window_changes_user_state = false;
        }
        self.newest_durable_txg = txg;
        self.maximum_published_txg = self.maximum_published_txg.max(txg);
        self.next_txg = txg + 1;
        self.save_snapshot();
        if let Some(mut pending) = self.pending_floor {
            let carrier = *pending.carrier_txg.get_or_insert(txg);
            if txg > carrier {
                pending.durable_publications_since_carrier += 1;
            }
            let is_active = match self.arm {
                Arm::FloorTxg => txg >= carrier + TXG_ARITHMETIC_GENERATIONS,
                Arm::FloorPerDiskCountActivation => pending.durable_publications_since_carrier >= COUNT_ACTIVATION_PUBLICATIONS,
                Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit => self.floor_written_on_every_surviving_disk(pending.value),
                Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum => true,
            };
            if is_active {
                self.floor_active = self.floor_active.max(pending.value);
                self.pending_floor = None;
            } else {
                self.pending_floor = Some(pending);
            }
        }
        self.prune_damage();
        self.record_check();
    }

    fn empty_publication(&mut self) -> bool {
        let txg = self.next_txg;
        let old_metadata = std::mem::take(&mut self.pool.metadata);
        self.free_blocks(&old_metadata, txg);
        match self.allocate(self.empty_publication_cost, txg) {
            None => {
                self.metrics.stalls += 1;
                self.halted = true;
                false
            }
            Some(blocks) => {
                self.pool.metadata = blocks;
                self.metrics.forced_publications += 1;
                self.publish(txg);
                !self.halted
            }
        }
    }

    /// F 只增不减。
    fn raise_floor_to(&mut self, target: u64) {
        if target > self.floor_carried {
            self.floor_carried = target;
            self.pending_floor = Some(PendingFloor { value: target, carrier_txg: None, durable_publications_since_carrier: 0 });
        }
    }

    /// 有界处置的第一步：把正在攒的窗口（里面已有这次删掉的对象）先发布出去。
    fn commit_open_window(&mut self, txg: u64) -> bool {
        match self.allocate(METADATA_BLOCKS_PER_PUBLICATION, txg) {
            None => {
                self.metrics.stalls += 1;
                self.halted = true;
                false
            }
            Some(blocks) => {
                self.pool.metadata = blocks;
                self.metrics.forced_publications += 1;
                self.publish(txg);
                !self.halted
            }
        }
    }

    fn is_goal_released(&self, goal_txg: u64) -> bool {
        self.reuse_bound() >= goal_txg
    }

    /// 发布正在攒的窗口之后按臂处置，返回做了几次空发布。
    /// stop_when_writable = false 时一直做到这次删的块放回为止（判据 1 的探针用）。
    fn push_after_commit(&mut self, goal_txg: u64, stop_when_writable: bool) -> u64 {
        let maximum_empty_publications = match self.arm {
            Arm::RingPersisted => self.ring_depth - 1,
            Arm::ProtectedGenerationsAtMinimum => 0,
            Arm::FloorTxg => TXG_ARITHMETIC_GENERATIONS + 2,
            Arm::FloorPerDisk | Arm::FloorPerDiskCountActivation | Arm::FloorPerDiskTxgUpperLimit => FLOOR_PER_DISK_PUBLICATION_BOUND - 1,
        };
        if self.arm == Arm::FloorTxg {
            self.raise_floor_to(self.floor_upper_limit());
        }
        let mut publications = 0u64;
        for _ in 0..maximum_empty_publications {
            if self.halted {
                break;
            }
            let is_done = match self.arm {
                Arm::FloorTxg => self.pending_floor.is_none(),
                Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorPerDisk | Arm::FloorPerDiskCountActivation | Arm::FloorPerDiskTxgUpperLimit => {
                    if stop_when_writable { self.user_allocatable() >= USER_BLOCKS } else { self.is_goal_released(goal_txg) }
                }
            };
            if is_done {
                break;
            }
            match self.arm {
                Arm::FloorPerDisk | Arm::FloorPerDiskCountActivation | Arm::FloorPerDiskTxgUpperLimit => {
                    // 等上限够到 t 才抬、一次抬到 t（预注册：每次都抬到当时的上限会让带新 F 的根反复重来）
                    if self.floor_carried < goal_txg && self.floor_upper_limit() >= goal_txg {
                        self.raise_floor_to(goal_txg);
                    }
                }
                Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg => {}
            }
            if !self.empty_publication() {
                break;
            }
            publications += 1;
        }
        publications
    }

    /// 故障世界里 F 臂在故障之前的强制盘紧：抬到上限并推到生效（E135 跑前补登第 1 条）。
    fn forced_pressure(&mut self) {
        let maximum_empty_publications = match self.arm {
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum => return,
            Arm::FloorTxg => TXG_ARITHMETIC_GENERATIONS + 2,
            Arm::FloorPerDisk | Arm::FloorPerDiskCountActivation | Arm::FloorPerDiskTxgUpperLimit => FLOOR_PER_DISK_PUBLICATION_BOUND - 1,
        };
        self.raise_floor_to(self.floor_upper_limit());
        for _ in 0..maximum_empty_publications {
            if self.halted || self.pending_floor.is_none() {
                break;
            }
            if !self.empty_publication() {
                break;
            }
        }
    }

    /// 一个用户窗口：（删一个对象）、放掉上一次的元数据、准入、建一个对象、写元数据、发布。
    fn run_user_window(&mut self, delete_first: bool, crash_before_root: bool) -> WindowOutcome {
        if self.halted {
            return WindowOutcome::Halted;
        }
        self.metrics.user_windows += 1;
        let open_txg = self.next_txg;
        let mut is_deleted = false;
        if delete_first && !self.pool.objects.is_empty() {
            let victim = usize::try_from(self.next_random() % self.pool.objects.len() as u64).expect("下标装得进 usize");
            let freed_object = self.pool.objects.swap_remove(victim);
            self.free_blocks(&freed_object, open_txg);
            is_deleted = true;
        }
        let old_metadata = std::mem::take(&mut self.pool.metadata);
        self.free_blocks(&old_metadata, open_txg);
        self.current_window_changes_user_state = is_deleted;
        let mut create_txg = open_txg;
        if self.user_allocatable() < USER_BLOCKS {
            self.metrics.admissions_under_pressure += 1;
            if !self.commit_open_window(open_txg) {
                return WindowOutcome::Halted;
            }
            let committed_txg = self.newest_durable_txg;
            let publications = 1 + self.push_after_commit(committed_txg, true);
            self.metrics.maximum_publications_per_admission = self.metrics.maximum_publications_per_admission.max(publications);
            if self.halted {
                return WindowOutcome::Halted;
            }
            if self.user_allocatable() < USER_BLOCKS {
                if self.df_raw() >= USER_BLOCKS {
                    self.metrics.false_enospc_raw += 1;
                }
                if self.df_guaranteed() >= USER_BLOCKS {
                    self.metrics.false_enospc_guaranteed += 1;
                } else {
                    self.metrics.true_enospc += 1;
                }
                return WindowOutcome::NoSpace;
            }
            create_txg = self.next_txg;
            let committed_metadata = std::mem::take(&mut self.pool.metadata);
            self.free_blocks(&committed_metadata, create_txg);
        }
        let blocks = self.allocate(USER_BLOCKS, create_txg).expect("user_allocatable 说够，allocate 却拿不到");
        let mut object = [0usize; USER_BLOCKS_PER_OBJECT];
        object.copy_from_slice(&blocks);
        self.pool.objects.push(object);
        self.current_window_changes_user_state = true;
        match self.allocate(METADATA_BLOCKS_PER_PUBLICATION, create_txg) {
            None => {
                self.metrics.stalls += 1;
                self.halted = true;
                WindowOutcome::Halted
            }
            Some(metadata) => {
                self.pool.metadata = metadata;
                if crash_before_root {
                    return WindowOutcome::AllocatedWithoutPublish;
                }
                self.publish(create_txg);
                WindowOutcome::Created
            }
        }
    }

    /// 恢复之后 F 的两个值（候选下界, 生效值）。
    fn floors_after_recovery(&self, chosen: &RootRecord) -> (u64, u64) {
        let candidacy = self.candidacy_floor();
        let settled = match self.arm {
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum => 0,
            // E135 臂 D 原样：最新的「txg + K_min ≤ 所选根」的有效根所带的 F
            Arm::FloorTxg => self.valid_roots().iter()
                .filter(|root| root.txg + TXG_ARITHMETIC_GENERATIONS <= chosen.txg)
                .max_by_key(|root| root.txg).map(|root| root.floor).unwrap_or(0).min(candidacy),
            // 每块幸存盘各取「该盘有效根所带 F 的最大值」，再取最小值
            Arm::FloorPerDisk | Arm::FloorPerDiskCountActivation | Arm::FloorPerDiskTxgUpperLimit => {
                let roots = self.valid_roots_with_disk();
                self.surviving_disks().iter()
                    .map(|disk| roots.iter().filter(|(root_disk, _)| root_disk == disk).map(|(_, root)| root.floor).max().unwrap_or(0))
                    .min().unwrap_or(0).min(candidacy)
            }
        };
        (candidacy, settled)
    }

    fn adopt_floors(&mut self, candidacy: u64, settled: u64) {
        self.floor_carried = candidacy;
        self.floor_active = settled;
        self.pending_floor = if candidacy > settled {
            let roots = self.valid_roots();
            let carrier = roots.iter().filter(|root| root.floor >= candidacy).map(|root| root.txg).min();
            let durable_since = match carrier {
                None => 0,
                Some(carrier_txg) => roots.iter().map(|root| root.txg).filter(|txg| *txg > carrier_txg).collect::<BTreeSet<u64>>().len() as u64,
            };
            Some(PendingFloor { value: candidacy, carrier_txg: carrier, durable_publications_since_carrier: durable_since })
        } else {
            None
        };
    }

    /// 崩溃之后的恢复：先验全部候选再择新（D22（单元原子性怎么合成） 已定项 7），分配状态从所选根重新载入，取新实例代号。
    fn crash_and_recover(&mut self) {
        let chosen = match self.newest_valid_root() {
            None => {
                self.metrics.unrecoverable += 1;
                self.halted = true;
                return;
            }
            Some(root) => root,
        };
        if self.damaged_txgs.contains(&chosen.txg) {
            self.metrics.unrecoverable += 1;
        }
        let (candidacy, settled) = self.floors_after_recovery(&chosen);
        if !self.restore_snapshot(chosen.instance, chosen.txg) {
            self.metrics.unrecoverable += 1;
            self.halted = true;
            return;
        }
        self.instance_rows.push((self.instance, chosen.txg));
        self.instance += 1;
        self.newest_durable_txg = chosen.txg;
        self.next_txg = self.maximum_published_txg + 1;
        self.adopt_floors(candidacy, settled);
        self.save_snapshot();
        self.record_check();
    }

    /// 管理员回退到候选集里最旧的根（D23（journal 的角色与格式） 已定项 14 的形态）。
    fn admin_rollback(&mut self) -> bool {
        let floor = self.candidacy_floor();
        let target = match self.valid_roots().into_iter().filter(|root| root.txg >= floor).min_by_key(|root| root.txg) {
            None => return false,
            Some(root) => root,
        };
        let (candidacy, settled) = self.floors_after_recovery(&target);
        let abandoned_pool = Rc::new(self.pool.clone());
        let abandoned_newest_txg = self.visible_roots().iter().map(|root| root.txg).max().unwrap_or(0);
        if !self.restore_snapshot(target.instance, target.txg) {
            return false;
        }
        self.abandoned_view = Some((abandoned_pool, abandoned_newest_txg));
        self.admin_rollback_happened = true;
        self.instance_rows.push((target.instance, target.txg));
        for abandoned_instance in (target.instance + 1)..=self.instance {
            self.instance_rows.push((abandoned_instance, 0));
        }
        self.instance += 1;
        self.newest_durable_txg = target.txg;
        let newest_ring_txg = self.visible_roots().iter().map(|root| root.txg).max().unwrap_or(0);
        self.next_txg = newest_ring_txg.max(self.maximum_published_txg) + 1;
        self.adopt_floors(candidacy, settled);
        self.save_snapshot();
        self.record_check();
        true
    }

    /// C281 的构造：回退那次发布写了单元、根槽 FUA 之前崩溃。回退行没落盘，被抛弃的根仍按实例表有效。
    fn admin_rollback_then_crash_before_root(&mut self) {
        let durable_rows = self.instance_rows.clone();
        let durable_instance = self.instance;
        let durable_newest = self.newest_durable_txg;
        let durable_next = self.next_txg;
        let durable_floors = (self.floor_carried, self.floor_active, self.pending_floor);
        if !self.admin_rollback() {
            return;
        }
        let txg = self.next_txg;
        let _ = self.allocate(BLOCKS_PER_PUBLICATION, txg);
        self.instance_rows = durable_rows;
        self.instance = durable_instance;
        self.newest_durable_txg = durable_newest;
        self.next_txg = durable_next;
        self.floor_carried = durable_floors.0;
        self.floor_active = durable_floors.1;
        self.pending_floor = durable_floors.2;
        self.crash_and_recover();
        self.halted = true;
    }
}

/// 故障落在哪个窗口、盘紧提前几个窗口（E135 跑前补登第 1 条）。
fn schedule(seed: u64, ring_depth: u64, world: World) -> (u64, u64) {
    let mut state = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1;
    state ^= state >> 29;
    let event_window = 2 * ring_depth + 20 + state % 150;
    let lead = match world {
        World::AdminRollback => TXG_ARITHMETIC_GENERATIONS + 2,
        World::TornNewest | World::DiskLoss | World::SlotWriteFail => if seed % 2 == 0 { 0 } else { TXG_ARITHMETIC_GENERATIONS },
        World::Steady | World::RollbackFirstPublish => 0,
    };
    (event_window, lead)
}

fn run_world(arm: Arm, world: World, slots_per_region: u64, seed: u64, lost_disk: usize) -> Metrics {
    let keep_snapshots = match world {
        World::TornNewest | World::DiskLoss | World::AdminRollback | World::RollbackFirstPublish => true,
        World::Steady | World::SlotWriteFail => false,
    };
    let mut simulation = Simulation::new(arm, slots_per_region, EMPTY_PUBLICATION_COST_SWEEP[0], seed, keep_snapshots, None);
    simulation.prefill(STEADY_FILL);
    let (event_window, lead) = schedule(seed, simulation.ring_depth, world);
    for window in 0..WINDOWS_PER_WORLD {
        if simulation.halted {
            break;
        }
        if world.forces_pressure_before_event() && arm.uses_floor() && window + lead == event_window {
            simulation.forced_pressure();
        }
        if window == event_window {
            match world {
                World::TornNewest => simulation.tear_next_publication = true,
                World::SlotWriteFail => simulation.fail_next_slot_write = true,
                World::AdminRollback => {
                    simulation.admin_rollback();
                }
                World::RollbackFirstPublish => simulation.admin_rollback_then_crash_before_root(),
                World::Steady | World::DiskLoss => {}
            }
        }
        simulation.run_user_window(true, false);
        if world == World::DiskLoss && window == event_window && !simulation.halted {
            simulation.disk_lost[lost_disk] = true;
            simulation.crash_and_recover();
            simulation.halted = true;
        }
    }
    simulation.metrics
}

/// 写失败组合：空集、单个 k、两个 k，k ∈ 0..5，共 16 种。
fn failure_patterns() -> Vec<Vec<u64>> {
    let mut patterns = vec![Vec::new()];
    for first in 0..ACTIVATION_FAULT_ATTEMPTS {
        patterns.push(vec![first]);
    }
    for first in 0..ACTIVATION_FAULT_ATTEMPTS {
        for second in (first + 1)..ACTIVATION_FAULT_ATTEMPTS {
            patterns.push(vec![first, second]);
        }
    }
    patterns
}

/// activation_faults 的前缀：跑到第 E 个窗口，F 臂只抬一次 F（不推空）。
fn activation_fault_prefix(arm: Arm, slots_per_region: u64, seed: u64) -> Option<Simulation> {
    let mut simulation = Simulation::new(arm, slots_per_region, EMPTY_PUBLICATION_COST_SWEEP[0], seed, true, None);
    simulation.prefill(STEADY_FILL);
    let (event_window, _) = schedule(seed, simulation.ring_depth, World::Steady);
    for _ in 0..event_window {
        simulation.run_user_window(true, false);
    }
    if simulation.halted {
        return None;
    }
    if arm.uses_floor() {
        simulation.raise_floor_to(simulation.floor_upper_limit());
    }
    Some(simulation)
}

fn run_activation_fault_case(prefix: &Simulation, pattern: &[u64], crash_window: u64, lost_disk: usize) -> Metrics {
    let mut simulation = prefix.clone();
    simulation.metrics = Metrics::fresh();
    let base = simulation.root_write_attempts;
    for offset in pattern {
        simulation.planned_root_write_failures.insert(base + offset);
    }
    for _ in 1..crash_window {
        simulation.run_user_window(true, false);
    }
    let _ = simulation.run_user_window(true, true);
    if !simulation.halted {
        simulation.disk_lost[lost_disk] = true;
        simulation.crash_and_recover();
    }
    simulation.metrics
}

/// near_full：预热 2N + 10 个窗口，只建不删直到第一次 ENOSPC，再做 400 个「删一个、建一个」。
fn run_near_full(arm: Arm, slots_per_region: u64, empty_publication_cost: u64, seed: u64, reserve_override: Option<u64>) -> Metrics {
    let mut simulation = Simulation::new(arm, slots_per_region, empty_publication_cost, seed, false, reserve_override);
    simulation.prefill(STEADY_FILL);
    let warm_up = 2 * simulation.ring_depth + 10;
    for _ in 0..warm_up {
        simulation.run_user_window(true, false);
    }
    let stalls_while_warming = simulation.metrics.stalls;
    simulation.metrics = Metrics::fresh();
    simulation.metrics.stalls = stalls_while_warming;
    for _ in 0..CAPACITY_BLOCKS {
        match simulation.run_user_window(false, false) {
            WindowOutcome::Created => {}
            WindowOutcome::NoSpace | WindowOutcome::Halted | WindowOutcome::AllocatedWithoutPublish => break,
        }
    }
    simulation.metrics.halted_before_counting = u64::from(simulation.halted);
    for _ in 0..WINDOWS_PER_WORLD {
        if simulation.halted {
            break;
        }
        match simulation.run_user_window(true, false) {
            WindowOutcome::NoSpace => simulation.metrics.write_after_delete_failed += 1,
            WindowOutcome::Created | WindowOutcome::Halted | WindowOutcome::AllocatedWithoutPublish => {}
        }
    }
    simulation.metrics
}

/// 判据 1（跨装置闸）：steady 下最后一个窗口，在发布之前与发布刚完成各量一次。
fn steady_probe(arm: Arm, slots_per_region: u64) -> (u64, u64, u64, u64) {
    let mut simulation = Simulation::new(arm, slots_per_region, EMPTY_PUBLICATION_COST_SWEEP[0], 1, false, None);
    simulation.prefill(STEADY_FILL);
    for _ in 0..(WINDOWS_PER_WORLD - 1) {
        simulation.run_user_window(true, false);
    }
    let outcome = simulation.run_user_window(true, true);
    assert_eq!(outcome, WindowOutcome::AllocatedWithoutPublish, "steady 下不该走盘紧处置");
    let txg = simulation.next_txg;
    let pinned_before = simulation.pinned_count();
    let fake_before = simulation.count_fake_candidates().0;
    simulation.publish(txg);
    let pinned_after = simulation.pinned_count();
    let fake_after = simulation.count_fake_candidates().0;
    (pinned_before, pinned_after, fake_before, fake_after)
}

/// 判据 1（跑前式子）：从 steady 触发一次完整的有界处置，做到这次删的块放回为止。
/// 返回（用了几次发布，含发布正在攒的窗口；之后还扣着几块；这次删的块放回了没有）。
fn push_probe(arm: Arm, slots_per_region: u64, empty_publication_cost: u64, extra_windows: u64) -> (u64, u64, bool) {
    let mut simulation = Simulation::new(arm, slots_per_region, empty_publication_cost, 1, false, None);
    simulation.prefill(STEADY_FILL);
    for _ in 0..(200 + extra_windows) {
        simulation.run_user_window(true, false);
    }
    let open_txg = simulation.next_txg;
    let victim = simulation.pool.objects.swap_remove(0);
    simulation.free_blocks(&victim, open_txg);
    let old_metadata = std::mem::take(&mut simulation.pool.metadata);
    simulation.free_blocks(&old_metadata, open_txg);
    assert!(simulation.commit_open_window(open_txg), "steady 下发布正在攒的窗口不该卡死");
    let goal_txg = simulation.newest_durable_txg;
    let publications = 1 + simulation.push_after_commit(goal_txg, false);
    (publications, simulation.pinned_count(), simulation.is_goal_released(goal_txg))
}

/// 预注册判据 1 的期望：(允许的发布数集合, 最大发布数, 最大扣住块数)。
fn expected_push(arm: Arm, slots_per_region: u64, empty_publication_cost: u64) -> (Vec<u64>, u64, u64) {
    let ring_depth = REGION_COUNT * slots_per_region;
    if slots_per_region == 1 {
        return (vec![3], 3, METADATA_BLOCKS_PER_PUBLICATION + empty_publication_cost);
    }
    match arm {
        Arm::RingPersisted => (vec![ring_depth], ring_depth, METADATA_BLOCKS_PER_PUBLICATION + (ring_depth - 2) * empty_publication_cost),
        Arm::FloorPerDisk => (vec![6, 7], 7, METADATA_BLOCKS_PER_PUBLICATION + 5 * empty_publication_cost),
        Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDiskCountActivation | Arm::FloorPerDiskTxgUpperLimit => (Vec::new(), 0, 0),
    }
}

/// 三种 t 的落点（t mod 3）各做一次处置探针：(发布数集合, 最大发布数, 最大扣住块数, 全部放回)。
fn push_probe_over_landings(arm: Arm, slots_per_region: u64, empty_publication_cost: u64) -> (Vec<u64>, u64, u64, bool) {
    let results: Vec<(u64, u64, bool)> = (0..REGION_COUNT).map(|extra| push_probe(arm, slots_per_region, empty_publication_cost, extra)).collect();
    let publication_set: Vec<u64> = results.iter().map(|(publications, _, _)| *publications).collect::<BTreeSet<u64>>().into_iter().collect();
    let maximum_publications = results.iter().map(|(publications, _, _)| *publications).max().unwrap_or(0);
    let maximum_pinned = results.iter().map(|(_, pinned, _)| *pinned).max().unwrap_or(0);
    let is_all_released = results.iter().all(|(_, _, is_released)| *is_released);
    (publication_set, maximum_publications, maximum_pinned, is_all_released)
}

fn format_set(values: &[u64]) -> String {
    values.iter().map(|value| value.to_string()).collect::<Vec<String>>().join("/")
}

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!(
        "name=config r={REGION_COUNT} disks={DISK_COUNT} k_min_txg={TXG_ARITHMETIC_GENERATIONS} retained_roots={MINIMUM_RETAINED_ROOTS} \
         bound_g={FLOOR_PER_DISK_PUBLICATION_BOUND} user_blocks={USER_BLOCKS} metadata_blocks={METADATA_BLOCKS_PER_PUBLICATION} \
         capacity={CAPACITY_BLOCKS} windows={WINDOWS_PER_WORLD} seeds={SEED_COUNT} activation_seeds={ACTIVATION_FAULT_SEED_COUNT} \
         sweep_seeds={RESERVE_SWEEP_SEED_COUNT} block_bytes={BLOCK_BYTES}"
    )));

    // 判据 1：跨装置闸与跑前式子
    let mut criterion_one_holds = true;
    for slots_per_region in SLOTS_PER_REGION_SWEEP {
        let ring_depth = REGION_COUNT * slots_per_region;
        let (pinned_before, pinned_after, _, _) = steady_probe(Arm::RingPersisted, slots_per_region);
        let expect_before = ring_depth * BLOCKS_PER_PUBLICATION;
        let expect_after = (ring_depth - 1) * BLOCKS_PER_PUBLICATION;
        criterion_one_holds &= pinned_before == expect_before && pinned_after == expect_after;
        println!("{}", emitter.emit_raw(&format!(
            "name=crit1_pinned arm=A s={slots_per_region} pinned_before={pinned_before} expect_before={expect_before} \
             pinned_after={pinned_after} expect_after={expect_after}"
        )));
        let (_, _, fake_before, fake_after) = steady_probe(Arm::ProtectedGenerationsAtMinimum, slots_per_region);
        let expect_fake_before = ring_depth - TXG_ARITHMETIC_GENERATIONS;
        let expect_fake_after = ring_depth.saturating_sub(TXG_ARITHMETIC_GENERATIONS + 1);
        criterion_one_holds &= fake_before == expect_fake_before && fake_after == expect_fake_after;
        println!("{}", emitter.emit_raw(&format!(
            "name=crit1_fake arm=C s={slots_per_region} fake_before={fake_before} expect_before={expect_fake_before} \
             fake_after={fake_after} expect_after={expect_fake_after}"
        )));
    }
    for arm in [Arm::RingPersisted, Arm::FloorPerDisk] {
        for slots_per_region in SLOTS_PER_REGION_SWEEP {
            for empty_publication_cost in EMPTY_PUBLICATION_COST_SWEEP {
                let (publication_set, maximum_publications, maximum_pinned, is_all_released) = push_probe_over_landings(arm, slots_per_region, empty_publication_cost);
                let (expect_set, expect_maximum, expect_pinned) = expected_push(arm, slots_per_region, empty_publication_cost);
                let holds = is_all_released && maximum_publications == expect_maximum && maximum_pinned == expect_pinned
                    && publication_set.iter().all(|publications| expect_set.contains(publications));
                criterion_one_holds &= holds;
                println!("{}", emitter.emit_raw(&format!(
                    "name=crit1_push arm={} s={slots_per_region} empty_cost={empty_publication_cost} publications={} max_publications={maximum_publications} \
                     expect_publications={} expect_max={expect_maximum} max_pinned={maximum_pinned} expect_pinned={expect_pinned} released={} holds={}",
                    arm.name(), format_set(&publication_set), format_set(&expect_set), u8::from(is_all_released), u8::from(holds)
                )));
            }
        }
    }

    // 故障世界
    let mut candidate_fakes = [0u64; 2];
    let mut candidate_unrecoverable = [0u64; 2];
    let mut floor_per_disk_retention = 0u64;
    let mut txg_upper_limit_retention = 0u64;
    let mut first_publish_unrecoverable = [0u64; 6];
    for world in WORLDS {
        for slots_per_region in SLOTS_PER_REGION_SWEEP {
            for (arm_index, arm) in ARMS.iter().enumerate() {
                let lost_disks: &[usize] = if world == World::DiskLoss { &[0, 1] } else { &[0] };
                for &lost_disk in lost_disks {
                    let mut total = Metrics::fresh();
                    for seed in 0..SEED_COUNT {
                        total.add(&run_world(*arm, world, slots_per_region, seed, lost_disk));
                    }
                    println!("{}", emitter.emit_raw(&format!(
                        "name=cell arm={} world={} s={slots_per_region} lost_disk={lost_disk} fakes={} unrecoverable={} retention_violations={} \
                         checks={} min_candidates={} avg_candidates_x100={} avg_nonempty_x100={} stalls={}",
                        arm.name(), world.name(), total.fake_candidates, total.unrecoverable, total.retention_violations, total.checks,
                        total.minimum_candidates_or_zero(), total.average_candidates_in_hundredths(),
                        total.average_nonempty_candidates_in_hundredths(), total.stalls
                    )));
                    if world == World::RollbackFirstPublish {
                        first_publish_unrecoverable[arm_index] += total.unrecoverable;
                        continue;
                    }
                    match arm {
                        Arm::RingPersisted => {
                            candidate_fakes[0] += total.fake_candidates;
                            candidate_unrecoverable[0] += total.unrecoverable;
                        }
                        Arm::FloorPerDisk => {
                            candidate_fakes[1] += total.fake_candidates;
                            candidate_unrecoverable[1] += total.unrecoverable;
                            floor_per_disk_retention += total.retention_violations;
                        }
                        Arm::FloorPerDiskTxgUpperLimit => {
                            if world == World::SlotWriteFail {
                                txg_upper_limit_retention += total.retention_violations;
                            }
                        }
                        Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDiskCountActivation => {}
                    }
                }
            }
        }
    }

    // activation_faults
    let patterns = failure_patterns();
    let mut floor_txg_activation_fakes = 0u64;
    let mut protected_generations_activation_unrecoverable = 0u64;
    let mut count_activation_double_fakes = 0u64;
    let mut detail_lines = 0u64;
    for slots_per_region in ACTIVATION_FAULT_SLOTS_SWEEP {
        for arm in ARMS {
            let mut by_class = [[Metrics::fresh(); 3]; DISK_COUNT];
            for seed in 0..ACTIVATION_FAULT_SEED_COUNT {
                let prefix = match activation_fault_prefix(arm, slots_per_region, seed) {
                    None => continue,
                    Some(simulation) => simulation,
                };
                for pattern in &patterns {
                    for crash_window in 1..=ACTIVATION_FAULT_CRASH_WINDOWS {
                        for lost_disk in 0..DISK_COUNT {
                            let case = run_activation_fault_case(&prefix, pattern, crash_window, lost_disk);
                            by_class[lost_disk][pattern.len()].add(&case);
                            if arm.is_candidate() && (case.fake_candidates > 0 || case.unrecoverable > 0) && detail_lines < DETAIL_LINES_MAXIMUM {
                                detail_lines += 1;
                                println!("{}", emitter.emit_raw(&format!(
                                    "name=detail_activation arm={} s={slots_per_region} seed={seed} failures={} crash_window={crash_window} \
                                     lost_disk={lost_disk} fakes={} unrecoverable={}",
                                    arm.name(), format_set(pattern), case.fake_candidates, case.unrecoverable
                                )));
                            }
                        }
                    }
                }
            }
            for (lost_disk, classes) in by_class.iter().enumerate() {
                for (failure_count, total) in classes.iter().enumerate() {
                    println!("{}", emitter.emit_raw(&format!(
                        "name=activation arm={} s={slots_per_region} lost_disk={lost_disk} failures={failure_count} fakes={} unrecoverable={} \
                         retention_violations={} checks={}",
                        arm.name(), total.fake_candidates, total.unrecoverable, total.retention_violations, total.checks
                    )));
                    match arm {
                        Arm::RingPersisted => {
                            candidate_fakes[0] += total.fake_candidates;
                            candidate_unrecoverable[0] += total.unrecoverable;
                        }
                        Arm::FloorPerDisk => {
                            candidate_fakes[1] += total.fake_candidates;
                            candidate_unrecoverable[1] += total.unrecoverable;
                            floor_per_disk_retention += total.retention_violations;
                        }
                        Arm::FloorTxg => floor_txg_activation_fakes += total.fake_candidates,
                        Arm::ProtectedGenerationsAtMinimum => protected_generations_activation_unrecoverable += total.unrecoverable,
                        Arm::FloorPerDiskCountActivation => {
                            if failure_count == 2 {
                                count_activation_double_fakes += total.fake_candidates;
                            }
                        }
                        Arm::FloorPerDiskTxgUpperLimit => txg_upper_limit_retention += total.retention_violations,
                    }
                }
            }
        }
    }

    // near_full
    let mut candidate_false_guaranteed = [0u64; 2];
    let mut candidate_bound_exceeded_cells = [0u64; 2];
    let mut candidate_write_after_delete_failed = [0u64; 2];
    let mut candidate_stalls = [0u64; 2];
    let mut ring_raw_false_enospc_at_sixteen_cost_five = 0u64;
    for slots_per_region in SLOTS_PER_REGION_SWEEP {
        for empty_publication_cost in EMPTY_PUBLICATION_COST_SWEEP {
            for arm in ARMS {
                let ring_depth = REGION_COUNT * slots_per_region;
                let mut total = Metrics::fresh();
                for seed in 0..SEED_COUNT {
                    total.add(&run_near_full(arm, slots_per_region, empty_publication_cost, seed, None));
                }
                let reserve = arm.reserve_blocks(ring_depth, empty_publication_cost);
                let residue = arm.push_residue_blocks(ring_depth, empty_publication_cost);
                let bound = arm.publication_bound(ring_depth);
                println!("{}", emitter.emit_raw(&format!(
                    "name=near_full arm={} s={slots_per_region} empty_cost={empty_publication_cost} reserve={reserve} residue={residue} \
                     hidden_blocks={} hidden_bytes={} false_enospc_raw={} false_enospc_guaranteed={} true_enospc={} write_after_delete_failed={} \
                     max_publications={} bound={bound} admissions_under_pressure={} forced_per_window_x100={} stalls={} halted_before_counting={} \
                     fakes={} unrecoverable={} retention_violations={} min_candidates={} avg_candidates_x100={} avg_nonempty_x100={}",
                    arm.name(), reserve + residue, (reserve + residue) * BLOCK_BYTES, total.false_enospc_raw, total.false_enospc_guaranteed,
                    total.true_enospc, total.write_after_delete_failed, total.maximum_publications_per_admission, total.admissions_under_pressure,
                    total.forced_publications_per_window_in_hundredths(), total.stalls, total.halted_before_counting, total.fake_candidates,
                    total.unrecoverable, total.retention_violations, total.minimum_candidates_or_zero(),
                    total.average_candidates_in_hundredths(), total.average_nonempty_candidates_in_hundredths()
                )));
                if arm == Arm::RingPersisted && slots_per_region == 16 && empty_publication_cost == 5 {
                    ring_raw_false_enospc_at_sixteen_cost_five += total.false_enospc_raw;
                }
                let candidate_index = match arm {
                    Arm::RingPersisted => 0,
                    Arm::FloorPerDisk => 1,
                    Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDiskCountActivation | Arm::FloorPerDiskTxgUpperLimit => continue,
                };
                candidate_false_guaranteed[candidate_index] += total.false_enospc_guaranteed;
                candidate_bound_exceeded_cells[candidate_index] += u64::from(total.maximum_publications_per_admission > bound);
                candidate_write_after_delete_failed[candidate_index] += total.write_after_delete_failed;
                candidate_stalls[candidate_index] += total.stalls;
                candidate_fakes[candidate_index] += total.fake_candidates;
                candidate_unrecoverable[candidate_index] += total.unrecoverable;
                if arm == Arm::FloorPerDisk {
                    floor_per_disk_retention += total.retention_violations;
                }
            }
        }
    }

    // 保留池扫描
    let mut minimum_no_stall_reserve = [[u64::MAX; 2]; 2];
    for (candidate_index, arm) in [Arm::RingPersisted, Arm::FloorPerDisk].iter().enumerate() {
        for (cost_index, empty_publication_cost) in EMPTY_PUBLICATION_COST_SWEEP.iter().enumerate() {
            let ring_depth = REGION_COUNT * RESERVE_SWEEP_SLOTS_PER_REGION;
            let formula = arm.reserve_blocks(ring_depth, *empty_publication_cost);
            for reserve in (formula - RESERVE_SWEEP_BELOW)..=(formula + RESERVE_SWEEP_ABOVE) {
                let mut total = Metrics::fresh();
                for seed in 0..RESERVE_SWEEP_SEED_COUNT {
                    total.add(&run_near_full(*arm, RESERVE_SWEEP_SLOTS_PER_REGION, *empty_publication_cost, seed, Some(reserve)));
                }
                println!("{}", emitter.emit_raw(&format!(
                    "name=reserve_sweep arm={} s={RESERVE_SWEEP_SLOTS_PER_REGION} empty_cost={empty_publication_cost} reserve={reserve} formula={formula} \
                     stalls={} false_enospc_guaranteed={} write_after_delete_failed={} max_publications={} fakes={}",
                    arm.name(), total.stalls, total.false_enospc_guaranteed, total.write_after_delete_failed,
                    total.maximum_publications_per_admission, total.fake_candidates
                )));
                if total.stalls == 0 {
                    minimum_no_stall_reserve[candidate_index][cost_index] = minimum_no_stall_reserve[candidate_index][cost_index].min(reserve);
                }
                if reserve >= formula {
                    candidate_false_guaranteed[candidate_index] += total.false_enospc_guaranteed;
                    candidate_write_after_delete_failed[candidate_index] += total.write_after_delete_failed;
                    candidate_stalls[candidate_index] += total.stalls;
                    candidate_fakes[candidate_index] += total.fake_candidates;
                    candidate_unrecoverable[candidate_index] += total.unrecoverable;
                }
            }
        }
    }

    println!("{}", emitter.emit_raw(&format!(
        "name=verdict crit1_holds={} crit2_d_activation_fakes={floor_txg_activation_fakes} crit2_c_activation_unrecoverable={protected_generations_activation_unrecoverable} \
         crit2_gcount_double_fakes={count_activation_double_fakes} crit2_gtxgcap_retention={txg_upper_limit_retention} \
         crit2_a_raw_false_enospc_s16_cost5={ring_raw_false_enospc_at_sixteen_cost_five} \
         crit3_a_fakes={} crit3_a_unrecoverable={} crit3_g_fakes={} crit3_g_unrecoverable={} \
         crit4_a_false_guaranteed={} crit4_g_false_guaranteed={} crit5_a_bound_exceeded_cells={} crit5_g_bound_exceeded_cells={} \
         crit5_a_write_after_delete_failed={} crit5_g_write_after_delete_failed={} crit6_a_stalls={} crit6_g_stalls={} crit7_g_retention={floor_per_disk_retention}",
        u8::from(criterion_one_holds), candidate_fakes[0], candidate_unrecoverable[0], candidate_fakes[1], candidate_unrecoverable[1],
        candidate_false_guaranteed[0], candidate_false_guaranteed[1], candidate_bound_exceeded_cells[0], candidate_bound_exceeded_cells[1],
        candidate_write_after_delete_failed[0], candidate_write_after_delete_failed[1], candidate_stalls[0], candidate_stalls[1]
    )));
    println!("{}", emitter.emit_raw(&format!(
        "name=crit6_min_no_stall_reserve a_cost1={} a_cost5={} g_cost1={} g_cost5={}",
        minimum_no_stall_reserve[0][0], minimum_no_stall_reserve[0][1], minimum_no_stall_reserve[1][0], minimum_no_stall_reserve[1][1]
    )));
    println!("{}", emitter.emit_raw(&format!(
        "name=crit8_c281 a={} c={} d={} g={} g_count={} g_txg_cap={}",
        first_publish_unrecoverable[0], first_publish_unrecoverable[1], first_publish_unrecoverable[2],
        first_publish_unrecoverable[3], first_publish_unrecoverable[4], first_publish_unrecoverable[5]
    )));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regions_zero_and_two_live_on_disk_zero_region_one_on_disk_one() {
        let simulation = Simulation::new(Arm::RingPersisted, 4, 1, 1, false, None);
        let disks: Vec<usize> = (0..12).map(|slot| simulation.disk_of_slot(slot)).collect();
        assert_eq!(disks, vec![0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0], "区域 0、2 在盘 0，区域 1 在盘 1");
    }

    #[test]
    fn criterion_one_pinned_blocks_match_e123_both_readings() {
        for (slots_per_region, before, after) in [(1u64, 39u64, 26u64), (4, 156, 143), (16, 624, 611)] {
            let (pinned_before, pinned_after, _, _) = steady_probe(Arm::RingPersisted, slots_per_region);
            assert_eq!(pinned_before, before, "S={slots_per_region} 发布之前应扣 N × 13");
            assert_eq!(pinned_after, after, "S={slots_per_region} 发布刚完成应扣 (N − 1) × 13");
        }
    }

    #[test]
    fn criterion_one_fake_candidates_match_e92_both_readings() {
        for (slots_per_region, before, after) in [(1u64, 0u64, 0u64), (4, 9, 8), (16, 45, 44)] {
            let (_, _, fake_before, fake_after) = steady_probe(Arm::ProtectedGenerationsAtMinimum, slots_per_region);
            assert_eq!(fake_before, before, "S={slots_per_region} 发布之前假候选应为 N − K");
            assert_eq!(fake_after, after, "S={slots_per_region} 发布之后假候选应为 N − K − 1");
        }
    }

    #[test]
    fn ring_push_uses_ring_depth_publications_and_leaves_preregistered_residue() {
        for (slots_per_region, empty_publication_cost, publications, pinned) in
            [(1u64, 1u64, 3u64, 6u64), (1, 5, 3, 10), (4, 1, 12, 15), (4, 5, 12, 55), (16, 1, 48, 51), (16, 5, 48, 235)]
        {
            let (publication_set, maximum_publications, maximum_pinned, is_all_released) =
                push_probe_over_landings(Arm::RingPersisted, slots_per_region, empty_publication_cost);
            assert!(is_all_released, "S={slots_per_region} c={empty_publication_cost} 处置之后这次删的块必须放回");
            assert_eq!(publication_set, vec![publications], "S={slots_per_region} c={empty_publication_cost} 臂 A 每种落点都是 N 次发布");
            assert_eq!(maximum_publications, publications);
            assert_eq!(maximum_pinned, pinned, "S={slots_per_region} c={empty_publication_cost} 臂 A 扣住 5 + (N − 2) × c");
            if slots_per_region > 1 {
                assert_eq!(maximum_pinned, Arm::RingPersisted.push_residue_blocks(REGION_COUNT * slots_per_region, empty_publication_cost), "残留式子与实测一致");
            }
        }
    }

    #[test]
    fn floor_per_disk_push_uses_six_or_seven_publications_and_leaves_preregistered_residue() {
        for (slots_per_region, empty_publication_cost, publications, maximum, pinned) in [
            (1u64, 1u64, vec![3u64], 3u64, 6u64),
            (1, 5, vec![3], 3, 10),
            (4, 1, vec![6, 7], 7, 10),
            (4, 5, vec![6, 7], 7, 30),
            (16, 1, vec![6, 7], 7, 10),
            (16, 5, vec![6, 7], 7, 30),
        ] {
            let (publication_set, maximum_publications, maximum_pinned, is_all_released) =
                push_probe_over_landings(Arm::FloorPerDisk, slots_per_region, empty_publication_cost);
            assert!(is_all_released, "S={slots_per_region} c={empty_publication_cost} 处置之后这次删的块必须放回");
            assert_eq!(publication_set, publications, "S={slots_per_region} c={empty_publication_cost} 臂 G 三种落点的发布数");
            assert_eq!(maximum_publications, maximum, "S={slots_per_region} c={empty_publication_cost} 臂 G 最多 B_G 次");
            assert_eq!(maximum_pinned, pinned, "S={slots_per_region} c={empty_publication_cost} 臂 G 扣住最多 5 + 5c（S = 1 时 5 + c）");
            if slots_per_region > 1 {
                assert_eq!(maximum_pinned, Arm::FloorPerDisk.push_residue_blocks(REGION_COUNT * slots_per_region, empty_publication_cost), "残留式子与实测一致");
            }
        }
    }

    fn activation_totals(arm: Arm, slots_per_region: u64, seeds: u64, failure_count: Option<usize>) -> Metrics {
        let mut total = Metrics::fresh();
        for seed in 0..seeds {
            let prefix = activation_fault_prefix(arm, slots_per_region, seed).expect("steady 前缀不该卡死");
            for pattern in failure_patterns().iter().filter(|pattern| failure_count.is_none_or(|count| pattern.len() == count)) {
                for crash_window in 1..=ACTIVATION_FAULT_CRASH_WINDOWS {
                    for lost_disk in 0..DISK_COUNT {
                        total.add(&run_activation_fault_case(&prefix, pattern, crash_window, lost_disk));
                    }
                }
            }
        }
        total
    }

    #[test]
    fn floor_txg_produces_fake_candidates_when_faults_hit_activation_interval() {
        let total = activation_totals(Arm::FloorTxg, 4, 2, None);
        assert!(total.fake_candidates > 0, "按 txg 算活化，活化区间里写失败再丢盘必须冒出假候选（第二轮反推腿那一构造）");
    }

    #[test]
    fn protected_generations_runtime_is_unrecoverable_when_a_jump_is_followed_by_disk_loss() {
        let total = activation_totals(Arm::ProtectedGenerationsAtMinimum, 4, 2, None);
        assert!(total.unrecoverable > 0, "K = 3 在跳号下不够：写失败 + 丢盘必须出不可恢复");
    }

    #[test]
    fn ring_arm_is_clean_when_faults_hit_activation_interval() {
        let total = activation_totals(Arm::RingPersisted, 4, 2, None);
        assert_eq!(total.fake_candidates, 0, "臂 A 在同一串故障下不许有假候选");
        assert_eq!(total.unrecoverable, 0, "臂 A 在同一串故障下不许不可恢复");
    }

    /// 判据 2 的 G_txgcap 那一格记「未验」：两个靶子世界都是先抬 F、后写失败，
    /// 抬 F 时盘 1 上最新的根 ≥ 最新 − 2 > F，之后的跳号只让盘 1 留着那个旧根，它仍 ≥ F。
    /// 这条对照要的是「写失败跳号之后再抬 F」，这一轮的世界里没有这个顺序。钉住这个观测，世界改了它会红。
    #[test]
    fn txg_upper_limit_control_is_not_reached_because_floor_is_raised_before_the_jump() {
        let total: u64 = (0..8).map(|seed| run_world(Arm::FloorPerDiskTxgUpperLimit, World::SlotWriteFail, 4, seed, 0).retention_violations).sum();
        assert_eq!(total, 0, "先抬 F 再跳号时按 txg 数 4 代的上限不越过盘 1 上最新的根；要测到这条对照得先跳号再抬 F");
    }

    #[test]
    fn floor_per_disk_is_clean_when_faults_hit_activation_interval() {
        for slots_per_region in [4u64, 16] {
            let total = activation_totals(Arm::FloorPerDisk, slots_per_region, 2, None);
            assert_eq!(total.fake_candidates, 0, "S={slots_per_region} 按盘生效，活化区间里写失败再丢盘不许有假候选");
            assert_eq!(total.unrecoverable, 0, "S={slots_per_region} 按盘生效不许不可恢复");
            assert_eq!(total.retention_violations, 0, "S={slots_per_region} 按盘取上限不许越过用户口径");
        }
    }

    #[test]
    fn floor_per_disk_is_clean_in_fault_worlds() {
        for world in [World::TornNewest, World::DiskLoss, World::SlotWriteFail, World::AdminRollback] {
            for slots_per_region in [4u64, 16] {
                for lost_disk in 0..DISK_COUNT {
                    let total = (0..6).fold(Metrics::fresh(), |mut sum, seed| {
                        sum.add(&run_world(Arm::FloorPerDisk, world, slots_per_region, seed, lost_disk));
                        sum
                    });
                    assert_eq!(total.fake_candidates, 0, "{} S={slots_per_region} 丢盘 {lost_disk} 不许有假候选", world.name());
                    assert_eq!(total.unrecoverable, 0, "{} S={slots_per_region} 丢盘 {lost_disk} 不许不可恢复", world.name());
                    assert_eq!(total.retention_violations, 0, "{} S={slots_per_region} 丢盘 {lost_disk} 不许违反最少保留", world.name());
                }
            }
        }
    }

    #[test]
    fn ring_arm_meets_both_space_requirements_near_full() {
        for seed in 0..3 {
            let total = run_near_full(Arm::RingPersisted, 4, 5, seed, None);
            assert_eq!(total.false_enospc_guaranteed, 0, "种子 {seed}：df_guaranteed ≥ 8 时写必须成功");
            assert_eq!(total.write_after_delete_failed, 0, "种子 {seed}：删一个对象之后同样大小的写必须成功");
            assert!(total.maximum_publications_per_admission <= 12, "种子 {seed}：一次准入不超过 N = 12 次发布");
            assert_eq!(total.stalls, 0, "种子 {seed}：跑前保留池下不许卡死");
            assert!(total.true_enospc > 0, "种子 {seed}：填满阶段必须真的走到 ENOSPC，否则上面几条什么都没测");
        }
    }

    /// 判据 5 的观测：臂 G 在空发布开销 5 块时，填满之后删一个对象，下一次写仍报 ENOSPC（df 也报 < 8，不是假性）。
    #[test]
    fn floor_per_disk_fails_write_after_delete_near_full_at_cost_five() {
        let total = run_near_full(Arm::FloorPerDisk, 4, 5, 0, None);
        assert!(total.write_after_delete_failed > 0, "S=4 c=5 种子 0：删了之后那次写失败（判据 5 判臂 G 出局的那一格）");
        assert_eq!(total.false_enospc_guaranteed, 0, "失败时 df_guaranteed 也报 < 8");
        assert!(total.maximum_publications_per_admission <= FLOOR_PER_DISK_PUBLICATION_BOUND, "没有越过界 7");
        assert_eq!(total.retention_violations, 0, "近满盘上也不许违反最少保留");
    }

    /// 变异 M6 的敏感取样点：新 F 只写在一块盘上时崩溃（两块盘都在），恢复后的生效值必须还是旧的。
    /// 这一轮的世界里没有「活化区间内崩溃、不丢盘」这一格，所以只能在这里钉。
    #[test]
    fn recovery_does_not_activate_floor_carried_on_only_one_disk() {
        let mut simulation = activation_fault_prefix(Arm::FloorPerDisk, 4, 0).expect("steady 前缀不该卡死");
        let new_floor = simulation.floor_carried;
        let old_active = simulation.floor_active;
        assert!(new_floor > old_active, "前缀要真的抬了 F");
        assert_eq!(simulation.run_user_window(true, false), WindowOutcome::Created);
        assert!(simulation.pending_floor.is_some(), "只写了一个带新 F 的根，不该已经生效");
        assert!(!simulation.floor_written_on_every_surviving_disk(new_floor), "前提：新 F 只在一块盘上");
        simulation.crash_and_recover();
        assert!(simulation.floor_active < new_floor, "恢复后生效值取各盘所带 F 的最小值，只在一块盘上的新 F 不许生效");
        assert!(simulation.pending_floor.is_some(), "新 F 还要照生效条件继续等");
    }

    #[test]
    fn raw_df_detector_fires_for_ring_arm_near_full() {
        let total = run_near_full(Arm::RingPersisted, 4, 5, 0, None);
        assert!(total.false_enospc_raw > 0, "df_raw 把保留池算成空闲，臂 A 报 ENOSPC 时它必须 ≥ 8（检测器的阳性对照）");
    }
}
