//! 区域扫描里的单元头解析与映射反查（D26（后台整理与放置回收） 已定项 13 第 1 条点名的三段里的「乙」）：两份独立写的 WGSL 核
//! （第 8 条，不共用核函数）对同一批记录各算一次，逐条交回这条记录分进哪一类。这里是两份核共用的缓冲布局、打包、回读解码与 CPU 参照，
//! 都是纯函数；起 GPU 的在 `background_gpu`。
//!
//! 放上 GPU 的是核心层区域扫描（`singlefs_core::region_scan::scan_region_over_the_identity_index`）里逐条记录、彼此不相干的那一部分：
//! 一条记录在钉住、隔离两判之后的分类——读到的单元头按核心层三个 parse 的判法解（magic、版本、类标签、flags、头校验和、载荷校验和、
//! 预留位、结构字段）、反查出映射 key，再查中央映射里有哪几条条目的位置项指着这个落点、这把 key 在不在映射里，分成可搬、已死、不一致。
//! 钉住、隔离两判（按槽号查两张集合，决定读不读这个单元）、读盘、「跨 R 边界就整次拒」「两条可搬记录同一把 key 就整次拒」两条整次拒法
//! 都是整个 R 的事，留在 CPU 上：交进来的每条记录都是区域扫描会去读的那一种（R 那块盘上、未释放、起点在 R 里、不跨 R 的边界、
//! 不钉住也不隔离），读的结局随记录一起交（`None` 是读不到）。
//!
//! CPU 参照（[`cpu_reference_classification`]）就是核心层那一份：对这一条记录单独跑一次 `scan_region_over_the_identity_index`，
//! 钉住、隔离两张集合给空的（交进来的记录本来就不钉住、不隔离），读单元的闭包交回随记录交来的那份字节。
//!
//! 缓冲布局（两份核按同一个绑定号读写，各有各的输出缓冲；查表那一份只绑 0、1、2、3、6，按索引查的那一份七个都绑）：
//! - 绑定 0，uniform：这一段的记录数、映射条目数、落点索引的项数、一个不用的字（缓冲占 [`REGION_RECORD_PARAMETER_BUFFER_BYTES`]）。
//! - 绑定 1，只读存储：这一段每一片的单元字节首尾相接，末尾补 0 到 4 字节的整数倍（读不到的记录占 0 字节）。
//! - 绑定 2，只读存储：每一片一项描述，[`RECORD_DESCRIPTOR_WORDS`] 个 `u32`：在输入里的起点字节、字节数、跨度、读到没有（1 是读到）、
//!   盘、槽号低 32 位、槽号高 32 位、这一片是哪一种（从单元头起的一片，或单元头一片之后的一片）、单元在这一片之后还有几个字节、
//!   逐条走映射那一份核的寄存器、按索引查那一份核的寄存器（见下面「一个单元切成几片」）。
//! - 绑定 3，只读存储：中央映射每条一项，[`MAPPING_ENTRY_WORDS`] 个 `u32`：key 的字节数、key 按小端装进 [`KEY_WORDS`] 个字（不满的补 0）、
//!   两条位置项各三个字（盘、槽号低 32 位、高 32 位），按条目表的次序。
//! - 绑定 4，只读存储：落点索引，每项 [`LOCATION_ITEM_WORDS`] 个 `u32`（盘、槽号低 32 位、高 32 位、条目在表里的序号），
//!   每条条目每个不同的落点一项，按（盘、槽号、序号）升序。
//! - 绑定 5，只读存储：按 key 排的条目序号，按（key 的字节数、key 的字）升序，相同的按序号。
//! - 绑定 6，读写存储：每一片一项结果，[`REGION_RECORD_OUTCOME_WORDS`] 个 `u32`：结果的类、两个细节字、诞生代号低 32 位与高 32 位、
//!   key 的字节数与 key 的字（只有可搬那一类填）；单元头一片之后的那一片只写结果的类与第一个细节字（这一片算完的寄存器）。
//!   开算之前整块填 [`REGION_RECORD_OUTCOME_NOT_WRITTEN_FILL_WORD`]：核没写到的那一条读回来结果的类认不得，按 GPU 出错交回。
//!
//! 映射条目、落点索引与按 key 排的序号一项都没有时，各缓冲照样放一项全 0 的（wgpu 不许长度为 0 的绑定），条数照参数里的数。
//!
//! 整份映射每段都交一份；它连同一条记录都装不下这张卡此刻剩下的地方时，按落点（盘、槽）的区间把映射切成几片逐片交
//! （[`region_record_plan_on_a_gpu`]，用户 2026-10-28 定「装不下应该自适应的分片」），核不改：一片里是位置项指进这段区间的条目（按条目表的次序），
//! 一条条目的另一条位置项指到区间外时换成指区间里的这一条（两条位置项同一落点，核与核心层都只记一次）。这样一条记录的落点所在的那一片
//! （它的「本片」）里有指着它的全部条目、次序同整份映射，本片的分类就是整份映射下的分类，只差一样：本片里没有条目指着它、也没有条目带它头里那把 key
//! （已死）时，那把 key 可能在别的片里——别的片里没有条目指着它，分出来只会是已死或「key 与位置不符」，有一片是后者就改判后者。
//! 合法两份、各写各的（第 8 条），逐条走映射那一份核的各片按 [`walking_classification_over_the_mapping_slices`]、按索引查的那一份按
//! [`searching_classification_over_the_mapping_slices`]。
//!
//! 一个单元切成几片（用户 2026-10-28 定「这部分不能因为装不下就拒绝，装不下应该自适应的分片，缩小逐次进行」）：最大那条记录连同一个落点上的全部条目
//! 都装不下时，比 [`UNIT_HEAD_PIECE_INPUT_BYTES`] 长的单元都切成头一片与之后的几片（[`region_record_plan_on_a_gpu`]）。核解头、查结构、查映射只读头一片
//! （最宽的头连同其后的预留位不过 625 字节），头一片之后的字节只进载荷 CRC。CRC-32C 的寄存器更新对「寄存器、字节」是线性的：
//! 从寄存器 `r` 起喂一段字节 `T` 的结果，等于从 `r` 起喂 `|T|` 个 0 字节的结果异或从 0 起喂 `T` 的结果。所以先交头一片之后的几片：每一片两份核各自
//! 从上一片算完的寄存器接着算（头一片之后的头一片从 0 起），交回算完的寄存器（结果的类 [`OUTCOME_KIND_REGISTER_AFTER_THE_PIECE`]）；
//! 同一个单元的几片各在前后不同的段里，后一段交之前前一段的寄存器已经读回。最后交头一片（映射照它切片），描述带着单元在头一片之后还有几个字节与
//! 各自那一份核算完的那一个寄存器：核照常从全 1 起喂头一片里载荷的那几个字节，再喂那么多个 0 字节、异或带进来的寄存器，取反就是整个载荷的 CRC，
//! 别的判法与整个单元交时一样（长度按头一片加之后的字节算）。两份核各用各的寄存器、各用各的算法喂 0 字节（查表那一份按字节查表，逐位移那一份逐位移），
//! 不共用核函数（第 8 条）。没切开的单元照旧整个交，描述里之后的字节是 0、寄存器是 0，核算出的与切之前逐位相同。

use std::collections::BTreeSet;
use std::ops::Range;

use singlefs_core::address::{CheckpointTxg, SlotNumber};
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::region_scan::{
    scan_region_over_the_identity_index, AllocationInRegion, CentralMappingEntry,
    CentralMappingIdentityIndex, MappedUnitClass, MovableUnit, RegionScanInconsistency,
    ScannedAllocation, UnitHeaderUnreadable,
};
use singlefs_core::unit::{
    UnitError, UnitStructureDefect, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
};

use crate::checksum_kernels::UnrecognizedVerdictWord;
use crate::memory_limit_and_chunks::{
    chunks_that_fit_on_the_gpu, input_buffer_bytes, pieces_cut_to_fit_on_the_gpu,
    ChunkBoundsOfAGpu, ChunkFootprint, PieceOfAnEntry, SmallestPieceLargerThanTheRoom,
};
use crate::two_kernel_reconciliation::{
    ConsecutiveUncertaintyCounts, ReconciledBatch, TwoKernelOutcomesOfAnEntry,
};

/// 按条目表的次序逐条看映射、按字节查表算校验和的那一份核。
pub const WALKING_KERNEL_SOURCE: &str =
    include_str!("region_record_kernel_walking_the_mapping.wgsl");
pub const WALKING_KERNEL_ENTRY_POINT: &str = "classify_records_walking_the_mapping_in_table_order";
/// 在两张排好序的索引上二分查映射、逐位移算校验和的那一份核。
pub const SEARCHING_KERNEL_SOURCE: &str =
    include_str!("region_record_kernel_searching_sorted_indexes.wgsl");
pub const SEARCHING_KERNEL_ENTRY_POINT: &str =
    "classify_records_searching_the_sorted_mapping_indexes";

/// 两份核的工作组都是 64 个调用（两份 WGSL 里的 `@workgroup_size(64)`），每条记录一个调用。
pub const REGION_RECORD_INVOCATIONS_PER_WORKGROUP: u32 = 64;
/// 参数缓冲：四个 `u32`。
pub const REGION_RECORD_PARAMETER_BUFFER_BYTES: u64 = 16;
pub const RECORD_DESCRIPTOR_WORDS: usize = 11;
pub const RECORD_DESCRIPTOR_BYTES: u64 = 44;
/// 切开的单元头一片的输入字节数：两份核在头一片里读到的最远那个字节是码 2 索引节点最宽的头（86 + 2 × 255 字节，key 宽一个字节最大 255）
/// 连同其后 29 字节预留位，共 625 字节，补到整字是 628。头一片之后的字节只进载荷 CRC，随便切。
pub const UNIT_HEAD_PIECE_INPUT_BYTES: u64 = 628;
/// 映射 key 在核里占几个字：key 不超过这么多字的字节（[`KEY_BYTES_THE_KERNELS_HOLD`]），长了的这一批交不成。
pub const KEY_WORDS: usize = 7;
pub const KEY_BYTES_THE_KERNELS_HOLD: usize = 4 * KEY_WORDS;
pub const MAPPING_ENTRY_WORDS: usize = 1 + KEY_WORDS + 6;
pub const LOCATION_ITEM_WORDS: usize = 4;
pub const REGION_RECORD_OUTCOME_WORDS: usize = 6 + KEY_WORDS;
pub const REGION_RECORD_OUTCOME_BYTES: u64 = 52;
/// 结果缓冲开算之前的填充字：结果的类只会被核写成认得的那几个。
pub const REGION_RECORD_OUTCOME_NOT_WRITTEN_FILL_WORD: u32 = 0xFFFF_FFFF;
/// 两份核各一个结果缓冲与一个回读暂存缓冲。
const REGION_RECORD_OUTCOME_AND_READBACK_BUFFERS: u64 = 4;

/// 结果的类（两份 WGSL 里各写一份同值的常量）。
const OUTCOME_KIND_MOVABLE: u32 = 0;
const OUTCOME_KIND_DEAD: u32 = 1;
const OUTCOME_KIND_UNIT_CLASS_DISAGREES_WITH_THE_MAPPING_KEY: u32 = 2;
const OUTCOME_KIND_HEADER_KEY_DISAGREES_WITH_THE_MAPPING_LOCATION: u32 = 3;
const OUTCOME_KIND_DEVICE_READ_FAILED: u32 = 4;
const OUTCOME_KIND_SPAN_IS_NOT_A_UNIT_FOOTPRINT: u32 = 5;
const OUTCOME_KIND_REFUSED_BY_THE_PARSER: u32 = 6;
/// 单元头一片之后的那一片交回的那一类：第一个细节字是这一片算完的寄存器（两份 WGSL 里各写一份同值的常量）。
const OUTCOME_KIND_REGISTER_AFTER_THE_PIECE: u32 = 7;

/// 描述里这一片是哪一种（两份 WGSL 里各写一份同值的常量）：从单元头起的一片（整个单元，或切开的单元的头一片），与单元头一片之后的一片。
const PIECE_KIND_UNIT_FROM_ITS_START: u32 = 0;
const PIECE_KIND_AFTER_THE_HEAD: u32 = 1;

/// 解析拒收的码（结果的类是「解析拒收」时第一个细节字），与核心层 `UnitError` 的成员一一对应。
const UNIT_ERROR_TOO_SHORT: u32 = 0;
const UNIT_ERROR_BAD_MAGIC: u32 = 1;
const UNIT_ERROR_UNKNOWN_FORMAT_VERSION: u32 = 2;
const UNIT_ERROR_NON_ZERO_FLAGS: u32 = 3;
/// 第二个细节字：期望的类标签在低 8 位，读到的在次低 8 位。
const UNIT_ERROR_WRONG_UNIT_CLASS: u32 = 4;
const UNIT_ERROR_HEADER_CHECKSUM_MISMATCH: u32 = 5;
const UNIT_ERROR_PAYLOAD_CHECKSUM_MISMATCH: u32 = 6;
const UNIT_ERROR_ENCRYPTION_RESERVED_BYTES_NOT_ZERO: u32 = 7;
/// 第二个细节字是结构缺陷的码（[`unit_structure_defect_of_the_code`]）。
const UNIT_ERROR_STRUCTURE: u32 = 8;

/// 要分类的一条记录：调用方给的条目键（同一条记录下一轮还用同一个键，连续不确定按它数）、记录本身、读到的单元字节（`None` 是读不到）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionRecordToClassify<'unit, EntryKey> {
    pub entry_key: EntryKey,
    pub record: AllocationRecord,
    pub unit_bytes: Option<&'unit [u8]>,
}

/// 这条记录是不是区域扫描会去读的那一种：R 那块盘上、未释放、起点在 R 里、不罩出 R（钉住与隔离由调用方先判过，这里看不到那两张集合）。
#[must_use]
pub fn record_is_one_the_region_scan_reads(
    region: CompactionRegion,
    record: &AllocationRecord,
) -> bool {
    record.device == region.device()
        && !record.is_released
        && region.contains_slot(record.slot)
        && record.slot.0 + u64::from(record.span_slots) <= region.end_slot().0
}

