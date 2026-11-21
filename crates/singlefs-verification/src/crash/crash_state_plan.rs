//! 崩溃状态的计划：段的展开与切片、原地覆写的撕裂态、每次写落不落的选法、按序号取状态，与照计划排出的枚举表、读回那一路的状态。

use super::*;

/// 一条读回的流按序号段走状态要的两张表：哪几次写取三态（[`TearableInPlaceOverwrites`]）、枚举写表（录制流里的写在前，撕裂镜像与重放
/// 接在后面，[`WritesWithTornImages`]）。两张表只随流（基镜像、写表、分段）定，与展开、片、序号段都无关。
/// 建一次要把每次原地覆写罩住的范围整段读出、摆出等长的撕裂镜像：用过的盘上重做 mkfs、清零罩着旧 journal 环那一类，一份上 GB。
/// 所以一个阶段只建一份、各核对线程经 [`CrashStatesOfTheReadBackFlow`] 借用，不在每一段（每条线程）里各建一份
/// （`tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs` 按甲二展开的那一条：观察者录入段原来每条线程各建一份，八条线程同时建就撞了 16G 的内存上限）。
pub(crate) struct TornImageTablesOfTheReadBackFlow {
    tearable: TearableInPlaceOverwrites,
    writes_with_torn_images: WritesWithTornImages,
}

impl TornImageTablesOfTheReadBackFlow {
    /// 按崩溃点重放的口径建：每次原地覆写取三态（[`TearableInPlaceOverwrites::of`]），撕裂镜像与重放接在录制流后面。
    ///
    /// # Panics
    /// 同 [`TearableInPlaceOverwrites::of`]。
    #[must_use]
    pub(crate) fn of_the_read_back_flow(
        base: &MemoryPool,
        writes: &[RetainedWrite],
        segments: &[Vec<usize>],
    ) -> Self {
        let tearable = TearableInPlaceOverwrites::of(base, writes);
        let writes_with_torn_images = WritesWithTornImages::of(base, writes, segments, &tearable);
        Self {
            tearable,
            writes_with_torn_images,
        }
    }

    /// 枚举写表：每个状态的镜像都叠在这一份上（[`CrashImage::writes`]）。
    #[must_use]
    pub(crate) fn enumerated_writes(&self) -> &[RetainedWrite] {
        &self.writes_with_torn_images.writes
    }

    /// 按每段的展开与片排好状态的次序，交回能按序号段走状态的那一份；它只借这两张表，几条线程可以同时借用同一份。
    /// `writes`、`segments` 与建表时给的相同。`expansion` 只在这里逐段问一次，所以它不必能跨线程。
    ///
    /// # Panics
    /// 片越过了段的域（`CrashStatePlan::new`）。
    #[must_use]
    pub(crate) fn crash_states_planned_by<'tables>(
        &'tables self,
        base: &'tables MemoryPool,
        writes: &[RetainedWrite],
        segments: &'tables [Vec<usize>],
        expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
        segment_slices: &[Option<CrashPointReplaySegmentSlice>],
    ) -> CrashStatesOfTheReadBackFlow<'tables> {
        CrashStatesOfTheReadBackFlow {
            base,
            plan: CrashStatePlan::new(
                writes,
                segments,
                expansion,
                segment_slices,
                &self.tearable,
                &self.writes_with_torn_images,
            ),
        }
    }
}

/// 一条读回的流排好次序的全部状态（[`TornImageTablesOfTheReadBackFlow::crash_states_planned_by`]）：按序号段逐个交镜像或判。
/// 只读，几条核对线程同时借同一份、各走各的序号段（崩溃放量判段的 CPU 那一路、观察者录入段）。
pub(crate) struct CrashStatesOfTheReadBackFlow<'tables> {
    base: &'tables MemoryPool,
    plan: CrashStatePlan<'tables>,
}

impl CrashStatesOfTheReadBackFlow<'_> {
    /// 逐个交 `ordinals` 里每个状态的镜像（不判），交回整条流的状态数。
    ///
    /// # Panics
    /// `ordinals` 越过整条流的状态数。
    #[must_use]
    pub(crate) fn visit_the_states_in_the_ordinals(
        &self,
        ordinals: Range<u64>,
        visit_state: &mut dyn FnMut(u64, CrashPointReplayPublishOfState, &CrashImage<'_>),
    ) -> u64 {
        let plan = &self.plan;
        assert!(
            ordinals.end <= plan.state_count,
            "序号段 {ordinals:?} 越过整条流的 {} 个状态",
            plan.state_count
        );
        let enumerated_writes = plan.writes_with_torn_images.writes.as_slice();
        for ordinal in ordinals {
            let image = CrashImage {
                base: self.base,
                writes: enumerated_writes,
                persisted: plan.persisted_writes_of_state(ordinal),
            };
            visit_state(ordinal, plan.publish_of_state(ordinal), &image);
        }
        plan.state_count
    }
}

/// 每一段展开出来的状态序号区间（与 `segments` 同序，不展开的段是空区间），与整条流的状态数（最后那个全持久状态的序号是它减一）。
/// 崩溃放量流水线按它把状态分给崩溃点：一个点管它起头的那几段。
#[must_use]
pub fn crash_state_ranges_by_segment(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> (Vec<Range<u64>>, u64) {
    crash_state_ranges_by_segment_with_slices(
        base,
        writes,
        segments,
        expansion,
        &no_segment_slices(segments.len()),
    )
}

/// 同 [`crash_state_ranges_by_segment`]，分片的段的序号区间是它的片长（[`CrashPointReplaySegmentSlice`]）。
///
/// # Panics
/// 片越过了段的域（`CrashStatePlan::new`）。
#[must_use]
pub fn crash_state_ranges_by_segment_with_slices(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
) -> (Vec<Range<u64>>, u64) {
    let tearable = TearableInPlaceOverwrites::of(base, writes);
    let writes_with_torn_images = WritesWithTornImages::of(base, writes, segments, &tearable);
    let plan = CrashStatePlan::new(
        writes,
        segments,
        expansion,
        segment_slices,
        &tearable,
        &writes_with_torn_images,
    );
    (plan.state_ranges_by_segment.clone(), plan.state_count)
}

/// 崩溃点重放枚举用的几张表，给崩溃放量第 ① 段抽事实用（`crate::crash_facts`）：枚举写表（录制流里的写在前、撕裂镜像与重放接在后面）、
/// 哪几次写取三态、每次原地覆写的撕裂镜像与重放接在枚举写表的哪里、每段展开出来的状态序号区间、状态总数。
/// 与 [`judge_crash_state_range`] 按同一套 `CrashStatePlan` 算：事实那边按这几张表现算的掩码与这里的 `persisted_writes_of_state` 逐位相同。
/// 一次原地覆写的撕裂镜像接在枚举写表的哪里：写表下标、撕裂镜像在枚举写表里的下标、（重放在枚举写表里的下标，重放的是写表里哪一次写）。
pub struct TornImageTableRow {
    pub write_index: usize,
    pub torn_image_index: usize,
    pub replays: Vec<(usize, usize)>,
}

pub struct CrashStateEnumerationTables {
    pub enumerated_writes: Vec<RetainedWrite>,
    pub is_tearable_by_write: Vec<bool>,
    pub torn_images: Vec<TornImageTableRow>,
    pub state_ranges_by_segment: Vec<Range<u64>>,
    /// 每一段这一趟的片（与段同序，不分片的段是 `None`）：分片的段在 `state_ranges_by_segment` 里的区间是片长。
    pub segment_slices: Vec<Option<CrashPointReplaySegmentSlice>>,
    pub state_count: u64,
}

#[must_use]
pub fn crash_state_enumeration_tables(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> CrashStateEnumerationTables {
    crash_state_enumeration_tables_with_slices(
        base,
        writes,
        segments,
        expansion,
        &no_segment_slices(segments.len()),
    )
}

/// 同 [`crash_state_enumeration_tables`]，分片的段照 `segment_slices`。
///
/// # Panics
/// 片越过了段的域（`CrashStatePlan::new`）。
#[must_use]
pub fn crash_state_enumeration_tables_with_slices(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
) -> CrashStateEnumerationTables {
    crash_state_enumeration_tables_with_slices_choosing_the_torn_images(
        base,
        writes,
        segments,
        expansion,
        segment_slices,
        TornImagesOfTheInPlaceOverwrites::Appended,
    )
}

/// 枚举写表末尾接不接原地覆写的撕裂镜像（连同它们的重放）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TornImagesOfTheInPlaceOverwrites {
    /// 崩溃点重放的口径：每次原地覆写（[`TearableInPlaceOverwrites::of`]）取三态，撕裂镜像与重放接在录制流后面。
    Appended,
    /// 每次写只取两态（[`TearableInPlaceOverwrites::none`]），枚举写表就是录制流：给定状态表的持久掩码从不点名撕裂态时用
    /// （随机历史每步的镜像全是「前缀全落、之后一个都没落」）。接了也是从不落的写，只白占掩码的位与它罩住的落点的写数。
    LeftOut,
}

/// 同 [`crash_state_enumeration_tables_with_slices`]，撕裂镜像接不接照 `torn_images`。
///
/// # Panics
/// 同 [`crash_state_enumeration_tables_with_slices`]。
#[must_use]
pub fn crash_state_enumeration_tables_with_slices_choosing_the_torn_images(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
    torn_images: TornImagesOfTheInPlaceOverwrites,
) -> CrashStateEnumerationTables {
    let tearable = match torn_images {
        TornImagesOfTheInPlaceOverwrites::Appended => TearableInPlaceOverwrites::of(base, writes),
        TornImagesOfTheInPlaceOverwrites::LeftOut => TearableInPlaceOverwrites::none(writes.len()),
    };
    let writes_with_torn_images = WritesWithTornImages::of(base, writes, segments, &tearable);
    let plan = CrashStatePlan::new(
        writes,
        segments,
        expansion,
        segment_slices,
        &tearable,
        &writes_with_torn_images,
    );
    CrashStateEnumerationTables {
        enumerated_writes: writes_with_torn_images.writes.clone(),
        is_tearable_by_write: tearable.is_tearable_by_write.clone(),
        torn_images: writes_with_torn_images
            .torn_image_by_write
            .iter()
            .map(|(write_index, torn)| TornImageTableRow {
                write_index: *write_index,
                torn_image_index: torn.torn_image_index,
                replays: torn.replays.clone(),
            })
            .collect(),
        state_ranges_by_segment: plan.state_ranges_by_segment.clone(),
        segment_slices: segment_slices.to_vec(),
        state_count: plan.state_count,
    }
}

/// 整条流在这种展开下的状态数，与 [`judge_crash_state_range`] 交回的相同（不判任何状态）。
#[must_use]
pub fn crash_point_replay_plan_state_count(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> u64 {
    let tearable = TearableInPlaceOverwrites::of(base, writes);
    let writes_with_torn_images = WritesWithTornImages::of(base, writes, segments, &tearable);
    CrashStatePlan::new(
        writes,
        segments,
        expansion,
        &no_segment_slices(segments.len()),
        &tearable,
        &writes_with_torn_images,
    )
    .state_count
}

/// 一段在这一趟里只判它的域（按它的展开方式展开出的全部状态，按混合进位编号、0 起；编号的次序见 [`CrashStatePlan::persisted_writes_of_state`]）
/// 里编号 [`Self::start_within_the_domain`, + `Self::state_count`) 那一截：按段登记的段整域装不进 u64 时分片枚举（C790），
/// 这一段在流里的序号区间就是这一截那么长，段内编号 = 片起点 + （序号 − 段在流里的起点）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrashPointReplaySegmentSlice {
    pub start_within_the_domain: StateCount1280Bits,
    pub state_count: u64,
}

/// 每一段都不分片（与段同序）。
#[must_use]
pub fn no_segment_slices(segment_count: usize) -> Vec<Option<CrashPointReplaySegmentSlice>> {
    vec![None; segment_count]
}

/// 一段按它的展开方式拆编号时混合进位的一位：第 0 位最低。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrashPointReplayDigitOfASegment {
    /// 甲二段里单元写那一组整组落不落（两进制，1 是全落）；只有单元写不空的甲二段有，排在最低位。
    CopyOnWriteGroup,
    /// 写表下标那一次写的落法（它能取几态就是几进制，[`WriteLandingChoices::landing_of_digit`]）。
    LandingOfAWrite(usize),
}

