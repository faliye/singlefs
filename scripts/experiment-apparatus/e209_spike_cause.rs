//! E209（尖峰成因）的推的那一路：纯算术，不 `use` 任何 `crates/` 里的东西（跑前登记 `research/prompts/e209-preregistration.md` 第五节「装置写在哪」第 3 条）。
//!
//! 做的事：读量的那一路打出的 `E209RESULT` 产物（每个项一份：一个点上放同一段历史、再做一个脚本），算旗上操作、节点的新鲜度、
//! 与实现不共用代码的释放链闭包、问题单五行各自的判器、旗线与几何的敏感性、行表差与它落在 `crates/` 的哪个函数，逐量各占一行报；
//! 合成臂（答案已知的手造日志）与钉绝对值的断言写在单测里，不进产物。
//!
//! 用法：
//!   `e209-spike-cause report [--crates-root <仓根>] [--out <新文件>] <产物文件>…`   每个量各占一行，末尾打 `name=verdict` 行；给了 `--crates-root` 才查行表各行在 `crates/` 里的函数在第几行
//!   `e209-spike-cause list-items`                                   登记的项名与所属段（入口自证核它与历史用例、入口的名单同一份）
//! 退出码：0 读完；2 用法错或产物读不了。读不到的量报 `unread`，不当 0。
//! 单测钉合成臂与第七节 ③ 的绝对值：`cargo test --release --bin e209-spike-cause`。
//! 弄坏开关：无（变异表 `scripts/mutation/tables/e209_spike_cause.tsv` 逐条改源码，单测要红）。

#![allow(dead_code, reason = "模型的库函数一部分只有单测与报告的某一支用")]

#[path = "e209_spike_cause_scripts.rs"]
mod scripts;

use std::collections::{BTreeMap, BTreeSet};

/// 历史用例不是这个 bin 的目标（它拷进副本、当验证包的测试编，见入口 `e209-spike-cause-run.sh`）；当文本读进来，让门禁 crate-sources-reachable 认它有着落。
const _HISTORIES_PAYLOAD: &str = include_str!("e209_spike_cause_histories.rs");

/// 分配记录树一片叶罩多少槽（`crates/singlefs-format/src/lib.rs` 的 `ALLOCATION_RECORD_TREE_LEAF_SLOTS`）。
const LEAF_SLOTS: u64 = 812;
/// 内部节点扇出（同文件的 `ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`）。
const INTERNAL_FANOUT: u64 = 169;
/// 固定点上界（`crates/singlefs-core/src/transaction.rs` 的 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`）。
const SETTLING_ROUNDS_LIMIT: u64 = 64;
/// 旗线下限与倍数（E207 装置 `e207_strict_scan_steps.rs` 的 `SPIKE_FLOOR`、`SPIKE_MULTIPLE`；读法 1）。
const SPIKE_FLOOR: u64 = 28;
const SPIKE_MULTIPLE: u64 = 4;
/// 旗线灵敏度：下限 L 取这三个，主读第二个（读法 1）。
const FLOOR_LINES: [u64; 3] = [14, 28, 56];
const MAIN_FLOOR_LINE_POSITION: usize = 1;
const LOW_FLOOR_LINE_POSITION: usize = 0;
/// 行一的检验力：旗上操作与不旗操作各至少这么多个（失败条款：检验力不足）。
const FLAGGED_OPERATIONS_AT_LEAST: usize = 3;
/// 行四：挂载带出占比线（第六节 Q8、Q9）与单一一步占比线。
const MOUNT_SHARE_CARRIED_AT_LEAST: f64 = 0.9;
const MOUNT_SHARE_NOT_CARRIED_BELOW: f64 = 0.5;
const ONE_STEP_SHARE_AT_LEAST: f64 = 0.9;
/// 行五：前三大行占比线、行数、噪声倍数。
const TOP_ROWS_SHARE_AT_LEAST: f64 = 0.9;
const TOP_ROW_COUNT: usize = 3;
const NOISE_MULTIPLE: u64 = 10;
/// 几何敏感性里每个训练脚本（第六节「训练集」）。
const TRAINING_SCRIPTS: [&str; 4] = ["walk_forward", "walk_backward", "first_repeated", "create_then_first"];
const MANIPULATION_SCRIPTS: [&str; 3] = ["walk_forward", "walk_backward", "walk_from_middle"];

// ---- 节点与几何（第一节读法 3；独立于实现的位置算术）----

/// 分配记录树里的一个节点：位置三元组 `(层级, 盘号, 盘内序号)` 或根。派生的全序把根排在最末（同实现）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Node {
    Position { level: u32, device: u32, index_in_device: u64 },
    Root,
}

type NodeSet = BTreeSet<Node>;
/// 上一版每个节点的两份旧落点：节点 → [(盘号, 槽号)]。
type PointerGraph = BTreeMap<Node, Vec<(u32, u64)>>;

/// 合成日志里用的抽象节点：编号写成第 0 层第 0 盘的盘内序号。
fn synthetic_node(number: u64) -> Node {
    Node::Position { level: 0, device: 0, index_in_device: number }
}

fn synthetic_set(numbers: &[u64]) -> NodeSet {
    numbers.iter().map(|number| synthetic_node(*number)).collect()
}

fn parse_node(text: &str) -> Option<Node> {
    if text == "root" {
        return Some(Node::Root);
    }
    let mut parts = text.split('/');
    let node = Node::Position {
        level: parts.next()?.parse().ok()?,
        device: parts.next()?.parse().ok()?,
        index_in_device: parts.next()?.parse().ok()?,
    };
    parts.next().is_none().then_some(node)
}

/// `-` 是空；坏一个节点整串读不出（交 `None`，不当空）。
fn parse_node_set(text: &str) -> Option<NodeSet> {
    if text == "-" {
        return Some(NodeSet::new());
    }
    text.split(',').map(parse_node).collect()
}

/// 一个槽所在的那片叶（`allocation_record_tree.rs` 的 `leaf_of_slot`：槽号 ÷ 812）。
fn leaf_of_slot(device: u32, slot: u64) -> Node {
    Node::Position { level: 0, device, index_in_device: slot / LEAF_SLOTS }
}

/// 一个节点的父节点（`parent_of`：下标 ÷ 169；父的层级等于根的层级时父是根）；根没有父。
fn parent_of(node: Node, root_level: u32) -> Option<Node> {
    match node {
        Node::Root => None,
        Node::Position { level, device, index_in_device } => {
            let parent_level = level + 1;
            if parent_level == root_level {
                Some(Node::Root)
            } else {
                Some(Node::Position { level: parent_level, device, index_in_device: index_in_device / INTERNAL_FANOUT })
            }
        }
    }
}

/// 释放链闭包（独立模型）：从种子起，每个节点按上一版指针查它的两份旧落点，旧落点所在的叶与叶的全部祖先并进来，新并进的接着走。
/// 上一版没有的节点没有旧落点，走到它为止。
fn release_chain_closure(graph: &PointerGraph, root_level: u32, seeds: &NodeSet) -> NodeSet {
    let mut reached = seeds.clone();
    let mut waiting: Vec<Node> = seeds.iter().copied().collect();
    while let Some(node) = waiting.pop() {
        let Some(locations) = graph.get(&node) else {
            continue;
        };
        for (device, slot) in locations {
            let mut climbing = Some(leaf_of_slot(*device, *slot));
            while let Some(ancestor) = climbing {
                if reached.insert(ancestor) {
                    waiting.push(ancestor);
                }
                climbing = parent_of(ancestor, root_level);
            }
        }
    }
    reached
}

/// 单个节点作种子时释放链走的层数（一层 = 待走的这一批全部走完一遍；至少 1）。
fn release_chain_depth(graph: &PointerGraph, root_level: u32, start: Node) -> u64 {
    let mut reached: NodeSet = BTreeSet::from([start]);
    let mut frontier = vec![start];
    let mut depth = 0;
    while !frontier.is_empty() {
        depth += 1;
        let mut next = Vec::new();
        for node in frontier {
            let Some(locations) = graph.get(&node) else {
                continue;
            };
            for (device, slot) in locations {
                let mut climbing = Some(leaf_of_slot(*device, *slot));
                while let Some(ancestor) = climbing {
                    if reached.insert(ancestor) {
                        next.push(ancestor);
                    }
                    climbing = parent_of(ancestor, root_level);
                }
            }
        }
        frontier = next;
    }
    depth
}

// ---- 旗（读法 1）----

/// 下中位数：排好序取第 ⌊(n−1)/2⌋ 个；空取 0。
fn lower_median(values: &[u64]) -> u64 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted.get(sorted.len().saturating_sub(1) / 2).copied().unwrap_or(0)
}

/// 旗线：`max(下限, 4 × 下中位数)`；一个项一次挂载里全部操作的 `final_nodes` 一起算。
fn spike_line(floor: u64, final_nodes: &[u64]) -> u64 {
    floor.max(SPIKE_MULTIPLE * lower_median(final_nodes))
}

/// 一次挂载里每个操作在三条旗线下是不是旗上，次序同 `FLOOR_LINES`。
fn flags_by_line(final_nodes: &[u64]) -> Vec<[bool; 3]> {
    let lines: Vec<u64> = FLOOR_LINES.iter().map(|floor| spike_line(*floor, final_nodes)).collect();
    final_nodes.iter().map(|nodes| [*nodes >= lines[0], *nodes >= lines[1], *nodes >= lines[2]]).collect()
}

/// Q0 复现闸：一个点的 `walk_forward` 里旗上操作的个数（三条旗线）与判定。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ReproductionGate {
    flagged_by_line: [usize; 3],
    verdict: &'static str,
}

fn reproduction_gate(final_nodes: &[u64]) -> ReproductionGate {
    let flags = flags_by_line(final_nodes);
    let count = |position: usize| flags.iter().filter(|row| row[position]).count();
    let flagged_by_line = [count(0), count(1), count(2)];
    let verdict = if flagged_by_line[MAIN_FLOOR_LINE_POSITION] > 0 {
        "reproduced"
    } else if flagged_by_line[LOW_FLOOR_LINE_POSITION] > 0 {
        "only_at_the_low_line"
    } else {
        "not_reproduced"
    };
    ReproductionGate { flagged_by_line, verdict }
}

/// 一个项一次挂载里旗上操作超过半数：中位数被拉高，旗线失去意义。
fn median_is_inflated(flags: &[bool]) -> bool {
    flags.iter().filter(|flagged| **flagged).count() * 2 > flags.len()
}
/// inode 叶容器每片记录数（E203 模型同一个常量）。
const INODE_LEAF_RECORDS: u64 = 233;
/// 每段槽数（E203 模型同一个常量）。
const SEGMENT_SLOTS: u64 = 64;

