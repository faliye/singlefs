//! 格的结果与逐格报告、各集合与操作的报告、尖峰、复现探针与各项核对的报告、旧判据的钉。

use super::*;

/// 一格的结果：每条臂的两端（三点集合另有中点）读数。
#[derive(Clone, Debug, Default)]
pub(super) struct CellOutcome {
    /// 臂 → α（读法第 5 条的量；三点以上取低端对高端）。
    pub(super) alpha: BTreeMap<&'static str, Option<f64>>,
    /// 臂 → 原步数的 α（`looser=yes` 的格与读法第 5 条的量不同）。
    pub(super) raw_alpha: BTreeMap<&'static str, Option<f64>>,
}

impl CellOutcome {
    fn band_of(&self, arm: Arm) -> Option<Band> {
        self.alpha.get(arm.tag()).copied().flatten().map(band)
    }
}

/// 一个点一个状态一档一种取法一条臂取出来的那一端；读不到交 `None`。
fn end_of(point: &PointReadings<'_>, state: &str, tier: &str, pick: Pick, steady: bool, arm: Arm) -> Option<(End, Option<f64>, &'static str)> {
    let in_state: Vec<&Operation> = point.operations.iter().filter(|operation| operation.state == state).collect();
    let linear_row = largest_lg_row(&in_state);
    let series: Vec<(&Operation, u64)> = in_state
        .iter()
        .filter(|operation| in_the_tier(operation, tier, point.item.units_per_small_file) && kept_for_the_filter(operation, steady))
        .filter_map(|operation| arm_steps(operation, arm, linear_row).map(|steps| (*operation, steps)))
        .collect();
    let (operation, steps) = pick_from(&series, pick)?;
    let user_units = denominator(operation)?;
    Some(((steps, user_units, measured_total(operation)?), measured_total_by_mapped_units(operation), linear_row.unwrap_or("none")))
}

/// 一格（集合 × 状态 × 档 × 取法 × 剔不剔旗）：每条臂一行 `name=cell`，交回各臂的 α。
pub(super) fn report_cell(points: &Points<'_>, set: &ComparisonSet, state: &str, tier: &str, pick: Pick, steady: bool, out: &mut Lines) -> CellOutcome {
    let mut outcome = CellOutcome::default();
    for arm in ARMS {
        let ends: Vec<Option<(End, Option<f64>, &'static str)>> = set
            .points
            .iter()
            .map(|name| points.get(*name).and_then(|point| end_of(point, state, tier, pick, steady, arm)))
            .collect();
        let low = ends.first().copied().flatten();
        let high = ends.last().copied().flatten();
        let reading = low.zip(high).map(|(low, high)| two_point_reading(low.0, high.0));
        let by_mapped = low.zip(high).and_then(|(low, high)| {
            let looser = low.0 .1 != high.0 .1;
            let quantity = |end: End| if looser { end.0 as f64 / end.1 as f64 } else { end.0 as f64 };
            positive_alpha(quantity(low.0), quantity(high.0), low.1?, high.1?)
        });
        let slopes: String = if ends.len() > 2 && ends.iter().all(Option::is_some) {
            let looser = ends.iter().flatten().map(|end| end.0 .1).collect::<BTreeSet<u64>>().len() > 1;
            let quantity = |end: End| if looser { end.0 as f64 / end.1 as f64 } else { end.0 as f64 };
            let values: Vec<(f64, f64)> = ends.iter().flatten().map(|end| (quantity(end.0), end.0 .2)).collect();
            let upper = positive_alpha(values[values.len() - 2].0, values[values.len() - 1].0, values[values.len() - 2].1, values[values.len() - 1].1);
            format!(
                "{} upper_pair_alpha={}",
                local_slopes(&values).into_iter().map(alpha_text).collect::<Vec<_>>().join(","),
                alpha_text(upper)
            )
        } else if ends.len() > 2 && high.is_none() {
            // 作废条款第 7 条：高端缺了不拿两点冒充三点，α 报 unread；低端与中点都在时只另报这两点的 α（总量十倍）。
            let lower_pair = ends[0].zip(ends[1]).and_then(|(lower_end, middle)| two_point_reading(lower_end.0, middle.0).alpha);
            format!("high_end_missing lower_pair_alpha={}", alpha_text(lower_pair))
        } else {
            "two_points".to_string()
        };
        let ends_text: Vec<String> = ends
            .iter()
            .map(|end| end.map_or("unread".to_string(), |end| format!("{}/{}/{}", end.0 .0, end.0 .1, end.0 .2)))
            .collect();
        let alpha = reading.and_then(|reading| reading.alpha);
        out.push(format!(
            "name=cell segment={} set={} state={state} tier={tier} pick={} filter={} arm={} ends={} looser={} alpha={} band={} raw_alpha={} alpha_by_mapped_units={} local_slopes={} linear_row={}",
            set.segment,
            set.name,
            pick.tag(),
            if steady { "steady" } else { "all" },
            arm.tag(),
            ends_text.join(","),
            reading.map_or("unread", |reading| if reading.looser { "yes" } else { "no" }),
            alpha_text(alpha),
            band_text(alpha),
            alpha_text(reading.and_then(|reading| reading.raw_alpha)),
            alpha_text(by_mapped),
            slopes,
            low.map_or("none", |end| end.2)
        ));
        outcome.alpha.insert(arm.tag(), alpha);
        outcome.raw_alpha.insert(arm.tag(), reading.and_then(|reading| reading.raw_alpha));
    }
    outcome
}

// ---- 判读：行一、行二、作废条款第 1、2 条（第六节、第十一节）----
/// 一格的定位与结果。
pub(super) struct JudgedCell {
    pub(super) segment: &'static str,
    pub(super) set: &'static str,
    pub(super) state: &'static str,
    pub(super) tier: &'static str,
    pub(super) pick: Pick,
    pub(super) steady: bool,
    pub(super) outcome: CellOutcome,
}

/// 作废条款第 1、2 条：（段 × 档）各判各的，第四段与第一至三段不合并。
fn void_segment(segment: &'static str) -> &'static str {
    segment
}

/// 一个（段 × 档）里控制臂的账：应判的格（被测臂不随或介于）、`C1` 判随的、`C3` 判随的。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ControlTally {
    pub(super) applicable: u64,
    pub(super) scan_red: u64,
    pub(super) linear_red: u64,
}

impl ControlTally {
    pub(super) fn scan_void(self) -> bool {
        self.scan_red < self.applicable
    }
    pub(super) fn linear_void(self) -> bool {
        self.linear_red < self.applicable
    }
}

pub(super) fn control_tallies(cells: &[JudgedCell]) -> BTreeMap<(&'static str, &'static str), ControlTally> {
    let mut tallies: BTreeMap<(&'static str, &'static str), ControlTally> = BTreeMap::new();
    for cell in cells {
        let tally = tallies.entry((void_segment(cell.segment), cell.tier)).or_default();
        if matches!(cell.outcome.band_of(Arm::Tested), Some(Band::DoesNotGrow | Band::Between)) {
            tally.applicable += 1;
            tally.scan_red += u64::from(cell.outcome.band_of(Arm::ScanEveryPublish) == Some(Band::Grows));
            tally.linear_red += u64::from(cell.outcome.band_of(Arm::LargestLgRowLinear) == Some(Band::Grows));
        }
    }
    tallies
}

/// 一组格对行一、行二的判读（第六节 Q1–Q3、Q10 那一列；只报，合取留给主 agent）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RowReading {
    cells: u64,
    pub(super) unread: u64,
    /// `steady` 格全部「不随」且 `C1`、`C3` 在这些格都「随」。
    pub(super) row_one_holds: bool,
    /// 任一 `later_*` 的 `steady` 格「随」且 `C1`、`C3` 在该格「随」。
    pub(super) row_two_flips: bool,
    /// F2：某（集合、状态）里 `first` 的 `all` 格「随」而 `steady` 与 `later_*` 全不随。
    pub(super) only_the_first_grows: bool,
}