/// 分片的段拆编号要的：混合进位的每一位是什么、几进制，片起点拆成的各位。
#[derive(Clone, Debug, PartialEq, Eq)]
struct DigitsOfASlicedSegment {
    positions: Vec<CrashPointReplayDigitOfASegment>,
    radices: Vec<u8>,
    slice_start_digits: Vec<u8>,
}

/// 一段怎么展开成崩溃状态（前面的段全持久、这一段落一部分、后面的段全不落）。
/// 段里每次写各取几态见 [`TearableInPlaceOverwrites`]：原地覆写的写取三态（没持久 / 新旧都读不出 / 持久），其余取两态。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CrashPointReplaySegmentExpansion {
    /// 不展开：只以整段持久进入后面的状态。
    NotExpanded,
    /// 段内每次写各取它的几态、任意组合，去掉整段全持久那一个（D13（验证路线） 已定项 4 的全量）：
    /// n 个写里 m 个是原地覆写时 3^m · 2^(n−m) − 1 个；m = 0 时就是任意真子集 2^n − 1 个。
    EveryProperSubset,
    /// 甲二（崩溃点重放规模三轮判完那一种展开）：段内原地写（单元写之外的写）各取它的几态、任意组合 × 单元写（COW）只取
    /// 全不落或全落，去掉整段全落那一个。原地写 k 个（其中 m 个是原地覆写）、单元写 c 个：3^m · 2^(k−m) ·（c > 0 时 2，否则 1）− 1 个，
    /// m = 0 时 c > 0 是 2^(k+1) − 1、c = 0 是 2^k − 1（与全量相同）。
    /// 它看不见「单元写只落一部分」才显出来的那一类（C561 那一形就是），替不了全量展开。
    InPlaceSubsetsWithCopyOnWriteNoneOrAll,
}

impl CrashPointReplaySegmentExpansion {
    /// 进计划哈希的名字（崩溃放量的块键要分得开两种展开方式，哪怕状态数碰巧相同）。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::NotExpanded => "not_expanded",
            Self::EveryProperSubset => "every_proper_subset",
            Self::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
                "in_place_subsets_with_copy_on_write_none_or_all"
            }
        }
    }

    /// 这一段按这种展开出几个状态；`tearable` 里的写取三态，其余取两态。
    ///
    /// # Panics
    /// 一段的状态数装不进 u64（段里的写太多：2 态的写六十几个就到了）。装不装得下要先问的调用方用 [`Self::checked_state_count`]。
    #[must_use]
    pub fn state_count(
        self,
        writes: &[RetainedWrite],
        segment: &[usize],
        tearable: &TearableInPlaceOverwrites,
    ) -> u64 {
        self.checked_state_count(writes, segment, tearable)
            .and_then(|state_count| state_count.to_u64())
            .expect("一段的状态数装得进 u64")
    }

    /// 同 [`Self::state_count`]，交回 1280 位的态数（[`StateCount1280Bits`]，装得进 u64 的照样取得出 u64）；越过 1280 位交 `None`、
    /// 不 panic（崩溃放量录入段排计划之前先用它定按段登记的段整域展不展得开，`crate::crash_amplification::states_of_the_flow_to_record`）。
    #[must_use]
    pub fn checked_state_count(
        self,
        writes: &[RetainedWrite],
        segment: &[usize],
        tearable: &TearableInPlaceOverwrites,
    ) -> Option<StateCount1280Bits> {
        match self {
            Self::NotExpanded => Some(StateCount1280Bits::ZERO),
            Self::EveryProperSubset => tearable
                .checked_landing_combinations_of(segment)?
                .checked_sub(&StateCount1280Bits::from_u64(1)),
            Self::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
                let (in_place_writes, copy_on_write_writes) =
                    in_place_and_copy_on_write_writes_of(writes, segment);
                let copy_on_write_choices: u64 = if copy_on_write_writes.is_empty() {
                    1
                } else {
                    2
                };
                tearable
                    .checked_landing_combinations_of(&in_place_writes)?
                    .checked_mul_u64(copy_on_write_choices)?
                    .checked_sub(&StateCount1280Bits::from_u64(1))
            }
        }
    }

    /// 这一段拆编号时混合进位的各位（第 0 位最低）与各位的进制：全量是段里每次写；甲二是原地写，单元写不空时最低一位是单元写那一组；
    /// 不展开的段一位都没有。编号的次序与 [`CrashStatePlan::persisted_writes_of_state`] 拆不分片的段内序号相同。
    fn digits_of_the_segment(
        self,
        segment: &[usize],
        in_place_and_copy_on_write: &(Vec<usize>, Vec<usize>),
        tearable: &TearableInPlaceOverwrites,
    ) -> Vec<(CrashPointReplayDigitOfASegment, u8)> {
        let radix_of_a_write = |write_index: usize| {
            u8::try_from(tearable.landing_choices_of(write_index).count())
                .expect("一次写至多三态，进制装得进 u8")
        };
        let landing_digits = |write_indexes: &[usize]| {
            write_indexes
                .iter()
                .map(|write_index| {
                    (
                        CrashPointReplayDigitOfASegment::LandingOfAWrite(*write_index),
                        radix_of_a_write(*write_index),
                    )
                })
                .collect::<Vec<_>>()
        };
        match self {
            Self::NotExpanded => Vec::new(),
            Self::EveryProperSubset => landing_digits(segment),
            Self::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
                let (in_place_writes, copy_on_write_writes) = in_place_and_copy_on_write;
                let mut digits = Vec::with_capacity(in_place_writes.len() + 1);
                if !copy_on_write_writes.is_empty() {
                    digits.push((CrashPointReplayDigitOfASegment::CopyOnWriteGroup, 2));
                }
                digits.extend(landing_digits(in_place_writes));
                digits
            }
        }
    }
}

/// 一次写在一个崩溃状态里怎么落（D13（验证路线） 已定项 4；第三态是代码审阅第 4 条「原地覆写补第三态、全量也跑」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteLandingInCrashState {
    /// 没持久：那个位置上是崩溃前的旧字节。
    NotPersisted,
    /// 新旧都读不出：只有原地覆写的写取这一态，镜像里那一段是 [`torn_image_of_in_place_overwrite`] 的字节。
    Torn,
    /// 整次写都持久了。
    Persisted,
}

