//! 从产物取逐项按差量的输入、池化退路读数、按段的行与总数、汇总。

use super::*;

// ---- B8、B4 的输入：从计数产物里读（读不到的列进缺口，不当 0）----
/// B8 式子里 `c_x` 那几行（登记第五节表里写调用次数的行）。
const CALLED_ROWS: [&str; 18] = [
    "inode_record_linear_find",
    "files_other_than_the_first_plan",
    "file_other_than_the_first_linear_find",
    "inodes_with_an_extent_entry_set",
    "extent_upper_segment_plan",
    "extent_roles_rewritten_scan",
    "root_header_linear_find",
    "checkpoint_cost_accounting_tree_plan",
    "cluster_segment_sum",
    "inode_number_watermark_steps",
    "allocator_release_linear_find",
    "user_data_slot_linear_scan",
    "record_intersection_filter",
    "empty_segment_linear_scan",
    "fallback_slot_linear_scan",
    "accounting_entries_of_this_publish",
    "instance_table_chain",
    // s37 没有 B8 式子，只读调用次数 pooled_fallback_calls 好标「缺 s37 一行」
    ROW_WITHOUT_A_BY_DELTA_FORMULA,
];

/// 固定点每一轮的某个差量读数（一轮取读到的最大那个）；某一轮一条都没有记缺口。
fn per_round_maxima(
    product: &Product,
    at: &RoundKey,
    key: &str,
    rounds: u64,
    gaps: &mut Vec<String>,
) -> Vec<u64> {
    let readings = product.deltas_of(at, key, &["between"]);
    (1..=rounds)
        .map(|round| {
            let best = readings
                .iter()
                .filter(|(fixed_point_round, _)| *fixed_point_round == round)
                .map(|(_, value)| *value)
                .max();
            best.unwrap_or_else(|| {
                gaps.push(format!("{key}@round{round}"));
                0
            })
        })
        .collect()
}

