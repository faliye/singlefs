//! 挂载态的读（里程碑「第二个事务」并行线二）：池打开之后留一份内存里的派生态——根、树表条目、读过的树节点与
//! 解出来的记录——之后「按 inode 与偏移读 N 字节」只解引用数据单元，不重走一次冷恢复、一次都不扫 journal 环。
//!
//! 压着它的条款：
//! - D21（权威态与派生态的分界） 已定项 9 / 已定项 10：索引是派生态，挂载态里的这一份全是从盘上读回来的，
//!   丢了就从盘上重走 ⇒ 这里一个字节都不回写、不缓存数据单元的载荷，判定要的每个值都能再走一遍权威态得出。
//! - D19（块指针的结构与宽度预算） 已定项 5：位置条目**是提示**，解引用先按提示读，读不到或校验和不对再查中央映射；
//!   随定案生效的硬规则 3「提示过期的多跳率要有运行时观测点」⇒ [`ReadPathObservation`]。
//! - D4（校验和位置） 已定项 5：单元恒 32768 含头 ⇒ 净荷 32634 不是 4096 的整数倍，文件偏移到单元做除法而不是移位，
//!   一成多的 4 KiB 页跨在两个单元上、那一次读要读两个单元（[`DataUnitSpanOfARead`]）。
//! - D17（实现分层与第三方管道） 已定项 5：第一版是纯用户态库、只对块设备抽象编程；那条射程 ① 要实现给出
//!   「一次 API 调用发了几个设备级操作」的读数交装置比对 ⇒ [`ReadPathObservation::device_reads_issued`]。
//!
//! - D8（核心索引结构） 已定项 3（C490（extent 叶 key 的 offset 段没定单位） 2026-09-23 定）：extent 叶记录 key 的 offset 段
//!   是**文件字节偏移**，第 n 个数据单元是 n × 净荷容量；数据单元头里的锚点偏移是同一个数（D9（加密） 已定项 6）。
//!   [`MountedPoolForRead::open_file`] 当场核：这个 inode 的第 i 条记录的 offset 段要等于 i × 净荷容量（没有洞、没有错位），
//!   记录条数要等于文件大小按 D4（校验和位置） 已定项 5 的除法算出来的单元数；对不上就拒绝打开这个文件，不猜。
//!   于是「第 i 条记录就是文件第 i 个数据单元」这件事由 key 本身担保，解引用时再核单元头的锚点偏移与 key 相等。
//!
//! extent 树的节点只在打开时读（[`MountedPoolForRead::extent_tree_reads_at_open`]）：之后按偏移读一个字节都不再碰它，
//! 顺序读 M 个单元时 extent 节点的读取次数 ≤ 树高 + 叶数（里程碑「第二个事务」并行线一验收第 2 条）。

use std::cell::Cell;

use singlefs_format::{DATA_UNIT_BYTES, INODE_RECORD_BYTES, MAPPING_KEY_BYTES, NODE_BYTES};

use crate::address::{
    DataUnitIndexInFile, FileOffsetInBytes, InodeNumber, SlotNumber, TreeIdentifier,
};
use crate::checksum::crc32_castagnoli;
use crate::pointer::{DataPointer, LocationEntry, NodePointer};
use crate::records::{
    mapping_key_for_data, parse_extent_record, parse_inode_internal_entry, parse_mapping_entry,
    InodeRecord, TreeTableEntry, TREE_KIND_EXTENT, TREE_KIND_INODE,
};
use crate::recovery::{
    choose_root, choose_system_configuration, read_mapped_tree_node_via_hint_then_central_mapping,
    read_mapped_tree_root, read_tree_root, read_unit_via_locations, replay_journal,
    rollback_high_water_of_root, scan_journal, JournalScanReport, MappedTreeNodeClass, PoolReader,
    RecoveryFailure,
};
use crate::root_record::RootRecord;
use crate::unit::{
    data_unit_payload, data_unit_payload_capacity, parse_data_unit, parse_index_node,
    parse_packed_unit, unit_filesystem_identifier, DataUnitHeader, UnitError, PACKED_TYPE_INODE,
};
use crate::write_request_split::data_unit_count_of_a_sequential_write;

/// extent 叶记录 key 三段各 8 字节里 inode 那一段的起点（D8（核心索引结构） 已定项 3 的 (locality_id, inode, offset)）。
const EXTENT_KEY_INODE_SEGMENT_OFFSET_IN_BYTES: usize = 8;
/// extent 叶记录 key 第三段（offset 段，文件字节偏移，D8（核心索引结构） 已定项 3）的起点。
const EXTENT_KEY_THIRD_SEGMENT_OFFSET_IN_BYTES: usize = 16;
/// extent 树的根兼叶层级：第一版一棵树只有一个节点，记录直接装在根里（`transaction::build_file_version_units`）。
const EXTENT_TREE_ROOT_LEVEL: u8 = 0;
/// 中央映射树的根兼叶层级：第一版同样只有一个节点，55 字节的映射条目直接装在根里。
/// 挂载态把这一片**整个**读进内存，于是解引用时查映射不再发设备读——[`OpenFileForRead::read_at`] 报的
/// `device_reads_issued` 是按这个前提算的（提示过期的一次解引用 = 两条提示各试一次 + 映射落点一次 = 3）。
/// 映射长到根成了内部节点的那天这条前提不再成立，第一版不走树：在这里拒绝，不把内部条目当映射条目解
/// （D19（块指针的结构与宽度预算） 已定项 5「挂载态怎么读映射」：整片读、多层拒绝打开，第一版的限制；
/// 那一维正是已定项 5 自陈零测量的缓存命中维，C418（位置权威三臂的缓存命中维零测量））。
const CENTRAL_MAPPING_TREE_ROOT_LEVEL: u8 = 0;
/// inode 树根的层级：根是内部节点，每条条目指一片码 3 叶容器（D8（核心索引结构） 已定项 6）。
const INODE_TREE_ROOT_LEVEL: u8 = 1;
/// extent 叶记录 key 宽 24，inode 树 key 宽 8（`recovery::key_width_for_kind` 同一份登记）。
const EXTENT_KEY_WIDTH_IN_BYTES: usize = 24;
const INODE_KEY_WIDTH_IN_BYTES: usize = 8;

