//! GPU 判器的输入表（崩溃放量第 ① 段的另一半「GPU 判器现在实现、把整套判法搬上去」）。
//!
//! 一个崩溃状态的镜像 = 基线 + 这个状态里持久的写按枚举写表次序叠上去（`singlefs_verification::memory_pool::CrashImage::read`）。
//! 判器读盘只读**结构槽**：系统配置槽、根环槽、journal 记录槽、单元区里的 16 KiB 槽（单元读 16 KiB 或 32 KiB）。
//! 这里把每个结构槽叫一个**落点**（[`JudgeLocation`]），把它在全部崩溃状态里可能呈现的每一种字节叫一个**版本**（[`JudgeLocationVersion`]）：
//! 版本由「盖住这个落点的那几次写各自持久没持久」决定，几次写里只有崩溃那一段里的几次是自由的，别的段整段持久或整段没持久，
//! 所以可达的组合很少（每个落点几个到几十个），按段逐个枚举、去重，字节按 `CrashImage::read` 同一条规则叠出来。
//! CPU 在这里把每个版本的字节、校验和（CRC-32C；整槽宽校验和字段也是 CRC-32C 加 28 个 0）、与每次写逐扇区等不等算一次；
//! GPU 每个状态只做组合逻辑：由持久掩码算出每个落点的组合、查到版本、按字节解结构、走恢复与 checker 的判法（`crate::crash_judge_gpu`）。
//!
//! 表的字节序列化进 KV 时按录入键存（建表与打包的函数在录入那一组判法单元的入口里，表的形态变了录入那一把键就变），
//! [`JudgeTables::digest`] 只按内容取（与事实表同一条规矩）。
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use crate::memory_pool::{
    publishes_in, root_identity_of_write, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RecordStreamContinuity, RecordStreamSeam, RetainedWrite,
    WrittenContents, SECTOR_BYTES,
};
use crate::segments::StepKind;
use crate::sha256::{sha256_digest, DIGEST_BYTES};
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

mod checks_of_a_version;
mod locations_and_their_versions;
mod word_layout_of_the_tables;
mod writes_and_publishes_of_the_tables;

pub use checks_of_a_version::*;
use locations_and_their_versions::*;
pub use word_layout_of_the_tables::*;
use writes_and_publishes_of_the_tables::*;

