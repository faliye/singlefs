//! 读数里取出每轮最大值、按增量的判定、窗口与点的读数、每次操作的步数与固定点的臂选择。

use super::*;

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
fn by_delta_input(
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
fn pooled_fallback_of_the_judgement(judgement: &JudgementReadings) -> (u64, u64, u64) {
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
fn by_delta_of_the_judgement(
    judgement: &JudgementReadings,
    unit_area_slots: u64,
) -> Result<u64, Vec<String>> {
    let mut total = 0;
    for segment in SEGMENTS_BEFORE_ADMISSION {
        total += by_delta_total(&by_delta_input(judgement, segment, unit_area_slots)?);
    }
    Ok(total)
}

fn judgement_of(item: &PointArmReadings, settle: &Settle) -> Option<JudgementReadings> {
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

pub(super) type ReadingsByPointAndArm<'items> = BTreeMap<(String, String), &'items PointArmReadings>;

pub(super) fn window_settle<'items>(
    item: &'items PointArmReadings,
    operation_name: &str,
    operation_index: u64,
) -> Option<&'items Settle> {
    item.window.iter().find(|settle| {
        settle.state == "same_mount" && settle.operation_name == operation_name && settle.operation_index == operation_index
    })
}

// ---- 每次操作（第一节读法第 1–3 条）----
/// E206 产物没有 `units_per_small_file`：点名里带 `_units_<K>` 的取 K，别的点的小文件各 1 个单元（2999 字节）。
pub(super) fn units_by_the_point_name(point: &str) -> u64 {
    point
        .split("_units_")
        .nth(1)
        .and_then(|rest| rest.split('_').next())
        .and_then(|text| text.parse().ok())
        .unwrap_or(1)
}

/// 一次固定点的 B8 合计（三段之和，读法第 1 条照 E206 模型 `by_delta_of_settle` 的口径，不带 P3）；读不到交 `None`。
pub(super) fn by_delta_of_settle(item: &PointArmReadings, settle: &Settle) -> Option<u64> {
    let judgement = judgement_of(item, settle)?;
    by_delta_of_the_judgement(&judgement, item.unit_area_slots).ok()
}

/// B8 里含 `lg(Z)` 因子的行：(行名, 系数, Z)，这一行的步数里 `lg(Z)` 那一项 = 系数 × lg(Z)（控制臂 `C3` 把它换成系数 × Z）。
pub(super) fn lg_terms(input: &ByDeltaInput) -> Vec<(&'static str, u64, u64)> {
    let touched = input.touched_files;
    let calls = |row: &str| input.calls_in_the_row(row);
    vec![
        ("inode_record_linear_find", calls("inode_record_linear_find"), input.inode_leaf_containers),
        ("files_other_than_the_first_plan", calls("files_other_than_the_first_plan") * touched, input.other_files),
        ("file_other_than_the_first_linear_find", calls("file_other_than_the_first_linear_find"), input.other_files),
        ("inodes_with_an_extent_entry_set", calls("inodes_with_an_extent_entry_set") * touched, input.other_files),
        (
            "extent_upper_segment_plan",
            calls("extent_upper_segment_plan") * touched * input.extent_upper_levels,
            input.extent_upper_nodes,
        ),
        ("deadlist_changes_indexes", input.deadlist_changed_entries, input.deadlist_entries),
        ("allocator_release_linear_find", calls("allocator_release_linear_find"), input.allocation_records),
        ("record_intersection_filter", calls("record_intersection_filter"), input.allocation_records),
        ("mapped_units_whole_table", input.mapped_units_changed_over_all_rounds, input.mapped_units),
    ]
}

/// 含 `lg` 的一行在一次操作里的三样：(系数 × lg(Z), 系数 × Z, 这一行的 B8 步数)。
pub(super) type LgRow = (u64, u64, u64);

/// 一次固定点三段里每个含 `lg` 的行：行名 → (系数 × lg(Z), 系数 × Z, 这一行的步数)。
fn lg_rows_of_settle(item: &PointArmReadings, settle: &Settle) -> Option<BTreeMap<&'static str, LgRow>> {
    let judgement = judgement_of(item, settle)?;
    let mut rows: BTreeMap<&'static str, LgRow> = BTreeMap::new();
    for segment in SEGMENTS_BEFORE_ADMISSION {
        let input = by_delta_input(&judgement, segment, item.unit_area_slots).ok()?;
        let steps_of_the_rows = by_delta_rows(&input);
        for (row, coefficient, lg_argument) in lg_terms(&input) {
            let cell = rows.entry(row).or_default();
            cell.0 += coefficient * lg(lg_argument);
            cell.1 += coefficient * lg_argument;
            cell.2 += steps_of_the_rows.iter().find(|(name, _)| *name == row).map_or(0, |(_, steps)| *steps);
        }
    }
    Some(rows)
}

