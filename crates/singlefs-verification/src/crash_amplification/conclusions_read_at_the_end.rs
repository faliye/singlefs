//! 一趟收尾时读一次的判定：按发布数计数向量的分歧（[`count_vector_disagreements_of_the_flow`]）、判完的结论读一次与核对器比。

use super::*;

// ============================== 计数向量每一趟都判 ==============================

/// 每个状态归哪次发布，按段的序号区间现算（不逐状态造镜像）：GPU 判器那一路存块时照它记按发布分的态数，每一趟的核也照它算该有多少。
pub(super) struct PublishesOfTheStates {
    ranges_and_publishes: Vec<(Range<u64>, CrashPointReplayPublishOfState)>,
    pub(super) state_count: u64,
}

impl PublishesOfTheStates {
    pub(super) fn of_the_plan(flow: &CrashFlow<'_>, plan: &CrashPlan) -> Self {
        let expansion = |segment_index: usize, segment: &[usize]| {
            plan.segment_expansion(flow, segment_index, segment)
        };
        let (ranges_by_segment, state_count) = crash_state_ranges_by_segment_with_slices(
            flow.base,
            flow.writes,
            flow.segments,
            &expansion,
            plan.segment_slices(),
        );
        Self {
            ranges_and_publishes: ranges_by_segment
                .into_iter()
                .zip(publish_of_each_segment(flow.writes, flow.segments))
                .collect(),
            state_count,
        }
    }

    /// 整条流序号 `ordinals` 那一段里按发布分的态数：每段与它重叠的那几个归这一段的那次发布，最后全部持久那一个（序号是状态数减一，
    /// 不在任何一段的区间里）归 [`CrashPointReplayPublishOfState::EveryWritePersisted`]。
    pub(super) fn state_counts_of(&self, ordinals: &Range<u64>) -> StateCountsOfABlock {
        let mut counts = StateCountsOfABlock::default();
        for (range, publish) in &self.ranges_and_publishes {
            let overlap = range
                .end
                .min(ordinals.end)
                .saturating_sub(range.start.max(ordinals.start));
            counts.count_states_of_a_publish(*publish, overlap);
        }
        if ordinals.start < self.state_count && ordinals.end >= self.state_count {
            counts
                .count_states_of_a_publish(CrashPointReplayPublishOfState::EveryWritePersisted, 1);
        }
        counts
    }
}

/// 这条流这台机器领的每一块（`is_mine`），`conclusions` 那一处的计数向量与计划对不对得上（J7：闭式、按发布分的态数、逐条不变量计数每一趟都判，
/// 这一趟判的与库里复用的一样判）：按发布分的态数逐项等于按段的序号区间现算的（[`PublishesOfTheStates`]，合起来就是这一块的状态数，
/// 全部块合起来就是整条流的闭式）；清单里每条不变量一行，判到 + 不适用 = 状态数、违例不超过判到。一块对不上一句；判法单元指纹那一套
/// （[`ConclusionsStoredUnder::TheJudgingUnitsFingerprint`]）不存计数向量，不判。还没判的块不判（对账另报）。
///
/// # Errors
/// 库读失败。
pub fn count_vector_disagreements_of_the_flow(
    store: &VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    is_mine: &dyn Fn(u64) -> bool,
) -> Result<Vec<String>, VerdictStoreError> {
    let ConclusionsStoredUnder::TheConclusionKind {
        kind,
        version_of_each_point,
    } = conclusions
    else {
        return Ok(Vec::new());
    };
    let publishes = PublishesOfTheStates::of_the_plan(flow, plan);
    let mut disagreements = Vec::new();
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        let version = ConclusionsStoredUnder::conclusion_version_of_the_point(
            version_of_each_point,
            point_index,
        );
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if !is_mine(this_block) {
                continue;
            }
            let Some(stored) =
                store.read_conclusion_block(&point.block_key(&local), kind, &version)?
            else {
                continue;
            };
            disagreements.extend(count_vector_disagreements_of_a_block(
                &publishes, point, &local, &stored,
            ));
        }
    }
    Ok(disagreements)
}

/// 一块判器那一份结论读回来的计数向量与计划对不对得上（[`count_vector_disagreements_of_the_flow`] 与判段收尾那一遍读
/// [`read_the_judged_conclusions_once_at_the_end`] 共用）：按发布分的态数照按段的区间现算、逐条不变量计数的行是池级 checker 的清单、
/// 每条判到加不适用等于块长且违例不多于判到；对不上的逐条一句，按这个次序。
fn count_vector_disagreements_of_a_block(
    publishes: &PublishesOfTheStates,
    point: &PlannedCrashPoint,
    local: &Range<u64>,
    stored: &StoredConclusionBlock,
) -> Vec<String> {
    let mut disagreements = Vec::new();
    let states = local.end - local.start;
    let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
    let expected_states_by_publish = publishes
        .state_counts_of(&ordinals)
        .into_count_vector(states)
        .states_by_publish;
    let place = format!("点 {} 块 {}", point.name, local.start);
    if stored.count_vector.states_by_publish != expected_states_by_publish {
        disagreements.push(format!(
            "{place}：按发布分的态数 {:?}，按段的区间现算该是 {expected_states_by_publish:?}",
            stored.count_vector.states_by_publish
        ));
    }
    let invariant_names: Vec<&str> = stored
        .count_vector
        .per_invariant
        .keys()
        .map(String::as_str)
        .collect();
    let mut expected_names: Vec<&str> = IMPLEMENTED_INVARIANTS.to_vec();
    expected_names.sort_unstable();
    if invariant_names != expected_names {
        disagreements.push(format!(
            "{place}：逐条不变量计数的行与池级 checker 的清单不同（{} 行，清单 {} 条）",
            invariant_names.len(),
            expected_names.len()
        ));
    }
    for (invariant, counts) in &stored.count_vector.per_invariant {
        if counts.judged + counts.not_applicable != states || counts.violated > counts.judged {
            disagreements.push(format!(
                "{place}：{invariant} 判到 {}、违例 {}、不适用 {}，这一块 {states} 个状态",
                counts.judged, counts.violated, counts.not_applicable
            ));
        }
    }
    disagreements
}

