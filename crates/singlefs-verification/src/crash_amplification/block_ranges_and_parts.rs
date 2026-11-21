//! 按块区间分活：几台机器按权重分块（[`HostShare`]）、环境变量给的块区间（[`BlockRangeOfTheFlow`]），
//! 切了片的流按部分选区间（[`selection_of_the_part_range`]）。

use super::*;

// ============================== 按环境跑：一台机器（几张卡）领自己那一份 ==============================

/// 这台机器领哪一份：`SINGLEFS_CRASH_AMPLIFICATION_SHARE=<i>/<n>`，没设是 `0/1`（整份自己跑）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostShare {
    pub index: usize,
    pub count: usize,
}

impl HostShare {
    /// # Errors
    /// 写法不是 `<i>/<n>`、n 为 0、i 不小于 n。
    pub fn parse(text: &str) -> Result<Self, String> {
        let (index, count) = text
            .split_once('/')
            .ok_or_else(|| format!("分片写法要是 <i>/<n>，实际「{text}」"))?;
        let index: usize = index
            .trim()
            .parse()
            .map_err(|_| format!("分片序号不是数：「{text}」"))?;
        let count: usize = count
            .trim()
            .parse()
            .map_err(|_| format!("分片总数不是数：「{text}」"))?;
        if count == 0 || index >= count {
            return Err(format!("分片要 0 ≤ i < n，实际「{text}」"));
        }
        Ok(Self { index, count })
    }
}

/// 这台机器领哪一份与几台机器的权重（[`host_share_and_weights_of_the_share_variables`]）：给定状态表（[`GivenStatesEngine`]）、
/// 崩溃枚举用例与对拍两台分片的读法。按块序号区间选块（[`BLOCK_RANGE_ENVIRONMENT_VARIABLE`]）只有崩溃放量流水线认，这几路设了它就拒，
/// 不悄悄判整份。
///
/// # Errors
/// 设了 [`BLOCK_RANGE_ENVIRONMENT_VARIABLE`]；同 [`host_share_and_weights_of_the_share_variables`]。
pub(crate) fn host_share_and_weights_from_the_environment() -> Result<(HostShare, Vec<u64>), String>
{
    if std::env::var_os(BLOCK_RANGE_ENVIRONMENT_VARIABLE).is_some() {
        return Err(format!(
            "{BLOCK_RANGE_ENVIRONMENT_VARIABLE} 只给崩溃放量流水线（run_from_environment）按块序号区间选块；给定状态表、崩溃枚举用例与对拍这几路不认它：\
             去掉这个变量再跑，要拆片的照崩溃放量分项拆"
        ));
    }
    host_share_and_weights_of_the_share_variables()
}

/// 这台机器领哪一份（`SINGLEFS_CRASH_AMPLIFICATION_SHARE`，没设是整份）与几台机器的权重（`SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS=<w0>,<w1>,…`，
/// 没设等分）：流水线（[`run_from_environment`]）与给定状态表（[`GivenStatesEngine`]）两台分片同一种读法。
///
/// # Errors
/// 分片写法错（[`HostShare::parse`]）；权重不是数、个数与分片总数不等。
pub(super) fn host_share_and_weights_of_the_share_variables(
) -> Result<(HostShare, Vec<u64>), String> {
    let share = match std::env::var("SINGLEFS_CRASH_AMPLIFICATION_SHARE") {
        Ok(text) => HostShare::parse(&text)?,
        Err(_) => HostShare { index: 0, count: 1 },
    };
    let host_weights: Vec<u64> = match std::env::var("SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS") {
        Ok(text) => {
            let weights: Vec<u64> = text
                .split(',')
                .map(|weight| {
                    weight
                        .trim()
                        .parse::<u64>()
                        .map_err(|_| format!("主机权重不是数：「{text}」"))
                })
                .collect::<Result<_, _>>()?;
            if weights.len() != share.count {
                return Err(format!(
                    "主机权重 {} 个，分片总数 {}",
                    weights.len(),
                    share.count
                ));
            }
            weights
        }
        Err(_) => vec![1; share.count],
    };
    Ok((share, host_weights))
}

