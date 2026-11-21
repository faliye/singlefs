//! 两条臂的报告、常驻内存报告、段间报告、用户规模步数报告与旁量报告。

use super::*;

/// Q6、Q8（B4 臂）与 Q7：交回 (Q6 总结, Q8 的 B4 臂总结)。
pub(super) fn arms_report(counts: &Product, out: &mut String) -> (Summary, Summary) {
    let mut by_delta_bands = Vec::new();
    let mut narrowed_bands = Vec::new();
    let mut by_delta_unread = false;
    let mut narrowed_unread = false;
    for state in STATES {
        for kind in KINDS {
            let mut inputs: BTreeMap<String, Result<StepsByRow, Vec<String>>> = BTreeMap::new();
            for point in counts.points.keys() {
                inputs.insert(
                    point.clone(),
                    by_delta_rows_by_segment(counts, point, state, kind),
                );
            }
            for (point, input) in &inputs {
                if let Err(gaps) = input {
                    by_delta_unread = true;
                    narrowed_unread = true;
                    let _ = writeln!(
                        out,
                        "name=b8_input_gap point={point} state={state} kind={kind} unread={}",
                        gaps.join(",")
                    );
                }
            }
            for axis in AXES {
                let by_delta_total = |point: &str| match inputs.get(point) {
                    Some(Ok(rows)) => Some(total_of(rows, &SEGMENTS_BEFORE_ADMISSION)),
                    _ => None,
                };
                let reading = alpha_text_for_points(counts, axis, &by_delta_total);
                if reading != AxisReading::NoEnds {
                    let _ = writeln!(
                        out,
                        "name=q6 axis={axis} state={state} kind={kind} {}",
                        reading_text(&reading)
                    );
                    by_delta_bands.push(reading_band(&reading));
                }
                let narrowed_before_the_second_judgement = |point: &str| match inputs.get(point) {
                    Some(Ok(rows)) => Some(total_of(rows, &SEGMENTS_BEFORE_THE_SECOND_JUDGEMENT)),
                    _ => None,
                };
                let narrowed_reading =
                    alpha_text_for_points(counts, axis, &narrowed_before_the_second_judgement);
                if narrowed_reading != AxisReading::NoEnds {
                    let _ = writeln!(
                        out,
                        "name=q8 arm=b4 axis={axis} state={state} kind={kind} {}",
                        reading_text(&narrowed_reading)
                    );
                    narrowed_bands.push(reading_band(&narrowed_reading));
                }
                let today_before_the_second_judgement = |point: &str| {
                    Some(total_of(
                        &today_rows_by_segment(counts, point, state, kind),
                        &SEGMENTS_BEFORE_THE_SECOND_JUDGEMENT,
                    ))
                };
                let today_reading =
                    alpha_text_for_points(counts, axis, &today_before_the_second_judgement);
                if today_reading != AxisReading::NoEnds {
                    let _ = writeln!(
                        out,
                        "name=q8 arm=b1 axis={axis} state={state} kind={kind} {}",
                        reading_text(&today_reading)
                    );
                }
            }
            // 贡献最大的三行（高端点）：B8 在三段合计里步数最大的三行，与它们吃的差量输入名。
            if let Some((point, Ok(rows))) = counts
                .points
                .keys()
                .filter_map(|point_name| inputs.get(point_name).map(|result| (point_name, result)))
                .max_by_key(|(point_name, _)| {
                    counts.points[*point_name].unit_area_slots
                        + counts.points[*point_name].unlinks
                        + counts.points[*point_name].data_units
                        + counts.points[*point_name].files
                })
            {
                let mut by_row: BTreeMap<&str, u64> = BTreeMap::new();
                for ((_, row), steps) in rows {
                    *by_row.entry(*row).or_default() += steps;
                }
                let mut ranked: Vec<(&str, u64)> = by_row.into_iter().collect();
                ranked.sort_by_key(|(_, steps)| std::cmp::Reverse(*steps));
                let top: Vec<String> = ranked
                    .iter()
                    .take(3)
                    .map(|(row, steps)| format!("{row}:{steps}"))
                    .collect();
                let _ = writeln!(
                    out,
                    "name=q6_top_rows point={point} state={state} kind={kind} rows={}",
                    top.join(",")
                );
            }
        }
    }
    (
        summarize(&by_delta_bands, by_delta_unread),
        summarize(&narrowed_bands, narrowed_unread),
    )
}

