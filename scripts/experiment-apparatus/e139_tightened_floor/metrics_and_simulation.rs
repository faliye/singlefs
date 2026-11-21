//! 每次模拟的度量与整个模拟器（池、写入窗口、回收、准入、挂载与崩溃的步进）。

use e7_index_bench::truncate_f64_to_usize;
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct Metrics {
    pub(super) trajectory: Trajectory,
    pub(super) fake_candidates: u64,
    pub(super) unrecoverable: u64,
    pub(super) retention_violations: u64,
    pub(super) checks: u64,
    pub(super) minimum_candidates: u64,
    pub(super) candidate_sum: u64,
    pub(super) nonempty_candidate_sum: u64,
    pub(super) minimum_distinct_states: u64,
    pub(super) distinct_state_sum: u64,
    pub(super) stalls: u64,
    pub(super) forced_publications: u64,
    pub(super) user_windows: u64,
    pub(super) admissions_under_pressure: u64,
    pub(super) maximum_publications_per_admission: u64,
    pub(super) false_enospc_raw: u64,
    pub(super) false_enospc_arm: u64,
    pub(super) true_enospc: u64,
    pub(super) write_after_delete_failed: u64,
    pub(super) first_write_after_delete_failure_window: u64,
    pub(super) last_write_after_delete_failure_window: u64,
    pub(super) halted_before_counting: u64,
    pub(super) root_write_failures_injected: u64,
    pub(super) lag_sum: u64,
    pub(super) lag_maximum: u64,
    pub(super) lag_samples: u64,
    /// 第 2 次重跑的新量；第 1 次的每一行都不读它，也不打印它。
    pub(super) second_rerun: SecondRerunObservations,
    /// 第 3 次重跑的新量；第 1、2 次的每一行都不读它，也不打印它。
    pub(super) core_gate: CoreGateObservations,
}

