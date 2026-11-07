//! logged_ops 树的条目（D8（核心索引结构） 已定项 5 的意图树；字段表是
//! 第三节 G1「单树、定宽联合条目」）：key 10 = 种类 2 + 签发 txg 8，value 34 定宽、按种类解，条目 44，一个码 2 节点装 369 条。
//! 第一版装两种意图：整理意图（种类 1）：R（设备 4 + 起始槽 6 + 槽数 2）、`c0` 8、`segs0` 8、补零 6；截断意图（种类 2，判决
//! `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1）：inode 号 8、对象出生代 8、目标 size 8、补零 10。别的种类码保留，读者拒收
//! （照树表条目 flags「未知位一律拒收」的政策，D8（核心索引结构） 已定项 8）。
//!
//! **不存续做点、不存暂停标志**：暂停就是意图留着不推进；续做每批照目标态从 R 起点重扫，
//! 删意图之前对整个 R 全量核目标态。
//!
//! 和 `crate::lineage_tree`、`crate::tombstone_tree` 一样只算、不发一个写：这里只有字段表的读写，签发、推进、删意图归发布入口。
//!
//! 整份在 `verus!` 里，由 Verus 直接验（里程碑四出口 5「core 直验」编解码与记录格式这一簇）：条目的写法 [`LoggedOperationEntry::spec_bytes`]、
//! 读法 [`LoggedOperationEntry::spec_parse`] 与往返引理 [`LoggedOperationEntry::lemma_parse_round_trip`]；整理区域 R 带类型不变量。

use singlefs_format::{
    CLUSTER_SEGMENT_SLOTS, LOGGED_OPERATIONS_ENTRY_BYTES, LOGGED_OPERATIONS_KEY_BYTES,
};
use vstd::prelude::*;

use crate::address::{CheckpointTxg, DeviceIdentity, InodeNumber, SlotNumber};
// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
use crate::bytes::*;
// 同上：spec_index_node_entry_capacity 平时的构建里没有。
use crate::unit::*;

verus! {

broadcast use group_little_endian_round_trips;

// ===== 这一份文件用到的 singlefs-format 常量的规格（写法同 unit.rs 头上那一段）=====
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::CLUSTER_SEGMENT_SLOTS ] -> (cluster_segment_slots: u64) returns 64u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::LOGGED_OPERATIONS_ENTRY_BYTES ] -> (entry_bytes: u64) returns 44u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::LOGGED_OPERATIONS_KEY_BYTES ] -> (key_bytes: u64) returns 10u64;

/// logged_ops 树的 key 宽：种类 2 + 签发 txg 8。码 2 节点头里的自述 key 宽要与它相符。
pub const LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES: usize = 10;

/// logged_ops 条目的种类码：1 = 整理；2 = 截断（[`LOGGED_OPERATION_KIND_TRUNCATION`]）。别的码保留：读者拒收，不当未来版本的条目猜着读。
pub const LOGGED_OPERATION_KIND_COMPACTION: u16 = 1;

/// logged_ops 条目的种类码：2 = 截断。
pub const LOGGED_OPERATION_KIND_TRUNCATION: u16 = 2;

/// 整理意图 value 末尾的补零：34 − (4 + 6 + 2 + 8 + 8)。读者遇到非零一律拒收（同树表条目预留位的政策）。
const COMPACTION_INTENT_PADDING_BYTES: usize = 6;

/// 截断意图 value 末尾的补零：34 − (8 + 8 + 8)。读者遇到非零一律拒收（同整理意图的补零）。
const TRUNCATION_INTENT_PADDING_BYTES: usize = 10;

/// 槽号在盘上占 6 字节（D3（空间分配） 已定项 7）：起始槽要小于它才写得下（2⁴⁸，写成字面量：Verus 不展开常量里的移位）。
const SIX_BYTE_SLOT_NUMBER_LIMIT: u64 = 0x1_0000_0000_0000;

/// logged_ops 条目的种类。封闭集合：加一种意图就加一个成员，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LoggedOperationKind {
    /// 整理意图（D26（后台整理与放置回收） 已定项 2；提示类，D8（核心索引结构） 已定项 5）。
    Compaction,
    /// 截断意图（承诺类，D8（核心索引结构） 已定项 5）。
    Truncation,
}

impl LoggedOperationKind {
    /// 写进 key 前 2 字节的种类码（[`Self::code`] 的规格）。
    pub open spec fn spec_code(self) -> u16 {
        match self {
            LoggedOperationKind::Compaction => LOGGED_OPERATION_KIND_COMPACTION,
            LoggedOperationKind::Truncation => LOGGED_OPERATION_KIND_TRUNCATION,
        }
    }

    /// 盘上读到的种类码认的那一种（[`Self::of_code`] 的规格）。
    pub open spec fn spec_of_code(kind_code: u16) -> Option<Self> {
        if kind_code == LOGGED_OPERATION_KIND_COMPACTION {
            Some(LoggedOperationKind::Compaction)
        } else if kind_code == LOGGED_OPERATION_KIND_TRUNCATION {
            Some(LoggedOperationKind::Truncation)
        } else {
            None
        }
    }

    /// 写进 key 前 2 字节的种类码。
    #[must_use]
    pub const fn code(self) -> (kind_code: u16)
        ensures
            kind_code == self.spec_code(),
    {
        match self {
            LoggedOperationKind::Compaction => LOGGED_OPERATION_KIND_COMPACTION,
            LoggedOperationKind::Truncation => LOGGED_OPERATION_KIND_TRUNCATION,
        }
    }

    /// 盘上读到的种类码认不认得：认不得的（保留码）交回 `None`。
    #[must_use]
    pub const fn of_code(kind_code: u16) -> (kind: Option<Self>)
        ensures
            kind == Self::spec_of_code(kind_code),
    {
        // 常量模式的 match 写成 if：Verus 不认按常量匹配，两种码之外的保留码照旧交回 None。
        if kind_code == LOGGED_OPERATION_KIND_COMPACTION {
            Some(LoggedOperationKind::Compaction)
        } else if kind_code == LOGGED_OPERATION_KIND_TRUNCATION {
            Some(LoggedOperationKind::Truncation)
        } else {
            None
        }
    }
}

/// 整理意图的区域 R：一块盘上一个 64 槽对齐的聚簇段（起始槽 64 对齐、槽数 = 64；第一版两盘同槽时 R 罩住每块盘上的同号槽）。
/// 只从 [`CompactionRegion::of_one_cluster_segment`] 进来，所以槽数不存：恒是 [`CLUSTER_SEGMENT_SLOTS`]。
///
/// 类型不变量（[`CompactionRegion::is_one_aligned_cluster_segment`]）：起始槽是 64 的倍数、装得进 6 字节。
/// 字段私有，只有 [`CompactionRegion::of_one_cluster_segment`] 造得出来，所以每一个值都满足它。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CompactionRegion {
    device: DeviceIdentity,
    start_slot: SlotNumber,
}