pub(super) fn row_reading(cells: &[&JudgedCell]) -> RowReading {
    let steady: Vec<&&JudgedCell> = cells.iter().filter(|cell| cell.steady).collect();
    fn controls_red(cell: &JudgedCell) -> bool {
        cell.outcome.band_of(Arm::ScanEveryPublish) == Some(Band::Grows) && cell.outcome.band_of(Arm::LargestLgRowLinear) == Some(Band::Grows)
    }
    let unread = cells.iter().filter(|cell| cell.outcome.band_of(Arm::Tested).is_none()).count();
    let row_one_holds = !steady.is_empty()
        && steady.iter().all(|cell| cell.outcome.band_of(Arm::Tested) == Some(Band::DoesNotGrow) && controls_red(cell));
    let row_two_flips = steady
        .iter()
        .any(|cell| cell.pick != Pick::First && cell.outcome.band_of(Arm::Tested) == Some(Band::Grows) && controls_red(cell));
    let mut only_the_first_grows = false;
    let groups: BTreeSet<(&str, &str, &str)> = cells.iter().map(|cell| (cell.set, cell.state, cell.tier)).collect();
    for (set, state, tier) in groups {
        let of_group: Vec<&&JudgedCell> = cells.iter().filter(|cell| (cell.set, cell.state, cell.tier) == (set, state, tier)).collect();
        let first_all_grows = of_group
            .iter()
            .any(|cell| cell.pick == Pick::First && !cell.steady && cell.outcome.band_of(Arm::Tested) == Some(Band::Grows));
        let rest_do_not = of_group
            .iter()
            .filter(|cell| cell.steady || cell.pick != Pick::First)
            .all(|cell| cell.outcome.band_of(Arm::Tested).is_some_and(|band| band != Band::Grows));
        only_the_first_grows |= first_all_grows && rest_do_not;
    }
    RowReading {
        cells: u64::try_from(cells.len()).unwrap_or(u64::MAX),
        unread: u64::try_from(unread).unwrap_or(u64::MAX),
        row_one_holds,
        row_two_flips,
        only_the_first_grows,
    }
}

/// 一组集合逐格报（每个集合 × 档 × 状态 × 取法 × 剔不剔旗），交回全部格。
pub(super) fn judge_sets(points: &Points<'_>, sets: &[ComparisonSet], out: &mut Lines) -> Vec<JudgedCell> {
    let mut cells = Vec::new();
    for set in sets {
        for &tier in set.tiers {
            for state in ["same_mount", "remounted"] {
                for pick in PICKS {
                    for steady in [false, true] {
                        let outcome = report_cell(points, set, state, tier, pick, steady, out);
                        cells.push(JudgedCell { segment: set.segment, set: set.name, state, tier, pick, steady, outcome });
                    }
                }
            }
        }
    }
    cells
}

pub(super) fn report_row_reading((reading_name, quantity): (&str, &str), segment: &str, tier: &str, cells: &[&JudgedCell], tallies: &BTreeMap<(&'static str, &'static str), ControlTally>, out: &mut Lines) {
    let reading = row_reading(cells);
    let tally = tallies.iter().find(|((in_segment, in_tier), _)| *in_segment == void_segment_name(segment) && *in_tier == tier).map(|(_, tally)| *tally).unwrap_or_default();
    out.push(format!(
        "name=verdict quantity={quantity} reading={reading_name} segment={segment} tier={tier} cells={} cells_unread={} row_one_holds={} row_two_flips={} only_the_first_grows={} scan_control_void={} linear_control_void={}",
        reading.cells,
        reading.unread,
        reading.row_one_holds,
        reading.row_two_flips,
        reading.only_the_first_grows,
        tally.scan_void(),
        tally.linear_void()
    ));
}

/// 报告行里的段名到作废条款按段那一键（同 `void_segment`，只是交回的不借静态生命期）。
fn void_segment_name(segment: &str) -> &str {
    match segment {
        "new" => void_segment("new"),
        "big" => void_segment("big"),
        _ => void_segment("retro"),
    }
}

