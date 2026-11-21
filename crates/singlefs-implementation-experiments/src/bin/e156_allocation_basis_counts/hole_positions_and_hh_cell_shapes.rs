//! 岔路 1 的族 Hh(k, 位置, S, ρ)（第 4 次重跑登记第五节 5.3、第一节 1.3 W1–W9）：洞的位置、负载密度、分配器来源与一格的形状。

use super::*;

// ============================================================================================
// 岔路 1 的族 Hh(k, 位置, S, ρ)：第 4 次重跑登记第五节 5.3、第一节 1.3 W1–W9。
// ============================================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum HolePosition {
    Front,
    Back,
}

impl HolePosition {
    pub(super) fn label(self) -> &'static str {
        match self {
            HolePosition::Front => "front",
            HolePosition::Back => "back",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum WorkloadDensity {
    EveryPublishOverwrites,
    OneOverwriteInFourPublishes,
}

impl WorkloadDensity {
    pub(super) fn label(self) -> &'static str {
        match self {
            WorkloadDensity::EveryPublishOverwrites => "1",
            WorkloadDensity::OneOverwriteInFourPublishes => "1/4",
        }
    }
    pub(super) fn publishes_per_overwrite(self) -> u64 {
        match self {
            WorkloadDensity::EveryPublishOverwrites => 1,
            WorkloadDensity::OneOverwriteInFourPublishes => 4,
        }
    }
}

/// W7：分配器从哪来。只有产品路径一种（mkfs 同一个进程里装根环表，之后用每次可写挂载交回的）。第 5 次重跑登记 5.5 E1 撤了
/// 「不装根环表」那一种（C551：没装根环表的分配器在第一次记根就断言，到不了观测点），成员名不改：`name=q1_cell` 行打
/// `allocator={:?}`，改名会让每一格多一处不同。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum HhAllocatorSource {
    ProductPathWithRootRing,
}

pub(super) fn genesis_slot_on_device_zero(
    locations: &[singlefs_core::pointer::LocationEntry; 2],
) -> SlotNumber {
    locations
        .iter()
        .find(|location| location.device == DeviceIdentity(0))
        .map(|location| location.slot)
        .expect("两份位置条目里有一份在盘 0")
}

pub(super) fn allocator_for_hh(
    source: HhAllocatorSource,
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    genesis: &MakeFilesystemOutput,
) -> PoolAllocator {
    match source {
        HhAllocatorSource::ProductPathWithRootRing => {
            allocator_after_make_filesystem(parameters, devices, genesis)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct HhCellShape {
    pub(super) slots_per_region: u64,
    pub(super) density: WorkloadDensity,
    pub(super) hole_count: u64,
    pub(super) position: Option<HolePosition>,
}

impl HhCellShape {
    pub(super) fn position_label(&self) -> &'static str {
        self.position.map_or("-", HolePosition::label)
    }
    pub(super) fn fields(&self) -> String {
        format!(
            "s={} rho={} k={} position={}",
            self.slots_per_region,
            self.density.label(),
            self.hole_count,
            self.position_label()
        )
    }
}
