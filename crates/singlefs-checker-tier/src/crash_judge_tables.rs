//! GPU 判器的输入表（崩溃放量第 ① 段的另一半，用户 2026-09-28 定「GPU 判器现在实现、把整套判法搬上去」）。
//!
//! 一个崩溃状态的镜像 = 基线 + 这个状态里持久的写按枚举写表次序叠上去（`singlefs_harness::memory_pool::CrashImage::read`）。
//! 判器读盘只读**结构槽**：系统配置槽、根环槽、journal 记录槽、单元区里的 16 KiB 槽（单元读 16 KiB 或 32 KiB）。
//! 这里把每个结构槽叫一个**落点**（[`JudgeLocation`]），把它在全部崩溃状态里可能呈现的每一种字节叫一个**版本**（[`JudgeVersion`]）：
//! 版本由「盖住这个落点的那几次写各自持久没持久」决定，几次写里只有崩溃那一段里的几次是自由的，别的段整段持久或整段没持久，
//! 所以可达的组合很少（每个落点几个到几十个），按段逐个枚举、去重，字节按 `CrashImage::read` 同一条规则叠出来。
//! CPU 在这里把每个版本的字节、校验和（CRC-32C；整槽宽校验和字段也是 CRC-32C 加 28 个 0）、与每次写逐扇区等不等算一次；
//! GPU 每个状态只做组合逻辑：由持久掩码算出每个落点的组合、查到版本、按字节解结构、走恢复与 checker 的判法（`crate::crash_judge_gpu`）。
//!
//! 表的字节序列化进 KV 时键带 [`JUDGE_TABLES_VERSION`] 与表的摘要：换了表的形态就是新一轮（与事实表同一条规矩）。
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_field_holds};
use singlefs_core::journal::slot_after_the_journal_ring;
use singlefs_core::recovery::PoolReader;
use singlefs_core::root_ring::region_start;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_format::{
    DATA_UNIT_BYTES, DATA_UNIT_HEADER_BYTES, JOURNAL_HEADER_BYTES, JOURNAL_RECORD_BYTES,
    JOURNAL_RING_START_SLOT, NODE_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES, ROOT_RECORD_BYTES,
    ROOT_RING_REGIONS, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_harness::memory_pool::{
    publishes_in, root_identity_of_write, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RecordStreamContinuity, RecordStreamSeam, RetainedWrite,
    WrittenContents, SECTOR_BYTES,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

use crate::crash::{
    layer0_enumeration_tables, reuse_is_not_proven_illegal_by_the_reclaim_predicate,
    Layer0EnumerationTables, Layer0SegmentExpansion, Layer0VersionTable,
};
use crate::crash_facts::{
    persisted_writes_of_state_by_the_segment_expansions, segment_expansion_facts,
    SegmentExpansionFact, SegmentFact, TornImageFact, MAXIMUM_ENUMERATED_WRITES,
};

/// 表的形态版本：落点、版本、检查字或发布表的布局变了就抬它（4：版本表最前面两条全零单元、落点第 9–11 字记它之后没写过的槽，
/// [`ZERO_UNIT_VERSIONS`]；5：期望版本表第 5 字记这一版有没有文件，[`EXPECTED_VERSION_WITHOUT_A_FILE`]；6：版本表每条多一个第 8 字，
/// [`VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES`]；7：段表每条 [`SEGMENT_WORDS`] 个字、带段的展开方式与甲二那一组单元写，
/// 多一张点名的持久掩码表（[`JudgeStateSource::GivenPersistedSets`]））。
pub const JUDGE_TABLES_VERSION: u32 = 7;

/// 一个落点最多被几次写盖住：GPU 内核里组合掩码是两个 u32（版本表里一个版本的第 0 字是低 32 位、第 7 字是高 32 位）。
/// 错位复用那条历史把系统配置 0 号槽写了 34 次，32 位装不下。
pub const MAXIMUM_WRITES_PER_LOCATION: usize = 64;
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
    }
}

/// 一个落点在某个持久组合下呈现的字节与 CPU 先算好的几样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgeVersion {
    /// 第 i 位 = 这个落点的 `writes[i]` 在这个组合里持久。
    pub combination: u64,
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
    pub versions: Vec<JudgeVersion>,
}

/// 枚举写表里的一次写，判器要的几样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgeWrite {
    pub device: u32,
    pub offset: u64,
    pub length: u32,
    pub kind: StepKind,
    /// 这次写落在哪个落点（落点表下标）；清零写不落在任何落点。
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

