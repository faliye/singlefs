//! E206（发布固定点轮数的代价）的计数模型：纯算术，不 `use` 任何 `crates/` 里的东西（跑前登记
//! `research/prompts/e206-preregistration.md` 第五节「装置写在哪」第 2 条）。
//!
//! 做的事：抽象固定点（第七节 ③）、节点上界与增长阶、B8（逐项按差量）的本地式子（E203（准入之前扫类的步代价）登记第五节 B8 一表与修订 7 的 s36）、
//! P3 的式子与常驻内存（第五节）、读量的那一路打出的 `E206RESULT` 行，逐项报 Q1–Q10、失败条款、作废条款与停机条款的判读。
//!
//! 用法：
//!   `e206-fixed-point-cost report <产物>…`                      读产物，每个量各占一行，打 `name=verdict`
//!   `e206-fixed-point-cost anchors`                             打第七节 ③ 与几何锚点（模型自己算的）
//!   `e206-fixed-point-cost e203-anchors <base 产物> <files_10000 产物>`  第七节 ② 第 3 条：读 E203 那两份计数产物的同一次挂载覆盖写第 1、2 轮，比 2102、2098、152344、4226
//! 退出码：0 读完；1 第七节 ② 第 3 条对不上；2 用法错或产物读不了。读不到的量报 `unread`，不当 0。
//! 单测钉第七节锚点的绝对值：`cargo test --release --bin e206-fixed-point-cost`。

#![allow(dead_code, reason = "模型的库函数一部分只有单测与报告的某一支用")]

use std::collections::{BTreeMap, BTreeSet};

/// 历史用例不是这个 bin 的目标（它拷进副本、当验证包的测试编，见入口 `e206-fixed-point-cost-run.sh`）；当文本读进来，让门禁 crate-sources-reachable 认它有着落。
const _HISTORIES_PAYLOAD: &str = include_str!("e206_fixed_point_cost_histories.rs");

/// 分配记录树一片叶罩多少槽（`crates/singlefs-format/src/lib.rs:163`）。
const LEAF_SLOTS: u64 = 812;
/// 内部节点扇出（`crates/singlefs-format/src/lib.rs:169`）。
const INTERNAL_FANOUT: u64 = 169;
/// 固定点上界（`crates/singlefs-core/src/transaction.rs:13466`）。
const SETTLING_ROUNDS_LIMIT: usize = 64;
/// 两块盘（第五节参数 `e142_parameters(512, 512)`）。
const DISKS: u64 = 2;
/// 单元区之前的槽数（6 MiB 环，`common_admission_pool` 同 E203）。
const UNIT_AREA_START_SLOT: u64 = 1408;
/// inode 叶容器每片记录数（E203 模型同一个常量）。
const INODE_LEAF_RECORDS: u64 = 233;
/// 每段槽数（E203 模型同一个常量）。
const SEGMENT_SLOTS: u64 = 64;
/// Q1、Q4 的门槛（第六节表后：8 = 2 × 源码注释「三四轮」的 4）。
const MANY_ROUNDS_ABOVE: u64 = 8;
/// Q4 有效轮数门槛。
const EFFECTIVE_ROUNDS_AT_MOST: f64 = 4.0;
/// Q6 写放大门槛（第六节表后，推的）。
const WRITE_AMPLIFICATION_AT_MOST: f64 = 1.25;
/// Q7 前提比门槛。
const PREMISE_RATIO_AT_MOST: f64 = 0.5;
/// Q10 判「释放链解释得了」的比例与容差。
const CHAIN_EXPLAINS_AT_LEAST: f64 = 0.9;
const CHAIN_TOLERANCE: i64 = 2;
/// 增长阶的轴端点（读法第 6 条）。
const AXIS_LOW: f64 = 10.0;
const AXIS_HIGH: f64 = 10_000.0;

// ---- 增长阶与判定带 ----
fn alpha(low_quantity: f64, high_quantity: f64, low_position: f64, high_position: f64) -> f64 {
    (high_quantity / low_quantity).ln() / (high_position / low_position).ln()
}

/// 读法第 6 条的 α：端点取 10 → 10⁴。
fn alpha_on_the_axis(low_quantity: f64, high_quantity: f64) -> f64 {
    alpha(low_quantity, high_quantity, AXIS_LOW, AXIS_HIGH)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Band {
    DoesNotGrow,
    Between,
    Grows,
}

impl Band {
    fn tag(self) -> &'static str {
        match self {
            Self::DoesNotGrow => "does_not_grow",
            Self::Between => "between",
            Self::Grows => "grows",
        }
    }
}

const BAND_DOES_NOT_GROW_AT_MOST: f64 = 0.25;
const BAND_GROWS_AT_LEAST: f64 = 0.5;

fn band(growth: f64) -> Band {
    if growth >= BAND_GROWS_AT_LEAST {
        Band::Grows
    } else if growth <= BAND_DOES_NOT_GROW_AT_MOST {
        Band::DoesNotGrow
    } else {
        Band::Between
    }
}

/// Q1 的三档：不多、多、撞上界（第六节表）。
fn rounds_verdict(peak_rounds: u64, did_not_settle: bool) -> &'static str {
    if did_not_settle || peak_rounds >= SETTLING_ROUNDS_LIMIT as u64 {
        "hits_the_limit"
    } else if peak_rounds > MANY_ROUNDS_ABOVE {
        "many"
    } else {
        "not_many"
    }
}

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

// ---- 几何（第七节 ③，`calc/anchors.py` 照 `allocation_record_tree.rs:97`–`:119` 的找法）----
/// (每盘叶数, 根层级, 全树节点上界)。
fn tree_geometry(unit_area_slots: u64) -> (u64, u32, u64) {
    let slots = UNIT_AREA_START_SLOT + unit_area_slots;
    let leaves = slots.div_ceil(LEAF_SLOTS);
    let mut level = 1_u32;
    loop {
        let span = LEAF_SLOTS * INTERNAL_FANOUT.pow(level - 1);
        if DISKS * slots.div_ceil(span) <= INTERNAL_FANOUT {
            break;
        }
        level += 1;
    }
    let below_the_root: u64 = (0..level)
        .map(|layer| DISKS * slots.div_ceil(LEAF_SLOTS * INTERNAL_FANOUT.pow(layer)))
        .sum();
    (leaves, level, 1 + below_the_root)
}