/// 造不出一个整理区域的两种起始槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionRegionRefusal {
    /// 起始槽不是 64 的倍数：R 不对齐。
    StartOffAClusterSegmentBoundary { start_slot: SlotNumber },
    /// 起始槽装不进盘上 6 字节的槽号。
    StartBeyondTheSixByteSlotNumber { start_slot: SlotNumber },
}

impl CompactionRegion {
    /// 类型不变量：起始槽是 64 的倍数、装得进 6 字节。
    #[verifier::type_invariant]
    spec fn is_one_aligned_cluster_segment(self) -> bool {
        &&& self.start_slot.0 % CLUSTER_SEGMENT_SLOTS == 0
        &&& self.start_slot.0 < SIX_BYTE_SLOT_NUMBER_LIMIT
    }

    /// R 所在的盘。
    pub closed spec fn spec_device(self) -> DeviceIdentity {
        self.device
    }

    /// R 的起始槽。
    pub closed spec fn spec_start_slot(self) -> SlotNumber {
        self.start_slot
    }

    /// 起始槽是 64 的倍数、装得进 6 字节（类型不变量的公开说法：每一个值都满足它，各个读它的方法在 `ensures` 里交出来）。
    pub open spec fn spec_start_is_aligned_and_fits(self) -> bool {
        &&& self.spec_start_slot().0 % 64 == 0
        &&& self.spec_start_slot().0 < 0x1_0000_0000_0000
    }

    /// 这个槽号在不在 R 里（[`Self::contains_slot`] 的规格）。
    pub open spec fn spec_contains_slot(self, slot: SlotNumber) -> bool {
        self.spec_start_slot().0 <= slot.0 && slot.0 < self.spec_start_slot().0 + 64
    }

    /// [`Self::of_one_cluster_segment`] 的判法：先判对齐，再判装不装得进 6 字节。
    pub closed spec fn spec_of_one_cluster_segment(
        device: DeviceIdentity,
        start_slot: SlotNumber,
    ) -> Result<Self, CompactionRegionRefusal> {
        if start_slot.0 % 64 != 0 {
            Err(CompactionRegionRefusal::StartOffAClusterSegmentBoundary { start_slot })
        } else if start_slot.0 >= 0x1_0000_0000_0000 {
            Err(CompactionRegionRefusal::StartBeyondTheSixByteSlotNumber { start_slot })
        } else {
            Ok(CompactionRegion { device, start_slot })
        }
    }

    /// 判过的区域交回的盘与起始槽就是给的那两样；判法的结论（对齐、装得进）就是不变量。
    pub proof fn lemma_of_one_cluster_segment(device: DeviceIdentity, start_slot: SlotNumber)
        ensures
            Self::spec_of_one_cluster_segment(device, start_slot) is Ok <==> (start_slot.0 % 64 == 0
                && start_slot.0 < 0x1_0000_0000_0000),
            Self::spec_of_one_cluster_segment(device, start_slot) matches Ok(region) ==> region.spec_device()
                == device && region.spec_start_slot() == start_slot,
    {
    }

    /// 一个区域由它的盘与起始槽定：两样相同的区域相同；它自己的盘与起始槽再判一次就是它自己。
    pub proof fn lemma_region_is_its_device_and_start(self)
        requires
            self.spec_start_is_aligned_and_fits(),
        ensures
            Self::spec_of_one_cluster_segment(self.spec_device(), self.spec_start_slot()) == Ok::<
                Self,
                CompactionRegionRefusal,
            >(self),
    {
    }

    /// `device` 上从 `start_slot` 起的那一个聚簇段。
    ///
    /// # Errors
    /// 起始槽不是 64 的倍数 ⇒ `StartOffAClusterSegmentBoundary`；装不进 6 字节 ⇒ `StartBeyondTheSixByteSlotNumber`。
    pub fn of_one_cluster_segment(
        device: DeviceIdentity,
        start_slot: SlotNumber,
    ) -> (region: Result<Self, CompactionRegionRefusal>)
        ensures
            region == Self::spec_of_one_cluster_segment(device, start_slot),
            region matches Ok(made) ==> made.spec_start_is_aligned_and_fits(),
    {
        if !start_slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {
            return Err(CompactionRegionRefusal::StartOffAClusterSegmentBoundary { start_slot });
        }
        if start_slot.0 >= SIX_BYTE_SLOT_NUMBER_LIMIT {
            return Err(CompactionRegionRefusal::StartBeyondTheSixByteSlotNumber { start_slot });
        }
        Ok(Self { device, start_slot })
    }

    #[must_use]
    pub const fn device(self) -> (device: DeviceIdentity)
        ensures
            device == self.spec_device(),
            self.spec_start_is_aligned_and_fits(),
    {
        proof {
            use_type_invariant(self);
        }
        self.device
    }

    #[must_use]
    pub const fn start_slot(self) -> (start_slot: SlotNumber)
        ensures
            start_slot == self.spec_start_slot(),
            self.spec_start_is_aligned_and_fits(),
    {
        proof {
            use_type_invariant(self);
        }
        self.start_slot
    }

    /// 槽数：恒是一个聚簇段。
    #[must_use]
    pub const fn slot_count(self) -> (slot_count: u64)
        ensures
            slot_count == 64,
    {
        CLUSTER_SEGMENT_SLOTS
    }

    /// R 之后的第一个槽（开区间的右端）。
    #[must_use]
    pub const fn end_slot(self) -> (end_slot: SlotNumber)
        ensures
            end_slot.0 == self.spec_start_slot().0 + 64,
    {
        proof {
            use_type_invariant(self);
        }
        SlotNumber(self.start_slot.0 + CLUSTER_SEGMENT_SLOTS)
    }

    /// 这个槽号在不在 R 里（只看槽号：两盘同槽时 R 罩住每块盘上的同号槽）。
    #[must_use]
    pub const fn contains_slot(self, slot: SlotNumber) -> (inside: bool)
        ensures
            inside == self.spec_contains_slot(slot),
    {
        proof {
            use_type_invariant(self);
        }
        slot.0 >= self.start_slot.0 && slot.0 < self.start_slot.0 + CLUSTER_SEGMENT_SLOTS
    }
}

