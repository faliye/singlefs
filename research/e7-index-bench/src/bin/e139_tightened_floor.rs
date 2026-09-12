//! E139：按盘回退下界的收严形态 —— D16（发布语义） 已定项 1 第三轮判决之后的重跑。
//!
//! 预注册 `research/prompts/e139-preregistration.md`。装置从 E138（按盘回退下界与推空的空间要求） 的源码拷出来改：
//! ① 两条被测臂 G2R / G2S：准入按最坏残留收费、界与保留池给 k_tol = 2 次根槽写失败留余量、
//!    保留池与残留按声明的最坏空发布开销 c_max 算、G2S 的上限按用户 2026-09-12 的口径取第 4 新的非空根；
//! ② 新增五个世界：活化区间内崩溃不丢盘、先跳号再抬 F、推空期间根槽写失败、坏槽、空发布开销在挂载之后变大。

use e7_index_bench::Emitter;
use std::collections::{BTreeSet, HashSet, VecDeque};
use std::rc::Rc;

/// R = 3（D22（单元原子性怎么合成） 已定项 2）。
const REGION_COUNT: u64 = 3;
const DISK_COUNT: usize = 2;
/// 臂 C、臂 D 的 K_min：C222 按轮转算术给的 3。
const TXG_ARITHMETIC_GENERATIONS: u64 = 3;
/// 用户定案（2026-09-11）：盘紧时最少保留 4 个；2026-09-12 澄清「4」数的是不同的用户可见状态（臂 G2S）。
const MINIMUM_RETAINED_ROOTS: usize = 4;
/// 对照臂 G_txgcap：抬 F 的上限按 txg 数几代。
const TXG_UPPER_LIMIT_GENERATIONS: u64 = 4;
const USER_BLOCKS_PER_OBJECT: usize = 8;
const USER_BLOCKS: u64 = USER_BLOCKS_PER_OBJECT as u64;
const METADATA_BLOCKS_PER_PUBLICATION: u64 = 5;
const BLOCKS_PER_PUBLICATION: u64 = USER_BLOCKS + METADATA_BLOCKS_PER_PUBLICATION;
/// 臂 C、臂 D 沿用 E135（动态回退下界） 的保留池。
const CONTROL_ARM_RESERVE_BLOCKS: u64 = BLOCKS_PER_PUBLICATION;
/// E138 臂 G 的界：发布正在攒的窗口 + 至多 6 次空发布。
const FLOOR_PER_DISK_PUBLICATION_BOUND: u64 = 7;
/// 预注册：一次处置里容忍几次根槽写失败，每次失败最坏让盘 1 的槽轮空一次、界加 2。
const TOLERATED_ROOT_WRITE_FAILURES: u64 = 2;
const PUBLICATIONS_PER_TOLERATED_FAILURE: u64 = 2;
/// 预注册的无故障界：按根数读 7（E138 已钉），按状态数读 4。
const TIGHTENED_ROOTS_BASE_BOUND: u64 = 7;
const TIGHTENED_STATES_BASE_BOUND: u64 = 4;
const TIGHTENED_ROOTS_PUBLICATION_BOUND: u64 = TIGHTENED_ROOTS_BASE_BOUND + PUBLICATIONS_PER_TOLERATED_FAILURE * TOLERATED_ROOT_WRITE_FAILURES;
const TIGHTENED_STATES_PUBLICATION_BOUND: u64 = TIGHTENED_STATES_BASE_BOUND + PUBLICATIONS_PER_TOLERATED_FAILURE * TOLERATED_ROOT_WRITE_FAILURES;
const CAPACITY_BLOCKS: usize = 4000;
const STEADY_FILL: f64 = 0.70;
const WINDOWS_PER_WORLD: u64 = 400;
const SEED_COUNT: u64 = 24;
const FAULT_SEED_COUNT: u64 = 8;
const RESERVE_SWEEP_SEED_COUNT: u64 = 8;
const SLOTS_PER_REGION_SWEEP: [u64; 3] = [1, 4, 16];
const FAULT_SLOTS_SWEEP: [u64; 2] = [4, 16];
/// (挂载时的空发布开销 c₀, 声明的最坏开销 c_max)。
const COST_PAIRS: [(u64, u64); 2] = [(1, 5), (5, 10)];
/// 第 k 次根槽写失败，k ∈ 0..5。
const ACTIVATION_FAULT_ATTEMPTS: u64 = 5;
/// 第 j 个窗口分配完、根槽写之前崩溃，j ∈ 1..=8。
const ACTIVATION_FAULT_CRASH_WINDOWS: u64 = 8;
/// activation_crash_no_loss：第一次崩溃恢复之后再跑几个窗口才第二次崩溃并丢盘。
const WINDOWS_BETWEEN_CRASHES: u64 = 2;
/// skip_then_raise：抬 F 之前先让区域 1 的根槽连续写失败几次。
const SKIP_THEN_RAISE_REPEATS: [u64; 3] = [1, 2, 3];
/// drain_failure：3 次失败的组合只取前几种；臂 A 的失败位置只扫到第几次根槽写。
const DRAIN_FAILURE_TRIPLE_PATTERNS: usize = 20;
const DRAIN_FAILURE_RING_ARM_OFFSET_LIMIT: u64 = 12;
const DRAIN_FAILURE_WINDOWS_BEFORE: u64 = 5;
const DRAIN_FAILURE_WINDOWS_AFTER: u64 = 20;
/// broken_slot：槽 1（区域 0，盘 0）从此每次写都失败。
const BROKEN_SLOT: usize = 1;
const BROKEN_SLOT_WINDOWS: u64 = 1000;
/// cost_growth：近满盘删建循环做到第几个窗口时开销变大；超过声明上界那一格多几块。
const COST_GROWTH_AT_WINDOW: u64 = 100;
const COST_OVER_DECLARED_EXTRA: u64 = 3;
const RESERVE_SWEEP_BELOW: u64 = 6;
const RESERVE_SWEEP_ABOVE: u64 = 8;
const RESERVE_SWEEP_SLOTS_PER_REGION: u64 = 16;
/// 快照留多少份：要大于最大的根环深度 48。
const SNAPSHOTS_KEPT: usize = 64;
const BLOCK_BYTES: u64 = 16384;
const DETAIL_LINES_MAXIMUM: u64 = 24;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    RingPersisted,
    ProtectedGenerationsAtMinimum,
    FloorTxg,
    FloorPerDisk,
    FloorPerDiskTxgUpperLimit,
    TightenedRoots,
    TightenedStates,
    TightenedRootsAtMount,
}

const ARMS: [Arm; 8] = [
    Arm::RingPersisted,
    Arm::ProtectedGenerationsAtMinimum,
    Arm::FloorTxg,
    Arm::FloorPerDisk,
    Arm::FloorPerDiskTxgUpperLimit,
    Arm::TightenedRoots,
    Arm::TightenedStates,
    Arm::TightenedRootsAtMount,
];

