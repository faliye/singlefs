//! 判定行（第六节各行，登记 5.5 第一段的够判规则）。

use super::*;

// ───────────────────────────── 判定行（第六节各行，5.5 第一段的够判规则） ─────────────────────────────

/// 每行各报各的判定，合取留给人（第六节开头）。退出码：S5 在产物那一趟没过就 3。
pub(super) fn report_verdicts(
    output: &mut ResultLines,
    outcomes: &[&StreamOutcome],
    discriminating: &[Vec<(Arm, bool)>],
    timing_control_holds: bool,
    thread_lines_identical: bool,
) -> i32 {
    let mut determinism_clause_holds = true;
    let mut unit_check_fields = Vec::new();
    let mut recovery_reuse_fields = Vec::new();
    let mut time_split_fields = Vec::new();
    let mut checker_reuse_fields = Vec::new();
    let mut device_memory_fields = Vec::new();
    let found = |stream_index: usize, arm: Arm| {
        discriminating[stream_index]
            .iter()
            .any(|(candidate, has_one)| *candidate == arm && *has_one)
    };
    for (stream_index, outcome) in outcomes.iter().enumerate() {
        let name = outcome.name.label();
        let union = &outcome.union;
        for summary in [
            &outcome.small,
            &outcome.stride,
            &outcome.union,
            &outcome.quick,
        ] {
            determinism_clause_holds &= summary.consult_report_differs == 0
                && summary.ignore_report_differs == 0
                && summary.verdicts_differ == 0;
        }
        let pc1 = outcome.pc1.map_or(
            "not_in_this_stream",
            |(distinct_unchanged, checks_grew_exactly, _positions)| {
                bool_text(distinct_unchanged && checks_grew_exactly)
            },
        );
        unit_check_fields.push(format!(
            "{name}_r1_at_or_above_half_d1_estimate={} {name}_enough_to_judge=false {name}_v7_every_state_without_unit_reads={} {name}_content_collisions_zero={} {name}_pc1_holds={pc1}",
            bool_text(reaches_half(u128::from(union.distinct_unit_contents), u128::from(union.per_state_unit_checks))),
            bool_text(union.per_state_unit_checks == 0),
            bool_text(union.content_collisions == 0),
        ));
        for arm in [Arm::RecoveryConsult, Arm::RecoveryIgnore] {
            let slot = arm.slot();
            let flips = union.arm_inconsistent[slot] >= 1;
            recovery_reuse_fields.push(format!(
                "{name}_{arm}_inconsistent_zero={} {name}_{arm}_flips_to_every_state_rerun={} {name}_{arm}_enough_to_judge={} {name}_{arm}_hits_positive={} {name}_{arm}_pc2b_constant_key_inconsistent={} {name}_{arm}_pc2c_found_a_position={}",
                bool_text(!flips),
                bool_text(flips),
                bool_text(flips),
                bool_text(union.arm_hits[slot] > 0),
                bool_text(outcome.small.arm_constant_key_inconsistent[slot] >= 1),
                bool_text(found(stream_index, arm)),
                arm = arm.label(),
            ));
        }
        recovery_reuse_fields.push(format!(
            "{name}_k2_pair_keys_at_or_above_half_d1_estimate={} {name}_k2_pair_enough_to_judge=false",
            bool_text(reaches_half(
                u128::from(union.arm_distinct_keys[Arm::RecoveryPair.slot()]),
                u128::from(union.states)
            )),
        ));
        let share_verdict = ShareVerdict::of(outcome.weighted, threshold_half());
        let replay_share_over_one_percent =
            union.states_replaying_longer_than_the_checker * 100 > union.states;
        let shorter_share_over_one_percent =
            union.states_shorter_than_their_segments * 100 > union.states;
        time_split_fields.push(format!(
            "{name}_verdict={} {name}_enough_to_judge={} {name}_v3_segments_voided={} {name}_v3_replay_voided={}",
            share_verdict.label(),
            bool_text(share_is_enough_to_judge(outcome.weighted) && timing_control_holds),
            bool_text(shorter_share_over_one_percent),
            bool_text(replay_share_over_one_percent),
        ));
        for arm in [Arm::WalkUnits, Arm::AllUnits, Arm::FullReadSet] {
            let slot = arm.slot();
            let flips = union.arm_inconsistent[slot] >= 1;
            let constant_key_separates = union.arm_constant_key_inconsistent[slot] >= 1
                || union.arm_differing_pair[slot].is_some();
            checker_reuse_fields.push(format!(
                "{name}_{arm}_inconsistent_zero={} {name}_{arm}_flips_cannot_reuse={} {name}_{arm}_enough_to_judge={} {name}_{arm}_hits_positive={} {name}_{arm}_pc4b_separates={} {name}_{arm}_pc4c_found_a_position={} {name}_{arm}_keys_at_or_above_half_d1_estimate={}",
                bool_text(!flips),
                bool_text(flips),
                bool_text(flips),
                bool_text(union.arm_hits[slot] > 0),
                bool_text(constant_key_separates),
                bool_text(found(stream_index, arm)),
                bool_text(reaches_half(u128::from(union.arm_distinct_keys[slot]), u128::from(union.states))),
                arm = arm.label(),
            ));
        }
        checker_reuse_fields.push(format!(
            "{name}_s6_walk_reads_inside_normal_reads={}",
            bool_text(union.walk_reads_outside_normal_reads == 0)
        ));
        let per_state_bytes = mask_bytes_per_state(outcome.write_table_length)
            + u128::from(KEYS_STORED_PER_STATE * KEY_BYTES);
        let at_extrapolation = per_state_bytes * EXTRAPOLATED_STATES;
        device_memory_fields.push(format!(
            "{name}_per_state_parts_bytes={per_state_bytes} {name}_one_shot_per_state_parts_exceed_largest_card={} {name}_one_shot_per_state_parts_exceed_five_cards={}",
            bool_text(at_extrapolation > LARGEST_CARD_MEBIBYTES * MEBIBYTE),
            bool_text(at_extrapolation > FIVE_CARDS_MEBIBYTES * MEBIBYTE),
        ));
    }
    output.line(format!(
        "name=verdict scope=segment_one row=g1 {}",
        unit_check_fields.join(" ")
    ));
    output.line(format!(
        "name=verdict scope=segment_one row=g2 {}",
        recovery_reuse_fields.join(" ")
    ));
    output.line(format!(
        "name=verdict scope=segment_one row=g3 {} pc3_holds={} v6_threads_result_lines_identical={}",
        time_split_fields.join(" "),
        bool_text(timing_control_holds),
        bool_text(thread_lines_identical),
    ));
    output.line(format!(
        "name=verdict scope=segment_one row=g4 {}",
        checker_reuse_fields.join(" ")
    ));
    output.line(format!(
        "name=verdict scope=segment_one row=g5 {}",
        device_memory_fields.join(" ")
    ));
    output.line(format!(
        "name=verdict scope=segment_one row=s5 clause_S5_holds={}",
        bool_text(determinism_clause_holds)
    ));
    if determinism_clause_holds {
        0
    } else {
        3
    }
}