/// 读路径的运行时观测点：并行线二验收第 4 条要的两个数（这次读了几个单元、位置提示过期的多跳次数），
/// 加上 D17（实现分层与第三方管道） 已定项 5 射程 ① 要的「这次 API 调用发了几个设备级操作」。
/// 计数只加不减：一次读返回这一次的，[`MountedPoolForRead::observation_since_open`] 是打开之后累计的。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReadPathObservation {
    /// 读回了几个数据单元。跨两个单元的页是 2。
    pub data_units_read: u64,
    /// 解引用了几次数据单元：多跳率的分母。
    pub data_unit_dereferences: u64,
    /// 位置提示不算数、转去查中央映射的次数（D19（块指针的结构与宽度预算） 已定项 5 硬规则 3 的观测点）：多跳率的分子。
    pub stale_location_hint_hops: u64,
    /// 向块层发了几次读（每条位置条目试一次就算一次，读不回字节也算——请求已经发出去了）。
    pub device_reads_issued: u64,
}

impl ReadPathObservation {
    /// 两份计数逐项相加：一次读由若干次解引用合成，一次挂载由若干次读合成。
    #[must_use]
    pub fn plus(self, later: Self) -> Self {
        Self {
            data_units_read: self.data_units_read + later.data_units_read,
            data_unit_dereferences: self.data_unit_dereferences + later.data_unit_dereferences,
            stale_location_hint_hops: self.stale_location_hint_hops
                + later.stale_location_hint_hops,
            device_reads_issued: self.device_reads_issued + later.device_reads_issued,
        }
    }

    /// 提示过期的多跳率，按百万分之几报（不引浮点；一次解引用都没有时报 0）。
    #[must_use]
    pub fn stale_location_hint_hops_per_million_dereferences(self) -> u64 {
        if self.data_unit_dereferences == 0 {
            return 0;
        }
        let parts = u128::from(self.stale_location_hint_hops) * 1_000_000
            / u128::from(self.data_unit_dereferences);
        u64::try_from(parts).expect("多跳次数不超过解引用次数 ⇒ 比率不超过一百万")
    }
}

/// 一次读碰到的数据单元序号闭区间（D4（校验和位置） 已定项 5：文件偏移到单元做除法）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataUnitSpanOfARead {
    pub first: DataUnitIndexInFile,
    pub last_inclusive: DataUnitIndexInFile,
}

impl DataUnitSpanOfARead {
    /// 这次读要读两个（或更多）单元：净荷 32634 不是 4096 的整数倍，约 12.5% 的 4 KiB 页落在这一档
    /// （D4（校验和位置） 已定项 5 的 E140（单元含头逼出的跨单元页与读侧拷贝） 那一段）。
    #[must_use]
    pub const fn spans_more_than_one_data_unit(self) -> bool {
        self.last_inclusive.0 > self.first.0
    }
}

/// `[first_byte, last_byte_inclusive]` 这段文件字节落在哪几个数据单元上：两端各除以净荷容量
/// （D4（校验和位置） 已定项 5：文件偏移到单元做除法而不是移位）。
/// 参数是**闭区间**，长度为 0 的读表达不出来——那一档由 [`OpenFileForRead::read_at`] 先分出去。
#[must_use]
pub fn data_unit_span_covering(
    first_byte: FileOffsetInBytes,
    last_byte_inclusive: FileOffsetInBytes,
) -> DataUnitSpanOfARead {
    let payload_capacity = payload_capacity_in_bytes();
    DataUnitSpanOfARead {
        first: DataUnitIndexInFile(first_byte.0 / payload_capacity),
        last_inclusive: DataUnitIndexInFile(last_byte_inclusive.0 / payload_capacity),
    }
}

/// 一个数据单元装得下多少字节用户数据，换成 `u64`：唯一的来源是 [`crate::unit::data_unit_payload_capacity`]。
fn payload_capacity_in_bytes() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

/// 挂载态里留着的一条 extent 叶记录：盘上那 24 字节 key 与 88 字节数据指针。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtentLeafRecordInMountState {
    key: [u8; 24],
    data_unit_pointer: DataPointer,
}

impl ExtentLeafRecordInMountState {
    /// key 第二段：这条记录属于哪个 inode。
    #[must_use]
    pub fn inode_number(&self) -> InodeNumber {
        InodeNumber(u64::from_le_bytes(
            self.key[EXTENT_KEY_INODE_SEGMENT_OFFSET_IN_BYTES
                ..EXTENT_KEY_INODE_SEGMENT_OFFSET_IN_BYTES + 8]
                .try_into()
                .expect("key 24 字节，第二段落在 [8, 16)"),
        ))
    }

    /// key 第三段：这条记录指的那个数据单元第一个字节在文件里的偏移（D8（核心索引结构） 已定项 3）。
    #[must_use]
    pub fn file_offset_in_bytes(&self) -> FileOffsetInBytes {
        FileOffsetInBytes(u64::from_le_bytes(
            self.key[EXTENT_KEY_THIRD_SEGMENT_OFFSET_IN_BYTES
                ..EXTENT_KEY_THIRD_SEGMENT_OFFSET_IN_BYTES + 8]
                .try_into()
                .expect("key 24 字节，第三段落在 [16, 24)"),
        ))
    }

    /// 这条记录指的那个数据单元的指针（位置条目是提示，D19（块指针的结构与宽度预算） 已定项 5）。
    #[must_use]
    pub const fn data_unit_pointer(&self) -> DataPointer {
        self.data_unit_pointer
    }
}

/// 中央映射里的一条：27 字节 key 加两条位置条目（D19（块指针的结构与宽度预算） 已定项 10）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct CentralMappingEntryInMountState {
    key: Vec<u8>,
    locations: [LocationEntry; 2],
}