/// 按块序号区间选块：`<流名>:<起块>-<止块>`，几条流用逗号隔开（`tree_split_AccountingLeafSplit:0-512,tree_split_AccountingRootSplit:0-40`）。
/// 块序号是一条流全部点的块按次序编的号（[`PlannedCrashPoint::blocks`]，从 0 起）；区间半开，起块不大于止块，止块不过这条流的块数，
/// 一条流在里面至多一段。起块等于止块是空区间（`<流名>:<块>-<块>`）：这一片确实不判这条流，一块不选，只有这一种写法一块不选。
/// 是调用方（入口脚本、任务服务）交下来的选范围参数，只有崩溃放量流水线（[`run_from_environment`]）认：设了它，
/// 这一趟每条流只判点到的那一段里的块；设了而没点到这条流的，这一趟在录入之前交错（流名写错一个字不许绿着一块不判）。
/// 对账（[`EnvironmentRun::unjudged_blocks`]）与读违例只数本片（那一段里的块，不分份）。没设是整条流，对账与读违例数整条流。与 `SINGLEFS_CRASH_AMPLIFICATION_SHARE` 同给时取交集：区间定这一片，
/// 份定这一片在几台之间怎么分（块序号照整条流的编号轮转，与不给区间时同一块归同一份）。只挑判哪几块，块键、判法单元指纹、录入键都不变，
/// 别的片与不分片的趟判过的块照复用。别的读份的路设了它就拒（[`host_share_and_weights_from_the_environment`]）。
///
/// 登了分片段的流（按段登记的段整域装不进 u64，[`DomainKindOfARegisteredSegment::Slice`]）写成 `<流名>:<份号>/<起块>-<止块>`，
/// 与上面的写法在同一个变量里混写（`position_addressed_ExtentLowerSegmentGrowsToTwoLevels:0/0-65536,tree_split_AccountingRootSplit:0-40`）。
/// 这种流的块按整域编号：分片的段整域切成的块（一块 [`STATES_PER_BLOCK`] 个态，从段内编号 0 起切）与别的块照计划的次序排成一列，
/// 这一列按固定的份长（[`BlocksPerPartAsAPowerOfTwo::OF_THE_AMPLIFICATION`]）切成几份，份号与份内块序号都是十进制、装得进双 128 位
/// （[`Unsigned256Bits`]）。份只由流的内容（写表与展开）定，不读库里的分片进度：同一个（份，起块，止块）先判后判、库里判了多少，
/// 点到的都是同一批状态，块键与不设区间、照进度续跑判到同一批状态时相同（[`selection_of_the_part_range`]）。这一趟分片的段的片由区间定，
/// 不推进分片进度，录入键带片（[`recording_of_the_run_keyed_by_its_slices`]）。没登分片段的流写份号、登了的流不写份号，都在录入之前拒。
pub const BLOCK_RANGE_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_BLOCK_RANGE";

/// 一条流这一趟判哪几块（[`BLOCK_RANGE_ENVIRONMENT_VARIABLE`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockRangeOfTheFlow {
    /// 没设区间变量：整条流。
    EveryBlockOfTheFlow,
    /// 区间变量点了这条流：块序号落在这个半开区间里的块（起块不大于止块、止块不过这条流的块数，[`BlockRangesOfTheFlows::range_of_the_flow`] 核过）；
    /// 起块等于止块是空区间，一块不选，对账与读违例一块不数。
    BlocksInTheRange(Range<u64>),
}

impl BlockRangeOfTheFlow {
    /// 第 `block_sequence` 块在不在这一片里（份另算）。
    #[must_use]
    pub fn holds(&self, block_sequence: u64) -> bool {
        match self {
            Self::EveryBlockOfTheFlow => true,
            Self::BlocksInTheRange(range) => range.contains(&block_sequence),
        }
    }

    /// 这一片的块序号，按次序（`blocks_of_the_flow` 是这条流的块数）。
    fn block_sequences(&self, blocks_of_the_flow: u64) -> Range<u64> {
        match self {
            Self::EveryBlockOfTheFlow => 0..blocks_of_the_flow,
            Self::BlocksInTheRange(range) => range.clone(),
        }
    }

    /// 驱动行末尾接的两个字段：` block_range=<起块>-<止块> blocks_of_the_flow=<这条流的块数>`（空区间照写 `<块>-<块>`）；
    /// 没设区间变量是空串（行与不给区间时逐字相同）。入口核一条流的几片连起来罩全 `0..blocks_of_the_flow` 照这两个字段。
    #[must_use]
    pub fn fields_of_the_line(&self, blocks_of_the_flow: u64) -> String {
        match self {
            Self::EveryBlockOfTheFlow => String::new(),
            Self::BlocksInTheRange(range) => format!(
                " block_range={}-{} blocks_of_the_flow={blocks_of_the_flow}",
                range.start, range.end
            ),
        }
    }
}

/// 设了区间变量的每一趟在排出计划、定下这一片之后打的那一行的行首（[`block_range_line`]）。只录、只排派活计划、被叫停、
/// 这台机器一块没选中与判完的那一趟都打，在这几种交回之前打。
pub const BLOCK_RANGE_LINE_PREFIX: &str = "CRASH_AMPLIFICATION_BLOCK_RANGE";

/// 设了区间变量时这一趟打的那一行：
/// `CRASH_AMPLIFICATION_BLOCK_RANGE flow=<流名> share=<份号>/<份数> block_range=<起块>-<止块> blocks_of_the_flow=<这条流的块数> blocks_selected_here=<这台机器选中的块数>`；
/// 没设区间变量交 `None`、不打。入口按 `flow=` 把同一条流各片的 `block_range=` 收齐，核它们连起来恰好罩全 `0..blocks_of_the_flow`。
#[must_use]
pub fn block_range_line(
    flow_name: &str,
    share: HostShare,
    block_range: &BlockRangeOfTheFlow,
    blocks_of_the_flow: u64,
    blocks_selected_here: usize,
) -> Option<String> {
    match block_range {
        BlockRangeOfTheFlow::EveryBlockOfTheFlow => None,
        BlockRangeOfTheFlow::BlocksInTheRange(_) => Some(format!(
            "{BLOCK_RANGE_LINE_PREFIX} flow={flow_name} share={}/{}{} blocks_selected_here={blocks_selected_here}",
            share.index,
            share.count,
            block_range.fields_of_the_line(blocks_of_the_flow)
        )),
    }
}