/// 两份核各自的 CRC-32C 寄存器（不取反、不带全 1 初值的那一个；见本模块文件头「一个单元切成几片」）：一片之前带进去的，或单元头一片之后那几字节
/// 从 0 起算完的。两份各算各的、各用各的，不互相顶替（第 8 条：一份核算错只坏它自己那一份）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrcRegistersOfTheTwoKernels {
    pub walking_kernel: u32,
    pub searching_kernel: u32,
}

impl CrcRegistersOfTheTwoKernels {
    /// 还没喂任何字节：两份都是 0。没切开的单元带它，头一片之后的头一片从它起。
    pub const BEFORE_ANY_BYTE: CrcRegistersOfTheTwoKernels = CrcRegistersOfTheTwoKernels {
        walking_kernel: 0,
        searching_kernel: 0,
    };
}

/// 交给两份核的一片。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegionRecordPieceForTheKernels<'unit> {
    /// 一条记录读到的单元从头起的那些字节（读不到是 `None`）：没切开的是整个单元（`bytes_of_the_unit_after_this_piece` 是 0、寄存器是
    /// [`CrcRegistersOfTheTwoKernels::BEFORE_ANY_BYTE`]）；切开的是头一片，单元在它之后还有 `bytes_of_the_unit_after_this_piece` 个字节，
    /// 两份核各自从 0 起把它们算成了 `registers_of_the_bytes_after_this_piece`。两份核照整个单元分类。
    UnitFromItsStart {
        record: AllocationRecord,
        bytes: Option<&'unit [u8]>,
        bytes_of_the_unit_after_this_piece: u32,
        registers_of_the_bytes_after_this_piece: CrcRegistersOfTheTwoKernels,
    },
    /// 切开的单元头一片之后的一片：两份核各自从 `registers_before_this_piece` 里自己那一个起接着算这几个字节，交回算完的寄存器。
    PieceAfterTheHead {
        record: AllocationRecord,
        bytes: &'unit [u8],
        registers_before_this_piece: CrcRegistersOfTheTwoKernels,
    },
}

impl<'unit> RegionRecordPieceForTheKernels<'unit> {
    /// 一条记录整个交（没切开）。
    #[must_use]
    pub fn of_a_whole_record<EntryKey>(record: &RegionRecordToClassify<'unit, EntryKey>) -> Self {
        RegionRecordPieceForTheKernels::UnitFromItsStart {
            record: record.record,
            bytes: record.unit_bytes,
            bytes_of_the_unit_after_this_piece: 0,
            registers_of_the_bytes_after_this_piece: CrcRegistersOfTheTwoKernels::BEFORE_ANY_BYTE,
        }
    }

    /// 这一片是哪条记录的。
    #[must_use]
    pub fn record(&self) -> AllocationRecord {
        match self {
            RegionRecordPieceForTheKernels::UnitFromItsStart { record, .. }
            | RegionRecordPieceForTheKernels::PieceAfterTheHead { record, .. } => *record,
        }
    }

    /// 这一片交给核的字节（读不到是 `None`）。
    #[must_use]
    pub fn bytes(&self) -> Option<&'unit [u8]> {
        match self {
            RegionRecordPieceForTheKernels::UnitFromItsStart { bytes, .. } => *bytes,
            RegionRecordPieceForTheKernels::PieceAfterTheHead { bytes, .. } => Some(bytes),
        }
    }
}

/// 一段记录打包好的输入字节（已补到 4 字节的整数倍）与描述的字。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedRegionRecordChunk {
    pub input_bytes: Vec<u8>,
    pub descriptor_words: Vec<u32>,
}

impl PackedRegionRecordChunk {
    #[must_use]
    pub fn record_count(&self) -> usize {
        self.descriptor_words.len() / RECORD_DESCRIPTOR_WORDS
    }
}

fn low_and_high_words_of_the_slot(slot: SlotNumber) -> [u32; 2] {
    [
        u32::try_from(slot.0 & u64::from(u32::MAX)).expect("取了低 32 位"),
        u32::try_from(slot.0 >> 32).expect("盘上槽号是 6 字节，高 32 位装得进 u32"),
    ]
}

/// 把一段的几片按次序打包：每一片的字节首尾相接（起点是前面那些字节数之和，不按字对齐），读不到的占 0 字节；描述带上这一片是哪一种、
/// 单元在它之后还有几个字节与两份核各自的寄存器（本模块文件头的布局）。
///
/// # Panics
/// 一段的输入超过 `u32` 能表示的字节数：切批按绑定上限切过（外壳把绑定上限压在 `u32` 之内）。
#[must_use]
pub fn packed_region_record_chunk(
    pieces: &[RegionRecordPieceForTheKernels<'_>],
) -> PackedRegionRecordChunk {
    let total_unit_bytes: usize = pieces
        .iter()
        .map(|piece| piece.bytes().map_or(0, <[u8]>::len))
        .sum();
    let padded_bytes =
        input_buffer_bytes(u64::try_from(total_unit_bytes).expect("usize 装得进 u64"));
    let mut input_bytes =
        Vec::with_capacity(usize::try_from(padded_bytes).expect("一段的输入装得进内存"));
    let mut descriptor_words = Vec::with_capacity(pieces.len() * RECORD_DESCRIPTOR_WORDS);
    for piece in pieces {
        let byte_offset = u32::try_from(input_bytes.len())
            .expect("一段的输入不超过 u32 字节：切批按绑定上限切过");
        let piece_bytes = piece.bytes().unwrap_or(&[]);
        let byte_length =
            u32::try_from(piece_bytes.len()).expect("一个单元不超过 u32 字节：切批按绑定上限切过");
        input_bytes.extend_from_slice(piece_bytes);
        let record = piece.record();
        let [slot_low, slot_high] = low_and_high_words_of_the_slot(record.slot);
        let (piece_kind, bytes_of_the_unit_after_the_piece, registers) = match piece {
            RegionRecordPieceForTheKernels::UnitFromItsStart {
                bytes_of_the_unit_after_this_piece,
                registers_of_the_bytes_after_this_piece,
                ..
            } => (
                PIECE_KIND_UNIT_FROM_ITS_START,
                *bytes_of_the_unit_after_this_piece,
                *registers_of_the_bytes_after_this_piece,
            ),
            RegionRecordPieceForTheKernels::PieceAfterTheHead {
                registers_before_this_piece,
                ..
            } => (PIECE_KIND_AFTER_THE_HEAD, 0, *registers_before_this_piece),
        };
        descriptor_words.extend_from_slice(&[
            byte_offset,
            byte_length,
            u32::from(record.span_slots),
            u32::from(piece.bytes().is_some()),
            record.device.0,
            slot_low,
            slot_high,
            piece_kind,
            bytes_of_the_unit_after_the_piece,
            registers.walking_kernel,
            registers.searching_kernel,
        ]);
    }
    input_bytes.resize(
        usize::try_from(padded_bytes).expect("一段的输入装得进内存"),
        0,
    );
    PackedRegionRecordChunk {
        input_bytes,
        descriptor_words,
    }
}

/// 映射里有一条 key 长过核装得下的字节数（第一版不支持：核心层写的 key 都是 27 字节，长的只有坏盘读来的）。这一批一张卡都不交。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MappingKeyLongerThanTheKernelsHold {
    pub entry_position: usize,
    pub key_bytes: usize,
}

/// 交给两份核的映射：条目表、落点索引、按 key 排的序号（见本模块文件头的布局）。一份映射建一次，同一批的各段共用。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MappingTablesForTheKernels {
    /// 条目表的字；没有条目时是一项全 0 的。
    pub entry_words: Vec<u32>,
    /// 落点索引的字；没有项时是一项全 0 的。
    pub location_item_words: Vec<u32>,
    /// 按 key 排的条目序号；没有条目时是一个 0。
    pub key_sorted_entry_positions: Vec<u32>,
    pub entry_count: u32,
    pub location_item_count: u32,
}

impl MappingTablesForTheKernels {
    /// 走一遍条目表建三样。
    ///
    /// # Errors
    /// 有一条 key 长过 [`KEY_BYTES_THE_KERNELS_HOLD`]（[`MappingKeyLongerThanTheKernelsHold`]）。
    ///
    /// # Panics
    /// 条目数或落点项数装不进 `u32`：一版映射的条目数远小于此。
    pub fn of_the_mapping_entries(
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Self, MappingKeyLongerThanTheKernelsHold> {
        let mut entry_words = Vec::with_capacity(mapping_entries.len() * MAPPING_ENTRY_WORDS);
        let mut key_words_by_entry: Vec<(u32, [u32; KEY_WORDS])> =
            Vec::with_capacity(mapping_entries.len());
        let mut location_items: Vec<(u32, u64, u32)> =
            Vec::with_capacity(2 * mapping_entries.len());
        for (entry_position, entry) in mapping_entries.iter().enumerate() {
            if entry.key.len() > KEY_BYTES_THE_KERNELS_HOLD {
                return Err(MappingKeyLongerThanTheKernelsHold {
                    entry_position,
                    key_bytes: entry.key.len(),
                });
            }
            let mut key_bytes = [0_u8; KEY_BYTES_THE_KERNELS_HOLD];
            key_bytes[..entry.key.len()].copy_from_slice(&entry.key);
            let (key_word_bytes, _no_leftover) = key_bytes.as_chunks::<4>();
            let key_words: [u32; KEY_WORDS] = std::array::from_fn(|word_position| {
                u32::from_le_bytes(key_word_bytes[word_position])
            });
            let key_length = u32::try_from(entry.key.len()).expect("不超过 28");
            entry_words.push(key_length);
            entry_words.extend_from_slice(&key_words);
            for location in &entry.locations {
                let [slot_low, slot_high] = low_and_high_words_of_the_slot(location.slot);
                entry_words.extend_from_slice(&[location.device.0, slot_low, slot_high]);
            }
            key_words_by_entry.push((key_length, key_words));
            let entry_position_word =
                u32::try_from(entry_position).expect("一版映射的条目数装得进 u32");
            let [first_location, second_location] = &entry.locations;
            location_items.push((
                first_location.device.0,
                first_location.slot.0,
                entry_position_word,
            ));
            // 两条位置项指同一落点时只记一项：同一条条目在一个落点上只出现一次（核心层按落点的索引也这样记）。
            if (second_location.device, second_location.slot)
                != (first_location.device, first_location.slot)
            {
                location_items.push((
                    second_location.device.0,
                    second_location.slot.0,
                    entry_position_word,
                ));
            }
        }
        location_items.sort_unstable();
        let mut key_sorted_entry_positions: Vec<u32> = (0..mapping_entries.len())
            .map(|entry_position| {
                u32::try_from(entry_position).expect("一版映射的条目数装得进 u32")
            })
            .collect();
        key_sorted_entry_positions.sort_by(|left, right| {
            let left_position = usize::try_from(*left).expect("u32 装得进 usize");
            let right_position = usize::try_from(*right).expect("u32 装得进 usize");
            key_words_by_entry[left_position]
                .cmp(&key_words_by_entry[right_position])
                .then(left.cmp(right))
        });
        let entry_count = u32::try_from(mapping_entries.len()).expect("一版映射的条目数装得进 u32");
        let location_item_count = u32::try_from(location_items.len()).expect("装得进 u32");
        let mut location_item_words: Vec<u32> = location_items
            .iter()
            .flat_map(|(device, slot, entry_position)| {
                let [slot_low, slot_high] = low_and_high_words_of_the_slot(SlotNumber(*slot));
                [*device, slot_low, slot_high, *entry_position]
            })
            .collect();
        if entry_words.is_empty() {
            entry_words.resize(MAPPING_ENTRY_WORDS, 0);
            key_sorted_entry_positions.push(0);
        }
        if location_item_words.is_empty() {
            location_item_words.resize(LOCATION_ITEM_WORDS, 0);
        }
        Ok(Self {
            entry_words,
            location_item_words,
            key_sorted_entry_positions,
            entry_count,
            location_item_count,
        })
    }

    /// 三个缓冲在卡上合占的字节数。
    #[must_use]
    pub fn bytes_on_the_gpu(&self) -> u64 {
        let words = self.entry_words.len()
            + self.location_item_words.len()
            + self.key_sorted_entry_positions.len();
        4 * u64::try_from(words).expect("装得进 u64")
    }

    /// 三个缓冲里最大的那一个的字节数：它也要装得进这张卡的绑定上限。
    #[must_use]
    pub fn largest_buffer_bytes(&self) -> u64 {
        let words = self
            .entry_words
            .len()
            .max(self.location_item_words.len())
            .max(self.key_sorted_entry_positions.len());
        4 * u64::try_from(words).expect("装得进 u64")
    }
}

/// 区域扫描那一段一段在卡上占的：参数缓冲与整份映射（三个缓冲，每段都交一份）；每条一项描述、两份核各一项结果与一项回读暂存。
/// 按条排的最大缓冲是结果那一个。
#[must_use]
pub fn region_record_chunk_footprint(
    mapping_tables: &MappingTablesForTheKernels,
) -> ChunkFootprint {
    region_record_chunk_footprint_of_mapping_table_bytes(mapping_tables.bytes_on_the_gpu())
}

/// 同 [`region_record_chunk_footprint`]，映射的三个缓冲按合占的字节数给。
#[must_use]
pub fn region_record_chunk_footprint_of_mapping_table_bytes(
    mapping_table_bytes: u64,
) -> ChunkFootprint {
    ChunkFootprint {
        bytes_per_chunk_besides_the_entries: REGION_RECORD_PARAMETER_BUFFER_BYTES
            + mapping_table_bytes,
        bytes_per_entry_besides_its_input: RECORD_DESCRIPTOR_BYTES
            + REGION_RECORD_OUTCOME_AND_READBACK_BUFFERS * REGION_RECORD_OUTCOME_BYTES,
        bytes_per_entry_in_the_largest_entry_buffer: REGION_RECORD_OUTCOME_BYTES
            .max(RECORD_DESCRIPTOR_BYTES),
    }
}

/// 映射的一个落点：盘与槽号，按（盘、槽号）排。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MappingLocation {
    pub device: u32,
    pub slot: u64,
}

