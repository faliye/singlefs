//! E135：动态回退下界 F —— D16（发布语义） 已定项 1 的第五条臂。
//!
//! 预注册 `research/prompts/e135-preregistration.md`（含写装置时的「跑前补登」五条，都在第一次跑之前写下）。
//!
//! 块级计数模型：每个块记（分配代，释放代）；txg 为 T 的根引用块 b ⟺ 分配代 ≤ T < 释放代。
//! 复用一个已释放的块，就把它上一段 [分配代, 释放代) 里的根记成「引用了被改写的内容」。
//! 候选集里任何一个根落在这样的区间里，就是一个假候选（D22（单元原子性怎么合成）「它们作为回退候选是假的」）。
//!
//! ## 判据（跑前写死，全文在预注册里）
//!
//! 1. 跨装置闸（E123）：臂 A 在 steady 下发布之前扣 N × 13 块、发布刚完成扣 (N − 1) × 13 块。
//! 2. 跨装置闸（E92）：臂 C 在 steady 下假候选发布之前 N − 3、发布之后 N − 4（不小于 0）。
//! 3. 判别力：臂 B 在 slot_write_fail、臂 E 在 disk0_loss、臂 F 在 admin_rollback，合计假候选 > 0。
//! 4. 正确性：臂 A、臂 D 在 rollback_first_publish 以外的世界里假候选恒 0。
//! 5. 腾出量：臂 D 抬 F 之后第 K_min + 1 次发布时生效，腾出 max(0, N − 7) × 13 块。
//! 6. 盘满：near_full 下报假性 ENOSPC、卡死、可退代数；臂 D 卡死多于臂 A 则 D 在这一格输一次。
//! 7. C281：rollback_first_publish 里各臂的不可恢复数，单独报。

use e7_index_bench::Emitter;
use std::collections::{HashSet, VecDeque};

/// R = 3（D22（单元原子性怎么合成） 已定项 2）。
const REGION_COUNT: u64 = 3;
/// K_min = max(崩溃恢复 2, 两盘掉盘 0 最坏回退 2 + 1)（C222 逐字）。
const MINIMUM_PROTECTED_GENERATIONS: u64 = 3;
/// 一次非空发布的 13 块（E123 的 BLOCKS_PER_FSYNC）拆成用户对象 8 块 + 元数据 5 块（跑前补登第 5 条）。
const USER_BLOCKS_PER_OBJECT: usize = 8;
const METADATA_BLOCKS_PER_PUBLICATION: u64 = 5;
const BLOCKS_PER_PUBLICATION: u64 = USER_BLOCKS_PER_OBJECT as u64 + METADATA_BLOCKS_PER_PUBLICATION;
/// 保留池 = 一次非空发布的开销，只有发布看得见它（D23「死锁 2」一行的破法）。
const RESERVE_BLOCKS: u64 = BLOCKS_PER_PUBLICATION;
const CAPACITY_BLOCKS: usize = 4000;
const STEADY_FILL: f64 = 0.70;
const NEAR_FULL_FILL: f64 = 0.97;
const PUBLICATIONS_PER_WORLD: u64 = 400;
const SEED_COUNT: u64 = 24;
const SLOTS_PER_REGION_SWEEP: [u64; 3] = [1, 4, 16];
const EMPTY_PUBLICATION_COST_SWEEP: [u64; 2] = [1, 5];
/// 快照留多少份：要大于最大的根环深度 48，回退与崩溃恢复才找得到所选根那一刻的分配状态。
const SNAPSHOTS_KEPT: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    RingPersisted,
    RingArithmetic,
    ProtectedGenerationsAtMinimum,
    Floor,
    FloorNoDelay,
    FloorChosenRoot,
}