/// 登了分片段的流设了区间时这一趟打的那一行（同一个行首 [`BLOCK_RANGE_LINE_PREFIX`]）：
/// `CRASH_AMPLIFICATION_BLOCK_RANGE flow=<流名> share=<份号>/<份数> block_range=<份号>/<起块>-<止块> parts_of_the_flow=<份数> blocks_of_the_part=<这一份的块数> blocks_selected_here=<这台机器选中的块数>`
/// （`share=` 是两台分份，`block_range=` 里斜杠前是这条流按整域切的份号，两者不是一回事）。
#[must_use]
pub fn block_range_line_of_a_part(
    flow_name: &str,
    share: HostShare,
    part: &PartOfASlicedFlowSelected,
    blocks_selected_here: usize,
) -> String {
    format!(
        "{BLOCK_RANGE_LINE_PREFIX} flow={flow_name} share={}/{}{} blocks_selected_here={blocks_selected_here}",
        share.index,
        share.count,
        part.fields_of_the_line()
    )
}

/// 区间变量（[`BLOCK_RANGE_ENVIRONMENT_VARIABLE`]）读出的各条流的区间：写法核过（流名不空、块序号是数、起块不大于止块、一条流至多一段）；
/// 点没点到一条流在录入之前核（[`Self::written_range_of_the_flow`]），止块过没过这条流的块数要等排出计划才核（[`Self::range_of_the_flow`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockRangesOfTheFlows {
    ranges: Vec<(String, WrittenBlockRangeOfTheFlow)>,
}

/// 区间变量里给一条流写的那一段（还没对这条流的块数核）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WrittenBlockRangeOfTheFlow {
    /// `<流名>:<起块>-<止块>`：没登分片段的流，块序号是整条流的块按次序编的号。
    BlocksOfTheFlow(Range<u64>),
    /// `<流名>:<份号>/<起块>-<止块>`：登了分片段的流，第几份里份内块序号的半开区间（[`selection_of_the_part_range`]）。
    BlocksOfAPart(PartRangeOfASlicedFlow),
}

/// 登了分片段的流这一趟判的那一段：第 `part` 份里份内块序号 `blocks`（半开，起块不大于止块）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartRangeOfASlicedFlow {
    pub part: Unsigned256Bits,
    pub blocks: Range<Unsigned256Bits>,
}

