//! 池级 checker：从根环里每一条自证过的根走下去（遍历方向），再扫一遍单元头与 journal 环（扫描方向），判第一版那几条不变量
//! （清单与条数在 `image::IMPLEMENTED_INVARIANTS`，别在这里另抄一个数）。
//! 最新的那条根另判 I-7.2（能不能完整走完）；引用集合按全部有效根取并集——根环里更早的根仍是可回退的状态，
//! 它们引用的单元仍占着空间（I-3.1 与 I-5.1 的这一读法，见 records 2026-09-13-总审核 十一·七）。

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, ALLOCATION_RECORD_BYTES, ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES,
    CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES,
    EXTENT_TREE_INTERNAL_ENTRY_BYTES, EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, INODE_INTERNAL_ENTRY,
    JOURNAL_RECORD_BYTES, JOURNAL_SAFETY_FACTOR, LINEAGE_CONTAINER_RECORDS,
    LOGGED_OPERATIONS_ENTRY_BYTES, LOGGED_OPERATIONS_KEY_BYTES, MAPPING_ENTRY_BYTES,
    MAPPING_KEY_BYTES, NODE_BYTES, NODE_POINTER_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES,
    RETIREMENT_CONTAINER_RECORDS, ROOT_RECORD_NO_RING_SLOT_PINNED, SLOT_BYTES,
    TOMBSTONE_CONTAINER_RECORDS, TREE_IDENTIFIER_WATERMARK_AT_MKFS, TREE_TABLE_ENTRY_BYTES,
};

use crate::rebuild::{
    choose_current_versions, rebuild_index_tables, CentralMappingKey, DataUnitPosition,
    ObjectIdentity, PublishedUnitCopy, UnitPlacement,
};

use crate::position_addressed::{
    allocation_record_fits_in_the_leaf, allocation_record_tree_cell_range,
    allocation_record_tree_child_of_entry_key, allocation_record_tree_root_level,
    data_unit_payload_capacity_in_bytes, extent_lower_cell_range, extent_lower_child_of_entry_key,
    extent_lower_span_in_data_units, extent_upper_cell_range, extent_upper_child_of_entry_key,
    extent_upper_leaf_entry_view, extent_upper_span_in_inodes, AllocationRecordTreeCell,
    ExtentUpperLeafEntryView, TailAfterTheLastDataUnit,
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
    PointerLocation, PointerView, PoolGeometry, VerdictsNamingTheViolationSites,
    ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
    ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
    ALLOCATION_RECORD_TREE_ROOT_OF_A_VERSION_APPLIED_ONLY_BY_RECORDS_IS_BORN_WITH_THE_VERSION,
    COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
    DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
    DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
    DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
    EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY,
    FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
    FIRST_TXG_OF_EACH_INSTANCE_IS_ABOVE_EVERY_ROOT_AND_RECORD_OF_THE_OLDER_INSTANCES,
    FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
    FREE_ALLOCATED_AND_UNRECLAIMABLE_SUM_TO_THE_CAPACITY_AND_THE_COMMITTED_RESERVATION_MATCHES_ITS_CLOSED_FORM,
    HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
    INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
    INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
    INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
    INSTANCE_TABLE_ROW_HIGH_WATER_IS_ONE_A_RECOVERY_COULD_HAVE_APPLIED,
    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
    JOURNAL_RECORDS_LIVE_IN_THE_RING_SLOT_THEIR_COUNTER_NAMES,
    JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
    LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
    LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
    LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
    LIVE_SNAPSHOTS_ARE_ENUMERABLE, MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
    NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
    NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    NO_ALLOCATION_RECORD_TREE_IN_THE_TREE_TABLE_OF_THE_NEWEST_ROOT,
    NO_COMPACTION_INTENT_WITH_A_WELL_FORMED_REGION_IN_ANY_WALKED_VERSION,
    NO_DATA_UNIT_WITHIN_THE_SIZE_OF_A_RECORD_IN_ANY_WALKED_VERSION,
    NO_INODE_RECORD_IN_ANY_WALKED_VERSION, NO_INSTANCE_TABLE_ROW_UNDER_THE_NEWEST_ROOT,
    NO_INSTANCE_WITH_AN_OLDER_INSTANCE_TO_COMPARE,
    NO_INTENT_ENTRY_IN_THE_LOGGED_OPERATIONS_OF_ANY_WALKED_VERSION,
    NO_MAPPING_KEY_PLACED_DIFFERENTLY_IN_TWO_WALKED_VERSIONS,
    NO_RECORD_CARRYING_THE_QUARANTINE_MARK_UNDER_THE_NEWEST_ROOT,
    NO_SELF_VERIFIED_JOURNAL_RECORD_WITH_A_COUNTER_FROM_ONE,
    NO_UNRELEASED_ALLOCATION_RECORD_COVERING_A_USABLE_UNIT_HEADER,
    NO_VERSION_APPLIED_ONLY_BY_RECORDS_REWRITING_ITS_ALLOCATION_RECORD_TREE,
    ONE_OBJECT_BIRTH_PER_INODE_NUMBER, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
    PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE,
    POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER, PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
    QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
    REFERENCE_INTERVALS_ARE_EXACT,
    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
    SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
    SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
    SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY,
    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
    TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
    TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
    UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
    UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
    UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
    WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
};
use crate::{
    back_chain_of_record_header, check_index_node_keys, check_internal_node_separators,
    check_journal_record, check_separators_of_a_node_whose_subtree_coverage_the_walk_judges,
    check_unit, checksum_field_holds, crc32_castagnoli_table, data_unit_bytes, index_node_view,
    key_schema_for_tree_kind, packed_unit_view, read_six_byte_unsigned, read_u16, read_u32,
    read_u64, ContentUnitClass, NamedEntryView, Verdict, WriterImplementationIdentifier,
    FORMAT_VERSION_OFFSET, INODE_RECORD_OBJECT_BIRTH_OFFSET, INODE_RECORD_SIZE_OFFSET,
    INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET, INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET,
    KEY_SCHEMA_ACCOUNTING, KEY_SCHEMA_ALLOCATION, KEY_SCHEMA_CONTAINER_NUMBER, KEY_SCHEMA_DEADLIST,
    KEY_SCHEMA_EXTENT, KEY_SCHEMA_INODE, KEY_SCHEMA_LOGGED_OPERATIONS, KEY_SCHEMA_MAPPING,
    KEY_SCHEMA_TREE_TABLE, MAPPING_KEY_BIRTH_TREE_OFFSET, MAPPING_KEY_BIRTH_TXG_OFFSET,
    MAPPING_KEY_INSTANCE_OFFSET, MAPPING_KEY_TAIL_OFFSET, MAPPING_KEY_TAIL_PADDING,
    MAPPING_KEY_UNIT_CLASS_OFFSET, PACKED_TYPE_INODE, PACKED_TYPE_INSTANCE_TABLE,
    PACKED_TYPE_LINEAGE, PACKED_TYPE_RETIREMENT, PACKED_TYPE_TOMBSTONE,
    TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET, TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET,
    TOMBSTONE_RECORD_RANGE_END_OFFSET, TOMBSTONE_RECORD_RANGE_START_OFFSET,
    TOMBSTONE_WHOLE_OBJECT_RANGE_END, TOMBSTONE_WHOLE_OBJECT_RANGE_START,
    UNIT_FORMAT_VERSION_THIS_CHECKER_READS, UNIT_HEADER_CHECKSUM_OFFSET,
    WRITE_ORDER_TRANSACTION_OFFSET,
};

