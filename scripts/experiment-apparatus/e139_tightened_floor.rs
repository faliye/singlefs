//! E139：按盘回退下界的收严形态 —— D16（发布语义） 已定项 1 第三轮判决之后的重跑。
//!
//! 预注册 `research/prompts/e139-preregistration.md`。装置从 E138（按盘回退下界与推空的空间要求） 的源码拷出来改：
//! ① 两条被测臂 G2R / G2S：准入按最坏残留收费、界与保留池给 k_tol = 2 次根槽写失败留余量、
//!    保留池与残留按声明的最坏空发布开销 c_max 算、G2S 的上限按用户 2026-09-12 的口径取第 4 新的非空根；
//! ② 新增五个世界：活化区间内崩溃不丢盘、先跳号再抬 F、推空期间根槽写失败、坏槽、空发布开销在挂载之后变大。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
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
/// 第 2 次重跑（`research/prompts/e139-r2-prereg.md` 第五节「`disk_stuck`」）：占用两档、卡住的盘两块（盘 1 是原取样点，盘 0 是 H3）。
const STUCK_FILLS: [f64; 2] = [0.30, 0.70];
const STUCK_DISKS: [usize; 2] = [1, 0];
/// 刻度：从卡住那一刻起第 1、2、4 圈（一圈 = 3S 个 txg）。
const STUCK_MARK_REVOLUTIONS: [u64; 3] = [1, 2, 4];
/// 卡住段最多做几圈个删建窗口（一圈 3S 个窗口）；收尾再做几圈。
const STUCK_WINDOW_REVOLUTION_LIMIT: u64 = 16;
const STUCK_TAIL_REVOLUTIONS: u64 = 2;
/// `disk_stuck` 的快照上限：卡住段最多 16 × 3S = 768 个窗口加预热 106 个，留 1024 份，卡住那块盘上的旧根在恢复时仍读得到它的快照。
const STUCK_SNAPSHOTS_KEPT: usize = 1024;
/// 「有上界」的门槛（登记第一节读法第 10 条 ⓒ）：s(2→4) 小于 1/2。写成 2 × 扣住的增量 < 放掉的块数，不取整。
const STUCK_BOUNDED_SLOPE_DENOMINATOR: i64 = 2;

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
    /// 第 1 次重跑（`research/prompts/e139-r1-prereg.md`）：臂 G2S 加一圈窗口的一族，不进 `ARMS`。
    TightenedStatesWindowed(WindowArm),
    /// 第 2 次重跑第 1 行（`research/prompts/e139-r2-prereg.md` 第五节「CF」）：窗口一族的同一条臂，候选下界取条款形，不进 `ARMS`。行名 `f_`。
    ClauseFloorWindowed(WindowArm),
    /// 第 2 次重跑第 2 行（同上「RGall」）：回收门槛与同取窗口的另外五处一起退回环里最旧有效根，候选下界取条款形，不进 `ARMS`。
    WholeRing(WholeRingArm),
}

/// RGall 一族的臂（登记第五节「RGall」那张表）。装置里没有窗口这一项：回收门槛、候选集、抬 F 的上限与处置目标、滞后量都取整环。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WholeRingArm {
    /// RGall：某块幸存盘整环里一条有效根都没有时，那块盘不进「每块幸存盘上最新的有效根」取小（读法 P，`crates/` 今天的上限形状）。被测。
    ReadingP,
    /// RGall_Q：那样的盘记 0、F 不抬（读法 Q）。被测。
    ReadingQ,
    /// 阳性对照：RGall 只有回收门槛里的「最旧有效根」换成窗口缩短一行（txg + 3(S − 1) > N）的那一段里的最旧有效根。
    ReadingPShort,
    ReadingQShort,
    /// 只给单测 K15：RGall_Q 的候选下界取旧式（不并系统配置 F），用来对 Q 与原 G2S 逐处同式。不进任何一行产物。
    #[cfg_attr(not(test), allow(dead_code, reason = "只给单测 K15 用，产物里没有这条臂"))]
    ReadingQWithTheOldCandidateFloor,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WholeRingCeilingReading {
    DisksWithoutAValidRootAreSkipped,
    DisksWithoutAValidRootCountAsZero,
}

impl WholeRingArm {
    fn name(self) -> &'static str {
        match self {
            WholeRingArm::ReadingP => "RGall_ring_all",
            WholeRingArm::ReadingQ => "RGall_Q_ring_all",
            WholeRingArm::ReadingPShort => "RGall_short",
            WholeRingArm::ReadingQShort => "RGall_Q_short",
            WholeRingArm::ReadingQWithTheOldCandidateFloor => "RGall_Q_old_candidate_floor",
        }
    }
    fn ceiling_reading(self) -> WholeRingCeilingReading {
        match self {
            WholeRingArm::ReadingP | WholeRingArm::ReadingPShort => WholeRingCeilingReading::DisksWithoutAValidRootAreSkipped,
            WholeRingArm::ReadingQ | WholeRingArm::ReadingQShort | WholeRingArm::ReadingQWithTheOldCandidateFloor => {
                WholeRingCeilingReading::DisksWithoutAValidRootCountAsZero
            }
        }
    }
    fn is_reclaim_window_shortened(self) -> bool {
        match self {
            WholeRingArm::ReadingPShort | WholeRingArm::ReadingQShort => true,
            WholeRingArm::ReadingP | WholeRingArm::ReadingQ | WholeRingArm::ReadingQWithTheOldCandidateFloor => false,
        }
    }
    fn uses_the_clause_candidate_floor(self) -> bool {
        match self {
            WholeRingArm::ReadingP | WholeRingArm::ReadingQ | WholeRingArm::ReadingPShort | WholeRingArm::ReadingQShort => true,
            WholeRingArm::ReadingQWithTheOldCandidateFloor => false,
        }
    }
}

/// RGall 一族进全部格的两条被测臂，与它们的阳性对照（只进 steady、S ∈ {4, 16}）。
const WHOLE_RING_ARMS: [WholeRingArm; 2] = [WholeRingArm::ReadingP, WholeRingArm::ReadingQ];
const WHOLE_RING_SHORT_ARMS: [WholeRingArm; 2] = [WholeRingArm::ReadingPShort, WholeRingArm::ReadingQShort];

