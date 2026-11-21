//! 全部臂与全部几何、实例行与根记录、按 txg 取根环区域与设备。

use super::*;

pub(super) fn all_arms() -> Vec<Arm> {
    let mut arms = Vec::new();
    for gate in [
        GateReading::Serial,
        GateReading::SerialRejudge,
        GateReading::Union,
        GateReading::UnionFullDeduction,
    ] {
        for reclaim in [
            ReclaimTiming::AfterEffectiveFloor,
            ReclaimTiming::HoldUntilCovered,
            ReclaimTiming::HoldUntilCoveredDouble,
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
pub(super) struct Geometry {
    pub(super) capacity_slots_per_disk: u64,
    pub(super) object_units: u64,
    pub(super) fixpoint_cost: u64,
    pub(super) slots_per_region: u64,
    pub(super) padding_deletes: u64,
}

pub(super) fn all_geometries() -> Vec<Geometry> {
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
pub(super) struct InstanceRow {
    pub(super) instance: u64,
    pub(super) selected_root_txg: u64,
    pub(super) applied_transaction_high_water: u64,
    pub(super) is_rollback: bool,
}

/// 有两个字段专供 H6（管理员回退）用：回退要把
/// `pool.current_fixpoint_slots` / `pool.current_instance_table_slots` 也载回 R_old 那一刻的值，
/// 不能靠"槽状态快照里最低的几个 Allocated 槽"去猜——那是个只在"分配器总从最低空槽发"这条
/// 不变量下才成立的假设，猜错了下一次发布释放的就不是 R_old 真正持有的那几个槽。
/// H4／H5 的崩溃-恢复分支仍不实现（见文件头「输出体量说明」旁的范围记录），对象表快照
/// 仍然留给以后接上那两类历史时再加。
#[derive(Clone)]
pub(super) struct RootRecord {
    pub(super) txg: u64,
    pub(super) instance: u64,
    pub(super) rollback_floor: u64,
    pub(super) non_empty: bool,
    pub(super) occupied_pub: u64,
    pub(super) slots_snapshot: Rc<Vec<SlotState>>,
    pub(super) fixpoint_slots_snapshot: Rc<Vec<usize>>,
    pub(super) instance_table_slots_snapshot: Rc<Vec<usize>>,
}

pub(super) fn region_of_txg(txg: u64) -> u64 {
    txg % RING_REGIONS
}

pub(super) fn device_of_txg(txg: u64) -> u64 {
    REGION_DEVICES[usize::try_from(region_of_txg(txg)).expect("区域号")]
}

#[cfg(test)]
#[path = "arms_geometries_and_root_records_tests.rs"]
mod tests;
