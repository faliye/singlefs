//! 逐百单元序列的读法与报告、头对朴素、放置差距、状态行与对账拆分的报告。

use super::*;

// ---- 复量段 hundred_units_per_file_repeat（登记设计员修订 2）----
/// 复量序列的六个点，总量由低到高；高端 `files_1000_units_100` 读第一段的产物，别的五点（含 `files_100_units_100` 再跑一次）读复量段的产物。
pub(super) const HUNDRED_UNITS_PER_FILE_SERIES: [&str; 6] =
    ["files_56_units_100", "files_100_units_100", "files_178_units_100", "files_316_units_100", "files_562_units_100", "files_1000_units_100"];
/// 公共窗：六点都有的量窗操作（高端的扩展窗口 W5–W7 低端没有，不进公共窗）。
const COMMON_WINDOW_OPERATIONS: [&str; 4] = ["W1", "W2", "W3", "W4"];
/// 序列至少读到几个点才判；少于它记 `not_enough_points`。
const SERIES_POINTS_AT_LEAST: usize = 3;
/// 稳的判语里「随」与「不随」要几段相邻段（五段里）。
const SERIES_ADJACENT_SEGMENTS_AT_LEAST: usize = 3;
/// 阳性对照 `C1`、`C3` 在公共窗的 β 至少多少（读法 4 的「随」）。
const SERIES_CONTROL_BETA_AT_LEAST: f64 = BAND_GROWS_AT_LEAST;

/// 普通最小二乘斜率：ln Q 对 ln x，各点同权；点是 (Q, x)。少于两点或有不是正数的交 `None`。
fn least_squares_slope(points: &[(f64, f64)]) -> Option<f64> {
    if points.len() < 2 || points.iter().any(|(quantity, position)| *quantity <= 0.0 || *position <= 0.0) {
        return None;
    }
    let count = points.len() as f64;
    let mean_position = points.iter().map(|(_, position)| position.ln()).sum::<f64>() / count;
    let mean_quantity = points.iter().map(|(quantity, _)| quantity.ln()).sum::<f64>() / count;
    let covariance: f64 = points.iter().map(|(quantity, position)| (position.ln() - mean_position) * (quantity.ln() - mean_quantity)).sum();
    let variance: f64 = points.iter().map(|(_, position)| (position.ln() - mean_position).powi(2)).sum();
    // 各点的 Q 相同时，均值的舍入误差会让协方差落在 ±1e-16 量级而不是恰为 0；比打印精度（三位小数）小得多的斜率
    // 统一写正零，否则打出来是 -0.000。
    (variance > 0.0).then(|| covariance / variance).map(|slope| if slope.abs() < 1e-9 { 0.0 } else { slope })
}

/// 一条序列的读数：读到的点数、相邻段的 α、β、随 / 介于 / 不随各几段（`unread` 是算不出 α 的段数）与判语。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct SeriesReading {
    pub(super) points: usize,
    alphas: Vec<Option<f64>>,
    pub(super) beta: Option<f64>,
    pub(super) grows: usize,
    pub(super) between: usize,
    pub(super) does_not_grow: usize,
    pub(super) unread: usize,
    pub(super) verdict: &'static str,
}

/// 判语（登记修订 2 的四行加 `not_enough_points`）：`g` = 随的段数、`d` = 不随的段数。
fn series_verdict(points: usize, beta: Option<f64>, grows: usize, does_not_grow: usize) -> &'static str {
    let Some(beta) = beta.filter(|_| points >= SERIES_POINTS_AT_LEAST) else {
        return "not_enough_points";
    };
    if beta >= BAND_GROWS_AT_LEAST && grows >= SERIES_ADJACENT_SEGMENTS_AT_LEAST && does_not_grow == 0 {
        "stable_grows"
    } else if beta <= BAND_DOES_NOT_GROW_AT_MOST && does_not_grow >= SERIES_ADJACENT_SEGMENTS_AT_LEAST && grows == 0 {
        "stable_does_not_grow"
    } else if beta > BAND_DOES_NOT_GROW_AT_MOST && beta < BAND_GROWS_AT_LEAST && grows == 0 && does_not_grow == 0 {
        "stable_between"
    } else {
        "unstable"
    }
}

/// 一列 (Q, x)（x 由低到高）的序列读数。
pub(super) fn series_reading(points: &[(f64, f64)]) -> SeriesReading {
    let alphas = local_slopes(points);
    let beta = least_squares_slope(points);
    let segments_in = |wanted: Band| alphas.iter().flatten().filter(|value| band(**value) == wanted).count();
    let (grows, between, does_not_grow) = (segments_in(Band::Grows), segments_in(Band::Between), segments_in(Band::DoesNotGrow));
    SeriesReading {
        points: points.len(),
        verdict: series_verdict(points.len(), beta, grows, does_not_grow),
        unread: alphas.iter().filter(|value| value.is_none()).count(),
        alphas,
        beta,
        grows,
        between,
        does_not_grow,
    }
}

