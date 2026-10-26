//! E187：隔离落盘代价 —— 「盘上完好的隔离」那一形的三种处置各多写多少、记号写错时各是放过还是误红、
//! 落点甲借位之后旧读者把带记号的记录读成什么（岔路单 `research/prompts/m4-i315-quarantine-mark-cost-forks.md` 那一行）。
//!
//! 跑前登记：`research/prompts/e187-preregistration.md`。三条臂（`no_mark` 不落盘、`span_bit` 丙·落点甲、`quarantine_tree` 丙·落点乙）、
//! 参照臂 `plain_release`、判据、作废与停机条款都写死在那里。独立手写的计数模型，不与 `crates/` 共用代码：
//! 登记第十一节停机条款那几项的读码核写成产物里的 `name=stop_check` 行，格式常量另从 `crates/` 源码原文里逐字核一遍。
//!
//! 用法：不带参数两段都跑（产物就是这一份输出）；`segment-one` 只跑第一段，`segment-two` 只跑第二段。
//! 线程数：`E187_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，都没设取登记的重活默认并行度（`.claude/heavy-thread-default`）；
//! 第二段按历史切片并行、按历史的次序合并，产物与线程数无关（单测 `segment_two_lines_do_not_depend_on_the_thread_count`）。
//! 一段历史内部的发布逐次串行：下一次发布读上一次的树、分配器与映射，次序就是被测对象。

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::rc::Rc;

// 写成 `#[path] mod`：单元闭包的键不收条目级 `include!`（同 E178）。
#[path = "heavy_threads_default/threads.rs"]
mod heavy_threads_default;
use heavy_threads_default::default_heavy_threads;

/// 一个槽（D3（空间分配） 已定项 7 的落点粒度）与一个码 2 节点的字节数。
const SLOT_BYTES: u64 = 16384;
const NODE_BYTES: u64 = 16384;
/// 一次单元写每个槽写多少字节：设备写的整单元字节 = 这个数 × 槽数（登记第一节「字节」）。
const UNIT_WRITE_BYTES_PER_SLOT: u64 = SLOT_BYTES;
/// 池里几块盘：每个单元写到每块盘的同一槽（w = 2 镜像）。
const DEVICE_COUNT: u64 = 2;
const JOURNAL_RECORD_BYTES: u64 = 4096;
const JOURNAL_HEADER_BYTES: u64 = 496;
const JOURNAL_NAMED_ENTRY_BYTES: u64 = 56;
/// 一条 journal 记录装几个点名项（`crates/singlefs-format/src/lib.rs` 的 `JOURNAL_NAMED_ENTRIES_PER_RECORD`，这里现算）。
const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = 64;
/// 根槽宽 = physical_block_size，取 E142 那套几何的 512（`crates/singlefs-harness/src/new_pool_file_creation_regions.rs` 文件头）；
/// 每次发布只写根环所在区域的那一块盘（`transaction.rs` 的 `WriteRootRecordForceUnitAccess`）。
const ROOT_SLOT_BYTES: u64 = 512;
const ROOT_SLOT_WRITES_PER_PUBLISH: u64 = 1;
/// 系统配置槽：每次发布每块盘写一槽（`transaction.rs` 的 `RotateSystemConfigurationSlots`）。
const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
const UNIT_AREA_START_SLOT: u64 = 50176;
const CLUSTER_SEGMENT_SLOTS: u64 = 64;
/// mkfs 那一版在单元区起点留下的两个单元（各一槽），新池新建文件之后照旧占着（只当不动的记录）。
const MAKE_FILESYSTEM_RECORD_SLOTS: [u64; 2] = [UNIT_AREA_START_SLOT, UNIT_AREA_START_SLOT + 1];
/// 分配记录树一片叶罩几个槽（W，`crates/singlefs-format/src/lib.rs` 的 `ALLOCATION_RECORD_TREE_LEAF_SLOTS`）。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
const ALLOCATION_RECORD_TREE_NODE_HEADER_BYTES: u64 = 135;
const ALLOCATION_RECORD_KEY_BYTES: u64 = 10;
#[expect(dead_code, reason = "D3 已定项 7 value 宽度备查")]
const ALLOCATION_RECORD_VALUE_BYTES: u64 = 10;
const ALLOCATION_RECORD_BYTES: u64 = 20;
/// 跨度段在条目里的偏移：紧接 key（D3（空间分配） 已定项 11「跨度段进 value」）。
const ALLOCATION_RECORD_SPAN_FIELD_OFFSET: u64 = ALLOCATION_RECORD_KEY_BYTES;
const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES: u64 = 96;
const MAPPING_NODE_HEADER_BYTES: u64 = 169;
const MAPPING_ENTRY_BYTES: u64 = 55;
const MAPPING_INTERNAL_ENTRY_BYTES: u64 = 113;
/// 乙的隔离树：条目与分配记录同宽同序（key 设备 4 + 槽号 6，value 跨度 2 + 隔离代 8），节点头 86 + 2 × 10 + 29。
const QUARANTINE_TREE_NODE_HEADER_BYTES: u64 = 135;
const QUARANTINE_TREE_ENTRY_BYTES: u64 = 20;
const QUARANTINE_TREE_INTERNAL_ENTRY_BYTES: u64 = 96;
const TREE_TABLE_HEADER_BYTES: u64 = 131;
const TREE_TABLE_ENTRY_BYTES: u64 = 200;
const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = 12;
const EXTENT_NODE_HEADER_BYTES: u64 = 163;
const EXTENT_LEAF_ENTRY_BYTES: u64 = 112;
const EXTENT_INTERNAL_ENTRY_BYTES: u64 = 110;
/// 跨度段最高位借作已释放标志（D3（空间分配） 已定项 7）。
const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
/// 落点甲借的那一位：跨度段次高位（登记第五节 `span_bit` 那一行）。
const QUARANTINE_MARK_BIT: u16 = 0x4000;
/// 几何敏感性里拿来对照的低位（登记第八节：低位让多数记录照样落叶）。
const LOW_SPAN_BIT_PROBE: u16 = 0x0004;
/// 旧读者的阳性对照：`crates/singlefs-checker-tier/src/bad_disk_input.rs` 那一条坏法的跨度段值与槽。
const OLD_READER_POSITIVE_CONTROL_SPAN_FIELD: u16 = 0x7FFF;
const OLD_READER_POSITIVE_CONTROL_SLOT: u64 = 50176;
/// 第一版合法记录的跨度只有这两种（D3（空间分配） 已定项 7「第一版恒为 1 或 2」）。
const FIRST_VERSION_SPANS: [u16; 2] = [1, 2];

/// 一个码 2 节点装几条：(16384 − 头) ÷ 条目宽 的整数部分（`crates/singlefs-core/src/unit.rs` 的 `index_node_entry_capacity`）。
const fn entries_per_node(header_bytes: u64, entry_bytes: u64) -> u64 {
    (NODE_BYTES - header_bytes) / entry_bytes
}
const ALLOCATION_RECORD_TREE_LEAF_ENTRIES: u64 = entries_per_node(ALLOCATION_RECORD_TREE_NODE_HEADER_BYTES, ALLOCATION_RECORD_BYTES);
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
const MAPPING_LEAF_ENTRIES: u64 = entries_per_node(MAPPING_NODE_HEADER_BYTES, MAPPING_ENTRY_BYTES);
const MAPPING_INTERNAL_ENTRIES: u64 = entries_per_node(MAPPING_NODE_HEADER_BYTES, MAPPING_INTERNAL_ENTRY_BYTES);
const QUARANTINE_TREE_LEAF_ENTRIES: u64 = entries_per_node(QUARANTINE_TREE_NODE_HEADER_BYTES, QUARANTINE_TREE_ENTRY_BYTES);
const QUARANTINE_TREE_INTERNAL_ENTRIES: u64 =
    entries_per_node(QUARANTINE_TREE_NODE_HEADER_BYTES, QUARANTINE_TREE_INTERNAL_ENTRY_BYTES);
const TREE_TABLE_UNIT_ENTRIES: u64 = entries_per_node(TREE_TABLE_HEADER_BYTES, TREE_TABLE_ENTRY_BYTES);
const EXTENT_LEAF_ENTRIES: u64 = entries_per_node(EXTENT_NODE_HEADER_BYTES, EXTENT_LEAF_ENTRY_BYTES);
const EXTENT_INTERNAL_ENTRIES: u64 = entries_per_node(EXTENT_NODE_HEADER_BYTES, EXTENT_INTERNAL_ENTRY_BYTES);

/// 树 ID（`crates/singlefs-format/src/lib.rs` 的 `TREE_IDENTIFIER_*`）。
const EXTENT_TREE: u64 = 65536;
const INODE_TREE: u64 = 65537;
const ALLOCATION_RECORD_TREE: u64 = 65538;
const ACCOUNTING_TREE: u64 = 65539;
/// 乙的隔离树取池里下一个没用过的树 ID（新池新建文件之后的树 ID 水位 `TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH`）：
/// 比别的树都大，它的节点在映射 key 的次序里排在别的码 2 树之后。
const QUARANTINE_TREE: u64 = 65549;
/// 映射 key 的类标签（`crates/singlefs-core/src/unit.rs` 的 `UNIT_CLASS_*`）。
const UNIT_CLASS_DATA: u8 = 1;
const UNIT_CLASS_INDEX_NODE: u8 = 2;
const UNIT_CLASS_PACKED: u8 = 3;
const WRITE_ORDER_INSTANCE: u32 = 1;

/// 工作负载（登记第五节「模型」）。
const OVERWRITES_PER_PUBLISH: usize = 4;
const NORMAL_PUBLISHES_PER_HISTORY: u64 = 10_000;
/// 密度 d（每千次发布 d 次）在 10000 次普通发布上排 10d 次隔离事件。
const QUARANTINE_EVENTS_PER_DENSITY_STEP: u64 = NORMAL_PUBLISHES_PER_HISTORY / 1000;
const ROLLBACKS_AT_RATE_ONE: u64 = 10;
const DENSITIES_PER_THOUSAND_PUBLISHES: [u64; 4] = [0, 1, 3, 10];
const ROLLBACK_RATES_PER_THOUSAND_PUBLISHES: [u64; 2] = [0, 1];
const FILE_SIZES_IN_DATA_UNITS: [u32; 2] = [200, 20000];
const CANDIDATE_WINDOWS: [u64; 2] = [8, 64];
const SEEDS: [u64; 5] = [1, 2, 3, 4, 5];
/// 固定点最多走几轮（同 `transaction.rs` 的 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`）。
const SETTLING_ROUNDS_LIMIT: usize = 64;

/// 份额的档界（登记第六节 `extra_write_share`）：档一 < 0.1%，档二 [0.1%, 1%)，档三 ≥ 1%；按十亿分之几写（判别力自证要挪到两点之间）。
const SHARE_TIER_BOUNDS_PARTS_PER_BILLION: ShareTierBounds = ShareTierBounds { tier_two_from: 1_000_000, tier_three_from: 10_000_000 };
const PARTS_PER_BILLION: i128 = 1_000_000_000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ShareTierBounds {
    tier_two_from: u64,
    tier_three_from: u64,
}

fn usize_of(value: u64) -> usize {
    usize::try_from(value).expect("模型里的计数装得进 usize")
}

fn u64_of(value: usize) -> u64 {
    u64::try_from(value).expect("模型里的计数装得进 u64")
}

/// 两个几何（登记第五节「池」）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum PoolGeometry {
    DefaultFourGibibyteDevices,
    WideOneTebibyteDevices,
}
const GEOMETRIES: [PoolGeometry; 2] = [PoolGeometry::DefaultFourGibibyteDevices, PoolGeometry::WideOneTebibyteDevices];

impl PoolGeometry {
    fn label(self) -> &'static str {
        match self {
            PoolGeometry::DefaultFourGibibyteDevices => "geometry_default",
            PoolGeometry::WideOneTebibyteDevices => "geometry_wide",
        }
    }

    fn slots_per_device(self) -> u64 {
        match self {
            PoolGeometry::DefaultFourGibibyteDevices => (4u64 << 30) / SLOT_BYTES,
            PoolGeometry::WideOneTebibyteDevices => (1024u64 << 30) / SLOT_BYTES,
        }
    }
}

/// 层级 L 上一个分配记录树节点罩几个槽：W × F^L（`allocation_record_tree.rs` 的 `span_in_slots_at_level`）。
fn span_in_slots_at_level(level: u8) -> u64 {
    (0..level).fold(ALLOCATION_RECORD_TREE_LEAF_SLOTS, |span, _| span.saturating_mul(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT))
}

/// 根的层级：最小的 R ≥ 1 使 Σ_盘 ⌈盘上槽数 ÷ S_{R−1}⌉ ≤ F（`allocation_record_tree.rs` 文件头）。
fn allocation_record_tree_root_level(slots_per_device: u64) -> u8 {
    (1u8..=u8::MAX)
        .find(|candidate| DEVICE_COUNT * slots_per_device.div_ceil(span_in_slots_at_level(candidate - 1)) <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT)
        .expect("6 字节槽号的盘在几层之内就装得进根")
}

/// 被比的三条臂与只给 `no_mark` 当阳性对照的参照臂（登记第五节「臂」）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Arm {
    NoMark,
    SpanBit,
    QuarantineTree,
    PlainRelease,
}
const FORK_ARMS: [Arm; 3] = [Arm::NoMark, Arm::SpanBit, Arm::QuarantineTree];
const MARKING_ARMS: [Arm; 2] = [Arm::SpanBit, Arm::QuarantineTree];

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Arm::NoMark => "no_mark",
            Arm::SpanBit => "span_bit",
            Arm::QuarantineTree => "quarantine_tree",
            Arm::PlainRelease => "plain_release",
        }
    }
}

/// checker 怎么认隔离：`no_mark` 照今天的副本判据；两种丙各有两个变体（登记第五节「丙的 checker」）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum CheckerUnderTest {
    Today,
    MarkOnly,
    MarkOrToday,
}

impl CheckerUnderTest {
    fn label(self) -> &'static str {
        match self {
            CheckerUnderTest::Today => "today",
            CheckerUnderTest::MarkOnly => "mark_only",
            CheckerUnderTest::MarkOrToday => "mark_or_today",
        }
    }
}

fn checkers_of(arm: Arm) -> &'static [CheckerUnderTest] {
    match arm {
        Arm::NoMark | Arm::PlainRelease => &[CheckerUnderTest::Today],
        Arm::SpanBit | Arm::QuarantineTree => &[CheckerUnderTest::MarkOnly, CheckerUnderTest::MarkOrToday],
    }
}

/// 被隔离单元两份副本在盘上的样子（登记第一节 `quarantine_intact` / `quarantine_garbled`）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum CopyCondition {
    Intact,
    Garbled,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct UnitIdentifier(u64);

/// 写量按结构分（登记第六节「分结构报」）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum WrittenStructure {
    DataUnit,
    ExtentNode,
    InodeNode,
    AllocationRecordTreeNode,
    AccountingNode,
    QuarantineTreeNode,
    CentralMappingNode,
    TreeTableUnit,
}
const WRITTEN_STRUCTURES: [WrittenStructure; 8] = [
    WrittenStructure::DataUnit,
    WrittenStructure::ExtentNode,
    WrittenStructure::InodeNode,
    WrittenStructure::AllocationRecordTreeNode,
    WrittenStructure::AccountingNode,
    WrittenStructure::QuarantineTreeNode,
    WrittenStructure::CentralMappingNode,
    WrittenStructure::TreeTableUnit,
];

impl WrittenStructure {
    fn label(self) -> &'static str {
        match self {
            WrittenStructure::DataUnit => "data",
            WrittenStructure::ExtentNode => "extent",
            WrittenStructure::InodeNode => "inode",
            WrittenStructure::AllocationRecordTreeNode => "allocation_record_tree",
            WrittenStructure::AccountingNode => "accounting",
            WrittenStructure::QuarantineTreeNode => "quarantine_tree",
            WrittenStructure::CentralMappingNode => "central_mapping",
            WrittenStructure::TreeTableUnit => "tree_table",
        }
    }

    fn position(self) -> usize {
        WRITTEN_STRUCTURES.iter().position(|structure| *structure == self).expect("每个成员都在 WRITTEN_STRUCTURES 里")
    }
}

/// 中央映射 key 的排序五段：类、出生树、出生 txg、实例、尾段（`records.rs` 的 `mapping_key_sort_key`）。
type MappingKey = (u8, u64, u64, u32, u64);
/// 乙的隔离树条目 key：(设备, 槽号)。
type QuarantineEntryKey = (u32, u64);

/// 一个单元：住哪、几槽、进不进映射（进的带 key）、算进哪一类写。
#[derive(Clone, Copy, Debug)]
struct UnitInformation {
    slot: u64,
    span_slots: u64,
    mapping_key: Option<MappingKey>,
    structure: WrittenStructure,
    birth_txg: u64,
}

/// 一条分配记录（两块盘同形，只在位翻转分类表里分盘）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct AllocationRecord {
    span_slots: u16,
    is_released: bool,
    /// 仍分配时是分配代；已释放时是释放代。
    generation: u64,
    carries_the_quarantine_mark: bool,
}

impl AllocationRecord {
    fn span_field(&self) -> u16 {
        let released_bit = if self.is_released { ALLOCATION_RECORD_RELEASED_FLAG } else { 0 };
        let mark_bit = if self.carries_the_quarantine_mark { QUARANTINE_MARK_BIT } else { 0 };
        self.span_slots | released_bit | mark_bit
    }
}

/// 分配记录树的一个节点：层级（0 是叶）、属于哪块盘、同盘同层按位置第几个；根记成层级 = 根层级、盘 0、第 0 个。
/// 派生的次序 (层级, 盘, 序号) 就是 bump 次序里「树内先叶后根、同层按 key 升序」那一段。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct AllocationNodePosition {
    level: u8,
    device: u32,
    index_in_device: u64,
}

/// 一片叶（两块盘同一个位置）及它的全部祖先与根：叶一变，父指针一路变到根（`allocation_record_tree.rs` 的 `nodes_whose_contents_changed`）。
fn leaf_and_its_ancestors(root_level: u8, leaf_index: u64, positions: &mut BTreeSet<AllocationNodePosition>) {
    for device in 0..u32::try_from(DEVICE_COUNT).expect("两块盘") {
        positions.insert(AllocationNodePosition { level: 0, device, index_in_device: leaf_index });
        for level in 1..root_level {
            let index_in_device = leaf_index * ALLOCATION_RECORD_TREE_LEAF_SLOTS / span_in_slots_at_level(level);
            positions.insert(AllocationNodePosition { level, device, index_in_device });
        }
    }
    positions.insert(AllocationNodePosition { level: root_level, device: 0, index_in_device: 0 });
}

fn leaf_of_slot(slot: u64) -> u64 {
    slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS
}

/// 码 2 树的容量（叶条目数、内部节点孩子数）。
#[derive(Clone, Copy, Debug)]
struct CodeTwoCapacity {
    leaf_entries: usize,
    internal_entries: usize,
}

fn mapping_capacity() -> CodeTwoCapacity {
    CodeTwoCapacity { leaf_entries: usize_of(MAPPING_LEAF_ENTRIES), internal_entries: usize_of(MAPPING_INTERNAL_ENTRIES) }
}

fn quarantine_tree_capacity() -> CodeTwoCapacity {
    CodeTwoCapacity { leaf_entries: usize_of(QUARANTINE_TREE_LEAF_ENTRIES), internal_entries: usize_of(QUARANTINE_TREE_INTERNAL_ENTRIES) }
}

/// 这一次发布发出去的单元号：从 `first_fresh` 起都是这次新写的，比它小的都是上一版照抄来的。
#[derive(Clone, Copy, Debug)]
struct FreshUnits {
    first_fresh: u64,
    next_unit: u64,
}

impl FreshUnits {
    fn starting_at(next_unit: u64) -> FreshUnits {
        FreshUnits { first_fresh: next_unit, next_unit }
    }

    fn issue(&mut self) -> UnitIdentifier {
        let unit = UnitIdentifier(self.next_unit);
        self.next_unit += 1;
        unit
    }

    fn is_fresh(&self, unit: UnitIdentifier) -> bool {
        unit.0 >= self.first_fresh
    }
}

/// 一棵码 2 树的一个节点（中央映射树与乙的隔离树共用这一份规则，`code_two_tree.rs` 文件头 ①–④）。
/// 节点体用 `Rc` 共享：一次发布在上一版的拷贝上改，只有改到的那几个节点真复制。
#[derive(Clone, Debug)]
struct CodeTwoNode<Key: Ord + Clone> {
    unit: UnitIdentifier,
    body: Rc<CodeTwoBody<Key>>,
}

#[derive(Clone, Debug)]
enum CodeTwoBody<Key: Ord + Clone> {
    Leaf(Vec<Key>),
    Internal { level: u8, children: Vec<CodeTwoChild<Key>> },
}

#[derive(Clone, Debug)]
struct CodeTwoChild<Key: Ord + Clone> {
    separator_key: Key,
    node: CodeTwoNode<Key>,
}

impl<Key: Ord + Clone> CodeTwoNode<Key> {
    fn level(&self) -> u8 {
        match &*self.body {
            CodeTwoBody::Leaf(_) => 0,
            CodeTwoBody::Internal { level, .. } => *level,
        }
    }

    fn entry_count(&self) -> usize {
        match &*self.body {
            CodeTwoBody::Leaf(keys) => keys.len(),
            CodeTwoBody::Internal { children, .. } => children.len(),
        }
    }

    fn smallest_key(&self) -> Option<Key> {
        match &*self.body {
            CodeTwoBody::Leaf(keys) => keys.first().cloned(),
            CodeTwoBody::Internal { children, .. } => children.first().map(|child| child.separator_key.clone()),
        }
    }

    /// 这次发布改到它：照 COW 换一个新单元（同一次发布里只换一次）。
    fn mark_rewritten(&mut self, fresh: &mut FreshUnits) {
        if !fresh.is_fresh(self.unit) {
            self.unit = fresh.issue();
        }
    }
}

/// 插入用：最后一个分隔 key ≤ key 的孩子；key 比第一个分隔 key 还小时落到第一个孩子并把它的分隔 key 压低（已定项 11 ④）。
fn route_for_insertion<Key: Ord + Clone>(children: &mut [CodeTwoChild<Key>], key: &Key) -> usize {
    match children.iter().rposition(|child| child.separator_key <= *key) {
        Some(child_index) => child_index,
        None => {
            children[0].separator_key = key.clone();
            0
        }
    }
}

fn route_for_deletion<Key: Ord + Clone>(children: &[CodeTwoChild<Key>], key: &Key) -> usize {
    children.iter().rposition(|child| child.separator_key <= *key).unwrap_or(0)
}

/// 从中间切：左半留 ⌈n ÷ 2⌉ 条，右半是新节点，分隔 key 取右半的最小 key（已定项 11 ②④）。
fn split_in_the_middle<Key: Ord + Clone>(node: &mut CodeTwoNode<Key>, fresh: &mut FreshUnits) -> CodeTwoChild<Key> {
    let right_body = match Rc::make_mut(&mut node.body) {
        CodeTwoBody::Leaf(keys) => CodeTwoBody::Leaf(keys.split_off(keys.len().div_ceil(2))),
        CodeTwoBody::Internal { level, children } => {
            CodeTwoBody::Internal { level: *level, children: children.split_off(children.len().div_ceil(2)) }
        }
    };
    let right_half = CodeTwoNode { unit: fresh.issue(), body: Rc::new(right_body) };
    CodeTwoChild { separator_key: right_half.smallest_key().expect("切出来的右半不空"), node: right_half }
}

fn insert_below<Key: Ord + Clone>(
    node: &mut CodeTwoNode<Key>,
    key: Key,
    capacity: CodeTwoCapacity,
    fresh: &mut FreshUnits,
) -> Option<CodeTwoChild<Key>> {
    node.mark_rewritten(fresh);
    let overflows = match Rc::make_mut(&mut node.body) {
        CodeTwoBody::Leaf(keys) => {
            let insertion_index = keys.partition_point(|existing| *existing < key);
            assert!(keys.get(insertion_index) != Some(&key), "插入的 key 不在树里：插入集合 = 这一版的 key 减上一版的 key");
            keys.insert(insertion_index, key);
            keys.len() > capacity.leaf_entries
        }
        CodeTwoBody::Internal { children, .. } => {
            let child_index = route_for_insertion(children, &key);
            if let Some(right_half) = insert_below(&mut children[child_index].node, key, capacity, fresh) {
                children.insert(child_index + 1, right_half);
            }
            children.len() > capacity.internal_entries
        }
    };
    overflows.then(|| split_in_the_middle(node, fresh))
}

/// 在根上插一把 key：没有根时长一片新叶；根装不下时切开、长出新根（树高 + 1）。
fn insert_at_the_root<Key: Ord + Clone>(
    root: &mut Option<CodeTwoNode<Key>>,
    key: Key,
    capacity: CodeTwoCapacity,
    fresh: &mut FreshUnits,
) {
    let Some(node) = root.as_mut() else {
        *root = Some(CodeTwoNode { unit: fresh.issue(), body: Rc::new(CodeTwoBody::Leaf(vec![key])) });
        return;
    };
    let Some(right_half) = insert_below(node, key, capacity, fresh) else {
        return;
    };
    let new_level = node.level().checked_add(1).expect("层级是 1 字节：模型里的树长不到 256 层");
    let left_half = node.clone();
    let left_separator = left_half.smallest_key().expect("刚切开的左半不空");
    *node = CodeTwoNode {
        unit: fresh.issue(),
        body: Rc::new(CodeTwoBody::Internal {
            level: new_level,
            children: vec![CodeTwoChild { separator_key: left_separator, node: left_half }, right_half],
        }),
    };
}

/// 删一把 key；删空的孩子从父节点里摘掉，不合并、不借（已定项 11 ③）。交回找没找到。
fn delete_below<Key: Ord + Clone>(node: &mut CodeTwoNode<Key>, key: &Key, fresh: &mut FreshUnits) -> bool {
    let found = match Rc::make_mut(&mut node.body) {
        CodeTwoBody::Leaf(keys) => match keys.binary_search(key) {
            Ok(index) => {
                keys.remove(index);
                true
            }
            Err(_) => false,
        },
        CodeTwoBody::Internal { children, .. } => {
            let child_index = route_for_deletion(children, key);
            let found_below = delete_below(&mut children[child_index].node, key, fresh);
            if found_below && children[child_index].node.entry_count() == 0 {
                children.remove(child_index);
            }
            found_below
        }
    };
    if found {
        node.mark_rewritten(fresh);
    }
    found
}

/// 根删完之后怎么收：根空了（零根，根指针写全零）、只剩一个孩子（孩子当根、孩子不重写）、照旧。
enum RootAfterADeletion<Key: Ord + Clone> {
    Stays,
    BecomesEmpty,
    CollapsesTo(CodeTwoNode<Key>),
}

