//! 判读用的读数、轮与结算、各点各臂的读数、从产物行读出结算与逐轮最大值、逐项按差量的输入与轮间计价的读数。

use super::*;

/// 一次判（E203 的 `row`、`delta`、`state`、`publish` 行）。
#[derive(Clone, Debug, Default)]
pub(super) struct JudgementReadings {
    /// (段, 行) → (调用数, 查询层数之和)。
    pub(super) rows: BTreeMap<(String, String), (u64, u64)>,
    /// (段, 行) → 步数（产物行的 `steps`；s37 与 s29 的步数报数用，不进 B8 式子）。
    pub(super) row_steps: BTreeMap<(String, String), u64>,
    /// (段, 固定点第几轮, 名字, 值)，次序照产物。
    pub(super) deltas: Vec<(String, u64, String, u64)>,
    pub(super) state: Fields,
    pub(super) fixed_point_rounds: Option<u64>,
}

/// 一轮（第 2、3、5、6 组）。
#[derive(Clone, Debug, Default)]
pub(super) struct Round {
    pub(super) round: u64,
    pub(super) roles_rewritten: u64,
    pub(super) nodes_rewritten: u64,
    pub(super) rewrite_set_size: u64,
    pub(super) release_dirty_new: u64,
    pub(super) placement_dirty_new: u64,
    pub(super) exit: String,
    pub(super) roles_changed: u64,
    pub(super) records_changed: u64,
    pub(super) tree_keys_changed: Vec<u64>,
    /// 行 → (调用数, 步数, 查询层数之和)。
    pub(super) rows: BTreeMap<String, (u64, u64, u64)>,
}

/// 一次固定点（第 1、4、6、7 组）：量窗里全有，历史里只有第 1 组。
#[derive(Clone, Debug, Default)]
pub(super) struct Settle {
    pub(super) state: String,
    pub(super) operation: String,
    pub(super) operation_index: u64,
    pub(super) judgement: u64,
    pub(super) rounds: u64,
    pub(super) outcome: String,
    pub(super) final_roles: u64,
    pub(super) final_nodes: u64,
    pub(super) round_work: u64,
    pub(super) final_set_hash: String,
    pub(super) seed_nodes: u64,
    pub(super) closure_hops: u64,
    pub(super) closure_lookup_steps: u64,
    pub(super) slots_placed: u64,
    pub(super) previous_nodes: u64,
    pub(super) release_chain_depth: u64,
    pub(super) release_chain_nodes: u64,
    pub(super) active_leaves: Vec<u64>,
    pub(super) detail: Vec<Round>,
}

impl Settle {
    pub(super) fn did_not_settle(&self) -> bool {
        self.outcome == "did_not_settle"
    }

    /// 这一条固定点所在那次判的键：`<状态>/<操作>/<第几次>/<第几次判>`。
    pub(super) fn publish_key(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.state, self.operation, self.operation_index, self.judgement
        )
    }

    /// 全部固定点里的两道之间步数（第 5 组之和）。
    pub(super) fn fixed_point_steps(&self) -> u64 {
        self.detail
            .iter()
            .flat_map(|round| round.rows.values())
            .map(|(_, steps, _)| steps)
            .sum()
    }
}

/// 一份产物：一个 `<点>__<臂>`。
#[derive(Clone, Debug, Default)]
pub(super) struct PointArmReadings {
    pub(super) name: String,
    pub(super) point: String,
    pub(super) arm: String,
    pub(super) unit_area_slots: u64,
    unlinks: u64,
    data_units: u64,
    files: u64,
    pub(super) constants: Fields,
    pub(super) unit_bytes: u64,
    /// 历史里每次固定点，次序照发生：直方图里的 `sequence` 解开。
    pub(super) history: Vec<Settle>,
    pub(super) histogram_calls: u64,
    pub(super) histogram_rounds_three_or_more: u64,
    pub(super) history_settle_lines: u64,
    pub(super) history_line: Option<Fields>,
    pub(super) window: Vec<Settle>,
    pub(super) judgements: BTreeMap<String, JudgementReadings>,
    pub(super) image_sha256_at_the_history_end: Option<String>,
    pub(super) reconciliations: Vec<(String, u64, u64)>,
    pub(super) clause_violations: Vec<String>,
    pub(super) refused: Option<String>,
    pub(super) done: bool,
}