/// 一个点某状态某发布（第 1 轮）在点名几段里的 B8 输入。缺口列出读不到的读数名。
/// 与登记第七节 ③3 的 Δ0 同一套字段；每个字段怎么从产物里读，见本函数里逐行的注释。
pub(super) fn by_delta_input_from_the_product(
    product: &Product,
    point: &str,
    state: &str,
    kind: &str,
    segments: &[&str],
) -> Result<ByDeltaInput, Vec<String>> {
    let at = round_key(point, state, kind, 1);
    let Some(axes) = product.points.get(point) else {
        return Err(vec![format!("point@{point}")]);
    };
    let mut gaps: Vec<String> = Vec::new();
    let mut state_field = |key: &str| -> u64 {
        product.state_value(&at, key).unwrap_or_else(|| {
            gaps.push(format!("state.{key}"));
            0
        })
    };
    let inode_leaf_containers = state_field("inode_leaf_containers");
    let other_files = state_field("other_files");
    let allocation_records = state_field("allocation_records");
    let units = state_field("units");
    let mapped_units = state_field("mapped_units");
    let deadlist_entries = state_field("deadlist_entries");
    let extent_upper_nodes = state_field("extent_upper_nodes");
    let extent_upper_levels = state_field("extent_upper_levels");
    let accounting_tree_levels = state_field("accounting_tree_levels");
    let deadlist_tree_levels = state_field("deadlist_tree_levels");
    let allocation_tree_levels = state_field("allocation_tree_levels");
    let central_mapping_levels = state_field("central_mapping_levels");
    let between = segments.contains(&"between");
    let rounds = if between {
        product
            .publishes
            .get(&at)
            .and_then(|fields| number(fields, "fixed_point_rounds"))
            .unwrap_or_else(|| {
                gaps.push("publish.fixed_point_rounds".to_string());
                0
            })
    } else {
        0
    };
    let calls: BTreeMap<&'static str, u64> = CALLED_ROWS
        .iter()
        .map(|row| (*row, product.calls_of(&at, segments, row)))
        .collect();
    let called = |row: &str| calls.get(row).copied().unwrap_or(0) > 0;
    // 这一次发布本身的量（不分段）：取三段里读到的最大那个；该有而没有的记缺口。
    let publish_level = |key: &str, needed: bool, gap_lines: &mut Vec<String>| -> u64 {
        let best = product
            .deltas_of(&at, key, &SEGMENTS_BEFORE_ADMISSION)
            .into_iter()
            .map(|(_, value)| value)
            .max();
        best.unwrap_or_else(|| {
            if needed {
                gap_lines.push(format!("delta.{key}"));
            }
            0
        })
    };
    let touched_files = publish_level(
        "touched_files",
        called("files_other_than_the_first_plan"),
        &mut gaps,
    );
    let lower_nodes_of_touched_files = publish_level(
        "lower_nodes_of_touched_files",
        called("files_other_than_the_first_plan"),
        &mut gaps,
    );
    let accounting_rows = publish_level(
        "accounting_rows_written_by_the_built_on_publish",
        called("checkpoint_cost_accounting_tree_plan")
            || called("accounting_entries_of_this_publish"),
        &mut gaps,
    );
    let deadlist_changed_entries = publish_level("deadlist_entries_changed", between, &mut gaps);
    let moved_entries = publish_level("moved_entries", between, &mut gaps);
    let intersecting_records: u64 = product
        .deltas_of(&at, "intersecting_records", segments)
        .iter()
        .map(|(_, value)| value)
        .sum();
    let changed_leaves_per_round =
        per_round_maxima(product, &at, "changed_leaves", rounds, &mut gaps);
    let released_per_round =
        per_round_maxima(product, &at, "round_placements_released", rounds, &mut gaps);
    let rewritten_per_round =
        per_round_maxima(product, &at, "round_roles_rewritten", rounds, &mut gaps);
    let changed_slots_and_records_per_round: Vec<u64> = released_per_round
        .iter()
        .zip(&rewritten_per_round)
        .map(|(released, rewritten)| released + rewritten)
        .collect();
    let dirty_per_call: Vec<u64> = product
        .deltas_of(&at, "dirty_records", segments)
        .into_iter()
        .map(|(_, value)| value)
        .collect();
    let keys_changed = product.deltas_of(&at, "code_two_keys_changed", segments);
    if keys_changed.len() as u64 != 3 * rounds {
        gaps.push(format!(
            "delta.code_two_keys_changed:{}_for_{}_rounds",
            keys_changed.len(),
            rounds
        ));
    }
    let heights = [
        accounting_tree_levels,
        deadlist_tree_levels,
        central_mapping_levels,
    ];
    let code_two_trees_over_all_rounds: Vec<(u64, u64)> = keys_changed
        .iter()
        .enumerate()
        .map(|(index, (_, value))| (*value, heights[index % 3]))
        .collect();
    let container_rewrites_over_all_rounds: Vec<u64> = product
        .deltas_of(&at, "container_units_rewritten", segments)
        .into_iter()
        .map(|(_, value)| value)
        .collect();
    let release_query_levels =
        product.query_levels_of(&at, segments, "release_judgement_version_indexes");
    let deadlist_query_levels = product.query_levels_of(&at, segments, "deadlist_changes_indexes");
    let mapped_units_changed_over_all_rounds = 2 * rewritten_per_round.iter().sum::<u64>();
    let mapped_units_query_levels =
        mapped_units_queries_by_delta(mapped_units_changed_over_all_rounds, mapped_units);
    if !gaps.is_empty() {
        return Err(gaps);
    }
    Ok(ByDeltaInput {
        inode_leaf_containers,
        other_files,
        touched_files,
        lower_nodes_of_touched_files,
        extent_upper_levels,
        extent_upper_nodes,
        accounting_rows_of_the_generation: accounting_rows,
        accounting_tree_levels,
        settling_rounds: rounds,
        changed_leaves_per_round,
        allocation_tree_levels,
        allocation_records,
        segments_per_disk: segments_of(axes.unit_area_slots),
        units,
        mapped_units,
        deadlist_entries,
        calls,
        intersecting_records,
        dirty_per_call,
        code_two_trees_over_all_rounds,
        container_rewrites_over_all_rounds,
        moved_entries,
        central_mapping_levels,
        deadlist_query_levels,
        deadlist_changed_entries,
        release_query_levels,
        changed_slots_and_records_per_round,
        mapped_units_changed_over_all_rounds,
        mapped_units_query_levels,
    })
}

/// s36 在 B8 下按角色查的那一项（登记第五节 s36 一行「`mapped_units` 整表换成差量，按角色查走随一版维护的索引」，第十二节修订 7）：
/// 要问的只有这一轮进出 `mapped_units` 的角色，每个查一次上一版 M 条的按角色索引，记 ⌈log₂(M + 1)⌉ 步。
/// B1 照抄每个角色逐个查的次数（产物里这一行的 `query_levels`）是整表的量，不进 B8。
fn mapped_units_queries_by_delta(changed_roles_over_all_rounds: u64, mapped_units: u64) -> u64 {
    changed_roles_over_all_rounds * lg(mapped_units)
}

// ---- s37（池级回落按叶取落点）：B8、B4 没有这一行的式子，按调用次数标点 ----
/// 一个点某状态某发布：准入之前三段里 s37 的调用次数 pooled_fallback_calls、s37 的步数、s29 的步数（B1 一侧，读不到的行记 0）。
fn pooled_fallback_reading(
    counts: &Product,
    point: &str,
    state: &str,
    kind: &str,
) -> (u64, u64, u64) {
    let at = round_key(point, state, kind, 1);
    (
        counts.calls_of(&at, &SEGMENTS_BEFORE_ADMISSION, ROW_WITHOUT_A_BY_DELTA_FORMULA),
        counts.steps_of(&at, &SEGMENTS_BEFORE_ADMISSION, ROW_WITHOUT_A_BY_DELTA_FORMULA),
        counts.steps_of(&at, &SEGMENTS_BEFORE_ADMISSION, "fallback_slot_linear_scan"),
    )
}