/// 一次写在崩溃状态里能取哪几态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteLandingChoices {
    NotPersistedOrPersisted,
    NotPersistedTornOrPersisted,
}

impl WriteLandingChoices {
    fn count(self) -> u64 {
        match self {
            Self::NotPersistedOrPersisted => 2,
            Self::NotPersistedTornOrPersisted => 3,
        }
    }

    /// 混合进制里这一位上的数字是哪一态。最大的那个数字是「持久」：一段里每个写都取最大时就是整段全持久（展开时去掉的那一个，
    /// 它排在这一段序号的最后），只取两态的写上数字就是今天子集掩码的那一位（0 没持久、1 持久），次序与补第三态之前逐个相同。
    fn landing_of_digit(self, digit: u64) -> WriteLandingInCrashState {
        match self {
            Self::NotPersistedOrPersisted => match digit {
                0 => WriteLandingInCrashState::NotPersisted,
                1 => WriteLandingInCrashState::Persisted,
                out_of_range => unreachable!("两态的写上数字只有 0、1：拿到 {out_of_range}"),
            },
            Self::NotPersistedTornOrPersisted => match digit {
                0 => WriteLandingInCrashState::NotPersisted,
                1 => WriteLandingInCrashState::Torn,
                2 => WriteLandingInCrashState::Persisted,
                out_of_range => unreachable!("三态的写上数字只有 0、1、2：拿到 {out_of_range}"),
            },
        }
    }
}

/// 写表里哪几次写是原地覆写、撕裂时出第三态「新旧都读不出」（代码审阅第 4 条「原地覆写补第三态、全量也跑」；
/// 认法照 `research/prompts/m2-rev-b3a-implementer-report.md` 第四节），三条都成立才算：
/// 1. 不是单元写：单元写是 COW，落在分配器交出来的槽上，那一槽的旧内容没有谁还要（回收谓词管着），撕裂与没持久一样，照旧两态；
/// 2. 长于一个扇区：崩溃点重放的撕裂粒度是 [`SECTOR_BYTES`]，一个扇区的写撕不开（根槽写是一个物理块，照旧两态）；
/// 3. 它罩住的范围里原来有东西：基镜像那一段有非零字节，或写表里更早有一次同一块盘上、带字节（不是整段清零或 discard）的写与它重叠。
///
/// discard 照这三条认（D13（验证路线） 已定项 4 原地覆写的字面，判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S6）：
/// 它的「新字节」是 0，撕裂镜像前一半清了、后一半还是旧的；不长于一个扇区、或罩住的范围里原来全 0 的只取两态。
/// 一次 discard 在一个崩溃状态里只取这几态之一，「段内部分落地」（几个扇区落了、别的没落的任意组合）不另枚举。
///
/// 撕裂态原来并进「没持久」（D13（验证路线） 已定项 4 原句）；对原地覆写不成立——撕裂时那一处的旧内容也没了，「没持久」留着完整的旧内容。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TearableInPlaceOverwrites {
    is_tearable_by_write: Vec<bool>,
}

impl TearableInPlaceOverwrites {
    /// 按上面三条逐次认。
    ///
    /// # Panics
    /// 一次根槽写长于一个扇区又罩住了旧内容：第一版不支持根槽写的第三态（它的撕裂镜像接在枚举用的写表末尾，会被择根与记录核对器
    /// 当成又一条根槽写）。根槽写是一个物理块，崩溃点重放的池都按 512 字节的物理块建，走不到。
    #[must_use]
    pub fn of(base: &MemoryPool, writes: &[RetainedWrite]) -> Self {
        let is_tearable_by_write: Vec<bool> = (0..writes.len())
            .map(|write_index| is_tearable_in_place_overwrite(base, writes, write_index))
            .collect();
        for (write, is_tearable) in writes.iter().zip(&is_tearable_by_write) {
            assert!(
                !(*is_tearable && write.kind == StepKind::RootRecordFua),
                "根槽写（盘 {} 偏移 {}、{} 字节）长于一个扇区又罩住旧内容：第一版不支持根槽写的第三态",
                write.device.0,
                write.offset.0,
                write.length_in_bytes()
            );
        }
        Self {
            is_tearable_by_write,
        }
    }

    /// 每次写都只取两态（补第三态之前的口径）。
    #[must_use]
    pub fn none(write_count: usize) -> Self {
        Self {
            is_tearable_by_write: vec![false; write_count],
        }
    }

    /// 写表下标 `write_index` 那次写是不是原地覆写。
    #[must_use]
    pub fn contains(&self, write_index: usize) -> bool {
        self.is_tearable_by_write[write_index]
    }

    /// 原地覆写的写，按写表下标从小到大。
    #[must_use]
    pub fn write_indexes(&self) -> Vec<usize> {
        self.is_tearable_by_write
            .iter()
            .enumerate()
            .filter(|(_, is_tearable)| **is_tearable)
            .map(|(write_index, _)| write_index)
            .collect()
    }

    fn landing_choices_of(&self, write_index: usize) -> WriteLandingChoices {
        if self.is_tearable_by_write[write_index] {
            WriteLandingChoices::NotPersistedTornOrPersisted
        } else {
            WriteLandingChoices::NotPersistedOrPersisted
        }
    }

    /// 这几次写各取它的几态、全部组合的个数；越过 1280 位交回 `None`。
    fn checked_landing_combinations_of(
        &self,
        write_indexes: &[usize],
    ) -> Option<StateCount1280Bits> {
        write_indexes.iter().try_fold(
            StateCount1280Bits::from_u64(1),
            |combinations, write_index| {
                combinations.checked_mul_u64(self.landing_choices_of(*write_index).count())
            },
        )
    }
}

/// [`TearableInPlaceOverwrites`] 的三条判据，逐次判。
fn is_tearable_in_place_overwrite(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    write_index: usize,
) -> bool {
    let write = &writes[write_index];
    let is_copy_on_write = match write.kind {
        StepKind::UnitWrite => true,
        // discard 是原地的范围动作（「范围写零」）：照三条认，长于一个扇区、罩住旧内容的取第三态（D13（验证路线） 已定项 4 原地覆写的字面，
        // 判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S6）。
        StepKind::Discard => false,
        StepKind::ZeroFill
        | StepKind::JournalRecord
        | StepKind::RootRecordFua
        | StepKind::SystemConfigurationSlot
        | StepKind::Barrier => false,
    };
    if is_copy_on_write || write.length_in_bytes() <= SECTOR_BYTES {
        return false;
    }
    let write_end = write.offset.0 + write.length_in_bytes();
    let base_holds_nonzero_bytes_there = base.devices.get(&write.device).is_some_and(|device| {
        device
            .written_sectors()
            .range(write.offset.0 / SECTOR_BYTES..write_end.div_ceil(SECTOR_BYTES))
            .any(|(_, bytes)| bytes.iter().any(|byte| *byte != 0))
    });
    let an_earlier_write_with_bytes_overlaps = writes[..write_index].iter().any(|earlier| {
        let earlier_has_bytes = match earlier.contents {
            WrittenContents::Bytes(_) => true,
            WrittenContents::Zeros { .. } => false,
        };
        earlier_has_bytes
            && earlier.device == write.device
            && earlier.offset.0 < write_end
            && write.offset.0 < earlier.offset.0 + earlier.length_in_bytes()
    });
    base_holds_nonzero_bytes_there || an_earlier_write_with_bytes_overlaps
}

/// 一次原地覆写撕裂时那一段的字节，「新旧都读不出」：新旧不同的那一截（第一个不同的字节到最后一个不同的字节）前一半是新的、
/// 后一半还是旧的，这一截之外新旧本来就相同。第一个不同的字节是新的、最后一个是旧的，所以新旧有两个以上的字节不同时它与新、旧都不同；
/// 整段校验和罩着的写（系统配置槽、journal 记录）新旧两份校验和都对不上（校验和碰撞不在模型里，D13（验证路线） 已定项 4 射程）。
/// 新旧只差一个字节时撕不出第三种内容，交回旧的；完全相同时交回新的（也就是旧的）——这一态的镜像与另外两态之一逐字节相同。
///
/// # Panics
/// `old` 与这次写不一样长。
#[must_use]
pub fn torn_image_of_in_place_overwrite(old: &[u8], new: &WrittenContents) -> Vec<u8> {
    let length = usize::try_from(new.length_in_bytes()).expect("写长装得进 usize");
    assert_eq!(old.len(), length, "旧字节与这次写一样长");
    let mut new_bytes = vec![0u8; length];
    new.copy_range_into(0, &mut new_bytes);
    let differs = |index: &usize| old[*index] != new_bytes[*index];
    let Some(first_differing_byte) = (0..length).find(differs) else {
        return new_bytes;
    };
    let last_differing_byte = (0..length)
        .rev()
        .find(differs)
        .expect("有第一个不同的字节就有最后一个");
    let differing_span_in_bytes = last_differing_byte - first_differing_byte + 1;
    let first_old_byte = first_differing_byte + differing_span_in_bytes / 2;
    let mut torn = new_bytes;
    torn[first_old_byte..].copy_from_slice(&old[first_old_byte..]);
    torn
}