/// 一条整理意图（D26（后台整理与放置回收） 已定项 1 / 已定项 2；字段表判决 G1）。三个字段都是值，不是增量
/// （D8（核心索引结构） 已定项 5 硬要求 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionIntent {
    pub region: CompactionRegion,
    /// `c0`：签发那一刻的分配代；目标态是「R 内不存在分配代 ≤ `c0` 且未释放的落点」（D26 已定项 2）。
    pub allocation_generation_at_issue: CheckpointTxg,
    /// `segs0`：签发那次发布写下的记账第 11 项（全空聚簇段数）里 R 所在盘的值；只作观测量，停机不靠它。
    pub empty_cluster_segments_at_issue: u64,
}

/// 一条截断意图：把 `inode`（对象出生代 `object_birth`）
/// 截到 `target_size_in_bytes` 字节。目标是值、不是增量（D8（核心索引结构） 已定项 5 硬要求 1）：意图那次发布已把记录的 size 写成它，
/// 各批只从尾部放掉 size 之外还挂着的单元，最后一批与删这一条同一次发布。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TruncationIntent {
    pub inode: InodeNumber,
    pub object_birth: CheckpointTxg,
    pub target_size_in_bytes: u64,
}

/// logged_ops 条目的 value，按种类分。种类码从成员推出来，key 与 value 对不上的条目造不出来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoggedOperation {
    Compaction(CompactionIntent),
    Truncation(TruncationIntent),
}

/// logged_ops 条目的 key：种类 + 签发 txg。全序按逐字段无符号整数比（D8（核心索引结构） 已定项 11：小端存储不构成 memcmp 序），
/// 先按种类、再按签发先后。一次发布至多签发一条，所以 key 不带序号。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LoggedOperationKey {
    pub kind: LoggedOperationKind,
    pub issued_txg: CheckpointTxg,
}

/// logged_ops 树里的一条：签发那次发布的 txg 与它声明的意图。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoggedOperationEntry {
    pub issued_txg: CheckpointTxg,
    pub operation: LoggedOperation,
}

/// 读者拒收一条 logged_ops 条目的原因。条目是盘上读来的字节、不是我们的不变量：调用方把它当损坏处置。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoggedOperationEntryRefusal {
    /// 条目宽（码 2 节点头里的盘上字段）不是字段表的 44。
    WidthIsNotTheFieldTable { entry_bytes: usize },
    /// 种类码是保留码。
    UnknownKind { kind_code: u16 },
    /// 整理意图的 R 起始槽不对齐、或装不进 6 字节。
    CompactionRegionRefused(CompactionRegionRefusal),
    /// 整理意图的 R 槽数不是一个聚簇段（64）。
    CompactionRegionSlotCountIsNotOneClusterSegment { slot_count: u16 },
    /// 整理意图 value 末尾那 6 字节补零有非零字节。
    CompactionPaddingNotZero,
    /// 截断意图 value 末尾那 10 字节补零有非零字节。
    TruncationPaddingNotZero,
}

/// 读者拒收一个 logged_ops 根的原因：根里的字节是盘上读来的，调用方把它当损坏处置（可写挂载拒）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoggedOperationsRootRefusal {
    EntryRefused(LoggedOperationEntryRefusal),
    EntriesNotInStrictlyAscendingKeyOrder,
    MoreThanOneCompactionIntent { compaction_intents: usize },
    MoreThanOneTruncationIntent { truncation_intents: usize },
}


impl LoggedOperation {
    /// 这一条意图的种类（[`Self::kind`] 的规格）。
    pub open spec fn spec_kind(self) -> LoggedOperationKind {
        match self {
            LoggedOperation::Compaction(_intent) => LoggedOperationKind::Compaction,
            LoggedOperation::Truncation(_intent) => LoggedOperationKind::Truncation,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> (kind: LoggedOperationKind)
        ensures
            kind == self.spec_kind(),
    {
        match self {
            LoggedOperation::Compaction(_intent) => LoggedOperationKind::Compaction,
            LoggedOperation::Truncation(_intent) => LoggedOperationKind::Truncation,
        }
    }
}

impl LoggedOperationKey {
    /// 全序上严格在前：先比种类码、再比签发 txg（[`Self::sort_key`] 那一对按元组比的序）。
    pub open spec fn spec_precedes(self, other: Self) -> bool {
        ||| self.kind.spec_code() < other.kind.spec_code()
        ||| (self.kind.spec_code() == other.kind.spec_code() && self.issued_txg.0 < other.issued_txg.0)
    }

    #[must_use]
    pub fn sort_key(&self) -> (sort_key: (u16, u64))
        ensures
            sort_key == (self.kind.spec_code(), self.issued_txg.0),
    {
        (self.kind.code(), self.issued_txg.0)
    }

    /// 这一个 key 在全序上严格在 `other` 之前：两个 `sort_key` 按元组比（元组比较没有 Verus 规格，逐字段写出来，同一个序）。
    #[must_use]
    pub fn precedes(&self, other: &Self) -> (precedes: bool)
        ensures
            precedes == self.spec_precedes(*other),
    {
        let (this_kind_code, this_issued_txg) = self.sort_key();
        let (other_kind_code, other_issued_txg) = other.sort_key();
        this_kind_code < other_kind_code || (this_kind_code == other_kind_code
            && this_issued_txg < other_issued_txg)
    }
}

/// 一串条目里 `position` 那一条的 key 严格在下一条之前。
pub open spec fn spec_pair_ascends(entries: Seq<LoggedOperationEntry>, position: int) -> bool {
    entries[position].spec_key().spec_precedes(entries[position + 1].spec_key())
}

/// 一串条目按 key 严格升序（相邻两条前一条严格在前）。
pub open spec fn spec_keys_strictly_ascend(entries: Seq<LoggedOperationEntry>) -> bool {
    forall|position: int| 0 <= position && position + 1 < entries.len() ==> #[trigger] spec_pair_ascends(entries, position)
}

/// 一串条目里某一种意图有几条。
pub open spec fn spec_number_of_kind(entries: Seq<LoggedOperationEntry>, kind: LoggedOperationKind) -> nat
    decreases entries.len(),
{
    if entries.len() == 0 {
        0
    } else {
        spec_number_of_kind(entries.drop_last(), kind) + if entries.last().operation.spec_kind() == kind {
            1nat
        } else {
            0nat
        }
    }
}

/// 一串条目按 key 严格升序（[`spec_keys_strictly_ascend`]）：两处读写根条目的断言与拒收共用这一判。
fn keys_strictly_ascend(entries: &[LoggedOperationEntry]) -> (ascending: bool)
    ensures
        ascending == spec_keys_strictly_ascend(entries@),
{
    let mut later_position: usize = 1;
    while later_position < entries.len()
        invariant
            1 <= later_position,
            forall|position: int|
                0 <= position && position + 1 < later_position && position + 1 < entries@.len()
                    ==> #[trigger] spec_pair_ascends(entries@, position),
        decreases entries@.len() - later_position,
    {
        if !entries[later_position - 1].key().precedes(&entries[later_position].key()) {
            assert(!spec_pair_ascends(entries@, later_position - 1));
            return false;
        }
        later_position += 1;
    }
    true
}

/// 一串条目里 `kind` 那一种意图有几条（[`spec_number_of_kind`]）。
fn number_of_entries_of_kind(entries: &[LoggedOperationEntry], kind: LoggedOperationKind) -> (count: usize)
    ensures
        count == spec_number_of_kind(entries@, kind),
{
    let mut count: usize = 0;
    let mut checked_count: usize = 0;
    while checked_count < entries.len()
        invariant
            checked_count <= entries@.len(),
            count == spec_number_of_kind(entries@.take(checked_count as int), kind),
            count <= checked_count,
        decreases entries@.len() - checked_count,
    {
        proof {
            lemma_number_of_kind_of_a_longer_prefix(entries@, checked_count as int, kind);
        }
        // 比种类码：两种码不同，码相同就是同一种。
        if entries[checked_count].operation.kind().code() == kind.code() {
            count += 1;
        }
        checked_count += 1;
    }
    assert(entries@.take(checked_count as int) =~= entries@);
    count
}

fn entry_bytes() -> (entry_bytes: usize)
    ensures
        entry_bytes == LOGGED_OPERATIONS_ENTRY_BYTES,
{
    usize::try_from(LOGGED_OPERATIONS_ENTRY_BYTES).expect("44")
}

impl LoggedOperationEntry {
    /// 这一条的 key（[`Self::key`] 的规格）。
    pub open spec fn spec_key(self) -> LoggedOperationKey {
        LoggedOperationKey { kind: self.operation.spec_kind(), issued_txg: self.issued_txg }
    }

    /// 写得出、读得回的一条：整理意图的 R 满足类型不变量（每一个造得出来的 R 都满足，见 [`CompactionRegion::start_slot`]）。
    pub open spec fn is_well_formed(self) -> bool {
        self.operation matches LoggedOperation::Compaction(intent) ==> intent.region.spec_start_is_aligned_and_fits()
    }

    /// 条目 44 的 value 那 34 字节。
    pub open spec fn spec_value_bytes(self) -> Seq<u8> {
        match self.operation {
            LoggedOperation::Compaction(intent) => little_endian_bytes(intent.region.spec_device().0 as nat, 4)
                + little_endian_bytes(intent.region.spec_start_slot().0 as nat, 6) + little_endian_bytes(64, 2)
                + little_endian_bytes(intent.allocation_generation_at_issue.0 as nat, 8) + little_endian_bytes(
                intent.empty_cluster_segments_at_issue as nat,
                8,
            ) + zero_bytes(6),
            LoggedOperation::Truncation(intent) => little_endian_bytes(intent.inode.0 as nat, 8) + little_endian_bytes(
                intent.object_birth.0 as nat,
                8,
            ) + little_endian_bytes(intent.target_size_in_bytes as nat, 8) + zero_bytes(10),
        }
    }

    /// 盘上 44 字节（[`Self::to_bytes`] 交出的）：种类 2 + 签发 txg 8 + value 34。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        little_endian_bytes(self.operation.spec_kind().spec_code() as nat, 2) + little_endian_bytes(
            self.issued_txg.0 as nat,
            8,
        ) + self.spec_value_bytes()
    }

