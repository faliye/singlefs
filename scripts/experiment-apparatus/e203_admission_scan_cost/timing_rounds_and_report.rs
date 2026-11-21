//! 计时的轮、独占纳秒、轮的档与计时报告。

use e7_index_bench::usize_from_u64;
use super::*;

// ---- 计时：Q3、Q10 与包含自检 ----
pub(super) const BASE_POINT_NAME: &str = "base";

/// 一轮里某一段所有行（含清单外那一行）的独占计时之和。
pub(super) fn exclusive_nanos_in_the_segment(timing: &Product, at: &RoundKey, segment: &str) -> u64 {
    let listed: u64 = all_rows()
        .iter()
        .map(|(row, _)| timing.nanos_of(at, &[segment], row))
        .sum();
    listed + timing.nanos_of(at, &[segment], UNLISTED_ROW)
}

/// 包含自检（作废条款第 5 条）：这一轮三段里每段 Σ 独占计时都不大于段挂钟；一段没有挂钟时它的独占计时也得是 0。
pub(super) fn round_is_usable(timing: &Product, at: &RoundKey) -> bool {
    if !timing.rows_exist(at) {
        return false;
    }
    SEGMENTS_BEFORE_ADMISSION.iter().all(|segment| {
        let exclusive = exclusive_nanos_in_the_segment(timing, at, segment);
        match timing.wall_of(at, segment) {
            Some(wall) => round_is_valid(exclusive, wall),
            None => exclusive == 0,
        }
    })
}

pub(super) fn usable_rounds(timing: &Product, point: &str, state: &str, kind: &str) -> Vec<u64> {
    (1..=ROUNDS)
        .filter(|round| round_is_usable(timing, &round_key(point, state, kind, *round)))
        .collect()
}

/// 一轮里扫类各行在准入之前三段的独占计时之和。
pub(super) fn scan_nanos(timing: &Product, at: &RoundKey) -> u64 {
    scan_rows()
        .iter()
        .map(|row| timing.nanos_of(at, &SEGMENTS_BEFORE_ADMISSION, row))
        .sum()
}

