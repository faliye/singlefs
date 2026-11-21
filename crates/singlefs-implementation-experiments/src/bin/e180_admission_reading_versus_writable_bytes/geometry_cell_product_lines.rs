//! 一个几何格的产物行：头、锚点、各臂、残差、轨迹、族与格的汇总、判定、敏感性与独立算术。

use super::*;

/// 一个几何格上跑这几族（并行，每族自己一池），交回按 `families` 的次序排好的结果。
/// 每族的探测线程数按这个几何格的全部族数分（[`probe_threads_of_the_geometry`]），只跑其中一族（一片）时也一样。
pub(super) fn run_geometry_cell(
    geometry: Geometry,
    families: &[&'static str],
    total_threads: usize,
) -> Vec<FamilyOutcome> {
    let probe_threads = probe_threads_of_the_geometry(geometry, total_threads);
    std::thread::scope(|scope| {
        let handles: Vec<_> = families
            .iter()
            .copied()
            .map(|family| {
                std::thread::Builder::new()
                    .stack_size(THREAD_STACK_BYTES)
                    .spawn_scoped(scope, move || run_family(geometry, family, probe_threads))
                    .expect("起族线程")
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| stop_with("family_thread_panicked_outside_the_guard", ""))
            })
            .collect()
    })
}

// ---------------------------------------------------------------------------------------------------------------------
// 汇总行
// ---------------------------------------------------------------------------------------------------------------------

/// 登记第七节 B1：每块盘单元区槽数的独立算术（默认 211968、窄 146465），与分配器报的一致。
pub(super) fn append_geometry_header(
    output: &mut Vec<String>,
    geometry: Geometry,
    unit_area_slots_per_device: &[u64],
) {
    let mut holds = true;
    for (index, reported) in unit_area_slots_per_device.iter().enumerate() {
        if *reported != geometry.unit_area_slots_of_device(index) {
            holds = false;
        }
    }
    let expected_literal = match geometry.name {
        "default_geometry"
        | "default_geometry_with_four_slots_per_region"
        | "default_geometry_with_sixteen_slots_per_region" => Some((211_968u64, 211_968u64)),
        "narrow_devices_geometry" => Some((146_465, 146_465)),
        "unequal_devices_geometry" => Some((211_968, 146_465)),
        _ => None,
    };
    let literal_holds = expected_literal
        .is_none_or(|(first, second)| unit_area_slots_per_device == [first, second]);
    output.push(
        ProductLine::new("geometry")
            .field("geometry", geometry.name)
            .field("device_bytes_0", geometry.device_bytes[0])
            .field("device_bytes_1", geometry.device_bytes[1])
            .field("slots_per_region", geometry.slots_per_region)
            .field("unit_area_slots_0", unit_area_slots_per_device[0])
            .field("unit_area_slots_1", unit_area_slots_per_device[1])
            .field("unit_area_slots_match_the_independent_arithmetic", holds)
            .field(
                "unit_area_slots_match_the_registered_numbers",
                literal_holds,
            )
            .finish(),
    );
}

pub(super) fn append_anchors(
    output: &mut Vec<String>,
    geometry: Geometry,
    anchors: &FirstFileAnchors,
) {
    let open_segment_closed_free_slots_anchor_holds = anchors.open_segment_start_slot
        == Some(50_240)
        && anchors.closed_free_slots_in_the_open_segment_on_device_zero == Some(52);
    let first_data_unit_slot_anchor_holds = anchors.first_data_unit_slot == Some(50_180);
    output.push(
        ProductLine::new("anchors")
            .field("geometry", geometry.name)
            .field(
                "open_segment_start_slot",
                format!("{:?}", anchors.open_segment_start_slot),
            )
            .field(
                "closed_free_slots_in_the_open_segment_on_device_zero",
                format!(
                    "{:?}",
                    anchors.closed_free_slots_in_the_open_segment_on_device_zero
                ),
            )
            .field(
                "open_segment_closed_free_slots_anchor_expected",
                "segment_50240_with_52_closed_free_slots",
            )
            .field(
                "open_segment_closed_free_slots_anchor_holds",
                open_segment_closed_free_slots_anchor_holds,
            )
            .field(
                "first_data_unit_slot",
                format!("{:?}", anchors.first_data_unit_slot),
            )
            .field("first_data_unit_slot_anchor_expected", 50_180)
            .field(
                "first_data_unit_slot_anchor_holds",
                first_data_unit_slot_anchor_holds,
            )
            .finish(),
    );
}