/// 第 1 次重跑的七条臂（`research/prompts/e139-r1-prereg.md` 第五节）：回收门槛改成一圈窗口（txg + 3S > N；
/// 核心层后来退回整个环，那个判法函数已删，见 C842（一圈窗口判法成了死代码））之后的臂 G2S，与它的读法臂、旧门槛对照、阳性对照。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WindowArm {
    /// G2SW：被测；抬 F 的上限取读法 P，滞后量取读法 a。
    Window,
    /// G2SW_Q：抬 F 的上限取读法 Q（窗口里没有有效根的盘记 0）。
    DiskHold,
    /// G2SW_L：滞后量里「第 4 新的非空有效根」取整环（读法 b）。
    RingLag,
    /// G2SW_ringgate：只把回收门槛换回旧式整环取法（C660 判别力自证）。
    RingGate,
    /// 三条被测臂的阳性对照：只有回收门槛的窗口缩短一行（txg + 3(S − 1) > N）。
    WindowShort,
    DiskHoldShort,
    RingLagShort,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ReclaimGate {
    /// max(F_生效, 窗口里最旧有效根)。
    OldestValidRootInTheWindow,
    /// max(F_生效, 环里最旧有效根)，旧式。
    OldestValidRootInTheWholeRing,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CeilingReading {
    /// 读法 P：窗口里一条有效根都没有的盘不进「每块幸存盘上最新的有效根」取小。
    DisksWithoutAWindowRootAreSkipped,
    /// 读法 Q：那样的盘记 0，F 不抬。
    DisksWithoutAWindowRootCountAsZero,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LagBoundReading {
    /// 读法 a：「第 4 新的非空有效根」与抬 F 的上限同一个窗口里取。
    FourthNewestInTheWindow,
    /// 读法 b：整环取（原 G2S 的取法）。
    FourthNewestInTheWholeRing,
}

impl WindowArm {
    fn name(self) -> &'static str {
        match self {
            WindowArm::Window => "G2SW_tightened_states_window",
            WindowArm::DiskHold => "G2SW_Q_disk_hold",
            WindowArm::RingLag => "G2SW_L_ring_lag",
            WindowArm::RingGate => "G2SW_ringgate",
            WindowArm::WindowShort => "G2SW_short",
            WindowArm::DiskHoldShort => "G2SW_Q_short",
            WindowArm::RingLagShort => "G2SW_L_short",
        }
    }
    fn reclaim_gate(self) -> ReclaimGate {
        match self {
            WindowArm::RingGate => ReclaimGate::OldestValidRootInTheWholeRing,
            WindowArm::Window | WindowArm::DiskHold | WindowArm::RingLag | WindowArm::WindowShort | WindowArm::DiskHoldShort | WindowArm::RingLagShort => {
                ReclaimGate::OldestValidRootInTheWindow
            }
        }
    }
    fn is_reclaim_window_shortened(self) -> bool {
        match self {
            WindowArm::WindowShort | WindowArm::DiskHoldShort | WindowArm::RingLagShort => true,
            WindowArm::Window | WindowArm::DiskHold | WindowArm::RingLag | WindowArm::RingGate => false,
        }
    }
    fn ceiling_reading(self) -> CeilingReading {
        match self {
            WindowArm::DiskHold | WindowArm::DiskHoldShort => CeilingReading::DisksWithoutAWindowRootCountAsZero,
            WindowArm::Window | WindowArm::RingLag | WindowArm::RingGate | WindowArm::WindowShort | WindowArm::RingLagShort => {
                CeilingReading::DisksWithoutAWindowRootAreSkipped
            }
        }
    }
    fn lag_bound_reading(self) -> LagBoundReading {
        match self {
            WindowArm::RingLag | WindowArm::RingLagShort => LagBoundReading::FourthNewestInTheWholeRing,
            WindowArm::Window | WindowArm::DiskHold | WindowArm::RingGate | WindowArm::WindowShort | WindowArm::DiskHoldShort => {
                LagBoundReading::FourthNewestInTheWindow
            }
        }
    }
}

/// 进全部格的四条臂（被测三条加旧门槛对照）。
const WINDOW_ARMS: [WindowArm; 4] = [WindowArm::Window, WindowArm::DiskHold, WindowArm::RingLag, WindowArm::RingGate];
/// 窗口这一项的阳性对照：只进 steady、S ∈ {4, 16}。
const WINDOW_SHORT_ARMS: [WindowArm; 3] = [WindowArm::WindowShort, WindowArm::DiskHoldShort, WindowArm::RingLagShort];
const WINDOW_SHORT_SLOTS_SWEEP: [u64; 2] = [4, 16];

/// 一圈窗口：txg + 窗口长度 > 锚。窗口长度是环长 R × S；阳性对照臂的回收门槛缩短一行（少 R）。
fn txg_is_inside_window(txg: u64, anchor: u64, window_length: u64) -> bool {
    txg + window_length > anchor
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
            Arm::TightenedStatesWindowed(window_arm) | Arm::ClauseFloorWindowed(window_arm) => window_arm.name(),
            Arm::WholeRing(whole_ring_arm) => whole_ring_arm.name(),
        }
    }
    fn uses_floor(self) -> bool {
        match self {
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum => false,
            Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_)
            | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => true,
        }
    }
    /// 候选下界取条款形（把系统配置 F 并进去，第 2 次重跑第 1 行）。
    fn uses_the_clause_candidate_floor(self) -> bool {
        match self {
            Arm::ClauseFloorWindowed(_) => true,
            Arm::WholeRing(whole_ring_arm) => whole_ring_arm.uses_the_clause_candidate_floor(),
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates
            | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_) => false,
        }
    }
    /// 第 2 次重跑的新量（下界差、ⓐⓑⓒ、刻度）只在这一族臂上采；`w_` 行不打印它们，旧臂不为它们多算。
    fn collects_second_rerun_observations(self) -> bool {
        match self {
            Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => true,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates
            | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_) => false,
        }
    }
    /// 第 1 次的四样轨迹（门槛差、上限差、滞后量、`df`）读哪些臂：窗口一族（两个下界口径）与 RGall 一族。
    fn records_the_four_trajectories(self) -> bool {
        match self {
            Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => true,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates
            | Arm::TightenedRootsAtMount => false,
        }
    }
    /// 被测的两条：判据 3–8 只对它们判。
    fn is_candidate(self) -> bool {
        match self {
            Arm::TightenedRoots | Arm::TightenedStates => true,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_)
            | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => false,
        }
    }
    /// 收严形态三条共有的：准入按最坏残留收费、界带 k_tol 的余量。
    fn is_tightened(self) -> bool {
        match self {
            Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => true,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit => false,
        }
    }
    /// 上限按第 4 新的非空根取（用户 2026-09-12 的口径）。
    fn counts_states(self) -> bool {
        match self {
            Arm::TightenedStates | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => true,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedRootsAtMount => false,
        }
    }
    /// 保留池、残留与 df 按哪个开销算：被测臂按声明的最坏值，其余按挂载时的值。
    fn formula_cost(self, mount_cost: u64, declared_maximum_cost: u64) -> u64 {
        match self {
            Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => declared_maximum_cost,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRootsAtMount => mount_cost,
        }
    }
    /// 预注册的界 B：一次准入最多用几次发布（含发布正在攒的窗口）。
    fn publication_bound(self, ring_depth: u64) -> u64 {
        match self {
            Arm::RingPersisted => ring_depth,
            Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit => FLOOR_PER_DISK_PUBLICATION_BOUND,
            Arm::TightenedRoots | Arm::TightenedRootsAtMount => TIGHTENED_ROOTS_PUBLICATION_BOUND,
            Arm::TightenedStates | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => TIGHTENED_STATES_PUBLICATION_BOUND,
            Arm::ProtectedGenerationsAtMinimum => 1,
            Arm::FloorTxg => 1 + TXG_ARITHMETIC_GENERATIONS + 2,
        }
    }
    /// 预注册：保留池 = 上一个窗口的元数据 + 这次处置要拿的块 = 2 × 5 + (B − 1) × c。
    fn reserve_blocks(self, ring_depth: u64, formula_cost: u64) -> u64 {
        match self {
            Arm::RingPersisted | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_)
            | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => {
                2 * METADATA_BLOCKS_PER_PUBLICATION + (self.publication_bound(ring_depth) - 1) * formula_cost
            }
            Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg => CONTROL_ARM_RESERVE_BLOCKS,
        }
    }
    /// 预注册：一次完整处置之后还扣着的最坏块数 = 5 + (B − 2) × c。
    fn push_residue_blocks(self, ring_depth: u64, formula_cost: u64) -> u64 {
        match self {
            Arm::RingPersisted | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_)
            | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => {
                METADATA_BLOCKS_PER_PUBLICATION + (self.publication_bound(ring_depth) - 2) * formula_cost
            }
            Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg => 0,
        }
    }
    /// drain_failure 里失败位置扫到第几次根槽写（第 0 次是发布正在攒的窗口）。
    fn drain_failure_offset_limit(self, ring_depth: u64) -> u64 {
        match self {
            Arm::RingPersisted => ring_depth.min(DRAIN_FAILURE_RING_ARM_OFFSET_LIMIT),
            Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_)
            | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg => {
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

/// 第 1 次重跑第八节的轨迹：被谓词消费的量报「吃光初始供给之后的峰值、为正的次数、期末值」（`test-discipline.md`「端点不是轨迹」）。
#[derive(Clone, Copy, Debug)]
struct ObservedQuantity {
    samples: u64,
    positive_samples: u64,
    maximum: i64,
    maximum_after_supply: i64,
    minimum: i64,
    /// 一次运行里最后一个读数；几次运行合计时取各次最后读数的最大值。
    last: i64,
}

impl ObservedQuantity {
    fn fresh() -> Self {
        ObservedQuantity { samples: 0, positive_samples: 0, maximum: i64::MIN, maximum_after_supply: i64::MIN, minimum: i64::MAX, last: i64::MIN }
    }
    fn observe(&mut self, value: i64, is_supply_exhausted: bool) {
        self.samples += 1;
        self.positive_samples += u64::from(value > 0);
        self.maximum = self.maximum.max(value);
        if is_supply_exhausted {
            self.maximum_after_supply = self.maximum_after_supply.max(value);
        }
        self.minimum = self.minimum.min(value);
        self.last = value;
    }
    fn add(&mut self, other: &ObservedQuantity) {
        self.samples += other.samples;
        self.positive_samples += other.positive_samples;
        self.maximum = self.maximum.max(other.maximum);
        self.maximum_after_supply = self.maximum_after_supply.max(other.maximum_after_supply);
        self.minimum = self.minimum.min(other.minimum);
        self.last = self.last.max(other.last);
    }
    /// 没读数的字段写 na，不写成 0（读不到不等于 0）。
    fn render(&self, label: &str) -> String {
        let shown = |value: i64, unobserved: i64| if value == unobserved { "na".to_string() } else { value.to_string() };
        format!(
            "{label}_samples={} {label}_positive={} {label}_max={} {label}_max_after_supply={} {label}_min={} {label}_last_max={}",
            self.samples, self.positive_samples, shown(self.maximum, i64::MIN), shown(self.maximum_after_supply, i64::MIN),
            shown(self.minimum, i64::MAX), shown(self.last, i64::MIN)
        )
    }
}

#[derive(Clone, Copy, Debug)]
struct Trajectory {
    /// 第六节第 6 行：这条臂自己的回收门槛减整环门槛，每次 `allocate` 之前读一次。
    gate_difference: ObservedQuantity,
    /// 这条臂的抬 F 上限减原 G2S 整环取法的上限，每次决定抬不抬 F 时读一次。
    ceiling_difference: ObservedQuantity,
    /// 滞后量与这条臂自己的 `df`，每个用户窗口读一次。
    lag: ObservedQuantity,
    df: ObservedQuantity,
    df_below_user_blocks_windows: u64,
}

impl Trajectory {
    fn fresh() -> Self {
        Trajectory {
            gate_difference: ObservedQuantity::fresh(),
            ceiling_difference: ObservedQuantity::fresh(),
            lag: ObservedQuantity::fresh(),
            df: ObservedQuantity::fresh(),
            df_below_user_blocks_windows: 0,
        }
    }
    fn add(&mut self, other: &Trajectory) {
        self.gate_difference.add(&other.gate_difference);
        self.ceiling_difference.add(&other.ceiling_difference);
        self.lag.add(&other.lag);
        self.df.add(&other.df);
        self.df_below_user_blocks_windows += other.df_below_user_blocks_windows;
    }
    fn render(&self) -> String {
        format!(
            "{} {} {} {} df_below_user_blocks_windows={}",
            self.gate_difference.render("gate_difference"), self.ceiling_difference.render("ceiling_difference"),
            self.lag.render("lag"), self.df.render("df"), self.df_below_user_blocks_windows
        )
    }
}

fn signed_difference(left: u64, right: u64) -> i64 {
    i64::try_from(left).expect("块数与 txg 装得进 i64") - i64::try_from(right).expect("块数与 txg 装得进 i64")
}

/// 刻度那一次采样（登记第一节读法第 10 条 ⓒ）：第一个落盘 txg ≥ T₀ + k × 3S 的那次根落盘之后读 `pinned_count()`，
/// 连同这条臂到那时为止放掉的块数（`Simulation::free_blocks` 累计）。
#[derive(Clone, Copy, Debug)]
struct StuckMark {
    published_txg: u64,
    pinned_blocks: u64,
    blocks_freed_in_total: u64,
}

/// 两个刻度之间：（扣住的块数增量，这条臂放掉的块数）。放掉的块数为 0 时没有斜率。
fn stuck_slope_between(earlier: &StuckMark, later: &StuckMark) -> Option<(i64, i64)> {
    let freed_between = signed_difference(later.blocks_freed_in_total, earlier.blocks_freed_in_total);
    if freed_between <= 0 {
        return None;
    }
    Some((signed_difference(later.pinned_blocks, earlier.pinned_blocks), freed_between))
}

/// 「有上界」：s < 1/2，写成 2 × 增量 < 放掉的块数，不取整（登记第一节读法第 10 条 ⓒ）。
fn is_stuck_slope_bounded(pinned_increase: i64, blocks_freed_between: i64) -> bool {
    STUCK_BOUNDED_SLOPE_DENOMINATOR * pinned_increase < blocks_freed_between
}

fn slope_in_hundredths(pinned_increase: i64, blocks_freed_between: i64) -> i64 {
    pinned_increase * 100 / blocks_freed_between
}

/// 第 2 次重跑的新量，只在 `Arm::collects_second_rerun_observations` 的臂上采、只在 `f_` 行里打印（`w_` 行不读它们）。
#[derive(Clone, Copy, Debug)]
struct SecondRerunObservations {
    /// 下界差 g = 条款形候选下界 − 旧候选下界：每次 `record_check` 与每次 `admin_rollback` 读一次。
    candidate_floor_gap: ObservedQuantity,
    rollback_target_changed: u64,
    fakes_with_the_old_floor: u64,
    checks_old_set_strictly_larger: u64,
    checks_new_set_strictly_larger: u64,
    system_configuration_below_root_floor_events: u64,
    /// ⓐ 每次 `crash_and_recover` 做完（`adopt_floors` 之后）的 `pinned_count()`。
    pinned_after_recovery: ObservedQuantity,
    /// ⓑ 每次根落盘之后：这条臂的门槛下扣住的块数减窗口门槛下扣住的块数。
    extra_pinned_against_the_window_gate: ObservedQuantity,
    /// ⓑ 幸存盘上住着窗口外有效根的槽数 > 0 的采样上：多扣的块数之和、槽数之和、多扣的块数除以槽数的最大值（乘 100 取整）。
    samples_with_a_hole: u64,
    extra_pinned_sum_over_samples_with_a_hole: i64,
    hole_slots_sum_over_samples_with_a_hole: u64,
    extra_pinned_per_hole_maximum_in_hundredths: i64,
    /// ⓒ 每次根落盘之后的 `pinned_count()`。
    pinned_after_publication: ObservedQuantity,
    /// 采样时幸存盘上有一条有效根满足 txg + 2 × 3S ≤ N 的采样数（登记第五节「别的世界不量斜率」）。
    stale_beyond_two_revolutions_samples: u64,
    /// 每次决定抬不抬 F 时：某块幸存盘整环里一条有效根都没有的次数。
    disk_without_valid_root_events: u64,
    /// 刻度 k = 1、2、4（`STUCK_MARK_REVOLUTIONS`）；没走到的是 `None`。几次运行合计时不合并，只留自己的。
    marks: [Option<StuckMark>; 3],
}

impl SecondRerunObservations {
    fn fresh() -> Self {
        SecondRerunObservations {
            candidate_floor_gap: ObservedQuantity::fresh(),
            rollback_target_changed: 0,
            fakes_with_the_old_floor: 0,
            checks_old_set_strictly_larger: 0,
            checks_new_set_strictly_larger: 0,
            system_configuration_below_root_floor_events: 0,
            pinned_after_recovery: ObservedQuantity::fresh(),
            extra_pinned_against_the_window_gate: ObservedQuantity::fresh(),
            samples_with_a_hole: 0,
            extra_pinned_sum_over_samples_with_a_hole: 0,
            hole_slots_sum_over_samples_with_a_hole: 0,
            extra_pinned_per_hole_maximum_in_hundredths: i64::MIN,
            pinned_after_publication: ObservedQuantity::fresh(),
            stale_beyond_two_revolutions_samples: 0,
            disk_without_valid_root_events: 0,
            marks: [None; 3],
        }
    }
    fn add(&mut self, other: &SecondRerunObservations) {
        self.candidate_floor_gap.add(&other.candidate_floor_gap);
        self.rollback_target_changed += other.rollback_target_changed;
        self.fakes_with_the_old_floor += other.fakes_with_the_old_floor;
        self.checks_old_set_strictly_larger += other.checks_old_set_strictly_larger;
        self.checks_new_set_strictly_larger += other.checks_new_set_strictly_larger;
        self.system_configuration_below_root_floor_events += other.system_configuration_below_root_floor_events;
        self.pinned_after_recovery.add(&other.pinned_after_recovery);
        self.extra_pinned_against_the_window_gate.add(&other.extra_pinned_against_the_window_gate);
        self.samples_with_a_hole += other.samples_with_a_hole;
        self.extra_pinned_sum_over_samples_with_a_hole += other.extra_pinned_sum_over_samples_with_a_hole;
        self.hole_slots_sum_over_samples_with_a_hole += other.hole_slots_sum_over_samples_with_a_hole;
        self.extra_pinned_per_hole_maximum_in_hundredths = self.extra_pinned_per_hole_maximum_in_hundredths.max(other.extra_pinned_per_hole_maximum_in_hundredths);
        self.pinned_after_publication.add(&other.pinned_after_publication);
        self.stale_beyond_two_revolutions_samples += other.stale_beyond_two_revolutions_samples;
        self.disk_without_valid_root_events += other.disk_without_valid_root_events;
    }
    /// `f_attribution` 行的字段：第六节第 4、6、7、8 行与 V5–V7 要的量。
    fn render_attribution(&self) -> String {
        format!(
            "{} rollback_target_changed={} fakes_with_the_old_floor={} checks_old_set_strictly_larger={} checks_new_set_strictly_larger={} \
             system_configuration_below_root_floor_events={}",
            self.candidate_floor_gap.render("candidate_floor_gap"), self.rollback_target_changed, self.fakes_with_the_old_floor,
            self.checks_old_set_strictly_larger, self.checks_new_set_strictly_larger, self.system_configuration_below_root_floor_events
        )
    }
    /// `f_trajectory` 行接在第 1 次四样轨迹之后的字段：代价三样与两个推的量。
    fn render_costs(&self) -> String {
        let per_hole_maximum = if self.extra_pinned_per_hole_maximum_in_hundredths == i64::MIN { "na".to_string() } else { self.extra_pinned_per_hole_maximum_in_hundredths.to_string() };
        format!(
            "{} {} holes_samples={} extra_pinned_sum_at_holes={} hole_slots_sum_at_holes={} extra_pinned_per_hole_max_x100={per_hole_maximum} {} \
             stale_beyond_two_revolutions={} disk_without_valid_root_events={}",
            self.pinned_after_recovery.render("pinned_after_recovery"), self.extra_pinned_against_the_window_gate.render("extra_pinned_vs_window_gate"),
            self.samples_with_a_hole, self.extra_pinned_sum_over_samples_with_a_hole, self.hole_slots_sum_over_samples_with_a_hole,
            self.pinned_after_publication.render("pinned_after_publication"), self.stale_beyond_two_revolutions_samples, self.disk_without_valid_root_events
        )
    }
}

#[derive(Clone, Copy, Debug)]
struct Metrics {
    trajectory: Trajectory,
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
    /// 第 2 次重跑的新量；第 1 次的每一行都不读它，也不打印它。
    second_rerun: SecondRerunObservations,
}

impl Metrics {
    fn fresh() -> Self {
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
        }
    }
    fn add(&mut self, other: &Metrics) {
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
    /// 快照留多少份；默认 `SNAPSHOTS_KEPT`。`disk_stuck` 里卡住那块盘上的旧根可以比 64 次发布还老，恢复要读得到它的快照，那里放大（`STUCK_SNAPSHOTS_KEPT`）。
    snapshots_kept_limit: usize,
    keep_snapshots: bool,
    tear_next_publication: bool,
    fail_next_slot_write: bool,
    /// 第几次根槽写要失败（按根槽写的绝对序号，重发也算一次）。
    planned_root_write_failures: BTreeSet<u64>,
    root_write_attempts: u64,
    /// broken_slot 世界：这个槽从此每次写都失败。
    broken_slot: Option<usize>,
    /// 系统配置里记的 F（第 2 次重跑第五节「CF」）：`raise_floor_to` 真抬时先并进来，崩溃、丢盘、管理员回退都不降它；每块盘都写、不失败（登记差异 ⑨）。
    system_configuration_floor: u64,
    /// disk_stuck 世界：这块盘上的根槽从此每次写都失败（照 `publish` 的推进一格、槽留旧根）。
    stuck_disk: Option<usize>,
    /// 到现在为止这条臂放掉的块数（`free_blocks` 里 Live 变 Freed 的块数）；刻度之间的斜率用它。
    blocks_freed_in_total: u64,
    /// 刻度的起点 T₀（卡住或坏槽开始那一刻的 `next_txg`）；设了才记刻度。
    stuck_marks_origin_txg: Option<u64>,
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
    /// 初始供给吃光了没有（第一次准入不够）：轨迹的「吃光之后的峰值」从这一刻起记。
    is_supply_exhausted: bool,
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
        }
    }

    fn next_random(&mut self) -> u64 {
        self.random_state ^= self.random_state >> 12;
        self.random_state ^= self.random_state << 25;
        self.random_state ^= self.random_state >> 27;
        self.random_state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// 区域 = txg mod R，区内槽 = ⌊txg / R⌋ mod S（区内公式是 layout/01-first-txn.md 的预想）。
    fn slot_index(&self, txg: u64) -> usize {
        let region = txg % REGION_COUNT;
        let slot_within_region = (txg / REGION_COUNT) % self.slots_per_region;
        usize::try_from(region * self.slots_per_region + slot_within_region).expect("槽号装得进 usize")
    }

    fn region_of_txg(txg: u64) -> u64 {
        txg % REGION_COUNT
    }

    /// 区域 0、2 在盘 0，区域 1 在盘 1（layout/01-first-txn.md 的预想归属）。
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

    fn window_arm(&self) -> Option<WindowArm> {
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
    fn is_inside_candidate_window(&self, txg: u64) -> bool {
        match self.window_arm() {
            None => true,
            Some(_) => txg_is_inside_window(txg, self.newest_durable_txg, self.ring_depth),
        }
    }

    /// W1：窗口里最旧的有效根；窗口里一条都没有时 `None`。锚 = 这个进程记到的最新根的 txg（`allocator.rs` 的 `record_root_written_by_this_process`）。
    fn oldest_valid_txg_in_the_reclaim_window(&self, window_length: u64) -> Option<u64> {
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
    fn windowed_states_upper_limit(&self, window_arm: WindowArm) -> u64 {
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
    fn windowed_lag_bound_txg(&self, window_arm: WindowArm) -> u64 {
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
    fn observe_ceiling_difference(&mut self) {
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
    fn floor_upper_limit(&self) -> u64 {
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
    fn disposal_target(&self, goal_txg: u64) -> u64 {
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
    fn candidacy_floor(&self) -> u64 {
        if self.arm.uses_floor() {
            self.visible_roots().iter().map(|root| root.floor).max().unwrap_or(0)
        } else {
            0
        }
    }

    /// 第 2 次重跑第 1 行：条款形的候选下界 = max(`candidacy_floor()`, 系统配置里的 F)（D16 已定项 1「生效」那一行：F_生效 取三者最大，系统配置里读得出的 F 是其中一项）。
    /// 只换候选集的三处（`count_fake_candidates`、`admin_rollback` 选目标、`is_retention_violated`）；恢复之后的 F（`floors_after_recovery`）与回收门槛不动（登记差异 ③）。
    fn candidate_floor(&self) -> u64 {
        if self.arm.uses_the_clause_candidate_floor() { self.candidacy_floor().max(self.system_configuration_floor) } else { self.candidacy_floor() }
    }

    /// 可再分配的谓词：已释放 ∧ 释放代 ≤ 这个界。
    fn reuse_bound(&self) -> u64 {
        match self.arm {
            Arm::RingPersisted => self.oldest_persisted_valid_txg(),
            Arm::ProtectedGenerationsAtMinimum => (self.newest_durable_txg + 1).saturating_sub(TXG_ARITHMETIC_GENERATIONS),
            Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount => {
                self.floor_active.max(self.oldest_persisted_valid_txg())
            }
            Arm::TightenedStatesWindowed(window_arm) | Arm::ClauseFloorWindowed(window_arm) => self.floor_active.max(self.oldest_valid_txg_for_the_reclaim_gate(window_arm)),
            Arm::WholeRing(whole_ring_arm) => self.floor_active.max(self.oldest_valid_txg_for_the_whole_ring_gate(whole_ring_arm)),
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

    /// 滞后量的下界 max(第 4 新的非空有效根, 环里最旧有效根)：窗口一族按各自的读法 a / b，别的臂（含 RGall 一族）整环取。
    fn lag_bound_txg(&self) -> u64 {
        match self.window_arm() {
            Some(window_arm) => self.windowed_lag_bound_txg(window_arm),
            None => self.fourth_newest_nonempty_valid_txg().max(self.oldest_persisted_valid_txg()),
        }
    }

    /// 按状态数读时一次处置也放不回的已释放块（滞后量）：释放代 > max(第 4 新的非空根, 环里最旧有效根)，
    /// 且不是空发布放掉的（那是推空自己的残留，另外按最坏值扣）。正在攒的窗口放掉的也算在内。别的臂是 0。
    fn lag_blocks(&self) -> u64 {
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
    fn allocate(&mut self, count: u64, txg: u64) -> Option<Vec<usize>> {
        self.observe_gate_difference();
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
                self.blocks_freed_in_total += 1;
            }
        }
    }

    /// (假候选数, 候选数, 其中非空发布的根数, 其中不同的用户可见状态数)。
    /// 状态数：非空候选各算一个；候选集里比最旧非空候选还旧的空根共享同一个更早的状态，再算一个。
    fn count_fake_candidates(&self) -> (u64, u64, u64, u64) {
        self.count_fake_candidates_at_floor(self.candidate_floor())
    }

    /// 同一个状态上按给定的下界数候选集：第 2 次重跑拿它同时数旧下界与条款形下界（`record_check`）。
    fn count_fake_candidates_at_floor(&self, floor: u64) -> (u64, u64, u64, u64) {
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
        self.prune_damage();
        self.record_check();
        self.observe_after_the_root_is_durable();
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
            self.system_configuration_floor = self.system_configuration_floor.max(target);
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
            Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_)
            | Arm::WholeRing(_) => {
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
    fn forced_pressure(&mut self) {
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
    fn extra_pinned_blocks_against_the_window_gate(&self) -> i64 {
        let window_gate = self.floor_active.max(self.oldest_valid_txg_in_the_reclaim_window(self.ring_depth).unwrap_or(0));
        signed_difference(self.pinned_count(), self.count_freed_after(window_gate))
    }

    /// ⓑ：幸存盘上住着窗口外有效根的槽数（窗口锚是这个进程记到的最新根）。
    fn slots_holding_a_valid_root_outside_the_window(&self) -> u64 {
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
    fn oldest_candidate_root_at_floor(&self, floor: u64) -> Option<RootRecord> {
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

    fn admin_rollback(&mut self) -> bool {
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
    simulation.stuck_marks_origin_txg = Some(simulation.next_txg);
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
            Arm::TightenedStates | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => (vec![3], 3, METADATA_BLOCKS_PER_PUBLICATION + cost, 0),
            Arm::RingPersisted | Arm::FloorPerDisk | Arm::TightenedRoots => (vec![3], 3, METADATA_BLOCKS_PER_PUBLICATION + cost, 0),
            Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRootsAtMount => (Vec::new(), 0, 0, 0),
        };
    }
    match arm {
        Arm::RingPersisted => (vec![ring_depth], ring_depth, METADATA_BLOCKS_PER_PUBLICATION + (ring_depth - 2) * cost, 0),
        Arm::FloorPerDisk | Arm::TightenedRoots => (vec![6, 7], 7, METADATA_BLOCKS_PER_PUBLICATION + 5 * cost, 0),
        // 目标是第 4 新的非空根（3 个窗口之前），发布正在攒的窗口之后立刻抬，带新 F 的第一个根 t + 1，最多再 2 次两块盘都有；
        // 滞后量 = t 与前两个非空窗口各放掉的 8 块对象 + 5 块元数据
        Arm::TightenedStates | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => {
            (vec![3, 4], 4, METADATA_BLOCKS_PER_PUBLICATION + 2 * cost, 3 * BLOCKS_PER_PUBLICATION)
        }
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
        Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_)
        | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => None,
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

/// 几何敏感性（跑前登记第八节）：原取样点与方向相反的取样点。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GeometryPoint {
    /// 原取样点：跳号在区域 1（盘 1，独占一块盘），坏槽在槽 1（区域 0，盘 0）。
    Original,
    /// H1 / H2：跳号换成区域 0（盘 0），坏槽换成区域 1 的第一个槽（槽号 S，盘 1）。
    Reversed,
}

/// H2 的前缀：同 `activation_prefix`，只是跳号落在 `jump_region` 而不是区域 1。
fn activation_prefix_with_jump_region(arm: Arm, slots_per_region: u64, seed: u64, skip_repeats: u64, jump_region: u64) -> Option<Simulation> {
    let (mount_cost, declared_maximum_cost) = COST_PAIRS[0];
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, true, None);
    simulation.prefill(STEADY_FILL);
    let (event_window, _) = schedule(seed, simulation.ring_depth, World::Steady);
    for _ in 0..event_window {
        simulation.run_user_window(true, false);
    }
    let mut remaining = skip_repeats;
    while remaining > 0 && !simulation.halted {
        if Simulation::region_of_txg(simulation.next_txg) == jump_region {
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

/// H1：同 `run_broken_slot`，坏槽换成 `broken_slot` 号槽。
fn run_broken_slot_at(arm: Arm, slots_per_region: u64, mount_cost: u64, declared_maximum_cost: u64, seed: u64, broken_slot: usize) -> Metrics {
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, false, None);
    simulation.prefill(STEADY_FILL);
    let warm_up = 2 * simulation.ring_depth + 10;
    for _ in 0..warm_up {
        simulation.run_user_window(true, false);
    }
    simulation.metrics = Metrics::fresh();
    simulation.counting_window = 0;
    simulation.broken_slot = Some(broken_slot);
    simulation.stuck_marks_origin_txg = Some(simulation.next_txg);
    for _ in 0..BROKEN_SLOT_WINDOWS {
        if simulation.halted {
            break;
        }
        simulation.run_counted_delete_create_window();
    }
    simulation.metrics
}

/// 第六节第 6 行里「注入根槽写失败」的六类世界。
const WORLD_CLASSES_INJECTING_ROOT_WRITE_FAILURES: [&str; 6] =
    ["slot_write_fail", "activation_faults", "activation_crash_no_loss", "skip_then_raise", "drain_failure", "broken_slot"];
const NEAR_FULL_CLASS: &str = "near_full";
const DRAIN_FAILURE_CLASS: &str = "drain_failure";
const BROKEN_SLOT_CLASS: &str = "broken_slot";
const COST_GROWTH_CLASS: &str = "cost_growth";
const DISK_STUCK_CLASS: &str = "disk_stuck";
const DISK_STUCK_REVERSED_CLASS: &str = "disk_stuck_h3";
/// 一族产物行的行名：第 1 次重跑的 `w_` 行与第 2 次重跑的 `f_` 行同形，只换名字（登记第五节「CF」：行名把 `w_` 换成 `f_`）。
/// 同一类世界里的臂各行逐字比时按这些行名认行（不含几何点后缀）。
struct LineNames {
    world: &'static str,
    activation: &'static str,
    near_full: &'static str,
    drain_failure: &'static str,
    broken_slot: &'static str,
    cost_growth: &'static str,
    activation_reversed: &'static str,
    broken_slot_reversed: &'static str,
    crit1_push: &'static str,
    trajectory: &'static str,
    detail_activation: &'static str,
    detail_drain: &'static str,
    readings_diff_row: &'static str,
    verdict_window: &'static str,
    /// 只有第 2 次重跑有的行；第 1 次的 `w_` 一族不出这几类，留空。
    disk_stuck: &'static str,
    disk_stuck_h3: &'static str,
    stuck_growth: &'static str,
    broken_slot_growth: &'static str,
    broken_slot_reversed_growth: &'static str,
}

const FIRST_RERUN_LINES: LineNames = LineNames {
    world: "w_cell",
    activation: "w_activation",
    near_full: "w_near_full",
    drain_failure: "w_drain_failure",
    broken_slot: "w_broken_slot",
    cost_growth: "w_cost_growth",
    activation_reversed: "w_activation_h2",
    broken_slot_reversed: "w_broken_slot_h1",
    crit1_push: "w_crit1_push",
    trajectory: "w_trajectory",
    detail_activation: "w_detail_activation",
    detail_drain: "w_detail_drain",
    readings_diff_row: "w_readings_diff_row",
    verdict_window: "verdict_window",
    disk_stuck: "",
    disk_stuck_h3: "",
    stuck_growth: "",
    broken_slot_growth: "",
    broken_slot_reversed_growth: "",
};

const SECOND_RERUN_LINES: LineNames = LineNames {
    world: "f_cell",
    activation: "f_activation",
    near_full: "f_near_full",
    drain_failure: "f_drain_failure",
    broken_slot: "f_broken_slot",
    cost_growth: "f_cost_growth",
    activation_reversed: "f_activation_h2",
    broken_slot_reversed: "f_broken_slot_h1",
    crit1_push: "f_crit1_push",
    trajectory: "f_trajectory",
    detail_activation: "f_detail_activation",
    detail_drain: "f_detail_drain",
    readings_diff_row: "f_readings_diff_row",
    verdict_window: "verdict_f_window",
    disk_stuck: "f_disk_stuck",
    disk_stuck_h3: "f_disk_stuck_h3",
    stuck_growth: "f_stuck_growth",
    broken_slot_growth: "f_broken_slot_growth",
    broken_slot_reversed_growth: "f_broken_slot_h1_growth",
};

/// 一族臂：进全部格的臂、只进 steady 的阳性对照，与几条对照关系里的角色（自证对照、两条读法臂）。
struct RowFamily {
    lines: &'static LineNames,
    arms: Vec<Arm>,
    short_arms: Vec<Arm>,
    window_arm: Arm,
    ring_gate_arm: Arm,
    disk_hold_arm: Arm,
    ring_lag_arm: Arm,
    /// 第 2 次重跑一族：多打 `f_attribution` 行与 `disk_stuck`、刻度。
    is_second_rerun: bool,
}

/// 第 1 次重跑的一族：窗口臂四条加三条阳性对照，候选下界取旧式。
fn first_rerun_family() -> RowFamily {
    RowFamily {
        lines: &FIRST_RERUN_LINES,
        arms: WINDOW_ARMS.iter().map(|window_arm| Arm::TightenedStatesWindowed(*window_arm)).collect(),
        short_arms: WINDOW_SHORT_ARMS.iter().map(|window_arm| Arm::TightenedStatesWindowed(*window_arm)).collect(),
        window_arm: Arm::TightenedStatesWindowed(WindowArm::Window),
        ring_gate_arm: Arm::TightenedStatesWindowed(WindowArm::RingGate),
        disk_hold_arm: Arm::TightenedStatesWindowed(WindowArm::DiskHold),
        ring_lag_arm: Arm::TightenedStatesWindowed(WindowArm::RingLag),
        is_second_rerun: false,
    }
}

/// 第 2 次重跑的一族：同一批窗口臂候选下界取条款形，再加 RGall 两条被测臂与它们的阳性对照。
fn second_rerun_family() -> RowFamily {
    let mut arms: Vec<Arm> = WINDOW_ARMS.iter().map(|window_arm| Arm::ClauseFloorWindowed(*window_arm)).collect();
    arms.extend(WHOLE_RING_ARMS.iter().map(|whole_ring_arm| Arm::WholeRing(*whole_ring_arm)));
    let mut short_arms: Vec<Arm> = WINDOW_SHORT_ARMS.iter().map(|window_arm| Arm::ClauseFloorWindowed(*window_arm)).collect();
    short_arms.extend(WHOLE_RING_SHORT_ARMS.iter().map(|whole_ring_arm| Arm::WholeRing(*whole_ring_arm)));
    RowFamily {
        lines: &SECOND_RERUN_LINES,
        arms,
        short_arms,
        window_arm: Arm::ClauseFloorWindowed(WindowArm::Window),
        ring_gate_arm: Arm::ClauseFloorWindowed(WindowArm::RingGate),
        disk_hold_arm: Arm::ClauseFloorWindowed(WindowArm::DiskHold),
        ring_lag_arm: Arm::ClauseFloorWindowed(WindowArm::RingLag),
        is_second_rerun: true,
    }
}

fn activation_class(variant: ActivationVariant) -> &'static str {
    match variant {
        ActivationVariant::Plain => "activation_faults",
        ActivationVariant::CrashWithoutLossFirst => "activation_crash_no_loss",
        ActivationVariant::SkipThenRaise { .. } => "skip_then_raise",
    }
}

/// 第六节第 8–10 行三个量各自「是否 > 0」；假性 ENOSPC 只在判的格里算进来，不判的格这一位恒 false（数在行里另报）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct RowJudgement {
    has_fake_candidates: bool,
    has_unrecoverable: bool,
    has_false_enospc_in_a_judged_row: bool,
}

impl RowJudgement {
    fn of(metrics: &Metrics, is_false_enospc_judged: bool) -> Self {
        RowJudgement {
            has_fake_candidates: metrics.fake_candidates > 0,
            has_unrecoverable: metrics.unrecoverable > 0,
            has_false_enospc_in_a_judged_row: is_false_enospc_judged && metrics.false_enospc_arm > 0,
        }
    }
}

/// 产物里的一行（一个世界、一条臂、一组参数；种子在行内合计）。
struct WindowRow {
    line_name: &'static str,
    world_class: String,
    /// 行键：世界 / 变体、S、开销对、丢盘、失败组合，不含臂。
    row_key: String,
    arm: Arm,
    /// 整行去掉臂字段之后的文本：第六节第 7 行 G2SW 与 G2SW_ringgate 逐字比它。
    text_without_arm: String,
    judgement: RowJudgement,
    is_report_only_world: bool,
    gate_difference_positive_samples: u64,
    /// 第 2 次重跑逐行比、归因与代价汇总用：这一行（种子合计之后）的全部计数。
    metrics: Metrics,
    /// 这类行的结果字段里有没有 `false_enospc_arm`（活化一族的行没有，逐行比时不比这一项）。
    has_false_enospc_field: bool,
}

struct WindowRowDraft<'metrics> {
    line_name: &'static str,
    world_class: &'metrics str,
    row_key: String,
    arm: Arm,
    outcome_fields: String,
    judgement: RowJudgement,
    is_report_only_world: bool,
    has_false_enospc_field: bool,
    metrics: &'metrics Metrics,
}

/// 打印一行结果与它的轨迹行（第 2 次重跑的一族再打一行 `f_attribution`），并记进行簿。
fn emit_window_row(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, family: &RowFamily, draft: WindowRowDraft) {
    println!("{}", emitter.emit_raw(&format!("name={} arm={} {} {}", draft.line_name, draft.arm.name(), draft.row_key, draft.outcome_fields)));
    let new_costs = if family.is_second_rerun { format!(" {}", draft.metrics.second_rerun.render_costs()) } else { String::new() };
    println!("{}", emitter.emit_raw(&format!(
        "name={} line={} arm={} {} {}{new_costs}", family.lines.trajectory, draft.line_name, draft.arm.name(), draft.row_key, draft.metrics.trajectory.render()
    )));
    if family.is_second_rerun {
        println!("{}", emitter.emit_raw(&format!(
            "name=f_attribution line={} arm={} {} {}", draft.line_name, draft.arm.name(), draft.row_key, draft.metrics.second_rerun.render_attribution()
        )));
    }
    rows.push(WindowRow {
        line_name: draft.line_name,
        world_class: draft.world_class.to_string(),
        text_without_arm: format!("name={} {} {}", draft.line_name, draft.row_key, draft.outcome_fields),
        row_key: draft.row_key,
        arm: draft.arm,
        judgement: draft.judgement,
        is_report_only_world: draft.is_report_only_world,
        gate_difference_positive_samples: draft.metrics.trajectory.gate_difference.positive_samples,
        metrics: *draft.metrics,
        has_false_enospc_field: draft.has_false_enospc_field,
    });
}

fn world_outcome_fields(total: &Metrics) -> String {
    format!(
        "fakes={} unrecoverable={} retention_violations={} checks={} min_candidates={} avg_candidates_x100={} avg_nonempty_x100={} min_states={} \
         avg_states_x100={} stalls={} false_enospc_arm={}",
        total.fake_candidates, total.unrecoverable, total.retention_violations, total.checks, total.minimum_candidates_or_zero(),
        total.average_candidates_in_hundredths(), total.average_nonempty_candidates_in_hundredths(), total.minimum_distinct_states_or_zero(),
        total.average_distinct_states_in_hundredths(), total.stalls, total.false_enospc_arm
    )
}

fn activation_outcome_fields(total: &Metrics) -> String {
    format!("fakes={} unrecoverable={} retention_violations={} checks={}", total.fake_candidates, total.unrecoverable, total.retention_violations, total.checks)
}

fn fresh_total_over_seeds(seed_count: u64, mut run_seed: impl FnMut(u64) -> Metrics) -> Metrics {
    let mut total = Metrics::fresh();
    for seed in 0..seed_count {
        total.add(&run_seed(seed));
    }
    total
}

/// 六个基本世界 × S × 丢盘 × 四条臂；阳性对照臂只进 steady、S ∈ {4, 16}（跑前登记第五节）。
fn emit_window_world_rows(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, family: &RowFamily) {
    for world in WORLDS {
        for slots_per_region in SLOTS_PER_REGION_SWEEP {
            for arm in family.arms.iter().copied() {
                let lost_disks: &[usize] = if world == World::DiskLoss { &[0, 1] } else { &[0] };
                for &lost_disk in lost_disks {
                    let total = fresh_total_over_seeds(SEED_COUNT, |seed| run_world(arm, world, slots_per_region, seed, lost_disk));
                    emit_window_row(emitter, rows, family, WindowRowDraft {
                        line_name: family.lines.world,
                        world_class: world.name(),
                        row_key: format!("world={} s={slots_per_region} lost_disk={lost_disk}", world.name()),
                        arm,
                        outcome_fields: world_outcome_fields(&total),
                        judgement: RowJudgement::of(&total, false),
                        is_report_only_world: world == World::RollbackFirstPublish,
                        has_false_enospc_field: true,
                        metrics: &total,
                    });
                }
            }
        }
    }
    for arm in family.short_arms.iter().copied() {
        for slots_per_region in WINDOW_SHORT_SLOTS_SWEEP {
            let total = fresh_total_over_seeds(SEED_COUNT, |seed| run_world(arm, World::Steady, slots_per_region, seed, 0));
            emit_window_row(emitter, rows, family, WindowRowDraft {
                line_name: family.lines.world,
                world_class: World::Steady.name(),
                row_key: format!("world={} s={slots_per_region} lost_disk=0", World::Steady.name()),
                arm,
                outcome_fields: world_outcome_fields(&total),
                judgement: RowJudgement::of(&total, false),
                is_report_only_world: false,
                has_false_enospc_field: true,
                metrics: &total,
            });
        }
    }
}

/// 活化一族：原取样点五种变体，或 H2（跳号换成区域 0，只跑 skip_then_raise 三种）。
fn emit_window_activation_rows(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, family: &RowFamily, point: GeometryPoint) {
    let patterns = failure_patterns();
    let (line_name, variants): (&'static str, Vec<ActivationVariant>) = match point {
        GeometryPoint::Original => {
            let mut variants = vec![ActivationVariant::Plain, ActivationVariant::CrashWithoutLossFirst];
            variants.extend(SKIP_THEN_RAISE_REPEATS.iter().map(|repeats| ActivationVariant::SkipThenRaise { repeats: *repeats }));
            (family.lines.activation, variants)
        }
        GeometryPoint::Reversed => (family.lines.activation_reversed, SKIP_THEN_RAISE_REPEATS.iter().map(|repeats| ActivationVariant::SkipThenRaise { repeats: *repeats }).collect()),
    };
    let mut detail_lines = 0u64;
    for variant in variants {
        let skip_repeats = match variant {
            ActivationVariant::SkipThenRaise { repeats } => repeats,
            ActivationVariant::Plain | ActivationVariant::CrashWithoutLossFirst => 0,
        };
        for slots_per_region in FAULT_SLOTS_SWEEP {
            for arm in family.arms.iter().copied() {
                let mut by_class = [[Metrics::fresh(); 3]; DISK_COUNT];
                for seed in 0..FAULT_SEED_COUNT {
                    let prefix = match point {
                        GeometryPoint::Original => activation_prefix(arm, slots_per_region, seed, skip_repeats),
                        GeometryPoint::Reversed => activation_prefix_with_jump_region(arm, slots_per_region, seed, skip_repeats, 0),
                    };
                    let Some(prefix) = prefix else {
                        continue;
                    };
                    for pattern in &patterns {
                        for crash_window in 1..=ACTIVATION_FAULT_CRASH_WINDOWS {
                            for lost_disk in 0..DISK_COUNT {
                                let case = run_activation_case(&prefix, pattern, crash_window, lost_disk, variant);
                                by_class[lost_disk][pattern.len()].add(&case);
                                if (case.fake_candidates > 0 || case.unrecoverable > 0) && detail_lines < DETAIL_LINES_MAXIMUM {
                                    detail_lines += 1;
                                    println!("{}", emitter.emit_raw(&format!(
                                        "name={} line={line_name} variant={} arm={} s={slots_per_region} seed={seed} failures={} crash_window={crash_window} \
                                         lost_disk={lost_disk} fakes={} unrecoverable={}",
                                        family.lines.detail_activation, variant.name(), arm.name(), format_set(pattern), case.fake_candidates, case.unrecoverable
                                    )));
                                }
                            }
                        }
                    }
                }
                for (lost_disk, classes) in by_class.iter().enumerate() {
                    for (failure_count, total) in classes.iter().enumerate() {
                        emit_window_row(emitter, rows, family, WindowRowDraft {
                            line_name,
                            world_class: activation_class(variant),
                            row_key: format!("variant={} s={slots_per_region} lost_disk={lost_disk} failures={failure_count}", variant.name()),
                            arm,
                            outcome_fields: activation_outcome_fields(total),
                            judgement: RowJudgement::of(total, false),
                            is_report_only_world: false,
                            has_false_enospc_field: false,
                            metrics: total,
                        });
                    }
                }
            }
        }
    }
}

/// near_full：两档开销对 × S × 四条臂（判假性 ENOSPC）。
fn emit_window_near_full_rows(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, family: &RowFamily) {
    for slots_per_region in SLOTS_PER_REGION_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in family.arms.iter().copied() {
                let ring_depth = REGION_COUNT * slots_per_region;
                let total = fresh_total_over_seeds(SEED_COUNT, |seed| run_near_full(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, None));
                let formula_cost = arm.formula_cost(mount_cost, declared_maximum_cost);
                let reserve = arm.reserve_blocks(ring_depth, formula_cost);
                let residue = arm.push_residue_blocks(ring_depth, formula_cost);
                let bound = arm.publication_bound(ring_depth);
                let outcome_fields = format!(
                    "reserve={reserve} residue={residue} hidden_blocks={} hidden_bytes={} avg_lag_x100={} max_lag={} false_enospc_raw={} false_enospc_arm={} \
                     true_enospc={} write_after_delete_failed={} first_failure_window={} last_failure_window={} max_publications={} bound={bound} \
                     admissions_under_pressure={} forced_per_window_x100={} stalls={} halted_before_counting={} fakes={} unrecoverable={} \
                     retention_violations={} min_candidates={} avg_candidates_x100={} avg_nonempty_x100={} min_states={} avg_states_x100={}",
                    reserve + residue, (reserve + residue) * BLOCK_BYTES, total.average_lag_in_hundredths(), total.lag_maximum,
                    total.false_enospc_raw, total.false_enospc_arm, total.true_enospc, total.write_after_delete_failed,
                    total.first_failure_window_or_zero(), total.last_write_after_delete_failure_window, total.maximum_publications_per_admission,
                    total.admissions_under_pressure, total.forced_publications_per_window_in_hundredths(), total.stalls, total.halted_before_counting,
                    total.fake_candidates, total.unrecoverable, total.retention_violations, total.minimum_candidates_or_zero(),
                    total.average_candidates_in_hundredths(), total.average_nonempty_candidates_in_hundredths(),
                    total.minimum_distinct_states_or_zero(), total.average_distinct_states_in_hundredths()
                );
                emit_window_row(emitter, rows, family, WindowRowDraft {
                    line_name: family.lines.near_full,
                    world_class: NEAR_FULL_CLASS,
                    row_key: format!("s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost}"),
                    arm,
                    outcome_fields,
                    judgement: RowJudgement::of(&total, true),
                    is_report_only_world: false,
                    has_false_enospc_field: true,
                    metrics: &total,
                });
            }
        }
    }
}

/// drain_failure：失败 1、2、3 次（判假性 ENOSPC 只对 ≤ k_tol 的格）。
fn emit_window_drain_failure_rows(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, family: &RowFamily) {
    let mut detail_lines = 0u64;
    for slots_per_region in FAULT_SLOTS_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in family.arms.iter().copied() {
                let ring_depth = REGION_COUNT * slots_per_region;
                let bound = arm.publication_bound(ring_depth);
                let drain_patterns = drain_failure_patterns(arm.drain_failure_offset_limit(ring_depth));
                let mut by_count = [Metrics::fresh(); 4];
                let mut cases_by_count = [0u64; 4];
                let mut runs_with_false = [0u64; 4];
                let mut runs_with_write_after_delete = [0u64; 4];
                for seed in 0..FAULT_SEED_COUNT {
                    let Some(prefix) = drain_failure_prefix(arm, slots_per_region, mount_cost, declared_maximum_cost, seed) else {
                        continue;
                    };
                    for pattern in &drain_patterns {
                        let case = run_drain_failure_case(&prefix, pattern);
                        by_count[pattern.len()].add(&case);
                        cases_by_count[pattern.len()] += 1;
                        runs_with_false[pattern.len()] += u64::from(case.false_enospc_arm > 0);
                        runs_with_write_after_delete[pattern.len()] += u64::from(case.write_after_delete_failed > 0);
                        if (case.false_enospc_arm > 0 || case.fake_candidates > 0 || case.unrecoverable > 0) && detail_lines < 2 * DETAIL_LINES_MAXIMUM {
                            detail_lines += 1;
                            println!("{}", emitter.emit_raw(&format!(
                                "name={} arm={} s={slots_per_region} c0={mount_cost} seed={seed} failures={} false_enospc_arm={} fakes={} unrecoverable={} \
                                 write_after_delete_failed={} max_publications={} stalls={}",
                                family.lines.detail_drain, arm.name(), format_set(pattern), case.false_enospc_arm, case.fake_candidates, case.unrecoverable,
                                case.write_after_delete_failed, case.maximum_publications_per_admission, case.stalls
                            )));
                        }
                    }
                }
                for failure_count in 1..=3usize {
                    let total = by_count[failure_count];
                    let outcome_fields = format!(
                        "cases={} runs_with_false_enospc={} runs_with_write_after_delete_failed={} false_enospc_arm={} false_enospc_raw={} \
                         write_after_delete_failed={} max_publications={} bound={bound} stalls={} fakes={} unrecoverable={} retention_violations={} injected={}",
                        cases_by_count[failure_count], runs_with_false[failure_count], runs_with_write_after_delete[failure_count],
                        total.false_enospc_arm, total.false_enospc_raw, total.write_after_delete_failed, total.maximum_publications_per_admission,
                        total.stalls, total.fake_candidates, total.unrecoverable, total.retention_violations, total.root_write_failures_injected
                    );
                    emit_window_row(emitter, rows, family, WindowRowDraft {
                        line_name: family.lines.drain_failure,
                        world_class: DRAIN_FAILURE_CLASS,
                        row_key: format!("s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost} failures={failure_count}"),
                        arm,
                        outcome_fields,
                        judgement: RowJudgement::of(&total, failure_count as u64 <= TOLERATED_ROOT_WRITE_FAILURES),
                        is_report_only_world: false,
                        has_false_enospc_field: true,
                        metrics: &total,
                    });
                }
            }
        }
    }
}

/// broken_slot：原取样点（槽 1，区域 0、盘 0）或 H1（槽 S，区域 1、盘 1）。
fn emit_window_broken_slot_rows(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, growth: &mut Vec<StuckGrowthSummary>, family: &RowFamily, point: GeometryPoint) {
    let (line_name, growth_line_name) = match point {
        GeometryPoint::Original => (family.lines.broken_slot, family.lines.broken_slot_growth),
        GeometryPoint::Reversed => (family.lines.broken_slot_reversed, family.lines.broken_slot_reversed_growth),
    };
    for slots_per_region in FAULT_SLOTS_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in family.arms.iter().copied() {
                let ring_depth = REGION_COUNT * slots_per_region;
                let bound = arm.publication_bound(ring_depth);
                let mut total = Metrics::fresh();
                let mut seeds_with_no_space = 0u64;
                let mut seeds_halted = 0u64;
                let mut seed_marks: Vec<[Option<StuckMark>; 3]> = Vec::new();
                for seed in 0..FAULT_SEED_COUNT {
                    let one = match point {
                        GeometryPoint::Original => run_broken_slot(arm, slots_per_region, mount_cost, declared_maximum_cost, seed),
                        GeometryPoint::Reversed => {
                            let first_slot_of_region_one = usize::try_from(slots_per_region).expect("槽数装得进 usize");
                            run_broken_slot_at(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, first_slot_of_region_one)
                        }
                    };
                    seeds_with_no_space += u64::from(one.write_after_delete_failed > 0);
                    seeds_halted += u64::from(one.stalls > 0);
                    seed_marks.push(one.second_rerun.marks);
                    total.add(&one);
                }
                let outcome_fields = format!(
                    "seeds={FAULT_SEED_COUNT} seeds_with_no_space={seeds_with_no_space} seeds_halted={seeds_halted} first_failure_window={} false_enospc_arm={} \
                     false_enospc_raw={} write_after_delete_failed={} max_publications={} bound={bound} stalls={} fakes={} unrecoverable={} \
                     retention_violations={} injected={} forced_per_window_x100={} max_lag={}",
                    total.first_failure_window_or_zero(), total.false_enospc_arm, total.false_enospc_raw, total.write_after_delete_failed,
                    total.maximum_publications_per_admission, total.stalls, total.fake_candidates, total.unrecoverable, total.retention_violations,
                    total.root_write_failures_injected, total.forced_publications_per_window_in_hundredths(), total.lag_maximum
                );
                let row_key = format!("s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost}");
                if family.is_second_rerun {
                    growth.push(emit_stuck_growth_row(emitter, growth_line_name, arm, &row_key, &seed_marks));
                }
                emit_window_row(emitter, rows, family, WindowRowDraft {
                    line_name,
                    world_class: BROKEN_SLOT_CLASS,
                    row_key,
                    arm,
                    outcome_fields,
                    judgement: RowJudgement::of(&total, true),
                    is_report_only_world: false,
                    has_false_enospc_field: true,
                    metrics: &total,
                });
            }
        }
    }
}

/// cost_growth：开销升到声明的上界（判）与超过上界 3 块（只报）。
fn emit_window_cost_growth_rows(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, family: &RowFamily) {
    for slots_per_region in FAULT_SLOTS_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in family.arms.iter().copied() {
                let ring_depth = REGION_COUNT * slots_per_region;
                let bound = arm.publication_bound(ring_depth);
                for grown_cost in [declared_maximum_cost, declared_maximum_cost + COST_OVER_DECLARED_EXTRA] {
                    let is_within_declared = grown_cost <= declared_maximum_cost;
                    let total = fresh_total_over_seeds(FAULT_SEED_COUNT, |seed| run_cost_growth(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, grown_cost));
                    let outcome_fields = format!(
                        "false_enospc_arm={} false_enospc_raw={} true_enospc={} write_after_delete_failed={} first_failure_window={} max_publications={} bound={bound} \
                         stalls={} fakes={} unrecoverable={} retention_violations={} forced_per_window_x100={}",
                        total.false_enospc_arm, total.false_enospc_raw, total.true_enospc, total.write_after_delete_failed, total.first_failure_window_or_zero(),
                        total.maximum_publications_per_admission, total.stalls, total.fake_candidates, total.unrecoverable, total.retention_violations,
                        total.forced_publications_per_window_in_hundredths()
                    );
                    emit_window_row(emitter, rows, family, WindowRowDraft {
                        line_name: family.lines.cost_growth,
                        world_class: COST_GROWTH_CLASS,
                        row_key: format!("s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost} grown_to={grown_cost} within_declared={}", u8::from(is_within_declared)),
                        arm,
                        outcome_fields,
                        judgement: RowJudgement::of(&total, is_within_declared),
                        is_report_only_world: false,
                        has_false_enospc_field: true,
                        metrics: &total,
                    });
                }
            }
        }
    }
}

/// 旧登记判据 1 的跑前式子，套在新臂上：steady 里环没有洞，窗口等于整环，发布数、扣住块数与滞后量应与 G2S 一样。返回不成立的行数。
fn emit_window_crit1_rows(emitter: &mut Emitter, family: &RowFamily) -> (u64, Vec<String>) {
    let mut rows_not_holding = 0u64;
    let mut line_bodies: Vec<String> = Vec::new();
    for arm in family.arms.iter().copied() {
        for slots_per_region in SLOTS_PER_REGION_SWEEP {
            for (cost, _) in COST_PAIRS {
                let (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released) = push_probe_over_landings(arm, slots_per_region, cost);
                let (expect_set, expect_maximum, expect_pinned, expect_lag) = expected_push(arm, slots_per_region, cost);
                let holds = is_all_released && maximum_publications == expect_maximum && maximum_pinned == expect_pinned && maximum_lag == expect_lag
                    && publication_set.iter().all(|publications| expect_set.contains(publications));
                rows_not_holding += u64::from(!holds);
                let line_body = format!(
                    "arm={} s={slots_per_region} empty_cost={cost} publications={} max_publications={maximum_publications} \
                     expect_publications={} expect_max={expect_maximum} max_pinned={maximum_pinned} expect_pinned={expect_pinned} \
                     lag={maximum_lag} expect_lag={expect_lag} released={} holds={}",
                    arm.name(), format_set(&publication_set), format_set(&expect_set), u8::from(is_all_released), u8::from(holds)
                );
                println!("{}", emitter.emit_raw(&format!("name={} {line_body}", family.lines.crit1_push)));
                line_bodies.push(line_body);
            }
        }
    }
    (rows_not_holding, line_bodies)
}

fn is_original_sampling_row(row: &WindowRow, family: &RowFamily) -> bool {
    row.line_name != family.lines.activation_reversed && row.line_name != family.lines.broken_slot_reversed && row.line_name != family.lines.disk_stuck_h3
}

fn count_rows(rows: &[&WindowRow], predicate: impl Fn(&WindowRow) -> bool) -> usize {
    rows.iter().filter(|row| predicate(row)).count()
}

/// 第六节第 8–10 行：被测三条臂与旧门槛对照各自的逐行判定计数。
fn emit_window_row_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily, crit1_rows_not_holding: u64) {
    for arm in family.arms.iter().copied() {
        let arm_rows: Vec<&WindowRow> = rows.iter().filter(|row| row.arm == arm && is_original_sampling_row(row, family)).collect();
        println!("{}", emitter.emit_raw(&format!(
            "name={} arm={} rows={} rows_with_fake_candidates={} rows_with_unrecoverable={} judged_rows_with_false_enospc={} \
             rollback_first_publish_rows={} rollback_first_publish_rows_with_fake_candidates={} rollback_first_publish_rows_with_unrecoverable={} crit1_push_rows_not_holding={}",
            family.lines.verdict_window, arm.name(), arm_rows.len(),
            count_rows(&arm_rows, |row| !row.is_report_only_world && row.judgement.has_fake_candidates),
            count_rows(&arm_rows, |row| !row.is_report_only_world && row.judgement.has_unrecoverable),
            count_rows(&arm_rows, |row| row.judgement.has_false_enospc_in_a_judged_row),
            count_rows(&arm_rows, |row| row.is_report_only_world),
            count_rows(&arm_rows, |row| row.is_report_only_world && row.judgement.has_fake_candidates),
            count_rows(&arm_rows, |row| row.is_report_only_world && row.judgement.has_unrecoverable),
            crit1_rows_not_holding
        )));
    }
    for arm in family.short_arms.iter().copied() {
        let arm_rows: Vec<&WindowRow> = rows.iter().filter(|row| row.arm == arm).collect();
        println!("{}", emitter.emit_raw(&format!(
            "name={}_short arm={} rows={} rows_where_the_detector_saw_zero_fake_candidates={}",
            family.lines.verdict_window, arm.name(), arm_rows.len(), count_rows(&arm_rows, |row| !row.judgement.has_fake_candidates)
        )));
    }
}

/// 第六节第 6 行：G2SW 每类世界里门槛差为正的次数；注入根槽写失败的六类里合计为 0 的类数。
fn emit_window_gate_reach_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) {
    let window_rows: Vec<&WindowRow> = rows.iter().filter(|row| row.arm == family.window_arm && is_original_sampling_row(row, family)).collect();
    let mut classes: Vec<&str> = Vec::new();
    for row in &window_rows {
        if !classes.contains(&row.world_class.as_str()) {
            classes.push(row.world_class.as_str());
        }
    }
    let mut injection_classes_without_window_effect = 0u64;
    for class in classes {
        let class_rows: Vec<&&WindowRow> = window_rows.iter().filter(|row| row.world_class == class).collect();
        let positive_samples_total: u64 = class_rows.iter().map(|row| row.gate_difference_positive_samples).sum();
        let is_injecting = WORLD_CLASSES_INJECTING_ROOT_WRITE_FAILURES.contains(&class);
        injection_classes_without_window_effect += u64::from(is_injecting && positive_samples_total == 0);
        println!("{}", emitter.emit_raw(&format!(
            "name={}_gate_reach class={class} injects_root_write_failures={} rows={} rows_with_positive_gate_difference={} positive_samples_total={positive_samples_total}",
            family.lines.verdict_window, u8::from(is_injecting), class_rows.len(), class_rows.iter().filter(|row| row.gate_difference_positive_samples > 0).count()
        )));
    }
    println!("{}", emitter.emit_raw(&format!(
        "name={}_gate_reach_summary injection_classes={} injection_classes_without_window_effect={injection_classes_without_window_effect}",
        family.lines.verdict_window, WORLD_CLASSES_INJECTING_ROOT_WRITE_FAILURES.len()
    )));
}

