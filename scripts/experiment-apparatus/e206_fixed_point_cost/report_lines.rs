//! 逐点逐臂的报告行：轴行、报告项、实测锚点、对照、相对今日的臂、敏感性、判定与总报告、E203 的判定。

use e7_index_bench::i64_from_u64;
use super::*;

type ReadingsByPointAndArm<'items> = BTreeMap<(String, String), &'items PointArmReadings>;

fn settles_list(item: &PointArmReadings) -> Vec<&Settle> {
    all_settles(item).collect()
}

/// 一根轴上一个量的 α 与相邻两点局部斜率；端点读不到报 `unread`。
fn axis_line(
    name: &str,
    arm: &str,
    axis: &str,
    points: &[&str; 4],
    items: &ReadingsByPointAndArm<'_>,
    quantity: &dyn Fn(&PointArmReadings) -> Option<f64>,
) -> Option<f64> {
    let values: Vec<Option<f64>> = points
        .iter()
        .map(|point| {
            items
                .get(&((*point).to_string(), arm.to_string()))
                .and_then(|item| quantity(item))
        })
        .collect();
    let slopes: Vec<String> = (0..3)
        .map(|index| match (values[index], values[index + 1]) {
            (Some(low), Some(high)) if low > 0.0 && high > 0.0 => format!(
                "{:.3}",
                alpha(low, high, AXIS_POSITIONS[index], AXIS_POSITIONS[index + 1])
            ),
            _ => "unread".to_string(),
        })
        .collect();
    let growth = match (values[0], values[3]) {
        (Some(low), Some(high)) if low > 0.0 && high > 0.0 => Some(alpha_on_the_axis(low, high)),
        _ => None,
    };
    println!(
        "name={name} arm={arm} axis={axis} low_end={} high_end={} alpha={} band={} local_slopes={}",
        values[0].map_or("unread".to_string(), |value| format!("{value:.0}")),
        values[3].map_or("unread".to_string(), |value| format!("{value:.0}")),
        growth.map_or("unread".to_string(), |value| format!("{value:.3}")),
        growth.map_or("unread", |value| band(value).tag()),
        slopes.join(",")
    );
    growth
}

/// 一次固定点的 B8 合计，P1、P2a 照自己的读数，P3 只换每轮那几行（Q8）；读不到交 `None`。
fn by_delta_of_settle(item: &PointArmReadings, settle: &Settle) -> Option<(u64, u64)> {
    let judgement = judgement_of(item, settle)?;
    let total = by_delta_of_the_judgement(&judgement, item.unit_area_slots).ok()?;
    let between = by_delta_input(&judgement, "between", item.unit_area_slots).ok()?;
    let between_rounds_pricing_total = total - by_delta_rows_that_between_rounds_pricing_changes(&between)
        + between_rounds_pricing_reading(settle, &between, item.unit_bytes).sum_of_rounds;
    Some((total, between_rounds_pricing_total))
}

/// 一份产物窗口里各次固定点在准入之前三段里 s37 的 (读到几次判, pooled_fallback_calls > 0 的几次判, pooled_fallback_calls 之和, s37 步数之和, s29 步数之和)。
/// E206 的产物是 s37 进计数装置之前量的，读不到那一行时全是 0；pooled_fallback_calls > 0 的判，B8 合计标「缺 s37 一行，只对清单内的步」。
pub(super) fn pooled_fallback_of_the_window(item: &PointArmReadings) -> (u64, u64, u64, u64, u64) {
    let (mut read, mut marked, mut calls, mut steps, mut per_disk_fallback_steps) = (0, 0, 0, 0, 0);
    for settle in &item.window {
        let Some(judgement) = judgement_of(item, settle) else {
            continue;
        };
        let (judgement_calls, judgement_steps, judgement_per_disk_fallback_steps) = pooled_fallback_of_the_judgement(&judgement);
        read += 1;
        marked += u64::from(judgement_calls > 0);
        calls += judgement_calls;
        steps += judgement_steps;
        per_disk_fallback_steps += judgement_per_disk_fallback_steps;
    }
    (read, marked, calls, steps, per_disk_fallback_steps)
}

