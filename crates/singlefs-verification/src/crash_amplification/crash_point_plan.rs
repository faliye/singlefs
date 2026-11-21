//! 崩溃点与计划：一条流的崩溃点（[`PlannedCrashPoint`]）、按块排的计划（[`CrashPlan`]、[`plan_crash_points_with_slices`]）、
//! 每段放量到几次写（[`ExpandUpToWritesPerSegment`]）、流末那个点，与判器判一个点时另读的流程输入（`JudgeInputsOfTheFlow`）。

use super::*;

/// 一块最多几个状态：一块是一个 GPU 批、一次原子写、对账与分工的一个单位（里程碑二「增补 4」第十一项「一块取 2¹⁶ 个状态」）。
pub const STATES_PER_BLOCK: u64 = 1 << 16;

/// 放量的展开上限（一段的写数不超过它就展开全部真子集，超过就不展开）：这就是全量域，环境里没设
/// [`EXPAND_UP_TO_ENVIRONMENT_VARIABLE`] 时取它。
pub const EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION: usize = 30;

/// 缩崩溃域的环境变量（指定范围的一种「允许缩域」）：设了就按它缩，只往小缩——放量取它与
/// [`EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION`] 里小的那个，崩溃枚举用例取它与用例自己的展开里小的那个
/// （[`ExpandUpToWritesPerSegment::narrowed_expansion`]）。缩过域判的块与全量的块键不同（块键含段展开），各存各的。
pub const EXPAND_UP_TO_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO";

/// 展开上限：一段的写数不超过它，这一段展开全部真子集（[`CrashPointReplaySegmentExpansion::EveryProperSubset`]）；超过就不展开。
/// 每一段都不超过它，这一趟就是全域（`exhaustive=true`）。放量的那一个是 [`EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpandUpToWritesPerSegment(pub usize);

impl ExpandUpToWritesPerSegment {
    /// 放量这一趟的展开上限：[`EXPAND_UP_TO_ENVIRONMENT_VARIABLE`] 没设取引擎常量（全量域），设了取它与常量里小的那个。
    ///
    /// # Panics
    /// 设了而认不出（不是非负整数、空串、不是 UTF-8）：写明变量名与值，不悄悄退回全量域（缩域写错照样跑完、照样退 0，
    /// 跑的就不是要的那个域）。
    #[must_use]
    pub fn of_the_amplification_from_the_environment() -> Self {
        Self::of_the_amplification_and_the_variable_value(
            process_environment_variable(EXPAND_UP_TO_ENVIRONMENT_VARIABLE).as_deref(),
        )
    }

    /// 变量的值（`None` 是没设）换成放量的展开上限：解析经数值环境变量共用的那一份
    /// （[`number_from_the_variable_value_panicking_when_unreadable`]），没设取引擎常量，设了只往小缩。
    ///
    /// # Panics
    /// 设了而不是非负整数：消息带变量名与原值。
    #[must_use]
    pub fn of_the_amplification_and_the_variable_value(value: Option<&str>) -> Self {
        Self(Self::narrowing_of_the_variable_value(value).map_or(
            EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION,
            |narrowed| {
                narrowed
                    .0
                    .min(EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION)
            },
        ))
    }

    /// 环境里缩域的那个上限：[`EXPAND_UP_TO_ENVIRONMENT_VARIABLE`] 没设是 `None`（不缩）。
    ///
    /// # Panics
    /// 同 [`Self::of_the_amplification_and_the_variable_value`]。
    #[must_use]
    pub fn narrowing_from_the_environment() -> Option<Self> {
        Self::narrowing_of_the_variable_value(
            process_environment_variable(EXPAND_UP_TO_ENVIRONMENT_VARIABLE).as_deref(),
        )
    }

    /// 变量的值换成缩域的上限（`None` 是没设）。
    ///
    /// # Panics
    /// 设了而不是非负整数：消息带变量名与原值。
    fn narrowing_of_the_variable_value(value: Option<&str>) -> Option<Self> {
        number_from_the_variable_value_panicking_when_unreadable::<usize>(
            EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
            value,
        )
        .map(Self)
    }

    /// 一段照 `own_expansion`（用例自己的展开）与这个上限里小的那个展开：写数超过上限的段不展开，没超过的照用例自己的
    /// （全部真子集、甲二、不展开三种，从大到小）；缩只往小缩，用例本来不展开的照样不展开。
    #[must_use]
    pub fn narrowed_expansion(
        self,
        own_expansion: CrashPointReplaySegmentExpansion,
        segment: &[usize],
    ) -> CrashPointReplaySegmentExpansion {
        match self.expansion_of(segment) {
            CrashPointReplaySegmentExpansion::NotExpanded => {
                CrashPointReplaySegmentExpansion::NotExpanded
            }
            CrashPointReplaySegmentExpansion::EveryProperSubset
            | CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
                own_expansion
            }
        }
    }

    /// 一段按这个上限怎么展开。
    #[must_use]
    pub fn expansion_of(self, segment: &[usize]) -> CrashPointReplaySegmentExpansion {
        if segment.len() <= self.0 {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    }

    /// 每一段都展开到了（全域）。
    #[must_use]
    pub fn covers_every_segment(self, segments: &[Vec<usize>]) -> bool {
        segments.iter().all(|segment| segment.len() <= self.0)
    }
}

/// 一条流程里的一个崩溃点：名称（绑在设它的函数上）与它在写表里发出的写（下标区间）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashPointSpan {
    pub name: String,
    /// 设这个点的函数所在的文件（仓内路径）。共用前缀的点写在公共模块里：几条流调同一个函数，就得到同一个身份、同一个库里的节点。
    pub code_file_in_repository: String,
    pub writes: Range<usize>,
    /// 测试代码里预设的 ignore：这个点的段一律不展开（只留全持久那一个状态），路径照样往下走。
    pub ignored: bool,
}

/// 一条录好的测试流程：崩溃枚举要的五样，加上各点（名字、设它的文件、写区间；按流程次序、首尾相接）。
/// 版本表是两样：带文件的那几版（`versions`）与文件从那一版起没有了的那几版（`versions_without_a_file`，删除流的意图那一版，
/// D8（核心索引结构） 已定项 16 第 4 条；文件从不消失的流是空的），两个判器都按 [`CrashPointReplayVersionTable`] 认（R 报告「停下交主 agent」第一项）。
/// 同一个根两边都有是用例写错了：判的时候 [`CrashPointReplayVersionTable::new`] 停下。
pub struct CrashFlow<'flow> {
    pub base: &'flow MemoryPool,
    pub writes: &'flow [RetainedWrite],
    pub segments: &'flow [Vec<usize>],
    pub judged_root_index: usize,
    pub versions: &'flow [PublishedVersion],
    pub versions_without_a_file: &'flow [PublishedVersionWithoutAFile],
    pub expansion: &'flow (dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
    pub crash_points: Vec<CrashPointSpan>,
}