/// 一次被量的操作（读法第 1 条：量窗里 `Measured::measure` 的一次调用，含它的几次固定点）。
#[derive(Clone, Debug, Default)]
pub(super) struct Operation {
    pub(super) state: String,
    pub(super) operation_name: String,
    pub(super) operation_index: u64,
    /// 这个状态里的次序（0 起，照产物里第一次出现）。
    pub(super) order: usize,
    /// 读法第 2 条：外生的用户单元数（E207 读声明行；E206 产物按构造补）。
    pub(super) user_units: Option<u64>,
    pub(super) old_units: u64,
    pub(super) new_units: u64,
    /// 读法第 3 条：操作之前的 `allocation_records`（主读）与 `mapped_units`（另报）。
    pub(super) allocation_records: Option<u64>,
    pub(super) mapped_units: Option<u64>,
    /// 旧判据的旋钮 x（点的 F × K 或 E206 的轴位置），只与旧判据对照，不进判定。
    pub(super) knob: u64,
    /// 各次固定点 B8 合计之和；有一次读不到就是 `None`。
    pub(super) steps: Option<u64>,
    /// 几次固定点里 final_nodes 最大的那一次（尖峰旗吃它），与它在 `item.window` 里的下标。
    pub(super) final_nodes: u64,
    pub(super) peak_settle: Option<usize>,
    /// 各轮释放的落点数之和（`Rs`，只给变异 M3 的对照，不进判定）。
    pub(super) releases: u64,
    /// 几次固定点在准入之前三段里 s37 的调用次数 pooled_fallback_calls、s37 的步数、s29 的步数；pooled_fallback_calls > 0 的操作 B8、B4 合计缺 s37 一行（见 `ROW_WITHOUT_A_BY_DELTA_FORMULA`）。
    pub(super) pooled_fallback_calls: u64,
    /// 操作之前的规模读数行里 `frozen_publish_set`（发布落盘失败冻结着等重发时是 1）；读不到交 `None`（旧产物没有这个字段）。
    pub(super) frozen_publish_set: Option<u64>,
    pub(super) pooled_fallback_steps: u64,
    pub(super) fallback_scan_steps: u64,
    /// 含 `lg` 的行：行名 → (系数 × lg(Z), 系数 × Z)，几次固定点相加。
    pub(super) lg_rows: BTreeMap<&'static str, LgRow>,
    pub(super) flagged: bool,
}

/// 读法第 3 条：量到的总量 x 取操作之前的 `allocation_records`。
pub(super) fn measured_total(operation: &Operation) -> Option<f64> {
    operation.allocation_records.map(|records| records as f64)
}

/// 读法第 3 条的另报：x 取 `mapped_units`。
pub(super) fn measured_total_by_mapped_units(operation: &Operation) -> Option<f64> {
    operation.mapped_units.map(|units| units as f64)
}

/// 读法第 5 条的分母：外生的 `user_units`，不用实现算出来的 `Rs`。
pub(super) fn denominator(operation: &Operation) -> Option<u64> {
    operation.user_units
}