pub(super) fn join_numbers<Item: std::fmt::Display>(values: &[Item]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

/// 一根轴上五轮每轮一个 α：两端都有效的轮才算；不足五轮记 `fewer_than_five_valid_rounds`，五轮不同档记 `unstable`。
fn rounds_band(
    timing: &Product,
    axis: &str,
    state: &str,
    kind: &str,
    quantity: &dyn Fn(&RoundKey) -> u64,
) -> Option<(Vec<f64>, String)> {
    let points = axis_points_for_alpha(timing, axis);
    let (low, high) = (points.first()?, points.last()?);
    if low.0 == high.0 {
        return None;
    }
    let mut alphas = Vec::new();
    let mut valid = 0;
    for round in 1..=ROUNDS {
        let (low_at, high_at) = (
            round_key(low.0, state, kind, round),
            round_key(high.0, state, kind, round),
        );
        if !(round_is_usable(timing, &low_at) && round_is_usable(timing, &high_at)) {
            continue;
        }
        valid += 1;
        let (low_value, high_value) = (quantity(&low_at), quantity(&high_at));
        if low_value > 0 && high_value > 0 {
            alphas.push(alpha(
                to_f64(low_value),
                to_f64(high_value),
                to_f64(axis_value(axis, low.1)),
                to_f64(axis_value(axis, high.1)),
            ));
        }
    }
    let verdict = if valid < ROUNDS || alphas.len() < usize_from_u64(valid) {
        if valid < ROUNDS {
            "fewer_than_five_valid_rounds".to_string()
        } else {
            "one_end_is_zero".to_string()
        }
    } else {
        band_of_the_rounds(&alphas)
            .map_or("unstable".to_string(), |one_band| format!("{one_band:?}"))
    };
    Some((alphas, verdict))
}

/// 计时产物里出现过的点名（行、段挂钟、完成标记）。
pub(super) fn point_names_of_the_timing(timing: &Product) -> BTreeSet<String> {
    timing
        .rows
        .keys()
        .map(|key| key.0.clone())
        .chain(timing.segment_nanos.keys().map(|key| key.0.clone()))
        .chain(timing.items_done.iter().cloned())
        .filter(|point| !point.is_empty())
        .collect()
}

/// 计时产物不打 `name=point` 行（登记第十二节修订 7）：每个点的轴值取同名计数产物里那一行。
/// 两边都没有的点交回读不到：它进不了任何一根轴，Q3、Q10 报读不到，F3 不判。
pub(super) fn timing_with_the_point_axes_of_the_counts(
    counts: &Product,
    timing: &Product,
) -> (Product, Vec<String>) {
    let mut with_axes = timing.clone();
    let mut unread = Vec::new();
    for point in point_names_of_the_timing(timing) {
        if with_axes.points.contains_key(&point) {
            continue;
        }
        match counts.points.get(&point) {
            Some(axes) => {
                with_axes.points.insert(point, *axes);
            }
            None => unread.push(point),
        }
    }
    (with_axes, unread)
}

/// Q3 与 Q10。交回 F3 是否触发：某一段的残差在某根轴上五轮都 α ≥ 0.5 记触发；
/// 没触发而有点读不到轴值、或一根轴都没算成的，记不判（`None`），不当没触发。
pub(super) fn timing_report(
    timing: &Product,
    points_with_the_axes_unread: &[String],
    out: &mut String,
) -> Option<bool> {
    let mut residual_grows = false;
    let mut judged_any_axis = false;
    for point in points_with_the_axes_unread {
        let _ = writeln!(out, "name=q3 point={point} result=axes_unread");
        let _ = writeln!(out, "name=q10 point={point} result=axes_unread");
    }
    for state in STATES {
        for kind in KINDS {
            for axis in AXES {
                let scan = |at: &RoundKey| scan_nanos(timing, at);
                if let Some((alphas, verdict)) = rounds_band(timing, axis, state, kind, &scan) {
                    let _ = writeln!(
                        out,
                        "name=q3 axis={axis} state={state} kind={kind} alphas={} band={verdict}",
                        join_numbers(
                            &alphas
                                .iter()
                                .map(|growth| format!("{growth:.6}"))
                                .collect::<Vec<_>>()
                        )
                    );
                }
                for segment in SEGMENTS_BEFORE_ADMISSION {
                    let residual = |at: &RoundKey| {
                        let wall = timing.wall_of(at, segment).unwrap_or(0);
                        wall.saturating_sub(exclusive_nanos_in_the_segment(timing, at, segment))
                    };
                    if let Some((alphas, verdict)) =
                        rounds_band(timing, axis, state, kind, &residual)
                    {
                        judged_any_axis = true;
                        if verdict == "GrowsWithTheAxis" {
                            residual_grows = true;
                        }
                        let _ = writeln!(out, "name=q10 axis={axis} state={state} kind={kind} segment={segment} alphas={} band={verdict}", join_numbers(&alphas.iter().map(|growth| format!("{growth:.6}")).collect::<Vec<_>>()));
                    }
                }
            }
        }
    }
    // 残差占段挂钟的比（高端那一点第 1 轮，每段）
    for state in STATES {
        for point in timing.points.keys() {
            let at = round_key(point, state, "overwrite", 1);
            for segment in SEGMENTS_BEFORE_ADMISSION {
                if let Some(wall) = timing.wall_of(&at, segment) {
                    let residual =
                        wall.saturating_sub(exclusive_nanos_in_the_segment(timing, &at, segment));
                    let _ = writeln!(out, "name=q10_share point={point} state={state} segment={segment} residual_nanos={residual} segment_nanos={wall}");
                }
            }
        }
    }
    if residual_grows {
        Some(true)
    } else if !judged_any_axis || !points_with_the_axes_unread.is_empty() {
        None
    } else {
        Some(false)
    }
}

#[cfg(test)]
#[path = "timing_rounds_and_report_tests.rs"]
mod tests;
