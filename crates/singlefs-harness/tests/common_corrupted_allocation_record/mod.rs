//! 把分配记录树里第一条记录挪到单元区末尾之外、再把整条链的校验和重新封好：造「分配记录越界」的坏盘，给 harness 档的回退用例用。
//! 抄自 checker 档 `singlefs_checker_tier::bad_disk_input` 里的同名函数与它用到的 12 个小工具（D13（验证路线） 已定项 15：宁可多一份，不让 harness 的测试依赖 checker 档）；
//! 两份各自维护，改坏盘造法时两处都要看。
#![allow(dead_code, reason = "几个测试文件各自 mod 进来，只用到其中一部分")]

use singlefs_checker::crc32_castagnoli_table;
use singlefs_format::{
    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE, NONCE_MAC_ALGORITHM_RESERVED_BYTES,
    WIDE_CHECKSUM_BYTES,
};

const ALLOCATION_RECORD_SLOT_OFFSET: usize = 4;
const ALLOCATION_RECORD_SPAN_OFFSET: usize = 10;
/// 码 2 索引节点头里 key 宽那一字节的偏移（D18 已定项 18，2026-09-14 用户定案）。
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
const LOCATION_UNIT_CHECKSUM_OFFSET_IN_ENTRY: usize = 10;
/// 码 2 / 码 3 指针（86 字节）里两条位置条目的偏移，以及条目里整单元校验和那 4 字节在条目内的偏移
/// （D19 已定项 4 / 已定项 11：头部 50 + 位置条目 14 × 2；位置条目 = 设备 4 + 槽号 6 + 整单元校验和 4）。
const POINTER_FIRST_LOCATION_OFFSET: usize = 50;
const POINTER_SECOND_LOCATION_OFFSET: usize = 64;
const ROOT_SELF_CHECKSUM_OFFSET: usize = ROOT_TREE_TABLE_POINTER_OFFSET + 86 + 8 + 8;
/// 根记录（371 字节）里三条指针的偏移：树表紧跟 magic 4 + fsid 16 + flags 4 + 实例代号 4 + txg 8；
/// 自证校验和在 138，实例表与中央映射树根紧跟在它之后（`singlefs-core` 的 `root_record.rs` 同一张表，这里按字段表另写一份）。
const ROOT_TREE_TABLE_POINTER_OFFSET: usize = 4 + 16 + 4 + 4 + 8;
/// 分配记录里跨度那个 u16 的最高位是「已释放」标志（`singlefs-core` 的 `ALLOCATION_RECORD_RELEASED_FLAG`），
/// 低 15 位才是跨度。「跨度越过单元区末尾」那一条坏法把整个字段写成这个数：跨度取低 15 位的最大值、已释放位清掉
/// （还着的记录才会被重建拿去 `mark_allocated`，普查 R6 / R7）。
const SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED: u16 = 0x7FFF;
const TREE_KIND_ALLOCATION_RECORDS: u16 = 3;
/// 树表条目里「树的种类」那个 u16 的偏移。
const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
/// 树表条目（200 字节）里根指针那 86 字节的偏移：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2（D8 已定项 8）。
const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET: usize = 14;
/// 单元与索引节点共同前缀里头校验和字段的偏移（D18 已定项 7 的字段表：magic 4 + 格式版本 2 + 类标签 1 + flags 1 + 声明长度 2）。
const UNIT_HEADER_CHECKSUM_OFFSET: usize = 10;

fn read_u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2 字节"))
}

fn read_six_byte_unsigned_at(bytes: &[u8], offset: usize) -> u64 {
    let mut widened = [0u8; 8];
    widened[..6].copy_from_slice(&bytes[offset..offset + 6]);
    u64::from_le_bytes(widened)
}

fn write_six_byte_unsigned_at(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 6].copy_from_slice(&value.to_le_bytes()[..6]);
}

/// 把一个 32 字节的宽校验和字段按新内容重算：字段自身按 0 参与，低 4 字节写 CRC-32C、其余 28 字节恒 0
/// （`singlefs-format` 的 `WIDE_CHECKSUM_BYTES` 注、checker 的 `checksum_field_holds` 同一条口径）。
fn reseal_wide_checksum_field(bytes: &mut [u8], cover_end: usize, field_offset: usize) {
    let field_width = usize::try_from(WIDE_CHECKSUM_BYTES).expect("32");
    bytes[field_offset..field_offset + field_width].fill(0);
    let digest = crc32_castagnoli_table(&bytes[..cover_end]).to_le_bytes();
    bytes[field_offset..field_offset + digest.len()].copy_from_slice(&digest);
}

