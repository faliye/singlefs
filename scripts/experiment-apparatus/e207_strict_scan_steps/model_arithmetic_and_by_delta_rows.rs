//! 指数与带、按增量的各行式子，产物行的解析、点与轮的读数、一行一行读进来。

use super::*;

// ---- 增长阶与判定带 ----
pub(super) fn alpha(low_quantity: f64, high_quantity: f64, low_position: f64, high_position: f64) -> f64 {
    (high_quantity / low_quantity).ln() / (high_position / low_position).ln()
}

/// 读法第 6 条的 α：端点取 10 → 10⁴。
pub(super) fn alpha_on_the_axis(low_quantity: f64, high_quantity: f64) -> f64 {
    alpha(low_quantity, high_quantity, AXIS_LOW, AXIS_HIGH)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Band {
    DoesNotGrow,
    Between,
    Grows,
}

impl Band {
    pub(super) fn tag(self) -> &'static str {
        match self {
            Self::DoesNotGrow => "does_not_grow",
            Self::Between => "between",
            Self::Grows => "grows",
        }
    }
}

pub(super) const BAND_DOES_NOT_GROW_AT_MOST: f64 = 0.25;
pub(super) const BAND_GROWS_AT_LEAST: f64 = 0.5;

pub(super) fn band(growth: f64) -> Band {
    if growth >= BAND_GROWS_AT_LEAST {
        Band::Grows
    } else if growth <= BAND_DOES_NOT_GROW_AT_MOST {
        Band::DoesNotGrow
    } else {
        Band::Between
    }
}

// Q1 的三档：不多、多、撞上界（第六节表）。
pub(super) fn lg(value: u64) -> u64 {
    u64::from(u64::BITS - value.leading_zeros())
}

/// 每层 64 叉的分层位图要几层：最小的 l ≥ 1 使 64ˡ ≥ n（E203 同一个式子）。
fn levels_64(value: u64) -> u64 {
    let mut level = 1_u32;
    while 64_u64.pow(level) < value {
        level += 1;
    }
    u64::from(level)
}

// ---- 几何（第七节 ③，`calc/anchors.py` 照 `allocation_record_tree.rs:97`–`:119` 的找法）----


// ---- B8（逐项按差量）：E203 登记第五节「B8 每一行的做法与步数式子」与修订 7 的 s36，本地重写 ----
/// B8 式子吃的输入：一次发布在一段里的差量与调用次数（量的那一路读到的）。
#[derive(Clone, Debug, Default)]
pub(super) struct ByDeltaInput {
    pub(super) inode_leaf_containers: u64,
    pub(super) other_files: u64,
    pub(super) touched_files: u64,
    pub(super) lower_nodes_of_touched_files: u64,
    pub(super) extent_upper_levels: u64,
    pub(super) extent_upper_nodes: u64,
    pub(super) accounting_rows_of_the_generation: u64,
    pub(super) accounting_tree_levels: u64,
    pub(super) settling_rounds: u64,
    pub(super) changed_leaves_per_round: Vec<u64>,
    pub(super) allocation_tree_levels: u64,
    pub(super) allocation_records: u64,
    pub(super) segments_per_disk: u64,
    pub(super) mapped_units: u64,
    pub(super) deadlist_entries: u64,
    pub(super) calls: BTreeMap<String, u64>,
    pub(super) intersecting_records: u64,
    pub(super) dirty_per_call: Vec<u64>,
    pub(super) code_two_trees_over_all_rounds: Vec<(u64, u64)>,
    pub(super) container_rewrites_over_all_rounds: Vec<u64>,
    pub(super) moved_entries: u64,
    pub(super) central_mapping_levels: u64,
    pub(super) deadlist_query_levels: u64,
    pub(super) deadlist_changed_entries: u64,
    pub(super) release_query_levels: u64,
    pub(super) changed_slots_and_records_per_round: Vec<u64>,
    pub(super) mapped_units_changed_over_all_rounds: u64,
}

impl ByDeltaInput {
    pub(super) fn calls_in_the_row(&self, row: &str) -> u64 {
        self.calls.get(row).copied().unwrap_or(0)
    }
}