/// Q3 的 T1e5：总量相近、改动差十倍，只报 α_u = ln(Q_hi ÷ Q_lo) ÷ ln(u_hi ÷ u_lo)（Q = 步数 ÷ user_units），另报原步数的；不按 x 判。
pub(super) fn report_similar_total_set(points: &Points<'_>, out: &mut Lines) {
    let names = ["files_100_units_1000", "files_10_units_10000"];
    for state in ["same_mount", "remounted"] {
        for pick in PICKS {
            for steady in [false, true] {
                let ends: Vec<Option<End>> = names
                    .iter()
                    .map(|name| points.get(*name).and_then(|point| end_of(point, state, "user_units_equal_declared", pick, steady, Arm::Tested)).map(|end| end.0))
                    .collect();
                let (per_unit, raw) = match (ends[0], ends[1]) {
                    (Some(low), Some(high)) => (
                        positive_alpha(low.0 as f64 / low.1 as f64, high.0 as f64 / high.1 as f64, low.1 as f64, high.1 as f64),
                        positive_alpha(low.0 as f64, high.0 as f64, low.1 as f64, high.1 as f64),
                    ),
                    _ => (None, None),
                };
                out.push(format!(
                    "name=t1e5 state={state} pick={} filter={} alpha_against_the_user_units_per_unit={} band_per_unit={} alpha_against_the_user_units_raw={} band_raw={}",
                    pick.tag(),
                    if steady { "steady" } else { "all" },
                    alpha_text(per_unit),
                    band_text(per_unit),
                    alpha_text(raw),
                    band_text(raw)
                ));
            }
        }
    }
}

/// F3：`looser=yes` 的格原步数 α ≥ 0.5 而步数 ÷ user_units 的 α ≤ 0.25。
pub(super) fn report_looser_instances(cells: &[JudgedCell], out: &mut Lines) {
    let mut instances = 0;
    for cell in cells {
        let raw = cell.outcome.raw_alpha.get(Arm::Tested.tag()).copied().flatten();
        let per_unit = cell.outcome.alpha.get(Arm::Tested.tag()).copied().flatten();
        if raw.is_some_and(|raw| raw >= BAND_GROWS_AT_LEAST) && per_unit.is_some_and(|value| value <= BAND_DOES_NOT_GROW_AT_MOST) && raw != per_unit {
            instances += 1;
            out.push(format!(
                "name=f3_looser_instance set={} state={} tier={} pick={} filter={} raw_alpha={} per_unit_alpha={}",
                cell.set,
                cell.state,
                cell.tier,
                cell.pick.tag(),
                if cell.steady { "steady" } else { "all" },
                alpha_text(raw),
                alpha_text(per_unit)
            ));
        }
    }
    out.push(format!("name=f3_looser_instances count={instances}"));
}

/// Q6：旗上的操作，逐点逐状态照报；第八节轨迹：B8 步数与 final_nodes 整列打（每个操作一行）。
pub(super) fn report_operations(points: &Points<'_>, out: &mut Lines) {
    for (name, point) in points {
        for operation in &point.operations {
            out.push(format!(
                "name=op_row item={} state={} operation={} operation_index={} order={} user_units={} old_units={} new_units={} allocation_records={} mapped_units={} steps={} final_nodes={} flagged={}",
                point.item.name,
                operation.state,
                operation.operation_name,
                operation.operation_index,
                operation.order,
                operation.user_units.map_or("unread".to_string(), |value| value.to_string()),
                operation.old_units,
                operation.new_units,
                operation.allocation_records.map_or("unread".to_string(), |value| value.to_string()),
                operation.mapped_units.map_or("unread".to_string(), |value| value.to_string()),
                operation.steps.map_or("unread".to_string(), |value| value.to_string()),
                operation.final_nodes,
                operation.flagged
            ));
        }
        for operation in &point.operations {
            out.push(format!(
                "name=op_pooled_fallback item={} state={} operation={} operation_index={} pooled_fallback_calls={} pooled_fallback_steps={} per_disk_fallback_steps={} by_delta_arms_total={}",
                point.item.name,
                operation.state,
                operation.operation_name,
                operation.operation_index,
                operation.pooled_fallback_calls,
                operation.pooled_fallback_steps,
                operation.fallback_scan_steps,
                if operation.pooled_fallback_calls > 0 { "missing_pooled_fallback_row_only_for_the_listed_steps" } else { "complete" }
            ));
        }
        for state in ["same_mount", "remounted"] {
            let in_state: Vec<&Operation> = point.operations.iter().filter(|operation| operation.state == state).collect();
            let nodes: Vec<u64> = in_state.iter().map(|operation| operation.final_nodes).collect();
            let flagged: Vec<String> = in_state
                .iter()
                .filter(|operation| operation.flagged)
                .map(|operation| format!("{}#{}:{}", operation.operation_name, operation.operation_index, operation.final_nodes))
                .collect();
            let peak = in_state.iter().filter_map(|operation| operation.steps.map(|steps| (steps, *operation))).max_by_key(|(steps, _)| *steps);
            out.push(format!(
                "name=q6_spike_flags point={name} state={state} operations={} median_final_nodes={} line={} flagged={} flagged_operations={} peak_steps={} peak_at={} final_steps={}",
                in_state.len(),
                lower_median(&nodes),
                spike_line(&nodes),
                flagged.len(),
                if flagged.is_empty() { "-".to_string() } else { flagged.join(",") },
                peak.map_or("unread".to_string(), |(steps, _)| steps.to_string()),
                peak.map_or("unread".to_string(), |(_, operation)| format!("{}#{}", operation.operation_name, operation.operation_index)),
                in_state.last().and_then(|operation| operation.steps).map_or("unread".to_string(), |steps| steps.to_string())
            ));
        }
    }
}

