//! 一格的汇总（第六节 M1–M6、8.1 轨迹、8.3 差额敏感性）。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// 一格的汇总（第六节 M1–M6、8.1 轨迹、8.3 差额敏感性）
// ---------------------------------------------------------------------------------------------------------------------

/// 一条删除的汇总，全取自逐次读数（两块盘里取大的那一块；S6 已核两盘 W 相等）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DeleteSummary {
    pub(super) publishes: u64,
    pub(super) is_completed: bool,
    /// 被拒那一次：(第几次发布, 种类, 发布之前的 borrow)。
    pub(super) refusal: Option<(u64, DeleteRefusal, i128)>,
    pub(super) apex: i128,
    apex_position: u64,
    pub(super) reserve_at_the_apex: i128,
    reserve_minimum: i128,
    reserve_maximum: i128,
    reserve_at_the_start: i128,
    reserve_at_the_last_publish: i128,
    apex_between: i128,
    apex_between_position: u64,
    publishes_with_a_positive_valley_borrow: u64,
    publishes_with_a_positive_between_borrow: u64,
    borrow_before_the_last_publish: i128,
    pub(super) intent_new_slots: u64,
    pub(super) largest_batch_new_slots: u64,
    pub(super) largest_new_slots: u64,
    pub(super) publishes_whose_new_slots_exceed_the_landing_demand: u64,
    largest_new_slots_per_mille_of_the_landing_demand: u64,
}

fn largest_over_devices(values: &[i128]) -> i128 {
    values.iter().copied().max().expect("至少一块盘")
}

pub(super) fn summary_of_the_delete(run: &DeleteRun) -> DeleteSummary {
    let readings = &run.readings;
    let first = readings.first().expect("至少意图那一次");
    let last = readings.last().expect("至少意图那一次");
    let valley_borrow: Vec<i128> = readings
        .iter()
        .map(|reading| largest_over_devices(&reading.borrow_at_the_valley()))
        .collect();
    let between_borrow: Vec<i128> = readings
        .iter()
        .map(|reading| largest_over_devices(&reading.borrow_between_publishes()))
        .collect();
    let reserve: Vec<i128> = readings
        .iter()
        .map(|reading| largest_over_devices(&reading.reserve_slots))
        .collect();
    let (apex_index, apex) = valley_borrow
        .iter()
        .copied()
        .enumerate()
        .max_by_key(|(_, borrow)| *borrow)
        .expect("有");
    let (apex_between_index, apex_between) = between_borrow
        .iter()
        .copied()
        .enumerate()
        .max_by_key(|(_, borrow)| *borrow)
        .expect("有");
    let new_slots_of = |reading: &DeletePublishReading| {
        reading.new_slots.iter().copied().max().expect("至少一块盘")
    };
    let landing_ratios: Vec<(u64, u64)> = readings
        .iter()
        .filter_map(|reading| {
            reading
                .shape
                .map(|shape| (new_slots_of(reading), shape.landing_demand_slots))
        })
        .collect();
    DeleteSummary {
        publishes: u64::try_from(readings.len()).expect("发布数"),
        is_completed: run.is_completed,
        refusal: last.refusal.map(|refusal| {
            (
                last.position,
                refusal,
                largest_over_devices(&last.borrow_between_publishes()),
            )
        }),
        apex,
        apex_position: readings[apex_index].position,
        reserve_at_the_apex: reserve[apex_index],
        reserve_minimum: reserve.iter().copied().min().expect("有"),
        reserve_maximum: reserve.iter().copied().max().expect("有"),
        reserve_at_the_start: reserve[0],
        reserve_at_the_last_publish: *reserve.last().expect("有"),
        apex_between,
        apex_between_position: readings[apex_between_index].position,
        publishes_with_a_positive_valley_borrow: u64::try_from(
            valley_borrow.iter().filter(|borrow| **borrow > 0).count(),
        )
        .expect("次数"),
        publishes_with_a_positive_between_borrow: u64::try_from(
            between_borrow.iter().filter(|borrow| **borrow > 0).count(),
        )
        .expect("次数"),
        borrow_before_the_last_publish: *between_borrow.last().expect("有"),
        intent_new_slots: new_slots_of(first),
        largest_batch_new_slots: readings[1..].iter().map(new_slots_of).max().unwrap_or(0),
        largest_new_slots: readings.iter().map(new_slots_of).max().expect("有"),
        publishes_whose_new_slots_exceed_the_landing_demand: u64::try_from(
            landing_ratios
                .iter()
                .filter(|(new_slots, demand)| new_slots > demand)
                .count(),
        )
        .expect("次数"),
        largest_new_slots_per_mille_of_the_landing_demand: landing_ratios
            .iter()
            .filter(|(_, demand)| *demand > 0)
            .map(|(new_slots, demand)| new_slots * 1000 / demand)
            .max()
            .unwrap_or(0),
    }
}

pub(super) fn summary_fields(summary: &DeleteSummary, slots_per_region: u64) -> String {
    let refusal = summary.refusal.map_or_else(
        || "refused=none refused_at=none borrow_at_refusal=none".to_owned(),
        |(position, refusal, borrow)| {
            format!(
                "refused={} refused_at={position} borrow_at_refusal={borrow}",
                refusal.label()
            )
        },
    );
    let delta = smallest_unbuilt_delta_that_flips_the_cover(
        summary.apex,
        summary.reserve_at_the_apex,
        slots_per_region,
    )
    .map_or_else(|| "none".to_owned(), |delta| delta.to_string());
    format!(
        "publishes={} completed={} {refusal} apex={} apex_at={} reserve_at_apex={} apex_minus_reserve_at_apex={} apex_minus_reserve_minimum={} m1_reserve_covers_apex={} delta_flip={delta} reserve_minimum={} reserve_maximum={} reserve_at_start={} reserve_at_last_publish={} apex_between={} apex_between_at={} positive_valley_publishes={} positive_between_publishes={} borrow_before_last_publish={} intent_new_slots={} largest_batch_new_slots={} largest_new_slots={} w_exceeds_landing_publishes={} largest_w_per_mille_of_landing={}",
        summary.publishes,
        summary.is_completed,
        summary.apex,
        summary.apex_position,
        summary.reserve_at_the_apex,
        summary.apex - summary.reserve_at_the_apex,
        summary.apex - summary.reserve_minimum,
        reserve_covers_the_apex(summary.apex, summary.reserve_at_the_apex),
        summary.reserve_minimum,
        summary.reserve_maximum,
        summary.reserve_at_the_start,
        summary.reserve_at_the_last_publish,
        summary.apex_between,
        summary.apex_between_position,
        summary.publishes_with_a_positive_valley_borrow,
        summary.publishes_with_a_positive_between_borrow,
        summary.borrow_before_the_last_publish,
        summary.intent_new_slots,
        summary.largest_batch_new_slots,
        summary.largest_new_slots,
        summary.publishes_whose_new_slots_exceed_the_landing_demand,
        summary.largest_new_slots_per_mille_of_the_landing_demand,
    )
}
