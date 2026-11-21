//! 发布与操作与步的输入、操作的视图与计算、步的选择子、项的视图、第一次挂载与旗位。

use super::*;

// ---- 每次操作的视图：旗、新鲜度、攒下的量（读法 4–6、第六节 Q6 的四个量）----

/// 新鲜度的四个变体：`C_mount` 或 `C_ever`（挂载清不清零）× 宽读（`seed ∪ pulled` 对定下集合的并）或窄读（`pulled` 对拉进集合的并）。
pub(super) const FRESHNESS_VARIANTS: [&str; 4] = ["mount_wide", "mount_narrow", "ever_wide", "ever_narrow"];

/// 一次发布喂视图的输入（节点身份）。
#[derive(Clone, Debug, Default)]
pub(super) struct PublishInput {
    pub(super) seed: NodeSet,
    pub(super) pulled: NodeSet,
    pub(super) final_set: NodeSet,
    /// 上一版的节点（真实日志里是指针图的键）。
    pub(super) previous_nodes: NodeSet,
    pub(super) graph: PointerGraph,
    pub(super) root_level: u32,
    pub(super) released_records_pre: u64,
}

#[derive(Clone, Debug, Default)]
pub(super) struct OperationInput {
    pub(super) name: String,
    pub(super) selector: String,
    pub(super) user_units: u64,
    pub(super) old_units: u64,
    pub(super) new_units: u64,
    pub(super) final_nodes: u64,
    pub(super) publishes: Vec<PublishInput>,
}

#[derive(Clone, Debug)]
pub(super) enum StepInput {
    Operation(OperationInput),
    /// 重挂载：它自己的发布（写行、暖机）。
    Remount(Vec<PublishInput>),
}

/// 一次用户操作的视图：判器只读它。
#[derive(Clone, Debug, Default)]
pub(super) struct OperationView {
    pub(super) item: String,
    pub(super) point: String,
    pub(super) script: String,
    pub(super) epoch: u64,
    pub(super) ordinal_user: u64,
    pub(super) step_index: u64,
    pub(super) name: String,
    pub(super) selector: String,
    pub(super) user_units: u64,
    pub(super) old_units: u64,
    pub(super) new_units: u64,
    pub(super) final_nodes: u64,
    /// 三条旗线下是不是旗上（次序同 `FLOOR_LINES`）；`flagged` 是主读那一条。
    pub(super) flag_by_line: [bool; 3],
    pub(super) flagged: bool,
    pub(super) median_inflated: bool,
    /// 四个变体的新鲜度（次序同 `FRESHNESS_VARIANTS`）。
    pub(super) fresh: [u64; 4],
    /// 上一版里本挂载没带出过的节点数（宽读）。
    pub(super) stale_nodes: u64,
    /// 上一版里单节点作种子时释放链的最大层数。
    pub(super) chain_depth: u64,
    /// 进固定点时上一版记录里已释放的条数。
    pub(super) released_backlog: u64,
    /// 本挂载里距上一次旗上操作过了几次发布（没有旗上过的从挂载起算；第 0 次挂载从池建好起算）。
    pub(super) publishes_since_flag: u64,
    pub(super) previous_nodes: u64,
    pub(super) seed: NodeSet,
    pub(super) pulled: NodeSet,
}

impl OperationView {
    /// 行三的八格量：(量名, 绝对值, 除以上一版节点数的相对值)。
    pub(super) fn quantities(&self) -> Vec<(&'static str, f64, f64)> {
        let nodes = self.previous_nodes.max(1) as f64;
        [
            ("stale_nodes", self.stale_nodes),
            ("chain_depth", self.chain_depth),
            ("released_backlog", self.released_backlog),
            ("publishes_since_flag", self.publishes_since_flag),
        ]
        .map(|(name, value)| (name, value as f64, value as f64 / nodes))
        .to_vec()
    }
}