/// E206 产物没有声明行：按 E206 历史的构造补（读法第 2 条，只用于回扫）。`live` = [大文件（D 个单元）, F 个小文件（各 K 个）]，
/// W2 第 i 次取 `live[(i − 1) × len ÷ 32 % len]`，一个文件第一次被覆盖写时是它建时的单元数、之后是 1；W1、W4 覆盖写第一个文件（1）；
/// W3 一次建—写—删（1 + 1 + 1）；阴性对照（1）。别的操作不补。
pub(super) fn constructed_user_units(item: &PointArmReadings, operation_name: &str, operation_index: u64, touched: &mut BTreeSet<usize>) -> Option<(u64, u64, u64)> {
    match operation_name {
        "W1" | "W4" => Some((1, 1, 1)),
        "W3" => Some((3, 0, 0)),
        "negative_control" => Some((1, 1, 1)),
        "W2" => {
            let length = usize::try_from(item.files + 1).ok()?;
            let share = usize::try_from(operation_index.checked_sub(1)?).ok()?;
            let position = share * length / 32 % length;
            let built = if position == 0 { item.data_units } else { item.units_per_small_file };
            let old = if touched.insert(position) { built } else { 1 };
            Some((old.max(1), old, 1))
        }
        _ => None,
    }
}

/// 一份产物的全部被量操作，照状态分开、照第一次出现的次序；B8、x、声明与尖峰旗都在这里接上。
pub(super) fn operations_of(item: &PointArmReadings) -> Vec<Operation> {
    let mut operations: Vec<Operation> = Vec::new();
    let mut position_of: BTreeMap<(String, String, u64), usize> = BTreeMap::new();
    for (index, settle) in item.window.iter().enumerate() {
        let key = (settle.state.clone(), settle.operation_name.clone(), settle.operation_index);
        let at = *position_of.entry(key).or_insert_with(|| {
            let order = operations.iter().filter(|operation| operation.state == settle.state).count();
            operations.push(Operation {
                state: settle.state.clone(),
                operation_name: settle.operation_name.clone(),
                operation_index: settle.operation_index,
                order,
                steps: Some(0),
                knob: item.files * item.units_per_small_file,
                ..Operation::default()
            });
            operations.len() - 1
        });
        let operation = &mut operations[at];
        operation.steps = operation.steps.zip(by_delta_of_settle(item, settle)).map(|(sum, steps)| sum + steps);
        if operation.peak_settle.is_none() || settle.final_nodes > operation.final_nodes {
            operation.final_nodes = settle.final_nodes;
            operation.peak_settle = Some(index);
        }
        if let Some(judgement) = judgement_of(item, settle) {
            let mut gaps = Vec::new();
            operation.releases += per_round_maxima(&judgement, "round_placements_released", settle.rounds, &mut gaps).iter().sum::<u64>();
            let (calls, steps, per_disk_fallback_steps) = pooled_fallback_of_the_judgement(&judgement);
            operation.pooled_fallback_calls += calls;
            operation.pooled_fallback_steps += steps;
            operation.fallback_scan_steps += per_disk_fallback_steps;
        }
        match lg_rows_of_settle(item, settle) {
            Some(rows) => {
                for (row, (lg_part, linear_part, row_steps)) in rows {
                    let cell = operation.lg_rows.entry(row).or_default();
                    cell.0 += lg_part;
                    cell.1 += linear_part;
                    cell.2 += row_steps;
                }
            }
            None => operation.steps = None,
        }
    }
    let declared: BTreeMap<&(String, String, u64), &(u64, u64, u64)> = item.declared.iter().map(|(key, value)| (key, value)).collect();
    let mut touched = BTreeSet::new();
    for operation in &mut operations {
        let key = (operation.state.clone(), operation.operation_name.clone(), operation.operation_index);
        let units = if item.declared.is_empty() {
            constructed_user_units(item, &operation.operation_name, operation.operation_index, &mut touched)
        } else {
            declared.get(&key).map(|value| **value)
        };
        if let Some((user_units, old_units, new_units)) = units {
            operation.user_units = Some(user_units);
            operation.old_units = old_units;
            operation.new_units = new_units;
        }
        let prefix = format!("{}/{}/{}/", operation.state, operation.operation_name, operation.operation_index);
        if let Some(judgement) = item.judgements.get(&prefix) {
            operation.allocation_records = number(&judgement.state, "allocation_records");
            operation.mapped_units = number(&judgement.state, "mapped_units");
            operation.frozen_publish_set = number(&judgement.state, "frozen_publish_set");
        }
    }
    for state in ["same_mount", "remounted"] {
        let nodes: Vec<u64> = operations.iter().filter(|operation| operation.state == state).map(|operation| operation.final_nodes).collect();
        let line = spike_line(&nodes);
        for operation in operations.iter_mut().filter(|operation| operation.state == state) {
            operation.flagged = operation.final_nodes >= line;
        }
    }
    operations
}