impl BlockRangesOfTheFlows {
    /// # Errors
    /// 有一段不是 `<流名>:<起块>-<止块>` 或 `<流名>:<份号>/<起块>-<止块>`（空着的、少冒号或连字符的）、流名空着、块序号不是数
    /// （份的写法里份号与块序号要是不带前导 0 的十进制、装得进双 128 位）；起块大于止块；同一条流写了两段。
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut ranges: Vec<(String, WrittenBlockRangeOfTheFlow)> = Vec::new();
        for entry in text.split(',') {
            let entry = entry.trim();
            let written_form = || {
                format!(
                    "{BLOCK_RANGE_ENVIRONMENT_VARIABLE} 每一段写成 <流名>:<起块>-<止块>（登了分片段的流写 <流名>:<份号>/<起块>-<止块>，\
                     份号与块序号是十进制、装得进双 128 位），实际「{entry}」（整个值「{text}」）"
                )
            };
            let (flow_name, range_text) = entry.rsplit_once(':').ok_or_else(written_form)?;
            if flow_name.is_empty() {
                return Err(written_form());
            }
            let start_is_above_the_end = || {
                format!(
                    "{BLOCK_RANGE_ENVIRONMENT_VARIABLE} 的区间是半开的，起块不能大于止块（这一片不判这条流的写空区间 <流名>:<块>-<块>）：「{entry}」"
                )
            };
            let written = match range_text.split_once('/') {
                None => {
                    let (start_text, end_text) =
                        range_text.split_once('-').ok_or_else(written_form)?;
                    let start: u64 = start_text.trim().parse().map_err(|_| written_form())?;
                    let end: u64 = end_text.trim().parse().map_err(|_| written_form())?;
                    if start > end {
                        return Err(start_is_above_the_end());
                    }
                    WrittenBlockRangeOfTheFlow::BlocksOfTheFlow(start..end)
                }
                Some((part_text, blocks_text)) => {
                    let decimal = |number_text: &str| {
                        Unsigned256Bits::from_decimal_text(number_text).ok_or_else(written_form)
                    };
                    let part = decimal(part_text)?;
                    let (start_text, end_text) =
                        blocks_text.split_once('-').ok_or_else(written_form)?;
                    let start = decimal(start_text)?;
                    let end = decimal(end_text)?;
                    if start > end {
                        return Err(start_is_above_the_end());
                    }
                    WrittenBlockRangeOfTheFlow::BlocksOfAPart(PartRangeOfASlicedFlow {
                        part,
                        blocks: start..end,
                    })
                }
            };
            if ranges.iter().any(|(named, _)| named == flow_name) {
                return Err(format!(
                    "{BLOCK_RANGE_ENVIRONMENT_VARIABLE} 里流 {flow_name} 写了不止一段（一条流一趟至多判一段）：「{text}」"
                ));
            }
            ranges.push((flow_name.to_string(), written));
        }
        Ok(Self { ranges })
    }

    /// 读 [`BLOCK_RANGE_ENVIRONMENT_VARIABLE`]：没设交 `None`（整条流）。
    ///
    /// # Errors
    /// 值不是 UTF-8；写法错（[`Self::parse`]）。
    pub fn from_the_environment() -> Result<Option<Self>, String> {
        match std::env::var(BLOCK_RANGE_ENVIRONMENT_VARIABLE) {
            Ok(text) => Self::parse(&text).map(Some),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(std::env::VarError::NotUnicode(value)) => Err(format!(
                "{BLOCK_RANGE_ENVIRONMENT_VARIABLE} 的值不是 UTF-8：{value:?}"
            )),
        }
    }

    /// 变量里给名叫 `flow_name` 的那条流写的那一段（还没对块数核）。
    ///
    /// # Errors
    /// 变量里没点到这条流（流名写错、漏写）：错误文字点名变量、这条流的名字与变量里写了哪几条流，并说这一片不判这条流要显式写空区间。
    pub fn written_range_of_the_flow(
        &self,
        flow_name: &str,
    ) -> Result<WrittenBlockRangeOfTheFlow, String> {
        match self.ranges.iter().find(|(named, _)| named == flow_name) {
            Some((_, range)) => Ok(range.clone()),
            None => Err(format!(
                "{BLOCK_RANGE_ENVIRONMENT_VARIABLE} 设了却没点到流 {flow_name}（变量里写的流：{}）：流名照 CRASH_AMPLIFICATION_RECORDED flow= 那一个逐字写；\
                 这一片确实不判这条流的，显式写空区间 {flow_name}:<块>-<块>",
                self.ranges
                    .iter()
                    .map(|(named, _)| named.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            )),
        }
    }

    /// 名叫 `flow_name`、没登分片段、一共 `blocks_of_the_flow` 块的那条流这一趟判哪几块：照变量里点它的那一段（空区间一块不选）。
    ///
    /// # Errors
    /// 没点到这条流（[`Self::written_range_of_the_flow`]）；点到这条流的那一段写了份号（份只给登了分片段的流）；
    /// 止块过了这条流的块数（越界：流变短或区间写错，不悄悄截掉）。
    pub fn range_of_the_flow(
        &self,
        flow_name: &str,
        blocks_of_the_flow: u64,
    ) -> Result<BlockRangeOfTheFlow, String> {
        let range = match self.written_range_of_the_flow(flow_name)? {
            WrittenBlockRangeOfTheFlow::BlocksOfTheFlow(range) => range,
            WrittenBlockRangeOfTheFlow::BlocksOfAPart(_) => {
                return Err(format!(
                    "{BLOCK_RANGE_ENVIRONMENT_VARIABLE} 给流 {flow_name} 写了份号：这条流没登分片段（按段登记的段整域都装得进 u64），\
                     不按份拆，区间写成 {flow_name}:<起块>-<止块>（这条流一共 {blocks_of_the_flow} 块）"
                ));
            }
        };
        if range.end > blocks_of_the_flow {
            return Err(format!(
                "{BLOCK_RANGE_ENVIRONMENT_VARIABLE} 给流 {flow_name} 的区间 {}-{} 越界：这条流只有 {blocks_of_the_flow} 块（块序号 0 到 {}，止块至多 {blocks_of_the_flow}）",
                range.start,
                range.end,
                blocks_of_the_flow.saturating_sub(1)
            ));
        }
        Ok(BlockRangeOfTheFlow::BlocksInTheRange(range))
    }

    /// 名叫 `flow_name`、登了分片段的那条流这一趟判的那一段（份号与份内块序号，还没对份数与这一份的块数核，[`selection_of_the_part_range`] 核）。
    ///
    /// # Errors
    /// 没点到这条流（[`Self::written_range_of_the_flow`]）；点到这条流的那一段没写份号（这条流的块按整域编号、按份拆，
    /// 不写份号的块序号点不到同一批状态）。
    pub fn part_range_of_the_sliced_flow(
        &self,
        flow_name: &str,
    ) -> Result<PartRangeOfASlicedFlow, String> {
        match self.written_range_of_the_flow(flow_name)? {
            WrittenBlockRangeOfTheFlow::BlocksOfAPart(part_range) => Ok(part_range),
            WrittenBlockRangeOfTheFlow::BlocksOfTheFlow(range) => Err(format!(
                "{BLOCK_RANGE_ENVIRONMENT_VARIABLE} 给流 {flow_name} 写的 {}-{} 没写份号：这条流登了分片段（按段登记的段整域装不进 u64），\
                 块按整域编号、按份拆，区间写成 {flow_name}:<份号>/<起块>-<止块>（份号与块序号是十进制）",
                range.start, range.end
            )),
        }
    }
}

/// 登了分片段的流按份拆开时一份几块：2^`.0` 块。崩溃放量取 [`Self::OF_THE_AMPLIFICATION`]；
/// [`selection_of_the_part_range`] 收它作参数，测试拿小的份长造出「不止一份」的流。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlocksPerPartAsAPowerOfTwo(pub usize);

impl BlocksPerPartAsAPowerOfTwo {
    /// 崩溃放量的份长：一份 2^239 块、至多 2^255 个态（一块 [`STATES_PER_BLOCK`] = 2^16 个态）。一份的态数、份内块序号与份内态的编号
    /// 都装得进双 128 位（[`Unsigned256Bits`]）；份数是整条流的块数除以 2^239 向上取整，登记的段整域在 3^290 个态以内时
    /// 份数在 2^205 以内，同样装得进。
    pub const OF_THE_AMPLIFICATION: Self = Self(239);
}

