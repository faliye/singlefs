//! GPU 判器的输入表（崩溃放量第 ① 段的另一半，用户 2026-09-28 定「GPU 判器现在实现、把整套判法搬上去」）。
//!
//! 一个崩溃状态的镜像 = 基线 + 这个状态里持久的写按枚举写表次序叠上去（`singlefs_harness::memory_pool::CrashImage::read`）。
//! 判器读盘只读**结构槽**：系统配置槽、根环槽、journal 记录槽、单元区里的 16 KiB 槽（单元读 16 KiB 或 32 KiB）。
//! 这里把每个结构槽叫一个**落点**（[`JudgeLocation`]），把它在全部崩溃状态里可能呈现的每一种字节叫一个**版本**（[`JudgingUnitsFingerprint`]）：
//! 版本由「盖住这个落点的那几次写各自持久没持久」决定，几次写里只有崩溃那一段里的几次是自由的，别的段整段持久或整段没持久，
//! 所以可达的组合很少（每个落点几个到几十个），按段逐个枚举、去重，字节按 `CrashImage::read` 同一条规则叠出来。
//! CPU 在这里把每个版本的字节、校验和（CRC-32C；整槽宽校验和字段也是 CRC-32C 加 28 个 0）、与每次写逐扇区等不等算一次；
//! GPU 每个状态只做组合逻辑：由持久掩码算出每个落点的组合、查到版本、按字节解结构、走恢复与 checker 的判法（`crate::crash_judge_gpu`）。
//!
//! 表的字节序列化进 KV 时按录入键存（建表与打包的函数在录入那一组判法单元的入口里，表的形态变了录入那一把键就变），
//! [`JudgeTables::digest`] 只按内容取（与事实表同一条规矩）。
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_field_holds};
use singlefs_core::journal::slot_after_the_journal_ring;
use singlefs_core::recovery::PoolReader;
use singlefs_core::root_ring::region_start;
use singlefs_core::system_configuration::{SystemConfiguration, SYSTEM_CONFIGURATION_MAGIC};
use singlefs_format::{
    DATA_UNIT_BYTES, DATA_UNIT_HEADER_BYTES, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
    JOURNAL_HEADER_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, NODE_BYTES,
    NONCE_MAC_ALGORITHM_RESERVED_BYTES, ROOT_RECORD_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_harness::memory_pool::{
    publishes_in, root_identity_of_write, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RecordStreamContinuity, RecordStreamSeam, RetainedWrite,
    WrittenContents, SECTOR_BYTES,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

use crate::crash_judge_conclusions::ClaimedVersionOfAGivenState;

use crate::crash::{
    crash_state_enumeration_tables_with_slices_choosing_the_torn_images, no_segment_slices,
    reuse_is_not_proven_illegal_by_the_reclaim_predicate, CrashPointReplaySegmentExpansion,
    CrashPointReplaySegmentSlice, CrashPointReplayVersionTable, CrashStateEnumerationTables,
    TornImagesOfTheInPlaceOverwrites,
};
use crate::crash_facts::{
    check_the_slice_starts_against_the_expansions,
    persisted_writes_of_state_by_the_segment_expansions, segment_expansion_facts,
    segment_facts_of_the_tables, SegmentExpansionFact, SegmentFact, TornImageFact,
    MAXIMUM_ENUMERATED_WRITES,
};

/// 一个落点最多被几次写盖住：GPU 内核里组合掩码是 [`COMBINATION_WORDS_OF_A_VERSION`] 个 u32（[`CombinationOfTheWritesOnALocation`]，
/// 版本表里的布局见 [`version_word_of_the_combination_word`]）。随机历史越过原分配记录墙那一族把系统配置 0 号槽录制写了 88 到 111 次
/// （2026-10-18 调查员量的，全量那一趟建不出判表），64 位装不下；256 留出余量。清零写进落点模型之后用过的盘上重做 mkfs 那条流
/// 每个落点至多 4 次（系统配置槽：[0, 根环基址) 清零、系统配置槽写与两者的撕裂镜像；根槽：区域清零、它的撕裂镜像、根槽写；
/// journal 记录槽：环清零与它的撕裂镜像。推的，读 mkfs 写序与撕裂镜像的建法），不动这个上限。
pub const MAXIMUM_WRITES_PER_LOCATION: usize = 256;
/// 组合掩码按 u64 几个字（[`CombinationOfTheWritesOnALocation`] 的存法）。
const COMBINATION_U64_WORDS: usize = MAXIMUM_WRITES_PER_LOCATION / 64;
const _: () = assert!(COMBINATION_U64_WORDS * 64 == MAXIMUM_WRITES_PER_LOCATION);
/// 组合掩码按 u32 几个字（版本表与 GPU 内核里的字数；内核里的同名常量由 `crate::crash_judge_gpu` 从这里生成）。
pub const COMBINATION_WORDS_OF_A_VERSION: usize = MAXIMUM_WRITES_PER_LOCATION / 32;
const _: () = assert!(COMBINATION_WORDS_OF_A_VERSION >= 2);
/// 一个落点最多枚举几种版本：越过就是这条流的落点太复杂，GPU 判不了（先报错，不悄悄漏版本）。
pub const MAXIMUM_VERSIONS_PER_LOCATION: usize = 4096;
/// 一个落点最长 32 KiB：64 个扇区，逐扇区等不等的掩码是一个 u64。
pub const MAXIMUM_LOCATION_SECTORS: usize = 64;

/// 落点是哪一类结构槽。封闭集合，`match` 不写通配臂；GPU 表里的码就是 [`JudgeLocationKind::code`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JudgeLocationKind {
    /// 系统配置槽 4096：每盘两个（偏移 0 与固定结构槽距）。
    SystemConfiguration,
    /// 根环槽：宽 `physical_block_size`，区域起点 + 槽号 × 固定结构槽距。
    RootSlot,
    /// journal 记录槽 4096：环起点 + 记录序号 × 4096。
    JournalRecord,
    /// 单元区里一个 16 KiB 槽的起点，读 32 KiB（码 2 只读前 16 KiB）；盘尾不够 32 KiB 时只有 16 KiB。
    UnitSlot,
}

impl JudgeLocationKind {
    #[must_use]
    pub fn code(self) -> u32 {
        match self {
            JudgeLocationKind::SystemConfiguration => 0,
            JudgeLocationKind::RootSlot => 1,
            JudgeLocationKind::JournalRecord => 2,
            JudgeLocationKind::UnitSlot => 3,
        }
    }
}

/// 枚举写表里一次写的种类，GPU 表里的码。
#[must_use]
pub fn step_kind_code(kind: StepKind) -> u32 {
    match kind {
        StepKind::ZeroFill => 0,
        StepKind::UnitWrite => 1,
        StepKind::JournalRecord => 2,
        StepKind::RootRecordFua => 3,
        StepKind::SystemConfigurationSlot => 4,
        StepKind::Barrier => 5,
        // 新码接在最后，旧的几个码不动。discard 与清零写都不落在任何一个落点上（[`JudgeWrite::location`] 是 `None`）：它罩住的每个落点
        // 把它算进自己的写里，落点的每个版本按 `CrashImage::read` 同一条叠法叠出来（持久那一态读回 0、撕裂那一态是撕裂镜像的字节、
        // 没持久那一态是旧字节，取几态照 D13（验证路线） 已定项 4），内核只按组合查版本，不按这个码分支。
        StepKind::Discard => 6,
    }
}

/// 盖住一个落点的那几次写各自持久没持久：第 i 位 = 落点的 `writes[i]` 持久（i < [`MAXIMUM_WRITES_PER_LOCATION`]，建表时核过）。
/// 按 u64 存、低位字在前；比大小按它当一个无符号整数比（落点的版本表按它升序）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CombinationOfTheWritesOnALocation {
    words: [u64; COMBINATION_U64_WORDS],
}

impl CombinationOfTheWritesOnALocation {
    /// 哪一次写都没持久。
    pub const NOTHING_PERSISTED: Self = Self {
        words: [0; COMBINATION_U64_WORDS],
    };

    /// 落点的第 `position` 次写记成持久。
    ///
    /// # Panics
    /// `position` 不小于 [`MAXIMUM_WRITES_PER_LOCATION`]：建表时已拒多于它的落点（[`JudgeTablesRefusal::TooManyWritesOnOneLocation`]），走到这里是没经那一判。
    pub fn mark_persisted(&mut self, position: usize) {
        assert!(
            position < MAXIMUM_WRITES_PER_LOCATION,
            "落点的第 {position} 次写：建表时已核过一个落点不多于 {MAXIMUM_WRITES_PER_LOCATION} 次写"
        );
        self.words[position / 64] |= 1u64 << (position % 64);
    }

    /// 落点的第 `position` 次写持久没持久。
    ///
    /// # Panics
    /// 同 [`Self::mark_persisted`]。
    #[must_use]
    pub fn is_persisted(&self, position: usize) -> bool {
        assert!(
            position < MAXIMUM_WRITES_PER_LOCATION,
            "落点的第 {position} 次写：建表时已核过一个落点不多于 {MAXIMUM_WRITES_PER_LOCATION} 次写"
        );
        self.words[position / 64] & (1u64 << (position % 64)) != 0
    }

    /// 按 u32 的字，低位字在前（第 k 字是第 32k 到 32k + 31 次写）：打包进版本表、与 GPU 内核比的就是这几个字。
    #[must_use]
    pub fn thirty_two_bit_words(&self) -> [u32; COMBINATION_WORDS_OF_A_VERSION] {
        let mut thirty_two_bit_words = [0u32; COMBINATION_WORDS_OF_A_VERSION];
        for (word_index, word) in self.words.iter().enumerate() {
            thirty_two_bit_words[2 * word_index] =
                u32::try_from(word & 0xFFFF_FFFF).expect("低 32 位");
            thirty_two_bit_words[2 * word_index + 1] = u32::try_from(word >> 32).expect("高 32 位");
        }
        thirty_two_bit_words
    }
}

impl Ord for CombinationOfTheWritesOnALocation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.words.iter().rev().cmp(other.words.iter().rev())
    }
}

impl PartialOrd for CombinationOfTheWritesOnALocation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// 只取前 64 次写那几位：掩码在第 64 位往上全是 0，与整个组合按位与，就是与它最低那个 u64 字按位与（对拍用例看低几位用）。
impl std::ops::BitAnd<u64> for CombinationOfTheWritesOnALocation {
    type Output = u64;

    fn bitand(self, mask_of_the_first_sixty_four_writes: u64) -> u64 {
        self.words[0] & mask_of_the_first_sixty_four_writes
    }
}

/// 一个落点在某个持久组合下呈现的字节与 CPU 先算好的几样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgingUnitsFingerprint {
    /// 第 i 位 = 这个落点的 `writes[i]` 在这个组合里持久。
    pub combination: CombinationOfTheWritesOnALocation,
    /// 这个落点整段的字节（长度 = 落点长度）。
    pub bytes: Vec<u8>,
    /// 按落点种类各有含义的四个检查字（[`version_checks`]）。
    pub checks: [u32; 4],
    /// 与落点的每次写（同序）逐扇区等不等：第 s 位 = 落点的第 s 个扇区上的字节就是那次写写下的。
    pub sector_equals: Vec<u64>,
}

/// 一个结构槽：在哪块盘、什么偏移、多长、哪一类、被枚举写表里哪几次写盖住（升序）、有哪几种版本。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgeLocation {
    pub device: u32,
    pub offset: u64,
    pub length: u32,
    pub kind: JudgeLocationKind,
    /// 盖住这个落点的枚举写表下标，升序。
    pub writes: Vec<u32>,
    pub versions: Vec<JudgingUnitsFingerprint>,
}

/// 枚举写表里的一次写，判器要的几样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgeWrite {
    pub device: u32,
    pub offset: u64,
    pub length: u32,
    pub kind: StepKind,
    /// 这次写落在哪个落点（落点表下标）；清零写与 discard（连同它们的撕裂镜像）不落在任何落点：discard 一次按 64 槽整段发、
    /// 清零写一次清一整段固定结构，都罩住几个到几十个落点，只进它罩住的那几个落点的写（[`JudgeLocation::writes`]）。
    pub location: Option<u32>,
    /// 根槽写写下的根身份（实例代号，checkpoint_txg）。
    pub root_identity: Option<(u32, u64)>,
    /// 根槽写的字节（记录核对器按恢复落到的那一版找它的根记录、读它的实例表）。
    pub root_bytes: Vec<u8>,
    /// 这次写是一份单元副本时，它之后与它重叠的每次写：（写表下标，过没过回收谓词，盖住这份副本的哪几个扇区）。
    pub later_overlapping_writes: Vec<LaterOverlappingWrite>,
}

/// 一份单元副本之后、与它重叠的一次写（记录核对器第二条判据的「更晚、过了回收谓词的写」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaterOverlappingWrite {
    pub write_index: u32,
    pub reuse_is_not_proven_illegal: bool,
    /// 这次写盖住副本的哪几个扇区（按副本自己的扇区序号）。
    pub covered_sectors: u64,
}

/// 一次发布在写表里的样子（`singlefs_harness::memory_pool::publishes_in`），单元写按偏移分组（两盘各一份副本）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgePublish {
    pub root_write: u32,
    pub instance: u32,
    pub checkpoint_txg: u64,
    pub records: Vec<u32>,
    /// 每组是同一个偏移上的几份副本（写表下标）。
    pub unit_copy_groups: Vec<Vec<u32>>,
}

/// 期望版本表里的一版：有文件（读回的要是 `chunks` 那几个单元），还是文件从这一版起没有了（`crash::CrashPointReplayVersionTable` 的两样，
/// D8（核心索引结构） 已定项 16 第 4 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileOfTheExpectedVersion {
    WithAFile,
    WithoutAFile,
}

/// 一个已发布版本的内容按数据单元切开之后，每个单元该有的载荷 CRC 与声明长度（与走读读出的内容比）；没有文件的那一版不切，`chunks` 是空的。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedVersionChunks {
    pub instance: u32,
    pub checkpoint_txg: u64,
    pub file: FileOfTheExpectedVersion,
    /// （载荷区 CRC-32C，与数据单元头里偏移 101 的那 4 字节同一个覆盖范围；声明长度）。
    pub chunks: Vec<(u32, u32)>,
}

/// GPU 判器一条流的全部输入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgeTables {
    pub recorded_write_count: u32,
    pub enumerated_write_count: u32,
    pub is_tearable: Vec<bool>,
    pub torn_images: Vec<TornImageFact>,
    pub segments: Vec<SegmentFact>,
    /// 每一段的段内序号怎么拆（与 `segments` 同序；段表第 6–8 字，判决 `research/prompts/m3-crash-enumeration-merge-r2-main-verification.md`
    /// 第二节 L6-2）。
    pub segment_expansions: Vec<SegmentExpansionFact>,
    /// 状态从哪来：按段展开拆序号，或一张点名的持久掩码清单（[`JudgeStateSource`]）。
    pub state_source: JudgeStateSource,
    pub state_count: u64,
    /// 池里的盘，按 `PoolReader::device_identities` 的次序。
    pub devices: Vec<u32>,
    pub device_bytes: u64,
    /// 单元区起点（字节）：基线里择出的那一份系统配置记的几何（[`system_configuration_the_locations_are_laid_out_by`]）。单元区里没有落点的
    /// 16 KiB 槽每个崩溃状态都是全零（[`ZERO_UNIT_VERSIONS`]）。
    pub unit_area_start_in_bytes: u64,
    pub judged_root_index: u32,
    pub writes: Vec<JudgeWrite>,
    /// 按 (盘, 偏移) 升序。
    pub locations: Vec<JudgeLocation>,
    pub publishes: Vec<JudgePublish>,
    pub expected_versions: Vec<ExpectedVersionChunks>,
}

/// 一张判表的状态从哪来。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JudgeStateSource {
    /// 序号按段展开拆成持久掩码（[`JudgeTables::segment_expansions`]），状态数是各段展开出来的加最后全部持久那一个。
    SegmentExpansions,
    /// 给定状态表（判决 `research/prompts/m3-crash-enumeration-merge-r2-main-verification.md` 第二节 L6-3：随机历史两项上 GPU 要的那一种展开）：
    /// 第 k 个状态就是第 k 个持久掩码（按枚举写表给，每个与枚举写表一样长），状态数是清单的长度；`claimed_versions` 与它逐个对应，
    /// 是第 k 个状态那一栏恢复自称的那一版（恢复内核记录核对器第二条判据照它取那一版，D13（验证路线） 已定项 7 第二截）。
    GivenPersistedSets {
        persisted_sets: Vec<Vec<bool>>,
        claimed_versions: Vec<ClaimedVersionOfAGivenState>,
    },
}

/// 建表要的一条流（与 `crash_amplification::CrashFlow` 同一组东西，不带崩溃点）。
pub struct JudgeFlow<'flow> {
    pub base: &'flow MemoryPool,
    pub writes: &'flow [RetainedWrite],
    pub segments: &'flow [Vec<usize>],
    pub judged_root_index: usize,
    pub versions: &'flow [PublishedVersion],
    pub expansion: &'flow dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
}

/// 建表时撞上的、GPU 判不了的形态。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JudgeTablesRefusal {
    /// 基线里每块盘的两个系统配置槽都解不开（[`system_configuration_the_locations_are_laid_out_by`] 一份都找不到）：落点几何无从取。
    BaseSystemConfigurationUnreadable,
    /// 一次写的起点不是任何一个落点的起点，或长过那个落点。
    WriteNotAlignedToALocation { write_index: usize },
    /// 一个落点被多于 [`MAXIMUM_WRITES_PER_LOCATION`] 次写盖住。
    TooManyWritesOnOneLocation {
        device: u32,
        offset: u64,
        writes: usize,
    },
    /// 一个落点的可达版本多于 [`MAXIMUM_VERSIONS_PER_LOCATION`]。
    TooManyVersionsOnOneLocation { device: u32, offset: u64 },
    /// 一次录制流里的写不在任何一段里。
    WriteOutsideEverySegment { write_index: usize },
    /// 枚举写表的写多于内核持久掩码的宽（[`MAXIMUM_ENUMERATED_WRITES`]）：掩码表满了。
    EnumeratedWritesPastTheMaskWidth { enumerated_writes: usize },
    /// 一段按它的展开方式展开出的状态数装不进 u64（段内序号拆不开），或与它的序号区间对不上：段表核段长没过；分片的段是片起点与它的
    /// 各位对不上，或片越过了段的域（`crate::crash_facts::check_the_slice_starts_against_the_expansions`）。
    SegmentStateCountNotRepresentable { segment_index: usize },
    /// 打包之后这张表的起点或长度装不进目录的 u32：这张表满了（[`JudgeTablesWords::every_table_name`] 里的名字）。
    PackedTablePastThirtyTwoBitOffsets {
        table: &'static str,
        words_up_to_its_end: u64,
    },
    /// 点名的持久掩码清单里第 `position` 个与枚举写表不一样长。
    GivenPersistedSetOfAnotherLength {
        position: usize,
        length: usize,
        enumerated_writes: usize,
    },
    /// 给定状态表的第 `seam_index` 道接缝比前一道早、或越过录制流（[`build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams`]）。
    SeamsNotInOrderWithinTheRecordedWrites { seam_index: usize },
    /// 第 `seam_index` 道是掉电接缝，清单里第 `position` 个掩码在它之前那一截没全落（被判的状态要落在掉电之后）。
    GivenPersistedSetNotWhollyPersistedBeforeAPowerLossSeam { position: usize, seam_index: usize },
    /// 崩溃接缝之前那一截里写表下标 `root_write` 的根槽写，清单里第 `position` 个掩码与第 0 个落没落不一样：
    /// 一张表的发布表只能按一种剔法分，这几个状态要分到不同的表里。
    GivenPersistedSetsDisagreeOnARootWriteBeforeACrashSeam { root_write: usize, position: usize },
}

fn slot_bytes() -> u64 {
    SLOT_BYTES
}

/// 一次写每个扇区的落地：与崩溃点重放一致（0 没持久、1 撕裂、2 持久）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Landing {
    NotPersisted,
    Torn,
    Persisted,
}

/// 从一条流建全部表（文件从不消失的流：没有「没有文件的那几版」）。
///
/// # Errors
/// [`JudgeTablesRefusal`] 的各成员。
///
/// # Panics
/// 枚举写表里的写越过盘尾（录制器只录落下了的写）。
pub fn build_judge_tables(flow: &JudgeFlow<'_>) -> Result<JudgeTables, JudgeTablesRefusal> {
    build_judge_tables_against_the_version_table(flow, &[])
}

/// 同 [`build_judge_tables`]，期望版本表另带文件从那一版起没有了的那几版（删除流，`crash_amplification::CrashFlow::versions_without_a_file`）：
/// 恢复内核的 oracle 照 `crash::classified_oracle_violation_for_the_version_table` 认它们（`crash_judge_recovery.wgsl` 的 `classify_oracle`）。
///
/// # Errors
/// [`JudgeTablesRefusal`] 的各成员。
///
/// # Panics
/// 枚举写表里的写越过盘尾；同一个根既在 `flow.versions` 里又在 `versions_without_a_file` 里（[`CrashPointReplayVersionTable::new`]）。
pub fn build_judge_tables_against_the_version_table(
    flow: &JudgeFlow<'_>,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
) -> Result<JudgeTables, JudgeTablesRefusal> {
    build_judge_tables_against_the_version_table_with_slices(
        flow,
        versions_without_a_file,
        &no_segment_slices(flow.segments.len()),
    )
}

