//! 崩溃放量三段流的第 ① 段：CPU 把核对要的**事实**无脑写进 KV，不判对错。
//!
//! 一条流的枚举写表解析一次（用户 2026-09-28 定：CPU checker → 写入 KV → 读出 KV 核对（CPU 或 GPU）→ 写入 KV → 查出红汇总；
//! 同一种逻辑的活合在一段里做，不在每个状态上来回切）：每次单元写的类、两道校验和对不对、整单元 CRC、落在哪；每条根记录的字段、
//! 自证校验和对不对、四条指针；基线镜像（mkfs 之后）里已有的单元与根；哪几次写取三态、撕裂镜像与重放接在枚举写表的哪里；
//! 每一段展开出几个状态。**状态本身不存**：序号一到，掩码由 [`FlowFacts::persisted_writes_of_state`] 现算，与崩溃点重放的
//! `CrashStatePlan::persisted_writes_of_state` 同一套数字次序。
//!
//! 第 ② 段读事实判状态：GPU 内核在 `crash_verify_gpu`，CPU 参照实现是这里的 [`verify_state_from_facts`]——两份按同一张事实表、
//! 同一组判据算，逐状态相等是 GPU 那一份的判别力自证；第 ③ 段（`crash_amplification::compare_judge_and_verifier`）再把它们
//! 与 CPU 判器（`crash::judge_crash_image`）的判定逐状态比。
//!
//! 事实表进 KV 时按录入键存（录入那一组判法单元的键连同录入的输入摘要，`crate::crash_library::recording_key_of_the_inputs`；抽取事实的
//! 函数在录入那一组的入口里），[`FlowFacts::digest`] 只按内容取；核对表的键带核对那一组判法单元的键：换了代码就是新一轮，
//! 旧行留着当历史、不覆盖，每一段各自对账（用户 2026-09-28 定「事实表和核对表按代码摘要分版本存储、旧行保留为历史、每段独立对账」）。
use std::collections::BTreeMap;
use std::ops::Range;

use singlefs_checker::image::parse_node_pointer;
use singlefs_checker::{check_root_slot, check_unit, crc32_castagnoli_table};
use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::recovery::PoolReader;
use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES, ROOT_RECORD_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

use crate::crash::{
    add_to_the_mixed_radix_digits, crash_state_enumeration_tables_with_slices, no_segment_slices,
    CrashPointReplaySegmentExpansion, CrashPointReplaySegmentSlice, CrashStateEnumerationTables,
    StateCount1280Bits, STATE_COUNT_BYTES,
};

/// 事实表字节的布局版本：字节以 [`FACTS_LAYOUT_MAGIC`] 起头，布局变了就换它（版本 1 没有头、段里没有片起点；版本 2 每段带片起点那一格）。
/// 只给读库里的旧字节认布局用（读到别的头就拒，不按新布局错解），不进任何复用键：[`FlowFacts::digest`] 不收这个头。
pub const FACTS_LAYOUT_VERSION: u32 = 2;

/// 事实表字节的头：布局版本 [`FACTS_LAYOUT_VERSION`] 的写法。版本 1 的字节以录制流写数起头（小端 u32 两个），撞不上这 8 个字节
/// （头当成两个写数是 10 亿次写以上）；读到别的头就拒，不按新布局错解旧字节。
pub const FACTS_LAYOUT_MAGIC: &[u8; 8] = b"SFFACTS2";

/// 「基线镜像里本来就有」的单元或根，在事实里用这个来源号（不是任何一次写）。
pub const BASE_IMAGE_SOURCE: u32 = u32::MAX;

/// 根记录里四条节点指针的偏移（`singlefs_format::ROOT_RECORD_BYTES` 那一行的字段表：magic 4 + fsid 16 + flags 4 + 实例代号 4 +
/// checkpoint_txg 8 = 36 起是树表指针 86，之后树 ID 水位 8、回退下界 F 8、自证校验和 32，170 起实例表指针 86、256 起中央映射树根指针 86、
/// 342 起分配记录树根指针 86）。
pub const ROOT_POINTER_OFFSETS: [usize; 4] = [36, 170, 256, 342];
const ROOT_MAGIC: &[u8; 4] = b"SFSR";
const UNIT_MAGIC: &[u8; 4] = b"SFSU";
const UNIT_CLASS_OFFSET: usize = 6;
const SECTOR_BYTES: u64 = 512;

/// 核对结论的位：一位一条判据，0 就是这个状态在这几条上都成立。
pub mod verdict_bits {
    /// 最新持久的根指到的单元（整单元 CRC 与位置条目相符）自己的两道校验和有一道对不上。只判根指到的：崩溃状态里没被任何根引用的
    /// 半截单元校验和本来就对不上，池级 checker 也不判它们（第一版 GPU 对拍就打中过这一条：核对红、判器绿）。
    pub const UNIT_CHECKSUMS: u32 = 1 << 0;
    /// 最新持久的那条根自证校验和对不上（或 magic、flags 不合）。
    pub const NEWEST_ROOT_SELF_CHECKSUM: u32 = 1 << 1;
    /// 最新持久的那条根的某条非零指针，两份位置条目都指不到一个整单元 CRC 相符、且在这个状态里在盘上的单元。
    pub const NEWEST_ROOT_POINTER_TARGET: u32 = 1 << 2;
    /// 这个状态里一条根都没有（基线里也没有）：不该出现在崩溃点重放的池上，出现就是事实表或掩码算错。
    pub const NO_ROOT_AT_ALL: u32 = 1 << 3;
    /// 分片的段（C790）片起点的各位加片内序号越过了段的域：只有 GPU 核对内核记这一位（表与派活的序号对不上）；CPU 参照在同一处停下
    /// （`persisted_writes_of_state_by_the_segment_expansions` 断言），上卡之前已核过片起点（`check_the_slice_starts_against_the_expansions`），走不到。
    pub const SLICE_PAST_ITS_DOMAIN: u32 = 1 << 4;
}