/// 打开挂载态的读没做成。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpenPoolForReadFailure {
    /// 沿根记录读树表、树根、inode 叶容器时判红的那一条：与冷启动走读（`recovery::walk_to_file`）同一套口径。
    Walk(RecoveryFailure),
    /// 树表 0 条：这一版还没发布过文件版本，一个文件都打不开（不是坏盘）。
    TreeTableHasNoEntries,
    /// 树表条目解不开（条目长度、flags 未知位、预留 76 非零）。
    TreeTableEntryMalformed,
    /// 树表里没有这个种类的树、或者它的根指针全零（还没根）。
    TreeTableHasNoTreeOfKind { kind: u16 },
    /// 某个节点的自述层级不是这棵树该有的（extent 根兼叶是 0，inode 根是 1）。
    TreeLevelUnexpected {
        tree: TreeIdentifier,
        expected_level: u8,
        found_level: u8,
    },
    /// 中央映射树长成了多层（根不是根兼叶）：第一版挂载态把映射整片读进来、之后查映射不发读，
    /// 多层映射第一版不支持，拒绝打开（D19（块指针的结构与宽度预算） 已定项 5「挂载态怎么读映射」）。
    /// 在读映射树根之后、读任何别的单元之前返回。
    CentralMappingWithMoreThanOneLevelIsNotSupportedInTheFirstVersion {
        mapping_tree: TreeIdentifier,
        mapping_root_level: u8,
    },
    /// 读回来的单元解不开（头坏了、载荷校验和不对、补齐非零）。
    UnitMalformed {
        what: &'static str,
        error: UnitError,
    },
    /// inode 叶容器的头与 inode 树内部条目里的身份引用不符（I-9.2（条目身份与子头相符））。
    InodeLeafContainerDisagreesWithTheInternalEntry { detail: &'static str },
    /// inode 记录解不开（填充 / flags / 预留非零，I-9.7）。
    InodeRecordMalformed,
    /// 节点里的一条记录 / 条目解不开：盘上自述的条目宽窄于字段表。
    RecordMalformed { what: &'static str },
}

impl From<RecoveryFailure> for OpenPoolForReadFailure {
    fn from(failure: RecoveryFailure) -> Self {
        OpenPoolForReadFailure::Walk(failure)
    }
}

/// 打开一个文件没做成。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpenFileFailure {
    /// inode 树里没有这个号。
    NoSuchInode { inode: InodeNumber },
    /// 这个 inode 的第 `unit_index_in_file` 条 extent 叶记录，key 的 offset 段不是那个单元第一个字节的文件偏移
    /// （`unit_index_in_file` × 净荷容量，D8（核心索引结构） 已定项 3）：记录错位、有洞，或 offset 段写成了单元序号。
    /// 报第一处对不上的那一条；不按位次猜着往下读。
    ExtentRecordKeyIsNotTheFileOffsetOfItsUnit {
        inode: InodeNumber,
        unit_index_in_file: DataUnitIndexInFile,
        key_file_offset: FileOffsetInBytes,
        expected_file_offset: FileOffsetInBytes,
    },
    /// 这个 inode 的 extent 叶记录条数与「文件大小按 D4（校验和位置） 已定项 5 的除法算出来的单元数」对不上：
    /// 文件有洞（稀疏）或尾巴上多出记录。第一版不写稀疏文件：在这里拒绝，不猜。
    ExtentRecordCountDoesNotMatchTheFileSize {
        inode: InodeNumber,
        extent_records: u64,
        data_units_implied_by_the_file_size: u64,
        file_size_in_bytes: u64,
    },
}

/// 一次读没做成，按调用方要做的决定分：调用方自己越界 / 盘上是坏数据 / 读不到 / 盘上的自描述与查找路径不符。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileReadFailure {
    /// 读的区间越过文件末尾。第一版不做短读、不补零：越界当场拒绝，由调用方按 size 自己切。
    RangeBeyondEndOfFile {
        file_size_in_bytes: u64,
        offset: FileOffsetInBytes,
        length_in_bytes: u64,
    },
    /// 位置提示与中央映射的位置条目都读回了字节，而整单元校验和一条都对不上：这是**坏数据**，不是读不到。
    DataUnitChecksumMismatchEverywhere {
        unit_index_in_file: DataUnitIndexInFile,
        hinted_slot: SlotNumber,
    },
    /// 位置提示不算数，而中央映射里没有这个单元的 key：解引用的唯一入口空了（D19（块指针的结构与宽度预算） 已定项 5）。
    CentralMappingMiss {
        unit_index_in_file: DataUnitIndexInFile,
        hinted_slot: SlotNumber,
    },
    /// 提示与映射的位置条目都读不回字节（盘不在、越界）。
    DataUnitUnreadable {
        unit_index_in_file: DataUnitIndexInFile,
        hinted_slot: SlotNumber,
    },
    /// 单元读回来了、解不开。
    DataUnitMalformed {
        unit_index_in_file: DataUnitIndexInFile,
        error: UnitError,
    },
    /// 单元的自描述与查找路径不符：五元组、出生身份、声明长度、跨单元那一页上两个锚点偏移的次序。
    DataUnitDisagreesWithTheLookupPath {
        unit_index_in_file: DataUnitIndexInFile,
        invariant: &'static str,
        detail: &'static str,
    },
}

/// 一次读的结果：读出来的字节，加这一次的运行时观测点。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileReadOutput {
    pub bytes: Vec<u8>,
    pub observation: ReadPathObservation,
}

/// 打开挂载态时沿 extent 树读了几个节点，与这棵树的高度、叶片数（里程碑「第二个事务」并行线一验收第 2 条的观测点）。
/// 打开之后按偏移读一个 extent 节点都不再读 ⇒ 顺序读 M 个单元时 extent 节点的读取次数就是 `node_reads`，
/// 验收要它 ≤ `height + leaves`，不是每个单元从 inode 重走一遍。第一版 extent 树只有一个节点（根兼叶；长出内部节点那一档
/// 写侧在落盘之前拒掉，`transaction::PublishError::ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtentTreeReadsAtOpen {
    /// 打开时向块层要过几个 extent 树节点（每读一个节点算一次，不按位置条目数）。
    pub node_reads: u64,
    /// 树高：根的层级加一（层级 0 是叶）。
    pub height: u64,
    /// 叶片数：读到的层级 0 的节点个数。
    pub leaves: u64,
}

/// 挂载态：池打开之后留在内存里的派生态（D21（权威态与派生态的分界））。**不存数据单元的载荷**——
/// 第一版不做读缓存、不做预读，每次读都去盘上解引用。
pub struct MountedPoolForRead {
    root: RootRecord,
    filesystem_identifier_in_unit_headers: u64,
    extent_tree: TreeIdentifier,
    extent_tree_reads_at_open: ExtentTreeReadsAtOpen,
    /// extent 树根兼叶里的记录，**按盘上次序**（= key 升序，I-9.12（分隔 key 落在孩子区间之外） 那一族由 checker 判）。
    extent_leaf_records: Vec<ExtentLeafRecordInMountState>,
    /// inode 树全部叶容器里的记录，按叶序、叶内次序。
    inode_records: Vec<InodeRecord>,
    central_mapping_entries: Vec<CentralMappingEntryInMountState>,
    observation_since_open: Cell<ReadPathObservation>,
    /// 打开这一趟里树节点（extent 树根、inode 树根、inode 叶容器）的位置提示读不出、转去查中央映射的次数
    /// （D19（块指针的结构与宽度预算） 已定项 5 硬规则 3 的观测点；与 [`ReadPathObservation`] 分开数，
    /// 那边的多跳率分母是数据单元的解引用次数）。
    tree_node_stale_location_hint_hops_at_open: u64,
}

