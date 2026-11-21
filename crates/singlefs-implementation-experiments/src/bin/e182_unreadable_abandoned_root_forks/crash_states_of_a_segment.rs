use super::*;

// ============================================================================
// 八、崩溃状态：按崩溃点重放同一套取法切段（`writes_and_segments_with_stream_indexes`）；一个状态 = 之前的段全持久 + 这一段的一个子集。
//    段内写不超过 `FULL_EXPANSION_WRITES_AT_MOST_LOCAL` 条的段全量展开（任意真子集）；超过的取甲二
//    （`crash::in_place_subsets_with_copy_on_write_none_or_all_expansion` 的 `InPlaceSubsetsWithCopyOnWriteNoneOrAll`：段内原地写任意子集 × 单元写只取全不落或全落）。
//    全持久那个状态是下一段的空子集，整条流末尾另算一个。这里的状态只有「没持久 / 持久」两态（登记「全部崩溃状态」写的是任意子集，
//    I2 的 2^m 也是这个口径），不取原地覆写的第三态「新旧都读不出」。
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Expansion {
    FullSubsets,
    QuickInPlaceSubsetsTimesUnitWritesNoneOrAll,
    FinalState,
}

impl Expansion {
    pub(super) fn name(self) -> &'static str {
        match self {
            Expansion::FullSubsets => "full_subsets",
            Expansion::QuickInPlaceSubsetsTimesUnitWritesNoneOrAll => {
                "quick_in_place_subsets_times_unit_writes_none_or_all"
            }
            Expansion::FinalState => "final_state",
        }
    }
}

pub(super) fn expansion_of_segment(segment_write_count: usize) -> Expansion {
    if segment_write_count <= FULL_EXPANSION_WRITES_AT_MOST_LOCAL {
        Expansion::FullSubsets
    } else {
        Expansion::QuickInPlaceSubsetsTimesUnitWritesNoneOrAll
    }
}

/// 一段里的原地写与单元写（单元写落在新分出来的槽上，其余都原地覆盖；与崩溃点重放引擎同一条分法）。
fn in_place_and_unit_writes_of(
    writes: &[RetainedWrite],
    segment: &[usize],
) -> (Vec<usize>, Vec<usize>) {
    segment
        .iter()
        .copied()
        .partition(|write_index| writes[*write_index].kind != StepKind::UnitWrite)
}

/// 这一段按它的展开方式出几个状态（全持久那个不算）；展不开（指数越过 62）交 `None`。
pub(super) fn state_count_of_segment(writes: &[RetainedWrite], segment: &[usize]) -> Option<u64> {
    match expansion_of_segment(segment.len()) {
        Expansion::FullSubsets => {
            let exponent = u32::try_from(segment.len())
                .ok()
                .filter(|bits| *bits <= 62)?;
            Some((1u64 << exponent) - 1)
        }
        Expansion::QuickInPlaceSubsetsTimesUnitWritesNoneOrAll => {
            let (in_place, unit_writes) = in_place_and_unit_writes_of(writes, segment);
            let exponent = u32::try_from(in_place.len())
                .ok()
                .filter(|bits| *bits <= 40)?;
            let unit_write_choices: u64 = if unit_writes.is_empty() { 1 } else { 2 };
            Some((1u64 << exponent) * unit_write_choices - 1)
        }
        Expansion::FinalState => Some(0),
    }
}

/// 第 `segment_index` 段的每个状态：交 `visit` 一份「写表里每次写持不持久」。返回出了几个状态。
/// `segment_index` 等于段数时是整条流末尾那个全持久的状态。
pub(super) fn for_each_state_of_segment(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    segment_index: usize,
    visit: &mut dyn FnMut(&[bool]) -> Result<(), String>,
) -> Result<u64, String> {
    let mut persisted = vec![false; writes.len()];
    for earlier in segments.iter().take(segment_index) {
        for write_index in earlier {
            persisted[*write_index] = true;
        }
    }
    let Some(segment) = segments.get(segment_index) else {
        visit(&persisted_with_every_write(writes.len()))?;
        return Ok(1);
    };
    let expected = state_count_of_segment(writes, segment).ok_or_else(|| {
        format!(
            "第 {segment_index} 段（{} 次写）的状态数展不开",
            segment.len()
        )
    })?;
    let mut visited: u64 = 0;
    match expansion_of_segment(segment.len()) {
        Expansion::FullSubsets => {
            // 全 1 的掩码 2^m − 1（= expected）就是全持久，已经是下一段的空子集，所以只取 0..expected。
            for mask in 0..expected {
                for (bit, write_index) in segment.iter().enumerate() {
                    persisted[*write_index] = (mask >> bit) & 1 == 1;
                }
                visit(&persisted)?;
                visited += 1;
            }
        }
        Expansion::QuickInPlaceSubsetsTimesUnitWritesNoneOrAll => {
            let (in_place, unit_writes) = in_place_and_unit_writes_of(writes, segment);
            let unit_write_choices: u64 = if unit_writes.is_empty() { 1 } else { 2 };
            let in_place_masks: u64 = 1u64 << in_place.len();
            for unit_choice in 0..unit_write_choices {
                for write_index in &unit_writes {
                    persisted[*write_index] = unit_choice == 1;
                }
                for mask in 0..in_place_masks {
                    if unit_choice + 1 == unit_write_choices && mask + 1 == in_place_masks {
                        continue;
                    }
                    for (bit, write_index) in in_place.iter().enumerate() {
                        persisted[*write_index] = (mask >> bit) & 1 == 1;
                    }
                    visit(&persisted)?;
                    visited += 1;
                }
            }
        }
        Expansion::FinalState => {}
    }
    if visited != expected {
        return Err(format!(
            "第 {segment_index} 段枚举出 {visited} 个状态，闭式是 {expected}"
        ));
    }
    Ok(visited)
}

pub(super) fn persisted_with_every_write(write_count: usize) -> Vec<bool> {
    vec![true; write_count]
}

#[cfg(test)]
mod tests;