/// 一个在盘上的单元（一次单元写，或基线镜像里本来就有的）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitFact {
    /// 枚举写表里的下标；基线里的是 [`BASE_IMAGE_SOURCE`]。
    pub source: u32,
    pub device: u32,
    pub slot: u64,
    pub unit_bytes: u32,
    pub class: u8,
    /// 池级 checker `check_unit` 判头校验和、载荷 CRC、magic、flags、预留位都过。
    pub checksums_hold: bool,
    /// 整单元 CRC-32C：位置条目里的校验和比的是它（`walk.rs` 的 `read_unit_without_judging`）。
    pub whole_crc32c: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationFact {
    pub device: u32,
    pub slot: u64,
    pub checksum: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerFact {
    pub all_zero: bool,
    pub locations: [LocationFact; 2],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootFact {
    /// 枚举写表里的下标；基线里的是 [`BASE_IMAGE_SOURCE`]。
    pub source: u32,
    pub instance: u32,
    pub checkpoint_txg: u64,
    pub rollback_floor: u64,
    pub self_checksum_holds: bool,
    pub pointers: [PointerFact; 4],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegmentFact {
    /// 这一段里录制流的写（写表下标，升序）。
    pub writes: Vec<u32>,
    /// 这一段展开出来的状态序号区间（不展开的段是空区间；分片的段是片长那么长）。
    pub states: Range<u64>,
    /// 分片的段（C790）：这一趟判它的域里从哪个编号起；不分片的段是 `None`。段内编号 = 片起点 + （序号 − `states.start`）。
    pub slice_start: Option<SegmentSliceStartFact>,
}

/// 分片的段的片起点：1280 位的编号本身（`crate::crash::StateCount1280Bits`，进字节是 [`STATE_COUNT_BYTES`] 个字节、即 20 个 u64 字），
/// 与它按这一段的混合进位（[`SegmentExpansionFact::digit_radices`]）拆成的各位（第 0 位最低，一位一个数字：单元写那一组落不落、
/// 或那一次写的落法）。拆片内第 i 个态时两个内核与 CPU 参照都从这几位起加 i、逐位进位（[`add_to_the_mixed_radix_digits`]），不做宽整数除法。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegmentSliceStartFact {
    pub start_within_the_domain: StateCount1280Bits,
    pub digits: Vec<u8>,
}

impl SegmentSliceStartFact {
    /// 片起点按这一段的各位拆开。
    ///
    /// # Panics
    /// 片起点不小于各进制之积（拆不下：片起点越过了段的域）。
    #[must_use]
    pub fn of(start_within_the_domain: StateCount1280Bits, radices: &[u8]) -> Self {
        Self {
            digits: start_within_the_domain
                .mixed_radix_digits(radices)
                .expect("片起点小于段的域，拆得进各位"),
            start_within_the_domain,
        }
    }
}

/// 一段的段内序号怎么拆成每次写的落地：`crate::crash::CrashPointReplaySegmentExpansion` 的三种，甲二那一种带上拆的时候要的两组写
/// （判决 `research/prompts/m3-crash-enumeration-merge-r2-main-verification.md` 第二节 L6-2：判表加一列段的展开方式、两个内核照它拆）。
/// 与 `crate::crash::CrashStatePlan::persisted_writes_of_state` 同一套数字次序：全量时段内序号 r 按混合进制拆到段里全部的写上
/// （第一次写是最低位）；甲二时单元写一个都没有就同全量（只拆到原地写上），有单元写时 r / 2 拆到原地写上、r 除以 2 的余数是「单元写全落」。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SegmentExpansionFact {
    /// 不展开：段内一个状态都没有，只以整段持久进入后面的状态。
    NotExpanded,
    /// 段内每次写各取它的几态、任意组合，去掉整段全持久那一个。
    EveryProperSubset,
    /// 甲二：原地写（单元写之外的写）各取它的几态、任意组合 × 单元写只取全不落或全落，去掉整段全落那一个。两组写各按段内次序。
    InPlaceSubsetsWithCopyOnWriteNoneOrAll {
        in_place_writes: Vec<u32>,
        copy_on_write_writes: Vec<u32>,
    },
}

/// GPU 表里段的展开方式码（判器内核与第 ② 段核对内核的段表第 6 字，[`segment_expansion_code_wgsl_constants`] 生成进内核）。
pub const SEGMENT_EXPANSION_CODE_NOT_EXPANDED: u32 = 0;
pub const SEGMENT_EXPANSION_CODE_EVERY_PROPER_SUBSET: u32 = 1;
pub const SEGMENT_EXPANSION_CODE_IN_PLACE_SUBSETS_WITH_COPY_ON_WRITE_NONE_OR_ALL: u32 = 2;

impl SegmentExpansionFact {
    /// 一段按 `expansion` 展开时的拆法：甲二那一种按 [`in_place_and_copy_on_write_writes_of_the_segment`] 分两组。
    #[must_use]
    pub fn of(
        expansion: CrashPointReplaySegmentExpansion,
        writes: &[RetainedWrite],
        segment: &[usize],
    ) -> Self {
        match expansion {
            CrashPointReplaySegmentExpansion::NotExpanded => Self::NotExpanded,
            CrashPointReplaySegmentExpansion::EveryProperSubset => Self::EveryProperSubset,
            CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
                let (in_place_writes, copy_on_write_writes) =
                    in_place_and_copy_on_write_writes_of_the_segment(writes, segment);
                Self::InPlaceSubsetsWithCopyOnWriteNoneOrAll {
                    in_place_writes,
                    copy_on_write_writes,
                }
            }
        }
    }

    /// GPU 表里的码。
    #[must_use]
    pub fn code(&self) -> u32 {
        match self {
            Self::NotExpanded => SEGMENT_EXPANSION_CODE_NOT_EXPANDED,
            Self::EveryProperSubset => SEGMENT_EXPANSION_CODE_EVERY_PROPER_SUBSET,
            Self::InPlaceSubsetsWithCopyOnWriteNoneOrAll { .. } => {
                SEGMENT_EXPANSION_CODE_IN_PLACE_SUBSETS_WITH_COPY_ON_WRITE_NONE_OR_ALL
            }
        }
    }

    /// 段内序号逐位拆到哪几次写上（次序就是数位次序，第一次写是最低位），与这一段之外还要整组一起落的单元写：
    /// 全量与不展开时是段里全部的写、没有整组；甲二时是原地写、整组是单元写（单元写为空时同全量）。
    #[must_use]
    pub fn digit_writes_and_the_copy_on_write_group<'segment>(
        &'segment self,
        segment: &'segment SegmentFact,
    ) -> (&'segment [u32], &'segment [u32]) {
        match self {
            Self::NotExpanded | Self::EveryProperSubset => (&segment.writes, &[]),
            Self::InPlaceSubsetsWithCopyOnWriteNoneOrAll {
                in_place_writes,
                copy_on_write_writes,
            } => (in_place_writes, copy_on_write_writes),
        }
    }

    /// 这一段拆编号时混合进位每一位的进制（第 0 位最低）：数位那一组写各是它能取几态（`is_tearable` 里为真的三态、其余两态），
    /// 整组落的单元写那一组不空时最低一位是它（两进制）。与 `crate::crash` 里分片的段拆编号的各位同一个次序。
    #[must_use]
    pub fn digit_radices(&self, segment: &SegmentFact, is_tearable: &[bool]) -> Vec<u8> {
        if *self == Self::NotExpanded {
            return Vec::new();
        }
        let (digit_writes, copy_on_write_group) =
            self.digit_writes_and_the_copy_on_write_group(segment);
        let mut radices = Vec::with_capacity(digit_writes.len() + 1);
        if !copy_on_write_group.is_empty() {
            radices.push(2);
        }
        radices.extend(digit_writes.iter().map(|write| {
            if is_tearable[usize::try_from(*write).expect("下标装得进 usize")] {
                3
            } else {
                2
            }
        }));
        radices
    }

    /// 这一段按这种拆法的整域态数（1280 位）：各进制之积减去整段全持久那一个；不展开的段是 0。越过 1280 位交 `None`。
    #[must_use]
    pub fn domain_state_count(
        &self,
        segment: &SegmentFact,
        is_tearable: &[bool],
    ) -> Option<StateCount1280Bits> {
        if *self == Self::NotExpanded {
            return Some(StateCount1280Bits::ZERO);
        }
        self.digit_radices(segment, is_tearable)
            .iter()
            .try_fold(StateCount1280Bits::from_u64(1), |product, radix| {
                product.checked_mul_u64(u64::from(*radix))
            })?
            .checked_sub(&StateCount1280Bits::from_u64(1))
    }

    /// 这一段按这种拆法展开出几个状态（`is_tearable` 里为真的写取三态，其余取两态）；装不进 u64 交 `None`。
    #[must_use]
    pub fn state_count(&self, segment: &SegmentFact, is_tearable: &[bool]) -> Option<u64> {
        let combinations_of = |write_indexes: &[u32]| {
            write_indexes.iter().try_fold(1u64, |combinations, write| {
                let choices = if is_tearable[usize::try_from(*write).expect("下标装得进 usize")]
                {
                    3
                } else {
                    2
                };
                combinations.checked_mul(choices)
            })
        };
        match self {
            Self::NotExpanded => Some(0),
            Self::EveryProperSubset => combinations_of(&segment.writes)?.checked_sub(1),
            Self::InPlaceSubsetsWithCopyOnWriteNoneOrAll {
                in_place_writes,
                copy_on_write_writes,
            } => {
                let copy_on_write_choices = if copy_on_write_writes.is_empty() {
                    1
                } else {
                    2
                };
                combinations_of(in_place_writes)?
                    .checked_mul(copy_on_write_choices)?
                    .checked_sub(1)
            }
        }
    }
}

/// 一段里的原地写与单元写（COW），各按段内次序：单元写落在新分出来的槽上，其余（系统配置槽、journal 记录、根槽、清零、discard、屏障）都是原地覆盖。
/// 与 `crate::crash` 里甲二拆段的那一份同一条分法（那一份是 `crash.rs` 的私有函数 `in_place_and_copy_on_write_writes_of`，崩溃点重放引擎那一路
/// 合并时改成调这一份）。
#[must_use]
pub fn in_place_and_copy_on_write_writes_of_the_segment(
    writes: &[RetainedWrite],
    segment: &[usize],
) -> (Vec<u32>, Vec<u32>) {
    let write_index = |index: usize| u32::try_from(index).expect("写表下标装得进 u32");
    let (in_place_writes, copy_on_write_writes): (Vec<usize>, Vec<usize>) = segment
        .iter()
        .copied()
        .partition(|index| match writes[*index].kind {
            StepKind::UnitWrite => false,
            StepKind::ZeroFill
            | StepKind::Discard
            | StepKind::SystemConfigurationSlot
            | StepKind::JournalRecord
            | StepKind::RootRecordFua
            | StepKind::Barrier => true,
        });
    (
        in_place_writes.into_iter().map(write_index).collect(),
        copy_on_write_writes.into_iter().map(write_index).collect(),
    )
}