    /// [`Self::parse`] 的判法。
    pub open spec fn spec_parse(bytes: Seq<u8>) -> Result<Self, LoggedOperationEntryRefusal> {
        let kind_code = little_endian_value(bytes.subrange(0, 2)) as u16;
        let issued_txg = CheckpointTxg(little_endian_value(bytes.subrange(2, 10)) as u64);
        if bytes.len() != LOGGED_OPERATIONS_ENTRY_BYTES {
            Err(LoggedOperationEntryRefusal::WidthIsNotTheFieldTable { entry_bytes: bytes.len() as usize })
        } else {
            match LoggedOperationKind::spec_of_code(kind_code) {
                None => Err(LoggedOperationEntryRefusal::UnknownKind { kind_code }),
                Some(LoggedOperationKind::Compaction) => match spec_parse_compaction_intent_at(bytes, 10) {
                    Ok(intent) => Ok(LoggedOperationEntry { issued_txg, operation: LoggedOperation::Compaction(intent) }),
                    Err(refusal) => Err(refusal),
                },
                Some(LoggedOperationKind::Truncation) => match spec_parse_truncation_intent_at(bytes, 10) {
                    Ok(intent) => Ok(LoggedOperationEntry { issued_txg, operation: LoggedOperation::Truncation(intent) }),
                    Err(refusal) => Err(refusal),
                },
            }
        }
    }

    /// 往返：写出的 44 字节读回同一条。
    pub proof fn lemma_parse_round_trip(self)
        requires
            self.is_well_formed(),
        ensures
            Self::spec_parse(self.spec_bytes()) == Ok::<Self, LoggedOperationEntryRefusal>(self),
    {
        lemma_little_endian_capacities();
        let bytes = self.spec_bytes();
        let kind_bytes = little_endian_bytes(self.operation.spec_kind().spec_code() as nat, 2);
        let issued_bytes = little_endian_bytes(self.issued_txg.0 as nat, 8);
        assert(bytes.subrange(0, bytes.len() as int) =~= bytes);
        lemma_split_a_concatenated_range(bytes, 0, kind_bytes + issued_bytes, self.spec_value_bytes());
        lemma_split_a_concatenated_range(bytes, 0, kind_bytes, issued_bytes);
        match self.operation {
            LoggedOperation::Compaction(intent) => {
                let device = little_endian_bytes(intent.region.spec_device().0 as nat, 4);
                let start = little_endian_bytes(intent.region.spec_start_slot().0 as nat, 6);
                let count = little_endian_bytes(64, 2);
                let generation = little_endian_bytes(intent.allocation_generation_at_issue.0 as nat, 8);
                let segments = little_endian_bytes(intent.empty_cluster_segments_at_issue as nat, 8);
                lemma_split_a_concatenated_range(bytes, 10, device + start + count + generation + segments, zero_bytes(6));
                lemma_split_a_concatenated_range(bytes, 10, device + start + count + generation, segments);
                lemma_split_a_concatenated_range(bytes, 10, device + start + count, generation);
                lemma_split_a_concatenated_range(bytes, 10, device + start, count);
                lemma_split_a_concatenated_range(bytes, 10, device, start);
                intent.region.lemma_region_is_its_device_and_start();
            },
            LoggedOperation::Truncation(intent) => {
                let inode = little_endian_bytes(intent.inode.0 as nat, 8);
                let birth = little_endian_bytes(intent.object_birth.0 as nat, 8);
                let size = little_endian_bytes(intent.target_size_in_bytes as nat, 8);
                lemma_split_a_concatenated_range(bytes, 10, inode + birth + size, zero_bytes(10));
                lemma_split_a_concatenated_range(bytes, 10, inode + birth, size);
                lemma_split_a_concatenated_range(bytes, 10, inode, birth);
            },
        }
    }