fn append_arm_tally(
    output: &mut Vec<String>,
    scope: &str,
    geometry: &str,
    family: &str,
    arm: Arm,
    tally: &ArmTally,
) {
    output.push(
        ProductLine::new("arm_tally")
            .field("scope", scope)
            .field("geometry", geometry)
            .field("family", family)
            .field("arm", arm.label())
            .field("over_states", tally.over_states)
            .field("not_over_states", tally.not_over_states)
            .field(
                "not_over_below_lower_bound_states",
                tally.not_over_below_lower_bound_states,
            )
            .field("undetermined_states", tally.undetermined_states)
            .field("max_over_bytes", tally.maximum_over_bytes)
            .field(
                "max_over_at",
                if tally.maximum_over_at.is_empty() {
                    "none"
                } else {
                    tally.maximum_over_at.as_str()
                },
            )
            .field("under_states", tally.under_states)
            .field("max_under_bytes", tally.maximum_under_bytes)
            .field(
                "max_under_at",
                if tally.maximum_under_at.is_empty() {
                    "none"
                } else {
                    tally.maximum_under_at.as_str()
                },
            )
            .field(
                "mean_under_bytes_over_the_states_that_under_report",
                tally
                    .sum_under_bytes
                    .checked_div(u128::from(tally.under_states))
                    .unwrap_or(0),
            )
            .finish(),
    );
}

fn append_residual_tally(
    output: &mut Vec<String>,
    scope: &str,
    geometry: &str,
    family: &str,
    device_index: usize,
    tally: &ResidualTally,
) {
    output.push(
        ProductLine::new("residual_tally")
            .field("scope", scope)
            .field("geometry", geometry)
            .field("family", family)
            .field("device", device_index)
            .field("states", tally.states)
            .field("rewritten_check_residual_nonzero_states", tally.rewritten_check_residual_nonzero_states)
            .field("maximum_absolute_rewritten_check_residual", tally.maximum_absolute_rewritten_check_residual)
            .field(
                "first_state_with_a_nonzero_rewritten_check_residual",
                if tally.first_state_with_a_nonzero_rewritten_check_residual.is_empty() {
                    "none"
                } else {
                    tally.first_state_with_a_nonzero_rewritten_check_residual.as_str()
                },
            )
            .field("original_formula_residual_nonzero_states", tally.original_formula_residual_nonzero_states)
            .field("maximum_absolute_original_formula_residual", tally.maximum_absolute_original_formula_residual)
            .field(
                "first_state_with_a_nonzero_original_formula_residual",
                if tally.first_state_with_a_nonzero_original_formula_residual.is_empty() {
                    "none"
                } else {
                    tally.first_state_with_a_nonzero_original_formula_residual.as_str()
                },
            )
            .field(
                "states_with_overlap_allocated_and_isolated",
                tally.states_with_overlap,
            )
            .field("states_with_held_ledger", tally.states_with_held_ledger)
            .field(
                "control_rewritten_check_residual_without_the_overlap_term_stayed_zero_in_states_with_overlap",
                tally.control_without_overlap_term_stayed_zero,
            )
            .finish(),
    );
}