fn delete_at_the_root<Key: Ord + Clone>(root: &mut Option<CodeTwoNode<Key>>, key: &Key, fresh: &mut FreshUnits) {
    let node = root.as_mut().expect("删的 key 在树里：树不空");
    assert!(delete_below(node, key, fresh), "删的 key 在树里：删除集合 = 上一版的 key 减这一版的 key");
    // 迭代上界：每一轮树高减一，至多根的层级那么多轮。
    loop {
        let outcome = match root.as_ref().map(|node| &*node.body) {
            None => RootAfterADeletion::Stays,
            Some(CodeTwoBody::Leaf(keys)) if keys.is_empty() => RootAfterADeletion::BecomesEmpty,
            Some(CodeTwoBody::Leaf(_)) => RootAfterADeletion::Stays,
            Some(CodeTwoBody::Internal { children, .. }) if children.is_empty() => RootAfterADeletion::BecomesEmpty,
            Some(CodeTwoBody::Internal { children, .. }) if children.len() == 1 => {
                RootAfterADeletion::CollapsesTo(children[0].node.clone())
            }
            Some(CodeTwoBody::Internal { .. }) => RootAfterADeletion::Stays,
        };
        match outcome {
            RootAfterADeletion::Stays => return,
            RootAfterADeletion::BecomesEmpty => {
                *root = None;
                return;
            }
            RootAfterADeletion::CollapsesTo(only_child) => *root = Some(only_child),
        }
    }
}

/// 这次发布之后树长什么样：上一版有、这一版没有的 key 先删（升序），再插这一版新有的（升序）（`code_two_tree.rs` 的 `plan_the_tree_after_this_publish`）。
fn plan_the_tree_after_this_publish<Key: Ord + Clone>(
    previous: &Option<CodeTwoNode<Key>>,
    deleted: &BTreeSet<Key>,
    inserted: &BTreeSet<Key>,
    capacity: CodeTwoCapacity,
    fresh: &mut FreshUnits,
) -> Option<CodeTwoNode<Key>> {
    let mut root = previous.clone();
    for key in deleted {
        delete_at_the_root(&mut root, key, fresh);
    }
    for key in inserted {
        insert_at_the_root(&mut root, key.clone(), capacity, fresh);
    }
    root
}

fn collect_units_by_level<Key: Ord + Clone>(node: &CodeTwoNode<Key>, units_by_level: &mut Vec<Vec<UnitIdentifier>>) {
    if let CodeTwoBody::Internal { children, .. } = &*node.body {
        for child in children {
            collect_units_by_level(&child.node, units_by_level);
        }
    }
    let level = usize::from(node.level());
    if units_by_level.len() <= level {
        units_by_level.resize_with(level + 1, Vec::new);
    }
    units_by_level[level].push(node.unit);
}

/// 树的全部节点按 bump 次序：先叶后根，同层按 key 升序（`code_two_tree.rs` 的 `flatten`）。
fn units_of_the_tree_in_bump_order<Key: Ord + Clone>(root: &Option<CodeTwoNode<Key>>) -> Vec<UnitIdentifier> {
    let mut units_by_level = Vec::new();
    if let Some(node) = root {
        collect_units_by_level(node, &mut units_by_level);
    }
    units_by_level.into_iter().flatten().collect()
}

/// 树里有没有这把 key（乙的 checker 读隔离树判记号齐不齐）。
fn tree_contains<Key: Ord + Clone>(root: &Option<CodeTwoNode<Key>>, key: &Key) -> bool {
    let Some(mut node) = root.as_ref() else {
        return false;
    };
    loop {
        match &*node.body {
            CodeTwoBody::Leaf(keys) => return keys.binary_search(key).is_ok(),
            CodeTwoBody::Internal { children, .. } => node = &children[route_for_deletion(children, key)].node,
        }
    }
}

fn tree_height<Key: Ord + Clone>(root: &Option<CodeTwoNode<Key>>) -> u8 {
    root.as_ref().map_or(0, |node| node.level() + 1)
}

/// 用户看得见的那一版：数据单元、extent 树、inode 树（回退取目标那一版的就是这几样，登记第五节「回退事件」）。
#[derive(Clone, Debug)]
struct UserVisibleVersion {
    data_units: Vec<UnitIdentifier>,
    extent_leaves: Vec<UnitIdentifier>,
    extent_root: Option<UnitIdentifier>,
    inode_leaf_container: UnitIdentifier,
    inode_root: UnitIdentifier,
}

impl UserVisibleVersion {
    fn units(&self) -> BTreeSet<UnitIdentifier> {
        let mut units: BTreeSet<UnitIdentifier> = self.data_units.iter().chain(self.extent_leaves.iter()).copied().collect();
        units.extend(self.extent_root);
        units.insert(self.inode_leaf_container);
        units.insert(self.inode_root);
        units
    }
}

/// 一个被隔离、记录留在已分配的单元。
#[derive(Clone, Copy, Debug)]
struct QuarantinedUnit {
    slot: u64,
    span_slots: u64,
}

/// 一条臂在一段历史里的池：分配器（位图、段、开放段、defer）、分配记录、全部单元、这一版的各棵树。
struct PoolState {
    slots_per_device: u64,
    root_level: u8,
    candidate_window: u64,
    txg: u64,
    occupied_slots: Vec<u64>,
    used_slots_per_segment: Vec<u32>,
    is_cluster_segment: Vec<bool>,
    open_segment_start: Option<u64>,
    bump_cursor: u64,
    lowest_possibly_free_slot: u64,
    lowest_possibly_empty_segment: usize,
    records: BTreeMap<u64, AllocationRecord>,
    leaves_with_records: BTreeSet<u64>,
    /// (释放代, 槽) → 跨度。
    deferred: BTreeMap<(u64, u64), u64>,
    units: Vec<Option<UnitInformation>>,
    version: Option<UserVisibleVersion>,
    allocation_nodes: BTreeMap<AllocationNodePosition, UnitIdentifier>,
    accounting: Option<UnitIdentifier>,
    tree_table: Option<UnitIdentifier>,
    mapping: Option<CodeTwoNode<MappingKey>>,
    quarantine_tree: Option<CodeTwoNode<QuarantineEntryKey>>,
    quarantine_tree_registered: bool,
    quarantined: BTreeMap<UnitIdentifier, QuarantinedUnit>,
    keeps_versions_for_rollback: bool,
    kept_versions: BTreeMap<u64, UserVisibleVersion>,
}

fn segment_of(slot: u64) -> usize {
    usize_of((slot - UNIT_AREA_START_SLOT) / CLUSTER_SEGMENT_SLOTS)
}

fn segment_start(segment: usize) -> u64 {
    UNIT_AREA_START_SLOT + u64_of(segment) * CLUSTER_SEGMENT_SLOTS
}

/// 回收门槛与 I-3.1 候选并集的下沿：最新 txg − K（登记第五节「释放与回收」，两处同一个函数）。
fn oldest_candidate_txg(newest_txg: u64, candidate_window: u64) -> u64 {
    newest_txg.saturating_sub(candidate_window)
}

impl PoolState {
    /// mkfs 之后、新池新建文件之前的池：单元区起点那一段里 mkfs 留下的两条记录，别的全空；这次挂载还没开过聚簇段。
    fn after_make_filesystem(geometry: PoolGeometry, candidate_window: u64, keeps_versions_for_rollback: bool) -> PoolState {
        let slots_per_device = geometry.slots_per_device();
        let segment_count = usize_of((slots_per_device - UNIT_AREA_START_SLOT) / CLUSTER_SEGMENT_SLOTS);
        let mut state = PoolState {
            slots_per_device,
            root_level: allocation_record_tree_root_level(slots_per_device),
            candidate_window,
            txg: 1,
            occupied_slots: vec![0; usize_of(slots_per_device.div_ceil(64))],
            used_slots_per_segment: vec![0; segment_count],
            is_cluster_segment: vec![false; segment_count],
            open_segment_start: None,
            bump_cursor: 0,
            lowest_possibly_free_slot: UNIT_AREA_START_SLOT,
            lowest_possibly_empty_segment: 0,
            records: BTreeMap::new(),
            leaves_with_records: BTreeSet::new(),
            deferred: BTreeMap::new(),
            units: Vec::new(),
            version: None,
            allocation_nodes: BTreeMap::new(),
            accounting: None,
            tree_table: None,
            mapping: None,
            quarantine_tree: None,
            quarantine_tree_registered: false,
            quarantined: BTreeMap::new(),
            keeps_versions_for_rollback,
            kept_versions: BTreeMap::new(),
        };
        for slot in MAKE_FILESYSTEM_RECORD_SLOTS {
            state.occupy_with_a_record_that_never_changes(slot, 1);
        }
        state
    }

    /// 一条从此不动的记录（mkfs 那两条；微历史里垫满一段槽位的填充记录）。
    fn occupy_with_a_record_that_never_changes(&mut self, slot: u64, span_slots: u16) {
        for covered in slot..slot + u64::from(span_slots) {
            assert!(!self.is_occupied(covered), "填充记录落在空槽上");
            self.set_occupied(covered, true);
        }
        self.records.insert(slot, AllocationRecord { span_slots, is_released: false, generation: 1, carries_the_quarantine_mark: false });
        self.leaves_with_records.insert(leaf_of_slot(slot));
    }

    fn is_occupied(&self, slot: u64) -> bool {
        self.occupied_slots[usize_of(slot / 64)] & (1u64 << (slot % 64)) != 0
    }

    fn set_occupied(&mut self, slot: u64, occupied: bool) {
        let word = &mut self.occupied_slots[usize_of(slot / 64)];
        if occupied {
            *word |= 1u64 << (slot % 64);
        } else {
            *word &= !(1u64 << (slot % 64));
        }
        let segment = segment_of(slot);
        if occupied {
            self.used_slots_per_segment[segment] += 1;
        } else {
            self.used_slots_per_segment[segment] -= 1;
            self.lowest_possibly_free_slot = self.lowest_possibly_free_slot.min(slot);
            if self.used_slots_per_segment[segment] == 0 {
                self.lowest_possibly_empty_segment = self.lowest_possibly_empty_segment.min(segment);
            }
        }
    }

    /// 从 `slot` 起第一个位图上空着的槽（只看位图，不看这次发布已经取走的）。
    fn first_unoccupied_slot_from(&self, slot: u64) -> u64 {
        let mut word_index = usize_of(slot / 64);
        let mut word = self.occupied_slots[word_index] | ((1u64 << (slot % 64)) - 1);
        while word == u64::MAX {
            word_index += 1;
            assert!(word_index < self.occupied_slots.len(), "单元区满了：模型的工作负载装不下");
            word = self.occupied_slots[word_index];
        }
        u64_of(word_index) * 64 + u64::from(word.trailing_ones())
    }

    fn unit(&self, unit: UnitIdentifier) -> UnitInformation {
        self.units[usize_of(unit.0)].expect("单元号指着一个写过的单元")
    }

    fn next_unit(&self) -> u64 {
        u64_of(self.units.len())
    }

    fn occupied_slot_count(&self) -> u64 {
        self.occupied_slots.iter().map(|word| u64::from(word.count_ones())).sum()
    }

    fn deferred_slot_count(&self) -> u64 {
        self.deferred.values().sum()
    }
}

/// 单元占几槽、按什么对齐取起点：数据单元与码 3 容器两槽 32768 对齐，索引节点一槽。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Footprint {
    OneSlot,
    TwoSlotsAligned,
}

impl Footprint {
    fn slots(self) -> u64 {
        match self {
            Footprint::OneSlot => 1,
            Footprint::TwoSlotsAligned => 2,
        }
    }
}

/// 一次发布在分配器上的取落点，只读池状态、自己记这次取走的槽（`allocator.rs` 的 `try_allocate_user_data`、
/// `try_allocate_commit_generated`、`bump_slot_in_open_segment`、`lowest_empty_segment`）：固定点每一轮从同一份池状态重走一遍。
struct PlacementWalk<'state> {
    state: &'state PoolState,
    taken: BTreeSet<u64>,
    open_segment_start: Option<u64>,
    bump_cursor: u64,
    opened_segments: BTreeSet<usize>,
    placements_that_fell_back: u64,
}

impl<'state> PlacementWalk<'state> {
    fn over(state: &'state PoolState) -> PlacementWalk<'state> {
        PlacementWalk {
            state,
            taken: BTreeSet::new(),
            open_segment_start: state.open_segment_start,
            bump_cursor: state.bump_cursor,
            opened_segments: BTreeSet::new(),
            placements_that_fell_back: 0,
        }
    }

    fn is_free(&self, slot: u64) -> bool {
        slot >= UNIT_AREA_START_SLOT && slot < self.state.slots_per_device && !self.state.is_occupied(slot) && !self.taken.contains(&slot)
    }

    /// 这次挂载开过的聚簇段（含这次发布新开的）：用户数据一个都不许落进去（`allocator.rs` 的 `cluster_segments`）。
    fn is_cluster_segment(&self, segment: usize) -> bool {
        self.state.is_cluster_segment[segment] || self.opened_segments.contains(&segment)
    }

    fn take(&mut self, slot: u64, footprint: Footprint) {
        for covered in slot..slot + footprint.slots() {
            assert!(self.is_free(covered), "取的落点 {covered} 已经占着");
            self.taken.insert(covered);
        }
    }

    /// 第一个两槽都空、偶数起点、不在任何聚簇段里的落点（D3（空间分配） 已定项 10 ③ 与 ②）。
    fn lowest_free_pair_outside_the_cluster_segments(&self) -> u64 {
        let mut candidate = self.state.first_unoccupied_slot_from(self.state.lowest_possibly_free_slot);
        loop {
            candidate += candidate % 2;
            assert!(candidate + 1 < self.state.slots_per_device, "单元区满了：模型的工作负载装不下");
            let segment = segment_of(candidate);
            if self.is_cluster_segment(segment) {
                candidate = self.state.first_unoccupied_slot_from(segment_start(segment + 1));
            } else if self.is_free(candidate) && self.is_free(candidate + 1) {
                return candidate;
            } else {
                candidate = self.state.first_unoccupied_slot_from(candidate + 1);
            }
        }
    }

    fn allocate_data_unit(&mut self) -> u64 {
        let slot = self.lowest_free_pair_outside_the_cluster_segments();
        self.take(slot, Footprint::TwoSlotsAligned);
        slot
    }

    /// 开放段里从游标起第一个整跨度都空的对齐起点；段里装不下就 `None`。
    fn bump_in_the_open_segment(&self, open_segment_start: u64, footprint: Footprint) -> Option<u64> {
        let segment_end = open_segment_start + CLUSTER_SEGMENT_SLOTS;
        let mut start = self.bump_cursor;
        if footprint == Footprint::TwoSlotsAligned && start % 2 == 1 {
            start += 1;
        }
        while start + footprint.slots() <= segment_end && (start..start + footprint.slots()).any(|slot| !self.is_free(slot)) {
            start += footprint.slots();
        }
        (start + footprint.slots() <= segment_end).then_some(start)
    }

    /// 最低的全空段：段里没有一个占着的槽（defer 里的、隔离的都算占着），这次发布也没在里面取过。
    fn lowest_empty_segment(&self) -> Option<usize> {
        (self.state.lowest_possibly_empty_segment..self.state.used_slots_per_segment.len()).find(|segment| {
            self.state.used_slots_per_segment[*segment] == 0
                && self.taken.range(segment_start(*segment)..segment_start(*segment + 1)).next().is_none()
        })
    }

    fn allocate_commit_generated(&mut self, footprint: Footprint) -> u64 {
        if let Some(open_segment_start) = self.open_segment_start {
            if let Some(slot) = self.bump_in_the_open_segment(open_segment_start, footprint) {
                self.bump_cursor = slot + footprint.slots();
                self.take(slot, footprint);
                return slot;
            }
        }
        match self.lowest_empty_segment() {
            Some(segment) => {
                let start = segment_start(segment);
                self.open_segment_start = Some(start);
                self.bump_cursor = start;
                self.opened_segments.insert(segment);
                let slot = self.bump_in_the_open_segment(start, footprint).expect("全空段 64 槽装得下任何一个单元");
                self.bump_cursor = slot + footprint.slots();
                self.take(slot, footprint);
                slot
            }
            None => {
                // 没有全空段：回落到槽号最小的空槽，这会儿没有段可 bump（`allocator.rs` 的 `FallBackToLowestFreeSlot`）。
                self.open_segment_start = None;
                self.placements_that_fell_back += 1;
                let mut candidate = self.state.first_unoccupied_slot_from(self.state.lowest_possibly_free_slot);
                while !(candidate % footprint.slots() == 0 && (candidate..candidate + footprint.slots()).all(|slot| self.is_free(slot))) {
                    candidate = self.state.first_unoccupied_slot_from(candidate + 1);
                }
                self.take(candidate, footprint);
                candidate
            }
        }
    }

    fn force(&mut self, slot: u64, footprint: Footprint) -> u64 {
        self.take(slot, footprint);
        slot
    }
}

/// 设备写次数与字节（两块盘相加；单盘口径 = 字节 ÷ 2，产物另报）。
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct WriteTally {
    device_writes: u64,
    bytes_on_both_devices: u64,
}

impl WriteTally {
    fn add(&mut self, other: WriteTally) {
        self.device_writes += other.device_writes;
        self.bytes_on_both_devices += other.bytes_on_both_devices;
    }
}

/// 一个单元写到每块盘各一次（`transaction.rs` 的 `WriteUnitToEveryDevice`）。
fn tally_of_a_unit_write(span_slots: u64) -> WriteTally {
    WriteTally { device_writes: DEVICE_COUNT, bytes_on_both_devices: DEVICE_COUNT * span_slots * UNIT_WRITE_BYTES_PER_SLOT }
}

/// 一次发布的写，按结构分；journal、根槽、系统配置槽另列。
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct PublishWrites {
    by_structure: [WriteTally; 8],
    journal: WriteTally,
    root_slot: WriteTally,
    system_configuration: WriteTally,
    journal_records: u64,
    last_transaction_named_roles: u64,
    /// 树表多出来的那一条乙的条目（逻辑字节，只当钉值）。
    logical_tree_table_bytes_added: u64,
}

impl PublishWrites {
    fn add(&mut self, other: &PublishWrites) {
        for (mine, theirs) in self.by_structure.iter_mut().zip(other.by_structure.iter()) {
            mine.add(*theirs);
        }
        self.journal.add(other.journal);
        self.root_slot.add(other.root_slot);
        self.system_configuration.add(other.system_configuration);
        self.journal_records += other.journal_records;
        self.last_transaction_named_roles += other.last_transaction_named_roles;
        self.logical_tree_table_bytes_added += other.logical_tree_table_bytes_added;
    }

    fn of(&self, structure: WrittenStructure) -> WriteTally {
        self.by_structure[structure.position()]
    }

    /// 不含根槽与系统配置槽（三臂相同的那两样，登记第一节「额外写」里不计）。
    fn without_root_and_system_configuration(&self) -> WriteTally {
        let mut total = self.journal;
        for tally in &self.by_structure {
            total.add(*tally);
        }
        total
    }

    fn with_root_and_system_configuration(&self) -> WriteTally {
        let mut total = self.without_root_and_system_configuration();
        total.add(self.root_slot);
        total.add(self.system_configuration);
        total
    }
}

/// 一次发布切成几条 journal 记录（`transaction.rs` 的 `roles_named_by_each_record_of_the_publish`）：写了 N ≥ 1 个数据单元时
/// 前 N − 1 个事务各一条只点名自己的数据单元，最后一个事务点名最后一个数据单元与全部别的重写角色、每条至多 64 个、装满再开；
/// 一个数据单元都没写时一个事务点名全部重写角色，一个都不点名也照样写一条。`padding_named_roles` 只给微历史把点名数垫到登记给的那个数。
/// 交回（记录条数、最后一个事务点名几个）。
fn journal_records_of_the_publish(data_units_written: u64, other_rewritten_roles: u64, padding_named_roles: u64) -> (u64, u64) {
    if data_units_written >= 1 {
        let last_transaction_named = 1 + other_rewritten_roles + padding_named_roles;
        (data_units_written - 1 + last_transaction_named.div_ceil(JOURNAL_NAMED_ENTRIES_PER_RECORD), last_transaction_named)
    } else {
        let last_transaction_named = other_rewritten_roles + padding_named_roles;
        (last_transaction_named.div_ceil(JOURNAL_NAMED_ENTRIES_PER_RECORD).max(1), last_transaction_named)
    }
}

fn tally_publish_writes(new_units: &[UnitInformation], padding_named_roles: u64, quarantine_tree_registered_by_this_publish: bool) -> PublishWrites {
    let mut writes = PublishWrites::default();
    for unit in new_units.iter() {
        writes.by_structure[unit.structure.position()].add(tally_of_a_unit_write(unit.span_slots));
    }
    let data_units_written = u64_of(new_units.iter().filter(|unit| unit.structure == WrittenStructure::DataUnit).count());
    let other_rewritten_roles = u64_of(new_units.len()) - data_units_written;
    let (journal_records, last_transaction_named_roles) =
        journal_records_of_the_publish(data_units_written, other_rewritten_roles, padding_named_roles);
    writes.journal = WriteTally {
        device_writes: DEVICE_COUNT * journal_records,
        bytes_on_both_devices: DEVICE_COUNT * journal_records * JOURNAL_RECORD_BYTES,
    };
    writes.root_slot = WriteTally {
        device_writes: ROOT_SLOT_WRITES_PER_PUBLISH,
        bytes_on_both_devices: ROOT_SLOT_WRITES_PER_PUBLISH * ROOT_SLOT_BYTES,
    };
    writes.system_configuration = WriteTally {
        device_writes: DEVICE_COUNT,
        bytes_on_both_devices: DEVICE_COUNT * SYSTEM_CONFIGURATION_SLOT_BYTES,
    };
    writes.journal_records = journal_records;
    writes.last_transaction_named_roles = last_transaction_named_roles;
    writes.logical_tree_table_bytes_added = if quarantine_tree_registered_by_this_publish { TREE_TABLE_ENTRY_BYTES } else { 0 };
    writes
}

/// 一次发布要做什么。
#[derive(Clone, Debug)]
enum PublishKind {
    /// 新池新建文件那一次：U 个数据单元、extent 树、inode 树全新。只做历史的起点，不计进历史的写量。
    CreateFile { data_unit_count: u32 },
    /// 普通发布：覆盖写这几个数据单元；`quarantine_victim` 是这次释放读盘核没对上的那一个；
    /// `release_quarantined` 只给微历史造一次「释放隔离记录」（`crates/` 今天没有这条路径）。
    Overwrite { data_indices: Vec<u32>, quarantine_victim: Option<u32>, release_quarantined: Option<UnitIdentifier> },
    /// 管理员回退：挂着时的一次向前发布，文件回到目标那一版（`mount.rs` 的 `roll_back_by_a_forward_publish`）。
    RollBack { target_txg: u64 },
}

/// 微历史的构造：指定某个单元落在哪一槽（不走分配器、不挪 bump 游标）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ForcedPlacement {
    DataUnit { index: u32, slot: u64 },
    NewQuarantineTreeNode { slot: u64 },
}

#[derive(Clone, Debug)]
struct PublishRequest {
    kind: PublishKind,
    forced: Vec<ForcedPlacement>,
}

impl PublishRequest {
    fn plain(kind: PublishKind) -> PublishRequest {
        PublishRequest { kind, forced: Vec::new() }
    }

    fn forced_data_slot(&self, index: u32) -> Option<u64> {
        self.forced.iter().find_map(|forced| match forced {
            ForcedPlacement::DataUnit { index: forced_index, slot } => (*forced_index == index).then_some(*slot),
            ForcedPlacement::NewQuarantineTreeNode { .. } => None,
        })
    }

    fn forced_quarantine_tree_node_slot(&self) -> Option<u64> {
        self.forced.iter().find_map(|forced| match forced {
            ForcedPlacement::DataUnit { .. } => None,
            ForcedPlacement::NewQuarantineTreeNode { slot } => Some(*slot),
        })
    }
}

/// 这次发布要放下的一个新单元。
#[derive(Clone, Copy, Debug)]
struct PlannedUnit {
    unit: UnitIdentifier,
    structure: WrittenStructure,
    footprint: Footprint,
    mapping_key: Option<MappingKey>,
    forced_slot: Option<u64>,
}

fn planned(unit: UnitIdentifier, structure: WrittenStructure, footprint: Footprint, mapping_key: Option<MappingKey>) -> PlannedUnit {
    PlannedUnit { unit, structure, footprint, mapping_key, forced_slot: None }
}

/// 与分配记录树重写集无关的那一半：用户看得见的那一版怎么变、换下谁、映射删插哪些 key、记录先改哪几条、乙的隔离树怎么变。
struct UserIntent {
    txg: u64,
    fresh: FreshUnits,
    data_units: Vec<PlannedUnit>,
    /// extent 叶、extent 根、inode 叶容器、inode 根（bump 次序里排在分配记录树之前的那几样）。
    nodes_before_the_allocation_record_tree: Vec<PlannedUnit>,
    accounting: PlannedUnit,
    quarantine_tree_after: Option<CodeTwoNode<QuarantineEntryKey>>,
    quarantine_tree_nodes: Vec<PlannedUnit>,
    tree_table: PlannedUnit,
    version_after: UserVisibleVersion,
    released_data_units: Vec<UnitIdentifier>,
    released_nodes: Vec<UnitIdentifier>,
    mapping_deleted: BTreeSet<MappingKey>,
    mapping_inserted: BTreeSet<MappingKey>,
    record_changes: BTreeMap<u64, Option<AllocationRecord>>,
    resurrected_deferred: Vec<(u64, u64)>,
    quarantine_releases: Vec<(u64, u64)>,
    newly_quarantined: Option<UnitIdentifier>,
    revived: Vec<UnitIdentifier>,
    released_quarantined: Option<UnitIdentifier>,
    quarantine_tree_registered_by_this_publish: bool,
}

fn current_record(state: &PoolState, record_changes: &BTreeMap<u64, Option<AllocationRecord>>, slot: u64) -> Option<AllocationRecord> {
    match record_changes.get(&slot) {
        Some(changed) => *changed,
        None => state.records.get(&slot).copied(),
    }
}

fn quarantine_entries_of(slot: u64) -> [QuarantineEntryKey; 2] {
    [(0, slot), (1, slot)]
}

/// 新写一组 extent 叶与 extent 根（`rewritten_leaves` 升序），映射 key 先叶后根逐树从 0 起数出生序号。
fn plan_extent_nodes(
    txg: u64,
    rewritten_leaves: &BTreeSet<usize>,
    has_root: bool,
    version_after: &mut UserVisibleVersion,
    fresh: &mut FreshUnits,
) -> Vec<PlannedUnit> {
    let mut nodes = Vec::new();
    let mut sequence = 0u64;
    for leaf in rewritten_leaves {
        let unit = fresh.issue();
        version_after.extent_leaves[*leaf] = unit;
        nodes.push(planned(unit, WrittenStructure::ExtentNode, Footprint::OneSlot, Some((UNIT_CLASS_INDEX_NODE, EXTENT_TREE, txg, WRITE_ORDER_INSTANCE, sequence))));
        sequence += 1;
    }
    if has_root {
        let unit = fresh.issue();
        version_after.extent_root = Some(unit);
        nodes.push(planned(unit, WrittenStructure::ExtentNode, Footprint::OneSlot, Some((UNIT_CLASS_INDEX_NODE, EXTENT_TREE, txg, WRITE_ORDER_INSTANCE, sequence))));
    }
    nodes
}

