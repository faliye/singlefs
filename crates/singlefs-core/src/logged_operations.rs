//! logged_ops 树的条目（D8（核心索引结构） 已定项 5 的意图树；字段表是判决 `research/prompts/m4-gc-r1-main-verification.md`
//! 第三节 G1「单树、定宽联合条目」）：key 10 = 种类 2 + 签发 txg 8，value 34 定宽、按种类解，条目 44，一个码 2 节点装 369 条。
//! 第一版装两种意图：整理意图（种类 1）：R（设备 4 + 起始槽 6 + 槽数 2）、`c0` 8、`segs0` 8、补零 6；截断意图（种类 2，判决
//! `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1）：inode 号 8、对象出生代 8、目标 size 8、补零 10。别的种类码保留，读者拒收
//! （照树表条目 flags「未知位一律拒收」的政策，D8（核心索引结构） 已定项 8）。
//!
//! **不存续做点、不存暂停标志**（判决 G1，攻方 F1 / F4）：暂停就是意图留着不推进；续做每批照目标态从 R 起点重扫，
//! 删意图之前对整个 R 全量核目标态。
//!
//! 和 `crate::lineage_tree`、`crate::tombstone_tree` 一样只算、不发一个写：这里只有字段表的读写，签发、推进、删意图归发布入口。

use singlefs_format::{
    CLUSTER_SEGMENT_SLOTS, LOGGED_OPERATIONS_ENTRY_BYTES, LOGGED_OPERATIONS_KEY_BYTES,
};

use crate::address::{CheckpointTxg, DeviceIdentity, InodeNumber, SlotNumber};
use crate::bytes::{ByteReader, ByteWriter};
use crate::unit::index_node_entry_capacity;

/// logged_ops 树的 key 宽：种类 2 + 签发 txg 8（判决 G1）。码 2 节点头里的自述 key 宽要与它相符。
pub const LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES: usize = 10;

/// logged_ops 条目的种类码：1 = 整理（判决 G1）；2 = 截断（[`LOGGED_OPERATION_KIND_TRUNCATION`]）。别的码保留：读者拒收，不当未来版本的条目猜着读。
pub const LOGGED_OPERATION_KIND_COMPACTION: u16 = 1;

/// logged_ops 条目的种类码：2 = 截断（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1）。
pub const LOGGED_OPERATION_KIND_TRUNCATION: u16 = 2;

/// 整理意图 value 末尾的补零：34 − (4 + 6 + 2 + 8 + 8)。读者遇到非零一律拒收（同树表条目预留位的政策）。
const COMPACTION_INTENT_PADDING_BYTES: usize = 6;

/// 截断意图 value 末尾的补零：34 − (8 + 8 + 8)。读者遇到非零一律拒收（同整理意图的补零）。
const TRUNCATION_INTENT_PADDING_BYTES: usize = 10;

/// 槽号在盘上占 6 字节（D3（空间分配） 已定项 7）：起始槽要小于它才写得下。
const SIX_BYTE_SLOT_NUMBER_LIMIT: u64 = 1 << 48;

/// logged_ops 条目的种类。封闭集合：加一种意图就加一个成员，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LoggedOperationKind {
    /// 整理意图（D26（后台整理与放置回收） 已定项 2；提示类，D8（核心索引结构） 已定项 5）。
    Compaction,
    /// 截断意图（承诺类，D8（核心索引结构） 已定项 5；判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1）。
    Truncation,
}

impl LoggedOperationKind {
    /// 写进 key 前 2 字节的种类码。
    #[must_use]
    pub const fn code(self) -> u16 {
        match self {
            LoggedOperationKind::Compaction => LOGGED_OPERATION_KIND_COMPACTION,
            LoggedOperationKind::Truncation => LOGGED_OPERATION_KIND_TRUNCATION,
        }
    }

    /// 盘上读到的种类码认不认得：认不得的（保留码）交回 `None`。
    #[must_use]
    pub const fn of_code(kind_code: u16) -> Option<Self> {
        match kind_code {
            LOGGED_OPERATION_KIND_COMPACTION => Some(LoggedOperationKind::Compaction),
            LOGGED_OPERATION_KIND_TRUNCATION => Some(LoggedOperationKind::Truncation),
            _reserved_kind_code => None,
        }
    }
}

