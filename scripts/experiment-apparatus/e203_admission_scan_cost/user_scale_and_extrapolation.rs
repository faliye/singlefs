//! 用户关心的规模上的值、一毫秒报告、对角外推格与外推核对报告。

use super::*;

// ---- Q4、Q5：用户关心的规模上的外推 ----
/// 用户关心的规模 G-user：每盘 16 TiB 的单元区，10⁶ 次 unlink，F = 1000，D 两档。
pub(super) struct UserScale {
    pub(super) unit_area_slots: f64,
    pub(super) unlinks: f64,
    pub(super) data_units: f64,
    pub(super) files: f64,
}

pub(super) fn user_scale(data_units: f64) -> UserScale {
    UserScale {
        unit_area_slots: to_f64(user_scale_unit_area_slots()),
        unlinks: 1.0e6,
        data_units,
        files: 1000.0,
    }
}

fn axis_target(axis: &str, scale: &UserScale) -> f64 {
    match axis {
        "U" => scale.unit_area_slots,
        "u" => scale.unlinks,
        "D" => scale.data_units,
        _ => scale.files,
    }
}

/// 沿一根轴的 (x, y) 点列取 x 处的值：有这个 x 的点就取它；落在两点之间线性内插；大于最后一点时用最后两点的斜率线性外推。
pub(super) fn value_on_axis(points: &[(f64, f64)], at_position: f64) -> Option<f64> {
    if let Some((_, value_at)) = points
        .iter()
        .find(|(px, _)| (*px - at_position).abs() < 0.5)
    {
        return Some(*value_at);
    }
    let [.., (earlier_position, earlier_value), (last_position, last_value)] = points else {
        return None;
    };
    if at_position > *last_position {
        return Some(linear_extrapolation(
            *earlier_position,
            *earlier_value,
            *last_position,
            *last_value,
            at_position,
        ));
    }
    let upper = points.iter().position(|(px, _)| *px > at_position)?;
    let (low, high) = (points.get(upper.checked_sub(1)?)?, points.get(upper)?);
    Some(low.1 + (high.1 - low.1) * (at_position - low.0) / (high.0 - low.0))
}

/// 一个量在 G-user 上的值：B0 上的值，加上每根轴上（一次只动一根轴）从 B0 到 G-user 那一轴取值的增量。
/// 交回值与没有两个点可外推的轴（那根轴的增量记 0，调用方标「未外推」）。
pub(super) fn user_scale_value(
    product: &Product,
    state: &str,
    kind: &str,
    round: u64,
    quantity: &dyn Fn(&RoundKey) -> f64,
    scale: &UserScale,
) -> (f64, Vec<&'static str>) {
    let base = quantity(&round_key(BASE_POINT_NAME, state, kind, round));
    let mut total = base;
    let mut missing = Vec::new();
    for axis in AXES {
        let points: Vec<(f64, f64)> = axis_points_for_extrapolation(product, axis)
            .iter()
            .map(|(point, axes)| {
                (
                    to_f64(axis_value(axis, axes)),
                    quantity(&round_key(point, state, kind, round)),
                )
            })
            .collect();
        match value_on_axis(&points, axis_target(axis, scale)) {
            Some(value_at) => total += value_at - base,
            None => missing.push(axis),
        }
    }
    (total, missing)
}

/// τ = 一行的独占计时 ÷ 步数，取那一行步数最大的那个量过的点（登记第六节 Q4）。
fn tau_of_the_row(
    counts: &Product,
    timing: &Product,
    state: &str,
    kind: &str,
    round: u64,
    row: &str,
) -> f64 {
    let points: Vec<(u64, u64)> = counts
        .points
        .keys()
        .map(|point| {
            (
                counts.steps_of(
                    &round_key(point, state, kind, 1),
                    &SEGMENTS_BEFORE_ADMISSION,
                    row,
                ),
                timing.nanos_of(
                    &round_key(point, state, kind, round),
                    &SEGMENTS_BEFORE_ADMISSION,
                    row,
                ),
            )
        })
        .collect();
    tau_nanos_per_step(&points)
}