/// B8 每一行一次发布准入之前的步数，次序同 E203 登记第五节。
pub(super) fn by_delta_rows(input: &ByDeltaInput) -> Vec<(&'static str, u64)> {
    let per_round_trees: u64 = input
        .code_two_trees_over_all_rounds
        .iter()
        .map(|(delta, height)| delta * height)
        .sum();
    let changed_leaves: u64 = input.changed_leaves_per_round.iter().sum();
    let mapped_changed = input.mapped_units_changed_over_all_rounds;
    vec![
        ("whole_allocator_copy_before_the_judgement", 0),
        (
            "inode_record_linear_find",
            input.calls_in_the_row("inode_record_linear_find")
                * (lg(input.inode_leaf_containers) + lg(INODE_LEAF_RECORDS)),
        ),
        (
            "files_other_than_the_first_plan",
            input.calls_in_the_row("files_other_than_the_first_plan")
                * input.touched_files
                * (lg(input.other_files) + input.lower_nodes_of_touched_files),
        ),
        (
            "file_other_than_the_first_linear_find",
            input.calls_in_the_row("file_other_than_the_first_linear_find") * lg(input.other_files),
        ),
        (
            "inodes_with_an_extent_entry_set",
            input.calls_in_the_row("inodes_with_an_extent_entry_set")
                * input.touched_files
                * lg(input.other_files),
        ),
        (
            "extent_upper_segment_plan",
            input.calls_in_the_row("extent_upper_segment_plan")
                * input.touched_files
                * input.extent_upper_levels
                * lg(input.extent_upper_nodes),
        ),
        (
            "extent_roles_rewritten_scan",
            input.calls_in_the_row("extent_roles_rewritten_scan")
                * input.touched_files
                * input.lower_nodes_of_touched_files,
        ),
        (
            "root_header_linear_find",
            input.calls_in_the_row("root_header_linear_find"),
        ),
        (
            "checkpoint_cost_accounting_tree_plan",
            input.calls_in_the_row("checkpoint_cost_accounting_tree_plan")
                * input.accounting_rows_of_the_generation
                * input.accounting_tree_levels,
        ),
        (
            "cluster_segment_sum",
            input.calls_in_the_row("cluster_segment_sum"),
        ),
        (
            "allocation_record_tree_node_set",
            changed_leaves * input.allocation_tree_levels,
        ),
        (
            "allocation_record_tree_plan",
            changed_leaves * input.allocation_tree_levels,
        ),
        (
            "inode_number_watermark_steps",
            input.calls_in_the_row("inode_number_watermark_steps"),
        ),
        (
            "deadlist_changes_indexes",
            input.deadlist_query_levels
                + input.deadlist_changed_entries * lg(input.deadlist_entries),
        ),
        ("multi_level_tree_planning", per_round_trees),
        (
            "mapped_units_of_container_trees",
            input.container_rewrites_over_all_rounds.iter().sum(),
        ),
        (
            "moved_entries_path_tail_filter",
            input.settling_rounds * input.moved_entries * input.central_mapping_levels,
        ),
        (
            "release_judgement_version_indexes",
            input.release_query_levels,
        ),
        (
            "whole_allocator_copy_in_a_settling_round",
            input.changed_slots_and_records_per_round.iter().sum(),
        ),
        (
            "allocator_release_linear_find",
            input.calls_in_the_row("allocator_release_linear_find") * lg(input.allocation_records),
        ),
        (
            "user_data_slot_linear_scan",
            input.calls_in_the_row("user_data_slot_linear_scan")
                * (levels_64(input.segments_per_disk) + 32),
        ),
        (
            "record_intersection_filter",
            input.calls_in_the_row("record_intersection_filter") * lg(input.allocation_records)
                + input.intersecting_records,
        ),
        (
            "empty_segment_linear_scan",
            input.calls_in_the_row("empty_segment_linear_scan")
                * levels_64(input.segments_per_disk),
        ),
        (
            "fallback_slot_linear_scan",
            input.calls_in_the_row("fallback_slot_linear_scan")
                * (levels_64(input.segments_per_disk) + 64),
        ),
        ("records_of_each_leaf", input.dirty_per_call.iter().sum()),
        (
            "accounting_entries_of_this_publish",
            input.calls_in_the_row("accounting_entries_of_this_publish")
                * input.accounting_rows_of_the_generation,
        ),
        (
            "instance_table_chain",
            input.calls_in_the_row("instance_table_chain"),
        ),
        (
            "mapped_units_whole_table",
            mapped_changed + mapped_changed * lg(input.mapped_units),
        ),
    ]
}

