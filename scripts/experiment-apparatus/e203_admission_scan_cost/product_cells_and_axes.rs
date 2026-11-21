//! 产物的格与点、轴的读法与取点、轴的值。

use super::*;

impl Product {
    pub(super) fn row_cell(
        &self,
        point: &str,
        state: &str,
        kind: &str,
        round: u64,
        segment: &str,
        row: &str,
    ) -> RowCell {
        self.rows
            .get(&(
                point.to_string(),
                state.to_string(),
                kind.to_string(),
                round,
                segment.to_string(),
                row.to_string(),
            ))
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn steps_of(&self, at: &RoundKey, segments: &[&str], row: &str) -> u64 {
        segments
            .iter()
            .map(|segment| self.row_cell(&at.0, &at.1, &at.2, at.3, segment, row).steps)
            .sum()
    }

    pub(super) fn calls_of(&self, at: &RoundKey, segments: &[&str], row: &str) -> u64 {
        segments
            .iter()
            .map(|segment| self.row_cell(&at.0, &at.1, &at.2, at.3, segment, row).calls)
            .sum()
    }

    pub(super) fn nanos_of(&self, at: &RoundKey, segments: &[&str], row: &str) -> u64 {
        segments
            .iter()
            .map(|segment| {
                self.row_cell(&at.0, &at.1, &at.2, at.3, segment, row)
                    .exclusive_nanos
            })
            .sum()
    }

    pub(super) fn query_levels_of(&self, at: &RoundKey, segments: &[&str], row: &str) -> u64 {
        segments
            .iter()
            .map(|segment| {
                self.row_cell(&at.0, &at.1, &at.2, at.3, segment, row)
                    .query_levels
            })
            .sum()
    }

    pub(super) fn wall_of(&self, at: &RoundKey, segment: &str) -> Option<u64> {
        self.segment_nanos
            .get(&(
                at.0.clone(),
                at.1.clone(),
                at.2.clone(),
                at.3,
                segment.to_string(),
            ))
            .copied()
    }

    pub(super) fn state_value(&self, at: &RoundKey, key: &str) -> Option<u64> {
        number(self.states.get(at)?, key)
    }

    /// 这一轮这一发布里 `key` 的差量读数：(固定点第几轮, 值)，只取点名的几段。
    pub(super) fn deltas_of(&self, at: &RoundKey, key: &str, segments: &[&str]) -> Vec<(u64, u64)> {
        self.deltas
            .iter()
            .filter(|fields| {
                fields.get("point") == Some(&at.0)
                    && fields.get("state") == Some(&at.1)
                    && fields.get("kind") == Some(&at.2)
                    && number(fields, "round") == Some(at.3)
                    && fields.get("key").map(String::as_str) == Some(key)
                    && fields
                        .get("segment")
                        .is_some_and(|segment| segments.contains(&segment.as_str()))
            })
            .map(|fields| {
                (
                    number(fields, "fixed_point_round").unwrap_or(0),
                    number(fields, "value").unwrap_or(0),
                )
            })
            .collect()
    }

    /// 这一轮这一发布有没有进过 `caller` 段（经调用方进门）。
    pub(super) fn had_caller_segment(&self, at: &RoundKey) -> Option<bool> {
        Some(self.publishes.get(at)?.get("had_caller_segment")? == "true")
    }

    pub(super) fn rows_exist(&self, at: &RoundKey) -> bool {
        self.rows
            .keys()
            .any(|key| key.0 == at.0 && key.1 == at.1 && key.2 == at.2 && key.3 == at.3)
    }
}

/// 作废条款 8：被量的覆盖写判了不止一次、补满之后取到的轮数仍不足五轮，那个点那种状态的覆盖写那一格量不到。
/// 交回 (点, 状态, 取到几轮, 尝试几次)。
pub(super) fn cells_not_measurable(product: &Product) -> Vec<(String, String, u64, u64)> {
    product
        .rounds_taken
        .iter()
        .filter(|(_, (taken, _))| *taken < ROUNDS)
        .map(|((point, state), (taken, attempts))| {
            (point.clone(), state.clone(), *taken, *attempts)
        })
        .collect()
}

/// 作废条款（冻结的发布）：被量的发布进判之前读一次分配器里冻结着等重发的那一次是不是空（规模读数行的 `frozen_publish_set`），
/// 不空的那一格所在的点整点作废。交回 (作废的点, 这一份产物里没带这个字段的格数)。
pub(super) fn points_with_a_frozen_publish(product: &Product) -> (BTreeSet<String>, u64) {
    let mut voided = BTreeSet::new();
    let mut unread = 0;
    for (key, fields) in &product.states {
        match number(fields, "frozen_publish_set") {
            Some(0) => {}
            Some(_) => {
                voided.insert(key.0.clone());
            }
            None => unread += 1,
        }
    }
    (voided, unread)
}

/// 把整点拿掉：点轴值、各行读数、规模读数、发布行、差量、段挂钟与各点的附带读数一起拿。
pub(super) fn without_the_points(product: &Product, points: &BTreeSet<String>) -> Product {
    let mut kept = product.clone();
    kept.points.retain(|point, _| !points.contains(point));
    kept.rows.retain(|key, _| !points.contains(&key.0));
    kept.states.retain(|key, _| !points.contains(&key.0));
    kept.publishes.retain(|key, _| !points.contains(&key.0));
    kept.segment_nanos.retain(|key, _| !points.contains(&key.0));
    kept.deltas.retain(|fields| {
        !points.contains(fields.get("point").map_or("", String::as_str))
    });
    kept.unlink_publishes.retain(|key, _| !points.contains(&key.0));
    kept.history_unlinks.retain(|point, _| !points.contains(point));
    kept.rounds_taken.retain(|key, _| !points.contains(&key.0));
    kept.items_done.retain(|point| !points.contains(point));
    kept
}

/// 把量不到的那几格的覆盖写读数整格拿掉（它们不当 0 进任何一个量，别的格不动）：规模读数、各行读数、差量、段挂钟一起拿。
pub(super) fn without_the_cells_not_measurable(
    product: &Product,
    cells: &[(String, String, u64, u64)],
) -> Product {
    let mut kept = product.clone();
    let is_dropped = |point: &str, state: &str, kind: &str| {
        kind == "overwrite"
            && cells
                .iter()
                .any(|(cell_point, cell_state, _, _)| cell_point == point && cell_state == state)
    };
    kept.rows
        .retain(|key, _| !is_dropped(&key.0, &key.1, &key.2));
    kept.states
        .retain(|key, _| !is_dropped(&key.0, &key.1, &key.2));
    kept.publishes
        .retain(|key, _| !is_dropped(&key.0, &key.1, &key.2));
    kept.segment_nanos
        .retain(|key, _| !is_dropped(&key.0, &key.1, &key.2));
    kept.deltas.retain(|fields| {
        let text_of = |key: &str| fields.get(key).map_or("", String::as_str).to_string();
        !is_dropped(&text_of("point"), &text_of("state"), &text_of("kind"))
    });
    kept
}

pub(super) fn round_key(point: &str, state: &str, kind: &str, round: u64) -> RoundKey {
    (
        point.to_string(),
        state.to_string(),
        kind.to_string(),
        round,
    )
}

pub(super) fn scan_rows() -> Vec<&'static str> {
    ROW_TABLE
        .iter()
        .filter(|(_, kind)| *kind == RowKind::Scan)
        .map(|(row, _)| *row)
        .collect()
}