/// 第六节第 7 行：同一组参数上 G2SW_ringgate 那一行与 G2SW 那一行，除臂名外整行比。
fn emit_window_self_proof_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) {
    let ringgate_text_by_key: HashMap<(&str, &str), &str> = rows.iter()
        .filter(|row| row.arm == family.ring_gate_arm)
        .map(|row| ((row.line_name, row.row_key.as_str()), row.text_without_arm.as_str())).collect();
    let mut compared = 0u64;
    let mut differing = 0u64;
    let mut differing_by_class: Vec<(String, u64)> = Vec::new();
    for row in rows.iter().filter(|row| row.arm == family.window_arm && is_original_sampling_row(row, family)) {
        let Some(ringgate_text) = ringgate_text_by_key.get(&(row.line_name, row.row_key.as_str())) else {
            continue;
        };
        compared += 1;
        if *ringgate_text != row.text_without_arm.as_str() {
            differing += 1;
            match differing_by_class.iter_mut().find(|(class, _)| *class == row.world_class) {
                Some((_, count)) => *count += 1,
                None => differing_by_class.push((row.world_class.clone(), 1)),
            }
        }
    }
    for (class, count) in &differing_by_class {
        println!("{}", emitter.emit_raw(&format!("name={}_self_proof_class class={class} rows_differing_from_ringgate={count}", family.lines.verdict_window)));
    }
    println!("{}", emitter.emit_raw(&format!("name={}_self_proof rows_compared={compared} rows_differing_from_ringgate={differing}", family.lines.verdict_window)));
}