fn settle_from_fields(fields: &Fields) -> Settle {
    let value = |key: &str| number(fields, key).unwrap_or(0);
    let text = |key: &str| fields.get(key).cloned().unwrap_or_default();
    Settle {
        state: text("state"),
        operation: text("op"),
        operation_index: value("op_index"),
        judgement: value("judgement"),
        rounds: value("rounds"),
        outcome: text("outcome"),
        final_roles: value("final_roles"),
        final_nodes: value("final_nodes"),
        round_work: value("round_work"),
        final_set_hash: text("final_set_hash"),
        seed_nodes: value("seed_nodes"),
        closure_hops: value("closure_hops"),
        closure_lookup_steps: value("closure_lookup_steps"),
        slots_placed: value("slots_placed"),
        previous_nodes: value("previous_nodes"),
        release_chain_depth: value("release_chain_depth"),
        release_chain_nodes: value("release_chain_nodes"),
        active_leaves: numbers(&text("active_leaves_by_device")),
        detail: Vec::new(),
    }
}

/// 直方图 `sequence` 里的一次：轮数:角色数:节点数:轮功:哈希:落点槽数:结局。
fn settle_from_the_sequence(entry: &str) -> Option<Settle> {
    let parts: Vec<&str> = entry.split(':').collect();
    let [rounds, final_roles, final_nodes, round_work, hash, slots_placed, outcome] = parts[..]
    else {
        return None;
    };
    Some(Settle {
        rounds: rounds.parse().ok()?,
        final_roles: final_roles.parse().ok()?,
        final_nodes: final_nodes.parse().ok()?,
        round_work: round_work.parse().ok()?,
        final_set_hash: hash.to_string(),
        slots_placed: slots_placed.parse().ok()?,
        outcome: match outcome {
            "s" => "settled",
            "r" => "placement_refused",
            "d" => "did_not_settle",
            _ => "other_error",
        }
        .to_string(),
        ..Settle::default()
    })
}

/// 把一行 `E206RESULT` 收进 `item`。
pub(super) fn take_line(item: &mut PointArmReadings, fields: &Fields) {
    let name = fields.get("name").map_or("", String::as_str);
    let value = |key: &str| number(fields, key).unwrap_or(0);
    let text = |key: &str| fields.get(key).cloned().unwrap_or_default();
    match name {
        "point" => {
            item.name = text("item");
            item.point = text("point");
            item.arm = text("arm");
            item.unit_area_slots = value("unit_area_slots");
            item.unlinks = value("unlinks");
            item.data_units = value("data_units");
            item.files = value("files");
        }
        "constants" => item.constants = fields.clone(),
        "sizes" => item.unit_bytes = value("unit_bytes"),
        "history_histogram" => {
            item.histogram_calls += value("calls");
            item.histogram_rounds_three_or_more += value("rounds_three_or_more");
            item.history.extend(
                text("sequence")
                    .split(';')
                    .filter_map(settle_from_the_sequence),
            );
        }
        "history_settle" => item.history_settle_lines += 1,
        "history" => item.history_line = Some(fields.clone()),
        "image_sha256" => item.image_sha256_at_the_history_end = fields.get("sha256").cloned(),
        "settle" => item.window.push(settle_from_fields(fields)),
        "round" => {
            if let Some(settle) = item.window.last_mut() {
                settle.detail.push(Round {
                    round: value("round"),
                    roles_rewritten: value("roles_rewritten"),
                    nodes_rewritten: value("nodes_rewritten"),
                    rewrite_set_size: value("rewrite_set_size"),
                    release_dirty_new: value("release_dirty_new_leaves")
                        + value("release_dirty_new_internal"),
                    placement_dirty_new: value("placement_dirty_new_leaves")
                        + value("placement_dirty_new_internal"),
                    exit: text("round_exit"),
                    roles_changed: value("roles_changed_between_rounds"),
                    records_changed: value("records_changed_between_rounds"),
                    tree_keys_changed: numbers(&text("tree_keys_changed_between_rounds")),
                    rows: BTreeMap::new(),
                });
            }
        }
        "round_row" => {
            if let Some(round) = item
                .window
                .last_mut()
                .and_then(|settle| settle.detail.last_mut())
            {
                round.rows.insert(
                    text("row"),
                    (value("calls"), value("steps"), value("query_levels")),
                );
            }
        }
        "state" => {
            let prefix = format!("{}/{}/{}/", text("state"), text("op"), value("op_index"));
            item.judgements.entry(prefix).or_default().state = fields.clone();
        }
        "row" => {
            item.judgements
                .entry(text("publish"))
                .or_default()
                .rows
                .insert(
                    (text("segment"), text("row")),
                    (value("calls"), value("query_levels")),
                );
            item.judgements
                .entry(text("publish"))
                .or_default()
                .row_steps
                .insert((text("segment"), text("row")), value("steps"));
        }
        "delta" => {
            item.judgements
                .entry(text("publish"))
                .or_default()
                .deltas
                .push((
                    text("segment"),
                    value("fixed_point_round"),
                    text("key"),
                    value("value"),
                ));
        }
        "publish" => {
            item.judgements
                .entry(text("publish"))
                .or_default()
                .fixed_point_rounds = number(fields, "fixed_point_rounds")
        }
        "reconciliation" => item.reconciliations.push((
            text("op"),
            value("image_changed_unit_slots"),
            value("slots_placed"),
        )),
        "clause_violation" => item.clause_violations.push(text("clause")),
        "history_refused" | "window_refused" => item.refused = Some(text("cause")),
        "item_done" => item.done = true,
        _ => {}
    }
}

