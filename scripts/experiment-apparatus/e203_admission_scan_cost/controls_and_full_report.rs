//! 对照报告、状态差异字段、跳过调用方或判定的发布、整份报告、报告入口与锚点行。

use super::*;

/// 阳性对照与阴性对照（登记第五节）。交回每条是否过。
pub(super) fn controls_report(counts: &Product, timing: Option<&Product>, out: &mut String) -> bool {
    let mut all_pass = true;
    let mut record = |name: &str, state: &str, passed: bool, detail: String, report_text: &mut String| {
        all_pass &= passed;
        let _ = writeln!(
            report_text,
            "name=verdict control={name} state={state} passed={passed} {detail}"
        );
    };
    let copy_row = "whole_allocator_copy_before_the_judgement";
    let settle_copy_row = "whole_allocator_copy_in_a_settling_round";
    for state in STATES {
        let at = |point: &str| round_key(point, state, "overwrite", 1);
        let (Some(low), Some(high)) = (
            axis_points_for_alpha(counts, "U")
                .first()
                .map(|point_axes| point_axes.0.clone()),
            axis_points_for_alpha(counts, "U")
                .last()
                .map(|point_axes| point_axes.0.clone()),
        ) else {
            record("axis_u_has_two_points", state, false, String::new(), out);
            continue;
        };
        let steps =
            |point: &str, segments: &[&str], row: &str| counts.steps_of(&at(point), segments, row);
        // B1 计数：s1 的 α(U) ≥ 0.95
        let (copy_low, copy_high) = (
            steps(&low, &["caller"], copy_row),
            steps(&high, &["caller"], copy_row),
        );
        let copy_alpha = alpha(
            to_f64(copy_low.max(1)),
            to_f64(copy_high.max(1)),
            to_f64(1 << 14),
            to_f64(1 << 24),
        );
        record(
            "b1_counts_whole_copy_alpha_at_least_0_95",
            state,
            copy_alpha >= 0.95,
            format!("alpha={copy_alpha:.6}"),
            out,
        );
        // B8 与 B4：退化后的 α(U) ≥ 0.9
        let rows_at = |point: &str| {
            (
                by_delta_rows_by_segment(counts, point, state, "overwrite"),
                today_rows_by_segment(counts, point, state, "overwrite"),
            )
        };
        let (by_delta_low, today_low) = rows_at(&low);
        let (by_delta_high, today_high) = rows_at(&high);
        if let (Ok(by_delta_low), Ok(by_delta_high)) = (by_delta_low, by_delta_high) {
            let degraded = |by_delta: &StepsByRow, today: &StepsByRow| {
                let by_delta_rest = total_of(by_delta, &SEGMENTS_BEFORE_ADMISSION)
                    - by_delta.get(&("caller", copy_row)).copied().unwrap_or(0)
                    - by_delta
                        .get(&("between", settle_copy_row))
                        .copied()
                        .unwrap_or(0);
                degraded_by_delta_total(
                    by_delta_rest,
                    today.get(&("caller", copy_row)).copied().unwrap_or(0),
                    today
                        .get(&("between", settle_copy_row))
                        .copied()
                        .unwrap_or(0),
                )
            };
            let passed = positive_control_passes(
                degraded(&by_delta_low, &today_low),
                degraded(&by_delta_high, &today_high),
                0.9,
            );
            record(
                "b8_degraded_alpha_at_least_0_9",
                state,
                passed,
                String::new(),
                out,
            );
            let narrowed_degraded = |by_delta: &StepsByRow, today: &StepsByRow| {
                total_of(by_delta, &SEGMENTS_BEFORE_THE_SECOND_JUDGEMENT)
                    - by_delta.get(&("caller", copy_row)).copied().unwrap_or(0)
                    + today.get(&("caller", copy_row)).copied().unwrap_or(0)
            };
            let narrowed_passed = positive_control_passes(
                narrowed_degraded(&by_delta_low, &today_low),
                narrowed_degraded(&by_delta_high, &today_high),
                0.9,
            );
            record(
                "b4_degraded_alpha_at_least_0_9",
                state,
                narrowed_passed,
                String::new(),
                out,
            );
            // 阴性对照：s33 的 B1 步数与 B8 的 s12 步数在 U 轴两端 |α| ≤ 0.1
            let negative_today = negative_control_passes(
                today_low
                    .get(&("before_landing", "units_of_a_publish_to_land"))
                    .copied()
                    .unwrap_or(0)
                    .max(1),
                today_high
                    .get(&("before_landing", "units_of_a_publish_to_land"))
                    .copied()
                    .unwrap_or(0)
                    .max(1),
            );
            record(
                "negative_units_to_land_b1",
                state,
                negative_today,
                String::new(),
                out,
            );
            let root_head = |rows: &StepsByRow| {
                rows.iter()
                    .filter(|((_, row), _)| *row == "root_header_linear_find")
                    .map(|(_, row_steps)| row_steps)
                    .sum::<u64>()
            };
            let negative_by_delta = negative_control_passes(
                root_head(&by_delta_low).max(1),
                root_head(&by_delta_high).max(1),
            );
            record(
                "negative_root_header_b8",
                state,
                negative_by_delta,
                String::new(),
                out,
            );
        } else {
            record(
                "b8_b4_controls_inputs_read",
                state,
                false,
                "unread=b8_input".to_string(),
                out,
            );
        }
        // B1 计时：s1 的独占计时五轮每轮 α(U) ≥ 0.7
        if let Some(timing) = timing {
            let mut alphas = Vec::new();
            for round in 1..=ROUNDS {
                let (low_at, high_at) = (
                    round_key(&low, state, "overwrite", round),
                    round_key(&high, state, "overwrite", round),
                );
                if !(round_is_usable(timing, &low_at) && round_is_usable(timing, &high_at)) {
                    continue;
                }
                let (low_nanos, high_nanos) = (
                    timing.nanos_of(&low_at, &["caller"], copy_row),
                    timing.nanos_of(&high_at, &["caller"], copy_row),
                );
                if low_nanos > 0 && high_nanos > 0 {
                    alphas.push(alpha(
                        to_f64(low_nanos),
                        to_f64(high_nanos),
                        to_f64(1 << 14),
                        to_f64(1 << 24),
                    ));
                }
            }
            let passed =
                alphas.len() as u64 == ROUNDS && alphas.iter().all(|growth| *growth >= 0.7);
            record(
                "b1_timing_whole_copy_alpha_at_least_0_7_in_every_round",
                state,
                passed,
                format!(
                    "alphas={}",
                    join_numbers(
                        &alphas
                            .iter()
                            .map(|growth| format!("{growth:.6}"))
                            .collect::<Vec<_>>()
                    )
                ),
                out,
            );
        }
    }
    all_pass
}