/// 第六节第 12 行：G2SW_Q、G2SW_L 各与 G2SW 的逐行判定比。
fn emit_window_reading_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) {
    let window_judgement_by_key: HashMap<(&str, &str), RowJudgement> = rows.iter()
        .filter(|row| row.arm == family.window_arm && is_original_sampling_row(row, family))
        .map(|row| ((row.line_name, row.row_key.as_str()), row.judgement)).collect();
    let mut detail_lines = 0u64;
    for reading_arm in [family.disk_hold_arm, family.ring_lag_arm] {
        let mut compared = 0u64;
        let mut differing = 0u64;
        for row in rows.iter().filter(|row| row.arm == reading_arm && is_original_sampling_row(row, family)) {
            let Some(window_judgement) = window_judgement_by_key.get(&(row.line_name, row.row_key.as_str())) else {
                continue;
            };
            compared += 1;
            if *window_judgement != row.judgement {
                differing += 1;
                if detail_lines < 2 * DETAIL_LINES_MAXIMUM {
                    detail_lines += 1;
                    println!("{}", emitter.emit_raw(&format!(
                        "name={} reading_arm={} line={} {} window_judgement={window_judgement:?} reading_judgement={:?}",
                        family.lines.readings_diff_row, reading_arm.name(), row.line_name, row.row_key, row.judgement
                    )));
                }
            }
        }
        println!("{}", emitter.emit_raw(&format!(
            "name={}_readings reading_arm={} rows_compared={compared} rows_with_a_judgement_different_from_window={differing}", family.lines.verdict_window, reading_arm.name()
        )));
    }
}

/// 第八节几何敏感性：H1（坏槽换到盘 1）、H2（跳号换到盘 0）的逐行判定与原取样点比。
fn emit_window_geometry_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) {
    let mut points = vec![("h1", family.lines.broken_slot_reversed, family.lines.broken_slot), ("h2", family.lines.activation_reversed, family.lines.activation)];
    if family.is_second_rerun {
        points.push(("h3", family.lines.disk_stuck_h3, family.lines.disk_stuck));
    }
    for arm in family.arms.iter().copied() {
        for (point_name, reversed_line, original_line) in points.iter().copied() {
            let original_judgement_by_key: HashMap<&str, RowJudgement> = rows.iter()
                .filter(|row| row.arm == arm && row.line_name == original_line)
                .map(|row| (row.row_key.as_str(), row.judgement)).collect();
            let mut compared = 0u64;
            let mut differing = 0u64;
            for row in rows.iter().filter(|row| row.arm == arm && row.line_name == reversed_line) {
                let Some(original_judgement) = original_judgement_by_key.get(row.row_key.as_str()) else {
                    continue;
                };
                compared += 1;
                differing += u64::from(*original_judgement != row.judgement);
            }
            if compared == 0 && family.is_second_rerun {
                continue;
            }
            println!("{}", emitter.emit_raw(&format!(
                "name={}_geometry point={point_name} arm={} rows_compared={compared} rows_with_a_judgement_different_from_the_original_point={differing}",
                family.lines.verdict_window, arm.name()
            )));
        }
    }
}

/// 第 1 次重跑的全部新臂行与它们的判决行；接在原产物全部行之后、`name=done` 之前。返回行簿给第 2 次重跑逐行比。
fn run_window_arms(emitter: &mut Emitter) -> (Vec<WindowRow>, Vec<String>) {
    let family = first_rerun_family();
    let family = &family;
    println!("{}", emitter.emit_raw(&format!(
        "name=w_config window_arms={} short_arms={} short_slots={} window_length_rule=txg_plus_ring_depth_greater_than_anchor short_window_length_rule=txg_plus_ring_depth_minus_regions_greater_than_anchor \
         geometry_points=h1/h2 seeds={SEED_COUNT} fault_seeds={FAULT_SEED_COUNT}",
        WINDOW_ARMS.iter().map(|arm| arm.name()).collect::<Vec<&str>>().join("/"),
        WINDOW_SHORT_ARMS.iter().map(|arm| arm.name()).collect::<Vec<&str>>().join("/"),
        format_set(&WINDOW_SHORT_SLOTS_SWEEP)
    )));
    let mut rows: Vec<WindowRow> = Vec::new();
    let mut unused_growth: Vec<StuckGrowthSummary> = Vec::new();
    let (crit1_rows_not_holding, crit1_line_bodies) = emit_window_crit1_rows(emitter, family);
    emit_window_world_rows(emitter, &mut rows, family);
    emit_window_activation_rows(emitter, &mut rows, family, GeometryPoint::Original);
    emit_window_near_full_rows(emitter, &mut rows, family);
    emit_window_drain_failure_rows(emitter, &mut rows, family);
    emit_window_broken_slot_rows(emitter, &mut rows, &mut unused_growth, family, GeometryPoint::Original);
    emit_window_cost_growth_rows(emitter, &mut rows, family);
    emit_window_activation_rows(emitter, &mut rows, family, GeometryPoint::Reversed);
    emit_window_broken_slot_rows(emitter, &mut rows, &mut unused_growth, family, GeometryPoint::Reversed);
    emit_window_row_verdicts(emitter, &rows, family, crit1_rows_not_holding);
    emit_window_gate_reach_verdicts(emitter, &rows, family);
    emit_window_self_proof_verdicts(emitter, &rows, family);
    emit_window_reading_verdicts(emitter, &rows, family);
    emit_window_geometry_verdicts(emitter, &rows, family);
    (rows, crit1_line_bodies)
}

/// 一行刻度与斜率的判（登记第六节第 17 行）：8 个种子的 s(2→4) 都小于 1/2 为有上界；有一个不小于就是没有上界；一个种子都没有 s(2→4) 时不判。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StuckBound {
    Bounded,
    Unbounded,
    NotJudged,
}

impl StuckBound {
    fn name(self) -> &'static str {
        match self {
            StuckBound::Bounded => "bounded",
            StuckBound::Unbounded => "unbounded",
            StuckBound::NotJudged => "not_judged",
        }
    }
}

struct StuckGrowthSummary {
    line_name: &'static str,
    arm: Arm,
    row_key: String,
    bound: StuckBound,
}

/// 每个种子两个刻度之间的（扣住的增量，放掉的块数）；两个刻度没都走到的种子不进。
fn marks_slopes(seed_marks: &[[Option<StuckMark>; 3]], from_index: usize, to_index: usize) -> Vec<(i64, i64)> {
    seed_marks.iter().filter_map(|marks| match (&marks[from_index], &marks[to_index]) {
        (Some(earlier), Some(later)) => stuck_slope_between(earlier, later),
        _ => None,
    }).collect()
}

fn slope_range_in_hundredths(slopes: &[(i64, i64)]) -> (String, String) {
    let hundredths: Vec<i64> = slopes.iter().map(|(increase, freed)| slope_in_hundredths(*increase, *freed)).collect();
    match (hundredths.iter().min(), hundredths.iter().max()) {
        (Some(smallest), Some(largest)) => (smallest.to_string(), largest.to_string()),
        _ => ("na".to_string(), "na".to_string()),
    }
}

fn render_stuck_marks(marks: &[Option<StuckMark>; 3]) -> String {
    STUCK_MARK_REVOLUTIONS.iter().zip(marks.iter()).map(|(revolutions, mark)| match mark {
        Some(mark) => format!("mark{revolutions}_txg={} mark{revolutions}_pinned={} mark{revolutions}_freed={}", mark.published_txg, mark.pinned_blocks, mark.blocks_freed_in_total),
        None => format!("mark{revolutions}_txg=na mark{revolutions}_pinned=na mark{revolutions}_freed=na"),
    }).collect::<Vec<String>>().join(" ")
}

/// 刻度与斜率一行（`f_stuck_growth`、`f_broken_slot_growth`、`f_broken_slot_h1_growth` 同形）：8 个种子合计的刻度、两段斜率的种子最小与最大（乘 100 取整）、没走到的种子数、判。
fn emit_stuck_growth_row(emitter: &mut Emitter, line_name: &'static str, arm: Arm, row_key: &str, seed_marks: &[[Option<StuckMark>; 3]]) -> StuckGrowthSummary {
    let mut mark_fields: Vec<String> = Vec::new();
    for (mark_index, revolutions) in STUCK_MARK_REVOLUTIONS.iter().enumerate() {
        let reached: Vec<&StuckMark> = seed_marks.iter().filter_map(|marks| marks[mark_index].as_ref()).collect();
        let pinned_sum: u64 = reached.iter().map(|mark| mark.pinned_blocks).sum();
        mark_fields.push(format!("mark{revolutions}_seeds_reached={} mark{revolutions}_pinned_sum={pinned_sum}", reached.len()));
    }
    let first_to_second = marks_slopes(seed_marks, 0, 1);
    let second_to_fourth = marks_slopes(seed_marks, 1, 2);
    let (first_to_second_smallest, first_to_second_largest) = slope_range_in_hundredths(&first_to_second);
    let (second_to_fourth_smallest, second_to_fourth_largest) = slope_range_in_hundredths(&second_to_fourth);
    let seeds_not_below_one_half = second_to_fourth.iter().filter(|(increase, freed)| !is_stuck_slope_bounded(*increase, *freed)).count();
    let bound = if second_to_fourth.is_empty() {
        StuckBound::NotJudged
    } else if seeds_not_below_one_half == 0 {
        StuckBound::Bounded
    } else {
        StuckBound::Unbounded
    };
    println!("{}", emitter.emit_raw(&format!(
        "name={line_name} arm={} {row_key} seeds={} {} s12_seeds={} s12_min_x100={first_to_second_smallest} s12_max_x100={first_to_second_largest} \
         s24_seeds={} s24_min_x100={second_to_fourth_smallest} s24_max_x100={second_to_fourth_largest} s24_seeds_not_below_one_half={seeds_not_below_one_half} \
         seeds_without_mark4={} bound={}",
        arm.name(), seed_marks.len(), mark_fields.join(" "), first_to_second.len(), second_to_fourth.len(),
        seed_marks.iter().filter(|marks| marks[STUCK_MARK_REVOLUTIONS.len() - 1].is_none()).count(), bound.name()
    )));
    StuckGrowthSummary { line_name, arm, row_key: row_key.to_string(), bound }
}

/// 收尾的三种形态（登记第五节「`disk_stuck`」收尾 e）：只崩不丢盘，或丢盘 0 / 1 之后恢复。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StuckEnding {
    CrashOnly,
    LostDisk(usize),
}

const STUCK_ENDINGS: [StuckEnding; 3] = [StuckEnding::CrashOnly, StuckEnding::LostDisk(0), StuckEnding::LostDisk(1)];

impl StuckEnding {
    fn name(self) -> String {
        match self {
            StuckEnding::CrashOnly => "crash".to_string(),
            StuckEnding::LostDisk(disk) => format!("lost_disk_{disk}"),
        }
    }
}

/// `disk_stuck` 的卡住段（登记第五节）：keep_snapshots 开，占用 `fill`，预热 2 × 3S + 10 个删建窗口，计数清零，记 T₀ = `next_txg`；
/// 之后 `stuck_disk` 上的根槽每次写都失败，做删建窗口直到第一个落盘 txg ≥ T₀ + 4 × 3S（第 4 个刻度走到）、停机，或做满 16 × 3S 个窗口。
fn disk_stuck_segment(arm: Arm, slots_per_region: u64, stuck_disk: usize, fill: f64, seed: u64) -> Simulation {
    let (mount_cost, declared_maximum_cost) = COST_PAIRS[0];
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, true, None);
    simulation.snapshots_kept_limit = STUCK_SNAPSHOTS_KEPT;
    simulation.prefill(fill);
    let warm_up = 2 * simulation.ring_depth + 10;
    for _ in 0..warm_up {
        simulation.run_user_window(true, false);
    }
    simulation.metrics = Metrics::fresh();
    simulation.counting_window = 0;
    simulation.stuck_disk = Some(stuck_disk);
    simulation.stuck_marks_origin_txg = Some(simulation.next_txg);
    let window_limit = STUCK_WINDOW_REVOLUTION_LIMIT * simulation.ring_depth;
    for _ in 0..window_limit {
        let is_last_mark_taken = simulation.metrics.second_rerun.marks[STUCK_MARK_REVOLUTIONS.len() - 1].is_some();
        if simulation.halted || is_last_mark_taken {
            break;
        }
        simulation.run_counted_delete_create_window();
    }
    simulation
}

/// `disk_stuck` 的收尾：从卡住段的末态拷一份，卡住解除，丢盘（若有）之后崩溃恢复，再做 2 × 3S 个删建窗口。卡住段已经停机的不收尾。
fn disk_stuck_ending(stuck_end: &Simulation, ending: StuckEnding) -> Metrics {
    let mut simulation = stuck_end.clone();
    simulation.stuck_disk = None;
    simulation.stuck_marks_origin_txg = None;
    if !simulation.halted {
        if let StuckEnding::LostDisk(disk) = ending {
            simulation.disk_lost[disk] = true;
        }
        simulation.crash_and_recover();
        let tail_windows = STUCK_TAIL_REVOLUTIONS * simulation.ring_depth;
        for _ in 0..tail_windows {
            if simulation.halted {
                break;
            }
            simulation.run_counted_delete_create_window();
        }
    }
    simulation.metrics
}

fn stuck_outcome_fields(total: &Metrics) -> String {
    format!(
        "seeds={FAULT_SEED_COUNT} fakes={} unrecoverable={} false_enospc_arm={} true_enospc={} write_after_delete_failed={} stalls={} injected={}",
        total.fake_candidates, total.unrecoverable, total.false_enospc_arm, total.true_enospc, total.write_after_delete_failed, total.stalls, total.root_write_failures_injected
    )
}

