//! 第 7 次重跑：洞数各档的峰值增长、阳性对照与各档单元的判定。

use super::*;

/// W28：档序列的判定。输入是按注入洞数升序的各档 Q1a 峰值，不看臂的定义（登记第一节 1.1）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum TierGrowth {
    /// 相邻差全 ≥ 0 且最后一个相邻差 > 0：到最后一档仍在涨，没见上界。
    StillRising,
    /// 相邻差全 ≥ 0、最后一个相邻差 = 0：最后两档已平，这几档里的上界 = 最后一档的峰值。
    FlatAtTheLastTiers {
        upper_bound: i64,
    },
    NotMonotonic,
    FewerThanThreeTiers,
}

impl TierGrowth {
    pub(super) fn label(self) -> String {
        match self {
            TierGrowth::StillRising => "still_rising".to_string(),
            TierGrowth::FlatAtTheLastTiers { upper_bound } => {
                format!("flat_upper_bound_{upper_bound}")
            }
            TierGrowth::NotMonotonic => "not_monotonic".to_string(),
            TierGrowth::FewerThanThreeTiers => "fewer_than_three_tiers".to_string(),
        }
    }

    /// 不带上界数值的判定类别，几何敏感性与判别力自证按它比。
    pub(super) fn kind(self) -> &'static str {
        match self {
            TierGrowth::StillRising => "still_rising",
            TierGrowth::FlatAtTheLastTiers { .. } => "flat",
            TierGrowth::NotMonotonic => "not_monotonic",
            TierGrowth::FewerThanThreeTiers => "fewer_than_three_tiers",
        }
    }
}

fn judge_tier_peaks(peaks_in_ascending_tier_order: &[i64]) -> TierGrowth {
    if peaks_in_ascending_tier_order.len() < 3 {
        return TierGrowth::FewerThanThreeTiers;
    }
    let differences: Vec<i64> = peaks_in_ascending_tier_order
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .collect();
    if differences.iter().any(|difference| *difference < 0) {
        return TierGrowth::NotMonotonic;
    }
    let last_difference = *differences.last().expect("三档至少有两个相邻差");
    if last_difference > 0 {
        TierGrowth::StillRising
    } else {
        TierGrowth::FlatAtTheLastTiers {
            upper_bound: *peaks_in_ascending_tier_order.last().expect("至少三档"),
        }
    }
}

/// PC-档序列的合成用例（登记 5.3）：(说明, 实际, 应得)。
pub(super) fn tier_sequence_positive_control_cases() -> Vec<(&'static str, String, String)> {
    vec![
        (
            "rising_one_two_three",
            judge_tier_peaks(&[1, 2, 3]).label(),
            "still_rising".to_string(),
        ),
        (
            "flat_one_two_two",
            judge_tier_peaks(&[1, 2, 2]).label(),
            "flat_upper_bound_2".to_string(),
        ),
        (
            "not_monotonic_one_three_two",
            judge_tier_peaks(&[1, 3, 2]).label(),
            "not_monotonic".to_string(),
        ),
        (
            "flat_five_five_five",
            judge_tier_peaks(&[5, 5, 5]).label(),
            "flat_upper_bound_5".to_string(),
        ),
        (
            "two_tiers_are_refused",
            judge_tier_peaks(&[1, 2]).label(),
            "fewer_than_three_tiers".to_string(),
        ),
    ]
}

/// PC-同槽（登记 5.3、E25）：把洞的环槽当作这个进程在第二个洞的挂载里写过——那一槽里的旧根不再算数，门槛按去掉它之后重算。
pub(super) fn valid_root_txgs_without_the_old_root_in_the_slot(
    valid_root_txgs: &BTreeSet<u64>,
    old_root_txg_in_the_slot: Option<u64>,
) -> BTreeSet<u64> {
    valid_root_txgs
        .iter()
        .copied()
        .filter(|txg| Some(*txg) != old_root_txg_in_the_slot)
        .collect()
}

/// R7 的五个量：每个量是一类档序列（登记第六节）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum TierQuantity {
    /// R7a：读法甲，前位置、全部洞落在第一圈里的各档。
    InsideTheFirstRing,
    /// R7b：后位置的各档。
    BackPosition,
    /// R7c：读法乙，前位置各档（含第一个落进第二圈的洞）接 Hc 的两档。
    AcrossRings,
    /// R7d：同槽叠洞，前位置。
    SameSlotFront,
    /// R7d：同槽叠洞，后位置。
    SameSlotBack,
}

pub(super) const E156_TIER_QUANTITIES: [TierQuantity; 5] = [
    TierQuantity::InsideTheFirstRing,
    TierQuantity::BackPosition,
    TierQuantity::AcrossRings,
    TierQuantity::SameSlotFront,
    TierQuantity::SameSlotBack,
];