mod allocation_record_and_tree_birth_judgements;
mod capacity_sum_and_committed_reservation;
mod central_mapping_against_allocation_records;
mod deletion_state;
mod extent_and_allocation_record_tree_traversal;
mod inode_and_container_tree_traversal;
mod instance_first_txgs_and_high_water;
mod instance_table;
mod journal_publishes_and_replay;
mod journal_ring_scan;
mod logged_operation_intents_and_fragmentation;
mod pool_image_judgement;
mod published_predicate_and_total_order;
mod reference_interval_appearance_rule;
mod referenced_unit_identity;
mod references_without_judging;
mod rollback_floor_raises;
mod scan_rebuild_comparison;
mod snapshots_and_deadlists;
mod space_accounting_of_the_newest_root;
#[cfg(test)]
mod synthetic_images_for_tests;
mod tombstone_lineage_and_retirement_judgements;
mod tombstone_lineage_and_retirement_scan;
mod tree_table_and_central_mapping_traversal;
mod tree_table_entry_judgements;
mod unit_header_scan_and_instance_carriers;

pub use self::allocation_record_and_tree_birth_judgements::*;
pub use self::capacity_sum_and_committed_reservation::*;
use self::central_mapping_against_allocation_records::*;
use self::deletion_state::*;
use self::instance_first_txgs_and_high_water::*;
pub use self::instance_table::*;
use self::journal_publishes_and_replay::*;
pub use self::journal_ring_scan::*;
use self::logged_operation_intents_and_fragmentation::*;
pub use self::pool_image_judgement::*;
use self::published_predicate_and_total_order::*;
use self::reference_interval_appearance_rule::*;
use self::referenced_unit_identity::*;
use self::references_without_judging::*;
use self::rollback_floor_raises::*;
pub use self::scan_rebuild_comparison::*;
use self::snapshots_and_deadlists::*;
pub use self::space_accounting_of_the_newest_root::*;
#[cfg(test)]
use self::synthetic_images_for_tests::*;
pub use self::tombstone_lineage_and_retirement_judgements::*;
pub use self::tombstone_lineage_and_retirement_scan::*;
use self::tree_table_entry_judgements::*;
use self::unit_header_scan_and_instance_carriers::*;

const TREE_KIND_EXTENT: u16 = 1;
const TREE_KIND_INODE: u16 = 2;
const TREE_KIND_ALLOCATION: u16 = 3;
const TREE_KIND_ACCOUNTING: u16 = 4;
const TREE_KIND_LIVELIST: u16 = 6;
const TREE_KIND_SPARSE_SIDE_TABLE: u16 = 7;
const TREE_KIND_DEADLIST: u16 = 8;
/// 第一个文件版本那次零根注册的四棵树（D8（核心索引结构） 已定项 16 第 3 条、D18（块里携带什么信息） 已定项 24 第 1 / 5 / 6 条）的种类码，
/// 是 D8（核心索引结构） 已定项 9 登记表的码 9..12，checker 按自己读的条款另写一份。
const TREE_KIND_DELETED_INODES: u16 = 9;
const TREE_KIND_TOMBSTONE: u16 = 10;
const TREE_KIND_LINEAGE: u16 = 11;
const TREE_KIND_LOGGED_OPERATIONS: u16 = 12;
/// 退役树：码 13（D8（核心索引结构） 已定项 9 登记表；
/// 第三节第 1 条），checker 按自己读的条款另写一份。
const TREE_KIND_RETIREMENT: u16 = 13;
/// 只读快照的 inode 根与 extent 根：码 14、15（D8（核心索引结构） 已定项 9 登记表；判决
/// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1：一个快照两条，一张树表里可以有多对），checker 按自己读的条款另写一份。
const TREE_KIND_SNAPSHOT_INODE_ROOT: u16 = 14;
const TREE_KIND_SNAPSHOT_EXTENT_ROOT: u16 = 15;
/// 树表条目里 flags 2 字节的偏移（树 ID 8 + 条目长度 2 + 种类 2，D8（核心索引结构） 已定项 8 的字段表）。
const TREE_TABLE_ENTRY_FLAGS_OFFSET: usize = 8 + 2 + 2;
/// 码 14 条目 flags 位 0：这个快照销毁中（销毁意图写在被销毁快照
/// 自己的码 14 条目上，只占 flags 位 0；码 15 不带记号，跟着头 ID 配上的那条码 14 算）。从写意图那一刻起它不再钉住块
/// （D5（快照 / 空间记账机制） 已定项 20 的例外句，同一判决 S9）。
const TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED: u16 = 1;
/// deadlist 条目 43 字节里三段的偏移（D5（快照 / 空间记账机制） 已定项 12：头的树 ID 8 + 纪元 8 + 中央映射 key 27，value 空）。
const DEADLIST_ENTRY_HEAD_OFFSET: usize = 0;
const DEADLIST_ENTRY_EPOCH_OFFSET: usize = 8;
const DEADLIST_ENTRY_MAPPING_KEY_OFFSET: usize = 16;
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
/// `deleted_inodes` 根的条目：key = inode 号 8 字节、无 value（D8（核心索引结构） 已定项 16）。
const DELETED_INODES_ENTRY_BYTES: usize = 8;
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

/// 走一个 inode 树的孩子（叶容器，或类型段 0 指着的内部节点）交回的东西：它下面按叶序排的叶容器，这一枝的叶是不是全都到得了，
/// 以及这一遍是不是第一次走到它。
/// I-9.12（分隔 key 落在孩子区间之外） 与 I-9.4 的叶序都连别的版本已经走过的一起算（缓存的，不重读盘）：分隔 key 由这一版的父条目定，
/// 别的版本走那一遍判的是它们自己的父条目（`.claude/kb/checks-owed.md` C896 的口径）。
struct InodeSubtreeInTheWalk {
    leaves_in_leaf_order: Vec<InodeLeafContainerInLeafOrder>,
    every_leaf_reached: EveryLeafReached,
    walked_first_by_this_version: bool,
}