/// `disk_stuck`（登记第五节，第 2 行第三样代价的目标形态）：被测 RGall、RGall_Q 与对照 G2SW、G2SW_ringgate，全部开条款形候选下界。
fn emit_disk_stuck_rows(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, growth: &mut Vec<StuckGrowthSummary>, family: &RowFamily) {
    let stuck_arms = [
        Arm::WholeRing(WholeRingArm::ReadingP),
        Arm::WholeRing(WholeRingArm::ReadingQ),
        Arm::ClauseFloorWindowed(WindowArm::Window),
        Arm::ClauseFloorWindowed(WindowArm::RingGate),
    ];
    for stuck_disk in STUCK_DISKS {
        let (line_name, world_class) = if stuck_disk == 1 { (family.lines.disk_stuck, DISK_STUCK_CLASS) } else { (family.lines.disk_stuck_h3, DISK_STUCK_REVERSED_CLASS) };
        for slots_per_region in FAULT_SLOTS_SWEEP {
            for arm in stuck_arms {
                for fill in STUCK_FILLS {
                    let growth_key = format!("s={slots_per_region} stuck_disk={stuck_disk} fill={fill:.2}");
                    let mut totals_by_ending = [Metrics::fresh(); 3];
                    let mut seed_marks: Vec<[Option<StuckMark>; 3]> = Vec::new();
                    for seed in 0..FAULT_SEED_COUNT {
                        let stuck_end = disk_stuck_segment(arm, slots_per_region, stuck_disk, fill, seed);
                        let marks = stuck_end.metrics.second_rerun.marks;
                        seed_marks.push(marks);
                        let slope_text = |from_index: usize, to_index: usize| match marks_slopes(&[marks], from_index, to_index).first() {
                            Some((increase, freed)) => slope_in_hundredths(*increase, *freed).to_string(),
                            None => "na".to_string(),
                        };
                        println!("{}", emitter.emit_raw(&format!(
                            "name=f_stuck_marks line={line_name} arm={} {growth_key} seed={seed} {} s12_x100={} s24_x100={}",
                            arm.name(), render_stuck_marks(&marks), slope_text(0, 1), slope_text(1, 2)
                        )));
                        for (ending_index, ending) in STUCK_ENDINGS.iter().enumerate() {
                            totals_by_ending[ending_index].add(&disk_stuck_ending(&stuck_end, *ending));
                        }
                    }
                    growth.push(emit_stuck_growth_row(emitter, family.lines.stuck_growth, arm, &growth_key, &seed_marks));
                    for (ending_index, ending) in STUCK_ENDINGS.iter().enumerate() {
                        let total = &totals_by_ending[ending_index];
                        emit_window_row(emitter, rows, family, WindowRowDraft {
                            line_name,
                            world_class,
                            row_key: format!("s={slots_per_region} fill={fill:.2} ending={}", ending.name()),
                            arm,
                            outcome_fields: stuck_outcome_fields(total),
                            judgement: RowJudgement::of(total, false),
                            is_report_only_world: false,
                            has_false_enospc_field: true,
                            metrics: total,
                        });
                    }
                }
            }
        }
    }
}

fn line_suffix(line_name: &str) -> &str {
    line_name.get(2..).unwrap_or(line_name)
}

fn judgement_text(judgement: &RowJudgement) -> String {
    format!(
        "fake{}_unrecoverable{}_false_enospc{}",
        u8::from(judgement.has_fake_candidates), u8::from(judgement.has_unrecoverable), u8::from(judgement.has_false_enospc_in_a_judged_row)
    )
}

fn observed_or_na(value: i64) -> String {
    if value == i64::MIN { "na".to_string() } else { value.to_string() }
}

/// 行键里去掉某一个扫过的字段（按前缀认），剩下的拿来在两个取样点之间配行。
fn row_key_without_the_field(row_key: &str, field_prefix: &str) -> String {
    row_key.split(' ').filter(|token| !token.starts_with(field_prefix)).collect::<Vec<&str>>().join(" ")
}

/// 第六节第 6 行：变了的行归哪一类（第一节读法第 4 条）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RowChangeClass {
    /// 甲：只是数的口径变了。
    CountBasisOnly,
    /// 乙：走法也变了。
    WalkChanged,
    /// 丙：这一行下界差从没为正而行变了，装置错（V5）。
    FloorGapNeverPositive,
}

impl RowChangeClass {
    fn name(self) -> &'static str {
        match self {
            RowChangeClass::CountBasisOnly => "jia_count_basis_only",
            RowChangeClass::WalkChanged => "yi_walk_changed",
            RowChangeClass::FloorGapNeverPositive => "bing_floor_gap_never_positive",
        }
    }
}

fn classify_changed_row(first_row: &WindowRow, row: &WindowRow) -> RowChangeClass {
    let old = &first_row.metrics;
    let new = &row.metrics;
    let observations = &row.metrics.second_rerun;
    if observations.candidate_floor_gap.positive_samples == 0 {
        return RowChangeClass::FloorGapNeverPositive;
    }
    let is_same_walk = observations.rollback_target_changed == 0
        && observations.fakes_with_the_old_floor == old.fake_candidates
        && new.unrecoverable == old.unrecoverable
        && (!row.has_false_enospc_field || new.false_enospc_arm == old.false_enospc_arm);
    if is_same_walk { RowChangeClass::CountBasisOnly } else { RowChangeClass::WalkChanged }
}

#[derive(Clone, Copy, Debug)]
struct CompareTally {
    rows_compared: u64,
    rows_with_a_count_changed: u64,
    rows_with_a_judgement_changed: u64,
    count_basis_only: u64,
    walk_changed: u64,
    floor_gap_never_positive: u64,
}

impl CompareTally {
    fn fresh() -> Self {
        CompareTally { rows_compared: 0, rows_with_a_count_changed: 0, rows_with_a_judgement_changed: 0, count_basis_only: 0, walk_changed: 0, floor_gap_never_positive: 0 }
    }
    fn add_class(&mut self, class: RowChangeClass) {
        match class {
            RowChangeClass::CountBasisOnly => self.count_basis_only += 1,
            RowChangeClass::WalkChanged => self.walk_changed += 1,
            RowChangeClass::FloorGapNeverPositive => self.floor_gap_never_positive += 1,
        }
    }
    fn add(&mut self, other: &CompareTally) {
        self.rows_compared += other.rows_compared;
        self.rows_with_a_count_changed += other.rows_with_a_count_changed;
        self.rows_with_a_judgement_changed += other.rows_with_a_judgement_changed;
        self.count_basis_only += other.count_basis_only;
        self.walk_changed += other.walk_changed;
        self.floor_gap_never_positive += other.floor_gap_never_positive;
    }
    fn render(&self) -> String {
        format!(
            "rows_compared={} rows_with_a_count_changed={} rows_with_a_judgement_changed={} jia_count_basis_only={} yi_walk_changed={} bing_floor_gap_never_positive={}",
            self.rows_compared, self.rows_with_a_count_changed, self.rows_with_a_judgement_changed, self.count_basis_only, self.walk_changed, self.floor_gap_never_positive
        )
    }
}

fn comparison_key(row: &WindowRow) -> String {
    format!("{}|{}|{}", line_suffix(row.line_name), row.arm.name(), row.row_key)
}

/// 第六节第 5、6、7 行：第 2 次重跑的每一行与第 1 次同 (行名去掉前缀, 臂, 行键) 的行比假候选、不可恢复、假性 ENOSPC 三个计数；变了的逐行打、逐行归因。
/// 没有第 1 次对应行的（RGall 一族、`disk_stuck`）不进比；返回比的合计与没有对应行的行数。
fn emit_row_comparison(emitter: &mut Emitter, first_rows: &[WindowRow], rows: &[WindowRow]) -> (CompareTally, u64) {
    let first_by_key: HashMap<String, &WindowRow> = first_rows.iter().map(|row| (comparison_key(row), row)).collect();
    let mut tallies: BTreeMap<(String, &'static str), CompareTally> = BTreeMap::new();
    let mut rows_without_counterpart = 0u64;
    for row in rows {
        let Some(first_row) = first_by_key.get(&comparison_key(row)) else {
            rows_without_counterpart += 1;
            continue;
        };
        let old = &first_row.metrics;
        let new = &row.metrics;
        let is_count_changed = old.fake_candidates != new.fake_candidates
            || old.unrecoverable != new.unrecoverable
            || (row.has_false_enospc_field && old.false_enospc_arm != new.false_enospc_arm);
        let is_judgement_changed = first_row.judgement != row.judgement;
        let tally = tallies.entry((row.world_class.clone(), row.arm.name())).or_insert_with(CompareTally::fresh);
        tally.rows_compared += 1;
        tally.rows_with_a_judgement_changed += u64::from(is_judgement_changed);
        if is_count_changed {
            tally.rows_with_a_count_changed += 1;
            let class = classify_changed_row(first_row, row);
            tally.add_class(class);
            let enospc_pair = if row.has_false_enospc_field { format!("{}/{}", old.false_enospc_arm, new.false_enospc_arm) } else { "na".to_string() };
            println!("{}", emitter.emit_raw(&format!(
                "name=f_changed_row line={} arm={} {} fakes_old_new={}/{} unrecoverable_old_new={}/{} false_enospc_arm_old_new={enospc_pair} \
                 fakes_with_the_old_floor={} rollback_target_changed={} floor_gap_positive_samples={} judgement_old={} judgement_new={} class={}",
                row.line_name, row.arm.name(), row.row_key, old.fake_candidates, new.fake_candidates, old.unrecoverable, new.unrecoverable,
                new.second_rerun.fakes_with_the_old_floor, new.second_rerun.rollback_target_changed, new.second_rerun.candidate_floor_gap.positive_samples,
                judgement_text(&first_row.judgement), judgement_text(&row.judgement), class.name()
            )));
        }
    }
    let mut totals = CompareTally::fresh();
    for ((class, arm_name), tally) in &tallies {
        println!("{}", emitter.emit_raw(&format!("name=verdict_f_compare class={class} arm={arm_name} {}", tally.render())));
        totals.add(tally);
    }
    println!("{}", emitter.emit_raw(&format!("name=verdict_f_compare_total {} rows_without_a_first_rerun_counterpart={rows_without_counterpart}", totals.render())));
    (totals, rows_without_counterpart)
}

/// 第六节第 4 行：下界差 g 为正的采样，每类世界与合计；返回 g 为正的采样总数。
fn emit_reach_verdicts(emitter: &mut Emitter, rows: &[WindowRow]) -> u64 {
    let mut classes: Vec<&str> = Vec::new();
    for row in rows {
        if !classes.contains(&row.world_class.as_str()) {
            classes.push(row.world_class.as_str());
        }
    }
    let mut positive_samples_total = 0u64;
    for class in classes {
        let class_rows: Vec<&WindowRow> = rows.iter().filter(|row| row.world_class == class).collect();
        let positive_samples: u64 = class_rows.iter().map(|row| row.metrics.second_rerun.candidate_floor_gap.positive_samples).sum();
        let peak = class_rows.iter().map(|row| row.metrics.second_rerun.candidate_floor_gap.maximum).max().unwrap_or(i64::MIN);
        positive_samples_total += positive_samples;
        println!("{}", emitter.emit_raw(&format!(
            "name=verdict_f_reach class={class} rows={} rows_with_positive_gap={} positive_samples_total={positive_samples} peak_gap={}",
            class_rows.len(), class_rows.iter().filter(|row| row.metrics.second_rerun.candidate_floor_gap.positive_samples > 0).count(), observed_or_na(peak)
        )));
    }
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_f_reach_summary positive_samples_total={positive_samples_total} cf_never_reached={}", u8::from(positive_samples_total == 0)
    )));
    positive_samples_total
}

/// 第六节第 8 行与 V6、V7：每条臂与合计的「旧候选集严格大于条款形」「条款形严格大于旧」次数；系统配置 F 低于根上 F 的次数。返回合计。
fn emit_direction_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) -> (u64, u64, u64) {
    let mut totals = (0u64, 0u64, 0u64);
    for arm in family.arms.iter().chain(family.short_arms.iter()).copied() {
        let arm_rows: Vec<&WindowRow> = rows.iter().filter(|row| row.arm == arm).collect();
        let old_larger: u64 = arm_rows.iter().map(|row| row.metrics.second_rerun.checks_old_set_strictly_larger).sum();
        let new_larger: u64 = arm_rows.iter().map(|row| row.metrics.second_rerun.checks_new_set_strictly_larger).sum();
        let below: u64 = arm_rows.iter().map(|row| row.metrics.second_rerun.system_configuration_below_root_floor_events).sum();
        let target_changed: u64 = arm_rows.iter().map(|row| row.metrics.second_rerun.rollback_target_changed).sum();
        println!("{}", emitter.emit_raw(&format!(
            "name=verdict_f_direction arm={} rows={} checks_old_set_strictly_larger={old_larger} checks_new_set_strictly_larger={new_larger} \
             rollback_target_changed={target_changed} system_configuration_below_root_floor_events={below}",
            arm.name(), arm_rows.len()
        )));
        totals = (totals.0 + old_larger, totals.1 + new_larger, totals.2 + below);
    }
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_f_direction_total checks_old_set_strictly_larger={} checks_new_set_strictly_larger={} system_configuration_below_root_floor_events={}",
        totals.0, totals.1, totals.2
    )));
    totals
}

/// 第六节第 11–13 行：条款形下不为零的行（假候选、不可恢复、有这一字段的行里的假性 ENOSPC），被测臂与对照臂逐行点名、不截断。
fn emit_nonzero_rows(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) {
    for row in rows.iter().filter(|row| family.arms.contains(&row.arm)) {
        let is_nonzero = row.metrics.fake_candidates > 0 || row.metrics.unrecoverable > 0 || (row.has_false_enospc_field && row.metrics.false_enospc_arm > 0);
        if !is_nonzero {
            continue;
        }
        let false_enospc_text = if row.has_false_enospc_field { row.metrics.false_enospc_arm.to_string() } else { "na".to_string() };
        println!("{}", emitter.emit_raw(&format!(
            "name=f_nonzero_row line={} arm={} {} fakes={} unrecoverable={} false_enospc_arm={false_enospc_text} judged_false_enospc={} report_only_world={}",
            row.line_name, row.arm.name(), row.row_key, row.metrics.fake_candidates, row.metrics.unrecoverable,
            u8::from(row.judgement.has_false_enospc_in_a_judged_row), u8::from(row.is_report_only_world)
        )));
    }
}

/// 第六节第 14 行：RGall（读法 P）与 RGall_Q 逐行的三位判定比，另报某块幸存盘整环里一条有效根都没有的次数。返回判定不同的行数。
fn emit_rgall_reading_verdicts(emitter: &mut Emitter, rows: &[WindowRow]) -> u64 {
    let reading_p = Arm::WholeRing(WholeRingArm::ReadingP);
    let reading_q = Arm::WholeRing(WholeRingArm::ReadingQ);
    let reading_p_judgement_by_key: HashMap<(&str, &str), RowJudgement> = rows.iter()
        .filter(|row| row.arm == reading_p)
        .map(|row| ((row.line_name, row.row_key.as_str()), row.judgement)).collect();
    let mut compared = 0u64;
    let mut differing = 0u64;
    for row in rows.iter().filter(|row| row.arm == reading_q) {
        let Some(reading_p_judgement) = reading_p_judgement_by_key.get(&(row.line_name, row.row_key.as_str())) else {
            continue;
        };
        compared += 1;
        if *reading_p_judgement != row.judgement {
            differing += 1;
            println!("{}", emitter.emit_raw(&format!(
                "name=f_rgall_readings_diff_row line={} {} reading_p={} reading_q={}", row.line_name, row.row_key, judgement_text(reading_p_judgement), judgement_text(&row.judgement)
            )));
        }
    }
    let events_of = |arm: Arm| -> u64 { rows.iter().filter(|row| row.arm == arm).map(|row| row.metrics.second_rerun.disk_without_valid_root_events).sum() };
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_f_rgall_readings rows_compared={compared} rows_with_a_judgement_different_between_p_and_q={differing} \
         disk_without_valid_root_events_p={} disk_without_valid_root_events_q={}",
        events_of(reading_p), events_of(reading_q)
    )));
    differing
}

/// 第六节第 15、16 行：代价三样按（类, 臂）合并之后的轨迹（峰值、为正的采样数、期末值，全程与吃光供给之后各报）。
fn emit_cost_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) {
    let mut merged: BTreeMap<(String, &'static str), SecondRerunObservations> = BTreeMap::new();
    for row in rows.iter().filter(|row| family.arms.contains(&row.arm)) {
        merged.entry((row.world_class.clone(), row.arm.name())).or_insert_with(SecondRerunObservations::fresh).add(&row.metrics.second_rerun);
    }
    for ((class, arm_name), observations) in &merged {
        println!("{}", emitter.emit_raw(&format!("name=verdict_f_cost class={class} arm={arm_name} {}", observations.render_costs())));
    }
}

/// 第六节第 17 行：每个 (行名, 臂) 上有上界、没有上界、不判的行数；返回 (`disk_stuck` 里 G2SW 没有上界的行数, `broken_slot` 与 H1 里 RGall 没有上界的行数)。
fn emit_growth_verdicts(emitter: &mut Emitter, growth: &[StuckGrowthSummary]) -> (u64, u64) {
    let mut line_and_arm_pairs: Vec<(&'static str, Arm)> = Vec::new();
    for summary in growth {
        if !line_and_arm_pairs.contains(&(summary.line_name, summary.arm)) {
            line_and_arm_pairs.push((summary.line_name, summary.arm));
        }
    }
    for (line_name, arm) in line_and_arm_pairs {
        let selected: Vec<&StuckGrowthSummary> = growth.iter().filter(|summary| summary.line_name == line_name && summary.arm == arm).collect();
        let count_of = |bound: StuckBound| selected.iter().filter(|summary| summary.bound == bound).count();
        println!("{}", emitter.emit_raw(&format!(
            "name=verdict_f_growth line={line_name} arm={} rows={} bounded={} unbounded={} not_judged={}",
            arm.name(), selected.len(), count_of(StuckBound::Bounded), count_of(StuckBound::Unbounded), count_of(StuckBound::NotJudged)
        )));
    }
    let window_arm = Arm::ClauseFloorWindowed(WindowArm::Window);
    let is_whole_ring = |arm: Arm| matches!(arm, Arm::WholeRing(_));
    let window_unbounded_in_disk_stuck = growth.iter()
        .filter(|summary| summary.line_name == SECOND_RERUN_LINES.stuck_growth && summary.arm == window_arm && summary.bound == StuckBound::Unbounded).count() as u64;
    let whole_ring_unbounded_in_broken_slot = growth.iter()
        .filter(|summary| (summary.line_name == SECOND_RERUN_LINES.broken_slot_growth || summary.line_name == SECOND_RERUN_LINES.broken_slot_reversed_growth)
            && is_whole_ring(summary.arm) && summary.bound == StuckBound::Unbounded).count() as u64;
    (window_unbounded_in_disk_stuck, whole_ring_unbounded_in_broken_slot)
}

/// 第八节几何敏感性（刻度与判）：同一臂、同一组参数上两个取样点的「有上界 / 没有上界」逐行比；返回判不同的行数。
fn emit_growth_pairing(
    emitter: &mut Emitter,
    kind: &str,
    growth: &[StuckGrowthSummary],
    is_base: impl Fn(&StuckGrowthSummary) -> bool,
    is_other: impl Fn(&StuckGrowthSummary) -> bool,
    pairing_key: impl Fn(&StuckGrowthSummary) -> String,
) -> u64 {
    let mut arms: Vec<Arm> = Vec::new();
    for summary in growth.iter().filter(|summary| is_base(summary)) {
        if !arms.contains(&summary.arm) {
            arms.push(summary.arm);
        }
    }
    let mut total_differing = 0u64;
    for arm in arms {
        let base_bound_by_key: HashMap<String, StuckBound> = growth.iter().filter(|summary| summary.arm == arm && is_base(summary)).map(|summary| (pairing_key(summary), summary.bound)).collect();
        let mut compared = 0u64;
        let mut differing = 0u64;
        for summary in growth.iter().filter(|summary| summary.arm == arm && is_other(summary)) {
            let Some(base_bound) = base_bound_by_key.get(&pairing_key(summary)) else {
                continue;
            };
            compared += 1;
            differing += u64::from(*base_bound != summary.bound);
        }
        total_differing += differing;
        println!("{}", emitter.emit_raw(&format!(
            "name=verdict_f_growth_geometry point={kind} arm={} rows_compared={compared} rows_with_a_different_bound={differing}", arm.name()
        )));
    }
    total_differing
}

/// 占用两档（0.30 与 0.70）上 `disk_stuck` 行的三位判定逐行比；返回判定不同的行数。
fn emit_stuck_fill_verdicts(emitter: &mut Emitter, rows: &[WindowRow]) -> u64 {
    let is_stuck_row = |row: &&WindowRow| row.world_class == DISK_STUCK_CLASS || row.world_class == DISK_STUCK_REVERSED_CLASS;
    let mut total_differing = 0u64;
    let mut arms: Vec<Arm> = Vec::new();
    for row in rows.iter().filter(is_stuck_row) {
        if !arms.contains(&row.arm) {
            arms.push(row.arm);
        }
    }
    for arm in arms {
        let low_fill_judgement_by_key: HashMap<(&str, String), RowJudgement> = rows.iter()
            .filter(is_stuck_row)
            .filter(|row| row.arm == arm && row.row_key.contains("fill=0.30"))
            .map(|row| ((row.line_name, row_key_without_the_field(&row.row_key, "fill=")), row.judgement)).collect();
        let mut compared = 0u64;
        let mut differing = 0u64;
        for row in rows.iter().filter(is_stuck_row).filter(|row| row.arm == arm && row.row_key.contains("fill=0.70")) {
            let Some(low_fill_judgement) = low_fill_judgement_by_key.get(&(row.line_name, row_key_without_the_field(&row.row_key, "fill="))) else {
                continue;
            };
            compared += 1;
            differing += u64::from(*low_fill_judgement != row.judgement);
        }
        total_differing += differing;
        println!("{}", emitter.emit_raw(&format!(
            "name=verdict_f_fill_geometry arm={} rows_compared={compared} rows_with_a_judgement_different_between_fills={differing}", arm.name()
        )));
    }
    total_differing
}

/// 登记第五节「别的世界不量斜率」：洞里的旧根至多活两圈。非 `broken_slot`、非 `disk_stuck` 的行里采样时幸存盘上仍有 txg + 2 × 3S ≤ N 的有效根的行逐行点名；返回这样的行数。
fn emit_stale_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) -> u64 {
    let is_stuck_world = |class: &str| class == BROKEN_SLOT_CLASS || class == DISK_STUCK_CLASS || class == DISK_STUCK_REVERSED_CLASS;
    let mut rows_outside_the_stuck_worlds = 0u64;
    let mut rows_in_the_stuck_worlds = 0u64;
    for row in rows.iter().filter(|row| family.arms.contains(&row.arm) || family.short_arms.contains(&row.arm)) {
        let samples = row.metrics.second_rerun.stale_beyond_two_revolutions_samples;
        if samples == 0 {
            continue;
        }
        if is_stuck_world(&row.world_class) {
            rows_in_the_stuck_worlds += 1;
        } else {
            rows_outside_the_stuck_worlds += 1;
            println!("{}", emitter.emit_raw(&format!("name=f_stale_row line={} arm={} {} stale_beyond_two_revolutions={samples}", row.line_name, row.arm.name(), row.row_key)));
        }
    }
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_f_stale rows_outside_the_stuck_worlds_with_a_root_beyond_two_revolutions={rows_outside_the_stuck_worlds} \
         rows_in_the_stuck_worlds_with_a_root_beyond_two_revolutions={rows_in_the_stuck_worlds}"
    )));
    rows_outside_the_stuck_worlds
}