/// 打开挂载态：把根记录到树表条目到两棵用户可见的树这一段读回内存，之后按偏移读不再碰它们。
///
/// 读的次序：中央映射树的根（自举豁免，只按提示读，D19（块指针的结构与宽度预算） 已定项 8）→ 树表（同样豁免）
/// → 树表条目 → extent 树根兼叶 → inode 树根 → 每一片 inode 叶容器。
///
/// **树节点的位置提示读不出时经中央映射回退**（D19（块指针的结构与宽度预算） 已定项 8）：extent 树根、inode 树根、
/// 每一片 inode 叶容器走 `recovery::read_mapped_tree_node_via_hint_then_central_mapping`，与冷启动走读
/// （`recovery::walk_to_file`）同一条；查的是这一趟最先读进来的那片映射条目。映射树根与树表是自举豁免，只按提示读。
///
/// 打开这一段发了几次块层读**不由核心层自己数**：这几步走的是恢复路径那几个读者（`read_tree_root`、
/// `read_unit_via_locations`），它们不带计数。要这个数就在块层数（D17（实现分层与第三方管道） 已定项 5 射程 ①：
/// 录制钩子挂在块设备接口上），装置侧的计数器在 `singlefs_harness::read_tally`。
/// 核心层自己报的那几个数只罩打开之后的每一次读（[`ReadPathObservation`]）。
///
/// # Errors
/// 见 [`OpenPoolForReadFailure`]：读不到 / 解不开 / 树表 0 条 / 树表里缺树 / 层级不对 / 叶容器身份不符。
pub fn open_pool_for_read(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<MountedPoolForRead, OpenPoolForReadFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let filesystem_identifier_in_unit_headers =
        unit_filesystem_identifier(&root.filesystem_identifier);

    // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）；它自己不进映射（自举豁免）。
    // 它不进树表，是哪棵树由根记录里它那条根指针的出生树说（第一个文件版本那次从水位发的号）。
    let mapping_tree = root.mapping_root.head.birth_tree;
    let mapping_root = read_tree_root(
        reader,
        mapping_tree,
        usize::try_from(MAPPING_KEY_BYTES).expect("27"),
        &root.mapping_root,
        root,
        filesystem_identifier_in_unit_headers,
    )?;
    if mapping_root.level != CENTRAL_MAPPING_TREE_ROOT_LEVEL {
        return Err(
            OpenPoolForReadFailure::CentralMappingWithMoreThanOneLevelIsNotSupportedInTheFirstVersion {
                mapping_tree,
                mapping_root_level: mapping_root.level,
            },
        );
    }
    // 映射条目的宽度是映射树根自述的（`read_tree_root` 只判了它 ≥ key 宽 27）：切到偏移 55 之前判一次，
    // 窄的报 `RecordMalformed`，不按字段表的固定偏移切下去（panic 面普查 R2）。
    let mut central_mapping_entries = Vec::with_capacity(mapping_root.entries.len());
    for entry_bytes in &mapping_root.entries {
        let (key, locations) =
            parse_mapping_entry(entry_bytes).ok_or(OpenPoolForReadFailure::RecordMalformed {
                what: "映射条目",
            })?;
        central_mapping_entries.push(CentralMappingEntryInMountState { key, locations });
    }

    // 树表单元：一次读就拿到每棵树的根指针连校验和（2026-09-17 用户指示：树表条目 200 字节带着根指针，
    // 不用再翻别的结构找根、也不用另读一次才能验）。
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table = parse_index_node(&tree_table_bytes).map_err(|error| {
        OpenPoolForReadFailure::UnitMalformed {
            what: "树表单元",
            error,
        }
    })?;
    if tree_table.entries.is_empty() {
        return Err(OpenPoolForReadFailure::TreeTableHasNoEntries);
    }
    let mut tree_table_entries = Vec::with_capacity(tree_table.entries.len());
    for entry_bytes in &tree_table.entries {
        tree_table_entries.push(
            TreeTableEntry::parse(entry_bytes)
                .ok_or(OpenPoolForReadFailure::TreeTableEntryMalformed)?,
        );
    }
    let entry_of_kind = |kind: u16| {
        tree_table_entries
            .iter()
            .find(|entry| entry.kind == kind && entry.root != NodePointer::empty_root())
            .copied()
            .ok_or(OpenPoolForReadFailure::TreeTableHasNoTreeOfKind { kind })
    };

    // 树节点的提示读不出时查的就是上面整片读进来的这份映射条目：查映射不再发设备读。
    let central_mapping_locations_of_key = |mapping_key: &[u8]| {
        Ok(central_mapping_entries
            .iter()
            .find(|entry| entry.key == mapping_key)
            .map(|entry| entry.locations))
    };
    let mut tree_node_stale_location_hint_hops: usize = 0;
    let extent_entry = entry_of_kind(TREE_KIND_EXTENT)?;
    let extent_root = read_mapped_tree_root(
        reader,
        extent_entry.tree,
        EXTENT_KEY_WIDTH_IN_BYTES,
        &extent_entry.root,
        root,
        filesystem_identifier_in_unit_headers,
        &central_mapping_locations_of_key,
        &mut tree_node_stale_location_hint_hops,
    )?;
    let extent_tree_node_reads = 1;
    if extent_root.level != EXTENT_TREE_ROOT_LEVEL {
        return Err(OpenPoolForReadFailure::TreeLevelUnexpected {
            tree: extent_entry.tree,
            expected_level: EXTENT_TREE_ROOT_LEVEL,
            found_level: extent_root.level,
        });
    }
    let mut extent_leaf_records = Vec::with_capacity(extent_root.entries.len());
    for record_bytes in &extent_root.entries {
        let (key, data_unit_pointer) =
            parse_extent_record(record_bytes).ok_or(OpenPoolForReadFailure::RecordMalformed {
                what: "extent 叶记录",
            })?;
        extent_leaf_records.push(ExtentLeafRecordInMountState {
            key,
            data_unit_pointer,
        });
    }

    let inode_entry = entry_of_kind(TREE_KIND_INODE)?;
    let inode_root = read_mapped_tree_root(
        reader,
        inode_entry.tree,
        INODE_KEY_WIDTH_IN_BYTES,
        &inode_entry.root,
        root,
        filesystem_identifier_in_unit_headers,
        &central_mapping_locations_of_key,
        &mut tree_node_stale_location_hint_hops,
    )?;
    if inode_root.level != INODE_TREE_ROOT_LEVEL {
        return Err(OpenPoolForReadFailure::TreeLevelUnexpected {
            tree: inode_entry.tree,
            expected_level: INODE_TREE_ROOT_LEVEL,
            found_level: inode_root.level,
        });
    }
    let mut inode_records: Vec<InodeRecord> = Vec::new();
    for entry_bytes in &inode_root.entries {
        let (_separator_key, identity_in_the_entry, child) = parse_inode_internal_entry(
            entry_bytes,
        )
        .ok_or(OpenPoolForReadFailure::RecordMalformed {
            what: "inode 树内部条目",
        })?;
        let container_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
            reader,
            &child,
            MappedTreeNodeClass::PackedRecordUnit,
            &central_mapping_locations_of_key,
            &mut tree_node_stale_location_hint_hops,
        )?;
        let container = parse_packed_unit(&container_bytes).map_err(|error| {
            OpenPoolForReadFailure::UnitMalformed {
                what: "inode 叶容器",
                error,
            }
        })?;
        if container.identity != identity_in_the_entry {
            return Err(
                OpenPoolForReadFailure::InodeLeafContainerDisagreesWithTheInternalEntry {
                    detail: "叶头的四元组身份与内部条目里的身份引用不符",
                },
            );
        }
        if identity_in_the_entry.record_type != PACKED_TYPE_INODE {
            return Err(
                OpenPoolForReadFailure::InodeLeafContainerDisagreesWithTheInternalEntry {
                    detail: "内部条目的打包记录类型段不是 2",
                },
            );
        }
        if u64::try_from(container.record_width).expect("记录宽") != INODE_RECORD_BYTES {
            return Err(
                OpenPoolForReadFailure::InodeLeafContainerDisagreesWithTheInternalEntry {
                    detail: "叶容器自述的记录宽不是 140",
                },
            );
        }
        if container.filesystem_identifier != filesystem_identifier_in_unit_headers {
            return Err(
                OpenPoolForReadFailure::InodeLeafContainerDisagreesWithTheInternalEntry {
                    detail: "叶容器的 fsid 与根不符",
                },
            );
        }
        for record_bytes in &container.records {
            inode_records.push(
                InodeRecord::parse(record_bytes)
                    .ok_or(OpenPoolForReadFailure::InodeRecordMalformed)?,
            );
        }
    }

    Ok(MountedPoolForRead {
        root: *root,
        filesystem_identifier_in_unit_headers,
        extent_tree: extent_entry.tree,
        extent_tree_reads_at_open: ExtentTreeReadsAtOpen {
            node_reads: extent_tree_node_reads,
            height: u64::from(extent_root.level) + 1,
            // 根的层级核过是 0（根兼叶）：读到的叶就是根这一片。
            leaves: 1,
        },
        extent_leaf_records,
        inode_records,
        central_mapping_entries,
        observation_since_open: Cell::new(ReadPathObservation::default()),
        tree_node_stale_location_hint_hops_at_open: u64::try_from(
            tree_node_stale_location_hint_hops,
        )
        .expect("多跳次数不超过这一趟读的树节点数"),
    })
}