/// 同 [`build_judge_tables_against_the_version_table`]，分片的段照 `segment_slices`（与段同序，C790）：段表里它的序号区间是片长、
/// 带片起点拆成的各位（内核从那几位起加片内序号、逐位进位）。
///
/// # Errors
/// 同 [`build_judge_tables_against_the_version_table`]。
///
/// # Panics
/// 同 [`build_judge_tables_against_the_version_table`]；片越过了段的域（`crate::crash` 建计划时停下）。
pub fn build_judge_tables_against_the_version_table_with_slices(
    flow: &JudgeFlow<'_>,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
) -> Result<JudgeTables, JudgeTablesRefusal> {
    build_judge_tables_from_the_state_source(
        flow,
        versions_without_a_file,
        JudgeStateSource::SegmentExpansions,
        &[],
        PublishesSplitFrom::TheEnumeratedWrites,
        segment_slices,
        TornImagesOfTheInPlaceOverwrites::Appended,
    )
}

/// GPU 的发布表从哪一张写表分（[`judge_publishes`]）：与同一批状态上 CPU 记录核对器拿到的那一张相同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PublishesSplitFrom {
    /// 枚举写表（录制流接撕裂镜像与重放）：崩溃点重放的 CPU 记录核对器（`crash::check_records`）拿的就是它。
    TheEnumeratedWrites,
    /// 只取录制流：随机历史的 CPU 记录核对器（`crash_injection::judge_a_given_state_on_the_cpu`）拿的写表不带撕裂镜像与重放。
    /// 枚举写表接在后面的撕裂根槽写分不出另一次发布：mkfs 的根槽写叠在 mkfs 之后的基线上（创世发布进记录核对器，用户 2026-10-10 定），
    /// 撕出来与原字节相同，接成一次发布就把录制流末尾没等到根槽写的单元写与记录写归给它。
    TheRecordedWritesOnly,
}

/// 给定状态表（[`JudgeStateSource::GivenPersistedSets`]）：GPU 判 `given_persisted_sets` 里的每一个持久掩码（按枚举写表给，第 k 个是序号 k 的状态），
/// 不按段展开拆序号。落点的版本只取这几个掩码叠得出来的那几种；枚举写表照 `flow` 的段建（撕裂镜像与重放接在哪由段定）。
/// 期望版本表同 [`build_judge_tables_against_the_version_table`]。发布表与崩溃点重放一样从枚举写表分（掩码可以点名撕裂态，对的是崩溃点重放的 CPU 判器）。
///
/// # Errors
/// [`JudgeTablesRefusal`] 的各成员；掩码与枚举写表不一样长是 [`JudgeTablesRefusal::GivenPersistedSetOfAnotherLength`]。
///
/// # Panics
/// 同 [`build_judge_tables_against_the_version_table`]。
pub fn build_judge_tables_for_the_given_persisted_sets(
    flow: &JudgeFlow<'_>,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    given_persisted_sets: Vec<Vec<bool>>,
) -> Result<JudgeTables, JudgeTablesRefusal> {
    let claimed_versions = vec![
        ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery;
        given_persisted_sets.len()
    ];
    build_judge_tables_from_the_state_source(
        flow,
        versions_without_a_file,
        JudgeStateSource::GivenPersistedSets {
            persisted_sets: given_persisted_sets,
            claimed_versions,
        },
        &[],
        PublishesSplitFrom::TheEnumeratedWrites,
        &no_segment_slices(flow.segments.len()),
        TornImagesOfTheInPlaceOverwrites::Appended,
    )
}

/// 同 [`build_judge_tables_for_the_given_persisted_sets`]，录制流在 `seams` 那几道接缝处断开（随机历史的第一截在掉电之后、第三截在
/// 历史与挂载之间；用户 2026-10-10 定「第三截也上 GPU，GPU 的发布表认接缝」）：发布表照记录核对器同一个分法
/// （`singlefs_harness::memory_pool::publishes_in` 的 `RecordStreamContinuity::BrokenAtSeams`）分，内核照旧只读发布表，所以接缝怎么断
/// 在建表这一刻定死。崩溃那道接缝之前那一截剔哪几次发布要看那几次的根槽写落没落，这一张表里的每个持久掩码在那几个根槽写上都要一样
/// （随机历史一张表就是一个第一次崩溃状态上的二次崩溃，历史那一截都是同一个掩码）；掉电那道接缝之前那一截要在每个掩码里全落了。
/// `seams` 是空的就是一条录制流。发布表只从录制流分（随机历史的 CPU 记录核对器拿的写表不带撕裂镜像与重放，[`PublishesSplitFrom`]），
/// 这一点与 [`build_judge_tables_for_the_given_persisted_sets`] 不同。
///
/// # Errors
/// 同 [`build_judge_tables_for_the_given_persisted_sets`]；接缝不按录制流下标从小到大或越过录制流
/// （[`JudgeTablesRefusal::SeamsNotInOrderWithinTheRecordedWrites`]）；掉电接缝之前有一个掩码没全落
/// （[`JudgeTablesRefusal::GivenPersistedSetNotWhollyPersistedBeforeAPowerLossSeam`]）；崩溃接缝之前某个根槽写在两个掩码里一个落一个没落
/// （[`JudgeTablesRefusal::GivenPersistedSetsDisagreeOnARootWriteBeforeACrashSeam`]）。
///
/// # Panics
/// 同 [`build_judge_tables_against_the_version_table`]。
pub fn build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams(
    flow: &JudgeFlow<'_>,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    given_persisted_sets: Vec<Vec<bool>>,
    seams: &[RecordStreamSeam],
) -> Result<JudgeTables, JudgeTablesRefusal> {
    let claimed_versions = vec![
        ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery;
        given_persisted_sets.len()
    ];
    build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_with_their_claimed_versions(
        flow,
        versions_without_a_file,
        given_persisted_sets,
        &claimed_versions,
        seams,
    )
}

/// 同 [`build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams`]，每个状态另带那一栏恢复自称的那一版（`claimed_versions`，
/// 与 `given_persisted_sets` 逐个对应；崩溃注入第二截那张表里挂载之后的池那个状态写挂载与那次发布写出的最新那条根）：进判表
/// （[`JudgeStateSource::GivenPersistedSets`] 的 `claimed_versions`，打包成 `claimed_versions` 那一张），恢复内核照它判第二条判据。
///
/// # Errors
/// 同 [`build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams`]。
///
/// # Panics
/// 同 [`build_judge_tables_against_the_version_table`]；`claimed_versions` 与 `given_persisted_sets` 不一样长（建表的一方写错了）。
pub fn build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_with_their_claimed_versions(
    flow: &JudgeFlow<'_>,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    given_persisted_sets: Vec<Vec<bool>>,
    claimed_versions: &[ClaimedVersionOfAGivenState],
    seams: &[RecordStreamSeam],
) -> Result<JudgeTables, JudgeTablesRefusal> {
    build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_choosing_the_torn_images(
        flow,
        versions_without_a_file,
        given_persisted_sets,
        claimed_versions,
        seams,
        TornImagesOfTheInPlaceOverwrites::Appended,
    )
}

/// 同 [`build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_with_their_claimed_versions`]，枚举写表不接撕裂镜像与重放
/// （[`TornImagesOfTheInPlaceOverwrites::LeftOut`]）：枚举写表就是录制流，每个持久掩码按录制流给（与 `flow.writes` 一样长）。
/// 随机历史每步的镜像从不摆撕裂态，接上的撕裂镜像在每个掩码里都没落，却照样占掩码一位、照样记进它罩住的落点的写数
/// （系统配置槽每次轮换都是一次可撕的原地覆写，那一槽的写数因此翻倍）；不接它们，判表上的每个状态叠出的镜像不变。
///
/// # Errors
/// 同 [`build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams`]；掩码与录制流不一样长是
/// [`JudgeTablesRefusal::GivenPersistedSetOfAnotherLength`]。
///
/// # Panics
/// 同 [`build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_with_their_claimed_versions`]。
pub fn build_judge_tables_for_the_given_persisted_sets_of_the_recorded_writes_broken_at_the_seams_with_their_claimed_versions(
    flow: &JudgeFlow<'_>,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    given_persisted_sets: Vec<Vec<bool>>,
    claimed_versions: &[ClaimedVersionOfAGivenState],
    seams: &[RecordStreamSeam],
) -> Result<JudgeTables, JudgeTablesRefusal> {
    build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_choosing_the_torn_images(
        flow,
        versions_without_a_file,
        given_persisted_sets,
        claimed_versions,
        seams,
        TornImagesOfTheInPlaceOverwrites::LeftOut,
    )
}

/// 带接缝的给定状态表那两个入口共用的本体：撕裂镜像接不接照 `torn_images`。
fn build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_choosing_the_torn_images(
    flow: &JudgeFlow<'_>,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    given_persisted_sets: Vec<Vec<bool>>,
    claimed_versions: &[ClaimedVersionOfAGivenState],
    seams: &[RecordStreamSeam],
    torn_images: TornImagesOfTheInPlaceOverwrites,
) -> Result<JudgeTables, JudgeTablesRefusal> {
    assert_eq!(
        claimed_versions.len(),
        given_persisted_sets.len(),
        "恢复自称的那一版那一栏与持久掩码逐个对应"
    );
    build_judge_tables_from_the_state_source(
        flow,
        versions_without_a_file,
        JudgeStateSource::GivenPersistedSets {
            persisted_sets: given_persisted_sets,
            claimed_versions: claimed_versions.to_vec(),
        },
        seams,
        PublishesSplitFrom::TheRecordedWritesOnly,
        &no_segment_slices(flow.segments.len()),
        torn_images,
    )
}

/// `seams_of_the_given_sets` 只在给定状态表上可以不空（[`build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams`]）；
/// 按段展开那一路恒给空表（崩溃点重放的流是一条录制流）。`publishes_split_from` 说发布表从哪一张写表分（[`PublishesSplitFrom`]）。
/// `segment_slices` 与段同序（分片的段，C790）；给定状态表那两路恒不分片。`torn_images` 说枚举写表接不接撕裂镜像
/// （按段展开那一路恒接：它的状态里有撕裂态）。
fn build_judge_tables_from_the_state_source(
    flow: &JudgeFlow<'_>,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    state_source: JudgeStateSource,
    seams_of_the_given_sets: &[RecordStreamSeam],
    publishes_split_from: PublishesSplitFrom,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
    torn_images: TornImagesOfTheInPlaceOverwrites,
) -> Result<JudgeTables, JudgeTablesRefusal> {
    let _version_table_without_contradictions =
        CrashPointReplayVersionTable::new(flow.versions, versions_without_a_file);
    let tables: CrashStateEnumerationTables =
        crash_state_enumeration_tables_with_slices_choosing_the_torn_images(
            flow.base,
            flow.writes,
            flow.segments,
            flow.expansion,
            segment_slices,
            torn_images,
        );
    let enumerated = &tables.enumerated_writes;
    if enumerated.len() > MAXIMUM_ENUMERATED_WRITES {
        return Err(JudgeTablesRefusal::EnumeratedWritesPastTheMaskWidth {
            enumerated_writes: enumerated.len(),
        });
    }
    if let JudgeStateSource::GivenPersistedSets {
        persisted_sets: given_persisted_sets,
        claimed_versions: _,
    } = &state_source
    {
        if let Some((position, persisted)) = given_persisted_sets
            .iter()
            .enumerate()
            .find(|(_, persisted)| persisted.len() != enumerated.len())
        {
            return Err(JudgeTablesRefusal::GivenPersistedSetOfAnotherLength {
                position,
                length: persisted.len(),
                enumerated_writes: enumerated.len(),
            });
        }
    }
    let devices: Vec<u32> = flow
        .base
        .device_identities()
        .into_iter()
        .map(|device| device.0)
        .collect();
    let device_bytes = flow
        .base
        .device_size_in_bytes(flow.base.device_identities()[0])
        .expect("池里第一块盘说得出字节数");
    let geometry = base_geometry(flow.base)?;
    let mut locations =
        enumerate_locations(flow.base, enumerated, &devices, device_bytes, &geometry);
    locations.sort_by_key(|location| (location.device, location.offset));
    // 每次写盖住哪几个落点
    for (write_index, write) in enumerated.iter().enumerate() {
        let write_start = write.offset.0;
        let write_end = write_start + write.length_in_bytes();
        for location in &mut locations {
            if location.device != write.device.0 {
                continue;
            }
            let location_end = location.offset + u64::from(location.length);
            if location.offset < write_end && write_start < location_end {
                location
                    .writes
                    .push(u32::try_from(write_index).expect("写表下标装得进 u32"));
            }
        }
    }
    for location in &locations {
        if location.writes.len() > MAXIMUM_WRITES_PER_LOCATION {
            return Err(JudgeTablesRefusal::TooManyWritesOnOneLocation {
                device: location.device,
                offset: location.offset,
                writes: location.writes.len(),
            });
        }
    }
    // 每次录制流里的写在哪一段
    let mut segment_of_write: Vec<Option<usize>> = vec![None; flow.writes.len()];
    for (segment_index, segment) in flow.segments.iter().enumerate() {
        for write_index in segment {
            segment_of_write[*write_index] = Some(segment_index);
        }
    }
    for (write_index, segment) in segment_of_write.iter().enumerate() {
        if segment.is_none() {
            return Err(JudgeTablesRefusal::WriteOutsideEverySegment { write_index });
        }
    }
    let segment_of_write: Vec<usize> = segment_of_write
        .into_iter()
        .map(|segment| segment.expect("上面判过每次写都在一段里"))
        .collect();
    let extended = ExtendedWriteOrigins::of(&tables, flow.writes.len());
    let expanded_segments: BTreeSet<usize> = tables
        .state_ranges_by_segment
        .iter()
        .enumerate()
        .filter(|(_, states)| !states.is_empty())
        .map(|(index, _)| index)
        .collect();
    for location in &mut locations {
        let combinations = match &state_source {
            JudgeStateSource::SegmentExpansions => reachable_combinations_of_the_expanded_segments(
                &segment_of_write,
                &extended,
                flow.segments.len(),
                &expanded_segments,
                location,
            )?,
            JudgeStateSource::GivenPersistedSets {
                persisted_sets: given_persisted_sets,
                claimed_versions: _,
            } => combinations_of_the_given_persisted_sets(location, given_persisted_sets)?,
        };
        push_the_versions_of_the_combinations(flow.base, enumerated, location, combinations);
    }
    locations.retain(|location| location_is_read_by_the_judge(location, &geometry));
    // 下标按去掉之后的落点表算（写表、GPU 的落点下标都指这一份）
    let location_index_by_start: BTreeMap<(u32, u64), usize> = locations
        .iter()
        .enumerate()
        .map(|(index, location)| ((location.device, location.offset), index))
        .collect();
    let writes = judge_writes(
        enumerated,
        flow.writes,
        &locations,
        &location_index_by_start,
    )?;
    let writes_the_publishes_are_split_from: &[RetainedWrite] = match publishes_split_from {
        PublishesSplitFrom::TheEnumeratedWrites => enumerated,
        PublishesSplitFrom::TheRecordedWritesOnly => &enumerated[..flow.writes.len()],
    };
    let publishes = judge_publishes(
        writes_the_publishes_are_split_from,
        flow.writes.len(),
        &state_source,
        seams_of_the_given_sets,
    )?;
    let expected_versions = flow
        .versions
        .iter()
        .map(expected_version_chunks)
        .chain(
            versions_without_a_file
                .iter()
                .map(|version| ExpectedVersionChunks {
                    instance: version.instance.0,
                    checkpoint_txg: version.checkpoint_txg.0,
                    file: FileOfTheExpectedVersion::WithoutAFile,
                    chunks: Vec::new(),
                }),
        )
        .collect();
    let segments: Vec<SegmentFact> =
        segment_facts_of_the_tables(flow.writes, flow.segments, flow.expansion, &tables);
    let segment_expansions = segment_expansion_facts(flow.writes, flow.segments, flow.expansion);
    // 核段长：不分片的段按它的展开方式现算的状态数装得进 u64、与它的序号区间相同（内核按这一份拆满 64 位的段内序号）；
    // 分片的段核片起点与它的各位对得上、片没越过段的域（内核从片起点的各位起加片内序号）
    for (segment_index, (segment, expansion)) in
        segments.iter().zip(&segment_expansions).enumerate()
    {
        if segment.slice_start.is_none()
            && expansion.state_count(segment, &tables.is_tearable_by_write)
                != Some(segment.states.end - segment.states.start)
        {
            return Err(JudgeTablesRefusal::SegmentStateCountNotRepresentable { segment_index });
        }
    }
    check_the_slice_starts_against_the_expansions(
        &segments,
        &segment_expansions,
        &tables.is_tearable_by_write,
    )
    .map_err(
        |segment_index| JudgeTablesRefusal::SegmentStateCountNotRepresentable { segment_index },
    )?;
    let state_count = match &state_source {
        JudgeStateSource::SegmentExpansions => tables.state_count,
        JudgeStateSource::GivenPersistedSets {
            persisted_sets: given_persisted_sets,
            claimed_versions: _,
        } => u64::try_from(given_persisted_sets.len()).expect("清单长装得进 u64"),
    };
    let judge_tables = JudgeTables {
        recorded_write_count: u32::try_from(flow.writes.len()).expect("写数装得进 u32"),
        enumerated_write_count: u32::try_from(enumerated.len()).expect("写数装得进 u32"),
        is_tearable: tables.is_tearable_by_write.clone(),
        torn_images: tables
            .torn_images
            .iter()
            .map(|torn| TornImageFact {
                write_index: u32::try_from(torn.write_index).expect("下标装得进 u32"),
                torn_image_index: u32::try_from(torn.torn_image_index).expect("下标装得进 u32"),
                replays: torn
                    .replays
                    .iter()
                    .map(|(replay_index, replayed)| {
                        (
                            u32::try_from(*replay_index).expect("下标装得进 u32"),
                            u32::try_from(*replayed).expect("下标装得进 u32"),
                        )
                    })
                    .collect(),
            })
            .collect(),
        segments,
        segment_expansions,
        state_source,
        state_count,
        devices,
        device_bytes,
        unit_area_start_in_bytes: geometry.unit_area_start_slot * slot_bytes(),
        judged_root_index: u32::try_from(flow.judged_root_index).expect("下标装得进 u32"),
        writes,
        locations,
        publishes,
        expected_versions,
    };
    judge_tables.check_the_packed_directory()?;
    Ok(judge_tables)
}

/// 基线里择出的那一份系统配置记的几何（[`system_configuration_the_locations_are_laid_out_by`]）：落点从它来。
struct BaseGeometry {
    fixed_structure_slot_spacing: u64,
    physical_block_size: u64,
    root_ring_slots_per_region: u64,
    region_devices: [u32; 3],
    journal_ring_bytes: u64,
    unit_area_start_slot: u64,
}

/// 基线里 `device` 上偏移 `offset` 那一个系统配置槽自证得过（`SystemConfiguration::parse_slot` 交 Ok）时解出的那一份；读不出、解不开交 None。
fn self_verified_system_configuration_slot_at(
    base: &MemoryPool,
    device: singlefs_core::address::DeviceIdentity,
    offset: u64,
) -> Option<SystemConfiguration> {
    let slot_bytes_of_system_configuration =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let bytes = PoolReader::read(
        base,
        device,
        DeviceOffsetInBytes(offset),
        slot_bytes_of_system_configuration,
    )?;
    SystemConfiguration::parse_slot(&bytes).ok()
}

/// 格式允许的每一档槽距上槽 1 的偏移（槽 i 在 i × 槽距，槽 1 的偏移就是槽距）：不小于 [`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`]、
/// 扇区宽 [`SECTOR_BYTES`] 的整数倍、槽 1 整槽落在根环基址之前，同 core 的 `slot_one_found_by_trying_every_slot_spacing_the_format_allows`
/// 与 GPU 两个内核逐档读槽 1 的那两个循环（`slot_one_by_trying_every_spacing`、`checker_slot_one_by_reading_every_spacing`）。
/// (1 MiB − 4096 − 4096) ÷ 512 + 1 = 2033 档。择落点几何（[`system_configuration_the_locations_are_laid_out_by`]）与建系统配置落点
/// （[`enumerate_locations`]）都按这一份试。
fn slot_one_offsets_of_every_spacing_the_format_allows() -> impl Iterator<Item = u64> {
    let largest_spacing = region_start(0).0 - SYSTEM_CONFIGURATION_SLOT_BYTES;
    (FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES..=largest_spacing)
        .step_by(usize::try_from(SECTOR_BYTES).expect("512"))
}