/// 一个点某状态某发布某轮里扫类各行在点名几段里的步数之和。
pub(super) fn scan_steps_at(product: &Product, at: &RoundKey, segments: &[&str]) -> u64 {
    scan_rows()
        .iter()
        .map(|row| product.steps_of(at, segments, row))
        .sum()
}

/// 同上，取第 1 轮（计数是确定的）。
pub(super) fn scan_steps(product: &Product, point: &str, state: &str, kind: &str, segments: &[&str]) -> u64 {
    scan_steps_at(product, &round_key(point, state, kind, 1), segments)
}

// ---- 一根轴的两端 ----
/// 增长阶的低端：U 轴取 2¹⁴；u、D、F 三轴从 x = 10 起算（登记读法第 6 条，x = 1 那一点只报数）。
fn axis_lowest_position(axis: &str) -> u64 {
    if axis == "U" {
        1 << 14
    } else {
        10
    }
}

/// 增长阶的高端：U 轴 2²⁴，别的三轴 10⁴；更大的点（第三段、第二段的 X1 等）不进 α。
fn axis_highest_position(axis: &str) -> u64 {
    if axis == "U" {
        1 << 24
    } else {
        10_000
    }
}

/// 这根轴上在 α 范围里的点，按轴值升序。
pub(super) fn axis_points_for_alpha<'borrowed>(
    product: &'borrowed Product,
    axis: &str,
) -> Vec<(&'borrowed String, &'borrowed PointAxes)> {
    axis_points(product, axis)
        .into_iter()
        .filter(|(_, point_axes)| {
            (axis_lowest_position(axis)..=axis_highest_position(axis))
                .contains(&axis_value(axis, point_axes))
        })
        .collect()
}