/// 一轮里一次覆盖写准入之前扫类挂钟在 G-user 上的推值 T，与没能外推的轴。
fn user_scale_nanos(
    counts: &Product,
    timing: &Product,
    state: &str,
    round: u64,
    scale: &UserScale,
) -> (f64, Vec<&'static str>) {
    let mut total = 0.0;
    let mut missing = Vec::new();
    for row in scan_rows() {
        let steps = |at: &RoundKey| {
            to_f64(counts.steps_of(
                &round_key(&at.0, &at.1, &at.2, 1),
                &SEGMENTS_BEFORE_ADMISSION,
                row,
            ))
        };
        let (mut steps_at_user_scale, row_missing) =
            user_scale_value(counts, state, "overwrite", 1, &steps, scale);
        if row == "cluster_segment_sum" && state == "same_mount" {
            // 开过的聚簇段数封顶每盘段数 S（登记第六节 Q4）：这一行每次调用看的段数不会超过 S。
            let calls = to_f64(counts.calls_of(
                &round_key(BASE_POINT_NAME, state, "overwrite", 1),
                &SEGMENTS_BEFORE_ADMISSION,
                row,
            ));
            steps_at_user_scale =
                steps_at_user_scale.min(calls * to_f64(segments_of(user_scale_unit_area_slots())));
        }
        for axis in row_missing {
            if !missing.contains(&axis) {
                missing.push(axis);
            }
        }
        total +=
            steps_at_user_scale * tau_of_the_row(counts, timing, state, "overwrite", round, row);
    }
    (total, missing)
}

/// Q4 四格（两档 D_user × 两种挂载状态）；交回四格的档（只含判得出的），与有没有哪一格的 D、F 轴没外推。
pub(super) fn one_millisecond_report(
    counts: &Product,
    timing: &Product,
    out: &mut String,
) -> (Vec<FormulaCell>, bool) {
    let mut cells = Vec::new();
    let mut reference_only = false;
    for state in STATES {
        for data_units in [10.0, 268_435_456.0] {
            let scale = user_scale(data_units);
            let mut nanos = Vec::new();
            let mut missing_axes = Vec::new();
            let usable = usable_rounds(timing, BASE_POINT_NAME, state, "overwrite");
            for round in &usable {
                let (value, missing) = user_scale_nanos(counts, timing, state, *round, &scale);
                nanos.push(value);
                missing_axes = missing;
            }
            let cell = (usable.len() as u64 == ROUNDS).then(|| one_millisecond_cell(&nanos));
            let missing_data_or_files =
                missing_axes.iter().any(|axis| *axis == "D" || *axis == "F");
            reference_only |= missing_data_or_files;
            let _ = writeln!(
                out,
                "name=q4 state={state} data_units_user={data_units:.0} valid_rounds={} cell={} nanos_of_each_round={} axes_not_extrapolated={}",
                usable.len(),
                cell.map_or("fewer_than_five_valid_rounds".to_string(), |one_cell| format!("{one_cell:?}")),
                join_numbers(&nanos.iter().map(|value| format!("{value:.0}")).collect::<Vec<_>>()),
                missing_axes.join(",")
            );
            if let Some(cell) = cell {
                cells.push(cell);
            }
        }
    }
    (cells, reference_only)
}

/// 第二段的对角点 X1 = (2²⁰, 10⁴, 10⁴, 1000)（登记第五节取样点）：Q5 另拿 B0 与各轴推它。
const DIAGONAL_POINT: PointAxes = PointAxes {
    unit_area_slots: 1 << 20,
    unlinks: 10_000,
    data_units: 10_000,
    files: 1000,
};

/// Q5 一格的判定行：五轮挂钟比都量到、每一轮都满足 [`extrapolation_is_checked`] 才算核过。交回核过没有。
fn extrapolation_cell_line(
    out: &mut String,
    axis: &str,
    state: &str,
    predicted_steps: f64,
    measured_steps: f64,
    ratios: &[f64],
) -> bool {
    let passed = ratios.len() as u64 == ROUNDS
        && ratios
            .iter()
            .all(|ratio| extrapolation_is_checked(predicted_steps, measured_steps, *ratio));
    let _ = writeln!(
        out,
        "name=q5 axis={axis} state={state} predicted_steps={predicted_steps:.0} measured_steps={measured_steps:.0} nanos_ratios={} passed={passed}",
        join_numbers(&ratios.iter().map(|ratio| format!("{ratio:.3}")).collect::<Vec<_>>())
    );
    passed
}