/// 下中位数：排好序取第 ⌊(n − 1) ÷ 2⌋ 个；空的交 0。
pub(super) fn lower_median(values: &[u64]) -> u64 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted.get(sorted.len().saturating_sub(1) / 2).copied().unwrap_or(0)
}

/// 读法第 8 条的旗线：max(28, 4 × 中位数)。
pub(super) fn spike_line(final_nodes: &[u64]) -> u64 {
    SPIKE_FLOOR.max(SPIKE_MULTIPLE * lower_median(final_nodes))
}

// ---- 被测臂与控制臂（第五节阳性对照表）：控制臂在同一批读数上加项，同一把尺判 ----
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Arm {
    Tested,
    ScanEveryPublish,
    ScanEveryPublishFourTimes,
    LargestLgRowLinear,
}

pub(super) const ARMS: [Arm; 4] = [Arm::Tested, Arm::ScanEveryPublish, Arm::ScanEveryPublishFourTimes, Arm::LargestLgRowLinear];

impl Arm {
    pub(super) fn tag(self) -> &'static str {
        match self {
            Self::Tested => "tested",
            Self::ScanEveryPublish => "c1",
            Self::ScanEveryPublishFourTimes => "c2",
            Self::LargestLgRowLinear => "c3",
        }
    }
}

/// `C3` 换线性的那一行：这个点这个状态量窗全部操作里，含 `lg` 的行中步数合计最大的一行（同值取名字在前的）。
pub(super) fn largest_lg_row(operations: &[&Operation]) -> Option<&'static str> {
    let mut totals: BTreeMap<&'static str, u64> = BTreeMap::new();
    for operation in operations {
        for (row, (_, _, row_steps)) in &operation.lg_rows {
            *totals.entry(row).or_default() += row_steps;
        }
    }
    totals.iter().fold(None, |best: Option<(&'static str, u64)>, (row, total)| match best {
        Some((_, best_total)) if best_total >= *total => best,
        _ => Some((*row, *total)),
    })
    .map(|(row, _)| row)
}

/// 一次操作在一条臂下的步数：`C1` 加操作之前的记录数 × 1，`C2` × 4，`C3` 把选中那一行的系数 × lg(Z) 换成系数 × Z。
pub(super) fn arm_steps(operation: &Operation, arm: Arm, linear_row: Option<&str>) -> Option<u64> {
    let steps = operation.steps?;
    match arm {
        Arm::Tested => Some(steps),
        Arm::ScanEveryPublish => Some(steps + SCAN_EVERY_PUBLISH_STEPS_PER_RECORD * operation.allocation_records?),
        Arm::ScanEveryPublishFourTimes => Some(steps + SCAN_EVERY_PUBLISH_FOUR_TIMES_STEPS_PER_RECORD * operation.allocation_records?),
        Arm::LargestLgRowLinear => {
            let (lg_part, linear_part, _) = operation.lg_rows.get(linear_row?).copied().unwrap_or_default();
            Some(steps.checked_sub(lg_part)? + linear_part)
        }
    }
}

// ---- 读法第 7 条：第一次与后来的 ----
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Pick {
    First,
    LaterPeak,
    LaterMedian,
}

pub(super) const PICKS: [Pick; 3] = [Pick::First, Pick::LaterPeak, Pick::LaterMedian];

impl Pick {
    pub(super) fn tag(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::LaterPeak => "later_peak",
            Self::LaterMedian => "later_median",
        }
    }
}