/// Q7：B8 常驻内存与维护（每个点，第 1 轮，覆盖写那一种）。
pub(super) fn memory_report(counts: &Product, out: &mut String) {
    for state in STATES {
        let mut maintenance_by_point: BTreeMap<String, u64> = BTreeMap::new();
        for (point, axes) in &counts.points {
            let at = round_key(point, state, "overwrite", 1);
            let unit_bytes = number(&counts.sizes, "unit_bytes");
            let key_bytes = number(&counts.sizes, "key_bytes");
            let row_bytes = number(&counts.sizes, "accounting_row_bytes");
            let (Some(unit_bytes), Some(key_bytes), Some(accounting_row_bytes)) =
                (unit_bytes, key_bytes, row_bytes)
            else {
                let _ = writeln!(out, "name=q7_gap point={point} state={state} unread=sizes");
                continue;
            };
            let Ok(input) = by_delta_input_from_the_product(
                counts,
                point,
                state,
                "overwrite",
                &SEGMENTS_BEFORE_ADMISSION,
            ) else {
                let _ = writeln!(
                    out,
                    "name=q7_gap point={point} state={state} unread=b8_input"
                );
                continue;
            };
            let memory_input = MemoryInput {
                allocation_records: input.allocation_records,
                segments_per_disk: segments_of(axes.unit_area_slots),
                units: input.units,
                mapped_units: input.mapped_units,
                files_in_the_index: input.other_files + 1,
                extent_upper_nodes: input.extent_upper_nodes,
                deadlist_entries: input.deadlist_entries,
                accounting_rows_of_the_generation: input.accounting_rows_of_the_generation,
                unit_bytes,
                key_bytes,
                accounting_row_bytes,
            };
            let memory = by_delta_memory(&memory_input);
            let last_round = input.settling_rounds;
            let deltas = |key: &str| {
                counts
                    .deltas_of(&at, key, &["between"])
                    .into_iter()
                    .filter(|(round, _)| *round == last_round)
                    .map(|(_, value)| value)
                    .max()
                    .unwrap_or(0)
            };
            let records_changed =
                deltas("round_placements_released") + deltas("round_roles_rewritten");
            let rewritten_roles = deltas("round_roles_rewritten");
            let upper_rewritten = counts
                .deltas_of(
                    &at,
                    "extent_upper_nodes_rewritten",
                    &SEGMENTS_BEFORE_ADMISSION,
                )
                .into_iter()
                .map(|(_, value)| value)
                .max()
                .unwrap_or(0);
            let maintenance = MaintenanceInput {
                records_changed,
                segments_flipped: records_changed,
                units_delta: rewritten_roles,
                mapped_delta: rewritten_roles,
                files_delta: input.touched_files,
                upper_delta: upper_rewritten,
                deadlist_delta: input.deadlist_changed_entries,
            };
            let steps = by_delta_maintenance(&memory_input, &maintenance);
            maintenance_by_point.insert(point.clone(), steps);
            let peak_round = input
                .changed_slots_and_records_per_round
                .iter()
                .copied()
                .max()
                .unwrap_or(0);
            let _ = writeln!(
                out,
                "name=q7 point={point} state={state} payload_bytes={} payload_bytes_times_two={} segment_summaries={} units_by_role={} mapped_units_by_role={} record_index={} maintenance_steps={steps} undo_log_bytes={} delta_view_peak_bytes={}",
                memory.payload(),
                2 * memory.payload(),
                memory.segment_summaries,
                memory.units_by_role,
                memory.mapped_units_by_role,
                memory.record_index,
                undo_log_bytes(records_changed),
                undo_log_bytes(peak_round)
            );
        }
        for axis in AXES {
            let reading = alpha_text_for_points(counts, axis, &|point: &str| {
                maintenance_by_point.get(point).copied()
            });
            if reading != AxisReading::NoEnds {
                let _ = writeln!(
                    out,
                    "name=q7_maintenance_alpha axis={axis} state={state} {}",
                    reading_text(&reading)
                );
            }
        }
    }
}

