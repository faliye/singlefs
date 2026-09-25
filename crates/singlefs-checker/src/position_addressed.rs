//! 按 key 空间定形状的两棵派生树（分配记录树、extent 树，D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K1 / K2）在 checker 这边的几何：
//! 只与实现共享格式常量模块（D13（验证路线） 已定项 5），几何、key 的字节、位置怎么算按条款另写一份，不用实现的解析。
//!
//! - 分配记录树：叶 k 罩一块盘上的槽 `[k × W, (k + 1) × W)`，层级 L 的节点罩 `W × F^L` 个槽（W = 812、F = 169）；根罩整个 key 空间，
//!   它的层级是最小的 R ≥ 1 使 Σ_盘 ⌈盘上槽数 ÷ W·F^(R−1)⌉ ≤ F。节点头的 key 区间写这个节点按位置规定罩的那一段
//!   （D18（块里携带什么信息） 已定项 2 对按位置寻址的树那一句）；父条目的 key 是孩子那一段的起点。
//! - extent 树：上段按 inode 号的位置（叶罩 143 个 inode 号、内部扇出 147），下段一个文件一棵按数据单元号的位置（叶罩 144 个单元、扇出 147），
//!   上段叶条目 113 带标签 0 / 1 / 2（没有单元 / 下段根指针 / 内联数据指针）。

use singlefs_format::{
    ALLOCATION_RECORD_TREE_INTERNAL_FANOUT, ALLOCATION_RECORD_TREE_LEAF_SLOTS, DATA_POINTER_BYTES,
    DATA_UNIT_BYTES, DATA_UNIT_PAYLOAD_OFFSET, EXTENT_TREE_INTERNAL_FANOUT,
    EXTENT_TREE_LOWER_LEAF_DATA_UNITS, EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES,
    EXTENT_TREE_UPPER_LEAF_INODES, NODE_POINTER_BYTES,
};

use crate::{read_six_byte_unsigned, read_u32, read_u64};

/// 分配记录树 key 里槽号那一段是 6 字节：一块盘上最大的槽号。
const LARGEST_SLOT_NUMBER: u64 = (1 << 48) - 1;

/// 一个数据单元装多少字节用户数据（32768 − 含预留位的头 134）：extent key 的 offset 段是文件字节偏移（D8（核心索引结构） 已定项 3），
/// 第 n 个单元是 n × 它。
#[must_use]
pub fn data_unit_payload_capacity_in_bytes() -> u64 {
    DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET
}

/// 分配记录树层级 L 上一个节点罩几个槽：W × F^L，乘到装不进 u64 就停在最大值。
#[must_use]
pub fn allocation_record_tree_span_in_slots(level: u8) -> u64 {
    (0..level).fold(ALLOCATION_RECORD_TREE_LEAF_SLOTS, |span, _| {
        span.saturating_mul(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT)
    })
}

/// 这个池的分配记录树根该在哪一层（`device_slots` 是池里每块盘的槽数）；盘多到连 255 层都装不下时 `None`。
#[must_use]
pub fn allocation_record_tree_root_level(device_slots: &[u64]) -> Option<u8> {
    (1..=u8::MAX).find(|root_level| {
        let child_span = allocation_record_tree_span_in_slots(root_level - 1);
        device_slots
            .iter()
            .map(|slots| slots.div_ceil(child_span))
            .sum::<u64>()
            <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
    })
}

/// 分配记录树里一个节点的位置：根，或根之下 (层级, 盘, 同盘同层序号)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AllocationRecordTreeCell {
    Root,
    BelowTheRoot { level: u8, device: u32, index: u64 },
}

/// 分配记录树 key（设备 4, 槽号 6，小端）的字节。
#[must_use]
pub fn allocation_record_key(device: u32, slot: u64) -> Vec<u8> {
    let mut key = device.to_le_bytes().to_vec();
    key.extend_from_slice(&slot.to_le_bytes()[..6]);
    key
}

/// 一个分配记录树节点按位置规定罩的那一段，(最小, 最大) 两把 key 的字节。
#[must_use]
pub fn allocation_record_tree_cell_range(cell: AllocationRecordTreeCell) -> (Vec<u8>, Vec<u8>) {
    match cell {
        AllocationRecordTreeCell::Root => (vec![0u8; 10], vec![0xFFu8; 10]),
        AllocationRecordTreeCell::BelowTheRoot {
            level,
            device,
            index,
        } => {
            let span = allocation_record_tree_span_in_slots(level);
            let first = index.saturating_mul(span).min(LARGEST_SLOT_NUMBER);
            let last = index
                .saturating_mul(span)
                .saturating_add(span - 1)
                .min(LARGEST_SLOT_NUMBER);
            (
                allocation_record_key(device, first),
                allocation_record_key(device, last),
            )
        }
    }
}

