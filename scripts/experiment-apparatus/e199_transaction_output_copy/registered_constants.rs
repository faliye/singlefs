//! 出自被测条款与登记模型常量的各个字节宽与计数、多层树的种类。

use super::*;

// ── 出自被测条款的常量（登记第五节「模型常量」第一份那张表；第七节「出自被测条款本身的」） ──

/// 容器单元字节（D18（块里携带什么信息） 已定项 24 第 2 条「一个 32 KiB 容器」）。
pub(super) const UNIT_BYTES: u64 = 32768;
/// 节点字节（D8（核心索引结构） 已定项 2）。
pub(super) const NODE_BYTES: u64 = 16384;
/// 码 2 头 115 加两个 key 宽 8，条目 120（D18（块里携带什么信息） 已定项 24 第 1 条）。
pub(super) const CODE_TWO_HEADER_BYTES: u64 = 115;
pub(super) const CODE_TWO_KEY_BYTES: u64 = 8;
pub(super) const CODE_TWO_ENTRY_BYTES: u64 = 120;
/// 码 2 节点扇出：⌊(16384 − (115 + 2 × 8)) ÷ 120⌋。
pub(super) const CODE_TWO_FANOUT: u64 = 135;
/// 码 3 单元的记录区起点（D18（块里携带什么信息） 第 336 行 cap(136, 140) = 233 那一式的 136）。
pub(super) const CODE_THREE_RECORD_AREA_START: u64 = 136;
/// 一片墓碑、谱系、退役容器与一片 inode 叶装几条（D18（块里携带什么信息） 已定项 24 第 2、5 条，第 303 行）。
pub(super) const TOMBSTONE_CONTAINER_RECORDS: u64 = 815;
pub(super) const LINEAGE_CONTAINER_RECORDS: u64 = 1359;
pub(super) const RETIREMENT_CONTAINER_RECORDS: u64 = 959;
pub(super) const INODE_LEAF_RECORDS: u64 = 233;
/// 盘上记录宽：墓碑 40、谱系 24、退役 34、inode 140（只用在第七节那四条单元装满的加法断言里）。
pub(super) const TOMBSTONE_RECORD_DISK_BYTES: u64 = 40;
pub(super) const LINEAGE_RECORD_DISK_BYTES: u64 = 24;
pub(super) const RETIREMENT_RECORD_DISK_BYTES: u64 = 34;
pub(super) const INODE_RECORD_DISK_BYTES: u64 = 140;

// ── 取实现值的模型常量（不出自被测条款；登记第五节第二份那张表与第三份加的两行） ──

/// 多层码 2 树的叶容量、内部容量、key 字节与 key 字段数（`transaction.rs:2679` 注释；`code_two_tree.rs:37`、`40`、`44`）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct MultiLevelTreeKind {
    pub(super) leaf_entries: u64,
    pub(super) internal_entries: u64,
    key_bytes: u64,
    key_fields: u64,
}
pub(super) const ACCOUNTING_TREE: MultiLevelTreeKind = MultiLevelTreeKind { leaf_entries: 477, internal_entries: 150, key_bytes: 22, key_fields: 4 };
pub(super) const DEADLIST_TREE: MultiLevelTreeKind = MultiLevelTreeKind { leaf_entries: 376, internal_entries: 125, key_bytes: 43, key_fields: 7 };
pub(super) const CENTRAL_MAPPING_TREE: MultiLevelTreeKind = MultiLevelTreeKind { leaf_entries: 294, internal_entries: 143, key_bytes: 27, key_fields: 5 };
/// 一条映射条目的字节（`transaction.rs` 注释「映射条目 55」）。
pub(super) const MAPPING_ENTRY_BYTES: u64 = 55;
/// 分配记录树：叶罩 812 槽；内部节点头 135、内部条目 96，扇出 169（`singlefs-format/src/lib.rs`）。
pub(super) const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
pub(super) const ALLOCATION_RECORD_TREE_INTERNAL_HEADER_BYTES: u64 = 135;
pub(super) const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES: u64 = 96;
pub(super) const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
/// 记账条目数：池级 3 行、两块盘各 6 行、台阶行 1（`transaction.rs:7915`、`7955` 的 `ALL`）。
const POOL_ACCOUNTING_ROWS: u64 = 3;
const PER_DEVICE_ACCOUNTING_ROWS: u64 = 6;
pub(super) const DEVICE_COUNT: u64 = 2;
const STEP_ACCOUNTING_ROWS: u64 = 1;
pub(super) const ACCOUNTING_ENTRIES: u64 = POOL_ACCOUNTING_ROWS + DEVICE_COUNT * PER_DEVICE_ACCOUNTING_ROWS + STEP_ACCOUNTING_ROWS;
/// 实例表片数（模型选择，一片 32 KiB 单元，照抄）与固定点单元数（16 个 16 KiB 节点，每次发布全部重写）。
pub(super) const INSTANCE_TABLE_UNITS: u64 = 1;
pub(super) const FIXED_POINT_NODES: u64 = 16;
/// 指向码 1 数据单元的指针 88：头部 50 + 位置条目 14 × 2 + 写序 10（`crates/singlefs-core/src/pointer.rs:828` 注释）。
pub(super) const DATA_POINTER_HEADER_BYTES: u64 = 50;
pub(super) const POSITION_ENTRY_BYTES: u64 = 14;
pub(super) const DATA_POINTER_WRITE_ORDER_BYTES: u64 = 10;
pub(super) const DATA_POINTER_BYTES: u64 = 88;

// ── 内存宽度：装置自己的类型，`size_of` 回比（登记第五节；读法第 3 条：不是 `crates/` 里 Rust 类型的真宽） ──

