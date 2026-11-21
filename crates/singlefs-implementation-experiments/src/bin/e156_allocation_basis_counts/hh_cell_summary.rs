//! 岔路 1 的族 Hh：一格的汇总行与轨迹。

use super::*;

pub(super) fn set_text(values: &BTreeSet<u64>) -> String {
    let parts: Vec<String> = values.iter().map(ToString::to_string).collect();
    format!("[{}]", parts.join(","))
}

/// 轨迹（登记第八节 8.1）：峰值、环转过新池新建文件那一圈之后的峰值、为正的观测点数、期末值。
fn trajectory_text<Value: Copy + Ord + Default + std::fmt::Display>(
    values: &[(u64, Value)],
    after_the_first_ring_txg: u64,
) -> String {
    let zero = Value::default();
    let peak = values.iter().map(|(_, value)| *value).max().unwrap_or(zero);
    let peak_after_ring = values
        .iter()
        .filter(|(txg, _)| *txg >= after_the_first_ring_txg)
        .map(|(_, value)| *value)
        .max();
    let positive = values.iter().filter(|(_, value)| *value > zero).count();
    let final_value = values.last().map(|(_, value)| *value);
    format!(
        "peak={peak} peak_after_first_ring={} positive_points={positive} final={}",
        peak_after_ring.map_or_else(|| "none".to_string(), |value| value.to_string()),
        final_value.map_or_else(|| "none".to_string(), |value| value.to_string())
    )
}

