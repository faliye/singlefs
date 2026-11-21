use super::*;

// ───────────────────────────── 排计划、取块键 ─────────────────────────────

/// 展开方式：全量（每段任意真子集，原地覆写取第三态），或甲二（每段按甲二展开）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Expansion {
    Full,
    InPlaceSubsets,
}

impl Expansion {
    pub(super) fn name(self) -> &'static str {
        match self {
            Expansion::Full => "full",
            Expansion::InPlaceSubsets => "in_place_subsets",
        }
    }

    fn function(
        self,
    ) -> &'static (dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync) {
        match self {
            Expansion::Full => &full_expansion,
            Expansion::InPlaceSubsets => &in_place_subsets_with_copy_on_write_none_or_all_expansion,
        }
    }
}

/// 计划里的一块：块键（crates 的 `PlannedCrashPoint::block_key` 算的）、这一块的态数、它属于计划里第几个崩溃点。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct BlockOfAPlan {
    pub(super) key: VerdictBlockKey,
    pub(super) states: u64,
    pub(super) point_index: usize,
}

/// 一条流在一种展开下排出来的计划，只留这个装置要数的东西。
#[derive(Debug)]
pub(super) struct PlannedBlocks {
    /// 计划里最后一个点（流的末节点）的状态序号尾：整条流的状态数。
    pub(super) state_count: u64,
    /// `crash_state_count_with_torn_in_place_overwrites` 数的整条流状态数（登记停机 T4：与上一个相等）。
    pub(super) state_count_by_the_counting_function: u64,
    pub(super) point_count: usize,
    pub(super) blocks: Vec<BlockOfAPlan>,
    /// 不管任何段的点（末节点在内）的块数：两种展开的计划哈希在这些点上相同（登记 5.2「同流两种展开」）。
    pub(super) blocks_of_points_without_segments: usize,
}

/// 整条流在 `expansion` 下的状态数，按 1280 位算；装不进 u64 交 `None`（不 panic：登记 1.1「装不进 u64 记 beyond_u64」）。
pub(super) fn state_count_of_the_flow(
    base: &MemoryPool,
    dump: &FlowDump,
    expansion: Expansion,
) -> Option<u64> {
    let tearable = TearableInPlaceOverwrites::of(base, &dump.writes);
    let mut total = StateCount1280Bits::from_u64(1);
    for (segment_index, segment) in dump.segments.iter().enumerate() {
        let segment_count = expansion.function()(segment_index, segment).checked_state_count(
            &dump.writes,
            segment,
            &tearable,
        )?;
        total = total.checked_add(&segment_count)?;
    }
    total.to_u64()
}

/// 一条流（`segment_count` 段、`states` 个状态）切出的块数的上界：总态数除每块态数向上取整，再加每个点最多一块的零头；
/// 点数不超过段数加末节点（`crash_points_of_the_steps` 一步至少含一段），这里多留 1 的余量。不排计划就能算，
/// 也就是在 `PlannedCrashPoint::blocks` 把块一次摆进内存之前拿它挡。
fn block_count_bound_of(states: u64, segment_count: usize) -> u64 {
    blocks_needed_for(states).saturating_add(
        u64::try_from(segment_count)
            .unwrap_or(u64::MAX)
            .saturating_add(2),
    )
}

/// 整条流在 `expansion` 下要排计划的态数：装得进 u64、且块数上界不超过 `most_blocks` 才有值；
/// 没有值的流照报态数（[`state_count_of_the_flow`]），只是不排计划、不取块键。
pub(super) fn state_count_to_plan(
    base: &MemoryPool,
    dump: &FlowDump,
    expansion: Expansion,
    most_blocks: u64,
) -> Option<u64> {
    state_count_of_the_flow(base, dump, expansion)
        .filter(|states| block_count_bound_of(*states, dump.segments.len()) <= most_blocks)
}

/// 两态闭式 1 + Σ(2^|段| − 1) 的本地复算；有一段不少于 64 写，或总和装不进 u64，交 `None`。
pub(super) fn two_state_closed_form(segments: &[Vec<usize>]) -> Option<u64> {
    let mut total: u64 = 1;
    for segment in segments {
        if segment.len() > LONGEST_SEGMENT_WITH_A_CLOSED_FORM {
            return None;
        }
        let shift = u32::try_from(segment.len()).expect("段写数不超过 63，装得进 u32");
        total = total.checked_add((1u64 << shift) - 1)?;
    }
    Some(total)
}