impl MappingLocation {
    #[must_use]
    pub fn of_the_location_entry(location: &singlefs_core::pointer::LocationEntry) -> Self {
        Self {
            device: location.device.0,
            slot: location.slot.0,
        }
    }

    #[must_use]
    pub fn of_the_record(record: &AllocationRecord) -> Self {
        Self {
            device: record.device.0,
            slot: record.slot.0,
        }
    }
}

/// 映射的一片：落点在 `[first_location, location_past_the_slice)` 里的那一段（`None` 是从头起、到末尾止），位置项指进这一段的条目按条目表的次序，
/// 一条位置项指到段外的换成指段里的那一条（见本模块文件头）；连同交给核的三张表与这张卡上这一片之下记录切成的几段（按这张卡分到的记录里的位置）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MappingSliceWithItsChunks {
    pub first_location: Option<MappingLocation>,
    pub location_past_the_slice: Option<MappingLocation>,
    pub entries: Vec<CentralMappingEntry>,
    pub tables: MappingTablesForTheKernels,
    pub record_chunks: Vec<Range<usize>>,
}

impl MappingSliceWithItsChunks {
    #[must_use]
    pub fn holds_the_location(&self, location: MappingLocation) -> bool {
        self.first_location
            .is_none_or(|first_location| first_location <= location)
            && self
                .location_past_the_slice
                .is_none_or(|location_past_the_slice| location < location_past_the_slice)
    }
}

/// 一片映射交给核时三张表合占的字节数与最大那一张的字节数：`entry_count` 条条目、`location_item_count` 项落点（都按 [`MappingTablesForTheKernels`]
/// 的建法：一项都没有时照样放一项全 0 的）。
fn mapping_table_bytes_of(entry_count: u64, location_item_count: u64) -> (u64, u64) {
    let entry_words = MAPPING_ENTRY_WORDS_AS_U64 * entry_count.max(1);
    let location_item_words = LOCATION_ITEM_WORDS_AS_U64 * location_item_count.max(1);
    let key_sorted_words = entry_count.max(1);
    (
        4 * (entry_words + location_item_words + key_sorted_words),
        4 * entry_words.max(location_item_words).max(key_sorted_words),
    )
}

const MAPPING_ENTRY_WORDS_AS_U64: u64 = 14;
const LOCATION_ITEM_WORDS_AS_U64: u64 = 4;

/// 一条记录交给核的输入字节数（读不到的是 0）。
fn input_bytes_of_the_record<EntryKey>(record: &RegionRecordToClassify<'_, EntryKey>) -> u64 {
    u64::try_from(record.unit_bytes.map_or(0, <[u8]>::len)).expect("装得进 u64")
}

/// 一片映射：位置项指进 `[first_location, location_past_the_slice)` 的条目按条目表的次序，位置项指到段外的换成指段里的那一条。
fn entries_of_the_mapping_slice(
    mapping_entries: &[CentralMappingEntry],
    first_location: Option<MappingLocation>,
    location_past_the_slice: Option<MappingLocation>,
) -> Vec<CentralMappingEntry> {
    let in_the_slice = |location: &singlefs_core::pointer::LocationEntry| {
        let location = MappingLocation::of_the_location_entry(location);
        first_location.is_none_or(|first_location| first_location <= location)
            && location_past_the_slice
                .is_none_or(|location_past_the_slice| location < location_past_the_slice)
    };
    mapping_entries
        .iter()
        .filter_map(|entry| {
            let [first, second] = &entry.locations;
            match (in_the_slice(first), in_the_slice(second)) {
                (true, true) => Some(entry.clone()),
                (true, false) => Some(CentralMappingEntry {
                    key: entry.key.clone(),
                    locations: [*first, *first],
                }),
                (false, true) => Some(CentralMappingEntry {
                    key: entry.key.clone(),
                    locations: [*second, *second],
                }),
                (false, false) => None,
            }
        })
        .collect()
}

/// 区域扫描这一批在一张卡上怎么切（交之前算完，[`region_record_plan_on_a_gpu`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionRecordPlanOnAGpu {
    /// 每条记录从单元头起交给核的那一片的字节数，按记录的次序：没切开的是整个单元（读不到的是 0），切开的是 [`UNIT_HEAD_PIECE_INPUT_BYTES`]。
    pub bytes_from_the_start_of_each_record: Vec<u64>,
    /// 切开的单元头一片之后的那几片，按交的次序切成的几段：片的 `entry_position` 是那条记录在这一批里的位置，`input_bytes` 从单元的头一个字节数起；
    /// 同一个单元的几片按字节的次序、各在前后不同的段里（后一片接着前一片算完的寄存器算）。没有切开的单元时是空。
    pub chunks_of_the_pieces_after_the_heads: Vec<Vec<PieceOfAnEntry>>,
    /// 映射的几片，各片之下记录照每条从单元头起那一片的字节数切成几段。没有记录时是空。
    pub mapping_slices: Vec<MappingSliceWithItsChunks>,
}

/// 一张卡上区域扫描这一批分到它的记录怎么切（交之前算完），先整、后切（用户 2026-10-28 定「装不下应该自适应的分片，缩小逐次进行」）：
/// 每条记录交整个单元、映射整份或按落点切成几片（[`mapping_slices_under_the_input_lengths`]）装得下就这样交；装不下（最大那条记录连同一个落点上的
/// 全部条目都装不下）就把比 [`UNIT_HEAD_PIECE_INPUT_BYTES`] 长的单元都切开：每条从单元头起只交头一片、映射照它再切，头一片之后的字节按
/// `memory_limit_and_chunks::pieces_cut_to_fit_on_the_gpu` 切成单独成一段都装得下的几片（这几段只交一份空映射，核只接着算寄存器；见本模块文件头
/// 「一个单元切成几片」）。没有记录交回空的切法。
///
/// # Errors
/// 切开之后头一片连同一个落点上的全部条目、或头一片之后最小的一片仍装不下（[`SmallestPieceLargerThanTheRoom`]，`entry_position` 是那一条在 `records` 里的位置）；
/// 没有比头一片长的单元可切时，照整个单元那一步的报。
///
/// # Panics
/// `mapping_tables` 不是 `mapping_entries` 建出来的（片从整份映射里取，key 的长短在建整份那三张表时判过）。
pub fn region_record_plan_on_a_gpu<EntryKey>(
    records: &[RegionRecordToClassify<'_, EntryKey>],
    mapping_entries: &[CentralMappingEntry],
    mapping_tables: &MappingTablesForTheKernels,
    room: ChunkBoundsOfAGpu,
) -> Result<RegionRecordPlanOnAGpu, SmallestPieceLargerThanTheRoom> {
    let input_lengths: Vec<u64> = records.iter().map(input_bytes_of_the_record).collect();
    let whole_units_not_held = match mapping_slices_under_the_input_lengths(
        &input_lengths,
        mapping_entries,
        mapping_tables,
        room,
    ) {
        Ok(mapping_slices) => {
            return Ok(RegionRecordPlanOnAGpu {
                bytes_from_the_start_of_each_record: input_lengths,
                chunks_of_the_pieces_after_the_heads: Vec::new(),
                mapping_slices,
            });
        }
        Err(whole_units_not_held) => whole_units_not_held,
    };
    let positions_of_the_units_cut: Vec<usize> = input_lengths
        .iter()
        .enumerate()
        .filter(|(_position, input_bytes)| **input_bytes > UNIT_HEAD_PIECE_INPUT_BYTES)
        .map(|(position, _input_bytes)| position)
        .collect();
    if positions_of_the_units_cut.is_empty() {
        return Err(whole_units_not_held);
    }
    let bytes_from_the_start_of_each_record: Vec<u64> = input_lengths
        .iter()
        .map(|input_bytes| (*input_bytes).min(UNIT_HEAD_PIECE_INPUT_BYTES))
        .collect();
    let mapping_slices = mapping_slices_under_the_input_lengths(
        &bytes_from_the_start_of_each_record,
        mapping_entries,
        mapping_tables,
        room,
    )?;
    let bytes_after_the_heads: Vec<u64> = positions_of_the_units_cut
        .iter()
        .map(|position| input_lengths[*position] - UNIT_HEAD_PIECE_INPUT_BYTES)
        .collect();
    let chunks_of_the_pieces_after_the_heads = pieces_cut_to_fit_on_the_gpu(
        &bytes_after_the_heads,
        region_record_chunk_footprint_of_mapping_table_bytes(mapping_table_bytes_of(0, 0).0),
        room,
    )
    .map_err(|smallest_piece| SmallestPieceLargerThanTheRoom {
        entry_position: positions_of_the_units_cut[smallest_piece.entry_position],
        smallest_piece_bytes: smallest_piece.smallest_piece_bytes,
    })?
    .into_iter()
    .map(|chunk| {
        chunk
            .into_iter()
            .map(|piece| PieceOfAnEntry {
                entry_position: positions_of_the_units_cut[piece.entry_position],
                input_bytes: UNIT_HEAD_PIECE_INPUT_BYTES + piece.input_bytes.start
                    ..UNIT_HEAD_PIECE_INPUT_BYTES + piece.input_bytes.end,
            })
            .collect()
    })
    .collect();
    Ok(RegionRecordPlanOnAGpu {
        bytes_from_the_start_of_each_record,
        chunks_of_the_pieces_after_the_heads,
        mapping_slices,
    })
}

/// 区域扫描那一段与输入无关、定得下来的最小的一片：切开的单元头一片连同只放一条条目、一个落点的那一片映射，与头一片之后一个字节的一片连同空映射，
/// 取大的。一个落点上不止一条条目的那一片随这一批的映射变，挂着时逐批判（[`region_record_plan_on_a_gpu`]）：一张卡这一批切不出就这一批不分给它、
/// 活照卡的顺序填给别的卡（`crate::gpus_of_this_host::plans_of_a_batch_filling_the_gpus_in_order`）。
#[must_use]
pub fn bytes_of_the_smallest_region_record_piece() -> u64 {
    let head_piece_with_one_entry =
        region_record_chunk_footprint_of_mapping_table_bytes(mapping_table_bytes_of(1, 1).0)
            .bytes_held_by_a_chunk(UNIT_HEAD_PIECE_INPUT_BYTES, 1);
    let piece_after_the_head_of_one_byte =
        region_record_chunk_footprint_of_mapping_table_bytes(mapping_table_bytes_of(0, 0).0)
            .bytes_held_by_a_chunk(1, 1);
    head_piece_with_one_entry.max(piece_after_the_head_of_one_byte)
}

/// 每条记录照 `input_lengths` 交（从单元头起的那一片的字节数）时映射怎么切：整份映射连同记录装得下就是一片整份映射、记录照常切段；装不下就按落点的区间
/// 把映射切成几片，每片照它的三张表切记录（见本模块文件头）。片按落点的次序排，一个落点上的全部条目落在同一片里。`mapping_tables` 是整份映射的三张表。
/// 没有记录交回空。
///
/// 切法：每条条目的每个不同的落点记一项，按（落点、条目序号）排、按落点归组，从前往后贪心装：装进当前这一片之后三张表合占的不超过
/// 「这张卡剩下的地方扣掉参数缓冲与最大那条记录的一条」、最大那张表装得进绑定上限就装，装不下就在这个落点上另起一片。
///
/// # Errors
/// 最大那条记录连同只放一个落点的那一片映射都装不下（[`SmallestPieceLargerThanTheRoom`]，`entry_position` 是那一条的位置）。
///
/// # Panics
/// `mapping_tables` 不是 `mapping_entries` 建出来的。
pub fn mapping_slices_under_the_input_lengths(
    input_lengths: &[u64],
    mapping_entries: &[CentralMappingEntry],
    mapping_tables: &MappingTablesForTheKernels,
    room: ChunkBoundsOfAGpu,
) -> Result<Vec<MappingSliceWithItsChunks>, SmallestPieceLargerThanTheRoom> {
    if input_lengths.is_empty() {
        return Ok(Vec::new());
    }
    if mapping_tables.largest_buffer_bytes() <= room.largest_storage_binding_in_bytes {
        if let Ok(record_chunks) = chunks_that_fit_on_the_gpu(
            input_lengths,
            region_record_chunk_footprint(mapping_tables),
            room,
        ) {
            return Ok(vec![MappingSliceWithItsChunks {
                first_location: None,
                location_past_the_slice: None,
                entries: mapping_entries.to_vec(),
                tables: mapping_tables.clone(),
                record_chunks,
            }]);
        }
    }
    let footprint_without_the_mapping = region_record_chunk_footprint_of_mapping_table_bytes(0);
    let (largest_record_position, largest_record_input_bytes) = input_lengths
        .iter()
        .copied()
        .enumerate()
        .max_by_key(|(_position, input_bytes)| *input_bytes)
        .expect("上面判过有记录");
    let largest_record_piece_bytes =
        footprint_without_the_mapping.bytes_held_by_a_chunk(largest_record_input_bytes, 1);
    let smallest_piece_with = |mapping_table_bytes: u64| SmallestPieceLargerThanTheRoom {
        entry_position: largest_record_position,
        smallest_piece_bytes: largest_record_piece_bytes + mapping_table_bytes,
    };
    let room_for_the_mapping_tables = room.memory_limit.map_or(u64::MAX, |memory_limit| {
        memory_limit.0.saturating_sub(largest_record_piece_bytes)
    });
    let record_fits_the_binding = input_buffer_bytes(largest_record_input_bytes)
        <= room.largest_storage_binding_in_bytes
        && footprint_without_the_mapping.bytes_per_entry_in_the_largest_entry_buffer
            <= room.largest_storage_binding_in_bytes;
    let mut location_items: Vec<(MappingLocation, usize)> =
        Vec::with_capacity(2 * mapping_entries.len());
    for (entry_position, entry) in mapping_entries.iter().enumerate() {
        let [first, second] = &entry.locations;
        let first_location = MappingLocation::of_the_location_entry(first);
        let second_location = MappingLocation::of_the_location_entry(second);
        location_items.push((first_location, entry_position));
        if second_location != first_location {
            location_items.push((second_location, entry_position));
        }
    }
    location_items.sort_unstable();
    // 每一片的起点（头一片从头起）；跨落点的一组项不拆。
    let mut slice_starts: Vec<Option<MappingLocation>> = vec![None];
    let mut slice_of_the_entry: Vec<Option<usize>> = vec![None; mapping_entries.len()];
    let mut entries_in_the_slice: u64 = 0;
    let mut location_items_in_the_slice: u64 = 0;
    let fits = |entry_count: u64, location_item_count: u64| -> bool {
        let (table_bytes, largest_table_bytes) =
            mapping_table_bytes_of(entry_count, location_item_count);
        record_fits_the_binding
            && table_bytes <= room_for_the_mapping_tables
            && largest_table_bytes <= room.largest_storage_binding_in_bytes
    };
    let mut group_start = 0;
    while group_start < location_items.len() {
        let location = location_items[group_start].0;
        let group_end = group_start
            + location_items[group_start..]
                .iter()
                .take_while(|(item_location, _entry)| *item_location == location)
                .count();
        let group = &location_items[group_start..group_end];
        let current_slice = slice_starts.len() - 1;
        let entries_new_to_the_slice = u64::try_from(
            group
                .iter()
                .filter(|(_location, entry_position)| {
                    slice_of_the_entry[*entry_position] != Some(current_slice)
                })
                .count(),
        )
        .expect("装得进 u64");
        let group_items = u64::try_from(group.len()).expect("装得进 u64");
        if !fits(
            entries_in_the_slice + entries_new_to_the_slice,
            location_items_in_the_slice + group_items,
        ) {
            if !fits(group_items, group_items) {
                return Err(smallest_piece_with(
                    mapping_table_bytes_of(group_items, group_items).0,
                ));
            }
            if entries_in_the_slice != 0 {
                slice_starts.push(Some(location));
                entries_in_the_slice = 0;
                location_items_in_the_slice = 0;
            }
        }
        let slice_of_the_group = slice_starts.len() - 1;
        for (_location, entry_position) in group {
            if slice_of_the_entry[*entry_position] != Some(slice_of_the_group) {
                slice_of_the_entry[*entry_position] = Some(slice_of_the_group);
                entries_in_the_slice += 1;
            }
        }
        location_items_in_the_slice += group_items;
        group_start = group_end;
    }
    if !fits(entries_in_the_slice, location_items_in_the_slice) {
        return Err(smallest_piece_with(mapping_table_bytes_of(0, 0).0));
    }
    let mut slices = Vec::with_capacity(slice_starts.len());
    for (slice_position, first_location) in slice_starts.iter().enumerate() {
        let location_past_the_slice = slice_starts.get(slice_position + 1).copied().flatten();
        let entries =
            entries_of_the_mapping_slice(mapping_entries, *first_location, location_past_the_slice);
        let tables = MappingTablesForTheKernels::of_the_mapping_entries(&entries)
            .expect("片里的 key 取自整份映射，整份映射的三张表建得成");
        let record_chunks =
            chunks_that_fit_on_the_gpu(input_lengths, region_record_chunk_footprint(&tables), room)
                .expect("每条记录连同这一片映射单独成一段装得下：片按最大那条记录的一条留了地方");
        slices.push(MappingSliceWithItsChunks {
            first_location: *first_location,
            location_past_the_slice,
            entries,
            tables,
            record_chunks,
        });
    }
    Ok(slices)
}

/// 「key 与位置不符」那一类。
fn header_key_disagrees_with_the_mapping_location() -> AllocationInRegion {
    AllocationInRegion::Inconsistent(
        RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation,
    )
}

/// 逐条走映射那一份核的一条记录在各片映射下的分类合成整份映射下的分类（见本模块文件头）：取本片的；本片是已死而别的片里有一片是
/// 「key 与位置不符」（那把 key 在别的片里）就改判后者。
#[must_use]
pub fn walking_classification_over_the_mapping_slices(
    classification_in_each_slice: &[AllocationInRegion],
    home_slice: usize,
) -> AllocationInRegion {
    let header_key_mapped_in_another_slice =
        classification_in_each_slice
            .iter()
            .enumerate()
            .any(|(slice_position, classification)| {
                slice_position != home_slice
                    && *classification == header_key_disagrees_with_the_mapping_location()
            });
    let home_classification = classification_in_each_slice[home_slice].clone();
    if home_classification == AllocationInRegion::Dead && header_key_mapped_in_another_slice {
        header_key_disagrees_with_the_mapping_location()
    } else {
        home_classification
    }
}

/// 按索引查的那一份核的一条记录在各片映射下的分类合成整份映射下的分类：与 [`walking_classification_over_the_mapping_slices`] 判法相同、各写各的
/// （第 8 条）——从本片的起，依次看别的片，起点是已死、看到一片「key 与位置不符」就换成它。
#[must_use]
pub fn searching_classification_over_the_mapping_slices(
    classification_in_each_slice: &[AllocationInRegion],
    home_slice: usize,
) -> AllocationInRegion {
    let mut merged = classification_in_each_slice[home_slice].clone();
    for (slice_position, classification) in classification_in_each_slice.iter().enumerate() {
        if slice_position == home_slice {
            continue;
        }
        let key_found_elsewhere = matches!(
            classification,
            AllocationInRegion::Inconsistent(
                RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation
            )
        );
        if key_found_elsewhere && matches!(merged, AllocationInRegion::Dead) {
            merged = classification.clone();
        }
    }
    merged
}

/// 两份核交回的一段。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BothKernelOutcomesOfARegionRecordChunk {
    /// 单元头一片之后那几片的一段：每片两份核各自算完的寄存器，按段里的次序。
    RegistersAfterThePieces(Vec<CrcRegistersOfTheTwoKernels>),
    /// 从单元头起的那几片的一段：每条两份的分类（逐条走映射那一份在前），按段里的次序。
    Classifications(Vec<(AllocationInRegion, AllocationInRegion)>),
}