/// 只读挂载没做成。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MountReadOnlyFailure {
    /// 择系统配置、择根这一段判红（一条自证过的根都没有也在内）。
    Recovery(RecoveryFailure),
    /// 根择到了，沿它打开挂载态时判红。
    Open(OpenPoolForReadFailure),
}

/// 一次只读挂载：择到的根、施加 journal 前缀之后的根、这次扫环的报告，与打开好的挂载态。
pub struct MountedReadOnly {
    pub mounted: MountedPoolForRead,
    /// 恢复择到的那条根（施加前缀之前）。
    pub chosen_root: RootRecord,
    /// 施加前缀之后的根：挂载态是沿它打开的。
    pub effective_root: RootRecord,
    pub journal: JournalScanReport,
}

/// 只读挂载：择系统配置 → 择根 → 全环扫描 journal → 按前缀口径施加 → 沿施加之后的根打开挂载态。
/// 一个字节都不写（并行线二「这条线不写盘」）。
///
/// **扫环只在这一步发生一次**：之后每一次 [`OpenFileForRead::read_at`] 都不碰 journal 环。
/// 层 0 的每个恢复结果上也能跑这一条——`reader` 换成那个崩溃镜像即可（并行线二验收第 3 条；
/// 枚举那一半归 crash-verifier）。
///
/// # Errors
/// 见 [`MountReadOnlyFailure`]。
pub fn mount_read_only(reader: &dyn PoolReader) -> Result<MountedReadOnly, MountReadOnlyFailure> {
    let system_configuration =
        choose_system_configuration(reader).map_err(MountReadOnlyFailure::Recovery)?;
    let chosen_root = choose_root(reader, &system_configuration)
        .ok_or(MountReadOnlyFailure::Recovery(RecoveryFailure::NoValidRoot))?;
    let records = scan_journal(reader, &system_configuration);
    let (journal, effective_root) = replay_journal(
        reader,
        &chosen_root,
        system_configuration.immutable.sizes.journal_ring_bytes,
        &records,
        true,
        rollback_high_water_of_root(reader, &chosen_root),
    );
    let mounted =
        open_pool_for_read(reader, &effective_root).map_err(MountReadOnlyFailure::Open)?;
    Ok(MountedReadOnly {
        mounted,
        chosen_root,
        effective_root,
        journal,
    })
}

impl MountedPoolForRead {
    /// 这次挂载读的是哪一条根。
    #[must_use]
    pub const fn root(&self) -> RootRecord {
        self.root
    }

    /// 打开之后每次读累起来的观测点：多跳率取它算（D19（块指针的结构与宽度预算） 已定项 5 硬规则 3）。
    #[must_use]
    pub fn observation_since_open(&self) -> ReadPathObservation {
        self.observation_since_open.get()
    }

    /// 打开这一趟里树节点的位置提示读不出、经中央映射回退的次数（D19（块指针的结构与宽度预算） 已定项 5 硬规则 3）。
    #[must_use]
    pub const fn tree_node_stale_location_hint_hops_at_open(&self) -> u64 {
        self.tree_node_stale_location_hint_hops_at_open
    }

    /// 打开时沿 extent 树读了几个节点、树高、叶片数：之后按偏移读不再碰 extent 树，顺序读 M 个单元的 extent 节点读取次数就是它。
    #[must_use]
    pub const fn extent_tree_reads_at_open(&self) -> ExtentTreeReadsAtOpen {
        self.extent_tree_reads_at_open
    }

    /// 挂载态里这一版的 inode 记录（按叶序、叶内次序）。
    #[must_use]
    pub fn inode_records(&self) -> &[InodeRecord] {
        &self.inode_records
    }

    /// 挂载态里这一版的 extent 叶记录（按盘上次序）。
    #[must_use]
    pub fn extent_leaf_records(&self) -> &[ExtentLeafRecordInMountState] {
        &self.extent_leaf_records
    }