// ---- 停机条款、作废条款、失败条款的判读 ----
/// 停机条款 3：计数产物与计时产物同一个点、同一轮上的规模读数逐字段相同（除了 `build` 字段）。交回不同的字段个数。
fn state_fields_that_differ(counts: &Product, timing: &Product) -> u64 {
    let mut differing = 0;
    for (key, count_fields) in &counts.states {
        let Some(timing_fields) = timing.states.get(key) else {
            differing += 1;
            continue;
        };
        for (field, value) in count_fields {
            if field != "build" && timing_fields.get(field) != Some(value) {
                differing += 1;
            }
        }
    }
    differing
}

/// 停机条款 4：每个被量的发布都经调用方进门（有 `caller` 段），并且进了落点判之前那一段。交回违例个数。
fn publishes_that_skipped_the_caller_or_the_judge(counts: &Product) -> u64 {
    let mut violations = 0;
    for key in counts.publishes.keys() {
        let had_caller = counts.had_caller_segment(key) == Some(true);
        let entered_the_judge = counts.rows.keys().any(|row| {
            row.0 == key.0
                && row.1 == key.1
                && row.2 == key.2
                && row.3 == key.3
                && row.4 == "before_landing"
        });
        if !had_caller || !entered_the_judge {
            violations += 1;
        }
    }
    violations
}

