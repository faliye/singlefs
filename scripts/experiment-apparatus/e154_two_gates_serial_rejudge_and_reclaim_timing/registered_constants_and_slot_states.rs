//! 登记的常量、槽的状态、闸的读法、回收时点、停法与一条臂。

// ===== 常量（登记「读条款时读到的数」「判问法时自己算出来的」） =====

/// 根环区域数 R（D22（单元原子性怎么合成） 已定项 2）。
pub(super) const RING_REGIONS: u64 = 3;
/// 区域设备归属（D2（RAID 条带策略） 已定项 7，`crates/singlefs-format/src/lib.rs:192` 同值）。
pub(super) const REGION_DEVICES: [u64; 3] = [0, 1, 0];
/// 一次挂载允许的连续实例切换次数上限（D23（journal 的角色与格式） 已定项 14）。
pub(super) const INSTANCE_SWITCH_LIMIT: u64 = 3;
/// 一次处置容忍的根槽写失败次数（D16（发布语义） 已定项 1）。
const TOLERATED_ROOT_WRITE_FAILURE_COUNT: u64 = 2;
/// 一次准入最多的发布尝试预算 B = 4 + 2 × k_tol（D16（发布语义） 已定项 1）。
pub(super) const RETRY_BUDGET_PUBLICATIONS: u64 = 4 + 2 * TOLERATED_ROOT_WRITE_FAILURE_COUNT;
/// 最少保留的可退到的状态数（D16（发布语义） 已定项 1）。
pub(super) const MINIMUM_RETAINED_STATES: u64 = 4;
/// 重试循环的最多触动次数 J_max = 3S + 5（登记「五、5.4」）。
pub(super) fn retry_touch_budget(slots_per_region: u64) -> u64 {
    3 * slots_per_region + 5
}

// ===== 5.1 槽状态 =====

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum SlotState {
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
pub(super) enum GateReading {
    Serial,
    SerialRejudge,
    Union,
    UnionFullDeduction,
}

impl GateReading {
    pub(super) fn label(self) -> &'static str {
        match self {
            GateReading::Serial => "串",
            GateReading::SerialRejudge => "串重判",
            GateReading::Union => "合",
            GateReading::UnionFullDeduction => "合-全扣",
        }
    }
    /// 第一道闸不过时，这个读法会不会推空重判第一道闸。
    pub(super) fn rejudges_first_gate(self) -> bool {
        matches!(self, GateReading::SerialRejudge)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum ReclaimTiming {
    AfterEffectiveFloor,
    HoldUntilCovered,
    HoldUntilCoveredDouble,
}

impl ReclaimTiming {
    pub(super) fn label(self) -> &'static str {
        match self {
            ReclaimTiming::AfterEffectiveFloor => "G7",
            ReclaimTiming::HoldUntilCovered => "F扣",
            ReclaimTiming::HoldUntilCoveredDouble => "F-双扣",
        }
    }
    pub(super) fn holds_before_release(self) -> bool {
        matches!(
            self,
            ReclaimTiming::HoldUntilCovered | ReclaimTiming::HoldUntilCoveredDouble
        )
    }
    /// F-扣撇 在第一道闸再减一次扣住槽数；F-扣 不减。
    pub(super) fn first_gate_excludes_held(self) -> bool {
        matches!(self, ReclaimTiming::HoldUntilCoveredDouble)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum StopPolicy {
    StopAtTarget,
    FillBudget,
}

impl StopPolicy {
    pub(super) fn label(self) -> &'static str {
        match self {
            StopPolicy::StopAtTarget => "止",
            StopPolicy::FillBudget => "满",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct Arm {
    pub(super) gate: GateReading,
    pub(super) reclaim: ReclaimTiming,
    pub(super) stop: StopPolicy,
}

impl Arm {
    pub(super) fn label(self) -> String {
        format!(
            "{}-{}-{}",
            self.gate.label(),
            self.reclaim.label(),
            self.stop.label()
        )
    }
    pub(super) fn is_real_baseline(self) -> bool {
        self.gate == GateReading::SerialRejudge && self.reclaim == ReclaimTiming::AfterEffectiveFloor
    }
}

#[cfg(test)]
#[path = "registered_constants_and_slot_states_tests.rs"]
mod tests;
