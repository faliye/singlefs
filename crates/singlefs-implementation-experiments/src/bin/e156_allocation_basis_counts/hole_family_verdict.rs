//! 第 7 次重跑：同槽对照的读数与洞族的判定行（问题单 H1）。

use super::*;

/// PC-同槽的汇总输入：每个 (S, ρ) 一格 Hs(2, 后) 与它的对照格，「每」列 Q1a 峰值各一个。
#[derive(Clone, Copy, Debug)]
pub(super) struct SameSlotControlReading {
    pub(super) unchanged_peak_every_publish: i64,
    pub(super) control_peak_every_publish: i64,
}

pub(super) fn emit_hole_family_verdict(
    emitter: &mut Emitter,
    units: &[TierUnit],
    tier_sequence_cases_wrong: usize,
    same_slot_controls: &[SameSlotControlReading],
) {
    for quantity in E156_TIER_QUANTITIES {
        let of_quantity: Vec<&TierUnit> = units
            .iter()
            .filter(|unit| unit.quantity == quantity)
            .collect();
        let judged: Vec<&TierUnit> = of_quantity
            .iter()
            .copied()
            .filter(|unit| unit.growth != TierGrowth::FewerThanThreeTiers)
            .collect();
        let too_few: Vec<String> = of_quantity
            .iter()
            .filter(|unit| unit.growth == TierGrowth::FewerThanThreeTiers)
            .map(|unit| unit.sampling_point_name())
            .collect();
        let count_of = |kind: &str| {
            judged
                .iter()
                .filter(|unit| unit.growth.kind() == kind)
                .count()
        };
        let kinds: BTreeSet<&str> = judged.iter().map(|unit| unit.growth.kind()).collect();
        let geometry = if judged.is_empty() {
            "no_unit_judged".to_string()
        } else if kinds.len() == 1 {
            format!(
                "consistent_on_{}_sampling_points_only_on_these",
                judged.len()
            )
        } else {
            "unstable".to_string()
        };
        let per_unit: Vec<String> = of_quantity
            .iter()
            .map(|unit| format!("{}:{}", unit.sampling_point_name(), unit.growth.label()))
            .collect();
        emitter.emit(&format!(
            "name=r7_verdict quantity={} units={} judged={} fewer_than_three_tiers_units={} still_rising={} flat={} not_monotonic={} geometry_sensitivity={geometry} per_unit={}",
            quantity.label(),
            of_quantity.len(),
            judged.len(),
            bracketed(&too_few),
            count_of("still_rising"),
            count_of("flat"),
            count_of("not_monotonic"),
            per_unit.join(","),
        ));
    }
    let names_where = |quantities: &[TierQuantity], predicate: &dyn Fn(TierGrowth) -> bool| {
        units
            .iter()
            .filter(|unit| quantities.contains(&unit.quantity) && predicate(unit.growth))
            .map(|unit| format!("{}:{}", unit.quantity.label(), unit.sampling_point_name()))
            .collect::<Vec<String>>()
    };
    let criterion_units_not_monotonic = names_where(
        &[TierQuantity::InsideTheFirstRing, TierQuantity::AcrossRings],
        &|growth| growth == TierGrowth::NotMonotonic,
    );
    let mut report_only_units_unlike_the_prediction =
        names_where(&[TierQuantity::BackPosition], &|growth| {
            matches!(growth, TierGrowth::StillRising | TierGrowth::NotMonotonic)
        });
    report_only_units_unlike_the_prediction.extend(names_where(
        &[TierQuantity::SameSlotFront, TierQuantity::SameSlotBack],
        &|growth| {
            matches!(
                growth,
                TierGrowth::FlatAtTheLastTiers { .. } | TierGrowth::NotMonotonic
            )
        },
    ));
    let same_slot_back_units_flat_against_the_clause_range =
        names_where(&[TierQuantity::SameSlotBack], &|growth| {
            matches!(growth, TierGrowth::FlatAtTheLastTiers { .. })
        });
    let mut sampling_points_where_the_two_ring_readings_disagree: Vec<String> = Vec::new();
    for inside in units
        .iter()
        .filter(|unit| unit.quantity == TierQuantity::InsideTheFirstRing)
    {
        for across in units.iter().filter(|unit| {
            unit.quantity == TierQuantity::AcrossRings
                && unit.sampling_point_name() == inside.sampling_point_name()
        }) {
            let kinds = (inside.growth.kind(), across.growth.kind());
            if kinds == ("flat", "still_rising") || kinds == ("still_rising", "flat") {
                sampling_points_where_the_two_ring_readings_disagree.push(format!(
                    "{}:r7a_{}_r7c_{}",
                    inside.sampling_point_name(),
                    kinds.0,
                    kinds.1
                ));
            }
        }
    }
    let ring_units: Vec<&TierUnit> = units
        .iter()
        .filter(|unit| {
            matches!(
                unit.quantity,
                TierQuantity::InsideTheFirstRing | TierQuantity::AcrossRings
            )
        })
        .collect();
    let dropping_changed = ring_units
        .iter()
        .filter(|unit| unit.growth.kind() != unit.growth_without_the_last_tier.kind())
        .count();
    let threshold_tested = units
        .iter()
        .filter(|unit| unit.threshold_moved_flips.is_some())
        .count();
    let threshold_flipping = units
        .iter()
        .filter(|unit| unit.threshold_moved_flips == Some(true))
        .count();
    let same_slot_unseen = same_slot_controls
        .iter()
        .filter(|reading| {
            reading.control_peak_every_publish >= reading.unchanged_peak_every_publish
        })
        .count();
    emitter.emit(&format!(
        "name=r7_verdict quantity=summary units={} criterion_units_not_monotonic={} report_only_units_unlike_the_prediction={} sampling_points_where_the_two_ring_readings_disagree={} same_slot_back_units_flat_against_the_clause_range={} dropping_the_last_tier_changed_units={dropping_changed}_of_{} threshold_moved_units_flipping={threshold_flipping}_of_{threshold_tested} tier_sequence_control_cases_wrong={tier_sequence_cases_wrong} same_slot_controls_run={} same_slot_controls_without_a_drop={same_slot_unseen}",
        units.len(),
        bracketed(&criterion_units_not_monotonic),
        bracketed(&report_only_units_unlike_the_prediction),
        bracketed(&sampling_points_where_the_two_ring_readings_disagree),
        bracketed(&same_slot_back_units_flat_against_the_clause_range),
        ring_units.len(),
        same_slot_controls.len(),
    ));
}
