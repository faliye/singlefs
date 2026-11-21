use super::*;

// ============================================================================================
// 几何锚点的本地移植（登记 7.2 A30、A31、A33；独立的那一份是 `anchors_e183.py`，命令一）。
// 只用上面的本地常量与条款句：D8 已定项 14（分配记录树根层级）、D28 已定项 1（W 与环窗口）、已定项 3（链重写）。不读 `crates/` 的几何函数。
// ============================================================================================

/// 一块盘上单元区的头一槽与末一槽：单元区起点 = journal 环末尾的下一个槽，末一槽 = 盘的槽数 − 1。
pub(super) fn unit_area_first_and_last_slot(
    device_bytes: u64,
    journal_ring_bytes: u64,
) -> (u64, u64) {
    (
        JOURNAL_RING_START_SLOT + journal_ring_bytes / SLOT_BYTES,
        device_bytes / SLOT_BYTES - 1,
    )
}

pub(super) fn unit_area_slots(device_bytes: u64, journal_ring_bytes: u64) -> u64 {
    let (first_slot, last_slot) = unit_area_first_and_last_slot(device_bytes, journal_ring_bytes);
    last_slot - first_slot + 1
}

/// 一片叶之上第 `level` 层一个位置罩多少槽：812 × 169^level。
fn slots_covered_by_one_position_at_level(level: u32) -> u64 {
    ALLOCATION_RECORD_TREE_LEAF_SLOTS * ALLOCATION_RECORD_TREE_INTERNAL_FANOUT.pow(level)
}

/// 分配记录树根层级：最小的 R ≥ 1 使两块盘合计的 ⌈盘的槽数 ÷ (812 × 169^(R−1))⌉ 不超过 169（D8 已定项 14；盘的槽数是绝对槽数）。
fn allocation_record_tree_root_level(device_bytes: u64) -> u32 {
    let device_slots = device_bytes / SLOT_BYTES;
    let mut root_level = 1u32;
    while POOL_DEVICE_COUNT
        * device_slots.div_ceil(slots_covered_by_one_position_at_level(root_level - 1))
        > ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
    {
        root_level += 1;
    }
    root_level
}

/// N_ℓ：第 ℓ 层（0 … 根层级 − 1）两块盘合计罩到的位置数，每块盘按单元区首末槽 ⌊末 ÷ 跨度⌋ − ⌊首 ÷ 跨度⌋ + 1。
pub(super) fn positions_at_each_level(device_bytes: u64, journal_ring_bytes: u64) -> Vec<u64> {
    let (first_slot, last_slot) = unit_area_first_and_last_slot(device_bytes, journal_ring_bytes);
    (0..allocation_record_tree_root_level(device_bytes))
        .map(|level| {
            let span = slots_covered_by_one_position_at_level(level);
            POOL_DEVICE_COUNT * (last_slot / span - first_slot / span + 1)
        })
        .collect()
}

/// 一次删除类发布每盘新占槽数的上界 W = 24 + (6 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26))，K = 144（D28 已定项 1「环窗口」一条）。
fn slots_one_deletion_publish_takes(
    positions: &[u64],
    retirement_tree_internal_levels: u64,
) -> u64 {
    let bound_of_one_level = 2 * (DELETION_BATCH_DATA_UNITS + 26);
    24 + 6
        + retirement_tree_internal_levels
        + positions
            .iter()
            .map(|count| (*count).min(bound_of_one_level))
            .sum::<u64>()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RingWindowByTheClauses {
    pub(super) unit_area_first_slot: u64,
    pub(super) unit_area_slots: u64,
    pub(super) root_level: u32,
    pub(super) positions_at_each_level: Vec<u64>,
    pub(super) deletion_publish_slots: u64,
    pub(super) ring_window_slots: u64,
}

impl RingWindowByTheClauses {
    pub(super) fn ring_window_bytes(&self) -> u64 {
        self.ring_window_slots * SLOT_BYTES
    }
}

/// 环窗口 = (3S + 1) × W 个槽（D28 已定项 1「环窗口」一条）。
pub(super) fn ring_window_by_the_clauses(
    device_bytes: u64,
    journal_ring_bytes: u64,
    slots_per_region: u64,
    retirement_tree_internal_levels: u64,
) -> RingWindowByTheClauses {
    let positions = positions_at_each_level(device_bytes, journal_ring_bytes);
    let deletion_publish_slots =
        slots_one_deletion_publish_takes(&positions, retirement_tree_internal_levels);
    let (first_slot, _) = unit_area_first_and_last_slot(device_bytes, journal_ring_bytes);
    RingWindowByTheClauses {
        unit_area_first_slot: first_slot,
        unit_area_slots: unit_area_slots(device_bytes, journal_ring_bytes),
        root_level: allocation_record_tree_root_level(device_bytes),
        deletion_publish_slots,
        ring_window_slots: (ROOT_RING_REGIONS * slots_per_region + 1) * deletion_publish_slots,
        positions_at_each_level: positions,
    }
}

/// 实例切换预留里链重写那一半，每块盘：(N_switch + 1) × 32768 × max(1, ⌈(rows0 + N_switch) ÷ 369⌉)（D28 已定项 3）。
pub(super) fn instance_table_chain_pages(instance_rows_of_the_switch_reserve: u64) -> u64 {
    (instance_rows_of_the_switch_reserve + INSTANCE_SWITCHES_ALLOWED_PER_MOUNT)
        .div_ceil(INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE)
        .max(1)
}

pub(super) fn chain_rewrite_bytes_of_the_switch_reserve(
    instance_rows_of_the_switch_reserve: u64,
) -> u64 {
    (INSTANCE_SWITCHES_ALLOWED_PER_MOUNT + 1)
        * INSTANCE_TABLE_PAGE_BYTES
        * instance_table_chain_pages(instance_rows_of_the_switch_reserve)
}

#[cfg(test)]
mod tests;
