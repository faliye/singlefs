//! 计时的包含自检、每步纳秒、线性外推、式子格，以及一行产物与整份产物的读法、行种类表与行的取用。

use super::*;

// ---- 计时的包含自检与 Q4、Q5（登记第六节）----
/// 一轮里某一段 Σ 独占计时不大于段挂钟才有效（作废条款第 5 条）。
pub(super) fn round_is_valid(exclusive_sum_nanos: u64, segment_nanos: u64) -> bool {
    exclusive_sum_nanos <= segment_nanos
}

/// τ = 一行的独占计时 ÷ 步数，取那一行步数最大的那个量过的点。points 是 (步数, 纳秒)。
pub(super) fn tau_nanos_per_step(points: &[(u64, u64)]) -> f64 {
    let (steps, nanos) = points
        .iter()
        .max_by_key(|(steps, _)| *steps)
        .copied()
        .unwrap_or((1, 0));
    to_f64(nanos) / to_f64(steps.max(1))
}

/// 沿一根轴用最后两点的斜率线性外推。
pub(super) fn linear_extrapolation(
    earlier_position: f64,
    earlier_value: f64,
    last_position: f64,
    last_value: f64,
    at_position: f64,
) -> f64 {
    last_value
        + (last_value - earlier_value) * (at_position - last_position)
            / (last_position - earlier_position)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FormulaCell {
    AtLeastOneMillisecond,
    BelowOneMillisecond,
    Unstable,
}

/// Q4 一格：五个 T 都 ≥ 2 毫秒记「≥ 1 毫秒」，都 ≤ 0.5 毫秒记「< 1 毫秒」，否则不稳定。
pub(super) fn one_millisecond_cell(nanos_of_each_round: &[f64]) -> FormulaCell {
    if nanos_of_each_round
        .iter()
        .all(|round_nanos| *round_nanos >= 2.0e6)
    {
        FormulaCell::AtLeastOneMillisecond
    } else if nanos_of_each_round
        .iter()
        .all(|round_nanos| *round_nanos <= 0.5e6)
    {
        FormulaCell::BelowOneMillisecond
    } else {
        FormulaCell::Unstable
    }
}

/// Q5 一格：扫类合计步数相对差 ≤ 10%，且扫类挂钟比在 [0.5, 2]。
pub(super) fn extrapolation_is_checked(
    steps_extrapolated: f64,
    steps_measured: f64,
    nanos_ratio: f64,
) -> bool {
    (steps_extrapolated - steps_measured).abs() <= 0.10 * steps_measured
        && (0.5..=2.0).contains(&nanos_ratio)
}

// ---- 读量的那一路打出的 E203RESULT 行 ----
pub(super) type Fields = BTreeMap<String, String>;

pub(super) fn parse_line(line: &str) -> Option<Fields> {
    let rest = line.strip_prefix("E203RESULT ")?;
    let fields: Fields = rest
        .split_whitespace()
        .filter_map(|pair| pair.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    fields.contains_key("name").then_some(fields)
}

pub(super) fn number(fields: &Fields, key: &str) -> Option<u64> {
    fields.get(key)?.parse().ok()
}

/// 一个点：四根轴的取值（登记第五节取样点）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PointAxes {
    pub(super) unit_area_slots: u64,
    pub(super) unlinks: u64,
    pub(super) data_units: u64,
    pub(super) files: u64,
}

/// 某个点某个状态某种发布某一轮某一段某一行的读数。
#[derive(Clone, Debug, Default)]
pub(super) struct RowCell {
    pub(super) calls: u64,
    pub(super) steps: u64,
    pub(super) scale: u64,
    /// 修订 4：`steps` 里走到命中为止那一部分的步，与那一部分每次调用处读的表长之和。
    pub(super) steps_to_the_hit: u64,
    pub(super) scale_to_the_hit: u64,
    /// 没带上面两样的行读数条数（插计数脚本修订 4 之前的产物）：不为 0 时调用处那十行这一格读不到。
    pub(super) lines_without_the_part_to_the_hit: u64,
    pub(super) exclusive_nanos: u64,
    queries: u64,
    pub(super) query_levels: u64,
}

/// 点、状态、发布种类、轮、段、行。
pub(super) type CellKey = (String, String, String, u64, String, String);
/// 点、状态、发布种类、轮。
pub(super) type RoundKey = (String, String, String, u64);

#[derive(Clone, Default)]
pub(super) struct Product {
    pub(super) constants: Fields,
    pub(super) sizes: Fields,
    pub(super) header: Fields,
    pub(super) points: BTreeMap<String, PointAxes>,
    pub(super) rows: BTreeMap<CellKey, RowCell>,
    pub(super) states: BTreeMap<RoundKey, Fields>,
    pub(super) publishes: BTreeMap<RoundKey, Fields>,
    pub(super) deltas: Vec<Fields>,
    pub(super) segment_nanos: BTreeMap<(String, String, String, u64, String), u64>,
    pub(super) items_done: BTreeSet<String>,
    /// 被量的 unlink 每一轮判了几次：(点, 状态, 轮) → 次数。
    pub(super) unlink_publishes: BTreeMap<(String, String, u64), u64>,
    /// 历史第 4 步 u 次 unlink 每次判了几次：点 → (次数, 最少, 最多)。
    pub(super) history_unlinks: BTreeMap<String, (u64, u64, u64)>,
    /// 被量的覆盖写补满五轮用了几次尝试：(点, 状态) → (取到, 尝试)。
    pub(super) rounds_taken: BTreeMap<(String, String), (u64, u64)>,
}

pub(super) fn read_product(text: &str) -> Product {
    let mut product = Product::default();
    for fields in text.lines().filter_map(parse_line) {
        let get = |key: &str| fields.get(key).cloned().unwrap_or_default();
        let round = number(&fields, "round").unwrap_or(0);
        let round_key = (get("point"), get("state"), get("kind"), round);
        match fields["name"].as_str() {
            "constants" => product.constants = fields.clone(),
            "sizes" => product.sizes = fields.clone(),
            "header" => product.header = fields.clone(),
            "point" => {
                let axes = PointAxes {
                    unit_area_slots: number(&fields, "unit_area_slots").unwrap_or(0),
                    unlinks: number(&fields, "unlinks").unwrap_or(0),
                    data_units: number(&fields, "data_units").unwrap_or(0),
                    files: number(&fields, "files").unwrap_or(0),
                };
                product.points.insert(get("point"), axes);
            }
            "row" => {
                let key = (
                    get("point"),
                    get("state"),
                    get("kind"),
                    round,
                    get("segment"),
                    get("row"),
                );
                let cell = product.rows.entry(key).or_default();
                cell.calls += number(&fields, "calls").unwrap_or(0);
                cell.steps += number(&fields, "steps").unwrap_or(0);
                cell.scale += number(&fields, "scale").unwrap_or(0);
                match (
                    number(&fields, "steps_to_the_hit"),
                    number(&fields, "scale_to_the_hit"),
                ) {
                    (Some(steps_to_the_hit), Some(scale_to_the_hit)) => {
                        cell.steps_to_the_hit += steps_to_the_hit;
                        cell.scale_to_the_hit += scale_to_the_hit;
                    }
                    _ => cell.lines_without_the_part_to_the_hit += 1,
                }
                cell.exclusive_nanos += number(&fields, "exclusive_nanos").unwrap_or(0);
                cell.queries += number(&fields, "queries").unwrap_or(0);
                cell.query_levels += number(&fields, "query_levels").unwrap_or(0);
            }
            "state" => {
                product.states.insert(round_key, fields.clone());
            }
            "publish" => {
                product.publishes.insert(round_key, fields.clone());
            }
            "delta" => product.deltas.push(fields.clone()),
            "segment_nanos" => {
                let key = (
                    get("point"),
                    get("state"),
                    get("kind"),
                    round,
                    get("segment"),
                );
                *product.segment_nanos.entry(key).or_default() +=
                    number(&fields, "nanos").unwrap_or(0);
            }
            "unlink_publishes" => {
                product.unlink_publishes.insert(
                    (get("point"), get("state"), round),
                    number(&fields, "publishes").unwrap_or(0),
                );
            }
            "history_unlinks" => {
                product.history_unlinks.insert(
                    get("point"),
                    (
                        number(&fields, "count").unwrap_or(0),
                        number(&fields, "fewest").unwrap_or(0),
                        number(&fields, "most").unwrap_or(0),
                    ),
                );
            }
            "rounds_taken" => {
                product.rounds_taken.insert(
                    (get("point"), get("state")),
                    (
                        number(&fields, "taken").unwrap_or(0),
                        number(&fields, "attempts").unwrap_or(0),
                    ),
                );
            }
            "item_done" => {
                product.items_done.insert(get("point"));
            }
            _ => {}
        }
    }
    product
}

// ---- 清单（登记第三节）：计数名、类型（登记里的 s 号只在登记与报告正文里用，产物与模型里一律写计数名）----
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RowKind {
    Scan,
    Copy,
    NegativeControl,
}

pub(super) const ROW_TABLE: [(&str, RowKind); 36] = [
    ("whole_allocator_copy_before_the_judgement", RowKind::Scan),
    ("inode_leaf_container_copy", RowKind::Copy),
    ("inode_leaf_writer_copy", RowKind::Copy),
    ("inode_record_linear_find", RowKind::Scan),
    ("files_other_than_the_first_plan", RowKind::Scan),
    ("files_other_than_the_first_lower_node_copy", RowKind::Copy),
    ("file_other_than_the_first_linear_find", RowKind::Scan),
    ("inodes_with_an_extent_entry_set", RowKind::Scan),
    ("extent_upper_segment_plan", RowKind::Scan),
    ("tombstone_container_copy", RowKind::Copy),
    ("retirement_container_copy", RowKind::Copy),
    ("extent_roles_rewritten_scan", RowKind::Scan),
    ("root_header_linear_find", RowKind::Scan),
    ("checkpoint_cost_accounting_tree_plan", RowKind::Scan),
    ("cluster_segment_sum", RowKind::Scan),
    ("allocation_record_tree_node_set", RowKind::Scan),
    ("allocation_record_tree_plan", RowKind::Scan),
    ("lineage_container_copy", RowKind::Copy),
    ("inode_number_watermark_steps", RowKind::Scan),
    ("deadlist_changes_indexes", RowKind::Scan),
    ("multi_level_tree_planning", RowKind::Scan),
    ("mapped_units_of_container_trees", RowKind::Scan),
    ("moved_entries_path_tail_filter", RowKind::Scan),
    ("release_judgement_version_indexes", RowKind::Scan),
    ("whole_allocator_copy_in_a_settling_round", RowKind::Scan),
    ("allocator_release_linear_find", RowKind::Scan),
    ("user_data_slot_linear_scan", RowKind::Scan),
    ("record_intersection_filter", RowKind::Scan),
    ("empty_segment_linear_scan", RowKind::Scan),
    ("fallback_slot_linear_scan", RowKind::Scan),
    ("records_of_each_leaf", RowKind::Scan),
    ("accounting_entries_of_this_publish", RowKind::Scan),
    ("instance_table_chain", RowKind::Scan),
    ("units_of_a_publish_to_land", RowKind::NegativeControl),
    ("mapped_units_whole_table", RowKind::Scan),
    // s37：主 agent 定的，量的一侧（B1）进扫类合计（准入之前三段）；推的一侧（B8、B4）没有这一行的式子，见 `ROW_WITHOUT_A_BY_DELTA_FORMULA`。
    (ROW_WITHOUT_A_BY_DELTA_FORMULA, RowKind::Scan),
];

/// 池级回落按叶取落点（s37）：B8 对这一步的做法没人定过，by_delta_rows 里没有它的式子，B8、B4 合计里它是 0；
/// 一个点只要在准入之前三段里调到过它（pooled_fallback_calls > 0），那一点的 B8、B4 合计就标「缺 s37 一行，只对清单内的步」，不拿这一点判 B8 站不站得住。
pub(super) const ROW_WITHOUT_A_BY_DELTA_FORMULA: &str = "pooled_fallback_by_the_leaf";

pub(super) fn all_rows() -> Vec<(&'static str, RowKind)> {
    ROW_TABLE.to_vec()
}

fn is_a_scan_row(name: &str) -> bool {
    ROW_TABLE
        .iter()
        .any(|(row, kind)| *row == name && *kind == RowKind::Scan)
}

// ---- 读数的取法 ----
/// 清单外归口的一行（插计数时没挂归口守卫的步记在这里，不丢）。
pub(super) const UNLISTED_ROW: &str = "unlisted_index_or_plan_work";
/// 准入之前的三段（登记读法第 1 条）：调用方段、落点判之前、两道之间。
pub(super) const SEGMENTS_BEFORE_ADMISSION: [&str; 3] = ["caller", "before_landing", "between"];
/// 落点判之前的两段（B4 臂的射程）。
pub(super) const SEGMENTS_BEFORE_THE_SECOND_JUDGEMENT: [&str; 2] = ["caller", "before_landing"];
pub(super) const STATES: [&str; 2] = ["same_mount", "remounted"];
pub(super) const KINDS: [&str; 3] = ["overwrite", "unlink_intent", "unlink_final"];
pub(super) const AXES: [&str; 4] = ["U", "u", "D", "F"];
/// 计时每个点量几轮。
pub(super) const ROUNDS: u64 = 5;

#[cfg(test)]
#[path = "timing_formulas_and_row_reading_tests.rs"]
mod tests;