/// inode 树一个内部节点的一条条目在 I-9.12（分隔 key 落在孩子区间之外） 里要比的：这一版父条目的分隔 key，与这个孩子下面各片叶装的
/// 最小、最大 key（[`Walk::judge_separators_against_the_inode_children`]）。
struct SeparatorAndInodeChildRange {
    separator: u64,
    smallest_key_of_the_child: u64,
    largest_key_of_the_child: u64,
    /// 孩子是这一遍第一次走到的，还是别的版本先走过、区间取先走那一遍记下的叶序：只用在违例说明里。
    child_walked_first_by_this_version: bool,
}

impl SeparatorAndInodeChildRange {
    /// 违例说明里接在孩子的 key 之后的那半句：这一遍第一次走到的孩子不加，别的版本先走过的写明区间是缓存的叶序。
    const fn where_the_range_of_the_child_comes_from(&self) -> &'static str {
        if self.child_walked_first_by_this_version {
            ""
        } else {
            "（这个孩子别的版本先走过，区间取先走那一遍记下的叶序）"
        }
    }
}

/// 一枝 inode 子树下面的叶容器是不是全都到得了：每条条目的孩子都读得出、类是类型段说的那一类、叶容器解得开且至少一条记录
/// （记进叶序）。只有全都到得了的枝，内部节点头里的 key 区间才按子树覆盖区间判（I-1.1，`Walk::walk_inode_internal_node`）：
/// 少了一片，叶序里的最小、最大记录号就不是子树里到得了的那两个，判了会把读不出的孩子再算成一次区间不符。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EveryLeafReached {
    Yes,
    No,
}

impl EveryLeafReached {
    fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::Yes, Self::Yes) => Self::Yes,
            (Self::No, _) | (_, Self::No) => Self::No,
        }
    }
}

/// 别的版本共享一个 inode 树单元时接上的那一份：它下面按叶序排的叶容器，与那一枝的叶是不是全都到得了。
#[derive(Clone, Debug)]
struct InodeLeafOrderBelowAWalkedUnit {
    leaves_in_leaf_order: Vec<InodeLeafContainerInLeafOrder>,
    every_leaf_reached: EveryLeafReached,
}
/// 打包记录类型登记的记录宽：类型 2 是 140（D8（核心索引结构） 已定项 6），类型 4 是 88（实例表行）。
const fn registered_record_width(record_type: u16) -> Option<usize> {
    match record_type {
        PACKED_TYPE_INODE => Some(140),
        PACKED_TYPE_INSTANCE_TABLE => Some(88),
        // 墓碑记录 40、谱系记录 24（D18（块里携带什么信息） 已定项 24 第 2、5 条）。
        PACKED_TYPE_TOMBSTONE => Some(40),
        PACKED_TYPE_LINEAGE => Some(24),
        // 退役记录 34。
        PACKED_TYPE_RETIREMENT => Some(34),
        _ => None,
    }
}
const STATISTIC_ALLOCATED_BYTES: u16 = 1;
const STATISTIC_FREE_BYTES: u16 = 2;
/// defer 队列待释放（D5（快照 / 空间记账机制） 已定项 4 第 5 项的行号，带设备维，单位字节）：I-3.11（已分配减 defer 等于最新根走读） 读它。
const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;
/// 记账标签 3「不可回收」（D5（快照 / 空间记账机制） 已定项 4 第 3 项，逐盘）：I-3.20 (a) 读它。
const STATISTIC_UNRECLAIMABLE_BYTES: u16 = 3;
/// 记账标签 6「已承诺预留」（D5（快照 / 空间记账机制） 已定项 4 第 6 项，池级一行）：I-3.20 (b) 读它。
const STATISTIC_COMMITTED_RESERVATION_BYTES: u16 = 6;
/// 碎片度的 runs（D5（快照 / 空间记账机制） 已定项 4 第 10 项的行号，带设备维）：单元区空闲槽的连续段数，I-3.16 判它。
const STATISTIC_FRAGMENTATION_RUNS: u16 = 10;
/// 全空聚簇段数（D5（快照 / 空间记账机制） 已定项 4 第 11 项的行号，带设备维）：I-3.16 判它。
const STATISTIC_EMPTY_CLUSTER_SEGMENTS: u16 = 11;
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
/// 删除中的 inode 记录的 nlink（D8（核心索引结构） 已定项 16 第 1 条：意图那次发布把记录写成 nlink = 0）：I-1.13 认删除中要两侧那条记录都是它。
const NLINK_OF_AN_INODE_RECORD_BEING_DELETED: u32 = 0;
/// 码 3 单元头里写序的实例代号（D18（块里携带什么信息） 已定项 11 的类身份段表：写序 93 起，实例代号 4 字节在前）。
const PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET: usize = 93;
/// inode 记录里改动计数的位置（偏移 88，D8（核心索引结构） 已定项 6 的字段表）：I-9.26 比它、对象出生代与容器诞生代号。
const INODE_RECORD_CHANGE_COUNT_OFFSET: usize = 88;
/// 码 3 单元头里的诞生代号（D18（块里携带什么信息） 已定项 11 的类身份段表：偏移 73）。
const PACKED_UNIT_BIRTH_TXG_OFFSET: usize = 73;
/// 记账 key（统计量 2 + 8 + 设备 4 + 代 8，`KEY_SCHEMA_ACCOUNTING`）里代段的偏移：I-3.13 拿它比这一版的 txg。
const ACCOUNTING_KEY_GENERATION_OFFSET: usize = 2 + 8 + 4;
/// 树表条目里 `previous_snapshot_txg` 的偏移（树 ID 8 + 种类 2 + 2 + 2 + 根指针 86；之后 8 字节是出生 txg），I-9.25 判它。
const TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86;
const INODE_RECORD_BLOCKS_OFFSET: usize = 48;
/// `blocks` 的计量单位：512 字节一块，`blocks` = ⌈`size` ÷ 512⌉，是逻辑长度的块数、不表示分到的空间（D8（核心索引结构） 已定项 6，
/// C480（inode 记录的 blocks 怎么算全仓没有条款））。
const INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES: u64 = 512;
/// inode 记录截断字段前后两段恒 0 的预留（D8（核心索引结构） 已定项 6：偏移 108 填充 4、112 flags 8；判决
/// `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R1 把 120–135 赋义为截断字段之后剩下 136 起 4 字节），I-9.7 判它们。
const INODE_RECORD_RESERVED_BYTES_BEFORE_THE_TRUNCATION_FIELD: std::ops::Range<usize> = 108..120;
const INODE_RECORD_RESERVED_BYTES_AFTER_THE_TRUNCATION_FIELD: std::ops::Range<usize> = 136..140;

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
/// deadlist 树叶条目在不判违例的那几处次读里要几个字节（[`mapping_keys_named_by_the_deadlist_of_the_newest_root`] 这一类）：key 43、value 空
/// （D5（快照 / 空间记账机制） 已定项 12）。这几处只守「切到偏移 43 之前得有 43 个字节」；条目宽等不等于字段表宽由主走读按
/// I-1.10（码 2 条目宽等于字段表宽） 判（[`deadlist_leaf_entry_field_table_bytes`]）。
fn deadlist_entry_bytes() -> usize {
    KEY_SCHEMA_DEADLIST.width()
}
/// deadlist 树叶条目的字段表宽（I-1.10（码 2 条目宽等于字段表宽））：头的树 ID 8 + 纪元 8 + 中央映射 key 27 = 43，value 空
/// （D5（快照 / 空间记账机制） 已定项 12）。
/// 中央映射 key 从偏移 8 + 8 = 16 起（[`DEADLIST_ENTRY_MAPPING_KEY_OFFSET`]），到条目末尾。checker 按自己读的条款另写一份，不引核心层的宽
/// （D13（验证路线） 已定项 5）。
fn deadlist_leaf_entry_field_table_bytes() -> usize {
    DEADLIST_ENTRY_MAPPING_KEY_OFFSET + mapping_key_bytes()
}
/// deadlist 树内部节点条目的字段表宽：本树 key + 子指针 86（D8（核心索引结构） 已定项 11「内部节点的条目 = 本树 key + 子指针 86」）；
/// deadlist 的 key 就是整条叶条目（value 空），所以是 43 + 86 = 129。
fn deadlist_internal_entry_field_table_bytes() -> usize {
    deadlist_leaf_entry_field_table_bytes() + node_pointer_bytes()
}
/// logged_ops 树叶条目 44：key 10（种类 2 + 签发 txg 8）+ value 34。
fn logged_operations_entry_bytes() -> usize {
    usize::try_from(LOGGED_OPERATIONS_ENTRY_BYTES).expect("44")
}
/// 从树表条目读一棵树的根时，根头里的 key 区间怎么判（I-1.1，D18（块里携带什么信息） 已定项 2「区间取子树覆盖区间」）：inode 树与快照 inode 根
/// 的孩子是码 3 叶容器，层级 ≥ 1 的根区间按叶里的记录号判；墓碑树、谱系树、退役树的 key 是容器号，层级 1 的根贴紧首末条目、层级 ≥ 2 的
/// 按孩子头判；别的走到这一支的（`deleted_inodes` 这类层级 0 的根）贴紧首末条目。
fn key_range_reading_of_a_tree_walked_from_its_tree_table_entry(kind: u16) -> KeyRangeReading {
    match kind {
        TREE_KIND_INODE | TREE_KIND_SNAPSHOT_INODE_ROOT => {
            KeyRangeReading::SubtreeCoverageOfTheInodeTreeJudgedByTheCaller
        }
        TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE | TREE_KIND_RETIREMENT => {
            KeyRangeReading::SubtreeCoverageOfAContainerTreeAboveLevelOneJudgedByTheCaller
        }
        _ => KeyRangeReading::FirstAndLastEntries,
    }
}