fn append_trajectory(
    output: &mut Vec<String>,
    geometry: &str,
    family: &str,
    quantity: &str,
    device_index: usize,
    tally: &TrajectoryTally,
) {
    output.push(
        ProductLine::new("trajectory")
            .field("geometry", geometry)
            .field("family", family)
            .field("quantity", quantity)
            .field("device", device_index)
            .field("states", tally.states)
            .field("first_value_slots", tally.first_value)
            .field(
                "peak_after_the_first_slots",
                tally
                    .peak_after_the_first
                    .map_or("none".to_string(), |peak| peak.to_string()),
            )
            .field(
                "positive_states_after_the_first",
                tally.positive_states_after_the_first,
            )
            .field("last_value_slots", tally.last_value)
            .finish(),
    );
}

pub(super) fn append_family_summary(
    output: &mut Vec<String>,
    geometry: &str,
    outcome: &FamilyOutcome,
) {
    let tally = &outcome.tally;
    output.push(
        ProductLine::new("family_summary")
            .field("geometry", geometry)
            .field("family", outcome.family)
            .field("seconds", outcome.seconds)
            .field("states", tally.states)
            .field("exact_states", tally.exact_states)
            .field("censored_states", tally.censored_states)
            .field("unreadable_states", tally.unreadable_states)
            .field("probes_run", tally.probes_run)
            .field("non_monotone_states", tally.non_monotone_states)
            .field("control_failures", tally.control_failures.len())
            .field("shortcut_pairs_checked", tally.shortcut_pairs_checked)
            .field("shortcut_differences", tally.shortcut_differences.len())
            .field(
                "records_disagree_with_the_disk_states",
                tally.records_disagree_with_the_disk_states,
            )
            .field(
                "decomposition_failure_states",
                tally.decomposition_failure_states,
            )
            .field(
                "not_whole_slot_available_states",
                tally.not_whole_slot_available_states,
            )
            .field("lag_unattributed_states", tally.lag_unattributed_states)
            .field(
                "held_ledger_mismatch_device_states",
                tally.stop_clause_held_ledger_mismatch_states,
            )
            .field("unmount_pushes_checked", tally.unmount_pushes_checked)
            .field(
                "unmount_pushes_differing",
                tally.unmount_pushes_differing.len(),
            )
            .field("panics", tally.panics.len())
            .finish(),
    );
    for detail in tally
        .control_failures
        .iter()
        .chain(&tally.shortcut_differences)
        .chain(&tally.unmount_pushes_differing)
    {
        output.push(
            ProductLine::new("problem_detail")
                .field("geometry", geometry)
                .field("family", outcome.family)
                .field("detail", detail)
                .finish(),
        );
    }
    for (arm, arm_tally) in &tally.arms {
        append_arm_tally(output, "family", geometry, outcome.family, *arm, arm_tally);
    }
    for (device_index, residual) in tally.residuals.iter().enumerate() {
        append_residual_tally(
            output,
            "family",
            geometry,
            outcome.family,
            device_index,
            residual,
        );
    }
    for (name, trajectories) in [
        (outcome.trajectory_names[0], &tally.closed_minus_ring_window),
        (outcome.trajectory_names[1], &tally.closed_alone),
        (
            outcome.trajectory_names[2],
            &tally.closed_minus_all_common_commitments,
        ),
    ] {
        for (device_index, trajectory) in trajectories.iter().enumerate() {
            append_trajectory(
                output,
                geometry,
                outcome.family,
                name,
                device_index,
                trajectory,
            );
        }
    }
}

/// 同一个几何格上几族的累计并起来（只并探测过的族；长挂载族不带探测，它的轨迹按族报）。
fn merged_counts_of_the_probed_families(outcomes: &[FamilyOutcome]) -> FamilyTally {
    let mut merged = FamilyTally::default();
    for outcome in outcomes
        .iter()
        .filter(|outcome| outcome.family != LONG_MOUNT_FAMILY)
    {
        merged.merge_counts_of(&outcome.tally);
    }
    merged
}

fn per_mille(part: u64, whole: u64) -> u64 {
    (part * 1000).checked_div(whole).unwrap_or(0)
}