impl Metrics {
    pub(super) fn fresh() -> Self {
        Metrics {
            trajectory: Trajectory::fresh(),
            fake_candidates: 0,
            unrecoverable: 0,
            retention_violations: 0,
            checks: 0,
            minimum_candidates: u64::MAX,
            candidate_sum: 0,
            nonempty_candidate_sum: 0,
            minimum_distinct_states: u64::MAX,
            distinct_state_sum: 0,
            stalls: 0,
            forced_publications: 0,
            user_windows: 0,
            admissions_under_pressure: 0,
            maximum_publications_per_admission: 0,
            false_enospc_raw: 0,
            false_enospc_arm: 0,
            true_enospc: 0,
            write_after_delete_failed: 0,
            first_write_after_delete_failure_window: u64::MAX,
            last_write_after_delete_failure_window: 0,
            halted_before_counting: 0,
            root_write_failures_injected: 0,
            lag_sum: 0,
            lag_maximum: 0,
            lag_samples: 0,
            second_rerun: SecondRerunObservations::fresh(),
            core_gate: CoreGateObservations::fresh(),
        }
    }
    pub(super) fn add(&mut self, other: &Metrics) {
        self.trajectory.add(&other.trajectory);
        self.fake_candidates += other.fake_candidates;
        self.unrecoverable += other.unrecoverable;
        self.retention_violations += other.retention_violations;
        self.checks += other.checks;
        self.minimum_candidates = self.minimum_candidates.min(other.minimum_candidates);
        self.candidate_sum += other.candidate_sum;
        self.nonempty_candidate_sum += other.nonempty_candidate_sum;
        self.minimum_distinct_states = self.minimum_distinct_states.min(other.minimum_distinct_states);
        self.distinct_state_sum += other.distinct_state_sum;
        self.stalls += other.stalls;
        self.forced_publications += other.forced_publications;
        self.user_windows += other.user_windows;
        self.admissions_under_pressure += other.admissions_under_pressure;
        self.maximum_publications_per_admission = self.maximum_publications_per_admission.max(other.maximum_publications_per_admission);
        self.false_enospc_raw += other.false_enospc_raw;
        self.false_enospc_arm += other.false_enospc_arm;
        self.true_enospc += other.true_enospc;
        self.write_after_delete_failed += other.write_after_delete_failed;
        self.first_write_after_delete_failure_window = self.first_write_after_delete_failure_window.min(other.first_write_after_delete_failure_window);
        self.last_write_after_delete_failure_window = self.last_write_after_delete_failure_window.max(other.last_write_after_delete_failure_window);
        self.halted_before_counting += other.halted_before_counting;
        self.root_write_failures_injected += other.root_write_failures_injected;
        self.lag_sum += other.lag_sum;
        self.lag_maximum = self.lag_maximum.max(other.lag_maximum);
        self.lag_samples += other.lag_samples;
        self.second_rerun.add(&other.second_rerun);
        self.core_gate.add(&other.core_gate);
    }
    pub(super) fn minimum_candidates_or_zero(&self) -> u64 {
        if self.minimum_candidates == u64::MAX { 0 } else { self.minimum_candidates }
    }
    pub(super) fn minimum_distinct_states_or_zero(&self) -> u64 {
        if self.minimum_distinct_states == u64::MAX { 0 } else { self.minimum_distinct_states }
    }
    pub(super) fn average_candidates_in_hundredths(&self) -> u64 {
        (self.candidate_sum * 100).checked_div(self.checks).unwrap_or(0)
    }
    pub(super) fn average_nonempty_candidates_in_hundredths(&self) -> u64 {
        (self.nonempty_candidate_sum * 100).checked_div(self.checks).unwrap_or(0)
    }
    pub(super) fn average_distinct_states_in_hundredths(&self) -> u64 {
        (self.distinct_state_sum * 100).checked_div(self.checks).unwrap_or(0)
    }
    pub(super) fn forced_publications_per_window_in_hundredths(&self) -> u64 {
        (self.forced_publications * 100).checked_div(self.user_windows).unwrap_or(0)
    }
    pub(super) fn average_lag_in_hundredths(&self) -> u64 {
        (self.lag_sum * 100).checked_div(self.lag_samples).unwrap_or(0)
    }
    pub(super) fn first_failure_window_or_zero(&self) -> u64 {
        if self.first_write_after_delete_failure_window == u64::MAX { 0 } else { self.first_write_after_delete_failure_window }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum WindowOutcome {
    Created,
    AllocatedWithoutPublish,
    NoSpace,
    Halted,
}

#[derive(Clone)]
pub(super) struct Simulation {
    pub(super) arm: Arm,
    slots_per_region: u64,
    pub(super) ring_depth: u64,
    /// 现在每次空发布写几块（cost_growth 世界里会变）。
    pub(super) empty_publication_cost: u64,
    /// 保留池与残留按这条臂的式子算（被测臂按声明的最坏值 c_max，见 Arm::formula_cost）。
    reserve_blocks: u64,
    push_residue_blocks: u64,
    random_state: u64,
    pub(super) pool: PoolState,
    pub(super) damaged_txgs: HashSet<u64>,
    pub(super) ring: Vec<Option<RootRecord>>,
    pub(super) disk_lost: [bool; DISK_COUNT],
    pub(super) instance: u64,
    instance_rows: Vec<(u64, u64)>,
    pub(super) newest_durable_txg: u64,
    pub(super) maximum_published_txg: u64,
    pub(super) next_txg: u64,
    pub(super) floor_carried: u64,
    pub(super) floor_active: u64,
    pub(super) pending_floor: Option<PendingFloor>,
    snapshots: VecDeque<((u64, u64), Rc<PoolState>)>,
    /// 快照留多少份；默认 `SNAPSHOTS_KEPT`。`disk_stuck` 里卡住那块盘上的旧根可以比 64 次发布还老，恢复要读得到它的快照，那里放大（`STUCK_SNAPSHOTS_KEPT`）。
    pub(super) snapshots_kept_limit: usize,
    keep_snapshots: bool,
    pub(super) tear_next_publication: bool,
    pub(super) fail_next_slot_write: bool,
    /// 第几次根槽写要失败（按根槽写的绝对序号，重发也算一次）。
    pub(super) planned_root_write_failures: BTreeSet<u64>,
    pub(super) root_write_attempts: u64,
    /// broken_slot 世界：这个槽从此每次写都失败。
    pub(super) broken_slot: Option<usize>,
    /// 系统配置里记的 F（第 2 次重跑第五节「CF」）：`raise_floor_to` 真抬时先并进来，崩溃、丢盘、管理员回退都不降它；每块盘都写、不失败（登记差异 ⑨）。
    pub(super) system_configuration_floor: u64,
    /// disk_stuck 世界：这块盘上的根槽从此每次写都失败（照 `publish` 的推进一格、槽留旧根）。
    pub(super) stuck_disk: Option<usize>,
    /// 到现在为止这条臂放掉的块数（`free_blocks` 里 Live 变 Freed 的块数）；刻度之间的斜率用它。
    pub(super) blocks_freed_in_total: u64,
    /// 刻度的起点 T₀（卡住或坏槽开始那一刻的 `next_txg`）；设了才记刻度。
    pub(super) stuck_marks_origin_txg: Option<u64>,
    pub(super) current_window_changes_user_state: bool,
    /// 非空发布的 txg：空发布与推空的根不改用户可见状态。mkfs 的第 0 代算一个状态。
    nonempty_txgs: HashSet<u64>,
    /// 空发布的 txg：它们放掉的只有上一次的元数据，是推空自己的残留，不算滞后量。
    empty_txgs: HashSet<u64>,
    admin_rollback_happened: bool,
    pub(super) halted: bool,
    /// 近满盘循环里数到第几个窗口，给「删了再写失败」记窗口号。
    pub(super) counting_window: u64,
    pub(super) metrics: Metrics,
    /// 管理员回退之前盘上那一份分配状态，被抛弃时间线的根引用的是它（E135 的 abandoned_view 原样）。
    abandoned_view: Option<(Rc<PoolState>, u64)>,
    /// 初始供给吃光了没有（第一次准入不够）：轨迹的「吃光之后的峰值」从这一刻起记。
    is_supply_exhausted: bool,
    /// 第 3 次重跑：CG 臂的状态；别的臂是 `None`。
    pub(super) core_gate: Option<CoreGateState>,
}

impl Simulation {
    pub(super) fn new(arm: Arm, slots_per_region: u64, mount_cost: u64, declared_maximum_cost: u64, seed: u64, keep_snapshots: bool, reserve_override: Option<u64>) -> Self {
        let ring_depth = REGION_COUNT * slots_per_region;
        let formula_cost = arm.formula_cost(mount_cost, declared_maximum_cost);
        let seed_root = RootRecord { txg: 0, instance: 1, floor: 0, is_self_verified: true };
        let mut nonempty_txgs = HashSet::new();
        nonempty_txgs.insert(0);
        Simulation {
            arm,
            slots_per_region,
            ring_depth,
            empty_publication_cost: mount_cost,
            reserve_blocks: reserve_override.unwrap_or_else(|| arm.reserve_blocks(ring_depth, formula_cost)),
            push_residue_blocks: arm.push_residue_blocks(ring_depth, formula_cost),
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
            snapshots_kept_limit: SNAPSHOTS_KEPT,
            keep_snapshots,
            tear_next_publication: false,
            fail_next_slot_write: false,
            planned_root_write_failures: BTreeSet::new(),
            root_write_attempts: 0,
            broken_slot: None,
            system_configuration_floor: 0,
            stuck_disk: None,
            blocks_freed_in_total: 0,
            stuck_marks_origin_txg: None,
            current_window_changes_user_state: false,
            nonempty_txgs,
            empty_txgs: HashSet::new(),
            admin_rollback_happened: false,
            halted: false,
            counting_window: 0,
            metrics: Metrics::fresh(),
            abandoned_view: None,
            is_supply_exhausted: false,
            core_gate: if arm.uses_the_core_gate() { Some(CoreGateState::fresh()) } else { None },
        }
    }

    fn next_random(&mut self) -> u64 {
        self.random_state ^= self.random_state >> 12;
        self.random_state ^= self.random_state << 25;
        self.random_state ^= self.random_state >> 27;
        self.random_state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// 区域 = txg mod R，区内槽 = ⌊txg / R⌋ mod S（区内公式是 layout/01-first-txn.md 的预想）。
    pub(super) fn slot_index(&self, txg: u64) -> usize {
        let region = txg % REGION_COUNT;
        let slot_within_region = (txg / REGION_COUNT) % self.slots_per_region;
        usize::try_from(region * self.slots_per_region + slot_within_region).expect("槽号装得进 usize")
    }

    pub(super) fn region_of_txg(txg: u64) -> u64 {
        txg % REGION_COUNT
    }

    /// 区域 0、2 在盘 0，区域 1 在盘 1（layout/01-first-txn.md 的预想归属）。
    pub(super) fn disk_of_slot(&self, slot: usize) -> usize {
        let region = slot as u64 / self.slots_per_region;
        if region == 1 { 1 } else { 0 }
    }

    fn slot_survives(&self, slot: usize) -> bool {
        !self.disk_lost[self.disk_of_slot(slot)]
    }

    pub(super) fn surviving_disks(&self) -> Vec<usize> {
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

    pub(super) fn valid_roots(&self) -> Vec<RootRecord> {
        self.valid_roots_with_disk().into_iter().map(|(_, root)| root).collect()
    }

    pub(super) fn oldest_persisted_valid_txg(&self) -> u64 {
        self.valid_roots().iter().map(|root| root.txg).min().unwrap_or(0)
    }

    fn newest_valid_root(&self) -> Option<RootRecord> {
        self.valid_roots().into_iter().max_by_key(|root| (root.instance, root.txg))
    }

    pub(super) fn newest_valid_txg_on_disk(&self, disk: usize) -> Option<u64> {
        self.valid_roots_with_disk().iter().filter(|(root_disk, _)| *root_disk == disk).map(|(_, root)| root.txg).max()
    }

    fn newest_valid_txg_on_every_surviving_disk(&self) -> u64 {
        self.surviving_disks().iter().map(|disk| self.newest_valid_txg_on_disk(*disk).unwrap_or(0)).min().unwrap_or(0)
    }

    /// 按根数读的那一半上限：第 4 新的持久有效根；有效根不足 4 个时取最旧的那个。
    fn fourth_newest_valid_txg(&self) -> u64 {
        let distinct_newest_first: Vec<u64> = self.valid_roots().iter().map(|root| root.txg).collect::<BTreeSet<u64>>().into_iter().rev().collect();
        if distinct_newest_first.len() >= MINIMUM_RETAINED_ROOTS {
            distinct_newest_first[MINIMUM_RETAINED_ROOTS - 1]
        } else {
            distinct_newest_first.last().copied().unwrap_or(0)
        }
    }

    /// 按状态数读的那一半上限：第 4 新的非空持久有效根；非空有效根不足 4 个时取最旧的**有效根**（不论空不空）——
    /// 比最旧非空根更老的空根带着更老的状态，跳过它们等于丢状态。第一次运行取的是最旧的非空根，G2S 在 S = 1 的
    /// 五个故障世界里因此 41 次越过盘上最新的根（`research/results/e139-tightened-floor-2026-09-12-run1.out`）。
    pub(super) fn fourth_newest_nonempty_valid_txg(&self) -> u64 {
        let distinct_newest_first: Vec<u64> = self.valid_roots().iter().map(|root| root.txg)
            .filter(|txg| self.nonempty_txgs.contains(txg)).collect::<BTreeSet<u64>>().into_iter().rev().collect();
        if distinct_newest_first.len() >= MINIMUM_RETAINED_ROOTS {
            distinct_newest_first[MINIMUM_RETAINED_ROOTS - 1]
        } else {
            self.oldest_persisted_valid_txg()
        }
    }

    /// 上限：min(每块幸存盘上最新的持久有效根, 第 4 新的持久有效根)。
    fn per_disk_retention_upper_limit(&self) -> u64 {
        self.fourth_newest_valid_txg().min(self.newest_valid_txg_on_every_surviving_disk())
    }

    /// 「4」数不同用户可见状态的口径下的上限：min(每块幸存盘上最新的持久有效根, 第 4 新的非空持久有效根)。
    pub(super) fn per_disk_states_upper_limit(&self) -> u64 {
        self.fourth_newest_nonempty_valid_txg().min(self.newest_valid_txg_on_every_surviving_disk())
    }

    pub(super) fn window_arm(&self) -> Option<WindowArm> {
        match self.arm {
            Arm::TightenedStatesWindowed(window_arm) | Arm::ClauseFloorWindowed(window_arm) => Some(window_arm),
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates
            | Arm::TightenedRootsAtMount | Arm::WholeRing(_) => None,
        }
    }

    fn whole_ring_arm(&self) -> Option<WholeRingArm> {
        match self.arm {
            Arm::WholeRing(whole_ring_arm) => Some(whole_ring_arm),
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates
            | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) => None,
        }
    }

    /// RGall 一族的整环取法：每块幸存盘上最新的有效根取小；某块盘整环里一条有效根都没有时，读法 P 不让它进取小、读法 Q 记 0。
    fn newest_valid_txg_on_every_surviving_disk_in_the_whole_ring(&self, reading: WholeRingCeilingReading) -> u64 {
        let newest_on_disk = |disk: usize| self.newest_valid_txg_on_disk(disk);
        match reading {
            WholeRingCeilingReading::DisksWithoutAValidRootAreSkipped => self.surviving_disks().iter().filter_map(|disk| newest_on_disk(*disk)).min().unwrap_or(0),
            WholeRingCeilingReading::DisksWithoutAValidRootCountAsZero => self.surviving_disks().iter().map(|disk| newest_on_disk(*disk).unwrap_or(0)).min().unwrap_or(0),
        }
    }

    /// RGall 一族抬 F 的上限：整环里第 4 新的非空有效根与每块幸存盘上最新的有效根取小（原 G2S 的取法，加上读法 P / Q 这一支）。
    fn whole_ring_upper_limit(&self, reading: WholeRingCeilingReading) -> u64 {
        self.fourth_newest_nonempty_valid_txg().min(self.newest_valid_txg_on_every_surviving_disk_in_the_whole_ring(reading))
    }

    /// RGall 一族回收门槛里「最旧有效根」那一项：整环；阳性对照只把它换成缩短一行的窗口里的最旧有效根。
    fn oldest_valid_txg_for_the_whole_ring_gate(&self, whole_ring_arm: WholeRingArm) -> u64 {
        if whole_ring_arm.is_reclaim_window_shortened() {
            self.oldest_valid_txg_in_the_reclaim_window(self.ring_depth - REGION_COUNT).unwrap_or(0)
        } else {
            self.oldest_persisted_valid_txg()
        }
    }

    /// W2：候选集的窗口，锚 = 这个进程记到的最新根的 txg（实现 `mount.rs` 的 `rollback_candidate` 锚在现行那一版的根上）。非窗口臂恒在窗口里。
    pub(super) fn is_inside_candidate_window(&self, txg: u64) -> bool {
        match self.window_arm() {
            None => true,
            Some(_) => txg_is_inside_window(txg, self.newest_durable_txg, self.ring_depth),
        }
    }

    /// W1：窗口里最旧的有效根；窗口里一条都没有时 `None`。锚 = 这个进程记到的最新根的 txg（`allocator.rs` 的 `record_root_written_by_this_process`）。
    pub(super) fn oldest_valid_txg_in_the_reclaim_window(&self, window_length: u64) -> Option<u64> {
        self.valid_roots().iter().map(|root| root.txg).filter(|txg| txg_is_inside_window(*txg, self.newest_durable_txg, window_length)).min()
    }

    /// W1 的第二项：这条臂回收门槛里「最旧有效根」那一项（取窗口还是取整环由臂定）。
    fn oldest_valid_txg_for_the_reclaim_gate(&self, window_arm: WindowArm) -> u64 {
        match window_arm.reclaim_gate() {
            ReclaimGate::OldestValidRootInTheWholeRing => self.oldest_persisted_valid_txg(),
            ReclaimGate::OldestValidRootInTheWindow => {
                let window_length = if window_arm.is_reclaim_window_shortened() { self.ring_depth - REGION_COUNT } else { self.ring_depth };
                self.oldest_valid_txg_in_the_reclaim_window(window_length).unwrap_or(0)
            }
        }
    }

    /// W3：幸存盘上窗口里的有效根（盘, 根）。锚 = 幸存盘上最新的有效根的 txg（`mount.rs` 的 `roots_of_the_ring_for_the_ceiling` 取法）。
    fn valid_roots_with_disk_in_the_ceiling_window(&self) -> Vec<(usize, RootRecord)> {
        let roots = self.valid_roots_with_disk();
        match roots.iter().map(|(_, root)| root.txg).max() {
            None => Vec::new(),
            Some(anchor) => roots.into_iter().filter(|(_, root)| txg_is_inside_window(root.txg, anchor, self.ring_depth)).collect(),
        }
    }

    /// W3：窗口里第 4 新的非空有效根；窗口里非空有效根不足 4 个时取窗口里最旧的有效根。
    fn fourth_newest_nonempty_valid_txg_in_the_window(&self) -> u64 {
        let roots = self.valid_roots_with_disk_in_the_ceiling_window();
        let distinct_newest_first: Vec<u64> = roots.iter().map(|(_, root)| root.txg)
            .filter(|txg| self.nonempty_txgs.contains(txg)).collect::<BTreeSet<u64>>().into_iter().rev().collect();
        if distinct_newest_first.len() >= MINIMUM_RETAINED_ROOTS {
            distinct_newest_first[MINIMUM_RETAINED_ROOTS - 1]
        } else {
            roots.iter().map(|(_, root)| root.txg).min().unwrap_or(0)
        }
    }

    /// W3：每块幸存盘上窗口里最新的有效根取小；某块盘在窗口里一条有效根都没有时按读法 P（不进取小）或 Q（记 0）。
    fn newest_valid_txg_on_every_surviving_disk_in_the_window(&self, reading: CeilingReading) -> u64 {
        let roots = self.valid_roots_with_disk_in_the_ceiling_window();
        let newest_on_disk = |disk: usize| roots.iter().filter(|(root_disk, _)| *root_disk == disk).map(|(_, root)| root.txg).max();
        match reading {
            CeilingReading::DisksWithoutAWindowRootAreSkipped => self.surviving_disks().iter().filter_map(|disk| newest_on_disk(*disk)).min().unwrap_or(0),
            CeilingReading::DisksWithoutAWindowRootCountAsZero => self.surviving_disks().iter().map(|disk| newest_on_disk(*disk).unwrap_or(0)).min().unwrap_or(0),
        }
    }

    /// W3：加了窗口的抬 F 上限。
    pub(super) fn windowed_states_upper_limit(&self, window_arm: WindowArm) -> u64 {
        self.fourth_newest_nonempty_valid_txg_in_the_window().min(self.newest_valid_txg_on_every_surviving_disk_in_the_window(window_arm.ceiling_reading()))
    }

    /// W3：加了窗口的处置目标；窗口里非空有效根不足 4 个时照目标本身。
    fn windowed_disposal_target(&self, goal_txg: u64) -> u64 {
        let nonempty_count = self.valid_roots_with_disk_in_the_ceiling_window().iter()
            .filter(|(_, root)| self.nonempty_txgs.contains(&root.txg)).map(|(_, root)| root.txg).collect::<BTreeSet<u64>>().len();
        if nonempty_count >= MINIMUM_RETAINED_ROOTS { goal_txg.min(self.fourth_newest_nonempty_valid_txg_in_the_window()) } else { goal_txg }
    }

    /// W4：`df` 滞后量的下界 max(第 4 新的非空有效根, 环里最旧有效根)；「第 4 新的非空有效根」取窗口（读法 a）还是整环（读法 b）由臂定，
    /// 「环里最旧有效根」两种读法都取整环。
    pub(super) fn windowed_lag_bound_txg(&self, window_arm: WindowArm) -> u64 {
        let fourth_newest = match window_arm.lag_bound_reading() {
            LagBoundReading::FourthNewestInTheWindow => self.fourth_newest_nonempty_valid_txg_in_the_window(),
            LagBoundReading::FourthNewestInTheWholeRing => self.fourth_newest_nonempty_valid_txg(),
        };
        fourth_newest.max(self.oldest_persisted_valid_txg())
    }

    /// 第六节第 6 行：这条臂自己的回收门槛减整环门槛（每次 `allocate` 之前读一次）。非窗口臂不读。
    fn observe_gate_difference(&mut self) {
        if !self.arm.records_the_four_trajectories() {
            return;
        }
        let own_gate = self.reuse_bound();
        let whole_ring_gate = self.floor_active.max(self.oldest_persisted_valid_txg());
        let is_supply_exhausted = self.is_supply_exhausted;
        self.metrics.trajectory.gate_difference.observe(signed_difference(own_gate, whole_ring_gate), is_supply_exhausted);
    }

    /// 第八节：这条臂的抬 F 上限减原 G2S 整环取法的上限（每次决定抬不抬 F 时读一次）。非窗口臂不读。
    pub(super) fn observe_ceiling_difference(&mut self) {
        if !self.arm.records_the_four_trajectories() {
            return;
        }
        self.observe_surviving_disks_without_a_valid_root();
        let own_ceiling = self.floor_upper_limit();
        let whole_ring = self.per_disk_states_upper_limit();
        let is_supply_exhausted = self.is_supply_exhausted;
        self.metrics.trajectory.ceiling_difference.observe(signed_difference(own_ceiling, whole_ring), is_supply_exhausted);
    }

    /// 第 2 次重跑第六节第 14 行：每次决定抬不抬 F 时，某块幸存盘整环里一条有效根都没有的次数（RGall 读法 P 与 Q 只在这种状态上分开）。
    fn observe_surviving_disks_without_a_valid_root(&mut self) {
        if !self.arm.collects_second_rerun_observations() {
            return;
        }
        let has_a_disk_without_a_valid_root = self.surviving_disks().iter().any(|disk| self.newest_valid_txg_on_disk(*disk).is_none());
        self.metrics.second_rerun.disk_without_valid_root_events += u64::from(has_a_disk_without_a_valid_root);
    }

    /// 第八节：滞后量与这条臂自己的 `df`（每个用户窗口读一次）。非窗口臂不读。
    fn observe_lag_and_df(&mut self, lag: u64) {
        if !self.arm.records_the_four_trajectories() {
            return;
        }
        let df = self.df_arm();
        let is_supply_exhausted = self.is_supply_exhausted;
        self.metrics.trajectory.lag.observe(signed_difference(lag, 0), is_supply_exhausted);
        self.metrics.trajectory.df.observe(signed_difference(df, 0), is_supply_exhausted);
        self.metrics.trajectory.df_below_user_blocks_windows += u64::from(df < USER_BLOCKS);
    }

    /// 这条臂抬 F 的上限。
    pub(super) fn floor_upper_limit(&self) -> u64 {
        match self.arm {
            Arm::FloorTxg => (self.newest_durable_txg + 1).saturating_sub(TXG_ARITHMETIC_GENERATIONS),
            Arm::FloorPerDiskTxgUpperLimit => (self.newest_durable_txg + 1).saturating_sub(TXG_UPPER_LIMIT_GENERATIONS),
            Arm::FloorPerDisk | Arm::TightenedRoots | Arm::TightenedRootsAtMount => self.per_disk_retention_upper_limit(),
            Arm::TightenedStates => self.per_disk_states_upper_limit(),
            Arm::TightenedStatesWindowed(window_arm) | Arm::ClauseFloorWindowed(window_arm) => self.windowed_states_upper_limit(window_arm),
            Arm::WholeRing(whole_ring_arm) => self.whole_ring_upper_limit(whole_ring_arm.ceiling_reading()),
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum => 0,
        }
    }

    /// 处置的目标：按状态数读时不越过第 4 新的非空根（非空有效根不足 4 个时照目标本身）。
    pub(super) fn disposal_target(&self, goal_txg: u64) -> u64 {
        if self.window_arm().is_some() {
            return self.windowed_disposal_target(goal_txg);
        }
        if self.arm.counts_states() {
            let nonempty_count = self.valid_roots().iter().filter(|root| self.nonempty_txgs.contains(&root.txg)).map(|root| root.txg).collect::<BTreeSet<u64>>().len();
            if nonempty_count >= MINIMUM_RETAINED_ROOTS { goal_txg.min(self.fourth_newest_nonempty_valid_txg()) } else { goal_txg }
        } else {
            goal_txg
        }
    }

    /// 候选集的下界：幸存盘上全部自证合法的根（含被抛弃时间线的根）所带 F 的最大值。
    pub(super) fn candidacy_floor(&self) -> u64 {
        if self.arm.uses_floor() {
            self.visible_roots().iter().map(|root| root.floor).max().unwrap_or(0)
        } else {
            0
        }
    }

    /// 第 2 次重跑第 1 行：条款形的候选下界 = max(`candidacy_floor()`, 系统配置里的 F)（D16 已定项 1「生效」那一行：F_生效 取三者最大，系统配置里读得出的 F 是其中一项）。
    /// 只换候选集的三处（`count_fake_candidates`、`admin_rollback` 选目标、`is_retention_violated`）；恢复之后的 F（`floors_after_recovery`）与回收门槛不动（登记差异 ③）。
    pub(super) fn candidate_floor(&self) -> u64 {
        if self.arm.uses_the_clause_candidate_floor() { self.candidacy_floor().max(self.system_configuration_floor) } else { self.candidacy_floor() }
    }

    /// 可再分配的谓词：已释放 ∧ 释放代 ≤ 这个界。
    pub(super) fn reuse_bound(&self) -> u64 {
        match self.arm {
            Arm::RingPersisted => self.oldest_persisted_valid_txg(),
            Arm::ProtectedGenerationsAtMinimum => (self.newest_durable_txg + 1).saturating_sub(TXG_ARITHMETIC_GENERATIONS),
            Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount => {
                self.floor_active.max(self.oldest_persisted_valid_txg())
            }
            Arm::TightenedStatesWindowed(window_arm) | Arm::ClauseFloorWindowed(window_arm) => self.floor_active.max(self.oldest_valid_txg_for_the_reclaim_gate(window_arm)),
            Arm::WholeRing(whole_ring_arm) => self.floor_active.max(self.oldest_valid_txg_for_the_whole_ring_gate(whole_ring_arm).min(self.oldest_core_gate_pin())),
        }
    }

    /// CG-P：钉着的槽里最小的钉值；不是 CG 臂或一个都没钉时 `u64::MAX`（取最小时不起作用）。
    fn oldest_core_gate_pin(&self) -> u64 {
        self.core_gate.as_ref().and_then(|state| state.pinned_slots.values().min().copied()).unwrap_or(u64::MAX)
    }

    /// 第 3 次重跑第六节第 4 行：每次 `allocate` 之前读 CG 门槛与同一状态上旧式门槛之差、F 那一项之差、钉值起没起作用、扣住那一截在不在等。只在 CG 臂上做。
    fn observe_the_core_gate(&mut self) {
        let (Some(state), Some(whole_ring_arm)) = (self.core_gate.as_ref(), self.whole_ring_arm()) else {
            return;
        };
        let core_gate = self.reuse_bound();
        let old_style_gate = state.shadow_floor_active.max(self.oldest_valid_txg_for_the_whole_ring_gate(whole_ring_arm));
        let gate_difference = signed_difference(core_gate, old_style_gate);
        let is_floor_term_different = self.floor_active != state.shadow_floor_active;
        let is_pin_effective = self.oldest_core_gate_pin() < self.oldest_persisted_valid_txg();
        let is_held_waiting = self.floor_active < state.effective_floor;
        let is_supply_exhausted = self.is_supply_exhausted;
        let observations = &mut self.metrics.core_gate;
        observations.gate_difference.observe(gate_difference, is_supply_exhausted);
        observations.gate_difference_nonzero_samples += u64::from(gate_difference != 0);
        observations.gate_difference_negative_samples += u64::from(gate_difference < 0);
        observations.floor_term_difference_nonzero_samples += u64::from(is_floor_term_different);
        observations.pin_effective_samples += u64::from(is_pin_effective);
        observations.held_waiting_samples += u64::from(is_held_waiting);
    }

    fn count_freed_after(&self, bound: u64) -> u64 {
        self.pool.blocks.iter().filter(|state| match state {
            BlockState::Unused | BlockState::Live { .. } => false,
            BlockState::Freed { freed_at, .. } => *freed_at > bound,
        }).count() as u64
    }

    pub(super) fn reusable_count(&self) -> u64 {
        let bound = self.reuse_bound();
        self.pool.blocks.iter().filter(|state| match state {
            BlockState::Unused => true,
            BlockState::Live { .. } => false,
            BlockState::Freed { freed_at, .. } => *freed_at <= bound,
        }).count() as u64
    }

    /// 已释放而还不可再分配的块数。
    pub(super) fn pinned_count(&self) -> u64 {
        self.count_freed_after(self.reuse_bound())
    }

    /// 滞后量的下界 max(第 4 新的非空有效根, 环里最旧有效根)：窗口一族按各自的读法 a / b，别的臂（含 RGall 一族）整环取。
    pub(super) fn lag_bound_txg(&self) -> u64 {
        match self.window_arm() {
            Some(window_arm) => self.windowed_lag_bound_txg(window_arm),
            None => self.fourth_newest_nonempty_valid_txg().max(self.oldest_persisted_valid_txg()),
        }
    }

    /// 按状态数读时一次处置也放不回的已释放块（滞后量）：释放代 > max(第 4 新的非空根, 环里最旧有效根)，
    /// 且不是空发布放掉的（那是推空自己的残留，另外按最坏值扣）。正在攒的窗口放掉的也算在内。别的臂是 0。
    pub(super) fn lag_blocks(&self) -> u64 {
        if self.arm.counts_states() {
            let bound = self.lag_bound_txg();
            self.pool.blocks.iter().filter(|state| match state {
                BlockState::Unused | BlockState::Live { .. } => false,
                BlockState::Freed { freed_at, .. } => *freed_at > bound && !self.empty_txgs.contains(freed_at),
            }).count() as u64
        } else {
            0
        }
    }

    /// E135 的 df：没用过的 + 已释放的，扣住的块与保留池都算空闲。
    fn df_raw(&self) -> u64 {
        self.pool.blocks.iter().filter(|state| match state {
            BlockState::Unused | BlockState::Freed { .. } => true,
            BlockState::Live { .. } => false,
        }).count() as u64
    }

    /// 收严臂：当前活着的元数据（上一个窗口的 5 块，或推空最后一次发布的 c 块）算保留池的占用——保留池正是为它留的，
    /// 不从用户的空闲里扣。别的臂照 E138 不记这一笔。
    fn live_metadata_credit(&self) -> u64 {
        if self.arm.is_tightened() { self.pool.metadata.len() as u64 } else { 0 }
    }

    /// 这条臂报的 df：扣掉保留池与推空最坏残留（G2S 再扣滞后量）；收严臂把活着的元数据记回保留池。
    fn df_arm(&self) -> u64 {
        (self.df_raw() + self.live_metadata_credit()).saturating_sub(self.reserve_blocks + self.push_residue_blocks + self.lag_blocks())
    }

    /// 普通分配看得见的：扣掉保留池；收严形态再按最坏残留收费（不许超过这条臂自己报的 df）。
    fn user_allocatable(&self) -> u64 {
        let plain = (self.reusable_count() + self.live_metadata_credit()).saturating_sub(self.reserve_blocks);
        if self.arm.is_tightened() { plain.min(self.df_arm()) } else { plain }
    }

    /// 取 count 块：先取可再分配的已释放块，再取没用过的。复用一块就把它上一段区间记成损坏。
    pub(super) fn allocate(&mut self, count: u64, txg: u64) -> Option<Vec<usize>> {
        self.observe_gate_difference();
        self.observe_the_core_gate();
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

    pub(super) fn free_blocks(&mut self, blocks: &[usize], txg: u64) {
        for &index in blocks {
            if let BlockState::Live { allocated_at } = self.pool.blocks[index] {
                self.pool.blocks[index] = BlockState::Freed { allocated_at, freed_at: txg };
                self.blocks_freed_in_total += 1;
            }
        }
    }

    /// (假候选数, 候选数, 其中非空发布的根数, 其中不同的用户可见状态数)。
    /// 状态数：非空候选各算一个；候选集里比最旧非空候选还旧的空根共享同一个更早的状态，再算一个。
    pub(super) fn count_fake_candidates(&self) -> (u64, u64, u64, u64) {
        self.count_fake_candidates_at_floor(self.candidate_floor())
    }

    /// 同一个状态上按给定的下界数候选集：第 2 次重跑拿它同时数旧下界与条款形下界（`record_check`）。
    pub(super) fn count_fake_candidates_at_floor(&self, floor: u64) -> (u64, u64, u64, u64) {
        let candidates: Vec<RootRecord> = self.valid_roots().into_iter().filter(|root| root.txg >= floor).filter(|root| self.is_inside_candidate_window(root.txg)).collect();
        let fake = candidates.iter().filter(|root| self.damaged_txgs.contains(&root.txg)).count() as u64;
        let nonempty_txgs: BTreeSet<u64> = candidates.iter().map(|root| root.txg).filter(|txg| self.nonempty_txgs.contains(txg)).collect();
        let oldest_nonempty = nonempty_txgs.iter().next().copied();
        let has_older_empty = candidates.iter().any(|root| match oldest_nonempty {
            None => true,
            Some(oldest) => root.txg < oldest,
        });
        let distinct_states = nonempty_txgs.len() as u64 + u64::from(has_older_empty && !candidates.is_empty());
        let nonempty = candidates.iter().filter(|root| self.nonempty_txgs.contains(&root.txg)).count() as u64;
        (fake, candidates.len() as u64, nonempty, distinct_states)
    }

    /// 最少保留：没丢盘时 F_候选 ≤ 这条臂的上限（按根数或按状态数）；丢盘之后查幸存盘上最新的有效根在候选集里；管理员回退之后不查。
    fn is_retention_violated(&self) -> bool {
        if !self.arm.uses_floor() || self.admin_rollback_happened {
            return false;
        }
        let floor = self.candidate_floor();
        if self.is_any_disk_lost() {
            self.surviving_disks().iter().any(|disk| self.newest_valid_txg_on_disk(*disk).is_some_and(|newest| newest < floor || !self.is_inside_candidate_window(newest)))
        } else if let Some(window_arm) = self.window_arm() {
            floor > self.windowed_states_upper_limit(window_arm)
        } else if let Some(whole_ring_arm) = self.whole_ring_arm() {
            floor > self.whole_ring_upper_limit(whole_ring_arm.ceiling_reading())
        } else if self.arm.counts_states() {
            floor > self.per_disk_states_upper_limit()
        } else {
            floor > self.per_disk_retention_upper_limit()
        }
    }

    fn record_check(&mut self) {
        let (fake, candidates, nonempty, distinct_states) = self.count_fake_candidates();
        self.metrics.fake_candidates += fake;
        self.metrics.checks += 1;
        self.metrics.minimum_candidates = self.metrics.minimum_candidates.min(candidates);
        self.metrics.candidate_sum += candidates;
        self.metrics.nonempty_candidate_sum += nonempty;
        self.metrics.minimum_distinct_states = self.metrics.minimum_distinct_states.min(distinct_states);
        self.metrics.distinct_state_sum += distinct_states;
        if self.is_retention_violated() {
            self.metrics.retention_violations += 1;
        }
        self.observe_candidate_floors_in_the_check(fake, candidates);
    }

    /// 第 2 次重跑第六节第 4、7、8 行与 V6、V7：同一状态上用旧下界与条款形下界各数一遍候选集。只在 `collects_second_rerun_observations` 的臂上做。
    fn observe_candidate_floors_in_the_check(&mut self, new_fakes: u64, new_candidates: u64) {
        if !self.arm.collects_second_rerun_observations() {
            return;
        }
        let old_floor = self.candidacy_floor();
        let new_floor = self.candidate_floor();
        let is_supply_exhausted = self.is_supply_exhausted;
        self.metrics.second_rerun.candidate_floor_gap.observe(signed_difference(new_floor, old_floor), is_supply_exhausted);
        let (old_fakes, old_candidates) = if new_floor == old_floor {
            (new_fakes, new_candidates)
        } else {
            let (fakes, candidates, _, _) = self.count_fake_candidates_at_floor(old_floor);
            (fakes, candidates)
        };
        let is_system_configuration_below_root_floor = self.system_configuration_floor < old_floor;
        let observations = &mut self.metrics.second_rerun;
        observations.fakes_with_the_old_floor += old_fakes;
        observations.checks_old_set_strictly_larger += u64::from(old_candidates > new_candidates);
        observations.checks_new_set_strictly_larger += u64::from(new_candidates > old_candidates);
        observations.system_configuration_below_root_floor_events += u64::from(is_system_configuration_below_root_floor);
    }

    fn prune_damage(&mut self) {
        let oldest_visible_txg = self.ring.iter().flatten().map(|root| root.txg).min().unwrap_or(0);
        self.damaged_txgs.retain(|txg| *txg >= oldest_visible_txg);
    }

    pub(super) fn prefill(&mut self, fill: f64) {
        let target_live = truncate_f64_to_usize(CAPACITY_BLOCKS as f64 * fill);
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
        while self.snapshots.len() > self.snapshots_kept_limit {
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
    pub(super) fn floor_written_on_every_surviving_disk(&self, value: u64) -> bool {
        let roots = self.valid_roots_with_disk();
        self.surviving_disks().iter().all(|disk| roots.iter().any(|(root_disk, root)| root_disk == disk && root.floor >= value))
    }

    /// 发布 txg 为 t 的根。根槽写失败时 txg 推进一格再发（D23（journal 的角色与格式） 已定项 14 的索引行），写失败的槽留着旧内容；重发不再分配块。
    pub(super) fn publish(&mut self, txg: u64) {
        let slot = self.slot_index(txg);
        let attempt = self.root_write_attempts;
        self.root_write_attempts += 1;
        let is_planned_failure = self.planned_root_write_failures.remove(&attempt);
        if self.fail_next_slot_write || is_planned_failure || self.broken_slot == Some(slot) || self.stuck_disk == Some(self.disk_of_slot(slot)) {
            self.fail_next_slot_write = false;
            self.metrics.root_write_failures_injected += 1;
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
        } else {
            self.empty_txgs.insert(txg);
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
                Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => {
                    self.floor_written_on_every_surviving_disk(pending.value)
                }
                Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum => true,
            };
            if is_active {
                self.floor_active = self.floor_active.max(pending.value);
                self.pending_floor = None;
            } else {
                self.pending_floor = Some(pending);
            }
        }
        self.advance_the_core_gate_after_a_durable_root(slot);
        self.prune_damage();
        self.record_check();
        self.observe_after_the_root_is_durable();
    }

    /// 每块幸存盘上都有一条持久、自证合法、有效、实例号不小于 `since_instance` 的根（这个进程恢复之后写的根；装置的回退另开实例，登记差异 ⑯）。
    fn is_rooted_on_every_surviving_disk_from_instance(&self, since_instance: u64) -> bool {
        let roots = self.valid_roots_with_disk();
        self.surviving_disks().iter().all(|disk| roots.iter().any(|(root_disk, root)| root_disk == disk && root.instance >= since_instance))
    }

    /// CG 臂一条自证合法的根落盘之后：写进的那一槽解钉（CG-P）；扣住那一截在本实例的根落满每块幸存盘时放开（CG-F，登记差异 ⑮）；
    /// 旧式影子的等生效照原样（`floor_written_on_every_surviving_disk`）跟。
    fn advance_the_core_gate_after_a_durable_root(&mut self, slot: usize) {
        let Some(state) = self.core_gate.as_ref() else {
            return;
        };
        let is_shadow_pending_floor_active = state.shadow_pending_floor_value.is_some_and(|value| self.floor_written_on_every_surviving_disk(value));
        let is_held_part_released = state.held_until_rooted_from_instance.is_some_and(|instance| self.is_rooted_on_every_surviving_disk_from_instance(instance));
        let held_effective_floor = state.effective_floor;
        let mutable_core_gate_state = self.core_gate.as_mut().expect("上面刚读过 CG 状态");
        mutable_core_gate_state.pinned_slots.remove(&slot);
        if is_shadow_pending_floor_active {
            let pending_value = mutable_core_gate_state.shadow_pending_floor_value.take().expect("影子在等生效才判得出生效");
            mutable_core_gate_state.shadow_floor_active = mutable_core_gate_state.shadow_floor_active.max(pending_value);
        }
        if is_held_part_released {
            self.floor_active = self.floor_active.max(held_effective_floor);
            mutable_core_gate_state.held_until_rooted_from_instance = None;
        }
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
    pub(super) fn raise_floor_to(&mut self, target: u64) {
        if target > self.floor_carried {
            self.system_configuration_floor = self.system_configuration_floor.max(target);
            self.floor_carried = target;
            self.pending_floor = Some(PendingFloor { value: target, carrier_txg: None, durable_publications_since_carrier: 0 });
        }
        if let Some(state) = self.core_gate.as_mut() {
            if target > state.shadow_floor_carried {
                state.shadow_floor_carried = target;
                state.shadow_pending_floor_value = Some(target);
            }
        }
    }

    /// 有界处置的第一步：把正在攒的窗口（里面已有这次删掉的对象）先发布出去。
    pub(super) fn commit_open_window(&mut self, txg: u64) -> bool {
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

    pub(super) fn is_goal_released(&self, goal_txg: u64) -> bool {
        self.reuse_bound() >= goal_txg
    }

    fn maximum_empty_publications(&self) -> u64 {
        match self.arm {
            Arm::RingPersisted => self.ring_depth - 1,
            Arm::ProtectedGenerationsAtMinimum => 0,
            Arm::FloorTxg => TXG_ARITHMETIC_GENERATIONS + 2,
            Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_)
            | Arm::WholeRing(_) => {
                self.arm.publication_bound(self.ring_depth) - 1
            }
        }
    }

    /// 发布正在攒的窗口之后按臂处置，返回做了几次空发布。
    /// stop_when_writable = false 时一直做到目标放回为止（判据 1 的探针用）。
    /// 收严臂：目标放回了就停（再推也只是把自己的元数据扣进 F 之上）。
    pub(super) fn push_after_commit(&mut self, goal_txg: u64, stop_when_writable: bool) -> u64 {
        let maximum_empty_publications = self.maximum_empty_publications();
        let target = self.disposal_target(goal_txg);
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
                Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit => {
                    if stop_when_writable { self.user_allocatable() >= USER_BLOCKS } else { self.is_goal_released(target) }
                }
                Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => {
                    (stop_when_writable && self.user_allocatable() >= USER_BLOCKS) || self.is_goal_released(target)
                }
            };
            if is_done {
                break;
            }
            match self.arm {
                Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_)
                | Arm::WholeRing(_) => {
                    // 等上限够到目标才抬、一次抬到目标（预注册：每次都抬到当时的上限会让带新 F 的根反复重来）
                    self.observe_ceiling_difference();
                    if self.floor_carried < target && self.floor_upper_limit() >= target {
                        self.raise_floor_to(target);
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
    pub(super) fn forced_pressure(&mut self) {
        if !self.arm.uses_floor() {
            return;
        }
        let maximum_empty_publications = self.maximum_empty_publications();
        self.observe_ceiling_difference();
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
    pub(super) fn run_user_window(&mut self, delete_first: bool, crash_before_root: bool) -> WindowOutcome {
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
        let lag = self.lag_blocks();
        self.metrics.lag_sum += lag;
        self.metrics.lag_maximum = self.metrics.lag_maximum.max(lag);
        self.metrics.lag_samples += 1;
        self.observe_lag_and_df(lag);
        let mut create_txg = open_txg;
        if self.user_allocatable() < USER_BLOCKS {
            self.metrics.admissions_under_pressure += 1;
            self.is_supply_exhausted = true;
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
                if self.df_arm() >= USER_BLOCKS {
                    self.metrics.false_enospc_arm += 1;
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

    /// 近满盘循环里的一个删建窗口：记「删了再写失败」与它的窗口号。
    pub(super) fn run_counted_delete_create_window(&mut self) -> WindowOutcome {
        self.counting_window += 1;
        let outcome = self.run_user_window(true, false);
        if outcome == WindowOutcome::NoSpace {
            self.metrics.write_after_delete_failed += 1;
            self.metrics.first_write_after_delete_failure_window = self.metrics.first_write_after_delete_failure_window.min(self.counting_window);
            self.metrics.last_write_after_delete_failure_window = self.metrics.last_write_after_delete_failure_window.max(self.counting_window);
        }
        outcome
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
            Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_)
            | Arm::WholeRing(_) => {
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

    /// CG-F 与 CG-P 在择根之前读的值；不是 CG 臂或开关关着时 `None`。`first_txg` 是恢复之后新实例第一次发布的 txg。
    fn read_the_core_gate_at_recovery(&self, first_txg: u64) -> Option<CoreGateReadingAtRecovery> {
        if !self.core_gate.as_ref().is_some_and(|state| state.is_switched_on) {
            return None;
        }
        let roots = self.valid_roots_with_disk();
        let newest_root_on_each_surviving_disk: Vec<Option<RootRecord>> = self.surviving_disks().iter()
            .map(|disk| roots.iter().filter(|(root_disk, _)| root_disk == disk).map(|(_, root)| *root).max_by_key(|root| (root.txg, root.instance)))
            .collect();
        let highest_on_the_newest_roots = newest_root_on_each_surviving_disk.iter().flatten().map(|root| root.floor).max().unwrap_or(0);
        let highest_on_the_abandoned_roots = (0..self.ring.len())
            .filter(|slot| self.slot_survives(*slot))
            .filter_map(|slot| self.ring[slot])
            .filter(|root| root.is_self_verified && !self.is_instance_valid(root))
            .map(|root| root.floor).max().unwrap_or(0);
        let effective_floor = highest_on_the_newest_roots.max(highest_on_the_abandoned_roots).max(self.system_configuration_floor);
        let newest_valid_txg_on_every_surviving_disk = if newest_root_on_each_surviving_disk.iter().all(Option::is_some) {
            newest_root_on_each_surviving_disk.iter().flatten().map(|root| root.txg).min()
        } else {
            None
        };
        let pin_window_end = first_txg.min(self.ring_depth);
        let pinned_slots: BTreeMap<usize, u64> = (0..self.ring.len())
            .filter(|slot| self.slot_survives(*slot))
            .filter(|slot| self.ring[*slot].is_some_and(|root| !root.is_self_verified))
            .filter_map(|slot| (0..pin_window_end).find(|txg| self.slot_index(*txg) == slot).map(|txg| (slot, txg)))
            .collect();
        Some(CoreGateReadingAtRecovery { effective_floor, newest_valid_txg_on_every_surviving_disk, pinned_slots })
    }

    /// 恢复之后取 F：CG 臂照 CG-F / CG-P（新根带 F_生效，F 那一项当场 min(F_生效, m)、扣到本实例的根落满每块幸存盘，钉槽换成这一次读到的），
    /// 别的臂照原样；旧式影子总照 `floors_after_recovery` 的两个值取。
    fn adopt_floors_after_recovery(&mut self, candidacy: u64, settled: u64, core_gate_reading: Option<CoreGateReadingAtRecovery>) {
        self.adopt_the_core_gate_shadow_floors(candidacy, settled);
        let Some(reading) = core_gate_reading else {
            self.adopt_floors(candidacy, settled);
            return;
        };
        let floor_term_now = match reading.newest_valid_txg_on_every_surviving_disk {
            Some(newest_valid_txg_on_every_surviving_disk) => reading.effective_floor.min(newest_valid_txg_on_every_surviving_disk),
            None => FLOOR_TERM_WHEN_A_SURVIVING_DISK_HAS_NO_VALID_ROOT,
        };
        self.floor_carried = reading.effective_floor;
        self.floor_active = floor_term_now;
        self.pending_floor = None;
        let instance = self.instance;
        let state = self.core_gate.as_mut().expect("读得到 CG 的值就是 CG 臂");
        state.effective_floor = reading.effective_floor;
        state.held_until_rooted_from_instance = if floor_term_now < reading.effective_floor { Some(instance) } else { None };
        state.pinned_slots = reading.pinned_slots;
    }

    /// 管理员回退之后取 F：CG 臂开关开着时照 CG-R 一样都不动（`floor_carried`、F 那一项、等生效与扣住的两截、钉槽），别的照原样重算；旧式影子照原样重算。
    fn adopt_floors_after_admin_rollback(&mut self, candidacy: u64, settled: u64) {
        self.adopt_the_core_gate_shadow_floors(candidacy, settled);
        let keeps_the_floors = self.core_gate.as_ref().is_some_and(|state| state.is_switched_on);
        if !keeps_the_floors {
            self.adopt_floors(candidacy, settled);
        }
    }

    /// 旧式影子照 `adopt_floors` 的取法记下（RGall 一族的等生效只看 `floor_written_on_every_surviving_disk`，载体那两项不用）。
    fn adopt_the_core_gate_shadow_floors(&mut self, candidacy: u64, settled: u64) {
        if let Some(state) = self.core_gate.as_mut() {
            state.shadow_floor_carried = candidacy;
            state.shadow_floor_active = settled;
            state.shadow_pending_floor_value = if candidacy > settled { Some(candidacy) } else { None };
        }
    }

    /// 崩溃之后的恢复：先验全部候选再择新（D22（单元原子性怎么合成） 已定项 7），分配状态从所选根重新载入，取新实例代号。
    pub(super) fn crash_and_recover(&mut self) {
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
        let core_gate_reading = self.read_the_core_gate_at_recovery(self.maximum_published_txg + 1);
        if !self.restore_snapshot(chosen.instance, chosen.txg) {
            self.metrics.unrecoverable += 1;
            self.halted = true;
            return;
        }
        self.instance_rows.push((self.instance, chosen.txg));
        self.instance += 1;
        self.newest_durable_txg = chosen.txg;
        self.next_txg = self.maximum_published_txg + 1;
        self.adopt_floors_after_recovery(candidacy, settled, core_gate_reading);
        self.observe_pinned_after_recovery();
        self.save_snapshot();
        self.record_check();
    }

    /// ⓐ（登记第一节读法第 10 条）：每次 `crash_and_recover` 做完（`adopt_floors` 之后）这条臂的 `pinned_count()`。采样单位是一次恢复。
    fn observe_pinned_after_recovery(&mut self) {
        if !self.arm.collects_second_rerun_observations() {
            return;
        }
        let is_supply_exhausted = self.is_supply_exhausted;
        let pinned_blocks = self.pinned_count();
        self.metrics.second_rerun.pinned_after_recovery.observe(signed_difference(pinned_blocks, 0), is_supply_exhausted);
    }

    /// ⓑ：这条臂的门槛下扣住的块数减窗口门槛 max(F_生效, 窗口里最旧有效根) 下扣住的块数，即释放代落在两道门槛之间的已释放块数（两道门槛相等时 0）。
    pub(super) fn extra_pinned_blocks_against_the_window_gate(&self) -> i64 {
        let window_gate = self.floor_active.max(self.oldest_valid_txg_in_the_reclaim_window(self.ring_depth).unwrap_or(0));
        signed_difference(self.pinned_count(), self.count_freed_after(window_gate))
    }

    /// ⓑ：幸存盘上住着窗口外有效根的槽数（窗口锚是这个进程记到的最新根）。
    pub(super) fn slots_holding_a_valid_root_outside_the_window(&self) -> u64 {
        self.valid_roots_with_disk().iter()
            .filter(|(_, root)| !txg_is_inside_window(root.txg, self.newest_durable_txg, self.ring_depth))
            .count() as u64
    }

    /// 第 2 次重跑代价三样与刻度的采样点：每次根落盘之后（`publish` 里 `record_check` 之后）。
    fn observe_after_the_root_is_durable(&mut self) {
        if !self.arm.collects_second_rerun_observations() {
            return;
        }
        let is_supply_exhausted = self.is_supply_exhausted;
        let pinned_blocks = self.pinned_count();
        let extra_pinned_blocks = self.extra_pinned_blocks_against_the_window_gate();
        let hole_slots = self.slots_holding_a_valid_root_outside_the_window();
        let ring_depth = self.ring_depth;
        let published_txg = self.newest_durable_txg;
        let has_a_root_beyond_two_revolutions = self.valid_roots().iter().any(|root| root.txg + 2 * ring_depth <= published_txg);
        let blocks_freed_in_total = self.blocks_freed_in_total;
        let mark_origin_txg = self.stuck_marks_origin_txg;
        let observations = &mut self.metrics.second_rerun;
        observations.pinned_after_publication.observe(signed_difference(pinned_blocks, 0), is_supply_exhausted);
        observations.extra_pinned_against_the_window_gate.observe(extra_pinned_blocks, is_supply_exhausted);
        if hole_slots > 0 {
            observations.samples_with_a_hole += 1;
            observations.extra_pinned_sum_over_samples_with_a_hole += extra_pinned_blocks;
            observations.hole_slots_sum_over_samples_with_a_hole += hole_slots;
            let hole_slots_signed = signed_difference(hole_slots, 0);
            observations.extra_pinned_per_hole_maximum_in_hundredths = observations.extra_pinned_per_hole_maximum_in_hundredths.max(extra_pinned_blocks * 100 / hole_slots_signed);
        }
        observations.stale_beyond_two_revolutions_samples += u64::from(has_a_root_beyond_two_revolutions);
        if let Some(origin_txg) = mark_origin_txg {
            for (mark_index, revolutions) in STUCK_MARK_REVOLUTIONS.iter().enumerate() {
                if observations.marks[mark_index].is_none() && published_txg >= origin_txg + revolutions * ring_depth {
                    observations.marks[mark_index] = Some(StuckMark { published_txg, pinned_blocks, blocks_freed_in_total });
                }
            }
        }
    }

    /// 管理员回退到候选集里最旧的根（D23（journal 的角色与格式） 已定项 14 的形态）。
    /// 候选集里最旧的根：按实例表有效 ∧ txg ≥ 给定的下界 ∧ 在候选集的窗口里（RGall 一族与旧臂没有窗口这一项）。
    pub(super) fn oldest_candidate_root_at_floor(&self, floor: u64) -> Option<RootRecord> {
        self.valid_roots().into_iter().filter(|root| root.txg >= floor).filter(|root| self.is_inside_candidate_window(root.txg)).min_by_key(|root| root.txg)
    }

    /// 第 2 次重跑第六节第 4、7 行：管理员回退选目标时读一次下界差；新旧下界选出的目标不同就记一次。
    fn observe_rollback_target_choice(&mut self) {
        if !self.arm.collects_second_rerun_observations() {
            return;
        }
        let old_floor = self.candidacy_floor();
        let new_floor = self.candidate_floor();
        let target_with_the_old_floor = self.oldest_candidate_root_at_floor(old_floor).map(|root| root.txg);
        let target_with_the_new_floor = self.oldest_candidate_root_at_floor(new_floor).map(|root| root.txg);
        let is_supply_exhausted = self.is_supply_exhausted;
        let observations = &mut self.metrics.second_rerun;
        observations.candidate_floor_gap.observe(signed_difference(new_floor, old_floor), is_supply_exhausted);
        observations.rollback_target_changed += u64::from(target_with_the_old_floor != target_with_the_new_floor);
    }

    pub(super) fn admin_rollback(&mut self) -> bool {
        self.observe_rollback_target_choice();
        let target = match self.oldest_candidate_root_at_floor(self.candidate_floor()) {
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
        self.adopt_floors_after_admin_rollback(candidacy, settled);
        self.save_snapshot();
        self.record_check();
        true
    }

    /// C281 的构造：回退那次发布写了单元、根槽 FUA 之前崩溃。回退行没落盘，被抛弃的根仍按实例表有效。
    pub(super) fn admin_rollback_then_crash_before_root(&mut self) {
        let durable_rows = self.instance_rows.clone();
        let durable_instance = self.instance;
        let durable_newest = self.newest_durable_txg;
        let durable_next = self.next_txg;
        let durable_floors = (self.floor_carried, self.floor_active, self.pending_floor);
        let durable_core_gate = self.core_gate.clone();
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
        self.core_gate = durable_core_gate;
        self.crash_and_recover();
        self.halted = true;
    }
}