impl CrashFlow<'_> {
    /// 判器读的版本表（[`CrashPointReplayVersionTable::new`]）。
    #[must_use]
    pub fn version_table(&self) -> CrashPointReplayVersionTable<'_> {
        CrashPointReplayVersionTable::new(self.versions, self.versions_without_a_file)
    }
}

/// 一个崩溃点在这一趟里的计划：它管哪几段、这几段的状态在整条流里的序号区间、它的复用键与路径编号、分成几块。
pub struct PlannedCrashPoint {
    pub name: String,
    pub ignored: bool,
    pub segments: Vec<usize>,
    /// 这个点管的状态在整条流里的序号（首尾相接的一段；一段不展开时可能是空的）。
    pub ordinals: Range<u64>,
    pub reuse_key: ReuseKey,
    /// 从根到这个点的路径：每个点的名字（`<流程>.<周期>.<步>`）按次序用 `/` 接起来。同名同数据的前缀在几条流里是同一串；
    /// 名字相同、数据不同的分叉在路径这一级并成一段，靠身份（复用键里的父镜像与写表）区分。
    pub path_number: String,
    /// 身份 `<仓内路径>::<崩溃点名称>::<复用键>`：只作索引（路径 → 身份记在库的路径索引表里、进覆盖报告），不进块键：
    /// 两个设点文件、两个名字下内容相同的点是同一把块键。
    pub identity: String,
    /// 判器判这个点的状态时另读的流程输入的摘要（`JudgeInputsOfTheFlow::digest_for_a_point`）；进 `plan_hash`。
    pub judge_inputs: [u8; DIGEST_BYTES],
    /// 点内每段的状态数与展开、判器另读的输入，加上按发布分的态数要往下游找的那几段各归哪次发布（`publishes_looked_up_downstream_of_the_point`）。
    /// 分片的段（C790）只写它整域的态数，不写这一趟的片起点与片长：几趟之间计划哈希不变。
    pub plan_hash: [u8; DIGEST_BYTES],
    /// 这个点管分片的段时（C790）它的状态按段切成的几截，按点内序号首尾相接；不管分片的段的点是空的（块照整点切，块键与分片之前逐个相同）。
    pub pieces_around_the_slices: Vec<PieceOfAPointWithASlice>,
}

/// 管分片的段的点里的一截状态（点内序号区间）：几段不分片的段连成一截，或一段分片的段自成一截。块按截切，块键按截取：
/// 分片的那一截锚在段内的绝对编号上（片起点 + 片内序号），几趟之间片起点、片长怎么变，同一个编号的块都是同一个块键；
/// 不分片的那几截锚在截内序号上（截号进计划哈希那一格），片长变了它们在点内的位置挪了，块键也不变。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PieceOfAPointWithASlice {
    pub local_states: Range<u64>,
    pub kind: KindOfAPiece,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::large_enum_variant,
    reason = "一个点至多几截，分片那一截的两个 1280 位数按值存，比装箱多占的只是几百字节；读块键时不用再解一层引用"
)]
pub enum KindOfAPiece {
    /// 几段不分片的段：这个点里第几截这样的截（从 0 起）。
    SegmentsNotSliced { piece_index: usize },
    /// 一段分片的段：流里的段号、它在点内的段序、这一趟的片起点、整域的态数。
    SlicedSegment {
        segment_index: usize,
        segment_in_point: usize,
        start_within_the_domain: StateCount1280Bits,
        domain_state_count: StateCount1280Bits,
    },
}

impl PlannedCrashPoint {
    /// 这个点的块：点内序号区间（从 0 起），每块不超过 [`STATES_PER_BLOCK`] 个状态。管分片的段的点按截切（[`PieceOfAPointWithASlice`]），
    /// 每截从截头起切，最后一块可以不满。
    #[must_use]
    pub fn blocks(&self) -> Vec<Range<u64>> {
        let blocks_of = |states: &Range<u64>| {
            let count = states.end - states.start;
            (0..count.div_ceil(STATES_PER_BLOCK))
                .map(|index| {
                    states.start + index * STATES_PER_BLOCK
                        ..states.start + ((index + 1) * STATES_PER_BLOCK).min(count)
                })
                .collect::<Vec<Range<u64>>>()
        };
        if self.pieces_around_the_slices.is_empty() {
            return blocks_of(&(0..self.ordinals.end - self.ordinals.start));
        }
        self.pieces_around_the_slices
            .iter()
            .flat_map(|piece| blocks_of(&piece.local_states))
            .collect()
    }

    /// 点内序号 `local_start` 落在哪一截（管分片的段的点）。
    ///
    /// # Panics
    /// 这个点不管分片的段，或 `local_start` 不在任何一截里（块不是 [`Self::blocks`] 切出来的）。
    fn piece_of(&self, local_start: u64) -> &PieceOfAPointWithASlice {
        self.pieces_around_the_slices
            .iter()
            .find(|piece| piece.local_states.contains(&local_start))
            .expect("块是按截切的，块头落在某一截里")
    }

    /// `local` 这一块是不是分片的段里的块（录入时块长可以随片长变，[`VerdictStore::record_block_of_a_slice`]）。
    #[must_use]
    pub fn is_a_block_of_a_sliced_segment(&self, local: &Range<u64>) -> bool {
        !self.pieces_around_the_slices.is_empty()
            && matches!(
                self.piece_of(local.start).kind,
                KindOfAPiece::SlicedSegment { .. }
            )
    }

    /// `local` 这一块的块键。不分片的块（整点切的与管分片的段的点里不分片的那几截）块长进计划哈希那一格
    /// （[`plan_hash_with_the_length_of_the_block`]）：同一块起点、块长不同的两块不是同一把块键，不互相复用。分片的段里的块
    /// （[`Self::block_key_of_a_slice_block`]）块长随片长变、块键不带块长：录入表记着每块的块长，录过、判过的块改不了块长
    /// （`VerdictStore::record_block_of_a_slice`），续点照录入表里的块长往后走，判过的块不会被一块更长的同起点块复用。
    pub fn block_key(&self, local: &Range<u64>) -> VerdictBlockKey {
        if self.pieces_around_the_slices.is_empty() {
            return self.block_key_under(
                plan_hash_with_the_length_of_the_block(&self.plan_hash, local),
                local.start,
            );
        }
        let piece = self.piece_of(local.start);
        let offset_in_the_piece = local.start - piece.local_states.start;
        match &piece.kind {
            KindOfAPiece::SegmentsNotSliced { piece_index } => {
                let mut message = self.plan_hash.to_vec();
                message.extend_from_slice(format!("piece_not_sliced={piece_index}\n").as_bytes());
                self.block_key_under(
                    plan_hash_with_the_length_of_the_block(&sha256_digest(&message), local),
                    offset_in_the_piece,
                )
            }
            KindOfAPiece::SlicedSegment {
                segment_in_point,
                start_within_the_domain,
                ..
            } => self.block_key_of_a_slice_block(
                *segment_in_point,
                &start_within_the_domain
                    .checked_add_u64(offset_in_the_piece)
                    .expect("片里的编号不越过整域，装得进 1280 位"),
            ),
        }
    }