/// 每一段按 `expansion` 的拆法（与 `segments` 同序）。`expansion` 与枚举、抽事实时给的同一个。
#[must_use]
pub fn segment_expansion_facts(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Vec<SegmentExpansionFact> {
    segments
        .iter()
        .enumerate()
        .map(|(segment_index, segment)| {
            SegmentExpansionFact::of(expansion(segment_index, segment), writes, segment)
        })
        .collect()
}

/// 段的展开方式码在内核里的常量（WGSL 源码）：判器的三个内核与第 ② 段核对内核都接在源码最前面，不在 WGSL 里另写一份。
#[must_use]
pub fn segment_expansion_code_wgsl_constants() -> String {
    format!(
        "const SEGMENT_EXPANSION_CODE_NOT_EXPANDED: u32 = {SEGMENT_EXPANSION_CODE_NOT_EXPANDED}u;\n\
         const SEGMENT_EXPANSION_CODE_EVERY_PROPER_SUBSET: u32 = {SEGMENT_EXPANSION_CODE_EVERY_PROPER_SUBSET}u;\n\
         const SEGMENT_EXPANSION_CODE_IN_PLACE_SUBSETS_WITH_COPY_ON_WRITE_NONE_OR_ALL: u32 = {SEGMENT_EXPANSION_CODE_IN_PLACE_SUBSETS_WITH_COPY_ON_WRITE_NONE_OR_ALL}u;\n"
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TornImageFact {
    pub write_index: u32,
    pub torn_image_index: u32,
    /// （重放在枚举写表里的下标，重放的是写表里哪一次写）。
    pub replays: Vec<(u32, u32)>,
}

/// 一条流的全部事实。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowFacts {
    pub recorded_write_count: u32,
    pub enumerated_write_count: u32,
    pub is_tearable: Vec<bool>,
    pub torn_images: Vec<TornImageFact>,
    pub segments: Vec<SegmentFact>,
    pub state_count: u64,
    pub units: Vec<UnitFact>,
    pub roots: Vec<RootFact>,
    /// 基线里最新的那条根（按 checkpoint_txg 取大）在 `roots` 里的下标；基线里没有根就是 None。
    pub newest_base_root: Option<u32>,
}

/// 枚举写表最多几次写：GPU 判器与核对内核的持久掩码各是 [`PERSISTED_MASK_WORDS`] 个 u32 一位一次写。
/// 随机历史越过原分配记录墙那一族一段录制写 4436 到 5265 次（2026-10-18 调查员量的，全量那一趟建不出判表的 32 张），
/// 2048 装不下；8192 留出余量。
pub const MAXIMUM_ENUMERATED_WRITES: usize = 8192;
/// 持久掩码几个 u32（位 i 在第 i / 32 字的第 i % 32 位）：GPU 两个内核里的 `array<u32, PERSISTED_MASK_WORDS>` 由这里生成，不在 WGSL 里另写一个数。
pub const PERSISTED_MASK_WORDS: usize = MAXIMUM_ENUMERATED_WRITES / 32;
const _: () = assert!(PERSISTED_MASK_WORDS * 32 == MAXIMUM_ENUMERATED_WRITES);

/// 从一条流抽事实（第 ① 段）。`expansion` 与枚举时给的同一个。
///
/// # Panics
/// 枚举写表超过 GPU 内核掩码装得下的 [`MAXIMUM_ENUMERATED_WRITES`]；写表里有单元写不是普通写（清零写不是单元）。
#[must_use]
pub fn extract_flow_facts(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> FlowFacts {
    extract_flow_facts_with_slices(
        base,
        writes,
        segments,
        expansion,
        &no_segment_slices(segments.len()),
    )
}

/// 同 [`extract_flow_facts`]，分片的段照 `segment_slices`（与段同序）：它在事实表里的序号区间是片长、带片起点（[`SegmentSliceStartFact`]）。
///
/// # Panics
/// 同 [`extract_flow_facts`]；片越过了段的域（`crate::crash` 建计划时停下）。
#[must_use]
pub fn extract_flow_facts_with_slices(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
) -> FlowFacts {
    let tables: CrashStateEnumerationTables = crash_state_enumeration_tables_with_slices(
        base,
        writes,
        segments,
        expansion,
        segment_slices,
    );
    let enumerated = &tables.enumerated_writes;
    assert!(
        enumerated.len() <= MAXIMUM_ENUMERATED_WRITES,
        "枚举写表 {} 次写，GPU 内核的掩码只装得下 {MAXIMUM_ENUMERATED_WRITES}",
        enumerated.len()
    );
    let mut units = base_units(base);
    let mut roots = base_roots(base);
    let newest_base_root = roots
        .iter()
        .enumerate()
        .max_by_key(|(_, root)| root.checkpoint_txg)
        .map(|(index, _)| u32::try_from(index).expect("根数装得进 u32"));
    for (write_index, write) in enumerated.iter().enumerate() {
        let source = u32::try_from(write_index).expect("写表下标装得进 u32");
        match write.kind {
            StepKind::UnitWrite => {
                let bytes = write.bytes().expect("单元写是普通写：清零不是单元");
                units.push(unit_fact(source, write.device.0, write.offset.0, bytes));
            }
            StepKind::RootRecordFua => {
                let bytes = write.bytes().expect("根槽写是普通写");
                roots.push(root_fact(source, bytes));
            }
            // discard（与它的撕裂镜像）把罩住的单元改成 0 或撕裂的字节：那几个槽在它持久的状态里不再是原来那个单元。
            StepKind::Discard => {
                let units_the_discard_changes =
                    units_whose_bytes_a_discard_changes(base, enumerated, &units, source, write);
                units.extend(units_the_discard_changes);
            }
            StepKind::ZeroFill
            | StepKind::JournalRecord
            | StepKind::SystemConfigurationSlot
            | StepKind::Barrier => {}
        }
    }
    FlowFacts {
        recorded_write_count: u32::try_from(writes.len()).expect("写数装得进 u32"),
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
        segments: segment_facts_of_the_tables(writes, segments, expansion, &tables),
        state_count: tables.state_count,
        units,
        roots,
        newest_base_root,
    }
}

/// 每一段的事实（写、序号区间、分片的段的片起点与它拆成的各位），与段同序：事实表与 GPU 判表都照这一份建段表。
#[must_use]
pub fn segment_facts_of_the_tables(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    tables: &CrashStateEnumerationTables,
) -> Vec<SegmentFact> {
    segments
        .iter()
        .enumerate()
        .zip(&tables.state_ranges_by_segment)
        .zip(&tables.segment_slices)
        .map(|(((segment_index, segment), states), slice)| {
            let mut fact = SegmentFact {
                writes: segment
                    .iter()
                    .map(|write| u32::try_from(*write).expect("下标装得进 u32"))
                    .collect(),
                states: states.clone(),
                slice_start: None,
            };
            if let Some(slice) = slice {
                let radices =
                    SegmentExpansionFact::of(expansion(segment_index, segment), writes, segment)
                        .digit_radices(&fact, &tables.is_tearable_by_write);
                fact.slice_start = Some(SegmentSliceStartFact::of(
                    slice.start_within_the_domain,
                    &radices,
                ));
            }
            fact
        })
        .collect()
}

/// 分片的段的片起点与它的各位对得上、片没越过段的域（片起点 + 片长不超过整域的态数），各位都小于各自的进制；不分片的段不核
/// （它们照旧核「展开出的状态数等于序号区间」）。交回第一处对不上的段号。事实表读回之后、上卡与 CPU 参照拆序号之前核一次。
///
/// # Errors
/// 第一处对不上的段号。
pub fn check_the_slice_starts_against_the_expansions(
    segments: &[SegmentFact],
    segment_expansions: &[SegmentExpansionFact],
    is_tearable: &[bool],
) -> Result<(), usize> {
    for (segment_index, (segment, expansion)) in segments.iter().zip(segment_expansions).enumerate()
    {
        let Some(slice_start) = &segment.slice_start else {
            continue;
        };
        let radices = expansion.digit_radices(segment, is_tearable);
        let agrees = StateCount1280Bits::of_mixed_radix_digits(&slice_start.digits, &radices)
            == Some(slice_start.start_within_the_domain)
            && expansion
                .domain_state_count(segment, is_tearable)
                .zip(
                    slice_start
                        .start_within_the_domain
                        .checked_add_u64(segment.states.end - segment.states.start),
                )
                .is_some_and(|(domain, slice_end)| !radices.is_empty() && slice_end <= domain);
        if !agrees {
            return Err(segment_index);
        }
    }
    Ok(())
}

fn unit_fact(source: u32, device: u32, offset: u64, bytes: &[u8]) -> UnitFact {
    UnitFact {
        source,
        device,
        slot: offset / SLOT_BYTES,
        unit_bytes: u32::try_from(bytes.len()).expect("单元长度装得进 u32"),
        class: bytes.get(UNIT_CLASS_OFFSET).copied().unwrap_or(0),
        checksums_hold: check_unit(bytes).is_ok(),
        whole_crc32c: crc32_castagnoli_table(bytes),
    }
}

fn zero_pointer() -> PointerFact {
    PointerFact {
        all_zero: true,
        locations: [LocationFact {
            device: 0,
            slot: 0,
            checksum: 0,
        }; 2],
    }
}

fn root_fact(source: u32, bytes: &[u8]) -> RootFact {
    let record_bytes = usize::try_from(ROOT_RECORD_BYTES).expect("466");
    let mut filesystem_identifier = [0u8; 16];
    if bytes.len() >= 20 {
        filesystem_identifier.copy_from_slice(&bytes[4..20]);
    }
    // 自证：magic、整槽校验和、flags；fsid 拿它自己的（同一个池的判定归池级 checker，这里只判它自己站不站得住）
    let view = check_root_slot(bytes, &filesystem_identifier).ok();
    let pointers = ROOT_POINTER_OFFSETS.map(|offset| {
        if bytes.len() >= record_bytes {
            let pointer = parse_node_pointer(&bytes[offset..]);
            PointerFact {
                all_zero: pointer.all_zero,
                locations: pointer.locations.map(|location| LocationFact {
                    device: location.device,
                    slot: location.slot,
                    checksum: location.checksum,
                }),
            }
        } else {
            zero_pointer()
        }
    });
    RootFact {
        source,
        instance: view.as_ref().map_or(0, |view| view.instance),
        checkpoint_txg: view.as_ref().map_or(0, |view| view.checkpoint_txg),
        rollback_floor: view.as_ref().map_or(0, |view| view.rollback_floor),
        self_checksum_holds: view.is_some(),
        pointers,
    }
}

/// 一次 discard（或它的撕裂镜像，来源号 `source`）落了之后，被它改掉字节的单元各添一条以这次写为来源的单元事实：字节是那个单元
/// 叠上这次写之后的样子（整单元 CRC 与两道校验和按它算）。状态里这次写持久时 [`current_unit_at`] 取到的是这一条，根指过去按整单元 CRC
/// 对不上，与 CPU 判器在镜像上读到的一致（discard 罩住的单元读回 0 或撕裂的字节）；没持久时取到的还是原来那个单元。
/// 「原来那个单元」取这次写之前最后一次落在那个槽上的单元写（没有就取基线里的），与撕裂镜像按「这次写之前的写全落了」取旧字节
/// 同一个口径（D13（验证路线） 已定项 4 射程「已知边角」）；更早的 discard 添的那几条不是单元，不拿来叠。字节没变的单元不添
/// （撕裂镜像后一半罩住的还是旧字节）。
fn units_whose_bytes_a_discard_changes(
    base: &MemoryPool,
    enumerated: &[RetainedWrite],
    units_before_the_discard: &[UnitFact],
    source: u32,
    discard: &RetainedWrite,
) -> Vec<UnitFact> {
    let discard_start = discard.offset.0;
    let discard_end = discard_start + discard.length_in_bytes();
    let write_of_source =
        |unit_source: u32| &enumerated[usize::try_from(unit_source).expect("写表下标装得进 usize")];
    // `units_before_the_discard` 按来源号从小到大排（基线的在最前，之后按写表次序），同一槽后来的盖掉先来的。
    let mut latest_unit_by_slot: BTreeMap<u64, &UnitFact> = BTreeMap::new();
    for unit in units_before_the_discard {
        let is_from_the_base_or_a_unit_write = unit.source == BASE_IMAGE_SOURCE
            || write_of_source(unit.source).kind == StepKind::UnitWrite;
        let unit_start = unit.slot * SLOT_BYTES;
        let unit_end = unit_start + u64::from(unit.unit_bytes);
        if unit.device == discard.device.0
            && is_from_the_base_or_a_unit_write
            && unit_start < discard_end
            && discard_start < unit_end
        {
            latest_unit_by_slot.insert(unit.slot, unit);
        }
    }
    latest_unit_by_slot
        .into_values()
        .filter_map(|unit| {
            let unit_start = unit.slot * SLOT_BYTES;
            let unit_end = unit_start + u64::from(unit.unit_bytes);
            let original: Vec<u8> = if unit.source == BASE_IMAGE_SOURCE {
                PoolReader::read(
                    base,
                    discard.device,
                    DeviceOffsetInBytes(unit_start),
                    usize::try_from(unit.unit_bytes).expect("单元长度装得进 usize"),
                )
                .expect("基线里的单元在建事实时读出来过")
            } else {
                write_of_source(unit.source)
                    .bytes()
                    .expect("单元写是普通写")
                    .to_vec()
            };
            let overlap_start = unit_start.max(discard_start);
            let overlap_end = unit_end.min(discard_end);
            let offset_in_the_unit = |position: u64| {
                usize::try_from(position - unit_start).expect("单元内偏移装得进 usize")
            };
            let mut present = original.clone();
            discard.contents.copy_range_into(
                usize::try_from(overlap_start - discard_start).expect("写内偏移装得进 usize"),
                &mut present[offset_in_the_unit(overlap_start)..offset_in_the_unit(overlap_end)],
            );
            (present != original).then(|| unit_fact(source, unit.device, unit_start, &present))
        })
        .collect()
}

/// 基线镜像里的单元：写过的扇区里、16 KiB 槽起点上带单元 magic 的，按类读整单元。
fn base_units(base: &MemoryPool) -> Vec<UnitFact> {
    let mut units = Vec::new();
    for (device, sparse) in &base.devices {
        for (sector, bytes) in sparse.written_sectors() {
            let offset = sector * SECTOR_BYTES;
            if !offset.is_multiple_of(SLOT_BYTES) || bytes.len() < 8 || &bytes[..4] != UNIT_MAGIC {
                continue;
            }
            let unit_bytes = match bytes[UNIT_CLASS_OFFSET] {
                2 => NODE_BYTES,
                _ => DATA_UNIT_BYTES,
            };
            let Some(unit) = PoolReader::read(
                base,
                *device,
                DeviceOffsetInBytes(offset),
                usize::try_from(unit_bytes).expect("单元长度装得进 usize"),
            ) else {
                continue;
            };
            units.push(unit_fact(BASE_IMAGE_SOURCE, device.0, offset, &unit));
        }
    }
    units
}

/// 基线镜像里的根记录：写过的扇区里带根 magic 的（根槽写是一个物理块，崩溃点重放的池按 512 字节建）。
fn base_roots(base: &MemoryPool) -> Vec<RootFact> {
    let mut roots = Vec::new();
    for sparse in base.devices.values() {
        for bytes in sparse.written_sectors().values() {
            if bytes.len() >= 4 && &bytes[..4] == ROOT_MAGIC {
                roots.push(root_fact(BASE_IMAGE_SOURCE, bytes));
            }
        }
    }
    roots
}

impl FlowFacts {
    /// 事实表的摘要：只按内容取（布局头 [`FACTS_LAYOUT_MAGIC`] 之后的字节），进核对键。抽取规则变了由录入那一组判法单元的键管，
    /// 不靠手写的版本号。
    #[must_use]
    pub fn digest(&self) -> [u8; DIGEST_BYTES] {
        let bytes = self.to_bytes();
        sha256_digest(&bytes[FACTS_LAYOUT_MAGIC.len()..])
    }

    /// 序号落在哪一段；等于段数就是最后全部持久那一个状态。
    #[must_use]
    pub fn segment_of_state(&self, ordinal: u64) -> usize {
        self.segments
            .partition_point(|segment| segment.states.end <= ordinal)
    }

    /// 这个状态的持久集合，按枚举写表给；与崩溃点重放 `CrashStatePlan::persisted_writes_of_state` 同一套数字次序
    /// （两态的写数字 0 / 1 = 没持久 / 持久，三态的写 0 / 1 / 2 = 没持久 / 撕裂 / 持久；撕裂那一态记成原写没持久、撕裂镜像持久，
    /// 撕裂镜像之后的重放跟着被重放的那次写落不落）。
    ///
    /// # Panics
    /// 序号越过状态总数。
    #[must_use]
    pub fn persisted_writes_of_state(&self, ordinal: u64) -> Vec<bool> {
        persisted_writes_of_state(
            usize::try_from(self.recorded_write_count).expect("装得进 usize"),
            usize::try_from(self.enumerated_write_count).expect("装得进 usize"),
            &self.is_tearable,
            &self.torn_images,
            &self.segments,
            self.state_count,
            ordinal,
        )
    }

    /// 同 [`Self::persisted_writes_of_state`]，每一段照 `segment_expansions` 那一段的拆法（有甲二段的流）。
    ///
    /// # Panics
    /// 同 [`persisted_writes_of_state_by_the_segment_expansions`]。
    #[must_use]
    pub fn persisted_writes_of_state_by_the_segment_expansions(
        &self,
        segment_expansions: &[SegmentExpansionFact],
        ordinal: u64,
    ) -> Vec<bool> {
        persisted_writes_of_state_by_the_segment_expansions(
            usize::try_from(self.recorded_write_count).expect("装得进 usize"),
            usize::try_from(self.enumerated_write_count).expect("装得进 usize"),
            &self.is_tearable,
            &self.torn_images,
            &self.segments,
            segment_expansions,
            self.state_count,
            ordinal,
        )
    }

    /// 序列化：全部小端定宽，字段次序就是 [`Self::from_bytes`] 读的次序。
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = FACTS_LAYOUT_MAGIC.to_vec();
        let put32 = |serialized: &mut Vec<u8>, value: u32| {
            serialized.extend_from_slice(&value.to_le_bytes())
        };
        let put64 = |serialized: &mut Vec<u8>, value: u64| {
            serialized.extend_from_slice(&value.to_le_bytes())
        };
        put32(&mut out, self.recorded_write_count);
        put32(&mut out, self.enumerated_write_count);
        put32(
            &mut out,
            u32::try_from(self.is_tearable.len()).expect("装得进"),
        );
        out.extend(self.is_tearable.iter().map(|tearable| u8::from(*tearable)));
        put32(
            &mut out,
            u32::try_from(self.torn_images.len()).expect("装得进"),
        );
        for torn in &self.torn_images {
            put32(&mut out, torn.write_index);
            put32(&mut out, torn.torn_image_index);
            put32(&mut out, u32::try_from(torn.replays.len()).expect("装得进"));
            for (replay_index, replayed) in &torn.replays {
                put32(&mut out, *replay_index);
                put32(&mut out, *replayed);
            }
        }
        put32(
            &mut out,
            u32::try_from(self.segments.len()).expect("装得进"),
        );
        for segment in &self.segments {
            put32(
                &mut out,
                u32::try_from(segment.writes.len()).expect("装得进"),
            );
            for write in &segment.writes {
                put32(&mut out, *write);
            }
            put64(&mut out, segment.states.start);
            put64(&mut out, segment.states.end);
            match &segment.slice_start {
                None => out.push(0),
                Some(slice_start) => {
                    out.push(1);
                    out.extend_from_slice(&slice_start.start_within_the_domain.to_le_bytes());
                    put32(
                        &mut out,
                        u32::try_from(slice_start.digits.len()).expect("装得进"),
                    );
                    out.extend_from_slice(&slice_start.digits);
                }
            }
        }
        put64(&mut out, self.state_count);
        put32(&mut out, u32::try_from(self.units.len()).expect("装得进"));
        for unit in &self.units {
            put32(&mut out, unit.source);
            put32(&mut out, unit.device);
            put64(&mut out, unit.slot);
            put32(&mut out, unit.unit_bytes);
            out.push(unit.class);
            out.push(u8::from(unit.checksums_hold));
            put32(&mut out, unit.whole_crc32c);
        }
        put32(&mut out, u32::try_from(self.roots.len()).expect("装得进"));
        for root in &self.roots {
            put32(&mut out, root.source);
            put32(&mut out, root.instance);
            put64(&mut out, root.checkpoint_txg);
            put64(&mut out, root.rollback_floor);
            out.push(u8::from(root.self_checksum_holds));
            for pointer in &root.pointers {
                out.push(u8::from(pointer.all_zero));
                for location in &pointer.locations {
                    put32(&mut out, location.device);
                    put64(&mut out, location.slot);
                    put32(&mut out, location.checksum);
                }
            }
        }
        match self.newest_base_root {
            Some(index) => put32(&mut out, index),
            None => put32(&mut out, u32::MAX),
        }
        out
    }

    /// 反序列化；不是布局版本 [`FACTS_LAYOUT_VERSION`] 的头（[`FACTS_LAYOUT_MAGIC`]）、字节不够或对不上返回 None。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut cursor = Cursor { bytes, at: 0 };
        if cursor.take(FACTS_LAYOUT_MAGIC.len())? != FACTS_LAYOUT_MAGIC {
            return None;
        }
        let recorded_write_count = cursor.u32()?;
        let enumerated_write_count = cursor.u32()?;
        let tearable_count = cursor.u32()? as usize;
        let mut is_tearable = Vec::with_capacity(tearable_count);
        for _ in 0..tearable_count {
            is_tearable.push(cursor.u8()? != 0);
        }
        let torn_count = cursor.u32()? as usize;
        let mut torn_images = Vec::with_capacity(torn_count);
        for _ in 0..torn_count {
            let write_index = cursor.u32()?;
            let torn_image_index = cursor.u32()?;
            let replay_count = cursor.u32()? as usize;
            let mut replays = Vec::with_capacity(replay_count);
            for _ in 0..replay_count {
                replays.push((cursor.u32()?, cursor.u32()?));
            }
            torn_images.push(TornImageFact {
                write_index,
                torn_image_index,
                replays,
            });
        }
        let segment_count = cursor.u32()? as usize;
        let mut segments = Vec::with_capacity(segment_count);
        for _ in 0..segment_count {
            let write_count = cursor.u32()? as usize;
            let mut writes = Vec::with_capacity(write_count);
            for _ in 0..write_count {
                writes.push(cursor.u32()?);
            }
            let start = cursor.u64()?;
            let end = cursor.u64()?;
            let slice_start = match cursor.u8()? {
                0 => None,
                1 => {
                    let start_within_the_domain = StateCount1280Bits::from_le_bytes(
                        cursor.take(STATE_COUNT_BYTES)?.try_into().ok()?,
                    );
                    let digit_count = cursor.u32()? as usize;
                    let digits = cursor.take(digit_count)?.to_vec();
                    Some(SegmentSliceStartFact {
                        start_within_the_domain,
                        digits,
                    })
                }
                _ => return None,
            };
            segments.push(SegmentFact {
                writes,
                states: start..end,
                slice_start,
            });
        }
        let state_count = cursor.u64()?;
        let unit_count = cursor.u32()? as usize;
        let mut units = Vec::with_capacity(unit_count);
        for _ in 0..unit_count {
            units.push(UnitFact {
                source: cursor.u32()?,
                device: cursor.u32()?,
                slot: cursor.u64()?,
                unit_bytes: cursor.u32()?,
                class: cursor.u8()?,
                checksums_hold: cursor.u8()? != 0,
                whole_crc32c: cursor.u32()?,
            });
        }
        let root_count = cursor.u32()? as usize;
        let mut roots = Vec::with_capacity(root_count);
        for _ in 0..root_count {
            let source = cursor.u32()?;
            let instance = cursor.u32()?;
            let checkpoint_txg = cursor.u64()?;
            let rollback_floor = cursor.u64()?;
            let self_checksum_holds = cursor.u8()? != 0;
            let mut pointers = [zero_pointer(); 4];
            for pointer in &mut pointers {
                pointer.all_zero = cursor.u8()? != 0;
                for location in &mut pointer.locations {
                    location.device = cursor.u32()?;
                    location.slot = cursor.u64()?;
                    location.checksum = cursor.u32()?;
                }
            }
            roots.push(RootFact {
                source,
                instance,
                checkpoint_txg,
                rollback_floor,
                self_checksum_holds,
                pointers,
            });
        }
        let newest_base_root = match cursor.u32()? {
            u32::MAX => None,
            index => Some(index),
        };
        if cursor.at != bytes.len() {
            return None;
        }
        Some(Self {
            recorded_write_count,
            enumerated_write_count,
            is_tearable,
            torn_images,
            segments,
            state_count,
            units,
            roots,
            newest_base_root,
        })
    }
}

/// 从判定库里读回、核过摘要的一份事实表：第 ② 段核对（CPU 参照与 GPU 核对器）只收它。字段私有，只有
/// [`FlowFactsReadBack::from_the_library_bytes`] 造得出（读库那一步在 `crash_library::read_flow_facts_back`：事实表先入库、再读库，
/// 核对只照库里那一份做；这一份不引判定库，判法闭包里没有存取布局）。读它照读 [`FlowFacts`]。
#[derive(Debug)]
pub struct FlowFactsReadBack {
    facts: FlowFacts,
}

impl FlowFactsReadBack {
    /// 库里按 `digest` 存的那份字节：解码、核摘要。
    ///
    /// # Errors
    /// 解不开；解出来的摘要对不上（库里那份被改过）。
    pub fn from_the_library_bytes(
        bytes: &[u8],
        digest: &[u8; DIGEST_BYTES],
    ) -> Result<Self, String> {
        if !bytes.starts_with(FACTS_LAYOUT_MAGIC) {
            return Err(format!(
                "事实表的布局不是版本 {FACTS_LAYOUT_VERSION}（头不是 {}）：旧布局（版本 1，没有片起点）的事实表不认、不按新布局解，重跑录入段",
                String::from_utf8_lossy(FACTS_LAYOUT_MAGIC)
            ));
        }
        let facts = FlowFacts::from_bytes(bytes)
            .ok_or_else(|| "事实表解不出（字节坏了或编码换了）：重跑录入段".to_string())?;
        if facts.digest() != *digest {
            return Err("事实表摘要对不上：库里那份坏了，重跑录入段".to_string());
        }
        Ok(Self { facts })
    }
}

impl std::ops::Deref for FlowFactsReadBack {
    type Target = FlowFacts;

    fn deref(&self) -> &Self::Target {
        &self.facts
    }
}

/// 序号 → 持久掩码，按枚举写表给，每一段都按全量（[`SegmentExpansionFact::EveryProperSubset`]）拆：不展开的段序号区间是空的、
/// 拆不到它，所以「每段要么全量展开、要么不展开」的流（崩溃放量按展开上限切的那一种）照用这一份；有甲二段的流用
/// [`persisted_writes_of_state_by_the_segment_expansions`]。数字次序见那一份。
///
/// # Panics
/// 序号越过状态总数；同 [`persisted_writes_of_state_by_the_segment_expansions`]。
#[must_use]
pub fn persisted_writes_of_state(
    recorded_write_count: usize,
    enumerated_write_count: usize,
    is_tearable: &[bool],
    torn_images: &[TornImageFact],
    segments: &[SegmentFact],
    state_count: u64,
    ordinal: u64,
) -> Vec<bool> {
    persisted_writes_of_state_by_the_segment_expansions(
        recorded_write_count,
        enumerated_write_count,
        is_tearable,
        torn_images,
        segments,
        &vec![SegmentExpansionFact::EveryProperSubset; segments.len()],
        state_count,
        ordinal,
    )
}

/// 序号 → 持久掩码，按枚举写表给，每一段照 `segment_expansions` 那一段的拆法（事实表与 GPU 判器的输入表共用这一份；崩溃点重放的
/// `CrashStatePlan::persisted_writes_of_state` 是同一套数字次序：两态的写数字 0 / 1 = 没持久 / 持久，三态的写 0 / 1 / 2 = 没持久 / 撕裂 / 持久；
/// 甲二段有单元写时段内序号的最低一位（除以 2 的余数）是「单元写全落」、其余拆到原地写上；撕裂那一态记成原写没持久、撕裂镜像持久，
/// 撕裂镜像之后的重放跟着被重放的那次写落不落）。
///
/// # Panics
/// 序号越过状态总数；`segment_expansions` 与 `segments` 不一样长；序号所在那一段的拆法展开出的状态数与它的序号区间对不上（拆法给错了）。
#[must_use]
#[allow(
    clippy::too_many_arguments,
    reason = "拆一个序号要的几样各自独立：两张写表的长度、哪几次写取三态、撕裂镜像与重放、段、段的拆法、状态总数、序号"
)]
pub fn persisted_writes_of_state_by_the_segment_expansions(
    recorded_write_count: usize,
    enumerated_write_count: usize,
    is_tearable: &[bool],
    torn_images: &[TornImageFact],
    segments: &[SegmentFact],
    segment_expansions: &[SegmentExpansionFact],
    state_count: u64,
    ordinal: u64,
) -> Vec<bool> {
    assert!(
        ordinal < state_count,
        "序号 {ordinal} 越过状态总数 {state_count}"
    );
    assert_eq!(segment_expansions.len(), segments.len(), "每一段一个拆法");
    let index_of = |write: &u32| usize::try_from(*write).expect("下标装得进 usize");
    // 0 没持久、1 撕裂、2 持久
    let mut landings = vec![0u8; recorded_write_count];
    let segment_of_state = segments.partition_point(|segment| segment.states.end <= ordinal);
    for segment in &segments[..segment_of_state] {
        for write in &segment.writes {
            landings[index_of(write)] = 2;
        }
    }
    let sliced_segment = segments.get(segment_of_state).and_then(|segment| {
        segment
            .slice_start
            .as_ref()
            .map(|slice_start| (segment, slice_start))
    });
    if let Some((segment, slice_start)) = sliced_segment {
        // 分片的段：片起点拆成的各位加上片内序号、逐位进位（GPU 两个内核同一个算法），再照每一位落
        let expansion = &segment_expansions[segment_of_state];
        let radices = expansion.digit_radices(segment, is_tearable);
        let mut digits = slice_start.digits.clone();
        assert!(
            digits.len() == radices.len()
                && add_to_the_mixed_radix_digits(
                    &mut digits,
                    &radices,
                    ordinal - segment.states.start
                )
                && digits
                    .iter()
                    .zip(&radices)
                    .any(|(digit, radix)| *digit + 1 != *radix),
            "段 {segment_of_state} 的片起点（{}）加片内第 {} 个越过了它按 {expansion:?} 展开的域",
            slice_start.start_within_the_domain.decimal_text(),
            ordinal - segment.states.start
        );
        let (digit_writes, copy_on_write_group) =
            expansion.digit_writes_and_the_copy_on_write_group(segment);
        let mut digits_from_the_lowest = digits.iter();
        if !copy_on_write_group.is_empty() && digits_from_the_lowest.next() == Some(&1) {
            for write in copy_on_write_group {
                landings[index_of(write)] = 2;
            }
        }
        for (write, digit) in digit_writes.iter().zip(digits_from_the_lowest) {
            let index = index_of(write);
            let choices = if is_tearable[index] { 3 } else { 2 };
            landings[index] = match (choices, *digit) {
                (2, 0) | (3, 0) => 0,
                (2, 1) | (3, 2) => 2,
                (3, 1) => 1,
                _ => unreachable!("数字不超过态数"),
            };
        }
    } else if let Some(segment) = segments.get(segment_of_state) {
        let expansion = &segment_expansions[segment_of_state];
        assert_eq!(
            expansion.state_count(segment, is_tearable),
            Some(segment.states.end - segment.states.start),
            "段 {segment_of_state} 的拆法（{expansion:?}）展开出的状态数与它的序号区间 {:?} 对不上",
            segment.states
        );
        let mut remaining = ordinal - segment.states.start;
        let (digit_writes, copy_on_write_group) =
            expansion.digit_writes_and_the_copy_on_write_group(segment);
        if !copy_on_write_group.is_empty() {
            if remaining % 2 == 1 {
                for write in copy_on_write_group {
                    landings[index_of(write)] = 2;
                }
            }
            remaining /= 2;
        }
        for write in digit_writes {
            let index = index_of(write);
            let choices = if is_tearable[index] { 3 } else { 2 };
            let digit = remaining % choices;
            remaining /= choices;
            landings[index] = match (choices, digit) {
                (2, 0) | (3, 0) => 0,
                (2, 1) | (3, 2) => 2,
                (3, 1) => 1,
                _ => unreachable!("数字不超过态数"),
            };
        }
        assert_eq!(remaining, 0, "段内序号超出这几次写的组合数");
    }
    let mut persisted = vec![false; enumerated_write_count];
    for (index, landing) in landings.iter().enumerate() {
        persisted[index] = *landing == 2;
    }
    for torn in torn_images {
        if landings[torn.write_index as usize] == 1 {
            persisted[torn.torn_image_index as usize] = true;
            for (replay_index, replayed) in &torn.replays {
                persisted[*replay_index as usize] = landings[*replayed as usize] == 2;
            }
        }
    }
    persisted
}