pub(super) fn by_delta_total(input: &ByDeltaInput) -> u64 {
    by_delta_rows(input).iter().map(|(_, steps)| steps).sum()
}

// ---- 产物的读法 ----
pub(super) type Fields = BTreeMap<String, String>;

pub(super) fn fields_of(line: &str) -> Fields {
    line.split(' ')
        .skip(1)
        .filter_map(|word| {
            word.split_once('=')
                .map(|(key, value)| (key.to_string(), value.to_string()))
        })
        .collect()
}

pub(super) fn number(fields: &Fields, key: &str) -> Option<u64> {
    fields.get(key).and_then(|value| value.parse().ok())
}

fn numbers(text: &str) -> Vec<u64> {
    text.split(',')
        .filter_map(|value| value.parse().ok())
        .collect()
}

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
    /// 这一轮走出来的有记录的节点与这一版的节点对不上（读数行 `holding_differs=1`）。
    pub(super) holding_differs: bool,
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
    pub(super) operation_name: String,
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
    pub(super) closure_nodes_added: u64,
    pub(super) slots_placed: u64,
    pub(super) previous_nodes: u64,
    /// 固定点入口时分配记录树的记录数与树高（对账分辨读数、复量的钉用；E206 产物同名）。
    pub(super) previous_records: u64,
    pub(super) tree_height: u64,
    pub(super) release_chain_depth: u64,
    pub(super) release_chain_nodes: u64,
    pub(super) active_leaves: Vec<u64>,
    /// 第 4 组（E207 产物才有；E206 产物读成 `none` 与 0）。
    pub(super) first_merge_site: String,
    pub(super) seed_nodes_first_merge: u64,
    pub(super) pulled_nodes_first_merge: u64,
    pub(super) pulled_age_minimum: u64,
    pub(super) pulled_age_median: u64,
    pub(super) pulled_age_maximum: u64,
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
            self.state, self.operation_name, self.operation_index, self.judgement
        )
    }

    /// 全部固定点里的两道之间步数（第 5 组之和）。
    fn fixed_point_steps(&self) -> u64 {
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
    /// 探针（E207 产物才有；E206 产物读成 `standard`）。
    pub(super) probe: String,
    /// 每个小文件的数据单元数 K（E207 产物的 `units_per_small_file`；E206 产物按形状补：`_units_<K>` 的点取 K，别的取 1）。
    pub(super) units_per_small_file: u64,
    pub(super) unit_area_slots: u64,
    pub(super) unlinks: u64,
    pub(super) data_units: u64,
    pub(super) files: u64,
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
    /// 对账行逐操作（修订 3：`base`、`files_1000` 两点两种状态每个量窗操作）：(状态, 操作, 第几次) → (盘上变了的单元槽数, 计数记的落点槽数)，次序照产物。
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    pub(super) reconciliation_rows: Vec<((String, String, u64), (u64, u64))>,
    /// 每个量窗操作做完之后的镜像哈希（`image_after_the_operation`，修订 3）：(状态, 操作, 第几次) → sha256，次序照产物。
    pub(super) images_after_the_operations: Vec<((String, String, u64), String)>,
    /// 重挂载前后的镜像哈希（`image_at_the_remount`）：`before_the_crash` / `after_the_mount` → sha256。
    pub(super) images_at_the_remount: BTreeMap<String, String>,
    /// 产物头里三份被测核心文件的 sha256（`name=source_sha256`）：文件名 → sha256。
    pub(super) source_sha256: BTreeMap<String, String>,
    pub(super) clause_violations: Vec<String>,
    /// 第 3 组声明行：(状态, 操作, 第几次) → (user_units, old_units, new_units)，次序照产物。
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    pub(super) declared: Vec<((String, String, u64), (u64, u64, u64))>,
    pub(super) refused: Option<String>,
    pub(super) done: bool,
}

/// 读数行里标「哪一种操作、第几次」的两个键。E207 自己的产物写全；E206 的留存产物逐字是另一对，读它时只能照它的写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct OperationKeys {
    pub(super) name: &'static str,
    pub(super) index: &'static str,
}