/// 作废条款（冻结的发布）：被量的每次操作之前读一次冻结着等重发的那一次是不是空，不空的那一点整点作废并报出来。
/// 交回作废的点名；没带这个字段的操作数另报在判决行里（读不到不当「空」，也不当「不空」）。
pub(super) fn points_with_a_frozen_publish(points: &Points<'_>, out: &mut Lines) -> BTreeSet<String> {
    let (mut voided, mut unread, mut operations) = (BTreeSet::new(), 0, 0);
    for (name, point) in points {
        for operation in &point.operations {
            operations += 1;
            match operation.frozen_publish_set {
                None => unread += 1,
                Some(0) => {}
                Some(_) => {
                    voided.insert(name.clone());
                }
            }
        }
    }
    for name in &voided {
        out.push(format!("name=void_frozen_publish point={name} result=point_voided"));
    }
    out.push(format!(
        "name=verdict quantity=void_frozen_publish operations={operations} operations_without_the_reading={unread} points_voided={} voided_points={}",
        voided.len(),
        if voided.is_empty() { "-".to_string() } else { voided.iter().cloned().collect::<Vec<_>>().join(",") }
    ));
    voided
}

/// s37 的标记汇总：每份产物每个点报 pooled_fallback_calls > 0 的操作数；岔路单够判条件落在这些点上的，停下交主 agent（先由设计员给 B8 定这一步的做法）。
pub(super) fn report_missing_the_pooled_fallback_row(points: &Points<'_>, out: &mut Lines) {
    let (mut operations, mut marked) = (0, 0);
    let mut marked_points = BTreeSet::new();
    for (name, point) in points {
        for operation in &point.operations {
            operations += 1;
            if operation.pooled_fallback_calls > 0 {
                marked += 1;
                marked_points.insert(name.clone());
            }
        }
    }
    out.push(format!(
        "name=verdict quantity=missing_pooled_fallback operations={operations} operations_with_pooled_fallback_calls={marked} points_marked={} marked_points={}",
        marked_points.len(),
        if marked_points.is_empty() { "-".to_string() } else { marked_points.into_iter().collect::<Vec<_>>().join(",") }
    ));
}

// ---- 行三：Q7 尖峰成因、Q8 重现条件 ----
/// Q7 的 H1：首次合并走在释放站点、新拉进来的占 final_nodes 至少 80%、年龄中位数至少 100 个 settle_call；交回成不成立与不满足的那几条。
pub(super) fn release_chain_cause_holds(site: &str, pulled: u64, final_nodes: u64, age_median: u64) -> (bool, Vec<&'static str>) {
    let mut failing = Vec::new();
    if site != "release_not_covered" {
        failing.push("site");
    }
    if final_nodes == 0 || (pulled as f64) < RELEASE_CHAIN_PULLED_SHARE_AT_LEAST * final_nodes as f64 {
        failing.push("pulled_share");
    }
    if age_median < RELEASE_CHAIN_AGE_MEDIAN_AT_LEAST {
        failing.push("age_median");
    }
    (failing.is_empty(), failing)
}

pub(super) fn report_spike_causes(points: &Points<'_>, out: &mut Lines) {
    let (mut flagged, mut holding) = (0, 0);
    for point in points.values() {
        for operation in point.operations.iter().filter(|operation| operation.flagged) {
            let Some(settle) = operation.peak_settle.and_then(|index| point.item.window.get(index)) else {
                continue;
            };
            let (holds, failing) = release_chain_cause_holds(&settle.first_merge_site, settle.pulled_nodes_first_merge, settle.final_nodes, settle.pulled_age_median);
            flagged += 1;
            holding += u64::from(holds);
            out.push(format!(
                "name=q7_h1 item={} state={} operation={} operation_index={} first_merge_site={} seed_nodes_first_merge={} pulled_nodes_first_merge={} final_nodes={} pulled_share={:.3} pulled_age_minimum={} pulled_age_median={} pulled_age_maximum={} release_chain_depth={} rounds={} holds={holds} failing={}",
                point.item.name,
                operation.state,
                operation.operation_name,
                operation.operation_index,
                settle.first_merge_site,
                settle.seed_nodes_first_merge,
                settle.pulled_nodes_first_merge,
                settle.final_nodes,
                if settle.final_nodes == 0 { 0.0 } else { settle.pulled_nodes_first_merge as f64 / settle.final_nodes as f64 },
                settle.pulled_age_minimum,
                settle.pulled_age_median,
                settle.pulled_age_maximum,
                settle.release_chain_depth,
                settle.rounds,
                if failing.is_empty() { "-".to_string() } else { failing.join(",") }
            ));
        }
    }
    out.push(format!("name=verdict quantity=q7 flagged_operations={flagged} stale_chain_cause_holds_on={holding} stale_chain_cause_fails_on={}", flagged - holding));
}

fn flagged_at(point: &PointReadings<'_>, state: &str, operation_name: &str, operation_index: u64) -> Option<bool> {
    point
        .operations
        .iter()
        .find(|operation| operation.state == state && operation.operation_name == operation_name && operation.operation_index == operation_index)
        .map(|operation| operation.flagged)
}