/// 把一串步算成视图。`history` 是历史那一段放完之后池建好以来带出过的节点（定下的、拉进来的），`history_publishes` 是历史的发布数。
/// 挂载序号在重挂载处加一：`C_mount` 清零（再装上重挂载自己的发布）、`ordinal_user` 从 1 重数、「距上次旗上」重数；`C_ever` 不清零。
pub(super) fn compute_views(item: &str, point: &str, script: &str, history: (&NodeSet, &NodeSet), history_publishes: u64, steps: &[StepInput]) -> Vec<OperationView> {
    // 先按挂载切开，每次挂载里全部操作的 final_nodes 一起定旗线
    let mut epoch_of_the_operation = Vec::new();
    let mut epoch = 0_u64;
    for step in steps {
        match step {
            StepInput::Remount(_) => epoch += 1,
            StepInput::Operation(_) => epoch_of_the_operation.push(epoch),
        }
    }
    let operations: Vec<&OperationInput> = steps.iter().filter_map(|step| if let StepInput::Operation(operation) = step { Some(operation) } else { None }).collect();
    let mut flags: Vec<[bool; 3]> = vec![[false; 3]; operations.len()];
    let mut inflated: Vec<bool> = vec![false; operations.len()];
    for each_epoch in 0..=epoch {
        let members: Vec<usize> = (0..operations.len()).filter(|position| epoch_of_the_operation[*position] == each_epoch).collect();
        let nodes: Vec<u64> = members.iter().map(|position| operations[*position].final_nodes).collect();
        let by_line = flags_by_line(&nodes);
        let main_flags: Vec<bool> = by_line.iter().map(|row| row[MAIN_FLOOR_LINE_POSITION]).collect();
        for (member, row) in members.iter().zip(&by_line) {
            flags[*member] = *row;
            inflated[*member] = median_is_inflated(&main_flags);
        }
    }
    let mut mount_final = history.0.clone();
    let mut mount_pulled = history.1.clone();
    let mut ever_final = history.0.clone();
    let mut ever_pulled = history.1.clone();
    let mut since_flag = history_publishes;
    let mut ordinal = 0;
    let mut position = 0;
    let mut views = Vec::new();
    for (step_index, step) in steps.iter().enumerate() {
        match step {
            StepInput::Remount(publishes) => {
                mount_final.clear();
                mount_pulled.clear();
                ordinal = 0;
                since_flag = 0;
                for publish in publishes {
                    mount_final.extend(publish.final_set.iter().copied());
                    mount_pulled.extend(publish.pulled.iter().copied());
                    ever_final.extend(publish.final_set.iter().copied());
                    ever_pulled.extend(publish.pulled.iter().copied());
                    since_flag += 1;
                }
            }
            StepInput::Operation(operation) => {
                ordinal += 1;
                let mut fresh = [0_u64; 4];
                let first = operation.publishes.first();
                let stale_nodes = first.map_or(0, |publish| publish.previous_nodes.difference(&mount_final).count() as u64);
                let chain_depth = first.map_or(0, |publish| publish.previous_nodes.iter().map(|node| release_chain_depth(&publish.graph, publish.root_level, *node)).max().unwrap_or(0));
                let released_backlog = first.map_or(0, |publish| publish.released_records_pre);
                let previous_nodes = first.map_or(0, |publish| publish.previous_nodes.len() as u64);
                let publishes_since_flag = since_flag;
                let mut seed = NodeSet::new();
                let mut pulled = NodeSet::new();
                for publish in &operation.publishes {
                    let wide: NodeSet = publish.seed.union(&publish.pulled).copied().collect();
                    fresh[0] += wide.difference(&mount_final).count() as u64;
                    fresh[1] += publish.pulled.difference(&mount_pulled).count() as u64;
                    fresh[2] += wide.difference(&ever_final).count() as u64;
                    fresh[3] += publish.pulled.difference(&ever_pulled).count() as u64;
                    mount_final.extend(publish.final_set.iter().copied());
                    mount_pulled.extend(publish.pulled.iter().copied());
                    ever_final.extend(publish.final_set.iter().copied());
                    ever_pulled.extend(publish.pulled.iter().copied());
                    since_flag += 1;
                }
                if let Some(publish) = first {
                    seed = publish.seed.clone();
                    pulled = publish.pulled.clone();
                }
                let flag_by_line = flags[position];
                let flagged = flag_by_line[MAIN_FLOOR_LINE_POSITION];
                if flagged {
                    since_flag = 0;
                }
                views.push(OperationView {
                    item: item.to_string(),
                    point: point.to_string(),
                    script: script.to_string(),
                    epoch: epoch_of_the_operation[position],
                    ordinal_user: ordinal,
                    step_index: u64::try_from(step_index).expect("下标") + 1,
                    name: operation.name.clone(),
                    selector: operation.selector.clone(),
                    user_units: operation.user_units,
                    old_units: operation.old_units,
                    new_units: operation.new_units,
                    final_nodes: operation.final_nodes,
                    flag_by_line,
                    flagged,
                    median_inflated: inflated[position],
                    fresh,
                    stale_nodes,
                    chain_depth,
                    released_backlog,
                    publishes_since_flag,
                    previous_nodes,
                    seed,
                    pulled,
                });
                position += 1;
            }
        }
    }
    views
}