/// V8：条款形下的 G2SW、G2SW_Q、G2SW_L 的 x 按定义恒 0（回收门槛就是窗口门槛）；有一次不为 0 的行数。
fn count_window_arm_rows_with_a_nonzero_extra_pinned(rows: &[WindowRow]) -> u64 {
    let window_arms = [Arm::ClauseFloorWindowed(WindowArm::Window), Arm::ClauseFloorWindowed(WindowArm::DiskHold), Arm::ClauseFloorWindowed(WindowArm::RingLag)];
    rows.iter().filter(|row| window_arms.contains(&row.arm)).filter(|row| {
        let quantity = &row.metrics.second_rerun.extra_pinned_against_the_window_gate;
        quantity.samples > 0 && (quantity.maximum != 0 || quantity.minimum != 0)
    }).count() as u64
}

/// 第 2 次重跑的全部行与判决行（登记第五至八节）：窗口一族与 RGall 一族在条款形候选下界下跑第 1 次的每一格，再加 `disk_stuck`；
/// 接在第 1 次重跑的全部行之后、`name=done` 之前。`first_rows` 与 `first_crit1_bodies` 是第 1 次那一族的行簿，逐行比用。
fn run_second_rerun_arms(emitter: &mut Emitter, first_rows: &[WindowRow], first_crit1_bodies: &[String]) {
    let family = second_rerun_family();
    let family = &family;
    let arm_names = |arms: &[Arm]| arms.iter().map(|arm| arm.name()).collect::<Vec<&str>>().join("/");
    println!("{}", emitter.emit_raw(&format!(
        "name=f_config f_arms={} f_short_arms={} short_slots={} disk_stuck_arms=RGall_ring_all/RGall_Q_ring_all/G2SW_tightened_states_window/G2SW_ringgate stuck_disks=1/0 \
         stuck_fills=0.30/0.70 mark_revolutions=1/2/4 stuck_window_revolution_limit={STUCK_WINDOW_REVOLUTION_LIMIT} stuck_tail_revolutions={STUCK_TAIL_REVOLUTIONS} \
         stuck_endings=crash/lost_disk_0/lost_disk_1 bounded_slope_rule=twice_the_pinned_increase_below_the_blocks_freed geometry_points=h1/h2/h3 seeds={SEED_COUNT} fault_seeds={FAULT_SEED_COUNT}",
        arm_names(&family.arms), arm_names(&family.short_arms), format_set(&WINDOW_SHORT_SLOTS_SWEEP)
    )));
    let mut rows: Vec<WindowRow> = Vec::new();
    let mut growth: Vec<StuckGrowthSummary> = Vec::new();
    let (crit1_rows_not_holding, crit1_line_bodies) = emit_window_crit1_rows(emitter, family);
    emit_window_world_rows(emitter, &mut rows, family);
    emit_window_activation_rows(emitter, &mut rows, family, GeometryPoint::Original);
    emit_window_near_full_rows(emitter, &mut rows, family);
    emit_window_drain_failure_rows(emitter, &mut rows, family);
    emit_window_broken_slot_rows(emitter, &mut rows, &mut growth, family, GeometryPoint::Original);
    emit_window_cost_growth_rows(emitter, &mut rows, family);
    emit_window_activation_rows(emitter, &mut rows, family, GeometryPoint::Reversed);
    emit_window_broken_slot_rows(emitter, &mut rows, &mut growth, family, GeometryPoint::Reversed);
    emit_disk_stuck_rows(emitter, &mut rows, &mut growth, family);
    emit_window_row_verdicts(emitter, &rows, family, crit1_rows_not_holding);
    emit_window_gate_reach_verdicts(emitter, &rows, family);
    emit_window_self_proof_verdicts(emitter, &rows, family);
    emit_window_reading_verdicts(emitter, &rows, family);
    emit_window_geometry_verdicts(emitter, &rows, family);
    emit_second_rerun_verdict_lines(emitter, family, first_rows, first_crit1_bodies, &rows, &growth, &crit1_line_bodies);
}

/// 第 2 次重跑的判决行（登记第六节第 4–17 行与 V5–V8、F1–F11 的计数）：只读行簿，不再跑模型；单测拿一小撮行簿验它不 panic。
fn emit_second_rerun_verdict_lines(
    emitter: &mut Emitter,
    family: &RowFamily,
    first_rows: &[WindowRow],
    first_crit1_bodies: &[String],
    rows: &[WindowRow],
    growth: &[StuckGrowthSummary],
    crit1_line_bodies: &[String],
) {
    let positive_gap_samples = emit_reach_verdicts(emitter, &rows);
    let (comparison, _) = emit_row_comparison(emitter, first_rows, &rows);
    let first_arm_tokens: HashSet<&str> = first_crit1_bodies.iter().filter_map(|body| body.split(' ').next()).collect();
    let first_bodies: HashSet<&str> = first_crit1_bodies.iter().map(String::as_str).collect();
    let crit1_compared = crit1_line_bodies.iter().filter(|body| body.split(' ').next().is_some_and(|token| first_arm_tokens.contains(token))).count();
    let crit1_differing = crit1_line_bodies.iter()
        .filter(|body| body.split(' ').next().is_some_and(|token| first_arm_tokens.contains(token)) && !first_bodies.contains(body.as_str())).count();
    let rgall_crit1_not_holding = crit1_line_bodies.iter().filter(|body| body.starts_with("arm=RGall") && body.ends_with("holds=0")).count();
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_f_crit1_compare rows_compared={crit1_compared} rows_with_a_differing_line={crit1_differing} rgall_crit1_rows_not_holding={rgall_crit1_not_holding}"
    )));
    let (old_set_larger, new_set_larger, below_root_floor) = emit_direction_verdicts(emitter, &rows, family);
    emit_nonzero_rows(emitter, &rows, family);
    let readings_differing = emit_rgall_reading_verdicts(emitter, &rows);
    emit_cost_verdicts(emitter, &rows, family);
    let (window_unbounded_in_disk_stuck, whole_ring_unbounded_in_broken_slot) = emit_growth_verdicts(emitter, &growth);
    let h3_differing = emit_growth_pairing(emitter, "h3", &growth,
        |summary| summary.line_name == family.lines.stuck_growth && summary.row_key.contains("stuck_disk=1"),
        |summary| summary.line_name == family.lines.stuck_growth && summary.row_key.contains("stuck_disk=0"),
        |summary| row_key_without_the_field(&summary.row_key, "stuck_disk="));
    let fill_differing = emit_growth_pairing(emitter, "fill", &growth,
        |summary| summary.line_name == family.lines.stuck_growth && summary.row_key.contains("fill=0.30"),
        |summary| summary.line_name == family.lines.stuck_growth && summary.row_key.contains("fill=0.70"),
        |summary| row_key_without_the_field(&summary.row_key, "fill="));
    let h1_differing = emit_growth_pairing(emitter, "h1", &growth,
        |summary| summary.line_name == family.lines.broken_slot_growth,
        |summary| summary.line_name == family.lines.broken_slot_reversed_growth,
        |summary| summary.row_key.clone());
    let stuck_fill_judgement_differing = emit_stuck_fill_verdicts(emitter, &rows);
    let stale_rows_outside_the_stuck_worlds = emit_stale_verdicts(emitter, &rows, family);
    let window_arm_rows_with_a_nonzero_extra_pinned = count_window_arm_rows_with_a_nonzero_extra_pinned(&rows);
    let is_rgall_short = |arm: Arm| matches!(arm, Arm::WholeRing(WholeRingArm::ReadingPShort | WholeRingArm::ReadingQShort));
    let rgall_short_rows_with_zero_fakes = rows.iter().filter(|row| is_rgall_short(row.arm) && !row.judgement.has_fake_candidates).count();
    let rgall_rows: Vec<&WindowRow> = rows.iter().filter(|row| WHOLE_RING_ARMS.iter().any(|whole_ring_arm| row.arm == Arm::WholeRing(*whole_ring_arm))).collect();
    let rgall_false_enospc_outside_the_judged_range = rgall_rows.iter().filter(|row| {
        row.metrics.false_enospc_arm > 0
            && (row.world_class == DISK_STUCK_CLASS || row.world_class == DISK_STUCK_REVERSED_CLASS || (row.world_class == DRAIN_FAILURE_CLASS && row.row_key.contains("failures=3")))
    }).count();
    let rgall_rows_not_zero = rgall_rows.iter().filter(|row| {
        (!row.is_report_only_world && (row.judgement.has_fake_candidates || row.judgement.has_unrecoverable)) || row.judgement.has_false_enospc_in_a_judged_row
    }).count();
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_f_clauses f1_cf_never_reached={} f3_rgall_short_rows_where_the_detector_saw_zero_fake_candidates={rgall_short_rows_with_zero_fakes} \
         f4_window_arm_unbounded_disk_stuck_rows={window_unbounded_in_disk_stuck} f5_rgall_rows_with_different_readings={readings_differing} \
         f7_growth_rows_with_a_different_bound_h3={h3_differing} f7_growth_rows_with_a_different_bound_fill={fill_differing} f7_growth_rows_with_a_different_bound_h1={h1_differing} \
         f7_stuck_rows_with_a_different_judgement_between_fills={stuck_fill_judgement_differing} \
         f8_rgall_false_enospc_rows_outside_the_judged_range={rgall_false_enospc_outside_the_judged_range} f9_rgall_rows_with_fakes_unrecoverable_or_judged_false_enospc={rgall_rows_not_zero} \
         f10_rows_outside_the_stuck_worlds_with_a_root_beyond_two_revolutions={stale_rows_outside_the_stuck_worlds} f11_rgall_broken_slot_unbounded_rows={whole_ring_unbounded_in_broken_slot} \
         v5_changed_rows_with_a_never_positive_floor_gap={} v6_checks_new_set_strictly_larger={new_set_larger} v7_system_configuration_below_root_floor_events={below_root_floor} \
         v8_window_arm_rows_with_a_nonzero_extra_pinned={window_arm_rows_with_a_nonzero_extra_pinned} checks_old_set_strictly_larger={old_set_larger} \
         changed_rows_total={} positive_gap_samples_total={positive_gap_samples}",
        u8::from(positive_gap_samples == 0), comparison.floor_gap_never_positive, comparison.rows_with_a_count_changed
    )));
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
    let (first_rerun_rows, first_rerun_crit1_bodies) = run_window_arms(&mut emitter);
    run_second_rerun_arms(&mut emitter, &first_rerun_rows, &first_rerun_crit1_bodies);
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

/// 第 1 次重跑的单测（`research/prompts/e139-r1-prereg.md` 第七节的锚点 K1–K9 与第九节要补的几条）。
/// 独立核算脚本 `anchors.py`（登记第十三节末尾）按槽公式摆根、写失败的槽留旧根，这里的期望值照它的输出抄，不读装置。
#[cfg(test)]
mod window_tests {
    use super::*;

    fn window_simulation(window_arm: WindowArm, slots_per_region: u64, keep_snapshots: bool) -> Simulation {
        let mut simulation = Simulation::new(Arm::TightenedStatesWindowed(window_arm), slots_per_region, 1, 5, 1, keep_snapshots, None);
        simulation.prefill(STEADY_FILL);
        simulation
    }

    /// 只建不删的窗口一个接一个发布；`failed_txg` 那个 txg 的根槽写失败（推进一格再发，那个 txg 从没写成）。
    fn publish_creating_windows_up_to(simulation: &mut Simulation, newest_txg: u64, failed_txg: Option<u64>) {
        while simulation.next_txg <= newest_txg {
            if Some(simulation.next_txg) == failed_txg {
                simulation.fail_next_slot_write = true;
            }
            assert_eq!(simulation.run_user_window(false, false), WindowOutcome::Created, "txg {} 的窗口要发布成功", simulation.next_txg);
        }
    }

    fn sorted_ring_txgs(simulation: &Simulation) -> Vec<u64> {
        let mut txgs: Vec<u64> = simulation.ring.iter().flatten().map(|root| root.txg).collect();
        txgs.sort_unstable();
        txgs
    }

    /// 锚 2 / 4 / 7 / 9 共用的状态：S = 4，txg 1..22 依次发布，21 的根槽写失败（22 重发）。
    fn anchor_two_state(window_arm: WindowArm, keep_snapshots: bool) -> Simulation {
        let mut simulation = window_simulation(window_arm, 4, keep_snapshots);
        publish_creating_windows_up_to(&mut simulation, 22, Some(21));
        simulation
    }

    /// 锚 3 / 5 共用的状态：S = 1，txg 1..9 依次发布，7（区域 1、盘 1）写失败（8 重发）。
    fn anchor_three_state(window_arm: WindowArm, keep_snapshots: bool) -> Simulation {
        let mut simulation = window_simulation(window_arm, 1, keep_snapshots);
        publish_creating_windows_up_to(&mut simulation, 9, Some(7));
        simulation
    }

    #[test]
    fn k1_window_is_txg_greater_than_anchor_minus_ring_length() {
        let lowest_inside = (0..=35u64).find(|txg| txg_is_inside_window(*txg, 35, REGION_COUNT * 8)).expect("窗口里有 txg");
        let highest_outside = (0..=35u64).filter(|txg| !txg_is_inside_window(*txg, 35, REGION_COUNT * 8)).max().expect("窗口外有 txg");
        assert_eq!((lowest_inside, highest_outside), (12, 11), "anchor1 window lowest in = 12 highest out = 11");
        let mut simulation = window_simulation(WindowArm::Window, 8, false);
        simulation.newest_durable_txg = 35;
        assert_eq!(simulation.ring_depth, 24, "S = 8 时环长 3S = 24");
        assert!(simulation.is_inside_candidate_window(12) && !simulation.is_inside_candidate_window(11), "候选集的窗口判定走同一个谓词");
    }

    #[test]
    fn k2_window_arms_keep_the_tightened_states_bound_reserve_and_residue() {
        for window_arm in WINDOW_ARMS.iter().chain(WINDOW_SHORT_ARMS.iter()) {
            let arm = Arm::TightenedStatesWindowed(*window_arm);
            assert_eq!(arm.publication_bound(12), 4 + 4, "{} B = 4 + 2 × k_tol = 8", window_arm.name());
            assert_eq!((arm.reserve_blocks(12, 5), arm.push_residue_blocks(12, 5)), (35 + 10, 30 + 5), "{} c_max = 5：保留池 45、残留 35", window_arm.name());
            assert_eq!((arm.reserve_blocks(12, 10), arm.push_residue_blocks(12, 10)), (70 + 10, 60 + 5), "{} c_max = 10：保留池 80、残留 65", window_arm.name());
        }
    }

    #[test]
    fn k3_ring_has_nine_outside_the_window_and_the_two_gates_differ_by_two() {
        for (window_arm, expected_gate) in [(WindowArm::Window, 11u64), (WindowArm::DiskHold, 11), (WindowArm::RingLag, 11), (WindowArm::RingGate, 9)] {
            let mut simulation = anchor_two_state(window_arm, false);
            assert_eq!(sorted_ring_txgs(&simulation), vec![9, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 22], "{} anchor2 ring", window_arm.name());
            assert_eq!(simulation.newest_durable_txg, 22);
            assert_eq!(simulation.oldest_persisted_valid_txg(), 9, "anchor2 ring_oldest = 9");
            assert_eq!(simulation.oldest_valid_txg_in_the_reclaim_window(simulation.ring_depth), Some(11), "anchor2 window_oldest = 11");
            assert_eq!(simulation.reuse_bound(), expected_gate, "{} 回收门槛（F_生效 = 0）", window_arm.name());
            // 释放代 10、11 的已释放块：窗口门槛放得回，整环门槛放不回
            let reusable_before = simulation.reusable_count();
            simulation.pool.blocks[0] = BlockState::Freed { allocated_at: 1, freed_at: 10 };
            simulation.pool.blocks[1] = BlockState::Freed { allocated_at: 1, freed_at: 11 };
            let reusable_gain = simulation.reusable_count() - reusable_before;
            assert_eq!(reusable_gain, if window_arm == WindowArm::RingGate { 0 } else { 2 }, "{} 释放代 10、11 的块可再分配的个数", window_arm.name());
        }
    }

    #[test]
    fn k4_root_nine_outside_the_window_is_not_a_fake_candidate_but_would_be_without_the_window() {
        for window_arm in WINDOW_ARMS {
            let mut simulation = anchor_two_state(window_arm, false);
            simulation.damaged_txgs.clear();
            simulation.damaged_txgs.insert(9);
            let (fake, candidates, _, _) = simulation.count_fake_candidates();
            assert_eq!((fake, candidates), (0, 11), "{} 根 9 在窗口外：不进候选集、不记假候选，候选 11 个", window_arm.name());
            let without_window = simulation.valid_roots().iter().filter(|root| simulation.damaged_txgs.contains(&root.txg)).count();
            assert_eq!(without_window, 1, "没有窗口这一项时根 9 记成 1 个假候选");
        }
    }

    #[test]
    fn k5_ceiling_readings_p_and_q_and_the_two_lag_bound_readings_differ_on_the_skipped_disk() {
        let simulation = anchor_three_state(WindowArm::Window, false);
        let ring_by_slot: Vec<Option<u64>> = simulation.ring.iter().map(|root| root.map(|record| record.txg)).collect();
        assert_eq!(ring_by_slot, vec![Some(9), Some(4), Some(8)], "anchor3 ring = {{0: 9, 1: 4, 2: 8}}");
        assert_eq!(simulation.windowed_states_upper_limit(WindowArm::Window), 8, "读法 P：盘 1 窗口里没有根，不进取小，上限 8");
        assert_eq!(simulation.windowed_states_upper_limit(WindowArm::DiskHold), 0, "读法 Q：盘 1 记 0，上限 0");
        assert_eq!(simulation.per_disk_states_upper_limit(), 4, "原 G2S 整环取法：上限 4");
        assert_eq!(simulation.windowed_lag_bound_txg(WindowArm::Window), 8, "读法 a：滞后量下界 8");
        assert_eq!(simulation.windowed_lag_bound_txg(WindowArm::RingLag), 4, "读法 b：滞后量下界 4");
    }

    #[test]
    fn k6_window_arms_in_steady_use_three_or_four_publications_and_lag_three_windows() {
        for window_arm in WINDOW_ARMS {
            for (slots_per_region, cost, publications, maximum, pinned, lag) in [
                (1u64, 1u64, vec![3u64], 3u64, 6u64, 0u64),
                (1, 5, vec![3], 3, 10, 0),
                (4, 1, vec![3, 4], 4, 7, 39),
                (4, 5, vec![3, 4], 4, 15, 39),
                (16, 1, vec![3, 4], 4, 7, 39),
                (16, 5, vec![3, 4], 4, 15, 39),
            ] {
                let (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released) =
                    push_probe_over_landings(Arm::TightenedStatesWindowed(window_arm), slots_per_region, cost);
                assert!(is_all_released, "{} S={slots_per_region} c={cost} 处置之后目标必须放回", window_arm.name());
                assert_eq!(publication_set, publications, "{} S={slots_per_region} c={cost} 三种落点的发布数", window_arm.name());
                assert_eq!(maximum_publications, maximum, "{} S={slots_per_region} c={cost} 最多 4 次", window_arm.name());
                assert_eq!(maximum_pinned, pinned, "{} S={slots_per_region} c={cost} 滞后量之外扣住最多 5 + 2c", window_arm.name());
                assert_eq!(maximum_lag, lag, "{} S={slots_per_region} c={cost} 滞后量 lag 3 windows x 13 = 39", window_arm.name());
            }
        }
    }

    #[test]
    fn k7_gate_difference_counter_reads_two_on_the_anchor_two_state_and_zero_in_steady() {
        for (window_arm, expected_difference) in [(WindowArm::Window, 2i64), (WindowArm::RingGate, 0)] {
            let mut simulation = anchor_two_state(window_arm, false);
            simulation.metrics = Metrics::fresh();
            simulation.allocate(1, 23).expect("Unused 块还很多");
            let gate_difference = simulation.metrics.trajectory.gate_difference;
            assert_eq!((gate_difference.samples, gate_difference.maximum), (1, expected_difference), "{} 门槛差 d = 11 − 9", window_arm.name());
            assert_eq!(gate_difference.positive_samples, u64::from(expected_difference > 0), "{} d > 0 记一次", window_arm.name());
        }
        let mut steady = window_simulation(WindowArm::Window, 4, false);
        for _ in 0..60 {
            steady.run_user_window(true, false);
        }
        let steady_difference = steady.metrics.trajectory.gate_difference;
        assert!(steady_difference.samples > 0, "steady 里 allocate 要真的被读到，否则下一条什么都没测");
        assert_eq!((steady_difference.positive_samples, steady_difference.maximum), (0, 0), "没有写失败的 steady 里 d 恒为 0");
    }