/// 一张卡上这一批分到它的记录照 [`region_record_plan_on_a_gpu`] 的切法逐段交两份核（`both_kernel_outcomes_of_a_chunk`：一段的几片，与这一段交哪一片映射——
/// 头一片之后那几片的段是 `None`，外壳交空映射、交回寄存器；从单元头起的段是那一片映射、交回分类），再按记录接回：
/// 先按次序交头一片之后的那几段，每片带着同一个单元上一片算完的寄存器（头一片之后的头一片带 [`CrcRegistersOfTheTwoKernels::BEFORE_ANY_BYTE`]），
/// 读回的寄存器记成这个单元到这一片为止的；再逐片映射交从单元头起的那几段，切开的单元带着之后还有几个字节与之后那几片算完的寄存器
/// （见本模块文件头「一个单元切成几片」）。只有一片映射的照核交回的；几片的，每条记录两份各按自己那一种合法合成
/// （[`walking_classification_over_the_mapping_slices`]、[`searching_classification_over_the_mapping_slices`]），本片是它的落点所在的那一片。
/// 交回每条两份的分类，按 `records` 的次序。
///
/// # Errors
/// 某一段交不成：照原样交回，后面的段不交。
///
/// # Panics
/// 切法与 `records` 对不上（条数不等、一片映射之下的几段没有一条不漏地罩住 `records`、切开的单元没读到、片越过单元的末尾、一段里同一个单元出现两片）；
/// 一段交回的条数与它的片数不等，或交回的那一种与这一段的种类不符（外壳照有没有映射片定交回哪一种）；`records` 非空而没有一片映射。
pub fn region_record_classifications_of_the_plan<'unit, EntryKey, SubmissionFailure>(
    records: &[RegionRecordToClassify<'unit, EntryKey>],
    plan: &RegionRecordPlanOnAGpu,
    mut both_kernel_outcomes_of_a_chunk: impl FnMut(
        &[RegionRecordPieceForTheKernels<'unit>],
        Option<&MappingSliceWithItsChunks>,
    ) -> Result<
        BothKernelOutcomesOfARegionRecordChunk,
        SubmissionFailure,
    >,
) -> Result<Vec<(AllocationInRegion, AllocationInRegion)>, SubmissionFailure> {
    if records.is_empty() {
        return Ok(Vec::new());
    }
    assert_eq!(
        plan.bytes_from_the_start_of_each_record.len(),
        records.len(),
        "切法给每条记录一个从单元头起的字节数"
    );
    assert!(!plan.mapping_slices.is_empty(), "有记录就至少一片映射");
    let mut registers_of_the_bytes_after_the_head =
        vec![CrcRegistersOfTheTwoKernels::BEFORE_ANY_BYTE; records.len()];
    for chunk in &plan.chunks_of_the_pieces_after_the_heads {
        let pieces: Vec<RegionRecordPieceForTheKernels<'unit>> = chunk
            .iter()
            .map(|piece| {
                let unit_bytes = records[piece.entry_position]
                    .unit_bytes
                    .expect("切开的单元读到了：读不到的那一条交 0 字节，不比头一片长、不切");
                let piece_start =
                    usize::try_from(piece.input_bytes.start).expect("片在单元里，装得进 usize");
                let piece_end =
                    usize::try_from(piece.input_bytes.end).expect("片在单元里，装得进 usize");
                RegionRecordPieceForTheKernels::PieceAfterTheHead {
                    record: records[piece.entry_position].record,
                    bytes: &unit_bytes[piece_start..piece_end],
                    registers_before_this_piece: registers_of_the_bytes_after_the_head
                        [piece.entry_position],
                }
            })
            .collect();
        assert!(
            chunk
                .iter()
                .enumerate()
                .all(
                    |(position_in_the_chunk, piece)| chunk[..position_in_the_chunk]
                        .iter()
                        .all(|earlier| earlier.entry_position != piece.entry_position)
                ),
            "一段里同一个单元至多一片：后一片要接前一片读回的寄存器"
        );
        let BothKernelOutcomesOfARegionRecordChunk::RegistersAfterThePieces(registers_after) =
            both_kernel_outcomes_of_a_chunk(&pieces, None)?
        else {
            panic!("头一片之后那几片的段交回寄存器：外壳照没有映射片交回这一种");
        };
        assert_eq!(
            registers_after.len(),
            pieces.len(),
            "两份核每片各交一个寄存器"
        );
        for (piece, registers) in chunk.iter().zip(registers_after) {
            registers_of_the_bytes_after_the_head[piece.entry_position] = registers;
        }
    }
    let pieces_from_the_start: Vec<RegionRecordPieceForTheKernels<'unit>> = records
        .iter()
        .zip(&plan.bytes_from_the_start_of_each_record)
        .zip(&registers_of_the_bytes_after_the_head)
        .map(|((to_classify, bytes_from_the_start), registers)| {
            let bytes_after_this_piece = input_bytes_of_the_record(to_classify)
                .checked_sub(*bytes_from_the_start)
                .expect("从单元头起的那一片不比单元长");
            RegionRecordPieceForTheKernels::UnitFromItsStart {
                record: to_classify.record,
                bytes: to_classify.unit_bytes.map(|unit_bytes| {
                    &unit_bytes[..usize::try_from(*bytes_from_the_start)
                        .expect("不比单元长，装得进 usize")]
                }),
                bytes_of_the_unit_after_this_piece: u32::try_from(bytes_after_this_piece)
                    .expect("一个单元不超过 32 KiB"),
                registers_of_the_bytes_after_this_piece: *registers,
            }
        })
        .collect();
    let mut classifications_in_each_slice: Vec<Vec<(AllocationInRegion, AllocationInRegion)>> =
        Vec::with_capacity(plan.mapping_slices.len());
    for slice in &plan.mapping_slices {
        let mut classifications_in_this_slice = Vec::with_capacity(records.len());
        for record_chunk in &slice.record_chunks {
            assert_eq!(
                record_chunk.start,
                classifications_in_this_slice.len(),
                "一片之下的几段按记录的次序首尾相接"
            );
            let BothKernelOutcomesOfARegionRecordChunk::Classifications(chunk_classifications) =
                both_kernel_outcomes_of_a_chunk(
                    &pieces_from_the_start[record_chunk.clone()],
                    Some(slice),
                )?
            else {
                panic!("从单元头起那几片的段交回分类：外壳照有映射片交回这一种");
            };
            assert_eq!(
                chunk_classifications.len(),
                record_chunk.len(),
                "两份核每条各交一个分类"
            );
            classifications_in_this_slice.extend(chunk_classifications);
        }
        assert_eq!(
            classifications_in_this_slice.len(),
            records.len(),
            "一片之下的几段罩住每一条记录"
        );
        classifications_in_each_slice.push(classifications_in_this_slice);
    }
    if let [only_slice] = classifications_in_each_slice.as_slice() {
        return Ok(only_slice.clone());
    }
    Ok(records
        .iter()
        .enumerate()
        .map(|(record_position, record)| {
            let home_slice = plan
                .mapping_slices
                .iter()
                .position(|slice| {
                    slice.holds_the_location(MappingLocation::of_the_record(&record.record))
                })
                .expect("各片的落点区间首尾相接、罩住全部落点");
            let walking_in_each_slice: Vec<AllocationInRegion> = classifications_in_each_slice
                .iter()
                .map(|in_this_slice| in_this_slice[record_position].0.clone())
                .collect();
            let searching_in_each_slice: Vec<AllocationInRegion> = classifications_in_each_slice
                .iter()
                .map(|in_this_slice| in_this_slice[record_position].1.clone())
                .collect();
            (
                walking_classification_over_the_mapping_slices(&walking_in_each_slice, home_slice),
                searching_classification_over_the_mapping_slices(
                    &searching_in_each_slice,
                    home_slice,
                ),
            )
        })
        .collect())
}

/// 把单元头一片之后那几片的一段读回的结果字节解成每片算完的寄存器（结果的类是 [`OUTCOME_KIND_REGISTER_AFTER_THE_PIECE`]，第一个细节字是寄存器）。
///
/// # Errors
/// 某一片结果的类不是那一类（核没写到的那一片还是填充字）。
///
/// # Panics
/// 字节数不是每条 [`REGION_RECORD_OUTCOME_BYTES`] 的整数倍：回读缓冲按片数建，长度对不上是外壳的 bug。
pub fn decoded_registers_after_the_pieces(
    readback_bytes: &[u8],
) -> Result<Vec<u32>, UnrecognizedVerdictWord> {
    let (outcome_records, leftover_bytes) =
        readback_bytes.as_chunks::<{ 4 * REGION_RECORD_OUTCOME_WORDS }>();
    assert!(
        leftover_bytes.is_empty(),
        "回读的字节数是每条 52 字节的整数倍"
    );
    outcome_records
        .iter()
        .enumerate()
        .map(|(entry_position, outcome_record)| {
            let (word_bytes, _no_leftover) = outcome_record.as_chunks::<4>();
            let [outcome_kind, register_after_the_piece, ..]: [u32; REGION_RECORD_OUTCOME_WORDS] =
                std::array::from_fn(|word_position| u32::from_le_bytes(word_bytes[word_position]));
            if outcome_kind == OUTCOME_KIND_REGISTER_AFTER_THE_PIECE {
                Ok(register_after_the_piece)
            } else {
                Err(UnrecognizedVerdictWord {
                    entry_position,
                    verdict_word: outcome_kind,
                })
            }
        })
        .collect()
}

/// 参数缓冲的字节。
#[must_use]
pub fn region_record_parameter_bytes(
    record_count: u32,
    mapping_tables: &MappingTablesForTheKernels,
) -> Vec<u8> {
    [
        record_count,
        mapping_tables.entry_count,
        mapping_tables.location_item_count,
        0,
    ]
    .iter()
    .flat_map(|word| word.to_le_bytes())
    .collect()
}

/// 按小端把字交给缓冲。
#[must_use]
pub fn little_endian_bytes_of_the_words(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|word| word.to_le_bytes()).collect()
}