fn clause_line(out: &mut String, clause: &str, fired: Option<bool>, detail: &str) {
    let fired = fired.map_or("unknown".to_string(), |flag| flag.to_string());
    let _ = writeln!(out, "name=verdict clause={clause} fired={fired} {detail}");
}

/// 一份计数产物（可带一份计时产物）的全部判读。
pub(super) fn full_report(counts: &Product, timing: Option<&Product>, out: &mut String) {
    // 作废条款 8：量不到的格整格拿掉，不当 0 进任何一个量；每一格另点名。
    let cells_of_counts = cells_not_measurable(counts);
    let cells_of_timing = timing.map(cells_not_measurable).unwrap_or_default();
    let (frozen_in_counts, frozen_unread_in_counts) = points_with_a_frozen_publish(counts);
    let (frozen_in_timing, frozen_unread_in_timing) = timing.map_or((BTreeSet::new(), 0), points_with_a_frozen_publish);
    for (build, points) in [("counts", &frozen_in_counts), ("timing", &frozen_in_timing)] {
        for point in points {
            let _ = writeln!(out, "name=void_frozen_publish build={build} point={point} result=point_voided");
        }
    }
    let _ = writeln!(
        out,
        "name=verdict clause=void_frozen_publish points_voided_in_counts={} points_voided_in_timing={} cells_without_the_reading_in_counts={frozen_unread_in_counts} cells_without_the_reading_in_timing={frozen_unread_in_timing}",
        frozen_in_counts.len(),
        frozen_in_timing.len()
    );
    let voided_points: BTreeSet<String> = frozen_in_counts.union(&frozen_in_timing).cloned().collect();
    let counts = &without_the_points(counts, &voided_points);
    let timing = timing.map(|product| without_the_points(product, &voided_points));
    let timing = timing.as_ref();
    let cleaned_counts = without_the_cells_not_measurable(counts, &cells_of_counts);
    let cleaned_timing =
        timing.map(|product| without_the_cells_not_measurable(product, &cells_of_timing));
    let (measurable_counts, measurable_timing) = (&cleaned_counts, cleaned_timing.as_ref());
    // 计时产物里各点的轴值从同名计数产物取（修订 7）；计时这一侧的每个量都读这一份
    let (timing_with_axes, points_with_the_axes_unread) = match measurable_timing {
        Some(product) => {
            let (with_axes, unread) = timing_with_the_point_axes_of_the_counts(measurable_counts, product);
            let _ = writeln!(
                out,
                "name=timing_point_axes points_of_the_timing={} taken_from_the_counts={} unread={}",
                point_names_of_the_timing(product).len(),
                with_axes.points.len() - product.points.len(),
                unread.len()
            );
            (Some(with_axes), unread)
        }
        None => (None, Vec::new()),
    };
    let timing_read_with_axes = timing_with_axes.as_ref();
    for (build, cells) in [("counts", &cells_of_counts), ("timing", &cells_of_timing)] {
        for (point, state, taken, attempts) in cells {
            let _ = writeln!(
                out,
                "name=void_8_cell build={build} point={point} state={state} kind=overwrite taken={taken} attempts={attempts} result=not_measurable"
            );
        }
    }
    let _ = writeln!(
        out,
        "name=verdict clause=void_8 cells_not_measurable_in_counts={} cells_not_measurable_in_timing={}",
        cells_of_counts.len(),
        cells_of_timing.len()
    );
    counts_report(measurable_counts, out);
    let (by_delta_summary, narrowed_summary) = arms_report(measurable_counts, out);
    pooled_fallback_report(measurable_counts, out);
    memory_report(measurable_counts, out);
    between_segment_report(measurable_counts, timing_read_with_axes, out);
    user_scale_steps_report(measurable_counts, timing_read_with_axes, out);
    side_quantities_report(measurable_counts, out);
    let _ = writeln!(out, "name=verdict quantity=Q6 result={by_delta_summary:?}");
    let _ = writeln!(
        out,
        "name=verdict quantity=Q8_b4_arm result={narrowed_summary:?}"
    );
    let controls_pass = controls_report(measurable_counts, timing_read_with_axes, out);
    let _ = writeln!(
        out,
        "name=verdict quantity=controls all_passed={controls_pass}"
    );
    // 停机条款与作废条款
    let constant_mismatches = out
        .lines()
        .filter(|line| line.starts_with("name=constant_check") && line.ends_with("match=false"))
        .count();
    // 修订 4：超上界的格与不等的格一样算不符（`disagreements` 里含着，另报其中超上界的几格）
    let three_two = reconciliation_report(measurable_counts, out);
    let _ = writeln!(
        out,
        "name=verdict clause=stop_2 constants_not_matching_or_unread={constant_mismatches} three_two_disagreements={} three_two_cells_over_the_bound={}",
        three_two.disagreements, three_two.over_the_bound
    );
    let _ = writeln!(
        out,
        "name=verdict clause=stop_4 publishes_that_skipped_the_caller_or_the_judge={}",
        publishes_that_skipped_the_caller_or_the_judge(measurable_counts)
    );
    let _ = writeln!(
        out,
        "name=verdict clause=void_6 points={} points_with_the_done_marker={}",
        measurable_counts.points.len(),
        measurable_counts.items_done.len()
    );
    // 失败条款 F1–F6
    let mut residual_clause = None;
    let mut flips_today_clause = None;
    let mut extrapolation_clause = None;
    if let Some(timing_read_with_axes) = timing_read_with_axes {
        let _ = writeln!(
            out,
            "name=verdict clause=stop_3 state_fields_that_differ={}",
            state_fields_that_differ(measurable_counts, timing_read_with_axes)
        );
        let header_ok = timing_read_with_axes.header.get("profile").map(String::as_str) == Some("release")
            && timing_read_with_axes.header.get("opt_level").map(String::as_str) == Some("3")
            && timing_read_with_axes.header.get("test_threads").map(String::as_str) == Some("1");
        let _ = writeln!(out, "name=verdict clause=void_7 timing_header_is_release_opt_level_3_single_thread={header_ok}");
        residual_clause = timing_report(timing_read_with_axes, &points_with_the_axes_unread, out);
        let (cells, reference_only) = one_millisecond_report(measurable_counts, timing_read_with_axes, out);
        let checks = extrapolation_check_report(measurable_counts, timing_read_with_axes, out);
        // 一格都没核（轴上不足三个点）就是没判，不记「核过」。
        extrapolation_clause = (!checks.is_empty()).then(|| checks.iter().any(|ok| !*ok));
        // F6 打中（Q5 有一格核不过）：Q4 记「外推不可信，不判」，F1 跟着不判（登记第十节 F6）
        let extrapolation_not_trusted = extrapolation_clause == Some(true);
        flips_today_clause = if cells.len() == 4 && !reference_only && !extrapolation_not_trusted {
            Some(
                cells
                    .iter()
                    .all(|one_cell| *one_cell == FormulaCell::AtLeastOneMillisecond),
            )
        } else {
            None
        };
        let _ = writeln!(out, "name=verdict quantity=Q4 cells={} reference_only_because_data_or_files_axes_are_missing={reference_only} not_judged_because_the_extrapolation_is_not_trusted={extrapolation_not_trusted}", cells.iter().map(|one_cell| format!("{one_cell:?}")).collect::<Vec<_>>().join(","));
        let _ = writeln!(
            out,
            "name=verdict quantity=Q5 cells_passed={} cells_failed={}",
            checks.iter().filter(|ok| **ok).count(),
            checks.iter().filter(|ok| !**ok).count()
        );
    } else {
        let _ = writeln!(
            out,
            "name=verdict quantity=timing_quantities result=no_timing_product_given"
        );
    }
    clause_line(
        out,
        "F1",
        flips_today_clause,
        "meaning=all_four_Q4_cells_at_least_one_millisecond_flips_B1",
    );
    clause_line(
        out,
        "F2",
        Some(by_delta_summary == Summary::GrowsOnSomeAxis),
        "meaning=Q6_B8_still_grows_on_some_axis",
    );
    clause_line(
        out,
        "F3",
        residual_clause,
        "meaning=residual_grows_on_some_axis_in_all_five_rounds",
    );
    let unlink_not_two = measurable_counts
        .unlink_publishes
        .values()
        .filter(|value| **value != 2)
        .count()
        + measurable_counts
            .history_unlinks
            .values()
            .filter(|(_, fewest, most)| *fewest != 2 || *most != 2)
            .count();
    clause_line(
        out,
        "F4",
        Some(unlink_not_two > 0),
        &format!("unlinks_not_two_publishes={unlink_not_two}"),
    );
    clause_line(
        out,
        "F5",
        Some(narrowed_summary == Summary::GrowsOnSomeAxis),
        "meaning=B4_arm_before_the_landing_judgement_still_grows",
    );
    clause_line(
        out,
        "F6",
        extrapolation_clause,
        "meaning=some_Q5_cell_failed",
    );
}