fn lg(value: u64) -> u64 {
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


// ---- B8（逐项按差量）：E203 登记第五节「B8 每一行的做法与步数式子」与修订 7 的 s36，本地重写 ----
/// B8 式子吃的输入：一次发布在一段里的差量与调用次数（量的那一路读到的）。
#[derive(Clone, Debug, Default)]
struct ByDeltaInput {
    inode_leaf_containers: u64,
    other_files: u64,
    touched_files: u64,
    lower_nodes_of_touched_files: u64,
    extent_upper_levels: u64,
    extent_upper_nodes: u64,
    accounting_rows_of_the_generation: u64,
    accounting_tree_levels: u64,
    settling_rounds: u64,
    changed_leaves_per_round: Vec<u64>,
    allocation_tree_levels: u64,
    allocation_records: u64,
    segments_per_disk: u64,
    mapped_units: u64,
    deadlist_entries: u64,
    calls: BTreeMap<String, u64>,
    intersecting_records: u64,
    dirty_per_call: Vec<u64>,
    code_two_trees_over_all_rounds: Vec<(u64, u64)>,
    container_rewrites_over_all_rounds: Vec<u64>,
    moved_entries: u64,
    central_mapping_levels: u64,
    deadlist_query_levels: u64,
    deadlist_changed_entries: u64,
    release_query_levels: u64,
    changed_slots_and_records_per_round: Vec<u64>,
    mapped_units_changed_over_all_rounds: u64,
}

impl ByDeltaInput {
    fn calls_in_the_row(&self, row: &str) -> u64 {
        self.calls.get(row).copied().unwrap_or(0)
    }
}


/// B8 每一行一次发布准入之前的步数，次序同 E203 登记第五节。
fn by_delta_rows(input: &ByDeltaInput) -> Vec<(&'static str, u64)> {
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

fn by_delta_total(input: &ByDeltaInput) -> u64 {
    by_delta_rows(input).iter().map(|(_, steps)| steps).sum()
}

type Fields = BTreeMap<String, String>;

fn fields_of(line: &str) -> Fields {
    line.split(' ')
        .skip(1)
        .filter_map(|word| {
            word.split_once('=')
                .map(|(key, value)| (key.to_string(), value.to_string()))
        })
        .collect()
}

fn number(fields: &Fields, key: &str) -> Option<u64> {
    fields.get(key).and_then(|value| value.parse().ok())
}

fn numbers(text: &str) -> Vec<u64> {
    text.split(',')
        .filter_map(|value| value.parse().ok())
        .collect()
}

/// 一次判（E203 的 `row`、`delta`、`state`、`publish` 行）。
#[derive(Clone, Debug, Default)]
struct JudgementReadings {
    /// (段, 行) → (调用数, 查询层数之和)。
    rows: BTreeMap<(String, String), (u64, u64)>,
    /// (段, 行) → 步数（产物行的 `steps`；s37 与 s29 的步数报数用，不进 B8 式子）。
    row_steps: BTreeMap<(String, String), u64>,
    /// (段, 固定点第几轮, 名字, 值)，次序照产物。
    deltas: Vec<(String, u64, String, u64)>,
    state: Fields,
    fixed_point_rounds: Option<u64>,
}

/// 一轮（第 2、3、5、6 组）。

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
const ROW_WITHOUT_A_BY_DELTA_FORMULA: &str = "pooled_fallback_by_the_leaf";
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
        .map(|row| ((*row).to_string(), row_cell(*row).0))
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

// ---- 产物的读法 ----

/// 一次发布（固定点）的节点读数：`name=nodes` 行与同一次的 `name=pointer_graph` 行。
#[derive(Clone, Debug, Default)]
struct PublishRecord {
    /// `user`、`write_row`、`warm_up`。
    kind: String,
    order: u64,
    settle_call: u64,
    site: String,
    seed: NodeSet,
    pulled: NodeSet,
    final_set: NodeSet,
    released_records_pre: u64,
    graph: PointerGraph,
    root_level: u32,
    /// 走链读数自查（边的重走与实现并进的结果相同）。
    walk_matches: bool,
    edges: Vec<(Node, Node)>,
}

/// 一次固定点的读数（`name=settle` 行）：旗吃 `final_nodes`。
#[derive(Clone, Debug, Default)]
struct SettleReading {
    settle_call: u64,
    judgement: u64,
    rounds: u64,
    outcome: String,
    final_nodes: u64,
    /// 每一轮：(出口, 这一轮各行步数之和)。
    rounds_detail: Vec<(String, u64)>,
}

/// 一次被量的用户操作（声明行起头）。
#[derive(Clone, Debug, Default)]
struct Operation {
    epoch: u64,
    step_index: u64,
    name: String,
    user_units: u64,
    old_units: u64,
    new_units: u64,
    settles: Vec<SettleReading>,
    publishes: Vec<PublishRecord>,
}

/// 重挂载自己的发布（写行一次、暖机若干次）。
#[derive(Clone, Debug, Default)]
struct RemountRecord {
    epoch: u64,
    step_index: u64,
    settles: Vec<SettleReading>,
    publishes: Vec<PublishRecord>,
}

/// 一份产物：一个项。
#[derive(Clone, Debug, Default)]
struct MeasuredItem {
    name: String,
    point: String,
    arm: String,
    script: String,
    build: String,
    files: u64,
    units_per_file: u64,
    unit_area_slots: u64,
    constants: Fields,
    history_user_changes: Option<u64>,
    history_settle_calls: u64,
    history_carried_final: NodeSet,
    history_carried_pulled: NodeSet,
    /// `script_step` 行：(第几步, 步名, 挂载序号)。
    step_lines: Vec<(u64, String, u64)>,
    operations: Vec<Operation>,
    remounts: Vec<RemountRecord>,
    images: Vec<(String, String)>,
    judgements: BTreeMap<String, JudgementReadings>,
    first_state: Fields,
    is_twin: bool,
    done: bool,
    refused: bool,
    /// 去掉产物头之后的全部读数行（确定性孪生项逐字节对拍用）。
    body_lines: Vec<String>,
}

/// 读数行当前挂在哪个对象上：`settle` 之后的 `round`、`round_row` 行要接到它的最后一次固定点。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Operation(usize),
    Remount(usize),
}

fn text_of(fields: &Fields, key: &str) -> String {
    fields.get(key).cloned().unwrap_or_default()
}

fn value_of(fields: &Fields, key: &str) -> u64 {
    number(fields, key).unwrap_or(0)
}

fn publish_from_fields(fields: &Fields) -> PublishRecord {
    let set = |key: &str| parse_node_set(&text_of(fields, key)).unwrap_or_default();
    let edges = text_of(fields, "edges")
        .split(',')
        .filter_map(|edge| edge.split_once('>'))
        .filter_map(|(from, to)| Some((parse_node(from)?, parse_node(to)?)))
        .collect();
    PublishRecord {
        kind: text_of(fields, "publish_kind"),
        order: value_of(fields, "publish_order"),
        settle_call: value_of(fields, "settle_call"),
        site: text_of(fields, "first_merge_site"),
        seed: set("seed"),
        pulled: set("pulled"),
        final_set: set("final"),
        released_records_pre: value_of(fields, "released_records_pre"),
        walk_matches: text_of(fields, "edges_walk_matches_the_implementation") == "1",
        edges,
        ..PublishRecord::default()
    }
}

/// `graph=节点@盘.槽+盘.槽;…` 的读法：节点与两份旧落点。读不出的交 `None`。
fn parse_pointer_graph(text: &str) -> Option<PointerGraph> {
    if text == "-" {
        return Some(PointerGraph::new());
    }
    let mut graph = PointerGraph::new();
    for entry in text.split(',') {
        let (node, places) = entry.split_once('@')?;
        let locations: Option<Vec<(u32, u64)>> = places
            .split('+')
            .map(|place| {
                let (device, slot) = place.split_once('.')?;
                Some((device.parse().ok()?, slot.parse().ok()?))
            })
            .collect();
        graph.insert(parse_node(node)?, locations?);
    }
    Some(graph)
}

fn settle_from_fields(fields: &Fields) -> SettleReading {
    SettleReading {
        settle_call: value_of(fields, "settle_call"),
        judgement: value_of(fields, "judgement"),
        rounds: value_of(fields, "rounds"),
        outcome: text_of(fields, "outcome"),
        final_nodes: value_of(fields, "final_nodes"),
        rounds_detail: Vec::new(),
    }
}

/// 读数行里标「第几次挂载、哪一步、第几步」的三个键拼成的判键前缀（E203 的 `state`、`row`、`delta`、`publish` 行按它归一次判）。
fn judgement_prefix(fields: &Fields) -> String {
    format!("{}/{}/{}/", value_of(fields, "mount_epoch"), text_of(fields, "operation"), value_of(fields, "operation_index"))
}

struct Reader {
    item: MeasuredItem,
    current: Option<Target>,
    last_publish: Option<Target>,
}

impl Reader {
    fn operation_index(&self, fields: &Fields) -> Option<usize> {
        let step_index = value_of(fields, "operation_index");
        self.item.operations.iter().position(|operation| operation.step_index == step_index)
    }

    fn remount_index(&self, fields: &Fields) -> Option<usize> {
        let epoch = value_of(fields, "mount_epoch");
        self.item.remounts.iter().position(|remount| remount.epoch == epoch)
    }

    fn target_of(&mut self, fields: &Fields) -> Option<Target> {
        if text_of(fields, "phase") == "remount" {
            let position = self.remount_index(fields).unwrap_or_else(|| {
                self.item.remounts.push(RemountRecord {
                    epoch: value_of(fields, "mount_epoch"),
                    step_index: value_of(fields, "operation_index"),
                    ..RemountRecord::default()
                });
                self.item.remounts.len() - 1
            });
            Some(Target::Remount(position))
        } else {
            self.operation_index(fields).map(Target::Operation)
        }
    }

    fn settles_of(&mut self, target: Target) -> &mut Vec<SettleReading> {
        match target {
            Target::Operation(position) => &mut self.item.operations[position].settles,
            Target::Remount(position) => &mut self.item.remounts[position].settles,
        }
    }

    fn publishes_of(&mut self, target: Target) -> &mut Vec<PublishRecord> {
        match target {
            Target::Operation(position) => &mut self.item.operations[position].publishes,
            Target::Remount(position) => &mut self.item.remounts[position].publishes,
        }
    }

    fn take_line(&mut self, fields: &Fields) {
        let name = fields.get("name").map_or("", String::as_str);
        match name {
            "header" => self.item.is_twin = text_of(fields, "twin") == "yes",
            "point" => {
                let item = &mut self.item;
                item.name = text_of(fields, "item");
                item.point = text_of(fields, "point");
                item.arm = text_of(fields, "arm");
                item.script = text_of(fields, "script");
                item.build = text_of(fields, "build");
                item.files = value_of(fields, "files");
                item.units_per_file = value_of(fields, "units_per_small_file");
                item.unit_area_slots = value_of(fields, "unit_area_slots");
            }
            "constants" => self.item.constants = fields.clone(),
            "history" => {
                self.item.history_user_changes = number(fields, "user_changes");
                self.item.history_settle_calls = value_of(fields, "settle_calls");
            }
            "history_carried" => {
                self.item.history_carried_final = parse_node_set(&text_of(fields, "ever_final")).unwrap_or_default();
                self.item.history_carried_pulled = parse_node_set(&text_of(fields, "ever_pulled")).unwrap_or_default();
            }
            "image_sha256" => self.item.images.push((text_of(fields, "when"), text_of(fields, "sha256"))),
            "script_step" => self.item.step_lines.push((value_of(fields, "step_index"), text_of(fields, "operation"), value_of(fields, "mount_epoch"))),
            "declared" => {
                self.item.operations.push(Operation {
                    epoch: value_of(fields, "mount_epoch"),
                    step_index: value_of(fields, "operation_index"),
                    name: text_of(fields, "operation"),
                    user_units: value_of(fields, "user_units"),
                    old_units: value_of(fields, "old_units"),
                    new_units: value_of(fields, "new_units"),
                    ..Operation::default()
                });
            }
            "settle" => {
                if let Some(target) = self.target_of(fields) {
                    self.settles_of(target).push(settle_from_fields(fields));
                    self.current = Some(target);
                }
            }
            "round" => {
                if let Some(target) = self.current {
                    if let Some(settle) = self.settles_of(target).last_mut() {
                        settle.rounds_detail.push((text_of(fields, "round_exit"), 0));
                    }
                }
            }
            "round_row" => {
                if let Some(target) = self.current {
                    if let Some(round) = self.settles_of(target).last_mut().and_then(|settle| settle.rounds_detail.last_mut()) {
                        round.1 += value_of(fields, "steps");
                    }
                }
            }
            "nodes" => {
                if let Some(target) = self.target_of(fields) {
                    self.publishes_of(target).push(publish_from_fields(fields));
                    self.last_publish = Some(target);
                }
            }
            "pointer_graph" => {
                if let Some(target) = self.last_publish {
                    let graph = parse_pointer_graph(&text_of(fields, "graph"));
                    let root_level = u32::try_from(value_of(fields, "root_level")).unwrap_or(0);
                    if let Some(publish) = self.publishes_of(target).last_mut() {
                        publish.graph = graph.unwrap_or_default();
                        publish.root_level = root_level;
                    }
                }
            }
            "state" => {
                let prefix = judgement_prefix(fields);
                if self.item.first_state.is_empty() {
                    self.item.first_state = fields.clone();
                }
                self.item.judgements.entry(prefix).or_default().state = fields.clone();
            }
            "row" => {
                let cell = self.item.judgements.entry(text_of(fields, "publish")).or_default();
                let key = (text_of(fields, "segment"), text_of(fields, "row"));
                cell.rows.insert(key.clone(), (value_of(fields, "calls"), value_of(fields, "query_levels")));
                cell.row_steps.insert(key, value_of(fields, "steps"));
            }
            "delta" => {
                self.item.judgements.entry(text_of(fields, "publish")).or_default().deltas.push((
                    text_of(fields, "segment"),
                    value_of(fields, "fixed_point_round"),
                    text_of(fields, "key"),
                    value_of(fields, "value"),
                ));
            }
            "publish" => {
                self.item.judgements.entry(text_of(fields, "publish")).or_default().fixed_point_rounds = number(fields, "fixed_point_rounds");
            }
            "history_refused" | "window_refused" => self.item.refused = true,
            "item_done" => self.item.done = true,
            _ => {}
        }
    }
}

/// 读一份产物的正文：每个项一份，`E209RESULT` 行以外的行不读。
fn read_item_text(text: &str) -> Result<MeasuredItem, String> {
    let mut reader = Reader { item: MeasuredItem::default(), current: None, last_publish: None };
    for line in text.lines().filter(|line| line.starts_with("E209RESULT ")) {
        let fields = fields_of(line);
        reader.take_line(&fields);
        if fields.get("name").map(String::as_str) != Some("header") && fields.get("name").map(String::as_str) != Some("source_sha256") && !line.starts_with("E209RESULT name=anchor") {
            reader.item.body_lines.push(line.to_string());
        }
    }
    if reader.item.name.is_empty() {
        return Err("产物里没有 name=point 行".to_string());
    }
    Ok(reader.item)
}

fn read_item(path: &str) -> Result<MeasuredItem, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("读不了 {path}：{error}"))?;
    read_item_text(&text).map_err(|error| format!("{path}：{error}"))
}

/// 一次判的固定点的 B8 合计（三段之和）；读不到交 `None`。
fn judgement_of(item: &MeasuredItem, operation: &Operation, settle: &SettleReading) -> Option<JudgementReadings> {
    let key = format!("{}/{}/{}/{}", operation.epoch, operation.name, operation.step_index, settle.judgement);
    let prefix = judgement_prefix_of(operation);
    let mut judgement = item.judgements.get(&key)?.clone();
    judgement.state = item.judgements.get(&prefix)?.state.clone();
    Some(judgement)
}

fn judgement_prefix_of(operation: &Operation) -> String {
    format!("{}/{}/{}/", operation.epoch, operation.name, operation.step_index)
}

/// 一个操作各次固定点的 B8 合计之和；有一次读不到整个操作交 `None`（不当 0）。
fn steps_of_the_operation(item: &MeasuredItem, operation: &Operation) -> Option<u64> {
    rows_of_the_operation(item, operation).map(|rows| rows.values().sum())
}

/// 一个操作各次固定点三段里 B8 每一行的步数之和（行名 → 步数）。
fn rows_of_the_operation(item: &MeasuredItem, operation: &Operation) -> Option<BTreeMap<&'static str, u64>> {
    let mut rows: BTreeMap<&'static str, u64> = BTreeMap::new();
    for settle in &operation.settles {
        let judgement = judgement_of(item, operation, settle)?;
        for segment in SEGMENTS_BEFORE_ADMISSION {
            let input = by_delta_input(&judgement, segment, item.unit_area_slots).ok()?;
            for (row, steps) in by_delta_rows(&input) {
                *rows.entry(row).or_insert(0) += steps;
            }
        }
    }
    Some(rows)
}

// ---- 每次操作的视图：旗、新鲜度、攒下的量（读法 4–6、第六节 Q6 的四个量）----

/// 新鲜度的四个变体：`C_mount` 或 `C_ever`（挂载清不清零）× 宽读（`seed ∪ pulled` 对定下集合的并）或窄读（`pulled` 对拉进集合的并）。
const FRESHNESS_VARIANTS: [&str; 4] = ["mount_wide", "mount_narrow", "ever_wide", "ever_narrow"];

/// 一次发布喂视图的输入（节点身份）。
#[derive(Clone, Debug, Default)]
struct PublishInput {
    seed: NodeSet,
    pulled: NodeSet,
    final_set: NodeSet,
    /// 上一版的节点（真实日志里是指针图的键）。
    previous_nodes: NodeSet,
    graph: PointerGraph,
    root_level: u32,
    released_records_pre: u64,
}

#[derive(Clone, Debug, Default)]
struct OperationInput {
    name: String,
    selector: String,
    user_units: u64,
    old_units: u64,
    new_units: u64,
    final_nodes: u64,
    publishes: Vec<PublishInput>,
}

#[derive(Clone, Debug)]
enum StepInput {
    Operation(OperationInput),
    /// 重挂载：它自己的发布（写行、暖机）。
    Remount(Vec<PublishInput>),
}

/// 一次用户操作的视图：判器只读它。
#[derive(Clone, Debug, Default)]
struct OperationView {
    item: String,
    point: String,
    script: String,
    epoch: u64,
    ordinal_user: u64,
    step_index: u64,
    name: String,
    selector: String,
    user_units: u64,
    old_units: u64,
    new_units: u64,
    final_nodes: u64,
    /// 三条旗线下是不是旗上（次序同 `FLOOR_LINES`）；`flagged` 是主读那一条。
    flag_by_line: [bool; 3],
    flagged: bool,
    median_inflated: bool,
    /// 四个变体的新鲜度（次序同 `FRESHNESS_VARIANTS`）。
    fresh: [u64; 4],
    /// 上一版里本挂载没带出过的节点数（宽读）。
    stale_nodes: u64,
    /// 上一版里单节点作种子时释放链的最大层数。
    chain_depth: u64,
    /// 进固定点时上一版记录里已释放的条数。
    released_backlog: u64,
    /// 本挂载里距上一次旗上操作过了几次发布（没有旗上过的从挂载起算；第 0 次挂载从池建好起算）。
    publishes_since_flag: u64,
    previous_nodes: u64,
    seed: NodeSet,
    pulled: NodeSet,
}