/// 枚举用的写表：录制流里的写原样在前（下标不变），其后每个原地覆写的写接一条撕裂镜像（罩住那次写整个范围、字节是
/// [`torn_image_of_in_place_overwrite`] 按「这次写之前的写全落了」时的旧字节算的），再接同一段里在它之后、同一块盘上与它重叠的写
/// 各一份重放——那几次写落了时要盖在撕裂镜像上面，与录制流里的先后一致（[`CrashImage`] 按写表次序叠）。
/// 一个状态的持久集合按这张表给：撕裂那一态是「原来那次写没持久、它的撕裂镜像持久」，重放跟着被重放的那次写落不落。
struct WritesWithTornImages {
    writes: Vec<RetainedWrite>,
    /// 原地覆写的写（写表下标）→ 它的撕裂镜像与之后那几份重放在 `writes` 里的位置。
    torn_image_by_write: BTreeMap<usize, TornImageOfOneWrite>,
}

struct TornImageOfOneWrite {
    torn_image_index: usize,
    /// （重放在 `writes` 里的下标，重放的是写表里哪一次写）。
    replays: Vec<(usize, usize)>,
}

impl WritesWithTornImages {
    fn of(
        base: &MemoryPool,
        writes: &[RetainedWrite],
        segments: &[Vec<usize>],
        tearable: &TearableInPlaceOverwrites,
    ) -> Self {
        let mut segment_of_write: BTreeMap<usize, usize> = BTreeMap::new();
        for (segment_index, segment) in segments.iter().enumerate() {
            for write_index in segment {
                segment_of_write.insert(*write_index, segment_index);
            }
        }
        let mut writes_with_torn_images = writes.to_vec();
        let mut torn_image_by_write = BTreeMap::new();
        let tearable_write_indexes = tearable.write_indexes();
        if tearable_write_indexes.is_empty() {
            return Self {
                writes: writes_with_torn_images,
                torn_image_by_write,
            };
        }
        // 每次写之前的镜像：基镜像上按次序叠写表里更早的写（这次写之前的写全落了）。
        let mut image_before_the_write = base.clone();
        let mut next_write_to_apply = 0usize;
        for write_index in tearable_write_indexes {
            image_before_the_write.apply_writes(&writes[next_write_to_apply..write_index]);
            next_write_to_apply = write_index;
            let write = &writes[write_index];
            let old = PoolReader::read(
                &image_before_the_write,
                write.device,
                write.offset,
                usize::try_from(write.length_in_bytes()).expect("写长装得进 usize"),
            )
            .expect("录到的写落在池里的盘上、在盘内、按扇区对齐");
            let torn_image_index = writes_with_torn_images.len();
            writes_with_torn_images.push(RetainedWrite {
                device: write.device,
                kind: write.kind,
                is_force_unit_access: write.is_force_unit_access,
                offset: write.offset,
                contents: WrittenContents::Bytes(torn_image_of_in_place_overwrite(
                    &old,
                    &write.contents,
                )),
            });
            let write_end = write.offset.0 + write.length_in_bytes();
            let later_overlapping_writes_in_the_segment: Vec<usize> = segment_of_write
                .get(&write_index)
                .map(|segment_index| {
                    segments[*segment_index]
                        .iter()
                        .copied()
                        .filter(|later_index| {
                            let later = &writes[*later_index];
                            *later_index > write_index
                                && later.device == write.device
                                && later.offset.0 < write_end
                                && write.offset.0 < later.offset.0 + later.length_in_bytes()
                        })
                        .collect()
                })
                .unwrap_or_default();
            let replays = later_overlapping_writes_in_the_segment
                .into_iter()
                .map(|replayed_write_index| {
                    writes_with_torn_images.push(writes[replayed_write_index].clone());
                    (writes_with_torn_images.len() - 1, replayed_write_index)
                })
                .collect();
            torn_image_by_write.insert(
                write_index,
                TornImageOfOneWrite {
                    torn_image_index,
                    replays,
                },
            );
        }
        Self {
            writes: writes_with_torn_images,
            torn_image_by_write,
        }
    }
}

/// 一段里的原地写与单元写（COW），各按段内次序：分法只有一份，在 [`crate::crash_facts::in_place_and_copy_on_write_writes_of_the_segment`]
/// （GPU 判表与第 ② 段核对的拆法也照它），这里只把写表下标换回 `usize`。
fn in_place_and_copy_on_write_writes_of(
    writes: &[RetainedWrite],
    segment: &[usize],
) -> (Vec<usize>, Vec<usize>) {
    let write_index_of = |index: u32| usize::try_from(index).expect("写表下标装得进 usize");
    let (in_place_writes, copy_on_write_writes) =
        crate::crash_facts::in_place_and_copy_on_write_writes_of_the_segment(writes, segment);
    (
        in_place_writes.into_iter().map(write_index_of).collect(),
        copy_on_write_writes
            .into_iter()
            .map(write_index_of)
            .collect(),
    )
}