fn index_node_key_width(node: &[u8]) -> usize {
    usize::from(node[INDEX_NODE_KEY_WIDTH_OFFSET])
}

/// 码 2 节点明文头的末尾 = 86 + 2 × key 宽（也是载荷 CRC 罩的那一段的起点）。
fn index_node_plaintext_header_end(node: &[u8]) -> usize {
    usize::try_from(INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE).expect("86")
        + 2 * index_node_key_width(node)
}

fn index_node_payload_checksum_offset(node: &[u8]) -> usize {
    index_node_plaintext_header_end(node) - 10
}

/// 一个码 2 节点改完之后把它自己的两道校验和重算：先载荷 CRC（它住在头里），再头校验和（它罩着载荷 CRC 那 4 字节）。
fn reseal_index_node(node: &mut [u8]) {
    let header_end = index_node_plaintext_header_end(node);
    let payload_checksum_offset = index_node_payload_checksum_offset(node);
    let payload_digest = crc32_castagnoli_table(&node[header_end..]).to_le_bytes();
    node[payload_checksum_offset..payload_checksum_offset + payload_digest.len()]
        .copy_from_slice(&payload_digest);
    reseal_wide_checksum_field(node, header_end, UNIT_HEADER_CHECKSUM_OFFSET);
}

/// 把一条指针（86 或 88 字节）两条位置条目里的整单元校验和都改成 `checksum`。
fn write_unit_checksum_into_both_locations(pointer: &mut [u8], checksum: u32) {
    for location_offset in [
        POINTER_FIRST_LOCATION_OFFSET,
        POINTER_SECOND_LOCATION_OFFSET,
    ] {
        let field = location_offset + LOCATION_UNIT_CHECKSUM_OFFSET_IN_ENTRY;
        pointer[field..field + 4].copy_from_slice(&checksum.to_le_bytes());
    }
}

/// 码 2 索引节点头里层级那一字节的偏移（0 = 叶）。
const INDEX_NODE_LEVEL_OFFSET: usize = 50;

fn index_node_is_a_leaf(node: &[u8]) -> bool {
    node[INDEX_NODE_LEVEL_OFFSET] == 0
}

/// 一个码 2 节点条目区的样子：从哪起、几条、每条多宽（都按节点自述的字段读，没判过）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IndexNodeEntryLayout {
    start_in_the_node: usize,
    entry_count: usize,
    entry_width: usize,
}