/// 一列照次序排好的 (操作, 步数) 里取一样：`first` = 次序第一的；`later_peak` = 其余里步数最大的（同值取在前的）；
/// `later_median` = 其余按步数排好序的下中位数。
pub(super) fn pick_from<'operations>(series: &[(&'operations Operation, u64)], pick: Pick) -> Option<(&'operations Operation, u64)> {
    match pick {
        Pick::First => series.first().copied(),
        Pick::LaterPeak => series.iter().skip(1).copied().fold(None, |best: Option<(&Operation, u64)>, candidate| match best {
            Some((_, steps)) if steps >= candidate.1 => best,
            _ => Some(candidate),
        }),
        Pick::LaterMedian => {
            let mut later: Vec<(&Operation, u64)> = series.iter().skip(1).copied().collect();
            later.sort_by_key(|(_, steps)| *steps);
            later.get(later.len().saturating_sub(1) / 2).copied()
        }
    }
}

/// 读法第 8 条：`steady` 剔除旗上的操作，`all` 不剔除。
pub(super) fn kept_for_the_filter(operation: &Operation, steady: bool) -> bool {
    !(steady && operation.flagged)
}

/// 读法第 7 条的档：`one_user_unit` = user_units 为 1；`user_units_equal_declared` = user_units 等于这个点声明的 K（K ≥ 100）。
pub(super) fn in_the_tier(operation: &Operation, tier: &str, units_per_small_file: u64) -> bool {
    match tier {
        "one_user_unit" => operation.user_units == Some(1),
        "user_units_equal_declared" => units_per_small_file >= LARGE_CHANGE_UNITS_AT_LEAST && operation.user_units == Some(units_per_small_file),
        _ => false,
    }
}

// ---- 读法第 4、5 条：两点读数 ----
/// 一端：(步数, user_units, x)。
pub(super) type End = (u64, u64, f64);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TwoPointReading {
    pub(super) looser: bool,
    pub(super) alpha: Option<f64>,
    pub(super) raw_alpha: Option<f64>,
}

pub(super) fn positive_alpha(low_quantity: f64, high_quantity: f64, low_measured_total: f64, high_measured_total: f64) -> Option<f64> {
    (low_quantity > 0.0 && high_quantity > 0.0 && low_measured_total > 0.0 && high_measured_total > 0.0 && low_measured_total != high_measured_total)
        .then(|| alpha(low_quantity, high_quantity, low_measured_total, high_measured_total))
}

/// 读法第 5 条：两端 user_units 相同读原步数；不同读步数 ÷ user_units，标 `looser`，同时报原步数的 α。
pub(super) fn two_point_reading(low: End, high: End) -> TwoPointReading {
    let looser = low.1 != high.1;
    let quantity = |end: End| if looser { end.0 as f64 / end.1 as f64 } else { end.0 as f64 };
    TwoPointReading {
        looser,
        alpha: positive_alpha(quantity(low), quantity(high), low.2, high.2),
        raw_alpha: positive_alpha(low.0 as f64, high.0 as f64, low.2, high.2),
    }
}

/// 三点以上：相邻两点的局部斜率（第八节「局部斜率」）。
pub(super) fn local_slopes(points: &[(f64, f64)]) -> Vec<Option<f64>> {
    points.windows(2).map(|pair| positive_alpha(pair[0].0, pair[1].0, pair[0].1, pair[1].1)).collect()
}

pub(super) fn alpha_text(value: Option<f64>) -> String {
    value.map_or("unread".to_string(), |value| format!("{value:.3}"))
}

pub(super) fn band_text(value: Option<f64>) -> &'static str {
    value.map_or("unread", |value| band(value).tag())
}

// ---- 比较集合（读法第 6 条）与格 ----
/// 一个点读好的东西：产物与它的操作。
pub(super) struct PointReadings<'items> {
    pub(super) item: &'items PointArmReadings,
    pub(super) operations: Vec<Operation>,
}

pub(super) type Points<'items> = BTreeMap<String, PointReadings<'items>>;

/// 报告行：各报告函数往这里推，`report` 读完全部产物之后逐行打；单测直接读它。
pub(super) type Lines = Vec<String>;