struct Cursor<'bytes> {
    bytes: &'bytes [u8],
    at: usize,
}

impl Cursor<'_> {
    fn take(&mut self, length: usize) -> Option<&[u8]> {
        let slice = self.bytes.get(self.at..self.at + length)?;
        self.at += length;
        Some(slice)
    }
    fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|slice| slice[0])
    }
    fn u32(&mut self) -> Option<u32> {
        self.take(4)
            .map(|slice| u32::from_le_bytes(slice.try_into().expect("4 字节")))
    }
    fn u64(&mut self) -> Option<u64> {
        self.take(8)
            .map(|slice| u64::from_le_bytes(slice.try_into().expect("8 字节")))
    }
}

// ============================== 第 ② 段的 CPU 参照：只读事实判一个状态 ==============================

/// 在这个状态里，(盘, 槽) 上现在是哪个单元：持久的单元写里来源号最大的那一次；一次都没有就是基线里的（有的话）。
fn current_unit_at<'facts>(
    facts: &'facts FlowFacts,
    persisted: &[bool],
    device: u32,
    slot: u64,
) -> Option<&'facts UnitFact> {
    let rank = |unit: &UnitFact| {
        if unit.source == BASE_IMAGE_SOURCE {
            -1i64
        } else {
            i64::from(unit.source)
        }
    };
    let mut best: Option<&UnitFact> = None;
    for unit in &facts.units {
        if unit.device != device || unit.slot != slot {
            continue;
        }
        let present = unit.source == BASE_IMAGE_SOURCE || persisted[unit.source as usize];
        if !present {
            continue;
        }
        if best.is_none_or(|current| rank(unit) > rank(current)) {
            best = Some(unit);
        }
    }
    best
}