impl TierQuantity {
    pub(super) fn label(self) -> &'static str {
        match self {
            TierQuantity::InsideTheFirstRing => "r7a",
            TierQuantity::BackPosition => "r7b",
            TierQuantity::AcrossRings => "r7c",
            TierQuantity::SameSlotFront => "r7d_front",
            TierQuantity::SameSlotBack => "r7d_back",
        }
    }

    fn class_label(self) -> &'static str {
        match self {
            TierQuantity::InsideTheFirstRing => "Hh-front-inside-the-first-ring",
            TierQuantity::BackPosition => "Hh-back",
            TierQuantity::AcrossRings => "Hh-front-then-Hc-front",
            TierQuantity::SameSlotFront => "Hs-front",
            TierQuantity::SameSlotBack => "Hs-back",
        }
    }

    fn selects(self, outcome: &HhRunOutcome, front_hole_count_inside_the_first_ring: u64) -> bool {
        let position = outcome.shape.position;
        let hole_count = outcome.shape.hole_count;
        let is_family = |family: &str| outcome.family == family;
        match self {
            TierQuantity::InsideTheFirstRing => {
                is_family(E156_FAMILY_HOLE_COUNT_SWEEP)
                    && position == Some(HolePosition::Front)
                    && hole_count <= front_hole_count_inside_the_first_ring
            }
            TierQuantity::BackPosition => {
                is_family(E156_FAMILY_HOLE_COUNT_SWEEP) && position == Some(HolePosition::Back)
            }
            TierQuantity::AcrossRings => {
                (is_family(E156_FAMILY_HOLE_COUNT_SWEEP)
                    && position == Some(HolePosition::Front)
                    && hole_count <= front_hole_count_inside_the_first_ring + 1)
                    || (is_family(E156_FAMILY_CONTINUED_SWEEP)
                        && position == Some(HolePosition::Front))
            }
            TierQuantity::SameSlotFront => {
                is_family(E156_FAMILY_SAME_RING_SLOT) && position == Some(HolePosition::Front)
            }
            TierQuantity::SameSlotBack => {
                is_family(E156_FAMILY_SAME_RING_SLOT) && position == Some(HolePosition::Back)
            }
        }
    }
}

/// 一个统计单位 (量, S, ρ, 回收时点) 的判定结果。
#[derive(Clone, Debug)]
pub(super) struct TierUnit {
    pub(super) quantity: TierQuantity,
    pub(super) slots_per_region: u64,
    pub(super) density: WorkloadDensity,
    pub(super) timing: ReclaimTiming,
    pub(super) growth: TierGrowth,
    pub(super) growth_without_the_last_tier: TierGrowth,
    /// 门槛挪到两个观测值之间，第一个越过的档由有变无：`Some(true)`；少于两个不同的峰值做不出：`None`。
    pub(super) threshold_moved_flips: Option<bool>,
}

impl TierUnit {
    pub(super) fn sampling_point_name(&self) -> String {
        format!(
            "s={}_rho={}_timing={}",
            self.slots_per_region,
            self.density.label(),
            self.timing.label()
        )
    }
}

/// W27：Q1a（甲-T1 多扣，「实」「每」两个回收时点各一份）在一条历史上的峰值与第一次到峰值的 txg，峰值取全部观测点。
pub(super) fn q1a_peak_of(outcome: &HhRunOutcome, timing: ReclaimTiming) -> (i64, u64) {
    let mut peak = 0i64;
    let mut txg_of_peak = 0u64;
    for (index, observation) in outcome.observations.iter().enumerate() {
        let value =
            i64::try_from(observation.over_withheld(Arm::FloorRule, timing)).expect("落在 i64 内");
        if index == 0 || value > peak {
            peak = value;
            txg_of_peak = observation.txg;
        }
    }
    (peak, txg_of_peak)
}

fn first_tier_with_peak_above(tiers: &[u64], peaks: &[i64], threshold_slots: i64) -> Option<u64> {
    tiers
        .iter()
        .zip(peaks)
        .find(|(_, peak)| **peak > threshold_slots)
        .map(|(tier, _)| *tier)
}

/// 第 4 次 8.2 ② 的做法：门槛挪到两个不同的峰值之间，第一个越过的档必须由有变无。
pub(super) fn threshold_moved_flips(tiers: &[u64], peaks: &[i64]) -> Option<bool> {
    let distinct: BTreeSet<i64> = peaks.iter().copied().collect();
    if distinct.len() < 2 {
        return None;
    }
    let largest = *distinct.iter().next_back().expect("至少两个不同的峰值");
    let second_largest = *distinct.iter().rev().nth(1).expect("至少两个不同的峰值");
    Some(
        first_tier_with_peak_above(tiers, peaks, second_largest)
            != first_tier_with_peak_above(tiers, peaks, largest),
    )
}

pub(super) fn bracketed<Value: ToString>(values: &[Value]) -> String {
    let parts: Vec<String> = values.iter().map(ToString::to_string).collect();
    format!("[{}]", parts.join(","))
}