/// 登了分片段的流按整域编号的块里的一截（照计划的次序）：几块不分片的块，或一段分片的段整域切成的块。
#[derive(Clone, Debug, PartialEq, Eq)]
enum StretchOfTheBlocksOfASlicedFlow {
    BlocksNotSliced {
        blocks: u64,
    },
    SlicedSegment {
        segment_index: usize,
        domain_state_count: StateCount1280Bits,
    },
}

impl StretchOfTheBlocksOfASlicedFlow {
    /// 这一截按整域编号有几块：分片的段整域按 [`STATES_PER_BLOCK`] 从段内编号 0 起切，末块可以不满。
    fn blocks(&self) -> StateCount1280Bits {
        match self {
            Self::BlocksNotSliced { blocks } => StateCount1280Bits::from_u64(*blocks),
            Self::SlicedSegment {
                domain_state_count, ..
            } => blocks_of_a_domain(domain_state_count),
        }
    }
}

/// `domain_state_count` 个态从编号 0 起按 [`STATES_PER_BLOCK`] 切成几块（末块可以不满）。
fn blocks_of_a_domain(domain_state_count: &StateCount1280Bits) -> StateCount1280Bits {
    let (whole_blocks, states_left_over) = domain_state_count.divided_by_u64(STATES_PER_BLOCK);
    if states_left_over == 0 {
        whole_blocks
    } else {
        whole_blocks
            .checked_add_u64(1)
            .expect("整块数比态数小，加 1 装得进 1280 位")
    }
}

/// 照 `plan`（分片的段片长为 0 的那一份，[`placeholder_slices_of_the_sliced_segments`]）把这条流的块按整域编号排成几截：
/// 点按次序、点里的截（[`PieceOfAPointWithASlice`]）按次序，不分片的截与不管分片的段的点数它们的块，分片的段那一截数整域的块；
/// 相邻的不分片的几截并成一截。
fn stretches_of_the_blocks_of_a_sliced_flow(
    plan: &CrashPlan,
) -> Vec<StretchOfTheBlocksOfASlicedFlow> {
    fn push_blocks_not_sliced(blocks: u64, stretches: &mut Vec<StretchOfTheBlocksOfASlicedFlow>) {
        match stretches.last_mut() {
            Some(StretchOfTheBlocksOfASlicedFlow::BlocksNotSliced { blocks: so_far }) => {
                *so_far += blocks;
            }
            Some(StretchOfTheBlocksOfASlicedFlow::SlicedSegment { .. }) | None => {
                stretches.push(StretchOfTheBlocksOfASlicedFlow::BlocksNotSliced { blocks });
            }
        }
    }
    let mut stretches: Vec<StretchOfTheBlocksOfASlicedFlow> = Vec::new();
    for point in &plan.points {
        if point.pieces_around_the_slices.is_empty() {
            push_blocks_not_sliced(
                u64::try_from(point.blocks().len()).expect("块数装得进 u64"),
                &mut stretches,
            );
            continue;
        }
        for piece in &point.pieces_around_the_slices {
            match &piece.kind {
                KindOfAPiece::SegmentsNotSliced { .. } => push_blocks_not_sliced(
                    (piece.local_states.end - piece.local_states.start).div_ceil(STATES_PER_BLOCK),
                    &mut stretches,
                ),
                KindOfAPiece::SlicedSegment {
                    segment_index,
                    domain_state_count,
                    ..
                } => stretches.push(StretchOfTheBlocksOfASlicedFlow::SlicedSegment {
                    segment_index: *segment_index,
                    domain_state_count: *domain_state_count,
                }),
            }
        }
    }
    stretches
}

/// 登了分片段的流给不出这一段区间：都在录入之前、写库之前交回（[`selection_of_the_part_range`]）。数都存成十进制数字串（拒绝句照写）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PartRangeRefused {
    /// 份号不在 `0..parts_of_the_flow`。
    PartPastTheParts {
        part_text: String,
        parts_of_the_flow_text: String,
        blocks_per_part_text: String,
        blocks_of_the_last_part_text: String,
    },
    /// 止块过了这一份的块数（越界：不悄悄截掉）。
    BlocksPastThePart {
        part_range_text: String,
        part_text: String,
        parts_of_the_flow_text: String,
        blocks_of_the_part_text: String,
    },
    /// 一段分片的段在这一段区间里的块超过一趟的片长上界（[`LARGEST_REGISTERED_SEGMENT_SLICE_STATES`] 个态：GPU 内核按 32 位加片内序号）。
    SliceLongerThanTheLargestSlice {
        segment_index: usize,
        blocks_in_the_range_text: String,
    },
    /// 整条流按整域编号的块数、份数或一份的块数装不进双 128 位：份长与「拆到双 128 位装得下」只对份数与份内块数装得进的流定了，
    /// 更大的域怎么拆没定，第一版不支持（在任何录入、写库之前交回）。
    PartNumbersPastTwoHundredFiftySixBitsUnsupported { blocks_of_the_flow_text: String },
}

