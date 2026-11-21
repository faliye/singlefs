//! 几何敏感性与判别力自证（8.2）。

use super::*;

// ───────────────────────────── 几何敏感性与判别力自证（8.2） ─────────────────────────────

/// 一个比值量在两个取样点上：各自的判定（门槛 0.5）、翻不翻面，门槛挪到两点之间时判定函数分不分得开。
fn ratio_sensitivity_fields(name: &str, first: (u128, u128), second: (u128, u128)) -> String {
    let first_side = reaches_half(first.0, first.1);
    let second_side = reaches_half(second.0, second.1);
    let equal = first.0 * second.1 == second.0 * first.1;
    let separates = if equal || first.1 == 0 || second.1 == 0 {
        None
    } else {
        let threshold_numerator = first.0 * second.1 + second.0 * first.1;
        let threshold_denominator = 2 * first.1 * second.1;
        Some(
            ratio_reaches(first.0, first.1, threshold_numerator, threshold_denominator)
                != ratio_reaches(
                    second.0,
                    second.1,
                    threshold_numerator,
                    threshold_denominator,
                ),
        )
    };
    format!(
        "{name}_d1={} {name}_quick={} {name}_stable={} {name}_threshold_moved_between_separates={}",
        ratio_text(first.0, first.1),
        ratio_text(second.0, second.1),
        bool_text(first_side == second_side),
        separates.map_or("threshold_cannot_move_in", bool_text),
    )
}

pub(super) fn share_sensitivity_fields(
    name: &str,
    first: UnitShareEstimate,
    second: UnitShareEstimate,
) -> String {
    let threshold = threshold_half();
    let first_verdict = ShareVerdict::of(first, threshold);
    let second_verdict = ShareVerdict::of(second, threshold);
    let separates = |pick: fn(UnitShareEstimate) -> f64| -> Option<bool> {
        let (left, right) = (pick(first), pick(second));
        if (left - right).abs() < f64::EPSILON {
            return None;
        }
        let moved = (left + right) / 2.0;
        Some(ShareVerdict::of(first, moved) != ShareVerdict::of(second, moved))
    };
    format!(
        "{name}_first_point_verdict={} {name}_second_point_verdict={} {name}_stable={} {name}_lower_threshold_moved_between_separates={} {name}_upper_threshold_moved_between_separates={}",
        first_verdict.label(),
        second_verdict.label(),
        bool_text(first_verdict == second_verdict),
        separates(|estimate| estimate.lower).map_or("threshold_cannot_move_in", bool_text),
        separates(|estimate| estimate.upper).map_or("threshold_cannot_move_in", bool_text),
    )
}

fn capacity_sensitivity_fields(
    name: &str,
    first_required: u128,
    second_required: u128,
    capacity: u128,
) -> String {
    let first_verdict = CapacityVerdict::of(first_required, capacity);
    let second_verdict = CapacityVerdict::of(second_required, capacity);
    let separates = if first_required == second_required {
        None
    } else {
        let moved = first_required.midpoint(second_required);
        Some(
            CapacityVerdict::of(first_required, moved)
                != CapacityVerdict::of(second_required, moved),
        )
    };
    format!(
        "{name}_d1={} {name}_quick={} {name}_stable={} {name}_threshold_moved_between_separates={}",
        first_verdict.label(),
        second_verdict.label(),
        bool_text(first_verdict == second_verdict),
        separates.map_or("threshold_cannot_move_in", bool_text),
    )
}

/// 8.2 展开方式那一行（D1 与 D1-quick）：R1、K_pair / N、三条 K4 臂的 K / N、s_lo 与 s_hi、G5 的每一句。
pub(super) fn report_geometry_sensitivity(output: &mut ResultLines, outcome: &StreamOutcome) {
    let pair = |summary: &CellSummary, arm: Arm| {
        (
            u128::from(summary.arm_distinct_keys[arm.slot()]),
            u128::from(summary.states),
        )
    };
    let mut fields = vec![ratio_sensitivity_fields(
        "r1",
        (
            u128::from(outcome.union.distinct_unit_contents),
            u128::from(outcome.union.per_state_unit_checks),
        ),
        (
            u128::from(outcome.quick.distinct_unit_contents),
            u128::from(outcome.quick.per_state_unit_checks),
        ),
    )];
    for arm in [
        Arm::RecoveryPair,
        Arm::WalkUnits,
        Arm::AllUnits,
        Arm::FullReadSet,
    ] {
        fields.push(ratio_sensitivity_fields(
            &format!("{}_keys_over_states", arm.label()),
            pair(&outcome.union, arm),
            pair(&outcome.quick, arm),
        ));
    }
    for arm in ARMS {
        fields.push(format!(
            "{}_inconsistent_d1={} {}_inconsistent_quick={}",
            arm.label(),
            outcome.union.arm_inconsistent[arm.slot()],
            arm.label(),
            outcome.quick.arm_inconsistent[arm.slot()],
        ));
    }
    fields.push(share_sensitivity_fields(
        "unit_share",
        outcome.weighted,
        unweighted_unit_share(&outcome.quick),
    ));
    for arm in [Arm::WalkUnits, Arm::AllUnits, Arm::FullReadSet] {
        let sampled_parts = device_memory_parts(&outcome.union, outcome.write_table_length, arm);
        let quick = device_memory_parts(&outcome.quick, outcome.write_table_length, arm);
        let extrapolate = |parts: &DeviceMemoryParts, states: u64| {
            parts
                .linear(u128::from(states), EXTRAPOLATED_STATES)
                .total()
        };
        fields.push(capacity_sensitivity_fields(
            &format!("g5_{}_q5b_largest_card", arm.label()),
            extrapolate(&sampled_parts, outcome.union.states),
            extrapolate(&quick, outcome.quick.states),
            LARGEST_CARD_MEBIBYTES * MEBIBYTE,
        ));
        fields.push(capacity_sensitivity_fields(
            &format!("g5_{}_q5b_five_cards_total", arm.label()),
            extrapolate(&sampled_parts, outcome.union.states),
            extrapolate(&quick, outcome.quick.states),
            FIVE_CARDS_MEBIBYTES * MEBIBYTE,
        ));
        fields.push(format!(
            "g5_{}_bytes_per_state_d1={} g5_{}_bytes_per_state_quick={}",
            arm.label(),
            ratio_text(sampled_parts.total(), u128::from(outcome.union.states)),
            arm.label(),
            ratio_text(quick.total(), u128::from(outcome.quick.states)),
        ));
    }
    output.line(format!(
        "name=geometry_sensitivity knob=expansion stream={} {}",
        outcome.name.label(),
        fields.join(" ")
    ));
}