/// 整理意图的区域 R：一块盘上一个 64 槽对齐的聚簇段（判决 G1：起始槽 64 对齐、槽数 = 64；第一版两盘同槽时 R 罩住每块盘上的同号槽，
/// 攻方 K-n）。只从 [`CompactionRegion::of_one_cluster_segment`] 进来，所以槽数不存：恒是 [`CLUSTER_SEGMENT_SLOTS`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CompactionRegion {
    device: DeviceIdentity,
    start_slot: SlotNumber,
}

/// 造不出一个整理区域的两种起始槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionRegionRefusal {
    /// 起始槽不是 64 的倍数：R 不对齐（判决 G1「R 的解码拒不对齐的值」）。
    StartOffAClusterSegmentBoundary { start_slot: SlotNumber },
    /// 起始槽装不进盘上 6 字节的槽号。
    StartBeyondTheSixByteSlotNumber { start_slot: SlotNumber },
}

impl CompactionRegion {
    /// `device` 上从 `start_slot` 起的那一个聚簇段。
    ///
    /// # Errors
    /// 起始槽不是 64 的倍数 ⇒ `StartOffAClusterSegmentBoundary`；装不进 6 字节 ⇒ `StartBeyondTheSixByteSlotNumber`。
    pub fn of_one_cluster_segment(
        device: DeviceIdentity,
        start_slot: SlotNumber,
    ) -> Result<Self, CompactionRegionRefusal> {
        if !start_slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {
            return Err(CompactionRegionRefusal::StartOffAClusterSegmentBoundary { start_slot });
        }
        if start_slot.0 >= SIX_BYTE_SLOT_NUMBER_LIMIT {
            return Err(CompactionRegionRefusal::StartBeyondTheSixByteSlotNumber { start_slot });
        }
        Ok(Self { device, start_slot })
    }

    #[must_use]
    pub const fn device(self) -> DeviceIdentity {
        self.device
    }

    #[must_use]
    pub const fn start_slot(self) -> SlotNumber {
        self.start_slot
    }

    /// 槽数：恒是一个聚簇段。
    #[must_use]
    pub const fn slot_count(self) -> u64 {
        CLUSTER_SEGMENT_SLOTS
    }

    /// R 之后的第一个槽（开区间的右端）。
    #[must_use]
    pub const fn end_slot(self) -> SlotNumber {
        SlotNumber(self.start_slot.0 + CLUSTER_SEGMENT_SLOTS)
    }

    /// 这个槽号在不在 R 里（只看槽号：两盘同槽时 R 罩住每块盘上的同号槽）。
    #[must_use]
    pub const fn contains_slot(self, slot: SlotNumber) -> bool {
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
    /// `segs0`：签发那次发布写下的记账第 11 项（全空聚簇段数）里 R 所在盘的值；只作观测量，停机不靠它（判决 G1 / G4）。
    pub empty_cluster_segments_at_issue: u64,
}

/// 一条截断意图（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K2）：把 `inode`（对象出生代 `object_birth`）
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

impl LoggedOperation {
    #[must_use]
    pub const fn kind(&self) -> LoggedOperationKind {
        match self {
            LoggedOperation::Compaction(_intent) => LoggedOperationKind::Compaction,
            LoggedOperation::Truncation(_intent) => LoggedOperationKind::Truncation,
        }
    }
}

/// logged_ops 条目的 key：种类 + 签发 txg。全序按逐字段无符号整数比（D8（核心索引结构） 已定项 11：小端存储不构成 memcmp 序），
/// 先按种类、再按签发先后。一次发布至多签发一条（判决 G1 e1），所以 key 不带序号。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LoggedOperationKey {
    pub kind: LoggedOperationKind,
    pub issued_txg: CheckpointTxg,
}

impl LoggedOperationKey {
    #[must_use]
    pub fn sort_key(&self) -> (u16, u64) {
        (self.kind.code(), self.issued_txg.0)
    }
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

fn entry_bytes() -> usize {
    usize::try_from(LOGGED_OPERATIONS_ENTRY_BYTES).expect("44")
}

impl LoggedOperationEntry {
    #[must_use]
    pub const fn key(&self) -> LoggedOperationKey {
        LoggedOperationKey {
            kind: self.operation.kind(),
            issued_txg: self.issued_txg,
        }
    }

    /// 条目 44：种类 0 | 签发 txg 2 | 整理：R 的设备 10 | R 的起始槽 14 | R 的槽数 20 | `c0` 22 | `segs0` 30 | 补零 38；
    /// 截断：inode 号 10 | 对象出生代 18 | 目标 size 26 | 补零 34，小端。
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(entry_bytes());
        writer.put_u16(self.operation.kind().code());
        writer.put_u64(self.issued_txg.0);
        writer.assert_position(LOGGED_OPERATIONS_KEY_BYTES, "logged_ops 条目的 key");
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
        writer.assert_position(LOGGED_OPERATIONS_ENTRY_BYTES, "logged_ops 条目");
        writer.into_bytes()
    }