/// Q8：四个探针项各列 P1、W1 第 1 次在不在旗上、W1–W4 里第一个旗上操作、W5–W7 的旗上操作与相邻两个之间隔几次固定点、重挂载后 W1 第 1 次。
pub(super) fn report_reproduction_probes(points: &Points<'_>, out: &mut Lines) {
    let mut outcome: BTreeMap<&str, (Option<bool>, Option<bool>)> = BTreeMap::new();
    for (probe, name) in [
        ("standard", "files_1000_units_100"),
        ("cold_first", "files_1000_units_100__head__cold_first"),
        ("create_first", "files_1000_units_100__head__create_first"),
        ("young_first", "files_1000_units_100__head__young_first"),
    ] {
        let Some(point) = points.get(name) else {
            out.push(format!("name=q8_probe probe={probe} item=unread"));
            continue;
        };
        let same: Vec<&Operation> = point.operations.iter().filter(|operation| operation.state == "same_mount").collect();
        let first_flag = same.iter().find(|operation| operation.flagged && matches!(operation.operation_name.as_str(), "W1" | "W2" | "W3" | "W4"));
        let extended: Vec<(&Operation, usize)> = same
            .iter()
            .filter(|operation| operation.flagged && matches!(operation.operation_name.as_str(), "W5" | "W6" | "W7"))
            .filter_map(|operation| operation.peak_settle.map(|index| (*operation, index)))
            .collect();
        let gaps: Vec<String> = extended.windows(2).map(|pair| (pair[1].1 - pair[0].1).to_string()).collect();
        let prefix = flagged_at(point, "same_mount", "P1", 1);
        let first_overwrite = flagged_at(point, "same_mount", "W1", 1);
        outcome.insert(probe, (prefix, first_overwrite));
        out.push(format!(
            "name=q8_probe probe={probe} item={} probe_prefix_flagged={} first_file_first_overwrite_flagged={} first_flag_in_the_base_window={} extended_flags={} settles_between_extended_flags={} remounted_first_file_first_overwrite_flagged={}",
            point.item.name,
            prefix.map_or("none".to_string(), |flag| flag.to_string()),
            first_overwrite.map_or("unread".to_string(), |flag| flag.to_string()),
            first_flag.map_or("none".to_string(), |operation| format!("{}#{}", operation.operation_name, operation.operation_index)),
            if extended.is_empty() { "-".to_string() } else { extended.iter().map(|(operation, _)| format!("{}#{}", operation.operation_name, operation.operation_index)).collect::<Vec<_>>().join(",") },
            if gaps.is_empty() { "-".to_string() } else { gaps.join(",") },
            flagged_at(point, "remounted", "W1", 1).map_or("unread".to_string(), |flag| flag.to_string())
        ));
    }
    out.push(format!("name=verdict quantity=q8 outcome={}", probe_outcome(&outcome)));
}

/// Q8 的判读线（第六节末表）：H1、H2、H3 或都不是；三个探针项缺一个记 `unread`。
pub(super) fn probe_outcome(outcome: &BTreeMap<&str, (Option<bool>, Option<bool>)>) -> &'static str {
    let get = |probe: &str| outcome.get(probe).copied();
    let (Some(cold), Some(create), Some(young)) = (get("cold_first"), get("create_first"), get("young_first")) else {
        return "unread";
    };
    let probes = [cold, create, young];
    if probes.iter().any(|(prefix, first)| prefix.is_none() || first.is_none()) {
        return "unread";
    }
    let flag = |pair: (Option<bool>, Option<bool>)| (pair.0.unwrap_or(false), pair.1.unwrap_or(false));
    let (cold, create, young) = (flag(cold), flag(create), flag(young));
    if cold == (true, false) && create == (false, true) && young == (false, true) {
        "h1"
    } else if [cold, create, young].iter().all(|pair| *pair == (true, false)) {
        "h2"
    } else if [cold, create, young].iter().all(|pair| *pair == (false, true)) {
        "h3"
    } else {
        "none_of_the_three"
    }
}

/// Q9（附带）：`files_then_big_10000` 每个（状态、操作、第几次）的步数 ÷ `files_10000` 同一格的步数，取最大。
pub(super) fn report_then_big_ratio(points: &Points<'_>, out: &mut Lines) {
    let (Some(then_big), Some(files)) = (points.get("files_then_big_10000"), points.get("files_10000")) else {
        out.push("name=verdict quantity=q9 maximum_ratio=unread exceeds=unread".to_string());
        return;
    };
    let mut best: Option<(f64, String)> = None;
    for operation in &then_big.operations {
        let other = files.operations.iter().find(|candidate| {
            (candidate.state.as_str(), candidate.operation_name.as_str(), candidate.operation_index) == (operation.state.as_str(), operation.operation_name.as_str(), operation.operation_index)
        });
        if let (Some(steps), Some(Some(base))) = (operation.steps, other.map(|other| other.steps)) {
            if base > 0 {
                let ratio = steps as f64 / base as f64;
                if best.as_ref().is_none_or(|(value, _)| ratio > *value) {
                    best = Some((ratio, format!("{}/{}#{}", operation.state, operation.operation_name, operation.operation_index)));
                }
            }
        }
    }
    match best {
        Some((ratio, at)) => out.push(format!("name=verdict quantity=q9 maximum_ratio={ratio:.3} at={at} exceeds={}", ratio > THEN_BIG_RATIO_AT_MOST)),
        None => out.push("name=verdict quantity=q9 maximum_ratio=unread exceeds=unread".to_string()),
    }
}

// ---- 作废条款、停机条款与失败条款里能从产物核的几样 ----
/// 作废条款第 5 条：有 item_done、声明行数等于量窗操作数（E207 产物）、没被拒；F4：没走满、没撞上界、没有条款违例。
/// 第七节 ① 第 2、3 条（F4）：重写集只增不减；定下那一轮没有新弄脏、有记录的节点对得上。照产物的逐轮读数另算一遍（产物里实现那一侧另打
/// `clause_violation` 行，两边各报）；交回 (重写集比上一轮小的轮数, 定下那一轮不干净的轮数)。
pub(super) fn fixed_point_premise_breaks(settles: &[Settle]) -> (u64, u64) {
    let (mut shrank, mut settled_unclean) = (0, 0);
    for settle in settles {
        let mut size_before = 0;
        for round in &settle.detail {
            if round.rewrite_set_size < size_before {
                shrank += 1;
            }
            size_before = round.rewrite_set_size;
            if round.exit == "settled" && (round.release_dirty_new + round.placement_dirty_new != 0 || round.holding_differs) {
                settled_unclean += 1;
            }
        }
    }
    (shrank, settled_unclean)
}