    #[must_use]
    pub const fn key(&self) -> (key: LoggedOperationKey)
        ensures
            key == self.spec_key(),
    {
        LoggedOperationKey {
            kind: self.operation.kind(),
            issued_txg: self.issued_txg,
        }
    }

    /// 条目 44：种类 0 | 签发 txg 2 | 整理：R 的设备 10 | R 的起始槽 14 | R 的槽数 20 | `c0` 22 | `segs0` 30 | 补零 38；
    /// 截断：inode 号 10 | 对象出生代 18 | 目标 size 26 | 补零 34，小端。
    #[must_use]
    pub fn to_bytes(&self) -> (bytes: Vec<u8>)
        ensures
            bytes@ == self.spec_bytes(),
            self.is_well_formed(),
    {
        let mut writer = ByteWriter::new(entry_bytes());
        writer.put_u16(self.operation.kind().code());
        writer.put_u64(self.issued_txg.0);
        writer.assert_position(LOGGED_OPERATIONS_KEY_BYTES, "logged_ops 条目的 key");
        let ghost after_the_key = writer.written();
        match &self.operation {
            LoggedOperation::Compaction(intent) => {
                writer.put_u32(intent.region.device().0);
                writer.assert_position(14, "整理意图 R 的起始槽");
                writer.put_six_byte_unsigned(intent.region.start_slot().0);
                writer.assert_position(20, "整理意图 R 的槽数");
                writer.put_u16(
                    u16::try_from(intent.region.slot_count())
                        .expect("一个聚簇段 64 槽装得进 2 字节"),
                );
                writer.assert_position(22, "整理意图的 c0");
                writer.put_u64(intent.allocation_generation_at_issue.0);
                writer.assert_position(30, "整理意图的 segs0");
                writer.put_u64(intent.empty_cluster_segments_at_issue);
                writer.assert_position(38, "整理意图的补零");
                writer.skip(COMPACTION_INTENT_PADDING_BYTES);
            }
            LoggedOperation::Truncation(intent) => {
                writer.put_u64(intent.inode.0);
                writer.assert_position(18, "截断意图的对象出生代");
                writer.put_u64(intent.object_birth.0);
                writer.assert_position(26, "截断意图的目标 size");
                writer.put_u64(intent.target_size_in_bytes);
                writer.assert_position(34, "截断意图的补零");
                writer.skip(TRUNCATION_INTENT_PADDING_BYTES);
            }
        }
        assert(writer.written() =~= after_the_key + self.spec_value_bytes());
        writer.assert_position(LOGGED_OPERATIONS_ENTRY_BYTES, "logged_ops 条目");
        let bytes = writer.into_bytes();
        assert(bytes@ =~= self.spec_bytes());
        bytes
    }

    /// logged_ops 条目 44 的读者。
    ///
    /// # Errors
    /// 宽不是 44 ⇒ `WidthIsNotTheFieldTable`；种类码保留 ⇒ `UnknownKind`；整理意图的 R 起始槽不是 64 的倍数 ⇒
    /// `CompactionRegionRefused`；R 槽数不是 64 ⇒ `CompactionRegionSlotCountIsNotOneClusterSegment`；补零非零 ⇒ `CompactionPaddingNotZero`；
    /// 截断意图的补零非零 ⇒ `TruncationPaddingNotZero`。
    pub fn parse(bytes: &[u8]) -> (entry: Result<Self, LoggedOperationEntryRefusal>)
        ensures
            entry == Self::spec_parse(bytes@),
            entry matches Ok(parsed) ==> parsed.is_well_formed(),
    {
        if bytes.len() != entry_bytes() {
            return Err(LoggedOperationEntryRefusal::WidthIsNotTheFieldTable {
                entry_bytes: bytes.len(),
            });
        }
        let mut reader = ByteReader::at(bytes, 0);
        let kind_code = reader.get_u16();
        let issued_txg = CheckpointTxg(reader.get_u64());
        // Option::ok_or 接 ? 写成 match：认不得的种类码照旧交回 UnknownKind。
        let kind = match LoggedOperationKind::of_code(kind_code) {
            Some(kind) => kind,
            None => return Err(LoggedOperationEntryRefusal::UnknownKind { kind_code }),
        };
        let operation = match kind {
            LoggedOperationKind::Compaction => {
                LoggedOperation::Compaction(parse_compaction_intent(&mut reader)?)
            }
            LoggedOperationKind::Truncation => {
                LoggedOperation::Truncation(parse_truncation_intent(&mut reader)?)
            }
        };
        Ok(Self {
            issued_txg,
            operation,
        })
    }
}

/// 整理意图那 34 字节 value 从 `source` 的 `position` 起读出来的样子（[`parse_compaction_intent`] 的判法）：先判槽数、再判补零、最后判 R。
pub open spec fn spec_parse_compaction_intent_at(
    source: Seq<u8>,
    position: int,
) -> Result<CompactionIntent, LoggedOperationEntryRefusal> {
    let device = DeviceIdentity(little_endian_value(source.subrange(position, position + 4)) as u32);
    let start_slot = SlotNumber(little_endian_value(source.subrange(position + 4, position + 10)) as u64);
    let slot_count = little_endian_value(source.subrange(position + 10, position + 12)) as u16;
    let allocation_generation_at_issue = CheckpointTxg(
        little_endian_value(source.subrange(position + 12, position + 20)) as u64,
    );
    let empty_cluster_segments_at_issue = little_endian_value(source.subrange(position + 20, position + 28)) as u64;
    if slot_count != 64 {
        Err(LoggedOperationEntryRefusal::CompactionRegionSlotCountIsNotOneClusterSegment { slot_count })
    } else if source.subrange(position + 28, position + 34) != zero_bytes(6) {
        Err(LoggedOperationEntryRefusal::CompactionPaddingNotZero)
    } else {
        match CompactionRegion::spec_of_one_cluster_segment(device, start_slot) {
            Ok(region) => Ok(CompactionIntent { region, allocation_generation_at_issue, empty_cluster_segments_at_issue }),
            Err(refusal) => Err(LoggedOperationEntryRefusal::CompactionRegionRefused(refusal)),
        }
    }
}