    /// logged_ops 条目 44 的读者。
    ///
    /// # Errors
    /// 宽不是 44 ⇒ `WidthIsNotTheFieldTable`；种类码保留 ⇒ `UnknownKind`；整理意图的 R 起始槽不是 64 的倍数 ⇒
    /// `CompactionRegionRefused`；R 槽数不是 64 ⇒ `CompactionRegionSlotCountIsNotOneClusterSegment`；补零非零 ⇒ `CompactionPaddingNotZero`；
    /// 截断意图的补零非零 ⇒ `TruncationPaddingNotZero`。
    pub fn parse(bytes: &[u8]) -> Result<Self, LoggedOperationEntryRefusal> {
        if bytes.len() != entry_bytes() {
            return Err(LoggedOperationEntryRefusal::WidthIsNotTheFieldTable {
                entry_bytes: bytes.len(),
            });
        }
        let mut reader = ByteReader::at(bytes, 0);
        let kind_code = reader.get_u16();
        let issued_txg = CheckpointTxg(reader.get_u64());
        let kind = LoggedOperationKind::of_code(kind_code)
            .ok_or(LoggedOperationEntryRefusal::UnknownKind { kind_code })?;
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

/// 整理意图那 34 字节 value（读者已在 key 之后）。
fn parse_compaction_intent(
    reader: &mut ByteReader<'_>,
) -> Result<CompactionIntent, LoggedOperationEntryRefusal> {
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
    if padding.iter().any(|byte| *byte != 0) {
        return Err(LoggedOperationEntryRefusal::CompactionPaddingNotZero);
    }
    let region = CompactionRegion::of_one_cluster_segment(device, start_slot)
        .map_err(LoggedOperationEntryRefusal::CompactionRegionRefused)?;
    Ok(CompactionIntent {
        region,
        allocation_generation_at_issue,
        empty_cluster_segments_at_issue,
    })
}

/// 截断意图那 34 字节 value（读者已在 key 之后）。
fn parse_truncation_intent(
    reader: &mut ByteReader<'_>,
) -> Result<TruncationIntent, LoggedOperationEntryRefusal> {
    let inode = InodeNumber(reader.get_u64());
    let object_birth = CheckpointTxg(reader.get_u64());
    let target_size_in_bytes = reader.get_u64();
    let padding = reader.take(TRUNCATION_INTENT_PADDING_BYTES);
    if padding.iter().any(|padding_byte| *padding_byte != 0) {
        return Err(LoggedOperationEntryRefusal::TruncationPaddingNotZero);
    }
    Ok(TruncationIntent {
        inode,
        object_birth,
        target_size_in_bytes,
    })
}

/// 一个 logged_ops 码 2 根（层级 0）装得下几条：(16384 − 含预留位的头 135) ÷ 44 的整数部分 = 369（判决 G1「一节点 369 条」）。
#[must_use]
pub fn logged_operations_one_root_node_holds() -> usize {
    index_node_entry_capacity(LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES, entry_bytes())
}

/// logged_ops 根要装的条目：每条 44 字节（[`LoggedOperationEntry::to_bytes`]），按 key 的全序升序。
/// logged_ops 第一版只有一个码 2 根、根之下不长层：全池活整理意图至多一条（判决 G1 e1）、活截断意图至多一条（判决 m4-trunc-r1 K1），
/// 一个根装 369 条。
///
/// # Panics
/// `entries` 不是按 key 严格升序、或多于一个根装得下的：条目由发布路径按「签发加一条、删掉一条」维护，至多一条活整理意图，
/// 乱序或装不下说明调用方拼错了。
#[must_use]
pub fn logged_operations_root_entries(entries: &[LoggedOperationEntry]) -> Vec<Vec<u8>> {
    assert!(
        entries
            .windows(2)
            .all(|pair| pair[0].key().sort_key() < pair[1].key().sort_key()),
        "logged_ops 的条目按 key 升序、不重复：发布路径按集合维护它"
    );
    assert!(
        entries.len() <= logged_operations_one_root_node_holds(),
        "logged_ops 至多一条活整理意图与一条活截断意图，一个根装得下"
    );
    entries.iter().map(LoggedOperationEntry::to_bytes).collect()
}

/// 从一个 logged_ops 根的条目读回意图（条目是盘上读来的字节，逐条照字段表解）。
///
/// # Errors
/// 有一条过不了 [`LoggedOperationEntry::parse`]（宽不是 44、种类码保留、R 不对齐或槽数不是 64、补零非零）、
/// 几条不按 key 严格升序、整理意图多于一条（全池至多一条，判决 G1 e1）、截断意图多于一条（全池至多一条，判决 m4-trunc-r1 K1）
/// ⇒ [`LoggedOperationsRootRefusal`]。
pub fn logged_operation_entries_of_the_root_entries(
    entries: &[Vec<u8>],
) -> Result<Vec<LoggedOperationEntry>, LoggedOperationsRootRefusal> {
    let parsed: Vec<LoggedOperationEntry> = entries
        .iter()
        .map(|entry| LoggedOperationEntry::parse(entry))
        .collect::<Result<_, _>>()
        .map_err(LoggedOperationsRootRefusal::EntryRefused)?;
    if !parsed
        .windows(2)
        .all(|pair| pair[0].key().sort_key() < pair[1].key().sort_key())
    {
        return Err(LoggedOperationsRootRefusal::EntriesNotInStrictlyAscendingKeyOrder);
    }
    let compaction_intents = parsed
        .iter()
        .filter(|entry| entry.operation.kind() == LoggedOperationKind::Compaction)
        .count();
    if compaction_intents > 1 {
        return Err(LoggedOperationsRootRefusal::MoreThanOneCompactionIntent {
            compaction_intents,
        });
    }
    let truncation_intents = parsed
        .iter()
        .filter(|entry| entry.operation.kind() == LoggedOperationKind::Truncation)
        .count();
    if truncation_intents > 1 {
        return Err(LoggedOperationsRootRefusal::MoreThanOneTruncationIntent {
            truncation_intents,
        });
    }
    Ok(parsed)
}

/// 读者拒收一个 logged_ops 根的原因：根里的字节是盘上读来的，调用方把它当损坏处置（可写挂载拒）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoggedOperationsRootRefusal {
    EntryRefused(LoggedOperationEntryRefusal),
    EntriesNotInStrictlyAscendingKeyOrder,
    MoreThanOneCompactionIntent { compaction_intents: usize },
    MoreThanOneTruncationIntent { truncation_intents: usize },
}

/// 一版 logged_ops 里那一条活整理意图（全池至多一条，判决 G1 e1）：签发它的那次发布的 txg 与意图本身。没有交回 `None`。
///
/// # Panics
/// 有两条以上的整理意图：发布路径只在没有活意图时签发，盘上读回的条目由 [`logged_operation_entries_of_the_root_entries`] 拒过多于一条的。
#[must_use]
pub fn live_compaction_intent_of_the_entries(
    entries: &[LoggedOperationEntry],
) -> Option<(CheckpointTxg, CompactionIntent)> {
    let mut intents = entries.iter().filter_map(|entry| match entry.operation {
        LoggedOperation::Compaction(intent) => Some((entry.issued_txg, intent)),
        LoggedOperation::Truncation(_intent) => None,
    });
    let live = intents.next();
    assert!(
        intents.next().is_none(),
        "全池活整理意图至多一条（判决 G1 e1）：发布路径只在没有活意图时签发"
    );
    live
}

/// 一版 logged_ops 里那一条活截断意图（全池至多一条，判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1）：签发它的那次发布的
/// txg 与意图本身。没有交回 `None`。
///
/// # Panics
/// 有两条以上的截断意图：签发入口断言上一版没有截断条目，盘上读回的条目由 [`logged_operation_entries_of_the_root_entries`] 拒过多于一条的。
#[must_use]
pub fn live_truncation_intent_of_the_entries(
    entries: &[LoggedOperationEntry],
) -> Option<(CheckpointTxg, TruncationIntent)> {
    let mut intents = entries.iter().filter_map(|entry| match entry.operation {
        LoggedOperation::Truncation(intent) => Some((entry.issued_txg, intent)),
        LoggedOperation::Compaction(_intent) => None,
    });
    let live = intents.next();
    assert!(
        intents.next().is_none(),
        "全池活截断意图至多一条（判决 m4-trunc-r1 K1）：签发入口断言上一版没有截断条目"
    );
    live
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 截断条目 44 字节逐字节钉住（判决 m4-trunc-r1 K1）：key = 种类 2 + 签发 txg；value = inode 号、对象出生代、目标 size、补零 10。
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