/// 阳性对照：`C1`、`C3` 在公共窗的 β 都要 ≥ 0.5，不是（或读不到）主格的判语作废。
pub(super) fn controls_are_red(scan_beta: Option<f64>, linear_beta: Option<f64>) -> bool {
    scan_beta.zip(linear_beta).is_some_and(|(scan, linear)| scan >= SERIES_CONTROL_BETA_AT_LEAST && linear >= SERIES_CONTROL_BETA_AT_LEAST)
}

/// 一个点一个状态一档一种取法一条臂取出来的那一端，只在 `common` 为真时丢掉公共窗之外的操作（W5–W7、P1、阴性对照）；读不到交 `None`。
fn end_among(point: &PointReadings<'_>, state: &str, tier: &str, pick: Pick, steady: bool, arm: Arm, common: bool) -> Option<(End, Option<f64>)> {
    let in_state: Vec<&Operation> = point
        .operations
        .iter()
        .filter(|operation| operation.state == state)
        .filter(|operation| !common || COMMON_WINDOW_OPERATIONS.contains(&operation.operation_name.as_str()))
        .collect();
    let linear_row = largest_lg_row(&in_state);
    let wanted = |operation: &Operation| in_the_tier(operation, tier, point.item.units_per_small_file) && kept_for_the_filter(operation, steady);
    let series: Vec<(&Operation, u64)> = in_state
        .iter()
        .filter(|operation| wanted(operation))
        .filter_map(|operation| arm_steps(operation, arm, linear_row).map(|steps| (*operation, steps)))
        .collect();
    let (operation, steps) = pick_from(&series, pick)?;
    let user_units = denominator(operation)?;
    Some(((steps, user_units, measured_total(operation)?), measured_total_by_mapped_units(operation)))
}

/// 序列里这个点这个状态用不用：做完了（有 `item_done`），且第一次碰、仍是 K 个单元的 W2 操作至少 5 个（F8）。
fn point_is_usable_in_the_series(point: &PointReadings<'_>, state: &str) -> bool {
    let first_touches = point
        .operations
        .iter()
        .filter(|operation| operation.state == state && operation.operation_name == "W2" && operation.old_units == point.item.units_per_small_file)
        .count();
    point.item.done && first_touches >= FIRST_TOUCHES_AT_LEAST
}

/// 一格的序列（六点里读得到的那几个）：每点的一端（读不到的 `None`）、序列读数、`mapped_units` 作 x 的 β。
fn series_of_the_cell(points: &Points<'_>, state: &str, tier: &str, pick: Pick, steady: bool, arm: Arm, common: bool) -> (Vec<Option<End>>, SeriesReading, Option<f64>) {
    let ends: Vec<Option<(End, Option<f64>)>> = HUNDRED_UNITS_PER_FILE_SERIES
        .iter()
        .map(|name| points.get(*name).filter(|point| point_is_usable_in_the_series(point, state)).and_then(|point| end_among(point, state, tier, pick, steady, arm, common)))
        .collect();
    let read: Vec<(End, Option<f64>)> = ends.iter().flatten().copied().collect();
    let looser = read.iter().map(|(end, _)| end.1).collect::<BTreeSet<u64>>().len() > 1;
    let quantity = |end: End| if looser { end.0 as f64 / end.1 as f64 } else { end.0 as f64 };
    let by_total: Vec<(f64, f64)> = read.iter().map(|(end, _)| (quantity(*end), end.2)).collect();
    let by_mapped: Vec<(f64, f64)> = read.iter().filter_map(|(end, mapped)| mapped.map(|position| (quantity(*end), position))).collect();
    let mapped_beta = (by_mapped.len() == read.len()).then(|| least_squares_slope(&by_mapped)).flatten();
    (ends.iter().map(|end| end.map(|(end, _)| end)).collect(), series_reading(&by_total), mapped_beta)
}

/// 序列读数里相邻段 α 的文字：读不到的写 `unread`。
pub(super) fn alphas_text(reading: &SeriesReading) -> String {
    if reading.alphas.is_empty() {
        return "-".to_string();
    }
    reading.alphas.iter().map(|value| alpha_text(*value)).collect::<Vec<_>>().join(",")
}

/// 去向（登记修订 2 的去向表）：主格的判语、`series_void` 与读到的点数对岔路单第 10 行意味着什么。
pub(super) fn series_route(verdict: &str, series_void: &str) -> &'static str {
    if series_void != "-" || verdict == "not_enough_points" {
        return "series_not_made";
    }
    match verdict {
        "stable_grows" => "supports_the_flip_on_the_common_window",
        "stable_between" | "stable_does_not_grow" => "does_not_support_the_flip",
        _ => "unstable_the_cell_stays_open",
    }
}

/// 按操作取出两份产物的每一次固定点的六样（`rounds`、`final_roles`、`final_nodes`、`round_work`、`final_set_hash`、`previous_records`），只取 W1–W4；
/// 键是 `<状态>/<操作>/<第几次>/<第几次判>`。
type SettleReadings = (u64, u64, u64, u64, String, u64);