/// 落点几何取基线里哪一份系统配置：恢复择得出就取恢复择的那一份（`singlefs_core::recovery::choose_system_configuration`：每块盘的槽 0、
/// 槽 1 比世代号，全池没有一个槽 0 可择时逐档读槽 1、取世代号最大的；GPU 两个内核逐档读时也取世代号最大的，`crash_judge_common.wgsl` 的
/// `slot_one_by_trying_every_spacing`），几何与 GPU 读根槽的落点同源（hand_out 第二位报告「交主 agent 定的」第 2 条：取第一份自证得过的、
/// 同一块盘上另有一份记别的槽距、世代号更大的槽时两边几何不同）。恢复择不出（整池拒：带读者不收之值、每区槽数越界）时退回下面那一种找法，
/// 取第一份自证得过的；一份都没有才交 None。CPU 恢复与池级 checker 判得了的镜像（一块盘两槽全废、或每块盘的槽 0 都抹掉而槽 1 在），
/// GPU 判表照样建得出。次序：先按池里盘的次序读每块盘的槽 0；全池没有一个槽 0 自证得过时槽距无处可读，按池里盘的次序在格式允许的每一档槽距
/// （[`slot_one_offsets_of_every_spacing_the_format_allows`]）上各读一槽，只收自己记的槽距正好等于它所在偏移的那一槽。
/// 几何那几样（槽距、根槽宽、每区槽数、区域盘、环长）是池级不可变字段，哪块盘哪一槽交出来都一样；这里只取几何，不判读者收不收那几个池级值
/// （那一判在 GPU 恢复内核与池级 checker 内核里，与 CPU 同判）：带读者不收之值的那一份照样取它的几何——读者遇到它整池拒（恢复报
/// `SystemConfigurationValueRefused`、池级 checker 只交 I-7.13），几何不进结论；跳过它去取更远一档槽距上的槽，那一份记的槽距与写表里
/// 根槽写的落点对不上，判表反而建不出（`WriteNotAlignedToALocation`）。几何取哪一份都不决定 GPU 读不读得到逐档那几处槽：
/// 那几处另建落点（[`enumerate_locations`]）。
fn system_configuration_the_locations_are_laid_out_by(
    base: &MemoryPool,
) -> Option<SystemConfiguration> {
    if let Ok(chosen_by_the_recovery_reader) =
        singlefs_core::recovery::choose_system_configuration(base)
    {
        return Some(chosen_by_the_recovery_reader);
    }
    let devices = PoolReader::device_identities(base);
    if let Some(slot_zero) = devices
        .iter()
        .find_map(|device| self_verified_system_configuration_slot_at(base, *device, 0))
    {
        return Some(slot_zero);
    }
    // 迭代上界：盘数 × 档数（2033 档）；找到第一份就交回
    devices.iter().find_map(|device| {
        slot_one_offsets_of_every_spacing_the_format_allows().find_map(|candidate_spacing| {
            self_verified_system_configuration_slot_at(base, *device, candidate_spacing).filter(
                |slot_one| {
                    u64::from(slot_one.immutable.sizes.fixed_structure_slot_spacing)
                        == candidate_spacing
                },
            )
        })
    })
}

fn base_geometry(base: &MemoryPool) -> Result<BaseGeometry, JudgeTablesRefusal> {
    let system_configuration = system_configuration_the_locations_are_laid_out_by(base)
        .ok_or(JudgeTablesRefusal::BaseSystemConfigurationUnreadable)?;
    let sizes = &system_configuration.immutable.sizes;
    Ok(BaseGeometry {
        fixed_structure_slot_spacing: u64::from(sizes.fixed_structure_slot_spacing),
        physical_block_size: u64::from(sizes.physical_block_size),
        root_ring_slots_per_region: sizes.root_ring_slots_per_region.count(),
        region_devices: system_configuration
            .immutable
            .region_devices
            .map(|device| device.0),
        journal_ring_bytes: sizes.journal_ring_bytes,
        unit_area_start_slot: slot_after_the_journal_ring(sizes.journal_ring_bytes).0,
    })
}

/// 全部落点（还没填写与版本）：系统配置每盘两槽、根环全部槽、journal 环里基线或枚举写碰过的记录槽、单元区里基线或枚举写碰过的 16 KiB 槽；
/// 「碰过」不算清零写（[`StepKind::ZeroFill`]，连同它的撕裂镜像），理由在碰过的范围那一段。
/// 只在池里的盘（`devices`）上建落点：系统配置记的区域盘可以不在池里（卸载掉电之后只剩一块盘的镜像，丢掉的那块住着根环的两个区域），
/// CPU 的读者读那块盘交 None（`MemoryPool::read`，恢复与池级 checker 都当这一槽读不出），GPU 在那块盘上找不到落点、`read_at` 交没有版本的切片，
/// 同一个结局；给那块盘建落点就要从基线读它的字节，读不出。
fn enumerate_locations(
    base: &MemoryPool,
    enumerated: &[RetainedWrite],
    devices: &[u32],
    device_bytes: u64,
    geometry: &BaseGeometry,
) -> Vec<JudgeLocation> {
    let mut locations: Vec<JudgeLocation> = Vec::new();
    let mut starts: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut push = |device: u32, offset: u64, length: u64, kind: JudgeLocationKind| {
        if !devices.contains(&device)
            || offset + length > device_bytes
            || !starts.insert((device, offset))
        {
            return;
        }
        locations.push(JudgeLocation {
            device,
            offset,
            length: u32::try_from(length).expect("落点长度装得进 u32"),
            kind,
            writes: Vec::new(),
            versions: Vec::new(),
        });
    };
    for device in devices {
        push(
            *device,
            0,
            SYSTEM_CONFIGURATION_SLOT_BYTES,
            JudgeLocationKind::SystemConfiguration,
        );
        push(
            *device,
            geometry.fixed_structure_slot_spacing,
            SYSTEM_CONFIGURATION_SLOT_BYTES,
            JudgeLocationKind::SystemConfiguration,
        );
    }
    for region in 0..ROOT_RING_REGIONS {
        let device = geometry.region_devices[usize::try_from(region).expect("区域号")];
        for slot in 0..geometry.root_ring_slots_per_region {
            push(
                device,
                region_start(region).0 + slot * geometry.fixed_structure_slot_spacing,
                geometry.physical_block_size,
                JudgeLocationKind::RootSlot,
            );
        }
    }
    // 碰过的字节范围：基线写过的扇区，加枚举写表里每次不是清零写的写的范围。清零写不算碰过：mkfs 一次清整个 journal 环（第一版 768 MiB，
    // 每块盘 196 608 个记录槽），照算就给每一槽建一个落点，GPU 每个状态逐个落点扫环（`crash_judge_common.wgsl` 的 `scan_journal`）。
    // 不算它不少判一个字节：清零写写的全是 0，它的撕裂镜像每个字节不是 0 就是「这次写之前的写全落了」时的旧字节（`crash::torn_image_of_in_place_overwrite`），
    // 旧字节非零的扇区是基线写过的、或录制流里更早一次带字节的写写的，都已算碰过；只被清零写碰过的槽在每个崩溃状态里都读出全 0。
    // GPU 读没有落点的槽，结局与读全 0 相同：单元区里交全零单元（`zero_unit_version_over_a_slot_nothing_wrote`），journal 环里那一槽不进扫环、
    // 全 0 的记录本来也自证不过，逐档读槽 1 读成不自证、全 0 的槽本来也自证不过；系统配置每盘两槽与根环全部槽照建落点，不看碰没碰过。
    // 清零写照样记进它罩住的每个落点的写（下面「每次写盖住哪几个落点」按字节范围记），落点的版本照叠法叠出来。
    let mut touched_ranges: BTreeMap<u32, Vec<Range<u64>>> = BTreeMap::new();
    for (device, sparse) in &base.devices {
        let ranges = touched_ranges.entry(device.0).or_default();
        for sector in sparse.written_sectors().keys() {
            ranges.push(sector * SECTOR_BYTES..(sector + 1) * SECTOR_BYTES);
        }
    }
    for write in enumerated
        .iter()
        .filter(|write| write_puts_bytes_of_its_own(write.kind))
    {
        touched_ranges
            .entry(write.device.0)
            .or_default()
            .push(write.offset.0..write.offset.0 + write.length_in_bytes());
    }
    let ring_start = JOURNAL_RING_START_SLOT * slot_bytes();
    let ring_end = ring_start + geometry.journal_ring_bytes;
    let unit_area_start = geometry.unit_area_start_slot * slot_bytes();
    for (device, ranges) in &touched_ranges {
        let mut journal_records: BTreeSet<u64> = BTreeSet::new();
        let mut unit_slots: BTreeSet<u64> = BTreeSet::new();
        for range in ranges {
            let journal_overlap = range.start.max(ring_start)..range.end.min(ring_end);
            if journal_overlap.start < journal_overlap.end {
                let first = (journal_overlap.start - ring_start) / JOURNAL_RECORD_BYTES;
                let last = (journal_overlap.end - 1 - ring_start) / JOURNAL_RECORD_BYTES;
                journal_records.extend(first..=last);
            }
            let unit_overlap = range.start.max(unit_area_start)..range.end;
            if unit_overlap.start < unit_overlap.end {
                let first = unit_overlap.start / slot_bytes();
                let last = (unit_overlap.end - 1) / slot_bytes();
                unit_slots.extend(first..=last);
            }
        }
        for record in journal_records {
            push(
                *device,
                ring_start + record * JOURNAL_RECORD_BYTES,
                JOURNAL_RECORD_BYTES,
                JudgeLocationKind::JournalRecord,
            );
        }
        for slot in unit_slots {
            let offset = slot * slot_bytes();
            let length = if offset + DATA_UNIT_BYTES <= device_bytes {
                DATA_UNIT_BYTES
            } else {
                NODE_BYTES
            };
            push(*device, offset, length, JudgeLocationKind::UnitSlot);
        }
    }
    // 逐档读到的槽 1：全池没有一个可择的槽 0 时，GPU 两个内核照 CPU 在每一档槽距上各读一槽
    // （[`slot_one_offsets_of_every_spacing_the_format_allows`]），偏移 0 与几何槽距两处之外没有落点的那几档 `read_at` 交没有版本的切片、
    // 读成不自证，而 CPU 在那里读到自证得过的槽（逐档读到 8192 处带读者不收之值、12288 处在飞上限加一的槽，CPU 报 I-7.13）。
    // 每一档上基线或枚举写碰过那 4096 字节的建一个系统配置落点；哪一版都不自证的建完版本再去掉（[`location_is_read_by_the_judge`]）。
    // 碰过的范围只取根环基址之前的那几段（基线写过的扇区可以上十万个，逐档比的只是系统配置那 1 MiB 里的几个）
    let root_ring_base = region_start(0).0;
    for (device, ranges) in &touched_ranges {
        let ranges_before_the_root_ring: Vec<&Range<u64>> = ranges
            .iter()
            .filter(|range| range.start < root_ring_base)
            .collect();
        for offset in slot_one_offsets_of_every_spacing_the_format_allows() {
            if ranges_before_the_root_ring.iter().any(|range| {
                range.start < offset + SYSTEM_CONFIGURATION_SLOT_BYTES && offset < range.end
            }) {
                push(
                    *device,
                    offset,
                    SYSTEM_CONFIGURATION_SLOT_BYTES,
                    JudgeLocationKind::SystemConfiguration,
                );
            }
        }
    }
    locations
}

/// 一次写会不会写下自己的非零字节（[`enumerate_locations`] 按它算碰过的范围）：清零写（连同它的撕裂镜像，种类照抄被撕的那次写）不会，
/// 别的种类会。discard 写的也是 0，照旧算碰过：它罩住的每个单元槽各建一个落点（discard 进判表那一批的写法，
/// `tests::a_discard_over_a_data_unit_lands_in_every_unit_slot_location_it_covers_in_its_three_states` 钉着），这里不动它。
fn write_puts_bytes_of_its_own(kind: StepKind) -> bool {
    match kind {
        StepKind::ZeroFill => false,
        StepKind::Discard
        | StepKind::UnitWrite
        | StepKind::JournalRecord
        | StepKind::RootRecordFua
        | StepKind::SystemConfigurationSlot
        | StepKind::Barrier => true,
    }
}

/// 建完版本之后这个落点留不留在判表里：系统配置落点在偏移 0 与几何槽距上的照留（每块盘两槽，判器按它们读槽 0 与槽 1）；
/// 逐档建的别的那几处至少一版自证得过（magic 对、整槽校验和过，与 GPU 两个内核判「自证」同一条：恢复内核 `parse_system_configuration`、
/// 走读内核 `checker_read_system_configuration_slot` 开头那一判）才留，一版都不自证的留不留 GPU 都读成不自证。别的种类照留。
fn location_is_read_by_the_judge(location: &JudgeLocation, geometry: &BaseGeometry) -> bool {
    match location.kind {
        JudgeLocationKind::SystemConfiguration => {
            location.offset == 0
                || location.offset == geometry.fixed_structure_slot_spacing
                || location.versions.iter().any(|version| {
                    version.bytes.starts_with(&SYSTEM_CONFIGURATION_MAGIC)
                        && version.checks[SYSTEM_CONFIGURATION_SLOT_CHECKSUM_HOLDS_CHECK] == 1
                })
        }
        JudgeLocationKind::RootSlot
        | JudgeLocationKind::JournalRecord
        | JudgeLocationKind::UnitSlot => true,
    }
}

/// 枚举写表里录制流之外的那几条（撕裂镜像、重放）各自跟着哪次录制写：撕裂镜像跟着被撕的那次写，重放跟着被撕的那次写与被重放的那次写。
struct ExtendedWriteOrigins {
    /// 枚举写表下标 → （被撕的那次写，被重放的那次写（撕裂镜像自己没有））。
    origin: BTreeMap<usize, (usize, Option<usize>)>,
}

impl ExtendedWriteOrigins {
    fn of(tables: &CrashStateEnumerationTables, recorded_write_count: usize) -> Self {
        let mut origin = BTreeMap::new();
        for torn in &tables.torn_images {
            origin.insert(torn.torn_image_index, (torn.write_index, None));
            for (replay_index, replayed) in &torn.replays {
                origin.insert(*replay_index, (torn.write_index, Some(*replayed)));
            }
        }
        for index in origin.keys() {
            assert!(
                *index >= recorded_write_count,
                "撕裂镜像与重放都接在录制流之后"
            );
        }
        Self { origin }
    }
}

/// 一个落点在全部崩溃状态里可达的组合：按崩溃在哪一段（含最后全部持久那一个）逐段枚举自由写的落地、算组合、去重。
/// 展开了的段里每次写都当自由写（甲二只取其中一部分组合，多出来的版本没有状态查它，不改判定）。
fn reachable_combinations_of_the_expanded_segments(
    segment_of_write: &[usize],
    extended: &ExtendedWriteOrigins,
    segment_count: usize,
    expanded_segments: &BTreeSet<usize>,
    location: &JudgeLocation,
) -> Result<BTreeSet<CombinationOfTheWritesOnALocation>, JudgeTablesRefusal> {
    let recorded_write_count = segment_of_write.len();
    let mut combinations: BTreeSet<CombinationOfTheWritesOnALocation> = BTreeSet::new();
    // 一次写的落地按它在哪一段与这个组合里给它的落地算
    let landing_of = |write_index: usize, cut: usize, free: &BTreeMap<usize, Landing>| -> Landing {
        if let Some(landing) = free.get(&write_index) {
            return *landing;
        }
        if segment_of_write[write_index] < cut {
            Landing::Persisted
        } else {
            Landing::NotPersisted
        }
    };
    let persisted_in = |entry: usize, cut: usize, free: &BTreeMap<usize, Landing>| -> bool {
        if entry < recorded_write_count {
            return landing_of(entry, cut, free) == Landing::Persisted;
        }
        let (torn_write, replayed) = extended.origin[&entry];
        if landing_of(torn_write, cut, free) != Landing::Torn {
            return false;
        }
        match replayed {
            None => true,
            Some(replayed) => landing_of(replayed, cut, free) == Landing::Persisted,
        }
    };
    let combination_of =
        |cut: usize, free: &BTreeMap<usize, Landing>| -> CombinationOfTheWritesOnALocation {
            let mut combination = CombinationOfTheWritesOnALocation::NOTHING_PERSISTED;
            for (position, entry) in location.writes.iter().enumerate() {
                if persisted_in(usize::try_from(*entry).expect("下标"), cut, free) {
                    combination.mark_persisted(position);
                }
            }
            combination
        };
    let mut cuts: Vec<usize> = expanded_segments.iter().copied().collect();
    cuts.push(segment_count);
    for cut in cuts {
        // 这一段里的自由写：盖住落点的录制写在这一段的，加上扩展写的来路里在这一段的
        let mut free_writes: BTreeSet<usize> = BTreeSet::new();
        for entry in &location.writes {
            let entry = usize::try_from(*entry).expect("下标");
            if entry < recorded_write_count {
                if segment_of_write[entry] == cut {
                    free_writes.insert(entry);
                }
            } else {
                let (torn_write, replayed) = extended.origin[&entry];
                if segment_of_write[torn_write] == cut {
                    free_writes.insert(torn_write);
                }
                if let Some(replayed) = replayed {
                    if segment_of_write[replayed] == cut {
                        free_writes.insert(replayed);
                    }
                }
            }
        }
        let free_writes: Vec<usize> = free_writes.into_iter().collect();
        let tearable = |write_index: usize| {
            extended
                .origin
                .values()
                .any(|(torn_write, replayed)| *torn_write == write_index && replayed.is_none())
        };
        let choices: Vec<usize> = free_writes
            .iter()
            .map(|write_index| if tearable(*write_index) { 3 } else { 2 })
            .collect();
        let total: usize = choices.iter().product();
        if total > MAXIMUM_VERSIONS_PER_LOCATION {
            return Err(JudgeTablesRefusal::TooManyVersionsOnOneLocation {
                device: location.device,
                offset: location.offset,
            });
        }
        for mut assignment in 0..total {
            let mut free: BTreeMap<usize, Landing> = BTreeMap::new();
            for (write_index, choice_count) in free_writes.iter().zip(&choices) {
                let digit = assignment % choice_count;
                assignment /= choice_count;
                let landing = match (choice_count, digit) {
                    (2, 0) | (3, 0) => Landing::NotPersisted,
                    (2, 1) | (3, 2) => Landing::Persisted,
                    (3, 1) => Landing::Torn,
                    _ => unreachable!("数字不超过态数"),
                };
                free.insert(*write_index, landing);
            }
            combinations.insert(combination_of(cut, &free));
            if combinations.len() > MAXIMUM_VERSIONS_PER_LOCATION {
                return Err(JudgeTablesRefusal::TooManyVersionsOnOneLocation {
                    device: location.device,
                    offset: location.offset,
                });
            }
        }
    }
    Ok(combinations)
}

/// 给定状态表上一个落点的组合：每个点名的持久掩码在这个落点的写上各取一个组合、去重。
fn combinations_of_the_given_persisted_sets(
    location: &JudgeLocation,
    given_persisted_sets: &[Vec<bool>],
) -> Result<BTreeSet<CombinationOfTheWritesOnALocation>, JudgeTablesRefusal> {
    let mut combinations: BTreeSet<CombinationOfTheWritesOnALocation> = BTreeSet::new();
    for persisted in given_persisted_sets {
        let mut combination = CombinationOfTheWritesOnALocation::NOTHING_PERSISTED;
        for (position, entry) in location.writes.iter().enumerate() {
            if persisted[usize::try_from(*entry).expect("下标")] {
                combination.mark_persisted(position);
            }
        }
        combinations.insert(combination);
        if combinations.len() > MAXIMUM_VERSIONS_PER_LOCATION {
            return Err(JudgeTablesRefusal::TooManyVersionsOnOneLocation {
                device: location.device,
                offset: location.offset,
            });
        }
    }
    Ok(combinations)
}

/// 一个落点的每个组合叠出字节、算检查字与逐扇区等不等，按组合升序记进它的版本表（与 `CrashImage::read` 同一条叠法）。
fn push_the_versions_of_the_combinations(
    base: &MemoryPool,
    enumerated: &[RetainedWrite],
    location: &mut JudgeLocation,
    combinations: BTreeSet<CombinationOfTheWritesOnALocation>,
) {
    let base_bytes = PoolReader::read(
        base,
        singlefs_core::address::DeviceIdentity(location.device),
        DeviceOffsetInBytes(location.offset),
        usize::try_from(location.length).expect("落点长度装得进 usize"),
    )
    .expect("落点在池里的盘上（enumerate_locations 只给池里的盘建落点）、在盘内、按扇区对齐");
    let location_start = location.offset;
    let location_length = u64::from(location.length);
    for combination in combinations {
        let mut bytes = base_bytes.clone();
        for (position, entry) in location.writes.iter().enumerate() {
            if !combination.is_persisted(position) {
                continue;
            }
            let write = &enumerated[usize::try_from(*entry).expect("下标")];
            overlay(&mut bytes, location_start, write);
        }
        let checks = version_checks(location.kind, &bytes);
        let sector_equals = location
            .writes
            .iter()
            .map(|entry| {
                sector_equality(
                    &bytes,
                    location_start,
                    location_length,
                    &enumerated[usize::try_from(*entry).expect("下标")],
                )
            })
            .collect();
        location.versions.push(JudgingUnitsFingerprint {
            combination,
            bytes,
            checks,
            sector_equals,
        });
    }
}