    #[test]
    fn k8_window_boundary_is_strict_the_hole_left_by_a_failed_write_keeps_generation_zero_outside() {
        let mut simulation = window_simulation(WindowArm::Window, 4, false);
        publish_creating_windows_up_to(&mut simulation, 12, Some(5));
        assert_eq!(sorted_ring_txgs(&simulation), vec![0, 1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12], "anchor8 ring");
        assert_eq!(simulation.newest_durable_txg, 12);
        assert_eq!(simulation.oldest_valid_txg_in_the_reclaim_window(simulation.ring_depth), Some(1), "anchor8 window_oldest '>' = 1");
        let with_greater_or_equal = sorted_ring_txgs(&simulation).into_iter().filter(|txg| txg + simulation.ring_depth >= 12).min();
        assert_eq!(with_greater_or_equal, Some(0), "anchor8 window_oldest '>=' = 0");
    }

    #[test]
    fn k9_shortened_window_gate_is_fourteen_on_the_anchor_two_state() {
        let simulation = anchor_two_state(WindowArm::WindowShort, false);
        assert_eq!(simulation.oldest_valid_txg_in_the_reclaim_window(simulation.ring_depth - REGION_COUNT), Some(14), "anchor9 short-window oldest = 14");
        assert_eq!(simulation.reuse_bound(), 14, "缩短一行的窗口：回收门槛 14");
        assert_eq!(anchor_two_state(WindowArm::Window, false).reuse_bound(), 11, "对照：整窗口门槛 11");
    }

    /// 变异 MW2 / MW8 的敏感取样点：管理员回退之后、下一次发布之前，锚是回退目标 11，不是回退前的最大 txg 22。
    #[test]
    fn after_admin_rollback_the_window_is_anchored_at_the_rollback_target() {
        let mut simulation = anchor_two_state(WindowArm::Window, true);
        assert!(simulation.admin_rollback(), "窗口里最旧的候选根 11 是回退目标");
        assert_eq!(simulation.newest_durable_txg, 11, "回退之后锚停在回退目标（装置的取法，登记第十一节差异 ⑦）");
        assert_eq!(simulation.maximum_published_txg, 22, "回退前的最大 txg 仍是 22（变异会拿它当锚）");
        assert_eq!(simulation.reuse_bound(), 9, "窗口锚在 11：根 9 在窗口里，门槛 9（锚取 22 时是 11）");
        let (_, candidates, _, _) = simulation.count_fake_candidates();
        assert_eq!(candidates, 2, "候选集 {{9, 11}}（锚取 22 时只剩 {{11}}）");
    }

    #[test]
    fn rollback_target_is_the_oldest_root_inside_the_window_not_the_oldest_in_the_ring() {
        let mut simulation = anchor_two_state(WindowArm::Window, true);
        simulation.admin_rollback();
        assert_eq!(simulation.newest_durable_txg, 11, "W2：回退目标取窗口里最旧的候选根 11，不取窗口外的 9");
        let mut ring_gate_simulation = anchor_two_state(WindowArm::RingGate, true);
        ring_gate_simulation.admin_rollback();
        assert_eq!(ring_gate_simulation.newest_durable_txg, 11, "G2SW_ringgate 的 W2 照旧（只有 W1 换回旧式）");
    }

    /// 登记第五节 W2 的「等价」一条：这台装置里丢盘只在崩溃时发生，恢复后锚 = 择中的根 = 幸存盘上最新的有效根，它总在窗口里，
    /// 所以「每块幸存盘上最新的有效根在候选集里」那一查加不加窗口同值。
    #[test]
    fn after_disk_loss_the_newest_valid_root_on_every_surviving_disk_is_inside_the_candidate_window() {
        for lost_disk in 0..DISK_COUNT {
            for mut simulation in [anchor_two_state(WindowArm::Window, true), anchor_three_state(WindowArm::Window, true)] {
                simulation.disk_lost[lost_disk] = true;
                simulation.crash_and_recover();
                for disk in simulation.surviving_disks() {
                    let newest = simulation.newest_valid_txg_on_disk(disk).expect("幸存盘上有有效根");
                    assert!(simulation.is_inside_candidate_window(newest), "丢盘 {lost_disk} 之后幸存盘 {disk} 上最新的有效根 {newest} 要在窗口里");
                }
            }
        }
    }

    #[test]
    fn k5_ceiling_difference_counter_reads_the_two_readings_against_the_whole_ring_ceiling() {
        for (window_arm, expected_difference) in [(WindowArm::Window, 4i64), (WindowArm::DiskHold, -4)] {
            let mut simulation = anchor_three_state(window_arm, false);
            simulation.metrics = Metrics::fresh();
            simulation.observe_ceiling_difference();
            let ceiling_difference = simulation.metrics.trajectory.ceiling_difference;
            assert_eq!((ceiling_difference.samples, ceiling_difference.maximum, ceiling_difference.minimum), (1, expected_difference, expected_difference),
                "{} 上限差 = 窗口上限 − 整环上限（读法 P 8 − 4，读法 Q 0 − 4）", window_arm.name());
        }
    }

    #[test]
    fn jump_region_prefix_with_region_one_is_the_original_prefix() {
        for slots_per_region in [4u64, 16] {
            for seed in 0..3 {
                let original = activation_prefix(Arm::TightenedStatesWindowed(WindowArm::Window), slots_per_region, seed, 2).expect("前缀不该卡死");
                let copy = activation_prefix_with_jump_region(Arm::TightenedStatesWindowed(WindowArm::Window), slots_per_region, seed, 2, 1).expect("前缀不该卡死");
                assert_eq!((sorted_ring_txgs(&original), original.newest_durable_txg, original.floor_carried, original.root_write_attempts),
                    (sorted_ring_txgs(&copy), copy.newest_durable_txg, copy.floor_carried, copy.root_write_attempts), "S={slots_per_region} 种子 {seed}");
            }
        }
    }

    #[test]
    fn window_arms_agree_with_the_original_states_arm_in_steady_where_the_ring_has_no_hole() {
        for slots_per_region in [4u64, 16] {
            for seed in 0..3 {
                let original = run_world(Arm::TightenedStates, World::Steady, slots_per_region, seed, 0);
                for window_arm in WINDOW_ARMS {
                    let windowed = run_world(Arm::TightenedStatesWindowed(window_arm), World::Steady, slots_per_region, seed, 0);
                    assert_eq!(world_outcome_fields(&windowed), world_outcome_fields(&original), "{} S={slots_per_region} 种子 {seed}：环里没有洞时与原 G2S 逐字相同", window_arm.name());
                }
            }
        }
    }
}

/// 第 2 次重跑的单测（`research/prompts/e139-r2-prereg.md` 第七节的锚点 K2r、K5r、K6r、K9r、K10–K16，与第九节变异要的取样点）。
/// 独立核算脚本 `anchors_r2.py`（登记第十三节末尾）按槽公式摆根、写失败的槽留旧根，这里的期望值照它的输出抄，不读装置。
#[cfg(test)]
mod second_rerun_tests {
    use super::*;

    const CLAUSE_WINDOW_ARM: Arm = Arm::ClauseFloorWindowed(WindowArm::Window);
    const CLAUSE_RING_GATE_ARM: Arm = Arm::ClauseFloorWindowed(WindowArm::RingGate);
    const WHOLE_RING_READING_P_ARM: Arm = Arm::WholeRing(WholeRingArm::ReadingP);
    const WHOLE_RING_READING_Q_ARM: Arm = Arm::WholeRing(WholeRingArm::ReadingQ);

    fn simulation_with(arm: Arm, slots_per_region: u64, fill: f64, keep_snapshots: bool) -> Simulation {
        let mut simulation = Simulation::new(arm, slots_per_region, 1, 5, 1, keep_snapshots, None);
        simulation.prefill(fill);
        simulation
    }

    /// 只建不删的窗口一个接一个发布；`failed_txg` 那个 txg 的根槽写失败（推进一格再发，那个 txg 从没写成）。
    fn publish_creating_windows_up_to(simulation: &mut Simulation, newest_txg: u64, failed_txg: Option<u64>) {
        while simulation.next_txg <= newest_txg {
            if Some(simulation.next_txg) == failed_txg {
                simulation.fail_next_slot_write = true;
            }
            assert_eq!(simulation.run_user_window(false, false), WindowOutcome::Created, "txg {} 的窗口要发布成功", simulation.next_txg);
        }
    }

    fn run_delete_create_windows(simulation: &mut Simulation, windows: u64) {
        for _ in 0..windows {
            assert_eq!(simulation.run_user_window(true, false), WindowOutcome::Created, "卡住的盘不盘紧时每个窗口都要发布成功");
        }
    }

    fn sorted_ring_txgs(simulation: &Simulation) -> Vec<u64> {
        let mut txgs: Vec<u64> = simulation.ring.iter().flatten().map(|root| root.txg).collect();
        txgs.sort_unstable();
        txgs
    }

    fn sorted_valid_txgs(simulation: &Simulation) -> Vec<u64> {
        let mut txgs: Vec<u64> = simulation.valid_roots().iter().map(|root| root.txg).collect();
        txgs.sort_unstable();
        txgs
    }

    /// 锚 K11 / K9r 共用的状态：S = 4，txg 1..22 依次发布，21 的根槽写失败（22 重发）。
    fn anchor_two_state(arm: Arm, keep_snapshots: bool) -> Simulation {
        let mut simulation = simulation_with(arm, 4, STEADY_FILL, keep_snapshots);
        publish_creating_windows_up_to(&mut simulation, 22, Some(21));
        simulation
    }

    /// 锚 K5r 的状态：S = 1，txg 1..9 依次发布，7（区域 1、盘 1）写失败（8 重发）。
    fn anchor_three_state(arm: Arm) -> Simulation {
        let mut simulation = simulation_with(arm, 1, STEADY_FILL, false);
        publish_creating_windows_up_to(&mut simulation, 9, Some(7));
        simulation
    }

    /// 锚 K10：S = 4，占用 0.70，10 个删建窗口（txg 1..10，不盘紧）；直接抬 F 到 6（系统配置记 6）；第 11 个窗口的根撕裂、崩溃恢复；恢复之后 `damaged_txgs` 设成 {2, 3}。
    fn k10_state(arm: Arm) -> Simulation {
        let mut simulation = simulation_with(arm, 4, STEADY_FILL, true);
        run_delete_create_windows(&mut simulation, 10);
        simulation.raise_floor_to(6);
        simulation.tear_next_publication = true;
        simulation.run_user_window(true, false);
        simulation.damaged_txgs = HashSet::from([2u64, 3]);
        simulation
    }

    #[test]
    fn k2r_whole_ring_arms_keep_the_tightened_states_bound_reserve_and_residue() {
        for whole_ring_arm in WHOLE_RING_ARMS.iter().chain(WHOLE_RING_SHORT_ARMS.iter()) {
            let arm = Arm::WholeRing(*whole_ring_arm);
            assert_eq!(arm.publication_bound(12), 4 + 4, "{} B = 4 + 2 × k_tol = 8", arm.name());
            assert_eq!((arm.reserve_blocks(12, 5), arm.push_residue_blocks(12, 5)), (35 + 10, 30 + 5), "{} c_max = 5：保留池 45、残留 35", arm.name());
            assert_eq!((arm.reserve_blocks(12, 10), arm.push_residue_blocks(12, 10)), (70 + 10, 60 + 5), "{} c_max = 10：保留池 80、残留 65", arm.name());
        }
    }

    /// 变异 MF1–MF5、MC3 的取样点：根上带的 F 都是 0、系统配置里是 6（`raise_floor_to` 记下的），撕裂的根在恢复之后不自证。
    #[test]
    fn k10_system_configuration_floor_six_above_root_floors_zero_lifts_the_candidate_floor_and_the_rollback_target() {
        for arm in [CLAUSE_WINDOW_ARM, WHOLE_RING_READING_P_ARM, WHOLE_RING_READING_Q_ARM] {
            let simulation = k10_state(arm);
            assert_eq!(simulation.slot_index(11), 11, "{} K10 torn_slot = 11", arm.name());
            assert_eq!(sorted_valid_txgs(&simulation), (0..=10).collect::<Vec<u64>>(), "{} K10 valid = [0..10]", arm.name());
            assert_eq!((simulation.floor_carried, simulation.floor_active), (0, 0), "{} 差异 ③ 不动：恢复之后 floor_carried、floor_active 取各根所带的 F，仍是 0", arm.name());
            assert_eq!(simulation.system_configuration_floor, 6, "{} 系统配置里记的是 raise_floor_to 抬到的 6", arm.name());
            assert_eq!(simulation.candidacy_floor(), 0, "{} 旧下界：根上带的 F 都是 0", arm.name());
            assert_eq!(simulation.candidate_floor(), 6, "{} K10a：条款形下界 = 6", arm.name());
            assert_eq!(simulation.candidate_floor(), simulation.candidacy_floor() + 6, "{} g = 6", arm.name());
            let (old_fakes, old_candidates, _, _) = simulation.count_fake_candidates_at_floor(simulation.candidacy_floor());
            let (new_fakes, new_candidates, _, _) = simulation.count_fake_candidates();
            assert_eq!((old_candidates, new_candidates, old_fakes, new_fakes), (11, 5, 2, 0), "{} K10 候选数 旧 11、CF 5；假候选 旧 2、CF 0", arm.name());
            assert_eq!(simulation.oldest_candidate_root_at_floor(simulation.candidacy_floor()).map(|root| root.txg), Some(0), "{} 回退目标 旧 0", arm.name());
            assert_eq!(simulation.oldest_candidate_root_at_floor(simulation.candidate_floor()).map(|root| root.txg), Some(6), "{} 回退目标 CF 6", arm.name());
            let mut rolled_back = simulation.clone();
            assert!(rolled_back.admin_rollback(), "{} 回退要成功", arm.name());
            assert_eq!(rolled_back.newest_durable_txg, 6, "{} admin_rollback 真的回到条款形下界选的目标 6", arm.name());
            assert_eq!(rolled_back.metrics.second_rerun.rollback_target_changed, 1, "{} 新旧下界选的目标不同记一次", arm.name());
            let gap = simulation.metrics.second_rerun.candidate_floor_gap;
            assert_eq!((gap.maximum, gap.minimum), (6, 0), "{} 恢复那次 record_check 读到 g = 6，撕裂之前的 g 全是 0", arm.name());
            let recovered = simulation.metrics.second_rerun.pinned_after_recovery;
            assert_eq!((recovered.samples, recovered.maximum, recovered.last), (1, 130, 130), "{} K10 pinned_after_recovery = 13 × 10，恢复只采一次", arm.name());
            assert_eq!(simulation.metrics.second_rerun.system_configuration_below_root_floor_events, 0, "{} K16", arm.name());
        }
    }

    /// 同一状态上旧候选下界的臂（开关关着）逐次相同：候选下界仍是 0。
    #[test]
    fn k10_the_old_candidate_floor_arm_does_not_see_the_system_configuration_floor() {
        let simulation = k10_state(Arm::TightenedStatesWindowed(WindowArm::Window));
        assert_eq!(simulation.system_configuration_floor, 6, "开关开不开都记系统配置 F");
        assert_eq!(simulation.candidate_floor(), 0, "开关关着：候选下界仍是旧的 0");
        let (fakes, candidates, _, _) = simulation.count_fake_candidates();
        assert_eq!((fakes, candidates), (2, 11), "旧下界：候选 11、假候选 2");
        assert_eq!(simulation.metrics.second_rerun.pinned_after_recovery.samples, 0, "旧臂不采第 2 次重跑的新量");
    }

    /// 变异 MR1、MR2、MC1 的取样点：第 1 次 K3 的构造上 RGall 的门槛取整环（9）、候选集不滤窗口（12 个）、上限 18；窗口那一族是 11 / 11。
    #[test]
    fn k11_rgall_gate_is_nine_and_candidates_twelve_where_the_window_arm_has_gate_eleven_and_candidates_eleven() {
        let rgall = anchor_two_state(WHOLE_RING_READING_P_ARM, false);
        let window = anchor_two_state(CLAUSE_WINDOW_ARM, false);
        let mut expected_ring: Vec<u64> = vec![9];
        expected_ring.extend(11..=20);
        expected_ring.push(22);
        for simulation in [&rgall, &window] {
            assert_eq!(sorted_ring_txgs(simulation), expected_ring, "K11 ring = [9, 11, …, 20, 22]");
            assert_eq!(simulation.newest_durable_txg, 22);
        }
        assert_eq!((rgall.reuse_bound(), window.reuse_bound()), (9, 11), "K11 rgall_gate = 9 window_gate = 11（F_生效 = 0）");
        assert_eq!((rgall.count_fake_candidates().1, window.count_fake_candidates().1), (12, 11), "K11 rgall_candidates = 12 window_candidates = 11");
        assert_eq!(rgall.floor_upper_limit(), 18, "K11 rgall_ceiling = min(盘 0 最新 20, 盘 1 最新 22, 第 4 新的非空 18)");
        assert_eq!(rgall.slots_holding_a_valid_root_outside_the_window(), 1, "K11 holes (slots outside window) = 1");
        for (arm, mut simulation) in [(WHOLE_RING_READING_P_ARM, rgall), (CLAUSE_WINDOW_ARM, window)] {
            let extra_pinned_before = simulation.extra_pinned_blocks_against_the_window_gate();
            // 释放代 10、11 各放一块已释放块：落在 (9, 11] 里，整环门槛扣住、窗口门槛放得回
            simulation.pool.blocks[0] = BlockState::Freed { allocated_at: 1, freed_at: 10 };
            simulation.pool.blocks[1] = BlockState::Freed { allocated_at: 1, freed_at: 11 };
            let extra_pinned_after = simulation.extra_pinned_blocks_against_the_window_gate();
            if arm == WHOLE_RING_READING_P_ARM {
                assert_eq!((extra_pinned_before, extra_pinned_after), (10, 10 + 2), "RGall：释放代 10、11 的元数据各 5 块加手放的 2 块，x = 12，h = 1");
            } else {
                assert_eq!((extra_pinned_before, extra_pinned_after), (0, 0), "V8：窗口那一族的回收门槛就是窗口门槛，x 恒 0");
            }
        }
    }

    /// 变异 MR3、MR4 的取样点：第 1 次 K5 的构造（S = 1，txg 1..9，7 写失败）上 RGall 的上限与滞后量下界取整环（4），窗口读法是 8。
    #[test]
    fn k5r_rgall_ceiling_and_lag_bound_read_the_whole_ring() {
        for arm in [WHOLE_RING_READING_P_ARM, WHOLE_RING_READING_Q_ARM] {
            let simulation = anchor_three_state(arm);
            assert_eq!(simulation.floor_upper_limit(), 4, "{} K5r rgall_ceiling = 4（窗口读法 P 是 8）", arm.name());
            assert_eq!(simulation.lag_bound_txg(), 4, "{} K5r rgall_lag_bound = 4（读法 a 是 8）", arm.name());
        }
        let window = anchor_three_state(CLAUSE_WINDOW_ARM);
        assert_eq!((window.floor_upper_limit(), window.lag_bound_txg()), (8, 8), "对照：条款形下界的窗口臂仍是 8 / 8（只换候选下界，不换窗口取法）");
    }

    /// 变异 MR5 的取样点：S = 1，txg 1..5 全部非空，管理员回退到候选集里最旧的根 3；盘 1 一条有效根都没有：读法 P 上限 3、读法 Q 上限 0。
    #[test]
    fn k12_rgall_reading_p_skips_the_disk_without_a_valid_root_and_reading_q_counts_it_as_zero() {
        for (arm, expected_ceiling) in [(WHOLE_RING_READING_P_ARM, 3u64), (WHOLE_RING_READING_Q_ARM, 0)] {
            let mut simulation = simulation_with(arm, 1, STEADY_FILL, true);
            publish_creating_windows_up_to(&mut simulation, 5, None);
            let ring_by_slot: Vec<Option<u64>> = simulation.ring.iter().map(|root| root.map(|record| record.txg)).collect();
            assert_eq!(ring_by_slot, vec![Some(3), Some(4), Some(5)], "K12 ring = {{0: 3, 1: 4, 2: 5}}");
            assert!(simulation.admin_rollback(), "候选集里最旧的根 3 是回退目标");
            assert_eq!(sorted_valid_txgs(&simulation), vec![3], "K12 valid_after_rollback = {{0: 3}}：盘 1 上一条有效根都没有");
            assert_eq!(simulation.newest_valid_txg_on_disk(1), None);
            assert_eq!(simulation.floor_upper_limit(), expected_ceiling, "{} K12 ceiling_P = 3 ceiling_Q = 0", arm.name());
            assert_eq!(simulation.metrics.second_rerun.disk_without_valid_root_events, 0, "回退之前没有读过上限");
            simulation.observe_ceiling_difference();
            assert_eq!(simulation.metrics.second_rerun.disk_without_valid_root_events, 1, "{} K12 disk_without_valid_root_events 记一次", arm.name());
        }
    }

    /// 变异 MR6 的取样点：K11 的状态套 RGall_short 的修饰（txg + 9 > 22 那一段），门槛 14；整环是 9。
    #[test]
    fn k9r_shortened_window_gate_of_the_rgall_positive_controls_is_fourteen() {
        for whole_ring_arm in WHOLE_RING_SHORT_ARMS {
            let simulation = anchor_two_state(Arm::WholeRing(whole_ring_arm), false);
            assert_eq!(simulation.reuse_bound(), 14, "{} K9r rgall_short_gate = 14", whole_ring_arm.name());
        }
        assert_eq!(anchor_two_state(WHOLE_RING_READING_P_ARM, false).reuse_bound(), 9, "对照：RGall 整环门槛 9");
    }