    /// 点内第 `segment_in_point` 段（分片的段）里从段内编号 `start_within_the_domain` 起的那一块的块键：块起点那 8 字节是编号的低 64 位，
    /// 高位与段序进计划哈希那一格。只由计划哈希（不带片起点与片长）与编号定，几趟之间片怎么切都对得上。
    #[must_use]
    pub fn block_key_of_a_slice_block(
        &self,
        segment_in_point: usize,
        start_within_the_domain: &StateCount1280Bits,
    ) -> VerdictBlockKey {
        let (above_sixty_four_bits, low_sixty_four_bits) =
            start_within_the_domain.split_at_sixty_four_bits();
        let mut message = self.plan_hash.to_vec();
        message.extend_from_slice(
            format!(
                "sliced_segment_in_point={segment_in_point} block_start_above_sixty_four_bits={}\n",
                above_sixty_four_bits.decimal_text()
            )
            .as_bytes(),
        );
        self.block_key_under(sha256_digest(&message), low_sixty_four_bits)
    }

    /// 块键只由内容定：段写表与起步镜像（复用键）、计划哈希（段展开、判器另读的输入、态数）、段内序号；设点文件、点名、身份都不进。
    fn block_key_under(&self, plan_hash: [u8; DIGEST_BYTES], block_start: u64) -> VerdictBlockKey {
        VerdictBlockKey {
            node_input_digest: NodeInputDigest(*self.reuse_key.as_bytes()),
            enumeration_plan_hash: EnumerationPlanHash(plan_hash),
            block_start: BlockStartStateIndex(block_start),
        }
    }

    /// 分片的段（C790）在这个点里的内容摘要：分片进度按它记（不按流名、段号）。点的复用键、计划哈希与这一段在点内的段序拼起来取 SHA-256。
    #[must_use]
    pub fn sliced_segment_content_digest(&self, segment_in_point: usize) -> [u8; DIGEST_BYTES] {
        let mut message = self.reuse_key.as_bytes().to_vec();
        message.extend_from_slice(&self.plan_hash);
        message
            .extend_from_slice(format!("sliced_segment_in_point={segment_in_point}\n").as_bytes());
        sha256_digest(&message)
    }
}

/// 不分片的块的块键里计划哈希那一格：点（或截）的计划哈希接上这一块的块长取 SHA-256。块长不进键时，块宽（[`STATES_PER_BLOCK`]）加大一倍，
/// 新块与起点相同的旧块同一把块键，判段问复用只问块在不在、红不红，新块后一半从没判过的状态也算作复用的绿。
fn plan_hash_with_the_length_of_the_block(
    plan_hash: &[u8; DIGEST_BYTES],
    local: &Range<u64>,
) -> [u8; DIGEST_BYTES] {
    let mut message = plan_hash.to_vec();
    message
        .extend_from_slice(format!("states_in_the_block={}\n", local.end - local.start).as_bytes());
    sha256_digest(&message)
}

/// 一条流程的整份计划：各点，加上每段归哪个点（ignore 的点的段不展开，判段怎么展开时要它）、每段这一趟的片（C790）。
pub struct CrashPlan {
    pub points: Vec<PlannedCrashPoint>,
    pub(super) owner_of_segment: Vec<usize>,
    pub(super) segment_slices: Vec<Option<CrashPointReplaySegmentSlice>>,
}

impl CrashPlan {
    /// 每段这一趟的片（与段同序，不分片的段是 `None`）：计划、事实表、判表、判与核对都照这一份。
    #[must_use]
    pub fn segment_slices(&self) -> &[Option<CrashPointReplaySegmentSlice>] {
        &self.segment_slices
    }

    /// 这一趟分片的段：（点号，那一截）。
    #[must_use]
    pub fn sliced_segments(&self) -> Vec<(usize, &PieceOfAPointWithASlice)> {
        self.points
            .iter()
            .enumerate()
            .flat_map(|(point_index, point)| {
                point
                    .pieces_around_the_slices
                    .iter()
                    .filter(|piece| matches!(piece.kind, KindOfAPiece::SlicedSegment { .. }))
                    .map(move |piece| (point_index, piece))
            })
            .collect()
    }

    /// 一段怎么展开：归 ignore 的点就不展开，其余照流程自己的展开方式。计划与核对都经它，两处不会不一致。
    #[must_use]
    pub fn segment_expansion(
        &self,
        flow: &CrashFlow<'_>,
        segment_index: usize,
        segment: &[usize],
    ) -> CrashPointReplaySegmentExpansion {
        if self.points[self.owner_of_segment[segment_index]].ignored {
            CrashPointReplaySegmentExpansion::NotExpanded
        } else {
            (flow.expansion)(segment_index, segment)
        }
    }
}

/// 每段归哪个点：含这一段最后一次写的那个点。一段跨了两个点（上一次发布的系统配置槽写与下一次发布的单元写没被屏障隔开时）归后面
/// 那个点：那一段里「上一个点收尾的几次写没落」的状态，是走进后面那个点的过程；上游的点只看自己与更前面的写，不因下游的改动而变
/// （输入决定从这个点往后要不要重验）。
pub(super) fn owner_of_each_segment(flow: &CrashFlow<'_>) -> Vec<usize> {
    flow.segments
        .iter()
        .map(|segment| {
            let last = segment.iter().copied().max().expect("段里至少一次写");
            flow.crash_points
                .iter()
                .position(|point| point.writes.contains(&last))
                .expect("每次写都落在某个崩溃点里")
        })
        .collect()
}