/// 最长段：段写数的最大值（登记 1.1）。
pub(super) fn longest_segment_of(segments: &[Vec<usize>]) -> usize {
    segments.iter().map(Vec::len).max().unwrap_or(0)
}

/// 按本地的块长，`states` 个状态切几块。
pub(super) fn blocks_needed_for(states: u64) -> u64 {
    states.div_ceil(LOCAL_STATES_PER_BLOCK)
}

/// 按 crates 的算法排计划、取每块的块键：节点照崩溃枚举用例那一路切（`crash_points_of_the_steps`），计划由 `plan_crash_points` 排。
///
/// # Errors
/// 写表里没有根槽 FUA 写（被判的根没有）；整条流的态数装不进 u64；块数上界超过 [`MOST_BLOCKS_PLANNED_PER_FLOW`]。
///
/// # Panics
/// `plan_crash_points` 自己的断言（调用方包一层 `catch_unwind`）。
pub(super) fn plan_blocks(
    base: &MemoryPool,
    dump: &FlowDump,
    expansion: Expansion,
    item: CrashPointReplayItem<'_>,
) -> Result<PlannedBlocks, String> {
    plan_blocks_under(base, dump, expansion, item, MOST_BLOCKS_PLANNED_PER_FLOW)
}

/// [`plan_blocks`]，块数上界的上限由调用方给（单测拿小上限在真流上走拒绝那一支）。
pub(super) fn plan_blocks_under(
    base: &MemoryPool,
    dump: &FlowDump,
    expansion: Expansion,
    item: CrashPointReplayItem<'_>,
    most_blocks: u64,
) -> Result<PlannedBlocks, String> {
    let judged_root_index = dump
        .judged_root_index
        .ok_or_else(|| format!("{}：写表里没有根槽 FUA 写，排不了计划", dump.label))?;
    match state_count_of_the_flow(base, dump, expansion) {
        None => return Err(format!("{}：整条流的态数装不进 u64，不排计划", dump.label)),
        Some(states) => {
            let bound = block_count_bound_of(states, dump.segments.len());
            if bound > most_blocks {
                return Err(format!(
                    "{}：整条流 {states} 个状态，块数上界 {bound} 超过上限 {most_blocks}，不排计划",
                    dump.label
                ));
            }
        }
    }
    let flow = CrashFlow {
        base,
        writes: &dump.writes,
        segments: &dump.segments,
        judged_root_index,
        versions: &dump.versions_with_a_file,
        versions_without_a_file: &dump.versions_without_a_file,
        expansion: expansion.function(),
        crash_points: crash_points_of_the_steps(item, &dump.writes, &dump.segments),
    };
    let plan = plan_crash_points(&flow);
    let mut blocks = Vec::new();
    let mut blocks_of_points_without_segments = 0usize;
    for (point_index, point) in plan.points.iter().enumerate() {
        for range in point.blocks() {
            blocks.push(BlockOfAPlan {
                key: point.block_key(&range),
                states: range.end - range.start,
                point_index,
            });
            if point.segments.is_empty() {
                blocks_of_points_without_segments += 1;
            }
        }
    }
    Ok(PlannedBlocks {
        state_count: plan
            .points
            .last()
            .map_or(0, |end_of_flow| end_of_flow.ordinals.end),
        state_count_by_the_counting_function: crash_state_count_with_torn_in_place_overwrites(
            base,
            &dump.writes,
            &dump.segments,
            expansion.function(),
        ),
        point_count: plan.points.len(),
        blocks,
        blocks_of_points_without_segments,
    })
}

/// 默认的项：名字与设点文件都是这个装置自己的（不进块键）。
pub(super) fn default_item() -> CrashPointReplayItem<'static> {
    CrashPointReplayItem {
        name: ITEM_NAME_OF_THE_FLOWS,
        setting_file: SETTING_FILE_OF_THE_FLOWS,
    }
}

#[cfg(test)]
mod tests;