fn publish_input_of(record: &PublishRecord) -> PublishInput {
    PublishInput {
        seed: record.seed.clone(),
        pulled: record.pulled.clone(),
        final_set: record.final_set.clone(),
        previous_nodes: record.graph.keys().copied().collect(),
        graph: record.graph.clone(),
        root_level: record.root_level,
        released_records_pre: record.released_records_pre,
    }
}

/// 选择子的名字：声明行的步名去掉 `overwrite_` 头；新建与截断照 `Step::selector_text`。
fn selector_of_the_step(step_name: &str) -> String {
    match step_name {
        "create_one" => "new_file".to_string(),
        "truncate_to_the_same_size" => "truncate".to_string(),
        other => other.strip_prefix("overwrite_").unwrap_or(other).to_string(),
    }
}

/// 一个项按脚本次序排成步（`script_step` 行定次序；缺行的项这里交空，由脚本核对那一条报）。
fn steps_of_the_item(item: &MeasuredItem) -> Vec<StepInput> {
    let mut steps = Vec::new();
    for (step_index, step_name, _epoch) in &item.step_lines {
        if step_name == "remount" {
            let remount = item.remounts.iter().find(|remount| remount.step_index == *step_index);
            steps.push(StepInput::Remount(remount.map_or_else(Vec::new, |remount| remount.publishes.iter().map(publish_input_of).collect())));
        } else if let Some(operation) = item.operations.iter().find(|operation| operation.step_index == *step_index) {
            steps.push(StepInput::Operation(OperationInput {
                name: operation.name.clone(),
                selector: selector_of_the_step(&operation.name),
                user_units: operation.user_units,
                old_units: operation.old_units,
                new_units: operation.new_units,
                final_nodes: operation.settles.iter().map(|settle| settle.final_nodes).max().unwrap_or(0),
                publishes: operation.publishes.iter().map(publish_input_of).collect(),
            }));
        }
    }
    steps
}

pub(super) fn views_of_the_item(item: &MeasuredItem) -> Vec<OperationView> {
    compute_views(
        &item.name,
        &item.point,
        &item.script,
        (&item.history_carried_final, &item.history_carried_pulled),
        item.history_settle_calls,
        &steps_of_the_item(item),
    )
}

/// 第 0 次挂载里的用户操作（按步次序）。
pub(super) fn first_mount(views: &[OperationView]) -> Vec<&OperationView> {
    views.iter().filter(|view| view.epoch == 0).collect()
}

/// 第 `epoch` 次挂载里序号为 `ordinal` 的操作的旗；没有这个序号交 `None`。
pub(super) fn flag_at(views: &[&OperationView], ordinal: u64) -> Option<bool> {
    views.iter().find(|view| view.ordinal_user == ordinal).map(|view| view.flagged)
}