/// 给一条流程的每个崩溃点定计划：含它最后一次写的段归它（[`owner_of_each_segment`]）；写表按它管的那几段取、父输出是它第一次管到的写
/// 之前的镜像；状态序号按点的次序首尾相接地划分整条流；ignore 的点的段不展开。最后那个全持久状态不归最后一个点，另立成这条流自己的
/// 末节点（[`END_OF_FLOW_NODE_NAME`]，计划里最后一项）：它的写表是空的、父输出是整张写表叠完的镜像、设点文件取最后一个点的
/// （「多记录流与删除流共享到 C」，主 agent 定）。最后一个点的计划因此只由它自己与它之前的写定，
/// 一条流在别的流的前缀上收尾时，那几个点的块键与接着往下走的流逐个相同——按发布分的态数要往下游找的点除外
/// （[`publishes_looked_up_downstream_of_the_point`]：那几段归的发布，也就是下游第一次根槽写的位置与身份或「后面没有根槽写」，进计划哈希；
/// 「乙」）：多记录流的 C 尾（C 那次发布根槽写之后的系统配置槽轮换）后面没有根槽写，删除流的 C 尾归意图那次发布，
/// 两条流在这一块上不共享，C 尾之前的点照旧共享。
/// 计划不看判法：镜像判定的结论按判定定义那一把判法单元键存（[`ConclusionVersionsOfThePlan`]，GPU 判的与 CPU 判的同一把），改了判法树与块键都不动。判器另读的流程输入（`JudgeInputsOfTheFlow`）
/// 进每个点的计划哈希，不进复用键与身份。
///
/// # Panics
/// 崩溃点的写区间不是首尾相接地罩住整张写表；某个点的名称或文件路径不合身份的规矩（同一个名字在一条流程里可以出现几次：
/// 同一个函数调几次就是几个点，树上的位置分得开）；
/// 各段的状态序号接不上（枚举计划与切段不一致，是这里的 bug）；
/// 对基镜像本身照 journal 恢复，落到的根比基镜像里最新的那条根晚（B0 的前提，
/// `assert_journal_recovery_of_the_base_image_lands_no_later_than_its_newest_root`）。
#[must_use]
pub fn plan_crash_points(flow: &CrashFlow<'_>) -> CrashPlan {
    plan_crash_points_with_slices(flow, &no_segment_slices(flow.segments.len()))
}