fn window_settle_readings(item: &PointArmReadings) -> Vec<(String, SettleReadings)> {
    item.window
        .iter()
        .filter(|settle| COMMON_WINDOW_OPERATIONS.contains(&settle.operation_name.as_str()))
        .map(|settle| {
            (
                settle.publish_key(),
                (settle.rounds, settle.final_roles, settle.final_nodes, settle.round_work, settle.final_set_hash.clone(), settle.previous_records),
            )
        })
        .collect()
}

/// 这一份产物的 W1–W4 固定点对另一份同名的：(这一份有几次, 不同的键，次序照这一份、另一份独有的接在后面)。
pub(super) fn settles_against(today: &PointArmReadings, before: &PointArmReadings) -> (usize, Vec<String>) {
    let (today_settles, before_settles) = (window_settle_readings(today), window_settle_readings(before));
    let before_by_key: BTreeMap<&String, &SettleReadings> = before_settles.iter().map(|(key, readings)| (key, readings)).collect();
    let today_keys: BTreeSet<&String> = today_settles.iter().map(|(key, _)| key).collect();
    let mut differing: Vec<String> = today_settles
        .iter()
        .filter(|(key, readings)| before_by_key.get(key) != Some(&readings))
        .map(|(key, _)| key.clone())
        .collect();
    differing.extend(before_settles.iter().map(|(key, _)| key).filter(|key| !today_keys.contains(key)).cloned());
    (today_settles.len(), differing)
}

fn first_or_dash(keys: &[String]) -> &str {
    keys.first().map_or("-", String::as_str)
}

/// 钉（失败条款 ②）：复量段再跑的 `files_100_units_100` 对第一段同名产物，W1–W4 每个固定点六样逐个相同。
fn report_hundred_units_per_file_pin(points: &Points<'_>, pin_against: &[PointArmReadings], out: &mut Lines) -> bool {
    let (Some(rerun), Some(first)) = (points.get("files_100_units_100"), pin_against.iter().find(|item| item.point == "files_100_units_100" && item.arm == "head")) else {
        out.push("name=verdict quantity=hundred_units_per_file_pin matches=not_run".to_string());
        return true;
    };
    let (settles, differing) = settles_against(rerun.item, first);
    out.push(format!(
        "name=verdict quantity=hundred_units_per_file_pin settles={settles} differing={} matches={} first_differing={}",
        differing.len(),
        differing.is_empty() && settles > 0,
        first_or_dash(&differing)
    ));
    differing.is_empty() && settles > 0
}

/// 序列用到的产物（复量段的、第一段高端的、钉的）头里三份被测核心文件的 sha256 逐个相同（作废条款 ①）；一份产物缺这三行记 `not_run`。
fn report_sources_agree(items: &[&PointArmReadings], out: &mut Lines) -> bool {
    let missing = items.iter().filter(|item| item.source_sha256.len() < 3).count();
    let mut values: BTreeMap<&String, BTreeSet<&String>> = BTreeMap::new();
    for item in items {
        for (file, sha) in &item.source_sha256 {
            values.entry(file).or_default().insert(sha);
        }
    }
    let agree = missing == 0 && !values.is_empty() && values.values().all(|shas| shas.len() == 1);
    out.push(format!(
        "name=verdict quantity=sources_agree products={} products_without_the_three_lines={missing} files={} files_with_more_than_one_value={} agree={}",
        items.len(),
        values.len(),
        values.values().filter(|shas| shas.len() > 1).count(),
        if missing > 0 { "not_run".to_string() } else { agree.to_string() }
    ));
    agree
}

