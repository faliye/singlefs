//! 池级 checker：从根环里每一条自证过的根走下去（遍历方向），再扫一遍单元头与 journal 环（扫描方向），判第一版那几条不变量
//! （清单与条数在 `image::IMPLEMENTED_INVARIANTS`，别在这里另抄一个数）。
//! 最新的那条根另判 I-7.2（能不能完整走完）；引用集合按全部有效根取并集——根环里更早的根仍是可回退的状态，
//! 它们引用的单元仍占着空间（I-3.1 与 I-5.1 的这一读法 2026-09-14 用户收尾弹窗定甲，见 records 2026-09-13-总审核 十一·七）。

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, ALLOCATION_RECORD_BYTES, ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES,
    CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES,
    EXTENT_TREE_INTERNAL_ENTRY_BYTES, EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, INODE_INTERNAL_ENTRY,
    JOURNAL_RECORD_BYTES, JOURNAL_SAFETY_FACTOR, LINEAGE_CONTAINER_RECORDS, MAPPING_ENTRY_BYTES,
    NODE_BYTES, NODE_POINTER_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES,
    RETIREMENT_CONTAINER_RECORDS, SLOT_BYTES, TOMBSTONE_CONTAINER_RECORDS,
    TREE_IDENTIFIER_WATERMARK_AT_MKFS, TREE_TABLE_ENTRY_BYTES,
};

use crate::position_addressed::{
    allocation_record_fits_in_the_leaf, allocation_record_tree_cell_range,
    allocation_record_tree_child_of_entry_key, allocation_record_tree_root_level,
    data_unit_payload_capacity_in_bytes, extent_lower_cell_range, extent_lower_child_of_entry_key,
    extent_lower_span_in_data_units, extent_upper_cell_range, extent_upper_child_of_entry_key,
    extent_upper_leaf_entry_view, extent_upper_span_in_inodes, AllocationRecordTreeCell,
    ExtentUpperLeafEntryView,
};

use crate::image::{
    chosen_system_configurations, decision_width_declared_by_the_chosen_system_configurations,
    judge_journal_ring_geometry_fields_agree, judge_location_entries_order, judge_location_order,
    judge_own_device_number_is_the_device_identity,
    judge_pointer_encryption_and_compression_fields_take_the_first_version_values,
    judge_pointer_extent_offset_is_zero, judge_pointer_mac_and_nonce_are_zero,
    judge_root_record_algorithm_nonce_and_mac_are_zero,
    judge_root_records_live_in_the_slot_their_txg_names,
    judge_root_slots_aligned_to_the_decision_width,
    judge_some_device_carries_a_self_verified_system_configuration,
    judge_system_configuration_slot_generations_follow_the_rotation,
    judge_system_configuration_values_the_reader_accepts, parse_data_pointer, parse_node_pointer,
    read_referenced_unit, root_slot_positions, system_configuration_slot_readings, valid_roots,
    verified_system_configuration_slots, ImageReader, InvariantVerdict, Judgements,
    PointerLocation, PointerView, PoolGeometry,
    ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
    FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
    HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
    INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
    INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
    INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
    JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
    LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
    LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
    LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER, MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
    NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
    NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    ONE_OBJECT_BIRTH_PER_INODE_NUMBER, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
    POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER, PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
    QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE, REFERENCE_INTERVALS_ARE_EXACT,
    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
    TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
    TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
    UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
};
use crate::{
    back_chain_of_record_header, check_index_node_keys, check_internal_node_separators,
    check_journal_record, check_unit, checksum_field_holds, crc32_castagnoli_table,
    index_node_view, key_schema_for_tree_kind, packed_unit_view, read_six_byte_unsigned, read_u16,
    read_u32, read_u64, NamedEntryView, Verdict, FORMAT_VERSION_OFFSET, KEY_SCHEMA_ACCOUNTING,
    KEY_SCHEMA_ALLOCATION, KEY_SCHEMA_CONTAINER_NUMBER, KEY_SCHEMA_EXTENT, KEY_SCHEMA_INODE,
    KEY_SCHEMA_MAPPING, KEY_SCHEMA_TREE_TABLE, UNIT_FORMAT_VERSION_THIS_CHECKER_READS,
};

const TREE_KIND_EXTENT: u16 = 1;
const TREE_KIND_INODE: u16 = 2;
const TREE_KIND_ALLOCATION: u16 = 3;
const TREE_KIND_ACCOUNTING: u16 = 4;
const TREE_KIND_LIVELIST: u16 = 6;
const TREE_KIND_SPARSE_SIDE_TABLE: u16 = 7;
const TREE_KIND_DEADLIST: u16 = 8;
/// 第一个文件版本那次零根注册的四棵树（D8（核心索引结构） 已定项 16 第 3 条、D18（块里携带什么信息） 已定项 24 第 1 / 5 / 6 条）的种类码，
/// 是 D8（核心索引结构） 已定项 9 登记表的码 9..12（只追加不重用码，接在 8 之后），checker 按自己读的条款另写一份。
const TREE_KIND_DELETED_INODES: u16 = 9;
const TREE_KIND_TOMBSTONE: u16 = 10;
const TREE_KIND_LINEAGE: u16 = 11;
const TREE_KIND_LOGGED_OPERATIONS: u16 = 12;
/// 退役树：码 13（D8（核心索引结构） 已定项 9 登记表，接在 logged_ops 之后追加；判决 `research/prompts/m3-retire-r1-main-verification.md`
/// 第三节第 1 条），checker 按自己读的条款另写一份。
const TREE_KIND_RETIREMENT: u16 = 13;
/// 十二棵进树表的树的种类，按发号次序（D8（核心索引结构） 已定项 8 ②「十三棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65548」：
/// extent、inode、分配记录、记账、中央映射、livelist、稀疏旁表、deadlist，再接 D18（块里携带什么信息） 已定项 24 第 6 条列的
/// `deleted_inodes`、墓碑树、谱系树、logged_ops，最后是退役树；中央映射树不进树表，这里没有它）。
const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 12] = [
    TREE_KIND_EXTENT,
    TREE_KIND_INODE,
    TREE_KIND_ALLOCATION,
    TREE_KIND_ACCOUNTING,
    TREE_KIND_LIVELIST,
    TREE_KIND_SPARSE_SIDE_TABLE,
    TREE_KIND_DEADLIST,
    TREE_KIND_DELETED_INODES,
    TREE_KIND_TOMBSTONE,
    TREE_KIND_LINEAGE,
    TREE_KIND_LOGGED_OPERATIONS,
    TREE_KIND_RETIREMENT,
];
const PACKED_TYPE_TOMBSTONE: u16 = 1;
/// 退役记录的打包记录类型（D18（块里携带什么信息） 已定项 11 登记的 3；记录宽 34，判决 m3-retire-r1 第三节第 2 条）。
const PACKED_TYPE_RETIREMENT: u16 = 3;
const PACKED_TYPE_INODE: u16 = 2;
/// 谱系记录的打包记录类型（D18（块里携带什么信息） 已定项 24 第 5 条，不复用登记过的 3）。
const PACKED_TYPE_LINEAGE: u16 = 5;
/// `deleted_inodes` 根的条目：key = inode 号 8 字节、无 value（D8（核心索引结构） 已定项 16）。
const DELETED_INODES_ENTRY_BYTES: usize = 8;
const PACKED_TYPE_INSTANCE_TABLE: u16 = 4;
/// inode 树内部条目的类型段 0：子单元是码 2 的 inode 树内部节点（I-9.2（条目身份与子头相符）「类型段 ∈ {0, 2}」）。
const INODE_ENTRY_TYPE_INTERNAL_NODE: u16 = 0;

/// inode 树内部条目的类型段（D8（核心索引结构） 已定项 6，I-9.2（条目身份与子头相符））：盘上读来的 2 字节，语义上允许未知值，
/// 做成显式成员（`code-discipline.md`「分支：穷举写全，不写通配臂」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InodeInternalEntryType {
    /// 类型段 0：子单元是码 2 的内部节点，身份引用的出生树、容器号、出生代三段为 0。
    InternalNode,
    /// 类型段 2：子单元是码 3、打包记录类型 2 的叶容器。
    LeafContainer,
    /// 条款之外的值：I-9.2 判该条目损坏，子单元不往下走。
    Unregistered(u16),
}

impl InodeInternalEntryType {
    const fn of(code: u16) -> Self {
        match code {
            INODE_ENTRY_TYPE_INTERNAL_NODE => Self::InternalNode,
            PACKED_TYPE_INODE => Self::LeafContainer,
            unregistered => Self::Unregistered(unregistered),
        }
    }
}

/// inode 树内部条目里条款认得的那两种类型段（[`InodeInternalEntryType`] 去掉 `Unregistered`）：类型段不认得的条目在读孩子之前就判完、
/// 不往下读（`Walk::walk_inode_internal_node`），往下读孩子的那几步只见这两种。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegisteredInodeInternalEntryType {
    /// 类型段 0：子单元是码 2 的内部节点。
    InternalNode,
    /// 类型段 2：子单元是码 3、打包记录类型 2 的叶容器。
    LeafContainer,
}

/// 墓碑树、谱系树内部条目的类型段 0：子单元是下一层码 2 节点（根长高之后，D18（块里携带什么信息） 已定项 24 第 1、5 条）。
const CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE: u16 = 0;

/// 墓碑树、谱系树内部条目的类型段（I-9.23（墓碑树条目身份与子头相符） 类型段 ∈ {0, 1}、I-9.17（谱系树条目身份与子头相符） ∈ {0, 5}）：
/// 盘上读来的 2 字节，未知值做成显式成员。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContainerTreeEntryType {
    /// 类型段 0：子单元是码 2 节点，身份引用的出生树、容器号、出生代三段为 0。
    InternalNode,
    /// 类型段等于这棵树的打包记录类型（墓碑 1、谱系 5）：子单元是码 3 容器。
    Container,
    /// 条款之外的值：I-9.17 / I-9.23 判该条目损坏，子单元不往下走。
    Unregistered(u16),
}

impl ContainerTreeEntryType {
    fn of(code: u16, record_type_of_the_tree: u16) -> Self {
        match code {
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE => Self::InternalNode,
            container_type if container_type == record_type_of_the_tree => Self::Container,
            unregistered => Self::Unregistered(unregistered),
        }
    }
}

/// 叶序里的一片 inode 叶容器：容器号与它装的最小、最大 inode 号（I-9.4（容器号不超最小 key） 第二、三句沿叶序逐对比）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InodeLeafContainerInLeafOrder {
    container: u64,
    smallest_inode: u64,
    largest_inode: u64,
}

/// 走一个 inode 树的孩子（叶容器，或类型段 0 指着的内部节点）交回的东西：它下面按叶序排的叶容器，以及这一遍是不是第一次走到它。
/// I-9.12（分隔 key 落在孩子区间之外） 只判这一遍第一次走到的孩子（别的版本走过的那时已按它们自己的父条目判过）；
/// I-9.4 的叶序连别的版本已经走过的一起排（缓存的，不重读盘）。
struct InodeSubtreeInTheWalk {
    leaves_in_leaf_order: Vec<InodeLeafContainerInLeafOrder>,
    walked_first_by_this_version: bool,
}
/// 打包记录类型登记的记录宽：类型 2 是 140（D8（核心索引结构） 已定项 6），类型 4 是 88（实例表行）。
const fn registered_record_width(record_type: u16) -> Option<usize> {
    match record_type {
        PACKED_TYPE_INODE => Some(140),
        PACKED_TYPE_INSTANCE_TABLE => Some(88),
        // 墓碑记录 40、谱系记录 24（D18（块里携带什么信息） 已定项 24 第 2、5 条）。
        PACKED_TYPE_TOMBSTONE => Some(40),
        PACKED_TYPE_LINEAGE => Some(24),
        // 退役记录 34（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2 条）。
        PACKED_TYPE_RETIREMENT => Some(34),
        _ => None,
    }
}
const STATISTIC_ALLOCATED_BYTES: u16 = 1;
const STATISTIC_FREE_BYTES: u16 = 2;
/// defer 队列待释放（D5（快照 / 空间记账机制） 已定项 4 第 5 项的行号，带设备维，单位字节）：I-3.11（已分配减 defer 等于最新根走读） 读它。
const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;
/// inode 号水位（D5（快照 / 空间记账机制） 已定项 4 第 12 项的行号）。
const STATISTIC_INODE_WATERMARK: u16 = 12;
/// 待删除但意图未完成占用（D5（快照 / 空间记账机制） 已定项 4 第 4 项的行号，不带设备维）：恒 0 的保留位，I-3.4（可用空间扣待删占用） 判它。
const STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES: u16 = 4;
/// 记账行里值那 8 字节的偏移：key 22（统计量 2 + 树 8 + 设备 4 + 代 8）之后（D8（核心索引结构） 已定项 11 的记账条目 34）。
const ACCOUNTING_ROW_VALUE_OFFSET: usize = 22;
/// 记账行里设备段的偏移（统计量 2 + 树 8 之后）。
const ACCOUNTING_ROW_DEVICE_OFFSET: usize = 2 + 8;
/// 不带设备维的统计量把设备段写成这个保留值（D5（快照 / 空间记账机制） 已定项 10）。
const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;
/// inode 记录里 nlink 的偏移（D8（核心索引结构） 已定项 6 字段表「24 mode / uid / gid / nlink，各 4」，4 字节；
/// `.claude/kb/layout/03-third-txn.md`「inode 记录的 nlink 第一次为 0」那一行）。
const INODE_RECORD_NLINK_OFFSET: usize = 36;
/// 墓碑记录里对象 ID 的偏移（`.claude/kb/layout/03-third-txn.md`「二、墓碑记录」：偏移 0、8 字节，被删的 inode 号）。
const TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET: usize = 0;
/// 码 3 单元头里写序的实例代号（D18（块里携带什么信息） 已定项 11 的类身份段表：写序 93 起，实例代号 4 字节在前）。
const PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET: usize = 93;
/// inode 记录 140 里 `size` 与 `blocks` 两个字段的偏移（D8（核心索引结构） 已定项 6 的偏移表），按字段表另写一份、不用实现的解析。
const INODE_RECORD_SIZE_OFFSET: usize = 40;
/// inode 记录里对象出生代（偏移 8）与改动计数（偏移 88）的位置（D8（核心索引结构） 已定项 6 的字段表）：I-9.26 比这两个与容器诞生代号。
const INODE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;
const INODE_RECORD_CHANGE_COUNT_OFFSET: usize = 88;
/// 码 3 单元头里的诞生代号（D18（块里携带什么信息） 已定项 11 的类身份段表：偏移 73）。
const PACKED_UNIT_BIRTH_TXG_OFFSET: usize = 73;
/// 记账 key（统计量 2 + 8 + 设备 4 + 代 8，`KEY_SCHEMA_ACCOUNTING`）里代段的偏移：I-3.13 拿它比这一版的 txg。
const ACCOUNTING_KEY_GENERATION_OFFSET: usize = 2 + 8 + 4;
/// 树表条目里 `previous_snapshot_txg` 的偏移（树 ID 8 + 种类 2 + 2 + 2 + 根指针 86；之后 8 字节是出生 txg），I-9.25 判它。
const TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86;
const INODE_RECORD_BLOCKS_OFFSET: usize = 48;
/// `blocks` 的计量单位：512 字节一块，`blocks` = ⌈`size` ÷ 512⌉，是逻辑长度的块数、不表示分到的空间（D8（核心索引结构） 已定项 6，
/// C480（inode 记录的 blocks 怎么算全仓没有条款） 用户 2026-09-23 定）。
const INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES: u64 = 512;

fn data_unit_bytes() -> usize {
    usize::try_from(DATA_UNIT_BYTES).expect("32768")
}
fn node_bytes() -> usize {
    usize::try_from(NODE_BYTES).expect("16384")
}
fn tree_table_entry_bytes() -> usize {
    usize::try_from(TREE_TABLE_ENTRY_BYTES).expect("200")
}
fn allocation_record_bytes() -> usize {
    usize::try_from(ALLOCATION_RECORD_BYTES).expect("20")
}
fn extent_leaf_record_bytes() -> usize {
    usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112")
}
fn inode_internal_entry_bytes() -> usize {
    usize::try_from(INODE_INTERNAL_ENTRY).expect("120")
}
fn accounting_entry_bytes() -> usize {
    usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34")
}
/// 中央映射条目今天走读要几个字节：key 27 + 位置条目 14 × 2。条目一律 55 已由 D19（块指针的结构与宽度预算） 已定项 10 定，
/// 但这里**不判条目宽是不是正好 55**（把映射树接进 I-1.10 还欠在 C307），只是「切到偏移 55 之前得有 55 个字节」。
fn mapping_entry_bytes() -> usize {
    usize::try_from(MAPPING_ENTRY_BYTES).expect("55")
}
/// 一棵树的条目该有多宽（各棵树的条目字段表）：走读按固定偏移切条目之前先比它（I-1.10（码 2 条目宽等于字段表宽））。
/// 条目宽是索引节点头里的一个**盘上字段**，`index_node_view` 只判了它 ≥ key 宽
/// （extent 24、inode 8、分配记录 10、记账 22），够不着「装得下整条记录」。
///
/// ⚠️ **中央映射树（种类 5）不在这张表里**：它的条目宽 D19（块指针的结构与宽度预算） 已定项 10 已定一律 55，
/// 但把种类 5 接进这张表、让 I-1.10（码 2 条目宽等于字段表宽） 也判它，还欠在 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树），
/// 所以今天 I-1.10 不判它。`key_schema_for_tree_kind` 罩 5 个种类、这里只罩 4 个，
/// 两张表的差就是那一棵。
fn field_table_width_of_tree_kind(kind: u16) -> Option<usize> {
    match kind {
        TREE_KIND_EXTENT => Some(extent_leaf_record_bytes()),
        TREE_KIND_INODE => Some(inode_internal_entry_bytes()),
        TREE_KIND_ALLOCATION => Some(allocation_record_bytes()),
        TREE_KIND_ACCOUNTING => Some(accounting_entry_bytes()),
        TREE_KIND_DELETED_INODES => Some(DELETED_INODES_ENTRY_BYTES),
        // 墓碑树、谱系树的根条目照 inode 树内部节点的 120 字节形态（D18（块里携带什么信息） 已定项 24 第 1、5 条）。
        TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE | TREE_KIND_RETIREMENT => {
            Some(inode_internal_entry_bytes())
        }
        _ => None,
    }
}

/// 读一个码 2 节点时内部节点的 key 区间怎么判（I-1.1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyRangeReading {
    /// 区间贴紧首末两条条目的 key：叶一律这样；inode 树根、树表与根兼叶的那几棵今天也这样写。
    FirstAndLastEntries,
    /// 多层码 2 树（记账树、中央映射树）：叶贴紧首末条目；内部节点的区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2），
    /// 要等孩子读回来才判得了，由走读那一方判（`Walk::walk_code_two_subtree` ③），读节点这一步只判分隔 key 严格递增。
    SubtreeCoverageOfInternalNodesJudgedByTheCaller,
    /// 按位置寻址的两棵树（分配记录树、extent 树，D8（核心索引结构） 已定项 14）：每个节点的区间是它的位置规定罩的那一段
    /// （D18（块里携带什么信息） 已定项 2 对按位置寻址的树那一句），不贴紧首末条目，由走读那一方按位置判；读节点这一步只判条目 key 严格递增、节点不空。
    PrescribedByThePositionJudgedByTheCaller,
    /// 同上，读的是 extent 树上段的根：层级 0 的根兼叶可以一条条目都没有（第一个文件删完之后的零条目根，D8（核心索引结构） 已定项 16 第 5 条），
    /// 别的节点照旧不许空。
    PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller,
}

/// 主走读读一个码 2 节点（[`Walk::read_index_node`]）的三种结局。「先走过」按读回那一份的落点认，调用方看这个结局，不在读之前另按
/// 第 0 条位置条目问一遍（第 0 条那一份对不上、读回的是被先走过的版本读过的第 1 条那一份时，两个键会把「先走过」认成「走不下去」，
/// 第 32 批实现员报告停下第 2 条；判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 1 条）。
#[derive(Debug)]
enum IndexNodeReading {
    /// 这一遍第一次走到，头、出生身份、树 ID 与 key 都判过：节点连同读回那一份的落点（这个单元在走读里的身份，往下走时记成父单元）。
    WalkedFirstByThisVersion {
        view: crate::IndexNodeView,
        placement_it_was_read_from: (u32, u64),
    },
    /// 读回那一份的落点别的版本先走过：它下面的落点已经并进这一版（[`Walk::merge_placements_below_a_unit_walked_by_an_earlier_version`]），不重走。
    WalkedByAnEarlierVersion {
        placement_it_was_read_from: (u32, u64),
    },
    /// 指针全零，或两份都读不出、头用不了、条目区解不开（后几种走读失败已记）：这一支不往下走。
    NotWalkable,
}

/// 多层码 2 树一个节点的条目宽在走读里怎么对待。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EntryWidthInTheWalk {
    /// 按 I-1.10（码 2 条目宽等于字段表宽） 判：记账树叶 34、内部节点 108。
    JudgedAsInvariantOneTen { field_table_bytes: usize },
    /// 只守走读按固定偏移切要几个字节，不判「该有多宽」：中央映射树（条目宽已定 55，见 D19（块指针的结构与宽度预算） 已定项 10；I-1.10 还不罩它，接进射程欠在 C307）。
    GuardedForTheWalkOnly { bytes_the_walk_needs: usize },
}

impl EntryWidthInTheWalk {
    /// 这棵树在不在 I-1.10（码 2 条目宽等于字段表宽） 的射程里：按 I-1.10 判条目宽的那一种就在。
    const fn entry_field_table_registration(self) -> EntryFieldTableRegistration {
        match self {
            EntryWidthInTheWalk::JudgedAsInvariantOneTen { .. } => {
                EntryFieldTableRegistration::Registered
            }
            EntryWidthInTheWalk::GuardedForTheWalkOnly { .. } => {
                EntryFieldTableRegistration::NotRegistered
            }
        }
    }
}

/// 一个码 2 节点所在的树登没登记条目字段表，即它在不在 I-1.10（码 2 条目宽等于字段表宽） 的射程里。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EntryFieldTableRegistration {
    /// extent、inode、分配记录、记账四棵：条目宽 0 而条目数非 0 报 I-1.10——宽 0 不是任何登记宽（实审 A3-checker-2，主 agent 定）。
    Registered,
    /// 树表与中央映射树：I-1.10 那一行明写不罩（映射树的条目宽已定 55、接进 I-1.10 欠在 C307，树表条目宽两边都没写、账在 C476），
    /// 条目宽 0 而条目数非 0 只记走读失败、不报违例。
    NotRegistered,
}

/// 走一棵多层码 2 树要的几样。
#[derive(Clone, Copy, Debug)]
struct MultiLevelTreeInTheWalk<'name> {
    tree: u64,
    schema: crate::KeySchema,
    leaf_entry_width: EntryWidthInTheWalk,
    internal_entry_width: EntryWidthInTheWalk,
    name: &'name str,
}

/// 走一个节点连同它下面的结果：交回它头里的 key 区间（父节点判两条不等式与覆盖区间要它）与这棵子树走全了没有。
#[derive(Clone, Debug, PartialEq, Eq)]
enum SubtreeInTheWalk {
    Walked {
        smallest_key: Vec<u8>,
        largest_key: Vec<u8>,
        is_complete: bool,
    },
    /// 这一遍之前别的根已经走过它（`visited_units`）：那时按它自己的父条目判过，这里不再判。
    AlreadyWalkedFromAnotherRoot,
    /// 读不出、头用不了、层级不对或条目宽不对：走读失败已记，这一支不往下走。
    NotWalkable,
}

/// 指向码 2 / 码 3 的节点指针宽：头部 50 + 位置条目 14 × 2 + 实例代号 4 + 出生序号 4。
fn node_pointer_bytes() -> usize {
    usize::try_from(NODE_POINTER_BYTES).expect("86")
}

/// 多层码 2 树内部节点的条目宽：本树 key + 子指针 86（D8（核心索引结构） 已定项 11：记账树 108、中央映射树 113）。
fn internal_entry_bytes(schema: crate::KeySchema) -> usize {
    schema.width() + node_pointer_bytes()
}

/// 一个走过的单元自己的指针引用的一个落点 (设备, 槽)，连同第一次引用它的是谁。
type PlacementReferencedBelowAWalkedUnit = ((u32, u64), String);

/// 一次走读的状态：判定累加器、被引用单元的物理范围、走读有没有断。
struct Walk<'reader> {
    reader: &'reader dyn ImageReader,
    judgements: Judgements,
    filesystem_identifier_low: u64,
    /// (设备, 起点槽, 跨度槽) → 第一次引用它的是谁。
    references: BTreeMap<(u32, u64, u64), String>,
    /// (设备, 起点槽) → 走过的指针里指向它的那几条位置项带的单元校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 读盘核比的那一个）：
    /// 隔离的记录按它判那一份对不对得上（`quarantined_slots_exempted_per_device`）。同一个槽被复用过时几版指着不同的单元，都留着。
    location_entry_checksums: BTreeMap<(u32, u64), BTreeSet<u32>>,
    /// (设备, 起点槽)：整个走读（各版合起来）走过的单元，再遇到就不再往下走——不同版之间共享没改过的单元是合法的 COW，
    /// 走过一遍就够。同一版里再遇到同一个单元不是共享，那一格由 `placements_referenced_in_this_version` 判。
    visited_units: BTreeSet<(u32, u64)>,
    /// 正在走的这一版（根环里的一条根，或由记录施加出来的一版）里，走读跟着指针引用过的落点 (设备, 槽) → 第一次引用它的是谁。
    /// 同一版里第二次引用同一个落点，就是两个不同的引用占着同一段物理范围：I-5.1（物理范围不重叠） 判红
    /// （`references` 按 (设备, 起点槽, 跨度) 去重，两条引用落点与跨度都相同时那边看不出来）。
    /// 每走一版之前清空（`begin_walking_a_version`）。中央映射条目不进这里：映射条目与树里的指针指着同一个单元是条款定的两处引用
    /// （D19（块指针的结构与宽度预算） 已定项 5 / 8），不是第二次引用。
    placements_referenced_in_this_version: BTreeMap<(u32, u64), String>,
    /// 走读此刻正在跟着哪个单元里的指针往下走（它的落点 (设备, 槽)）；`None` 是根记录（或由记录施加出来那一版的新根段）里的指针。
    walk_parent_unit: Option<(u32, u64)>,
    /// 走过的单元 (设备, 槽) → 它自己的指针引用的落点（连同第一次引用它的是谁），按走到的次序。一个单元只走一次（`visited_units`），
    /// 别的版本再引用它时不重走：按这张表把它下面的落点（孩子、孩子的孩子……）并进那一版的集合再比交叉（I-5.1（物理范围不重叠）），
    /// 不重读盘（实审 B1 留下的缺口，用户 2026-09-27 定补）。
    placements_referenced_below_a_walked_unit:
        BTreeMap<(u32, u64), Vec<PlacementReferencedBelowAWalkedUnit>>,
    /// 走过的 inode 树单元 (设备, 槽)（叶容器，或类型段 0 指着的内部节点）→ 它下面按叶序的叶容器：别的版本再引用它时，
    /// I-9.4（容器号不超最小 key） 的叶序照这张表接上，不重读盘。
    inode_leaf_order_below_a_walked_unit: BTreeMap<(u32, u64), Vec<InodeLeafContainerInLeafOrder>>,
    walk_failures: Vec<String>,
    /// 走过的全部树表条目里的树 ID（I-7.8 的一半）。
    tree_identifiers_in_tables: BTreeSet<u64>,
    /// inode 号 → 对象出生代；数据单元 → (对象, 对象出生代)。I-9.10 在最后对。
    inode_object_birth: BTreeMap<u64, u64>,
    /// 走到过 inode 树的根、并且它是层级大于 0 的内部节点（I-9.6（水位大于两处最大号） 与 I-9.12（分隔 key 落在孩子区间之外） 都要它）。
    inode_tree_walked: bool,
    /// 遍历侧算出来的「该树 inode 树内最大 key」：逐片叶容器逐条记录取最大，与记账里的水位是两条独立路径（I-9.6）。
    largest_inode_number_in_the_inode_tree: Option<u64>,
    data_unit_objects: Vec<(u64, u64, String)>,
    /// 最新根下面记账树里的行：(统计量, 设备) → 值。
    accounting: BTreeMap<(u16, u32), u64>,
    accounting_seen: bool,
    /// 走全了的记账树里 inode 号水位那一统计量（不带设备维）的每一行 (代段, 值)，按走到的次序（key 升序）：
    /// 每次发布写的那一行与台阶行（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）。只取最新根那一遍（`check_pool_image` 走完最新根就拷走）。
    inode_number_watermark_rows: Vec<(u64, u64)>,
    /// 最新根指着的实例表里的行：别的根按它判有效（回退之后被抛弃时间线的根不进 I-3.1 的并集），
    /// I-1.8（归并后版本全序） 的已发布谓词也读它。
    instance_table_rows: Vec<InstanceTableRow>,
    /// 主走读沿最新根的实例表链走没走到末片（链指针记录说「没有下一片」的那一片）：走到了，`instance_table_rows` 就是整张表的行，
    /// 回退候选集按它滤掉了被崩溃恢复抛弃的根；没走到（链上一片读不出、解不开、链指针记录不合法、链成了环）就滤不全。
    /// I-9.29（同号只许一个出生代） 判不了的门读它，与候选集的滤法用同一遍走读（判决 `research/prompts/m3-batch23-code-r1-main-verification.md`
    /// 第三节第 1 条，攻方改法 A）。
    abandoned_roots_filtering: AbandonedRootsFilteringOfTheCandidateSet,
    /// 挂载根（池级 checker 里就是最新的那条根）的实例代号与 txg：已发布谓词的另外两个输入
    /// （I-1.2（块头写序已发布） / I-4.2（无被引用未提交块） 在遍历方向上各判一格要它）。
    mount_root_instance: u32,
    mount_root_checkpoint_txg: u64,
    /// 遍历方向上判过出生身份的被引用单元数：一个都没有时 I-1.2 与 I-4.2 报「不适用」，不报成立。
    referenced_units_judged_against_their_pointer: u64,
    /// 正在走的是哪一版、它的 `checkpoint_txg`：I-9.25 与 I-3.13 按这一版的 txg 判，I-4.9 只在不是最新的候选根上判。
    version_being_walked: VersionBeingWalked,
    /// 走过的被引用单元（按读回那一份的落点认，与 `visited_units` 同一个键）→ 头里的诞生代号：别的版本再引用它时不重读，
    /// I-4.9（非最新候选根引用的单元不晚于它） 照这张表判。
    birth_txg_of_walked_units: BTreeMap<(u32, u64), u64>,
    /// I-4.9 在不是最新的候选根上判过几个被引用单元：一个都没有（候选集只剩最新那条根）时整条报不适用。
    units_judged_against_an_older_candidate_root: u64,
    /// 跟着指针记下的每个落点 (设备, 槽) → 它是哪个单元（那条指针第 0 条位置条目的落点，与 `visited_units`、`birth_txg_of_walked_units`
    /// 同一个键）：I-3.5 比相邻两版引用了哪些单元时，两份副本的落点折成一个单元。读回的是第 1 条那一份时，那个落点改记成它自己
    /// （[`Walk::read_a_unit_to_walk_and_the_placement_it_was_read_from`]，判决 `research/prompts/m3-batch32-code-r1-main-verification.md`
    /// 第三节第 1 条，攻方 allrbmap）：第 0 条落点上是别的单元或坏副本时，I-3.5 取的是读回那一份的头里的诞生代号。
    unit_of_each_placement: BTreeMap<(u32, u64), (u32, u64)>,
    /// 跟着指针记下的落点里，记的那一刻走读正在一棵用户可见树（extent、inode、`deleted_inodes`）里往下走的那些 → 那棵树
    /// （I-3.5 (b) 认「落在 R 树表里某棵用户可见树的子树里」按它）。别的版本共享这棵子树时不重走，照这张表接上。
    user_visible_tree_of_each_placement: BTreeMap<(u32, u64), UserVisibleTreeOfTheWalk>,
    /// 走读此刻在哪棵用户可见树里往下走；不在任何一棵里是 `None`。
    user_visible_tree_being_walked: Option<UserVisibleTreeOfTheWalk>,
    /// 正在走的这一版树表里每棵根指针非全零的用户可见树，带根指针的出生 txg（I-3.5 (b) 拿它比 R⁻.txg）。
    user_visible_tree_roots_of_this_version: Vec<UserVisibleTreeRootOfAVersion>,
    /// 走过的树表单元 (设备, 槽) → 它每一条条目里 I-9.25 与 I-3.5 读的几样：别的版本共享这片树表时照这份判，不重读盘。
    tree_table_entries_of_a_walked_tree_table: BTreeMap<(u32, u64), Vec<TreeTableEntryOfTheWalk>>,
    /// 走过的实例表各片（按读回那一份的落点）→ 头里的诞生代号：I-3.5 要它。不进 `birth_txg_of_walked_units`——
    /// 那张表 I-4.9 也读，而实例表单元按 I-1.2 那一行的例外不进 I-4.9。
    birth_txg_of_walked_instance_table_pages: BTreeMap<(u32, u64), u64>,
    /// 已经走完的每一版（根环里的候选根与由记录施加出来的那几版）：I-3.5 在全部走完之后按那一刻的折叠表折成单元、逐对比。
    versions_walked_for_the_appearance_rule: Vec<VersionPlacementsForTheAppearanceRule>,
}

/// 用户可见树的种类码：回退取回退目标那一版的几棵（extent、inode、`deleted_inodes`，`crates/singlefs-core/src/mount.rs`
/// 的 `version_to_publish_the_rollback_after`；D23（journal 的角色与格式） 已定项 14）。I-3.5 (b) 只认这几棵里的单元。
const USER_VISIBLE_TREE_KINDS: [u16; 3] =
    [TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_DELETED_INODES];

/// 一棵用户可见树：种类与树 ID。I-3.5 按它给落点打标签。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct UserVisibleTreeOfTheWalk {
    kind: u16,
    tree: u64,
}

/// 一版树表里一棵根指针非全零的用户可见树，带根指针头里的出生 txg。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UserVisibleTreeRootOfAVersion {
    tree: UserVisibleTreeOfTheWalk,
    root_pointer_birth_txg: u64,
}

/// 树表的一条条目里 I-9.25 与 I-3.5 读的几样（D8（核心索引结构） 已定项 8 的字段表：树 ID 0、种类 10、根指针 14..100、
/// `previous_snapshot_txg` 100）。走读第一次读到这片树表时记下。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TreeTableEntryOfTheWalk {
    tree: u64,
    kind: u16,
    previous_snapshot_txg: u64,
    /// 种类是用户可见树、根指针非全零时是那棵树的根；别的条目 `None`。
    user_visible_root: Option<UserVisibleTreeRootOfAVersion>,
}

/// 一条树表条目按字段表切出来的几样；条目比字段表窄（走读守卫那一格，`walk_tree_table_entry` 记走读失败）时 `None`。
fn tree_table_entry_of_the_walk(entry: &[u8]) -> Option<TreeTableEntryOfTheWalk> {
    if entry.len() < tree_table_entry_bytes() {
        return None;
    }
    let tree = read_u64(entry, 0);
    let kind = read_u16(entry, 10);
    let root_pointer = parse_node_pointer(&entry[14..100]);
    let user_visible_root = (USER_VISIBLE_TREE_KINDS.contains(&kind) && !root_pointer.all_zero)
        .then_some(UserVisibleTreeRootOfAVersion {
            tree: UserVisibleTreeOfTheWalk { kind, tree },
            root_pointer_birth_txg: root_pointer.birth_txg,
        });
    Some(TreeTableEntryOfTheWalk {
        tree,
        kind,
        previous_snapshot_txg: read_u64(entry, TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET),
        user_visible_root,
    })
}

/// 走完的一版留给 I-3.5 的东西：它是哪一版、引用过哪些落点（还没折成单元）、树表里几棵用户可见树的根、走读断没断。
/// 落点到全部版本都走完才折：折叠表（`Walk::unit_of_each_placement`）在后走的版本读回第 1 条那一份时还会改
/// （[`Walk::read_a_unit_to_walk_and_the_placement_it_was_read_from`]），每版走完当场折的话，先走那一版存下的单元跟不上这一改，
/// I-3.5 随根环次序翻面（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S1 那一行与第三节第 1 条，攻方 F1）。
/// GPU 走读内核同一个取法（每版只存落点表的下标，判时按落点表那一行此刻的单元认）。
struct VersionPlacementsForTheAppearanceRule {
    instance: u32,
    checkpoint_txg: u64,
    placements: Vec<(u32, u64)>,
    user_visible_tree_roots: Vec<UserVisibleTreeRootOfAVersion>,
    references_are_complete: bool,
}

/// 走完的一版在 I-3.5 眼里的样子：它是哪一版、引用了哪些单元（各落在哪棵用户可见树里）、树表里几棵用户可见树的根、走读断没断。
struct VersionReferencesForTheAppearanceRule {
    instance: u32,
    checkpoint_txg: u64,
    /// 这一版引用的单元（落点按全部版本走完之后的折叠表折成单元）→ 它落在哪棵用户可见树里（不在任何一棵里是 `None`）。
    units: BTreeMap<(u32, u64), Option<UserVisibleTreeOfTheWalk>>,
    user_visible_tree_roots: Vec<UserVisibleTreeRootOfAVersion>,
    /// 这一版走读一条走读失败都没记：记了就是引用集合数不全，它参与的每一对报不适用。
    references_are_complete: bool,
}

/// 回退候选集里相邻两版之间有没有洞（I-3.5 的缺口条款；I-3.1 在短环上洞里的误红归下一批调查，那一处要接同一份判法，
/// `research/prompts/m3-birth-floor-r3-main-verification.md` 第二节第 5 行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HoleInTheCandidateSet {
    /// R.txg = R⁻.txg + 1。
    NoHole,
    /// 有洞，但最新根的实例表行证明 R 所在的实例直接接在 R⁻ 那一版上（gap-row）：这一对照常判。
    HoleBridgedByTheInstanceTableRows,
    /// 有洞、实例表行接不上：用户可见树子树里的单元不判（N3），别的照 (a) 判。
    HoleNotBridgedByTheInstanceTableRows,
}

/// 相邻两版 (R⁻, R) 之间的洞：R⁻ 那一侧是同 txg 并成的一条（几个实例都算 R⁻ 的实例）。接得上 ⟺ 对 R⁻ 的某个实例 i：
/// 最新根的实例表里有一行 (i, T = R⁻.txg)，R 的实例 v > i，且 i 与 v 之间的每个实例 j 都有一行、都是 (j, 0, 0)。
/// 迭代上界是实例表的行数（不按实例代号逐个数，代号是盘上读来的 4 字节）。
fn hole_in_the_candidate_set_between(
    previous_txg: u64,
    previous_instances: &BTreeSet<u32>,
    version_txg: u64,
    version_instance: u32,
    instance_table_rows_of_the_newest_root: &[InstanceTableRow],
) -> HoleInTheCandidateSet {
    if previous_txg.checked_add(1) == Some(version_txg) {
        return HoleInTheCandidateSet::NoHole;
    }
    let bridged_from = |previous_instance: u32| {
        if version_instance <= previous_instance {
            return false;
        }
        let row_of_the_previous_version =
            instance_table_rows_of_the_newest_root.iter().any(|row| {
                row.instance == previous_instance && row.published_checkpoint_txg == previous_txg
            });
        let rows_between: Vec<&InstanceTableRow> = instance_table_rows_of_the_newest_root
            .iter()
            .filter(|row| row.instance > previous_instance && row.instance < version_instance)
            .collect();
        let instances_between: BTreeSet<u32> =
            rows_between.iter().map(|row| row.instance).collect();
        let every_instance_between_has_a_row = u64::try_from(instances_between.len())
            .expect("实例表行数装得进 u64")
            == u64::from(version_instance - previous_instance - 1);
        let every_row_between_published_nothing = rows_between.iter().all(|row| {
            row.published_checkpoint_txg == 0 && row.applied_transaction_high_water_mark == 0
        });
        row_of_the_previous_version
            && every_instance_between_has_a_row
            && every_row_between_published_nothing
    };
    if previous_instances
        .iter()
        .any(|previous_instance| bridged_from(*previous_instance))
    {
        return HoleInTheCandidateSet::HoleBridgedByTheInstanceTableRows;
    }
    HoleInTheCandidateSet::HoleNotBridgedByTheInstanceTableRows
}

/// I-3.5（引用区间的精确性） 的出现规则（TF + β + gap + gap-row + N3，`research/prompts/m3-birth-floor-r3-main-verification.md`、
/// 用户 2026-10-03 定丙；措辞是 `.claude/kb/invariants.md` I-3.5 那一行）：回退候选集（走过的每一版，与 I-7.4 同一份）按 txg 升序，
/// 同 txg 的几条并成一条（R⁻ 侧引用集合取并，R 侧逐条各判一次），相邻两条 (R⁻, R) 逐对判。R 引用而 R⁻ 不引用的每个单元 b
/// （第一次出现与再次出现都判）满足 (a) birth(b) > R⁻.txg；或 (b) b 落在 R 树表里一棵用户可见树的子树里，那棵树的根指针出生 txg
/// 严格小于 R⁻.txg，且 R 里那棵子树每个读得出的单元头的诞生代号都不大于那个出生 txg。birth(b) 取单元头里的诞生代号，头读不出的不判。
/// 有洞（[`hole_in_the_candidate_set_between`]）而实例表行接不上时，用户可见树子树里的单元不判（N3）。R 或 R⁻ 走读记过失败的那一对不判；
/// 一个单元都没判到时整条报不适用。
fn judge_the_appearance_rule_of_adjacent_candidate_versions(
    versions: &[VersionReferencesForTheAppearanceRule],
    birth_txg_of_walked_units: &BTreeMap<(u32, u64), u64>,
    birth_txg_of_walked_instance_table_pages: &BTreeMap<(u32, u64), u64>,
    instance_table_rows_of_the_newest_root: &[InstanceTableRow],
    judgements: &mut Judgements,
) {
    let birth_txg_of = |unit: &(u32, u64)| {
        birth_txg_of_walked_units
            .get(unit)
            .or_else(|| birth_txg_of_walked_instance_table_pages.get(unit))
            .copied()
    };
    let mut versions_of_each_txg: BTreeMap<u64, Vec<&VersionReferencesForTheAppearanceRule>> =
        BTreeMap::new();
    for version in versions {
        versions_of_each_txg
            .entry(version.checkpoint_txg)
            .or_default()
            .push(version);
    }
    let groups: Vec<(u64, Vec<&VersionReferencesForTheAppearanceRule>)> =
        versions_of_each_txg.into_iter().collect();
    let mut units_judged: u64 = 0;
    for adjacent in groups.windows(2) {
        let (previous_txg, previous_versions) = (adjacent[0].0, &adjacent[0].1);
        let versions_at_this_txg = &adjacent[1].1;
        let previous_references_are_complete = previous_versions
            .iter()
            .all(|previous_version| previous_version.references_are_complete);
        let units_referenced_by_the_previous_version: BTreeSet<(u32, u64)> = previous_versions
            .iter()
            .flat_map(|previous_version| previous_version.units.keys().copied())
            .collect();
        let previous_instances: BTreeSet<u32> = previous_versions
            .iter()
            .map(|previous_version| previous_version.instance)
            .collect();
        for version in versions_at_this_txg {
            if !previous_references_are_complete || !version.references_are_complete {
                continue;
            }
            let hole = hole_in_the_candidate_set_between(
                previous_txg,
                &previous_instances,
                version.checkpoint_txg,
                version.instance,
                instance_table_rows_of_the_newest_root,
            );
            let trees_inherited_whole: BTreeSet<UserVisibleTreeOfTheWalk> = version
                .user_visible_tree_roots
                .iter()
                .filter(|root| {
                    root.root_pointer_birth_txg < previous_txg
                        && version
                            .units
                            .iter()
                            .filter(|(_, tree)| **tree == Some(root.tree))
                            .filter_map(|(unit, _)| birth_txg_of(unit))
                            .all(|birth_txg| birth_txg <= root.root_pointer_birth_txg)
                })
                .map(|root| root.tree)
                .collect();
            for (unit, user_visible_tree) in &version.units {
                if units_referenced_by_the_previous_version.contains(unit) {
                    continue;
                }
                if hole == HoleInTheCandidateSet::HoleNotBridgedByTheInstanceTableRows
                    && user_visible_tree.is_some()
                {
                    continue;
                }
                let Some(birth_txg) = birth_txg_of(unit) else {
                    continue;
                };
                units_judged += 1;
                let born_after_the_previous_version = birth_txg > previous_txg;
                let in_a_tree_inherited_whole =
                    user_visible_tree.is_some_and(|tree| trees_inherited_whole.contains(&tree));
                judgements.judge(
                    REFERENCE_INTERVALS_ARE_EXACT,
                    born_after_the_previous_version || in_a_tree_inherited_whole,
                    || {
                        format!(
                            "相邻两版 (实例 {previous_instances:?}、txg {previous_txg}) → (实例 {}、txg {})（{hole:?}）：后一版引用、前一版不引用的单元盘 {} 槽 {}（用户可见树 {user_visible_tree:?}）诞生代号 {birth_txg} 不大于前一版的 txg，也不在一棵整棵继承下来的用户可见树里（根指针出生 txg 小于前一版的 txg、子树里读得出的诞生代号都不大于它）",
                            version.instance, version.checkpoint_txg, unit.0, unit.1
                        )
                    },
                );
            }
        }
    }
    if units_judged == 0 {
        judgements.not_applicable(
            REFERENCE_INTERVALS_ARE_EXACT,
            "回退候选集里相邻两版之间一个新出现、头读得出的单元都没判到：候选集只剩一版，或相邻两版有一版走读记过失败，或缺口那一对新出现的只有用户可见树里的单元",
        );
    }
}

/// 一版的树表指针与映射根指针是从哪里读来的：根环里的根读根记录，由记录造出来的版本读那条记录的新根段（D23（journal 的角色与格式）
/// 已定项 15）。两处的树表指针整条全零时读法相同，只在说明里分开写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeTablePointerHolder {
    RootRecord,
    NewRootSegmentOfTheRecord,
}

/// 走读此刻在走哪一版（D16（发布语义） 已定项 1 的回退候选集里的根，或由记录施加出来、根槽从没落盘的那一版），带它的 `checkpoint_txg`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VersionBeingWalked {
    /// (txg, 实例) 最大的那条根：先走它。
    NewestRoot { checkpoint_txg: u64 },
    /// 回退候选集里不是最新的根（I-4.9 只判这一类）。
    OlderCandidateRoot { checkpoint_txg: u64 },
    /// 由记录施加出来、根槽从没落盘的那一版（[`versions_applied_only_by_records`]）。
    AppliedOnlyByRecords { checkpoint_txg: u64 },
}

impl VersionBeingWalked {
    const fn checkpoint_txg(self) -> u64 {
        match self {
            Self::NewestRoot { checkpoint_txg }
            | Self::OlderCandidateRoot { checkpoint_txg }
            | Self::AppliedOnlyByRecords { checkpoint_txg } => checkpoint_txg,
        }
    }
}

/// 实例表里的一行（D18（块里携带什么信息） 已定项 11 的字段表：`kind` 1 + 实例代号 4 + 所选根的 `checkpoint_txg` 8 +
/// 属于该实例的最大已施加事务号 W 8 + flags 1 + 预留 66）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InstanceTableRow {
    instance: u32,
    /// 这一行记的 T：该实例被施加到哪一代为止（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的 `T_pub`）。
    published_checkpoint_txg: u64,
    /// 这一行记的 W：属于该实例的、被这次重放施加的最大事务号。
    applied_transaction_high_water_mark: u64,
}

/// 实例表一片末尾那条链指针记录（D18（块里携带什么信息） 已定项 11：`kind 1 | 有无下一片 1 | 位置指针 86（无下一片时清零占位）| 预留 0`）
/// 在 checker 这边的读法，按字段表另写一份，不用实现的解析。「有无下一片」的「有」按 1 读：条款只写了字段名，
/// 0 是 mkfs 那一片写的「无」（字节表七 m1），1 与拒收别的值是按字段名与同一张表里 flags 字节的口径读的。
#[derive(PartialEq)]
enum ChainRecordView {
    LastPage,
    NextPage(PointerView),
    /// 记录宽不是 88、kind 不是 1、有无下一片不是 0 或 1、无下一片而位置指针不全零、有下一片而位置指针全零、
    /// 有下一片而位置指针头部有一段不是第一版的取值（[`pointer_head_fields_take_the_first_version_values`]）。
    Malformed,
}

/// 一条指针头部 extent 偏移及之前那几段——MAC 16 与 nonce 12、算法类型、压缩算法码、压后长度、extent 偏移——都是第一版的取值
/// （I-2.4 在走读跟随的指针上逐段判的那几格，`image::judge_pointer_mac_and_nonce_are_zero` 等三个判法读的同一组字段）。
/// 实例表链指针记录说「有下一片」时只在这几段都过时算合法：有一段不过是 I-3.14 第五形，这一片解不开（用户 2026-10-06 定 K2-1 甲，
/// `research/prompts/m3-batch10o-code-r1-forks.md`；实现 `InstanceTableChainRecord::parse` 经 `NodePointer::read_with_head_fields_judgement_from`
/// 判同一组，D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14、C581）。
fn pointer_head_fields_take_the_first_version_values(pointer: &PointerView) -> bool {
    let fields = pointer.encryption_and_compression_fields;
    pointer.is_mac_and_nonce_all_zero
        && fields.algorithm_type == 0
        && fields.compression_algorithm_code == 0
        && fields.compressed_length == 0
        && pointer.is_extent_offset_zero
}

/// 链指针记录里位置指针的偏移：kind 1 字节 + 有无下一片 1 字节。
const CHAIN_RECORD_POINTER_OFFSET: usize = 2;
/// 实例表记录宽（D18（块里携带什么信息） 已定项 11）：链指针记录与行记录同宽。
const INSTANCE_TABLE_RECORD_BYTES: usize = 88;

fn chain_record_view(record: &[u8]) -> ChainRecordView {
    if record.len() != INSTANCE_TABLE_RECORD_BYTES || record[0] != 1 {
        return ChainRecordView::Malformed;
    }
    let pointer = parse_node_pointer(&record[CHAIN_RECORD_POINTER_OFFSET..]);
    match record[1] {
        0 if pointer.all_zero => ChainRecordView::LastPage,
        1 if !pointer.all_zero && pointer_head_fields_take_the_first_version_values(&pointer) => {
            ChainRecordView::NextPage(pointer)
        }
        _flag_or_pointer_that_does_not_match => ChainRecordView::Malformed,
    }
}

/// 实例表一条记录的种类：首字节 `kind`（D18（块里携带什么信息） 已定项 11 只登记两种：0 行、1 链指针记录）。
/// 别的值没登记（用户 2026-10-06 定：`kind` 不是 0 或 1 ⇒ 这一片解不开），留着原值好写进违例说明。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InstanceTableRecordKind {
    Row,
    ChainRecord,
    Unregistered(u8),
}

impl InstanceTableRecordKind {
    fn of_record(record: &[u8]) -> Self {
        match record
            .first()
            .copied()
            .expect("实例表记录宽 88：judge_packed_container 判过记录宽等于登记的宽")
        {
            0 => Self::Row,
            1 => Self::ChainRecord,
            unregistered_kind => Self::Unregistered(unregistered_kind),
        }
    }
}

/// 实例表行记录（`kind` 0）里 flags 字节的偏移：kind 1 + 实例代号 4 + 所选根的 checkpoint_txg 8 + W 8（D18（块里携带什么信息） 已定项 11）。
const INSTANCE_TABLE_ROW_FLAGS_OFFSET: usize = 21;
/// 行记录 flags 第一版唯一合法的值（D18（块里携带什么信息） 已定项 11：「第一版恒 0、非 0 拒收」，bit0 退役也在内）。
const INSTANCE_TABLE_ROW_FLAGS_OF_THE_FIRST_VERSION: u8 = 0;

/// I-3.8（实例表行唯一且低于挂载根）在最新根的实例表第 0 片就解不开（I-3.14 不成立）时报的不适用理由：一行都没收，没有行可判
/// （不落到 `Judgements::into_report` 的缺省理由「这个镜像上没有它判的对象」）。
const INSTANCE_TABLE_ROWS_NOT_JUDGED_AS_THE_FIRST_PAGE_OF_THE_NEWEST_ROOTS_TABLE_DOES_NOT_DECODE:
    &str = "最新根的实例表那一片解不开（I-3.14 不成立）、行没收全：没有行可判";

/// 一片实例表的记录形态（I-3.14 那几格）合不合法。不合法的那一片解不开：用户 2026-10-06 给 D18（块里携带什么信息） 已定项 11 补
/// 「kind 不是 0 或 1 ⇒ 这一片解不开（表不可读）」，实现的 `InstanceTablePage::parse` 对「最后一条不是合法的链指针记录」「它前面有不是行的记录」
/// 「行的 flags 不是 0」都交 None；主走读照「片解不开」走（[`Walk::walk_instance_table_chain`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InstanceTablePageRecordForms {
    WellFormed,
    IllFormedSoThePageDoesNotDecode,
}

/// I-3.14（实例表每片记录形态合法）在一片上判三件，各一格：每条记录的 `kind` 只有 0 或 1；`kind` 1（链指针记录）不出现在最后一条之前；
/// `kind` 0 行的 flags 字节为 0。最后一条是不是合法的链指针记录是同一条不变量的第四格，在 `walk_instance_table_chain` 里判。
/// 交回三件是不是都成立：有一件不成立，这一片解不开（[`InstanceTablePageRecordForms`]）。
fn judge_instance_table_page_record_forms(
    judgements: &mut Judgements,
    records: &[Vec<u8>],
    what: &str,
) -> InstanceTablePageRecordForms {
    use crate::image::INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED;
    let kinds: Vec<InstanceTableRecordKind> = records
        .iter()
        .map(|record| InstanceTableRecordKind::of_record(record))
        .collect();
    let unregistered_kinds: Vec<(usize, u8)> = kinds
        .iter()
        .enumerate()
        .filter_map(|(position, kind)| match kind {
            InstanceTableRecordKind::Unregistered(raw_kind) => Some((position, *raw_kind)),
            InstanceTableRecordKind::Row | InstanceTableRecordKind::ChainRecord => None,
        })
        .collect();
    judgements.judge(
        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
        unregistered_kinds.is_empty(),
        || {
            format!(
                "{what}：（第几条, kind）{unregistered_kinds:?} 的 kind 不是 0（行）或 1（链指针记录）：这一片解不开"
            )
        },
    );
    let last_position = records.len().saturating_sub(1);
    let chain_records_before_the_last: Vec<usize> = kinds
        .iter()
        .enumerate()
        .filter_map(|(position, kind)| match kind {
            InstanceTableRecordKind::ChainRecord if position < last_position => Some(position),
            InstanceTableRecordKind::ChainRecord
            | InstanceTableRecordKind::Row
            | InstanceTableRecordKind::Unregistered(_) => None,
        })
        .collect();
    judgements.judge(
        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
        chain_records_before_the_last.is_empty(),
        || {
            format!(
                "{what}：第 {chain_records_before_the_last:?} 条是链指针记录，不在这一片（{} 条）的末尾",
                records.len()
            )
        },
    );
    let rows_with_flags_off_the_first_version: Vec<(usize, u8)> = records
        .iter()
        .zip(&kinds)
        .enumerate()
        .filter_map(|(position, (record, kind))| match kind {
            InstanceTableRecordKind::Row => {
                let flags = record[INSTANCE_TABLE_ROW_FLAGS_OFFSET];
                (flags != INSTANCE_TABLE_ROW_FLAGS_OF_THE_FIRST_VERSION)
                    .then_some((position, flags))
            }
            InstanceTableRecordKind::ChainRecord | InstanceTableRecordKind::Unregistered(_) => None,
        })
        .collect();
    judgements.judge(
        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
        rows_with_flags_off_the_first_version.is_empty(),
        || {
            format!(
                "{what}：（第几条, flags）{rows_with_flags_off_the_first_version:?} 的行 flags 不是 0"
            )
        },
    );
    if unregistered_kinds.is_empty()
        && chain_records_before_the_last.is_empty()
        && rows_with_flags_off_the_first_version.is_empty()
    {
        InstanceTablePageRecordForms::WellFormed
    } else {
        InstanceTablePageRecordForms::IllFormedSoThePageDoesNotDecode
    }
}

/// 指针尾段装的那个量：码 1 的指针 88 尾段是写序的事务号（6 字节），码 2 / 码 3 的指针 86 尾段是出生序号（4 字节）
/// （`image::parse_data_pointer` / `image::parse_node_pointer` 各解一份；`KEY_SCHEMA_MAPPING` 那一行的注同一个口径）。
/// 码 3 的写序里还有一个事务号，指针不带它 ⇒ 不比它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TailOfTheBirthIdentity {
    TransactionNumber(u64),
    BirthSequence(u32),
}

impl TailOfTheBirthIdentity {
    fn birth_identity_tail_number(self) -> u64 {
        match self {
            TailOfTheBirthIdentity::TransactionNumber(transaction) => transaction,
            TailOfTheBirthIdentity::BirthSequence(birth_sequence) => u64::from(birth_sequence),
        }
    }

    fn name(self) -> &'static str {
        match self {
            TailOfTheBirthIdentity::TransactionNumber(_) => "事务号",
            TailOfTheBirthIdentity::BirthSequence(_) => "出生序号",
        }
    }
}

/// 中央映射 key 里各段的偏移（D19（块指针的结构与宽度预算） 已定项 6 / 10：类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 尾段 6；
/// 尾段码 1 是事务号 6 字节，码 2 / 码 3 是出生序号 4 + 补零 2）。按字段表另写一份，不用实现的解析。
const MAPPING_KEY_UNIT_CLASS_OFFSET: usize = 0;
const MAPPING_KEY_BIRTH_TREE_OFFSET: usize = 1;
const MAPPING_KEY_BIRTH_TXG_OFFSET: usize = 1 + 8;
const MAPPING_KEY_INSTANCE_OFFSET: usize = 1 + 8 + 8;
const MAPPING_KEY_TAIL_OFFSET: usize = 1 + 8 + 8 + 4;
/// 码 2 / 码 3 的尾段是出生序号 4 字节，其后补零 2 字节（D19（块指针的结构与宽度预算） 已定项 10「末尾补零到 27」）。
const MAPPING_KEY_TAIL_PADDING: std::ops::Range<usize> =
    MAPPING_KEY_TAIL_OFFSET + 4..MAPPING_KEY_TAIL_OFFSET + 6;

/// 单元头里与映射 key 的出生树那一段相比的字段（D19（块指针的结构与宽度预算） 已定项 6 的收严句）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BirthTreeInTheHeader {
    /// 码 1 五元组里的树（偏移 43）、码 3 类身份段里的出生树（偏移 43）。
    AtOffsetFortyThree(u64),
    /// 码 2：类身份段里没有叫「出生树」的字段，取哪个还开着（C289（码 2 的出生树取哪个字段没写）），今天不比。
    FieldStillOpenUnderC289,
}

/// 映射 key 的出生树在被指单元头里对的是哪个字段。类标签不在 1 / 2 / 3 里的没有这一格（交 None，那一格按类标签判）。
fn birth_tree_in_the_header_as_the_mapping_key_names_it(
    unit: &[u8],
) -> Option<BirthTreeInTheHeader> {
    match MappingKeyUnitClass::of(unit[6]) {
        MappingKeyUnitClass::DataUnit | MappingKeyUnitClass::PackedRecordUnit => {
            Some(BirthTreeInTheHeader::AtOffsetFortyThree(read_u64(unit, 43)))
        }
        MappingKeyUnitClass::IndexNode => Some(BirthTreeInTheHeader::FieldStillOpenUnderC289),
        MappingKeyUnitClass::Unregistered(_) => None,
    }
}

/// 中央映射 key 的类标签（D19（块指针的结构与宽度预算） 已定项 6：取被命名对象在 D18（块里携带什么信息） 已定项 11 登记表里的码）
/// 在走读里的读法：盘上读来的一字节，语义上允许未知值，做成显式成员（`code-discipline.md`「分支：穷举写全，不写通配臂」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MappingKeyUnitClass {
    DataUnit,
    IndexNode,
    PackedRecordUnit,
    /// 码 0 与登记表之外的码：按 I-1.6（类标签登记且三处相等） 判损坏。
    Unregistered(u8),
}

impl MappingKeyUnitClass {
    const fn of(tag: u8) -> Self {
        match tag {
            1 => Self::DataUnit,
            2 => Self::IndexNode,
            3 => Self::PackedRecordUnit,
            unregistered => Self::Unregistered(unregistered),
        }
    }
}

/// 一个被引用单元的头里，I-1.2（块头写序已发布） 的判定输入：诞生代号 b、写序 (i, n) 与出生序号。
/// 落点按 D18（块里携带什么信息） 已定项 7 的字段表（码 1：诞生代号 75、写序 91；码 2：诞生代号 52 + 2k、
/// 写序 68 + 2k、出生序号 72 + 2k）与已定项 11 的类身份段表（码 3：诞生代号 73、写序 93、出生序号 103）各解一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BirthIdentityInTheHeader {
    write_order: WriteOrderUnderThePublishedPredicate,
    tail: TailOfTheBirthIdentity,
}

/// 从一个被引用单元的头里取出 I-1.2 的判定输入。类标签不在 1 / 2 / 3 里的没有这一格（那一格归 I-1.6（类标签登记且三处相等））。
fn birth_identity_in_the_header(unit: &[u8]) -> Option<BirthIdentityInTheHeader> {
    match unit[6] {
        1 => Some(BirthIdentityInTheHeader {
            write_order: WriteOrderUnderThePublishedPredicate::DataUnit {
                birth_txg: read_u64(unit, 75),
                instance: read_u32(unit, 91),
                transaction: read_six_byte_unsigned(unit, 95),
            },
            tail: TailOfTheBirthIdentity::TransactionNumber(read_six_byte_unsigned(unit, 95)),
        }),
        2 => {
            let key_width = usize::from(unit[51]);
            Some(BirthIdentityInTheHeader {
                write_order: WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
                    birth_txg: read_u64(unit, 52 + 2 * key_width),
                    instance: read_u32(unit, 68 + 2 * key_width),
                },
                tail: TailOfTheBirthIdentity::BirthSequence(read_u32(unit, 72 + 2 * key_width)),
            })
        }
        3 => Some(BirthIdentityInTheHeader {
            write_order: WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
                birth_txg: read_u64(unit, 73),
                instance: read_u32(unit, 93),
            },
            tail: TailOfTheBirthIdentity::BirthSequence(read_u32(unit, 103)),
        }),
        _outside_the_three_unit_classes => None,
    }
}

impl<'reader> Walk<'reader> {
    /// 遍历方向上的两格，判的是同一个被引用单元，输入也是同一份（头里的诞生代号 b、写序 (i, n)、出生序号）：
    ///
    /// ① **I-1.2（块头写序已发布）**：头里的这几段与**引用它的那条指针**记的相同。两处不等时「头记录的写序」
    ///    就不是这个单元的真实身份，②那个谓词判的东西是假的。恢复路径今天在遍历方向判的正是这一格
    ///    （`crates/singlefs-core/src/recovery.rs` 的 `RecoveryFailure::InvariantViolated { invariant: "I-1.2" }`
    ///    四处：树根出生序号与指针不符、inode 叶的出生身份与子指针不符、inode 叶写序实例与子指针不符、
    ///    数据单元头与指针的出生身份不符），池级 checker 自己按字段表解一份，不从实现拿偏移。
    ///    **扫描方向那一半（「任一单元」都按谓词判，i > 挂载根实例 ⇒ 判损坏）池级 checker 判不了**，
    ///    理由写在 [`PublishedPredicate::holds_for`] 与 `.claude/kb/invariants.md` I-1.2 的状态列里。
    ///
    /// ② **I-4.2（无被引用未提交块）**：这几段按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的
    ///    已发布谓词判成已发布——「未提交」在本工程里就是「按那个谓词判不成已发布」，被引用的块必须已提交。
    ///    谓词与 I-1.8（归并后版本全序） 扫描方向用的是同一份 [`PublishedPredicate`]。
    ///
    /// **打包记录类型 4 的实例表单元不进这个函数**：I-1.2 那一行的「**例外**」写着它只免谓词那一半，
    /// 它的现行版本由根记录持有的物理指针唯一确定；它的出生身份与指针相等那一格在 `walk_instance_table_chain` 里单判
    /// （[`Self::judge_birth_identity_against_the_pointer`]），不进 I-4.2、也不进 I-4.9 与「判过几个被引用单元」那个数。
    ///
    /// 诞生代号记在 `placement_it_was_read_from` 下面：读回、自证过的那一份的落点，就是这个单元在走读里的身份
    /// （[`Self::read_a_unit_to_walk_and_the_placement_it_was_read_from`]）；I-4.9 那一行「按落点认，读落点上单元头里的诞生代号」。
    fn judge_birth_identity_of_a_referenced_unit(
        &mut self,
        unit: &[u8],
        pointer: &PointerView,
        placement_it_was_read_from: (u32, u64),
        what: &str,
    ) {
        let Some(identity) = birth_identity_in_the_header(unit) else {
            return;
        };
        self.referenced_units_judged_against_their_pointer += 1;
        let birth_txg = identity.write_order.birth_txg();
        self.birth_txg_of_walked_units
            .insert(placement_it_was_read_from, birth_txg);
        self.judge_unit_birth_against_an_older_candidate_root(birth_txg, what);
        self.judge_birth_identity_against_the_pointer(identity, pointer, what);
        let mount_root_instance = self.mount_root_instance;
        let mount_root_checkpoint_txg = self.mount_root_checkpoint_txg;
        let published = PublishedPredicate {
            mount_root_instance,
            mount_root_checkpoint_txg,
            instance_table_rows: &self.instance_table_rows,
        };
        let is_published = published.holds_for(identity.write_order);
        // 借的是 `instance_table_rows` 这一个字段，`judge` 要的是 `judgements` 那一个：两处不重叠。
        // 不 clone：层 0 每个崩溃状态都要在每个被引用单元上走一遍这里。
        let rows = &self.instance_table_rows;
        self.judgements.judge("I-4.2", is_published, || {
            format!(
                "{what}：头里的诞生代号 {}、写序实例 {} 按已发布谓词判不成已发布（挂载根实例 {mount_root_instance}、txg {mount_root_checkpoint_txg}，实例表行 {rows:?}）——被引用的块必须是已提交的块",
                identity.write_order.birth_txg(),
                identity.write_order.instance()
            )
        });
    }

    /// I-1.2（块头写序已发布） 遍历方向那一格：头里的诞生代号、写序实例与尾段（码 1 事务号、码 2 / 码 3 出生序号）与引用它的指针记的相同。
    /// 被引用单元（[`Self::judge_birth_identity_of_a_referenced_unit`]）与实例表链上每一片（打包记录类型 4 按 I-1.2 那一行的例外只免谓词那一半，
    /// `research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行）都判它。
    fn judge_birth_identity_against_the_pointer(
        &mut self,
        identity: BirthIdentityInTheHeader,
        pointer: &PointerView,
        what: &str,
    ) {
        let matches_the_pointer = identity.write_order.birth_txg() == pointer.birth_txg
            && identity.write_order.instance() == pointer.instance
            && identity.tail.birth_identity_tail_number() == pointer.tail;
        self.judgements.judge("I-1.2", matches_the_pointer, || {
            format!(
                "{what}：头里的出生身份（诞生代号 {}、写序实例 {}、{} {}）与引用它的指针记的（诞生代号 {}、实例 {}、尾段 {}）不符",
                identity.write_order.birth_txg(),
                identity.write_order.instance(),
                identity.tail.name(),
                identity.tail.birth_identity_tail_number(),
                pointer.birth_txg,
                pointer.instance,
                pointer.tail
            )
        });
    }

    /// 一遍新的走读：判定接着 `judgements` 往下记，别的状态都是空的；挂载根取 (`mount_root_instance`, `mount_root_checkpoint_txg`)。
    fn starting_with(
        reader: &'reader dyn ImageReader,
        judgements: Judgements,
        filesystem_identifier_low: u64,
        mount_root_instance: u32,
        mount_root_checkpoint_txg: u64,
    ) -> Self {
        Walk {
            reader,
            judgements,
            filesystem_identifier_low,
            references: BTreeMap::new(),
            location_entry_checksums: BTreeMap::new(),
            visited_units: BTreeSet::new(),
            placements_referenced_in_this_version: BTreeMap::new(),
            walk_parent_unit: None,
            placements_referenced_below_a_walked_unit: BTreeMap::new(),
            inode_leaf_order_below_a_walked_unit: BTreeMap::new(),
            walk_failures: Vec::new(),
            tree_identifiers_in_tables: BTreeSet::new(),
            inode_object_birth: BTreeMap::new(),
            inode_tree_walked: false,
            largest_inode_number_in_the_inode_tree: None,
            data_unit_objects: Vec::new(),
            accounting: BTreeMap::new(),
            accounting_seen: false,
            inode_number_watermark_rows: Vec::new(),
            instance_table_rows: Vec::new(),
            abandoned_roots_filtering:
                AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable,
            mount_root_instance,
            mount_root_checkpoint_txg,
            referenced_units_judged_against_their_pointer: 0,
            version_being_walked: VersionBeingWalked::NewestRoot {
                checkpoint_txg: mount_root_checkpoint_txg,
            },
            birth_txg_of_walked_units: BTreeMap::new(),
            units_judged_against_an_older_candidate_root: 0,
            unit_of_each_placement: BTreeMap::new(),
            user_visible_tree_of_each_placement: BTreeMap::new(),
            user_visible_tree_being_walked: None,
            user_visible_tree_roots_of_this_version: Vec::new(),
            tree_table_entries_of_a_walked_tree_table: BTreeMap::new(),
            birth_txg_of_walked_instance_table_pages: BTreeMap::new(),
            versions_walked_for_the_appearance_rule: Vec::new(),
        }
    }

    /// I-4.9（非最新候选根引用的单元不晚于它）：正在走的是回退候选集里不是最新的一条根时，它引用的一个单元的诞生代号不大于这条根的
    /// `checkpoint_txg`（D16（发布语义） 已定项 7：单元先于根落盘）。最新那条根归 I-4.2（无被引用未提交块），由记录施加出来的那一版不在射程里。
    fn judge_unit_birth_against_an_older_candidate_root(&mut self, birth_txg: u64, what: &str) {
        if let VersionBeingWalked::OlderCandidateRoot { checkpoint_txg } = self.version_being_walked
        {
            self.units_judged_against_an_older_candidate_root += 1;
            self.judgements.judge(
                UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
                birth_txg <= checkpoint_txg,
                || {
                    format!(
                        "{what}：诞生代号 {birth_txg} 大于引用它的候选根的 txg {checkpoint_txg}"
                    )
                },
            );
        }
    }

    /// I-1.12（指针出生树等于被指单元头）：走读跟随的一条指针偏移 34 的出生树等于被指单元头里的出生树——码 1、码 3 取头偏移 43；
    /// 码 2 取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定，不判。inode 内部条目指着叶容器那一类由 I-9.2（条目身份与子头相符） 罩住，
    /// 调用点不在那里。
    fn judge_pointer_birth_tree_against_the_unit_header(
        &mut self,
        unit: &[u8],
        pointer: &PointerView,
        what: &str,
    ) {
        match birth_tree_in_the_header_as_the_mapping_key_names_it(unit) {
            Some(BirthTreeInTheHeader::AtOffsetFortyThree(birth_tree_in_the_header)) => {
                self.judgements.judge(
                    POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
                    pointer.birth_tree == birth_tree_in_the_header,
                    || {
                        format!(
                            "{what}：指针记的出生树 {} 与单元头里的出生树 {birth_tree_in_the_header} 不符",
                            pointer.birth_tree
                        )
                    },
                );
            }
            Some(BirthTreeInTheHeader::FieldStillOpenUnderC289) | None => {}
        }
    }

    fn note_reference(&mut self, location: &PointerLocation, span: u64, what: &str) {
        self.references
            .entry((location.device, location.slot, span))
            .or_insert_with(|| what.to_string());
        self.location_entry_checksums
            .entry((location.device, location.slot))
            .or_default()
            .insert(location.checksum);
    }

    /// 开始走一版（根环里的一条根，或由记录施加出来的一版）：这一版引用过哪些落点从空的记起。
    fn begin_walking_a_version(&mut self) {
        self.placements_referenced_in_this_version.clear();
        self.walk_parent_unit = None;
        self.user_visible_tree_being_walked = None;
        self.user_visible_tree_roots_of_this_version.clear();
    }

    /// 一版走完：把它引用过的落点、树表里几棵用户可见树的根与走读断没断记下，留给 I-3.5 在全部版本走完之后折成单元、逐对比
    /// （[`VersionPlacementsForTheAppearanceRule`]）。`walk_failures_before` 是走这一版之前走读失败记了几条。
    fn note_the_version_just_walked_for_the_appearance_rule(
        &mut self,
        instance: u32,
        checkpoint_txg: u64,
        walk_failures_before: usize,
    ) {
        self.versions_walked_for_the_appearance_rule
            .push(VersionPlacementsForTheAppearanceRule {
                instance,
                checkpoint_txg,
                placements: self
                    .placements_referenced_in_this_version
                    .keys()
                    .copied()
                    .collect(),
                user_visible_tree_roots: self.user_visible_tree_roots_of_this_version.clone(),
                references_are_complete: self.walk_failures.len() == walk_failures_before,
            });
    }

    /// 全部版本走完之后，每一版引用过的落点按此刻的折叠表折成单元（带用户可见树的标签）：I-3.5 逐对比的输入
    /// （[`VersionPlacementsForTheAppearanceRule`] 说为什么不在每版走完时折）。同一个单元由几个落点折来时，标签取落点序最大的那个落点的。
    fn versions_folded_into_units_by_the_final_fold_table(
        &self,
    ) -> Vec<VersionReferencesForTheAppearanceRule> {
        self.versions_walked_for_the_appearance_rule
            .iter()
            .map(|version| VersionReferencesForTheAppearanceRule {
                instance: version.instance,
                checkpoint_txg: version.checkpoint_txg,
                units: version
                    .placements
                    .iter()
                    .map(|placement| {
                        let unit = *self.unit_of_each_placement.get(placement).expect(
                            "这一版引用过的每个落点都经 `note_references_of_a_pointer_followed_by_the_walk` 记过（这一版或先走的那一版），折叠表只改不删",
                        );
                        (
                            unit,
                            self.user_visible_tree_of_each_placement.get(placement).copied(),
                        )
                    })
                    .collect(),
                user_visible_tree_roots: version.user_visible_tree_roots.clone(),
                references_are_complete: version.references_are_complete,
            })
            .collect()
    }

    /// 往下走 `unit` 里的指针之前调：之后跟着指针记下的落点都算在 `unit` 下面（`placements_referenced_below_a_walked_unit`）。
    /// 交回之前那一个，走完用 [`Self::stop_walking_below`] 换回去；两次调用之间的循环不许提前返回。
    fn start_walking_below(&mut self, unit: (u32, u64)) -> Option<(u32, u64)> {
        self.walk_parent_unit.replace(unit)
    }

    fn stop_walking_below(&mut self, previous_parent_unit: Option<(u32, u64)>) {
        self.walk_parent_unit = previous_parent_unit;
    }

    /// I-5.1（物理范围不重叠） 的「同一版里第二次引用同一个落点」那一格：这一版引用过它就判红，没有就记下是谁第一次引用它。
    fn judge_a_placement_referenced_in_this_version(&mut self, placement: (u32, u64), what: &str) {
        match self.placements_referenced_in_this_version.get(&placement) {
            Some(first_reference_in_this_version) => {
                self.judgements.judge("I-5.1", false, || {
                    format!(
                        "盘 {} 槽 {}：被同一版引用了两次——先是 {first_reference_in_this_version}，又是 {what}",
                        placement.0, placement.1
                    )
                });
            }
            None => {
                self.judgements.judge("I-5.1", true, String::new);
                self.placements_referenced_in_this_version
                    .insert(placement, what.to_string());
            }
        }
    }

    /// 按位置条目读一个走读要往下走的单元（每条位置条目照判 I-2.1，同 [`read_referenced_unit`]），另交回「走过」（`visited_units`）要认的落点：
    /// 读回、自证过的那一份的落点，不是第 0 条位置条目的（攻方 F-KEY，判决 `research/prompts/m3-batch31-code-r1-main-verification.md`
    /// 第二节 K2 观察 O3 那一行、第三节第 2 条；同实例表链那一处 `placement_it_was_read_from` 的写法）。第 0 条位置条目指着这一遍先走过的
    /// 别的单元、第 1 条那一份完好时，按第 0 条认会把读回来的这一份当成走过、它的头与格式（I-1.3、出生身份、I-1.7）一个字节都不判。
    /// 交回的落点就是这个单元在走读里的身份：走过、诞生代号那张表、并进前一版、父单元（它下面记的落点）、inode 叶序与 I-3.5 的折叠表
    /// 一律按它认（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 1 条，攻方 allrbmap：少改一把键就有一组形翻面）。
    /// 折叠表在这里改：读回的是第 1 条那一份、那个落点此刻还折在第 0 条落点那个单元上时，改记成它自己。调用方先记过引用
    /// （[`Self::note_references_of_a_pointer_followed_by_the_walk`]）再调它。
    fn read_a_unit_to_walk_and_the_placement_it_was_read_from(
        &mut self,
        pointer: &PointerView,
        unit_bytes: usize,
        what: &str,
    ) -> Option<(Vec<u8>, (u32, u64))> {
        let (unit, location_it_was_read_from) =
            crate::image::read_referenced_unit_with_the_location_it_was_read_from(
                self.reader,
                &mut self.judgements,
                &pointer.locations,
                unit_bytes,
                what,
            )?;
        let placement_it_was_read_from = (
            location_it_was_read_from.device,
            location_it_was_read_from.slot,
        );
        let first_placement = (pointer.locations[0].device, pointer.locations[0].slot);
        if placement_it_was_read_from != first_placement
            && self.unit_of_each_placement.get(&placement_it_was_read_from)
                == Some(&first_placement)
        {
            self.unit_of_each_placement
                .insert(placement_it_was_read_from, placement_it_was_read_from);
        }
        Some((unit, placement_it_was_read_from))
    }

    /// 一个单元别的版本已经走过（`visited_units` 命中），这一版又引用它：不重走，把它下面的落点（孩子、孩子的孩子……，
    /// 按 `placements_referenced_below_a_walked_unit` 那张表）逐个并进这一版的集合再比交叉——老一版在共享子树之外又指了子树里面的
    /// 某个单元，就在这里红（实审 B1 留下的缺口）。一个单元的孩子只展开一次（表里有环时也走得完：每个单元至多进一次 `expanded`）。
    fn merge_placements_below_a_unit_walked_by_an_earlier_version(
        &mut self,
        unit_walked_earlier: (u32, u64),
    ) {
        let mut expanded: BTreeSet<(u32, u64)> = BTreeSet::new();
        let mut pending = vec![unit_walked_earlier];
        while let Some(unit) = pending.pop() {
            if !expanded.insert(unit) {
                continue;
            }
            if let Some(birth_txg) = self.birth_txg_of_walked_units.get(&unit).copied() {
                self.judge_unit_birth_against_an_older_candidate_root(
                    birth_txg,
                    &format!("别的版本先走过的盘 {} 槽 {} 的单元", unit.0, unit.1),
                );
            }
            let Some(below) = self.placements_referenced_below_a_walked_unit.get(&unit) else {
                continue;
            };
            for (placement, what) in below.clone() {
                self.judge_a_placement_referenced_in_this_version(placement, &what);
                pending.push(placement);
            }
        }
    }

    /// 走读跟着一条指针往下（树里的指针、根记录里的指针、实例表的链指针；中央映射条目不走这里）：每条位置条目记进引用集合，
    /// 并判 **I-5.1（物理范围不重叠）** 的「同一版里第二次引用同一个落点」那一格——两个 inode 的 extent 指着同一个数据单元、
    /// 两条子指针指着同一个节点，都在这里红。一条指针自己的两条位置条目落在同一个 (设备, 槽) 上是一个引用、不算两次
    /// （两条位置条目设备身份相同那一格归 I-2.5（位置条目按设备身份升序））。
    fn note_references_of_a_pointer_followed_by_the_walk(
        &mut self,
        pointer: &PointerView,
        span: u64,
        what: &str,
    ) {
        let unit = (pointer.locations[0].device, pointer.locations[0].slot);
        for (index, location) in pointer.locations.iter().enumerate() {
            self.note_reference(location, span, what);
            let placement = (location.device, location.slot);
            self.unit_of_each_placement.entry(placement).or_insert(unit);
            if let Some(tree) = self.user_visible_tree_being_walked {
                self.user_visible_tree_of_each_placement
                    .entry(placement)
                    .or_insert(tree);
            }
            let is_an_earlier_location_entry_of_this_pointer = pointer.locations[..index]
                .iter()
                .any(|earlier| (earlier.device, earlier.slot) == placement);
            if is_an_earlier_location_entry_of_this_pointer {
                continue;
            }
            self.judge_a_placement_referenced_in_this_version(placement, what);
            if let Some(parent_unit) = self.walk_parent_unit {
                self.placements_referenced_below_a_walked_unit
                    .entry(parent_unit)
                    .or_default()
                    .push((placement, what.to_string()));
            }
        }
    }

    /// 共同前缀与类身份段：I-1.6（类标签登记且三处相等）、I-2.4（头校验和覆盖范围）、I-2.3（补齐为零且在载荷 CRC 内）、I-1.4（fsid）。
    /// I-2.4 判三格，三种码一样判（实审 A3-checker-2）：头校验和；偏移 4 的格式版本是这一版读得了的那一版（头校验和之后判，
    /// 版本在它的覆盖内）；明文头之后那 29 字节 nonce / MAC / 算法类型预留位全 0（载荷 CRC 过了之后判，它们在载荷 CRC 的覆盖内，
    /// 与 `check_unit` 同一个次序；「恒 0、读者遇到非 0 一律判该结构损坏」）。
    /// 返回头能不能用：头校验和过、类标签对、格式版本认得、预留位全 0（载荷 CRC 不过时预留位不判，那一格 I-2.3 已红）。
    fn judge_unit_header(&mut self, unit: &[u8], expected_class: u8, what: &str) -> bool {
        let class = unit[6];
        let copy_matches = match class {
            1 | 3 => unit[42] == class,
            _ => true,
        };
        self.judgements.judge(
            "I-1.6",
            matches!(class, 1..=3) && class == expected_class && unit[7] == 0 && copy_matches,
            || {
                format!(
                    "{what}：类标签 {class}（要 {expected_class}）、flags {}、类身份段首字节 {}",
                    unit[7], unit[42]
                )
            },
        );
        let (header_end, payload_start, declared, payload_crc_offset, fsid_offset) =
            match expected_class {
                1 => (
                    105usize,
                    134usize,
                    usize::from(read_u16(unit, 8)),
                    101usize,
                    83usize,
                ),
                3 => (107, 136, usize::from(read_u16(unit, 8)), 89, 81),
                _ => {
                    let key_width = usize::from(unit[51]);
                    (
                        86 + 2 * key_width,
                        115 + 2 * key_width,
                        usize::from(read_u16(unit, 8)),
                        76 + 2 * key_width,
                        60 + 2 * key_width,
                    )
                }
            };
        let header_holds = unit.len() > header_end && checksum_field_holds(unit, header_end, 10);
        self.judgements.judge("I-2.4", header_holds, || {
            format!("{what}：头校验和不罩 [0, {header_end}) 或对不上")
        });
        if !header_holds || class != expected_class {
            self.walk_failures.push(format!("{what} 的头用不了"));
            return false;
        }
        let format_version = read_u16(unit, FORMAT_VERSION_OFFSET);
        let format_version_is_recognized = format_version == UNIT_FORMAT_VERSION_THIS_CHECKER_READS;
        self.judgements
            .judge("I-2.4", format_version_is_recognized, || {
                format!(
                    "{what}：头里的格式版本 {format_version} 不是这一版读得了的 {UNIT_FORMAT_VERSION_THIS_CHECKER_READS}，不按今天的字段表往下解"
                )
            });
        if !format_version_is_recognized {
            self.walk_failures
                .push(format!("{what} 的头用不了（格式版本不认得）"));
            return false;
        }
        let padding_start = payload_start + declared;
        let padding_zero =
            padding_start <= unit.len() && unit[padding_start..].iter().all(|byte| *byte == 0);
        let payload_crc_holds =
            read_u32(unit, payload_crc_offset) == crc32_castagnoli_table(&unit[header_end..]);
        self.judgements
            .judge("I-2.3", padding_zero && payload_crc_holds, || {
                format!(
                    "{what}：声明长度 {declared} 之后的补齐{}、载荷 CRC{}",
                    if padding_zero { "为 0" } else { "非 0" },
                    if payload_crc_holds { "对" } else { "不对" }
                )
            });
        let fsid = read_u64(unit, fsid_offset);
        self.judgements
            .judge("I-1.4", fsid == self.filesystem_identifier_low, || {
                format!("{what}：头里的 fsid {fsid:#x} 与系统配置的低 8 字节不符")
            });
        if !payload_crc_holds {
            return true;
        }
        // 预留位不看系统配置里的加密类型就判恒 0：这一版 checker 只收加密类型 0 的系统配置，它判的池都是加密关着的。
        let reserved_bytes = usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29");
        let reserved_bytes_are_zero = unit[header_end..header_end + reserved_bytes]
            .iter()
            .all(|byte| *byte == 0);
        self.judgements.judge("I-2.4", reserved_bytes_are_zero, || {
            format!("{what}：明文头之后那 29 字节 nonce / MAC / 算法类型预留位有非 0 字节，加密关着时它们恒 0")
        });
        if !reserved_bytes_are_zero {
            self.walk_failures
                .push(format!("{what} 的头用不了（加密预留位非 0）"));
            return false;
        }
        true
    }

    /// 解一个头已经判过能用的码 2 节点（`index_node_view`）；解不开记走读失败。解不开的理由是「条目宽 0 而条目数非 0」、
    /// 这棵树又登记了条目字段表时，再报 **I-1.10（码 2 条目宽等于字段表宽）**：宽 0 不是任何登记宽，而声明长度 0 过得了
    /// 「= 条目数 × 条目宽」，走读里按宽判 I-1.10 的那几处在解码之后，碰不到这一格（实审 A3-checker-2，主 agent 定）。
    fn index_node_view_judging_a_zero_entry_width(
        &mut self,
        unit: &[u8],
        entry_field_table_registration: EntryFieldTableRegistration,
        what: &str,
    ) -> Option<crate::IndexNodeView> {
        match index_node_view(unit) {
            Ok(view) => Some(view),
            Err(refusal) => {
                if refusal == Verdict::EntryWidthZeroWithEntries {
                    match entry_field_table_registration {
                        EntryFieldTableRegistration::Registered => {
                            self.judgements.judge("I-1.10", false, || {
                                format!(
                                    "{what}：头里自述的条目宽 0 而条目数不是 0，0 不是这棵树登记的条目字段表宽度"
                                )
                            });
                        }
                        EntryFieldTableRegistration::NotRegistered => {}
                    }
                }
                self.walk_failures.push(format!("{what} 的条目区解不开"));
                None
            }
        }
    }

    /// 读一个码 2 节点并判头、树 ID（I-1.3）与 key 区间（I-1.1）。`key_range_reading` 说内部节点的 key 区间怎么判：
    /// 首末两条条目的 key（inode 树根、树表等今天的写法），或多层码 2 树的子树覆盖区间（孩子读回来之后由走读那一方判）。
    /// 「走过」、诞生代号与并进前一版都按读回那一份的落点认（同 [`Self::read_a_unit_to_walk_and_the_placement_it_was_read_from`]），
    /// 「先走过」作为一种结局交回调用方（[`IndexNodeReading`]）。
    fn read_index_node(
        &mut self,
        pointer_bytes: &[u8],
        expected_tree: u64,
        schema: crate::KeySchema,
        what: &str,
        key_range_reading: KeyRangeReading,
        entry_field_table_registration: EntryFieldTableRegistration,
    ) -> IndexNodeReading {
        let pointer = parse_node_pointer(pointer_bytes);
        if pointer.all_zero {
            return IndexNodeReading::NotWalkable;
        }
        judge_location_order(&mut self.judgements, &pointer, what);
        judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &pointer, what);
        judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
            &mut self.judgements,
            &pointer,
            what,
        );
        judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, what);
        self.note_references_of_a_pointer_followed_by_the_walk(&pointer, 1, what);
        let Some((unit, placement_it_was_read_from)) = self
            .read_a_unit_to_walk_and_the_placement_it_was_read_from(&pointer, node_bytes(), what)
        else {
            self.walk_failures
                .push(format!("{what} 两份都读不到对得上的"));
            return IndexNodeReading::NotWalkable;
        };
        if !self.visited_units.insert(placement_it_was_read_from) {
            self.merge_placements_below_a_unit_walked_by_an_earlier_version(
                placement_it_was_read_from,
            );
            return IndexNodeReading::WalkedByAnEarlierVersion {
                placement_it_was_read_from,
            };
        }
        if !self.judge_unit_header(&unit, 2, what) {
            return IndexNodeReading::NotWalkable;
        }
        self.judge_birth_identity_of_a_referenced_unit(
            &unit,
            &pointer,
            placement_it_was_read_from,
            what,
        );
        let Some(view) = self.index_node_view_judging_a_zero_entry_width(
            &unit,
            entry_field_table_registration,
            what,
        ) else {
            return IndexNodeReading::NotWalkable;
        };
        self.judgements
            .judge("I-1.3", view.tree_identifier == expected_tree, || {
                format!(
                    "{what}：头里的树 ID {} 不是引用它的树 {expected_tree}",
                    view.tree_identifier
                )
            });
        let keys = match (key_range_reading, view.level) {
            (KeyRangeReading::FirstAndLastEntries, _)
            | (KeyRangeReading::SubtreeCoverageOfInternalNodesJudgedByTheCaller, 0) => {
                check_index_node_keys(&view, schema)
            }
            (KeyRangeReading::PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller, 0)
                if view.entries.is_empty() =>
            {
                if schema.width() == view.key_width {
                    Ok(())
                } else {
                    Err(Verdict::KeyWidthMismatch)
                }
            }
            (KeyRangeReading::SubtreeCoverageOfInternalNodesJudgedByTheCaller, _)
            | (KeyRangeReading::PrescribedByThePositionJudgedByTheCaller, _)
            | (KeyRangeReading::PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller, _) => {
                check_internal_node_separators(&view, schema)
            }
        };
        self.judgements.judge("I-1.1", keys.is_ok(), || {
            format!("{what}：key 宽 / key 区间与条目不符（{keys:?}）")
        });
        IndexNodeReading::WalkedFirstByThisVersion {
            view,
            placement_it_was_read_from,
        }
    }

    fn walk_root(&mut self, record: &[u8], is_newest: bool) {
        self.begin_walking_a_version();
        let checkpoint_txg = read_u64(record, ROOT_RECORD_CHECKPOINT_TXG_OFFSET);
        self.version_being_walked = if is_newest {
            VersionBeingWalked::NewestRoot { checkpoint_txg }
        } else {
            VersionBeingWalked::OlderCandidateRoot { checkpoint_txg }
        };
        // 实例表：根记录直接持有第 0 片（自证单元，I-1.3 豁免），后面各片由前一片的链指针记录指着。
        let first_page_pointer = parse_node_pointer(&record[170..256]);
        let mount_root_instance = u32::from_le_bytes(record[24..28].try_into().expect("4 字节"));
        self.walk_instance_table_chain(
            &first_page_pointer,
            is_newest.then_some(mount_root_instance),
        );
        // 树表 0 条那一版的分配记录树的根住根记录（C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案）：
        // 那一版没有树表条目可放，也还没登记过任何树 ⇒ 那棵树的树 ID 是 0（由根记录独占持有，同树表单元与实例表单元）。
        // 按位置寻址、可以多层（D8（核心索引结构） 已定项 14）：整棵走下去。带文件的一版这一项恒全零，全零指针不走。
        if !parse_node_pointer(&record[342..428]).all_zero {
            self.walk_allocation_record_tree(&record[342..428], 0, "树表 0 条那一版的分配记录树");
        }
        // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
        let version_name = format!(
            "根（实例 {}、txg {}）",
            read_u32(record, ROOT_RECORD_INSTANCE_OFFSET),
            read_u64(record, ROOT_RECORD_CHECKPOINT_TXG_OFFSET)
        );
        self.walk_tree_table_and_central_mapping_root(
            &record[36..122],
            &record[256..342],
            &version_name,
            TreeTablePointerHolder::RootRecord,
        );
    }

    /// 由记录施加出来、根槽从没落盘的那一版（[`versions_applied_only_by_records`] 认出来的）：它没有根记录，
    /// 树表指针、映射根指针、实例表指针与「树表 0 条那一版的分配记录树的根」都在那条记录的新根段里（D23（journal 的角色与格式）
    /// 已定项 15：施加一条记录 = 把所选根的七个字段换成记录里的，用户 2026-10-02「回到甲」），从那里走下去，判定与根环里的候选根同一套。
    /// 新根段里树表指针整条全零时与根环里的根同一读法（用户 2026-10-06 定 Q1「甲 两类都判损坏」，判决
    /// `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第五节第 1 条）：core 施加记录之后拼出的根交给同一个根记录读者，
    /// 照两条位置条目读盘 0 槽 0、读不出（[`Walk::follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core`]），
    /// 主走读里这一版记一次走读失败，I-4.1 那一份另起的走读（重放施加出来的那一版也走这里）判红。
    fn walk_version_applied_only_by_records(&mut self, version: &VersionAppliedOnlyByRecords) {
        self.begin_walking_a_version();
        self.version_being_walked = VersionBeingWalked::AppliedOnlyByRecords {
            checkpoint_txg: version.checkpoint_txg,
        };
        let first_page_pointer =
            parse_node_pointer(&version.new_root_segment[NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER]);
        self.walk_instance_table_chain(&first_page_pointer, None);
        let allocation_record_tree_root =
            &version.new_root_segment[NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER];
        if !parse_node_pointer(allocation_record_tree_root).all_zero {
            self.walk_allocation_record_tree(
                allocation_record_tree_root,
                0,
                "树表 0 条那一版的分配记录树",
            );
        }
        let (tree_table_pointer, mapping_root_pointer) =
            version.tree_table_and_mapping_root_pointers();
        let version_name = format!(
            "由记录施加出来的那一版（实例 {}、txg {}）",
            version.instance, version.checkpoint_txg
        );
        self.walk_tree_table_and_central_mapping_root(
            tree_table_pointer,
            mapping_root_pointer,
            &version_name,
            TreeTablePointerHolder::NewRootSegmentOfTheRecord,
        );
    }

    /// 一版的树表（树 ID 0）与它下面每一条树表条目，再加中央映射树的根：根环里的根从根记录里取这两条指针，
    /// 由记录施加出来的那一版从记录的新根段里取（两处都是 86 字节的码 2 指针）。树表读不出时映射根也不走（走读已经断了）；
    /// 树表别的版本先走过时条目不重走，映射根照样往下走（共享的就并进这一版）。
    /// `version_name` 是这一版的称呼（哪条根、或由哪条记录施加出来），I-9.16 判红时写进第一处违例。
    /// 树表指针整条全零时两类都照 core 的根记录读者跟过去（[`Walk::follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core`]），
    /// 映射根不走；`tree_table_pointer_holder` 只用来在说明里写明这条指针是从哪里读来的。
    fn walk_tree_table_and_central_mapping_root(
        &mut self,
        tree_table_pointer: &[u8],
        mapping_root_pointer: &[u8],
        version_name: &str,
        tree_table_pointer_holder: TreeTablePointerHolder,
    ) {
        // 树表单元：不属于任何树（树 ID 0）。
        let tree_table_pointer_view = parse_node_pointer(tree_table_pointer);
        if tree_table_pointer_view.all_zero {
            self.follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core(
                &tree_table_pointer_view,
                version_name,
                tree_table_pointer_holder,
            );
            return;
        }
        let tree_table = self.read_index_node(
            tree_table_pointer,
            0,
            KEY_SCHEMA_TREE_TABLE,
            "树表单元",
            KeyRangeReading::FirstAndLastEntries,
            EntryFieldTableRegistration::NotRegistered,
        );
        match tree_table {
            IndexNodeReading::WalkedFirstByThisVersion {
                view: tree_table,
                placement_it_was_read_from: tree_table_placement,
            } => self.walk_the_entries_of_a_tree_table_walked_first_by_this_version(
                &tree_table,
                tree_table_placement,
                version_name,
            ),
            // 别的版本先走过这片树表（零单元发布、卸载与抬 F 的空发布照抄上一版的树表与映射根）：它下面的落点已经并进这一版
            // （`read_index_node` 里的 `merge_placements_below_a_unit_walked_by_an_earlier_version`），条目照先走那一版记下的判；
            // 映射根照样往下并（这一版也引用它，I-3.5 比相邻两版的引用集合要它，`research/prompts/m3-birth-floor-r3-main-verification.md`）。
            // 「先走过」按读回那一份的落点认，由 `read_index_node` 交回（第 0 条那一份对不上、第 1 条那一份被先走那一版读过时也认得出）。
            IndexNodeReading::WalkedByAnEarlierVersion {
                placement_it_was_read_from: tree_table_placement,
            } => {
                self.judge_the_tree_table_entries_an_earlier_version_walked(tree_table_placement);
            }
            // 树表读不出、头用不了：走读已经断了，映射根也不走。
            IndexNodeReading::NotWalkable => return,
        }
        // 中央映射树不进树表，引用它的是那条映射根指针（根记录里的，或由记录施加出来那一版的新根段里的）：
        // I-1.3（块头树 ID 一致） 的「实际引用它的树」按那条指针头部的出生树取（D19（块指针的结构与宽度预算） 已定项 7），
        // 与树表条目里的树 ID 对树表指着的根是同一个读法。号不写死映射树的号（mkfs 那条流上是 65540）：回退到树表 0 条的一版之后再发第一个文件版本，
        // 十二棵树从水位重新发号（C511（回退到无文件那一版之后诞生代怎么接））。
        // 映射树可以是多层（D8（核心索引结构） 已定项 11）：整棵走下去，每个节点按父条目核层级、区间与分隔 key（I-1.1）。
        // 叶的条目宽**只守走读要几个字节，不判「映射条目该有多宽」**：后者 D19（块指针的结构与宽度预算） 已定项 10 已定一律 55，
        // 但 I-1.10（码 2 条目宽等于字段表宽） 的射程里还没有中央映射树（`field_table_width_of_tree_kind` 不认种类 5），
        // 接进去欠在 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树）。
        let mapping_tree = MultiLevelTreeInTheWalk {
            tree: parse_node_pointer(mapping_root_pointer).birth_tree,
            schema: KEY_SCHEMA_MAPPING,
            leaf_entry_width: EntryWidthInTheWalk::GuardedForTheWalkOnly {
                bytes_the_walk_needs: mapping_entry_bytes(),
            },
            internal_entry_width: EntryWidthInTheWalk::GuardedForTheWalkOnly {
                bytes_the_walk_needs: internal_entry_bytes(KEY_SCHEMA_MAPPING),
            },
            name: "中央映射树",
        };
        let mut leaf_entries = Vec::new();
        self.walk_code_two_subtree(mapping_root_pointer, None, &mapping_tree, &mut leaf_entries);
        self.walk_central_mapping_entries(&leaf_entries);
    }

    /// 一版的树表指针整条全零：根环里自证过的根（用户 2026-10-05 定甲，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md`
    /// 第五节第 1 条「甲 checker 判它损坏」），以及由记录造出来的两类版本——由记录施加出来的那几版、重放施加出来的那一版
    /// （用户 2026-10-06 定 Q1「甲 两类都判损坏」，判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第五节第 1 条）。
    /// core 的根记录读者不把全零当特例，照它那两条位置条目（都是盘 0 槽 0）去读、读不出（可写挂载拒成
    /// `Recovery(UnitUnreadable { slot: SlotNumber(0) })`；施加记录之后拼出的根交给同一个读者）。checker 跟过去：两条位置条目照 I-2.1 判，
    /// 这一版的走读记一次失败（最新根红 I-7.2、I-4.8、I-7.4，别的候选根与由记录施加出来的那几版红 I-4.8、I-7.4，重放施加出来的那一版红 I-4.1），
    /// 不再读成「这一版没有树表」。
    /// 不判 I-2.5：那一行把整条全零的指针整条豁免于升序判定（用户 2026-10-06 定 Q2「乙 照今天，不红 I-2.5」）。
    fn follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core(
        &mut self,
        tree_table_pointer: &PointerView,
        version_name: &str,
        tree_table_pointer_holder: TreeTablePointerHolder,
    ) {
        let holder = match tree_table_pointer_holder {
            TreeTablePointerHolder::RootRecord => "根记录",
            TreeTablePointerHolder::NewRootSegmentOfTheRecord => "记录新根段",
        };
        let what = format!(
            "{version_name}的树表（{holder}里树表指针全零，照 core 的根记录读者读它那两条位置条目）"
        );
        let unit = read_referenced_unit(
            self.reader,
            &mut self.judgements,
            &tree_table_pointer.locations,
            node_bytes(),
            &what,
        );
        self.walk_failures.push(match unit {
            None => format!("{what} 两份都读不到对得上的"),
            Some(_) => format!("{what} 读回来的那一份不在单元区里，不是一张树表"),
        });
    }

    /// 这一版第一个走到的树表：判条目次序（I-9.16）与每一条的 previous_snapshot_txg（I-9.25），把条目里 I-9.25 与 I-3.5 读的几样记下
    /// （别的版本共享这片树表时照它判），再逐条往下走；走进用户可见树时给记下的落点打上那棵树的标签（I-3.5）。
    fn walk_the_entries_of_a_tree_table_walked_first_by_this_version(
        &mut self,
        tree_table: &crate::IndexNodeView,
        tree_table_placement: (u32, u64),
        version_name: &str,
    ) {
        judge_tree_table_entries_ordering(&mut self.judgements, &tree_table.entries, version_name);
        // I-9.27 只看同一张表里条目之间的关系、与哪一版无关：共享这片树表的别的版本不重判（与 I-9.16 同）。
        judge_tree_table_entry_head_identifiers(
            &mut self.judgements,
            &tree_table.entries,
            version_name,
        );
        let entries_of_the_walk: Vec<TreeTableEntryOfTheWalk> = tree_table
            .entries
            .iter()
            .filter_map(|entry| tree_table_entry_of_the_walk(entry))
            .collect();
        self.tree_table_entries_of_a_walked_tree_table
            .insert(tree_table_placement, entries_of_the_walk.clone());
        for entry in &entries_of_the_walk {
            self.judge_previous_snapshot_txg_of_a_tree_table_entry(*entry);
        }
        self.user_visible_tree_roots_of_this_version = entries_of_the_walk
            .iter()
            .filter_map(|entry| entry.user_visible_root)
            .collect();
        let previous_parent_unit = self.start_walking_below(tree_table_placement);
        for entry in &tree_table.entries {
            self.user_visible_tree_being_walked = tree_table_entry_of_the_walk(entry)
                .and_then(|entry_of_the_walk| entry_of_the_walk.user_visible_root)
                .map(|root| root.tree);
            self.walk_tree_table_entry(entry);
            self.user_visible_tree_being_walked = None;
        }
        self.stop_walking_below(previous_parent_unit);
    }

    /// 中央映射树叶里的条目逐条走（D19（块指针的结构与宽度预算） 已定项 6 / 10：key 27 = 类标签 1 + 出生树 8 + 出生 txg 8 +
    /// 实例代号 4 + 尾段 6，value = 位置条目 14 × 2）。入参的条目宽由走叶的那一步判过 ≥ 55（`EntryWidthInTheWalk::GuardedForTheWalkOnly`），
    /// 这里按字段表的固定偏移切。每一条：
    /// ① 两条位置条目按升序判（I-2.5）；
    /// ② key 的类标签不在登记表里（码 0、4…255）判 **映射 key 与单元头相符**（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`，用户 2026-09-27 定新立、
    ///    编号待主 agent 写回 kb 时定），那个单元不读——不知道它多长，按哪个长度读都是猜；
    /// ③ 指的物理范围（跨度按 key 的类：码 2 一槽、码 1 / 码 3 两槽）记进引用集合，参与 I-3.1（已分配统计对得上） 与 I-5.1（物理范围不重叠）：
    ///    映射是解引用与释放判定的唯一入口（D19（块指针的结构与宽度预算） 已定项 5），它指的那一段占着空间。与树里指向同一单元的那条引用
    ///    同 (设备, 起点槽, 跨度)，记成一个；不进「同一版第二次引用」那一判（映射条目与树里的指针指同一个单元是条款定的两处引用）；
    /// ④ 两份各读一遍（I-2.1），再判映射 key 与单元头相符（实审 B1 暂记在 I-1.6 / I-1.2 下的两处搬到这一条，实审 B2）：
    ///    读出来的单元头里的类标签要等于 key 的类标签；头里的出生身份（诞生代号、写序实例、尾段）要等于 key 里的
    ///    （与 `judge_birth_identity_of_a_referenced_unit` 拿指针比同一个读法）；码 2 / 码 3 的尾段是出生序号 4 字节，其后 2 字节补零恒 0；
    ///    码 1 / 码 3 的 key 里的出生树要等于单元头偏移 43 的出生树（码 1 五元组的树、码 3 类身份段的出生树）。
    ///    码 2 的出生树不比：取哪个字段还开着（C289（码 2 的出生树取哪个字段没写））。
    fn walk_central_mapping_entries(&mut self, entries: &[Vec<u8>]) {
        for entry in entries {
            let locations_pointer: Vec<u8> =
                [vec![0u8; 50], entry[27..55].to_vec(), vec![0u8; 8]].concat();
            let view = parse_node_pointer(&locations_pointer);
            judge_location_order(&mut self.judgements, &view, "映射条目");
            let key_class_tag = entry[MAPPING_KEY_UNIT_CLASS_OFFSET];
            let unit_bytes = match MappingKeyUnitClass::of(key_class_tag) {
                MappingKeyUnitClass::DataUnit | MappingKeyUnitClass::PackedRecordUnit => {
                    data_unit_bytes()
                }
                MappingKeyUnitClass::IndexNode => node_bytes(),
                MappingKeyUnitClass::Unregistered(unregistered_tag) => {
                    self.judgements.judge(MAPPING_KEY_MATCHES_THE_UNIT_HEADER, false, || {
                        format!(
                            "映射条目：key 的类标签 {unregistered_tag} 不在登记表里（1 / 2 / 3），它指的单元不读"
                        )
                    });
                    continue;
                }
            };
            let span_in_slots = u64::try_from(unit_bytes).expect("单元字节数") / SLOT_BYTES;
            for location in &view.locations {
                self.note_reference(location, span_in_slots, "映射条目指的单元");
            }
            let Some(unit) = read_referenced_unit(
                self.reader,
                &mut self.judgements,
                &view.locations,
                unit_bytes,
                "映射条目指的单元",
            ) else {
                self.walk_failures
                    .push("映射条目指的单元两份都读不到对得上的".to_string());
                continue;
            };
            let header_class_tag = unit[6];
            self.judgements.judge(
                MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
                header_class_tag == key_class_tag,
                || {
                    format!(
                        "映射条目：key 的类标签是 {key_class_tag}，它指的单元头里的类标签是 {header_class_tag}"
                    )
                },
            );
            if header_class_tag != key_class_tag {
                continue;
            }
            let identity = birth_identity_in_the_header(&unit)
                .expect("头里的类标签等于 key 的类标签、上面已判过在登记表里（1 / 2 / 3）");
            let key_birth_txg = read_u64(entry, MAPPING_KEY_BIRTH_TXG_OFFSET);
            let key_instance = read_u32(entry, MAPPING_KEY_INSTANCE_OFFSET);
            let key_tail = match identity.tail {
                TailOfTheBirthIdentity::TransactionNumber(_) => {
                    read_six_byte_unsigned(entry, MAPPING_KEY_TAIL_OFFSET)
                }
                TailOfTheBirthIdentity::BirthSequence(_) => {
                    u64::from(read_u32(entry, MAPPING_KEY_TAIL_OFFSET))
                }
            };
            let matches_the_key = identity.write_order.birth_txg() == key_birth_txg
                && identity.write_order.instance() == key_instance
                && identity.tail.birth_identity_tail_number() == key_tail;
            self.judgements
                .judge(MAPPING_KEY_MATCHES_THE_UNIT_HEADER, matches_the_key, || {
                    format!(
                        "映射条目：key 里的出生身份（诞生代号 {key_birth_txg}、实例 {key_instance}、尾段 {key_tail}）与它指的单元头里的（诞生代号 {}、写序实例 {}、{} {}）不符",
                        identity.write_order.birth_txg(),
                        identity.write_order.instance(),
                        identity.tail.name(),
                        identity.tail.birth_identity_tail_number()
                    )
                });
            match identity.tail {
                TailOfTheBirthIdentity::TransactionNumber(_) => {}
                TailOfTheBirthIdentity::BirthSequence(_) => {
                    let padding = &entry[MAPPING_KEY_TAIL_PADDING];
                    self.judgements.judge(
                        MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
                        padding.iter().all(|byte| *byte == 0),
                        || {
                            format!(
                                "映射条目：码 {key_class_tag} 的 key 尾段在出生序号之后的补零 2 字节是 {padding:?}，不是 0"
                            )
                        },
                    );
                }
            }
            let key_birth_tree = read_u64(entry, MAPPING_KEY_BIRTH_TREE_OFFSET);
            match birth_tree_in_the_header_as_the_mapping_key_names_it(&unit)
                .expect("头里的类标签等于 key 的类标签、上面已判过在登记表里（1 / 2 / 3）")
            {
                BirthTreeInTheHeader::AtOffsetFortyThree(header_birth_tree) => {
                    self.judgements.judge(
                        MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
                        header_birth_tree == key_birth_tree,
                        || {
                            format!(
                                "映射条目：码 {key_class_tag} 的 key 里的出生树 {key_birth_tree} 与它指的单元头里的出生树 {header_birth_tree} 不符"
                            )
                        },
                    );
                }
                BirthTreeInTheHeader::FieldStillOpenUnderC289 => {}
            }
        }
    }

    /// 多层码 2 树（记账树、中央映射树，D8（核心索引结构） 已定项 11）的一个节点连同它下面：节点照 `read_index_node` 判头、
    /// 出生身份（I-1.2 / I-4.2）、树 ID（I-1.3）与自己的 key（I-1.1：叶的区间贴紧首末条目、内部节点的分隔 key 严格递增），
    /// 再按引用它的父条目判它在树里的身份——**I-1.1（块头自述逻辑地址）**：索引节点的身份是树 ID + 层级 + key 区间
    /// （D18（块里携带什么信息） 已定项 2 的子树覆盖区间），全从块头自带的那几样读，不另存旁路表：
    /// ① 孩子头里的层级 = 父层级 − 1（层级 0 是叶）；
    /// ② 分隔 key_i ≤ 第 i 个孩子头里的最小 key，且 > 第 i − 1 个孩子头里的最大 key（它管的区间就是父条目给的那一段）；
    /// ③ 内部节点头里的区间 = [第一个孩子头里的最小 key, 最后一个孩子头里的最大 key]。
    /// 条目宽：叶按 `leaf_entry_width`、内部节点按 `internal_entry_width`（记账树判 I-1.10，中央映射树只守走读要几个字节）。
    /// 叶里的条目按走到的次序（= key 升序）追加进 `leaf_entries`。
    ///
    /// 迭代与递归的上界：每一层下降层级减一（①不成立的孩子不往下走），每个单元只走一次（`visited_units`）。
    fn walk_code_two_subtree(
        &mut self,
        pointer_bytes: &[u8],
        expected_level: Option<u8>,
        tree: &MultiLevelTreeInTheWalk<'_>,
        leaf_entries: &mut Vec<Vec<u8>>,
    ) -> SubtreeInTheWalk {
        let pointer = parse_node_pointer(pointer_bytes);
        let what = match expected_level {
            None => format!("{}（树 {}）的根", tree.name, tree.tree),
            Some(level) => format!("{}（树 {}）层级 {level} 的节点", tree.name, tree.tree),
        };
        if pointer.all_zero {
            if expected_level.is_some() {
                self.walk_failures
                    .push(format!("{what}：父条目里的子指针全零"));
            }
            return SubtreeInTheWalk::NotWalkable;
        }
        let (view, node_placement) = match self.read_index_node(
            pointer_bytes,
            tree.tree,
            tree.schema,
            &what,
            KeyRangeReading::SubtreeCoverageOfInternalNodesJudgedByTheCaller,
            tree.leaf_entry_width.entry_field_table_registration(),
        ) {
            IndexNodeReading::WalkedFirstByThisVersion {
                view,
                placement_it_was_read_from,
            } => (view, placement_it_was_read_from),
            IndexNodeReading::WalkedByAnEarlierVersion { .. } => {
                return SubtreeInTheWalk::AlreadyWalkedFromAnotherRoot;
            }
            IndexNodeReading::NotWalkable => return SubtreeInTheWalk::NotWalkable,
        };
        // 头里自述的 key 宽是盘上读来的一字节：与这棵树的 key 形态不等时 `read_index_node` 已经按 I-1.1 判红，这里不往下走——
        // 下面按它切子指针（`entry[key_width..key_width + 86]`）、把头里的 key 交回父节点按形态逐字段读，宽了越过条目、窄了越过 key，
        // 都是当场越界（代码审阅 6c 第 6 条）。
        if view.key_width != tree.schema.width() {
            self.walk_failures.push(format!(
                "{what}：头里自述的 key 宽 {} 不是这棵树的 key 宽 {}，这一支走不下去",
                view.key_width,
                tree.schema.width()
            ));
            return SubtreeInTheWalk::NotWalkable;
        }
        if let Some(level) = expected_level {
            self.judgements.judge("I-1.1", view.level == level, || {
                format!(
                    "{what}：头里的层级是 {}，引用它的父节点要层级 {level}（父层级减一）",
                    view.level
                )
            });
            if view.level != level {
                self.walk_failures
                    .push(format!("{what}：层级不是父层级减一，这一支走不下去"));
                return SubtreeInTheWalk::NotWalkable;
            }
        }
        let entry_width_reading = if view.level == 0 {
            tree.leaf_entry_width
        } else {
            tree.internal_entry_width
        };
        if !self.entry_width_holds(&view, entry_width_reading, &what) {
            return SubtreeInTheWalk::NotWalkable;
        }
        if view.level == 0 {
            leaf_entries.extend(view.entries.iter().cloned());
            return SubtreeInTheWalk::Walked {
                smallest_key: view.smallest_key,
                largest_key: view.largest_key,
                is_complete: true,
            };
        }
        let key_width = view.key_width;
        let fields = |key: &[u8]| tree.schema.fields(key);
        let mut is_complete = true;
        let mut children: Vec<(Vec<u8>, SubtreeInTheWalk)> = Vec::with_capacity(view.entries.len());
        let previous_parent_unit = self.start_walking_below(node_placement);
        for entry in &view.entries {
            let separator_key = entry[..key_width].to_vec();
            let child_pointer_bytes = &entry[key_width..key_width + node_pointer_bytes()];
            judge_location_order(
                &mut self.judgements,
                &parse_node_pointer(child_pointer_bytes),
                &format!("{what} 的子指针"),
            );
            let child = self.walk_code_two_subtree(
                child_pointer_bytes,
                Some(view.level - 1),
                tree,
                leaf_entries,
            );
            match &child {
                SubtreeInTheWalk::Walked {
                    is_complete: child_is_complete,
                    ..
                } => is_complete &= *child_is_complete,
                SubtreeInTheWalk::AlreadyWalkedFromAnotherRoot => {}
                SubtreeInTheWalk::NotWalkable => is_complete = false,
            }
            children.push((separator_key, child));
        }
        self.stop_walking_below(previous_parent_unit);
        // ② 两条不等式：只判读回来、这一遍第一次走到的孩子（别的根走过的那几个，那时已经按它们自己的父条目判过）。
        for (position, (separator_key, child)) in children.iter().enumerate() {
            let SubtreeInTheWalk::Walked { smallest_key, .. } = child else {
                continue;
            };
            self.judgements.judge(
                "I-1.1",
                fields(separator_key) <= fields(smallest_key),
                || format!("{what}：第 {position} 条的分隔 key 大于这个孩子头里的最小 key"),
            );
            if position == 0 {
                continue;
            }
            if let (_, SubtreeInTheWalk::Walked { largest_key, .. }) = &children[position - 1] {
                self.judgements
                    .judge("I-1.1", fields(separator_key) > fields(largest_key), || {
                        format!("{what}：第 {position} 条的分隔 key 不大于左邻孩子头里的最大 key")
                    });
            }
        }
        // ③ 头里的区间是子树覆盖区间。
        if let (
            Some((_, SubtreeInTheWalk::Walked { smallest_key, .. })),
            Some((_, SubtreeInTheWalk::Walked { largest_key, .. })),
        ) = (children.first(), children.last())
        {
            let covers = *smallest_key == view.smallest_key && *largest_key == view.largest_key;
            self.judgements.judge("I-1.1", covers, || {
                format!("{what}：头里的 key 区间不是子树覆盖区间（第一个孩子的最小 key 到最后一个孩子的最大 key）")
            });
        }
        SubtreeInTheWalk::Walked {
            smallest_key: view.smallest_key,
            largest_key: view.largest_key,
            is_complete,
        }
    }

    /// 一个多层码 2 树节点的条目宽：按 I-1.10 判（记账树），或只守走读要几个字节（中央映射树）；不成立就不往下切。
    fn entry_width_holds(
        &mut self,
        view: &crate::IndexNodeView,
        reading: EntryWidthInTheWalk,
        what: &str,
    ) -> bool {
        match reading {
            EntryWidthInTheWalk::JudgedAsInvariantOneTen { field_table_bytes } => {
                let entry_width = view.entry_width;
                self.judgements
                    .judge("I-1.10", entry_width == field_table_bytes, || {
                        format!(
                            "{what}：头里自述的条目宽 {entry_width} 不等于这一层的条目字段表宽度 {field_table_bytes}"
                        )
                    });
                entry_width == field_table_bytes
            }
            EntryWidthInTheWalk::GuardedForTheWalkOnly {
                bytes_the_walk_needs,
            } => {
                if view.entry_width < bytes_the_walk_needs {
                    self.walk_failures.push(format!(
                        "{what}自述的条目宽 {} 小于走读要的 {bytes_the_walk_needs} 字节，这一支走不下去",
                        view.entry_width
                    ));
                    return false;
                }
                true
            }
        }
    }

    /// 树表的一条条目：条目宽是树表单元头里的一个**盘上字段**（`index_node_view` 只判了它 ≥ key 宽 8），而下面三步——
    /// 读树 ID（偏移 0）、读种类（偏移 10）、切根指针（`entry[14..100]`）——都按字段表的固定偏移切，窄一个字节就当场越界。
    /// 切之前先判一次，窄的如实记一条「走读断了」（I-4.8（近 K 代根校验和自洽） / I-7.4（近 K 代块未被复用） 由它说话），
    /// **不判违例**：I-1.10（码 2 条目宽等于字段表宽） 判的是「树表指着的那棵树的根节点」的条目宽、住在这两处切片之后，
    /// 罩不到树表自己（C504（树表条目宽在走读里无守卫，今天没坏法打得到），2026-09-23 用户定案「只加走读守卫，不动条款」，
    /// I-1.10（码 2 条目宽等于字段表宽） 的射程一个字不扩）。守卫与 `references_under_root` /
    /// `allocation_record_count_under_root` 那两处同款。
    fn walk_tree_table_entry(&mut self, entry: &[u8]) {
        if entry.len() < tree_table_entry_bytes() {
            self.walk_failures.push(format!(
                "树表单元自述的条目宽 {} 小于走读要的 {} 字节，树表条目这一支走不下去",
                entry.len(),
                tree_table_entry_bytes()
            ));
            return;
        }
        let tree = read_u64(entry, 0);
        let kind = read_u16(entry, 10);
        self.tree_identifiers_in_tables.insert(tree);
        let what = format!("树 {tree}（种类 {kind}）的根");
        let Some(schema) = key_schema_for_tree_kind(kind) else {
            return;
        };
        if kind == TREE_KIND_INODE {
            let pointer = parse_node_pointer(&entry[14..100]);
            if !pointer.all_zero {
                if let Some(bytes) = self.reader.read(
                    pointer.locations[0].device,
                    pointer.locations[0].slot * SLOT_BYTES,
                    node_bytes(),
                ) {
                    self.judgements.judge("I-9.1", bytes[6] == 2, || {
                        format!("树表指向的 inode 树根类标签是 {}，不是 2", bytes[6])
                    });
                }
            }
        }
        // 记账树可以是多层（D8（核心索引结构） 已定项 11）：整棵走下去，每个节点按父条目判层级、区间与分隔 key（I-1.1），
        // 条目宽按层判 I-1.10（叶 34、内部节点 108 = key 22 + 子指针 86）。记账行只取走全了的那一遍：有一个节点读不出、
        // 头用不了或条目宽不对，行就数不全，I-3.1 那几条按「这一版没有记账可比」报不适用，不拿半张账去比。
        if kind == TREE_KIND_ACCOUNTING {
            let accounting_tree = MultiLevelTreeInTheWalk {
                tree,
                schema,
                leaf_entry_width: EntryWidthInTheWalk::JudgedAsInvariantOneTen {
                    field_table_bytes: field_table_width_of_tree_kind(TREE_KIND_ACCOUNTING)
                        .expect("记账树登记了条目字段表"),
                },
                internal_entry_width: EntryWidthInTheWalk::JudgedAsInvariantOneTen {
                    field_table_bytes: internal_entry_bytes(schema),
                },
                name: "记账树",
            };
            let mut rows = Vec::new();
            if let SubtreeInTheWalk::Walked {
                is_complete: true, ..
            } = self.walk_code_two_subtree(&entry[14..100], None, &accounting_tree, &mut rows)
            {
                self.accounting_seen = true;
                // I-3.13：记账代 = checkpoint 号（D16（发布语义） 已定项 6），每条根的记账树只装这条根那一代的行；只判走全了的树。
                // inode 号水位那一统计量的台阶行例外（D5（快照 / 空间记账机制） 已定项 2 射程「I-3.13 对台阶行同样不适用」）：
                // 那一统计量、不带设备维、代段早于这一版 txg 的行就是台阶行，跳过；它代段等于这一版 txg 的那一行与别的统计量照判。
                let accounting_version_txg = self.version_being_walked.checkpoint_txg();
                for row in &rows {
                    let row_generation = read_u64(row, ACCOUNTING_KEY_GENERATION_OFFSET);
                    let is_an_inode_number_watermark_step = read_u16(row, 0)
                        == STATISTIC_INODE_WATERMARK
                        && read_u32(row, ACCOUNTING_ROW_DEVICE_OFFSET)
                            == STATISTIC_NO_DEVICE_DIMENSION
                        && row_generation < accounting_version_txg;
                    if is_an_inode_number_watermark_step {
                        continue;
                    }
                    self.judgements.judge(
                        ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
                        row_generation == accounting_version_txg,
                        || {
                            format!(
                                "记账树（树 {tree}）里统计量 {}、盘 {} 那一行的代段 {row_generation} 不是这一版的 txg {accounting_version_txg}",
                                read_u16(row, 0),
                                read_u32(row, 10)
                            )
                        },
                    );
                }
                for row in &rows {
                    self.accounting
                        .insert((read_u16(row, 0), read_u32(row, 10)), read_u64(row, 22));
                    if read_u16(row, 0) == STATISTIC_INODE_WATERMARK
                        && read_u32(row, ACCOUNTING_ROW_DEVICE_OFFSET)
                            == STATISTIC_NO_DEVICE_DIMENSION
                    {
                        self.inode_number_watermark_rows.push((
                            read_u64(row, ACCOUNTING_KEY_GENERATION_OFFSET),
                            read_u64(row, ACCOUNTING_ROW_VALUE_OFFSET),
                        ));
                    }
                }
            }
            return;
        }
        // 分配记录树与 extent 树按 key 空间定形状（D8（核心索引结构） 已定项 14）：整棵按位置走下去，每个节点核它罩的就是它的位置规定的那一段。
        if kind == TREE_KIND_ALLOCATION {
            if !parse_node_pointer(&entry[14..100]).all_zero {
                self.walk_allocation_record_tree(&entry[14..100], tree, "分配记录树");
            }
            return;
        }
        if kind == TREE_KIND_EXTENT {
            if !parse_node_pointer(&entry[14..100]).all_zero {
                self.walk_extent_upper_node(&entry[14..100], None, tree);
            }
            return;
        }
        let IndexNodeReading::WalkedFirstByThisVersion {
            view: node,
            placement_it_was_read_from: node_placement,
        } = self.read_index_node(
            &entry[14..100],
            tree,
            schema,
            &what,
            KeyRangeReading::FirstAndLastEntries,
            match field_table_width_of_tree_kind(kind) {
                Some(_) => EntryFieldTableRegistration::Registered,
                None => EntryFieldTableRegistration::NotRegistered,
            },
        )
        else {
            return;
        };
        // I-1.10：头里自述的条目宽等于这棵树登记的条目字段表宽度，**先于**按字段表解条目判
        // （与 I-1.7（打包容器合法与判定顺序） 判码 3 记录宽同一纪律）。不等就不往下解：下面每一支都按
        // 24 / 34 / 112 / 120 这些固定偏移切条目，窄一个字节就当场越界（普查 R13）。
        // 中央映射树（种类 5）走不到这里判：`field_table_width_of_tree_kind` 里没有它；条目宽已定 55（D19（块指针的结构与宽度预算） 已定项 10），
        // 把种类 5 接进这一判欠在 C307。
        if let Some(entry_field_table_bytes) = field_table_width_of_tree_kind(kind) {
            let entry_width = node.entry_width;
            self.judgements
                .judge("I-1.10", entry_width == entry_field_table_bytes, || {
                    format!(
                        "{what}：头里自述的条目宽 {entry_width} 不等于这棵树的条目字段表宽度 {entry_field_table_bytes}"
                    )
                });
            if entry_width != entry_field_table_bytes {
                return;
            }
        }
        if kind == TREE_KIND_INODE {
            self.walk_inode_root(&node, tree, node_placement);
        }
        // 墓碑树、谱系树：码 2 根之下是码 3 容器（类型段墓碑 {0, 1}、谱系 {0, 5}，D18（块里携带什么信息） 已定项 24）。
        // 这里走到每片容器、判每条内部条目的身份（墓碑树 I-9.23、谱系树 I-9.17）、容器的头与容器格式（单元算进可达与已有统计）；
        // 记录本身的那几条（I-9.18–I-9.22、I-9.24、I-9.29 与 I-9.10 墓碑那一个等号）在删除态扫描里每一版各判。
        if kind == TREE_KIND_TOMBSTONE {
            self.walk_container_tree_root(
                &node,
                node_placement,
                tree,
                PACKED_TYPE_TOMBSTONE,
                TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
                "墓碑树",
            );
        }
        if kind == TREE_KIND_LINEAGE {
            self.walk_container_tree_root(
                &node,
                node_placement,
                tree,
                PACKED_TYPE_LINEAGE,
                LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
                "谱系树",
            );
        }
        // 退役树（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5、8 条）：形态照墓碑树，类型段 {0, 3}，
        // 每条内部条目按 I-9.31 判身份；记录那几条（I-9.32–I-9.35）在墓碑与谱系那一遍里每一版各判。
        if kind == TREE_KIND_RETIREMENT {
            self.walk_container_tree_root(
                &node,
                node_placement,
                tree,
                PACKED_TYPE_RETIREMENT,
                RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
                "退役树",
            );
        }
    }

    /// I-9.25：`previous_snapshot_txg` 只写在 inode 树那条条目里，0 = 不适用；inode 树那条不大于这一版的 txg
    /// （D5（快照 / 空间记账机制） 已定项 9；「≤ 根 txg」那一半用户 2026-10-01 补定）。走到的每一版（候选集里每条，含由记录施加出来的）
    /// 拿自己的 txg 判自己树表里的每一条：树表被几版共享时每一版各判一次（[`Self::judge_the_tree_table_entries_an_earlier_version_walked`]），
    /// 不只在第一个走到它的那一版判（主 agent 2026-10-03 定改回 kb 字面）。
    fn judge_previous_snapshot_txg_of_a_tree_table_entry(
        &mut self,
        entry: TreeTableEntryOfTheWalk,
    ) {
        let version_txg = self.version_being_walked.checkpoint_txg();
        let TreeTableEntryOfTheWalk {
            tree,
            kind,
            previous_snapshot_txg,
            ..
        } = entry;
        let previous_snapshot_txg_holds = if kind == TREE_KIND_INODE {
            previous_snapshot_txg <= version_txg
        } else {
            previous_snapshot_txg == 0
        };
        self.judgements.judge(
            PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
            previous_snapshot_txg_holds,
            || {
                format!(
                    "树 {tree}（种类 {kind}）的树表条目 previous_snapshot_txg {previous_snapshot_txg}：不是 inode 树的条目恒 0，inode 树的不大于这一版的 txg {version_txg}"
                )
            },
        );
    }

    /// 这一版的树表单元别的版本先走过（`visited_units`）：不重读盘，照先走那一版记下的条目判这一版的 I-9.25，
    /// 这一版树表里几棵用户可见树的根也照它取（I-3.5）。先走那一版头用不了、没记下条目的，这里没有对象。
    fn judge_the_tree_table_entries_an_earlier_version_walked(
        &mut self,
        tree_table_placement: (u32, u64),
    ) {
        let Some(entries) = self
            .tree_table_entries_of_a_walked_tree_table
            .get(&tree_table_placement)
            .cloned()
        else {
            return;
        };
        for entry in &entries {
            self.judge_previous_snapshot_txg_of_a_tree_table_entry(*entry);
        }
        self.user_visible_tree_roots_of_this_version = entries
            .iter()
            .filter_map(|entry| entry.user_visible_root)
            .collect();
    }

    /// 墓碑树或谱系树的码 2 根（或根长高之后的内部节点）往下走：条目 120 字节（分隔 key 8 + 身份引用 26 + 子指针 86），
    /// 类型段 = `record_type` 的走到码 3 容器（判容器头、出生树、容器格式，单元记进这一版走到的落点），类型段 0 的走进下一层码 2 节点
    /// （层级 = 父层级 − 1），别的类型段判 `identity_invariant` 红、不往下走；孩子头里自证过的类标签与类型段说的那一类不符的同样判红、
    /// 不往下走（[`Self::judge_a_container_tree_entry_whose_child_is_of_another_class`]，孩子读多长按它头里的类标签定、不按类型段定）——
    /// 另有一份头里就是类型段那一类、按那一类的长度自证得过的除外：判红之后照类型段那一支走那一份（F-W，
    /// [`some_copy_of_the_child_verifies_as_the_class_the_entry_type_names`]）；
    /// 这两支不往下走的孩子每一份副本照判 I-2.1（[`Self::judge_every_copy_of_a_child_the_walk_does_not_go_into`]）。每条条目的身份按 `identity_invariant`（墓碑树 I-9.23、
    /// 谱系树 I-9.17）单独判定，身份不符不记走读失败（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T7 取 a）。
    /// 用 checker 自己的读法，不调 `singlefs_core`（D13（验证路线） 已定项 5）。
    /// 递归的上界：每进一层类型段 0，层级减一（层级 0 的码 2 节点在这两棵树里没有条款定的形状，记走读失败）。
    ///
    /// 往下记的落点都记在 `node_placement` 这个节点下面（`placements_referenced_below_a_walked_unit`）：别的版本共享这个节点时
    /// 照这张表把它下面的容器并进那一版（I-5.1 与 I-3.5 都要），不记在外面那片树表下面。
    fn walk_container_tree_root(
        &mut self,
        node: &crate::IndexNodeView,
        node_placement: (u32, u64),
        tree: u64,
        record_type: u16,
        identity_invariant: &'static str,
        name: &str,
    ) {
        let previous_parent_unit = self.start_walking_below(node_placement);
        self.walk_the_entries_of_a_container_tree_node(
            node,
            tree,
            record_type,
            identity_invariant,
            name,
        );
        self.stop_walking_below(previous_parent_unit);
    }

    /// [`Self::walk_container_tree_root`] 那一个节点的条目逐条走（循环里有 `continue`，所以与换走读父单元那两步分开写）。
    fn walk_the_entries_of_a_container_tree_node(
        &mut self,
        node: &crate::IndexNodeView,
        tree: u64,
        record_type: u16,
        identity_invariant: &'static str,
        name: &str,
    ) {
        for entry in &node.entries {
            let entry_type = read_u16(entry, 16);
            let identity = (
                read_u64(entry, 8),
                entry_type,
                read_u64(entry, 18),
                read_u64(entry, 26),
            );
            let child = parse_node_pointer(&entry[34..120]);
            let what = format!("{name}内部条目的子指针");
            judge_location_order(&mut self.judgements, &child, &what);
            judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &child, &what);
            judge_pointer_extent_offset_is_zero(&mut self.judgements, &child, &what);
            judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
                &mut self.judgements,
                &child,
                &what,
            );
            let class_the_entry_type_names =
                match ContainerTreeEntryType::of(entry_type, record_type) {
                    ContainerTreeEntryType::InternalNode => Some(MappingKeyUnitClass::IndexNode),
                    ContainerTreeEntryType::Container => {
                        Some(MappingKeyUnitClass::PackedRecordUnit)
                    }
                    ContainerTreeEntryType::Unregistered(_) => None,
                };
            if let Some(class_the_entry_type_names) = class_the_entry_type_names {
                if let Some(child_class) = child_unit_class_other_than_the_entry_type_names(
                    self.reader,
                    &child,
                    Some(class_the_entry_type_names),
                ) {
                    // 另一份头里就是类型段那一类、按那一类的长度自证得过（F-W，判决
                    // `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「K2 H-V 两份都看之后判错」那一行、第三节第 1 条）：
                    // 读者按类型段那一类的长度逐份读、用对得上的那一份，那一份的子树照类型段那一支走、照判（槽数按类型段那一类登记）；
                    // 条目那一条照判红。没有这样一份才不往下走。
                    if some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(
                        self.reader,
                        &child,
                        class_the_entry_type_names,
                    ) {
                        self.judgements.judge(identity_invariant, false, || {
                            format!(
                                "{name}内部条目类型段 {entry_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）；另一份头里就是类型段那一类、按那一类的长度自证得过，照类型段那一支往下走那一份"
                            )
                        });
                    } else {
                        self.judge_a_container_tree_entry_whose_child_is_of_another_class(
                            &child,
                            entry_type,
                            class_the_entry_type_names,
                            child_class,
                            identity_invariant,
                            name,
                        );
                        continue;
                    }
                }
            }
            match ContainerTreeEntryType::of(entry_type, record_type) {
                ContainerTreeEntryType::InternalNode => {
                    // 「子单元是码 2」与 I-1.6（类标签登记且三处相等） 重合，照 I-9.2（条目身份与子头相符） 的先例留作合取
                    // （第二节 T7）：读不出的孩子这一小句不判，读不出归 I-2.1 与走读失败。
                    // 孩子的类只认按它自己那一类的长度整份自证过的副本（F-ID-b，用户 2026-10-07 定，判决
                    // `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 N1 盘序翻面 A3 那一行、第四节）：头里是码 3、
                    // 只有前 16 KiB 与位置条目里的校验和对得上的那一份不算，不随它在哪块盘翻面。
                    // 两份都没自证过的当读不出，「子单元是码 2」这一小句不判（用户 2026-10-07 定，`records/2026-10-01-删除与并发事务.md` 同日那一条）。
                    let (birth_tree, _, container, container_birth) = identity;
                    let child_class = class_of_the_first_copy_verifying_at_the_length_of_its_class(
                        self.reader,
                        &child,
                    );
                    let entry_holds = birth_tree == 0
                        && container == 0
                        && container_birth == 0
                        && child_class.is_none_or(|class| class == MappingKeyUnitClass::IndexNode);
                    self.judgements.judge(identity_invariant, entry_holds, || {
                        format!(
                            "{name}内部条目类型段 0：孩子自证过的类 {child_class:?}（要码 2；两份都没自证过的不判），身份引用的出生树 / 容器号 / 出生代 {birth_tree} / {container} / {container_birth}（要三段为 0）"
                        )
                    });
                    self.walk_the_internal_node_below_a_container_tree_entry(
                        node,
                        entry,
                        &child,
                        tree,
                        record_type,
                        identity_invariant,
                        name,
                    );
                }
                ContainerTreeEntryType::Container => {
                    self.walk_the_container_below_a_container_tree_entry(
                        &child,
                        identity,
                        tree,
                        record_type,
                        identity_invariant,
                        name,
                    );
                }
                ContainerTreeEntryType::Unregistered(unregistered_type) => {
                    // 先登记引用再判红：不登记的话这个孩子占的槽不进引用并集，I-3.1（已分配统计对得上） 连带红（第二节 T7）。
                    // 孩子的形状由这一层定：层级 1 的节点下面是码 3 容器（两槽），层级 ≥ 2 的下面是码 2 节点（一槽）。
                    // 孩子不往下走，每一份副本照判 I-2.1（第 29 批，H-J3a）。
                    let span_in_slots = if node.level >= 2 { 1 } else { 2 };
                    self.note_references_of_a_pointer_followed_by_the_walk(
                        &child,
                        span_in_slots,
                        &what,
                    );
                    self.judge_every_copy_of_a_child_the_walk_does_not_go_into(&child, &what);
                    self.judgements.judge(identity_invariant, false, || {
                        format!("{name}内部条目的类型段 {unregistered_type} 不在 {{0, {record_type}}} 里")
                    });
                }
            }
        }
    }

    /// 墓碑树、谱系树里一条条目的类型段说孩子是一类（类型段 0 ⇒ 码 2，类型段 = 打包记录类型 ⇒ 码 3），孩子头里自证过的类标签是另一类：
    /// I-9.23 / I-9.17 字面「0 ⇒ 子单元是码 2 … 1（谱系 5）⇒ 子单元是码 3」不成立，按 `identity_invariant` 判红，
    /// 「身份不符单独判定，不再记走读失败」：不往下走、不记走读失败（只在没有一份按类型段那一类自证过时走到这里，F-W）。孩子按它自己那一类占的槽先登记引用（同类型段不认得那一支：
    /// 不登记的话它占的槽不进引用并集，I-3.1（已分配统计对得上） 连带红）。每一份副本照判 I-2.1
    /// （[`Self::judge_every_copy_of_a_child_the_walk_does_not_go_into`]，第 29 批，H-J3b：另一份校验和对不上的照红）。
    fn judge_a_container_tree_entry_whose_child_is_of_another_class(
        &mut self,
        child: &PointerView,
        entry_type: u16,
        class_the_entry_type_names: MappingKeyUnitClass,
        child_class: MappingKeyUnitClass,
        identity_invariant: &'static str,
        name: &str,
    ) {
        let span_in_slots = slots_spanned_by_a_unit_of_the_class(child_class)
            .expect("child_unit_class_other_than_the_entry_type_names 只交回有长度的登记码");
        let what = format!("{name}内部条目的子指针");
        self.note_references_of_a_pointer_followed_by_the_walk(child, span_in_slots, &what);
        self.judge_every_copy_of_a_child_the_walk_does_not_go_into(child, &what);
        self.judgements.judge(identity_invariant, false, || {
            format!(
                "{name}内部条目类型段 {entry_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）"
            )
        });
    }

    /// 走读不往下读的孩子（墓碑树、谱系树、inode 树内部条目里类型段不认得的，与孩子自证成类型段之外另一类的；墓碑树、谱系树层级 1 节点里
    /// 类型段 0 的，第 31 批 F-L0）：每一份副本照判
    /// I-2.1（`.claude/kb/invariants.md` I-2.1「任一被引用的块，其校验和与内容匹配」；判决 `research/prompts/m3-batch27-code-r1-main-verification.md`
    /// 第二节 H-J3a、H-J3b 两行与第三节第 1 条）。一份对不对得上按
    /// [`copy_matches_its_location_checksum_at_the_length_its_header_names`] 判；两份都对不上记走读失败（I-4.8、I-7.2、I-7.4 照旧跟着）。
    /// 只判这两样：不往下走，引用由调用方按它定的形状先登记。
    fn judge_every_copy_of_a_child_the_walk_does_not_go_into(
        &mut self,
        child: &PointerView,
        what: &str,
    ) {
        let mut some_copy_matches = false;
        for location in &child.locations {
            let matches = copy_matches_its_location_checksum_at_the_length_its_header_names(
                self.reader,
                location,
            );
            self.judgements.judge("I-2.1", matches, || {
                format!(
                    "{what}（走读不往下读的孩子）在盘 {} 槽 {} 的那一份按头里那一类的长度（没登记的 16 KiB、32 KiB 都试）与位置条目里的校验和对不上",
                    location.device, location.slot
                )
            });
            some_copy_matches |= matches;
        }
        if !some_copy_matches {
            self.walk_failures.push(format!(
                "{what}（走读不往下读的孩子）两份都与位置条目里的校验和对不上"
            ));
        }
    }

    /// 墓碑树、谱系树里类型段 0 那条条目下面的码 2 节点：层级要是父层级 − 1，条目宽 120 才往下走（[`Self::walk_container_tree_root`]）。
    /// 父节点层级 ≤ 1 时孩子不往下走、记走读失败；孩子每一份副本照判 I-2.1（判决 `research/prompts/m3-batch29-code-r1-main-verification.md`
    /// 第二节 K2 L1 第三支那一行、第三节第 1 条，攻方改法 F-L0），引用照旧不登记。
    #[allow(
        clippy::too_many_arguments,
        reason = "条目、孩子指针与树的三样都是这一步要的输入，各有名字与类型，不为凑短签名收成结构体"
    )]
    fn walk_the_internal_node_below_a_container_tree_entry(
        &mut self,
        node: &crate::IndexNodeView,
        entry: &[u8],
        child: &PointerView,
        tree: u64,
        record_type: u16,
        identity_invariant: &'static str,
        name: &str,
    ) {
        if node.level <= 1 {
            self.judge_every_copy_of_a_child_the_walk_does_not_go_into(
                child,
                &format!("{name}内部条目的子指针"),
            );
            self.walk_failures.push(format!(
                "{name}层级 {} 的节点有类型段 0 的条目：孩子会是层级 0 的码 2 节点，这两棵树里没有条款定的形状",
                node.level
            ));
            return;
        }
        let IndexNodeReading::WalkedFirstByThisVersion {
            view: child_node,
            placement_it_was_read_from: child_placement,
        } = self.read_index_node(
            &entry[34..120],
            tree,
            KEY_SCHEMA_CONTAINER_NUMBER,
            &format!("{name}根之下的内部节点"),
            KeyRangeReading::FirstAndLastEntries,
            EntryFieldTableRegistration::Registered,
        )
        else {
            return;
        };
        self.judgements
            .judge("I-1.1", child_node.level + 1 == node.level, || {
                format!(
                    "{name}内部节点层级 {} 不是父层级 {} − 1",
                    child_node.level, node.level
                )
            });
        if child_node.level + 1 == node.level
            && child_node.entry_width == inode_internal_entry_bytes()
        {
            self.walk_container_tree_root(
                &child_node,
                child_placement,
                tree,
                record_type,
                identity_invariant,
                name,
            );
        }
    }

    /// 墓碑树、谱系树里类型段 = `record_type` 那条条目下面的码 3 容器：先登记引用、读容器，按 `identity_invariant` 判条目身份
    /// （容器头四段与身份引用逐字相等、子指针头部出生树相等、类标签 3 与打包记录类型重合 I-1.6 / I-1.7 的那几小句留作合取；
    /// 容器两份都读不出时只判子指针头部出生树那一小句）；
    /// 这一遍第一次走到时再判树 ID（I-1.3）、出生身份（I-1.2 / I-4.2）与容器格式（I-1.7）。
    fn walk_the_container_below_a_container_tree_entry(
        &mut self,
        child: &PointerView,
        identity: (u64, u16, u64, u64),
        tree: u64,
        record_type: u16,
        identity_invariant: &'static str,
        name: &str,
    ) {
        let what = format!("{name}内部条目的子指针");
        self.note_references_of_a_pointer_followed_by_the_walk(child, 2, &what);
        let Some((unit, placement_it_was_read_from)) = self
            .read_a_unit_to_walk_and_the_placement_it_was_read_from(
                child,
                data_unit_bytes(),
                &what,
            )
        else {
            // 容器两份都读不出：「子指针头部出生树 == 身份引用出生树」只比父条目里的两段，照判（判决
            // `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H3 H-B 那一行、第三节第 1 条）；比容器头的那几小句不判。
            let identity_birth_tree = identity.0;
            self.judgements.judge(
                identity_invariant,
                child.birth_tree == identity_birth_tree,
                || {
                    format!(
                        "{name}内部条目类型段 {record_type}：容器两份都读不出（比容器头的那几小句不判），子指针头部的出生树 {} 不等于身份引用的出生树 {identity_birth_tree}",
                        child.birth_tree
                    )
                },
            );
            self.walk_failures
                .push(format!("{name}的容器两份都读不到对得上的"));
            return;
        };
        let header_identity_of_the_container = (
            read_u64(&unit, 43),
            read_u16(&unit, 51),
            read_u64(&unit, 53),
            read_u64(&unit, 61),
        );
        let container_entry_holds = unit[6] == 3
            && header_identity_of_the_container == identity
            && header_identity_of_the_container.1 == record_type
            && child.birth_tree == identity.0;
        self.judgements
            .judge(identity_invariant, container_entry_holds, || {
                format!(
                    "{name}内部条目的身份 {identity:?} 与容器头 {header_identity_of_the_container:?}（类 {}、子指针出生树 {}）不符",
                    unit[6], child.birth_tree
                )
            });
        if !self.visited_units.insert(placement_it_was_read_from) {
            self.merge_placements_below_a_unit_walked_by_an_earlier_version(
                placement_it_was_read_from,
            );
            return;
        }
        let container_name = format!("{name}的容器");
        self.judgements
            .judge("I-1.3", read_u64(&unit, 43) == tree, || {
                format!(
                    "{container_name}的出生树 {} 不是 {tree}",
                    read_u64(&unit, 43)
                )
            });
        self.judge_birth_identity_of_a_referenced_unit(
            &unit,
            child,
            placement_it_was_read_from,
            &container_name,
        );
        // 容器格式（类标签、记录类型、记录宽与声明长度）按 I-1.7 那一套判；记录本身的不变量归删除态扫描（I-9.18 起那几条）。
        let _records = self.judge_packed_container(&unit, record_type, &container_name);
    }

    /// I-9.1：inode 树根恒码 2（读到这里时它已经按码 2 判过头）；层级 ≥ 1 的根往下走（[`Self::walk_inode_internal_node`]），
    /// 走完按叶序判 I-9.4（容器号不超最小 key） 的第二、三句（[`Self::judge_container_numbers_along_the_leaf_order`]）。
    fn walk_inode_root(
        &mut self,
        root: &crate::IndexNodeView,
        tree: u64,
        root_placement: (u32, u64),
    ) {
        if root.level == 0 {
            return;
        }
        self.inode_tree_walked = true;
        let mut leaves_in_leaf_order: Vec<InodeLeafContainerInLeafOrder> = Vec::new();
        self.walk_inode_internal_node(root, tree, root_placement, &mut leaves_in_leaf_order);
        self.judge_container_numbers_along_the_leaf_order(&leaves_in_leaf_order);
    }

    /// I-9.4（容器号不超最小 key） 的第二句：沿叶序（这一版的 inode 树从左到右）逐对比，容器号严格递增。原来的第三句（左容器的最大 key < 右容器号）
    /// 随 N1 删掉（用户 2026-10-08 定案 X4，判决 `research/prompts/m3-leaf-death-r3-main-verification.md` 第五节）：左从右借之后右片保留身份，
    /// 它的容器号可以不大于左片的最大 key。第一句（记录 inode ≥ 容器号）与第四句（同一片容器里记录号沿记录序严格递增）在走叶容器时逐条判。
    /// 叶序里有别的版本已经走过的叶（缓存的容器号与 key 区间）；读不出的叶不在叶序里，它两边的叶直接相邻着比
    /// （合法镜像上容器号与 key 沿叶序单调，隔一片比照样成立）。
    fn judge_container_numbers_along_the_leaf_order(
        &mut self,
        leaves_in_leaf_order: &[InodeLeafContainerInLeafOrder],
    ) {
        for adjacent in leaves_in_leaf_order.windows(2) {
            let (left, right) = (adjacent[0], adjacent[1]);
            self.judgements
                .judge("I-9.4", left.container < right.container, || {
                    format!(
                        "沿叶序容器号不严格递增：容器 {} 之后是容器 {}",
                        left.container, right.container
                    )
                });
        }
    }

    /// inode 树的一个内部节点（根，或类型段 0 指着的码 2 节点）：条目逐条判 I-9.2（条目身份与子头相符） 并往下走——类型段 2 走到叶容器
    /// （[`Self::walk_inode_leaf_container`]），类型段 0 走进下一层内部节点（[`Self::walk_inode_internal_child`]），别的类型段判红、不往下走；
    /// 逐条记下「分隔 key 与这个孩子的 key 区间」，走完按 I-9.12（分隔 key 落在孩子区间之外） 判一遍。
    /// 它下面按叶序的叶容器追加进 `leaves_in_leaf_order`，也记进 `inode_leaf_order_below_a_walked_unit`（别的版本共享它时接上叶序）。
    /// 递归的上界：每进一层类型段 0，层级减一（[`Self::walk_inode_internal_child`] 判层级 = 父层级 − 1 且 > 0 才往下走）。
    fn walk_inode_internal_node(
        &mut self,
        node: &crate::IndexNodeView,
        tree: u64,
        node_placement: (u32, u64),
        leaves_in_leaf_order: &mut Vec<InodeLeafContainerInLeafOrder>,
    ) {
        let leaves_before_this_node = leaves_in_leaf_order.len();
        // (分隔 key, 这个孩子装的最小 key, 最大 key)：读不出孩子的那几条不进来，I-9.12 只判读得出的那些。
        let mut separator_and_child_range: Vec<(u64, u64, u64)> = Vec::new();
        let previous_parent_unit = self.start_walking_below(node_placement);
        for entry in &node.entries {
            let record_type = read_u16(entry, 16);
            let entry_type = InodeInternalEntryType::of(record_type);
            let identity = (
                read_u64(entry, 8),
                record_type,
                read_u64(entry, 18),
                read_u64(entry, 26),
            );
            let child = parse_node_pointer(&entry[34..120]);
            judge_location_order(&mut self.judgements, &child, "inode 内部条目的子指针");
            judge_pointer_mac_and_nonce_are_zero(
                &mut self.judgements,
                &child,
                "inode 内部条目的子指针",
            );
            judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
                &mut self.judgements,
                &child,
                "inode 内部条目的子指针",
            );
            judge_pointer_extent_offset_is_zero(
                &mut self.judgements,
                &child,
                "inode 内部条目的子指针",
            );
            // I-9.2 判孩子的类按孩子头里的类标签认，不按类型段定读多长（同 I-9.17 / I-9.23，判决
            // `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N4 那一行「inode 树 I-9.2 同形」）：
            // 孩子自证过的类与类型段说的不符（类型段不认得的，孩子读得出）判 I-9.2 红，按它自己那一类占的槽登记引用，不往下走、不记走读失败；
            // 每一份副本照判 I-2.1（第 29 批，H-J3b：自证成另一类的之外那一份校验和对不上的照红）。
            // 另有一份按类型段那一类自证过的，判红之后照类型段那一支往下走那一份（第 30 批，F-W，见下）。
            let class_the_entry_type_names = match entry_type {
                InodeInternalEntryType::LeafContainer => {
                    Some(MappingKeyUnitClass::PackedRecordUnit)
                }
                InodeInternalEntryType::InternalNode => Some(MappingKeyUnitClass::IndexNode),
                InodeInternalEntryType::Unregistered(_) => None,
            };
            // 认出另一类、另有一份头里就是类型段那一类且按那一类的长度自证得过（F-W，同墓碑树、谱系树那一处，
            // `Self::walk_the_entries_of_a_container_tree_node`）：I-9.2 照判红，再照类型段那一支往下走那一份（槽数按类型段那一类登记）。
            let child_class_the_walk_does_not_go_below =
                match child_unit_class_other_than_the_entry_type_names(
                    self.reader,
                    &child,
                    class_the_entry_type_names,
                ) {
                    None => None,
                    Some(child_class) => {
                        let a_copy_of_the_class_the_entry_type_names_verifies =
                            class_the_entry_type_names.is_some_and(|class| {
                                some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(
                                    self.reader,
                                    &child,
                                    class,
                                )
                            });
                        if a_copy_of_the_class_the_entry_type_names_verifies {
                            self.judgements.judge("I-9.2", false, || {
                                format!(
                                    "inode 内部条目类型段 {record_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）；另一份头里就是类型段那一类、按那一类的长度自证得过，照类型段那一支往下走那一份"
                                )
                            });
                            None
                        } else {
                            Some(child_class)
                        }
                    }
                };
            if let Some(child_class) = child_class_the_walk_does_not_go_below {
                let span_in_slots = slots_spanned_by_a_unit_of_the_class(child_class).expect(
                    "child_unit_class_other_than_the_entry_type_names 只交回有长度的登记码",
                );
                self.note_references_of_a_pointer_followed_by_the_walk(
                    &child,
                    span_in_slots,
                    "inode 树的孩子",
                );
                self.judge_every_copy_of_a_child_the_walk_does_not_go_into(
                    &child,
                    "inode 树的孩子",
                );
                self.judgements.judge("I-9.2", false, || {
                    format!(
                        "inode 内部条目类型段 {record_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）"
                    )
                });
                continue;
            }
            let registered_entry_type = match entry_type {
                InodeInternalEntryType::LeafContainer => {
                    RegisteredInodeInternalEntryType::LeafContainer
                }
                InodeInternalEntryType::InternalNode => {
                    RegisteredInodeInternalEntryType::InternalNode
                }
                InodeInternalEntryType::Unregistered(unregistered_type) => {
                    // 类型段不认得、上面也没认出孩子自证过的类：不往下走孩子，与墓碑树、谱系树类型段不认得那一支同判
                    // （`Self::walk_the_entries_of_a_container_tree_node`；`records/2026-10-01-删除与并发事务.md` 第 27 批，
                    // 第 26 批 H 报告「交主 agent」第 1 条）：I-9.2 红。孩子先按这一层定的形状登记引用
                    // （层级 1 的节点下面是码 3 叶容器两槽，层级 ≥ 2 的下面是码 2 节点一槽），不登记的话它占的槽不进引用并集，I-3.1 连带红。
                    // 孩子不往下走，每一份副本照判 I-2.1、两份都对不上记走读失败（第 29 批，H-J3a：第 27 批这一支两样都不记，
                    // 位置条目里的校验和坏了也报 I-2.1 成立）。
                    let child_span_in_slots_by_the_level = if node.level >= 2 { 1 } else { 2 };
                    self.note_references_of_a_pointer_followed_by_the_walk(
                        &child,
                        child_span_in_slots_by_the_level,
                        "inode 树的孩子",
                    );
                    self.judge_every_copy_of_a_child_the_walk_does_not_go_into(
                        &child,
                        "inode 树的孩子",
                    );
                    self.judgements.judge("I-9.2", false, || {
                        format!(
                            "inode 内部条目的类型段 {unregistered_type} 不在 {{0, 2}} 里（孩子没有一份按它头里那一类的长度自证过，不往下走）"
                        )
                    });
                    continue;
                }
            };
            let (length, span_in_slots) = match registered_entry_type {
                RegisteredInodeInternalEntryType::LeafContainer => (data_unit_bytes(), 2),
                RegisteredInodeInternalEntryType::InternalNode => (node_bytes(), 1),
            };
            self.note_references_of_a_pointer_followed_by_the_walk(
                &child,
                span_in_slots,
                "inode 树的孩子",
            );
            let Some((unit, placement_it_was_read_from)) = self
                .read_a_unit_to_walk_and_the_placement_it_was_read_from(
                    &child,
                    length,
                    "inode 树的孩子",
                )
            else {
                judge_an_inode_entry_whose_child_does_not_read_back(
                    &mut self.judgements,
                    registered_entry_type,
                    identity,
                    child.birth_tree,
                );
                self.walk_failures
                    .push("inode 树的孩子两份都读不到对得上的".to_string());
                continue;
            };
            let subtree = match registered_entry_type {
                RegisteredInodeInternalEntryType::LeafContainer => {
                    let header_identity = (
                        read_u64(&unit, 43),
                        read_u16(&unit, 51),
                        read_u64(&unit, 53),
                        read_u64(&unit, 61),
                    );
                    let entry_holds = unit[6] == 3
                        && header_identity == identity
                        && child.birth_tree == identity.0;
                    self.judgements.judge("I-9.2", entry_holds, || {
                        format!(
                            "inode 内部条目的身份 {identity:?} 与孩子的头 {header_identity:?}（类 {}）不符",
                            unit[6]
                        )
                    });
                    self.walk_inode_leaf_container(&unit, &child, tree, placement_it_was_read_from)
                }
                RegisteredInodeInternalEntryType::InternalNode => {
                    let (birth_tree, _, container, container_birth) = identity;
                    let entry_holds =
                        unit[6] == 2 && birth_tree == 0 && container == 0 && container_birth == 0;
                    self.judgements.judge("I-9.2", entry_holds, || {
                        format!(
                            "inode 内部条目类型段 0：孩子的类标签 {}（要 2），身份引用的出生树 / 容器号 / 出生代 {birth_tree} / {container} / {container_birth}（要三段为 0）",
                            unit[6]
                        )
                    });
                    self.walk_inode_internal_child(
                        &unit,
                        &child,
                        node.level,
                        tree,
                        placement_it_was_read_from,
                    )
                }
            };
            if subtree.walked_first_by_this_version {
                let smallest = subtree
                    .leaves_in_leaf_order
                    .iter()
                    .map(|leaf| leaf.smallest_inode)
                    .min();
                let largest = subtree
                    .leaves_in_leaf_order
                    .iter()
                    .map(|leaf| leaf.largest_inode)
                    .max();
                if let (Some(smallest), Some(largest)) = (smallest, largest) {
                    separator_and_child_range.push((read_u64(entry, 0), smallest, largest));
                }
            }
            leaves_in_leaf_order.extend(subtree.leaves_in_leaf_order);
        }
        self.stop_walking_below(previous_parent_unit);
        self.inode_leaf_order_below_a_walked_unit.insert(
            node_placement,
            leaves_in_leaf_order[leaves_before_this_node..].to_vec(),
        );
        self.judge_separators_against_the_inode_children(&separator_and_child_range);
    }

    /// 一片 inode 叶容器（码 3 打包记录类型 2）：这一遍第一次走到时判树 ID（I-1.3）、出生身份（I-1.2 / I-4.2）、容器（I-1.7 等）与容器里
    /// 每条记录（I-9.13、I-9.7、I-9.4 第一句与第四句、I-9.15），交回它的容器号与装的最小、最大 inode 号；别的版本已经走过时不重读，
    /// 从 `inode_leaf_order_below_a_walked_unit` 接上它在叶序里的那一格，它下面的落点并进这一版（I-5.1）。
    /// 走过、并进前一版、叶序与诞生代号都按读回的那一份的落点 `placement_it_was_read_from` 认（F-KEY 与攻方 allrbmap，
    /// [`Self::read_a_unit_to_walk_and_the_placement_it_was_read_from`]）。
    fn walk_inode_leaf_container(
        &mut self,
        unit: &[u8],
        child: &PointerView,
        tree: u64,
        placement_it_was_read_from: (u32, u64),
    ) -> InodeSubtreeInTheWalk {
        if !self.visited_units.insert(placement_it_was_read_from) {
            self.merge_placements_below_a_unit_walked_by_an_earlier_version(
                placement_it_was_read_from,
            );
            return InodeSubtreeInTheWalk {
                leaves_in_leaf_order: self
                    .inode_leaf_order_below_a_walked_unit
                    .get(&placement_it_was_read_from)
                    .cloned()
                    .unwrap_or_default(),
                walked_first_by_this_version: false,
            };
        }
        let mut leaves_in_leaf_order: Vec<InodeLeafContainerInLeafOrder> = Vec::new();
        self.judgements
            .judge("I-1.3", read_u64(unit, 43) == tree, || {
                format!("inode 树叶的出生树 {} 不是 {tree}", read_u64(unit, 43))
            });
        self.judge_birth_identity_of_a_referenced_unit(
            unit,
            child,
            placement_it_was_read_from,
            "inode 树的叶容器",
        );
        if let Some(records) =
            self.judge_packed_container(unit, PACKED_TYPE_INODE, "inode 树的叶容器")
        {
            let container = read_u64(unit, 53);
            self.judgements.judge("I-9.13", !records.is_empty(), || {
                "类型 2 容器一条记录都没有".to_string()
            });
            let mut smallest_inode_in_this_container: Option<u64> = None;
            let mut largest_inode_in_this_container: Option<u64> = None;
            let mut inode_of_the_previous_record: Option<u64> = None;
            for record in records {
                let inode = read_u64(&record, 0);
                // I-9.4 第四句：同一片打包记录类型 2 容器里记录号沿记录序严格递增（判决
                // `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T11 取甲）。与第二、三句合起来就是整棵树的 key 沿叶序严格递增。
                if let Some(previous_inode) = inode_of_the_previous_record {
                    self.judgements.judge("I-9.4", inode > previous_inode, || {
                        format!(
                            "容器 {container} 里记录号沿记录序不严格递增：inode {previous_inode} 之后是 inode {inode}"
                        )
                    });
                }
                inode_of_the_previous_record = Some(inode);
                self.judgements.judge(
                    "I-9.7",
                    record[108..140].iter().all(|byte| *byte == 0),
                    || format!("inode {inode} 的记录偏移 108 起三段不全为 0"),
                );
                self.judgements.judge("I-9.4", inode >= container, || {
                    format!("inode {inode} 小于它所在容器的号 {container}")
                });
                let size_in_bytes = read_u64(&record, INODE_RECORD_SIZE_OFFSET);
                let blocks = read_u64(&record, INODE_RECORD_BLOCKS_OFFSET);
                let logical_length_in_blocks =
                    size_in_bytes.div_ceil(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES);
                self.judgements
                        .judge("I-9.15", blocks == logical_length_in_blocks, || {
                            format!(
                                "inode {inode} 的记录 blocks {blocks} 不等于 ⌈size {size_in_bytes} ÷ {INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES}⌉ = {logical_length_in_blocks}"
                            )
                        });
                // I-9.26：对象出生代 ≤ 改动计数 ≤ 这片容器头的诞生代号（D8（核心索引结构） 已定项 6 偏移 88；下界「创建算一次改动」
                // 用户 2026-10-01 补定）。上界不是等于：容器因别的记录改动而重写时，没改的那条记录的改动计数小于容器诞生代号是合法的。
                let object_birth = read_u64(&record, INODE_RECORD_OBJECT_BIRTH_OFFSET);
                let change_count = read_u64(&record, INODE_RECORD_CHANGE_COUNT_OFFSET);
                let container_birth_txg = read_u64(unit, PACKED_UNIT_BIRTH_TXG_OFFSET);
                self.judgements.judge(
                    INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
                    object_birth <= change_count && change_count <= container_birth_txg,
                    || {
                        format!(
                            "inode {inode} 的改动计数 {change_count} 不在对象出生代 {object_birth} 与所在容器的诞生代号 {container_birth_txg} 之间"
                        )
                    },
                );
                self.inode_object_birth.insert(inode, object_birth);
                smallest_inode_in_this_container =
                    Some(smallest_inode_in_this_container.map_or(inode, |seen| seen.min(inode)));
                largest_inode_in_this_container =
                    Some(largest_inode_in_this_container.map_or(inode, |seen| seen.max(inode)));
                self.largest_inode_number_in_the_inode_tree = Some(
                    self.largest_inode_number_in_the_inode_tree
                        .map_or(inode, |seen| seen.max(inode)),
                );
            }
            if let (Some(smallest), Some(largest)) = (
                smallest_inode_in_this_container,
                largest_inode_in_this_container,
            ) {
                leaves_in_leaf_order.push(InodeLeafContainerInLeafOrder {
                    container,
                    smallest_inode: smallest,
                    largest_inode: largest,
                });
            }
        }
        self.inode_leaf_order_below_a_walked_unit
            .insert(placement_it_was_read_from, leaves_in_leaf_order.clone());
        InodeSubtreeInTheWalk {
            leaves_in_leaf_order,
            walked_first_by_this_version: true,
        }
    }

    /// 类型段 0 指着的码 2 内部节点：这一遍第一次走到时判头（I-1.6 / I-2.4 / I-2.3 / I-1.4）、出生身份（I-1.2 / I-4.2）、树 ID（I-1.3）、
    /// key（I-1.1，与 inode 根同一个读法：区间贴紧首末条目，C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间） 还开着）、
    /// 层级 = 父层级 − 1（I-1.1）与条目宽 120（I-1.10），再往下走（[`Self::walk_inode_internal_node`]）；
    /// 别的版本已经走过时不重读，从 `inode_leaf_order_below_a_walked_unit` 接上它下面的叶序，它下面的落点并进这一版（I-5.1）。
    /// 层级 0 的码 2 节点在 inode 树里没有条款定的形状（叶是码 3 容器，I-9.1 那一行只给根留了「空的根」）：记走读失败、不往下走。
    /// 走过、父单元、叶序、并进前一版与诞生代号都按读回的那一份的落点 `placement_it_was_read_from` 认（F-KEY 与攻方 allrbmap，
    /// 同 [`Self::walk_inode_leaf_container`]）。
    fn walk_inode_internal_child(
        &mut self,
        unit: &[u8],
        child: &PointerView,
        parent_level: u8,
        tree: u64,
        placement_it_was_read_from: (u32, u64),
    ) -> InodeSubtreeInTheWalk {
        let not_walked_below = InodeSubtreeInTheWalk {
            leaves_in_leaf_order: Vec::new(),
            walked_first_by_this_version: true,
        };
        if !self.visited_units.insert(placement_it_was_read_from) {
            self.merge_placements_below_a_unit_walked_by_an_earlier_version(
                placement_it_was_read_from,
            );
            return InodeSubtreeInTheWalk {
                leaves_in_leaf_order: self
                    .inode_leaf_order_below_a_walked_unit
                    .get(&placement_it_was_read_from)
                    .cloned()
                    .unwrap_or_default(),
                walked_first_by_this_version: false,
            };
        }
        let what = "inode 树的内部节点";
        if !self.judge_unit_header(unit, 2, what) {
            return not_walked_below;
        }
        self.judge_birth_identity_of_a_referenced_unit(
            unit,
            child,
            placement_it_was_read_from,
            what,
        );
        let Some(view) = self.index_node_view_judging_a_zero_entry_width(
            unit,
            EntryFieldTableRegistration::Registered,
            what,
        ) else {
            return not_walked_below;
        };
        self.judgements
            .judge("I-1.3", view.tree_identifier == tree, || {
                format!(
                    "{what}：头里的树 ID {} 不是引用它的树 {tree}",
                    view.tree_identifier
                )
            });
        let keys = check_index_node_keys(&view, KEY_SCHEMA_INODE);
        self.judgements.judge("I-1.1", keys.is_ok(), || {
            format!("{what}：key 宽 / key 区间与条目不符（{keys:?}）")
        });
        let expected_level = parent_level.checked_sub(1);
        self.judgements
            .judge("I-1.1", Some(view.level) == expected_level, || {
                format!(
                    "{what}：头里的层级是 {}，引用它的父节点层级是 {parent_level}（要父层级减一）",
                    view.level
                )
            });
        if Some(view.level) != expected_level {
            self.walk_failures
                .push(format!("{what}：层级不是父层级减一，这一支走不下去"));
            return not_walked_below;
        }
        if view.level == 0 {
            self.walk_failures.push(format!(
                "{what}：类型段 0 的条目指着层级 0 的码 2 节点，inode 树里没有这种形状，这一支走不下去"
            ));
            return not_walked_below;
        }
        let entry_width = view.entry_width;
        self.judgements.judge(
            "I-1.10",
            entry_width == inode_internal_entry_bytes(),
            || {
                format!(
                    "{what}：头里自述的条目宽 {entry_width} 不等于 inode 树的条目字段表宽度 {}",
                    inode_internal_entry_bytes()
                )
            },
        );
        if entry_width != inode_internal_entry_bytes() {
            return not_walked_below;
        }
        let mut leaves_in_leaf_order = Vec::new();
        self.walk_inode_internal_node(
            &view,
            tree,
            placement_it_was_read_from,
            &mut leaves_in_leaf_order,
        );
        InodeSubtreeInTheWalk {
            leaves_in_leaf_order,
            walked_first_by_this_version: true,
        }
    }

    /// I-9.12（分隔 key 落在孩子区间之外）：一个 inode 树内部节点的条目按分隔 key 严格递增；第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界，
    /// 且 > 第 i−1 个孩子的 key 区间上界。孩子的 key 区间：叶容器按它装的记录现算，类型段 0 的内部节点按它下面各片叶装的记录现算
    /// （D18（块里携带什么信息） 已定项 2「子树里到得了的最小 key 与最大 key」）。
    /// 判的是读得出、这一遍第一次走到的那些孩子；一个都读不出来时这条在这份镜像上没有对象（末尾按「走到过 inode 树」报不适用）。
    fn judge_separators_against_the_inode_children(
        &mut self,
        separator_and_child_range: &[(u64, u64, u64)],
    ) {
        for (position, (separator, smallest, _largest_of_this_child)) in
            separator_and_child_range.iter().enumerate()
        {
            self.judgements.judge("I-9.12", separator <= smallest, || {
                format!("第 {position} 条的分隔 key {separator} 大于这个孩子的最小 key {smallest}")
            });
            if position > 0 {
                let (previous_separator, _, previous_largest) =
                    separator_and_child_range[position - 1];
                self.judgements
                    .judge("I-9.12", *separator > previous_separator, || {
                        format!("第 {position} 条的分隔 key {separator} 不大于上一条的 {previous_separator}")
                    });
                self.judgements
                    .judge("I-9.12", *separator > previous_largest, || {
                        format!(
                            "第 {position} 条的分隔 key {separator} 不大于上一个孩子的最大 key {previous_largest}"
                        )
                    });
            }
        }
    }

    /// 沿链走一张实例表（D18（块里携带什么信息） 已定项 11：根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片）。
    /// 每一片判 I-2.5（位置条目升序）、I-2.1（校验和）、码 3 容器那几条（`judge_packed_container`）、I-1.1（身份四元组 =
    /// (0, 4, 片序号, 0)：链指针指到了别的单元、链成了环，都在这一格红）与 I-3.14（实例表每片记录形态合法：最后一条是合法的链指针记录，
    /// 加 `judge_instance_table_page_record_forms` 那三件）；
    /// 挂载根那一张（`mount_root_instance_when_newest` 给了实例代号）把各片的记录交给 `judge_instance_table_rows` 判行那一半，
    /// 走到末片（链指针记录说「没有下一片」）时记下候选集滤得全（`abandoned_roots_filtering`）。
    /// 迭代上界：第 k 片要求容器号是 k，走过的单元不再走（`visited_units`，按读回的那一份的落点认），每一轮是盘上一个不同的单元。
    /// 提前出口，链都不再往下走：读不出、身份不对（被引用的单元坏了或换了人，记进走读失败；头用不了由 `judge_unit_header` 记）；
    /// 容器记录不合法、I-3.14 不成立（这一片自己写错了，只红 I-1.7 / I-3.14 那一格；这一片解不开，行不收）；走到已经走过的单元（前一条根走过这条链）。
    /// I-3.14 不成立的五形（kind 不是 0 或 1、片中间有链指针记录、行的 flags 不是 0、最后一条不是合法的链指针记录、它说有下一片而位置指针头部
    /// 有一段不是第一版的取值）一律当这一片解不开，与实现 `InstanceTablePage::parse` 同一个读法（用户 2026-10-06 给 D18（块里携带什么信息）
    /// 已定项 11 补的那一句；第五形用户 2026-10-06 定 K2-1 甲）。第五形那条指针不跟随，走读不往下走，它头部那几段 I-2.4 不判（走读不跟随的指针 I-2.4 不判）。
    fn walk_instance_table_chain(
        &mut self,
        first_page_pointer: &PointerView,
        mount_root_instance_when_newest: Option<u32>,
    ) {
        use crate::image::INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED;
        let mut pointer = *first_page_pointer;
        let mut page_index: u64 = 0;
        let mut pages: Vec<Vec<Vec<u8>>> = Vec::new();
        // 第 0 片由根记录里的指针指着（外面那一层的走读父单元照旧）；第 k + 1 片由第 k 片末尾的链指针记录指着，记在第 k 片下面。
        let parent_unit_when_the_chain_starts = self.walk_parent_unit;
        let mut walk_stopped_at_a_page_that_does_not_decode = false;
        loop {
            let (what, pointer_what) = if page_index == 0 {
                ("实例表单元".to_string(), "实例表指针".to_string())
            } else {
                (
                    format!("实例表第 {page_index} 片"),
                    format!("实例表第 {} 片的链指针", page_index - 1),
                )
            };
            judge_location_order(&mut self.judgements, &pointer, &pointer_what);
            judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &pointer, &pointer_what);
            judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
                &mut self.judgements,
                &pointer,
                &pointer_what,
            );
            judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, &pointer_what);
            self.note_references_of_a_pointer_followed_by_the_walk(&pointer, 2, &what);
            let Some((unit, placement_it_was_read_from)) = self
                .read_a_unit_to_walk_and_the_placement_it_was_read_from(
                    &pointer,
                    data_unit_bytes(),
                    &what,
                )
            else {
                self.walk_failures
                    .push(format!("{what}两份都读不到对得上的"));
                break;
            };
            // 「走过」按读回、自证过的那一份的落点认（攻方 H-b，`research/prompts/m3-batch25-code-r1-opus-output.md` 第 1.3 节）：
            // 第 k 片指针的盘 0 位置条目指回走过的片、盘 1 那条完好时，按第 0 条位置条目认会把读到手的这一片当成走过、丢掉它的行。
            // 别处的键（父单元、诞生代号那张表、并进前一版的落点表）同一个落点（与走读别的单元同一个读法，攻方 allrbmap）。
            if !self.visited_units.insert(placement_it_was_read_from) {
                self.merge_placements_below_a_unit_walked_by_an_earlier_version(
                    placement_it_was_read_from,
                );
                break;
            }
            self.walk_parent_unit = Some(placement_it_was_read_from);
            let Some(records) =
                self.judge_packed_container(&unit, PACKED_TYPE_INSTANCE_TABLE, &what)
            else {
                break;
            };
            self.judge_pointer_birth_tree_against_the_unit_header(&unit, &pointer, &what);
            if let Some(birth_identity) = birth_identity_in_the_header(&unit) {
                self.birth_txg_of_walked_instance_table_pages.insert(
                    placement_it_was_read_from,
                    birth_identity.write_order.birth_txg(),
                );
                self.judge_birth_identity_against_the_pointer(birth_identity, &pointer, &what);
            }
            let identity = (
                read_u64(&unit, 43),
                read_u16(&unit, 51),
                read_u64(&unit, 53),
                read_u64(&unit, 61),
            );
            let expected_identity = (0, PACKED_TYPE_INSTANCE_TABLE, page_index, 0);
            self.judgements
                .judge("I-1.1", identity == expected_identity, || {
                    format!(
                        "{what}：身份四元组 (出生树, 类型, 容器号, 容器出生代) = {identity:?}，链上第 {page_index} 片该是 {expected_identity:?}"
                    )
                });
            if identity != expected_identity {
                self.walk_failures
                    .push(format!("{what}：指到的单元不是链上第 {page_index} 片"));
                break;
            }
            let record_forms =
                judge_instance_table_page_record_forms(&mut self.judgements, &records, &what);
            let chain = records
                .last()
                .map_or(ChainRecordView::Malformed, |last| chain_record_view(last));
            self.judgements.judge(
                INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
                chain != ChainRecordView::Malformed,
                || {
                    format!(
                        "{what}：最后一条不是合法的链指针记录（kind 1、有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零且头部 MAC / nonce、算法类型、压缩算法码、压后长度、extent 偏移都是第一版的取值）"
                    )
                },
            );
            // I-3.14 不成立的一片解不开（D18（块里携带什么信息） 已定项 11 用户 2026-10-06 补句；实现 `InstanceTablePage::parse` 同样交 None），
            // 照上面「容器记录不合法」那一处出口走：这一片的行不收、链不往下走、没走到末片（候选集滤不全，I-9.29 的门关）。
            // 不记走读失败——走读失败驱动 I-7.4 / I-4.8「引用的单元已被复用或抹头」，这里没有哪个被引用的单元坏了，是这一片自己的记录写错了。
            match record_forms {
                InstanceTablePageRecordForms::WellFormed => {}
                InstanceTablePageRecordForms::IllFormedSoThePageDoesNotDecode => {
                    walk_stopped_at_a_page_that_does_not_decode = true;
                    break;
                }
            }
            match chain {
                ChainRecordView::LastPage => {
                    pages.push(records);
                    if mount_root_instance_when_newest.is_some() {
                        self.abandoned_roots_filtering =
                            AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable;
                    }
                    break;
                }
                ChainRecordView::NextPage(next_page) => {
                    pages.push(records);
                    pointer = next_page;
                    page_index += 1;
                }
                // 链指针记录自相矛盾：上面 I-3.14 那一格已经红了，这一片同样解不开（行不收），链不往下走。
                ChainRecordView::Malformed => {
                    walk_stopped_at_a_page_that_does_not_decode = true;
                    break;
                }
            }
        }
        self.walk_parent_unit = parent_unit_when_the_chain_starts;
        if let Some(mount_root_instance) = mount_root_instance_when_newest {
            if !pages.is_empty() {
                self.judge_instance_table_rows(&pages, mount_root_instance);
            } else if walk_stopped_at_a_page_that_does_not_decode {
                self.judgements.not_applicable(
                    "I-3.8",
                    INSTANCE_TABLE_ROWS_NOT_JUDGED_AS_THE_FIRST_PAGE_OF_THE_NEWEST_ROOTS_TABLE_DOES_NOT_DECODE,
                );
            }
        }
    }

    /// 码 3 容器：I-1.7（记录数 × 记录宽 ≤ 声明长度 ≤ 32768 − 136、记录宽 = 登记的宽）。返回记录，用不了返回 None。
    /// I-3.8（实例表行唯一且低于挂载根）：整张表（链上各片的行接起来）里 `kind` 0 行按实例代号唯一、每行的实例代号 < 挂载根（最新根）的实例。
    /// 记录形态（`kind` 只有 0 或 1、链指针记录恰好在每一片末尾、行的 flags 为 0）归 I-3.14，在 `walk_instance_table_chain` 里每一片判；
    /// 这里只收 `kind` 0 的行，别的记录跳过。「行只在回收条件成立后删」对着一个镜像判不了。
    fn judge_instance_table_rows(&mut self, pages: &[Vec<Vec<u8>>], mount_root_instance: u32) {
        let mut instances: Vec<u32> = Vec::new();
        let mut unique = true;
        let mut below_mount_root = true;
        for records in pages {
            for row in records {
                match InstanceTableRecordKind::of_record(row) {
                    InstanceTableRecordKind::Row => {
                        let instance = u32::from_le_bytes(row[1..5].try_into().expect("4 字节"));
                        if instances.contains(&instance) {
                            unique = false;
                        }
                        instances.push(instance);
                        self.instance_table_rows.push(parse_instance_table_row(row));
                        if instance >= mount_root_instance {
                            below_mount_root = false;
                        }
                    }
                    InstanceTableRecordKind::ChainRecord
                    | InstanceTableRecordKind::Unregistered(_) => {}
                }
            }
        }
        self.judgements.judge(
            "I-3.8",
            unique && below_mount_root,
            || {
                format!(
                    "实例表（{} 片）：行的实例代号 {instances:?}（唯一 = {unique}，都低于挂载根实例 {mount_root_instance} = {below_mount_root}）",
                    pages.len()
                )
            },
        );
    }

    fn judge_packed_container(
        &mut self,
        unit: &[u8],
        expected_type: u16,
        what: &str,
    ) -> Option<Vec<Vec<u8>>> {
        if !self.judge_unit_header(unit, 3, what) {
            return None;
        }
        let record_type = read_u16(unit, 51);
        let count = usize::from(read_u16(unit, 69));
        let width = usize::from(read_u16(unit, 71));
        let declared = usize::from(read_u16(unit, 8));
        let legal = record_type == expected_type
            && registered_record_width(record_type) == Some(width)
            && count * width <= declared
            && declared <= data_unit_bytes() - 136;
        self.judgements.judge("I-1.7", legal, || format!("{what}：类型 {record_type}、记录宽 {width}、记录数 {count}、声明长度 {declared} 不合法"));
        legal.then(|| {
            unit[136..136 + count * width]
                .chunks(width.max(1))
                .map(<[u8]>::to_vec)
                .collect()
        })
    }

    /// extent 树里指向一个数据单元的一条（下段叶的 extent 叶记录，或上段叶条目里内联的那一个）：数据指针 88 字节，
    /// 所在的 key 是 (0, `inode`, `offset`)。读那个数据单元、判头、出生身份、树 ID 与五元组。
    fn walk_extent_data_pointer(
        &mut self,
        pointer_bytes: &[u8],
        inode: u64,
        offset: u64,
        tree: u64,
    ) {
        let pointer = parse_data_pointer(pointer_bytes);
        judge_location_order(&mut self.judgements, &pointer, "extent 记录的数据指针");
        judge_pointer_mac_and_nonce_are_zero(
            &mut self.judgements,
            &pointer,
            "extent 记录的数据指针",
        );
        judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
            &mut self.judgements,
            &pointer,
            "extent 记录的数据指针",
        );
        judge_pointer_extent_offset_is_zero(
            &mut self.judgements,
            &pointer,
            "extent 记录的数据指针",
        );
        self.note_references_of_a_pointer_followed_by_the_walk(
            &pointer,
            2,
            &format!("inode {inode} 偏移 {offset} 的数据单元"),
        );
        let Some((unit, placement_it_was_read_from)) = self
            .read_a_unit_to_walk_and_the_placement_it_was_read_from(
                &pointer,
                data_unit_bytes(),
                "数据单元",
            )
        else {
            self.walk_failures
                .push("数据单元两份都读不到对得上的".to_string());
            return;
        };
        if !self.visited_units.insert(placement_it_was_read_from) {
            self.merge_placements_below_a_unit_walked_by_an_earlier_version(
                placement_it_was_read_from,
            );
            return;
        }
        if !self.judge_unit_header(&unit, 1, "数据单元") {
            return;
        }
        self.judge_birth_identity_of_a_referenced_unit(
            &unit,
            &pointer,
            placement_it_was_read_from,
            "数据单元",
        );
        self.judge_pointer_birth_tree_against_the_unit_header(&unit, &pointer, "数据单元");
        self.judgements
            .judge("I-1.3", read_u64(&unit, 43) == tree, || {
                format!(
                    "数据单元头里的树 ID {} 不是 extent 树 {tree}",
                    read_u64(&unit, 43)
                )
            });
        let five_tuple_holds = read_u64(&unit, 51) == inode
            && read_u64(&unit, 67) == offset
            && read_u64(&unit, 43) == tree;
        self.judgements.judge("I-1.1", five_tuple_holds, || {
            format!("数据单元五元组（树 {}、对象 {}、锚点 {}）与 extent key（{tree}, {inode}, {offset}）不符", read_u64(&unit, 43), read_u64(&unit, 51), read_u64(&unit, 67))
        });
        self.data_unit_objects.push((
            inode,
            read_u64(&unit, 59),
            format!("inode {inode} 偏移 {offset} 的数据单元"),
        ));
    }

    /// 判一个按位置寻址的节点在树里的位置（I-1.1（块头自述逻辑地址）：索引节点的身份是树 ID + 层级 + key 区间，
    /// 按位置寻址的树的区间是这个节点按位置规定罩的那一段，D18（块里携带什么信息） 已定项 2）与这一层的条目宽（I-1.10）。
    /// 交回这一支还往不往下走。
    fn position_and_entry_width_hold(
        &mut self,
        view: &crate::IndexNodeView,
        level: u8,
        prescribed_range: &(Vec<u8>, Vec<u8>),
        entry_width: usize,
        what: &str,
    ) -> bool {
        self.judgements.judge("I-1.1", view.level == level, || {
            format!(
                "{what}：头里的层级是 {}，它的位置要层级 {level}",
                view.level
            )
        });
        if view.level != level {
            self.walk_failures.push(format!(
                "{what}：层级不是它的位置规定的那一层，这一支走不下去"
            ));
            return false;
        }
        self.judgements.judge(
            "I-1.1",
            view.smallest_key == prescribed_range.0 && view.largest_key == prescribed_range.1,
            || format!("{what}：头里的 key 区间不是它的位置规定罩的那一段"),
        );
        self.judgements
            .judge("I-1.10", view.entry_width == entry_width, || {
                format!(
                    "{what}：头里自述的条目宽 {} 不等于这一层的条目字段表宽度 {entry_width}",
                    view.entry_width
                )
            });
        view.entry_width == entry_width
    }

    /// 分配记录树（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址）整棵走下去：根的层级是池几何定的那一层（checker 自己按每块盘的槽数算），
    /// 每个节点核层级、它罩的就是它的位置规定的那一段、条目宽（叶 20、内部 96），内部条目的 key 是一个孩子那一段的起点、按位置严格递增，
    /// 叶里每条记录落在这片叶里（末槽不越过叶的末槽）。
    fn walk_allocation_record_tree(&mut self, root_pointer_bytes: &[u8], tree: u64, name: &str) {
        let pool_devices = self.reader.devices();
        let device_slots: Vec<u64> = pool_devices
            .iter()
            .map(|device| self.reader.device_bytes(*device).unwrap_or(0) / SLOT_BYTES)
            .collect();
        let Some(root_level) = allocation_record_tree_root_level(&device_slots) else {
            self.walk_failures.push(format!(
                "{name}（树 {tree}）：池里的盘多到根装不下，算不出根该在哪一层"
            ));
            return;
        };
        self.walk_allocation_record_node(
            root_pointer_bytes,
            AllocationRecordTreeCell::Root,
            root_level,
            tree,
            name,
            &pool_devices,
        );
    }

    /// 递归的上界：每一层下降层级减一，每个单元只走一次（`visited_units`）。
    fn walk_allocation_record_node(
        &mut self,
        pointer_bytes: &[u8],
        cell: AllocationRecordTreeCell,
        level: u8,
        tree: u64,
        name: &str,
        pool_devices: &[u32],
    ) {
        let what = match cell {
            AllocationRecordTreeCell::Root => format!("{name}（树 {tree}）的根"),
            AllocationRecordTreeCell::BelowTheRoot {
                level: cell_level,
                device,
                index,
            } => format!("{name}（树 {tree}）层级 {cell_level} 盘 {device} 第 {index} 格的节点"),
        };
        let IndexNodeReading::WalkedFirstByThisVersion {
            view,
            placement_it_was_read_from: node_placement,
        } = self.read_index_node(
            pointer_bytes,
            tree,
            KEY_SCHEMA_ALLOCATION,
            &what,
            KeyRangeReading::PrescribedByThePositionJudgedByTheCaller,
            EntryFieldTableRegistration::Registered,
        )
        else {
            return;
        };
        let entry_width = if level == 0 {
            allocation_record_bytes()
        } else {
            usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96")
        };
        if !self.position_and_entry_width_hold(
            &view,
            level,
            &allocation_record_tree_cell_range(cell),
            entry_width,
            &what,
        ) {
            return;
        }
        if level == 0 {
            for entry in &view.entries {
                let record = parse_allocation_record(entry);
                self.judgements.judge(
                    "I-1.1",
                    allocation_record_fits_in_the_leaf(
                        cell,
                        record.device,
                        record.slot,
                        record.span_slots,
                    ),
                    || {
                        format!(
                            "{what}：盘 {} 槽 {} 跨 {} 的记录不落在这片叶按位置罩的那一段里，或末槽越过叶的末槽",
                            record.device, record.slot, record.span_slots
                        )
                    },
                );
            }
            return;
        }
        let key_width = KEY_SCHEMA_ALLOCATION.width();
        let mut previous_child: Option<AllocationRecordTreeCell> = None;
        let previous_parent_unit = self.start_walking_below(node_placement);
        for entry in &view.entries {
            let child = allocation_record_tree_child_of_entry_key(
                cell,
                level,
                &entry[..key_width],
                pool_devices,
            )
            .filter(|child| previous_child.is_none_or(|previous| previous < *child));
            self.judgements.judge("I-1.1", child.is_some(), || {
                format!("{what}：一条内部条目的 key 不是这个节点里一个孩子那一段的起点，或孩子不按位置严格递增")
            });
            let Some(child) = child else {
                self.walk_failures.push(format!(
                    "{what}：内部条目指不到一个合法的孩子，这一支走不下去"
                ));
                continue;
            };
            previous_child = Some(child);
            let child_pointer_bytes = &entry[key_width..key_width + node_pointer_bytes()];
            judge_location_order(
                &mut self.judgements,
                &parse_node_pointer(child_pointer_bytes),
                &format!("{what} 的子指针"),
            );
            self.walk_allocation_record_node(
                child_pointer_bytes,
                child,
                level - 1,
                tree,
                name,
                pool_devices,
            );
        }
        self.stop_walking_below(previous_parent_unit);
    }

    /// extent 树上段一个节点连同它下面（D8（核心索引结构） 已定项 14：上段按 inode 号按位置寻址）：`position` 是 (层级, 序号)，
    /// 根那一个（`None`）的层级取它自己头里的、序号 0。每个节点核层级、位置规定的那一段、条目宽（叶 113、内部 110）；叶条目核标签
    /// （0 / 1 / 2）、inode 号落在叶罩的那一段里，标签 1 走进那个文件的下段，标签 2 走那个内联的数据单元（key (0, inode, 0)）。
    fn walk_extent_upper_node(
        &mut self,
        pointer_bytes: &[u8],
        position: Option<(u8, u64)>,
        tree: u64,
    ) {
        let what = match position {
            None => format!("extent 树（树 {tree}）的根"),
            Some((level, index)) => {
                format!("extent 树（树 {tree}）上段层级 {level} 第 {index} 格的节点")
            }
        };
        let IndexNodeReading::WalkedFirstByThisVersion {
            view,
            placement_it_was_read_from: node_placement,
        } = self.read_index_node(
            pointer_bytes,
            tree,
            KEY_SCHEMA_EXTENT,
            &what,
            match position {
                None => {
                    KeyRangeReading::PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller
                }
                Some(_) => KeyRangeReading::PrescribedByThePositionJudgedByTheCaller,
            },
            EntryFieldTableRegistration::Registered,
        )
        else {
            return;
        };
        let (level, index) = position.unwrap_or((view.level, 0));
        let entry_width = if level == 0 {
            usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113")
        } else {
            usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110")
        };
        if !self.position_and_entry_width_hold(
            &view,
            level,
            &extent_upper_cell_range(level, index),
            entry_width,
            &what,
        ) {
            return;
        }
        let span = extent_upper_span_in_inodes(level);
        let first_inode = index.saturating_mul(span);
        let last_inode = first_inode.saturating_add(span - 1);
        let previous_parent_unit = self.start_walking_below(node_placement);
        if level == 0 {
            for entry in &view.entries {
                let entry_view = extent_upper_leaf_entry_view(entry);
                let inode_of_the_entry = match &entry_view {
                    ExtentUpperLeafEntryView::NoDataUnit { inode }
                    | ExtentUpperLeafEntryView::LowerSegmentRoot { inode, .. }
                    | ExtentUpperLeafEntryView::InlineDataUnit { inode, .. } => Some(*inode),
                    ExtentUpperLeafEntryView::Malformed => None,
                };
                self.judgements.judge(
                    "I-1.1",
                    inode_of_the_entry
                        .is_some_and(|inode| (first_inode..=last_inode).contains(&inode)),
                    || {
                        format!("{what}：一条上段叶条目解不开（标签不是 0 / 1 / 2 或该是零的字节不是零），或它的 inode 号不在这片叶罩的那一段里")
                    },
                );
                match entry_view {
                    ExtentUpperLeafEntryView::LowerSegmentRoot { inode, pointer } => {
                        judge_location_order(
                            &mut self.judgements,
                            &parse_node_pointer(&pointer),
                            &format!("{what} 里 inode {inode} 的下段根指针"),
                        );
                        self.walk_extent_lower_node(&pointer, None, inode, tree);
                    }
                    ExtentUpperLeafEntryView::InlineDataUnit { inode, pointer } => {
                        self.walk_extent_data_pointer(&pointer, inode, 0, tree);
                    }
                    ExtentUpperLeafEntryView::NoDataUnit { .. }
                    | ExtentUpperLeafEntryView::Malformed => {}
                }
            }
            self.stop_walking_below(previous_parent_unit);
            return;
        }
        let key_width = KEY_SCHEMA_EXTENT.width();
        let mut previous_child: Option<(u8, u64)> = None;
        for entry in &view.entries {
            let child = extent_upper_child_of_entry_key(level, index, &entry[..key_width])
                .filter(|child| previous_child.is_none_or(|previous| previous < *child));
            self.judgements.judge("I-1.1", child.is_some(), || {
                format!("{what}：一条内部条目的 key 不是这个节点里一个孩子那一段的起点，或孩子不按位置严格递增")
            });
            let Some(child) = child else {
                self.walk_failures.push(format!(
                    "{what}：内部条目指不到一个合法的孩子，这一支走不下去"
                ));
                continue;
            };
            previous_child = Some(child);
            let child_pointer_bytes = &entry[key_width..key_width + node_pointer_bytes()];
            judge_location_order(
                &mut self.judgements,
                &parse_node_pointer(child_pointer_bytes),
                &format!("{what} 的子指针"),
            );
            self.walk_extent_upper_node(child_pointer_bytes, Some(child), tree);
        }
        self.stop_walking_below(previous_parent_unit);
    }

    /// 一个文件 extent 树下段的一个节点连同它下面（按数据单元号按位置寻址）：根那一个（`None`）的层级取它自己头里的、序号 0。
    /// 每个节点核层级、位置规定的那一段、条目宽（叶 112、内部 110）；叶里每条 extent 叶记录的 inode 段是这个文件、offset 段是
    /// 单元号 × 净荷容量、落在叶罩的那一段里，再走它指的数据单元。
    fn walk_extent_lower_node(
        &mut self,
        pointer_bytes: &[u8],
        position: Option<(u8, u64)>,
        inode: u64,
        tree: u64,
    ) {
        let what = match position {
            None => format!("extent 树（树 {tree}）inode {inode} 下段的根"),
            Some((level, index)) => {
                format!("extent 树（树 {tree}）inode {inode} 下段层级 {level} 第 {index} 格的节点")
            }
        };
        let IndexNodeReading::WalkedFirstByThisVersion {
            view,
            placement_it_was_read_from: node_placement,
        } = self.read_index_node(
            pointer_bytes,
            tree,
            KEY_SCHEMA_EXTENT,
            &what,
            KeyRangeReading::PrescribedByThePositionJudgedByTheCaller,
            EntryFieldTableRegistration::Registered,
        )
        else {
            return;
        };
        let (level, index) = position.unwrap_or((view.level, 0));
        let entry_width = if level == 0 {
            extent_leaf_record_bytes()
        } else {
            usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110")
        };
        if !self.position_and_entry_width_hold(
            &view,
            level,
            &extent_lower_cell_range(inode, level, index),
            entry_width,
            &what,
        ) {
            return;
        }
        let previous_parent_unit = self.start_walking_below(node_placement);
        if level == 0 {
            let span = extent_lower_span_in_data_units(0);
            let first_unit = index.saturating_mul(span);
            let last_unit = first_unit.saturating_add(span - 1);
            let payload = data_unit_payload_capacity_in_bytes();
            for record in &view.entries {
                let (record_inode, offset) = (read_u64(record, 8), read_u64(record, 16));
                self.judgements.judge(
                    "I-1.1",
                    read_u64(record, 0) == 0
                        && record_inode == inode
                        && offset.is_multiple_of(payload)
                        && (first_unit..=last_unit).contains(&(offset / payload)),
                    || {
                        format!("{what}：extent 叶记录的 key（inode {record_inode}、偏移 {offset}）不是 (0, 这个文件, 单元号 × 净荷容量)，或不落在这片叶罩的那一段里")
                    },
                );
                self.walk_extent_data_pointer(&record[24..112], record_inode, offset, tree);
            }
            self.stop_walking_below(previous_parent_unit);
            return;
        }
        let key_width = KEY_SCHEMA_EXTENT.width();
        let mut previous_child: Option<(u8, u64)> = None;
        for entry in &view.entries {
            let child = extent_lower_child_of_entry_key(inode, level, index, &entry[..key_width])
                .filter(|child| previous_child.is_none_or(|previous| previous < *child));
            self.judgements.judge("I-1.1", child.is_some(), || {
                format!("{what}：一条内部条目的 key 不是这个文件里一个孩子那一段的起点，或孩子不按位置严格递增")
            });
            let Some(child) = child else {
                self.walk_failures.push(format!(
                    "{what}：内部条目指不到一个合法的孩子，这一支走不下去"
                ));
                continue;
            };
            previous_child = Some(child);
            let child_pointer_bytes = &entry[key_width..key_width + node_pointer_bytes()];
            judge_location_order(
                &mut self.judgements,
                &parse_node_pointer(child_pointer_bytes),
                &format!("{what} 的子指针"),
            );
            self.walk_extent_lower_node(child_pointer_bytes, Some(child), inode, tree);
        }
        self.stop_walking_below(previous_parent_unit);
    }
}

/// I-7.8 扫描方向的第二道：一个码 2 节点是不是「写序实例 i 崩在发布之前写出的孤儿」——最新根指着的实例表里有 i 的一行
/// (i, Ti, ·)、Ti > 0，而节点的诞生代号 > Ti（改法 D，`research/prompts/m2-wave3-code-r1-main-verification.md`
/// 第三节 Y1 与第四节第 1 条，被攻过零轮）。
///
/// 为什么排除：行 (i, Ti, ·) 说实例 i 被施加到 Ti 为止（D23（journal 的角色与格式） 已定项 14，写行时取恢复之后的有效根），
/// 诞生代号 > Ti 的实例 i 节点就没有发布过。只数诞生代号 ≤ 根环最大 txg 那一道挡不住它：下一个实例第一次发布取的 txg 可以
/// 等于孤儿的诞生代号（D23（journal 的角色与格式） 已定项 14 注 3 的「≥ max(…) + 1」取等号），孤儿就被数进「出现过的」、在合法状态上判红。
/// 不排除 Ti = 0 的行（恢复写的中间实例行 (i, 0, 0) 与没发布过根的实例行，行里说不出它发布到哪一代）。
/// 节点头 fsid 不是本池的读不出写序实例（`unit_write_order_instance` 交 None），照旧算。
fn written_after_its_instance_was_last_applied(
    node: &[u8],
    birth_txg: u64,
    instance_table_rows: &[InstanceTableRow],
    filesystem_identifier_low: u64,
) -> bool {
    unit_write_order_instance(node, filesystem_identifier_low).is_some_and(|write_order_instance| {
        instance_table_rows.iter().any(|row| {
            row.instance == write_order_instance
                && row.published_checkpoint_txg > 0
                && birth_txg > row.published_checkpoint_txg
        })
    })
}

/// 扫描方向认领一个单元之后跳过几槽（D18（块里携带什么信息） 已定项 7 认领规则，用户 2026-10-03 定按整单元跳）：
/// 起点槽上解得出一个单元头——magic 对、类标签是 1 / 2 / 3、头校验和过（码 1 明文头末尾 105、码 2 86 + 2k、码 3 107，校验和都在 10）——
/// 就认领它：码 1 数据单元、码 3 打包记录单元两槽，码 2 节点一槽；被认领的单元占着的后几槽是它的载荷，那里的探针不再参与认领。
/// 不看 fsid：别的池的单元同样占着它那几槽（它是不是本池的、发没发布，归 I-1.4 / I-1.2）。解不出头 ⇒ None，这一槽不认领、下一槽照扫。
fn slots_claimed_by_a_unit_header(header: &[u8]) -> Option<u64> {
    if header.len() < 52 || &header[..4] != b"SFSU" {
        return None;
    }
    let (plain_header_end, slots_of_the_unit) = match header[6] {
        1 => (105, DATA_UNIT_BYTES / SLOT_BYTES),
        2 => (86 + 2 * usize::from(header[51]), NODE_BYTES / SLOT_BYTES),
        3 => (107, DATA_UNIT_BYTES / SLOT_BYTES),
        _outside_the_three_unit_classes => return None,
    };
    (plain_header_end <= header.len()
        && checksum_field_holds(header, plain_header_end, UNIT_HEADER_CHECKSUM_OFFSET))
    .then_some(slots_of_the_unit)
}

/// 扫描方向：候选槽上头校验和过的码 2 节点，取它头里的树 ID。槽按从小到大扫、照认领规则按整单元跳
/// （[`slots_claimed_by_a_unit_header`]）：用户文件内容里恰好对齐到槽边界、头校验和自洽的码 2 伪头落在它那个数据单元的第二槽，
/// 不认领就被数成「出现过的」树 ID，合法镜像上 I-7.8 误红（判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md`
/// 第二节 S2 第一行，改法 F4）。只数诞生代号不超过 `newest_published_txg` 的节点：
/// 根环里没有一条根发布过的那一代，单元还不算「出现过」（I-7.8 的这一读法 2026-09-14 用户收尾弹窗定甲；字面读法会把
/// 崩在发布之前的那个事务写出的孤儿节点也算进去，见 records 十一·七）。再排除最新根的实例表判得出没发布过的孤儿
/// （[`written_after_its_instance_was_last_applied`]）。
///
/// 认领规则留的缝（判决同一行「F4 只修一半」，C89（单元头收口的遗留重算） 第 ② 笔）：释放了的两槽数据单元，第一槽被一槽的码 2 节点拿走、
/// 第二槽的旧载荷还在时，那个节点只认领一槽，旧载荷里的伪头照样被数进来。
///
/// 循环：每块盘按槽号升序走一遍候选槽（轮数 = 候选槽数），跨轮携带「下一个没被认领的槽」，只增不减；没有提前出口。
fn scanned_tree_identifiers(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    newest_published_txg: u64,
    instance_table_rows: &[InstanceTableRow],
    filesystem_identifier_low: u64,
) -> BTreeSet<u64> {
    let mut identifiers = BTreeSet::new();
    for device in reader.devices() {
        let mut slots = reader.candidate_unit_slots(device).unwrap_or_else(|| {
            let end = reader.device_bytes(device).unwrap_or(0) / SLOT_BYTES;
            (geometry.unit_area_start_slot..end).collect()
        });
        slots.sort_unstable();
        slots.dedup();
        let mut first_slot_not_claimed = 0u64;
        for slot in slots {
            if slot < first_slot_not_claimed {
                continue;
            }
            let Some(bytes) = reader.read(device, slot * SLOT_BYTES, node_bytes()) else {
                continue;
            };
            let Some(claimed_slots) = slots_claimed_by_a_unit_header(&bytes) else {
                continue;
            };
            first_slot_not_claimed = slot + claimed_slots;
            if bytes[6] != 2 {
                continue;
            }
            let header_end = 86 + 2 * usize::from(bytes[51]);
            if header_end < bytes.len() && checksum_field_holds(&bytes, header_end, 10) {
                let birth_txg = read_u64(&bytes, 52 + 2 * usize::from(bytes[51]));
                if birth_txg <= newest_published_txg
                    && !written_after_its_instance_was_last_applied(
                        &bytes,
                        birth_txg,
                        instance_table_rows,
                        filesystem_identifier_low,
                    )
                {
                    identifiers.insert(read_u64(&bytes, 42));
                }
            }
        }
    }
    identifiers
}

/// 扫描方向读单元头读多少：码 2 头最宽 86 + 2 × 255 = 596 字节，读一块 4096 就罩得住三类。
const UNIT_HEADER_SCAN_BYTES: usize = 4096;

/// 单元头写序里的实例代号：头校验和过、fsid 与本池相同才算（码 1：fsid 83、写序 91、头末 105；码 2：fsid 60 + 2k、
/// 写序 68 + 2k、头末 86 + 2k；码 3：fsid 81、写序 93、头末 107——D18（块里携带什么信息） 已定项 18 的偏移表）。
fn unit_write_order_instance(bytes: &[u8], filesystem_identifier_low: u64) -> Option<u32> {
    if bytes.len() < 52 || &bytes[..4] != b"SFSU" {
        return None;
    }
    let (header_end, fsid_offset, instance_offset) = match bytes[6] {
        1 => (105, 83, 91),
        2 => {
            let key_span = 2 * usize::from(bytes[51]);
            (86 + key_span, 60 + key_span, 68 + key_span)
        }
        3 => (107, 81, 93),
        _ => return None,
    };
    if header_end > bytes.len()
        || !checksum_field_holds(bytes, header_end, 10)
        || read_u64(bytes, fsid_offset) != filesystem_identifier_low
    {
        return None;
    }
    Some(read_u32(bytes, instance_offset))
}

/// 盘上带实例代号的东西：根环里自证过的根、journal 环里 fsid 相同的记录、单元区里头校验和过且 fsid 相同的单元写序。
/// 读得出的都收进 `carriers`；有一个该读的槽读不出时照样往下读，只记下 `some_slot_is_unreadable`（I-7.7（系统配置实例代号不低于根环）
/// ① 那一半读不全报不适用，② 在读得出的这些上照判）。
struct InstanceCarriersOnDisk {
    carriers: Vec<(String, u32)>,
    some_slot_is_unreadable: bool,
}

fn instance_carriers(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    filesystem_identifier_low: u64,
) -> InstanceCarriersOnDisk {
    let mut some_slot_is_unreadable = false;
    let root_slot_bytes = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    for (_, _, device, offset) in root_slot_positions(geometry) {
        if reader.read(device, offset, root_slot_bytes).is_none() {
            some_slot_is_unreadable = true;
        }
    }
    let mut carriers: Vec<(String, u32)> = roots
        .iter()
        .map(|(region, slot, root)| (format!("根环区域 {region} 槽 {slot} 的根"), root.instance))
        .collect();
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    for device in reader.devices() {
        let journal_slots = reader
            .candidate_journal_slots(device)
            .unwrap_or_else(|| (0..geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES).collect());
        for slot in journal_slots {
            let offset =
                geometry.journal_ring_start_slot * SLOT_BYTES + slot * JOURNAL_RECORD_BYTES;
            let Some(bytes) = reader.read(device, offset, record_bytes) else {
                some_slot_is_unreadable = true;
                continue;
            };
            if let Ok(record) = check_journal_record(&bytes, filesystem_identifier_low) {
                carriers.push((
                    format!("盘 {device} journal 环槽 {slot} 的记录"),
                    record.instance,
                ));
            }
        }
        let unit_slots = reader.candidate_unit_slots(device).unwrap_or_else(|| {
            let end = reader.device_bytes(device).unwrap_or(0) / SLOT_BYTES;
            (geometry.unit_area_start_slot..end).collect()
        });
        for slot in unit_slots {
            let Some(bytes) = reader.read(device, slot * SLOT_BYTES, UNIT_HEADER_SCAN_BYTES) else {
                some_slot_is_unreadable = true;
                continue;
            };
            if let Some(instance) = unit_write_order_instance(&bytes, filesystem_identifier_low) {
                carriers.push((format!("盘 {device} 槽 {slot} 的单元写序"), instance));
            }
        }
    }
    InstanceCarriersOnDisk {
        carriers,
        some_slot_is_unreadable,
    }
}

/// I-7.7（系统配置实例代号不低于根环） 的 ① ②（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：
/// 每盘的实例代号取它两槽里全部自证过（fsid 与本池相同）的槽中最大的；① 本池的每个根记录、journal 记录、单元写序的实例代号
/// 不超过各盘的最大者，② 各盘不等时较大者不出现在它们里——两句合起来就是「每一个带号的东西都不超过各盘最大号里最小的那个」。
fn judge_instance_carriers(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    judgements: &mut Judgements,
) {
    let highest_by_device: Vec<Option<u32>> = verified_system_configuration_slots(reader)
        .into_iter()
        .map(|(_, slots)| {
            slots
                .iter()
                .filter(|(view, _)| view.filesystem_identifier == geometry.filesystem_identifier)
                .map(|(view, _)| view.journal_instance)
                .max()
        })
        .collect();
    let Some(highest_by_device) = highest_by_device.into_iter().collect::<Option<Vec<u32>>>()
    else {
        judgements.not_applicable(
            "I-7.7",
            "有一块盘没有本池 fsid 的系统配置槽：那块盘归不归这个池由 I-1.4 判",
        );
        return;
    };
    let filesystem_identifier_low = u64::from_le_bytes(
        geometry.filesystem_identifier[..8]
            .try_into()
            .expect("fsid 前 8 字节"),
    );
    let InstanceCarriersOnDisk {
        carriers,
        some_slot_is_unreadable,
    } = instance_carriers(reader, geometry, roots, filesystem_identifier_low);
    let pool_highest = highest_by_device.iter().copied().max().unwrap_or(0);
    let pool_lowest = highest_by_device.iter().copied().min().unwrap_or(0);
    // ① 条款逐字「任一根环区域、journal 记录槽、单元头读不出时这一半报「不适用」」：读不全时这一半不判，整条不报成立。
    // ② 的不适用条件是另一件事（镜像记不下这次挂载独占打开的集合），读不全时照样在读得出的载体上判：找到较大者就红；
    // 找不到时读不出的那几个槽里可能就坐着它，这一半也判不了（代码审阅 6c 第 9 条：原来一个槽读不出就 ① ② 一起报不适用）。
    if some_slot_is_unreadable {
        judgements.part_of_the_range_not_judged(
            "I-7.7",
            "根环、journal 环或单元区有读不出的槽：① 那一半按条款报不适用，② 在读得出的载体上没找到较大者，也判不全",
        );
    } else {
        let above_every_disk = carriers
            .iter()
            .find(|(_, instance)| *instance > pool_highest);
        judgements.judge("I-7.7", above_every_disk.is_none(), || {
            let (what, instance) = above_every_disk.expect("判红时有");
            format!("{what}带实例代号 {instance}，高于各盘系统配置的最大者 {pool_highest}（①）")
        });
    }
    // ② 只在各盘的号不等时有对象（「各盘的实例代号……不等时，较大者不出现在……」）；各盘相等时高于它的载体是 ① 那一半的事。
    let disks_disagree = pool_lowest < pool_highest;
    let witnessed_by_one_disk = carriers
        .iter()
        .find(|(_, instance)| disks_disagree && *instance > pool_lowest);
    if witnessed_by_one_disk.is_some() || !some_slot_is_unreadable {
        judgements.judge("I-7.7", witnessed_by_one_disk.is_none(), || {
            let (what, instance) = witnessed_by_one_disk.expect("判红时有");
            format!("各盘实例代号 {highest_by_device:?} 不等，而{what}带着较大的 {instance}（②）")
        });
    }
}

/// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备身份 4, 16 KiB 槽号 6) + value (跨度段 2，最高位是已释放标志,
/// 分配代或释放代 8)。checker 按字段表自己解一份，不与实现共用代码（D13（验证路线） 已定项 5）。
const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
const ALLOCATION_RECORD_SPAN_FIELD_OFFSET: usize = 10;
const ALLOCATION_RECORD_GENERATION_OFFSET: usize = 12;
/// 树表条目 200（D8（核心索引结构） 已定项 8）：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 +
/// `previous_snapshot_txg` 8 + **诞生 txg 8** + 头 ID 8 + 预留 76。
const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8;
/// 树表条目里种类 2 字节的偏移（树 ID 8 + 条目长度 2 之后）。
const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 8 + 2;
/// 树表条目里头 ID 8 字节的偏移（树 ID 8 + 条目长度 2 + 种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8，
/// D8（核心索引结构） 已定项 8 的字段表），I-9.27 判它。
const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8 + 8;
/// 一个数据单元占几个 16 KiB 槽（32768 字节，起点 32768 对齐 ⇒ 起点槽号是它的整数倍）：I-5.5 按槽对找落点。
const DATA_UNIT_SLOTS: u64 = DATA_UNIT_BYTES / SLOT_BYTES;

/// I-9.27：树表条目的头 ID 按种类该等于什么（`.claude/kb/invariants.md` I-9.27 那一行；D8（核心索引结构） 已定项 8「头 ID 8 字节的取值」
/// 句末的关系判据：判的是关系，不拿字面 65537 比）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeadIdentifierOfATreeTableEntry {
    /// 种类 2（inode 树）：等于这条条目自己的树 ID。
    ItsOwnTreeIdentifier,
    /// 种类 1（extent 树）、9（`deleted_inodes`）、10（墓碑树）、13（退役树）：等于同一张树表里种类 2 条目的树 ID；
    /// 种类 2 条目多于一条时放宽成「等于其中某一条的」（收严留给多头那一批），一条都没有时判红。
    TreeIdentifierOfAnInodeTreeEntryOfTheSameTable,
    /// 种类 3、4、6、7、8、11、12（无归属，D5（快照 / 空间记账机制） 已定项 9）：等于 0。
    Zero,
    /// 别的种类码：那一行不判。
    NotJudged,
}

impl HeadIdentifierOfATreeTableEntry {
    const fn of_tree_kind(kind: u16) -> Self {
        match kind {
            TREE_KIND_INODE => Self::ItsOwnTreeIdentifier,
            TREE_KIND_EXTENT
            | TREE_KIND_DELETED_INODES
            | TREE_KIND_TOMBSTONE
            | TREE_KIND_RETIREMENT => Self::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable,
            TREE_KIND_ALLOCATION
            | TREE_KIND_ACCOUNTING
            | TREE_KIND_LIVELIST
            | TREE_KIND_SPARSE_SIDE_TABLE
            | TREE_KIND_DEADLIST
            | TREE_KIND_LINEAGE
            | TREE_KIND_LOGGED_OPERATIONS => Self::Zero,
            _ => Self::NotJudged,
        }
    }

    /// 判红时写进第一处违例的那半句：这一条的头 ID 该等于什么。
    fn expectation(self, tree: u64, inode_tree_identifiers: &[u64]) -> String {
        match self {
            Self::ItsOwnTreeIdentifier => format!("inode 树条目的头 ID 该等于它自己的树 ID {tree}"),
            Self::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable => format!(
                "该等于同一张树表里种类 2 条目的树 ID（这张表里的种类 2 条目：{inode_tree_identifiers:?}）"
            ),
            Self::Zero => "该等于 0".to_string(),
            Self::NotJudged => "这一种不判".to_string(),
        }
    }
}

/// I-9.16（树表条目按树 ID 严格升序且合发号次序）：一版的树表判一次，四道任一道不成立即判红，第一处违例写明是哪一版、哪两条（下标从 0 数）
/// 或哪一种。
/// ① 盘上相邻两条的树 ID（条目打头 8 字节）严格升序；② 按发号次序（[`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`]），
/// 树表里有的种类相邻两棵的树 ID 严格升序；③ 同一张树表里每一种种类码（登记过的与没登记的都算）至多一条；④ 树表非空时
/// 进树表的十二种（[`TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES`]，第 50 批加了退役树）各恰好一条
/// （判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y3「种类 1 不止一条；种类 3 不是恰好一条」那一行；
/// 十一种取自判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-9.16 ④ 放过缺种类的树表（C1）」那一行）。
/// 0 条的树表四道都成立，1 条的 ① ② ③ 成立。checker 自己按字段表切一份，不与实现共用代码（D13（验证路线） 已定项 5；
/// 实现那一侧是 `crates/singlefs-core/src/recovery.rs` 的 `tree_table_entries_each_kind_at_most_once` 与重建一版时缺了有根的树报的
/// 「树表里缺一棵有根的树」）。③ ④ 先于 ② 判：同一张表两道都不成立时第一处违例写 ③ ④ 那一道。
/// ① 与 I-1.1 在树表上判的「key 严格递增」是同一个谓词（树表的 key 就是树 ID）：只违反 ① 的镜像两条一起红，只违反 ② 的只红这一条。
/// 条目宽不到种类那 2 字节（盘上字段，`index_node_view` 只保证 ≥ key 宽 8）时 ② 读不出种类：只判 ①，② 记成射程里判不了的那一部分。
fn judge_tree_table_entries_ordering(
    judgements: &mut Judgements,
    tree_table_entries: &[Vec<u8>],
    version_name: &str,
) {
    let tree_identifier_of = |entry: &Vec<u8>| read_u64(entry, 0);
    let first_pair_out_of_the_on_disk_order = tree_table_entries
        .windows(2)
        .enumerate()
        .find(|(_, pair)| tree_identifier_of(&pair[0]) >= tree_identifier_of(&pair[1]));
    if let Some((index, pair)) = first_pair_out_of_the_on_disk_order {
        judgements.judge("I-9.16", false, || {
            format!(
                "{version_name}的树表：第 {index} 条（树 ID {}）与第 {} 条（树 ID {}）不按树 ID 严格升序（① 盘上次序）",
                tree_identifier_of(&pair[0]),
                index + 1,
                tree_identifier_of(&pair[1])
            )
        });
        return;
    }
    let kinds_are_readable = tree_table_entries
        .iter()
        .all(|entry| entry.len() >= TREE_TABLE_ENTRY_KIND_OFFSET + 2);
    if !kinds_are_readable {
        judgements.part_of_the_range_not_judged(
            "I-9.16",
            "树表单元自述的条目宽不到种类那 2 字节：② 发号次序读不出种类，只判了 ① 盘上次序",
        );
        judgements.judge("I-9.16", true, String::new);
        return;
    }
    judge_tree_table_entry_kind_counts(judgements, tree_table_entries, version_name);
    // (种类, 下标, 树 ID)，按发号次序；树表里没有的种类跳过。同一种有两条时取先出现的那条（那一张表 ③ 已判红）。
    let entries_in_the_issuing_order: Vec<(u16, usize, u64)> =
        TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER
            .iter()
            .filter_map(|kind| {
                tree_table_entries
                    .iter()
                    .enumerate()
                    .find(|(_, entry)| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == *kind)
                    .map(|(index, entry)| (*kind, index, tree_identifier_of(entry)))
            })
            .collect();
    let first_pair_out_of_the_issuing_order = entries_in_the_issuing_order
        .windows(2)
        .find(|pair| pair[0].2 >= pair[1].2);
    judgements.judge("I-9.16", first_pair_out_of_the_issuing_order.is_none(), || {
        let pair = first_pair_out_of_the_issuing_order.expect("判红时找到过那一对");
        format!(
            "{version_name}的树表：种类 {} 那条（第 {} 条，树 ID {}）与发号次序上的下一棵、种类 {} 那条（第 {} 条，树 ID {}）不按树 ID 严格升序（② 发号次序）",
            pair[0].0, pair[0].1, pair[0].2, pair[1].0, pair[1].1, pair[1].2
        )
    });
}

/// I-9.16 ④（[`judge_tree_table_entry_kind_counts`]）：树表非空时各要恰好一条的种类，就是进树表的那十二种
/// （[`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`]，同一份，不另抄）。读者重建一版时十三棵树的号逐棵从树表取：
/// 十二棵按种类找条目、取条目里的树 ID，中央映射树不进树表、取根记录里它那条根指针的出生树；缺哪一种都重建不出这一版
/// （实现那一侧是 `crates/singlefs-core/src/recovery.rs` 的 `rebuild_version` 逐种取号时报的「树表里没有那棵树」）。
/// 只读前四种根指针的那几行读法前提读窄了（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1 C1 那一行更正）。
const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 12] =
    TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER;

/// I-9.16 的 ③ ④（[`judge_tree_table_entries_ordering`]）：③ 每一种种类码至多一条；④ 树表非空时
/// [`TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES`] 各恰好一条。条目都读得出种类那 2 字节（调用方判过）。
/// 0 条的表不判（④ 的前提不成立，③ 没有对象）。第一处违例写明是哪一版、哪一种、几条。
fn judge_tree_table_entry_kind_counts(
    judgements: &mut Judgements,
    tree_table_entries: &[Vec<u8>],
    version_name: &str,
) {
    if tree_table_entries.is_empty() {
        return;
    }
    let mut entries_of_each_kind: BTreeMap<u16, usize> = BTreeMap::new();
    for entry in tree_table_entries {
        *entries_of_each_kind
            .entry(read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET))
            .or_insert(0) += 1;
    }
    let first_kind_carried_more_than_once = entries_of_each_kind
        .iter()
        .find(|(_, entries)| **entries > 1)
        .map(|(kind, entries)| (*kind, *entries));
    judgements.judge(
        "I-9.16",
        first_kind_carried_more_than_once.is_none(),
        || {
            let (kind, entries) = first_kind_carried_more_than_once.expect("判红时找到过那一种");
            format!("{version_name}的树表：种类 {kind} 有 {entries} 条（③ 每种至多一条）")
        },
    );
    let first_rooted_kind_missing = TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES
        .iter()
        .copied()
        .find(|kind| !entries_of_each_kind.contains_key(kind));
    judgements.judge("I-9.16", first_rooted_kind_missing.is_none(), || {
        let kind = first_rooted_kind_missing.expect("判红时找到过那一种");
        format!(
            "{version_name}的树表有 {} 条，种类 {kind} 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
            tree_table_entries.len()
        )
    });
}

/// I-9.27（树表条目的头 ID 取值）：一版的树表判一次（树表被几版共享时只在第一个走到它的那一版判，同 I-9.16），每条条目按种类判一格
/// （[`HeadIdentifierOfATreeTableEntry`]），第一处违例写明是哪一版、第几条（下标从 0 数）。树表 0 条的版本一格也不判。
/// checker 自己按字段表切一份，不与实现共用代码（D13（验证路线） 已定项 5）。
/// 条目宽不到头 ID 那 8 字节（盘上字段，`index_node_view` 只保证 ≥ key 宽 8）时读不出头 ID：这张树表不判，记成射程里判不了的那一部分
/// （同 I-9.16 读不出种类那一格的处置）。
fn judge_tree_table_entry_head_identifiers(
    judgements: &mut Judgements,
    tree_table_entries: &[Vec<u8>],
    version_name: &str,
) {
    let head_identifiers_are_readable = tree_table_entries
        .iter()
        .all(|entry| entry.len() >= TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8);
    if !head_identifiers_are_readable {
        judgements.part_of_the_range_not_judged(
            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
            "树表单元自述的条目宽不到头 ID 那 8 字节（条目偏移 116 起）：这张树表的头 ID 读不出",
        );
        return;
    }
    let tree_identifier_of = |entry: &Vec<u8>| read_u64(entry, 0);
    let kind_of = |entry: &Vec<u8>| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET);
    let inode_tree_identifiers: Vec<u64> = tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_INODE)
        .map(tree_identifier_of)
        .collect();
    for (index, entry) in tree_table_entries.iter().enumerate() {
        let tree = tree_identifier_of(entry);
        let kind = kind_of(entry);
        let head_identifier = read_u64(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
        let expected = HeadIdentifierOfATreeTableEntry::of_tree_kind(kind);
        let holds = match expected {
            HeadIdentifierOfATreeTableEntry::ItsOwnTreeIdentifier => head_identifier == tree,
            HeadIdentifierOfATreeTableEntry::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable => {
                inode_tree_identifiers.contains(&head_identifier)
            }
            HeadIdentifierOfATreeTableEntry::Zero => head_identifier == 0,
            HeadIdentifierOfATreeTableEntry::NotJudged => continue,
        };
        judgements.judge(HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES, holds, || {
            format!(
                "{version_name}的树表：第 {index} 条（树 ID {tree}、种类 {kind}）的头 ID 是 {head_identifier}，{}",
                expected.expectation(tree, &inode_tree_identifiers)
            )
        });
    }
}

/// 一条分配记录在盘上的样子。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AllocationRecordView {
    device: u32,
    slot: u64,
    /// 跨度段去掉已释放标志之后的值：这条记录罩住 [槽号, 槽号 + 跨度) 这些槽。
    span_slots: u64,
    /// 仍分配时是分配代；已释放时是释放代（D3（空间分配） 已定项 7）。
    generation: u64,
    is_released: bool,
}

fn parse_allocation_record(entry: &[u8]) -> AllocationRecordView {
    let span_field = read_u16(entry, ALLOCATION_RECORD_SPAN_FIELD_OFFSET);
    AllocationRecordView {
        device: read_u32(entry, 0),
        slot: read_six_byte_unsigned(entry, 4),
        span_slots: u64::from(span_field & !ALLOCATION_RECORD_RELEASED_FLAG),
        generation: read_u64(entry, ALLOCATION_RECORD_GENERATION_OFFSET),
        is_released: span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0,
    }
}

/// 树的种类码（D8（核心索引结构） 已定项 9 的登记表）在引用集合这一遍里的读法：盘上读到的码语义上允许未知，
/// 做成一个显式成员，`match` 照样穷举（`code-discipline.md`「分支：穷举写全，不写通配臂」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeKindForReferenceScan {
    Extent,
    Inode,
    Allocation,
    Accounting,
    /// `deleted_inodes`：只有一个码 2 根（层级 0），条目不引用别的单元。
    DeletedInodes,
    /// 墓碑树、谱系树：码 2 根之下是码 3 容器（条目照 inode 树内部节点的 120 字节形态）。
    ContainerTree,
    /// 条目格式还没有条款的树（livelist 6、稀疏旁表 7、deadlist 8、logged_ops 12 与这一版不认识的码）：day-1 注册的那几棵根指针为零、
    /// 走到这里就是它们真有了根节点，那时这条根的引用集合数不全。
    WithoutWalkableEntryFormat(u16),
}

impl TreeKindForReferenceScan {
    const fn of(code: u16) -> Self {
        match code {
            TREE_KIND_EXTENT => TreeKindForReferenceScan::Extent,
            TREE_KIND_INODE => TreeKindForReferenceScan::Inode,
            TREE_KIND_ALLOCATION => TreeKindForReferenceScan::Allocation,
            TREE_KIND_ACCOUNTING => TreeKindForReferenceScan::Accounting,
            TREE_KIND_DELETED_INODES => TreeKindForReferenceScan::DeletedInodes,
            TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE | TREE_KIND_RETIREMENT => {
                TreeKindForReferenceScan::ContainerTree
            }
            unregistered => TreeKindForReferenceScan::WithoutWalkableEntryFormat(unregistered),
        }
    }
}

/// 一条有效根自己引用了哪些落点、它的树表里有哪些条目：I-3.9（释放代落在停止引用它的那一格区间里）与
/// I-9.14（树表条目的诞生 txg 跨根不变）按回退候选集里每条根各走一遍取这份东西（I-7.4（近 K 代块未被复用） 已经这么走）。
/// **与主走读那一遍分开跑**：主走读用 `visited_units` 跳过走过的单元，共享子树只算进第一条走到它的根，
/// 而这两条要的正是「这条根自己引用了什么」；这一遍只读不判，别的不变量的评估次数不受它影响。
#[derive(Clone)]
struct RootReferences {
    /// 这条根引用的落点 (设备身份, 起点槽)：与分配记录的 key 同形。只在 `depth` 是「走到叶」时数得全。
    placements: BTreeSet<(u32, u64)>,
    /// 树表条目：树 ID → 诞生 txg。
    tree_table_birth_txg: BTreeMap<u64, u64>,
    /// 树表单元自己落在哪 (设备身份, 起点槽)：几条根共用同一个树表单元时，它们的条目不是两处独立的证据。
    tree_table_placement: Option<(u32, u64)>,
    /// 树表读成什么样：I-9.28 按它列出指针非零而条目没读全的根（报「部分没判」）。
    tree_table_reading: TreeTableReading,
    /// 这条根的分配记录树叶里的记录（I-3.9 只取最新根那一份：当前的账）。
    allocation_records: Vec<AllocationRecordView>,
    /// 这一遍按哪个深度扫的：`placements` 与 `is_complete` 只有「走到叶」那个深度上作数。
    depth: ReferenceScanDepth,
    /// 该读的单元读不出、或走到了没有条款的形状：这条根的引用集合数不全，I-3.9 判不了。
    is_complete: bool,
}

/// 一条根的树表这一遍读成什么样（[`references_of_root`] 只读不判；I-9.28 拿它判「部分没判」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeTableReading {
    /// 树表指针全零：core 的根记录读者照它读盘 0 槽 0、读不出（用户 2026-10-05 定甲），与 [`TreeTableReading::Unreadable`] 一样算没读到，
    /// 不再算「这条根没有树表」。
    PointerAllZero,
    /// 指针非零，两条位置条目都读不出、或整单元校验和对不上、或解不开成码 2 节点。
    Unreadable,
    /// 读出来了，有条目比字段表短（那几条不解字段）。
    SomeEntryShorterThanTheFieldTable,
    /// 读出来了，每条条目都按字段表解了。
    EveryEntryRead,
}

impl RootReferences {
    fn note(&mut self, pointer: &PointerView) {
        if pointer.all_zero {
            return;
        }
        for location in &pointer.locations {
            self.placements.insert((location.device, location.slot));
        }
    }
}

/// 按位置条目读一个单元、不判任何不变量：两条位置条目里第一份校验和对得上的那一份（判定归主走读的 I-2.1）。
fn read_unit_without_judging(
    reader: &dyn ImageReader,
    pointer: &PointerView,
    unit_bytes: usize,
) -> Option<Vec<u8>> {
    pointer.locations.iter().find_map(|location| {
        reader
            .read(location.device, location.slot * SLOT_BYTES, unit_bytes)
            .filter(|content| crc32_castagnoli_table(content) == location.checksum)
    })
}

/// 共同前缀里类标签的偏移（D18（块里携带什么信息） 已定项 7 / 11：三种码同一处）。
const UNIT_HEADER_CLASS_TAG_OFFSET: usize = 6;

/// 一类单元的整单元长度：码 1 数据单元与码 3 打包记录单元 32 KiB，码 2 节点 16 KiB；登记表之外的码没有长度（交 None）。
fn unit_bytes_of_the_class(class: MappingKeyUnitClass) -> Option<usize> {
    match class {
        MappingKeyUnitClass::DataUnit | MappingKeyUnitClass::PackedRecordUnit => {
            Some(data_unit_bytes())
        }
        MappingKeyUnitClass::IndexNode => Some(node_bytes()),
        MappingKeyUnitClass::Unregistered(_) => None,
    }
}

/// 一类单元占几个 16 KiB 槽（引用集合按它记）：32 KiB 的两槽，16 KiB 的一槽。
fn slots_spanned_by_a_unit_of_the_class(class: MappingKeyUnitClass) -> Option<u64> {
    unit_bytes_of_the_class(class)
        .map(|unit_bytes| u64::try_from(unit_bytes).expect("16384 或 32768") / SLOT_BYTES)
}

/// 树里一条内部条目的孩子是不是条目类型段说的那一类：只读孩子的单元头取类标签，不按类型段定读多长（判决
/// `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N4 那一行：按类型段定长度时，类型段与孩子真实的类不符，
/// 按那个长度读回的整单元校验和对不上，身份那一判跑不到、却记了 I-2.1 与走读失败）。
/// 两条位置条目都看：头里的类标签是登记过的码、且按那一类的长度读回的整单元校验和对得上，这一份才算自证过的孩子；
/// 有一份自证过的孩子是 `class_the_entry_type_names` 之外的一类，交回那一类（两份各自证成两个另一类时交回位置条目在前的那一份的）；
/// 自证过的都是类型段那一类、或哪一份都自证不过（类标签不在登记表里、读不出、校验和对不上：陈旧副本、坏副本），交回 None
/// （照类型段那一支走，读不出照旧归 I-2.1）。
/// 不在第一份自证过的副本处停（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H1 H-V 那一行、第三节第 2 条：
/// 两份按各自的位置条目都自证得过、却是两类时，只看第一份的话结论随那一份在哪块盘翻面）。
/// 头里的类就是类型段那一类的副本不影响结论（判决 `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K2 N1 两故障 H1 那一行：
/// 它是陈旧副本时，第二份照样要看）。
/// 类型段不认得的条目（`class_the_entry_type_names` 是 None）没有「它说的那一类」，交回孩子自证过的那一类。
/// 只认按它自己的长度自证过的孩子：头里的类标签坏了的单元按那个长度对不上校验和，不会被当成「另一类」。只读、不判。
fn child_unit_class_other_than_the_entry_type_names(
    reader: &dyn ImageReader,
    child: &PointerView,
    class_the_entry_type_names: Option<MappingKeyUnitClass>,
) -> Option<MappingKeyUnitClass> {
    child.locations.iter().find_map(|location| {
        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)
            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)
    })
}

/// 一份副本（一条位置条目）按它自己那一类的长度整份自证过时交回那一类：头按最短的单元（码 2，16 KiB）那么长读取类标签
/// （读者只认整块的读，32 KiB 的单元头也在这前 16 KiB 里），类标签是登记过的码、按那一类的长度读回的整单元校验和与位置条目对得上；
/// 类标签没登记、读不出、校验和对不上交 None。走读里「自证过」只有这一份判据（判决 `research/prompts/m3-batch32-code-r1-main-verification.md`
/// 第二节 K1 P2 那一行、第三节第 3 条：先前三处各抄一遍）：认孩子是别类（[`child_unit_class_other_than_the_entry_type_names`]）、
/// 认另一份是类型段那一类（[`some_copy_of_the_child_verifies_as_the_class_the_entry_type_names`]，F-W）、
/// 类型段 0 认「子单元是码 2」（[`class_of_the_first_copy_verifying_at_the_length_of_its_class`]，F-ID-b）都调它。只读、不判。
fn class_of_a_copy_verifying_at_the_length_of_its_class(
    reader: &dyn ImageReader,
    location: &PointerLocation,
) -> Option<MappingKeyUnitClass> {
    let offset = location.slot * SLOT_BYTES;
    let class_in_the_header = MappingKeyUnitClass::of(
        reader.read(location.device, offset, node_bytes())?[UNIT_HEADER_CLASS_TAG_OFFSET],
    );
    let unit_bytes = unit_bytes_of_the_class(class_in_the_header)?;
    reader
        .read(location.device, offset, unit_bytes)
        .filter(|content| crc32_castagnoli_table(content) == location.checksum)
        .map(|_content| class_in_the_header)
}

/// 两条位置条目里第一份按它自己那一类的长度整份自证过的副本是哪一类（[`class_of_a_copy_verifying_at_the_length_of_its_class`]）；
/// 哪一份都没自证过交 None，调用方当读不出（用户 2026-10-07 定）。
/// 墓碑树、谱系树类型段 0 那一支认「子单元是码 2」用它（F-ID-b，用户 2026-10-07 定）。只读、不判。
fn class_of_the_first_copy_verifying_at_the_length_of_its_class(
    reader: &dyn ImageReader,
    child: &PointerView,
) -> Option<MappingKeyUnitClass> {
    child
        .locations
        .iter()
        .find_map(|location| class_of_a_copy_verifying_at_the_length_of_its_class(reader, location))
}

/// [`child_unit_class_other_than_the_entry_type_names`] 认出另一类之后再问一句：有没有一份副本头里的类标签就是 `class_the_entry_type_names`、
/// 按那一类的长度读回的整单元校验和对得上（判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「K2 H-V 两份都看之后判错」那一行、
/// 第三节第 1 条，攻方改法 F-W）。有的话读者用的就是这一份（core 按条目那一类的长度逐份读、对得上就用），走读照类型段那一支走它的子树；
/// 槽数也按这一类登记（C739 后一半）。只在认出另一类之后调：常见的条目不多读。只读、不判。
///
/// # Panics
/// `class_the_entry_type_names` 没有长度：调用方只拿类型段认得的那两类（码 2、码 3）来问。
fn some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(
    reader: &dyn ImageReader,
    child: &PointerView,
    class_the_entry_type_names: MappingKeyUnitClass,
) -> bool {
    assert!(
        unit_bytes_of_the_class(class_the_entry_type_names).is_some(),
        "类型段认得的那一类是码 2 或码 3，都有长度"
    );
    child.locations.iter().any(|location| {
        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)
            == Some(class_the_entry_type_names)
    })
}

/// 走读不往下读的孩子的一份副本（一条位置条目）与位置条目里的整单元校验和对不对得上：头按最短的单元（码 2，16 KiB）那么长读，
/// 头里的类标签是登记过的码就只按那一类的长度比；没登记的码、或头读不出，16 KiB 与 32 KiB 两种长度都对不上才算对不上
/// （判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行的改法 F-α）。只读、不判。
fn copy_matches_its_location_checksum_at_the_length_its_header_names(
    reader: &dyn ImageReader,
    location: &PointerLocation,
) -> bool {
    let offset = location.slot * SLOT_BYTES;
    let length_of_the_class_in_the_header = reader
        .read(location.device, offset, node_bytes())
        .map(|header| MappingKeyUnitClass::of(header[UNIT_HEADER_CLASS_TAG_OFFSET]))
        .and_then(unit_bytes_of_the_class);
    let lengths_to_compare = match length_of_the_class_in_the_header {
        Some(unit_bytes) => vec![unit_bytes],
        None => vec![node_bytes(), data_unit_bytes()],
    };
    lengths_to_compare.into_iter().any(|unit_bytes| {
        reader
            .read(location.device, offset, unit_bytes)
            .is_some_and(|content| crc32_castagnoli_table(&content) == location.checksum)
    })
}

/// inode 树一条内部条目的孩子两份都读不出（[`child_unit_class_other_than_the_entry_type_names`] 也没认出另一类）时，I-9.2 里不看孩子就判得了的那几小句，
/// 与墓碑树、谱系树读不出孩子时同判（`Walk::walk_the_entries_of_a_container_tree_node`；判决
/// `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K1 N1 那一行「inode 树读不出孩子时 I-9.2 不判」）：
/// 类型段 0 判身份引用三段为 0（「子单元是码 2」那一小句孩子读不出不判）；类型段 2 判「子指针头部的出生树 == 身份引用的出生树」
/// （只比父条目里的两段，判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H3 H-B 那一行、第三节第 1 条），
/// 子头四段与身份引用相等、子容器的打包记录类型 == 2 那几小句要比孩子的头，读不出不判。
/// 类型段不认得的条目不走到这里：调用方不往下走它的孩子、判该条目损坏（第 27 批，与墓碑树、谱系树类型段不认得那一支同判），
/// 每一份副本只判 I-2.1（第 29 批，`Walk::judge_every_copy_of_a_child_the_walk_does_not_go_into`）。
/// 读不出照旧归 I-2.1 与走读失败（调用方记）。
fn judge_an_inode_entry_whose_child_does_not_read_back(
    judgements: &mut Judgements,
    entry_type: RegisteredInodeInternalEntryType,
    identity: (u64, u16, u64, u64),
    birth_tree_in_the_child_pointer_head: u64,
) {
    match entry_type {
        RegisteredInodeInternalEntryType::LeafContainer => {
            let identity_birth_tree = identity.0;
            judgements.judge(
                "I-9.2",
                birth_tree_in_the_child_pointer_head == identity_birth_tree,
                || {
                    format!(
                        "inode 内部条目类型段 2：孩子两份都读不出（比孩子头的那几小句不判），子指针头部的出生树 {birth_tree_in_the_child_pointer_head} 不等于身份引用的出生树 {identity_birth_tree}"
                    )
                },
            );
        }
        RegisteredInodeInternalEntryType::InternalNode => {
            let (birth_tree, _, container, container_birth) = identity;
            let identity_is_zero = birth_tree == 0 && container == 0 && container_birth == 0;
            judgements.judge("I-9.2", identity_is_zero, || {
                format!(
                    "inode 内部条目类型段 0：孩子两份都读不出（类标签那一小句不判），身份引用的出生树 / 容器号 / 出生代 {birth_tree} / {container} / {container_birth}（要三段为 0）"
                )
            });
        }
    }
}

/// 这一遍里读过的码 2 单元：(设备身份, 起点槽, 位置条目里的整单元校验和) → 解开的节点（解不开记 None）。
/// 候选集里几条根常指着同一个单元（暖机那几代都指着 mkfs 种的第 0 版树表，共享子树在每条根上都要算一遍），
/// 按位置条目记一份，省掉重复的整单元 CRC——层 0 每个崩溃状态都要跑这一遍。
type IndexNodeCache = BTreeMap<(u32, u64, u32), Option<crate::IndexNodeView>>;

fn read_index_node_without_judging(
    reader: &dyn ImageReader,
    pointer: &PointerView,
    cache: &mut IndexNodeCache,
) -> Option<crate::IndexNodeView> {
    let key = (
        pointer.locations[0].device,
        pointer.locations[0].slot,
        pointer.locations[0].checksum,
    );
    if let Some(cached) = cache.get(&key) {
        return cached.clone();
    }
    let parsed = read_unit_without_judging(reader, pointer, node_bytes())
        .and_then(|unit| index_node_view(&unit).ok());
    cache.insert(key, parsed.clone());
    parsed
}

/// 这一遍要走多深。I-9.14（树表条目的诞生 txg 跨根不变） 只要每条根的树表；I-3.9（释放代落在停止引用它的那一格区间里）
/// 要这条根引用的全部落点，而那要把每棵树都走到叶——最新根那棵账里一条已释放记录都没有时，别的根就不必走那么深
/// （层 0 上多数崩溃状态是这一类：最新的根还是暖机那几代，树表是空的）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReferenceScanDepth {
    TreeTableOnly,
    EveryReferencedPlacement,
}

/// 从一条根走一遍，取它的树表条目；`depth` 是「走到叶」时再取它引用的全部落点与分配记录。只读不判。
fn references_of_root(
    reader: &dyn ImageReader,
    record: &[u8],
    depth: ReferenceScanDepth,
    cache: &mut IndexNodeCache,
) -> RootReferences {
    let mut references = RootReferences {
        placements: BTreeSet::new(),
        tree_table_birth_txg: BTreeMap::new(),
        tree_table_placement: None,
        tree_table_reading: TreeTableReading::PointerAllZero,
        allocation_records: Vec::new(),
        depth,
        is_complete: true,
    };
    let instance_table = parse_node_pointer(&record[170..256]);
    references.note(&instance_table);
    // 实例表链上第 1 片起的落点（D18（块里携带什么信息） 已定项 11 的链）也是这条根引用的：写行那次发布整条链重写、逐片释放旧链，
    // 不数它们，旧链第 1 片起那几条已释放记录在 I-3.9 这一遍里找不到引用过它的根、被当成「见证它释放的根已不在候选集里」跳过。
    if depth == ReferenceScanDepth::EveryReferencedPlacement {
        note_instance_table_pages_after_the_first(reader, &instance_table, &mut references);
    }
    // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。映射条目指的单元与树里引用的是同一批，这一遍不另数
    // （主走读把它们记进 `references`，与树里指向同一单元的那条引用同 (设备, 起点槽, 跨度)，记成一个）。
    let mapping_root = parse_node_pointer(&record[256..342]);
    references.note(&mapping_root);
    // 树表 0 条那一版的分配记录树的根（C512（树表 0 条的一版上被换下的单元记在哪））：它的落点同样只有根记录指着，
    // 不数它，被抛弃根的影子账与 I-3.9 的引用集合就少一片。
    let allocation_record_tree_root = parse_node_pointer(&record[342..428]);
    references.note(&allocation_record_tree_root);
    // 那棵树按位置寻址、可以多层（D8（核心索引结构） 已定项 14）：根之下的节点只有父节点指着，走到叶那个深度上逐个数进来。
    // 它的记录就是这一版的账（树表 0 条的一版上被换下的单元记在这里，C512）：I-3.9 取最新根的账时要读到它们（代码审阅 6c 第 9 条）。
    if depth == ReferenceScanDepth::EveryReferencedPlacement
        && !allocation_record_tree_root.all_zero
    {
        match allocation_record_tree_without_judging(reader, &allocation_record_tree_root, cache) {
            Some((node_pointers, records)) => {
                for pointer in &node_pointers {
                    references.note(pointer);
                }
                references.allocation_records.extend(records);
            }
            None => references.is_complete = false,
        }
    }
    let tree_table_pointer = parse_node_pointer(&record[36..122]);
    references.note(&tree_table_pointer);
    if tree_table_pointer.all_zero {
        references.is_complete = false;
        return references;
    }
    references.tree_table_placement = Some((
        tree_table_pointer.locations[0].device,
        tree_table_pointer.locations[0].slot,
    ));
    let Some(tree_table) = read_index_node_without_judging(reader, &tree_table_pointer, cache)
    else {
        references.is_complete = false;
        references.tree_table_reading = TreeTableReading::Unreadable;
        return references;
    };
    references.tree_table_reading = TreeTableReading::EveryEntryRead;
    for entry in &tree_table.entries {
        // 条目宽是节点头里的一个字段：头校验和过、载荷 CRC 过而条目宽被改窄的镜像也要能判（那由 I-1.7 / I-1.1 说话），
        // 这一遍只如实记「数不全」，不按短条目去解字段。
        if entry.len() < tree_table_entry_bytes() {
            references.is_complete = false;
            references.tree_table_reading = TreeTableReading::SomeEntryShorterThanTheFieldTable;
            continue;
        }
        let tree = read_u64(entry, 0);
        references
            .tree_table_birth_txg
            .insert(tree, read_u64(entry, TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET));
        let tree_root = parse_node_pointer(&entry[14..100]);
        if tree_root.all_zero {
            // day-1 注册、还没有根节点的树（livelist 6、稀疏旁表 7、deadlist 8）：不占落点。
            continue;
        }
        references.note(&tree_root);
        if depth == ReferenceScanDepth::TreeTableOnly {
            continue;
        }
        let Some(node) = read_index_node_without_judging(reader, &tree_root, cache) else {
            references.is_complete = false;
            continue;
        };
        collect_tree_references(
            reader,
            cache,
            &mut references,
            &tree_root,
            &node,
            read_u16(entry, 10),
        );
    }
    // 中央映射树根之下的节点（多层时，D8（核心索引结构） 已定项 11）：只有它们的父节点指着，不数它们，这条根的引用集合就少几片。
    // 只在「走到叶」那个深度上数（引用集合只在那个深度上作数）。
    if depth == ReferenceScanDepth::EveryReferencedPlacement && !mapping_root.all_zero {
        match read_index_node_without_judging(reader, &mapping_root, cache) {
            Some(mapping_root_node) => note_every_node_below(
                reader,
                cache,
                &mut references,
                &mapping_root_node,
                KEY_SCHEMA_MAPPING,
            ),
            None => references.is_complete = false,
        }
    }
    references
}

/// 沿实例表链（根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片，D18（块里携带什么信息） 已定项 11）把第 1 片起每一片的
/// 指针记进这条根的引用集合。只读不判：一片读不出、解不开、身份不是 (0, 4, 片序号, 0)、I-3.14 不成立（链指针记录之前有不是行的记录、
/// 行的 flags 不是 0、最后一条不是合法的链指针记录、它说有下一片而位置指针头部有一段不是第一版的取值），都只如实记成数不全
/// （判定归主走读：I-2.1、I-1.1、I-3.14）。第五形那条指针不记进引用集合，与主走读不跟随它同一个读法（[`chain_record_view`]）。
/// 迭代上界：第 k 片要求容器号是 k，每一轮读的是盘上一个不同的单元；跨轮带的只有下一片的指针；提前出口是走到最后一片与上面那几样读不下去。
fn note_instance_table_pages_after_the_first(
    reader: &dyn ImageReader,
    first_page_pointer: &PointerView,
    references: &mut RootReferences,
) {
    let mut pointer = *first_page_pointer;
    let mut page_index: u64 = 0;
    loop {
        let Some(page) = read_unit_without_judging(reader, &pointer, data_unit_bytes())
            .and_then(|unit| packed_unit_view(&unit).ok())
        else {
            references.is_complete = false;
            return;
        };
        let identity = (
            page.birth_tree,
            page.record_type,
            page.container,
            page.container_birth,
        );
        if identity != (0, PACKED_TYPE_INSTANCE_TABLE, page_index, 0) {
            references.is_complete = false;
            return;
        }
        // I-3.14 不成立的一片解不开（D18（块里携带什么信息） 已定项 11，用户 2026-10-06 补句）：链指针记录之前有不是行的记录
        // （kind 不是 0 或 1、片中间有链指针记录）或行的 flags 不是 0，与主走读、只读不判的读法同一个读法，数不全。
        let records_before_the_last = page
            .records
            .split_last()
            .map_or(&[][..], |(_, before)| before);
        if records_before_the_last.iter().any(|record| {
            record.len() != INSTANCE_TABLE_RECORD_BYTES
                || InstanceTableRecordKind::of_record(record) != InstanceTableRecordKind::Row
                || record[INSTANCE_TABLE_ROW_FLAGS_OFFSET]
                    != INSTANCE_TABLE_ROW_FLAGS_OF_THE_FIRST_VERSION
        }) {
            references.is_complete = false;
            return;
        }
        match page
            .records
            .last()
            .map_or(ChainRecordView::Malformed, |last| chain_record_view(last))
        {
            ChainRecordView::LastPage => return,
            ChainRecordView::NextPage(next_page) => {
                references.note(&next_page);
                pointer = next_page;
                page_index += 1;
            }
            ChainRecordView::Malformed => {
                references.is_complete = false;
                return;
            }
        }
    }
}

/// 多层码 2 树（记账树、中央映射树）一个节点下面的全部节点都记进引用集合（它们各占一个槽）：按内部条目里的子指针逐层往下，
/// 条目宽窄于 key 宽 + 86、孩子读不出、孩子的层级不是父层级减一，都如实记成数不全、不往下走（那几样由主走读的 I-1.1 / I-1.10 说话）。
/// 递归的上界：每下一层层级减一。
fn note_every_node_below(
    reader: &dyn ImageReader,
    cache: &mut IndexNodeCache,
    references: &mut RootReferences,
    node: &crate::IndexNodeView,
    schema: crate::KeySchema,
) {
    if node.level == 0 {
        return;
    }
    let key_width = schema.width();
    if node.key_width != key_width || node.entry_width < internal_entry_bytes(schema) {
        references.is_complete = false;
        return;
    }
    for entry in &node.entries {
        let child_pointer = parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]);
        references.note(&child_pointer);
        let Some(child) = read_index_node_without_judging(reader, &child_pointer, cache) else {
            references.is_complete = false;
            continue;
        };
        // 孩子的层级是盘上读来的一字节：拿父层级减一去比（父层级这里 > 0），不拿孩子的加一——255 加一在 u8 上溢出（代码审阅 6c 第 6 条）。
        if node.level.checked_sub(1) != Some(child.level) {
            references.is_complete = false;
            continue;
        }
        note_every_node_below(reader, cache, references, &child, schema);
    }
}

/// inode 树一个内部节点下面的孩子都记进引用集合：类型段 2 的叶容器记它的落点；类型段 0 的内部节点（I-9.2（条目身份与子头相符））
/// 记它的落点再往下数它的孩子。读不出、层级不是父层级减一、层级 0、条目窄于字段表、类型段不认得，都如实记成数不全（判定归主走读）。
/// 递归的上界：每下一层层级减一，层级 0 不往下走。
fn note_every_inode_child_below(
    reader: &dyn ImageReader,
    cache: &mut IndexNodeCache,
    references: &mut RootReferences,
    node: &crate::IndexNodeView,
) {
    for entry in &node.entries {
        let child_pointer = parse_node_pointer(&entry[34..120]);
        references.note(&child_pointer);
        match InodeInternalEntryType::of(read_u16(entry, 16)) {
            InodeInternalEntryType::LeafContainer => {}
            InodeInternalEntryType::InternalNode => {
                let Some(child) = read_index_node_without_judging(reader, &child_pointer, cache)
                else {
                    references.is_complete = false;
                    continue;
                };
                if node.level.checked_sub(1) != Some(child.level)
                    || child.level == 0
                    || child.entry_width < inode_internal_entry_bytes()
                {
                    references.is_complete = false;
                    continue;
                }
                note_every_inode_child_below(reader, cache, references, &child);
            }
            InodeInternalEntryType::Unregistered(_) => references.is_complete = false,
        }
    }
}

/// 一棵分配记录树整棵读、不判（按绝对槽号按位置寻址，D8（核心索引结构） 已定项 14）：交回每个节点的指针（根在内）与全部记录。
/// 节点读不出、层级不是它的位置规定的那一层、内部条目指不到一个合法的孩子、条目窄于字段表：交回 `None`（数不全；判定归主走读的
/// I-1.1 / I-1.10 / I-2.1）。迭代上界：每个节点至多进一次（按第一条位置条目记过的不再读），每下一层层级减一。
fn allocation_record_tree_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    cache: &mut IndexNodeCache,
) -> Option<(Vec<PointerView>, Vec<AllocationRecordView>)> {
    let pool_devices = reader.devices();
    let device_slots: Vec<u64> = pool_devices
        .iter()
        .map(|device| reader.device_bytes(*device).unwrap_or(0) / SLOT_BYTES)
        .collect();
    let root_level = allocation_record_tree_root_level(&device_slots)?;
    let internal_entry_bytes =
        usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96");
    let key_width = KEY_SCHEMA_ALLOCATION.width();
    let mut node_pointers = Vec::new();
    let mut records = Vec::new();
    let mut seen: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut pending = vec![(*root_pointer, AllocationRecordTreeCell::Root, root_level)];
    while let Some((pointer, cell, level)) = pending.pop() {
        if pointer.all_zero
            || !seen.insert((pointer.locations[0].device, pointer.locations[0].slot))
        {
            return None;
        }
        node_pointers.push(pointer);
        let node = read_index_node_without_judging(reader, &pointer, cache)?;
        if node.level != level || node.key_width != key_width {
            return None;
        }
        if level == 0 {
            if node.entry_width < allocation_record_bytes() {
                return None;
            }
            records.extend(
                node.entries
                    .iter()
                    .map(|entry| parse_allocation_record(entry)),
            );
            continue;
        }
        if node.entry_width < internal_entry_bytes {
            return None;
        }
        for entry in &node.entries {
            let child = allocation_record_tree_child_of_entry_key(
                cell,
                level,
                &entry[..key_width],
                &pool_devices,
            )?;
            pending.push((
                parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]),
                child,
                level - 1,
            ));
        }
    }
    Some((node_pointers, records))
}

/// 一棵 extent 树整棵读、不判（上段与每个文件的下段，D8（核心索引结构） 已定项 14）：交回每个节点的指针（根在内）与全部数据指针
/// （下段叶的 extent 叶记录与上段叶条目里内联的那一个）。读不下去的同 [`allocation_record_tree_without_judging`] 交回 `None`。
fn extent_tree_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    cache: &mut IndexNodeCache,
) -> Option<(Vec<PointerView>, Vec<PointerView>)> {
    extent_tree_with_the_inode_of_each_data_pointer_without_judging(reader, root_pointer, cache)
        .map(|extent_tree| {
            (
                extent_tree.node_pointers,
                extent_tree
                    .data_pointers_of_inodes
                    .into_iter()
                    .map(|(_, data_pointer)| data_pointer)
                    .collect(),
            )
        })
}

/// [`extent_tree_with_the_inode_of_each_data_pointer_without_judging`] 读出的一棵 extent 树：每个节点的指针（根在内），
/// 与每个数据指针连同它属于哪个 inode 号。
struct ExtentTreeReadWithoutJudging {
    node_pointers: Vec<PointerView>,
    data_pointers_of_inodes: Vec<(u64, PointerView)>,
}

/// [`extent_tree_without_judging`] 那一遍，每个数据指针带上它属于哪个 inode 号（上段叶条目的 inode 号；下段叶记录属于引它那条上段条目的号）：
/// 删除态扫描按号数数据单元（I-4.7（删除中 inode 有账）、I-4.6（中间态其余不变量成立））。
fn extent_tree_with_the_inode_of_each_data_pointer_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    cache: &mut IndexNodeCache,
) -> Option<ExtentTreeReadWithoutJudging> {
    /// 待读的一个节点：上段 (层级, 序号)，或某个 inode 下段 (层级, 序号)；根那一个层级取它自己头里的。
    enum Pending {
        Upper(PointerView, Option<(u8, u64)>),
        Lower(PointerView, u64, Option<(u8, u64)>),
    }
    let internal_entry_bytes = usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110");
    let upper_leaf_entry_bytes = usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113");
    let key_width = KEY_SCHEMA_EXTENT.width();
    let mut node_pointers = Vec::new();
    let mut data_pointers = Vec::new();
    let mut seen: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut pending = vec![Pending::Upper(*root_pointer, None)];
    while let Some(next) = pending.pop() {
        let (pointer, lower_of_inode, position) = match next {
            Pending::Upper(pointer, position) => (pointer, None, position),
            Pending::Lower(pointer, inode, position) => (pointer, Some(inode), position),
        };
        if pointer.all_zero
            || !seen.insert((pointer.locations[0].device, pointer.locations[0].slot))
        {
            return None;
        }
        node_pointers.push(pointer);
        let node = read_index_node_without_judging(reader, &pointer, cache)?;
        let (level, index) = position.unwrap_or((node.level, 0));
        if node.level != level || node.key_width != key_width {
            return None;
        }
        match (lower_of_inode, level) {
            (None, 0) => {
                if node.entry_width < upper_leaf_entry_bytes {
                    return None;
                }
                for entry in &node.entries {
                    match extent_upper_leaf_entry_view(entry) {
                        ExtentUpperLeafEntryView::NoDataUnit { .. } => {}
                        ExtentUpperLeafEntryView::LowerSegmentRoot {
                            inode,
                            pointer: lower_root_pointer,
                        } => {
                            pending.push(Pending::Lower(
                                parse_node_pointer(&lower_root_pointer),
                                inode,
                                None,
                            ));
                        }
                        ExtentUpperLeafEntryView::InlineDataUnit {
                            inode,
                            pointer: inline_data_pointer,
                        } => {
                            data_pointers.push((inode, parse_data_pointer(&inline_data_pointer)));
                        }
                        ExtentUpperLeafEntryView::Malformed => return None,
                    }
                }
            }
            (Some(inode), 0) => {
                if node.entry_width < extent_leaf_record_bytes() {
                    return None;
                }
                data_pointers.extend(
                    node.entries
                        .iter()
                        .map(|record| (inode, parse_data_pointer(&record[24..112]))),
                );
            }
            (None, _) => {
                if node.entry_width < internal_entry_bytes {
                    return None;
                }
                for entry in &node.entries {
                    let child = extent_upper_child_of_entry_key(level, index, &entry[..key_width])?;
                    pending.push(Pending::Upper(
                        parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]),
                        Some(child),
                    ));
                }
            }
            (Some(inode), _) => {
                if node.entry_width < internal_entry_bytes {
                    return None;
                }
                for entry in &node.entries {
                    let child =
                        extent_lower_child_of_entry_key(inode, level, index, &entry[..key_width])?;
                    pending.push(Pending::Lower(
                        parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]),
                        inode,
                        Some(child),
                    ));
                }
            }
        }
    }
    Some(ExtentTreeReadWithoutJudging {
        node_pointers,
        data_pointers_of_inodes: data_pointers,
    })
}

/// 一棵树的根节点下面还引用了什么：extent 树与分配记录树整棵按位置读（每个节点、数据指针、分配记录）、
/// 「inode 树内部条目的子指针」与记账树根之下的全部节点；条目格式没有条款的树走不下去，如实记成数不全。
fn collect_tree_references(
    reader: &dyn ImageReader,
    cache: &mut IndexNodeCache,
    references: &mut RootReferences,
    tree_root: &PointerView,
    node: &crate::IndexNodeView,
    tree_kind_code: u16,
) {
    // 条目宽窄于字段表的（节点头里那个字段被改过）不按短条目去解，如实记成数不全：宽度本身由 I-1.7 / I-1.1 说话。
    match TreeKindForReferenceScan::of(tree_kind_code) {
        TreeKindForReferenceScan::Extent => {
            match extent_tree_without_judging(reader, tree_root, cache) {
                Some((node_pointers, data_pointers)) => {
                    for pointer in node_pointers.iter().chain(&data_pointers) {
                        references.note(pointer);
                    }
                }
                None => references.is_complete = false,
            }
        }
        TreeKindForReferenceScan::Inode => {
            if node.level > 0 && node.entry_width >= inode_internal_entry_bytes() {
                note_every_inode_child_below(reader, cache, references, node);
            } else {
                references.is_complete = false;
            }
        }
        TreeKindForReferenceScan::Allocation => {
            match allocation_record_tree_without_judging(reader, tree_root, cache) {
                Some((node_pointers, records)) => {
                    for pointer in &node_pointers {
                        references.note(pointer);
                    }
                    references.allocation_records.extend(records);
                }
                None => references.is_complete = false,
            }
        }
        TreeKindForReferenceScan::Accounting => {
            // 记账行不引用别的单元；多层时根之下的节点各占一个槽（D8（核心索引结构） 已定项 11：内部条目 = key 22 + 子指针 86）。
            note_every_node_below(reader, cache, references, node, KEY_SCHEMA_ACCOUNTING);
        }
        // `deleted_inodes` 根的条目只是号，不引用别的单元（根自己由调用方记）。
        TreeKindForReferenceScan::DeletedInodes => {}
        TreeKindForReferenceScan::ContainerTree => {
            if node.level == 1 && node.entry_width >= inode_internal_entry_bytes() {
                for entry in &node.entries {
                    references.note(&parse_node_pointer(&entry[34..120]));
                }
            } else {
                // 根长高之后（层级 > 1）的引用集合这一遍还不往下数，如实记成数不全。
                references.is_complete = false;
            }
        }
        TreeKindForReferenceScan::WithoutWalkableEntryFormat(_) => references.is_complete = false,
    }
}

/// 走读到的一版里 I-5.5 要的三样：称呼（判红时写进第一处违例）、`checkpoint_txg` 与它的树表指针
/// （根环里的根取根记录偏移 36 起的 86 字节，由记录施加出来的那一版取记录新根段里的那一条）。
struct VersionToJudgeTheFirstDataUnitOf<'record> {
    name: String,
    checkpoint_txg: u64,
    tree_table_pointer: &'record [u8],
}

/// I-5.5（从 mkfs 水位建树那一版的 t1 落点）：走读到的每一版（回退候选集里每条根，含由记录施加出来的那几版）各自重读一遍树表，
/// ① 树表非空，且种类 1（extent 树）条目的树 ID 等于 mkfs 种下的水位 65536；② 树表里每条条目的诞生 txg（条目偏移 108）都等于这一版的
/// `checkpoint_txg`——两样都成立就是从 mkfs 水位第一次建树的那一版：它的 extent 树引用的数据单元里，每块盘上落点槽号最小的那一个
/// 要等于这一版的分配记录在那块盘上算出的落点（[`first_data_unit_slot_the_allocation_records_leave_on_a_device`]：从单元区起始槽号 U 起，
/// 起点最小、32768 对齐、两槽都空、不在开放段里的那一对；U 是所选系统配置偏移 417）。不写死 U + 4：mkfs 的实例表（U、U + 1）被 COW 走、
/// 过了回收门槛之后，实现照 D3（空间分配） 已定项 8 第 1 条把数据放在 U（用户 2026-10-04 定 I-5.5 按分配记录现算，判决
/// `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K2）。这一版没有数据单元时不判，不满足 ①② 的版本不判。
/// 只读、不判别的不变量：主走读按 `visited_units` 只走一遍共享子树，共享的 extent 树只算进第一条走到它的版本，这一条要的是
/// 「这一版自己引用了哪些数据单元」（与 [`references_of_root`] 同一个缘故）。
///
/// 下面几格这一版判不了，报部分没判（[`InvariantVerdict::PartiallyJudged`]，判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`
/// 第二节 Y3）：没有违例时整条报部分没判，逐版列出没判的那一版（实例、txg）与为什么，形态同 I-9.28 树表没读全的那几条根：
/// 树表读不出、条目宽不到字段表的 200 字节（认不出这一版是不是这一类）；认出来的那一版 extent 树读不全（数据单元数不全）、
/// 分配记录树读不全（落点算不出）；种类 1 条目不止一条（① 读哪一条）、种类 3（分配记录树）条目不是恰好一条（落点按哪一棵算）——
/// 这两格的树表本身由 I-9.16 ③ ④ 判红，这里只记没判。
/// 这一版有数据单元、而池里某块盘上一个都没有：那块盘上没有「最小的那一个」，该盘跳过、不记（多盘布局下合法；GPU 判器本来就跳过）。
fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
    reader: &dyn ImageReader,
    versions: &[VersionToJudgeTheFirstDataUnitOf<'_>],
    unit_area_start_slot: u64,
    judgements: &mut Judgements,
) {
    let invariant = FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK;
    let devices = reader.devices();
    let mut cache = IndexNodeCache::new();
    // 没判的那几版：「（版本称呼，含实例与 txg）：为什么」，逐版一条，走完一起报（`Judgements::partially_judged` 只记第一次给的说明）。
    let mut versions_not_judged: Vec<String> = Vec::new();
    // 认出来的从 mkfs 水位第一次建树的那几版（不论有没有数据单元）：一版都没认出来时报专门的不适用理由。
    let mut versions_built_from_the_mkfs_watermark = 0u64;
    for version in versions {
        let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
        if tree_table_pointer.all_zero {
            // 用户 2026-10-05 定甲：core 照全零的树表指针读盘 0 槽 0、读不出，与树表读不出同一处置（此前当这一版没有树表、不判也不记）。
            versions_not_judged.push(format!(
                "{}：树表指针全零（core 照它读盘 0 槽 0、读不出），认不出它是不是从 mkfs 水位第一次建树的那一版",
                version.name
            ));
            continue;
        }
        let Some(tree_table) =
            read_index_node_without_judging(reader, &tree_table_pointer, &mut cache)
        else {
            versions_not_judged.push(format!(
                "{}：树表读不出，认不出它是不是从 mkfs 水位第一次建树的那一版",
                version.name
            ));
            continue;
        };
        if tree_table.entries.is_empty() {
            continue;
        }
        if tree_table
            .entries
            .iter()
            .any(|entry| entry.len() < tree_table_entry_bytes())
        {
            versions_not_judged.push(format!(
                "{}：树表条目宽不到字段表的 200 字节，认不出它是不是从 mkfs 水位第一次建树的那一版",
                version.name
            ));
            continue;
        }
        let extent_entries: Vec<&Vec<u8>> = tree_table
            .entries
            .iter()
            .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_EXTENT)
            .collect();
        let extent_entry = match extent_entries.as_slice() {
            [] => continue,
            [only_extent_entry] => *only_extent_entry,
            [_, _, ..] => {
                versions_not_judged.push(format!(
                    "{}：树表里种类 1（extent 树）条目有 {} 条，认法 ① 读不出哪一条（由 I-9.16 ③ ④ 判）",
                    version.name,
                    extent_entries.len()
                ));
                continue;
            }
        };
        let is_built_from_the_mkfs_watermark = read_u64(extent_entry, 0)
            == TREE_IDENTIFIER_WATERMARK_AT_MKFS
            && tree_table.entries.iter().all(|entry| {
                read_u64(entry, TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET) == version.checkpoint_txg
            });
        if !is_built_from_the_mkfs_watermark {
            continue;
        }
        versions_built_from_the_mkfs_watermark += 1;
        let extent_root = parse_node_pointer(&extent_entry[14..100]);
        if extent_root.all_zero {
            continue;
        }
        let Some((_, data_pointers)) =
            extent_tree_without_judging(reader, &extent_root, &mut cache)
        else {
            versions_not_judged.push(format!(
                "{}（从 mkfs 水位第一次建树的那一版）：extent 树读不全，它引用的数据单元数不全",
                version.name
            ));
            continue;
        };
        let mut smallest_data_unit_slot_on_each_device: BTreeMap<u32, u64> = BTreeMap::new();
        for location in data_pointers
            .iter()
            .filter(|pointer| !pointer.all_zero)
            .flat_map(|pointer| pointer.locations)
        {
            smallest_data_unit_slot_on_each_device
                .entry(location.device)
                .and_modify(|smallest_slot| *smallest_slot = (*smallest_slot).min(location.slot))
                .or_insert(location.slot);
        }
        if smallest_data_unit_slot_on_each_device.is_empty() {
            continue;
        }
        let allocation_entries: Vec<&Vec<u8>> = tree_table
            .entries
            .iter()
            .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_ALLOCATION)
            .collect();
        let [allocation_entry] = allocation_entries.as_slice() else {
            versions_not_judged.push(format!(
                "{}（从 mkfs 水位第一次建树的那一版）：树表里种类 3（分配记录树）条目有 {} 条，落点按哪一棵算不出（由 I-9.16 ③ ④ 判）",
                version.name,
                allocation_entries.len()
            ));
            continue;
        };
        let Some((_, allocation_records)) = allocation_record_tree_without_judging(
            reader,
            &parse_node_pointer(&allocation_entry[14..100]),
            &mut cache,
        ) else {
            versions_not_judged.push(format!(
                "{}（从 mkfs 水位第一次建树的那一版）：分配记录树读不全，落点算不出",
                version.name
            ));
            continue;
        };
        for device in &devices {
            // 这一版在这块盘上一个数据单元都没有：没有「最小的那一个」，该盘跳过、不记。
            if let Some(&smallest_slot) = smallest_data_unit_slot_on_each_device.get(device) {
                let expected_slot = first_data_unit_slot_the_allocation_records_leave_on_a_device(
                    &allocation_records,
                    *device,
                    version.checkpoint_txg,
                    unit_area_start_slot,
                    open_cluster_segment_named_by_the_tree_table_on_a_device(
                        &tree_table_pointer,
                        *device,
                    ),
                );
                judgements.judge(invariant, smallest_slot == expected_slot, || {
                    format!(
                        "{}（从 mkfs 水位第一次建树的那一版）：盘 {device} 上落点最小的数据单元在槽 {smallest_slot}，不是这一版的分配记录从单元区起始槽号 {unit_area_start_slot} 起算出的落点 {expected_slot}",
                        version.name
                    )
                });
            }
        }
    }
    if !versions_not_judged.is_empty() {
        judgements.partially_judged(
            invariant,
            format!(
                "判到的那几版落点都对得上；这几版没判：{}",
                versions_not_judged.join("；")
            ),
        );
    }
    // 一版都没认出来时的专门理由（判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P3「四支豁免」那一行：
    // 此前落到 `Judgements::into_report` 的缺省理由，与「这个函数一次都没跑」分不开）。认出来了而一版都没判到（认出来的那几版在池里的盘上都没有数据单元）
    // 另写一句（用户 2026-10-06 定 Q4「甲 另写一句」，判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第五节第 4 条）。
    // `not_applicable` 不改已经评估过的：认出来的版本里有一版判到了，这一句不出现。
    if versions_built_from_the_mkfs_watermark == 0 {
        judgements.not_applicable(invariant, NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK);
    } else {
        judgements.not_applicable(
            invariant,
            VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT,
        );
    }
}

/// I-5.5 一次都没判、因为走读到的版本里没有一版满足认法 ①② 时的不适用理由。
pub const NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK: &str = "回退候选集里每条根与由记录施加出来的那几版，没有一版是从 mkfs 水位第一次建树的那一版（树表非空、种类 1 条目的树 ID 是 65536、每条条目的诞生 txg 等于这一版的 txg）：一版都不判";

/// I-5.5 一次都没判、因为认出来的从 mkfs 水位第一次建树的那几版在池里的盘上都没有数据单元时的不适用理由（那一行「V 没有数据单元时不判」）：
/// extent 树根全零、一条非零的数据指针都没有，或数据指针的落点全在池里没有的盘上（判法逐盘只看池里有的盘），三样都走到这一句
/// （判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P4、第三节第 1 条 R3：此前只说「没有数据单元」，后一样实际有）。
pub const VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT: &str = "认出了从 mkfs 水位第一次建树的那一版，但认出来的那几版在池里的盘上都没有数据单元（extent 树根全零、一条非零的数据指针都没有，或数据指针的落点全在池里没有的盘上）：一版都不判";

/// 一块盘上的开放段：起点是 64 的整数倍的 [`CLUSTER_SEGMENT_SLOTS`] 个槽（D3（空间分配） 已定项 10 ①）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OpenClusterSegment {
    first_slot: u64,
}

impl OpenClusterSegment {
    fn holds_slot(self, slot: u64) -> bool {
        self.first_slot <= slot && slot < self.first_slot + CLUSTER_SEGMENT_SLOTS
    }
}

/// I-5.5 拿哪一段当这一版在这块盘上的开放段：这一版树表单元在这块盘上那一份所在的段（树表单元最末从开放段 bump，
/// D3（空间分配） 已定项 10 ⑤）。树表指针在这块盘上没有位置条目时交回 `None`（这块盘上不排除哪一段）。
fn open_cluster_segment_named_by_the_tree_table_on_a_device(
    tree_table_pointer: &PointerView,
    device: u32,
) -> Option<OpenClusterSegment> {
    tree_table_pointer
        .locations
        .iter()
        .find(|location| location.device == device)
        .map(|location| OpenClusterSegment {
            first_slot: location.slot - location.slot % CLUSTER_SEGMENT_SLOTS,
        })
}

/// I-5.5 的判据（用户 2026-10-04 定：按这一版的分配记录现算，D3（空间分配） 已定项 8 第 1 条、已定项 10 ② ③）：
/// 从单元区起始槽号 U 起，这一版在 `device` 上起点最小、起点 32768 对齐（槽号是 [`DATA_UNIT_SLOTS`] 的整数倍；U 是 64 的整数倍，
/// 读系统配置时判过）、两槽都空、不在开放段里的那一对槽的起点。「空」= 没有被「已释放、或分配代早于这一版」的分配记录罩着。
/// 不看盘尾：比每一个被占的槽都靠后的那一对总是空的，交回的落点越过盘尾时与任何真落点都不等，照样判红。
/// 迭代上界：跳过开放段至多一次（候选只增不减），其余每一轮跳过的那一对里至少有一个被占的槽，合计不超过被占槽数 + 1 轮。
///
/// 下面几样 checker 在这一版的盘面上复算不出，只能近似（攻方报告 `research/prompts/m3-formal-code-b17-r1-opus-output.md`
/// 「K2-1、K2-2 打中之后的四句」第 2 句与「改法表」），都照实写进 `.claude/kb/invariants.md` I-5.5 那一行：
/// - 回收门槛：已释放的记录一律当占着。实现按「释放代 ≤ max(F_生效, 环里最旧有效根)」回收（D3（空间分配） 已定项 7 清扫准入），
///   这一版根里的 F、txg 与根环槽数盘上有，被抛弃的根与挂载那一刻读不出的槽盘上没有，门槛复算不出；记录在被重新分配之前一直写着已释放。
///   ⇒ 写死从 U + 4 起扫的坏实现（攻方 H1）在「U、U + 1 已回收、还没被重新分配」的那几版上判成立，抓不到。
/// - 这一版自己分配的记录（未释放、分配代等于这一版的 txg）当空：它们就是这一版取落点时还空着的槽（数据单元与提交内生块都在里面）。
/// - 开放段盘上没有字段：拿这一版树表单元所在的段当它（[`open_cluster_segment_named_by_the_tree_table_on_a_device`]）；
///   这次挂载更早开过、已经用尽的段认不出（段用尽才换段，用尽的段里剩不下起点对齐的空槽对）。
/// - 分配准入合取（D3（空间分配） 已定项 7）里隔离位（影子账）、抬 F 扣住位、保护期内的 R 盘上都没有：没有分配记录罩着的槽一律当空。
fn first_data_unit_slot_the_allocation_records_leave_on_a_device(
    allocation_records: &[AllocationRecordView],
    device: u32,
    version_checkpoint_txg: u64,
    unit_area_start_slot: u64,
    open_cluster_segment: Option<OpenClusterSegment>,
) -> u64 {
    let occupied_slots: BTreeSet<u64> = allocation_records
        .iter()
        .filter(|record| {
            record.device == device
                && (record.is_released || record.generation < version_checkpoint_txg)
        })
        .flat_map(|record| record.slot..record.slot.saturating_add(record.span_slots))
        .collect();
    let mut candidate = unit_area_start_slot;
    loop {
        if let Some(segment) = open_cluster_segment.filter(|segment| segment.holds_slot(candidate))
        {
            candidate = segment.first_slot + CLUSTER_SEGMENT_SLOTS;
            continue;
        }
        if (candidate..candidate + DATA_UNIT_SLOTS).all(|slot| !occupied_slots.contains(&slot)) {
            return candidate;
        }
        candidate += DATA_UNIT_SLOTS;
    }
}

/// 候选集里的一条根：它在 `roots` 里的下标、它的 checkpoint_txg 与这一遍取到的引用集合。
struct ScannedCandidateRoot {
    root_index: usize,
    checkpoint_txg: u64,
    references: RootReferences,
}

/// 一棵树的树表条目在某一条有效根那里的样子（I-9.14 按树 ID 归并这些）。
struct TreeTableEntrySighting {
    root_checkpoint_txg: u64,
    /// 这条根的树表单元落在哪 (设备身份, 起点槽)：两条根共用一个树表单元时，它们的条目不是两处独立的证据。
    tree_table_placement: (u32, u64),
    birth_txg: u64,
}

/// I-3.9（释放代落在停止引用它的那一格区间里）：对最新根那棵账里每条带已释放标志的分配记录，在候选集里取还引用这个落点的
/// 最新一条有效根（L）与最早不再引用它的有效根（T，L 之后的第一条），释放代必须落在 `(L, T]` 里；每条有效根都还引用它的
/// （L 之后没有根），它不许带已释放标志。候选集里一条根都没引用过它的（已经回收、还没被复用：见证它释放的根已不在候选集里）
/// 跳过不判，全是这一类时整条报不适用——三处都照 `.claude/kb/invariants.md` I-3.9 那一行。
///
/// 为什么是区间：根环里的 txg 可以有洞——一次发布的根槽从没写过、而它的 journal 记录在挂载时被施加（D23（journal 的角色与格式）：
/// 记录自带新根段），那次发布做的释放就写着一个环里没有根的 txg。`crash_enumeration_fixed_script_stream.rs` 的残留记录那条流
/// 正是这个形状（txg 5 的根槽从没写过、jsn 5 被施加、释放代 5、环里最早不再引用那个落点的根是 txg 6），按「等于 T」判，
/// 那条流上 12 个合法状态判红。L 与 T 之间没有洞时 `(L, T]` 只有 T 一个值，就是「等于最早不再引用它的那条根的 txg」；
/// C374（释放代与树表诞生 txg 只有验收断言盯着） 点名的判别力（发布 B 那次的释放代从 4 写成 3）照样红。
fn judge_release_generations(
    scanned: &[ScannedCandidateRoot],
    newest_position: usize,
    judgements: &mut Judgements,
) {
    let released: Vec<&AllocationRecordView> = scanned[newest_position]
        .references
        .allocation_records
        .iter()
        .filter(|record| record.is_released)
        .collect();
    if released.is_empty() {
        judgements.not_applicable("I-3.9", "最新的根下面没有带已释放标志的分配记录");
        return;
    }
    for root in scanned {
        assert_eq!(
            root.references.depth,
            ReferenceScanDepth::EveryReferencedPlacement,
            "最新根那棵账里有已释放记录时，每条候选根都按「走到叶」扫过（`judge_release_generation_and_tree_table_birth` 定的深度）"
        );
    }
    // 守卫按记录收窄（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 10 行）：比 L 晚的候选根有一条走不完，
    // 这个落点可能藏在它读不出的节点下面（L 算小、T 算早），这一条跳过；L 及更早的根走不完不碍事——L 取的是还引用它的最大 txg。
    let mut skipped_behind_an_incomplete_later_root = 0u64;
    let mut witnessed_records = 0u64;
    for record in released {
        let placement = (record.device, record.slot);
        let Some(last_referencing_txg) = scanned
            .iter()
            .filter(|root| root.references.placements.contains(&placement))
            .map(|root| root.checkpoint_txg)
            .max()
        else {
            // 候选集里一条根都没引用过它：见证它释放的那条根已经不在候选集里（F_生效 抬过释放代之后落点被回收，
            // 盘上那条记录仍写着已释放，D16（发布语义） 已定项 1 / D3（空间分配） 已定项 7）。这个镜像上找不到那条根，这一条判不了。
            continue;
        };
        let a_later_root_is_incomplete = scanned
            .iter()
            .any(|root| root.checkpoint_txg > last_referencing_txg && !root.references.is_complete);
        if a_later_root_is_incomplete {
            skipped_behind_an_incomplete_later_root += 1;
            continue;
        }
        witnessed_records += 1;
        let first_root_without_it = scanned
            .iter()
            .map(|root| root.checkpoint_txg)
            .filter(|txg| *txg > last_referencing_txg)
            .min();
        let in_the_witnessed_interval = record.generation > last_referencing_txg
            && first_root_without_it.is_some_and(|txg| record.generation <= txg);
        judgements.judge("I-3.9", in_the_witnessed_interval, || {
            match first_root_without_it {
                Some(txg) => format!(
                    "盘 {} 槽 {} 的分配记录释放代 {} 不在 ({last_referencing_txg}, {txg}] 里：还引用它的最新一条有效根是 txg {last_referencing_txg}，最早不再引用它的是 txg {txg}",
                    record.device, record.slot, record.generation
                ),
                None => format!(
                    "盘 {} 槽 {} 的分配记录带已释放标志（释放代 {}），而候选集里每一条有效根都还引用它",
                    record.device, record.slot, record.generation
                ),
            }
        });
    }
    if skipped_behind_an_incomplete_later_root > 0 {
        judgements.part_of_the_range_not_judged(
            "I-3.9",
            "有带已释放标志的记录落在比还引用它的最后一条根晚、又走不完的候选根后面（单元读不出，或走到条目格式没有条款的形状）：那几条的 L 与 T 定不下",
        );
    }
    if witnessed_records == 0 {
        judgements.not_applicable(
            "I-3.9",
            "带已释放标志的记录一条都没有候选根引用过：见证它们释放的根已不在候选集里",
        );
    }
}

/// I-9.14（树表条目的诞生 txg 跨根不变）：按树 ID 归并候选集里每条根的树表条目，诞生 txg 必须相同。
/// 只在这棵树的条目出现在**两个不同的树表单元**里时才判——几条根共用一个树表单元（暖机那几代都指着 mkfs 种的第 0 版）时，
/// 拿同一份字节比自己是恒真的判定，那正是 C374（释放代与树表诞生 txg 只有验收断言盯着） 点的那一类；这时报「不适用」，不报成立。
///
/// ⚠️ **「跨根」的射程只到同一条时间线**（C511（回退到无文件那一版之后诞生代怎么接）：2026-09-23 用户定案收窄）。
/// 收窄不在这个函数里，在**入参**上：`scanned` 是回退候选集——按最新那条根指着的实例表判仍然有效的那一组根
/// （D23（journal 的角色与格式） 已定项 14：(i, T) 有效 ⟺ 实例表里没有实例 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti），
/// 筛在 `check_pool_image` 里 `abandoned` 那一条上。有行 (i, Ti, Wi) 且 T > Ti 的根在被崩溃恢复切掉的那条旧线上，
/// 一进不来，所以这里比的恒是同一条线上的根。
/// **为什么收窄**：同一条线上的根之间才有「这棵树只在第一次出现时设诞生 txg」这层关系可比；被崩溃恢复切掉的旧线上的根不与现行线比。
/// 管理员回退是挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14），不切时间线、树表条目照抄，不走到这一格。
/// 跨线（含被崩溃恢复抛弃的旧线）的诞生 txg 一致归 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）
/// （[`judge_tree_identifier_birth_txg_across_every_readable_root`]）：这一条不再有「只有被切掉的那条线记得不同，不许红」那一格
/// （判别力第 ③ 条与它的正向用例删掉，用户 2026-10-04 定；那份镜像今天红 I-9.28、I-9.14 照旧成立）。
/// 没有被抛弃时间线的那一格由 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite` 盯着。
fn judge_tree_table_birth_txg(scanned: &[ScannedCandidateRoot], judgements: &mut Judgements) {
    let mut by_tree: BTreeMap<u64, Vec<TreeTableEntrySighting>> = BTreeMap::new();
    for root in scanned {
        let Some(tree_table_placement) = root.references.tree_table_placement else {
            continue;
        };
        for (tree, birth_txg) in &root.references.tree_table_birth_txg {
            by_tree
                .entry(*tree)
                .or_default()
                .push(TreeTableEntrySighting {
                    root_checkpoint_txg: root.checkpoint_txg,
                    tree_table_placement,
                    birth_txg: *birth_txg,
                });
        }
    }
    let mut compared_trees = 0u64;
    for (tree, sightings) in &by_tree {
        let distinct_tree_tables: BTreeSet<(u32, u64)> = sightings
            .iter()
            .map(|sighting| sighting.tree_table_placement)
            .collect();
        if distinct_tree_tables.len() < 2 {
            continue;
        }
        compared_trees += 1;
        let first_birth_txg = sightings[0].birth_txg;
        let birth_txg_is_the_same_across_roots = sightings
            .iter()
            .all(|sighting| sighting.birth_txg == first_birth_txg);
        judgements.judge("I-9.14", birth_txg_is_the_same_across_roots, || {
            let seen: Vec<String> = sightings
                .iter()
                .map(|sighting| {
                    format!(
                        "根 txg {} 记 {}",
                        sighting.root_checkpoint_txg, sighting.birth_txg
                    )
                })
                .collect();
            format!("树 {tree} 的树表条目诞生 txg 跨根不一：{}", seen.join("、"))
        });
    }
    if compared_trees == 0 {
        judgements.not_applicable(
            "I-9.14",
            "没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
        );
    }
}

/// 一棵树的树表条目在根环里某一条读得出的根那里的样子（I-9.28 按树 ID 归并这些；不分时间线，所以带上实例代号）。
struct TreeTableEntrySightingInTheRing {
    root_instance: u32,
    root_checkpoint_txg: u64,
    /// 这条根的树表单元落在哪 (设备身份, 起点槽)：两条根共用一个树表单元时，它们的条目不是两处独立的证据。
    tree_table_placement: (u32, u64),
    birth_txg: u64,
}

/// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同，用户 2026-10-04 定新立）：根环里每一条读得出且自证合法的根（`roots`，
/// 不按实例表判有效与否、不按 F 与窗口滤，含被崩溃恢复抛弃的旧线上的根）各读一遍树表，按树 ID（条目偏移 0）归并，诞生 txg 必须相同。
/// 判法 checker 自己写（D13（验证路线） 已定项 5），与 I-9.14 的 [`judge_tree_table_birth_txg`] 分开：那一条只比回退候选集。
///
/// 四种结局（用户 2026-10-04 定「树表读不出算部分没判」、加第四种判定；细则主 agent 定，零轮）：
/// - 有一棵树在两处记的诞生 txg 不一 ⇒ 违例；
/// - 都对得上，而有读得出、自证过的根树表指针非零、树表却读不出或有条目比字段表短 ⇒ 部分没判（[`InvariantVerdict::PartiallyJudged`]），
///   逐条列出这些根（盘号、实例、txg、为什么没判）：它们的条目没进比较，读不出的那一份可能正是对不上的那一份；
/// - 都没有，也没有一棵树的条目出现在**两个不同的树表单元**里 ⇒ 不适用（几条根共用一个树表单元时拿同一份字节比自己是恒真判定，
///   C374（释放代与树表诞生 txg 只有验收断言盯着） 那一类），不报成立；
/// - 其余 ⇒ 成立。
///
/// 树表指针全零的根照 core 的根记录读者读盘 0 槽 0、读不出，与树表读不出一样报部分没判（用户 2026-10-05 定甲，判决
/// `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第五节第 1 条；此前算「这条根没有树表，不算没判到」）。
fn judge_tree_identifier_birth_txg_across_every_readable_root(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut sightings_by_tree: BTreeMap<u64, Vec<TreeTableEntrySightingInTheRing>> =
        BTreeMap::new();
    let mut roots_whose_tree_table_was_not_read_in_full: Vec<String> = Vec::new();
    for (region, _, root) in roots {
        let references = references_of_root(
            reader,
            &root.record_bytes,
            ReferenceScanDepth::TreeTableOnly,
            cache,
        );
        let why_not_judged = match references.tree_table_reading {
            TreeTableReading::EveryEntryRead => None,
            TreeTableReading::PointerAllZero => {
                Some("树表指针全零（core 的根记录读者照它读盘 0 槽 0、读不出）")
            }
            TreeTableReading::Unreadable => Some("树表读不出"),
            TreeTableReading::SomeEntryShorterThanTheFieldTable => Some("树表有条目比字段表短"),
        };
        if let Some(why_not_judged) = why_not_judged {
            // 下标在范围内不是这里判的：`geometry_of` 已经把 R > 3 的槽拒掉，`roots` 的区域号都来自 `root_slot_positions`。
            let device = geometry.region_devices[usize::try_from(*region).expect(
                "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
            )];
            roots_whose_tree_table_was_not_read_in_full.push(format!(
                "盘 {device} 上的根（实例 {}、txg {}）：{why_not_judged}",
                root.instance, root.checkpoint_txg
            ));
        }
        let Some(tree_table_placement) = references.tree_table_placement else {
            continue;
        };
        for (tree, birth_txg) in &references.tree_table_birth_txg {
            sightings_by_tree
                .entry(*tree)
                .or_default()
                .push(TreeTableEntrySightingInTheRing {
                    root_instance: root.instance,
                    root_checkpoint_txg: root.checkpoint_txg,
                    tree_table_placement,
                    birth_txg: *birth_txg,
                });
        }
    }
    let mut compared_trees = 0u64;
    for (tree, sightings) in &sightings_by_tree {
        let distinct_tree_tables: BTreeSet<(u32, u64)> = sightings
            .iter()
            .map(|sighting| sighting.tree_table_placement)
            .collect();
        if distinct_tree_tables.len() < 2 {
            continue;
        }
        compared_trees += 1;
        let first_birth_txg = sightings[0].birth_txg;
        let birth_txg_is_the_same_in_every_readable_root = sightings
            .iter()
            .all(|sighting| sighting.birth_txg == first_birth_txg);
        judgements.judge(
            TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
            birth_txg_is_the_same_in_every_readable_root,
            || {
                let seen: Vec<String> = sightings
                    .iter()
                    .map(|sighting| {
                        format!(
                            "根（实例 {}、txg {}）记 {}",
                            sighting.root_instance,
                            sighting.root_checkpoint_txg,
                            sighting.birth_txg
                        )
                    })
                    .collect();
                format!(
                    "树 {tree} 的树表条目诞生 txg 在根环全部读得出的根里不一：{}",
                    seen.join("、")
                )
            },
        );
    }
    if !roots_whose_tree_table_was_not_read_in_full.is_empty() {
        judgements.partially_judged(
            TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
            format!(
                "比到的 {compared_trees} 棵树诞生 txg 都对得上；这几条读得出、自证过的根树表条目没读全（树表指针全零、树表读不出或有条目比字段表短），没进比较：{}",
                roots_whose_tree_table_was_not_read_in_full.join("；")
            ),
        );
    } else if compared_trees == 0 {
        judgements.not_applicable(
            TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
            "根环里读得出的根中，没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
        );
    }
}

/// I-3.9 与 I-9.14 共用的这一遍：候选集里每条根各走一遍取引用集合与树表。候选集只剩一条根时两条都报「不适用」——
/// 「最早不再引用它的那条有效根」与「跨根相同」都要第二条根才有内容（2026-09-18 用户定案随 C374 立条时定的口径）。
fn judge_release_generation_and_tree_table_birth(
    reader: &dyn ImageReader,
    roots: &[(u64, u64, crate::RootView)],
    candidate_indexes: &[usize],
    newest_index: usize,
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    if candidate_indexes.len() < 2 {
        judgements.not_applicable(
            "I-3.9",
            "回退候选集里只有一条根：没有第二条根能见证「不再引用这个落点」",
        );
        judgements.not_applicable(
            "I-9.14",
            "回退候选集里只有一条根：树表条目没有第二条根的那一份可比",
        );
        return;
    }
    // 最新根那棵账定这一遍走多深：它一条已释放记录都没有时 I-3.9 判不了，别的根只要树表（I-9.14 那一半）。
    let newest = references_of_root(
        reader,
        &roots[newest_index].2.record_bytes,
        ReferenceScanDepth::EveryReferencedPlacement,
        cache,
    );
    let depth = if newest
        .allocation_records
        .iter()
        .any(|record| record.is_released)
    {
        ReferenceScanDepth::EveryReferencedPlacement
    } else {
        ReferenceScanDepth::TreeTableOnly
    };
    let mut scanned: Vec<ScannedCandidateRoot> = Vec::with_capacity(candidate_indexes.len());
    for index in candidate_indexes.iter().copied() {
        scanned.push(ScannedCandidateRoot {
            root_index: index,
            checkpoint_txg: roots[index].2.checkpoint_txg,
            references: if index == newest_index {
                newest.clone()
            } else {
                references_of_root(reader, &roots[index].2.record_bytes, depth, cache)
            },
        });
    }
    scanned.sort_unstable_by_key(|root| root.checkpoint_txg);
    let newest_position = scanned
        .iter()
        .position(|root| root.root_index == newest_index)
        .expect("最新根在回退候选集里（`check_pool_image` 无条件把它放进去）");
    judge_release_generations(&scanned, newest_position, judgements);
    judge_tree_table_birth_txg(&scanned, judgements);
}

/// I-5.4 按版本那一层判的一版：称呼（判红与部分没判时点名，说清是候选根、由记录施加出来的那一版、还是下一次挂载先施加的那一版）、
/// 树表指针与「树表 0 条那一版的分配记录树根」（根环里的根取根记录偏移 36 与 342 起的 86 字节，
/// 另两类取记录新根段偏移 0 与 274 起的那两条：D23（journal 的角色与格式） 已定项 4 的字段表，施加一条记录 = 把所选根的这几个字段换成记录里的）。
struct VersionToJudgeTheAllocationRecordsOf<'record> {
    name: String,
    tree_table_pointer: &'record [u8],
    allocation_record_tree_root_held_outside_the_tree_table: &'record [u8],
}

/// I-5.4（分配记录罩住的槽互不相交）：候选集里每条有效根的分配记录树，同一块盘上任意两条记录（不论已分配、已释放、释放代是否已低于
/// 回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交（`.claude/kb/invariants.md` I-5.4 那一行）。I-5.1（物理范围不重叠） 管的是
/// 树里的引用，判不出记录之间的重叠：代码三方第二轮 Z1-d 的镜像上引用互不重叠，而两条分配记录罩住同一个槽，下一次可写挂载重建分配器时 panic。
/// 几条根指着同一个分配记录树根时（照抄上一版的根）那棵树只判一次；每个 (树, 盘) 判一格，那块盘上只有一条记录也算判过。
/// 树按位置寻址、可以多层（D8（核心索引结构） 已定项 14）：整棵读下来，同一块盘上跨叶的记录一起比。
/// 走不到记录的根不判：树表或分配记录树读不出、位置对不上（由 I-2.1 / I-1.1 / I-7.2 说话）。
///
/// 同一条下另判一格「一条根只有一棵分配记录树」：树表非空（带树）的那一版，根记录里「树表 0 条那一版的分配记录树根」那 86 字节
/// （偏移 342）必须全零——那一版的账住树表种类 3 那一条，根记录这一项只有树表 0 条、写过行的那一版写非零
/// （D22（单元原子性怎么合成） 已定项 7 字段表「分配记录树根指针」那一行；判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`
/// 第二节 Y1 `allocation_record_tree_root` 那一行）。非零时这一版有两棵分配记录树，「这条根的分配记录树」不止一棵。
/// 不变量清单里没有一条专管根记录这一项，暂挂在 I-5.4 下（归属交主 agent 定）。
/// 树表指针非零而树表读不出的根，这一格判不了（认不出带不带树），它那一版自己的分配记录树（树表种类 3 那一条指着的那一棵）也走不到：
/// 不计成立，I-5.4 报部分没判、逐版列出（判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2
/// 「I-5.4 按版本：坏盘」那一行的改法 F1：此前这种根整版不进评估，别的版本的树走到了叶就报成立）。
/// 树表读得出、0 条而根记录那一项非零的根（写过行的那一版，C512（树表 0 条的一版上被换下的单元记在哪）），它那一版的分配记录树就是
/// 根记录直接持有的那一棵：那棵树走不到叶时同样不计成立，I-5.4 报部分没判、逐版列出（F1 的同形，第 33 批造盘面证实：此前同样整版不进评估）。
/// 这一格成立的那一次评估只在**这一版自己的**分配记录树（树表种类 3 那一条指着的那一棵）走到叶时计进 I-5.4（判决
/// `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-5.4 挂法（C2）」那一行，按版本那一层是
/// `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2 的零轮收严）：没走到的那几版不计，I-5.4 报部分没判、逐版列出没计的那几版
/// ——否则带树一版上这一格成立一次就压过不适用或别的版本没判到的那一半，这一版分配记录树两份清零的镜像上 I-5.4 报成立。
/// 不成立照报违例（[`Judgements::partially_judged`] 的契约：判出违例照报违例）。
///
/// 按版本那一层判的不只是候选根：由记录施加出来的那几版（[`versions_applied_only_by_records`]）与下一次挂载先施加的那一版
/// （[`version_the_next_mount_applies_first`]）也各判一版，树表指针与「树表 0 条那一版的分配记录树根」取记录新根段里的，判法与候选根同一套
/// （[`VersionToJudgeTheAllocationRecordsOf`]；判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2
/// 「由记录施加出来的版本与下一次挂载先施加的那一版」那一行的改法 RV：此前这两类版本整版不进，后一类上两条记录罩住同一个槽，
/// 正是这一条立条时要拦的「下一次可写挂载重建分配器时」那件事）。读不出的照上面几格报部分没判，点名的称呼说清是哪一类版本。
fn judge_allocation_records_disjoint(
    reader: &dyn ImageReader,
    versions: &[VersionToJudgeTheAllocationRecordsOf<'_>],
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut seen_allocation_nodes: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
    // 走到了叶的分配记录树根（按第一条位置条目认，与 `seen_allocation_nodes` 同一个键）：别的版本照抄同一个根时，
    // 它那一版的树也算走到了叶（那棵树只判一次）；先走的那一版读不出的，照抄它的那一版同样算没走到。
    let mut allocation_roots_walked_to_their_leaves: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
    // 带树一版「根记录里树表 0 条那一版的分配记录树根全零」那一格：不成立当场照报违例；成立的那几版按走到的次序记下哪一版、
    // 这一版自己的分配记录树走没走到叶，走完全部候选根之后才逐版定计不计进评估（判决
    // `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2「I-5.4『成立要至少一棵树走到叶』」那一行的零轮收严：
    // 按版本记，别的版本的树走到了叶不替这一版作保）。
    let mut versions_where_the_root_record_allocation_tree_root_is_zero: Vec<(String, bool)> =
        Vec::new();
    // 树表指针非零而树表读不出的那几版：认不出带不带树，「根记录里分配记录树根全零」那一格与这一版自己的分配记录树都没判。
    let mut versions_whose_tree_table_does_not_read_back: Vec<String> = Vec::new();
    // 树表指针全零的那几版：core 照它读盘 0 槽 0、读不出（用户 2026-10-05 定甲），与树表读不出同一处置；此前当这一版没有树表。
    let mut versions_whose_tree_table_pointer_is_all_zero: Vec<String> = Vec::new();
    // 树表读得出、0 条，分配记录树由根记录直接持有（C512（树表 0 条的一版上被换下的单元记在哪））而那棵树走不到叶的那几版：
    // 这一版自己的分配记录树没判（判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2「I-5.4 按版本：坏盘」
    // 那一行改法 F1 的同形，`research/prompts/m3-formal-code-b30-r1-opus-output.md` 第 201 行记下、第 33 批造盘面证实）。
    let mut versions_whose_root_record_allocation_record_tree_does_not_read_back: Vec<String> =
        Vec::new();
    for version in versions {
        let mut this_version_carries_trees_and_a_zero_root_record_allocation_tree_root = false;
        let mut this_version_holds_its_allocation_record_tree_in_the_root_record = false;
        // 这一版的分配记录树：树表里种类 3 那一条指着的根，以及树表 0 条那一版由根记录（或记录新根段）直接持有的那一棵
        // （C512（树表 0 条的一版上被换下的单元记在哪），代码审阅 6c 第 9 条：原来这一棵不读）。带文件的一版后者恒全零。
        let mut allocation_roots = vec![parse_node_pointer(
            version.allocation_record_tree_root_held_outside_the_tree_table,
        )];
        let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
        if tree_table_pointer.all_zero {
            versions_whose_tree_table_pointer_is_all_zero.push(version.name.clone());
        } else {
            let tree_table_read_back =
                read_index_node_without_judging(reader, &tree_table_pointer, cache);
            if tree_table_read_back.is_none() {
                versions_whose_tree_table_does_not_read_back.push(version.name.clone());
            }
            if let Some(tree_table) = tree_table_read_back {
                this_version_holds_its_allocation_record_tree_in_the_root_record =
                    tree_table.entries.is_empty() && !allocation_roots[0].all_zero;
                if !tree_table.entries.is_empty() {
                    if allocation_roots[0].all_zero {
                        this_version_carries_trees_and_a_zero_root_record_allocation_tree_root =
                            true;
                    } else {
                        judgements.judge("I-5.4", false, || {
                            format!(
                                "{}的树表有 {} 条（带树的一版），根记录里树表 0 条那一版的分配记录树根却不是全零：这一版有两棵分配记录树",
                                version.name,
                                tree_table.entries.len()
                            )
                        });
                    }
                }
                allocation_roots.extend(
                    tree_table
                        .entries
                        .iter()
                        .filter(|entry| {
                            entry.len() >= tree_table_entry_bytes()
                                && read_u16(entry, 10) == TREE_KIND_ALLOCATION
                        })
                        .map(|entry| parse_node_pointer(&entry[14..100])),
                );
            }
        }
        let mut this_version_allocation_record_tree_walked_to_its_leaves = false;
        for allocation_root in allocation_roots {
            if allocation_root.all_zero {
                continue;
            }
            let first_location = &allocation_root.locations[0];
            let allocation_root_key = (
                first_location.device,
                first_location.slot,
                first_location.checksum,
            );
            if !seen_allocation_nodes.insert(allocation_root_key) {
                if allocation_roots_walked_to_their_leaves.contains(&allocation_root_key) {
                    this_version_allocation_record_tree_walked_to_its_leaves = true;
                }
                continue;
            }
            let Some((_, records)) =
                allocation_record_tree_without_judging(reader, &allocation_root, cache)
            else {
                continue;
            };
            judge_allocation_record_ranges(&records, &version.name, judgements);
            allocation_roots_walked_to_their_leaves.insert(allocation_root_key);
            this_version_allocation_record_tree_walked_to_its_leaves = true;
        }
        if this_version_carries_trees_and_a_zero_root_record_allocation_tree_root {
            versions_where_the_root_record_allocation_tree_root_is_zero.push((
                version.name.clone(),
                this_version_allocation_record_tree_walked_to_its_leaves,
            ));
        }
        if this_version_holds_its_allocation_record_tree_in_the_root_record
            && !this_version_allocation_record_tree_walked_to_its_leaves
        {
            versions_whose_root_record_allocation_record_tree_does_not_read_back
                .push(version.name.clone());
        }
    }
    let some_allocation_record_tree_walked_to_its_leaves =
        !allocation_roots_walked_to_their_leaves.is_empty();
    let mut versions_not_counted: Vec<&str> = Vec::new();
    for (version, this_version_allocation_record_tree_walked_to_its_leaves) in
        &versions_where_the_root_record_allocation_tree_root_is_zero
    {
        if *this_version_allocation_record_tree_walked_to_its_leaves {
            judgements.judge("I-5.4", true, String::new);
        } else {
            versions_not_counted.push(version);
        }
    }
    if !some_allocation_record_tree_walked_to_its_leaves {
        judgements.not_applicable(
            "I-5.4",
            "候选集里没有一条根的分配记录树走得到叶：第 0 代树表是空的，或树表 / 分配记录树读不出、位置对不上",
        );
    }
    // `partially_judged` 只留第一次给的说明：两种没判到的版本合成一句报。
    let mut unjudged_parts: Vec<String> = Vec::new();
    if !versions_not_counted.is_empty() {
        unjudged_parts.push(if some_allocation_record_tree_walked_to_its_leaves {
            format!(
                "带树一版根记录里分配记录树根全零那一格成立而没计（那一版自己的分配记录树读不出、走不到叶，别的版本的树走到了叶不替它作保）：{}",
                versions_not_counted.join("、")
            )
        } else {
            format!(
                "候选集里没有一条根的分配记录树走得到叶，记录之间互不相交一格都没判；带树一版根记录里分配记录树根全零那一格成立而没计：{}",
                versions_not_counted.join("、")
            )
        });
    }
    if !versions_whose_tree_table_does_not_read_back.is_empty() {
        unjudged_parts.push(format!(
            "树表指针非零而树表读不出的版本，认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：{}",
            versions_whose_tree_table_does_not_read_back.join("、")
        ));
    }
    if !versions_whose_tree_table_pointer_is_all_zero.is_empty() {
        unjudged_parts.push(format!(
            "树表指针全零的版本（core 照它读盘 0 槽 0、读不出），认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：{}",
            versions_whose_tree_table_pointer_is_all_zero.join("、")
        ));
    }
    if !versions_whose_root_record_allocation_record_tree_does_not_read_back.is_empty() {
        unjudged_parts.push(format!(
            "树表 0 条、分配记录树由根记录直接持有的版本，那棵树读不出、走不到叶，它自己的分配记录一条都没判（别的版本的树走到了叶不替它作保）：{}",
            versions_whose_root_record_allocation_record_tree_does_not_read_back.join("、")
        ));
    }
    if !unjudged_parts.is_empty() {
        judgements.partially_judged("I-5.4", unjudged_parts.join("；"));
    }
}

/// 一个起点槽上的单元头能不能用、能用时头里的诞生代号（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的偏移表：
/// 码 1 诞生代号 75、明文头末尾 105；码 2 诞生代号 52 + 2k、明文头末尾 86 + 2k；码 3 诞生代号 73、明文头末尾 107；
/// 头校验和三类都在 10）。「能用」= magic 对、类标签是 1 / 2 / 3、头校验和过——头读不出、类标签不认、头校验和不过的
/// 这一格没有对象（I-3.10 那一行射程 ③：读不出本身由 I-1.1 与 I-2.1 管）。
fn birth_txg_in_a_usable_unit_header(header: &[u8]) -> Option<u64> {
    if header.len() < UNIT_HEADER_SCAN_BYTES || &header[..4] != b"SFSU" {
        return None;
    }
    let (plain_header_end, birth_txg_offset) = match header[6] {
        1 => (105, 75),
        2 => {
            let key_span = 2 * usize::from(header[51]);
            (86 + key_span, 52 + key_span)
        }
        3 => (107, 73),
        _outside_the_three_unit_classes => return None,
    };
    checksum_field_holds(header, plain_header_end, UNIT_HEADER_CHECKSUM_OFFSET)
        .then(|| read_u64(header, birth_txg_offset))
}

/// 下一次挂载的恢复会先施加的那一版的树表指针（改法 E，`research/prompts/m2-wave3-code-r1-main-verification.md`
/// 第三节 Y4 与第四节第 2 条，被攻过零轮）：那一版由 [`version_the_next_mount_applies_first`] 认，树表指针取它新根段里的
/// （D23（journal 的角色与格式） 已定项 4 的字段表）。I-3.10 经这里读，I-5.4 按版本那一层读同一个函数认出的同一版。
fn tree_table_pointer_of_the_version_the_next_mount_applies_first(
    version_the_next_mount_applies_first: Option<&VersionTheNextMountAppliesFirst>,
) -> Option<PointerView> {
    version_the_next_mount_applies_first.map(|version| {
        parse_node_pointer(&version.new_root_segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER])
    })
}

/// 下一次挂载的恢复会先施加的那一版：重放施加的第一次发布，根槽没落盘、还没有哪一次恢复施加过它（与 [`VersionAppliedOnlyByRecords`] 不是一类：
/// 那一类已被施加、实例表里有行）。
struct VersionTheNextMountAppliesFirst {
    instance: u32,
    checkpoint_txg: u64,
    new_root_segment: Vec<u8>,
}

/// 下一次挂载的恢复会先施加的那一版：它的 (实例代号, txg) 与那次发布末条记录的整段新根段。I-3.10 只取树表指针
/// （[`tree_table_pointer_of_the_version_the_next_mount_applies_first`]），I-5.4 按版本那一层还要「树表 0 条那一版的分配记录树根」；
/// 两条都经这一个函数认这一版，同一个条件。
///
/// 认法（改法 R1，判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P1 与第三节第 1 条）：所选根取最新根
/// （与 I-4.1 同一个择法），照 I-4.1 那一份重放（[`replay_the_next_mount_would_do`]）施加的**第一次**发布、取它末条那一份新根段
/// （已定项 15：core 施加末条那一份，`crates/singlefs-core/src/recovery.rs` 拼新根那一段）。core 的施加单位是一次发布：
/// 从链首起连续走到末条才整次施加，末条没落、同一次发布前面某条没落（断号）、反向链对不上、序号或 txg 接不上、末条之后同一次发布还有记录、
/// 点名单元验不过的那次发布都不施加；这里用同一份重放，那次发布同样不认（此前只看带末条标志的那一条，断号的发布 core 不施加、这里照认）。
/// 重放一次发布都不施加、锚点认不定、所选根 txg 没有后继时没有这一版。前缀停在后面某次发布的点名单元上时，停之前整次收下的发布
/// core 照施加，第一次那一版照认（I-4.1 对这种镜像报不适用，那是 I-4.1 自己的射程）。这一版的 txg 比最新根大，根环里没有它的根。
///
/// **只接这一步**：恢复连着施加几次发布时，第二次及以后那几版的分配记录这里读不到；读它只多判 I-3.10 一格，
/// 它的分配记录与它罩住的单元是同一次发布写出来的，不论施加与否两个数都同源。
fn version_the_next_mount_applies_first(
    reader: &dyn ImageReader,
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    newest_root: &crate::RootView,
    journal_ring_bytes: u64,
) -> Option<VersionTheNextMountAppliesFirst> {
    let records_the_replay_reads = records_the_replay_reads(records_by_device);
    let last_record_of_the_first_publish_applied = match replay_the_next_mount_would_do(
        reader,
        &records_the_replay_reads,
        newest_root,
        in_flight_record_limit_of_the_journal_ring(journal_ring_bytes),
    ) {
        ReplayTheNextMountWouldDo::AnchorUndecided
        | ReplayTheNextMountWouldDo::ChosenRootTxgHasNoSuccessor
        | ReplayTheNextMountWouldDo::NothingApplied => None,
        ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify {
            last_record_of_the_first_publish_applied_before_it,
        } => last_record_of_the_first_publish_applied_before_it,
        ReplayTheNextMountWouldDo::Applied {
            last_record_of_the_first_publish,
            publishes: _,
            records: _,
            last_record_counter: _,
            last_record: _,
        } => Some(last_record_of_the_first_publish),
    }?;
    Some(VersionTheNextMountAppliesFirst {
        instance: last_record_of_the_first_publish_applied.instance,
        checkpoint_txg: last_record_of_the_first_publish_applied.checkpoint_txg,
        new_root_segment: last_record_of_the_first_publish_applied
            .new_root_segment
            .clone(),
    })
}

/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）读哪几片分配记录：候选集里每条根的分配记录树
/// （树表里种类 3 那一条指着的根），树表 0 条那一版的分配记录树（根记录直接持有，C512（树表 0 条的一版上被换下的单元记在哪）），
/// 由记录施加出来的那几版的分配记录树（它们的树表在记录的新根段里），以及下一次挂载会先施加的那一版的分配记录树
/// （[`tree_table_pointer_of_the_version_the_next_mount_applies_first`]）。只读不判：判定归 I-2.1 / I-1.1 那几条的走读。
fn allocation_record_node_pointers_of_the_candidate_versions(
    reader: &dyn ImageReader,
    roots: &[(u64, u64, crate::RootView)],
    candidate_indexes: &[usize],
    versions_applied_only_by_records: &[VersionAppliedOnlyByRecords],
    tree_table_pointer_of_the_version_the_next_mount_applies_first: Option<PointerView>,
    cache: &mut IndexNodeCache,
) -> Vec<PointerView> {
    let mut tree_table_pointers: Vec<PointerView> = Vec::new();
    let mut allocation_node_pointers: Vec<PointerView> = Vec::new();
    for index in candidate_indexes.iter().copied() {
        let record = &roots[index].2.record_bytes;
        tree_table_pointers.push(parse_node_pointer(&record[36..122]));
        allocation_node_pointers.push(parse_node_pointer(&record[342..428]));
    }
    for version in versions_applied_only_by_records {
        let (tree_table_pointer, _) = version.tree_table_and_mapping_root_pointers();
        tree_table_pointers.push(parse_node_pointer(tree_table_pointer));
    }
    tree_table_pointers.extend(tree_table_pointer_of_the_version_the_next_mount_applies_first);
    for tree_table_pointer in &tree_table_pointers {
        if tree_table_pointer.all_zero {
            continue;
        }
        let Some(tree_table) = read_index_node_without_judging(reader, tree_table_pointer, cache)
        else {
            continue;
        };
        for entry in &tree_table.entries {
            if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION
            {
                continue;
            }
            allocation_node_pointers.push(parse_node_pointer(&entry[14..100]));
        }
    }
    allocation_node_pointers.retain(|pointer| !pointer.all_zero);
    allocation_node_pointers
}

/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）：任一**未带已释放标志**的分配记录，它的分配代等于它罩住的那个单元头里的
/// 诞生代号——记录与单元是同一次发布写出来的，两个数同源（`.claude/kb/invariants.md` I-3.10 那一行）。射程照那一行：
/// ① 只管未释放的记录（已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里））；
/// ② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头——记录的 key 就是起点槽，按它读；
/// ③ 那个槽上读不出可用的单元头时这一格没有对象，不判（[`birth_txg_in_a_usable_unit_header`]）。
/// 它拦的是：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着。
/// 几条根指着同一个分配记录树节点时（照抄上一版的根）那个节点只读一次；同一条记录（盘、槽、代）出现在几片里只判一次。
/// 一格都没判到时整条报不适用并带理由，不报成立。
fn judge_allocation_generations_against_unit_births(
    reader: &dyn ImageReader,
    allocation_node_pointers: &[PointerView],
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut seen_allocation_nodes: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
    let mut examined_records: BTreeSet<(u32, u64, u64)> = BTreeSet::new();
    let mut judged_records = 0u64;
    for pointer in allocation_node_pointers {
        let first_location = &pointer.locations[0];
        if !seen_allocation_nodes.insert((
            first_location.device,
            first_location.slot,
            first_location.checksum,
        )) {
            continue;
        }
        let Some((_, records)) = allocation_record_tree_without_judging(reader, pointer, cache)
        else {
            continue;
        };
        for record in records {
            if record.is_released
                || !examined_records.insert((record.device, record.slot, record.generation))
            {
                continue;
            }
            let Some(birth_txg) = reader
                .read(
                    record.device,
                    record.slot * SLOT_BYTES,
                    UNIT_HEADER_SCAN_BYTES,
                )
                .as_deref()
                .and_then(birth_txg_in_a_usable_unit_header)
            else {
                continue;
            };
            judged_records += 1;
            judgements.judge("I-3.10", record.generation == birth_txg, || {
                format!(
                    "盘 {} 槽 {} 的分配记录（未释放、跨 {} 槽）分配代 {}，它罩住的单元头里的诞生代号是 {birth_txg}",
                    record.device, record.slot, record.span_slots, record.generation
                )
            });
        }
    }
    if judged_records == 0 {
        judgements.not_applicable(
            "I-3.10",
            if examined_records.is_empty() {
                "候选集里没有一棵分配记录树走得到未释放的记录（第 0 代树表是空的，或分配记录树读不出、位置对不上）"
            } else {
                "未释放的分配记录罩住的起点槽上一个可用的单元头都读不出：读不出本身归 I-1.1 与 I-2.1"
            },
        );
    }
}

/// 一条根的树表里用户可见的两棵树（inode 树、extent 树）那两条根指针的盘上原样（D16（发布语义） 已定项 1「「非空」从盘上怎么认」：
/// 比的是树表条目里这两棵树的根指针，不比树表单元自己的落点——每一次发布（连空发布）都重写树表单元）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct UserVisibleTreeRootPointers {
    inode_tree: Option<Vec<u8>>,
    extent_tree: Option<Vec<u8>>,
}

impl UserVisibleTreeRootPointers {
    /// 没有前一条有效根时拿它比：两棵树的条目都没有（mkfs 种的第 0 版树表就是这样）。
    const ABSENT: Self = Self {
        inode_tree: None,
        extent_tree: None,
    };
}

/// 一条根的树表里 inode 树与 extent 树那两条根指针（树表条目偏移 14 起的 86 字节）。只读不判。
/// 树表指针全零、树表读不出或解不开、条目窄于字段表、同一种树出现两条 ⇒ None：这条根算不算非空判不了，不按空或非空猜。
/// 条目格式没有条款的那几种树（livelist、稀疏旁表、deadlist 与不认识的码）与分配记录树、记账树不进「非空」的比较，跳过。
fn user_visible_tree_root_pointers(
    reader: &dyn ImageReader,
    record: &[u8],
    cache: &mut IndexNodeCache,
) -> Option<UserVisibleTreeRootPointers> {
    let tree_table_pointer = parse_node_pointer(&record[36..122]);
    if tree_table_pointer.all_zero {
        return None;
    }
    let tree_table = read_index_node_without_judging(reader, &tree_table_pointer, cache)?;
    let mut pointers = UserVisibleTreeRootPointers::ABSENT;
    for entry in &tree_table.entries {
        if entry.len() < tree_table_entry_bytes() {
            return None;
        }
        let pointer_of_this_tree = match TreeKindForReferenceScan::of(read_u16(entry, 10)) {
            TreeKindForReferenceScan::Inode => &mut pointers.inode_tree,
            TreeKindForReferenceScan::Extent => &mut pointers.extent_tree,
            TreeKindForReferenceScan::Allocation
            | TreeKindForReferenceScan::Accounting
            | TreeKindForReferenceScan::DeletedInodes
            | TreeKindForReferenceScan::ContainerTree
            | TreeKindForReferenceScan::WithoutWalkableEntryFormat(_) => continue,
        };
        if pointer_of_this_tree
            .replace(entry[14..100].to_vec())
            .is_some()
        {
            return None;
        }
    }
    Some(pointers)
}

/// 实例表一行（`kind` 0 的记录）按字段表解出来：主走读判行那一半（`judge_instance_table_rows`）与 I-7.9 读一条根自己的实例表共用。
fn parse_instance_table_row(row: &[u8]) -> InstanceTableRow {
    InstanceTableRow {
        instance: u32::from_le_bytes(row[1..5].try_into().expect("4 字节")),
        published_checkpoint_txg: u64::from_le_bytes(row[5..13].try_into().expect("8 字节")),
        applied_transaction_high_water_mark: u64::from_le_bytes(
            row[13..21].try_into().expect("8 字节"),
        ),
    }
}

/// 一条根自己指着的那张实例表的全部行，沿链读（D18（块里携带什么信息） 已定项 11：根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片）。
/// 只读不判：哪一片读不出（两份的整单元 CRC 都对不上位置条目）、解不开（头判不过、类不是码 3、记录类型不是 4、记录宽不是 88、
/// 记录数 × 记录宽超过声明长度或声明长度超过容量、身份不是 (0, 4, 片序号, 0)、链指针记录之前有不是行的记录、行的 flags 不是 0、
/// 最后一条不是合法的链指针记录，含它说有下一片而位置指针头部有一段不是第一版的取值）⇒ None，判定归主走读那一遍（I-2.1、I-1.7、I-1.1、I-3.14）。
/// I-3.14 的五形与主走读、与实现同一个读法：解不开。
/// 迭代上界：第 k 轮要求读到的单元身份里容器号是 k，两轮读到同一个单元就要它的容器号同时等于两个数 ⇒ 每一轮读的是盘上不同的单元，
/// 轮数不超过盘上装得下的 32 KiB 单元数。跨轮带的是下一片的指针、片序号与已经接起来的行；提前出口只有读不出、解不开。
fn instance_table_rows_of_root_without_judging(
    reader: &dyn ImageReader,
    record: &[u8],
) -> Option<Vec<InstanceTableRow>> {
    let mut pointer = parse_node_pointer(&record[170..256]);
    let mut page_index: u64 = 0;
    let mut rows: Vec<InstanceTableRow> = Vec::new();
    loop {
        let unit = read_unit_without_judging(reader, &pointer, data_unit_bytes())?;
        if !matches!(check_unit(&unit), Ok(3)) {
            return None;
        }
        let identity = (
            read_u64(&unit, 43),
            read_u16(&unit, 51),
            read_u64(&unit, 53),
            read_u64(&unit, 61),
        );
        let record_count = usize::from(read_u16(&unit, 69));
        let record_width = usize::from(read_u16(&unit, 71));
        let declared_length = usize::from(read_u16(&unit, 8));
        if identity != (0, PACKED_TYPE_INSTANCE_TABLE, page_index, 0)
            || record_width != INSTANCE_TABLE_RECORD_BYTES
            || record_count * record_width > declared_length
            || declared_length > data_unit_bytes() - 136
        {
            return None;
        }
        let records: Vec<&[u8]> = unit[136..136 + record_count * record_width]
            .chunks(record_width)
            .collect();
        let (chain_record, rows_of_this_page) = records.split_last()?;
        for row in rows_of_this_page {
            if row[0] != 0
                || row[INSTANCE_TABLE_ROW_FLAGS_OFFSET]
                    != INSTANCE_TABLE_ROW_FLAGS_OF_THE_FIRST_VERSION
            {
                return None;
            }
            rows.push(parse_instance_table_row(row));
        }
        match chain_record_view(chain_record) {
            ChainRecordView::LastPage => return Some(rows),
            ChainRecordView::NextPage(next_page) => {
                pointer = next_page;
                page_index += 1;
            }
            ChainRecordView::Malformed => return None,
        }
    }
}

/// 按一张实例表，这条根在不在被抛弃的时间线上：有它那个实例的行 (i, Ti, Wi) 且它的 txg > Ti
/// （D23（journal 的角色与格式） 已定项 14 回退段的候选集规则）。
fn abandoned_by_instance_table_rows(
    instance_table_rows: &[InstanceTableRow],
    root: &crate::RootView,
) -> bool {
    instance_table_rows.iter().any(|row| {
        row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
    })
}

/// 一条根记录指着的那张实例表，沿链读出来的全部行（读法与 I-7.9 读一条根自己的实例表相同，只读不判）。
/// 给记录核对器（`singlefs_harness::crash::check_records_against`）按「恢复落到的那一版的实例表」判一次发布在不在被抛弃的时间线上用
/// （D13（验证路线） 已定项 7 的记录核对器；判据是 D23（journal 的角色与格式） 已定项 14 回退段的候选集规则）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceTableOfRootRecord {
    rows: Vec<InstanceTableRow>,
}

impl InstanceTableOfRootRecord {
    /// `root_record` 是一条根记录的字节（至少到实例表指针的末尾，偏移 256）。
    /// 链上哪一片读不出、解不开 ⇒ None（读法见 `instance_table_rows_of_root_without_judging`）。
    #[must_use]
    pub fn read(reader: &dyn ImageReader, root_record: &[u8]) -> Option<Self> {
        instance_table_rows_of_root_without_judging(reader, root_record).map(|rows| Self { rows })
    }

    /// (实例代号, checkpoint_txg) 那一版在这张表下被抛弃：有它那个实例的行 (i, Ti, Wi) 且它的 txg > Ti。
    #[must_use]
    pub fn abandons(&self, instance: u32, checkpoint_txg: u64) -> bool {
        self.rows
            .iter()
            .any(|row| row.instance == instance && checkpoint_txg > row.published_checkpoint_txg)
    }
}

/// D16（发布语义） 已定项 1 的抬 F 上限：min(每块盘上最新的有效根的 txg, 第 4 新的不同状态的非空有效根的 txg)，不足 4 个时取最旧有效根的 txg。
/// `distinct_non_empty_valid_root_txgs` 是去重之后计入的那几条（[`newest_first_distinct_state_txgs`]）。
/// 对计入的集合单调不减：多计入一条，第 4 新的只会更新或不变，而最旧有效根不新于任何一条计入的根。
fn rollback_floor_ceiling_from(
    newest_valid_root_txg_on_every_device: u64,
    distinct_non_empty_valid_root_txgs: &[u64],
    oldest_valid_root_txg: u64,
) -> u64 {
    let mut newest_first = distinct_non_empty_valid_root_txgs.to_vec();
    newest_first.sort_unstable_by(|left, right| right.cmp(left));
    let fourth_newest_non_empty_or_oldest_valid = newest_first
        .get(3)
        .copied()
        .unwrap_or(oldest_valid_root_txg);
    newest_valid_root_txg_on_every_device.min(fourth_newest_non_empty_or_oldest_valid)
}

/// 数「第 4 新的非空有效根」时去重（D16（发布语义） 已定项 1「「非空」从盘上怎么认」末段）：按 (txg, 实例) 从新到旧，
/// 一条非空根只有在它的两棵用户可见树的根指针与每一条比它新、已计入的根都不同时才计入。`always_counted` 是空不空判不了的那几条
/// （树表读不出，状态也就读不出）：算上限的上沿时它们一律按「计入、状态与谁都不同」算，它们不挡更旧的根计入——计入的集合只多不少，
/// 上沿不低于真值；算下沿时不传它们。交回计入的那几条的 txg。
///
/// 循环：按 (txg, 实例) 从新到旧走一遍（轮数 = 两串条数之和），跨轮携带已计入的状态；没有提前出口。
fn newest_first_distinct_state_txgs(
    determinable_non_empty: &[(u64, u32, UserVisibleTreeRootPointers)],
    always_counted: &[(u64, u32)],
) -> Vec<u64> {
    let mut newest_first: Vec<(u64, u32, Option<&UserVisibleTreeRootPointers>)> =
        determinable_non_empty
            .iter()
            .map(|(txg, instance, pointers)| (*txg, *instance, Some(pointers)))
            .chain(
                always_counted
                    .iter()
                    .map(|(txg, instance)| (*txg, *instance, None)),
            )
            .collect();
    newest_first.sort_unstable_by_key(|(txg, instance, _)| std::cmp::Reverse((*txg, *instance)));
    let mut counted_states: Vec<&UserVisibleTreeRootPointers> = Vec::new();
    let mut counted = Vec::new();
    for (txg, _, pointers) in newest_first {
        match pointers {
            None => counted.push(txg),
            Some(state) => {
                if !counted_states.contains(&state) {
                    counted_states.push(state);
                    counted.push(txg);
                }
            }
        }
    }
    counted
}

/// 按一条抬 F 的根之前的根算出来的上限。有效根里有树表读不出的（它或它前一条读不出，它空不空判不了）时上限只知道一个区间：
/// 那几条都算空是下沿、都算非空是上沿（[`rollback_floor_ceiling_from`] 对非空集合单调）；树表都读得出时两沿相等。
struct RollbackFloorCeilingBeforeTheRaise {
    lowest_possible: u64,
    highest_possible: u64,
    newest_valid_root_per_device: BTreeMap<u32, u64>,
    non_empty_valid_root_txgs: Vec<u64>,
    valid_root_txgs_whose_emptiness_is_undeterminable: Vec<u64>,
    oldest_valid_root_txg: u64,
}

/// 算一条抬 F 的根 `raising_root` 那一刻的上限（I-7.9（回退下界 F 不高于抬 F 的上限），用户 2026-09-24 定）：
/// 只用根环里 txg 比它小、落在锚在这张镜像根环里最新根上的一圈窗口里的根（txg > N − 3S，`window_anchored_at_the_newest_root_of_the_image`），
/// 挡的是环上有洞时洞里留下的旧根（理由见 [`judge_rollback_floor_raises_against_their_ceilings`]）。抬 F 的上限本身取环里全部有效根
/// （用户 2026-10-04 定 RGall），原来另取的那个锚在它自己 txg 上的窗口（照上限只在一圈窗口里取）随之拿掉：N 是镜像里最新根的 txg、
/// 不小于被判那条根的 txg，那个窗口本来就罩着这个窗口，拿掉不改判定。有效 = 按**它自己**指着的实例表不被抛弃 ∧ txg ≥ `floor_before_the_raise`（抬之前的 F）；
/// 非空 = 按 (txg, 实例) 排，跟前一条有效根比两条用户可见树的根指针，最旧的那条跟 [`UserVisibleTreeRootPointers::ABSENT`] 比。
/// 它自己指着的实例表读不出、它之前一条有效根都没有 ⇒ None（上限无从算起：抬 F 的入口在这两格上拒，盘上出现这样一条抬 F 的根，
/// 要么表后来坏了，要么撑它的根已被环盖掉）。
fn rollback_floor_ceiling_before_the_raise(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    raising_root: &crate::RootView,
    floor_before_the_raise: u64,
    window_anchored_at_the_newest_root_of_the_image: OneRevolutionWindow,
    cache: &mut IndexNodeCache,
) -> Option<RollbackFloorCeilingBeforeTheRaise> {
    let instance_table_rows_of_the_raising_root =
        instance_table_rows_of_root_without_judging(reader, &raising_root.record_bytes)?;
    let mut valid_roots_before: Vec<(u32, &crate::RootView)> = roots
        .iter()
        .filter(|(_, _, root)| {
            root.checkpoint_txg < raising_root.checkpoint_txg
                && window_anchored_at_the_newest_root_of_the_image.contains(root.checkpoint_txg)
                && root.checkpoint_txg >= floor_before_the_raise
                && !abandoned_by_instance_table_rows(&instance_table_rows_of_the_raising_root, root)
        })
        .map(|(region, _, root)| {
            // 下标在范围内不是这里判的：`geometry_of` 已经把 R > 3 的槽拒掉，`roots` 的区域号都来自 `root_slot_positions`。
            let device = geometry.region_devices[usize::try_from(*region).expect(
                "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
            )];
            (device, root)
        })
        .collect();
    valid_roots_before.sort_unstable_by_key(|(_, root)| (root.checkpoint_txg, root.instance));
    let oldest_valid_root_txg = valid_roots_before.first()?.1.checkpoint_txg;
    let mut newest_valid_root_per_device: BTreeMap<u32, u64> = BTreeMap::new();
    for (device, root) in &valid_roots_before {
        let newest = newest_valid_root_per_device
            .entry(*device)
            .or_insert(root.checkpoint_txg);
        *newest = (*newest).max(root.checkpoint_txg);
    }
    let newest_valid_root_txg_on_every_device = newest_valid_root_per_device
        .values()
        .copied()
        .min()
        .expect("上面 first() 过：至少一条有效根，它的盘在这张表里");
    let mut non_empty_valid_roots: Vec<(u64, u32, UserVisibleTreeRootPointers)> = Vec::new();
    let mut valid_roots_whose_emptiness_is_undeterminable: Vec<(u64, u32)> = Vec::new();
    let mut previous_valid_root_pointers = Some(UserVisibleTreeRootPointers::ABSENT);
    for (_, root) in &valid_roots_before {
        let pointers = user_visible_tree_root_pointers(reader, &root.record_bytes, cache);
        match (&pointers, &previous_valid_root_pointers) {
            (Some(this_root), Some(previous_root)) => {
                if this_root != previous_root {
                    non_empty_valid_roots.push((
                        root.checkpoint_txg,
                        root.instance,
                        this_root.clone(),
                    ));
                }
            }
            (None, _) | (_, None) => {
                valid_roots_whose_emptiness_is_undeterminable
                    .push((root.checkpoint_txg, root.instance));
            }
        }
        previous_valid_root_pointers = pointers;
    }
    let non_empty_valid_root_txgs: Vec<u64> = non_empty_valid_roots
        .iter()
        .map(|(txg, _, _)| *txg)
        .collect();
    let valid_root_txgs_whose_emptiness_is_undeterminable: Vec<u64> =
        valid_roots_whose_emptiness_is_undeterminable
            .iter()
            .map(|(txg, _)| *txg)
            .collect();
    Some(RollbackFloorCeilingBeforeTheRaise {
        lowest_possible: rollback_floor_ceiling_from(
            newest_valid_root_txg_on_every_device,
            &newest_first_distinct_state_txgs(&non_empty_valid_roots, &[]),
            oldest_valid_root_txg,
        ),
        highest_possible: rollback_floor_ceiling_from(
            newest_valid_root_txg_on_every_device,
            &newest_first_distinct_state_txgs(
                &non_empty_valid_roots,
                &valid_roots_whose_emptiness_is_undeterminable,
            ),
            oldest_valid_root_txg,
        ),
        newest_valid_root_per_device,
        non_empty_valid_root_txgs,
        valid_root_txgs_whose_emptiness_is_undeterminable,
        oldest_valid_root_txg,
    })
}

/// I-7.9（回退下界 F 不高于抬 F 的上限）：只判**抬 F 的那一条根**，拿它之前的根算上限（用户 2026-09-24 定，
/// 里程碑「覆盖写、释放、回退与复用」收口表第 26 行；定义是实二报告 `impl-m2-checker2` 第八节那一种）。
///
/// 抬 F 的根：同一实例里 txg 比它小的最新那条根（它的前一条）带的 F 比它带的低。回退那次与新实例的第一条根没有同实例的前一条，
/// 不算抬——它带的是恢复算出来的 F_生效，而按它自己的实例表，当初撑起那个 F 的非空根已在被抛弃的时间线上，拿它算上限会在合法的
/// 「抬 F 之后回退到候选集里的根」上判红（固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version`）。
/// 一次抬 F 推几次空发布、每次都带新 F 时，只有第一条是抬 F 的根（后几条的前一条已带新 F）。
///
/// 上限只用根环里 txg 比它小、落在锚在镜像最新根上的一圈窗口里的根算（[`rollback_floor_ceiling_before_the_raise`]），
/// 不在后来每张镜像上重算：抬之后再回退，后来的根会把当初撑着 F 的非空根判成无效，而 F 不回落。
/// ⚠️ 后来的镜像上当初撑着上限的根可以已被盖掉：环按 txg 从旧到新被盖时，盖掉的是一段最旧的前缀，重算只会更宽；环上有洞时不是这样——
/// 洞那一槽留着上一圈的根，比它新的根反倒先被盖掉，按 [`rollback_floor_ceiling_from`] 的单调性重算出来的上限变低、在合法镜像上判红
/// （里程碑三第 8 批调查的 inject-6 种子 1 那一形）。窗口只锚在被判那条根上挡不住它：洞里那条旧根 t − 3S（t 是洞的 txg、t 晚于抬 F 那条根）
/// 落在那个窗口里。按这张镜像的最新根 N 取一圈窗口就挡住了：洞里留下的根都 ≤ N − 3S；而窗口里 txg 比抬 F 那条根小的根在抬 F 那一刻
/// 就在环里、之后不会被盖（盖它要一条 txg ≥ 它 + 3S > N 的根），所以留下的是抬 F 那一刻那批根去掉最旧的一段前缀，重算只会更宽。
/// 抬 F 那一刻的上限取环里全部有效根（RGall）时同一个道理成立：那一刻环里的根（窗口外洞里的也算）去掉 ≤ N − 3S 的那一段，仍是去掉最旧的一段前缀。
/// 带卸载记号那一支同一个机理：洞里那条旧根顶上来、比它新的前一条已被盖掉，「txg 比它小的最新那条根」变旧、在合法镜像上判红
/// （随机历史种子 31045、1037 那两段，9-6 报告第六节 Q1）；两支都取同一组根（用户 2026-10-04 定「乙，两支都取两个窗口」，
/// 同日 RGall 之后锚在被判那条根上的那个窗口被锚在镜像最新根上的罩住，只留后者）。
///
/// 按这条根 flags 里的卸载记号分两支（D22（单元原子性怎么合成） 已定项 7，用户 2026-09-26 定 (b)+(c) 合用）：
/// 带记号的（正常卸载那一串）只判 F 不高于它之前的根（锚在镜像最新根上的一圈窗口里）里 txg 最大那条的 txg，一条都没有不判；不带的（准入抬 F）照下面的准入上限判。
///
/// 不带记号那一支三种结局：F ≤ 上限的下沿 ⇒ 成立；F > 上限的上沿 ⇒ 违例；落在两沿之间、或上限无从算起 ⇒ 这条根不判。
/// 一条根都没判到时整条报不适用并带理由，不报成立。
fn judge_rollback_floor_raises_against_their_ceilings(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    window_anchored_at_the_newest_root_of_the_image: OneRevolutionWindow,
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut raising_roots_judged = 0u64;
    let mut raising_roots_not_judged = 0u64;
    for (_, _, raising_root) in roots {
        let Some(previous_root_of_the_same_instance) = roots
            .iter()
            .map(|(_, _, root)| root)
            .filter(|root| {
                root.instance == raising_root.instance
                    && root.checkpoint_txg < raising_root.checkpoint_txg
            })
            .max_by_key(|root| root.checkpoint_txg)
        else {
            continue;
        };
        let floor_before_the_raise = previous_root_of_the_same_instance.rollback_floor;
        if raising_root.rollback_floor <= floor_before_the_raise {
            continue;
        }
        let (raising_instance, raising_txg) = (raising_root.instance, raising_root.checkpoint_txg);
        let raised_floor = raising_root.rollback_floor;
        match raising_root.unmount_marker {
            // 带卸载记号那一支：正常卸载把 F 抬到现行那一版的 txg、不另判上限（D16（发布语义） 已定项 1「抬 F 的上限」），
            // 判 F 不高于它之前的根里 txg 最大的那条的 txg。「之前的根」与准入那一支取同一组：txg 比它小、落在锚在这张镜像最新根上的
            // 一圈窗口里（用户 2026-10-04 定「乙，两支都取两个窗口」；同日 RGall 之后锚在它自己 txg 上的那一个被这一个罩住、拿掉）；
            // 一条都没有（镜像比它新出一圈以上、它前面那几条已被盖掉）这条根不判。记号是不是只由卸载入口打，盘上没有别的东西核它，归层 0（C557）。
            crate::UnmountMarkerView::WrittenByTheUnmountSequence => {
                let Some(newest_root_txg_before_the_raise) = roots
                    .iter()
                    .map(|(_, _, root)| root.checkpoint_txg)
                    .filter(|txg| {
                        *txg < raising_txg
                            && window_anchored_at_the_newest_root_of_the_image.contains(*txg)
                    })
                    .max()
                else {
                    raising_roots_not_judged += 1;
                    continue;
                };
                raising_roots_judged += 1;
                judgements.judge(
                    "I-7.9",
                    raised_floor <= newest_root_txg_before_the_raise,
                    || {
                        format!(
                            "实例 {raising_instance} txg {raising_txg} 那条带卸载记号的根把回退下界 F 从 {floor_before_the_raise} 抬到 {raised_floor}，高于根环里 txg 比它小的最新那条根的 txg {newest_root_txg_before_the_raise}"
                        )
                    },
                );
                continue;
            }
            crate::UnmountMarkerView::NotWrittenByTheUnmountSequence => {}
        }
        let Some(ceiling) = rollback_floor_ceiling_before_the_raise(
            reader,
            geometry,
            roots,
            raising_root,
            floor_before_the_raise,
            window_anchored_at_the_newest_root_of_the_image,
            cache,
        ) else {
            raising_roots_not_judged += 1;
            continue;
        };
        if raised_floor > ceiling.lowest_possible && raised_floor <= ceiling.highest_possible {
            raising_roots_not_judged += 1;
            continue;
        }
        raising_roots_judged += 1;
        judgements.judge("I-7.9", raised_floor <= ceiling.lowest_possible, || {
            format!(
                "实例 {raising_instance} txg {raising_txg} 那条根把回退下界 F 从 {floor_before_the_raise} 抬到 {raised_floor}，高于它之前的根算出的抬 F 上限 {}：每块盘上最新的有效根 {:?}，非空有效根 {:?}，空不空判不了的有效根 {:?}，最旧有效根 txg {}",
                ceiling.highest_possible,
                ceiling.newest_valid_root_per_device,
                ceiling.non_empty_valid_root_txgs,
                ceiling.valid_root_txgs_whose_emptiness_is_undeterminable,
                ceiling.oldest_valid_root_txg
            )
        });
    }
    if raising_roots_judged == 0 {
        judgements.not_applicable(
            "I-7.9",
            if raising_roots_not_judged == 0 {
                "根环里没有抬 F 的根：每条根带的 F 都不高于同一实例里它前一条根带的（回退与新实例的第一条根不算抬）"
            } else {
                "根环里抬 F 的根都判不了：它自己指着的实例表读不出、它之前一条有效根都没有，或有效根的树表读不出、上限落在它带的 F 两边，或带卸载记号、锚在镜像最新根上的一圈窗口里没有它之前的根"
            },
        );
    }
}

/// I-7.12（系统配置 F 不低于同盘根上的 F）：每块盘分开判——这块盘两槽里自证过（整槽校验和过且 fsid 与本池相同）的系统配置槽中
/// 回退下界 F 的最大值，不低于这块盘根槽里每一条自证过的根带的 F（主 agent 2026-09-26 立，按 SysPre 的写序推的）。
/// 根住哪块盘按它所在区域的设备身份认（`PoolGeometry::region_devices`）。每块有自证过的系统配置槽的盘判一格（这块盘上一条根都没有时
/// 那一格照样成立：没有要比的根）；两槽都自证不过的盘报「不适用」、不判。一块盘都判不了时整条报不适用并带理由。
fn judge_system_configuration_floor_against_the_roots_on_each_device(
    verified_system_configurations: &[(u32, Vec<crate::SystemConfigurationView>)],
    roots: &[(u64, u64, crate::RootView)],
    geometry: &PoolGeometry,
    judgements: &mut Judgements,
) {
    let mut devices_judged = 0u64;
    for (device, slots) in verified_system_configurations {
        let Some(highest_system_configuration_floor) =
            slots.iter().map(|view| view.rollback_floor).max()
        else {
            continue;
        };
        devices_judged += 1;
        let highest_floor_root_on_this_device = roots
            .iter()
            .filter(|(region, _, _)| {
                // 下标在范围内不是这里判的：`geometry_of` 已经把 R > 3 的槽拒掉，`roots` 的区域号都来自 `root_slot_positions`。
                geometry.region_devices[usize::try_from(*region).expect(
                    "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
                )] == *device
            })
            .map(|(_, _, root)| root)
            .max_by_key(|root| root.rollback_floor);
        let holds = highest_floor_root_on_this_device
            .is_none_or(|root| root.rollback_floor <= highest_system_configuration_floor);
        judgements.judge("I-7.12", holds, || {
            let root = highest_floor_root_on_this_device.expect("判红只在这块盘上有根时");
            format!(
                "盘 {device}：两槽里自证过的系统配置带的 F 最大 {highest_system_configuration_floor}，低于这块盘上实例 {} txg {} 那条根带的 F {}",
                root.instance, root.checkpoint_txg, root.rollback_floor
            )
        });
    }
    if devices_judged == 0 {
        judgements.not_applicable(
            "I-7.12",
            "池里没有一块盘有自证过、fsid 与本池相同的系统配置槽：没有系统配置里的 F 可比",
        );
    }
}

/// 一棵分配记录树的全部记录上逐盘判 I-5.4：同一块盘上的记录按起点排好，相邻两条不相交就是两两不相交（跨度非负）。
fn judge_allocation_record_ranges(
    records: &[AllocationRecordView],
    version_name: &str,
    judgements: &mut Judgements,
) {
    let mut ranges_per_device: BTreeMap<u32, Vec<(u64, u64)>> = BTreeMap::new();
    for record in records {
        ranges_per_device
            .entry(record.device)
            .or_default()
            .push((record.slot, record.span_slots));
    }
    for (device, mut ranges) in ranges_per_device {
        ranges.sort_unstable();
        match ranges
            .windows(2)
            .find(|pair| pair[0].0 + pair[0].1 > pair[1].0)
        {
            Some(pair) => {
                let ((first_slot, first_span), (second_slot, second_span)) = (pair[0], pair[1]);
                judgements.judge("I-5.4", false, || {
                    format!(
                        "{version_name}那棵分配记录树、盘 {device}：槽 {first_slot} 跨 {first_span} 的记录与槽 {second_slot} 跨 {second_span} 的记录罩住同一个槽"
                    )
                });
            }
            None => judgements.judge("I-5.4", true, String::new),
        }
    }
}

/// 根环里没被走过的那几条根（低于 F 的、被抛弃的）：判隔离的记录缺位置项时才走一遍，只为收它们指针里的位置项。
struct RootsNotWalked<'reader> {
    reader: &'reader dyn ImageReader,
    filesystem_identifier_low: u64,
    mount_root_instance: u32,
    mount_root_checkpoint_txg: u64,
    root_records: Vec<Vec<u8>>,
}

/// 判隔离的记录要的位置项校验和：(盘, 起点槽) → 指着那一份的各条位置项带的单元校验和。先用走过的那几版收到的；
/// 查一份没收到时，把根环里没走过的根整遍走一次（判定丢掉，只收位置项）补进来，只补这一次。
struct LocationEntryChecksumsForQuarantine<'reader> {
    known: BTreeMap<(u32, u64), BTreeSet<u32>>,
    roots_not_walked_yet: Option<RootsNotWalked<'reader>>,
}

impl LocationEntryChecksumsForQuarantine<'_> {
    fn at(&mut self, device: u32, slot: u64) -> Option<&BTreeSet<u32>> {
        if !self.known.contains_key(&(device, slot)) {
            if let Some(roots) = self.roots_not_walked_yet.take() {
                let mut harvesting_walk = Walk::starting_with(
                    roots.reader,
                    Judgements::default(),
                    roots.filesystem_identifier_low,
                    roots.mount_root_instance,
                    roots.mount_root_checkpoint_txg,
                );
                for record in &roots.root_records {
                    harvesting_walk.walk_root(record, false);
                }
                for (place, checksums) in harvesting_walk.location_entry_checksums {
                    self.known.entry(place).or_default().extend(checksums);
                }
            }
        }
        self.known.get(&(device, slot))
    }
}

/// 收口表第 43 行取丁-defer（用户 2026-09-28 定，被攻过一轮；`research/prompts/abandoned-floor-r1-main-verification.md` M1）：
/// I-3.1（已分配统计对得上） 的「实际遍历」并上这样的槽——F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的分配记录树里
/// 罩住它的记录是已释放、释放代高于回收门槛 `reclaim_threshold` = max(F_生效, 环里最旧有效根) 的（还在 defer 里，分配器还不许回收，
/// 记账照算已分配；门槛 2026-10-02 起取一圈窗口，2026-10-04 用户定 RGall 退回整个环，D16（发布语义） 已定项 1）。逐盘交回槽数。
/// 最新根的树表或分配记录树读不出时一槽都不并（照旧判）。
fn deferred_slots_referenced_only_below_the_floor_per_device(
    reader: &dyn ImageReader,
    filesystem_identifier_low: u64,
    newest_root: &crate::RootView,
    roots_below_the_floor: &[&crate::RootView],
    walked_slots: &BTreeSet<(u32, u64)>,
    reclaim_threshold: u64,
    cache: &mut IndexNodeCache,
) -> BTreeMap<u32, u64> {
    let mut deferred_per_device: BTreeMap<u32, u64> = BTreeMap::new();
    let tree_table_pointer = parse_node_pointer(&newest_root.record_bytes[36..122]);
    if tree_table_pointer.all_zero {
        return deferred_per_device;
    }
    let Some(tree_table) = read_index_node_without_judging(reader, &tree_table_pointer, cache)
    else {
        return deferred_per_device;
    };
    let mut released_above_the_floor: BTreeSet<(u32, u64)> = BTreeSet::new();
    for entry in &tree_table.entries {
        if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION {
            continue;
        }
        let allocation_root = parse_node_pointer(&entry[14..100]);
        if allocation_root.all_zero {
            continue;
        }
        let Some((_, records)) =
            allocation_record_tree_without_judging(reader, &allocation_root, cache)
        else {
            continue;
        };
        for record in records {
            if allocation_record_is_released_above_the_reclaim_threshold(&record, reclaim_threshold)
            {
                released_above_the_floor.extend(
                    (record.slot..record.slot + record.span_slots)
                        .map(|slot| (record.device, slot)),
                );
            }
        }
    }
    let mut slots_referenced_by_the_roots_below_the_floor: Vec<(u32, u64)> = Vec::new();
    for root in roots_below_the_floor {
        let mut walk_of_the_root_below_the_floor = Walk::starting_with(
            reader,
            Judgements::default(),
            filesystem_identifier_low,
            newest_root.instance,
            newest_root.checkpoint_txg,
        );
        walk_of_the_root_below_the_floor.walk_root(&root.record_bytes, false);
        for (device, start_slot, span_slots) in walk_of_the_root_below_the_floor.references.keys() {
            slots_referenced_by_the_roots_below_the_floor
                .extend((*start_slot..*start_slot + *span_slots).map(|slot| (*device, slot)));
        }
    }
    let deferred = deferred_slots_among_the_references_of_the_selected_roots(
        slots_referenced_by_the_roots_below_the_floor,
        walked_slots,
        &released_above_the_floor,
    );
    for (device, _) in deferred {
        *deferred_per_device.entry(device).or_insert(0) += 1;
    }
    deferred_per_device
}

/// 一条根按最新根指着的实例表是有效的还是被抛弃的（D23（journal 的角色与格式） 已定项 14 候选集：(i, T) 有效 ⟺ 无 i 的行，
/// 或有行 (i, Ti, Wi) 且 T ≤ Ti）。丁-defer 选根与它的回收门槛都按它分。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootStandingInTheNewestRootsTable {
    Valid,
    Abandoned,
}

/// 丁-defer 的选根式（收口表第 43 行，用户 2026-09-28 定）：引用要并进 I-3.1（已分配统计对得上） 的「实际遍历」的根，
/// 是按最新根的实例表有效、txg 在 F_生效 之下的那几条。被抛弃的根不并：它们独占的槽由影子账隔离、在现行账里不是已释放。
/// 「被抛弃的根也并」那一式（遗留判决 LR5）在合法盘面上与这一式并进同一组槽，只在双故障形下可能不同，
/// 等价性留档在 `tests` 的小世界穷举里（`deferred_selection_merging_the_abandoned_roots_too_selects_the_same_slots_on_every_small_world_but_the_double_fault_form`）。
fn root_references_are_merged_into_the_deferred_slots_below_the_floor(
    standing: RootStandingInTheNewestRootsTable,
    checkpoint_txg: u64,
    effective_rollback_floor: u64,
) -> bool {
    match standing {
        RootStandingInTheNewestRootsTable::Valid => checkpoint_txg < effective_rollback_floor,
        RootStandingInTheNewestRootsTable::Abandoned => false,
    }
}

/// 丁-defer 的门槛与分配器的回收门槛同一个：max(F_生效, 环里最旧有效根)（D16（发布语义） 已定项 1；`research/prompts/m3-ring-r3-main-verification.md`
/// 第二节第 5 行把它与回收门槛同取一圈窗口，用户 2026-10-04 定 RGall 一起退回整个环，`research/prompts/m3-reclaim-window-r1-main-verification.md`
/// 第六节 Q1）——释放代落在 (F_生效, 环里最旧有效根] 的槽分配器已经回收、不在「已分配」里，再按 F_生效 并进来就多算。
/// 环里一条有效根都没有时取 F_生效。
fn deferred_slots_reclaim_threshold(
    standing_and_txg_of_each_readable_root: impl IntoIterator<
        Item = (RootStandingInTheNewestRootsTable, u64),
    >,
    effective_rollback_floor: u64,
) -> u64 {
    standing_and_txg_of_each_readable_root
        .into_iter()
        .filter(|(standing, _)| match standing {
            RootStandingInTheNewestRootsTable::Valid => true,
            RootStandingInTheNewestRootsTable::Abandoned => false,
        })
        .map(|(_, checkpoint_txg)| checkpoint_txg)
        .min()
        .map_or(effective_rollback_floor, |oldest_valid_root_in_the_ring| {
            oldest_valid_root_in_the_ring.max(effective_rollback_floor)
        })
}

/// 最新根的账里这条记录还在 defer 里：已释放、释放代高于回收门槛（分配器还不许回收，记账照算已分配）。
fn allocation_record_is_released_above_the_reclaim_threshold(
    record: &AllocationRecordView,
    reclaim_threshold: u64,
) -> bool {
    record.is_released && record.generation > reclaim_threshold
}

/// 丁-defer 并进来的槽：选中的根引用着（`slots_referenced_by_the_selected_roots`，(盘, 槽)，可以重复）、候选集那一遍没走到、
/// 最新根的账里还在 defer 里的。
fn deferred_slots_among_the_references_of_the_selected_roots(
    slots_referenced_by_the_selected_roots: impl IntoIterator<Item = (u32, u64)>,
    walked_slots: &BTreeSet<(u32, u64)>,
    released_above_the_reclaim_threshold: &BTreeSet<(u32, u64)>,
) -> BTreeSet<(u32, u64)> {
    slots_referenced_by_the_selected_roots
        .into_iter()
        .filter(|key| {
            !walked_slots.contains(key) && released_above_the_reclaim_threshold.contains(key)
        })
        .collect()
}

/// 隔离的记录（D19（块指针的结构与宽度预算） 已定项 5：释放之前读盘核出对不上的那个单元，每块盘上的分配记录都留在已分配，
/// 映射条目去掉、没有任何根再引用它）在 I-3.1（已分配统计对得上） 与 I-3.11（已分配减 defer 等于最新根走读） 上怎么认
/// （用户 2026-09-25 定；**按单元判**是主 agent 同日定的读法，I-3.1 / I-3.11 两行的措辞随后由书记员改）：已分配而没有根引用的记录，
/// 它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出、或与指着它的位置项里的单元校验和对不上（与 D19 已定项 5 硬规则 1
/// 写者读盘核的是同一个判据：实四丙交回 ⑤，丢一次写落在复用槽上时那一份是一个完整的旧单元、自证照样过，按自证判就不豁免），
/// 这个单元在每块盘上的那几条记录都豁免；这个单元每一份都读得出且对得上，照旧判违例。位置项取走过的每一版里指着那一份的
/// （[`LocationEntryChecksumsForQuarantine`]），走过的版本都不指着它时再走一遍根环里没走过的根（低于 F 的、被抛弃的）补上。按单元而不按份判：写者只要一份核出对不上就把两块盘的记录一起留在已分配
/// （D19 已定项 5「另一块盘上那一份对得上也一起留，各盘的账保持对称」），按份判的话对得上那块盘上的那一条恒红。
///
/// 读的是 `root` 那棵分配记录树（树表里种类 3 那一条指着的根，按位置整棵走下去，`allocation_record_tree_without_judging`）：未释放、`referenced_start_slots`
/// 里没有哪个引用起在它那一槽的记录，按单元（起点槽、跨度——第一版一个单元两盘同槽）归组；一组的每一份按 (盘, 槽, 跨度) 读那一整份单元、
/// 算整单元 CRC-32C 与位置项里的比——有一份读不出或对不上，整组豁免。逐盘交回豁免的槽数；
/// 树表或分配记录树读不出、位置对不上时那一棵一格都不豁免（照旧判）。
fn quarantined_slots_exempted_per_device(
    reader: &dyn ImageReader,
    root: &crate::RootView,
    referenced_start_slots: &BTreeSet<(u32, u64)>,
    location_entry_checksums: &mut LocationEntryChecksumsForQuarantine<'_>,
    cache: &mut IndexNodeCache,
) -> BTreeMap<u32, u64> {
    let mut exempted: BTreeMap<u32, u64> = BTreeMap::new();
    let tree_table_pointer = parse_node_pointer(&root.record_bytes[36..122]);
    if tree_table_pointer.all_zero {
        return exempted;
    }
    let Some(tree_table) = read_index_node_without_judging(reader, &tree_table_pointer, cache)
    else {
        return exempted;
    };
    // 一个单元（起点槽、跨度）→ 记着它、没有根引用的那几块盘。
    let mut unreferenced_units: BTreeMap<(u64, u64), Vec<u32>> = BTreeMap::new();
    for entry in &tree_table.entries {
        if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION {
            continue;
        }
        let allocation_root = parse_node_pointer(&entry[14..100]);
        if allocation_root.all_zero {
            continue;
        }
        let Some((_, records)) =
            allocation_record_tree_without_judging(reader, &allocation_root, cache)
        else {
            continue;
        };
        for record in records {
            if record.is_released || referenced_start_slots.contains(&(record.device, record.slot))
            {
                continue;
            }
            unreferenced_units
                .entry((record.slot, record.span_slots))
                .or_default()
                .push(record.device);
        }
    }
    for ((slot, span_slots), devices_of_the_unit) in unreferenced_units {
        let mut copy_matches_its_location_entry = |device: u32| {
            let Some(unit) = usize::try_from(span_slots * SLOT_BYTES)
                .ok()
                .and_then(|unit_bytes| reader.read(device, slot * SLOT_BYTES, unit_bytes))
            else {
                return false;
            };
            match location_entry_checksums.at(device, slot) {
                Some(checksums) => checksums.contains(&crc32_castagnoli_table(&unit)),
                // 走过的版本与根环里别的根都没有一条位置项指着这一份（引用它的那几版已经转出根环）：D19 已定项 5 的判据没有东西可比，
                // 这一格条款没写，照改之前的读法退回自证（magic、头校验和、载荷 CRC）——交主 agent（实七报告）。
                None => check_unit(&unit).is_ok(),
            }
        };
        let some_copy_fails = devices_of_the_unit
            .iter()
            .any(|device| !copy_matches_its_location_entry(*device));
        if some_copy_fails {
            for device in devices_of_the_unit {
                *exempted.entry(device).or_insert(0) += span_slots;
            }
        }
    }
    exempted
}

/// 一条根的分配记录树里有几条记录：按 checker 自己的字段表读这条根的树表，找种类 3（分配记录）的条目，数它根节点叶里的条目
/// （第一版分配记录树只有一个节点；一条记录记一个单元，D3（空间分配） 已定项 7）。只读不判——给理想模型的对拍数分配记录墙的真条数
/// （增补 3 第 2 件代码三方第一轮判决第三节第 1 条：从镜像上现数，不用分配器的状态）。
/// 树表里没有分配记录树的条目、或那一条的根指针全零时是 0 条：树表 0 条的一版上没有分配记录树，mkfs 写的两个单元的记录要到
/// 第一个文件版本才落盘。树表指针全零、树表或分配记录树的节点读不出或条目比字段表窄、分配记录树有内部节点（内部条目格式没有条款，
/// 总审核 D8-D11 发现 16）时数不出，交回 None。
#[must_use]
pub fn allocation_record_count_under_root(
    reader: &dyn ImageReader,
    root: &crate::RootView,
) -> Option<usize> {
    let mut cache = IndexNodeCache::new();
    let tree_table_pointer = parse_node_pointer(&root.record_bytes[36..122]);
    if tree_table_pointer.all_zero {
        return None;
    }
    let tree_table = read_index_node_without_judging(reader, &tree_table_pointer, &mut cache)?;
    let mut records = 0;
    for entry in &tree_table.entries {
        if entry.len() < tree_table_entry_bytes() {
            return None;
        }
        if read_u16(entry, 10) != TREE_KIND_ALLOCATION {
            continue;
        }
        let allocation_root = parse_node_pointer(&entry[14..100]);
        if allocation_root.all_zero {
            continue;
        }
        let (_, records_of_the_tree) =
            allocation_record_tree_without_judging(reader, &allocation_root, &mut cache)?;
        records += records_of_the_tree.len();
    }
    Some(records)
}

/// 单元头校验和字段的偏移（共同前缀，三类同一处）。
const UNIT_HEADER_CHECKSUM_OFFSET: usize = 10;
/// 写序那 10 字节里事务号低 48 位从第 4 字节起（D18（块里携带什么信息） 已定项 7：实例代号 4 + 事务号 6）。
const WRITE_ORDER_TRANSACTION_OFFSET: usize = 4;

/// I-1.8（归并后版本全序） 罩得到的两类单元。码 2 不进这一条（`.claude/kb/invariants.md` I-1.8 逐字「码 2 不进这条」：
/// 它的写序已收窄成只有实例代号，多版在条目级合并、不在容器级择版本）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContentUnitClass {
    /// 码 1 数据单元。
    Data,
    /// 码 3 打包记录单元。
    PackedRecord,
}

/// 一类内容单元的头里，这一条要读的那几段。码 1 按 D18（块里携带什么信息） 已定项 7 的字段表（五元组 42、诞生代号 75、
/// fsid 83、写序 91、载荷 CRC 101、明文头末尾 105）；码 3 按已定项 11 的类身份段表（标签 42、出生树 43、类型 51、容器号 53、
/// 容器出生代 61、记录数 69、记录宽 71、诞生代号 73、fsid 81、载荷校验和 89、写序 93、出生序号 103、明文头末尾 107）。
struct ContentUnitHeaderOffsets {
    plain_header_end: usize,
    payload_checksum: usize,
    filesystem_identifier: usize,
    birth_txg: usize,
    write_order: usize,
    /// 类身份段里与版本无关的那一段：码 1 的五元组、码 3 的（标签, 出生树, 类型, 容器号, 容器出生代）。
    object_key: std::ops::Range<usize>,
    /// 归并键的两段：「类身份段全部字段含写序」**去掉载荷校验和那一段**。
    /// 载荷校验和不进归并键——它正是「同一组的成员载荷相同」要比的那个量，进了键这一条就恒真。
    merge_key_before_payload_checksum: std::ops::Range<usize>,
    merge_key_after_payload_checksum: std::ops::Range<usize>,
}

/// 已发布谓词的输入，按单元类的两种取法分成两个成员（`.claude/kb/invariants.md` I-1.2（块头写序已发布） 那一行逐字：
/// 「有行 (i, T_pub, W) ⇒ 码 1 要 b ≤ T_pub ∨ n ≤ W、码 2 / 3 要 b ≤ T_pub」）：码 1 那一支带事务号 n，
/// 码 2 / 码 3 那一支不带——它们的取法不读 n，分成两个成员就不必给一个用不上的 n 编个值。
/// 码 2 与码 3 在谓词上是同一支，而它们在 I-1.8（归并后版本全序） 上不是（码 2 不进 I-1.8），
/// 所以这个枚举与 [`ContentUnitClass`] 是两件事，不合并。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteOrderUnderThePublishedPredicate {
    /// 码 1：诞生代号 b 与写序 (i, n)。
    DataUnit {
        birth_txg: u64,
        instance: u32,
        transaction: u64,
    },
    /// 码 2 / 码 3：诞生代号 b 与写序的实例代号 i。
    IndexOrPackedRecordUnit { birth_txg: u64, instance: u32 },
}

impl WriteOrderUnderThePublishedPredicate {
    fn instance(self) -> u32 {
        match self {
            WriteOrderUnderThePublishedPredicate::DataUnit { instance, .. }
            | WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit { instance, .. } => {
                instance
            }
        }
    }

    fn birth_txg(self) -> u64 {
        match self {
            WriteOrderUnderThePublishedPredicate::DataUnit { birth_txg, .. }
            | WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit { birth_txg, .. } => {
                birth_txg
            }
        }
    }
}

impl ContentUnitClass {
    /// 扫描方向读到的头解成谓词的输入：码 1 带事务号，码 3 的取法不读它。
    fn write_order_under_the_published_predicate(
        self,
        birth_txg: u64,
        instance: u32,
        transaction: u64,
    ) -> WriteOrderUnderThePublishedPredicate {
        match self {
            ContentUnitClass::Data => WriteOrderUnderThePublishedPredicate::DataUnit {
                birth_txg,
                instance,
                transaction,
            },
            ContentUnitClass::PackedRecord => {
                WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
                    birth_txg,
                    instance,
                }
            }
        }
    }

    fn of(unit_class_tag: u8) -> Option<Self> {
        match unit_class_tag {
            1 => Some(ContentUnitClass::Data),
            3 => Some(ContentUnitClass::PackedRecord),
            _outside_this_invariants_reach => None,
        }
    }

    fn header_offsets(self) -> ContentUnitHeaderOffsets {
        match self {
            ContentUnitClass::Data => ContentUnitHeaderOffsets {
                plain_header_end: 105,
                payload_checksum: 101,
                filesystem_identifier: 83,
                birth_txg: 75,
                write_order: 91,
                object_key: 42..75,
                merge_key_before_payload_checksum: 42..101,
                merge_key_after_payload_checksum: 105..105,
            },
            ContentUnitClass::PackedRecord => ContentUnitHeaderOffsets {
                plain_header_end: 107,
                payload_checksum: 89,
                filesystem_identifier: 81,
                birth_txg: 73,
                write_order: 93,
                object_key: 42..69,
                merge_key_before_payload_checksum: 42..89,
                merge_key_after_payload_checksum: 93..107,
            },
        }
    }

    fn name(self) -> &'static str {
        match self {
            ContentUnitClass::Data => "码 1 数据",
            ContentUnitClass::PackedRecord => "码 3 打包记录",
        }
    }
}

/// 已发布谓词（I-1.2（块头写序已发布） 的谓词，C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 /
/// D18（块里携带什么信息） 已定项 1）的输入：挂载根（池级 checker 里就是最新的那条根）的实例代号与 txg，
/// 加上它指着的那一版实例表里的行。I-1.8（归并后版本全序） 的射程逐字是「可读**已发布**单元」，扫描方向要拿这个谓词过滤。
struct PublishedPredicate<'rows> {
    mount_root_instance: u32,
    mount_root_checkpoint_txg: u64,
    instance_table_rows: &'rows [InstanceTableRow],
}

impl PublishedPredicate<'_> {
    /// 写序 (i, n) 与诞生代号 b 判没判成已发布。
    ///
    /// **i > 挂载根实例这一支在这里判成「不是已发布」，不判损坏**：I-1.2（块头写序已发布） 那一行逐字是
    /// 「i > 挂载根实例 ⇒ 判损坏」，而那句话站在恢复刚发布完新根的那一刻——那时不存在比挂载根更高的实例。
    /// 池级 checker 对着的是任意一份崩溃镜像：上一次挂载取了号、写出单元、一个根都没发布就崩了，盘上就有
    /// i > 最新根实例的单元，而它是合法的在飞垃圾。所以扫描方向这一支判不了（射程见 `.claude/kb/invariants.md`
    /// I-1.2 的状态列），这里只把它挡在「已发布」之外：I-1.8（归并后版本全序） 的射程要的正是这个。
    /// 被引用的单元走的是另一条路——遍历方向由 I-4.2（无被引用未提交块） 判，见
    /// [`Walk::judge_birth_identity_of_a_referenced_unit`]。
    fn holds_for(&self, write_order: WriteOrderUnderThePublishedPredicate) -> bool {
        match write_order.instance().cmp(&self.mount_root_instance) {
            Ordering::Greater => false,
            Ordering::Equal => write_order.birth_txg() <= self.mount_root_checkpoint_txg,
            Ordering::Less => match self
                .instance_table_rows
                .iter()
                .find(|row| row.instance == write_order.instance())
            {
                None => true,
                Some(row) => match write_order {
                    WriteOrderUnderThePublishedPredicate::DataUnit {
                        birth_txg,
                        transaction,
                        ..
                    } => {
                        birth_txg <= row.published_checkpoint_txg
                            || transaction <= row.applied_transaction_high_water_mark
                    }
                    WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
                        birth_txg,
                        ..
                    } => birth_txg <= row.published_checkpoint_txg,
                },
            },
        }
    }
}

/// I-1.8 ② 的那个全序键。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TotalOrderKey {
    /// 码 3：(诞生代号, 实例代号, 事务号) 三元，诞生代号打头（`.claude/kb/invariants.md` I-1.8 那一行写死）。
    OfPackedRecordUnit(Vec<u64>),
    /// 码 1：**两处条款给的是两个键，没定哪一个作数，这一半停在这里**——I-1.8 那一行逐字「码 1 的键是写序」，
    /// 而 D18（块里携带什么信息） 已定项 1 的射程逐字「同 key 已发布版本的定序靠诞生代号 + 每事务每 key 一种净效果就够，
    /// 写序只在同 txg 跨事务改同 key 时顺带定序」。按前者判，同一实例的两版在盘上能撞出写序逐字节相同——
    /// I-1.8（归并后版本全序） 那一行自己的括注逐字写着「同一实例连着两个 checkpoint 都不分配新事务号时两版写序逐字节相同」，
    /// 而码 3 用「诞生代号打头」解决了同一个问题、码 1 没有；按后者判它们由诞生代号分得开。
    /// ⚠️ 不要拿 `reuse_after_raising_the_floor.rs` 那条脚本当证据：2026-09-21 之前它上面确实有一对，
    /// 那是事务号重号（空发布把按实例的计数拉回去）造出来的，重号修掉之后那条脚本上八个写序互不相同。
    /// 哪一个作数要定案，定之前码 1 不进 ② 这一半；① 那一半（同一组的成员载荷相同）码 1 照判。
    UndecidedForDataUnit,
}

/// 扫描方向读到的一个码 1 / 码 3 已发布单元（「可读」的后半截由 `payload_checksum_holds_on_disk` 现算）。
struct ScannedContentUnit {
    device: u32,
    slot: u64,
    unit_class: ContentUnitClass,
    /// 「类身份段全部字段含写序」去掉载荷校验和那一段：同一份内容的两个副本在这上面逐字节相同。
    merge_key: Vec<u8>,
    /// 类身份段里与版本无关的那一段：I-1.8 里「同 key」的那个 key。
    object_key: Vec<u8>,
    total_order_key: TotalOrderKey,
    payload_checksum: u32,
}

impl ScannedContentUnit {
    fn describe(&self) -> String {
        format!(
            "盘 {} 槽 {} 的{}单元",
            self.device,
            self.slot,
            self.unit_class.name()
        )
    }
}

/// 一个候选槽上的头解成扫描项：magic、类标签在这一条的射程内、头校验和过、fsid 与本池相同、按已发布谓词判成已发布。
/// 任一步不成立就不是这一条判的对象（各自另有不变量说话：I-1.6 / I-2.4 / I-1.4 / I-1.2）。
fn content_unit_of_header(
    header: &[u8],
    device: u32,
    slot: u64,
    filesystem_identifier_low: u64,
    published: &PublishedPredicate<'_>,
) -> Option<ScannedContentUnit> {
    if header.len() < UNIT_HEADER_SCAN_BYTES || &header[..4] != b"SFSU" {
        return None;
    }
    let unit_class = ContentUnitClass::of(header[6])?;
    let offsets = unit_class.header_offsets();
    if !checksum_field_holds(
        header,
        offsets.plain_header_end,
        UNIT_HEADER_CHECKSUM_OFFSET,
    ) || read_u64(header, offsets.filesystem_identifier) != filesystem_identifier_low
    {
        return None;
    }
    let birth_txg = read_u64(header, offsets.birth_txg);
    let instance = read_u32(header, offsets.write_order);
    let transaction =
        read_six_byte_unsigned(header, offsets.write_order + WRITE_ORDER_TRANSACTION_OFFSET);
    if !published.holds_for(unit_class.write_order_under_the_published_predicate(
        birth_txg,
        instance,
        transaction,
    )) {
        return None;
    }
    let total_order_key = match unit_class {
        ContentUnitClass::Data => TotalOrderKey::UndecidedForDataUnit,
        ContentUnitClass::PackedRecord => {
            TotalOrderKey::OfPackedRecordUnit(vec![birth_txg, u64::from(instance), transaction])
        }
    };
    Some(ScannedContentUnit {
        device,
        slot,
        unit_class,
        merge_key: [
            &header[offsets.merge_key_before_payload_checksum],
            &header[offsets.merge_key_after_payload_checksum],
        ]
        .concat(),
        object_key: header[offsets.object_key].to_vec(),
        total_order_key,
        payload_checksum: read_u32(header, offsets.payload_checksum),
    })
}

/// 扫描方向：候选槽上的码 1 / 码 3 已发布单元。32768 的单元占两个槽，第二个槽上没有 magic，扫描自然跳过它。
fn scanned_content_units(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    filesystem_identifier_low: u64,
    published: &PublishedPredicate<'_>,
) -> Vec<ScannedContentUnit> {
    let mut units = Vec::new();
    for device in reader.devices() {
        let slots = reader.candidate_unit_slots(device).unwrap_or_else(|| {
            let end = reader.device_bytes(device).unwrap_or(0) / SLOT_BYTES;
            (geometry.unit_area_start_slot..end).collect()
        });
        for slot in slots {
            let Some(header) = reader.read(device, slot * SLOT_BYTES, UNIT_HEADER_SCAN_BYTES)
            else {
                continue;
            };
            if let Some(unit) =
                content_unit_of_header(&header, device, slot, filesystem_identifier_low, published)
            {
                units.push(unit);
            }
        }
    }
    units
}

/// 这一份在盘上读得出来、而且它的载荷与它自己头里那个载荷校验和对得上——「可读」的后半截。
/// 头校验和过而载荷对不上的那一份不可读（撕裂、被盖），不进 I-1.8 的归并组。
fn payload_checksum_holds_on_disk(reader: &dyn ImageReader, unit: &ScannedContentUnit) -> bool {
    let offsets = unit.unit_class.header_offsets();
    let Some(bytes) = reader.read(unit.device, unit.slot * SLOT_BYTES, data_unit_bytes()) else {
        return false;
    };
    bytes.len() > offsets.plain_header_end
        && read_u32(&bytes, offsets.payload_checksum)
            == crc32_castagnoli_table(&bytes[offsets.plain_header_end..])
}

/// I-1.8（归并后版本全序）：码 1 / 3 的可读已发布单元按「类身份段全部字段含写序」归并成组之后，
/// ① **同一组的成员载荷相同**（码 3 由载荷校验和判、码 1 由载荷 CRC 判），不同即判损坏、不择；
/// ② **同 key 的各组两两全序键不等**——码 3 的键是 (诞生代号, 实例代号, 事务号) 三元。
///    **② 这一半今天只判码 3**：码 1 的键在两处条款里不是同一个，见 `TotalOrderKey::UndecidedForDataUnit`。
/// 射程 w = 2（每次写两盘各一份，D2（RAID 条带策略） 已定项 6）：正常镜像上一组就是同一个单元的两个副本。
/// **「可读」的后半截只在要判红的那一步才现算**：一组里载荷校验和字段全相同时 ① 已经成立、不必把载荷读回来；
/// 字段不同才逐份验载荷——整单元 CRC 是这里最贵的一步，而层 0 每个崩溃状态都要跑这一遍。
/// 交回判了几次：一次都没判到时调用方报「不适用」，不报成立（`test-discipline.md`「检查本身也可能是错的」）。
fn judge_merged_version_total_order(
    reader: &dyn ImageReader,
    units: &[ScannedContentUnit],
    judgements: &mut Judgements,
) -> u64 {
    let mut groups: BTreeMap<&Vec<u8>, Vec<&ScannedContentUnit>> = BTreeMap::new();
    for unit in units {
        groups.entry(&unit.merge_key).or_default().push(unit);
    }
    let mut judged = 0u64;
    for members in groups.values() {
        if members.len() < 2 {
            continue;
        }
        let first_payload_checksum = members[0].payload_checksum;
        let every_member_carries_the_same_payload_checksum = members
            .iter()
            .all(|member| member.payload_checksum == first_payload_checksum);
        // 校验和字段全相同时这一条已经成立，`||` 短路掉后半截：那半截要把每一份载荷整个读回来算 CRC。
        let payloads_are_the_same = every_member_carries_the_same_payload_checksum || {
            let readable_payload_checksums: BTreeSet<u32> = members
                .iter()
                .filter(|member| payload_checksum_holds_on_disk(reader, member))
                .map(|member| member.payload_checksum)
                .collect();
            readable_payload_checksums.len() <= 1
        };
        judged += 1;
        judgements.judge("I-1.8", payloads_are_the_same, || {
            let members_and_their_payloads: Vec<String> = members
                .iter()
                .map(|member| {
                    format!(
                        "{} 载荷校验和 {:#010x}",
                        member.describe(),
                        member.payload_checksum
                    )
                })
                .collect();
            format!(
                "归并成一组的成员载荷不同（{}）：同一组的成员载荷必须相同，不同即判损坏、不择",
                members_and_their_payloads.join("、")
            )
        });
    }
    // 一组里的成员共享归并键，对象 key 与全序键都是归并键的子段 ⇒ 取第一个成员当这一组的代表。
    // 码 1 的全序键没定（`TotalOrderKey::UndecidedForDataUnit`），② 这一半只判码 3；对象 key 首字节是类标签，
    // 一个 key 上的各组必然同类，不会把两类混进同一个桶。
    let mut groups_by_object_key: BTreeMap<&Vec<u8>, Vec<&ScannedContentUnit>> = BTreeMap::new();
    for members in groups.values() {
        if members[0].total_order_key == TotalOrderKey::UndecidedForDataUnit {
            continue;
        }
        groups_by_object_key
            .entry(&members[0].object_key)
            .or_default()
            .push(members[0]);
    }
    for representatives in groups_by_object_key.values() {
        if representatives.len() < 2 {
            continue;
        }
        let distinct_total_order_keys: BTreeSet<&TotalOrderKey> = representatives
            .iter()
            .map(|representative| &representative.total_order_key)
            .collect();
        judged += 1;
        judgements.judge(
            "I-1.8",
            distinct_total_order_keys.len() == representatives.len(),
            || {
                let groups_on_this_key: Vec<String> = representatives
                    .iter()
                    .map(|representative| {
                        format!(
                            "{} 全序键 {:?}",
                            representative.describe(),
                            representative.total_order_key
                        )
                    })
                    .collect();
                format!(
                    "同一个 key 上归并出 {} 组，两两全序键不等这一条不成立（{}）",
                    representatives.len(),
                    groups_on_this_key.join("、")
                )
            },
        );
    }
    judged
}

/// mkfs 把第 0 代种进根环（D22（单元原子性怎么合成） 已定项 8）：I-7.3（环健康性） 的例外那一支按这个代号判。
const GENESIS_CHECKPOINT_TXG: u64 = 0;

/// I-7.3（环健康性）：S（根槽里自证校验和通过的记录集合）中除代号最大者外，至少还存在一条更早代号的记录，
/// 不存在即判红——某次提交把上一代直接覆盖了，轮换逻辑已失效。**例外只有一个**：S 全部是第 0 代时判绿
/// （`.claude/kb/invariants.md` I-7.3 那一行的例外原样照搬；「崩溃后回退到这个态的镜像同样判绿」落在同一支上——
/// 判据只读 S 里的代号，不读这个态是怎么来的）。
/// 调用点在 I-7.1（根槽有效集非空） 之后：S 空时这一条报「不适用」，没有「代号最大者」可谈。
fn judge_root_ring_health(roots: &[(u64, u64, crate::RootView)], judgements: &mut Judgements) {
    let newest_checkpoint_txg = roots
        .iter()
        .map(|(_, _, root)| root.checkpoint_txg)
        .max()
        .expect("S 非空：`check_pool_image` 判过 I-7.1 之后 S 空就返回了，走不到这里");
    let earlier_generation_exists = roots
        .iter()
        .any(|(_, _, root)| root.checkpoint_txg < newest_checkpoint_txg);
    let every_root_is_the_genesis_generation = roots
        .iter()
        .all(|(_, _, root)| root.checkpoint_txg == GENESIS_CHECKPOINT_TXG);
    judgements.judge(
        "I-7.3",
        earlier_generation_exists || every_root_is_the_genesis_generation,
        || {
            let generations: Vec<u64> = roots
                .iter()
                .map(|(_, _, root)| root.checkpoint_txg)
                .collect();
            format!(
                "根环里自证过的根代号 {generations:?}：最大的是 {newest_checkpoint_txg}，除它之外一条更早代号的记录都没有，而它们又不全是第 0 代 ⇒ 上一代被直接覆盖了，下一次撕裂无路可退"
            )
        },
    );
}

/// journal 记录的 jsn 计数器从 1 起、全池接着走（D23（journal 的角色与格式） 已定项 9 / 已定项 19 注 3：
/// 新实例从前缀末 + 1 接着写、不归零）。
const FIRST_JOURNAL_COUNTER: u64 = 1;

/// 记录头里提交标记那 1 字节（D23（journal 的角色与格式） 已定项 7，事务号之后那 1 字节）解出来的样子：
/// 0 不带、1 带，别的值是盘上读到的未知编码——做成显式成员，不并进「不带」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommitMarker {
    Absent,
    Present,
    Unrecognized(u8),
}

impl CommitMarker {
    const fn of(byte: u8) -> Self {
        match byte {
            0 => Self::Absent,
            1 => Self::Present,
            unrecognized => Self::Unrecognized(unrecognized),
        }
    }
}

/// 扫描方向读到的一条 journal 记录：它的实例代号、事务号、落在哪个环槽、反向链字段，以及「下一条记录的反向链该等于的值」。
struct ScannedJournalRecord {
    instance: u32,
    /// 记录头里的 checkpoint_txg：这条记录施加出来的是哪一版（「由记录施加出来、根槽从没落盘的那一版」按它认）。
    checkpoint_txg: u64,
    /// 记录头里的新根段 364 字节原样（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8 + 实例表指针 86 + 分配记录树根指针 86 + 卸载记号 4，
    /// D23（journal 的角色与格式） 已定项 4 / 已定项 15）：
    /// 这条记录施加出来的那一版从哪个树表、哪个映射根走下去。
    new_root_segment: Vec<u8>,
    /// 记录头里的事务号（D23（journal 的角色与格式） 已定项 7）：I-8.7（实例内事务号不重号） 判的就是它。
    transaction: u64,
    /// 记录头里的提交标记：I-8.8（前缀里的事务不被切开） 判的就是它。
    commit_marker: CommitMarker,
    /// 记录头里的本次发布内序号（D23（journal 的角色与格式） 已定项 4）原样：I-8.9（一次发布的记录序号连续且只有末条带标志） 判它。
    ordinal_within_publish: u32,
    /// 记录标志那 1 字节原样（D23（journal 的角色与格式） 已定项 4 / 已定项 17）：I-8.9 判它的位 0 与其余位。
    record_flags_byte: u8,
    slot: u64,
    back_chain: u32,
    /// CRC32C(这一条的 487 字节头，`header_csum` 那 32 字节按零参与)——I-8.6（反向链算法） 的算式。
    chain_value_the_next_record_must_carry: u32,
    /// 每个点名项的两条位置条目（D23（journal 的角色与格式） 已定项 17）：I-2.5（位置条目按设备身份升序） 逐项判。
    named_entry_locations: Vec<[PointerLocation; 2]>,
    /// 点名项解出来的样子（类标签、出生树、诞生代号、key 尾段与两条位置条目）：I-8.11（点名项与被指单元头相同） 逐项判。
    named_entries: Vec<NamedEntryView>,
}

/// 一块盘的 journal 环里自证过的记录，按 jsn 计数器索引。计数器与环槽一一对应（记录 n 落 `(计数器 − 1) mod 槽数 × 4096`，
/// D23（journal 的角色与格式） 已定项 18）⇒ 同一块盘上两条自证过的记录不会撞同一个计数器。
fn scanned_journal_records_of_device(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    device: u32,
    filesystem_identifier_low: u64,
) -> BTreeMap<u64, ScannedJournalRecord> {
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let slots = reader
        .candidate_journal_slots(device)
        .unwrap_or_else(|| (0..geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES).collect());
    let mut records = BTreeMap::new();
    for slot in slots {
        let offset = geometry.journal_ring_start_slot * SLOT_BYTES + slot * JOURNAL_RECORD_BYTES;
        let Some(bytes) = reader.read(device, offset, record_bytes) else {
            continue;
        };
        let Ok(record) = check_journal_record(&bytes, filesystem_identifier_low) else {
            continue;
        };
        records.insert(
            record.counter,
            ScannedJournalRecord {
                instance: record.instance,
                checkpoint_txg: record.checkpoint_txg,
                new_root_segment: record.new_root_segment,
                transaction: record.transaction,
                commit_marker: CommitMarker::of(record.commit_marker_byte),
                ordinal_within_publish: record.ordinal_within_publish,
                record_flags_byte: record.record_flags_byte,
                slot,
                back_chain: record.back_chain,
                chain_value_the_next_record_must_carry: back_chain_of_record_header(&bytes),
                named_entry_locations: record
                    .named
                    .iter()
                    .map(|named| {
                        named.locations.map(|(named_device, named_slot, checksum)| {
                            PointerLocation {
                                device: named_device,
                                slot: named_slot,
                                checksum,
                            }
                        })
                    })
                    .collect(),
                named_entries: record.named,
            },
        );
    }
    records
}

/// I-2.5（位置条目按设备身份升序） 的「journal 点名项里的位置条目数组」那一半（代码审阅 6c 第 9 条：点名项解出来就丢了）：
/// 环里每条自证过的记录（两块盘各一份，逐盘判：一块盘上的写错了另一块不该跟着错）的每个点名项判一次。
fn judge_location_order_of_journal_named_entries(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    judgements: &mut Judgements,
) {
    for (device, records) in records_by_device {
        for (counter, record) in records {
            for (index, locations) in record.named_entry_locations.iter().enumerate() {
                judge_location_entries_order(
                    judgements,
                    locations,
                    &format!(
                        "盘 {device} journal 环槽 {} 的记录（计数器 {counter}）第 {index} 个点名项",
                        record.slot
                    ),
                );
            }
        }
    }
}

/// I-8.6（反向链算法） 对一条记录（一块盘上的那一份）该有的反向链：按「本实例内逻辑前一条」= 同一实例、计数器小 1 的那一条找。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpectedBackChain {
    /// 计数器 1（全池第一条，必是本实例第一条）、或计数器 − 1 那一条是别的实例的（这一条是本实例第一条）⇒ 0；
    /// 计数器 − 1 那一条是同一实例的 ⇒ 它的头算出来的链值。
    Value(u32),
    /// 计数器 0：计数器从 1 起（D23（journal 的角色与格式） 已定项 18「记录 n 落在 (计数器 − 1) mod 槽数」），这一格没有逻辑前一条，
    /// 链无从定义——用户 2026-09-27 定判违例，记在 I-8.6 下。
    CounterZeroHasNoLogicalPredecessor,
    /// 计数器 − 1 那一条读不出、自证不过、或环转过一圈之后那一格坐着上一圈的记录：本实例内逻辑前一条不在这块盘上，判不了。
    PredecessorNotOnThisDevice,
}

fn expected_back_chain_of(
    records: &BTreeMap<u64, ScannedJournalRecord>,
    counter: u64,
    record: &ScannedJournalRecord,
) -> ExpectedBackChain {
    match counter {
        0 => ExpectedBackChain::CounterZeroHasNoLogicalPredecessor,
        FIRST_JOURNAL_COUNTER => ExpectedBackChain::Value(0),
        _later_counter => match counter
            .checked_sub(1)
            .and_then(|previous_counter| records.get(&previous_counter))
        {
            Some(previous) if previous.instance == record.instance => {
                ExpectedBackChain::Value(previous.chain_value_the_next_record_must_carry)
            }
            Some(_record_of_another_instance) => ExpectedBackChain::Value(0),
            None => ExpectedBackChain::PredecessorNotOnThisDevice,
        },
    }
}

/// 恢复眼里的 journal 环：键是 (实例代号, 计数器)，每个键取按盘的次序（`reader.devices()`，与恢复扫环按 `device_identities()` 的次序相同）
/// 第一块盘上自证过的那一份——与实现扫环建表同一个键（`crates/singlefs-core/src/recovery.rs` 的 `scan_journal`），两块盘同一计数器坐着
/// 不同实例的记录时两条都留（I-4.1 那一行：`records_the_replay_reads` 先改了键，这两处同改，主 agent 2026-10-03 定）。I-8.6（反向链算法） 那一判逐盘判两份（一块盘上的链坏了另一块不该跟着坏）；认「恢复施加过 / 下一次挂载会施加」
/// 哪几条记录时照恢复的读法只看这一份（实审 A2a 报告「checker 要跟着改什么」第 1 条，`research/prompts/m2-rev-a2a-implementer-report.md`）。
fn records_as_the_recovery_reads_them(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
) -> BTreeMap<(u32, u64), &ScannedJournalRecord> {
    let mut first_copies: BTreeMap<(u32, u64), &ScannedJournalRecord> = BTreeMap::new();
    for (_, records) in records_by_device {
        for (counter, record) in records {
            first_copies
                .entry((record.instance, *counter))
                .or_insert(record);
        }
    }
    first_copies
}

/// I-8.6「不等的记录不进重放前缀」（代码审阅 6c 第 25 条，用户 2026-09-27 定恢复补反向链检查、checker 的前缀口径跟着对齐）：
/// 照恢复的判法（`crates/singlefs-core/src/recovery.rs` 的 `judge_back_chain_against_the_previous_record_on_disk`，实审 A2a）——
/// 本实例内逻辑前一条（同一实例、计数器小 1，在 [`records_as_the_recovery_reads_them`] 里按 (实例代号, 计数器 − 1) 找）在盘上、
/// 它的头算出来的链值与这一条的反向链不等，这一条进不了前缀。
/// 恢复只判这一种：前缀都接在所选根覆盖的那一条之后、同一实例，计数器 1、前一条是别的实例、前一条不在盘上都不判
/// （A2a 报告那一节第 2 条；I-8.6 那一判对前两种照条款判「链恒 0」，前缀这里不跟着判）。计数器 0 进不了前缀
/// （jsn 从所选根覆盖的那一条 + 1 起严格连续，计数器从 1 起，D23（journal 的角色与格式） 已定项 18）。
fn record_can_enter_a_replay_prefix(
    records_as_the_recovery_reads_them: &BTreeMap<(u32, u64), &ScannedJournalRecord>,
    counter: u64,
    record: &ScannedJournalRecord,
) -> bool {
    let Some(previous_counter) = counter.checked_sub(1) else {
        return false;
    };
    records_as_the_recovery_reads_them
        .get(&(record.instance, previous_counter))
        .is_none_or(|previous| record.back_chain == previous.chain_value_the_next_record_must_carry)
}

/// 逐盘扫一遍 journal 环，把自证过的记录按盘收齐：判 journal 的四条不变量（I-8.6（反向链算法）、
/// I-8.7（实例内事务号不重号）、I-8.8（前缀里的事务不被切开）、I-8.9（一次发布的记录序号连续且只有末条带标志））
/// 读的是同一批记录，扫一次四条一起判，不各扫一遍。
fn scanned_journal_records_by_device(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    filesystem_identifier_low: u64,
) -> Vec<(u32, BTreeMap<u64, ScannedJournalRecord>)> {
    reader
        .devices()
        .into_iter()
        .map(|device| {
            (
                device,
                scanned_journal_records_of_device(
                    reader,
                    geometry,
                    device,
                    filesystem_identifier_low,
                ),
            )
        })
        .collect()
}

/// I-8.6（反向链算法）：任一 journal 记录的反向链 = CRC32C(**本实例内逻辑前一条**记录的 487 字节头，`header_csum` 那 32 字节
/// 按零参与)；**本实例写出的第一条记录反向链恒 0**。逐盘判：环两盘互为镜像，一块盘上的链坏了另一块不该跟着坏。
/// 「本实例内逻辑前一条」= 同一实例、计数器小 1 的那一条，三种情形穷举：
/// ① 计数器 == 1 ⇒ 全池第一条，必是本实例第一条 ⇒ 反向链恒 0；
/// ② 计数器 − 1 那一条在同一块盘上自证过、实例代号相同 ⇒ 反向链 = 它的头算出来的链值；
/// ③ 计数器 − 1 那一条自证过而实例代号不同 ⇒ 这一条是本实例写出的第一条（一个实例的计数器连续、接着上一个实例走，
///    D23（journal 的角色与格式） 已定项 19 注 3）⇒ 反向链恒 0。**跨实例边界的两条记录之间不比链值**——I-8.6 明写不判那一格。
/// 计数器 − 1 那一条读不出、自证不过、或环转过一圈之后那一格坐着上一圈的记录（计数器对不上）⇒ 本实例内逻辑前一条不在盘上，
/// 这一条判不了，跳过。
fn judge_journal_back_chain(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    judgements: &mut Judgements,
) {
    let mut judged_records = 0u64;
    for (device, records) in records_by_device {
        for (counter, record) in records {
            // 计数器是盘上读来的 6 字节，自证过的记录也可以带 0（代码审阅 6c 第 6 条：减一在 0 上下溢，checker 曾当场 panic）；
            // 实审 B1 把它当判不了跳过，用户 2026-09-27 定判违例（`ExpectedBackChain::CounterZeroHasNoLogicalPredecessor`）。
            let expected_back_chain = match expected_back_chain_of(records, *counter, record) {
                ExpectedBackChain::Value(expected_back_chain) => expected_back_chain,
                ExpectedBackChain::CounterZeroHasNoLogicalPredecessor => {
                    judged_records += 1;
                    judgements.judge("I-8.6", false, || {
                        format!(
                            "盘 {device} journal 环槽 {} 的记录（实例 {}）计数器 0：计数器从 1 起，这一格没有本实例内逻辑前一条，反向链无从定义",
                            record.slot, record.instance
                        )
                    });
                    continue;
                }
                ExpectedBackChain::PredecessorNotOnThisDevice => continue,
            };
            judged_records += 1;
            judgements.judge("I-8.6", record.back_chain == expected_back_chain, || {
                format!(
                    "盘 {device} journal 环槽 {} 的记录（实例 {}、计数器 {counter}）反向链 {:#010x}，本实例内逻辑前一条的头算出来的是 {expected_back_chain:#010x}",
                    record.slot, record.instance, record.back_chain
                )
            });
        }
    }
    if judged_records == 0 {
        judgements.not_applicable(
            "I-8.6",
            "环里没有一条自证过的记录找得到本实例内逻辑前一条：链判不了",
        );
    }
}

/// 不承载事务的空发布记录在记录头上写的事务号（D23（journal 的角色与格式） 已定项 19 ①）。
const TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH: u64 = 0;

/// I-8.7（实例内事务号不重号）：**同一实例**在环里写出的记录按 jsn 排下来，一个事务号只占一段。两条判据，
/// 红了在违例说明里标出是哪一条（「判据①」「判据②」）：
/// ① 按计数器相邻的两条非 0 记录，事务号**不减**（相等或严格递增：D23（journal 的角色与格式） 已定项 7
///    「一个事务可以跨多条记录」，同一事务的记录共享事务号）；
/// ② 一个事务号的记录在计数器上**连续**：被别的记录（事务号 0 的空发布也算）隔开之后不再出现。
/// 这是已定项 7「事务号按实例计数、从 1 起」「记录按事务号顺序追加……这是实例表行的 W 能当精确前缀的依据」的投影：
/// 同一实例里号不许重（一个号不许给两个隔开的事务用）、也不许往回走。
/// 逐盘判：环两盘互为镜像，一块盘上的记录坏了另一块不该跟着坏（与 I-8.6（反向链算法） 同一个口径）。
///
/// **射程**（照判据原样，多判一格少判一格都是错）：
/// ① **跨实例不判**：事务号各实例各算各的，两个实例各有一个 1 不是违例；这一条只比同一个实例代号的记录。
/// ② **事务号 0 不进 ① 的序列**：它是不承载事务的空发布记录（已定项 19 ①），夹在同一实例两个**不同**的非 0 号中间
///    （抬回退下界那两次空发布就是这个形态）照样合法；夹在两条**同号**记录中间就是把那个号隔开了，② 红
///    （一次发布的记录不会被另一次空发布插进来）。
/// ③ 两条同号记录之间只有读不出的槽、或别的实例的记录 ⇒ 看不到本实例在那几个计数器上写过什么，② 不判红。
/// ④ 同一块盘上同一实例非 0 的记录不足两条 ⇒ 没有一对号可比，整条报「不适用」并带理由。
///
/// 判的一格是「同一块盘、同一实例、按计数器排下来相邻的两条非 0 记录」：号相等时看它们之间有没有本实例别的记录（②），
/// 号不等时要求严格变大（①，号不等时「不减」就是严格变大）；这个号以前出现过（被别的非 0 号隔开之后又出现，②）
/// 一定伴着一处 ① 红，不另判，只在正好是同一对时写进 ① 那句说明。
/// `records` 按计数器索引，计数器与 jsn 在同一实例里同序（jsn = 实例代号 32 位 + 计数器 48 位，
/// D23（journal 的角色与格式） 已定项 9）⇒ 按计数器遍历就是按 jsn 排序。
///
/// ⚠️ 它钉的是「事务号计数只活在内存里」那个前提（`crates/singlefs-core/src/transaction.rs` 的
/// `publish_overwrite` 取这个实例见过的最大事务号加一、不取上一条记录上的事务号加一）：那条链式传递成立与否
/// 压在「每条恢复路径之后都取新实例代号」上，而在这一条之前没有任何东西判得出它失效
/// （代码轮第一轮三方判决 `research/prompts/m2-checker-supp-code-r1-main-verification.md` 判定四）。
/// 两个各自提交了的事务撞号而又紧挨着时，号不减、同号相连，这一条与「一事务两条」分不开——分得开的是提交标记，
/// 归 I-8.8（前缀里的事务不被切开）。
fn judge_transaction_numbers_per_instance(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    judgements: &mut Judgements,
) {
    let mut judged_pairs = 0u64;
    for (device, records) in records_by_device {
        let mut newest_non_empty_publish_by_instance: BTreeMap<u32, (u64, u64)> = BTreeMap::new();
        // (实例代号, 事务号) → 这个号在这块盘上第一次出现的计数器。
        let mut first_counter_by_transaction: BTreeMap<(u32, u64), u64> = BTreeMap::new();
        for (counter, record) in records {
            if record.transaction == TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH {
                continue;
            }
            let first_counter_of_this_number = *first_counter_by_transaction
                .entry((record.instance, record.transaction))
                .or_insert(*counter);
            let previous = newest_non_empty_publish_by_instance
                .insert(record.instance, (*counter, record.transaction));
            let Some((previous_counter, previous_transaction)) = previous else {
                continue;
            };
            judged_pairs += 1;
            if record.transaction == previous_transaction {
                let separating_record = records
                    .range(previous_counter + 1..*counter)
                    .find(|(_, between)| between.instance == record.instance);
                judgements.judge("I-8.7", separating_record.is_none(), || {
                    let (separating_counter, separating) =
                        separating_record.expect("判红的这一支里，隔开它们的那条记录找到了");
                    format!(
                        "判据②：盘 {device} 实例 {} 事务号 {} 的两条记录（计数器 {previous_counter}、{counter}）中间坐着计数器 {separating_counter}（环槽 {}）上事务号 {} 的记录——这个号被隔开之后又出现",
                        record.instance, record.transaction, separating.slot, separating.transaction
                    )
                });
                continue;
            }
            // 号不等时，「被别的非 0 号隔开之后又出现」（②）一定伴着一处 ①：号先变走再变回来，中间必有一对是减的。
            // 所以 ② 的这一半不单独判，只在 ① 红的这一对上正好也是它时写进同一句说明。
            let reappears_after_another_number = first_counter_of_this_number != *counter;
            judgements.judge("I-8.7", record.transaction > previous_transaction, || {
                let decrease = format!(
                    "判据①：盘 {device} 实例 {} 的两条记录：计数器 {previous_counter} 上事务号 {previous_transaction}，计数器 {counter}（环槽 {}）上事务号 {}——同一实例按 jsn 排下来事务号减了",
                    record.instance, record.slot, record.transaction
                );
                if reappears_after_another_number {
                    format!(
                        "{decrease}；判据②：事务号 {} 在计数器 {first_counter_of_this_number} 出现过，被别的号隔开之后又在计数器 {counter} 出现",
                        record.transaction
                    )
                } else {
                    decrease
                }
            });
        }
    }
    if judged_pairs == 0 {
        judgements.not_applicable(
            "I-8.7",
            "环里没有哪个实例写出过两条非 0 事务号的记录：一对可比的号都凑不出来",
        );
    }
}

/// I-8.8（前缀里的事务不被切开）：同一实例在环里写出的记录，按事务看提交标记。判据各自单独判，
/// 红了在违例说明里逐条列出是哪几条（「判据③」「提交标记字节」「判据④」；① ② 是 I-8.7（实例内事务号不重号） 的）：
/// ③ 一组里带提交标记的至多一条，且它是这一组计数器最大的那一条；
/// 提交标记字节：那 1 字节只许 0 或 1，别的值判红（不许把它静默读成「不带」）；
/// ④ 一组里一条提交标记都没有时，它后面这个实例不许再有任何记录（事务号 0 的空发布也算）：
///    D23（journal 的角色与格式） 已定项 7「一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加」——
///    失败之后这个实例不再写任何记录（2026-09-23 主 agent 定案：空发布也在内）。
/// 逐盘判（与 I-8.6（反向链算法）、I-8.7 同一个口径）。一组 = 同一块盘上实例代号与事务号都相同的非 0 记录：
/// 事务号按实例计数（已定项 7），两个实例各有一个 1 是两个事务。组里的次序不在这里判——同号记录连不连续归 I-8.7 的 ②。
///
/// **射程**：
/// 一、**事务号 0 不成组**：它是不承载事务的空发布记录（D23（journal 的角色与格式） 已定项 19 ①
///    「不进『丢掉提交标记没出现的尾巴』判定」）；它自己带提交标记 1，拿它成组 ③ 当场就红。它照样受「提交标记字节」管。
/// 二、④ 看的是同一实例在这一组之后的每一条记录，不分号大号小、不分是不是事务号 0：没提交的组后面跟着一条空发布也红；
///    别的实例的记录不算（新实例接着写是失败之后该有的样子）。
/// 三、**一组里一条提交标记都没有，本身合法**：那是提交标记还没落盘的尾巴，恢复要丢掉的正是它（④ 管它是不是尾巴）。
///
/// **不适用**：同一块盘上没有哪一组两条以上（③ 没有对象）、也没有哪一组一条提交标记都没有（④ 没有对象）
/// ⇒ 整条报「不适用」，不报成立。「提交标记字节」这时照样判、判出别的值照样红，只是它单独成立不算这一条成立——
/// 第一版一事务一条、每条都带提交标记，可达镜像上都是这个形态，这一条的判别力在坏镜像上
/// （`crates/singlefs-harness/tests/checker_known_bad_images.rs`）。
fn judge_commit_markers_per_transaction(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    judgements: &mut Judgements,
) {
    let mut first_misplaced_commit_marker: Option<String> = None;
    let mut first_unrecognized_commit_marker: Option<String> = None;
    let mut first_record_after_an_uncommitted_transaction: Option<String> = None;
    let mut judged_transactions = 0u64;
    for (device, records) in records_by_device {
        for (counter, record) in records {
            match record.commit_marker {
                CommitMarker::Absent | CommitMarker::Present => {}
                CommitMarker::Unrecognized(byte) => {
                    first_unrecognized_commit_marker.get_or_insert_with(|| {
                        format!(
                            "盘 {device} 实例 {} 计数器 {counter}（环槽 {}）那条记录的提交标记字节是 {byte}——只许 0 或 1",
                            record.instance, record.slot
                        )
                    });
                }
            }
        }
        let mut records_by_transaction: BTreeMap<(u32, u64), Vec<(u64, &ScannedJournalRecord)>> =
            BTreeMap::new();
        for (counter, record) in records
            .iter()
            .filter(|(_, scanned)| scanned.transaction != TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH)
        {
            records_by_transaction
                .entry((record.instance, record.transaction))
                .or_default()
                .push((*counter, record));
        }
        for ((instance, transaction), members) in &records_by_transaction {
            let (last_counter, _) = members
                .last()
                .expect("每一组至少一条：组是拿记录一条条追加出来的");
            let member_counters: Vec<u64> = members.iter().map(|(counter, _)| *counter).collect();
            let commit_marked_counters: Vec<u64> = members
                .iter()
                .filter(|(_, member)| member.commit_marker == CommitMarker::Present)
                .map(|(counter, _)| *counter)
                .collect();
            if members.len() >= 2 {
                judged_transactions += 1;
            }
            let commit_marker_misplaced = match commit_marked_counters.as_slice() {
                [] => false,
                [only_commit_marked] => only_commit_marked != last_counter,
                [_, _, ..] => true,
            };
            if commit_marker_misplaced {
                first_misplaced_commit_marker.get_or_insert_with(|| {
                    format!(
                        "盘 {device} 实例 {instance} 事务号 {transaction} 的记录（计数器 {member_counters:?}）里带提交标记的是计数器 {commit_marked_counters:?}——至多一条、且只许是这一组计数器最大的 {last_counter}"
                    )
                });
            }
            if !commit_marked_counters.is_empty() {
                continue;
            }
            judged_transactions += 1;
            if let Some((later_counter, later)) = records
                .range(last_counter + 1..)
                .find(|(_, later)| later.instance == *instance)
            {
                first_record_after_an_uncommitted_transaction.get_or_insert_with(|| {
                    format!(
                        "盘 {device} 实例 {instance} 事务号 {transaction} 的记录（计数器 {member_counters:?}）一条提交标记都没有，后面计数器 {later_counter}（环槽 {}）上却还有同实例事务号 {} 的记录——没提交的事务之后这个实例不许再写任何记录",
                        later.slot, later.transaction
                    )
                });
            }
        }
    }
    let violated_criteria: Vec<String> = [
        ("判据③", first_misplaced_commit_marker),
        ("提交标记字节", first_unrecognized_commit_marker),
        ("判据④", first_record_after_an_uncommitted_transaction),
    ]
    .into_iter()
    .filter_map(|(criterion, first_violation)| {
        first_violation.map(|detail| format!("{criterion}：{detail}"))
    })
    .collect();
    if !violated_criteria.is_empty() {
        judgements.judge("I-8.8", false, || violated_criteria.join("；"));
    } else if judged_transactions > 0 {
        judgements.judge("I-8.8", true, String::new);
    } else {
        judgements.not_applicable(
            "I-8.8",
            "环里没有哪一组（同一块盘上实例代号与事务号都相同的非 0 记录）落了两条以上、也没有哪一组一条提交标记都没有：③ ④ 没有判的对象，提交标记字节单独成立不算这一条成立",
        );
    }
}

/// 记录标志位 0：本次发布末条（D23（journal 的角色与格式） 已定项 17）。其余位只许 0（已定项 4）。
const RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH: u8 = 0b0000_0001;

/// I-8.9 违例说明里的判据标签，次序照这里：红了在违例说明里逐条列出是哪几条（每条只留第一处）。
const PUBLISH_ORDINAL_CRITERIA: [&str; 7] = [
    "标志其余位",
    "序号 0",
    "跳号",
    "不从 1 起",
    "多于一条末条",
    "末条之后还有记录",
    "没有末条",
];

/// I-8.9（一次发布的记录序号连续且只有末条带标志）：同一实例、同一 checkpoint_txg 的记录（一次发布写出的全部记录），
/// 「本次发布内序号」依次是 1..N、与 jsn 同步，记录标志位 0 恰好只在序号 N 那一条上为 1，其余位为 0
/// （D23（journal 的角色与格式） 已定项 4 / 已定项 17）。逐盘判（与 I-8.6（反向链算法）、I-8.7、I-8.8 同一个口径）：
/// 一组 = 同一块盘上实例代号与 checkpoint_txg 都相同的自证过的记录，`records` 按计数器索引 ⇒ 组里按计数器升序。
/// 判据各自单独判，违例说明里写明红的是哪几条（标签见 `PUBLISH_ORDINAL_CRITERIA`）：
/// - 标志其余位：记录标志位 0 之外有位为 1；
/// - 序号 0；
/// - 跳号：组里两条记录的序号之差不等于计数器之差（与 jsn 同步：jsn 连号时序号也连号）；
/// - 不从 1 起：组里计数器最小那一条的前一个计数器上坐着一条自证过的、不属于这一组的记录 ⇒ 它是这次发布的第一条，序号要是 1；
/// - 多于一条末条：组里带末条标志的多于一条；
/// - 末条之后还有记录：带末条标志的那一条之后，这一组还有计数器更大的记录（它不是序号 N 那一条）；
/// - 没有末条：组里计数器最大那一条的下一个计数器上坐着**同一实例**、不属于这一组的记录（这个实例已经往下写了，
///   这次发布写完了），这一组却一条末条标志都没有。
///
/// **射程**：只判读得出的记录，读不出的那条不判（I-8.9 判据原句）——
/// 一、一组的末条可能还没落盘（崩在一次发布的记录之间）：计数器最大那一条的下一个计数器上没有自证过的记录、
///    或坐着别的实例的记录（崩溃之后新实例从读得出的最大号 + 1 接着写，D23（journal 的角色与格式） 已定项 14 注 3），
///    就不知道这一组写完没有，「没有末条」不判；
/// 二、组里夹着读不出的几条：「跳号」按序号之差与计数器之差比，隔着它们照样判得了；
/// 三、「不从 1 起」只在前一个计数器上有自证过的记录时判（环里第一条、前一条读不出，都判不了）。
///
/// **不适用**：环里一条自证过的记录都没有。
fn judge_publish_ordinals_and_last_record_flags(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    judgements: &mut Judgements,
) {
    let mut first_violation_by_criterion: BTreeMap<&'static str, String> = BTreeMap::new();
    let mut note_violation = |criterion: &'static str, detail: String| {
        first_violation_by_criterion
            .entry(criterion)
            .or_insert(detail);
    };
    let mut judged_records = 0u64;
    for (device, records) in records_by_device {
        let mut records_by_publish: BTreeMap<(u32, u64), Vec<(u64, &ScannedJournalRecord)>> =
            BTreeMap::new();
        for (counter, record) in records {
            judged_records += 1;
            if record.record_flags_byte & !RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0 {
                note_violation(
                    "标志其余位",
                    format!(
                        "盘 {device} 实例 {} 计数器 {counter}（环槽 {}）那条记录的记录标志是 {:#010b}——位 0 之外只许 0",
                        record.instance, record.slot, record.record_flags_byte
                    ),
                );
            }
            if record.ordinal_within_publish == 0 {
                note_violation(
                    "序号 0",
                    format!(
                        "盘 {device} 实例 {} 计数器 {counter}（环槽 {}）那条记录的本次发布内序号是 0——从 1 起",
                        record.instance, record.slot
                    ),
                );
            }
            records_by_publish
                .entry((record.instance, record.checkpoint_txg))
                .or_default()
                .push((*counter, record));
        }
        for ((instance, checkpoint_txg), members) in &records_by_publish {
            let member_counters: Vec<u64> = members.iter().map(|(counter, _)| *counter).collect();
            let member_ordinals: Vec<u32> = members
                .iter()
                .map(|(_, member)| member.ordinal_within_publish)
                .collect();
            let (first_counter, first_member) = *members
                .first()
                .expect("每一组至少一条：组是拿记录一条条追加出来的");
            let (last_counter, _) = *members.last().expect("同上");
            for (counter, member) in &members[1..] {
                let counter_distance = counter - first_counter;
                let ordinal_distance = i128::from(member.ordinal_within_publish)
                    - i128::from(first_member.ordinal_within_publish);
                if ordinal_distance != i128::from(counter_distance) {
                    note_violation(
                        "跳号",
                        format!(
                            "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布的记录（计数器 {member_counters:?}）序号是 {member_ordinals:?}——序号之差要等于计数器之差"
                        ),
                    );
                    break;
                }
            }
            let record_before_the_first_member = first_counter
                .checked_sub(1)
                .and_then(|previous_counter| records.get(&previous_counter));
            if record_before_the_first_member.is_some() && first_member.ordinal_within_publish != 1
            {
                note_violation(
                    "不从 1 起",
                    format!(
                        "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布的第一条（计数器 {first_counter}，前一个计数器上坐着别的发布的记录）序号是 {}——要从 1 起",
                        first_member.ordinal_within_publish
                    ),
                );
            }
            let flagged_counters: Vec<u64> = members
                .iter()
                .filter(|(_, member)| {
                    member.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0
                })
                .map(|(counter, _)| *counter)
                .collect();
            match flagged_counters.as_slice() {
                [] => {
                    let this_instance_wrote_on = records
                        .get(&(last_counter + 1))
                        .is_some_and(|next| next.instance == *instance);
                    if this_instance_wrote_on {
                        note_violation(
                            "没有末条",
                            format!(
                                "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布的记录（计数器 {member_counters:?}）一条都不带末条标志，而同一实例在计数器 {} 上已经往下写了",
                                last_counter + 1
                            ),
                        );
                    }
                }
                [only_flagged] => {
                    if *only_flagged != last_counter {
                        note_violation(
                            "末条之后还有记录",
                            format!(
                                "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布带末条标志的是计数器 {only_flagged}，这一组后面还有计数器 {member_counters:?} 里更大的记录"
                            ),
                        );
                    }
                }
                [_, _, ..] => {
                    note_violation(
                        "多于一条末条",
                        format!(
                            "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布带末条标志的有计数器 {flagged_counters:?} 这几条——只许一条"
                        ),
                    );
                }
            }
        }
    }
    let violated_criteria: Vec<String> = PUBLISH_ORDINAL_CRITERIA
        .into_iter()
        .filter_map(|criterion| {
            first_violation_by_criterion
                .get(criterion)
                .map(|detail| format!("{criterion}：{detail}"))
        })
        .collect();
    if !violated_criteria.is_empty() {
        judgements.judge("I-8.9", false, || violated_criteria.join("；"));
    } else if judged_records > 0 {
        judgements.judge("I-8.9", true, String::new);
    } else {
        judgements.not_applicable(
            "I-8.9",
            "环里一条自证过的记录都没有：没有哪一次发布的序号与末条标志可判",
        );
    }
}

/// 记录新根段七段的落点（D23（journal 的角色与格式） 已定项 4 / 已定项 15 的字段表：树表指针 86、映射根指针 86、树 ID 水位 8、F 8、
/// 实例表指针 86、分配记录树根指针 86、卸载记号 4）。
const NEW_ROOT_SEGMENT_TREE_TABLE_POINTER: std::ops::Range<usize> = 0..86;
const NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER: std::ops::Range<usize> = 86..172;
const NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK: std::ops::Range<usize> = 172..180;
const NEW_ROOT_SEGMENT_ROLLBACK_FLOOR: std::ops::Range<usize> = 180..188;
const NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER: std::ops::Range<usize> = 188..274;
const NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER: std::ops::Range<usize> = 274..360;
const NEW_ROOT_SEGMENT_UNMOUNT_MARKER: std::ops::Range<usize> = 360..364;
/// 根记录里与新根段七段同名的七段（D22（单元原子性怎么合成） 已定项 7 的偏移表：flags 20（位 0 是卸载记号）、树表指针 36、
/// 树 ID 水位 122、F 130、实例表指针 170、映射根指针 256、树表 0 条那一版的分配记录树根 342）。
const ROOT_RECORD_FLAGS: std::ops::Range<usize> = 20..24;
const ROOT_RECORD_TREE_TABLE_POINTER: std::ops::Range<usize> = 36..122;
const ROOT_RECORD_TREE_IDENTIFIER_WATERMARK: std::ops::Range<usize> = 122..130;
const ROOT_RECORD_ROLLBACK_FLOOR: std::ops::Range<usize> = 130..138;
const ROOT_RECORD_INSTANCE_TABLE_POINTER: std::ops::Range<usize> = 170..256;
const ROOT_RECORD_MAPPING_ROOT_POINTER: std::ops::Range<usize> = 256..342;

/// I-8.10 逐字节比的七对：(这一段叫什么, 新根段里的落点, 根记录里的落点)。卸载记号对的是根记录 flags 整 4 字节：
/// 两边同一个编码（位 0），根记录 flags 其余位非 0 的根自证不过、进不了这里，新根段那 4 字节位 0 之外非 0 的记录也进不了这里（RR1）。
const NEW_ROOT_SEGMENT_FIELDS_COMPARED_WITH_THE_ROOT: &[(
    &str,
    std::ops::Range<usize>,
    std::ops::Range<usize>,
)] = &[
    (
        "树表指针",
        NEW_ROOT_SEGMENT_TREE_TABLE_POINTER,
        ROOT_RECORD_TREE_TABLE_POINTER,
    ),
    (
        "映射根指针",
        NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER,
        ROOT_RECORD_MAPPING_ROOT_POINTER,
    ),
    (
        "树 ID 水位",
        NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK,
        ROOT_RECORD_TREE_IDENTIFIER_WATERMARK,
    ),
    (
        "回退下界 F",
        NEW_ROOT_SEGMENT_ROLLBACK_FLOOR,
        ROOT_RECORD_ROLLBACK_FLOOR,
    ),
    (
        "实例表指针",
        NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER,
        ROOT_RECORD_INSTANCE_TABLE_POINTER,
    ),
    (
        "分配记录树根指针",
        NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER,
        ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT,
    ),
    (
        "卸载记号",
        NEW_ROOT_SEGMENT_UNMOUNT_MARKER,
        ROOT_RECORD_FLAGS,
    ),
];
/// 根记录里实例代号 4 字节与 `checkpoint_txg` 8 字节的偏移（D22（单元原子性怎么合成） 已定项 7 的偏移表，与 `check_root_slot` 读的同一处）：
/// I-9.16 判红时拿它们说是哪一版根。
const ROOT_RECORD_INSTANCE_OFFSET: usize = 24;
const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;
/// 根记录里「树表 0 条那一版的分配记录树的根」86 在 342（D22（单元原子性怎么合成） 已定项 7 的偏移表）。
const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT: std::ops::Range<usize> = 342..428;

/// 一圈窗口：窗口 = txg > N − 3S（3S 是环长，根环槽数），锚 N 取镜像根环里择到的最新根的 txg。回收门槛、候选过滤、I-3.1（已分配统计对得上）
/// 丁-defer 门槛、抬 F 的上限等原来同取一圈窗口的六处，用户 2026-10-04 定 RGall 一起退回「环里最旧有效根」
/// （`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1），checker 里那两处不再用它。剩下的用处只有 I-7.9
/// （回退下界 F 不高于抬 F 的上限）在后来的镜像上重算上限时挡洞里留下的旧根（[`judge_rollback_floor_raises_against_their_ceilings`]）：
/// 镜像里 txg ≤ N − 3S 的根只能是洞那一槽留下的，比它新、抬 F 那一刻也在环里的根已被盖掉，拿它重算会把上限算低。
#[derive(Clone, Copy, Debug)]
struct OneRevolutionWindow {
    /// 锚 N。
    anchor_txg: u64,
    root_ring_slot_count: u64,
}

impl OneRevolutionWindow {
    /// txg 落在窗口里：txg > N − 3S，写成 txg + 3S > N（N < 3S 时整个环都在窗口里，不减出负数）。
    fn contains(self, checkpoint_txg: u64) -> bool {
        checkpoint_txg.saturating_add(self.root_ring_slot_count) > self.anchor_txg
    }
}

/// 由记录施加出来、根槽从没落盘的那一版：一条 journal 记录自带新根段（树表指针与映射根指针），恢复把它施加在所选根之上，
/// 这一版就成了现行版；之后写行那次发布把它换下的单元放进 defer——那些单元仍占着空间、仍算在「已分配」里，
/// 而根环里没有任何一条根引用它们（它自己的根槽一次都没写过）。
struct VersionAppliedOnlyByRecords {
    instance: u32,
    checkpoint_txg: u64,
    new_root_segment: Vec<u8>,
}

/// [`versions_applied_only_by_records`] 给一版取新根段时取的是哪一条记录的：带末条标志的那一条一到就换上它、之后不再换；
/// 在它之前只读到同一次发布别的记录时先占着先到的那一条。
enum SegmentTakenForTheVersion {
    FromTheLastRecordOfThePublish(Vec<u8>),
    FromTheFirstRecordReadBecauseNoLastRecordWasSeen(Vec<u8>),
}

impl VersionAppliedOnlyByRecords {
    fn tree_table_and_mapping_root_pointers(&self) -> (&[u8], &[u8]) {
        (
            &self.new_root_segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER],
            &self.new_root_segment[NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER],
        )
    }
}

/// 回退候选集要补上的那几版：**由记录施加出来、根槽从没落盘的那一版**（增补 2 收口表第 54 行，
/// 2026-09-23 用户定候选 b，`research/prompts/m2-placement-falsepositive-r1-main-verification.md` Z3）。
/// 只看盘上这一份镜像能读出的东西，四条同时成立才算一版：
///
/// ① **它被施加过**：最新那条根指着的实例表里有这个实例的行 (i, Ti, Wi)，而这条记录的 checkpoint_txg ≤ Ti。
///    行里的 Ti 就是那次恢复停在哪一版（`crates/singlefs-core/src/mount.rs` 写行时取恢复之后的有效根的 txg），
///    施加出来的每一版的 txg 都不超过它；没有这个实例的行 ⇒ 还没有哪一次恢复把它施加出来，环里的记录只是残留
///    （层 0 残留记录那条流里「链接得到残留记录」的 19 个状态就是这一格：施加要等下一次挂载，这一刻还不算）。
///    记录要带提交标记（D23（journal 的角色与格式） 已定项 7：不带的是恢复要丢的尾巴）。
/// ② **它的根槽读不出**：根环里自证过的根没有一条是 (i, 这个 txg)。读得出的那一版照常由根环那一路走。
/// ③ **不低于回退下界**：txg ≥ 最新根带的 F，与根环里的候选根同一条（D16（发布语义） 已定项 1）。
/// ④ **它换下的单元还没到回收的时候**：根环里有一条没被实例表判抛弃的根，txg 比它小（RGall：原来只看一圈窗口里的根，用户 2026-10-04 定退回整个环）。
///    理由：一个单元只被这一版引用时，它的释放代大于这一版的 txg；回收要「释放代 ≤ max(F_生效, 环里最旧有效根)」
///    （D3（空间分配） 已定项 7）。环里最旧的有效根比这一版老 ⇒ 最旧有效根 < 这一版 < 释放代，③ 又让 F ≤ 这一版 ⇒ 回收不了，
///    它们还算在已分配里；反过来环里最旧的有效根已经比这一版新，这一版里只有它引用的单元释放代都不超过那条根，已可回收复用，
///    再把它并进来就是拿已经不归它的槽去比。
///
/// 一版由那条记录的 (实例代号, checkpoint_txg) 认；两块盘各一份镜像记录，取先读到的那一份。
/// 一次发布切成几条记录时，新根段取带末条标志的那一条的（改法 FL，判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md`
/// 第二节 P1-b 与第三节第 1 条）：core 的恢复走到带末条标志的那一条才整次施加，施加的是那一条的新根段
/// （`crates/singlefs-core/src/recovery.rs` 拼新根那一段「施加末条那一份」）。合法写者每条记录都带同一份新根段（D23（journal 的角色与格式）
/// 已定项 15），两种取法同一份字节；坏写者把前几条写成别的根时，取先到的那条就是拿 core 没施加的那一版去判。
/// 环里读不到带末条标志的那一条时退回先到的那一条（这一版仍被实例表的行判为施加过，只是末条这一刻读不出）。
fn versions_applied_only_by_records(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    roots: &[(u64, u64, crate::RootView)],
    instance_table_rows: &[InstanceTableRow],
    rollback_floor: u64,
) -> Vec<VersionAppliedOnlyByRecords> {
    let is_abandoned = |instance: u32, checkpoint_txg: u64| {
        instance_table_rows
            .iter()
            .any(|row| row.instance == instance && checkpoint_txg > row.published_checkpoint_txg)
    };
    let Some(oldest_valid_root_txg) = roots
        .iter()
        .filter(|(_, _, root)| !is_abandoned(root.instance, root.checkpoint_txg))
        .map(|(_, _, root)| root.checkpoint_txg)
        .min()
    else {
        return Vec::new();
    };
    let mut versions: BTreeMap<(u32, u64), SegmentTakenForTheVersion> = BTreeMap::new();
    // 按恢复的读法（每个计数器取第一块盘上的那一份）：反向链对不上的记录不进重放前缀（I-8.6（反向链算法）），不算被哪一次恢复施加过。
    let records_as_the_recovery_reads_them = records_as_the_recovery_reads_them(records_by_device);
    for ((_, counter), record) in &records_as_the_recovery_reads_them {
        let applied_by_a_recovery = record.commit_marker == CommitMarker::Present
            && record_can_enter_a_replay_prefix(
                &records_as_the_recovery_reads_them,
                *counter,
                record,
            )
            && instance_table_rows.iter().any(|row| {
                row.instance == record.instance
                    && record.checkpoint_txg <= row.published_checkpoint_txg
            });
        let root_slot_is_readable = roots.iter().any(|(_, _, root)| {
            root.instance == record.instance && root.checkpoint_txg == record.checkpoint_txg
        });
        let at_or_above_the_floor = record.checkpoint_txg >= rollback_floor;
        let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.checkpoint_txg;
        if applied_by_a_recovery
            && !root_slot_is_readable
            && at_or_above_the_floor
            && its_units_are_not_reclaimable_yet
        {
            let carries_the_last_record_flag =
                record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0;
            let taken_so_far = versions.get(&(record.instance, record.checkpoint_txg));
            let takes_this_record = match taken_so_far {
                None => true,
                Some(SegmentTakenForTheVersion::FromTheLastRecordOfThePublish(_)) => false,
                Some(
                    SegmentTakenForTheVersion::FromTheFirstRecordReadBecauseNoLastRecordWasSeen(_),
                ) => carries_the_last_record_flag,
            };
            if takes_this_record {
                let new_root_segment = record.new_root_segment.clone();
                versions.insert(
                    (record.instance, record.checkpoint_txg),
                    if carries_the_last_record_flag {
                        SegmentTakenForTheVersion::FromTheLastRecordOfThePublish(new_root_segment)
                    } else {
                        SegmentTakenForTheVersion::FromTheFirstRecordReadBecauseNoLastRecordWasSeen(
                            new_root_segment,
                        )
                    },
                );
            }
        }
    }
    versions
        .into_iter()
        .map(
            |((instance, checkpoint_txg), taken)| VersionAppliedOnlyByRecords {
                instance,
                checkpoint_txg,
                new_root_segment: match taken {
                    SegmentTakenForTheVersion::FromTheLastRecordOfThePublish(new_root_segment)
                    | SegmentTakenForTheVersion::FromTheFirstRecordReadBecauseNoLastRecordWasSeen(
                        new_root_segment,
                    ) => new_root_segment,
                },
            },
        )
        .collect()
}

/// I-7.4（近 K 代块未被复用） 被抛弃根那一半（代码审阅 6c 第 7 条，用户 2026-09-27 定「checker 现在就补被抛弃根那一半」）：
/// I-7.4 那一行「被抛弃时间线的根……引用的块在离开根环之前同样不许重新分配、不许抹头」（C314（回退可以复用被抛弃的根引用的单元） 取影子账）。
/// 根环里每条被最新根指着的实例表判抛弃的根（有行 (i, Ti, ·) 且 T > Ti，与回退候选集同一个判法）单独走一遍：用一份自己的走读，
/// 判定丢掉、引用不进 I-3.1（已分配统计对得上） 的并集（被抛弃的根引用的单元由影子账隔离、不在当前账里），只看它引用的块读回来对不对——
/// 有一个单元校验和对不上或头用不了，就是它指着的块被重新分配或抹头了，I-7.4 按这条根判一格红，说明里带着读回来对不上的是哪些。
/// 今天崩溃恢复抛弃时间线、那次挂载看不见被抛弃的根而复用了它的槽（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）） 那一形在这里照实红。
/// 根环有读不出的槽、最新根的实例表里又有行（有根可以被判抛弃）时，那个槽里可能坐着一条被抛弃的根：这一半判不了，
/// I-7.4 没有违例时整条报「不适用」、不报成立（用户同日定「被抛弃的根读不出时报判不了、不报绿」）。
fn judge_blocks_referenced_by_abandoned_roots(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    newest_index: usize,
    instance_table_rows: &[InstanceTableRow],
    filesystem_identifier_low: u64,
    judgements: &mut Judgements,
) {
    if instance_table_rows.is_empty() {
        return;
    }
    let root_slot_bytes = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    let some_root_slot_is_unreadable = root_slot_positions(geometry)
        .into_iter()
        .any(|(_, _, device, offset)| reader.read(device, offset, root_slot_bytes).is_none());
    if some_root_slot_is_unreadable {
        judgements.part_of_the_range_not_judged(
            "I-7.4",
            "根环有读不出的槽、最新根的实例表里又有行：那个槽里可能坐着一条被抛弃的根，它引用的块有没有被重新分配或抹头判不了",
        );
    }
    let newest_root = &roots[newest_index].2;
    for (index, (_, _, root)) in roots.iter().enumerate() {
        let abandoned = instance_table_rows.iter().any(|row| {
            row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
        });
        if index == newest_index || !abandoned {
            continue;
        }
        let mut walk_of_the_abandoned_root = Walk::starting_with(
            reader,
            Judgements::default(),
            filesystem_identifier_low,
            newest_root.instance,
            newest_root.checkpoint_txg,
        );
        walk_of_the_abandoned_root.walk_root(&root.record_bytes, false);
        let first_copy_that_does_not_match = walk_of_the_abandoned_root
            .judgements
            .first_violation_of("I-2.1")
            .map(str::to_string);
        let walk_failures = walk_of_the_abandoned_root.walk_failures;
        let walked_into_a_reallocated_or_erased_unit =
            first_copy_that_does_not_match.is_some() || !walk_failures.is_empty();
        let (instance, root_txg) = (root.instance, root.checkpoint_txg);
        // 树表指针整条全零的根：走读照 core 的根记录读者读盘 0 槽 0、读不出（`follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core`），
        // 红的是这条根自己坏了，说明不写成「引用的单元已被重新分配或抹头」（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md`
        // 第二节 P2-e 与第三节第 1 条，攻方读码报的那一句）。判定不变。
        let tree_table_pointer_is_all_zero =
            parse_node_pointer(&root.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER]).all_zero;
        judgements.judge("I-7.4", !walked_into_a_reallocated_or_erased_unit, || {
            let read_back_wrong: Vec<String> = first_copy_that_does_not_match
                .into_iter()
                .chain(walk_failures)
                .collect();
            if tree_table_pointer_is_all_zero {
                format!(
                    "被抛弃的根（实例 {instance}、txg {root_txg}）走不完：根记录里树表指针全零，core 的根记录读者照它读盘 0 槽 0、读不出（这条根本身损坏，不只是它引用的单元被重新分配或抹头）：{}",
                    read_back_wrong.join("；")
                )
            } else {
                format!(
                    "被抛弃的根（实例 {instance}、txg {root_txg}）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：{}",
                    read_back_wrong.join("；")
                )
            }
        });
    }
}

/// I-8.10（记录新根段与同代根相同）：逐盘，按 (实例代号, `checkpoint_txg`) 分组，组里带末条标志（记录标志位 0）的恰一条时，
/// 它的新根段七段与根环里同 (实例代号, `checkpoint_txg`) 的每条自证过的根的同名七段逐字节相同（D23（journal 的角色与格式） 已定项 15；
/// 用户 2026-10-03 定从四段扩到七段）：新根段 [0..86) = 根 [36..122)、[86..172) = [256..342)、[172..180) = [122..130)、
/// [180..188) = [130..138)、[188..274) = [170..256)、[274..360) = [342..428)、[360..364) = flags [20..24)
/// （[`NEW_ROOT_SEGMENT_FIELDS_COMPARED_WITH_THE_ROOT`]）。
/// 末条标志不唯一的组归 I-8.9（一次发布的记录序号连续且只有末条带标志），不判；同 (实例, txg) 有两条以上不同内容的根不另分，逐条比。
fn judge_new_root_segment_of_the_last_record_against_the_roots(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    roots: &[(u64, u64, crate::RootView)],
    judgements: &mut Judgements,
) {
    let mut pairs_compared: u64 = 0;
    for (device, records) in records_by_device {
        let mut last_records_of_each_publish: BTreeMap<(u32, u64), Vec<u64>> = BTreeMap::new();
        for (counter, record) in records {
            if record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0 {
                last_records_of_each_publish
                    .entry((record.instance, record.checkpoint_txg))
                    .or_default()
                    .push(*counter);
            }
        }
        for ((instance, checkpoint_txg), counters) in last_records_of_each_publish {
            let [counter] = counters.as_slice() else {
                continue;
            };
            let segment = &records[counter].new_root_segment;
            for (region, slot, root) in roots {
                if (root.instance, root.checkpoint_txg) != (instance, checkpoint_txg) {
                    continue;
                }
                pairs_compared += 1;
                let root_bytes = &root.record_bytes;
                let differing_fields: Vec<&str> = NEW_ROOT_SEGMENT_FIELDS_COMPARED_WITH_THE_ROOT
                    .iter()
                    .filter(|(_, in_the_segment, in_the_root)| {
                        segment[in_the_segment.clone()] != root_bytes[in_the_root.clone()]
                    })
                    .map(|(name, _, _)| *name)
                    .collect();
                judgements.judge(
                    LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
                    differing_fields.is_empty(),
                    || {
                        format!(
                            "盘 {device} journal 计数器 {counter} 那条带末条标志的记录（实例 {instance}、txg {checkpoint_txg}）的新根段与区域 {region} 槽 {slot} 那条同代根这几段不逐字节相同：{}",
                            differing_fields.join("、")
                        )
                    },
                );
            }
        }
    }
    if pairs_compared == 0 {
        judgements.not_applicable(
            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
            "环里没有哪条带末条标志、同 (实例, txg) 只此一条的记录在根环里有同代的自证根：新根段没有可比的根",
        );
    }
}

/// I-8.11（点名项与被指单元头相同）：环里每条自证过的记录（逐盘）的每个点名项的每条位置条目，按类读那一份（码 2 一个节点，码 1 / 码 3
/// 一个数据单元宽；类标签不在登记表里的不知道多长，不读），整单元校验和与位置条目里的相符时，单元头里的类标签、诞生代号、写序实例代号与尾段
/// （码 1 事务号 6 字节；码 2 / 码 3 出生序号 4 + 补零 2，按 6 字节整数比同时判了补零恒 0）与点名项逐字段相同；出生树只判码 1、码 3
/// （头偏移 43），码 2 随 C289（码 2 的出生树取哪个字段没写） 不判。校验和不相符的不判：落点被合法复用之后，那里就不是它点名的单元了。
fn judge_named_entries_against_the_unit_headers(
    reader: &dyn ImageReader,
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    judgements: &mut Judgements,
) {
    let mut locations_compared: u64 = 0;
    for (device, records) in records_by_device {
        for (counter, record) in records {
            for (index, named) in record.named_entries.iter().enumerate() {
                let unit_bytes = match MappingKeyUnitClass::of(named.unit_class) {
                    MappingKeyUnitClass::DataUnit | MappingKeyUnitClass::PackedRecordUnit => {
                        data_unit_bytes()
                    }
                    MappingKeyUnitClass::IndexNode => node_bytes(),
                    MappingKeyUnitClass::Unregistered(_) => continue,
                };
                for (location_device, location_slot, checksum) in named.locations {
                    let Some(unit) =
                        reader.read(location_device, location_slot * SLOT_BYTES, unit_bytes)
                    else {
                        continue;
                    };
                    if crc32_castagnoli_table(&unit) != checksum {
                        continue;
                    }
                    locations_compared += 1;
                    let named_instance = read_u32(&named.key_tail, 0);
                    let named_tail = read_six_byte_unsigned(&named.key_tail, 4);
                    let identity_matches =
                        birth_identity_in_the_header(&unit).is_some_and(|identity| {
                            identity.write_order.birth_txg() == named.birth_txg
                                && identity.write_order.instance() == named_instance
                                && identity.tail.birth_identity_tail_number() == named_tail
                        });
                    let birth_tree_matches =
                        match birth_tree_in_the_header_as_the_mapping_key_names_it(&unit) {
                            Some(BirthTreeInTheHeader::AtOffsetFortyThree(birth_tree)) => {
                                birth_tree == named.birth_tree
                            }
                            Some(BirthTreeInTheHeader::FieldStillOpenUnderC289) | None => true,
                        };
                    let holds =
                        unit[6] == named.unit_class && identity_matches && birth_tree_matches;
                    judgements.judge(NAMED_ENTRY_MATCHES_THE_UNIT_HEADER, holds, || {
                        format!(
                            "盘 {device} journal 计数器 {counter} 那条记录第 {index} 个点名项（类 {}、出生树 {}、诞生代号 {}、实例 {named_instance}、尾段 {named_tail}）与盘 {location_device} 槽 {location_slot} 那一份单元头（类 {}）不符",
                            named.unit_class, named.birth_tree, named.birth_txg, unit[6]
                        )
                    });
                }
            }
        }
    }
    if locations_compared == 0 {
        judgements.not_applicable(
            NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
            "环里没有哪个点名项的位置条目指着一份整单元校验和与它相符的单元：点名项没有可比的单元头",
        );
    }
}

/// I-8.12（同实例记录的 txg 不减）：逐盘，同一实例代号的自证记录按 jsn 计数器从小到大，`checkpoint_txg` 不减；
/// 只比同实例的前后两条，中间夹着别的实例的记录不管；换实例之后严格大于不判（设备失而复得下没核）。
fn judge_journal_record_txg_per_instance(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    judgements: &mut Judgements,
) {
    for (device, records) in records_by_device {
        let mut previous_record_of_each_instance: BTreeMap<u32, (u64, u64)> = BTreeMap::new();
        for (counter, record) in records {
            if let Some((previous_counter, previous_txg)) = previous_record_of_each_instance
                .insert(record.instance, (*counter, record.checkpoint_txg))
            {
                judgements.judge(
                    JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE,
                    record.checkpoint_txg >= previous_txg,
                    || {
                        format!(
                            "盘 {device} 实例 {} 的 journal 记录：计数器 {previous_counter} 的 txg {previous_txg} 之后，计数器 {counter} 的 txg {} 变小了",
                            record.instance, record.checkpoint_txg
                        )
                    },
                );
            }
        }
    }
}

/// I-3.12（实例表行 T 低于引用它的根的 txg）：根环里每条自证过的根，它自己指着的实例表（沿链读，只读不判）每一行 (i, T, W) 满足
/// T < 这条根的 `checkpoint_txg`（D18（块里携带什么信息） 已定项 11、D23（journal 的角色与格式） 已定项 14）。链上某一片读不出的根不判；
/// 一行都没判到时整条报不适用。
fn judge_instance_table_rows_against_the_roots_referencing_them(
    reader: &dyn ImageReader,
    roots: &[(u64, u64, crate::RootView)],
    judgements: &mut Judgements,
) {
    let mut rows_judged: u64 = 0;
    for (_, _, root) in roots {
        let Some(rows) = instance_table_rows_of_root_without_judging(reader, &root.record_bytes)
        else {
            continue;
        };
        for row in rows {
            rows_judged += 1;
            judgements.judge(
                INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
                row.published_checkpoint_txg < root.checkpoint_txg,
                || {
                    format!(
                        "实例 {}、txg {} 的根指着的实例表里，实例 {} 那一行的 T {} 不小于这条根的 txg",
                        root.instance,
                        root.checkpoint_txg,
                        row.instance,
                        row.published_checkpoint_txg
                    )
                },
            );
        }
    }
    if rows_judged == 0 {
        judgements.not_applicable(
            INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
            "根环里没有哪条根指着一张读得出、有行的实例表",
        );
    }
}

/// [`JOURNAL_RING_GEOMETRY_FIELDS_AGREE`]（I-8.1）读 journal 环的那一格：逐盘，环里同一实例、同一 `checkpoint_txg` 的自证记录
/// （同一次发布）不多于系统配置里记的在飞记录数上限条，每个 (盘, 实例, txg) 判一格。上限取池里第一块择到系统配置的盘那一份
/// （各盘的几何本来是同一份，`check_pool_image` 走根环也只取它）。环里一条记录都没有时这一格没有对象，三个字段等式那一半照判。
fn judge_records_of_one_publish_within_the_in_flight_record_limit(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    declared_in_flight_record_limit: u64,
    judgements: &mut Judgements,
) {
    for (device, records) in records_by_device {
        let mut records_per_publish: BTreeMap<(u32, u64), u64> = BTreeMap::new();
        for record in records.values() {
            *records_per_publish
                .entry((record.instance, record.checkpoint_txg))
                .or_insert(0) += 1;
        }
        for ((instance, checkpoint_txg), record_count) in records_per_publish {
            judgements.judge(
                JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
                record_count <= declared_in_flight_record_limit,
                || {
                    format!(
                        "盘 {device} journal 环里实例 {instance}、txg {checkpoint_txg} 的自证记录 {record_count} 条，多于系统配置记的在飞上限 {declared_in_flight_record_limit}"
                    )
                },
            );
        }
    }
}

/// 恢复照 D23（journal 的角色与格式） 已定项 14 取重放前缀、施加之后的结局，checker 自己复刻一份（R-独，`research/prompts/m3-inv-a-r1-main-verification.md`
/// 第二节第 2 行）：只读，不与实现共用代码（D13（验证路线） 已定项 5）。
enum ReplayTheNextMountWouldDo<'records> {
    /// 所选根那次发布读得出的记录里带末条标志的多于一条：锚点认哪一条条款没写，实现在任何写之前拒挂（已定项 14「这一版的失败处置」）。
    AnchorUndecided,
    /// 所选根的 `checkpoint_txg` 已是 u64 的最大值，接在它后面的发布取不到 txg：实现报损坏、一条都不施加。
    ChosenRootTxgHasNoSuccessor,
    /// 前缀停在一条点名单元两份没都验过的记录上（第四条）：起点含介质损坏，用户 2026-10-01 定这种镜像不算 I-4.1 的合法镜像。
    /// 停之前已经整次收下的发布 core 照施加（`crates/singlefs-core/src/recovery.rs` 的 `replay_journal`：前缀停在验不过的那一条之前，
    /// 施加到它之前最后一个末条为止）；`last_record_of_the_first_publish_applied_before_it` 是其中第一次发布的末条，一次都没收下时是 `None`。
    StoppedAtANamedUnitThatDoesNotVerify {
        last_record_of_the_first_publish_applied_before_it: Option<&'records ScannedJournalRecord>,
    },
    /// 一次发布都没施加：所选根那一版就是下一次挂载择到的那一版，它走不走得完归 I-7.2（最新根可遍历）。
    NothingApplied,
    /// 施加了 `publishes` 次发布、共 `records` 条记录；`last_record` 是最后一次发布的末条，施加出来的那一版照它的新根段
    /// （已定项 15：一次发布的每条记录带同一份新根段）。`last_record_of_the_first_publish` 是第一次发布的末条：
    /// 下一次挂载先施加的那一版照它的新根段（[`version_the_next_mount_applies_first`]，I-3.10 与 I-5.4 读它）。
    Applied {
        publishes: u64,
        records: u64,
        last_record_counter: u64,
        last_record: &'records ScannedJournalRecord,
        last_record_of_the_first_publish: &'records ScannedJournalRecord,
    },
}

/// 恢复读者眼里的一条记录：提交标记只许 0 / 1、记录标志位 0 之外全 0、本次发布内序号不是 0——三样有一样不成，实现的记录读者当这条记录损坏、
/// 读不出（`crates/singlefs-core/src/journal.rs` 解记录头那一段；D23（journal 的角色与格式） 已定项 4 / 已定项 7 / 已定项 17）。
/// checker 的扫描（`check_journal_record`）收下它们，好让 I-8.8 / I-8.9 判红；重放这里照恢复的读法把它们当读不出。
fn record_reads_as_the_recovery_reader_reads_it(record: &ScannedJournalRecord) -> bool {
    let commit_marker_recognized = match record.commit_marker {
        CommitMarker::Absent | CommitMarker::Present => true,
        CommitMarker::Unrecognized(_) => false,
    };
    commit_marker_recognized
        && record.record_flags_byte & !RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH == 0
        && record.ordinal_within_publish != 0
}

/// 重放的在飞上限：按环长与格式常量 F 现算（D23（journal 的角色与格式） 已定项 18），I-4.1 与下一次挂载先施加的那一版用同一个数。
fn in_flight_record_limit_of_the_journal_ring(journal_ring_bytes: u64) -> u64 {
    journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR
}

/// 重放前缀读的那一份环：键是 (实例代号, 计数器)，每个键取按盘的次序第一块盘上恢复读者读得出的那一份
/// （「两份镜像任一份自证校验和过即在」，已定项 14）。与实现扫环建表同一个键（`crates/singlefs-core/src/recovery.rs` 的 `scan_journal`
/// 按 (实例代号, 计数器) 先到先得）：两块盘同一计数器坐着不同实例的记录时两条都留下，不拿先读到的那块盘上的别的实例那一条挤掉
/// 所选根那个实例的（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 8 行）。
fn records_the_replay_reads(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
) -> BTreeMap<(u32, u64), &ScannedJournalRecord> {
    let mut first_copies: BTreeMap<(u32, u64), &ScannedJournalRecord> = BTreeMap::new();
    for (_, records) in records_by_device {
        for (counter, record) in records {
            if record_reads_as_the_recovery_reader_reads_it(record) {
                first_copies
                    .entry((record.instance, *counter))
                    .or_insert(record);
            }
        }
    }
    first_copies
}

/// 重放前缀的反向链一关（I-8.6（反向链算法）「不等的记录不进重放前缀」）：本实例内逻辑前一条 = 同一实例、计数器小 1 的那一条，
/// 在 [`records_the_replay_reads`] 里按 (实例代号, 计数器) 找；在、而它的头算出来的链值与这一条的反向链不等，这一条进不了前缀；
/// 不在不判（与实现 `judge_back_chain_against_the_previous_record_on_disk` 按 (实例, 计数器 − 1) 找、找不到不判同一个读法）。
/// 计数器 0 进不了前缀（计数器从 1 起，D23（journal 的角色与格式） 已定项 18）。
fn record_back_chain_lets_it_into_the_replay_prefix(
    records_the_replay_reads: &BTreeMap<(u32, u64), &ScannedJournalRecord>,
    counter: u64,
    record: &ScannedJournalRecord,
) -> bool {
    let Some(previous_counter) = counter.checked_sub(1) else {
        return false;
    };
    records_the_replay_reads
        .get(&(record.instance, previous_counter))
        .is_none_or(|previous| record.back_chain == previous.chain_value_the_next_record_must_carry)
}

/// 一条记录的点名单元是不是每一个都两份都读得出、整单元校验和都对（前缀第四条「一个点名单元的两份都读得出、校验和都对才施加」）。
/// 类标签不在登记表里的点名项不知道该读多长：当验不过。
fn named_units_of_the_record_verify_on_both_copies(
    reader: &dyn ImageReader,
    record: &ScannedJournalRecord,
) -> bool {
    record.named_entries.iter().all(|named| {
        let unit_bytes = match MappingKeyUnitClass::of(named.unit_class) {
            MappingKeyUnitClass::DataUnit | MappingKeyUnitClass::PackedRecordUnit => {
                data_unit_bytes()
            }
            MappingKeyUnitClass::IndexNode => node_bytes(),
            MappingKeyUnitClass::Unregistered(_) => return false,
        };
        named
            .locations
            .iter()
            .all(|(location_device, location_slot, checksum)| {
                reader
                    .read(*location_device, location_slot * SLOT_BYTES, unit_bytes)
                    .is_some_and(|unit| crc32_castagnoli_table(&unit) == *checksum)
            })
    })
}

/// 带末条标志的这一条之后，同一实例里计数器更大、读得出的记录有没有同一个 `checkpoint_txg` 的（已定项 4 读者规则「带标志的那条之后还有记录」：
/// 按计数器往后找，同实例里先遇到 txg 更大的就停——同一实例 txg 随计数器不减时与全表读法同一个结果）。
fn readable_record_of_the_same_publish_follows(
    records_the_replay_reads: &BTreeMap<(u32, u64), &ScannedJournalRecord>,
    counter: u64,
    record: &ScannedJournalRecord,
) -> bool {
    for later in records_the_replay_reads
        .range((record.instance, counter.saturating_add(1))..=(record.instance, u64::MAX))
        .map(|(_, later)| *later)
    {
        if later.checkpoint_txg > record.checkpoint_txg {
            return false;
        }
        if later.checkpoint_txg == record.checkpoint_txg {
            return true;
        }
    }
    false
}

/// R-独：照 D23（journal 的角色与格式） 已定项 14 的全部规则复刻下一次挂载会施加哪一段记录。
///
/// - **锚点**（注 1，读法乙）：所选根同实例、同 `checkpoint_txg`、带末条标志的记录。恰一条 ⇒ 链首是它的计数器 + 1；
///   零条 ⇒ 链首取水位之上第一条可读记录，它要序号 1、`checkpoint_txg` = 所选根 + 1；多于一条 ⇒ 锚点认不定。
/// - 只看「水位之上」的记录：同实例、`checkpoint_txg` 大于所选根（注 1「前缀规则不跨实例边界」，注 2 按实例为主比）。
///   在飞上限按环长与格式常量 F 现算（已定项 18），逐条数到上限就停。
/// - 逐条：计数器接得上（断号即止）、反向链对得上（I-8.6 的前缀口径，[`record_back_chain_lets_it_into_the_replay_prefix`]）、一次发布的首条序号是 1、
///   发布之内 txg 不变且序号逐一递增、前一条不带提交标记时事务号不变（第三条）、末条带提交标记、带末条标志的那条之后同一次发布没有可读记录；
///   这些有一样不成就停在这一条之前。
/// - 点名单元两份都验过才收（第四条）；验不过就停，结局是「起点含介质损坏」。
/// - 第五条：前缀按发布边界截，末条没到的那次发布整体不施加。
fn replay_the_next_mount_would_do<'records>(
    reader: &dyn ImageReader,
    records_the_replay_reads: &BTreeMap<(u32, u64), &'records ScannedJournalRecord>,
    chosen_root: &crate::RootView,
    in_flight_record_limit: u64,
) -> ReplayTheNextMountWouldDo<'records> {
    let anchors: Vec<u64> = records_the_replay_reads
        .iter()
        .filter(|(_, record)| {
            record.instance == chosen_root.instance
                && record.checkpoint_txg == chosen_root.checkpoint_txg
                && record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0
        })
        .map(|((_, counter), _)| *counter)
        .collect();
    let mut expected_next_counter = match anchors.as_slice() {
        [] => None,
        [anchor] => Some(
            anchor
                .checked_add(1)
                .expect("计数器是记录头里 6 字节读来的，加一不会越过 u64"),
        ),
        [_, _, ..] => return ReplayTheNextMountWouldDo::AnchorUndecided,
    };
    let Some(chain_start_txg_without_anchor) = chosen_root.checkpoint_txg.checked_add(1) else {
        return ReplayTheNextMountWouldDo::ChosenRootTxgHasNoSuccessor;
    };
    let mut open_publish: Vec<&ScannedJournalRecord> = Vec::new();
    let mut publishes_applied: u64 = 0;
    let mut records_applied: u64 = 0;
    let mut last_applied: Option<(u64, &ScannedJournalRecord)> = None;
    let mut last_record_of_the_first_publish_applied: Option<&ScannedJournalRecord> = None;
    let above_the_chosen_root = records_the_replay_reads.iter().filter(|(_, record)| {
        record.instance == chosen_root.instance
            && record.checkpoint_txg > chosen_root.checkpoint_txg
    });
    for (records_taken_before_this_one, ((_, counter), record)) in
        (0_u64..).zip(above_the_chosen_root)
    {
        let (counter, record) = (*counter, *record);
        if records_taken_before_this_one >= in_flight_record_limit {
            break;
        }
        let continues_the_chain = match expected_next_counter {
            Some(expected) => counter == expected,
            None => {
                record.checkpoint_txg == chain_start_txg_without_anchor
                    && record.ordinal_within_publish == 1
            }
        };
        if !continues_the_chain
            || !record_back_chain_lets_it_into_the_replay_prefix(
                records_the_replay_reads,
                counter,
                record,
            )
        {
            break;
        }
        let fits_the_open_publish = match open_publish.last() {
            None => record.ordinal_within_publish == 1,
            Some(previous) => {
                record.checkpoint_txg == previous.checkpoint_txg
                    && u64::from(record.ordinal_within_publish)
                        == u64::from(previous.ordinal_within_publish) + 1
                    && (previous.commit_marker == CommitMarker::Present
                        || record.transaction == previous.transaction)
            }
        };
        if !fits_the_open_publish {
            break;
        }
        let ends_its_publish =
            record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0;
        if ends_its_publish
            && (record.commit_marker != CommitMarker::Present
                || readable_record_of_the_same_publish_follows(
                    records_the_replay_reads,
                    counter,
                    record,
                ))
        {
            break;
        }
        if !named_units_of_the_record_verify_on_both_copies(reader, record) {
            return ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify {
                last_record_of_the_first_publish_applied_before_it:
                    last_record_of_the_first_publish_applied,
            };
        }
        expected_next_counter = Some(
            counter
                .checked_add(1)
                .expect("计数器是记录头里 6 字节读来的，加一不会越过 u64"),
        );
        open_publish.push(record);
        if ends_its_publish {
            publishes_applied += 1;
            records_applied += u64::try_from(open_publish.len()).expect("一次发布的记录条数");
            // 施加出来的那一版照末条那一份新根段（已定项 15），第一次发布的那一版同样取它的末条（改法 FL 的那一取法）。
            last_record_of_the_first_publish_applied.get_or_insert(record);
            open_publish.clear();
            last_applied = Some((counter, record));
        }
    }
    match last_applied {
        None => ReplayTheNextMountWouldDo::NothingApplied,
        Some((last_record_counter, last_record)) => ReplayTheNextMountWouldDo::Applied {
            publishes: publishes_applied,
            records: records_applied,
            last_record_counter,
            last_record,
            last_record_of_the_first_publish: last_record_of_the_first_publish_applied.expect(
                "一次发布收到末条时 last_applied 与它在同一处写进 Some、之后都不回 None：last_applied 是 Some，它就是 Some",
            ),
        },
    }
}

/// [`TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE`]（I-4.1）：所选根（重放之后那一版）指向的树全可遍历。
/// 所选根取根环里 (txg, 实例) 最大的那条（与 I-7.2 同一个择法）；重放照 [`replay_the_next_mount_would_do`]；施加出来的那一版 =
/// 所选根换上末条记录的新根段七段（实例表指针与树表 0 条那一版的分配记录树的根也从新根段来，已定项 15，与实现恢复拼新根同一个读法），
/// 用一份另起的走读走一遍（判定丢掉，那些归别的不变量），走读断了（任一单元两份都读不对、头用不了、条目解不开）判红。
/// 一条都不施加、锚点认不定、所选根 txg 没有后继、前缀停在点名单元验不过（起点含介质损坏）都报不适用。
fn judge_trees_of_the_replayed_version_are_traversable(
    reader: &dyn ImageReader,
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    chosen_root: &crate::RootView,
    journal_ring_bytes: u64,
    filesystem_identifier_low: u64,
    judgements: &mut Judgements,
) {
    let records_the_replay_reads = records_the_replay_reads(records_by_device);
    match replay_the_next_mount_would_do(
        reader,
        &records_the_replay_reads,
        chosen_root,
        in_flight_record_limit_of_the_journal_ring(journal_ring_bytes),
    ) {
        ReplayTheNextMountWouldDo::AnchorUndecided => judgements.not_applicable(
            TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
            "所选根那次发布读得出的记录里带末条标志的多于一条：锚点认哪一条条款没写，实现在任何写之前拒挂，重放不出一版",
        ),
        ReplayTheNextMountWouldDo::ChosenRootTxgHasNoSuccessor => judgements.not_applicable(
            TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
            "所选根的 checkpoint_txg 是 u64 的最大值：接在它后面的发布取不到 txg，重放不出一版",
        ),
        ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify {
            last_record_of_the_first_publish_applied_before_it: _,
        } => {
            judgements.not_applicable(
                TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
                "重放前缀停在一条点名单元两份没都验过的记录上：起点含介质损坏，不在 I-4.1 判的合法镜像里",
            );
        }
        ReplayTheNextMountWouldDo::NothingApplied => judgements.not_applicable(
            TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
            "所选根之上一次发布都不施加：重放之后那一版就是所选根，它可不可遍历归 I-7.2",
        ),
        ReplayTheNextMountWouldDo::Applied {
            publishes,
            records,
            last_record_counter,
            last_record,
            last_record_of_the_first_publish: _,
        } => {
            let replayed_version = VersionAppliedOnlyByRecords {
                instance: last_record.instance,
                checkpoint_txg: last_record.checkpoint_txg,
                new_root_segment: last_record.new_root_segment.clone(),
            };
            let mut walk_of_the_replayed_version = Walk::starting_with(
                reader,
                Judgements::default(),
                filesystem_identifier_low,
                chosen_root.instance,
                chosen_root.checkpoint_txg,
            );
            walk_of_the_replayed_version.walk_version_applied_only_by_records(&replayed_version);
            let walk_failures = walk_of_the_replayed_version.walk_failures;
            judgements.judge(
                TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
                walk_failures.is_empty(),
                || {
                    format!(
                        "所选根（实例 {}、txg {}）之上重放 {publishes} 次发布、{records} 条记录（末条计数器 {last_record_counter}）施加出的那一版走不完：{}",
                        chosen_root.instance,
                        chosen_root.checkpoint_txg,
                        walk_failures.join("；")
                    )
                },
            );
        }
    }
}

/// 走读记下的被引用落点（(设备, 起点槽, 跨度) → 第一次引用它的是谁）逐盘加起来是多少槽。
/// I-3.1（已分配统计对得上） 拿走完全部候选版本之后那一份算，I-3.11（已分配减 defer 等于最新根走读） 拿只走过最新根那一刻的那一份算：
/// 两条是同一个走法、同一个加法，差别只在取哪一刻。
fn slots_referenced_per_device(
    references: &BTreeMap<(u32, u64, u64), String>,
) -> BTreeMap<u32, u64> {
    let mut slots_per_device: BTreeMap<u32, u64> = BTreeMap::new();
    for (device, _start_slot, span_slots) in references.keys() {
        *slots_per_device.entry(*device).or_insert(0) += span_slots;
    }
    slots_per_device
}

/// 删除态扫描要判的一版（I-9.5（删除队列里的号仍在树里） 那一行的共用判法：回退候选集里每条根，加由记录施加出来的那几版）：
/// 称呼（判红与判不了时写进说明）、它的 `checkpoint_txg`（「在谱系上」从这一版自己的 txg 起算，I-9.20 比死亡 txg 的上界）
/// 与它的树表指针（根环里的根取根记录偏移 36 起的 86 字节，由记录施加出来的那一版取记录新根段里的那一条）。
struct VersionToScanTheDeletionStateOf<'record> {
    name: String,
    checkpoint_txg: u64,
    tree_table_pointer: &'record [u8],
}

/// 删除态扫描在一版上读出的一条 inode 记录里要的三段（D8（核心索引结构） 已定项 6 字段表：对象出生代偏移 8、nlink 偏移 36、size 偏移 40）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InodeRecordOfTheDeletionScan {
    object_birth: u64,
    nlink: u32,
    size_in_bytes: u64,
}

/// 删除态扫描在一版上读不全、配不上的是哪一样：那一版在射程里判不了，报 `part_of_the_range_not_judged`、不判红
/// （判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 F4-b、F4-c）。读不出、解不开、形状不对由主走读判
/// （I-2.1、I-1.1、I-1.7、I-1.10、I-9.1、I-9.27 等），这一遍只如实记「判不了」，不进 `walk_failures`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeletionScanGap {
    TreeTablePointerAllZero,
    TreeTableUnreadable,
    TreeTableEntryShorterThanTheFieldTable,
    DeletedInodesHeadIdentifierNamesNoSingleInodeTree,
    InodeTreeNamedByNoSingleDeletedInodesEntry,
    ExtentTreeHeadIdentifierNamesNoSingleInodeTree,
    DeletedInodesRootUnreadable,
    InodeTreeUnreadable,
    ExtentTreeOfTheInodeTreeNotSingle,
    ExtentTreeUnreadable,
    AccountingTreeUnreadable,
    TombstoneTreeUnreadable,
    TombstoneTreeOfTheVersionUnreadable,
    TombstoneTreeHeadIdentifierNamesNoSingleInodeTree,
    InodeTreeNamedByNoSingleTombstoneTreeEntry,
    LineageTreeEntryNotSingle,
    LineageTreeUnreadable,
    LineageRecordsInconsistent,
    QueuedInodeWithoutARecord,
    RetirementTreeUnreadable,
    RetirementTreeHeadIdentifierNamesNoSingleInodeTree,
    ParentVersionNotSingle,
}

impl DeletionScanGap {
    fn reason(self) -> &'static str {
        match self {
            DeletionScanGap::TreeTablePointerAllZero => {
                "走读到的某一版树表指针全零（core 照它读盘 0 槽 0、读不出）：那一版的删除态射程里判不了"
            }
            DeletionScanGap::TreeTableUnreadable => {
                "走读到的某一版树表读不出（两份都对不上整单元校验和或解不开）：那一版的删除态射程里判不了"
            }
            DeletionScanGap::TreeTableEntryShorterThanTheFieldTable => {
                "走读到的某一版树表条目宽不到字段表的 200 字节：那一版的删除态射程里判不了"
            }
            DeletionScanGap::DeletedInodesHeadIdentifierNamesNoSingleInodeTree => {
                "走读到的某一版 deleted_inodes 条目偏移 116 的头 ID 配不上同一张树表里恰好一条 inode 树条目：那一版判不了"
            }
            DeletionScanGap::InodeTreeNamedByNoSingleDeletedInodesEntry => {
                "走读到的某一版树表里有一棵 inode 树，同一张表里偏移 116 的头 ID 指着它的 deleted_inodes 条目不是恰好一条：那一版这棵树的队列与记录判不了"
            }
            DeletionScanGap::ExtentTreeHeadIdentifierNamesNoSingleInodeTree => {
                "走读到的某一版树表里有一棵 extent 树，偏移 116 的头 ID 配不上同一张表里恰好一条 inode 树条目：那一版的数据单元数判不了"
            }
            DeletionScanGap::DeletedInodesRootUnreadable => {
                "走读到的某一版 deleted_inodes 根读不出，或不是层级 0、key 宽与条目宽 8 的码 2 根：那一版的队列判不了"
            }
            DeletionScanGap::InodeTreeUnreadable => {
                "走读到的某一版 inode 树读不全（节点或叶容器读不出、层级或条目宽不对、类型段不认得）：那一版的记录判不了"
            }
            DeletionScanGap::ExtentTreeOfTheInodeTreeNotSingle => {
                "走读到的某一版树表里头 ID 指着这棵 inode 树的 extent 树条目不是恰好一条：那一版的数据单元数判不了"
            }
            DeletionScanGap::ExtentTreeUnreadable => {
                "走读到的某一版 extent 树读不全：那一版每个号的数据单元数判不了"
            }
            DeletionScanGap::AccountingTreeUnreadable => {
                "走读到的某一版记账树读不全（节点读不出、层级或条目宽不对）：那一版的记账行判不了"
            }
            DeletionScanGap::TombstoneTreeUnreadable => {
                "所选根的树表或墓碑树读不全：水位大于墓碑对象 ID 那一半判不了"
            }
            DeletionScanGap::TombstoneTreeOfTheVersionUnreadable => {
                "走读到的某一版墓碑树读不全（节点或容器读不出、层级或条目宽不对、类型段不认得、容器的打包记录类型或记录宽不是登记的）：那一版的墓碑判不了"
            }
            DeletionScanGap::TombstoneTreeHeadIdentifierNamesNoSingleInodeTree => {
                "走读到的某一版墓碑树条目偏移 116 的头 ID 配不上同一张树表里恰好一条 inode 树条目：那一版墓碑对 inode 记录的那几判判不了"
            }
            DeletionScanGap::InodeTreeNamedByNoSingleTombstoneTreeEntry => {
                "走读到的某一版树表里有一棵 inode 树，同一张表里偏移 116 的头 ID 指着它的墓碑树条目不是恰好一条：那一版这棵树的队列号判不了"
            }
            DeletionScanGap::LineageTreeEntryNotSingle => {
                "走读到的某一版树表里谱系树条目不是恰好一条：那一版的「在谱系上」判不了"
            }
            DeletionScanGap::LineageTreeUnreadable => {
                "走读到的某一版谱系树读不全（节点或容器读不出、层级或条目宽不对、类型段不认得、容器的打包记录类型或记录宽不是登记的）：那一版的「在谱系上」判不了"
            }
            DeletionScanGap::LineageRecordsInconsistent => {
                "走读到的某一版谱系记录不合法（有一条 T_old 不小于 T_rb，或同一个 T_rb 有两条）：那一版的父链接不上，「在谱系上」判不了"
            }
            DeletionScanGap::QueuedInodeWithoutARecord => {
                "走读到的某一版 deleted_inodes 里有一个号在同一版 inode 树里没有记录：那个号要的对象出生代没有，它的谱系上墓碑判不了（这一形由 I-9.5 判红）"
            }
            DeletionScanGap::RetirementTreeUnreadable => {
                "走读到的某一版退役树读不全（节点或容器读不出、层级或条目宽不对、类型段不认得、容器的打包记录类型或记录宽不是登记的）：那一版的退役记录判不了"
            }
            DeletionScanGap::RetirementTreeHeadIdentifierNamesNoSingleInodeTree => {
                "走读到的某一版退役树条目偏移 116 的头 ID 配不上同一张树表里恰好一条 inode 树条目：那一版退役记录对叶容器的那两判判不了"
            }
            DeletionScanGap::ParentVersionNotSingle => {
                "走读到的某一版的父版（那个 txg）在回退候选集里不止一版、树表不同：拿哪一版的叶身份集比判不了"
            }
        }
    }
}

/// 删除态扫描读 I-9.5、I-4.7、I-4.6 三条要的那一组：树表、`deleted_inodes` 与 inode 树那几样一版里任一样判不了（树表读不出、条目窄、
/// `deleted_inodes` 与 inode 树两个方向配不上、队列或记录读不出），三条在那一版都判不了；extent 树那一侧的三种（extent 树指着这棵 inode 树的
/// 不是恰好一条、extent 树读不全、extent 树的头 ID 配不上 inode 树）只记 I-4.7、I-4.6，I-9.5 不读 extent 树、照判
/// （`research/prompts/m3-batch21-code-r1-main-verification.md` 第二节 K1 那一行）。
const DELETION_QUEUE_SCAN_INVARIANTS: [&str; 3] = [
    DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
];

/// 删除态扫描读过的码 3 单元：解开的容器与头里写序的实例代号（I-9.6 墓碑那一半按已发布谓词滤容器要它）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct PackedUnitOfTheDeletionScan {
    view: crate::PackedUnitView,
    write_order_instance: u32,
}

/// 删除态扫描里读过的码 3 单元：(设备身份, 起点槽, 位置条目里的整单元校验和) → 解开的单元（解不开记 None），同 [`IndexNodeCache`]：
/// 候选集里几版常共用同一片 inode 叶容器，按位置条目记一份，省掉重复的整单元 CRC。
type PackedUnitCache = BTreeMap<(u32, u64, u32), Option<PackedUnitOfTheDeletionScan>>;

fn read_packed_unit_without_judging(
    reader: &dyn ImageReader,
    pointer: &PointerView,
    cache: &mut PackedUnitCache,
) -> Option<PackedUnitOfTheDeletionScan> {
    if pointer.all_zero {
        return None;
    }
    let key = (
        pointer.locations[0].device,
        pointer.locations[0].slot,
        pointer.locations[0].checksum,
    );
    if let Some(cached) = cache.get(&key) {
        return cached.clone();
    }
    let parsed = read_unit_without_judging(reader, pointer, data_unit_bytes()).and_then(|unit| {
        packed_unit_view(&unit)
            .ok()
            .map(|view| PackedUnitOfTheDeletionScan {
                view,
                write_order_instance: read_u32(&unit, PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET),
            })
    });
    cache.insert(key, parsed.clone());
    parsed
}

/// 一版的 `deleted_inodes` 号集，只读不判（D8（核心索引结构） 已定项 16：码 2、层级 0 的一个根，key = inode 号 8 字节、无 value；
/// 队列空时树表条目的根指针置零）。
fn deleted_inodes_queue_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    cache: &mut IndexNodeCache,
) -> Result<BTreeSet<u64>, DeletionScanGap> {
    if root_pointer.all_zero {
        return Ok(BTreeSet::new());
    }
    let root = read_index_node_without_judging(reader, root_pointer, cache)
        .ok_or(DeletionScanGap::DeletedInodesRootUnreadable)?;
    if root.level != 0
        || root.key_width != DELETED_INODES_ENTRY_BYTES
        || root.entry_width < DELETED_INODES_ENTRY_BYTES
    {
        return Err(DeletionScanGap::DeletedInodesRootUnreadable);
    }
    Ok(root
        .entries
        .iter()
        .map(|entry| read_u64(entry, 0))
        .collect())
}

/// 一版 inode 树的全部记录（号 → nlink 与 size），只读不判：根层级 0 是零条目根（一条记录都没有，D8（核心索引结构） 已定项 6、16 第 5 条），
/// 层级 ≥ 1 的往下走（[`collect_inode_records_below`]）。
fn inode_records_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
) -> Result<BTreeMap<u64, InodeRecordOfTheDeletionScan>, DeletionScanGap> {
    let mut records = BTreeMap::new();
    if root_pointer.all_zero {
        return Ok(records);
    }
    let root = read_index_node_without_judging(reader, root_pointer, index_cache)
        .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
    if root.level == 0 {
        return if root.entries.is_empty() {
            Ok(records)
        } else {
            Err(DeletionScanGap::InodeTreeUnreadable)
        };
    }
    collect_inode_records_below(reader, &root, index_cache, packed_cache, &mut records)?;
    Ok(records)
}

/// inode 树一个内部节点下面的记录逐条收进 `records`：类型段 2 的叶容器（码 3 打包记录类型 2，记录宽 140）读它的每条记录，
/// 类型段 0 的内部节点（层级 = 父层级 − 1，且 ≥ 1）往下走。读不出、形状不对的交回 [`DeletionScanGap::InodeTreeUnreadable`]。
/// 同一个号出现两条时后读到的那条盖掉先读到的：号的唯一性由主走读的 I-9.4 判——同一片叶里同号两条、乱序由第四句（同一片容器里
/// 记录号沿记录序严格递增，判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T11 取甲），跨叶的由第二、三句
/// （沿叶序容器号严格递增、左容器最大 key < 右容器号），这一遍只读不判。递归的上界：每下一层层级减一。
fn collect_inode_records_below(
    reader: &dyn ImageReader,
    node: &crate::IndexNodeView,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
    records: &mut BTreeMap<u64, InodeRecordOfTheDeletionScan>,
) -> Result<(), DeletionScanGap> {
    if node.entry_width < inode_internal_entry_bytes() {
        return Err(DeletionScanGap::InodeTreeUnreadable);
    }
    let inode_record_bytes =
        registered_record_width(PACKED_TYPE_INODE).expect("类型 2 登记了记录宽 140");
    for entry in &node.entries {
        let child_pointer = parse_node_pointer(&entry[34..120]);
        match InodeInternalEntryType::of(read_u16(entry, 16)) {
            InodeInternalEntryType::LeafContainer => {
                let container =
                    read_packed_unit_without_judging(reader, &child_pointer, packed_cache)
                        .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
                if container.view.record_type != PACKED_TYPE_INODE
                    || container.view.record_width < inode_record_bytes
                {
                    return Err(DeletionScanGap::InodeTreeUnreadable);
                }
                for record in &container.view.records {
                    records.insert(
                        read_u64(record, 0),
                        InodeRecordOfTheDeletionScan {
                            object_birth: read_u64(record, INODE_RECORD_OBJECT_BIRTH_OFFSET),
                            nlink: read_u32(record, INODE_RECORD_NLINK_OFFSET),
                            size_in_bytes: read_u64(record, INODE_RECORD_SIZE_OFFSET),
                        },
                    );
                }
            }
            InodeInternalEntryType::InternalNode => {
                if child_pointer.all_zero {
                    return Err(DeletionScanGap::InodeTreeUnreadable);
                }
                let child = read_index_node_without_judging(reader, &child_pointer, index_cache)
                    .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
                if node.level.checked_sub(1) != Some(child.level) || child.level == 0 {
                    return Err(DeletionScanGap::InodeTreeUnreadable);
                }
                collect_inode_records_below(reader, &child, index_cache, packed_cache, records)?;
            }
            InodeInternalEntryType::Unregistered(_) => {
                return Err(DeletionScanGap::InodeTreeUnreadable);
            }
        }
    }
    Ok(())
}

/// 一版 extent 树里每个 inode 号的数据单元数，只读不判：上段叶条目内联的那一个数一个，下段叶的每条 extent 叶记录数一个；
/// 全零的数据指针不指单元，不数。根指针全零的树一个单元都没有。
fn data_unit_count_of_each_inode_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    cache: &mut IndexNodeCache,
) -> Result<BTreeMap<u64, u64>, DeletionScanGap> {
    let mut data_units_of_each_inode: BTreeMap<u64, u64> = BTreeMap::new();
    if root_pointer.all_zero {
        return Ok(data_units_of_each_inode);
    }
    let extent_tree = extent_tree_with_the_inode_of_each_data_pointer_without_judging(
        reader,
        root_pointer,
        cache,
    )
    .ok_or(DeletionScanGap::ExtentTreeUnreadable)?;
    for (inode, _) in extent_tree
        .data_pointers_of_inodes
        .iter()
        .filter(|(_, data_pointer)| !data_pointer.all_zero)
    {
        *data_units_of_each_inode.entry(*inode).or_insert(0) += 1;
    }
    Ok(data_units_of_each_inode)
}

/// 一版记账树的全部行，只读不判（D8（核心索引结构） 已定项 11：多层码 2 树，叶条目 34、内部条目 = key 22 + 子指针 86）：
/// 走全了才交回，有一个节点读不出、key 宽或条目宽不对、孩子层级不是父层级减一都交回 [`DeletionScanGap::AccountingTreeUnreadable`]。
/// 递归的上界：每下一层层级减一。
fn collect_accounting_rows_without_judging(
    reader: &dyn ImageReader,
    node_pointer: &PointerView,
    cache: &mut IndexNodeCache,
    rows: &mut Vec<Vec<u8>>,
) -> Result<(), DeletionScanGap> {
    let node = read_index_node_without_judging(reader, node_pointer, cache)
        .ok_or(DeletionScanGap::AccountingTreeUnreadable)?;
    let key_width = KEY_SCHEMA_ACCOUNTING.width();
    if node.key_width != key_width {
        return Err(DeletionScanGap::AccountingTreeUnreadable);
    }
    if node.level == 0 {
        if node.entry_width < accounting_entry_bytes() {
            return Err(DeletionScanGap::AccountingTreeUnreadable);
        }
        rows.extend(node.entries.iter().cloned());
        return Ok(());
    }
    if node.entry_width < internal_entry_bytes(KEY_SCHEMA_ACCOUNTING) {
        return Err(DeletionScanGap::AccountingTreeUnreadable);
    }
    for entry in &node.entries {
        let child_pointer = parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]);
        if child_pointer.all_zero {
            return Err(DeletionScanGap::AccountingTreeUnreadable);
        }
        let child_level = read_index_node_without_judging(reader, &child_pointer, cache)
            .ok_or(DeletionScanGap::AccountingTreeUnreadable)?
            .level;
        if node.level.checked_sub(1) != Some(child_level) {
            return Err(DeletionScanGap::AccountingTreeUnreadable);
        }
        collect_accounting_rows_without_judging(reader, &child_pointer, cache, rows)?;
    }
    Ok(())
}

/// 删除态扫描（判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 F4、第七节骨架）：要判的每一版（回退候选集里每条根，
/// 加由记录施加出来的那几版；树表单元相同的几版内容相同，只扫一次）各自重读一遍树表与几棵树，只读、不进 `walk_failures`，
/// 判 I-3.4（[`judge_pending_deletion_occupancy_of_a_version`]）与 I-9.5、I-4.7、I-4.6（[`judge_deletion_queue_of_a_version`]）。
/// 主走读按 `visited_units` 只走一遍共享子树，这几条要的是「这一版自己的」队列、记录与单元数，所以另走一遍（与 [`references_of_root`] 同一个缘故）。
/// 树表 0 条的那一版没有这几棵树，不判；树表指针全零的那一版照 core 的根记录读者读盘 0 槽 0、读不出，报射程里判不了
/// （用户 2026-10-05 定甲，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第五节第 1 条；此前同样算没有这几棵树）。
fn judge_deletion_state_of_the_versions(
    reader: &dyn ImageReader,
    versions: &[VersionToScanTheDeletionStateOf<'_>],
    index_cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut packed_cache = PackedUnitCache::new();
    let mut tree_tables_scanned: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
    for version in versions {
        let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
        if tree_table_pointer.all_zero {
            for invariant in DELETION_QUEUE_SCAN_INVARIANTS.into_iter().chain([
                PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
                DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
            ]) {
                judgements.part_of_the_range_not_judged(
                    invariant,
                    DeletionScanGap::TreeTablePointerAllZero.reason(),
                );
            }
            continue;
        }
        let first_location = tree_table_pointer.locations[0];
        if !tree_tables_scanned.insert((
            first_location.device,
            first_location.slot,
            first_location.checksum,
        )) {
            continue;
        }
        let gap = match read_index_node_without_judging(reader, &tree_table_pointer, index_cache) {
            None => Some(DeletionScanGap::TreeTableUnreadable),
            Some(tree_table) if tree_table.entries.is_empty() => None,
            Some(tree_table)
                if tree_table
                    .entries
                    .iter()
                    .any(|entry| entry.len() < tree_table_entry_bytes()) =>
            {
                Some(DeletionScanGap::TreeTableEntryShorterThanTheFieldTable)
            }
            Some(tree_table) => {
                judge_pending_deletion_occupancy_of_a_version(
                    reader,
                    &version.name,
                    &tree_table.entries,
                    index_cache,
                    judgements,
                );
                judge_deletion_queue_of_a_version(
                    reader,
                    &version.name,
                    &tree_table.entries,
                    index_cache,
                    &mut packed_cache,
                    judgements,
                );
                None
            }
        };
        if let Some(gap) = gap {
            for invariant in DELETION_QUEUE_SCAN_INVARIANTS.into_iter().chain([
                PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
                DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
            ]) {
                judgements.part_of_the_range_not_judged(invariant, gap.reason());
            }
        }
    }
    // 一次都没判到时的理由（判到过的照判定报，`Judgements::not_applicable` 不改已经评估过的）。
    judgements.not_applicable(
        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
        "走读到的每一版树表里都没有 deleted_inodes 条目（树表 0 条）：没有队列可比",
    );
    judgements.not_applicable(
        PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
        "走读到的每一版树表里都没有记账树（树表 0 条）",
    );
    judgements.not_applicable(
        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
        "走读到的每一版里都没有处于删除中的 inode（nlink = 0 的记录、有数据单元而没有 nlink ≥ 1 记录的号）",
    );
    judgements.not_applicable(
        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
        "走读到的每一版 inode 树里都没有 nlink = 0 的记录",
    );
    judgements.not_applicable(
        DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
        "走读到的每一版树表里都没有 deleted_inodes 条目（树表 0 条）：没有队列可判",
    );
}

/// I-3.4（可用空间扣待删占用） 只落「记账标签 4 恒 0」（判决第二节 F5 三格）：这一版树表里每一棵记账树（种类 4）走全了，
/// 要有 (4, 树, 0xFFFF_FFFF, 代) 那一行（缺行判红），每一行标签 4 的值都是 0；读不全的报射程里判不了。根指针全零的不判（没有记账树）。
fn judge_pending_deletion_occupancy_of_a_version(
    reader: &dyn ImageReader,
    version_name: &str,
    tree_table_entries: &[Vec<u8>],
    index_cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let invariant = PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO;
    for accounting_entry in tree_table_entries
        .iter()
        .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_ACCOUNTING)
    {
        let tree = read_u64(accounting_entry, 0);
        let root_pointer = parse_node_pointer(&accounting_entry[14..100]);
        if root_pointer.all_zero {
            continue;
        }
        let mut rows = Vec::new();
        if let Err(gap) =
            collect_accounting_rows_without_judging(reader, &root_pointer, index_cache, &mut rows)
        {
            judgements.part_of_the_range_not_judged(invariant, gap.reason());
            continue;
        }
        let pending_deletion_rows: Vec<&Vec<u8>> = rows
            .iter()
            .filter(|row| read_u16(row, 0) == STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES)
            .collect();
        let has_the_pending_deletion_row = pending_deletion_rows.iter().any(|row| {
            read_u32(row, ACCOUNTING_ROW_DEVICE_OFFSET) == STATISTIC_NO_DEVICE_DIMENSION
        });
        judgements.judge(invariant, has_the_pending_deletion_row, || {
            format!(
                "{version_name}的记账树（树 {tree}）走全了，没有待删除但意图未完成占用（标签 {STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES}、设备段 {STATISTIC_NO_DEVICE_DIMENSION:#x}）那一行"
            )
        });
        for row in pending_deletion_rows {
            let value = read_u64(row, ACCOUNTING_ROW_VALUE_OFFSET);
            judgements.judge(invariant, value == 0, || {
                format!(
                    "{version_name}的记账树（树 {tree}）里待删除但意图未完成占用（标签 {STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES}、设备段 {:#x}）那一行的值是 {value}，不是 0",
                    read_u32(row, ACCOUNTING_ROW_DEVICE_OFFSET)
                )
            });
        }
    }
}

/// I-9.5（两个方向）、I-4.7（对象取丙）、I-4.6（取乙）在一版上判：这一版树表里每一条 `deleted_inodes` 条目经偏移 116 的头 ID 配同一张表里
/// 树 ID 等于它的那一条 inode 树条目（判决第二节 F4-b；配不上、配到不止一条记判不了），extent 树同样按头 ID 配这棵 inode 树；
/// 反方向也配：每一棵 inode 树都要恰好一条 `deleted_inodes` 条目指着它，每一棵 extent 树的头 ID 都要配上恰好一条 inode 树，
/// 不是恰好一条的记判不了。
/// I-9.5 每一版每个方向各判一次（队列与 nlink = 0 的号集合两两比，干净的一版也真比过）；I-4.7 每个处于删除中的号判一次、
/// I-4.6 每条 nlink = 0 的记录判一次（一个都没有时没有对象）。
/// I-9.24（删除队列至多一条 key）：每一条 `deleted_inodes` 条目都判，读队列在配对之前，配不配得上 inode 树都判；空队列报成立
/// （判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T8 取 a）。
fn judge_deletion_queue_of_a_version(
    reader: &dyn ImageReader,
    version_name: &str,
    tree_table_entries: &[Vec<u8>],
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
    judgements: &mut Judgements,
) {
    let kind_of = |entry: &&Vec<u8>| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET);
    let head_identifier_of =
        |entry: &&Vec<u8>| read_u64(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
    for deleted_inodes_entry in tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_DELETED_INODES)
    {
        let inode_tree = head_identifier_of(&deleted_inodes_entry);
        let queue_read_back = deleted_inodes_queue_without_judging(
            reader,
            &parse_node_pointer(&deleted_inodes_entry[14..100]),
            index_cache,
        );
        match &queue_read_back {
            Ok(queue) => {
                judgements.judge(
                    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
                    queue.len() <= 1,
                    || {
                        format!(
                        "{version_name}：deleted_inodes（树 {}）里有 {} 条 key {queue:?}，多于一条",
                        read_u64(deleted_inodes_entry, 0),
                        queue.len()
                    )
                    },
                );
            }
            Err(gap) => {
                judgements.part_of_the_range_not_judged(
                    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
                    gap.reason(),
                );
            }
        }
        let inode_tree_entries: Vec<&Vec<u8>> = tree_table_entries
            .iter()
            .filter(|entry| kind_of(entry) == TREE_KIND_INODE && read_u64(entry, 0) == inode_tree)
            .collect();
        let read_back = match inode_tree_entries.as_slice() {
            [inode_tree_entry] => queue_read_back.and_then(|queue| {
                inode_records_without_judging(
                    reader,
                    &parse_node_pointer(&inode_tree_entry[14..100]),
                    index_cache,
                    packed_cache,
                )
                .map(|records| (queue, records))
            }),
            [] | [_, _, ..] => {
                Err(DeletionScanGap::DeletedInodesHeadIdentifierNamesNoSingleInodeTree)
            }
        };
        let (queue, records) = match read_back {
            Ok(queue_and_records) => queue_and_records,
            Err(gap) => {
                for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
                    judgements.part_of_the_range_not_judged(invariant, gap.reason());
                }
                continue;
            }
        };
        judge_the_deletion_queue_against_the_inode_records(
            version_name,
            inode_tree,
            &queue,
            &records,
            judgements,
        );
        let extent_tree_entries: Vec<&Vec<u8>> = tree_table_entries
            .iter()
            .filter(|entry| {
                kind_of(entry) == TREE_KIND_EXTENT && head_identifier_of(entry) == inode_tree
            })
            .collect();
        let data_units_of_each_inode = match extent_tree_entries.as_slice() {
            [extent_tree_entry] => data_unit_count_of_each_inode_without_judging(
                reader,
                &parse_node_pointer(&extent_tree_entry[14..100]),
                index_cache,
            ),
            [] | [_, _, ..] => Err(DeletionScanGap::ExtentTreeOfTheInodeTreeNotSingle),
        };
        match data_units_of_each_inode {
            Ok(data_units_of_each_inode) => {
                judge_inodes_being_deleted_against_the_queue_and_their_data_units(
                    version_name,
                    &queue,
                    &records,
                    &data_units_of_each_inode,
                    judgements,
                );
                judge_data_unit_counts_of_the_inodes_being_deleted_against_their_size(
                    version_name,
                    &records,
                    &data_units_of_each_inode,
                    judgements,
                );
            }
            Err(gap) => {
                for invariant in [
                    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
                    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
                ] {
                    judgements.part_of_the_range_not_judged(invariant, gap.reason());
                }
            }
        }
    }
    // 反方向也配（代码轮判决 `research/prompts/m3-batch18-code-r1-main-verification.md` 第二节「K2 漏判 L1」，改法 A）：上面那一遍只从
    // deleted_inodes 条目一侧去配，树表里没有指着某棵 inode 树的 deleted_inodes 条目时那棵树一次都不判、也不记判不了。这里每一棵 inode 树
    // 都要恰好一条 deleted_inodes 条目的头 ID 指着它，不是恰好一条的记判不了（判决 F4-b「配不上记判不了」）。放在上面那一遍之后：
    // deleted_inodes 头 ID 配不上 inode 树的那一形同时也是这一形，先记上面那一句更近的理由。
    for inode_tree_entry in tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_INODE)
    {
        let inode_tree = read_u64(inode_tree_entry, 0);
        let deleted_inodes_entries_naming_the_inode_tree = tree_table_entries
            .iter()
            .filter(|entry| {
                kind_of(entry) == TREE_KIND_DELETED_INODES
                    && head_identifier_of(entry) == inode_tree
            })
            .count();
        if deleted_inodes_entries_naming_the_inode_tree != 1 {
            for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
                judgements.part_of_the_range_not_judged(
                    invariant,
                    DeletionScanGap::InodeTreeNamedByNoSingleDeletedInodesEntry.reason(),
                );
            }
        }
    }
    // extent 树那一侧也反方向配（代码轮判决 `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节 S1、第三节第 1 条）：
    // 上面那一遍只读头 ID 等于这棵 inode 树的 extent 树，头 ID 配不上 inode 树的 extent 树一次都不读、也不记判不了——一条空根条目
    // 冒充这棵 inode 树的 extent 树、真 extent 树头 ID 指别处时，I-4.7、I-4.6 按零个单元判、报没有对象。这里每一棵 extent 树的头 ID
    // 都要配上恰好一条 inode 树，配不上的记判不了（设计轮判决 F4-b「配不上记判不了」）。放在上面那一遍之后：没有 extent 树指着
    // inode 树的那一形同时也是这一形，先记上面那一句更近的理由。I-9.5 不读 extent 树，不记。
    for extent_tree_entry in tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_EXTENT)
    {
        let inode_tree = head_identifier_of(&extent_tree_entry);
        let inode_trees_named_by_the_extent_tree = tree_table_entries
            .iter()
            .filter(|entry| kind_of(entry) == TREE_KIND_INODE && read_u64(entry, 0) == inode_tree)
            .count();
        if inode_trees_named_by_the_extent_tree != 1 {
            for invariant in [
                INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
                DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
            ] {
                judgements.part_of_the_range_not_judged(
                    invariant,
                    DeletionScanGap::ExtentTreeHeadIdentifierNamesNoSingleInodeTree.reason(),
                );
            }
        }
    }
}

/// I-9.5 两个方向，一版各判一次：① 队列里的每个号在这棵 inode 树里有记录且 nlink = 0；② nlink = 0 的每条记录，号在队列里。
fn judge_the_deletion_queue_against_the_inode_records(
    version_name: &str,
    inode_tree: u64,
    queue: &BTreeSet<u64>,
    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
    judgements: &mut Judgements,
) {
    let invariant = DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES;
    let first_queued_inode_without_a_record_of_nlink_zero = queue
        .iter()
        .find(|inode| records.get(inode).is_none_or(|record| record.nlink != 0));
    judgements.judge(
        invariant,
        first_queued_inode_without_a_record_of_nlink_zero.is_none(),
        || {
            let inode = first_queued_inode_without_a_record_of_nlink_zero
                .expect("判红时队列里有一个号没有 nlink = 0 的记录");
            match records.get(inode) {
                None => format!(
                    "{version_name}：deleted_inodes 里的号 {inode} 在同一版的 inode 树（树 {inode_tree}）里没有记录"
                ),
                Some(record) => format!(
                    "{version_name}：deleted_inodes 里的号 {inode} 在同一版的 inode 树（树 {inode_tree}）里的记录 nlink = {}，不是 0",
                    record.nlink
                ),
            }
        },
    );
    let first_record_of_nlink_zero_outside_the_queue = records
        .iter()
        .find(|(inode, record)| record.nlink == 0 && !queue.contains(inode))
        .map(|(inode, _)| *inode);
    judgements.judge(
        invariant,
        first_record_of_nlink_zero_outside_the_queue.is_none(),
        || {
            let inode = first_record_of_nlink_zero_outside_the_queue
                .expect("判红时有一条 nlink = 0 的记录不在队列里");
            format!(
                "{version_name}：inode 树（树 {inode_tree}）里 inode {inode} 的记录 nlink = 0，号不在同一版的 deleted_inodes 里"
            )
        },
    );
}

/// I-4.7，对象取丙（判决第二节 F1）：nlink = 0 的记录的号，加上 extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 记录的号；
/// 每个这样的号要么在队列里，要么一个数据单元都不剩。
fn judge_inodes_being_deleted_against_the_queue_and_their_data_units(
    version_name: &str,
    queue: &BTreeSet<u64>,
    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
    data_units_of_each_inode: &BTreeMap<u64, u64>,
    judgements: &mut Judgements,
) {
    let mut inodes_being_deleted: BTreeSet<u64> = records
        .iter()
        .filter(|(_, record)| record.nlink == 0)
        .map(|(inode, _)| *inode)
        .collect();
    inodes_being_deleted.extend(
        data_units_of_each_inode
            .iter()
            .filter(|(inode, data_units)| {
                **data_units > 0 && !records.get(inode).is_some_and(|record| record.nlink >= 1)
            })
            .map(|(inode, _)| *inode),
    );
    for inode in inodes_being_deleted {
        let data_units_left = data_units_of_each_inode.get(&inode).copied().unwrap_or(0);
        judgements.judge(
            INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
            queue.contains(&inode) || data_units_left == 0,
            || {
                let why_being_deleted = match records.get(&inode) {
                    Some(_) => "记录 nlink = 0",
                    None => "inode 树里没有它的记录",
                };
                format!(
                    "{version_name}：inode {inode} 处于删除中（{why_being_deleted}），extent 树里还有 {data_units_left} 个数据单元，号也不在 deleted_inodes 里"
                )
            },
        );
    }
}

/// I-4.6 取乙：每条 nlink = 0 的记录，数据单元数与 size 对得上——size 为 0 时 0 或 1 个，size > 0 时 ⌈size ÷ 数据单元净荷容量⌉
/// （净荷容量 checker 自己算一份，[`data_unit_payload_capacity_in_bytes`]；实现那一侧同一式在 `crates/singlefs-core/src/write_request_split.rs`）。
fn judge_data_unit_counts_of_the_inodes_being_deleted_against_their_size(
    version_name: &str,
    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
    data_units_of_each_inode: &BTreeMap<u64, u64>,
    judgements: &mut Judgements,
) {
    let payload_capacity_in_bytes = data_unit_payload_capacity_in_bytes();
    for (inode, record) in records.iter().filter(|(_, record)| record.nlink == 0) {
        let data_units = data_units_of_each_inode.get(inode).copied().unwrap_or(0);
        let size_in_bytes = record.size_in_bytes;
        let data_units_the_size_asks_for = size_in_bytes.div_ceil(payload_capacity_in_bytes);
        let matches_the_size = if size_in_bytes == 0 {
            data_units <= 1
        } else {
            data_units == data_units_the_size_asks_for
        };
        judgements.judge(
            DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
            matches_the_size,
            || {
                format!(
                    "{version_name}：inode {inode} 的记录 nlink = 0、size {size_in_bytes}，按净荷容量 {payload_capacity_in_bytes} 要 {} 个数据单元，extent 树里有 {data_units} 个",
                    if size_in_bytes == 0 {
                        "0 或 1".to_string()
                    } else {
                        data_units_the_size_asks_for.to_string()
                    }
                )
            },
        );
    }
}

/// I-9.6 墓碑那一半：所选根（最新的那条根）的树表里墓碑树（种类 10）整棵读、不判，交回已发布的（按所选根指着的实例表判，
/// [`PublishedPredicate`]）墓碑容器里全部记录的对象 ID 最大值，一条都没有时 `Ok(None)`。被崩溃恢复抛弃的时间线上写的墓碑
/// 不在所选根的墓碑树里（`.claude/kb/invariants.md` I-9.6 那一行）。墓碑树根之下：类型段 1 的条目指码 3 墓碑容器，
/// 类型段 0 的指下一层码 2 节点（层级 = 父层级 − 1，且 ≥ 1）；读不全交回 [`DeletionScanGap::TombstoneTreeUnreadable`]。
fn largest_published_tombstone_object_identifier(
    reader: &dyn ImageReader,
    tree_table_pointer_of_the_chosen_root: &[u8],
    published: &PublishedPredicate<'_>,
    index_cache: &mut IndexNodeCache,
) -> Result<Option<u64>, DeletionScanGap> {
    let tree_table_pointer = parse_node_pointer(tree_table_pointer_of_the_chosen_root);
    if tree_table_pointer.all_zero {
        return Ok(None);
    }
    let tree_table = read_index_node_without_judging(reader, &tree_table_pointer, index_cache)
        .ok_or(DeletionScanGap::TombstoneTreeUnreadable)?;
    if tree_table
        .entries
        .iter()
        .any(|entry| entry.len() < tree_table_entry_bytes())
    {
        return Err(DeletionScanGap::TombstoneTreeUnreadable);
    }
    let mut packed_cache = PackedUnitCache::new();
    let mut largest_object_identifier: Option<u64> = None;
    let mut pending_nodes: Vec<PointerView> = tree_table
        .entries
        .iter()
        .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_TOMBSTONE)
        .map(|entry| parse_node_pointer(&entry[14..100]))
        .filter(|root_pointer| !root_pointer.all_zero)
        .collect();
    let tombstone_record_bytes =
        registered_record_width(PACKED_TYPE_TOMBSTONE).expect("类型 1 登记了记录宽 40");
    // 迭代上界：每个码 2 节点只从它的父条目入队一次，类型段 0 的孩子层级比父低一、层级 0 不入队，所以入队的节点数有限。
    while let Some(node_pointer) = pending_nodes.pop() {
        let node = read_index_node_without_judging(reader, &node_pointer, index_cache)
            .ok_or(DeletionScanGap::TombstoneTreeUnreadable)?;
        if node.level == 0 || node.entry_width < inode_internal_entry_bytes() {
            return Err(DeletionScanGap::TombstoneTreeUnreadable);
        }
        for entry in &node.entries {
            let child_pointer = parse_node_pointer(&entry[34..120]);
            match read_u16(entry, 16) {
                0 => {
                    let child_level =
                        read_index_node_without_judging(reader, &child_pointer, index_cache)
                            .ok_or(DeletionScanGap::TombstoneTreeUnreadable)?
                            .level;
                    if node.level.checked_sub(1) != Some(child_level) || child_level == 0 {
                        return Err(DeletionScanGap::TombstoneTreeUnreadable);
                    }
                    pending_nodes.push(child_pointer);
                }
                PACKED_TYPE_TOMBSTONE => {
                    let container =
                        read_packed_unit_without_judging(reader, &child_pointer, &mut packed_cache)
                            .ok_or(DeletionScanGap::TombstoneTreeUnreadable)?;
                    if container.view.record_type != PACKED_TYPE_TOMBSTONE
                        || container.view.record_width < tombstone_record_bytes
                    {
                        return Err(DeletionScanGap::TombstoneTreeUnreadable);
                    }
                    let is_published = published.holds_for(
                        WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
                            birth_txg: container.view.birth_txg,
                            instance: container.write_order_instance,
                        },
                    );
                    if !is_published {
                        continue;
                    }
                    for record in &container.view.records {
                        let object_identifier =
                            read_u64(record, TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET);
                        largest_object_identifier = Some(
                            largest_object_identifier.map_or(object_identifier, |largest| {
                                largest.max(object_identifier)
                            }),
                        );
                    }
                }
                _type_segment_outside_the_tombstone_tree => {
                    return Err(DeletionScanGap::TombstoneTreeUnreadable);
                }
            }
        }
    }
    Ok(largest_object_identifier)
}

/// 墓碑记录 40 字节里另外四段的偏移（D18（块里携带什么信息） 已定项 24 第 2 条：对象 ID 8 + 对象出生代 8 + 区间起 8 + 区间止 8 + 死亡 txg 8），
/// 按字段表另写一份、不用实现的解析。
const TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;
const TOMBSTONE_RECORD_RANGE_START_OFFSET: usize = 16;
const TOMBSTONE_RECORD_RANGE_END_OFFSET: usize = 24;
const TOMBSTONE_RECORD_DEATH_TXG_OFFSET: usize = 32;
/// 第一版墓碑只许整对象：区间 [0, 2⁶⁴ − 1]（D18（块里携带什么信息） 已定项 24 第 2 条）。
const TOMBSTONE_WHOLE_OBJECT_RANGE_START: u64 = 0;
const TOMBSTONE_WHOLE_OBJECT_RANGE_END: u64 = u64::MAX;
/// 谱系记录 24 字节：T_rb 8（回退那次发布的 txg）+ T_old 8（回退目标的 txg）+ H_cur 8（cur，即回退之前的现行那一版的 inode 号水位），
/// 读作「T_rb 的父是 T_old」（D18（块里携带什么信息） 已定项 24 第 5 条；三段的逐字节次序条款没定，实现取 T_rb | T_old | H_cur，按字段表另写一份、不用实现的解析）。
const LINEAGE_RECORD_ROLLBACK_TXG_OFFSET: usize = 0;
const LINEAGE_RECORD_PARENT_TXG_OFFSET: usize = 8;
const LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET: usize = 16;

/// 只供测试的墓碑容器容量旋钮压到的每片记录数（D18（块里携带什么信息） 已定项 24 第 3 条：旋钮不落盘，池级 checker 从镜像外拿；
/// 判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T5 取 K1）。取值 1..=815，构造时验。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TombstoneContainerCapacityKnob {
    records_per_container: u64,
}

impl TombstoneContainerCapacityKnob {
    /// 1..=815 之外交回 `None`（815 = ⌊(32768 − 136) ÷ 40⌋，产品容量 `TOMBSTONE_CONTAINER_RECORDS`）。
    #[must_use]
    pub fn new(records_per_container: u64) -> Option<Self> {
        (1..=TOMBSTONE_CONTAINER_RECORDS)
            .contains(&records_per_container)
            .then_some(Self {
                records_per_container,
            })
    }

    #[must_use]
    pub const fn records_per_container(self) -> u64 {
        self.records_per_container
    }
}

/// 只供测试的谱系容器容量旋钮压到的每片记录数（同墓碑容器那一个：旋钮不落盘，池级 checker 从镜像外拿；判决
/// `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2，I-9.37）。取值 1..=1359，构造时验。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineageContainerCapacityKnob {
    records_per_container: u64,
}

impl LineageContainerCapacityKnob {
    /// 1..=1359 之外交回 `None`（1359 = ⌊(32768 − 136) ÷ 24⌋，产品容量 `LINEAGE_CONTAINER_RECORDS`）。
    #[must_use]
    pub fn new(records_per_container: u64) -> Option<Self> {
        (1..=LINEAGE_CONTAINER_RECORDS)
            .contains(&records_per_container)
            .then_some(Self {
                records_per_container,
            })
    }

    #[must_use]
    pub const fn records_per_container(self) -> u64 {
        self.records_per_container
    }
}

/// 只供测试的退役容器容量旋钮压到的每片记录数（同上，I-9.37）。取值 1..=959，构造时验。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetirementContainerCapacityKnob {
    records_per_container: u64,
}

impl RetirementContainerCapacityKnob {
    /// 1..=959 之外交回 `None`（959 = ⌊(32768 − 136) ÷ 34⌋，产品容量 `RETIREMENT_CONTAINER_RECORDS`）。
    #[must_use]
    pub fn new(records_per_container: u64) -> Option<Self> {
        (1..=RETIREMENT_CONTAINER_RECORDS)
            .contains(&records_per_container)
            .then_some(Self {
                records_per_container,
            })
    }

    #[must_use]
    pub const fn records_per_container(self) -> u64 {
        self.records_per_container
    }
}

/// 写这张镜像时三棵容器树各压没压容量（从镜像外给）：`None` 的那一棵当没装旋钮，照产品容量判（墓碑树 I-9.21，谱系树、退役树 I-9.36）；
/// `Some` 的那一棵按旋钮值判（墓碑树 I-9.22，谱系树、退役树 I-9.37）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerCapacityKnobs {
    pub tombstone: Option<TombstoneContainerCapacityKnob>,
    pub lineage: Option<LineageContainerCapacityKnob>,
    pub retirement: Option<RetirementContainerCapacityKnob>,
}

impl ContainerCapacityKnobs {
    /// 三棵都没装旋钮（产品镜像，[`check_pool_image`] 那一份）。
    pub const PRODUCT: Self = Self {
        tombstone: None,
        lineage: None,
        retirement: None,
    };
}

/// 删除线新九条（I-9.17–I-9.23、I-9.29；I-9.24 的在 [`judge_deletion_state_of_the_versions`] 末尾）一次都没判到时各自报的不适用理由：
/// 每条一句、说清这张镜像上为什么没有它判的对象，不落到 `Judgements::into_report` 的缺省理由「这个镜像上没有它判的对象」。
/// 公开给「逐条判定、不在豁免里的一律要求成立」的用例逐字认（豁免只认这一句，别的不适用照红）。
/// I-9.18（删除队列里的号有谱系上的墓碑）：每一版的队列都是空的。
pub const NO_QUEUED_INODE_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版 deleted_inodes 里都没有号：没有队列里的号可判";
/// I-9.19（谱系上的墓碑不指活对象）：没有一条谱系上的墓碑在同一版 inode 树里有同号同对象出生代的记录。
pub const NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD: &str =
    "走读到的每一版里都没有在谱系上、而 inode 树里还有同号同对象出生代记录的墓碑";
/// I-9.20（谱系上的墓碑不重号且记录合法）：每一版墓碑树都是零根。
pub const NO_TOMBSTONE_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版墓碑树都是零根：没有墓碑记录可判";
/// I-9.29（同号只许一个出生代）：没有一版收到号（inode 记录与谱系上的墓碑都没有）。
pub const NO_INODE_RECORD_OR_TOMBSTONE_ON_THE_LINEAGE_TO_COMPARE: &str =
    "走读到的每一版里都没有 inode 记录与谱系上的墓碑：没有号可比";
/// I-9.21（产品镜像里非最右墓碑容器装满），旧入口：没有非最右的容器。
pub const NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER: &str =
    "走读到的每一版墓碑树都是零根或只一片容器：没有非最右的容器";
/// I-9.22（测试镜像里墓碑容器不超旋钮值），旧入口：当没装旋钮，这一条不判。
pub const NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY: &str =
    "池级 checker 的旧入口，当没装只供测试的墓碑容量旋钮：旋钮那一条不判";
/// I-9.21，新入口给了旋钮值：产品容量那一条不判。
pub const PRODUCT_TOMBSTONE_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOB: &str =
    "这张镜像用了只供测试的墓碑容量旋钮（新入口给了旋钮值）：产品容量那一条不判";
/// I-9.22，新入口给了旋钮值：每一版墓碑树都是零根。
pub const NO_TOMBSTONE_CONTAINER_UNDER_THE_KNOB: &str =
    "走读到的每一版墓碑树都是零根：没有墓碑容器可判";
/// I-9.36（产品镜像里非最右谱系与退役容器装满），没装旋钮的那一棵（或两棵）上：每一版都是零根或只一片容器。
pub const NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER: &str =
    "走读到的每一版里没装旋钮的谱系树与退役树都是零根或只一片容器：没有非最右的容器";
/// I-9.36，谱系树与退役树都装了旋钮（新入口给了两个旋钮值）：产品容量那一条不判。
pub const PRODUCT_LINEAGE_AND_RETIREMENT_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOBS: &str =
    "这张镜像的谱系与退役容量都用了只供测试的旋钮（新入口给了两个旋钮值）：产品容量那一条不判";
/// I-9.37（测试镜像里谱系与退役容器不超旋钮值），两棵都没装旋钮（旧入口，或新入口没给这两个旋钮值）。
pub const NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB: &str =
    "池级 checker 没拿到只供测试的谱系与退役容量旋钮（旧入口，或新入口没给这两个旋钮值）：旋钮那一条不判";
/// I-9.37，装了旋钮的那一棵（或两棵）在每一版都是零根。
pub const NO_LINEAGE_OR_RETIREMENT_CONTAINER_UNDER_THE_KNOBS: &str =
    "走读到的每一版里装了旋钮的谱系树与退役树都是零根：没有容器可判";
/// I-9.17（谱系树条目身份与子头相符）：主走读一条谱系树内部条目都没走到。
pub const NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版谱系树都是零根：没有谱系树内部条目可判";
/// I-9.23（墓碑树条目身份与子头相符）：主走读一条墓碑树内部条目都没走到。
pub const NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版墓碑树都是零根：没有墓碑树内部条目可判";

/// 退役记录那几条（I-9.32–I-9.35）在一版上的那一组：一版的树表读不出、退役树读不全时这几条在那一版判不了。
const RETIREMENT_SCAN_INVARIANTS: [&str; 4] = [
    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
];
/// I-9.32：走读到的每一版退役树都是零根。
pub const NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版退役树都是零根：没有退役记录可判";
/// I-9.33、I-9.34：没有一条退役记录的死亡 txg 落在它那一版的父链上。
pub const NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版里没有一条退役记录在谱系上：没有对象可判";
/// I-9.35：没有一版的父版在回退候选集里读得出。
pub const NO_VERSION_WHOSE_PARENT_IS_IN_THE_CANDIDATE_SET: &str =
    "走读到的每一版的父版都不在回退候选集里（或都没有 inode 树）：没有相邻两版可比";
/// I-9.31（退役树条目身份与子头相符）：主走读一条退役树内部条目都没走到。
pub const NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版退役树都是零根：没有退役树内部条目可判";

/// 墓碑与谱系那一遍在一版上判 I-9.18、I-9.19、I-9.20、I-9.29 要的那一组：一版的树表读不出、墓碑树读不全时这几条在那一版判不了
/// （I-9.21 / I-9.22 只记装没装旋钮的那一条，[`capacity_invariant_of`]）。
const TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS: [&str; 4] = [
    QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
];

/// 回退候选集滤没滤得掉被崩溃恢复抛弃的根：候选集按最新根指着的实例表判被抛弃的根（D23（journal 的角色与格式） 已定项 14 回退段的候选集规则），
/// 那张表读不全时滤不全，被抛弃时间线上的根可能进了候选集。I-9.29（同号只许一个出生代） 跨版本比，这时记判不了
/// （`.claude/kb/invariants.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」）：被抛弃的线上同号可以是另一个对象。
/// 读没读全看的是候选集滤法用的那一遍（主走读，`Walk::walk_instance_table_chain`），不另读一遍：另一个读法（只读不判的
/// [`instance_table_rows_of_root_without_judging`]）拒收的比主走读多（载荷 CRC 对不上；I-3.14 的五形两边一样当解不开，第 10k 批、第 10u 批），
/// 用它当门会在候选集已经滤全时报判不了（判决 `research/prompts/m3-batch23-code-r1-main-verification.md` 第二节 K2 N5）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AbandonedRootsFilteringOfTheCandidateSet {
    /// 主走读沿最新根的实例表链走到了末片（链指针记录说「没有下一片」）：整张表的行都收下了，被抛弃的根都按它滤掉了。
    FilteredByTheNewestRootsInstanceTable,
    /// 主走读沿最新根的实例表链没走到末片（一片读不出、解不开、身份不对、链指针记录不合法、链成了环）：被抛弃的根可能留在候选集里。
    UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable,
}

/// 墓碑树、谱系树、退役树沿叶序第一片容器的容器号（D18（块里携带什么信息） 已定项 24 第 1、5 条：容器号从 1 起，下一个号 = 树里最大号 + 1，
/// 零根时 1；I-9.21、I-9.22、I-9.36、I-9.37 判它，第 56 批 G2）。
const FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE: u64 = 1;

/// 装没装旋钮决定墓碑容器容量判哪一条：没装（旧入口）判产品那一条 I-9.21，装了判测试那一条 I-9.22。
const fn capacity_invariant_of(
    capacity_knob: Option<TombstoneContainerCapacityKnob>,
) -> &'static str {
    match capacity_knob {
        None => NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        Some(_) => TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
    }
}

/// I-9.36 / I-9.37 判的那两棵容器树：说明里怎么称呼、产品容量。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LineageOrRetirementTree {
    Lineage,
    Retirement,
}

impl LineageOrRetirementTree {
    const fn name(self) -> &'static str {
        match self {
            LineageOrRetirementTree::Lineage => "谱系树",
            LineageOrRetirementTree::Retirement => "退役树",
        }
    }

    /// 一片码 3 容器的产品容量（D18（块里携带什么信息） 已定项 24：开放容器满了才关）。
    const fn product_records_per_container(self) -> u64 {
        match self {
            LineageOrRetirementTree::Lineage => LINEAGE_CONTAINER_RECORDS,
            LineageOrRetirementTree::Retirement => RETIREMENT_CONTAINER_RECORDS,
        }
    }
}

/// 一棵谱系树或退役树装没装旋钮（旋钮值，每片记录数）决定它判哪一条：没装判产品那一条 I-9.36，装了判测试那一条 I-9.37（同 [`capacity_invariant_of`]）。
const fn lineage_or_retirement_capacity_invariant_of(
    knob_records_per_container: Option<u64>,
) -> &'static str {
    match knob_records_per_container {
        None => NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        Some(_) => LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
    }
}

/// 一条墓碑记录的五段。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TombstoneRecordOfTheScan {
    object: u64,
    object_birth: u64,
    range_start: u64,
    range_end: u64,
    death_txg: u64,
}

impl TombstoneRecordOfTheScan {
    fn of(record: &[u8]) -> Self {
        Self {
            object: read_u64(record, TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET),
            object_birth: read_u64(record, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET),
            range_start: read_u64(record, TOMBSTONE_RECORD_RANGE_START_OFFSET),
            range_end: read_u64(record, TOMBSTONE_RECORD_RANGE_END_OFFSET),
            death_txg: read_u64(record, TOMBSTONE_RECORD_DEATH_TXG_OFFSET),
        }
    }

    fn covers_the_whole_object(self) -> bool {
        self.range_start < self.range_end
            && self.range_start == TOMBSTONE_WHOLE_OBJECT_RANGE_START
            && self.range_end == TOMBSTONE_WHOLE_OBJECT_RANGE_END
    }
}

/// 一片墓碑容器：头里的容器号（I-9.21 / I-9.22 沿叶序比它，不沿根的条目读：根长高之后根的条目是类型段 0、容器号恒 0）与它的记录。
struct TombstoneContainerOfTheScan {
    container: u64,
    records: Vec<TombstoneRecordOfTheScan>,
}

/// 一版树表里的一棵墓碑树：树 ID、头 ID（I-9.27：等于这张表里那棵 inode 树的树 ID）与按叶序读出的容器（读不全交回判不了的理由）。
struct TombstoneTreeOfTheScan {
    tree: u64,
    head_identifier: u64,
    containers: Result<Vec<TombstoneContainerOfTheScan>, DeletionScanGap>,
}

/// 一条谱系记录：T_rb 的父是 T_old；H_cur 是回退之前的现行那一版的 inode 号水位。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LineageRecordOfTheScan {
    rollback_txg: u64,
    parent_txg: u64,
    inode_number_watermark_of_the_version_rolled_back_from: u64,
}

impl LineageRecordOfTheScan {
    fn of(record: &[u8]) -> Self {
        Self {
            rollback_txg: read_u64(record, LINEAGE_RECORD_ROLLBACK_TXG_OFFSET),
            parent_txg: read_u64(record, LINEAGE_RECORD_PARENT_TXG_OFFSET),
            inode_number_watermark_of_the_version_rolled_back_from: read_u64(
                record,
                LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET,
            ),
        }
    }
}

/// 一版的父链在 txg 轴上占的那几段（每段闭区间）：「在谱系上」= 死亡 txg 落在其中一段里。
struct LineageChainOfAVersion {
    segments_on_the_chain: Vec<(u64, u64)>,
}

impl LineageChainOfAVersion {
    fn holds_txg(&self, txg: u64) -> bool {
        self.segments_on_the_chain
            .iter()
            .any(|(lowest_txg, highest_txg)| (*lowest_txg..=*highest_txg).contains(&txg))
    }
}

/// 一版里一棵 inode 树的全部记录（号 → 记录），读不全交回判不了的理由。
type InodeRecordsOfAnInodeTreeOfTheScan =
    Result<BTreeMap<u64, InodeRecordOfTheDeletionScan>, DeletionScanGap>;

/// 跨版本那张表（I-9.29）：(inode 树的树 ID, inode 号) → 对象出生代 → 第一次在哪一版的什么地方见到它。
type ObjectBirthsOfEachInodeNumber = BTreeMap<(u64, u64), BTreeMap<u64, String>>;

/// 「在谱系上」的父链（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T3 取 a）：从这一版自己的 txg 起、
/// 按这一版树表里谱系树的记录往回走——有谱系记录 (T_rb, T_old) 的那个 txg 的父是 T_old，其余 txg 的父是 txg − 1（I-9.18 那一行）。
/// 记录按 T_rb 从大到小过一遍：T_rb 大于此刻走到的 txg 的不在这条链上（写在被后来的回退抛弃的分支上，或晚于这一版），跳过；
/// 否则 [T_rb, 此刻] 整段在链上、跳到 T_old。最后 [0, 此刻] 也在链上。按段走是 O(L log L)，不按 txg 一步步走。
/// T_old ≥ T_rb、同一个 T_rb 两条的谱系接不成一条链，交回 [`DeletionScanGap::LineageRecordsInconsistent`]。
fn lineage_chain_of_a_version(
    version_txg: u64,
    lineage_records: &[LineageRecordOfTheScan],
) -> Result<LineageChainOfAVersion, DeletionScanGap> {
    let mut records_with_the_latest_rollback_first = lineage_records.to_vec();
    records_with_the_latest_rollback_first
        .sort_unstable_by_key(|record| std::cmp::Reverse(record.rollback_txg));
    let some_record_is_not_younger_than_its_parent = records_with_the_latest_rollback_first
        .iter()
        .any(|record| record.parent_txg >= record.rollback_txg);
    let some_rollback_has_two_records = records_with_the_latest_rollback_first
        .windows(2)
        .any(|adjacent| adjacent[0].rollback_txg == adjacent[1].rollback_txg);
    if some_record_is_not_younger_than_its_parent || some_rollback_has_two_records {
        return Err(DeletionScanGap::LineageRecordsInconsistent);
    }
    let mut segments_on_the_chain = Vec::new();
    let mut txg_walked_to = version_txg;
    // 迭代上界是谱系记录条数；跨轮携带的是此刻走到的 txg，每接上一段就跳到更小的 T_old（上面判过 T_old < T_rb ≤ 此刻），只减不增。
    for record in &records_with_the_latest_rollback_first {
        if record.rollback_txg > txg_walked_to {
            continue;
        }
        segments_on_the_chain.push((record.rollback_txg, txg_walked_to));
        txg_walked_to = record.parent_txg;
    }
    segments_on_the_chain.push((0, txg_walked_to));
    Ok(LineageChainOfAVersion {
        segments_on_the_chain,
    })
}

/// 墓碑树或谱系树（根恒码 2、条目 120 字节，类型段 0 指下一层码 2 节点、类型段 = `record_type` 指码 3 容器）整棵读、不判，
/// 按叶序交回每片容器；节点层级或条目宽不对、类型段不认得、容器的打包记录类型或记录宽不是登记的那一个都交回 `gap`
/// （读不出、形状不对由主走读判：I-2.1、I-1.7、I-9.17、I-9.23 等，这一遍只如实记判不了——I-1.7 的坏镜像因此只红 I-1.7）。
/// 根指针全零（零根）交回空。
fn containers_of_a_container_tree_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    record_type: u16,
    gap: DeletionScanGap,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
) -> Result<Vec<crate::PackedUnitView>, DeletionScanGap> {
    let mut containers = Vec::new();
    if root_pointer.all_zero {
        return Ok(containers);
    }
    let root = read_index_node_without_judging(reader, root_pointer, index_cache).ok_or(gap)?;
    collect_containers_of_a_container_tree_below(
        reader,
        &root,
        record_type,
        gap,
        index_cache,
        packed_cache,
        &mut containers,
    )?;
    Ok(containers)
}

/// [`containers_of_a_container_tree_without_judging`] 的一个码 2 节点：条目逐条，类型段 0 的往下一层（层级 = 父层级 − 1 且 ≥ 1），
/// 容器按条目次序追加进 `containers`。递归的上界：每下一层层级减一。
fn collect_containers_of_a_container_tree_below(
    reader: &dyn ImageReader,
    node: &crate::IndexNodeView,
    record_type: u16,
    gap: DeletionScanGap,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
    containers: &mut Vec<crate::PackedUnitView>,
) -> Result<(), DeletionScanGap> {
    if node.level == 0 || node.entry_width < inode_internal_entry_bytes() {
        return Err(gap);
    }
    let registered_width = registered_record_width(record_type);
    for entry in &node.entries {
        let child_pointer = parse_node_pointer(&entry[34..120]);
        match ContainerTreeEntryType::of(read_u16(entry, 16), record_type) {
            ContainerTreeEntryType::InternalNode => {
                if child_pointer.all_zero {
                    return Err(gap);
                }
                let child = read_index_node_without_judging(reader, &child_pointer, index_cache)
                    .ok_or(gap)?;
                if node.level.checked_sub(1) != Some(child.level) || child.level == 0 {
                    return Err(gap);
                }
                collect_containers_of_a_container_tree_below(
                    reader,
                    &child,
                    record_type,
                    gap,
                    index_cache,
                    packed_cache,
                    containers,
                )?;
            }
            ContainerTreeEntryType::Container => {
                let container =
                    read_packed_unit_without_judging(reader, &child_pointer, packed_cache)
                        .ok_or(gap)?;
                if container.view.record_type != record_type
                    || Some(container.view.record_width) != registered_width
                {
                    return Err(gap);
                }
                containers.push(container.view);
            }
            ContainerTreeEntryType::Unregistered(_) => return Err(gap),
        }
    }
    Ok(())
}

/// 一版的墓碑树按叶序读出的每片容器与记录，只读不判。
fn tombstone_containers_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
) -> Result<Vec<TombstoneContainerOfTheScan>, DeletionScanGap> {
    let containers = containers_of_a_container_tree_without_judging(
        reader,
        root_pointer,
        PACKED_TYPE_TOMBSTONE,
        DeletionScanGap::TombstoneTreeOfTheVersionUnreadable,
        index_cache,
        packed_cache,
    )?;
    Ok(containers
        .iter()
        .map(|container| TombstoneContainerOfTheScan {
            container: container.container,
            records: container
                .records
                .iter()
                .map(|record| TombstoneRecordOfTheScan::of(record))
                .collect(),
        })
        .collect())
}

/// 一片谱系容器：头里的容器号（I-9.36 / I-9.37 沿叶序比它）与它的记录。
struct LineageContainerOfTheScan {
    container: u64,
    records: Vec<LineageRecordOfTheScan>,
}

/// 一版的谱系树按叶序读出的每片容器与记录，只读不判。
fn lineage_containers_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
) -> Result<Vec<LineageContainerOfTheScan>, DeletionScanGap> {
    let containers = containers_of_a_container_tree_without_judging(
        reader,
        root_pointer,
        PACKED_TYPE_LINEAGE,
        DeletionScanGap::LineageTreeUnreadable,
        index_cache,
        packed_cache,
    )?;
    Ok(containers
        .iter()
        .map(|container| LineageContainerOfTheScan {
            container: container.container,
            records: container
                .records
                .iter()
                .map(|record| LineageRecordOfTheScan::of(record))
                .collect(),
        })
        .collect())
}

/// 一版的谱系树里的全部谱系记录，只读不判。
fn lineage_records_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
) -> Result<Vec<LineageRecordOfTheScan>, DeletionScanGap> {
    let containers = containers_of_a_container_tree_without_judging(
        reader,
        root_pointer,
        PACKED_TYPE_LINEAGE,
        DeletionScanGap::LineageTreeUnreadable,
        index_cache,
        packed_cache,
    )?;
    Ok(containers
        .iter()
        .flat_map(|container| container.records.iter())
        .map(|record| LineageRecordOfTheScan::of(record))
        .collect())
}

/// 退役记录（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2 条，34 字节小端）各段的偏移：被退役身份的出生树 0、
/// 打包记录类型 8（2 字节）、容器号 10、容器出生代 18，死亡 txg 26。checker 按条款另写一份，不用实现的读法。
const RETIREMENT_RECORD_BIRTH_TREE_OFFSET: usize = 0;
const RETIREMENT_RECORD_RECORD_TYPE_OFFSET: usize = 8;
const RETIREMENT_RECORD_CONTAINER_OFFSET: usize = 10;
const RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET: usize = 18;
const RETIREMENT_RECORD_DEATH_TXG_OFFSET: usize = 26;

/// 一片打包记录容器的身份四段（出生树、打包记录类型、容器号、容器出生代）：inode 树内部条目里的身份引用、退役记录里被退役的那一个都是这一形。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ContainerIdentityOfTheScan {
    birth_tree: u64,
    record_type: u16,
    container: u64,
    container_birth: u64,
}

impl ContainerIdentityOfTheScan {
    /// 从一段按「出生树 8 | 打包记录类型 2 | 容器号 8 | 容器出生代 8」排的字节里读（`offset` 是出生树那一段的起点）。
    fn read_at(bytes: &[u8], offset: usize) -> Self {
        Self {
            birth_tree: read_u64(bytes, offset),
            record_type: read_u16(
                bytes,
                offset + RETIREMENT_RECORD_RECORD_TYPE_OFFSET - RETIREMENT_RECORD_BIRTH_TREE_OFFSET,
            ),
            container: read_u64(
                bytes,
                offset + RETIREMENT_RECORD_CONTAINER_OFFSET - RETIREMENT_RECORD_BIRTH_TREE_OFFSET,
            ),
            container_birth: read_u64(
                bytes,
                offset + RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET
                    - RETIREMENT_RECORD_BIRTH_TREE_OFFSET,
            ),
        }
    }
}

/// 一条退役记录：被退役的身份与死亡 txg。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RetirementRecordOfTheScan {
    retired: ContainerIdentityOfTheScan,
    death_txg: u64,
}

impl RetirementRecordOfTheScan {
    fn of(record: &[u8]) -> Self {
        Self {
            retired: ContainerIdentityOfTheScan::read_at(
                record,
                RETIREMENT_RECORD_BIRTH_TREE_OFFSET,
            ),
            death_txg: read_u64(record, RETIREMENT_RECORD_DEATH_TXG_OFFSET),
        }
    }
}

/// 一片退役容器：头里的容器号与它的记录（I-9.32 数每片不多于 959 条）。
struct RetirementContainerOfTheScan {
    container: u64,
    records: Vec<RetirementRecordOfTheScan>,
}

/// 一版树表里的一棵退役树：树 ID、头 ID（I-9.27：等于这张表里那棵 inode 树的树 ID）与按叶序读出的容器（读不全交回判不了的理由）。
struct RetirementTreeOfTheScan {
    tree: u64,
    head_identifier: u64,
    containers: Result<Vec<RetirementContainerOfTheScan>, DeletionScanGap>,
}

/// 一版的退役树按叶序读出的每片容器与记录，只读不判。
fn retirement_containers_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
) -> Result<Vec<RetirementContainerOfTheScan>, DeletionScanGap> {
    let containers = containers_of_a_container_tree_without_judging(
        reader,
        root_pointer,
        PACKED_TYPE_RETIREMENT,
        DeletionScanGap::RetirementTreeUnreadable,
        index_cache,
        packed_cache,
    )?;
    Ok(containers
        .iter()
        .map(|container| RetirementContainerOfTheScan {
            container: container.container,
            records: container
                .records
                .iter()
                .map(|record| RetirementRecordOfTheScan::of(record))
                .collect(),
        })
        .collect())
}

/// 一版 inode 树里每片叶容器的身份（内部条目里类型段 2 那几条的身份引用，不开叶：判决第三节第 8 条 I-9.34「读内部条目的身份引用即可」），
/// 只读不判。根指针全零、零条目根交回空。读不出、形状不对的交回 [`DeletionScanGap::InodeTreeUnreadable`]（主走读判 I-9.2 那些）。
fn inode_leaf_identities_without_judging(
    reader: &dyn ImageReader,
    root_pointer: &PointerView,
    index_cache: &mut IndexNodeCache,
) -> Result<BTreeSet<ContainerIdentityOfTheScan>, DeletionScanGap> {
    let mut identities = BTreeSet::new();
    if root_pointer.all_zero {
        return Ok(identities);
    }
    let root = read_index_node_without_judging(reader, root_pointer, index_cache)
        .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
    if root.level == 0 {
        return if root.entries.is_empty() {
            Ok(identities)
        } else {
            Err(DeletionScanGap::InodeTreeUnreadable)
        };
    }
    collect_inode_leaf_identities_below(reader, &root, index_cache, &mut identities)?;
    Ok(identities)
}

/// [`inode_leaf_identities_without_judging`] 的一个内部节点：类型段 2 的条目收它的身份引用，类型段 0 的往下一层（层级 = 父层级 − 1 且 ≥ 1）。
/// 递归的上界：每下一层层级减一。
fn collect_inode_leaf_identities_below(
    reader: &dyn ImageReader,
    node: &crate::IndexNodeView,
    index_cache: &mut IndexNodeCache,
    identities: &mut BTreeSet<ContainerIdentityOfTheScan>,
) -> Result<(), DeletionScanGap> {
    if node.entry_width < inode_internal_entry_bytes() {
        return Err(DeletionScanGap::InodeTreeUnreadable);
    }
    for entry in &node.entries {
        match InodeInternalEntryType::of(read_u16(entry, 16)) {
            InodeInternalEntryType::LeafContainer => {
                identities.insert(ContainerIdentityOfTheScan::read_at(entry, 8));
            }
            InodeInternalEntryType::InternalNode => {
                let child_pointer = parse_node_pointer(&entry[34..120]);
                if child_pointer.all_zero {
                    return Err(DeletionScanGap::InodeTreeUnreadable);
                }
                let child = read_index_node_without_judging(reader, &child_pointer, index_cache)
                    .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
                if node.level.checked_sub(1) != Some(child.level) || child.level == 0 {
                    return Err(DeletionScanGap::InodeTreeUnreadable);
                }
                collect_inode_leaf_identities_below(reader, &child, index_cache, identities)?;
            }
            InodeInternalEntryType::Unregistered(_) => {
                return Err(DeletionScanGap::InodeTreeUnreadable);
            }
        }
    }
    Ok(())
}

/// I-9.35 跨相邻两版比要的、一版里的那几样：txg、父版的 txg（这一版谱系树里有 T_rb 等于这一版 txg 的记录时取它的 T_old，否则 txg − 1；
/// txg 0 没有父版；谱系树读不全交回判不了的理由）、每棵 inode 树（按树 ID）的叶身份集、经头 ID 配到每棵 inode 树的退役树里死亡 txg 等于这一版
/// txg 的退役记录的身份集。树表 0 条的一版没有 inode 树（当父版时它的每棵树都是空集）。
struct VersionOfTheRetirementCompleteness {
    name: String,
    checkpoint_txg: u64,
    parent_txg: Result<Option<u64>, DeletionScanGap>,
    leaf_identities_of_each_inode_tree:
        BTreeMap<u64, Result<BTreeSet<ContainerIdentityOfTheScan>, DeletionScanGap>>,
    retired_at_this_txg_of_each_inode_tree:
        BTreeMap<u64, Result<BTreeSet<ContainerIdentityOfTheScan>, DeletionScanGap>>,
    /// 这一版树表读不出、条目窄：它当父版时比不了。
    unreadable: Option<DeletionScanGap>,
}

/// I-9.32（退役记录合法）在一版的一棵退役树上：每条记录被退役身份的打包记录类型 == 2、出生树 ≠ 0，容器出生代 < 死亡 txg ≤ 这一版的 txg；
/// 每片退役容器的记录数 ≤ 959（正推 K5.1）。不按谱系滤：每条都判。
fn judge_every_retirement_record_of_a_version_is_well_formed(
    version: &VersionToScanTheDeletionStateOf<'_>,
    retirement_tree: u64,
    containers: &[RetirementContainerOfTheScan],
    judgements: &mut Judgements,
) {
    let records_at_most = usize::try_from(RETIREMENT_CONTAINER_RECORDS).expect("959 装得进 usize");
    for container in containers {
        let record_count = container.records.len();
        judgements.judge(
            EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
            record_count <= records_at_most,
            || {
                format!(
                    "{}退役树 {retirement_tree} 里容器 {} 装了 {record_count} 条退役记录，多于一片的 {records_at_most} 条",
                    version.name, container.container
                )
            },
        );
        for record in &container.records {
            let retired = record.retired;
            let holds = retired.record_type == PACKED_TYPE_INODE
                && retired.birth_tree != 0
                && retired.container_birth < record.death_txg
                && record.death_txg <= version.checkpoint_txg;
            judgements.judge(EVERY_RETIREMENT_RECORD_IS_WELL_FORMED, holds, || {
                format!(
                    "{}退役树 {retirement_tree} 容器 {} 里的退役记录 {retired:?}（死于 txg {}）：被退役身份的类型要是 2、出生树不为 0，\
                     容器出生代 < 死亡 txg ≤ 这一版的 txg {}",
                    version.name, container.container, record.death_txg, version.checkpoint_txg
                )
            });
        }
    }
}

/// I-9.33（谱系上的退役记录按身份不重复）在一版的一棵退役树上：死亡 txg 在这一版父链上的那几条，按身份不重复。
fn judge_retirement_records_on_the_lineage_of_a_version_are_distinct(
    version_name: &str,
    retirement_tree: u64,
    containers: &[RetirementContainerOfTheScan],
    lineage_chain: &LineageChainOfAVersion,
    judgements: &mut Judgements,
) {
    let mut seen: BTreeMap<ContainerIdentityOfTheScan, u64> = BTreeMap::new();
    for record in containers
        .iter()
        .flat_map(|container| container.records.iter())
        .filter(|record| lineage_chain.holds_txg(record.death_txg))
    {
        let earlier_death_txg = seen.insert(record.retired, record.death_txg);
        judgements.judge(
            RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
            earlier_death_txg.is_none(),
            || {
                format!(
                    "{version_name}退役树 {retirement_tree} 里谱系上有两条退役记录指同一个身份 {:?}（死于 txg {:?} 与 {}）",
                    record.retired, earlier_death_txg, record.death_txg
                )
            },
        );
    }
}

/// I-9.34（谱系上的退役记录不指本版活叶）在一版的一棵退役树上：死亡 txg 在这一版父链上的退役记录指认的身份不在头 ID 配得上的那棵 inode 树的叶身份集里。
fn judge_retirement_records_on_the_lineage_name_no_live_leaf(
    version_name: &str,
    retirement_tree: u64,
    containers: &[RetirementContainerOfTheScan],
    leaf_identities: &BTreeSet<ContainerIdentityOfTheScan>,
    lineage_chain: &LineageChainOfAVersion,
    judgements: &mut Judgements,
) {
    for record in containers
        .iter()
        .flat_map(|container| container.records.iter())
        .filter(|record| lineage_chain.holds_txg(record.death_txg))
    {
        judgements.judge(
            RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
            !leaf_identities.contains(&record.retired),
            || {
                format!(
                    "{version_name}退役树 {retirement_tree} 里谱系上的退役记录（死于 txg {}）指的身份 {:?} 是这一版 inode 树里的一片叶",
                    record.death_txg, record.retired
                )
            },
        );
    }
}

/// I-9.35（退役完备）：每一版、它每棵 inode 树，父版（[`VersionOfTheRetirementCompleteness::parent_txg`]）在回退候选集里读得出时，
/// 父版那棵树的叶身份集 − 这一版的叶身份集 = 这一版死亡 txg 等于自己 txg 的退役记录的身份集；两个方向（漏记、多记）各报。
/// 父版那个 txg 不在候选集里的不判（崩溃跳号、最旧那一版）；父版读不全、同 txg 有不止一版的报射程里判不了。
/// 候选集滤不掉被崩溃恢复抛弃的根时整条报部分没判（照 I-9.29 的门：被抛弃线上的同一个 txg 可以是另一版）。
fn judge_retirement_records_account_for_every_leaf_gone_since_the_parent_version(
    versions: &[VersionOfTheRetirementCompleteness],
    abandoned_roots_filtering: AbandonedRootsFilteringOfTheCandidateSet,
    judgements: &mut Judgements,
) {
    match abandoned_roots_filtering {
        AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable => {}
        AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable => {
            judgements.partially_judged(
                RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                format!(
                    "最新根的实例表读不出（主走读沿链没走到末片）：回退候选集里滤不掉被崩溃恢复抛弃的根，{} 版的父版一版都没比",
                    versions.len()
                ),
            );
            return;
        }
    }
    let empty_leaf_set: BTreeSet<ContainerIdentityOfTheScan> = BTreeSet::new();
    for version in versions {
        if version.leaf_identities_of_each_inode_tree.is_empty() {
            continue;
        }
        let parent_txg = match version.parent_txg {
            Ok(Some(parent_txg)) => parent_txg,
            Ok(None) => continue,
            Err(gap) => {
                judgements.part_of_the_range_not_judged(
                    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                    gap.reason(),
                );
                continue;
            }
        };
        let parents: Vec<&VersionOfTheRetirementCompleteness> = versions
            .iter()
            .filter(|candidate| candidate.checkpoint_txg == parent_txg)
            .collect();
        let parent = match parents.as_slice() {
            [] => continue,
            [parent] => *parent,
            [_, _, ..] => {
                judgements.part_of_the_range_not_judged(
                    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                    DeletionScanGap::ParentVersionNotSingle.reason(),
                );
                continue;
            }
        };
        if let Some(gap) = parent.unreadable {
            judgements.part_of_the_range_not_judged(
                RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                gap.reason(),
            );
            continue;
        }
        for (inode_tree, leaf_identities) in &version.leaf_identities_of_each_inode_tree {
            let parent_leaf_identities =
                match parent.leaf_identities_of_each_inode_tree.get(inode_tree) {
                    None => &empty_leaf_set,
                    Some(Ok(identities)) => identities,
                    Some(Err(gap)) => {
                        judgements.part_of_the_range_not_judged(
                            RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                            gap.reason(),
                        );
                        continue;
                    }
                };
            let leaf_identities = match leaf_identities {
                Ok(identities) => identities,
                Err(gap) => {
                    judgements.part_of_the_range_not_judged(
                        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                        gap.reason(),
                    );
                    continue;
                }
            };
            let retired_at_this_txg = match version
                .retired_at_this_txg_of_each_inode_tree
                .get(inode_tree)
            {
                None => &empty_leaf_set,
                Some(Ok(identities)) => identities,
                Some(Err(gap)) => {
                    judgements.part_of_the_range_not_judged(
                        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                        gap.reason(),
                    );
                    continue;
                }
            };
            let gone: BTreeSet<ContainerIdentityOfTheScan> = parent_leaf_identities
                .difference(leaf_identities)
                .copied()
                .collect();
            let missing: Vec<&ContainerIdentityOfTheScan> =
                gone.difference(retired_at_this_txg).collect();
            let extra: Vec<&ContainerIdentityOfTheScan> =
                retired_at_this_txg.difference(&gone).collect();
            judgements.judge(
                RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
                missing.is_empty() && extra.is_empty(),
                || {
                    format!(
                        "{}（txg {}）的 inode 树 {inode_tree} 比父版（txg {parent_txg}）少了的叶与死于这一版的退役记录对不上：漏记 {missing:?}，多记 {extra:?}",
                        version.name, version.checkpoint_txg
                    )
                },
            );
        }
    }
}

/// I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上）：最新的根下面没有走全了的记账树，或记账里没有 inode 号水位那一行。
pub const NO_INODE_NUMBER_WATERMARK_ROW_UNDER_THE_NEWEST_ROOT: &str =
    "最新的根下面没有走全了的记账树、或记账里没有 inode 号水位那一行：没有台阶行可判";

/// 一版记账里 inode 号水位那一统计量的台阶行 (g, H)（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」），从那一统计量
/// （不带设备维）的行 (代段, 值) 认：代段早于这一版 txg 的行都是台阶行；代段等于这一版 txg 的那一行（每次发布都写的那一行），值与代段最大的
/// 台阶行不同（或没有更早的台阶行）才是台阶行——这一版那次发布本身水位变了。按代段升序。与写路径同一条认法、按字段表另写一份，
/// 不用实现的读法。代段晚于这一版 txg 的行不认（I-3.13 判它）。
fn inode_number_watermark_steps_of_a_version(
    version_txg: u64,
    inode_number_watermark_rows: &[(u64, u64)],
) -> Vec<(u64, u64)> {
    let mut steps: Vec<(u64, u64)> = inode_number_watermark_rows
        .iter()
        .copied()
        .filter(|(row_generation, _)| *row_generation < version_txg)
        .collect();
    steps.sort_unstable();
    if let Some(&(_, value_of_this_version)) = inode_number_watermark_rows
        .iter()
        .find(|(row_generation, _)| *row_generation == version_txg)
    {
        if steps.last().is_none_or(|(_, latest_step_watermark)| {
            *latest_step_watermark != value_of_this_version
        }) {
            steps.push((version_txg, value_of_this_version));
        }
    }
    steps
}

/// I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，W3 的 C2aL 形态：D5（快照 / 空间记账机制） 已定项 2 射程、D18（块里携带什么信息）
/// 已定项 24 第 5 条）在最新的根上判，台阶行 (g, H) 取 [`inode_number_watermark_steps_of_a_version`]：
/// ① 记账里的 inode 号水位（那一统计量代段最大的那一行，D8（核心索引结构） 已定项 6「读现行值」）不小于每一级台阶的 H；
/// ② 最新根 inode 树里每条记录 (号 n, 对象出生代 b)、与最新根谱系上的每条墓碑 (n, b)：没有一级台阶 H > n 且 b > g
///    （号 n 在 g 之前就发过，晚于 g 出生的对象占着它就是同一条时间线上的重发）；
/// ④ 最新根谱系树里的每条谱系记录 (T_rb, T_old, H_cur)：代段 ≤ T_rb − 1 的台阶里最大的 H 等于 H_cur，没有这样的台阶判红；
///    H_cur 为 0 也判红——不另写一判：台阶的水位至少是第一个 inode 号 1（发布路径与只供测试的压回开关都不写 0），等式在 H_cur = 0 时不成立
///    （谱系接不成一条链时整份不信、报射程里判不了，同删除态扫描「在谱系上」那一判）。
/// 正推草案的 ③（水位大于初值时有一级台阶 H ≥ 水位、g ≤ 最新根的 txg）不判：台阶行与每次发布写的那一行同 key 维度、只差代段，
/// 那一行按上面的认法要么本身是台阶（H = 水位），要么前一级台阶的 H 就等于水位，③ 在单张镜像上恒成立。
/// 树表、谱系树、inode 树、墓碑树读不全的那一部分报射程里判不了（同 I-9.5 那一行的共用判法），已判的照报；
/// 只读不判、不进 `walk_failures`。
// TODO(D18（块里携带什么信息） 已定项 24 第 5 条「有克隆头时这一判与台阶行都按头」)：`crates/` 今天没有克隆路径，这一判不按树段分头；
// 做克隆时台阶行按 (统计量, 树) 分、记录与墓碑按头 ID 配那棵树的台阶。
fn judge_inode_number_watermark_steps_of_the_newest_root(
    reader: &dyn ImageReader,
    tree_table_pointer_of_the_newest_root: &[u8],
    newest_txg: u64,
    inode_number_watermark_rows: &[(u64, u64)],
    index_cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let invariant =
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE;
    let Some(&(_, inode_number_watermark)) = inode_number_watermark_rows
        .iter()
        .max_by_key(|(row_generation, _)| *row_generation)
    else {
        judgements.not_applicable(
            invariant,
            NO_INODE_NUMBER_WATERMARK_ROW_UNDER_THE_NEWEST_ROOT,
        );
        return;
    };
    let steps = inode_number_watermark_steps_of_a_version(newest_txg, inode_number_watermark_rows);
    // ①
    for (step_txg, step_watermark) in &steps {
        judgements.judge(invariant, *step_watermark <= inode_number_watermark, || {
            format!(
                "最新根（txg {newest_txg}）记账里的 inode 号水位 {inode_number_watermark} 小于 txg {step_txg} 那一级台阶的水位 {step_watermark}"
            )
        });
    }
    let tree_table_pointer = parse_node_pointer(tree_table_pointer_of_the_newest_root);
    if tree_table_pointer.all_zero {
        judgements.part_of_the_range_not_judged(
            invariant,
            DeletionScanGap::TreeTablePointerAllZero.reason(),
        );
        return;
    }
    let Some(tree_table) =
        read_index_node_without_judging(reader, &tree_table_pointer, index_cache)
    else {
        judgements
            .part_of_the_range_not_judged(invariant, DeletionScanGap::TreeTableUnreadable.reason());
        return;
    };
    if tree_table
        .entries
        .iter()
        .any(|entry| entry.len() < tree_table_entry_bytes())
    {
        judgements.part_of_the_range_not_judged(
            invariant,
            DeletionScanGap::TreeTableEntryShorterThanTheFieldTable.reason(),
        );
        return;
    }
    let entries_of_kind = |kind: u16| -> Vec<&Vec<u8>> {
        tree_table
            .entries
            .iter()
            .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == kind)
            .collect()
    };
    let mut packed_cache = PackedUnitCache::new();
    let lineage_records = match entries_of_kind(TREE_KIND_LINEAGE).as_slice() {
        [lineage_entry] => lineage_records_without_judging(
            reader,
            &parse_node_pointer(&lineage_entry[14..100]),
            index_cache,
            &mut packed_cache,
        ),
        [] | [_, _, ..] => Err(DeletionScanGap::LineageTreeEntryNotSingle),
    };
    // ④：谱系记录读得出、接得成一条链（同删除态扫描「在谱系上」那一判的前提：接不成链的谱系整份不信）才判。
    let lineage = lineage_records.and_then(|records| {
        lineage_chain_of_a_version(newest_txg, &records)
            .map(|lineage_chain| (records, lineage_chain))
    });
    match &lineage {
        Ok((records, _)) => {
            for record in records {
                let watermark_rolled_back_from =
                    record.inode_number_watermark_of_the_version_rolled_back_from;
                let largest_step_before_the_rollback = steps
                    .iter()
                    .filter(|(step_txg, _)| *step_txg < record.rollback_txg)
                    .map(|(_, step_watermark)| *step_watermark)
                    .max();
                judgements.judge(
                    invariant,
                    largest_step_before_the_rollback == Some(watermark_rolled_back_from),
                    || {
                        format!(
                            "最新根（txg {newest_txg}）谱系记录 (T_rb {}, T_old {}) 带的 H_cur {watermark_rolled_back_from} 不等于代段 ≤ T_rb − 1 的台阶里最大的水位 {largest_step_before_the_rollback:?}（H_cur 为 0 也算不等）",
                            record.rollback_txg, record.parent_txg
                        )
                    },
                );
            }
        }
        Err(gap) => judgements.part_of_the_range_not_judged(invariant, gap.reason()),
    }
    // ②
    let step_the_number_was_issued_before = |number: u64, object_birth: u64| {
        steps
            .iter()
            .find(|(step_txg, step_watermark)| *step_watermark > number && object_birth > *step_txg)
            .copied()
    };
    for inode_tree_entry in entries_of_kind(TREE_KIND_INODE) {
        match inode_records_without_judging(
            reader,
            &parse_node_pointer(&inode_tree_entry[14..100]),
            index_cache,
            &mut packed_cache,
        ) {
            Ok(records) => {
                for (inode, record) in &records {
                    let step = step_the_number_was_issued_before(*inode, record.object_birth);
                    judgements.judge(invariant, step.is_none(), || {
                        format!(
                            "最新根（txg {newest_txg}）inode 树里 {inode} 号的对象出生代 {}，晚于台阶 {step:?}（txg, 水位）：那一级台阶说这个号在那之前就发过",
                            record.object_birth
                        )
                    });
                }
            }
            Err(gap) => judgements.part_of_the_range_not_judged(invariant, gap.reason()),
        }
    }
    for tombstone_tree_entry in entries_of_kind(TREE_KIND_TOMBSTONE) {
        let containers = tombstone_containers_without_judging(
            reader,
            &parse_node_pointer(&tombstone_tree_entry[14..100]),
            index_cache,
            &mut packed_cache,
        );
        match (containers, &lineage) {
            (Ok(containers), Ok((_, lineage_chain))) => {
                for tombstone in containers
                    .iter()
                    .flat_map(|container| container.records.iter())
                    .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
                {
                    let step =
                        step_the_number_was_issued_before(tombstone.object, tombstone.object_birth);
                    judgements.judge(invariant, step.is_none(), || {
                        format!(
                            "最新根（txg {newest_txg}）谱系上的墓碑（{} 号，对象出生代 {}，死于 txg {}）晚于台阶 {step:?}（txg, 水位）出生：那一级台阶说这个号在那之前就发过",
                            tombstone.object, tombstone.object_birth, tombstone.death_txg
                        )
                    });
                }
            }
            (Err(gap), _) => judgements.part_of_the_range_not_judged(invariant, gap.reason()),
            (Ok(_), Err(gap)) => judgements.part_of_the_range_not_judged(invariant, gap.reason()),
        }
    }
}

/// 墓碑与谱系那几条（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节与第四节用户定案）：删除态扫描那一份版本清单里
/// 每一版（树表与 txg 都相同的几版只扫一次——「在谱系上」从这一版自己的 txg 起算，txg 不同就各扫一遍）重读树表、谱系树、墓碑树、
/// inode 树与 `deleted_inodes`，只读不判、不进 `walk_failures`：判 I-9.18、I-9.19、I-9.20、I-9.21 / I-9.22（看装没装旋钮）与 I-9.10 墓碑那一个等号，
/// 把各版的号 → 对象出生代收进一张跨版本的表，全部扫完判 I-9.29；候选集滤不掉被抛弃的根（[`AbandonedRootsFilteringOfTheCandidateSet`]）时
/// 那张表一个号都不比、I-9.29 报部分没判。读不全、配不上的报射程里判不了（同 I-9.5 那一行的共用判法）；
/// 不按已发布与否滤墓碑（第二节 T4）。I-9.10 墓碑那一个等号读不全、配不上时不记判不了，与它数据单元那一半同口径
/// （C701（别的判法静默跳过读不出的部分））。
fn judge_tombstones_and_the_lineage_of_the_versions(
    reader: &dyn ImageReader,
    versions: &[VersionToScanTheDeletionStateOf<'_>],
    abandoned_roots_filtering: AbandonedRootsFilteringOfTheCandidateSet,
    container_capacity_knobs: ContainerCapacityKnobs,
    index_cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let capacity_knob = container_capacity_knobs.tombstone;
    let lineage_and_retirement_capacity_invariants = [
        lineage_or_retirement_capacity_invariant_of(
            container_capacity_knobs
                .lineage
                .map(LineageContainerCapacityKnob::records_per_container),
        ),
        lineage_or_retirement_capacity_invariant_of(
            container_capacity_knobs
                .retirement
                .map(RetirementContainerCapacityKnob::records_per_container),
        ),
    ];
    let mut packed_cache = PackedUnitCache::new();
    let mut tree_tables_and_txgs_scanned: BTreeSet<(u32, u64, u32, u64)> = BTreeSet::new();
    let mut object_births: ObjectBirthsOfEachInodeNumber = BTreeMap::new();
    let mut versions_of_the_retirement_completeness: Vec<VersionOfTheRetirementCompleteness> =
        Vec::new();
    let capacity_invariant = capacity_invariant_of(capacity_knob);
    let unreadable_version = |version: &VersionToScanTheDeletionStateOf<'_>,
                              gap: DeletionScanGap| {
        VersionOfTheRetirementCompleteness {
            name: version.name.clone(),
            checkpoint_txg: version.checkpoint_txg,
            parent_txg: Err(gap),
            leaf_identities_of_each_inode_tree: BTreeMap::new(),
            retired_at_this_txg_of_each_inode_tree: BTreeMap::new(),
            unreadable: Some(gap),
        }
    };
    for version in versions {
        let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
        if tree_table_pointer.all_zero {
            for invariant in TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS
                .into_iter()
                .chain(RETIREMENT_SCAN_INVARIANTS)
                .chain([capacity_invariant])
                .chain(lineage_and_retirement_capacity_invariants)
            {
                judgements.part_of_the_range_not_judged(
                    invariant,
                    DeletionScanGap::TreeTablePointerAllZero.reason(),
                );
            }
            versions_of_the_retirement_completeness.push(unreadable_version(
                version,
                DeletionScanGap::TreeTablePointerAllZero,
            ));
            continue;
        }
        let first_location = tree_table_pointer.locations[0];
        if !tree_tables_and_txgs_scanned.insert((
            first_location.device,
            first_location.slot,
            first_location.checksum,
            version.checkpoint_txg,
        )) {
            continue;
        }
        // 树表读不出、条目比字段表窄：这一版的墓碑与谱系判不了；树表 0 条的一版没有这几棵树，不判。
        let tree_table = read_index_node_without_judging(reader, &tree_table_pointer, index_cache);
        let tree_table_gap = if let Some(tree_table_read_back) = &tree_table {
            tree_table_read_back
                .entries
                .iter()
                .any(|entry| entry.len() < tree_table_entry_bytes())
                .then_some(DeletionScanGap::TreeTableEntryShorterThanTheFieldTable)
        } else {
            Some(DeletionScanGap::TreeTableUnreadable)
        };
        if let Some(gap) = tree_table_gap {
            for invariant in TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS
                .into_iter()
                .chain(RETIREMENT_SCAN_INVARIANTS)
                .chain([capacity_invariant])
                .chain(lineage_and_retirement_capacity_invariants)
            {
                judgements.part_of_the_range_not_judged(invariant, gap.reason());
            }
            versions_of_the_retirement_completeness.push(unreadable_version(version, gap));
            continue;
        }
        let Some(tree_table) = tree_table else {
            continue;
        };
        if tree_table.entries.is_empty() {
            // 树表 0 条的一版没有 inode 树：当父版时它每棵树的叶身份集都是空集。
            versions_of_the_retirement_completeness.push(VersionOfTheRetirementCompleteness {
                name: version.name.clone(),
                checkpoint_txg: version.checkpoint_txg,
                parent_txg: Ok(None),
                leaf_identities_of_each_inode_tree: BTreeMap::new(),
                retired_at_this_txg_of_each_inode_tree: BTreeMap::new(),
                unreadable: None,
            });
            continue;
        }
        versions_of_the_retirement_completeness.push(
            judge_tombstones_and_the_lineage_of_a_version(
                reader,
                version,
                &tree_table.entries,
                capacity_knob,
                container_capacity_knobs,
                index_cache,
                &mut packed_cache,
                &mut object_births,
                judgements,
            ),
        );
    }
    judge_retirement_records_account_for_every_leaf_gone_since_the_parent_version(
        &versions_of_the_retirement_completeness,
        abandoned_roots_filtering,
        judgements,
    );
    match abandoned_roots_filtering {
        AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable => {
            judge_one_object_birth_per_inode_number(&object_births, judgements);
        }
        AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable => {
            judgements.partially_judged(
                ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
                format!(
                    "最新根的实例表读不出（主走读沿链没走到末片：有一片读不出、解不开或链指针记录不合法）：回退候选集里滤不掉被崩溃恢复抛弃的根，收到的 {} 个号一个都没比",
                    object_births.len()
                ),
            );
        }
    }
    // 一次都没判到时的理由（判到过的照判定报，`Judgements::not_applicable` 不改已经评估过的）。
    judgements.not_applicable(
        QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
        NO_QUEUED_INODE_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
        NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD,
    );
    judgements.not_applicable(
        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
        NO_TOMBSTONE_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
        NO_INODE_RECORD_OR_TOMBSTONE_ON_THE_LINEAGE_TO_COMPARE,
    );
    judgements.not_applicable(
        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
        NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
        NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
        NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
        NO_VERSION_WHOSE_PARENT_IS_IN_THE_CANDIDATE_SET,
    );
    match capacity_knob {
        None => {
            judgements.not_applicable(
                NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
                NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER,
            );
            judgements.not_applicable(
                TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
                NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY,
            );
        }
        Some(_) => {
            judgements.not_applicable(
                NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
                PRODUCT_TOMBSTONE_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOB,
            );
            judgements.not_applicable(
                TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
                NO_TOMBSTONE_CONTAINER_UNDER_THE_KNOB,
            );
        }
    }
    let (product_capacity_reason, knob_capacity_reason) = match (
        container_capacity_knobs.lineage,
        container_capacity_knobs.retirement,
    ) {
        (None, None) => (
            NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER,
            NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB,
        ),
        (Some(_), None) | (None, Some(_)) => (
            NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER,
            NO_LINEAGE_OR_RETIREMENT_CONTAINER_UNDER_THE_KNOBS,
        ),
        (Some(_), Some(_)) => (
            PRODUCT_LINEAGE_AND_RETIREMENT_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOBS,
            NO_LINEAGE_OR_RETIREMENT_CONTAINER_UNDER_THE_KNOBS,
        ),
    };
    judgements.not_applicable(
        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        product_capacity_reason,
    );
    judgements.not_applicable(
        LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
        knob_capacity_reason,
    );
}

/// [`judge_tombstones_and_the_lineage_of_the_versions`] 的一版：读谱系（父链）、每棵 inode 树的记录与每棵墓碑树，再逐条判。
#[allow(
    clippy::too_many_arguments,
    reason = "版本、树表条目、墓碑旋钮与三棵容器树的旋钮、两份缓存、跨版本那张表与判定累加器各有名字与类型，不为凑短签名收成结构体"
)]
fn judge_tombstones_and_the_lineage_of_a_version(
    reader: &dyn ImageReader,
    version: &VersionToScanTheDeletionStateOf<'_>,
    tree_table_entries: &[Vec<u8>],
    capacity_knob: Option<TombstoneContainerCapacityKnob>,
    container_capacity_knobs: ContainerCapacityKnobs,
    index_cache: &mut IndexNodeCache,
    packed_cache: &mut PackedUnitCache,
    object_births: &mut ObjectBirthsOfEachInodeNumber,
    judgements: &mut Judgements,
) -> VersionOfTheRetirementCompleteness {
    let entries_of_kind = |kind: u16| -> Vec<&Vec<u8>> {
        tree_table_entries
            .iter()
            .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == kind)
            .collect()
    };
    // 一张树表恰好一条谱系树条目（I-9.16 ④）：这一版的父链从它的记录与这一版自己的 txg 起算。
    let lineage_tree = match entries_of_kind(TREE_KIND_LINEAGE).as_slice() {
        [lineage_entry] => lineage_containers_without_judging(
            reader,
            &parse_node_pointer(&lineage_entry[14..100]),
            index_cache,
            packed_cache,
        )
        .map(|containers| (read_u64(lineage_entry, 0), containers)),
        [] | [_, _, ..] => Err(DeletionScanGap::LineageTreeEntryNotSingle),
    };
    // I-9.36 / I-9.37 在谱系树上（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2）：读不全的报射程里判不了。
    let lineage_capacity_knob = container_capacity_knobs
        .lineage
        .map(LineageContainerCapacityKnob::records_per_container);
    match &lineage_tree {
        Ok((lineage_tree_identifier, containers)) => {
            judge_lineage_or_retirement_container_capacity_of_a_version(
                &version.name,
                LineageOrRetirementTree::Lineage,
                *lineage_tree_identifier,
                &containers
                    .iter()
                    .map(|container| (container.container, container.records.len()))
                    .collect::<Vec<(u64, usize)>>(),
                lineage_capacity_knob,
                judgements,
            );
        }
        Err(gap) => judgements.part_of_the_range_not_judged(
            lineage_or_retirement_capacity_invariant_of(lineage_capacity_knob),
            gap.reason(),
        ),
    }
    let lineage_records = lineage_tree.map(|(_, containers)| {
        containers
            .into_iter()
            .flat_map(|container| container.records)
            .collect::<Vec<LineageRecordOfTheScan>>()
    });
    let lineage_chain = lineage_records
        .clone()
        .and_then(|records| lineage_chain_of_a_version(version.checkpoint_txg, &records));
    // I-9.35 的父版：这一版谱系树里有 T_rb 等于这一版 txg 的记录时取它的 T_old，否则 txg − 1（正推 K5.1）。
    let parent_txg = lineage_records.as_ref().map_err(|gap| *gap).map(|records| {
        records
            .iter()
            .find(|record| record.rollback_txg == version.checkpoint_txg)
            .map_or(version.checkpoint_txg.checked_sub(1), |record| {
                Some(record.parent_txg)
            })
    });
    let mut inode_trees: Vec<(u64, InodeRecordsOfAnInodeTreeOfTheScan)> = Vec::new();
    for inode_tree_entry in entries_of_kind(TREE_KIND_INODE) {
        inode_trees.push((
            read_u64(inode_tree_entry, 0),
            inode_records_without_judging(
                reader,
                &parse_node_pointer(&inode_tree_entry[14..100]),
                index_cache,
                packed_cache,
            ),
        ));
    }
    let mut tombstone_trees: Vec<TombstoneTreeOfTheScan> = Vec::new();
    for tombstone_tree_entry in entries_of_kind(TREE_KIND_TOMBSTONE) {
        tombstone_trees.push(TombstoneTreeOfTheScan {
            tree: read_u64(tombstone_tree_entry, 0),
            head_identifier: read_u64(
                tombstone_tree_entry,
                TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
            ),
            containers: tombstone_containers_without_judging(
                reader,
                &parse_node_pointer(&tombstone_tree_entry[14..100]),
                index_cache,
                packed_cache,
            ),
        });
    }
    for (inode_tree, records) in &inode_trees {
        match records {
            Ok(records) => {
                for (inode, record) in records {
                    object_births
                        .entry((*inode_tree, *inode))
                        .or_default()
                        .entry(record.object_birth)
                        .or_insert_with(|| format!("{}的 inode 记录", version.name));
                }
            }
            Err(gap) => {
                judgements
                    .part_of_the_range_not_judged(ONE_OBJECT_BIRTH_PER_INODE_NUMBER, gap.reason());
            }
        }
    }
    for tombstone_tree in &tombstone_trees {
        let containers = match &tombstone_tree.containers {
            Ok(containers) => containers,
            Err(gap) => {
                for invariant in TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS
                    .into_iter()
                    .chain([capacity_invariant_of(capacity_knob)])
                {
                    judgements.part_of_the_range_not_judged(invariant, gap.reason());
                }
                continue;
            }
        };
        judge_tombstone_container_capacity_of_a_version(
            &version.name,
            tombstone_tree.tree,
            containers,
            capacity_knob,
            judgements,
        );
        judge_every_tombstone_of_a_version_is_well_formed(
            version,
            tombstone_tree.tree,
            containers,
            judgements,
        );
        match &lineage_chain {
            Ok(lineage_chain) => judge_tombstones_on_the_lineage_of_a_version_are_distinct(
                &version.name,
                tombstone_tree.tree,
                containers,
                lineage_chain,
                judgements,
            ),
            Err(gap) => judgements.part_of_the_range_not_judged(
                TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
                gap.reason(),
            ),
        }
        // 墓碑树经头 ID 配同一张表里树 ID 等于它的那一棵 inode 树（同 extent 树，I-9.27）：配不上、或那棵树的记录读不全的，墓碑对记录那几判判不了。
        let paired_inode_trees: Vec<&(u64, InodeRecordsOfAnInodeTreeOfTheScan)> = inode_trees
            .iter()
            .filter(|(inode_tree, _)| *inode_tree == tombstone_tree.head_identifier)
            .collect();
        let records = match paired_inode_trees.as_slice() {
            [(_, Ok(records))] => records,
            [(_, Err(gap))] => {
                for invariant in [
                    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
                    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
                ] {
                    judgements.part_of_the_range_not_judged(invariant, gap.reason());
                }
                continue;
            }
            [] | [_, _, ..] => {
                for invariant in [
                    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
                    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
                ] {
                    judgements.part_of_the_range_not_judged(
                        invariant,
                        DeletionScanGap::TombstoneTreeHeadIdentifierNamesNoSingleInodeTree.reason(),
                    );
                }
                continue;
            }
        };
        judge_tombstones_against_the_object_births_of_the_records_by_number(
            &version.name,
            containers,
            records,
            judgements,
        );
        match &lineage_chain {
            Ok(lineage_chain) => {
                judge_tombstones_on_the_lineage_name_no_live_object(
                    &version.name,
                    containers,
                    records,
                    lineage_chain,
                    judgements,
                );
                for tombstone in containers
                    .iter()
                    .flat_map(|container| container.records.iter())
                    .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
                {
                    object_births
                        .entry((tombstone_tree.head_identifier, tombstone.object))
                        .or_default()
                        .entry(tombstone.object_birth)
                        .or_insert_with(|| {
                            format!(
                                "{}墓碑树里谱系上的墓碑（死于 txg {}）",
                                version.name, tombstone.death_txg
                            )
                        });
                }
            }
            Err(gap) => {
                for invariant in [
                    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
                    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
                ] {
                    judgements.part_of_the_range_not_judged(invariant, gap.reason());
                }
            }
        }
    }
    for (inode_tree, records) in &inode_trees {
        judge_queued_inodes_of_an_inode_tree_have_tombstones_on_the_lineage(
            reader,
            version,
            tree_table_entries,
            *inode_tree,
            records,
            &tombstone_trees,
            &lineage_chain,
            index_cache,
            judgements,
        );
    }
    // 退役树那几条（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条）：I-9.32 每条记录都判；I-9.33、I-9.34 只判死亡 txg
    // 在这一版父链上的；I-9.35 要的那几样收下来，扫完各版再跨相邻两版比。
    let mut leaf_identities_of_each_inode_tree = BTreeMap::new();
    for inode_tree_entry in entries_of_kind(TREE_KIND_INODE) {
        leaf_identities_of_each_inode_tree.insert(
            read_u64(inode_tree_entry, 0),
            inode_leaf_identities_without_judging(
                reader,
                &parse_node_pointer(&inode_tree_entry[14..100]),
                index_cache,
            ),
        );
    }
    let mut retired_at_this_txg_of_each_inode_tree = BTreeMap::new();
    let retirement_capacity_knob = container_capacity_knobs
        .retirement
        .map(RetirementContainerCapacityKnob::records_per_container);
    for retirement_tree_entry in entries_of_kind(TREE_KIND_RETIREMENT) {
        let retirement_tree = RetirementTreeOfTheScan {
            tree: read_u64(retirement_tree_entry, 0),
            head_identifier: read_u64(
                retirement_tree_entry,
                TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
            ),
            containers: retirement_containers_without_judging(
                reader,
                &parse_node_pointer(&retirement_tree_entry[14..100]),
                index_cache,
                packed_cache,
            ),
        };
        let containers = match &retirement_tree.containers {
            Ok(containers) => containers,
            Err(gap) => {
                for invariant in RETIREMENT_SCAN_INVARIANTS.into_iter().chain([
                    lineage_or_retirement_capacity_invariant_of(retirement_capacity_knob),
                ]) {
                    judgements.part_of_the_range_not_judged(invariant, gap.reason());
                }
                retired_at_this_txg_of_each_inode_tree
                    .insert(retirement_tree.head_identifier, Err(*gap));
                continue;
            }
        };
        judge_lineage_or_retirement_container_capacity_of_a_version(
            &version.name,
            LineageOrRetirementTree::Retirement,
            retirement_tree.tree,
            &containers
                .iter()
                .map(|container| (container.container, container.records.len()))
                .collect::<Vec<(u64, usize)>>(),
            retirement_capacity_knob,
            judgements,
        );
        judge_every_retirement_record_of_a_version_is_well_formed(
            version,
            retirement_tree.tree,
            containers,
            judgements,
        );
        retired_at_this_txg_of_each_inode_tree.insert(
            retirement_tree.head_identifier,
            Ok(containers
                .iter()
                .flat_map(|container| container.records.iter())
                .filter(|record| record.death_txg == version.checkpoint_txg)
                .map(|record| record.retired)
                .collect::<BTreeSet<ContainerIdentityOfTheScan>>()),
        );
        let lineage_chain = match &lineage_chain {
            Ok(lineage_chain) => lineage_chain,
            Err(gap) => {
                for invariant in [
                    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
                    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
                ] {
                    judgements.part_of_the_range_not_judged(invariant, gap.reason());
                }
                continue;
            }
        };
        judge_retirement_records_on_the_lineage_of_a_version_are_distinct(
            &version.name,
            retirement_tree.tree,
            containers,
            lineage_chain,
            judgements,
        );
        // 退役树经头 ID 配同一张表里树 ID 等于它的那一棵 inode 树（同墓碑树，I-9.27）：配不上、或那棵树读不全的，I-9.34 判不了。
        let paired_leaf_identities: Vec<
            &Result<BTreeSet<ContainerIdentityOfTheScan>, DeletionScanGap>,
        > = leaf_identities_of_each_inode_tree
            .iter()
            .filter(|(inode_tree, _)| **inode_tree == retirement_tree.head_identifier)
            .map(|(_, identities)| identities)
            .collect();
        match paired_leaf_identities.as_slice() {
            [Ok(leaf_identities)] => judge_retirement_records_on_the_lineage_name_no_live_leaf(
                &version.name,
                retirement_tree.tree,
                containers,
                leaf_identities,
                lineage_chain,
                judgements,
            ),
            [Err(gap)] => judgements.part_of_the_range_not_judged(
                RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
                gap.reason(),
            ),
            [] | [_, _, ..] => judgements.part_of_the_range_not_judged(
                RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
                DeletionScanGap::RetirementTreeHeadIdentifierNamesNoSingleInodeTree.reason(),
            ),
        }
    }
    // 有一棵退役树的头 ID 配不上这张表里任何一棵 inode 树（I-9.27 判它）：它的记录算不到哪棵 inode 树头上，这一版每棵 inode 树的
    // 退役集合都判不了——I-9.35 在这一版报射程里判不了，不按「没有退役记录」判漏记。
    let some_retirement_tree_names_no_inode_tree = retired_at_this_txg_of_each_inode_tree
        .keys()
        .any(|head_identifier| !leaf_identities_of_each_inode_tree.contains_key(head_identifier));
    if some_retirement_tree_names_no_inode_tree {
        for inode_tree in leaf_identities_of_each_inode_tree.keys() {
            retired_at_this_txg_of_each_inode_tree.insert(
                *inode_tree,
                Err(DeletionScanGap::RetirementTreeHeadIdentifierNamesNoSingleInodeTree),
            );
        }
    }
    VersionOfTheRetirementCompleteness {
        name: version.name.clone(),
        checkpoint_txg: version.checkpoint_txg,
        parent_txg,
        leaf_identities_of_each_inode_tree,
        retired_at_this_txg_of_each_inode_tree,
        unreadable: None,
    }
}

/// I-9.18（删除队列里的号有谱系上的墓碑）在一版的一棵 inode 树上：经头 ID 配这棵树的 `deleted_inodes`（恰好一条）与墓碑树（恰好一条），
/// 队列里每个号 N 要在墓碑树里有 (N, N 的记录的对象出生代, 整对象) 且死亡 txg 在这一版的父链上；N 没有记录的那一形判不了、不判红
/// （`.claude/kb/invariants.md` I-9.18 那一行：这一形只让 I-9.5 红）。队列空的一版没有对象。
#[allow(
    clippy::too_many_arguments,
    reason = "版本、树表条目、这棵 inode 树与它的记录、墓碑树、父链、缓存与判定累加器各有名字与类型，不为凑短签名收成结构体"
)]
fn judge_queued_inodes_of_an_inode_tree_have_tombstones_on_the_lineage(
    reader: &dyn ImageReader,
    version: &VersionToScanTheDeletionStateOf<'_>,
    tree_table_entries: &[Vec<u8>],
    inode_tree: u64,
    records: &InodeRecordsOfAnInodeTreeOfTheScan,
    tombstone_trees: &[TombstoneTreeOfTheScan],
    lineage_chain: &Result<LineageChainOfAVersion, DeletionScanGap>,
    index_cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let invariant = QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE;
    let deleted_inodes_entries: Vec<&Vec<u8>> = tree_table_entries
        .iter()
        .filter(|entry| {
            read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_DELETED_INODES
                && read_u64(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET) == inode_tree
        })
        .collect();
    let queue = match deleted_inodes_entries.as_slice() {
        [deleted_inodes_entry] => deleted_inodes_queue_without_judging(
            reader,
            &parse_node_pointer(&deleted_inodes_entry[14..100]),
            index_cache,
        ),
        [] | [_, _, ..] => Err(DeletionScanGap::InodeTreeNamedByNoSingleDeletedInodesEntry),
    };
    let queue = match queue {
        Ok(queue) => queue,
        Err(gap) => {
            judgements.part_of_the_range_not_judged(invariant, gap.reason());
            return;
        }
    };
    if queue.is_empty() {
        return;
    }
    let records = match records {
        Ok(records) => records,
        Err(gap) => {
            judgements.part_of_the_range_not_judged(invariant, gap.reason());
            return;
        }
    };
    let paired_tombstone_trees: Vec<&TombstoneTreeOfTheScan> = tombstone_trees
        .iter()
        .filter(|tombstone_tree| tombstone_tree.head_identifier == inode_tree)
        .collect();
    let (tombstone_tree, containers) = match paired_tombstone_trees.as_slice() {
        [tombstone_tree] => match &tombstone_tree.containers {
            Ok(containers) => (tombstone_tree.tree, containers),
            Err(gap) => {
                judgements.part_of_the_range_not_judged(invariant, gap.reason());
                return;
            }
        },
        [] | [_, _, ..] => {
            judgements.part_of_the_range_not_judged(
                invariant,
                DeletionScanGap::InodeTreeNamedByNoSingleTombstoneTreeEntry.reason(),
            );
            return;
        }
    };
    let lineage_chain = match lineage_chain {
        Ok(lineage_chain) => lineage_chain,
        Err(gap) => {
            judgements.part_of_the_range_not_judged(invariant, gap.reason());
            return;
        }
    };
    for inode in &queue {
        let Some(record) = records.get(inode) else {
            judgements.part_of_the_range_not_judged(
                invariant,
                DeletionScanGap::QueuedInodeWithoutARecord.reason(),
            );
            continue;
        };
        let tombstones_of_the_inode: Vec<TombstoneRecordOfTheScan> = containers
            .iter()
            .flat_map(|container| container.records.iter())
            .filter(|tombstone| tombstone.object == *inode)
            .copied()
            .collect();
        let has_a_whole_object_tombstone_on_the_lineage =
            tombstones_of_the_inode.iter().any(|tombstone| {
                tombstone.object_birth == record.object_birth
                    && tombstone.covers_the_whole_object()
                    && lineage_chain.holds_txg(tombstone.death_txg)
            });
        judgements.judge(invariant, has_a_whole_object_tombstone_on_the_lineage, || {
            format!(
                "{}：deleted_inodes 里的号 {inode}（记录的对象出生代 {}）在墓碑树（树 {tombstone_tree}）里没有在谱系上的整对象墓碑；同号的墓碑 {tombstones_of_the_inode:?}",
                version.name, record.object_birth
            )
        });
    }
}

/// I-9.10（对象出生代三处一致） 墓碑那一个等号（判决第二节 T2 取 a）：这一版墓碑树里每一条墓碑按号认，这一版 inode 树里有这个号的记录时，
/// 墓碑的对象出生代 == 记录的。不看谱系（回退到删之前那一形的墓碑在谱系外，也比），删完之后记录不在、这一判没有对象。
fn judge_tombstones_against_the_object_births_of_the_records_by_number(
    version_name: &str,
    containers: &[TombstoneContainerOfTheScan],
    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
    judgements: &mut Judgements,
) {
    for tombstone in containers
        .iter()
        .flat_map(|container| container.records.iter())
    {
        let Some(record) = records.get(&tombstone.object) else {
            continue;
        };
        judgements.judge("I-9.10", record.object_birth == tombstone.object_birth, || {
            format!(
                "{version_name}：墓碑（号 {}、对象出生代 {}、死于 txg {}）与这一版 inode 记录的对象出生代 {} 不符",
                tombstone.object, tombstone.object_birth, tombstone.death_txg, record.object_birth
            )
        });
    }
}

/// I-9.19（谱系上的墓碑不指活对象）：这一版在谱系上的每条墓碑，这一版 inode 树里有号与对象出生代都相同的记录时，那条记录 nlink = 0。
fn judge_tombstones_on_the_lineage_name_no_live_object(
    version_name: &str,
    containers: &[TombstoneContainerOfTheScan],
    records: &BTreeMap<u64, InodeRecordOfTheDeletionScan>,
    lineage_chain: &LineageChainOfAVersion,
    judgements: &mut Judgements,
) {
    for tombstone in containers
        .iter()
        .flat_map(|container| container.records.iter())
        .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
    {
        let Some(record) = records
            .get(&tombstone.object)
            .filter(|record| record.object_birth == tombstone.object_birth)
        else {
            continue;
        };
        judgements.judge(
            TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
            record.nlink == 0,
            || {
                format!(
                    "{version_name}：谱系上的墓碑（号 {}、对象出生代 {}、死于 txg {}）指着 nlink {} 的记录",
                    tombstone.object, tombstone.object_birth, tombstone.death_txg, record.nlink
                )
            },
        );
    }
}

/// I-9.20（谱系上的墓碑不重号且记录合法） 管全部墓碑的那几句（判决第二节 T14、第四节用户定案「补」）：每条整对象 [0, 2⁶⁴ − 1]
/// （区间起 < 区间止是它的推论，照条款原文合取）、对象出生代 ≤ 死亡 txg ≤ 这一版的 txg；每片容器记录数 ≤ 815（与 I-1.7 重合，留作合取）。
fn judge_every_tombstone_of_a_version_is_well_formed(
    version: &VersionToScanTheDeletionStateOf<'_>,
    tombstone_tree: u64,
    containers: &[TombstoneContainerOfTheScan],
    judgements: &mut Judgements,
) {
    let invariant = TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED;
    let version_name = &version.name;
    let version_txg = version.checkpoint_txg;
    for container in containers {
        let record_count =
            u64::try_from(container.records.len()).expect("一片容器的记录数装得进 u64");
        judgements.judge(invariant, record_count <= TOMBSTONE_CONTAINER_RECORDS, || {
            format!(
                "{version_name}：墓碑树（树 {tombstone_tree}）的容器 {} 装了 {record_count} 条，多于 {TOMBSTONE_CONTAINER_RECORDS}",
                container.container
            )
        });
        for tombstone in &container.records {
            judgements.judge(invariant, tombstone.covers_the_whole_object(), || {
                format!(
                    "{version_name}：墓碑（号 {}、对象出生代 {}）的区间 [{}, {}] 不是整对象 [0, 2⁶⁴ − 1]",
                    tombstone.object, tombstone.object_birth, tombstone.range_start, tombstone.range_end
                )
            });
            let death_txg_in_range =
                tombstone.object_birth <= tombstone.death_txg && tombstone.death_txg <= version_txg;
            judgements.judge(invariant, death_txg_in_range, || {
                format!(
                    "{version_name}：墓碑（号 {}、对象出生代 {}）的死亡 txg {} 不在 [对象出生代, 这一版的 txg {version_txg}] 里",
                    tombstone.object, tombstone.object_birth, tombstone.death_txg
                )
            });
        }
    }
}

/// I-9.20 不重号那一句（只判在谱系上的，判决第二节 T14）：这一版在谱系上的墓碑按 (号, 对象出生代) 不重复。
fn judge_tombstones_on_the_lineage_of_a_version_are_distinct(
    version_name: &str,
    tombstone_tree: u64,
    containers: &[TombstoneContainerOfTheScan],
    lineage_chain: &LineageChainOfAVersion,
    judgements: &mut Judgements,
) {
    let mut objects_on_the_lineage: BTreeSet<(u64, u64)> = BTreeSet::new();
    for tombstone in containers
        .iter()
        .flat_map(|container| container.records.iter())
        .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
    {
        let is_the_first_on_the_lineage =
            objects_on_the_lineage.insert((tombstone.object, tombstone.object_birth));
        judgements.judge(
            TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
            is_the_first_on_the_lineage,
            || {
                format!(
                    "{version_name}：墓碑树（树 {tombstone_tree}）里在谱系上的墓碑 (号 {}、对象出生代 {}) 不止一条（又一条死于 txg {}）",
                    tombstone.object, tombstone.object_birth, tombstone.death_txg
                )
            },
        );
    }
}

/// I-9.36 / I-9.37（第 53 批，判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2，与 I-9.21 / I-9.22 同形）：这一版谱系树或
/// 退役树按叶序的容器（容器号, 记录数），读容器头、不沿根的条目读。有对象时（没装旋钮至少两片，装了至少一片）判容器号与记录数的三句
/// （第 56 批 G2，判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节）：沿叶序第一片容器号是 1、相邻两片加一、
/// 每片（含最右那片）至少一条——写侧只经两个追加函数开新片，新号 = 最右号 + 1、零根时 1，有新记录才开、没有删片的路
/// （D18（块里携带什么信息） 已定项 24 第 1、5 条）；再按旋钮：没装判 I-9.36——最右那片之外每片记录数 == 产品容量
/// （谱系 1359、退役 959；开放容器满了才关），零根或只一片容器没有对象；装了旋钮判 I-9.37——每片记录数 ≤ 旋钮值、最右那片之外每片记录数
/// == 旋钮值（满了才关在旋钮下的那一半，D18（块里携带什么信息） 已定项 24 第 3 条「满了才关；关闭路径加只供测试的容量旋钮」，第 57 批 H1，
/// 判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节），零根没有对象。
fn judge_lineage_or_retirement_container_capacity_of_a_version(
    version_name: &str,
    tree_kind: LineageOrRetirementTree,
    tree: u64,
    container_numbers_and_record_counts: &[(u64, usize)],
    knob_records_per_container: Option<u64>,
    judgements: &mut Judgements,
) {
    let invariant = lineage_or_retirement_capacity_invariant_of(knob_records_per_container);
    let tree_name = tree_kind.name();
    let judged_when_the_tree_has_at_least = match knob_records_per_container {
        None => 2,
        Some(_) => 1,
    };
    if container_numbers_and_record_counts.len() < judged_when_the_tree_has_at_least {
        return;
    }
    if let Some((first_container_in_leaf_order, _)) = container_numbers_and_record_counts.first() {
        let first_container_in_leaf_order = *first_container_in_leaf_order;
        judgements.judge(
            invariant,
            first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,
            || {
                format!(
                    "{version_name}：{tree_name}（树 {tree}）沿叶序第一片容器的容器号是 {first_container_in_leaf_order}，不是 {FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE}（容器号从 1 起）"
                )
            },
        );
    }
    for adjacent in container_numbers_and_record_counts.windows(2) {
        let (left_container, right_container) = (adjacent[0].0, adjacent[1].0);
        let right_container_follows_the_left_by_one =
            left_container.checked_add(1) == Some(right_container);
        judgements.judge(invariant, right_container_follows_the_left_by_one, || {
            format!(
                "{version_name}：{tree_name}（树 {tree}）沿叶序容器号不是逐片加一：容器 {left_container} 之后是容器 {right_container}（下一个号 = 最右号 + 1）"
            )
        });
    }
    for (position_in_leaf_order, (container, record_count)) in
        container_numbers_and_record_counts.iter().enumerate()
    {
        let record_count = u64::try_from(*record_count).expect("一片容器的记录数装得进 u64");
        let lineage_or_retirement_container_holds_a_record = record_count > 0;
        judgements.judge(invariant, lineage_or_retirement_container_holds_a_record, || {
            format!(
                "{version_name}：{tree_name}（树 {tree}）的容器 {container} 一条记录都没有（有新记录才开新片）"
            )
        });
        match knob_records_per_container {
            None => {
                let is_the_rightmost =
                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();
                if is_the_rightmost {
                    continue;
                }
                let product_records = tree_kind.product_records_per_container();
                judgements.judge(invariant, record_count == product_records, || {
                    format!(
                        "{version_name}：{tree_name}（树 {tree}）里不是最右的容器 {container} 装了 {record_count} 条，不是 {product_records}（满了才关）"
                    )
                });
            }
            Some(records_the_knob_allows) => {
                judgements.judge(invariant, record_count <= records_the_knob_allows, || {
                    format!(
                        "{version_name}：{tree_name}（树 {tree}）的容器 {container} 装了 {record_count} 条，多于旋钮值 {records_the_knob_allows}"
                    )
                });
                let is_the_rightmost_under_the_knob =
                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();
                if is_the_rightmost_under_the_knob {
                    continue;
                }
                let non_rightmost_container_holds_the_knob_value =
                    record_count == records_the_knob_allows;
                judgements.judge(
                    invariant,
                    non_rightmost_container_holds_the_knob_value,
                    || {
                        format!(
                            "{version_name}：{tree_name}（树 {tree}）里不是最右的容器 {container} 装了 {record_count} 条，不是旋钮值 {records_the_knob_allows}（满了才关）"
                        )
                    },
                );
            }
        }
    }
}

/// I-9.21 / I-9.22（判决第二节 T5 取 K1、T6 每版）：这一版墓碑树按叶序的容器，读容器头、不沿根的条目读。有对象时（没装旋钮至少两片，
/// 装了至少一片）判容器号与记录数的三句（第 56 批 G2，同 [`judge_lineage_or_retirement_container_capacity_of_a_version`]）：沿叶序第一片容器号是 1、
/// 相邻两片加一、每片（含最右那片）至少一条；再按旋钮：没装判 I-9.21——最右那片之外每片记录数 == 815，零根或只一片容器没有对象；
/// 装了旋钮判 I-9.22——每片记录数 ≤ 旋钮值、最右那片之外每片记录数 == 旋钮值（满了才关在旋钮下的那一半，第 57 批 H1，
/// 同 [`judge_lineage_or_retirement_container_capacity_of_a_version`]），零根没有对象。
fn judge_tombstone_container_capacity_of_a_version(
    version_name: &str,
    tombstone_tree: u64,
    containers: &[TombstoneContainerOfTheScan],
    capacity_knob: Option<TombstoneContainerCapacityKnob>,
    judgements: &mut Judgements,
) {
    let invariant = capacity_invariant_of(capacity_knob);
    let judged_when_the_tree_has_at_least = match capacity_knob {
        None => 2,
        Some(_) => 1,
    };
    if containers.len() < judged_when_the_tree_has_at_least {
        return;
    }
    if let Some(first_in_leaf_order) = containers.first() {
        judgements.judge(
            invariant,
            first_in_leaf_order.container == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,
            || {
                format!(
                    "{version_name}：墓碑树（树 {tombstone_tree}）沿叶序第一片容器的容器号是 {}，不是 {FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE}（容器号从 1 起）",
                    first_in_leaf_order.container
                )
            },
        );
    }
    for adjacent in containers.windows(2) {
        let (left, right) = (&adjacent[0], &adjacent[1]);
        let right_tombstone_container_follows_the_left_by_one =
            left.container.checked_add(1) == Some(right.container);
        judgements.judge(
            invariant,
            right_tombstone_container_follows_the_left_by_one,
            || {
                format!(
                    "{version_name}：墓碑树（树 {tombstone_tree}）沿叶序容器号不是逐片加一：容器 {} 之后是容器 {}（下一个号 = 最右号 + 1）",
                    left.container, right.container
                )
            },
        );
    }
    for (position_in_leaf_order, container) in containers.iter().enumerate() {
        let record_count =
            u64::try_from(container.records.len()).expect("一片容器的记录数装得进 u64");
        let tombstone_container_holds_a_record = record_count > 0;
        judgements.judge(invariant, tombstone_container_holds_a_record, || {
            format!(
                "{version_name}：墓碑树（树 {tombstone_tree}）的容器 {} 一条记录都没有（有新记录才开新片）",
                container.container
            )
        });
        match capacity_knob {
            None => {
                let is_the_rightmost = position_in_leaf_order + 1 == containers.len();
                if is_the_rightmost {
                    continue;
                }
                judgements.judge(invariant, record_count == TOMBSTONE_CONTAINER_RECORDS, || {
                    format!(
                        "{version_name}：墓碑树（树 {tombstone_tree}）里不是最右的容器 {} 装了 {record_count} 条，不是 {TOMBSTONE_CONTAINER_RECORDS}（满了才关）",
                        container.container
                    )
                });
            }
            Some(knob) => {
                let knob_records = knob.records_per_container();
                judgements.judge(invariant, record_count <= knob_records, || {
                    format!(
                        "{version_name}：墓碑树（树 {tombstone_tree}）的容器 {} 装了 {record_count} 条，多于旋钮值 {knob_records}",
                        container.container
                    )
                });
                let is_the_rightmost_tombstone_container_under_the_knob =
                    position_in_leaf_order + 1 == containers.len();
                if is_the_rightmost_tombstone_container_under_the_knob {
                    continue;
                }
                let non_rightmost_tombstone_container_holds_the_knob_value =
                    record_count == knob_records;
                judgements.judge(
                    invariant,
                    non_rightmost_tombstone_container_holds_the_knob_value,
                    || {
                        format!(
                            "{version_name}：墓碑树（树 {tombstone_tree}）里不是最右的容器 {} 装了 {record_count} 条，不是旋钮值 {knob_records}（满了才关）",
                            container.container
                        )
                    },
                );
            }
        }
    }
}

/// I-9.29（同号只许一个出生代）：各版的 inode 记录与各版自己谱系上的墓碑合起来，同一棵 inode 树里每个号只对应一个对象出生代
/// （判决第二节 T10 乙）。每个号判一次。
fn judge_one_object_birth_per_inode_number(
    object_births: &ObjectBirthsOfEachInodeNumber,
    judgements: &mut Judgements,
) {
    for ((inode_tree, inode), births) in object_births {
        judgements.judge(ONE_OBJECT_BIRTH_PER_INODE_NUMBER, births.len() == 1, || {
            let where_each_birth_was_seen: Vec<String> = births
                .iter()
                .map(|(object_birth, first_seen)| format!("对象出生代 {object_birth}（{first_seen}）"))
                .collect();
            format!(
                "inode 树（树 {inode_tree}）里 inode {inode} 在回退候选集里对应不止一个对象出生代：{}",
                where_each_birth_was_seen.join("、")
            )
        });
    }
}

/// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。当三棵容器树都没装只供测试的容量旋钮（产品镜像）：
/// I-9.21（产品镜像里非最右墓碑容器装满） 与 I-9.36（产品镜像里非最右谱系与退役容器装满） 照产品容量判，
/// I-9.22、I-9.37（测试镜像里容器不超旋钮值） 报不适用。
#[must_use]
pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
    check_pool_image_given_the_container_capacity_knobs(reader, ContainerCapacityKnobs::PRODUCT)
}

/// 池级 checker 带旋钮值的入口（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T5 取 K1）：旋钮不落盘
/// （D18（块里携带什么信息） 已定项 24 第 3 条），写这张镜像时压没压墓碑容器容量由调用方从镜像外给。`None` 与 [`check_pool_image`] 相同；
/// `Some` 时 I-9.22 按旋钮值判、I-9.21 报不适用。
#[must_use]
pub fn check_pool_image_given_the_tombstone_container_capacity_knob(
    reader: &dyn ImageReader,
    tombstone_container_capacity_knob: Option<TombstoneContainerCapacityKnob>,
) -> Vec<(&'static str, InvariantVerdict)> {
    check_pool_image_given_the_container_capacity_knobs(
        reader,
        ContainerCapacityKnobs {
            tombstone: tombstone_container_capacity_knob,
            lineage: None,
            retirement: None,
        },
    )
}

/// 池级 checker 带三棵容器树旋钮值的入口（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2，同墓碑那一个入口的
/// 判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T5 取 K1）：旋钮不落盘，写这张镜像时三棵容器树各压没压容量由调用方
/// 从镜像外给。[`ContainerCapacityKnobs::PRODUCT`] 与 [`check_pool_image`] 相同；墓碑旋钮 `Some` 时 I-9.22 按旋钮值判、I-9.21 报不适用；
/// 谱系或退役旋钮 `Some` 时那一棵按 I-9.37 判，`None` 的那一棵照 I-9.36 判。
#[must_use]
pub fn check_pool_image_given_the_container_capacity_knobs(
    reader: &dyn ImageReader,
    container_capacity_knobs: ContainerCapacityKnobs,
) -> Vec<(&'static str, InvariantVerdict)> {
    let system_configurations = chosen_system_configurations(reader);
    let mut root_ring_judgements = Judgements::default();
    let chosen: Vec<_> = system_configurations
        .iter()
        .filter_map(|(device, chosen)| {
            chosen
                .clone()
                .map(|(view, geometry)| (*device, view, geometry))
        })
        .collect();
    // 一块盘的两个系统配置槽都无效时**不早退**：恢复会改用别的盘上那一份、照常挂上
    // （`crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration`，D22（单元原子性怎么合成） 已定项 8
    // 「系统配置每盘放一份」买的就是这份冗余）。早退会让这种镜像上每一条不变量都报「不适用」，
    // 而它是一个挂得上的合法镜像——故障注入让一块盘的两个槽先后写失败就造得出它，判定会被静默放过
    // （.claude/kb/checks-owed.md 的 C461）。⚠️ 「哪块盘的系统配置全废了」今天没有编号报得出来，仍欠在 C461。
    // 任一盘任一槽自证过而带这一版读者不收的池级值（格式版本、加密类型、槽距、physical_block_size、环长、journal 环起点、
    // 根环起点、单元区起始槽号）：实现整池拒绝挂载
    // （`RecoveryFailure::SystemConfigurationValueRefused`），checker 报违例、别的不变量同「挂不上的镜像」一样不作保
    // （用户 2026-09-27 定系统配置越界整池拒，实审 A3-checker-2）。只有一部分槽带越界值时不拿别的槽照判下去。
    let system_configuration_slot_readings_of_each_device =
        system_configuration_slot_readings(reader);
    let any_system_configuration_slot_carries_a_refused_value =
        judge_system_configuration_values_the_reader_accepts(
            &system_configuration_slot_readings_of_each_device,
            &mut root_ring_judgements,
        );
    if any_system_configuration_slot_carries_a_refused_value {
        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
            root_ring_judgements.not_applicable(
                invariant,
                "池里有一槽自证过的系统配置带这一版读者不收的池级值：实现整池拒绝挂载，这不是一个挂得上的镜像",
            );
        }
        return root_ring_judgements.into_report();
    }
    // I-8.1 的三个字段等式（在飞上限、最坏占用、F = 格式常量）任一盘任一可择槽有一格不等：实现择系统配置时同判、整池拒绝挂载
    // （`RecoveryFailure::SystemConfigurationValueRefused`），checker 与上面 I-7.13 那一支同口径——报 I-8.1 违例、别的不变量一律不作保
    // （`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 4 行）。三格先判进一份只装它们的判定里：整池不作保时
    // 别的不变量（连同已判过成立的 I-7.13）都报不适用；三格都成立时再判进这一份，照常往下判。
    let mut judgements_of_the_journal_ring_geometry_fields_alone = Judgements::default();
    judge_journal_ring_geometry_fields_agree(
        &system_configuration_slot_readings_of_each_device,
        &mut judgements_of_the_journal_ring_geometry_fields_alone,
    );
    if judgements_of_the_journal_ring_geometry_fields_alone
        .violation_count(JOURNAL_RING_GEOMETRY_FIELDS_AGREE)
        > 0
    {
        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
            judgements_of_the_journal_ring_geometry_fields_alone.not_applicable(
                invariant,
                "池里有一槽可择的系统配置 journal 环几何三个字段（在飞上限、最坏占用、F）有一格不等：实现整池拒绝挂载，这不是一个挂得上的镜像",
            );
        }
        return judgements_of_the_journal_ring_geometry_fields_alone.into_report();
    }
    judge_journal_ring_geometry_fields_agree(
        &system_configuration_slot_readings_of_each_device,
        &mut root_ring_judgements,
    );
    // I-8.5（V-池）：全池皆废判红；一块盘交得出可择的系统配置就照常往下判（单盘全废走旁路报告，不进违例通道）。
    // 带越界值的槽、环几何三格不等的槽整池拒挂，上面那两支整池报不适用、不判这一条。
    // 一块盘都交不出可择的系统配置、而有槽自证过（incompat 位不认得、R / S 越界这类自证过而不可择的槽）：这不是一个挂得上的镜像，
    // I-8.5 同别的不变量一起报不适用，不判成成立（`InvariantVerdict` 的「不适用必须带理由，不许静默算成立」）；一个槽都没自证过的照判红。
    if chosen.is_empty() {
        let mut judgement_of_some_device_carrying_a_self_verified_slot = Judgements::default();
        judge_some_device_carries_a_self_verified_system_configuration(
            &system_configuration_slot_readings_of_each_device,
            &mut judgement_of_some_device_carrying_a_self_verified_slot,
        );
        match judgement_of_some_device_carrying_a_self_verified_slot
            .first_violation_of(SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION)
        {
            Some(detail) => {
                let detail = detail.to_owned();
                root_ring_judgements.judge(
                    SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
                    false,
                    || detail,
                );
            }
            None => root_ring_judgements.not_applicable(
                SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
                "池里有系统配置槽自证过，而没有一块盘交得出可择的系统配置：这不是一个挂得上的镜像",
            ),
        }
        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
            root_ring_judgements.not_applicable(
                invariant,
                "池里没有一块盘交得出有效的系统配置：这不是一个挂得上的镜像",
            );
        }
        return root_ring_judgements.into_report();
    }
    judge_some_device_carries_a_self_verified_system_configuration(
        &system_configuration_slot_readings_of_each_device,
        &mut root_ring_judgements,
    );
    let geometry = chosen[0].2;
    // 各盘择到的系统配置要属于同一个池：fsid 逐盘相同（一块别的池的旧盘插进来，实例代号可以恰好也是 1，I-7.7 看不出来）。
    for (device, view, _) in &chosen {
        root_ring_judgements.judge(
            "I-1.4",
            view.filesystem_identifier == geometry.filesystem_identifier,
            || {
                format!(
                    "盘 {device} 择到的系统配置 fsid 与盘 {} 的不同：这块盘不属于这个池",
                    chosen[0].0
                )
            },
        );
        // I-7.15：各盘所选的系统配置槽设备数相同（不要求等于盘表：降级镜像里 checker 手里的盘可以少于设备数）。
        // 实现那一侧 `choose_system_configuration` 在几块盘的设备数对不上时报 `SystemConfigurationsDisagree`、整池挂不上。
        root_ring_judgements.judge(
            DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
            view.device_count == chosen[0].1.device_count,
            || {
                format!(
                    "盘 {device} 择到的系统配置设备数 {} 与盘 {} 的 {} 不同",
                    view.device_count, chosen[0].0, chosen[0].1.device_count
                )
            },
        );
    }
    judge_system_configuration_slot_generations_follow_the_rotation(
        &system_configuration_slot_readings_of_each_device,
        &mut root_ring_judgements,
    );
    let chosen_geometries: Vec<PoolGeometry> = chosen
        .iter()
        .map(|(_, _, geometry_of_this_device)| *geometry_of_this_device)
        .collect();
    let decision_width =
        decision_width_declared_by_the_chosen_system_configurations(&chosen_geometries)
            .expect("上面判过 chosen 非空：至少一块盘择到了系统配置");
    judge_root_slots_aligned_to_the_decision_width(
        &geometry,
        decision_width,
        &mut root_ring_judgements,
    );
    let devices = reader.devices();
    let regions = usize::try_from(geometry.regions).expect("R");
    let region_devices = &geometry.region_devices[..regions.min(3)];
    let distinct: BTreeSet<u32> = region_devices.iter().copied().collect();
    let heaviest = devices
        .iter()
        .map(|device| {
            region_devices
                .iter()
                .filter(|candidate| *candidate == device)
                .count()
        })
        .max()
        .unwrap_or(0);
    let pigeonhole = regions.div_ceil(devices.len().max(1));
    root_ring_judgements.judge(
        "I-7.6",
        distinct.len() == regions.min(devices.len()) && heaviest <= pigeonhole && region_devices.iter().all(|device| devices.contains(device)),
        || format!("根环区域的设备 {region_devices:?}：不同值 {}、最重的盘背 {heaviest} 个（上界 {pigeonhole}）", distinct.len()),
    );
    let filesystem_identifier_low = u64::from_le_bytes(
        geometry.filesystem_identifier[..8]
            .try_into()
            .expect("8 字节"),
    );
    let roots = valid_roots(reader, &geometry);
    judge_root_record_algorithm_nonce_and_mac_are_zero(&mut root_ring_judgements, &roots);
    judge_root_records_live_in_the_slot_their_txg_names(
        &roots,
        &geometry,
        &mut root_ring_judgements,
    );
    judge_instance_table_rows_against_the_roots_referencing_them(
        reader,
        &roots,
        &mut root_ring_judgements,
    );
    // 每块盘两槽里自证过、fsid 与本池相同的系统配置槽（与实现取号同一读法，D18（块里携带什么信息） 已定项 11）：
    // I-7.12 逐盘比，回退候选集的 F 生效值取它们带的 F 的最大值（C556（checker 与层 0 不读系统配置里的 F））。
    let verified_system_configurations_of_this_pool: Vec<(
        u32,
        Vec<crate::SystemConfigurationView>,
    )> = verified_system_configuration_slots(reader)
        .into_iter()
        .map(|(device, slots)| {
            (
                device,
                slots
                    .into_iter()
                    .map(|(view, _)| view)
                    .filter(|view| view.filesystem_identifier == geometry.filesystem_identifier)
                    .collect(),
            )
        })
        .collect();
    judge_own_device_number_is_the_device_identity(
        &verified_system_configurations_of_this_pool,
        &mut root_ring_judgements,
    );
    judge_system_configuration_floor_against_the_roots_on_each_device(
        &verified_system_configurations_of_this_pool,
        &roots,
        &geometry,
        &mut root_ring_judgements,
    );
    judge_instance_carriers(reader, &geometry, &roots, &mut root_ring_judgements);
    // I-8.6、I-8.7、I-8.8 与 I-8.9 只读 journal 环，不读根：根环全灭的镜像上它们照样判得了（那一格归 I-7.1）。
    let journal_records_by_device =
        scanned_journal_records_by_device(reader, &geometry, filesystem_identifier_low);
    judge_journal_back_chain(&journal_records_by_device, &mut root_ring_judgements);
    judge_location_order_of_journal_named_entries(
        &journal_records_by_device,
        &mut root_ring_judgements,
    );
    judge_transaction_numbers_per_instance(&journal_records_by_device, &mut root_ring_judgements);
    judge_commit_markers_per_transaction(&journal_records_by_device, &mut root_ring_judgements);
    judge_publish_ordinals_and_last_record_flags(
        &journal_records_by_device,
        &mut root_ring_judgements,
    );
    judge_journal_record_txg_per_instance(&journal_records_by_device, &mut root_ring_judgements);
    judge_records_of_one_publish_within_the_in_flight_record_limit(
        &journal_records_by_device,
        geometry.declared_in_flight_record_limit,
        &mut root_ring_judgements,
    );
    judge_named_entries_against_the_unit_headers(
        reader,
        &journal_records_by_device,
        &mut root_ring_judgements,
    );
    judge_new_root_segment_of_the_last_record_against_the_roots(
        &journal_records_by_device,
        &roots,
        &mut root_ring_judgements,
    );
    root_ring_judgements.judge("I-7.1", !roots.is_empty(), || {
        "根环里一条自证过的根都没有".to_string()
    });
    if roots.is_empty() {
        root_ring_judgements.not_applicable(
            "I-7.3",
            "根环里一条自证过的根都没有：S 空，没有「代号最大者」可谈",
        );
        return root_ring_judgements.into_report();
    }
    judge_root_ring_health(&roots, &mut root_ring_judgements);
    // 择根：按 (txg, 实例) 取最大（D22（单元原子性怎么合成） 已定项 7，与实现 `recovery::choose_root` 同一个读法）；
    // 最大的不止一条时取读根环次序里最前的那条（`roots` 按 `root_slot_positions` 的区域、槽次序排，与 `recovery::every_root_ring_slot` 同序）：
    // 只在严格更大时才换。`max_by_key` 在键相同时交回最后一个，与 `choose_root` 相反（代码轮第二轮判决辩方 D3）。
    let newest_index = roots
        .iter()
        .enumerate()
        .fold(
            None,
            |newest_so_far: Option<(usize, (u64, u32))>, (index, (_, _, root))| {
                let choice_key = (root.checkpoint_txg, root.instance);
                match newest_so_far {
                    Some((_, newest_key)) if choice_key <= newest_key => newest_so_far,
                    Some(_) | None => Some((index, choice_key)),
                }
            },
        )
        .map(|(index, _)| index)
        .expect("上面判过根环里至少一条自证过的根");
    let mut walk = Walk::starting_with(
        reader,
        root_ring_judgements,
        filesystem_identifier_low,
        roots[newest_index].2.instance,
        roots[newest_index].2.checkpoint_txg,
    );
    // 先走最新的根：它的走读断没断就是 I-7.2；记账行只取最新根下面的。
    walk.walk_root(&roots[newest_index].2.record_bytes, true);
    walk.note_the_version_just_walked_for_the_appearance_rule(
        roots[newest_index].2.instance,
        roots[newest_index].2.checkpoint_txg,
        0,
    );
    let newest_failures = walk.walk_failures.clone();
    // I-3.11（已分配减 defer 等于最新根走读）要的「从最新有效根走读到的、这块盘上被引用的槽数」：走法与 I-3.1 同一个
    // （`note_reference` 记下的 (设备, 起点槽, 跨度)），只取最新根——这一刻 `references` 里还只有最新根这一遍记下的。
    let slots_referenced_by_the_newest_root = slots_referenced_per_device(&walk.references);
    let start_slots_referenced_by_the_newest_root: BTreeSet<(u32, u64)> = walk
        .references
        .keys()
        .map(|(device, start_slot, _)| (*device, *start_slot))
        .collect();
    // I-4.8（近 K 代根校验和自洽）与 I-7.4（近 K 代块未被复用）：候选集里任一根（最新根也在候选集里）出发遍历，所有块的校验和
    // 都与父指针一致、走读不断——最新根那一次各算一格；候选集只剩最新根时两条都还判得到（本地攻方腿：全称量词在单元素集合上照样成立）。
    let newest_txg = roots[newest_index].2.checkpoint_txg;
    let newest_walked_into_reused_or_erased_unit =
        !newest_failures.is_empty() || walk.judgements.violation_count("I-2.1") > 0;
    walk.judgements
        .judge("I-4.8", !newest_walked_into_reused_or_erased_unit, || {
            format!("最新根（txg {newest_txg}）出发的遍历有单元对不上或读不出")
        });
    walk.judgements
        .judge("I-7.4", !newest_walked_into_reused_or_erased_unit, || {
            format!("最新根（txg {newest_txg}）引用的单元已被复用或抹头（校验和对不上或头用不了）")
        });
    let accounting_seen = walk.accounting_seen;
    let accounting = walk.accounting.clone();
    let inode_number_watermark_rows_of_the_newest_root = walk.inode_number_watermark_rows.clone();
    // 别的根只取按最新根指着的实例表仍然有效的：(i, T) 有效 ⟺ 无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti
    // （D23（journal 的角色与格式） 已定项 14 回退段的候选集规则）；被抛弃时间线的根引用的单元由影子账隔离、不在当前账里。
    // 再加一条：txg ≥ 回退下界 F 的生效值（D16（发布语义） 已定项 1 的回退候选集，`effective_rollback_floor`）；F 之下的根引用的单元可以已被回收复用，
    // 它们不在当前账里、也不再是「近 K 代」——I-2.1 只在候选集里的根上判。
    let instance_table_rows = walk.instance_table_rows.clone();
    let abandoned_by_the_newest_roots_table = |root: &crate::RootView| {
        instance_table_rows.iter().any(|row| {
            row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
        })
    };
    // 上面那张表是主走读沿链判下来的行，链断在哪一片就只有那一片之前的行：读不全时滤不掉的根认不出来。
    // 滤没滤全就看这同一遍走读走没走到末片（`Walk::walk_instance_table_chain` 记下的），不另读一遍。
    let abandoned_roots_filtering = walk.abandoned_roots_filtering;
    // 回退候选集的下界取 F 生效值（D16（发布语义） 已定项 1「生效」，SysPre）：max(各幸存盘最新持久有效根所带 F 的最大值,
    // 池里自证过的系统配置槽带的 F)。有效 = 按最新根指着的实例表判不是被抛弃的；每块盘取落在它上面的根环区域里 (txg, 实例) 最大的那一条
    // （最新根的实例表读不出时一行都没有、不滤）；被抛弃时间线上的根带的 F 也算进来（D16（发布语义） 已定项 1，用户 2026-09-28 定「算进」）。
    // 只读根上的 F 时，带新 F 的根全坏、系统配置里的新 F 还在的合法镜像上，F 之下、单元已被合法复用的根会被当成候选，
    // I-7.4（近 K 代块未被复用） 等按候选集判的几条误红（C556（checker 与层 0 不读系统配置里的 F））。
    let mut newest_valid_root_on_each_device: BTreeMap<u32, &crate::RootView> = BTreeMap::new();
    for (region, _, root) in &roots {
        if abandoned_by_the_newest_roots_table(root) {
            continue;
        }
        let device = geometry.region_devices[usize::try_from(*region).expect("区域号")];
        let newest_on_this_device = newest_valid_root_on_each_device
            .entry(device)
            .or_insert(root);
        if (root.checkpoint_txg, root.instance)
            > (
                newest_on_this_device.checkpoint_txg,
                newest_on_this_device.instance,
            )
        {
            *newest_on_this_device = root;
        }
    }
    let highest_floor_on_the_abandoned_roots = roots
        .iter()
        .filter(|(_, _, root)| abandoned_by_the_newest_roots_table(root))
        .map(|(_, _, root)| root.rollback_floor)
        .max()
        .unwrap_or(0);
    let highest_floor_on_the_newest_valid_and_the_abandoned_roots =
        newest_valid_root_on_each_device
            .values()
            .map(|root| root.rollback_floor)
            .max()
            .unwrap_or(0)
            .max(highest_floor_on_the_abandoned_roots);
    let effective_rollback_floor = verified_system_configurations_of_this_pool
        .iter()
        .flat_map(|(_, slots)| slots.iter().map(|view| view.rollback_floor))
        .fold(
            highest_floor_on_the_newest_valid_and_the_abandoned_roots,
            u64::max,
        );
    // 一圈窗口：N 取最新根（它按自己的实例表恒有效），3S 是根环槽数。只给 I-7.9 挡洞里留下的旧根用（[`OneRevolutionWindow`]）；
    // 候选过滤与丁-defer 门槛不再看它（RGall）。
    let window = OneRevolutionWindow {
        anchor_txg: newest_txg,
        root_ring_slot_count: geometry.regions * geometry.slots_per_region,
    };
    // 掉出遍历的根槽按理由各记一次（几样都占的各边都记）：I-3.1 判红时这几个数就是「遍历为什么少算」的机理标识。
    let mut root_slots_dropped_as_abandoned = 0u64;
    let mut root_slots_dropped_below_floor = 0u64;
    let candidate_indexes: Vec<usize> = roots
        .iter()
        .enumerate()
        .filter(|(index, (_, _, root))| {
            // 被抛弃：按最新根指着的实例表判（D23（journal 的角色与格式） 已定项 14 候选集）。
            let abandoned = abandoned_by_the_newest_roots_table(root);
            let below_floor = root.checkpoint_txg < effective_rollback_floor;
            // 原来还有「一圈窗口之外」一格（环上有洞时洞那一槽里留着的上一圈的根）：候选过滤是原来同取一圈窗口的六处之一，
            // 用户 2026-10-04 定 RGall 一起退回整个环，那条根照样是候选、照样走。
            let walked = *index == newest_index || (!abandoned && !below_floor);
            if !walked {
                if abandoned {
                    root_slots_dropped_as_abandoned += 1;
                }
                if below_floor {
                    root_slots_dropped_below_floor += 1;
                }
            }
            walked
        })
        .map(|(index, _)| index)
        .collect();
    for index in candidate_indexes.iter().copied() {
        let (_, _, root) = &roots[index];
        if index != newest_index {
            let mismatches_before = walk.judgements.violation_count("I-2.1");
            let failures_before = walk.walk_failures.len();
            walk.walk_root(&root.record_bytes, false);
            walk.note_the_version_just_walked_for_the_appearance_rule(
                root.instance,
                root.checkpoint_txg,
                failures_before,
            );
            // 这条候选根引用的单元有一个校验和对不上或头用不了，就是它指着的块被复用或抹头了（I-7.4（近 K 代块未被复用）），
            // 从它出发的遍历也就不自洽（I-4.8（近 K 代根校验和自洽））；两条按每条候选根各判一格。
            let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
                > mismatches_before
                || walk.walk_failures.len() > failures_before;
            let root_txg = root.checkpoint_txg;
            walk.judgements
                .judge("I-7.4", !walked_into_reused_or_erased_unit, || {
                    format!(
                        "候选根 txg {root_txg} 引用的单元已被复用或抹头（校验和对不上或头用不了）"
                    )
                });
            walk.judgements
                .judge("I-4.8", !walked_into_reused_or_erased_unit, || {
                    format!("候选根 txg {root_txg} 出发的遍历有单元对不上或读不出")
                });
        }
    }
    // 回退候选集补上「由记录施加出来、根槽从没落盘的那一版」（2026-09-23 用户定候选 b）：它们没有根槽，
    // 上面按根环走的那一遍走不到，而它们换下的单元还在 defer 里、仍算在已分配里（增补 2 收口表第 54 行那 12 个状态差的 65 536 字节）。
    // 走法与判定照候选根：引用进同一个并集（I-3.1 / I-5.1），单元照判 I-2.1 等，I-7.4 / I-4.8 每一版各判一格。
    let versions_applied_only_by_records = versions_applied_only_by_records(
        &journal_records_by_device,
        &roots,
        &instance_table_rows,
        effective_rollback_floor,
    );
    for version in &versions_applied_only_by_records {
        let mismatches_before = walk.judgements.violation_count("I-2.1");
        let failures_before = walk.walk_failures.len();
        walk.walk_version_applied_only_by_records(version);
        walk.note_the_version_just_walked_for_the_appearance_rule(
            version.instance,
            version.checkpoint_txg,
            failures_before,
        );
        let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
            > mismatches_before
            || walk.walk_failures.len() > failures_before;
        let (instance, version_txg) = (version.instance, version.checkpoint_txg);
        walk.judgements
            .judge("I-7.4", !walked_into_reused_or_erased_unit, || {
                format!(
                    "由记录施加出来的那一版（实例 {instance}、txg {version_txg}）引用的单元已被复用或抹头（校验和对不上或头用不了）"
                )
            });
        walk.judgements
            .judge("I-4.8", !walked_into_reused_or_erased_unit, || {
                format!(
                    "由记录施加出来的那一版（实例 {instance}、txg {version_txg}）出发的遍历有单元对不上或读不出"
                )
            });
    }
    judge_the_appearance_rule_of_adjacent_candidate_versions(
        &walk.versions_folded_into_units_by_the_final_fold_table(),
        &walk.birth_txg_of_walked_units,
        &walk.birth_txg_of_walked_instance_table_pages,
        &walk.instance_table_rows,
        &mut walk.judgements,
    );
    // I-5.5：回退候选集里每条根与由记录施加出来的那几版，各自重读树表与 extent 树（只读），认出从 mkfs 水位第一次建树的那一版再判落点；
    // U 取所选系统配置偏移 417（`geometry` 是第一块择到系统配置的盘的那一份，I-7.13 判过它是环长现算的那个槽）。
    let versions_to_judge_the_first_data_unit_of: Vec<VersionToJudgeTheFirstDataUnitOf<'_>> =
        candidate_indexes
            .iter()
            .map(|index| {
                let root = &roots[*index].2;
                VersionToJudgeTheFirstDataUnitOf {
                    name: format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),
                    checkpoint_txg: root.checkpoint_txg,
                    tree_table_pointer: &root.record_bytes[36..122],
                }
            })
            .chain(versions_applied_only_by_records.iter().map(|version| {
                VersionToJudgeTheFirstDataUnitOf {
                    name: format!(
                        "由记录施加出来的那一版（实例 {}、txg {}）",
                        version.instance, version.checkpoint_txg
                    ),
                    checkpoint_txg: version.checkpoint_txg,
                    tree_table_pointer: version.tree_table_and_mapping_root_pointers().0,
                }
            }))
            .collect();
    judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
        reader,
        &versions_to_judge_the_first_data_unit_of,
        geometry.unit_area_start_slot,
        &mut walk.judgements,
    );
    if walk.units_judged_against_an_older_candidate_root == 0 {
        walk.judgements.not_applicable(
            UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
            "回退候选集里只有最新那条根、或不是最新的候选根一个被引用单元都没走到：没有「不是最新的候选根」可判（最新那条归 I-4.2）",
        );
    }
    judge_blocks_referenced_by_abandoned_roots(
        reader,
        &geometry,
        &roots,
        newest_index,
        &instance_table_rows,
        filesystem_identifier_low,
        &mut walk.judgements,
    );
    judge_trees_of_the_replayed_version_are_traversable(
        reader,
        &journal_records_by_device,
        &roots[newest_index].2,
        geometry.journal_ring_bytes,
        filesystem_identifier_low,
        &mut walk.judgements,
    );
    let referenced_units_judged_against_their_pointer =
        walk.referenced_units_judged_against_their_pointer;
    let mut location_entry_checksums = LocationEntryChecksumsForQuarantine {
        known: std::mem::take(&mut walk.location_entry_checksums),
        roots_not_walked_yet: Some(RootsNotWalked {
            reader,
            filesystem_identifier_low,
            mount_root_instance: roots[newest_index].2.instance,
            mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,
            root_records: roots
                .iter()
                .enumerate()
                .filter(|(index, _)| !candidate_indexes.contains(index))
                .map(|(_, (_, _, root))| root.record_bytes.clone())
                .collect(),
        }),
    };
    let mut judgements = walk.judgements;
    judgements.judge("I-7.2", newest_failures.is_empty(), || {
        format!("最新的根走不完：{}", newest_failures.join("；"))
    });
    // I-1.2 与 I-4.2 在遍历方向上判的是同一批单元：走读一个被引用单元都没走到时两条都没有对象
    // （根槽读得出而树表读不出、或每个单元的头都用不了），报「不适用」并带理由，不报成立。
    if referenced_units_judged_against_their_pointer == 0 {
        for invariant in ["I-1.2", "I-4.2"] {
            judgements.not_applicable(
                invariant,
                "走读一个被引用的单元都没读到（实例表单元按 I-1.2 那一行的例外不进这两条）：出生身份与已发布谓词都没有对象",
            );
        }
    }
    // I-1.8：扫描方向按已发布谓词过滤之后归并成组。谓词要挂载根的 (实例代号, txg) 与它指着的那一版实例表，
    // 所以这一遍排在走读之后（`instance_table_rows` 是走最新根时取的）。
    let published = PublishedPredicate {
        mount_root_instance: roots[newest_index].2.instance,
        mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,
        instance_table_rows: &instance_table_rows,
    };
    let content_units =
        scanned_content_units(reader, &geometry, filesystem_identifier_low, &published);
    if judge_merged_version_total_order(reader, &content_units, &mut judgements) == 0 {
        judgements.not_applicable(
            "I-1.8",
            "扫描方向没有两份归并到一起、或两组归并到同一个 key 的码 1 / 码 3 已发布单元：归并与定序都比不出来",
        );
    }
    // 这两遍各自按候选集里每条根读树表与树节点：共用一份按位置条目记的缓存，候选根常指着同一个单元，层 0 每个崩溃状态都跑。
    let mut index_node_cache = IndexNodeCache::new();
    judge_release_generation_and_tree_table_birth(
        reader,
        &roots,
        &candidate_indexes,
        newest_index,
        &mut index_node_cache,
        &mut judgements,
    );
    judge_tree_identifier_birth_txg_across_every_readable_root(
        reader,
        &geometry,
        &roots,
        &mut index_node_cache,
        &mut judgements,
    );
    // I-5.4 按版本那一层：回退候选集里每条根，加由记录施加出来的那几版与下一次挂载先施加的那一版（改法 RV，见
    // [`judge_allocation_records_disjoint`]）。坏盘上同一版两类都认得出时判两遍：同一棵分配记录树按根节点只读一次，违例不重报。
    let version_the_next_mount_applies_first = version_the_next_mount_applies_first(
        reader,
        &journal_records_by_device,
        &roots[newest_index].2,
        geometry.journal_ring_bytes,
    );
    let versions_to_judge_the_allocation_records_of: Vec<VersionToJudgeTheAllocationRecordsOf<'_>> =
        candidate_indexes
            .iter()
            .map(|index| {
                let root = &roots[*index].2;
                VersionToJudgeTheAllocationRecordsOf {
                    name: format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),
                    tree_table_pointer: &root.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
                    allocation_record_tree_root_held_outside_the_tree_table: &root.record_bytes
                        [ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT],
                }
            })
            .chain(versions_applied_only_by_records.iter().map(|version| {
                VersionToJudgeTheAllocationRecordsOf {
                    name: format!(
                        "由记录施加出来的那一版（实例 {}、txg {}）",
                        version.instance, version.checkpoint_txg
                    ),
                    tree_table_pointer: &version.new_root_segment
                        [NEW_ROOT_SEGMENT_TREE_TABLE_POINTER],
                    allocation_record_tree_root_held_outside_the_tree_table: &version
                        .new_root_segment[NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER],
                }
            }))
            .chain(version_the_next_mount_applies_first.iter().map(|version| {
                VersionToJudgeTheAllocationRecordsOf {
                    name: format!(
                        "下一次挂载先施加的那一版（实例 {}、txg {}）",
                        version.instance, version.checkpoint_txg
                    ),
                    tree_table_pointer: &version.new_root_segment
                        [NEW_ROOT_SEGMENT_TREE_TABLE_POINTER],
                    allocation_record_tree_root_held_outside_the_tree_table: &version
                        .new_root_segment[NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER],
                }
            }))
            .collect();
    judge_allocation_records_disjoint(
        reader,
        &versions_to_judge_the_allocation_records_of,
        &mut index_node_cache,
        &mut judgements,
    );
    let allocation_node_pointers = allocation_record_node_pointers_of_the_candidate_versions(
        reader,
        &roots,
        &candidate_indexes,
        &versions_applied_only_by_records,
        tree_table_pointer_of_the_version_the_next_mount_applies_first(
            version_the_next_mount_applies_first.as_ref(),
        ),
        &mut index_node_cache,
    );
    judge_allocation_generations_against_unit_births(
        reader,
        &allocation_node_pointers,
        &mut index_node_cache,
        &mut judgements,
    );
    judge_rollback_floor_raises_against_their_ceilings(
        reader,
        &geometry,
        &roots,
        window,
        &mut index_node_cache,
        &mut judgements,
    );
    // 删除态扫描（I-9.5、I-3.4、I-4.7、I-4.6；判哪几版、配对与读不全的报法照 I-9.5 那一行的共用判法，判决
    // `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 F4）：回退候选集里每条根与由记录施加出来的那几版，各自重读一遍（只读）。
    let versions_to_scan_the_deletion_state_of: Vec<VersionToScanTheDeletionStateOf<'_>> =
        candidate_indexes
            .iter()
            .map(|index| {
                let root = &roots[*index].2;
                VersionToScanTheDeletionStateOf {
                    name: format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),
                    checkpoint_txg: root.checkpoint_txg,
                    tree_table_pointer: &root.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
                }
            })
            .chain(versions_applied_only_by_records.iter().map(|version| {
                VersionToScanTheDeletionStateOf {
                    name: format!(
                        "由记录施加出来的那一版（实例 {}、txg {}）",
                        version.instance, version.checkpoint_txg
                    ),
                    checkpoint_txg: version.checkpoint_txg,
                    tree_table_pointer: version.tree_table_and_mapping_root_pointers().0,
                }
            }))
            .collect();
    judge_deletion_state_of_the_versions(
        reader,
        &versions_to_scan_the_deletion_state_of,
        &mut index_node_cache,
        &mut judgements,
    );
    // 墓碑与谱系那几条（I-9.18–I-9.22、I-9.29、I-9.10 墓碑那一个等号）：同一份版本清单，每一版从自己的 txg 与自己的谱系树起算。
    judge_tombstones_and_the_lineage_of_the_versions(
        reader,
        &versions_to_scan_the_deletion_state_of,
        abandoned_roots_filtering,
        container_capacity_knobs,
        &mut index_node_cache,
        &mut judgements,
    );
    // I-9.17、I-9.23 在主走读里每条内部条目判；一条条目都没走到时的理由。
    judgements.not_applicable(
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION,
    );
    // I-4.4（意图只描述目标态）：第一版的意图载体没有可装增量的字段，报不适用并写理由（用户 2026-10-05 定）。
    judgements.not_applicable(
        INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
        INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
    );
    // I-9.6（水位大于两处最大号）：记账里那条「inode 号水位」要大于遍历侧算出的 inode 树内最大 key。
    // 两条独立路径——水位是发布路径在记账树里写下的一个数，最大 key 是 checker 逐片叶容器逐条记录数出来的。
    // 另一半（> 全部已发布的墓碑记录的对象 ID）接在这一判后面：墓碑是打包记录类型 1，意图那次发布往墓碑树开放容器追加一条
    // （D8（核心索引结构） 已定项 16 第 1 条；`deleted_inodes` 的形态已定，C118 还清）。
    let inode_number_watermark = accounting
        .get(&(STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION))
        .copied();
    match (
        inode_number_watermark,
        walk.largest_inode_number_in_the_inode_tree,
    ) {
        (Some(watermark), Some(largest_inode)) => {
            judgements.judge("I-9.6", watermark > largest_inode, || {
                format!(
                    "记账里的 inode 号水位 {watermark} 不大于 inode 树内最大 key {largest_inode}"
                )
            });
        }
        (None, _) => judgements.not_applicable(
            "I-9.6",
            "最新的根下面没有记账树、或记账里没有 inode 号水位那一行",
        ),
        (Some(_), None) => {
            judgements.not_applicable("I-9.6", "最新的根下面走不到 inode 树里的任何一条记录")
        }
    }
    // I-9.6 墓碑那一半：水位 > 所选根的墓碑树里全部已发布的（按实例表判）墓碑记录的对象 ID；一条墓碑都没有时没有对象。
    if let Some(watermark) = inode_number_watermark {
        match largest_published_tombstone_object_identifier(
            reader,
            &roots[newest_index].2.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
            &published,
            &mut index_node_cache,
        ) {
            Ok(Some(largest_tombstone_object)) => {
                judgements.judge("I-9.6", watermark > largest_tombstone_object, || {
                    format!(
                        "记账里的 inode 号水位 {watermark} 不大于所选根（txg {newest_txg}）的墓碑树里已发布的墓碑记录的最大对象 ID {largest_tombstone_object}"
                    )
                });
            }
            Ok(None) => {}
            Err(gap) => judgements.part_of_the_range_not_judged("I-9.6", gap.reason()),
        }
    }
    // I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上）：挨着 I-9.6，同样只在最新的根上判。
    judge_inode_number_watermark_steps_of_the_newest_root(
        reader,
        &roots[newest_index].2.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
        newest_txg,
        &inode_number_watermark_rows_of_the_newest_root,
        &mut index_node_cache,
        &mut judgements,
    );
    if !walk.inode_tree_walked {
        judgements.not_applicable(
            "I-9.12",
            "最新的根下面走不到 inode 树的内部节点（树表 0 条或根读不出）",
        );
    }
    for (inode, unit_object_birth, what) in &walk.data_unit_objects {
        if let Some(record_birth) = walk.inode_object_birth.get(inode) {
            judgements.judge("I-9.10", record_birth == unit_object_birth, || {
                format!(
                    "{what} 的对象出生代 {unit_object_birth} 与 inode 记录的 {record_birth} 不符"
                )
            });
        }
    }
    // I-5.1：同一块盘上，不同的引用不许占重叠的槽（两条位置条目落在不同盘上是显式的多副本）。
    let mut per_device: BTreeMap<u32, Vec<(u64, u64, &String)>> = BTreeMap::new();
    for ((device, slot, span), what) in &walk.references {
        per_device
            .entry(*device)
            .or_default()
            .push((*slot, *span, what));
    }
    for (device, mut ranges) in per_device.clone() {
        ranges.sort_unstable_by_key(|(slot, span, _)| (*slot, *span));
        for pair in ranges.windows(2) {
            let (first_slot, first_span, first_what) = pair[0];
            let (second_slot, _, second_what) = pair[1];
            judgements.judge("I-5.1", first_slot + first_span <= second_slot, || {
                format!("盘 {device}：{first_what}（槽 {first_slot} 跨 {first_span}）与 {second_what}（槽 {second_slot}）重叠")
            });
        }
    }
    // I-3.1 判红时，说明文字里带一段机理标识：遍历覆盖了哪些根、剩下的按什么理由没覆盖。
    // 判读的一方（`singlefs-harness` 的 `history::allocation_statistic_mechanism`）按它分辨「记账多算」是哪一种机理造成的：
    // 环转过一圈把 F 之上的根挤出了环，还是回退下界把环里读得到的根挡在了外面——两者的签名（只有 I-3.1 红、记账多于遍历）
    // 一模一样，不带这一段就只能按签名认，机理不同的新问题会被「已知红」清单接走（代码三方 m2-supp3-item3-code-r1 判决 K6 的假阴那一半）。
    let root_ring_slot_count = geometry.regions * geometry.slots_per_region;
    let oldest_readable_root_txg = roots
        .iter()
        .map(|(_, _, root)| root.checkpoint_txg)
        .min()
        .unwrap_or(0);
    let readable_root_slot_count = u64::try_from(roots.len()).expect("根槽数");
    let walked_root_slot_count = u64::try_from(candidate_indexes.len()).expect("候选根槽数");
    let walked_versions_applied_only_by_records =
        u64::try_from(versions_applied_only_by_records.len()).expect("版本数");
    // 「并进遍历的由记录施加出来的版本」那一段是 2026-09-23 候选 b 加的；判读的一方按字段名取数（`leading_number_after`），
    // 不看各段的次序，所以插在「遍历的候选根槽」之后、与它挨着读。
    let mechanism = move || {
        format!(
            "；机理：根环槽数 {root_ring_slot_count}、最新根 txg {newest_txg}、环里自证过的根槽 {readable_root_slot_count} 个、最老的自证过的根 txg {oldest_readable_root_txg}、遍历的候选根槽 {walked_root_slot_count} 个、并进遍历的由记录施加出来的版本 {walked_versions_applied_only_by_records} 个、被实例表判抛弃的根槽 {root_slots_dropped_as_abandoned} 个、回退下界 F {effective_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"
        )
    };
    // I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。
    // I-3.11：同一块盘上「已分配」减「defer 待释放」对只走最新根那一遍得到的和。
    let slots_referenced_by_every_walked_version = slots_referenced_per_device(&walk.references);
    let start_slots_referenced_by_every_walked_version: BTreeSet<(u32, u64)> = walk
        .references
        .keys()
        .map(|(device, start_slot, _)| (*device, *start_slot))
        .collect();
    // 隔离的记录（已分配而没有根引用、checker 自己读那一份也读不出或与位置项里的校验和对不上）在这两条上豁免：I-3.1 对全部走过的版本的引用、
    // I-3.11 对最新根这一遍的引用各算一份（`quarantined_slots_exempted_per_device`）。
    let newest_root_view = &roots[newest_index].2;
    let quarantined_exempted_against_every_walked_version = quarantined_slots_exempted_per_device(
        reader,
        newest_root_view,
        &start_slots_referenced_by_every_walked_version,
        &mut location_entry_checksums,
        &mut index_node_cache,
    );
    let walked_slots_of_every_walked_version: BTreeSet<(u32, u64)> = walk
        .references
        .keys()
        .flat_map(|(device, start_slot, span_slots)| {
            (*start_slot..*start_slot + *span_slots).map(move |slot| (*device, slot))
        })
        .collect();
    let standing_in_the_newest_roots_table = |root: &crate::RootView| {
        if abandoned_by_the_newest_roots_table(root) {
            RootStandingInTheNewestRootsTable::Abandoned
        } else {
            RootStandingInTheNewestRootsTable::Valid
        }
    };
    let roots_below_the_floor: Vec<&crate::RootView> = roots
        .iter()
        .map(|(_, _, root)| root)
        .filter(|root| {
            root_references_are_merged_into_the_deferred_slots_below_the_floor(
                standing_in_the_newest_roots_table(root),
                root.checkpoint_txg,
                effective_rollback_floor,
            )
        })
        .collect();
    let deferred_threshold = deferred_slots_reclaim_threshold(
        roots.iter().map(|(_, _, root)| {
            (
                standing_in_the_newest_roots_table(root),
                root.checkpoint_txg,
            )
        }),
        effective_rollback_floor,
    );
    let deferred_below_the_floor_against_every_walked_version =
        deferred_slots_referenced_only_below_the_floor_per_device(
            reader,
            filesystem_identifier_low,
            newest_root_view,
            &roots_below_the_floor,
            &walked_slots_of_every_walked_version,
            deferred_threshold,
            &mut index_node_cache,
        );
    let quarantined_exempted_against_the_newest_root = quarantined_slots_exempted_per_device(
        reader,
        newest_root_view,
        &start_slots_referenced_by_the_newest_root,
        &mut location_entry_checksums,
        &mut index_node_cache,
    );
    // 最新根的记账是按它自己带的 F 记的（抬 F 的回收在写第一条带新 F 的根之前做，`mount::raise_rollback_floor`）：它带的 F 低于
    // F 生效值时——先把新 F 写进系统配置之后、带新 F 的根落盘之前（D16（发布语义） 已定项 1「抬 F 那一串」），或带新 F 的根全读不出——
    // 它的「已分配」还算着 F 生效值之下那些根引用的槽，与按 F 生效值取的候选集并集不可比，I-3.1 这一格报不适用（实二的读法，交主 agent）。
    let accounting_was_written_under_the_effective_floor =
        roots[newest_index].2.rollback_floor == effective_rollback_floor;
    if accounting_seen {
        for device in &devices {
            let quarantined_exempted = quarantined_exempted_against_every_walked_version
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES;
            let deferred_below_the_floor = deferred_below_the_floor_against_every_walked_version
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES;
            let walked = slots_referenced_by_every_walked_version
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES
                + quarantined_exempted
                + deferred_below_the_floor;
            let allocated = accounting
                .get(&(STATISTIC_ALLOCATED_BYTES, *device))
                .copied();
            if accounting_was_written_under_the_effective_floor {
                judgements.judge("I-3.1", allocated == Some(walked), || {
                    format!(
                        "盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}（其中隔离豁免 {quarantined_exempted}、F 之下仍在 defer 的 {deferred_below_the_floor}）{}",
                        mechanism()
                    )
                });
            }
            // 单元区起点是系统配置里读来的 8 字节、空闲与已分配是记账行里读来的 8 字节：减法在起点越过盘末时下溢、加法在两个大值上溢出，
            // checker 当场 panic（代码审阅 6c 第 6 条那一族）。起点越过盘末时单元区没有容量可比（`None`），这一格与盘容量读不出同样判红；
            // 两个值加起来溢出也判红、不回绕（与 I-3.11 那一判同一个写法）。
            let capacity = reader.device_bytes(*device).and_then(|bytes| {
                (bytes / SLOT_BYTES)
                    .checked_sub(geometry.unit_area_start_slot)
                    .map(|unit_area_slots| unit_area_slots * SLOT_BYTES)
            });
            let free = accounting.get(&(STATISTIC_FREE_BYTES, *device)).copied();
            judgements.judge("I-5.2", matches!((free, allocated, capacity), (Some(free), Some(allocated), Some(capacity)) if free.checked_add(allocated) == Some(capacity)), || {
                format!("盘 {device}：空闲 {free:?} + 已分配 {allocated:?} ≠ 单元区 {capacity:?}")
            });
            // I-3.11（已分配减 defer 等于最新根走读）：写者把活单元记成已释放、放进 defer 时，已分配与空闲两个和照样对得上
            // （I-3.1 / I-5.2 都绿），只有这一条的差对不上。判据写成加法（defer + 最新根走读 == 已分配），盘上读来的值大到溢出也判红、不回绕。
            let deferred = accounting
                .get(&(STATISTIC_DEFER_QUEUE_BYTES, *device))
                .copied();
            let quarantined_exempted_newest = quarantined_exempted_against_the_newest_root
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES;
            let referenced_by_the_newest_root = slots_referenced_by_the_newest_root
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES
                + quarantined_exempted_newest;
            judgements.judge(
                "I-3.11",
                matches!((allocated, deferred), (Some(allocated), Some(deferred)) if deferred.checked_add(referenced_by_the_newest_root) == Some(allocated)),
                || {
                    format!("盘 {device}：记账的已分配 {allocated:?} 减 defer 待释放 {deferred:?}，不等于从最新根（txg {newest_txg}）走读到的 {referenced_by_the_newest_root}（其中隔离豁免 {quarantined_exempted_newest}）")
                },
            );
        }
        if !accounting_was_written_under_the_effective_floor {
            judgements.not_applicable(
                "I-3.1",
                "最新根带的 F 低于 F 生效值（系统配置里的新 F 已落、带它的根还没落或全读不出）：最新根的记账按它自己的 F 记，与按 F 生效值取的候选集并集不可比",
            );
        }
    } else {
        judgements.not_applicable("I-3.1", "最新的根下面还没有记账树（第 0 代树表）");
        judgements.not_applicable("I-5.2", "最新的根下面还没有记账树（第 0 代树表）");
        judgements.not_applicable("I-3.11", "最新的根下面还没有记账树（第 0 代树表）");
    }
    // I-7.8：根环全部有效根的水位取 max，要大于盘上出现过的最大树 ID（码 2 单元头 ∪ 走过的树表条目；
    // 单元头那一半不数没发布过的孤儿，读法见 `scanned_tree_identifiers`）。
    let watermark = roots
        .iter()
        .map(|(_, _, root)| root.tree_identifier_watermark)
        .max()
        .unwrap_or(0);
    let newest_published_txg = roots
        .iter()
        .map(|(_, _, root)| root.checkpoint_txg)
        .max()
        .unwrap_or(0);
    let mut seen = scanned_tree_identifiers(
        reader,
        &geometry,
        newest_published_txg,
        &instance_table_rows,
        filesystem_identifier_low,
    );
    seen.extend(walk.tree_identifiers_in_tables.iter().copied());
    let highest_seen = seen.iter().max().copied().unwrap_or(0);
    judgements.judge("I-7.8", watermark > highest_seen, || {
        format!("根环水位最大 {watermark}，盘上出现过的最大树 ID {highest_seen}")
    });
    // 再判最新那条根自己的水位（`m3-inv-a-r1` 判决第二节第 9 行，零轮）：只把最新那条根的水位压低时，上面取 max 那一格被更早的根盖住，
    // 而恢复按所选根的水位往下发号（core 的 `recover` 在这种镜像上报 I-7.8）。先判上面那一格，它的违例说明不变。
    let newest_root_watermark = roots[newest_index].2.tree_identifier_watermark;
    judgements.judge("I-7.8", newest_root_watermark > highest_seen, || {
        format!(
            "最新那条根（txg {newest_txg}）的水位 {newest_root_watermark}，盘上出现过的最大树 ID {highest_seen}"
        )
    });
    judgements.into_report()
}

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_format::{
        TREE_IDENTIFIER_ACCOUNTING, TREE_IDENTIFIER_ALLOCATION_RECORDS,
        TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST, TREE_IDENTIFIER_DELETED_INODES,
        TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LINEAGE,
        TREE_IDENTIFIER_LIVELIST, TREE_IDENTIFIER_LOGGED_OPERATIONS,
        TREE_IDENTIFIER_SPARSE_SIDE_TABLE, TREE_IDENTIFIER_TOMBSTONE,
        TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, TREE_IDENTIFIER_WATERMARK_AT_MKFS,
    };

    /// checker 自己那份种类码与格式常量里的树 ID 值域不相交（D8（核心索引结构） 已定项 9 射程、已定项 11，
    /// 用户 2026-10-01 定案「树 ID 整体挪开」）：每个种类码都小于 65536（树的种类那 2 字节取不到的最小值），
    /// 每个树 ID 与两个水位都不小于它。
    #[test]
    fn every_tree_kind_code_the_checker_reads_is_below_every_tree_identifier() {
        let first_value_the_kind_field_cannot_hold = u64::from(u16::MAX) + 1;
        for kind in TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER {
            assert!(
                u64::from(kind) < first_value_the_kind_field_cannot_hold,
                "种类码 {kind} 小于 65536"
            );
        }
        for identifier in [
            TREE_IDENTIFIER_EXTENT,
            TREE_IDENTIFIER_INODE,
            TREE_IDENTIFIER_ALLOCATION_RECORDS,
            TREE_IDENTIFIER_ACCOUNTING,
            TREE_IDENTIFIER_CENTRAL_MAPPING,
            TREE_IDENTIFIER_LIVELIST,
            TREE_IDENTIFIER_SPARSE_SIDE_TABLE,
            TREE_IDENTIFIER_DEADLIST,
            TREE_IDENTIFIER_DELETED_INODES,
            TREE_IDENTIFIER_TOMBSTONE,
            TREE_IDENTIFIER_LINEAGE,
            TREE_IDENTIFIER_LOGGED_OPERATIONS,
            TREE_IDENTIFIER_WATERMARK_AT_MKFS,
            TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
        ] {
            assert!(
                identifier >= first_value_the_kind_field_cannot_hold,
                "树 ID（或水位）{identifier} 不小于 65536：与 checker 读的种类码值域不相交"
            );
        }
    }

    /// 一条只带键、反向链与链值的扫描记录（别的字段这几条用例不读）。
    fn scanned_record_of(
        instance: u32,
        back_chain: u32,
        chain_value_the_next_record_must_carry: u32,
    ) -> ScannedJournalRecord {
        ScannedJournalRecord {
            instance,
            checkpoint_txg: 0,
            new_root_segment: Vec::new(),
            transaction: 0,
            commit_marker: CommitMarker::Present,
            ordinal_within_publish: 1,
            record_flags_byte: RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH,
            slot: 0,
            back_chain,
            chain_value_the_next_record_must_carry,
            named_entry_locations: Vec::new(),
            named_entries: Vec::new(),
        }
    }

    /// 两块盘同一计数器坐着不同实例的记录（盘 0 计数器 4 是实例 2 的，盘 1 计数器 4 是实例 1 的）：恢复眼里的环按 (实例代号, 计数器)
    /// 建键，两条都留下，不拿先读到的盘 0 那条挤掉实例 1 的（I-4.1 那一行：`records_the_replay_reads` 先改、这一处同改）。
    #[test]
    fn the_ring_as_the_recovery_reads_it_keeps_both_records_of_two_instances_at_the_same_counter() {
        let records_by_device = vec![
            (0, BTreeMap::from([(4, scanned_record_of(2, 0, 0x22))])),
            (1, BTreeMap::from([(4, scanned_record_of(1, 0x0b, 0x11))])),
        ];
        let ring = records_as_the_recovery_reads_them(&records_by_device);
        assert_eq!(
            ring.keys().copied().collect::<Vec<_>>(),
            vec![(1, 4), (2, 4)],
            "两条都在，按 (实例代号, 计数器) 认"
        );
        assert_eq!(
            ring[&(1, 4)].chain_value_the_next_record_must_carry,
            0x11,
            "实例 1 那一条是盘 1 上那一份"
        );
    }

    /// 反向链那一关按 (实例代号, 计数器 − 1) 找本实例内逻辑前一条：计数器 3 上先读到的是实例 2 的记录、实例 1 的那一条在盘 1 上，
    /// 实例 1 计数器 4 那一条的反向链与实例 1 计数器 3 那条的链值不等 ⇒ 进不了前缀；对得上 ⇒ 进得去。
    /// 按计数器找时先读到的是实例 2 的那条，当成「前一条是别的实例」放过去。
    #[test]
    fn the_back_chain_gate_looks_up_the_predecessor_of_the_same_instance_past_another_instance_at_that_counter(
    ) {
        let records_by_device = vec![
            (0, BTreeMap::from([(3, scanned_record_of(2, 0, 0x22))])),
            (1, BTreeMap::from([(3, scanned_record_of(1, 0, 0x11))])),
        ];
        let ring = records_as_the_recovery_reads_them(&records_by_device);
        assert!(
            !record_can_enter_a_replay_prefix(&ring, 4, &scanned_record_of(1, 0x12, 0)),
            "实例 1 计数器 4 的反向链 0x12 与实例 1 计数器 3 的链值 0x11 不等：不进前缀"
        );
        assert!(
            record_can_enter_a_replay_prefix(&ring, 4, &scanned_record_of(1, 0x11, 0)),
            "对得上：进前缀"
        );
    }

    /// 一圈窗口（I-7.9 挡洞里留下的旧根用的那一个）：N = 35、环长 24 ⇒ 窗口 = txg > 11；N 小于环长时整个环都在窗口里。
    #[test]
    fn the_one_revolution_window_keeps_the_txgs_above_the_newest_minus_the_ring_length() {
        let window = OneRevolutionWindow {
            anchor_txg: 35,
            root_ring_slot_count: 24,
        };
        assert!(!window.contains(11), "35 − 24 = 11 不在窗口里");
        assert!(window.contains(12), "12 在窗口里");
        assert!(window.contains(35), "N 自己在窗口里");
        let young_ring = OneRevolutionWindow {
            anchor_txg: 7,
            root_ring_slot_count: 24,
        };
        assert!(young_ring.contains(0), "N < 环长：第 0 代也在窗口里");
    }

    // ---- 丁-defer 选根式的小世界（遗留判决 `research/prompts/m3-leftover-code-r1-main-verification.md` LR5） ----
    //
    // 取值域与上界（穷举的范围就是这几条）：
    // - 一块盘（盘 0），单元槽 0..SMALL_WORLD_UNIT_SLOT_COUNT（2 个），每个单元跨 1 槽；一版引用的单元是这 2 槽的任一子集。
    // - 根环 SMALL_WORLD_ROOT_RING_SLOT_COUNT（4）个位置，txg t 的根落在位置 t mod 4（与 `root_ring::target_for_publish` 一样按 txg 取槽），
    //   位置上一条根被同余的新 txg 盖掉就出了环（回收门槛与候选集取环里全部根，RGall；小世界里发布不跳号，环里的根本来都在一圈之内）。
    // - 从 mkfs（实例 1、txg 1、引用槽 0、F = 0）起，最多施加 SMALL_WORLD_OPERATION_LIMIT（5）步，每步三种之一：
    //   发布（同一实例 txg + 1，引用换成「现行引用 ∪ 可分配的槽」的任一子集：换下的记成已释放、释放代 = 新 txg，新引用的记成已分配）；
    //   抬 F（F 抬到 (F, 现行 txg] 里任一值：先写系统配置，再推一次带新 F 的空发布）；
    //   崩溃恢复落到环里一条根 L（按现行那一版的表有效、txg ≥ F，含现行那一版自己）：环里 txg 高于 L 的根
    //   所属的实例各记一行 (实例, L 的 txg)、并进 L 的表；新实例代号 = 用过的最大 + 1，写行 txg = 环里最大 txg + 1，写行那次发布从 L 起，
    //   引用照「发布」那样任选。
    //   可分配的槽 = 现行没引用、账里没有记录或已释放且释放代 ≤ 回收门槛（max(F, 环里最旧有效根)）、
    //   环里按现行那一版的表被抛弃的根都不引用（影子账隔离）的槽。
    // - 判的那一刻，环里有根的位置任取一个子集读不出（含现行那一版自己的根槽：那时最新根换成读得出的最大那条，账与表都取它的）。
    // - 不在小世界里的：由记录施加出来、根槽从没落盘的那一版（候选集另补的那一种），读得出而走读读到一半断掉的根，
    //   系统配置读不出（F 生效值只取根上的）。
    //
    // 判法：每个（盘面、读不出的位置子集）上，按 `deferred_slots_reclaim_threshold`、
    // `allocation_record_is_released_above_the_reclaim_threshold`、`deferred_slots_among_the_references_of_the_selected_roots`
    // 各算一遍丁-defer 并进的槽：选根一次用 `root_references_are_merged_into_the_deferred_slots_below_the_floor`（原式），
    // 一次用「被抛弃 ∨ txg < F」（被抛弃根引用的也并，LR5 的那一式）。两式只在这一形下可能不同：
    // 多并的槽 S 在最新根的账里是已释放、释放代 g，换下它的那一版 r（释放之前的现行那一版，引用 S）的根还在环里、而那一槽读不出——
    // 一次故障是把根抛弃的那次恢复（读坏或崩溃），另一次是 r 的根槽读不出。r 读得出时：r ≥ F 就被候选集走到；
    // r < F 原式已并进 S；r 出了环，环里有效的根都 ≥ g、门槛 ≥ g，S 不算 defer。

    const SMALL_WORLD_UNIT_SLOT_COUNT: u64 = 2;
    const SMALL_WORLD_ROOT_RING_SLOT_COUNT: u64 = 4;
    const SMALL_WORLD_OPERATION_LIMIT: usize = 5;
    const SMALL_WORLD_DEVICE: u32 = 0;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallWorldVersion {
        instance: u32,
        checkpoint_txg: u64,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum SmallWorldRecord {
        Allocated,
        Released {
            release_generation: u64,
            /// 换下这一槽的那次发布之前的现行那一版（引用着这一槽）：只给判例外那一形用，盘上没有这个字段。
            version_released_from: SmallWorldVersion,
        },
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallWorldRoot {
        version: SmallWorldVersion,
        rollback_floor: u64,
        referenced_unit_slots: BTreeSet<u64>,
        account: BTreeMap<u64, SmallWorldRecord>,
        /// 实例表：实例 → 它最后一版的 txg Ti；(i, T) 被抛弃 ⟺ 有 i 的行且 T > Ti。
        instance_table_rows: BTreeMap<u32, u64>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallWorldPool {
        /// 根环位置 → 住在那里的根。
        ring: BTreeMap<u64, SmallWorldRoot>,
        current: SmallWorldRoot,
        system_configuration_floor: u64,
        highest_instance_used: u32,
    }

    fn small_world_standing(
        instance_table_rows: &BTreeMap<u32, u64>,
        version: SmallWorldVersion,
    ) -> RootStandingInTheNewestRootsTable {
        match instance_table_rows.get(&version.instance) {
            Some(last_txg_of_the_instance)
                if version.checkpoint_txg > *last_txg_of_the_instance =>
            {
                RootStandingInTheNewestRootsTable::Abandoned
            }
            Some(_) | None => RootStandingInTheNewestRootsTable::Valid,
        }
    }

    /// 写进一条新根之前那一刻可分配的槽：门槛按 `instance_table_rows` 判有效、取环里全部有效根。
    fn small_world_allocatable_unit_slots(
        pool: &SmallWorldPool,
        base: &SmallWorldRoot,
        instance_table_rows: &BTreeMap<u32, u64>,
    ) -> BTreeSet<u64> {
        let reclaim_threshold = deferred_slots_reclaim_threshold(
            pool.ring.values().map(|root| {
                (
                    small_world_standing(instance_table_rows, root.version),
                    root.version.checkpoint_txg,
                )
            }),
            pool.system_configuration_floor,
        );
        let isolated_by_the_shadow_ledger: BTreeSet<u64> = pool
            .ring
            .values()
            .filter(|root| {
                small_world_standing(instance_table_rows, root.version)
                    == RootStandingInTheNewestRootsTable::Abandoned
            })
            .flat_map(|root| root.referenced_unit_slots.iter().copied())
            .collect();
        (0..SMALL_WORLD_UNIT_SLOT_COUNT)
            .filter(|unit_slot| {
                !base.referenced_unit_slots.contains(unit_slot)
                    && !isolated_by_the_shadow_ledger.contains(unit_slot)
                    && match base.account.get(unit_slot) {
                        None => true,
                        Some(SmallWorldRecord::Allocated) => false,
                        Some(SmallWorldRecord::Released {
                            release_generation, ..
                        }) => *release_generation <= reclaim_threshold,
                    }
            })
            .collect()
    }

    /// `candidates` 的全部子集。
    fn small_world_subsets(candidates: &BTreeSet<u64>) -> Vec<BTreeSet<u64>> {
        let members: Vec<u64> = candidates.iter().copied().collect();
        (0..1u32 << members.len())
            .map(|membership_mask| {
                members
                    .iter()
                    .enumerate()
                    .filter(|(member_index, _)| membership_mask & (1 << member_index) != 0)
                    .map(|(_, unit_slot)| *unit_slot)
                    .collect()
            })
            .collect()
    }

    /// 从 `base` 那一版发一版新的、写进根环、成为现行那一版。
    fn small_world_publish(
        pool: &SmallWorldPool,
        base: &SmallWorldRoot,
        version: SmallWorldVersion,
        referenced_unit_slots: BTreeSet<u64>,
        instance_table_rows: BTreeMap<u32, u64>,
    ) -> SmallWorldPool {
        let mut account = base.account.clone();
        for released in base
            .referenced_unit_slots
            .difference(&referenced_unit_slots)
        {
            account.insert(
                *released,
                SmallWorldRecord::Released {
                    release_generation: version.checkpoint_txg,
                    version_released_from: base.version,
                },
            );
        }
        for allocated in referenced_unit_slots.difference(&base.referenced_unit_slots) {
            account.insert(*allocated, SmallWorldRecord::Allocated);
        }
        let root = SmallWorldRoot {
            version,
            rollback_floor: pool.system_configuration_floor,
            referenced_unit_slots,
            account,
            instance_table_rows,
        };
        let mut next = pool.clone();
        next.ring.insert(
            version.checkpoint_txg % SMALL_WORLD_ROOT_RING_SLOT_COUNT,
            root.clone(),
        );
        next.current = root;
        next
    }

    fn small_world_successors(pool: &SmallWorldPool) -> Vec<SmallWorldPool> {
        let mut successors = Vec::new();
        let current = &pool.current;
        let next_version_of_this_instance = SmallWorldVersion {
            instance: current.version.instance,
            checkpoint_txg: current.version.checkpoint_txg + 1,
        };
        // 发布。
        let allocatable =
            small_world_allocatable_unit_slots(pool, current, &current.instance_table_rows);
        let reachable: BTreeSet<u64> = current
            .referenced_unit_slots
            .union(&allocatable)
            .copied()
            .collect();
        for referenced_unit_slots in small_world_subsets(&reachable) {
            successors.push(small_world_publish(
                pool,
                current,
                next_version_of_this_instance,
                referenced_unit_slots,
                current.instance_table_rows.clone(),
            ));
        }
        // 抬 F：先写系统配置，再推一次带新 F 的空发布。
        for new_floor in pool.system_configuration_floor + 1..=current.version.checkpoint_txg {
            let mut raised = pool.clone();
            raised.system_configuration_floor = new_floor;
            successors.push(small_world_publish(
                &raised,
                current,
                next_version_of_this_instance,
                current.referenced_unit_slots.clone(),
                current.instance_table_rows.clone(),
            ));
        }
        // 崩溃恢复落到环里一条根 L。
        let highest_txg_in_the_ring = pool
            .ring
            .values()
            .map(|root| root.version.checkpoint_txg)
            .max()
            .expect("环里至少有现行那一版");
        for landed in pool.ring.values() {
            let landed_txg = landed.version.checkpoint_txg;
            let can_land = small_world_standing(&current.instance_table_rows, landed.version)
                == RootStandingInTheNewestRootsTable::Valid
                && landed_txg >= pool.system_configuration_floor;
            if !can_land {
                continue;
            }
            let mut instance_table_rows = landed.instance_table_rows.clone();
            for newer in pool
                .ring
                .values()
                .filter(|root| root.version.checkpoint_txg > landed_txg)
            {
                let row = instance_table_rows
                    .entry(newer.version.instance)
                    .or_insert(landed_txg);
                *row = (*row).min(landed_txg);
            }
            let mut recovered = pool.clone();
            recovered.highest_instance_used += 1;
            recovered.current = landed.clone();
            let row_publish_version = SmallWorldVersion {
                instance: recovered.highest_instance_used,
                checkpoint_txg: highest_txg_in_the_ring + 1,
            };
            let allocatable_after_the_recovery =
                small_world_allocatable_unit_slots(&recovered, landed, &instance_table_rows);
            let reachable_after_the_recovery: BTreeSet<u64> = landed
                .referenced_unit_slots
                .union(&allocatable_after_the_recovery)
                .copied()
                .collect();
            for referenced_unit_slots in small_world_subsets(&reachable_after_the_recovery) {
                successors.push(small_world_publish(
                    &recovered,
                    landed,
                    row_publish_version,
                    referenced_unit_slots,
                    instance_table_rows.clone(),
                ));
            }
        }
        successors
    }

    fn small_world_slot_keys(unit_slots: &BTreeSet<u64>) -> impl Iterator<Item = (u32, u64)> + '_ {
        unit_slots
            .iter()
            .map(|unit_slot| (SMALL_WORLD_DEVICE, *unit_slot))
    }

    /// 两式在一个（盘面、读不出的位置）上各并进的槽，连同判的那一刻的最新根。
    struct SmallWorldDeferredSlotsOfBothSelections<'pool> {
        original: BTreeSet<(u32, u64)>,
        merging_the_abandoned_roots_too: BTreeSet<(u32, u64)>,
        newest: &'pool SmallWorldRoot,
    }

    /// 两式在一个（盘面、读不出的位置）上各并进哪些槽。读不出之后环里一条根都不剩时交 `None`。
    fn small_world_deferred_slots_of_both_selections<'pool>(
        pool: &'pool SmallWorldPool,
        unreadable_positions: &BTreeSet<u64>,
    ) -> Option<SmallWorldDeferredSlotsOfBothSelections<'pool>> {
        let readable: Vec<&SmallWorldRoot> = pool
            .ring
            .iter()
            .filter(|(position, _)| !unreadable_positions.contains(position))
            .map(|(_, root)| root)
            .collect();
        let newest = *readable
            .iter()
            .max_by_key(|root| (root.version.checkpoint_txg, root.version.instance))?;
        let standing =
            |version: SmallWorldVersion| small_world_standing(&newest.instance_table_rows, version);
        let effective_rollback_floor = readable
            .iter()
            .filter(|root| {
                root.version == newest.version
                    || standing(root.version) == RootStandingInTheNewestRootsTable::Abandoned
            })
            .map(|root| root.rollback_floor)
            .fold(pool.system_configuration_floor, u64::max);
        let walked_slots: BTreeSet<(u32, u64)> = readable
            .iter()
            .filter(|root| {
                root.version == newest.version
                    || (standing(root.version) == RootStandingInTheNewestRootsTable::Valid
                        && root.version.checkpoint_txg >= effective_rollback_floor)
            })
            .flat_map(|root| small_world_slot_keys(&root.referenced_unit_slots))
            .collect();
        let reclaim_threshold = deferred_slots_reclaim_threshold(
            readable
                .iter()
                .map(|root| (standing(root.version), root.version.checkpoint_txg)),
            effective_rollback_floor,
        );
        let released_above_the_reclaim_threshold: BTreeSet<(u32, u64)> = newest
            .account
            .iter()
            .filter(|(unit_slot, record)| {
                let (generation, is_released) = match record {
                    SmallWorldRecord::Allocated => (0, false),
                    SmallWorldRecord::Released {
                        release_generation, ..
                    } => (*release_generation, true),
                };
                allocation_record_is_released_above_the_reclaim_threshold(
                    &AllocationRecordView {
                        device: SMALL_WORLD_DEVICE,
                        slot: **unit_slot,
                        span_slots: 1,
                        generation,
                        is_released,
                    },
                    reclaim_threshold,
                )
            })
            .map(|(unit_slot, _)| (SMALL_WORLD_DEVICE, *unit_slot))
            .collect();
        let original = deferred_slots_among_the_references_of_the_selected_roots(
            readable
                .iter()
                .filter(|root| {
                    root_references_are_merged_into_the_deferred_slots_below_the_floor(
                        standing(root.version),
                        root.version.checkpoint_txg,
                        effective_rollback_floor,
                    )
                })
                .flat_map(|root| small_world_slot_keys(&root.referenced_unit_slots)),
            &walked_slots,
            &released_above_the_reclaim_threshold,
        );
        // LR5 那一式：被抛弃 ∨ txg < F（不经原式的选根函数，原式被改坏时这一式不跟着变）。
        let merging_the_abandoned_roots_too =
            deferred_slots_among_the_references_of_the_selected_roots(
                readable
                    .iter()
                    .filter(|root| {
                        standing(root.version) == RootStandingInTheNewestRootsTable::Abandoned
                            || root.version.checkpoint_txg < effective_rollback_floor
                    })
                    .flat_map(|root| small_world_slot_keys(&root.referenced_unit_slots)),
                &walked_slots,
                &released_above_the_reclaim_threshold,
            );
        Some(SmallWorldDeferredSlotsOfBothSelections {
            original,
            merging_the_abandoned_roots_too,
            newest,
        })
    }

    /// LR5：「被抛弃根引用的也并」与原式在小世界的每个盘面上并进同一组槽，只在双故障形下可能不同
    /// （取值域、上界、两式与双故障形的写法见上面那一段注释）。
    #[test]
    fn deferred_selection_merging_the_abandoned_roots_too_selects_the_same_slots_on_every_small_world_but_the_double_fault_form(
    ) {
        let mkfs_root = SmallWorldRoot {
            version: SmallWorldVersion {
                instance: 1,
                checkpoint_txg: 1,
            },
            rollback_floor: 0,
            referenced_unit_slots: BTreeSet::from([0]),
            account: BTreeMap::from([(0, SmallWorldRecord::Allocated)]),
            instance_table_rows: BTreeMap::new(),
        };
        let mkfs_pool = SmallWorldPool {
            ring: BTreeMap::from([(1, mkfs_root.clone())]),
            current: mkfs_root,
            system_configuration_floor: 0,
            highest_instance_used: 1,
        };
        let mut seen: BTreeSet<SmallWorldPool> = BTreeSet::from([mkfs_pool.clone()]);
        let mut frontier = vec![mkfs_pool];
        for _ in 0..SMALL_WORLD_OPERATION_LIMIT {
            let mut next_frontier = Vec::new();
            for pool in &frontier {
                for successor in small_world_successors(pool) {
                    if seen.insert(successor.clone()) {
                        next_frontier.push(successor);
                    }
                }
            }
            frontier = next_frontier;
        }
        let mut judged_worlds = 0u64;
        let mut worlds_where_the_original_merges_something = 0u64;
        let mut worlds_in_the_double_fault_form_where_the_two_differ = 0u64;
        for pool in &seen {
            let occupied_positions: BTreeSet<u64> = pool.ring.keys().copied().collect();
            for unreadable_positions in small_world_subsets(&occupied_positions) {
                let Some(SmallWorldDeferredSlotsOfBothSelections {
                    original,
                    merging_the_abandoned_roots_too,
                    newest,
                }) = small_world_deferred_slots_of_both_selections(pool, &unreadable_positions)
                else {
                    continue;
                };
                judged_worlds += 1;
                if !original.is_empty() {
                    worlds_where_the_original_merges_something += 1;
                }
                if original == merging_the_abandoned_roots_too {
                    continue;
                }
                for (_, unit_slot) in
                    original.symmetric_difference(&merging_the_abandoned_roots_too)
                {
                    let version_released_from = match newest.account.get(unit_slot) {
                        Some(SmallWorldRecord::Released {
                            version_released_from,
                            ..
                        }) => *version_released_from,
                        Some(SmallWorldRecord::Allocated) | None => panic!(
                            "并进来的槽 {unit_slot} 在最新根的账里不是已释放：{pool:?}，读不出 {unreadable_positions:?}"
                        ),
                    };
                    let position =
                        version_released_from.checkpoint_txg % SMALL_WORLD_ROOT_RING_SLOT_COUNT;
                    let releasing_parent_is_in_the_ring_and_unreadable = pool
                        .ring
                        .get(&position)
                        .is_some_and(|root| root.version == version_released_from)
                        && unreadable_positions.contains(&position);
                    assert!(
                        releasing_parent_is_in_the_ring_and_unreadable,
                        "两式在槽 {unit_slot} 上不同，而换下它的那一版 {version_released_from:?} 的根不是「在环里、读不出」（推翻 LR5 的等价）：\
                         原式 {original:?}，被抛弃根引用的也并 {merging_the_abandoned_roots_too:?}；盘面 {pool:?}，读不出的位置 {unreadable_positions:?}"
                    );
                }
                worlds_in_the_double_fault_form_where_the_two_differ += 1;
            }
        }
        assert_eq!(
            (
                seen.len(),
                judged_worlds,
                worlds_where_the_original_merges_something,
                worlds_in_the_double_fault_form_where_the_two_differ,
            ),
            (180_794, 2_710_980, 10_204, 1_082),
            "盘面数、判过的（盘面、读不出的位置）数、原式并进了槽的数、双故障形下两式不同的数"
        );
    }

    /// 默认 768 MiB 环的单元区起始槽号（I-5.5 那几条合成用例的 U）。
    const UNIT_AREA_START_SLOT_OF_THE_DEFAULT_JOURNAL_RING: u64 = 50176;

    /// 盘 0 上一条分配记录。
    fn allocation_record_on_device_zero(
        slot: u64,
        span_slots: u64,
        generation: u64,
        is_released: bool,
    ) -> AllocationRecordView {
        AllocationRecordView {
            device: 0,
            slot,
            span_slots,
            generation,
            is_released,
        }
    }

    /// I-5.5 的落点不进开放段（D3（空间分配） 已定项 10 ②）：U 起的那一段 64 槽全被更早的版本占着，下一段是这一版的开放段
    /// （树表单元在里面），落点越过开放段、落在再下一段的起点。
    #[test]
    fn the_first_data_unit_slot_skips_the_open_cluster_segment_of_the_version() {
        let unit_area_start = UNIT_AREA_START_SLOT_OF_THE_DEFAULT_JOURNAL_RING;
        let version_checkpoint_txg = 7;
        let allocation_records = [
            allocation_record_on_device_zero(unit_area_start, CLUSTER_SEGMENT_SLOTS, 5, false),
            allocation_record_on_device_zero(
                unit_area_start + CLUSTER_SEGMENT_SLOTS + 12,
                1,
                version_checkpoint_txg,
                false,
            ),
        ];
        assert_eq!(
            first_data_unit_slot_the_allocation_records_leave_on_a_device(
                &allocation_records,
                0,
                version_checkpoint_txg,
                unit_area_start,
                Some(OpenClusterSegment {
                    first_slot: unit_area_start + CLUSTER_SEGMENT_SLOTS,
                }),
            ),
            unit_area_start + 2 * CLUSTER_SEGMENT_SLOTS,
            "U 那一段占满、下一段是开放段：落点是再下一段的起点"
        );
    }

    /// 已释放的记录一律当占着（回收门槛复算不出的那个近似）：U、U + 1 与 U + 2 两条记录是这一版自己释放的（释放代等于这一版的 txg），
    /// 落点不取 U，取 U + 4。
    #[test]
    fn a_released_allocation_record_keeps_its_slots_out_of_the_first_data_unit_slot() {
        let unit_area_start = UNIT_AREA_START_SLOT_OF_THE_DEFAULT_JOURNAL_RING;
        let version_checkpoint_txg = 29;
        let allocation_records = [
            allocation_record_on_device_zero(unit_area_start, 2, version_checkpoint_txg, true),
            allocation_record_on_device_zero(unit_area_start + 2, 1, version_checkpoint_txg, true),
        ];
        assert_eq!(
            first_data_unit_slot_the_allocation_records_leave_on_a_device(
                &allocation_records,
                0,
                version_checkpoint_txg,
                unit_area_start,
                None,
            ),
            unit_area_start + 4,
            "U、U + 1 与 U + 2 已释放（当占着）：落点是 U + 4"
        );
    }

    /// 一条字段表宽（200 字节）的树表条目，只填 I-9.27 读的三样：树 ID（偏移 0）、种类（偏移 10）、头 ID（偏移 116）。
    fn tree_table_entry_with_head_identifier(
        tree: u64,
        kind: u16,
        head_identifier: u64,
    ) -> Vec<u8> {
        let mut entry = vec![0u8; tree_table_entry_bytes()];
        entry[..8].copy_from_slice(&tree.to_le_bytes());
        entry[TREE_TABLE_ENTRY_KIND_OFFSET..TREE_TABLE_ENTRY_KIND_OFFSET + 2]
            .copy_from_slice(&kind.to_le_bytes());
        entry[TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET..TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8]
            .copy_from_slice(&head_identifier.to_le_bytes());
        entry
    }

    /// 一张树表上 I-9.27 的判定。
    fn head_identifier_verdict_of(tree_table_entries: &[Vec<u8>]) -> InvariantVerdict {
        let mut judgements = Judgements::default();
        judge_tree_table_entry_head_identifiers(
            &mut judgements,
            tree_table_entries,
            "合成的那一版",
        );
        judgements
            .into_report()
            .into_iter()
            .find(|(invariant, _)| *invariant == HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES)
            .expect("I-9.27 在清单里")
            .1
    }

    /// I-9.27 多于一条种类 2 条目时放宽成「等于其中某一条的树 ID」（`.claude/kb/invariants.md` I-9.27 那一行，收严留给多头那一批）：
    /// 表里两条 inode 树（65537、65549），extent 树条目的头 ID 写第二条的树 ID 65549——成立；只认第一条的读法在这里误红。
    #[test]
    fn an_extent_entry_naming_the_second_of_two_inode_tree_entries_holds_the_head_identifier_relation(
    ) {
        let tree_table_entries = [
            tree_table_entry_with_head_identifier(65536, TREE_KIND_EXTENT, 65549),
            tree_table_entry_with_head_identifier(65537, TREE_KIND_INODE, 65537),
            tree_table_entry_with_head_identifier(65549, TREE_KIND_INODE, 65549),
        ];
        assert_eq!(
            head_identifier_verdict_of(&tree_table_entries),
            InvariantVerdict::Holds,
            "extent 树条目的头 ID 等于两条 inode 树条目里第二条的树 ID：放宽之后成立"
        );
    }

    /// 同一张两条 inode 树的表，extent 树条目的头 ID 写成两条都不是的 65550：放宽之后照样红。
    #[test]
    fn an_extent_entry_naming_neither_of_two_inode_tree_entries_violates_the_head_identifier_relation(
    ) {
        let tree_table_entries = [
            tree_table_entry_with_head_identifier(65536, TREE_KIND_EXTENT, 65550),
            tree_table_entry_with_head_identifier(65537, TREE_KIND_INODE, 65537),
            tree_table_entry_with_head_identifier(65549, TREE_KIND_INODE, 65549),
        ];
        assert!(
            matches!(
                head_identifier_verdict_of(&tree_table_entries),
                InvariantVerdict::Violated(_)
            ),
            "extent 树条目的头 ID 不等于表里任何一条 inode 树条目的树 ID：判红"
        );
    }

    // ───── 第 57 批 H1、H2、H3（判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节）：容器容量的两个判法直接交 (容器号, 记录数) 表 ─────

    /// 容量判法判的三棵容器树。
    #[derive(Clone, Copy, Debug)]
    enum ContainerTreeOfTheCapacityJudges {
        Tombstone,
        Lineage,
        Retirement,
    }

    impl ContainerTreeOfTheCapacityJudges {
        /// 一片容器的产品容量（墓碑 815、谱系 1359、退役 959）。
        fn product_records_per_container(self) -> u64 {
            match self {
                ContainerTreeOfTheCapacityJudges::Tombstone => TOMBSTONE_CONTAINER_RECORDS,
                ContainerTreeOfTheCapacityJudges::Lineage => {
                    LineageOrRetirementTree::Lineage.product_records_per_container()
                }
                ContainerTreeOfTheCapacityJudges::Retirement => {
                    LineageOrRetirementTree::Retirement.product_records_per_container()
                }
            }
        }

        /// 装没装旋钮时判法判的那一条（墓碑 I-9.21 / I-9.22，谱系与退役 I-9.36 / I-9.37）。
        fn capacity_invariant(self, knob_records_per_container: Option<u64>) -> &'static str {
            match self {
                ContainerTreeOfTheCapacityJudges::Tombstone => capacity_invariant_of(
                    knob_records_per_container.map(tombstone_container_capacity_knob_of),
                ),
                ContainerTreeOfTheCapacityJudges::Lineage
                | ContainerTreeOfTheCapacityJudges::Retirement => {
                    lineage_or_retirement_capacity_invariant_of(knob_records_per_container)
                }
            }
        }
    }

    fn tombstone_container_capacity_knob_of(
        records_per_container: u64,
    ) -> TombstoneContainerCapacityKnob {
        TombstoneContainerCapacityKnob::new(records_per_container).expect("旋钮值在 1..=815 里")
    }

    /// 一条整对象墓碑（容量判法只数条数，不读字段）。
    fn whole_object_tombstone_for_the_capacity_judge() -> TombstoneRecordOfTheScan {
        TombstoneRecordOfTheScan {
            object: 2,
            object_birth: 3,
            range_start: TOMBSTONE_WHOLE_OBJECT_RANGE_START,
            range_end: TOMBSTONE_WHOLE_OBJECT_RANGE_END,
            death_txg: 4,
        }
    }

    /// 把沿叶序的 (容器号, 记录数) 表直接交这一棵的容量判法（墓碑树交 [`judge_tombstone_container_capacity_of_a_version`]，
    /// 谱系树与退役树交 [`judge_lineage_or_retirement_container_capacity_of_a_version`]），旋钮值 `knob_records_per_container`（`None` 是没装）。
    fn capacity_judgements_of_the_table(
        tree: ContainerTreeOfTheCapacityJudges,
        container_numbers_and_record_counts: &[(u64, u64)],
        knob_records_per_container: Option<u64>,
    ) -> Judgements {
        let mut judgements = Judgements::default();
        let record_count_as_usize = |record_count: u64| {
            usize::try_from(record_count).expect("一片容器的记录数装得进 usize")
        };
        let judge_the_lineage_or_retirement_table =
            |tree_kind: LineageOrRetirementTree,
             tree_identifier: u64,
             judgements_of_the_table: &mut Judgements| {
                let table: Vec<(u64, usize)> = container_numbers_and_record_counts
                    .iter()
                    .map(|(container, record_count)| {
                        (*container, record_count_as_usize(*record_count))
                    })
                    .collect();
                judge_lineage_or_retirement_container_capacity_of_a_version(
                    "单测那一版",
                    tree_kind,
                    tree_identifier,
                    &table,
                    knob_records_per_container,
                    judgements_of_the_table,
                );
            };
        match tree {
            ContainerTreeOfTheCapacityJudges::Tombstone => {
                let containers: Vec<TombstoneContainerOfTheScan> =
                    container_numbers_and_record_counts
                        .iter()
                        .map(|(container, record_count)| TombstoneContainerOfTheScan {
                            container: *container,
                            records: vec![
                                whole_object_tombstone_for_the_capacity_judge();
                                record_count_as_usize(*record_count)
                            ],
                        })
                        .collect();
                judge_tombstone_container_capacity_of_a_version(
                    "单测那一版",
                    TREE_IDENTIFIER_TOMBSTONE,
                    &containers,
                    knob_records_per_container.map(tombstone_container_capacity_knob_of),
                    &mut judgements,
                );
            }
            ContainerTreeOfTheCapacityJudges::Lineage => judge_the_lineage_or_retirement_table(
                LineageOrRetirementTree::Lineage,
                TREE_IDENTIFIER_LINEAGE,
                &mut judgements,
            ),
            ContainerTreeOfTheCapacityJudges::Retirement => judge_the_lineage_or_retirement_table(
                LineageOrRetirementTree::Retirement,
                singlefs_format::TREE_IDENTIFIER_RETIREMENT,
                &mut judgements,
            ),
        }
        judgements
    }

    /// 这张表交这一棵的容量判法：那一条只红一处（只一句不成立），违例说明里有 `detail_fragment`。
    fn assert_the_capacity_judge_violates_only_one_sentence(
        tree: ContainerTreeOfTheCapacityJudges,
        container_numbers_and_record_counts: &[(u64, u64)],
        knob_records_per_container: Option<u64>,
        detail_fragment: &str,
    ) {
        let invariant = tree.capacity_invariant(knob_records_per_container);
        let judgements = capacity_judgements_of_the_table(
            tree,
            container_numbers_and_record_counts,
            knob_records_per_container,
        );
        assert_eq!(
            judgements.violation_count(invariant),
            1,
            "{tree:?} 表 {container_numbers_and_record_counts:?}（旋钮 {knob_records_per_container:?}）：{invariant} 只一句不成立（第一处：{:?}）",
            judgements.first_violation_of(invariant)
        );
        assert!(
            judgements
                .first_violation_of(invariant)
                .is_some_and(|detail| detail.contains(detail_fragment)),
            "{tree:?} 表 {container_numbers_and_record_counts:?}：{invariant} 的违例说明写出「{detail_fragment}」，实际 {:?}",
            judgements.first_violation_of(invariant)
        );
    }

    /// 这张表交这一棵的容量判法：那一条判过、一处都不红。
    fn assert_the_capacity_judge_holds(
        tree: ContainerTreeOfTheCapacityJudges,
        container_numbers_and_record_counts: &[(u64, u64)],
        knob_records_per_container: Option<u64>,
    ) {
        let invariant = tree.capacity_invariant(knob_records_per_container);
        let judgements = capacity_judgements_of_the_table(
            tree,
            container_numbers_and_record_counts,
            knob_records_per_container,
        );
        assert_eq!(
            judgements.violation_count(invariant),
            0,
            "{tree:?} 表 {container_numbers_and_record_counts:?}（旋钮 {knob_records_per_container:?}）：{invariant} 一处都不红（第一处：{:?}）",
            judgements.first_violation_of(invariant)
        );
        assert!(
            matches!(
                judgements
                    .into_report()
                    .into_iter()
                    .find(|(name, _)| *name == invariant)
                    .map(|(_, verdict)| verdict),
                Some(InvariantVerdict::Holds)
            ),
            "{tree:?} 表 {container_numbers_and_record_counts:?}：{invariant} 判过而成立"
        );
    }

    /// H2 产品支（没装旋钮）的对照：第一片装满产品容量、号 1、2，最右那片 1 条：判过、一处不红（下面三形各只改一处）。
    fn assert_a_full_first_container_and_one_more_hold_on_the_product_branch(
        tree: ContainerTreeOfTheCapacityJudges,
    ) {
        let full = tree.product_records_per_container();
        assert_the_capacity_judge_holds(tree, &[(1, full), (2, 1)], None);
    }

    /// H2 产品支第一形（攻方 `mut-w-start2`）：第一片装满、号 2、3——容量那句成立，只「第一片容器号是 1」那句不成立。
    fn assert_product_containers_numbered_two_and_three_violate_only_the_first_number_sentence(
        tree: ContainerTreeOfTheCapacityJudges,
    ) {
        let full = tree.product_records_per_container();
        assert_the_capacity_judge_violates_only_one_sentence(
            tree,
            &[(2, full), (3, 1)],
            None,
            "沿叶序第一片容器的容器号是 2，不是 1",
        );
    }

    /// H2 产品支第二形（攻方 `mut-w-gap2`）：第一片装满、号 1、3——只「相邻两片加一」那句不成立。
    fn assert_product_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
        tree: ContainerTreeOfTheCapacityJudges,
    ) {
        let full = tree.product_records_per_container();
        assert_the_capacity_judge_violates_only_one_sentence(
            tree,
            &[(1, full), (3, 1)],
            None,
            "沿叶序容器号不是逐片加一：容器 1 之后是容器 3",
        );
    }

    /// H2 产品支第三形（攻方 `mut-w-eager`）：第一片装满、号 1、2，最右那片 0 条——只「每片至少一条」那句不成立。
    fn assert_product_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
        tree: ContainerTreeOfTheCapacityJudges,
    ) {
        let full = tree.product_records_per_container();
        assert_the_capacity_judge_violates_only_one_sentence(
            tree,
            &[(1, full), (2, 0)],
            None,
            "容器 2 一条记录都没有",
        );
    }

    /// H3 产品支：第一片装满、号 0、1——相邻加一成立，只「第一片容器号是 1」那句不成立（「第一片 ≤ 1」放过它）。
    fn assert_product_containers_numbered_zero_and_one_violate_only_the_first_number_sentence(
        tree: ContainerTreeOfTheCapacityJudges,
    ) {
        let full = tree.product_records_per_container();
        assert_the_capacity_judge_violates_only_one_sentence(
            tree,
            &[(0, full), (1, 1)],
            None,
            "沿叶序第一片容器的容器号是 0，不是 1",
        );
    }

    /// H1 旋钮支：旋钮 1 写出的两片各 1 条交旋钮 2 判（攻方 `k2_knob_bad_images` 那一形）——容器号与「≤ 旋钮值」都成立，
    /// 只「不是最右的那片装满旋钮值」那句不成立；同一张表交旋钮 1 判过、一处不红。
    fn assert_two_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
        tree: ContainerTreeOfTheCapacityJudges,
    ) {
        assert_the_capacity_judge_holds(tree, &[(1, 1), (2, 1)], Some(1));
        assert_the_capacity_judge_violates_only_one_sentence(
            tree,
            &[(1, 1), (2, 1)],
            Some(2),
            "不是最右的容器 1 装了 1 条，不是旋钮值 2",
        );
    }

    #[test]
    fn a_full_first_tombstone_container_and_one_more_hold_on_the_product_branch() {
        assert_a_full_first_container_and_one_more_hold_on_the_product_branch(
            ContainerTreeOfTheCapacityJudges::Tombstone,
        );
    }

    #[test]
    fn a_full_first_lineage_container_and_one_more_hold_on_the_product_branch() {
        assert_a_full_first_container_and_one_more_hold_on_the_product_branch(
            ContainerTreeOfTheCapacityJudges::Lineage,
        );
    }

    #[test]
    fn a_full_first_retirement_container_and_one_more_hold_on_the_product_branch() {
        assert_a_full_first_container_and_one_more_hold_on_the_product_branch(
            ContainerTreeOfTheCapacityJudges::Retirement,
        );
    }

    #[test]
    fn product_tombstone_containers_numbered_two_and_three_violate_only_the_first_number_sentence()
    {
        assert_product_containers_numbered_two_and_three_violate_only_the_first_number_sentence(
            ContainerTreeOfTheCapacityJudges::Tombstone,
        );
    }

    #[test]
    fn product_lineage_containers_numbered_two_and_three_violate_only_the_first_number_sentence() {
        assert_product_containers_numbered_two_and_three_violate_only_the_first_number_sentence(
            ContainerTreeOfTheCapacityJudges::Lineage,
        );
    }

    #[test]
    fn product_retirement_containers_numbered_two_and_three_violate_only_the_first_number_sentence()
    {
        assert_product_containers_numbered_two_and_three_violate_only_the_first_number_sentence(
            ContainerTreeOfTheCapacityJudges::Retirement,
        );
    }

    #[test]
    fn product_tombstone_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
    ) {
        assert_product_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
            ContainerTreeOfTheCapacityJudges::Tombstone,
        );
    }

    #[test]
    fn product_lineage_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence()
    {
        assert_product_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
            ContainerTreeOfTheCapacityJudges::Lineage,
        );
    }

    #[test]
    fn product_retirement_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
    ) {
        assert_product_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
            ContainerTreeOfTheCapacityJudges::Retirement,
        );
    }

    #[test]
    fn product_tombstone_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
    ) {
        assert_product_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
            ContainerTreeOfTheCapacityJudges::Tombstone,
        );
    }

    #[test]
    fn product_lineage_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence()
    {
        assert_product_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
            ContainerTreeOfTheCapacityJudges::Lineage,
        );
    }

    #[test]
    fn product_retirement_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
    ) {
        assert_product_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
            ContainerTreeOfTheCapacityJudges::Retirement,
        );
    }

    #[test]
    fn product_tombstone_containers_numbered_zero_and_one_violate_only_the_first_number_sentence() {
        assert_product_containers_numbered_zero_and_one_violate_only_the_first_number_sentence(
            ContainerTreeOfTheCapacityJudges::Tombstone,
        );
    }

    #[test]
    fn product_lineage_containers_numbered_zero_and_one_violate_only_the_first_number_sentence() {
        assert_product_containers_numbered_zero_and_one_violate_only_the_first_number_sentence(
            ContainerTreeOfTheCapacityJudges::Lineage,
        );
    }

    #[test]
    fn product_retirement_containers_numbered_zero_and_one_violate_only_the_first_number_sentence()
    {
        assert_product_containers_numbered_zero_and_one_violate_only_the_first_number_sentence(
            ContainerTreeOfTheCapacityJudges::Retirement,
        );
    }

    #[test]
    fn two_tombstone_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
    ) {
        assert_two_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
            ContainerTreeOfTheCapacityJudges::Tombstone,
        );
    }

    #[test]
    fn two_lineage_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
    ) {
        assert_two_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
            ContainerTreeOfTheCapacityJudges::Lineage,
        );
    }

    #[test]
    fn two_retirement_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
    ) {
        assert_two_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
            ContainerTreeOfTheCapacityJudges::Retirement,
        );
    }
}
