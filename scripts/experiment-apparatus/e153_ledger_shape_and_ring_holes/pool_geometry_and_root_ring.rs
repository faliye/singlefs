//! 池与根环的常量、设备、根环的区域与槽序换算。

use e7_index_bench::usize_from_u64;
#[cfg(test)]
use super::*;

// ---------------------------------------------------------------------
// 常量（登记第一节「池与根环」、第七节 A1/A3）
// ---------------------------------------------------------------------

/// 单元区起始绝对槽号（`allocator.rs` 706-712 的单测几何）。
pub(super) const UNIT_AREA_START_SLOT: u64 = 50176;
/// 单元区容量：每盘 4 GiB / 16384 − 50176（A3）。
pub(super) const UNIT_AREA_CAPACITY_SLOTS: u64 = 211968;
/// 根环区域数（D22 已定项 2）。
pub(super) const ROOT_RING_REGIONS: u64 = 3;
/// 区域按 txg 落哪块设备：0/1/0（D16 已定项 8）。
const ROOT_RING_REGION_DEVICE: [Device; 3] = [Device::Disk0, Device::Disk1, Device::Disk0];
/// 分配记录条目宽：甲-T1 与基线 20 字节（K2）。
pub(super) const RECORD_WIDTH_BYTES_SINGLE_GENERATION: u64 = 20;
/// 分配记录条目宽：G12（同时存分配代与释放代）28 字节（K2）。
pub(super) const RECORD_WIDTH_BYTES_DUAL_GENERATION: u64 = 28;
/// 分配记录树节点头宽：86 + 2×10 + 29（A1）。
const RECORD_TREE_NODE_HEADER_BYTES: u64 = 86 + 2 * 10 + 29;
/// 单节点能装的 20 字节条目数（A1）。
pub(super) const RECORDS_PER_NODE_SINGLE_GENERATION: u64 = (16384 - RECORD_TREE_NODE_HEADER_BYTES) / RECORD_WIDTH_BYTES_SINGLE_GENERATION;
/// 单节点能装的 28 字节条目数（A1）。
pub(super) const RECORDS_PER_NODE_DUAL_GENERATION: u64 = (16384 - RECORD_TREE_NODE_HEADER_BYTES) / RECORD_WIDTH_BYTES_DUAL_GENERATION;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub(super) enum Device {
    Disk0,
    Disk1,
}

/// 根环槽在哪个区域：`txg mod 3`。
fn root_ring_region(txg: u64) -> u64 {
    txg % ROOT_RING_REGIONS
}

/// 根环槽落哪块设备：区域的设备归属（D16 已定项 8）。
pub(super) fn root_ring_device(txg: u64) -> Device {
    ROOT_RING_REGION_DEVICE[usize_from_u64(root_ring_region(txg))]
}

/// 区内槽号：`(txg div 3) mod S`（D22 已定项 2）。
fn root_ring_slot_in_region(txg: u64, slots_per_region: u64) -> u64 {
    (txg / ROOT_RING_REGIONS) % slots_per_region
}

/// 展平之后的环坐标（0..N），只用于本模型内部索引，不是格式意义上的槽号。
pub(super) fn root_ring_flat_index(txg: u64, slots_per_region: u64) -> u64 {
    root_ring_region(txg) * slots_per_region + root_ring_slot_in_region(txg, slots_per_region)
}

#[cfg(test)]
#[path = "pool_geometry_and_root_ring_tests.rs"]
mod tests;
