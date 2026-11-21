//! 峰值、轮数压低的判定、回落与复现、链解释的比值、同终集同读数的判定、单步对照与逐项的完整性、模型对不上、对照差。

use e7_index_bench::i64_from_u64;
use super::*;

// ---- 一条臂一个点上的量（第六节、第八节）----
#[derive(Clone, Debug, Default)]
pub(super) struct Peaks {
    pub(super) rounds: u64,
    pub(super) rounds_at: String,
    pub(super) round_work: u64,
    pub(super) final_roles: u64,
    pub(super) final_nodes: u64,
    pub(super) effective_rounds: f64,
    pub(super) did_not_settle: bool,
}

pub(super) fn peaks_of<'settles>(settles: impl Iterator<Item = &'settles Settle>) -> Peaks {
    let mut peaks = Peaks::default();
    for settle in settles {
        if settle.rounds > peaks.rounds {
            peaks.rounds = settle.rounds;
            peaks.rounds_at = format!("{}/{}/{}", settle.state, settle.operation, settle.operation_index);
        }
        peaks.round_work = peaks.round_work.max(settle.round_work);
        peaks.final_roles = peaks.final_roles.max(settle.final_roles);
        peaks.final_nodes = peaks.final_nodes.max(settle.final_nodes);
        peaks.did_not_settle |= settle.did_not_settle();
        if settle.final_roles > 0 {
            peaks.effective_rounds = peaks
                .effective_rounds
                .max(settle.round_work as f64 / settle.final_roles as f64);
        }
    }
    peaks
}

pub(super) fn all_settles(item: &PointArmReadings) -> impl Iterator<Item = &Settle> {
    item.history.iter().chain(&item.window)
}

pub(super) fn rounds_held_down_verdict(peak_rounds: u64, effective_rounds: f64) -> &'static str {
    if peak_rounds <= MANY_ROUNDS_ABOVE && effective_rounds <= EFFECTIVE_ROUNDS_AT_MOST {
        "held_down"
    } else {
        "not_held_down"
    }
}

/// Q9：`R*` 那一次之后 8 次发布内回不回到 `≤ 4`；同一状态里两次 `≥ 9` 是不是隔了 8 次以上。
pub(super) fn fallback_and_recurrence(sequence: &[u64]) -> (bool, bool) {
    let peak = sequence.iter().copied().max().unwrap_or(0);
    let peak_at = sequence
        .iter()
        .position(|rounds| *rounds == peak)
        .unwrap_or(0);
    let falls_back = sequence
        .iter()
        .skip(peak_at + 1)
        .take(8)
        .any(|rounds| *rounds <= 4)
        || peak <= 4;
    let many: Vec<usize> = sequence
        .iter()
        .enumerate()
        .filter(|(_, rounds)| **rounds >= 9)
        .map(|(index, _)| index)
        .collect();
    let recurs = many.windows(2).any(|pair| pair[1] - pair[0] > 8);
    (falls_back, recurs)
}

/// Q10：`rounds ≥ 9` 的固定点里 `|rounds − (释放链层数 + 2)| ≤ 2` 的比例；一次都没有交 `None`（记「没出现」）。
pub(super) fn chain_explained_ratio(settles: &[Settle]) -> Option<f64> {
    let many: Vec<&Settle> = settles.iter().filter(|settle| settle.rounds >= 9).collect();
    if many.is_empty() {
        return None;
    }
    let explained = many
        .iter()
        .filter(|settle| {
            (i64_from_u64(settle.rounds) - (i64_from_u64(settle.release_chain_depth) + 2)).abs()
                <= CHAIN_TOLERANCE
        })
        .count();
    Some(explained as f64 / many.len() as f64)
}

/// `release_not_covered` 在长轮固定点各轮里的占比（Q10 报数不判），`placement_not_covered` 占多数时 F4 判得出。
pub(super) fn exit_share(settles: &[Settle], exit: &str) -> f64 {
    let rounds: Vec<&Round> = settles
        .iter()
        .filter(|settle| settle.rounds >= 9)
        .flat_map(|settle| &settle.detail)
        .collect();
    rounds.iter().filter(|round| round.exit == exit).count() as f64 / rounds.len().max(1) as f64
}

/// 两段固定点序列逐个比哈希（Q5、F3、对照）：(相同的个数, 比的个数, 第一处不同的下标)。
pub(super) fn same_final_sets(one: &[&Settle], other: &[&Settle]) -> (usize, usize, Option<usize>) {
    let compared = one.len().max(other.len());
    let mut same = 0;
    let mut first_difference = None;
    for index in 0..compared {
        match (one.get(index), other.get(index)) {
            (Some(left), Some(right)) if left.final_set_hash == right.final_set_hash => same += 1,
            _ => first_difference = first_difference.or(Some(index)),
        }
    }
    (same, compared, first_difference)
}

/// `closure_off`、`p2b_never` 的对照：逐轮读数与 `p1` 逐字段相同（轮数、角色数、节点数、轮功、哈希、落点）。
pub(super) fn same_readings(one: &[&Settle], other: &[&Settle]) -> bool {
    one.len() == other.len()
        && one.iter().zip(other).all(|(left, right)| {
            (
                left.rounds,
                left.final_roles,
                left.final_nodes,
                left.round_work,
                &left.final_set_hash,
                left.slots_placed,
            ) == (
                right.rounds,
                right.final_roles,
                right.final_nodes,
                right.round_work,
                &right.final_set_hash,
                right.slots_placed,
            )
        })
}