/// 截断意图那 34 字节 value 从 `source` 的 `position` 起读出来的样子（[`parse_truncation_intent`] 的判法）。
pub open spec fn spec_parse_truncation_intent_at(
    source: Seq<u8>,
    position: int,
) -> Result<TruncationIntent, LoggedOperationEntryRefusal> {
    if source.subrange(position + 24, position + 34) != zero_bytes(10) {
        Err(LoggedOperationEntryRefusal::TruncationPaddingNotZero)
    } else {
        Ok(TruncationIntent {
            inode: InodeNumber(little_endian_value(source.subrange(position, position + 8)) as u64),
            object_birth: CheckpointTxg(little_endian_value(source.subrange(position + 8, position + 16)) as u64),
            target_size_in_bytes: little_endian_value(source.subrange(position + 16, position + 24)) as u64,
        })
    }
}

/// 整理意图那 34 字节 value（读者已在 key 之后）。
fn parse_compaction_intent(
    reader: &mut ByteReader<'_>,
) -> (intent: Result<CompactionIntent, LoggedOperationEntryRefusal>)
    requires
        old(reader).has_left(34),
    ensures
        intent == spec_parse_compaction_intent_at(old(reader).source(), old(reader).spec_position() as int),
        intent matches Ok(parsed) ==> parsed.region.spec_start_is_aligned_and_fits(),
{
    let device = DeviceIdentity(reader.get_u32());
    let start_slot = SlotNumber(reader.get_six_byte_unsigned());
    let slot_count = reader.get_u16();
    let allocation_generation_at_issue = CheckpointTxg(reader.get_u64());
    let empty_cluster_segments_at_issue = reader.get_u64();
    let padding = reader.take(COMPACTION_INTENT_PADDING_BYTES);
    if u64::from(slot_count) != CLUSTER_SEGMENT_SLOTS {
        return Err(
            LoggedOperationEntryRefusal::CompactionRegionSlotCountIsNotOneClusterSegment {
                slot_count,
            },
        );
    }
    if !every_byte_is_zero(padding) {
        return Err(LoggedOperationEntryRefusal::CompactionPaddingNotZero);
    }
    // Result::map_err 接 ? 写成 match：拒收的原因照旧包成 CompactionRegionRefused。
    let region = match CompactionRegion::of_one_cluster_segment(device, start_slot) {
        Ok(region) => region,
        Err(refusal) => return Err(LoggedOperationEntryRefusal::CompactionRegionRefused(refusal)),
    };
    Ok(CompactionIntent {
        region,
        allocation_generation_at_issue,
        empty_cluster_segments_at_issue,
    })
}

/// 截断意图那 34 字节 value（读者已在 key 之后）。
fn parse_truncation_intent(
    reader: &mut ByteReader<'_>,
) -> (intent: Result<TruncationIntent, LoggedOperationEntryRefusal>)
    requires
        old(reader).has_left(34),
    ensures
        intent == spec_parse_truncation_intent_at(old(reader).source(), old(reader).spec_position() as int),
{
    let inode = InodeNumber(reader.get_u64());
    let object_birth = CheckpointTxg(reader.get_u64());
    let target_size_in_bytes = reader.get_u64();
    let padding = reader.take(TRUNCATION_INTENT_PADDING_BYTES);
    if !every_byte_is_zero(padding) {
        return Err(LoggedOperationEntryRefusal::TruncationPaddingNotZero);
    }
    Ok(TruncationIntent {
        inode,
        object_birth,
        target_size_in_bytes,
    })
}

/// 一个 logged_ops 码 2 根（层级 0）装得下几条：(16384 − 含预留位的头 135) ÷ 44 的整数部分 = 369。
#[must_use]
pub fn logged_operations_one_root_node_holds() -> (holds: usize)
    ensures
        holds == spec_index_node_entry_capacity(10, 44),
        holds == 369,
{
    index_node_entry_capacity(LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES, entry_bytes())
}