/// 只读事实判一个状态，交回 [`verdict_bits`] 的位。GPU 内核（`crash_verify_gpu`）按同一张表、同一组判据算，两份逐状态相等。
#[must_use]
pub(crate) fn verify_state_from_facts(facts: &FlowFacts, ordinal: u64) -> u32 {
    let persisted = facts.persisted_writes_of_state(ordinal);
    verify_persisted_set_from_facts(facts, &persisted)
}

/// [`verify_state_from_facts`] 拿到掩码之后的那一半（测试里直接喂掩码用）。
#[must_use]
pub(crate) fn verify_persisted_set_from_facts(facts: &FlowFacts, persisted: &[bool]) -> u32 {
    let mut bits = 0u32;
    // 最新持久的根：写表里持久的根槽写取来源号最大的；一条都没有就取基线里最新的
    let mut newest: Option<&RootFact> = None;
    for root in &facts.roots {
        if root.source == BASE_IMAGE_SOURCE || !persisted[root.source as usize] {
            continue;
        }
        if newest.is_none_or(|current| root.source > current.source) {
            newest = Some(root);
        }
    }
    if newest.is_none() {
        newest = facts
            .newest_base_root
            .and_then(|index| facts.roots.get(index as usize));
    }
    let Some(root) = newest else {
        return bits | verdict_bits::NO_ROOT_AT_ALL;
    };
    if !root.self_checksum_holds {
        return bits | verdict_bits::NEWEST_ROOT_SELF_CHECKSUM;
    }
    for pointer in &root.pointers {
        if pointer.all_zero {
            continue;
        }
        let target = pointer.locations.iter().find_map(|location| {
            current_unit_at(facts, persisted, location.device, location.slot)
                .filter(|unit| unit.whole_crc32c == location.checksum)
        });
        match target {
            None => bits |= verdict_bits::NEWEST_ROOT_POINTER_TARGET,
            Some(unit) if !unit.checksums_hold => bits |= verdict_bits::UNIT_CHECKSUMS,
            Some(_) => {}
        }
    }
    bits
}