/// 复量段（修订 2）的每一格、阳性对照、判语与去向；交回钉与来源是不是都过了。
pub(super) fn report_hundred_units_per_file_series(points: &Points<'_>, pin_against: &[PointArmReadings], out: &mut Lines) -> bool {
    let pin_matches = report_hundred_units_per_file_pin(points, pin_against, out);
    let used: Vec<&PointArmReadings> = HUNDRED_UNITS_PER_FILE_SERIES.iter().filter_map(|name| points.get(*name).map(|point| point.item)).chain(pin_against.iter()).collect();
    let sources_agree = report_sources_agree(&used, out);
    let mut primary: BTreeMap<&'static str, SeriesReading> = BTreeMap::new();
    for state in ["same_mount", "remounted"] {
        for tier in ["one_user_unit", "user_units_equal_declared"] {
            for pick in PICKS {
                for steady in [false, true] {
                    for (window, common) in [("common_window", true), ("own_window", false)] {
                        for arm in ARMS {
                            let (ends, reading, mapped_beta) = series_of_the_cell(points, state, tier, pick, steady, arm, common);
                            let is_primary = common && state == "same_mount" && tier == "one_user_unit" && pick == Pick::LaterPeak && steady;
                            if is_primary {
                                primary.insert(arm.tag(), reading.clone());
                            }
                            out.push(format!(
                                "name=series_cell state={state} tier={tier} pick={} filter={} window={window} arm={} primary={} points={} ends={} alphas={} beta={} beta_band={} grows={} between={} does_not_grow={} unread_segments={} beta_by_mapped_units={} verdict={}",
                                pick.tag(),
                                if steady { "steady" } else { "all" },
                                arm.tag(),
                                if is_primary && arm == Arm::Tested { "yes" } else { "no" },
                                reading.points,
                                ends.iter().map(|end| end.map_or("unread".to_string(), |end| format!("{}/{}/{}", end.0, end.1, end.2))).collect::<Vec<_>>().join(","),
                                alphas_text(&reading),
                                alpha_text(reading.beta),
                                band_text(reading.beta),
                                reading.grows,
                                reading.between,
                                reading.does_not_grow,
                                reading.unread,
                                alpha_text(mapped_beta),
                                reading.verdict
                            ));
                        }
                    }
                }
            }
        }
    }
    let tested = primary.get(Arm::Tested.tag()).cloned();
    let controls_red = controls_are_red(
        primary.get(Arm::ScanEveryPublish.tag()).and_then(|reading| reading.beta),
        primary.get(Arm::LargestLgRowLinear.tag()).and_then(|reading| reading.beta),
    );
    let series_void = if !pin_matches {
        "pin_differs"
    } else if !sources_agree {
        if used.iter().any(|item| item.source_sha256.len() < 3) { "sources_unread" } else { "sources_differ" }
    } else if !controls_red {
        "control_not_red"
    } else {
        "-"
    };
    let verdict = tested.as_ref().map_or("not_enough_points", |reading| reading.verdict);
    out.push(format!(
        "name=verdict quantity=hundred_units_per_file_series outcome={verdict} series_void={series_void} points={} route={}",
        tested.as_ref().map_or(0, |reading| reading.points),
        series_route(verdict, series_void)
    ));
    pin_matches && sources_agree
}

// ---- 对账分辨段 reconciliation_split（登记设计员修订 3）----
/// 逐操作取镜像与对账的两个点（`head` 与 `plain` 两个构建都跑）；`base` 是阴性对照。
const SPLIT_POINTS: [&str; 2] = ["base", "files_1000"];
/// 池自己的 `name=state` 行对 E206 的三个点（`files_10000` 只做这一行，离线，不重跑）。
const STATE_ROW_POINTS: [&str; 3] = ["base", "files_1000", "files_10000"];
/// `state_rows_against_e206` 比的四个池字段。
const POOL_STATE_FIELDS: [&str; 4] = ["allocation_records", "cluster_segments", "mapped_units", "units"];

fn operation_label(state: &str, operation_name: &str, operation_index: u64) -> String {
    format!("{state}/{operation_name}/{operation_index}")
}

/// `head_against_plain`：逐操作比两个构建的镜像哈希，重挂载前后各是否相同。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct HeadAgainstPlain {
    pub(super) operations: usize,
    pub(super) differing: usize,
    pub(super) first_differing: String,
    pub(super) before_the_crash_same: bool,
    pub(super) after_the_mount_same: bool,
}

pub(super) fn head_against_plain(head: &PointArmReadings, plain: &PointArmReadings) -> Option<HeadAgainstPlain> {
    if head.images_after_the_operations.is_empty() || plain.images_after_the_operations.is_empty() {
        return None;
    }
    let plain_by_operation: BTreeMap<&(String, String, u64), &String> = plain.images_after_the_operations.iter().map(|(key, sha)| (key, sha)).collect();
    let head_operations: BTreeSet<&(String, String, u64)> = head.images_after_the_operations.iter().map(|(key, _)| key).collect();
    let mut differing: Vec<String> = head
        .images_after_the_operations
        .iter()
        .filter(|(key, sha)| plain_by_operation.get(key) != Some(&sha))
        .map(|((state, operation_name, operation_index), _)| operation_label(state, operation_name, *operation_index))
        .collect();
    differing.extend(
        plain
            .images_after_the_operations
            .iter()
            .filter(|(key, _)| !head_operations.contains(key))
            .map(|((state, operation_name, operation_index), _)| operation_label(state, operation_name, *operation_index)),
    );
    let same_at = |when: &str| matches!((head.images_at_the_remount.get(when), plain.images_at_the_remount.get(when)), (Some(left), Some(right)) if left == right);
    Some(HeadAgainstPlain {
        operations: head.images_after_the_operations.len(),
        first_differing: first_or_dash(&differing).to_string(),
        differing: differing.len(),
        before_the_crash_same: same_at("before_the_crash"),
        after_the_mount_same: same_at("after_the_mount"),
    })
}

/// `placement_gap`：每个操作「计数记的落点槽数 − 盘上变了的单元槽数」的符号分布。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct PlacementGap {
    pub(super) operations: usize,
    pub(super) negative: usize,
    pub(super) zero: usize,
    pub(super) positive: usize,
    pub(super) first_negative: String,
}