impl PartRangeRefused {
    /// 交给调用方的那一句：点名区间变量与流，说这条流有几份、这一份（或末一份）有几块。
    #[must_use]
    pub fn sentence(&self, flow_name: &str) -> String {
        match self {
            Self::PartPastTheParts {
                part_text,
                parts_of_the_flow_text,
                blocks_per_part_text,
                blocks_of_the_last_part_text,
            } => format!(
                "{BLOCK_RANGE_ENVIRONMENT_VARIABLE} 给流 {flow_name} 的份号 {part_text} 越界：这条流有 {parts_of_the_flow_text} 份（份号从 0 起），\
                 每份 {blocks_per_part_text} 块，末一份 {blocks_of_the_last_part_text} 块"
            ),
            Self::BlocksPastThePart {
                part_range_text,
                part_text,
                parts_of_the_flow_text,
                blocks_of_the_part_text,
            } => format!(
                "{BLOCK_RANGE_ENVIRONMENT_VARIABLE} 给流 {flow_name} 的区间 {part_range_text} 越界：这条流有 {parts_of_the_flow_text} 份，\
                 第 {part_text} 份有 {blocks_of_the_part_text} 块（止块至多 {blocks_of_the_part_text}）"
            ),
            Self::SliceLongerThanTheLargestSlice {
                segment_index,
                blocks_in_the_range_text,
            } => format!(
                "{BLOCK_RANGE_ENVIRONMENT_VARIABLE} 给流 {flow_name} 的区间点到第 {segment_index} 段（分片的段）{blocks_in_the_range_text} 块，一趟至多 {} 块\
                 （{LARGEST_REGISTERED_SEGMENT_SLICE_STATES} 个态，GPU 内核按 32 位加片内序号）：把区间拆成几段、分几趟判",
                LARGEST_REGISTERED_SEGMENT_SLICE_STATES / STATES_PER_BLOCK
            ),
            Self::PartNumbersPastTwoHundredFiftySixBitsUnsupported {
                blocks_of_the_flow_text,
            } => format!(
                "{BLOCK_RANGE_ENVIRONMENT_VARIABLE} 给流 {flow_name} 的份：这条流按整域编号 {blocks_of_the_flow_text} 块，份数或一份的块数装不进双 128 位，\
                 更大的域怎么拆没定，第一版不支持（没写库）"
            ),
        }
    }
}

/// 登了分片段的流这一趟判的那一份与那一段（驱动行与 [`BLOCK_RANGE_LINE_PREFIX`] 那一行上的字段，[`Self::fields_of_the_line`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartOfASlicedFlowSelected {
    pub part_range: PartRangeOfASlicedFlow,
    pub parts_of_the_flow: Unsigned256Bits,
    pub blocks_of_the_part: Unsigned256Bits,
}

impl PartOfASlicedFlowSelected {
    /// 驱动行末尾接的三个字段：` block_range=<份号>/<起块>-<止块> parts_of_the_flow=<份数> blocks_of_the_part=<这一份的块数>`（都是十进制）。
    /// 入口核一条流的几片罩没罩全：同一份的几段连起来罩全 `0..blocks_of_the_part`，几份罩全 `0..parts_of_the_flow`。
    #[must_use]
    pub fn fields_of_the_line(&self) -> String {
        format!(
            " block_range={}/{}-{} parts_of_the_flow={} blocks_of_the_part={}",
            self.part_range.part.decimal_text(),
            self.part_range.blocks.start.decimal_text(),
            self.part_range.blocks.end.decimal_text(),
            self.parts_of_the_flow.decimal_text(),
            self.blocks_of_the_part.decimal_text()
        )
    }
}

/// 登了分片段的流的一段区间定下的这一趟：每段的片（与段同序；分片的段的片由区间定，不读库里的进度）、照这几片排出的计划里点到的块
/// （块序号是全部点的块按次序编的号，从 0 起；点到的块在计划里首尾相接）、那份计划一共几块，与驱动行上的份。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionOfAPartRange {
    pub segment_slices: Vec<Option<CrashPointReplaySegmentSlice>>,
    pub blocks_of_the_plan: Range<u64>,
    pub blocks_in_the_plan: u64,
    pub part: PartOfASlicedFlowSelected,
}

/// `value` 除以 2^`power` 向上取整。
fn quotient_by_a_power_of_two_rounded_up(
    value: &StateCount1280Bits,
    power: usize,
) -> StateCount1280Bits {
    let quotient = value.shifted_right(power);
    let multiple_below = quotient
        .checked_shift_left(power)
        .expect("商左移回去不大于被除数，装得进 1280 位");
    if multiple_below == *value {
        quotient
    } else {
        quotient
            .checked_add_u64(1)
            .expect("商比被除数小，加 1 装得进 1280 位")
    }
}