/// 一个比较集合：段（`new` = 第一至三段的新点、`big` = 第四段、`retro` = 回扫）、名字、点（低到高）、判哪几档。
pub(super) struct ComparisonSet {
    pub(super) segment: &'static str,
    pub(super) name: &'static str,
    pub(super) points: &'static [&'static str],
    pub(super) tiers: &'static [&'static str],
}

pub(super) const NEW_POINT_SETS: [ComparisonSet; 6] = [
    ComparisonSet { segment: "new", name: "K100", points: &["files_100_units_100", "files_1000_units_100"], tiers: &["one_user_unit", "user_units_equal_declared"] },
    ComparisonSet { segment: "new", name: "K1000", points: &["files_10_units_1000", "files_100_units_1000"], tiers: &["one_user_unit", "user_units_equal_declared"] },
    ComparisonSet { segment: "new", name: "FTB", points: &["files_then_big_1000", "files_then_big_10000"], tiers: &["one_user_unit"] },
    ComparisonSet { segment: "new", name: "D100", points: &["files_100_units_100", "files_100_units_1000"], tiers: &["user_units_equal_declared"] },
    ComparisonSet { segment: "new", name: "D10", points: &["files_10_units_1000", "files_10_units_10000"], tiers: &["user_units_equal_declared"] },
    ComparisonSet {
        segment: "new",
        name: "K1000_other_unit_area",
        points: &["files_10_units_1000_unit_area_2_pow_17", "files_100_units_1000_unit_area_2_pow_20"],
        tiers: &["one_user_unit", "user_units_equal_declared"],
    },
];

pub(super) const BIG_POINT_SETS: [ComparisonSet; 1] = [ComparisonSet {
    segment: "big",
    name: "K1000L",
    points: &["files_10_units_1000_unit_area_2_pow_21", "files_100_units_1000_unit_area_2_pow_21", "files_1000_units_1000"],
    tiers: &["one_user_unit", "user_units_equal_declared"],
}];

pub(super) const RETRO_SETS: [ComparisonSet; 4] = [
    ComparisonSet { segment: "retro", name: "axis_files", points: &["base", "files_100", "files_1000", "files_10000"], tiers: &["one_user_unit"] },
    ComparisonSet {
        segment: "retro",
        name: "axis_data_units",
        points: &["base", "data_units_100", "data_units_1000", "data_units_10000"],
        tiers: &["one_user_unit"],
    },
    ComparisonSet { segment: "retro", name: "axis_unlinks", points: &["base", "unlinks_100", "unlinks_1000", "unlinks_10000"], tiers: &["one_user_unit"] },
    ComparisonSet {
        segment: "retro",
        name: "axis_unit_area",
        points: &["files_1000_unit_area_2_pow_14", "files_1000", "files_1000_unit_area_2_pow_20", "files_1000_unit_area_2_pow_24"],
        tiers: &["one_user_unit"],
    },
];

/// 第六节 Q1–Q3 与几何敏感性那两份（第八节）：(读的是什么, 登记的量号, 档, 集合)。报告行 `reading=` 带第一列，`quantity=` 带量号。
pub(super) const NEW_POINT_QUANTITIES: [(&str, &str, &str, &[&str]); 5] = [
    ("one_unit_change_against_the_total", "q1", "one_user_unit", &["K100", "K1000", "FTB"]),
    ("many_unit_change_against_the_total", "q2", "user_units_equal_declared", &["K100", "K1000"]),
    ("per_unit_steps_as_the_change_grows", "q3", "user_units_equal_declared", &["D100", "D10"]),
    ("many_unit_change_against_the_total_on_the_other_unit_area", "q2_other_unit_area", "user_units_equal_declared", &["K1000_other_unit_area"]),
    ("one_unit_change_against_the_total_on_the_other_unit_area", "q1_other_unit_area", "one_user_unit", &["K1000_other_unit_area"]),
];
/// Q10 与 Q4 在报告行里的 `reading=`。
pub(super) const BIG_POINT_READING: &str = "big_points_against_the_total";
pub(super) const RETRO_READING: &str = "e206_products_read_again";
