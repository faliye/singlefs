//! 行表里的函数行与位置、世界、常量与历史与声明序列的核对、闭包核对、孪生差异、集合的比较。

use super::*;

/// 源文件里 `fn <函数名>` 恰好出现在一行的行号（1 起）；0 行或多行交 `None`（位置待定）。
pub(super) fn line_of_the_function(source: &str, function: &str) -> Option<usize> {
    if function.is_empty() {
        return None;
    }
    let heads = [format!("fn {function}("), format!("fn {function}<")];
    let lines: Vec<usize> = source.lines().enumerate().filter(|(_, line)| heads.iter().any(|head| line.contains(head.as_str()))).map(|(position, _)| position + 1).collect();
    (lines.len() == 1).then(|| lines[0])
}

/// 行名在 `crates/` 里的位置：`<文件>:<行>`；没给仓根或查不到交 `position_pending`。
pub(super) fn position_of_the_row(row: &str, crates_root: Option<&str>) -> String {
    let Some((_, file, function)) = ROW_FUNCTIONS.iter().find(|(name, _, _)| *name == row) else {
        return "position_pending".to_string();
    };
    let Some(root) = crates_root else {
        return "position_pending".to_string();
    };
    let path = format!("{root}/crates/singlefs-core/src/{file}");
    let line = std::fs::read_to_string(&path).ok().and_then(|source| line_of_the_function(&source, function));
    line.map_or_else(|| "position_pending".to_string(), |line| format!("crates/singlefs-core/src/{file}:{line}"))
}

/// Q13：每个额外节点多几步，保留 3 位小数。多出的步数或多出的节点不是正数时没有这个比（旗上操作没比可比操作多步、或没多节点），记没有。
pub(super) fn steps_per_extra_node(extra_steps: i64, extra_nodes: i64) -> Option<f64> {
    (extra_steps > 0 && extra_nodes > 0).then(|| (extra_steps as f64 / extra_nodes as f64 * 1000.0).round() / 1000.0)
}

// ---- 装置与实现对得上的检查（第七节 ②、③；停机条款）----

pub(super) fn yes_no(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

/// 一个项的读数世界：不是孪生的项、各项的操作视图。
pub(super) struct World<'items> {
    pub(super) items: Vec<&'items MeasuredItem>,
    pub(super) views: BTreeMap<String, Vec<OperationView>>,
}

impl<'items> World<'items> {
    pub(super) fn new(items: &'items [MeasuredItem]) -> Self {
        let real: Vec<&MeasuredItem> = items.iter().filter(|item| !item.is_twin).collect();
        let views = real.iter().map(|item| (item.name.clone(), views_of_the_item(item))).collect();
        Self { items: real, views }
    }

    pub(super) fn head_item(&self, point: &str, script: &str) -> Option<&'items MeasuredItem> {
        let name = scripts::item_name(point, "head", script);
        self.items.iter().copied().find(|item| item.name == name)
    }

    pub(super) fn views_of(&self, point: &str, script: &str) -> Option<&Vec<OperationView>> {
        self.views.get(&scripts::item_name(point, "head", script))
    }

    pub(super) fn points(&self) -> BTreeSet<String> {
        self.items.iter().filter(|item| item.arm == "head").map(|item| item.point.clone()).collect()
    }

    pub(super) fn all_views_at(&self, point: &str) -> Vec<&OperationView> {
        self.items.iter().filter(|item| item.arm == "head" && item.point == point).flat_map(|item| self.views[&item.name].iter()).collect()
    }

    fn views_at_the_points(&self, points: &[&str]) -> Vec<&OperationView> {
        points.iter().flat_map(|point| self.all_views_at(point)).collect()
    }

    /// 一个点上每个脚本第 0 次挂载里的操作。
    pub(super) fn first_mount_by_script(&self, point: &str) -> BTreeMap<String, Vec<&OperationView>> {
        self.items
            .iter()
            .filter(|item| item.arm == "head" && item.point == point)
            .map(|item| (item.script.clone(), first_mount(&self.views[&item.name])))
            .collect()
    }
}

