use super::*;

// ───────────────────────────── 读一条流：规模、计划、块键 ─────────────────────────────

/// 一条流读完的结果：`name=flow` 那一行，两种展开各自的计划，和读的过程中发现的、crates 两条路给的数对不上的地方。
#[derive(Debug)]
pub(super) struct FlowReading {
    pub(super) row: FlowRow,
    pub(super) full: Option<PlannedBlocks>,
    pub(super) in_place_subsets: Option<PlannedBlocks>,
    /// 登记停机 T4：同一条流上 crates 的两条路给的全量态数不等（计划的末节点序号尾对计数函数），或各块态数之和与整条流的态数不等。
    pub(super) count_disagreements: Vec<String>,
}

fn count_disagreements_of(
    label: &str,
    expansion: Expansion,
    counted: Option<u64>,
    plan: &PlannedBlocks,
) -> Vec<String> {
    let mut disagreements = Vec::new();
    if counted != Some(plan.state_count) {
        disagreements.push(format!(
            "{label} {}：本地按段数的态数 {counted:?}，计划的末节点序号尾 {}",
            expansion.name(),
            plan.state_count
        ));
    }
    if plan.state_count != plan.state_count_by_the_counting_function {
        disagreements.push(format!(
            "{label} {}：计划的末节点序号尾 {}，计数函数 {}",
            expansion.name(),
            plan.state_count,
            plan.state_count_by_the_counting_function
        ));
    }
    let block_total: u64 = plan.blocks.iter().map(|block| block.states).sum();
    if block_total != plan.state_count {
        disagreements.push(format!(
            "{label} {}：各块态数之和 {block_total}，整条流的态数 {}",
            expansion.name(),
            plan.state_count
        ));
    }
    disagreements
}

/// 读一条流：规模（写数、段数、最长段、三种态数）和两种展开的计划。`base` 是这条流的基镜像（直接录出来的池，或由转储重建的）。
///
/// # Errors
/// 计划排不了（写表里没有根槽写）。
///
/// # Panics
/// `plan_crash_points` 自己的断言（调用方包一层 `catch_unwind`）。
pub(super) fn read_the_flow(dump: &FlowDump, base: &MemoryPool) -> Result<FlowReading, String> {
    read_the_flow_under(dump, base, MOST_BLOCKS_PLANNED_PER_FLOW)
}

/// [`read_the_flow`]，块数上界的上限由调用方给。态数照报；块数上界超过上限的展开不排计划，`blocks_*` 写 `not_planned`，
/// 与「装不进 u64」的流同一种待遇（对照里要计划的几条也跟着不跑）。
pub(super) fn read_the_flow_under(
    dump: &FlowDump,
    base: &MemoryPool,
    most_blocks: u64,
) -> Result<FlowReading, String> {
    let full = state_count_of_the_flow(base, dump, Expansion::Full);
    let in_place_subsets = state_count_of_the_flow(base, dump, Expansion::InPlaceSubsets);
    let two_state = two_state_closed_form(&dump.segments);
    let started = Instant::now();
    let full_plan = state_count_to_plan(base, dump, Expansion::Full, most_blocks)
        .map(|_| plan_blocks_under(base, dump, Expansion::Full, default_item(), most_blocks))
        .transpose()?;
    let in_place_subsets_plan =
        state_count_to_plan(base, dump, Expansion::InPlaceSubsets, most_blocks)
            .map(|_| {
                plan_blocks_under(
                    base,
                    dump,
                    Expansion::InPlaceSubsets,
                    default_item(),
                    most_blocks,
                )
            })
            .transpose()?;
    let keys_seconds = started.elapsed().as_secs_f64();
    let mut count_disagreements = Vec::new();
    if let Some(plan) = &full_plan {
        count_disagreements.extend(count_disagreements_of(
            &dump.label,
            Expansion::Full,
            full,
            plan,
        ));
    }
    if let Some(plan) = &in_place_subsets_plan {
        count_disagreements.extend(count_disagreements_of(
            &dump.label,
            Expansion::InPlaceSubsets,
            in_place_subsets,
            plan,
        ));
    }
    let row = FlowRow {
        family: dump.family.clone(),
        tier: dump.tier.clone(),
        label: dump.label.clone(),
        seed: dump.seed,
        ending: dump.ending.clone(),
        writes: dump.writes.len(),
        segments: dump.segments.len(),
        longest_segment: longest_segment_of(&dump.segments),
        two_state,
        full,
        in_place_subsets,
        blocks_full: full_plan.as_ref().map(|plan| plan.blocks.len()),
        blocks_in_place_subsets: in_place_subsets_plan.as_ref().map(|plan| plan.blocks.len()),
        points: full_plan
            .as_ref()
            .or(in_place_subsets_plan.as_ref())
            .map(|plan| plan.point_count),
        keys_seconds,
        two_state_agrees_with_crates: two_state
            .map(|local| local == closed_form_state_count_of_write_index_segments(&dump.segments)),
    };
    Ok(FlowReading {
        row,
        full: full_plan,
        in_place_subsets: in_place_subsets_plan,
        count_disagreements,
    })
}

#[cfg(test)]
mod tests;