/// 把一次写盖到落点的字节上（与 `CrashImage::read` 同一条叠法）。
fn overlay(bytes: &mut [u8], location_start: u64, write: &RetainedWrite) {
    let location_end = location_start + u64::try_from(bytes.len()).expect("长度");
    let write_start = write.offset.0;
    let write_end = write_start + write.length_in_bytes();
    let overlap_start = location_start.max(write_start);
    let overlap_end = location_end.min(write_end);
    if overlap_start >= overlap_end {
        return;
    }
    let destination = usize::try_from(overlap_start - location_start).expect("偏移");
    let source = usize::try_from(overlap_start - write_start).expect("偏移");
    let overlap_length = usize::try_from(overlap_end - overlap_start).expect("长度");
    write.contents.copy_range_into(
        source,
        &mut bytes[destination..destination + overlap_length],
    );
}

/// 落点每个扇区上的字节是不是这次写写下的（这次写没盖到的扇区算不是）。
fn sector_equality(
    bytes: &[u8],
    location_start: u64,
    location_length: u64,
    write: &RetainedWrite,
) -> u64 {
    let sector_count = usize::try_from(location_length / SECTOR_BYTES).expect("扇区数");
    assert!(
        sector_count <= MAXIMUM_LOCATION_SECTORS,
        "落点不超过 64 个扇区"
    );
    let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
    let write_start = write.offset.0;
    let write_end = write_start + write.length_in_bytes();
    let mut mask = 0u64;
    for sector in 0..sector_count {
        let sector_start = location_start + u64::try_from(sector).expect("扇区号") * SECTOR_BYTES;
        if sector_start < write_start || sector_start + SECTOR_BYTES > write_end {
            continue;
        }
        let in_write = usize::try_from(sector_start - write_start).expect("偏移");
        let on_disk = &bytes[sector * sector_bytes..(sector + 1) * sector_bytes];
        if write.contents.range_still_on_disk(in_write, on_disk) {
            mask |= 1u64 << sector;
        }
    }
    mask
}

/// 系统配置落点的检查字里「整槽校验和字段对得上」那一字的下标（[`version_checks`] 系统配置那一臂的头一字；GPU 内核读成切片的 `c0`）。
const SYSTEM_CONFIGURATION_SLOT_CHECKSUM_HOLDS_CHECK: usize = 0;

/// 每一类落点的检查字。
///
/// - 系统配置：`[整槽校验和字段对得上, 0, 0, 0]`。
/// - 根槽：`[整槽校验和字段对得上（罩整个槽宽）, 0, 0, 0]`。
/// - journal 记录：`[整条校验和字段对得上, 载荷 CRC 对得上（点名项区在 4096 之内）, 反向链值 = CRC-32C(头 496 字节、校验和字段按 0), 0]`。
/// - 单元槽：`[前 16 KiB 的 CRC-32C, 前 32 KiB 的 CRC-32C（落点只有 16 KiB 时 0）, 标志位, 0]`；标志位：位 0 头校验和按自述的类过、
///   位 1 载荷 CRC 按自述的类过、位 2 落点有 32 KiB、位 3 类身份段之后的 29 字节预留位全 0。类从偏移 6 读，码 2 的 key 宽从偏移 51 读。
#[must_use]
pub fn version_checks(kind: JudgeLocationKind, bytes: &[u8]) -> [u32; 4] {
    match kind {
        JudgeLocationKind::SystemConfiguration => [
            u32::from(wide_checksum_field_holds(
                bytes,
                usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096"),
                155,
            )),
            0,
            0,
            0,
        ],
        JudgeLocationKind::RootSlot => {
            let holds = bytes.len() >= usize::try_from(ROOT_RECORD_BYTES).expect("466")
                && wide_checksum_field_holds(bytes, bytes.len(), 138);
            [u32::from(holds), 0, 0, 0]
        }
        JudgeLocationKind::JournalRecord => {
            let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
            let header_bytes =
                usize::try_from(JOURNAL_HEADER_BYTES).expect("记录头 496 字节装得进 usize");
            let header_holds = wide_checksum_field_holds(bytes, record_bytes, 46);
            let named_count = usize::try_from(u32::from_le_bytes(
                bytes[12..16].try_into().expect("4 字节"),
            ))
            .expect("点名项数");
            let payload_end = header_bytes.saturating_add(named_count.saturating_mul(56));
            let payload_holds = payload_end <= record_bytes
                && crc32_castagnoli(&bytes[header_bytes..payload_end])
                    == u32::from_le_bytes(bytes[95..99].try_into().expect("4 字节"));
            let mut header = bytes[..header_bytes].to_vec();
            header[46..78].fill(0);
            [
                u32::from(header_holds),
                u32::from(payload_holds),
                crc32_castagnoli(&header),
                0,
            ]
        }
        JudgeLocationKind::UnitSlot => {
            let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
            let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
            let crc_16k = if bytes.len() >= node_bytes {
                crc32_castagnoli(&bytes[..node_bytes])
            } else {
                0
            };
            let has_32k = bytes.len() >= data_unit_bytes;
            let crc_32k = if has_32k {
                crc32_castagnoli(&bytes[..data_unit_bytes])
            } else {
                0
            };
            let reserved = usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29");
            let (header_end, unit_end, payload_crc_offset): (Option<usize>, usize, usize) =
                match bytes.get(6).copied() {
                    Some(1) => (
                        Some(usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105")),
                        data_unit_bytes,
                        101,
                    ),
                    Some(2) => {
                        let key_width = usize::from(bytes.get(51).copied().unwrap_or(0));
                        let header_end = 86 + 2 * key_width;
                        (Some(header_end), node_bytes, header_end - 10)
                    }
                    Some(3) => (Some(107), data_unit_bytes, 89),
                    _ => (None, 0, 0),
                };
            let mut flags = 0u32;
            if let Some(header_end) = header_end {
                if bytes.len() >= unit_end {
                    if wide_checksum_field_holds(bytes, header_end, 10) {
                        flags |= 1;
                    }
                    let stored = u32::from_le_bytes(
                        bytes[payload_crc_offset..payload_crc_offset + 4]
                            .try_into()
                            .expect("4 字节"),
                    );
                    if crc32_castagnoli(&bytes[header_end..unit_end]) == stored {
                        flags |= 2;
                    }
                    if bytes[header_end..header_end + reserved]
                        .iter()
                        .all(|byte| *byte == 0)
                    {
                        flags |= 8;
                    }
                    // 位 4：码 1 声明长度之后到单元末尾全 0（`unit::data_unit_payload` 的补齐判定）
                    if bytes[6] == 1 {
                        let declared = usize::from(u16::from_le_bytes([bytes[8], bytes[9]]));
                        let payload_start = header_end + reserved;
                        if payload_start + declared <= unit_end
                            && bytes[payload_start + declared..unit_end]
                                .iter()
                                .all(|byte| *byte == 0)
                        {
                            flags |= 16;
                        }
                    }
                }
            }
            if has_32k {
                flags |= 4;
            }
            [crc_16k, crc_32k, flags, unit_layout_checks(bytes)]
        }
    }
}

/// 单元槽版本的第四个检查字：池级 checker 按**引用它的那一方期望的类**的字段表判头（`walk.rs` 的 `judge_unit_header`），
/// 与单元自述的类无关，所以三种布局各算一遍：位 4L + 0 头校验和过、4L + 1 载荷 CRC 过、4L + 2 明文头之后 29 字节预留位全 0、
/// 4L + 3 声明长度之后到单元末尾全 0；L = 0 按码 1 的布局（头 105、载荷 CRC 101、载荷 134 起、单元 32768）、
/// L = 1 按码 2（头 86 + 2k、载荷 CRC 76 + 2k、条目 115 + 2k 起、单元 16384，k 取偏移 51）、L = 2 按码 3（107、89、136、32768）。
/// 落点不够那种布局的单元长时那一组位全 0。
#[must_use]
pub fn unit_layout_checks(bytes: &[u8]) -> u32 {
    let mut flags = 0u32;
    let key_width = usize::from(bytes.get(51).copied().unwrap_or(0));
    let layouts: [(usize, usize, usize, usize); 3] = [
        (
            105,
            101,
            134,
            usize::try_from(DATA_UNIT_BYTES).expect("32768"),
        ),
        (
            86 + 2 * key_width,
            76 + 2 * key_width,
            115 + 2 * key_width,
            usize::try_from(NODE_BYTES).expect("16384"),
        ),
        (
            107,
            89,
            136,
            usize::try_from(DATA_UNIT_BYTES).expect("32768"),
        ),
    ];
    let reserved = usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29");
    for (layout, (header_end, payload_crc_offset, payload_start, unit_end)) in
        layouts.into_iter().enumerate()
    {
        if bytes.len() < unit_end || header_end + reserved > unit_end {
            continue;
        }
        let bit = u32::try_from(4 * layout).expect("位号");
        if wide_checksum_field_holds(bytes, header_end, 10) {
            flags |= 1 << bit;
        }
        let stored = u32::from_le_bytes(
            bytes[payload_crc_offset..payload_crc_offset + 4]
                .try_into()
                .expect("4 字节"),
        );
        if crc32_castagnoli(&bytes[header_end..unit_end]) == stored {
            flags |= 1 << (bit + 1);
        }
        if bytes[header_end..header_end + reserved]
            .iter()
            .all(|byte| *byte == 0)
        {
            flags |= 1 << (bit + 2);
        }
        let declared = usize::from(u16::from_le_bytes([bytes[8], bytes[9]]));
        let padding_start = payload_start + declared;
        if padding_start <= unit_end && bytes[padding_start..unit_end].iter().all(|byte| *byte == 0)
        {
            flags |= 1 << (bit + 3);
        }
    }
    flags
}

/// 写表：每次写的落点、根身份、根字节、之后与它重叠的写。
fn judge_writes(
    enumerated: &[RetainedWrite],
    recorded: &[RetainedWrite],
    locations: &[JudgeLocation],
    location_index_by_start: &BTreeMap<(u32, u64), usize>,
) -> Result<Vec<JudgeWrite>, JudgeTablesRefusal> {
    let mut writes = Vec::with_capacity(enumerated.len());
    for (write_index, write) in enumerated.iter().enumerate() {
        let location = match write.kind {
            // discard 按段整段发、罩住几十个单元槽，清零写一次清 [0, 根环基址)、一个根环区域或整个 journal 环，都不落在哪一个落点上：
            // 它罩住的每个落点各把它记进自己的写（建落点那一步按字节范围记），版本的字节按叠法叠出来。写表里它这一行只给持久掩码与
            // 「之后与一份副本重叠的写」用。
            StepKind::Discard | StepKind::ZeroFill => None,
            StepKind::UnitWrite
            | StepKind::JournalRecord
            | StepKind::RootRecordFua
            | StepKind::SystemConfigurationSlot
            | StepKind::Barrier => {
                let location_index = location_index_by_start
                    .get(&(write.device.0, write.offset.0))
                    .copied()
                    .ok_or(JudgeTablesRefusal::WriteNotAlignedToALocation { write_index })?;
                if write.length_in_bytes() > u64::from(locations[location_index].length) {
                    return Err(JudgeTablesRefusal::WriteNotAlignedToALocation { write_index });
                }
                Some(u32::try_from(location_index).expect("下标装得进 u32"))
            }
        };
        let root_identity = (write.kind == StepKind::RootRecordFua).then(|| {
            let (instance, checkpoint_txg) = root_identity_of_write(write);
            (instance.0, checkpoint_txg.0)
        });
        let root_bytes = match (&write.contents, write.kind) {
            (WrittenContents::Bytes(bytes), StepKind::RootRecordFua) => bytes.clone(),
            (WrittenContents::Bytes(_) | WrittenContents::Zeros { .. }, _) => Vec::new(),
        };
        let later_overlapping_writes = if write.kind == StepKind::UnitWrite {
            later_overlapping_writes_of(enumerated, recorded, write_index)
        } else {
            Vec::new()
        };
        writes.push(JudgeWrite {
            device: write.device.0,
            offset: write.offset.0,
            length: u32::try_from(write.length_in_bytes()).expect("写长装得进 u32"),
            kind: write.kind,
            location,
            root_identity,
            root_bytes,
            later_overlapping_writes,
        });
    }
    Ok(writes)
}

/// 一份单元副本之后与它重叠的每次写，连同回收谓词的判定（与 `crash::unit_copy_is_missing_under_the_persisted_set` 同一个读法：
/// 判定只看写表，与状态无关）。谓词按录制流那张表算（枚举写表的扩展条目不在录制流里；副本在录制流里）。
fn later_overlapping_writes_of(
    enumerated: &[RetainedWrite],
    recorded: &[RetainedWrite],
    copy: usize,
) -> Vec<LaterOverlappingWrite> {
    let copy_write = &enumerated[copy];
    let copy_start = copy_write.offset.0;
    let copy_end = copy_start + copy_write.length_in_bytes();
    let earlier_publish_txg =
        crate::crash::checkpoint_txg_of_the_publish_that_made_the_write(enumerated, copy);
    let mut laters = Vec::new();
    for later_index in copy + 1..enumerated.len() {
        let later = &enumerated[later_index];
        if later.device != copy_write.device {
            continue;
        }
        let later_start = later.offset.0;
        let later_end = later_start + later.length_in_bytes();
        if !(later_start < copy_end && copy_start < later_end) {
            continue;
        }
        let mut covered_sectors = 0u64;
        let sector_count = usize::try_from((copy_end - copy_start) / SECTOR_BYTES).expect("扇区数");
        for sector in 0..sector_count {
            let sector_offset = copy_start + u64::try_from(sector).expect("扇区号") * SECTOR_BYTES;
            if later_start < sector_offset + SECTOR_BYTES && sector_offset < later_end {
                covered_sectors |= 1u64 << sector;
            }
        }
        // 谓词按写表里这次写之前的那一段算，枚举写表里录制流之外的条目（撕裂镜像、重放）不改它：它们接在录制流之后，
        // 在它们「之前」的就是整条录制流；`reuse_is_not_proven_illegal_by_the_reclaim_predicate` 只看录制流那一段的根槽写与系统配置槽写。
        let table: &[RetainedWrite] = if later_index < recorded.len() {
            recorded
        } else {
            enumerated
        };
        let reuse_is_not_proven_illegal = reuse_is_not_proven_illegal_by_the_reclaim_predicate(
            table,
            earlier_publish_txg,
            later_index.min(table.len()),
        );
        laters.push(LaterOverlappingWrite {
            write_index: u32::try_from(later_index).expect("下标装得进 u32"),
            reuse_is_not_proven_illegal,
            covered_sectors,
        });
    }
    laters
}

/// 发布表：与记录核对器同一个分法（`publishes_in`）。按段展开、或给定状态表不带接缝时是一条录制流（持久集合不进分法）；
/// 给定状态表带接缝时在接缝处断开，崩溃接缝之前那一截剔哪几次发布按这张表的持久掩码定（每个掩码在那几个根槽写上都一样，这里核），
/// 掉电接缝之前那一截要在每个掩码里全落了（这里核，不交 `publishes_in` 去 panic）。接缝的下标是录制流的下标（`recorded_write_count`
/// 之前）。`enumerated` 是发布表从哪一张写表分（调用方照 [`PublishesSplitFrom`] 给）：给整张枚举写表时，接在录制流之后的撕裂镜像与重放归最后一截。
fn judge_publishes(
    enumerated: &[RetainedWrite],
    recorded_write_count: usize,
    state_source: &JudgeStateSource,
    seams_of_the_given_sets: &[RecordStreamSeam],
) -> Result<Vec<JudgePublish>, JudgeTablesRefusal> {
    let representative_persisted: Option<&Vec<bool>> = match state_source {
        JudgeStateSource::SegmentExpansions => {
            assert!(
                seams_of_the_given_sets.is_empty(),
                "按段展开那一路恒给空的接缝表（build_judge_tables_from_the_state_source 的两个调用点）"
            );
            None
        }
        JudgeStateSource::GivenPersistedSets {
            persisted_sets: given_persisted_sets,
            claimed_versions: _,
        } => {
            if seams_of_the_given_sets.is_empty() {
                None
            } else {
                given_persisted_sets_agree_across_the_seams(
                    enumerated,
                    recorded_write_count,
                    given_persisted_sets,
                    seams_of_the_given_sets,
                )?;
                given_persisted_sets.first()
            }
        }
    };
    let publishes = match representative_persisted {
        Some(persisted) => publishes_in(
            enumerated,
            persisted,
            RecordStreamContinuity::BrokenAtSeams(seams_of_the_given_sets),
        ),
        None => publishes_in(
            enumerated,
            &vec![false; enumerated.len()],
            RecordStreamContinuity::OneRecording,
        ),
    };
    Ok(judge_publishes_of(enumerated, publishes))
}

/// 给定状态表的接缝核三样（[`judge_publishes`]）：接缝按录制流下标从小到大、不越过录制流；每道掉电接缝之前那一截在每个掩码里全落了；
/// 每道崩溃接缝之前那一截（从上一道接缝起）里的每个根槽写在每个掩码里落没落都一样。
fn given_persisted_sets_agree_across_the_seams(
    enumerated: &[RetainedWrite],
    recorded_write_count: usize,
    given_persisted_sets: &[Vec<bool>],
    seams: &[RecordStreamSeam],
) -> Result<(), JudgeTablesRefusal> {
    let mut first_write_of_the_recording = 0usize;
    for (seam_index, seam) in seams.iter().enumerate() {
        let first_write_after_the_seam = seam.first_write_after_the_seam();
        if first_write_after_the_seam < first_write_of_the_recording
            || first_write_after_the_seam > recorded_write_count
        {
            return Err(JudgeTablesRefusal::SeamsNotInOrderWithinTheRecordedWrites { seam_index });
        }
        match seam {
            RecordStreamSeam::AfterAPowerLoss { .. } => {
                if let Some(position) = given_persisted_sets.iter().position(|persisted| {
                    !persisted[..first_write_after_the_seam]
                        .iter()
                        .all(|is_persisted| *is_persisted)
                }) {
                    return Err(
                        JudgeTablesRefusal::GivenPersistedSetNotWhollyPersistedBeforeAPowerLossSeam {
                            position,
                            seam_index,
                        },
                    );
                }
            }
            RecordStreamSeam::AfterACrash { .. } => {
                for root_write in (first_write_of_the_recording..first_write_after_the_seam)
                    .filter(|write_index| enumerated[*write_index].kind == StepKind::RootRecordFua)
                {
                    let Some(first) = given_persisted_sets.first() else {
                        break;
                    };
                    if let Some(position) = given_persisted_sets
                        .iter()
                        .position(|persisted| persisted[root_write] != first[root_write])
                    {
                        return Err(
                            JudgeTablesRefusal::GivenPersistedSetsDisagreeOnARootWriteBeforeACrashSeam {
                                root_write,
                                position,
                            },
                        );
                    }
                }
            }
        }
        first_write_of_the_recording = first_write_after_the_seam;
    }
    Ok(())
}

/// `publishes_in` 分出的每次发布换成判器的发布表（单元写按偏移分组）。
fn judge_publishes_of(
    enumerated: &[RetainedWrite],
    publishes: Vec<singlefs_harness::memory_pool::PublishWrites>,
) -> Vec<JudgePublish> {
    publishes
        .into_iter()
        .map(|publish| {
            let mut copies_by_offset: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
            for unit in &publish.units {
                copies_by_offset
                    .entry(enumerated[*unit].offset.0)
                    .or_default()
                    .push(u32::try_from(*unit).expect("下标装得进 u32"));
            }
            JudgePublish {
                root_write: u32::try_from(publish.root).expect("下标装得进 u32"),
                instance: publish.instance,
                checkpoint_txg: publish.checkpoint_txg,
                records: publish
                    .records
                    .iter()
                    .map(|record| u32::try_from(*record).expect("下标装得进 u32"))
                    .collect(),
                unit_copy_groups: copies_by_offset.into_values().collect(),
            }
        })
        .collect()
}

/// 数据单元的载荷容量：32768 − 头 105 − 预留 29。
fn data_unit_payload_capacity() -> usize {
    usize::try_from(DATA_UNIT_BYTES).expect("32768")
        - usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105")
        - usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29")
}

/// 一个已发布版本的内容按写侧的切法切成单元，每个单元该有的载荷区 CRC（预留位 29 个 0 + 载荷 + 补 0 到单元末尾）与声明长度。
fn expected_version_chunks(version: &PublishedVersion) -> ExpectedVersionChunks {
    let capacity = data_unit_payload_capacity();
    let unit_count = version.content.len().div_ceil(capacity).max(1);
    let payload_region = usize::try_from(DATA_UNIT_BYTES).expect("32768")
        - usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105");
    let reserved = usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29");
    let chunks = (0..unit_count)
        .map(|index| {
            let start = (index * capacity).min(version.content.len());
            let end = ((index + 1) * capacity).min(version.content.len());
            let chunk = &version.content[start..end];
            let mut region = vec![0u8; payload_region];
            region[reserved..reserved + chunk.len()].copy_from_slice(chunk);
            (
                crc32_castagnoli(&region),
                u32::try_from(chunk.len()).expect("声明长度装得进 u32"),
            )
        })
        .collect();
    ExpectedVersionChunks {
        instance: version.instance.0,
        checkpoint_txg: version.checkpoint_txg.0,
        file: FileOfTheExpectedVersion::WithAFile,
        chunks,
    }
}

impl JudgeTables {
    /// 序号 → 持久掩码：按段展开时与崩溃点重放、事实表同一套数字次序（每段照它的展开方式拆，
    /// `crate::crash_facts::persisted_writes_of_state_by_the_segment_expansions`）；给定状态表时是清单里的第 `ordinal` 个。
    ///
    /// # Panics
    /// 序号越过状态总数。
    #[must_use]
    pub fn persisted_writes_of_state(&self, ordinal: u64) -> Vec<bool> {
        match &self.state_source {
            JudgeStateSource::SegmentExpansions => {
                persisted_writes_of_state_by_the_segment_expansions(
                    usize::try_from(self.recorded_write_count).expect("装得进 usize"),
                    usize::try_from(self.enumerated_write_count).expect("装得进 usize"),
                    &self.is_tearable,
                    &self.torn_images,
                    &self.segments,
                    &self.segment_expansions,
                    self.state_count,
                    ordinal,
                )
            }
            JudgeStateSource::GivenPersistedSets {
                persisted_sets: given_persisted_sets,
                claimed_versions: _,
            } => given_persisted_sets[usize::try_from(ordinal).expect("序号装得进 usize")].clone(),
        }
    }

    /// 打包之后每张表的起点与长度都装得进目录的 u32（[`pack_tables`] 照它写目录）：装不下就是那张表满了，GPU 判不了这条流，
    /// 交 [`JudgeTablesRefusal::PackedTablePastThirtyTwoBitOffsets`]（建表时调，不留到上卡才 panic）。
    ///
    /// # Errors
    /// 第一张越过 u32 的表。
    pub fn check_the_packed_directory(&self) -> Result<(), JudgeTablesRefusal> {
        let words = self.words();
        let mut words_up_to_the_end =
            u64::try_from(words.every_table().len() * 2).expect("目录字数装得进 u64");
        for (table, name) in words
            .every_table()
            .iter()
            .zip(JudgeTablesWords::every_table_name())
        {
            words_up_to_the_end += u64::try_from(table.len()).expect("表长装得进 u64");
            if words_up_to_the_end > u64::from(u32::MAX) {
                return Err(JudgeTablesRefusal::PackedTablePastThirtyTwoBitOffsets {
                    table: name,
                    words_up_to_its_end: words_up_to_the_end,
                });
            }
        }
        Ok(())
    }

    /// 一个落点在一个持久集合下的版本（CPU 参照：GPU 内核按同一条规则查）。
    #[must_use]
    pub fn version_of<'location>(
        &self,
        location: &'location JudgeLocation,
        persisted: &[bool],
    ) -> Option<&'location JudgingUnitsFingerprint> {
        let mut combination = CombinationOfTheWritesOnALocation::NOTHING_PERSISTED;
        for (position, entry) in location.writes.iter().enumerate() {
            if persisted[usize::try_from(*entry).expect("下标")] {
                combination.mark_persisted(position);
            }
        }
        location
            .versions
            .iter()
            .find(|version| version.combination == combination)
    }

    /// 每个落点之后那一段谁都没写过的槽（与 `locations` 同序，落点表第 9–11 字）。单元区里基线或枚举写表碰过的每个槽都有落点
    /// （`enumerate_locations`），所以一个落点与同一块盘上下一个落点之间、单元区里的槽在每个崩溃状态里都没写过：
    /// 单元槽落点从它下一槽起，别的落点从它末字节之后的头一个整槽起，都不早于单元区起点；到下一个落点起点所在的槽（不含）或盘尾为止。
    ///
    /// # Panics
    /// 盘大到槽号装不进 u32（64 TiB 及以上）。
    #[must_use]
    pub fn unit_slots_nothing_wrote_after_each_location(
        &self,
    ) -> Vec<UnitSlotsNothingWroteAfterALocation> {
        let slot_number = |bytes: u64| {
            u32::try_from(bytes / slot_bytes()).expect("盘不到 64 TiB，槽号装得进 u32")
        };
        let unit_area_start_slot = self.unit_area_start_in_bytes / slot_bytes();
        let device_end_slot = self.device_bytes / slot_bytes();
        let first_slot_a_whole_data_unit_does_not_fit_from = self
            .device_bytes
            .checked_sub(DATA_UNIT_BYTES)
            .map_or(0, |last_start| slot_number(last_start) + 1);
        self.locations
            .iter()
            .enumerate()
            .map(|(index, location)| {
                let first_slot_after_the_location = match location.kind {
                    JudgeLocationKind::UnitSlot => location.offset / slot_bytes() + 1,
                    JudgeLocationKind::SystemConfiguration
                    | JudgeLocationKind::RootSlot
                    | JudgeLocationKind::JournalRecord => {
                        (location.offset + u64::from(location.length)).div_ceil(slot_bytes())
                    }
                };
                let first_slot = first_slot_after_the_location.max(unit_area_start_slot);
                let end_slot = self
                    .locations
                    .get(index + 1)
                    .filter(|next| next.device == location.device)
                    .map_or(device_end_slot, |next| next.offset / slot_bytes())
                    .min(device_end_slot);
                let (first_slot, end_slot) = if first_slot < end_slot {
                    (first_slot, end_slot)
                } else {
                    (0, 0)
                };
                UnitSlotsNothingWroteAfterALocation {
                    first_slot: u32::try_from(first_slot).expect("盘不到 64 TiB，槽号装得进 u32"),
                    end_slot: u32::try_from(end_slot).expect("盘不到 64 TiB，槽号装得进 u32"),
                    first_slot_a_whole_data_unit_does_not_fit_from,
                }
            })
            .collect()
    }

    /// 全部表的摘要：只按内容取（表的形态怎么建、怎么摊由录入那一组判法单元的键管，不靠手写的版本号）。
    #[must_use]
    pub fn digest(&self) -> [u8; DIGEST_BYTES] {
        sha256_digest(&self.words().digest_bytes())
    }

    /// 摊成 GPU 内核读的几张 u32 表（[`JudgeTablesWords`]）。
    #[must_use]
    #[allow(
        clippy::too_many_lines,
        reason = "每张表的布局写在它填的地方，拆开会把布局与填法分到两处"
    )]
    pub fn words(&self) -> JudgeTablesWords {
        let split = |value: u64| -> (u32, u32) {
            (
                u32::try_from(value & 0xFFFF_FFFF).expect("低 32 位"),
                u32::try_from(value >> 32).expect("高 32 位"),
            )
        };
        // 一张表里的偏移与条数：建表时 [`Self::check_the_packed_directory`] 核过整份打包装得进 u32，这里装不下只会是没经建表就摊的表，
        // 记成全 1、交给那一核拒（不在这里 panic）
        let index = |value: usize| u32::try_from(value).unwrap_or(u32::MAX);
        // 掩码计划
        let mut recorded = Vec::new();
        let mut replays = Vec::new();
        let torn_of_write: BTreeMap<u32, &TornImageFact> = self
            .torn_images
            .iter()
            .map(|torn| (torn.write_index, torn))
            .collect();
        for write_index in 0..self.recorded_write_count {
            let tearable = self.is_tearable[usize::try_from(write_index).expect("下标")];
            match torn_of_write.get(&write_index) {
                Some(torn) => {
                    recorded.push(if tearable { 3 } else { 2 });
                    recorded.push(torn.torn_image_index);
                    recorded.push(index(replays.len() / 2));
                    recorded.push(index(torn.replays.len()));
                    for (replay_index, replayed) in &torn.replays {
                        replays.push(*replay_index);
                        replays.push(*replayed);
                    }
                }
                None => {
                    recorded.push(if tearable { 3 } else { 2 });
                    recorded.push(NONE);
                    recorded.push(0);
                    recorded.push(0);
                }
            }
        }
        // 段表每条 [`SEGMENT_WORDS`] 个字：数位那一组写的起点与条数、状态起点 lo/hi、状态终点 lo/hi、展开方式码（[`SegmentExpansionFact::code`]）、
        // 整组落的单元写那一组的起点与条数（只有甲二段有）；两组合起来是段里全部的写，之前的段整段持久时两组都置。
        // 第 9、10 字是分片的段（C790）片起点拆成的各位在段写表里的起点与位数（一位一个字：单元写那一组那一位在前，再是数位那一组写各一位）；
        // 不分片的段是 NONE 与 0
        let mut segments = Vec::new();
        let mut segment_writes = Vec::new();
        for (segment, expansion) in self.segments.iter().zip(&self.segment_expansions) {
            let (digit_writes, copy_on_write_group) =
                expansion.digit_writes_and_the_copy_on_write_group(segment);
            segments.push(index(segment_writes.len()));
            segments.push(index(digit_writes.len()));
            let (start_lo, start_hi) = split(segment.states.start);
            let (end_lo, end_hi) = split(segment.states.end);
            segments.extend([start_lo, start_hi, end_lo, end_hi]);
            segment_writes.extend(digit_writes.iter().copied());
            segments.push(expansion.code());
            segments.push(index(segment_writes.len()));
            segments.push(index(copy_on_write_group.len()));
            segment_writes.extend(copy_on_write_group.iter().copied());
            match &segment.slice_start {
                None => segments.extend([NONE, 0]),
                Some(slice_start) => {
                    segments.push(index(segment_writes.len()));
                    segments.push(index(slice_start.digits.len()));
                    segment_writes.extend(slice_start.digits.iter().map(|digit| u32::from(*digit)));
                }
            }
        }
        // 给定状态表：每个状态 ⌈枚举写数 ÷ 32⌉ 个字的持久掩码（位 i 在第 i / 32 字的第 i % 32 位），按序号接；按段展开时这张表是空的
        let mut given_persisted_sets = Vec::new();
        // 给定状态表每个状态那一栏恢复自称的那一版，每个 [`CLAIMED_VERSION_WORDS`] 个字：码（[`CLAIMED_VERSION_LANDED_ON_BY_THE_CONSULTED_RECOVERY`]
        // 一族）、实例代号、checkpoint_txg lo/hi（没写根的两种码后三个字是 NONE），按序号接；按段展开时这张表是空的
        let mut claimed_versions = Vec::new();
        if let JudgeStateSource::GivenPersistedSets {
            persisted_sets: _,
            claimed_versions: claimed_version_of_each_state,
        } = &self.state_source
        {
            for claimed_version in claimed_version_of_each_state {
                claimed_versions.extend(claimed_version_words(*claimed_version));
            }
        }
        if let JudgeStateSource::GivenPersistedSets {
            persisted_sets: sets,
            claimed_versions: _,
        } = &self.state_source
        {
            let words_per_state = usize::try_from(self.enumerated_write_count)
                .expect("装得进 usize")
                .div_ceil(32);
            for persisted in sets {
                let mut mask = vec![0u32; words_per_state];
                for (write_index, is_persisted) in persisted.iter().enumerate() {
                    if *is_persisted {
                        mask[write_index / 32] |= 1u32 << (write_index % 32);
                    }
                }
                given_persisted_sets.extend(mask);
            }
        }
        // 写表
        let mut writes = Vec::new();
        let mut root_bytes = Vec::new();
        let mut laters = Vec::new();
        for write in &self.writes {
            let (offset_lo, offset_hi) = split(write.offset);
            let (instance, txg_lo, txg_hi) = match write.root_identity {
                Some((instance, txg)) => {
                    let (lo, hi) = split(txg);
                    (instance, lo, hi)
                }
                None => (NONE, NONE, NONE),
            };
            let root_bytes_offset = if write.root_bytes.is_empty() {
                NONE
            } else {
                let offset = index(root_bytes.len());
                root_bytes.extend(bytes_to_words(&write.root_bytes));
                offset
            };
            writes.extend([
                write.device,
                offset_lo,
                offset_hi,
                write.length,
                step_kind_code(write.kind),
                write.location.unwrap_or(NONE),
                instance,
                txg_lo,
                txg_hi,
                root_bytes_offset,
                index(laters.len() / LATER_WORDS),
                index(write.later_overlapping_writes.len()),
            ]);
            for later in &write.later_overlapping_writes {
                let (covered_lo, covered_hi) = split(later.covered_sectors);
                laters.extend([
                    later.write_index,
                    u32::from(later.reuse_is_not_proven_illegal),
                    covered_lo,
                    covered_hi,
                ]);
            }
        }
        // 落点与版本。版本表最前面两条不是哪个落点的版本（[`ZERO_UNIT_VERSIONS`]）：第 0 条是一段 32 KiB 全零单元、第 1 条是同一段的前 16 KiB
        // （盘尾只装得下 16 KiB 的槽），组合都是 0、字节起点都是字节表的第 0 字、检查字按单元槽算（[`version_checks`]）、第 6 字 NONE。
        // 落点的第 9–11 字是它之后谁都没写过的槽（[`UnitSlotsNothingWroteAfterALocation`]）
        let mut locations = Vec::new();
        let mut location_writes = Vec::new();
        let mut versions = Vec::new();
        let mut version_bytes = Vec::new();
        let mut equal_masks = Vec::new();
        let zero_unit = vec![0u8; usize::try_from(DATA_UNIT_BYTES).expect("32768")];
        let zero_unit_lengths: [u64; ZERO_UNIT_VERSIONS] = [DATA_UNIT_BYTES, NODE_BYTES];
        for zero_unit_bytes in zero_unit_lengths {
            let checks = version_checks(
                JudgeLocationKind::UnitSlot,
                &zero_unit[..usize::try_from(zero_unit_bytes).expect("单元长装得进 usize")],
            );
            versions.extend(packed_version_words(
                &CombinationOfTheWritesOnALocation::NOTHING_PERSISTED,
                0,
                checks,
                NONE,
                crc_shifted_past_sixteen_zero_kibibytes(
                    JudgeLocationKind::UnitSlot,
                    &zero_unit[..usize::try_from(zero_unit_bytes).expect("单元长装得进 usize")],
                ),
            ));
        }
        version_bytes.extend(bytes_to_words(&zero_unit));
        let gaps = self.unit_slots_nothing_wrote_after_each_location();
        for (location_index, location) in self.locations.iter().enumerate() {
            let (offset_lo, offset_hi) = split(location.offset);
            locations.extend([
                location.device,
                offset_lo,
                offset_hi,
                location.length,
                location.kind.code(),
                index(location_writes.len()),
                index(location.writes.len()),
                index(versions.len() / VERSION_WORDS),
                index(location.versions.len()),
                gaps[location_index].first_slot,
                gaps[location_index].end_slot,
                gaps[location_index].first_slot_a_whole_data_unit_does_not_fit_from,
            ]);
            location_writes.extend(location.writes.iter().copied());
            for version in &location.versions {
                versions.extend(packed_version_words(
                    &version.combination,
                    index(version_bytes.len()),
                    version.checks,
                    index(equal_masks.len() / 2),
                    crc_shifted_past_sixteen_zero_kibibytes(location.kind, &version.bytes),
                ));
                version_bytes.extend(bytes_to_words(&version.bytes));
                for mask in &version.sector_equals {
                    let (lo, hi) = split(*mask);
                    equal_masks.extend([lo, hi]);
                }
            }
        }
        // 发布表
        let mut publishes = Vec::new();
        let mut publish_records = Vec::new();
        let mut copy_groups = Vec::new();
        let mut copies = Vec::new();
        for publish in &self.publishes {
            let (txg_lo, txg_hi) = split(publish.checkpoint_txg);
            publishes.extend([
                publish.root_write,
                publish.instance,
                txg_lo,
                txg_hi,
                index(publish_records.len()),
                index(publish.records.len()),
                index(copy_groups.len() / 2),
                index(publish.unit_copy_groups.len()),
            ]);
            publish_records.extend(publish.records.iter().copied());
            for group in &publish.unit_copy_groups {
                copy_groups.extend([index(copies.len()), index(group.len())]);
                copies.extend(group.iter().copied());
            }
        }
        // 期望版本
        let mut expected = Vec::new();
        let mut chunks = Vec::new();
        for version in &self.expected_versions {
            let (txg_lo, txg_hi) = split(version.checkpoint_txg);
            expected.extend([
                version.instance,
                txg_lo,
                txg_hi,
                index(chunks.len() / 2),
                index(version.chunks.len()),
                match version.file {
                    FileOfTheExpectedVersion::WithAFile => EXPECTED_VERSION_WITH_A_FILE,
                    FileOfTheExpectedVersion::WithoutAFile => EXPECTED_VERSION_WITHOUT_A_FILE,
                },
            ]);
            for (crc, length) in &version.chunks {
                chunks.extend([*crc, *length]);
            }
        }
        let (device_bytes_lo, device_bytes_hi) = split(self.device_bytes);
        let (state_count_lo, state_count_hi) = split(self.state_count);
        let mut header = vec![
            self.recorded_write_count,
            self.enumerated_write_count,
            index(self.segments.len()),
            index(self.devices.len()),
            device_bytes_lo,
            device_bytes_hi,
            index(self.locations.len()),
            index(self.publishes.len()),
            index(self.expected_versions.len()),
            self.judged_root_index,
            state_count_lo,
            state_count_hi,
        ];
        header.extend(self.devices.iter().copied());
        header.resize(HEADER_WORDS, 0);
        JudgeTablesWords {
            header,
            recorded,
            replays,
            segments,
            segment_writes,
            writes,
            root_bytes,
            laters,
            locations,
            location_writes,
            versions,
            version_bytes,
            equal_masks,
            publishes,
            publish_records,
            copy_groups,
            copies,
            expected,
            chunks,
            given_persisted_sets,
            claimed_versions,
        }
    }
}