/// 一棵树的条目该有多宽（各棵树的条目字段表）：走读按固定偏移切条目之前先比它（I-1.10（码 2 条目宽等于字段表宽））。
/// 条目宽是索引节点头里的一个**盘上字段**，`index_node_view` 只判了它 ≥ key 宽
/// （extent 24、inode 8、分配记录 10、记账 22），够不着「装得下整条记录」。
///
/// ⚠️ **中央映射树（种类 5）不在这张表里**：它的条目宽 D19（块指针的结构与宽度预算） 已定项 10 已定一律 55，
/// 但把种类 5 接进这张表、让 I-1.10（码 2 条目宽等于字段表宽） 也判它，还欠在 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树），
/// 所以今天 I-1.10 不判它。`key_schema_for_tree_kind` 罩的种类里只有它不在这张表里。
/// deadlist 树（种类 8）登在这里的是叶条目的宽；它的内部节点另按 [`deadlist_internal_entry_field_table_bytes`] 判。
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
        // logged_ops 树的条目 44 = key 10 + value 34。
        TREE_KIND_LOGGED_OPERATIONS => Some(logged_operations_entry_bytes()),
        // deadlist 树叶条目 43 = 头的树 ID 8 + 纪元 8 + 中央映射 key 27：实现拒宽于 43 的条目，checker 同判等于（判决
        // `research/prompts/m3-close-w13-code-r1-main-verification.md` 第二节 K5 那一行）。
        TREE_KIND_DEADLIST => Some(deadlist_leaf_entry_field_table_bytes()),
        _ => None,
    }
}

/// 读一个码 2 节点时内部节点的 key 区间怎么判（I-1.1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyRangeReading {
    /// 区间贴紧首末两条条目的 key：叶一律这样；树表与根兼叶的那几棵（`deleted_inodes`、logged_ops 根之外的层级 0 根）也这样写。
    FirstAndLastEntries,
    /// inode 树（D18（块里携带什么信息） 已定项 2）：层级 0 的根兼叶贴紧首末条目；层级 ≥ 1 的节点（根，与类型段 0 指着的内部节点）
    /// 区间是子树覆盖区间 [最左那片的最小记录号, 最右那片的最大记录号]，要等叶容器读回来才判得了，由走读那一方判
    /// （`Walk::walk_inode_internal_node`），读节点这一步只判分隔 key 严格递增；零条目的根（删空之后）只判 key 宽。
    SubtreeCoverageOfTheInodeTreeJudgedByTheCaller,
    /// 墓碑树、谱系树、退役树（key 是容器号）：层级 1 的节点（孩子是码 3 容器）区间就是首末两条条目的分隔 key，贴紧判；
    /// 层级 ≥ 2 的节点区间是子树覆盖区间 [第一个孩子头里的最小 key, 最后一个孩子头里的最大 key]，由走读那一方读回孩子时判
    /// （`Walk::walk_the_internal_node_below_a_container_tree_entry`），读节点这一步只判分隔 key 严格递增。
    SubtreeCoverageOfAContainerTreeAboveLevelOneJudgedByTheCaller,
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
/// 第 0 条位置条目问一遍（第 0 条那一份对不上、读回的是被先走过的版本读过的第 1 条那一份时，两个键会把「先走过」认成「走不下去」）。
#[derive(Debug)]
enum IndexNodeReading {
    /// 这一遍第一次走到，头、出生身份、树 ID 与 key 都判过：节点连同读回那一份的落点（这个单元在走读里的身份，往下走时记成父单元）。
    WalkedFirstByThisVersion {
        view: crate::IndexNodeView,
        placement_it_was_read_from: (u32, u64),
    },
    /// 读回那一份的落点别的版本先走过：它下面的落点已经并进这一版（[`Walk::merge_placements_below_a_unit_walked_by_an_earlier_version`]），不重走。
    /// `header_the_earlier_walk_could_use` 是这一次读回的那一份解开的节点：头里是码 2、先走那一遍头用得了时才有
    /// （[`Walk::view_of_a_node_an_earlier_version_walked`]）；调用方拿它照判这一版的父条目定的层级、key 区间与子树覆盖区间（I-1.1，
    /// `.claude/kb/checks-owed.md` C896），不往下走。
    WalkedByAnEarlierVersion {
        placement_it_was_read_from: (u32, u64),
        header_the_earlier_walk_could_use: Option<crate::IndexNodeView>,
    },
    /// 指针全零，或两份都读不出、头用不了、条目区解不开（后几种走读失败已记）：这一支不往下走。
    NotWalkable,
}