impl OperationView {
    /// 行三的八格量：(量名, 绝对值, 除以上一版节点数的相对值)。
    fn quantities(&self) -> Vec<(&'static str, f64, f64)> {
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
fn compute_views(item: &str, point: &str, script: &str, history: (&NodeSet, &NodeSet), history_publishes: u64, steps: &[StepInput]) -> Vec<OperationView> {
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

fn views_of_the_item(item: &MeasuredItem) -> Vec<OperationView> {
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
fn first_mount<'views>(views: &'views [OperationView]) -> Vec<&'views OperationView> {
    views.iter().filter(|view| view.epoch == 0).collect()
}

/// 第 `epoch` 次挂载里序号为 `ordinal` 的操作的旗；没有这个序号交 `None`。
fn flag_at(views: &[&OperationView], ordinal: u64) -> Option<bool> {
    views.iter().find(|view| view.ordinal_user == ordinal).map(|view| view.flagged)
}

// ---- 行一：旗跟「碰到本挂载没带出过的节点」走（Q2、Q3）----

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SetMismatches {
    flagged: usize,
    unflagged: usize,
    mismatches: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuleVerdict {
    Holds,
    DoesNotHold,
    TooFewFlaggedOperations,
    NotRun,
}

impl RuleVerdict {
    fn tag(self) -> &'static str {
        match self {
            Self::Holds => "holds",
            Self::DoesNotHold => "does_not_hold",
            Self::TooFewFlaggedOperations => "too_few_flagged_operations",
            Self::NotRun => "not_run",
        }
    }
}

#[derive(Clone, Debug, Default)]
struct RuleReading {
    variant: usize,
    /// 训练集里旗上操作新鲜度的最小值；训练集没有旗上操作交 `None`。
    threshold: Option<u64>,
    flagged_in_training: usize,
    /// 训练集里新鲜度不小于门槛却不旗的操作。
    defining_failures: Vec<String>,
    script: SetMismatches,
    geometry: SetMismatches,
}

impl RuleReading {
    /// 合成对照的绿：定规则没失败、两类检验集上没有不符（不看检验力）。
    fn agrees(&self) -> bool {
        self.threshold.is_some() && self.defining_failures.is_empty() && self.script.mismatches.is_empty() && self.geometry.mismatches.is_empty()
    }

    fn verdict(&self) -> RuleVerdict {
        if self.threshold.is_none() {
            RuleVerdict::NotRun
        } else if !self.agrees() {
            RuleVerdict::DoesNotHold
        } else if [&self.script, &self.geometry].iter().any(|set| set.flagged < FLAGGED_OPERATIONS_AT_LEAST || set.unflagged < FLAGGED_OPERATIONS_AT_LEAST) {
            RuleVerdict::TooFewFlaggedOperations
        } else {
            RuleVerdict::Holds
        }
    }
}

fn label_of(view: &OperationView, variant: usize) -> String {
    format!("{}#{}:fresh={}:observed_flag={}", view.item, view.ordinal_user, view.fresh[variant], u8::from(view.flagged))
}

/// 门槛下预测（`fresh ≥ 门槛` 旗上）对观测旗的不符。
fn mismatches_at(views: &[&OperationView], variant: usize, threshold: u64) -> SetMismatches {
    SetMismatches {
        flagged: views.iter().filter(|view| view.flagged).count(),
        unflagged: views.iter().filter(|view| !view.flagged).count(),
        mismatches: views.iter().filter(|view| (view.fresh[variant] >= threshold) != view.flagged).map(|view| label_of(view, variant)).collect(),
    }
}

fn threshold_from_training(training: &[&OperationView], variant: usize) -> Option<u64> {
    training.iter().filter(|view| view.flagged).map(|view| view.fresh[variant]).min()
}

/// 用训练集定规则（门槛取旗上操作新鲜度的最小值）、在两类检验集上检。
fn judge_freshness_rule(variant: usize, training: &[&OperationView], script: &[&OperationView], geometry: &[&OperationView]) -> RuleReading {
    let threshold = threshold_from_training(training, variant);
    let Some(threshold) = threshold else {
        return RuleReading { variant, ..RuleReading::default() };
    };
    RuleReading {
        variant,
        threshold: Some(threshold),
        flagged_in_training: training.iter().filter(|view| view.flagged).count(),
        defining_failures: training.iter().filter(|view| !view.flagged && view.fresh[variant] >= threshold).map(|view| label_of(view, variant)).collect(),
        script: mismatches_at(script, variant, threshold),
        geometry: mismatches_at(geometry, variant, threshold),
    }
}

/// 行一的两个翻面观测（Q3）：三个脚本旗上操作的选择子集合与个数，与「再碰还旗」。
#[derive(Clone, Debug, Default)]
struct Manipulation {
    flagged_by_script: Vec<(String, usize, BTreeSet<String>)>,
    position_follows: bool,
    count_follows: bool,
    retouch_operations: usize,
    retouch_flagged: Vec<String>,
}

fn judge_manipulation(first_mount_by_script: &BTreeMap<String, Vec<&OperationView>>, every_operation: &[&OperationView]) -> Manipulation {
    let flagged_by_script: Vec<(String, usize, BTreeSet<String>)> = MANIPULATION_SCRIPTS
        .iter()
        .filter_map(|script| first_mount_by_script.get(*script).map(|views| (script, views)))
        .map(|(script, views)| {
            let selectors: BTreeSet<String> = views.iter().filter(|view| view.flagged).map(|view| view.selector.clone()).collect();
            ((*script).to_string(), views.iter().filter(|view| view.flagged).count(), selectors)
        })
        .collect();
    let distinct_selector_sets: BTreeSet<&BTreeSet<String>> = flagged_by_script.iter().map(|(_, _, selectors)| selectors).collect();
    let distinct_counts: BTreeSet<usize> = flagged_by_script.iter().map(|(_, count, _)| *count).collect();
    let retouched: Vec<&&OperationView> = every_operation.iter().filter(|view| view.fresh[0] == 0).collect();
    Manipulation {
        position_follows: distinct_selector_sets.len() >= 2,
        count_follows: distinct_counts.len() >= 2,
        retouch_operations: retouched.len(),
        retouch_flagged: retouched.iter().filter(|view| view.flagged).map(|view| format!("{}#{}", view.item, view.ordinal_user)).collect(),
        flagged_by_script,
    }
}

// ---- 行二：旗只由挂载之后的第几次操作定（Q4、Q5）----

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct OrdinalEquality {
    pairs: usize,
    first_difference: Option<String>,
}

/// 同一个点内全部脚本两两比：第 0 次挂载里每个都有值的序号上旗是否相同；一边没有值的序号不比。
fn judge_ordinal_equality(first_mount_by_script: &BTreeMap<String, Vec<&OperationView>>) -> OrdinalEquality {
    let scripts: Vec<(&String, &Vec<&OperationView>)> = first_mount_by_script.iter().collect();
    let mut reading = OrdinalEquality::default();
    for (position, (first_script_name, first_script_views)) in scripts.iter().enumerate() {
        for (second_script_name, second_script_views) in &scripts[position + 1..] {
            reading.pairs += 1;
            let shorter = u64::try_from(first_script_views.len().min(second_script_views.len())).expect("个数");
            for ordinal in 1..=shorter {
                if flag_at(first_script_views, ordinal) != flag_at(second_script_views, ordinal) && reading.first_difference.is_none() {
                    reading.first_difference = Some(format!("{first_script_name}/{second_script_name}#{ordinal}"));
                }
            }
        }
    }
    reading
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShiftRow {
    script: String,
    filler_count: u64,
    first_ordinal: Option<u64>,
    first_flagged_ordinal: Option<u64>,
    first_flagged_selector: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OrdinalShift {
    walk_forward_first_flagged_ordinal: Option<u64>,
    rows: Vec<ShiftRow>,
    verdict: &'static str,
}

/// 第一个旗上操作的序号与选择子。
fn first_flagged(views: &[&OperationView]) -> (Option<u64>, String) {
    views.iter().find(|view| view.flagged).map_or((None, String::new()), |view| (Some(view.ordinal_user), view.selector.clone()))
}

/// 把 `first` 挪到序号 `n + 1`：每个填充脚本里 `first` 的序号、旗与第一个旗上操作；判「旗跟操作走」「旗跟序号走」「都不是」。
fn judge_ordinal_shift(walk_forward: &[&OperationView], fillers: &[(String, u64, Vec<&OperationView>)]) -> OrdinalShift {
    let (walk_forward_ordinal, _) = first_flagged(walk_forward);
    let rows: Vec<ShiftRow> = fillers
        .iter()
        .map(|(script, count, views)| {
            let (flagged_ordinal, selector) = first_flagged(views);
            ShiftRow {
                script: script.clone(),
                filler_count: *count,
                first_ordinal: views.iter().find(|view| view.selector == "first").map(|view| view.ordinal_user),
                first_flagged_ordinal: flagged_ordinal,
                first_flagged_selector: selector,
            }
        })
        .collect();
    let follows_the_operation = !rows.is_empty() && rows.iter().all(|row| row.first_ordinal == Some(row.filler_count + 1) && row.first_flagged_ordinal == row.first_ordinal);
    let follows_the_ordinal = !rows.is_empty() && walk_forward_ordinal.is_some() && rows.iter().all(|row| row.first_flagged_ordinal == walk_forward_ordinal);
    let verdict = if follows_the_operation {
        "follows_the_operation"
    } else if follows_the_ordinal {
        "follows_the_ordinal"
    } else {
        "neither"
    };
    OrdinalShift { walk_forward_first_flagged_ordinal: walk_forward_ordinal, rows, verdict }
}

// ---- 行三：旗由攒下的量跨过门槛决定（Q6、Q7）----

#[derive(Clone, Debug, PartialEq)]
struct ThresholdFit {
    threshold: f64,
    errors: usize,
    flagged_values: usize,
    unflagged_values: usize,
}

fn distinct_values(points: &[(f64, bool)], flagged: bool) -> usize {
    let mut values: Vec<f64> = points.iter().filter(|point| point.1 == flagged).map(|point| point.0).collect();
    values.sort_by(f64::total_cmp);
    values.dedup();
    values.len()
}

/// 使分错数最少的门槛（`值 ≥ 门槛` 旗上）；分错数相同取小的门槛。候选是出现过的每个值。
fn best_threshold(points: &[(f64, bool)]) -> ThresholdFit {
    let mut candidates: Vec<f64> = points.iter().map(|point| point.0).collect();
    candidates.sort_by(f64::total_cmp);
    candidates.dedup();
    let errors_at = |threshold: f64| points.iter().filter(|(value, flagged)| (*value >= threshold) != *flagged).count();
    let mut best = ThresholdFit { threshold: f64::INFINITY, errors: points.len() + 1, flagged_values: 0, unflagged_values: 0 };
    for candidate in candidates {
        let errors = errors_at(candidate);
        if errors < best.errors {
            best.threshold = candidate;
            best.errors = errors;
        }
    }
    best.flagged_values = distinct_values(points, true);
    best.unflagged_values = distinct_values(points, false);
    best
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QuantityVerdict {
    Holds,
    DoesNotHold,
    OnlyInTheTrainingGeometry,
    TooFewDistinctValues,
    TestGeometryNotRun,
    NotRun,
}

impl QuantityVerdict {
    fn tag(self) -> &'static str {
        match self {
            Self::Holds => "holds",
            Self::DoesNotHold => "does_not_hold",
            Self::OnlyInTheTrainingGeometry => "only_in_the_training_geometry",
            Self::TooFewDistinctValues => "too_few_distinct_values",
            Self::TestGeometryNotRun => "test_geometry_not_run",
            Self::NotRun => "not_run",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct QuantityReading {
    pooled: ThresholdFit,
    geometry_errors: usize,
    verdict: QuantityVerdict,
}

/// 一格（一个量、绝对或相对）：池化（训练集加检验集一）里找门槛，再用这个门槛（不重拟）在检验集二上数分错。
fn judge_stored_quantity(pooled: &[(f64, bool)], geometry: &[(f64, bool)]) -> QuantityReading {
    let fit = best_threshold(pooled);
    let geometry_errors = geometry.iter().filter(|(value, flagged)| (*value >= fit.threshold) != *flagged).count();
    let verdict = if pooled.is_empty() || fit.flagged_values == 0 {
        QuantityVerdict::NotRun
    } else if fit.errors > 0 {
        QuantityVerdict::DoesNotHold
    } else if fit.flagged_values < 2 || fit.unflagged_values < 2 {
        QuantityVerdict::TooFewDistinctValues
    } else if geometry.is_empty() {
        QuantityVerdict::TestGeometryNotRun
    } else if geometry_errors > 0 {
        QuantityVerdict::OnlyInTheTrainingGeometry
    } else {
        QuantityVerdict::Holds
    };
    QuantityReading { pooled: fit, geometry_errors, verdict }
}

/// Q7 的一个取样点：`first` 前面先做 m 次冷文件覆盖写之后，`first` 那一步的读数。
#[derive(Clone, Debug, PartialEq, Eq)]
struct SweepPoint {
    presweep_count: u64,
    flagged: bool,
    stale_nodes: u64,
    publishes_since_flag: u64,
    final_nodes: u64,
    first_flagged_ordinal: Option<u64>,
    first_flagged_selector: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SweepReading {
    flagged_counts: Vec<u64>,
    unflagged_counts: Vec<u64>,
    between: Option<(u64, u64)>,
    verdict: &'static str,
}

fn judge_presweep(points: &[SweepPoint]) -> SweepReading {
    let flagged_counts: Vec<u64> = points.iter().filter(|point| point.flagged).map(|point| point.presweep_count).collect();
    let unflagged_counts: Vec<u64> = points.iter().filter(|point| !point.flagged).map(|point| point.presweep_count).collect();
    let between = match (flagged_counts.iter().max(), unflagged_counts.iter().min()) {
        (Some(largest_flagged), Some(smallest_unflagged)) if largest_flagged < smallest_unflagged => Some((*largest_flagged, *smallest_unflagged)),
        _ => None,
    };
    let verdict = if flagged_counts.is_empty() || unflagged_counts.is_empty() {
        "threshold_sides_not_both_sampled"
    } else if between.is_some() {
        "threshold_between_two_sample_points"
    } else {
        "not_monotone_in_the_presweep_count"
    };
    SweepReading { flagged_counts, unflagged_counts, between, verdict }
}

// ---- 行四：重挂载之后不旗，是因为挂载那一步已经把那批节点带出（Q8、Q9）----

#[derive(Clone, Debug, PartialEq)]
struct MountReading {
    premise_flagged_after_the_mount: bool,
    nodes_of_the_spike: usize,
    carried: usize,
    share: f64,
    verdict: &'static str,
    /// 带出的节点各占挂载自己的哪一步（步名 → 首次出现在这一步的节点数）。
    shares_by_step: Vec<(String, usize)>,
    /// 最大一步占带出节点的比例 ≥ 线时它的步名；不到线交 `None`。
    located_step: Option<String>,
    one_step_verdict: &'static str,
}

/// `spike_nodes` 是不挂载的项里尖峰那次操作的 `seed ∪ pulled`；`mount_steps` 是挂载自己的发布（写行、暖机）按次序各步的定下集合；
/// `flagged_after_the_mount` 是挂载之后第一次覆盖写是否旗上（前提）。
fn judge_mount_carries(flagged_after_the_mount: bool, spike_nodes: &NodeSet, mount_steps: &[(String, NodeSet)]) -> MountReading {
    let carried: NodeSet = spike_nodes.iter().copied().filter(|node| mount_steps.iter().any(|(_, set)| set.contains(node))).collect();
    let share = if spike_nodes.is_empty() { 0.0 } else { carried.len() as f64 / spike_nodes.len() as f64 };
    let verdict = if flagged_after_the_mount {
        "mount_did_not_stop_the_spike"
    } else if share >= MOUNT_SHARE_CARRIED_AT_LEAST {
        "mount_carried_the_nodes"
    } else if share < MOUNT_SHARE_NOT_CARRIED_BELOW {
        "not_flagged_but_not_carried_by_the_mount"
    } else {
        "partly_carried"
    };
    let mut shares_by_step: Vec<(String, usize)> = mount_steps.iter().map(|(name, _)| (name.clone(), 0)).collect();
    for node in &carried {
        if let Some(position) = mount_steps.iter().position(|(_, set)| set.contains(node)) {
            shares_by_step[position].1 += 1;
        }
    }
    let largest = shares_by_step.iter().max_by_key(|(_, count)| *count).cloned();
    let located_step = largest.filter(|(_, count)| !carried.is_empty() && *count as f64 / carried.len() as f64 >= ONE_STEP_SHARE_AT_LEAST).map(|(name, _)| name);
    let one_step_verdict = if carried.is_empty() {
        "no_carried_nodes"
    } else if located_step.is_some() {
        "located_one_step"
    } else {
        "spread_over_several_steps"
    };
    MountReading {
        premise_flagged_after_the_mount: flagged_after_the_mount,
        nodes_of_the_spike: spike_nodes.len(),
        carried: carried.len(),
        share,
        verdict,
        shares_by_step,
        located_step,
        one_step_verdict,
    }
}

/// 重挂载自己的发布按次序命名：写行一次，暖机 1、2…
fn mount_step_name(publish: &PublishRecord) -> String {
    if publish.kind == "write_row" {
        "write_row".to_string()
    } else {
        format!("warm_up_{}", publish.order)
    }
}

/// 步名对应的计划函数（`crates/singlefs-core/src/mount.rs`），行号照 `--crates-root` 现取。
fn plan_function_of_the_mount_step(step_name: &str) -> &'static str {
    if step_name == "write_row" {
        "row_publish_plan_on_file_version"
    } else {
        "empty_publish_plan_after_file_version"
    }
}

// ---- 行五：多出来的步数落在哪几行（Q11、Q12、Q13）----

/// 旗上操作的可比操作：同点别的脚本项里同一选择子、同一声明单元数、不在旗上的操作，每个项取头一个，按项名字典序取前两个。
fn pick_comparables<'views>(target: &OperationView, candidates: &'views [OperationView]) -> Vec<&'views OperationView> {
    let mut sorted: Vec<&OperationView> = candidates
        .iter()
        .filter(|view| view.point == target.point && view.item != target.item && view.selector == target.selector && view.old_units == target.old_units && view.new_units == target.new_units && !view.flagged)
        .collect();
    sorted.sort_by(|left, right| (&left.item, left.step_index).cmp(&(&right.item, right.step_index)));
    sorted.dedup_by(|later, earlier| later.item == earlier.item);
    sorted.into_iter().take(2).collect()
}

/// 分辨：多出的步数不小于噪声的 10 倍；噪声 0 记「噪声 0」，算分辨够。
fn extra_is_resolved(extra: u64, noise: u64) -> bool {
    noise == 0 || extra >= NOISE_MULTIPLE * noise
}

/// 逐行差 `d_r`（旗上操作的行值减可比操作的行值）：前三大行的和占 `extra` 的比例，与那三行。
fn top_rows_share(differences: &[(String, i64)], extra: i64) -> (f64, Vec<(String, i64)>) {
    let mut sorted = differences.to_vec();
    sorted.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let top: Vec<(String, i64)> = sorted.into_iter().take(TOP_ROW_COUNT).filter(|row| row.1 > 0).collect();
    let share = if extra > 0 { top.iter().map(|row| row.1).sum::<i64>() as f64 / extra as f64 } else { 0.0 };
    (share, top)
}

/// 一个旗上操作的行五判定：分辨不够不进判；前三大行占比不到线记分散。
fn classify_extra(extra: i64, noise: u64, top_share: f64) -> &'static str {
    if extra <= 0 {
        "no_extra_steps"
    } else if !extra_is_resolved(u64::try_from(extra).expect("正数"), noise) {
        "not_resolved"
    } else if top_share >= TOP_ROWS_SHARE_AT_LEAST {
        "located_in_the_top_rows"
    } else {
        "spread_over_many_rows"
    }
}

/// 每轮的差占全部轮的差之和的比例：旗上操作与可比操作的固定点按轮对齐（缺的轮记 0），每轮带它的并入出口。
/// 轮内步数是 `round_row` 的原始计数（含整份分配器复制），与 `extra` 的 B8 模型步数不是同一口径，所以分母是各轮差的和，不是 `extra`；和不为正记 0。
fn round_shares(flagged: &[(String, u64)], comparable: &[(String, u64)]) -> Vec<(usize, String, f64)> {
    let rounds = flagged.len().max(comparable.len());
    let differences: Vec<i64> = (0..rounds)
        .map(|position| flagged.get(position).map_or(0, |round| round.1) as i64 - comparable.get(position).map_or(0, |round| round.1) as i64)
        .collect();
    let total: i64 = differences.iter().sum();
    (0..rounds)
        .map(|position| {
            let exit = flagged.get(position).map_or_else(|| "none".to_string(), |round| round.0.clone());
            (position + 1, exit, if total > 0 { differences[position] as f64 / total as f64 } else { 0.0 })
        })
        .collect()
}

/// B8 每一行在 `crates/` 里的步：(行名, 文件（相对 `crates/singlefs-core/src/`）, 函数名)。函数名空是「位置待定」。
const ROW_FUNCTIONS: [(&str, &str, &str); 28] = [
    ("whole_allocator_copy_before_the_judgement", "transaction.rs", "publish_version_of_trees_relocating_units_after_a_check"),
    ("inode_record_linear_find", "transaction.rs", "inode_record_of"),
    ("files_other_than_the_first_plan", "transaction.rs", "plan_the_files_other_than_the_first_after_this_publish"),
    ("file_other_than_the_first_linear_find", "transaction.rs", "file_other_than_the_first"),
    ("inodes_with_an_extent_entry_set", "transaction.rs", "inodes_with_an_extent_upper_leaf_entry"),
    ("extent_upper_segment_plan", "extent_tree.rs", "plan_the_extent_upper_segment"),
    ("extent_roles_rewritten_scan", "transaction.rs", "extent_tree_roles_rewritten"),
    ("root_header_linear_find", "transaction.rs", "unit"),
    ("checkpoint_cost_accounting_tree_plan", "admission.rs", "accounting_tree_nodes_after_the_publish_built_on"),
    ("cluster_segment_sum", "allocator.rs", "unblocked_slot_pairs_outside_the_cluster_segments"),
    ("allocation_record_tree_node_set", "transaction.rs", "settle_the_allocation_record_tree"),
    ("allocation_record_tree_plan", "allocation_record_tree.rs", "plan_the_tree_after_this_publish"),
    ("inode_number_watermark_steps", "transaction.rs", "inode_number_watermark_steps"),
    ("deadlist_changes_indexes", "transaction.rs", "deadlist_changes_of_this_publish"),
    ("multi_level_tree_planning", "transaction.rs", "plan_the_multi_level_trees"),
    ("mapped_units_of_container_trees", "transaction.rs", "mapped_units_of_a_container_tree"),
    ("moved_entries_path_tail_filter", "transaction.rs", "plan_the_multi_level_trees"),
    ("release_judgement_version_indexes", "transaction.rs", "placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots"),
    ("whole_allocator_copy_in_a_settling_round", "transaction.rs", "settle_the_allocation_record_tree"),
    ("allocator_release_linear_find", "allocator.rs", "release_leaving_the_record_allocated_on"),
    ("user_data_slot_linear_scan", "allocator.rs", "lowest_unblocked_slot_pair_in_the_segment"),
    ("record_intersection_filter", "allocator.rs", "make_room_for_record_on_device"),
    ("empty_segment_linear_scan", "allocator.rs", "lowest_empty_segment_outside_the_region"),
    ("fallback_slot_linear_scan", "allocator.rs", "lowest_commit_generated_fallback_slot_outside_the_region"),
    ("records_of_each_leaf", "allocation_record_tree.rs", "records_of_each_leaf"),
    ("accounting_entries_of_this_publish", "transaction.rs", "accounting_entries_of_this_publish"),
    ("instance_table_chain", "transaction.rs", "instance_table_chain_to_release"),
    ("mapped_units_whole_table", "transaction.rs", ""),
];

/// 源文件里 `fn <函数名>` 恰好出现在一行的行号（1 起）；0 行或多行交 `None`（位置待定）。
fn line_of_the_function(source: &str, function: &str) -> Option<usize> {
    if function.is_empty() {
        return None;
    }
    let heads = [format!("fn {function}("), format!("fn {function}<")];
    let lines: Vec<usize> = source.lines().enumerate().filter(|(_, line)| heads.iter().any(|head| line.contains(head.as_str()))).map(|(position, _)| position + 1).collect();
    (lines.len() == 1).then(|| lines[0])
}

/// 行名在 `crates/` 里的位置：`<文件>:<行>`；没给仓根或查不到交 `position_pending`。
fn position_of_the_row(row: &str, crates_root: Option<&str>) -> String {
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
fn steps_per_extra_node(extra_steps: i64, extra_nodes: i64) -> Option<f64> {
    (extra_steps > 0 && extra_nodes > 0).then(|| (extra_steps as f64 / extra_nodes as f64 * 1000.0).round() / 1000.0)
}

// ---- 装置与实现对得上的检查（第七节 ②、③；停机条款）----

fn yes_no(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

/// 一个项的读数世界：不是孪生的项、各项的操作视图。
struct World<'items> {
    items: Vec<&'items MeasuredItem>,
    views: BTreeMap<String, Vec<OperationView>>,
}

impl<'items> World<'items> {
    fn new(items: &'items [MeasuredItem]) -> Self {
        let real: Vec<&MeasuredItem> = items.iter().filter(|item| !item.is_twin).collect();
        let views = real.iter().map(|item| (item.name.clone(), views_of_the_item(item))).collect();
        Self { items: real, views }
    }

    fn head_item(&self, point: &str, script: &str) -> Option<&'items MeasuredItem> {
        let name = scripts::item_name(point, "head", script);
        self.items.iter().copied().find(|item| item.name == name)
    }

    fn views_of(&self, point: &str, script: &str) -> Option<&Vec<OperationView>> {
        self.views.get(&scripts::item_name(point, "head", script))
    }

    fn points(&self) -> BTreeSet<String> {
        self.items.iter().filter(|item| item.arm == "head").map(|item| item.point.clone()).collect()
    }

    fn all_views_at(&self, point: &str) -> Vec<&OperationView> {
        self.items.iter().filter(|item| item.arm == "head" && item.point == point).flat_map(|item| self.views[&item.name].iter()).collect()
    }

    fn views_at_the_points(&self, points: &[&str]) -> Vec<&OperationView> {
        points.iter().flat_map(|point| self.all_views_at(point)).collect()
    }

    /// 一个点上每个脚本第 0 次挂载里的操作。
    fn first_mount_by_script(&self, point: &str) -> BTreeMap<String, Vec<&OperationView>> {
        self.items
            .iter()
            .filter(|item| item.arm == "head" && item.point == point)
            .map(|item| (item.script.clone(), first_mount(&self.views[&item.name])))
            .collect()
    }
}

/// 第七节 ② 第 1 条：产物头里实现的常量与本地常量逐个比；不等的项数。
/// 只比计数版：登记第七节 ② 第 1 条写的是「与计数版产物头逐个比」，plain 版只拷不插、产物里没有常量行，不核。
fn constants_mismatches(item: &MeasuredItem) -> Vec<String> {
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
fn history_mismatch(item: &MeasuredItem) -> Option<String> {
    let expected = scripts::notes_of_the_history(item.files);
    (item.history_user_changes != Some(expected)).then(|| format!("user_changes={:?}≠{expected}", item.history_user_changes))
}

/// F12：产物里 `script_step` 行的步序列与脚本表逐步比；有一处不同交描述。
fn declared_sequence_mismatch(item: &MeasuredItem) -> Option<String> {
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
struct ClosureCheck {
    publishes: usize,
    closure_mismatches: Vec<String>,
    final_not_covering: usize,
    final_nodes_mismatches: usize,
    walk_mismatches: usize,
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

fn closure_check(item: &MeasuredItem) -> ClosureCheck {
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
fn twin_difference(original: &MeasuredItem, twin: &MeasuredItem) -> Option<usize> {
    let longer = original.body_lines.len().max(twin.body_lines.len());
    (0..longer).find(|position| original.body_lines.get(*position) != twin.body_lines.get(*position)).map(|position| position + 1)
}

/// 第七节 ③ 第 4 条：计数版与 plain 版在历史末与脚本末的 `image_sha256` 是否逐字节相同。
fn plain_matches_counts(counts: &MeasuredItem, plain: &MeasuredItem) -> bool {
    let pick = |item: &MeasuredItem, when: &str| item.images.iter().find(|(label, _)| label == when).map(|(_, sha)| sha.clone());
    ["history_end", "script_end"].iter().all(|when| pick(counts, when).is_some() && pick(counts, when) == pick(plain, when))
}

/// 第七节 ③ 第 6 条：历史末的 `mapped_units` ≥ F × K + 10（第一个量窗操作之前那一行规模读数）。
fn live_units_shortfall(item: &MeasuredItem) -> Option<String> {
    let lower = item.files * item.units_per_file + scripts::BIG_FILE_UNITS;
    match number(&item.first_state, "mapped_units") {
        Some(mapped) if mapped >= lower => None,
        other => Some(format!("mapped_units={other:?}<{lower}")),
    }
}

/// 第七节 ③ 第 8 条：阴性对照（`first_repeated` 末尾的 `truncate_to_the_same_size`）轮数不超过 4。
fn negative_control_rounds(item: &MeasuredItem) -> Option<u64> {
    item.operations.iter().find(|operation| operation.name == "truncate_to_the_same_size").and_then(|operation| operation.settles.iter().map(|settle| settle.rounds).max())
}

// ---- 报告：每个量各占一行，各报各的判定（第六节）----

fn list_text(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(",")
    }
}

fn first_labels(labels: &[String]) -> String {
    list_text(&labels.iter().take(5).cloned().collect::<Vec<_>>())
}

/// 训练集、检验集一、检验集二（第六节开头）。
struct Sets<'views> {
    training: Vec<&'views OperationView>,
    script: Vec<&'views OperationView>,
    geometry: Vec<&'views OperationView>,
}

fn sets_of<'views>(world: &'views World<'_>) -> Sets<'views> {
    let training = world
        .all_views_at(scripts::TRAINING_POINT)
        .into_iter()
        .filter(|view| TRAINING_SCRIPTS.contains(&view.script.as_str()) && view.epoch == 0)
        .collect();
    let script = world.all_views_at(scripts::TRAINING_POINT).into_iter().filter(|view| !TRAINING_SCRIPTS.contains(&view.script.as_str())).collect();
    let geometry = world.views_at_the_points(&scripts::TEST_GEOMETRY_POINTS);
    Sets { training, script, geometry }
}

fn checks_lines(world: &World<'_>, all_items: &[MeasuredItem]) -> (Vec<String>, BTreeMap<&'static str, u64>) {
    let mut lines = Vec::new();
    let mut counts: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut bump = |name: &'static str, failed: bool| {
        *counts.entry(name).or_insert(0) += u64::from(failed);
    };
    for item in &world.items {
        let constants = constants_mismatches(item);
        let history = history_mismatch(item);
        let sequence = declared_sequence_mismatch(item);
        let closure = closure_check(item);
        let shortfall = if item.arm == "head" { live_units_shortfall(item) } else { None };
        bump("items_without_item_done", !item.done);
        bump("items_refused", item.refused);
        bump("constants_mismatches", !constants.is_empty());
        bump("history_mismatches", history.is_some());
        bump("script_sequence_mismatches", sequence.is_some());
        bump("closure_model_mismatches", !closure.closure_mismatches.is_empty());
        bump("final_set_not_covering", closure.final_not_covering > 0);
        bump("final_nodes_mismatches", closure.final_nodes_mismatches > 0);
        bump("edge_walk_mismatches", closure.walk_mismatches > 0);
        bump("live_units_shortfalls", shortfall.is_some());
        lines.push(format!(
            "name=item_checks item={} done={} refused={} constants={} history={} script_sequence={} closure_publishes={} closure_mismatches={} final_not_covering={} final_nodes_mismatches={} edge_walk_mismatches={} live_units={}",
            item.name,
            yes_no(item.done),
            yes_no(item.refused),
            list_text(&constants),
            history.unwrap_or_else(|| "ok".to_string()),
            sequence.unwrap_or_else(|| "ok".to_string()),
            closure.publishes,
            first_labels(&closure.closure_mismatches),
            closure.final_not_covering,
            closure.final_nodes_mismatches,
            closure.walk_mismatches,
            shortfall.unwrap_or_else(|| "ok".to_string())
        ));
        if item.script == "first_repeated" {
            let rounds = negative_control_rounds(item);
            bump("negative_control_over_four_rounds", rounds.is_none_or(|rounds| rounds > 4));
            lines.push(format!("name=negative_control item={} rounds={}", item.name, rounds.map_or("unread".to_string(), |rounds| rounds.to_string())));
        }
    }
    let twins: Vec<&MeasuredItem> = all_items.iter().filter(|item| item.is_twin).collect();
    let mut twin_failures = 0;
    for twin in &twins {
        let original = world.items.iter().find(|item| item.name == twin.name);
        let difference = original.and_then(|original| twin_difference(original, twin));
        twin_failures += u64::from(original.is_none() || difference.is_some());
        lines.push(format!("name=determinism_twin item={} identical={} first_difference_line={}", twin.name, yes_no(original.is_some() && difference.is_none()), difference.map_or("-".to_string(), |line| line.to_string())));
    }
    counts.insert("twin_differences", twin_failures);
    let mut plain_failures = 0;
    for plain in world.items.iter().filter(|item| item.arm == "plain") {
        let counts_item = world.head_item(&plain.point, &plain.script);
        let equal = counts_item.is_some_and(|counts_item| plain_matches_counts(counts_item, plain));
        plain_failures += u64::from(!equal);
        lines.push(format!("name=plain_versus_counts item={} image_sha256_equal={}", plain.name, yes_no(equal)));
    }
    counts.insert("plain_differences", plain_failures);
    (lines, counts)
}

fn freshness_lines(sets: &Sets<'_>) -> Vec<String> {
    let mut lines = Vec::new();
    for (variant, tag) in FRESHNESS_VARIANTS.iter().enumerate() {
        let reading = judge_freshness_rule(variant, &sets.training, &sets.script, &sets.geometry);
        lines.push(format!(
            "name=freshness_rule variant={tag} threshold={} flagged_in_training={} training_failures={} training_failure_labels={} script_flagged={} script_unflagged={} script_mismatches={} script_mismatch_labels={} geometry_flagged={} geometry_unflagged={} geometry_mismatches={} geometry_mismatch_labels={} verdict={}",
            reading.threshold.map_or("unread".to_string(), |value| value.to_string()),
            reading.flagged_in_training,
            reading.defining_failures.len(),
            first_labels(&reading.defining_failures),
            reading.script.flagged,
            reading.script.unflagged,
            reading.script.mismatches.len(),
            first_labels(&reading.script.mismatches),
            reading.geometry.flagged,
            reading.geometry.unflagged,
            reading.geometry.mismatches.len(),
            first_labels(&reading.geometry.mismatches),
            reading.verdict().tag()
        ));
    }
    lines
}

fn manipulation_lines(world: &World<'_>) -> Vec<String> {
    let first_mount_by_script = world.first_mount_by_script(scripts::TRAINING_POINT);
    let every: Vec<&OperationView> = world.all_views_at(scripts::TRAINING_POINT);
    let reading = judge_manipulation(&first_mount_by_script, &every);
    let mut lines: Vec<String> = reading
        .flagged_by_script
        .iter()
        .map(|(script, count, selectors)| format!("name=freshness_manipulation_script script={script} flagged_operations={count} selectors={}", list_text(&selectors.iter().cloned().collect::<Vec<_>>())))
        .collect();
    lines.push(format!(
        "name=freshness_manipulation scripts={} position_follows={} count_follows={} retouched_operations={} retouched_and_flagged={} retouched_flagged_labels={}",
        reading.flagged_by_script.len(),
        yes_no(reading.position_follows),
        yes_no(reading.count_follows),
        reading.retouch_operations,
        reading.retouch_flagged.len(),
        list_text(&reading.retouch_flagged)
    ));
    lines
}

fn ordinal_lines(world: &World<'_>, point: &str) -> Vec<String> {
    let by_script = world.first_mount_by_script(point);
    let equality = judge_ordinal_equality(&by_script);
    let mut lines = vec![format!(
        "name=ordinal_equality point={point} script_pairs={} flag_depends_only_on_the_ordinal={} first_difference={}",
        equality.pairs,
        yes_no(equality.pairs > 0 && equality.first_difference.is_none()),
        equality.first_difference.clone().unwrap_or_else(|| "-".to_string())
    )];
    let Some(walk_forward) = by_script.get("walk_forward") else {
        return lines;
    };
    let fillers: Vec<(String, u64, Vec<&OperationView>)> = by_script
        .iter()
        .filter_map(|(script, views)| {
            let count = script.strip_prefix("filler_create_").or_else(|| script.strip_prefix("filler_young_"))?.parse().ok()?;
            Some((script.clone(), count, views.clone()))
        })
        .collect();
    let shift = judge_ordinal_shift(walk_forward, &fillers);
    for row in &shift.rows {
        lines.push(format!(
            "name=ordinal_shift_row point={point} script={} filler_count={} first_ordinal={} first_flagged_ordinal={} first_flagged_selector={}",
            row.script,
            row.filler_count,
            row.first_ordinal.map_or("-".to_string(), |value| value.to_string()),
            row.first_flagged_ordinal.map_or("-".to_string(), |value| value.to_string()),
            if row.first_flagged_selector.is_empty() { "-" } else { &row.first_flagged_selector }
        ));
    }
    lines.push(format!(
        "name=ordinal_shift point={point} walk_forward_first_flagged_ordinal={} fillers={} verdict={}",
        shift.walk_forward_first_flagged_ordinal.map_or("-".to_string(), |value| value.to_string()),
        shift.rows.len(),
        shift.verdict
    ));
    lines
}

fn quantity_lines(world: &World<'_>, sets: &Sets<'_>) -> Vec<String> {
    let pooled_views: Vec<&OperationView> = world.all_views_at(scripts::TRAINING_POINT).into_iter().filter(|view| view.epoch == 0).collect();
    let mut lines = Vec::new();
    for (quantity_position, name) in ["stale_nodes", "chain_depth", "released_backlog", "publishes_since_flag"].iter().enumerate() {
        for relative in [false, true] {
            let pick = |views: &[&OperationView]| -> Vec<(f64, bool)> {
                views.iter().map(|view| {
                    let (_, absolute, ratio) = view.quantities()[quantity_position];
                    (if relative { ratio } else { absolute }, view.flagged)
                }).collect()
            };
            let reading = judge_stored_quantity(&pick(&pooled_views), &pick(&sets.geometry));
            lines.push(format!(
                "name=stored_quantity quantity={name} relative={} threshold={:.4} pooled_errors={} flagged_values={} unflagged_values={} geometry_errors={} verdict={}",
                yes_no(relative),
                reading.pooled.threshold,
                reading.pooled.errors,
                reading.pooled.flagged_values,
                reading.pooled.unflagged_values,
                reading.geometry_errors,
                reading.verdict.tag()
            ));
        }
    }
    lines
}

fn presweep_lines(world: &World<'_>, point: &str) -> Vec<String> {
    let mut points = Vec::new();
    let mut scripts_of_the_sweep: Vec<(u64, String)> = vec![(0, "walk_forward".to_string())];
    scripts_of_the_sweep.extend([1, 2, 4, 8, 16, 32, 64].map(|count| (count, format!("presweep_{count}"))));
    for (count, script) in scripts_of_the_sweep {
        let Some(views) = world.views_of(point, &script) else {
            continue;
        };
        let first_mount_views = first_mount(views);
        let Some(first) = first_mount_views.iter().find(|view| view.selector == "first") else {
            continue;
        };
        let (flagged_ordinal, selector) = first_flagged(&first_mount_views);
        points.push(SweepPoint {
            presweep_count: count,
            flagged: first.flagged,
            stale_nodes: first.stale_nodes,
            publishes_since_flag: first.publishes_since_flag,
            final_nodes: first.final_nodes,
            first_flagged_ordinal: flagged_ordinal,
            first_flagged_selector: selector,
        });
    }
    let mut lines: Vec<String> = points
        .iter()
        .map(|sweep| {
            format!(
                "name=presweep_point point={point} presweep_count={} first_flagged={} stale_nodes={} publishes_since_flag={} final_nodes={} first_flagged_ordinal={} first_flagged_selector={}",
                sweep.presweep_count,
                yes_no(sweep.flagged),
                sweep.stale_nodes,
                sweep.publishes_since_flag,
                sweep.final_nodes,
                sweep.first_flagged_ordinal.map_or("-".to_string(), |value| value.to_string()),
                if sweep.first_flagged_selector.is_empty() { "-" } else { &sweep.first_flagged_selector }
            )
        })
        .collect();
    let reading = judge_presweep(&points);
    lines.push(format!(
        "name=presweep point={point} flagged_counts={} unflagged_counts={} between={} verdict={}",
        list_text(&reading.flagged_counts.iter().map(u64::to_string).collect::<Vec<_>>()),
        list_text(&reading.unflagged_counts.iter().map(u64::to_string).collect::<Vec<_>>()),
        reading.between.map_or("-".to_string(), |(flagged, unflagged)| format!("{flagged}..{unflagged}")),
        reading.verdict
    ));
    lines
}

/// 一个点上 `mount_then_first` 的行四读数（前提、集合、带出的那一步）；缺项交 `None`。
fn mount_reading_at(world: &World<'_>, point: &str) -> Option<(MountReading, Vec<PublishRecord>)> {
    let walk_forward = world.head_item(point, "walk_forward")?;
    let walk_views = world.views_of(point, "walk_forward")?;
    let first_in_walk = walk_views.iter().find(|view| view.selector == "first")?;
    let spike: NodeSet = first_in_walk.seed.union(&first_in_walk.pulled).copied().collect();
    let _ = walk_forward;
    let mounted = world.head_item(point, "mount_then_first")?;
    let mounted_views = world.views_of(point, "mount_then_first")?;
    let first_after = mounted_views.iter().find(|view| view.epoch == 1 && view.selector == "first")?;
    let remount = mounted.remounts.iter().find(|remount| remount.epoch == 1)?;
    let steps: Vec<(String, NodeSet)> = remount.publishes.iter().map(|publish| (mount_step_name(publish), publish.final_set.clone())).collect();
    Some((judge_mount_carries(first_after.flagged, &spike, &steps), remount.publishes.clone()))
}

fn mount_lines(world: &World<'_>, point: &str, crates_root: Option<&str>) -> Vec<String> {
    let Some((reading, publishes)) = mount_reading_at(world, point) else {
        return vec![format!("name=mount_carries point={point} verdict=unread reason=the_walk_forward_or_mount_then_first_item_is_missing")];
    };
    let mut lines = vec![format!(
        "name=mount_carries point={point} first_after_the_mount_flagged={} nodes_of_the_spike={} carried_by_the_mount={} share={:.3} verdict={} remount_publishes={}",
        yes_no(reading.premise_flagged_after_the_mount),
        reading.nodes_of_the_spike,
        reading.carried,
        reading.share,
        reading.verdict,
        publishes.len()
    )];
    let steps_text: Vec<String> = reading.shares_by_step.iter().map(|(step, count)| format!("{step}:{count}")).collect();
    let (site, plan_function) = reading.located_step.as_ref().map_or(("-".to_string(), "-"), |step| {
        let record = publishes.iter().find(|publish| &mount_step_name(publish) == step);
        (record.map_or("-".to_string(), |publish| publish.site.clone()), plan_function_of_the_mount_step(step))
    });
    let plan_position = crates_root.map_or("position_pending".to_string(), |root| {
        let source = std::fs::read_to_string(format!("{root}/crates/singlefs-core/src/mount.rs")).unwrap_or_default();
        line_of_the_function(&source, plan_function).map_or("position_pending".to_string(), |line| format!("crates/singlefs-core/src/mount.rs:{line}"))
    });
    lines.push(format!(
        "name=mount_carried_step point={point} carried_nodes_by_first_step={} located_step={} verdict={} first_merge_site={site} plan_function={plan_function} plan_position={plan_position}",
        list_text(&steps_text),
        reading.located_step.clone().unwrap_or_else(|| "-".to_string()),
        reading.one_step_verdict
    ));
    lines
}

/// Q10：先碰再挂载与两次挂载、挂载前碰的不是同一批：逐项报，不判。
fn mount_variant_lines(world: &World<'_>, point: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for script in ["first_mount_first", "mount_twice_first", "young_mount_first", "old_mount_old"] {
        let (Some(item), Some(views)) = (world.head_item(point, script), world.views_of(point, script)) else {
            continue;
        };
        let carried_by_the_mount: NodeSet = item.remounts.iter().flat_map(|remount| remount.publishes.iter()).flat_map(|publish| publish.final_set.iter().copied()).collect();
        let before_the_mount: NodeSet = views.iter().filter(|view| view.epoch == 0).flat_map(|view| view.seed.union(&view.pulled).copied()).collect();
        for view in views.iter().filter(|view| view.epoch > 0) {
            let nodes: NodeSet = view.seed.union(&view.pulled).copied().collect();
            lines.push(format!(
                "name=mount_variant item={} operation={} step_index={} flagged={} fresh_mount_wide={} fresh_ever_wide={} nodes={} in_the_mount_publishes={} touched_before_the_mount={} not_in_the_mount_publishes={}",
                item.name,
                view.name,
                view.step_index,
                yes_no(view.flagged),
                view.fresh[0],
                view.fresh[2],
                nodes.len(),
                nodes.intersection(&carried_by_the_mount).count(),
                nodes.intersection(&before_the_mount).count(),
                nodes.difference(&carried_by_the_mount).count()
            ));
        }
    }
    lines
}

/// 行五（Q11、Q12、Q13）：一个点上每个旗上操作与它的可比操作的行表差。
fn extra_lines(world: &World<'_>, point: &str, crates_root: Option<&str>) -> Vec<String> {
    let candidates: Vec<OperationView> = world.all_views_at(point).into_iter().cloned().collect();
    let mut lines = Vec::new();
    let steps_of = |view: &OperationView| -> Option<BTreeMap<&'static str, u64>> {
        let item = world.items.iter().find(|item| item.name == view.item)?;
        let operation = item.operations.iter().find(|operation| operation.step_index == view.step_index)?;
        rows_of_the_operation(item, operation)
    };
    let rounds_of = |view: &OperationView| -> Vec<(String, u64)> {
        world
            .items
            .iter()
            .find(|item| item.name == view.item)
            .and_then(|item| item.operations.iter().find(|operation| operation.step_index == view.step_index))
            .map_or_else(Vec::new, |operation| operation.settles.iter().flat_map(|settle| settle.rounds_detail.iter().cloned()).collect())
    };
    for target in candidates.iter().filter(|view| view.flagged && view.epoch == 0) {
        let comparables = pick_comparables(target, &candidates);
        let label = format!("{}#{}", target.item, target.step_index);
        let (Some(first), Some(second)) = (comparables.first(), comparables.get(1)) else {
            lines.push(format!("name=extra_steps operation={label} comparables={} verdict=no_comparable_operation", comparables.len()));
            continue;
        };
        let (Some(rows), Some(first_rows), Some(second_rows)) = (steps_of(target), steps_of(first), steps_of(second)) else {
            lines.push(format!("name=extra_steps operation={label} verdict=unread reason=a_judgement_reading_is_missing"));
            continue;
        };
        let total = |rows: &BTreeMap<&'static str, u64>| rows.values().sum::<u64>() as i64;
        let extra = total(&rows) - total(&first_rows);
        let noise = (total(&second_rows) - total(&first_rows)).unsigned_abs();
        let differences: Vec<(String, i64)> = rows.iter().map(|(row, steps)| ((*row).to_string(), *steps as i64 - first_rows.get(row).copied().unwrap_or(0) as i64)).collect();
        let (share, top) = top_rows_share(&differences, extra);
        let verdict = classify_extra(extra, noise, share);
        let top_text: Vec<String> = top.iter().map(|(row, difference)| format!("{row}:{difference}")).collect();
        let round_text: Vec<String> = round_shares(&rounds_of(target), &rounds_of(first)).iter().map(|(round, exit, round_share)| format!("{round}:{exit}:{round_share:.3}")).collect();
        lines.push(format!(
            "name=extra_steps operation={label} selector={} comparable_first={}#{} comparable_second={}#{} steps={} steps_first={} steps_second={} extra={extra} noise={noise} top_rows={} top_share={share:.3} round_shares_of_the_raw_difference={} verdict={verdict}",
            target.selector,
            first.item,
            first.step_index,
            second.item,
            second.step_index,
            total(&rows),
            total(&first_rows),
            total(&second_rows),
            list_text(&top_text),
            list_text(&round_text)
        ));
        if verdict == "located_in_the_top_rows" {
            for (row, difference) in &top {
                lines.push(format!("name=extra_steps_position operation={label} row={row} difference={difference} position={}", position_of_the_row(row, crates_root)));
            }
        }
        let final_of = |view: &OperationView| view.seed.union(&view.pulled).count() as i64;
        lines.push(format!(
            "name=extra_steps_per_node operation={label} extra_nodes={} steps_per_extra_node={}",
            final_of(target) - final_of(first),
            steps_per_extra_node(extra, final_of(target) - final_of(first)).map_or("not_defined_without_positive_extra".to_string(), |value| format!("{value:.3}"))
        ));
    }
    lines
}

/// 旗清单（Q1）：每个项每个用户操作一行。
fn operation_lines(world: &World<'_>) -> Vec<String> {
    let mut lines = Vec::new();
    for item in &world.items {
        for view in &world.views[&item.name] {
            lines.push(format!(
                "name=operation_row item={} mount_epoch={} ordinal_user={} step_index={} operation={} final_nodes={} flagged={} flagged_at_the_low_line={} flagged_at_the_high_line={} median_inflated={} fresh_mount_wide={} fresh_mount_narrow={} fresh_ever_wide={} fresh_ever_narrow={} seed_nodes={} pulled_nodes={} stale_nodes={} chain_depth={} released_backlog={} publishes_since_flag={}",
                view.item,
                view.epoch,
                view.ordinal_user,
                view.step_index,
                view.name,
                view.final_nodes,
                yes_no(view.flagged),
                yes_no(view.flag_by_line[LOW_FLOOR_LINE_POSITION]),
                yes_no(view.flag_by_line[2]),
                yes_no(view.median_inflated),
                view.fresh[0],
                view.fresh[1],
                view.fresh[2],
                view.fresh[3],
                view.seed.len(),
                view.pulled.len(),
                view.stale_nodes,
                view.chain_depth,
                view.released_backlog,
                view.publishes_since_flag
            ));
        }
    }
    lines
}

/// 复现闸（Q0）：每个点的 `walk_forward` 里旗上操作的个数；返回行与每点的判定。
fn gate_lines(world: &World<'_>) -> (Vec<String>, Vec<(String, &'static str)>) {
    let mut lines = Vec::new();
    let mut verdicts = Vec::new();
    for point in world.points() {
        let Some(views) = world.views_of(&point, "walk_forward") else {
            continue;
        };
        let nodes: Vec<u64> = first_mount(views).iter().map(|view| view.final_nodes).collect();
        let gate = reproduction_gate(&nodes);
        lines.push(format!(
            "name=reproduction_gate point={point} operations={} median_final_nodes={} line_at_the_main_floor={} flagged_at_14={} flagged_at_28={} flagged_at_56={} verdict={}",
            nodes.len(),
            lower_median(&nodes),
            spike_line(SPIKE_FLOOR, &nodes),
            gate.flagged_by_line[0],
            gate.flagged_by_line[1],
            gate.flagged_by_line[2],
            gate.verdict
        ));
        verdicts.push((point, gate.verdict));
    }
    (lines, verdicts)
}

/// 几何敏感性（第八节）：同一个量在每个取样点上的判定串；一个点与训练点判不同记不稳定。
fn geometry_sensitivity_line(quantity: &str, per_point: &[(String, String)]) -> String {
    let stable = per_point.windows(2).all(|pair| pair[0].1 == pair[1].1);
    let text: Vec<String> = per_point.iter().map(|(point, verdict)| format!("{point}:{verdict}")).collect();
    format!("name=geometry_sensitivity quantity={quantity} points={} per_point={} stable={}", per_point.len(), list_text(&text), yes_no(stable))
}

fn geometry_sensitivity_lines(world: &World<'_>, sets: &Sets<'_>, gate: &[(String, &'static str)], crates_root: Option<&str>) -> Vec<String> {
    let mut lines = Vec::new();
    let _ = crates_root;
    let points: Vec<String> = world.points().into_iter().collect();
    lines.push(geometry_sensitivity_line("reproduction_gate", &gate.iter().map(|(point, verdict)| (point.clone(), (*verdict).to_string())).collect::<Vec<_>>()));
    let threshold = threshold_from_training(&sets.training, 0);
    let rule: Vec<(String, String)> = points
        .iter()
        .map(|point| {
            let views = world.all_views_at(point);
            let verdict = threshold.map_or("not_run".to_string(), |threshold| format!("mismatches_{}", mismatches_at(&views, 0, threshold).mismatches.len()));
            (point.clone(), verdict)
        })
        .collect();
    lines.push(geometry_sensitivity_line("freshness_rule_mount_wide", &rule));
    let equality: Vec<(String, String)> = points.iter().map(|point| (point.clone(), yes_no(judge_ordinal_equality(&world.first_mount_by_script(point)).first_difference.is_none()).to_string())).collect();
    lines.push(geometry_sensitivity_line("ordinal_equality", &equality));
    let mount: Vec<(String, String)> = points.iter().map(|point| (point.clone(), mount_reading_at(world, point).map_or("unread", |(reading, _)| reading.verdict).to_string())).collect();
    lines.push(geometry_sensitivity_line("mount_carries", &mount));
    let located: Vec<(String, String)> = points
        .iter()
        .map(|point| {
            let verdicts: BTreeSet<String> = extra_lines(world, point, None).iter().filter_map(|line| line.rsplit_once("verdict=").map(|(_, verdict)| verdict.to_string())).filter(|verdict| !verdict.contains(' ')).collect();
            (point.clone(), list_text(&verdicts.into_iter().collect::<Vec<_>>()))
        })
        .collect();
    lines.push(geometry_sensitivity_line("extra_steps_location", &located));
    lines
}

/// 读完全部产物之后的报告：每个量各占一行，末尾 `name=verdict`。
fn report(items: &[MeasuredItem], crates_root: Option<&str>) -> Vec<String> {
    let world = World::new(items);
    let sets = sets_of(&world);
    let (check_lines, failures) = checks_lines(&world, items);
    let (gate_text, gate_verdicts) = gate_lines(&world);
    let mut lines = Vec::new();
    lines.extend(check_lines);
    lines.extend(gate_text);
    lines.extend(operation_lines(&world));
    lines.extend(freshness_lines(&sets));
    lines.extend(manipulation_lines(&world));
    for point in world.points() {
        lines.extend(ordinal_lines(&world, &point));
        lines.extend(mount_lines(&world, &point, crates_root));
        lines.extend(extra_lines(&world, &point, crates_root));
    }
    lines.extend(quantity_lines(&world, &sets));
    lines.extend(presweep_lines(&world, scripts::TRAINING_POINT));
    for point in world.points() {
        lines.extend(mount_variant_lines(&world, &point));
        if point != scripts::TRAINING_POINT {
            lines.extend(presweep_lines(&world, &point));
        }
    }
    lines.extend(geometry_sensitivity_lines(&world, &sets, &gate_verdicts, crates_root));
    let count = |name: &str| failures.get(name).copied().unwrap_or(0);
    let zero_or = |name: &str| if count(name) == 0 { "zero".to_string() } else { count(name).to_string() };
    let training_gate = gate_verdicts.iter().find(|(point, _)| point == scripts::TRAINING_POINT).map_or("not_run", |(_, verdict)| *verdict);
    lines.push(format!(
        "name=verdict items_read={} items_without_item_done={} items_refused={} constants_mismatches={} history_mismatches={} script_sequence_mismatches={} closure_model_mismatches={} final_set_not_covering={} final_nodes_mismatches={} edge_walk_mismatches={} live_units_shortfalls={} negative_control_over_four_rounds={} twin_differences={} plain_differences={} reproduction_at_the_training_point={training_gate}",
        world.items.len(),
        zero_or("items_without_item_done"),
        zero_or("items_refused"),
        zero_or("constants_mismatches"),
        zero_or("history_mismatches"),
        zero_or("script_sequence_mismatches"),
        zero_or("closure_model_mismatches"),
        zero_or("final_set_not_covering"),
        zero_or("final_nodes_mismatches"),
        zero_or("edge_walk_mismatches"),
        zero_or("live_units_shortfalls"),
        zero_or("negative_control_over_four_rounds"),
        zero_or("twin_differences"),
        zero_or("plain_differences")
    ));
    lines
}

fn list_items() -> Vec<String> {
    scripts::registered_items().into_iter().map(|(name, segment)| format!("{name} {segment}")).collect()
}

/// `report` 的参数：`--crates-root <仓根>`、`--out <新文件>`（报告行同时写进这个文件）、其余是产物文件。
fn split_report_arguments(arguments: &[String]) -> (Option<String>, Option<String>, Vec<String>) {
    let mut crates_root: Option<String> = None;
    let mut output_path: Option<String> = None;
    let mut paths = Vec::new();
    let mut rest = arguments.iter();
    while let Some(argument) = rest.next() {
        if argument == "--crates-root" {
            crates_root = rest.next().cloned();
        } else if argument == "--out" {
            output_path = rest.next().cloned();
        } else {
            paths.push(argument.clone());
        }
    }
    (crates_root, output_path, paths)
}

/// 报告行写进一个排他新建的文件：同名文件已在就拒（产物不覆盖）。
fn write_new_report_file(path: &str, lines: &[String]) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("报告文件 {path} 不能新建：{error}"))?;
    for line in lines {
        writeln!(file, "{line}").map_err(|error| format!("写 {path} 失败：{error}"))?;
    }
    Ok(())
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("list-items") => {
            for line in list_items() {
                println!("{line}");
            }
        }
        Some("report") => {
            let (crates_root, output_path, paths) = split_report_arguments(&arguments[1..]);
            let mut items = Vec::new();
            for path in &paths {
                match read_item(path) {
                    Ok(item) => items.push(item),
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                }
            }
            let lines: Vec<String> = report(&items, crates_root.as_deref()).into_iter().map(|line| format!("E209RESULT {line}")).collect();
            for line in &lines {
                println!("{line}");
            }
            if let Some(path) = output_path {
                if let Err(error) = write_new_report_file(&path, &lines) {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            }
        }
        _ => {
            eprintln!("用法：e209-spike-cause report [--crates-root <仓根>] [--out <新文件>] <产物文件>… | list-items");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 合成日志（第五节「阳性对照」）：节点抽象成编号，tree = 1..12；a 种子 {1}、拉进 {2..6}，b 种子 {7}、拉进空，c 种子 {8}、拉进 {9..12} ----

    fn publish_of(name: &str) -> PublishInput {
        let (seed, pulled): (Vec<u64>, Vec<u64>) = match name {
            "a" => (vec![1], vec![2, 3, 4, 5, 6]),
            "b" => (vec![7], vec![]),
            "c" => (vec![8], vec![9, 10, 11, 12]),
            other => panic!("合成日志里没有操作 {other}"),
        };
        let final_numbers: Vec<u64> = seed.iter().chain(&pulled).copied().collect();
        PublishInput {
            seed: synthetic_set(&seed),
            pulled: synthetic_set(&pulled),
            final_set: synthetic_set(&final_numbers),
            previous_nodes: synthetic_set(&(1..=12).collect::<Vec<u64>>()),
            ..PublishInput::default()
        }
    }

    fn synthetic_steps(sequence: &[&str]) -> Vec<StepInput> {
        sequence
            .iter()
            .map(|name| {
                if *name == "remount" {
                    StepInput::Remount(Vec::new())
                } else {
                    StepInput::Operation(OperationInput { name: (*name).to_string(), selector: (*name).to_string(), final_nodes: 7, publishes: vec![publish_of(name)], ..OperationInput::default() })
                }
            })
            .collect()
    }

    fn synthetic_views(item: &str, sequence: &[&str]) -> Vec<OperationView> {
        compute_views(item, "synthetic", item, (&NodeSet::new(), &NodeSet::new()), 0, &synthetic_steps(sequence))
    }

    /// 三条生成规则：`fresh ≥ 5`、用户序号 1 或 3、`Q_stale ≥ 6`（量名在产物里叫 `stale_nodes`）。
    fn with_flags(mut views: Vec<OperationView>, rule: &str) -> Vec<OperationView> {
        for view in &mut views {
            view.flagged = match rule {
                "by_nodes" => view.fresh[0] >= 5,
                "by_ordinal" => view.ordinal_user == 1 || view.ordinal_user == 3,
                "by_quantity" => view.stale_nodes >= 6,
                other => panic!("没有生成规则 {other}"),
            };
        }
        views
    }

    fn flagged_ordinals(views: &[OperationView]) -> Vec<u64> {
        views.iter().filter(|view| view.flagged).map(|view| view.ordinal_user).collect()
    }

    fn pair(rule: &str) -> (Vec<OperationView>, Vec<OperationView>) {
        (with_flags(synthetic_views("forward_script", &["a", "b", "c", "a", "c"]), rule), with_flags(synthetic_views("rotated_script", &["b", "c", "a", "c", "a"]), rule))
    }

    fn refs(views: &[OperationView]) -> Vec<&OperationView> {
        views.iter().collect()
    }

    #[test]
    fn the_synthetic_logs_have_the_registered_freshness_and_stored_quantity() {
        let (forward_views, rotated_views) = pair("by_nodes");
        let column = |views: &[OperationView], variant: usize| views.iter().map(|view| view.fresh[variant]).collect::<Vec<_>>();
        assert_eq!(column(&forward_views, 0), [6, 1, 5, 0, 0], "宽读：种子与拉进都算");
        assert_eq!(column(&rotated_views, 0), [1, 5, 6, 0, 0]);
        assert_eq!(column(&forward_views, 1), [5, 0, 4, 0, 0], "窄读：只算拉进的");
        assert_eq!(forward_views.iter().map(|view| view.stale_nodes).collect::<Vec<_>>(), [12, 6, 5, 0, 0]);
        assert_eq!(rotated_views.iter().map(|view| view.stale_nodes).collect::<Vec<_>>(), [12, 11, 6, 0, 0]);
        assert_eq!(forward_views.iter().map(|view| view.ordinal_user).collect::<Vec<_>>(), [1, 2, 3, 4, 5], "序号从 1 数");
    }

    #[test]
    fn the_three_generation_rules_flag_the_registered_ordinals() {
        let expected = [("by_nodes", vec![1, 3], vec![2, 3]), ("by_ordinal", vec![1, 3], vec![1, 3]), ("by_quantity", vec![1, 2], vec![1, 2, 3])];
        for (rule, on_forward, on_rotated) in expected {
            let (forward_views, rotated_views) = pair(rule);
            assert_eq!(flagged_ordinals(&forward_views), on_forward, "{rule} 在 X 上");
            assert_eq!(flagged_ordinals(&rotated_views), on_rotated, "{rule} 在 Y 上");
        }
    }

    #[test]
    fn the_freshness_judge_row_of_the_matrix_is_green_red_red() {
        let (forward_views, rotated_views) = pair("by_nodes");
        let reading = judge_freshness_rule(0, &refs(&forward_views), &refs(&rotated_views), &[]);
        assert_eq!((reading.threshold, reading.agrees(), reading.script.mismatches.len()), (Some(5), true, 0), "gen_by_nodes 绿：0 处不符");
        let (forward_views, rotated_views) = pair("by_ordinal");
        let reading = judge_freshness_rule(0, &refs(&forward_views), &refs(&rotated_views), &[]);
        assert!(!reading.agrees());
        assert!(reading.defining_failures.is_empty());
        assert_eq!(reading.script.mismatches.len(), 2, "gen_by_ordinal 红：Y 上 2 处不符");
        assert!(reading.script.mismatches[0].contains("#1:") && reading.script.mismatches[1].contains("#2:"), "序号 1、2：{:?}", reading.script.mismatches);
        let (forward_views, rotated_views) = pair("by_quantity");
        let reading = judge_freshness_rule(0, &refs(&forward_views), &refs(&rotated_views), &[]);
        assert_eq!(reading.threshold, Some(1), "m 取旗上操作新鲜度的最小值：6、1 取 1");
        assert_eq!(reading.defining_failures.len(), 1, "gen_by_quantity 红：X 上定规则失败 1 处");
        assert!(reading.defining_failures[0].contains("#3:fresh=5"), "序号 3 的 fresh = 5：{:?}", reading.defining_failures);
        assert_eq!(reading.verdict(), RuleVerdict::DoesNotHold);
    }

    #[test]
    fn the_threshold_boundary_turns_the_green_cell_red_outside_two_to_five() {
        let (forward_views, _) = pair("by_nodes");
        let wrong = |threshold: u64| mismatches_at(&refs(&forward_views), 0, threshold).mismatches.len();
        for threshold in 2..=5 {
            assert_eq!(wrong(threshold), 0, "m = {threshold} 在有效区间里");
        }
        assert_eq!(wrong(6), 1, "m = 6：c 的 fresh = 5 被判不旗");
        assert_eq!(wrong(1), 1, "m = 1：b 的 fresh = 1 被判旗");
    }

    #[test]
    fn the_ordinal_judge_row_of_the_matrix_is_red_green_red() {
        let first_difference = |rule: &str| {
            let (forward_views, rotated_views) = pair(rule);
            let by_script = BTreeMap::from([("forward_script".to_string(), refs(&forward_views)), ("rotated_script".to_string(), refs(&rotated_views))]);
            judge_ordinal_equality(&by_script).first_difference
        };
        assert_eq!(first_difference("by_nodes"), Some("forward_script/rotated_script#1".to_string()), "首处不同在序号 1");
        assert_eq!(first_difference("by_ordinal"), None, "gen_by_ordinal 绿");
        assert_eq!(first_difference("by_quantity"), Some("forward_script/rotated_script#3".to_string()));
    }

    #[test]
    fn a_pair_of_scripts_of_different_lengths_is_not_compared_past_the_shorter_one() {
        let long = with_flags(synthetic_views("long", &["a", "b", "c", "a", "c", "a"]), "by_ordinal");
        let short = with_flags(synthetic_views("short", &["a", "b", "c", "a"]), "by_ordinal");
        let by_script = BTreeMap::from([("long".to_string(), refs(&long)), ("short".to_string(), refs(&short))]);
        let reading = judge_ordinal_equality(&by_script);
        assert_eq!((reading.pairs, reading.first_difference), (1, None), "短的那边没有值的序号不报不同");
    }

    #[test]
    fn the_stored_quantity_judge_row_of_the_matrix_is_red_red_green() {
        let pooled = |rule: &str| {
            let (forward_views, rotated_views) = pair(rule);
            forward_views.iter().chain(&rotated_views).map(|view| (view.stale_nodes as f64, view.flagged)).collect::<Vec<_>>()
        };
        assert_eq!(judge_stored_quantity(&pooled("by_nodes"), &[]).verdict, QuantityVerdict::DoesNotHold, "Q = 12 在 X 上旗上、在 Y 上不旗");
        assert_eq!(judge_stored_quantity(&pooled("by_ordinal"), &[]).verdict, QuantityVerdict::DoesNotHold, "Q = 6 在 X 上不旗、在 Y 上旗");
        let reading = judge_stored_quantity(&pooled("by_quantity"), &[]);
        assert_eq!((reading.pooled.threshold, reading.pooled.errors, reading.pooled.flagged_values, reading.pooled.unflagged_values), (6.0, 0, 3, 2), "θ = 6，旗上取值 {{12,11,6}}，不旗取值 {{5,0}}");
        assert_eq!(reading.verdict, QuantityVerdict::TestGeometryNotRun, "检验几何还没跑");
    }

    #[test]
    fn the_threshold_of_the_stored_quantity_is_not_refitted_on_the_test_geometry() {
        let training = [(12.0, true), (11.0, true), (6.0, true), (5.0, false), (0.0, false)];
        let doubled_scale = [(24.0, true), (12.0, true), (10.0, false), (0.0, false)];
        let reading = judge_stored_quantity(&training, &doubled_scale);
        assert_eq!((reading.geometry_errors, reading.verdict), (1, QuantityVerdict::OnlyInTheTrainingGeometry), "量程乘 2 之后门槛 6 把 10 错判成旗上");
    }

    #[test]
    fn the_freshness_mount_variants_are_told_apart_by_a_remount() {
        let since_mount = |rule: &str| {
            let mut views = synthetic_views("w", &["a", "remount", "a", "b"]);
            for view in &mut views {
                view.flagged = match rule {
                    "since_mount" => view.fresh[0] >= 5,
                    "since_creation" => view.fresh[2] >= 5,
                    other => panic!("{other}"),
                };
            }
            views
        };
        let mount_views = since_mount("since_mount");
        assert_eq!(mount_views.iter().map(|view| view.fresh[0]).collect::<Vec<_>>(), [6, 6, 1], "挂载清零：重挂载之后的 a 又是 6");
        assert_eq!(mount_views.iter().map(|view| view.fresh[2]).collect::<Vec<_>>(), [6, 0, 1], "不清零：重挂载之后的 a 是 0");
        assert_eq!(mount_views.iter().map(|view| view.epoch).collect::<Vec<_>>(), [0, 1, 1]);
        assert_eq!(mount_views.iter().map(|view| view.ordinal_user).collect::<Vec<_>>(), [1, 1, 2], "重挂载之后用户序号从 1 重数");
        let creation_views = since_mount("since_creation");
        let check = |views: &[OperationView], variant: usize| judge_freshness_rule(variant, &refs(views), &[], &[]);
        assert!(check(&mount_views, 0).agrees() && check(&creation_views, 2).agrees(), "各配各的判器绿");
        assert_eq!(check(&creation_views, 0).defining_failures.len(), 1, "挂载判器对不清零的生成规则红 1 处");
        assert_eq!(check(&mount_views, 2).defining_failures.len(), 1, "不清零判器对清零的生成规则红 1 处");
    }

    #[test]
    fn the_spike_flag_uses_the_lower_median_and_the_floor() {
        assert_eq!((SPIKE_FLOOR, SPIKE_MULTIPLE), (28, 4), "同 E207 装置的两个常量");
        assert_eq!(spike_line(SPIKE_FLOOR, &[7, 7, 7, 433, 7]), 28);
        assert_eq!(spike_line(SPIKE_FLOOR, &[3, 3, 3, 3, 20]), 28);
        assert_eq!(lower_median(&[1, 2, 3, 4]), 2, "下中位数");
        let flags = flags_by_line(&[7, 7, 7, 60, 7]);
        assert_eq!(flags.iter().map(|row| row[MAIN_FLOOR_LINE_POSITION]).collect::<Vec<_>>(), [false, false, false, true, false], "下中位数 7 → 旗线 28，60 旗上；均值会把旗线抬到 68");
    }

    #[test]
    fn the_reproduction_gate_tells_the_low_line_from_the_main_line() {
        assert_eq!(reproduction_gate(&[3, 3, 3, 20, 3]), ReproductionGate { flagged_by_line: [1, 0, 0], verdict: "only_at_the_low_line" }, "中位数 3：L = 28 不旗、L = 14 旗上");
        assert_eq!(reproduction_gate(&[3, 3, 3, 60, 3]).verdict, "reproduced");
        assert_eq!(reproduction_gate(&[7, 7, 7, 7, 7]).verdict, "not_reproduced");
        assert_eq!(reproduction_gate(&[7, 7, 7, 20, 7]).verdict, "not_reproduced", "中位数 7 时 4 × 7 = 28 盖过 14，低旗线也不旗");
    }

    #[test]
    fn the_median_is_inflated_when_more_than_half_are_flagged() {
        assert!(median_is_inflated(&[true, true, true, false, false]));
        assert!(!median_is_inflated(&[true, true, false, false]));
    }

    fn mount_steps(carries: &str) -> Vec<(String, NodeSet)> {
        let (write_row, warm_up) = match carries {
            "carries" => (vec![1, 7], vec![2, 3, 4, 5, 6, 8]),
            "none" => (vec![7, 8], vec![7, 8]),
            "spread" => (vec![1, 2, 3, 7], vec![4, 5, 6, 8]),
            other => panic!("{other}"),
        };
        vec![("write_row".to_string(), synthetic_set(&write_row)), ("warm_up_1".to_string(), synthetic_set(&warm_up))]
    }

    #[test]
    fn the_mount_judge_is_green_red_amber_on_the_synthetic_mounts() {
        let spike = synthetic_set(&[2, 3, 4, 5, 6]);
        let carries = judge_mount_carries(false, &spike, &mount_steps("carries"));
        assert_eq!((carries.verdict, carries.located_step.as_deref(), carries.one_step_verdict), ("mount_carried_the_nodes", Some("warm_up_1"), "located_one_step"));
        let none = judge_mount_carries(true, &spike, &mount_steps("none"));
        assert_eq!(none.verdict, "mount_did_not_stop_the_spike", "挂载后 T 旗上：前提不成立");
        let spread = judge_mount_carries(false, &spike, &mount_steps("spread"));
        assert_eq!((spread.verdict, spread.located_step, spread.one_step_verdict), ("mount_carried_the_nodes", None, "spread_over_several_steps"));
        assert_eq!(spread.shares_by_step, [("write_row".to_string(), 2), ("warm_up_1".to_string(), 3)], "各占 40% 与 60%");
    }

    #[test]
    fn the_extra_steps_judge_separates_located_spread_and_unresolved() {
        let rows = |values: &[i64]| values.iter().enumerate().map(|(position, value)| (format!("row_{position}"), *value)).collect::<Vec<_>>();
        let (share, top) = top_rows_share(&rows(&[700, 250, 30, 20]), 1000);
        assert_eq!((top.len(), (share * 1000.0).round()), (3, 980.0), "前三大行占 98%");
        assert_eq!(classify_extra(1000, 0, share), "located_in_the_top_rows");
        let (share, _) = top_rows_share(&rows(&[100; 10]), 1000);
        assert_eq!(classify_extra(1000, 0, share), "spread_over_many_rows", "十行各 100，前三大行占 30%");
        let (share, _) = top_rows_share(&rows(&[600, 200, 150, 50]), 1000);
        assert_eq!(classify_extra(1000, 0, share), "located_in_the_top_rows", "前三大 95%、前两大只有 80%");
        assert!(extra_is_resolved(1000, 5) && !extra_is_resolved(1000, 150), "c2 − c1 = 5 分辨够、150 分辨不够（6.7 < 10）");
        assert_eq!(classify_extra(1000, 150, 0.98), "not_resolved");
        assert_eq!(classify_extra(1000, 0, 0.98), "located_in_the_top_rows", "噪声 0 算分辨够");
        let thresholds = (ONE_STEP_SHARE_AT_LEAST, MOUNT_SHARE_CARRIED_AT_LEAST, TOP_ROWS_SHARE_AT_LEAST);
        assert_eq!(thresholds, (0.9, 0.9, 0.9), "三条占比线都是 0.9，挪到 0.99 或 0.5 的自证由变异表走");
    }

    #[test]
    fn the_steps_per_extra_node_exists_only_for_a_positive_extra_in_steps_and_in_nodes() {
        assert_eq!(steps_per_extra_node(7428, 46), Some(161.478), "7428 ÷ 46 保留 3 位");
        assert_eq!(steps_per_extra_node(-10730, 26), None, "多出的步数为负：没有这个比");
        assert_eq!(steps_per_extra_node(0, 26), None, "多出的步数为 0：没有这个比");
        assert_eq!(steps_per_extra_node(100, -5), None, "节点反而少了：没有这个比");
        assert_eq!(steps_per_extra_node(100, 0), None, "节点没多：没有这个比");
    }

    #[test]
    fn the_round_shares_are_shares_of_the_sum_of_the_round_differences() {
        let rounds = |values: &[(&str, u64)]| values.iter().map(|(exit, steps)| (exit.to_string(), *steps)).collect::<Vec<_>>();
        let shares = round_shares(&rounds(&[("release_not_covered", 300), ("settled", 700)]), &rounds(&[("release_not_covered", 100), ("settled", 100)]));
        assert_eq!(shares, [(1, "release_not_covered".to_string(), 0.25), (2, "settled".to_string(), 0.75)], "差 200 与 600，和 800：占 25% 与 75%，两份加起来是 1");
        let shares = round_shares(&rounds(&[("settled", 500)]), &rounds(&[("settled", 100), ("settled", 400)]));
        assert_eq!(shares, [(1, "settled".to_string(), 0.0), (2, "none".to_string(), 0.0)], "差的和是 0：都记 0，缺的轮出口记 none");
    }

    fn comparable_view(item: &str, flagged: bool) -> OperationView {
        OperationView { item: item.to_string(), point: "p".to_string(), selector: "first".to_string(), old_units: 1, new_units: 1, flagged, step_index: 1, ..OperationView::default() }
    }

    #[test]
    fn the_comparables_are_the_first_two_unflagged_operations_by_item_name() {
        let target = comparable_view("p__head__walk_forward", true);
        let candidates = vec![
            comparable_view("p__head__walk_forward", true),
            comparable_view("p__head__c_script", false),
            comparable_view("p__head__a_script", false),
            comparable_view("p__head__b_script", false),
            comparable_view("p__head__flagged_script", true),
        ];
        let names: Vec<&str> = pick_comparables(&target, &candidates).iter().map(|view| view.item.as_str()).collect();
        assert_eq!(names, ["p__head__a_script", "p__head__b_script"]);
    }

    #[test]
    fn the_ordinal_shift_judge_separates_following_the_operation_from_following_the_ordinal() {
        let view = |ordinal: u64, selector: &str, flagged: bool| OperationView { ordinal_user: ordinal, selector: selector.to_string(), flagged, ..OperationView::default() };
        let walk = vec![view(1, "first", true), view(2, "old", false)];
        let walk_refs: Vec<&OperationView> = walk.iter().collect();
        let filler = |count: u64, flagged_ordinal: u64| -> (String, u64, Vec<OperationView>) {
            let views = (1..=count + 1).map(|ordinal| view(ordinal, if ordinal == count + 1 { "first" } else { "new_file" }, ordinal == flagged_ordinal)).collect();
            (format!("filler_create_{count}"), count, views)
        };
        let judged = |fillers: Vec<(String, u64, Vec<OperationView>)>| {
            let refs: Vec<(String, u64, Vec<&OperationView>)> = fillers.iter().map(|(name, count, views)| (name.clone(), *count, views.iter().collect())).collect();
            judge_ordinal_shift(&walk_refs, &refs).verdict
        };
        assert_eq!(judged(vec![filler(1, 2), filler(4, 5)]), "follows_the_operation", "first 在序号 n + 1 旗上");
        assert_eq!(judged(vec![filler(1, 1), filler(4, 1)]), "follows_the_ordinal", "旗留在序号 1");
        assert_eq!(judged(vec![filler(1, 2), filler(4, 1)]), "neither");
    }

    #[test]
    fn the_presweep_judge_needs_both_sides_of_the_threshold() {
        let point = |count: u64, flagged: bool| SweepPoint { presweep_count: count, flagged, stale_nodes: 0, publishes_since_flag: 0, final_nodes: 0, first_flagged_ordinal: None, first_flagged_selector: String::new() };
        let reading = judge_presweep(&[point(0, true), point(1, true), point(4, false), point(8, false)]);
        assert_eq!((reading.between, reading.verdict), (Some((1, 4)), "threshold_between_two_sample_points"));
        assert_eq!(judge_presweep(&[point(0, true), point(1, true)]).verdict, "threshold_sides_not_both_sampled");
        assert_eq!(judge_presweep(&[point(0, false), point(1, true)]).verdict, "not_monotone_in_the_presweep_count");
    }
}

#[cfg(test)]
mod pins {
    use super::*;

    fn position(level: u32, device: u32, index_in_device: u64) -> Node {
        Node::Position { level, device, index_in_device }
    }

    #[test]
    fn the_position_arithmetic_is_pinned_on_the_registered_slots() {
        assert_eq!(LEAF_SLOTS * INTERNAL_FANOUT, 137_228, "第七节 ③ 第 2 条");
        for (slot, leaf) in [(0, 0), (811, 0), (812, 1), (137_227, 168), (137_228, 169)] {
            assert_eq!(leaf_of_slot(0, slot), position(0, 0, leaf), "槽 {slot}");
        }
        assert_eq!(parent_of(position(0, 0, 168), 3), Some(position(1, 0, 0)));
        assert_eq!(parent_of(position(0, 0, 169), 3), Some(position(1, 0, 1)), "叶 169 是第二个第一层节点的头一个孩子");
        assert_eq!(parent_of(position(1, 2, 7), 2), Some(Node::Root), "父的层级等于根的层级时父是根");
        assert_eq!(parent_of(Node::Root, 2), None);
        let per_device = |slots_per_device: u64| {
            let leaves = slots_per_device.div_ceil(LEAF_SLOTS);
            (leaves, leaves.div_ceil(INTERNAL_FANOUT))
        };
        assert_eq!(per_device(1 << 17), (162, 1));
        assert_eq!(per_device(1 << 19), (646, 4));
        assert_eq!(per_device(1 << 20), (1292, 8));
    }

    #[test]
    fn the_history_user_changes_are_pinned_on_the_registered_points() {
        let notes: Vec<u64> = [10, 75, 100, 200].map(scripts::notes_of_the_history).to_vec();
        assert_eq!(notes, [42, 172, 222, 422], "第四节第 5 条 `python3 -I` 现算的值");
        assert_eq!(scripts::sample_point_named("files_100_units_100").map(|point| (point.files, point.units_per_file)), Some((100, 100)));
    }

    fn graph_with_two_devices() -> PointerGraph {
        PointerGraph::from([
            (position(0, 0, 0), vec![(0, 812 * 2 + 5), (1, 7)]),
            (position(0, 0, 2), vec![(0, 812 * 5)]),
        ])
    }

    #[test]
    fn the_closure_model_follows_both_old_locations_and_adds_the_ancestors_and_the_root() {
        let closure = release_chain_closure(&graph_with_two_devices(), 2, &BTreeSet::from([position(0, 0, 0)]));
        let expected: NodeSet = [position(0, 0, 0), position(0, 0, 2), position(0, 0, 5), position(0, 1, 0), position(1, 0, 0), position(1, 1, 0), Node::Root].into();
        assert_eq!(closure, expected, "第二块盘的落点拉进 (0,1,0) 与它的祖先，叶 (0,0,2) 自己的旧落点又拉进叶 5；每次都爬到根");
        assert!(closure.contains(&Node::Root));
        assert_eq!(release_chain_closure(&graph_with_two_devices(), 2, &BTreeSet::from([position(0, 0, 9)])), BTreeSet::from([position(0, 0, 9)]), "上一版没有的节点没有旧落点，走到它为止");
        assert_eq!(release_chain_depth(&graph_with_two_devices(), 2, position(0, 0, 0)), 3);
        assert_eq!(release_chain_depth(&graph_with_two_devices(), 2, position(0, 0, 9)), 1);
    }

    #[test]
    fn the_pointer_graph_text_gives_both_locations_of_every_node() {
        let graph = parse_pointer_graph("0/0/1@0.5+1.900,root@0.9+1.19").expect("读得出");
        assert_eq!(graph.get(&position(0, 0, 1)), Some(&vec![(0, 5), (1, 900)]), "两份落点都读");
        assert_eq!(graph.get(&Node::Root), Some(&vec![(0, 9), (1, 19)]));
        assert_eq!(parse_pointer_graph("-"), Some(PointerGraph::new()));
        assert_eq!(parse_pointer_graph("0/0/1@0.x"), None, "读不出整串不读");
        assert_eq!(parse_node_set("0/0/1,root"), Some(BTreeSet::from([position(0, 0, 1), Node::Root])));
        assert_eq!(parse_node_set("0/0/1,bad"), None);
    }

    const PRODUCT: &str = "\
E209RESULT name=header for_item=files_100_units_100__head__mount_then_first twin=no
E209RESULT name=point item=files_100_units_100__head__mount_then_first point=files_100_units_100 arm=head script=mount_then_first build=counts unit_area_slots=524288 unlinks=10 data_units=10 files=100 units_per_small_file=100 shape=small_files_of_units
E209RESULT name=constants item=files_100_units_100__head__mount_then_first leaf_slots=812 internal_fanout=169 settling_rounds_at_most=64 disks=2
E209RESULT name=history item=files_100_units_100__head__mount_then_first user_changes=222 settle_calls=222
E209RESULT name=history_carried item=files_100_units_100__head__mount_then_first ever_final=0/0/1,0/0/2 ever_pulled=-
E209RESULT name=script_step item=files_100_units_100__head__mount_then_first step_index=1 operation=remount mount_epoch=0
E209RESULT name=settle item=files_100_units_100__head__mount_then_first phase=remount mount_epoch=1 operation=remount operation_index=1 settle_call=223 judgement=0 rounds=1 outcome=settled final_nodes=2
E209RESULT name=nodes item=files_100_units_100__head__mount_then_first phase=remount mount_epoch=1 operation=remount operation_index=1 settle_call=223 publish_kind=write_row publish_order=0 first_merge_site=none seed=0/0/1 pulled=0/0/3 final=0/0/1,0/0/3 released_records_pre=4 edges=0/0/1>0/0/3 edges_walk_matches_the_implementation=1
E209RESULT name=pointer_graph item=files_100_units_100__head__mount_then_first phase=remount mount_epoch=1 operation=remount operation_index=1 settle_call=223 publish_kind=write_row publish_order=0 root_level=2 graph=0/0/1@0.2437+1.5
E209RESULT name=script_step item=files_100_units_100__head__mount_then_first step_index=2 operation=overwrite_first mount_epoch=1
E209RESULT name=declared item=files_100_units_100__head__mount_then_first phase=script mount_epoch=1 operation=overwrite_first operation_index=2 user_units=1 old_units=1 new_units=1
E209RESULT name=settle item=files_100_units_100__head__mount_then_first phase=script mount_epoch=1 operation=overwrite_first operation_index=2 settle_call=224 judgement=0 rounds=2 outcome=settled final_nodes=3
E209RESULT name=nodes item=files_100_units_100__head__mount_then_first phase=script mount_epoch=1 operation=overwrite_first operation_index=2 settle_call=224 publish_kind=user publish_order=0 first_merge_site=release_not_covered seed=0/0/1 pulled=0/0/2,0/0/3 final=0/0/1,0/0/2,0/0/3 released_records_pre=4 edges=- edges_walk_matches_the_implementation=1
E209RESULT name=item_done item=files_100_units_100__head__mount_then_first
";

    #[test]
    fn a_product_is_read_back_with_the_remount_publishes_and_the_seed_and_pulled_sets_apart() {
        let item = read_item_text(PRODUCT).expect("读得出");
        assert_eq!((item.name.as_str(), item.point.as_str(), item.script.as_str(), item.done), ("files_100_units_100__head__mount_then_first", "files_100_units_100", "mount_then_first", true));
        assert_eq!(item.remounts.len(), 1, "重挂载自己的发布读进来：重挂载阶段不暂停读数");
        let write_row = &item.remounts[0].publishes[0];
        assert_eq!((write_row.kind.as_str(), write_row.seed.clone(), write_row.pulled.clone()), ("write_row", synthetic_set(&[1]), synthetic_set(&[3])), "种子与拉进各归各");
        assert_eq!(write_row.edges, [(synthetic_node(1), synthetic_node(3))]);
        assert_eq!(write_row.graph.get(&synthetic_node(1)), Some(&vec![(0, 2437), (1, 5)]));
        let operation = &item.operations[0];
        assert_eq!((operation.epoch, operation.step_index, operation.settles[0].final_nodes, operation.publishes[0].final_set.len()), (1, 2, 3, 3));
        assert!(item.body_lines.iter().all(|line| !line.contains("name=header")), "孪生对拍去掉产物头");
        assert_eq!(item.history_carried_final, synthetic_set(&[1, 2]));
    }

    #[test]
    fn the_closure_check_reads_the_product_publishes_and_reports_the_final_node_count() {
        let item = read_item_text(PRODUCT).expect("读得出");
        let check = closure_check(&item);
        assert_eq!((check.publishes, check.final_not_covering, check.final_nodes_mismatches, check.walk_mismatches), (2, 0, 0, 0));
        assert_eq!(check.closure_mismatches.len(), 2, "这份产物的指针图只有一个节点、第二个发布没有图：模型闭包与测到的不等，对拍红");
    }

    #[test]
    fn the_mount_judge_runs_on_a_product_with_the_remount_publishes() {
        let items = vec![read_item_text(PRODUCT).expect("读得出")];
        let world = World::new(&items);
        assert!(mount_reading_at(&world, "files_100_units_100").is_none(), "没有 walk_forward 项：取不到");
        let views = world.views_of("files_100_units_100", "mount_then_first").expect("有视图");
        assert_eq!((views.len(), views[0].epoch, views[0].ordinal_user), (1, 1, 1));
        assert_eq!(views[0].fresh[0], 1, "宽读：{{1,2,3}} 去掉重挂载带出的 {{1,3}} 只剩 {{2}}");
        assert_eq!(views[0].fresh[2], 0, "不清零：历史里带出过 {{1,2}}、重挂载带出 {{1,3}}，一个不剩");
    }

    #[test]
    fn the_declared_sequence_is_checked_against_the_script_table() {
        let mut item = MeasuredItem { script: "old_mount_old".to_string(), ..MeasuredItem::default() };
        let steps = [(1, "overwrite_old", 0), (2, "remount", 0), (3, "overwrite_old", 1), (4, "overwrite_first", 1)];
        item.step_lines = steps.iter().map(|(index, name, epoch)| (*index, (*name).to_string(), *epoch)).collect();
        item.operations = [(1, "overwrite_old"), (3, "overwrite_old"), (4, "overwrite_first")].iter().map(|(index, name)| Operation { step_index: *index, name: (*name).to_string(), ..Operation::default() }).collect();
        assert_eq!(declared_sequence_mismatch(&item), None);
        item.step_lines[3].1 = "overwrite_young".to_string();
        assert!(declared_sequence_mismatch(&item).is_some_and(|text| text.contains("第 4 步")), "少一步、多一步、选择子取错都报");
        item.step_lines.pop();
        assert!(declared_sequence_mismatch(&item).is_some_and(|text| text.contains("步数")));
    }

    #[test]
    fn the_constants_and_the_history_are_checked_against_the_registered_values() {
        let mut item = read_item_text(PRODUCT).expect("读得出");
        assert!(constants_mismatches(&item).is_empty() && history_mismatch(&item).is_none());
        item.constants.insert("leaf_slots".to_string(), "811".to_string());
        item.history_user_changes = Some(221);
        assert_eq!(constants_mismatches(&item).len(), 1);
        item.arm = "plain".to_string();
        assert!(constants_mismatches(&item).is_empty(), "plain 版没有常量行：不核常量");
        item.arm = "head".to_string();
        assert!(history_mismatch(&item).is_some());
        assert_eq!(live_units_shortfall(&item).is_some(), true, "没有规模读数：不当 0");
        item.first_state = fields_of("E209RESULT name=state mapped_units=10010");
        assert_eq!(live_units_shortfall(&item), None, "F × K + 10 = 10010");
    }

    #[test]
    fn the_twin_and_the_plain_comparisons_catch_a_different_line_and_a_different_disk() {
        let original = read_item_text(PRODUCT).expect("读得出");
        let mut twin = original.clone();
        assert_eq!(twin_difference(&original, &twin), None);
        twin.body_lines[3] = "E209RESULT name=history item=x user_changes=223 settle_calls=222".to_string();
        assert_eq!(twin_difference(&original, &twin), Some(4));
        let with_images = |history_end: &str, script_end: &str| MeasuredItem { images: vec![("history_end".to_string(), history_end.to_string()), ("script_end".to_string(), script_end.to_string())], ..MeasuredItem::default() };
        assert!(plain_matches_counts(&with_images("aa", "bb"), &with_images("aa", "bb")));
        assert!(!plain_matches_counts(&with_images("aa", "bb"), &with_images("aa", "cc")));
        assert!(!plain_matches_counts(&with_images("aa", "bb"), &MeasuredItem::default()), "plain 没有读数：不当相同");
    }

    #[test]
    fn the_row_positions_are_found_by_function_name_and_unknown_ones_stay_pending() {
        assert_eq!(ROW_FUNCTIONS.len(), 28);
        let rows: Vec<&str> = by_delta_rows(&ByDeltaInput::default()).iter().map(|(row, _)| *row).collect();
        assert_eq!(rows, ROW_FUNCTIONS.iter().map(|(row, _, _)| *row).collect::<Vec<_>>(), "28 行的名字与次序同 by_delta_rows");
        let source = "pub fn other() {}\n    pub fn inode_record_of(&self) {}\nfn twice() {}\nfn twice(x: u64) {}\n";
        assert_eq!(line_of_the_function(source, "inode_record_of"), Some(2));
        assert_eq!(line_of_the_function(source, "twice"), None, "两行都有：不认");
        assert_eq!(line_of_the_function(source, ""), None);
        assert_eq!(position_of_the_row("mapped_units_whole_table", Some("/nonexistent")), "position_pending");
        assert_eq!(position_of_the_row("no_such_row", None), "position_pending");
    }

    #[test]
    fn the_by_delta_rows_are_pinned_on_a_constructed_input() {
        let input = ByDeltaInput { settling_rounds: 3, changed_leaves_per_round: vec![2, 3], allocation_tree_levels: 4, ..ByDeltaInput::default() };
        let rows: BTreeMap<&str, u64> = by_delta_rows(&input).into_iter().collect();
        assert_eq!(rows["allocation_record_tree_node_set"], 20, "5 片叶 × 4 层");
        assert_eq!(by_delta_total(&input), 40, "node_set 与 plan 两行各 20");
    }

    #[test]
    fn the_registered_item_names_round_trip_and_the_segments_add_up() {
        let items = scripts::registered_items();
        let in_segment = |segment: &str| items.iter().filter(|(_, candidate)| *candidate == segment).count();
        assert_eq!((in_segment("first"), in_segment("second"), in_segment("gate"), in_segment("third"), in_segment("fourth")), (6, 20, 1, 47, 4));
        assert_eq!(list_items().len(), items.len());
    }

    #[test]
    fn the_report_output_file_is_new_and_holds_the_same_lines() {
        let arguments: Vec<String> = ["--crates-root", "/root", "--out", "/new.out", "a.out", "b.out"].iter().map(|text| text.to_string()).collect();
        assert_eq!(split_report_arguments(&arguments), (Some("/root".to_string()), Some("/new.out".to_string()), vec!["a.out".to_string(), "b.out".to_string()]));
        let directory = std::env::temp_dir().join(format!("e209-report-out-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("report.out").to_string_lossy().to_string();
        let lines = vec!["E209RESULT name=a".to_string(), "E209RESULT name=b".to_string()];
        write_new_report_file(&path, &lines).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "E209RESULT name=a\nE209RESULT name=b\n");
        assert!(write_new_report_file(&path, &["E209RESULT name=c".to_string()]).is_err(), "同名文件已在：拒，不覆盖");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "E209RESULT name=a\nE209RESULT name=b\n", "拒了之后原文件不变");
        std::fs::remove_dir_all(&directory).unwrap();
    }
}
