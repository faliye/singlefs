//! 第二次重跑的整趟与它的判定行。

use super::*;

/// 第 2 次重跑的全部行与判决行（登记第五至八节）：窗口一族与 RGall 一族在条款形候选下界下跑第 1 次的每一格，再加 `disk_stuck`；
/// 接在第 1 次重跑的全部行之后、`name=done` 之前。`first_rows` 与 `first_crit1_bodies` 是第 1 次那一族的行簿，逐行比用。
pub(super) fn run_second_rerun_arms(emitter: &mut Emitter, first_rows: &[WindowRow], first_crit1_bodies: &[String]) {
    let family = second_rerun_family();
    let family = &family;
    let arm_names = |arms: &[Arm]| arms.iter().map(|arm| arm.name()).collect::<Vec<&str>>().join("/");
    println!("{}", emitter.emit_raw(&format!(
        "name=f_config f_arms={} f_short_arms={} short_slots={} disk_stuck_arms=RGall_ring_all/RGall_Q_ring_all/G2SW_tightened_states_window/G2SW_ringgate stuck_disks=1/0 \
         stuck_fills=0.30/0.70 mark_revolutions=1/2/4 stuck_window_revolution_limit={STUCK_WINDOW_REVOLUTION_LIMIT} stuck_tail_revolutions={STUCK_TAIL_REVOLUTIONS} \
         stuck_endings=crash/lost_disk_0/lost_disk_1 bounded_slope_rule=twice_the_pinned_increase_below_the_blocks_freed geometry_points=h1/h2/h3 seeds={SEED_COUNT} fault_seeds={FAULT_SEED_COUNT}",
        arm_names(&family.arms), arm_names(&family.short_arms), format_set(&WINDOW_SHORT_SLOTS_SWEEP)
    )));
    let mut rows: Vec<WindowRow> = Vec::new();
    let mut growth: Vec<StuckGrowthSummary> = Vec::new();
    let (crit1_rows_not_holding, crit1_line_bodies) = emit_window_crit1_rows(emitter, family);
    emit_window_world_rows(emitter, &mut rows, family);
    emit_window_activation_rows(emitter, &mut rows, family, GeometryPoint::Original);
    emit_window_near_full_rows(emitter, &mut rows, family);
    emit_window_drain_failure_rows(emitter, &mut rows, family);
    emit_window_broken_slot_rows(emitter, &mut rows, &mut growth, family, GeometryPoint::Original);
    emit_window_cost_growth_rows(emitter, &mut rows, family);
    emit_window_activation_rows(emitter, &mut rows, family, GeometryPoint::Reversed);
    emit_window_broken_slot_rows(emitter, &mut rows, &mut growth, family, GeometryPoint::Reversed);
    emit_disk_stuck_rows(emitter, &mut rows, &mut growth, family);
    emit_window_row_verdicts(emitter, &rows, family, crit1_rows_not_holding);
    emit_window_gate_reach_verdicts(emitter, &rows, family);
    emit_window_self_proof_verdicts(emitter, &rows, family);
    emit_window_reading_verdicts(emitter, &rows, family);
    emit_window_geometry_verdicts(emitter, &rows, family);
    emit_second_rerun_verdict_lines(emitter, family, first_rows, first_crit1_bodies, &rows, &growth, &crit1_line_bodies);
}