/// 同 [`plan_crash_points`]，分片的段照 `segment_slices`（与段同序，C790）：它在流里的序号区间是片长；管它的点按截切块
/// （[`PieceOfAPointWithASlice`]），计划哈希里它只写整域的态数，不写片起点与片长。
///
/// # Panics
/// 同 [`plan_crash_points`]；`segment_slices` 与段数不同；片越过段的域，或分片的段整域越过 1280 位。
#[must_use]
pub fn plan_crash_points_with_slices(
    flow: &CrashFlow<'_>,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
) -> CrashPlan {
    let mut expected_start = 0usize;
    for point in &flow.crash_points {
        assert_eq!(
            point.writes.start, expected_start,
            "崩溃点 {} 的写区间没有接上前一个点",
            point.name
        );
        expected_start = point.writes.end;
    }
    assert_eq!(
        expected_start,
        flow.writes.len(),
        "崩溃点的写区间没有罩住整张写表"
    );
    let owner_of_segment = owner_of_each_segment(flow);
    let ignored_points: Vec<bool> = flow
        .crash_points
        .iter()
        .map(|point| point.ignored)
        .collect();
    let expansion = |segment_index: usize, segment: &[usize]| {
        if ignored_points[owner_of_segment[segment_index]] {
            CrashPointReplaySegmentExpansion::NotExpanded
        } else {
            (flow.expansion)(segment_index, segment)
        }
    };
    let (ranges_by_segment, state_count) = crash_state_ranges_by_segment_with_slices(
        flow.base,
        flow.writes,
        flow.segments,
        &expansion,
        segment_slices,
    );
    // 分片的段整域的态数（进计划哈希，不写片起点与片长）；不分片的段是 None
    let tearable = segment_slices
        .iter()
        .any(Option::is_some)
        .then(|| TearableInPlaceOverwrites::of(flow.base, flow.writes));
    let domain_of_each_sliced_segment: Vec<Option<StateCount1280Bits>> = flow
        .segments
        .iter()
        .enumerate()
        .map(|(segment_index, segment)| {
            segment_slices[segment_index].map(|_slice| {
                expansion(segment_index, segment)
                    .checked_state_count(
                        flow.writes,
                        segment,
                        tearable
                            .as_ref()
                            .expect("有分片的段才进到这里，可撕表建过了"),
                    )
                    .expect("分片的段整域的态数装得进 1280 位：录入段在写库之前核过")
            })
        })
        .collect();
    // 一段有没有状态：分片的段照它整域算（片走完了、这一趟片长是 0 也算有），计划哈希几趟之间不变
    let segment_has_states: Vec<bool> = ranges_by_segment
        .iter()
        .zip(&domain_of_each_sliced_segment)
        .map(|(range, domain)| match domain {
            Some(domain) => !domain.is_zero(),
            None => !range.is_empty(),
        })
        .collect();
    let publish_of_segment = publish_of_each_segment(flow.writes, flow.segments);
    let expansion_names: Vec<&'static str> = flow
        .segments
        .iter()
        .enumerate()
        .map(|(segment_index, segment)| expansion(segment_index, segment).name())
        .collect();
    let judge_inputs_of_the_flow = JudgeInputsOfTheFlow::of(flow);
    let mut parent_image = SectorImage::of_base(flow.base);
    let mut applied_writes = 0usize;
    let mut path_segments: Vec<String> = Vec::new();
    let mut next_ordinal = 0u64;
    let mut planned = Vec::new();
    for (point_index, point) in flow.crash_points.iter().enumerate() {
        let owned: Vec<usize> = (0..flow.segments.len())
            .filter(|segment| owner_of_segment[*segment] == point_index)
            .collect();
        let owned_writes: BTreeSet<usize> = owned
            .iter()
            .flat_map(|segment| flow.segments[*segment].iter().copied())
            .collect();
        let first_write = owned_writes.first().copied().unwrap_or(point.writes.start);
        let last_write = owned_writes.last().map_or(first_write, |last| last + 1);
        assert!(
            first_write >= applied_writes,
            "崩溃点 {} 管的写在前一个点已经叠进父镜像的写之前",
            point.name
        );
        parent_image.overlay(&flow.writes[applied_writes..first_write]);
        applied_writes = first_write;
        let owned_segments: Vec<Vec<usize>> = owned
            .iter()
            .map(|segment| flow.segments[*segment].clone())
            .collect();
        let attributes = StepAttributes {
            write_table_with_barriers: step_write_table_with_barriers(
                flow.writes,
                &owned_segments,
                first_write..last_write,
            ),
            parent_output: parent_image.digest(),
        };
        let reuse_key = attributes.reuse_key();
        let label =
            NodeLabel::new(&point.code_file_in_repository, &point.name).expect("崩溃点标签合规矩");
        let identity = label.identity_text(&reuse_key);
        assert!(
            !point.name.contains('/') && !point.name.contains('|'),
            "崩溃点名字 {} 里不许有 / 与 |：它是路径段，也是节点名的一截",
            point.name
        );
        path_segments.push(point.name.clone());
        let path_number = path_segments.join("/");
        let start_ordinal = next_ordinal;
        let mut plan_text = String::new();
        let mut pieces_around_the_slices: Vec<PieceOfAPointWithASlice> = Vec::new();
        let point_has_a_sliced_segment = owned
            .iter()
            .any(|segment| segment_slices[*segment].is_some());
        let mut states_outside_the_slices = 0u64;
        for (segment_in_point, segment) in owned.iter().enumerate() {
            let range = &ranges_by_segment[*segment];
            assert_eq!(
                range.start, next_ordinal,
                "段 {segment} 的状态序号没有接上前一段"
            );
            next_ordinal = range.end;
            let local_states = range.start - start_ordinal..range.end - start_ordinal;
            // 计划哈希只写点内的段序，不写流里的段序：同一个前缀节点在几条流里都是同一份计划
            match (
                &segment_slices[*segment],
                &domain_of_each_sliced_segment[*segment],
            ) {
                (Some(slice), Some(domain)) => {
                    // 分片的段只写整域的态数：片起点与片长不进计划哈希，几趟之间块键对得上
                    plan_text.push_str(&format!(
                        "segment={segment_in_point} sliced_domain_states={} expansion={}\n",
                        domain.decimal_text(),
                        expansion_names[*segment]
                    ));
                    pieces_around_the_slices.push(PieceOfAPointWithASlice {
                        local_states,
                        kind: KindOfAPiece::SlicedSegment {
                            segment_index: *segment,
                            segment_in_point,
                            start_within_the_domain: slice.start_within_the_domain,
                            domain_state_count: *domain,
                        },
                    });
                }
                (None, _) | (Some(_), None) => {
                    plan_text.push_str(&format!(
                        "segment={segment_in_point} states={} expansion={}\n",
                        range.end - range.start,
                        expansion_names[*segment]
                    ));
                    states_outside_the_slices += range.end - range.start;
                    if point_has_a_sliced_segment {
                        match pieces_around_the_slices.last_mut() {
                            Some(PieceOfAPointWithASlice {
                                local_states: piece_states,
                                kind: KindOfAPiece::SegmentsNotSliced { .. },
                            }) => piece_states.end = local_states.end,
                            Some(PieceOfAPointWithASlice {
                                kind: KindOfAPiece::SlicedSegment { .. },
                                ..
                            })
                            | None => {
                                let piece_index = pieces_around_the_slices
                                    .iter()
                                    .filter(|piece| {
                                        matches!(piece.kind, KindOfAPiece::SegmentsNotSliced { .. })
                                    })
                                    .count();
                                pieces_around_the_slices.push(PieceOfAPointWithASlice {
                                    local_states,
                                    kind: KindOfAPiece::SegmentsNotSliced { piece_index },
                                });
                            }
                        }
                    }
                }
            }
        }
        // 按发布分的态数要往下游找的段：它归的那次发布（下游第一次根槽写的位置与身份，或后面没有根槽写）进计划哈希，
        // 下游不同的两条流在这几块上不共享；不看下游的点计划文本不变，照旧共享
        let publishes_looked_up_downstream = publishes_looked_up_downstream_of_the_point(
            &owned,
            &owned_writes,
            &publish_of_segment,
            &segment_has_states,
        );
        for (segment_in_point, publish) in &publishes_looked_up_downstream {
            plan_text.push_str(&format!(
                "segment={segment_in_point} publish_looked_up_downstream={}\n",
                publish.name()
            ));
        }
        let ordinals = start_ordinal..next_ordinal;
        if point_has_a_sliced_segment {
            // 点的状态数随片长变：只写分片的段之外的（分片的段的整域已写在它那一行）
            plan_text.push_str(&format!(
                "states_outside_the_slices={states_outside_the_slices}\n"
            ));
        } else {
            plan_text.push_str(&format!("states={}\n", ordinals.end - ordinals.start));
        }
        // 判器另读的输入进计划哈希、不进复用键：树（身份、路径）照旧只由流程的这一步与它起步的镜像定，判定块按判器真读到的输入分
        let judge_inputs =
            judge_inputs_of_the_flow.digest_for_a_point(flow, first_write, last_write);
        plan_text.push_str(&format!(
            "judge_inputs={}\n",
            hexadecimal_text(&judge_inputs)
        ));
        planned.push(PlannedCrashPoint {
            name: point.name.clone(),
            ignored: point.ignored,
            segments: owned,
            ordinals,
            reuse_key,
            identity,
            path_number,
            judge_inputs,
            plan_hash: sha256_digest(plan_text.as_bytes()),
            pieces_around_the_slices,
        });
    }
    planned.push(end_of_flow_node(
        flow,
        &judge_inputs_of_the_flow,
        parent_image,
        applied_writes,
        &mut path_segments,
        next_ordinal..state_count,
    ));
    CrashPlan {
        points: planned,
        owner_of_segment,
        segment_slices: segment_slices.to_vec(),
    }
}

/// 一个点管的段里，按发布分的态数要往下游找的那几段：这一段归的那次发布（[`publish_of_each_segment`]，崩溃点重放计数与崩溃放量计数向量
/// 同一份归法）不是这个点自己管的写里的根槽写，而是这个点之后第一次根槽写，或者后面没有根槽写可归
/// （[`CrashPointReplayPublishOfState::AfterTheLastRoot`]）。交回（段在点内的序号，它归的那次发布），按点内的段序；一个状态都没有的段不算
/// （它不进计数向量）。同一个前缀节点在两条流里下游不同，这几段的态数就归到不同的发布上，块键得分开（「乙」：
/// 看下游的块把后面第一次根槽写也算进块键，其余块照旧共享）。
///
/// # Panics
/// 归法给某一段交回了 [`CrashPointReplayPublishOfState::EveryWritePersisted`]：那一格只给整条流最后全部持久的那一个状态，不给段。
fn publishes_looked_up_downstream_of_the_point(
    owned_segments: &[usize],
    owned_writes: &BTreeSet<usize>,
    publish_of_segment: &[CrashPointReplayPublishOfState],
    segment_has_states: &[bool],
) -> Vec<(usize, CrashPointReplayPublishOfState)> {
    owned_segments
        .iter()
        .enumerate()
        .filter(|(_segment_in_point, segment)| segment_has_states[**segment])
        .filter_map(|(segment_in_point, segment)| {
            let publish = publish_of_segment[*segment];
            let is_looked_up_downstream = match publish {
                CrashPointReplayPublishOfState::UpToTheRootOf {
                    root_write_index, ..
                } => !owned_writes.contains(&root_write_index),
                CrashPointReplayPublishOfState::AfterTheLastRoot => true,
                CrashPointReplayPublishOfState::EveryWritePersisted => panic!(
                    "publish_of_each_segment 只交回根槽写那一格或 after_the_last_root，段 {segment} 归成了整条流最后全部持久那一个状态"
                ),
            };
            is_looked_up_downstream.then_some((segment_in_point, publish))
        })
        .collect()
}