pub(super) fn placement_gap(head: &PointArmReadings) -> Option<PlacementGap> {
    if head.reconciliation_rows.is_empty() {
        return None;
    }
    let rows = &head.reconciliation_rows;
    let first_negative = rows
        .iter()
        .find(|(_, (changed, placed))| placed < changed)
        .map_or("-".to_string(), |((state, operation_name, operation_index), _)| operation_label(state, operation_name, *operation_index));
    Some(PlacementGap {
        operations: rows.len(),
        negative: rows.iter().filter(|(_, (changed, placed))| placed < changed).count(),
        zero: rows.iter().filter(|(_, (changed, placed))| placed == changed).count(),
        positive: rows.iter().filter(|(_, (changed, placed))| placed > changed).count(),
        first_negative,
    })
}

/// 五项（`rounds`、`final_roles`、`final_nodes`、`round_work`、`final_set_hash`）在两份产物里逐个固定点不同的键，按键排序（同 `report_reconciliation` 的 `first_differing`）。
fn first_differing_publish(today: &PointArmReadings, before: &PointArmReadings) -> Option<String> {
    let keyed = |item: &PointArmReadings| -> BTreeMap<String, (u64, u64, u64, u64, String)> {
        window_settle_readings(item).into_iter().map(|(key, readings)| (key, (readings.0, readings.1, readings.2, readings.3, readings.4))).collect()
    };
    let (today_settles, before_settles) = (keyed(today), keyed(before));
    let keys: BTreeSet<String> = today_settles.keys().chain(before_settles.keys()).cloned().collect();
    keys.into_iter().find(|key| today_settles.get(key) != before_settles.get(key))
}

/// 一份产物里键是 `key`（`<状态>/<操作>/<第几次>/<第几次判>`）的固定点。
fn settle_named<'items>(item: &'items PointArmReadings, key: &str) -> Option<&'items Settle> {
    item.window.iter().find(|settle| settle.publish_key() == key)
}

/// 这个键所在那次操作之前的池状态行（键去掉最后一段「第几次判」）。
fn state_of_the_publish(item: &PointArmReadings, key: &str) -> Option<Fields> {
    let (operation_prefix, _) = key.rsplit_once('/')?;
    item.judgements.get(&format!("{operation_prefix}/")).map(|judgement| judgement.state.clone())
}

/// `state_rows_against_e206`：池自己的 `name=state` 行（四个字段）今天对 E206 逐个量窗操作。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct StateRows {
    pub(super) operations: usize,
    pub(super) differing_operations: usize,
    pub(super) by_field: [usize; 4],
    pub(super) same_mount_differing_operations: usize,
    pub(super) first_differing: String,
    /// 五项里第一处不同的发布上，`previous_records`、`previous_nodes`、`tree_height` 与池的 `name=state` 行在两边是否相同；没有这样的发布是 `None`。
    inputs_equal_at_the_first_differing_publish: Option<bool>,
}

pub(super) fn state_rows_against_e206(today: &PointReadings<'_>, before: &PointReadings<'_>) -> Option<StateRows> {
    let state_of = |item: &PointArmReadings, operation: &Operation| -> Option<Fields> {
        item.judgements.get(&format!("{}/{}/{}/", operation.state, operation.operation_name, operation.operation_index)).map(|judgement| judgement.state.clone())
    };
    let compared: Vec<(&Operation, Fields, Fields)> = today
        .operations
        .iter()
        .filter_map(|operation| Some((operation, state_of(today.item, operation).filter(|fields| !fields.is_empty())?, state_of(before.item, operation).filter(|fields| !fields.is_empty())?)))
        .collect();
    if compared.is_empty() || compared.iter().any(|(_, _, old)| number(old, "cluster_segments").is_none() || number(old, "allocation_records").is_none()) {
        return None;
    }
    let differs_in = |new: &Fields, old: &Fields, field: &str| number(new, field) != number(old, field);
    let differing: Vec<&(&Operation, Fields, Fields)> = compared.iter().filter(|(_, new, old)| POOL_STATE_FIELDS.iter().any(|field| differs_in(new, old, field))).collect();
    let mut by_field = [0; 4];
    for (slot, field) in by_field.iter_mut().zip(POOL_STATE_FIELDS) {
        *slot = compared.iter().filter(|(_, new, old)| differs_in(new, old, field)).count();
    }
    let inputs_equal = first_differing_publish(today.item, before.item).map(|key| {
        let same_settle = settle_named(today.item, &key)
            .zip(settle_named(before.item, &key))
            .is_some_and(|(new, old)| (new.previous_records, new.previous_nodes, new.tree_height) == (old.previous_records, old.previous_nodes, old.tree_height));
        let same_state = state_of_the_publish(today.item, &key)
            .zip(state_of_the_publish(before.item, &key))
            .is_some_and(|(new, old)| POOL_STATE_FIELDS.iter().all(|field| !differs_in(&new, &old, field)));
        same_settle && same_state
    });
    Some(StateRows {
        operations: compared.len(),
        differing_operations: differing.len(),
        by_field,
        same_mount_differing_operations: differing.iter().filter(|(operation, _, _)| operation.state == "same_mount").count(),
        first_differing: differing.first().map_or("-".to_string(), |(operation, _, _)| operation_label(&operation.state, &operation.operation_name, operation.operation_index)),
        inputs_equal_at_the_first_differing_publish: inputs_equal,
    })
}