pub(super) fn read_item(path: &str) -> Result<PointArmReadings, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("读不了 {path}：{error}"))?;
    let mut item = PointArmReadings::default();
    for line in text.lines().filter(|line| line.starts_with("E206RESULT ")) {
        take_line(&mut item, &fields_of(line));
    }
    if item.name.is_empty() {
        return Err(format!("{path} 里没有 name=point 行"));
    }
    Ok(item)
}

// ---- 产物里一次判喂给 B8 的输入（E203 模型 `by_delta_input_from_the_product` 的本地写法）----
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
    // s37：B8 对这一步的做法没人定过，by_delta_rows 里没有它的式子，只读调用次数 pooled_fallback_calls 好标「缺 s37 一行」
    ROW_WITHOUT_A_BY_DELTA_FORMULA,
];
/// 池级回落按叶取落点（s37）：这一步 B8 合计里是 0；准入之前三段里 pooled_fallback_calls > 0 的判，B8、B4 合计标「缺 s37 一行，只对清单内的步」，不拿它判 B8 站不站得住。
pub(super) const ROW_WITHOUT_A_BY_DELTA_FORMULA: &str = "pooled_fallback_by_the_leaf";
const SEGMENTS_BEFORE_ADMISSION: [&str; 3] = ["caller", "before_landing", "between"];

/// 一次判的固定点某一读数每轮的最大值（两道之间那一段）；某一轮读不到记缺口。
fn per_round_maxima(
    judgement: &JudgementReadings,
    key: &str,
    rounds: u64,
    gaps: &mut Vec<String>,
) -> Vec<u64> {
    (1..=rounds)
        .map(|round| {
            let best = judgement
                .deltas
                .iter()
                .filter(|(segment, fixed_point_round, name, _)| {
                    segment == "between" && *fixed_point_round == round && name == key
                })
                .map(|(_, _, _, value)| *value)
                .max();
            best.unwrap_or_else(|| {
                gaps.push(format!("{key}@round{round}"));
                0
            })
        })
        .collect()
}

fn publish_level(judgement: &JudgementReadings, key: &str) -> u64 {
    judgement
        .deltas
        .iter()
        .filter(|(_, _, name, _)| name == key)
        .map(|(_, _, _, value)| *value)
        .max()
        .unwrap_or(0)
}

fn segment_values(judgement: &JudgementReadings, segment: &str, key: &str) -> Vec<u64> {
    judgement
        .deltas
        .iter()
        .filter(|(in_segment, _, name, _)| in_segment == segment && name == key)
        .map(|(_, _, _, value)| *value)
        .collect()
}