/// 末节点的名字（[`plan_crash_points`]）：每条流最后全部持久的那一个状态单独一个节点。
pub const END_OF_FLOW_NODE_NAME: &str = "end_of_flow";

/// 一条流的末节点（[`plan_crash_points`]）：管各段之后剩下的那一个状态（整张写表全部持久），写表是空的、父输出是 `parent_image` 叠上
/// `applied_writes` 之后全部写的镜像，设点文件取最后一个点的；判器另读的输入按「这个点在写表末尾」取。计划哈希只写它的状态数与判器另读的输入。
///
/// # Panics
/// 流里一个崩溃点都没有；各段之后剩下的不是恰好一个状态（`crash_state_ranges_by_segment` 交回的状态数是各段之和加一）。
fn end_of_flow_node(
    flow: &CrashFlow<'_>,
    judge_inputs_of_the_flow: &JudgeInputsOfTheFlow,
    mut parent_image: SectorImage,
    applied_writes: usize,
    path_segments: &mut Vec<String>,
    ordinals: Range<u64>,
) -> PlannedCrashPoint {
    assert_eq!(
        ordinals.end - ordinals.start,
        1,
        "各段之后只剩整条流最后全部持久的那一个状态：各点的序号区间加末节点要正好是整条流的状态数"
    );
    let last_point = flow
        .crash_points
        .last()
        .expect("一条流至少一个崩溃点：末节点的设点文件取最后一个点的");
    parent_image.overlay(&flow.writes[applied_writes..]);
    let write_count = flow.writes.len();
    let attributes = StepAttributes {
        write_table_with_barriers: step_write_table_with_barriers(
            flow.writes,
            &[],
            write_count..write_count,
        ),
        parent_output: parent_image.digest(),
    };
    let reuse_key = attributes.reuse_key();
    let label = NodeLabel::new(&last_point.code_file_in_repository, END_OF_FLOW_NODE_NAME)
        .expect("最后一个点的设点文件合规矩，末节点的名字不空");
    let identity = label.identity_text(&reuse_key);
    path_segments.push(END_OF_FLOW_NODE_NAME.to_string());
    let judge_inputs = judge_inputs_of_the_flow.digest_for_a_point(flow, write_count, write_count);
    let plan_text = format!(
        "end_of_flow states={}\njudge_inputs={}\n",
        ordinals.end - ordinals.start,
        hexadecimal_text(&judge_inputs)
    );
    PlannedCrashPoint {
        name: END_OF_FLOW_NODE_NAME.to_string(),
        ignored: false,
        segments: Vec::new(),
        ordinals,
        reuse_key,
        path_number: path_segments.join("/"),
        identity,
        judge_inputs,
        plan_hash: sha256_digest(plan_text.as_bytes()),
        pieces_around_the_slices: Vec::new(),
    }
}

// ============================== 判器另读的流程输入：进计划哈希 ==============================

/// 写表里的一次写在判器输入里的写法：盘、种类、FUA、偏移、长度、内容摘要（与 `step_write_table_with_barriers` 同一套字段），
/// 后面接它在整条流里的段号（不接的写 `-`）。
fn judge_input_line_of_a_write(
    write: &RetainedWrite,
    segment_in_the_flow: Option<usize>,
) -> String {
    let segment_text =
        segment_in_the_flow.map_or_else(|| "-".to_string(), |segment| segment.to_string());
    format!(
        "W {} {} {} {} {} {} {segment_text}\n",
        write.device.0,
        write.kind.name(),
        u8::from(write.is_force_unit_access),
        write.offset.0,
        write.length_in_bytes(),
        content_digest_text(&write.contents),
    )
}

/// 一个点最后一次写之后的写表，判器判这个点的状态时读到多少（由整条流的根槽写定）。
enum WritesAfterThePoint {
    /// 根槽写的 txg 沿写表严格递增：之后的写只有「没做完的那次发布」的根槽写读得到（判据见 `JudgeInputsOfTheFlow`）。
    ReadOnlyThroughTheUnfinishedPublish,
    /// 根槽写的 txg 不是严格递增：之后的写都可能读到。第 k 项是写表第 k 次写往后逐行链起来的摘要（`k` 从 0 到写数）。
    MayAllBeRead {
        suffix_digests: Vec<[u8; DIGEST_BYTES]>,
    },
}