/// 登了分片段的流的一段区间（第几份、份内起块到止块）定下这一趟：这条流的块按整域编号排成一列（[`stretches_of_the_blocks_of_a_sliced_flow`]：
/// 分片的段整域从段内编号 0 起按 [`STATES_PER_BLOCK`] 切），这一列按一份 2^`blocks_per_part` 块切成几份；区间点到的块落在分片的段里的，
/// 这一段这一趟的片就是那几块（片起点 = 段内块号 × [`STATES_PER_BLOCK`]，片长到区间末或整域末），没点到的分片的段片长为 0；
/// 落在不分片的块里的照计划的次序点。份与片只由 `plan_of_placeholder_slices`（[`placeholder_slices_of_the_sliced_segments`] 排的那一份）定，
/// 不读库：分片的段里的块键锚在段内编号上（[`PlannedCrashPoint::block_key_of_a_slice_block`]），同一批状态不论先判后判、照进度续跑判
/// 还是照区间判，都是同一把块键。
///
/// # Errors
/// 份号越界、止块过了这一份的块数、点到一段分片的段的块超过一趟的片长上界、份数或一份的块数装不进双 128 位（[`PartRangeRefused`]）。
pub fn selection_of_the_part_range(
    plan_of_placeholder_slices: &CrashPlan,
    segment_count: usize,
    part_range: &PartRangeOfASlicedFlow,
    blocks_per_part: BlocksPerPartAsAPowerOfTwo,
) -> Result<SelectionOfAPartRange, PartRangeRefused> {
    let stretches = stretches_of_the_blocks_of_a_sliced_flow(plan_of_placeholder_slices);
    let mut blocks_of_the_flow = Some(StateCount1280Bits::ZERO);
    for stretch in &stretches {
        blocks_of_the_flow =
            blocks_of_the_flow.and_then(|so_far| so_far.checked_add(&stretch.blocks()));
    }
    let unsupported = |blocks_of_the_flow_text: String| {
        PartRangeRefused::PartNumbersPastTwoHundredFiftySixBitsUnsupported {
            blocks_of_the_flow_text,
        }
    };
    let blocks_of_the_flow =
        blocks_of_the_flow.ok_or_else(|| unsupported("越过 1280 位".to_string()))?;
    let power = blocks_per_part.0;
    let parts_of_the_flow = quotient_by_a_power_of_two_rounded_up(&blocks_of_the_flow, power)
        .narrowed_to_256_bits()
        .ok_or_else(|| unsupported(blocks_of_the_flow.decimal_text()))?;
    // 一份的块数：2^power 装不进 1280 位时一份就罩全整条流
    let blocks_per_part_wide = StateCount1280Bits::from_u64(1).checked_shift_left(power);
    let blocks_of_a_part_starting_at = |part_start: &StateCount1280Bits| {
        let blocks_left = blocks_of_the_flow
            .checked_sub(part_start)
            .expect("份起点在整条流的块数以内");
        match blocks_per_part_wide {
            Some(per_part) if per_part < blocks_left => per_part,
            Some(_) | None => blocks_left,
        }
    };
    if part_range.part >= parts_of_the_flow {
        let last_part_start = parts_of_the_flow
            .checked_sub(&Unsigned256Bits::from_u64(1))
            .map_or(StateCount1280Bits::ZERO, |last_part| {
                last_part
                    .widened()
                    .checked_shift_left(power)
                    .expect("末一份的起点在整条流的块数以内")
            });
        return Err(PartRangeRefused::PartPastTheParts {
            part_text: part_range.part.decimal_text(),
            parts_of_the_flow_text: parts_of_the_flow.decimal_text(),
            blocks_per_part_text: blocks_per_part_wide
                .and_then(|per_part| per_part.narrowed_to_256_bits())
                .ok_or_else(|| unsupported(blocks_of_the_flow.decimal_text()))?
                .decimal_text(),
            blocks_of_the_last_part_text: blocks_of_a_part_starting_at(&last_part_start)
                .narrowed_to_256_bits()
                .ok_or_else(|| unsupported(blocks_of_the_flow.decimal_text()))?
                .decimal_text(),
        });
    }
    let part_start = part_range
        .part
        .widened()
        .checked_shift_left(power)
        .expect("份号小于份数，份起点在整条流的块数以内");
    let blocks_of_the_part = blocks_of_a_part_starting_at(&part_start)
        .narrowed_to_256_bits()
        .ok_or_else(|| unsupported(blocks_of_the_flow.decimal_text()))?;
    if part_range.blocks.end > blocks_of_the_part {
        return Err(PartRangeRefused::BlocksPastThePart {
            part_range_text: format!(
                "{}/{}-{}",
                part_range.part.decimal_text(),
                part_range.blocks.start.decimal_text(),
                part_range.blocks.end.decimal_text()
            ),
            part_text: part_range.part.decimal_text(),
            parts_of_the_flow_text: parts_of_the_flow.decimal_text(),
            blocks_of_the_part_text: blocks_of_the_part.decimal_text(),
        });
    }
    let range_start = part_start
        .checked_add(&part_range.blocks.start.widened())
        .expect("起块在这一份以内");
    let range_end = part_start
        .checked_add(&part_range.blocks.end.widened())
        .expect("止块在这一份以内");
    let mut segment_slices = no_segment_slices(segment_count);
    let mut stretch_start = StateCount1280Bits::ZERO;
    let mut blocks_in_the_plan = 0u64;
    let mut blocks_of_the_plan: Option<Range<u64>> = None;
    for stretch in &stretches {
        let stretch_blocks = stretch.blocks();
        let stretch_end = stretch_start
            .checked_add(&stretch_blocks)
            .expect("各截的块数合起来就是整条流的块数");
        // 区间与这一截的交集，按截内的块号
        let overlap_start = range_start.max(stretch_start);
        let overlap_end = range_end.min(stretch_end);
        let overlap_in_the_stretch = (overlap_start < overlap_end).then(|| {
            (
                overlap_start
                    .checked_sub(&stretch_start)
                    .expect("交集在这一截以内"),
                overlap_end
                    .checked_sub(&stretch_start)
                    .expect("交集在这一截以内"),
            )
        });
        let (blocks_of_the_stretch_in_the_plan, selected_in_the_stretch) = match stretch {
            StretchOfTheBlocksOfASlicedFlow::BlocksNotSliced { blocks } => (
                *blocks,
                overlap_in_the_stretch.map(|(start, end)| {
                    start.to_u64().expect("不分片的块号装得进 u64")
                        ..end.to_u64().expect("不分片的块号装得进 u64")
                }),
            ),
            StretchOfTheBlocksOfASlicedFlow::SlicedSegment {
                segment_index,
                domain_state_count,
            } => {
                let (slice, blocks_of_the_slice) = match &overlap_in_the_stretch {
                    None => (
                        CrashPointReplaySegmentSlice {
                            start_within_the_domain: StateCount1280Bits::ZERO,
                            state_count: 0,
                        },
                        0,
                    ),
                    Some((start, end)) => {
                        let blocks_in_the_range =
                            end.checked_sub(start).expect("交集的止块不小于起块");
                        let blocks_of_the_slice = blocks_in_the_range
                            .to_u64()
                            .filter(|blocks| {
                                *blocks
                                    <= LARGEST_REGISTERED_SEGMENT_SLICE_STATES / STATES_PER_BLOCK
                            })
                            .ok_or_else(|| PartRangeRefused::SliceLongerThanTheLargestSlice {
                                segment_index: *segment_index,
                                blocks_in_the_range_text: blocks_in_the_range.decimal_text(),
                            })?;
                        let start_within_the_domain = start
                            .checked_mul_u64(STATES_PER_BLOCK)
                            .expect("段内块号乘一块的态数不越过整域");
                        let states_left = domain_state_count
                            .checked_sub(&start_within_the_domain)
                            .expect("片起点在整域以内");
                        let state_count = states_left
                            .to_u64()
                            .map_or(blocks_of_the_slice * STATES_PER_BLOCK, |left| {
                                left.min(blocks_of_the_slice * STATES_PER_BLOCK)
                            });
                        (
                            CrashPointReplaySegmentSlice {
                                start_within_the_domain,
                                state_count,
                            },
                            blocks_of_the_slice,
                        )
                    }
                };
                segment_slices[*segment_index] = Some(slice);
                (
                    blocks_of_the_slice,
                    overlap_in_the_stretch.map(|_| 0..blocks_of_the_slice),
                )
            }
        };
        if let Some(selected) = selected_in_the_stretch {
            let in_the_plan =
                blocks_in_the_plan + selected.start..blocks_in_the_plan + selected.end;
            // 区间在整域编号里首尾相接，各截点到的块在计划里同样首尾相接：并起来取头尾
            blocks_of_the_plan = Some(match blocks_of_the_plan {
                Some(so_far) => so_far.start..in_the_plan.end,
                None => in_the_plan,
            });
        }
        blocks_in_the_plan += blocks_of_the_stretch_in_the_plan;
        stretch_start = stretch_end;
    }
    Ok(SelectionOfAPartRange {
        segment_slices,
        blocks_of_the_plan: blocks_of_the_plan.unwrap_or(0..0),
        blocks_in_the_plan,
        part: PartOfASlicedFlowSelected {
            part_range: part_range.clone(),
            parts_of_the_flow,
            blocks_of_the_part,
        },
    })
}