/// 恢复自称的那一版那一张表每个状态几个字（布局写在 [`JudgeTables::words`] 填它的地方；内核里的同名常量由 `crate::crash_judge_gpu` 从这里生成）。
pub const CLAIMED_VERSION_WORDS: usize = 4;
/// 那一张表每个状态第 0 字的码：没写（取看 journal 那一遍落到的根）、写了而那一版没有根、写了一条根（第 1–3 字是它的实例代号与 checkpoint_txg lo/hi）。
pub const CLAIMED_VERSION_LANDED_ON_BY_THE_CONSULTED_RECOVERY: u32 = 0;
pub const CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITHOUT_A_ROOT: u32 = 1;
pub const CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITH_A_ROOT: u32 = 2;

/// 一个状态那一栏恢复自称的那一版在判表里的 [`CLAIMED_VERSION_WORDS`] 个字。
fn claimed_version_words(
    claimed_version: ClaimedVersionOfAGivenState,
) -> [u32; CLAIMED_VERSION_WORDS] {
    match claimed_version {
        ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery => [
            CLAIMED_VERSION_LANDED_ON_BY_THE_CONSULTED_RECOVERY,
            NONE,
            NONE,
            NONE,
        ],
        ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(None) => [
            CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITHOUT_A_ROOT,
            NONE,
            NONE,
            NONE,
        ],
        ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(Some((
            instance,
            checkpoint_txg,
        ))) => [
            CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITH_A_ROOT,
            instance.0,
            u32::try_from(checkpoint_txg.0 & 0xFFFF_FFFF).expect("低 32 位"),
            u32::try_from(checkpoint_txg.0 >> 32).expect("高 32 位"),
        ],
    }
}

/// 段表每条几个 u32（布局写在 [`JudgeTables::words`] 填它的地方；内核里的同名常量由 `crate::crash_judge_gpu` 从这里生成）。
pub const SEGMENT_WORDS: usize = 11;

/// 表里「没有」的写法。
pub const NONE: u32 = u32::MAX;
/// 版本表最前面几条是全零单元，不是哪个落点的版本：第 0 条 32 KiB、第 1 条 16 KiB（盘尾只装得下 16 KiB 的槽；布局写在 [`JudgeTables::words`]
/// 填它们的地方）。GPU 读者 `read_at` 读单元区里一个没有落点的槽时交回它们（判决 `research/prompts/m3-batch29-code-r1-main-verification.md`
/// 第三节第 2 条，攻方改法 F-零槽）：那一槽基线与枚举写表里哪一次写都没碰过，CPU 稀疏读者在每个崩溃状态里读出的都是零。
/// 落点的第 9–11 字说它之后哪几槽是这样的槽（[`UnitSlotsNothingWroteAfterALocation`]）。
pub const ZERO_UNIT_VERSIONS: usize = 2;