const ARMS: [Arm; 6] = [Arm::RingPersisted, Arm::RingArithmetic, Arm::ProtectedGenerationsAtMinimum, Arm::Floor, Arm::FloorNoDelay, Arm::FloorChosenRoot];

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::RingPersisted => "A_ring_persisted",
            Arm::RingArithmetic => "B_ring_arithmetic",
            Arm::ProtectedGenerationsAtMinimum => "C_k_runtime",
            Arm::Floor => "D_floor",
            Arm::FloorNoDelay => "E_floor_no_delay",
            Arm::FloorChosenRoot => "F_floor_chosen_root",
        }
    }
    fn uses_floor(self) -> bool {
        match self {
            Arm::RingPersisted | Arm::RingArithmetic | Arm::ProtectedGenerationsAtMinimum => false,
            Arm::Floor | Arm::FloorNoDelay | Arm::FloorChosenRoot => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum World {
    Steady,
    TornNewest,
    Disk0Loss,
    SlotWriteFail,
    AdminRollback,
    NearFull,
    RollbackFirstPublish,
}

const WORLDS: [World; 7] = [World::Steady, World::TornNewest, World::Disk0Loss, World::SlotWriteFail, World::AdminRollback, World::NearFull, World::RollbackFirstPublish];

impl World {
    fn name(self) -> &'static str {
        match self {
            World::Steady => "steady",
            World::TornNewest => "torn_newest",
            World::Disk0Loss => "disk0_loss",
            World::SlotWriteFail => "slot_write_fail",
            World::AdminRollback => "admin_rollback",
            World::NearFull => "near_full",
            World::RollbackFirstPublish => "rollback_first_publish",
        }
    }
    /// 跑前补登第 1 条：这四个世界给 F 臂在故障之前安排一次强制盘紧。
    fn forces_pressure_before_event(self) -> bool {
        match self {
            World::TornNewest | World::Disk0Loss | World::SlotWriteFail | World::AdminRollback => true,
            World::Steady | World::NearFull | World::RollbackFirstPublish => false,
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
}

#[derive(Clone, Copy, Default, Debug)]
struct Metrics {
    fake_candidates: u64,
    checks: u64,
    false_enospc: u64,
    true_enospc: u64,
    stalls: u64,
    forced_publications: u64,
    minimum_candidates: u64,
    candidate_sum: u64,
    unrecoverable: u64,
    /// near_full 开始计数那一刻实际达到的占用块数（有的臂到不了 97%，那一截就是窗口扣住的代价）
    live_at_start: u64,
    /// 补满阶段就卡死停下、计数阶段一个窗口都没跑的种子数（不带进来，那一格会读成「0 次卡死」）
    halted_before_counting: u64,
}

struct Simulation {
    arm: Arm,
    slots_per_region: u64,
    ring_depth: u64,
    empty_publication_cost: u64,
    random_state: u64,
    pool: PoolState,
    damaged_txgs: HashSet<u64>,
    ring: Vec<Option<RootRecord>>,
    disk0_lost: bool,
    instance: u64,
    instance_rows: Vec<(u64, u64)>,
    newest_durable_txg: u64,
    maximum_published_txg: u64,
    next_txg: u64,
    floor_carried: u64,
    floor_active: u64,
    pending_floor: Option<PendingFloor>,
    snapshots: VecDeque<((u64, u64), PoolState)>,
    keep_snapshots: bool,
    tear_next_publication: bool,
    fail_next_slot_write: bool,
    halted: bool,
    metrics: Metrics,
    /// 管理员回退之前盘上那一份分配状态：被抛弃时间线的根引用的是它。回退把分配器换成所选根那一份，
    /// 那一份看不见被抛弃时间线后来写下的内容；此后每次复用都要照这一份补记损坏，
    /// 否则「回退行没写、被抛弃的根仍然有效」这类错装置看不见（变异 M8 第一轮漏抓的就是它）。
    abandoned_view: Option<(PoolState, u64)>,
}

impl Simulation {
    fn new(arm: Arm, slots_per_region: u64, empty_publication_cost: u64, seed: u64, keep_snapshots: bool) -> Self {
        let ring_depth = REGION_COUNT * slots_per_region;
        let seed_root = RootRecord { txg: 0, instance: 1, floor: 0, is_self_verified: true };
        Simulation {
            arm,
            slots_per_region,
            ring_depth,
            empty_publication_cost,
            random_state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
            pool: PoolState { blocks: vec![BlockState::Unused; CAPACITY_BLOCKS], objects: Vec::new(), metadata: Vec::new() },
            damaged_txgs: HashSet::new(),
            // mkfs 把第 0 代种进全部槽（D22（单元原子性怎么合成） 已定项 8）
            ring: vec![Some(seed_root); ring_depth as usize],
            disk0_lost: false,
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
            halted: false,
            metrics: Metrics { minimum_candidates: u64::MAX, ..Metrics::default() },
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
    fn slot_is_on_disk0(&self, slot: usize) -> bool {
        let region = slot as u64 / self.slots_per_region;
        region != 1
    }

    fn slot_survives(&self, slot: usize) -> bool {
        !(self.disk0_lost && self.slot_is_on_disk0(slot))
    }

    fn is_instance_valid(&self, root: &RootRecord) -> bool {
        match self.instance_rows.iter().filter(|(row_instance, _)| *row_instance == root.instance).map(|(_, limit)| *limit).min() {
            None => true,
            Some(limit) => root.txg <= limit,
        }
    }

    /// 盘上还读得到、自证合法的根（不管实例表）。
    fn visible_roots(&self) -> Vec<RootRecord> {
        (0..self.ring.len())
            .filter(|slot| self.slot_survives(*slot))
            .filter_map(|slot| self.ring[slot])
            .filter(|root| root.is_self_verified)
            .collect()
    }

    fn valid_roots(&self) -> Vec<RootRecord> {
        self.visible_roots().into_iter().filter(|root| self.is_instance_valid(root)).collect()
    }

    fn oldest_persisted_valid_txg(&self) -> u64 {
        self.valid_roots().iter().map(|root| root.txg).min().unwrap_or(0)
    }

    fn newest_valid_root(&self) -> Option<RootRecord> {
        self.valid_roots().into_iter().max_by_key(|root| (root.instance, root.txg))
    }

    /// 候选集的下界：臂 D / E 取全部自证合法的根所带 F 的最大值（含被抛弃时间线的根），臂 F 只取最新有效根自己的。
    fn candidacy_floor(&self) -> u64 {
        match self.arm {
            Arm::RingPersisted | Arm::RingArithmetic | Arm::ProtectedGenerationsAtMinimum => 0,
            Arm::Floor | Arm::FloorNoDelay => self.visible_roots().iter().map(|root| root.floor).max().unwrap_or(0),
            Arm::FloorChosenRoot => self.newest_valid_root().map(|root| root.floor).unwrap_or(0),
        }
    }

    /// 可再分配的谓词：已释放 ∧ 释放代 ≤ 这个界。
    fn reuse_bound(&self) -> u64 {
        match self.arm {
            Arm::RingPersisted => self.oldest_persisted_valid_txg(),
            Arm::RingArithmetic => (self.newest_durable_txg + 1).saturating_sub(self.ring_depth),
            Arm::ProtectedGenerationsAtMinimum => (self.newest_durable_txg + 1).saturating_sub(MINIMUM_PROTECTED_GENERATIONS),
            Arm::Floor | Arm::FloorNoDelay | Arm::FloorChosenRoot => self.floor_active.max(self.oldest_persisted_valid_txg()),
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

    /// 已释放而还不可再分配的块数（判据 1 量的那个「扣住」）。
    fn pinned_count(&self) -> u64 {
        let bound = self.reuse_bound();
        self.pool.blocks.iter().filter(|state| match state {
            BlockState::Unused | BlockState::Live { .. } => false,
            BlockState::Freed { freed_at, .. } => *freed_at > bound,
        }).count() as u64
    }

    /// 用户看到的空闲（`df`）：没用过的 + 已释放的，不管扣没扣住。
    fn df_free(&self) -> u64 {
        self.pool.blocks.iter().filter(|state| match state {
            BlockState::Unused | BlockState::Freed { .. } => true,
            BlockState::Live { .. } => false,
        }).count() as u64
    }

    /// 普通分配看得见的：扣掉保留池。
    fn user_allocatable(&self) -> u64 {
        self.reusable_count().saturating_sub(RESERVE_BLOCKS)
    }

    /// 取 count 块：先取可再分配的已释放块（按下标），再取没用过的。复用一块就把它上一段区间记成损坏。
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
                // 这一块上一段被 [allocated_at, freed_at) 里的根引用；比环里最旧的还旧的根已经不在盘上，不必记。
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

    fn count_fake_candidates(&self) -> (u64, u64) {
        let floor = self.candidacy_floor();
        let candidates: Vec<RootRecord> = self.valid_roots().into_iter().filter(|root| root.txg >= floor).collect();
        let fake = candidates.iter().filter(|root| self.damaged_txgs.contains(&root.txg)).count() as u64;
        (fake, candidates.len() as u64)
    }

    fn record_check(&mut self) {
        let (fake, candidates) = self.count_fake_candidates();
        self.metrics.fake_candidates += fake;
        self.metrics.checks += 1;
        self.metrics.minimum_candidates = self.metrics.minimum_candidates.min(candidates);
        self.metrics.candidate_sum += candidates;
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

    fn live_count(&self) -> u64 {
        self.pool.blocks.iter().filter(|state| matches!(state, BlockState::Live { .. })).count() as u64
    }

    /// 预热之后按正常写路径只建不删，直到占用到 fill：走同一条准入与盘紧处置，复用照记损坏。
    /// 到不了就停——那一截就是这条臂在这个 S 上被窗口扣住的空间。返回实际达到的占用块数。
    fn fill_by_creating(&mut self, fill: f64) -> u64 {
        let target_live = (CAPACITY_BLOCKS as f64 * fill) as u64;
        for _ in 0..CAPACITY_BLOCKS {
            if self.halted || self.live_count() >= target_live {
                break;
            }
            if self.user_allocatable() < USER_BLOCKS_PER_OBJECT as u64 {
                self.respond_to_pressure();
            }
            if self.halted || self.user_allocatable() < USER_BLOCKS_PER_OBJECT as u64 {
                break;
            }
            let txg = self.next_txg;
            let old_metadata = std::mem::take(&mut self.pool.metadata);
            self.free_blocks(&old_metadata, txg);
            let blocks = self.allocate(USER_BLOCKS_PER_OBJECT as u64, txg).expect("user_allocatable 说够，allocate 却拿不到");
            let mut object = [0usize; USER_BLOCKS_PER_OBJECT];
            object.copy_from_slice(&blocks);
            self.pool.objects.push(object);
            match self.allocate(METADATA_BLOCKS_PER_PUBLICATION, txg) {
                None => {
                    self.metrics.stalls += 1;
                    self.halted = true;
                    break;
                }
                Some(metadata) => {
                    self.pool.metadata = metadata;
                    self.publish(txg);
                }
            }
        }
        self.live_count()
    }

    fn save_snapshot(&mut self) {
        if !self.keep_snapshots {
            return;
        }
        self.snapshots.push_back(((self.instance, self.newest_durable_txg), self.pool.clone()));
        while self.snapshots.len() > SNAPSHOTS_KEPT {
            self.snapshots.pop_front();
        }
    }

    fn restore_snapshot(&mut self, instance: u64, txg: u64) -> bool {
        match self.snapshots.iter().rev().find(|(key, _)| *key == (instance, txg)) {
            Some((_, state)) => {
                self.pool = state.clone();
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

    /// 发布 txg 为 t 的根。根槽写失败时 txg 推进一格再发（D23 已定项 14 的索引行），写失败的槽留着旧内容。
    fn publish(&mut self, txg: u64) {
        let slot = self.slot_index(txg);
        if self.fail_next_slot_write {
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
        self.newest_durable_txg = txg;
        self.maximum_published_txg = self.maximum_published_txg.max(txg);
        self.next_txg = txg + 1;
        self.save_snapshot();
        if let Some(mut pending) = self.pending_floor {
            let carrier = *pending.carrier_txg.get_or_insert(txg);
            let delay = match self.arm {
                Arm::FloorNoDelay => 0,
                Arm::Floor | Arm::FloorChosenRoot => MINIMUM_PROTECTED_GENERATIONS,
                Arm::RingPersisted | Arm::RingArithmetic | Arm::ProtectedGenerationsAtMinimum => 0,
            };
            if txg >= carrier + delay {
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

    /// 盘紧时各臂怎么办（预注册「臂」那张表的最后一列）。
    fn respond_to_pressure(&mut self) {
        match self.arm {
            Arm::ProtectedGenerationsAtMinimum => {}
            Arm::RingPersisted | Arm::RingArithmetic => {
                for _ in 0..=self.ring_depth {
                    if self.halted || self.user_allocatable() >= USER_BLOCKS_PER_OBJECT as u64 {
                        break;
                    }
                    if !self.empty_publication() {
                        break;
                    }
                }
            }
            Arm::Floor | Arm::FloorNoDelay | Arm::FloorChosenRoot => {
                self.raise_floor();
                for _ in 0..=(MINIMUM_PROTECTED_GENERATIONS + 1) {
                    if self.halted || self.pending_floor.is_none() {
                        break;
                    }
                    if !self.empty_publication() {
                        break;
                    }
                }
            }
        }
    }

    /// 把 F 一次抬到上限 当前 − K_min + 1；只增不减。
    fn raise_floor(&mut self) {
        let target = (self.newest_durable_txg + 1).saturating_sub(MINIMUM_PROTECTED_GENERATIONS);
        if target > self.floor_carried {
            self.floor_carried = target;
            self.pending_floor = Some(PendingFloor { value: target, carrier_txg: None });
        }
    }

    /// 一个非空窗口的前半：删一个对象、放掉上一次的元数据、建一个对象、写这一次的元数据。不发布。
    fn prepare_window(&mut self) -> Option<u64> {
        if self.user_allocatable() < USER_BLOCKS_PER_OBJECT as u64 {
            self.respond_to_pressure();
        }
        if self.halted {
            return None;
        }
        let txg = self.next_txg;
        if !self.pool.objects.is_empty() {
            let victim = usize::try_from(self.next_random() % self.pool.objects.len() as u64).expect("下标装得进 usize");
            let freed_object = self.pool.objects.swap_remove(victim);
            self.free_blocks(&freed_object, txg);
        }
        let old_metadata = std::mem::take(&mut self.pool.metadata);
        self.free_blocks(&old_metadata, txg);
        if self.user_allocatable() >= USER_BLOCKS_PER_OBJECT as u64 {
            let blocks = self.allocate(USER_BLOCKS_PER_OBJECT as u64, txg).expect("user_allocatable 说够，allocate 却拿不到");
            let mut object = [0usize; USER_BLOCKS_PER_OBJECT];
            object.copy_from_slice(&blocks);
            self.pool.objects.push(object);
        } else if self.df_free() >= USER_BLOCKS_PER_OBJECT as u64 {
            self.metrics.false_enospc += 1;
        } else {
            self.metrics.true_enospc += 1;
        }
        match self.allocate(METADATA_BLOCKS_PER_PUBLICATION, txg) {
            None => {
                self.metrics.stalls += 1;
                self.halted = true;
                None
            }
            Some(blocks) => {
                self.pool.metadata = blocks;
                Some(txg)
            }
        }
    }

    fn run_window(&mut self) {
        if let Some(txg) = self.prepare_window() {
            self.publish(txg);
        }
    }

    /// 恢复之后 F 的两个值：候选下界照臂的定义取；生效值只认「带着它的根之后已有 K_min 次持久发布」的那个。
    fn floors_after_recovery(&self, chosen: &RootRecord) -> (u64, u64) {
        let candidacy = match self.arm {
            Arm::RingPersisted | Arm::RingArithmetic | Arm::ProtectedGenerationsAtMinimum => 0,
            Arm::Floor | Arm::FloorNoDelay => self.visible_roots().iter().map(|root| root.floor).max().unwrap_or(0),
            Arm::FloorChosenRoot => chosen.floor,
        };
        let settled = match self.arm {
            Arm::RingPersisted | Arm::RingArithmetic | Arm::ProtectedGenerationsAtMinimum => 0,
            Arm::FloorNoDelay => candidacy,
            Arm::Floor | Arm::FloorChosenRoot => self.valid_roots().iter()
                .filter(|root| root.txg + MINIMUM_PROTECTED_GENERATIONS <= chosen.txg)
                .max_by_key(|root| root.txg).map(|root| root.floor).unwrap_or(0).min(candidacy),
        };
        (candidacy, settled)
    }

    fn adopt_floors(&mut self, candidacy: u64, settled: u64) {
        self.floor_carried = candidacy;
        self.floor_active = settled;
        self.pending_floor = if candidacy > settled {
            let carrier = self.valid_roots().iter().filter(|root| root.floor >= candidacy).map(|root| root.txg).min();
            Some(PendingFloor { value: candidacy, carrier_txg: carrier })
        } else {
            None
        };
    }

    /// 崩溃之后的恢复：先验全部候选再择新（D22 已定项 7），分配状态从所选根重新载入，取新实例代号。
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
        let (candidacy, settled) = match self.arm {
            Arm::FloorChosenRoot => (target.floor, target.floor.min(self.floors_after_recovery(&target).1)),
            Arm::RingPersisted | Arm::RingArithmetic | Arm::ProtectedGenerationsAtMinimum | Arm::Floor | Arm::FloorNoDelay => self.floors_after_recovery(&target),
        };
        let abandoned_pool = self.pool.clone();
        let abandoned_newest_txg = self.visible_roots().iter().map(|root| root.txg).max().unwrap_or(0);
        if !self.restore_snapshot(target.instance, target.txg) {
            return false;
        }
        self.abandoned_view = Some((abandoned_pool, abandoned_newest_txg));
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
        // 回退之后的复用照 abandoned_view 补记损坏（admin_rollback 里设好），这里只管分配
        if !self.admin_rollback() {
            return;
        }
        let txg = self.next_txg;
        let _ = self.allocate(BLOCKS_PER_PUBLICATION, txg);
        // 崩溃：回退行、新实例代号、F 的状态都没持久；分配器写下的块已经落盘
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

/// 故障落在哪个非空窗口、盘紧提前几个窗口（跑前补登第 1 条）。
fn schedule(seed: u64, ring_depth: u64, world: World) -> (u64, u64) {
    let mut state = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1;
    state ^= state >> 29;
    let event_window = 2 * ring_depth + 20 + state % 150;
    let lead = match world {
        // 盘紧那个窗口算第 1 个：回退落在第 K_min + 3 个非空窗口
        World::AdminRollback => MINIMUM_PROTECTED_GENERATIONS + 2,
        // 盘紧那个窗口算第 1 个：故障落在第 1 个或第 K_min + 1 个非空窗口
        World::TornNewest | World::Disk0Loss | World::SlotWriteFail => if seed % 2 == 0 { 0 } else { MINIMUM_PROTECTED_GENERATIONS },
        World::Steady | World::NearFull | World::RollbackFirstPublish => 0,
    };
    (event_window, lead)
}

fn run_world(arm: Arm, world: World, slots_per_region: u64, empty_publication_cost: u64, seed: u64) -> Metrics {
    let keep_snapshots = match world {
        World::TornNewest | World::Disk0Loss | World::AdminRollback | World::RollbackFirstPublish => true,
        World::Steady | World::SlotWriteFail | World::NearFull => false,
    };
    let mut simulation = Simulation::new(arm, slots_per_region, empty_publication_cost, seed, keep_snapshots);
    simulation.prefill(STEADY_FILL);
    let (event_window, lead) = schedule(seed, simulation.ring_depth, world);
    let mut first_counted_window = 0u64;
    if world == World::NearFull {
        let warm_up = 2 * simulation.ring_depth + 10;
        for _ in 0..warm_up {
            simulation.run_window();
        }
        let achieved_live = simulation.fill_by_creating(NEAR_FULL_FILL);
        // 补满阶段的卡死要带进计数：清零时把它抹掉，那一格就会被读成「0 次卡死」
        let stalls_while_filling = simulation.metrics.stalls;
        simulation.metrics = Metrics {
            minimum_candidates: u64::MAX,
            live_at_start: achieved_live,
            stalls: stalls_while_filling,
            halted_before_counting: u64::from(simulation.halted),
            ..Metrics::default()
        };
        first_counted_window = warm_up;
    }
    for window in first_counted_window..PUBLICATIONS_PER_WORLD {
        if simulation.halted {
            break;
        }
        if world.forces_pressure_before_event() && arm.uses_floor() && window + lead == event_window {
            simulation.respond_to_pressure();
        }
        if window == event_window {
            match world {
                World::TornNewest => simulation.tear_next_publication = true,
                World::SlotWriteFail => simulation.fail_next_slot_write = true,
                World::AdminRollback => {
                    simulation.admin_rollback();
                }
                World::RollbackFirstPublish => {
                    simulation.admin_rollback_then_crash_before_root();
                }
                World::Steady | World::Disk0Loss | World::NearFull => {}
            }
        }
        simulation.run_window();
        if world == World::Disk0Loss && window == event_window && !simulation.halted {
            simulation.disk0_lost = true;
            simulation.crash_and_recover();
            simulation.halted = true;
        }
    }
    simulation.metrics
}

/// 判据 1 / 2：steady 下最后一个窗口，在发布之前与发布刚完成各量一次。
fn steady_probe(arm: Arm, slots_per_region: u64) -> (u64, u64, u64, u64) {
    let mut simulation = Simulation::new(arm, slots_per_region, 1, 1, false);
    simulation.prefill(STEADY_FILL);
    for _ in 0..(PUBLICATIONS_PER_WORLD - 1) {
        simulation.run_window();
    }
    let txg = simulation.prepare_window().expect("steady 下不该卡死");
    let pinned_before = simulation.pinned_count();
    let fake_before = simulation.count_fake_candidates().0;
    simulation.publish(txg);
    let pinned_after = simulation.pinned_count();
    let fake_after = simulation.count_fake_candidates().0;
    (pinned_before, pinned_after, fake_before, fake_after)
}

/// 判据 5：臂 D 在 steady 下抬一次 F，数到生效要几次发布、生效那一刻多放出几块。
fn reclaim_probe(slots_per_region: u64) -> (u64, u64) {
    let mut simulation = Simulation::new(Arm::Floor, slots_per_region, 1, 1, false);
    simulation.prefill(STEADY_FILL);
    for _ in 0..200 {
        simulation.run_window();
    }
    simulation.raise_floor();
    let raised_to = simulation.floor_carried;
    let mut publications = 0u64;
    while simulation.pending_floor.is_some() {
        let before = simulation.floor_active;
        assert!(simulation.empty_publication(), "steady 下空发布不该卡死");
        publications += 1;
        assert!(publications <= MINIMUM_PROTECTED_GENERATIONS + 2, "F 迟迟不生效");
        if simulation.floor_active != before {
            break;
        }
    }
    let oldest = simulation.oldest_persisted_valid_txg();
    let freed_by_floor = simulation.pool.blocks.iter().filter(|state| matches!(state,
        BlockState::Freed { freed_at, .. } if *freed_at > oldest && *freed_at <= raised_to)).count() as u64;
    (publications, freed_by_floor)
}

fn expected_reclaim_blocks(slots_per_region: u64) -> u64 {
    (REGION_COUNT * slots_per_region).saturating_sub(7) * BLOCKS_PER_PUBLICATION
}

fn sum_metrics(arm: Arm, world: World, slots_per_region: u64, empty_publication_cost: u64) -> Metrics {
    let mut total = Metrics { minimum_candidates: u64::MAX, ..Metrics::default() };
    for seed in 0..SEED_COUNT {
        let run = run_world(arm, world, slots_per_region, empty_publication_cost, seed);
        total.fake_candidates += run.fake_candidates;
        total.checks += run.checks;
        total.false_enospc += run.false_enospc;
        total.true_enospc += run.true_enospc;
        total.stalls += run.stalls;
        total.forced_publications += run.forced_publications;
        total.minimum_candidates = total.minimum_candidates.min(run.minimum_candidates);
        total.candidate_sum += run.candidate_sum;
        total.unrecoverable += run.unrecoverable;
        total.live_at_start += run.live_at_start;
        total.halted_before_counting += run.halted_before_counting;
    }
    total
}

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!(
        "name=config r={REGION_COUNT} k_min={MINIMUM_PROTECTED_GENERATIONS} blocks_per_publication={BLOCKS_PER_PUBLICATION} \
         user_blocks={USER_BLOCKS_PER_OBJECT} metadata_blocks={METADATA_BLOCKS_PER_PUBLICATION} reserve={RESERVE_BLOCKS} \
         capacity={CAPACITY_BLOCKS} publications={PUBLICATIONS_PER_WORLD} seeds={SEED_COUNT}"
    )));

    let mut criterion_one_holds = true;
    let mut criterion_two_holds = true;
    for slots_per_region in SLOTS_PER_REGION_SWEEP {
        let ring_depth = REGION_COUNT * slots_per_region;
        let (pinned_before, pinned_after, _, _) = steady_probe(Arm::RingPersisted, slots_per_region);
        let expect_before = ring_depth * BLOCKS_PER_PUBLICATION;
        let expect_after = (ring_depth - 1) * BLOCKS_PER_PUBLICATION;
        criterion_one_holds &= pinned_before == expect_before && pinned_after == expect_after;
        println!("{}", emitter.emit_raw(&format!(
            "name=crit1_pinned arm=A s={slots_per_region} ring_depth={ring_depth} pinned_before={pinned_before} expect_before={expect_before} \
             pinned_after={pinned_after} expect_after={expect_after}"
        )));
        let (_, _, fake_before, fake_after) = steady_probe(Arm::ProtectedGenerationsAtMinimum, slots_per_region);
        let expect_fake_before = ring_depth - MINIMUM_PROTECTED_GENERATIONS;
        let expect_fake_after = ring_depth.saturating_sub(MINIMUM_PROTECTED_GENERATIONS + 1);
        criterion_two_holds &= fake_before == expect_fake_before && fake_after == expect_fake_after;
        println!("{}", emitter.emit_raw(&format!(
            "name=crit2_fake arm=C s={slots_per_region} fake_before={fake_before} expect_before={expect_fake_before} \
             fake_after={fake_after} expect_after={expect_fake_after}"
        )));
    }

    let mut criterion_five_holds = true;
    for slots_per_region in SLOTS_PER_REGION_SWEEP {
        let (publications, freed) = reclaim_probe(slots_per_region);
        let expect = expected_reclaim_blocks(slots_per_region);
        criterion_five_holds &= publications == MINIMUM_PROTECTED_GENERATIONS + 1 && freed == expect;
        println!("{}", emitter.emit_raw(&format!(
            "name=crit5_reclaim arm=D s={slots_per_region} publications_to_active={publications} expect_publications={} \
             reclaimed={freed} expect_reclaimed={expect}",
            MINIMUM_PROTECTED_GENERATIONS + 1
        )));
    }

    let mut target_hits = (0u64, 0u64, 0u64);
    let mut correctness = (0u64, 0u64);
    let mut near_full_stalls = [(0u64, 0u64); 2];
    let mut first_publish_unrecoverable = [0u64; 6];
    for world in WORLDS {
        let costs: &[u64] = if world == World::NearFull { &EMPTY_PUBLICATION_COST_SWEEP } else { &EMPTY_PUBLICATION_COST_SWEEP[..1] };
        for &empty_publication_cost in costs {
            for slots_per_region in SLOTS_PER_REGION_SWEEP {
                for (arm_index, arm) in ARMS.iter().enumerate() {
                    let total = sum_metrics(*arm, world, slots_per_region, empty_publication_cost);
                    let average_candidates_in_hundredths = if total.checks == 0 { 0 } else { total.candidate_sum * 100 / total.checks };
                    let minimum_candidates = if total.minimum_candidates == u64::MAX { 0 } else { total.minimum_candidates };
                    println!("{}", emitter.emit_raw(&format!(
                        "name=cell arm={} world={} s={slots_per_region} empty_cost={empty_publication_cost} fakes={} checks={} \
                         false_enospc={} true_enospc={} stalls={} forced_pubs={} min_candidates={minimum_candidates} \
                         avg_candidates_x100={average_candidates_in_hundredths} unrecoverable={} live_at_start_avg={} halted_before_counting={}",
                        arm.name(), world.name(), total.fake_candidates, total.checks, total.false_enospc, total.true_enospc,
                        total.stalls, total.forced_publications, total.unrecoverable, total.live_at_start / SEED_COUNT,
                        total.halted_before_counting
                    )));
                    match (*arm, world) {
                        (Arm::RingArithmetic, World::SlotWriteFail) => target_hits.0 += total.fake_candidates,
                        (Arm::FloorNoDelay, World::Disk0Loss) => target_hits.1 += total.fake_candidates,
                        (Arm::FloorChosenRoot, World::AdminRollback) => target_hits.2 += total.fake_candidates,
                        _ => {}
                    }
                    if world != World::RollbackFirstPublish {
                        match arm {
                            Arm::RingPersisted => correctness.0 += total.fake_candidates,
                            Arm::Floor => correctness.1 += total.fake_candidates,
                            Arm::RingArithmetic | Arm::ProtectedGenerationsAtMinimum | Arm::FloorNoDelay | Arm::FloorChosenRoot => {}
                        }
                    }
                    if world == World::NearFull && empty_publication_cost == EMPTY_PUBLICATION_COST_SWEEP[0] {
                        match arm {
                            Arm::RingPersisted => near_full_stalls[0].0 += total.stalls,
                            Arm::Floor => near_full_stalls[0].1 += total.stalls,
                            Arm::RingArithmetic | Arm::ProtectedGenerationsAtMinimum | Arm::FloorNoDelay | Arm::FloorChosenRoot => {}
                        }
                    }
                    if world == World::NearFull && empty_publication_cost == EMPTY_PUBLICATION_COST_SWEEP[1] {
                        match arm {
                            Arm::RingPersisted => near_full_stalls[1].0 += total.stalls,
                            Arm::Floor => near_full_stalls[1].1 += total.stalls,
                            Arm::RingArithmetic | Arm::ProtectedGenerationsAtMinimum | Arm::FloorNoDelay | Arm::FloorChosenRoot => {}
                        }
                    }
                    if world == World::RollbackFirstPublish {
                        first_publish_unrecoverable[arm_index] += total.unrecoverable;
                    }
                }
            }
        }
    }

    println!("{}", emitter.emit_raw(&format!(
        "name=verdict crit1_holds={} crit2_holds={} crit5_holds={} crit3_b_slot_write_fail={} crit3_e_disk0_loss={} \
         crit3_f_admin_rollback={} crit4_a_fakes={} crit4_d_fakes={} crit6_stalls_a_cost1={} crit6_stalls_d_cost1={} \
         crit6_stalls_a_cost5={} crit6_stalls_d_cost5={}",
        u8::from(criterion_one_holds), u8::from(criterion_two_holds), u8::from(criterion_five_holds),
        target_hits.0, target_hits.1, target_hits.2, correctness.0, correctness.1,
        near_full_stalls[0].0, near_full_stalls[0].1, near_full_stalls[1].0, near_full_stalls[1].1
    )));
    println!("{}", emitter.emit_raw(&format!(
        "name=crit7_c281 a={} b={} c={} d={} e={} f={}",
        first_publish_unrecoverable[0], first_publish_unrecoverable[1], first_publish_unrecoverable[2],
        first_publish_unrecoverable[3], first_publish_unrecoverable[4], first_publish_unrecoverable[5]
    )));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_index_rotates_across_regions_then_within_region() {
        let simulation = Simulation::new(Arm::RingPersisted, 2, 1, 1, false);
        let slots: Vec<usize> = (0..7).map(|txg| simulation.slot_index(txg)).collect();
        assert_eq!(slots, vec![0, 2, 4, 1, 3, 5, 0], "区域 = txg mod 3、区内槽 = ⌊txg / 3⌋ mod S");
    }

    #[test]
    fn instance_row_invalidates_only_later_roots_of_that_instance() {
        let mut simulation = Simulation::new(Arm::RingPersisted, 1, 1, 1, false);
        simulation.instance_rows.push((1, 10));
        let kept = RootRecord { txg: 10, instance: 1, floor: 0, is_self_verified: true };
        let dropped = RootRecord { txg: 11, instance: 1, floor: 0, is_self_verified: true };
        let other = RootRecord { txg: 99, instance: 2, floor: 0, is_self_verified: true };
        assert!(simulation.is_instance_valid(&kept), "T ≤ Ti 的根有效");
        assert!(!simulation.is_instance_valid(&dropped), "T > Ti 的根无效");
        assert!(simulation.is_instance_valid(&other), "没有行的实例全部有效");
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
    fn criterion_two_fake_candidates_match_e92_both_readings() {
        for (slots_per_region, before, after) in [(1u64, 0u64, 0u64), (4, 9, 8), (16, 45, 44)] {
            let (_, _, fake_before, fake_after) = steady_probe(Arm::ProtectedGenerationsAtMinimum, slots_per_region);
            assert_eq!(fake_before, before, "S={slots_per_region} 发布之前假候选应为 N − K");
            assert_eq!(fake_after, after, "S={slots_per_region} 发布之后假候选应为 N − K − 1");
        }
    }

    #[test]
    fn criterion_five_reclaim_after_minimum_protected_generations_plus_one_publications() {
        for (slots_per_region, reclaimed) in [(1u64, 0u64), (4, 65), (16, 533)] {
            let (publications, freed) = reclaim_probe(slots_per_region);
            assert_eq!(publications, MINIMUM_PROTECTED_GENERATIONS + 1, "S={slots_per_region} 抬 F 之后第 K_min + 1 次发布生效");
            assert_eq!(freed, reclaimed, "S={slots_per_region} 生效时多放出 max(0, N − 7) × 13 块");
        }
    }

    #[test]
    fn arithmetic_oldest_root_produces_fake_candidates_after_slot_write_failure() {
        let total: u64 = (0..8).map(|seed| run_world(Arm::RingArithmetic, World::SlotWriteFail, 4, 1, seed).fake_candidates).sum();
        assert!(total > 0, "按算术取最旧根，根槽写失败之后必须冒出假候选（判据 3 的靶子）");
    }

    #[test]
    fn persisted_oldest_root_has_no_fake_candidates_after_slot_write_failure() {
        let total: u64 = (0..8).map(|seed| run_world(Arm::RingPersisted, World::SlotWriteFail, 4, 1, seed).fake_candidates).sum();
        assert_eq!(total, 0, "按盘上内容取最旧根不许有假候选");
    }

    #[test]
    fn floor_without_activation_delay_produces_fake_candidates_after_disk_loss() {
        let total: u64 = [4u64, 16].iter().flat_map(|slots| (0..SEED_COUNT).map(move |seed| (*slots, seed)))
            .map(|(slots, seed)| run_world(Arm::FloorNoDelay, World::Disk0Loss, slots, 1, seed).fake_candidates).sum();
        assert!(total > 0, "去掉 K_min 的活化延迟，掉盘之后必须冒出假候选（判据 3 的靶子）");
    }

    #[test]
    fn floor_taken_from_chosen_root_produces_fake_candidates_after_admin_rollback() {
        let total: u64 = [4u64, 16].iter().flat_map(|slots| (0..SEED_COUNT).map(move |seed| (*slots, seed)))
            .map(|(slots, seed)| run_world(Arm::FloorChosenRoot, World::AdminRollback, slots, 1, seed).fake_candidates).sum();
        assert!(total > 0, "回退之后只认所选根自己的 F，必须冒出假候选（判据 3 的靶子）");
    }

    #[test]
    fn floor_arm_has_no_fake_candidates_in_failure_worlds() {
        for world in [World::TornNewest, World::Disk0Loss, World::SlotWriteFail, World::AdminRollback] {
            // S = 4 时 F 能抬的余量只有一两代，变异可能恰好同值；S = 16 是敏感的取样点（mutation-sampling.md 第三类）
            let total: u64 = [4u64, 16].iter().flat_map(|slots| (0..8).map(move |seed| (*slots, seed)))
                .map(|(slots, seed)| run_world(Arm::Floor, world, slots, 1, seed).fake_candidates).sum();
            assert_eq!(total, 0, "臂 D 在 {} 里不许有假候选", world.name());
        }
    }

    #[test]
    fn ring_arm_has_no_fake_candidates_after_admin_rollback() {
        let total: u64 = [4u64, 16].iter().flat_map(|slots| (0..8).map(move |seed| (*slots, seed)))
            .map(|(slots, seed)| run_world(Arm::RingPersisted, World::AdminRollback, slots, 1, seed).fake_candidates).sum();
        assert_eq!(total, 0, "写了回退行，被抛弃的根就不在候选集里，新时间线复用它们的块不算假候选");
    }

    #[test]
    fn rollback_first_publication_crash_is_unrecoverable_for_ring_arm() {
        let total: u64 = (0..8).map(|seed| run_world(Arm::RingPersisted, World::RollbackFirstPublish, 4, 1, seed).unrecoverable).sum();
        assert!(total > 0, "C281：回退那次发布根槽落盘之前崩溃，恢复挑中被抛弃的根，它引用的块已被改写");
    }
}