/// 停机条款第 2 条：轮功 `W`（settle 行的 `round_work`）与逐轮重写角色数之和 `Σ_r A_r`（round 行的 `roles_rewritten`）对不上的固定点个数；
/// 没有逐轮读数的（历史那一段只有第 1 组）不比。
pub(super) fn round_work_mismatches(settles: &[Settle]) -> u64 {
    let mismatched = settles
        .iter()
        .filter(|settle| !settle.detail.is_empty() && settle.detail.iter().map(|round| round.roles_rewritten).sum::<u64>() != settle.round_work)
        .count();
    u64::try_from(mismatched).unwrap_or(u64::MAX)
}

/// 第五节「量窗」：一个状态里每种操作该有几次。W1 32、W2 32、W3 8、W4 8；探针项同一次挂载多一次 P1；带扩展窗口的点同一次挂载另有
/// W5 200、W6 110（10 批，每批建一次、写十次）、W7 11（第一个文件 1 次、冷文件 5 个、新文件 5 个）；对账点两种状态末尾各一次阴性对照。
pub(super) const WINDOW_OPERATIONS: [(&str, u64); 4] = [("W1", 32), ("W2", 32), ("W3", 8), ("W4", 8)];
pub(super) const EXTENDED_WINDOW_OPERATIONS: [(&str, u64); 3] = [("W5", 200), ("W6", 110), ("W7", 11)];
const EXTENDED_WINDOW_POINTS: [&str; 2] = ["files_1000_units_100", "files_30000"];
const NEGATIVE_CONTROL_POINTS: [&str; 3] = ["base", "files_1000", "files_10000"];

fn expected_window(point: &str, probe: &str, state: &str) -> BTreeMap<&'static str, u64> {
    let mut expected: BTreeMap<&'static str, u64> = WINDOW_OPERATIONS.into_iter().collect();
    if state == "same_mount" && probe != "standard" {
        expected.insert("P1", 1);
    }
    if state == "same_mount" && EXTENDED_WINDOW_POINTS.contains(&point) {
        expected.extend(EXTENDED_WINDOW_OPERATIONS);
    }
    if NEGATIVE_CONTROL_POINTS.contains(&point) {
        expected.insert("negative_control", 1);
    }
    expected
}

/// 作废条款第 5 条的一半（读不到不算读到 0）：每个状态里每种操作实际有几次（不同的第几次）与该有的比，交回对不上的 `<状态>/<操作>:<有>≠<该有>`。
pub(super) fn window_shape_mismatches(point: &PointReadings<'_>) -> Vec<String> {
    let mut mismatches = Vec::new();
    for state in ["same_mount", "remounted"] {
        let mut seen: BTreeMap<&str, u64> = BTreeMap::new();
        for operation in point.operations.iter().filter(|operation| operation.state == state) {
            *seen.entry(operation.operation_name.as_str()).or_default() += 1;
        }
        let expected = expected_window(&point.item.point, &point.item.probe, state);
        let names: BTreeSet<&str> = seen.keys().copied().chain(expected.keys().copied()).collect();
        for name in names {
            let (have, want) = (seen.get(name).copied().unwrap_or(0), expected.get(name).copied().unwrap_or(0));
            if have != want {
                mismatches.push(format!("{state}/{name}:{have}≠{want}"));
            }
        }
    }
    mismatches
}

pub(super) fn report_item_checks(point: &PointReadings<'_>, out: &mut Lines) {
    let item = point.item;
    let (rewrite_set_shrank, settled_round_unclean) = fixed_point_premise_breaks(&item.window);
    let window_shape = if item.arm == "plain" { Vec::new() } else { window_shape_mismatches(point) };
    let declared_count = item.declared.len();
    let operations = point.operations.len();
    let did_not_settle = item.window.iter().filter(|settle| settle.did_not_settle()).count();
    let at_the_limit = item.window.iter().filter(|settle| settle.rounds >= SETTLING_ROUNDS_LIMIT).count();
    let constants_match = number(&item.constants, "leaf_slots") == Some(LEAF_SLOTS)
        && number(&item.constants, "internal_fanout") == Some(INTERNAL_FANOUT)
        && number(&item.constants, "settling_rounds_at_most") == Some(SETTLING_ROUNDS_LIMIT)
        && number(&item.constants, "disks") == Some(DISKS);
    let negative_rounds = item.window.iter().filter(|settle| settle.operation_name == "negative_control").map(|settle| settle.rounds).max();
    out.push(format!(
        "name=verdict quantity=item_checks item={} item_done={} refused={} declared_lines={declared_count} window_operations={operations} declared_lines_match={} constants_match={} did_not_settle={did_not_settle} rounds_at_the_limit={at_the_limit} clause_violations={} rewrite_set_shrank={rewrite_set_shrank} settled_round_unclean={settled_round_unclean} round_work_mismatches={} window_shape_ok={} window_shape_mismatches={} negative_control_rounds={} negative_control_ok={}",
        item.name,
        item.done,
        item.refused.as_deref().unwrap_or("none"),
        declared_count == operations,
        constants_match || item.arm == "plain",
        item.clause_violations.len(),
        round_work_mismatches(&item.window),
        window_shape.is_empty(),
        if window_shape.is_empty() { "-".to_string() } else { window_shape.join(",") },
        negative_rounds.map_or("not_in_this_point".to_string(), |rounds| rounds.to_string()),
        negative_rounds.is_none_or(|rounds| rounds <= NEGATIVE_CONTROL_ROUNDS_AT_MOST)
    ));
}