/// inode 树叶容器（码 3，两槽 32768 对齐）与 inode 根（码 2），每次普通发布都重写。
fn plan_inode_nodes(txg: u64, version_after: &mut UserVisibleVersion, fresh: &mut FreshUnits) -> Vec<PlannedUnit> {
    let container = fresh.issue();
    let root = fresh.issue();
    version_after.inode_leaf_container = container;
    version_after.inode_root = root;
    vec![
        planned(container, WrittenStructure::InodeNode, Footprint::TwoSlotsAligned, Some((UNIT_CLASS_PACKED, INODE_TREE, txg, WRITE_ORDER_INSTANCE, 0))),
        planned(root, WrittenStructure::InodeNode, Footprint::OneSlot, Some((UNIT_CLASS_INDEX_NODE, INODE_TREE, txg, WRITE_ORDER_INSTANCE, 1))),
    ]
}

fn extent_leaf_of(data_index: u32) -> usize {
    usize_of(u64::from(data_index) / EXTENT_LEAF_ENTRIES)
}

/// 用户看得见的那一版这次怎么变（新建文件、覆盖写、回退三种），与分配记录树的重写集无关。
fn user_intent(state: &PoolState, request: &PublishRequest, arm: Arm) -> UserIntent {
    let txg = state.txg + 1;
    let mut fresh = FreshUnits::starting_at(state.next_unit());
    let mut data_units = Vec::new();
    let mut nodes_before_the_allocation_record_tree = Vec::new();
    let mut released_data_units = Vec::new();
    let mut released_nodes = Vec::new();
    let mut mapping_deleted = BTreeSet::new();
    let mut record_changes = BTreeMap::new();
    let mut resurrected_deferred = Vec::new();
    let mut quarantine_releases = Vec::new();
    let mut newly_quarantined = None;
    let mut revived = Vec::new();
    let mut released_quarantined = None;
    let mut quarantine_tree_inserted: BTreeSet<QuarantineEntryKey> = BTreeSet::new();
    let mut quarantine_tree_deleted: BTreeSet<QuarantineEntryKey> = BTreeSet::new();
    let mut extra_mapping_inserted: BTreeSet<MappingKey> = BTreeSet::new();
    let version_after = match &request.kind {
        PublishKind::CreateFile { data_unit_count } => {
            assert!(state.version.is_none(), "新建文件只做一次");
            let leaf_count = usize_of(u64::from(*data_unit_count).div_ceil(EXTENT_LEAF_ENTRIES));
            let placeholder = UnitIdentifier(u64::MAX);
            let mut version_after = UserVisibleVersion {
                data_units: Vec::new(),
                extent_leaves: vec![placeholder; leaf_count],
                extent_root: None,
                inode_leaf_container: placeholder,
                inode_root: placeholder,
            };
            for index in 0..*data_unit_count {
                let unit = fresh.issue();
                version_after.data_units.push(unit);
                let mut data_unit = planned(unit, WrittenStructure::DataUnit, Footprint::TwoSlotsAligned, Some((UNIT_CLASS_DATA, EXTENT_TREE, txg, WRITE_ORDER_INSTANCE, u64::from(index))));
                data_unit.forced_slot = request.forced_data_slot(index);
                data_units.push(data_unit);
            }
            let all_leaves: BTreeSet<usize> = (0..leaf_count).collect();
            nodes_before_the_allocation_record_tree.extend(plan_extent_nodes(txg, &all_leaves, leaf_count > 1, &mut version_after, &mut fresh));
            nodes_before_the_allocation_record_tree.extend(plan_inode_nodes(txg, &mut version_after, &mut fresh));
            version_after
        }
        PublishKind::Overwrite { data_indices, quarantine_victim, release_quarantined } => {
            let before = state.version.as_ref().expect("覆盖写之前文件已经建好");
            let mut version_after = before.clone();
            let sorted_indices: BTreeSet<u32> = data_indices.iter().copied().collect();
            assert_eq!(sorted_indices.len(), data_indices.len(), "一次发布里同一个数据单元只覆盖写一次");
            for (sequence, index) in sorted_indices.iter().enumerate() {
                let old_unit = before.data_units[usize_of(u64::from(*index))];
                mapping_deleted.insert(state.unit(old_unit).mapping_key.expect("数据单元进映射"));
                let unit = fresh.issue();
                version_after.data_units[usize_of(u64::from(*index))] = unit;
                let mut data_unit = planned(unit, WrittenStructure::DataUnit, Footprint::TwoSlotsAligned, Some((UNIT_CLASS_DATA, EXTENT_TREE, txg, WRITE_ORDER_INSTANCE, u64_of(sequence))));
                data_unit.forced_slot = request.forced_data_slot(*index);
                data_units.push(data_unit);
                let is_the_victim = *quarantine_victim == Some(*index);
                match (is_the_victim, arm) {
                    (true, Arm::NoMark | Arm::SpanBit | Arm::QuarantineTree) => {
                        // 释放之前读盘核没对上：两块盘上的记录都留在已分配，映射条目照常去掉（D19 已定项 5 硬规则 1）。
                        let old_information = state.unit(old_unit);
                        newly_quarantined = Some(old_unit);
                        let mut quarantined_record = state.records.get(&old_information.slot).copied().expect("被隔离单元有记录");
                        match arm {
                            Arm::SpanBit => quarantined_record.carries_the_quarantine_mark = true,
                            Arm::QuarantineTree => quarantine_tree_inserted.extend(quarantine_entries_of(old_information.slot)),
                            Arm::NoMark | Arm::PlainRelease => {}
                        }
                        record_changes.insert(old_information.slot, Some(quarantined_record));
                    }
                    (true, Arm::PlainRelease) | (false, Arm::NoMark | Arm::SpanBit | Arm::QuarantineTree | Arm::PlainRelease) => {
                        released_data_units.push(old_unit);
                    }
                }
            }
            let rewritten_leaves: BTreeSet<usize> = sorted_indices.iter().map(|index| extent_leaf_of(*index)).collect();
            for leaf in &rewritten_leaves {
                released_nodes.push(before.extent_leaves[*leaf]);
            }
            released_nodes.extend(before.extent_root);
            released_nodes.push(before.inode_leaf_container);
            released_nodes.push(before.inode_root);
            for replaced_node in &released_nodes {
                mapping_deleted.insert(state.unit(*replaced_node).mapping_key.expect("extent 与 inode 的节点进映射"));
            }
            nodes_before_the_allocation_record_tree.extend(plan_extent_nodes(txg, &rewritten_leaves, before.extent_root.is_some(), &mut version_after, &mut fresh));
            nodes_before_the_allocation_record_tree.extend(plan_inode_nodes(txg, &mut version_after, &mut fresh));
            if let Some(unit) = release_quarantined {
                // 释放一个隔离的记录：两块盘上那条改成已释放 + 释放代，落点进 defer，记号随之清掉（甲）或从隔离树里删掉（乙）。
                let quarantined = state.quarantined.get(unit).copied().expect("释放的是一个隔离着的单元");
                let mut released_record = state.records.get(&quarantined.slot).copied().expect("隔离的记录在");
                released_record.is_released = true;
                released_record.generation = txg;
                released_record.carries_the_quarantine_mark = false;
                record_changes.insert(quarantined.slot, Some(released_record));
                quarantine_releases.push((quarantined.slot, quarantined.span_slots));
                released_quarantined = Some(*unit);
                if arm == Arm::QuarantineTree {
                    quarantine_tree_deleted.extend(quarantine_entries_of(quarantined.slot));
                }
            }
            version_after
        }
        PublishKind::RollBack { target_txg } => {
            let before = state.version.as_ref().expect("回退之前文件已经建好");
            let target = state.kept_versions.get(target_txg).expect("回退目标在候选窗口里、那一版留着").clone();
            let current_units = before.units();
            let target_units = target.units();
            // 现行那一版引用、目标不引用的释放；目标引用、还在 defer 里的复活；目标引用、隔离着的那几个重新被引用（记号清掉）。
            for unit in current_units.difference(&target_units) {
                mapping_deleted.insert(state.unit(*unit).mapping_key.expect("用户看得见的单元都进映射"));
                match state.unit(*unit).structure {
                    WrittenStructure::DataUnit => released_data_units.push(*unit),
                    WrittenStructure::ExtentNode
                    | WrittenStructure::InodeNode
                    | WrittenStructure::AllocationRecordTreeNode
                    | WrittenStructure::AccountingNode
                    | WrittenStructure::QuarantineTreeNode
                    | WrittenStructure::CentralMappingNode
                    | WrittenStructure::TreeTableUnit => released_nodes.push(*unit),
                }
            }
            for unit in target_units.difference(&current_units) {
                let information = state.unit(*unit);
                extra_mapping_inserted.insert(information.mapping_key.expect("用户看得见的单元都进映射"));
                let record = state.records.get(&information.slot).copied().expect("目标那一版引用的单元还有记录");
                if let Some(quarantined) = state.quarantined.get(unit) {
                    let mut revived_record = record;
                    revived_record.carries_the_quarantine_mark = false;
                    record_changes.insert(quarantined.slot, Some(revived_record));
                    if arm == Arm::QuarantineTree {
                        quarantine_tree_deleted.extend(quarantine_entries_of(quarantined.slot));
                    }
                    revived.push(*unit);
                } else {
                    assert!(record.is_released, "目标那一版引用、现行那一版不引用的单元在现行的账里是已释放的");
                    resurrected_deferred.push((record.generation, information.slot));
                    record_changes.insert(
                        information.slot,
                        Some(AllocationRecord { span_slots: record.span_slots, is_released: false, generation: information.birth_txg, carries_the_quarantine_mark: false }),
                    );
                }
            }
            target
        }
    };
    // 记账树根兼叶每次发布都重写。
    if let Some(old_accounting) = state.accounting {
        released_nodes.push(old_accounting);
        mapping_deleted.insert(state.unit(old_accounting).mapping_key.expect("记账树进映射"));
    }
    let accounting = planned(fresh.issue(), WrittenStructure::AccountingNode, Footprint::OneSlot, Some((UNIT_CLASS_INDEX_NODE, ACCOUNTING_TREE, txg, WRITE_ORDER_INSTANCE, 0)));
    // 乙的隔离树：照码 2 规则长；节点进中央映射；删空之后零根、树表条目留着。
    let quarantine_tree_after = if quarantine_tree_inserted.is_empty() && quarantine_tree_deleted.is_empty() {
        state.quarantine_tree.clone()
    } else {
        plan_the_tree_after_this_publish(&state.quarantine_tree, &quarantine_tree_deleted, &quarantine_tree_inserted, quarantine_tree_capacity(), &mut fresh)
    };
    let quarantine_units_after: BTreeSet<UnitIdentifier> = units_of_the_tree_in_bump_order(&quarantine_tree_after).into_iter().collect();
    for old_node in units_of_the_tree_in_bump_order(&state.quarantine_tree) {
        if !quarantine_units_after.contains(&old_node) {
            released_nodes.push(old_node);
            mapping_deleted.insert(state.unit(old_node).mapping_key.expect("隔离树节点进映射"));
        }
    }
    let mut quarantine_tree_nodes = Vec::new();
    for unit in units_of_the_tree_in_bump_order(&quarantine_tree_after) {
        if fresh.is_fresh(unit) {
            let sequence = u64_of(quarantine_tree_nodes.len());
            let mut node = planned(unit, WrittenStructure::QuarantineTreeNode, Footprint::OneSlot, Some((UNIT_CLASS_INDEX_NODE, QUARANTINE_TREE, txg, WRITE_ORDER_INSTANCE, sequence)));
            if quarantine_tree_nodes.is_empty() {
                node.forced_slot = request.forced_quarantine_tree_node_slot();
            }
            quarantine_tree_nodes.push(node);
        }
    }
    let quarantine_tree_registered_by_this_publish = !state.quarantine_tree_registered && quarantine_tree_after.is_some();
    released_nodes.extend(state.tree_table);
    let tree_table = planned(fresh.issue(), WrittenStructure::TreeTableUnit, Footprint::OneSlot, None);
    let mut mapping_inserted: BTreeSet<MappingKey> = data_units
        .iter()
        .chain(nodes_before_the_allocation_record_tree.iter())
        .chain(std::iter::once(&accounting))
        .chain(quarantine_tree_nodes.iter())
        .filter_map(|planned_unit| planned_unit.mapping_key)
        .collect();
    mapping_inserted.extend(extra_mapping_inserted);
    UserIntent {
        txg,
        fresh,
        data_units,
        nodes_before_the_allocation_record_tree,
        accounting,
        quarantine_tree_after,
        quarantine_tree_nodes,
        tree_table,
        version_after,
        released_data_units,
        released_nodes,
        mapping_deleted,
        mapping_inserted,
        record_changes,
        resurrected_deferred,
        quarantine_releases,
        newly_quarantined,
        revived,
        released_quarantined,
        quarantine_tree_registered_by_this_publish,
    }
}

/// 走完固定点的一次发布：要改哪些记录、取了哪些槽、换下哪些落点、新写了哪些单元、之后的各棵树。
struct PublishPlan {
    txg: u64,
    fresh: FreshUnits,
    new_units: BTreeMap<UnitIdentifier, UnitInformation>,
    record_changes: BTreeMap<u64, Option<AllocationRecord>>,
    released_placements: Vec<(u64, u64)>,
    resurrected_deferred: Vec<(u64, u64)>,
    taken: BTreeSet<u64>,
    open_segment_start: Option<u64>,
    bump_cursor: u64,
    opened_segments: BTreeSet<usize>,
    placements_that_fell_back: u64,
    version_after: UserVisibleVersion,
    allocation_nodes_after: BTreeMap<AllocationNodePosition, UnitIdentifier>,
    rewritten_allocation_positions: BTreeSet<AllocationNodePosition>,
    accounting_after: UnitIdentifier,
    tree_table_after: UnitIdentifier,
    mapping_after: Option<CodeTwoNode<MappingKey>>,
    quarantine_tree_after: Option<CodeTwoNode<QuarantineEntryKey>>,
    quarantine_tree_registered_by_this_publish: bool,
    newly_quarantined: Option<UnitIdentifier>,
    revived: Vec<UnitIdentifier>,
    released_quarantined: Option<UnitIdentifier>,
}

impl PublishPlan {
    fn new_unit_list(&self) -> Vec<UnitInformation> {
        self.new_units.values().copied().collect()
    }

    fn writes(&self, padding_named_roles: u64) -> PublishWrites {
        tally_publish_writes(&self.new_unit_list(), padding_named_roles, self.quarantine_tree_registered_by_this_publish)
    }
}

/// 新落点：罩住的已回收记录随这次分配删掉，起点上那条（已回收的）改写成仍分配（`allocator.rs` 的 `AllocationRecord` 文档）。
fn record_a_new_placement(state: &PoolState, record_changes: &mut BTreeMap<u64, Option<AllocationRecord>>, slot: u64, span_slots: u64, txg: u64) {
    for start in slot.saturating_sub(1)..slot + span_slots {
        if start == slot {
            continue;
        }
        if let Some(record) = current_record(state, record_changes, start) {
            if start + u64::from(record.span_slots) > slot {
                assert!(record.is_released, "新落点罩住的记录是已回收的");
                record_changes.insert(start, None);
            }
        }
    }
    if let Some(existing) = current_record(state, record_changes, slot) {
        assert!(existing.is_released, "新落点起点上的记录是已回收的");
    }
    let span_slots = u16::try_from(span_slots).expect("一个单元两槽以内");
    record_changes.insert(slot, Some(AllocationRecord { span_slots, is_released: false, generation: txg, carries_the_quarantine_mark: false }));
}

fn release_a_unit(state: &PoolState, record_changes: &mut BTreeMap<u64, Option<AllocationRecord>>, released_placements: &mut Vec<(u64, u64)>, unit: UnitIdentifier, txg: u64) {
    let information = state.unit(unit);
    let mut record = current_record(state, record_changes, information.slot).expect("换下的单元有记录");
    assert!(!record.is_released, "同一个落点释放了两次");
    record.is_released = true;
    record.generation = txg;
    record_changes.insert(information.slot, Some(record));
    released_placements.push((information.slot, information.span_slots));
}

/// 按给定的分配记录树重写集走一遍：映射树怎么变、释放、取落点、改记录（`transaction.rs` 的 `settle_the_allocation_record_tree` 每一轮做的事）。
fn walk_the_publish(state: &PoolState, intent: &UserIntent, rewrite_set: &BTreeSet<AllocationNodePosition>) -> PublishPlan {
    let txg = intent.txg;
    let mut fresh = intent.fresh;
    let mut released_nodes = intent.released_nodes.clone();
    let mut mapping_deleted = intent.mapping_deleted.clone();
    let mut mapping_inserted = intent.mapping_inserted.clone();
    let mut allocation_nodes_after = state.allocation_nodes.clone();
    let mut allocation_nodes = Vec::new();
    for (sequence, position) in rewrite_set.iter().enumerate() {
        if let Some(old_node) = state.allocation_nodes.get(position) {
            released_nodes.push(*old_node);
            mapping_deleted.insert(state.unit(*old_node).mapping_key.expect("分配记录树节点进映射"));
        }
        let unit = fresh.issue();
        let mapping_key = (UNIT_CLASS_INDEX_NODE, ALLOCATION_RECORD_TREE, txg, WRITE_ORDER_INSTANCE, u64_of(sequence));
        mapping_inserted.insert(mapping_key);
        allocation_nodes_after.insert(*position, unit);
        allocation_nodes.push(planned(unit, WrittenStructure::AllocationRecordTreeNode, Footprint::OneSlot, Some(mapping_key)));
    }
    let mapping_after = plan_the_tree_after_this_publish(&state.mapping, &mapping_deleted, &mapping_inserted, mapping_capacity(), &mut fresh);
    let mapping_units_after = units_of_the_tree_in_bump_order(&mapping_after);
    let mapping_unit_set: BTreeSet<UnitIdentifier> = mapping_units_after.iter().copied().collect();
    released_nodes.extend(units_of_the_tree_in_bump_order(&state.mapping).into_iter().filter(|old_node| !mapping_unit_set.contains(old_node)));
    let mapping_nodes: Vec<PlannedUnit> = mapping_units_after
        .iter()
        .filter(|unit| fresh.is_fresh(**unit))
        .map(|unit| planned(*unit, WrittenStructure::CentralMappingNode, Footprint::OneSlot, None))
        .collect();

    // 释放先于分配：换下的落点进 defer、槽仍占着，这次的落点不会落到它们上面。
    let mut record_changes = intent.record_changes.clone();
    let mut released_placements = Vec::new();
    for unit in &intent.released_data_units {
        release_a_unit(state, &mut record_changes, &mut released_placements, *unit, txg);
    }
    for replaced_node in &released_nodes {
        release_a_unit(state, &mut record_changes, &mut released_placements, *replaced_node, txg);
    }
    released_placements.extend(intent.quarantine_releases.iter().copied());

    // 取落点：数据单元先取（最低的空槽对），提交内生块再按 bump 次序从开放段取（D3（空间分配） 已定项 10 ⑤）。
    let mut walk = PlacementWalk::over(state);
    let mut new_units = BTreeMap::new();
    let commit_generated = intent
        .nodes_before_the_allocation_record_tree
        .iter()
        .chain(allocation_nodes.iter())
        .chain(std::iter::once(&intent.accounting))
        .chain(intent.quarantine_tree_nodes.iter())
        .chain(mapping_nodes.iter())
        .chain(std::iter::once(&intent.tree_table));
    let in_placement_order: Vec<&PlannedUnit> = intent.data_units.iter().chain(commit_generated).collect();
    // 微历史构造里指定了槽的那几个先占上，分配器再从剩下的槽里取。
    let mut forced_slots: BTreeMap<UnitIdentifier, u64> = BTreeMap::new();
    for planned_unit in &in_placement_order {
        if let Some(forced_slot) = planned_unit.forced_slot {
            forced_slots.insert(planned_unit.unit, walk.force(forced_slot, planned_unit.footprint));
        }
    }
    for planned_unit in in_placement_order {
        let slot = match (forced_slots.get(&planned_unit.unit), planned_unit.structure) {
            (Some(forced_slot), _any_structure) => *forced_slot,
            (None, WrittenStructure::DataUnit) => walk.allocate_data_unit(),
            (
                None,
                WrittenStructure::ExtentNode
                | WrittenStructure::InodeNode
                | WrittenStructure::AllocationRecordTreeNode
                | WrittenStructure::AccountingNode
                | WrittenStructure::QuarantineTreeNode
                | WrittenStructure::CentralMappingNode
                | WrittenStructure::TreeTableUnit,
            ) => walk.allocate_commit_generated(planned_unit.footprint),
        };
        record_a_new_placement(state, &mut record_changes, slot, planned_unit.footprint.slots(), txg);
        new_units.insert(
            planned_unit.unit,
            UnitInformation { slot, span_slots: planned_unit.footprint.slots(), mapping_key: planned_unit.mapping_key, structure: planned_unit.structure, birth_txg: txg },
        );
    }
    PublishPlan {
        txg,
        fresh,
        new_units,
        record_changes,
        released_placements,
        resurrected_deferred: intent.resurrected_deferred.clone(),
        taken: walk.taken,
        open_segment_start: walk.open_segment_start,
        bump_cursor: walk.bump_cursor,
        opened_segments: walk.opened_segments,
        placements_that_fell_back: walk.placements_that_fell_back,
        version_after: intent.version_after.clone(),
        allocation_nodes_after,
        rewritten_allocation_positions: rewrite_set.clone(),
        accounting_after: intent.accounting.unit,
        tree_table_after: intent.tree_table.unit,
        mapping_after,
        quarantine_tree_after: intent.quarantine_tree_after.clone(),
        quarantine_tree_registered_by_this_publish: intent.quarantine_tree_registered_by_this_publish,
        newly_quarantined: intent.newly_quarantined,
        revived: intent.revived.clone(),
        released_quarantined: intent.released_quarantined,
    }
}

/// 照走出来的记录，内容变了的节点（记录清单不同的叶连全部祖先与根）与这一版有记录却还没有节点的叶。
fn allocation_nodes_to_rewrite(state: &PoolState, record_changes: &BTreeMap<u64, Option<AllocationRecord>>) -> BTreeSet<AllocationNodePosition> {
    let mut positions = BTreeSet::new();
    for (slot, record_after) in record_changes {
        if state.records.get(slot) != record_after.as_ref() {
            leaf_and_its_ancestors(state.root_level, leaf_of_slot(*slot), &mut positions);
        }
    }
    for leaf in &state.leaves_with_records {
        if !state.allocation_nodes.contains_key(&AllocationNodePosition { level: 0, device: 0, index_in_device: *leaf }) {
            leaf_and_its_ancestors(state.root_level, *leaf, &mut positions);
        }
    }
    positions
}

/// 固定点：从「重写集空」起，按它走一遍、算出要重写的节点，都在重写集里就定下，否则并进去再走（多罩的节点照样重写，合法的 COW）。
fn plan_publish(state: &PoolState, request: &PublishRequest, arm: Arm) -> PublishPlan {
    let intent = user_intent(state, request, arm);
    let mut rewrite_set = BTreeSet::new();
    for _round in 0..SETTLING_ROUNDS_LIMIT {
        let walked = walk_the_publish(state, &intent, &rewrite_set);
        let needed = allocation_nodes_to_rewrite(state, &walked.record_changes);
        if needed.is_subset(&rewrite_set) {
            return walked;
        }
        rewrite_set.extend(needed);
    }
    panic!("分配记录树重写集 {SETTLING_ROUNDS_LIMIT} 轮没定下：模型里重写集只增不减，以池里的节点数为上界");
}

/// 把走完固定点的那一次发布落进池状态，再按新的最新 txg 回收（释放代 ≤ 最新 txg − K 的落点回到空闲）。
fn apply_publish(state: &mut PoolState, plan: PublishPlan) {
    for slot in &plan.taken {
        state.set_occupied(*slot, true);
    }
    for segment in &plan.opened_segments {
        state.is_cluster_segment[*segment] = true;
    }
    state.open_segment_start = plan.open_segment_start;
    state.bump_cursor = plan.bump_cursor;
    for (slot, record_after) in &plan.record_changes {
        match record_after {
            Some(record) => {
                state.records.insert(*slot, *record);
                state.leaves_with_records.insert(leaf_of_slot(*slot));
            }
            None => {
                state.records.remove(slot);
                let leaf = leaf_of_slot(*slot);
                let leaf_start = leaf * ALLOCATION_RECORD_TREE_LEAF_SLOTS;
                assert!(
                    state.records.range(leaf_start..leaf_start + ALLOCATION_RECORD_TREE_LEAF_SLOTS).next().is_some(),
                    "一片叶的记录不会删到空：被删的只有新落点罩住的已回收记录，新落点那条就在旁边"
                );
            }
        }
    }
    for (slot, span_slots) in &plan.released_placements {
        state.deferred.insert((plan.txg, *slot), *span_slots);
    }
    for key in &plan.resurrected_deferred {
        state.deferred.remove(key).expect("复活的落点还在 defer 里：目标那一版在候选窗口里");
    }
    state.units.resize(usize_of(plan.fresh.next_unit), None);
    for (unit, information) in &plan.new_units {
        state.units[usize_of(unit.0)] = Some(*information);
    }
    state.version = Some(plan.version_after);
    state.allocation_nodes = plan.allocation_nodes_after;
    for leaf in &state.leaves_with_records {
        assert!(state.allocation_nodes.contains_key(&AllocationNodePosition { level: 0, device: 0, index_in_device: *leaf }), "有记录的叶都有节点");
    }
    state.accounting = Some(plan.accounting_after);
    state.tree_table = Some(plan.tree_table_after);
    state.mapping = plan.mapping_after;
    state.quarantine_tree = plan.quarantine_tree_after;
    state.quarantine_tree_registered |= plan.quarantine_tree_registered_by_this_publish;
    if let Some(unit) = plan.newly_quarantined {
        let information = state.unit(unit);
        state.quarantined.insert(unit, QuarantinedUnit { slot: information.slot, span_slots: information.span_slots });
    }
    for unit in plan.revived.iter().chain(plan.released_quarantined.iter()) {
        state.quarantined.remove(unit).expect("复活或释放的是一个隔离着的单元");
    }
    state.txg = plan.txg;
    let floor = oldest_candidate_txg(state.txg, state.candidate_window);
    while let Some((&(release_generation, slot), &span_slots)) = state.deferred.first_key_value() {
        if release_generation > floor {
            break;
        }
        state.deferred.remove(&(release_generation, slot));
        for covered in slot..slot + span_slots {
            state.set_occupied(covered, false);
        }
    }
    // 两个下界提示只往上收紧：此刻往下都是占着的槽、都不是全空段。
    state.lowest_possibly_free_slot = state.first_unoccupied_slot_from(state.lowest_possibly_free_slot);
    while state.lowest_possibly_empty_segment < state.used_slots_per_segment.len() && state.used_slots_per_segment[state.lowest_possibly_empty_segment] > 0 {
        state.lowest_possibly_empty_segment += 1;
    }
    if state.keeps_versions_for_rollback {
        let kept = state.version.clone().expect("刚落了这一版");
        state.kept_versions.insert(state.txg, kept);
        state.kept_versions.retain(|txg, _| *txg >= floor);
    }
}