/// 一个落点之后、同一块盘上下一个落点之前那一段谁都没写过的槽（落点表第 9–11 字）：在单元区里、整 16 KiB 在盘内，
/// 槽号 [`Self::first_slot`, `Self::end_slot`)（没有这样的槽时两个都是 0）；`Self::first_slot_a_whole_data_unit_does_not_fit_from`
/// 是这块盘上从哪一槽起整 32 KiB 不在盘内（GPU 从那一槽起交 16 KiB 那条全零单元，同真落点在盘尾只有 16 KiB）。槽号都装得进 u32（盘不到 64 TiB）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitSlotsNothingWroteAfterALocation {
    pub first_slot: u32,
    pub end_slot: u32,
    pub first_slot_a_whole_data_unit_does_not_fit_from: u32,
}
/// 写表每条几个 u32。
pub const WRITE_WORDS: usize = 12;
/// 落点表每条几个 u32。
pub const LOCATION_WORDS: usize = 12;
/// 版本表每条几个 u32：组合第 0 字（第 0–31 次写）、字节起点、四个检查字（[`version_checks`]）、逐扇区等不等掩码起点、组合第 1 字
/// （第 32–63 次写）、第 8 字 [`VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES`]，再接组合其余的字
/// （[`version_word_of_the_combination_word`]；组合只有 64 位那时候的九个字偏移不动）。内核里的同名常量由 `crate::crash_judge_gpu` 从这里生成。
pub const VERSION_WORDS: usize =
    VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE + COMBINATION_WORDS_OF_A_VERSION - 2;
/// 版本表第 8 字（[`crc_shifted_past_sixteen_zero_kibibytes`]）：GPU 读者 `read_at` 拼读跨过落点末尾的 32 KiB 时拼整段 CRC 用
/// （`crash_judge_common.wgsl` 同名常量）。
pub const VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES: usize = 8;
/// 版本表里组合第 2 字（第 64–95 次写）在一条版本的第几字：接在第 8 字之后，其余各字依次往后。
pub const VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE: usize =
    VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES + 1;

/// 组合掩码第 `combination_word` 个 u32（[`CombinationOfTheWritesOnALocation::thirty_two_bit_words`]）在一条版本的第几字：第 0 字在第 0 字、
/// 第 1 字在第 7 字，其余从 [`VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE`] 起依次放（`crash_judge_common.wgsl` 的同名函数照同一个布局读）。
///
/// # Panics
/// `combination_word` 不小于 [`COMBINATION_WORDS_OF_A_VERSION`]。
#[must_use]
pub fn version_word_of_the_combination_word(combination_word: usize) -> usize {
    assert!(
        combination_word < COMBINATION_WORDS_OF_A_VERSION,
        "组合掩码只有 {COMBINATION_WORDS_OF_A_VERSION} 个字"
    );
    match combination_word {
        0 => 0,
        1 => 7,
        past_the_sixty_fourth_write => {
            VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE + past_the_sixty_fourth_write - 2
        }
    }
}

/// 一条版本打包进版本表的 [`VERSION_WORDS`] 个字（布局见 [`VERSION_WORDS`]）。
fn packed_version_words(
    combination: &CombinationOfTheWritesOnALocation,
    byte_start: u32,
    checks: [u32; 4],
    equal_masks_start: u32,
    crc_shifted_past_sixteen_zero_kibibytes_of_the_version: u32,
) -> [u32; VERSION_WORDS] {
    let mut words = [0u32; VERSION_WORDS];
    for (combination_word, word) in combination.thirty_two_bit_words().into_iter().enumerate() {
        words[version_word_of_the_combination_word(combination_word)] = word;
    }
    words[1] = byte_start;
    words[2..6].copy_from_slice(&checks);
    words[6] = equal_masks_start;
    words[VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES] =
        crc_shifted_past_sixteen_zero_kibibytes_of_the_version;
    words
}

/// 单元槽的一版字节的前 16 KiB 后面接 16 KiB 全零，那 32 KiB 的 CRC-32C 异或 16 KiB 全零的 CRC-32C（版本表第 8 字）。CRC-32C 对异或线性：
/// 它再异或任意一段 16 KiB 的 CRC-32C，就是「这一版前 16 KiB 接那一段」32 KiB 的 CRC-32C（第 32 批攻方 Z2，判决
/// `research/prompts/m3-batch33s-code-r1-main-verification.md` 第三节第 1 条）；GPU 读者拼「16 KiB 节点加下一槽」「空槽加下一已写槽前 16 KiB」
/// 两形的整段 CRC 靠它，不在内核里做 CRC 的乘法。别的落点（不是单元槽、或不到 16 KiB）记 0：不会被拼读。
#[must_use]
pub fn crc_shifted_past_sixteen_zero_kibibytes(kind: JudgeLocationKind, bytes: &[u8]) -> u32 {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    match kind {
        JudgeLocationKind::UnitSlot if bytes.len() >= node_bytes => {
            let mut followed_by_zeros = bytes[..node_bytes].to_vec();
            followed_by_zeros.resize(2 * node_bytes, 0);
            crc32_castagnoli(&followed_by_zeros) ^ crc32_castagnoli(&vec![0u8; node_bytes])
        }
        JudgeLocationKind::UnitSlot
        | JudgeLocationKind::SystemConfiguration
        | JudgeLocationKind::RootSlot
        | JudgeLocationKind::JournalRecord => 0,
    }
}
/// 更晚重叠写表每条几个 u32。
pub const LATER_WORDS: usize = 4;
/// 发布表每条几个 u32。
pub const PUBLISH_WORDS: usize = 8;
/// 期望版本表每条几个 u32：实例、txg 低高、单元 CRC 表起点、几个单元、有没有文件（第 5 字）。
pub const EXPECTED_WORDS: usize = 6;
/// 期望版本表第 5 字：这一版有文件（`crash_judge_recovery.wgsl` 同名常量）。
pub const EXPECTED_VERSION_WITH_A_FILE: u32 = 0;
/// 期望版本表第 5 字：文件从这一版起没有了（`crash_judge_recovery.wgsl` 同名常量）。
pub const EXPECTED_VERSION_WITHOUT_A_FILE: u32 = 1;
/// 头：12 个计数字段之后是盘的身份，共 32 个 u32（盘不超过 20 块）。
pub const HEADER_WORDS: usize = 32;

/// 摊平的几张 u32 表：GPU 内核的绑定各占一张。布局写在 [`JudgeTables::words`] 里填它们的地方。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgeTablesWords {
    pub header: Vec<u32>,
    pub recorded: Vec<u32>,
    pub replays: Vec<u32>,
    pub segments: Vec<u32>,
    pub segment_writes: Vec<u32>,
    pub writes: Vec<u32>,
    pub root_bytes: Vec<u32>,
    pub laters: Vec<u32>,
    pub locations: Vec<u32>,
    pub location_writes: Vec<u32>,
    pub versions: Vec<u32>,
    pub version_bytes: Vec<u32>,
    pub equal_masks: Vec<u32>,
    pub publishes: Vec<u32>,
    pub publish_records: Vec<u32>,
    pub copy_groups: Vec<u32>,
    pub copies: Vec<u32>,
    pub expected: Vec<u32>,
    pub chunks: Vec<u32>,
    /// 给定状态表的持久掩码（[`JudgeStateSource::GivenPersistedSets`]）；按段展开时是空的，内核按它空不空认状态从哪来。
    pub given_persisted_sets: Vec<u32>,
    /// 给定状态表每个状态那一栏恢复自称的那一版（[`CLAIMED_VERSION_WORDS`] 个字一个）；按段展开时是空的，内核按它空不空认要不要照它判第二条判据。
    pub claimed_versions: Vec<u32>,
}

impl JudgeTablesWords {
    /// 全部表按固定次序接成字节（摘要与上卡都用它）。
    #[must_use]
    pub fn digest_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for table in self.every_table() {
            out.extend_from_slice(&u32::try_from(table.len()).expect("表长").to_le_bytes());
            for word in table {
                out.extend_from_slice(&word.to_le_bytes());
            }
        }
        out
    }

    /// 每张表的名字，与 [`Self::every_table`] 同序（建表核目录时报哪张表满了用；内核里的表号常量也照这个次序）。
    #[must_use]
    pub fn every_table_name() -> [&'static str; JUDGE_TABLE_COUNT] {
        [
            "header",
            "recorded",
            "replays",
            "segments",
            "segment_writes",
            "writes",
            "root_bytes",
            "laters",
            "locations",
            "location_writes",
            "versions",
            "version_bytes",
            "equal_masks",
            "publishes",
            "publish_records",
            "copy_groups",
            "copies",
            "expected",
            "chunks",
            "given_persisted_sets",
            "claimed_versions",
        ]
    }

    /// 全部表，按绑定的次序。
    #[must_use]
    pub fn every_table(&self) -> [&Vec<u32>; JUDGE_TABLE_COUNT] {
        [
            &self.header,
            &self.recorded,
            &self.replays,
            &self.segments,
            &self.segment_writes,
            &self.writes,
            &self.root_bytes,
            &self.laters,
            &self.locations,
            &self.location_writes,
            &self.versions,
            &self.version_bytes,
            &self.equal_masks,
            &self.publishes,
            &self.publish_records,
            &self.copy_groups,
            &self.copies,
            &self.expected,
            &self.chunks,
            &self.given_persisted_sets,
            &self.claimed_versions,
        ]
    }
}

/// 打包表里几张表（内核里的 `TABLE_COUNT` 由 `crate::crash_judge_gpu` 从这里生成）。
pub const JUDGE_TABLE_COUNT: usize = 21;

/// 字节按小端 4 个一组装进 u32，末尾补 0。
#[must_use]
pub fn bytes_to_words(bytes: &[u8]) -> Vec<u32> {
    let (chunks, remainder) = bytes.as_chunks::<4>();
    let mut words: Vec<u32> = chunks
        .iter()
        .map(|chunk| u32::from_le_bytes(*chunk))
        .collect();
    if !remainder.is_empty() {
        let mut last = [0u8; 4];
        last[..remainder.len()].copy_from_slice(remainder);
        words.push(u32::from_le_bytes(last));
    }
    words
}