pub(super) const E207_OPERATION_KEYS: OperationKeys = OperationKeys { name: "operation", index: "operation_index" };
/// 读 E206 留存产物（`research/results/e206-fixed-point-cost-*`）的键，逐字照那批产物。
pub(super) const E206_OPERATION_KEYS: OperationKeys = OperationKeys { name: "op", index: "op_index" };

/// 一行读数是哪个实验写的，就用哪一对键：`E206RESULT` 行取 E206 的，其余取 E207 的。
pub(super) fn operation_keys_of(line: &str) -> OperationKeys {
    if line.starts_with("E206RESULT ") {
        E206_OPERATION_KEYS
    } else {
        E207_OPERATION_KEYS
    }
}

fn settle_from_fields(fields: &Fields, keys: OperationKeys) -> Settle {
    let value = |key: &str| number(fields, key).unwrap_or(0);
    let text = |key: &str| fields.get(key).cloned().unwrap_or_default();
    Settle {
        state: text("state"),
        operation_name: text(keys.name),
        operation_index: value(keys.index),
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
        closure_nodes_added: value("closure_nodes_added"),
        slots_placed: value("slots_placed"),
        previous_nodes: value("previous_nodes"),
        previous_records: value("previous_records"),
        tree_height: value("tree_height"),
        release_chain_depth: value("release_chain_depth"),
        release_chain_nodes: value("release_chain_nodes"),
        active_leaves: numbers(&text("active_leaves_by_device")),
        first_merge_site: fields.get("first_merge_site").cloned().unwrap_or_else(|| "none".to_string()),
        seed_nodes_first_merge: value("seed_nodes_first_merge"),
        pulled_nodes_first_merge: value("pulled_nodes_first_merge"),
        pulled_age_minimum: value("pulled_age_minimum"),
        pulled_age_median: value("pulled_age_median"),
        pulled_age_maximum: value("pulled_age_maximum"),
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

/// 把一行 `E206RESULT` 或 `E207RESULT` 收进 `item`（两个实验的读数行同名同字段，E207 多第 3、4 组）。
pub(super) fn take_line(item: &mut PointArmReadings, fields: &Fields, keys: OperationKeys) {
    let name = fields.get("name").map_or("", String::as_str);
    let value = |key: &str| number(fields, key).unwrap_or(0);
    let text = |key: &str| fields.get(key).cloned().unwrap_or_default();
    match name {
        "point" => {
            item.name = text("item");
            item.point = text("point");
            item.arm = text("arm");
            item.probe = fields.get("probe").cloned().unwrap_or_else(|| "standard".to_string());
            item.units_per_small_file = number(fields, "units_per_small_file").unwrap_or_else(|| units_by_the_point_name(&text("point")));
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
        "settle" => item.window.push(settle_from_fields(fields, keys)),
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
                    holding_differs: value("holding_differs") != 0,
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
            let prefix = format!("{}/{}/{}/", text("state"), text(keys.name), value(keys.index));
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
        "reconciliation" => {
            item.reconciliations.push((text(keys.name), value("image_changed_unit_slots"), value("slots_placed")));
            item.reconciliation_rows.push((
                (text("state"), text(keys.name), value(keys.index)),
                (value("image_changed_unit_slots"), value("slots_placed")),
            ));
        }
        "image_after_the_operation" => item.images_after_the_operations.push(((text("state"), text(keys.name), value(keys.index)), text("image_sha256"))),
        "image_at_the_remount" => {
            item.images_at_the_remount.insert(text("when"), text("image_sha256"));
        }
        "source_sha256" => {
            item.source_sha256.insert(text("file"), text("sha256"));
        }
        "clause_violation" => item.clause_violations.push(text("clause")),
        "declared" => item.declared.push((
            (text("state"), text(keys.name), value(keys.index)),
            (value("user_units"), value("old_units"), value("new_units")),
        )),
        "history_refused" | "window_refused" => item.refused = Some(text("cause")),
        "item_done" => item.done = true,
        _ => {}
    }
}

pub(super) fn read_item(path: &str) -> Result<PointArmReadings, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("读不了 {path}：{error}"))?;
    let mut item = PointArmReadings::default();
    for line in text
        .lines()
        .filter(|line| line.starts_with("E206RESULT ") || line.starts_with("E207RESULT "))
    {
        take_line(&mut item, &fields_of(line), operation_keys_of(line));
    }
    if item.name.is_empty() {
        return Err(format!("{path} 里没有 name=point 行"));
    }
    Ok(item)
}