/// 父节点（层级 `parent_level`；根时 `parent` 是 `Root`）里一条内部条目的 key 指的孩子：槽号落在孩子那一层的格点上、盘在池里；
/// 父节点在根之下时孩子与它同盘、落在它那一段里。对不上交回 `None`。
#[must_use]
pub fn allocation_record_tree_child_of_entry_key(
    parent: AllocationRecordTreeCell,
    parent_level: u8,
    entry_key: &[u8],
    pool_devices: &[u32],
) -> Option<AllocationRecordTreeCell> {
    let child_level = parent_level.checked_sub(1)?;
    let device = read_u32(entry_key, 0);
    let slot = read_six_byte_unsigned(entry_key, 4);
    let child_span = allocation_record_tree_span_in_slots(child_level);
    if !slot.is_multiple_of(child_span) || !pool_devices.contains(&device) {
        return None;
    }
    let index = slot / child_span;
    match parent {
        AllocationRecordTreeCell::Root => {}
        AllocationRecordTreeCell::BelowTheRoot {
            device: parent_device,
            index: parent_index,
            ..
        } => {
            if device != parent_device
                || index / ALLOCATION_RECORD_TREE_INTERNAL_FANOUT != parent_index
            {
                return None;
            }
        }
    }
    Some(AllocationRecordTreeCell::BelowTheRoot {
        level: child_level,
        device,
        index,
    })
}

/// 一条分配记录落不落在这片叶里：同盘、起点槽与末槽都在叶那一段里（D8（核心索引结构） 已定项 14「记录的末槽不越过它所在叶的末槽」）。
#[must_use]
pub fn allocation_record_fits_in_the_leaf(
    leaf: AllocationRecordTreeCell,
    device: u32,
    slot: u64,
    span_slots: u64,
) -> bool {
    let AllocationRecordTreeCell::BelowTheRoot {
        level: 0,
        device: leaf_device,
        index,
    } = leaf
    else {
        return false;
    };
    let first = index.saturating_mul(ALLOCATION_RECORD_TREE_LEAF_SLOTS);
    let last = first.saturating_add(ALLOCATION_RECORD_TREE_LEAF_SLOTS - 1);
    device == leaf_device && span_slots > 0 && slot >= first && slot + span_slots - 1 <= last
}

/// extent 树上段层级 L 上一个节点罩几个 inode 号。
#[must_use]
pub fn extent_upper_span_in_inodes(level: u8) -> u64 {
    (0..level).fold(EXTENT_TREE_UPPER_LEAF_INODES, |span, _| {
        span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
    })
}

/// extent 树下段层级 L 上一个节点罩几个数据单元。
#[must_use]
pub fn extent_lower_span_in_data_units(level: u8) -> u64 {
    (0..level).fold(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, |span, _| {
        span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
    })
}

/// extent key（locality 8, inode 8, offset 8，小端）的字节。
#[must_use]
pub fn extent_key(inode: u64, offset_in_bytes: u64) -> Vec<u8> {
    let mut key = 0u64.to_le_bytes().to_vec();
    key.extend_from_slice(&inode.to_le_bytes());
    key.extend_from_slice(&offset_in_bytes.to_le_bytes());
    key
}

/// 上段 (层级, 序号) 按位置规定罩的那一段：`[(0, 首个 inode, 0), (0, 末个 inode, u64 最大)]`。
#[must_use]
pub fn extent_upper_cell_range(level: u8, index: u64) -> (Vec<u8>, Vec<u8>) {
    let span = extent_upper_span_in_inodes(level);
    let first = index.saturating_mul(span);
    (
        extent_key(first, 0),
        extent_key(first.saturating_add(span - 1), u64::MAX),
    )
}

/// 下段 (层级, 序号) 按位置规定罩的那一段：`[(0, inode, 首个单元 × P), (0, inode, (末个单元 + 1) × P − 1)]`。
#[must_use]
pub fn extent_lower_cell_range(inode: u64, level: u8, index: u64) -> (Vec<u8>, Vec<u8>) {
    let span = extent_lower_span_in_data_units(level);
    let payload = data_unit_payload_capacity_in_bytes();
    let first_unit = index.saturating_mul(span);
    let last_unit = first_unit.saturating_add(span - 1);
    (
        extent_key(inode, first_unit.saturating_mul(payload)),
        extent_key(
            inode,
            last_unit
                .saturating_add(1)
                .saturating_mul(payload)
                .saturating_sub(1),
        ),
    )
}

/// 上段一个内部节点（层级 `parent_level`、序号 `parent_index`）里一条条目的 key 指的孩子 (层级, 序号)：key 是 (0, 首个 inode, 0)、
/// 首个 inode 落在孩子那一层的格点上、在父节点那一段里。对不上交回 `None`。
#[must_use]
pub fn extent_upper_child_of_entry_key(
    parent_level: u8,
    parent_index: u64,
    entry_key: &[u8],
) -> Option<(u8, u64)> {
    let child_level = parent_level.checked_sub(1)?;
    let (locality, first_inode, offset) = (
        read_u64(entry_key, 0),
        read_u64(entry_key, 8),
        read_u64(entry_key, 16),
    );
    let child_span = extent_upper_span_in_inodes(child_level);
    if locality != 0 || offset != 0 || !first_inode.is_multiple_of(child_span) {
        return None;
    }
    let child_index = first_inode / child_span;
    (child_index / EXTENT_TREE_INTERNAL_FANOUT == parent_index)
        .then_some((child_level, child_index))
}