/// logged_ops 根要装的条目：每条 44 字节（[`LoggedOperationEntry::to_bytes`]），按 key 的全序升序。
/// logged_ops 第一版只有一个码 2 根、根之下不长层：全池活整理意图至多一条、活截断意图至多一条，
/// 一个根装 369 条。
///
/// # Panics
/// `entries` 不是按 key 严格升序、或多于一个根装得下的：条目由发布路径按「签发加一条、删掉一条」维护，至多一条活整理意图，
/// 乱序或装不下说明调用方拼错了（`requires` 排除的两种）。
#[must_use]
pub fn logged_operations_root_entries(entries: &[LoggedOperationEntry]) -> (root_entries: Vec<Vec<u8>>)
    requires
        spec_keys_strictly_ascend(entries@),
        entries@.len() <= 369,
    ensures
        root_entries@.len() == entries@.len(),
        forall|position: int|
            0 <= position < entries@.len() ==> (#[trigger] root_entries@[position])@ == entries@[position].spec_bytes(),
{
    assert!(
        keys_strictly_ascend(entries),
        "logged_ops 的条目按 key 升序、不重复：发布路径按集合维护它"
    );
    assert!(
        entries.len() <= logged_operations_one_root_node_holds(),
        "logged_ops 至多一条活整理意图与一条活截断意图，一个根装得下"
    );
    let mut root_entries: Vec<Vec<u8>> = Vec::new();
    let mut written_count: usize = 0;
    while written_count < entries.len()
        invariant
            written_count <= entries@.len(),
            root_entries@.len() == written_count,
            forall|position: int|
                0 <= position < written_count ==> (#[trigger] root_entries@[position])@ == entries@[position].spec_bytes(),
        decreases entries@.len() - written_count,
    {
        root_entries.push(entries[written_count].to_bytes());
        written_count += 1;
    }
    root_entries
}

/// 从一个 logged_ops 根的条目读回意图（条目是盘上读来的字节，逐条照字段表解）。
///
/// # Errors
/// 有一条过不了 [`LoggedOperationEntry::parse`]（宽不是 44、种类码保留、R 不对齐或槽数不是 64、补零非零）、
/// 几条不按 key 严格升序、整理意图多于一条（全池至多一条）、截断意图多于一条（全池至多一条）
/// ⇒ [`LoggedOperationsRootRefusal`]。
pub fn logged_operation_entries_of_the_root_entries(
    entries: &[Vec<u8>],
) -> (parsed: Result<Vec<LoggedOperationEntry>, LoggedOperationsRootRefusal>)
    ensures
        match parsed {
            Ok(parsed_entries) => {
                &&& parsed_entries@.len() == entries@.len()
                &&& forall|position: int|
                    0 <= position < entries@.len() ==> LoggedOperationEntry::spec_parse(
                        (#[trigger] entries@[position])@,
                    ) == Ok::<LoggedOperationEntry, LoggedOperationEntryRefusal>(parsed_entries@[position])
                &&& spec_keys_strictly_ascend(parsed_entries@)
                &&& spec_number_of_kind(parsed_entries@, LoggedOperationKind::Compaction) <= 1
                &&& spec_number_of_kind(parsed_entries@, LoggedOperationKind::Truncation) <= 1
            },
            Err(LoggedOperationsRootRefusal::EntryRefused(refusal)) => exists|position: int|
                0 <= position < entries@.len() && LoggedOperationEntry::spec_parse((#[trigger] entries@[position])@)
                    == Err::<LoggedOperationEntry, LoggedOperationEntryRefusal>(refusal),
            Err(LoggedOperationsRootRefusal::EntriesNotInStrictlyAscendingKeyOrder) => forall|position: int|
                0 <= position < entries@.len() ==> LoggedOperationEntry::spec_parse(
                    (#[trigger] entries@[position])@,
                ) is Ok,
            Err(LoggedOperationsRootRefusal::MoreThanOneCompactionIntent { compaction_intents }) => compaction_intents
                > 1,
            Err(LoggedOperationsRootRefusal::MoreThanOneTruncationIntent { truncation_intents }) => truncation_intents
                > 1,
        },
{
    let mut parsed: Vec<LoggedOperationEntry> = Vec::new();
    let mut parsed_count: usize = 0;
    // 逐条解，第一条解不开的就交回它的原因（与 collect::<Result<_, _>> 停在第一条错上同一个次序）。
    while parsed_count < entries.len()
        invariant
            parsed_count <= entries@.len(),
            parsed@.len() == parsed_count,
            forall|position: int|
                0 <= position < parsed_count ==> LoggedOperationEntry::spec_parse((#[trigger] entries@[position])@)
                    == Ok::<LoggedOperationEntry, LoggedOperationEntryRefusal>(parsed@[position]),
        decreases entries@.len() - parsed_count,
    {
        match LoggedOperationEntry::parse(entries[parsed_count].as_slice()) {
            Ok(entry) => parsed.push(entry),
            Err(refusal) => return Err(LoggedOperationsRootRefusal::EntryRefused(refusal)),
        }
        parsed_count += 1;
    }
    if !keys_strictly_ascend(&parsed) {
        return Err(LoggedOperationsRootRefusal::EntriesNotInStrictlyAscendingKeyOrder);
    }
    let compaction_intents = number_of_entries_of_kind(&parsed, LoggedOperationKind::Compaction);
    if compaction_intents > 1 {
        return Err(LoggedOperationsRootRefusal::MoreThanOneCompactionIntent {
            compaction_intents,
        });
    }
    let truncation_intents = number_of_entries_of_kind(&parsed, LoggedOperationKind::Truncation);
    if truncation_intents > 1 {
        return Err(LoggedOperationsRootRefusal::MoreThanOneTruncationIntent {
            truncation_intents,
        });
    }
    Ok(parsed)
}

/// 每多看一条，计数的增量就是那一条是不是这一种（[`spec_number_of_kind`] 从末尾往前数，这里把它接到前缀上）。
proof fn lemma_number_of_kind_of_a_longer_prefix(
    entries: Seq<LoggedOperationEntry>,
    prefix_length: int,
    kind: LoggedOperationKind,
)
    requires
        0 <= prefix_length < entries.len(),
    ensures
        spec_number_of_kind(entries.take(prefix_length + 1), kind) == spec_number_of_kind(
            entries.take(prefix_length),
            kind,
        ) + if entries[prefix_length].operation.spec_kind() == kind {
            1nat
        } else {
            0nat
        },
{
    assert(entries.take(prefix_length + 1).drop_last() =~= entries.take(prefix_length));
}

/// 一版 logged_ops 里那一条活整理意图（全池至多一条，判决 G1 e1）：签发它的那次发布的 txg 与意图本身。没有交回 `None`。
///
/// # Panics
/// 有两条以上的整理意图（`requires` 排除的那一种）：发布路径只在没有活意图时签发，盘上读回的条目由
/// [`logged_operation_entries_of_the_root_entries`] 拒过多于一条的。
#[must_use]
pub fn live_compaction_intent_of_the_entries(
    entries: &[LoggedOperationEntry],
) -> (live: Option<(CheckpointTxg, CompactionIntent)>)
    requires
        spec_number_of_kind(entries@, LoggedOperationKind::Compaction) <= 1,
    ensures
        match live {
            Some((issued_txg, intent)) => exists|position: int|
                0 <= position < entries@.len() && #[trigger] entries@[position] == (LoggedOperationEntry {
                    issued_txg,
                    operation: LoggedOperation::Compaction(intent),
                }),
            None => spec_number_of_kind(entries@, LoggedOperationKind::Compaction) == 0,
        },
{
    let mut live: Option<(CheckpointTxg, CompactionIntent)> = None;
    let mut checked_count: usize = 0;
    // 逐条看：头一条整理意图记下，再遇到一条就是两条以上（与 filter_map 之后 next 两次同一个判法）。
    while checked_count < entries.len()
        invariant
            checked_count <= entries@.len(),
            spec_number_of_kind(entries@, LoggedOperationKind::Compaction) <= 1,
            match live {
                Some((issued_txg, intent)) => {
                    &&& exists|position: int|
                        0 <= position < checked_count && #[trigger] entries@[position] == (LoggedOperationEntry {
                            issued_txg,
                            operation: LoggedOperation::Compaction(intent),
                        })
                    &&& spec_number_of_kind(entries@.take(checked_count as int), LoggedOperationKind::Compaction)
                        == 1
                },
                None => spec_number_of_kind(entries@.take(checked_count as int), LoggedOperationKind::Compaction)
                    == 0,
            },
        decreases entries@.len() - checked_count,
    {
        proof {
            lemma_number_of_kind_of_a_longer_prefix(entries@, checked_count as int, LoggedOperationKind::Compaction);
            lemma_number_of_kind_of_a_prefix_is_at_most_the_whole(
                entries@,
                checked_count + 1,
                LoggedOperationKind::Compaction,
            );
        }
        match entries[checked_count].operation {
            LoggedOperation::Compaction(intent) => {
                assert!(
                    live.is_none(),
                    "全池活整理意图至多一条（判决 G1 e1）：发布路径只在没有活意图时签发"
                );
                live = Some((entries[checked_count].issued_txg, intent));
            }
            LoggedOperation::Truncation(_intent) => {}
        }
        checked_count += 1;
    }
    assert(entries@.take(checked_count as int) =~= entries@);
    live
}

/// 一版 logged_ops 里那一条活截断意图（全池至多一条）：签发它的那次发布的
/// txg 与意图本身。没有交回 `None`。
///
/// # Panics
/// 有两条以上的截断意图（`requires` 排除的那一种）：签发入口断言上一版没有截断条目，盘上读回的条目由
/// [`logged_operation_entries_of_the_root_entries`] 拒过多于一条的。
#[must_use]
pub fn live_truncation_intent_of_the_entries(
    entries: &[LoggedOperationEntry],
) -> (live: Option<(CheckpointTxg, TruncationIntent)>)
    requires
        spec_number_of_kind(entries@, LoggedOperationKind::Truncation) <= 1,
    ensures
        match live {
            Some((issued_txg, intent)) => exists|position: int|
                0 <= position < entries@.len() && #[trigger] entries@[position] == (LoggedOperationEntry {
                    issued_txg,
                    operation: LoggedOperation::Truncation(intent),
                }),
            None => spec_number_of_kind(entries@, LoggedOperationKind::Truncation) == 0,
        },
{
    let mut live: Option<(CheckpointTxg, TruncationIntent)> = None;
    let mut checked_count: usize = 0;
    // 逐条看：头一条截断意图记下，再遇到一条就是两条以上（与 filter_map 之后 next 两次同一个判法）。
    while checked_count < entries.len()
        invariant
            checked_count <= entries@.len(),
            spec_number_of_kind(entries@, LoggedOperationKind::Truncation) <= 1,
            match live {
                Some((issued_txg, intent)) => {
                    &&& exists|position: int|
                        0 <= position < checked_count && #[trigger] entries@[position] == (LoggedOperationEntry {
                            issued_txg,
                            operation: LoggedOperation::Truncation(intent),
                        })
                    &&& spec_number_of_kind(entries@.take(checked_count as int), LoggedOperationKind::Truncation)
                        == 1
                },
                None => spec_number_of_kind(entries@.take(checked_count as int), LoggedOperationKind::Truncation)
                    == 0,
            },
        decreases entries@.len() - checked_count,
    {
        proof {
            lemma_number_of_kind_of_a_longer_prefix(entries@, checked_count as int, LoggedOperationKind::Truncation);
            lemma_number_of_kind_of_a_prefix_is_at_most_the_whole(
                entries@,
                checked_count + 1,
                LoggedOperationKind::Truncation,
            );
        }
        match entries[checked_count].operation {
            LoggedOperation::Truncation(intent) => {
                assert!(
                    live.is_none(),
                    "全池活截断意图至多一条（判决 m4-trunc-r1 K1）：签发入口断言上一版没有截断条目"
                );
                live = Some((entries[checked_count].issued_txg, intent));
            }
            LoggedOperation::Compaction(_intent) => {}
        }
        checked_count += 1;
    }
    assert(entries@.take(checked_count as int) =~= entries@);
    live
}