/// 一个点一条臂的逐项报数（读法第 7 条：第 1 次、第 2–32 次窗内、历史内各一份；第八节轨迹）。
fn report_item(item: &PointArmReadings) {
    let peaks = peaks_of(all_settles(item));
    let history = peaks_of(item.history.iter());
    let first: Vec<&Settle> = item
        .window
        .iter()
        .filter(|settle| settle.state == "same_mount" && settle.operation == "W1" && settle.operation_index == 1)
        .collect();
    let rest = peaks_of(
        item.window
            .iter()
            .filter(|settle| settle.operation == "W1" && settle.operation_index >= 2),
    );
    let first_rounds: Vec<String> = first
        .iter()
        .map(|settle| settle.rounds.to_string())
        .collect();
    println!(
        "name=q1 item={} point={} arm={} peak_rounds={} at={} verdict={} first_overwrite_rounds={} window_rest_peak_rounds={} history_peak_rounds={}",
        item.name, item.point, item.arm, peaks.rounds, peaks.rounds_at, rounds_verdict(peaks.rounds, peaks.did_not_settle),
        first_rounds.join(","), rest.rounds, history.rounds
    );
    println!(
        "name=peaks item={} peak_round_work={} peak_final_roles={} peak_final_nodes={} peak_effective_rounds={:.3}",
        item.name, peaks.round_work, peaks.final_roles, peaks.final_nodes, peaks.effective_rounds
    );
    for (state, first_state) in [("same_mount", "same_mount"), ("remounted", "remounted")] {
        let sequence: Vec<u64> = item
            .window
            .iter()
            .filter(|settle| settle.state == first_state)
            .map(|settle| settle.rounds)
            .collect();
        let at_least = |floor: u64| sequence.iter().filter(|rounds| **rounds >= floor).count();
        let (falls_back, recurs) = fallback_and_recurrence(&sequence);
        let fourth = item
            .window
            .iter()
            .filter(|settle| settle.state == first_state && settle.operation == "W4")
            .all(|settle| settle.rounds <= 4);
        println!(
            "name=q9 item={} state={state} publishes={} peak={} rounds_at_least_3={} rounds_at_least_5={} rounds_at_least_9={} last={} falls_back={} last_window_at_most_4={fourth} recurs={recurs}",
            item.name, sequence.len(), sequence.iter().max().copied().unwrap_or(0), at_least(3), at_least(5), at_least(9),
            sequence.last().copied().unwrap_or(0), falls_back && fourth
        );
    }
    for settle in item.window.iter().filter(|settle| settle.rounds >= 9) {
        println!(
            "name=q10_many_rounds item={} at={}/{}/{} rounds={} release_chain_depth={} release_chain_nodes={} active_leaves={:?} rewritten_nodes={}",
            item.name, settle.state, settle.operation, settle.operation_index, settle.rounds, settle.release_chain_depth, settle.release_chain_nodes, settle.active_leaves, settle.final_nodes
        );
    }
    println!(
        "name=q10 item={} chain_explained_ratio={} release_exit_share={:.3} placement_exit_share={:.3}",
        item.name,
        chain_explained_ratio(&item.window).map_or("not_seen".to_string(), |ratio| format!("{ratio:.3}")),
        exit_share(&item.window, "release_not_covered"),
        exit_share(&item.window, "placement_not_covered")
    );
    println!(
        "name=f2 item={} rounds_at_least_5={}",
        item.name,
        rounds_at_least(item, 5)
    );
    let shrinks = item
        .clause_violations
        .iter()
        .filter(|clause| clause.as_str() == "rewrite_set_only_grows")
        .count();
    println!(
        "name=clause_checks item={} rewrite_set_shrank={shrinks} settled_round_not_covered={}",
        item.name,
        item.clause_violations.len() - shrinks
    );
    let (read, marked, calls, steps, per_disk_fallback_steps) = pooled_fallback_of_the_window(item);
    println!(
        "name=by_delta_pooled_fallback item={} judgements_read={read} judgements_with_pooled_fallback_calls={marked} pooled_fallback_calls={calls} pooled_fallback_steps={steps} per_disk_fallback_steps={per_disk_fallback_steps} by_delta_arms_total={}",
        item.name,
        if marked > 0 { "missing_pooled_fallback_row_only_for_the_listed_steps" } else { "complete" }
    );
    let (mut largest, mut sum, mut memory, mut maintenance, mut ratio) =
        (0_u64, 0_u64, 0_u64, 0_u64, 0.0_f64);
    let mut unread = 0;
    for settle in &item.window {
        let between = judgement_of(item, settle)
            .and_then(|judgement| by_delta_input(&judgement, "between", item.unit_area_slots).ok());
        let Some(between) = between else {
            unread += 1;
            continue;
        };
        let reading = between_rounds_pricing_reading(settle, &between, item.unit_bytes);
        largest = largest.max(reading.largest_round);
        sum = sum.max(reading.sum_of_rounds);
        memory = memory.max(reading.memory_bytes);
        maintenance = maintenance.max(reading.maintenance_steps);
        ratio = ratio.max(premise_ratio(settle));
    }
    println!(
        "name=q7 item={} largest_round={largest} sum_of_rounds={sum} sum_over_final_roles={:.3} premise_ratio={ratio:.3} premise={} memory_bytes={memory} maintenance_steps={maintenance} unread={unread}",
        item.name,
        sum as f64 / peaks.final_roles.max(1) as f64,
        if ratio <= PREMISE_RATIO_AT_MOST { "holds" } else { "fails" }
    );
    let by_delta: Vec<(u64, u64, &Settle)> = item
        .window
        .iter()
        .filter_map(|settle| {
            by_delta_of_settle(item, settle).map(|(today, between_rounds_pricing_total)| (today, between_rounds_pricing_total, settle))
        })
        .collect();
    for (state, first_state) in [("same_mount", "same_mount"), ("remounted", "remounted")] {
        let in_state: Vec<&(u64, u64, &Settle)> = by_delta
            .iter()
            .filter(|(_, _, settle)| settle.state == first_state)
            .collect();
        println!(
            "name=q8 item={} state={state} first_publish={} peak={} between_rounds_first_publish={} between_rounds_peak={} unread={}",
            item.name,
            in_state.first().map_or("unread".to_string(), |reading| reading.0.to_string()),
            in_state.iter().map(|reading| reading.0).max().map_or("unread".to_string(), |value| value.to_string()),
            in_state.first().map_or("unread".to_string(), |reading| reading.1.to_string()),
            in_state.iter().map(|reading| reading.1).max().map_or("unread".to_string(), |value| value.to_string()),
            item.window.len() - by_delta.len()
        );
    }
    let (compared, different) = between_rounds_pricing_control_gap(item);
    println!(
        "name=control_p3 item={} compared={compared} different={different} holds={}",
        item.name,
        compared > 0 && different == 0
    );
    for (operation, image_slots, slots_placed) in &item.reconciliations {
        println!("name=reconciliation item={} op={operation} image_changed_unit_slots={image_slots} slots_placed={slots_placed} equal={}", item.name, reconciliation_holds(*image_slots, *slots_placed));
    }
    for problem in model_mismatches(item) {
        println!("name=stop_condition_2 item={} problem={problem}", item.name);
    }
    if let Some(cause) = &item.refused {
        println!(
            "name=f8 item={} refused={cause} refused_for_not_settling={}",
            item.name,
            cause == "AllocationRecordTreeRewriteSetDidNotSettle"
        );
    }
}