/// 结果缓冲开算之前的内容：每个字都是填充字。
#[must_use]
pub fn region_record_outcome_buffer_before_the_kernel_writes(record_count: usize) -> Vec<u8> {
    std::iter::repeat_n(
        REGION_RECORD_OUTCOME_NOT_WRITTEN_FILL_WORD.to_le_bytes(),
        record_count * REGION_RECORD_OUTCOME_WORDS,
    )
    .flatten()
    .collect()
}

fn mapped_unit_class_of_the_tag(
    entry_position: usize,
    tag_word: u32,
) -> Result<MappedUnitClass, UnrecognizedVerdictWord> {
    let unrecognized = UnrecognizedVerdictWord {
        entry_position,
        verdict_word: tag_word,
    };
    match u8::try_from(tag_word) {
        Ok(UNIT_CLASS_DATA) => Ok(MappedUnitClass::Data),
        Ok(UNIT_CLASS_INDEX_NODE) => Ok(MappedUnitClass::IndexNode),
        Ok(UNIT_CLASS_PACKED) => Ok(MappedUnitClass::Packed),
        Ok(_other_tag) => Err(unrecognized),
        Err(_wider_than_a_byte) => Err(unrecognized),
    }
}

/// 结构缺陷的码：按核心层 `UnitStructureDefect` 成员的次序从 0 起。
fn unit_structure_defect_of_the_code(defect_code: u32) -> Option<UnitStructureDefect> {
    match defect_code {
        0 => Some(UnitStructureDefect::IndexNodeDeclaredLengthIsNotEntryCountTimesEntryWidth),
        1 => Some(UnitStructureDefect::IndexNodeEntryWidthZeroWithEntries),
        2 => Some(UnitStructureDefect::IndexNodeEntryWidthBelowTheKeyWidth),
        3 => Some(UnitStructureDefect::IndexNodeEntryAreaPastTheNodeEnd),
        4 => Some(UnitStructureDefect::PackedUnitClassTagCopyIsNotThePackedClass),
        5 => Some(UnitStructureDefect::PackedUnitDeclaredLengthIsNotRecordCountTimesRecordWidth),
        6 => Some(UnitStructureDefect::PackedUnitRecordWidthZeroWithRecords),
        7 => Some(UnitStructureDefect::PackedUnitRecordAreaPastTheUnitEnd),
        8 => Some(UnitStructureDefect::DataUnitClassTagCopyIsNotTheDataClass),
        9 => Some(UnitStructureDefect::DataUnitDeclaredLengthPastTheUnitEnd),
        10 => Some(UnitStructureDefect::DataUnitPaddingBytesNotZero),
        _other_code => None,
    }
}

fn unit_error_of_the_codes(
    entry_position: usize,
    error_code: u32,
    error_detail: u32,
) -> Result<UnitError, UnrecognizedVerdictWord> {
    let unrecognized = |verdict_word| UnrecognizedVerdictWord {
        entry_position,
        verdict_word,
    };
    match error_code {
        UNIT_ERROR_TOO_SHORT => Ok(UnitError::TooShort),
        UNIT_ERROR_BAD_MAGIC => Ok(UnitError::BadMagic),
        UNIT_ERROR_UNKNOWN_FORMAT_VERSION => Ok(UnitError::UnknownFormatVersion),
        UNIT_ERROR_NON_ZERO_FLAGS => Ok(UnitError::NonZeroFlags),
        UNIT_ERROR_WRONG_UNIT_CLASS => {
            let [expected, found, _unused_byte, _another_unused_byte] = error_detail.to_le_bytes();
            Ok(UnitError::WrongUnitClass { expected, found })
        }
        UNIT_ERROR_HEADER_CHECKSUM_MISMATCH => Ok(UnitError::HeaderChecksumMismatch),
        UNIT_ERROR_PAYLOAD_CHECKSUM_MISMATCH => Ok(UnitError::PayloadChecksumMismatch),
        UNIT_ERROR_ENCRYPTION_RESERVED_BYTES_NOT_ZERO => {
            Ok(UnitError::EncryptionReservedBytesNotZero)
        }
        UNIT_ERROR_STRUCTURE => unit_structure_defect_of_the_code(error_detail)
            .map(UnitError::Structure)
            .ok_or_else(|| unrecognized(error_detail)),
        unrecognized_code => Err(unrecognized(unrecognized_code)),
    }
}

fn decoded_region_record_outcome(
    entry_position: usize,
    outcome_words: &[u32; REGION_RECORD_OUTCOME_WORDS],
) -> Result<AllocationInRegion, UnrecognizedVerdictWord> {
    let [outcome_kind, detail_word, second_detail_word, birth_txg_low, birth_txg_high, key_length, key_words @ ..] =
        *outcome_words;
    let unrecognized = |verdict_word| UnrecognizedVerdictWord {
        entry_position,
        verdict_word,
    };
    let header_unreadable = |unreadable| {
        AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(unreadable))
    };
    match outcome_kind {
        OUTCOME_KIND_MOVABLE => {
            let key_bytes = usize::try_from(key_length).expect("u32 装得进 usize");
            if key_bytes > KEY_BYTES_THE_KERNELS_HOLD {
                return Err(unrecognized(key_length));
            }
            let mapping_key: Vec<u8> = key_words
                .iter()
                .flat_map(|word| word.to_le_bytes())
                .take(key_bytes)
                .collect();
            Ok(AllocationInRegion::Movable(MovableUnit {
                mapping_key,
                unit_class: mapped_unit_class_of_the_tag(entry_position, detail_word)?,
                birth_txg: CheckpointTxg(
                    u64::from(birth_txg_low) | (u64::from(birth_txg_high) << 32),
                ),
            }))
        }
        OUTCOME_KIND_DEAD => Ok(AllocationInRegion::Dead),
        OUTCOME_KIND_UNIT_CLASS_DISAGREES_WITH_THE_MAPPING_KEY => {
            Ok(AllocationInRegion::Inconsistent(
                RegionScanInconsistency::UnitClassDisagreesWithTheMappingKey {
                    header_unit_class: mapped_unit_class_of_the_tag(entry_position, detail_word)?,
                    mapping_key_class_tag: u8::try_from(second_detail_word)
                        .map_err(|_| unrecognized(second_detail_word))?,
                },
            ))
        }
        OUTCOME_KIND_HEADER_KEY_DISAGREES_WITH_THE_MAPPING_LOCATION => {
            Ok(AllocationInRegion::Inconsistent(
                RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation,
            ))
        }
        OUTCOME_KIND_DEVICE_READ_FAILED => {
            Ok(header_unreadable(UnitHeaderUnreadable::DeviceReadFailed))
        }
        OUTCOME_KIND_SPAN_IS_NOT_A_UNIT_FOOTPRINT => Ok(header_unreadable(
            UnitHeaderUnreadable::SpanIsNotAUnitFootprint {
                span_slots: u16::try_from(detail_word).map_err(|_| unrecognized(detail_word))?,
            },
        )),
        OUTCOME_KIND_REFUSED_BY_THE_PARSER => {
            Ok(header_unreadable(UnitHeaderUnreadable::RefusedByTheParser(
                unit_error_of_the_codes(entry_position, detail_word, second_detail_word)?,
            )))
        }
        unrecognized_kind => Err(unrecognized(unrecognized_kind)),
    }
}

/// 把读回的结果字节解成每条记录一个分类。
///
/// # Errors
/// 某一条的结果认不得（结果的类、类标签、解析拒收的码、跨度、key 的字节数有一样不是核会写的值；核没写到的那一条还是填充字）。
///
/// # Panics
/// 字节数不是每条 [`REGION_RECORD_OUTCOME_BYTES`] 的整数倍：回读缓冲按条数建，长度对不上是外壳的 bug。
pub fn decoded_region_record_outcomes(
    readback_bytes: &[u8],
) -> Result<Vec<AllocationInRegion>, UnrecognizedVerdictWord> {
    let (outcome_records, leftover_bytes) =
        readback_bytes.as_chunks::<{ 4 * REGION_RECORD_OUTCOME_WORDS }>();
    assert!(
        leftover_bytes.is_empty(),
        "回读的字节数是每条 52 字节的整数倍"
    );
    outcome_records
        .iter()
        .enumerate()
        .map(|(entry_position, outcome_record)| {
            let (word_bytes, _no_leftover) = outcome_record.as_chunks::<4>();
            let outcome_words: [u32; REGION_RECORD_OUTCOME_WORDS] =
                std::array::from_fn(|word_position| u32::from_le_bytes(word_bytes[word_position]));
            decoded_region_record_outcome(entry_position, &outcome_words)
        })
        .collect()
}

/// CPU 那一份实现对一条记录的分类（连续不确定的条目交它判；对拍也拿它当参照）：核心层的区域扫描对这一条记录单独跑一次，
/// 钉住、隔离两张集合给空的，读单元的闭包交回 `unit_bytes`。
///
/// # Panics
/// `record` 不是区域扫描会去读的那一种（[`record_is_one_the_region_scan_reads`]）：调用方只交 R 里的、未释放的、不跨 R 边界的记录。
#[must_use]
pub fn cpu_reference_classification(
    region: CompactionRegion,
    record: AllocationRecord,
    mapping_index: &CentralMappingIdentityIndex<'_>,
    unit_bytes: Option<&[u8]>,
) -> AllocationInRegion {
    assert!(
        record_is_one_the_region_scan_reads(region, &record),
        "交进来的记录要在 R 那块盘上、未释放、起点在 R 里、不罩出 R"
    );
    let scan = scan_region_over_the_identity_index(
        region,
        std::slice::from_ref(&record),
        mapping_index,
        &BTreeSet::new(),
        &BTreeSet::new(),
        |_device, _slot, _unit_bytes_requested| unit_bytes.map(<[u8]>::to_vec),
    )
    .expect("一条不跨 R 边界的记录：跨边界与同一把 key 判成可搬两次这两种整次拒都要不止这一条");
    let [allocation]: [ScannedAllocation; 1] = scan
        .allocations
        .try_into()
        .expect("R 那块盘上、未释放、起点在 R 里的一条记录正好分出一项");
    allocation.class
}

/// 区域扫描里一批记录的比对（第 8 条）：两份核的分类交进来（按 `records` 的次序，逐条走映射那一份在前），逐条比、更新计数；
/// 连续不确定到 N 轮的记录交 CPU 参照（[`cpu_reference_classification`]）现判，按身份的索引只在头一次要它时建。
/// 这一批出错就原样交回出错、计数不动、CPU 参照不调。
///
/// # Errors
/// `both_kernel_classifications` 是 `Err` 时交回它。
///
/// # Panics
/// 两份核交回的条数与 `records` 不等；同一个条目键在一批里出现两次；交 CPU 判的那一条不是区域扫描会去读的那一种。
pub fn reconciled_region_record_classifications<
    EntryKey: Eq + std::hash::Hash + Clone,
    BatchFailure,