/// 总判要的四个谓词与五项还有没有不同（修订 3「谓词与结局」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SplitFacts {
    /// `c_head`：`head_against_plain` 的 `differing > 0` 或镜像重挂载前后任一不同。
    pub(super) head_differs: bool,
    /// `c_gap`：`placement_gap` 的 `negative > 0`。
    pub(super) gap_negative: bool,
    /// `c_rerun`：`rerun_against_the_first_segment` 的 `differing > 0`。
    pub(super) rerun_differs: bool,
    /// `t_sign`：池结构的状态行有不同、同一次挂载里一处不差、`base` 一处不差。
    pub(super) signature_of_a_changed_tree: bool,
    /// 五项（对 E206 的对账）仍有不同。
    pub(super) five_items_differ: bool,
}

/// 结局表（次序就是判的次序）；读不到的交 `not_run`。表里六行之外的一种：没有任何谓词为真、五项也不再有不同，记 `nothing_left_to_split`。
pub(super) fn reconciliation_split_outcome(facts: Option<SplitFacts>) -> &'static str {
    let Some(facts) = facts else { return "not_run" };
    let counting = facts.head_differs || facts.gap_negative;
    if counting && !facts.signature_of_a_changed_tree {
        "counting_wrong"
    } else if counting && facts.signature_of_a_changed_tree {
        "both"
    } else if !facts.rerun_differs && facts.signature_of_a_changed_tree {
        "tree_changed"
    } else if !facts.rerun_differs && facts.five_items_differ {
        "neither_reading_differs"
    } else if facts.rerun_differs {
        "neither_not_reproducible"
    } else {
        "nothing_left_to_split"
    }
}

/// 一个点的四行读数（交回：谓词与五项有没有不同；读不全交 `None`）。
struct PointSplitReadings {
    head_differs: Option<bool>,
    gap_negative: Option<bool>,
    rerun_differs: Option<bool>,
    state_rows: Option<StateRows>,
    five_items_differ: Option<bool>,
}

fn report_head_against_plain(point: &str, reading: &Option<HeadAgainstPlain>, out: &mut Lines) {
    match reading {
        Some(reading) => out.push(format!(
            "name=verdict quantity=head_against_plain point={point} operations={} differing={} first_differing={} remount_before_the_crash_same={} remount_after_the_mount_same={}",
            reading.operations, reading.differing, reading.first_differing, reading.before_the_crash_same, reading.after_the_mount_same
        )),
        None => out.push(format!("name=verdict quantity=head_against_plain point={point} operations=not_run")),
    }
}

fn report_placement_gap(point: &str, reading: &Option<PlacementGap>, out: &mut Lines) {
    match reading {
        Some(reading) => out.push(format!(
            "name=verdict quantity=placement_gap point={point} operations={} negative={} first_negative={} zero={} positive={}",
            reading.operations, reading.negative, reading.first_negative, reading.zero, reading.positive
        )),
        None => out.push(format!("name=verdict quantity=placement_gap point={point} operations=not_run")),
    }
}

fn report_rerun(point: &str, reading: &Option<(usize, Vec<String>)>, out: &mut Lines) {
    match reading {
        Some((settles, differing)) => out.push(format!(
            "name=verdict quantity=rerun_against_the_first_segment point={point} settles={settles} differing={} first_differing={}",
            differing.len(),
            first_or_dash(differing)
        )),
        None => out.push(format!("name=verdict quantity=rerun_against_the_first_segment point={point} settles=not_run")),
    }
}

fn report_state_rows(point: &str, reading: &Option<StateRows>, out: &mut Lines) {
    match reading {
        Some(rows) => out.push(format!(
            "name=verdict quantity=state_rows_against_e206 point={point} operations={} differing_operations={} by_field={} same_mount_differing_operations={} first_differing={} inputs_equal_at_the_first_differing_publish={}",
            rows.operations,
            rows.differing_operations,
            POOL_STATE_FIELDS.iter().zip(rows.by_field).map(|(field, count)| format!("{field}:{count}")).collect::<Vec<_>>().join(","),
            rows.same_mount_differing_operations,
            rows.first_differing,
            rows.inputs_equal_at_the_first_differing_publish.map_or("-".to_string(), |equal| equal.to_string())
        )),
        None => out.push(format!("name=verdict quantity=state_rows_against_e206 point={point} operations=not_run")),
    }
}

