//! 几份测试共用的辅助：登记的基线几何与真实基线的那条臂。

use super::*;

pub(super) fn baseline_geometry() -> Geometry {
    Geometry {
        capacity_slots_per_disk: 600,
        object_units: 1,
        fixpoint_cost: 4,
        slots_per_region: 4,
        padding_deletes: 0,
    }
}

pub(super) fn real_baseline_arm() -> Arm {
    Arm { gate: GateReading::SerialRejudge, reclaim: ReclaimTiming::AfterEffectiveFloor, stop: StopPolicy::StopAtTarget }
}