/// 把「根槽 → 树表 → 分配记录树根 → …… → 最左那片叶」这条链上那片叶的第一条分配记录改成「起点贴着单元区末尾、跨度 32767 槽」
/// （越过单元区末尾，普查 R6 / R7），并把链上的校验和逐道重算：叶两道 → 上一层第 0 条条目子指针里的整单元校验和、那一层两道 →
/// …… → 分配记录树根两道 → 树表条目里那条根指针的整单元校验和 → 树表节点两道 → 根槽里树表指针的整单元校验和 → 根槽的自证校验和。
///
/// **分配记录树按绝对槽号按位置寻址之后（D8（核心索引结构） 已定项 14）**，这条记录先撞上的是叶那一判：起点不在这片叶按位置罩的那一段里
/// （读者报「分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽」）。「跨度越过单元区末尾」那一判只剩罩着盘末尾的那片叶上
/// 的记录够得着（一条落在叶里、末槽越过盘末尾的记录），这条坏法打不到它。
///
/// **字节由调用方从盘上读来、改完自己写回去**：`allocation_tree_nodes_from_the_root` 是从根起、每层沿第 0 条条目往下读到的那一路
/// （根在前、叶在末）。坏盘输入那一路读的是 [`MemoryPool`]，步 4 回退那一路读的是录制设备，两边共用这一份，偏移与重算口径只有这一处
/// （`code-discipline.md`「重复要生成，不许手抄」）。跨度字段整个写成
/// [`SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED`]：跨度取低 15 位的最大值、已释放位跟着清掉
/// （只有还着的记录才会被影子账拿去隔离）。
///
/// 交回「坏在哪」那句话；树表里没有分配记录树、这一路的末一个不是叶、中间有一层没有条目、或者叶里一条记录都没有时交回 `None`。
#[must_use]
pub fn move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal_the_chain(
    root_slot: &mut [u8],
    tree_table_node: &mut [u8],
    allocation_tree_nodes_from_the_root: &mut [Vec<u8>],
    unit_area_end_slot: u64,
) -> Option<String> {
    let leaf = allocation_tree_nodes_from_the_root.last_mut()?;
    let layout = IndexNodeEntryLayout::of(leaf);
    if layout.entry_count == 0 || !index_node_is_a_leaf(leaf) {
        return None;
    }
    let tree_table_layout = IndexNodeEntryLayout::of(tree_table_node);
    let allocation_entry_index = (0..tree_table_layout.entry_count).find(|index| {
        read_u16_at(
            tree_table_node,
            tree_table_layout.offset_of_entry(*index) + TREE_TABLE_ENTRY_KIND_OFFSET,
        ) == TREE_KIND_ALLOCATION_RECORDS
    })?;

    let first_entry = layout.offset_of_entry(0);
    let slot_offset = first_entry + ALLOCATION_RECORD_SLOT_OFFSET;
    let old_slot = read_six_byte_unsigned_at(leaf, slot_offset);
    let last_slot_of_the_unit_area = unit_area_end_slot.checked_sub(1)?;
    write_six_byte_unsigned_at(leaf, slot_offset, last_slot_of_the_unit_area);
    let span_offset = first_entry + ALLOCATION_RECORD_SPAN_OFFSET;
    let old_span_field = read_u16_at(leaf, span_offset);
    leaf[span_offset..span_offset + 2]
        .copy_from_slice(&SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED.to_le_bytes());
    reseal_index_node(leaf);
    // 迭代上界是这一路的节点数；每一轮把下一层的整单元校验和写进这一层第 0 条条目的子指针、再封这一层。
    for level_from_the_root in (0..allocation_tree_nodes_from_the_root.len() - 1).rev() {
        let (above, below) =
            allocation_tree_nodes_from_the_root.split_at_mut(level_from_the_root + 1);
        let node = above.last_mut()?;
        let child_checksum = crc32_castagnoli_table(&below[0]);
        let node_layout = IndexNodeEntryLayout::of(node);
        if node_layout.entry_count == 0 || index_node_is_a_leaf(node) {
            return None;
        }
        let pointer_start = node_layout.offset_of_entry(0) + index_node_key_width(node);
        write_unit_checksum_into_both_locations(
            &mut node[pointer_start..pointer_start + 86],
            child_checksum,
        );
        reseal_index_node(node);
    }
    let allocation_tree_root = allocation_tree_nodes_from_the_root.first()?;

    let root_pointer_start = tree_table_layout.offset_of_entry(allocation_entry_index)
        + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
    let allocation_tree_checksum = crc32_castagnoli_table(allocation_tree_root);
    write_unit_checksum_into_both_locations(
        &mut tree_table_node[root_pointer_start..root_pointer_start + 86],
        allocation_tree_checksum,
    );
    reseal_index_node(tree_table_node);

    let tree_table_checksum = crc32_castagnoli_table(tree_table_node);
    write_unit_checksum_into_both_locations(
        &mut root_slot[ROOT_TREE_TABLE_POINTER_OFFSET..ROOT_TREE_TABLE_POINTER_OFFSET + 86],
        tree_table_checksum,
    );
    let cover_end = root_slot.len();
    reseal_wide_checksum_field(root_slot, cover_end, ROOT_SELF_CHECKSUM_OFFSET);

    Some(format!(
        "分配器：这条根那棵账里第一条分配记录的槽号 {old_slot} → {last_slot_of_the_unit_area}（单元区末尾是 {unit_area_end_slot}）、跨度字段 {old_span_field:#06x} → {SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED:#06x}（跨度 {} → 32767 槽，已释放位清掉）；链上每一道校验和重算",
        old_span_field & SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED
    ))
}

impl IndexNodeEntryLayout {
    fn of(node: &[u8]) -> Self {
        Self {
            start_in_the_node: index_node_entries_start(node),
            entry_count: usize::from(read_u16_at(node, index_node_entry_count_offset(node))),
            entry_width: usize::from(read_u16_at(node, index_node_entry_width_offset(node))),
        }
    }
    fn offset_of_entry(self, index: usize) -> usize {
        self.start_in_the_node + index * self.entry_width
    }
}

fn index_node_entry_width_offset(node: &[u8]) -> usize {
    index_node_plaintext_header_end(node) - 2
}

fn index_node_entry_count_offset(node: &[u8]) -> usize {
    index_node_plaintext_header_end(node) - 4
}

fn index_node_entries_start(node: &[u8]) -> usize {
    index_node_plaintext_header_end(node)
        + usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29")
}