/// B4 臂两道之间那一段（S2）一轮的挂钟（纳秒）：S2 里没改的行取独占计时，改过的行取 B8 步数乘那一行 B1 的每步纳秒，再加 S2 残差。
/// 这一轮过不了包含自检时交回 `None`。
fn narrowed_between_nanos(
    timing: &Product,
    at: &RoundKey,
    by_delta: &StepsByRow,
    today: &StepsByRow,
    called: &BTreeSet<&'static str>,
) -> Option<f64> {
    if !round_is_usable(timing, at) {
        return None;
    }
    let wall = timing.wall_of(at, "between").unwrap_or(0);
    let exclusive = exclusive_nanos_in_the_segment(timing, at, "between");
    let mut nanos = to_f64(wall.saturating_sub(exclusive));
    for row in scan_rows() {
        let own = to_f64(timing.nanos_of(at, &["between"], row));
        if called.contains(row) {
            let today_steps = to_f64(today.get(&("between", row)).copied().unwrap_or(0));
            let per_step = if today_steps > 0.0 {
                own / today_steps
            } else {
                0.0
            };
            nanos += to_f64(by_delta.get(&("between", row)).copied().unwrap_or(0)) * per_step;
        } else {
            nanos += own;
        }
    }
    Some(nanos)
}

/// Q9：B4 臂两道之间那一段（S2）的步数与挂钟。挂钟 = S2 里没改的行的独占计时 + S2 残差 + 改过的行的 B8 步数 × 那一行 B1 的每步纳秒。
pub(super) fn between_segment_report(counts: &Product, timing: Option<&Product>, out: &mut String) {
    for state in STATES {
        for kind in KINDS {
            let mut steps_by_point: BTreeMap<String, u64> = BTreeMap::new();
            for point in counts.points.keys() {
                let Ok(by_delta) = by_delta_rows_by_segment(counts, point, state, kind) else {
                    continue;
                };
                let today = today_rows_by_segment(counts, point, state, kind);
                let called = rows_called_before_the_second_judgement(counts, point, state, kind);
                let [_, _, between] = narrowed_steps_by_segment(&today, &by_delta, &called);
                steps_by_point.insert(point.clone(), between);
                let mut nanos_text = String::new();
                if let Some(timing) = timing {
                    let mut per_round = Vec::new();
                    for round in 1..=ROUNDS {
                        let at = round_key(point, state, kind, round);
                        if let Some(nanos) =
                            narrowed_between_nanos(timing, &at, &by_delta, &today, &called)
                        {
                            per_round.push(nanos);
                        }
                    }
                    nanos_text = format!(
                        " nanos_of_each_valid_round={}",
                        join_numbers(
                            &per_round
                                .iter()
                                .map(|value| format!("{value:.0}"))
                                .collect::<Vec<_>>()
                        )
                    );
                }
                let _ = writeln!(out, "name=q9 point={point} state={state} kind={kind} steps_between={between}{nanos_text}");
            }
            for axis in AXES {
                let reading = alpha_text_for_points(counts, axis, &|point: &str| {
                    steps_by_point.get(point).copied()
                });
                if reading != AxisReading::NoEnds {
                    let _ = writeln!(
                        out,
                        "name=q9_alpha axis={axis} state={state} kind={kind} {}",
                        reading_text(&reading)
                    );
                }
            }
        }
    }
}

// ---- Q6、Q8、Q9 在用户关心的规模上（推的，与 Q4 同一个外推法）----
/// 一个按点算出的量在 G-user 上的值：先算出每个点的值（任何一个点读不到，整格读不到，不拿 0 补），
/// 再按 Q4 的外推法取 B0 上的值加每根轴从 B0 到 G-user 那一轴取值的增量。交回值与没能外推的轴。
pub(super) fn quantity_at_user_scale(
    counts: &Product,
    state: &str,
    kind: &str,
    scale: &UserScale,
    per_point: &dyn Fn(&str) -> Result<u64, Vec<String>>,
) -> Result<(f64, Vec<&'static str>), Vec<String>> {
    if !counts.points.contains_key(BASE_POINT_NAME) {
        return Err(vec!["point@base".to_string()]);
    }
    let mut values: BTreeMap<String, f64> = BTreeMap::new();
    let mut gaps: Vec<String> = Vec::new();
    for point in counts.points.keys() {
        match per_point(point) {
            Ok(value) => {
                values.insert(point.clone(), to_f64(value));
            }
            Err(point_gaps) => gaps.push(format!("{point}:{}", point_gaps.join("|"))),
        }
    }
    if !gaps.is_empty() {
        return Err(gaps);
    }
    let quantity = |at: &RoundKey| values.get(&at.0).copied().unwrap_or(0.0);
    Ok(user_scale_value(counts, state, kind, 1, &quantity, scale))
}