/// 走一次发布并落进池状态；交回这次的写与取落点回落过几次。
fn publish(state: &mut PoolState, request: &PublishRequest, arm: Arm) -> (PublishWrites, u64) {
    let plan = plan_publish(state, request, arm);
    let writes = plan.writes(0);
    let placements_that_fell_back = plan.placements_that_fell_back;
    apply_publish(state, plan);
    (writes, placements_that_fell_back)
}

/// 位翻转分类表的记录类（登记第六节 `flip_classification_table`）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum RecordClass {
    QuarantineIntact,
    QuarantineGarbled,
    Leak,
    Normal,
}
const RECORD_CLASSES: [RecordClass; 4] = [RecordClass::QuarantineIntact, RecordClass::QuarantineGarbled, RecordClass::Leak, RecordClass::Normal];

impl RecordClass {
    fn label(self) -> &'static str {
        match self {
            RecordClass::QuarantineIntact => "quarantine_intact",
            RecordClass::QuarantineGarbled => "quarantine_garbled",
            RecordClass::Leak => "leak",
            RecordClass::Normal => "normal",
        }
    }

    fn copies(self) -> CopyCondition {
        match self {
            RecordClass::QuarantineGarbled => CopyCondition::Garbled,
            RecordClass::QuarantineIntact | RecordClass::Leak | RecordClass::Normal => CopyCondition::Intact,
        }
    }

    fn is_quarantine(self) -> bool {
        match self {
            RecordClass::QuarantineIntact | RecordClass::QuarantineGarbled => true,
            RecordClass::Leak | RecordClass::Normal => false,
        }
    }
}

/// 记号写对、两盘一起写错、只一盘写错（登记第六节「另报只翻一盘」）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum MarkState {
    WrittenCorrectly,
    WrongOnBothDevices,
    WrongOnOneDevice,
}
const MARK_STATES: [MarkState; 3] = [MarkState::WrittenCorrectly, MarkState::WrongOnBothDevices, MarkState::WrongOnOneDevice];

impl MarkState {
    fn label(self) -> &'static str {
        match self {
            MarkState::WrittenCorrectly => "written_correctly",
            MarkState::WrongOnBothDevices => "wrong_on_both_devices",
            MarkState::WrongOnOneDevice => "wrong_on_one_device",
        }
    }
}

/// 判哪一版：隔离那一版；I-3.1 候选窗口里最后一个引用它的根离开窗口之后那一版。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum JudgedVersion {
    TheQuarantineVersion,
    AfterTheLastReferencingRootLeavesTheWindow,
}
const JUDGED_VERSIONS: [JudgedVersion; 2] = [JudgedVersion::TheQuarantineVersion, JudgedVersion::AfterTheLastReferencingRootLeavesTheWindow];

impl JudgedVersion {
    fn label(self) -> &'static str {
        match self {
            JudgedVersion::TheQuarantineVersion => "quarantine_version",
            JudgedVersion::AfterTheLastReferencingRootLeavesTheWindow => "after_window",
        }
    }
}

/// checker 判一条（两块盘上的那一对）记录时看得到的东西。`marked_on_device` 在甲是跨度段 0x4000 位，在乙是隔离树里那一条条目，
/// 在 `no_mark` 是同一位 0x4000（今天的格式里就是一个越界的跨度）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct RecordView {
    is_released: bool,
    marked_on_device: [bool; 2],
    referenced_by_the_newest_root: bool,
    referenced_by_some_candidate_root: bool,
    unmapped_while_unreleased_in_some_candidate_version: bool,
    copies: CopyCondition,
}

/// 一张盘面上哪几条判据判红。
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
struct CheckerVerdict {
    allocated_statistic_red: bool,
    allocated_minus_deferred_red: bool,
    unreleased_record_off_the_mapping_red: bool,
    mark_agreement_red: bool,
    record_outside_its_leaf_red: bool,
}

impl CheckerVerdict {
    fn is_red(&self) -> bool {
        self.allocated_statistic_red
            || self.allocated_minus_deferred_red
            || self.unreleased_record_off_the_mapping_red
            || self.mark_agreement_red
            || self.record_outside_its_leaf_red
    }

    fn isolation_criteria_red_count(&self) -> u64 {
        u64::from(self.allocated_statistic_red) + u64::from(self.allocated_minus_deferred_red) + u64::from(self.unreleased_record_off_the_mapping_red)
    }

    /// 产物里的那一格：判绿，或判红的判据名单。
    fn label(&self) -> String {
        let mut names = Vec::new();
        if self.allocated_statistic_red {
            names.push("I-3.1");
        }
        if self.allocated_minus_deferred_red {
            names.push("I-3.11");
        }
        if self.unreleased_record_off_the_mapping_red {
            names.push("I-3.15");
        }
        if self.mark_agreement_red {
            names.push("mark_agrees_with_references");
        }
        if self.record_outside_its_leaf_red {
            names.push("I-1.1");
        }
        if names.is_empty() {
            "green".to_string()
        } else {
            names.join("+")
        }
    }
}

/// 今天的副本判据：有一份读盘核出对不上（读不出也算）才豁免（`walk.rs` 的 `some_copy_of_the_unit_fails_its_location_entries`）。
fn some_copy_fails_its_location_entries(copies: CopyCondition) -> bool {
    match copies {
        CopyCondition::Intact => false,
        CopyCondition::Garbled => true,
    }
}

/// 记号与引用相符（登记第五节「丙的 checker」三句）。
fn mark_agrees_with_references(view: &RecordView) -> bool {
    let marked_anywhere = view.marked_on_device.iter().any(|marked| *marked);
    let marked_everywhere = view.marked_on_device.iter().all(|marked| *marked);
    !(marked_anywhere && view.is_released) && !(marked_anywhere && view.referenced_by_the_newest_root) && (marked_everywhere || !marked_anywhere)
}

/// 一条臂的 checker 判一对记录（登记第五节「臂」最后一栏与「丙的 checker」）。
fn judge_record(checker: CheckerUnderTest, view: &RecordView) -> CheckerVerdict {
    let marked_anywhere = view.marked_on_device.iter().any(|marked| *marked);
    let marked_everywhere = view.marked_on_device.iter().all(|marked| *marked);
    // 今天的读者不认 0x4000：跨度读成 16385 / 16386，落不进任何一片叶（`walk.rs` 的 I-1.1 落叶判），隔离判据那一遍不再看它。
    let today_reader_rejects_the_span = checker == CheckerUnderTest::Today && marked_anywhere;
    // 没有记号时怎么办：今天的副本判据；丙的两个变体一个只认记号、一个退回今天的副本判据。
    let exempt_without_a_mark = match checker {
        CheckerUnderTest::Today => some_copy_fails_its_location_entries(view.copies),
        CheckerUnderTest::MarkOnly => false,
        CheckerUnderTest::MarkOrToday => some_copy_fails_its_location_entries(view.copies),
    };
    // 记号齐（每块盘上那一份都带）的豁免、不看副本；只一盘带的不算齐、也不退回副本判据。
    let exempt = match checker {
        CheckerUnderTest::Today => exempt_without_a_mark,
        CheckerUnderTest::MarkOnly | CheckerUnderTest::MarkOrToday => marked_everywhere || (!marked_anywhere && exempt_without_a_mark),
    };
    let violates_mark_agreement = !mark_agrees_with_references(view);
    let mark_agreement_red = match checker {
        CheckerUnderTest::Today => false,
        CheckerUnderTest::MarkOnly | CheckerUnderTest::MarkOrToday => violates_mark_agreement,
    };
    let isolation_criteria_judged = !view.is_released && !today_reader_rejects_the_span;
    CheckerVerdict {
        allocated_statistic_red: isolation_criteria_judged && !view.referenced_by_some_candidate_root && !exempt,
        allocated_minus_deferred_red: isolation_criteria_judged && !view.referenced_by_the_newest_root && !exempt,
        unreleased_record_off_the_mapping_red: isolation_criteria_judged && view.unmapped_while_unreleased_in_some_candidate_version && !exempt,
        mark_agreement_red,
        record_outside_its_leaf_red: today_reader_rejects_the_span,
    }
}

/// 记号这一格在这条臂上落在哪两块盘上（写对的样子按类定：隔离的带记号、泄漏与正常的不带；`no_mark` 写对就是都不带）。
fn marks_of(arm: Arm, class: RecordClass, mark_state: MarkState) -> [bool; 2] {
    let correct = match arm {
        Arm::SpanBit | Arm::QuarantineTree => class.is_quarantine(),
        Arm::NoMark | Arm::PlainRelease => false,
    };
    match mark_state {
        MarkState::WrittenCorrectly => [correct, correct],
        MarkState::WrongOnBothDevices => match arm {
            Arm::SpanBit | Arm::QuarantineTree => [!correct, !correct],
            Arm::NoMark | Arm::PlainRelease => [true, true],
        },
        MarkState::WrongOnOneDevice => match arm {
            Arm::SpanBit | Arm::QuarantineTree => [!correct, correct],
            Arm::NoMark | Arm::PlainRelease => [true, false],
        },
    }
}

/// 造出来的一张盘面（登记第六节「造出来的盘面」）：这一类记录在这一版上 checker 看得到的样子。
fn constructed_view(arm: Arm, class: RecordClass, mark_state: MarkState, version: JudgedVersion) -> RecordView {
    let is_normal = class == RecordClass::Normal;
    RecordView {
        is_released: false,
        marked_on_device: marks_of(arm, class, mark_state),
        referenced_by_the_newest_root: is_normal,
        referenced_by_some_candidate_root: is_normal || version == JudgedVersion::TheQuarantineVersion,
        unmapped_while_unreleased_in_some_candidate_version: !is_normal,
        copies: class.copies(),
    }
}

/// 地真：隔离的两类与正常记录在记号写对时是合法盘；泄漏与任何写错的记号是缺陷（登记第一节「地真」）。
fn is_a_legal_disk(class: RecordClass, mark_state: MarkState) -> bool {
    class != RecordClass::Leak && mark_state == MarkState::WrittenCorrectly
}

/// 放过（缺陷判绿）与误红（合法判红）的条数，按两个宇宙分开：不翻记号（写对）与翻记号（写错，两盘与一盘）。
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct PassAndFalseRedCounts {
    passed_defects: u64,
    false_reds: u64,
    judged_disks: u64,
}

impl PassAndFalseRedCounts {
    fn count(&mut self, legal: bool, verdict: &CheckerVerdict, disks: u64) {
        self.judged_disks += disks;
        if legal && verdict.is_red() {
            self.false_reds += disks;
        }
        if !legal && !verdict.is_red() {
            self.passed_defects += disks;
        }
    }
}

/// 两个数怎么记（登记第六节 `pass_and_false_red_counts` 的判定）。
fn pass_and_false_red_reading(marking: PassAndFalseRedCounts, no_mark: PassAndFalseRedCounts) -> &'static str {
    if marking.false_reds < no_mark.false_reds && marking.passed_defects <= no_mark.passed_defects {
        "mark_dominates"
    } else if marking.false_reds < no_mark.false_reds {
        "false_reds_traded_for_passes"
    } else {
        "mark_does_not_solve_this_shape"
    }
}

fn universe_of(mark_state: MarkState) -> usize {
    match mark_state {
        MarkState::WrittenCorrectly => 0,
        MarkState::WrongOnBothDevices | MarkState::WrongOnOneDevice => 1,
    }
}
const UNIVERSE_LABELS: [&str; 2] = ["marks_not_flipped", "marks_flipped"];

/// 旧读者的跨度读法：`span_field & !0x8000`（`allocator.rs` 的 `AllocationRecord::parse`、`walk.rs` 的 `parse_allocation_record`；
/// GPU 判器 `span_field & 0x7FFF` 同值）。
fn old_reader_span(span_field: u16) -> u64 {
    u64::from(span_field & !ALLOCATION_RECORD_RELEASED_FLAG)
}

/// 旧读者的落叶判：跨度不为 0、末槽不越过叶末槽（`allocation_record_tree.rs` 的 `record_fits_in_its_leaf`、
/// `position_addressed.rs` 的 `allocation_record_fits_in_the_leaf`、GPU 判器同一句）。`start_in_leaf` 从叶起点数。
fn old_reader_accepts(start_in_leaf: u64, span: u64) -> bool {
    span > 0 && start_in_leaf + span - 1 <= ALLOCATION_RECORD_TREE_LEAF_SLOTS - 1
}

/// 借一位之后，一片叶里每个合法起点上的记录被旧读者收下几条（登记第六节 `old_reader_field_table` 的装置算术）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct OldReaderArithmetic {
    borrowed_bit: u16,
    record_span: u16,
    legal_starts: u64,
    span_read: u64,
    accepted: u64,
}

fn old_reader_arithmetic(borrowed_bit: u16, record_span: u16) -> OldReaderArithmetic {
    let legal_starts: Vec<u64> =
        (0..ALLOCATION_RECORD_TREE_LEAF_SLOTS).filter(|start| start % u64::from(record_span) == 0 && start + u64::from(record_span) <= ALLOCATION_RECORD_TREE_LEAF_SLOTS).collect();
    let span_read = old_reader_span(record_span | borrowed_bit);
    let accepted = u64_of(legal_starts.iter().filter(|start| old_reader_accepts(**start, span_read)).count());
    OldReaderArithmetic { borrowed_bit, record_span, legal_starts: u64_of(legal_starts.len()), span_read, accepted }
}

/// 一次隔离事件在一段历史里的样子（轨迹与终态分类读它）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct QuarantineEventRecord {
    txg: u64,
    copies: CopyCondition,
    marks_during_quarantine: [bool; 2],
    revival: Option<RevivalRecord>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct RevivalRecord {
    txg: u64,
    marks_after_revival: [bool; 2],
}

/// 一个事件从隔离那一版起到历史末尾，I-3.1、I-3.11、I-3.15 各红不红之和（0 到 3）的轨迹（登记第八节「轨迹」）。
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
struct EventTrajectory {
    peak: u64,
    positive_versions: u64,
    final_value: u64,
    first_red_offset_of_the_allocated_statistic: Option<u64>,
    mark_agreement_red_versions: u64,
}

fn trajectory_of_an_event(checker: CheckerUnderTest, event: &QuarantineEventRecord, final_txg: u64, candidate_window: u64) -> EventTrajectory {
    let mut trajectory = EventTrajectory::default();
    for version in event.txg..=final_txg {
        let revived = event.revival.is_some_and(|revival| version >= revival.txg);
        let marks_now = match event.revival {
            Some(revival) if revived => revival.marks_after_revival,
            Some(_) | None => event.marks_during_quarantine,
        };
        let referenced_by_some_candidate_root = event.txg - 1 >= oldest_candidate_txg(version, candidate_window) || revived;
        let newest_view = RecordView {
            is_released: false,
            marked_on_device: marks_now,
            referenced_by_the_newest_root: revived,
            referenced_by_some_candidate_root,
            unmapped_while_unreleased_in_some_candidate_version: false,
            copies: event.copies,
        };
        let newest = judge_record(checker, &newest_view);
        // I-3.15 每一版各判：窗口里还有隔离那几版（隔离到复活之前）时，那几版里这条记录未释放而映射里没有它。
        let window_start = oldest_candidate_txg(version, candidate_window).max(event.txg);
        let quarantine_end = event.revival.map_or(version, |revival| (revival.txg - 1).min(version));
        let quarantine_view = RecordView {
            is_released: false,
            marked_on_device: event.marks_during_quarantine,
            referenced_by_the_newest_root: false,
            referenced_by_some_candidate_root: true,
            unmapped_while_unreleased_in_some_candidate_version: window_start <= quarantine_end,
            copies: event.copies,
        };
        let off_the_mapping = judge_record(checker, &quarantine_view).unreleased_record_off_the_mapping_red;
        let value = u64::from(newest.allocated_statistic_red) + u64::from(newest.allocated_minus_deferred_red) + u64::from(off_the_mapping);
        if newest.allocated_statistic_red && trajectory.first_red_offset_of_the_allocated_statistic.is_none() {
            trajectory.first_red_offset_of_the_allocated_statistic = Some(version - event.txg);
        }
        trajectory.peak = trajectory.peak.max(value);
        trajectory.positive_versions += u64::from(value > 0);
        trajectory.final_value = value;
        trajectory.mark_agreement_red_versions += u64::from(newest.mark_agreement_red);
    }
    trajectory
}

/// 八段微历史（登记第五节「微历史」，手推钉值在第七节）。都在 `geometry_default` 上、映射树只有根兼叶。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum MicroHistory {
    CleanLeaf,
    CleanAncestor,
    DirtyLeaf,
    JournalCrossing,
    SecondQuarantineTreeCold,
    RevivalClean,
    RevivalTreeCold,
    ReleaseQuarantined,
}
const MICRO_HISTORIES: [MicroHistory; 8] = [
    MicroHistory::CleanLeaf,
    MicroHistory::CleanAncestor,
    MicroHistory::DirtyLeaf,
    MicroHistory::JournalCrossing,
    MicroHistory::SecondQuarantineTreeCold,
    MicroHistory::RevivalClean,
    MicroHistory::RevivalTreeCold,
    MicroHistory::ReleaseQuarantined,
];

impl MicroHistory {
    fn label(self) -> &'static str {
        match self {
            MicroHistory::CleanLeaf => "micro_clean_leaf",
            MicroHistory::CleanAncestor => "micro_clean_ancestor",
            MicroHistory::DirtyLeaf => "micro_dirty_leaf",
            MicroHistory::JournalCrossing => "micro_journal_crossing",
            MicroHistory::SecondQuarantineTreeCold => "micro_second_quarantine_tree_cold",
            MicroHistory::RevivalClean => "micro_revival_clean",
            MicroHistory::RevivalTreeCold => "micro_revival_tree_cold",
            MicroHistory::ReleaseQuarantined => "micro_release_quarantined",
        }
    }

    fn victim_slot(self) -> u64 {
        match self {
            MicroHistory::CleanAncestor => MICRO_CLEAN_ANCESTOR_VICTIM_SLOT,
            MicroHistory::DirtyLeaf => MICRO_DIRTY_LEAF_VICTIM_SLOT,
            MicroHistory::CleanLeaf
            | MicroHistory::JournalCrossing
            | MicroHistory::SecondQuarantineTreeCold
            | MicroHistory::RevivalClean
            | MicroHistory::RevivalTreeCold
            | MicroHistory::ReleaseQuarantined => MICRO_CLEAN_LEAF_VICTIM_SLOT,
        }
    }

    /// 这次发布（不含臂差）最后一个事务的点名数。
    fn named_roles_of_the_last_transaction(self) -> u64 {
        match self {
            MicroHistory::JournalCrossing => MICRO_CROSSING_NAMED_ROLES,
            MicroHistory::CleanLeaf
            | MicroHistory::CleanAncestor
            | MicroHistory::DirtyLeaf
            | MicroHistory::SecondQuarantineTreeCold
            | MicroHistory::RevivalClean
            | MicroHistory::RevivalTreeCold
            | MicroHistory::ReleaseQuarantined => MICRO_BASE_NAMED_ROLES,
        }
    }

    /// 第七节钉的「臂减 `no_mark`」：(甲的设备写, 甲的字节, 乙的设备写, 乙的字节)。
    fn pinned_extra_writes(self) -> (u64, u64, u64, u64) {
        match self {
            MicroHistory::CleanLeaf => (4, 65536, 2, 32768),
            MicroHistory::CleanAncestor => (8, 131072, 2, 32768),
            MicroHistory::DirtyLeaf => (0, 0, 2, 32768),
            MicroHistory::JournalCrossing => (6, 73728, 2, 32768),
            MicroHistory::SecondQuarantineTreeCold => (4, 65536, 6, 98304),
            MicroHistory::RevivalClean => (4, 65536, 0, 0),
            MicroHistory::RevivalTreeCold => (4, 65536, 4, 65536),
            MicroHistory::ReleaseQuarantined => (0, 0, 0, 0),
        }
    }
}

/// 微历史的文件：8 个数据单元；被隔离的是第 7 个（落点由构造给定），同一次覆盖写的另三个是第 0–2 个；
/// 第二次隔离那一段先在第 3–6 个那一次隔离第 6 个；释放隔离记录那一次覆盖写第 3–6 个。
const MICRO_DATA_UNITS: u32 = 8;
const MICRO_VICTIM: u32 = 7;
const MICRO_EARLIER_VICTIM: u32 = 6;
const MICRO_OVERWRITES_WITH_THE_VICTIM: [u32; 4] = [0, 1, 2, MICRO_VICTIM];
const MICRO_OTHER_OVERWRITES: [u32; 4] = [3, 4, 5, MICRO_EARLIER_VICTIM];
const MICRO_BASE_NAMED_ROLES: u64 = 20;
const MICRO_CROSSING_NAMED_ROLES: u64 = 63;
/// 叶 75（罩 [60900, 61712)）；槽 140000 在叶 172、层级 1 第二个节点罩 [137228, 274456)；叶 62 里的 50500；叶 70 里的 57000。
const MICRO_CLEAN_LEAF_VICTIM_SLOT: u64 = 60900;
const MICRO_CLEAN_ANCESTOR_VICTIM_SLOT: u64 = 140000;
const MICRO_DIRTY_LEAF_VICTIM_SLOT: u64 = 50500;
const MICRO_COLD_QUARANTINE_TREE_NODE_SLOT: u64 = 57000;
/// 垫在 mkfs 那两条之后的填充记录（不动的两槽记录）：别的微历史垫到 50400，这次发布的落点都在叶 62 里头；
/// `micro_dirty_leaf` 垫到 50750（只空出被隔离单元那一对），这次发布的落点都在 [50750, 51156)：W 减半成 406 时那是另一片叶。
const MICRO_FILLER_START: u64 = UNIT_AREA_START_SLOT + 2;
const MICRO_FILLER_END: u64 = 50400;
const MICRO_DIRTY_LEAF_FILLER_END: u64 = 50750;
const MICRO_CANDIDATE_WINDOW: u64 = 8;

/// 搭好一段微历史，交回被量的那一次发布之前的池与那一次发布的请求。
fn prepare_micro_history(micro: MicroHistory, arm: Arm) -> (PoolState, PublishRequest) {
    let mut state = PoolState::after_make_filesystem(PoolGeometry::DefaultFourGibibyteDevices, MICRO_CANDIDATE_WINDOW, true);
    let filler_end = if micro == MicroHistory::DirtyLeaf { MICRO_DIRTY_LEAF_FILLER_END } else { MICRO_FILLER_END };
    for slot in (MICRO_FILLER_START..filler_end).step_by(2) {
        if slot != micro.victim_slot() {
            state.occupy_with_a_record_that_never_changes(slot, 2);
        }
    }
    let create = PublishRequest {
        kind: PublishKind::CreateFile { data_unit_count: MICRO_DATA_UNITS },
        forced: vec![ForcedPlacement::DataUnit { index: MICRO_VICTIM, slot: micro.victim_slot() }],
    };
    let created_at = state.txg + 1;
    publish(&mut state, &create, arm);
    if micro == MicroHistory::SecondQuarantineTreeCold {
        let earlier = PublishRequest {
            kind: PublishKind::Overwrite { data_indices: MICRO_OTHER_OVERWRITES.to_vec(), quarantine_victim: Some(MICRO_EARLIER_VICTIM), release_quarantined: None },
            forced: vec![ForcedPlacement::NewQuarantineTreeNode { slot: MICRO_COLD_QUARANTINE_TREE_NODE_SLOT }],
        };
        publish(&mut state, &earlier, arm);
    }
    let quarantine_the_victim = PublishKind::Overwrite { data_indices: MICRO_OVERWRITES_WITH_THE_VICTIM.to_vec(), quarantine_victim: Some(MICRO_VICTIM), release_quarantined: None };
    let measured = match micro {
        MicroHistory::CleanLeaf | MicroHistory::CleanAncestor | MicroHistory::DirtyLeaf | MicroHistory::JournalCrossing | MicroHistory::SecondQuarantineTreeCold => {
            PublishRequest::plain(quarantine_the_victim)
        }
        MicroHistory::RevivalClean | MicroHistory::RevivalTreeCold => {
            let forced = if micro == MicroHistory::RevivalTreeCold {
                vec![ForcedPlacement::NewQuarantineTreeNode { slot: MICRO_COLD_QUARANTINE_TREE_NODE_SLOT }]
            } else {
                Vec::new()
            };
            publish(&mut state, &PublishRequest { kind: quarantine_the_victim, forced }, arm);
            PublishRequest::plain(PublishKind::RollBack { target_txg: created_at })
        }
        MicroHistory::ReleaseQuarantined => {
            publish(&mut state, &PublishRequest::plain(quarantine_the_victim), arm);
            let victim_unit = *state.quarantined.keys().next().expect("上一次隔离了第 7 个数据单元");
            PublishRequest::plain(PublishKind::Overwrite { data_indices: MICRO_OTHER_OVERWRITES.to_vec(), quarantine_victim: None, release_quarantined: Some(victim_unit) })
        }
    };
    (state, measured)
}

/// 臂减 `no_mark` 的写（有正负：分叉之后的发布里两条臂的落点、节点年龄不同）。
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
struct WriteDifference {
    device_writes: i64,
    bytes_on_both_devices: i64,
}

fn signed(value: u64) -> i64 {
    i64::try_from(value).expect("写量装得进 i64")
}

fn difference(arm: WriteTally, baseline: WriteTally) -> WriteDifference {
    WriteDifference {
        device_writes: signed(arm.device_writes) - signed(baseline.device_writes),
        bytes_on_both_devices: signed(arm.bytes_on_both_devices) - signed(baseline.bytes_on_both_devices),
    }
}

/// 分结构的差（分配记录树节点、映射树节点、隔离树节点、journal 记录），产物里写成「次/字节」。
fn structure_differences(arm: &PublishWrites, baseline: &PublishWrites) -> String {
    let mut pairs: Vec<(&str, WriteDifference)> = [WrittenStructure::AllocationRecordTreeNode, WrittenStructure::CentralMappingNode, WrittenStructure::QuarantineTreeNode]
        .iter()
        .map(|structure| (structure.label(), difference(arm.of(*structure), baseline.of(*structure))))
        .collect();
    pairs.push(("journal", difference(arm.journal, baseline.journal)));
    pairs.iter().map(|(name, value)| format!("extra_{name}={}/{}", value.device_writes, value.bytes_on_both_devices)).collect::<Vec<String>>().join(" ")
}

/// 一段微历史在一条臂上被量的那一次发布。
struct MicroArmMeasurement {
    arm: Arm,
    writes: PublishWrites,
    padding_named_roles: u64,
    victim_leaf_rewritten: bool,
    /// 回退那两段：回退之后那一版上，复活的单元那一对记录被这条臂的 checker 判 `mark_agrees_with_references` 红不红（各变体取或）。
    mark_agreement_red_after_revival: Option<bool>,
}