>(
    region: CompactionRegion,
    records: &[RegionRecordToClassify<'_, EntryKey>],
    mapping_entries: &[CentralMappingEntry],
    both_kernel_classifications: Result<
        Vec<(AllocationInRegion, AllocationInRegion)>,
        BatchFailure,
    >,
    uncertainty: &mut ConsecutiveUncertaintyCounts<EntryKey>,
) -> Result<ReconciledBatch<AllocationInRegion>, BatchFailure> {
    let both_kernel_classifications = both_kernel_classifications.map(|classifications| {
        assert_eq!(
            classifications.len(),
            records.len(),
            "两份核每条各交一个分类"
        );
        records
            .iter()
            .zip(classifications)
            .map(
                |(record, (walking, searching))| TwoKernelOutcomesOfAnEntry {
                    entry_key: record.entry_key.clone(),
                    outcome_by_one_kernel: walking,
                    outcome_by_the_independently_written_kernel: searching,
                },
            )
            .collect()
    });
    let mut mapping_index: Option<CentralMappingIdentityIndex<'_>> = None;
    uncertainty.reconcile_unless_the_batch_failed(both_kernel_classifications, |entry_position| {
        let index = mapping_index.get_or_insert_with(|| {
            CentralMappingIdentityIndex::of_the_mapping_entries(mapping_entries)
        });
        cpu_reference_classification(
            region,
            records[entry_position].record,
            index,
            records[entry_position].unit_bytes,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use singlefs_core::address::{DeviceIdentity, InstanceGeneration, TreeIdentifier};
    use singlefs_core::allocator::AllocationRecordQuarantineMark;
    use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
    use singlefs_core::records::{mapping_key_for_data, mapping_key_for_node};
    use singlefs_core::region_scan::scan_region;
    use singlefs_core::unit::{
        build_data_unit, build_index_node, build_packed_unit, DataUnitIdentity, PackedIdentity,
        WriteOrder, PACKED_TYPE_INODE,
    };

    const REGION_DEVICE: DeviceIdentity = DeviceIdentity(0);
    const MIRROR_DEVICE: DeviceIdentity = DeviceIdentity(1);
    const BIRTH_TXG: CheckpointTxg = CheckpointTxg(7);
    const WRITING_INSTANCE: InstanceGeneration = InstanceGeneration(2);
    const FILESYSTEM_IDENTIFIER: [u8; 16] = [7; 16];
    /// 测试自己定的两棵树号（不读格式里登记的树号）。
    const TREE_OF_THE_TEST_DATA: u64 = 11;
    const TREE_OF_THE_TEST_NODES: u64 = 12;

    fn region_under_test() -> CompactionRegion {
        CompactionRegion::of_one_cluster_segment(REGION_DEVICE, SlotNumber(64 * 900))
            .expect("64 的倍数")
    }

    fn slot_in_region(offset_in_slots: u64) -> SlotNumber {
        SlotNumber(region_under_test().start_slot().0 + offset_in_slots)
    }

    fn unreleased_record(slot: SlotNumber, span_slots: u16) -> AllocationRecord {
        AllocationRecord {
            device: REGION_DEVICE,
            slot,
            span_slots,
            generation: BIRTH_TXG,
            is_released: false,
            quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
        }
    }

    fn locations_at(slot: SlotNumber) -> [LocationEntry; 2] {
        [REGION_DEVICE, MIRROR_DEVICE].map(|device| LocationEntry {
            device,
            slot,
            unit_checksum: 0,
        })
    }

    fn entry(key: Vec<u8>, locations: [LocationEntry; 2]) -> CentralMappingEntry {
        CentralMappingEntry { key, locations }
    }

    fn data_unit_and_key(transaction: u64) -> (Vec<u8>, Vec<u8>) {
        let write_order = WriteOrder {
            instance: WRITING_INSTANCE,
            transaction,
        };
        let bytes = build_data_unit(
            DataUnitIdentity {
                tree: TreeIdentifier(TREE_OF_THE_TEST_DATA),
                object: 1,
                object_birth: CheckpointTxg(3),
                anchor_offset: 0,
            },
            BIRTH_TXG,
            &FILESYSTEM_IDENTIFIER,
            write_order,
            &[0x5A; 100],
        );
        let key = mapping_key_for_data(
            PointerHead {
                birth_tree: TreeIdentifier(TREE_OF_THE_TEST_DATA),
                birth_txg: BIRTH_TXG,
            },
            write_order,
        );
        (bytes, key)
    }

    /// 载荷两万字节、字节各不相同的数据单元：头一片之后的字节不全是 0（全 0 的那几字节从 0 起算出的寄存器也是 0，寄存器接没接上看不出来）。
    fn data_unit_with_a_long_payload_and_key(transaction: u64) -> (Vec<u8>, Vec<u8>) {
        let write_order = WriteOrder {
            instance: WRITING_INSTANCE,
            transaction,
        };
        let payload: Vec<u8> = (0..20_000_u32)
            .map(|position| {
                u8::try_from((position + u32::try_from(transaction).expect("小数")) % 251)
                    .expect("小于 251")
            })
            .collect();
        let bytes = build_data_unit(
            DataUnitIdentity {
                tree: TreeIdentifier(TREE_OF_THE_TEST_DATA),
                object: 1,
                object_birth: CheckpointTxg(3),
                anchor_offset: 0,
            },
            BIRTH_TXG,
            &FILESYSTEM_IDENTIFIER,
            write_order,
            &payload,
        );
        let key = mapping_key_for_data(
            PointerHead {
                birth_tree: TreeIdentifier(TREE_OF_THE_TEST_DATA),
                birth_txg: BIRTH_TXG,
            },
            write_order,
        );
        (bytes, key)
    }

    fn node_pointer(tree: u64, birth_sequence: u32) -> NodePointer {
        NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(tree),
                birth_txg: BIRTH_TXG,
            },
            locations: NodePointer::empty_root().locations,
            instance: WRITING_INSTANCE,
            birth_sequence: BirthSequence(birth_sequence),
        }
    }

    fn index_node_and_key(birth_sequence: u32) -> (Vec<u8>, Vec<u8>) {
        let bytes = build_index_node(
            TreeIdentifier(TREE_OF_THE_TEST_NODES),
            0,
            24,
            &[0; 24],
            &[0; 24],
            BIRTH_TXG,
            &FILESYSTEM_IDENTIFIER,
            WRITING_INSTANCE,
            BirthSequence(birth_sequence),
            112,
            &[],
        );
        let key = mapping_key_for_node(
            UNIT_CLASS_INDEX_NODE,
            node_pointer(TREE_OF_THE_TEST_NODES, birth_sequence),
        );
        (bytes, key)
    }

    fn packed_unit_and_key(birth_sequence: u32) -> (Vec<u8>, Vec<u8>) {
        let bytes = build_packed_unit(
            PackedIdentity {
                birth_tree: TreeIdentifier(TREE_OF_THE_TEST_NODES),
                record_type: PACKED_TYPE_INODE,
                container: 1,
                container_birth: BIRTH_TXG,
            },
            140,
            &[vec![1; 140]],
            BIRTH_TXG,
            &FILESYSTEM_IDENTIFIER,
            WriteOrder {
                instance: WRITING_INSTANCE,
                transaction: 9,
            },
            BirthSequence(birth_sequence),
        );
        let key = mapping_key_for_node(
            UNIT_CLASS_PACKED,
            node_pointer(TREE_OF_THE_TEST_NODES, birth_sequence),
        );
        (bytes, key)
    }

    #[test]
    fn pieces_are_packed_end_to_end_with_their_location_span_read_flag_kind_and_registers() {
        let unit_bytes_of_the_first = [0xA1_u8, 0xA2, 0xA3];
        let unit_bytes_of_the_third = [0xC1_u8, 0xC2];
        let piece_after_the_head = [0xD1_u8];
        let wide_slot = SlotNumber((5_u64 << 32) | 9);
        let whole_records = [
            RegionRecordToClassify {
                entry_key: 0,
                record: unreleased_record(slot_in_region(0), 2),
                unit_bytes: Some(unit_bytes_of_the_first.as_slice()),
            },
            RegionRecordToClassify {
                entry_key: 1,
                record: unreleased_record(slot_in_region(2), 1),
                unit_bytes: None,
            },
        ];
        let packed = packed_region_record_chunk(&[
            RegionRecordPieceForTheKernels::of_a_whole_record(&whole_records[0]),
            RegionRecordPieceForTheKernels::of_a_whole_record(&whole_records[1]),
            RegionRecordPieceForTheKernels::UnitFromItsStart {
                record: unreleased_record(wide_slot, 3),
                bytes: Some(&unit_bytes_of_the_third),
                bytes_of_the_unit_after_this_piece: 700,
                registers_of_the_bytes_after_this_piece: CrcRegistersOfTheTwoKernels {
                    walking_kernel: 0x1111,
                    searching_kernel: 0x2222,
                },
            },
            RegionRecordPieceForTheKernels::PieceAfterTheHead {
                record: unreleased_record(slot_in_region(4), 2),
                bytes: &piece_after_the_head,
                registers_before_this_piece: CrcRegistersOfTheTwoKernels {
                    walking_kernel: 0x3333,
                    searching_kernel: 0x4444,
                },
            },
        ]);
        assert_eq!(
            packed.input_bytes,
            vec![0xA1, 0xA2, 0xA3, 0xC1, 0xC2, 0xD1, 0, 0],
            "读不到的那一条占 0 字节，末尾补到 8 字节"
        );
        let region_start = u32::try_from(slot_in_region(0).0).expect("小槽号");
        assert_eq!(
            packed.descriptor_words,
            vec![
                0,
                3,
                2,
                1,
                0,
                region_start,
                0,
                0,
                0,
                0,
                0, //
                3,
                0,
                1,
                0,
                0,
                region_start + 2,
                0,
                0,
                0,
                0,
                0, //
                3,
                2,
                3,
                1,
                0,
                9,
                5,
                0,
                700,
                0x1111,
                0x2222, //
                5,
                1,
                2,
                1,
                0,
                region_start + 4,
                0,
                1,
                0,
                0x3333,
                0x4444,
            ],
            "起点、字节数、跨度、读到没有、盘、槽号低 32 位、高 32 位、哪一种片、单元在这一片之后的字节数、两份核各自的寄存器"
        );
        assert_eq!(packed.record_count(), 4);
    }

    #[test]
    fn mapping_tables_list_each_entry_with_its_key_length_and_both_locations() {
        let tables = MappingTablesForTheKernels::of_the_mapping_entries(&[entry(
            vec![1, 2, 3, 4, 5],
            [
                LocationEntry {
                    device: DeviceIdentity(3),
                    slot: SlotNumber((1_u64 << 32) | 2),
                    unit_checksum: 0,
                },
                LocationEntry {
                    device: DeviceIdentity(4),
                    slot: SlotNumber(6),
                    unit_checksum: 0,
                },
            ],
        )])
        .expect("5 字节的 key 装得下");
        assert_eq!(
            tables.entry_words,
            vec![5, 0x0403_0201, 5, 0, 0, 0, 0, 0, 3, 2, 1, 4, 6, 0],
            "key 的字节数、key 的七个字、两条位置项各三个字"
        );
        assert_eq!((tables.entry_count, tables.location_item_count), (1, 2));
    }

    #[test]
    fn the_location_index_is_sorted_by_device_and_slot_and_lists_an_entry_once_per_distinct_location(
    ) {
        let same_location_twice = [LocationEntry {
            device: DeviceIdentity(0),
            slot: SlotNumber(70),
            unit_checksum: 0,
        }; 2];
        let tables = MappingTablesForTheKernels::of_the_mapping_entries(&[
            entry(vec![1], locations_at(SlotNumber(90))),
            entry(vec![2], same_location_twice),
            entry(vec![3], locations_at(SlotNumber(70))),
        ])
        .expect("短 key 装得下");
        assert_eq!(
            tables.location_item_words,
            vec![
                0, 70, 0, 1, //
                0, 70, 0, 2, //
                0, 90, 0, 0, //
                1, 70, 0, 2, //
                1, 90, 0, 0,
            ],
            "按（盘、槽号、序号）升序；两条位置项指同一落点的那条条目只记一项"
        );
        assert_eq!(tables.location_item_count, 5);
    }

    #[test]
    fn the_key_index_is_sorted_by_key_length_and_words_with_ties_in_table_order() {
        let tables = MappingTablesForTheKernels::of_the_mapping_entries(&[
            entry(vec![9, 9], locations_at(SlotNumber(1))),
            entry(vec![1, 2, 3], locations_at(SlotNumber(2))),
            entry(vec![5], locations_at(SlotNumber(3))),
            entry(vec![9, 9], locations_at(SlotNumber(4))),
        ])
        .expect("短 key 装得下");
        assert_eq!(
            tables.key_sorted_entry_positions,
            vec![2, 0, 3, 1],
            "先比字节数、再比字；相同的 key 按条目表的次序"
        );
    }

    #[test]
    fn one_mapping_key_longer_than_the_kernels_hold_refuses_the_tables() {
        assert_eq!(
            MappingTablesForTheKernels::of_the_mapping_entries(&[
                entry(vec![1; 28], locations_at(SlotNumber(1))),
                entry(vec![1; 29], locations_at(SlotNumber(2))),
            ]),
            Err(MappingKeyLongerThanTheKernelsHold {
                entry_position: 1,
                key_bytes: 29
            }),
            "28 字节正好装满七个字，29 字节装不下"
        );
    }

    #[test]
    fn empty_mapping_tables_still_have_one_item_in_every_buffer() {
        let tables = MappingTablesForTheKernels::of_the_mapping_entries(&[]).expect("空映射建得成");
        assert_eq!(tables.entry_words, vec![0; MAPPING_ENTRY_WORDS]);
        assert_eq!(tables.location_item_words, vec![0; LOCATION_ITEM_WORDS]);
        assert_eq!(tables.key_sorted_entry_positions, vec![0]);
        assert_eq!((tables.entry_count, tables.location_item_count), (0, 0));
        assert_eq!(
            tables.bytes_on_the_gpu(),
            4 * u64::try_from(MAPPING_ENTRY_WORDS + LOCATION_ITEM_WORDS + 1).expect("小数")
        );
    }

    fn outcome_bytes(outcome_words: [u32; REGION_RECORD_OUTCOME_WORDS]) -> Vec<u8> {
        little_endian_bytes_of_the_words(&outcome_words)
    }

    #[test]
    fn every_outcome_kind_decodes_into_its_region_scan_class() {
        let (_bytes, data_key) = data_unit_and_key(4);
        let mut movable_key_words = [0_u32; KEY_WORDS];
        let mut padded_key = data_key.clone();
        padded_key.resize(KEY_BYTES_THE_KERNELS_HOLD, 0);
        for (word, word_bytes) in movable_key_words
            .iter_mut()
            .zip(padded_key.as_chunks::<4>().0)
        {
            *word = u32::from_le_bytes(*word_bytes);
        }
        let key_length = u32::try_from(data_key.len()).expect("27");
        let mut readback = Vec::new();
        let [key_word_0, key_word_1, key_word_2, key_word_3, key_word_4, key_word_5, key_word_6] =
            movable_key_words;
        readback.extend(outcome_bytes([
            OUTCOME_KIND_MOVABLE,
            u32::from(UNIT_CLASS_DATA),
            0,
            7,
            1,
            key_length,
            key_word_0,
            key_word_1,
            key_word_2,
            key_word_3,
            key_word_4,
            key_word_5,
            key_word_6,
        ]));
        readback.extend(outcome_bytes([
            OUTCOME_KIND_DEAD,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ]));
        readback.extend(outcome_bytes([
            OUTCOME_KIND_UNIT_CLASS_DISAGREES_WITH_THE_MAPPING_KEY,
            u32::from(UNIT_CLASS_PACKED),
            u32::from(UNIT_CLASS_INDEX_NODE),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ]));
        readback.extend(outcome_bytes([
            OUTCOME_KIND_HEADER_KEY_DISAGREES_WITH_THE_MAPPING_LOCATION,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ]));
        readback.extend(outcome_bytes([
            OUTCOME_KIND_DEVICE_READ_FAILED,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ]));
        readback.extend(outcome_bytes([
            OUTCOME_KIND_SPAN_IS_NOT_A_UNIT_FOOTPRINT,
            3,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ]));
        readback.extend(outcome_bytes([
            OUTCOME_KIND_REFUSED_BY_THE_PARSER,
            UNIT_ERROR_WRONG_UNIT_CLASS,
            u32::from(UNIT_CLASS_DATA) | (2 << 8),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ]));
        readback.extend(outcome_bytes([
            OUTCOME_KIND_REFUSED_BY_THE_PARSER,
            UNIT_ERROR_STRUCTURE,
            7,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ]));
        let unreadable = |unreadable| {
            AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(unreadable))
        };
        assert_eq!(
            decoded_region_record_outcomes(&readback),
            Ok(vec![
                AllocationInRegion::Movable(MovableUnit {
                    mapping_key: data_key,
                    unit_class: MappedUnitClass::Data,
                    birth_txg: CheckpointTxg((1 << 32) | 7),
                }),
                AllocationInRegion::Dead,
                AllocationInRegion::Inconsistent(
                    RegionScanInconsistency::UnitClassDisagreesWithTheMappingKey {
                        header_unit_class: MappedUnitClass::Packed,
                        mapping_key_class_tag: UNIT_CLASS_INDEX_NODE,
                    }
                ),
                AllocationInRegion::Inconsistent(
                    RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation
                ),
                unreadable(UnitHeaderUnreadable::DeviceReadFailed),
                unreadable(UnitHeaderUnreadable::SpanIsNotAUnitFootprint { span_slots: 3 }),
                unreadable(UnitHeaderUnreadable::RefusedByTheParser(
                    UnitError::WrongUnitClass {
                        expected: UNIT_CLASS_DATA,
                        found: 2
                    }
                )),
                unreadable(UnitHeaderUnreadable::RefusedByTheParser(
                    UnitError::Structure(UnitStructureDefect::PackedUnitRecordAreaPastTheUnitEnd)
                )),
            ])
        );
    }

    #[test]
    fn an_outcome_the_kernels_never_wrote_is_an_unrecognized_verdict_word() {
        let mut readback = outcome_bytes([OUTCOME_KIND_DEAD, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        readback.extend(region_record_outcome_buffer_before_the_kernel_writes(1));
        assert_eq!(
            decoded_region_record_outcomes(&readback),
            Err(UnrecognizedVerdictWord {
                entry_position: 1,
                verdict_word: REGION_RECORD_OUTCOME_NOT_WRITTEN_FILL_WORD
            }),
            "位置 1 的那一条还是填充字：核没写到它"
        );
    }

    /// 核心层整个 R 扫一遍的分类，与 CPU 参照逐条单独判的分类逐条相同：可搬的三类、已死、类标签不符、key 与位置不符、读不到、跨度不对、头校验和坏。
    #[test]
    fn the_cpu_reference_classifies_each_record_as_the_whole_region_scan_does() {
        let (data_bytes, data_key) = data_unit_and_key(4);
        let (node_bytes, node_key) = index_node_and_key(5);
        let (packed_bytes, packed_key) = packed_unit_and_key(6);
        let (dead_bytes, _dead_key) = data_unit_and_key(40);
        let (other_node_bytes, _other_node_key) = index_node_and_key(50);
        let (mislabelled_bytes, _mislabelled_key) = data_unit_and_key(60);
        let mut header_checksum_broken = data_unit_and_key(70).0;
        header_checksum_broken[20] ^= 1;
        let records_and_bytes: Vec<(AllocationRecord, Option<Vec<u8>>)> = vec![
            (unreleased_record(slot_in_region(0), 2), Some(data_bytes)),
            (unreleased_record(slot_in_region(2), 1), Some(node_bytes)),
            (unreleased_record(slot_in_region(4), 2), Some(packed_bytes)),
            (unreleased_record(slot_in_region(6), 2), Some(dead_bytes)),
            (
                unreleased_record(slot_in_region(8), 1),
                Some(other_node_bytes),
            ),
            (
                unreleased_record(slot_in_region(10), 2),
                Some(mislabelled_bytes),
            ),
            (unreleased_record(slot_in_region(12), 2), None),
            (unreleased_record(slot_in_region(14), 3), None),
            (
                unreleased_record(slot_in_region(18), 2),
                Some(header_checksum_broken),
            ),
        ];
        let mapping_entries = vec![
            entry(data_key, locations_at(slot_in_region(0))),
            entry(node_key.clone(), locations_at(slot_in_region(2))),
            entry(packed_key, locations_at(slot_in_region(4))),
            // 槽 8 上那个节点的 key 不在映射里，指着槽 8 的是槽 2 那个节点的 key
            entry(node_key, locations_at(slot_in_region(8))),
            // 槽 10 上是数据单元，指着它的条目 key 打头是节点的类标签
            entry(index_node_and_key(61).1, locations_at(slot_in_region(10))),
        ];
        let records: Vec<AllocationRecord> = records_and_bytes
            .iter()
            .map(|(record, _bytes)| *record)
            .collect();
        let whole_region_scan = scan_region(
            region_under_test(),
            &records,
            &mapping_entries,
            &BTreeSet::new(),
            &BTreeSet::new(),
            |_device, slot, _unit_bytes_requested| {
                records_and_bytes
                    .iter()
                    .find(|(record, _bytes)| record.slot == slot)
                    .and_then(|(_record, bytes)| bytes.clone())
            },
        )
        .expect("这个 R 扫得成");
        let mapping_index = CentralMappingIdentityIndex::of_the_mapping_entries(&mapping_entries);
        let one_by_one: Vec<AllocationInRegion> = records_and_bytes
            .iter()
            .map(|(record, bytes)| {
                cpu_reference_classification(
                    region_under_test(),
                    *record,
                    &mapping_index,
                    bytes.as_deref(),
                )
            })
            .collect();
        let whole: Vec<AllocationInRegion> = whole_region_scan
            .allocations
            .into_iter()
            .map(|allocation| allocation.class)
            .collect();
        assert_eq!(one_by_one, whole);
        assert!(
            matches!(one_by_one[0], AllocationInRegion::Movable(_))
                && matches!(one_by_one[1], AllocationInRegion::Movable(_))
                && matches!(one_by_one[2], AllocationInRegion::Movable(_))
                && one_by_one[3] == AllocationInRegion::Dead,
            "三类可搬与已死都在：{one_by_one:?}"
        );
    }

    #[test]
    #[should_panic(expected = "交进来的记录要在 R 那块盘上")]
    fn the_cpu_reference_refuses_a_record_outside_the_region() {
        let mapping_index = CentralMappingIdentityIndex::of_the_mapping_entries(&[]);
        let _classification = cpu_reference_classification(
            region_under_test(),
            unreleased_record(SlotNumber(region_under_test().end_slot().0), 1),
            &mapping_index,
            None,
        );
    }

    /// 两份核过得了 wgpu 自带的着色器校验（naga 的 WGSL 前端与校验器，起管线时 wgpu 走的同一步），入口点在、是计算着色器。不要 GPU。
    #[test]
    fn both_region_record_kernels_pass_the_shader_validator() {
        for (kernel_source, entry_point) in [
            (WALKING_KERNEL_SOURCE, WALKING_KERNEL_ENTRY_POINT),
            (SEARCHING_KERNEL_SOURCE, SEARCHING_KERNEL_ENTRY_POINT),
        ] {
            let module =
                wgpu::naga::front::wgsl::parse_str(kernel_source).unwrap_or_else(|error| {
                    panic!(
                        "{entry_point} 那一份解析不成：{}",
                        error.emit_to_string(kernel_source)
                    )
                });
            wgpu::naga::valid::Validator::new(
                wgpu::naga::valid::ValidationFlags::all(),
                wgpu::naga::valid::Capabilities::empty(),
            )
            .validate(&module)
            .unwrap_or_else(|error| {
                panic!(
                    "{entry_point} 那一份过不了校验：{}",
                    error.emit_to_string(kernel_source)
                )
            });
            assert!(
                module.entry_points.iter().any(|module_entry_point| {
                    module_entry_point.name == entry_point
                        && module_entry_point.stage == wgpu::naga::ShaderStage::Compute
                }),
                "{entry_point} 是这一份里的计算入口"
            );
        }
    }

    #[test]
    fn the_two_kernels_are_separate_sources_with_their_own_entry_points() {
        assert!(WALKING_KERNEL_SOURCE.contains(&format!("fn {WALKING_KERNEL_ENTRY_POINT}(")));
        assert!(SEARCHING_KERNEL_SOURCE.contains(&format!("fn {SEARCHING_KERNEL_ENTRY_POINT}(")));
        assert!(
            !SEARCHING_KERNEL_SOURCE.contains(WALKING_KERNEL_ENTRY_POINT)
                && !WALKING_KERNEL_SOURCE.contains(SEARCHING_KERNEL_ENTRY_POINT),
            "两份核不互相引用"
        );
    }

    /// 用户 2026-10-28 定「装不下应该自适应的分片」：上限只装得下最大那条 32 KiB 的记录连同一条条目的映射，映射按落点切成一片一个落点
    /// （两块盘上各六个），每条记录在每片之下各分一次类（两份核在 CPU 上的替身照核心层对这一片判），两份各按自己的合法合成，
    /// 与 CPU 参照对整份映射的分类逐条相同。槽 20 上那个单元头里的 key 只有一条条目带着、它指着槽 40（别的片）：本片判成已死，合成之后是
    /// 「key 与位置不符」。
    #[test]
    fn the_mapping_cut_into_slices_under_a_small_memory_limit_classifies_every_record_as_the_whole_mapping(
    ) {
        let (data_bytes, data_key) = data_unit_and_key(4);
        let (node_bytes, node_key) = index_node_and_key(5);
        let (packed_bytes, packed_key) = packed_unit_and_key(6);
        let (dead_bytes, _dead_key) = data_unit_and_key(40);
        let (other_node_bytes, _other_node_key) = index_node_and_key(50);
        let (mislabelled_bytes, _mislabelled_key) = data_unit_and_key(60);
        let (key_mapped_elsewhere_bytes, key_mapped_elsewhere) = data_unit_and_key(80);
        let records_and_bytes: Vec<(AllocationRecord, Option<Vec<u8>>)> = vec![
            (unreleased_record(slot_in_region(0), 2), Some(data_bytes)),
            (unreleased_record(slot_in_region(2), 1), Some(node_bytes)),
            (unreleased_record(slot_in_region(4), 2), Some(packed_bytes)),
            (unreleased_record(slot_in_region(6), 2), Some(dead_bytes)),
            (
                unreleased_record(slot_in_region(8), 1),
                Some(other_node_bytes),
            ),
            (
                unreleased_record(slot_in_region(10), 2),
                Some(mislabelled_bytes),
            ),
            (unreleased_record(slot_in_region(12), 2), None),
            (unreleased_record(slot_in_region(14), 3), None),
            (
                unreleased_record(slot_in_region(20), 2),
                Some(key_mapped_elsewhere_bytes),
            ),
        ];
        let mapping_entries = vec![
            entry(data_key, locations_at(slot_in_region(0))),
            entry(node_key.clone(), locations_at(slot_in_region(2))),
            entry(packed_key, locations_at(slot_in_region(4))),
            entry(node_key, locations_at(slot_in_region(8))),
            entry(index_node_and_key(61).1, locations_at(slot_in_region(10))),
            entry(key_mapped_elsewhere, locations_at(slot_in_region(40))),
        ];
        let records: Vec<RegionRecordToClassify<'_, usize>> = records_and_bytes
            .iter()
            .enumerate()
            .map(|(entry_key, (record, bytes))| RegionRecordToClassify {
                entry_key,
                record: *record,
                unit_bytes: bytes.as_deref(),
            })
            .collect();
        let mapping_tables = MappingTablesForTheKernels::of_the_mapping_entries(&mapping_entries)
            .expect("key 都是 27 字节");
        let one_entry_with_one_location_bytes = mapping_table_bytes_of(1, 1).0;
        let memory_limit =
            region_record_chunk_footprint_of_mapping_table_bytes(one_entry_with_one_location_bytes)
                .bytes_held_by_a_chunk(32768, 1);
        let room = ChunkBoundsOfAGpu {
            memory_limit: Some(singlefs_core::background_compute::GpuMemoryInBytes(
                memory_limit,
            )),
            largest_storage_binding_in_bytes: 1 << 30,
            largest_entry_count_per_dispatch: 1 << 20,
        };
        let plan = region_record_plan_on_a_gpu(&records, &mapping_entries, &mapping_tables, room)
            .expect("最大那条记录连同一个落点上的一条条目装得下");
        assert_eq!(
            plan.mapping_slices.len(),
            12,
            "两块盘上各六个落点，一片一个"
        );
        assert!(
            plan.chunks_of_the_pieces_after_the_heads.is_empty(),
            "整个单元装得下：一个单元都不切"
        );
        let classifications =
            region_record_classifications_of_the_plan(&records, &plan, |pieces, mapping_slice| {
                Ok::<_, ()>(stand_in_for_both_kernels(
                    &records_and_bytes,
                    pieces,
                    mapping_slice,
                ))
            })
            .expect("替身不出错");
        let whole_index = CentralMappingIdentityIndex::of_the_mapping_entries(&mapping_entries);
        let reference: Vec<AllocationInRegion> = records
            .iter()
            .map(|to_classify| {
                cpu_reference_classification(
                    region_under_test(),
                    to_classify.record,
                    &whole_index,
                    to_classify.unit_bytes,
                )
            })
            .collect();
        assert_eq!(
            reference[8],
            header_key_disagrees_with_the_mapping_location(),
            "槽 20 那一条按整份映射是 key 与位置不符"
        );
        assert_eq!(
            reference[3],
            AllocationInRegion::Dead,
            "槽 6 那一条按整份映射是已死"
        );
        assert_eq!(
            classifications,
            reference
                .into_iter()
                .map(|classification| (classification.clone(), classification))
                .collect::<Vec<_>>(),
            "映射切成几片、各片合成，两份照样逐条与 CPU 参照对整份映射的分类相同"
        );
    }

    /// CRC-32C 的寄存器（不取反、不带全 1 初值）逐位移喂 `bytes` 之后的值：替身算头一片之后那几片用它。
    fn crc_register_after_bytes(register_before: u32, bytes: &[u8]) -> u32 {
        let mut register = register_before;
        for byte in bytes {
            register ^= u32::from(*byte);
            for _bit in 0..8 {
                register = if register & 1 == 1 {
                    (register >> 1) ^ 0x82F6_3B78
                } else {
                    register >> 1
                };
            }
        }
        register
    }

    /// 两份核在 CPU 上的替身对一段的交法（见本模块文件头「一个单元切成几片」）。头一片之后的一片：两份各自从带进来的寄存器接着算这几个字节，
    /// 交回寄存器。从单元头起的一片：照记录在 `records_and_bytes` 里找回整个单元，核对这一片就是单元的开头、之后的字节数对、两份各自带进来的寄存器
    /// 就是之后那些字节从 0 起算出的；对得上就照核心层对整个单元、这一片映射分类，对不上那一份交「载荷校验和不对」（与整个单元的分类不同，用例比对时红）。
    fn stand_in_for_both_kernels(
        records_and_bytes: &[(AllocationRecord, Option<Vec<u8>>)],
        pieces: &[RegionRecordPieceForTheKernels<'_>],
        mapping_slice: Option<&MappingSliceWithItsChunks>,
    ) -> BothKernelOutcomesOfARegionRecordChunk {
        let whole_unit_of = |record: AllocationRecord| {
            records_and_bytes
                .iter()
                .find(|(whole_record, _bytes)| *whole_record == record)
                .expect("片是用例交进去的记录的")
                .1
                .clone()
        };
        let Some(mapping_slice) = mapping_slice else {
            return BothKernelOutcomesOfARegionRecordChunk::RegistersAfterThePieces(
                pieces
                    .iter()
                    .map(|piece| {
                        let RegionRecordPieceForTheKernels::PieceAfterTheHead {
                            bytes,
                            registers_before_this_piece,
                            ..
                        } = piece
                        else {
                            panic!("不交映射片的段全是头一片之后的片：{piece:?}");
                        };
                        CrcRegistersOfTheTwoKernels {
                            walking_kernel: crc_register_after_bytes(
                                registers_before_this_piece.walking_kernel,
                                bytes,
                            ),
                            searching_kernel: crc_register_after_bytes(
                                registers_before_this_piece.searching_kernel,
                                bytes,
                            ),
                        }
                    })
                    .collect(),
            );
        };
        let slice_index =
            CentralMappingIdentityIndex::of_the_mapping_entries(&mapping_slice.entries);
        let payload_checksum_mismatch =
            AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
                UnitHeaderUnreadable::RefusedByTheParser(UnitError::PayloadChecksumMismatch),
            ));
        BothKernelOutcomesOfARegionRecordChunk::Classifications(
            pieces
                .iter()
                .map(|piece| {
                    let RegionRecordPieceForTheKernels::UnitFromItsStart {
                        record,
                        bytes,
                        bytes_of_the_unit_after_this_piece,
                        registers_of_the_bytes_after_this_piece,
                    } = piece
                    else {
                        panic!("交映射片的段全是从单元头起的片：{piece:?}");
                    };
                    let whole_unit = whole_unit_of(*record);
                    let classification = cpu_reference_classification(
                        region_under_test(),
                        *record,
                        &slice_index,
                        whole_unit.as_deref(),
                    );
                    let piece_is_the_start_of_the_unit = match (bytes, &whole_unit) {
                        (None, None) => *bytes_of_the_unit_after_this_piece == 0,
                        (Some(bytes), Some(whole_unit)) => {
                            whole_unit.starts_with(bytes)
                                && usize::try_from(*bytes_of_the_unit_after_this_piece)
                                    .expect("小数")
                                    == whole_unit.len() - bytes.len()
                        }
                        (None, Some(_)) | (Some(_), None) => false,
                    };
                    let register_of_the_rest = whole_unit.as_ref().map_or(0, |whole_unit| {
                        crc_register_after_bytes(0, &whole_unit[bytes.map_or(0, <[u8]>::len)..])
                    });
                    let classification_given = |register_carried_in: u32| {
                        if piece_is_the_start_of_the_unit
                            && register_carried_in == register_of_the_rest
                        {
                            classification.clone()
                        } else {
                            payload_checksum_mismatch.clone()
                        }
                    };
                    (
                        classification_given(
                            registers_of_the_bytes_after_this_piece.walking_kernel,
                        ),
                        classification_given(
                            registers_of_the_bytes_after_this_piece.searching_kernel,
                        ),
                    )
                })
                .collect(),
        )
    }

    /// 用户 2026-10-28 定「这部分不能因为装不下就拒绝，装不下应该自适应的分片，缩小逐次进行」：上限 2 KiB，连一个 16 KiB 的索引节点都装不下
    /// （数据单元的载荷两万字节、字节各不相同，头一片之后的寄存器不是 0）：
    /// 比头一片长的单元都切成头一片与之后的几片（之后的几片按上限切、同一个单元的各片在前后不同的段里），两份核在 CPU 上的替身先逐片接着算寄存器、
    /// 再照头一片分类（替身核对头一片、之后的字节数与两份各自带进来的寄存器，对不上就交「载荷校验和不对」）；每条记录两份的分类与 CPU 参照对整个单元、
    /// 整份映射的分类逐条相同。三类单元、已死、读不到、跨度不对、key 与位置不符都在里面。
    #[test]
    fn units_cut_into_pieces_under_a_memory_limit_smaller_than_a_unit_classify_every_record_as_the_whole_unit(
    ) {
        let (data_bytes, data_key) = data_unit_with_a_long_payload_and_key(4);
        let (node_bytes, node_key) = index_node_and_key(5);
        let (packed_bytes, packed_key) = packed_unit_and_key(6);
        let (dead_bytes, _dead_key) = data_unit_with_a_long_payload_and_key(40);
        let (key_mapped_elsewhere_bytes, key_mapped_elsewhere) =
            data_unit_with_a_long_payload_and_key(80);
        let records_and_bytes: Vec<(AllocationRecord, Option<Vec<u8>>)> = vec![
            (unreleased_record(slot_in_region(0), 2), Some(data_bytes)),
            (unreleased_record(slot_in_region(2), 1), Some(node_bytes)),
            (unreleased_record(slot_in_region(4), 2), Some(packed_bytes)),
            (unreleased_record(slot_in_region(6), 2), Some(dead_bytes)),
            (unreleased_record(slot_in_region(12), 2), None),
            (unreleased_record(slot_in_region(14), 3), None),
            (
                unreleased_record(slot_in_region(20), 2),
                Some(key_mapped_elsewhere_bytes),
            ),
        ];
        let mapping_entries = vec![
            entry(data_key, locations_at(slot_in_region(0))),
            entry(node_key, locations_at(slot_in_region(2))),
            entry(packed_key, locations_at(slot_in_region(4))),
            entry(key_mapped_elsewhere, locations_at(slot_in_region(40))),
        ];
        let records: Vec<RegionRecordToClassify<'_, usize>> = records_and_bytes
            .iter()
            .enumerate()
            .map(|(entry_key, (record, bytes))| RegionRecordToClassify {
                entry_key,
                record: *record,
                unit_bytes: bytes.as_deref(),
            })
            .collect();
        let mapping_tables = MappingTablesForTheKernels::of_the_mapping_entries(&mapping_entries)
            .expect("key 都是 27 字节");
        let room = ChunkBoundsOfAGpu {
            memory_limit: Some(singlefs_core::background_compute::GpuMemoryInBytes(2048)),
            largest_storage_binding_in_bytes: 1 << 30,
            largest_entry_count_per_dispatch: 1 << 20,
        };
        let plan = region_record_plan_on_a_gpu(&records, &mapping_entries, &mapping_tables, room)
            .expect("切开之后头一片连同一个落点上的条目、之后的每一片都装得下 2 KiB");
        assert_eq!(
            plan.bytes_from_the_start_of_each_record,
            vec![
                UNIT_HEAD_PIECE_INPUT_BYTES,
                UNIT_HEAD_PIECE_INPUT_BYTES,
                UNIT_HEAD_PIECE_INPUT_BYTES,
                UNIT_HEAD_PIECE_INPUT_BYTES,
                0,
                0,
                UNIT_HEAD_PIECE_INPUT_BYTES,
            ],
            "读到的单元都切成头一片，读不到的交 0 字节"
        );
        let pieces_after_the_head_of_the_index_node: u64 = plan
            .chunks_of_the_pieces_after_the_heads
            .iter()
            .flatten()
            .filter(|piece| piece.entry_position == 1)
            .map(PieceOfAnEntry::input_byte_count)
            .sum();
        assert_eq!(
            pieces_after_the_head_of_the_index_node,
            16384 - UNIT_HEAD_PIECE_INPUT_BYTES,
            "索引节点头一片之后的几片一个字节不漏"
        );
        assert!(
            plan.chunks_of_the_pieces_after_the_heads.len() > 1,
            "头一片之后的字节切成了好几段"
        );
        let classifications =
            region_record_classifications_of_the_plan(&records, &plan, |pieces, mapping_slice| {
                Ok::<_, ()>(stand_in_for_both_kernels(
                    &records_and_bytes,
                    pieces,
                    mapping_slice,
                ))
            })
            .expect("替身不出错");
        let whole_index = CentralMappingIdentityIndex::of_the_mapping_entries(&mapping_entries);
        let reference: Vec<AllocationInRegion> = records
            .iter()
            .map(|to_classify| {
                cpu_reference_classification(
                    region_under_test(),
                    to_classify.record,
                    &whole_index,
                    to_classify.unit_bytes,
                )
            })
            .collect();
        assert!(
            matches!(reference[0], AllocationInRegion::Movable(_))
                && matches!(reference[1], AllocationInRegion::Movable(_))
                && matches!(reference[2], AllocationInRegion::Movable(_))
                && reference[3] == AllocationInRegion::Dead
                && reference[6] == header_key_disagrees_with_the_mapping_location(),
            "三类可搬、已死、key 与位置不符都在里面：{reference:?}"
        );
        assert_eq!(
            classifications,
            reference
                .into_iter()
                .map(|classification| (classification.clone(), classification))
                .collect::<Vec<_>>(),
            "单元切成几片交，两份照样逐条与 CPU 参照对整个单元的分类相同"
        );
    }

    /// 头一片之后那几片的结果读回来是每片的寄存器；核没写到的那一片（还是填充字）认不得，按 GPU 出错交回。
    #[test]
    fn registers_after_the_pieces_decode_only_from_the_kind_the_kernels_write_for_them() {
        let mut outcome_words = [0_u32; REGION_RECORD_OUTCOME_WORDS];
        outcome_words[0] = OUTCOME_KIND_REGISTER_AFTER_THE_PIECE;
        outcome_words[1] = 0xDEAD_BEEF;
        let mut readback = outcome_bytes(outcome_words);
        readback.extend(region_record_outcome_buffer_before_the_kernel_writes(1));
        assert_eq!(
            decoded_registers_after_the_pieces(&readback),
            Err(UnrecognizedVerdictWord {
                entry_position: 1,
                verdict_word: REGION_RECORD_OUTCOME_NOT_WRITTEN_FILL_WORD,
            }),
            "核没写到的那一片认不得"
        );
        assert_eq!(
            decoded_registers_after_the_pieces(&readback[..52]),
            Ok(vec![0xDEAD_BEEF]),
            "写了的那一片读出寄存器"
        );
    }

    /// 映射三张表的字节数按条数与落点项数现算，与真建出来的相同（一项都没有时各放一项）。
    #[test]
    fn mapping_table_bytes_counted_from_the_entries_match_the_tables_built() {
        let (data_bytes_unused, data_key) = data_unit_and_key(4);
        drop(data_bytes_unused);
        for (entries, expected_location_items) in [
            (Vec::new(), 0_u64),
            (
                vec![entry(data_key.clone(), locations_at(slot_in_region(0)))],
                2,
            ),
            (
                vec![
                    entry(data_key.clone(), locations_at(slot_in_region(0))),
                    entry(
                        data_key,
                        [
                            locations_at(slot_in_region(3))[0],
                            locations_at(slot_in_region(3))[0],
                        ],
                    ),
                ],
                3,
            ),
        ] {
            let tables = MappingTablesForTheKernels::of_the_mapping_entries(&entries)
                .expect("key 都是 27 字节");
            assert_eq!(
                mapping_table_bytes_of(
                    u64::try_from(entries.len()).expect("小数"),
                    expected_location_items
                ),
                (tables.bytes_on_the_gpu(), tables.largest_buffer_bytes()),
                "{} 条条目",
                entries.len()
            );
        }
    }
}