/// 第五节阴性对照的量：量窗末尾那一次的 `final_roles`（Q：`A_final` 在三根轴两端 |α| ≤ 0.1，轮数 ≤ 4）。
fn negative_control_settle(item: &PointArmReadings) -> Option<&Settle> {
    item.window
        .iter()
        .find(|settle| settle.operation == "negative_control")
}

fn window_settle<'items>(
    item: &'items PointArmReadings,
    operation: &str,
    operation_index: u64,
) -> Option<&'items Settle> {
    item.window.iter().find(|settle| {
        settle.state == "same_mount" && settle.operation == operation && settle.operation_index == operation_index
    })
}

/// 第七节 ② 第 1、2 条：不符走停机条款第 3 条，打 `matches=false`，不许默认哪一边对。
fn report_measured_anchors(items: &ReadingsByPointAndArm<'_>) {
    let files_rounds: [u64; 22] = [
        11, 24, 28, 32, 36, 40, 42, 44, 46, 50, 54, 58, 62, 64, 66, 70, 74, 76, 78, 80, 82, 84,
    ];
    let base_rounds = [7_u64, 14];
    let anchors: [(&str, &str, u64, &[u64], u64); 2] = [
        (
            "files_10000",
            "files_10000_first_overwrite",
            22,
            &files_rounds,
            1201,
        ),
        ("base", "base_first_overwrite", 2, &base_rounds, 21),
    ];
    for (point, anchor_identifier, rounds, roles, work) in anchors {
        let Some(item) = items.get(&(point.to_string(), "p1".to_string())) else {
            println!("name=anchor_measured anchor_identifier={anchor_identifier} result=unread");
            continue;
        };
        let Some(settle) = window_settle(item, "W1", 1) else {
            println!("name=anchor_measured anchor_identifier={anchor_identifier} result=unread");
            continue;
        };
        let got: Vec<u64> = settle
            .detail
            .iter()
            .map(|round| round.roles_rewritten)
            .collect();
        println!(
            "name=anchor_measured anchor_identifier={anchor_identifier} rounds={} expected_rounds={rounds} round_work={} expected_round_work={work} per_round_roles={got:?} matches={}",
            settle.rounds,
            settle.round_work,
            anchor_matches(settle, rounds, work, roles)
        );
        if point == "files_10000" {
            let second = window_settle(item, "W1", 2).map(|window_settle_reading| window_settle_reading.rounds);
            println!("name=anchor_measured anchor_identifier=files_10000_second_overwrite rounds={second:?} expected_rounds=2 matches={}", second == Some(2));
        }
    }
}

