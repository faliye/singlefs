//! 常量的字节宽、节点头与容量与内部条目宽、inode 容器容量、填充比与树的层数。

// ============================================================
// 一、常量（登记第三节，手抄自 `crates/singlefs-format/src/lib.rs`；数值与第七节 B1 锚点对拍）
// ============================================================

/// 码 2 节点占盘宽度（含头）。
pub(super) const NODE_BYTES: u64 = 16384;
/// 数据单元与码 3 容器占盘宽度（含头）。
pub(super) const UNIT_BYTES: u64 = 32768;
/// journal 记录固定宽度。
pub(super) const RECORD_BYTES: u64 = 4096;
/// 系统配置槽宽度。
pub(super) const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
/// 记录头宽度（`journal_named_items_per_record` 用它反推容量）。
const RECORD_HEADER_BYTES: u64 = 311;
/// 一条点名项的宽度。
const NAMED_ITEM_BYTES: u64 = 56;
/// 两块盘（`D`）。
pub(super) const DEVICE_COUNT: u64 = 2;
/// 主几何下 journal 环安全系数 `F`（D22 已定项 2）。
pub(super) const RING_SAFETY_FACTOR_MAIN: u64 = 3;
/// journal 环默认容量。
pub(super) const RING_DEFAULT_BYTES: u64 = 768 * 1024 * 1024;
/// 挂载时夹过的有效 `T_dirty`（D16 已定项 5）。
pub(super) const EFFECTIVE_DIRTY_DATA_BUDGET_BYTES: u64 = 256 * 1024 * 1024;
/// 主几何 `physical_block_size`。
pub(super) const PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES: u64 = 512;
/// 反向取样点 `physical_block_size`（8.2）。
pub(super) const REVERSE_PHYSICAL_BLOCK_SIZE_BYTES: u64 = 4096;

/// 码 2 节点头宽公式的常量部分：`86 + 2×key宽 + 29`（`anchors.py` 的 `hdr2`）。
const NODE_HEADER_BASE_BYTES: u64 = 86 + 29;
/// 节点指针宽（D19 已定项 8）。
const NODE_POINTER_BYTES: u64 = 86;
/// K11 反向取样点：内部条目再加身份引用 26 字节。
const IDENTITY_REFERENCE_EXTRA_BYTES: u64 = 26;

pub(super) const EXTENT_KEY_BYTES: u64 = 24;
pub(super) const EXTENT_LEAF_ENTRY_BYTES: u64 = 112;
pub(super) const INODE_KEY_BYTES: u64 = 8;
pub(super) const INODE_INTERNAL_ENTRY_BYTES: u64 = 120;
pub(super) const ALLOCATION_KEY_BYTES: u64 = 10;
pub(super) const ALLOCATION_LEAF_ENTRY_BYTES: u64 = 20;
pub(super) const ACCOUNTING_KEY_BYTES: u64 = 22;
pub(super) const ACCOUNTING_LEAF_ENTRY_BYTES: u64 = 34;
pub(super) const MAPPING_KEY_BYTES: u64 = 27;
pub(super) const MAPPING_LEAF_ENTRY_BYTES: u64 = 55;
pub(super) const TREE_TABLE_KEY_BYTES: u64 = 8;
pub(super) const TREE_TABLE_ENTRY_BYTES: u64 = 200;
/// 码 3 容器头宽（`UNIT_BYTES - 136`，独立于 `hdr2` 公式）。
pub(super) const INODE_CONTAINER_HEADER_BYTES: u64 = 136;
pub(super) const INODE_CONTAINER_ENTRY_BYTES: u64 = 140;

/// 记账池级（不带设备维）行数：inode 号水位、待删占用、已承诺预留。
pub(super) const ACCOUNTING_POOL_WIDE_ENTRIES: u64 = 3;
/// 记账每块盘各一行的行数。
pub(super) const ACCOUNTING_ENTRIES_PER_DEVICE: u64 = 6;
/// 树表条目数（主几何，7 棵：extent、inode、alloc、accounting、mapping、mapping 树自己、实例表；
/// 第七节 B1 锚点钉 81 容量、7 条目，不探树表嵌套轴——那是第 2 行 Q2.5，第三段做）。
pub(super) const TREE_TABLE_ENTRY_COUNT: u64 = 7;
/// 树表改根时相邻改动的条目数（extent、inode、分配记录、记账四棵树的根）。
pub(super) const TREE_TABLE_TOUCHED_ROOTS: u64 = 4;
/// 一次发布点名项超过 67 时怎么切（K7 主口径）。
pub(super) const NAMED_ITEMS_PER_RECORD_MAIN: u64 = (RECORD_BYTES - RECORD_HEADER_BYTES) / NAMED_ITEM_BYTES;

/// 码 2 节点头宽：`86 + 2×key宽 + 29`。
pub(super) const fn node_header_bytes(key_bytes: u64) -> u64 {
    NODE_HEADER_BASE_BYTES + 2 * key_bytes
}

/// 一个节点能装的条目数：`(节点宽 − 头宽) / 条目宽`（`anchors.py` 的 `cap`，叶与内部共用同一个公式，
/// 只是喂不同的 (key宽, 条目宽)）。
pub(super) const fn node_capacity(key_bytes: u64, entry_bytes: u64) -> u64 {
    (NODE_BYTES - node_header_bytes(key_bytes)) / entry_bytes
}

/// 内部节点条目宽：分隔 key + 子指针（K3 主几何）；K11 反向取样点再加身份引用 26 字节。
pub(super) const fn internal_entry_bytes(key_bytes: u64, identity_reference: bool) -> u64 {
    key_bytes + NODE_POINTER_BYTES + if identity_reference { IDENTITY_REFERENCE_EXTRA_BYTES } else { 0 }
}

pub(super) fn inode_container_capacity() -> u64 {
    (UNIT_BYTES - INODE_CONTAINER_HEADER_BYTES) / INODE_CONTAINER_ENTRY_BYTES
}

// ============================================================
// 二、K2：树高与各层节点数
// ============================================================

/// 填充率（K4）：主几何 φ=1；反向取样点 φ=0.5。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FillRatio {
    Full,
    Half,
}

impl FillRatio {
    pub(super) fn effective_capacity(self, raw_capacity: u64) -> u64 {
        match self {
            FillRatio::Full => raw_capacity,
            FillRatio::Half => (raw_capacity / 2).max(1),
        }
    }
}

/// 从叶到根每一层的节点数（K2）；`minimum_layers` 强制树至少这么高（inode 树恒 ≥ 2，根恒码 2）。
pub(super) fn tree_layers(entry_count: u64, leaf_capacity: u64, internal_capacity: u64, minimum_layers: usize) -> Vec<u64> {
    assert!(leaf_capacity >= 1, "叶容量必须 ≥ 1，否则树高不收敛");
    assert!(internal_capacity >= 1, "内部容量必须 ≥ 1，否则树高不收敛");
    let mut layers = Vec::new();
    let mut count = entry_count.div_ceil(leaf_capacity).max(1);
    layers.push(count);
    while count > 1 {
        count = count.div_ceil(internal_capacity);
        layers.push(count);
        assert!(layers.len() <= 64, "树高超过 64 层，多半是容量参数错了");
    }
    while layers.len() < minimum_layers {
        layers.push(1);
    }
    layers
}

#[cfg(test)]
#[path = "byte_widths_and_tree_shape_tests.rs"]
mod tests;