/// 进近满盘一族世界（drain_failure / broken_slot / cost_growth）的臂。
const NEAR_FULL_FAULT_ARMS: [Arm; 4] = [Arm::RingPersisted, Arm::FloorPerDisk, Arm::TightenedRoots, Arm::TightenedStates];
const COST_GROWTH_ARMS: [Arm; 5] = [Arm::RingPersisted, Arm::FloorPerDisk, Arm::TightenedRoots, Arm::TightenedStates, Arm::TightenedRootsAtMount];
const CANDIDATE_ARMS: [Arm; 2] = [Arm::TightenedRoots, Arm::TightenedStates];

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::RingPersisted => "A_ring_persisted",
            Arm::ProtectedGenerationsAtMinimum => "C_k_runtime",
            Arm::FloorTxg => "D_floor_txg",
            Arm::FloorPerDisk => "G_floor_per_disk",
            Arm::FloorPerDiskTxgUpperLimit => "G_txg_cap",
            Arm::TightenedRoots => "G2R_tightened_roots",
            Arm::TightenedStates => "G2S_tightened_states",
            Arm::TightenedRootsAtMount => "G2R_mount",
        }
    }
    fn uses_floor(self) -> bool {
        match self {
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum => false,
            Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount => true,
        }
    }
    /// 被测的两条：判据 3–8 只对它们判。
    fn is_candidate(self) -> bool {
        match self {
            Arm::TightenedRoots | Arm::TightenedStates => true,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRootsAtMount => false,
        }
    }
    /// 收严形态三条共有的：准入按最坏残留收费、界带 k_tol 的余量。
    fn is_tightened(self) -> bool {
        match self {
            Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount => true,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit => false,
        }
    }
    /// 上限按第 4 新的非空根取（用户 2026-09-12 的口径）。
    fn counts_states(self) -> bool {
        match self {
            Arm::TightenedStates => true,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedRootsAtMount => false,
        }
    }
    /// 保留池、残留与 df 按哪个开销算：被测臂按声明的最坏值，其余按挂载时的值。
    fn formula_cost(self, mount_cost: u64, declared_maximum_cost: u64) -> u64 {
        match self {
            Arm::TightenedRoots | Arm::TightenedStates => declared_maximum_cost,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRootsAtMount => mount_cost,
        }
    }
    /// 预注册的界 B：一次准入最多用几次发布（含发布正在攒的窗口）。
    fn publication_bound(self, ring_depth: u64) -> u64 {
        match self {
            Arm::RingPersisted => ring_depth,
            Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit => FLOOR_PER_DISK_PUBLICATION_BOUND,
            Arm::TightenedRoots | Arm::TightenedRootsAtMount => TIGHTENED_ROOTS_PUBLICATION_BOUND,
            Arm::TightenedStates => TIGHTENED_STATES_PUBLICATION_BOUND,
            Arm::ProtectedGenerationsAtMinimum => 1,
            Arm::FloorTxg => 1 + TXG_ARITHMETIC_GENERATIONS + 2,
        }
    }
    /// 预注册：保留池 = 上一个窗口的元数据 + 这次处置要拿的块 = 2 × 5 + (B − 1) × c。
    fn reserve_blocks(self, ring_depth: u64, formula_cost: u64) -> u64 {
        match self {
            Arm::RingPersisted | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount => {
                2 * METADATA_BLOCKS_PER_PUBLICATION + (self.publication_bound(ring_depth) - 1) * formula_cost
            }
            Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg => CONTROL_ARM_RESERVE_BLOCKS,
        }
    }
    /// 预注册：一次完整处置之后还扣着的最坏块数 = 5 + (B − 2) × c。
    fn push_residue_blocks(self, ring_depth: u64, formula_cost: u64) -> u64 {
        match self {
            Arm::RingPersisted | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount => {
                METADATA_BLOCKS_PER_PUBLICATION + (self.publication_bound(ring_depth) - 2) * formula_cost
            }
            Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg => 0,
        }
    }
    /// drain_failure 里失败位置扫到第几次根槽写（第 0 次是发布正在攒的窗口）。
    fn drain_failure_offset_limit(self, ring_depth: u64) -> u64 {
        match self {
            Arm::RingPersisted => ring_depth.min(DRAIN_FAILURE_RING_ARM_OFFSET_LIMIT),
            Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg => {
                self.publication_bound(ring_depth)
            }
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

/// 活化区间一族世界的三种形态。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ActivationVariant {
    /// E138 原样：抬一次 F，写失败 × 崩溃窗口 × 丢盘。
    Plain,
    /// 第 j 个窗口崩溃不丢盘、恢复，再跑 2 个窗口，然后再崩一次并丢盘。
    CrashWithoutLossFirst,
    /// 抬 F 之前先让区域 1 的根槽连续写失败 r 次（跳号），再照 Plain 枚举。
    SkipThenRaise { repeats: u64 },
}

impl ActivationVariant {
    fn name(self) -> String {
        match self {
            ActivationVariant::Plain => "activation_faults".to_string(),
            ActivationVariant::CrashWithoutLossFirst => "activation_crash_no_loss".to_string(),
            ActivationVariant::SkipThenRaise { repeats } => format!("skip_then_raise_r{repeats}"),
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
    minimum_distinct_states: u64,
    distinct_state_sum: u64,
    stalls: u64,
    forced_publications: u64,
    user_windows: u64,
    admissions_under_pressure: u64,
    maximum_publications_per_admission: u64,
    false_enospc_raw: u64,
    false_enospc_arm: u64,
    true_enospc: u64,
    write_after_delete_failed: u64,
    first_write_after_delete_failure_window: u64,
    last_write_after_delete_failure_window: u64,
    halted_before_counting: u64,
    root_write_failures_injected: u64,
    lag_sum: u64,
    lag_maximum: u64,
    lag_samples: u64,
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
    }
    fn minimum_candidates_or_zero(&self) -> u64 {
        if self.minimum_candidates == u64::MAX { 0 } else { self.minimum_candidates }
    }
    fn minimum_distinct_states_or_zero(&self) -> u64 {
        if self.minimum_distinct_states == u64::MAX { 0 } else { self.minimum_distinct_states }
    }
    fn average_candidates_in_hundredths(&self) -> u64 {
        if self.checks == 0 { 0 } else { self.candidate_sum * 100 / self.checks }
    }
    fn average_nonempty_candidates_in_hundredths(&self) -> u64 {
        if self.checks == 0 { 0 } else { self.nonempty_candidate_sum * 100 / self.checks }
    }
    fn average_distinct_states_in_hundredths(&self) -> u64 {
        if self.checks == 0 { 0 } else { self.distinct_state_sum * 100 / self.checks }
    }
    fn forced_publications_per_window_in_hundredths(&self) -> u64 {
        if self.user_windows == 0 { 0 } else { self.forced_publications * 100 / self.user_windows }
    }
    fn average_lag_in_hundredths(&self) -> u64 {
        if self.lag_samples == 0 { 0 } else { self.lag_sum * 100 / self.lag_samples }
    }
    fn first_failure_window_or_zero(&self) -> u64 {
        if self.first_write_after_delete_failure_window == u64::MAX { 0 } else { self.first_write_after_delete_failure_window }
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
    /// 现在每次空发布写几块（cost_growth 世界里会变）。
    empty_publication_cost: u64,
    /// 保留池与残留按这条臂的式子算（被测臂按声明的最坏值 c_max，见 Arm::formula_cost）。
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
    /// 第几次根槽写要失败（按根槽写的绝对序号，重发也算一次）。
    planned_root_write_failures: BTreeSet<u64>,
    root_write_attempts: u64,
    /// broken_slot 世界：这个槽从此每次写都失败。
    broken_slot: Option<usize>,
    current_window_changes_user_state: bool,
    /// 非空发布的 txg：空发布与推空的根不改用户可见状态。mkfs 的第 0 代算一个状态。
    nonempty_txgs: HashSet<u64>,
    /// 空发布的 txg：它们放掉的只有上一次的元数据，是推空自己的残留，不算滞后量。
    empty_txgs: HashSet<u64>,
    admin_rollback_happened: bool,
    halted: bool,
    /// 近满盘循环里数到第几个窗口，给「删了再写失败」记窗口号。
    counting_window: u64,
    metrics: Metrics,
    /// 管理员回退之前盘上那一份分配状态，被抛弃时间线的根引用的是它（E135 的 abandoned_view 原样）。
    abandoned_view: Option<(Rc<PoolState>, u64)>,
}

impl Simulation {
    fn new(arm: Arm, slots_per_region: u64, mount_cost: u64, declared_maximum_cost: u64, seed: u64, keep_snapshots: bool, reserve_override: Option<u64>) -> Self {
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
            keep_snapshots,
            tear_next_publication: false,
            fail_next_slot_write: false,
            planned_root_write_failures: BTreeSet::new(),
            root_write_attempts: 0,
            broken_slot: None,
            current_window_changes_user_state: false,
            nonempty_txgs,
            empty_txgs: HashSet::new(),
            admin_rollback_happened: false,
            halted: false,
            counting_window: 0,
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

    fn region_of_txg(txg: u64) -> u64 {
        txg % REGION_COUNT
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
    fn fourth_newest_nonempty_valid_txg(&self) -> u64 {
        let distinct_newest_first: Vec<u64> = self.valid_roots().iter().map(|root| root.txg)
            .filter(|txg| self.nonempty_txgs.contains(txg)).collect::<BTreeSet<u64>>().into_iter().rev().collect();
        if distinct_newest_first.len() >= MINIMUM_RETAINED_ROOTS {
            distinct_newest_first[MINIMUM_RETAINED_ROOTS - 1]
        } else {
            self.oldest_persisted_valid_txg()
        }
    }

    /// 用户定案的上限：min(每块幸存盘上最新的持久有效根, 第 4 新的持久有效根)。
    fn per_disk_retention_upper_limit(&self) -> u64 {
        self.fourth_newest_valid_txg().min(self.newest_valid_txg_on_every_surviving_disk())
    }

    /// 用户 2026-09-12 口径的上限：min(每块幸存盘上最新的持久有效根, 第 4 新的非空持久有效根)。
    fn per_disk_states_upper_limit(&self) -> u64 {
        self.fourth_newest_nonempty_valid_txg().min(self.newest_valid_txg_on_every_surviving_disk())
    }

    /// 这条臂抬 F 的上限。
    fn floor_upper_limit(&self) -> u64 {
        match self.arm {
            Arm::FloorTxg => (self.newest_durable_txg + 1).saturating_sub(TXG_ARITHMETIC_GENERATIONS),
            Arm::FloorPerDiskTxgUpperLimit => (self.newest_durable_txg + 1).saturating_sub(TXG_UPPER_LIMIT_GENERATIONS),
            Arm::FloorPerDisk | Arm::TightenedRoots | Arm::TightenedRootsAtMount => self.per_disk_retention_upper_limit(),
            Arm::TightenedStates => self.per_disk_states_upper_limit(),
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum => 0,
        }
    }

    /// 处置的目标：按状态数读时不越过第 4 新的非空根（非空有效根不足 4 个时照目标本身）。
    fn disposal_target(&self, goal_txg: u64) -> u64 {
        if self.arm.counts_states() {
            let nonempty_count = self.valid_roots().iter().filter(|root| self.nonempty_txgs.contains(&root.txg)).map(|root| root.txg).collect::<BTreeSet<u64>>().len();
            if nonempty_count >= MINIMUM_RETAINED_ROOTS { goal_txg.min(self.fourth_newest_nonempty_valid_txg()) } else { goal_txg }
        } else {
            goal_txg
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
            Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount => {
                self.floor_active.max(self.oldest_persisted_valid_txg())
            }
        }
    }

    fn count_freed_after(&self, bound: u64) -> u64 {
        self.pool.blocks.iter().filter(|state| match state {
            BlockState::Unused | BlockState::Live { .. } => false,
            BlockState::Freed { freed_at, .. } => *freed_at > bound,
        }).count() as u64
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
        self.count_freed_after(self.reuse_bound())
    }

    /// 按状态数读时一次处置也放不回的已释放块（滞后量）：释放代 > max(第 4 新的非空根, 环里最旧有效根)，
    /// 且不是空发布放掉的（那是推空自己的残留，另外按最坏值扣）。正在攒的窗口放掉的也算在内。别的臂是 0。
    fn lag_blocks(&self) -> u64 {
        if self.arm.counts_states() {
            let bound = self.fourth_newest_nonempty_valid_txg().max(self.oldest_persisted_valid_txg());
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

    /// (假候选数, 候选数, 其中非空发布的根数, 其中不同的用户可见状态数)。
    /// 状态数：非空候选各算一个；候选集里比最旧非空候选还旧的空根共享同一个更早的状态，再算一个。
    fn count_fake_candidates(&self) -> (u64, u64, u64, u64) {
        let floor = self.candidacy_floor();
        let candidates: Vec<RootRecord> = self.valid_roots().into_iter().filter(|root| root.txg >= floor).collect();
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
        let floor = self.candidacy_floor();
        if self.is_any_disk_lost() {
            self.surviving_disks().iter().any(|disk| self.newest_valid_txg_on_disk(*disk).is_some_and(|newest| newest < floor))
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

    /// 发布 txg 为 t 的根。根槽写失败时 txg 推进一格再发（D23（journal 的角色与格式） 已定项 14 的索引行），写失败的槽留着旧内容；重发不再分配块。
    fn publish(&mut self, txg: u64) {
        let slot = self.slot_index(txg);
        let attempt = self.root_write_attempts;
        self.root_write_attempts += 1;
        let is_planned_failure = self.planned_root_write_failures.remove(&attempt);
        if self.fail_next_slot_write || is_planned_failure || self.broken_slot == Some(slot) {
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
                Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount => {
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

    fn maximum_empty_publications(&self) -> u64 {
        match self.arm {
            Arm::RingPersisted => self.ring_depth - 1,
            Arm::ProtectedGenerationsAtMinimum => 0,
            Arm::FloorTxg => TXG_ARITHMETIC_GENERATIONS + 2,
            Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount => {
                self.arm.publication_bound(self.ring_depth) - 1
            }
        }
    }

    /// 发布正在攒的窗口之后按臂处置，返回做了几次空发布。
    /// stop_when_writable = false 时一直做到目标放回为止（判据 1 的探针用）。
    /// 收严臂：目标放回了就停（再推也只是把自己的元数据扣进 F 之上）。
    fn push_after_commit(&mut self, goal_txg: u64, stop_when_writable: bool) -> u64 {
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
                Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount => {
                    (stop_when_writable && self.user_allocatable() >= USER_BLOCKS) || self.is_goal_released(target)
                }
            };
            if is_done {
                break;
            }
            match self.arm {
                Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount => {
                    // 等上限够到目标才抬、一次抬到目标（预注册：每次都抬到当时的上限会让带新 F 的根反复重来）
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
    fn forced_pressure(&mut self) {
        if !self.arm.uses_floor() {
            return;
        }
        let maximum_empty_publications = self.maximum_empty_publications();
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
        let lag = self.lag_blocks();
        self.metrics.lag_sum += lag;
        self.metrics.lag_maximum = self.metrics.lag_maximum.max(lag);
        self.metrics.lag_samples += 1;
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
    fn run_counted_delete_create_window(&mut self) -> WindowOutcome {
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
            Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount => {
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
    let (mount_cost, declared_maximum_cost) = COST_PAIRS[0];
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, keep_snapshots, None);
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

/// 活化一族世界的前缀：跑到第 E 个窗口；skip_then_raise 先让区域 1 的根槽连续写失败 r 次；然后 F 臂只抬一次 F（不推空）。
fn activation_prefix(arm: Arm, slots_per_region: u64, seed: u64, skip_repeats: u64) -> Option<Simulation> {
    let (mount_cost, declared_maximum_cost) = COST_PAIRS[0];
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, true, None);
    simulation.prefill(STEADY_FILL);
    let (event_window, _) = schedule(seed, simulation.ring_depth, World::Steady);
    for _ in 0..event_window {
        simulation.run_user_window(true, false);
    }
    let mut remaining = skip_repeats;
    while remaining > 0 && !simulation.halted {
        if Simulation::region_of_txg(simulation.next_txg) == 1 {
            simulation.fail_next_slot_write = true;
            remaining -= 1;
        }
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

fn run_activation_case(prefix: &Simulation, pattern: &[u64], crash_window: u64, lost_disk: usize, variant: ActivationVariant) -> Metrics {
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
    match variant {
        ActivationVariant::Plain | ActivationVariant::SkipThenRaise { .. } => {}
        ActivationVariant::CrashWithoutLossFirst => {
            if !simulation.halted {
                simulation.crash_and_recover();
            }
            for _ in 0..WINDOWS_BETWEEN_CRASHES {
                simulation.run_user_window(true, false);
            }
            let _ = simulation.run_user_window(true, true);
        }
    }
    if !simulation.halted {
        simulation.disk_lost[lost_disk] = true;
        simulation.crash_and_recover();
    }
    simulation.metrics
}

/// near_full：预热 2N + 10 个窗口，只建不删直到第一次 ENOSPC，再做 400 个「删一个、建一个」。
fn run_near_full(arm: Arm, slots_per_region: u64, mount_cost: u64, declared_maximum_cost: u64, seed: u64, reserve_override: Option<u64>) -> Metrics {
    let mut simulation = fill_to_enospc(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, reserve_override);
    for _ in 0..WINDOWS_PER_WORLD {
        if simulation.halted {
            break;
        }
        simulation.run_counted_delete_create_window();
    }
    simulation.metrics
}

/// 近满盘一族的共同前缀：预热、填满到第一次 ENOSPC；计数从填满之后开始。
fn fill_to_enospc(arm: Arm, slots_per_region: u64, mount_cost: u64, declared_maximum_cost: u64, seed: u64, reserve_override: Option<u64>) -> Simulation {
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, false, reserve_override);
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
    simulation.counting_window = 0;
    simulation
}

/// cost_growth：near_full 的删建循环做到第 100 个窗口时空发布开销升到 grown_cost，再做 300 个窗口。
fn run_cost_growth(arm: Arm, slots_per_region: u64, mount_cost: u64, declared_maximum_cost: u64, seed: u64, grown_cost: u64) -> Metrics {
    let mut simulation = fill_to_enospc(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, None);
    for window in 0..WINDOWS_PER_WORLD {
        if simulation.halted {
            break;
        }
        if window == COST_GROWTH_AT_WINDOW {
            simulation.empty_publication_cost = grown_cost;
        }
        simulation.run_counted_delete_create_window();
    }
    simulation.metrics
}

/// drain_failure 的前缀：填满、再做 5 个删建窗口，计数清零。
fn drain_failure_prefix(arm: Arm, slots_per_region: u64, mount_cost: u64, declared_maximum_cost: u64, seed: u64) -> Option<Simulation> {
    let mut simulation = fill_to_enospc(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, None);
    for _ in 0..DRAIN_FAILURE_WINDOWS_BEFORE {
        simulation.run_user_window(true, false);
    }
    if simulation.halted {
        return None;
    }
    simulation.metrics = Metrics::fresh();
    simulation.counting_window = 0;
    Some(simulation)
}

/// drain_failure 的写失败组合：单个、两个（位置 ≤ limit），三个只取前几种。
fn drain_failure_patterns(offset_limit: u64) -> Vec<Vec<u64>> {
    let mut patterns = Vec::new();
    for first in 0..=offset_limit {
        patterns.push(vec![first]);
    }
    for first in 0..=offset_limit {
        for second in (first + 1)..=offset_limit {
            patterns.push(vec![first, second]);
        }
    }
    let mut triples = 0usize;
    'outer: for first in 0..=offset_limit {
        for second in (first + 1)..=offset_limit {
            for third in (second + 1)..=offset_limit {
                if triples >= DRAIN_FAILURE_TRIPLE_PATTERNS {
                    break 'outer;
                }
                patterns.push(vec![first, second, third]);
                triples += 1;
            }
        }
    }
    patterns
}

/// drain_failure 的一格：下一次准入处置里第 k 次根槽写失败（k 按组合），之后再做 20 个删建窗口。
fn run_drain_failure_case(prefix: &Simulation, pattern: &[u64]) -> Metrics {
    let mut simulation = prefix.clone();
    let base = simulation.root_write_attempts;
    for offset in pattern {
        simulation.planned_root_write_failures.insert(base + offset);
    }
    for _ in 0..DRAIN_FAILURE_WINDOWS_AFTER {
        if simulation.halted {
            break;
        }
        simulation.run_counted_delete_create_window();
    }
    simulation.metrics
}

/// broken_slot：占用 70% 预热 2N + 10 个窗口之后，槽 1 从此每次写都失败；再做 1000 个删建窗口。
fn run_broken_slot(arm: Arm, slots_per_region: u64, mount_cost: u64, declared_maximum_cost: u64, seed: u64) -> Metrics {
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, false, None);
    simulation.prefill(STEADY_FILL);
    let warm_up = 2 * simulation.ring_depth + 10;
    for _ in 0..warm_up {
        simulation.run_user_window(true, false);
    }
    simulation.metrics = Metrics::fresh();
    simulation.counting_window = 0;
    simulation.broken_slot = Some(BROKEN_SLOT);
    for _ in 0..BROKEN_SLOT_WINDOWS {
        if simulation.halted {
            break;
        }
        simulation.run_counted_delete_create_window();
    }
    simulation.metrics
}

/// 判据 1（跨装置闸）：steady 下最后一个窗口，在发布之前与发布刚完成各量一次。
fn steady_probe(arm: Arm, slots_per_region: u64) -> (u64, u64, u64, u64) {
    let (mount_cost, declared_maximum_cost) = COST_PAIRS[0];
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, 1, false, None);
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

/// 判据 1（跑前式子）：从 steady 触发一次完整的有界处置，做到目标放回为止。
/// 返回（用了几次发布，含发布正在攒的窗口；之后还扣着几块，不含滞后量；滞后量；目标放回了没有）。
fn push_probe(arm: Arm, slots_per_region: u64, cost: u64, extra_windows: u64) -> (u64, u64, u64, bool) {
    let mut simulation = Simulation::new(arm, slots_per_region, cost, cost, 1, false, None);
    simulation.prefill(STEADY_FILL);
    for _ in 0..(200 + extra_windows) {
        simulation.run_user_window(true, false);
    }
    let open_txg = simulation.next_txg;
    let victim = simulation.pool.objects.swap_remove(0);
    simulation.free_blocks(&victim, open_txg);
    let old_metadata = std::mem::take(&mut simulation.pool.metadata);
    simulation.free_blocks(&old_metadata, open_txg);
    // 这个窗口删了一个对象，发布出来的根改了用户可见状态
    simulation.current_window_changes_user_state = true;
    assert!(simulation.commit_open_window(open_txg), "steady 下发布正在攒的窗口不该卡死");
    let goal_txg = simulation.newest_durable_txg;
    let target = simulation.disposal_target(goal_txg);
    let publications = 1 + simulation.push_after_commit(goal_txg, false);
    let lag = simulation.lag_blocks();
    (publications, simulation.pinned_count() - lag, lag, simulation.is_goal_released(target))
}

/// 预注册判据 1 的期望：(允许的发布数集合, 最大发布数, 最大扣住块数（不含滞后量）, 滞后量)。
fn expected_push(arm: Arm, slots_per_region: u64, cost: u64) -> (Vec<u64>, u64, u64, u64) {
    let ring_depth = REGION_COUNT * slots_per_region;
    if slots_per_region == 1 {
        return match arm {
            // 环里只有 3 个根，环一转就把目标放回了；按状态数读时非空有效根不足 4 个，滞后量就是环之外的那些
            Arm::TightenedStates => (vec![3], 3, METADATA_BLOCKS_PER_PUBLICATION + cost, 0),
            Arm::RingPersisted | Arm::FloorPerDisk | Arm::TightenedRoots => (vec![3], 3, METADATA_BLOCKS_PER_PUBLICATION + cost, 0),
            Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRootsAtMount => (Vec::new(), 0, 0, 0),
        };
    }
    match arm {
        Arm::RingPersisted => (vec![ring_depth], ring_depth, METADATA_BLOCKS_PER_PUBLICATION + (ring_depth - 2) * cost, 0),
        Arm::FloorPerDisk | Arm::TightenedRoots => (vec![6, 7], 7, METADATA_BLOCKS_PER_PUBLICATION + 5 * cost, 0),
        // 目标是第 4 新的非空根（3 个窗口之前），发布正在攒的窗口之后立刻抬，带新 F 的第一个根 t + 1，最多再 2 次两块盘都有；
        // 滞后量 = t 与前两个非空窗口各放掉的 8 块对象 + 5 块元数据
        Arm::TightenedStates => (vec![3, 4], 4, METADATA_BLOCKS_PER_PUBLICATION + 2 * cost, 3 * BLOCKS_PER_PUBLICATION),
        Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRootsAtMount => (Vec::new(), 0, 0, 0),
    }
}

/// 三种 t 的落点（t mod 3）各做一次处置探针：(发布数集合, 最大发布数, 最大扣住块数, 最大滞后量, 全部放回)。
fn push_probe_over_landings(arm: Arm, slots_per_region: u64, cost: u64) -> (Vec<u64>, u64, u64, u64, bool) {
    let results: Vec<(u64, u64, u64, bool)> = (0..REGION_COUNT).map(|extra| push_probe(arm, slots_per_region, cost, extra)).collect();
    let publication_set: Vec<u64> = results.iter().map(|(publications, _, _, _)| *publications).collect::<BTreeSet<u64>>().into_iter().collect();
    let maximum_publications = results.iter().map(|(publications, _, _, _)| *publications).max().unwrap_or(0);
    let maximum_pinned = results.iter().map(|(_, pinned, _, _)| *pinned).max().unwrap_or(0);
    let maximum_lag = results.iter().map(|(_, _, lag, _)| *lag).max().unwrap_or(0);
    let is_all_released = results.iter().all(|(_, _, _, is_released)| *is_released);
    (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released)
}

fn format_set(values: &[u64]) -> String {
    values.iter().map(|value| value.to_string()).collect::<Vec<String>>().join("/")
}

fn candidate_index(arm: Arm) -> Option<usize> {
    match arm {
        Arm::TightenedRoots => Some(0),
        Arm::TightenedStates => Some(1),
        Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRootsAtMount => None,
    }
}

/// 判据 3–8 按格累计的账。
#[derive(Clone, Copy, Debug)]
struct CandidateLedger {
    fakes: u64,
    unrecoverable: u64,
    retention_violations: u64,
    false_enospc_cells: u64,
    write_after_delete_cells: u64,
    bound_exceeded_cells: u64,
    stall_cells: u64,
    maximum_publications_within_tolerance: u64,
    maximum_publications_beyond_tolerance: u64,
}

impl CandidateLedger {
    fn fresh() -> Self {
        CandidateLedger {
            fakes: 0,
            unrecoverable: 0,
            retention_violations: 0,
            false_enospc_cells: 0,
            write_after_delete_cells: 0,
            bound_exceeded_cells: 0,
            stall_cells: 0,
            maximum_publications_within_tolerance: 0,
            maximum_publications_beyond_tolerance: 0,
        }
    }
    fn add_correctness(&mut self, total: &Metrics) {
        self.fakes += total.fake_candidates;
        self.unrecoverable += total.unrecoverable;
        self.retention_violations += total.retention_violations;
    }
    /// 近满盘一族的一格：判据 4、5、6 按格记。
    fn add_space_cell(&mut self, total: &Metrics, bound: u64) {
        self.add_correctness(total);
        self.false_enospc_cells += u64::from(total.false_enospc_arm > 0);
        self.write_after_delete_cells += u64::from(total.write_after_delete_failed > 0);
        self.bound_exceeded_cells += u64::from(total.maximum_publications_per_admission > bound);
        self.stall_cells += u64::from(total.stalls > 0);
    }
}

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!(
        "name=config r={REGION_COUNT} disks={DISK_COUNT} k_min_txg={TXG_ARITHMETIC_GENERATIONS} retained={MINIMUM_RETAINED_ROOTS} \
         tolerated_failures={TOLERATED_ROOT_WRITE_FAILURES} bound_g={FLOOR_PER_DISK_PUBLICATION_BOUND} bound_g2r={TIGHTENED_ROOTS_PUBLICATION_BOUND} \
         bound_g2s={TIGHTENED_STATES_PUBLICATION_BOUND} user_blocks={USER_BLOCKS} metadata_blocks={METADATA_BLOCKS_PER_PUBLICATION} \
         capacity={CAPACITY_BLOCKS} windows={WINDOWS_PER_WORLD} seeds={SEED_COUNT} fault_seeds={FAULT_SEED_COUNT} sweep_seeds={RESERVE_SWEEP_SEED_COUNT} \
         cost_pairs={} block_bytes={BLOCK_BYTES}",
        COST_PAIRS.iter().map(|(mount, declared)| format!("{mount}:{declared}")).collect::<Vec<String>>().join("/")
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
    for arm in [Arm::RingPersisted, Arm::FloorPerDisk, Arm::TightenedRoots, Arm::TightenedStates] {
        for slots_per_region in SLOTS_PER_REGION_SWEEP {
            for (cost, _) in COST_PAIRS {
                let (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released) = push_probe_over_landings(arm, slots_per_region, cost);
                let (expect_set, expect_maximum, expect_pinned, expect_lag) = expected_push(arm, slots_per_region, cost);
                let holds = is_all_released && maximum_publications == expect_maximum && maximum_pinned == expect_pinned && maximum_lag == expect_lag
                    && publication_set.iter().all(|publications| expect_set.contains(publications));
                criterion_one_holds &= holds;
                println!("{}", emitter.emit_raw(&format!(
                    "name=crit1_push arm={} s={slots_per_region} empty_cost={cost} publications={} max_publications={maximum_publications} \
                     expect_publications={} expect_max={expect_maximum} max_pinned={maximum_pinned} expect_pinned={expect_pinned} \
                     lag={maximum_lag} expect_lag={expect_lag} released={} holds={}",
                    arm.name(), format_set(&publication_set), format_set(&expect_set), u8::from(is_all_released), u8::from(holds)
                )));
            }
        }
    }

    let mut ledgers = [CandidateLedger::fresh(); 2];

    // 故障世界（E138 原样）
    let mut first_publish_unrecoverable = [0u64; 8];
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
                         checks={} min_candidates={} avg_candidates_x100={} avg_nonempty_x100={} min_states={} avg_states_x100={} stalls={}",
                        arm.name(), world.name(), total.fake_candidates, total.unrecoverable, total.retention_violations, total.checks,
                        total.minimum_candidates_or_zero(), total.average_candidates_in_hundredths(),
                        total.average_nonempty_candidates_in_hundredths(), total.minimum_distinct_states_or_zero(),
                        total.average_distinct_states_in_hundredths(), total.stalls
                    )));
                    if world == World::RollbackFirstPublish {
                        first_publish_unrecoverable[arm_index] += total.unrecoverable;
                        continue;
                    }
                    if let Some(index) = candidate_index(*arm) {
                        ledgers[index].add_correctness(&total);
                    }
                }
            }
        }
    }

    // 活化一族：activation_faults（E138 原样）、activation_crash_no_loss、skip_then_raise
    let patterns = failure_patterns();
    let mut floor_txg_activation_fakes = 0u64;
    let mut txg_upper_limit_skip_retention = 0u64;
    let mut detail_lines = 0u64;
    let mut variants = vec![ActivationVariant::Plain, ActivationVariant::CrashWithoutLossFirst];
    for repeats in SKIP_THEN_RAISE_REPEATS {
        variants.push(ActivationVariant::SkipThenRaise { repeats });
    }
    for variant in variants {
        let skip_repeats = match variant {
            ActivationVariant::SkipThenRaise { repeats } => repeats,
            ActivationVariant::Plain | ActivationVariant::CrashWithoutLossFirst => 0,
        };
        for slots_per_region in FAULT_SLOTS_SWEEP {
            for arm in ARMS {
                let mut by_class = [[Metrics::fresh(); 3]; DISK_COUNT];
                for seed in 0..FAULT_SEED_COUNT {
                    let prefix = match activation_prefix(arm, slots_per_region, seed, skip_repeats) {
                        None => continue,
                        Some(simulation) => simulation,
                    };
                    for pattern in &patterns {
                        for crash_window in 1..=ACTIVATION_FAULT_CRASH_WINDOWS {
                            for lost_disk in 0..DISK_COUNT {
                                let case = run_activation_case(&prefix, pattern, crash_window, lost_disk, variant);
                                by_class[lost_disk][pattern.len()].add(&case);
                                if arm.is_candidate() && (case.fake_candidates > 0 || case.unrecoverable > 0 || case.retention_violations > 0) && detail_lines < DETAIL_LINES_MAXIMUM {
                                    detail_lines += 1;
                                    println!("{}", emitter.emit_raw(&format!(
                                        "name=detail_activation variant={} arm={} s={slots_per_region} seed={seed} failures={} crash_window={crash_window} \
                                         lost_disk={lost_disk} fakes={} unrecoverable={} retention_violations={}",
                                        variant.name(), arm.name(), format_set(pattern), case.fake_candidates, case.unrecoverable, case.retention_violations
                                    )));
                                }
                            }
                        }
                    }
                }
                for (lost_disk, classes) in by_class.iter().enumerate() {
                    for (failure_count, total) in classes.iter().enumerate() {
                        println!("{}", emitter.emit_raw(&format!(
                            "name=activation variant={} arm={} s={slots_per_region} lost_disk={lost_disk} failures={failure_count} fakes={} unrecoverable={} \
                             retention_violations={} checks={}",
                            variant.name(), arm.name(), total.fake_candidates, total.unrecoverable, total.retention_violations, total.checks
                        )));
                        if let Some(index) = candidate_index(arm) {
                            ledgers[index].add_correctness(total);
                        }
                        match (arm, variant) {
                            (Arm::FloorTxg, ActivationVariant::Plain) => floor_txg_activation_fakes += total.fake_candidates,
                            (Arm::FloorPerDiskTxgUpperLimit, ActivationVariant::SkipThenRaise { .. }) => txg_upper_limit_skip_retention += total.retention_violations,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // near_full（E138 原样，两档开销对）
    let mut floor_per_disk_near_full_write_after_delete_cost_five = 0u64;
    let mut ring_raw_false_enospc = 0u64;
    for slots_per_region in SLOTS_PER_REGION_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in ARMS {
                let ring_depth = REGION_COUNT * slots_per_region;
                let mut total = Metrics::fresh();
                let mut failure_windows: Vec<(u64, u64, u64)> = Vec::new();
                for seed in 0..SEED_COUNT {
                    let one = run_near_full(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, None);
                    if arm.is_candidate() && one.write_after_delete_failed > 0 {
                        failure_windows.push((seed, one.first_failure_window_or_zero(), one.last_write_after_delete_failure_window));
                    }
                    total.add(&one);
                }
                let formula_cost = arm.formula_cost(mount_cost, declared_maximum_cost);
                let reserve = arm.reserve_blocks(ring_depth, formula_cost);
                let residue = arm.push_residue_blocks(ring_depth, formula_cost);
                let bound = arm.publication_bound(ring_depth);
                println!("{}", emitter.emit_raw(&format!(
                    "name=near_full arm={} s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost} reserve={reserve} residue={residue} \
                     hidden_blocks={} hidden_bytes={} avg_lag_x100={} max_lag={} false_enospc_raw={} false_enospc_arm={} true_enospc={} \
                     write_after_delete_failed={} first_failure_window={} last_failure_window={} max_publications={} bound={bound} \
                     admissions_under_pressure={} forced_per_window_x100={} stalls={} halted_before_counting={} fakes={} unrecoverable={} \
                     retention_violations={} min_candidates={} avg_candidates_x100={} avg_nonempty_x100={} min_states={} avg_states_x100={}",
                    arm.name(), reserve + residue, (reserve + residue) * BLOCK_BYTES, total.average_lag_in_hundredths(), total.lag_maximum,
                    total.false_enospc_raw, total.false_enospc_arm, total.true_enospc, total.write_after_delete_failed,
                    total.first_failure_window_or_zero(), total.last_write_after_delete_failure_window, total.maximum_publications_per_admission,
                    total.admissions_under_pressure, total.forced_publications_per_window_in_hundredths(), total.stalls, total.halted_before_counting,
                    total.fake_candidates, total.unrecoverable, total.retention_violations, total.minimum_candidates_or_zero(),
                    total.average_candidates_in_hundredths(), total.average_nonempty_candidates_in_hundredths(),
                    total.minimum_distinct_states_or_zero(), total.average_distinct_states_in_hundredths()
                )));
                if !failure_windows.is_empty() {
                    println!("{}", emitter.emit_raw(&format!(
                        "name=near_full_failure_windows arm={} s={slots_per_region} c0={mount_cost} seeds_with_failure={} first_windows={} last_windows={}",
                        arm.name(), failure_windows.len(),
                        format_set(&failure_windows.iter().map(|(_, first, _)| *first).collect::<Vec<u64>>()),
                        format_set(&failure_windows.iter().map(|(_, _, last)| *last).collect::<Vec<u64>>())
                    )));
                }
                if arm == Arm::FloorPerDisk && mount_cost == 5 {
                    floor_per_disk_near_full_write_after_delete_cost_five += total.write_after_delete_failed;
                }
                if arm == Arm::RingPersisted {
                    ring_raw_false_enospc += total.false_enospc_raw;
                }
                if let Some(index) = candidate_index(arm) {
                    ledgers[index].add_space_cell(&total, bound);
                }
            }
        }
    }

    // drain_failure
    let mut ring_drain_false_enospc = 0u64;
    for slots_per_region in FAULT_SLOTS_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in NEAR_FULL_FAULT_ARMS {
                let ring_depth = REGION_COUNT * slots_per_region;
                let bound = arm.publication_bound(ring_depth);
                let drain_patterns = drain_failure_patterns(arm.drain_failure_offset_limit(ring_depth));
                let mut by_count = [Metrics::fresh(); 4];
                let mut cases_by_count = [0u64; 4];
                let mut runs_with_false = [0u64; 4];
                let mut runs_with_write_after_delete = [0u64; 4];
                for seed in 0..FAULT_SEED_COUNT {
                    let prefix = match drain_failure_prefix(arm, slots_per_region, mount_cost, declared_maximum_cost, seed) {
                        None => continue,
                        Some(simulation) => simulation,
                    };
                    for pattern in &drain_patterns {
                        let case = run_drain_failure_case(&prefix, pattern);
                        by_count[pattern.len()].add(&case);
                        cases_by_count[pattern.len()] += 1;
                        runs_with_false[pattern.len()] += u64::from(case.false_enospc_arm > 0);
                        runs_with_write_after_delete[pattern.len()] += u64::from(case.write_after_delete_failed > 0);
                        if arm.is_candidate() && (pattern.len() as u64) <= TOLERATED_ROOT_WRITE_FAILURES
                            && (case.false_enospc_arm > 0 || case.write_after_delete_failed > 0 || case.maximum_publications_per_admission > bound || case.stalls > 0)
                            && detail_lines < 2 * DETAIL_LINES_MAXIMUM
                        {
                            detail_lines += 1;
                            println!("{}", emitter.emit_raw(&format!(
                                "name=detail_drain arm={} s={slots_per_region} c0={mount_cost} seed={seed} failures={} false_enospc_arm={} \
                                 write_after_delete_failed={} max_publications={} stalls={}",
                                arm.name(), format_set(pattern), case.false_enospc_arm, case.write_after_delete_failed,
                                case.maximum_publications_per_admission, case.stalls
                            )));
                        }
                    }
                }
                for failure_count in 1..=3usize {
                    let total = by_count[failure_count];
                    println!("{}", emitter.emit_raw(&format!(
                        "name=drain_failure arm={} s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost} failures={failure_count} cases={} \
                         runs_with_false_enospc={} runs_with_write_after_delete_failed={} false_enospc_arm={} false_enospc_raw={} write_after_delete_failed={} \
                         max_publications={} bound={bound} stalls={} fakes={} unrecoverable={} retention_violations={} injected={}",
                        arm.name(), cases_by_count[failure_count], runs_with_false[failure_count], runs_with_write_after_delete[failure_count],
                        total.false_enospc_arm, total.false_enospc_raw, total.write_after_delete_failed, total.maximum_publications_per_admission,
                        total.stalls, total.fake_candidates, total.unrecoverable, total.retention_violations, total.root_write_failures_injected
                    )));
                    if arm == Arm::RingPersisted {
                        ring_drain_false_enospc += total.false_enospc_arm;
                    }
                    if let Some(index) = candidate_index(arm) {
                        if failure_count as u64 <= TOLERATED_ROOT_WRITE_FAILURES {
                            ledgers[index].add_space_cell(&total, bound);
                            ledgers[index].maximum_publications_within_tolerance =
                                ledgers[index].maximum_publications_within_tolerance.max(total.maximum_publications_per_admission);
                        } else {
                            ledgers[index].add_correctness(&total);
                            ledgers[index].maximum_publications_beyond_tolerance =
                                ledgers[index].maximum_publications_beyond_tolerance.max(total.maximum_publications_per_admission);
                        }
                    }
                }
            }
        }
    }

    // broken_slot
    let mut ring_broken_slot_false_or_stall = 0u64;
    for slots_per_region in FAULT_SLOTS_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in NEAR_FULL_FAULT_ARMS {
                let ring_depth = REGION_COUNT * slots_per_region;
                let bound = arm.publication_bound(ring_depth);
                let mut total = Metrics::fresh();
                let mut seeds_with_no_space = 0u64;
                let mut seeds_halted = 0u64;
                for seed in 0..FAULT_SEED_COUNT {
                    let one = run_broken_slot(arm, slots_per_region, mount_cost, declared_maximum_cost, seed);
                    seeds_with_no_space += u64::from(one.write_after_delete_failed > 0);
                    seeds_halted += u64::from(one.stalls > 0);
                    total.add(&one);
                }
                println!("{}", emitter.emit_raw(&format!(
                    "name=broken_slot arm={} s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost} seeds={FAULT_SEED_COUNT} seeds_with_no_space={seeds_with_no_space} \
                     seeds_halted={seeds_halted} first_failure_window={} false_enospc_arm={} false_enospc_raw={} write_after_delete_failed={} max_publications={} bound={bound} \
                     stalls={} fakes={} unrecoverable={} retention_violations={} injected={} forced_per_window_x100={} max_lag={}",
                    arm.name(), total.first_failure_window_or_zero(), total.false_enospc_arm, total.false_enospc_raw, total.write_after_delete_failed,
                    total.maximum_publications_per_admission, total.stalls, total.fake_candidates, total.unrecoverable, total.retention_violations,
                    total.root_write_failures_injected, total.forced_publications_per_window_in_hundredths(), total.lag_maximum
                )));
                if arm == Arm::RingPersisted {
                    ring_broken_slot_false_or_stall += total.false_enospc_arm + total.stalls;
                }
                if let Some(index) = candidate_index(arm) {
                    ledgers[index].add_space_cell(&total, bound);
                }
            }
        }
    }

    // cost_growth
    let mut mount_formula_growth_false_or_stall = 0u64;
    for slots_per_region in FAULT_SLOTS_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in COST_GROWTH_ARMS {
                let ring_depth = REGION_COUNT * slots_per_region;
                let bound = arm.publication_bound(ring_depth);
                for grown_cost in [declared_maximum_cost, declared_maximum_cost + COST_OVER_DECLARED_EXTRA] {
                    let is_within_declared = grown_cost <= declared_maximum_cost;
                    let mut total = Metrics::fresh();
                    for seed in 0..FAULT_SEED_COUNT {
                        total.add(&run_cost_growth(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, grown_cost));
                    }
                    println!("{}", emitter.emit_raw(&format!(
                        "name=cost_growth arm={} s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost} grown_to={grown_cost} within_declared={} \
                         false_enospc_arm={} false_enospc_raw={} true_enospc={} write_after_delete_failed={} first_failure_window={} max_publications={} bound={bound} \
                         stalls={} fakes={} unrecoverable={} retention_violations={} forced_per_window_x100={}",
                        arm.name(), u8::from(is_within_declared), total.false_enospc_arm, total.false_enospc_raw, total.true_enospc,
                        total.write_after_delete_failed, total.first_failure_window_or_zero(), total.maximum_publications_per_admission, total.stalls,
                        total.fake_candidates, total.unrecoverable, total.retention_violations, total.forced_publications_per_window_in_hundredths()
                    )));
                    if arm == Arm::TightenedRootsAtMount && is_within_declared {
                        mount_formula_growth_false_or_stall += total.false_enospc_arm + total.stalls;
                    }
                    if let Some(index) = candidate_index(arm) {
                        if is_within_declared {
                            ledgers[index].add_space_cell(&total, bound);
                        } else {
                            ledgers[index].add_correctness(&total);
                        }
                    }
                }
            }
        }
    }

    // 保留池扫描
    let mut minimum_no_stall_reserve = [[u64::MAX; 2]; 2];
    for (index, arm) in CANDIDATE_ARMS.iter().enumerate() {
        for (cost_index, (mount_cost, declared_maximum_cost)) in COST_PAIRS.iter().enumerate() {
            let ring_depth = REGION_COUNT * RESERVE_SWEEP_SLOTS_PER_REGION;
            let bound = arm.publication_bound(ring_depth);
            let formula = arm.reserve_blocks(ring_depth, arm.formula_cost(*mount_cost, *declared_maximum_cost));
            for reserve in (formula - RESERVE_SWEEP_BELOW)..=(formula + RESERVE_SWEEP_ABOVE) {
                let mut total = Metrics::fresh();
                for seed in 0..RESERVE_SWEEP_SEED_COUNT {
                    total.add(&run_near_full(*arm, RESERVE_SWEEP_SLOTS_PER_REGION, *mount_cost, *declared_maximum_cost, seed, Some(reserve)));
                }
                println!("{}", emitter.emit_raw(&format!(
                    "name=reserve_sweep arm={} s={RESERVE_SWEEP_SLOTS_PER_REGION} c0={mount_cost} c_max={declared_maximum_cost} reserve={reserve} formula={formula} \
                     stalls={} false_enospc_arm={} write_after_delete_failed={} max_publications={} fakes={}",
                    arm.name(), total.stalls, total.false_enospc_arm, total.write_after_delete_failed,
                    total.maximum_publications_per_admission, total.fake_candidates
                )));
                if total.stalls == 0 {
                    minimum_no_stall_reserve[index][cost_index] = minimum_no_stall_reserve[index][cost_index].min(reserve);
                }
                if reserve >= formula {
                    ledgers[index].add_space_cell(&total, bound);
                }
            }
        }
    }

    println!("{}", emitter.emit_raw(&format!(
        "name=verdict crit1_holds={} crit2_a_drain_false_enospc={ring_drain_false_enospc} crit2_a_broken_false_or_stall={ring_broken_slot_false_or_stall} \
         crit2_g_near_full_wad_cost5={floor_per_disk_near_full_write_after_delete_cost_five} crit2_g2r_mount_growth_false_or_stall={mount_formula_growth_false_or_stall} \
         crit2_d_activation_fakes={floor_txg_activation_fakes} crit2_gtxgcap_skip_retention={txg_upper_limit_skip_retention} crit2_a_raw_false_enospc={ring_raw_false_enospc}",
        u8::from(criterion_one_holds)
    )));
    for (index, arm) in CANDIDATE_ARMS.iter().enumerate() {
        let ledger = ledgers[index];
        println!("{}", emitter.emit_raw(&format!(
            "name=verdict_candidate arm={} crit3_fakes={} crit3_unrecoverable={} crit4_false_enospc_cells={} crit5_write_after_delete_cells={} \
             crit5_bound_exceeded_cells={} crit6_stall_cells={} crit7_retention={} crit8_max_publications_within_tolerance={} \
             beyond_tolerance_max_publications={} bound={}",
            arm.name(), ledger.fakes, ledger.unrecoverable, ledger.false_enospc_cells, ledger.write_after_delete_cells, ledger.bound_exceeded_cells,
            ledger.stall_cells, ledger.retention_violations, ledger.maximum_publications_within_tolerance, ledger.maximum_publications_beyond_tolerance,
            arm.publication_bound(REGION_COUNT * 16)
        )));
    }
    println!("{}", emitter.emit_raw(&format!(
        "name=crit6_min_no_stall_reserve g2r_pair0={} g2r_pair1={} g2s_pair0={} g2s_pair1={}",
        minimum_no_stall_reserve[0][0], minimum_no_stall_reserve[0][1], minimum_no_stall_reserve[1][0], minimum_no_stall_reserve[1][1]
    )));
    println!("{}", emitter.emit_raw(&format!(
        "name=crit9_c281 {}",
        ARMS.iter().enumerate().map(|(index, arm)| format!("{}={}", arm.name(), first_publish_unrecoverable[index])).collect::<Vec<String>>().join(" ")
    )));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regions_zero_and_two_live_on_disk_zero_region_one_on_disk_one() {
        let simulation = Simulation::new(Arm::RingPersisted, 4, 1, 5, 1, false, None);
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
        for (slots_per_region, cost, publications, pinned) in
            [(1u64, 1u64, 3u64, 6u64), (1, 5, 3, 10), (4, 1, 12, 15), (4, 5, 12, 55), (16, 1, 48, 51), (16, 5, 48, 235)]
        {
            let (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released) = push_probe_over_landings(Arm::RingPersisted, slots_per_region, cost);
            assert!(is_all_released, "S={slots_per_region} c={cost} 处置之后这次删的块必须放回");
            assert_eq!(publication_set, vec![publications], "S={slots_per_region} c={cost} 臂 A 每种落点都是 N 次发布");
            assert_eq!(maximum_publications, publications);
            assert_eq!(maximum_pinned, pinned, "S={slots_per_region} c={cost} 臂 A 扣住 5 + (N − 2) × c");
            assert_eq!(maximum_lag, 0, "臂 A 没有滞后量");
            if slots_per_region > 1 {
                assert_eq!(maximum_pinned, Arm::RingPersisted.push_residue_blocks(REGION_COUNT * slots_per_region, cost), "残留式子与实测一致");
            }
        }
    }

    #[test]
    fn roots_arms_push_use_six_or_seven_publications_and_leave_preregistered_residue() {
        for arm in [Arm::FloorPerDisk, Arm::TightenedRoots] {
            for (slots_per_region, cost, publications, maximum, pinned) in [
                (1u64, 1u64, vec![3u64], 3u64, 6u64),
                (1, 5, vec![3], 3, 10),
                (4, 1, vec![6, 7], 7, 10),
                (4, 5, vec![6, 7], 7, 30),
                (16, 1, vec![6, 7], 7, 10),
                (16, 5, vec![6, 7], 7, 30),
            ] {
                let (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released) = push_probe_over_landings(arm, slots_per_region, cost);
                assert!(is_all_released, "{} S={slots_per_region} c={cost} 处置之后这次删的块必须放回", arm.name());
                assert_eq!(publication_set, publications, "{} S={slots_per_region} c={cost} 三种落点的发布数", arm.name());
                assert_eq!(maximum_publications, maximum, "{} S={slots_per_region} c={cost} 最多 7 次", arm.name());
                assert_eq!(maximum_pinned, pinned, "{} S={slots_per_region} c={cost} 扣住最多 5 + 5c（S = 1 时 5 + c）", arm.name());
                assert_eq!(maximum_lag, 0, "按根数读没有滞后量");
                if slots_per_region > 1 && arm == Arm::FloorPerDisk {
                    assert_eq!(maximum_pinned, Arm::FloorPerDisk.push_residue_blocks(REGION_COUNT * slots_per_region, cost), "E138 臂 G 的残留式子与实测一致（G2R 的式子按 B_R = 11 留了余量，比实测大）");
                }
            }
        }
    }

    /// 判据 1 里 G2S 那一行：目标是第 4 新的非空根，3 或 4 次发布放回；滞后量 = 3 个非空窗口各放掉的 13 块。
    #[test]
    fn states_arm_push_uses_three_or_four_publications_and_lags_three_windows() {
        for (slots_per_region, cost, publications, maximum, pinned, lag) in [
            (1u64, 1u64, vec![3u64], 3u64, 6u64, 0u64),
            (1, 5, vec![3], 3, 10, 0),
            (4, 1, vec![3, 4], 4, 7, 39),
            (4, 5, vec![3, 4], 4, 15, 39),
            (16, 1, vec![3, 4], 4, 7, 39),
            (16, 5, vec![3, 4], 4, 15, 39),
        ] {
            let (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released) = push_probe_over_landings(Arm::TightenedStates, slots_per_region, cost);
            assert!(is_all_released, "S={slots_per_region} c={cost} 处置之后目标必须放回");
            assert_eq!(publication_set, publications, "S={slots_per_region} c={cost} 三种落点的发布数");
            assert_eq!(maximum_publications, maximum, "S={slots_per_region} c={cost} 最多 4 次");
            assert_eq!(maximum_pinned, pinned, "S={slots_per_region} c={cost} 滞后量之外扣住最多 5 + 2c");
            assert_eq!(maximum_lag, lag, "S={slots_per_region} c={cost} 滞后量是 3 × 13");
        }
    }

    fn activation_totals(arm: Arm, slots_per_region: u64, seeds: u64, variant: ActivationVariant, failure_count: Option<usize>) -> Metrics {
        let skip_repeats = match variant {
            ActivationVariant::SkipThenRaise { repeats } => repeats,
            ActivationVariant::Plain | ActivationVariant::CrashWithoutLossFirst => 0,
        };
        let mut total = Metrics::fresh();
        for seed in 0..seeds {
            let prefix = activation_prefix(arm, slots_per_region, seed, skip_repeats).expect("steady 前缀不该卡死");
            for pattern in failure_patterns().iter().filter(|pattern| failure_count.is_none_or(|count| pattern.len() == count)) {
                for crash_window in 1..=ACTIVATION_FAULT_CRASH_WINDOWS {
                    for lost_disk in 0..DISK_COUNT {
                        total.add(&run_activation_case(&prefix, pattern, crash_window, lost_disk, variant));
                    }
                }
            }
        }
        total
    }

    #[test]
    fn floor_txg_produces_fake_candidates_when_faults_hit_activation_interval() {
        let total = activation_totals(Arm::FloorTxg, 4, 2, ActivationVariant::Plain, None);
        assert!(total.fake_candidates > 0, "按 txg 算活化，活化区间里写失败再丢盘必须冒出假候选");
    }

    #[test]
    fn protected_generations_runtime_is_unrecoverable_when_a_jump_is_followed_by_disk_loss() {
        let total = activation_totals(Arm::ProtectedGenerationsAtMinimum, 4, 2, ActivationVariant::Plain, None);
        assert!(total.unrecoverable > 0, "K = 3 在跳号下不够：写失败 + 丢盘必须出不可恢复");
    }

    /// 判据 2 的靶子：先跳号再抬 F，按 txg 数 4 代的上限越过盘 1 上最新的根（E138 判「未验」的那一半）。
    #[test]
    fn txg_upper_limit_control_violates_retention_when_floor_is_raised_after_the_jump() {
        let total = activation_totals(Arm::FloorPerDiskTxgUpperLimit, 4, 2, ActivationVariant::SkipThenRaise { repeats: 2 }, Some(0));
        assert!(total.retention_violations > 0, "先跳号再抬 F 时按 txg 数 4 代的上限必须越过盘 1 上最新的根");
    }

    #[test]
    fn tightened_arms_are_clean_across_activation_variants() {
        for arm in CANDIDATE_ARMS {
            for variant in [ActivationVariant::Plain, ActivationVariant::CrashWithoutLossFirst, ActivationVariant::SkipThenRaise { repeats: 2 }] {
                for slots_per_region in [4u64, 16] {
                    let total = activation_totals(arm, slots_per_region, 2, variant, None);
                    assert_eq!(total.fake_candidates, 0, "{} {} S={slots_per_region} 不许有假候选", arm.name(), variant.name());
                    assert_eq!(total.unrecoverable, 0, "{} {} S={slots_per_region} 不许不可恢复", arm.name(), variant.name());
                    assert_eq!(total.retention_violations, 0, "{} {} S={slots_per_region} 不许越过用户口径", arm.name(), variant.name());
                }
            }
        }
    }

    #[test]
    fn tightened_arms_are_clean_in_fault_worlds() {
        for arm in CANDIDATE_ARMS {
            for world in [World::TornNewest, World::DiskLoss, World::SlotWriteFail, World::AdminRollback] {
                // S = 1 也要跑：非空有效根不足 4 个的那条分支只在 3 槽的环上走到（第一次运行 G2S 在这里 41 次违反最少保留）
                for slots_per_region in [1u64, 4, 16] {
                    for lost_disk in 0..DISK_COUNT {
                        let total = (0..6).fold(Metrics::fresh(), |mut sum, seed| {
                            sum.add(&run_world(arm, world, slots_per_region, seed, lost_disk));
                            sum
                        });
                        assert_eq!(total.fake_candidates, 0, "{} {} S={slots_per_region} 丢盘 {lost_disk} 不许有假候选", arm.name(), world.name());
                        assert_eq!(total.unrecoverable, 0, "{} {} S={slots_per_region} 丢盘 {lost_disk} 不许不可恢复", arm.name(), world.name());
                        assert_eq!(total.retention_violations, 0, "{} {} S={slots_per_region} 丢盘 {lost_disk} 不许违反最少保留", arm.name(), world.name());
                    }
                }
            }
        }
    }

    /// 判据 2 的靶子与判据 5：E138 的臂 G 在 c₀ = 5 近满盘删了再写要失败，按最坏残留收费的 G2R 不失败。
    #[test]
    fn charging_worst_residue_removes_write_after_delete_failure_at_cost_five() {
        let control = run_near_full(Arm::FloorPerDisk, 4, 5, 10, 0, None);
        assert!(control.write_after_delete_failed > 0, "E138 的臂 G 在 S=4 c₀=5 种子 0 删了再写要失败（判据 2 的靶子）");
        assert!(control.true_enospc > 0, "填满阶段必须真的走到 ENOSPC");
        let tightened = run_near_full(Arm::TightenedRoots, 4, 5, 10, 0, None);
        assert_eq!(tightened.write_after_delete_failed, 0, "G2R 按最坏残留收费之后删了再写不许失败");
        assert_eq!(tightened.false_enospc_arm, 0, "G2R 不许有假性 ENOSPC");
        assert!(tightened.maximum_publications_per_admission <= TIGHTENED_ROOTS_PUBLICATION_BOUND, "G2R 一次准入不超过 B_R");
        assert_eq!(tightened.stalls, 0, "跑前保留池下不许卡死");
        assert!(tightened.true_enospc > 0, "填满阶段必须真的走到 ENOSPC，否则上面几条什么都没测");
    }

    /// 两种读法的差别落成数：按根数读，推空之后候选集只剩 1 个状态；按状态数读永远不少于 4 个。
    #[test]
    fn states_arm_keeps_four_distinct_states_where_roots_arm_drops_to_one() {
        let roots = run_near_full(Arm::TightenedRoots, 4, 5, 10, 0, None);
        assert_eq!(roots.minimum_distinct_states_or_zero(), 1, "按根数读，推空之后候选集里只剩这次删除那一个状态");
        assert!(roots.minimum_candidates_or_zero() >= 4, "但按根数数它仍然不少于 4 个根");
        let states = run_near_full(Arm::TightenedStates, 4, 5, 10, 0, None);
        assert!(states.minimum_distinct_states_or_zero() >= 4, "按状态数读，候选集里任何时候都不少于 4 个不同状态");
        assert_eq!(states.retention_violations, 0, "按状态数读不许违反用户口径");
    }

    /// 判据 2 的靶子与判据 4：坏槽之下臂 A 出假性 ENOSPC 或卡死，两条收严臂都不出。
    #[test]
    fn broken_slot_breaks_ring_arm_but_not_tightened_arms() {
        let ring = run_broken_slot(Arm::RingPersisted, 4, 1, 5, 0);
        assert!(ring.false_enospc_arm + ring.stalls > 0, "坏槽把臂 A 的界冻住，必须出假性 ENOSPC 或卡死（判据 2 的靶子）");
        for arm in CANDIDATE_ARMS {
            let total = run_broken_slot(arm, 4, 1, 5, 0);
            assert_eq!(total.false_enospc_arm, 0, "{} 坏槽之下不许有假性 ENOSPC", arm.name());
            assert_eq!(total.write_after_delete_failed, 0, "{} 坏槽之下删了再写不许失败", arm.name());
            assert_eq!(total.stalls, 0, "{} 坏槽之下不许卡死", arm.name());
            assert!(total.root_write_failures_injected > 0, "{} 坏槽必须真的被写到过，否则上面什么都没测", arm.name());
        }
    }

    /// 判据 2 的靶子与判据 4、8：推空期间一次根槽写失败，臂 A 出假性 ENOSPC；G2R 在容忍次数之内不出、发布数不超过 B_R。
    #[test]
    fn drain_failure_breaks_ring_arm_and_stays_within_tightened_bound() {
        let ring_prefix = drain_failure_prefix(Arm::RingPersisted, 4, 5, 10, 0).expect("填满不该卡死");
        let ring = run_drain_failure_case(&ring_prefix, &[1]);
        assert!(ring.false_enospc_arm > 0, "推空第 1 次空发布的根槽写失败，臂 A 必须出假性 ENOSPC（第三轮反推腿 R1）");
        for arm in CANDIDATE_ARMS {
            let prefix = drain_failure_prefix(arm, 4, 5, 10, 0).expect("填满不该卡死");
            let bound = arm.publication_bound(12);
            for pattern in drain_failure_patterns(arm.drain_failure_offset_limit(12)).iter().filter(|pattern| pattern.len() <= 2) {
                let case = run_drain_failure_case(&prefix, pattern);
                assert_eq!(case.false_enospc_arm, 0, "{} 失败 {:?} 之下不许有假性 ENOSPC", arm.name(), pattern);
                assert!(case.maximum_publications_per_admission <= bound, "{} 失败 {:?} 之下发布数 {} 不许超过 B = {bound}", arm.name(), pattern, case.maximum_publications_per_admission);
                assert_eq!(case.stalls, 0, "{} 失败 {:?} 之下不许卡死", arm.name(), pattern);
                assert_eq!(case.fake_candidates, 0, "{} 失败 {:?} 之下不许有假候选", arm.name(), pattern);
            }
        }
    }

    /// 判据 2 的靶子与判据 4：保留池按挂载时的开销算，开销变大之后出假性 ENOSPC 或卡死；按声明的最坏值算的 G2R 不出。
    #[test]
    fn cost_growth_breaks_mount_formula_but_not_declared_maximum_formula() {
        let mount = run_cost_growth(Arm::TightenedRootsAtMount, 4, 1, 5, 0, 5);
        assert!(mount.false_enospc_arm + mount.stalls > 0, "按挂载时的开销算式子，开销升到 c_max 之后必须出假性 ENOSPC 或卡死（第三轮反推腿 R3）");
        let declared = run_cost_growth(Arm::TightenedRoots, 4, 1, 5, 0, 5);
        assert_eq!(declared.false_enospc_arm, 0, "按 c_max 算式子，开销升到 c_max 不许有假性 ENOSPC");
        assert_eq!(declared.write_after_delete_failed, 0, "按 c_max 算式子，开销升到 c_max 删了再写不许失败");
        assert_eq!(declared.stalls, 0, "按 c_max 算式子，开销升到 c_max 不许卡死");
    }

    /// 变异 M6 的敏感取样点（E138 原样）：新 F 只写在一块盘上时崩溃（两块盘都在），恢复后的生效值必须还是旧的。
    #[test]
    fn recovery_does_not_activate_floor_carried_on_only_one_disk() {
        let mut simulation = activation_prefix(Arm::TightenedRoots, 4, 0, 0).expect("steady 前缀不该卡死");
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

    /// 强制盘紧把 F 抬到上限，推空的空根进了环之后再抬一次：按状态数读的上限是第 4 新的非空根，抬完候选集里仍有 4 个不同状态；
    /// 按根数读的上限把空根也数进去，抬得更高（对照）。
    #[test]
    fn forced_pressure_keeps_four_states_for_states_arm() {
        let mut floors = Vec::new();
        for arm in [Arm::TightenedStates, Arm::TightenedRoots] {
            let mut simulation = Simulation::new(arm, 4, 1, 5, 3, false, None);
            simulation.prefill(STEADY_FILL);
            for _ in 0..100 {
                simulation.run_user_window(true, false);
            }
            simulation.forced_pressure();
            assert_eq!(simulation.run_user_window(true, false), WindowOutcome::Created);
            simulation.forced_pressure();
            assert!(simulation.floor_carried > 0, "{} 强制盘紧要真的抬了 F", arm.name());
            if arm.counts_states() {
                assert!(simulation.floor_carried <= simulation.per_disk_states_upper_limit(), "F 不许越过按状态数读的上限");
                assert_eq!(simulation.floor_carried, simulation.fourth_newest_nonempty_valid_txg(), "上限就是第 4 新的非空根");
                let (_, _, _, distinct_states) = simulation.count_fake_candidates();
                assert!(distinct_states >= 4, "抬完候选集里还要有 4 个不同状态，只剩 {distinct_states} 个");
            }
            floors.push(simulation.floor_carried);
        }
        assert!(floors[1] > floors[0], "环里有空根时按根数读抬得比按状态数读高（roots={} states={}）", floors[1], floors[0]);
    }

    #[test]
    fn raw_df_detector_fires_for_ring_arm_near_full() {
        let total = run_near_full(Arm::RingPersisted, 4, 5, 10, 0, None);
        assert!(total.false_enospc_raw > 0, "df_raw 把保留池算成空闲，臂 A 报 ENOSPC 时它必须 ≥ 8（检测器的阳性对照）");
    }

    /// skip_then_raise 的前缀真的跳了号：区域 1 连续失败 r 次之后，盘 1 上最新的根比最新根旧 3r 以上。
    #[test]
    fn skip_then_raise_prefix_really_lags_disk_one() {
        let prefix = activation_prefix(Arm::TightenedRoots, 4, 0, 3).expect("前缀不该卡死");
        assert_eq!(prefix.metrics.root_write_failures_injected, 3, "要恰好注入 3 次写失败");
        let newest = prefix.newest_durable_txg;
        let disk_one = prefix.newest_valid_txg_on_disk(1).expect("盘 1 上要有根");
        assert!(newest - disk_one >= 3, "盘 1 上最新的根要落后 3 个 txg 以上（newest={newest} disk_one={disk_one}）");
        assert!(prefix.floor_carried <= disk_one, "按盘取上限：抬到的 F 不许越过盘 1 上最新的根");
    }
    /// 跑后补的诊断（2026-09-12，第四轮三方论证正推腿打中）：`reserve_sweep` 的产物行不打印失败窗口号，
    /// 所以「按状态数读的删了再写失败全在填满后的前 3 个窗口」对保留池扫描的 18 格没有产物字段撑着。
    /// 这里把 G2S 在 S = 16、保留池 ≥ 跑前式子的每一格按种子重跑一遍，打印首末失败窗口并钉死：
    /// c₀ = 5 的 9 格首 1、末 3；c₀ = 1 的 9 格里 reserve = 式子 + 2..5 的四格末窗口是 6，其余末 3。
    /// 它只读装置、不改 `main` 的产物；产物留存为 `research/results/e139-tightened-floor-2026-09-12-reserve-sweep-windows.out`。
    #[test]
    fn states_arm_reserve_sweep_failure_windows_end_by_window_six() {
        let arm = Arm::TightenedStates;
        let ring_depth = REGION_COUNT * RESERVE_SWEEP_SLOTS_PER_REGION;
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            let formula = arm.reserve_blocks(ring_depth, arm.formula_cost(mount_cost, declared_maximum_cost));
            for reserve in formula..=(formula + RESERVE_SWEEP_ABOVE) {
                let mut first_window_minimum = u64::MAX;
                let mut last_window_maximum = 0u64;
                let mut write_after_delete_failed = 0u64;
                let mut seeds_failing = 0u64;
                for seed in 0..RESERVE_SWEEP_SEED_COUNT {
                    let one = run_near_full(arm, RESERVE_SWEEP_SLOTS_PER_REGION, mount_cost, declared_maximum_cost, seed, Some(reserve));
                    write_after_delete_failed += one.write_after_delete_failed;
                    if one.write_after_delete_failed > 0 {
                        seeds_failing += 1;
                        first_window_minimum = first_window_minimum.min(one.first_write_after_delete_failure_window);
                        last_window_maximum = last_window_maximum.max(one.last_write_after_delete_failure_window);
                    }
                }
                println!("RSWEEP_WINDOWS arm={} s={RESERVE_SWEEP_SLOTS_PER_REGION} c0={mount_cost} c_max={declared_maximum_cost} reserve={reserve} formula={formula} write_after_delete_failed={write_after_delete_failed} seeds_failing={seeds_failing} first_failure_window={first_window_minimum} last_failure_window={last_window_maximum}",
                    arm.name());
                let above_formula = reserve - formula;
                let (expected_first, expected_last) = match (mount_cost, above_formula) {
                    (5, _) => (1, 3),
                    (1, 2) => (2, 6),
                    (1, 3..=5) => (1, 6),
                    (1, _) => (2, 3),
                    (other_cost, _) => panic!("COST_PAIRS 只有 c₀ = 1 与 5，这里是 {other_cost}"),
                };
                assert_eq!(seeds_failing, RESERVE_SWEEP_SEED_COUNT, "c0={mount_cost} reserve={reserve}：每个种子都该在填满后失败过");
                assert_eq!((first_window_minimum, last_window_maximum), (expected_first, expected_last),
                    "c0={mount_cost} reserve={reserve}（式子 + {above_formula}）：首末失败窗口要钉在 ({expected_first}, {expected_last})");
            }
        }
    }
}
