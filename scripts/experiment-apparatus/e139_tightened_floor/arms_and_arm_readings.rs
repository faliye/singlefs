//! 被测的臂（准入与上限取法的几种组合）、窗口臂与整圈臂的枚举、各自读到的上限与滞后界。

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Arm {
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
pub(super) enum WholeRingArm {
    /// RGall：某块幸存盘整环里一条有效根都没有时，那块盘不进「每块幸存盘上最新的有效根」取小（读法 P，`crates/` 今天的上限形状）。被测。
    SkipsRootlessDisks,
    /// RGall_Q：那样的盘记 0、F 不抬（读法 Q）。被测。
    CountsRootlessDisksAsZero,
    /// 阳性对照：RGall 只有回收门槛里的「最旧有效根」换成窗口缩短一行（txg + 3(S − 1) > N）的那一段里的最旧有效根。
    SkipsRootlessDisksShortened,
    CountsRootlessDisksAsZeroShortened,
    /// 只给单测 K15：RGall_Q 的候选下界取旧式（不并系统配置 F），用来对 Q 与原 G2S 逐处同式。不进任何一行产物。
    #[cfg_attr(not(test), allow(dead_code, reason = "只给单测 K15 用，产物里没有这条臂"))]
    CountsRootlessDisksAsZeroWithTheOldCandidateFloor,
    /// 第 3 次重跑（`research/prompts/e139-r3-prereg.md` 第五节「臂」）：RGall 换上 `crates/singlefs-core/` 今天的回收门槛（CG 三处）。被测。
    CoreGateCeilingSkipsRootlessDisks,
    /// RGall_Q 换上 CG。被测（只报）。
    CoreGateCeilingCountsRootlessDisksAsZero,
    /// 两条 CG 被测臂的阳性对照：最旧那一项里「环里最旧有效根」换成缩短一行的窗口里的最旧有效根，钉值照并。
    CoreGateCeilingSkipsRootlessDisksShortened,
    CoreGateCeilingCountsRootlessDisksAsZeroShortened,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum WholeRingCeilingReading {
    DisksWithoutAValidRootAreSkipped,
    DisksWithoutAValidRootCountAsZero,
}

impl WholeRingArm {
    pub(super) fn name(self) -> &'static str {
        match self {
            WholeRingArm::SkipsRootlessDisks => "RGall_ring_all",
            WholeRingArm::CountsRootlessDisksAsZero => "RGall_Q_ring_all",
            WholeRingArm::SkipsRootlessDisksShortened => "RGall_short",
            WholeRingArm::CountsRootlessDisksAsZeroShortened => "RGall_Q_short",
            WholeRingArm::CountsRootlessDisksAsZeroWithTheOldCandidateFloor => "RGall_Q_old_candidate_floor",
            WholeRingArm::CoreGateCeilingSkipsRootlessDisks => "RGall_core_gate",
            WholeRingArm::CoreGateCeilingCountsRootlessDisksAsZero => "RGall_Q_core_gate",
            WholeRingArm::CoreGateCeilingSkipsRootlessDisksShortened => "RGall_core_gate_short",
            WholeRingArm::CoreGateCeilingCountsRootlessDisksAsZeroShortened => "RGall_Q_core_gate_short",
        }
    }
    pub(super) fn ceiling_reading(self) -> WholeRingCeilingReading {
        match self {
            WholeRingArm::SkipsRootlessDisks | WholeRingArm::SkipsRootlessDisksShortened => WholeRingCeilingReading::DisksWithoutAValidRootAreSkipped,
            WholeRingArm::CoreGateCeilingSkipsRootlessDisks | WholeRingArm::CoreGateCeilingSkipsRootlessDisksShortened => WholeRingCeilingReading::DisksWithoutAValidRootAreSkipped,
            WholeRingArm::CountsRootlessDisksAsZero | WholeRingArm::CountsRootlessDisksAsZeroShortened | WholeRingArm::CountsRootlessDisksAsZeroWithTheOldCandidateFloor => {
                WholeRingCeilingReading::DisksWithoutAValidRootCountAsZero
            }
            WholeRingArm::CoreGateCeilingCountsRootlessDisksAsZero | WholeRingArm::CoreGateCeilingCountsRootlessDisksAsZeroShortened => WholeRingCeilingReading::DisksWithoutAValidRootCountAsZero,
        }
    }
    pub(super) fn is_reclaim_window_shortened(self) -> bool {
        match self {
            WholeRingArm::SkipsRootlessDisksShortened | WholeRingArm::CountsRootlessDisksAsZeroShortened => true,
            WholeRingArm::CoreGateCeilingSkipsRootlessDisksShortened | WholeRingArm::CoreGateCeilingCountsRootlessDisksAsZeroShortened => true,
            WholeRingArm::SkipsRootlessDisks | WholeRingArm::CountsRootlessDisksAsZero | WholeRingArm::CountsRootlessDisksAsZeroWithTheOldCandidateFloor => false,
            WholeRingArm::CoreGateCeilingSkipsRootlessDisks | WholeRingArm::CoreGateCeilingCountsRootlessDisksAsZero => false,
        }
    }
    pub(super) fn uses_the_clause_candidate_floor(self) -> bool {
        match self {
            WholeRingArm::SkipsRootlessDisks | WholeRingArm::CountsRootlessDisksAsZero | WholeRingArm::SkipsRootlessDisksShortened | WholeRingArm::CountsRootlessDisksAsZeroShortened => true,
            WholeRingArm::CoreGateCeilingSkipsRootlessDisks | WholeRingArm::CoreGateCeilingCountsRootlessDisksAsZero | WholeRingArm::CoreGateCeilingSkipsRootlessDisksShortened | WholeRingArm::CoreGateCeilingCountsRootlessDisksAsZeroShortened => true,
            WholeRingArm::CountsRootlessDisksAsZeroWithTheOldCandidateFloor => false,
        }
    }
    /// 第 3 次重跑：回收门槛照 `crates/singlefs-core/` 今天的写法取（CG-F、CG-P、CG-R 三处）。
    pub(super) fn uses_the_core_gate(self) -> bool {
        match self {
            WholeRingArm::CoreGateCeilingSkipsRootlessDisks | WholeRingArm::CoreGateCeilingCountsRootlessDisksAsZero | WholeRingArm::CoreGateCeilingSkipsRootlessDisksShortened | WholeRingArm::CoreGateCeilingCountsRootlessDisksAsZeroShortened => true,
            WholeRingArm::SkipsRootlessDisks | WholeRingArm::CountsRootlessDisksAsZero | WholeRingArm::SkipsRootlessDisksShortened | WholeRingArm::CountsRootlessDisksAsZeroShortened | WholeRingArm::CountsRootlessDisksAsZeroWithTheOldCandidateFloor => false,
        }
    }
}

/// RGall 一族进全部格的两条被测臂，与它们的阳性对照（只进 steady、S ∈ {4, 16}）。
pub(super) const WHOLE_RING_ARMS: [WholeRingArm; 2] = [WholeRingArm::SkipsRootlessDisks, WholeRingArm::CountsRootlessDisksAsZero];
pub(super) const WHOLE_RING_SHORT_ARMS: [WholeRingArm; 2] = [WholeRingArm::SkipsRootlessDisksShortened, WholeRingArm::CountsRootlessDisksAsZeroShortened];
/// 第 3 次重跑的两条 CG 被测臂，与它们的阳性对照（进 steady、torn_newest、activation_faults，S ∈ {4, 16}）。
pub(super) const CORE_GATE_ARMS: [WholeRingArm; 2] = [WholeRingArm::CoreGateCeilingSkipsRootlessDisks, WholeRingArm::CoreGateCeilingCountsRootlessDisksAsZero];
pub(super) const CORE_GATE_SHORT_ARMS: [WholeRingArm; 2] = [WholeRingArm::CoreGateCeilingSkipsRootlessDisksShortened, WholeRingArm::CoreGateCeilingCountsRootlessDisksAsZeroShortened];

/// 第 1 次重跑的七条臂（`research/prompts/e139-r1-prereg.md` 第五节）：回收门槛改成一圈窗口（txg + 3S > N；
/// 核心层后来退回整个环，那个判法函数已删，见 C842（一圈窗口判法成了死代码））之后的臂 G2S，与它的读法臂、旧门槛对照、阳性对照。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum WindowArm {
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
pub(super) enum ReclaimGate {
    /// max(F_生效, 窗口里最旧有效根)。
    OldestValidRootInTheWindow,
    /// max(F_生效, 环里最旧有效根)，旧式。
    OldestValidRootInTheWholeRing,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum CeilingReading {
    /// 读法 P：窗口里一条有效根都没有的盘不进「每块幸存盘上最新的有效根」取小。
    DisksWithoutAWindowRootAreSkipped,
    /// 读法 Q：那样的盘记 0，F 不抬。
    DisksWithoutAWindowRootCountAsZero,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum LagBoundReading {
    /// 读法 a：「第 4 新的非空有效根」与抬 F 的上限同一个窗口里取。
    FourthNewestInTheWindow,
    /// 读法 b：整环取（原 G2S 的取法）。
    FourthNewestInTheWholeRing,
}

impl WindowArm {
    pub(super) fn name(self) -> &'static str {
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
    pub(super) fn reclaim_gate(self) -> ReclaimGate {
        match self {
            WindowArm::RingGate => ReclaimGate::OldestValidRootInTheWholeRing,
            WindowArm::Window | WindowArm::DiskHold | WindowArm::RingLag | WindowArm::WindowShort | WindowArm::DiskHoldShort | WindowArm::RingLagShort => {
                ReclaimGate::OldestValidRootInTheWindow
            }
        }
    }
    pub(super) fn is_reclaim_window_shortened(self) -> bool {
        match self {
            WindowArm::WindowShort | WindowArm::DiskHoldShort | WindowArm::RingLagShort => true,
            WindowArm::Window | WindowArm::DiskHold | WindowArm::RingLag | WindowArm::RingGate => false,
        }
    }
    pub(super) fn ceiling_reading(self) -> CeilingReading {
        match self {
            WindowArm::DiskHold | WindowArm::DiskHoldShort => CeilingReading::DisksWithoutAWindowRootCountAsZero,
            WindowArm::Window | WindowArm::RingLag | WindowArm::RingGate | WindowArm::WindowShort | WindowArm::RingLagShort => {
                CeilingReading::DisksWithoutAWindowRootAreSkipped
            }
        }
    }
    pub(super) fn lag_bound_reading(self) -> LagBoundReading {
        match self {
            WindowArm::RingLag | WindowArm::RingLagShort => LagBoundReading::FourthNewestInTheWholeRing,
            WindowArm::Window | WindowArm::DiskHold | WindowArm::RingGate | WindowArm::WindowShort | WindowArm::DiskHoldShort => {
                LagBoundReading::FourthNewestInTheWindow
            }
        }
    }
}

/// 进全部格的四条臂（被测三条加旧门槛对照）。
pub(super) const WINDOW_ARMS: [WindowArm; 4] = [WindowArm::Window, WindowArm::DiskHold, WindowArm::RingLag, WindowArm::RingGate];
/// 窗口这一项的阳性对照：只进 steady、S ∈ {4, 16}。
pub(super) const WINDOW_SHORT_ARMS: [WindowArm; 3] = [WindowArm::WindowShort, WindowArm::DiskHoldShort, WindowArm::RingLagShort];
pub(super) const WINDOW_SHORT_SLOTS_SWEEP: [u64; 2] = [4, 16];

/// 一圈窗口：txg + 窗口长度 > 锚。窗口长度是环长 R × S；阳性对照臂的回收门槛缩短一行（少 R）。
pub(super) fn txg_is_inside_window(txg: u64, anchor: u64, window_length: u64) -> bool {
    txg + window_length > anchor
}

pub(super) const ARMS: [Arm; 8] = [
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
pub(super) const NEAR_FULL_FAULT_ARMS: [Arm; 4] = [Arm::RingPersisted, Arm::FloorPerDisk, Arm::TightenedRoots, Arm::TightenedStates];
pub(super) const COST_GROWTH_ARMS: [Arm; 5] = [Arm::RingPersisted, Arm::FloorPerDisk, Arm::TightenedRoots, Arm::TightenedStates, Arm::TightenedRootsAtMount];
pub(super) const CANDIDATE_ARMS: [Arm; 2] = [Arm::TightenedRoots, Arm::TightenedStates];

impl Arm {
    pub(super) fn name(self) -> &'static str {
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
    pub(super) fn uses_floor(self) -> bool {
        match self {
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum => false,
            Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_)
            | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => true,
        }
    }
    /// 候选下界取条款形（把系统配置 F 并进去，第 2 次重跑第 1 行）。
    pub(super) fn uses_the_clause_candidate_floor(self) -> bool {
        match self {
            Arm::ClauseFloorWindowed(_) => true,
            Arm::WholeRing(whole_ring_arm) => whole_ring_arm.uses_the_clause_candidate_floor(),
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates
            | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_) => false,
        }
    }
    /// 第 3 次重跑：回收门槛照 `crates/singlefs-core/` 今天的写法取（只有 RGall 一族的 CG 臂）。
    pub(super) fn uses_the_core_gate(self) -> bool {
        match self {
            Arm::WholeRing(whole_ring_arm) => whole_ring_arm.uses_the_core_gate(),
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates
            | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) => false,
        }
    }
    /// 第 2 次重跑的新量（下界差、ⓐⓑⓒ、刻度）只在这一族臂上采；`w_` 行不打印它们，旧臂不为它们多算。
    pub(super) fn collects_second_rerun_observations(self) -> bool {
        match self {
            Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => true,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates
            | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_) => false,
        }
    }
    /// 第 1 次的四样轨迹（门槛差、上限差、滞后量、`df`）读哪些臂：窗口一族（两个下界口径）与 RGall 一族。
    pub(super) fn records_the_four_trajectories(self) -> bool {
        match self {
            Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => true,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates
            | Arm::TightenedRootsAtMount => false,
        }
    }
    /// 被测的两条：判据 3–8 只对它们判。
    pub(super) fn is_candidate(self) -> bool {
        match self {
            Arm::TightenedRoots | Arm::TightenedStates => true,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_)
            | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => false,
        }
    }
    /// 收严形态三条共有的：准入按最坏残留收费、界带 k_tol 的余量。
    pub(super) fn is_tightened(self) -> bool {
        match self {
            Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => true,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit => false,
        }
    }
    /// 上限按第 4 新的非空根取。
    pub(super) fn counts_states(self) -> bool {
        match self {
            Arm::TightenedStates | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => true,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedRootsAtMount => false,
        }
    }
    /// 保留池、残留与 df 按哪个开销算：被测臂按声明的最坏值，其余按挂载时的值。
    pub(super) fn formula_cost(self, mount_cost: u64, declared_maximum_cost: u64) -> u64 {
        match self {
            Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => declared_maximum_cost,
            Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRootsAtMount => mount_cost,
        }
    }
    /// 预注册的界 B：一次准入最多用几次发布（含发布正在攒的窗口）。
    pub(super) fn publication_bound(self, ring_depth: u64) -> u64 {
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
    pub(super) fn reserve_blocks(self, ring_depth: u64, formula_cost: u64) -> u64 {
        match self {
            Arm::RingPersisted | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_)
            | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => {
                2 * METADATA_BLOCKS_PER_PUBLICATION + (self.publication_bound(ring_depth) - 1) * formula_cost
            }
            Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg => CONTROL_ARM_RESERVE_BLOCKS,
        }
    }
    /// 预注册：一次完整处置之后还扣着的最坏块数 = 5 + (B − 2) × c。
    pub(super) fn push_residue_blocks(self, ring_depth: u64, formula_cost: u64) -> u64 {
        match self {
            Arm::RingPersisted | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_)
            | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => {
                METADATA_BLOCKS_PER_PUBLICATION + (self.publication_bound(ring_depth) - 2) * formula_cost
            }
            Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg => 0,
        }
    }
    /// drain_failure 里失败位置扫到第几次根槽写（第 0 次是发布正在攒的窗口）。
    pub(super) fn drain_failure_offset_limit(self, ring_depth: u64) -> u64 {
        match self {
            Arm::RingPersisted => ring_depth.min(DRAIN_FAILURE_RING_ARM_OFFSET_LIMIT),
            Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRoots | Arm::TightenedStates | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_)
            | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg => {
                self.publication_bound(ring_depth)
            }
        }
    }
}

#[cfg(test)]
#[path = "arms_and_arm_readings_tests.rs"]
mod tests;