/// 判器（CPU 的 `crash::judge_crash_image` 与 GPU 判器，读的是同一份 [`CrashFlow`]）判一个点的状态时，复用键（这一步带屏障的写表、
/// 起步镜像的扇区终值）之外还读到的流程输入。R1（`research/prompts/test-script-align-code-r1-main-verification.md`「二、判定与去向」）：
/// 判器读到的每一样都进那一块的键，只差这些的两条流不互相复用；前缀共享在判器读到的真相同时保留。逐样：
///
/// - 这个点之前的写表（不只是扇区终值）：记录核对器按写表分发布、按写的种类找根槽写与系统配置槽写
///   （`crash::check_records_against`、`crash::reuse_is_not_proven_illegal_by_the_reclaim_predicate`），种类、次序变了而扇区终值相同时判定会变。
///   按（写、整条流里的段号）逐行链起来取摘要。
/// - 版本表：oracle 只按恢复落到的根（实例，txg）找那一版、找比它旧的版本（`crash::classified_oracle_violation_for_the_version_table`）；
///   「没有文件的那几版」（`CrashFlow::versions_without_a_file`）与带文件的那几版按同一个上界进键，一行都没有时一个字都不加
///   （文件从不消失的流键不变）。
///   落到的根要么是不晚于基镜像里最新那条根的根（基镜像里带根 magic 的扇区；基镜像自己的 journal 不越过其中最新那条，排计划时断言：
///   `assert_journal_recovery_of_the_base_image_lands_no_later_than_its_newest_root`），要么是写表里到这个点为止写过的根槽写，要么是 journal 重放出来的
///   那一版——没做完的那次发布，它的根槽写在这个点之后、是下一条。够得着的版本是（txg，实例）不超过这几样根里最大那一条的行：
///   这个点之前一条根槽写都没有、又没有没做完的发布时（每条流的第一个点只有系统配置槽写），上界是基镜像里的根，版本表里够不着的行
///   不进，几条只差版本表的流在这里照样共享（test-script-align-code-r2 判决 J1）。没做完的那次发布在写表里没有根槽写，或基镜像里一条根都没有、
///   这个点之前也没有根槽写，上界取不出，整张表都进。
/// - 没做完的那次发布的根槽写（这个点最后一次写之前、最后一条根槽写之后还有单元写或记录写时，写表里下一条根槽写）：记录核对器按它分发布、
///   取 txg（`crash::checkpoint_txg_of_the_publish_that_made_the_write`），恢复落到它那一版时读它带的实例表
///   （`crash::instance_table_of_the_effective_root` 在整张写表里找写出这个根身份的根槽写）。
/// - 再往后的写：记录核对器对每次发布都看，但只在恢复落到的根 txg 不小于那次发布的 txg 时才核它的单元，只在它的根槽写的字节恰好在盘上时
///   才核「根在而记录不在」。根槽写的 txg 沿写表严格递增时这两样对这个点的状态都不成立：落到的根至多是没做完的那一版（前提是对基镜像本身
///   照 journal 恢复落到的根不晚于它里面最新的那条根、或落不到根，不另越过一版：排计划时断言，
///   `assert_journal_recovery_of_the_base_image_lands_no_later_than_its_newest_root`，不成立的流排计划时就拒），更晚的根槽写 txg 都更大、字节不在盘上。
///   不严格递增时推不出，之后的写整段进摘要（[`WritesAfterThePoint::MayAllBeRead`]）。
/// - 被判的根槽写（`judged_root_index`）不进键（的 M2-2 甲）：判器拿它记「根槽已持久」，不进逐项判定
///   （CPU 的 `crash::CrashStateJudgement::root_persisted`、GPU 判表头里那一个字都只记不判），被判根不同的两条流照样共享前缀；
///   「根槽已持久」的计数各项按自己的被判根事后数（[`states_with_the_root_slot_write_persisted`]，J7）。不把它在这个点之前
///   （每个状态都持久）、在点里（第几次写）、还是在之后（都不持久）拼进计划哈希，免得多记录流的 C 与删除流不共享。
/// - 起点镜像有哪几块盘、每块多大（扇区终值里看不出）。
struct JudgeInputsOfTheFlow {
    base_devices_text: String,
    /// 第 k 项：写表前 k 次写逐行链起来的摘要（`k` 从 0 到写数）。
    prefix_digests: Vec<[u8; DIGEST_BYTES]>,
    /// 每条根槽写：写表下标与它写下的根身份，（txg，实例）按择新的次序比。
    root_writes: Vec<(usize, (CheckpointTxg, InstanceGeneration))>,
    /// 基镜像里（txg，实例）最大的那条根；一条都没有是 `None`。
    newest_root_in_the_base_image: Option<(CheckpointTxg, InstanceGeneration)>,
    writes_after_the_point: WritesAfterThePoint,
}

/// 基镜像里（txg，实例）最大的那条根：写过的扇区里以根 magic 起头的（根槽写是一个物理块，崩溃点重放的池按 512 字节的扇区建），
/// 身份照根槽写的读法取（[`root_identity_written_by`]，不验自证校验和：判器输入只要知道恢复最多落到哪一代）。一条都没有是 `None`。
fn newest_root_in_the_base_image(base: &MemoryPool) -> Option<(CheckpointTxg, InstanceGeneration)> {
    base.devices
        .values()
        .flat_map(|device| device.written_sectors().values())
        .filter(|bytes| {
            bytes.len() >= ROOT_RECORD_IDENTITY_END && bytes[..ROOT_MAGIC.len()] == ROOT_MAGIC
        })
        .map(|bytes| {
            let (instance, checkpoint_txg) = root_identity_written_by(bytes);
            (checkpoint_txg, instance)
        })
        .max()
}

/// 根记录里身份那几个字节到哪为止：实例代号在偏移 24（4 字节）、checkpoint_txg 在偏移 28（8 字节）（[`root_identity_written_by`]）。
const ROOT_RECORD_IDENTITY_END: usize = 36;

/// B0 的前提（「窄一」放宽，`records/2026-10-13-复用键与全仓改名.md`「crates 红批交回」主 agent 判 ①）：
/// 对基镜像本身照 journal 恢复，落到的根不晚于基镜像里最新的那条根（（txg，实例）按择新的次序比），或者落不到根（`None`：两份系统配置槽
/// 都抹掉、或根都读不出，就没有可读的根）。
/// 恢复走判器那一条（`crash::judge_crash_image` 里的 `recover(…, JournalPolicy::Consult)`），读的是一次写都没落的崩溃镜像，也就是基镜像本身。
/// 要拒的只有越过去的一种：基镜像是崩溃状态（journal 记录落了、根槽写没落）时，恢复落到基镜像里没有根的更晚那一版；B0 按基镜像里的根截
/// 够得着的版本，就把这一版截掉了，判定不同的两条流会共用块键（test-script-align-code-r3 攻方 L1）。落到更早的根、或落不到根时，判器读的那一版
/// （或一版都不读）都在 B0 截下的范围里，块键罩得住。
///
/// # Panics
/// 落到的根比 `newest_root_of_the_base` 晚（连同基镜像里一条根都没有、恢复却落到了根）。
fn assert_journal_recovery_of_the_base_image_lands_no_later_than_its_newest_root(
    base: &MemoryPool,
    newest_root_of_the_base: Option<(CheckpointTxg, InstanceGeneration)>,
) {
    let base_with_no_write_persisted = CrashImage {
        base,
        writes: &[],
        persisted: Vec::new(),
    };
    let landed_root = recover(&base_with_no_write_persisted, JournalPolicy::Consult)
        .effective_root
        .map(|(instance, checkpoint_txg)| (checkpoint_txg, instance));
    let lands_no_later_than_the_newest_root = match (landed_root, newest_root_of_the_base) {
        (None, None | Some(_)) => true,
        (Some(_), None) => false,
        (Some(landed), Some(newest)) => landed <= newest,
    };
    assert!(
        lands_no_later_than_the_newest_root,
        "基镜像照 journal 恢复会越过它里面最新的根，B0 的前提不成立（要求恢复落到的根不晚于基镜像里最新的那条根，或落不到根）：\
         恢复落到（txg，实例）{landed_root:?}，基镜像里最新的根是 {newest_root_of_the_base:?}。\
         B0 按基镜像里的根截够得着的版本，恢复落到更晚的根时会截掉判器读得到的那一版，判定不同的两条流共用块键。\
         出路：基镜像先可写挂载一次（恢复把 journal 落成根、再发布）之后再取，或让流从 mkfs 起步；\
         要收这种基镜像，先定 B0 的上界怎么并上恢复落到的根（交设计，不在这里放宽）"
    );
}

