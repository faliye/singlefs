//! 取样域（登记 5.5）。

use super::*;

// ───────────────────────────── 取样域（5.5） ─────────────────────────────

/// 这条流的取样段（新池新建文件流全部；覆盖写固定脚本流段 12–28）。
pub(super) fn is_sampled_segment(stream: StreamName, segment_index: usize) -> bool {
    match stream {
        StreamName::First => true,
        StreamName::Second => (SECOND_STREAM_FIRST_SAMPLED_SEGMENT
            ..=SECOND_STREAM_LAST_SAMPLED_SEGMENT)
            .contains(&segment_index),
    }
}

/// 全部持久那一个在不在域里：新池新建文件流在（新池新建文件流全量 16777268 含它），覆盖写固定脚本流不在（段 29 起不取）。
pub(super) fn includes_every_write_persisted(stream: StreamName) -> bool {
    match stream {
        StreamName::First => true,
        StreamName::Second => false,
    }
}

pub(super) fn domain_expansions(
    stream: &PreparedStream,
    expansion: LocalExpansion,
) -> Vec<LocalExpansion> {
    (0..stream.segments.len())
        .map(|segment_index| {
            if is_sampled_segment(stream.name, segment_index) {
                expansion
            } else {
                LocalExpansion::Skipped
            }
        })
        .collect()
}

/// 大段的写数：24 写段（D1-stride 取步长、可行性档取段头的那两段：新池新建文件流段 8、覆盖写固定脚本流段 12）。
/// 重跑登记读法甲：第 1 次的 26 写段（3² · 2²⁴ − 1 个状态）在 C577 之后拆成轮换 2 写与单元 24 写，阈值照改成 24。
pub(super) const STRIDED_SEGMENT_WRITES: usize = 24;
/// 对照样本里「大段」的写数下限（18 写段与 24 写段，5.5）。
const CONTROL_SAMPLE_LARGE_SEGMENT_WRITES: usize = 18;

/// D1-small：取样段里写数少于 24 的每一段的全部状态（新池新建文件流再加全部持久那一个）；按全量计划的序号。
pub(super) fn small_domain_ordinals(stream: &PreparedStream, plan: &LocalPlan<'_>) -> Vec<u64> {
    let mut ordinals = Vec::new();
    for (segment_index, segment) in stream.segments.iter().enumerate() {
        if is_sampled_segment(stream.name, segment_index) && segment.len() < STRIDED_SEGMENT_WRITES
        {
            let first = plan.first_ordinal_by_segment[segment_index];
            ordinals.extend(first..first + plan.count_by_segment[segment_index]);
        }
    }
    if includes_every_write_persisted(stream.name) {
        ordinals.push(plan.expanded_state_count);
    }
    ordinals
}

/// D1-stride：24 写段里段内序号 ≡ 0 (mod 2311) 的状态。
pub(super) fn stride_domain_ordinals(stream: &PreparedStream, plan: &LocalPlan<'_>) -> Vec<u64> {
    let mut ordinals = Vec::new();
    for (segment_index, segment) in stream.segments.iter().enumerate() {
        if is_sampled_segment(stream.name, segment_index) && segment.len() == STRIDED_SEGMENT_WRITES
        {
            let first = plan.first_ordinal_by_segment[segment_index];
            let count = plan.count_by_segment[segment_index];
            ordinals.extend(
                (0..count)
                    .step_by(usize::try_from(STRIDE_STEP).expect("步长"))
                    .map(|within| first + within),
            );
        }
    }
    ordinals
}

/// 全部状态（D1-quick 用：整个甲二域，完整枚举）。
pub(super) fn every_ordinal(stream: &PreparedStream, plan: &LocalPlan<'_>) -> Vec<u64> {
    let last = plan.expanded_state_count + u64::from(includes_every_write_persisted(stream.name));
    (0..last).collect()
}

/// 对照样本（5.5）：D1 里每一段，18 写及以上的段取序号最小的 4096 个，其余段取全部；新池新建文件流的全部持久那一个也在。
pub(super) fn control_ordinals(
    stream: &PreparedStream,
    plan: &LocalPlan<'_>,
    domain_ordinals: &[u64],
) -> BTreeSet<u64> {
    let mut by_segment: BTreeMap<usize, Vec<u64>> = BTreeMap::new();
    for ordinal in domain_ordinals {
        by_segment
            .entry(plan.segment_of(*ordinal))
            .or_default()
            .push(*ordinal);
    }
    let mut chosen = BTreeSet::new();
    for (segment_index, mut ordinals) in by_segment {
        ordinals.sort_unstable();
        let is_large = stream
            .segments
            .get(segment_index)
            .is_some_and(|segment| segment.len() >= CONTROL_SAMPLE_LARGE_SEGMENT_WRITES);
        let taken = if is_large {
            CONTROL_SAMPLE_PER_LARGE_SEGMENT.min(ordinals.len())
        } else {
            ordinals.len()
        };
        chosen.extend(ordinals[..taken].iter().copied());
    }
    chosen
}

#[cfg(test)]
mod tests;