/// 对账分辨段（修订 3）：每点四行读数、`base` 阴性对照与 `files_1000` 的总判。交回阴性对照有没有过（读不到不算没过）。
pub(super) fn report_reconciliation_split(points: &Points<'_>, previous: &Points<'_>, retro: &Points<'_>, out: &mut Lines) -> bool {
    let mut by_point: BTreeMap<&str, PointSplitReadings> = BTreeMap::new();
    for point in SPLIT_POINTS {
        let head = points.get(point).filter(|reading| reading.item.done);
        let plain = points.get(&format!("{point}__plain")).filter(|reading| reading.item.done);
        let against_plain = head.zip(plain).and_then(|(head, plain)| head_against_plain(head.item, plain.item));
        let gap = head.and_then(|head| placement_gap(head.item));
        let rerun = head.zip(previous.get(point)).map(|(head, before)| settles_against(head.item, before.item));
        report_head_against_plain(point, &against_plain, out);
        report_placement_gap(point, &gap, out);
        report_rerun(point, &rerun, out);
        by_point.insert(
            point,
            PointSplitReadings {
                head_differs: against_plain.as_ref().map(|reading| reading.differing > 0 || !reading.before_the_crash_same || !reading.after_the_mount_same),
                gap_negative: gap.as_ref().map(|reading| reading.negative > 0),
                rerun_differs: rerun.as_ref().map(|(_, differing)| !differing.is_empty()),
                state_rows: None,
                five_items_differ: head.zip(retro.get(point)).map(|(today, before)| first_differing_publish(today.item, before.item).is_some()),
            },
        );
    }
    for point in STATE_ROW_POINTS {
        let today = if point == "files_10000" { previous.get(point) } else { points.get(point).filter(|reading| reading.item.done) };
        let rows = today.zip(retro.get(point)).and_then(|(today, before)| state_rows_against_e206(today, before));
        report_state_rows(point, &rows, out);
        if let Some(readings) = by_point.get_mut(point) {
            readings.state_rows = rows;
        }
    }
    let base_rows_differ = by_point.get("base").and_then(|readings| readings.state_rows.as_ref()).map(|rows| rows.differing_operations > 0);
    let mut negative_control_ok = true;
    for point in SPLIT_POINTS {
        let Some(readings) = by_point.get(point) else { continue };
        let signature = readings
            .state_rows
            .as_ref()
            .zip(base_rows_differ)
            .map(|(rows, base_differs)| rows.differing_operations > 0 && rows.same_mount_differing_operations == 0 && !base_differs);
        let facts = readings
            .head_differs
            .zip(readings.gap_negative)
            .zip(readings.rerun_differs)
            .zip(signature)
            .zip(readings.five_items_differ)
            .map(|((((head_differs, gap_negative), rerun_differs), signature_of_a_changed_tree), five_items_differ)| SplitFacts {
                head_differs,
                gap_negative,
                rerun_differs,
                signature_of_a_changed_tree,
                five_items_differ,
            });
        if point == "base" {
            // 阴性对照：四个谓词（状态行有没有不同算第四个）全假
            let named_true: Vec<&str> = [
                ("head_differs", readings.head_differs),
                ("gap_negative", readings.gap_negative),
                ("rerun_differs", readings.rerun_differs),
                ("state_rows_differ", base_rows_differ),
            ]
            .into_iter()
            .filter(|(_, value)| *value == Some(true))
            .map(|(name, _)| name)
            .collect();
            let unread = [readings.head_differs, readings.gap_negative, readings.rerun_differs, base_rows_differ].iter().any(Option::is_none);
            let outcome = if unread { "not_run" } else if named_true.is_empty() { "negative_control_clean" } else { "negative_control_red" };
            negative_control_ok &= outcome != "negative_control_red";
            out.push(format!(
                "name=verdict quantity=reconciliation_split point=base outcome={outcome} reason={}",
                if named_true.is_empty() { "-".to_string() } else { named_true.join(",") }
            ));
        } else {
            out.push(format!(
                "name=verdict quantity=reconciliation_split point={point} outcome={} reason={}",
                reconciliation_split_outcome(facts),
                facts.map_or("not_run".to_string(), |facts| format!(
                    "head_differs={},gap_negative={},rerun_differs={},changed_tree_signature={},five_items_differ={}",
                    facts.head_differs, facts.gap_negative, facts.rerun_differs, facts.signature_of_a_changed_tree, facts.five_items_differ
                ))
            ));
        }
    }
    negative_control_ok
}

// ---- 报告 ----
/// 点表里的键：今天的实现（`head`）标准项用点名，探针项与 `plain` 用项名；E206 的 p2a 用点名。
fn point_key(item: &PointArmReadings) -> String {
    if (item.arm == "head" && item.probe == "standard") || item.arm == "p2a" {
        item.point.clone()
    } else {
        item.name.clone()
    }
}