/// 多层码 2 树一个节点的条目宽在走读里怎么对待。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EntryWidthInTheWalk {
    /// 按 I-1.10（码 2 条目宽等于字段表宽） 判：记账树叶 34、内部节点 108；deadlist 树叶 43、内部节点 129；logged_ops 树叶 44、内部节点 96。
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
    /// extent、inode、分配记录、记账四棵：条目宽 0 而条目数非 0 报 I-1.10——宽 0 不是任何登记宽。
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
    /// 节点按谁认位置：中央映射树自己的节点是自举豁免（指针自己的位置条目是权威），记账树与 deadlist 树的节点按中央映射。
    position_authority: PositionAuthority,
}

/// 走一个节点连同它下面的结果：交回它头里的 key 区间（父节点判两条不等式与覆盖区间要它）与这棵子树走全了没有。
#[derive(Clone, Debug, PartialEq, Eq)]
enum SubtreeInTheWalk {
    Walked {
        smallest_key: Vec<u8>,
        largest_key: Vec<u8>,
        is_complete: bool,
    },
    /// 这一遍之前别的根已经走过它（`visited_units`）：不往下走。这一版的父条目定的那几格照判（`.claude/kb/checks-owed.md` C896）：
    /// 层级在 [`Walk::walk_code_two_subtree`] 里判，头里的 key 区间交回父节点判两条不等式与覆盖区间。头用不了、key 宽不是这棵树的、
    /// 层级不是父层级减一时 `key_range_in_its_header` 是 `None`（同首次走读那几支不往下判）。
    AlreadyWalkedFromAnotherRoot {
        key_range_in_its_header: Option<(Vec<u8>, Vec<u8>)>,
    },
    /// 读不出、头用不了、层级不对或条目宽不对：走读失败已记，这一支不往下走。
    NotWalkable,
}

impl SubtreeInTheWalk {
    /// 父节点判两条不等式与覆盖区间要的那一段 (头里的最小 key, 最大 key)：这一遍走的、与别的根先走过而这一版的层级对得上的孩子都有。
    fn key_range_in_the_header(&self) -> Option<(&[u8], &[u8])> {
        match self {
            Self::Walked {
                smallest_key,
                largest_key,
                ..
            }
            | Self::AlreadyWalkedFromAnotherRoot {
                key_range_in_its_header: Some((smallest_key, largest_key)),
            } => Some((smallest_key, largest_key)),
            Self::AlreadyWalkedFromAnotherRoot {
                key_range_in_its_header: None,
            }
            | Self::NotWalkable => None,
        }
    }
}

/// 走读跟着的一条引用要被指单元是什么：类标签与按这一条引用定的那几格（树、extent key、片号）。别的版本先走过这个单元、这一版又引用它时，
/// [`Walk::judge_the_cells_the_reference_decides_on_a_unit_walked_earlier`] 照它判（`records/2026-10-24-singlefs-checker-审计复核.md` 的 C01）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReferenceTheWalkFollowsToAUnit {
    /// 码 2 节点：头里的树 ID 是 `tree`。
    IndexNodeOfTree { tree: u64 },
    /// 码 3 容器（inode 叶容器，墓碑 / 谱系 / 退役树的容器）：偏移 43 的出生树是 `tree`。
    PackedContainerOfTree { tree: u64 },
    /// 实例表链上第 `page_index` 片（码 3 打包记录类型 4，I-1.3 豁免）。
    InstanceTablePage { page_index: u64 },
    /// extent 树 `tree` 里 key (`inode`, `offset`) 指着的数据单元（码 1）。
    DataUnitOfExtentKey { tree: u64, inode: u64, offset: u64 },
}