const VECTOR_HEADER_BYTES: u64 = size_of::<Vec<u8>>() as u64;
/// 墓碑记录 5 个 u64；谱系记录 3 个 u64；退役记录 u64 + u16 + 三个 u64（按 8 对齐）；inode 记录 8 个 u64；分配记录 5 个 u64。
pub(super) const TOMBSTONE_RECORD_MEMORY_BYTES: u64 = size_of::<[u64; 5]>() as u64;
pub(super) const LINEAGE_RECORD_MEMORY_BYTES: u64 = size_of::<[u64; 3]>() as u64;
pub(super) const RETIREMENT_RECORD_MEMORY_BYTES: u64 = size_of::<(u64, u16, [u64; 3])>() as u64;
pub(super) const INODE_RECORD_MEMORY_BYTES: u64 = size_of::<[u64; 8]>() as u64;
pub(super) const ALLOCATION_RECORD_MEMORY_BYTES: u64 = size_of::<[u64; 5]>() as u64;
/// 容器外壳 = 身份 32 + `Vec` 头 24；单元外壳 = 槽号 8 + 角色 16 + `Vec` 头 24。
pub(super) const CONTAINER_SHELL_BYTES: u64 = size_of::<([u64; 4], Vec<u8>)>() as u64;
pub(super) const UNIT_SHELL_BYTES: u64 = size_of::<(u64, [u64; 2], Vec<u8>)>() as u64;
/// 内存里一个容器单元、一个节点单元（字节加外壳）。
pub(super) const CONTAINER_UNIT_MEMORY_BYTES: u64 = UNIT_BYTES + UNIT_SHELL_BYTES;
pub(super) const NODE_UNIT_MEMORY_BYTES: u64 = NODE_BYTES + UNIT_SHELL_BYTES;
/// 容器树形状：每个节点 8（子数），每层一个 `Vec` 头 24。
pub(super) const SHAPE_NODE_BYTES: u64 = size_of::<u64>() as u64;
pub(super) const SHAPE_LEVEL_BYTES: u64 = size_of::<Vec<u64>>() as u64;
/// `mapped_units` 一项：身份 16 + `Vec` 头 24 + 映射 key；带位置项一项再加两条位置项。
const UNIT_IDENTITY_BYTES: u64 = size_of::<[u64; 2]>() as u64;
pub(super) const MAPPED_UNIT_ENTRY_BYTES: u64 = UNIT_IDENTITY_BYTES + VECTOR_HEADER_BYTES + CENTRAL_MAPPING_TREE.key_bytes;
pub(super) const MAPPED_UNIT_WITH_LOCATIONS_BYTES: u64 = MAPPED_UNIT_ENTRY_BYTES + 2 * POSITION_ENTRY_BYTES;
/// key 字节一项、叶条目一项（`Vec` 头加字节）。
pub(super) const MAPPING_KEY_BYTES_ENTRY_BYTES: u64 = VECTOR_HEADER_BYTES + CENTRAL_MAPPING_TREE.key_bytes;
pub(super) const MAPPING_LEAF_ENTRY_BYTES: u64 = VECTOR_HEADER_BYTES + MAPPING_ENTRY_BYTES;
/// 多层码 2 树形状：每个节点 = 位置 8 + 孩子表的 `Vec` 头 24；每个孩子 = key 结构 + 位置 8。
pub(super) const MULTI_LEVEL_SHAPE_NODE_BYTES: u64 = size_of::<(u64, Vec<u64>)>() as u64;
pub(super) const MULTI_LEVEL_CHILD_POSITION_BYTES: u64 = size_of::<u64>() as u64;
/// 分配记录树节点身份 16；`deleted_inodes` 一个号 32（`Vec` 头 24 + 号 8）。
pub(super) const ALLOCATION_TREE_NODE_IDENTITY_BYTES: u64 = size_of::<[u64; 2]>() as u64;
pub(super) const DELETED_INODE_NUMBER_BYTES: u64 = VECTOR_HEADER_BYTES + size_of::<u64>() as u64;
/// 第一个文件照抄的数据单元抄进 `data_units` 那一份是裸 `Vec<u8>`：字节加 `Vec` 头，没有单元外壳。
pub(super) const CARRIED_DATA_UNIT_BYTES: u64 = UNIT_BYTES + VECTOR_HEADER_BYTES;
/// 装置自己的数据指针类型 `[u8; 88]`。
pub(super) const DATA_POINTER_MEMORY_BYTES: u64 = size_of::<[u8; 88]>() as u64;
/// 乙1 换成引用计数之后照抄的一个指针（第二份第五节「× 8（只抄指针）」）。
pub(super) const REFERENCE_COUNTED_POINTER_BYTES: u64 = size_of::<std::sync::Arc<[u8; 32768]>>() as u64;

/// key 结构的内存宽 W_ck = 字段表 `Vec` 头 24 + 8 × 字段数 + 字节 `Vec` 头 24 + key 字节（映射 115、记账 102、deadlist 147）。
pub(super) const fn key_structure_bytes(kind: MultiLevelTreeKind) -> u64 {
    size_of::<Vec<u64>>() as u64 + 8 * kind.key_fields + VECTOR_HEADER_BYTES + kind.key_bytes
}
/// 映射条目表一项：key 结构 + 条目 `Vec` 头 + 条目字节（194）。
pub(super) const MAPPING_ENTRY_TABLE_BYTES: u64 = key_structure_bytes(CENTRAL_MAPPING_TREE) + VECTOR_HEADER_BYTES + MAPPING_ENTRY_BYTES;

#[cfg(test)]
#[path = "registered_constants_tests.rs"]
mod tests;