pub(super) fn points_of(items: &[PointArmReadings]) -> Points<'_> {
    items.iter().map(|item| (point_key(item), PointReadings { item, operations: operations_of(item) })).collect()
}

/// `quantity` 是 (读的是什么, 登记的量号)。
fn report_quantities(
    quantity: (&str, &str),
    segment: &str,
    tier: &str,
    cells: &[JudgedCell],
    sets: &[&str],
    tallies: &BTreeMap<(&'static str, &'static str), ControlTally>,
    out: &mut Lines,
) {
    let chosen: Vec<&JudgedCell> = cells.iter().filter(|cell| cell.tier == tier && sets.contains(&cell.set)).collect();
    report_row_reading(quantity, segment, tier, &chosen, tallies, out);
}

fn report_void(tallies: &BTreeMap<(&'static str, &'static str), ControlTally>, out: &mut Lines) {
    for ((segment, tier), tally) in tallies {
        out.push(format!(
            "name=verdict quantity=void_clauses segment={segment} tier={tier} applicable_cells={} scan_control_red={} linear_control_red={} scan_control_void={} linear_control_void={}",
            tally.applicable,
            tally.scan_red,
            tally.linear_red,
            tally.scan_void(),
            tally.linear_void()
        ));
    }
}

/// 读好的产物（这一段的与回扫的）上逐量报，行推进 `out`；交回第七节 ② 第 1 条的两行都相同没有（没给回扫交 true）。
pub(super) fn report_items(segment: &str, items: &[PointArmReadings], retro_items: &[PointArmReadings], out: &mut Lines) -> bool {
    let mut points = points_of(items);
    let retro = points_of(retro_items);
    out.push(format!(
        "name=model segment={segment} products={} retro_products={} leaf_slots={LEAF_SLOTS} internal_fanout={INTERNAL_FANOUT} settling_rounds_at_most={SETTLING_ROUNDS_LIMIT} disks={DISKS}",
        items.len(),
        retro_items.len()
    ));
    for point in points.values() {
        report_item_checks(point, out);
    }
    report_operations(&points, out);
    let voided = points_with_a_frozen_publish(&points, out);
    points.retain(|name, _| !voided.contains(name));
    report_missing_the_pooled_fallback_row(&points, out);
    let mut cells = Vec::new();
    match segment {
        "first" => {
            cells = judge_sets(&points, &NEW_POINT_SETS, out);
            report_similar_total_set(&points, out);
            report_spike_pin(&points, out);
            report_image_pins(&points, out);
            report_then_big_ratio(&points, out);
            report_reconciliation(&points, &retro, out);
        }
        "fourth" => cells = judge_sets(&points, &BIG_POINT_SETS, out),
        _ => {}
    }
    report_declared_against_mapped(&points, out);
    report_unit_counts(&points, out);
    report_spike_causes(&points, out);
    if segment == "second" {
        report_reproduction_probes(&points, out);
    }
    let mut pins_match = true;
    if !retro_items.is_empty() {
        cells.extend(judge_sets(&retro, &RETRO_SETS, out));
        report_retro_data_units_first_cold(&retro, out);
        pins_match = old_criterion_pins(retro_items, out);
    }
    report_looser_instances(&cells, out);
    let tallies = control_tallies(&cells);
    report_void(&tallies, out);
    if segment == "first" {
        for (reading, quantity, tier, sets) in NEW_POINT_QUANTITIES {
            report_quantities((reading, quantity), "new", tier, &cells, sets, &tallies, out);
        }
    }
    if segment == "fourth" {
        for tier in ["one_user_unit", "user_units_equal_declared"] {
            report_quantities((BIG_POINT_READING, "q10"), "big", tier, &cells, &["K1000L"], &tallies, out);
        }
    }
    if !retro_items.is_empty() {
        report_quantities((RETRO_READING, "q4"), "retro", "one_user_unit", &cells, &["axis_files", "axis_data_units", "axis_unlinks", "axis_unit_area"], &tallies, out);
    }
    pins_match
}

/// 复量段与对账分辨段在通用的逐点逐量读数（`report_items`）之外多读的东西；交回钉、来源与阴性对照有没有过。
pub(super) fn report_new_segment(segment: &str, items: &[PointArmReadings], previous: &[PointArmReadings], pin_against: &[PointArmReadings], retro: &[PointArmReadings], out: &mut Lines) -> bool {
    let points = points_of(items);
    match segment {
        "hundred_units_per_file_repeat" => report_hundred_units_per_file_series(&points, pin_against, out),
        _ => {
            let (previous_points, retro_points) = (points_of(previous), points_of(retro));
            let used: Vec<&PointArmReadings> = items.iter().chain(previous.iter()).collect();
            report_sources_agree(&used, out);
            report_reconciliation_split(&points, &previous_points, &retro_points, out)
        }
    }
}

/// 读一串产物路径。
pub(super) fn read_items(paths: &[String]) -> Result<Vec<PointArmReadings>, String> {
    paths.iter().map(|path| read_item(path)).collect()
}