    /// 打开一个文件：从挂载态里挑出它的 inode 记录与它的数据单元记录，当场核位次定位的前提。
    ///
    /// # Errors
    /// 见 [`OpenFileFailure`]：没有这个 inode / 记录不是按 key 升序 / 记录条数与文件大小算出的单元数对不上。
    pub fn open_file(&self, inode: InodeNumber) -> Result<OpenFileForRead<'_>, OpenFileFailure> {
        let inode_record = self
            .inode_records
            .iter()
            .find(|record| record.inode == inode.0)
            .copied()
            .ok_or(OpenFileFailure::NoSuchInode { inode })?;
        let data_unit_records: Vec<ExtentLeafRecordInMountState> = self
            .extent_leaf_records
            .iter()
            .filter(|record| record.inode_number() == inode)
            .copied()
            .collect();
        // 第 i 条记录的 offset 段要等于第 i 个单元第一个字节的文件偏移（D8（核心索引结构） 已定项 3）：
        // 于是「第 i 条记录 = 文件第 i 个数据单元」由 key 本身担保，读的时候按单元序号取记录。
        let payload_capacity = payload_capacity_in_bytes();
        for (position, record) in data_unit_records.iter().enumerate() {
            let unit_index_in_file =
                DataUnitIndexInFile(u64::try_from(position).expect("单元序号"));
            let expected_file_offset = unit_index_in_file.first_file_byte(payload_capacity);
            if record.file_offset_in_bytes() != expected_file_offset {
                return Err(
                    OpenFileFailure::ExtentRecordKeyIsNotTheFileOffsetOfItsUnit {
                        inode,
                        unit_index_in_file,
                        key_file_offset: record.file_offset_in_bytes(),
                        expected_file_offset,
                    },
                );
            }
        }
        // 文件没有洞、尾巴上也没有多出来的记录：读侧与写侧用同一条除法算单元数，
        // 不在这里另写一个（`write_request_split::data_unit_count_of_a_sequential_write`）。
        let data_units_implied_by_the_file_size =
            data_unit_count_of_a_sequential_write(inode_record.size);
        let extent_records = u64::try_from(data_unit_records.len()).expect("记录条数");
        if extent_records != data_units_implied_by_the_file_size {
            return Err(OpenFileFailure::ExtentRecordCountDoesNotMatchTheFileSize {
                inode,
                extent_records,
                data_units_implied_by_the_file_size,
                file_size_in_bytes: inode_record.size,
            });
        }
        Ok(OpenFileForRead {
            mount: self,
            inode_record,
            data_unit_records,
        })
    }

    /// 中央映射里这个 key 的两条位置条目（D19（块指针的结构与宽度预算） 已定项 5：解引用的唯一入口）。
    fn central_mapping_lookup(&self, key: &[u8]) -> Option<[LocationEntry; 2]> {
        self.central_mapping_entries
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.locations)
    }

    fn add_to_observation_since_open(&self, observation: ReadPathObservation) {
        self.observation_since_open
            .set(self.observation_since_open.get().plus(observation));
    }
}

/// 挂载态里打开着的一个文件：它的 inode 记录与它按位次排好的数据单元记录。派生态，丢了重开一次即可。
pub struct OpenFileForRead<'mount> {
    mount: &'mount MountedPoolForRead,
    inode_record: InodeRecord,
    /// 第 i 项就是文件第 i 个数据单元（位次定位，前提在 [`MountedPoolForRead::open_file`] 里核过）。
    data_unit_records: Vec<ExtentLeafRecordInMountState>,
}