/// 一批状态的核对位（第 ② 段按块调），按序号次序。
#[must_use]
pub fn verify_states_from_facts(facts: &FlowFactsReadBack, ordinals: Range<u64>) -> Vec<u32> {
    ordinals
        .map(|ordinal| verify_state_from_facts(facts, ordinal))
        .collect()
}

/// 同 [`verify_states_from_facts`]，每一段照 `segment_expansions` 那一段的拆法拆序号（有甲二段的流；GPU 那一份是
/// `crate::crash_verify_gpu::GpuFactsVerifier::new_by_the_segment_expansions`）。
///
/// # Panics
/// 同 [`persisted_writes_of_state_by_the_segment_expansions`]。
#[must_use]
pub fn verify_states_from_facts_by_the_segment_expansions(
    facts: &FlowFactsReadBack,
    segment_expansions: &[SegmentExpansionFact],
    ordinals: Range<u64>,
) -> Vec<u32> {
    if let Err(segment_index) = check_the_slice_starts_against_the_expansions(
        &facts.segments,
        segment_expansions,
        &facts.is_tearable,
    ) {
        panic!("事实表第 {segment_index} 段的片起点与它的各位对不上，或片越过了段的域：事实表坏了，重跑录入段");
    }
    ordinals
        .map(|ordinal| {
            verify_persisted_set_from_facts(
                facts,
                &facts.persisted_writes_of_state_by_the_segment_expansions(
                    segment_expansions,
                    ordinal,
                ),
            )
        })
        .collect()
}