/// 第五节阳性、阴性对照，与作废条款第 1、2、4、5、7 条的判读。
fn report_controls(items: &ReadingsByPointAndArm<'_>) {
    let item_of =
        |point: &str, arm: &str| items.get(&(point.to_string(), arm.to_string())).copied();
    for point in ["base", "files_100", "files_1000"] {
        let Some(today) = item_of(point, "p1") else {
            continue;
        };
        if let Some(single) = item_of(point, "single_step") {
            let holds = single_step_control_holds(
                &settles_list(single),
                &settles_list(today),
                &single.window,
                &today.window,
            ) && single.image_sha256_at_the_history_end
                == today.image_sha256_at_the_history_end;
            println!("name=control_single_step point={point} holds={holds} voids=p1_round_readings_if_false");
        }
        for (arm, voids) in [("closure_off", "p2a"), ("p2b_never", "p2b")] {
            if let Some(control) = item_of(point, arm) {
                println!(
                    "name=control_{arm} point={point} holds={} voids={voids}_if_false",
                    same_readings(&settles_list(control), &settles_list(today))
                );
            }
        }
    }
    if let Some(whole) = item_of("files_1000", "whole") {
        let holds = whole
            .window
            .iter()
            .filter(|settle| settle.outcome == "settled")
            .all(|settle| settle.final_roles >= settle.previous_nodes);
        println!("name=control_whole point=files_1000 holds={holds} voids=p2b_if_false");
    }
    for point in ["base", "files_1000"] {
        if let (Some(plain), Some(today)) = (item_of(point, "plain"), item_of(point, "p1")) {
            println!(
                "name=void_5 point={point} plain={:?} today={:?} same={} voids=all_measured_if_false",
                plain.image_sha256_at_the_history_end,
                today.image_sha256_at_the_history_end,
                plain.image_sha256_at_the_history_end.is_some() && plain.image_sha256_at_the_history_end == today.image_sha256_at_the_history_end
            );
        }
    }
    for arm in MEASURED_ARMS {
        for (axis, points) in AXES {
            let (low, high) = (item_of(points[0], arm), item_of(points[3], arm));
            let (Some(low), Some(high)) = (low, high) else {
                println!("name=control_negative arm={arm} axis={axis} result=unread");
                continue;
            };
            let (Some(low), Some(high)) =
                (negative_control_settle(low), negative_control_settle(high))
            else {
                println!("name=control_negative arm={arm} axis={axis} result=did_not_reach_the_fixed_point stop_condition=2");
                continue;
            };
            let growth = alpha_on_the_axis(
                low.final_roles.max(1) as f64,
                high.final_roles.max(1) as f64,
            );
            println!(
                "name=control_negative arm={arm} axis={axis} low_final_roles={} high_final_roles={} alpha={growth:.3} rounds={} holds={} voids={arm}_if_false",
                low.final_roles,
                high.final_roles,
                low.rounds.max(high.rounds),
                negative_control_holds(low, high)
            );
        }
    }
}

/// Q6：写放大比值的两档（第六节表后）。
pub(super) fn write_amplification_verdict(ratio: f64) -> &'static str {
    if ratio <= WRITE_AMPLIFICATION_AT_MOST {
        "acceptable"
    } else {
        "expensive"
    }
}