impl OpenFileForRead<'_> {
    /// 这个文件的 inode 记录。
    #[must_use]
    pub const fn inode_record(&self) -> InodeRecord {
        self.inode_record
    }

    /// 这个文件有几个数据单元。
    #[must_use]
    pub fn data_unit_count(&self) -> u64 {
        u64::try_from(self.data_unit_records.len()).expect("单元数")
    }

    /// 按偏移读 `length_in_bytes` 字节：定位单元、读整单元、验校验和、拷出要的那一段
    /// （跨单元的页读两个单元，D4（校验和位置） 已定项 5）。不走 journal、不重走冷恢复。
    ///
    /// # Errors
    /// 见 [`FileReadFailure`]：越界 / 校验和错 / 映射里没有 / 读不到 / 解不开 / 自描述与查找路径不符。
    ///
    /// # Panics
    /// 拷出来的字节数不等于请求的长度——那是本函数自己的不变量（每个单元的声明长度已经按文件大小核过），
    /// 不是盘上的输入能触发的。
    pub fn read_at(
        &self,
        reader: &dyn PoolReader,
        offset: FileOffsetInBytes,
        length_in_bytes: u64,
    ) -> Result<FileReadOutput, FileReadFailure> {
        let file_size_in_bytes = self.inode_record.size;
        let end_exclusive =
            offset
                .0
                .checked_add(length_in_bytes)
                .ok_or(FileReadFailure::RangeBeyondEndOfFile {
                    file_size_in_bytes,
                    offset,
                    length_in_bytes,
                })?;
        if end_exclusive > file_size_in_bytes {
            return Err(FileReadFailure::RangeBeyondEndOfFile {
                file_size_in_bytes,
                offset,
                length_in_bytes,
            });
        }
        if length_in_bytes == 0 {
            return Ok(FileReadOutput {
                bytes: Vec::new(),
                observation: ReadPathObservation::default(),
            });
        }
        let payload_capacity = payload_capacity_in_bytes();
        let span = data_unit_span_covering(offset, FileOffsetInBytes(end_exclusive - 1));
        let mut observation = ReadPathObservation::default();
        let mut bytes =
            Vec::with_capacity(usize::try_from(length_in_bytes).expect("一次读的长度装得进 usize"));
        let mut anchor_offset_of_the_previous_unit: Option<u64> = None;
        // 迭代次数的上界 = 这次读跨的单元数；循环体跨轮携带的只有上一个单元的锚点偏移与已拷出的字节。
        for unit_number in span.first.0..=span.last_inclusive.0 {
            let unit_index_in_file = DataUnitIndexInFile(unit_number);
            let record = *self
                .data_unit_records
                .get(usize::try_from(unit_number).expect("单元序号装得进 usize"))
                .expect(
                    "单元序号由 (文件大小 − 1) ÷ 净荷容量界住，而记录条数在 open_file 里核过等于单元数",
                );
            let unit_bytes =
                self.dereference_data_unit(reader, record, unit_index_in_file, &mut observation)?;
            let header = parse_data_unit(&unit_bytes).map_err(|error| {
                FileReadFailure::DataUnitMalformed {
                    unit_index_in_file,
                    error,
                }
            })?;
            self.check_data_unit_against_the_lookup_path(
                &header,
                record,
                unit_index_in_file,
                anchor_offset_of_the_previous_unit,
            )?;
            anchor_offset_of_the_previous_unit = Some(header.identity.anchor_offset);
            let payload =
                data_unit_payload(&unit_bytes, header.declared_length).map_err(|_error| {
                    FileReadFailure::DataUnitDisagreesWithTheLookupPath {
                        unit_index_in_file,
                        invariant: "I-2.3",
                        detail: "声明长度之后的补齐字节非零",
                    }
                })?;
            let unit_starts_at_file_byte = unit_number * payload_capacity;
            let copy_from = offset.0.max(unit_starts_at_file_byte) - unit_starts_at_file_byte;
            let copy_until = end_exclusive
                .min(unit_starts_at_file_byte + u64::from(header.declared_length))
                - unit_starts_at_file_byte;
            bytes.extend_from_slice(
                &payload[usize::try_from(copy_from).expect("单元内偏移")
                    ..usize::try_from(copy_until).expect("单元内偏移")],
            );
            observation.data_units_read += 1;
        }
        assert_eq!(
            u64::try_from(bytes.len()).expect("读出来的字节数"),
            length_in_bytes,
            "拷出来的字节数要等于请求的长度：各单元的声明长度在上面逐个核过"
        );
        self.mount.add_to_observation_since_open(observation);
        Ok(FileReadOutput { bytes, observation })
    }

    /// 解引用一个数据单元：先按父指针里的位置提示逐条试（整单元 CRC-32C 等于条目里的校验和才算读到），
    /// 两条都不算数时查中央映射再试一遍，并把这一次记进多跳计数（D19（块指针的结构与宽度预算） 已定项 5 与硬规则 3）。
    fn dereference_data_unit(
        &self,
        reader: &dyn PoolReader,
        record: ExtentLeafRecordInMountState,
        unit_index_in_file: DataUnitIndexInFile,
        observation: &mut ReadPathObservation,
    ) -> Result<Vec<u8>, FileReadFailure> {
        let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
        let pointer = record.data_unit_pointer;
        let hinted_slot = pointer.locations[0].slot;
        observation.data_unit_dereferences += 1;
        let by_hint =
            read_unit_by_location_entries(reader, &pointer.locations, data_unit_bytes, observation);
        let hint_outcome = match by_hint {
            UnitReadOutcome::Read(unit_bytes) => return Ok(unit_bytes),
            UnitReadOutcome::EveryLocationReadButTheChecksumMismatched => {
                UnitReadFailedBecause::ChecksumMismatch
            }
            UnitReadOutcome::NoLocationReturnedBytes => UnitReadFailedBecause::NoBytes,
        };
        // 提示不算数 ⇒ 走中央映射那条路，多跳一次。
        observation.stale_location_hint_hops += 1;
        let mapping_key = mapping_key_for_data(pointer.head, pointer.write_order);
        let mapped_locations = self.mount.central_mapping_lookup(&mapping_key).ok_or(
            FileReadFailure::CentralMappingMiss {
                unit_index_in_file,
                hinted_slot,
            },
        )?;
        match read_unit_by_location_entries(reader, &mapped_locations, data_unit_bytes, observation)
        {
            UnitReadOutcome::Read(unit_bytes) => Ok(unit_bytes),
            UnitReadOutcome::EveryLocationReadButTheChecksumMismatched => {
                Err(FileReadFailure::DataUnitChecksumMismatchEverywhere {
                    unit_index_in_file,
                    hinted_slot,
                })
            }
            UnitReadOutcome::NoLocationReturnedBytes => match hint_outcome {
                UnitReadFailedBecause::ChecksumMismatch => {
                    Err(FileReadFailure::DataUnitChecksumMismatchEverywhere {
                        unit_index_in_file,
                        hinted_slot,
                    })
                }
                UnitReadFailedBecause::NoBytes => Err(FileReadFailure::DataUnitUnreadable {
                    unit_index_in_file,
                    hinted_slot,
                }),
            },
        }
    }

    /// 单元头的自描述要与走到它的那条查找路径相符：五元组、出生身份、声明长度，以及跨单元那一页上两个锚点的次序。
    fn check_data_unit_against_the_lookup_path(
        &self,
        header: &DataUnitHeader,
        record: ExtentLeafRecordInMountState,
        unit_index_in_file: DataUnitIndexInFile,
        anchor_offset_of_the_previous_unit: Option<u64>,
    ) -> Result<(), FileReadFailure> {
        let disagrees = |invariant: &'static str, detail: &'static str| {
            FileReadFailure::DataUnitDisagreesWithTheLookupPath {
                unit_index_in_file,
                invariant,
                detail,
            }
        };
        if header.identity.tree != self.mount.extent_tree {
            return Err(disagrees("I-1.3", "数据单元头里的出生树不是 extent 树"));
        }
        if header.identity.object != self.inode_record.inode {
            return Err(disagrees("I-1.1", "数据单元头里的对象 ID 不是这个 inode"));
        }
        if header.identity.object_birth != self.inode_record.object_birth {
            return Err(disagrees("I-9.10", "对象出生代与 inode 记录不符"));
        }
        if header.filesystem_identifier != self.mount.filesystem_identifier_in_unit_headers {
            return Err(disagrees("I-1.4", "数据单元的 fsid 与根不符"));
        }
        let pointer = record.data_unit_pointer;
        if header.birth_txg != pointer.head.birth_txg || header.write_order != pointer.write_order {
            return Err(disagrees("I-1.2", "数据单元头与指针的出生身份不符"));
        }
        // 锚点偏移 = 这个单元第一个字节的文件偏移 = extent key 的 offset 段（D8（核心索引结构） 已定项 3；
        // D9（加密） 已定项 6「锚点偏移 = `key.offset − ptr.extent_off`」，第一版 extent 距起点偏移恒 0）。
        if header.identity.anchor_offset != record.file_offset_in_bytes().0 {
            return Err(disagrees(
                "I-1.1",
                "数据单元头里的锚点偏移与 extent key 的 offset 段不符",
            ));
        }
        if let Some(previous_anchor_offset) = anchor_offset_of_the_previous_unit {
            if header.identity.anchor_offset <= previous_anchor_offset {
                return Err(disagrees(
                    "I-1.1",
                    "跨单元的这次读里，后一个单元的锚点偏移不比前一个大",
                ));
            }
        }
        let declared_length_expected = self.payload_length_of_data_unit(unit_index_in_file);
        if u64::from(header.declared_length) != declared_length_expected {
            return Err(disagrees(
                "E142 走读同款",
                "数据单元的声明长度与文件大小按净荷容量算出来的不符",
            ));
        }
        Ok(())
    }

    /// 文件第 `unit_index_in_file` 个数据单元该装多少字节：除最后一个之外恒是净荷容量
    /// （D4（校验和位置） 已定项 5 的除法；与写侧切分同一条口径）。
    fn payload_length_of_data_unit(&self, unit_index_in_file: DataUnitIndexInFile) -> u64 {
        let payload_capacity = payload_capacity_in_bytes();
        let starts_at_file_byte = unit_index_in_file.0 * payload_capacity;
        self.inode_record
            .size
            .saturating_sub(starts_at_file_byte)
            .min(payload_capacity)
    }
}