#[cfg(test)]
mod tests {
    use super::build_judge_tables_for_the_given_persisted_sets;
    use super::build_judge_tables_for_the_given_persisted_sets_of_the_recorded_writes_broken_at_the_seams_with_their_claimed_versions;
    use super::{
        build_judge_tables, crc_shifted_past_sixteen_zero_kibibytes, judge_publishes,
        version_word_of_the_combination_word, ClaimedVersionOfAGivenState, JudgeFlow,
        JudgeLocationKind, JudgePublish, JudgeStateSource, JudgeTables, JudgeTablesRefusal,
        JudgeTablesWords, LOCATION_WORDS, NONE, SEGMENT_WORDS, VERSION_WORDS,
    };
    use crate::crash::CrashPointReplaySegmentExpansion;
    use crate::crash_facts::{
        SegmentExpansionFact, SegmentFact, MAXIMUM_ENUMERATED_WRITES,
        SEGMENT_EXPANSION_CODE_IN_PLACE_SUBSETS_WITH_COPY_ON_WRITE_NONE_OR_ALL,
    };
    use singlefs_core::address::{
        CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    };
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;
    use singlefs_core::checksum::crc32_castagnoli;
    use singlefs_core::journal::slot_after_the_journal_ring;
    use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
    use singlefs_core::recovery::PoolReader;
    use singlefs_core::root_ring::RootRingSlotsPerRegion;
    use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableSizes};
    use singlefs_format::{
        JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES,
        SYSTEM_CONFIGURATION_SLOT_BYTES,
    };
    use singlefs_harness::memory_pool::{
        writes_and_segments, CrashImage, MemoryPool, RecordStreamSeam, RetainedWrite,
        SparseBlockDevice, WrittenContents, SECTOR_BYTES,
    };
    use singlefs_harness::segments::{FixedGeometry, StepKind};
    use singlefs_harness::{RecordingBlockDevice, SharedStream};

    /// journal 环起点字段写成第一版常量 1024 之外的一个槽号：自证得过、读者不收（与 harness 场景
    /// `system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` 的同名常量同一个值）。
    const JOURNAL_RING_START_SLOT_THE_READER_REFUSES: u64 = 1025;

    /// 只有掩码计划那几样的判表（没有落点、写、发布）：words() 摊段表与给定状态表那两张看得出布局。
    fn tables_of_one_segment(
        segment: SegmentFact,
        expansion: SegmentExpansionFact,
        state_source: JudgeStateSource,
        state_count: u64,
    ) -> JudgeTables {
        JudgeTables {
            recorded_write_count: 5,
            enumerated_write_count: 40,
            is_tearable: vec![false; 5],
            torn_images: vec![],
            segments: vec![segment],
            segment_expansions: vec![expansion],
            state_source,
            state_count,
            devices: vec![0],
            device_bytes: 1 << 26,
            unit_area_start_in_bytes: 1 << 24,
            judged_root_index: 0,
            writes: vec![],
            locations: vec![],
            publishes: vec![],
            expected_versions: vec![],
        }
    }

    /// 段表一条 SEGMENT_WORDS 个字：甲二段先摊原地写那一组（数位）、再摊单元写那一组（整组落），两组合起来是段里全部的写，展开方式码在第 6 字。
    #[test]
    fn a_segment_in_place_subsets_with_copy_on_write_none_or_all_lays_out_its_two_groups_of_writes()
    {
        let tables = tables_of_one_segment(
            SegmentFact {
                writes: vec![2, 3, 4, 0, 1],
                states: 0..7,
                slice_start: None,
            },
            SegmentExpansionFact::InPlaceSubsetsWithCopyOnWriteNoneOrAll {
                in_place_writes: vec![0, 1],
                copy_on_write_writes: vec![2, 3, 4],
            },
            JudgeStateSource::SegmentExpansions,
            8,
        );
        let words = tables.words();
        assert_eq!(words.segments.len(), SEGMENT_WORDS);
        assert_eq!(
            words.segments,
            vec![
                0,
                2,
                0,
                0,
                7,
                0,
                SEGMENT_EXPANSION_CODE_IN_PLACE_SUBSETS_WITH_COPY_ON_WRITE_NONE_OR_ALL,
                2,
                3,
                NONE,
                0
            ]
        );
        assert_eq!(words.segment_writes, vec![0, 1, 2, 3, 4]);
        assert!(
            words.given_persisted_sets.is_empty(),
            "按段展开时给定状态表是空的"
        );
        assert_eq!(
            tables.persisted_writes_of_state(1)[2..5],
            [true, true, true],
            "序号 1 是单元写全落"
        );
    }

    /// 分片的段（C790）：段表第 9、10 字是片起点各位在段写表里的起点与位数，各位接在两组写之后（单元写那一组那一位在前）；
    /// 判表按片起点加片内序号拆，与不分片时同一个段内编号拆出的相同。
    #[test]
    fn a_sliced_segment_lays_out_the_digits_of_its_slice_start_after_its_two_groups_of_writes() {
        let expansion = SegmentExpansionFact::InPlaceSubsetsWithCopyOnWriteNoneOrAll {
            in_place_writes: vec![0, 1],
            copy_on_write_writes: vec![2, 3, 4],
        };
        let whole = tables_of_one_segment(
            SegmentFact {
                writes: vec![2, 3, 4, 0, 1],
                states: 0..7,
                slice_start: None,
            },
            expansion.clone(),
            JudgeStateSource::SegmentExpansions,
            8,
        );
        let sliced = tables_of_one_segment(
            SegmentFact {
                writes: vec![2, 3, 4, 0, 1],
                states: 0..2,
                slice_start: Some(crate::crash_facts::SegmentSliceStartFact::of(
                    crate::crash::StateCount1280Bits::from_u64(5),
                    &[2, 2, 2],
                )),
            },
            expansion,
            JudgeStateSource::SegmentExpansions,
            3,
        );
        let words = sliced.words();
        assert_eq!(words.segments[9..], [5, 3], "各位从段写表第 5 个字起、三位");
        assert_eq!(
            words.segment_writes,
            vec![0, 1, 2, 3, 4, 1, 0, 1],
            "5 = 1 + 2 × (0 + 2 × 1)：单元写那一位是 1，两次原地写那两位是 0、1"
        );
        for ordinal_within_the_slice in 0..2 {
            assert_eq!(
                sliced.persisted_writes_of_state(ordinal_within_the_slice),
                whole.persisted_writes_of_state(5 + ordinal_within_the_slice),
                "片内第 {ordinal_within_the_slice} 个"
            );
        }
    }

    /// 给定状态表：每个状态 ⌈枚举写数 ÷ 32⌉ 个字的掩码按序号接；序号 k 的持久集合就是清单里第 k 个。
    #[test]
    fn given_persisted_sets_are_laid_out_one_mask_per_state_and_read_back_by_ordinal() {
        let mut first = vec![false; 40];
        first[0] = true;
        first[33] = true;
        let mut second = vec![false; 40];
        second[39] = true;
        let tables = tables_of_one_segment(
            SegmentFact {
                writes: vec![0, 1, 2, 3, 4],
                states: 0..31,
                slice_start: None,
            },
            SegmentExpansionFact::EveryProperSubset,
            JudgeStateSource::GivenPersistedSets {
                persisted_sets: vec![first.clone(), second.clone()],
                claimed_versions: vec![
                    ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery;
                    2
                ],
            },
            2,
        );
        assert_eq!(
            tables.words().given_persisted_sets,
            vec![1, 1 << 1, 0, 1 << 7]
        );
        assert_eq!(tables.persisted_writes_of_state(0), first);
        assert_eq!(tables.persisted_writes_of_state(1), second);
        assert_eq!(tables.check_the_packed_directory(), Ok(()));
    }

    /// 表名与表一一对应、不重。
    #[test]
    fn every_table_has_one_distinct_name() {
        let mut names = JudgeTablesWords::every_table_name().to_vec();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), JudgeTablesWords::every_table_name().len());
    }

    /// 枚举写表多于内核持久掩码的宽（[`MAXIMUM_ENUMERATED_WRITES`] 次）：建表交「掩码表满了」，不 panic、不往下建。
    #[test]
    fn a_flow_with_more_enumerated_writes_than_the_mask_holds_is_refused_naming_the_mask() {
        let base = MemoryPool::with_devices(&[DeviceIdentity(0)], 1 << 26);
        let one_past_the_mask =
            u64::try_from(MAXIMUM_ENUMERATED_WRITES + 1).expect("掩码宽装得进 u64");
        let writes: Vec<RetainedWrite> = (0..one_past_the_mask)
            .map(|write_index| RetainedWrite {
                device: DeviceIdentity(0),
                kind: StepKind::JournalRecord,
                is_force_unit_access: false,
                offset: DeviceOffsetInBytes((1 << 22) + write_index * 4096),
                contents: WrittenContents::Bytes(vec![1u8; 4096]),
            })
            .collect();
        let segments: Vec<Vec<usize>> = (0..writes.len())
            .map(|write_index| vec![write_index])
            .collect();
        let expansion = |_segment_index: usize, _segment: &[usize]| {
            CrashPointReplaySegmentExpansion::NotExpanded
        };
        let refusal = build_judge_tables(&JudgeFlow {
            base: &base,
            writes: &writes,
            segments: &segments,
            judged_root_index: 0,
            versions: &[],
            expansion: &expansion,
        })
        .expect_err("比掩码宽多一次写，装不进掩码");
        assert_eq!(
            refusal,
            JudgeTablesRefusal::EnumeratedWritesPastTheMaskWidth {
                enumerated_writes: MAXIMUM_ENUMERATED_WRITES + 1
            }
        );
    }

    /// 随机历史越过原分配记录墙那一族最大的一张给定状态表（2026-10-18 调查员量的）：枚举写表 5709 次写（接撕裂镜像时；录制写最多 5265）。
    const ENUMERATED_WRITES_OF_THE_LARGEST_ALLOCATION_RECORD_WALL_TABLE: usize = 5709;
    /// 同一族里盘 0 的系统配置槽 0 号最多被录制写了几次。
    const WRITES_ON_THE_FIRST_SYSTEM_CONFIGURATION_SLOT_OF_THE_LARGEST_ALLOCATION_RECORD_WALL_TABLE: usize =
        111;
    /// 抬上限之前的两个上限：枚举写表 2048 次、一个落点 64 次写。
    const ENUMERATED_WRITES_THE_MASK_HELD_BEFORE_THE_RAISE: usize = 2048;
    const WRITES_PER_LOCATION_THE_COMBINATION_HELD_BEFORE_THE_RAISE: usize = 64;

    /// 两个上限抬到装得下随机历史越过原分配记录墙那一族：mkfs 之后的两块盘上，盘 0 的系统配置槽 0 号写 111 次、journal 环里再写
    /// 5598 条记录（共 5709 次写，录制流本身，不接撕裂镜像），两个持久掩码（一个都没落、全落）。建得出判表；那一槽 111 次写，
    /// 全落那一版组合的第 110 位是 1，打包进版本表时落在组合第 3 字（第 96–127 次写）的第 14 位、第 0 字全 1。
    /// 判别力（`crates/mutations.tsv` 两行）：枚举写表的上限改回 2048，建表交「掩码表满了」，这条红；一个落点的写数上限改回 64，
    /// 建表交「一个落点写太多」，这条红。
    #[test]
    fn a_given_states_table_of_five_thousand_seven_hundred_and_nine_writes_with_one_hundred_and_eleven_on_one_location_is_built(
    ) {
        let base = formatted_pool_of_two_devices();
        let system_configuration_slot_writes = (0
            ..WRITES_ON_THE_FIRST_SYSTEM_CONFIGURATION_SLOT_OF_THE_LARGEST_ALLOCATION_RECORD_WALL_TABLE)
            .map(|rewrite| RetainedWrite {
                device: DeviceIdentity(0),
                kind: StepKind::SystemConfigurationSlot,
                is_force_unit_access: true,
                offset: DeviceOffsetInBytes(0),
                contents: WrittenContents::Bytes(vec![
                    u8::try_from(rewrite % 251 + 1).expect("小于 256");
                    usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")
                ]),
            });
        let journal_ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
        let journal_record_writes = (0..ENUMERATED_WRITES_OF_THE_LARGEST_ALLOCATION_RECORD_WALL_TABLE
            - WRITES_ON_THE_FIRST_SYSTEM_CONFIGURATION_SLOT_OF_THE_LARGEST_ALLOCATION_RECORD_WALL_TABLE)
            .map(|record| RetainedWrite {
                device: DeviceIdentity(0),
                kind: StepKind::JournalRecord,
                is_force_unit_access: false,
                offset: DeviceOffsetInBytes(
                    journal_ring_start
                        + u64::try_from(record).expect("记录序号装得进 u64") * JOURNAL_RECORD_BYTES,
                ),
                contents: WrittenContents::Bytes(vec![
                    0x5a;
                    usize::try_from(JOURNAL_RECORD_BYTES)
                        .expect("4096")
                ]),
            });
        let writes: Vec<RetainedWrite> = system_configuration_slot_writes
            .chain(journal_record_writes)
            .collect();
        assert!(
            writes.len() > ENUMERATED_WRITES_THE_MASK_HELD_BEFORE_THE_RAISE
                && WRITES_ON_THE_FIRST_SYSTEM_CONFIGURATION_SLOT_OF_THE_LARGEST_ALLOCATION_RECORD_WALL_TABLE
                    > WRITES_PER_LOCATION_THE_COMBINATION_HELD_BEFORE_THE_RAISE,
            "前提：两样都越过抬之前的上限"
        );
        let segments = vec![(0..writes.len()).collect::<Vec<usize>>()];
        let not_expanded = |_segment_index: usize, _segment: &[usize]| {
            CrashPointReplaySegmentExpansion::NotExpanded
        };
        let every_write_persisted = vec![true; writes.len()];
        let tables =
            build_judge_tables_for_the_given_persisted_sets_of_the_recorded_writes_broken_at_the_seams_with_their_claimed_versions(
                &JudgeFlow {
                    base: &base,
                    writes: &writes,
                    segments: &segments,
                    judged_root_index: 0,
                    versions: &[],
                    expansion: &not_expanded,
                },
                &[],
                vec![vec![false; writes.len()], every_write_persisted.clone()],
                &[ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery; 2],
                &[],
            )
            .unwrap_or_else(|refusal| panic!("两个上限装得下这张表：{refusal:?}"));
        assert_eq!(
            usize::try_from(tables.enumerated_write_count).expect("装得进 usize"),
            ENUMERATED_WRITES_OF_THE_LARGEST_ALLOCATION_RECORD_WALL_TABLE,
            "枚举写表就是录制流"
        );
        let (location_index, first_system_configuration_slot) = tables
            .locations
            .iter()
            .enumerate()
            .find(|(_, location)| location.device == 0 && location.offset == 0)
            .expect("盘 0 的系统配置槽 0 号是判表里的一个落点");
        assert_eq!(
            first_system_configuration_slot.writes.len(),
            WRITES_ON_THE_FIRST_SYSTEM_CONFIGURATION_SLOT_OF_THE_LARGEST_ALLOCATION_RECORD_WALL_TABLE,
            "那一槽 111 次写"
        );
        let last_write_position =
            WRITES_ON_THE_FIRST_SYSTEM_CONFIGURATION_SLOT_OF_THE_LARGEST_ALLOCATION_RECORD_WALL_TABLE
                - 1;
        let every_write_persisted_version = tables
            .version_of(first_system_configuration_slot, &every_write_persisted)
            .expect("全落那个掩码在那一槽有一版");
        assert!(
            every_write_persisted_version
                .combination
                .is_persisted(last_write_position),
            "全落那一版组合的第 110 位是 1"
        );
        let version_index = first_system_configuration_slot
            .versions
            .iter()
            .position(|version| version == every_write_persisted_version)
            .expect("那一版在那一槽的版本表里");
        let words = tables.words();
        let first_version_of_the_slot =
            usize::try_from(words.locations[location_index * LOCATION_WORDS + 7])
                .expect("装得进 usize");
        let packed_version = &words.versions[(first_version_of_the_slot + version_index)
            * VERSION_WORDS
            ..(first_version_of_the_slot + version_index + 1) * VERSION_WORDS];
        assert_eq!(
            (
                packed_version[version_word_of_the_combination_word(0)],
                packed_version[version_word_of_the_combination_word(last_write_position / 32)]
                    & (1 << (last_write_position % 32))
            ),
            (u32::MAX, 1 << (last_write_position % 32)),
            "版本表里组合第 0 字全 1、第 110 次写落在组合第 3 字的第 14 位：{packed_version:?}"
        );
    }

    /// 一次合成的写：盘 0、`offset` 起 4096 字节；根槽写的字节里实例代号在偏移 24、checkpoint_txg 在偏移 28（`root_identity_of_write` 的读法）。
    fn synthetic_write(
        kind: StepKind,
        offset: u64,
        root_identity: Option<(u32, u64)>,
    ) -> RetainedWrite {
        let mut bytes = vec![0u8; 4096];
        if let Some((instance, checkpoint_txg)) = root_identity {
            bytes[24..28].copy_from_slice(&instance.to_le_bytes());
            bytes[28..36].copy_from_slice(&checkpoint_txg.to_le_bytes());
        }
        RetainedWrite {
            device: DeviceIdentity(0),
            kind,
            is_force_unit_access: kind == StepKind::RootRecordFua,
            offset: DeviceOffsetInBytes(offset),
            contents: WrittenContents::Bytes(bytes),
        }
    }

    /// 历史两次发布（实例 1 的 txg 1、2）、挂载一次发布（实例 2 的 txg 2）：写表下标 0..6 是历史、6..8 是挂载。
    fn history_of_two_publishes_then_a_mount_of_one() -> Vec<RetainedWrite> {
        vec![
            synthetic_write(StepKind::UnitWrite, 1 << 24, None),
            synthetic_write(StepKind::JournalRecord, 1 << 22, None),
            synthetic_write(StepKind::RootRecordFua, 1 << 20, Some((1, 1))),
            synthetic_write(StepKind::UnitWrite, (1 << 24) + (1 << 15), None),
            synthetic_write(StepKind::JournalRecord, (1 << 22) + 4096, None),
            synthetic_write(StepKind::RootRecordFua, (1 << 20) + 4096, Some((1, 2))),
            synthetic_write(StepKind::UnitWrite, (1 << 24) + (2 << 15), None),
            synthetic_write(StepKind::RootRecordFua, (1 << 20) + 8192, Some((2, 2))),
        ]
    }

    /// 给定状态表的发布表认崩溃接缝（用户 2026-10-10 定第三截照候选丁）：第一次崩溃截在历史第二次发布的根之前、恢复落到 txg 1，
    /// 接缝放在历史与挂载之间时，历史那一截只留落了根的那一次，挂载那次发布只带挂载写的单元；接缝放错一个位置（放在历史两次发布之间），
    /// 历史第二次发布照样进表、挂载的单元也分错——两份发布表不同，GPU 的记录核对器读的就是这一张，判得出接缝放错了。
    #[test]
    fn the_publishes_of_given_persisted_sets_follow_where_the_crash_seam_is_put() {
        let writes = history_of_two_publishes_then_a_mount_of_one();
        let persisted = vec![true, true, true, true, true, false, false, false];
        let state_source = JudgeStateSource::GivenPersistedSets {
            persisted_sets: vec![persisted],
            claimed_versions: vec![ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery],
        };
        let seam_at = |first_write_after_the_crash: usize| RecordStreamSeam::AfterACrash {
            first_write_after_the_crash,
            version_landed_on_after_the_crash: Some((InstanceGeneration(1), CheckpointTxg(1))),
        };
        let at_the_seam = judge_publishes(&writes, writes.len(), &state_source, &[seam_at(6)])
            .expect("历史那一截的根槽写只有一个掩码，核得过");
        let roots_and_units = |publishes: &[JudgePublish]| -> Vec<(u32, u32, u64, Vec<Vec<u32>>)> {
            publishes
                .iter()
                .map(|publish| {
                    (
                        publish.root_write,
                        publish.instance,
                        publish.checkpoint_txg,
                        publish.unit_copy_groups.clone(),
                    )
                })
                .collect()
        };
        assert_eq!(
            roots_and_units(&at_the_seam),
            vec![(2, 1, 1, vec![vec![0]]), (7, 2, 2, vec![vec![6]])],
            "接缝在历史与挂载之间：历史第二次发布没落根、恢复也没落到它，剔掉；挂载那次只带挂载写的单元"
        );
        let one_place_early = judge_publishes(&writes, writes.len(), &state_source, &[seam_at(3)])
            .expect("接缝之前那一截只有一个根槽写，核得过");
        assert_ne!(
            roots_and_units(&one_place_early),
            roots_and_units(&at_the_seam),
            "接缝放错位置，发布表不同"
        );
        assert_eq!(
            roots_and_units(
                &judge_publishes(&writes, writes.len(), &state_source, &[])
                    .expect("不带接缝是一条录制流")
            ),
            vec![
                (2, 1, 1, vec![vec![0]]),
                (5, 1, 2, vec![vec![3]]),
                (7, 2, 2, vec![vec![6]])
            ],
            "不带接缝时三次发布全在表里（今天的一条录制流）"
        );
    }

    /// 崩溃接缝之前某个根槽写在两个掩码里一落一不落：一张表只能按一种剔法分发布，建表拒、点名那个根槽写与掩码，不 panic。
    #[test]
    fn given_persisted_sets_that_disagree_on_a_root_write_before_the_crash_seam_are_refused() {
        let writes = history_of_two_publishes_then_a_mount_of_one();
        let landed_on_the_first = vec![true, true, true, true, true, false, false, false];
        let landed_on_the_second = vec![true, true, true, true, true, true, true, false];
        let refusal = judge_publishes(
            &writes,
            writes.len(),
            &JudgeStateSource::GivenPersistedSets {
                persisted_sets: vec![landed_on_the_first, landed_on_the_second],
                claimed_versions: vec![
                    ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery;
                    2
                ],
            },
            &[RecordStreamSeam::AfterACrash {
                first_write_after_the_crash: 6,
                version_landed_on_after_the_crash: Some((InstanceGeneration(1), CheckpointTxg(1))),
            }],
        )
        .expect_err("第 5 个写（历史第二次发布的根）在两个掩码里一落一不落");
        assert_eq!(
            refusal,
            JudgeTablesRefusal::GivenPersistedSetsDisagreeOnARootWriteBeforeACrashSeam {
                root_write: 5,
                position: 1
            }
        );
    }

    const SIXTEEN_KIBIBYTES: usize = 16384;

    fn sixteen_kibibytes_with_the_pattern(multiplier: usize, modulus: usize) -> Vec<u8> {
        (0..SIXTEEN_KIBIBYTES)
            .map(|index| u8::try_from(index * multiplier % modulus).expect("小于 256"))
            .collect()
    }

    /// 版本表第 8 字异或任意一段 16 KiB 的 CRC-32C，就是「这一版前 16 KiB 接那一段」32 KiB 的 CRC-32C：GPU 读者拼读空段最后一槽
    /// （前一半全零 16 KiB、后一半是下一个落点的前 16 KiB）靠这一条；前一半不是全零时同样成立。不是单元槽的落点记 0。
    #[test]
    fn the_shifted_crc_of_a_version_xored_with_the_crc_of_the_next_sixteen_kibibytes_is_the_crc_of_the_thirty_two_kibibytes(
    ) {
        let next = sixteen_kibibytes_with_the_pattern(13, 253);
        for first in [
            vec![0u8; SIXTEEN_KIBIBYTES],
            sixteen_kibibytes_with_the_pattern(7, 251),
        ] {
            let mut thirty_two_kibibytes = first.clone();
            thirty_two_kibibytes.extend_from_slice(&next);
            assert_eq!(
                crc_shifted_past_sixteen_zero_kibibytes(JudgeLocationKind::UnitSlot, &first)
                    ^ crc32_castagnoli(&next),
                crc32_castagnoli(&thirty_two_kibibytes),
                "拼出的 32 KiB CRC 与连续读出的 32 KiB 的 CRC 相同"
            );
        }
        assert_eq!(
            crc_shifted_past_sixteen_zero_kibibytes(
                JudgeLocationKind::JournalRecord,
                &next[..4096]
            ),
            0,
            "journal 记录槽不会被拼读，记 0"
        );
    }

    /// 两块盘 mkfs 一次的池的盘宽：与 harness 档建池用的盘同宽（稀疏盘，只存写过的扇区）。
    const FORMATTED_DEVICE_BYTES: u64 = 4 << 30;

    /// 两块盘 mkfs 一次的池（io_min 512、槽距 4096；根环区域 0 / 1 / 2 在盘 0 / 1 / 0 上），没有文件。
    fn formatted_pool_of_two_devices() -> MemoryPool {
        formatted_pool_of_two_devices_with_io_min(512)
    }

    /// 两块盘 mkfs 一次的池，mkfs 探测到的 io_min 是 `minimum_input_output_bytes`（槽距 = 4096 向上取整到它的整数倍；
    /// 物理块 512；根环区域 0 / 1 / 2 在盘 0 / 1 / 0 上），没有文件。
    fn formatted_pool_of_two_devices_with_io_min(minimum_input_output_bytes: u32) -> MemoryPool {
        let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
            [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(|identity| {
                    (
                        identity,
                        SparseBlockDevice::new(
                            FORMATTED_DEVICE_BYTES,
                            PhysicalBlockSizeInBytes(512),
                        ),
                    )
                })
                .collect();
        make_filesystem(
            &MakeFilesystemParameters {
                filesystem_identifier: *b"singlefs-tables!",
                region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
                geometry: SystemImmutableSizes {
                    physical_block_size: 512,
                    minimum_input_output_bytes,
                    fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(
                        minimum_input_output_bytes,
                    ),
                    journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
                    root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
                },
            },
            &mut devices,
        )
        .expect("两块空盘上 mkfs 做得成");
        MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.image.clone()))
                .collect(),
            device_size_in_bytes: FORMATTED_DEVICE_BYTES,
        }
    }

    /// 一张镜像建成的给定状态表（写表为空、一个空的持久集合：harness 场景交出来的镜像在给定状态引擎里就是这样建表的）。
    fn judge_tables_of_one_image(image: &MemoryPool) -> Result<JudgeTables, JudgeTablesRefusal> {
        let not_expanded = |_segment_index: usize, _segment: &[usize]| {
            CrashPointReplaySegmentExpansion::NotExpanded
        };
        build_judge_tables_for_the_given_persisted_sets(
            &JudgeFlow {
                base: image,
                writes: &[],
                segments: &[],
                judged_root_index: 0,
                versions: &[],
                expansion: &not_expanded,
            },
            &[],
            vec![Vec::new()],
        )
    }

    /// 系统配置槽那一类落点在哪几处（盘、偏移），升序。
    fn system_configuration_locations_of(tables: &JudgeTables) -> Vec<(u32, u64)> {
        tables
            .locations
            .iter()
            .filter(|location| location.kind == JudgeLocationKind::SystemConfiguration)
            .map(|location| (location.device, location.offset))
            .collect()
    }

    /// 盘 0 两槽与盘 1 的槽 0 都抹成 0、只剩盘 1 的槽 1 自证得过（CPU 恢复与池级 checker 都择它、照常判）：判表照样建得出，
    /// 落点几何取的就是那一槽——单元区起点是 journal 环末尾的下一槽，两块盘各有偏移 0 与槽距那两个系统配置槽的落点。
    /// 只读第一块盘槽 0 的取法在这里交 `BaseSystemConfigurationUnreadable`，这条红。
    #[test]
    fn the_judge_tables_take_their_geometry_from_the_one_self_verified_slot_one_of_the_second_device(
    ) {
        let mut image = formatted_pool_of_two_devices();
        let spacing = u64::from(SystemImmutableSizes::slot_spacing_for(512));
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        for (device, offset) in [
            (DeviceIdentity(0), 0),
            (DeviceIdentity(0), spacing),
            (DeviceIdentity(1), 0),
        ] {
            image
                .devices
                .get_mut(&device)
                .expect("池里有这块盘")
                .write(DeviceOffsetInBytes(offset), &vec![0u8; slot_bytes]);
        }
        let tables = judge_tables_of_one_image(&image)
            .expect("盘 1 的槽 1 自证得过：判表建得出，不报基线系统配置读不出");
        assert_eq!(
            tables.unit_area_start_in_bytes,
            slot_after_the_journal_ring(JOURNAL_RING_DEFAULT_BYTES).0 * SLOT_BYTES,
            "单元区起点取自盘 1 槽 1 记的环长"
        );
        assert_eq!(
            system_configuration_locations_of(&tables),
            vec![(0, 0), (0, spacing), (1, 0), (1, spacing)],
            "两块盘各有偏移 0 与槽距（盘 1 槽 1 记的 {spacing}）两个系统配置槽的落点"
        );
    }

    /// 两块盘的槽 0 都抹成 0、槽距处本池的槽 1 照旧，两倍槽距处另有一份世代号更大、记的槽距就是两倍槽距的槽 1（自证得过、读者收）：
    /// 恢复逐档读槽 1、取世代号最大的那一份（GPU 两个内核同），落点几何取的就是它（槽距是两倍槽距），与恢复择的逐项相同。
    /// 判别力（`crates/mutations.tsv`）：落点几何照旧取第一份自证得过的（槽距处那一份），几何与恢复、GPU 不同源，这条红。
    #[test]
    fn the_judge_tables_take_their_geometry_from_the_slot_one_the_recovery_reader_chooses_by_its_generation(
    ) {
        let mut image = formatted_pool_of_two_devices();
        let spacing = u64::from(SystemImmutableSizes::slot_spacing_for(512));
        let farther_spacing = 2 * spacing;
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let mut farther = SystemConfiguration::parse_slot(
                &PoolReader::read(&image, device, DeviceOffsetInBytes(spacing), slot_bytes)
                    .expect("槽 1 读得到"),
            )
            .expect("mkfs 写下的槽 1 自证得过");
            farther.immutable.sizes.fixed_structure_slot_spacing =
                u32::try_from(farther_spacing).expect("两倍槽距装得进 4 字节");
            farther.quantities.slot_generation += 10;
            let pool_device = image.devices.get_mut(&device).expect("池里有这块盘");
            pool_device.write(DeviceOffsetInBytes(farther_spacing), &farther.to_slot());
            pool_device.write(DeviceOffsetInBytes(0), &vec![0u8; slot_bytes]);
        }
        let chosen_by_the_recovery_reader =
            singlefs_core::recovery::choose_system_configuration(&image)
                .expect("恢复逐档读槽 1 择得出");
        assert_eq!(
            u64::from(
                chosen_by_the_recovery_reader
                    .immutable
                    .sizes
                    .fixed_structure_slot_spacing
            ),
            farther_spacing,
            "前提：恢复取世代号更大的那一份（两倍槽距那一槽）"
        );
        let geometry = super::base_geometry(&image).expect("基线的落点几何取得出");
        assert_eq!(
            geometry.fixed_structure_slot_spacing, farther_spacing,
            "落点几何取恢复择的那一份：槽距是两倍槽距"
        );
    }

    /// 两块盘各两槽都抹成 0：一份自证得过的系统配置都没有，判表交 `BaseSystemConfigurationUnreadable`、不建表。
    #[test]
    fn the_judge_tables_are_refused_when_no_system_configuration_slot_of_any_device_verifies_itself(
    ) {
        let mut image = formatted_pool_of_two_devices();
        let spacing = u64::from(SystemImmutableSizes::slot_spacing_for(512));
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            for offset in [0, spacing] {
                image
                    .devices
                    .get_mut(&device)
                    .expect("池里有这块盘")
                    .write(DeviceOffsetInBytes(offset), &vec![0u8; slot_bytes]);
            }
        }
        assert_eq!(
            judge_tables_of_one_image(&image).err(),
            Some(JudgeTablesRefusal::BaseSystemConfigurationUnreadable),
            "每块盘两槽都自证不过：落点几何无从取"
        );
    }

    /// 只剩盘 1 的池（盘 0 整块不在池里，而系统配置记的根环区域 0 与 2 在盘 0 上）：判表建得出、不 panic，
    /// 盘 0 上一个落点都不建（CPU 读那块盘交 None，GPU 在那块盘上找不到落点、读到没有版本的切片），盘 1 上的根环区域 1 照建。
    /// 给区域盘一律建落点的取法从基线读盘 0 的字节读不出，在「落点在盘内、按扇区对齐」那一处 panic，这条红。
    #[test]
    fn a_pool_without_a_device_that_holds_root_ring_regions_builds_its_tables_with_no_location_on_that_device(
    ) {
        let mut image = formatted_pool_of_two_devices();
        image
            .devices
            .remove(&DeviceIdentity(0))
            .expect("mkfs 之后池里有盘 0");
        let tables = judge_tables_of_one_image(&image)
            .expect("只剩盘 1 的池：判表建得出（盘 1 的系统配置槽自证得过）");
        assert_eq!(tables.devices, vec![1], "池里只有盘 1");
        assert!(
            tables.locations.iter().all(|location| location.device == 1),
            "落点都在盘 1 上：{:?}",
            tables
                .locations
                .iter()
                .map(|location| (location.device, location.offset, location.kind))
                .filter(|(device, _, _)| *device != 1)
                .collect::<Vec<_>>()
        );
        assert!(
            tables
                .locations
                .iter()
                .any(|location| location.kind == JudgeLocationKind::RootSlot),
            "盘 1 上的根环区域 1 照建落点"
        );
    }

    /// 枚举写表里一次 discard 按 64 槽整段发、罩住它之前写下的一份数据单元（C829（discard 与 mkfs 清零在段种类串里同名） 把它从清零里分出来之后，
    /// GPU 判器的落点模型接它）：判表建得出；discard 那一行不落在任何一个落点上，它罩住的 64 个单元槽各是一个落点、各把它记进自己的写；
    /// 那份数据单元的落点照 D13（验证路线） 已定项 4 的三态各有一个版本——discard 持久那一版读回全 0、没持久那一版是数据单元的字节、
    /// 撕裂那一版是撕裂镜像（`crash::torn_image_of_in_place_overwrite` 按「这次写之前的写全落了」时的旧字节算的）在这一槽上的那一截。
    /// 照旧在建表时拒 discard，或把 discard 当成落在它起点那一个落点上（长过那个落点就拒），这条在建表那一步就红。
    #[test]
    fn a_discard_over_a_data_unit_lands_in_every_unit_slot_location_it_covers_in_its_three_states()
    {
        let base = formatted_pool_of_two_devices();
        let unit_area_start_slot = slot_after_the_journal_ring(JOURNAL_RING_DEFAULT_BYTES).0;
        let discard_slots: u64 = 64;
        let discard_start = (unit_area_start_slot + discard_slots) * SLOT_BYTES;
        let data_unit_bytes: Vec<u8> = (0..32768_u32)
            .map(|index| u8::try_from(index % 251 + 1).expect("小于 256"))
            .collect();
        let writes = vec![
            RetainedWrite {
                device: DeviceIdentity(0),
                kind: StepKind::UnitWrite,
                is_force_unit_access: false,
                offset: DeviceOffsetInBytes(discard_start),
                contents: WrittenContents::Bytes(data_unit_bytes.clone()),
            },
            RetainedWrite {
                device: DeviceIdentity(0),
                kind: StepKind::Discard,
                is_force_unit_access: false,
                offset: DeviceOffsetInBytes(discard_start),
                contents: WrittenContents::Zeros {
                    length: discard_slots * SLOT_BYTES,
                },
            },
        ];
        let segments = vec![vec![0], vec![1]];
        let tables = build_judge_tables(&JudgeFlow {
            base: &base,
            writes: &writes,
            segments: &segments,
            judged_root_index: 0,
            versions: &[],
            expansion: &crate::crash::full_expansion,
        })
        .expect("带 discard 的流建得出判表");
        assert_eq!(tables.writes[1].kind, StepKind::Discard);
        assert_eq!(
            tables.writes[1].location, None,
            "discard 不落在任何一个落点上"
        );
        assert!(
            tables.is_tearable[1],
            "长于一个扇区、罩住非零旧内容的 discard 取第三态"
        );
        let torn_image_index = u32::try_from(writes.len()).expect("装得进 u32");
        assert_eq!(
            tables.writes[2].kind,
            StepKind::Discard,
            "枚举写表里接在录制流后面的那一条是 discard 的撕裂镜像"
        );
        let unit_slot_locations_the_discard_covers: Vec<u64> = tables
            .locations
            .iter()
            .filter(|location| {
                location.kind == JudgeLocationKind::UnitSlot
                    && location.device == 0
                    && location.offset >= discard_start
                    && location.writes.contains(&1)
            })
            .map(|location| location.offset)
            .collect();
        let slots_the_discard_starts_in: Vec<u64> = (0..discard_slots)
            .map(|slot| discard_start + slot * SLOT_BYTES)
            .collect();
        assert_eq!(
            unit_slot_locations_the_discard_covers, slots_the_discard_starts_in,
            "discard 罩住的每个单元槽各是一个落点、各把它记进自己的写"
        );
        let data_unit_location = tables
            .locations
            .iter()
            .find(|location| location.device == 0 && location.offset == discard_start)
            .expect("数据单元那一槽是一个落点");
        assert_eq!(
            data_unit_location.writes,
            vec![0, 1, torn_image_index],
            "数据单元那一槽的写：数据单元、discard、discard 的撕裂镜像"
        );
        let version_with = |persisted_bits: u64| {
            let mut combination =
                crate::crash_judge_tables::CombinationOfTheWritesOnALocation::NOTHING_PERSISTED;
            for position in 0..data_unit_location.writes.len() {
                if (persisted_bits >> position) & 1 == 1 {
                    combination.mark_persisted(position);
                }
            }
            data_unit_location
                .versions
                .iter()
                .find(|version| version.combination == combination)
                .unwrap_or_else(|| panic!("数据单元那一槽有组合 {persisted_bits:#b} 的版本"))
        };
        assert!(
            version_with(0b011).bytes.iter().all(|byte| *byte == 0),
            "discard 持久那一态读回全 0"
        );
        assert_eq!(
            version_with(0b001).bytes,
            data_unit_bytes,
            "discard 没持久那一态是数据单元的字节"
        );
        let mut old_bytes_under_the_discard = data_unit_bytes.clone();
        old_bytes_under_the_discard.resize(
            usize::try_from(discard_slots * SLOT_BYTES).expect("装得进 usize"),
            0,
        );
        let torn_image = crate::crash::torn_image_of_in_place_overwrite(
            &old_bytes_under_the_discard,
            &WrittenContents::Zeros {
                length: discard_slots * SLOT_BYTES,
            },
        );
        assert_eq!(
            version_with(0b101).bytes,
            torn_image[..data_unit_bytes.len()],
            "撕裂那一态是撕裂镜像在这一槽上的那一截"
        );
    }

    /// 两块盘的槽 0 都抹成 0、4096 处本池的槽 1 照旧，8192 处放一份 journal 环起点写成 1025 的槽（自证得过、读者不收；
    /// harness 场景「8192 处那份槽带不收之值」的造法）：全池没有一个可择的槽 0，GPU 两个内核逐档读槽 1，读得到 8192 处那一份才判得出 I-7.13。
    /// 判表在偏移 0 与几何槽距 4096 之外，给每块盘的 8192 建一个系统配置落点；4096 与 8192 那两槽之间各档（4608 … 7680、8704 … 11776）
    /// 碰过而哪一版都不自证，建完版本就去掉。只建偏移 0 与几何槽距两处的取法少了 8192，这条红；碰过的都留着的取法多出那几档，这条也红。
    #[test]
    fn a_slot_one_carrying_a_refused_value_found_while_reading_every_spacing_gets_a_system_configuration_location(
    ) {
        let mut image = formatted_pool_of_two_devices();
        let spacing = u64::from(SystemImmutableSizes::slot_spacing_for(512));
        let refused_offset = 2 * spacing;
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let mut refused = SystemConfiguration::parse_slot(
                &PoolReader::read(&image, device, DeviceOffsetInBytes(spacing), slot_bytes)
                    .expect("槽 1 读得到"),
            )
            .expect("mkfs 写下的槽 1 自证得过");
            refused.immutable.journal_ring_start_slot =
                SlotNumber(JOURNAL_RING_START_SLOT_THE_READER_REFUSES);
            let pool_device = image.devices.get_mut(&device).expect("池里有这块盘");
            pool_device.write(DeviceOffsetInBytes(refused_offset), &refused.to_slot());
            pool_device.write(DeviceOffsetInBytes(0), &vec![0u8; slot_bytes]);
        }
        let tables = judge_tables_of_one_image(&image).expect("4096 处的槽 1 自证得过：判表建得出");
        assert_eq!(
            system_configuration_locations_of(&tables),
            vec![
                (0, 0),
                (0, spacing),
                (0, refused_offset),
                (1, 0),
                (1, spacing),
                (1, refused_offset)
            ],
            "两块盘各有偏移 0、槽距 {spacing} 与逐档读到的 {refused_offset} 三个系统配置槽的落点，别的几档不留"
        );
    }

    /// io_min 8192（槽距 8192）的池，盘 1 槽 1 那 4096 字节整段清掉（稀疏盘上删掉那几个扇区：基线没写过、读出全 0），别处照 mkfs：
    /// 判表的几何取恢复择的那一份（槽距 8192），每块盘偏移 0 与几何槽距两处系统配置落点照建（[`super::location_is_read_by_the_judge`]：
    /// 这两处照留，判器按它们读槽 0 与槽 1），盘 1 的 8192 那一处没写过、不由逐档那一圈补出，只由几何槽距那一处建。
    /// 判别力（`crates/mutations.tsv`「E 崩溃点重放 J14 判表的系统配置槽 1 照旧摆在 4096」那一行）：槽 1 照旧摆在 4096，盘 1 的 8192 没有落点、
    /// 4096 那一处一版都不自证被去掉，这条红。盘上两槽都写过的池上逐档那一圈补出 8192，那一处改法看不出来。
    #[test]
    fn an_io_min_8192_pool_whose_slot_one_on_one_device_was_never_written_keeps_a_system_configuration_location_at_the_slot_spacing(
    ) {
        let mut image = formatted_pool_of_two_devices_with_io_min(8192);
        let spacing = u64::from(SystemImmutableSizes::slot_spacing_for(8192));
        assert_eq!(spacing, 8192, "io_min 8192 的池槽距 8192");
        let device_one = image
            .devices
            .get_mut(&DeviceIdentity(1))
            .expect("池里有盘 1");
        device_one.zero_fill(
            DeviceOffsetInBytes(spacing),
            SYSTEM_CONFIGURATION_SLOT_BYTES,
        );
        assert!(
            device_one
                .written_sectors_in(
                    DeviceOffsetInBytes(spacing),
                    SYSTEM_CONFIGURATION_SLOT_BYTES
                )
                .is_empty(),
            "盘 1 槽 1 那 4096 字节没有写过的扇区"
        );
        let tables = judge_tables_of_one_image(&image)
            .expect("两块盘的槽 0 与盘 0 的槽 1 自证得过：判表建得出");
        assert_eq!(
            system_configuration_locations_of(&tables),
            vec![(0, 0), (0, spacing), (1, 0), (1, spacing)],
            "两块盘各有偏移 0 与几何槽距 {spacing} 两个系统配置槽的落点，盘 1 那一槽没写过也建"
        );
    }

    /// 用过的盘上重做 mkfs 那条流的 journal 环长：16 MiB（每块盘 4096 个记录槽），第一版默认的 768 MiB 照样建得出表，只是撕裂镜像大。
    const SMALL_JOURNAL_RING_BYTES: u64 = 16 << 20;

    /// journal 环 16 MiB、io_min 512（槽距 4096、物理块 512；根环区域 0 / 1 / 2 在盘 0 / 1 / 0 上）的 mkfs 参数。
    fn make_filesystem_parameters_with_a_small_journal_ring(
        filesystem_identifier: [u8; 16],
    ) -> MakeFilesystemParameters {
        MakeFilesystemParameters {
            filesystem_identifier,
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            geometry: SystemImmutableSizes {
                physical_block_size: 512,
                minimum_input_output_bytes: 512,
                fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
                journal_ring_bytes: SMALL_JOURNAL_RING_BYTES,
                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
            },
        }
    }

    /// 旧池：两块空盘按 [`make_filesystem_parameters_with_a_small_journal_ring`] mkfs 一次（fsid `singlefs-tables!`），再在两块盘
    /// journal 环的头一个记录槽里各写一扇区非零字节（旧池写过的 journal 记录：只要非零，不要求自证）。
    fn used_pool_with_a_small_journal_ring() -> MemoryPool {
        let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
            [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(|identity| {
                    (
                        identity,
                        SparseBlockDevice::new(
                            FORMATTED_DEVICE_BYTES,
                            PhysicalBlockSizeInBytes(512),
                        ),
                    )
                })
                .collect();
        make_filesystem(
            &make_filesystem_parameters_with_a_small_journal_ring(*b"singlefs-tables!"),
            &mut devices,
        )
        .expect("两块空盘上 mkfs 做得成");
        let mut base = MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.image.clone()))
                .collect(),
            device_size_in_bytes: FORMATTED_DEVICE_BYTES,
        };
        let old_journal_record_sector: Vec<u8> = (0..512_u32)
            .map(|index| u8::try_from(index % 251 + 1).expect("小于 256"))
            .collect();
        for device in base.devices.values_mut() {
            device.write(
                DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES),
                &old_journal_record_sector,
            );
        }
        base
    }

    /// 在 `base` 上经内层入口拿另一个 fsid（`remade-tables!!!`，前 8 字节与旧池不同）重做 mkfs，交回录下的写与段。
    fn make_filesystem_over_the_used_pool(
        base: &MemoryPool,
    ) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = base
            .devices
            .iter()
            .map(|(identity, sparse)| {
                let mut device =
                    SparseBlockDevice::new(FORMATTED_DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
                device.image = sparse.clone();
                (
                    *identity,
                    RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
                )
            })
            .collect();
        let parameters = make_filesystem_parameters_with_a_small_journal_ring(*b"remade-tables!!!");
        make_filesystem(&parameters, &mut devices).expect("用过的盘上重做 mkfs 做得成");
        writes_and_segments(
            &stream.retained_operations(),
            &FixedGeometry {
                fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
                journal_ring_bytes: parameters.geometry.journal_ring_bytes,
                root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
            },
        )
    }

    /// 用过的盘上重做 mkfs（两趟写序：两块盘的 [0, 根环基址) 清零自成一段，其余八次清零与四个单元写一段，三条第 0 代根各一段，四个系统配置槽一段）
    /// 按甲二展开建判表：清零写不落在哪一个落点上；它罩住的每个落点各把它记进自己的写；它不给自己建落点（journal 环里只有旧池写过的那一槽是落点）；
    /// 任一个状态里读得出非零字节的扇区（基线写过的、或枚举写表里哪一次写写下了非零字节的）都在某个落点里；每个状态、每个落点按持久掩码查到的版本
    /// 与崩溃镜像在那里读出的字节逐字节相同。
    /// 判别力（`crates/mutations.tsv`「C656 GPU 判表收清零写」那两行）：建落点那一步漏记清零写罩住的落点，作废之后系统配置槽那一版照旧是旧池的字节，
    /// 逐字节比那一句红；清零写照旧算碰过，journal 环里每一槽都建落点，落点那一句红。照旧在建表时拒清零写，建表那一步就红。
    #[test]
    fn zero_fills_of_a_make_filesystem_over_a_used_pool_land_in_every_location_they_cover_and_add_no_location_of_their_own(
    ) {
        let base = used_pool_with_a_small_journal_ring();
        let (writes, segments) = make_filesystem_over_the_used_pool(&base);
        assert_eq!(
            segments.iter().map(Vec::len).collect::<Vec<usize>>(),
            vec![2, 12, 1, 1, 1, 4],
            "两趟写序的段序列"
        );
        let zero_fills: Vec<usize> = writes
            .iter()
            .enumerate()
            .filter(|(_, write)| write.kind == StepKind::ZeroFill)
            .map(|(write_index, _)| write_index)
            .collect();
        assert_eq!(zero_fills.len(), 10, "每块盘五次清零，两块盘十次");
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("mkfs 写第 0 代根");
        let tables = build_judge_tables(&JudgeFlow {
            base: &base,
            writes: &writes,
            segments: &segments,
            judged_root_index,
            versions: &[],
            expansion: &crate::crash::quick_tier_expansion,
        })
        .expect("用过的盘上重做 mkfs 那条流建得出判表");
        let enumerated = crate::crash::crash_state_enumeration_tables(
            &base,
            &writes,
            &segments,
            &crate::crash::quick_tier_expansion,
        )
        .enumerated_writes;
        let zero_fill_entries: Vec<usize> = enumerated
            .iter()
            .enumerate()
            .filter(|(_, write)| write.kind == StepKind::ZeroFill)
            .map(|(entry, _)| entry)
            .collect();
        assert!(
            zero_fill_entries.len() > zero_fills.len(),
            "罩住旧内容的清零写取第三态，枚举写表里接着它们的撕裂镜像"
        );
        for entry in &zero_fill_entries {
            assert_eq!(
                tables.writes[*entry].location, None,
                "清零写（或它的撕裂镜像）{entry} 不落在任何一个落点上"
            );
        }
        for location in &tables.locations {
            let location_end = location.offset + u64::from(location.length);
            for entry in &zero_fill_entries {
                let write = &enumerated[*entry];
                let covers = write.device.0 == location.device
                    && write.offset.0 < location_end
                    && location.offset < write.offset.0 + write.length_in_bytes();
                assert_eq!(
                    location
                        .writes
                        .contains(&u32::try_from(*entry).expect("装得进 u32")),
                    covers,
                    "落点 (盘 {}, 偏移 {}, {:?}) 的写里有没有清零写 {entry}",
                    location.device,
                    location.offset,
                    location.kind
                );
            }
        }
        let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
        let journal_record_locations: Vec<(u32, u64)> = tables
            .locations
            .iter()
            .filter(|location| location.kind == JudgeLocationKind::JournalRecord)
            .map(|location| (location.device, location.offset))
            .collect();
        assert_eq!(
            journal_record_locations,
            vec![(0, ring_start), (1, ring_start)],
            "清零写不给自己建落点：journal 环 {} 个记录槽里只有旧池写过的那一槽是落点",
            SMALL_JOURNAL_RING_BYTES / JOURNAL_RECORD_BYTES
        );
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
        let is_inside_some_location = |device: u32, sector_start: u64| {
            tables.locations.iter().any(|location| {
                location.device == device
                    && location.offset <= sector_start
                    && sector_start + SECTOR_BYTES <= location.offset + u64::from(location.length)
            })
        };
        for (device, sparse) in &base.devices {
            for (sector, bytes) in sparse.written_sectors() {
                if bytes.iter().any(|byte| *byte != 0) {
                    assert!(
                        is_inside_some_location(device.0, sector * SECTOR_BYTES),
                        "基线在盘 {} 第 {sector} 扇区有非零字节，它在某个落点里",
                        device.0
                    );
                }
            }
        }
        for (entry, write) in enumerated.iter().enumerate() {
            let WrittenContents::Bytes(bytes) = &write.contents else {
                continue;
            };
            for (sector_in_the_write, sector) in bytes.chunks(sector_bytes).enumerate() {
                if sector.iter().any(|byte| *byte != 0) {
                    let sector_start = write.offset.0
                        + u64::try_from(sector_in_the_write).expect("扇区号") * SECTOR_BYTES;
                    assert!(
                        is_inside_some_location(write.device.0, sector_start),
                        "枚举写 {entry}（{:?}）在盘 {} 偏移 {sector_start} 写下非零字节，那一扇区在某个落点里",
                        write.kind,
                        write.device.0
                    );
                }
            }
        }
        let mut versions_checked = 0u64;
        for ordinal in 0..tables.state_count {
            let persisted = tables.persisted_writes_of_state(ordinal);
            let image = CrashImage {
                base: &base,
                writes: &enumerated,
                persisted: persisted.clone(),
            };
            for location in &tables.locations {
                let version = tables.version_of(location, &persisted).unwrap_or_else(|| {
                    panic!(
                        "状态 {ordinal} 在落点 (盘 {}, 偏移 {}) 上的组合没有版本",
                        location.device, location.offset
                    )
                });
                let on_disk = PoolReader::read(
                    &image,
                    DeviceIdentity(location.device),
                    DeviceOffsetInBytes(location.offset),
                    usize::try_from(location.length).expect("长度"),
                )
                .expect("落点在盘内");
                assert!(
                    version.bytes == on_disk,
                    "状态 {ordinal} 落点 (盘 {}, 偏移 {}, {:?}) 的版本字节与崩溃镜像不同",
                    location.device,
                    location.offset,
                    location.kind
                );
                versions_checked += 1;
            }
        }
        assert!(versions_checked > 0, "真比过版本");
        println!(
            "CRASH_JUDGE_TABLES_ZERO_FILLS states={} locations={} versions_checked={versions_checked} most_writes_on_one_location={}",
            tables.state_count,
            tables.locations.len(),
            tables
                .locations
                .iter()
                .map(|location| location.writes.len())
                .max()
                .unwrap_or(0)
        );
    }
}