pub(super) fn append_cell_summary(
    output: &mut Vec<String>,
    geometry: &str,
    outcomes: &[FamilyOutcome],
) -> FamilyTally {
    let merged = merged_counts_of_the_probed_families(outcomes);
    for (arm, arm_tally) in &merged.arms {
        append_arm_tally(
            output,
            "geometry",
            geometry,
            "all_probed_families",
            *arm,
            arm_tally,
        );
    }
    for (device_index, residual) in merged.residuals.iter().enumerate() {
        append_residual_tally(
            output,
            "geometry",
            geometry,
            "all_probed_families",
            device_index,
            residual,
        );
    }
    let not_exact = merged.censored_states + merged.unreadable_states;
    output.push(
        ProductLine::new("writable_units_coverage")
            .field("geometry", geometry)
            .field("probed_states", merged.states)
            .field("exact_states", merged.exact_states)
            .field(
                "censored_by_the_journal_record_cap_states",
                merged.censored_states,
            )
            .field("unreadable_states", merged.unreadable_states)
            .field(
                "literal_per_mille_of_states_without_an_exact_writable_amount",
                per_mille(not_exact, merged.states),
            )
            .field(
                "literal_states_without_an_exact_writable_amount_exceed_ten_percent",
                not_exact * 10 > merged.states,
            )
            .field(
                "unreadable_only_per_mille_of_states",
                per_mille(merged.unreadable_states, merged.states),
            )
            .field(
                "unreadable_only_states_exceed_ten_percent",
                merged.unreadable_states * 10 > merged.states,
            )
            .finish(),
    );
    // 登记第六节 Q5、Q7 按族分开报：被抛弃根与净释放族与其余几族（登记第六节 Q5：扣住位族是第二段，这一段没有）。
    for scope in [
        "abandoned_root_and_net_release_only",
        "all_families_but_abandoned_root_and_net_release",
    ] {
        let mut residual_of_the_scope = vec![ResidualTally::default(); merged.residuals.len()];
        for outcome in outcomes {
            let is_abandoned_root_and_net_release_family =
                outcome.family == "abandoned_root_and_net_release";
            if (scope == "abandoned_root_and_net_release_only")
                == is_abandoned_root_and_net_release_family
            {
                for (mine, theirs) in residual_of_the_scope
                    .iter_mut()
                    .zip(&outcome.tally.residuals)
                {
                    mine.merge(theirs);
                }
            }
        }
        for (device_index, residual) in residual_of_the_scope.iter().enumerate() {
            append_residual_tally(output, scope, geometry, "see_scope", device_index, residual);
        }
    }
    merged
}

pub(super) fn append_verdict(
    output: &mut Vec<String>,
    geometry: &str,
    outcomes: &[FamilyOutcome],
    merged: &FamilyTally,
    local_constants_checked: usize,
) {
    let anchors = &outcomes[0].anchors;
    let open_segment_closed_free_slots_anchor_holds = anchors.open_segment_start_slot
        == Some(50_240)
        && anchors.closed_free_slots_in_the_open_segment_on_device_zero == Some(52);
    let first_data_unit_slot_anchor_holds = anchors.first_data_unit_slot == Some(50_180);
    output.push(
        ProductLine::new("verdict")
            .field("geometry", geometry)
            .field("local_constants_checked", local_constants_checked)
            .field("control_failures", merged.control_failures.len())
            .field("shortcut_differences", merged.shortcut_differences.len())
            .field(
                "open_segment_closed_free_slots_anchor_holds",
                open_segment_closed_free_slots_anchor_holds,
            )
            .field(
                "first_data_unit_slot_anchor_holds",
                first_data_unit_slot_anchor_holds,
            )
            .field(
                "unmount_pushes_anchor_differences",
                merged.unmount_pushes_differing.len(),
            )
            .field("panics", merged.panics.len())
            .field(
                "records_disagree_with_the_disk_states",
                merged.records_disagree_with_the_disk_states,
            )
            .field(
                "decomposition_failure_states",
                merged.decomposition_failure_states,
            )
            .field("lag_unattributed_states", merged.lag_unattributed_states)
            .field(
                "held_ledger_mismatch_device_states",
                merged.stop_clause_held_ledger_mismatch_states,
            )
            .field("non_monotone_states", merged.non_monotone_states)
            .field("unreadable_states", merged.unreadable_states)
            .field(
                "censored_by_the_journal_record_cap_states",
                merged.censored_states,
            )
            .finish(),
    );
}