/// 同 [`selection_of_the_part_range`]，从录入段那一份流（登记了的段已换成 [`expansion_in_the_recording`] 的那一份）与它登记的段的域起：
/// 先排分片的段片长为 0 的那份计划。
///
/// # Errors
/// 同 [`selection_of_the_part_range`]。
///
/// # Panics
/// 排那份计划时算态数装不下（调用方先核 [`states_of_the_flow_to_record`]）。
pub fn selection_of_the_part_range_of_the_flow(
    flow: &CrashFlow<'_>,
    domains: &[DomainOfARegisteredSegment],
    part_range: &PartRangeOfASlicedFlow,
    blocks_per_part: BlocksPerPartAsAPowerOfTwo,
) -> Result<SelectionOfAPartRange, PartRangeRefused> {
    let placeholder_slices =
        placeholder_slices_of_the_sliced_segments(flow.segments.len(), domains);
    let plan_of_placeholder_slices = plan_crash_points_with_slices(flow, &placeholder_slices);
    selection_of_the_part_range(
        &plan_of_placeholder_slices,
        flow.segments.len(),
        part_range,
        blocks_per_part,
    )
}

/// 这一趟这台机器判的块，按次序：落在这一片（`block_range`）里、又归这一份（`share_index`，按 `host_shares` 平滑轮转，块序号照整条流的编号）的块。
#[must_use]
pub fn blocks_of_this_share_in_the_block_range(
    blocks_of_the_flow: u64,
    host_shares: &WeightedShares,
    share_index: usize,
    block_range: &BlockRangeOfTheFlow,
) -> Vec<u64> {
    block_range
        .block_sequences(blocks_of_the_flow)
        .filter(|block| host_shares.owner_of_block(*block) == share_index)
        .collect()
}

#[cfg(test)]
mod tests;