pub(super) fn emit_cell_summary(emitter: &mut Emitter, outcome: &HhRunOutcome) {
    let shape = outcome.shape;
    let observations = &outcome.observations;
    let anchor_missing_txg_mismatches = observations
        .iter()
        .filter(|observation| {
            observation
                .anchor
                .is_some_and(|row| row.missing_txgs != observation.reading.missing_txgs)
        })
        .count();
    let anchor_live_hole_mismatches = observations
        .iter()
        .filter(|observation| {
            observation
                .anchor
                .is_some_and(|row| row.live_holes != observation.reading.live_holes)
        })
        .count();
    let anchor_floor_mismatches = observations
        .iter()
        .filter(|observation| {
            observation
                .anchor
                .is_some_and(|row| row.floor != observation.reading.ring_threshold)
        })
        .count();
    let anchor_lag_mismatches = observations
        .iter()
        .filter(|observation| {
            observation.anchor.is_some_and(|row| {
                row.lag != observation.reading.newest_root_txg - observation.reading.ring_threshold
            })
        })
        .count();
    let anchor_pinned_mismatches = observations
        .iter()
        .filter(|observation| {
            observation.anchor.is_some_and(|row| {
                row.pinned_slots != observation.reading.all_zero_slots_still_pinned
            })
        })
        .count();
    let anchor_rows_missing = observations
        .iter()
        .filter(|observation| observation.anchor.is_none())
        .count();
    let below_bound = |timing: ReclaimTiming| {
        observations
            .iter()
            .filter(|observation| {
                observation.anchor.is_some_and(|row| {
                    observation.over_withheld(Arm::FloorRule, timing) < row.tree_table_lower_bound
                })
            })
            .count()
    };
    let positive = |pick: &dyn Fn(&HhObservation) -> u64| {
        observations
            .iter()
            .filter(|observation| pick(observation) > 0)
            .count()
    };
    let (truncated, truncation_txg, truncation_error) = match &outcome.truncation {
        Some((txg, error)) => (true, txg.to_string(), error.replace(' ', "_")),
        None => (false, "none".to_string(), "none".to_string()),
    };
    let holes: BTreeSet<u64> = outcome
        .recoveries
        .iter()
        .map(|recovery| recovery.hole_txg)
        .collect();
    let recovery_ends: BTreeSet<u64> = outcome
        .recoveries
        .iter()
        .map(|recovery| recovery.last_txg_of_the_mount)
        .collect();
    emitter.emit(&format!(
        "name=q1_cell family={} {} allocator={:?} cut_point={:?} registered_end={} actual_end={} truncated={truncated} truncation_txg={truncation_txg} truncation_error={truncation_error} observations={} holes_injected={} recovery_ends={} genesis_root_in_ring={} slots_per_region_read_back={} new_pool_file_creation_matches_k1_1={} ring_wrap_overwrites_checked={} ring_wrap_overwrite_mismatches={} tree_table_release_checks={} tree_table_release_violations={} mount_schedule_mismatches={} cut_mismatches_restored_image={} w9_roots_compared={} w9_mismatches={} w9_failures={} a11_h_que_mismatches={anchor_missing_txg_mismatches} a11_h_dong_mismatches={anchor_live_hole_mismatches} a14_floor_mismatches={anchor_floor_mismatches} anchor_rows_missing={anchor_rows_missing} anchor_lag_mismatches={anchor_lag_mismatches} anchor_pinned_slot_mismatches={anchor_pinned_mismatches} a15_real_below_bound={} a15_every_below_bound={} k10_violation_points={} devices_unequal_points={} root_ring_missing_points={} abandoned_root_points={} roots_without_version_facts_points={} admission_refusals={} mount_admission_not_before_acquisition={} mount_floor_raise_sequences={} effective_floor_positive_points={} q1b_real_positive_points={} q1b_every_positive_points={} q1a_plus_real_positive_points={} q1a_plus_every_positive_points={} q1b_plus_positive_points={} a10_lags_real={} a10_lags_every={} i31_red_on_hole_states={} i31_judged_hole_states={}",
        outcome.family,
        shape.fields(),
        outcome.allocator_source,
        outcome.cut_point,
        outcome.registered_end,
        outcome.actual_end,
        observations.len(),
        set_text(&holes),
        set_text(&recovery_ends),
        outcome.genesis_root_in_ring,
        outcome.slots_per_region_read_back,
        outcome.new_pool_file_creation_matches_the_registered_anchor,
        outcome.ring_wrap_overwrites_checked,
        outcome.ring_wrap_overwrite_mismatches,
        outcome.tree_table_release_checks,
        outcome.tree_table_release_violations,
        outcome.mount_schedule_mismatches,
        outcome
            .recoveries
            .iter()
            .filter(|recovery| !recovery.cut_matches_the_restored_image)
            .count(),
        outcome.disk_cross_checked_roots,
        outcome.disk_cross_check_mismatches,
        if outcome.disk_cross_check_failures.is_empty() {
            "none".to_string()
        } else {
            outcome.disk_cross_check_failures.join(",")
        },
        below_bound(ReclaimTiming::Real),
        below_bound(ReclaimTiming::EveryPublish),
        positive(&|observation| observation.reading.device_zero.reclaim_rule_violations),
        observations
            .iter()
            .filter(|observation| !observation.reading.devices_equal)
            .count(),
        observations
            .iter()
            .filter(|observation| !observation.reading.root_ring_installed)
            .count(),
        positive(&|observation| observation.reading.abandoned_roots),
        positive(&|observation| observation.reading.roots_without_version_facts),
        outcome.admission_refusals,
        outcome
            .recoveries
            .iter()
            .filter(|recovery| recovery.mount_admission != "admitted_before_acquisition")
            .count(),
        outcome
            .recoveries
            .iter()
            .map(|recovery| recovery.mount_floor_raise_sequences)
            .sum::<u64>(),
        positive(&|observation| observation.reading.effective_floor),
        positive(&|observation| observation.reading.device_zero.interval_rule_real),
        positive(&|observation| observation.reading.device_zero.interval_rule_every_publish),
        positive(&|observation| observation.reading.device_zero.floor_rule_real_referenced_but_free),
        positive(&|observation| {
            observation
                .reading
                .device_zero
                .floor_rule_every_publish_referenced_but_free
        }),
        positive(&|observation| observation.reading.device_zero.interval_rule_referenced_but_free),
        set_text(&outcome.tree_table_lags_real),
        set_text(&outcome.tree_table_lags_every_publish),
        outcome.checker_red_on_hole_states,
        outcome.checker_judged_hole_states,
    ));
    let after_the_first_ring_txg = E156_ROOT_RING_REGIONS * shape.slots_per_region + 3;
    let series_u64 = |pick: &dyn Fn(&HhObservation) -> u64| -> Vec<(u64, u64)> {
        observations
            .iter()
            .map(|observation| (observation.txg, pick(observation)))
            .collect()
    };
    let series_i64 = |timing: ReclaimTiming| -> Vec<(u64, i64)> {
        observations
            .iter()
            .map(|observation| (observation.txg, observation.delta(timing)))
            .collect()
    };
    let trajectories: [(&str, String); 11] = [
        (
            "q1a_real",
            trajectory_text(
                &series_u64(&|observation| {
                    observation.over_withheld(Arm::FloorRule, ReclaimTiming::Real)
                }),
                after_the_first_ring_txg,
            ),
        ),
        (
            "q1a_every",
            trajectory_text(
                &series_u64(&|observation| {
                    observation.over_withheld(Arm::FloorRule, ReclaimTiming::EveryPublish)
                }),
                after_the_first_ring_txg,
            ),
        ),
        (
            "q1b_real",
            trajectory_text(
                &series_u64(&|observation| {
                    observation.over_withheld(Arm::LifetimeIntervalRule, ReclaimTiming::Real)
                }),
                after_the_first_ring_txg,
            ),
        ),
        (
            "q1b_every",
            trajectory_text(
                &series_u64(&|observation| {
                    observation
                        .over_withheld(Arm::LifetimeIntervalRule, ReclaimTiming::EveryPublish)
                }),
                after_the_first_ring_txg,
            ),
        ),
        (
            "delta_real",
            trajectory_text(&series_i64(ReclaimTiming::Real), after_the_first_ring_txg),
        ),
        (
            "delta_every",
            trajectory_text(
                &series_i64(ReclaimTiming::EveryPublish),
                after_the_first_ring_txg,
            ),
        ),
        (
            "threshold_lag",
            trajectory_text(
                &series_u64(&|observation| {
                    observation.reading.newest_root_txg - observation.reading.ring_threshold
                }),
                after_the_first_ring_txg,
            ),
        ),
        (
            "h_que",
            trajectory_text(
                &series_u64(&|observation| observation.reading.missing_txgs),
                after_the_first_ring_txg,
            ),
        ),
        (
            "h_dong",
            trajectory_text(
                &series_u64(&|observation| observation.reading.live_holes),
                after_the_first_ring_txg,
            ),
        ),
        (
            "self_reclaimed_real",
            trajectory_text(
                &observations
                    .iter()
                    .filter_map(|observation| {
                        observation
                            .self_reclaimed_real
                            .map(|count| (observation.txg, count))
                    })
                    .collect::<Vec<_>>(),
                after_the_first_ring_txg,
            ),
        ),
        (
            "pinned_slots",
            trajectory_text(
                &series_u64(&|observation| observation.reading.all_zero_slots_still_pinned),
                after_the_first_ring_txg,
            ),
        ),
    ];
    for (quantity, text) in trajectories {
        emitter.emit(&format!(
            "name=q1_trajectory family={} {} quantity={quantity} {text}",
            outcome.family,
            shape.fields()
        ));
    }
    let ring_length = E156_ROOT_RING_REGIONS * shape.slots_per_region;
    let lag_at_least_ring = observations
        .iter()
        .filter(|observation| {
            observation.reading.newest_root_txg - observation.reading.ring_threshold >= ring_length
        })
        .count();
    let self_reclaimed_zero = observations
        .iter()
        .filter(|observation| observation.self_reclaimed_real == Some(0))
        .count();
    emitter.emit(&format!(
        "name=q1_trajectory_extra family={} {} threshold_lag_at_least_3s_points={lag_at_least_ring} self_reclaimed_real_zero_points={self_reclaimed_zero}",
        outcome.family,
        shape.fields()
    ));
}
