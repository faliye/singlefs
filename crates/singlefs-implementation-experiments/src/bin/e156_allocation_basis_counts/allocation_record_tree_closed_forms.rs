//! R-3（E156 第 3 次重跑登记「七」7.2）：分配记录树按位置寻址之后，一次发布新写与替换的节点数的闭式。

use super::*;

// ============================================================================================
// R-3（E156 第 3 次重跑登记「七」7.2）：分配记录树按位置寻址之后，节点数不再是常数，只能从「这次发布
// 实际改动的记录的槽号」现算。这一段只用 [`E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS`] /
// [`E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`] 两个本地常量与 `allocator.records()` 的真实读数，
// 不调用 `crates/singlefs-core::allocation_record_tree` 的任何几何函数——那些函数就是被测的实装本身，
// 拿它们来算「期望值」会让期望值与实装共用同一处错误。
// ============================================================================================

/// 一个槽所在的分配记录树叶位置 ⌊s ÷ W⌋（本地常量 W）。
#[cfg(test)]
fn e156_leaf_position_of_slot(slot: u64) -> u64 {
    slot / E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS
}

/// 一个叶位置所在的层级 1 位置 ⌊k ÷ F⌋（本地常量 F）。
#[cfg(test)]
fn e156_level1_position_of_leaf(leaf: u64) -> u64 {
    leaf / E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
}

/// R-3：一次发布里分配记录树自身新写的节点数 = `devices × 叶数 + devices × 层级 1 数 + 1`（根只有一份，
/// 两块盘共享；叶与层级 1 逐盘各一份，登记「七」7.2 R-3 逐字）。
#[cfg(test)]
fn e156_allocation_record_tree_new_node_count(touched_leaves: &BTreeSet<u64>, devices: u64) -> u64 {
    let touched_level1: BTreeSet<u64> = touched_leaves
        .iter()
        .map(|&leaf| e156_level1_position_of_leaf(leaf))
        .collect();
    devices * u64::try_from(touched_leaves.len()).expect("叶数落在 u64 内")
        + devices * u64::try_from(touched_level1.len()).expect("层级 1 数落在 u64 内")
        + 1
}

/// R-3：这次发布里被换下（COW 释放）的分配记录树旧节点数——只有这次触达、且这个位置在这次发布之前
/// 已经有节点（叶位置 ∈ `existing_leaves`，或它的层级 1 位置 ∈ 现有层级 1 集合）的那些才会释放旧版本。
#[cfg(test)]
fn e156_allocation_record_tree_replaced_node_count(
    touched_leaves: &BTreeSet<u64>,
    existing_leaves: &BTreeSet<u64>,
    devices: u64,
) -> u64 {
    let touched_level1: BTreeSet<u64> = touched_leaves
        .iter()
        .map(|&leaf| e156_level1_position_of_leaf(leaf))
        .collect();
    let existing_level1: BTreeSet<u64> = existing_leaves
        .iter()
        .map(|&leaf| e156_level1_position_of_leaf(leaf))
        .collect();
    let leaves_kept = touched_leaves.intersection(existing_leaves).count();
    let level1_kept = touched_level1.intersection(&existing_level1).count();
    devices * u64::try_from(leaves_kept).expect("叶交集数落在 u64 内")
        + devices * u64::try_from(level1_kept).expect("层级 1 交集数落在 u64 内")
        + 1
}

/// R-3：D0 上这次发布之前，全部分配记录（不论已释放还是仍分配）所在的叶位置集合——「这个位置本来
/// 有没有节点」的判据（D8（核心索引结构） 已定项 14「没有记录的一段 = 全空闲：那片叶不写」）。
#[cfg(test)]
fn e156_existing_leaf_positions(
    allocator: &PoolAllocator,
    device: DeviceIdentity,
) -> BTreeSet<u64> {
    allocator
        .records()
        .iter()
        .filter(|record| record.device == device)
        .map(|record| e156_leaf_position_of_slot(record.slot.0))
        .collect()
}

/// R-3：D0 上这次发布翻成已释放或新加的全部记录所在的叶位置集合——真实记录的 `generation` 字段
/// 在被触达的那一刻（无论新分配还是刚被释放）都改写成这次的 txg（D3（空间分配） 已定项 7 逐字），
/// 用它筛出「这次改动的记录」不需要另外做前后快照 diff。
#[cfg(test)]
fn e156_touched_leaf_positions(
    allocator: &PoolAllocator,
    device: DeviceIdentity,
    txg: CheckpointTxg,
) -> BTreeSet<u64> {
    allocator
        .records()
        .iter()
        .filter(|record| record.device == device && record.generation == txg)
        .map(|record| e156_leaf_position_of_slot(record.slot.0))
        .collect()
}

/// R-3：一次发布（覆盖写或空发布）D0 释放槽数与 D0+D1 记录增量的闭式期望值。
/// `existing_leaves`：这次发布之前 D0 上已有记录的叶位置集合；`touched_leaves`：这次发布之后
/// D0 上 `generation == 本次 txg` 的记录所在的叶位置集合。
#[cfg(test)]
fn e156_closed_form_expected(
    existing_leaves: &BTreeSet<u64>,
    touched_leaves: &BTreeSet<u64>,
    is_overwrite: bool,
) -> (u64, u64) {
    let new_nodes = e156_allocation_record_tree_new_node_count(touched_leaves, 2);
    let replaced_nodes =
        e156_allocation_record_tree_replaced_node_count(touched_leaves, existing_leaves, 2);
    let (other_released, other_new_records) = if is_overwrite {
        (
            E156_OVERWRITE_OTHER_UNITS_RELEASED_SLOTS,
            E156_OVERWRITE_OTHER_UNITS_NEW_RECORD_SLOTS,
        )
    } else {
        (
            E156_EMPTY_PUBLISH_OTHER_UNITS_SLOTS,
            E156_EMPTY_PUBLISH_OTHER_UNITS_SLOTS,
        )
    };
    (
        other_released + replaced_nodes,
        2 * (other_new_records + new_nodes),
    )
}

/// R-3 本地常量③：根层级规则——最小的 R ≥ 1 使 Σ_盘 ⌈盘上槽数 ÷ (W × F^(R−1))⌉ ≤ F（D8（核心索引结构）
/// 已定项 14「根罩整个 key 空间、按盘分流」那一行）。只用来与实装的 `AllocationRecordTreeGeometry`
/// 回比（A-D8，只观测，F21：对不上不作废、不停机），不供 R-3 的节点数闭式使用。
pub(super) fn e156_root_level_for_symmetric_devices(
    unit_area_slots_per_device: u64,
    device_count: u64,
) -> u8 {
    let mut level: u32 = 1;
    loop {
        let child_span = E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS
            * E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT.pow(level - 1);
        let total_cells = device_count * unit_area_slots_per_device.div_ceil(child_span);
        if total_cells <= E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT {
            return u8::try_from(level).expect("根层级落在 u8 内（池子不会大到需要 256 层）");
        }
        level += 1;
    }
}

/// Q7d-1（R2 ⑤）：一组「这次发布自己的释放」样本的 (最小值, 最大值)——U12 单测用它，M30（Q7d-1 覆盖写那一组退回字面 10）改这里。
#[cfg(test)]
fn e156_minimum_and_maximum(samples: &[u64]) -> (u64, u64) {
    (
        samples.iter().min().copied().unwrap_or(0),
        samples.iter().max().copied().unwrap_or(0),
    )
}

#[cfg(test)]
mod tests;