/// 每个取样点、每种发布报 pooled_fallback_calls；pooled_fallback_calls = 0 的点 B8、B4 合计照常，pooled_fallback_calls > 0 的点标「缺 s37 一行，只对清单内的步」，逐点列出 s37 与 s29 的步数。
/// 交回 (pooled_fallback_calls > 0 的格数, 核了几格)。
pub(super) fn pooled_fallback_report(counts: &Product, out: &mut String) -> (u64, u64) {
    let (mut marked, mut checked) = (0, 0);
    for point in counts.points.keys() {
        for state in STATES {
            for kind in KINDS {
                if !counts.states.contains_key(&round_key(point, state, kind, 1)) {
                    continue;
                }
                checked += 1;
                let (calls, steps, per_disk_fallback_steps) = pooled_fallback_reading(counts, point, state, kind);
                let by_delta_total = match by_delta_rows_by_segment(counts, point, state, kind) {
                    Ok(rows) => total_of(&rows, &SEGMENTS_BEFORE_ADMISSION).to_string(),
                    Err(_) => "unread".to_string(),
                };
                let mark = if calls > 0 {
                    marked += 1;
                    "missing_pooled_fallback_row_only_for_the_listed_steps"
                } else {
                    "complete"
                };
                let _ = writeln!(
                    out,
                    "name=by_delta_pooled_fallback point={point} state={state} kind={kind} pooled_fallback_calls={calls} pooled_fallback_steps={steps} per_disk_fallback_steps={per_disk_fallback_steps} by_delta_total={by_delta_total} by_delta_arms_total={mark}"
                );
            }
        }
    }
    let _ = writeln!(
        out,
        "name=verdict quantity=by_delta_pooled_fallback cells_checked={checked} cells_with_pooled_fallback_calls={marked}"
    );
    (marked, checked)
}

// ---- Q6、Q7、Q8、Q9：B8、B4 的式子在量过的点上 ----
/// 一个点某状态某发布：三段各自的 B8 每行步数。读不到输入的交回缺口。
pub(super) fn by_delta_rows_by_segment(
    counts: &Product,
    point: &str,
    state: &str,
    kind: &str,
) -> Result<StepsByRow, Vec<String>> {
    let mut by_row = StepsByRow::new();
    for segment in SEGMENTS_BEFORE_ADMISSION {
        let input = by_delta_input_from_the_product(counts, point, state, kind, &[segment])?;
        for (row, steps) in by_delta_rows(&input) {
            by_row.insert((segment, row), steps);
        }
    }
    Ok(by_row)
}

pub(super) fn today_rows_by_segment(counts: &Product, point: &str, state: &str, kind: &str) -> StepsByRow {
    let at = round_key(point, state, kind, 1);
    let mut by_row = StepsByRow::new();
    for segment in SEGMENTS_BEFORE_ADMISSION {
        for row in scan_rows() {
            by_row.insert((segment, row), counts.steps_of(&at, &[segment], row));
        }
    }
    by_row
}

pub(super) fn total_of(by_row: &StepsByRow, segments: &[&str]) -> u64 {
    by_row
        .iter()
        .filter(|((segment, _), _)| segments.contains(segment))
        .map(|(_, steps)| steps)
        .sum()
}

/// 在 P0 或 S1 里调到过的行（B4：改的是函数，两道之间再调到也按 B8）。
pub(super) fn rows_called_before_the_second_judgement(
    counts: &Product,
    point: &str,
    state: &str,
    kind: &str,
) -> BTreeSet<&'static str> {
    let at = round_key(point, state, kind, 1);
    scan_rows()
        .into_iter()
        .filter(|row| counts.calls_of(&at, &SEGMENTS_BEFORE_THE_SECOND_JUDGEMENT, row) > 0)
        .collect()
}

pub(super) fn alpha_text_for_points(
    counts: &Product,
    axis: &str,
    quantity: &dyn Fn(&str) -> Option<u64>,
) -> AxisReading {
    let points = axis_points_for_alpha(counts, axis);
    let (Some(low), Some(high)) = (points.first(), points.last()) else {
        return AxisReading::NoEnds;
    };
    if low.0 == high.0 {
        return AxisReading::NoEnds;
    }
    let (Some(low_value), Some(high_value)) = (quantity(low.0), quantity(high.0)) else {
        return AxisReading::NoEnds;
    };
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

/// 三档里最坏的一档：任一格「随」就是「随」，全部「不随」才是「不随」，否则「介于」；有格读不到记「读不到」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Summary {
    GrowsOnSomeAxis,
    DoesNotGrowOnAnyAxis,
    BetweenOnSomeAxis,
    SomeInputUnread,
}

pub(super) fn summarize(bands: &[Option<Band>], any_unread: bool) -> Summary {
    if bands.contains(&Some(Band::GrowsWithTheAxis))
    {
        Summary::GrowsOnSomeAxis
    } else if any_unread || bands.is_empty() {
        Summary::SomeInputUnread
    } else if bands
        .iter()
        .all(|one_band| *one_band == Some(Band::DoesNotGrow))
    {
        Summary::DoesNotGrowOnAnyAxis
    } else {
        Summary::BetweenOnSomeAxis
    }
}

#[cfg(test)]
#[path = "by_delta_input_and_segment_totals_tests.rs"]
mod tests;