/// 期望版本表里的一版：有文件（读回的要是 `chunks` 那几个单元），还是文件从这一版起没有了（`crash::Layer0VersionTable` 的两样，
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
    /// 单元区起点（字节）：基线第一块盘系统配置槽 0 记的几何。单元区里没有落点的 16 KiB 槽每个崩溃状态都是全零（[`ZERO_UNIT_VERSIONS`]）。
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
    /// 第 k 个状态就是第 k 个持久掩码（按枚举写表给，每个与枚举写表一样长），状态数是清单的长度。
    GivenPersistedSets(Vec<Vec<bool>>),
}

/// 建表要的一条流（与 `crash_amplification::CrashFlow` 同一组东西，不带崩溃点）。
pub struct JudgeFlow<'flow> {
    pub base: &'flow MemoryPool,
    pub writes: &'flow [RetainedWrite],
    pub segments: &'flow [Vec<usize>],
    pub judged_root_index: usize,
    pub versions: &'flow [PublishedVersion],
    pub expansion: &'flow dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
}

/// 建表时撞上的、GPU 判不了的形态。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JudgeTablesRefusal {
    /// 基线里第一块盘的系统配置槽 0 解不开：落点几何无从取。
    BaseSystemConfigurationUnreadable,
    /// 枚举写表里有清零写：清零的范围可以横跨几千个槽，落点模型不接它。
    ZeroFillInTheEnumeratedWrites { write_index: usize },
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
    /// 一段按它的展开方式展开出的状态数装不进 u64（段内序号拆不开），或与它的序号区间对不上：段表核段长没过。
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

/// 一次写每个扇区的落地：与层 0 一致（0 没持久、1 撕裂、2 持久）。
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
/// 枚举写表里的写越过盘尾；同一个根既在 `flow.versions` 里又在 `versions_without_a_file` 里（[`Layer0VersionTable::new`]）。
pub fn build_judge_tables_against_the_version_table(
    flow: &JudgeFlow<'_>,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
) -> Result<JudgeTables, JudgeTablesRefusal> {
    build_judge_tables_from_the_state_source(
        flow,
        versions_without_a_file,
        JudgeStateSource::SegmentExpansions,
        &[],
        PublishesSplitFrom::TheEnumeratedWrites,
    )
}

/// GPU 的发布表从哪一张写表分（[`judge_publishes`]）：与同一批状态上 CPU 记录核对器拿到的那一张相同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PublishesSplitFrom {
    /// 枚举写表（录制流接撕裂镜像与重放）：层 0 的 CPU 记录核对器（`crash::check_records`）拿的就是它。
    TheEnumeratedWrites,
    /// 只取录制流：随机历史的 CPU 记录核对器（`crash_injection::judge_a_given_state_on_the_cpu`）拿的写表不带撕裂镜像与重放。
    /// 枚举写表接在后面的撕裂根槽写分不出另一次发布：mkfs 的根槽写叠在 mkfs 之后的基线上（创世发布进记录核对器，用户 2026-10-10 定），
    /// 撕出来与原字节相同，接成一次发布就把录制流末尾没等到根槽写的单元写与记录写归给它。
    TheRecordedWritesOnly,
}

/// 给定状态表（[`JudgeStateSource::GivenPersistedSets`]）：GPU 判 `given_persisted_sets` 里的每一个持久掩码（按枚举写表给，第 k 个是序号 k 的状态），
/// 不按段展开拆序号。落点的版本只取这几个掩码叠得出来的那几种；枚举写表照 `flow` 的段建（撕裂镜像与重放接在哪由段定）。
/// 期望版本表同 [`build_judge_tables_against_the_version_table`]。发布表与层 0 一样从枚举写表分（掩码可以点名撕裂态，对的是层 0 的 CPU 判器）。
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
    build_judge_tables_from_the_state_source(
        flow,
        versions_without_a_file,
        JudgeStateSource::GivenPersistedSets(given_persisted_sets),
        &[],
        PublishesSplitFrom::TheEnumeratedWrites,
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
    build_judge_tables_from_the_state_source(
        flow,
        versions_without_a_file,
        JudgeStateSource::GivenPersistedSets(given_persisted_sets),
        seams,
        PublishesSplitFrom::TheRecordedWritesOnly,
    )
}