/// 一次判在一段里的 B8 输入。读不到的列进缺口，不当 0（作废条款第 6 条）。
pub(super) fn by_delta_input(
    judgement: &JudgementReadings,
    segment: &str,
    unit_area_slots: u64,
) -> Result<ByDeltaInput, Vec<String>> {
    let mut gaps = Vec::new();
    let mut state_value = |key: &str| {
        number(&judgement.state, key).unwrap_or_else(|| {
            gaps.push(format!("state.{key}"));
            0
        })
    };
    let inode_leaf_containers = state_value("inode_leaf_containers");
    let other_files = state_value("other_files");
    let allocation_records = state_value("allocation_records");
    let mapped_units = state_value("mapped_units");
    let deadlist_entries = state_value("deadlist_entries");
    let extent_upper_nodes = state_value("extent_upper_nodes");
    let extent_upper_levels = state_value("extent_upper_levels");
    let accounting_tree_levels = state_value("accounting_tree_levels");
    let deadlist_tree_levels = state_value("deadlist_tree_levels");
    let allocation_tree_levels = state_value("allocation_tree_levels");
    let central_mapping_levels = state_value("central_mapping_levels");
    let between = segment == "between";
    let rounds = if between {
        judgement.fixed_point_rounds.unwrap_or_else(|| {
            gaps.push("publish.fixed_point_rounds".to_string());
            0
        })
    } else {
        0
    };
    let row_cell = |row: &str| {
        judgement
            .rows
            .get(&(segment.to_string(), row.to_string()))
            .copied()
            .unwrap_or((0, 0))
    };
    let calls: BTreeMap<String, u64> = CALLED_ROWS
        .iter()
        .map(|row| ((*row).to_string(), row_cell(row).0))
        .collect();
    let changed_leaves_per_round = per_round_maxima(judgement, "changed_leaves", rounds, &mut gaps);
    let released = per_round_maxima(judgement, "round_placements_released", rounds, &mut gaps);
    let rewritten = per_round_maxima(judgement, "round_roles_rewritten", rounds, &mut gaps);
    let keys_changed = segment_values(judgement, segment, "code_two_keys_changed");
    if keys_changed.len() as u64 != 3 * rounds {
        gaps.push(format!(
            "delta.code_two_keys_changed:{}_for_{rounds}_rounds",
            keys_changed.len()
        ));
    }
    let heights = [
        accounting_tree_levels,
        deadlist_tree_levels,
        central_mapping_levels,
    ];
    if !gaps.is_empty() {
        return Err(gaps);
    }
    Ok(ByDeltaInput {
        inode_leaf_containers,
        other_files,
        touched_files: publish_level(judgement, "touched_files"),
        lower_nodes_of_touched_files: publish_level(judgement, "lower_nodes_of_touched_files"),
        extent_upper_levels,
        extent_upper_nodes,
        accounting_rows_of_the_generation: publish_level(
            judgement,
            "accounting_rows_written_by_the_built_on_publish",
        ),
        accounting_tree_levels,
        settling_rounds: rounds,
        changed_leaves_per_round,
        allocation_tree_levels,
        allocation_records,
        segments_per_disk: unit_area_slots.div_ceil(SEGMENT_SLOTS),
        mapped_units,
        deadlist_entries,
        calls,
        intersecting_records: segment_values(judgement, segment, "intersecting_records")
            .iter()
            .sum(),
        dirty_per_call: segment_values(judgement, segment, "dirty_records"),
        code_two_trees_over_all_rounds: keys_changed
            .iter()
            .enumerate()
            .map(|(index, keys)| (*keys, heights[index % 3]))
            .collect(),
        container_rewrites_over_all_rounds: segment_values(
            judgement,
            segment,
            "container_units_rewritten",
        ),
        moved_entries: publish_level(judgement, "moved_entries"),
        central_mapping_levels,
        deadlist_query_levels: row_cell("deadlist_changes_indexes").1,
        deadlist_changed_entries: publish_level(judgement, "deadlist_entries_changed"),
        release_query_levels: row_cell("release_judgement_version_indexes").1,
        changed_slots_and_records_per_round: released
            .iter()
            .zip(&rewritten)
            .map(|(released, rewritten)| released + rewritten)
            .collect(),
        mapped_units_changed_over_all_rounds: 2 * rewritten.iter().sum::<u64>(),
    })
}