fn user_scale_text(label: &str, result: &Result<(f64, Vec<&'static str>), Vec<String>>) -> String {
    match result {
        Ok((steps, missing)) => format!(
            "{label}_steps={steps:.0} {label}_axes_not_extrapolated={}",
            missing.join(",")
        ),
        Err(gaps) => format!("{label}_unread={}", gaps.join(";")),
    }
}

/// Q6（B8 准入之前扫类步数）、Q8（B4 臂落点判之前剩下的步数）、Q9（B4 臂两道之间的步数与挂钟）在 G-user 上的值，
/// 两档 D_user × 两种挂载状态 × 三种发布；每个量旁边是 B1 今天的同一个量。有计时产物时另算 Q9 的挂钟（覆盖写，五轮各一个）。
pub(super) fn user_scale_steps_report(counts: &Product, timing: Option<&Product>, out: &mut String) {
    for state in STATES {
        for kind in KINDS {
            for data_units in [10.0, 268_435_456.0] {
                let scale = user_scale(data_units);
                let today_before_admission =
                    quantity_at_user_scale(counts, state, kind, &scale, &|point: &str| -> Result<
                        u64,
                        Vec<String>,
                    > {
                        Ok(scan_steps(
                            counts,
                            point,
                            state,
                            kind,
                            &SEGMENTS_BEFORE_ADMISSION,
                        ))
                    });
                let by_delta_before_admission =
                    quantity_at_user_scale(counts, state, kind, &scale, &|point: &str| -> Result<
                        u64,
                        Vec<String>,
                    > {
                        let rows = by_delta_rows_by_segment(counts, point, state, kind)?;
                        Ok(total_of(&rows, &SEGMENTS_BEFORE_ADMISSION))
                    });
                let today_before_landing_judgement =
                    quantity_at_user_scale(counts, state, kind, &scale, &|point: &str| -> Result<
                        u64,
                        Vec<String>,
                    > {
                        Ok(scan_steps(
                            counts,
                            point,
                            state,
                            kind,
                            &SEGMENTS_BEFORE_THE_SECOND_JUDGEMENT,
                        ))
                    });
                let narrowed_before_landing_judgement =
                    quantity_at_user_scale(counts, state, kind, &scale, &|point: &str| -> Result<
                        u64,
                        Vec<String>,
                    > {
                        let rows = by_delta_rows_by_segment(counts, point, state, kind)?;
                        Ok(total_of(&rows, &SEGMENTS_BEFORE_THE_SECOND_JUDGEMENT))
                    });
                let today_between =
                    quantity_at_user_scale(counts, state, kind, &scale, &|point: &str| -> Result<
                        u64,
                        Vec<String>,
                    > {
                        Ok(scan_steps(counts, point, state, kind, &["between"]))
                    });
                let narrowed_between =
                    quantity_at_user_scale(counts, state, kind, &scale, &|point: &str| -> Result<
                        u64,
                        Vec<String>,
                    > {
                        let by_delta = by_delta_rows_by_segment(counts, point, state, kind)?;
                        let today = today_rows_by_segment(counts, point, state, kind);
                        let called =
                            rows_called_before_the_second_judgement(counts, point, state, kind);
                        let [_, _, between] = narrowed_steps_by_segment(&today, &by_delta, &called);
                        Ok(between)
                    });
                let _ = writeln!(
                    out,
                    "name=q6_user_scale state={state} kind={kind} data_units_user={data_units:.0} {} {}",
                    user_scale_text("by_delta", &by_delta_before_admission),
                    user_scale_text("today", &today_before_admission)
                );
                let _ = writeln!(
                    out,
                    "name=q8_user_scale state={state} kind={kind} data_units_user={data_units:.0} {} {}",
                    user_scale_text("narrowed", &narrowed_before_landing_judgement),
                    user_scale_text("today", &today_before_landing_judgement)
                );
                let _ = writeln!(
                    out,
                    "name=q9_user_scale state={state} kind={kind} data_units_user={data_units:.0} {} {}",
                    user_scale_text("narrowed", &narrowed_between),
                    user_scale_text("today", &today_between)
                );
                if let (true, Some(timing)) = (kind == "overwrite", timing) {
                    let text = narrowed_between_nanos_at_user_scale(counts, timing, state, &scale);
                    let _ = writeln!(
                        out,
                        "name=q9_user_scale_nanos state={state} kind={kind} data_units_user={data_units:.0} {text}"
                    );
                }
            }
        }
    }
}

/// Q9 的挂钟在 G-user 上：每个有效轮一个值（五个有效轮才算），点上读不到、轮不能用的整格记读不到。
fn narrowed_between_nanos_at_user_scale(
    counts: &Product,
    timing: &Product,
    state: &str,
    scale: &UserScale,
) -> String {
    let usable = usable_rounds(timing, BASE_POINT_NAME, state, "overwrite");
    let mut nanos_of_each_round: Vec<f64> = Vec::new();
    let mut missing_axes: Vec<&'static str> = Vec::new();
    let mut unread: Vec<String> = Vec::new();
    for round in &usable {
        let mut values: BTreeMap<String, f64> = BTreeMap::new();
        for point in counts.points.keys() {
            let Ok(by_delta) = by_delta_rows_by_segment(counts, point, state, "overwrite") else {
                unread.push(format!("{point}:b8_input"));
                continue;
            };
            let today = today_rows_by_segment(counts, point, state, "overwrite");
            let called = rows_called_before_the_second_judgement(counts, point, state, "overwrite");
            let at = round_key(point, state, "overwrite", *round);
            match narrowed_between_nanos(timing, &at, &by_delta, &today, &called) {
                Some(nanos) => {
                    values.insert(point.clone(), nanos);
                }
                None => unread.push(format!("{point}@round{round}")),
            }
        }
        let quantity = |at: &RoundKey| values.get(&at.0).copied().unwrap_or(0.0);
        let (value, missing) =
            user_scale_value(counts, state, "overwrite", *round, &quantity, scale);
        nanos_of_each_round.push(value);
        missing_axes = missing;
    }
    unread.sort();
    unread.dedup();
    if !unread.is_empty() {
        return format!("unread={}", unread.join(";"));
    }
    if usable.len() as u64 != ROUNDS {
        return "fewer_than_five_valid_rounds".to_string();
    }
    format!(
        "nanos_of_each_round={} axes_not_extrapolated={}",
        join_numbers(
            &nanos_of_each_round
                .iter()
                .map(|value| format!("{value:.0}"))
                .collect::<Vec<_>>()
        ),
        missing_axes.join(",")
    )
}

/// Q11、Q12、Q13：附带，只报数。
pub(super) fn side_quantities_report(counts: &Product, out: &mut String) {
    for point in counts.points.keys() {
        for state in STATES {
            let at = round_key(point, state, "overwrite", 1);
            let after: u64 = scan_rows()
                .iter()
                .map(|row| counts.steps_of(&at, &["after"], row))
                .sum();
            let copies: Vec<String> = all_rows()
                .iter()
                .filter(|(_, kind)| *kind == RowKind::Copy)
                .map(|(row, _)| {
                    format!(
                        "{row}:{}",
                        counts.steps_of(&at, &SEGMENTS_BEFORE_ADMISSION, row)
                    )
                })
                .collect();
            let _ = writeln!(
                out,
                "name=q11 point={point} state={state} scan_steps_after_the_judgements={after}"
            );
            let _ = writeln!(
                out,
                "name=q13 point={point} state={state} copy_rows={}",
                copies.join(",")
            );
        }
        if let Some((count, fewest, most)) = counts.history_unlinks.get(point) {
            let _ = writeln!(out, "name=q12 point={point} history_unlinks={count} publishes_per_unlink_fewest={fewest} publishes_per_unlink_most={most}");
        }
    }
    for ((point, state, round), publishes) in &counts.unlink_publishes {
        let _ = writeln!(out, "name=q12_measured_unlink point={point} state={state} round={round} publishes={publishes}");
    }
    for ((point, state), (taken, attempts)) in &counts.rounds_taken {
        let _ = writeln!(out, "name=q12_overwrite_rounds point={point} state={state} taken={taken} attempts={attempts}");
    }
}

#[cfg(test)]
#[path = "arms_memory_and_scale_reports_tests.rs"]
mod tests;