/// 第七节 ② 第 1 条：产物头里实现的常量与本地常量逐个比；不等的项数。
/// 只比计数版：登记第七节 ② 第 1 条写的是「与计数版产物头逐个比」，plain 版只拷不插、产物里没有常量行，不核。
pub(super) fn constants_mismatches(item: &MeasuredItem) -> Vec<String> {
    let mut wrong = Vec::new();
    if item.arm == "plain" {
        return wrong;
    }
    for (key, expected) in [("leaf_slots", LEAF_SLOTS), ("internal_fanout", INTERNAL_FANOUT), ("settling_rounds_at_most", SETTLING_ROUNDS_LIMIT)] {
        if number(&item.constants, key) != Some(expected) {
            wrong.push(format!("{key}={:?}≠{expected}", number(&item.constants, key)));
        }
    }
    wrong
}

/// 第七节 ③ 第 1 条与 ② 第 2 条：历史里用户改动数等于 `2 + 2F + 2 × 10`。
pub(super) fn history_mismatch(item: &MeasuredItem) -> Option<String> {
    let expected = scripts::notes_of_the_history(item.files);
    (item.history_user_changes != Some(expected)).then(|| format!("user_changes={:?}≠{expected}", item.history_user_changes))
}

/// F12：产物里 `script_step` 行的步序列与脚本表逐步比；有一处不同交描述。
pub(super) fn declared_sequence_mismatch(item: &MeasuredItem) -> Option<String> {
    let expected = scripts::script_steps(&item.script)?;
    let seen: Vec<&str> = item.step_lines.iter().map(|(_, name, _)| name.as_str()).collect();
    let wanted: Vec<String> = expected.iter().map(|step| step.text()).collect();
    if seen.len() != wanted.len() {
        return Some(format!("步数 {} 对脚本表 {}", seen.len(), wanted.len()));
    }
    let declared_operations: Vec<&str> = item.operations.iter().map(|operation| operation.name.as_str()).collect();
    let wanted_operations: Vec<String> = expected.iter().filter(|step| step.is_a_user_operation()).map(|step| step.text()).collect();
    seen.iter()
        .zip(&wanted)
        .position(|(left, right)| *left != right)
        .map(|position| format!("第 {} 步 {} 对脚本表 {}", position + 1, seen[position], wanted[position]))
        .or_else(|| (declared_operations != wanted_operations).then(|| "声明行的操作序列与脚本表不同".to_string()))
}

/// 闭包模型对拍（第七节 ③ 第 3 条）：每个发布上独立模型算出的闭包与测到的 `seed ∪ pulled` 是否相等；另核 `final ⊇ seed ∪ pulled`、`final_nodes = |final|`。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ClosureCheck {
    pub(super) publishes: usize,
    pub(super) closure_mismatches: Vec<String>,
    pub(super) final_not_covering: usize,
    pub(super) final_nodes_mismatches: usize,
    pub(super) walk_mismatches: usize,
}

fn check_publish(record: &PublishRecord, label: &str, check: &mut ClosureCheck) {
    check.publishes += 1;
    let measured: NodeSet = record.seed.union(&record.pulled).copied().collect();
    let model = release_chain_closure(&record.graph, record.root_level, &record.seed);
    if model != measured {
        check.closure_mismatches.push(format!("{label}:model={}:measured={}", model.len(), measured.len()));
    }
    if !measured.is_subset(&record.final_set) {
        check.final_not_covering += 1;
    }
    if !record.walk_matches {
        check.walk_mismatches += 1;
    }
}