/// 一次判在准入之前三段里 s37 的调用次数 pooled_fallback_calls、s37 的步数、s29 的步数（读不到的行记 0）。
pub(super) fn pooled_fallback_of_the_judgement(judgement: &JudgementReadings) -> (u64, u64, u64) {
    let (mut calls, mut steps, mut per_disk_fallback_steps) = (0, 0, 0);
    for segment in SEGMENTS_BEFORE_ADMISSION {
        let key = |row: &str| (segment.to_string(), row.to_string());
        calls += judgement.rows.get(&key(ROW_WITHOUT_A_BY_DELTA_FORMULA)).map_or(0, |cell| cell.0);
        steps += judgement.row_steps.get(&key(ROW_WITHOUT_A_BY_DELTA_FORMULA)).copied().unwrap_or(0);
        per_disk_fallback_steps += judgement.row_steps.get(&key("fallback_slot_linear_scan")).copied().unwrap_or(0);
    }
    (calls, steps, per_disk_fallback_steps)
}

/// 一次判三段的 B8 合计（E203 第六节 Q6 的量）。
pub(super) fn by_delta_of_the_judgement(
    judgement: &JudgementReadings,
    unit_area_slots: u64,
) -> Result<u64, Vec<String>> {
    let mut total = 0;
    for segment in SEGMENTS_BEFORE_ADMISSION {
        total += by_delta_total(&by_delta_input(judgement, segment, unit_area_slots)?);
    }
    Ok(total)
}

/// P3 的阳性对照：两道之间那一段的 B8 输入并成一份累计，走同一个 P3 式子。
pub(super) fn between_rounds_pricing_cumulative_input(input: &ByDeltaInput) -> BetweenRoundsPricingInput {
    BetweenRoundsPricingInput {
        changed_leaves: input.changed_leaves_per_round.iter().sum(),
        code_two_keys: input.code_two_trees_over_all_rounds.clone(),
        containers: input.container_rewrites_over_all_rounds.iter().sum(),
        mapped_changed: input.mapped_units_changed_over_all_rounds,
        records_in_the_view: input.changed_slots_and_records_per_round.iter().sum(),
        release_finds: input.calls_in_the_row("allocator_release_linear_find"),
        intersection_finds: input.calls_in_the_row("record_intersection_filter"),
        intersecting_records: input.intersecting_records,
        dirty_records: input.dirty_per_call.iter().sum(),
        query_levels: input.deadlist_query_levels
            + input.deadlist_changed_entries * lg(input.deadlist_entries)
            + input.release_query_levels,
        user_data_finds: input.calls_in_the_row("user_data_slot_linear_scan"),
        empty_segment_finds: input.calls_in_the_row("empty_segment_linear_scan"),
        fallback_finds: input.calls_in_the_row("fallback_slot_linear_scan"),
    }
}

pub(super) fn between_rounds_unit_prices(input: &ByDeltaInput) -> BetweenRoundsUnitPrices {
    BetweenRoundsUnitPrices {
        allocation_tree_levels: input.allocation_tree_levels,
        allocation_records: input.allocation_records,
        mapped_units: input.mapped_units,
        segments_per_disk: input.segments_per_disk,
    }
}

/// B8 列里 P3 换掉的那几行之和（阳性对照比的另一边；`between_rounds_pricing_control_gap` 要它等于累计走 P3 式子）。
pub(super) fn by_delta_rows_that_between_rounds_pricing_changes(input: &ByDeltaInput) -> u64 {
    by_delta_rows(input)
        .iter()
        .filter(|(row, _)| ROWS_THAT_BETWEEN_ROUNDS_PRICING_CHANGES.contains(row))
        .map(|(_, steps)| steps)
        .sum()
}