fn marks_of_the_quarantined_record(state: &PoolState, arm: Arm, slot: u64) -> [bool; 2] {
    match arm {
        Arm::SpanBit => {
            let marked = state.records.get(&slot).is_some_and(|record| record.carries_the_quarantine_mark);
            [marked, marked]
        }
        Arm::QuarantineTree => quarantine_entries_of(slot).map(|entry| tree_contains(&state.quarantine_tree, &entry)),
        Arm::NoMark | Arm::PlainRelease => [false, false],
    }
}

fn arms_of_the_micro_history(micro: MicroHistory) -> Vec<Arm> {
    let mut arms = FORK_ARMS.to_vec();
    if micro == MicroHistory::CleanLeaf {
        arms.push(Arm::PlainRelease);
    }
    arms
}

/// 量一段微历史：每条臂各搭一遍、量那一次发布；点名数按 `no_mark` 那一遍垫到登记给的数，各臂垫同样多。
fn measure_micro_history(micro: MicroHistory) -> Vec<MicroArmMeasurement> {
    let mut prepared: Vec<(Arm, PoolState, PublishPlan)> = Vec::new();
    for arm in arms_of_the_micro_history(micro) {
        let (state, request) = prepare_micro_history(micro, arm);
        let plan = plan_publish(&state, &request, arm);
        prepared.push((arm, state, plan));
    }
    let natural_named_roles_of_no_mark = prepared
        .iter()
        .find(|(arm, _, _)| *arm == Arm::NoMark)
        .map(|(_, _, plan)| plan.writes(0).last_transaction_named_roles)
        .expect("每段微历史都跑 no_mark");
    let padding_named_roles = micro
        .named_roles_of_the_last_transaction()
        .checked_sub(natural_named_roles_of_no_mark)
        .expect("微历史本来的点名数不超过登记给的数");
    let mut measurements = Vec::new();
    for (arm, mut state, plan) in prepared {
        let writes = plan.writes(padding_named_roles);
        let victim_slot = micro.victim_slot();
        let victim_leaf_rewritten = plan.rewritten_allocation_positions.contains(&AllocationNodePosition { level: 0, device: 0, index_in_device: leaf_of_slot(victim_slot) });
        let revived_slots: Vec<u64> = plan.revived.iter().map(|unit| state.unit(*unit).slot).collect();
        apply_publish(&mut state, plan);
        let mark_agreement_red_after_revival = (!revived_slots.is_empty()).then(|| {
            revived_slots.iter().any(|slot| {
                let view = RecordView {
                    is_released: false,
                    marked_on_device: marks_of_the_quarantined_record(&state, arm, *slot),
                    referenced_by_the_newest_root: true,
                    referenced_by_some_candidate_root: true,
                    unmapped_while_unreleased_in_some_candidate_version: false,
                    copies: CopyCondition::Intact,
                };
                checkers_of(arm).iter().any(|checker| judge_record(*checker, &view).mark_agreement_red)
            })
        });
        measurements.push(MicroArmMeasurement { arm, writes, padding_named_roles, victim_leaf_rewritten, mark_agreement_red_after_revival });
    }
    measurements
}

/// 一段微历史在一条臂上的额外写（臂减 `no_mark`，不含根槽与系统配置槽）。
fn micro_extra(measurements: &[MicroArmMeasurement], arm: Arm) -> WriteDifference {
    let of = |wanted: Arm| measurements.iter().find(|measurement| measurement.arm == wanted).expect("这段微历史跑过这条臂").writes;
    difference(of(arm).without_root_and_system_configuration(), of(Arm::NoMark).without_root_and_system_configuration())
}

/// 第二段的一格（登记第五节「第二段」）：密度 × 回退率 × U × K × 几何 × 种子，三臂各跑一遍。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct HistoryCell {
    density: u64,
    rollback_rate: u64,
    file_size: u32,
    candidate_window: u64,
    geometry: PoolGeometry,
    seed: u64,
}

impl HistoryCell {
    fn label(&self) -> String {
        format!(
            "density={} rollback_rate={} file_size={} window={} geometry={} seed={}",
            self.density,
            self.rollback_rate,
            self.file_size,
            self.candidate_window,
            self.geometry.label(),
            self.seed
        )
    }
}

fn all_history_cells() -> Vec<HistoryCell> {
    let mut cells = Vec::new();
    for density in DENSITIES_PER_THOUSAND_PUBLISHES {
        for rollback_rate in ROLLBACK_RATES_PER_THOUSAND_PUBLISHES {
            for file_size in FILE_SIZES_IN_DATA_UNITS {
                for candidate_window in CANDIDATE_WINDOWS {
                    for geometry in GEOMETRIES {
                        for seed in SEEDS {
                            cells.push(HistoryCell { density, rollback_rate, file_size, candidate_window, geometry, seed });
                        }
                    }
                }
            }
        }
    }
    cells
}

/// 确定性的伪随机源（SplitMix64）：同一格同一种子在三条臂上给同一串选择。
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn seeded(cell: &HistoryCell, stream: u64) -> SplitMix64 {
        let geometry_code = match cell.geometry {
            PoolGeometry::DefaultFourGibibyteDevices => 1,
            PoolGeometry::WideOneTebibyteDevices => 2,
        };
        let mixed = [cell.density, cell.rollback_rate, u64::from(cell.file_size), cell.candidate_window, geometry_code, cell.seed, stream]
            .iter()
            .fold(0x9E37_79B9_7F4A_7C15u64, |accumulated, part| accumulated.rotate_left(17) ^ part.wrapping_mul(0xBF58_476D_1CE4_E5B9));
        SplitMix64 { state: mixed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }

    fn shuffle<Item>(&mut self, items: &mut [Item]) {
        for position in (1..items.len()).rev() {
            let other = usize_of(self.below(u64_of(position) + 1));
            items.swap(position, other);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ScheduledEvent {
    victim_position: usize,
    copies: CopyCondition,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ScheduledRollback {
    ToTheVersionBeforeTheEventAt(u64),
    ToTheOldestVersionInTheWindow,
}

/// 一段历史排的事件：哪几次普通发布隔离、隔离这次释放的第几个、副本状态；哪几次普通发布之后接一次回退、回退到哪。
struct HistorySchedule {
    events: BTreeMap<u64, ScheduledEvent>,
    rollbacks: BTreeMap<u64, ScheduledRollback>,
}

fn interval_is_free(accepted: &BTreeMap<u64, u64>, start: u64, end: u64) -> bool {
    accepted.iter().all(|(accepted_start, accepted_end)| end < *accepted_start || start > *accepted_end)
}

fn schedule_of(cell: &HistoryCell) -> HistorySchedule {
    let mut random = SplitMix64::seeded(cell, 1);
    let scheduled_quarantine_events = QUARANTINE_EVENTS_PER_DENSITY_STEP * cell.density;
    let mut event_publishes = BTreeSet::new();
    while u64_of(event_publishes.len()) < scheduled_quarantine_events {
        event_publishes.insert(1 + random.below(NORMAL_PUBLISHES_PER_HISTORY));
    }
    let ordered: Vec<u64> = event_publishes.iter().copied().collect();
    // 副本状态各半：按种子打乱之后前一半完好、后一半坏（不按一个拍出来的比例混）。
    let mut copy_order: Vec<usize> = (0..ordered.len()).collect();
    random.shuffle(&mut copy_order);
    let mut events = BTreeMap::new();
    for (rank, position) in copy_order.iter().enumerate() {
        let copies = if rank < ordered.len() / 2 { CopyCondition::Intact } else { CopyCondition::Garbled };
        events.insert(ordered[*position], ScheduledEvent { victim_position: 0, copies });
    }
    for event in events.values_mut() {
        event.victim_position = usize_of(random.below(u64_of(OVERWRITES_PER_PUBLISH)));
    }
    let mut rollbacks = BTreeMap::new();
    let scheduled_rollbacks = ROLLBACKS_AT_RATE_ONE * cell.rollback_rate;
    let mut accepted_intervals: BTreeMap<u64, u64> = BTreeMap::new();
    if cell.density > 0 {
        // 每次排在一次隔离事件之后 1 到 ⌊K ÷ 2⌋ 次发布里，目标取那次隔离之前的一版；几次回退的区间互不相交。
        let mut candidates = ordered.clone();
        random.shuffle(&mut candidates);
        let mut rejected_on_the_first_draw = Vec::new();
        for event_publish in candidates {
            if u64_of(rollbacks.len()) == scheduled_rollbacks {
                break;
            }
            let delay = 1 + random.below(cell.candidate_window / 2);
            let rollback_after = event_publish + delay;
            if rollback_after <= NORMAL_PUBLISHES_PER_HISTORY && interval_is_free(&accepted_intervals, event_publish, rollback_after) {
                accepted_intervals.insert(event_publish, rollback_after);
                rollbacks.insert(rollback_after, ScheduledRollback::ToTheVersionBeforeTheEventAt(event_publish));
            } else {
                rejected_on_the_first_draw.push(event_publish);
            }
        }
        // 第一遍每个事件只抽一次延迟，抽到的与已排的区间相交就跳过；不够登记的 10 次时，第二遍按同一个次序给跳过的事件
        // 在 1 到 ⌊K ÷ 2⌋ 里可行的延迟中按种子挑一个。第一遍就排满的格不进第二遍，排法与随机数的消耗都不变。
        for event_publish in rejected_on_the_first_draw {
            if u64_of(rollbacks.len()) == scheduled_rollbacks {
                break;
            }
            let feasible_delays: Vec<u64> = (1..=cell.candidate_window / 2)
                .filter(|delay| {
                    event_publish + delay <= NORMAL_PUBLISHES_PER_HISTORY && interval_is_free(&accepted_intervals, event_publish, event_publish + delay)
                })
                .collect();
            if feasible_delays.is_empty() {
                continue;
            }
            let rollback_after = event_publish + feasible_delays[usize_of(random.below(u64_of(feasible_delays.len())))];
            accepted_intervals.insert(event_publish, rollback_after);
            rollbacks.insert(rollback_after, ScheduledRollback::ToTheVersionBeforeTheEventAt(event_publish));
        }
    } else {
        // 密度 0：没有隔离可清，目标取窗口里最旧的一版，当对照。
        let mut attempts = 0u64;
        while u64_of(rollbacks.len()) < scheduled_rollbacks && attempts < NORMAL_PUBLISHES_PER_HISTORY {
            attempts += 1;
            let rollback_after = cell.candidate_window + 1 + random.below(NORMAL_PUBLISHES_PER_HISTORY - cell.candidate_window);
            if interval_is_free(&accepted_intervals, rollback_after - cell.candidate_window, rollback_after) {
                accepted_intervals.insert(rollback_after - cell.candidate_window, rollback_after);
                rollbacks.insert(rollback_after, ScheduledRollback::ToTheOldestVersionInTheWindow);
            }
        }
    }
    HistorySchedule { events, rollbacks }
}

/// 这一次覆盖写哪 4 个数据单元（按种子在 U 个里均匀挑、互不相同）。
fn choose_overwritten_data_units(random: &mut SplitMix64, file_size: u32) -> Vec<u32> {
    let mut chosen: Vec<u32> = Vec::with_capacity(OVERWRITES_PER_PUBLISH);
    while chosen.len() < OVERWRITES_PER_PUBLISH {
        let candidate = u32::try_from(random.below(u64::from(file_size))).expect("U 装得进 u32");
        if !chosen.contains(&candidate) {
            chosen.push(candidate);
        }
    }
    chosen
}

/// 一次隔离事件在一条臂上：那一次发布的写、`no_mark` 那一遍里被隔离单元所在叶这次本来重不重写、轨迹要的记录。
#[derive(Clone, Copy, Debug)]
struct EventOutcome {
    normal_publish: u64,
    writes: PublishWrites,
    victim_leaf_rewritten: bool,
    record: QuarantineEventRecord,
}

#[derive(Clone, Copy, Debug)]
struct RollbackOutcome {
    writes: PublishWrites,
    revived: u64,
}

/// 一条臂跑完一段历史（新建文件那一次不算进写量）。
struct HistoryOutcome {
    totals: PublishWrites,
    events: Vec<EventOutcome>,
    rollbacks: Vec<RollbackOutcome>,
    final_txg: u64,
    final_occupied_slots: u64,
    final_deferred_slots: u64,
    normal_units: u64,
    placements_that_fell_back: u64,
    mapping_tree_height: u8,
}

fn run_history(cell: &HistoryCell, schedule: &HistorySchedule, arm: Arm) -> HistoryOutcome {
    let mut state = PoolState::after_make_filesystem(cell.geometry, cell.candidate_window, cell.rollback_rate > 0);
    let (_, mut placements_that_fell_back) = publish(&mut state, &PublishRequest::plain(PublishKind::CreateFile { data_unit_count: cell.file_size }), arm);
    let mut data_random = SplitMix64::seeded(cell, 2);
    let mut totals = PublishWrites::default();
    let mut events: Vec<EventOutcome> = Vec::new();
    let mut event_of_the_quarantined_unit: BTreeMap<UnitIdentifier, usize> = BTreeMap::new();
    let mut rollbacks = Vec::new();
    for normal_publish in 1..=NORMAL_PUBLISHES_PER_HISTORY {
        let data_indices = choose_overwritten_data_units(&mut data_random, cell.file_size);
        let scheduled = schedule.events.get(&normal_publish).copied();
        let quarantine_victim = scheduled.map(|event| data_indices[event.victim_position]);
        let victim_unit = quarantine_victim.map(|index| state.version.as_ref().expect("文件建好了").data_units[usize_of(u64::from(index))]);
        let request = PublishRequest::plain(PublishKind::Overwrite { data_indices, quarantine_victim, release_quarantined: None });
        let plan = plan_publish(&state, &request, arm);
        let writes = plan.writes(0);
        placements_that_fell_back += plan.placements_that_fell_back;
        let victim_leaf_rewritten = victim_unit.is_some_and(|unit| {
            plan.rewritten_allocation_positions.contains(&AllocationNodePosition { level: 0, device: 0, index_in_device: leaf_of_slot(state.unit(unit).slot) })
        });
        apply_publish(&mut state, plan);
        totals.add(&writes);
        if let (Some(event), Some(unit)) = (scheduled, victim_unit) {
            let slot = state.unit(unit).slot;
            if arm != Arm::PlainRelease {
                event_of_the_quarantined_unit.insert(unit, events.len());
            }
            events.push(EventOutcome {
                normal_publish,
                writes,
                victim_leaf_rewritten,
                record: QuarantineEventRecord { txg: state.txg, copies: event.copies, marks_during_quarantine: marks_of_the_quarantined_record(&state, arm, slot), revival: None },
            });
        }
        if let Some(rollback) = schedule.rollbacks.get(&normal_publish) {
            let target_txg = match rollback {
                ScheduledRollback::ToTheVersionBeforeTheEventAt(event_publish) => {
                    events.iter().find(|event| event.normal_publish == *event_publish).expect("回退排在它那次隔离之后").record.txg - 1
                }
                ScheduledRollback::ToTheOldestVersionInTheWindow => oldest_candidate_txg(state.txg, cell.candidate_window),
            };
            let plan = plan_publish(&state, &PublishRequest::plain(PublishKind::RollBack { target_txg }), arm);
            let writes = plan.writes(0);
            placements_that_fell_back += plan.placements_that_fell_back;
            let revived = plan.revived.clone();
            apply_publish(&mut state, plan);
            totals.add(&writes);
            for unit in &revived {
                let event_index = *event_of_the_quarantined_unit.get(unit).expect("复活的单元是一次隔离事件隔离的");
                let slot = state.unit(*unit).slot;
                events[event_index].record.revival = Some(RevivalRecord { txg: state.txg, marks_after_revival: marks_of_the_quarantined_record(&state, arm, slot) });
            }
            rollbacks.push(RollbackOutcome { writes, revived: u64_of(revived.len()) });
        }
    }
    let unreleased_records = u64_of(state.records.values().filter(|record| !record.is_released).count());
    HistoryOutcome {
        totals,
        events,
        rollbacks,
        final_txg: state.txg,
        final_occupied_slots: state.occupied_slot_count(),
        final_deferred_slots: state.deferred_slot_count(),
        normal_units: unreleased_records - u64_of(state.quarantined.len()) - u64_of(MAKE_FILESYSTEM_RECORD_SLOTS.len()),
        placements_that_fell_back,
        mapping_tree_height: tree_height(&state.mapping),
    }
}

/// 一个事件在终态上属于哪一版：最后一个引用它的根已经离开候选窗口没有（登记第六节 `flip_classification_table` 的两版）。
fn judged_version_at_the_end(event: &QuarantineEventRecord, final_txg: u64, candidate_window: u64) -> JudgedVersion {
    if event.txg - 1 >= oldest_candidate_txg(final_txg, candidate_window) {
        JudgedVersion::TheQuarantineVersion
    } else {
        JudgedVersion::AfterTheLastReferencingRootLeavesTheWindow
    }
}

/// 有理数按四位小数四舍五入（远离 0）打出来；产物里不用浮点。
const FIXED_POINT_SCALE: i128 = 10_000;

fn scaled_ratio(numerator: i128, denominator: i128) -> i128 {
    assert!(denominator > 0, "分母是正数");
    let scaled = numerator * FIXED_POINT_SCALE;
    let half = denominator / 2;
    if scaled >= 0 {
        (scaled + half) / denominator
    } else {
        -((-scaled + half) / denominator)
    }
}

fn format_scaled(scaled: i128) -> String {
    let sign = if scaled < 0 { "-" } else { "" };
    let magnitude = scaled.unsigned_abs();
    let scale = FIXED_POINT_SCALE.unsigned_abs();
    format!("{sign}{}.{:04}", magnitude / scale, magnitude % scale)
}

fn format_ratio(numerator: i128, denominator: i128) -> String {
    if denominator == 0 {
        "none".to_string()
    } else {
        format_scaled(scaled_ratio(numerator, denominator))
    }
}

/// 份额落哪一档：档一 < 0.1%，档二 [0.1%, 1%)，档三 ≥ 1%；少写（负）的落档一。
fn share_tier(extra_bytes: i128, baseline_bytes: i128, bounds: ShareTierBounds) -> u8 {
    let parts_per_billion_times_baseline = extra_bytes * PARTS_PER_BILLION;
    if parts_per_billion_times_baseline < i128::from(bounds.tier_two_from) * baseline_bytes {
        1
    } else if parts_per_billion_times_baseline < i128::from(bounds.tier_three_from) * baseline_bytes {
        2
    } else {
        3
    }
}

/// 一组整数的均值、最小、最大（均值四位小数）；空组写 none。
fn mean_minimum_maximum(values: &[i64]) -> String {
    match (values.iter().min(), values.iter().max()) {
        (Some(minimum), Some(maximum)) => {
            let sum: i128 = values.iter().map(|value| i128::from(*value)).sum();
            format!("{}/{}/{}", format_ratio(sum, i128::from(signed(u64_of(values.len())))), minimum, maximum)
        }
        (None, _) | (_, None) => "none/none/none".to_string(),
    }
}

/// 一组有理数（每个按四位小数取整之后）的均值、最小、最大。
fn mean_minimum_maximum_of_ratios(ratios: &[(i64, u64)]) -> String {
    let scaled: Vec<i128> = ratios.iter().map(|(numerator, denominator)| scaled_ratio(i128::from(*numerator), i128::from(*denominator))).collect();
    match (scaled.iter().min(), scaled.iter().max()) {
        (Some(minimum), Some(maximum)) => {
            let sum: i128 = scaled.iter().sum();
            let count = i128::from(signed(u64_of(scaled.len())));
            format!("{}/{}/{}", format_scaled(scaled_ratio(sum, count * FIXED_POINT_SCALE)), format_scaled(*minimum), format_scaled(*maximum))
        }
        (None, _) | (_, None) => "none/none/none".to_string(),
    }
}

/// 一格在一条丙的臂上份额的两种口径。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ShareMeasurement {
    extra_bytes: i128,
    baseline_bytes: i128,
    tier: u8,
}

fn share_of(arm: WriteTally, baseline: WriteTally) -> ShareMeasurement {
    let extra_bytes = i128::from(difference(arm, baseline).bytes_on_both_devices);
    let baseline_bytes = i128::from(signed(baseline.bytes_on_both_devices));
    ShareMeasurement { extra_bytes, baseline_bytes, tier: share_tier(extra_bytes, baseline_bytes, SHARE_TIER_BOUNDS_PARTS_PER_BILLION) }
}

fn format_share(share: &ShareMeasurement) -> String {
    format_ratio(share.extra_bytes * 1_000_000, share.baseline_bytes)
}

/// 一格三条臂跑完之后，跨格汇总要的那几样。
struct CellSummary {
    cell: HistoryCell,
    lines: Vec<String>,
    /// (臂, 口径（0 不含根槽与系统配置槽、1 含）) → 份额。
    shares: BTreeMap<(Arm, usize), ShareMeasurement>,
    /// 臂 → 每次隔离那一次发布的设备写差之和与事件数（算法一）。
    per_event_device_writes: BTreeMap<Arm, (i64, u64)>,
    arms_agree_without_events: Option<bool>,
    schedule_realized: bool,
    pass_counts: BTreeMap<(Arm, CheckerUnderTest, usize), PassAndFalseRedCounts>,
    /// (臂, 变体) → (完好事件数, 期末为 0 的, 期末为正的)。
    trajectory_finals: BTreeMap<(Arm, CheckerUnderTest), (u64, u64, u64)>,
    span_bit_writes_extra_on_average: bool,
}

fn history_line(cell: &HistoryCell, arm: Arm, outcome: &HistoryOutcome) -> String {
    let without = outcome.totals.without_root_and_system_configuration();
    let with = outcome.totals.with_root_and_system_configuration();
    let quarantined_at_the_end = outcome.events.iter().filter(|event| event.record.revival.is_none()).count();
    format!(
        "name=history {} arm={} events={} rollbacks={} revived={} device_writes={} bytes={} bytes_single_device={} device_writes_with_root={} bytes_with_root={} \
         journal_records={} final_txg={} final_allocated_slots_per_device={} final_deferred_slots_per_device={} normal_units={} quarantined_units={} \
         mapping_tree_height={} placements_fell_back={}",
        cell.label(),
        arm.label(),
        outcome.events.len(),
        outcome.rollbacks.len(),
        outcome.rollbacks.iter().map(|rollback| rollback.revived).sum::<u64>(),
        without.device_writes,
        without.bytes_on_both_devices,
        without.bytes_on_both_devices / DEVICE_COUNT,
        with.device_writes,
        with.bytes_on_both_devices,
        outcome.totals.journal_records,
        outcome.final_txg,
        outcome.final_occupied_slots,
        outcome.final_deferred_slots,
        outcome.normal_units,
        quarantined_at_the_end,
        outcome.mapping_tree_height,
        outcome.placements_that_fell_back
    )
}

/// 一条丙的臂比 `no_mark` 多写多少（登记第六节前五行在工作负载格上的量）。
fn extra_writes_line(cell: &HistoryCell, arm: Arm, outcome: &HistoryOutcome, no_mark: &HistoryOutcome, summary: &mut CellSummary) -> String {
    assert_eq!(outcome.events.len(), no_mark.events.len(), "三条臂排的是同一串事件");
    assert_eq!(outcome.rollbacks.len(), no_mark.rollbacks.len(), "三条臂排的是同一串回退");
    let per_event: Vec<WriteDifference> = outcome
        .events
        .iter()
        .zip(no_mark.events.iter())
        .map(|(mine, baseline)| difference(mine.writes.without_root_and_system_configuration(), baseline.writes.without_root_and_system_configuration()))
        .collect();
    let device_writes: Vec<i64> = per_event.iter().map(|value| value.device_writes).collect();
    let bytes: Vec<i64> = per_event.iter().map(|value| value.bytes_on_both_devices).collect();
    let grouped = |leaf_rewritten_anyway: bool| -> String {
        let group: Vec<i64> = per_event
            .iter()
            .zip(no_mark.events.iter())
            .filter(|(_, baseline)| baseline.victim_leaf_rewritten == leaf_rewritten_anyway)
            .map(|(value, _)| value.device_writes)
            .collect();
        let sum: i64 = group.iter().sum();
        format!("{}(n={})", format_ratio(i128::from(sum), i128::from(signed(u64_of(group.len())))), group.len())
    };
    let mut structure_sum_arm = PublishWrites::default();
    let mut structure_sum_baseline = PublishWrites::default();
    for (mine, baseline) in outcome.events.iter().zip(no_mark.events.iter()) {
        structure_sum_arm.add(&mine.writes);
        structure_sum_baseline.add(&baseline.writes);
    }
    let event_count = i128::from(signed(u64_of(outcome.events.len())));
    let whole = difference(outcome.totals.without_root_and_system_configuration(), no_mark.totals.without_root_and_system_configuration());
    let revivals: Vec<(i64, u64)> = outcome
        .rollbacks
        .iter()
        .zip(no_mark.rollbacks.iter())
        .filter(|(_, baseline)| baseline.revived > 0)
        .map(|(mine, baseline)| (difference(mine.writes.without_root_and_system_configuration(), baseline.writes.without_root_and_system_configuration()).device_writes, baseline.revived))
        .collect();
    let revival_bytes: Vec<(i64, u64)> = outcome
        .rollbacks
        .iter()
        .zip(no_mark.rollbacks.iter())
        .filter(|(_, baseline)| baseline.revived > 0)
        .map(|(mine, baseline)| (difference(mine.writes.without_root_and_system_configuration(), baseline.writes.without_root_and_system_configuration()).bytes_on_both_devices, baseline.revived))
        .collect();
    let share_without = share_of(outcome.totals.without_root_and_system_configuration(), no_mark.totals.without_root_and_system_configuration());
    let share_with = share_of(outcome.totals.with_root_and_system_configuration(), no_mark.totals.with_root_and_system_configuration());
    summary.shares.insert((arm, 0), share_without);
    summary.shares.insert((arm, 1), share_with);
    summary.per_event_device_writes.insert(arm, (device_writes.iter().sum(), u64_of(device_writes.len())));
    format!(
        "name=extra_writes {} arm={} events={} per_event_device_writes_mean_min_max={} per_event_bytes_mean_min_max={} \
         per_event_device_writes_mean_when_the_leaf_is_rewritten_anyway={} per_event_device_writes_mean_when_the_leaf_is_not={} \
         per_event_structure_sums {} whole_history_device_writes_per_event={} whole_history_bytes_per_event={} \
         share_without_root_ppm={} tier_without_root={} share_with_root_ppm={} tier_with_root={} \
         revival_device_writes_per_revived_mean_min_max={} revival_bytes_per_revived_mean_min_max={}",
        cell.label(),
        arm.label(),
        outcome.events.len(),
        mean_minimum_maximum(&device_writes),
        mean_minimum_maximum(&bytes),
        grouped(true),
        grouped(false),
        structure_differences(&structure_sum_arm, &structure_sum_baseline),
        format_ratio(i128::from(whole.device_writes), event_count),
        format_ratio(i128::from(whole.bytes_on_both_devices), event_count),
        format_share(&share_without),
        share_without.tier,
        format_share(&share_with),
        share_with.tier,
        mean_minimum_maximum_of_ratios(&revivals),
        mean_minimum_maximum_of_ratios(&revival_bytes)
    )
}

/// 终态上的放过与误红：每个仍隔离的单元按副本与年龄归类、正常单元一类，每类每种记号状态各一张盘面（登记第六节 `pass_and_false_red_counts`）。
fn terminal_pass_and_false_red(cell: &HistoryCell, arm: Arm, outcome: &HistoryOutcome, summary: &mut CellSummary) -> Vec<String> {
    let mut lines = Vec::new();
    for checker in checkers_of(arm) {
        let mut counts = [PassAndFalseRedCounts::default(); 2];
        for mark_state in MARK_STATES {
            let normal_verdict = judge_record(*checker, &constructed_view(arm, RecordClass::Normal, mark_state, JudgedVersion::TheQuarantineVersion));
            counts[universe_of(mark_state)].count(is_a_legal_disk(RecordClass::Normal, mark_state), &normal_verdict, outcome.normal_units);
            for event in outcome.events.iter().filter(|event| event.record.revival.is_none()) {
                let class = match event.record.copies {
                    CopyCondition::Intact => RecordClass::QuarantineIntact,
                    CopyCondition::Garbled => RecordClass::QuarantineGarbled,
                };
                let version = judged_version_at_the_end(&event.record, outcome.final_txg, cell.candidate_window);
                let verdict = judge_record(*checker, &constructed_view(arm, class, mark_state, version));
                counts[universe_of(mark_state)].count(is_a_legal_disk(class, mark_state), &verdict, 1);
            }
        }
        for (universe, universe_counts) in counts.iter().enumerate() {
            summary.pass_counts.insert((arm, *checker, universe), *universe_counts);
            lines.push(format!(
                "name=terminal_pass_and_false_red {} arm={} checker={} universe={} leak_units=0 passed_defects={} false_reds={} judged_disks={}",
                cell.label(),
                arm.label(),
                checker.label(),
                UNIVERSE_LABELS[universe],
                universe_counts.passed_defects,
                universe_counts.false_reds,
                universe_counts.judged_disks
            ));
        }
    }
    lines
}

/// 每个 `quarantine_intact` 事件的轨迹，按臂与变体汇总（登记第六节 `no_mark_false_red_trajectory`）。
fn trajectory_lines(cell: &HistoryCell, arm: Arm, outcome: &HistoryOutcome, summary: &mut CellSummary) -> Vec<String> {
    let mut lines = Vec::new();
    for checker in checkers_of(arm) {
        let intact: Vec<EventTrajectory> = outcome
            .events
            .iter()
            .filter(|event| event.record.copies == CopyCondition::Intact)
            .map(|event| trajectory_of_an_event(*checker, &event.record, outcome.final_txg, cell.candidate_window))
            .collect();
        let final_zero = u64_of(intact.iter().filter(|trajectory| trajectory.final_value == 0).count());
        let final_positive = u64_of(intact.len()) - final_zero;
        let positive_versions: Vec<i64> = intact.iter().map(|trajectory| signed(trajectory.positive_versions)).collect();
        let peaks: Vec<i64> = intact.iter().map(|trajectory| signed(trajectory.peak)).collect();
        let first_red_offsets: Vec<i64> = intact.iter().filter_map(|trajectory| trajectory.first_red_offset_of_the_allocated_statistic).map(signed).collect();
        summary.trajectory_finals.insert((arm, *checker), (u64_of(intact.len()), final_zero, final_positive));
        lines.push(format!(
            "name=trajectory {} arm={} checker={} intact_events={} final_value_zero={} final_value_positive={} peak_mean_min_max={} \
             positive_versions_mean_min_max={} allocated_statistic_first_red_offset_mean_min_max={} mark_agreement_red_versions={}",
            cell.label(),
            arm.label(),
            checker.label(),
            intact.len(),
            final_zero,
            final_positive,
            mean_minimum_maximum(&peaks),
            mean_minimum_maximum(&positive_versions),
            mean_minimum_maximum(&first_red_offsets),
            intact.iter().map(|trajectory| trajectory.mark_agreement_red_versions).sum::<u64>()
        ));
    }
    lines
}

/// 一格：同一串事件在三条臂上各跑一遍，出这一格的全部产物行与跨格汇总要的数。
fn evaluate_cell(cell: &HistoryCell) -> CellSummary {
    let schedule = schedule_of(cell);
    let outcomes: Vec<(Arm, HistoryOutcome)> = FORK_ARMS.iter().map(|arm| (*arm, run_history(cell, &schedule, *arm))).collect();
    let mut summary = CellSummary {
        cell: *cell,
        lines: Vec::new(),
        shares: BTreeMap::new(),
        per_event_device_writes: BTreeMap::new(),
        arms_agree_without_events: None,
        schedule_realized: true,
        pass_counts: BTreeMap::new(),
        trajectory_finals: BTreeMap::new(),
        span_bit_writes_extra_on_average: false,
    };
    let no_mark = &outcomes[0].1;
    for (arm, outcome) in &outcomes {
        summary.lines.push(history_line(cell, *arm, outcome));
        let expected_events = cell.density * (NORMAL_PUBLISHES_PER_HISTORY / 1000);
        let expected_rollbacks = cell.rollback_rate * ROLLBACKS_AT_RATE_ONE;
        let rollbacks_revive = cell.density == 0 || outcome.rollbacks.iter().all(|rollback| rollback.revived >= 1);
        summary.schedule_realized &= u64_of(outcome.events.len()) == expected_events && u64_of(outcome.rollbacks.len()) == expected_rollbacks && rollbacks_revive;
    }
    for (arm, outcome) in outcomes.iter().skip(1) {
        let line = extra_writes_line(cell, *arm, outcome, no_mark, &mut summary);
        summary.lines.push(line);
    }
    if let Some((sum, count)) = summary.per_event_device_writes.get(&Arm::SpanBit) {
        summary.span_bit_writes_extra_on_average = *count > 0 && *sum > 0;
    }
    if cell.density == 0 && cell.rollback_rate == 0 {
        summary.arms_agree_without_events = Some(outcomes.iter().all(|(_, outcome)| {
            outcome.totals == no_mark.totals && outcome.final_occupied_slots == no_mark.final_occupied_slots && outcome.final_deferred_slots == no_mark.final_deferred_slots
        }));
    }
    for (arm, outcome) in &outcomes {
        let lines = trajectory_lines(cell, *arm, outcome, &mut summary);
        summary.lines.extend(lines);
        let lines = terminal_pass_and_false_red(cell, *arm, outcome, &mut summary);
        summary.lines.extend(lines);
    }
    summary.lines.push(format!(
        "name=cell_checks {} schedule_realized={} arms_agree_without_events={}",
        cell.label(),
        summary.schedule_realized,
        summary.arms_agree_without_events.map_or("not_this_cell".to_string(), |agree| agree.to_string())
    ));
    summary
}

/// 第二段的格按次序切片并行：每个线程从共享的下一个号取格，结果放回格的位置，合并按格序（与线程数无关）。
fn evaluate_cells(cells: &[HistoryCell], threads: usize) -> Vec<CellSummary> {
    let next_cell = std::sync::atomic::AtomicUsize::new(0);
    let slots: Vec<std::sync::Mutex<Option<CellSummary>>> = cells.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1) {
            scope.spawn(|| loop {
                let cell_index = next_cell.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if cell_index >= cells.len() {
                    break;
                }
                let summary = evaluate_cell(&cells[cell_index]);
                // 进度只进标准错误，不进产物。
                eprintln!("e187 第二段：格 {} / {} 跑完（{}）", cell_index + 1, cells.len(), cells[cell_index].label());
                *slots[cell_index].lock().expect("别的线程没在持锁时 panic") = Some(summary);
            });
        }
    });
    slots.into_iter().map(|slot| slot.into_inner().expect("锁没中毒").expect("每一格都评估过")).collect()
}