/// 第 2 次重跑的判决行（登记第六节第 4–17 行与 V5–V8、F1–F11 的计数）：只读行簿，不再跑模型；单测拿一小撮行簿验它不 panic。
fn emit_second_rerun_verdict_lines(
    emitter: &mut Emitter,
    family: &RowFamily,
    first_rows: &[WindowRow],
    first_crit1_bodies: &[String],
    rows: &[WindowRow],
    growth: &[StuckGrowthSummary],
    crit1_line_bodies: &[String],
) {
    let positive_gap_samples = emit_reach_verdicts(emitter, rows);
    let (comparison, _) = emit_row_comparison(emitter, first_rows, rows);
    let first_arm_tokens: HashSet<&str> = first_crit1_bodies.iter().filter_map(|body| body.split(' ').next()).collect();
    let first_bodies: HashSet<&str> = first_crit1_bodies.iter().map(String::as_str).collect();
    let crit1_compared = crit1_line_bodies.iter().filter(|body| body.split(' ').next().is_some_and(|token| first_arm_tokens.contains(token))).count();
    let crit1_differing = crit1_line_bodies.iter()
        .filter(|body| body.split(' ').next().is_some_and(|token| first_arm_tokens.contains(token)) && !first_bodies.contains(body.as_str())).count();
    let rgall_crit1_not_holding = crit1_line_bodies.iter().filter(|body| body.starts_with("arm=RGall") && body.ends_with("holds=0")).count();
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_f_crit1_compare rows_compared={crit1_compared} rows_with_a_differing_line={crit1_differing} rgall_crit1_rows_not_holding={rgall_crit1_not_holding}"
    )));
    let (old_set_larger, new_set_larger, below_root_floor) = emit_direction_verdicts(emitter, rows, family);
    emit_nonzero_rows(emitter, rows, family);
    let readings_differing = emit_rgall_reading_verdicts(emitter, rows);
    emit_cost_verdicts(emitter, rows, family);
    let (window_unbounded_in_disk_stuck, whole_ring_unbounded_in_broken_slot) = emit_growth_verdicts(emitter, growth);
    let stuck_disk_reversal_differing = emit_growth_pairing(emitter, "h3", growth,
        |summary| summary.line_name == family.lines.stuck_growth && summary.row_key.contains("stuck_disk=1"),
        |summary| summary.line_name == family.lines.stuck_growth && summary.row_key.contains("stuck_disk=0"),
        |summary| row_key_without_the_field(&summary.row_key, "stuck_disk="));
    let fill_differing = emit_growth_pairing(emitter, "fill", growth,
        |summary| summary.line_name == family.lines.stuck_growth && summary.row_key.contains("fill=0.30"),
        |summary| summary.line_name == family.lines.stuck_growth && summary.row_key.contains("fill=0.70"),
        |summary| row_key_without_the_field(&summary.row_key, "fill="));
    let broken_slot_reversal_differing = emit_growth_pairing(emitter, "h1", growth,
        |summary| summary.line_name == family.lines.broken_slot_growth,
        |summary| summary.line_name == family.lines.broken_slot_reversed_growth,
        |summary| summary.row_key.clone());
    let stuck_fill_judgement_differing = emit_stuck_fill_verdicts(emitter, rows);
    let stale_rows_outside_the_stuck_worlds = emit_stale_verdicts(emitter, rows, family);
    let window_arm_rows_with_a_nonzero_extra_pinned = count_window_arm_rows_with_a_nonzero_extra_pinned(rows);
    let is_rgall_short = |arm: Arm| matches!(arm, Arm::WholeRing(WholeRingArm::SkipsRootlessDisksShortened | WholeRingArm::CountsRootlessDisksAsZeroShortened));
    let rgall_short_rows_with_zero_fakes = rows.iter().filter(|row| is_rgall_short(row.arm) && !row.judgement.has_fake_candidates).count();
    let rgall_rows: Vec<&WindowRow> = rows.iter().filter(|row| WHOLE_RING_ARMS.iter().any(|whole_ring_arm| row.arm == Arm::WholeRing(*whole_ring_arm))).collect();
    let rgall_false_enospc_outside_the_judged_range = rgall_rows.iter().filter(|row| {
        row.metrics.false_enospc_arm > 0
            && (row.world_class == DISK_STUCK_CLASS || row.world_class == DISK_STUCK_REVERSED_CLASS || (row.world_class == DRAIN_FAILURE_CLASS && row.row_key.contains("failures=3")))
    }).count();
    let rgall_rows_not_zero = rgall_rows.iter().filter(|row| {
        (!row.is_report_only_world && (row.judgement.has_fake_candidates || row.judgement.has_unrecoverable)) || row.judgement.has_false_enospc_in_a_judged_row
    }).count();
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_f_clauses f1_cf_never_reached={} f3_rgall_short_rows_where_the_detector_saw_zero_fake_candidates={rgall_short_rows_with_zero_fakes} \
         f4_window_arm_unbounded_disk_stuck_rows={window_unbounded_in_disk_stuck} f5_rgall_rows_with_different_readings={readings_differing} \
         f7_growth_rows_with_a_different_bound_h3={stuck_disk_reversal_differing} f7_growth_rows_with_a_different_bound_fill={fill_differing} f7_growth_rows_with_a_different_bound_h1={broken_slot_reversal_differing} \
         f7_stuck_rows_with_a_different_judgement_between_fills={stuck_fill_judgement_differing} \
         f8_rgall_false_enospc_rows_outside_the_judged_range={rgall_false_enospc_outside_the_judged_range} f9_rgall_rows_with_fakes_unrecoverable_or_judged_false_enospc={rgall_rows_not_zero} \
         f10_rows_outside_the_stuck_worlds_with_a_root_beyond_two_revolutions={stale_rows_outside_the_stuck_worlds} f11_rgall_broken_slot_unbounded_rows={whole_ring_unbounded_in_broken_slot} \
         v5_changed_rows_with_a_never_positive_floor_gap={} v6_checks_new_set_strictly_larger={new_set_larger} v7_system_configuration_below_root_floor_events={below_root_floor} \
         v8_window_arm_rows_with_a_nonzero_extra_pinned={window_arm_rows_with_a_nonzero_extra_pinned} checks_old_set_strictly_larger={old_set_larger} \
         changed_rows_total={} positive_gap_samples_total={positive_gap_samples}",
        u8::from(positive_gap_samples == 0), comparison.floor_gap_never_positive, comparison.rows_with_a_count_changed
    )));
}

#[cfg(test)]
#[path = "second_rerun_arms_tests.rs"]
mod tests;