/// 这根轴上外推要用的点，按轴值升序：低端同 α（x = 1 那一点不用）。U 轴高端不封顶——第三段量了 X2（每盘 16 TiB 的单元区）之后，
/// Q4 在 U 那一部分直接取 X2 的量（登记第十节 F6「X2 量过之后 Q4 在 U 那一部分改用 X2 的量」）；记录数那三根轴封在登记取样点表的 10⁴：
/// Y1 的单元区加大到 2¹⁷ 那一档时它是 (2¹⁷, 10⁵, 10, 10)，四根轴里三根取 B0，会被当成 u 轴上的一点，封了它才不进 u 轴的外推（登记 u 轴只取到 10⁴）。
pub(super) fn axis_points_for_extrapolation<'borrowed>(
    product: &'borrowed Product,
    axis: &str,
) -> Vec<(&'borrowed String, &'borrowed PointAxes)> {
    axis_points(product, axis)
        .into_iter()
        .filter(|(_, point_axes)| {
            let position = axis_value(axis, point_axes);
            position >= axis_lowest_position(axis)
                && (axis == "U" || position <= axis_highest_position(axis))
        })
        .collect()
}

/// 一个量在一根轴上的读数。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum AxisReading {
    /// 这根轴上没有两个点。
    NoEnds,
    /// 有一端是 0：不算 α，写「一端为 0」。
    OneEndIsZero(u64, u64),
    Alpha(u64, u64, f64),
}

pub(super) fn read_axis(product: &Product, axis: &str, quantity: &dyn Fn(&str) -> u64) -> AxisReading {
    let points = axis_points_for_alpha(product, axis);
    let (Some(low), Some(high)) = (points.first(), points.last()) else {
        return AxisReading::NoEnds;
    };
    if low.0 == high.0 {
        return AxisReading::NoEnds;
    }
    let (low_value, high_value) = (quantity(low.0), quantity(high.0));
    if low_value == 0 || high_value == 0 {
        return AxisReading::OneEndIsZero(low_value, high_value);
    }
    AxisReading::Alpha(
        low_value,
        high_value,
        alpha(
            to_f64(low_value),
            to_f64(high_value),
            to_f64(axis_value(axis, low.1)),
            to_f64(axis_value(axis, high.1)),
        ),
    )
}

pub(super) fn reading_text(reading: &AxisReading) -> String {
    match reading {
        AxisReading::NoEnds => "low_end=NA high_end=NA alpha=no_two_points".to_string(),
        AxisReading::OneEndIsZero(low, high) => {
            format!("low_end={low} high_end={high} alpha=one_end_is_zero")
        }
        AxisReading::Alpha(low, high, growth) => {
            format!(
                "low_end={low} high_end={high} alpha={growth:.6} band={:?}",
                band(*growth)
            )
        }
    }
}

pub(super) fn reading_band(reading: &AxisReading) -> Option<Band> {
    match reading {
        AxisReading::Alpha(_, _, growth) => Some(band(*growth)),
        AxisReading::NoEnds | AxisReading::OneEndIsZero(..) => None,
    }
}

// ---- 报告 ----
/// B0 = (2¹⁷, 10, 10, 10)。
const BASE_POINT: PointAxes = PointAxes {
    unit_area_slots: 1 << 17,
    unlinks: 10,
    data_units: 10,
    files: 10,
};

pub(super) fn axis_points<'borrowed>(
    product: &'borrowed Product,
    axis: &str,
) -> Vec<(&'borrowed String, &'borrowed PointAxes)> {
    let mut points: Vec<(&String, &PointAxes)> = product
        .points
        .iter()
        .filter(|(_, point_axes)| match axis {
            "U" => {
                (point_axes.unlinks, point_axes.data_units, point_axes.files)
                    == (BASE_POINT.unlinks, BASE_POINT.data_units, BASE_POINT.files)
            }
            "u" => {
                (
                    point_axes.unit_area_slots,
                    point_axes.data_units,
                    point_axes.files,
                ) == (
                    BASE_POINT.unit_area_slots,
                    BASE_POINT.data_units,
                    BASE_POINT.files,
                )
            }
            "D" => {
                (
                    point_axes.unit_area_slots,
                    point_axes.unlinks,
                    point_axes.files,
                ) == (
                    BASE_POINT.unit_area_slots,
                    BASE_POINT.unlinks,
                    BASE_POINT.files,
                )
            }
            _ => {
                (
                    point_axes.unit_area_slots,
                    point_axes.unlinks,
                    point_axes.data_units,
                ) == (
                    BASE_POINT.unit_area_slots,
                    BASE_POINT.unlinks,
                    BASE_POINT.data_units,
                )
            }
        })
        .collect();
    points.sort_by_key(|(_, point_axes)| match axis {
        "U" => point_axes.unit_area_slots,
        "u" => point_axes.unlinks,
        "D" => point_axes.data_units,
        _ => point_axes.files,
    });
    points
}

pub(super) fn axis_value(axis: &str, point_axes: &PointAxes) -> u64 {
    match axis {
        "U" => point_axes.unit_area_slots,
        "u" => point_axes.unlinks,
        "D" => point_axes.data_units,
        _ => point_axes.files,
    }
}

#[cfg(test)]
#[path = "product_cells_and_axes_tests.rs"]
mod tests;