impl JudgeInputsOfTheFlow {
    fn of(flow: &CrashFlow<'_>) -> Self {
        let mut segment_of_write: Vec<Option<usize>> = vec![None; flow.writes.len()];
        for (segment_index, segment) in flow.segments.iter().enumerate() {
            for write_index in segment {
                segment_of_write[*write_index] = Some(segment_index);
            }
        }
        let lines: Vec<String> = flow
            .writes
            .iter()
            .zip(&segment_of_write)
            .map(|(write, segment)| judge_input_line_of_a_write(write, *segment))
            .collect();
        let chained = |previous: &[u8; DIGEST_BYTES], line: &str| {
            let mut message = previous.to_vec();
            message.extend_from_slice(line.as_bytes());
            sha256_digest(&message)
        };
        let mut prefix_digests = Vec::with_capacity(lines.len() + 1);
        prefix_digests.push(sha256_digest(b"writes before the point\n"));
        for line in &lines {
            let previous = *prefix_digests.last().expect("起头那一项先放进去了");
            prefix_digests.push(chained(&previous, line));
        }
        let root_writes: Vec<(usize, (CheckpointTxg, InstanceGeneration))> = flow
            .writes
            .iter()
            .enumerate()
            .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
            .map(|(index, write)| {
                let (instance, checkpoint_txg) = root_identity_of_write(write);
                (index, (checkpoint_txg, instance))
            })
            .collect();
        let txgs_of_the_root_writes_strictly_increase = root_writes
            .windows(2)
            .all(|pair| pair[0].1 .0 < pair[1].1 .0);
        let writes_after_the_point = if txgs_of_the_root_writes_strictly_increase {
            WritesAfterThePoint::ReadOnlyThroughTheUnfinishedPublish
        } else {
            let mut suffix_digests =
                vec![sha256_digest(b"writes after the point\n"); lines.len() + 1];
            for index in (0..lines.len()).rev() {
                suffix_digests[index] = chained(&suffix_digests[index + 1], &lines[index]);
            }
            WritesAfterThePoint::MayAllBeRead { suffix_digests }
        };
        let devices: Vec<String> = flow
            .base
            .devices
            .keys()
            .map(|device| device.0.to_string())
            .collect();
        let newest_root_of_the_base = newest_root_in_the_base_image(flow.base);
        assert_journal_recovery_of_the_base_image_lands_no_later_than_its_newest_root(
            flow.base,
            newest_root_of_the_base,
        );
        Self {
            base_devices_text: format!(
                "base_devices={} device_size_in_bytes={}\n",
                devices.join(","),
                flow.base.device_size_in_bytes
            ),
            prefix_digests,
            root_writes,
            newest_root_in_the_base_image: newest_root_of_the_base,
            writes_after_the_point,
        }
    }

    /// 管写表 `first_write..last_write` 的那个点：它的状态里判器另读的输入（逐样见 [`JudgeInputsOfTheFlow`]）逐行写下、取 SHA-256。
    fn digest_for_a_point(
        &self,
        flow: &CrashFlow<'_>,
        first_write: usize,
        last_write: usize,
    ) -> [u8; DIGEST_BYTES] {
        let mut text = self.base_devices_text.clone();
        text.push_str(&format!(
            "writes_before_the_point={}\n",
            hexadecimal_text(&self.prefix_digests[first_write])
        ));
        let writes_since_the_last_root = self
            .root_writes
            .iter()
            .rev()
            .find(|(index, _)| *index < last_write)
            .map_or(0, |(index, _)| index + 1)..last_write;
        let a_publish_is_unfinished = flow.writes[writes_since_the_last_root]
            .iter()
            .any(|write| matches!(write.kind, StepKind::UnitWrite | StepKind::JournalRecord));
        let unfinished_publish_root = if a_publish_is_unfinished {
            self.root_writes
                .iter()
                .find(|(index, _)| *index >= last_write)
        } else {
            None
        };
        match unfinished_publish_root {
            Some((index, _)) => text.push_str(&format!(
                "unfinished_publish_root={}",
                judge_input_line_of_a_write(&flow.writes[*index], None)
            )),
            None => text.push_str(if a_publish_is_unfinished {
                "unfinished_publish_root=absent_from_the_write_table\n"
            } else {
                "unfinished_publish_root=none\n"
            }),
        }
        let highest_reachable_root: Option<(CheckpointTxg, InstanceGeneration)> =
            if a_publish_is_unfinished && unfinished_publish_root.is_none() {
                None
            } else {
                self.root_writes
                    .iter()
                    .filter(|(index, _)| *index < last_write)
                    .chain(unfinished_publish_root)
                    .map(|(_, identity)| *identity)
                    .chain(self.newest_root_in_the_base_image)
                    .max()
            };
        match highest_reachable_root {
            Some((checkpoint_txg, instance)) => text.push_str(&format!(
                "versions_up_to_txg={} instance={}\n",
                checkpoint_txg.0, instance.0
            )),
            None => text.push_str("versions_up_to=every_version\n"),
        }
        for version in flow.versions.iter().filter(|version| {
            highest_reachable_root
                .is_none_or(|highest| (version.checkpoint_txg, version.instance) <= highest)
        }) {
            text.push_str(&format!(
                "version instance={} txg={} content={}\n",
                version.instance.0,
                version.checkpoint_txg.0,
                hexadecimal_text(&sha256_digest(&version.content))
            ));
        }
        for version in flow.versions_without_a_file.iter().filter(|version| {
            highest_reachable_root.is_none_or(|highest_root| {
                (version.checkpoint_txg, version.instance) <= highest_root
            })
        }) {
            text.push_str(&format!(
                "version_without_a_file instance={} txg={}\n",
                version.instance.0, version.checkpoint_txg.0
            ));
        }
        match &self.writes_after_the_point {
            WritesAfterThePoint::ReadOnlyThroughTheUnfinishedPublish => {
                text.push_str("writes_after_the_point=read_only_through_the_unfinished_publish\n")
            }
            WritesAfterThePoint::MayAllBeRead { suffix_digests } => text.push_str(&format!(
                "writes_after_the_point={}\n",
                hexadecimal_text(&suffix_digests[last_write])
            )),
        }
        sha256_digest(text.as_bytes())
    }
}

#[cfg(test)]
mod tests;