/// 一个落点最多被几次写盖住：GPU 内核里组合掩码是 [`COMBINATION_WORDS_OF_A_VERSION`] 个 u32（[`CombinationOfTheWritesOnALocation`]，
/// 版本表里的布局见 [`version_word_of_the_combination_word`]）。随机历史越过原分配记录墙那一族把系统配置 0 号槽录制写了 88 到 111 次
/// （调查员量的，全量那一趟建不出判表），64 位装不下；256 留出余量。清零写进落点模型之后用过的盘上重做 mkfs 那条流
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
        // discard 与清零写都不落在任何一个落点上（[`JudgeWrite::location`] 是 `None`）：它罩住的每个落点
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
pub struct JudgeLocationVersion {
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
    pub versions: Vec<JudgeLocationVersion>,
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

/// 一次发布在写表里的样子（`singlefs_verification::memory_pool::publishes_in`），单元写按偏移分组（两盘各一份副本）。
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
    /// 枚举写表接在后面的撕裂根槽写分不出另一次发布：mkfs 的根槽写叠在 mkfs 之后的基线上（创世发布进记录核对器），
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
/// 历史与挂载之间「第三截也上 GPU，GPU 的发布表认接缝」）：发布表照记录核对器同一个分法
/// （`singlefs_verification::memory_pool::publishes_in` 的 `RecordStreamContinuity::BrokenAtSeams`）分，内核照旧只读发布表，所以接缝怎么断
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
    ) -> Option<&'location JudgeLocationVersion> {
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
                    recorded.push(JUDGE_TABLE_ENTRY_ABSENT);
                    recorded.push(0);
                    recorded.push(0);
                }
            }
        }
        // 段表每条 [`SEGMENT_WORDS`] 个字：数位那一组写的起点与条数、状态起点 lo/hi、状态终点 lo/hi、展开方式码（[`SegmentExpansionFact::code`]）、
        // 整组落的单元写那一组的起点与条数（只有甲二段有）；两组合起来是段里全部的写，之前的段整段持久时两组都置。
        // 第 9、10 字是分片的段（C790）片起点拆成的各位在段写表里的起点与位数（一位一个字：单元写那一组那一位在前，再是数位那一组写各一位）；
        // 不分片的段是 JUDGE_TABLE_ENTRY_ABSENT 与 0
        let mut segments = Vec::new();
        let mut segment_writes = Vec::new();
        for (segment, expansion) in self.segments.iter().zip(&self.segment_expansions) {
            let (digit_writes, copy_on_write_group) =
                expansion.digit_writes_and_the_copy_on_write_group(segment);
            segments.push(index(segment_writes.len()));
            segments.push(index(digit_writes.len()));
            let (start_low, start_high) = split(segment.states.start);
            let (end_low, end_high) = split(segment.states.end);
            segments.extend([start_low, start_high, end_low, end_high]);
            segment_writes.extend(digit_writes.iter().copied());
            segments.push(expansion.code());
            segments.push(index(segment_writes.len()));
            segments.push(index(copy_on_write_group.len()));
            segment_writes.extend(copy_on_write_group.iter().copied());
            match &segment.slice_start {
                None => segments.extend([JUDGE_TABLE_ENTRY_ABSENT, 0]),
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
        // 一族）、实例代号、checkpoint_txg lo/hi（没写根的两种码后三个字是 JUDGE_TABLE_ENTRY_ABSENT），按序号接；按段展开时这张表是空的
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
            let (offset_low, offset_high) = split(write.offset);
            let (instance, txg_low, txg_high) = match write.root_identity {
                Some((instance, txg)) => {
                    let (low, high) = split(txg);
                    (instance, low, high)
                }
                None => (
                    JUDGE_TABLE_ENTRY_ABSENT,
                    JUDGE_TABLE_ENTRY_ABSENT,
                    JUDGE_TABLE_ENTRY_ABSENT,
                ),
            };
            let root_bytes_offset = if write.root_bytes.is_empty() {
                JUDGE_TABLE_ENTRY_ABSENT
            } else {
                let offset = index(root_bytes.len());
                root_bytes.extend(bytes_to_words(&write.root_bytes));
                offset
            };
            writes.extend([
                write.device,
                offset_low,
                offset_high,
                write.length,
                step_kind_code(write.kind),
                write.location.unwrap_or(JUDGE_TABLE_ENTRY_ABSENT),
                instance,
                txg_low,
                txg_high,
                root_bytes_offset,
                index(laters.len() / LATER_WORDS),
                index(write.later_overlapping_writes.len()),
            ]);
            for later in &write.later_overlapping_writes {
                let (covered_low, covered_high) = split(later.covered_sectors);
                laters.extend([
                    later.write_index,
                    u32::from(later.reuse_is_not_proven_illegal),
                    covered_low,
                    covered_high,
                ]);
            }
        }
        // 落点与版本。版本表最前面两条不是哪个落点的版本（[`ZERO_UNIT_VERSIONS`]）：第 0 条是一段 32 KiB 全零单元、第 1 条是同一段的前 16 KiB
        // （盘尾只装得下 16 KiB 的槽），组合都是 0、字节起点都是字节表的第 0 字、检查字按单元槽算（[`version_checks`]）、第 6 字 JUDGE_TABLE_ENTRY_ABSENT。
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
                JUDGE_TABLE_ENTRY_ABSENT,
                crc_shifted_past_sixteen_zero_kibibytes(
                    JudgeLocationKind::UnitSlot,
                    &zero_unit[..usize::try_from(zero_unit_bytes).expect("单元长装得进 usize")],
                ),
            ));
        }
        version_bytes.extend(bytes_to_words(&zero_unit));
        let gaps = self.unit_slots_nothing_wrote_after_each_location();
        for (location_index, location) in self.locations.iter().enumerate() {
            let (offset_low, offset_high) = split(location.offset);
            locations.extend([
                location.device,
                offset_low,
                offset_high,
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
                    let (low, high) = split(*mask);
                    equal_masks.extend([low, high]);
                }
            }
        }
        // 发布表
        let mut publishes = Vec::new();
        let mut publish_records = Vec::new();
        let mut copy_groups = Vec::new();
        let mut copies = Vec::new();
        for publish in &self.publishes {
            let (txg_low, txg_high) = split(publish.checkpoint_txg);
            publishes.extend([
                publish.root_write,
                publish.instance,
                txg_low,
                txg_high,
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
            let (txg_low, txg_high) = split(version.checkpoint_txg);
            expected.extend([
                version.instance,
                txg_low,
                txg_high,
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
        let (device_bytes_low, device_bytes_high) = split(self.device_bytes);
        let (state_count_low, state_count_high) = split(self.state_count);
        let mut header = vec![
            self.recorded_write_count,
            self.enumerated_write_count,
            index(self.segments.len()),
            index(self.devices.len()),
            device_bytes_low,
            device_bytes_high,
            index(self.locations.len()),
            index(self.publishes.len()),
            index(self.expected_versions.len()),
            self.judged_root_index,
            state_count_low,
            state_count_high,
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

#[cfg(test)]
mod tests {
    use super::{
        build_judge_tables,
        build_judge_tables_for_the_given_persisted_sets_of_the_recorded_writes_broken_at_the_seams_with_their_claimed_versions,
        version_word_of_the_combination_word, ClaimedVersionOfAGivenState, JudgeFlow,
        JudgeStateSource, JudgeTables, JudgeTablesRefusal, JUDGE_TABLE_ENTRY_ABSENT,
        LOCATION_WORDS, SEGMENT_WORDS, VERSION_WORDS,
    };
    use crate::crash::CrashPointReplaySegmentExpansion;
    use crate::crash_facts::{
        SegmentExpansionFact, SegmentFact, MAXIMUM_ENUMERATED_WRITES,
        SEGMENT_EXPANSION_CODE_IN_PLACE_SUBSETS_WITH_COPY_ON_WRITE_NONE_OR_ALL,
    };
    use crate::memory_pool::{MemoryPool, RetainedWrite, SparseBlockDevice, WrittenContents};
    use crate::segments::StepKind;
    use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;
    use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
    use singlefs_core::root_ring::RootRingSlotsPerRegion;
    use singlefs_core::system_configuration::SystemImmutableSizes;
    use singlefs_format::{
        JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES,
        SYSTEM_CONFIGURATION_SLOT_BYTES,
    };

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
    fn segment_in_place_subsets_with_copy_on_write_none_or_all_lays_out_its_two_groups_of_writes() {
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
                JUDGE_TABLE_ENTRY_ABSENT,
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
    fn sliced_segment_lays_out_the_digits_of_its_slice_start_after_its_two_groups_of_writes() {
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

    /// 枚举写表多于内核持久掩码的宽（[`MAXIMUM_ENUMERATED_WRITES`] 次）：建表交「掩码表满了」，不 panic、不往下建。
    #[test]
    fn flow_with_more_enumerated_writes_than_the_mask_holds_is_refused_naming_the_mask() {
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

    /// 随机历史越过原分配记录墙那一族最大的一张给定状态表（调查员量的）：枚举写表 5709 次写（接撕裂镜像时；录制写最多 5265）。
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
    fn given_states_table_of_five_thousand_seven_hundred_and_nine_writes_with_one_hundred_and_eleven_on_one_location_is_built(
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

    /// 两块盘 mkfs 一次的池的盘宽：与 harness 建池用的盘同宽（稀疏盘，只存写过的扇区）。
    pub(super) const FORMATTED_DEVICE_BYTES: u64 = 4 << 30;

    /// 两块盘 mkfs 一次的池（io_min 512、槽距 4096；根环区域 0 / 1 / 2 在盘 0 / 1 / 0 上），没有文件。
    pub(super) fn formatted_pool_of_two_devices() -> MemoryPool {
        formatted_pool_of_two_devices_with_minimum_input_output_bytes(512)
    }

    /// 两块盘 mkfs 一次的池，mkfs 探测到的 io_min 是 `minimum_input_output_bytes`（槽距 = 4096 向上取整到它的整数倍；
    /// 物理块 512；根环区域 0 / 1 / 2 在盘 0 / 1 / 0 上），没有文件。
    pub(super) fn formatted_pool_of_two_devices_with_minimum_input_output_bytes(
        minimum_input_output_bytes: u32,
    ) -> MemoryPool {
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
}