/// `single_step` 的对照（第五节表第一行）：不动点逐次相同，每个定下的固定点轮数不少于节点数，量窗至少一次比 `p1` 多轮。
pub(super) fn single_step_control_holds(
    single: &[&Settle],
    today: &[&Settle],
    single_window: &[Settle],
    today_window: &[Settle],
) -> bool {
    let same_set = single.len() == today.len()
        && single.iter().zip(today).all(|(left, right)| {
            left.final_set_hash == right.final_set_hash && left.final_roles == right.final_roles
        });
    let enough_rounds = single
        .iter()
        .filter(|settle| settle.outcome == "settled")
        .all(|settle| settle.rounds >= settle.final_nodes);
    let slower = single_window
        .iter()
        .zip(today_window)
        .any(|(left, right)| left.rounds > right.rounds);
    same_set && enough_rounds && slower
}

/// 作废条款第 6 条：直方图行数之和等于历史里固定点调用数，`rounds ≥ 3` 的单行条数对得上，有 `item_done`，量窗每条固定点逐轮齐。
pub(super) fn item_is_complete(item: &PointArmReadings) -> Result<(), String> {
    if !item.done {
        return Err(item
            .refused
            .clone()
            .map_or("no_item_done".to_string(), |cause| {
                format!("refused:{cause}")
            }));
    }
    if item.arm == "plain" {
        return Ok(());
    }
    let Some(history) = &item.history_line else {
        return Err("no_history_line".to_string());
    };
    if number(history, "settle_calls") != Some(item.histogram_calls)
        || item.history.len() as u64 != item.histogram_calls
    {
        return Err(format!(
            "histogram_calls={}_history={:?}",
            item.histogram_calls,
            history.get("settle_calls")
        ));
    }
    if item.history_settle_lines != item.histogram_rounds_three_or_more {
        return Err(format!(
            "history_settle_lines={}_three_or_more={}",
            item.history_settle_lines, item.histogram_rounds_three_or_more
        ));
    }
    if item.window.is_empty() {
        return Err("no_window".to_string());
    }
    for settle in &item.window {
        if settle.detail.len() as u64 != settle.rounds {
            return Err(format!(
                "rounds={}_detail={}",
                settle.rounds,
                settle.detail.len()
            ));
        }
    }
    Ok(())
}

/// 停机条款第 2 条：`W` 等于 `Σ_r A_r`，产物头常量等于本地常量。交回对不上的条数。
pub(super) fn model_mismatches(item: &PointArmReadings) -> Vec<String> {
    let mut problems = Vec::new();
    for settle in &item.window {
        let sum: u64 = settle
            .detail
            .iter()
            .map(|round| round.roles_rewritten)
            .sum();
        if sum != settle.round_work {
            problems.push(format!("round_work={}_sum={sum}", settle.round_work));
        }
    }
    for (key, expected) in [
        ("leaf_slots", LEAF_SLOTS),
        ("internal_fanout", INTERNAL_FANOUT),
        ("settling_rounds_at_most", SETTLING_ROUNDS_LIMIT as u64),
        ("disks", DISKS),
    ] {
        if item.arm != "plain" && number(&item.constants, key) != Some(expected) {
            problems.push(format!(
                "{key}={:?}_expected={expected}",
                item.constants.get(key)
            ));
        }
    }
    problems
}

/// P3 阳性对照：每个发布上累计走 P3 式子与 B8 列相同。交回比了几次与不同的次数。
pub(super) fn between_rounds_pricing_control_gap(item: &PointArmReadings) -> (u64, u64) {
    let (mut compared, mut different) = (0, 0);
    for (key, judgement) in &item.judgements {
        if key.ends_with('/') {
            continue;
        }
        let Some(position) = key.rfind('/') else {
            continue;
        };
        let mut merged = judgement.clone();
        if let Some(prefix) = item.judgements.get(&key[..=position]) {
            merged.state = prefix.state.clone();
        }
        if let Ok(input) = by_delta_input(&merged, "between", item.unit_area_slots) {
            compared += 1;
            if between_rounds_pricing_round_steps(&between_rounds_pricing_cumulative_input(&input), between_rounds_unit_prices(&input))
                != by_delta_rows_that_between_rounds_pricing_changes(&input)
            {
                different += 1;
            }
        }
    }
    (compared, different)
}

/// 这一条固定点那次判（`state` 从 `<状态>/<操作>/<第几次>/` 那一行并进来）。
pub(super) fn judgement_of(item: &PointArmReadings, settle: &Settle) -> Option<JudgementReadings> {
    let key = settle.publish_key();
    let mut judgement = item.judgements.get(&key)?.clone();
    let position = key.rfind('/')?;
    judgement.state = item.judgements.get(&key[..=position])?.state.clone();
    Some(judgement)
}

// ---- 报告：每个量各占一行，各报各的判定（第六节）----
pub(super) const AXES: [(&str, [&str; 4]); 3] = [
    ("files", ["base", "files_100", "files_1000", "files_10000"]),
    (
        "data_units",
        [
            "base",
            "data_units_100",
            "data_units_1000",
            "data_units_10000",
        ],
    ),
    (
        "unlinks",
        ["base", "unlinks_100", "unlinks_1000", "unlinks_10000"],
    ),
];
pub(super) const AXIS_POSITIONS: [f64; 4] = [10.0, 100.0, 1_000.0, 10_000.0];
pub(super) const MEASURED_ARMS: [&str; 4] = ["p1", "p2a", "p2b16", "p2b256"];