fn option_text(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_string(), |tier| tier.to_string())
}

fn judge_and_emit_one_tier_unit(
    emitter: &mut Emitter,
    quantity: TierQuantity,
    slots_per_region: u64,
    density: WorkloadDensity,
    timing: ReclaimTiming,
    cells: &[&HhRunOutcome],
) -> TierUnit {
    assert!(
        !cells.is_empty(),
        "统计单位 {} 在 s={slots_per_region} 上一格都没有",
        quantity.label()
    );
    let tiers: Vec<u64> = cells
        .iter()
        .map(|outcome| outcome.shape.hole_count)
        .collect();
    let families: Vec<&str> = cells.iter().map(|outcome| outcome.family).collect();
    let readings: Vec<(i64, u64)> = cells
        .iter()
        .map(|outcome| q1a_peak_of(outcome, timing))
        .collect();
    let peaks: Vec<i64> = readings.iter().map(|(peak, _)| *peak).collect();
    let growth = judge_tier_peaks(&peaks);
    let growth_without_the_last_tier = judge_tier_peaks(&peaks[..peaks.len() - 1]);
    let maximum_peak = *peaks.iter().max().expect("至少一档");
    let maximum_index = peaks
        .iter()
        .position(|peak| *peak == maximum_peak)
        .expect("最大值在序列里");
    let adjacent_differences: Vec<i64> = peaks.windows(2).map(|pair| pair[1] - pair[0]).collect();
    let moved_threshold_flips = threshold_moved_flips(&tiers, &peaks);
    let unit = TierUnit {
        quantity,
        slots_per_region,
        density,
        timing,
        growth,
        growth_without_the_last_tier,
        threshold_moved_flips: moved_threshold_flips,
    };
    emitter.emit(&format!(
        "name=r7_tier_peaks quantity={} class={} s={slots_per_region} rho={} timing={} tiers={} families={} peaks={} adjacent_differences={} growth={} maximum_peak={maximum_peak} maximum_peak_tier={} maximum_peak_family={} maximum_peak_cell={} maximum_peak_txg={} maximum_peak_is_on_the_largest_tier={} first_tier_above_two_slots={} first_tier_above_one_slot={} growth_without_the_last_tier={} dropping_the_last_tier_changes_the_judgement={} threshold_moved_flips={}",
        quantity.label(),
        quantity.class_label(),
        density.label(),
        timing.label(),
        bracketed(&tiers),
        bracketed(&families),
        bracketed(&peaks),
        bracketed(&adjacent_differences),
        growth.label(),
        tiers[maximum_index],
        families[maximum_index],
        unit.sampling_point_name(),
        readings[maximum_index].1,
        maximum_index + 1 == peaks.len(),
        option_text(first_tier_with_peak_above(
            &tiers,
            &peaks,
            E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS
        )),
        option_text(first_tier_with_peak_above(
            &tiers,
            &peaks,
            E156_THRESHOLD_ONE_SPAN_ONE_UNIT_SLOTS
        )),
        growth_without_the_last_tier.label(),
        growth.kind() != growth_without_the_last_tier.kind(),
        unit.threshold_moved_flips
            .map_or_else(|| "not_made".to_string(), |flips| flips.to_string()),
    ));
    unit
}

/// R7a–R7e：按量、S、ρ、回收时点逐个统计单位判 W28、报上界数。
pub(super) fn judge_and_emit_tier_units(
    emitter: &mut Emitter,
    outcomes: &[HhRunOutcome],
    hole_family_anchors: &BTreeMap<u64, HoleFamilyAnchorCellsOfOneSlotsPerRegion>,
) -> Vec<TierUnit> {
    let mut units = Vec::new();
    for quantity in E156_TIER_QUANTITIES {
        for slots_per_region in E156_SLOTS_PER_REGION_SAMPLING_POINTS {
            let inside_the_first_ring =
                hole_family_anchors[&slots_per_region].front_hole_count_inside_the_first_ring;
            for density in [
                WorkloadDensity::EveryPublishOverwrites,
                WorkloadDensity::OneOverwriteInFourPublishes,
            ] {
                let mut cells: Vec<&HhRunOutcome> = outcomes
                    .iter()
                    .filter(|outcome| {
                        outcome.shape.slots_per_region == slots_per_region
                            && outcome.shape.density == density
                            && quantity.selects(outcome, inside_the_first_ring)
                    })
                    .collect();
                cells.sort_by_key(|outcome| outcome.shape.hole_count);
                for timing in [ReclaimTiming::Real, ReclaimTiming::EveryPublish] {
                    units.push(judge_and_emit_one_tier_unit(
                        emitter,
                        quantity,
                        slots_per_region,
                        density,
                        timing,
                        &cells,
                    ));
                }
            }
        }
    }
    units
}

#[cfg(test)]
mod tests;
