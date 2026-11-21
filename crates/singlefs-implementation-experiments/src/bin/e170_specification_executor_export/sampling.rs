//! 取样（登记 5.4）：三批取样点与撕裂扫描。

use super::*;

// ───────────────────────────── 取样（登记 5.4） ─────────────────────────────

/// 批 1 在一个非空区间 [a, b) 里取的点：a、b − 1 与 a + ⌊i·(b − 1 − a)/7⌋（i = 1..6）。
fn first_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
    let first = state_range.start;
    let span = state_range.end - 1 - first;
    let mut points = vec![first, state_range.end - 1];
    for step in 1..FIRST_BATCH_DIVISOR {
        points.push(first + step * span / FIRST_BATCH_DIVISOR);
    }
    points
}

/// 批 2：a + ⌊(2i + 1)·(b − 1 − a)/14⌋（i = 0..6）。
fn second_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
    let first = state_range.start;
    let span = state_range.end - 1 - first;
    (0..SECOND_BATCH_DIVISOR / 2)
        .map(|step| first + (2 * step + 1) * span / SECOND_BATCH_DIVISOR)
        .collect()
}

/// 批 3：a + ⌊i·(b − 1 − a)/23⌋（i = 0..23）。
fn third_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
    let first = state_range.start;
    let span = state_range.end - 1 - first;
    (0..=THIRD_BATCH_DIVISOR)
        .map(|step| first + step * span / THIRD_BATCH_DIVISOR)
        .collect()
}

/// 带撕裂写的那一段扫了几个序号、取到几个带「1」的状态（登记 5.4「扫不到 4 个的照实报个数」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TornScan {
    pub(super) segment_index: usize,
    pub(super) scanned: u64,
    pub(super) found: usize,
}

/// 三批的状态序号（各自升序、互不重叠）与撕裂扫描的记录。
pub(super) struct SampledOrdinals {
    pub(super) batches: [BTreeSet<u64>; 3],
    pub(super) torn_scans: Vec<TornScan>,
}

pub(super) fn sample_ordinals(
    state_ranges_by_segment: &[std::ops::Range<u64>],
    state_count: u64,
    segment_has_a_torn_write: &[bool],
    landing_vector_of_ordinal: &dyn Fn(u64) -> String,
) -> SampledOrdinals {
    let non_empty = || {
        state_ranges_by_segment
            .iter()
            .filter(|range| !range.is_empty())
    };
    let mut first: BTreeSet<u64> = non_empty().flat_map(first_batch_points).collect();
    first.insert(state_count - 1);
    let mut torn_scans = Vec::new();
    for (segment_index, state_range) in state_ranges_by_segment.iter().enumerate() {
        if state_range.is_empty() || !segment_has_a_torn_write[segment_index] {
            continue;
        }
        let scan_end = state_range
            .end
            .min(state_range.start + TORN_SCAN_ORDINALS_PER_SEGMENT);
        let mut found = 0usize;
        let mut scanned = 0u64;
        for ordinal in state_range.start..scan_end {
            if found == TORN_STATES_PER_SEGMENT {
                break;
            }
            scanned += 1;
            if landing_vector_of_ordinal(ordinal).contains('1') && first.insert(ordinal) {
                found += 1;
            }
        }
        torn_scans.push(TornScan {
            segment_index,
            scanned,
            found,
        });
    }
    let second: BTreeSet<u64> = non_empty()
        .flat_map(second_batch_points)
        .filter(|ordinal| !first.contains(ordinal))
        .collect();
    let third: BTreeSet<u64> = non_empty()
        .flat_map(third_batch_points)
        .filter(|ordinal| !first.contains(ordinal) && !second.contains(ordinal))
        .collect();
    SampledOrdinals {
        batches: [first, second, third],
        torn_scans,
    }
}

#[cfg(test)]
mod tests;
