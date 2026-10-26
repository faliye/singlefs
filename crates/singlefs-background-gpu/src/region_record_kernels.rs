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
//! - 绑定 1，只读存储：这一段每条记录读到的单元字节首尾相接，末尾补 0 到 4 字节的整数倍（读不到的记录占 0 字节）。
//! - 绑定 2，只读存储：每条记录一项描述，[`RECORD_DESCRIPTOR_WORDS`] 个 `u32`：在输入里的起点字节、字节数、跨度、读到没有（1 是读到）、
//!   盘、槽号低 32 位、槽号高 32 位。
//! - 绑定 3，只读存储：中央映射每条一项，[`MAPPING_ENTRY_WORDS`] 个 `u32`：key 的字节数、key 按小端装进 [`KEY_WORDS`] 个字（不满的补 0）、
//!   两条位置项各三个字（盘、槽号低 32 位、高 32 位），按条目表的次序。
//! - 绑定 4，只读存储：落点索引，每项 [`LOCATION_ITEM_WORDS`] 个 `u32`（盘、槽号低 32 位、高 32 位、条目在表里的序号），
//!   每条条目每个不同的落点一项，按（盘、槽号、序号）升序。
//! - 绑定 5，只读存储：按 key 排的条目序号，按（key 的字节数、key 的字）升序，相同的按序号。
//! - 绑定 6，读写存储：每条记录一项结果，[`REGION_RECORD_OUTCOME_WORDS`] 个 `u32`：结果的类、两个细节字、诞生代号低 32 位与高 32 位、
//!   key 的字节数与 key 的字（只有可搬那一类填）。开算之前整块填 [`REGION_RECORD_OUTCOME_NOT_WRITTEN_FILL_WORD`]：核没写到的那一条
//!   读回来结果的类认不得，按 GPU 出错交回。
//!
//! 映射条目、落点索引与按 key 排的序号一项都没有时，各缓冲照样放一项全 0 的（wgpu 不许长度为 0 的绑定），条数照参数里的数。

use std::collections::BTreeSet;

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
use crate::memory_limit_and_chunks::{input_buffer_bytes, ChunkFootprint};
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
pub const RECORD_DESCRIPTOR_WORDS: usize = 7;
pub const RECORD_DESCRIPTOR_BYTES: u64 = 28;
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

/// 把一段记录按次序打包：每条读到的单元字节首尾相接（起点是前面那些字节数之和，不按字对齐），读不到的占 0 字节。
///
/// # Panics
/// 一段的输入超过 `u32` 能表示的字节数：切批按绑定上限切过（外壳把绑定上限压在 `u32` 之内）。
#[must_use]
pub fn packed_region_record_chunk<EntryKey>(
    records: &[RegionRecordToClassify<'_, EntryKey>],
) -> PackedRegionRecordChunk {
    let total_unit_bytes: usize = records
        .iter()
        .map(|record| record.unit_bytes.map_or(0, <[u8]>::len))
        .sum();
    let padded_bytes =
        input_buffer_bytes(u64::try_from(total_unit_bytes).expect("usize 装得进 u64"));
    let mut input_bytes =
        Vec::with_capacity(usize::try_from(padded_bytes).expect("一段的输入装得进内存"));
    let mut descriptor_words = Vec::with_capacity(records.len() * RECORD_DESCRIPTOR_WORDS);
    for record in records {
        let byte_offset = u32::try_from(input_bytes.len())
            .expect("一段的输入不超过 u32 字节：切批按绑定上限切过");
        let unit_bytes = record.unit_bytes.unwrap_or(&[]);
        let byte_length =
            u32::try_from(unit_bytes.len()).expect("一个单元不超过 u32 字节：切批按绑定上限切过");
        input_bytes.extend_from_slice(unit_bytes);
        let [slot_low, slot_high] = low_and_high_words_of_the_slot(record.record.slot);
        descriptor_words.extend_from_slice(&[
            byte_offset,
            byte_length,
            u32::from(record.record.span_slots),
            u32::from(record.unit_bytes.is_some()),
            record.record.device.0,
            slot_low,
            slot_high,
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
    ChunkFootprint {
        bytes_per_chunk_besides_the_entries: REGION_RECORD_PARAMETER_BUFFER_BYTES
            + mapping_tables.bytes_on_the_gpu(),
        bytes_per_entry_besides_its_input: RECORD_DESCRIPTOR_BYTES
            + REGION_RECORD_OUTCOME_AND_READBACK_BUFFERS * REGION_RECORD_OUTCOME_BYTES,
        bytes_per_entry_in_the_largest_entry_buffer: REGION_RECORD_OUTCOME_BYTES
            .max(RECORD_DESCRIPTOR_BYTES),
    }
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
    fn records_are_packed_end_to_end_with_their_location_span_and_read_flag() {
        let unit_bytes_of_the_first = [0xA1_u8, 0xA2, 0xA3];
        let unit_bytes_of_the_third = [0xC1_u8, 0xC2];
        let wide_slot = SlotNumber((5_u64 << 32) | 9);
        let packed = packed_region_record_chunk(&[
            RegionRecordToClassify {
                entry_key: 0,
                record: unreleased_record(slot_in_region(0), 2),
                unit_bytes: Some(&unit_bytes_of_the_first),
            },
            RegionRecordToClassify {
                entry_key: 1,
                record: unreleased_record(slot_in_region(2), 1),
                unit_bytes: None,
            },
            RegionRecordToClassify {
                entry_key: 2,
                record: unreleased_record(wide_slot, 3),
                unit_bytes: Some(&unit_bytes_of_the_third),
            },
        ]);
        assert_eq!(
            packed.input_bytes,
            vec![0xA1, 0xA2, 0xA3, 0xC1, 0xC2, 0, 0, 0],
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
                0, //
                3,
                0,
                1,
                0,
                0,
                region_start + 2,
                0, //
                3,
                2,
                3,
                1,
                0,
                9,
                5,
            ],
            "起点、字节数、跨度、读到没有、盘、槽号低 32 位、高 32 位"
        );
        assert_eq!(packed.record_count(), 3);
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
    fn a_mapping_key_longer_than_the_kernels_hold_refuses_the_tables() {
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
}