/// 下段一个内部节点里一条条目的 key 指的孩子 (层级, 序号)：key 是 (0, 这个文件, 孩子那一段的起点偏移)。对不上交回 `None`。
#[must_use]
pub fn extent_lower_child_of_entry_key(
    inode: u64,
    parent_level: u8,
    parent_index: u64,
    entry_key: &[u8],
) -> Option<(u8, u64)> {
    let child_level = parent_level.checked_sub(1)?;
    let (locality, key_inode, offset) = (
        read_u64(entry_key, 0),
        read_u64(entry_key, 8),
        read_u64(entry_key, 16),
    );
    let child_offset_span = extent_lower_span_in_data_units(child_level)
        .saturating_mul(data_unit_payload_capacity_in_bytes());
    if locality != 0 || key_inode != inode || !offset.is_multiple_of(child_offset_span) {
        return None;
    }
    let child_index = offset / child_offset_span;
    (child_index / EXTENT_TREE_INTERNAL_FANOUT == parent_index)
        .then_some((child_level, child_index))
}

/// 上段叶条目的样子（按字段表另写一份）：inode 号与标签；标签 1 的载荷是节点指针 86（后 2 字节补零），标签 2 的是数据指针 88。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtentUpperLeafEntryView {
    NoDataUnit {
        inode: u64,
    },
    LowerSegmentRoot {
        inode: u64,
        pointer: Vec<u8>,
    },
    InlineDataUnit {
        inode: u64,
        pointer: Vec<u8>,
    },
    /// 条目窄于 113、标签不是 0 / 1 / 2、key 的 locality 或 offset 段不是 0、字段表写零的字节不是零。
    Malformed,
}

/// 解一条上段叶条目。
#[must_use]
pub fn extent_upper_leaf_entry_view(entry: &[u8]) -> ExtentUpperLeafEntryView {
    let entry_bytes = usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113");
    if entry.len() < entry_bytes || read_u64(entry, 0) != 0 || read_u64(entry, 16) != 0 {
        return ExtentUpperLeafEntryView::Malformed;
    }
    let inode = read_u64(entry, 8);
    let payload = &entry[25..entry_bytes];
    let node_pointer_bytes = usize::try_from(NODE_POINTER_BYTES).expect("86");
    let data_pointer_bytes = usize::try_from(DATA_POINTER_BYTES).expect("88");
    match entry[24] {
        0 if payload.iter().all(|byte| *byte == 0) => {
            ExtentUpperLeafEntryView::NoDataUnit { inode }
        }
        1 if payload[node_pointer_bytes..].iter().all(|byte| *byte == 0) => {
            ExtentUpperLeafEntryView::LowerSegmentRoot {
                inode,
                pointer: payload[..node_pointer_bytes].to_vec(),
            }
        }
        2 => ExtentUpperLeafEntryView::InlineDataUnit {
            inode,
            pointer: payload[..data_pointer_bytes].to_vec(),
        },
        _tag_or_padding_the_field_table_does_not_allow => ExtentUpperLeafEntryView::Malformed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 4 GiB 两盘 3 层、1 TiB 两盘 4 层（判决 S1 的本地腿算术），与实现各算各的。
    #[test]
    fn the_root_level_follows_the_device_sizes() {
        let four_gibibytes = (4u64 << 30) / 16384;
        assert_eq!(
            allocation_record_tree_root_level(&[four_gibibytes, four_gibibytes]),
            Some(2)
        );
        let one_tebibyte = (1u64 << 40) / 16384;
        assert_eq!(
            allocation_record_tree_root_level(&[one_tebibyte, one_tebibyte]),
            Some(3)
        );
    }

    /// 一条跨过叶末槽的记录（起在叶的末槽、跨 2）不落在叶里；起在偶数槽的两槽记录落得进。
    #[test]
    fn a_record_crossing_the_last_slot_of_its_leaf_does_not_fit() {
        let leaf = AllocationRecordTreeCell::BelowTheRoot {
            level: 0,
            device: 0,
            index: 61,
        };
        let last_slot_of_the_leaf = 62 * 812 - 1;
        assert!(!allocation_record_fits_in_the_leaf(
            leaf,
            0,
            last_slot_of_the_leaf,
            2
        ));
        assert!(allocation_record_fits_in_the_leaf(
            leaf,
            0,
            last_slot_of_the_leaf - 1,
            2
        ));
    }
}