/// 按 `expansion` 展开、每次写只取两态（没持久 / 持久，补第三态之前的口径）时的状态数：各段展开出来的，再加全部持久那一个
/// （全量时就是 [`closed_form_state_count_of_write_index_segments`] 的闭式）。崩溃点重放枚举出来的状态数是
/// [`crash_state_count_with_torn_in_place_overwrites`]：写表里没有原地覆写的写时两者相等。
#[must_use]
pub fn crash_state_count(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> u64 {
    state_count_of_the_segments(
        writes,
        segments,
        expansion,
        &TearableInPlaceOverwrites::none(writes.len()),
    )
}

/// 崩溃点重放按 `expansion` 枚举出来的状态数（`enumerate_crash_states` 一族的 `states`）：原地覆写的写（[`TearableInPlaceOverwrites`]，要看基镜像）
/// 取三态、其余取两态，各段展开出来的，再加全部持久那一个。
#[must_use]
pub fn crash_state_count_with_torn_in_place_overwrites(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> u64 {
    state_count_of_the_segments(
        writes,
        segments,
        expansion,
        &TearableInPlaceOverwrites::of(base, writes),
    )
}

fn state_count_of_the_segments(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    tearable: &TearableInPlaceOverwrites,
) -> u64 {
    segments
        .iter()
        .enumerate()
        .map(|(segment_index, segment)| {
            expansion(segment_index, segment).state_count(writes, segment, tearable)
        })
        .try_fold(1u64, u64::checked_add)
        .expect("整条流的状态数装得进 u64")
}

/// 每一段都按甲二展开（[`CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll`]）。
#[must_use]
pub fn in_place_subsets_with_copy_on_write_none_or_all_expansion(
    _segment_index: usize,
    _segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll
}

/// 每一段都展开任意真子集（全量）。
#[must_use]
pub fn full_expansion(
    _segment_index: usize,
    _segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    CrashPointReplaySegmentExpansion::EveryProperSubset
}

/// 枚举次序里每个状态的序号怎么落到「第几段、段内每次写怎么落」：次序与单线程逐段逐个组合走相同——
/// 前面的段全持久 + 当前段的一个组合（全量时段内序号按混合进制拆成每次写的一态，数到组合数 − 2；甲二时见
/// [`Self::persisted_writes_of_state`]），最后再加全部持久那一个状态。
struct CrashStatePlan<'plan> {
    segments: &'plan [Vec<usize>],
    /// 每一段怎么展开，与 `segments` 同序。
    expansion_by_segment: Vec<CrashPointReplaySegmentExpansion>,
    /// 每一段展开出来的状态的序号区间，首尾相接、随段号递增；不展开的段是空区间。
    state_ranges_by_segment: Vec<Range<u64>>,
    /// 状态总数：各段展开出来的，再加最后全部持久那一个。
    state_count: u64,
    /// 每一段归哪次发布（[`publish_of_each_segment`]），与 `segments` 同序。
    publish_of_segment: Vec<CrashPointReplayPublishOfState>,
    /// 每一段的原地写与单元写（[`in_place_and_copy_on_write_writes_of`]），与 `segments` 同序。
    in_place_and_copy_on_write_by_segment: Vec<(Vec<usize>, Vec<usize>)>,
    /// 分片的段拆编号要的各位与片起点（[`CrashPointReplaySegmentSlice`]），与 `segments` 同序；不分片的段是 `None`。
    digits_of_each_sliced_segment: Vec<Option<DigitsOfASlicedSegment>>,
    /// 哪几次写取三态（[`TearableInPlaceOverwrites`]）。
    tearable: &'plan TearableInPlaceOverwrites,
    /// 枚举用的写表（录制流里的写在前，撕裂镜像与重放接在后面）：持久集合按它给。
    writes_with_torn_images: &'plan WritesWithTornImages,
}

impl<'plan> CrashStatePlan<'plan> {
    /// `expansion` 只在调用线程上逐段问一次，所以它不必能跨线程。`writes` 是录制流里的写（不带撕裂镜像）。
    /// `segment_slices` 与 `segments` 同序：分片的段在流里的序号区间是片长，不分片的段是它按展开方式展开出的全部状态。
    ///
    /// # Panics
    /// `segment_slices` 与段数不同；分片的段不展开、或片越过了它的域（片起点 + 片长大于整域的态数）、或整域越过 1280 位；
    /// 不分片的段的态数装不进 u64；整条流的状态数装不进 u64。
    fn new(
        writes: &[RetainedWrite],
        segments: &'plan [Vec<usize>],
        expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
        segment_slices: &[Option<CrashPointReplaySegmentSlice>],
        tearable: &'plan TearableInPlaceOverwrites,
        writes_with_torn_images: &'plan WritesWithTornImages,
    ) -> Self {
        assert_eq!(
            segment_slices.len(),
            segments.len(),
            "每一段一格片（不分片的段是 None）"
        );
        let expansion_by_segment: Vec<CrashPointReplaySegmentExpansion> = segments
            .iter()
            .enumerate()
            .map(|(segment_index, segment)| expansion(segment_index, segment))
            .collect();
        let in_place_and_copy_on_write_by_segment: Vec<(Vec<usize>, Vec<usize>)> = segments
            .iter()
            .map(|segment| in_place_and_copy_on_write_writes_of(writes, segment))
            .collect();
        let digits_of_each_sliced_segment: Vec<Option<DigitsOfASlicedSegment>> = segments
            .iter()
            .enumerate()
            .map(|(segment_index, segment)| {
                segment_slices[segment_index].map(|slice| {
                    let segment_expansion = expansion_by_segment[segment_index];
                    let domain = segment_expansion
                        .checked_state_count(writes, segment, tearable)
                        .expect("分片的段整域的态数装得进 1280 位：录入段在写库之前核过");
                    let slice_end = slice
                        .start_within_the_domain
                        .checked_add_u64(slice.state_count)
                        .expect("片尾装得进 1280 位：不超过整域");
                    assert!(
                        segment_expansion != CrashPointReplaySegmentExpansion::NotExpanded
                            && slice_end <= domain,
                        "段 {segment_index} 的片 [{}, {}) 越过了它按 {} 展开的域（{} 个状态）",
                        slice.start_within_the_domain.decimal_text(),
                        slice_end.decimal_text(),
                        segment_expansion.name(),
                        domain.decimal_text()
                    );
                    let (positions, radices): (Vec<CrashPointReplayDigitOfASegment>, Vec<u8>) =
                        segment_expansion
                            .digits_of_the_segment(
                                segment,
                                &in_place_and_copy_on_write_by_segment[segment_index],
                                tearable,
                            )
                            .into_iter()
                            .unzip();
                    let slice_start_digits = slice
                        .start_within_the_domain
                        .mixed_radix_digits(&radices)
                        .expect("片起点小于整域的态数，拆得进各位");
                    DigitsOfASlicedSegment {
                        positions,
                        radices,
                        slice_start_digits,
                    }
                })
            })
            .collect();
        let mut next_ordinal = 0u64;
        let state_ranges_by_segment = segments
            .iter()
            .zip(&expansion_by_segment)
            .zip(segment_slices)
            .map(|((segment, segment_expansion), slice)| {
                let first_ordinal = next_ordinal;
                let states_of_the_segment = match slice {
                    Some(slice) => slice.state_count,
                    None => segment_expansion.state_count(writes, segment, tearable),
                };
                next_ordinal = next_ordinal
                    .checked_add(states_of_the_segment)
                    .expect("整条流的状态数装得进 u64");
                first_ordinal..next_ordinal
            })
            .collect();
        Self {
            segments,
            expansion_by_segment,
            state_ranges_by_segment,
            state_count: next_ordinal + 1,
            publish_of_segment: publish_of_each_segment(writes, segments),
            in_place_and_copy_on_write_by_segment,
            digits_of_each_sliced_segment,
            tearable,
            writes_with_torn_images,
        }
    }

    /// 段内序号 `ordinal_within` 按混合进制拆到 `write_indexes` 这几次写上（第一次写是最低位，每位的进制是它能取几态）。
    ///
    /// # Panics
    /// 序号超出这几次写的组合数（计划算错了）。
    fn assign_landings(
        &self,
        landings: &mut [WriteLandingInCrashState],
        ordinal_within: u64,
        write_indexes: &[usize],
    ) {
        let mut remaining = ordinal_within;
        for write_index in write_indexes {
            let choices = self.tearable.landing_choices_of(*write_index);
            landings[*write_index] = choices.landing_of_digit(remaining % choices.count());
            remaining /= choices.count();
        }
        assert_eq!(
            remaining, 0,
            "段内序号 {ordinal_within} 超出这几次写（{write_indexes:?}）的组合数"
        );
    }

    /// 序号落在哪一段；等于段数说明是最后全部持久那一个状态。
    fn segment_of_state(&self, ordinal: u64) -> usize {
        self.state_ranges_by_segment
            .partition_point(|state_range| state_range.end <= ordinal)
    }

    /// 这个状态归哪次发布：所在那一段归的那一次；最后全部持久那一个状态单列一格。
    fn publish_of_state(&self, ordinal: u64) -> CrashPointReplayPublishOfState {
        self.publish_of_segment
            .get(self.segment_of_state(ordinal))
            .copied()
            .unwrap_or(CrashPointReplayPublishOfState::EveryWritePersisted)
    }

    /// 这个状态的持久集合，按枚举用的写表（[`WritesWithTornImages`]）给：所在的段之前每一段整段持久（展不展开都一样），
    /// 所在的段按段内序号 r（序号减去这一段的起点）拆到段里每次写上（[`Self::assign_landings`]，原地覆写的写三态、其余两态）：
    /// 全量时 r 拆到段里全部的写上；甲二时单元写一个都没有就同全量（只拆到原地写上），有单元写时 r / 2 拆到原地写上、
    /// r 除以 2 的余数是「单元写全落」——次序是原地写的每个组合先单元写全不落、再全落，整段全落那一个取不到。
    /// 每次写都只取两态时 r 就是补第三态之前的子集掩码，次序逐个相同。撕裂那一态记成「原来那次写没持久、它的撕裂镜像持久」，
    /// 撕裂镜像之后的重放跟着被重放的那次写落不落。
    fn persisted_writes_of_state(&self, ordinal: u64) -> Vec<bool> {
        let segment_of_state = self.segment_of_state(ordinal);
        let recorded_write_count = self.tearable.is_tearable_by_write.len();
        let mut landings = vec![WriteLandingInCrashState::NotPersisted; recorded_write_count];
        for segment in &self.segments[..segment_of_state] {
            for write_index in segment {
                landings[*write_index] = WriteLandingInCrashState::Persisted;
            }
        }
        if let (Some(_segment), Some(Some(digits_of_the_slice))) = (
            self.segments.get(segment_of_state),
            self.digits_of_each_sliced_segment.get(segment_of_state),
        ) {
            // 分片的段：段内编号 = 片起点 + 片内第几个；片起点拆成的各位加上片内序号、逐位进位，再照每一位落
            let ordinal_within_the_slice =
                ordinal - self.state_ranges_by_segment[segment_of_state].start;
            let mut digits = digits_of_the_slice.slice_start_digits.clone();
            assert!(
                add_to_the_mixed_radix_digits(
                    &mut digits,
                    &digits_of_the_slice.radices,
                    ordinal_within_the_slice
                ),
                "段 {segment_of_state} 片内第 {ordinal_within_the_slice} 个越过了它的域（计划算错了）"
            );
            for (position, digit) in digits_of_the_slice.positions.iter().zip(&digits) {
                match position {
                    CrashPointReplayDigitOfASegment::CopyOnWriteGroup => {
                        if *digit == 1 {
                            let (_in_place_writes, copy_on_write_writes) =
                                &self.in_place_and_copy_on_write_by_segment[segment_of_state];
                            for write_index in copy_on_write_writes {
                                landings[*write_index] = WriteLandingInCrashState::Persisted;
                            }
                        }
                    }
                    CrashPointReplayDigitOfASegment::LandingOfAWrite(write_index) => {
                        landings[*write_index] = self
                            .tearable
                            .landing_choices_of(*write_index)
                            .landing_of_digit(u64::from(*digit));
                    }
                }
            }
        } else if let Some(segment) = self.segments.get(segment_of_state) {
            let ordinal_within_the_segment =
                ordinal - self.state_ranges_by_segment[segment_of_state].start;
            match self.expansion_by_segment[segment_of_state] {
                CrashPointReplaySegmentExpansion::NotExpanded => {
                    unreachable!(
                        "不展开的段序号区间是空的，没有状态落在它里面（段 {segment_of_state}）"
                    )
                }
                CrashPointReplaySegmentExpansion::EveryProperSubset => {
                    self.assign_landings(&mut landings, ordinal_within_the_segment, segment);
                }
                CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
                    let (in_place_writes, copy_on_write_writes) =
                        &self.in_place_and_copy_on_write_by_segment[segment_of_state];
                    if copy_on_write_writes.is_empty() {
                        self.assign_landings(
                            &mut landings,
                            ordinal_within_the_segment,
                            in_place_writes,
                        );
                    } else {
                        self.assign_landings(
                            &mut landings,
                            ordinal_within_the_segment / 2,
                            in_place_writes,
                        );
                        if ordinal_within_the_segment % 2 == 1 {
                            for write_index in copy_on_write_writes {
                                landings[*write_index] = WriteLandingInCrashState::Persisted;
                            }
                        }
                    }
                }
            }
        }
        let mut persisted = vec![false; self.writes_with_torn_images.writes.len()];
        for (write_index, landing) in landings.iter().enumerate() {
            persisted[write_index] = match landing {
                WriteLandingInCrashState::Persisted => true,
                WriteLandingInCrashState::NotPersisted | WriteLandingInCrashState::Torn => false,
            };
        }
        for (write_index, torn_image) in &self.writes_with_torn_images.torn_image_by_write {
            match landings[*write_index] {
                WriteLandingInCrashState::Torn => {
                    persisted[torn_image.torn_image_index] = true;
                    for (replay_index, replayed_write_index) in &torn_image.replays {
                        persisted[*replay_index] = match landings[*replayed_write_index] {
                            WriteLandingInCrashState::Persisted => true,
                            WriteLandingInCrashState::NotPersisted
                            | WriteLandingInCrashState::Torn => false,
                        };
                    }
                }
                WriteLandingInCrashState::NotPersisted | WriteLandingInCrashState::Persisted => {}
            }
        }
        persisted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_pool::closed_form_state_count_of_write_index_segments;

    /// 写表里一串按次序摆的写，种类照给的：盘 0、每次一个扇区、内容全 0（只核计划的用例不读内容）。
    fn writes_of_kinds(kinds: &[StepKind]) -> Vec<RetainedWrite> {
        kinds
            .iter()
            .enumerate()
            .map(|(index, kind)| RetainedWrite {
                device: DeviceIdentity(0),
                kind: *kind,
                is_force_unit_access: false,
                offset: DeviceOffsetInBytes(
                    u64::try_from(index).expect("下标装得进 u64") * SECTOR_BYTES,
                ),
                contents: WrittenContents::Bytes(vec![0u8; 512]),
            })
            .collect()
    }

    /// 甲二那一串持久集合的参照走法（与计划分开写）：逐段，段内先按原地写的子集掩码从 0 数到 2^k − 1，每个掩码先「单元写全不落」、
    /// 有单元写时再「单元写全落」，去掉整段全落那一个；每段走完整段持久。
    fn persisted_sets_walking_the_in_place_subsets_with_copy_on_write_none_or_all(
        writes: &[RetainedWrite],
        segments: &[Vec<usize>],
    ) -> Vec<Vec<bool>> {
        let mut persisted_sets = Vec::new();
        let mut persisted_before = vec![false; writes.len()];
        for segment in segments {
            let in_place: Vec<usize> = segment
                .iter()
                .copied()
                .filter(|write_index| writes[*write_index].kind != StepKind::UnitWrite)
                .collect();
            let copy_on_write: Vec<usize> = segment
                .iter()
                .copied()
                .filter(|write_index| writes[*write_index].kind == StepKind::UnitWrite)
                .collect();
            for in_place_mask in 0..1u64 << in_place.len() {
                let mut with_in_place_subset = persisted_before.clone();
                for (bit, write_index) in in_place.iter().enumerate() {
                    if in_place_mask & (1 << bit) != 0 {
                        with_in_place_subset[*write_index] = true;
                    }
                }
                let every_in_place_write = in_place_mask == (1 << in_place.len()) - 1;
                if !(copy_on_write.is_empty() && every_in_place_write) {
                    persisted_sets.push(with_in_place_subset.clone());
                }
                if !copy_on_write.is_empty() && !every_in_place_write {
                    let mut with_every_copy_on_write = with_in_place_subset;
                    for write_index in &copy_on_write {
                        with_every_copy_on_write[*write_index] = true;
                    }
                    persisted_sets.push(with_every_copy_on_write);
                }
            }
            for write_index in segment {
                persisted_before[*write_index] = true;
            }
        }
        persisted_sets.push(persisted_before);
        persisted_sets
    }

    /// 甲二：计划按序号给出的持久集合与参照走法逐个相同，状态数等于闭式（有单元写的段 2^(k+1) − 1、没有的 2^k − 1，再加 1）；
    /// 全是单元写的段只有「全不落」一个状态，全是原地写的段同全量。
    #[test]
    fn the_in_place_subsets_plan_takes_every_in_place_subset_with_the_copy_on_write_writes_none_or_all(
    ) {
        let kinds = [
            StepKind::SystemConfigurationSlot,
            StepKind::SystemConfigurationSlot,
            StepKind::UnitWrite,
            StepKind::UnitWrite,
            StepKind::UnitWrite,
            StepKind::JournalRecord,
            StepKind::JournalRecord,
            StepKind::RootRecordFua,
            StepKind::UnitWrite,
            StepKind::UnitWrite,
            StepKind::SystemConfigurationSlot,
            StepKind::UnitWrite,
        ];
        let writes = writes_of_kinds(&kinds);
        let segments = vec![
            vec![0, 1, 2, 3, 4],
            vec![5, 6],
            vec![7],
            vec![8, 9],
            vec![10, 11],
        ];
        let walked = persisted_sets_walking_the_in_place_subsets_with_copy_on_write_none_or_all(
            &writes, &segments,
        );
        let tearable = TearableInPlaceOverwrites::none(writes.len());
        let writes_with_torn_images =
            WritesWithTornImages::of(&MemoryPool::default(), &writes, &segments, &tearable);
        let plan = CrashStatePlan::new(
            &writes,
            &segments,
            &in_place_subsets_with_copy_on_write_none_or_all_expansion,
            &no_segment_slices(segments.len()),
            &tearable,
            &writes_with_torn_images,
        );
        // [2 原地 + 3 单元] 7、[2 原地] 3、[1 原地] 1、[2 单元] 1、[1 原地 + 1 单元] 3，再加全部持久那一个。
        assert_eq!(plan.state_count, 7 + 3 + 1 + 1 + 3 + 1);
        assert_eq!(
            crash_state_count(
                &writes,
                &segments,
                &in_place_subsets_with_copy_on_write_none_or_all_expansion
            ),
            plan.state_count
        );
        assert_eq!(
            plan.state_count,
            u64::try_from(walked.len()).expect("状态数"),
            "状态数与参照走法的相同"
        );
        let by_ordinal: Vec<Vec<bool>> = (0..plan.state_count)
            .map(|ordinal| plan.persisted_writes_of_state(ordinal))
            .collect();
        assert_eq!(by_ordinal, walked, "第 k 个状态就是参照走法的第 k 个");
        assert_eq!(
            crash_state_count(&writes, &segments, &full_expansion),
            closed_form_state_count_of_write_index_segments(&segments),
            "全量的状态数就是闭式"
        );
    }

    /// 一次手摆的写：`length_in_bytes` 个 `fill` 字节。
    fn write_filled_with(
        device: u32,
        kind: StepKind,
        offset: u64,
        length_in_bytes: usize,
        fill: u8,
    ) -> RetainedWrite {
        RetainedWrite {
            device: DeviceIdentity(device),
            kind,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(offset),
            contents: WrittenContents::Bytes(vec![fill; length_in_bytes]),
        }
    }

    /// 撕裂镜像的字节：新旧不同的那一截前一半新、后一半旧，与新、旧都不同；只差一个字节时撕不出第三种、交回旧的；完全相同交回新的；
    /// 整段清零照同一条算（新的是全 0）。
    #[test]
    fn the_torn_image_keeps_the_first_half_of_the_differing_bytes_new_and_the_rest_old() {
        let old = [1u8, 1, 1, 1, 1, 1, 1, 1];
        let new = WrittenContents::Bytes(vec![1, 2, 2, 2, 2, 2, 2, 1]);
        let torn = torn_image_of_in_place_overwrite(&old, &new);
        assert_eq!(torn, vec![1, 2, 2, 2, 1, 1, 1, 1]);
        assert_ne!(torn.as_slice(), old.as_slice(), "与旧的不同");
        assert_ne!(Some(torn.as_slice()), new.as_bytes(), "与新的不同");
        assert_eq!(
            torn_image_of_in_place_overwrite(
                &old,
                &WrittenContents::Bytes(vec![1, 1, 1, 9, 1, 1, 1, 1])
            ),
            old.to_vec(),
            "只差一个字节：撕不出第三种内容，交回旧的"
        );
        assert_eq!(
            torn_image_of_in_place_overwrite(&old, &WrittenContents::Bytes(old.to_vec())),
            old.to_vec(),
            "完全相同"
        );
        assert_eq!(
            torn_image_of_in_place_overwrite(&old, &WrittenContents::Zeros { length: 8 }),
            vec![0, 0, 0, 0, 1, 1, 1, 1],
            "整段清零：前一半清了、后一半还是旧的"
        );
    }

    /// 补第三态之后计划给出的持久集合：一段里原地覆写的写三态、其余两态，按混合进制（第一次写是最低位，没持久 0、撕裂 1、持久 2）
    /// 逐个组合各出一次、整段全持久那一个不出；撕裂那一态是「原来那次写没持久、它的撕裂镜像持久」，同一段里在它之后、与它重叠的写
    /// 落了时它的重放也持久（盖在撕裂镜像上面）。参照走法是三层循环，与计划的算法分开写。
    #[test]
    fn the_state_plan_with_torn_in_place_overwrites_hands_out_every_landing_combination_once() {
        const SYSTEM_CONFIGURATION_SLOT: u64 = 4096;
        const JOURNAL_SLOT: u64 = 16 << 20;
        const UNIT_SLOT: u64 = 784 << 20;
        let mut base = MemoryPool::with_devices(&[DeviceIdentity(0)], 1 << 30);
        base.devices
            .get_mut(&DeviceIdentity(0))
            .expect("盘 0")
            .write(
                DeviceOffsetInBytes(SYSTEM_CONFIGURATION_SLOT),
                &[0x5Au8; 4096],
            );
        let writes = vec![
            write_filled_with(
                0,
                StepKind::SystemConfigurationSlot,
                SYSTEM_CONFIGURATION_SLOT,
                4096,
                1,
            ),
            write_filled_with(0, StepKind::UnitWrite, UNIT_SLOT, 512, 2),
            write_filled_with(0, StepKind::JournalRecord, JOURNAL_SLOT, 4096, 3),
            write_filled_with(
                0,
                StepKind::SystemConfigurationSlot,
                SYSTEM_CONFIGURATION_SLOT,
                4096,
                4,
            ),
            write_filled_with(0, StepKind::JournalRecord, JOURNAL_SLOT, 4096, 5),
        ];
        // 写 0（基镜像那一槽有旧内容）、写 3（与更早的写 0 重叠）、写 4（与更早的写 2 重叠）是原地覆写；写 3 与写 0 同段、在它之后。
        let segments = vec![vec![0, 1, 3], vec![2], vec![4]];
        let tearable = TearableInPlaceOverwrites::of(&base, &writes);
        assert_eq!(tearable.write_indexes(), vec![0, 3, 4]);
        let writes_with_torn_images =
            WritesWithTornImages::of(&base, &writes, &segments, &tearable);
        // 写表后面接：写 0 的撕裂镜像（5）、写 3 的重放（6）、写 3 的撕裂镜像（7）、写 4 的撕裂镜像（8）。
        assert_eq!(writes_with_torn_images.writes.len(), 9);
        assert_eq!(
            writes_with_torn_images.writes[5].contents,
            WrittenContents::Bytes(torn_image_of_in_place_overwrite(
                &[0x5Au8; 4096],
                &writes[0].contents
            )),
            "写 0 的撕裂镜像按基镜像上的旧字节算"
        );
        assert_eq!(writes_with_torn_images.writes[6], writes[3], "写 3 的重放");
        assert_eq!(
            writes_with_torn_images.writes[7].contents,
            WrittenContents::Bytes(torn_image_of_in_place_overwrite(
                &[1u8; 4096],
                &writes[3].contents
            )),
            "写 3 的撕裂镜像按「写 0 已落」时的旧字节算"
        );
        let plan = CrashStatePlan::new(
            &writes,
            &segments,
            &full_expansion,
            &no_segment_slices(segments.len()),
            &tearable,
            &writes_with_torn_images,
        );
        // 第一段 3 · 2 · 3 − 1、第二段 2 − 1、第三段 3 − 1，再加全部持久那一个。
        assert_eq!(plan.state_count, 17 + 1 + 2 + 1);
        assert_eq!(
            crash_state_count_with_torn_in_place_overwrites(
                &base,
                &writes,
                &segments,
                &full_expansion
            ),
            plan.state_count
        );
        use WriteLandingInCrashState::{NotPersisted, Persisted, Torn};
        let torn_image_index_of = |write_index: usize| match write_index {
            0 => Some(5),
            3 => Some(7),
            4 => Some(8),
            _ => None,
        };
        let landing_of = |persisted: &[bool], write_index: usize| {
            if persisted[write_index] {
                Persisted
            } else if torn_image_index_of(write_index).is_some_and(|index| persisted[index]) {
                Torn
            } else {
                NotPersisted
            }
        };
        let mut expected: Vec<[WriteLandingInCrashState; 5]> = Vec::new();
        for landing_of_write_three in [NotPersisted, Torn, Persisted] {
            for landing_of_write_one in [NotPersisted, Persisted] {
                for landing_of_write_zero in [NotPersisted, Torn, Persisted] {
                    expected.push([
                        landing_of_write_zero,
                        landing_of_write_one,
                        NotPersisted,
                        landing_of_write_three,
                        NotPersisted,
                    ]);
                }
            }
        }
        expected.pop();
        expected.push([Persisted, Persisted, NotPersisted, Persisted, NotPersisted]);
        expected.push([Persisted, Persisted, Persisted, Persisted, NotPersisted]);
        expected.push([Persisted, Persisted, Persisted, Persisted, Torn]);
        expected.push([Persisted; 5]);
        let mut by_ordinal: Vec<[WriteLandingInCrashState; 5]> = Vec::new();
        for ordinal in 0..plan.state_count {
            let persisted = plan.persisted_writes_of_state(ordinal);
            assert_eq!(persisted.len(), 9, "持久集合与枚举用的写表逐条对应");
            let landings = [0, 1, 2, 3, 4].map(|write_index| landing_of(&persisted, write_index));
            assert_eq!(
                persisted[6],
                landings[0] == Torn && landings[3] == Persisted,
                "写 3 的重放只在写 0 撕裂、写 3 落了时持久（第 {ordinal} 个状态）"
            );
            by_ordinal.push(landings);
        }
        assert_eq!(by_ordinal, expected, "第 k 个状态就是参照走法的第 k 个");
        // 写 0 撕裂、写 3 落了：那一槽读出来是写 3 的字节（重放盖在撕裂镜像上面）；写 3 没落：读出来是写 0 的撕裂镜像。
        let slot_in_state = |landing_of_write_three: WriteLandingInCrashState| {
            let ordinal = by_ordinal
                .iter()
                .position(|landings| {
                    landings[0] == Torn
                        && landings[1] == NotPersisted
                        && landings[3] == landing_of_write_three
                })
                .expect("有这个状态");
            let image = CrashImage {
                base: &base,
                writes: &writes_with_torn_images.writes,
                persisted: plan.persisted_writes_of_state(u64::try_from(ordinal).expect("序号")),
            };
            PoolReader::read(
                &image,
                DeviceIdentity(0),
                DeviceOffsetInBytes(SYSTEM_CONFIGURATION_SLOT),
                4096,
            )
            .expect("读得出")
        };
        assert_eq!(slot_in_state(Persisted), vec![4u8; 4096]);
        assert_eq!(
            Some(slot_in_state(NotPersisted).as_slice()),
            writes_with_torn_images.writes[5].contents.as_bytes()
        );
    }

    /// 甲二一段：两次原地覆写（三态）与一组两次单元写（整组落或不落），3·3·2 − 1 = 17 个态。分片的计划按片起点 + 片内序号拆出的持久集合，
    /// 与不分片的计划在段内同一个编号上拆出的逐个相同；几片首尾相接罩住整域时，各片拆出的态合起来与整域逐个相同、一个不多一个不少（编号到态是双射）。
    #[test]
    fn slices_of_an_in_place_subsets_segment_decode_the_same_states_as_the_whole_segment() {
        const SYSTEM_CONFIGURATION_SLOT: u64 = 4096;
        const SECOND_SYSTEM_CONFIGURATION_SLOT: u64 = 8 << 20;
        const UNIT_SLOT: u64 = 784 << 20;
        let mut base = MemoryPool::with_devices(&[DeviceIdentity(0)], 1 << 30);
        for slot in [SYSTEM_CONFIGURATION_SLOT, SECOND_SYSTEM_CONFIGURATION_SLOT] {
            base.devices
                .get_mut(&DeviceIdentity(0))
                .expect("盘 0")
                .write(DeviceOffsetInBytes(slot), &[0x5Au8; 4096]);
        }
        let writes = vec![
            write_filled_with(
                0,
                StepKind::SystemConfigurationSlot,
                SYSTEM_CONFIGURATION_SLOT,
                4096,
                1,
            ),
            write_filled_with(0, StepKind::UnitWrite, UNIT_SLOT, 512, 2),
            write_filled_with(
                0,
                StepKind::SystemConfigurationSlot,
                SECOND_SYSTEM_CONFIGURATION_SLOT,
                4096,
                3,
            ),
            write_filled_with(0, StepKind::UnitWrite, UNIT_SLOT + 16384, 512, 4),
        ];
        let segments = vec![vec![0, 1, 2, 3]];
        let tearable = TearableInPlaceOverwrites::of(&base, &writes);
        assert_eq!(
            tearable.write_indexes(),
            vec![0, 2],
            "两次系统配置槽写都罩住旧内容、取三态"
        );
        let writes_with_torn_images =
            WritesWithTornImages::of(&base, &writes, &segments, &tearable);
        let whole = CrashStatePlan::new(
            &writes,
            &segments,
            &in_place_subsets_with_copy_on_write_none_or_all_expansion,
            &no_segment_slices(1),
            &tearable,
            &writes_with_torn_images,
        );
        assert_eq!(whole.state_count, 17 + 1);
        let mut states_of_the_slices = Vec::new();
        for (start, state_count) in [(0u64, 5u64), (5, 7), (12, 5)] {
            let slices = vec![Some(CrashPointReplaySegmentSlice {
                start_within_the_domain: StateCount1280Bits::from_u64(start),
                state_count,
            })];
            let sliced = CrashStatePlan::new(
                &writes,
                &segments,
                &in_place_subsets_with_copy_on_write_none_or_all_expansion,
                &slices,
                &tearable,
                &writes_with_torn_images,
            );
            assert_eq!(
                sliced.state_count,
                state_count + 1,
                "片长加最后全部持久那一个"
            );
            for ordinal_within_the_slice in 0..state_count {
                let persisted = sliced.persisted_writes_of_state(ordinal_within_the_slice);
                assert_eq!(
                    persisted,
                    whole.persisted_writes_of_state(start + ordinal_within_the_slice),
                    "片 [{start}, {}) 第 {ordinal_within_the_slice} 个是段内编号 {} 的那个态",
                    start + state_count,
                    start + ordinal_within_the_slice
                );
                states_of_the_slices.push(persisted);
            }
        }
        let states_of_the_whole_segment: Vec<Vec<bool>> = (0..17)
            .map(|ordinal| whole.persisted_writes_of_state(ordinal))
            .collect();
        assert_eq!(states_of_the_slices, states_of_the_whole_segment);
        let distinct: std::collections::BTreeSet<Vec<bool>> =
            states_of_the_slices.iter().cloned().collect();
        assert_eq!(distinct.len(), 17, "17 个编号拆出 17 个不同的态");
    }

    /// 全量展开的一段（三次两态写，2^3 − 1 = 7 个态）同上：从段内编号 3 起的一片与整段在同一个编号上拆出的态逐个相同。
    #[test]
    fn slice_of_an_every_proper_subset_segment_decodes_the_same_states_as_the_whole_segment() {
        let writes = writes_of_kinds(&[
            StepKind::JournalRecord,
            StepKind::JournalRecord,
            StepKind::JournalRecord,
        ]);
        let segments = vec![vec![0, 1, 2]];
        let tearable = TearableInPlaceOverwrites::none(writes.len());
        let writes_with_torn_images =
            WritesWithTornImages::of(&MemoryPool::default(), &writes, &segments, &tearable);
        let whole = CrashStatePlan::new(
            &writes,
            &segments,
            &full_expansion,
            &no_segment_slices(1),
            &tearable,
            &writes_with_torn_images,
        );
        let slices = vec![Some(CrashPointReplaySegmentSlice {
            start_within_the_domain: StateCount1280Bits::from_u64(3),
            state_count: 4,
        })];
        let sliced = CrashStatePlan::new(
            &writes,
            &segments,
            &full_expansion,
            &slices,
            &tearable,
            &writes_with_torn_images,
        );
        for ordinal_within_the_slice in 0..4 {
            assert_eq!(
                sliced.persisted_writes_of_state(ordinal_within_the_slice),
                whole.persisted_writes_of_state(3 + ordinal_within_the_slice),
                "片内第 {ordinal_within_the_slice} 个"
            );
        }
    }

    /// 片越过段的域（片起点 + 片长大于整域的态数）：建计划时停下，不拆出域外的态。
    #[test]
    #[should_panic(expected = "越过了它按 every_proper_subset 展开的域")]
    fn slice_past_the_domain_of_its_segment_is_refused_when_the_plan_is_built() {
        let writes = writes_of_kinds(&[StepKind::JournalRecord, StepKind::JournalRecord]);
        let segments = vec![vec![0, 1]];
        let tearable = TearableInPlaceOverwrites::none(writes.len());
        let writes_with_torn_images =
            WritesWithTornImages::of(&MemoryPool::default(), &writes, &segments, &tearable);
        let slices = vec![Some(CrashPointReplaySegmentSlice {
            start_within_the_domain: StateCount1280Bits::from_u64(2),
            state_count: 2,
        })];
        let _plan = CrashStatePlan::new(
            &writes,
            &segments,
            &full_expansion,
            &slices,
            &tearable,
            &writes_with_torn_images,
        );
    }

    /// 单元区里一个槽的起点（崩溃点重放的池按 512 字节物理块建，单元区从 784 MiB 起）。
    const DISCARDED_UNIT_SLOT: u64 = 784 << 20;
    /// 一次 discard 罩住的字节数：一个 16 KiB 槽。
    const DISCARD_OF_ONE_SLOT_BYTES: u64 = 16384;

    /// 一次 discard（盘 0、`offset` 起 `length_in_bytes` 字节），写表里按「范围写零」记。
    fn discard_write(offset: u64, length_in_bytes: u64) -> RetainedWrite {
        RetainedWrite {
            device: DeviceIdentity(0),
            kind: StepKind::Discard,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(offset),
            contents: WrittenContents::Zeros {
                length: length_in_bytes,
            },
        }
    }

    /// 基镜像：盘 0 上 `DISCARDED_UNIT_SLOT` 起一整槽是旧的非零字节（discard 罩住的范围里原来有东西）。
    fn base_with_old_bytes_in_the_discarded_slot() -> MemoryPool {
        let mut base = MemoryPool::with_devices(&[DeviceIdentity(0)], 1 << 30);
        base.devices
            .get_mut(&DeviceIdentity(0))
            .expect("盘 0")
            .write(DeviceOffsetInBytes(DISCARDED_UNIT_SLOT), &[0x6Bu8; 16384]);
        base
    }

    /// 一次 16 KiB 的 discard 罩住非零旧内容：照 D13（验证路线） 已定项 4 原地覆写的字面取三态（判决 m4-batch2-code-r1 S6），
    /// 全量展开与甲二展开都是 1 + (3 − 1) 个状态；撕裂那一态的镜像那一槽前一半是 0、后一半还是旧字节，没持久那一态整槽是旧字节，
    /// 持久那一态整槽是 0。判据改回「discard 只取两态」时这一条红在「是原地覆写」那一句。
    #[test]
    fn discard_longer_than_a_sector_over_old_contents_enumerates_a_torn_third_state() {
        let base = base_with_old_bytes_in_the_discarded_slot();
        let writes = vec![discard_write(
            DISCARDED_UNIT_SLOT,
            DISCARD_OF_ONE_SLOT_BYTES,
        )];
        let segments = vec![vec![0]];
        let tearable = TearableInPlaceOverwrites::of(&base, &writes);
        assert!(
            tearable.contains(0),
            "16 KiB 的 discard 罩住非零旧内容：是原地覆写，多取第三态"
        );
        for expansion in [
            &full_expansion as &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
            &in_place_subsets_with_copy_on_write_none_or_all_expansion,
        ] {
            assert_eq!(
                crash_state_count_with_torn_in_place_overwrites(
                    &base, &writes, &segments, expansion
                ),
                3,
                "没持久、撕裂两个，再加全部持久那一个（全量展开与甲二展开相同）"
            );
        }
        let tables = crash_state_enumeration_tables(&base, &writes, &segments, &full_expansion);
        assert_eq!(tables.torn_images.len(), 1, "这次 discard 接一条撕裂镜像");
        let torn_image_index = tables.torn_images[0].torn_image_index;
        let mut expected_torn_bytes = vec![0x6Bu8; 16384];
        expected_torn_bytes[..8192].fill(0);
        assert_eq!(
            tables.enumerated_writes[torn_image_index].contents,
            WrittenContents::Bytes(expected_torn_bytes.clone()),
            "撕裂镜像：前一半清了、后一半还是旧字节"
        );
        let writes_with_torn_images =
            WritesWithTornImages::of(&base, &writes, &segments, &tearable);
        let plan = CrashStatePlan::new(
            &writes,
            &segments,
            &full_expansion,
            &no_segment_slices(segments.len()),
            &tearable,
            &writes_with_torn_images,
        );
        let torn_ordinal = (0..plan.state_count)
            .find(|ordinal| {
                let persisted = plan.persisted_writes_of_state(*ordinal);
                !persisted[0] && persisted[torn_image_index]
            })
            .expect("有一个状态是这次 discard 撕裂");
        let torn_state = CrashImage {
            base: &base,
            writes: &writes_with_torn_images.writes,
            persisted: plan.persisted_writes_of_state(torn_ordinal),
        };
        assert_eq!(
            PoolReader::read(
                &torn_state,
                DeviceIdentity(0),
                DeviceOffsetInBytes(DISCARDED_UNIT_SLOT),
                16384
            ),
            Some(expected_torn_bytes),
            "撕裂那一态的镜像真的生成出来：那一槽是撕裂镜像的字节"
        );
    }

    /// 不长于一个扇区的 discard、罩住的范围里原来全 0 的 discard 都只取两态（D13（验证路线） 已定项 4：不是原地覆写的写，撕裂并进「没持久」）：
    /// 两段各 2 − 1 个，再加全部持久那一个；写表后面不接撕裂镜像。
    #[test]
    fn discard_of_one_sector_or_over_zeros_takes_only_two_states() {
        let base = base_with_old_bytes_in_the_discarded_slot();
        let writes = vec![
            discard_write(DISCARDED_UNIT_SLOT, SECTOR_BYTES),
            discard_write(DISCARDED_UNIT_SLOT + (1 << 20), DISCARD_OF_ONE_SLOT_BYTES),
        ];
        let segments = vec![vec![0], vec![1]];
        let tearable = TearableInPlaceOverwrites::of(&base, &writes);
        assert_eq!(
            tearable.write_indexes(),
            Vec::<usize>::new(),
            "一个扇区的 discard 撕不开；罩住全 0 的 discard 原来没东西：都不是原地覆写"
        );
        assert_eq!(
            crash_state_count_with_torn_in_place_overwrites(
                &base,
                &writes,
                &segments,
                &full_expansion
            ),
            1 + 1 + 1
        );
        assert_eq!(
            crash_state_enumeration_tables(&base, &writes, &segments, &full_expansion)
                .enumerated_writes
                .len(),
            2,
            "写表后面不接撕裂镜像"
        );
    }
}