/// 前缀里某一种的条数不多于整串里的。
proof fn lemma_number_of_kind_of_a_prefix_is_at_most_the_whole(
    entries: Seq<LoggedOperationEntry>,
    prefix_length: int,
    kind: LoggedOperationKind,
)
    requires
        0 <= prefix_length <= entries.len(),
    ensures
        spec_number_of_kind(entries.take(prefix_length), kind) <= spec_number_of_kind(entries, kind),
    decreases entries.len() - prefix_length,
{
    if prefix_length < entries.len() {
        lemma_number_of_kind_of_a_longer_prefix(entries, prefix_length, kind);
        lemma_number_of_kind_of_a_prefix_is_at_most_the_whole(entries, prefix_length + 1, kind);
    } else {
        assert(entries.take(prefix_length) =~= entries);
    }
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;

    /// 截断条目 44 字节逐字节钉住：key = 种类 2 + 签发 txg；value = inode 号、对象出生代、目标 size、补零 10。
    #[test]
    fn a_truncation_entry_is_kind_two_then_the_issue_txg_inode_object_birth_target_size_and_ten_zero_bytes(
    ) {
        let entry = LoggedOperationEntry {
            issued_txg: CheckpointTxg(0x0102_0304_0506_0708),
            operation: LoggedOperation::Truncation(TruncationIntent {
                inode: InodeNumber(0x1112_1314_1516_1718),
                object_birth: CheckpointTxg(0x2122_2324_2526_2728),
                target_size_in_bytes: 0x3132_3334_3536_3738,
            }),
        };
        let bytes = entry.to_bytes();
        let mut expected = vec![2, 0];
        expected.extend_from_slice(&0x0102_0304_0506_0708u64.to_le_bytes());
        expected.extend_from_slice(&0x1112_1314_1516_1718u64.to_le_bytes());
        expected.extend_from_slice(&0x2122_2324_2526_2728u64.to_le_bytes());
        expected.extend_from_slice(&0x3132_3334_3536_3738u64.to_le_bytes());
        expected.extend_from_slice(&[0; 10]);
        assert_eq!(bytes, expected, "截断条目 44 字节逐字节");
        assert_eq!(LoggedOperationEntry::parse(&bytes), Ok(entry), "读回同一条");
    }

    /// 截断意图的补零有一个字节非零就拒收；一个根里两条截断意图拒收（全池至多一条）。
    #[test]
    fn a_truncation_entry_with_a_non_zero_padding_byte_or_a_second_truncation_entry_is_refused() {
        let entry = |issued_txg: u64| LoggedOperationEntry {
            issued_txg: CheckpointTxg(issued_txg),
            operation: LoggedOperation::Truncation(TruncationIntent {
                inode: InodeNumber(1),
                object_birth: CheckpointTxg(3),
                target_size_in_bytes: 100,
            }),
        };
        let mut padded = entry(5).to_bytes();
        padded[43] = 1;
        assert_eq!(
            LoggedOperationEntry::parse(&padded),
            Err(LoggedOperationEntryRefusal::TruncationPaddingNotZero)
        );
        assert_eq!(
            logged_operation_entries_of_the_root_entries(&[
                entry(5).to_bytes(),
                entry(6).to_bytes()
            ]),
            Err(LoggedOperationsRootRefusal::MoreThanOneTruncationIntent {
                truncation_intents: 2
            })
        );
        assert_eq!(
            live_truncation_intent_of_the_entries(&[entry(5)]),
            Some((CheckpointTxg(5), entry(5).operation_as_truncation())),
        );
        assert_eq!(live_compaction_intent_of_the_entries(&[entry(5)]), None);
    }

    impl LoggedOperationEntry {
        fn operation_as_truncation(&self) -> TruncationIntent {
            match self.operation {
                LoggedOperation::Truncation(intent) => intent,
                LoggedOperation::Compaction(_intent) => panic!("这里只造截断条目"),
            }
        }
    }
}