pub(super) fn run_report(paths: &[String]) -> ExitCode {
    let split = paths.iter().position(|path| path == "--timing");
    let (count_paths, timing_paths) = match split {
        Some(index) => (&paths[..index], &paths[index + 1..]),
        None => (paths, &paths[paths.len()..]),
    };
    let read_all = |list: &[String]| -> Option<String> {
        let mut text = String::new();
        for path in list {
            let Ok(part) = std::fs::read_to_string(path) else {
                eprintln!("读不了产物 {path}");
                return None;
            };
            text.push_str(&part);
        }
        Some(text)
    };
    let (Some(count_text), Some(timing_text)) = (read_all(count_paths), read_all(timing_paths))
    else {
        return ExitCode::from(2);
    };
    let counts = read_product(&count_text);
    let timing = (!timing_paths.is_empty()).then(|| read_product(&timing_text));
    let mut out = String::new();
    let _ = writeln!(
        out,
        "name=report_of counts_files={} timing_files={}",
        count_paths.len(),
        timing_paths.len()
    );
    full_report(&counts, timing.as_ref(), &mut out);
    print!("{out}");
    ExitCode::SUCCESS
}

pub(super) fn anchors_lines() -> String {
    let mut out = String::new();
    for (name, unit_area_slots) in [
        ("U=2^14", 1_u64 << 14),
        ("U=2^17", 1 << 17),
        ("U=2^20", 1 << 20),
        ("U=2^24", 1 << 24),
        ("G-user", user_scale_unit_area_slots()),
    ] {
        let _ = writeln!(
            out,
            "name=clone_vectors point={name} unit_area_slots={unit_area_slots} device_bytes={} segments={} elements_two_disks={} bytes_two_disks={} lg_segments={}",
            device_bytes(unit_area_slots),
            segments_of(unit_area_slots),
            allocator_copy_elements_from_the_unit_area(unit_area_slots),
            DISKS * bytes_per_disk(unit_area_slots),
            lg(segments_of(unit_area_slots))
        );
    }
    let total = by_delta_total(&delta_zero());
    let _ = writeln!(out, "name=b8_anchor_delta0 total={total}");
    out
}

#[cfg(test)]
#[path = "controls_and_full_report_tests.rs"]
mod tests;