const SCOPE_LABELS: [&str; 2] = ["without_root_and_system_configuration", "with_root_and_system_configuration"];

/// 两个份额谁大（交叉相乘比，分母都是正数）。
fn share_is_larger(left: &ShareMeasurement, right: &ShareMeasurement) -> bool {
    left.extra_bytes * right.baseline_bytes > right.extra_bytes * left.baseline_bytes
}

fn parts_per_billion_of(share: &ShareMeasurement) -> i128 {
    share.extra_bytes * PARTS_PER_BILLION / share.baseline_bytes
}

/// 第二段跨格汇总的判定（登记第六节、第十节、第十一节）。
struct SegmentTwoJudgements {
    quarantine_cost_negligible_holds: bool,
    span_bit_writes_nothing_extra_holds: bool,
    no_mark_false_red_persists_holds: bool,
    arms_agree_without_events_holds: bool,
    scheduled_events_realized_holds: bool,
}

fn segment_two_summary_lines(summaries: &[CellSummary]) -> (Vec<String>, SegmentTwoJudgements) {
    let mut lines = Vec::new();
    // 份额按密度：那一密度全部格里的最高档与最低档；另按 (U, 几何, K) 分开看档换不换。
    let mut quarantine_cost_negligible_holds = true;
    for arm in MARKING_ARMS {
        for density in DENSITIES_PER_THOUSAND_PUBLISHES {
            for (scope, scope_label) in SCOPE_LABELS.iter().enumerate() {
                let shares: Vec<(&HistoryCell, ShareMeasurement)> = summaries
                    .iter()
                    .filter(|summary| summary.cell.density == density)
                    .map(|summary| (&summary.cell, *summary.shares.get(&(arm, scope)).expect("每一格都算了两种口径")))
                    .collect();
                let mut tier_counts = [0u64; 3];
                for (_, share) in &shares {
                    tier_counts[usize::from(share.tier - 1)] += 1;
                }
                let largest = shares.iter().fold(None, |largest: Option<ShareMeasurement>, (_, share)| match largest {
                    Some(current) if !share_is_larger(share, &current) => Some(current),
                    Some(_) | None => Some(*share),
                });
                let highest_tier = shares.iter().map(|(_, share)| share.tier).max().expect("每个密度都有格");
                let lowest_tier = shares.iter().map(|(_, share)| share.tier).min().expect("每个密度都有格");
                if density == 10 && highest_tier > 1 {
                    quarantine_cost_negligible_holds = false;
                }
                lines.push(format!(
                    "name=share_tier_by_density arm={} density={} scope={} cells={} highest_tier={} lowest_tier={} tier_counts={}/{}/{} largest_share_ppm={}",
                    arm.label(),
                    density,
                    scope_label,
                    shares.len(),
                    highest_tier,
                    lowest_tier,
                    tier_counts[0],
                    tier_counts[1],
                    tier_counts[2],
                    largest.map_or("none".to_string(), |share| format_share(&share))
                ));
                for file_size in FILE_SIZES_IN_DATA_UNITS {
                    for geometry in GEOMETRIES {
                        for candidate_window in CANDIDATE_WINDOWS {
                            let tiers: Vec<u8> = shares
                                .iter()
                                .filter(|(cell, _)| cell.file_size == file_size && cell.geometry == geometry && cell.candidate_window == candidate_window)
                                .map(|(_, share)| share.tier)
                                .collect();
                            lines.push(format!(
                                "name=share_tier_by_point arm={} density={} scope={} file_size={} geometry={} window={} cells={} highest_tier={} lowest_tier={}",
                                arm.label(),
                                density,
                                scope_label,
                                file_size,
                                geometry.label(),
                                candidate_window,
                                tiers.len(),
                                tiers.iter().max().expect("每个点都有格"),
                                tiers.iter().min().expect("每个点都有格")
                            ));
                        }
                    }
                }
            }
        }
    }
    // 甲与乙谁的每次隔离额外写小（算法一，按格的均值比；同号记那一个便宜，换号记依赖取样点）。
    let mut span_bit_cheaper_cells = 0u64;
    let mut quarantine_tree_cheaper_cells = 0u64;
    let mut equal_cells = 0u64;
    for density in DENSITIES_PER_THOUSAND_PUBLISHES.iter().filter(|density| **density > 0) {
        for rollback_rate in ROLLBACK_RATES_PER_THOUSAND_PUBLISHES {
            for file_size in FILE_SIZES_IN_DATA_UNITS {
                for candidate_window in CANDIDATE_WINDOWS {
                    for geometry in GEOMETRIES {
                        let mut seeds_by_reading: [Vec<u64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
                        for summary in summaries.iter().filter(|summary| {
                            summary.cell.density == *density
                                && summary.cell.rollback_rate == rollback_rate
                                && summary.cell.file_size == file_size
                                && summary.cell.candidate_window == candidate_window
                                && summary.cell.geometry == geometry
                        }) {
                            let (span_bit_sum, _) = summary.per_event_device_writes[&Arm::SpanBit];
                            let (quarantine_tree_sum, _) = summary.per_event_device_writes[&Arm::QuarantineTree];
                            let reading = match span_bit_sum.cmp(&quarantine_tree_sum) {
                                std::cmp::Ordering::Less => 0,
                                std::cmp::Ordering::Greater => 1,
                                std::cmp::Ordering::Equal => 2,
                            };
                            seeds_by_reading[reading].push(summary.cell.seed);
                        }
                        span_bit_cheaper_cells += u64_of(seeds_by_reading[0].len());
                        quarantine_tree_cheaper_cells += u64_of(seeds_by_reading[1].len());
                        equal_cells += u64_of(seeds_by_reading[2].len());
                        let seeds = |reading: usize| seeds_by_reading[reading].iter().map(u64::to_string).collect::<Vec<String>>().join(",");
                        lines.push(format!(
                            "name=cheaper_placement_by_point density={density} rollback_rate={rollback_rate} file_size={file_size} window={candidate_window} geometry={} \
                             span_bit_cheaper_seeds=[{}] quarantine_tree_cheaper_seeds=[{}] equal_seeds=[{}]",
                            geometry.label(),
                            seeds(0),
                            seeds(1),
                            seeds(2)
                        ));
                    }
                }
            }
        }
    }
    let cheaper_reading = match (span_bit_cheaper_cells > 0, quarantine_tree_cheaper_cells > 0) {
        (true, true) => "depends_on_sampling_point",
        (true, false) => "span_bit_cheaper_everywhere_or_equal",
        (false, true) => "quarantine_tree_cheaper_everywhere_or_equal",
        (false, false) => "equal_everywhere",
    };
    lines.push(format!(
        "name=cheaper_placement_summary span_bit_cheaper_cells={span_bit_cheaper_cells} quarantine_tree_cheaper_cells={quarantine_tree_cheaper_cells} equal_cells={equal_cells} reading={cheaper_reading}"
    ));
    // 终态放过与误红：全部格相加，按臂、变体、宇宙；丙比 no_mark 怎么记。
    let mut totals: BTreeMap<(Arm, CheckerUnderTest, usize), PassAndFalseRedCounts> = BTreeMap::new();
    for summary in summaries {
        for (key, counts) in &summary.pass_counts {
            let total = totals.entry(*key).or_default();
            total.passed_defects += counts.passed_defects;
            total.false_reds += counts.false_reds;
            total.judged_disks += counts.judged_disks;
        }
    }
    for ((arm, checker, universe), counts) in &totals {
        let reading = match arm {
            Arm::SpanBit | Arm::QuarantineTree => pass_and_false_red_reading(*counts, totals[&(Arm::NoMark, CheckerUnderTest::Today, *universe)]),
            Arm::NoMark | Arm::PlainRelease => "baseline",
        };
        lines.push(format!(
            "name=terminal_pass_and_false_red_total arm={} checker={} universe={} passed_defects={} false_reds={} judged_disks={} reading={reading}",
            arm.label(),
            checker.label(),
            UNIVERSE_LABELS[*universe],
            counts.passed_defects,
            counts.false_reds,
            counts.judged_disks
        ));
    }
    // 轨迹：期末为 0（误红自己消掉）与期末仍为正的事件数，全部格相加。
    let mut no_mark_false_red_persists_holds = true;
    let mut trajectory_totals: BTreeMap<(Arm, CheckerUnderTest), (u64, u64, u64)> = BTreeMap::new();
    for summary in summaries {
        for (key, (events, final_zero, final_positive)) in &summary.trajectory_finals {
            let total = trajectory_totals.entry(*key).or_default();
            total.0 += events;
            total.1 += final_zero;
            total.2 += final_positive;
        }
    }
    for ((arm, checker), (events, final_zero, final_positive)) in &trajectory_totals {
        if *arm == Arm::NoMark && *final_zero > 0 {
            no_mark_false_red_persists_holds = false;
        }
        lines.push(format!(
            "name=trajectory_total arm={} checker={} intact_events={events} final_value_zero={final_zero} final_value_positive={final_positive}",
            arm.label(),
            checker.label()
        ));
    }
    // 判别力自证（登记第八节末段）：把档界挪到 U = 200 与 U = 20000 两点的甲份额之间，两点必须落进不同档；挪回原档界照原样判。
    let probe = |file_size: u32| -> ShareMeasurement {
        let summary = summaries
            .iter()
            .find(|summary| {
                summary.cell.density == 10
                    && summary.cell.rollback_rate == 0
                    && summary.cell.file_size == file_size
                    && summary.cell.candidate_window == CANDIDATE_WINDOWS[0]
                    && summary.cell.geometry == PoolGeometry::DefaultFourGibibyteDevices
                    && summary.cell.seed == SEEDS[0]
            })
            .expect("判别力自证那两格在格表里");
        summary.shares[&(Arm::SpanBit, 0)]
    };
    let small_file = probe(FILE_SIZES_IN_DATA_UNITS[0]);
    let large_file = probe(FILE_SIZES_IN_DATA_UNITS[1]);
    let (small_parts, large_parts) = (parts_per_billion_of(&small_file), parts_per_billion_of(&large_file));
    let moved_bound = u64::try_from((small_parts + large_parts) / 2 + 1).unwrap_or(1);
    let moved = ShareTierBounds { tier_two_from: moved_bound.max(1), tier_three_from: u64::MAX / 2 };
    let moved_tiers = (share_tier(small_file.extra_bytes, small_file.baseline_bytes, moved), share_tier(large_file.extra_bytes, large_file.baseline_bytes, moved));
    let restored_tiers = (
        share_tier(small_file.extra_bytes, small_file.baseline_bytes, SHARE_TIER_BOUNDS_PARTS_PER_BILLION),
        share_tier(large_file.extra_bytes, large_file.baseline_bytes, SHARE_TIER_BOUNDS_PARTS_PER_BILLION),
    );
    lines.push(format!(
        "name=share_tier_discrimination arm=span_bit density=10 rollback_rate=0 window={} geometry=geometry_default seed={} scope=without_root_and_system_configuration \
         small_file_share_ppb={small_parts} large_file_share_ppb={large_parts} moved_tier_two_from_ppb={moved_bound} moved_tiers={}/{} restored_tiers={}/{} \
         separates_when_moved={} restored_equals_original={}",
        CANDIDATE_WINDOWS[0],
        SEEDS[0],
        moved_tiers.0,
        moved_tiers.1,
        restored_tiers.0,
        restored_tiers.1,
        moved_tiers.0 != moved_tiers.1,
        restored_tiers.0 == small_file.tier && restored_tiers.1 == large_file.tier
    ));
    let judgements = SegmentTwoJudgements {
        quarantine_cost_negligible_holds,
        span_bit_writes_nothing_extra_holds: summaries.iter().all(|summary| !summary.span_bit_writes_extra_on_average),
        no_mark_false_red_persists_holds,
        arms_agree_without_events_holds: summaries.iter().filter_map(|summary| summary.arms_agree_without_events).all(|agree| agree),
        scheduled_events_realized_holds: summaries.iter().all(|summary| summary.schedule_realized),
    };
    (lines, judgements)
}

fn run_segment_two(threads: usize) -> Vec<String> {
    let cells = all_history_cells();
    let summaries = evaluate_cells(&cells, threads);
    let mut lines = vec![format!(
        "name=segment_two_config cells={} arms={} normal_publishes_per_history={NORMAL_PUBLISHES_PER_HISTORY} overwrites_per_publish={OVERWRITES_PER_PUBLISH} \
         densities_per_thousand={:?} rollback_rates_per_thousand={:?} file_sizes={:?} windows={:?} seeds={:?} threads={threads} \
         counted_writes=every_publish_after_the_file_creation normal_units=unreleased_records_of_live_units_without_the_two_make_filesystem_records leak_units=0",
        cells.len(),
        FORK_ARMS.iter().map(|arm| arm.label()).collect::<Vec<&str>>().join(","),
        DENSITIES_PER_THOUSAND_PUBLISHES,
        ROLLBACK_RATES_PER_THOUSAND_PUBLISHES,
        FILE_SIZES_IN_DATA_UNITS,
        CANDIDATE_WINDOWS,
        SEEDS
    )];
    for summary in &summaries {
        lines.extend(summary.lines.iter().cloned());
    }
    let (summary_lines, judgements) = segment_two_summary_lines(&summaries);
    lines.extend(summary_lines);
    lines.push(format!(
        "name=verdict segment=two quarantine_cost_negligible_holds={} span_bit_writes_nothing_extra_holds={} no_mark_false_red_persists_holds={} \
         arms_agree_without_events={} scheduled_events_realized={} unreadable_counts=zero",
        judgements.quarantine_cost_negligible_holds,
        judgements.span_bit_writes_nothing_extra_holds,
        judgements.no_mark_false_red_persists_holds,
        judgements.arms_agree_without_events_holds,
        judgements.scheduled_events_realized_holds
    ));
    lines
}

/// 停机条款（登记第十一节）的格式常量那一半：装置里的常量在 `crates/` 源码原文里逐字找得到。
const CRATES_FORMAT_SOURCE: &str = include_str!("../../crates/singlefs-format/src/lib.rs");
const CRATES_ALLOCATOR_SOURCE: &str = include_str!("../../crates/singlefs-core/src/allocator.rs");
const CRATES_CHECKER_WALK_SOURCE: &str = include_str!("../../crates/singlefs-checker/src/walk.rs");
const CRATES_GPU_JUDGE_SOURCE: &str = include_str!("../../crates/singlefs-checker-tier/src/crash_judge_common.wgsl");

fn crates_source_checks() -> Vec<(&'static str, &'static str, String, bool)> {
    let format_lines = [
        format!("pub const ALLOCATION_RECORD_BYTES: u64 = {ALLOCATION_RECORD_BYTES};"),
        format!("pub const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = {ALLOCATION_RECORD_TREE_LEAF_SLOTS};"),
        format!("pub const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES: u64 = {ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES};"),
        format!("pub const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = {ALLOCATION_RECORD_TREE_INTERNAL_FANOUT};"),
        format!("pub const ALLOCATION_RECORDS_TREE_INDEX_NODE_HEADER_BYTES: u64 = {ALLOCATION_RECORD_TREE_NODE_HEADER_BYTES};"),
        format!("pub const CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES: u64 = {MAPPING_NODE_HEADER_BYTES};"),
        format!("pub const MAPPING_ENTRY_BYTES: u64 = {MAPPING_ENTRY_BYTES};"),
        format!("pub const CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES: u64 = {MAPPING_INTERNAL_ENTRY_BYTES};"),
        format!("pub const JOURNAL_RECORD_BYTES: u64 = {JOURNAL_RECORD_BYTES};"),
        format!("pub const JOURNAL_HEADER_BYTES: u64 = {JOURNAL_HEADER_BYTES};"),
        format!("pub const JOURNAL_NAMED_ENTRY_BYTES: u64 = {JOURNAL_NAMED_ENTRY_BYTES};"),
        format!("pub const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = {JOURNAL_NAMED_ENTRIES_PER_RECORD};"),
        format!("pub const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = {SYSTEM_CONFIGURATION_SLOT_BYTES};"),
        format!("pub const UNIT_AREA_START_SLOT: u64 = {UNIT_AREA_START_SLOT};"),
        format!("pub const CLUSTER_SEGMENT_SLOTS: u64 = {CLUSTER_SEGMENT_SLOTS};"),
        format!("pub const TREE_TABLE_ENTRY_BYTES: u64 = {TREE_TABLE_ENTRY_BYTES};"),
        format!("pub const TREE_TABLE_UNIT_INDEX_NODE_HEADER_BYTES: u64 = {TREE_TABLE_HEADER_BYTES};"),
        format!("pub const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = {NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES};"),
        format!("pub const EXTENT_TREE_INDEX_NODE_HEADER_BYTES: u64 = {EXTENT_NODE_HEADER_BYTES};"),
        format!("pub const EXTENT_TREE_INTERNAL_ENTRY_BYTES: u64 = {EXTENT_INTERNAL_ENTRY_BYTES};"),
        format!("pub const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = {EXTENT_LEAF_ENTRIES};"),
        format!("pub const EXTENT_TREE_INTERNAL_FANOUT: u64 = {EXTENT_INTERNAL_ENTRIES};"),
        format!("pub const TREE_IDENTIFIER_EXTENT: u64 = {EXTENT_TREE};"),
        format!("pub const TREE_IDENTIFIER_INODE: u64 = {INODE_TREE};"),
        format!("pub const TREE_IDENTIFIER_ALLOCATION_RECORDS: u64 = {ALLOCATION_RECORD_TREE};"),
        format!("pub const TREE_IDENTIFIER_ACCOUNTING: u64 = {ACCOUNTING_TREE};"),
        format!("pub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = {QUARANTINE_TREE};"),
    ];
    let mut checks: Vec<(&'static str, &'static str, String, bool)> = format_lines
        .into_iter()
        .map(|line| {
            let found = CRATES_FORMAT_SOURCE.lines().any(|source_line| source_line.trim() == line);
            ("format_constant", "crates/singlefs-format/src/lib.rs", line, found)
        })
        .collect();
    let released_flag_line = format!("const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x{ALLOCATION_RECORD_RELEASED_FLAG:04X};");
    checks.push((
        "released_flag",
        "crates/singlefs-core/src/allocator.rs",
        format!("pub {released_flag_line}"),
        CRATES_ALLOCATOR_SOURCE.lines().any(|source_line| source_line.trim() == format!("pub {released_flag_line}")),
    ));
    checks.push((
        "released_flag",
        "crates/singlefs-checker/src/walk.rs",
        released_flag_line.clone(),
        CRATES_CHECKER_WALK_SOURCE.lines().any(|source_line| source_line.trim() == released_flag_line),
    ));
    let gpu_span_line = format!("let span = span_field & 0x{:04X}u;", !(ALLOCATION_RECORD_RELEASED_FLAG | QUARANTINE_MARK_BIT));
    checks.push((
        "gpu_span_mask",
        "crates/singlefs-checker-tier/src/crash_judge_common.wgsl",
        gpu_span_line.clone(),
        CRATES_GPU_JUDGE_SOURCE.lines().any(|source_line| source_line.trim() == gpu_span_line),
    ));
    checks
}

/// 停机条款的读码那一半（登记第十一节逐行）：执行员 2026-10-21 读此刻工作区的 `crates/` 写下的事实，行号是当天现取的。
/// 模型照这些写；任一条对不上就停机交回，不跑产物。
const CODE_READING_STOP_CHECKS: [(&str, &str, &str); 13] = [
    ("quarantine_leaves_both_records_untouched", "crates/singlefs-core/src/allocator.rs:2109 release_leaving_the_record_allocated_on; crates/singlefs-core/src/transaction.rs:12937; transaction.rs:4876 copies_failing_the_release_checksum_check", "devices_whose_record_stays_allocated_are_skipped_record_unchanged_both_copies_returned_when_either_fails_mapping_entry_dropped"),
    ("arms_share_accounting_mapping_and_tree_table_changes", "crates/singlefs-core/src/transaction.rs:8230; transaction.rs:12928-12948", "accounting_tree_rewritten_every_publish_quarantined_copies_stay_allocated_in_the_same_allocator"),
    ("allocation_record_tree_by_position", "crates/singlefs-core/src/allocation_record_tree.rs:1-15 273 365; crates/singlefs-format/src/lib.rs:163 169", "leaf_812_fanout_169_root_level_formula_changed_leaf_plus_all_ancestors_and_root_empty_leaves_not_written"),
    ("allocation_record_tree_enters_the_mapping", "crates/singlefs-core/src/transaction.rs:3226", "data_extent_inode_allocation_and_accounting_nodes_mapped_mapping_tree_and_tree_table_exempt"),
    ("mapping_key_order", "crates/singlefs-core/src/records.rs:488 mapping_key_sort_key", "class_birth_tree_birth_txg_instance_tail"),
    ("code_two_tree_rules", "crates/singlefs-core/src/code_two_tree.rs:1-17 637 plan_the_tree_after_this_publish; crates/singlefs-core/src/unit.rs:155", "deletes_then_inserts_in_key_order_split_in_the_middle_left_ceiling_half_only_empty_nodes_removed_root_with_one_child_collapses_capacity_node_minus_header_over_entry"),
    ("journal_named_roles", "crates/singlefs-core/src/transaction.rs:11578 roles_named_by_each_record_of_the_publish; crates/singlefs-format/src/lib.rs:253", "one_record_per_data_unit_but_the_last_shared_roles_named_in_the_last_transaction_64_per_record"),
    ("placements", "crates/singlefs-core/src/allocator.rs:2193 2245 2327 933 1737", "data_lowest_two_free_slots_even_outside_every_cluster_segment_opened_in_this_mount_commit_generated_bumped_in_the_open_segment_new_segment_lowest_with_no_occupied_slot"),
    ("reclaim_floor_and_defer", "crates/singlefs-core/src/allocator.rs:2374 reclaim_released_up_to", "released_records_with_release_generation_up_to_the_floor_return_to_free_record_kept_until_reallocated"),
    ("rollback_revives_only_released_records", "crates/singlefs-core/src/mount.rs:9237 roll_back_by_a_forward_publish", "current_only_units_released_target_units_released_in_the_current_ledger_resurrected_extent_and_inode_trees_taken_from_the_target"),
    ("writes_per_device", "crates/singlefs-core/src/transaction.rs:1439 persist_publish_writes 1593", "unit_to_every_device_journal_record_to_every_device_root_slot_to_one_region_device_system_configuration_slot_to_every_device"),
    ("checker_isolation_criterion", "crates/singlefs-checker/src/walk.rs:16223 some_copy_of_the_unit_fails_its_location_entries", "exempt_when_some_copy_is_unreadable_or_fails_its_location_entry_checksum_falls_back_to_self_verification_without_one"),
    ("old_readers", "allocator.rs:80 parse; allocation_record_tree.rs:273 738; recovery.rs:2361 2512; walk.rs:4394 5293 5752 6171; crash_judge_common.wgsl:2566", "see_old_reader_field_table_lines"),
];

/// 落点甲借位之后五处旧读者的读法（登记第六节 `old_reader_field_table`，读码核）：跨度段 0x4001 / 0x4002 时读出的五个字段与接着做什么。
const OLD_READER_FIELD_TABLE: [(&str, &str, &str, &str); 5] = [
    (
        "core_record_parser",
        "crates/singlefs-core/src/allocator.rs:80 AllocationRecord::parse",
        "device=as_written slot=as_written span=16385_or_16386 generation=as_written_allocation_generation released=false",
        "returns_the_record_to_its_caller_no_judgement_here",
    ),
    (
        "core_allocation_record_tree_reader",
        "crates/singlefs-core/src/allocation_record_tree.rs:273 record_fits_in_its_leaf 738",
        "device=as_written slot=as_written span=16385_or_16386 generation=as_written released=false",
        "refuses_the_tree_with_AllocationRecordOutsideThePoolGeometry_mount_and_recovery_fail",
    ),
    (
        "core_recovery_geometry",
        "crates/singlefs-core/src/recovery.rs:2361 allocation_records_fit_the_pool_geometry 2512",
        "device=as_written slot=as_written span=16385_or_16386",
        "reached_only_after_the_tree_reader_accepted_the_records_so_never_reached_for_these_records_standalone_it_refuses_only_on_an_overlap_or_past_the_unit_area_end",
    ),
    (
        "pool_checker",
        "crates/singlefs-checker/src/walk.rs:5293 parse_allocation_record 4394 6171 5752",
        "device=as_written slot=as_written span=16385_or_16386 generation=as_written released=false",
        "judges_I-1.1_red_in_the_judging_walk_and_the_non_judging_reader_keeps_the_record_with_span_16385_in_the_references_used_by_the_accounting_judgements",
    ),
    (
        "gpu_crash_judge",
        "crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2566",
        "device=as_written slot=as_written span=16385_or_16386 generation=as_written span_field_kept_raw",
        "stops_the_walk_at_FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY",
    ),
];

/// 第七节「出自被测条款本身」的钉值（不符走 `clause_anchor_mismatch`，不作废）。
fn clause_anchor_checks() -> Vec<(&'static str, bool, String)> {
    let first_version_spans_leave_the_mark_bit_clear = FIRST_VERSION_SPANS.iter().all(|span| span & QUARANTINE_MARK_BIT == 0);
    let new_pool = new_pool_file_creation_open_segment();
    vec![
        (
            "allocation_record_bytes",
            ALLOCATION_RECORD_BYTES == ALLOCATION_RECORD_KEY_BYTES + 10 && ALLOCATION_RECORD_SPAN_FIELD_OFFSET == 10,
            format!("record={ALLOCATION_RECORD_BYTES} key={ALLOCATION_RECORD_KEY_BYTES} span_field_offset={ALLOCATION_RECORD_SPAN_FIELD_OFFSET}"),
        ),
        ("released_flag", ALLOCATION_RECORD_RELEASED_FLAG == 0x8000, format!("flag=0x{ALLOCATION_RECORD_RELEASED_FLAG:04X}")),
        ("span_values_of_the_first_version", first_version_spans_leave_the_mark_bit_clear, format!("spans={FIRST_VERSION_SPANS:?} mark_bit=0x{QUARANTINE_MARK_BIT:04X}")),
        ("records_kept_per_quarantine", quarantine_entries_of(0).len() == 2, "records_kept=2".to_string()),
        ("mapping_entry_removed_on_quarantine", mapping_entry_removed_on_quarantine_in_every_arm(), "every_fork_arm_drops_the_victim_key".to_string()),
        ("segment_slots", CLUSTER_SEGMENT_SLOTS == 64 && DATA_UNIT_ALIGNMENT_SLOTS == 2, format!("segment={CLUSTER_SEGMENT_SLOTS} data_unit_alignment_slots={DATA_UNIT_ALIGNMENT_SLOTS}")),
        (
            "new_pool_open_segment",
            new_pool == (50240, 50304, 2),
            format!("open_segment=[{},{}) root_level={}", new_pool.0, new_pool.1, new_pool.2),
        ),
    ]
}

/// 32768 字节对齐 = 两个 16 KiB 槽（D3（空间分配） 已定项 10 ③）。
const DATA_UNIT_ALIGNMENT_SLOTS: u64 = 32768 / SLOT_BYTES;

/// 新池新建文件（一个数据单元）之后模型开的段与分配记录树根层级（D3（空间分配） 已定项 10 ⑤ 那张表的段与 t9）。
fn new_pool_file_creation_open_segment() -> (u64, u64, u8) {
    let mut state = PoolState::after_make_filesystem(PoolGeometry::DefaultFourGibibyteDevices, MICRO_CANDIDATE_WINDOW, false);
    publish(&mut state, &PublishRequest::plain(PublishKind::CreateFile { data_unit_count: 1 }), Arm::NoMark);
    let start = state.open_segment_start.expect("新建文件开了一段装提交内生块");
    (start, start + CLUSTER_SEGMENT_SLOTS, state.root_level)
}

/// 三条臂隔离那一次发布之后，被隔离单元那把映射 key 都不在映射里（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）。
fn mapping_entry_removed_on_quarantine_in_every_arm() -> bool {
    FORK_ARMS.iter().all(|arm| {
        let (mut state, request) = prepare_micro_history(MicroHistory::CleanLeaf, *arm);
        let victim = state.version.as_ref().expect("文件建好了").data_units[usize_of(u64::from(MICRO_VICTIM))];
        let victim_key = state.unit(victim).mapping_key.expect("数据单元进映射");
        let had_the_key = tree_contains(&state.mapping, &victim_key);
        publish(&mut state, &request, *arm);
        had_the_key && !tree_contains(&state.mapping, &victim_key) && state.quarantined.contains_key(&victim)
    })
}

/// 第七节「独立算出、用命令核过的」钉值（不符就作废）。
fn independent_anchor_checks() -> Vec<(&'static str, bool, String)> {
    let span_with_high_bit = [old_reader_arithmetic(QUARANTINE_MARK_BIT, 1), old_reader_arithmetic(QUARANTINE_MARK_BIT, 2)];
    let span_with_low_bit = [old_reader_arithmetic(LOW_SPAN_BIT_PROBE, 1), old_reader_arithmetic(LOW_SPAN_BIT_PROBE, 2)];
    let leaf_range = |leaf: u64| (leaf * ALLOCATION_RECORD_TREE_LEAF_SLOTS, (leaf + 1) * ALLOCATION_RECORD_TREE_LEAF_SLOTS);
    vec![
        (
            "allocation_leaf_capacity",
            ALLOCATION_RECORD_TREE_LEAF_ENTRIES == 812 && ALLOCATION_RECORD_TREE_INTERNAL_FANOUT == 169,
            format!("leaf={ALLOCATION_RECORD_TREE_LEAF_ENTRIES} fanout={ALLOCATION_RECORD_TREE_INTERNAL_FANOUT}"),
        ),
        ("mapping_capacities", MAPPING_LEAF_ENTRIES == 294 && MAPPING_INTERNAL_ENTRIES == 143, format!("leaf={MAPPING_LEAF_ENTRIES} internal={MAPPING_INTERNAL_ENTRIES}")),
        ("quarantine_tree_leaf_capacity", QUARANTINE_TREE_LEAF_ENTRIES == 812, format!("leaf={QUARANTINE_TREE_LEAF_ENTRIES}")),
        ("tree_table_unit_capacity", TREE_TABLE_UNIT_ENTRIES == 81, format!("entries={TREE_TABLE_UNIT_ENTRIES}")),
        (
            "extent_capacities",
            EXTENT_LEAF_ENTRIES == 144 && EXTENT_INTERNAL_ENTRIES == 147 && EXTENT_LEAF_ENTRIES * EXTENT_INTERNAL_ENTRIES == 21168,
            format!("leaf={EXTENT_LEAF_ENTRIES} internal={EXTENT_INTERNAL_ENTRIES} two_levels={}", EXTENT_LEAF_ENTRIES * EXTENT_INTERNAL_ENTRIES),
        ),
        ("journal_names_per_record", JOURNAL_NAMED_ENTRIES_PER_RECORD == 64, format!("names={JOURNAL_NAMED_ENTRIES_PER_RECORD}")),
        (
            "root_levels",
            allocation_record_tree_root_level(PoolGeometry::DefaultFourGibibyteDevices.slots_per_device()) == 2
                && allocation_record_tree_root_level(PoolGeometry::WideOneTebibyteDevices.slots_per_device()) == 3,
            format!(
                "default={} wide={}",
                allocation_record_tree_root_level(PoolGeometry::DefaultFourGibibyteDevices.slots_per_device()),
                allocation_record_tree_root_level(PoolGeometry::WideOneTebibyteDevices.slots_per_device())
            ),
        ),
        (
            "leaf_ranges",
            leaf_range(62) == (50344, 51156)
                && leaf_range(70) == (56840, 57652)
                && leaf_range(75) == (60900, 61712)
                && leaf_of_slot(140000) == 172
                && span_in_slots_at_level(1) == 137228,
            format!("leaf62={:?} leaf70={:?} leaf75={:?} leaf_of_140000={} level_one_span={}", leaf_range(62), leaf_range(70), leaf_range(75), leaf_of_slot(140000), span_in_slots_at_level(1)),
        ),
        (
            "old_reader_span_with_high_bit",
            span_with_high_bit[0].span_read == 16385 && span_with_high_bit[1].span_read == 16386 && span_with_high_bit.iter().all(|arithmetic| arithmetic.accepted == 0),
            format!("spans_read={}/{} accepted={}/{}", span_with_high_bit[0].span_read, span_with_high_bit[1].span_read, span_with_high_bit[0].accepted, span_with_high_bit[1].accepted),
        ),
        (
            "old_reader_span_with_low_bit",
            span_with_low_bit[0].span_read == 5
                && span_with_low_bit[1].span_read == 6
                && span_with_low_bit[0].legal_starts == 812
                && span_with_low_bit[0].accepted == 808
                && span_with_low_bit[1].legal_starts == 406
                && span_with_low_bit[1].accepted == 404,
            format!(
                "spans_read={}/{} accepted={}_of_{}/{}_of_{}",
                span_with_low_bit[0].span_read,
                span_with_low_bit[1].span_read,
                span_with_low_bit[0].accepted,
                span_with_low_bit[0].legal_starts,
                span_with_low_bit[1].accepted,
                span_with_low_bit[1].legal_starts
            ),
        ),
        (
            "logical_bytes_per_quarantine",
            logical_bytes_changed_by_the_span_bit() == 2 && 2 * QUARANTINE_TREE_ENTRY_BYTES == 40 && TREE_TABLE_ENTRY_BYTES == 200,
            format!("span_bit={} quarantine_tree={}+first_time_{}", logical_bytes_changed_by_the_span_bit(), 2 * QUARANTINE_TREE_ENTRY_BYTES, TREE_TABLE_ENTRY_BYTES),
        ),
    ]
}

/// 甲改的逻辑字节：两条记录跨度段小端的高字节由 0x00 变 0x40。
fn logical_bytes_changed_by_the_span_bit() -> u64 {
    let record = AllocationRecord { span_slots: 2, is_released: false, generation: 7, carries_the_quarantine_mark: false };
    let marked = AllocationRecord { span_slots: record.span_slots, is_released: record.is_released, generation: record.generation, carries_the_quarantine_mark: true };
    let changed_bytes_per_record = record.span_field().to_le_bytes().iter().zip(marked.span_field().to_le_bytes().iter()).filter(|(before, after)| before != after).count();
    DEVICE_COUNT * u64_of(changed_bytes_per_record)
}

/// 读码核的结论（`walk.rs` 5752、6171 那两处）：池级 checker 判 I-1.1 红的同时，不判的那一遍照收这条记录、按读错的跨度给记账判据用。
const POOL_CHECKER_KEEPS_THE_MISREAD_SPAN_FOR_THE_ACCOUNTING_JUDGEMENTS: bool = true;

fn micro_lines(micro: MicroHistory, measurements: &[MicroArmMeasurement]) -> Vec<String> {
    let no_mark = measurements.iter().find(|measurement| measurement.arm == Arm::NoMark).expect("每段微历史都跑 no_mark");
    measurements
        .iter()
        .map(|measurement| {
            let total = measurement.writes.without_root_and_system_configuration();
            let extra = micro_extra(measurements, measurement.arm);
            format!(
                "name=micro history={} arm={} device_writes={} bytes={} journal_records={} last_transaction_named_roles={} padding_named_roles={} victim_leaf_rewritten={} \
                 extra_device_writes={} extra_bytes={} extra_bytes_single_device={} {} logical_tree_table_bytes_added={} mark_agreement_red_after_revival={}",
                micro.label(),
                measurement.arm.label(),
                total.device_writes,
                total.bytes_on_both_devices,
                measurement.writes.journal_records,
                measurement.writes.last_transaction_named_roles,
                measurement.padding_named_roles,
                measurement.victim_leaf_rewritten,
                extra.device_writes,
                extra.bytes_on_both_devices,
                extra.bytes_on_both_devices / 2,
                structure_differences(&measurement.writes, &no_mark.writes),
                measurement.writes.logical_tree_table_bytes_added,
                measurement.mark_agreement_red_after_revival.map_or("not_a_revival".to_string(), |red| red.to_string())
            )
        })
        .collect()
}

/// 第一段的判定（登记第六节、第十节、第十一节）。
struct SegmentOneJudgements {
    stop_checks_hold: bool,
    clause_anchors_hold: bool,
    independent_anchors_hold: bool,
    micro_pins_hold: bool,
    positive_controls_hold: bool,
    revival_clears_the_mark: bool,
    old_readers_reject_the_mark_holds: bool,
    mark_hides_a_leak_holds: bool,
    mark_removes_the_false_red_holds: bool,
    span_bit_writes_nothing_extra_on_the_micro_histories_holds: bool,
}

fn run_segment_one() -> (Vec<String>, SegmentOneJudgements) {
    let mut lines = Vec::new();
    let mut stop_checks_hold = true;
    for (item, origin, fact) in CODE_READING_STOP_CHECKS {
        lines.push(format!("name=stop_check item={item} kind=code_reading where={} observed={fact} result=pass", origin.replace(' ', "_")));
    }
    for (item, origin, line, found) in crates_source_checks() {
        stop_checks_hold &= found;
        lines.push(format!("name=stop_check item={item} kind=source_text where={origin} expected_line={} result={}", line.replace(' ', "_"), if found { "pass" } else { "fail" }));
    }
    let mut clause_anchors_hold = true;
    for (name, holds, detail) in clause_anchor_checks() {
        clause_anchors_hold &= holds;
        lines.push(format!("name=clause_anchor anchor={name} holds={holds} {}", detail.replace(' ', "_")));
    }
    let mut independent_anchors_hold = true;
    for (name, holds, detail) in independent_anchor_checks() {
        independent_anchors_hold &= holds;
        lines.push(format!("name=independent_anchor anchor={name} holds={holds} {}", detail.replace(' ', "_")));
    }
    // 八段微历史与写量的阳性对照。
    let mut micro_pins_hold = true;
    let mut positive_controls_hold = true;
    let mut revival_clears_the_mark = true;
    let mut span_bit_writes_nothing_extra_on_the_micro_histories_holds = true;
    for micro in MICRO_HISTORIES {
        let measurements = measure_micro_history(micro);
        lines.extend(micro_lines(micro, &measurements));
        let span_bit = micro_extra(&measurements, Arm::SpanBit);
        let quarantine_tree = micro_extra(&measurements, Arm::QuarantineTree);
        let (span_bit_writes, span_bit_bytes, quarantine_tree_writes, quarantine_tree_bytes) = micro.pinned_extra_writes();
        let holds = span_bit == WriteDifference { device_writes: signed(span_bit_writes), bytes_on_both_devices: signed(span_bit_bytes) }
            && quarantine_tree == WriteDifference { device_writes: signed(quarantine_tree_writes), bytes_on_both_devices: signed(quarantine_tree_bytes) };
        micro_pins_hold &= holds;
        span_bit_writes_nothing_extra_on_the_micro_histories_holds &= span_bit.device_writes <= 0;
        for measurement in &measurements {
            revival_clears_the_mark &= measurement.mark_agreement_red_after_revival != Some(true);
        }
        lines.push(format!(
            "name=micro_pin history={} span_bit_extra={}/{} span_bit_expected={span_bit_writes}/{span_bit_bytes} quarantine_tree_extra={}/{} quarantine_tree_expected={quarantine_tree_writes}/{quarantine_tree_bytes} holds={holds}",
            micro.label(),
            span_bit.device_writes,
            span_bit.bytes_on_both_devices,
            quarantine_tree.device_writes,
            quarantine_tree.bytes_on_both_devices
        ));
        if micro == MicroHistory::CleanLeaf {
            let plain = micro_extra(&measurements, Arm::PlainRelease);
            let counted = plain == WriteDifference { device_writes: 4, bytes_on_both_devices: 65536 } && span_bit.device_writes == 4 && quarantine_tree.device_writes == 2;
            positive_controls_hold &= counted;
            lines.push(format!(
                "name=positive_control control=write_counting history={} plain_release_minus_no_mark={}/{} expected=4/65536 span_bit_minus_no_mark={} quarantine_tree_minus_no_mark={} span_bit_more_than_quarantine_tree={} holds={counted}",
                micro.label(),
                plain.device_writes,
                plain.bytes_on_both_devices,
                span_bit.device_writes,
                quarantine_tree.device_writes,
                span_bit.device_writes > quarantine_tree.device_writes
            ));
        }
        if micro == MicroHistory::SecondQuarantineTreeCold {
            lines.push(format!(
                "name=pairwise_with_absolute history={} quarantine_tree_more_than_span_bit={} quarantine_tree={} span_bit={}",
                micro.label(),
                quarantine_tree.device_writes > span_bit.device_writes,
                quarantine_tree.device_writes,
                span_bit.device_writes
            ));
        }
    }
    // checker 的阳性对照：每条臂、每个变体，泄漏判红、正常判绿（两版都判）。
    for arm in FORK_ARMS {
        for checker in checkers_of(arm) {
            for version in JUDGED_VERSIONS {
                let leak = judge_record(*checker, &constructed_view(arm, RecordClass::Leak, MarkState::WrittenCorrectly, version));
                let normal = judge_record(*checker, &constructed_view(arm, RecordClass::Normal, MarkState::WrittenCorrectly, version));
                let holds = leak.is_red() && !normal.is_red();
                positive_controls_hold &= holds;
                lines.push(format!(
                    "name=positive_control control=checker arm={} checker={} version={} leak={} normal={} holds={holds}",
                    arm.label(),
                    checker.label(),
                    version.label(),
                    leak.label(),
                    normal.label()
                ));
            }
        }
    }
    let positive_control_old_reader = !old_reader_accepts(OLD_READER_POSITIVE_CONTROL_SLOT % ALLOCATION_RECORD_TREE_LEAF_SLOTS, old_reader_span(OLD_READER_POSITIVE_CONTROL_SPAN_FIELD));
    positive_controls_hold &= positive_control_old_reader;
    lines.push(format!(
        "name=positive_control control=old_reader span_field=0x{OLD_READER_POSITIVE_CONTROL_SPAN_FIELD:04X} slot={OLD_READER_POSITIVE_CONTROL_SLOT} span_read={} judged_outside_the_leaf={positive_control_old_reader} holds={positive_control_old_reader}",
        old_reader_span(OLD_READER_POSITIVE_CONTROL_SPAN_FIELD)
    ));
    // 位翻转分类表：臂 × 变体 × 记录类 × 记号状态 × 版本；放过与误红按两个宇宙数。
    let mut constructed_counts: BTreeMap<(Arm, CheckerUnderTest, usize), PassAndFalseRedCounts> = BTreeMap::new();
    let mut mark_removes_the_false_red_holds = true;
    for arm in FORK_ARMS {
        for checker in checkers_of(arm) {
            for class in RECORD_CLASSES {
                for mark_state in MARK_STATES {
                    for version in JUDGED_VERSIONS {
                        let verdict = judge_record(*checker, &constructed_view(arm, class, mark_state, version));
                        let legal = is_a_legal_disk(class, mark_state);
                        let outcome = match (legal, verdict.is_red()) {
                            (true, true) => "false_red",
                            (true, false) => "correct_green",
                            (false, true) => "caught",
                            (false, false) => "passed_defect",
                        };
                        constructed_counts.entry((arm, *checker, universe_of(mark_state))).or_default().count(legal, &verdict, 1);
                        if arm != Arm::NoMark && class == RecordClass::QuarantineIntact && mark_state == MarkState::WrittenCorrectly && verdict.is_red() {
                            mark_removes_the_false_red_holds = false;
                        }
                        lines.push(format!(
                            "name=flip_classification arm={} checker={} class={} mark={} version={} verdict={} isolation_criteria_red={} legal_disk={legal} outcome={outcome}",
                            arm.label(),
                            checker.label(),
                            class.label(),
                            mark_state.label(),
                            version.label(),
                            verdict.label(),
                            verdict.isolation_criteria_red_count()
                        ));
                    }
                }
            }
        }
    }
    lines.push("name=flip_classification_medium arms=no_mark,span_bit,quarantine_tree verdict=I-2.1 note=node_checksum_mismatch_reddens_every_arm_alike_not_counted_in_passes_or_false_reds".to_string());
    let mut mark_hides_a_leak_holds = true;
    for ((arm, checker, universe), counts) in &constructed_counts {
        let baseline = constructed_counts[&(Arm::NoMark, CheckerUnderTest::Today, *universe)];
        let reading = match arm {
            Arm::SpanBit | Arm::QuarantineTree => {
                if counts.passed_defects > baseline.passed_defects {
                    mark_hides_a_leak_holds = false;
                }
                pass_and_false_red_reading(*counts, baseline)
            }
            Arm::NoMark | Arm::PlainRelease => "baseline",
        };
        lines.push(format!(
            "name=constructed_pass_and_false_red arm={} checker={} universe={} passed_defects={} false_reds={} judged_disks={} reading={reading}",
            arm.label(),
            checker.label(),
            UNIVERSE_LABELS[*universe],
            counts.passed_defects,
            counts.false_reds,
            counts.judged_disks
        ));
    }
    // 落点甲的旧读者：算术（装置）与读码（五处）。
    let mut arithmetic_accepts_nothing_with_the_high_bit = true;
    for borrowed_bit in [QUARANTINE_MARK_BIT, LOW_SPAN_BIT_PROBE] {
        for record_span in FIRST_VERSION_SPANS {
            let arithmetic = old_reader_arithmetic(borrowed_bit, record_span);
            if borrowed_bit == QUARANTINE_MARK_BIT {
                arithmetic_accepts_nothing_with_the_high_bit &= arithmetic.accepted == 0;
            }
            lines.push(format!(
                "name=old_reader_arithmetic borrowed_bit=0x{:04X} record_span={} span_field=0x{:04X} span_read={} legal_starts_per_leaf={} accepted={}",
                arithmetic.borrowed_bit,
                arithmetic.record_span,
                arithmetic.record_span | arithmetic.borrowed_bit,
                arithmetic.span_read,
                arithmetic.legal_starts,
                arithmetic.accepted
            ));
        }
    }
    for (reader, origin, fields, action) in OLD_READER_FIELD_TABLE {
        lines.push(format!("name=old_reader_field_table reader={reader} where={} fields={} next={action}", origin.replace(' ', "_"), fields.replace(' ', ",")));
    }
    let old_readers_reject_the_mark_holds = arithmetic_accepts_nothing_with_the_high_bit && !POOL_CHECKER_KEEPS_THE_MISREAD_SPAN_FOR_THE_ACCOUNTING_JUDGEMENTS;
    let judgements = SegmentOneJudgements {
        stop_checks_hold,
        clause_anchors_hold,
        independent_anchors_hold,
        micro_pins_hold,
        positive_controls_hold,
        revival_clears_the_mark,
        old_readers_reject_the_mark_holds,
        mark_hides_a_leak_holds,
        mark_removes_the_false_red_holds,
        span_bit_writes_nothing_extra_on_the_micro_histories_holds,
    };
    lines.push(format!(
        "name=verdict segment=one stop_checks_hold={} clause_anchors_hold={} independent_anchors_hold={} micro_pins_hold={} positive_controls_hold={} \
         revival_clears_the_mark={} old_readers_reject_the_mark_holds={} mark_hides_a_leak_holds={} mark_removes_the_false_red_holds={} \
         span_bit_writes_nothing_extra_on_the_micro_histories_holds={}",
        judgements.stop_checks_hold,
        judgements.clause_anchors_hold,
        judgements.independent_anchors_hold,
        judgements.micro_pins_hold,
        judgements.positive_controls_hold,
        judgements.revival_clears_the_mark,
        judgements.old_readers_reject_the_mark_holds,
        judgements.mark_hides_a_leak_holds,
        judgements.mark_removes_the_false_red_holds,
        judgements.span_bit_writes_nothing_extra_on_the_micro_histories_holds
    ));
    (lines, judgements)
}

/// 线程数：`E187_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，都没设取登记的重活默认并行度（`.claude/heavy-thread-default`）。
fn threads_from_the_environment() -> usize {
    std::env::var("E187_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|threads| *threads >= 1)
        .unwrap_or_else(|| default_heavy_threads(include_str!("../../.claude/heavy-thread-default")))
}

/// 串行段当场声明（`scripts/hosts/core_usage.py` 文件头「声明」）：环境变量没设就不写。
fn declare_the_core_usage(declaration: &str) {
    let Ok(path) = std::env::var("SINGLEFS_SERIAL_DECLARATIONS") else {
        return;
    };
    let seconds = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs());
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        // 声明写不进去只少一行提示，不影响产物。
        let _ignored = writeln!(file, "{seconds}\t{declaration}");
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RunMode {
    BothSegments,
    SegmentOne,
    SegmentTwo,
}

fn run_mode_of(arguments: &[String]) -> Option<RunMode> {
    match arguments {
        [] => Some(RunMode::BothSegments),
        [mode] if mode == "segment-one" => Some(RunMode::SegmentOne),
        [mode] if mode == "segment-two" => Some(RunMode::SegmentTwo),
        _unrecognized => None,
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(mode) = run_mode_of(&arguments) else {
        eprintln!("用法：e187-quarantine-mark-cost [segment-one | segment-two]（不带参数两段都跑）");
        std::process::exit(2);
    };
    let threads = threads_from_the_environment();
    let runs_segment_one = mode != RunMode::SegmentTwo;
    let runs_segment_two = mode != RunMode::SegmentOne;
    let mut lines = vec![format!(
        "name=config experiment=e187_quarantine_mark_cost model=counting_model_written_by_hand registration=research/prompts/e187-preregistration.md \
         segment_one={runs_segment_one} segment_two={runs_segment_two} threads_segment_one=1 threads_segment_two={threads} \
         slot_bytes={SLOT_BYTES} devices={DEVICE_COUNT} journal_record_bytes={JOURNAL_RECORD_BYTES} root_slot_bytes={ROOT_SLOT_BYTES} root_slot_writes_per_publish={ROOT_SLOT_WRITES_PER_PUBLISH} \
         system_configuration_slot_bytes={SYSTEM_CONFIGURATION_SLOT_BYTES} unit_area_start={UNIT_AREA_START_SLOT} segment_slots={CLUSTER_SEGMENT_SLOTS} \
         allocation_leaf_slots={ALLOCATION_RECORD_TREE_LEAF_SLOTS} allocation_fanout={ALLOCATION_RECORD_TREE_INTERNAL_FANOUT} mapping_leaf={MAPPING_LEAF_ENTRIES} mapping_internal={MAPPING_INTERNAL_ENTRIES} \
         quarantine_tree_leaf={QUARANTINE_TREE_LEAF_ENTRIES} extent_leaf={EXTENT_LEAF_ENTRIES} journal_names_per_record={JOURNAL_NAMED_ENTRIES_PER_RECORD} \
         quarantine_tree_identifier={QUARANTINE_TREE} share_tier_bounds_ppb={}/{} bytes_reported=both_devices_with_single_device_alongside",
        SHARE_TIER_BOUNDS_PARTS_PER_BILLION.tier_two_from,
        SHARE_TIER_BOUNDS_PARTS_PER_BILLION.tier_three_from
    )];
    let mut stop_checks_hold = true;
    if runs_segment_one {
        declare_the_core_usage("此刻串行：第一段只有八段微历史与几十张造出来的盘面，单线程几秒跑完");
        let (segment_one_lines, judgements) = run_segment_one();
        declare_the_core_usage("此刻并行");
        stop_checks_hold = judgements.stop_checks_hold;
        lines.extend(segment_one_lines);
    }
    if runs_segment_two {
        if stop_checks_hold {
            lines.extend(run_segment_two(threads));
        } else {
            lines.push("name=segment_two_not_run reason=stop_check_failed".to_string());
        }
    }
    let standard_output = std::io::stdout();
    let mut output = std::io::BufWriter::new(standard_output.lock());
    for line in &lines {
        writeln!(output, "E7RESULT {line}").expect("写得进标准输出");
    }
    writeln!(output, "E7RESULT name=done emitted={}", lines.len() + 1).expect("写得进标准输出");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 第七节「独立算出」的容量、根层级、叶界：常量断言写加法。
    #[test]
    fn capacities_root_levels_and_leaf_ranges_match_the_independently_computed_pins() {
        assert_eq!(ALLOCATION_RECORD_TREE_LEAF_ENTRIES, 800 + 12, "(16384 − 135) ÷ 20");
        assert_eq!(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT, 160 + 9, "(16384 − 135) ÷ 96");
        assert_eq!(MAPPING_LEAF_ENTRIES, 290 + 4, "(16384 − 169) ÷ 55");
        assert_eq!(MAPPING_INTERNAL_ENTRIES, 140 + 3, "(16384 − 169) ÷ 113");
        assert_eq!(QUARANTINE_TREE_LEAF_ENTRIES, 800 + 12, "(16384 − 135) ÷ 20");
        assert_eq!(TREE_TABLE_UNIT_ENTRIES, 80 + 1, "(16384 − 131) ÷ 200");
        assert_eq!(EXTENT_LEAF_ENTRIES, 140 + 4, "(16384 − 163) ÷ 112");
        assert_eq!(EXTENT_INTERNAL_ENTRIES, 140 + 7, "(16384 − 163) ÷ 110");
        assert_eq!(JOURNAL_NAMED_ENTRIES_PER_RECORD, 60 + 4, "(4096 − 487) ÷ 56");
        assert_eq!(allocation_record_tree_root_level(PoolGeometry::DefaultFourGibibyteDevices.slots_per_device()), 1 + 1);
        assert_eq!(allocation_record_tree_root_level(PoolGeometry::WideOneTebibyteDevices.slots_per_device()), 2 + 1);
        assert_eq!(62 * ALLOCATION_RECORD_TREE_LEAF_SLOTS, 50000 + 344);
        assert_eq!(70 * ALLOCATION_RECORD_TREE_LEAF_SLOTS, 56000 + 840);
        assert_eq!(75 * ALLOCATION_RECORD_TREE_LEAF_SLOTS, 60000 + 900);
        assert_eq!(leaf_of_slot(140000), 170 + 2);
        assert_eq!(span_in_slots_at_level(1), 137000 + 228);
    }

    #[test]
    fn every_format_constant_of_the_model_is_written_verbatim_in_the_crates_sources() {
        for (item, origin, line, found) in crates_source_checks() {
            assert!(found, "{item}：{origin} 里没有逐字的「{line}」");
        }
    }

    #[test]
    fn the_borrowed_high_bit_makes_every_record_overrun_its_leaf_for_the_old_reader() {
        for record_span in FIRST_VERSION_SPANS {
            let arithmetic = old_reader_arithmetic(QUARANTINE_MARK_BIT, record_span);
            assert_eq!(arithmetic.span_read, 16384 + u64::from(record_span));
            assert_eq!(arithmetic.accepted, 0, "跨度 16385 / 16386 落不进 812 槽的叶");
        }
    }

    #[test]
    fn the_borrowed_low_bit_keeps_most_records_in_the_leaf_with_a_wrong_span() {
        let single = old_reader_arithmetic(LOW_SPAN_BIT_PROBE, 1);
        let double = old_reader_arithmetic(LOW_SPAN_BIT_PROBE, 2);
        assert_eq!((single.span_read, single.legal_starts, single.accepted), (4 + 1, 812, 800 + 8));
        assert_eq!((double.span_read, double.legal_starts, double.accepted), (4 + 2, 406, 400 + 4));
    }

    #[test]
    fn the_old_reader_positive_control_span_is_judged_outside_the_leaf() {
        assert_eq!(old_reader_span(OLD_READER_POSITIVE_CONTROL_SPAN_FIELD), 32767);
        assert!(!old_reader_accepts(OLD_READER_POSITIVE_CONTROL_SLOT % ALLOCATION_RECORD_TREE_LEAF_SLOTS, old_reader_span(OLD_READER_POSITIVE_CONTROL_SPAN_FIELD)));
        assert!(old_reader_accepts(0, 2), "合法的两槽记录落得进叶");
    }

    /// 一段微历史：甲、乙各减 `no_mark` 等于第七节手推的绝对值。
    fn assert_the_micro_history_pins(micro: MicroHistory) {
        let measurements = measure_micro_history(micro);
        let (span_bit_writes, span_bit_bytes, quarantine_tree_writes, quarantine_tree_bytes) = micro.pinned_extra_writes();
        let span_bit = micro_extra(&measurements, Arm::SpanBit);
        let quarantine_tree = micro_extra(&measurements, Arm::QuarantineTree);
        assert_eq!(span_bit.device_writes, signed(span_bit_writes), "{}：甲多写的设备写", micro.label());
        assert_eq!(span_bit.bytes_on_both_devices, signed(span_bit_bytes), "{}：甲多写的字节", micro.label());
        assert_eq!(quarantine_tree.device_writes, signed(quarantine_tree_writes), "{}：乙多写的设备写", micro.label());
        assert_eq!(quarantine_tree.bytes_on_both_devices, signed(quarantine_tree_bytes), "{}：乙多写的字节", micro.label());
    }

    #[test]
    fn micro_clean_leaf_costs_the_span_bit_two_leaves_and_the_quarantine_tree_one_new_node() {
        assert_the_micro_history_pins(MicroHistory::CleanLeaf);
    }

    #[test]
    fn micro_clean_ancestor_costs_the_span_bit_a_leaf_and_a_level_one_node_on_each_device() {
        assert_the_micro_history_pins(MicroHistory::CleanAncestor);
    }

    #[test]
    fn micro_dirty_leaf_costs_the_span_bit_nothing() {
        assert_the_micro_history_pins(MicroHistory::DirtyLeaf);
    }

    #[test]
    fn micro_journal_crossing_costs_the_span_bit_one_more_journal_record() {
        assert_the_micro_history_pins(MicroHistory::JournalCrossing);
    }

    #[test]
    fn micro_second_quarantine_with_a_cold_tree_node_costs_the_quarantine_tree_the_cold_leaf() {
        assert_the_micro_history_pins(MicroHistory::SecondQuarantineTreeCold);
    }

    #[test]
    fn micro_revival_clean_costs_the_span_bit_the_leaf_and_clears_the_mark() {
        assert_the_micro_history_pins(MicroHistory::RevivalClean);
        let measurements = measure_micro_history(MicroHistory::RevivalClean);
        for measurement in &measurements {
            assert_eq!(measurement.mark_agreement_red_after_revival, Some(false), "{}：回退之后记号清掉了", measurement.arm.label());
        }
    }

    #[test]
    fn micro_revival_with_a_cold_tree_node_costs_both_placements_a_leaf() {
        assert_the_micro_history_pins(MicroHistory::RevivalTreeCold);
    }

    #[test]
    fn micro_release_of_a_quarantined_record_costs_neither_placement_anything() {
        assert_the_micro_history_pins(MicroHistory::ReleaseQuarantined);
    }

    #[test]
    fn plain_release_rewrites_the_victim_leaf_on_both_devices_where_no_mark_does_not() {
        let measurements = measure_micro_history(MicroHistory::CleanLeaf);
        assert_eq!(micro_extra(&measurements, Arm::PlainRelease), WriteDifference { device_writes: 4, bytes_on_both_devices: 4 * 16384 });
        let no_mark = measurements.iter().find(|measurement| measurement.arm == Arm::NoMark).expect("跑了 no_mark");
        let plain = measurements.iter().find(|measurement| measurement.arm == Arm::PlainRelease).expect("跑了 plain_release");
        assert!(!no_mark.victim_leaf_rewritten && plain.victim_leaf_rewritten, "只有照常释放的那一遍改了叶 75");
    }

    #[test]
    fn every_checker_reddens_a_leak_and_passes_a_normal_record() {
        for arm in FORK_ARMS {
            for checker in checkers_of(arm) {
                for version in JUDGED_VERSIONS {
                    assert!(judge_record(*checker, &constructed_view(arm, RecordClass::Leak, MarkState::WrittenCorrectly, version)).is_red(), "{} {} 泄漏判红", arm.label(), checker.label());
                    assert!(!judge_record(*checker, &constructed_view(arm, RecordClass::Normal, MarkState::WrittenCorrectly, version)).is_red(), "{} {} 正常判绿", arm.label(), checker.label());
                }
            }
        }
    }

    #[test]
    fn today_checker_reddens_an_intact_quarantine_and_exempts_a_garbled_one() {
        let intact = judge_record(CheckerUnderTest::Today, &constructed_view(Arm::NoMark, RecordClass::QuarantineIntact, MarkState::WrittenCorrectly, JudgedVersion::TheQuarantineVersion));
        assert_eq!(intact.label(), "I-3.11+I-3.15", "隔离那一版：老根还引用它，I-3.1 不红");
        let after_window = judge_record(CheckerUnderTest::Today, &constructed_view(Arm::NoMark, RecordClass::QuarantineIntact, MarkState::WrittenCorrectly, JudgedVersion::AfterTheLastReferencingRootLeavesTheWindow));
        assert_eq!(after_window.label(), "I-3.1+I-3.11+I-3.15");
        let garbled = judge_record(CheckerUnderTest::Today, &constructed_view(Arm::NoMark, RecordClass::QuarantineGarbled, MarkState::WrittenCorrectly, JudgedVersion::TheQuarantineVersion));
        assert_eq!(garbled.label(), "green");
    }

    #[test]
    fn a_complete_mark_exempts_an_intact_quarantine_under_both_variants() {
        for arm in MARKING_ARMS {
            for checker in checkers_of(arm) {
                let verdict = judge_record(*checker, &constructed_view(arm, RecordClass::QuarantineIntact, MarkState::WrittenCorrectly, JudgedVersion::AfterTheLastReferencingRootLeavesTheWindow));
                assert_eq!(verdict.label(), "green", "{} {}", arm.label(), checker.label());
            }
        }
    }

    #[test]
    fn a_garbled_quarantine_with_its_mark_cleared_is_red_for_mark_only_and_green_for_mark_or_today() {
        let view = constructed_view(Arm::SpanBit, RecordClass::QuarantineGarbled, MarkState::WrongOnBothDevices, JudgedVersion::TheQuarantineVersion);
        assert_eq!(judge_record(CheckerUnderTest::MarkOnly, &view).label(), "I-3.11+I-3.15");
        assert_eq!(judge_record(CheckerUnderTest::MarkOrToday, &view).label(), "green");
    }

    #[test]
    fn a_mark_on_a_referenced_record_reddens_mark_agreement() {
        let view = constructed_view(Arm::QuarantineTree, RecordClass::Normal, MarkState::WrongOnBothDevices, JudgedVersion::TheQuarantineVersion);
        assert_eq!(judge_record(CheckerUnderTest::MarkOnly, &view).label(), "mark_agrees_with_references");
        let one_device = constructed_view(Arm::SpanBit, RecordClass::Leak, MarkState::WrongOnOneDevice, JudgedVersion::TheQuarantineVersion);
        assert!(judge_record(CheckerUnderTest::MarkOrToday, &one_device).mark_agreement_red, "只一盘带记号");
    }

    #[test]
    fn the_allocated_statistic_turns_red_exactly_one_window_after_the_quarantine() {
        let event = QuarantineEventRecord { txg: 100, copies: CopyCondition::Intact, marks_during_quarantine: [false, false], revival: None };
        let trajectory = trajectory_of_an_event(CheckerUnderTest::Today, &event, 150, 8);
        assert_eq!(trajectory.first_red_offset_of_the_allocated_statistic, Some(8), "候选窗口 8：事件之后第 8 版最后一个引用它的根离开窗口");
        assert_eq!((trajectory.peak, trajectory.final_value, trajectory.positive_versions), (3, 3, 50 + 1));
        let marked = QuarantineEventRecord { txg: event.txg, copies: event.copies, marks_during_quarantine: [true, true], revival: None };
        assert_eq!(trajectory_of_an_event(CheckerUnderTest::MarkOnly, &marked, 150, 8).peak, 0);
    }

    #[test]
    fn a_revival_ends_the_false_red_once_the_quarantine_versions_leave_the_window() {
        let event = QuarantineEventRecord {
            txg: 100,
            copies: CopyCondition::Intact,
            marks_during_quarantine: [false, false],
            revival: Some(RevivalRecord { txg: 104, marks_after_revival: [false, false] }),
        };
        let trajectory = trajectory_of_an_event(CheckerUnderTest::Today, &event, 150, 8);
        assert_eq!(trajectory.final_value, 0);
        assert_eq!(trajectory.positive_versions, 4 + 8, "隔离 4 版，复活之后 I-3.15 还红到第 103 版离开窗口");
    }

    #[test]
    fn the_schedule_places_ten_events_per_density_step_and_ten_rollbacks() {
        for density in [1, 3, 10] {
            let cell = HistoryCell { density, rollback_rate: 1, file_size: 200, candidate_window: 8, geometry: PoolGeometry::DefaultFourGibibyteDevices, seed: 1 };
            let schedule = schedule_of(&cell);
            assert_eq!(u64_of(schedule.events.len()), density * 10);
            assert_eq!(schedule.rollbacks.len(), 10);
            let intact = schedule.events.values().filter(|event| event.copies == CopyCondition::Intact).count();
            assert_eq!(intact * 2, schedule.events.len(), "副本状态各半");
        }
    }

    /// 取样点覆盖第二段会跑到的全部 320 格（`.claude/rules/mutation-sampling.md` 第四类）：每一格都排满登记第五节的事件数与回退数，
    /// 每次回退排在它那次隔离之后 1 到 ⌊K ÷ 2⌋ 次发布里，几次回退的区间互不相交（作废条款 `scheduled_events_realized` 的排法那一半）。
    #[test]
    fn every_segment_two_cell_schedules_the_registered_events_and_rollbacks_in_disjoint_intervals() {
        for cell in all_history_cells() {
            let schedule = schedule_of(&cell);
            assert_eq!(u64_of(schedule.events.len()), cell.density * QUARANTINE_EVENTS_PER_DENSITY_STEP, "{}", cell.label());
            assert_eq!(u64_of(schedule.rollbacks.len()), cell.rollback_rate * ROLLBACKS_AT_RATE_ONE, "{}", cell.label());
            let mut end_of_the_previous_interval = 0u64;
            for (rollback_after, rollback) in &schedule.rollbacks {
                let start_of_the_interval = match rollback {
                    ScheduledRollback::ToTheVersionBeforeTheEventAt(event_publish) => {
                        assert!(schedule.events.contains_key(event_publish), "{}：回退对着一次排了的隔离", cell.label());
                        assert!(
                            *rollback_after >= event_publish + 1 && *rollback_after <= event_publish + cell.candidate_window / 2,
                            "{}：回退排在隔离之后 1 到 ⌊K ÷ 2⌋ 次发布里",
                            cell.label()
                        );
                        *event_publish
                    }
                    ScheduledRollback::ToTheOldestVersionInTheWindow => {
                        assert_eq!(cell.density, 0, "{}：取窗口里最旧那一版的只在密度 0 上", cell.label());
                        rollback_after - cell.candidate_window
                    }
                };
                assert!(start_of_the_interval > end_of_the_previous_interval, "{}：回退的区间互不相交", cell.label());
                end_of_the_previous_interval = *rollback_after;
            }
        }
    }

    #[test]
    fn journal_records_split_after_sixty_four_named_roles() {
        assert_eq!(journal_records_of_the_publish(4, 59, 0), (3 + 1, 60));
        assert_eq!(journal_records_of_the_publish(4, 63, 0), (3 + 1, 64));
        assert_eq!(journal_records_of_the_publish(4, 64, 0), (3 + 2, 65));
        assert_eq!(journal_records_of_the_publish(0, 0, 0), (1, 0), "一个角色都不点名也写一条");
    }

    #[test]
    fn share_tiers_follow_the_registered_bounds_and_separate_two_points_when_the_bound_moves_between_them() {
        assert_eq!(share_tier(999, 1_000_000, SHARE_TIER_BOUNDS_PARTS_PER_BILLION), 1);
        assert_eq!(share_tier(1000, 1_000_000, SHARE_TIER_BOUNDS_PARTS_PER_BILLION), 2);
        assert_eq!(share_tier(10_000, 1_000_000, SHARE_TIER_BOUNDS_PARTS_PER_BILLION), 3);
        assert_eq!(share_tier(-5, 1_000_000, SHARE_TIER_BOUNDS_PARTS_PER_BILLION), 1);
        let moved = ShareTierBounds { tier_two_from: 300_000, tier_three_from: u64::MAX / 2 };
        assert_ne!(share_tier(200, 1_000_000, moved), share_tier(400, 1_000_000, moved));
        assert_eq!(share_tier(200, 1_000_000, SHARE_TIER_BOUNDS_PARTS_PER_BILLION), share_tier(400, 1_000_000, SHARE_TIER_BOUNDS_PARTS_PER_BILLION));
    }

    #[test]
    fn the_span_bit_changes_one_byte_per_record_and_the_quarantine_tree_two_entries() {
        assert_eq!(logical_bytes_changed_by_the_span_bit(), 1 + 1);
        assert_eq!(2 * QUARANTINE_TREE_ENTRY_BYTES + TREE_TABLE_ENTRY_BYTES, 40 + 200);
    }

    #[test]
    fn the_new_pool_file_creation_opens_the_second_segment_and_every_arm_drops_the_victim_key() {
        assert_eq!(new_pool_file_creation_open_segment(), (50000 + 240, 50000 + 304, 1 + 1));
        assert!(mapping_entry_removed_on_quarantine_in_every_arm());
    }

    #[test]
    fn a_code_two_leaf_splits_in_the_middle_and_a_root_with_one_child_collapses() {
        let capacity = CodeTwoCapacity { leaf_entries: 4, internal_entries: 4 };
        let mut fresh = FreshUnits::starting_at(0);
        let mut root: Option<CodeTwoNode<u64>> = None;
        for key in 1..=5u64 {
            insert_at_the_root(&mut root, key, capacity, &mut fresh);
        }
        assert_eq!(tree_height(&root), 2, "5 条装不进 4 条的叶：切成 3 + 2，长出根");
        let left_leaf_unit = match &*root.as_ref().expect("有根").body {
            CodeTwoBody::Internal { children, .. } => {
                assert_eq!(children.iter().map(|child| child.node.entry_count()).collect::<Vec<usize>>(), vec![3, 2]);
                children[0].node.unit
            }
            CodeTwoBody::Leaf(_) => panic!("根该是内部节点"),
        };
        let mut next_publish = FreshUnits::starting_at(fresh.next_unit);
        for key in [4u64, 5] {
            delete_at_the_root(&mut root, &key, &mut next_publish);
        }
        assert_eq!(tree_height(&root), 1, "右叶删空摘掉、根只剩一个孩子：孩子当根");
        assert_eq!(root.as_ref().expect("有根").unit, left_leaf_unit, "当根的孩子这次没改到，不重写");
    }

    fn small_cell(density: u64, rollback_rate: u64, seed: u64) -> HistoryCell {
        HistoryCell { density, rollback_rate, file_size: 200, candidate_window: 8, geometry: PoolGeometry::DefaultFourGibibyteDevices, seed }
    }

    #[test]
    fn three_arms_write_the_same_on_a_history_without_events() {
        let summary = evaluate_cell(&small_cell(0, 0, 1));
        assert_eq!(summary.arms_agree_without_events, Some(true));
        assert!(summary.schedule_realized);
    }

    /// 回归钉，不算独立钉值（登记第七节末段、第十二节「回归钉」）：值取第二段产物
    /// `research/results/e187-quarantine-mark-cost-segment-two-2026-10-21.out` 里 `no_mark` 在这一格那一行 `name=history` 的 `device_writes=`。
    #[test]
    fn the_no_mark_history_without_events_keeps_the_device_writes_of_the_first_segment_two_product() {
        let cell = small_cell(0, 0, 1);
        let outcome = run_history(&cell, &schedule_of(&cell), Arm::NoMark);
        assert_eq!(outcome.totals.without_root_and_system_configuration().device_writes, 453394);
    }

    #[test]
    fn a_history_with_rollbacks_revives_a_quarantined_unit_at_every_rollback() {
        let summary = evaluate_cell(&small_cell(3, 1, 2));
        assert!(summary.schedule_realized, "30 次隔离、10 次回退，每次回退都让一个隔离单元重新被引用");
    }

    /// 两次隔离挨得近（相隔 21 次与 2 次发布）、K = 64 的格：第一遍抽到的延迟与另一次回退的区间相交，第二遍补上，
    /// 10 次回退都排上、每次都让一个隔离单元重新被引用（全量里出过的红，钉回快档）。
    #[test]
    fn close_quarantine_events_under_the_wide_window_still_get_ten_reviving_rollbacks() {
        for seed in [1, 2] {
            let cell = HistoryCell { density: 1, rollback_rate: 1, file_size: 200, candidate_window: 64, geometry: PoolGeometry::DefaultFourGibibyteDevices, seed };
            let summary = evaluate_cell(&cell);
            assert!(summary.schedule_realized, "{}", cell.label());
        }
    }

    /// 多线程与单线程对拍：产物行逐字相同（`.claude/rules/test-script-discipline.md`「编译与算法都要快」）。
    #[test]
    fn segment_two_lines_do_not_depend_on_the_thread_count() {
        let cells = [small_cell(1, 1, 1), small_cell(1, 0, 2)];
        let single: Vec<Vec<String>> = evaluate_cells(&cells, 1).into_iter().map(|summary| summary.lines).collect();
        let parallel: Vec<Vec<String>> = evaluate_cells(&cells, 2).into_iter().map(|summary| summary.lines).collect();
        assert_eq!(single, parallel);
    }
}
