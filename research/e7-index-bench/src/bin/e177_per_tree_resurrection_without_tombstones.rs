//! E177（逐树不写墓碑的复活数）装置：跑前登记 `research/prompts/e177-preregistration.md`（第一至十一节、修订 1、执行员修订 2）。
//!
//! 独立手写模型，不与 `crates/` 共用代码、不依赖 `crates/`：写路径照登记第三节 S2–S16、修订 1 的 W1–W8 与执行员修订 2 的
//! 核对 2–4（M1–M11）逐条手写；读侧三种（R-U、R-S、R-G）、三条臂（N、B、P）、三个几何（G-环、G-宽、G-紧）。
//! 格式常量写成本地常量，旁边写来源行；不引 `crates/` 的任何东西。登记没说到、非取一个读法不可的几格照执行员修订 2 的 C1–C16。
//!
//! 用法（在 research/ 下）：
//!   cargo run --release --bin e177-per-tree-resurrection-without-tombstones -- segment-one
//! 线程数取环境变量 E177_THREADS（默认 8，按段切片）；产物按段的次序输出，与线程数无关。
//! 产物是 `E7RESULT` 行，收尾 `E7RESULT name=done emitted=N`（N 把 config 行与 done 行都算进去）。

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

// ---------------------------------------------------------------------------------------------
// 格式常量（本地常量，值抄自 `crates/` 的登记位，行号是 2026-10-08 工作区现取的）
// ---------------------------------------------------------------------------------------------

/// 槽 16 KiB（`crates/singlefs-format/src/lib.rs:14`）。
const SLOT_BYTES: u64 = 16_384;
/// 每块盘 4 GiB（`crates/singlefs-format/src/lib.rs:308` 的测试镜像默认字节数）。
const DEVICE_BYTES: u64 = 4_294_967_296;
/// 一块盘的绝对槽数：262 144。
const DEVICE_SLOTS: u64 = DEVICE_BYTES / SLOT_BYTES;
/// 单元区起点（`crates/singlefs-format/src/lib.rs:291`）。
const UNIT_AREA_START_SLOT: u64 = 50_176;
/// G-环 的单元区槽数（`allocator.rs:342-348`）：211 968。
const RING_GEOMETRY_UNIT_AREA_SLOTS: u64 = DEVICE_SLOTS - UNIT_AREA_START_SLOT;
/// 聚簇段 64 槽（`crates/singlefs-format/src/lib.rs:294`）。
const CLUSTER_SEGMENT_SLOTS: u64 = 64;
/// 根环区域数（`crates/singlefs-format/src/lib.rs:261`）。
const ROOT_RING_REGIONS: u64 = 3;
/// mkfs 时每区槽数（`crates/singlefs-format/src/lib.rs:279`）。
const ROOT_RING_SLOTS_PER_REGION: u64 = 8;
/// A15 的环长：独立写成 24，不从上面两个常量算（修订 1 第 12.8 节 V12 那一行）。
const RING_LENGTH_IN_ROOTS_FOR_THE_REUSE_ASSERTION: u64 = 24;
/// Q14 的环窗：t − s ≤ 23 算环窗内（修订 1 第 12.5 节）。
const RING_WINDOW_IN_PUBLISHES: u64 = 23;
/// G-环 的占用上限：单元区的 10%，21 196 槽（修订 1 第 12.3 节）。
const OCCUPANCY_CAP_SLOTS: u64 = 21_196;
/// 池里两块盘（登记 5.1）。
const DEVICE_COUNT: u8 = 2;
/// 码 3 容器 32 KiB（`crates/singlefs-format/src/lib.rs:20`）与记录区起点 136（`:44`）。
const PACKED_CONTAINER_BYTES: u64 = 32_768;
const PACKED_RECORDS_OFFSET: u64 = 136;
/// inode 记录 140 字节（`crates/singlefs-format/src/lib.rs:104`）、墓碑记录 40 字节（`:113`）。
const INODE_RECORD_BYTES: u64 = 140;
const TOMBSTONE_RECORD_BYTES: u64 = 40;
/// 一片 inode 叶容器装 233 条（`crates/singlefs-format/src/lib.rs:110`）。
const INODE_LEAF_RECORDS: u64 = (PACKED_CONTAINER_BYTES - PACKED_RECORDS_OFFSET) / INODE_RECORD_BYTES;
/// 一片墓碑容器装 815 条（`crates/singlefs-format/src/lib.rs:116`；登记 K1）。
const TOMBSTONE_CONTAINER_RECORDS: u64 =
    (PACKED_CONTAINER_BYTES - PACKED_RECORDS_OFFSET) / TOMBSTONE_RECORD_BYTES;
/// 码 2 节点 16 KiB；头 = 86 + 2 × key 宽 + 预留 29（`crates/singlefs-core/src/unit.rs:129-152`）。
const NODE_BYTES: u64 = 16_384;
const INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE: u64 = 86;
const INDEX_NODE_RESERVED_BYTES: u64 = 29;
/// 子指针 86 字节（`crates/singlefs-format/src/lib.rs:89`）。
const NODE_POINTER_BYTES: u64 = 86;
/// 记账 key 22、条目 34（`crates/singlefs-format/src/lib.rs:143`、`:145`）；映射 key 27、条目 55（`:92`、`:95`）。
const ACCOUNTING_KEY_BYTES: u64 = 22;
const ACCOUNTING_ENTRY_BYTES: u64 = 34;
const MAPPING_KEY_BYTES: u64 = 27;
const MAPPING_ENTRY_BYTES: u64 = 55;
/// inode 树根条目 120 字节、key 8 字节（`crates/singlefs-format/src/lib.rs:107`）：一个根装 135 片叶容器。
const INODE_INTERNAL_ENTRY_BYTES: u64 = 120;
const INODE_KEY_BYTES: u64 = 8;
/// extent 树下段一片叶 144 个单元（`crates/singlefs-format/src/lib.rs:171`），内部扇出 147（`:168`）。
const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;
const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;
/// 分配记录树叶罩 812 槽（`crates/singlefs-format/src/lib.rs:155`），内部扇出 169（`:161`）。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
/// 产品批大小 = 下段一片叶的单元数（`crates/singlefs-core/src/unlink.rs` 的 `DeletionSwitches::PRODUCT`）。
const PRODUCT_UNLINK_BATCH_DATA_UNITS: u64 = EXTENT_TREE_LOWER_LEAF_DATA_UNITS;
/// 记账行：池级 3 行（`transaction.rs:4816`）、每盘 6 行（`transaction.rs:4856`）。
const ACCOUNTING_POOL_WIDE_ROWS: u64 = 3;
const ACCOUNTING_PER_DEVICE_ROWS: u64 = 6;
/// 不带设备维的统计量的设备段（模型自取，执行员修订 2 C6）。
const STATISTIC_WITHOUT_DEVICE: u64 = 0xFFFF_FFFF;
/// 新池新建文件写出的树表条目数（`crates/singlefs-format/src/lib.rs:188`；登记 K5）。
const TREE_TABLE_ENTRIES: u64 = 11;
/// 十二棵树的树 ID（`crates/singlefs-format/src/lib.rs:191-204`）。
const TREE_IDENTIFIER_EXTENT: u64 = 65_536;
const TREE_IDENTIFIER_INODE: u64 = 65_537;
const TREE_IDENTIFIER_ALLOCATION_RECORDS: u64 = 65_538;
const TREE_IDENTIFIER_ACCOUNTING: u64 = 65_539;
const TREE_IDENTIFIER_DELETED_INODES: u64 = 65_544;
const TREE_IDENTIFIER_TOMBSTONE: u64 = 65_545;
/// 单元类标签（D18 已定项 11 的登记表）：码 1 数据、码 2 索引节点、码 3 打包记录单元。
const UNIT_CLASS_DATA: u64 = 1;
const UNIT_CLASS_INDEX_NODE: u64 = 2;
const UNIT_CLASS_PACKED: u64 = 3;
/// 打包记录类型：1 墓碑、2 inode、4 实例表（`crates/singlefs-core/src/unit.rs:21-25`）。
const PACKED_TYPE_TOMBSTONE: u64 = 1;
const PACKED_TYPE_INODE: u64 = 2;
/// 第一个文件恒是 inode 1；新池新建文件的 txg 3（`crates/singlefs-format/src/lib.rs:301`）。
const FIRST_FILE_INODE: u64 = 1;
const NEW_POOL_FILE_CREATION_TXG: u64 = 3;
/// mkfs 那个进程取号 1，可写挂载取号 2（执行员修订 2 C5）。
const MAKE_FILESYSTEM_PROCESS_INSTANCE: u64 = 1;
const MOUNT_INSTANCE: u64 = 2;
/// mkfs 写在单元区里的两个单元（`crates/singlefs-core/src/make_filesystem.rs:54-55`）。
const INSTANCE_TABLE_GENESIS_SLOT: u64 = UNIT_AREA_START_SLOT;
const TREE_TABLE_GENESIS_SLOT: u64 = UNIT_AREA_START_SLOT + 2;
/// 整对象墓碑的区间止（D18 已定项 24 第 2 条）。
const WHOLE_OBJECT_RANGE_END: u64 = u64::MAX;
/// 阳性对照植入的单元用的出生树：哪棵真树的 ID 都不是，保证「身份全新」（登记 5.5 PC1）。
const PLANTED_BIRTH_TREE: u64 = 0xE177;
/// 阳性对照植入的单元在单元表之外，用这个号指它。
const PLANTED_UNIT: usize = usize::MAX;

/// 一个码 2 节点装得下几条定宽条目：(16384 − 头) ÷ 条目宽。
const fn index_node_entry_capacity(key_width: u64, entry_width: u64) -> u64 {
    (NODE_BYTES - (INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE + 2 * key_width + INDEX_NODE_RESERVED_BYTES))
        / entry_width
}

/// 一个 inode 树根（与墓碑树根同形）装得下几片叶容器：135。
const ROOT_NODE_CONTAINER_CAPACITY: u64 =
    index_node_entry_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES);

fn length_as_u64(length: usize) -> u64 {
    u64::try_from(length).expect("长度装得进 u64")
}

fn u64_as_index(value: u64) -> usize {
    usize::try_from(value).expect("下标装得进 usize")
}

// ---------------------------------------------------------------------------------------------
// 树、key 的名字空间、读法、臂、几何
// ---------------------------------------------------------------------------------------------

/// 树的种类（`crates/singlefs-core/src/records.rs:373-385` 的十一种，加码 5 中央映射树）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum TreeKind {
    Extent,
    Inode,
    AllocationRecords,
    Accounting,
    CentralMapping,
    Livelist,
    SparseSideTable,
    Deadlist,
    DeletedInodes,
    Tombstone,
    Lineage,
    LoggedOperations,
}

impl TreeKind {
    const fn kind_code(self) -> u16 {
        match self {
            TreeKind::Extent => 1,
            TreeKind::Inode => 2,
            TreeKind::AllocationRecords => 3,
            TreeKind::Accounting => 4,
            TreeKind::CentralMapping => 5,
            TreeKind::Livelist => 6,
            TreeKind::SparseSideTable => 7,
            TreeKind::Deadlist => 8,
            TreeKind::DeletedInodes => 9,
            TreeKind::Tombstone => 10,
            TreeKind::Lineage => 11,
            TreeKind::LoggedOperations => 12,
        }
    }
}

/// 登记第一节「11 种树」：码 5 中央映射树不在里面（A1、K5）。
const ELEVEN_TREE_KINDS: [TreeKind; 11] = [
    TreeKind::Extent,
    TreeKind::Inode,
    TreeKind::AllocationRecords,
    TreeKind::Accounting,
    TreeKind::Livelist,
    TreeKind::SparseSideTable,
    TreeKind::Deadlist,
    TreeKind::DeletedInodes,
    TreeKind::Tombstone,
    TreeKind::Lineage,
    TreeKind::LoggedOperations,
];

/// key 的名字空间：每棵有对象的树一个；分配记录树按两个口径各一个；R-S、R-G 比的分配记录与映射真值另各一个（执行员修订 2 C3）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Namespace {
    Extent,
    Inode,
    AllocationRecordsPresent,
    AllocationRecordsAllocated,
    AllocationUnitsOfTheVersion,
    Accounting,
    CentralMapping,
    CentralMappingUnitsOfTheVersion,
    DeletedInodes,
    Tombstone,
}

const NAMESPACE_COUNT: usize = 10;

const NAMESPACES: [Namespace; NAMESPACE_COUNT] = [
    Namespace::Extent,
    Namespace::Inode,
    Namespace::AllocationRecordsPresent,
    Namespace::AllocationRecordsAllocated,
    Namespace::AllocationUnitsOfTheVersion,
    Namespace::Accounting,
    Namespace::CentralMapping,
    Namespace::CentralMappingUnitsOfTheVersion,
    Namespace::DeletedInodes,
    Namespace::Tombstone,
];

impl Namespace {
    const fn position(self) -> usize {
        match self {
            Namespace::Extent => 0,
            Namespace::Inode => 1,
            Namespace::AllocationRecordsPresent => 2,
            Namespace::AllocationRecordsAllocated => 3,
            Namespace::AllocationUnitsOfTheVersion => 4,
            Namespace::Accounting => 5,
            Namespace::CentralMapping => 6,
            Namespace::CentralMappingUnitsOfTheVersion => 7,
            Namespace::DeletedInodes => 8,
            Namespace::Tombstone => 9,
        }
    }
}

/// 一条 key 的字段（不用的补 0）：extent (inode, 单元号)；inode (inode)；分配记录 (槽号)；记账 (统计量, 树, 盘, 代)；
/// 映射 (类标签, 出生树, 出生 txg, 实例代号, 尾段)；`deleted_inodes` (inode)；墓碑 (对象, 出生代, 区间起, 区间止, 死亡 txg)。
type KeyFields = [u64; 5];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct NamespacedKey {
    namespace: Namespace,
    fields: KeyFields,
}

/// 产物里报的十三行树：十一种 + 码 5 中央映射树，分配记录树两个口径各一行。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum ReportedTree {
    Extent,
    Inode,
    AllocationRecordsPresent,
    AllocationRecordsAllocated,
    Accounting,
    CentralMapping,
    Livelist,
    SparseSideTable,
    Deadlist,
    DeletedInodes,
    Tombstone,
    Lineage,
    LoggedOperations,
}

const REPORTED_TREES: [ReportedTree; 13] = [
    ReportedTree::Extent,
    ReportedTree::Inode,
    ReportedTree::AllocationRecordsPresent,
    ReportedTree::AllocationRecordsAllocated,
    ReportedTree::Accounting,
    ReportedTree::CentralMapping,
    ReportedTree::Livelist,
    ReportedTree::SparseSideTable,
    ReportedTree::Deadlist,
    ReportedTree::DeletedInodes,
    ReportedTree::Tombstone,
    ReportedTree::Lineage,
    ReportedTree::LoggedOperations,
];

impl ReportedTree {
    const fn name(self) -> &'static str {
        match self {
            ReportedTree::Extent => "extent",
            ReportedTree::Inode => "inode",
            ReportedTree::AllocationRecordsPresent => "allocation_records_a",
            ReportedTree::AllocationRecordsAllocated => "allocation_records_b",
            ReportedTree::Accounting => "accounting",
            ReportedTree::CentralMapping => "central_mapping",
            ReportedTree::Livelist => "livelist",
            ReportedTree::SparseSideTable => "sparse_side_table",
            ReportedTree::Deadlist => "deadlist",
            ReportedTree::DeletedInodes => "deleted_inodes",
            ReportedTree::Tombstone => "tombstone",
            ReportedTree::Lineage => "lineage",
            ReportedTree::LoggedOperations => "logged_operations",
        }
    }

    const fn kind(self) -> TreeKind {
        match self {
            ReportedTree::Extent => TreeKind::Extent,
            ReportedTree::Inode => TreeKind::Inode,
            ReportedTree::AllocationRecordsPresent | ReportedTree::AllocationRecordsAllocated => {
                TreeKind::AllocationRecords
            }
            ReportedTree::Accounting => TreeKind::Accounting,
            ReportedTree::CentralMapping => TreeKind::CentralMapping,
            ReportedTree::Livelist => TreeKind::Livelist,
            ReportedTree::SparseSideTable => TreeKind::SparseSideTable,
            ReportedTree::Deadlist => TreeKind::Deadlist,
            ReportedTree::DeletedInodes => TreeKind::DeletedInodes,
            ReportedTree::Tombstone => TreeKind::Tombstone,
            ReportedTree::Lineage => TreeKind::Lineage,
            ReportedTree::LoggedOperations => TreeKind::LoggedOperations,
        }
    }

    /// 这一行在这个读法下比哪一个名字空间：R-S、R-G 下分配记录（两个口径）与映射比「这一版的码 1 / 码 3 单元」那一份（C3）；
    /// 今天没有单元的五棵树没有名字空间。
    const fn namespace(self, reader: Reader) -> Option<Namespace> {
        let selects = match reader {
            Reader::UnitScan => false,
            Reader::SelectCurrent | Reader::SelectCurrentGatedByObject => true,
        };
        match self {
            ReportedTree::Extent => Some(Namespace::Extent),
            ReportedTree::Inode => Some(Namespace::Inode),
            ReportedTree::AllocationRecordsPresent => {
                if selects {
                    Some(Namespace::AllocationUnitsOfTheVersion)
                } else {
                    Some(Namespace::AllocationRecordsPresent)
                }
            }
            ReportedTree::AllocationRecordsAllocated => {
                if selects {
                    Some(Namespace::AllocationUnitsOfTheVersion)
                } else {
                    Some(Namespace::AllocationRecordsAllocated)
                }
            }
            ReportedTree::Accounting => Some(Namespace::Accounting),
            ReportedTree::CentralMapping => {
                if selects {
                    Some(Namespace::CentralMappingUnitsOfTheVersion)
                } else {
                    Some(Namespace::CentralMapping)
                }
            }
            ReportedTree::DeletedInodes => Some(Namespace::DeletedInodes),
            ReportedTree::Tombstone => Some(Namespace::Tombstone),
            ReportedTree::Livelist
            | ReportedTree::SparseSideTable
            | ReportedTree::Deadlist
            | ReportedTree::Lineage
            | ReportedTree::LoggedOperations => None,
        }
    }
}

/// 扫描重建的三种读法（登记 5.4）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Reader {
    UnitScan,
    SelectCurrent,
    SelectCurrentGatedByObject,
}

const READERS: [Reader; 3] = [
    Reader::UnitScan,
    Reader::SelectCurrent,
    Reader::SelectCurrentGatedByObject,
];

impl Reader {
    const fn name(self) -> &'static str {
        match self {
            Reader::UnitScan => "R-U",
            Reader::SelectCurrent => "R-S",
            Reader::SelectCurrentGatedByObject => "R-G",
        }
    }
}

/// 墓碑政策三条臂（登记 5.3）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Arm {
    WithoutTombstones,
    TodaysTombstones,
    DeathRecordForEveryKey,
}

const ARMS: [Arm; 3] = [
    Arm::WithoutTombstones,
    Arm::TodaysTombstones,
    Arm::DeathRecordForEveryKey,
];

impl Arm {
    const fn name(self) -> &'static str {
        match self {
            Arm::WithoutTombstones => "N",
            Arm::TodaysTombstones => "B",
            Arm::DeathRecordForEveryKey => "P",
        }
    }

    /// 臂 P 的死亡记录放在模型专用的区、不占槽（登记 5.3），写路径与臂 N 相同；臂 B 在意图那次多写墓碑。
    const fn write_path(self) -> WritePath {
        match self {
            Arm::WithoutTombstones | Arm::DeathRecordForEveryKey => WritePath::WithoutTombstones,
            Arm::TodaysTombstones => WritePath::WithTombstones,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum WritePath {
    WithoutTombstones,
    WithTombstones,
}

impl WritePath {
    fn arms(self) -> Vec<Arm> {
        ARMS.iter().copied().filter(|arm| arm.write_path() == self).collect()
    }
}

/// 三个几何（修订 1 第 12.3 节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Geometry {
    Ring,
    Wide,
    Tight,
}

const GEOMETRIES: [Geometry; 3] = [Geometry::Ring, Geometry::Wide, Geometry::Tight];

impl Geometry {
    const fn name(self) -> &'static str {
        match self {
            Geometry::Ring => "ring",
            Geometry::Wide => "wide",
            Geometry::Tight => "tight",
        }
    }
}

// ---------------------------------------------------------------------------------------------
// 多层码 2 树（记账树、中央映射树）：照 `crates/singlefs-core/src/code_two_tree.rs:425-705` 的规则手写
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CodeTwoNodePosition {
    level: u8,
    index_in_level: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CodeTwoNodeContents {
    Leaf { keys: Vec<KeyFields> },
    Internal { children: Vec<(KeyFields, CodeTwoNodePosition)> },
}

/// 一版树的形状：节点按位置的全序（层级升序、同层按 key 升序），根在最末。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CodeTwoShape {
    nodes: Vec<(CodeTwoNodePosition, CodeTwoNodeContents)>,
}

impl CodeTwoShape {
    fn root_position(&self) -> Option<CodeTwoNodePosition> {
        self.nodes.last().map(|(position, _)| *position)
    }

    fn contents_at(&self, position: CodeTwoNodePosition) -> &CodeTwoNodeContents {
        let found = self
            .nodes
            .binary_search_by_key(&position, |(node_position, _)| *node_position)
            .expect("形状里的子引用指着形状里的节点");
        &self.nodes[found].1
    }