/// 每轮的 P3 输入（第五节 P3 式子）：相邻两轮的差，第 1 轮的上一轮为空。
fn between_rounds_pricing_inputs(settle: &Settle, input: &ByDeltaInput) -> Vec<BetweenRoundsPricingInput> {
    let tree_height = |tree: usize| {
        input
            .code_two_trees_over_all_rounds
            .get(tree)
            .map_or(0, |(_, height)| *height)
    };
    let trees_per_round = 3;
    let mut changed_before = 0;
    let mut containers_before = 0;
    let mut inputs = Vec::new();
    for (index, round) in settle.detail.iter().enumerate() {
        let changed_now = input
            .changed_leaves_per_round
            .get(index)
            .copied()
            .unwrap_or(0);
        let containers_now: u64 = input
            .container_rewrites_over_all_rounds
            .iter()
            .skip(index * trees_per_round)
            .take(trees_per_round)
            .sum();
        let query_levels_now: u64 = [
            "deadlist_changes_indexes",
            "release_judgement_version_indexes",
        ]
        .iter()
        .filter_map(|row| round.rows.get(*row))
        .map(|(_, _, levels)| levels)
        .sum();
        let calls_now = |row: &str| round.rows.get(row).map_or(0, |(calls, _, _)| *calls);
        inputs.push(BetweenRoundsPricingInput {
            changed_leaves: changed_now.saturating_sub(changed_before),
            code_two_keys: round
                .tree_keys_changed
                .iter()
                .enumerate()
                .map(|(tree, keys)| (*keys, tree_height(tree)))
                .collect(),
            containers: containers_now.abs_diff(containers_before),
            mapped_changed: 2 * round.roles_changed,
            records_in_the_view: round.records_changed,
            release_finds: round.records_changed,
            intersection_finds: round.records_changed,
            intersecting_records: 0,
            dirty_records: round.records_changed,
            query_levels: if round.roles_rewritten == 0 {
                0
            } else {
                (query_levels_now * round.roles_changed).div_ceil(round.roles_rewritten)
            },
            user_data_finds: calls_now("user_data_slot_linear_scan").min(round.records_changed),
            empty_segment_finds: calls_now("empty_segment_linear_scan").min(round.records_changed),
            fallback_finds: calls_now("fallback_slot_linear_scan").min(round.records_changed),
        });
        changed_before = changed_now;
        containers_before = containers_now;
    }
    inputs
}

/// P3 一次固定点的读数（Q7）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct BetweenRoundsPricingReading {
    pub(super) largest_round: u64,
    pub(super) sum_of_rounds: u64,
    pub(super) memory_bytes: u64,
    pub(super) maintenance_steps: u64,
}

pub(super) fn between_rounds_pricing_reading(settle: &Settle, input: &ByDeltaInput, unit_bytes: u64) -> BetweenRoundsPricingReading {
    let prices = between_rounds_unit_prices(input);
    let steps: Vec<u64> = between_rounds_pricing_inputs(settle, input)
        .iter()
        .map(|round| between_rounds_pricing_round_steps(round, prices))
        .collect();
    let largest_roles = settle
        .detail
        .iter()
        .map(|round| round.roles_rewritten)
        .max()
        .unwrap_or(0);
    let largest_rewrite_set = settle
        .detail
        .iter()
        .map(|round| round.rewrite_set_size)
        .max()
        .unwrap_or(0);
    let records_changed: u64 = settle
        .detail
        .iter()
        .map(|round| round.records_changed)
        .sum();
    let roles_changed: u64 = settle.detail.iter().map(|round| round.roles_changed).sum();
    BetweenRoundsPricingReading {
        largest_round: steps.iter().copied().max().unwrap_or(0),
        sum_of_rounds: steps.iter().sum(),
        memory_bytes: largest_roles * (unit_bytes + 8)
            + largest_rewrite_set * 24
            + records_changed * 24,
        maintenance_steps: records_changed * lg(input.allocation_records)
            + roles_changed * lg(largest_roles),
    }
}

/// 前提比 ρ = Σ_r records_changed_between_rounds_r ÷ W（Q7）：走释放那一轮与取落点那一轮都算。
pub(super) fn premise_ratio(settle: &Settle) -> f64 {
    let numerator: u64 = settle
        .detail
        .iter()
        .map(|round| round.records_changed)
        .sum();
    numerator as f64 / settle.round_work.max(1) as f64
}