/// Q5 对角点那一格：B0 加各轴从 B0 到 X1 那一轴取值的增量（同 Q4 的外推法），与 X1 量的比。
/// 产物里没有 X1、或有一根轴没有两个点可外推，这一格不出（不记核过）。
fn diagonal_extrapolation_cell(
    counts: &Product,
    timing: &Product,
    state: &str,
    out: &mut String,
) -> Option<bool> {
    let diagonal = counts
        .points
        .iter()
        .find(|(_, point_axes)| **point_axes == DIAGONAL_POINT)
        .map(|(name, _)| name)?;
    let scale = UserScale {
        unit_area_slots: to_f64(DIAGONAL_POINT.unit_area_slots),
        unlinks: to_f64(DIAGONAL_POINT.unlinks),
        data_units: to_f64(DIAGONAL_POINT.data_units),
        files: to_f64(DIAGONAL_POINT.files),
    };
    let steps = |at: &RoundKey| to_f64(scan_steps_at(counts, at, &SEGMENTS_BEFORE_ADMISSION));
    let (predicted_steps, missing) =
        user_scale_value(counts, state, "overwrite", 1, &steps, &scale);
    if !missing.is_empty() {
        return None;
    }
    let measured_steps = to_f64(scan_steps(
        counts,
        diagonal,
        state,
        "overwrite",
        &SEGMENTS_BEFORE_ADMISSION,
    ));
    let nanos = |at: &RoundKey| to_f64(scan_nanos(timing, at));
    let mut ratios = Vec::new();
    for round in 1..=ROUNDS {
        let (predicted, _) = user_scale_value(timing, state, "overwrite", round, &nanos, &scale);
        let measured = to_f64(scan_nanos(
            timing,
            &round_key(diagonal, state, "overwrite", round),
        ));
        if measured > 0.0 {
            ratios.push(predicted / measured);
        }
    }
    Some(extrapolation_cell_line(
        out,
        "diagonal",
        state,
        predicted_steps,
        measured_steps,
        &ratios,
    ))
}

/// Q5 外推核对：每根轴从更小的点推这根轴上最大的那个点（U 轴拿 2¹⁴、2¹⁷、2²⁰ 推 2²⁴，u、D、F 三轴拿 ≤ 10³ 推 10⁴），
/// 第二段另拿 B0 与各轴推对角点 X1；步数相对差 ≤ 10%、挂钟比在 [0.5, 2]。交回每格是否核过。
/// 一根轴上不足三个点的那一格不出（第一段只有 U、u 两根轴）。
pub(super) fn extrapolation_check_report(counts: &Product, timing: &Product, out: &mut String) -> Vec<bool> {
    let mut checks = Vec::new();
    for state in STATES {
        for axis in AXES {
            let points = axis_points_for_alpha(counts, axis);
            let Some((target, smaller)) = points.split_last() else {
                continue;
            };
            if smaller.len() < 2 {
                continue;
            }
            let steps_series: Vec<(f64, f64)> = smaller
                .iter()
                .map(|(point, axes)| {
                    (
                        to_f64(axis_value(axis, axes)),
                        to_f64(scan_steps(
                            counts,
                            point,
                            state,
                            "overwrite",
                            &SEGMENTS_BEFORE_ADMISSION,
                        )),
                    )
                })
                .collect();
            let predicted_steps =
                value_on_axis(&steps_series, to_f64(axis_value(axis, target.1))).unwrap_or(0.0);
            let measured_steps = to_f64(scan_steps(
                counts,
                target.0,
                state,
                "overwrite",
                &SEGMENTS_BEFORE_ADMISSION,
            ));
            let mut ratios = Vec::new();
            for round in 1..=ROUNDS {
                let target_at = round_key(target.0, state, "overwrite", round);
                let nanos_series: Vec<(f64, f64)> = smaller
                    .iter()
                    .map(|(point, axes)| {
                        (
                            to_f64(axis_value(axis, axes)),
                            to_f64(scan_nanos(
                                timing,
                                &round_key(point, state, "overwrite", round),
                            )),
                        )
                    })
                    .collect();
                let predicted =
                    value_on_axis(&nanos_series, to_f64(axis_value(axis, target.1))).unwrap_or(0.0);
                let measured = to_f64(scan_nanos(timing, &target_at));
                if measured > 0.0 {
                    ratios.push(predicted / measured);
                }
            }
            checks.push(extrapolation_cell_line(
                out,
                axis,
                state,
                predicted_steps,
                measured_steps,
                &ratios,
            ));
        }
        if let Some(passed) = diagonal_extrapolation_cell(counts, timing, state, out) {
            checks.push(passed);
        }
    }
    checks
}

#[cfg(test)]
#[path = "user_scale_and_extrapolation_tests.rs"]
mod tests;