// ---------------------------------------------------------------------------------------------------------------------
// 几何敏感性（登记第八节）：每对方向相反的取样点上各判一次，翻面记不稳定；判别力自证：门槛挪到两点读数之间，判定必须由一边转到另一边
// ---------------------------------------------------------------------------------------------------------------------

/// 门槛放在两个读数中间，「读数高于门槛」的判定在两点上必须不同；两点读数相等时没有可自证的，交回 `None`。
fn threshold_between_flips_the_judgement(
    reading_on_one_side: i128,
    reading_on_the_other_side: i128,
) -> Option<bool> {
    if reading_on_one_side == reading_on_the_other_side {
        return None;
    }
    let twice_the_threshold = reading_on_one_side + reading_on_the_other_side;
    Some(
        (2 * reading_on_one_side > twice_the_threshold)
            != (2 * reading_on_the_other_side > twice_the_threshold),
    )
}

fn append_sensitivity_pair(
    output: &mut Vec<String>,
    quantity: &str,
    point_on_one_side: &str,
    reading_on_one_side: i128,
    point_on_the_other_side: &str,
    reading_on_the_other_side: i128,
    flips_on_the_registered_threshold: bool,
) {
    let self_check =
        match threshold_between_flips_the_judgement(reading_on_one_side, reading_on_the_other_side)
        {
            Some(true) => "flips",
            Some(false) => "does_not_flip_stop",
            None => "equal_readings_not_applicable",
        };
    output.push(
        ProductLine::new("geometry_sensitivity")
            .field("quantity", quantity)
            .field("point_a", point_on_one_side)
            .field("reading_a", reading_on_one_side)
            .field("point_b", point_on_the_other_side)
            .field("reading_b", reading_on_the_other_side)
            .field(
                "registered_threshold_judgement_flips",
                flips_on_the_registered_threshold,
            )
            .field(
                "verdict_is_unstable_across_the_pair",
                flips_on_the_registered_threshold,
            )
            .field("discrimination_self_check", self_check)
            .finish(),
    );
}