/// 第七节 ② 第 3 条：尖峰复现钉。
pub(super) fn report_spike_pin(points: &Points<'_>, out: &mut Lines) {
    let Some(point) = points.get("files_1000_units_100") else {
        out.push("name=verdict quantity=spike_pin matches=not_run".to_string());
        return;
    };
    let settle_of = |operation_index: u64| point.item.window.iter().find(|settle| settle.state == "same_mount" && settle.operation_name == "W1" && settle.operation_index == operation_index);
    let value = |settle: &Settle, key: &str| match key {
        "rounds" => settle.rounds,
        "final_roles" => settle.final_roles,
        "final_nodes" => settle.final_nodes,
        "round_work" => settle.round_work,
        _ => settle.closure_nodes_added,
    };
    let mut mismatches = Vec::new();
    for (operation_index, pins) in [(1, &SPIKE_PIN_FIRST[..]), (2, &SPIKE_PIN_SECOND[..])] {
        match settle_of(operation_index) {
            Some(settle) => mismatches.extend(
                pins.iter().filter(|(key, pinned)| value(settle, key) != *pinned).map(|(key, pinned)| format!("W1#{operation_index}.{key}={}≠{pinned}", value(settle, key))),
            ),
            None => mismatches.push(format!("W1#{operation_index}=unread")),
        }
    }
    let state = point.item.judgements.get("same_mount/W1/1/").map(|judgement| judgement.state.clone()).unwrap_or_default();
    mismatches.extend(
        SPIKE_PIN_STATE.iter().filter(|(key, pinned)| number(&state, key) != Some(*pinned)).map(|(key, pinned)| format!("state.{key}={:?}≠{pinned}", number(&state, key))),
    );
    out.push(format!(
        "name=verdict quantity=spike_pin matches={} mismatches={}",
        mismatches.is_empty(),
        if mismatches.is_empty() { "-".to_string() } else { mismatches.join(",") }
    ));
}

/// 第七节 ② 第 4 条与 F8：新点每个状态里第一次碰、仍是 K 个单元的 W2 操作，声明的单元差与 `mapped_units` 差之比落在 [0.5, 2]；这种操作至少 5 个。
pub(super) fn report_declared_against_mapped(points: &Points<'_>, out: &mut Lines) {
    for (name, point) in points.iter().filter(|(_, point)| point.item.units_per_small_file >= LARGE_CHANGE_UNITS_AT_LEAST && point.item.arm == "head") {
        for state in ["same_mount", "remounted"] {
            let in_state: Vec<&Operation> = point.operations.iter().filter(|operation| operation.state == state).collect();
            let mut first_touches = 0;
            let mut outside = Vec::new();
            for (position, operation) in in_state.iter().enumerate() {
                if operation.operation_name != "W2" || operation.old_units != point.item.units_per_small_file {
                    continue;
                }
                first_touches += 1;
                let after = in_state.get(position + 1).and_then(|next| next.mapped_units);
                let ratio = operation.mapped_units.zip(after).and_then(|(before, after)| {
                    (before > after).then(|| (operation.old_units - operation.new_units) as f64 / (before - after) as f64)
                });
                if !ratio.is_some_and(|ratio| (DECLARED_TO_MAPPED_AT_LEAST..=DECLARED_TO_MAPPED_AT_MOST).contains(&ratio)) {
                    outside.push(format!("W2#{}:{}", operation.operation_index, ratio.map_or("unread".to_string(), |ratio| format!("{ratio:.3}"))));
                }
            }
            out.push(format!(
                "name=verdict quantity=declared_against_mapped point={name} state={state} first_touches={first_touches} enough_first_touches={} ratios_outside={} outside={}",
                first_touches >= FIRST_TOUCHES_AT_LEAST,
                outside.len(),
                if outside.is_empty() { "-".to_string() } else { outside.join(",") }
            ));
        }
    }
}

/// 作废条款第 3 条：`plain` 与带计数的构建历史末 `image_sha256` 相同。
pub(super) fn report_image_pins(points: &Points<'_>, out: &mut Lines) {
    for name in ["base", "files_1000"] {
        let head = points.get(name).and_then(|point| point.item.image_sha256_at_the_history_end.clone());
        let plain = points.get(&format!("{name}__plain")).and_then(|point| point.item.image_sha256_at_the_history_end.clone());
        let verdict = match (&head, &plain) {
            (Some(head), Some(plain)) => (head == plain).to_string(),
            _ => "not_run".to_string(),
        };
        out.push(format!("name=verdict quantity=plain_image point={name} same_image={verdict}"));
    }
}

/// 第七节 ② 第 6 条（第四段）：历史末（量窗第一次操作之前）的 `mapped_units` 落在区间里；另报每点的 user_units ÷ mapped_units（第八节）。
pub(super) fn report_unit_counts(points: &Points<'_>, out: &mut Lines) {
    for (name, point) in points.iter().filter(|(_, point)| point.item.arm == "head") {
        let history_end = point.operations.iter().find(|operation| operation.state == "same_mount").and_then(|operation| operation.mapped_units);
        let share = history_end.map(|mapped| point.item.units_per_small_file as f64 / mapped as f64);
        out.push(format!(
            "name=occupancy point={name} units_per_small_file={} mapped_units_at_the_history_end={} change_share={} far_below_the_total={}",
            point.item.units_per_small_file,
            history_end.map_or("unread".to_string(), |value| value.to_string()),
            share.map_or("unread".to_string(), |share| format!("{share:.4}")),
            share.map_or("unread".to_string(), |share| (share <= FAR_BELOW_THE_TOTAL_AT_MOST).to_string())
        ));
        if let Some((_, low, high)) = BIG_POINT_MAPPED_UNITS.iter().find(|(big, _, _)| *big == name.as_str()) {
            out.push(format!(
                "name=verdict quantity=big_point_units point={name} mapped_units={} inside={}",
                history_end.map_or("unread".to_string(), |value| value.to_string()),
                history_end.map_or("not_run".to_string(), |value| (*low..=*high).contains(&value).to_string())
            ));
        }
    }
}