/// Q5：`fixed_point_steps(P2a) + 闭包走今天的线性 pointer_of 的步 − fixed_point_steps(P1)`。
pub(super) fn net_steps_linear(arm: &Settle, base: &Settle) -> i64 {
    i64_from_u64(arm.fixed_point_steps()) + i64_from_u64(arm.closure_lookup_steps)
        - i64_from_u64(base.fixed_point_steps())
}

/// Q5：同上，闭包每一跳按索引记 `lg(节点数)`。
pub(super) fn net_steps_indexed(arm: &Settle, base: &Settle) -> i64 {
    i64_from_u64(arm.fixed_point_steps()) + i64_from_u64(arm.closure_hops * lg(arm.previous_nodes))
        - i64_from_u64(base.fixed_point_steps())
}

/// 作废条款第 4 条：`A_final` 在轴两端 |α| ≤ 0.1，轮数 ≤ 4。
pub(super) fn negative_control_holds(low: &Settle, high: &Settle) -> bool {
    let growth = alpha_on_the_axis(
        low.final_roles.max(1) as f64,
        high.final_roles.max(1) as f64,
    );
    growth.abs() <= 0.1 && low.rounds.max(high.rounds) <= 4
}

/// Q5、Q6、F3、F4：改做法的臂与 P1 同一个点比。
fn report_arm_against_today(item: &PointArmReadings, today: &PointArmReadings) {
    let (same, compared, first_difference) =
        same_final_sets(&settles_list(item), &settles_list(today));
    let same_image = item.image_sha256_at_the_history_end == today.image_sha256_at_the_history_end;
    println!(
        "name=q5_same_final_set item={} same={same} compared={compared} ratio={:.4} first_difference={first_difference:?} same_image_at_the_history_end={same_image} final_set_differs={}",
        item.name,
        same as f64 / compared.max(1) as f64,
        same != compared || !same_image
    );
    let slots = |of: &PointArmReadings| {
        all_settles(of)
            .filter(|settle| settle.outcome == "settled")
            .map(|settle| settle.slots_placed)
            .sum::<u64>()
    };
    let ratio = slots(item) as f64 / slots(today).max(1) as f64;
    println!(
        "name=q6 item={} slots_placed={} today={} ratio={ratio:.4} verdict={} peak_final_roles={}",
        item.name,
        slots(item),
        slots(today),
        write_amplification_verdict(ratio),
        peaks_of(all_settles(item)).final_roles
    );
    let many_rounds: Vec<(&Settle, &Settle)> = item
        .window
        .iter()
        .zip(&today.window)
        .filter(|(_, base)| base.rounds >= 3)
        .collect();
    let net: i64 = many_rounds
        .iter()
        .map(|(arm, base)| net_steps_linear(arm, base))
        .sum();
    let net_by_index: i64 = many_rounds
        .iter()
        .map(|(arm, base)| net_steps_indexed(arm, base))
        .sum();
    println!(
        "name=q5_net_steps item={} publishes_with_at_least_3_rounds={} net_steps_linear={net} net_steps_indexed={net_by_index} linear={} indexed={}",
        item.name,
        many_rounds.len(),
        if net <= 0 { "saves" } else { "costs_more" },
        if net_by_index <= 0 { "saves" } else { "costs_more" }
    );
}

/// 第八节：`<点>:<轮数>:<判定>` 几条读数的判定都相同；不同就记「不稳定」。
pub(super) fn verdicts_agree(readings: &[String]) -> bool {
    let verdicts: BTreeSet<&str> = readings
        .iter()
        .filter_map(|reading| reading.rsplit(':').next())
        .collect();
    verdicts.len() <= 1
}

/// 第七节 ② 第 1、2 条：一次固定点的轮数、轮功与逐轮角色数都对得上。
pub(super) fn anchor_matches(settle: &Settle, rounds: u64, work: u64, roles: &[u64]) -> bool {
    let got: Vec<u64> = settle
        .detail
        .iter()
        .map(|round| round.roles_rewritten)
        .collect();
    settle.rounds == rounds && settle.round_work == work && got == roles
}

/// 第七节 ③ 对账、作废条款第 7 条：看盘与看记录两路的槽数相等。
pub(super) fn reconciliation_holds(image_changed_unit_slots: u64, slots_placed: u64) -> bool {
    image_changed_unit_slots == slots_placed
}

/// F2：一个点上 `rounds ≥ floor` 的固定点次数，历史与量窗都算。
pub(super) fn rounds_at_least(item: &PointArmReadings, floor: u64) -> usize {
    all_settles(item)
        .filter(|settle| settle.rounds >= floor)
        .count()
}