impl ReferenceTheWalkFollowsToAUnit {
    /// 这条引用要的类标签（D18（块里携带什么信息） 已定项 11 的登记码）。
    const fn unit_class(self) -> u8 {
        match self {
            Self::DataUnitOfExtentKey { .. } => 1,
            Self::IndexNodeOfTree { .. } => 2,
            Self::PackedContainerOfTree { .. } | Self::InstanceTablePage { .. } => 3,
        }
    }
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
    /// (设备, 起点槽)：整个走读（各版合起来）走过的单元，再遇到就不再往下走——不同版之间共享没改过的单元是合法的 COW，
    /// 走过一遍就够。同一版里再遇到同一个单元不是共享，那一格由 `placements_referenced_in_this_version` 判。
    visited_units: BTreeSet<(u32, u64)>,
    /// 走过的单元里（与 `visited_units` 同一个键）头用得了的那些：码 1 / 码 2 是 [`Walk::judge_unit_header`] 交回能用，码 3 是
    /// [`Walk::judge_packed_container`] 交回了记录。别的版本再引用它时，这一条引用自己定的那几格（出生身份、树 ID、五元组、片号、
    /// 指针出生树）只在这里有它时照判（[`Walk::judge_the_cells_the_reference_decides_on_a_unit_walked_earlier`]）：头用不了时那几段不可信，
    /// 先走那一遍也没判它们（码 3 容器先走那一遍在判头之前就判了树 ID 与出生身份，命中时不跟着判，头那一格已经红了）。
    walked_units_whose_header_the_walk_could_use: BTreeSet<(u32, u64)>,
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
    /// 不重读盘（实审 B1 留下的缺口）。
    placements_referenced_below_a_walked_unit:
        BTreeMap<(u32, u64), Vec<PlacementReferencedBelowAWalkedUnit>>,
    /// 走过的 inode 树单元 (设备, 槽)（叶容器，或类型段 0 指着的内部节点）→ 它下面按叶序的叶容器：别的版本再引用它时，
    /// I-9.4（容器号不超最小 key） 的叶序照这张表接上，不重读盘。
    inode_leaf_order_below_a_walked_unit: BTreeMap<(u32, u64), InodeLeafOrderBelowAWalkedUnit>,
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
    /// 走过的树表里第一条记账树条目的根指针（不全零的）出生 txg：I-7.18、I-7.19 认「建账的发布」读它（[`AccountOfTheNewestRoot`]）。
    /// 只取最新根那一遍（`check_pool_image` 走完最新根就拷走），GPU 走读内核在同一处记同一个数。
    accounting_tree_root_birth_txg: Option<u64>,
    /// 走全了的记账树里 inode 号水位那一统计量（不带设备维）的每一行 (代段, 值)，按走到的次序（key 升序）：
    /// 每次发布写的那一行与台阶行（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）。只取最新根那一遍（`check_pool_image` 走完最新根就拷走）。
    inode_number_watermark_rows: Vec<(u64, u64)>,
    /// 最新根指着的实例表里的行：别的根按它判有效（回退之后被抛弃时间线的根不进 I-3.1 的并集），
    /// I-1.8（归并后版本全序） 的已发布谓词也读它。
    instance_table_rows: Vec<InstanceTableRow>,
    /// 主走读沿最新根的实例表链走没走到末片（链指针记录说「没有下一片」的那一片）：走到了，`instance_table_rows` 就是整张表的行，
    /// 回退候选集按它滤掉了被崩溃恢复抛弃的根；没走到（链上一片读不出、解不开、链指针记录不合法、链成了环）就滤不全。
    /// I-9.29（同号只许一个出生代） 判不了的门读它，与候选集的滤法用同一遍走读
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
    /// （[`Walk::read_a_unit_to_walk_and_the_placement_it_was_read_from`]）：
    /// 第 0 条落点上是别的单元或坏副本时，I-3.5 取的是读回那一份的头里的诞生代号。
    unit_of_each_placement: BTreeMap<(u32, u64), (u32, u64)>,
    /// 第一次记下时是按中央映射 key 折的落点（豁免三类之外的码 1 / 2 / 3，[`PositionAuthority::TheCentralMapping`]）：I-3.5 比相邻两版时
    /// 这些落点按 `unit_of_each_placement` 折成的单元认（搬前、搬后两个落点是同一个单元
    /// 第三节 G3），不在这里的落点按它自己认、不折（C748（最终折叠表漏判引用区间的落点）：读回第 1 条那一份时折叠表会改，豁免三类的落点按它比
    /// 会把后一版新出现的落点折进前一版引用过的单元里、不判）。只在第一次记下时收，之后不改；GPU 走读内核记在落点表一行第 7 字的高位。
    placements_first_noted_through_the_central_mapping: BTreeSet<(u32, u64)>,
    /// 正在走的这一版记下的落点里，记的那一刻走读正在这一版的一棵用户可见树（extent、inode、`deleted_inodes`）里往下走的那些 → 那棵树
    /// （I-3.5 (b) 认「落在 R 树表里某棵用户可见树的子树里」按 R 自己这一张）。只记这一版：同一个落点在别的版本里可能只从快照那两棵树走得到、
    /// 或落在别的树里，跨版本共用一张表时先走那一版的标签（或没有标签）会顶替后走那一版的。
    /// 别的版本先走过的单元不重走，它下面并进这一版的落点
    /// 按并进来那一刻这一版在哪棵用户可见树里记（[`Walk::merge_placements_below_a_unit_walked_by_an_earlier_version`]）；
    /// 别的版本先走过的树表照 `user_visible_tree_of_each_placement_below_a_walked_tree_table` 接上。每走一版之前清空，一版走完交给 I-3.5。
    user_visible_tree_of_each_placement_in_this_version:
        BTreeMap<(u32, u64), UserVisibleTreeOfTheWalk>,
    /// 走过的树表单元 (设备, 槽) → 第一个走到它的那一版走完它的条目时，这一版记下的落点标签：别的版本共享这片树表时条目不重走、
    /// 树表下面的落点并进来的那一刻不在任何一棵树里，标签照这张接上（共享树表的两版树表条目逐字节相同，哪个落点在哪棵用户可见树里也相同）。
    user_visible_tree_of_each_placement_below_a_walked_tree_table:
        BTreeMap<(u32, u64), BTreeMap<(u32, u64), UserVisibleTreeOfTheWalk>>,
    /// 走读此刻在哪棵用户可见树里往下走；不在任何一棵里是 `None`。
    user_visible_tree_being_walked: Option<UserVisibleTreeOfTheWalk>,
    /// 正在走的这一版树表里每棵根指针非全零的用户可见树，带根指针的出生 txg（I-3.5 (b) 拿它比 R⁻.txg）。
    user_visible_tree_roots_of_this_version: Vec<UserVisibleTreeRootOfAVersion>,
    /// 这一版树表里第一条「走到没有条款的形状」的条目的种类（[`tree_table_entry_is_a_shape_without_a_clause`]）：这一版走完记进 I-3.5 的输入。
    kind_of_a_tree_table_entry_without_a_clause_in_this_version: Option<u16>,
    /// 走过的树表单元 (设备, 槽) → 它每一条条目里 I-9.25 与 I-3.5 读的几样：别的版本共享这片树表时照这份判，不重读盘。
    tree_table_entries_of_a_walked_tree_table: BTreeMap<(u32, u64), Vec<TreeTableEntryOfTheWalk>>,
    /// 走过的实例表各片（按读回那一份的落点）→ 头里的诞生代号：I-3.5 要它。不进 `birth_txg_of_walked_units`——
    /// 那张表 I-4.9 也读，而实例表单元按 I-1.2 那一行的例外不进 I-4.9。
    birth_txg_of_walked_instance_table_pages: BTreeMap<(u32, u64), u64>,
    /// 已经走完的每一版（根环里的候选根与由记录施加出来的那几版）：I-3.5 在全部走完之后按那一刻的折叠表折成单元、逐对比。
    versions_walked_for_the_appearance_rule: Vec<VersionPlacementsForTheAppearanceRule>,
    /// 正在走的这一版的中央映射（走它的树表条目之前只读读出，[`CentralMappingLocationsOfAVersion`]）：豁免三类之外的指针按它认位置
    /// 每走一版之前清空。
    central_mapping_of_this_version: CentralMappingLocationsOfAVersion,
    /// 中央映射 key → 走读认它作哪个单元（第一次跟着指针记下它时那条指针认的第 0 条位置，与 `unit_of_each_placement` 的值同一种）：
    /// 搬迁只改映射一条条目、单元逐字节不变（D19（块指针的结构与宽度预算） 已定项 8），搬前那一版在源落点、搬后那一版在目的落点，
    /// 两个落点折成同一个单元，I-3.5 不把搬过的单元当成后一版新出现的。
    unit_of_each_central_mapping_key: BTreeMap<CentralMappingKey, (u32, u64)>,
    /// 走读此刻走的是这一版自己的树，还是快照那两棵树（与活头、与别的快照共享单元）。
    tree_sharing_in_the_walk: TreeSharingInTheWalk,
    /// 正在走的这一版树表里带销毁中记号的码 14 与头 ID 配它的码 15 的树 ID：这两棵树从写意图那一刻起不再钉住块（D5（快照 / 空间记账机制）
    /// 已定项 20 的例外句），它们的单元可以已被删快照的批放掉、复用，走读不往下走（[`snapshot_trees_of_snapshots_being_destroyed`]）。
    snapshot_trees_being_destroyed_in_this_version: BTreeSet<u64>,
}