/// （盘，槽）→ 落在它上面的单元来源，测试与统计用。
#[must_use]
pub fn units_by_placement(facts: &FlowFacts) -> BTreeMap<(u32, u64), Vec<u32>> {
    let mut by_placement: BTreeMap<(u32, u64), Vec<u32>> = BTreeMap::new();
    for unit in &facts.units {
        by_placement
            .entry((unit.device, unit.slot))
            .or_default()
            .push(unit.source);
    }
    by_placement
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pointer_to(device: u32, slot: u64, checksum: u32) -> PointerFact {
        PointerFact {
            all_zero: false,
            locations: [
                LocationFact {
                    device,
                    slot,
                    checksum,
                },
                LocationFact {
                    device: device ^ 1,
                    slot,
                    checksum,
                },
            ],
        }
    }

    #[test]
    fn facts_round_trip_through_bytes() {
        let facts = FlowFacts {
            recorded_write_count: 3,
            enumerated_write_count: 4,
            is_tearable: vec![false, true, false],
            torn_images: vec![TornImageFact {
                write_index: 1,
                torn_image_index: 3,
                replays: vec![(3, 2)],
            }],
            segments: vec![
                SegmentFact {
                    writes: vec![0, 1],
                    states: 0..5,
                    slice_start: None,
                },
                SegmentFact {
                    writes: vec![2],
                    states: 5..6,
                    slice_start: None,
                },
            ],
            state_count: 7,
            units: vec![UnitFact {
                source: 0,
                device: 1,
                slot: 50176,
                unit_bytes: 32768,
                class: 1,
                checksums_hold: true,
                whole_crc32c: 0xdead_beef,
            }],
            roots: vec![RootFact {
                source: 2,
                instance: 1,
                checkpoint_txg: 3,
                rollback_floor: 0,
                self_checksum_holds: true,
                pointers: [pointer_to(1, 50176, 0xdead_beef); 4],
            }],
            newest_base_root: None,
        };
        let bytes = facts.to_bytes();
        assert_eq!(FlowFacts::from_bytes(&bytes), Some(facts.clone()));
        assert_eq!(
            FlowFacts::from_bytes(&bytes[..bytes.len() - 1]),
            None,
            "少一个字节认不回"
        );
        assert_ne!(facts.digest(), [0u8; DIGEST_BYTES]);
    }

    #[test]
    fn the_state_ordinal_decodes_to_the_same_persisted_set_as_crash_point_replay_mixed_radix() {
        // 段 0：写 0（两态）、写 1（三态）：序号 r 拆成 写0 取 r % 2、写1 取 (r / 2) % 3；共 6 个组合、去掉全持久那一个 → 5 个状态
        let facts = FlowFacts {
            recorded_write_count: 3,
            enumerated_write_count: 4,
            is_tearable: vec![false, true, false],
            torn_images: vec![TornImageFact {
                write_index: 1,
                torn_image_index: 3,
                replays: vec![],
            }],
            segments: vec![
                SegmentFact {
                    writes: vec![0, 1],
                    states: 0..5,
                    slice_start: None,
                },
                SegmentFact {
                    writes: vec![2],
                    states: 5..6,
                    slice_start: None,
                },
            ],
            state_count: 7,
            units: vec![],
            roots: vec![],
            newest_base_root: None,
        };
        assert_eq!(
            facts.persisted_writes_of_state(0),
            vec![false, false, false, false]
        );
        assert_eq!(
            facts.persisted_writes_of_state(1),
            vec![true, false, false, false]
        );
        assert_eq!(
            facts.persisted_writes_of_state(2),
            vec![false, false, false, true],
            "写 1 取撕裂：原写没持久、撕裂镜像（枚举表下标 3）持久"
        );
        assert_eq!(
            facts.persisted_writes_of_state(4),
            vec![false, true, false, false]
        );
        assert_eq!(
            facts.persisted_writes_of_state(5),
            vec![true, true, false, false],
            "段 1 的第一个状态：段 0 整段持久、写 2 没持久"
        );
        assert_eq!(
            facts.persisted_writes_of_state(6),
            vec![true, true, true, false],
            "最后全部持久"
        );
    }

    /// 一段 5 次写（两次原地写 0、1，三次单元写 2、3、4；判决 m3-crash-enumeration-merge-r2 第二节 L6-2 攻方模型那两种段序）按甲二拆：
    /// 序号 r 的最低一位是「单元写全落」，r / 2 按原地写的段内次序拆成两位，整段全落那一个取不到——7 个状态逐个照这个次序写出来比。
    fn facts_of_one_segment_in_the_order(segment_writes: Vec<u32>, states: u64) -> FlowFacts {
        FlowFacts {
            recorded_write_count: 5,
            enumerated_write_count: 5,
            is_tearable: vec![false; 5],
            torn_images: vec![],
            segments: vec![SegmentFact {
                writes: segment_writes,
                states: 0..states,
                slice_start: None,
            }],
            state_count: states + 1,
            units: vec![],
            roots: vec![],
            newest_base_root: None,
        }
    }

    fn in_place_subsets_with_copy_on_write_none_or_all_of_the_model() -> Vec<SegmentExpansionFact> {
        vec![
            SegmentExpansionFact::InPlaceSubsetsWithCopyOnWriteNoneOrAll {
                in_place_writes: vec![0, 1],
                copy_on_write_writes: vec![2, 3, 4],
            },
        ]
    }

    fn masks_of_the_in_place_subsets_with_copy_on_write_none_or_all_of_the_model() -> Vec<Vec<bool>>
    {
        // (原地写 0, 原地写 1, 单元写全落)，r = 2 × (写0 + 2 × 写1) + 单元写全落，r 从 0 数到 6
        (0..7u64)
            .map(|ordinal| {
                let copy_on_write = ordinal % 2 == 1;
                let in_place = ordinal / 2;
                vec![
                    in_place % 2 == 1,
                    (in_place / 2) % 2 == 1,
                    copy_on_write,
                    copy_on_write,
                    copy_on_write,
                ]
            })
            .collect()
    }

    #[test]
    fn in_place_subsets_with_copy_on_write_none_or_all_decode_with_the_in_place_writes_first_in_the_segment(
    ) {
        let facts = facts_of_one_segment_in_the_order(vec![0, 1, 2, 3, 4], 7);
        let expansions = in_place_subsets_with_copy_on_write_none_or_all_of_the_model();
        let decoded: Vec<Vec<bool>> = (0..7)
            .map(|ordinal| {
                facts.persisted_writes_of_state_by_the_segment_expansions(&expansions, ordinal)
            })
            .collect();
        assert_eq!(
            decoded,
            masks_of_the_in_place_subsets_with_copy_on_write_none_or_all_of_the_model()
        );
        assert_eq!(
            facts.persisted_writes_of_state_by_the_segment_expansions(&expansions, 7),
            vec![true; 5],
            "最后全部持久那一个"
        );
    }

    #[test]
    fn in_place_subsets_with_copy_on_write_none_or_all_decode_the_same_with_the_copy_on_write_writes_first_in_the_segment(
    ) {
        let facts = facts_of_one_segment_in_the_order(vec![2, 3, 4, 0, 1], 7);
        let expansions = in_place_subsets_with_copy_on_write_none_or_all_of_the_model();
        let decoded: Vec<Vec<bool>> = (0..7)
            .map(|ordinal| {
                facts.persisted_writes_of_state_by_the_segment_expansions(&expansions, ordinal)
            })
            .collect();
        assert_eq!(
            decoded,
            masks_of_the_in_place_subsets_with_copy_on_write_none_or_all_of_the_model(),
            "单元写排在段首也照原地写的段内次序拆，单元写只取全不落或全落"
        );
    }

    /// 段的拆法展开出的状态数与段的序号区间对不上（拆法给错了）就 panic，不悄悄拆成别的状态。
    #[test]
    #[should_panic(expected = "展开出的状态数与它的序号区间")]
    fn an_expansion_that_disagrees_with_the_state_range_of_its_segment_panics() {
        let facts = facts_of_one_segment_in_the_order(vec![0, 1, 2, 3, 4], 7);
        let _decoded = facts.persisted_writes_of_state_by_the_segment_expansions(
            &[SegmentExpansionFact::EveryProperSubset],
            3,
        );
    }

    /// 33 次两态写的一段（判决 m3-crash-enumeration-merge-r2 第二节 L1-7，攻方 decode_alias.py 那一形）：第 2³² + 5 个状态拆出写 0、写 2、写 32 持久，
    /// 不被截成低 32 位的 5。
    #[test]
    fn the_ordinal_within_a_thirty_three_write_segment_decodes_past_thirty_two_bits() {
        let writes = 33u32;
        let states = (1u64 << writes) - 1;
        let facts = FlowFacts {
            recorded_write_count: writes,
            enumerated_write_count: writes,
            is_tearable: vec![false; 33],
            torn_images: vec![],
            segments: vec![SegmentFact {
                writes: (0..writes).collect(),
                states: 0..states,
                slice_start: None,
            }],
            state_count: states + 1,
            units: vec![],
            roots: vec![],
            newest_base_root: None,
        };
        let persisted = facts.persisted_writes_of_state((1u64 << 32) + 5);
        let persisted_writes: Vec<usize> = persisted
            .iter()
            .enumerate()
            .filter(|(_, is_persisted)| **is_persisted)
            .map(|(write_index, _)| write_index)
            .collect();
        assert_eq!(persisted_writes, vec![0, 2, 32]);
    }

    /// 段的拆法从写的种类分组：单元写是 COW 那一组，别的都是原地写；按段内次序，与 crash.rs 甲二那一份分法相同。
    #[test]
    fn the_segment_expansion_fact_of_in_place_subsets_groups_the_unit_writes_as_copy_on_write() {
        use singlefs_harness::memory_pool::WrittenContents;
        let write_of = |kind: StepKind| RetainedWrite {
            device: singlefs_core::address::DeviceIdentity(0),
            kind,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(0),
            contents: WrittenContents::Bytes(vec![0u8; 512]),
        };
        let writes = vec![
            write_of(StepKind::UnitWrite),
            write_of(StepKind::JournalRecord),
            write_of(StepKind::UnitWrite),
            write_of(StepKind::RootRecordFua),
        ];
        assert_eq!(
            SegmentExpansionFact::of(
                CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll,
                &writes,
                &[0, 1, 2, 3]
            ),
            SegmentExpansionFact::InPlaceSubsetsWithCopyOnWriteNoneOrAll {
                in_place_writes: vec![1, 3],
                copy_on_write_writes: vec![0, 2],
            }
        );
        assert_eq!(
            SegmentExpansionFact::of(
                CrashPointReplaySegmentExpansion::NotExpanded,
                &writes,
                &[0, 1]
            ),
            SegmentExpansionFact::NotExpanded
        );
    }

    #[test]
    fn a_root_whose_pointer_targets_a_missing_unit_is_red_and_becomes_green_once_the_unit_lands() {
        let unit = UnitFact {
            source: 0,
            device: 0,
            slot: 7,
            unit_bytes: 16384,
            class: 2,
            checksums_hold: true,
            whole_crc32c: 42,
        };
        let root = RootFact {
            source: 1,
            instance: 1,
            checkpoint_txg: 1,
            rollback_floor: 0,
            self_checksum_holds: true,
            pointers: [
                pointer_to(0, 7, 42),
                zero_pointer(),
                zero_pointer(),
                zero_pointer(),
            ],
        };
        let facts = FlowFacts {
            recorded_write_count: 2,
            enumerated_write_count: 2,
            is_tearable: vec![false, false],
            torn_images: vec![],
            segments: vec![SegmentFact {
                writes: vec![0, 1],
                states: 0..3,
                slice_start: None,
            }],
            state_count: 4,
            units: vec![unit],
            roots: vec![root],
            newest_base_root: None,
        };
        assert_eq!(
            verify_persisted_set_from_facts(&facts, &[false, true]),
            verdict_bits::NEWEST_ROOT_POINTER_TARGET,
            "根落了、它指的单元没落"
        );
        assert_eq!(verify_persisted_set_from_facts(&facts, &[true, true]), 0);
        assert_eq!(
            verify_persisted_set_from_facts(&facts, &[true, false]),
            verdict_bits::NO_ROOT_AT_ALL,
            "没有根、基线里也没有"
        );
        let mut bad_unit = facts.clone();
        bad_unit.units[0].checksums_hold = false;
        assert_eq!(
            verify_persisted_set_from_facts(&bad_unit, &[true, true]),
            verdict_bits::UNIT_CHECKSUMS
        );
        let mut mismatch = facts.clone();
        mismatch.units[0].whole_crc32c = 43;
        assert_eq!(
            verify_persisted_set_from_facts(&mismatch, &[true, true]),
            verdict_bits::NEWEST_ROOT_POINTER_TARGET,
            "指针里的校验和与单元的整单元 CRC 对不上"
        );
    }

    /// 一段 70 次两态的原地写（3^70、2^70 都装不进 u64 的那一形）、只判它域里从 2^64 + 7 起的 300 个态：片内第 i 个拆出的持久集合
    /// 就是 2^64 + 7 + i 的二进制（第 0 位是第一次写），片起点进事实表的字节往返不丢、片起点那几位与编号对得上。
    #[test]
    fn a_sliced_segment_decodes_from_its_slice_start_plus_the_ordinal_within_the_slice() {
        const WRITES: u32 = 70;
        const SLICE_STATES: u64 = 300;
        let slice_start_value: u128 = (1u128 << 64) + 7;
        let mut start_within_the_domain = StateCount1280Bits::from_u64(u64::MAX);
        start_within_the_domain = start_within_the_domain
            .checked_add_u64(8)
            .expect("2^64 + 7 装得进 1280 位");
        let expansions = vec![
            SegmentExpansionFact::InPlaceSubsetsWithCopyOnWriteNoneOrAll {
                in_place_writes: (0..WRITES).collect(),
                copy_on_write_writes: vec![],
            },
        ];
        let mut facts = FlowFacts {
            recorded_write_count: WRITES,
            enumerated_write_count: WRITES,
            is_tearable: vec![false; 70],
            torn_images: vec![],
            segments: vec![SegmentFact {
                writes: (0..WRITES).collect(),
                states: 0..SLICE_STATES,
                slice_start: None,
            }],
            state_count: SLICE_STATES + 1,
            units: vec![],
            roots: vec![],
            newest_base_root: None,
        };
        let radices = expansions[0].digit_radices(&facts.segments[0], &facts.is_tearable);
        assert_eq!(radices, vec![2u8; 70]);
        facts.segments[0].slice_start =
            Some(SegmentSliceStartFact::of(start_within_the_domain, &radices));
        assert_eq!(
            check_the_slice_starts_against_the_expansions(
                &facts.segments,
                &expansions,
                &facts.is_tearable
            ),
            Ok(())
        );
        assert_eq!(
            FlowFacts::from_bytes(&facts.to_bytes()),
            Some(facts.clone()),
            "片起点往返事实表的字节不丢"
        );
        for ordinal_within_the_slice in [0u64, 1, 2, 255, SLICE_STATES - 1] {
            let number = slice_start_value + u128::from(ordinal_within_the_slice);
            let expected: Vec<bool> = (0..70).map(|bit| (number >> bit) & 1 == 1).collect();
            assert_eq!(
                facts.persisted_writes_of_state_by_the_segment_expansions(
                    &expansions,
                    ordinal_within_the_slice
                ),
                expected,
                "片内第 {ordinal_within_the_slice} 个是段内编号 2^64 + 7 + {ordinal_within_the_slice}"
            );
        }
    }

    /// 甲二一段（两次原地写、三次单元写，7 个态）从段内编号 3 起分片：片内第 i 个与不分片时编号 3 + i 拆出的逐个相同。
    #[test]
    fn a_slice_of_in_place_subsets_with_copy_on_write_none_or_all_decodes_like_the_whole_segment() {
        let whole = facts_of_one_segment_in_the_order(vec![0, 1, 2, 3, 4], 7);
        let expansions = in_place_subsets_with_copy_on_write_none_or_all_of_the_model();
        let mut sliced = facts_of_one_segment_in_the_order(vec![0, 1, 2, 3, 4], 4);
        let radices = expansions[0].digit_radices(&sliced.segments[0], &sliced.is_tearable);
        assert_eq!(radices, vec![2, 2, 2], "单元写那一组一位、两次原地写各一位");
        sliced.segments[0].slice_start = Some(SegmentSliceStartFact::of(
            StateCount1280Bits::from_u64(3),
            &radices,
        ));
        for ordinal_within_the_slice in 0..4 {
            assert_eq!(
                sliced.persisted_writes_of_state_by_the_segment_expansions(
                    &expansions,
                    ordinal_within_the_slice
                ),
                whole.persisted_writes_of_state_by_the_segment_expansions(
                    &expansions,
                    3 + ordinal_within_the_slice
                ),
                "片内第 {ordinal_within_the_slice} 个"
            );
        }
    }

    /// 片起点的各位与编号对不上、片越过段的域：核片起点那一步交回那一段的段号。
    #[test]
    fn a_slice_start_whose_digits_disagree_or_whose_slice_passes_the_domain_is_found() {
        let expansions = in_place_subsets_with_copy_on_write_none_or_all_of_the_model();
        let mut facts = facts_of_one_segment_in_the_order(vec![0, 1, 2, 3, 4], 4);
        facts.segments[0].slice_start = Some(SegmentSliceStartFact {
            start_within_the_domain: StateCount1280Bits::from_u64(3),
            digits: vec![0, 1, 0],
        });
        assert_eq!(
            check_the_slice_starts_against_the_expansions(
                &facts.segments,
                &expansions,
                &facts.is_tearable
            ),
            Err(0),
            "各位 [0, 1, 0] 是 2，不是 3"
        );
        facts.segments[0].slice_start = Some(SegmentSliceStartFact::of(
            StateCount1280Bits::from_u64(4),
            &[2, 2, 2],
        ));
        assert_eq!(
            check_the_slice_starts_against_the_expansions(
                &facts.segments,
                &expansions,
                &facts.is_tearable
            ),
            Err(0),
            "4 + 4 = 8 越过 7 个态的域"
        );
    }

    /// 旧布局（版本 1：没有头、段里没有片起点）的事实表字节读到就拒，拒跑句点名布局版本，不按新布局错解。
    #[test]
    fn a_facts_table_in_the_old_layout_is_refused_naming_the_layout_version() {
        let facts = FlowFacts {
            recorded_write_count: 1,
            enumerated_write_count: 1,
            is_tearable: vec![false],
            torn_images: vec![],
            segments: vec![],
            state_count: 1,
            units: vec![],
            roots: vec![],
            newest_base_root: None,
        };
        let bytes = facts.to_bytes();
        assert!(bytes.starts_with(FACTS_LAYOUT_MAGIC));
        // 一段都没有的事实表，去掉头就是版本 1 的字节
        let old_layout_bytes = &bytes[FACTS_LAYOUT_MAGIC.len()..];
        let refused = FlowFactsReadBack::from_the_library_bytes(old_layout_bytes, &facts.digest())
            .expect_err("旧布局的事实表不认");
        assert!(
            refused.contains("布局不是版本 2") && refused.contains("旧布局"),
            "{refused}"
        );
        assert!(
            FlowFactsReadBack::from_the_library_bytes(&bytes, &facts.digest()).is_ok(),
            "新布局的照读"
        );
    }

    /// 单元区里一个 64 槽对齐的整段起点（784 MiB = 50176 槽）。
    const DISCARDED_SEGMENT_START: u64 = 784 << 20;

    /// 基线：盘 0 上 `DISCARDED_SEGMENT_START` 那一槽是一个码 2 的单元（16 KiB，内容不必自证得过：这里只看整单元 CRC）。
    fn base_with_one_unit_at_the_discarded_segment() -> (MemoryPool, Vec<u8>) {
        let mut unit_bytes = vec![0x33u8; 16384];
        unit_bytes[..4].copy_from_slice(UNIT_MAGIC);
        unit_bytes[UNIT_CLASS_OFFSET] = 2;
        let mut base =
            MemoryPool::with_devices(&[singlefs_core::address::DeviceIdentity(0)], 1 << 30);
        base.devices
            .get_mut(&singlefs_core::address::DeviceIdentity(0))
            .expect("盘 0")
            .write(DeviceOffsetInBytes(DISCARDED_SEGMENT_START), &unit_bytes);
        (base, unit_bytes)
    }

    /// 罩住 `DISCARDED_SEGMENT_START` 起 64 槽的一次 discard。
    fn discard_of_the_segment() -> RetainedWrite {
        RetainedWrite {
            device: singlefs_core::address::DeviceIdentity(0),
            kind: StepKind::Discard,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(DISCARDED_SEGMENT_START),
            contents: singlefs_harness::memory_pool::WrittenContents::Zeros {
                length: 64 * SLOT_BYTES,
            },
        }
    }

    /// CPU 参照认 discard（C829（discard 与 mkfs 清零在段种类串里同名））：一次 discard 罩住最新的根指着的单元，它持久或撕裂的状态里
    /// 根指过去按整单元 CRC 对不上（与 CPU 判器在镜像上读回 0 或撕裂字节一致），没持久的状态里单元还在、判绿。
    /// 这一段 1 + (3 − 1) 个状态：没持久、撕裂（基线那一槽有旧内容，D13（验证路线） 已定项 4 的第三态）、全部持久。
    #[test]
    fn a_persisted_or_torn_discard_over_the_unit_the_newest_root_points_to_is_red_in_the_cpu_reference(
    ) {
        let (base, unit_bytes) = base_with_one_unit_at_the_discarded_segment();
        let writes = vec![discard_of_the_segment()];
        let segments = vec![vec![0]];
        let mut facts =
            extract_flow_facts(&base, &writes, &segments, &crate::crash::full_expansion);
        let slot = DISCARDED_SEGMENT_START / SLOT_BYTES;
        let base_crc = crc32_castagnoli_table(&unit_bytes);
        assert_eq!(
            facts
                .units
                .iter()
                .map(|unit| (unit.source, unit.slot, unit.whole_crc32c == base_crc))
                .collect::<Vec<_>>(),
            vec![
                (BASE_IMAGE_SOURCE, slot, true),
                (0, slot, false),
                (1, slot, false)
            ],
            "基线里的单元，discard 与它的撕裂镜像各在那一槽添一条字节变了的"
        );
        // 夹具单元不自证（`check_unit` 判它 flags 字节非零）：这里要判的只是根指过去整单元 CRC 对不对得上，
        // 基线来源的单元事实当成两道校验和都过，没持久那一态才只剩 discard 这一个变量。
        for unit in facts
            .units
            .iter_mut()
            .filter(|unit| unit.source == BASE_IMAGE_SOURCE)
        {
            unit.checksums_hold = true;
        }
        facts.roots = vec![RootFact {
            source: BASE_IMAGE_SOURCE,
            instance: 1,
            checkpoint_txg: 1,
            rollback_floor: 0,
            self_checksum_holds: true,
            pointers: [
                pointer_to(0, slot, base_crc),
                zero_pointer(),
                zero_pointer(),
                zero_pointer(),
            ],
        }];
        facts.newest_base_root = Some(0);
        assert_eq!(facts.state_count, 3);
        assert_eq!(
            (0..facts.state_count)
                .map(|ordinal| verify_state_from_facts(&facts, ordinal))
                .collect::<Vec<u32>>(),
            vec![
                0,
                verdict_bits::NEWEST_ROOT_POINTER_TARGET,
                verdict_bits::NEWEST_ROOT_POINTER_TARGET
            ],
            "没持久判绿；撕裂与持久两态里根指的单元不在了"
        );
    }

    /// 段的拆法里 discard 是原地写那一组（不是单元写那种 COW）。
    #[test]
    fn the_segment_expansion_fact_groups_a_discard_with_the_in_place_writes() {
        let unit_write = RetainedWrite {
            device: singlefs_core::address::DeviceIdentity(0),
            kind: StepKind::UnitWrite,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(DISCARDED_SEGMENT_START + (1 << 20)),
            contents: singlefs_harness::memory_pool::WrittenContents::Bytes(vec![0u8; 512]),
        };
        let writes = vec![unit_write, discard_of_the_segment()];
        assert_eq!(
            in_place_and_copy_on_write_writes_of_the_segment(&writes, &[0, 1]),
            (vec![1], vec![0])
        );
    }
}