/// 第一段那 16 个点（登记第五节「实现量与分段」）。
pub(super) const FIRST_SEGMENT_POINTS: [&str; 16] = [
    "base",
    "files_100",
    "files_1000",
    "files_10000",
    "data_units_100",
    "data_units_1000",
    "data_units_10000",
    "unlinks_100",
    "unlinks_1000",
    "unlinks_10000",
    "files_30000",
    "files_1000_units_10",
    "files_1000_units_100",
    "files_1000_unit_area_2_pow_14",
    "files_1000_unit_area_2_pow_20",
    "files_1000_unit_area_2_pow_24",
];

/// 第六节末尾的够判：P1、P3 要第一段全部点上 `p1` 的读数完整，P2 要全部点上 `p2a` 的读数完整（P2a 翻面时另要 P2b 的 Q6，由主 agent 对着 `name=q6` 判）。
pub(super) fn fork_rows_with_enough_readings(
    complete: &BTreeSet<(String, String)>,
) -> [(&'static str, bool); 3] {
    let all_on = |arm: &str| {
        FIRST_SEGMENT_POINTS
            .iter()
            .all(|point| complete.contains(&((*point).to_string(), arm.to_string())))
    };
    [
        ("P1", all_on("p1")),
        ("P2", all_on("p2a")),
        ("P3", all_on("p1")),
    ]
}

/// 第八节几何敏感性：同一判定在方向相反的旋钮上翻不翻。
fn report_sensitivity(items: &ReadingsByPointAndArm<'_>) {
    let groups: [(&str, &[&str]); 5] = [
        (
            "unit_area",
            &[
                "files_1000_unit_area_2_pow_14",
                "files_1000",
                "files_1000_unit_area_2_pow_20",
                "files_1000_unit_area_2_pow_24",
            ],
        ),
        (
            "active_leaves_source",
            &["files_10000", "files_1000_units_10"],
        ),
        ("inode_batch", &["files_10000", "batch_of_one_files_10000"]),
        (
            "order",
            &[
                "files_then_big_1000",
                "hot_then_cold_1000",
                "churn_random_1000",
                "files_1000",
            ],
        ),
        ("remount", &["files_10000", "remount_every_1000"]),
    ];
    for arm in MEASURED_ARMS {
        for (knob, points) in groups {
            let readings: Vec<String> = points
                .iter()
                .filter_map(|point| {
                    items
                        .get(&((*point).to_string(), arm.to_string()))
                        .map(|item| (point, peaks_of(all_settles(item))))
                })
                .map(|(point, peaks)| {
                    format!(
                        "{point}:{}:{}",
                        peaks.rounds,
                        rounds_verdict(peaks.rounds, peaks.did_not_settle)
                    )
                })
                .collect();
            if !readings.is_empty() {
                println!(
                    "name=sensitivity arm={arm} knob={knob} readings={} stable={}",
                    readings.join(","),
                    verdicts_agree(&readings)
                );
            }
        }
    }
    for point in ["files_1000", "files_10000"] {
        if let Some(item) = items.get(&(point.to_string(), "p1".to_string())) {
            let in_state = |state: &str| {
                item.window
                    .iter()
                    .filter(|settle| settle.state == state)
                    .map(|settle| settle.rounds)
                    .max()
                    .unwrap_or(0)
            };
            println!(
                "name=sensitivity_mount point={point} same_mount_peak={} remounted_peak={}",
                in_state("same_mount"),
                in_state("remounted")
            );
        }
    }
}

fn peak_of_window_between_rounds_pricing(item: &PointArmReadings, pick: &dyn Fn(&BetweenRoundsPricingReading) -> u64) -> Option<f64> {
    let readings: Vec<u64> = item
        .window
        .iter()
        .filter_map(|settle| {
            let judgement = judgement_of(item, settle)?;
            let between = by_delta_input(&judgement, "between", item.unit_area_slots).ok()?;
            Some(pick(&between_rounds_pricing_reading(settle, &between, item.unit_bytes)))
        })
        .collect();
    readings.iter().max().map(|value| *value as f64)
}

/// 第六节按臂的判定，各报各的，不合取（合取留给主 agent 对着岔路单做）。
fn report_verdicts(all: &[PointArmReadings], items: &ReadingsByPointAndArm<'_>) {
    let arms: Vec<&str> = MEASURED_ARMS
        .iter()
        .copied()
        .filter(|arm| items.keys().any(|(_, in_arm)| in_arm == arm))
        .collect();
    for arm in &arms {
        let of_arm: Vec<&PointArmReadings> = all.iter().filter(|item| item.arm == *arm).collect();
        let verdicts: Vec<(String, &'static str, Peaks)> = of_arm
            .iter()
            .map(|item| {
                let peaks = peaks_of(all_settles(item));
                (
                    item.point.clone(),
                    rounds_verdict(peaks.rounds, peaks.did_not_settle),
                    peaks,
                )
            })
            .collect();
        let named = |verdict: &str| -> String {
            let points: Vec<&str> = verdicts
                .iter()
                .filter(|(_, got, _)| *got == verdict)
                .map(|(point, _, _)| point.as_str())
                .collect();
            if points.is_empty() {
                "-".to_string()
            } else {
                points.join(",")
            }
        };
        println!(
            "name=verdict question=q1 arm={arm} points_many={} points_at_the_limit={}",
            named("many"),
            named("hits_the_limit")
        );
        let not_held: Vec<String> = verdicts
            .iter()
            .filter(|(_, _, peaks)| {
                rounds_held_down_verdict(peaks.rounds, peaks.effective_rounds) == "not_held_down"
            })
            .map(|(point, _, _)| point.clone())
            .collect();
        println!(
            "name=verdict question=q4 arm={arm} verdict={} points_not_held_down={}",
            if not_held.is_empty() {
                "held_down"
            } else {
                "not_held_down"
            },
            if not_held.is_empty() {
                "-".to_string()
            } else {
                not_held.join(",")
            }
        );
        let mut grows_on = Vec::new();
        for (axis, points) in AXES {
            let work = axis_line("q2", arm, axis, &points, items, &|item| {
                Some(peaks_of(all_settles(item)).round_work as f64)
            });
            if work.is_some_and(|growth| band(growth) == Band::Grows) {
                grows_on.push(axis);
            }
            let roles = axis_line("q3_final_roles", arm, axis, &points, items, &|item| {
                Some(peaks_of(all_settles(item)).final_roles as f64)
            });
            println!(
                "name=f5 arm={arm} axis={axis} triggered={}",
                roles.is_some_and(|growth| band(growth) == Band::Grows)
            );
            axis_line("q3_final_nodes", arm, axis, &points, items, &|item| {
                Some(peaks_of(all_settles(item)).final_nodes as f64)
            });
            axis_line("q7_sum_of_rounds", arm, axis, &points, items, &|item| {
                peak_of_window_between_rounds_pricing(item, &|reading| reading.sum_of_rounds)
            });
            axis_line("q7_memory_bytes", arm, axis, &points, items, &|item| {
                peak_of_window_between_rounds_pricing(item, &|reading| reading.memory_bytes)
            });
            axis_line("q8_first_publish", arm, axis, &points, items, &|item| {
                window_settle(item, "W1", 1)
                    .and_then(|settle| by_delta_of_settle(item, settle))
                    .map(|(today, _)| today as f64)
            });
            axis_line("q8_peak", arm, axis, &points, items, &|item| {
                item.window
                    .iter()
                    .filter_map(|settle| by_delta_of_settle(item, settle))
                    .map(|(today, _)| today)
                    .max()
                    .map(|value| value as f64)
            });
            axis_line("q8_p3_first_publish", arm, axis, &points, items, &|item| {
                window_settle(item, "W1", 1)
                    .and_then(|settle| by_delta_of_settle(item, settle))
                    .map(|(_, between_rounds_pricing_total)| between_rounds_pricing_total as f64)
            });
            axis_line("q8_p3_peak", arm, axis, &points, items, &|item| {
                item.window
                    .iter()
                    .filter_map(|settle| by_delta_of_settle(item, settle))
                    .map(|(_, between_rounds_pricing_total)| between_rounds_pricing_total)
                    .max()
                    .map(|value| value as f64)
            });
        }
        println!(
            "name=verdict question=q2 arm={arm} grows_on={}",
            if grows_on.is_empty() {
                "-".to_string()
            } else {
                grows_on.join(",")
            }
        );
        let unexplained: Vec<String> = of_arm
            .iter()
            .filter(|item| {
                chain_explained_ratio(&item.window)
                    .is_some_and(|ratio| ratio < CHAIN_EXPLAINS_AT_LEAST)
            })
            .map(|item| item.point.clone())
            .collect();
        println!(
            "name=verdict question=q10 arm={arm} points_the_chain_does_not_explain={}",
            if unexplained.is_empty() {
                "-".to_string()
            } else {
                unexplained.join(",")
            }
        );
        let many_placements: Vec<String> = of_arm
            .iter()
            .filter(|item| {
                rounds_held_down_verdict(
                    peaks_of(all_settles(item)).rounds,
                    peaks_of(all_settles(item)).effective_rounds,
                ) == "not_held_down"
            })
            .filter(|item| exit_share(&item.window, "placement_not_covered") > 0.5)
            .map(|item| item.point.clone())
            .collect();
        println!(
            "name=f4 arm={arm} points={}",
            if many_placements.is_empty() {
                "-".to_string()
            } else {
                many_placements.join(",")
            }
        );
    }
    for item in all.iter().filter(|item| item.arm == "p1") {
        let roles_changed_too_much = item.window.iter().any(|settle| {
            settle
                .detail
                .iter()
                .map(|round| round.roles_changed)
                .sum::<u64>()
                > 2 * settle.final_roles.max(1)
        });
        let premise_fails = item
            .window
            .iter()
            .any(|settle| premise_ratio(settle) > PREMISE_RATIO_AT_MOST);
        println!("name=f6 item={} premise_ratio_over_the_bound={premise_fails} roles_changed_over_twice_the_final={roles_changed_too_much}", item.name);
    }
}

pub(super) fn report(paths: &[String]) -> Result<(), String> {
    let mut all = Vec::new();
    for path in paths {
        all.push(read_item(path)?);
    }
    let mut items: ReadingsByPointAndArm<'_> = BTreeMap::new();
    for item in &all {
        match item_is_complete(item) {
            Ok(()) => {
                println!("name=item_complete item={} complete=true", item.name);
                items.insert((item.point.clone(), item.arm.clone()), item);
            }
            Err(problem) => println!(
                "name=void_6 item={} problem={problem} voided=this_point_this_arm",
                item.name
            ),
        }
        if item.arm != "plain" {
            report_item(item);
        }
    }
    report_measured_anchors(&items);
    report_controls(&items);
    for item in all
        .iter()
        .filter(|item| item.arm != "p1" && item.arm != "plain")
    {
        if let Some(today) = items.get(&(item.point.clone(), "p1".to_string())) {
            report_arm_against_today(item, today);
        }
    }
    report_verdicts(&all, &items);
    report_sensitivity(&items);
    let complete: BTreeSet<(String, String)> = items.keys().cloned().collect();
    for (row, enough) in fork_rows_with_enough_readings(&complete) {
        println!("name=fork_row row={row} enough_readings={enough}");
    }
    println!(
        "name=report_done items={} complete={}",
        all.len(),
        items.len()
    );
    Ok(())
}

/// 读 E203 一份计数产物里同一次挂载覆盖写第 `round` 轮那一次判（第七节 ② 第 3 条；只读这一份里这几样）。
pub(super) fn e203_judgement(path: &str, round: u64) -> Result<(JudgementReadings, u64), String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("读不了 {path}：{error}"))?;
    let mut judgement = JudgementReadings::default();
    let mut unit_area_slots = None;
    for line in text.lines().filter(|line| line.starts_with("E203RESULT ")) {
        let fields = fields_of(line);
        let value = |key: &str| number(&fields, key).unwrap_or(0);
        let text_of = |key: &str| fields.get(key).cloned().unwrap_or_default();
        if text_of("name") == "point" {
            unit_area_slots = number(&fields, "unit_area_slots");
        }
        if text_of("state") != "same_mount"
            || text_of("kind") != "overwrite"
            || number(&fields, "round") != Some(round)
        {
            continue;
        }
        match text_of("name").as_str() {
            "row" => {
                judgement.rows.insert(
                    (text_of("segment"), text_of("row")),
                    (value("calls"), value("query_levels")),
                );
                judgement
                    .row_steps
                    .insert((text_of("segment"), text_of("row")), value("steps"));
            }
            "delta" => judgement.deltas.push((
                text_of("segment"),
                value("fixed_point_round"),
                text_of("key"),
                value("value"),
            )),
            "state" => judgement.state = fields.clone(),
            "publish" => judgement.fixed_point_rounds = number(&fields, "fixed_point_rounds"),
            _ => {}
        }
    }
    Ok((
        judgement,
        unit_area_slots.ok_or_else(|| format!("{path} 里没有 name=point 行"))?,
    ))
}