/// 走读此刻在走哪一类树。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeSharingInTheWalk {
    /// 这一版自己的树（树表里码 14、码 15 之外的条目指着的）：同一版里一个落点被引用两次判 I-5.1，走到的单元进 I-3.5 那一版的引用集合。
    OwnTreesOfTheVersion,
    /// 活快照的两棵树（码 14 照 inode 树走、码 15 照 extent 树走）：它们与活头、与别的活快照共享没改过的单元是合法的 COW，
    /// 同一版里再遇到同一个单元不是第二次引用，已经走过的单元不重走、不并进这一版；只走快照独有的单元、判它们的结构。
    /// 走到的落点进引用并集（I-3.1、I-5.1 的跨引用重叠），不进 I-3.5 那一版的引用集合（快照子树里的单元不是用户可见树里新出现的，
    /// 判决 S8），也不进活头那几样累计（inode 号最大值、对象出生代、数据单元的对象）——那几样只判活头的树。
    SnapshotTreesSharingUnitsWithTheHead,
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
    /// 条目偏移 108 的诞生 txg：码 14 的就是那个快照的 S.txg（I-9.25 按它排前驱链）。
    birth_txg: u64,
    /// 种类是用户可见树、根指针非全零时是那棵树的根；别的条目 `None`。
    user_visible_root: Option<UserVisibleTreeRootOfAVersion>,
    /// 根指针整条全零：[`tree_table_entry_is_a_shape_without_a_clause`] 读它。
    root_pointer_is_all_zero: bool,
    /// 码 14 条目 flags 位 0 是不是「销毁中」：I-9.25 的前驱链跳过它。
    /// 别的种类恒 false。
    is_a_snapshot_being_destroyed: bool,
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
        birth_txg: read_u64(entry, TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET),
        user_visible_root,
        root_pointer_is_all_zero: root_pointer.all_zero,
        is_a_snapshot_being_destroyed: kind == TREE_KIND_SNAPSHOT_INODE_ROOT
            && read_u16(entry, TREE_TABLE_ENTRY_FLAGS_OFFSET)
                & TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED
                != 0,
    })
}