/// 判段收尾那一遍读交回的三样（[`read_the_judged_conclusions_once_at_the_end`]）：第 ③ 段的比对（没核过是 `None`）、计数向量对不上的几句、
/// 这一片的违例。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct JudgedConclusionsReadOnceAtTheEnd {
    pub(crate) comparison: Option<JudgeVerifierComparison>,
    pub(crate) count_vector_disagreements: Vec<String>,
    pub(crate) violations: Vec<StoredRedState>,
}

/// 判段收尾那一遍读里第 ③ 段比对要的两样（[`read_the_judged_conclusions_once_at_the_end`]）：核对表那一行的键与要比的块。
pub(crate) struct ComparisonWithTheVerifier<'comparison> {
    pub(crate) verifier_digest: &'comparison [u8; DIGEST_BYTES],
    pub(crate) is_compared: &'comparison dyn Fn(u64) -> bool,
}

/// 判段收尾读判器那一份结论的三样并成一遍：第 ③ 段的比对（[`compare_the_conclusions_and_the_verifier_of_the_blocks`]，`comparison_against` 给了才比：
/// 核对表那一行的键与要比的块）、计数向量（[`count_vector_disagreements_of_the_flow`]，`is_mine` 选的块）、这一片的违例
/// （[`violations_counted_by_the_block_range`]，`block_range` 里的块）。三样选的块的并集里每一块结论块只读一次（[`VerdictStore::read_conclusion_block`]），
/// 交给要它的那几样；比对另读核对表那一行。每一样照块序、照各自的选法记，读的是同几行，交回的与分开读的逐项相同（每块怎么记是两边共用的那几个函数）。
/// 判法单元指纹那一套（流水线各段单测）照分开的三个读。
///
/// # Errors
/// 库读失败。
pub(crate) fn read_the_judged_conclusions_once_at_the_end(
    store: &VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    comparison_against: Option<ComparisonWithTheVerifier<'_>>,
    is_mine: &dyn Fn(u64) -> bool,
    block_range: &BlockRangeOfTheFlow,
) -> Result<JudgedConclusionsReadOnceAtTheEnd, VerdictStoreError> {
    let ConclusionsStoredUnder::TheConclusionKind {
        kind,
        version_of_each_point,
    } = conclusions
    else {
        return Ok(JudgedConclusionsReadOnceAtTheEnd {
            comparison: match comparison_against {
                Some(ComparisonWithTheVerifier {
                    verifier_digest,
                    is_compared,
                }) => Some(compare_the_conclusions_and_the_verifier_of_the_blocks(
                    store,
                    plan,
                    verifier_digest,
                    conclusions,
                    is_compared,
                )?),
                None => None,
            },
            count_vector_disagreements: count_vector_disagreements_of_the_flow(
                store,
                flow,
                plan,
                conclusions,
                is_mine,
            )?,
            violations: read_the_violations_stored_under_among_the_counted_blocks(
                store,
                plan,
                conclusions,
                &|block| block_range.holds(block),
            )?,
        });
    };
    let publishes = PublishesOfTheStates::of_the_plan(flow, plan);
    let mut comparison = comparison_against
        .as_ref()
        .map(|_against| JudgeVerifierComparison::default());
    let mut count_vector_disagreements = Vec::new();
    let mut violations = Vec::new();
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        let version = ConclusionsStoredUnder::conclusion_version_of_the_point(
            version_of_each_point,
            point_index,
        );
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            let compared_against = comparison_against
                .as_ref()
                .filter(|against| (against.is_compared)(this_block))
                .map(|against| against.verifier_digest);
            let is_counted_in_the_count_vectors = is_mine(this_block);
            let is_counted_in_the_violations = block_range.holds(this_block);
            if compared_against.is_none()
                && !is_counted_in_the_count_vectors
                && !is_counted_in_the_violations
            {
                continue;
            }
            let key = point.block_key(&local);
            let stored = store.read_conclusion_block(&key, kind, &version)?;
            if let (Some(verifier_digest), Some(comparison)) =
                (compared_against, comparison.as_mut())
            {
                let verified = store.read_verifier_block(&key, verifier_digest)?;
                compare_a_block_with_the_verifier(
                    store,
                    point,
                    &local,
                    stored
                        .as_ref()
                        .map(|stored| stored.block.verdict_vector_number_of_each_state()),
                    verified.as_ref(),
                    comparison,
                );
            }
            let Some(stored) = stored else {
                continue;
            };
            if is_counted_in_the_count_vectors {
                count_vector_disagreements.extend(count_vector_disagreements_of_a_block(
                    &publishes, point, &local, &stored,
                ));
            }
            if is_counted_in_the_violations {
                violations.extend(red_states_of_a_block(
                    store,
                    point,
                    &local,
                    &stored.violations,
                ));
            }
        }
    }
    Ok(JudgedConclusionsReadOnceAtTheEnd {
        comparison,
        count_vector_disagreements,
        violations,
    })
}

#[cfg(test)]
mod tests;