    fn keys_in_order(&self) -> Vec<KeyFields> {
        self.nodes
            .iter()
            .filter_map(|(_, contents)| match contents {
                CodeTwoNodeContents::Leaf { keys } => Some(keys.iter().copied()),
                CodeTwoNodeContents::Internal { .. } => None,
            })
            .flatten()
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CodeTwoOrigin {
    CarriedFrom(CodeTwoNodePosition),
    Rewritten,
}

#[derive(Clone, Debug)]
struct CodeTwoPlan {
    shape: CodeTwoShape,
    /// 与 `shape.nodes` 同序。
    origins: Vec<CodeTwoOrigin>,
    /// 上一版里这次被换下的节点。
    replaced_previous: Vec<CodeTwoNodePosition>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CodeTwoCapacity {
    leaf_entries: usize,
    internal_entries: usize,
}

/// 记账树：叶 477 条、内部 150 条（执行员修订 2 核对 4 M10）。
fn accounting_tree_capacity() -> CodeTwoCapacity {
    CodeTwoCapacity {
        leaf_entries: u64_as_index(index_node_entry_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_ENTRY_BYTES)),
        internal_entries: u64_as_index(index_node_entry_capacity(
            ACCOUNTING_KEY_BYTES,
            ACCOUNTING_KEY_BYTES + NODE_POINTER_BYTES,
        )),
    }
}

/// 中央映射树：叶 294 条、内部 143 条（M10）。
fn mapping_tree_capacity() -> CodeTwoCapacity {
    CodeTwoCapacity {
        leaf_entries: u64_as_index(index_node_entry_capacity(MAPPING_KEY_BYTES, MAPPING_ENTRY_BYTES)),
        internal_entries: u64_as_index(index_node_entry_capacity(
            MAPPING_KEY_BYTES,
            MAPPING_KEY_BYTES + NODE_POINTER_BYTES,
        )),
    }
}

#[derive(Clone, Debug)]
enum PlanningNode {
    Leaf {
        keys: Vec<KeyFields>,
        carried_from: Option<CodeTwoNodePosition>,
    },
    Internal {
        level: u8,
        children: Vec<PlanningChild>,
        carried_from: Option<CodeTwoNodePosition>,
    },
}

#[derive(Clone, Debug)]
struct PlanningChild {
    separator_key: KeyFields,
    node: PlanningNode,
}

impl PlanningNode {
    fn empty_leaf() -> Self {
        PlanningNode::Leaf {
            keys: Vec::new(),
            carried_from: None,
        }
    }

    fn level(&self) -> u8 {
        match self {
            PlanningNode::Leaf { .. } => 0,
            PlanningNode::Internal { level, .. } => *level,
        }
    }

    fn entry_count(&self) -> usize {
        match self {
            PlanningNode::Leaf { keys, .. } => keys.len(),
            PlanningNode::Internal { children, .. } => children.len(),
        }
    }

    fn mark_rewritten(&mut self) {
        match self {
            PlanningNode::Leaf { carried_from, .. } | PlanningNode::Internal { carried_from, .. } => {
                *carried_from = None;
            }
        }
    }

    fn smallest_key(&self) -> Option<KeyFields> {
        match self {
            PlanningNode::Leaf { keys, .. } => keys.first().copied(),
            PlanningNode::Internal { children, .. } => children.first()?.node.smallest_key(),
        }
    }
}

fn planning_node_of(shape: &CodeTwoShape, position: CodeTwoNodePosition) -> PlanningNode {
    match shape.contents_at(position) {
        CodeTwoNodeContents::Leaf { keys } => PlanningNode::Leaf {
            keys: keys.clone(),
            carried_from: Some(position),
        },
        CodeTwoNodeContents::Internal { children } => PlanningNode::Internal {
            level: position.level,
            children: children
                .iter()
                .map(|(separator_key, child)| PlanningChild {
                    separator_key: *separator_key,
                    node: planning_node_of(shape, *child),
                })
                .collect(),
            carried_from: Some(position),
        },
    }
}

/// 插入找孩子：最后一个分隔 key ≤ key 的那一个；比第一个还小就落到第 0 个并把它的分隔 key 压低（规则 ④）。
fn route_for_insertion(children: &mut [PlanningChild], key: &KeyFields) -> usize {
    match children.iter().rposition(|child| child.separator_key <= *key) {
        Some(found) => found,
        None => {
            children[0].separator_key = *key;
            0
        }
    }
}

fn route_for_deletion(children: &[PlanningChild], key: &KeyFields) -> usize {
    children
        .iter()
        .rposition(|child| child.separator_key <= *key)
        .unwrap_or(0)
}

/// 从中间切：左半留 ⌈n ÷ 2⌉ 条，右半的分隔 key 取它自己的最小 key（规则 ②）。
fn split_in_the_middle(node: &mut PlanningNode) -> PlanningChild {
    match node {
        PlanningNode::Leaf { keys, .. } => {
            let right_keys = keys.split_off(keys.len().div_ceil(2));
            PlanningChild {
                separator_key: right_keys[0],
                node: PlanningNode::Leaf {
                    keys: right_keys,
                    carried_from: None,
                },
            }
        }
        PlanningNode::Internal {
            level, children, ..
        } => {
            let right_children = children.split_off(children.len().div_ceil(2));
            PlanningChild {
                separator_key: right_children[0].separator_key,
                node: PlanningNode::Internal {
                    level: *level,
                    children: right_children,
                    carried_from: None,
                },
            }
        }
    }
}

fn insert_below(node: &mut PlanningNode, key: KeyFields, capacity: CodeTwoCapacity) -> Option<PlanningChild> {
    node.mark_rewritten();
    let overflows = match node {
        PlanningNode::Leaf { keys, .. } => {
            let insertion_point = keys.partition_point(|existing| *existing < key);
            assert!(keys.get(insertion_point) != Some(&key), "插入的 key 不在树里");
            keys.insert(insertion_point, key);
            keys.len() > capacity.leaf_entries
        }
        PlanningNode::Internal { children, .. } => {
            let child_position = route_for_insertion(children, &key);
            if let Some(right_half) = insert_below(&mut children[child_position].node, key, capacity) {
                children.insert(child_position + 1, right_half);
            }
            children.len() > capacity.internal_entries
        }
    };
    if overflows {
        Some(split_in_the_middle(node))
    } else {
        None
    }
}

fn insert_at_the_root(root: &mut PlanningNode, key: KeyFields, capacity: CodeTwoCapacity) {
    let Some(right_half) = insert_below(root, key, capacity) else {
        return;
    };
    let new_root_level = root.level() + 1;
    let left_half = std::mem::replace(root, PlanningNode::empty_leaf());
    let left_separator = left_half.smallest_key().expect("刚切开的节点左半不空");
    *root = PlanningNode::Internal {
        level: new_root_level,
        children: vec![
            PlanningChild {
                separator_key: left_separator,
                node: left_half,
            },
            right_half,
        ],
        carried_from: None,
    };
}

/// 删一把 key；删空的孩子从父节点里摘掉（规则 ③）。交回找没找到。
fn delete_below(node: &mut PlanningNode, key: &KeyFields) -> bool {
    let found = match node {
        PlanningNode::Leaf { keys, .. } => match keys.binary_search(key) {
            Ok(position) => {
                keys.remove(position);
                true
            }
            Err(_) => false,
        },
        PlanningNode::Internal { children, .. } => {
            let child_position = route_for_deletion(children, key);
            if !delete_below(&mut children[child_position].node, key) {
                return false;
            }
            if children[child_position].node.entry_count() == 0 {
                children.remove(child_position);
            }
            true
        }
    };
    if found {
        node.mark_rewritten();
    }
    found
}

fn delete_at_the_root(root: &mut PlanningNode, key: &KeyFields) {
    assert!(delete_below(root, key), "上一版的形状按分隔 key 走得到它自己叶里的 key");
    loop {
        let only_child = match root {
            PlanningNode::Internal { children, .. } if children.is_empty() => {
                *root = PlanningNode::empty_leaf();
                return;
            }
            PlanningNode::Internal { children, .. } if children.len() == 1 => {
                children.pop().expect("刚判过恰一个孩子").node
            }
            PlanningNode::Internal { .. } | PlanningNode::Leaf { .. } => return,
        };
        *root = only_child;
    }
}

type NodesByLevel = Vec<Vec<(CodeTwoNodeContents, CodeTwoOrigin)>>;

fn flatten(node: PlanningNode, nodes_by_level: &mut NodesByLevel) -> CodeTwoNodePosition {
    let level = node.level();
    let (contents, carried_from) = match node {
        PlanningNode::Leaf { keys, carried_from } => (CodeTwoNodeContents::Leaf { keys }, carried_from),
        PlanningNode::Internal {
            children,
            carried_from,
            ..
        } => {
            let mut flattened_children = Vec::new();
            for child in children {
                let child_position = flatten(child.node, nodes_by_level);
                flattened_children.push((child.separator_key, child_position));
            }
            (
                CodeTwoNodeContents::Internal {
                    children: flattened_children,
                },
                carried_from,
            )
        }
    };
    let level_position = usize::from(level);
    if nodes_by_level.len() <= level_position {
        nodes_by_level.resize_with(level_position + 1, Vec::new);
    }
    let index_in_level = u32::try_from(nodes_by_level[level_position].len()).expect("一层的节点数装得进 u32");
    let origin = match carried_from {
        Some(previous_position) => CodeTwoOrigin::CarriedFrom(previous_position),
        None => CodeTwoOrigin::Rewritten,
    };
    nodes_by_level[level_position].push((contents, origin));
    CodeTwoNodePosition {
        level,
        index_in_level,
    }
}

/// 这次之后树长什么样：先按 key 升序删上一版有、这一版没有的，再按 key 升序插这一版有、上一版没有的。
fn plan_code_two_tree(
    previous: &CodeTwoShape,
    keys_after_this_publish: &BTreeSet<KeyFields>,
    capacity: CodeTwoCapacity,
) -> CodeTwoPlan {
    assert!(!keys_after_this_publish.is_empty(), "记账树、映射树这一版恒有 key");
    let previous_keys: BTreeSet<KeyFields> = previous.keys_in_order().into_iter().collect();
    let mut root = match previous.root_position() {
        Some(root_position) => planning_node_of(previous, root_position),
        None => PlanningNode::empty_leaf(),
    };
    for deleted in previous_keys.difference(keys_after_this_publish) {
        delete_at_the_root(&mut root, deleted);
    }
    for inserted in keys_after_this_publish.difference(&previous_keys) {
        insert_at_the_root(&mut root, *inserted, capacity);
    }
    let mut nodes_by_level: NodesByLevel = Vec::new();
    flatten(root, &mut nodes_by_level);
    let mut nodes = Vec::new();
    let mut origins = Vec::new();
    for (level, nodes_of_this_level) in nodes_by_level.into_iter().enumerate() {
        for (index_in_level, (contents, origin)) in nodes_of_this_level.into_iter().enumerate() {
            nodes.push((
                CodeTwoNodePosition {
                    level: u8::try_from(level).expect("层级是 1 字节"),
                    index_in_level: u32::try_from(index_in_level).expect("一层的节点数装得进 u32"),
                },
                contents,
            ));
            origins.push(origin);
        }
    }
    let carried: BTreeSet<CodeTwoNodePosition> = origins
        .iter()
        .filter_map(|origin| match origin {
            CodeTwoOrigin::CarriedFrom(previous_position) => Some(*previous_position),
            CodeTwoOrigin::Rewritten => None,
        })
        .collect();
    let replaced_previous = previous
        .nodes
        .iter()
        .map(|(position, _)| *position)
        .filter(|position| !carried.contains(position))
        .collect();
    CodeTwoPlan {
        shape: CodeTwoShape { nodes },
        origins,
        replaced_previous,
    }
}

/// 一版码 2 树：形状加每个节点这一版的单元（与 `shape.nodes` 同序）。
#[derive(Clone, Debug, Default)]
struct CodeTwoTreeVersion {
    shape: CodeTwoShape,
    units: Vec<UnitId>,
}

impl CodeTwoTreeVersion {
    fn unit_at(&self, position: CodeTwoNodePosition) -> UnitId {
        let found = self
            .shape
            .nodes
            .binary_search_by_key(&position, |(node_position, _)| *node_position)
            .expect("上一版的位置在上一版的形状里");
        self.units[found]
    }
}

// ---------------------------------------------------------------------------------------------
// 分配器：W2–W6（G-环、G-宽）与 G-紧 的最低落点；根环表 W5
// ---------------------------------------------------------------------------------------------

type UnitId = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnitFootprint {
    TwoSlotsAligned,
    OneSlot,
}

impl UnitFootprint {
    const fn slots(self) -> u64 {
        match self {
            UnitFootprint::TwoSlotsAligned => 2,
            UnitFootprint::OneSlot => 1,
        }
    }

    const fn alignment_in_slots(self) -> u64 {
        match self {
            UnitFootprint::TwoSlotsAligned => 2,
            UnitFootprint::OneSlot => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlacementRule {
    UserData,
    CommitGenerated(UnitFootprint),
}

impl PlacementRule {
    const fn span_slots(self) -> u64 {
        match self {
            PlacementRule::UserData => 2,
            PlacementRule::CommitGenerated(footprint) => footprint.slots(),
        }
    }
}

/// 一条分配记录（两块盘同槽、内容只差设备段，按槽号记一条，执行员修订 2 C1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AllocationRecord {
    span_slots: u64,
    /// 仍分配时是分配代，已释放时是释放代。
    generation: u64,
    is_released: bool,
    unit: UnitId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootRingOccupant {
    ValidRoot(u64),
    /// 挂载时读回全零的槽，钉在它可能住过的最旧 txg（M4）。
    AllZeroPinnedAt(u64),
}

impl RootRingOccupant {
    const fn txg(self) -> u64 {
        match self {
            RootRingOccupant::ValidRoot(txg) | RootRingOccupant::AllZeroPinnedAt(txg) => txg,
        }
    }
}

/// 根 t 写在哪个环槽：区 t mod 3、槽 ⌊t ÷ 3⌋ mod 8（`root_ring.rs:109-117`）。
fn root_ring_slot_of(txg: u64) -> (u64, u64) {
    (
        txg % ROOT_RING_REGIONS,
        (txg / ROOT_RING_REGIONS) % ROOT_RING_SLOTS_PER_REGION,
    )
}

/// 分配器眼里的根环（W5）。
#[derive(Clone, Debug)]
struct RootRingTable {
    latest_recorded_txg: u64,
    occupants: BTreeMap<(u64, u64), RootRingOccupant>,
    /// F_生效：整段历史不抬 F，新池是 0（W7）。
    effective_floor: u64,
}

impl RootRingTable {
    /// mkfs 那个进程：三个区的槽 0 各一条 txg 0（M3）。
    fn after_make_filesystem() -> Self {
        Self {
            latest_recorded_txg: 0,
            occupants: (0..ROOT_RING_REGIONS)
                .map(|region| ((region, 0), RootRingOccupant::ValidRoot(0)))
                .collect(),
            effective_floor: 0,
        }
    }

    /// 可写挂载从盘上读：有根的槽记它的 txg，读回全零的槽钉在 r + 3s（M4）。
    fn read_at_mount(roots_on_disk: &BTreeMap<(u64, u64), u64>, latest_recorded_txg: u64) -> Self {
        let mut occupants = BTreeMap::new();
        for region in 0..ROOT_RING_REGIONS {
            for slot in 0..ROOT_RING_SLOTS_PER_REGION {
                let occupant = match roots_on_disk.get(&(region, slot)) {
                    Some(txg) => RootRingOccupant::ValidRoot(*txg),
                    None => RootRingOccupant::AllZeroPinnedAt(region + ROOT_RING_REGIONS * slot),
                };
                occupants.insert((region, slot), occupant);
            }
        }
        Self {
            latest_recorded_txg,
            occupants,
            effective_floor: 0,
        }
    }

    fn reclaim_floor(&self) -> u64 {
        let oldest = self
            .occupants
            .values()
            .map(|occupant| occupant.txg())
            .min()
            .unwrap_or(self.effective_floor);
        self.effective_floor.max(oldest)
    }

    /// 记一条这个进程写的根，交回此刻的回收门槛 max(F_生效, 环里最旧)。
    fn record_root(&mut self, txg: u64) -> u64 {
        assert_eq!(txg, self.latest_recorded_txg + 1, "根的 txg 逐个加一，每一条都经分配器记");
        self.latest_recorded_txg = txg;
        self.occupants
            .insert(root_ring_slot_of(txg), RootRingOccupant::ValidRoot(txg));
        self.reclaim_floor()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PlacementRefused;

#[derive(Clone, Debug)]
struct Allocator {
    geometry: Geometry,
    /// 单元区槽数；G-宽 不设末端（`None`）。
    unit_area_slots: Option<u64>,
    allocated: Vec<bool>,
    ever_allocated: Vec<bool>,
    used_per_segment: Vec<u64>,
    allocated_slot_count: u64,
    /// 仍分配（没释放）的槽数：G-紧 的盘按 G-宽 下它的峰值定（C7）。
    live_slot_count: u64,
    open_segment: Option<u64>,
    bump_cursor: u64,
    /// 这次挂载开过的聚簇段（W2）。
    cluster_segments: BTreeSet<u64>,
    records: BTreeMap<u64, AllocationRecord>,
    records_per_leaf: BTreeMap<u64, u64>,
    reclaimed: BTreeSet<u64>,
    root_ring: Option<RootRingTable>,
    /// 这份分配器（固定点那一份拷贝）上动过记录的槽。
    dirty_slots: BTreeSet<u64>,
    reuse_placements: u64,
    first_reuse_at: Option<u64>,
}

impl Allocator {
    fn new(geometry: Geometry, unit_area_slots: Option<u64>) -> Self {
        let initial_slots = unit_area_slots.unwrap_or(0);
        Self {
            geometry,
            unit_area_slots,
            allocated: vec![false; u64_as_index(initial_slots)],
            ever_allocated: vec![false; u64_as_index(initial_slots)],
            used_per_segment: vec![0; u64_as_index(initial_slots.div_ceil(CLUSTER_SEGMENT_SLOTS))],
            allocated_slot_count: 0,
            live_slot_count: 0,
            open_segment: None,
            bump_cursor: 0,
            cluster_segments: BTreeSet::new(),
            records: BTreeMap::new(),
            records_per_leaf: BTreeMap::new(),
            reclaimed: BTreeSet::new(),
            root_ring: None,
            dirty_slots: BTreeSet::new(),
            reuse_placements: 0,
            first_reuse_at: None,
        }
    }

    fn rehearsal_copy(&self) -> Self {
        let mut copy = self.clone();
        copy.dirty_slots.clear();
        copy
    }

    fn offset_of(slot: u64) -> usize {
        u64_as_index(slot.checked_sub(UNIT_AREA_START_SLOT).expect("槽号在单元区内"))
    }

    fn segment_start_of(slot: u64) -> u64 {
        UNIT_AREA_START_SLOT + ((slot - UNIT_AREA_START_SLOT) / CLUSTER_SEGMENT_SLOTS) * CLUSTER_SEGMENT_SLOTS
    }

    fn is_inside_the_unit_area(&self, slot: u64) -> bool {
        slot >= UNIT_AREA_START_SLOT
            && self
                .unit_area_slots
                .is_none_or(|slots| slot < UNIT_AREA_START_SLOT + slots)
    }

    fn is_free(&self, slot: u64) -> bool {
        if !self.is_inside_the_unit_area(slot) {
            return false;
        }
        let offset = Self::offset_of(slot);
        offset >= self.allocated.len() || !self.allocated[offset]
    }

    fn grow_to_cover(&mut self, slot_end_exclusive: u64) {
        let needed = Self::offset_of(slot_end_exclusive);
        if self.allocated.len() < needed {
            assert_eq!(self.geometry, Geometry::Wide, "只有 G-宽 的单元区不设末端");
            let grown = needed.max(self.allocated.len() * 2).max(4096);
            self.allocated.resize(grown, false);
            self.ever_allocated.resize(grown, false);
            self.used_per_segment
                .resize(u64_as_index(length_as_u64(grown).div_ceil(CLUSTER_SEGMENT_SLOTS)), 0);
        }
    }

    fn mark_slot_allocated(&mut self, slot: u64) {
        let offset = Self::offset_of(slot);
        assert!(!self.allocated[offset], "跨度里有已分配的槽");
        self.allocated[offset] = true;
        self.ever_allocated[offset] = true;
        self.used_per_segment[offset / u64_as_index(CLUSTER_SEGMENT_SLOTS)] += 1;
        self.allocated_slot_count += 1;
    }

    fn mark_slot_free(&mut self, slot: u64) {
        let offset = Self::offset_of(slot);
        assert!(self.allocated[offset], "回收的跨度里有没分配的槽");
        self.allocated[offset] = false;
        self.used_per_segment[offset / u64_as_index(CLUSTER_SEGMENT_SLOTS)] -= 1;
        self.allocated_slot_count -= 1;
    }

    fn leaf_of_slot(slot: u64) -> u64 {
        slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS
    }

    fn count_record_in_its_leaf(&mut self, slot: u64) {
        *self.records_per_leaf.entry(Self::leaf_of_slot(slot)).or_insert(0) += 1;
    }

    fn uncount_record_in_its_leaf(&mut self, slot: u64) {
        let leaf = Self::leaf_of_slot(slot);
        let count = self.records_per_leaf.get_mut(&leaf).expect("记录所在的叶有计数");
        *count -= 1;
        if *count == 0 {
            self.records_per_leaf.remove(&leaf);
        }
    }

    /// 用户数据：起点最小、偶数、两槽都空；G-环、G-宽 不落在这次挂载开过的任何聚簇段里（W2），G-紧 不模聚簇段。
    fn lowest_user_data_slot(&self) -> Option<u64> {
        let mut candidate = UNIT_AREA_START_SLOT;
        loop {
            if let Some(slots) = self.unit_area_slots {
                if candidate + 1 >= UNIT_AREA_START_SLOT + slots {
                    return None;
                }
            }
            if self.geometry != Geometry::Tight {
                let segment_start = Self::segment_start_of(candidate);
                if self.cluster_segments.contains(&segment_start) {
                    candidate = segment_start + CLUSTER_SEGMENT_SLOTS;
                    continue;
                }
            }
            if self.is_free(candidate) && self.is_free(candidate + 1) {
                return Some(candidate);
            }
            candidate += 2;
        }
    }

    /// 单元区里最低的整 64 槽全空段（W3）。
    fn lowest_empty_segment(&self) -> Option<u64> {
        let full_segments = match self.unit_area_slots {
            Some(slots) => slots / CLUSTER_SEGMENT_SLOTS,
            None => u64::MAX,
        };
        let mut segment = 0;
        while segment < full_segments {
            let position = u64_as_index(segment);
            if position >= self.used_per_segment.len() || self.used_per_segment[position] == 0 {
                return Some(UNIT_AREA_START_SLOT + segment * CLUSTER_SEGMENT_SLOTS);
            }
            segment += 1;
        }
        None
    }

    /// 最低的整个跨度都空的起点，按对齐步长走（W3 的回落；G-紧 的提交内生块）。
    fn lowest_fitting_commit_generated_slot(&self, footprint: UnitFootprint) -> Option<u64> {
        let mut candidate = UNIT_AREA_START_SLOT;
        loop {
            if let Some(slots) = self.unit_area_slots {
                if candidate + footprint.slots() > UNIT_AREA_START_SLOT + slots {
                    return None;
                }
            }
            if (candidate..candidate + footprint.slots()).all(|slot| self.is_free(slot)) {
                return Some(candidate);
            }
            candidate += footprint.alignment_in_slots();
        }
    }

    /// 开放段里从游标起第一个整跨度都空的对齐起点（W3）。
    fn bump_slot_in_open_segment(&self, open: u64, footprint: UnitFootprint) -> Option<u64> {
        let segment_end = open + CLUSTER_SEGMENT_SLOTS;
        let mut start = self.bump_cursor;
        if footprint == UnitFootprint::TwoSlotsAligned && !start.is_multiple_of(2) {
            start += 1;
        }
        while start + footprint.slots() <= segment_end
            && (start..start + footprint.slots()).any(|slot| !self.is_free(slot))
        {
            start += footprint.alignment_in_slots();
        }
        if start + footprint.slots() > segment_end {
            None
        } else {
            Some(start)
        }
    }

    fn commit_generated_slot(&mut self, footprint: UnitFootprint) -> Option<u64> {
        if let Some(open) = self.open_segment {
            if let Some(slot) = self.bump_slot_in_open_segment(open, footprint) {
                self.bump_cursor = slot + footprint.slots();
                return Some(slot);
            }
        }
        match self.lowest_empty_segment() {
            Some(segment_start) => {
                self.open_segment = Some(segment_start);
                self.bump_cursor = segment_start;
                self.cluster_segments.insert(segment_start);
                let slot = self
                    .bump_slot_in_open_segment(segment_start, footprint)
                    .expect("全空段装得下任何一个单元");
                self.bump_cursor = slot + footprint.slots();
                Some(slot)
            }
            None => {
                let slot = self.lowest_fitting_commit_generated_slot(footprint)?;
                self.open_segment = None;
                Some(slot)
            }
        }
    }

    fn lowest_fitting_slot_without_segments(&self, rule: PlacementRule) -> Option<u64> {
        match rule {
            PlacementRule::UserData => self.lowest_user_data_slot(),
            PlacementRule::CommitGenerated(footprint) => self.lowest_fitting_commit_generated_slot(footprint),
        }
    }

    /// G-紧：取不到落点时把「扣着」里除这次发布换下的之外全部放回空闲，再取（原文 5.1；C7）。
    fn tight_slot(&mut self, rule: PlacementRule, txg: u64) -> Option<u64> {
        if let Some(slot) = self.lowest_fitting_slot_without_segments(rule) {
            return Some(slot);
        }
        self.reclaim_released_records_where(|record| record.generation < txg);
        self.lowest_fitting_slot_without_segments(rule)
    }

    fn allocate(&mut self, rule: PlacementRule, txg: u64, unit: UnitId) -> Result<u64, PlacementRefused> {
        let slot = match self.geometry {
            Geometry::Tight => self.tight_slot(rule, txg),
            Geometry::Ring | Geometry::Wide => match rule {
                PlacementRule::UserData => self.lowest_user_data_slot(),
                PlacementRule::CommitGenerated(footprint) => self.commit_generated_slot(footprint),
            },
        }
        .ok_or(PlacementRefused)?;
        self.record(slot, rule.span_slots(), txg, unit);
        Ok(slot)
    }

    /// 记下一次分配（W6）：起点就是这个槽的已回收记录改写成已分配，罩住的别的已回收记录删掉。
    fn record(&mut self, slot: u64, span: u64, txg: u64, unit: UnitId) {
        let end = slot + span;
        self.grow_to_cover(end);
        let is_reuse = (slot..end).any(|each| self.ever_allocated[Self::offset_of(each)]);
        if is_reuse {
            self.reuse_placements += 1;
            if self.first_reuse_at.is_none() {
                self.first_reuse_at = Some(txg);
            }
        }
        let intersecting: Vec<u64> = self
            .records
            .range(slot.saturating_sub(1)..end)
            .filter(|(record_slot, record)| **record_slot + record.span_slots > slot)
            .map(|(record_slot, _)| *record_slot)
            .collect();
        let mut rewritten_in_place = false;
        for record_slot in intersecting {
            assert!(
                self.reclaimed.remove(&record_slot),
                "与新落点相交的记录是已回收的：位图与记录分叉了"
            );
            self.dirty_slots.insert(record_slot);
            if record_slot == slot {
                let existing = self.records.get_mut(&record_slot).expect("刚找到这条记录");
                assert!(existing.is_released, "回收过的落点的记录是已释放的");
                *existing = AllocationRecord {
                    span_slots: span,
                    generation: txg,
                    is_released: false,
                    unit,
                };
                rewritten_in_place = true;
            } else {
                self.records.remove(&record_slot);
                self.uncount_record_in_its_leaf(record_slot);
            }
        }
        if !rewritten_in_place {
            self.records.insert(
                slot,
                AllocationRecord {
                    span_slots: span,
                    generation: txg,
                    is_released: false,
                    unit,
                },
            );
            self.count_record_in_its_leaf(slot);
        }
        self.dirty_slots.insert(slot);
        for each in slot..end {
            self.mark_slot_allocated(each);
        }
        self.live_slot_count += span;
    }

    /// 释放（W4）：记录改成已释放 + 释放代，槽仍占着。
    fn release(&mut self, slot: u64, span: u64, txg: u64) {
        let record = self.records.get_mut(&slot).expect("释放的落点在册");
        assert_eq!(record.span_slots, span, "释放的跨度与记录相同");
        assert!(!record.is_released, "同一个落点不释放两次");
        record.is_released = true;
        record.generation = txg;
        self.dirty_slots.insert(slot);
        self.live_slot_count -= span;
    }

    /// 回收满足条件的已释放记录：槽回到空闲，记录照旧写着已释放（W5、W6）。交回回收的落点 (槽, 跨度)。
    fn reclaim_released_records_where(&mut self, should_reclaim: impl Fn(&AllocationRecord) -> bool) -> Vec<(u64, u64)> {
        let candidates: Vec<(u64, u64)> = self
            .records
            .iter()
            .filter(|entry| {
                let (slot, record) = *entry;
                record.is_released && should_reclaim(record) && !self.reclaimed.contains(slot)
            })
            .map(|(slot, record)| (*slot, record.span_slots))
            .collect();
        for (slot, span) in &candidates {
            self.reclaimed.insert(*slot);
            for each in *slot..*slot + *span {
                self.mark_slot_free(each);
            }
        }
        candidates
    }

    /// 可写挂载重建分配器：没有开放段、聚簇段集合清空（M4）；G-环 装上从盘上读的根环表并按挂载那一刻的门槛回收。
    fn reset_at_mount(&mut self, roots_on_disk: &BTreeMap<(u64, u64), u64>, latest_recorded_txg: u64) {
        self.open_segment = None;
        self.bump_cursor = 0;
        self.cluster_segments.clear();
        if self.geometry == Geometry::Ring {
            let table = RootRingTable::read_at_mount(roots_on_disk, latest_recorded_txg);
            let floor = table.reclaim_floor();
            self.root_ring = Some(table);
            self.reclaim_released_records_where(|record| record.generation <= floor);
        }
    }

    /// 这次发布的落点取完之后记根（W5）：G-环 按门槛回收，交回回收的落点；G-宽、G-紧 不按环回收。
    fn record_root(&mut self, txg: u64) -> Vec<(u64, u64)> {
        match self.geometry {
            Geometry::Ring => {
                let floor = self
                    .root_ring
                    .as_mut()
                    .expect("G-环 的分配器装着根环表")
                    .record_root(txg);
                self.reclaim_released_records_where(|record| record.generation <= floor)
            }
            Geometry::Wide | Geometry::Tight => Vec::new(),
        }
    }

    /// 装着记录的叶（两块盘各一片）、它们的祖先与根（`allocation_record_tree.rs:335-363` 的口径）。
    fn nodes_holding_records(&self) -> BTreeSet<AllocationNode> {
        allocation_nodes_of_leaves(self.records_per_leaf.keys().copied())
    }
}

/// 分配记录树的一个节点：根之下按 (层级, 盘, 同盘同层序号)，根排最末（`allocation_record_tree.rs:44-56`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum AllocationNode {
    BelowTheRoot { level: u8, device: u8, index_in_device: u64 },
    Root,
}

fn allocation_span_in_slots_at_level(level: u8) -> u64 {
    (0..level).fold(ALLOCATION_RECORD_TREE_LEAF_SLOTS, |span, _| {
        span.saturating_mul(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT)
    })
}

/// 根的层级：最小的 R ≥ 1 使 Σ_盘 ⌈盘上绝对槽数 ÷ S_{R−1}⌉ ≤ F（`allocation_record_tree.rs:94-116`）。
fn allocation_root_level(device_slots: u64, devices: u64) -> u8 {
    (1..=u8::MAX)
        .find(|candidate| {
            device_slots.div_ceil(allocation_span_in_slots_at_level(candidate - 1)) * devices
                <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
        })
        .expect("两块盘时层级至多几层就装得下")
}

/// 三个几何都按 4 GiB 两盘的几何分叶与祖先（执行员修订 2 C8）。
fn allocation_tree_root_level() -> u8 {
    allocation_root_level(DEVICE_SLOTS, u64::from(DEVICE_COUNT))
}

fn allocation_parent_of(node: AllocationNode, root_level: u8) -> Option<AllocationNode> {
    match node {
        AllocationNode::Root => None,
        AllocationNode::BelowTheRoot {
            level,
            device,
            index_in_device,
        } => {
            if level + 1 == root_level {
                Some(AllocationNode::Root)
            } else {
                Some(AllocationNode::BelowTheRoot {
                    level: level + 1,
                    device,
                    index_in_device: index_in_device / ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
                })
            }
        }
    }
}

fn allocation_nodes_of_leaves(leaves: impl Iterator<Item = u64>) -> BTreeSet<AllocationNode> {
    let root_level = allocation_tree_root_level();
    let mut nodes = BTreeSet::new();
    nodes.insert(AllocationNode::Root);
    for leaf in leaves {
        for device in 0..DEVICE_COUNT {
            let mut current = Some(AllocationNode::BelowTheRoot {
                level: 0,
                device,
                index_in_device: leaf,
            });
            while let Some(node) = current {
                if !nodes.insert(node) && node != AllocationNode::Root {
                    break;
                }
                current = allocation_parent_of(node, root_level);
            }
        }
    }
    nodes
}

/// 从 `before` 到 `after`，记录逐槽变了的叶连祖先与根（`nodes_whose_contents_changed` 的口径；比的是 `after` 上动过的槽）。
fn allocation_nodes_whose_contents_changed(before: &Allocator, after: &Allocator) -> BTreeSet<AllocationNode> {
    let changed_leaves: BTreeSet<u64> = after
        .dirty_slots
        .iter()
        .filter(|slot| before.records.get(*slot) != after.records.get(*slot))
        .map(|slot| Allocator::leaf_of_slot(*slot))
        .collect();
    if changed_leaves.is_empty() {
        return BTreeSet::new();
    }
    allocation_nodes_of_leaves(changed_leaves.into_iter())
}

// ---------------------------------------------------------------------------------------------
// 单元与盘：槽阵列只模一份（两盘同槽同内容，登记 5.1）；一个单元的任一槽被之后的写盖过就读不出（W8）
// ---------------------------------------------------------------------------------------------

/// 择现行的分组（登记 5.4）：数据单元按 (出生树, 对象, 对象出生代, 锚点)，码 3 容器按 (出生树, 类型, 容器号, 容器出生代)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum SelectionGroup {
    DataUnit {
        birth_tree: u64,
        object: u64,
        object_birth: u64,
        anchor_unit: u64,
    },
    PackedContainer {
        birth_tree: u64,
        record_type: u64,
        container_number: u64,
        container_birth: u64,
    },
}

/// 择新键 (诞生代号, 实例代号, 事务号)。
type WriteOrder = (u64, u64, u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum UnitOwner {
    FileData,
    ExtentLowerNode,
    ExtentRoot,
    InodeContainer,
    InodeRoot,
    AllocationNode,
    AccountingNode,
    MappingNode,
    DeletedInodesRoot,
    TombstoneContainer,
    TombstoneRoot,
    TreeTable,
    InstanceTable,
}

impl UnitOwner {
    /// 这个单元算哪棵树的单元（判「没有对象」用）；数据单元、树表、实例表不算哪棵树的节点。
    const fn tree_kind(self) -> Option<TreeKind> {
        match self {
            UnitOwner::ExtentLowerNode | UnitOwner::ExtentRoot => Some(TreeKind::Extent),
            UnitOwner::InodeContainer | UnitOwner::InodeRoot => Some(TreeKind::Inode),
            UnitOwner::AllocationNode => Some(TreeKind::AllocationRecords),
            UnitOwner::AccountingNode => Some(TreeKind::Accounting),
            UnitOwner::MappingNode => Some(TreeKind::CentralMapping),
            UnitOwner::DeletedInodesRoot => Some(TreeKind::DeletedInodes),
            UnitOwner::TombstoneContainer | UnitOwner::TombstoneRoot => Some(TreeKind::Tombstone),
            UnitOwner::FileData | UnitOwner::TreeTable | UnitOwner::InstanceTable => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InodeRecordSummary {
    inode: u64,
    object_birth: u64,
    nlink: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TombstoneRecordSummary {
    object: u64,
    object_birth: u64,
    death_txg: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExtentRecordSummary {
    inode: u64,
    data_unit: u64,
    /// 它指着的那个数据单元的出生 txg（C9）。
    pointer_birth: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum UnitContents {
    DataUnit {
        object: u64,
        object_birth: u64,
        anchor_unit: u64,
    },
    InodeRecords(Vec<InodeRecordSummary>),
    TombstoneRecords(Vec<TombstoneRecordSummary>),
    ExtentRecords(Vec<ExtentRecordSummary>),
    /// (槽号, 是否已释放)。
    AllocationRecords(Vec<(u64, bool)>),
    AccountingKeys(Vec<KeyFields>),
    MappingKeys(Vec<KeyFields>),
    DeletedInodes(Vec<u64>),
    WithoutKeys,
}

/// 一次出现带的身份（臂 B 的墓碑效力按它判，登记 5.3）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OccurrenceTag {
    InodeRecord { inode: u64, object_birth: u64 },
    ExtentRecord { inode: u64, pointer_birth: u64 },
    DeletedInode { inode: u64 },
    Untagged,
}

#[derive(Clone, Copy, Debug)]
struct Occurrence {
    key: NamespacedKey,
    tag: OccurrenceTag,
}

fn single_field_key(value: u64) -> KeyFields {
    [value, 0, 0, 0, 0]
}

fn extent_key(inode: u64, data_unit: u64) -> KeyFields {
    [inode, data_unit, 0, 0, 0]
}

fn tombstone_key(record: TombstoneRecordSummary) -> KeyFields {
    [record.object, record.object_birth, 0, WHOLE_OBJECT_RANGE_END, record.death_txg]
}

fn namespaced(namespace: Namespace, fields: KeyFields) -> NamespacedKey {
    NamespacedKey { namespace, fields }
}

/// R-U 读这个单元收到的每一次 key 出现（登记 5.4 R-U 那一列）。
fn occurrences_of(contents: &UnitContents) -> Vec<Occurrence> {
    let untagged = |namespace: Namespace, fields: KeyFields| Occurrence {
        key: namespaced(namespace, fields),
        tag: OccurrenceTag::Untagged,
    };
    match contents {
        UnitContents::InodeRecords(records) => records
            .iter()
            .map(|record| Occurrence {
                key: namespaced(Namespace::Inode, single_field_key(record.inode)),
                tag: OccurrenceTag::InodeRecord {
                    inode: record.inode,
                    object_birth: record.object_birth,
                },
            })
            .collect(),
        UnitContents::TombstoneRecords(records) => records
            .iter()
            .map(|record| untagged(Namespace::Tombstone, tombstone_key(*record)))
            .collect(),
        UnitContents::ExtentRecords(records) => records
            .iter()
            .map(|record| Occurrence {
                key: namespaced(Namespace::Extent, extent_key(record.inode, record.data_unit)),
                tag: OccurrenceTag::ExtentRecord {
                    inode: record.inode,
                    pointer_birth: record.pointer_birth,
                },
            })
            .collect(),
        UnitContents::AllocationRecords(records) => {
            let mut occurrences = Vec::new();
            for (slot, is_released) in records {
                occurrences.push(untagged(Namespace::AllocationRecordsPresent, single_field_key(*slot)));
                if !is_released {
                    occurrences.push(untagged(Namespace::AllocationRecordsAllocated, single_field_key(*slot)));
                }
            }
            occurrences
        }
        UnitContents::AccountingKeys(keys) => keys
            .iter()
            .map(|fields| untagged(Namespace::Accounting, *fields))
            .collect(),
        UnitContents::MappingKeys(keys) => keys
            .iter()
            .map(|fields| untagged(Namespace::CentralMapping, *fields))
            .collect(),
        UnitContents::DeletedInodes(inodes) => inodes
            .iter()
            .map(|inode| Occurrence {
                key: namespaced(Namespace::DeletedInodes, single_field_key(*inode)),
                tag: OccurrenceTag::DeletedInode { inode: *inode },
            })
            .collect(),
        UnitContents::DataUnit { .. } | UnitContents::WithoutKeys => Vec::new(),
    }
}

#[derive(Clone, Debug)]
struct Unit {
    owner: UnitOwner,
    class: u64,
    group: Option<SelectionGroup>,
    write_order: WriteOrder,
    slot: u64,
    span_slots: u64,
    release_txg: Option<u64>,
    is_readable: bool,
    mapping_key: Option<KeyFields>,
    contents: UnitContents,
}

impl Unit {
    fn birth_txg(&self) -> u64 {
        self.write_order.0
    }
}

#[derive(Clone, Debug, Default)]
struct Disk {
    units: Vec<Unit>,
    occupant_of_slot: HashMap<u64, UnitId>,
    units_written: u64,
    units_made_unreadable: u64,
}

impl Disk {
    /// 写一个单元：它占的每一槽上原来那个单元读不出了（整个或部分盖住一样，W8）。交回这次变得读不出的单元。
    fn write(&mut self, unit: Unit) -> Vec<UnitId> {
        let unit_id = self.units.len();
        let mut made_unreadable = Vec::new();
        for slot in unit.slot..unit.slot + unit.span_slots {
            if let Some(previous) = self.occupant_of_slot.insert(slot, unit_id) {
                if self.units[previous].is_readable {
                    self.units[previous].is_readable = false;
                    self.units_made_unreadable += 1;
                    made_unreadable.push(previous);
                }
            }
        }
        self.units.push(unit);
        self.units_written += 1;
        made_unreadable
    }

    fn readable_units_by_scan(&self) -> u64 {
        length_as_u64(self.units.iter().filter(|unit| unit.is_readable).count())
    }
}

// ---------------------------------------------------------------------------------------------
// 一版的样子与一次发布的计划（S3–S13、M5–M11）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct InodeContainerVersion {
    container_number: u64,
    container_birth: u64,
    records: Vec<InodeRecordSummary>,
    unit: UnitId,
}

#[derive(Clone, Debug)]
struct TombstoneContainerVersion {
    container_number: u64,
    container_birth: u64,
    records: Vec<TombstoneRecordSummary>,
    unit: UnitId,
}

#[derive(Clone, Debug)]
struct Version {
    txg: u64,
    instance: u64,
    highest_transaction: u64,
    next_inode: u64,
    /// inode 1 的数据单元，按单元序号。
    file_units: Vec<UnitId>,
    extent_root: Option<UnitId>,
    extent_lower: BTreeMap<(u8, u64), UnitId>,
    inode_containers: Vec<InodeContainerVersion>,
    inode_root: Option<UnitId>,
    deleted_queue: Vec<u64>,
    deleted_root: Option<UnitId>,
    tombstone_containers: Vec<TombstoneContainerVersion>,
    tombstone_root: Option<UnitId>,
    allocation_nodes: BTreeMap<AllocationNode, UnitId>,
    accounting: CodeTwoTreeVersion,
    mapping: CodeTwoTreeVersion,
    tree_table: UnitId,
    instance_table: UnitId,
}

impl Version {
    fn live_record_of(&self, inode: u64) -> Option<InodeRecordSummary> {
        self.inode_containers
            .iter()
            .flat_map(|container| container.records.iter())
            .find(|record| record.inode == inode)
            .copied()
    }

    fn has_live_inode(&self, inode: u64) -> bool {
        self.live_record_of(inode).is_some_and(|record| record.nlink > 0)
    }

    /// 「有树、文件集合为空」（`transaction.rs:2968-2977`）：没有 inode 叶容器、没有数据单元、`deleted_inodes` 无根。
    fn is_with_trees_and_no_file(&self) -> bool {
        self.inode_containers.is_empty() && self.file_units.is_empty() && self.deleted_root.is_none()
    }
}

/// 一次发布做什么（M5、S12、S13）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PublishOperation {
    FirstFile,
    RowPublish,
    WarmUp,
    Overwrite,
    SequentialWrite { data_units: u64 },
    NewInodes { count: u64 },
    UnlinkIntent { inode: u64 },
    UnlinkBatch { data_units_kept: u64 },
    UnlinkFinal { inode: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlannedUnit {
    Rewritten,
    Carried(UnitId),
}

#[derive(Clone, Debug)]
struct PlannedInodeContainer {
    container_number: u64,
    container_birth: u64,
    records: Vec<InodeRecordSummary>,
    origin: PlannedUnit,
}

#[derive(Clone, Debug)]
struct PlannedTombstoneContainer {
    container_number: u64,
    container_birth: u64,
    records: Vec<TombstoneRecordSummary>,
    origin: PlannedUnit,
}

/// 一次发布里不依赖落点的那一半：每棵树这次之后的内容、哪些单元重写、哪些照抄，与换下的上一版单元（分配记录树、记账树、
/// 映射树、树表那几样由 [`assemble_publish`] 补）。
#[derive(Clone, Debug)]
struct LogicalPlan {
    txg: u64,
    instance: u64,
    transaction_base: u64,
    highest_transaction_after: u64,
    next_inode_after: u64,
    file_object_birth: u64,
    data_units: Vec<PlannedUnit>,
    instance_table_rewritten: bool,
    extent_lower: Vec<((u8, u64), PlannedUnit)>,
    extent_root: PlannedUnit,
    inode_containers: Vec<PlannedInodeContainer>,
    inode_root: PlannedUnit,
    deleted_queue: Vec<u64>,
    deleted_root: Option<PlannedUnit>,
    tombstone_containers: Vec<PlannedTombstoneContainer>,
    tombstone_root: Option<PlannedUnit>,
    tombstones_written: Vec<TombstoneRecordSummary>,
    accounting_keys: BTreeSet<KeyFields>,
    released_units: Vec<UnitId>,
}

/// extent 树下段层级 L 上一个节点罩几个单元：144 × 147^L（`extent_tree.rs:84-90`）。
fn extent_lower_span_in_data_units(level: u8) -> u64 {
    (0..level).fold(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, |span, _| {
        span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
    })
}

/// 下段的全部节点，先叶后根、同层按位置升序；只有一个单元的文件不建下段（`extent_tree.rs:210-226`）。
fn extent_lower_nodes_of_a_file(data_units: u64) -> Vec<(u8, u64)> {
    if data_units <= 1 {
        return Vec::new();
    }
    let root_level = (0..=u8::MAX)
        .find(|level| extent_lower_span_in_data_units(*level) >= data_units)
        .expect("罩得住");
    let mut nodes = Vec::new();
    for level in 0..=root_level {
        let nodes_in_this_level = data_units.div_ceil(extent_lower_span_in_data_units(level));
        for index in 0..nodes_in_this_level {
            nodes.push((level, index));
        }
    }
    nodes
}

fn extent_lower_node_last_data_unit(position: (u8, u64)) -> u64 {
    let span = extent_lower_span_in_data_units(position.0);
    position.1.saturating_mul(span).saturating_add(span - 1)
}

/// 记账 15 行的 key (统计量, 树, 盘, 代)：池级 3 行、每盘 6 行，代 = 这次的 txg（S3；统计量标签模型自编，C6）。
fn accounting_keys_of(txg: u64) -> BTreeSet<KeyFields> {
    let mut keys = BTreeSet::new();
    for row in 0..ACCOUNTING_POOL_WIDE_ROWS {
        keys.insert([1 + row, TREE_IDENTIFIER_INODE, STATISTIC_WITHOUT_DEVICE, txg, 0]);
    }
    for device in 0..u64::from(DEVICE_COUNT) {
        for row in 0..ACCOUNTING_PER_DEVICE_ROWS {
            keys.insert([1 + ACCOUNTING_POOL_WIDE_ROWS + row, 0, device, txg, 0]);
        }
    }
    keys
}

fn release_if_carried(planned: PlannedUnit, released_units: &mut Vec<UnitId>) {
    match planned {
        PlannedUnit::Carried(unit) => released_units.push(unit),
        PlannedUnit::Rewritten => {}
    }
}

fn mark_inode_root_rewritten(plan: &mut LogicalPlan) {
    release_if_carried(plan.inode_root, &mut plan.released_units);
    plan.inode_root = PlannedUnit::Rewritten;
}

fn mark_inode_container_rewritten(plan: &mut LogicalPlan, position: usize) {
    let origin = plan.inode_containers[position].origin;
    release_if_carried(origin, &mut plan.released_units);
    plan.inode_containers[position].origin = PlannedUnit::Rewritten;
}

/// 往 inode 树写记录（S5，`inode_tree.rs:181-272`）：已有的就地改，新号追加到最右有空位的那片，满了末尾新开一片。
fn plan_inode_record_writes(plan: &mut LogicalPlan, records_to_write: &[InodeRecordSummary]) {
    let mut rewritten_positions: BTreeSet<usize> = BTreeSet::new();
    for record in records_to_write {
        let holding = plan
            .inode_containers
            .iter()
            .position(|container| container.records.iter().any(|existing| existing.inode == record.inode));
        match holding {
            Some(position) => {
                let container = &mut plan.inode_containers[position];
                let at = container
                    .records
                    .iter()
                    .position(|existing| existing.inode == record.inode)
                    .expect("刚找到这条记录所在的容器");
                container.records[at] = *record;
                rewritten_positions.insert(position);
            }
            None => {
                if let Some(largest) = plan
                    .inode_containers
                    .last()
                    .and_then(|container| container.records.last())
                    .map(|existing| existing.inode)
                {
                    assert!(record.inode > largest, "新号大于树里每一个 key：插入落在最右的叶");
                }
                let rightmost_has_room = plan
                    .inode_containers
                    .last()
                    .is_some_and(|container| length_as_u64(container.records.len()) < INODE_LEAF_RECORDS);
                if rightmost_has_room {
                    let rightmost = plan.inode_containers.len() - 1;
                    plan.inode_containers[rightmost].records.push(*record);
                    rewritten_positions.insert(rightmost);
                } else {
                    plan.inode_containers.push(PlannedInodeContainer {
                        container_number: record.inode,
                        container_birth: plan.txg,
                        records: vec![*record],
                        origin: PlannedUnit::Rewritten,
                    });
                    rewritten_positions.insert(plan.inode_containers.len() - 1);
                }
            }
        }
    }
    assert!(
        length_as_u64(plan.inode_containers.len()) <= ROOT_NODE_CONTAINER_CAPACITY,
        "inode 树的叶容器不多于一个根装得下的 135 片"
    );
    for position in &rewritten_positions {
        mark_inode_container_rewritten(plan, *position);
    }
    if !rewritten_positions.is_empty() {
        mark_inode_root_rewritten(plan);
    }
}

/// 删一条 inode 记录（S5，`inode_tree.rs:286-317`）：删空的那片不落盘、右边各片左移、全部重写。
fn plan_inode_record_removal(plan: &mut LogicalPlan, inode: u64) {
    let position = plan
        .inode_containers
        .iter()
        .position(|container| container.records.iter().any(|existing| existing.inode == inode))
        .expect("删的记录在树里");
    plan.inode_containers[position]
        .records
        .retain(|existing| existing.inode != inode);
    if plan.inode_containers[position].records.is_empty() {
        let removed = plan.inode_containers.remove(position);
        release_if_carried(removed.origin, &mut plan.released_units);
        for later in position..plan.inode_containers.len() {
            mark_inode_container_rewritten(plan, later);
        }
    } else {
        mark_inode_container_rewritten(plan, position);
    }
    mark_inode_root_rewritten(plan);
}

/// 写文件内容（覆盖写、顺序写、新池新建文件）：整份换成 `data_units` 个新单元，上一版的单元一并换下（M11）；extent 下段整段重写、
/// 上段根重写（`transaction.rs:4723-4805`）。
fn plan_file_rewrite(plan: &mut LogicalPlan, previous: &Version, data_units: u64) {
    plan.released_units.extend(previous.file_units.iter().copied());
    plan.data_units = vec![PlannedUnit::Rewritten; u64_as_index(data_units)];
    plan.released_units.extend(previous.extent_lower.values().copied());
    plan.extent_lower = extent_lower_nodes_of_a_file(data_units)
        .into_iter()
        .map(|position| (position, PlannedUnit::Rewritten))
        .collect();
    if let Some(root) = previous.extent_root {
        plan.released_units.push(root);
    }
    plan.extent_root = PlannedUnit::Rewritten;
    plan.highest_transaction_after = plan.transaction_base + data_units - 1;
}

/// 删除的一批（`unlink.rs:166-235`）：留下前 `data_units_kept` 个单元，整落在留下段里的下段节点照抄，别的重写，上段根重写。
fn plan_deletion_batch(plan: &mut LogicalPlan, previous: &Version, data_units_kept: u64) {
    let kept = u64_as_index(data_units_kept);
    assert!(kept >= 1 && kept < previous.file_units.len(), "一批留下至少一个、至少截掉一个");
    plan.released_units.extend(previous.file_units[kept..].iter().copied());
    plan.data_units = previous.file_units[..kept]
        .iter()
        .map(|unit| PlannedUnit::Carried(*unit))
        .collect();
    let mut carried_positions = BTreeSet::new();
    let mut lower = Vec::new();
    for position in extent_lower_nodes_of_a_file(data_units_kept) {
        if extent_lower_node_last_data_unit(position) < data_units_kept {
            let unit = *previous
                .extent_lower
                .get(&position)
                .expect("罩的单元全在留下那一段里的节点，上一版同一个位置上必有");
            lower.push((position, PlannedUnit::Carried(unit)));
            carried_positions.insert(position);
        } else {
            lower.push((position, PlannedUnit::Rewritten));
        }
    }
    for (position, unit) in &previous.extent_lower {
        if !carried_positions.contains(position) {
            plan.released_units.push(*unit);
        }
    }
    plan.extent_lower = lower;
    release_if_carried(plan.extent_root, &mut plan.released_units);
    plan.extent_root = PlannedUnit::Rewritten;
}

/// 删 inode 1 的最后一批（`unlink.rs:243-264`）：剩下的数据单元全换下，下段整段换下，上段根重写成零条目。
fn plan_final_batch_of_the_first_file(plan: &mut LogicalPlan, previous: &Version) {
    plan.released_units.extend(previous.file_units.iter().copied());
    plan.data_units = Vec::new();
    plan.released_units.extend(previous.extent_lower.values().copied());
    plan.extent_lower = Vec::new();
    release_if_carried(plan.extent_root, &mut plan.released_units);
    plan.extent_root = PlannedUnit::Rewritten;
}

/// 意图那次往墓碑树最右的容器追加一条（S7，`container_tree.rs:423-473`）：满 815 开新容器，号 = 最大号 + 1；根重写。
fn plan_tombstone_append(plan: &mut LogicalPlan, record: TombstoneRecordSummary) {
    let rightmost_has_room = plan
        .tombstone_containers
        .last()
        .is_some_and(|container| length_as_u64(container.records.len()) < TOMBSTONE_CONTAINER_RECORDS);
    if rightmost_has_room {
        let rightmost = plan.tombstone_containers.len() - 1;
        let origin = plan.tombstone_containers[rightmost].origin;
        release_if_carried(origin, &mut plan.released_units);
        plan.tombstone_containers[rightmost].records.push(record);
        plan.tombstone_containers[rightmost].origin = PlannedUnit::Rewritten;
    } else {
        let container_number = plan
            .tombstone_containers
            .last()
            .map_or(1, |rightmost| rightmost.container_number + 1);
        plan.tombstone_containers.push(PlannedTombstoneContainer {
            container_number,
            container_birth: plan.txg,
            records: vec![record],
            origin: PlannedUnit::Rewritten,
        });
    }
    assert!(
        length_as_u64(plan.tombstone_containers.len()) <= ROOT_NODE_CONTAINER_CAPACITY,
        "墓碑树的容器不多于一个根装得下的：第一段的历史里根不长高"
    );
    if let Some(root) = plan.tombstone_root {
        release_if_carried(root, &mut plan.released_units);
    }
    plan.tombstone_root = Some(PlannedUnit::Rewritten);
    plan.tombstones_written.push(record);
}

/// 这次发布不依赖落点的那一半（M5、S5–S13）。
fn plan_logical(
    previous: &Version,
    operation: PublishOperation,
    txg: u64,
    write_path: WritePath,
    file_object_birth: u64,
) -> LogicalPlan {
    let instance = if txg == NEW_POOL_FILE_CREATION_TXG {
        MAKE_FILESYSTEM_PROCESS_INSTANCE
    } else {
        MOUNT_INSTANCE
    };
    assert!(previous.instance <= instance, "实例代号只增：mkfs 那个进程取 1，可写挂载取 2");
    let transaction_base = if operation == PublishOperation::RowPublish {
        1
    } else {
        previous.highest_transaction + 1
    };
    let mut plan = LogicalPlan {
        txg,
        instance,
        transaction_base,
        highest_transaction_after: transaction_base,
        next_inode_after: previous.next_inode,
        file_object_birth,
        data_units: previous.file_units.iter().map(|unit| PlannedUnit::Carried(*unit)).collect(),
        instance_table_rewritten: false,
        extent_lower: previous
            .extent_lower
            .iter()
            .map(|(position, unit)| (*position, PlannedUnit::Carried(*unit)))
            .collect(),
        extent_root: previous.extent_root.map_or(PlannedUnit::Rewritten, PlannedUnit::Carried),
        inode_containers: previous
            .inode_containers
            .iter()
            .map(|container| PlannedInodeContainer {
                container_number: container.container_number,
                container_birth: container.container_birth,
                records: container.records.clone(),
                origin: PlannedUnit::Carried(container.unit),
            })
            .collect(),
        inode_root: previous.inode_root.map_or(PlannedUnit::Rewritten, PlannedUnit::Carried),
        deleted_queue: previous.deleted_queue.clone(),
        deleted_root: previous.deleted_root.map(PlannedUnit::Carried),
        tombstone_containers: previous
            .tombstone_containers
            .iter()
            .map(|container| PlannedTombstoneContainer {
                container_number: container.container_number,
                container_birth: container.container_birth,
                records: container.records.clone(),
                origin: PlannedUnit::Carried(container.unit),
            })
            .collect(),
        tombstone_root: previous.tombstone_root.map(PlannedUnit::Carried),
        tombstones_written: Vec::new(),
        accounting_keys: accounting_keys_of(txg),
        released_units: Vec::new(),
    };
    let first_file_record = || {
        previous
            .live_record_of(FIRST_FILE_INODE)
            .expect("写文件内容的发布接在 inode 1 活着的一版之后")
    };
    match operation {
        PublishOperation::FirstFile => {
            plan_file_rewrite(&mut plan, previous, 1);
            plan_inode_record_writes(
                &mut plan,
                &[InodeRecordSummary {
                    inode: FIRST_FILE_INODE,
                    object_birth: txg,
                    nlink: 1,
                }],
            );
            plan.next_inode_after = FIRST_FILE_INODE + 1;
        }
        PublishOperation::RowPublish => {
            plan.instance_table_rewritten = true;
            plan.released_units.push(previous.instance_table);
        }
        PublishOperation::WarmUp => {}
        PublishOperation::Overwrite => {
            let record = first_file_record();
            plan_file_rewrite(&mut plan, previous, 1);
            plan_inode_record_writes(&mut plan, &[record]);
        }
        PublishOperation::SequentialWrite { data_units } => {
            let record = first_file_record();
            plan_file_rewrite(&mut plan, previous, data_units);
            plan_inode_record_writes(&mut plan, &[record]);
        }
        PublishOperation::NewInodes { count } => {
            let records: Vec<InodeRecordSummary> = (0..count)
                .map(|offset| InodeRecordSummary {
                    inode: previous.next_inode + offset,
                    object_birth: txg,
                    nlink: 1,
                })
                .collect();
            plan_inode_record_writes(&mut plan, &records);
            plan.next_inode_after = previous.next_inode + count;
        }
        PublishOperation::UnlinkIntent { inode } => {
            let record = previous.live_record_of(inode).expect("意图删的 inode 是一条活记录");
            assert!(record.nlink > 0, "意图删的 inode 还活着");
            plan_inode_record_writes(
                &mut plan,
                &[InodeRecordSummary {
                    inode,
                    object_birth: record.object_birth,
                    nlink: 0,
                }],
            );
            assert!(plan.deleted_queue.is_empty(), "发意图之前队列已走空");
            plan.deleted_queue.push(inode);
            if let Some(root) = plan.deleted_root {
                release_if_carried(root, &mut plan.released_units);
            }
            plan.deleted_root = Some(PlannedUnit::Rewritten);
            if write_path == WritePath::WithTombstones {
                plan_tombstone_append(
                    &mut plan,
                    TombstoneRecordSummary {
                        object: inode,
                        object_birth: record.object_birth,
                        death_txg: txg,
                    },
                );
            }
        }
        PublishOperation::UnlinkBatch { data_units_kept } => {
            let record = previous.live_record_of(FIRST_FILE_INODE).expect("分批删的是 inode 1");
            plan_deletion_batch(&mut plan, previous, data_units_kept);
            plan_inode_record_writes(&mut plan, &[record]);
        }
        PublishOperation::UnlinkFinal { inode } => {
            plan_inode_record_removal(&mut plan, inode);
            if inode == FIRST_FILE_INODE {
                plan_final_batch_of_the_first_file(&mut plan, previous);
            }
            plan.deleted_queue.retain(|queued| *queued != inode);
            assert!(plan.deleted_queue.is_empty(), "任何一版至多一条 key");
            if let Some(root) = plan.deleted_root {
                release_if_carried(root, &mut plan.released_units);
            }
            plan.deleted_root = None;
        }
    }
    plan
}

/// 这次发布重写的一个角色（M7 的次序就是 `assemble_publish` 排出来的次序）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Role {
    Data(usize),
    InstanceTable,
    ExtentLower((u8, u64)),
    ExtentRoot,
    InodeContainer(usize),
    InodeRoot,
    AllocationNode(AllocationNode),
    AccountingNode(CodeTwoNodePosition),
    DeletedInodesRoot,
    TombstoneContainer(usize),
    TombstoneRoot,
    MappingNode(CodeTwoNodePosition),
    TreeTable,
}

impl Role {
    /// 用户数据按政策函数；码 3 容器两槽对齐；码 2 节点一槽（`transaction.rs` `TransactionUnit::placement`）。
    const fn placement_rule(self) -> PlacementRule {
        match self {
            Role::Data(_) => PlacementRule::UserData,
            Role::InstanceTable | Role::InodeContainer(_) | Role::TombstoneContainer(_) => {
                PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned)
            }
            Role::ExtentLower(_)
            | Role::ExtentRoot
            | Role::InodeRoot
            | Role::AllocationNode(_)
            | Role::AccountingNode(_)
            | Role::DeletedInodesRoot
            | Role::TombstoneRoot
            | Role::MappingNode(_)
            | Role::TreeTable => PlacementRule::CommitGenerated(UnitFootprint::OneSlot),
        }
    }
}

/// 一次发布排好的样子：角色按 bump 次序、三棵靠落点的树（分配记录、记账、映射）这次之后的样子、要释放的上一版单元。
#[derive(Clone, Debug)]
struct AssembledPublish {
    roles: Vec<Role>,
    allocation_nodes: Vec<(AllocationNode, PlannedUnit)>,
    accounting_plan: CodeTwoPlan,
    mapping_plan: CodeTwoPlan,
    mapping_keys_of_rewritten: HashMap<Role, KeyFields>,
    releases: Vec<UnitId>,
}

fn node_mapping_key(plan: &LogicalPlan, unit_class: u64, tree: u64, birth_sequence: u64) -> KeyFields {
    [unit_class, tree, plan.txg, plan.instance, birth_sequence]
}

fn mapping_key_of_carried(disk: &Disk, unit: UnitId) -> KeyFields {
    disk.units[unit]
        .mapping_key
        .expect("照抄进来的单元在上一版里进映射、有一把 key")
}

/// 一个单元进映射：重写的按树内 bump 次序发出生序号（M9），照抄的沿用上一版的 key。
fn note_mapped_unit(
    role: Role,
    planned: PlannedUnit,
    rewritten_key: KeyFields,
    disk: &Disk,
    mapping_keys: &mut BTreeSet<KeyFields>,
    mapping_keys_of_rewritten: &mut HashMap<Role, KeyFields>,
) {
    let key = match planned {
        PlannedUnit::Rewritten => {
            mapping_keys_of_rewritten.insert(role, rewritten_key);
            rewritten_key
        }
        PlannedUnit::Carried(unit) => mapping_key_of_carried(disk, unit),
    };
    assert!(mapping_keys.insert(key), "进映射的单元各一把 key，互不相撞");
}

fn code_two_origin_as_planned(origin: CodeTwoOrigin, previous: &CodeTwoTreeVersion) -> PlannedUnit {
    match origin {
        CodeTwoOrigin::CarriedFrom(previous_position) => PlannedUnit::Carried(previous.unit_at(previous_position)),
        CodeTwoOrigin::Rewritten => PlannedUnit::Rewritten,
    }
}

/// 把逻辑计划与这一轮的分配记录树节点排成一次发布（`transaction.rs:5235-5344`、`:5390-5649`）。
fn assemble_publish(
    plan: &LogicalPlan,
    previous: &Version,
    disk: &Disk,
    allocation_nodes_after: &BTreeSet<AllocationNode>,
    allocation_rewritten: &BTreeSet<AllocationNode>,
) -> AssembledPublish {
    let allocation_nodes: Vec<(AllocationNode, PlannedUnit)> = allocation_nodes_after
        .iter()
        .map(|node| {
            let origin = if allocation_rewritten.contains(node) {
                PlannedUnit::Rewritten
            } else {
                match previous.allocation_nodes.get(node) {
                    Some(unit) => PlannedUnit::Carried(*unit),
                    None => PlannedUnit::Rewritten,
                }
            };
            (*node, origin)
        })
        .collect();
    let mut releases = plan.released_units.clone();
    for (node, unit) in &previous.allocation_nodes {
        if allocation_rewritten.contains(node) || !allocation_nodes_after.contains(node) {
            releases.push(*unit);
        }
    }
    let accounting_plan = plan_code_two_tree(&previous.accounting.shape, &plan.accounting_keys, accounting_tree_capacity());
    for position in &accounting_plan.replaced_previous {
        releases.push(previous.accounting.unit_at(*position));
    }

    let mut mapping_keys = BTreeSet::new();
    let mut mapping_keys_of_rewritten = HashMap::new();
    for (position, planned) in plan.data_units.iter().enumerate() {
        let rewritten_key = [
            UNIT_CLASS_DATA,
            TREE_IDENTIFIER_EXTENT,
            plan.txg,
            plan.instance,
            plan.transaction_base + length_as_u64(position),
        ];
        note_mapped_unit(Role::Data(position), *planned, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    let mut extent_sequence = 0;
    let extent_nodes = plan
        .extent_lower
        .iter()
        .map(|(position, planned)| (Role::ExtentLower(*position), *planned))
        .chain(std::iter::once((Role::ExtentRoot, plan.extent_root)));
    for (role, planned) in extent_nodes {
        let rewritten_key = node_mapping_key(plan, UNIT_CLASS_INDEX_NODE, TREE_IDENTIFIER_EXTENT, extent_sequence);
        if planned == PlannedUnit::Rewritten {
            extent_sequence += 1;
        }
        note_mapped_unit(role, planned, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    let mut inode_sequence = 0;
    for (position, container) in plan.inode_containers.iter().enumerate() {
        let rewritten_key = node_mapping_key(plan, UNIT_CLASS_PACKED, TREE_IDENTIFIER_INODE, inode_sequence);
        if container.origin == PlannedUnit::Rewritten {
            inode_sequence += 1;
        }
        note_mapped_unit(Role::InodeContainer(position), container.origin, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    let inode_root_key = node_mapping_key(plan, UNIT_CLASS_INDEX_NODE, TREE_IDENTIFIER_INODE, inode_sequence);
    note_mapped_unit(Role::InodeRoot, plan.inode_root, inode_root_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    let mut allocation_sequence = 0;
    for (node, planned) in &allocation_nodes {
        let rewritten_key = node_mapping_key(plan, UNIT_CLASS_INDEX_NODE, TREE_IDENTIFIER_ALLOCATION_RECORDS, allocation_sequence);
        if *planned == PlannedUnit::Rewritten {
            allocation_sequence += 1;
        }
        note_mapped_unit(Role::AllocationNode(*node), *planned, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    let mut accounting_sequence = 0;
    for ((position, _), origin) in accounting_plan.shape.nodes.iter().zip(&accounting_plan.origins) {
        let planned = code_two_origin_as_planned(*origin, &previous.accounting);
        let rewritten_key = node_mapping_key(plan, UNIT_CLASS_INDEX_NODE, TREE_IDENTIFIER_ACCOUNTING, accounting_sequence);
        if planned == PlannedUnit::Rewritten {
            accounting_sequence += 1;
        }
        note_mapped_unit(Role::AccountingNode(*position), planned, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    if let Some(planned) = plan.deleted_root {
        let rewritten_key = node_mapping_key(plan, UNIT_CLASS_INDEX_NODE, TREE_IDENTIFIER_DELETED_INODES, 0);
        note_mapped_unit(Role::DeletedInodesRoot, planned, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    let mut tombstone_sequence = 0;
    for (position, container) in plan.tombstone_containers.iter().enumerate() {
        let rewritten_key = node_mapping_key(plan, UNIT_CLASS_PACKED, TREE_IDENTIFIER_TOMBSTONE, tombstone_sequence);
        if container.origin == PlannedUnit::Rewritten {
            tombstone_sequence += 1;
        }
        note_mapped_unit(Role::TombstoneContainer(position), container.origin, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    if let Some(planned) = plan.tombstone_root {
        let rewritten_key = node_mapping_key(plan, UNIT_CLASS_INDEX_NODE, TREE_IDENTIFIER_TOMBSTONE, tombstone_sequence);
        note_mapped_unit(Role::TombstoneRoot, planned, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    let mapping_plan = plan_code_two_tree(&previous.mapping.shape, &mapping_keys, mapping_tree_capacity());
    for position in &mapping_plan.replaced_previous {
        releases.push(previous.mapping.unit_at(*position));
    }
    releases.push(previous.tree_table);

    let mut roles = Vec::new();
    for (position, planned) in plan.data_units.iter().enumerate() {
        if *planned == PlannedUnit::Rewritten {
            roles.push(Role::Data(position));
        }
    }
    if plan.instance_table_rewritten {
        roles.push(Role::InstanceTable);
    }
    for (position, planned) in &plan.extent_lower {
        if *planned == PlannedUnit::Rewritten {
            roles.push(Role::ExtentLower(*position));
        }
    }
    if plan.extent_root == PlannedUnit::Rewritten {
        roles.push(Role::ExtentRoot);
    }
    for (position, container) in plan.inode_containers.iter().enumerate() {
        if container.origin == PlannedUnit::Rewritten {
            roles.push(Role::InodeContainer(position));
        }
    }
    if plan.inode_root == PlannedUnit::Rewritten {
        roles.push(Role::InodeRoot);
    }
    for (node, planned) in &allocation_nodes {
        if *planned == PlannedUnit::Rewritten {
            roles.push(Role::AllocationNode(*node));
        }
    }
    for ((position, _), origin) in accounting_plan.shape.nodes.iter().zip(&accounting_plan.origins) {
        if *origin == CodeTwoOrigin::Rewritten {
            roles.push(Role::AccountingNode(*position));
        }
    }
    if plan.deleted_root == Some(PlannedUnit::Rewritten) {
        roles.push(Role::DeletedInodesRoot);
    }
    for (position, container) in plan.tombstone_containers.iter().enumerate() {
        if container.origin == PlannedUnit::Rewritten {
            roles.push(Role::TombstoneContainer(position));
        }
    }
    if plan.tombstone_root == Some(PlannedUnit::Rewritten) {
        roles.push(Role::TombstoneRoot);
    }
    for ((position, _), origin) in mapping_plan.shape.nodes.iter().zip(&mapping_plan.origins) {
        if *origin == CodeTwoOrigin::Rewritten {
            roles.push(Role::MappingNode(*position));
        }
    }
    roles.push(Role::TreeTable);
    AssembledPublish {
        roles,
        allocation_nodes,
        accounting_plan,
        mapping_plan,
        mapping_keys_of_rewritten,
        releases,
    }
}

/// 分配记录树重写哪几个节点走固定点（M8，`transaction.rs:6627-6704`）：每一轮在分配器拷贝上先释放、再按这一轮的角色取落点，
/// 内容变了的叶与装着记录的节点集对得上才定下。
fn settle_publish(
    plan: &LogicalPlan,
    previous: &Version,
    disk: &Disk,
    allocator: &Allocator,
) -> Result<AssembledPublish, PlacementRefused> {
    let previous_nodes: BTreeSet<AllocationNode> = previous.allocation_nodes.keys().copied().collect();
    let mut rewritten: BTreeSet<AllocationNode> = BTreeSet::new();
    let mut nodes_after = previous_nodes.clone();
    for _round in 0..64 {
        let rewritten_among_the_nodes_after: BTreeSet<AllocationNode> =
            rewritten.intersection(&nodes_after).copied().collect();
        let assembled = assemble_publish(plan, previous, disk, &nodes_after, &rewritten_among_the_nodes_after);
        let rewritten_by_the_plan: BTreeSet<AllocationNode> = assembled
            .allocation_nodes
            .iter()
            .filter(|(_, planned)| *planned == PlannedUnit::Rewritten)
            .map(|(node, _)| *node)
            .collect();
        let mut rehearsal = allocator.rehearsal_copy();
        for unit in &assembled.releases {
            let released = &disk.units[*unit];
            rehearsal.release(released.slot, released.span_slots, plan.txg);
        }
        let changed_by_the_release = allocation_nodes_whose_contents_changed(allocator, &rehearsal);
        if !changed_by_the_release.is_subset(&rewritten_by_the_plan) {
            rewritten.extend(changed_by_the_release);
            nodes_after.extend(rehearsal.nodes_holding_records());
            continue;
        }
        for role in &assembled.roles {
            rehearsal.allocate(role.placement_rule(), plan.txg, PLANTED_UNIT)?;
        }
        let changed = allocation_nodes_whose_contents_changed(allocator, &rehearsal);
        let holding_records = rehearsal.nodes_holding_records();
        if holding_records == nodes_after && changed.is_subset(&rewritten_by_the_plan) {
            return Ok(assembled);
        }
        rewritten.extend(changed);
        nodes_after = holding_records;
    }
    panic!("分配记录树的重写集 64 轮没定下来：产品路径上三四轮就定（M8）");
}

// ---------------------------------------------------------------------------------------------
// R-U：读每个读得出单元里的每条 key（增量计数；最后一次扫描与阳性对照另用整扫一遍的参照路径）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
struct KeyCounter {
    /// 这条臂下算数的、读得出单元里的出现次数。
    readable: u64,
    /// 其中供出它的单元在环窗内（释放代 s 满足 t − s ≤ 23，Q14）的出现次数。
    in_window: u64,
    in_truth: bool,
}

impl KeyCounter {
    fn contribution(self) -> (u64, u64, u64) {
        (
            u64::from(self.readable > 0 && !self.in_truth),
            u64::from(self.readable == 0 && self.in_truth),
            u64::from(self.in_window > 0 && !self.in_truth),
        )
    }
}

/// 臂 B 的墓碑效力按出现的身份判（登记 5.3）：inode 记录对 (对象, 出生代)；extent 记录对 inode 与数据指针出生 txg ∈ [b, t_d]；
/// `deleted_inodes` 对 inode。
fn tombstone_suppresses(tombstones_of_the_object: &[TombstoneRecordSummary], tag: OccurrenceTag) -> bool {
    match tag {
        OccurrenceTag::InodeRecord { object_birth, .. } => tombstones_of_the_object
            .iter()
            .any(|tombstone| tombstone.object_birth == object_birth),
        OccurrenceTag::ExtentRecord { pointer_birth, .. } => tombstones_of_the_object
            .iter()
            .any(|tombstone| tombstone.object_birth <= pointer_birth && pointer_birth <= tombstone.death_txg),
        OccurrenceTag::DeletedInode { .. } => !tombstones_of_the_object.is_empty(),
        OccurrenceTag::Untagged => false,
    }
}

fn object_of_tag(tag: OccurrenceTag) -> Option<u64> {
    match tag {
        OccurrenceTag::InodeRecord { inode, .. }
        | OccurrenceTag::ExtentRecord { inode, .. }
        | OccurrenceTag::DeletedInode { inode } => Some(inode),
        OccurrenceTag::Untagged => None,
    }
}

/// 臂 P：一条 key 最晚一条死亡记录的 t ≥ 供出它的单元版本的诞生代号就不收（登记 5.3）。
fn death_record_suppresses(death_time: &HashMap<NamespacedKey, u64>, key: &NamespacedKey, provider_birth: u64) -> bool {
    death_time.get(key).is_some_and(|death| *death >= provider_birth)
}

#[derive(Clone, Debug)]
struct UnitScanReading {
    arm: Arm,
    tracks_the_ring_window: bool,
    counters: HashMap<NamespacedKey, KeyCounter>,
    resurrected: [u64; NAMESPACE_COUNT],
    missed: [u64; NAMESPACE_COUNT],
    resurrected_in_window: [u64; NAMESPACE_COUNT],
    tombstones_by_object: HashMap<u64, Vec<TombstoneRecordSummary>>,
    tagged_units_by_object: HashMap<u64, Vec<UnitId>>,
    window_units: BTreeMap<UnitId, u64>,
}

impl UnitScanReading {
    fn new(arm: Arm, tracks_the_ring_window: bool) -> Self {
        Self {
            arm,
            tracks_the_ring_window,
            counters: HashMap::new(),
            resurrected: [0; NAMESPACE_COUNT],
            missed: [0; NAMESPACE_COUNT],
            resurrected_in_window: [0; NAMESPACE_COUNT],
            tombstones_by_object: HashMap::new(),
            tagged_units_by_object: HashMap::new(),
            window_units: BTreeMap::new(),
        }
    }

    fn counts(&self, occurrence: &Occurrence, provider_birth: u64, death_time: &HashMap<NamespacedKey, u64>) -> bool {
        match self.arm {
            Arm::WithoutTombstones => true,
            Arm::TodaysTombstones => match object_of_tag(occurrence.tag) {
                Some(object) => !self
                    .tombstones_by_object
                    .get(&object)
                    .is_some_and(|tombstones| tombstone_suppresses(tombstones, occurrence.tag)),
                None => true,
            },
            Arm::DeathRecordForEveryKey => !death_record_suppresses(death_time, &occurrence.key, provider_birth),
        }
    }

    fn change(&mut self, key: NamespacedKey, update: impl FnOnce(&mut KeyCounter)) {
        let counter = self.counters.entry(key).or_default();
        let before = counter.contribution();
        update(counter);
        let after = counter.contribution();
        let position = key.namespace.position();
        self.resurrected[position] = self.resurrected[position] + after.0 - before.0;
        self.missed[position] = self.missed[position] + after.1 - before.1;
        self.resurrected_in_window[position] = self.resurrected_in_window[position] + after.2 - before.2;
    }

    fn set_truth(&mut self, key: NamespacedKey, in_truth: bool) {
        self.change(key, |counter| counter.in_truth = in_truth);
    }

    /// 一个单元写下、读得出了。
    fn on_unit_readable(&mut self, unit_id: UnitId, unit: &Unit, death_time: &HashMap<NamespacedKey, u64>) {
        for occurrence in occurrences_of(&unit.contents) {
            if let Some(object) = object_of_tag(occurrence.tag) {
                let units = self.tagged_units_by_object.entry(object).or_default();
                if units.last() != Some(&unit_id) {
                    units.push(unit_id);
                }
            }
            if self.counts(&occurrence, unit.birth_txg(), death_time) {
                self.change(occurrence.key, |counter| counter.readable += 1);
            }
        }
    }

    /// 一个单元被之后的写盖过、读不出了。
    fn on_unit_unreadable(&mut self, unit_id: UnitId, unit: &Unit, death_time: &HashMap<NamespacedKey, u64>) {
        let was_in_window = self.window_units.remove(&unit_id).is_some();
        for occurrence in occurrences_of(&unit.contents) {
            if self.counts(&occurrence, unit.birth_txg(), death_time) {
                self.change(occurrence.key, |counter| {
                    counter.readable -= 1;
                    if was_in_window {
                        counter.in_window -= 1;
                    }
                });
            }
        }
    }

    /// 一个读得出的单元这次被释放：进环窗（只有 G-环 报 Q14）。
    fn on_unit_released(&mut self, unit_id: UnitId, unit: &Unit, txg: u64, death_time: &HashMap<NamespacedKey, u64>) {
        if !self.tracks_the_ring_window || !unit.is_readable {
            return;
        }
        self.window_units.insert(unit_id, txg);
        for occurrence in occurrences_of(&unit.contents) {
            if self.counts(&occurrence, unit.birth_txg(), death_time) {
                self.change(occurrence.key, |counter| counter.in_window += 1);
            }
        }
    }

    /// 扫描时刻 t：释放代 s 满足 t − s > 23 的单元出环窗。
    fn expire_the_window(&mut self, txg: u64, disk: &Disk, death_time: &HashMap<NamespacedKey, u64>) {
        let expired: Vec<UnitId> = self
            .window_units
            .iter()
            .filter(|(_, released_at)| txg - **released_at > RING_WINDOW_IN_PUBLISHES)
            .map(|(unit_id, _)| *unit_id)
            .collect();
        for unit_id in expired {
            self.window_units.remove(&unit_id);
            let unit = &disk.units[unit_id];
            for occurrence in occurrences_of(&unit.contents) {
                if self.counts(&occurrence, unit.birth_txg(), death_time) {
                    self.change(occurrence.key, |counter| counter.in_window -= 1);
                }
            }
        }
    }

    /// 臂 P：这条 key 这次死了，此刻盘上它的每次出现都早于死亡记录，一律不收。
    fn on_key_death(&mut self, key: NamespacedKey) {
        self.change(key, |counter| {
            counter.readable = 0;
            counter.in_window = 0;
        });
    }

    /// 臂 B：这次写下一条墓碑，此刻读得出单元里被它指认的出现一律不收（墓碑容器只增，写下就一直读得出）。
    fn on_tombstone(&mut self, tombstone: TombstoneRecordSummary, disk: &Disk) {
        let units = self
            .tagged_units_by_object
            .get(&tombstone.object)
            .cloned()
            .unwrap_or_default();
        let single = [tombstone];
        for unit_id in units {
            let unit = &disk.units[unit_id];
            if !unit.is_readable {
                continue;
            }
            let in_window = self.window_units.contains_key(&unit_id);
            for occurrence in occurrences_of(&unit.contents) {
                if object_of_tag(occurrence.tag) != Some(tombstone.object) {
                    continue;
                }
                let already_suppressed = self
                    .tombstones_by_object
                    .get(&tombstone.object)
                    .is_some_and(|tombstones| tombstone_suppresses(tombstones, occurrence.tag));
                if !already_suppressed && tombstone_suppresses(&single, occurrence.tag) {
                    self.change(occurrence.key, |counter| {
                        counter.readable -= 1;
                        if in_window {
                            counter.in_window -= 1;
                        }
                    });
                }
            }
        }
        self.tombstones_by_object
            .entry(tombstone.object)
            .or_default()
            .push(tombstone);
    }
}

// ---------------------------------------------------------------------------------------------
// R-S、R-G：每次扫描从读得出的单元现算（登记 5.4）；R-U 的整扫参照路径
// ---------------------------------------------------------------------------------------------

/// 供出一条 key 的单元的摘要：P 臂判死亡记录、Q14 判环窗、PC2 手算都读它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Provider {
    newest_birth: u64,
    freshest_release: Option<u64>,
    has_live_provider: bool,
    provider_count: u64,
}

type KeysWithProviders = BTreeMap<KeyFields, Provider>;

fn note_provider(keys: &mut KeysWithProviders, fields: KeyFields, unit: &Unit) {
    let provider = keys.entry(fields).or_insert(Provider {
        newest_birth: 0,
        freshest_release: None,
        has_live_provider: false,
        provider_count: 0,
    });
    provider.newest_birth = provider.newest_birth.max(unit.birth_txg());
    provider.provider_count += 1;
    match unit.release_txg {
        Some(released_at) => {
            provider.freshest_release = Some(provider.freshest_release.map_or(released_at, |current| current.max(released_at)));
        }
        None => provider.has_live_provider = true,
    }
}

#[derive(Clone, Debug, Default)]
struct UnitLists {
    data_units: Vec<UnitId>,
    inode_containers: Vec<UnitId>,
    tombstone_containers: Vec<UnitId>,
    accounting_leaves: Vec<UnitId>,
}

fn unit_by_id<'reading>(disk: &'reading Disk, planted: Option<&'reading Unit>, unit_id: UnitId) -> &'reading Unit {
    if unit_id == PLANTED_UNIT {
        planted.expect("植入的单元")
    } else {
        &disk.units[unit_id]
    }
}

fn consider_for_selection(newest: &mut BTreeMap<SelectionGroup, (WriteOrder, UnitId)>, unit_id: UnitId, unit: &Unit) {
    if let Some(group) = unit.group {
        let candidate = (unit.write_order, unit_id);
        newest
            .entry(group)
            .and_modify(|current| {
                if candidate.0 > current.0 {
                    *current = candidate;
                }
            })
            .or_insert(candidate);
    }
}

/// 一次扫描里一种读法读出的东西：每个名字空间的 key 与供出者。
#[derive(Clone, Debug)]
struct ReadingResult {
    keys: Vec<KeysWithProviders>,
}

impl ReadingResult {
    fn empty() -> Self {
        Self {
            keys: vec![KeysWithProviders::new(); NAMESPACE_COUNT],
        }
    }

    fn note(&mut self, namespace: Namespace, fields: KeyFields, unit: &Unit) {
        note_provider(&mut self.keys[namespace.position()], fields, unit);
    }
}

#[derive(Clone, Copy, Debug)]
struct ScanInputs<'scan> {
    disk: &'scan Disk,
    unit_lists: &'scan UnitLists,
    death_time: &'scan HashMap<NamespacedKey, u64>,
    planted: Option<&'scan Unit>,
    erased: Option<UnitId>,
}

impl ScanInputs<'_> {
    fn is_candidate(&self, unit_id: UnitId) -> bool {
        unit_id != PLANTED_UNIT && self.disk.units[unit_id].is_readable && Some(unit_id) != self.erased
    }

    fn passes_death_records(&self, arm: Arm, namespace: Namespace, fields: KeyFields, provider_birth: u64) -> bool {
        arm != Arm::DeathRecordForEveryKey
            || !death_record_suppresses(self.death_time, &namespaced(namespace, fields), provider_birth)
    }
}

/// R-S 与 R-G（登记 5.4、5.3 臂的效力、C3、C11）。
fn select_current_reading(reader: Reader, arm: Arm, inputs: &ScanInputs<'_>) -> ReadingResult {
    let disk = inputs.disk;
    let mut newest: BTreeMap<SelectionGroup, (WriteOrder, UnitId)> = BTreeMap::new();
    let lists = inputs.unit_lists;
    for unit_id in lists
        .data_units
        .iter()
        .chain(&lists.inode_containers)
        .chain(&lists.tombstone_containers)
    {
        if inputs.is_candidate(*unit_id) {
            consider_for_selection(&mut newest, *unit_id, &disk.units[*unit_id]);
        }
    }
    if let Some(planted_unit) = inputs.planted {
        consider_for_selection(&mut newest, PLANTED_UNIT, planted_unit);
    }
    let mut reading = ReadingResult::empty();
    let mut tombstones_by_object: HashMap<u64, Vec<TombstoneRecordSummary>> = HashMap::new();
    let mut accepted_containers: Vec<UnitId> = Vec::new();
    for (group, (_, unit_id)) in &newest {
        if let SelectionGroup::PackedContainer { .. } = group {
            accepted_containers.push(*unit_id);
        }
        let unit = unit_by_id(disk, inputs.planted, *unit_id);
        if let UnitContents::TombstoneRecords(records) = &unit.contents {
            for record in records {
                tombstones_by_object.entry(record.object).or_default().push(*record);
                let fields = tombstone_key(*record);
                if inputs.passes_death_records(arm, Namespace::Tombstone, fields, unit.birth_txg()) {
                    reading.note(Namespace::Tombstone, fields, unit);
                }
            }
        }
    }
    let suppressed = |inode: u64, tag: OccurrenceTag| {
        arm == Arm::TodaysTombstones
            && tombstones_by_object
                .get(&inode)
                .is_some_and(|tombstones| tombstone_suppresses(tombstones, tag))
    };
    let mut inode_tree_objects: BTreeSet<(u64, u64)> = BTreeSet::new();
    for (_, unit_id) in newest.values() {
        let unit = unit_by_id(disk, inputs.planted, *unit_id);
        if let UnitContents::InodeRecords(records) = &unit.contents {
            for record in records {
                let tag = OccurrenceTag::InodeRecord {
                    inode: record.inode,
                    object_birth: record.object_birth,
                };
                if suppressed(record.inode, tag) {
                    continue;
                }
                let fields = single_field_key(record.inode);
                if inputs.passes_death_records(arm, Namespace::Inode, fields, unit.birth_txg()) {
                    reading.note(Namespace::Inode, fields, unit);
                    inode_tree_objects.insert((record.inode, record.object_birth));
                }
                if record.nlink == 0 && inputs.passes_death_records(arm, Namespace::DeletedInodes, fields, unit.birth_txg()) {
                    reading.note(Namespace::DeletedInodes, fields, unit);
                }
            }
        }
    }
    let mut accepted_data_units: Vec<UnitId> = Vec::new();
    for (group, (_, unit_id)) in &newest {
        if let SelectionGroup::DataUnit { object, object_birth, .. } = group {
            let tag = OccurrenceTag::InodeRecord {
                inode: *object,
                object_birth: *object_birth,
            };
            if suppressed(*object, tag) {
                continue;
            }
            if reader == Reader::SelectCurrentGatedByObject && !inode_tree_objects.contains(&(*object, *object_birth)) {
                continue;
            }
            accepted_data_units.push(*unit_id);
        }
    }
    for unit_id in &accepted_data_units {
        let unit = unit_by_id(disk, inputs.planted, *unit_id);
        if let UnitContents::DataUnit { object, anchor_unit, .. } = unit.contents {
            let fields = extent_key(object, anchor_unit);
            if inputs.passes_death_records(arm, Namespace::Extent, fields, unit.birth_txg()) {
                reading.note(Namespace::Extent, fields, unit);
            }
        }
    }
    for unit_id in accepted_data_units.iter().chain(&accepted_containers) {
        let unit = unit_by_id(disk, inputs.planted, *unit_id);
        let slot_fields = single_field_key(unit.slot);
        if inputs.passes_death_records(arm, Namespace::AllocationUnitsOfTheVersion, slot_fields, unit.birth_txg()) {
            reading.note(Namespace::AllocationUnitsOfTheVersion, slot_fields, unit);
        }
        if let Some(mapping_fields) = unit.mapping_key {
            if inputs.passes_death_records(arm, Namespace::CentralMappingUnitsOfTheVersion, mapping_fields, unit.birth_txg()) {
                reading.note(Namespace::CentralMappingUnitsOfTheVersion, mapping_fields, unit);
            }
        }
    }
    let mut rows: BTreeMap<[u64; 3], (u64, UnitId)> = BTreeMap::new();
    let mut accounting_candidates: Vec<UnitId> = Vec::new();
    if inputs.planted.is_some_and(|planted_unit| planted_unit.owner == UnitOwner::AccountingNode) {
        accounting_candidates.push(PLANTED_UNIT);
    }
    if let Some(newest_leaf) = lists
        .accounting_leaves
        .iter()
        .rev()
        .find(|unit_id| inputs.is_candidate(**unit_id))
    {
        accounting_candidates.push(*newest_leaf);
    }
    for unit_id in accounting_candidates {
        let unit = unit_by_id(disk, inputs.planted, unit_id);
        if let UnitContents::AccountingKeys(keys) = &unit.contents {
            for key in keys {
                let row = [key[0], key[1], key[2]];
                let candidate = (key[3], unit_id);
                rows.entry(row)
                    .and_modify(|current| {
                        if candidate.0 > current.0 {
                            *current = candidate;
                        }
                    })
                    .or_insert(candidate);
            }
        }
    }
    for (row, (generation, unit_id)) in rows {
        let unit = unit_by_id(disk, inputs.planted, unit_id);
        let fields = [row[0], row[1], row[2], generation, 0];
        if inputs.passes_death_records(arm, Namespace::Accounting, fields, unit.birth_txg()) {
            reading.note(Namespace::Accounting, fields, unit);
        }
    }
    reading
}

/// R-U 的整扫参照路径：逐个读得出的单元收它的每次出现，按臂滤。与增量计数不共用代码，最后一次扫描与阳性对照用它。
fn unit_scan_reading_by_brute_force(arm: Arm, inputs: &ScanInputs<'_>) -> ReadingResult {
    let disk = inputs.disk;
    let mut candidates: Vec<UnitId> = (0..disk.units.len())
        .filter(|unit_id| inputs.is_candidate(*unit_id))
        .collect();
    if inputs.planted.is_some() {
        candidates.push(PLANTED_UNIT);
    }
    let mut tombstones_by_object: HashMap<u64, Vec<TombstoneRecordSummary>> = HashMap::new();
    if arm == Arm::TodaysTombstones {
        for unit_id in &candidates {
            let unit = unit_by_id(disk, inputs.planted, *unit_id);
            if let UnitContents::TombstoneRecords(records) = &unit.contents {
                for record in records {
                    tombstones_by_object.entry(record.object).or_default().push(*record);
                }
            }
        }
    }
    let mut reading = ReadingResult::empty();
    for unit_id in &candidates {
        let unit = unit_by_id(disk, inputs.planted, *unit_id);
        for occurrence in occurrences_of(&unit.contents) {
            let counted = match arm {
                Arm::WithoutTombstones => true,
                Arm::TodaysTombstones => !object_of_tag(occurrence.tag)
                    .and_then(|object| tombstones_by_object.get(&object))
                    .is_some_and(|tombstones| tombstone_suppresses(tombstones, occurrence.tag)),
                Arm::DeathRecordForEveryKey => {
                    !death_record_suppresses(inputs.death_time, &occurrence.key, unit.birth_txg())
                }
            };
            if counted {
                reading.note(occurrence.key.namespace, occurrence.key.fields, unit);
            }
        }
    }
    reading
}

fn read_with(reader: Reader, arm: Arm, inputs: &ScanInputs<'_>) -> ReadingResult {
    match reader {
        Reader::UnitScan => unit_scan_reading_by_brute_force(arm, inputs),
        Reader::SelectCurrent | Reader::SelectCurrentGatedByObject => select_current_reading(reader, arm, inputs),
    }
}

/// 一格的读数：复活 r、漏活 m、环窗内的复活 r_in（Q14）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CellMeasurement {
    resurrected: u64,
    missed: u64,
    resurrected_in_window: u64,
}

fn measure_against_truth(read: &KeysWithProviders, truth: &BTreeSet<KeyFields>, txg: u64) -> CellMeasurement {
    let mut measurement = CellMeasurement::default();
    for (fields, provider) in read {
        if !truth.contains(fields) {
            measurement.resurrected += 1;
            if provider
                .freshest_release
                .is_some_and(|released_at| txg - released_at <= RING_WINDOW_IN_PUBLISHES)
            {
                measurement.resurrected_in_window += 1;
            }
        }
    }
    measurement.missed = length_as_u64(truth.iter().filter(|fields| !read.contains_key(*fields)).count());
    measurement
}

// ---------------------------------------------------------------------------------------------
// 轨迹：峰值、峰值在哪一次扫描（报 txg）、为正轮数、期末（第八节）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Trajectory {
    resurrected_peak: u64,
    resurrected_peak_at: u64,
    resurrected_positive_scans: u64,
    resurrected_final: u64,
    missed_peak: u64,
    missed_positive_scans: u64,
    missed_positive_scans_in_unlink: u64,
    missed_final: u64,
    deaths_peak: u64,
    deaths_final: u64,
    in_window_peak: u64,
    in_window_positive_scans: u64,
    outside_window_peak: u64,
    outside_window_positive_scans: u64,
}

impl Trajectory {
    fn observe(&mut self, txg: u64, measurement: CellMeasurement, deaths: u64, is_in_unlink: bool) {
        if measurement.resurrected > self.resurrected_peak {
            self.resurrected_peak = measurement.resurrected;
            self.resurrected_peak_at = txg;
        }
        self.resurrected_positive_scans += u64::from(measurement.resurrected > 0);
        self.resurrected_final = measurement.resurrected;
        self.missed_peak = self.missed_peak.max(measurement.missed);
        self.missed_positive_scans += u64::from(measurement.missed > 0);
        self.missed_positive_scans_in_unlink += u64::from(measurement.missed > 0 && is_in_unlink);
        self.missed_final = measurement.missed;
        self.deaths_peak = self.deaths_peak.max(deaths);
        self.deaths_final = deaths;
        let outside = measurement.resurrected - measurement.resurrected_in_window;
        self.in_window_peak = self.in_window_peak.max(measurement.resurrected_in_window);
        self.in_window_positive_scans += u64::from(measurement.resurrected_in_window > 0);
        self.outside_window_peak = self.outside_window_peak.max(outside);
        self.outside_window_positive_scans += u64::from(outside > 0);
    }
}

// ---------------------------------------------------------------------------------------------
// 真值（第一节「复活数」「漏活数」、C2、C3）
// ---------------------------------------------------------------------------------------------

fn truth_of_the_version(version: &Version, disk: &Disk, allocator: &Allocator) -> Vec<BTreeSet<KeyFields>> {
    let mut truth = vec![BTreeSet::new(); NAMESPACE_COUNT];
    for position in 0..version.file_units.len() {
        truth[Namespace::Extent.position()].insert(extent_key(FIRST_FILE_INODE, length_as_u64(position)));
    }
    for container in &version.inode_containers {
        for record in &container.records {
            truth[Namespace::Inode.position()].insert(single_field_key(record.inode));
        }
    }
    for (slot, record) in &allocator.records {
        truth[Namespace::AllocationRecordsPresent.position()].insert(single_field_key(*slot));
        if !record.is_released {
            truth[Namespace::AllocationRecordsAllocated.position()].insert(single_field_key(*slot));
        }
    }
    let units_of_the_version = version
        .file_units
        .iter()
        .chain(version.inode_containers.iter().map(|container| &container.unit))
        .chain(version.tombstone_containers.iter().map(|container| &container.unit));
    for unit_id in units_of_the_version {
        let unit = &disk.units[*unit_id];
        assert!(
            unit.class == UNIT_CLASS_DATA || unit.class == UNIT_CLASS_PACKED,
            "这一版的码 1 / 码 3 单元：数据单元与打包容器"
        );
        truth[Namespace::AllocationUnitsOfTheVersion.position()].insert(single_field_key(unit.slot));
        truth[Namespace::CentralMappingUnitsOfTheVersion.position()]
            .insert(unit.mapping_key.expect("码 1、码 3 单元进映射"));
    }
    for fields in version.accounting.shape.keys_in_order() {
        truth[Namespace::Accounting.position()].insert(fields);
    }
    for fields in version.mapping.shape.keys_in_order() {
        truth[Namespace::CentralMapping.position()].insert(fields);
    }
    for inode in &version.deleted_queue {
        truth[Namespace::DeletedInodes.position()].insert(single_field_key(*inode));
    }
    for container in &version.tombstone_containers {
        for record in &container.records {
            truth[Namespace::Tombstone.position()].insert(tombstone_key(*record));
        }
    }
    truth
}

// ---------------------------------------------------------------------------------------------
// 一段历史在一个几何、一条写路径上的模拟
// ---------------------------------------------------------------------------------------------

/// 一次用户改动（S12，`mounted_session.rs:54-68`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UserChange {
    Overwrite,
    SequentialWrite { data_units: u64 },
    NewInodes { count: u64 },
    Unlink { inode: u64 },
}

/// 断言的计数：每条断言判了几次、违反几次（第七节、修订 1 第 12.6 节）。
#[derive(Clone, Debug, Default)]
struct AssertionTally {
    judged: BTreeMap<&'static str, u64>,
    violated: BTreeMap<&'static str, u64>,
}

impl AssertionTally {
    fn judge(&mut self, name: &'static str, holds: bool) {
        *self.judged.entry(name).or_insert(0) += 1;
        if !holds {
            *self.violated.entry(name).or_insert(0) += 1;
        }
    }

    fn merge(&mut self, other: &AssertionTally) {
        for (name, count) in &other.judged {
            *self.judged.entry(name).or_insert(0) += count;
        }
        for (name, count) in &other.violated {
            *self.violated.entry(name).or_insert(0) += count;
        }
    }

    fn violations_of(&self, name: &str) -> u64 {
        self.violated.get(name).copied().unwrap_or(0)
    }
}

/// 一次 unlink 发了几次（意图加各批），A5、A10 用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UnlinkPublishCount {
    inode: u64,
    data_units_at_the_intent: u64,
    publishes: u64,
}

struct Simulation {
    geometry: Geometry,
    write_path: WritePath,
    batch_data_units: u64,
    allocator: Allocator,
    disk: Disk,
    version: Version,
    roots_on_disk: BTreeMap<(u64, u64), u64>,
    truth: Vec<BTreeSet<KeyFields>>,
    ever_seen: Vec<BTreeSet<KeyFields>>,
    death_time: HashMap<NamespacedKey, u64>,
    unit_lists: UnitLists,
    unit_scan_readings: Vec<UnitScanReading>,
    trajectories: BTreeMap<(Arm, Reader, ReportedTree), Trajectory>,
    publishes: u64,
    scans: u64,
    refused_steps: u64,
    occupancy_peak: u64,
    live_slot_peak: u64,
    /// 一次发布取落点时占着的槽：这一版的活槽加这次换下的（执行员修订 2 C17）。
    slots_held_during_a_publish_peak: u64,
    exceeded_the_premise_at: Option<u64>,
    does_not_fit_at: Option<u64>,
    placement_log: Vec<Vec<(u64, u64)>>,
    scan_log: Vec<Vec<u64>>,
    tree_kinds_with_units: BTreeSet<TreeKind>,
    assertions: AssertionTally,
    unlink_publish_counts: Vec<UnlinkPublishCount>,
    reuse_at_exactly_the_ring_length: Vec<(u64, u64)>,
    cross_checks_every_scan: bool,
    /// 每次发布记根时回收的槽数，按 txg（A14 (ii)）。
    slots_reclaimed_by_publish: BTreeMap<u64, u64>,
}

impl Simulation {
    /// mkfs、mkfs 那个进程里的两次零单元暖机与新池新建文件（txg 3）、可写挂载的写行（txg 4）与暖机（txg 5）（12.4.1、M1–M5）。
    fn start(
        geometry: Geometry,
        write_path: WritePath,
        batch_data_units: u64,
        tight_unit_area_slots: Option<u64>,
        cross_checks_every_scan: bool,
    ) -> Self {
        let unit_area_slots = match geometry {
            Geometry::Ring => Some(RING_GEOMETRY_UNIT_AREA_SLOTS),
            Geometry::Wide => None,
            Geometry::Tight => Some(tight_unit_area_slots.expect("G-紧 的单元区槽数由 G-宽 的峰值定")),
        };
        let mut allocator = Allocator::new(geometry, unit_area_slots);
        let mut disk = Disk::default();
        let make_filesystem_unit = |owner: UnitOwner, class: u64, slot: u64, span_slots: u64| Unit {
            owner,
            class,
            group: None,
            write_order: (0, 0, 0),
            slot,
            span_slots,
            release_txg: None,
            is_readable: true,
            mapping_key: None,
            contents: UnitContents::WithoutKeys,
        };
        disk.write(make_filesystem_unit(UnitOwner::InstanceTable, UNIT_CLASS_PACKED, INSTANCE_TABLE_GENESIS_SLOT, 2));
        disk.write(make_filesystem_unit(UnitOwner::TreeTable, UNIT_CLASS_INDEX_NODE, TREE_TABLE_GENESIS_SLOT, 1));
        allocator.record(INSTANCE_TABLE_GENESIS_SLOT, 2, 0, 0);
        allocator.record(TREE_TABLE_GENESIS_SLOT, 1, 0, 1);
        let mut roots_on_disk = BTreeMap::new();
        for region in 0..ROOT_RING_REGIONS {
            roots_on_disk.insert((region, 0), 0);
        }
        if geometry == Geometry::Ring {
            allocator.root_ring = Some(RootRingTable::after_make_filesystem());
        }
        for zero_unit_txg in 1..NEW_POOL_FILE_CREATION_TXG {
            roots_on_disk.insert(root_ring_slot_of(zero_unit_txg), zero_unit_txg);
            allocator.record_root(zero_unit_txg);
        }
        let version = Version {
            txg: NEW_POOL_FILE_CREATION_TXG - 1,
            instance: MAKE_FILESYSTEM_PROCESS_INSTANCE,
            highest_transaction: 0,
            next_inode: FIRST_FILE_INODE,
            file_units: Vec::new(),
            extent_root: None,
            extent_lower: BTreeMap::new(),
            inode_containers: Vec::new(),
            inode_root: None,
            deleted_queue: Vec::new(),
            deleted_root: None,
            tombstone_containers: Vec::new(),
            tombstone_root: None,
            allocation_nodes: BTreeMap::new(),
            accounting: CodeTwoTreeVersion::default(),
            mapping: CodeTwoTreeVersion::default(),
            tree_table: 1,
            instance_table: 0,
        };
        let readings = write_path
            .arms()
            .into_iter()
            .map(|arm| UnitScanReading::new(arm, geometry == Geometry::Ring))
            .collect();
        let mut simulation = Self {
            geometry,
            write_path,
            batch_data_units,
            allocator,
            disk,
            version,
            roots_on_disk,
            truth: vec![BTreeSet::new(); NAMESPACE_COUNT],
            ever_seen: vec![BTreeSet::new(); NAMESPACE_COUNT],
            death_time: HashMap::new(),
            unit_lists: UnitLists::default(),
            unit_scan_readings: readings,
            trajectories: BTreeMap::new(),
            publishes: 0,
            scans: 0,
            refused_steps: 0,
            occupancy_peak: 0,
            live_slot_peak: 0,
            slots_held_during_a_publish_peak: 0,
            exceeded_the_premise_at: None,
            does_not_fit_at: None,
            placement_log: Vec::new(),
            scan_log: Vec::new(),
            tree_kinds_with_units: BTreeSet::new(),
            assertions: AssertionTally::default(),
            unlink_publish_counts: Vec::new(),
            reuse_at_exactly_the_ring_length: Vec::new(),
            cross_checks_every_scan,
            slots_reclaimed_by_publish: BTreeMap::new(),
        };
        simulation.publish_or_stop(PublishOperation::FirstFile);
        let latest_txg = simulation.version.txg;
        let roots_at_mount = simulation.roots_on_disk.clone();
        simulation.allocator.reset_at_mount(&roots_at_mount, latest_txg);
        simulation.publish_or_stop(PublishOperation::RowPublish);
        simulation.publish_or_stop(PublishOperation::WarmUp);
        simulation
    }

    fn has_stopped(&self) -> bool {
        self.does_not_fit_at.is_some()
    }

    fn publish_or_stop(&mut self, operation: PublishOperation) {
        if self.has_stopped() {
            return;
        }
        if self.publish(operation).is_err() {
            self.does_not_fit_at = Some(self.version.txg + 1);
        }
    }

    /// 一次发布：先释放这次换下的全部落点，再按 bump 次序取落点，写单元，再记根（M6）；之后扫一遍。
    fn publish(&mut self, operation: PublishOperation) -> Result<(), PlacementRefused> {
        let txg = self.version.txg + 1;
        let plan = plan_logical(&self.version, operation, txg, self.write_path, NEW_POOL_FILE_CREATION_TXG);
        let assembled = settle_publish(&plan, &self.version, &self.disk, &self.allocator)?;
        let allocator_before = self.allocator.clone();
        for unit_id in &assembled.releases {
            let (slot, span_slots) = (self.disk.units[*unit_id].slot, self.disk.units[*unit_id].span_slots);
            self.allocator.release(slot, span_slots, txg);
            self.disk.units[*unit_id].release_txg = Some(txg);
        }
        let first_new_unit = self.disk.units.len();
        let mut placements = Vec::new();
        let mut unit_of_role: HashMap<Role, UnitId> = HashMap::new();
        for (offset, role) in assembled.roles.iter().enumerate() {
            let unit_id = first_new_unit + offset;
            let slot = self
                .allocator
                .allocate(role.placement_rule(), txg, unit_id)
                .expect("固定点在拷贝上取得到的落点，真发时同一串分配照样取得到");
            placements.push((slot, role.placement_rule().span_slots()));
            unit_of_role.insert(*role, unit_id);
        }
        self.judge_cluster_segments_of_this_publish(&allocator_before, &assembled.roles, &placements);
        let new_units = self.build_units(&plan, &assembled, &placements);
        let mut made_unreadable = Vec::new();
        for unit in new_units {
            for slot in unit.slot..unit.slot + unit.span_slots {
                self.judge_reuse(slot, txg);
            }
            if let Some(kind) = unit.owner.tree_kind() {
                self.tree_kinds_with_units.insert(kind);
            }
            made_unreadable.extend(self.disk.write(unit));
        }
        let reclaimed_placements = self.allocator.record_root(txg);
        self.slots_reclaimed_by_publish
            .insert(txg, reclaimed_placements.iter().map(|(_, span)| *span).sum());
        self.roots_on_disk.insert(root_ring_slot_of(txg), txg);
        self.assertions.judge("reclaim_reports_whole_placements", reclaimed_placements.iter().all(|(_, span)| *span >= 1));
        self.version = self.build_version(&plan, &assembled, &unit_of_role);
        self.note_unit_lists(first_new_unit);
        if let Some(record) = plan.tombstones_written.first() {
            let tombstone_holds_the_intent_txg = record.death_txg == txg;
            self.assertions.judge("K3_tombstone_death_txg_is_the_intent_txg", tombstone_holds_the_intent_txg);
        }
        self.placement_log.push(placements);
        self.publishes += 1;
        let written: Vec<UnitId> = (first_new_unit..self.disk.units.len()).collect();
        self.scan(txg, &assembled.releases, &written, &made_unreadable, &plan.tombstones_written);
        Ok(())
    }

    fn note_unit_lists(&mut self, first_new_unit: UnitId) {
        for unit_id in first_new_unit..self.disk.units.len() {
            let unit = &self.disk.units[unit_id];
            let is_an_accounting_leaf = matches!(unit.contents, UnitContents::AccountingKeys(_));
            match unit.owner {
                UnitOwner::FileData => self.unit_lists.data_units.push(unit_id),
                UnitOwner::InodeContainer => self.unit_lists.inode_containers.push(unit_id),
                UnitOwner::TombstoneContainer => self.unit_lists.tombstone_containers.push(unit_id),
                UnitOwner::AccountingNode => {
                    if is_an_accounting_leaf {
                        self.unit_lists.accounting_leaves.push(unit_id);
                    }
                }
                UnitOwner::ExtentLowerNode
                | UnitOwner::ExtentRoot
                | UnitOwner::InodeRoot
                | UnitOwner::AllocationNode
                | UnitOwner::MappingNode
                | UnitOwner::DeletedInodesRoot
                | UnitOwner::TombstoneRoot
                | UnitOwner::TreeTable
                | UnitOwner::InstanceTable => {}
            }
        }
    }

    /// A15：G-环 每一次再分配（第 t 次发布写进的槽，上一回住在那里的单元释放代 s）都满足 t ≥ s + 24；等号那几格记下（A14、A15）。
    fn judge_reuse(&mut self, slot: u64, txg: u64) {
        let Some(previous_occupant) = self.disk.occupant_of_slot.get(&slot).copied() else {
            return;
        };
        let released_at = self.disk.units[previous_occupant].release_txg;
        match self.geometry {
            Geometry::Ring => {
                let holds = released_at.is_some_and(|released| txg >= released + RING_LENGTH_IN_ROOTS_FOR_THE_REUSE_ASSERTION);
                self.assertions.judge("A15_reuse_waits_a_ring_length", holds);
                if released_at.is_some_and(|released| txg == released + RING_LENGTH_IN_ROOTS_FOR_THE_REUSE_ASSERTION) {
                    self.reuse_at_exactly_the_ring_length.push((txg, slot));
                }
            }
            Geometry::Tight => {
                let holds = released_at.is_some_and(|released| released < txg);
                self.assertions.judge("tight_reuse_only_of_released_before_this_publish", holds);
            }
            Geometry::Wide => self.assertions.judge("wide_never_reuses", false),
        }
    }

    /// A18：用户数据落点不落在这次挂载开过的任何聚簇段里；每次开新段，那一段在这次发布开始时全空。另写一份按落点逐个核，不调分配函数。
    fn judge_cluster_segments_of_this_publish(&mut self, before: &Allocator, roles: &[Role], placements: &[(u64, u64)]) {
        if self.geometry == Geometry::Tight {
            return;
        }
        for (role, (slot, _)) in roles.iter().zip(placements) {
            if let Role::Data(_) = role {
                let segment = UNIT_AREA_START_SLOT
                    + ((slot - UNIT_AREA_START_SLOT) / CLUSTER_SEGMENT_SLOTS) * CLUSTER_SEGMENT_SLOTS;
                let in_a_cluster_segment = self.allocator.cluster_segments.contains(&segment);
                self.assertions.judge("A18_user_data_outside_cluster_segments", !in_a_cluster_segment);
            }
        }
        let opened: Vec<u64> = self
            .allocator
            .cluster_segments
            .difference(&before.cluster_segments)
            .copied()
            .collect();
        for segment in opened {
            let was_empty = (segment..segment + CLUSTER_SEGMENT_SLOTS).all(|slot| before.is_free(slot));
            self.assertions.judge("A18_opened_segment_was_empty", was_empty);
        }
    }

    fn data_unit_birth(&self, plan: &LogicalPlan, position: usize) -> u64 {
        match plan.data_units[position] {
            PlannedUnit::Rewritten => plan.txg,
            PlannedUnit::Carried(unit) => self.disk.units[unit].birth_txg(),
        }
    }

    /// 这次写的每个单元（落点先于内容：分配记录树的叶装的是取完落点之后的记录，M6）。
    fn build_units(&self, plan: &LogicalPlan, assembled: &AssembledPublish, placements: &[(u64, u64)]) -> Vec<Unit> {
        let data_units_after = length_as_u64(plan.data_units.len());
        let mut units = Vec::new();
        for (role, (slot, span_slots)) in assembled.roles.iter().zip(placements) {
            let mapping_key = assembled.mapping_keys_of_rewritten.get(role).copied();
            let mut write_order = (plan.txg, plan.instance, plan.transaction_base);
            let mut group = None;
            let (owner, class, contents) = match role {
                Role::Data(position) => {
                    let anchor_unit = length_as_u64(*position);
                    write_order.2 = plan.transaction_base + anchor_unit;
                    group = Some(SelectionGroup::DataUnit {
                        birth_tree: TREE_IDENTIFIER_EXTENT,
                        object: FIRST_FILE_INODE,
                        object_birth: plan.file_object_birth,
                        anchor_unit,
                    });
                    (
                        UnitOwner::FileData,
                        UNIT_CLASS_DATA,
                        UnitContents::DataUnit {
                            object: FIRST_FILE_INODE,
                            object_birth: plan.file_object_birth,
                            anchor_unit,
                        },
                    )
                }
                Role::InstanceTable => (UnitOwner::InstanceTable, UNIT_CLASS_PACKED, UnitContents::WithoutKeys),
                Role::ExtentLower((level, index)) => {
                    let contents = if *level == 0 {
                        let first = index * EXTENT_TREE_LOWER_LEAF_DATA_UNITS;
                        let last_exclusive = ((index + 1) * EXTENT_TREE_LOWER_LEAF_DATA_UNITS).min(data_units_after);
                        UnitContents::ExtentRecords(
                            (first..last_exclusive)
                                .map(|data_unit| ExtentRecordSummary {
                                    inode: FIRST_FILE_INODE,
                                    data_unit,
                                    pointer_birth: self.data_unit_birth(plan, u64_as_index(data_unit)),
                                })
                                .collect(),
                        )
                    } else {
                        UnitContents::WithoutKeys
                    };
                    (UnitOwner::ExtentLowerNode, UNIT_CLASS_INDEX_NODE, contents)
                }
                Role::ExtentRoot => {
                    let inline = if data_units_after == 1 {
                        vec![ExtentRecordSummary {
                            inode: FIRST_FILE_INODE,
                            data_unit: 0,
                            pointer_birth: self.data_unit_birth(plan, 0),
                        }]
                    } else {
                        Vec::new()
                    };
                    (UnitOwner::ExtentRoot, UNIT_CLASS_INDEX_NODE, UnitContents::ExtentRecords(inline))
                }
                Role::InodeContainer(position) => {
                    let container = &plan.inode_containers[*position];
                    group = Some(SelectionGroup::PackedContainer {
                        birth_tree: TREE_IDENTIFIER_INODE,
                        record_type: PACKED_TYPE_INODE,
                        container_number: container.container_number,
                        container_birth: container.container_birth,
                    });
                    (
                        UnitOwner::InodeContainer,
                        UNIT_CLASS_PACKED,
                        UnitContents::InodeRecords(container.records.clone()),
                    )
                }
                Role::InodeRoot => (UnitOwner::InodeRoot, UNIT_CLASS_INDEX_NODE, UnitContents::WithoutKeys),
                Role::AllocationNode(node) => {
                    let contents = match node {
                        AllocationNode::BelowTheRoot {
                            level: 0,
                            index_in_device,
                            ..
                        } => {
                            let first_slot = index_in_device * ALLOCATION_RECORD_TREE_LEAF_SLOTS;
                            UnitContents::AllocationRecords(
                                self.allocator
                                    .records
                                    .range(first_slot..first_slot + ALLOCATION_RECORD_TREE_LEAF_SLOTS)
                                    .map(|(record_slot, record)| (*record_slot, record.is_released))
                                    .collect(),
                            )
                        }
                        AllocationNode::BelowTheRoot { .. } | AllocationNode::Root => UnitContents::WithoutKeys,
                    };
                    (UnitOwner::AllocationNode, UNIT_CLASS_INDEX_NODE, contents)
                }
                Role::AccountingNode(position) => {
                    let contents = match assembled.accounting_plan.shape.contents_at(*position) {
                        CodeTwoNodeContents::Leaf { keys } => UnitContents::AccountingKeys(keys.clone()),
                        CodeTwoNodeContents::Internal { .. } => UnitContents::WithoutKeys,
                    };
                    (UnitOwner::AccountingNode, UNIT_CLASS_INDEX_NODE, contents)
                }
                Role::DeletedInodesRoot => (
                    UnitOwner::DeletedInodesRoot,
                    UNIT_CLASS_INDEX_NODE,
                    UnitContents::DeletedInodes(plan.deleted_queue.clone()),
                ),
                Role::TombstoneContainer(position) => {
                    let container = &plan.tombstone_containers[*position];
                    group = Some(SelectionGroup::PackedContainer {
                        birth_tree: TREE_IDENTIFIER_TOMBSTONE,
                        record_type: PACKED_TYPE_TOMBSTONE,
                        container_number: container.container_number,
                        container_birth: container.container_birth,
                    });
                    (
                        UnitOwner::TombstoneContainer,
                        UNIT_CLASS_PACKED,
                        UnitContents::TombstoneRecords(container.records.clone()),
                    )
                }
                Role::TombstoneRoot => (UnitOwner::TombstoneRoot, UNIT_CLASS_INDEX_NODE, UnitContents::WithoutKeys),
                Role::MappingNode(position) => {
                    let contents = match assembled.mapping_plan.shape.contents_at(*position) {
                        CodeTwoNodeContents::Leaf { keys } => UnitContents::MappingKeys(keys.clone()),
                        CodeTwoNodeContents::Internal { .. } => UnitContents::WithoutKeys,
                    };
                    (UnitOwner::MappingNode, UNIT_CLASS_INDEX_NODE, contents)
                }
                Role::TreeTable => (UnitOwner::TreeTable, UNIT_CLASS_INDEX_NODE, UnitContents::WithoutKeys),
            };
            units.push(Unit {
                owner,
                class,
                group,
                write_order,
                slot: *slot,
                span_slots: *span_slots,
                release_txg: None,
                is_readable: true,
                mapping_key,
                contents,
            });
        }
        units
    }

    fn unit_of_planned(planned: PlannedUnit, role: Role, unit_of_role: &HashMap<Role, UnitId>) -> UnitId {
        match planned {
            PlannedUnit::Carried(unit) => unit,
            PlannedUnit::Rewritten => *unit_of_role.get(&role).expect("重写的角色这次取了落点"),
        }
    }

    fn code_two_version_after(
        plan: &CodeTwoPlan,
        previous: &CodeTwoTreeVersion,
        role_of: fn(CodeTwoNodePosition) -> Role,
        unit_of_role: &HashMap<Role, UnitId>,
    ) -> CodeTwoTreeVersion {
        let units = plan
            .shape
            .nodes
            .iter()
            .zip(&plan.origins)
            .map(|((position, _), origin)| match origin {
                CodeTwoOrigin::CarriedFrom(previous_position) => previous.unit_at(*previous_position),
                CodeTwoOrigin::Rewritten => *unit_of_role.get(&role_of(*position)).expect("重写的节点这次取了落点"),
            })
            .collect();
        CodeTwoTreeVersion {
            shape: plan.shape.clone(),
            units,
        }
    }

    fn build_version(&self, plan: &LogicalPlan, assembled: &AssembledPublish, unit_of_role: &HashMap<Role, UnitId>) -> Version {
        let previous = &self.version;
        Version {
            txg: plan.txg,
            instance: plan.instance,
            highest_transaction: plan.highest_transaction_after,
            next_inode: plan.next_inode_after,
            file_units: plan
                .data_units
                .iter()
                .enumerate()
                .map(|(position, planned)| Self::unit_of_planned(*planned, Role::Data(position), unit_of_role))
                .collect(),
            extent_root: Some(Self::unit_of_planned(plan.extent_root, Role::ExtentRoot, unit_of_role)),
            extent_lower: plan
                .extent_lower
                .iter()
                .map(|(position, planned)| {
                    (*position, Self::unit_of_planned(*planned, Role::ExtentLower(*position), unit_of_role))
                })
                .collect(),
            inode_containers: plan
                .inode_containers
                .iter()
                .enumerate()
                .map(|(position, container)| InodeContainerVersion {
                    container_number: container.container_number,
                    container_birth: container.container_birth,
                    records: container.records.clone(),
                    unit: Self::unit_of_planned(container.origin, Role::InodeContainer(position), unit_of_role),
                })
                .collect(),
            inode_root: Some(Self::unit_of_planned(plan.inode_root, Role::InodeRoot, unit_of_role)),
            deleted_queue: plan.deleted_queue.clone(),
            deleted_root: plan
                .deleted_root
                .map(|planned| Self::unit_of_planned(planned, Role::DeletedInodesRoot, unit_of_role)),
            tombstone_containers: plan
                .tombstone_containers
                .iter()
                .enumerate()
                .map(|(position, container)| TombstoneContainerVersion {
                    container_number: container.container_number,
                    container_birth: container.container_birth,
                    records: container.records.clone(),
                    unit: Self::unit_of_planned(container.origin, Role::TombstoneContainer(position), unit_of_role),
                })
                .collect(),
            tombstone_root: plan
                .tombstone_root
                .map(|planned| Self::unit_of_planned(planned, Role::TombstoneRoot, unit_of_role)),
            allocation_nodes: assembled
                .allocation_nodes
                .iter()
                .map(|(node, planned)| (*node, Self::unit_of_planned(*planned, Role::AllocationNode(*node), unit_of_role)))
                .collect(),
            accounting: Self::code_two_version_after(&assembled.accounting_plan, &previous.accounting, Role::AccountingNode, unit_of_role),
            mapping: Self::code_two_version_after(&assembled.mapping_plan, &previous.mapping, Role::MappingNode, unit_of_role),
            tree_table: *unit_of_role.get(&Role::TreeTable).expect("树表每次都重写"),
            instance_table: if plan.instance_table_rewritten {
                *unit_of_role.get(&Role::InstanceTable).expect("写行那次重写实例表")
            } else {
                previous.instance_table
            },
        }
    }

    /// 第 t 次发布落盘之后扫一次（第一节「扫描时刻」）：先更新真值与死亡记录，再更新 R-U 的增量计数，再按三种读法量每一格。
    fn scan(
        &mut self,
        txg: u64,
        released: &[UnitId],
        written: &[UnitId],
        made_unreadable: &[UnitId],
        tombstones_written: &[TombstoneRecordSummary],
    ) {
        let new_truth = truth_of_the_version(&self.version, &self.disk, &self.allocator);
        let mut born = Vec::new();
        let mut died = Vec::new();
        for namespace in NAMESPACES {
            let position = namespace.position();
            for fields in new_truth[position].difference(&self.truth[position]) {
                born.push(namespaced(namespace, *fields));
            }
            for fields in self.truth[position].difference(&new_truth[position]) {
                died.push(namespaced(namespace, *fields));
            }
            self.ever_seen[position].extend(new_truth[position].iter().copied());
        }
        for key in &died {
            self.death_time.insert(*key, txg);
        }
        self.truth = new_truth;
        for reading in &mut self.unit_scan_readings {
            for unit_id in made_unreadable {
                reading.on_unit_unreadable(*unit_id, &self.disk.units[*unit_id], &self.death_time);
            }
            for unit_id in released {
                reading.on_unit_released(*unit_id, &self.disk.units[*unit_id], txg, &self.death_time);
            }
            for unit_id in written {
                reading.on_unit_readable(*unit_id, &self.disk.units[*unit_id], &self.death_time);
            }
            for key in &born {
                reading.set_truth(*key, true);
            }
            for key in &died {
                reading.set_truth(*key, false);
                if reading.arm == Arm::DeathRecordForEveryKey {
                    reading.on_key_death(*key);
                }
            }
            if reading.arm == Arm::TodaysTombstones {
                for tombstone in tombstones_written {
                    reading.on_tombstone(*tombstone, &self.disk);
                }
            }
            reading.expire_the_window(txg, &self.disk, &self.death_time);
        }
        self.scans += 1;
        self.occupancy_peak = self.occupancy_peak.max(self.allocator.allocated_slot_count);
        self.live_slot_peak = self.live_slot_peak.max(self.allocator.live_slot_count);
        let released_slots: u64 = released.iter().map(|unit_id| self.disk.units[*unit_id].span_slots).sum();
        self.slots_held_during_a_publish_peak = self
            .slots_held_during_a_publish_peak
            .max(self.allocator.live_slot_count + released_slots);
        if self.geometry == Geometry::Ring
            && self.allocator.allocated_slot_count > OCCUPANCY_CAP_SLOTS
            && self.exceeded_the_premise_at.is_none()
        {
            self.exceeded_the_premise_at = Some(txg);
        }
        let readable_by_scan = self.disk.readable_units_by_scan();
        self.assertions.judge(
            "A19_readable_units_equal_written_minus_overwritten",
            readable_by_scan + self.disk.units_made_unreadable == self.disk.units_written,
        );
        self.assertions.judge(
            "A2_accounting_rows_per_version",
            length_as_u64(self.truth[Namespace::Accounting.position()].len())
                == ACCOUNTING_POOL_WIDE_ROWS + ACCOUNTING_PER_DEVICE_ROWS * u64::from(DEVICE_COUNT),
        );
        self.measure_every_cell(txg);
        if self.cross_checks_every_scan {
            self.cross_check_the_unit_scan();
        }
    }

    fn measure_every_cell(&mut self, txg: u64) {
        let is_in_unlink = !self.version.deleted_queue.is_empty();
        let inputs = ScanInputs {
            disk: &self.disk,
            unit_lists: &self.unit_lists,
            death_time: &self.death_time,
            planted: None,
            erased: None,
        };
        let mut metrics_of_this_scan = Vec::new();
        for arm in self.write_path.arms() {
            let unit_scan = self
                .unit_scan_readings
                .iter()
                .find(|reading| reading.arm == arm)
                .expect("这条写路径的每条臂一份 R-U 计数");
            let select_current = select_current_reading(Reader::SelectCurrent, arm, &inputs);
            let gated = select_current_reading(Reader::SelectCurrentGatedByObject, arm, &inputs);
            for reader in READERS {
                for tree in REPORTED_TREES {
                    let (measurement, deaths) = match tree.namespace(reader) {
                        None => (CellMeasurement::default(), 0),
                        Some(namespace) => {
                            let position = namespace.position();
                            let deaths = length_as_u64(self.ever_seen[position].len() - self.truth[position].len());
                            let measurement = match reader {
                                Reader::UnitScan => CellMeasurement {
                                    resurrected: unit_scan.resurrected[position],
                                    missed: unit_scan.missed[position],
                                    resurrected_in_window: unit_scan.resurrected_in_window[position],
                                },
                                Reader::SelectCurrent => {
                                    measure_against_truth(&select_current.keys[position], &self.truth[position], txg)
                                }
                                Reader::SelectCurrentGatedByObject => {
                                    measure_against_truth(&gated.keys[position], &self.truth[position], txg)
                                }
                            };
                            (measurement, deaths)
                        }
                    };
                    self.trajectories
                        .entry((arm, reader, tree))
                        .or_default()
                        .observe(txg, measurement, deaths, is_in_unlink);
                    if arm == Arm::WithoutTombstones && reader == Reader::UnitScan && self.geometry == Geometry::Wide {
                        self.assertions.judge("A3_unit_scan_on_wide_reads_every_death", measurement.resurrected == deaths);
                    }
                    if arm == Arm::DeathRecordForEveryKey {
                        let name = if self.geometry == Geometry::Ring {
                            "A13_death_records_zero_on_ring"
                        } else {
                            "A4_death_records_zero"
                        };
                        self.assertions.judge(name, measurement.resurrected == 0 && measurement.missed == 0);
                    }
                    metrics_of_this_scan.extend([measurement.resurrected, measurement.missed, deaths]);
                }
            }
        }
        self.scan_log.push(metrics_of_this_scan);
    }

    /// 最后一次扫描（单测里每次扫描）把 R-U 的增量计数与整扫参照路径逐个名字空间比（两条路不共用代码）。
    fn cross_check_the_unit_scan(&mut self) {
        let txg = self.version.txg;
        let inputs = ScanInputs {
            disk: &self.disk,
            unit_lists: &self.unit_lists,
            death_time: &self.death_time,
            planted: None,
            erased: None,
        };
        let mut every_namespace_matches = true;
        for reading in &self.unit_scan_readings {
            let brute = unit_scan_reading_by_brute_force(reading.arm, &inputs);
            for namespace in NAMESPACES {
                let position = namespace.position();
                let measured = measure_against_truth(&brute.keys[position], &self.truth[position], txg);
                let window_matches = !reading.tracks_the_ring_window
                    || measured.resurrected_in_window == reading.resurrected_in_window[position];
                if measured.resurrected != reading.resurrected[position]
                    || measured.missed != reading.missed[position]
                    || !window_matches
                {
                    every_namespace_matches = false;
                }
            }
        }
        self.assertions.judge("unit_scan_incremental_equals_brute_force", every_namespace_matches);
    }

    /// 一次用户改动经会话（12.4.1 第 3 条、C15）：`crates/` 会拒的照拒、记一行、不发布。
    fn apply(&mut self, change: UserChange) {
        if self.has_stopped() {
            return;
        }
        match change {
            UserChange::Overwrite => {
                if self.version.has_live_inode(FIRST_FILE_INODE) {
                    self.publish_or_stop(PublishOperation::Overwrite);
                } else {
                    self.refused_steps += 1;
                }
            }
            UserChange::SequentialWrite { data_units } => {
                if self.version.has_live_inode(FIRST_FILE_INODE) {
                    self.publish_or_stop(PublishOperation::SequentialWrite { data_units });
                } else {
                    self.refused_steps += 1;
                }
            }
            UserChange::NewInodes { count } => self.publish_or_stop(PublishOperation::NewInodes { count }),
            UserChange::Unlink { inode } => {
                if self.version.is_with_trees_and_no_file() || !self.version.has_live_inode(inode) {
                    self.refused_steps += 1;
                    return;
                }
                let data_units_at_the_intent = if inode == FIRST_FILE_INODE {
                    length_as_u64(self.version.file_units.len())
                } else {
                    0
                };
                let publishes_before = self.publishes;
                self.publish_or_stop(PublishOperation::UnlinkIntent { inode });
                while !self.has_stopped() && !self.version.deleted_queue.is_empty() {
                    let step = if inode == FIRST_FILE_INODE {
                        let remaining = length_as_u64(self.version.file_units.len());
                        if remaining <= self.batch_data_units {
                            PublishOperation::UnlinkFinal { inode }
                        } else {
                            PublishOperation::UnlinkBatch {
                                data_units_kept: remaining - self.batch_data_units,
                            }
                        }
                    } else {
                        PublishOperation::UnlinkFinal { inode }
                    };
                    self.publish_or_stop(step);
                }
                self.unlink_publish_counts.push(UnlinkPublishCount {
                    inode,
                    data_units_at_the_intent,
                    publishes: self.publishes - publishes_before,
                });
            }
        }
    }

    /// 盘上见过的每个 inode 的对象出生代（号不复用，一个号一个出生代）。
    fn inode_births(&self) -> BTreeMap<u64, u64> {
        let mut births = BTreeMap::new();
        for unit_id in &self.unit_lists.inode_containers {
            if let UnitContents::InodeRecords(records) = &self.disk.units[*unit_id].contents {
                for record in records {
                    births.insert(record.inode, record.object_birth);
                }
            }
        }
        births
    }

    /// 现行那一版墓碑树里的全部墓碑（墓碑只增、现行容器恒读得出，盘上的墓碑就是这些）。
    fn tombstones_on_disk(&self) -> Vec<TombstoneRecordSummary> {
        self.version
            .tombstone_containers
            .iter()
            .flat_map(|container| container.records.iter().copied())
            .collect()
    }
}

// ---------------------------------------------------------------------------------------------
// 阳性对照（登记 5.5、C13、C14）：在每段历史的最后一次扫描上另做，不改主轨迹
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlantVariant {
    /// 臂 N；臂 B 取没有墓碑指认的 key。
    Plain,
    /// 臂 B 取有墓碑指认的 key。
    Tombstoned,
    /// 臂 P 删掉 k 的死亡记录再量。
    WithoutItsDeathRecord,
}

impl PlantVariant {
    const fn name(self) -> &'static str {
        match self {
            PlantVariant::Plain => "PC1",
            PlantVariant::Tombstoned => "PC1_tombstoned",
            PlantVariant::WithoutItsDeathRecord => "PC1_without_its_death_record",
        }
    }

    /// 期望增量（5.5）：臂 N +1；臂 B 没有墓碑指认 +1、有 +0；臂 P +0，删掉死亡记录 +1。
    const fn expected_increase(self, arm: Arm) -> u64 {
        match (arm, self) {
            (Arm::WithoutTombstones, _) | (Arm::TodaysTombstones, PlantVariant::Plain) => 1,
            (Arm::TodaysTombstones, _) => 0,
            (Arm::DeathRecordForEveryKey, PlantVariant::WithoutItsDeathRecord) => 1,
            (Arm::DeathRecordForEveryKey, _) => 0,
        }
    }
}

fn plant_variants_of(arm: Arm) -> Vec<PlantVariant> {
    match arm {
        Arm::WithoutTombstones => vec![PlantVariant::Plain],
        Arm::TodaysTombstones => vec![PlantVariant::Plain, PlantVariant::Tombstoned],
        Arm::DeathRecordForEveryKey => vec![PlantVariant::Plain, PlantVariant::WithoutItsDeathRecord],
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PositiveControl {
    kind: &'static str,
    arm: Arm,
    reader: Reader,
    tree: ReportedTree,
    expected: u64,
    /// `None`：这一格取不到可植的 key（C13），不算对上也不算对不上。
    measured: Option<u64>,
}

fn planted_unit(owner: UnitOwner, class: u64, group: Option<SelectionGroup>, birth: u64, slot: u64, mapping_key: Option<KeyFields>, contents: UnitContents) -> Unit {
    Unit {
        owner,
        class,
        group,
        write_order: (birth, MOUNT_INSTANCE, 0),
        slot,
        span_slots: 2,
        release_txg: None,
        is_readable: true,
        mapping_key,
        contents,
    }
}

/// 植入的数据单元（R-S、R-G 下 extent、分配记录、映射那三格）：身份全新（出生树取 `PLANTED_BIRTH_TREE`）。
fn planted_data_unit(object: u64, object_birth: u64, anchor_unit: u64, birth: u64, slot: u64, mapping_key: Option<KeyFields>) -> Unit {
    planted_unit(
        UnitOwner::FileData,
        UNIT_CLASS_DATA,
        Some(SelectionGroup::DataUnit {
            birth_tree: PLANTED_BIRTH_TREE,
            object,
            object_birth,
            anchor_unit,
        }),
        birth,
        slot,
        mapping_key,
        UnitContents::DataUnit {
            object,
            object_birth,
            anchor_unit,
        },
    )
}

/// 挑可植的 key 与造植入单元要的东西。
struct PlantingContext {
    births: BTreeMap<u64, u64>,
    tombstones: Vec<TombstoneRecordSummary>,
    /// 同一条臂下 R-S 重建出的 inode 树（R-G 门控看它，C13）。
    select_current_inodes: KeysWithProviders,
}

impl PlantingContext {
    fn tombstone_of(&self, object: u64, object_birth: u64) -> Option<TombstoneRecordSummary> {
        self.tombstones
            .iter()
            .find(|tombstone| tombstone.object == object && tombstone.object_birth == object_birth)
            .copied()
    }

    fn has_any_tombstone(&self, object: u64) -> bool {
        self.tombstones.iter().any(|tombstone| tombstone.object == object)
    }

    /// 植入数据单元用的对象（分配记录、映射那两格）：臂 B 有墓碑指认那一种取一条墓碑的对象；别的 R-S 取没人认的对象 0，
    /// R-G 取 R-S inode 树里一个没有墓碑的活对象（门控放得过）。
    fn object_for_a_planted_data_unit(&self, reader: Reader, variant: PlantVariant) -> Option<(u64, u64)> {
        match variant {
            PlantVariant::Tombstoned => self
                .tombstones
                .first()
                .map(|tombstone| (tombstone.object, tombstone.object_birth)),
            PlantVariant::Plain | PlantVariant::WithoutItsDeathRecord => match reader {
                Reader::SelectCurrentGatedByObject => self.select_current_inodes.keys().find_map(|fields| {
                    let inode = fields[0];
                    let birth = *self.births.get(&inode)?;
                    (!self.has_any_tombstone(inode)).then_some((inode, birth))
                }),
                Reader::UnitScan | Reader::SelectCurrent => Some((0, 0)),
            },
        }
    }

    /// 墓碑那一维的要求：有墓碑指认那一种要有对 (对象, 出生代) 的墓碑；没有墓碑指认那一种在臂 B 下要没有。
    fn tombstone_requirement_holds(&self, arm: Arm, variant: PlantVariant, object: u64, object_birth: u64) -> bool {
        let tombstoned = self.tombstone_of(object, object_birth).is_some();
        match variant {
            PlantVariant::Tombstoned => tombstoned,
            PlantVariant::Plain => arm != Arm::TodaysTombstones || !self.has_any_tombstone(object),
            PlantVariant::WithoutItsDeathRecord => true,
        }
    }
}

/// 给一条已死的 key 造植入单元（5.5 PC1 的单元类）；造不出（这一格没有这一种）交回 `None`。
fn plant_for_a_dead_key(
    reader: Reader,
    namespace: Namespace,
    variant: PlantVariant,
    arm: Arm,
    key: KeyFields,
    death: u64,
    context: &PlantingContext,
) -> Option<Unit> {
    let birth = death.checked_sub(1)?;
    let without_group = |owner: UnitOwner, class: u64, contents: UnitContents| {
        planted_unit(owner, class, None, birth, 0, None, contents)
    };
    let object_birth_of = |inode: u64| context.births.get(&inode).copied();
    match (reader, namespace) {
        (Reader::UnitScan, Namespace::Extent) => {
            let object = key[0];
            let pointer_birth = match variant {
                PlantVariant::Tombstoned => context.tombstones.iter().find(|tombstone| tombstone.object == object)?.object_birth,
                PlantVariant::Plain | PlantVariant::WithoutItsDeathRecord => {
                    if arm == Arm::TodaysTombstones && context.has_any_tombstone(object) && variant == PlantVariant::Plain {
                        return None;
                    }
                    birth
                }
            };
            Some(without_group(
                UnitOwner::ExtentLowerNode,
                UNIT_CLASS_INDEX_NODE,
                UnitContents::ExtentRecords(vec![ExtentRecordSummary {
                    inode: object,
                    data_unit: key[1],
                    pointer_birth,
                }]),
            ))
        }
        (Reader::UnitScan, Namespace::Inode) => {
            let object_birth = object_birth_of(key[0])?;
            context
                .tombstone_requirement_holds(arm, variant, key[0], object_birth)
                .then(|| {
                    without_group(
                        UnitOwner::InodeContainer,
                        UNIT_CLASS_PACKED,
                        UnitContents::InodeRecords(vec![InodeRecordSummary {
                            inode: key[0],
                            object_birth,
                            nlink: 1,
                        }]),
                    )
                })
        }
        (Reader::UnitScan, Namespace::DeletedInodes) => {
            let tombstoned = context.has_any_tombstone(key[0]);
            let holds = match variant {
                PlantVariant::Tombstoned => tombstoned,
                PlantVariant::Plain => arm != Arm::TodaysTombstones || !tombstoned,
                PlantVariant::WithoutItsDeathRecord => true,
            };
            holds.then(|| without_group(UnitOwner::DeletedInodesRoot, UNIT_CLASS_INDEX_NODE, UnitContents::DeletedInodes(vec![key[0]])))
        }
        (Reader::UnitScan, Namespace::AllocationRecordsPresent | Namespace::AllocationRecordsAllocated) => {
            (variant != PlantVariant::Tombstoned).then(|| {
                without_group(UnitOwner::AllocationNode, UNIT_CLASS_INDEX_NODE, UnitContents::AllocationRecords(vec![(key[0], false)]))
            })
        }
        (Reader::UnitScan, Namespace::Accounting) => (variant != PlantVariant::Tombstoned)
            .then(|| without_group(UnitOwner::AccountingNode, UNIT_CLASS_INDEX_NODE, UnitContents::AccountingKeys(vec![key]))),
        (Reader::UnitScan, Namespace::CentralMapping) => (variant != PlantVariant::Tombstoned)
            .then(|| without_group(UnitOwner::MappingNode, UNIT_CLASS_INDEX_NODE, UnitContents::MappingKeys(vec![key]))),
        (Reader::SelectCurrent | Reader::SelectCurrentGatedByObject, Namespace::Inode | Namespace::DeletedInodes) => {
            let object_birth = object_birth_of(key[0])?;
            if !context.tombstone_requirement_holds(arm, variant, key[0], object_birth) {
                return None;
            }
            Some(planted_unit(
                UnitOwner::InodeContainer,
                UNIT_CLASS_PACKED,
                Some(SelectionGroup::PackedContainer {
                    birth_tree: PLANTED_BIRTH_TREE,
                    record_type: PACKED_TYPE_INODE,
                    container_number: key[0],
                    container_birth: birth,
                }),
                birth,
                0,
                None,
                UnitContents::InodeRecords(vec![InodeRecordSummary {
                    inode: key[0],
                    object_birth,
                    nlink: 0,
                }]),
            ))
        }
        (Reader::SelectCurrent | Reader::SelectCurrentGatedByObject, Namespace::Extent) => {
            let object = key[0];
            let object_birth = object_birth_of(object)?;
            if reader == Reader::SelectCurrentGatedByObject
                && !context.select_current_inodes.contains_key(&single_field_key(object))
            {
                return None;
            }
            if !context.tombstone_requirement_holds(arm, variant, object, object_birth) {
                return None;
            }
            Some(planted_data_unit(object, object_birth, key[1], birth, 0, None))
        }
        (Reader::SelectCurrent | Reader::SelectCurrentGatedByObject, Namespace::AllocationUnitsOfTheVersion) => {
            let (object, object_birth) = context.object_for_a_planted_data_unit(reader, variant)?;
            Some(planted_data_unit(object, object_birth, 0, birth, key[0], None))
        }
        (Reader::SelectCurrent | Reader::SelectCurrentGatedByObject, Namespace::CentralMappingUnitsOfTheVersion) => {
            let original_birth = key[2];
            if key[0] == UNIT_CLASS_DATA {
                let (object, object_birth) = context.object_for_a_planted_data_unit(reader, variant)?;
                Some(planted_data_unit(object, object_birth, 0, original_birth, 0, Some(key)))
            } else if key[0] == UNIT_CLASS_PACKED && variant != PlantVariant::Tombstoned {
                let (owner, record_type, contents) = if key[1] == TREE_IDENTIFIER_INODE {
                    (UnitOwner::InodeContainer, PACKED_TYPE_INODE, UnitContents::InodeRecords(Vec::new()))
                } else {
                    (UnitOwner::TombstoneContainer, PACKED_TYPE_TOMBSTONE, UnitContents::TombstoneRecords(Vec::new()))
                };
                Some(planted_unit(
                    owner,
                    UNIT_CLASS_PACKED,
                    Some(SelectionGroup::PackedContainer {
                        birth_tree: PLANTED_BIRTH_TREE,
                        record_type,
                        container_number: 0,
                        container_birth: original_birth,
                    }),
                    original_birth,
                    0,
                    Some(key),
                    contents,
                ))
            } else {
                None
            }
        }
        (
            Reader::UnitScan,
            Namespace::AllocationUnitsOfTheVersion | Namespace::CentralMappingUnitsOfTheVersion | Namespace::Tombstone,
        )
        | (
            Reader::SelectCurrent | Reader::SelectCurrentGatedByObject,
            Namespace::AllocationRecordsPresent
            | Namespace::AllocationRecordsAllocated
            | Namespace::Accounting
            | Namespace::CentralMapping
            | Namespace::Tombstone,
        ) => None,
    }
}

impl Simulation {
    fn counts_for_the_hand_calculation(&self, arm: Arm, key: &NamespacedKey, provider_birth: u64, tag: OccurrenceTag) -> bool {
        match arm {
            Arm::WithoutTombstones => true,
            Arm::TodaysTombstones => {
                let tombstones = self.tombstones_on_disk();
                !object_of_tag(tag).is_some_and(|object| {
                    let of_the_object: Vec<TombstoneRecordSummary> =
                        tombstones.iter().filter(|tombstone| tombstone.object == object).copied().collect();
                    tombstone_suppresses(&of_the_object, tag)
                })
            }
            Arm::DeathRecordForEveryKey => !death_record_suppresses(&self.death_time, key, provider_birth),
        }
    }

    fn fallback_in_the_group(&self, erased: UnitId, candidates: &[UnitId]) -> Option<UnitId> {
        let group = self.disk.units[erased].group?;
        candidates
            .iter()
            .copied()
            .filter(|unit_id| *unit_id != erased && self.disk.units[*unit_id].is_readable && self.disk.units[*unit_id].group == Some(group))
            .max_by_key(|unit_id| self.disk.units[*unit_id].write_order)
    }

    /// PC2 的手算期望（C14）：抹掉的单元、它同组退回去的旧版本与基线读数里每条 key 有几个供出者，逐条算，不调扫描函数。
    fn erasure_for_the_hand_calculation(
        &self,
        reader: Reader,
        arm: Arm,
        namespace: Namespace,
        baseline: &ReadingResult,
    ) -> Option<(UnitId, u64)> {
        let truth = &self.truth[namespace.position()];
        let baseline_keys = &baseline.keys[namespace.position()];
        let only_provider = |fields: &KeyFields| baseline_keys.get(fields).is_some_and(|provider| provider.provider_count == 1);
        match reader {
            Reader::UnitScan => {
                let mut best: Option<(UnitId, u64)> = None;
                for (unit_id, unit) in self.disk.units.iter().enumerate() {
                    if !unit.is_readable || unit.release_txg.is_some() {
                        continue;
                    }
                    let live_keys = length_as_u64(
                        occurrences_of(&unit.contents)
                            .iter()
                            .filter(|occurrence| occurrence.key.namespace == namespace && truth.contains(&occurrence.key.fields))
                            .count(),
                    );
                    if live_keys > 0 && best.is_none_or(|(_, best_live)| live_keys > best_live) {
                        best = Some((unit_id, live_keys));
                    }
                }
                let (unit_id, _) = best?;
                let unit = &self.disk.units[unit_id];
                let expected = occurrences_of(&unit.contents)
                    .into_iter()
                    .filter(|occurrence| occurrence.key.namespace == namespace && truth.contains(&occurrence.key.fields))
                    .filter(|occurrence| self.counts_for_the_hand_calculation(arm, &occurrence.key, unit.birth_txg(), occurrence.tag))
                    .filter(|occurrence| only_provider(&occurrence.key.fields))
                    .count();
                Some((unit_id, length_as_u64(expected)))
            }
            Reader::SelectCurrent | Reader::SelectCurrentGatedByObject => {
                self.select_current_erasure(arm, namespace, baseline, truth)
            }
        }
    }

    fn select_current_erasure(
        &self,
        arm: Arm,
        namespace: Namespace,
        baseline: &ReadingResult,
        truth: &BTreeSet<KeyFields>,
    ) -> Option<(UnitId, u64)> {
        let baseline_keys = &baseline.keys[namespace.position()];
        let only_provider = |fields: &KeyFields| baseline_keys.get(fields).is_some_and(|provider| provider.provider_count == 1);
        let record_passes = |record: &InodeRecordSummary, provider_birth: u64| {
            let tag = OccurrenceTag::InodeRecord {
                inode: record.inode,
                object_birth: record.object_birth,
            };
            self.counts_for_the_hand_calculation(arm, &namespaced(namespace, single_field_key(record.inode)), provider_birth, tag)
        };
        match namespace {
            Namespace::Inode | Namespace::DeletedInodes => {
                let wants_queued_only = namespace == Namespace::DeletedInodes;
                let erased = if wants_queued_only {
                    let queued = *self.version.deleted_queue.first()?;
                    self.version
                        .inode_containers
                        .iter()
                        .find(|container| container.records.iter().any(|record| record.inode == queued))?
                        .unit
                } else {
                    self.version.inode_containers.iter().max_by_key(|container| container.records.len())?.unit
                };
                let fallback = self.fallback_in_the_group(erased, &self.unit_lists.inode_containers);
                let records_of = |unit_id: UnitId| {
                    if let UnitContents::InodeRecords(records) = &self.disk.units[unit_id].contents {
                        records.clone()
                    } else {
                        Vec::new()
                    }
                };
                let erased_birth = self.disk.units[erased].birth_txg();
                let fallback_records: Vec<(InodeRecordSummary, u64)> = fallback
                    .map(|unit_id| records_of(unit_id).into_iter().map(|record| (record, self.disk.units[unit_id].birth_txg())).collect())
                    .unwrap_or_default();
                let expected = records_of(erased)
                    .iter()
                    .filter(|record| !wants_queued_only || record.nlink == 0)
                    .filter(|record| record_passes(record, erased_birth))
                    .filter(|record| truth.contains(&single_field_key(record.inode)))
                    .filter(|record| only_provider(&single_field_key(record.inode)))
                    .filter(|record| {
                        !fallback_records.iter().any(|(older, older_birth)| {
                            older.inode == record.inode
                                && (!wants_queued_only || older.nlink == 0)
                                && record_passes(older, *older_birth)
                        })
                    })
                    .count();
                Some((erased, length_as_u64(expected)))
            }
            Namespace::Extent => {
                let erased = *self.version.file_units.first()?;
                let fields = extent_key(FIRST_FILE_INODE, 0);
                let fallback = self.fallback_in_the_group(erased, &self.unit_lists.data_units);
                let gate_holds = baseline.keys[Namespace::Inode.position()].contains_key(&single_field_key(FIRST_FILE_INODE));
                let data_passes = |unit_id: UnitId| {
                    let unit = &self.disk.units[unit_id];
                    let tag = OccurrenceTag::InodeRecord {
                        inode: FIRST_FILE_INODE,
                        object_birth: NEW_POOL_FILE_CREATION_TXG,
                    };
                    self.counts_for_the_hand_calculation(arm, &namespaced(Namespace::Extent, fields), unit.birth_txg(), tag)
                };
                let still_provided = fallback.is_some_and(|older| data_passes(older)) && gate_holds;
                let expected = u64::from(truth.contains(&fields) && only_provider(&fields) && !still_provided);
                Some((erased, expected))
            }
            Namespace::AllocationUnitsOfTheVersion | Namespace::CentralMappingUnitsOfTheVersion => {
                let erased = match self.version.file_units.first() {
                    Some(unit_id) => *unit_id,
                    None => self.version.inode_containers.first()?.unit,
                };
                let unit = &self.disk.units[erased];
                let fields = if namespace == Namespace::AllocationUnitsOfTheVersion {
                    single_field_key(unit.slot)
                } else {
                    unit.mapping_key?
                };
                Some((erased, u64::from(truth.contains(&fields) && only_provider(&fields))))
            }
            Namespace::Accounting => {
                let erased = *self.unit_lists.accounting_leaves.last()?;
                let expected = truth.iter().filter(|fields| only_provider(fields)).count();
                Some((erased, length_as_u64(expected)))
            }
            Namespace::AllocationRecordsPresent
            | Namespace::AllocationRecordsAllocated
            | Namespace::CentralMapping
            | Namespace::Tombstone => None,
        }
    }

    /// 最后一次扫描上，每条臂、每种读法、每棵有死亡的树各做 PC1、PC2（5.5）。
    fn positive_controls(&self) -> Vec<PositiveControl> {
        let txg = self.version.txg;
        let mut outcomes = Vec::new();
        for arm in self.write_path.arms() {
            let base_inputs = ScanInputs {
                disk: &self.disk,
                unit_lists: &self.unit_lists,
                death_time: &self.death_time,
                planted: None,
                erased: None,
            };
            let context = PlantingContext {
                births: self.inode_births(),
                tombstones: self.tombstones_on_disk(),
                select_current_inodes: select_current_reading(Reader::SelectCurrent, arm, &base_inputs).keys
                    [Namespace::Inode.position()]
                .clone(),
            };
            for reader in READERS {
                let baseline = read_with(reader, arm, &base_inputs);
                for tree in REPORTED_TREES {
                    let Some(namespace) = tree.namespace(reader) else {
                        continue;
                    };
                    let has_deaths = self
                        .trajectories
                        .get(&(arm, reader, tree))
                        .is_some_and(|trajectory| trajectory.deaths_peak > 0);
                    if !has_deaths {
                        continue;
                    }
                    let position = namespace.position();
                    let truth = &self.truth[position];
                    let baseline_measurement = measure_against_truth(&baseline.keys[position], truth, txg);
                    let selects = reader != Reader::UnitScan;
                    if selects && namespace == Namespace::Accounting {
                        let planted = planted_unit(
                            UnitOwner::AccountingNode,
                            UNIT_CLASS_INDEX_NODE,
                            None,
                            txg + 1,
                            0,
                            None,
                            UnitContents::AccountingKeys(vec![[1, TREE_IDENTIFIER_INODE, STATISTIC_WITHOUT_DEVICE, txg + 1, 0]]),
                        );
                        let inputs = ScanInputs { planted: Some(&planted), ..base_inputs };
                        let measured = measure_against_truth(&read_with(reader, arm, &inputs).keys[position], truth, txg);
                        outcomes.push(PositiveControl {
                            kind: "PC1_newer_accounting_row",
                            arm,
                            reader,
                            tree,
                            expected: 1,
                            measured: Some(increase_or_marker(measured.resurrected, baseline_measurement.resurrected)),
                        });
                    } else {
                        for variant in plant_variants_of(arm) {
                            let planted = self.ever_seen[position]
                                .difference(truth)
                                .filter(|fields| !baseline.keys[position].contains_key(*fields))
                                .find_map(|fields| {
                                    let death = *self.death_time.get(&namespaced(namespace, *fields))?;
                                    plant_for_a_dead_key(reader, namespace, variant, arm, *fields, death, &context)
                                        .map(|unit| (unit, *fields))
                                });
                            let measured = planted.map(|(unit, fields)| {
                                let mut death_time_used = None;
                                if variant == PlantVariant::WithoutItsDeathRecord {
                                    let mut without = self.death_time.clone();
                                    without.remove(&namespaced(namespace, fields));
                                    death_time_used = Some(without);
                                }
                                let inputs = ScanInputs {
                                    planted: Some(&unit),
                                    death_time: death_time_used.as_ref().unwrap_or(&self.death_time),
                                    ..base_inputs
                                };
                                let with_the_plant = measure_against_truth(&read_with(reader, arm, &inputs).keys[position], truth, txg);
                                increase_or_marker(with_the_plant.resurrected, baseline_measurement.resurrected)
                            });
                            outcomes.push(PositiveControl {
                                kind: variant.name(),
                                arm,
                                reader,
                                tree,
                                expected: variant.expected_increase(arm),
                                measured,
                            });
                        }
                    }
                    if let Some((erased, expected)) = self.erasure_for_the_hand_calculation(reader, arm, namespace, &baseline) {
                        let inputs = ScanInputs { erased: Some(erased), ..base_inputs };
                        let after = measure_against_truth(&read_with(reader, arm, &inputs).keys[position], truth, txg);
                        outcomes.push(PositiveControl {
                            kind: "PC2",
                            arm,
                            reader,
                            tree,
                            expected,
                            measured: Some(increase_or_marker(after.missed, baseline_measurement.missed)),
                        });
                    }
                }
            }
        }
        outcomes
    }
}

/// 阳性对照的增量；读数反而变小（不该发生）时交回 `u64::MAX`，让它与任何期望都对不上。
fn increase_or_marker(after: u64, before: u64) -> u64 {
    after.checked_sub(before).unwrap_or(u64::MAX)
}

// ---------------------------------------------------------------------------------------------
// 历史族（原文 5.2、修订 1 第 12.4 节）
// ---------------------------------------------------------------------------------------------

/// 带种子的确定性随机源（splitmix64）。
#[derive(Clone, Debug)]
struct SeededRandom {
    state: u64,
}

impl SeededRandom {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_value(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next_value() % bound
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeletionOrder {
    Ascending,
    EvenThenOdd,
    Random(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HistorySpec {
    /// F1 建删空 inode：n 个 inode（含 inode 1），逐个 unlink inode 2..n。
    BuildAndDeleteEmptyInodes { inode_count: u64, order: DeletionOrder },
    /// F8 大文件改写删（12.4.2）。
    LargeFileRewriteAndDelete {
        data_units: u64,
        rewrites: u64,
        overwrite_before_unlink: bool,
        batch: u64,
    },
    /// F3 混合随机。
    MixedRandom { seed: u64 },
}

/// F3 的步数（原文 5.2）。
const MIXED_RANDOM_STEPS: u64 = 300;
/// H-环锚 的覆盖写次数（12.4.4）。
const RING_ANCHOR_OVERWRITES: u64 = 22;
/// F1 每次建 50 个空 inode（原文 5.2、12.4.3）。
const NEW_INODES_PER_STEP: u64 = 50;

impl HistorySpec {
    const fn family(self) -> &'static str {
        match self {
            HistorySpec::BuildAndDeleteEmptyInodes { .. } => "F1",
            HistorySpec::LargeFileRewriteAndDelete { .. } => "F8",
            HistorySpec::MixedRandom { .. } => "F3",
        }
    }

    fn label(self) -> String {
        match self {
            HistorySpec::BuildAndDeleteEmptyInodes { inode_count, order } => {
                let order_name = match order {
                    DeletionOrder::Ascending => "ascending".to_string(),
                    DeletionOrder::EvenThenOdd => "even_then_odd".to_string(),
                    DeletionOrder::Random(seed) => format!("random_seed_{seed}"),
                };
                format!("family=F1 n={inode_count} order={order_name}")
            }
            HistorySpec::LargeFileRewriteAndDelete {
                data_units,
                rewrites,
                overwrite_before_unlink,
                batch,
            } => format!(
                "family=F8 n_u={data_units} k={rewrites} e={} batch={batch}",
                if overwrite_before_unlink { "OU" } else { "U" }
            ),
            HistorySpec::MixedRandom { seed } => format!("family=F3 seed={seed}"),
        }
    }

    const fn batch(self) -> u64 {
        match self {
            HistorySpec::LargeFileRewriteAndDelete { batch, .. } => batch,
            HistorySpec::BuildAndDeleteEmptyInodes { .. } | HistorySpec::MixedRandom { .. } => {
                PRODUCT_UNLINK_BATCH_DATA_UNITS
            }
        }
    }

    const fn is_overwrite_before_unlink(self) -> bool {
        matches!(
            self,
            HistorySpec::LargeFileRewriteAndDelete {
                overwrite_before_unlink: true,
                ..
            }
        )
    }
}

/// 第一段的 51 段：F1 14 段、F8 32 段、F3 5 段（12.5）。
fn segment_one_histories() -> Vec<HistorySpec> {
    let mut histories = Vec::new();
    for inode_count in [300, 900] {
        let mut orders = vec![DeletionOrder::Ascending, DeletionOrder::EvenThenOdd];
        orders.extend((1..=5).map(DeletionOrder::Random));
        for order in orders {
            histories.push(HistorySpec::BuildAndDeleteEmptyInodes { inode_count, order });
        }
    }
    for data_units in [1, 3, 145, 300] {
        for rewrites in [0, 5] {
            for overwrite_before_unlink in [false, true] {
                for batch in [1, PRODUCT_UNLINK_BATCH_DATA_UNITS] {
                    histories.push(HistorySpec::LargeFileRewriteAndDelete {
                        data_units,
                        rewrites,
                        overwrite_before_unlink,
                        batch,
                    });
                }
            }
        }
    }
    for seed in 1..=5 {
        histories.push(HistorySpec::MixedRandom { seed });
    }
    histories
}

fn live_inodes_of(version: &Version) -> Vec<u64> {
    version
        .inode_containers
        .iter()
        .flat_map(|container| container.records.iter())
        .filter(|record| record.nlink > 0)
        .map(|record| record.inode)
        .collect()
}

fn deletion_order_of(inode_count: u64, order: DeletionOrder) -> Vec<u64> {
    let inodes: Vec<u64> = (FIRST_FILE_INODE + 1..=inode_count).collect();
    match order {
        DeletionOrder::Ascending => inodes,
        DeletionOrder::EvenThenOdd => inodes
            .iter()
            .copied()
            .filter(|inode| inode % 2 == 0)
            .chain(inodes.iter().copied().filter(|inode| inode % 2 == 1))
            .collect(),
        DeletionOrder::Random(seed) => {
            let mut shuffled = inodes;
            let mut random = SeededRandom::new(seed);
            for position in (1..shuffled.len()).rev() {
                let other = u64_as_index(random.below(length_as_u64(position) + 1));
                shuffled.swap(position, other);
            }
            shuffled
        }
    }
}

/// 照历史驱动一段模拟；顺带判只在某个族上能判的断言（K4、A5、A10、A12）。
fn run_history(spec: HistorySpec, simulation: &mut Simulation) {
    match spec {
        HistorySpec::BuildAndDeleteEmptyInodes { inode_count, order } => {
            let mut created = 1;
            while created < inode_count {
                let count = NEW_INODES_PER_STEP.min(inode_count - created);
                simulation.apply(UserChange::NewInodes { count });
                created += count;
            }
            let containers = &simulation.version.inode_containers;
            simulation.assertions.judge(
                "K4_containers_after_building",
                length_as_u64(containers.len()) == inode_count.div_ceil(INODE_LEAF_RECORDS),
            );
            let numbers_are_first_records = containers
                .iter()
                .all(|container| container.records.first().is_some_and(|first| first.inode == container.container_number));
            simulation.assertions.judge("K4_container_number_is_its_first_record", numbers_are_first_records);
            for inode in deletion_order_of(inode_count, order) {
                let emptied_container = simulation
                    .version
                    .inode_containers
                    .iter()
                    .find(|container| container.records.len() == 1 && container.records[0].inode == inode)
                    .map(|container| container.container_number);
                simulation.apply(UserChange::Unlink { inode });
                if let Some(number) = emptied_container {
                    let still_written = simulation
                        .version
                        .inode_containers
                        .iter()
                        .any(|container| container.container_number == number);
                    simulation.assertions.judge("K4_emptied_container_not_written", !still_written);
                }
            }
        }
        HistorySpec::LargeFileRewriteAndDelete {
            data_units,
            rewrites,
            overwrite_before_unlink,
            ..
        } => {
            for _ in 0..=rewrites {
                simulation.apply(UserChange::SequentialWrite { data_units });
            }
            if overwrite_before_unlink {
                let extent_position = Namespace::Extent.position();
                let deaths_before = simulation.ever_seen[extent_position].len() - simulation.truth[extent_position].len();
                simulation.apply(UserChange::Overwrite);
                let txg = simulation.version.txg;
                let deaths_after = simulation.ever_seen[extent_position].len() - simulation.truth[extent_position].len();
                simulation.assertions.judge(
                    "A12_overwrite_kills_all_but_one_extent_key",
                    length_as_u64(deaths_after) == length_as_u64(deaths_before) + data_units - 1,
                );
                let released_data_units = simulation
                    .disk
                    .units
                    .iter()
                    .filter(|unit| unit.owner == UnitOwner::FileData && unit.release_txg == Some(txg))
                    .count();
                simulation
                    .assertions
                    .judge("A12_overwrite_releases_every_old_data_unit", length_as_u64(released_data_units) == data_units);
            }
            simulation.apply(UserChange::Unlink { inode: FIRST_FILE_INODE });
        }
        HistorySpec::MixedRandom { seed } => {
            let mut random = SeededRandom::new(seed);
            for _ in 0..MIXED_RANDOM_STEPS {
                let draw = random.below(10);
                let change = if draw < 3 {
                    UserChange::NewInodes {
                        count: 1 + random.below(20),
                    }
                } else if draw < 5 {
                    UserChange::Overwrite
                } else if draw < 6 {
                    UserChange::SequentialWrite {
                        data_units: 1 + random.below(200),
                    }
                } else {
                    let live = live_inodes_of(&simulation.version);
                    if live.is_empty() {
                        simulation.refused_steps += 1;
                        continue;
                    }
                    UserChange::Unlink {
                        inode: live[u64_as_index(random.below(length_as_u64(live.len())))],
                    }
                };
                simulation.apply(change);
            }
        }
    }
    for count in &simulation.unlink_publish_counts {
        let expected = 1 + count.data_units_at_the_intent.div_ceil(simulation.batch_data_units).max(1);
        simulation.assertions.judge("A5_A10_unlink_publishes", count.publishes == expected);
        let unlinked_inode_is_gone = !simulation.version.has_live_inode(count.inode);
        simulation.assertions.judge("unlinked_inode_stays_dead", unlinked_inode_is_gone);
    }
}

// ---------------------------------------------------------------------------------------------
// 一段历史：三个几何 × 两条写路径，各跑一遍，留摘要与产物行
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct CellSummary {
    geometry: Geometry,
    arm: Arm,
    reader: Reader,
    tree: ReportedTree,
    family: &'static str,
    is_overwrite_before_unlink: bool,
    trajectory: Trajectory,
    tree_has_units: bool,
    /// G-环 超出装置前提、G-紧 装不下的段：不算结果（12.3、原文 5.1）。
    is_excluded: bool,
}

#[derive(Clone, Debug)]
struct SimulationSummary {
    write_path: WritePath,
    lines: Vec<String>,
    cells: Vec<CellSummary>,
    positive_controls: Vec<(Geometry, PositiveControl)>,
    assertions: AssertionTally,
    placement_log: Vec<Vec<(u64, u64)>>,
    scan_log: Vec<Vec<u64>>,
    first_reuse_at: Option<u64>,
    live_slot_peak: u64,
    slots_held_during_a_publish_peak: u64,
}

fn optional_txg(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_string(), |txg| txg.to_string())
}

fn simulate(spec: HistorySpec, geometry: Geometry, write_path: WritePath, tight_unit_area_slots: Option<u64>, cross_checks_every_scan: bool) -> Simulation {
    let mut simulation = Simulation::start(geometry, write_path, spec.batch(), tight_unit_area_slots, cross_checks_every_scan);
    run_history(spec, &mut simulation);
    if !simulation.has_stopped() {
        simulation.cross_check_the_unit_scan();
    }
    simulation
}

fn summarize(spec: HistorySpec, simulation: &Simulation) -> SimulationSummary {
    let label = spec.label();
    let geometry = simulation.geometry;
    let write_path_name = match simulation.write_path {
        WritePath::WithoutTombstones => "N_P",
        WritePath::WithTombstones => "B",
    };
    let mut assertions = simulation.assertions.clone();
    assertions.judge("scans_equal_publishes", simulation.scans == simulation.publishes);
    let is_excluded = simulation.exceeded_the_premise_at.is_some() || simulation.has_stopped();
    let mut lines = vec![format!(
        "name=simulation {label} geometry={} write_path={write_path_name} publishes={} scans={} refused_steps={} units_written={} units_made_unreadable={} readable_units={} occupancy_peak={} live_slot_peak={} first_reuse_at={} reuses={} exceeded_the_premise_at={} does_not_fit_at={} unit_area_slots={}",
        geometry.name(),
        simulation.publishes,
        simulation.scans,
        simulation.refused_steps,
        simulation.disk.units_written,
        simulation.disk.units_made_unreadable,
        simulation.disk.readable_units_by_scan(),
        simulation.occupancy_peak,
        simulation.live_slot_peak,
        optional_txg(simulation.allocator.first_reuse_at),
        simulation.allocator.reuse_placements,
        optional_txg(simulation.exceeded_the_premise_at),
        optional_txg(simulation.does_not_fit_at),
        simulation.allocator.unit_area_slots.map_or_else(|| "unbounded".to_string(), |slots| slots.to_string()),
    )];
    let mut cells = Vec::new();
    for arm in simulation.write_path.arms() {
        for reader in READERS {
            for tree in REPORTED_TREES {
                let trajectory = simulation
                    .trajectories
                    .get(&(arm, reader, tree))
                    .cloned()
                    .unwrap_or_default();
                let tree_has_units = simulation.tree_kinds_with_units.contains(&tree.kind());
                let mut line = format!(
                    "name=trajectory {label} geometry={} arm={} reader={} tree={} kind_code={} publishes={} r_peak={} r_peak_at={} r_positive_scans={} r_final={} m_peak={} m_positive_scans={} m_positive_in_unlink={} m_final={} d_peak={} d_final={} tree_has_units={tree_has_units}",
                    geometry.name(),
                    arm.name(),
                    reader.name(),
                    tree.name(),
                    tree.kind().kind_code(),
                    simulation.publishes,
                    trajectory.resurrected_peak,
                    trajectory.resurrected_peak_at,
                    trajectory.resurrected_positive_scans,
                    trajectory.resurrected_final,
                    trajectory.missed_peak,
                    trajectory.missed_positive_scans,
                    trajectory.missed_positive_scans_in_unlink,
                    trajectory.missed_final,
                    trajectory.deaths_peak,
                    trajectory.deaths_final,
                );
                if geometry == Geometry::Ring {
                    line.push_str(&format!(
                        " r_in_peak={} r_in_positive_scans={} r_out_peak={} r_out_positive_scans={} first_reuse_at={} reuses={} occupancy_peak={}",
                        trajectory.in_window_peak,
                        trajectory.in_window_positive_scans,
                        trajectory.outside_window_peak,
                        trajectory.outside_window_positive_scans,
                        optional_txg(simulation.allocator.first_reuse_at),
                        simulation.allocator.reuse_placements,
                        simulation.occupancy_peak,
                    ));
                }
                lines.push(line);
                cells.push(CellSummary {
                    geometry,
                    arm,
                    reader,
                    tree,
                    family: spec.family(),
                    is_overwrite_before_unlink: spec.is_overwrite_before_unlink(),
                    trajectory,
                    tree_has_units,
                    is_excluded,
                });
            }
        }
    }
    let positive_controls = if simulation.has_stopped() {
        Vec::new()
    } else {
        simulation.positive_controls()
    };
    for control in &positive_controls {
        let (measured, matches) = match control.measured {
            Some(measured) => (measured.to_string(), (measured == control.expected).to_string()),
            None => ("not_plantable".to_string(), "not_plantable".to_string()),
        };
        lines.push(format!(
            "name=positive_control {label} geometry={} arm={} reader={} tree={} kind={} expected={} measured={measured} matches={matches}",
            geometry.name(),
            control.arm.name(),
            control.reader.name(),
            control.tree.name(),
            control.kind,
            control.expected,
        ));
    }
    let deaths_final_of = |arm: Arm, tree: ReportedTree| {
        simulation
            .trajectories
            .get(&(arm, Reader::UnitScan, tree))
            .map_or(0, |trajectory| trajectory.deaths_final)
    };
    let deaths_peak_of = |arm: Arm, tree: ReportedTree| {
        simulation
            .trajectories
            .get(&(arm, Reader::UnitScan, tree))
            .map_or(0, |trajectory| trajectory.deaths_peak)
    };
    if !simulation.has_stopped() {
        for arm in simulation.write_path.arms() {
            let accounting_versions = simulation.scans;
            assertions.judge(
                "A6_accounting_deaths_at_the_end",
                deaths_final_of(arm, ReportedTree::Accounting)
                    == (ACCOUNTING_POOL_WIDE_ROWS + ACCOUNTING_PER_DEVICE_ROWS * u64::from(DEVICE_COUNT)) * (accounting_versions - 1),
            );
            assertions.judge("tombstone_tree_never_loses_a_key", deaths_peak_of(arm, ReportedTree::Tombstone) == 0);
            match spec {
                HistorySpec::BuildAndDeleteEmptyInodes { inode_count, .. } => {
                    assertions.judge("A7_inode_deaths_at_the_end", deaths_final_of(arm, ReportedTree::Inode) == inode_count - 1);
                    assertions.judge(
                        "A7_deleted_inodes_deaths_at_the_end",
                        deaths_final_of(arm, ReportedTree::DeletedInodes) == inode_count - 1,
                    );
                    if simulation.write_path == WritePath::WithTombstones {
                        assertions.judge(
                            "K1_tombstone_containers",
                            length_as_u64(simulation.version.tombstone_containers.len())
                                == (inode_count - 1).div_ceil(TOMBSTONE_CONTAINER_RECORDS),
                        );
                    }
                }
                HistorySpec::LargeFileRewriteAndDelete { data_units, .. } => {
                    assertions.judge("A11_extent_deaths_at_the_end", deaths_final_of(arm, ReportedTree::Extent) == data_units);
                }
                HistorySpec::MixedRandom { .. } => {}
            }
        }
    }
    let unit_free_kinds_without_units = [
        TreeKind::Livelist,
        TreeKind::SparseSideTable,
        TreeKind::Deadlist,
        TreeKind::LoggedOperations,
        TreeKind::Lineage,
    ]
    .iter()
    .all(|kind| !simulation.tree_kinds_with_units.contains(kind));
    assertions.judge("kinds_6_7_8_11_12_have_no_units", unit_free_kinds_without_units);
    if geometry == Geometry::Ring && simulation.publishes >= 30 {
        assertions.judge("ring_reuses_at_least_once_in_a_long_history", simulation.allocator.first_reuse_at.is_some());
    }
    for control in &positive_controls {
        if let Some(measured) = control.measured {
            assertions.judge("positive_control_matches", measured == control.expected);
        }
    }
    let rendered: Vec<String> = assertions
        .judged
        .iter()
        .map(|(name, judged)| format!("{name}={judged}:{}", assertions.violations_of(name)))
        .collect();
    lines.push(format!(
        "name=assertions {label} geometry={} write_path={write_path_name} {}",
        geometry.name(),
        rendered.join(" ")
    ));
    SimulationSummary {
        write_path: simulation.write_path,
        lines,
        cells,
        positive_controls: positive_controls.into_iter().map(|control| (geometry, control)).collect(),
        assertions,
        placement_log: simulation.placement_log.clone(),
        scan_log: simulation.scan_log.clone(),
        first_reuse_at: simulation.allocator.first_reuse_at,
        live_slot_peak: simulation.live_slot_peak,
        slots_held_during_a_publish_peak: simulation.slots_held_during_a_publish_peak,
    }
}

#[derive(Clone, Debug)]
struct SegmentResult {
    lines: Vec<String>,
    cells: Vec<CellSummary>,
    positive_controls: Vec<(Geometry, PositiveControl)>,
    assertions: AssertionTally,
}

/// G-紧 的单元区：G-宽 下峰值活槽数的 1.25 倍，向上取偶（原文 5.1、C7）。
fn tight_unit_area_slots_from(live_slot_peak: u64) -> u64 {
    let raw = (live_slot_peak * 5).div_ceil(4);
    raw + raw % 2
}

const WRITE_PATHS: [WritePath; 2] = [WritePath::WithoutTombstones, WritePath::WithTombstones];

/// 一段历史：先 G-宽（G-紧 的盘按它定），再 G-环，再 G-紧；产物按 G-环、G-宽、G-紧 的次序（12.11 第 4 步）。
fn run_segment(spec: HistorySpec) -> SegmentResult {
    let wide: Vec<SimulationSummary> = WRITE_PATHS
        .iter()
        .map(|write_path| summarize(spec, &simulate(spec, Geometry::Wide, *write_path, None, false)))
        .collect();
    let ring: Vec<SimulationSummary> = WRITE_PATHS
        .iter()
        .map(|write_path| summarize(spec, &simulate(spec, Geometry::Ring, *write_path, None, false)))
        .collect();
    let tight: Vec<SimulationSummary> = wide
        .iter()
        .map(|wide_summary| {
            let slots = tight_unit_area_slots_from(wide_summary.slots_held_during_a_publish_peak);
            summarize(spec, &simulate(spec, Geometry::Tight, wide_summary.write_path, Some(slots), false))
        })
        .collect();
    let mut literal_tight_lines = Vec::new();
    for wide_summary in &wide {
        let literal_slots = tight_unit_area_slots_from(wide_summary.live_slot_peak);
        let literal = simulate(spec, Geometry::Tight, wide_summary.write_path, Some(literal_slots), false);
        literal_tight_lines.push(format!(
            "name=tight_literal_reading {} write_path={} live_slot_peak={} unit_area_slots={literal_slots} does_not_fit_at={} publishes={}",
            spec.label(),
            match wide_summary.write_path {
                WritePath::WithoutTombstones => "N_P",
                WritePath::WithTombstones => "B",
            },
            wide_summary.live_slot_peak,
            optional_txg(literal.does_not_fit_at),
            literal.publishes,
        ));
    }
    let mut assertions = AssertionTally::default();
    for (ring_summary, wide_summary) in ring.iter().zip(&wide) {
        let compared = ring_summary
            .first_reuse_at
            .map_or(ring_summary.placement_log.len(), |txg| u64_as_index(txg - NEW_POOL_FILE_CREATION_TXG))
            .min(ring_summary.placement_log.len())
            .min(wide_summary.placement_log.len());
        let placements_equal = ring_summary.placement_log[..compared] == wide_summary.placement_log[..compared];
        let scans_compared = compared.min(ring_summary.scan_log.len()).min(wide_summary.scan_log.len());
        let scans_equal = ring_summary.scan_log[..scans_compared] == wide_summary.scan_log[..scans_compared];
        assertions.judge("A16_ring_equals_wide_before_the_first_reuse", placements_equal && scans_equal);
    }
    let mut result = SegmentResult {
        lines: Vec::new(),
        cells: Vec::new(),
        positive_controls: Vec::new(),
        assertions,
    };
    for summary in ring.iter().chain(&wide).chain(&tight) {
        result.lines.extend(summary.lines.iter().cloned());
        result.cells.extend(summary.cells.iter().cloned());
        result.positive_controls.extend(summary.positive_controls.iter().cloned());
        result.assertions.merge(&summary.assertions);
    }
    result.lines.extend(literal_tight_lines);
    result.lines.push(format!(
        "name=segment_assertions {} A16_ring_equals_wide_before_the_first_reuse_violations={}",
        spec.label(),
        result.assertions.violations_of("A16_ring_equals_wide_before_the_first_reuse")
    ));
    result
}

/// 按段切片并行（`test-script-discipline.md`「编译与算法都要快」）：每段整段跑两遍、逐行比（C12）；按段的次序收回。
fn run_segments_in_parallel(histories: &[HistorySpec], threads: usize) -> Vec<(SegmentResult, bool)> {
    let next_segment = AtomicUsize::new(0);
    let collected: Mutex<Vec<Option<(SegmentResult, bool)>>> = Mutex::new(vec![None; histories.len()]);
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1) {
            scope.spawn(|| loop {
                let position = next_segment.fetch_add(1, Ordering::SeqCst);
                if position >= histories.len() {
                    break;
                }
                let first = run_segment(histories[position]);
                let second = run_segment(histories[position]);
                let identical = first.lines == second.lines;
                collected.lock().expect("收结果的锁没有中毒：工作线程不在持锁时 panic")[position] = Some((first, identical));
            });
        }
    });
    collected
        .into_inner()
        .expect("收结果的锁没有中毒")
        .into_iter()
        .map(|result| result.expect("每一段都收回了"))
        .collect()
}

// ---------------------------------------------------------------------------------------------
// 判定（第六节、修订 1 第 12.5 节）：门槛「峰值 > 0」，51 段取并、每个几何各判
// ---------------------------------------------------------------------------------------------

/// 判「复活」的门槛：r 峰值大于它（第六节）。
const RESURRECTION_THRESHOLD: u64 = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeStatus {
    NoObject,
    NoDeath,
    MeasuredZero,
    ZeroWithoutPositiveControl,
    ZeroButMissedLive,
    Resurrected,
    NothingJudged,
}

impl TreeStatus {
    const fn name(self) -> &'static str {
        match self {
            TreeStatus::NoObject => "no_object",
            TreeStatus::NoDeath => "no_death",
            TreeStatus::MeasuredZero => "measured_zero",
            TreeStatus::ZeroWithoutPositiveControl => "zero_without_positive_control",
            TreeStatus::ZeroButMissedLive => "zero_but_missed_live",
            TreeStatus::Resurrected => "resurrected",
            TreeStatus::NothingJudged => "nothing_judged",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PositiveControlTally {
    matched: u64,
    mismatched: u64,
    not_plantable: u64,
}

/// 一棵树在一组段上的状态（第六节四种之一，另加「r = 0 但漏活」「量到 0 而阳性对照没植成」「全被排除」）。
fn status_of(cells: &[&CellSummary], positive_controls: PositiveControlTally, threshold: u64) -> TreeStatus {
    let judged: Vec<&&CellSummary> = cells.iter().filter(|cell| !cell.is_excluded).collect();
    if judged.is_empty() {
        return TreeStatus::NothingJudged;
    }
    if !judged.iter().any(|cell| cell.tree_has_units) {
        return TreeStatus::NoObject;
    }
    if judged.iter().all(|cell| cell.trajectory.deaths_peak == 0) {
        return TreeStatus::NoDeath;
    }
    if judged.iter().any(|cell| cell.trajectory.resurrected_peak > threshold) {
        return TreeStatus::Resurrected;
    }
    if judged.iter().any(|cell| cell.trajectory.missed_peak > 0) {
        return TreeStatus::ZeroButMissedLive;
    }
    if positive_controls.matched > 0 && positive_controls.mismatched == 0 {
        TreeStatus::MeasuredZero
    } else {
        TreeStatus::ZeroWithoutPositiveControl
    }
}

/// 一个判定格：问题、几何、臂、读法、树，与只取 F8 e = OU 那 16 段与否（Q11 另列那一行）。
#[derive(Clone, Copy, Debug)]
struct VerdictCell {
    question: &'static str,
    geometry: Geometry,
    arm: Arm,
    reader: Reader,
    tree: ReportedTree,
    only_overwrite_before_unlink: bool,
}

fn eleven_tree_rows() -> Vec<ReportedTree> {
    REPORTED_TREES
        .iter()
        .copied()
        .filter(|tree| *tree != ReportedTree::CentralMapping)
        .collect()
}

fn verdict_cells() -> Vec<VerdictCell> {
    let mut cells = Vec::new();
    let mut add = |question: &'static str, geometry: Geometry, arms: &[Arm], readers: &[Reader], trees: &[ReportedTree], only_overwrite_before_unlink: bool| {
        for arm in arms {
            for reader in readers {
                for tree in trees {
                    cells.push(VerdictCell {
                        question,
                        geometry,
                        arm: *arm,
                        reader: *reader,
                        tree: *tree,
                        only_overwrite_before_unlink,
                    });
                }
            }
        }
    };
    let eleven = eleven_tree_rows();
    let mapping = [ReportedTree::CentralMapping];
    let both_selecting = [Reader::SelectCurrent, Reader::SelectCurrentGatedByObject];
    let without_tombstones = [Arm::WithoutTombstones];
    let todays_tombstones = [Arm::TodaysTombstones];
    let both_writing_arms = [Arm::WithoutTombstones, Arm::TodaysTombstones];
    add("Q1", Geometry::Wide, &without_tombstones, &[Reader::SelectCurrent], &eleven, false);
    add("Q2", Geometry::Wide, &without_tombstones, &[Reader::SelectCurrentGatedByObject], &eleven, false);
    add("Q4", Geometry::Wide, &todays_tombstones, &both_selecting, &eleven, false);
    add("Q6", Geometry::Wide, &both_writing_arms, &both_selecting, &mapping, false);
    add("Q8_Q1", Geometry::Tight, &without_tombstones, &[Reader::SelectCurrent], &eleven, false);
    add("Q8_Q2", Geometry::Tight, &without_tombstones, &[Reader::SelectCurrentGatedByObject], &eleven, false);
    add("Q8_Q4", Geometry::Tight, &todays_tombstones, &both_selecting, &eleven, false);
    add("Q8_Q6", Geometry::Tight, &both_writing_arms, &both_selecting, &mapping, false);
    add("Q9", Geometry::Ring, &without_tombstones, &[Reader::SelectCurrent], &eleven, false);
    add("Q10", Geometry::Ring, &without_tombstones, &[Reader::SelectCurrentGatedByObject], &eleven, false);
    add("Q11", Geometry::Ring, &todays_tombstones, &both_selecting, &eleven, false);
    add("Q11_OU", Geometry::Ring, &todays_tombstones, &both_selecting, &eleven, true);
    add("Q12", Geometry::Ring, &both_writing_arms, &both_selecting, &mapping, false);
    cells
}

fn positive_control_tally(controls: &[(Geometry, PositiveControl)], geometry: Geometry, arm: Arm, reader: Reader, tree: ReportedTree) -> PositiveControlTally {
    let mut tally = PositiveControlTally::default();
    for (control_geometry, control) in controls {
        if *control_geometry != geometry || control.arm != arm || control.reader != reader || control.tree != tree {
            continue;
        }
        match control.measured {
            Some(measured) if measured == control.expected => tally.matched += 1,
            Some(_) => tally.mismatched += 1,
            None => tally.not_plantable += 1,
        }
    }
    tally
}

/// 判定行与 Q5、Q13、Q14、F5、F7、F9、F10 的标注。
fn verdict_lines(results: &[(SegmentResult, bool)]) -> Vec<String> {
    let all_cells: Vec<&CellSummary> = results.iter().flat_map(|(result, _)| result.cells.iter()).collect();
    let all_controls: Vec<(Geometry, PositiveControl)> = results
        .iter()
        .flat_map(|(result, _)| result.positive_controls.iter().cloned())
        .collect();
    let mut lines = Vec::new();
    let mut status_by_cell: BTreeMap<(&'static str, Arm, Reader, ReportedTree), TreeStatus> = BTreeMap::new();
    for verdict in verdict_cells() {
        let cells: Vec<&CellSummary> = all_cells
            .iter()
            .copied()
            .filter(|cell| {
                cell.geometry == verdict.geometry
                    && cell.arm == verdict.arm
                    && cell.reader == verdict.reader
                    && cell.tree == verdict.tree
                    && (!verdict.only_overwrite_before_unlink || cell.is_overwrite_before_unlink)
            })
            .collect();
        let tally = positive_control_tally(&all_controls, verdict.geometry, verdict.arm, verdict.reader, verdict.tree);
        let status = status_of(&cells, tally, RESURRECTION_THRESHOLD);
        status_by_cell.insert((verdict.question, verdict.arm, verdict.reader, verdict.tree), status);
        let judged: Vec<&&CellSummary> = cells.iter().filter(|cell| !cell.is_excluded).collect();
        let families: BTreeSet<&'static str> = judged
            .iter()
            .filter(|cell| cell.trajectory.resurrected_peak > RESURRECTION_THRESHOLD)
            .map(|cell| cell.family)
            .collect();
        let peak = |pick: fn(&Trajectory) -> u64| judged.iter().map(|cell| pick(&cell.trajectory)).max().unwrap_or(0);
        let mut line = format!(
            "name=verdict question={} geometry={} arm={} reader={} tree={} status={} r_peak={} m_peak={} d_peak={} segments_judged={} segments_excluded={} families_with_resurrection={} pc_matched={} pc_mismatched={} pc_not_plantable={}",
            verdict.question,
            verdict.geometry.name(),
            verdict.arm.name(),
            verdict.reader.name(),
            verdict.tree.name(),
            status.name(),
            peak(|trajectory| trajectory.resurrected_peak),
            peak(|trajectory| trajectory.missed_peak),
            peak(|trajectory| trajectory.deaths_peak),
            judged.len(),
            cells.len() - judged.len(),
            if families.is_empty() {
                "none".to_string()
            } else {
                families.iter().copied().collect::<Vec<_>>().join(",")
            },
            tally.matched,
            tally.mismatched,
            tally.not_plantable,
        );
        if verdict.geometry == Geometry::Ring {
            let in_window = peak(|trajectory| trajectory.in_window_peak);
            let outside = peak(|trajectory| trajectory.outside_window_peak);
            let depends_on_placement = status == TreeStatus::Resurrected && in_window == 0 && outside > 0;
            line.push_str(&format!(" r_in_peak={in_window} r_out_peak={outside} F10_depends_on_placement={depends_on_placement}"));
        }
        if families.len() == 1 {
            line.push_str(" F7_only_one_family=true");
        }
        lines.push(line);
    }
    for (first, ring_question, tight_question) in [("Q1", "Q9", "Q8_Q1"), ("Q2", "Q10", "Q8_Q2"), ("Q4", "Q11", "Q8_Q4"), ("Q6", "Q12", "Q8_Q6")] {
        for ((question, arm, reader, tree), wide_status) in &status_by_cell {
            if *question != first {
                continue;
            }
            let ring_status = status_by_cell.get(&(ring_question, *arm, *reader, *tree)).copied();
            let tight_status = status_by_cell.get(&(tight_question, *arm, *reader, *tree)).copied();
            let same = ring_status == Some(*wide_status) && tight_status == Some(*wide_status);
            lines.push(format!(
                "name=verdict question=Q13 cells={first}/{ring_question}/{tight_question} arm={} reader={} tree={} wide={} ring={} tight={} three_point_same={same} F9_flips_with_geometry={}",
                arm.name(),
                reader.name(),
                tree.name(),
                wide_status.name(),
                ring_status.map_or("missing", TreeStatus::name),
                tight_status.map_or("missing", TreeStatus::name),
                !same,
            ));
        }
    }
    for geometry in GEOMETRIES {
        for arm in ARMS {
            for reader in READERS {
                for tree in REPORTED_TREES {
                    let cells: Vec<&&CellSummary> = all_cells
                        .iter()
                        .filter(|cell| {
                            cell.geometry == geometry && cell.arm == arm && cell.reader == reader && cell.tree == tree && !cell.is_excluded
                        })
                        .collect();
                    let missed_peak = cells.iter().map(|cell| cell.trajectory.missed_peak).max().unwrap_or(0);
                    if missed_peak == 0 {
                        continue;
                    }
                    let in_unlink: u64 = cells.iter().map(|cell| cell.trajectory.missed_positive_scans_in_unlink).sum();
                    let positive: u64 = cells.iter().map(|cell| cell.trajectory.missed_positive_scans).sum();
                    lines.push(format!(
                        "name=verdict question=Q5 geometry={} arm={} reader={} tree={} m_peak={missed_peak} m_positive_scans={positive} m_positive_scans_in_unlink={in_unlink} m_positive_scans_outside_unlink={}",
                        geometry.name(),
                        arm.name(),
                        reader.name(),
                        tree.name(),
                        positive - in_unlink,
                    ));
                }
            }
        }
    }
    lines
}

// ---------------------------------------------------------------------------------------------
// H-环锚（A14、A15、A17；对不上按 S-环2 停机）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct AnchorOutcome {
    lines: Vec<String>,
    every_anchor_holds: bool,
}

fn run_ring_anchor() -> AnchorOutcome {
    let mut simulation = Simulation::start(
        Geometry::Ring,
        WritePath::WithoutTombstones,
        PRODUCT_UNLINK_BATCH_DATA_UNITS,
        None,
        true,
    );
    let mut lines = Vec::new();
    let mut every_anchor_holds = true;
    let mut record = |assertion: &str, detail: String, holds: bool, lines: &mut Vec<String>| {
        every_anchor_holds &= holds;
        lines.push(format!("name=anchor history=H-ring-anchor assertion={assertion} {detail} holds={holds}"));
    };
    let instance_table = &simulation.disk.units[0];
    let tree_table = &simulation.disk.units[1];
    let start_holds = simulation.version.txg == 5
        && simulation.publishes == 3
        && instance_table.slot == INSTANCE_TABLE_GENESIS_SLOT
        && instance_table.span_slots == 2
        && tree_table.slot == TREE_TABLE_GENESIS_SLOT
        && tree_table.span_slots == 1;
    record(
        "A17_start",
        format!(
            "txg_after_the_mount={} publishes_from_txg_3={} instance_table_slot={} instance_table_span={} tree_table_slot={} tree_table_span={} effective_floor=0 isolated_slots=0",
            simulation.version.txg, simulation.publishes, instance_table.slot, instance_table.span_slots, tree_table.slot, tree_table.span_slots
        ),
        start_holds,
        &mut lines,
    );
    let first_file_data_slot = simulation.placement_log[0][0].0;
    record(
        "byte_table_new_pool_file_data_unit",
        format!("txg=3 slot={first_file_data_slot}"),
        first_file_data_slot == UNIT_AREA_START_SLOT + 4,
        &mut lines,
    );
    for _ in 0..RING_ANCHOR_OVERWRITES {
        simulation.apply(UserChange::Overwrite);
        let txg = simulation.version.txg;
        if txg == 25 {
            let is_free = simulation.allocator.is_free(TREE_TABLE_GENESIS_SLOT);
            record("A14_i", format!("txg=25 slot={TREE_TABLE_GENESIS_SLOT} free={is_free}"), !is_free, &mut lines);
        }
        if txg == 26 {
            let is_free = simulation.allocator.is_free(TREE_TABLE_GENESIS_SLOT);
            let reclaimed = simulation.slots_reclaimed_by_publish.get(&26).copied().unwrap_or(0);
            record(
                "A14_ii",
                format!("txg=26 slot={TREE_TABLE_GENESIS_SLOT} free={is_free} slots_reclaimed={reclaimed}"),
                is_free && reclaimed == 1,
                &mut lines,
            );
        }
        if txg == 27 {
            let data_slot = simulation.placement_log.last().expect("txg 27 发过")[0].0;
            record("A14_iii", format!("txg=27 data_unit_slot={data_slot}"), data_slot == TREE_TABLE_GENESIS_SLOT, &mut lines);
        }
    }
    let equality = simulation
        .reuse_at_exactly_the_ring_length
        .contains(&(27, TREE_TABLE_GENESIS_SLOT));
    record(
        "A15_equality",
        format!("txg=27 slot={TREE_TABLE_GENESIS_SLOT} released_at=3 ring_length={RING_LENGTH_IN_ROOTS_FOR_THE_REUSE_ASSERTION}"),
        equality,
        &mut lines,
    );
    let inequality_violations = simulation.assertions.violations_of("A15_reuse_waits_a_ring_length");
    record("A15_inequality", format!("violations={inequality_violations}"), inequality_violations == 0, &mut lines);
    let incremental_violations = simulation.assertions.violations_of("unit_scan_incremental_equals_brute_force");
    record("unit_scan_cross_check_every_scan", format!("violations={incremental_violations}"), incremental_violations == 0, &mut lines);
    AnchorOutcome {
        lines,
        every_anchor_holds,
    }
}

// ---------------------------------------------------------------------------------------------
// 作废与停机的总判定（第十一节、12.10）与主程序
// ---------------------------------------------------------------------------------------------

/// 7.2 类断言与别的作废条件：任一条有违反就整轮作废（7.1 类 K 不符走 F1，不作废）。
const VOIDING_ASSERTIONS: [&str; 14] = [
    "A2_accounting_rows_per_version",
    "A3_unit_scan_on_wide_reads_every_death",
    "A4_death_records_zero",
    "A5_A10_unlink_publishes",
    "A6_accounting_deaths_at_the_end",
    "A7_inode_deaths_at_the_end",
    "A11_extent_deaths_at_the_end",
    "A12_overwrite_kills_all_but_one_extent_key",
    "A13_death_records_zero_on_ring",
    "A16_ring_equals_wide_before_the_first_reuse",
    "A18_user_data_outside_cluster_segments",
    "A19_readable_units_equal_written_minus_overwritten",
    "positive_control_matches",
    "scans_equal_publishes",
];

const FURTHER_VOIDING_ASSERTIONS: [&str; 7] = [
    "A7_deleted_inodes_deaths_at_the_end",
    "A12_overwrite_releases_every_old_data_unit",
    "A15_reuse_waits_a_ring_length",
    "A18_opened_segment_was_empty",
    "unit_scan_incremental_equals_brute_force",
    "kinds_6_7_8_11_12_have_no_units",
    "ring_reuses_at_least_once_in_a_long_history",
];

const CLAUSE_ASSERTIONS: [&str; 5] = [
    "K1_tombstone_containers",
    "K3_tombstone_death_txg_is_the_intent_txg",
    "K4_containers_after_building",
    "K4_container_number_is_its_first_record",
    "K4_emptied_container_not_written",
];

fn summary_lines(results: &[(SegmentResult, bool)], anchor: &AnchorOutcome) -> Vec<String> {
    let mut total = AssertionTally::default();
    for (result, _) in results {
        total.merge(&result.assertions);
    }
    let mut lines = Vec::new();
    for name in VOIDING_ASSERTIONS.iter().chain(&FURTHER_VOIDING_ASSERTIONS).chain(&CLAUSE_ASSERTIONS) {
        lines.push(format!(
            "name=assertion_total assertion={name} judged={} violated={}",
            total.judged.get(name).copied().unwrap_or(0),
            total.violations_of(name)
        ));
    }
    let voiding_violations: u64 = VOIDING_ASSERTIONS
        .iter()
        .chain(&FURTHER_VOIDING_ASSERTIONS)
        .map(|name| total.violations_of(name))
        .sum();
    let clause_violations: u64 = CLAUSE_ASSERTIONS.iter().map(|name| total.violations_of(name)).sum();
    let nondeterministic = results.iter().filter(|(_, identical)| !identical).count();
    let eleven_kinds_match = ELEVEN_TREE_KINDS.len() == 11
        && ELEVEN_TREE_KINDS
            .iter()
            .map(|kind| kind.kind_code())
            .collect::<BTreeSet<u16>>()
            == [1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12].into_iter().collect::<BTreeSet<u16>>()
        && length_as_u64(ELEVEN_TREE_KINDS.len()) == TREE_TABLE_ENTRIES;
    lines.push(format!(
        "name=verdict scope=voiding voiding_assertion_violations={voiding_violations} nondeterministic_segments={nondeterministic} A1_K5_eleven_tree_kinds={eleven_kinds_match} void={}",
        voiding_violations > 0 || nondeterministic > 0 || !eleven_kinds_match
    ));
    lines.push(format!(
        "name=verdict scope=clause K_violations={clause_violations} F1_clause_may_be_wrong={}",
        clause_violations > 0
    ));
    lines.push(format!(
        "name=verdict scope=stop S_ring_2_anchors_hold={} stop={}",
        anchor.every_anchor_holds, !anchor.every_anchor_holds
    ));
    lines
}

fn threads_from_the_environment() -> usize {
    std::env::var("E177_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|threads| *threads >= 1)
        .unwrap_or(8)
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let threads = threads_from_the_environment();
    let mut emitted_lines: Vec<String> = Vec::new();
    match mode.as_str() {
        "segment-one" => {
            let histories = segment_one_histories();
            emitted_lines.push(format!(
                "name=config experiment=E177 segment=one histories={} geometries=ring,wide,tight arms=N,B,P readers=R-U,R-S,R-G reported_trees={} unit_area_start={UNIT_AREA_START_SLOT} ring_unit_area_slots={RING_GEOMETRY_UNIT_AREA_SLOTS} occupancy_cap={OCCUPANCY_CAP_SLOTS} ring_window={RING_WINDOW_IN_PUBLISHES} inode_leaf_records={INODE_LEAF_RECORDS} tombstone_container_records={TOMBSTONE_CONTAINER_RECORDS} allocation_root_level={}",
                histories.len(),
                REPORTED_TREES.len(),
                allocation_tree_root_level(),
            ));
            let anchor = run_ring_anchor();
            emitted_lines.extend(anchor.lines.iter().cloned());
            let results = run_segments_in_parallel(&histories, threads);
            for ((result, identical), spec) in results.iter().zip(&histories) {
                emitted_lines.extend(result.lines.iter().cloned());
                emitted_lines.push(format!("name=determinism {} two_runs_identical={identical}", spec.label()));
            }
            emitted_lines.extend(verdict_lines(&results));
            emitted_lines.extend(summary_lines(&results, &anchor));
        }
        "anchor" => {
            let anchor = run_ring_anchor();
            emitted_lines.extend(anchor.lines);
        }
        _ => {
            eprintln!("用法：e177-per-tree-resurrection-without-tombstones segment-one | anchor");
            std::process::exit(2);
        }
    }
    let emitted = emitted_lines.len() + 1;
    for line in &emitted_lines {
        println!("E7RESULT {line}");
    }
    println!("E7RESULT name=done emitted={emitted}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn violations(simulation: &Simulation, name: &str) -> u64 {
        simulation.assertions.violations_of(name)
    }

    fn judged(simulation: &Simulation, name: &str) -> u64 {
        simulation.assertions.judged.get(name).copied().unwrap_or(0)
    }

    fn trajectory_of(simulation: &Simulation, arm: Arm, reader: Reader, tree: ReportedTree) -> Trajectory {
        simulation
            .trajectories
            .get(&(arm, reader, tree))
            .cloned()
            .expect("每一格都有轨迹")
    }

    const SMALL_REWRITE: HistorySpec = HistorySpec::LargeFileRewriteAndDelete {
        data_units: 3,
        rewrites: 5,
        overwrite_before_unlink: true,
        batch: 1,
    };

    #[test]
    fn the_eleven_tree_kinds_are_the_records_constants_and_fill_the_tree_table() {
        let codes: BTreeSet<u16> = ELEVEN_TREE_KINDS.iter().map(|kind| kind.kind_code()).collect();
        assert_eq!(codes, [1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12].into_iter().collect::<BTreeSet<u16>>());
        assert_eq!(length_as_u64(ELEVEN_TREE_KINDS.len()), TREE_TABLE_ENTRIES);
        assert_eq!(TreeKind::CentralMapping.kind_code(), 5);
    }

    #[test]
    fn capacities_follow_the_header_formula() {
        assert_eq!(INODE_LEAF_RECORDS, 233);
        assert_eq!(TOMBSTONE_CONTAINER_RECORDS, 815);
        assert_eq!(ROOT_NODE_CONTAINER_CAPACITY, 135);
        assert_eq!(accounting_tree_capacity(), CodeTwoCapacity { leaf_entries: 477, internal_entries: 150 });
        assert_eq!(mapping_tree_capacity(), CodeTwoCapacity { leaf_entries: 294, internal_entries: 143 });
        assert_eq!(allocation_tree_root_level(), 2);
        assert_eq!(RING_GEOMETRY_UNIT_AREA_SLOTS, 211_968);
    }

    #[test]
    fn fifteen_accounting_rows_three_pool_wide_and_six_per_device() {
        let keys = accounting_keys_of(10);
        assert_eq!(length_as_u64(keys.len()), 15);
        assert!(keys.iter().all(|fields| fields[3] == 10));
    }

    fn keys_of(values: impl IntoIterator<Item = u64>) -> BTreeSet<KeyFields> {
        values.into_iter().map(single_field_key).collect()
    }

    fn leaves_of(shape: &CodeTwoShape) -> Vec<Vec<u64>> {
        shape
            .nodes
            .iter()
            .filter_map(|(_, contents)| match contents {
                CodeTwoNodeContents::Leaf { keys } => Some(keys.iter().map(|fields| fields[0]).collect()),
                CodeTwoNodeContents::Internal { .. } => None,
            })
            .collect()
    }

    #[test]
    fn nine_keys_into_a_leaf_of_eight_split_it_in_the_middle_and_grow_a_root() {
        let capacity = CodeTwoCapacity { leaf_entries: 8, internal_entries: 4 };
        let plan = plan_code_two_tree(&CodeTwoShape::default(), &keys_of(1..=9), capacity);
        assert_eq!(leaves_of(&plan.shape), vec![vec![1, 2, 3, 4, 5], vec![6, 7, 8, 9]]);
        assert_eq!(plan.shape.root_position(), Some(CodeTwoNodePosition { level: 1, index_in_level: 0 }));
        assert!(plan.origins.iter().all(|origin| *origin == CodeTwoOrigin::Rewritten));
    }

    #[test]
    fn deleting_every_key_of_a_leaf_drops_it_and_the_only_child_becomes_the_root_unrewritten() {
        let capacity = CodeTwoCapacity { leaf_entries: 8, internal_entries: 4 };
        let first = plan_code_two_tree(&CodeTwoShape::default(), &keys_of(1..=9), capacity);
        let second = plan_code_two_tree(&first.shape, &keys_of(6..=9), capacity);
        assert_eq!(leaves_of(&second.shape), vec![vec![6, 7, 8, 9]]);
        assert_eq!(second.shape.root_position(), Some(CodeTwoNodePosition { level: 0, index_in_level: 0 }));
        assert_eq!(second.origins, vec![CodeTwoOrigin::CarriedFrom(CodeTwoNodePosition { level: 0, index_in_level: 1 })]);
        assert_eq!(second.replaced_previous.len(), 2);
    }

    #[test]
    fn the_ring_anchor_reclaims_the_genesis_tree_table_at_txg_26_and_reuses_it_at_txg_27() {
        let anchor = run_ring_anchor();
        for line in &anchor.lines {
            assert!(line.ends_with("holds=true"), "{line}");
        }
        assert!(anchor.every_anchor_holds);
        assert_eq!(anchor.lines.len(), 8);
    }

    #[test]
    fn accounting_rows_die_fifteen_per_version_and_unlink_takes_one_publish_per_batch() {
        let spec = HistorySpec::LargeFileRewriteAndDelete {
            data_units: 3,
            rewrites: 0,
            overwrite_before_unlink: false,
            batch: 1,
        };
        let simulation = simulate(spec, Geometry::Wide, WritePath::WithoutTombstones, None, false);
        // 起点 3 次（txg 3、4、5）+ 顺序写 1 次 + unlink 4 次（意图 + 两批 + 最后一批）。
        assert_eq!(simulation.publishes, 8);
        assert_eq!(simulation.scans, 8);
        let accounting = trajectory_of(&simulation, Arm::WithoutTombstones, Reader::UnitScan, ReportedTree::Accounting);
        assert_eq!(accounting.deaths_final, 105);
        assert_eq!(simulation.unlink_publish_counts[0].publishes, 4);
        assert_eq!(violations(&simulation, "A2_accounting_rows_per_version"), 0);
        assert_eq!(violations(&simulation, "A5_A10_unlink_publishes"), 0);
    }

    #[test]
    fn f1_with_todays_tombstones_builds_and_drops_containers_as_the_clauses_say() {
        let spec = HistorySpec::BuildAndDeleteEmptyInodes {
            inode_count: 300,
            order: DeletionOrder::Ascending,
        };
        let simulation = simulate(spec, Geometry::Ring, WritePath::WithTombstones, None, false);
        assert_eq!(simulation.version.tombstone_containers.len(), 1);
        assert_eq!(simulation.version.tombstone_containers[0].records.len(), 299);
        assert!(judged(&simulation, "K4_emptied_container_not_written") >= 1);
        for name in [
            "K3_tombstone_death_txg_is_the_intent_txg",
            "K4_containers_after_building",
            "K4_container_number_is_its_first_record",
            "K4_emptied_container_not_written",
            "A5_A10_unlink_publishes",
            "A15_reuse_waits_a_ring_length",
        ] {
            assert_eq!(violations(&simulation, name), 0, "{name}");
        }
        let inode = trajectory_of(&simulation, Arm::TodaysTombstones, Reader::UnitScan, ReportedTree::Inode);
        assert_eq!(inode.deaths_final, 299);
        let deleted = trajectory_of(&simulation, Arm::TodaysTombstones, Reader::UnitScan, ReportedTree::DeletedInodes);
        assert_eq!(deleted.deaths_final, 299);
        let tombstone = trajectory_of(&simulation, Arm::TodaysTombstones, Reader::UnitScan, ReportedTree::Tombstone);
        assert_eq!(tombstone.deaths_peak, 0);
        // 建满 300 个时两片容器，号 1 与 234；删 234..300 之后右边那片删空不落盘。
        assert_eq!(simulation.version.inode_containers.len(), 1);
        assert_eq!(simulation.version.inode_containers[0].container_number, 1);
    }

    #[test]
    fn an_overwrite_before_unlink_kills_all_but_one_extent_key_and_releases_every_old_data_unit() {
        let simulation = simulate(SMALL_REWRITE, Geometry::Wide, WritePath::WithoutTombstones, None, false);
        assert_eq!(judged(&simulation, "A12_overwrite_kills_all_but_one_extent_key"), 1);
        assert_eq!(violations(&simulation, "A12_overwrite_kills_all_but_one_extent_key"), 0);
        assert_eq!(violations(&simulation, "A12_overwrite_releases_every_old_data_unit"), 0);
        let extent = trajectory_of(&simulation, Arm::WithoutTombstones, Reader::UnitScan, ReportedTree::Extent);
        assert_eq!(extent.deaths_final, 3);
        assert_eq!(simulation.unlink_publish_counts[0].publishes, 2);
        assert_eq!(simulation.unlink_publish_counts[0].data_units_at_the_intent, 1);
    }

    #[test]
    fn unit_scan_on_wide_reads_every_death_also_when_keys_are_reborn() {
        for spec in [
            HistorySpec::MixedRandom { seed: 1 },
            HistorySpec::BuildAndDeleteEmptyInodes {
                inode_count: 300,
                order: DeletionOrder::Random(2),
            },
        ] {
            let simulation = simulate(spec, Geometry::Wide, WritePath::WithoutTombstones, None, false);
            assert!(judged(&simulation, "A3_unit_scan_on_wide_reads_every_death") > 0);
            assert_eq!(violations(&simulation, "A3_unit_scan_on_wide_reads_every_death"), 0, "{}", spec.label());
        }
    }

    #[test]
    fn death_records_read_zero_on_every_geometry() {
        for geometry in [Geometry::Ring, Geometry::Wide] {
            for spec in [
                SMALL_REWRITE,
                HistorySpec::BuildAndDeleteEmptyInodes {
                    inode_count: 300,
                    order: DeletionOrder::EvenThenOdd,
                },
            ] {
                let simulation = simulate(spec, geometry, WritePath::WithoutTombstones, None, false);
                assert_eq!(violations(&simulation, "A4_death_records_zero"), 0, "{} {}", geometry.name(), spec.label());
                assert_eq!(violations(&simulation, "A13_death_records_zero_on_ring"), 0, "{} {}", geometry.name(), spec.label());
                assert!(judged(&simulation, "A4_death_records_zero") + judged(&simulation, "A13_death_records_zero_on_ring") > 0);
            }
        }
    }

    #[test]
    fn the_incremental_unit_scan_equals_the_brute_force_scan_at_every_scan() {
        for (spec, write_path) in [
            (
                HistorySpec::LargeFileRewriteAndDelete {
                    data_units: 145,
                    rewrites: 5,
                    overwrite_before_unlink: true,
                    batch: 1,
                },
                WritePath::WithoutTombstones,
            ),
            (
                HistorySpec::BuildAndDeleteEmptyInodes {
                    inode_count: 300,
                    order: DeletionOrder::Random(1),
                },
                WritePath::WithTombstones,
            ),
        ] {
            let simulation = simulate(spec, Geometry::Ring, write_path, None, true);
            assert!(judged(&simulation, "unit_scan_incremental_equals_brute_force") > 10);
            assert_eq!(violations(&simulation, "unit_scan_incremental_equals_brute_force"), 0, "{}", spec.label());
        }
    }

    #[test]
    fn ring_equals_wide_before_the_first_reuse_and_every_positive_control_matches() {
        let result = run_segment(SMALL_REWRITE);
        assert_eq!(result.assertions.violations_of("A16_ring_equals_wide_before_the_first_reuse"), 0);
        assert_eq!(result.assertions.judged.get("A16_ring_equals_wide_before_the_first_reuse").copied(), Some(2));
        assert_eq!(result.assertions.violations_of("positive_control_matches"), 0);
        assert!(result.positive_controls.iter().any(|(_, control)| control.measured.is_some()));
    }

    #[test]
    fn positive_controls_match_on_f1() {
        let result = run_segment(HistorySpec::BuildAndDeleteEmptyInodes {
            inode_count: 300,
            order: DeletionOrder::Ascending,
        });
        let measured: Vec<&PositiveControl> = result
            .positive_controls
            .iter()
            .map(|(_, control)| control)
            .filter(|control| control.measured.is_some())
            .collect();
        assert!(measured.len() > 50);
        for control in &measured {
            assert_eq!(control.measured, Some(control.expected), "{control:?}");
        }
        assert!(measured.iter().any(|control| control.kind == "PC2" && control.expected > 0));
        assert!(measured.iter().any(|control| control.kind == "PC1_tombstoned" && control.measured == Some(0)));
        assert!(measured.iter().any(|control| control.kind == "PC1_without_its_death_record" && control.measured == Some(1)));
    }

    #[test]
    fn user_data_never_lands_in_a_cluster_segment_of_this_mount() {
        let spec = HistorySpec::LargeFileRewriteAndDelete {
            data_units: 300,
            rewrites: 0,
            overwrite_before_unlink: false,
            batch: PRODUCT_UNLINK_BATCH_DATA_UNITS,
        };
        let simulation = simulate(spec, Geometry::Ring, WritePath::WithoutTombstones, None, false);
        assert!(judged(&simulation, "A18_user_data_outside_cluster_segments") >= 300);
        assert_eq!(violations(&simulation, "A18_user_data_outside_cluster_segments"), 0);
        assert_eq!(violations(&simulation, "A18_opened_segment_was_empty"), 0);
        // txg 6 的 300 个数据单元要越过写行那次开的段 50304：落点里有越过它的，也没有落进它的。
        let sequential_write = &simulation.placement_log[3];
        assert!(sequential_write.iter().take(300).any(|(slot, _)| *slot >= UNIT_AREA_START_SLOT + 3 * CLUSTER_SEGMENT_SLOTS));
        assert!(!sequential_write
            .iter()
            .take(300)
            .any(|(slot, _)| (UNIT_AREA_START_SLOT + 2 * CLUSTER_SEGMENT_SLOTS..UNIT_AREA_START_SLOT + 3 * CLUSTER_SEGMENT_SLOTS).contains(slot)));
    }

    #[test]
    fn readable_units_equal_written_minus_overwritten_on_the_ring_and_when_tight() {
        let mut ring = Simulation::start(Geometry::Ring, WritePath::WithoutTombstones, PRODUCT_UNLINK_BATCH_DATA_UNITS, None, false);
        for _ in 0..40 {
            ring.apply(UserChange::Overwrite);
        }
        assert!(judged(&ring, "A19_readable_units_equal_written_minus_overwritten") >= 40);
        assert_eq!(violations(&ring, "A19_readable_units_equal_written_minus_overwritten"), 0);
        assert!(ring.disk.units_made_unreadable > 0);
        let wide = simulate(SMALL_REWRITE, Geometry::Wide, WritePath::WithoutTombstones, None, false);
        let tight = simulate(
            SMALL_REWRITE,
            Geometry::Tight,
            WritePath::WithoutTombstones,
            Some(tight_unit_area_slots_from(wide.slots_held_during_a_publish_peak)),
            false,
        );
        assert_eq!(tight.does_not_fit_at, None);
        assert!(tight.disk.units_made_unreadable > 0);
        assert_eq!(wide.disk.units_made_unreadable, 0);
        assert_eq!(violations(&tight, "A19_readable_units_equal_written_minus_overwritten"), 0);
        assert_eq!(violations(&tight, "tight_reuse_only_of_released_before_this_publish"), 0);
    }

    #[test]
    fn the_ring_window_reaches_twenty_three_publishes_back_and_not_twenty_four() {
        let mut read = KeysWithProviders::new();
        let provider_released_at = |released_at: u64| Provider {
            newest_birth: 1,
            freshest_release: Some(released_at),
            has_live_provider: false,
            provider_count: 1,
        };
        read.insert(single_field_key(7), provider_released_at(100 - 23));
        read.insert(single_field_key(8), provider_released_at(100 - 24));
        let measurement = measure_against_truth(&read, &BTreeSet::new(), 100);
        assert_eq!(measurement.resurrected, 2);
        assert_eq!(measurement.resurrected_in_window, 1);
    }

    fn cell_with_peak(resurrected_peak: u64) -> CellSummary {
        CellSummary {
            geometry: Geometry::Wide,
            arm: Arm::WithoutTombstones,
            reader: Reader::SelectCurrent,
            tree: ReportedTree::Inode,
            family: "F1",
            is_overwrite_before_unlink: false,
            trajectory: Trajectory {
                resurrected_peak,
                deaths_peak: 5,
                ..Trajectory::default()
            },
            tree_has_units: true,
            is_excluded: false,
        }
    }

    #[test]
    fn moving_the_threshold_between_zero_and_the_peak_flips_the_status() {
        let cell = cell_with_peak(1);
        let tally = PositiveControlTally {
            matched: 1,
            mismatched: 0,
            not_plantable: 0,
        };
        assert_eq!(status_of(&[&cell], tally, RESURRECTION_THRESHOLD), TreeStatus::Resurrected);
        assert_eq!(status_of(&[&cell], tally, 1), TreeStatus::MeasuredZero);
        let quiet = cell_with_peak(0);
        assert_eq!(status_of(&[&quiet], tally, RESURRECTION_THRESHOLD), TreeStatus::MeasuredZero);
        assert_eq!(status_of(&[&quiet], PositiveControlTally::default(), RESURRECTION_THRESHOLD), TreeStatus::ZeroWithoutPositiveControl);
    }

    #[test]
    fn the_trajectory_keeps_the_peak_after_the_value_falls_back() {
        let mut trajectory = Trajectory::default();
        for (txg, resurrected) in [(3, 0), (4, 3), (5, 0)] {
            trajectory.observe(
                txg,
                CellMeasurement {
                    resurrected,
                    missed: resurrected,
                    resurrected_in_window: 0,
                },
                1,
                false,
            );
        }
        assert_eq!(trajectory.resurrected_peak, 3);
        assert_eq!(trajectory.resurrected_peak_at, 4);
        assert_eq!(trajectory.resurrected_positive_scans, 1);
        assert_eq!(trajectory.resurrected_final, 0);
        assert_eq!(trajectory.missed_peak, 3);
        assert_eq!(trajectory.outside_window_peak, 3);
    }

    #[test]
    fn a_tombstone_points_only_at_its_own_object_birth_and_its_pointer_range() {
        let tombstones = [TombstoneRecordSummary {
            object: 5,
            object_birth: 10,
            death_txg: 20,
        }];
        let record = |object_birth: u64| OccurrenceTag::InodeRecord { inode: 5, object_birth };
        assert!(tombstone_suppresses(&tombstones, record(10)));
        assert!(!tombstone_suppresses(&tombstones, record(11)));
        let extent = |pointer_birth: u64| OccurrenceTag::ExtentRecord { inode: 5, pointer_birth };
        assert!(!tombstone_suppresses(&tombstones, extent(9)));
        assert!(tombstone_suppresses(&tombstones, extent(10)));
        assert!(tombstone_suppresses(&tombstones, extent(20)));
        assert!(!tombstone_suppresses(&tombstones, extent(21)));
        assert!(tombstone_suppresses(&tombstones, OccurrenceTag::DeletedInode { inode: 5 }));
        assert!(!tombstone_suppresses(&tombstones, OccurrenceTag::Untagged));
    }

    fn handcrafted_unit(owner: UnitOwner, group: Option<SelectionGroup>, birth: u64, slot: u64, contents: UnitContents) -> Unit {
        Unit {
            owner,
            class: UNIT_CLASS_DATA,
            group,
            write_order: (birth, MOUNT_INSTANCE, 0),
            slot,
            span_slots: 2,
            release_txg: None,
            is_readable: true,
            mapping_key: Some([UNIT_CLASS_DATA, TREE_IDENTIFIER_EXTENT, birth, MOUNT_INSTANCE, slot]),
            contents,
        }
    }

    fn data_unit_of(object: u64, object_birth: u64, anchor_unit: u64, birth: u64, slot: u64) -> Unit {
        handcrafted_unit(
            UnitOwner::FileData,
            Some(SelectionGroup::DataUnit {
                birth_tree: TREE_IDENTIFIER_EXTENT,
                object,
                object_birth,
                anchor_unit,
            }),
            birth,
            slot,
            UnitContents::DataUnit {
                object,
                object_birth,
                anchor_unit,
            },
        )
    }

    fn inode_container_of(records: Vec<InodeRecordSummary>, birth: u64, slot: u64) -> Unit {
        handcrafted_unit(
            UnitOwner::InodeContainer,
            Some(SelectionGroup::PackedContainer {
                birth_tree: TREE_IDENTIFIER_INODE,
                record_type: PACKED_TYPE_INODE,
                container_number: 1,
                container_birth: 3,
            }),
            birth,
            slot,
            UnitContents::InodeRecords(records),
        )
    }

    fn read_handcrafted(units: Vec<Unit>, reader: Reader) -> ReadingResult {
        let mut disk = Disk::default();
        let mut lists = UnitLists::default();
        for unit in units {
            let unit_id = disk.units.len();
            match unit.owner {
                UnitOwner::FileData => lists.data_units.push(unit_id),
                UnitOwner::InodeContainer => lists.inode_containers.push(unit_id),
                UnitOwner::ExtentLowerNode
                | UnitOwner::ExtentRoot
                | UnitOwner::InodeRoot
                | UnitOwner::AllocationNode
                | UnitOwner::AccountingNode
                | UnitOwner::MappingNode
                | UnitOwner::DeletedInodesRoot
                | UnitOwner::TombstoneContainer
                | UnitOwner::TombstoneRoot
                | UnitOwner::TreeTable
                | UnitOwner::InstanceTable => {}
            }
            disk.write(unit);
        }
        let death_time = HashMap::new();
        let inputs = ScanInputs {
            disk: &disk,
            unit_lists: &lists,
            death_time: &death_time,
            planted: None,
            erased: None,
        };
        select_current_reading(reader, Arm::WithoutTombstones, &inputs)
    }

    #[test]
    fn the_gated_reader_drops_data_units_whose_object_has_no_inode_record() {
        let units = vec![data_unit_of(7, 4, 0, 4, 60_000)];
        let select_current = read_handcrafted(units.clone(), Reader::SelectCurrent);
        let gated = read_handcrafted(units, Reader::SelectCurrentGatedByObject);
        assert!(select_current.keys[Namespace::Extent.position()].contains_key(&extent_key(7, 0)));
        assert!(gated.keys[Namespace::Extent.position()].is_empty());
        let with_record = vec![
            data_unit_of(7, 4, 0, 4, 60_000),
            inode_container_of(vec![InodeRecordSummary { inode: 7, object_birth: 4, nlink: 1 }], 5, 60_010),
        ];
        let gated_with_record = read_handcrafted(with_record, Reader::SelectCurrentGatedByObject);
        assert!(gated_with_record.keys[Namespace::Extent.position()].contains_key(&extent_key(7, 0)));
    }

    #[test]
    fn select_current_takes_the_newest_version_of_a_group() {
        let units = vec![data_unit_of(1, 3, 0, 5, 60_000), data_unit_of(1, 3, 0, 7, 60_002)];
        let reading = read_handcrafted(units, Reader::SelectCurrent);
        let slots: Vec<KeyFields> = reading.keys[Namespace::AllocationUnitsOfTheVersion.position()]
            .keys()
            .copied()
            .collect();
        assert_eq!(slots, vec![single_field_key(60_002)]);
    }

    #[test]
    fn deleted_inodes_select_current_reads_the_records_with_nlink_zero() {
        let units = vec![inode_container_of(
            vec![
                InodeRecordSummary { inode: 4, object_birth: 3, nlink: 0 },
                InodeRecordSummary { inode: 5, object_birth: 3, nlink: 1 },
            ],
            6,
            60_000,
        )];
        let reading = read_handcrafted(units, Reader::SelectCurrent);
        let deleted: Vec<KeyFields> = reading.keys[Namespace::DeletedInodes.position()].keys().copied().collect();
        assert_eq!(deleted, vec![single_field_key(4)]);
        assert_eq!(reading.keys[Namespace::Inode.position()].len(), 2);
    }
}