/// `seams_of_the_given_sets` 只在给定状态表上可以不空（[`build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams`]）；
/// 按段展开那一路恒给空表（层 0 的流是一条录制流）。`publishes_split_from` 说发布表从哪一张写表分（[`PublishesSplitFrom`]）。
fn build_judge_tables_from_the_state_source(
    flow: &JudgeFlow<'_>,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    state_source: JudgeStateSource,
    seams_of_the_given_sets: &[RecordStreamSeam],
    publishes_split_from: PublishesSplitFrom,
) -> Result<JudgeTables, JudgeTablesRefusal> {
    let _version_table_without_contradictions =
        Layer0VersionTable::new(flow.versions, versions_without_a_file);
    let tables: Layer0EnumerationTables =
        layer0_enumeration_tables(flow.base, flow.writes, flow.segments, flow.expansion);
    let enumerated = &tables.enumerated_writes;
    if enumerated.len() > MAXIMUM_ENUMERATED_WRITES {
        return Err(JudgeTablesRefusal::EnumeratedWritesPastTheMaskWidth {
            enumerated_writes: enumerated.len(),
        });
    }
    if let JudgeStateSource::GivenPersistedSets(given_persisted_sets) = &state_source {
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
    for (write_index, write) in enumerated.iter().enumerate() {
        if write.kind == StepKind::ZeroFill {
            return Err(JudgeTablesRefusal::ZeroFillInTheEnumeratedWrites { write_index });
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
    let geometry = base_geometry(flow.base, devices[0])?;
    let mut locations =
        enumerate_locations(flow.base, enumerated, &devices, device_bytes, &geometry);
    locations.sort_by_key(|location| (location.device, location.offset));
    let location_index_by_start: BTreeMap<(u32, u64), usize> = locations
        .iter()
        .enumerate()
        .map(|(index, location)| ((location.device, location.offset), index))
        .collect();
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
            JudgeStateSource::GivenPersistedSets(given_persisted_sets) => {
                combinations_of_the_given_persisted_sets(location, given_persisted_sets)?
            }
        };
        push_the_versions_of_the_combinations(flow.base, enumerated, location, combinations);
    }
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
    let segments: Vec<SegmentFact> = flow
        .segments
        .iter()
        .zip(&tables.state_ranges_by_segment)
        .map(|(segment, states)| SegmentFact {
            writes: segment
                .iter()
                .map(|write| u32::try_from(*write).expect("下标装得进 u32"))
                .collect(),
            states: states.clone(),
        })
        .collect();
    let segment_expansions = segment_expansion_facts(flow.writes, flow.segments, flow.expansion);
    // 核段长：每段按它的展开方式现算的状态数装得进 u64、与它的序号区间相同（内核按这一份拆满 64 位的段内序号）
    for (segment_index, (segment, expansion)) in
        segments.iter().zip(&segment_expansions).enumerate()
    {
        if expansion.state_count(segment, &tables.is_tearable_by_write)
            != Some(segment.states.end - segment.states.start)
        {
            return Err(JudgeTablesRefusal::SegmentStateCountNotRepresentable { segment_index });
        }
    }
    let state_count = match &state_source {
        JudgeStateSource::SegmentExpansions => tables.state_count,
        JudgeStateSource::GivenPersistedSets(given_persisted_sets) => {
            u64::try_from(given_persisted_sets.len()).expect("清单长装得进 u64")
        }
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

/// 基线里第一块盘系统配置槽 0 记的几何：落点从它来。
struct BaseGeometry {
    fixed_structure_slot_spacing: u64,
    physical_block_size: u64,
    root_ring_slots_per_region: u64,
    region_devices: [u32; 3],
    journal_ring_bytes: u64,
    unit_area_start_slot: u64,
}

fn base_geometry(base: &MemoryPool, first_device: u32) -> Result<BaseGeometry, JudgeTablesRefusal> {
    let slot_bytes_of_system_configuration =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let bytes = PoolReader::read(
        base,
        singlefs_core::address::DeviceIdentity(first_device),
        DeviceOffsetInBytes(0),
        slot_bytes_of_system_configuration,
    )
    .ok_or(JudgeTablesRefusal::BaseSystemConfigurationUnreadable)?;
    let system_configuration = SystemConfiguration::parse_slot(&bytes)
        .map_err(|_refusal| JudgeTablesRefusal::BaseSystemConfigurationUnreadable)?;
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

/// 全部落点（还没填写与版本）：系统配置每盘两槽、根环全部槽、journal 环里基线或枚举写碰过的记录槽、单元区里基线或枚举写碰过的 16 KiB 槽。
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
        if offset + length > device_bytes || !starts.insert((device, offset)) {
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
    // 碰过的字节范围：基线写过的扇区，加枚举写表里每次写的范围
    let mut touched_ranges: BTreeMap<u32, Vec<Range<u64>>> = BTreeMap::new();
    for (device, sparse) in &base.devices {
        let ranges = touched_ranges.entry(device.0).or_default();
        for sector in sparse.written_sectors().keys() {
            ranges.push(sector * SECTOR_BYTES..(sector + 1) * SECTOR_BYTES);
        }
    }
    for write in enumerated {
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
    locations
}

/// 枚举写表里录制流之外的那几条（撕裂镜像、重放）各自跟着哪次录制写：撕裂镜像跟着被撕的那次写，重放跟着被撕的那次写与被重放的那次写。
struct ExtendedWriteOrigins {
    /// 枚举写表下标 → （被撕的那次写，被重放的那次写（撕裂镜像自己没有））。
    origin: BTreeMap<usize, (usize, Option<usize>)>,
}

impl ExtendedWriteOrigins {
    fn of(tables: &Layer0EnumerationTables, recorded_write_count: usize) -> Self {
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
) -> Result<BTreeSet<u64>, JudgeTablesRefusal> {
    let recorded_write_count = segment_of_write.len();
    let mut combinations: BTreeSet<u64> = BTreeSet::new();
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
    let combination_of = |cut: usize, free: &BTreeMap<usize, Landing>| -> u64 {
        let mut combination = 0u64;
        for (position, entry) in location.writes.iter().enumerate() {
            if persisted_in(usize::try_from(*entry).expect("下标"), cut, free) {
                combination |= 1u64 << position;
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
) -> Result<BTreeSet<u64>, JudgeTablesRefusal> {
    let mut combinations: BTreeSet<u64> = BTreeSet::new();
    for persisted in given_persisted_sets {
        let mut combination = 0u64;
        for (position, entry) in location.writes.iter().enumerate() {
            if persisted[usize::try_from(*entry).expect("下标")] {
                combination |= 1u64 << position;
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
    combinations: BTreeSet<u64>,
) {
    let base_bytes = PoolReader::read(
        base,
        singlefs_core::address::DeviceIdentity(location.device),
        DeviceOffsetInBytes(location.offset),
        usize::try_from(location.length).expect("落点长度装得进 usize"),
    )
    .expect("落点在盘内、按扇区对齐");
    let location_start = location.offset;
    let location_length = u64::from(location.length);
    for combination in combinations {
        let mut bytes = base_bytes.clone();
        for (position, entry) in location.writes.iter().enumerate() {
            if combination & (1u64 << position) == 0 {
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
        location.versions.push(JudgeVersion {
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

/// 每一类落点的检查字。
///
/// - 系统配置：`[整槽校验和字段对得上, 0, 0, 0]`。
/// - 根槽：`[整槽校验和字段对得上（罩整个槽宽）, 0, 0, 0]`。
/// - journal 记录：`[整条校验和字段对得上, 载荷 CRC 对得上（点名项区在 4096 之内）, 反向链值 = CRC-32C(头 487 字节、校验和字段按 0), 0]`。
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
            let holds = bytes.len() >= usize::try_from(ROOT_RECORD_BYTES).expect("457")
                && wide_checksum_field_holds(bytes, bytes.len(), 138);
            [u32::from(holds), 0, 0, 0]
        }
        JudgeLocationKind::JournalRecord => {
            let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
            let header_bytes =
                usize::try_from(JOURNAL_HEADER_BYTES).expect("记录头 487 字节装得进 usize");
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
        let location_index = location_index_by_start
            .get(&(write.device.0, write.offset.0))
            .copied()
            .ok_or(JudgeTablesRefusal::WriteNotAlignedToALocation { write_index })?;
        if write.length_in_bytes() > u64::from(locations[location_index].length) {
            return Err(JudgeTablesRefusal::WriteNotAlignedToALocation { write_index });
        }
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
            location: Some(u32::try_from(location_index).expect("下标装得进 u32")),
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
        JudgeStateSource::GivenPersistedSets(given_persisted_sets) => {
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
    /// 序号 → 持久掩码：按段展开时与层 0、事实表同一套数字次序（每段照它的展开方式拆，
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
            JudgeStateSource::GivenPersistedSets(given_persisted_sets) => {
                given_persisted_sets[usize::try_from(ordinal).expect("序号装得进 usize")].clone()
            }
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
    ) -> Option<&'location JudgeVersion> {
        let mut combination = 0u64;
        for (position, entry) in location.writes.iter().enumerate() {
            if persisted[usize::try_from(*entry).expect("下标")] {
                combination |= 1u64 << position;
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

    /// 全部表的摘要（连同形态版本）。
    #[must_use]
    pub fn digest(&self) -> [u8; DIGEST_BYTES] {
        let mut message = JUDGE_TABLES_VERSION.to_le_bytes().to_vec();
        message.extend_from_slice(&self.words().digest_bytes());
        sha256_digest(&message)
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
        // 整组落的单元写那一组的起点与条数（只有甲二段有）；两组合起来是段里全部的写，之前的段整段持久时两组都置
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
        }
        // 给定状态表：每个状态 ⌈枚举写数 ÷ 32⌉ 个字的持久掩码（位 i 在第 i / 32 字的第 i % 32 位），按序号接；按段展开时这张表是空的
        let mut given_persisted_sets = Vec::new();
        if let JudgeStateSource::GivenPersistedSets(sets) = &self.state_source {
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
            versions.extend([
                0,
                0,
                checks[0],
                checks[1],
                checks[2],
                checks[3],
                NONE,
                0,
                crc_shifted_past_sixteen_zero_kibibytes(
                    JudgeLocationKind::UnitSlot,
                    &zero_unit[..usize::try_from(zero_unit_bytes).expect("单元长装得进 usize")],
                ),
            ]);
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
                let (combination_lo, combination_hi) = split(version.combination);
                versions.extend([
                    combination_lo,
                    index(version_bytes.len()),
                    version.checks[0],
                    version.checks[1],
                    version.checks[2],
                    version.checks[3],
                    index(equal_masks.len() / 2),
                    combination_hi,
                    crc_shifted_past_sixteen_zero_kibibytes(location.kind, &version.bytes),
                ]);
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
        }
    }
}

/// 段表每条几个 u32（布局写在 [`JudgeTables::words`] 填它的地方；内核里的同名常量由 `crate::crash_judge_gpu` 从这里生成）。
pub const SEGMENT_WORDS: usize = 9;

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
/// 版本表每条几个 u32：组合低 32 位、字节起点、四个检查字（[`version_checks`]）、逐扇区等不等掩码起点、组合高 32 位、
/// 第 8 字 [`VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES`]。
pub const VERSION_WORDS: usize = 9;
/// 版本表第 8 字（[`crc_shifted_past_sixteen_zero_kibibytes`]）：GPU 读者 `read_at` 拼读跨过落点末尾的 32 KiB 时拼整段 CRC 用
/// （`crash_judge_common.wgsl` 同名常量）。
pub const VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES: usize = 8;

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
        ]
    }
}

/// 打包表里几张表（内核里的 `TABLE_COUNT` 由 `crate::crash_judge_gpu` 从这里生成）。
pub const JUDGE_TABLE_COUNT: usize = 20;

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
    use super::{
        build_judge_tables, crc_shifted_past_sixteen_zero_kibibytes, judge_publishes, JudgeFlow,
        JudgeLocationKind, JudgePublish, JudgeStateSource, JudgeTables, JudgeTablesRefusal,
        JudgeTablesWords, SEGMENT_WORDS,
    };
    use crate::crash::Layer0SegmentExpansion;
    use crate::crash_facts::{
        SegmentExpansionFact, SegmentFact,
        SEGMENT_EXPANSION_CODE_IN_PLACE_SUBSETS_WITH_COPY_ON_WRITE_NONE_OR_ALL,
    };
    use singlefs_core::address::{
        CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
    };
    use singlefs_core::checksum::crc32_castagnoli;
    use singlefs_harness::memory_pool::{
        MemoryPool, RecordStreamSeam, RetainedWrite, WrittenContents,
    };
    use singlefs_harness::segments::StepKind;

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
                3
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
            },
            SegmentExpansionFact::EveryProperSubset,
            JudgeStateSource::GivenPersistedSets(vec![first.clone(), second.clone()]),
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

    /// 枚举写表多于内核持久掩码的宽（2048 次）：建表交「掩码表满了」，不 panic、不往下建。
    #[test]
    fn a_flow_with_more_enumerated_writes_than_the_mask_holds_is_refused_naming_the_mask() {
        let base = MemoryPool::with_devices(&[DeviceIdentity(0)], 1 << 26);
        let writes: Vec<RetainedWrite> = (0..2049u64)
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
        let expansion =
            |_segment_index: usize, _segment: &[usize]| Layer0SegmentExpansion::NotExpanded;
        let refusal = build_judge_tables(&JudgeFlow {
            base: &base,
            writes: &writes,
            segments: &segments,
            judged_root_index: 0,
            versions: &[],
            expansion: &expansion,
        })
        .expect_err("2049 次写装不进掩码");
        assert_eq!(
            refusal,
            JudgeTablesRefusal::EnumeratedWritesPastTheMaskWidth {
                enumerated_writes: 2049
            }
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
        let state_source = JudgeStateSource::GivenPersistedSets(vec![persisted]);
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
            &JudgeStateSource::GivenPersistedSets(vec![landed_on_the_first, landed_on_the_second]),
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
}