// ---- 抽象固定点（第七节 ③）：节点是叶，`g[n]` 是节点旧落点那条记录所在的叶，次序照 `transaction.rs:13378`–`:13457` ----
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModelArm {
    Today,
    Closure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ModelOutcome {
    /// 定下那一轮的序号；走满上界没定是 `None`。
    rounds: Option<u64>,
    per_round: Vec<u64>,
}

impl ModelOutcome {
    /// 轮功 W = Σ_r A_r。
    fn round_work(&self) -> u64 {
        self.per_round.iter().sum()
    }

    /// 有效轮数 W ÷ A_final（读法第 4 条）。
    fn effective_rounds(&self) -> f64 {
        self.round_work() as f64 / *self.per_round.last().unwrap_or(&1) as f64
    }

    /// P3 的轮间差之和（第五节 P3 式子的抽象形：每轮只碰比上一轮多出来的角色）。
    fn sum_of_increments(&self) -> u64 {
        let mut before = 0;
        let mut sum = 0;
        for roles in &self.per_round {
            sum += roles.abs_diff(before);
            before = *roles;
        }
        sum
    }
}

fn close_over_the_release_chain(graph: &BTreeMap<u64, u64>, set: &mut BTreeSet<u64>) {
    let mut frontier: Vec<u64> = set.iter().copied().collect();
    while let Some(node) = frontier.pop() {
        let next = graph[&node];
        if set.insert(next) {
            frontier.push(next);
        }
    }
}

fn settle_model(
    graph: &BTreeMap<u64, u64>,
    dirtied_by_the_ordinary_roles: &BTreeSet<u64>,
    ordinary_roles: u64,
    leaf_of_the_new_placement: &dyn Fn(u64) -> u64,
    arm: ModelArm,
    limit: usize,
) -> ModelOutcome {
    let mut rewritten: BTreeSet<u64> = BTreeSet::new();
    let mut per_round = Vec::new();
    for round in 1..=limit {
        let plan = rewritten.clone();
        let roles = ordinary_roles + plan.len() as u64;
        per_round.push(roles);
        let mut release_dirty = dirtied_by_the_ordinary_roles.clone();
        release_dirty.extend(plan.iter().map(|node| graph[node]));
        if !release_dirty.is_subset(&plan) {
            rewritten.extend(release_dirty.iter().copied());
            if arm == ModelArm::Closure {
                close_over_the_release_chain(graph, &mut rewritten);
            }
            continue;
        }
        let mut changed = release_dirty;
        changed.extend((0..roles).map(leaf_of_the_new_placement));
        if changed.is_subset(&plan) {
            return ModelOutcome {
                rounds: Some(round as u64),
                per_round,
            };
        }
        rewritten.extend(changed);
        if arm == ModelArm::Closure {
            close_over_the_release_chain(graph, &mut rewritten);
        }
    }
    ModelOutcome {
        rounds: None,
        per_round,
    }
}

/// 第七节 ③ 的链：k 片叶，第 i 片叶的旧落点在第 i + 1 片，末片在自己；普通角色弄脏第 0 片，新落点全在链尾。
fn chain_outcome(leaves: u64, ordinary_roles: u64, arm: ModelArm) -> ModelOutcome {
    let graph: BTreeMap<u64, u64> = (0..leaves)
        .map(|leaf| (leaf, (leaf + 1).min(leaves - 1)))
        .collect();
    settle_model(
        &graph,
        &BTreeSet::from([0]),
        ordinary_roles,
        &|_| leaves - 1,
        arm,
        SETTLING_ROUNDS_LIMIT,
    )
}

/// 第七节 ③ 的跨叶图：`g = {0:1, 1:2, 2:2, 3:4, 4:4}`，m = 2，前 4 个新落点在叶 2、之后在叶 3。
fn crossing_outcome(arm: ModelArm) -> ModelOutcome {
    let graph = BTreeMap::from([(0, 1), (1, 2), (2, 2), (3, 4), (4, 4)]);
    settle_model(
        &graph,
        &BTreeSet::from([0]),
        2,
        &|index| if index < 4 { 2 } else { 3 },
        arm,
        SETTLING_ROUNDS_LIMIT,
    )
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

/// 这几行每轮一份，P3 把它们的输入换成轮间差（第五节 P3 式子）；别的行 P1、P2a、P3 同一份。
const ROWS_THAT_BETWEEN_ROUNDS_PRICING_CHANGES: [&str; 14] = [
    "allocation_record_tree_node_set",
    "allocation_record_tree_plan",
    "deadlist_changes_indexes",
    "multi_level_tree_planning",
    "mapped_units_of_container_trees",
    "release_judgement_version_indexes",
    "whole_allocator_copy_in_a_settling_round",
    "allocator_release_linear_find",
    "user_data_slot_linear_scan",
    "empty_segment_linear_scan",
    "fallback_slot_linear_scan",
    "record_intersection_filter",
    "records_of_each_leaf",
    "mapped_units_whole_table",
];

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

// ---- P3（第五节「P3 的式子」）----
/// 一轮喂给 P3 式子的输入。`P3Input::between_rounds` 取轮间差，阳性对照 `P3Input::cumulative` 照 B8 取相对上一版的累计，同一个式子。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct BetweenRoundsPricingInput {
    changed_leaves: u64,
    code_two_keys: Vec<(u64, u64)>,
    containers: u64,
    mapped_changed: u64,
    records_in_the_view: u64,
    release_finds: u64,
    intersection_finds: u64,
    intersecting_records: u64,
    dirty_records: u64,
    query_levels: u64,
    user_data_finds: u64,
    empty_segment_finds: u64,
    fallback_finds: u64,
}

#[derive(Clone, Copy, Debug)]
struct BetweenRoundsUnitPrices {
    allocation_tree_levels: u64,
    allocation_records: u64,
    mapped_units: u64,
    segments_per_disk: u64,
}

/// 一轮的步：s15、s16 两行，s20、s21，s36，s24、s25、s27、s30，s19、s23（E203 B8 那几行的单价）。
fn between_rounds_pricing_round_steps(input: &BetweenRoundsPricingInput, prices: BetweenRoundsUnitPrices) -> u64 {
    let trees: u64 = input
        .code_two_keys
        .iter()
        .map(|(keys, height)| keys * height)
        .sum();
    2 * input.changed_leaves * prices.allocation_tree_levels
        + trees
        + input.containers
        + input.mapped_changed
        + input.mapped_changed * lg(prices.mapped_units)
        + input.records_in_the_view
        + input.release_finds * lg(prices.allocation_records)
        + input.intersection_finds * lg(prices.allocation_records)
        + input.intersecting_records
        + input.dirty_records
        + input.query_levels
        + input.user_data_finds * (levels_64(prices.segments_per_disk) + 32)
        + input.empty_segment_finds * levels_64(prices.segments_per_disk)
        + input.fallback_finds * (levels_64(prices.segments_per_disk) + 64)
}

// ---- 产物的读法 ----
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
#[derive(Clone, Debug, Default)]
struct Round {
    round: u64,
    roles_rewritten: u64,
    nodes_rewritten: u64,
    rewrite_set_size: u64,
    release_dirty_new: u64,
    placement_dirty_new: u64,
    exit: String,
    roles_changed: u64,
    records_changed: u64,
    tree_keys_changed: Vec<u64>,
    /// 行 → (调用数, 步数, 查询层数之和)。
    rows: BTreeMap<String, (u64, u64, u64)>,
}

/// 一次固定点（第 1、4、6、7 组）：量窗里全有，历史里只有第 1 组。
#[derive(Clone, Debug, Default)]
struct Settle {
    state: String,
    operation: String,
    operation_index: u64,
    judgement: u64,
    rounds: u64,
    outcome: String,
    final_roles: u64,
    final_nodes: u64,
    round_work: u64,
    final_set_hash: String,
    seed_nodes: u64,
    closure_hops: u64,
    closure_lookup_steps: u64,
    slots_placed: u64,
    previous_nodes: u64,
    release_chain_depth: u64,
    release_chain_nodes: u64,
    active_leaves: Vec<u64>,
    detail: Vec<Round>,
}

impl Settle {
    fn did_not_settle(&self) -> bool {
        self.outcome == "did_not_settle"
    }

    /// 这一条固定点所在那次判的键：`<状态>/<操作>/<第几次>/<第几次判>`。
    fn publish_key(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.state, self.operation, self.operation_index, self.judgement
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
struct PointArmReadings {
    name: String,
    point: String,
    arm: String,
    unit_area_slots: u64,
    unlinks: u64,
    data_units: u64,
    files: u64,
    constants: Fields,
    unit_bytes: u64,
    /// 历史里每次固定点，次序照发生：直方图里的 `sequence` 解开。
    history: Vec<Settle>,
    histogram_calls: u64,
    histogram_rounds_three_or_more: u64,
    history_settle_lines: u64,
    history_line: Option<Fields>,
    window: Vec<Settle>,
    judgements: BTreeMap<String, JudgementReadings>,
    image_sha256_at_the_history_end: Option<String>,
    reconciliations: Vec<(String, u64, u64)>,
    clause_violations: Vec<String>,
    refused: Option<String>,
    done: bool,
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
fn take_line(item: &mut PointArmReadings, fields: &Fields) {
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

fn read_item(path: &str) -> Result<PointArmReadings, String> {
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

/// P3 的阳性对照：两道之间那一段的 B8 输入并成一份累计，走同一个 P3 式子。
fn between_rounds_pricing_cumulative_input(input: &ByDeltaInput) -> BetweenRoundsPricingInput {
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

fn between_rounds_unit_prices(input: &ByDeltaInput) -> BetweenRoundsUnitPrices {
    BetweenRoundsUnitPrices {
        allocation_tree_levels: input.allocation_tree_levels,
        allocation_records: input.allocation_records,
        mapped_units: input.mapped_units,
        segments_per_disk: input.segments_per_disk,
    }
}

/// B8 列里 P3 换掉的那几行之和（阳性对照比的另一边；`between_rounds_pricing_control_gap` 要它等于累计走 P3 式子）。
fn by_delta_rows_that_between_rounds_pricing_changes(input: &ByDeltaInput) -> u64 {
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
struct BetweenRoundsPricingReading {
    largest_round: u64,
    sum_of_rounds: u64,
    memory_bytes: u64,
    maintenance_steps: u64,
}

fn between_rounds_pricing_reading(settle: &Settle, input: &ByDeltaInput, unit_bytes: u64) -> BetweenRoundsPricingReading {
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
fn premise_ratio(settle: &Settle) -> f64 {
    let numerator: u64 = settle
        .detail
        .iter()
        .map(|round| round.records_changed)
        .sum();
    numerator as f64 / settle.round_work.max(1) as f64
}

// ---- 一条臂一个点上的量（第六节、第八节）----
#[derive(Clone, Debug, Default)]
struct Peaks {
    rounds: u64,
    rounds_at: String,
    round_work: u64,
    final_roles: u64,
    final_nodes: u64,
    effective_rounds: f64,
    did_not_settle: bool,
}

fn peaks_of<'settles>(settles: impl Iterator<Item = &'settles Settle>) -> Peaks {
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

fn all_settles(item: &PointArmReadings) -> impl Iterator<Item = &Settle> {
    item.history.iter().chain(&item.window)
}

fn rounds_held_down_verdict(peak_rounds: u64, effective_rounds: f64) -> &'static str {
    if peak_rounds <= MANY_ROUNDS_ABOVE && effective_rounds <= EFFECTIVE_ROUNDS_AT_MOST {
        "held_down"
    } else {
        "not_held_down"
    }
}

/// Q9：`R*` 那一次之后 8 次发布内回不回到 `≤ 4`；同一状态里两次 `≥ 9` 是不是隔了 8 次以上。
fn fallback_and_recurrence(sequence: &[u64]) -> (bool, bool) {
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
fn chain_explained_ratio(settles: &[Settle]) -> Option<f64> {
    let many: Vec<&Settle> = settles.iter().filter(|settle| settle.rounds >= 9).collect();
    if many.is_empty() {
        return None;
    }
    let explained = many
        .iter()
        .filter(|settle| {
            (settle.rounds as i64 - (settle.release_chain_depth as i64 + 2)).abs()
                <= CHAIN_TOLERANCE
        })
        .count();
    Some(explained as f64 / many.len() as f64)
}

/// `release_not_covered` 在长轮固定点各轮里的占比（Q10 报数不判），`placement_not_covered` 占多数时 F4 判得出。
fn exit_share(settles: &[Settle], exit: &str) -> f64 {
    let rounds: Vec<&Round> = settles
        .iter()
        .filter(|settle| settle.rounds >= 9)
        .flat_map(|settle| &settle.detail)
        .collect();
    rounds.iter().filter(|round| round.exit == exit).count() as f64 / rounds.len().max(1) as f64
}

/// 两段固定点序列逐个比哈希（Q5、F3、对照）：(相同的个数, 比的个数, 第一处不同的下标)。
fn same_final_sets(one: &[&Settle], other: &[&Settle]) -> (usize, usize, Option<usize>) {
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
fn same_readings(one: &[&Settle], other: &[&Settle]) -> bool {
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
fn single_step_control_holds(
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
fn item_is_complete(item: &PointArmReadings) -> Result<(), String> {
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
fn model_mismatches(item: &PointArmReadings) -> Vec<String> {
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
fn between_rounds_pricing_control_gap(item: &PointArmReadings) -> (u64, u64) {
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
fn judgement_of(item: &PointArmReadings, settle: &Settle) -> Option<JudgementReadings> {
    let key = settle.publish_key();
    let mut judgement = item.judgements.get(&key)?.clone();
    let position = key.rfind('/')?;
    judgement.state = item.judgements.get(&key[..=position])?.state.clone();
    Some(judgement)
}

// ---- 报告：每个量各占一行，各报各的判定（第六节）----
const AXES: [(&str, [&str; 4]); 3] = [
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
const AXIS_POSITIONS: [f64; 4] = [10.0, 100.0, 1_000.0, 10_000.0];
const MEASURED_ARMS: [&str; 4] = ["p1", "p2a", "p2b16", "p2b256"];

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
fn pooled_fallback_of_the_window(item: &PointArmReadings) -> (u64, u64, u64, u64, u64) {
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
            let second = window_settle(item, "W1", 2).map(|settle| settle.rounds);
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
fn write_amplification_verdict(ratio: f64) -> &'static str {
    if ratio <= WRITE_AMPLIFICATION_AT_MOST {
        "acceptable"
    } else {
        "expensive"
    }
}

/// Q5：`fixed_point_steps(P2a) + 闭包走今天的线性 pointer_of 的步 − fixed_point_steps(P1)`。
fn net_steps_linear(arm: &Settle, base: &Settle) -> i64 {
    arm.fixed_point_steps() as i64 + arm.closure_lookup_steps as i64
        - base.fixed_point_steps() as i64
}

/// Q5：同上，闭包每一跳按索引记 `lg(节点数)`。
fn net_steps_indexed(arm: &Settle, base: &Settle) -> i64 {
    arm.fixed_point_steps() as i64 + (arm.closure_hops * lg(arm.previous_nodes)) as i64
        - base.fixed_point_steps() as i64
}

/// 作废条款第 4 条：`A_final` 在轴两端 |α| ≤ 0.1，轮数 ≤ 4。
fn negative_control_holds(low: &Settle, high: &Settle) -> bool {
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
fn verdicts_agree(readings: &[String]) -> bool {
    let verdicts: BTreeSet<&str> = readings
        .iter()
        .filter_map(|reading| reading.rsplit(':').next())
        .collect();
    verdicts.len() <= 1
}

/// 第七节 ② 第 1、2 条：一次固定点的轮数、轮功与逐轮角色数都对得上。
fn anchor_matches(settle: &Settle, rounds: u64, work: u64, roles: &[u64]) -> bool {
    let got: Vec<u64> = settle
        .detail
        .iter()
        .map(|round| round.roles_rewritten)
        .collect();
    settle.rounds == rounds && settle.round_work == work && got == roles
}

/// 第七节 ③ 对账、作废条款第 7 条：看盘与看记录两路的槽数相等。
fn reconciliation_holds(image_changed_unit_slots: u64, slots_placed: u64) -> bool {
    image_changed_unit_slots == slots_placed
}

/// F2：一个点上 `rounds ≥ floor` 的固定点次数，历史与量窗都算。
fn rounds_at_least(item: &PointArmReadings, floor: u64) -> usize {
    all_settles(item)
        .filter(|settle| settle.rounds >= floor)
        .count()
}

/// 第一段那 16 个点（登记第五节「实现量与分段」）。
const FIRST_SEGMENT_POINTS: [&str; 16] = [
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
fn fork_rows_with_enough_readings(
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

fn report(paths: &[String]) -> Result<(), String> {
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
fn e203_judgement(path: &str, round: u64) -> Result<(JudgementReadings, u64), String> {
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

/// 第七节 ② 第 3 条的四个数：同一次挂载覆盖写第 1、2 轮，`base`、`files_10000`。
const E203_ANCHORS: [(u64, u64, u64); 2] = [(0, 2102, 2098), (1, 152_344, 4226)];

fn e203_anchors(paths: &[String]) -> Result<bool, String> {
    let mut all_match = true;
    for (index, first, second) in E203_ANCHORS {
        let path = &paths[usize::try_from(index).expect("下标")];
        for (round, expected) in [(1, first), (2, second)] {
            let (judgement, unit_area_slots) = e203_judgement(path, round)?;
            let got = by_delta_of_the_judgement(&judgement, unit_area_slots)
                .map_err(|gaps| format!("{path} 第 {round} 轮读不到：{gaps:?}"))?;
            println!("name=e203_anchor product={path} round={round} total={got} expected={expected} matches={}", got == expected);
            all_match &= got == expected;
        }
    }
    Ok(all_match)
}

/// `anchors` 子命令：第七节 ③ 模型自己算的。
fn print_anchors() {
    for leaves in [1, 5, 20, 70] {
        for (arm, name) in [(ModelArm::Today, "p1"), (ModelArm::Closure, "p2a")] {
            let outcome = chain_outcome(leaves, 3, arm);
            println!("name=chain leaves={leaves} ordinary_roles=3 arm={name} rounds={:?} W={} per_round={:?} sum_of_increments={}", outcome.rounds, outcome.round_work(), outcome.per_round, outcome.sum_of_increments());
        }
    }
    for (arm, name) in [(ModelArm::Today, "p1"), (ModelArm::Closure, "p2a")] {
        let outcome = crossing_outcome(arm);
        println!(
            "name=crossing arm={name} rounds={:?} W={} per_round={:?}",
            outcome.rounds,
            outcome.round_work(),
            outcome.per_round
        );
    }
    for exponent in [14, 17, 18, 19, 20, 24] {
        let (leaves, level, nodes) = tree_geometry(1 << exponent);
        println!("name=geometry unit_area_exponent={exponent} leaves_per_disk={leaves} root_level={level} nodes_at_most={nodes}");
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let code = match arguments.first().map(String::as_str) {
        Some("anchors") => {
            print_anchors();
            0
        }
        Some("report") if arguments.len() > 1 => match report(&arguments[1..]) {
            Ok(()) => 0,
            Err(problem) => {
                eprintln!("  ✗ {problem}\n     → 怎么办：把产物路径改对，或先跑入口把产物落下来");
                2
            }
        },
        Some("e203-anchors") if arguments.len() == 3 => match e203_anchors(&arguments[1..]) {
            Ok(true) => 0,
            Ok(false) => 1,
            Err(problem) => {
                eprintln!("  ✗ {problem}\n     → 怎么办：两份产物从 git 取（git show 84b4ae23f:research/results/<名>）");
                2
            }
        },
        _ => {
            eprintln!("用法：e206-fixed-point-cost report <产物>… | anchors | e203-anchors <base 产物> <files_10000 产物>");
            2
        }
    };
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn called(rows: &[(&str, u64)]) -> BTreeMap<String, u64> {
        rows.iter()
            .map(|(row, calls)| ((*row).to_string(), *calls))
            .collect()
    }

    /// 第七节 ② 第 3 条那两份 E203 产物（`git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-{base,files_10000}-2026-11-07.out`）同一次挂载覆盖写第 1、2 轮喂给 B8 的输入，caller 段全 0。
    fn e203_anchor_inputs() -> Vec<(u64, [ByDeltaInput; 2])> {
        vec![
            (
                2102,
                [
                    ByDeltaInput {
                        inode_leaf_containers: 1,
                        other_files: 11,
                        touched_files: 0,
                        lower_nodes_of_touched_files: 0,
                        extent_upper_levels: 1,
                        extent_upper_nodes: 1,
                        accounting_rows_of_the_generation: 15,
                        accounting_tree_levels: 1,
                        settling_rounds: 0,
                        changed_leaves_per_round: vec![],
                        allocation_tree_levels: 3,
                        allocation_records: 922,
                        segments_per_disk: 2048,
                        mapped_units: 35,
                        deadlist_entries: 0,
                        calls: called(&[
                            ("files_other_than_the_first_plan", 1),
                            ("inodes_with_an_extent_entry_set", 1),
                            ("extent_upper_segment_plan", 1),
                            ("extent_roles_rewritten_scan", 1),
                            ("root_header_linear_find", 3),
                            ("checkpoint_cost_accounting_tree_plan", 2),
                            ("cluster_segment_sum", 2),
                            ("inode_number_watermark_steps", 1),
                        ]),
                        intersecting_records: 0,
                        dirty_per_call: vec![],
                        code_two_trees_over_all_rounds: vec![],
                        container_rewrites_over_all_rounds: vec![],
                        moved_entries: 0,
                        central_mapping_levels: 1,
                        deadlist_query_levels: 0,
                        deadlist_changed_entries: 0,
                        release_query_levels: 0,
                        changed_slots_and_records_per_round: vec![],
                        mapped_units_changed_over_all_rounds: 0,
                    },
                    ByDeltaInput {
                        inode_leaf_containers: 1,
                        other_files: 11,
                        touched_files: 0,
                        lower_nodes_of_touched_files: 0,
                        extent_upper_levels: 1,
                        extent_upper_nodes: 1,
                        accounting_rows_of_the_generation: 15,
                        accounting_tree_levels: 1,
                        settling_rounds: 2,
                        changed_leaves_per_round: vec![4, 4],
                        allocation_tree_levels: 3,
                        allocation_records: 922,
                        segments_per_disk: 2048,
                        mapped_units: 35,
                        deadlist_entries: 0,
                        calls: called(&[
                            ("files_other_than_the_first_plan", 2),
                            ("inodes_with_an_extent_entry_set", 2),
                            ("extent_upper_segment_plan", 2),
                            ("extent_roles_rewritten_scan", 2),
                            ("root_header_linear_find", 3),
                            ("checkpoint_cost_accounting_tree_plan", 2),
                            ("inode_number_watermark_steps", 3),
                            ("allocator_release_linear_find", 84),
                            ("user_data_slot_linear_scan", 2),
                            ("record_intersection_filter", 28),
                            ("accounting_entries_of_this_publish", 2),
                        ]),
                        intersecting_records: 2,
                        dirty_per_call: vec![28, 56, 86],
                        code_two_trees_over_all_rounds: vec![
                            (30, 1),
                            (0, 0),
                            (10, 1),
                            (30, 1),
                            (0, 0),
                            (24, 1),
                        ],
                        container_rewrites_over_all_rounds: vec![0, 0, 0, 0, 0, 0],
                        moved_entries: 0,
                        central_mapping_levels: 1,
                        deadlist_query_levels: 48,
                        deadlist_changed_entries: 0,
                        release_query_levels: 114,
                        changed_slots_and_records_per_round: vec![14, 28],
                        mapped_units_changed_over_all_rounds: 42,
                    },
                ],
            ),
            (
                2098,
                [
                    ByDeltaInput {
                        inode_leaf_containers: 1,
                        other_files: 11,
                        touched_files: 0,
                        lower_nodes_of_touched_files: 0,
                        extent_upper_levels: 1,
                        extent_upper_nodes: 1,
                        accounting_rows_of_the_generation: 15,
                        accounting_tree_levels: 1,
                        settling_rounds: 0,
                        changed_leaves_per_round: vec![],
                        allocation_tree_levels: 3,
                        allocation_records: 948,
                        segments_per_disk: 2048,
                        mapped_units: 35,
                        deadlist_entries: 0,
                        calls: called(&[
                            ("files_other_than_the_first_plan", 1),
                            ("inodes_with_an_extent_entry_set", 1),
                            ("extent_upper_segment_plan", 1),
                            ("extent_roles_rewritten_scan", 1),
                            ("root_header_linear_find", 3),
                            ("checkpoint_cost_accounting_tree_plan", 2),
                            ("cluster_segment_sum", 2),
                            ("inode_number_watermark_steps", 1),
                        ]),
                        intersecting_records: 0,
                        dirty_per_call: vec![],
                        code_two_trees_over_all_rounds: vec![],
                        container_rewrites_over_all_rounds: vec![],
                        moved_entries: 0,
                        central_mapping_levels: 1,
                        deadlist_query_levels: 0,
                        deadlist_changed_entries: 0,
                        release_query_levels: 0,
                        changed_slots_and_records_per_round: vec![],
                        mapped_units_changed_over_all_rounds: 0,
                    },
                    ByDeltaInput {
                        inode_leaf_containers: 1,
                        other_files: 11,
                        touched_files: 0,
                        lower_nodes_of_touched_files: 0,
                        extent_upper_levels: 1,
                        extent_upper_nodes: 1,
                        accounting_rows_of_the_generation: 15,
                        accounting_tree_levels: 1,
                        settling_rounds: 2,
                        changed_leaves_per_round: vec![4, 4],
                        allocation_tree_levels: 3,
                        allocation_records: 948,
                        segments_per_disk: 2048,
                        mapped_units: 35,
                        deadlist_entries: 0,
                        calls: called(&[
                            ("files_other_than_the_first_plan", 2),
                            ("inodes_with_an_extent_entry_set", 2),
                            ("extent_upper_segment_plan", 2),
                            ("extent_roles_rewritten_scan", 2),
                            ("root_header_linear_find", 3),
                            ("checkpoint_cost_accounting_tree_plan", 2),
                            ("inode_number_watermark_steps", 3),
                            ("allocator_release_linear_find", 84),
                            ("user_data_slot_linear_scan", 2),
                            ("record_intersection_filter", 28),
                            ("accounting_entries_of_this_publish", 2),
                        ]),
                        intersecting_records: 0,
                        dirty_per_call: vec![28, 56, 84],
                        code_two_trees_over_all_rounds: vec![
                            (30, 1),
                            (0, 0),
                            (10, 1),
                            (30, 1),
                            (0, 0),
                            (24, 1),
                        ],
                        container_rewrites_over_all_rounds: vec![0, 0, 0, 0, 0, 0],
                        moved_entries: 0,
                        central_mapping_levels: 1,
                        deadlist_query_levels: 48,
                        deadlist_changed_entries: 0,
                        release_query_levels: 114,
                        changed_slots_and_records_per_round: vec![14, 28],
                        mapped_units_changed_over_all_rounds: 42,
                    },
                ],
            ),
            (
                152344,
                [
                    ByDeltaInput {
                        inode_leaf_containers: 43,
                        other_files: 10001,
                        touched_files: 0,
                        lower_nodes_of_touched_files: 0,
                        extent_upper_levels: 2,
                        extent_upper_nodes: 71,
                        accounting_rows_of_the_generation: 15,
                        accounting_tree_levels: 1,
                        settling_rounds: 0,
                        changed_leaves_per_round: vec![],
                        allocation_tree_levels: 3,
                        allocation_records: 47624,
                        segments_per_disk: 2048,
                        mapped_units: 10221,
                        deadlist_entries: 0,
                        calls: called(&[
                            ("files_other_than_the_first_plan", 1),
                            ("inodes_with_an_extent_entry_set", 1),
                            ("extent_upper_segment_plan", 1),
                            ("extent_roles_rewritten_scan", 1),
                            ("root_header_linear_find", 3),
                            ("checkpoint_cost_accounting_tree_plan", 2),
                            ("cluster_segment_sum", 2),
                            ("inode_number_watermark_steps", 1),
                        ]),
                        intersecting_records: 0,
                        dirty_per_call: vec![],
                        code_two_trees_over_all_rounds: vec![],
                        container_rewrites_over_all_rounds: vec![],
                        moved_entries: 0,
                        central_mapping_levels: 2,
                        deadlist_query_levels: 0,
                        deadlist_changed_entries: 0,
                        release_query_levels: 0,
                        changed_slots_and_records_per_round: vec![],
                        mapped_units_changed_over_all_rounds: 0,
                    },
                    ByDeltaInput {
                        inode_leaf_containers: 43,
                        other_files: 10001,
                        touched_files: 0,
                        lower_nodes_of_touched_files: 0,
                        extent_upper_levels: 2,
                        extent_upper_nodes: 71,
                        accounting_rows_of_the_generation: 15,
                        accounting_tree_levels: 1,
                        settling_rounds: 22,
                        changed_leaves_per_round: vec![
                            10, 14, 18, 22, 26, 28, 30, 32, 36, 40, 44, 48, 50, 52, 56, 60, 62, 64,
                            66, 68, 70, 70,
                        ],
                        allocation_tree_levels: 3,
                        allocation_records: 47624,
                        segments_per_disk: 2048,
                        mapped_units: 10221,
                        deadlist_entries: 0,
                        calls: called(&[
                            ("files_other_than_the_first_plan", 22),
                            ("inodes_with_an_extent_entry_set", 22),
                            ("extent_upper_segment_plan", 22),
                            ("extent_roles_rewritten_scan", 22),
                            ("root_header_linear_find", 3),
                            ("checkpoint_cost_accounting_tree_plan", 2),
                            ("inode_number_watermark_steps", 23),
                            ("allocator_release_linear_find", 4804),
                            ("user_data_slot_linear_scan", 2),
                            ("record_intersection_filter", 168),
                            ("empty_segment_linear_scan", 4),
                            ("accounting_entries_of_this_publish", 22),
                        ]),
                        intersecting_records: 152,
                        dirty_per_call: vec![
                            44, 96, 112, 128, 144, 160, 168, 176, 184, 200, 216, 232, 248, 256,
                            264, 280, 296, 304, 312, 320, 328, 336, 656,
                        ],
                        code_two_trees_over_all_rounds: vec![
                            (30, 1),
                            (0, 0),
                            (12, 2),
                            (30, 1),
                            (0, 0),
                            (38, 2),
                            (30, 1),
                            (0, 0),
                            (46, 2),
                            (30, 1),
                            (0, 0),
                            (54, 2),
                            (30, 1),
                            (0, 0),
                            (62, 2),
                            (30, 1),
                            (0, 0),
                            (70, 2),
                            (30, 1),
                            (0, 0),
                            (74, 2),
                            (30, 1),
                            (0, 0),
                            (78, 2),
                            (30, 1),
                            (0, 0),
                            (82, 2),
                            (30, 1),
                            (0, 0),
                            (90, 2),
                            (30, 1),
                            (0, 0),
                            (98, 2),
                            (30, 1),
                            (0, 0),
                            (106, 2),
                            (30, 1),
                            (0, 0),
                            (114, 2),
                            (30, 1),
                            (0, 0),
                            (118, 2),
                            (30, 1),
                            (0, 0),
                            (122, 2),
                            (30, 1),
                            (0, 0),
                            (130, 2),
                            (30, 1),
                            (0, 0),
                            (138, 2),
                            (30, 1),
                            (0, 0),
                            (142, 2),
                            (30, 1),
                            (0, 0),
                            (146, 2),
                            (30, 1),
                            (0, 0),
                            (150, 2),
                            (30, 1),
                            (0, 0),
                            (154, 2),
                            (30, 1),
                            (0, 0),
                            (158, 2),
                        ],
                        container_rewrites_over_all_rounds: vec![
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        ],
                        moved_entries: 0,
                        central_mapping_levels: 2,
                        deadlist_query_levels: 1540,
                        deadlist_changed_entries: 0,
                        release_query_levels: 15890,
                        changed_slots_and_records_per_round: vec![
                            22, 48, 56, 64, 72, 80, 84, 88, 92, 100, 108, 116, 124, 128, 132, 140,
                            148, 152, 156, 160, 164, 168,
                        ],
                        mapped_units_changed_over_all_rounds: 2402,
                    },
                ],
            ),
            (
                4226,
                [
                    ByDeltaInput {
                        inode_leaf_containers: 43,
                        other_files: 10001,
                        touched_files: 0,
                        lower_nodes_of_touched_files: 0,
                        extent_upper_levels: 2,
                        extent_upper_nodes: 71,
                        accounting_rows_of_the_generation: 15,
                        accounting_tree_levels: 1,
                        settling_rounds: 0,
                        changed_leaves_per_round: vec![],
                        allocation_tree_levels: 3,
                        allocation_records: 47640,
                        segments_per_disk: 2048,
                        mapped_units: 10221,
                        deadlist_entries: 0,
                        calls: called(&[
                            ("files_other_than_the_first_plan", 1),
                            ("inodes_with_an_extent_entry_set", 1),
                            ("extent_upper_segment_plan", 1),
                            ("extent_roles_rewritten_scan", 1),
                            ("root_header_linear_find", 3),
                            ("checkpoint_cost_accounting_tree_plan", 2),
                            ("cluster_segment_sum", 2),
                            ("inode_number_watermark_steps", 1),
                        ]),
                        intersecting_records: 0,
                        dirty_per_call: vec![],
                        code_two_trees_over_all_rounds: vec![],
                        container_rewrites_over_all_rounds: vec![],
                        moved_entries: 0,
                        central_mapping_levels: 2,
                        deadlist_query_levels: 0,
                        deadlist_changed_entries: 0,
                        release_query_levels: 0,
                        changed_slots_and_records_per_round: vec![],
                        mapped_units_changed_over_all_rounds: 0,
                    },
                    ByDeltaInput {
                        inode_leaf_containers: 43,
                        other_files: 10001,
                        touched_files: 0,
                        lower_nodes_of_touched_files: 0,
                        extent_upper_levels: 2,
                        extent_upper_nodes: 71,
                        accounting_rows_of_the_generation: 15,
                        accounting_tree_levels: 1,
                        settling_rounds: 2,
                        changed_leaves_per_round: vec![4, 4],
                        allocation_tree_levels: 3,
                        allocation_records: 47640,
                        segments_per_disk: 2048,
                        mapped_units: 10221,
                        deadlist_entries: 0,
                        calls: called(&[
                            ("files_other_than_the_first_plan", 2),
                            ("inodes_with_an_extent_entry_set", 2),
                            ("extent_upper_segment_plan", 2),
                            ("extent_roles_rewritten_scan", 2),
                            ("root_header_linear_find", 3),
                            ("checkpoint_cost_accounting_tree_plan", 2),
                            ("inode_number_watermark_steps", 3),
                            ("allocator_release_linear_find", 108),
                            ("user_data_slot_linear_scan", 2),
                            ("record_intersection_filter", 34),
                            ("accounting_entries_of_this_publish", 2),
                        ]),
                        intersecting_records: 32,
                        dirty_per_call: vec![40, 68, 134],
                        code_two_trees_over_all_rounds: vec![
                            (30, 1),
                            (0, 0),
                            (12, 2),
                            (30, 1),
                            (0, 0),
                            (26, 2),
                        ],
                        container_rewrites_over_all_rounds: vec![0, 0, 0, 0, 0, 0],
                        moved_entries: 0,
                        central_mapping_levels: 2,
                        deadlist_query_levels: 140,
                        deadlist_changed_entries: 0,
                        release_query_levels: 322,
                        changed_slots_and_records_per_round: vec![20, 34],
                        mapped_units_changed_over_all_rounds: 54,
                    },
                ],
            ),
        ]
    }

    fn rounded(value: f64) -> f64 {
        (value * 1000.0).round() / 1000.0
    }

    #[test]
    fn chain_of_one_settles_in_two_rounds_on_both_arms() {
        for arm in [ModelArm::Today, ModelArm::Closure] {
            let outcome = chain_outcome(1, 3, arm);
            assert_eq!(outcome.rounds, Some(2));
            assert_eq!(outcome.round_work(), 7);
            assert_eq!(outcome.per_round, vec![3, 4]);
        }
        assert_eq!(chain_outcome(1, 3, ModelArm::Today).sum_of_increments(), 4);
    }

    #[test]
    fn chain_of_five_takes_six_rounds_today_and_two_with_the_closure() {
        let today = chain_outcome(5, 3, ModelArm::Today);
        assert_eq!(today.rounds, Some(6));
        assert_eq!(today.round_work(), 33);
        assert_eq!(today.per_round, vec![3, 4, 5, 6, 7, 8]);
        assert_eq!(today.sum_of_increments(), 8);
        let closure = chain_outcome(5, 3, ModelArm::Closure);
        assert_eq!(closure.rounds, Some(2));
        assert_eq!(closure.round_work(), 11);
    }

    #[test]
    fn chain_of_twenty_takes_twenty_one_rounds_today_and_two_with_the_closure() {
        let today = chain_outcome(20, 3, ModelArm::Today);
        assert_eq!(today.rounds, Some(21));
        assert_eq!(today.round_work(), 273);
        assert_eq!(today.sum_of_increments(), 23);
        assert_eq!(rounded(today.effective_rounds()), rounded(273.0 / 23.0));
        let closure = chain_outcome(20, 3, ModelArm::Closure);
        assert_eq!(closure.rounds, Some(2));
        assert_eq!(closure.round_work(), 26);
        assert_eq!(closure.per_round, vec![3, 23]);
    }

    #[test]
    fn chain_of_seventy_runs_out_of_rounds_today_and_settles_with_the_closure() {
        let today = chain_outcome(70, 3, ModelArm::Today);
        assert_eq!(today.rounds, None);
        assert_eq!(today.per_round.len(), 64);
        assert_eq!(today.round_work(), 2208);
        let closure = chain_outcome(70, 3, ModelArm::Closure);
        assert_eq!(closure.rounds, Some(2));
        assert_eq!(closure.round_work(), 76);
        assert_eq!(closure.per_round, vec![3, 73]);
    }

    #[test]
    fn crossing_leaves_keep_one_more_round_with_the_closure() {
        let today = crossing_outcome(ModelArm::Today);
        assert_eq!(today.rounds, Some(6));
        assert_eq!(today.round_work(), 27);
        assert_eq!(today.per_round, vec![2, 3, 4, 5, 6, 7]);
        let closure = crossing_outcome(ModelArm::Closure);
        assert_eq!(closure.rounds, Some(3));
        assert_eq!(closure.round_work(), 14);
        assert_eq!(closure.per_round, vec![2, 5, 7]);
    }

    #[test]
    fn geometry_table_matches_the_registered_anchors() {
        let expected = [
            (14, 22, 45),
            (17, 164, 331),
            (18, 325, 655),
            (19, 648, 1305),
            (20, 1294, 2605),
            (24, 20664, 41577),
        ];
        for (exponent, leaves, nodes) in expected {
            let (got_leaves, _, got_nodes) = tree_geometry(1_u64 << exponent);
            assert_eq!((got_leaves, got_nodes), (leaves, nodes), "U = 2^{exponent}");
        }
    }

    #[test]
    fn local_constants_match_the_registered_values() {
        assert_eq!(LEAF_SLOTS, 812);
        assert_eq!(INTERNAL_FANOUT, 169);
        assert_eq!(SETTLING_ROUNDS_LIMIT, 64);
        assert_eq!(DISKS, 2);
    }

    #[test]
    fn growth_orders_match_the_registered_anchors() {
        assert_eq!(rounded(alpha_on_the_axis(2.0, 22.0)), 0.347);
        assert_eq!(rounded(alpha_on_the_axis(2.0, 64.0)), 0.502);
        assert_eq!(rounded(alpha_on_the_axis(21.0, 1201.0)), 0.586);
        assert_eq!(rounded(alpha_on_the_axis(14.0, 84.0)), 0.259);
    }

    #[test]
    fn bands_split_at_a_quarter_and_a_half() {
        assert_eq!(band(0.25), Band::DoesNotGrow);
        assert_eq!(band(0.26), Band::Between);
        assert_eq!(band(0.5), Band::Grows);
        assert_eq!(band(alpha_on_the_axis(21.0, 200.0)), Band::Between);
        assert_eq!(band(alpha_on_the_axis(21.0, 118.0)), Band::DoesNotGrow);
        assert_eq!(band(alpha_on_the_axis(21.0, 664.1)), Band::Grows);
    }

    #[test]
    fn rounds_verdict_has_three_levels() {
        assert_eq!(rounds_verdict(8, false), "not_many");
        assert_eq!(rounds_verdict(9, false), "many");
        assert_eq!(rounds_verdict(63, false), "many");
        assert_eq!(rounds_verdict(64, false), "hits_the_limit");
        assert_eq!(rounds_verdict(5, true), "hits_the_limit");
    }

    /// 第八节判别力自证：门槛落在两点之间，判定翻面。
    #[test]
    fn the_verdicts_flip_between_the_chain_of_five_and_the_chain_of_twenty() {
        let (short, long) = (
            chain_outcome(5, 3, ModelArm::Today),
            chain_outcome(20, 3, ModelArm::Today),
        );
        assert_eq!(
            rounds_verdict(short.rounds.expect("定下"), false),
            "not_many"
        );
        assert_eq!(rounds_verdict(long.rounds.expect("定下"), false), "many");
        assert_eq!(
            rounds_held_down_verdict(
                short.rounds.expect("定下"),
                short.effective_rounds().min(4.0)
            ),
            "held_down"
        );
        assert_eq!(
            rounds_held_down_verdict(long.rounds.expect("定下"), long.effective_rounds()),
            "not_held_down"
        );
        assert_eq!(
            band(alpha_on_the_axis(
                short.round_work() as f64,
                long.round_work() as f64
            )),
            Band::Between
        );
        assert_eq!(
            band(alpha_on_the_axis(short.round_work() as f64, 1000.0)),
            Band::Between
        );
        assert_eq!(
            band(alpha_on_the_axis(
                short.round_work() as f64,
                short.round_work() as f64 * 2.0
            )),
            Band::DoesNotGrow
        );
    }

    #[test]
    fn the_many_rounds_threshold_is_not_the_arm_definition() {
        assert_eq!(rounds_held_down_verdict(8, 4.0), "held_down");
        assert_eq!(rounds_held_down_verdict(9, 1.0), "not_held_down");
        assert_eq!(rounds_held_down_verdict(2, 4.01), "not_held_down");
    }

    #[test]
    fn local_by_delta_rows_reproduce_the_four_e203_totals() {
        for (expected, [landing, between]) in e203_anchor_inputs() {
            assert_eq!(
                by_delta_total(&landing) + by_delta_total(&between),
                expected
            );
        }
    }

    #[test]
    fn between_rounds_pricing_on_the_cumulative_inputs_equals_the_by_delta_rows() {
        for (_, [_, between]) in e203_anchor_inputs() {
            let steps = between_rounds_pricing_round_steps(&between_rounds_pricing_cumulative_input(&between), between_rounds_unit_prices(&between));
            assert_eq!(steps, by_delta_rows_that_between_rounds_pricing_changes(&between));
        }
    }

    fn fabricated_round(exit: &str, roles: u64, records: u64) -> Round {
        Round {
            round: 1,
            roles_rewritten: roles,
            exit: exit.to_string(),
            roles_changed: roles,
            records_changed: records,
            ..Round::default()
        }
    }

    #[test]
    fn the_premise_ratio_counts_the_release_rounds_too() {
        let settle = Settle {
            round_work: 10,
            detail: vec![
                fabricated_round("release_not_covered", 4, 4),
                fabricated_round("release_not_covered", 3, 2),
                fabricated_round("settled", 3, 2),
            ],
            ..Settle::default()
        };
        assert_eq!(rounded(premise_ratio(&settle)), 0.8);
    }

    #[test]
    fn between_rounds_pricing_reading_sums_the_rounds_and_keeps_the_largest() {
        let prices = ByDeltaInput {
            allocation_tree_levels: 2,
            allocation_records: 7,
            mapped_units: 3,
            segments_per_disk: 64,
            code_two_trees_over_all_rounds: vec![(1, 2), (1, 3), (1, 4)],
            ..ByDeltaInput::default()
        };
        let settle = Settle {
            detail: vec![
                Round {
                    tree_keys_changed: vec![1, 0, 0],
                    rewrite_set_size: 2,
                    ..fabricated_round("release_not_covered", 2, 1)
                },
                Round {
                    tree_keys_changed: vec![0, 0, 2],
                    rewrite_set_size: 5,
                    ..fabricated_round("settled", 3, 2)
                },
            ],
            ..Settle::default()
        };
        // 第 1 轮：树 2 × 1；mapped 4 + 4 × 2；view 1 + find 3 + 交集 3 + dirty 1 = 22
        // 第 2 轮：树 4 × 2；mapped 6 + 6 × 2；view 2 + find 6 + 交集 6 + dirty 2 = 42
        let reading = between_rounds_pricing_reading(&settle, &prices, 100);
        assert_eq!(reading.sum_of_rounds, 22 + 42);
        assert_eq!(reading.largest_round, 42);
        assert_eq!(reading.memory_bytes, 3 * 108 + 5 * 24 + 3 * 24);
        assert_eq!(reading.maintenance_steps, 3 * 3 + 5 * 2);
    }

    #[test]
    fn fallback_and_recurrence_read_the_sequence() {
        assert_eq!(fallback_and_recurrence(&[2, 22, 2, 2]), (true, false));
        assert_eq!(
            fallback_and_recurrence(&[2, 22, 9, 9, 9, 9, 9, 9, 9, 9, 9]),
            (false, false)
        );
        assert_eq!(
            fallback_and_recurrence(&[22, 2, 2, 2, 2, 2, 2, 2, 2, 9]),
            (true, true)
        );
    }

    #[test]
    fn chain_explained_ratio_only_reads_the_long_fixed_points() {
        let settle = |rounds, release_chain_depth| Settle {
            rounds,
            release_chain_depth,
            ..Settle::default()
        };
        assert_eq!(chain_explained_ratio(&[settle(2, 0)]), None);
        assert_eq!(
            chain_explained_ratio(&[settle(12, 10), settle(12, 7)]),
            Some(0.5)
        );
    }

    fn sequence_settle(hash: &str, rounds: u64, final_roles: u64, final_nodes: u64) -> Settle {
        Settle {
            rounds,
            final_roles,
            final_nodes,
            round_work: rounds * final_roles,
            final_set_hash: hash.to_string(),
            outcome: "settled".to_string(),
            ..Settle::default()
        }
    }

    #[test]
    fn same_final_sets_report_the_first_difference() {
        let (one, two, other) = (
            sequence_settle("a", 2, 3, 1),
            sequence_settle("b", 2, 3, 1),
            sequence_settle("c", 2, 3, 1),
        );
        assert_eq!(same_final_sets(&[&one, &two], &[&one, &two]), (2, 2, None));
        assert_eq!(
            same_final_sets(&[&one, &two], &[&one, &other]),
            (1, 2, Some(1))
        );
        assert_eq!(same_final_sets(&[&one], &[&one, &other]), (1, 2, Some(1)));
    }

    #[test]
    fn same_readings_needs_every_field_equal() {
        let today = sequence_settle("a", 2, 3, 1);
        let more_rounds = sequence_settle("a", 3, 3, 1);
        assert!(same_readings(&[&today], &[&today]));
        assert!(!same_readings(&[&today], &[&more_rounds]));
        assert!(!same_readings(&[&today], &[&today, &today]));
    }

    #[test]
    fn single_step_control_needs_the_same_set_enough_rounds_and_more_rounds() {
        let today = sequence_settle("a", 2, 5, 2);
        let single = sequence_settle("a", 4, 5, 2);
        let window_today = vec![today.clone()];
        let window_single = vec![single.clone()];
        assert!(single_step_control_holds(
            &[&single],
            &[&today],
            &window_single,
            &window_today
        ));
        assert!(
            !single_step_control_holds(
                &[&sequence_settle("b", 4, 5, 2)],
                &[&today],
                &window_single,
                &window_today
            ),
            "不动点变了"
        );
        assert!(
            !single_step_control_holds(
                &[&sequence_settle("a", 1, 5, 2)],
                &[&today],
                &window_single,
                &window_today
            ),
            "轮数少于节点数"
        );
        assert!(
            !single_step_control_holds(&[&single], &[&today], &window_today, &window_today),
            "量窗一次都没多轮"
        );
    }

    fn lines_of(text: &str) -> PointArmReadings {
        let mut item = PointArmReadings::default();
        for line in text.lines() {
            take_line(&mut item, &fields_of(line));
        }
        item
    }

    const COMPLETE_PRODUCT: &str = "E206RESULT name=point item=base__p1 point=base arm=p1 unit_area_slots=131072 unlinks=10 data_units=10 files=10\n\
E206RESULT name=constants item=base__p1 leaf_slots=812 internal_fanout=169 settling_rounds_at_most=64 disks=2\n\
E206RESULT name=history_histogram item=base__p1 bucket=0 calls=2 rounds_one=0 rounds_two=1 rounds_three_or_more=1 sequence=2:7:2:21:00000000000000aa:40:s;3:7:2:21:00000000000000ab:40:s\n\
E206RESULT name=history_settle item=base__p1 phase=history rounds=3\n\
E206RESULT name=history item=base__p1 settle_calls=2\n\
E206RESULT name=settle item=base__p1 phase=window state=same_mount op=W1 op_index=1 rounds=2 outcome=settled final_roles=14 final_nodes=4 round_work=21 final_set_hash=00000000000000ac judgement=0\n\
E206RESULT name=round item=base__p1 state=same_mount op=W1 op_index=1 round=1 roles_rewritten=7 nodes_rewritten=0 round_exit=release_not_covered\n\
E206RESULT name=round_row item=base__p1 round=1 row=records_of_each_leaf calls=2 steps=30 query_levels=1\n\
E206RESULT name=round item=base__p1 state=same_mount op=W1 op_index=1 round=2 roles_rewritten=14 nodes_rewritten=4 round_exit=settled\n\
E206RESULT name=item_done item=base__p1\n";

    #[test]
    fn complete_product_reads_back_into_every_group() {
        let item = lines_of(COMPLETE_PRODUCT);
        assert_eq!(item_is_complete(&item), Ok(()));
        assert_eq!(item.history.len(), 2);
        assert_eq!(item.history[1].rounds, 3);
        assert_eq!(item.history[1].final_set_hash, "00000000000000ab");
        assert_eq!(item.window.len(), 1);
        assert_eq!(item.window[0].publish_key(), "same_mount/W1/1/0");
        assert_eq!(
            item.window[0].detail[0].rows["records_of_each_leaf"],
            (2, 30, 1)
        );
        assert_eq!(item.window[0].fixed_point_steps(), 30);
        assert_eq!(peaks_of(all_settles(&item)).rounds, 3);
        assert!(model_mismatches(&item).is_empty());
    }

    #[test]
    fn product_without_item_done_is_void_and_not_zero() {
        let item =
            lines_of(&COMPLETE_PRODUCT.replace("E206RESULT name=item_done item=base__p1\n", ""));
        assert_eq!(item_is_complete(&item), Err("no_item_done".to_string()));
    }

    #[test]
    fn histogram_short_of_the_fixed_point_calls_is_void() {
        let item = lines_of(&COMPLETE_PRODUCT.replace("settle_calls=2", "settle_calls=3"));
        assert!(item_is_complete(&item).is_err());
    }

    #[test]
    fn single_history_line_missing_is_void() {
        let item = lines_of(&COMPLETE_PRODUCT.replace(
            "E206RESULT name=history_settle item=base__p1 phase=history rounds=3\n",
            "",
        ));
        assert!(item_is_complete(&item).is_err());
    }

    #[test]
    fn round_missing_from_the_window_is_void() {
        let item = lines_of(
            &COMPLETE_PRODUCT
                .replace(
                    "round=2 roles_rewritten=14 nodes_rewritten=4 round_exit=settled",
                    "round=2",
                )
                .replace("rounds=2 outcome", "rounds=3 outcome"),
        );
        assert!(item_is_complete(&item).is_err());
    }

    #[test]
    fn round_work_off_the_sum_of_the_rounds_stops_the_run() {
        let item = lines_of(
            &COMPLETE_PRODUCT.replace("round_work=21 final_set", "round_work=22 final_set"),
        );
        assert_eq!(
            model_mismatches(&item),
            vec!["round_work=22_sum=21".to_string()]
        );
    }

    #[test]
    fn constants_off_the_local_ones_stop_the_run() {
        let item = lines_of(&COMPLETE_PRODUCT.replace("leaf_slots=812", "leaf_slots=813"));
        assert_eq!(model_mismatches(&item).len(), 1);
    }

    #[test]
    fn refused_history_keeps_its_cause_and_the_did_not_settle_outcome_counts_as_the_limit() {
        let item = lines_of(
            "E206RESULT name=point item=x__p1 point=x arm=p1\n\
E206RESULT name=history_histogram item=x__p1 bucket=0 calls=1 sequence=64:0:0:2208:0000000000000000:0:d\n\
E206RESULT name=history_refused item=x__p1 cause=AllocationRecordTreeRewriteSetDidNotSettle\n",
        );
        assert_eq!(
            item_is_complete(&item),
            Err("refused:AllocationRecordTreeRewriteSetDidNotSettle".to_string())
        );
        let peaks = peaks_of(all_settles(&item));
        assert_eq!(
            rounds_verdict(peaks.rounds, peaks.did_not_settle),
            "hits_the_limit"
        );
    }

    #[test]
    fn write_amplification_splits_at_one_and_a_quarter() {
        assert_eq!(write_amplification_verdict(1.25), "acceptable");
        assert_eq!(write_amplification_verdict(1.26), "expensive");
    }

    #[test]
    fn net_steps_add_the_closure_to_the_arm_and_take_off_the_base() {
        let with_steps = |steps: u64| Settle {
            detail: vec![Round {
                rows: BTreeMap::from([("records_of_each_leaf".to_string(), (1, steps, 0))]),
                ..Round::default()
            }],
            ..Settle::default()
        };
        let arm = Settle {
            closure_lookup_steps: 7,
            closure_hops: 3,
            previous_nodes: 15,
            ..with_steps(100)
        };
        let base = with_steps(120);
        assert_eq!(net_steps_linear(&arm, &base), 100 + 7 - 120);
        assert_eq!(net_steps_indexed(&arm, &base), 100 + 3 * 4 - 120);
    }

    #[test]
    fn negative_control_needs_flat_roles_and_few_rounds() {
        let settle = |final_roles, rounds| Settle {
            final_roles,
            rounds,
            ..Settle::default()
        };
        assert!(negative_control_holds(&settle(14, 2), &settle(14, 4)));
        assert!(!negative_control_holds(&settle(14, 2), &settle(28, 2)));
        assert!(!negative_control_holds(&settle(14, 2), &settle(14, 5)));
    }

    #[test]
    fn the_pooled_fallback_adds_no_by_delta_steps_and_is_counted_by_the_judgements_that_call_it() {
        let judgement = |calls: u64, steps: u64| {
            let mut judgement = JudgementReadings {
                fixed_point_rounds: Some(0),
                ..JudgementReadings::default()
            };
            judgement.rows.insert(("between".to_string(), ROW_WITHOUT_A_BY_DELTA_FORMULA.to_string()), (calls, 0));
            judgement.row_steps.insert(("between".to_string(), ROW_WITHOUT_A_BY_DELTA_FORMULA.to_string()), steps);
            judgement.row_steps.insert(("caller".to_string(), "fallback_slot_linear_scan".to_string()), 4);
            judgement
        };
        assert_eq!(pooled_fallback_of_the_judgement(&judgement(0, 0)), (0, 0, 4));
        assert_eq!(pooled_fallback_of_the_judgement(&judgement(3, 500)), (3, 500, 4));
        // B8 的输入里读得到 pooled_fallback_calls，式子里没有它：合计一步不多
        let input = |calls: u64| {
            let mut read = judgement(calls, 500);
            read.state = fields_of("E206RESULT name=state inode_leaf_containers=1 other_files=0 allocation_records=1000 mapped_units=10 deadlist_entries=0 extent_upper_nodes=0 extent_upper_levels=0 accounting_tree_levels=1 deadlist_tree_levels=1 allocation_tree_levels=1 central_mapping_levels=1");
            by_delta_of_the_judgement(&read, 1 << 17)
        };
        assert_eq!(input(0), input(3), "s37 没有 B8 式子");
        let mut item = PointArmReadings::default();
        item.judgements.insert("same_mount/W1/1/".to_string(), JudgementReadings::default());
        item.judgements.insert("same_mount/W1/2/".to_string(), JudgementReadings::default());
        item.judgements.insert("same_mount/W1/1/1".to_string(), judgement(0, 0));
        item.judgements.insert("same_mount/W1/2/1".to_string(), judgement(3, 500));
        item.window = vec![
            Settle { state: "same_mount".to_string(), operation: "W1".to_string(), operation_index: 1, judgement: 1, ..Settle::default() },
            Settle { state: "same_mount".to_string(), operation: "W1".to_string(), operation_index: 2, judgement: 1, ..Settle::default() },
        ];
        assert_eq!(pooled_fallback_of_the_window(&item), (2, 1, 3, 500, 8));
    }

    #[test]
    fn an_unread_state_is_a_gap_and_not_zero() {
        let judgement = JudgementReadings {
            fixed_point_rounds: Some(0),
            ..JudgementReadings::default()
        };
        let gaps = by_delta_input(&judgement, "between", 1 << 17).expect_err("规模读数读不到");
        assert!(gaps.contains(&"state.allocation_records".to_string()));
    }

    #[test]
    fn an_unread_round_is_a_gap_and_not_zero() {
        let judgement = JudgementReadings {
            fixed_point_rounds: Some(1),
            ..JudgementReadings::default()
        };
        let gaps = by_delta_input(&judgement, "between", 1 << 17).expect_err("逐轮读数读不到");
        assert!(gaps.contains(&"changed_leaves@round1".to_string()));
    }

    #[test]
    fn the_state_line_is_merged_into_the_judgement_of_a_settle() {
        let item = lines_of(
            "E206RESULT name=state phase=window state=same_mount op=W1 op_index=1 units=37\n\
E206RESULT name=publish publish=same_mount/W1/1/0 fixed_point_rounds=2\n\
E206RESULT name=settle state=same_mount op=W1 op_index=1 judgement=0 rounds=2\n",
        );
        let judgement = judgement_of(&item, &item.window[0]).expect("那一次判读得到");
        assert_eq!(number(&judgement.state, "units"), Some(37));
        assert_eq!(judgement.fixed_point_rounds, Some(2));
    }

    #[test]
    fn the_e203_reader_takes_only_the_asked_round_of_the_same_mount_overwrite() {
        let path =
            std::env::temp_dir().join(format!("e206-e203-reader-{}.out", std::process::id()));
        std::fs::write(
            &path,
            "E203RESULT name=point point=base unit_area_slots=131072\n\
E203RESULT name=publish state=same_mount kind=overwrite round=1 fixed_point_rounds=2\n\
E203RESULT name=publish state=same_mount kind=overwrite round=2 fixed_point_rounds=9\n\
E203RESULT name=publish state=remounted kind=overwrite round=1 fixed_point_rounds=4\n",
        )
        .expect("临时文件写得下");
        let (judgement, unit_area_slots) =
            e203_judgement(path.to_str().expect("路径是 UTF-8"), 2).expect("读得到");
        std::fs::remove_file(&path).expect("临时文件删得掉");
        assert_eq!(judgement.fixed_point_rounds, Some(9));
        assert_eq!(unit_area_slots, 131_072);
    }

    #[test]
    fn verdicts_agree_flips_when_one_point_changes_level() {
        let readings =
            |text: &[&str]| -> Vec<String> { text.iter().map(|one| (*one).to_string()).collect() };
        assert!(verdicts_agree(&readings(&["a:2:not_many", "b:3:not_many"])));
        assert!(!verdicts_agree(&readings(&["a:2:not_many", "b:22:many"])));
    }

    #[test]
    fn the_measured_anchor_needs_rounds_work_and_every_round() {
        let settle = Settle {
            rounds: 2,
            round_work: 21,
            detail: vec![
                fabricated_round("release_not_covered", 7, 0),
                fabricated_round("settled", 14, 0),
            ],
            ..Settle::default()
        };
        assert!(anchor_matches(&settle, 2, 21, &[7, 14]));
        assert!(!anchor_matches(&settle, 3, 21, &[7, 14]));
        assert!(!anchor_matches(&settle, 2, 22, &[7, 14]));
        assert!(!anchor_matches(&settle, 2, 21, &[7, 15]));
    }

    #[test]
    fn reconciliation_needs_the_two_counts_equal() {
        assert!(reconciliation_holds(14, 14));
        assert!(!reconciliation_holds(14, 15));
    }

    #[test]
    fn rounds_at_least_reads_the_history_and_the_window() {
        let item = lines_of(COMPLETE_PRODUCT);
        assert_eq!(rounds_at_least(&item, 3), 1);
        assert_eq!(rounds_at_least(&item, 2), 3);
    }

    #[test]
    fn fork_rows_need_every_point_of_the_first_segment() {
        let on = |arm: &str| -> BTreeSet<(String, String)> {
            FIRST_SEGMENT_POINTS
                .iter()
                .map(|point| ((*point).to_string(), arm.to_string()))
                .collect()
        };
        assert_eq!(
            fork_rows_with_enough_readings(&on("p1")),
            [("P1", true), ("P2", false), ("P3", true)]
        );
        let mut short = on("p2a");
        short.remove(&("base".to_string(), "p2a".to_string()));
        assert_eq!(
            fork_rows_with_enough_readings(&short),
            [("P1", false), ("P2", false), ("P3", false)]
        );
        assert_eq!(fork_rows_with_enough_readings(&on("p2a"))[1], ("P2", true));
    }

    #[test]
    fn exit_share_only_counts_the_long_fixed_points() {
        let settles = [
            Settle {
                rounds: 9,
                detail: vec![
                    fabricated_round("release_not_covered", 1, 1),
                    fabricated_round("placement_not_covered", 1, 1),
                    fabricated_round("placement_not_covered", 1, 1),
                    fabricated_round("settled", 1, 1),
                ],
                ..Settle::default()
            },
            Settle {
                rounds: 2,
                detail: vec![fabricated_round("release_not_covered", 1, 1)],
                ..Settle::default()
            },
        ];
        assert_eq!(exit_share(&settles, "placement_not_covered"), 0.5);
        assert_eq!(exit_share(&settles, "release_not_covered"), 0.25);
    }
}