/// 树表的一条条目是不是 I-3.5 条款说的「走到没有条款的形状」（`crates/singlefs-checker/invariants/05-空间记账.md` I-3.5 那一行）：
/// 主走读走不下去，这一版的引用集合数不全。种类没登记的条目一律算（根指针全零的也算：它占的是另一种树的位置，条款没说它是什么）；
/// livelist、稀疏旁表登记了、条目格式没有条款：根指针全零是第一版的合法形状，非全零才算（它们真有了根节点，走读不往下走）。
/// 只读快照的两种条目没有自己的 key 形态，走读照活头那两种树往下走，不算。
fn tree_table_entry_is_a_shape_without_a_clause(kind: u16, root_pointer_is_all_zero: bool) -> bool {
    match kind {
        TREE_KIND_SNAPSHOT_INODE_ROOT | TREE_KIND_SNAPSHOT_EXTENT_ROOT => false,
        TREE_KIND_LIVELIST | TREE_KIND_SPARSE_SIDE_TABLE => !root_pointer_is_all_zero,
        other_kind => key_schema_for_tree_kind(other_kind).is_none(),
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

/// 一版中央映射叶里的条目：key（27 字节）→ 两条位置条目。同一个 key 在叶里出现两次的不收（哪一条作数没有条款：叶里 key 严格递增归 I-1.1，
/// I-1.13 那一遍报 `CentralMappingTreeNamesOneKeyTwice`），走读对它照提示走。映射树读不全时是空表：整版照提示走，读不全由主走读的映射那一遍
/// 记走读失败、判 I-2.1 / I-1.1。
#[derive(Clone, Debug, Default)]
struct CentralMappingLocationsOfAVersion {
    locations_of_each_key: BTreeMap<CentralMappingKey, [PointerLocation; 2]>,
}

impl CentralMappingLocationsOfAVersion {
    /// 只读不判：从一版的映射根指针读出叶里全部条目。根指针全零（树表 0 条的一版）或读不全时交回空表。
    fn read_without_judging(
        reader: &dyn ImageReader,
        mapping_root_pointer: &PointerView,
        cache: &mut IndexNodeCache,
    ) -> Self {
        let mut leaf_entries = Vec::new();
        if mapping_root_pointer.all_zero
            || central_mapping_leaf_entries_without_judging(
                reader,
                mapping_root_pointer,
                None,
                cache,
                &mut leaf_entries,
            )
            .is_none()
        {
            return Self::default();
        }
        let mut locations_of_each_key: BTreeMap<CentralMappingKey, [PointerLocation; 2]> =
            BTreeMap::new();
        let mut keys_named_twice: BTreeSet<CentralMappingKey> = BTreeSet::new();
        for entry in &leaf_entries {
            let key = CentralMappingKey(entry[..mapping_key_bytes()].to_vec());
            if locations_of_each_key
                .insert(key.clone(), locations_of_a_mapping_entry(entry).locations)
                .is_some()
            {
                keys_named_twice.insert(key);
            }
        }
        for key in &keys_named_twice {
            locations_of_each_key.remove(key);
        }
        Self {
            locations_of_each_key,
        }
    }

    /// 一条指针按 `authority` 认的位置：豁免三类、全零指针、映射里查不到的，原样交回；查得到的换成映射条目的两条位置条目
    /// （[`PointerView::located_by_the_central_mapping`]，指针自己的留作提示）。
    fn locate(&self, pointer: &PointerView, authority: PositionAuthority) -> PointerView {
        match authority {
            PositionAuthority::ThePointerItself => *pointer,
            PositionAuthority::TheCentralMapping(class) => {
                if pointer.all_zero {
                    return *pointer;
                }
                match self
                    .locations_of_each_key
                    .get(&central_mapping_key_of_a_pointer(pointer, class))
                {
                    // 只换位置、不换内容：同一把 key 的单元逐字节不变（搬迁原样拷、头不盖戳，D19（块指针的结构与宽度预算） 已定项 8「被搬单元逐字节不变」；
                    // 判决 G3 的 N6），过期的提示与映射条目在同一序号上带的整单元校验和相同、只差落点。校验和不同的那一条指针
                    // 与它点名的单元对不上，不是过期的提示：照它自己的位置条目读、判。
                    Some(mapping_locations)
                        if mapping_locations.iter().zip(&pointer.location_hints).all(
                            |(mapping_location, hint)| mapping_location.checksum == hint.checksum,
                        ) =>
                    {
                        pointer.located_by_the_central_mapping(*mapping_locations)
                    }
                    Some(_) | None => *pointer,
                }
            }
        }
    }
}

impl<'reader> Walk<'reader> {
    /// 走读要跟随的一条指针，按 `authority` 认它指的单元落在哪（[`CentralMappingLocationsOfAVersion::locate`]，查的是这一版的中央映射）。
    fn located(&self, pointer: &PointerView, authority: PositionAuthority) -> PointerView {
        self.central_mapping_of_this_version
            .locate(pointer, authority)
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

    /// 由记录施加出来、根槽从没落盘的那一版（[`versions_applied_only_by_records`] 认出来的）：它没有根记录，
    /// 树表指针、映射根指针、实例表指针与「树表 0 条那一版的分配记录树的根」都在那条记录的新根段里（D23（journal 的角色与格式）
    /// 已定项 15：施加一条记录 = 把所选根的七个字段换成记录里的），从那里走下去，判定与根环里的候选根同一套。
    /// 新根段里树表指针整条全零时与根环里的根同一读法（ Q1「甲 两类都判损坏」，判决
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
}

/// 一张树表里不再钉住块的那几棵快照树的树 ID：带销毁中记号的码 14（flags 位 0
/// 第三节 S4），连同头 ID 指着它们的码 15。条目比字段表短的不解（那一格由调用方记成数不全）。
fn snapshot_trees_of_snapshots_being_destroyed(tree_table_entries: &[Vec<u8>]) -> BTreeSet<u64> {
    let entries_read_by_the_field_table = tree_table_entries
        .iter()
        .filter(|entry| entry.len() >= tree_table_entry_bytes());
    let snapshot_inode_roots_being_destroyed: BTreeSet<u64> = entries_read_by_the_field_table
        .clone()
        .filter(|entry| {
            read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_SNAPSHOT_INODE_ROOT
                && read_u16(entry, TREE_TABLE_ENTRY_FLAGS_OFFSET)
                    & TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED
                    != 0
        })
        .map(|entry| read_u64(entry, 0))
        .collect();
    let snapshot_extent_roots_of_them = entries_read_by_the_field_table
        .filter(|entry| {
            read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_SNAPSHOT_EXTENT_ROOT
                && snapshot_inode_roots_being_destroyed
                    .contains(&read_u64(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET))
        })
        .map(|entry| read_u64(entry, 0));
    snapshot_inode_roots_being_destroyed
        .iter()
        .copied()
        .chain(snapshot_extent_roots_of_them)
        .collect()
}

/// mkfs 把第 0 代种进根环（D22（单元原子性怎么合成） 已定项 8）：I-7.3（环健康性） 的例外那一支按这个代号判。
const GENESIS_CHECKPOINT_TXG: u64 = 0;

/// I-7.3（环健康性）：S（根槽里自证校验和通过的记录集合）中除代号最大者外，至少还存在一条更早代号的记录，
/// 不存在即判红——某次提交把上一代直接覆盖了，轮换逻辑已失效。**例外只有一个**：S 全部是第 0 代时判绿
/// （`crates/singlefs-checker/invariants/02-根环.md` I-7.3 那一行的例外原样照搬；「崩溃后回退到这个态的镜像同样判绿」落在同一支上——
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

/// txg `checkpoint_txg` 按落点式该住的根环槽（I-7.17：区域 txg mod R、槽 (txg div R) mod S；第 0 代根种在每个区域的槽 0，这里取区域 0 槽 0）。
fn root_ring_slot_the_txg_names(checkpoint_txg: u64, geometry: &PoolGeometry) -> (u64, u64) {
    if checkpoint_txg == 0 {
        return (0, 0);
    }
    let region = checkpoint_txg
        .checked_rem(geometry.regions)
        .expect("系统配置择得到 ⇒ R ≥ 1（geometry_of 拒 R = 0）");
    let slot = (checkpoint_txg / geometry.regions)
        .checked_rem(geometry.slots_per_region)
        .expect("系统配置择得到 ⇒ S ≥ 1");
    (region, slot)
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

    /// checker 自己那份种类码与格式常量里的树 ID 值域不相交（D8（核心索引结构） 已定项 9 射程、已定项 11）：
    /// 每个种类码都小于 65536（树的种类那 2 字节取不到的最小值），
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

    /// 哪些树表条目算「走到没有条款的形状」：种类没登记的不管根指针、livelist 与稀疏旁表只在根指针非全零时、
    /// 走读走得下去的那几种（只读快照两种连同有 key 形态的）都不算。
    #[test]
    fn only_unregistered_kinds_and_rooted_livelist_or_sparse_side_table_entries_are_shapes_without_a_clause(
    ) {
        for root_pointer_is_all_zero in [true, false] {
            for kind in [0, TREE_KIND_NOT_REGISTERED, u16::MAX] {
                assert!(
                    tree_table_entry_is_a_shape_without_a_clause(kind, root_pointer_is_all_zero),
                    "种类 {kind} 没登记，根指针全零 {root_pointer_is_all_zero}"
                );
            }
            for kind in [
                TREE_KIND_EXTENT,
                TREE_KIND_INODE,
                TREE_KIND_ALLOCATION,
                TREE_KIND_ACCOUNTING,
                TREE_KIND_DEADLIST,
                TREE_KIND_DELETED_INODES,
                TREE_KIND_TOMBSTONE,
                TREE_KIND_LINEAGE,
                TREE_KIND_LOGGED_OPERATIONS,
                TREE_KIND_RETIREMENT,
                TREE_KIND_SNAPSHOT_INODE_ROOT,
                TREE_KIND_SNAPSHOT_EXTENT_ROOT,
            ] {
                assert!(
                    !tree_table_entry_is_a_shape_without_a_clause(kind, root_pointer_is_all_zero),
                    "种类 {kind} 走读走得下去，根指针全零 {root_pointer_is_all_zero}"
                );
            }
        }
        for kind in [TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE] {
            assert!(
                !tree_table_entry_is_a_shape_without_a_clause(kind, true),
                "种类 {kind} 根指针全零是第一版的合法形状"
            );
            assert!(
                tree_table_entry_is_a_shape_without_a_clause(kind, false),
                "种类 {kind} 真有了根节点：走读不往下走，引用没收全"
            );
        }
    }
}