pub(super) fn closure_check(item: &MeasuredItem) -> ClosureCheck {
    let mut check = ClosureCheck::default();
    for operation in &item.operations {
        for (position, record) in operation.publishes.iter().enumerate() {
            check_publish(record, &format!("{}#{}/{}", item.name, operation.step_index, position), &mut check);
            let settled = operation.settles.iter().find(|settle| settle.settle_call == record.settle_call);
            if settled.is_some_and(|settle| settle.final_nodes != record.final_set.len() as u64) {
                check.final_nodes_mismatches += 1;
            }
        }
    }
    for remount in &item.remounts {
        for (position, record) in remount.publishes.iter().enumerate() {
            check_publish(record, &format!("{}#remount{}/{}", item.name, remount.epoch, position), &mut check);
            let settled = remount.settles.iter().find(|settle| settle.settle_call == record.settle_call);
            if settled.is_some_and(|settle| settle.final_nodes != record.final_set.len() as u64) {
                check.final_nodes_mismatches += 1;
            }
        }
    }
    check
}

/// 第七节 ③ 第 5 条：确定性孪生项与原项去掉产物头之后逐字节相同；不同交第一处不同的行号。
pub(super) fn twin_difference(original: &MeasuredItem, twin: &MeasuredItem) -> Option<usize> {
    let longer = original.body_lines.len().max(twin.body_lines.len());
    (0..longer).find(|position| original.body_lines.get(*position) != twin.body_lines.get(*position)).map(|position| position + 1)
}

/// 第七节 ③ 第 4 条：计数版与 plain 版在历史末与脚本末的 `image_sha256` 是否逐字节相同。
pub(super) fn plain_matches_counts(counts: &MeasuredItem, plain: &MeasuredItem) -> bool {
    let pick = |item: &MeasuredItem, when: &str| item.images.iter().find(|(label, _)| label == when).map(|(_, sha)| sha.clone());
    ["history_end", "script_end"].iter().all(|when| pick(counts, when).is_some() && pick(counts, when) == pick(plain, when))
}

/// 第七节 ③ 第 6 条：历史末的 `mapped_units` ≥ F × K + 10（第一个量窗操作之前那一行规模读数）。
pub(super) fn live_units_shortfall(item: &MeasuredItem) -> Option<String> {
    let lower = item.files * item.units_per_file + scripts::BIG_FILE_UNITS;
    match number(&item.first_state, "mapped_units") {
        Some(mapped) if mapped >= lower => None,
        other => Some(format!("mapped_units={other:?}<{lower}")),
    }
}

/// 第七节 ③ 第 8 条：阴性对照（`first_repeated` 末尾的 `truncate_to_the_same_size`）轮数不超过 4。
pub(super) fn negative_control_rounds(item: &MeasuredItem) -> Option<u64> {
    item.operations.iter().find(|operation| operation.name == "truncate_to_the_same_size").and_then(|operation| operation.settles.iter().map(|settle| settle.rounds).max())
}

// ---- 报告：每个量各占一行，各报各的判定（第六节）----

pub(super) fn list_text(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(",")
    }
}

pub(super) fn first_labels(labels: &[String]) -> String {
    list_text(&labels.iter().take(5).cloned().collect::<Vec<_>>())
}

/// 训练集、检验集一、检验集二（第六节开头）。
pub(super) struct Sets<'views> {
    pub(super) training: Vec<&'views OperationView>,
    pub(super) script: Vec<&'views OperationView>,
    pub(super) geometry: Vec<&'views OperationView>,
}

pub(super) fn sets_of<'views>(world: &'views World<'_>) -> Sets<'views> {
    let training = world
        .all_views_at(scripts::TRAINING_POINT)
        .into_iter()
        .filter(|view| TRAINING_SCRIPTS.contains(&view.script.as_str()) && view.epoch == 0)
        .collect();
    let script = world.all_views_at(scripts::TRAINING_POINT).into_iter().filter(|view| !TRAINING_SCRIPTS.contains(&view.script.as_str())).collect();
    let geometry = world.views_at_the_points(&scripts::TEST_GEOMETRY_POINTS);
    Sets { training, script, geometry }
}