/// 第七节 ② 第 2 条：今天的树对 E206 的 p2a 产物，三个对账点 W1–W4 每个固定点（按状态、操作、第几次、第几次判对上）的五项逐个相同。
pub(super) fn report_reconciliation(points: &Points<'_>, retro: &Points<'_>, out: &mut Lines) {
    for name in ["base", "files_1000", "files_10000"] {
        let (Some(today), Some(before)) = (points.get(name), retro.get(name)) else {
            out.push(format!("name=verdict quantity=reconciliation_with_e206 point={name} matches=not_run"));
            continue;
        };
        let in_window = |settle: &&Settle| matches!(settle.operation_name.as_str(), "W1" | "W2" | "W3" | "W4");
        let keyed = |item: &PointArmReadings| -> BTreeMap<String, (u64, u64, u64, u64, String)> {
            item.window
                .iter()
                .filter(in_window)
                .map(|settle| (settle.publish_key(), (settle.rounds, settle.final_roles, settle.final_nodes, settle.round_work, settle.final_set_hash.clone())))
                .collect()
        };
        let (today_settles, before_settles) = (keyed(today.item), keyed(before.item));
        let keys: BTreeSet<&String> = today_settles.keys().chain(before_settles.keys()).collect();
        let differing: Vec<&String> = keys.into_iter().filter(|key| today_settles.get(*key) != before_settles.get(*key)).collect();
        out.push(format!(
            "name=verdict quantity=reconciliation_with_e206 point={name} settles_today={} settles_e206={} differing={} matches={} first_differing={}",
            today_settles.len(),
            before_settles.len(),
            differing.len(),
            differing.is_empty() && !today_settles.is_empty(),
            differing.first().map_or("-", |key| key.as_str())
        ));
    }
}

/// 回扫（Q4）的另一份：`data_units` 轴 W2 第 1 次（同一次挂载）步数 ÷ user_units，标 `looser=yes`，同时报原步数的 α。
pub(super) fn report_retro_data_units_first_cold(retro: &Points<'_>, out: &mut Lines) {
    let ends: Vec<Option<End>> = RETRO_SETS[1]
        .points
        .iter()
        .map(|name| {
            let point = retro.get(*name)?;
            let operation = point.operations.iter().find(|operation| operation.state == "same_mount" && operation.operation_name == "W2" && operation.operation_index == 1)?;
            Some((operation.steps?, denominator(operation)?, measured_total(operation)?))
        })
        .collect();
    let reading = ends[0].zip(ends[3]).map(|(low, high)| two_point_reading(low, high));
    out.push(format!(
        "name=q4_data_units_first_cold_overwrite ends={} looser={} alpha={} band={} raw_alpha={} raw_band={}",
        ends.iter().map(|end| end.map_or("unread".to_string(), |end| format!("{}/{}/{}", end.0, end.1, end.2))).collect::<Vec<_>>().join(","),
        reading.map_or("unread", |reading| if reading.looser { "yes" } else { "no" }),
        alpha_text(reading.and_then(|reading| reading.alpha)),
        band_text(reading.and_then(|reading| reading.alpha)),
        alpha_text(reading.and_then(|reading| reading.raw_alpha)),
        band_text(reading.and_then(|reading| reading.raw_alpha))
    ));
}

/// 第七节 ② 第 1 条：照旧判据（旋钮 x、原步数）重算两行，与登记抄下的逐字符比；交回两行都相同没有。
/// 旧判据的一行（E206 合并报告 `q8_*` 行的写法，轴位置 10、100、1000、10000）：四个点的量，读不到的写 `unread`。
pub(super) fn old_criterion_line(name: &str, axis: &str, values: &[Option<f64>; 4]) -> String {
    let slopes: Vec<String> = (0..3)
        .map(|index| match (values[index], values[index + 1]) {
            (Some(low), Some(high)) if low > 0.0 && high > 0.0 => format!("{:.3}", alpha(low, high, AXIS_POSITIONS[index], AXIS_POSITIONS[index + 1])),
            _ => "unread".to_string(),
        })
        .collect();
    let growth = match (values[0], values[3]) {
        (Some(low), Some(high)) if low > 0.0 && high > 0.0 => Some(alpha_on_the_axis(low, high)),
        _ => None,
    };
    format!(
        "name={name} arm=p2a axis={axis} low_end={} high_end={} alpha={} band={} local_slopes={}",
        values[0].map_or("unread".to_string(), |value| format!("{value:.0}")),
        values[3].map_or("unread".to_string(), |value| format!("{value:.0}")),
        growth.map_or("unread".to_string(), |value| format!("{value:.3}")),
        growth.map_or("unread", |value| band(value).tag()),
        slopes.join(",")
    )
}

/// 第七节 ② 第 1 条：照旧判据（旋钮 x、原步数）重算两行，与登记抄下的逐字符比；交回两行都相同没有。
pub(super) fn old_criterion_pins(retro_items: &[PointArmReadings], out: &mut Lines) -> bool {
    let items: ReadingsByPointAndArm<'_> = retro_items.iter().map(|item| ((item.point.clone(), item.arm.clone()), item)).collect();
    let values_of = |axis: &str, quantity: &dyn Fn(&PointArmReadings) -> Option<f64>| -> [Option<f64>; 4] {
        let points = AXES.iter().find(|(candidate, _)| *candidate == axis).map(|(_, points)| *points).expect("轴在表里");
        let mut values = [None; 4];
        for (value, point) in values.iter_mut().zip(points.iter()) {
            *value = items.get(&((*point).to_string(), "p2a".to_string())).and_then(|item| quantity(item));
        }
        values
    };
    let first = old_criterion_line(
        "q8_first_publish",
        "files",
        &values_of("files", &|item| window_settle(item, "W1", 1).and_then(|settle| by_delta_of_settle(item, settle)).map(|steps| steps as f64)),
    );
    let peak = old_criterion_line(
        "q8_peak",
        "data_units",
        &values_of("data_units", &|item| item.window.iter().filter_map(|settle| by_delta_of_settle(item, settle)).max().map(|steps| steps as f64)),
    );
    let (first_matches, peak_matches) = (first == OLD_CRITERION_FIRST_PUBLISH_FILES, peak == OLD_CRITERION_PEAK_DATA_UNITS);
    out.push(first);
    out.push(peak);
    out.push(format!("name=verdict quantity=old_criterion_pins first_publish_files_matches={first_matches} peak_data_units_matches={peak_matches}"));
    first_matches && peak_matches
}