/// 按一组位置条目读一个单元的结果。三种，`match` 穷举。
enum UnitReadOutcome {
    Read(Vec<u8>),
    /// 有条目读回了字节，而没有一条的整单元校验和对得上：盘上是坏数据。
    EveryLocationReadButTheChecksumMismatched,
    /// 一条都没读回字节：盘不在、越界。
    NoLocationReturnedBytes,
}

/// 一组位置条目没读成的原因（[`UnitReadOutcome`] 去掉成功那一支）。
enum UnitReadFailedBecause {
    ChecksumMismatch,
    NoBytes,
}

/// 按两条位置条目逐条试着读一个单元：整单元 CRC-32C 等于条目里那 4 字节校验和才算读到
/// （D19（块指针的结构与宽度预算） 已定项 4 的密文校验和段，加密关时是整单元 CRC-32C）。
/// 每试一条就记一次块层读——请求已经发出去了，读不回字节也算。
fn read_unit_by_location_entries(
    reader: &dyn PoolReader,
    locations: &[LocationEntry; 2],
    unit_bytes: usize,
    observation: &mut ReadPathObservation,
) -> UnitReadOutcome {
    let mut some_location_returned_bytes = false;
    for location in locations {
        observation.device_reads_issued += 1;
        let Some(unit_bytes_read) = reader.read(
            location.device,
            location.slot.to_device_offset(),
            unit_bytes,
        ) else {
            continue;
        };
        some_location_returned_bytes = true;
        if crc32_castagnoli(&unit_bytes_read) == location.unit_checksum {
            return UnitReadOutcome::Read(unit_bytes_read);
        }
    }
    if some_location_returned_bytes {
        UnitReadOutcome::EveryLocationReadButTheChecksumMismatched
    } else {
        UnitReadOutcome::NoLocationReturnedBytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// D4（校验和位置） 已定项 5 的那一成多：净荷 32634 不是 4096 的整数倍 ⇒ 4 KiB 的页里有约 12.5% 跨两个单元。
    /// 取样点取满一个完整周期：32634 与 4096 的最小公倍数正好是 2048 个单元 = 16317 页，一个页边界与单元边界
    /// 重合的都没有（gcd(32634, 4096) = 2，16317 是奇数）⇒ 2047 个单元内边界各造出一页跨单元的。
    #[test]
    fn about_one_in_eight_four_kibibyte_pages_spans_two_data_units() {
        let page_bytes = 4096u64;
        let payload_capacity = payload_capacity_in_bytes();
        assert_eq!(payload_capacity, 32634, "净荷 32768 − 头 105 − 预留 29");
        let pages_in_the_period = payload_capacity * 2048 / page_bytes;
        let mut spanning = 0u64;
        for page in 0..pages_in_the_period {
            let first_byte = FileOffsetInBytes(page * page_bytes);
            let last_byte = FileOffsetInBytes(page * page_bytes + page_bytes - 1);
            if data_unit_span_covering(first_byte, last_byte).spans_more_than_one_data_unit() {
                spanning += 1;
            }
        }
        assert_eq!(
            pages_in_the_period, 16317,
            "一个周期 2048 个单元 = 16317 页"
        );
        assert_eq!(spanning, 2047, "2047 个单元内边界各造出一页跨单元的");
        assert_eq!(
            spanning * 1000 / pages_in_the_period,
            125,
            "跨单元的页占千分之 125（≈ 4096 ÷ 32634），实际 {spanning} / {pages_in_the_period}"
        );
    }

    /// 一个字节的读只碰一个单元，边界上那个字节归后一个单元（净荷容量是第一个单元的末字节之后）。
    #[test]
    fn a_single_byte_read_covers_exactly_one_unit_and_the_boundary_byte_belongs_to_the_next_unit() {
        let single_byte = |file_byte: u64| {
            data_unit_span_covering(FileOffsetInBytes(file_byte), FileOffsetInBytes(file_byte))
        };
        assert_eq!(single_byte(32_633).first, DataUnitIndexInFile(0));
        assert_eq!(single_byte(32_633).last_inclusive, DataUnitIndexInFile(0));
        assert_eq!(single_byte(32_634).first, DataUnitIndexInFile(1));
        assert!(!single_byte(32_634).spans_more_than_one_data_unit());
    }

    /// 单元边界两侧那几页：页 7 跨 0 与 1，页 8 整个落在单元 1 里。
    #[test]
    fn the_page_holding_the_first_unit_boundary_spans_two_units_and_the_next_page_does_not() {
        let page = |index: u64| {
            data_unit_span_covering(
                FileOffsetInBytes(index * 4096),
                FileOffsetInBytes(index * 4096 + 4095),
            )
        };
        assert!(
            page(7).spans_more_than_one_data_unit(),
            "32634 落在第 7 页里"
        );
        assert_eq!(page(7).first, DataUnitIndexInFile(0));
        assert_eq!(page(7).last_inclusive, DataUnitIndexInFile(1));
        assert!(!page(8).spans_more_than_one_data_unit());
        assert_eq!(page(8).first, DataUnitIndexInFile(1));
    }

    /// 多跳率按百万分之几报，一次解引用都没有时报 0。
    #[test]
    fn the_stale_hint_hop_rate_is_reported_in_parts_per_million_and_is_zero_without_dereferences() {
        assert_eq!(
            ReadPathObservation::default().stale_location_hint_hops_per_million_dereferences(),
            0
        );
        let observation = ReadPathObservation {
            data_units_read: 8,
            data_unit_dereferences: 8,
            stale_location_hint_hops: 1,
            device_reads_issued: 9,
        };
        assert_eq!(
            observation.stale_location_hint_hops_per_million_dereferences(),
            125_000
        );
    }
}