    /// K6r：steady 里从稳态点触发一次完整处置，RGall 两条臂各 3 或 4 次发布、扣住最多 5 + 2c、滞后量正好 39；S = 1 时 3 次。
    #[test]
    fn k6r_rgall_arms_in_steady_use_three_or_four_publications_and_lag_three_windows() {
        for whole_ring_arm in WHOLE_RING_ARMS {
            for (slots_per_region, cost, publications, maximum, pinned, lag) in [
                (1u64, 1u64, vec![3u64], 3u64, 6u64, 0u64),
                (1, 5, vec![3], 3, 10, 0),
                (4, 1, vec![3, 4], 4, 7, 39),
                (4, 5, vec![3, 4], 4, 15, 39),
                (16, 1, vec![3, 4], 4, 7, 39),
                (16, 5, vec![3, 4], 4, 15, 39),
            ] {
                let arm = Arm::WholeRing(whole_ring_arm);
                let (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released) = push_probe_over_landings(arm, slots_per_region, cost);
                assert!(is_all_released, "{} S={slots_per_region} c={cost} 处置之后目标必须放回", arm.name());
                assert_eq!(publication_set, publications, "{} S={slots_per_region} c={cost} 三种落点的发布数", arm.name());
                assert_eq!(maximum_publications, maximum, "{} S={slots_per_region} c={cost} 最多 4 次", arm.name());
                assert_eq!(maximum_pinned, pinned, "{} S={slots_per_region} c={cost} 滞后量之外扣住最多 5 + 2c", arm.name());
                assert_eq!(maximum_lag, lag, "{} S={slots_per_region} c={cost} 滞后量 lag 3 windows x 13 = 39", arm.name());
            }
        }
    }

    fn stuck_from_the_first_window(arm: Arm, stuck_disk: usize, fill: f64) -> Simulation {
        let mut simulation = simulation_with(arm, 4, fill, false);
        simulation.stuck_disk = Some(stuck_disk);
        simulation.stuck_marks_origin_txg = Some(simulation.next_txg);
        simulation
    }

    /// 变异 MS1、MS2、MC1、MC2 的取样点：S = 4，占用 0.30，盘 1 从第一个窗口起卡住（区域 1 的槽写不进去），不盘紧；每个窗口放掉 13 块。
    #[test]
    fn k13_disk_one_stuck_rgall_pins_thirteen_blocks_per_window_and_the_window_arm_stays_at_ninety_one() {
        for (windows, newest_txg, window_gate, rgall_pinned, window_pinned) in [(8u64, 12u64, 2u64, 104u64, 91u64), (16, 24, 14, 208, 91), (32, 48, 38, 416, 91)] {
            let mut rgall = stuck_from_the_first_window(WHOLE_RING_READING_P_ARM, 1, 0.30);
            let mut window = stuck_from_the_first_window(CLAUSE_WINDOW_ARM, 1, 0.30);
            let mut ring_gate = stuck_from_the_first_window(CLAUSE_RING_GATE_ARM, 1, 0.30);
            for simulation in [&mut rgall, &mut window, &mut ring_gate] {
                run_delete_create_windows(simulation, windows);
                assert_eq!(simulation.newest_durable_txg, newest_txg, "K13 w = {windows}：N = {newest_txg}");
                assert_eq!(simulation.slots_holding_a_valid_root_outside_the_window(), 4, "K13 w = {windows}：盘 1 的 4 个槽一直住着第 0 代，窗口外");
            }
            assert_eq!((rgall.reuse_bound(), window.reuse_bound(), ring_gate.reuse_bound()), (0, window_gate, 0), "K13 w = {windows}：RGall 门槛 0、窗口门槛 {window_gate}");
            assert_eq!((rgall.pinned_count(), window.pinned_count(), ring_gate.pinned_count()), (rgall_pinned, window_pinned, rgall_pinned), "K13 w = {windows}：扣住的块数");
        }
        let mut rgall = stuck_from_the_first_window(WHOLE_RING_READING_P_ARM, 1, 0.30);
        let mut window = stuck_from_the_first_window(CLAUSE_WINDOW_ARM, 1, 0.30);
        let mut ring_gate = stuck_from_the_first_window(CLAUSE_RING_GATE_ARM, 1, 0.30);
        for simulation in [&mut rgall, &mut window, &mut ring_gate] {
            run_delete_create_windows(simulation, 16);
        }
        assert_eq!(
            (rgall.extra_pinned_blocks_against_the_window_gate(), ring_gate.extra_pinned_blocks_against_the_window_gate(), window.extra_pinned_blocks_against_the_window_gate()),
            (117, 117, 0), "K13 w = 16 时 x = 117（释放代落在 (0, 14] 里的 9 个窗口 × 13），窗口臂按定义 0"
        );
    }

    /// 刻度（T₀ = 1）k = 1、2、4 在第 9、17、33 个窗口；RGall 117 / 221 / 429，窗口臂 91 / 91 / 91；s(2→4) RGall = 208 / 208 = 1，窗口臂 = 0。
    #[test]
    fn k13_marks_at_one_two_and_four_revolutions_give_slope_one_for_rgall_and_zero_for_the_window_arm() {
        for (arm, expected_pinned, expected_increase_two_to_four) in [(WHOLE_RING_READING_P_ARM, [117u64, 221, 429], 208i64), (CLAUSE_WINDOW_ARM, [91, 91, 91], 0)] {
            let mut simulation = stuck_from_the_first_window(arm, 1, 0.30);
            let mut windows_at_mark: [Option<u64>; 3] = [None; 3];
            for window in 1..=40u64 {
                run_delete_create_windows(&mut simulation, 1);
                for (mark_index, mark) in simulation.metrics.second_rerun.marks.iter().enumerate() {
                    if mark.is_some() && windows_at_mark[mark_index].is_none() {
                        windows_at_mark[mark_index] = Some(window);
                    }
                }
            }
            assert_eq!(windows_at_mark, [Some(9), Some(17), Some(33)], "{} K13 刻度在第 9、17、33 个窗口", arm.name());
            let marks = simulation.metrics.second_rerun.marks.map(|mark| mark.expect("三个刻度都走到了"));
            assert_eq!(marks.iter().map(|mark| mark.published_txg).collect::<Vec<u64>>(), vec![14, 26, 50], "{} K13 mark N", arm.name());
            assert_eq!(marks.iter().map(|mark| mark.pinned_blocks).collect::<Vec<u64>>(), expected_pinned.to_vec(), "{} K13 刻度上扣住的块数", arm.name());
            let (increase, freed) = stuck_slope_between(&marks[1], &marks[2]).expect("两个刻度之间放掉了块");
            assert_eq!((increase, freed), (expected_increase_two_to_four, 208), "{} s(2→4)：增量 / 两个刻度之间 16 个窗口 × 13 块", arm.name());
        }
    }

    /// 判别力自证：门槛 1/2 落在两个点（s = 1 与 s = 0）之间；挪到 0 与 1 之外任一侧，两个点由分开变成同判。
    #[test]
    fn k13_bounded_threshold_one_half_separates_the_two_slopes_and_moving_it_outside_zero_to_one_makes_them_agree() {
        let rgall_slope = (208i64, 208i64);
        let window_slope = (0i64, 208i64);
        let is_below = |numerator: i64, denominator: i64, slope: (i64, i64)| slope.0 * denominator < slope.1 * numerator;
        assert_eq!((is_stuck_slope_bounded(rgall_slope.0, rgall_slope.1), is_stuck_slope_bounded(window_slope.0, window_slope.1)), (false, true), "门槛 1/2：RGall 没有上界、窗口臂有上界");
        assert_eq!((is_below(1, 2, rgall_slope), is_below(1, 2, window_slope)), (false, true), "本测试自己的式子在 1/2 上与装置一致");
        assert_eq!((is_below(0, 1, rgall_slope), is_below(0, 1, window_slope)), (false, false), "门槛挪到 0：两个点同判没有上界");
        assert_eq!((is_below(2, 1, rgall_slope), is_below(2, 1, window_slope)), (true, true), "门槛挪到 2：两个点同判有上界");
        assert_eq!((is_below(1, 1, rgall_slope), is_below(1, 1, window_slope)), (false, true), "门槛恰为 1：仍分开");
        assert!(!is_stuck_slope_bounded(104, 208) && is_stuck_slope_bounded(103, 208), "恰好 1/2 不算有上界，不取整");
    }

    /// 变异 MS2 的取样点：盘 0 卡住（H3，区域 0 与 2）。
    #[test]
    fn k14_disk_zero_stuck_rgall_pins_thirteen_blocks_per_window_and_the_window_arm_stays_at_thirty_nine() {
        for (windows, newest_txg, window_gate, rgall_pinned, window_pinned, hole_slots) in [(4u64, 10u64, 0u64, 52u64, 52u64, 0u64), (8, 22, 13, 104, 39, 8), (16, 46, 37, 208, 39, 8)] {
            let mut rgall = stuck_from_the_first_window(WHOLE_RING_READING_P_ARM, 0, 0.30);
            let mut window = stuck_from_the_first_window(CLAUSE_WINDOW_ARM, 0, 0.30);
            for simulation in [&mut rgall, &mut window] {
                run_delete_create_windows(simulation, windows);
                assert_eq!(simulation.newest_durable_txg, newest_txg, "K14 w = {windows}：N = {newest_txg}");
                assert_eq!(simulation.slots_holding_a_valid_root_outside_the_window(), hole_slots, "K14 w = {windows}：窗口外的槽数");
            }
            assert_eq!((rgall.reuse_bound(), window.reuse_bound()), (0, window_gate), "K14 w = {windows}：门槛");
            assert_eq!((rgall.pinned_count(), window.pinned_count()), (rgall_pinned, window_pinned), "K14 w = {windows}：扣住的块数");
        }
    }

    /// `Metrics` 里除轨迹与第 2 次重跑的新量之外的全部字段，K15 逐个比。
    fn core_metrics_signature(metrics: &Metrics) -> String {
        format!("{:?}", [
            metrics.fake_candidates, metrics.unrecoverable, metrics.retention_violations, metrics.checks, metrics.minimum_candidates, metrics.candidate_sum,
            metrics.nonempty_candidate_sum, metrics.minimum_distinct_states, metrics.distinct_state_sum, metrics.stalls, metrics.forced_publications, metrics.user_windows,
            metrics.admissions_under_pressure, metrics.maximum_publications_per_admission, metrics.false_enospc_raw, metrics.false_enospc_arm, metrics.true_enospc,
            metrics.write_after_delete_failed, metrics.first_write_after_delete_failure_window, metrics.last_write_after_delete_failure_window, metrics.halted_before_counting,
            metrics.root_write_failures_injected, metrics.lag_sum, metrics.lag_maximum, metrics.lag_samples,
        ])
    }

    /// 检查本身分得出差别：两条确实不同的臂（臂 A 与 G2S）在 steady 里的签名不同。
    #[test]
    fn k15_the_signature_tells_two_different_arms_apart() {
        let ring = core_metrics_signature(&run_world(Arm::RingPersisted, World::Steady, 4, 0, 0));
        let states = core_metrics_signature(&run_world(Arm::TightenedStates, World::Steady, 4, 0, 0));
        assert_ne!(ring, states, "臂 A 与 G2S 的 Metrics 字段不同，签名要看得出来");
    }

    /// K15 等价留档：候选下界取旧式的 RGall_Q 与原 G2S 逐处同式（回收门槛、候选集、上限、处置目标、滞后量都取整环、读法 Q），`Metrics` 除轨迹之外每个字段逐个相等。
    #[test]
    fn k15_rgall_reading_q_with_the_old_candidate_floor_agrees_with_the_original_states_arm_in_every_field_except_the_trajectory() {
        let original = Arm::TightenedStates;
        let whole_ring = Arm::WholeRing(WholeRingArm::ReadingQWithTheOldCandidateFloor);
        for world in WORLDS {
            for slots_per_region in SLOTS_PER_REGION_SWEEP {
                let lost_disks: &[usize] = if world == World::DiskLoss { &[0, 1] } else { &[0] };
                for &lost_disk in lost_disks {
                    for seed in 0..8u64 {
                        assert_eq!(
                            core_metrics_signature(&run_world(whole_ring, world, slots_per_region, seed, lost_disk)),
                            core_metrics_signature(&run_world(original, world, slots_per_region, seed, lost_disk)),
                            "K15 {} S={slots_per_region} 丢盘 {lost_disk} 种子 {seed}", world.name()
                        );
                    }
                }
            }
        }
        for seed in 0..2u64 {
            assert_eq!(
                core_metrics_signature(&run_near_full(whole_ring, 4, 1, 5, seed, None)), core_metrics_signature(&run_near_full(original, 4, 1, 5, seed, None)),
                "K15 near_full 种子 {seed}"
            );
            assert_eq!(
                core_metrics_signature(&run_broken_slot(whole_ring, 4, 1, 5, seed)), core_metrics_signature(&run_broken_slot(original, 4, 1, 5, seed)),
                "K15 broken_slot 种子 {seed}"
            );
            let whole_ring_prefix = drain_failure_prefix(whole_ring, 4, 1, 5, seed).expect("填满不该卡死");
            let original_prefix = drain_failure_prefix(original, 4, 1, 5, seed).expect("填满不该卡死");
            for pattern in drain_failure_patterns(original.drain_failure_offset_limit(12)) {
                assert_eq!(
                    core_metrics_signature(&run_drain_failure_case(&whole_ring_prefix, &pattern)), core_metrics_signature(&run_drain_failure_case(&original_prefix, &pattern)),
                    "K15 drain_failure 种子 {seed} 失败 {pattern:?}"
                );
            }
        }
    }

    /// K16：条款形下界的臂在各个世界里每次 `record_check`，系统配置 F ≥ 根上带的 F（「先写后抬」在装置里的不变量）；条款形候选集从不大于旧候选集的反面（V6）。
    #[test]
    fn k16_the_system_configuration_floor_never_falls_below_the_root_floors_and_the_clause_candidate_set_is_never_larger() {
        let clause_arms = [
            CLAUSE_WINDOW_ARM, Arm::ClauseFloorWindowed(WindowArm::DiskHold), Arm::ClauseFloorWindowed(WindowArm::RingLag), CLAUSE_RING_GATE_ARM,
            WHOLE_RING_READING_P_ARM, WHOLE_RING_READING_Q_ARM,
        ];
        let mut gap_samples_seen = 0u64;
        for arm in clause_arms {
            for world in WORLDS {
                for slots_per_region in [1u64, 4] {
                    for seed in 0..4u64 {
                        let observations = run_world(arm, world, slots_per_region, seed, 0).second_rerun;
                        assert_eq!(observations.system_configuration_below_root_floor_events, 0, "K16 {} {} S={slots_per_region} 种子 {seed}", arm.name(), world.name());
                        assert_eq!(observations.checks_new_set_strictly_larger, 0, "V6 {} {} S={slots_per_region} 种子 {seed}", arm.name(), world.name());
                        gap_samples_seen += observations.candidate_floor_gap.samples;
                    }
                }
            }
        }
        assert!(gap_samples_seen > 0, "K16 要真的读到过下界差，否则上面两条什么都没测");
    }

    /// 第 1 次的 `w_` 行不许被这一次的改动碰：G2SW 在 steady 与 admin_rollback 的 S = 4 两行，照第 1 次产物
    /// `research/results/e139-tightened-floor-2026-10-04.out` 第 934、1054 行钉死（旧下界、旧行格式）。
    #[test]
    fn first_rerun_world_rows_of_the_window_arm_stay_as_in_the_first_product() {
        let arm = Arm::TightenedStatesWindowed(WindowArm::Window);
        for (world, expected_fields) in [
            (World::Steady, "fakes=0 unrecoverable=0 retention_violations=0 checks=9600 min_candidates=12 avg_candidates_x100=1200 avg_nonempty_x100=1200 min_states=2 avg_states_x100=1186 stalls=0 false_enospc_arm=0"),
            (World::AdminRollback, "fakes=0 unrecoverable=0 retention_violations=0 checks=9677 min_candidates=1 avg_candidates_x100=1174 avg_nonempty_x100=1170 min_states=1 avg_states_x100=1156 stalls=0 false_enospc_arm=0"),
        ] {
            let total = fresh_total_over_seeds(SEED_COUNT, |seed| run_world(arm, world, 4, seed, 0));
            assert_eq!(world_outcome_fields(&total), expected_fields, "{} S=4 种子 0..24 与第 1 次产物逐字相同", world.name());
        }
    }

    /// 路径跑到了（阴性结果要能与「代码没跑到」分开）：`disk_stuck` 的卡住段在低占用上三个刻度都走到、写失败真的注入过；三种收尾都做了一次恢复。
    #[test]
    fn disk_stuck_segment_reaches_the_three_marks_and_every_ending_runs_one_recovery() {
        let stuck_end = disk_stuck_segment(WHOLE_RING_READING_P_ARM, 4, 1, 0.30, 0);
        assert!(stuck_end.metrics.second_rerun.marks.iter().all(|mark| mark.is_some()), "占用 0.30、S = 4：三个刻度都走到");
        assert!(stuck_end.metrics.root_write_failures_injected > 0, "卡住的盘上的槽真的被写过、写失败了");
        assert_eq!(stuck_end.stuck_disk, Some(1));
        for ending in STUCK_ENDINGS {
            let metrics = disk_stuck_ending(&stuck_end, ending);
            assert_eq!(metrics.second_rerun.pinned_after_recovery.samples, 1, "{} 收尾恰做一次恢复", ending.name());
        }
    }

    /// `broken_slot` 与 H1 的刻度起点设在坏槽开始的那一刻：条款形下界的臂走到三个刻度，旧臂不采。
    #[test]
    fn broken_slot_runs_record_marks_only_for_the_second_rerun_arms() {
        let clause = run_broken_slot(CLAUSE_WINDOW_ARM, 4, 1, 5, 0);
        assert!(clause.second_rerun.marks.iter().all(|mark| mark.is_some()), "坏槽 1000 个窗口里三个刻度都走到");
        let old = run_broken_slot(Arm::TightenedStatesWindowed(WindowArm::Window), 4, 1, 5, 0);
        assert!(old.second_rerun.marks.iter().all(|mark| mark.is_none()), "旧臂不采刻度");
    }

    /// 判决行的烟测：拿一小撮行簿（steady 与 admin_rollback 两个世界、S = 4、种子 0 与 1，两族每条臂各一行，再加一条臂的两个卡住段刻度）把判决行跑一遍，不 panic。
    #[test]
    fn the_second_rerun_verdict_lines_run_on_a_handful_of_rows() {
        let first_family = first_rerun_family();
        let second_family = second_rerun_family();
        let mut emitter = Emitter::new();
        let mut first_rows: Vec<WindowRow> = Vec::new();
        let mut rows: Vec<WindowRow> = Vec::new();
        for (family, target_rows) in [(&first_family, &mut first_rows), (&second_family, &mut rows)] {
            for world in [World::Steady, World::AdminRollback] {
                for arm in family.arms.iter().copied() {
                    let total = fresh_total_over_seeds(2, |seed| run_world(arm, world, 4, seed, 0));
                    emit_window_row(&mut emitter, target_rows, family, WindowRowDraft {
                        line_name: family.lines.world,
                        world_class: world.name(),
                        row_key: format!("world={} s=4 lost_disk=0", world.name()),
                        arm,
                        outcome_fields: world_outcome_fields(&total),
                        judgement: RowJudgement::of(&total, false),
                        is_report_only_world: false,
                        has_false_enospc_field: true,
                        metrics: &total,
                    });
                }
            }
        }
        assert_eq!((first_rows.len(), rows.len()), (8, 12), "第 1 次一族 4 条臂 × 2 个世界，第 2 次一族 6 条臂 × 2 个世界");
        let seed_marks: Vec<[Option<StuckMark>; 3]> = (0..2u64).map(|seed| disk_stuck_segment(WHOLE_RING_READING_P_ARM, 4, 1, 0.30, seed).metrics.second_rerun.marks).collect();
        let growth = vec![emit_stuck_growth_row(&mut emitter, second_family.lines.stuck_growth, WHOLE_RING_READING_P_ARM, "s=4 stuck_disk=1 fill=0.30", &seed_marks)];
        assert_eq!(growth[0].bound, StuckBound::Unbounded, "RGall 在占用 0.30、盘 1 卡住时 s(2→4) = 1，判没有上界");
        emit_second_rerun_verdict_lines(&mut emitter, &second_family, &first_rows, &[], &rows, &growth, &[]);
    }

    /// `disk_stuck` 世界全部组合的预演（4 条臂 × S ∈ {4, 16} × 卡住的盘 × 占用两档 × 8 个种子 × 三种收尾）：不 panic、写失败真的注入过。
    /// 耗时，平常不跑（`--ignored` 手动跑一次，整份产物跑之前用）。
    #[test]
    #[ignore = "整个 disk_stuck 世界的预演，耗时；产物跑之前手动跑一次"]
    fn disk_stuck_world_runs_every_combination_without_panicking() {
        for arm in [WHOLE_RING_READING_P_ARM, WHOLE_RING_READING_Q_ARM, CLAUSE_WINDOW_ARM, CLAUSE_RING_GATE_ARM] {
            for slots_per_region in FAULT_SLOTS_SWEEP {
                for stuck_disk in STUCK_DISKS {
                    for fill in STUCK_FILLS {
                        for seed in 0..FAULT_SEED_COUNT {
                            let stuck_end = disk_stuck_segment(arm, slots_per_region, stuck_disk, fill, seed);
                            assert!(stuck_end.metrics.root_write_failures_injected > 0, "{} S={slots_per_region} 卡住的盘 {stuck_disk} 占用 {fill:.2} 种子 {seed}：写失败真的注入过", arm.name());
                            for ending in STUCK_ENDINGS {
                                let _ending_metrics = disk_stuck_ending(&stuck_end, ending);
                            }
                        }
                    }
                }
            }
        }
    }
}