pub(super) fn append_geometry_sensitivity(
    output: &mut Vec<String>,
    merged_by_geometry: &BTreeMap<&'static str, FamilyTally>,
    trajectories: &BTreeMap<(&'static str, &'static str), TrajectoryTally>,
) {
    let pairs = [
        ("default_geometry", "narrow_devices_geometry"),
        (
            "default_geometry_with_four_slots_per_region",
            "default_geometry_with_sixteen_slots_per_region",
        ),
        ("default_geometry", "unequal_devices_geometry"),
    ];
    for (first, second) in pairs {
        let (Some(first_tally), Some(second_tally)) = (
            merged_by_geometry.get(first),
            merged_by_geometry.get(second),
        ) else {
            continue;
        };
        for arm in all_arms() {
            let over = |tally: &FamilyTally| {
                tally
                    .arms
                    .get(&arm)
                    .map_or(0, |arm_tally| arm_tally.over_states)
            };
            let (reading_first, reading_second) = (over(first_tally), over(second_tally));
            append_sensitivity_pair(
                output,
                &format!("over_reporting_states_of_{}", arm.label()),
                first,
                i128::from(reading_first),
                second,
                i128::from(reading_second),
                (reading_first >= 1) != (reading_second >= 1),
            );
        }
    }
    for (first, second) in [("default_geometry", "narrow_devices_geometry")] {
        let (Some(first_tally), Some(second_tally)) = (
            merged_by_geometry.get(first),
            merged_by_geometry.get(second),
        ) else {
            continue;
        };
        let nonzero = |tally: &FamilyTally, pick: fn(&ResidualTally) -> u64| -> u64 {
            tally.residuals.iter().map(pick).sum()
        };
        for (quantity, pick) in [
            (
                "rewritten_check_residual_nonzero_states",
                (|residual: &ResidualTally| residual.rewritten_check_residual_nonzero_states)
                    as fn(&ResidualTally) -> u64,
            ),
            (
                "original_formula_residual_nonzero_states",
                (|residual: &ResidualTally| residual.original_formula_residual_nonzero_states)
                    as fn(&ResidualTally) -> u64,
            ),
        ] {
            let (reading_first, reading_second) =
                (nonzero(first_tally, pick), nonzero(second_tally, pick));
            append_sensitivity_pair(
                output,
                quantity,
                first,
                i128::from(reading_first),
                second,
                i128::from(reading_second),
                (reading_first >= 1) != (reading_second >= 1),
            );
        }
    }
    for (first, second) in [
        (
            "default_geometry_with_sixteen_slots_per_region",
            "default_geometry_with_four_slots_per_region",
        ),
        ("default_geometry", "narrow_devices_geometry"),
    ] {
        for family in ["overwrite", "many_mounts", "long_mount"] {
            let (Some(first_trajectory), Some(second_trajectory)) = (
                trajectories.get(&(first, family)),
                trajectories.get(&(second, family)),
            ) else {
                continue;
            };
            let peak = |trajectory: &TrajectoryTally| trajectory.peak_after_the_first.unwrap_or(0);
            let (reading_first, reading_second) = (peak(first_trajectory), peak(second_trajectory));
            append_sensitivity_pair(
                output,
                &format!("peak_closed_minus_ring_window_slots_device_0_{family}"),
                first,
                reading_first,
                second,
                reading_second,
                (reading_first > 0) != (reading_second > 0),
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// 入口
// ---------------------------------------------------------------------------------------------------------------------

/// 登记第七节 B2、B3：独立算术。
pub(super) fn append_independent_arithmetic(output: &mut Vec<String>) {
    let payload_by_subtraction = DATA_UNIT_BYTES - 134;
    // B3：每盘分配记录树层级 0 罩 262 个位置、层级 1 罩 2 个，两盘 528 加根 1。
    const LEAF_SLOTS: u64 = 812;
    const INTERNAL_FANOUT: u64 = 169;
    let first_slot = DEFAULT_UNIT_AREA_START_SLOT;
    let last_slot = FOUR_GIBIBYTES / SLOT_BYTES - 1;
    let level_zero_positions = last_slot / LEAF_SLOTS - first_slot / LEAF_SLOTS + 1;
    let level_one_span = LEAF_SLOTS * INTERNAL_FANOUT;
    let level_one_positions = last_slot / level_one_span - first_slot / level_one_span + 1;
    let nodes_over_two_devices = DEVICE_COUNT * (level_zero_positions + level_one_positions) + 1;
    output.push(
        ProductLine::new("independent_arithmetic")
            .field("payload_bytes_by_subtraction", payload_by_subtraction)
            .field("payload_bytes_by_subtraction_holds", payload_by_subtraction == DATA_UNIT_PAYLOAD_BYTES)
            .field("allocation_record_tree_level_zero_positions_per_device", level_zero_positions)
            .field("allocation_record_tree_level_one_positions_per_device", level_one_positions)
            .field("allocation_record_tree_nodes_over_two_default_devices", nodes_over_two_devices)
            .field("allocation_record_tree_nodes_by_independent_arithmetic_holds", nodes_over_two_devices == 529)
            .field(
                "allocation_record_tree_nodes_anchor_implementation_side",
                "not_observable_through_a_public_entry_allocation_record_tree_positions_over_the_unit_areas_is_private",
            )
            .finish(),
    );
}

#[cfg(test)]
mod tests;
