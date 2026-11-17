//! E207（B8严格判据下的步数）的推的那一路：纯算术，不 `use` 任何 `crates/` 里的东西（跑前登记
//! `research/prompts/e207-preregistration.md` 第五节「装置写在哪」第 3 条）。
//!
//! 做的事：读量的那一路打出的 `E207RESULT` 产物与 E206（发布固定点轮数的代价）的 p2a 产物（只读，回扫），按第一节读法 1–9
//! 算每次操作的 B8 步数（E203（准入之前扫类的步代价）登记第五节 B8 一表的本地式子）、外生的 `user_units`、量到的总量 x，
//! 逐格报 Q1–Q10、控制臂 `C1`、`C2`、`C3`、尖峰旗、节点年龄与首次合并、失败条款、作废条款与停机条款的判读。每个量各占一行，不合取。
//!
//! 用法：
//!   `e207-strict-scan-steps report --segment first|second|third|fourth|hundred_units_per_file_repeat|reconciliation_split <E207 产物>… [--retro <E206 p2a 产物>…] [--previous <第一段产物>…] [--pin-against <第一段产物>…]`
//!       读产物，每个量各占一行，末尾打 `name=verdict` 行；回扫只在给了 `--retro` 时做
//!   `e207-strict-scan-steps old-criterion-pins <E206 p2a 产物>…`  第七节 ② 第 1 条：照旧判据重算 `q8_first_publish` 的 files 轴与 `q8_peak` 的 data_units 轴两行
//! 退出码：0 读完；1 第七节 ② 第 1 条的两行与登记抄下的不逐字符相同；2 用法错或产物读不了。读不到的量报 `unread`，不当 0。
//! 单测钉第七节 ③ 的绝对值：`cargo test --release --bin e207-strict-scan-steps`。

#![allow(dead_code, reason = "模型的库函数一部分只有单测与报告的某一支用")]

use std::collections::{BTreeMap, BTreeSet};

/// 历史用例与紧凑盘不是这个 bin 的目标（它们拷进副本、当验证包的测试编，见入口 `e207-strict-scan-steps-run.sh`）；当文本读进来，让门禁 crate-sources-reachable 认它们有着落。
const _HISTORIES_PAYLOAD: &str = include_str!("e207_strict_scan_steps_histories.rs");
const _COMPACT_DEVICE_PAYLOAD: &str = include_str!("e207_strict_scan_steps_compact_device.rs");

/// 分配记录树一片叶罩多少槽（`crates/singlefs-format/src/lib.rs:163`）。
const LEAF_SLOTS: u64 = 812;
/// 内部节点扇出（`crates/singlefs-format/src/lib.rs:169`）。
const INTERNAL_FANOUT: u64 = 169;
/// 固定点上界（`crates/singlefs-core/src/transaction.rs:14111` 的 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`）。
const SETTLING_ROUNDS_LIMIT: u64 = 64;
/// 两块盘（第五节参数 `e142_parameters(512, 512)`）。
const DISKS: u64 = 2;
/// 单元区之前的槽数（6 MiB 环，`common_admission_pool` 同 E203）。
const UNIT_AREA_START_SLOT: u64 = 1408;
/// inode 叶容器每片记录数（E203 模型同一个常量）。
const INODE_LEAF_RECORDS: u64 = 233;
/// 每段槽数（E203 模型同一个常量）。
const SEGMENT_SLOTS: u64 = 64;
/// 读法第 8 条的尖峰旗：final_nodes ≥ max(28, 4 × 同点同状态量窗全部操作 final_nodes 的中位数)。
const SPIKE_FLOOR: u64 = 28;
const SPIKE_MULTIPLE: u64 = 4;
/// Q7 的 H1：新并进（顺着释放链拉进来的）占 final_nodes 至少 80%，年龄中位数至少 100 个 settle_call。
const RELEASE_CHAIN_PULLED_SHARE_AT_LEAST: f64 = 0.8;
const RELEASE_CHAIN_AGE_MEDIAN_AT_LEAST: u64 = 100;
/// 控制臂 `C1`、`C2` 每条记录几步（第五节阳性对照表）。
const SCAN_EVERY_PUBLISH_STEPS_PER_RECORD: u64 = 1;
const SCAN_EVERY_PUBLISH_FOUR_TIMES_STEPS_PER_RECORD: u64 = 4;
/// Q9 的倍数线（正推一路的取法，第四节第 6 条）。
const THEN_BIG_RATIO_AT_MOST: f64 = 2.0;
/// 第七节 ② 第 4 条：声明的单元差与 `mapped_units` 差之比的区间。
const DECLARED_TO_MAPPED_AT_LEAST: f64 = 0.5;
const DECLARED_TO_MAPPED_AT_MOST: f64 = 2.0;
/// 第八节：改动占总量超过 5% 的点在「远小于总量」上不计。
const FAR_BELOW_THE_TOTAL_AT_MOST: f64 = 0.05;
/// 阴性对照：没有普通分配的发布轮数上界（第五节）。
const NEGATIVE_CONTROL_ROUNDS_AT_MOST: u64 = 4;
/// `user_units_equal_declared` 档的 K 下限（读法第 7 条）。
const LARGE_CHANGE_UNITS_AT_LEAST: u64 = 100;
/// F8：每个新点每个状态里「第一次碰、仍是 K 个单元」的 W2 操作至少几个。
const FIRST_TOUCHES_AT_LEAST: usize = 5;
/// 第七节 ② 第 3 条的尖峰复现钉（`files_1000_units_100` 同一次挂载 W1 第 1、2 次）。
const SPIKE_PIN_FIRST: [(&str, u64); 5] =
    [("rounds", 3), ("final_roles", 452), ("final_nodes", 433), ("round_work", 917), ("closure_nodes_added", 410)];
const SPIKE_PIN_STATE: [(&str, u64); 2] = [("mapped_units", 101_581), ("allocation_records", 238_324)];
const SPIKE_PIN_SECOND: [(&str, u64); 4] = [("rounds", 2), ("final_roles", 20), ("final_nodes", 7), ("round_work", 32)];
/// 第七节 ② 第 6 条：第四段历史末 `mapped_units` 的区间。
const BIG_POINT_MAPPED_UNITS: [(&str, u64, u64); 3] = [
    ("files_1000_units_1000", 1_000_000, 1_050_000),
    ("files_100_units_1000_unit_area_2_pow_21", 100_000, 110_000),
    ("files_10_units_1000_unit_area_2_pow_21", 10_000, 12_000),
];
/// 第七节 ② 第 1 条：登记抄下的两行（`research/results/e206-fixed-point-cost-report-2026-11-09.out:695`、`:706`）。
const OLD_CRITERION_FIRST_PUBLISH_FILES: &str =
    "name=q8_first_publish arm=p2a axis=files low_end=2102 high_end=15440 alpha=0.289 band=between local_slopes=0.045,0.295,0.526";
const OLD_CRITERION_PEAK_DATA_UNITS: &str =
    "name=q8_peak arm=p2a axis=data_units low_end=3763 high_end=3231038 alpha=0.978 band=grows local_slopes=0.660,1.019,1.255";

// ---- 增长阶与判定带（E206 模型同名函数，抄来）----
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
    /// 这一轮走出来的有记录的节点与这一版的节点对不上（读数行 `holding_differs=1`）。
    holding_differs: bool,
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
    operation_name: String,
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
    closure_nodes_added: u64,
    slots_placed: u64,
    previous_nodes: u64,
    /// 固定点入口时分配记录树的记录数与树高（对账分辨读数、复量的钉用；E206 产物同名）。
    previous_records: u64,
    tree_height: u64,
    release_chain_depth: u64,
    release_chain_nodes: u64,
    active_leaves: Vec<u64>,
    /// 第 4 组（E207 产物才有；E206 产物读成 `none` 与 0）。
    first_merge_site: String,
    seed_nodes_first_merge: u64,
    pulled_nodes_first_merge: u64,
    pulled_age_minimum: u64,
    pulled_age_median: u64,
    pulled_age_maximum: u64,
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
struct PointArmReadings {
    name: String,
    point: String,
    arm: String,
    /// 探针（E207 产物才有；E206 产物读成 `standard`）。
    probe: String,
    /// 每个小文件的数据单元数 K（E207 产物的 `units_per_small_file`；E206 产物按形状补：`_units_<K>` 的点取 K，别的取 1）。
    units_per_small_file: u64,
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
    /// 对账行逐操作（修订 3：`base`、`files_1000` 两点两种状态每个量窗操作）：(状态, 操作, 第几次) → (盘上变了的单元槽数, 计数记的落点槽数)，次序照产物。
    reconciliation_rows: Vec<((String, String, u64), (u64, u64))>,
    /// 每个量窗操作做完之后的镜像哈希（`image_after_the_operation`，修订 3）：(状态, 操作, 第几次) → sha256，次序照产物。
    images_after_the_operations: Vec<((String, String, u64), String)>,
    /// 重挂载前后的镜像哈希（`image_at_the_remount`）：`before_the_crash` / `after_the_mount` → sha256。
    images_at_the_remount: BTreeMap<String, String>,
    /// 产物头里三份被测核心文件的 sha256（`name=source_sha256`）：文件名 → sha256。
    source_sha256: BTreeMap<String, String>,
    clause_violations: Vec<String>,
    /// 第 3 组声明行：(状态, 操作, 第几次) → (user_units, old_units, new_units)，次序照产物。
    declared: Vec<((String, String, u64), (u64, u64, u64))>,
    refused: Option<String>,
    done: bool,
}

/// 读数行里标「哪一种操作、第几次」的两个键。E207 自己的产物写全；E206 的留存产物逐字是另一对，读它时只能照它的写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OperationKeys {
    name: &'static str,
    index: &'static str,
}

const E207_OPERATION_KEYS: OperationKeys = OperationKeys { name: "operation", index: "operation_index" };
/// 读 E206 留存产物（`research/results/e206-fixed-point-cost-*`）的键，逐字照那批产物。
const E206_OPERATION_KEYS: OperationKeys = OperationKeys { name: "op", index: "op_index" };

/// 一行读数是哪个实验写的，就用哪一对键：`E206RESULT` 行取 E206 的，其余取 E207 的。
fn operation_keys_of(line: &str) -> OperationKeys {
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
fn take_line(item: &mut PointArmReadings, fields: &Fields, keys: OperationKeys) {
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

fn read_item(path: &str) -> Result<PointArmReadings, String> {
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

type ReadingsByPointAndArm<'items> = BTreeMap<(String, String), &'items PointArmReadings>;

fn window_settle<'items>(
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
fn units_by_the_point_name(point: &str) -> u64 {
    point
        .split("_units_")
        .nth(1)
        .and_then(|rest| rest.split('_').next())
        .and_then(|text| text.parse().ok())
        .unwrap_or(1)
}

/// 一次固定点的 B8 合计（三段之和，读法第 1 条照 E206 模型 `by_delta_of_settle` 的口径，不带 P3）；读不到交 `None`。
fn by_delta_of_settle(item: &PointArmReadings, settle: &Settle) -> Option<u64> {
    let judgement = judgement_of(item, settle)?;
    by_delta_of_the_judgement(&judgement, item.unit_area_slots).ok()
}

/// B8 里含 `lg(Z)` 因子的行：(行名, 系数, Z)，这一行的步数里 `lg(Z)` 那一项 = 系数 × lg(Z)（控制臂 `C3` 把它换成系数 × Z）。
fn lg_terms(input: &ByDeltaInput) -> Vec<(&'static str, u64, u64)> {
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
type LgRow = (u64, u64, u64);

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
struct Operation {
    state: String,
    operation_name: String,
    operation_index: u64,
    /// 这个状态里的次序（0 起，照产物里第一次出现）。
    order: usize,
    /// 读法第 2 条：外生的用户单元数（E207 读声明行；E206 产物按构造补）。
    user_units: Option<u64>,
    old_units: u64,
    new_units: u64,
    /// 读法第 3 条：操作之前的 `allocation_records`（主读）与 `mapped_units`（另报）。
    allocation_records: Option<u64>,
    mapped_units: Option<u64>,
    /// 旧判据的旋钮 x（点的 F × K 或 E206 的轴位置），只与旧判据对照，不进判定。
    knob: u64,
    /// 各次固定点 B8 合计之和；有一次读不到就是 `None`。
    steps: Option<u64>,
    /// 几次固定点里 final_nodes 最大的那一次（尖峰旗吃它），与它在 `item.window` 里的下标。
    final_nodes: u64,
    peak_settle: Option<usize>,
    /// 各轮释放的落点数之和（`Rs`，只给变异 M3 的对照，不进判定）。
    releases: u64,
    /// 几次固定点在准入之前三段里 s37 的调用次数 pooled_fallback_calls、s37 的步数、s29 的步数；pooled_fallback_calls > 0 的操作 B8、B4 合计缺 s37 一行（见 `ROW_WITHOUT_A_BY_DELTA_FORMULA`）。
    pooled_fallback_calls: u64,
    /// 操作之前的规模读数行里 `frozen_publish_set`（发布落盘失败冻结着等重发时是 1）；读不到交 `None`（旧产物没有这个字段）。
    frozen_publish_set: Option<u64>,
    pooled_fallback_steps: u64,
    fallback_scan_steps: u64,
    /// 含 `lg` 的行：行名 → (系数 × lg(Z), 系数 × Z)，几次固定点相加。
    lg_rows: BTreeMap<&'static str, LgRow>,
    flagged: bool,
}

/// 读法第 3 条：量到的总量 x 取操作之前的 `allocation_records`。
fn measured_total(operation: &Operation) -> Option<f64> {
    operation.allocation_records.map(|records| records as f64)
}

/// 读法第 3 条的另报：x 取 `mapped_units`。
fn measured_total_by_mapped_units(operation: &Operation) -> Option<f64> {
    operation.mapped_units.map(|units| units as f64)
}

/// 读法第 5 条的分母：外生的 `user_units`，不用实现算出来的 `Rs`。
fn denominator(operation: &Operation) -> Option<u64> {
    operation.user_units
}

/// E206 产物没有声明行：按 E206 历史的构造补（读法第 2 条，只用于回扫）。`live` = [大文件（D 个单元）, F 个小文件（各 K 个）]，
/// W2 第 i 次取 `live[(i − 1) × len ÷ 32 % len]`，一个文件第一次被覆盖写时是它建时的单元数、之后是 1；W1、W4 覆盖写第一个文件（1）；
/// W3 一次建—写—删（1 + 1 + 1）；阴性对照（1）。别的操作不补。
fn constructed_user_units(item: &PointArmReadings, operation_name: &str, operation_index: u64, touched: &mut BTreeSet<usize>) -> Option<(u64, u64, u64)> {
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
fn operations_of(item: &PointArmReadings) -> Vec<Operation> {
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
fn lower_median(values: &[u64]) -> u64 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted.get(sorted.len().saturating_sub(1) / 2).copied().unwrap_or(0)
}

/// 读法第 8 条的旗线：max(28, 4 × 中位数)。
fn spike_line(final_nodes: &[u64]) -> u64 {
    SPIKE_FLOOR.max(SPIKE_MULTIPLE * lower_median(final_nodes))
}

// ---- 被测臂与控制臂（第五节阳性对照表）：控制臂在同一批读数上加项，同一把尺判 ----
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    Tested,
    ScanEveryPublish,
    ScanEveryPublishFourTimes,
    LargestLgRowLinear,
}

const ARMS: [Arm; 4] = [Arm::Tested, Arm::ScanEveryPublish, Arm::ScanEveryPublishFourTimes, Arm::LargestLgRowLinear];

impl Arm {
    fn tag(self) -> &'static str {
        match self {
            Self::Tested => "tested",
            Self::ScanEveryPublish => "c1",
            Self::ScanEveryPublishFourTimes => "c2",
            Self::LargestLgRowLinear => "c3",
        }
    }
}

/// `C3` 换线性的那一行：这个点这个状态量窗全部操作里，含 `lg` 的行中步数合计最大的一行（同值取名字在前的）。
fn largest_lg_row(operations: &[&Operation]) -> Option<&'static str> {
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
fn arm_steps(operation: &Operation, arm: Arm, linear_row: Option<&str>) -> Option<u64> {
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
enum Pick {
    First,
    LaterPeak,
    LaterMedian,
}

const PICKS: [Pick; 3] = [Pick::First, Pick::LaterPeak, Pick::LaterMedian];

impl Pick {
    fn tag(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::LaterPeak => "later_peak",
            Self::LaterMedian => "later_median",
        }
    }
}

/// 一列照次序排好的 (操作, 步数) 里取一样：`first` = 次序第一的；`later_peak` = 其余里步数最大的（同值取在前的）；
/// `later_median` = 其余按步数排好序的下中位数。
fn pick_from<'operations>(series: &[(&'operations Operation, u64)], pick: Pick) -> Option<(&'operations Operation, u64)> {
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
fn kept_for_the_filter(operation: &Operation, steady: bool) -> bool {
    !(steady && operation.flagged)
}

/// 读法第 7 条的档：`one_user_unit` = user_units 为 1；`user_units_equal_declared` = user_units 等于这个点声明的 K（K ≥ 100）。
fn in_the_tier(operation: &Operation, tier: &str, units_per_small_file: u64) -> bool {
    match tier {
        "one_user_unit" => operation.user_units == Some(1),
        "user_units_equal_declared" => units_per_small_file >= LARGE_CHANGE_UNITS_AT_LEAST && operation.user_units == Some(units_per_small_file),
        _ => false,
    }
}

// ---- 读法第 4、5 条：两点读数 ----
/// 一端：(步数, user_units, x)。
type End = (u64, u64, f64);

#[derive(Clone, Copy, Debug, PartialEq)]
struct TwoPointReading {
    looser: bool,
    alpha: Option<f64>,
    raw_alpha: Option<f64>,
}

fn positive_alpha(low_quantity: f64, high_quantity: f64, low_measured_total: f64, high_measured_total: f64) -> Option<f64> {
    (low_quantity > 0.0 && high_quantity > 0.0 && low_measured_total > 0.0 && high_measured_total > 0.0 && low_measured_total != high_measured_total)
        .then(|| alpha(low_quantity, high_quantity, low_measured_total, high_measured_total))
}

/// 读法第 5 条：两端 user_units 相同读原步数；不同读步数 ÷ user_units，标 `looser`，同时报原步数的 α。
fn two_point_reading(low: End, high: End) -> TwoPointReading {
    let looser = low.1 != high.1;
    let quantity = |end: End| if looser { end.0 as f64 / end.1 as f64 } else { end.0 as f64 };
    TwoPointReading {
        looser,
        alpha: positive_alpha(quantity(low), quantity(high), low.2, high.2),
        raw_alpha: positive_alpha(low.0 as f64, high.0 as f64, low.2, high.2),
    }
}

/// 三点以上：相邻两点的局部斜率（第八节「局部斜率」）。
fn local_slopes(points: &[(f64, f64)]) -> Vec<Option<f64>> {
    points.windows(2).map(|pair| positive_alpha(pair[0].0, pair[1].0, pair[0].1, pair[1].1)).collect()
}

fn alpha_text(value: Option<f64>) -> String {
    value.map_or("unread".to_string(), |value| format!("{value:.3}"))
}

fn band_text(value: Option<f64>) -> &'static str {
    value.map_or("unread", |value| band(value).tag())
}

// ---- 比较集合（读法第 6 条）与格 ----
/// 一个点读好的东西：产物与它的操作。
struct PointReadings<'items> {
    item: &'items PointArmReadings,
    operations: Vec<Operation>,
}

type Points<'items> = BTreeMap<String, PointReadings<'items>>;

/// 报告行：各报告函数往这里推，`report` 读完全部产物之后逐行打；单测直接读它。
type Lines = Vec<String>;

/// 一个比较集合：段（`new` = 第一至三段的新点、`big` = 第四段、`retro` = 回扫）、名字、点（低到高）、判哪几档。
struct ComparisonSet {
    segment: &'static str,
    name: &'static str,
    points: &'static [&'static str],
    tiers: &'static [&'static str],
}

const NEW_POINT_SETS: [ComparisonSet; 6] = [
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

const BIG_POINT_SETS: [ComparisonSet; 1] = [ComparisonSet {
    segment: "big",
    name: "K1000L",
    points: &["files_10_units_1000_unit_area_2_pow_21", "files_100_units_1000_unit_area_2_pow_21", "files_1000_units_1000"],
    tiers: &["one_user_unit", "user_units_equal_declared"],
}];

const RETRO_SETS: [ComparisonSet; 4] = [
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
const NEW_POINT_QUANTITIES: [(&str, &str, &str, &[&str]); 5] = [
    ("one_unit_change_against_the_total", "q1", "one_user_unit", &["K100", "K1000", "FTB"]),
    ("many_unit_change_against_the_total", "q2", "user_units_equal_declared", &["K100", "K1000"]),
    ("per_unit_steps_as_the_change_grows", "q3", "user_units_equal_declared", &["D100", "D10"]),
    ("many_unit_change_against_the_total_on_the_other_unit_area", "q2_other_unit_area", "user_units_equal_declared", &["K1000_other_unit_area"]),
    ("one_unit_change_against_the_total_on_the_other_unit_area", "q1_other_unit_area", "one_user_unit", &["K1000_other_unit_area"]),
];
/// Q10 与 Q4 在报告行里的 `reading=`。
const BIG_POINT_READING: &str = "big_points_against_the_total";
const RETRO_READING: &str = "e206_products_read_again";

/// 一格的结果：每条臂的两端（三点集合另有中点）读数。
#[derive(Clone, Debug, Default)]
struct CellOutcome {
    /// 臂 → α（读法第 5 条的量；三点以上取低端对高端）。
    alpha: BTreeMap<&'static str, Option<f64>>,
    /// 臂 → 原步数的 α（`looser=yes` 的格与读法第 5 条的量不同）。
    raw_alpha: BTreeMap<&'static str, Option<f64>>,
}

impl CellOutcome {
    fn band_of(&self, arm: Arm) -> Option<Band> {
        self.alpha.get(arm.tag()).copied().flatten().map(band)
    }
}

/// 一个点一个状态一档一种取法一条臂取出来的那一端；读不到交 `None`。
fn end_of(point: &PointReadings<'_>, state: &str, tier: &str, pick: Pick, steady: bool, arm: Arm) -> Option<(End, Option<f64>, &'static str)> {
    let in_state: Vec<&Operation> = point.operations.iter().filter(|operation| operation.state == state).collect();
    let linear_row = largest_lg_row(&in_state);
    let series: Vec<(&Operation, u64)> = in_state
        .iter()
        .filter(|operation| in_the_tier(operation, tier, point.item.units_per_small_file) && kept_for_the_filter(operation, steady))
        .filter_map(|operation| arm_steps(operation, arm, linear_row).map(|steps| (*operation, steps)))
        .collect();
    let (operation, steps) = pick_from(&series, pick)?;
    let user_units = denominator(operation)?;
    Some(((steps, user_units, measured_total(operation)?), measured_total_by_mapped_units(operation), linear_row.unwrap_or("none")))
}

/// 一格（集合 × 状态 × 档 × 取法 × 剔不剔旗）：每条臂一行 `name=cell`，交回各臂的 α。
fn report_cell(points: &Points<'_>, set: &ComparisonSet, state: &str, tier: &str, pick: Pick, steady: bool, out: &mut Lines) -> CellOutcome {
    let mut outcome = CellOutcome::default();
    for arm in ARMS {
        let ends: Vec<Option<(End, Option<f64>, &'static str)>> = set
            .points
            .iter()
            .map(|name| points.get(*name).and_then(|point| end_of(point, state, tier, pick, steady, arm)))
            .collect();
        let low = ends.first().copied().flatten();
        let high = ends.last().copied().flatten();
        let reading = low.zip(high).map(|(low, high)| two_point_reading(low.0, high.0));
        let by_mapped = low.zip(high).and_then(|(low, high)| {
            let looser = low.0 .1 != high.0 .1;
            let quantity = |end: End| if looser { end.0 as f64 / end.1 as f64 } else { end.0 as f64 };
            positive_alpha(quantity(low.0), quantity(high.0), low.1?, high.1?)
        });
        let slopes: String = if ends.len() > 2 && ends.iter().all(Option::is_some) {
            let looser = ends.iter().flatten().map(|end| end.0 .1).collect::<BTreeSet<u64>>().len() > 1;
            let quantity = |end: End| if looser { end.0 as f64 / end.1 as f64 } else { end.0 as f64 };
            let values: Vec<(f64, f64)> = ends.iter().flatten().map(|end| (quantity(end.0), end.0 .2)).collect();
            let upper = positive_alpha(values[values.len() - 2].0, values[values.len() - 1].0, values[values.len() - 2].1, values[values.len() - 1].1);
            format!(
                "{} upper_pair_alpha={}",
                local_slopes(&values).into_iter().map(alpha_text).collect::<Vec<_>>().join(","),
                alpha_text(upper)
            )
        } else if ends.len() > 2 && high.is_none() {
            // 作废条款第 7 条：高端缺了不拿两点冒充三点，α 报 unread；低端与中点都在时只另报这两点的 α（总量十倍）。
            let lower_pair = ends[0].zip(ends[1]).and_then(|(low, middle)| two_point_reading(low.0, middle.0).alpha);
            format!("high_end_missing lower_pair_alpha={}", alpha_text(lower_pair))
        } else {
            "two_points".to_string()
        };
        let ends_text: Vec<String> = ends
            .iter()
            .map(|end| end.map_or("unread".to_string(), |end| format!("{}/{}/{}", end.0 .0, end.0 .1, end.0 .2)))
            .collect();
        let alpha = reading.and_then(|reading| reading.alpha);
        out.push(format!(
            "name=cell segment={} set={} state={state} tier={tier} pick={} filter={} arm={} ends={} looser={} alpha={} band={} raw_alpha={} alpha_by_mapped_units={} local_slopes={} linear_row={}",
            set.segment,
            set.name,
            pick.tag(),
            if steady { "steady" } else { "all" },
            arm.tag(),
            ends_text.join(","),
            reading.map_or("unread", |reading| if reading.looser { "yes" } else { "no" }),
            alpha_text(alpha),
            band_text(alpha),
            alpha_text(reading.and_then(|reading| reading.raw_alpha)),
            alpha_text(by_mapped),
            slopes,
            low.map_or("none", |end| end.2)
        ));
        outcome.alpha.insert(arm.tag(), alpha);
        outcome.raw_alpha.insert(arm.tag(), reading.and_then(|reading| reading.raw_alpha));
    }
    outcome
}

// ---- 判读：行一、行二、作废条款第 1、2 条（第六节、第十一节）----
/// 一格的定位与结果。
struct JudgedCell {
    segment: &'static str,
    set: &'static str,
    state: &'static str,
    tier: &'static str,
    pick: Pick,
    steady: bool,
    outcome: CellOutcome,
}

/// 作废条款第 1、2 条：（段 × 档）各判各的，第四段与第一至三段不合并。
fn void_segment(segment: &'static str) -> &'static str {
    segment
}

/// 一个（段 × 档）里控制臂的账：应判的格（被测臂不随或介于）、`C1` 判随的、`C3` 判随的。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ControlTally {
    applicable: u64,
    scan_red: u64,
    linear_red: u64,
}

impl ControlTally {
    fn scan_void(self) -> bool {
        self.scan_red < self.applicable
    }
    fn linear_void(self) -> bool {
        self.linear_red < self.applicable
    }
}

fn control_tallies(cells: &[JudgedCell]) -> BTreeMap<(&'static str, &'static str), ControlTally> {
    let mut tallies: BTreeMap<(&'static str, &'static str), ControlTally> = BTreeMap::new();
    for cell in cells {
        let tally = tallies.entry((void_segment(cell.segment), cell.tier)).or_default();
        if matches!(cell.outcome.band_of(Arm::Tested), Some(Band::DoesNotGrow | Band::Between)) {
            tally.applicable += 1;
            tally.scan_red += u64::from(cell.outcome.band_of(Arm::ScanEveryPublish) == Some(Band::Grows));
            tally.linear_red += u64::from(cell.outcome.band_of(Arm::LargestLgRowLinear) == Some(Band::Grows));
        }
    }
    tallies
}

/// 一组格对行一、行二的判读（第六节 Q1–Q3、Q10 那一列；只报，合取留给主 agent）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RowReading {
    cells: u64,
    unread: u64,
    /// `steady` 格全部「不随」且 `C1`、`C3` 在这些格都「随」。
    row_one_holds: bool,
    /// 任一 `later_*` 的 `steady` 格「随」且 `C1`、`C3` 在该格「随」。
    row_two_flips: bool,
    /// F2：某（集合、状态）里 `first` 的 `all` 格「随」而 `steady` 与 `later_*` 全不随。
    only_the_first_grows: bool,
}

fn row_reading(cells: &[&JudgedCell]) -> RowReading {
    let steady: Vec<&&JudgedCell> = cells.iter().filter(|cell| cell.steady).collect();
    fn controls_red(cell: &JudgedCell) -> bool {
        cell.outcome.band_of(Arm::ScanEveryPublish) == Some(Band::Grows) && cell.outcome.band_of(Arm::LargestLgRowLinear) == Some(Band::Grows)
    }
    let unread = cells.iter().filter(|cell| cell.outcome.band_of(Arm::Tested).is_none()).count();
    let row_one_holds = !steady.is_empty()
        && steady.iter().all(|cell| cell.outcome.band_of(Arm::Tested) == Some(Band::DoesNotGrow) && controls_red(cell));
    let row_two_flips = steady
        .iter()
        .any(|cell| cell.pick != Pick::First && cell.outcome.band_of(Arm::Tested) == Some(Band::Grows) && controls_red(cell));
    let mut only_the_first_grows = false;
    let groups: BTreeSet<(&str, &str, &str)> = cells.iter().map(|cell| (cell.set, cell.state, cell.tier)).collect();
    for (set, state, tier) in groups {
        let of_group: Vec<&&JudgedCell> = cells.iter().filter(|cell| (cell.set, cell.state, cell.tier) == (set, state, tier)).collect();
        let first_all_grows = of_group
            .iter()
            .any(|cell| cell.pick == Pick::First && !cell.steady && cell.outcome.band_of(Arm::Tested) == Some(Band::Grows));
        let rest_do_not = of_group
            .iter()
            .filter(|cell| cell.steady || cell.pick != Pick::First)
            .all(|cell| cell.outcome.band_of(Arm::Tested).is_some_and(|band| band != Band::Grows));
        only_the_first_grows |= first_all_grows && rest_do_not;
    }
    RowReading {
        cells: u64::try_from(cells.len()).unwrap_or(u64::MAX),
        unread: u64::try_from(unread).unwrap_or(u64::MAX),
        row_one_holds,
        row_two_flips,
        only_the_first_grows,
    }
}

/// 一组集合逐格报（每个集合 × 档 × 状态 × 取法 × 剔不剔旗），交回全部格。
fn judge_sets<'sets>(points: &Points<'_>, sets: &'sets [ComparisonSet], out: &mut Lines) -> Vec<JudgedCell> {
    let mut cells = Vec::new();
    for set in sets {
        for &tier in set.tiers {
            for state in ["same_mount", "remounted"] {
                for pick in PICKS {
                    for steady in [false, true] {
                        let outcome = report_cell(points, set, state, tier, pick, steady, out);
                        cells.push(JudgedCell { segment: set.segment, set: set.name, state, tier, pick, steady, outcome });
                    }
                }
            }
        }
    }
    cells
}

fn report_row_reading((reading_name, quantity): (&str, &str), segment: &str, tier: &str, cells: &[&JudgedCell], tallies: &BTreeMap<(&'static str, &'static str), ControlTally>, out: &mut Lines) {
    let reading = row_reading(cells);
    let tally = tallies.iter().find(|((in_segment, in_tier), _)| *in_segment == void_segment_name(segment) && *in_tier == tier).map(|(_, tally)| *tally).unwrap_or_default();
    out.push(format!(
        "name=verdict quantity={quantity} reading={reading_name} segment={segment} tier={tier} cells={} cells_unread={} row_one_holds={} row_two_flips={} only_the_first_grows={} scan_control_void={} linear_control_void={}",
        reading.cells,
        reading.unread,
        reading.row_one_holds,
        reading.row_two_flips,
        reading.only_the_first_grows,
        tally.scan_void(),
        tally.linear_void()
    ));
}

/// 报告行里的段名到作废条款按段那一键（同 `void_segment`，只是交回的不借静态生命期）。
fn void_segment_name(segment: &str) -> &str {
    match segment {
        "new" => void_segment("new"),
        "big" => void_segment("big"),
        _ => void_segment("retro"),
    }
}

/// Q3 的 T1e5：总量相近、改动差十倍，只报 α_u = ln(Q_hi ÷ Q_lo) ÷ ln(u_hi ÷ u_lo)（Q = 步数 ÷ user_units），另报原步数的；不按 x 判。
fn report_similar_total_set(points: &Points<'_>, out: &mut Lines) {
    let names = ["files_100_units_1000", "files_10_units_10000"];
    for state in ["same_mount", "remounted"] {
        for pick in PICKS {
            for steady in [false, true] {
                let ends: Vec<Option<End>> = names
                    .iter()
                    .map(|name| points.get(*name).and_then(|point| end_of(point, state, "user_units_equal_declared", pick, steady, Arm::Tested)).map(|end| end.0))
                    .collect();
                let (per_unit, raw) = match (ends[0], ends[1]) {
                    (Some(low), Some(high)) => (
                        positive_alpha(low.0 as f64 / low.1 as f64, high.0 as f64 / high.1 as f64, low.1 as f64, high.1 as f64),
                        positive_alpha(low.0 as f64, high.0 as f64, low.1 as f64, high.1 as f64),
                    ),
                    _ => (None, None),
                };
                out.push(format!(
                    "name=t1e5 state={state} pick={} filter={} alpha_against_the_user_units_per_unit={} band_per_unit={} alpha_against_the_user_units_raw={} band_raw={}",
                    pick.tag(),
                    if steady { "steady" } else { "all" },
                    alpha_text(per_unit),
                    band_text(per_unit),
                    alpha_text(raw),
                    band_text(raw)
                ));
            }
        }
    }
}

/// F3：`looser=yes` 的格原步数 α ≥ 0.5 而步数 ÷ user_units 的 α ≤ 0.25。
fn report_looser_instances(cells: &[JudgedCell], out: &mut Lines) {
    let mut instances = 0;
    for cell in cells {
        let raw = cell.outcome.raw_alpha.get(Arm::Tested.tag()).copied().flatten();
        let per_unit = cell.outcome.alpha.get(Arm::Tested.tag()).copied().flatten();
        if raw.is_some_and(|raw| raw >= BAND_GROWS_AT_LEAST) && per_unit.is_some_and(|value| value <= BAND_DOES_NOT_GROW_AT_MOST) && raw != per_unit {
            instances += 1;
            out.push(format!(
                "name=f3_looser_instance set={} state={} tier={} pick={} filter={} raw_alpha={} per_unit_alpha={}",
                cell.set,
                cell.state,
                cell.tier,
                cell.pick.tag(),
                if cell.steady { "steady" } else { "all" },
                alpha_text(raw),
                alpha_text(per_unit)
            ));
        }
    }
    out.push(format!("name=f3_looser_instances count={instances}"));
}

/// Q6：旗上的操作，逐点逐状态照报；第八节轨迹：B8 步数与 final_nodes 整列打（每个操作一行）。
fn report_operations(points: &Points<'_>, out: &mut Lines) {
    for (name, point) in points {
        for operation in &point.operations {
            out.push(format!(
                "name=op_row item={} state={} operation={} operation_index={} order={} user_units={} old_units={} new_units={} allocation_records={} mapped_units={} steps={} final_nodes={} flagged={}",
                point.item.name,
                operation.state,
                operation.operation_name,
                operation.operation_index,
                operation.order,
                operation.user_units.map_or("unread".to_string(), |value| value.to_string()),
                operation.old_units,
                operation.new_units,
                operation.allocation_records.map_or("unread".to_string(), |value| value.to_string()),
                operation.mapped_units.map_or("unread".to_string(), |value| value.to_string()),
                operation.steps.map_or("unread".to_string(), |value| value.to_string()),
                operation.final_nodes,
                operation.flagged
            ));
        }
        for operation in &point.operations {
            out.push(format!(
                "name=op_pooled_fallback item={} state={} operation={} operation_index={} pooled_fallback_calls={} pooled_fallback_steps={} per_disk_fallback_steps={} by_delta_arms_total={}",
                point.item.name,
                operation.state,
                operation.operation_name,
                operation.operation_index,
                operation.pooled_fallback_calls,
                operation.pooled_fallback_steps,
                operation.fallback_scan_steps,
                if operation.pooled_fallback_calls > 0 { "missing_pooled_fallback_row_only_for_the_listed_steps" } else { "complete" }
            ));
        }
        for state in ["same_mount", "remounted"] {
            let in_state: Vec<&Operation> = point.operations.iter().filter(|operation| operation.state == state).collect();
            let nodes: Vec<u64> = in_state.iter().map(|operation| operation.final_nodes).collect();
            let flagged: Vec<String> = in_state
                .iter()
                .filter(|operation| operation.flagged)
                .map(|operation| format!("{}#{}:{}", operation.operation_name, operation.operation_index, operation.final_nodes))
                .collect();
            let peak = in_state.iter().filter_map(|operation| operation.steps.map(|steps| (steps, *operation))).max_by_key(|(steps, _)| *steps);
            out.push(format!(
                "name=q6_spike_flags point={name} state={state} operations={} median_final_nodes={} line={} flagged={} flagged_operations={} peak_steps={} peak_at={} final_steps={}",
                in_state.len(),
                lower_median(&nodes),
                spike_line(&nodes),
                flagged.len(),
                if flagged.is_empty() { "-".to_string() } else { flagged.join(",") },
                peak.map_or("unread".to_string(), |(steps, _)| steps.to_string()),
                peak.map_or("unread".to_string(), |(_, operation)| format!("{}#{}", operation.operation_name, operation.operation_index)),
                in_state.last().and_then(|operation| operation.steps).map_or("unread".to_string(), |steps| steps.to_string())
            ));
        }
    }
}

/// 作废条款（冻结的发布）：被量的每次操作之前读一次冻结着等重发的那一次是不是空，不空的那一点整点作废并报出来。
/// 交回作废的点名；没带这个字段的操作数另报在判决行里（读不到不当「空」，也不当「不空」）。
fn points_with_a_frozen_publish(points: &Points<'_>, out: &mut Lines) -> BTreeSet<String> {
    let (mut voided, mut unread, mut operations) = (BTreeSet::new(), 0, 0);
    for (name, point) in points {
        for operation in &point.operations {
            operations += 1;
            match operation.frozen_publish_set {
                None => unread += 1,
                Some(0) => {}
                Some(_) => {
                    voided.insert(name.clone());
                }
            }
        }
    }
    for name in &voided {
        out.push(format!("name=void_frozen_publish point={name} result=point_voided"));
    }
    out.push(format!(
        "name=verdict quantity=void_frozen_publish operations={operations} operations_without_the_reading={unread} points_voided={} voided_points={}",
        voided.len(),
        if voided.is_empty() { "-".to_string() } else { voided.iter().cloned().collect::<Vec<_>>().join(",") }
    ));
    voided
}

/// s37 的标记汇总：每份产物每个点报 pooled_fallback_calls > 0 的操作数；岔路单够判条件落在这些点上的，停下交主 agent（先由设计员给 B8 定这一步的做法）。
fn report_missing_the_pooled_fallback_row(points: &Points<'_>, out: &mut Lines) {
    let (mut operations, mut marked) = (0, 0);
    let mut marked_points = BTreeSet::new();
    for (name, point) in points {
        for operation in &point.operations {
            operations += 1;
            if operation.pooled_fallback_calls > 0 {
                marked += 1;
                marked_points.insert(name.clone());
            }
        }
    }
    out.push(format!(
        "name=verdict quantity=missing_pooled_fallback operations={operations} operations_with_pooled_fallback_calls={marked} points_marked={} marked_points={}",
        marked_points.len(),
        if marked_points.is_empty() { "-".to_string() } else { marked_points.into_iter().collect::<Vec<_>>().join(",") }
    ));
}

// ---- 行三：Q7 尖峰成因、Q8 重现条件 ----
/// Q7 的 H1：首次合并走在释放站点、新拉进来的占 final_nodes 至少 80%、年龄中位数至少 100 个 settle_call；交回成不成立与不满足的那几条。
fn release_chain_cause_holds(site: &str, pulled: u64, final_nodes: u64, age_median: u64) -> (bool, Vec<&'static str>) {
    let mut failing = Vec::new();
    if site != "release_not_covered" {
        failing.push("site");
    }
    if final_nodes == 0 || (pulled as f64) < RELEASE_CHAIN_PULLED_SHARE_AT_LEAST * final_nodes as f64 {
        failing.push("pulled_share");
    }
    if age_median < RELEASE_CHAIN_AGE_MEDIAN_AT_LEAST {
        failing.push("age_median");
    }
    (failing.is_empty(), failing)
}

fn report_spike_causes(points: &Points<'_>, out: &mut Lines) {
    let (mut flagged, mut holding) = (0, 0);
    for point in points.values() {
        for operation in point.operations.iter().filter(|operation| operation.flagged) {
            let Some(settle) = operation.peak_settle.and_then(|index| point.item.window.get(index)) else {
                continue;
            };
            let (holds, failing) = release_chain_cause_holds(&settle.first_merge_site, settle.pulled_nodes_first_merge, settle.final_nodes, settle.pulled_age_median);
            flagged += 1;
            holding += u64::from(holds);
            out.push(format!(
                "name=q7_h1 item={} state={} operation={} operation_index={} first_merge_site={} seed_nodes_first_merge={} pulled_nodes_first_merge={} final_nodes={} pulled_share={:.3} pulled_age_minimum={} pulled_age_median={} pulled_age_maximum={} release_chain_depth={} rounds={} holds={holds} failing={}",
                point.item.name,
                operation.state,
                operation.operation_name,
                operation.operation_index,
                settle.first_merge_site,
                settle.seed_nodes_first_merge,
                settle.pulled_nodes_first_merge,
                settle.final_nodes,
                if settle.final_nodes == 0 { 0.0 } else { settle.pulled_nodes_first_merge as f64 / settle.final_nodes as f64 },
                settle.pulled_age_minimum,
                settle.pulled_age_median,
                settle.pulled_age_maximum,
                settle.release_chain_depth,
                settle.rounds,
                if failing.is_empty() { "-".to_string() } else { failing.join(",") }
            ));
        }
    }
    out.push(format!("name=verdict quantity=q7 flagged_operations={flagged} stale_chain_cause_holds_on={holding} stale_chain_cause_fails_on={}", flagged - holding));
}

fn flagged_at(point: &PointReadings<'_>, state: &str, operation_name: &str, operation_index: u64) -> Option<bool> {
    point
        .operations
        .iter()
        .find(|operation| operation.state == state && operation.operation_name == operation_name && operation.operation_index == operation_index)
        .map(|operation| operation.flagged)
}

/// Q8：四个探针项各列 P1、W1 第 1 次在不在旗上、W1–W4 里第一个旗上操作、W5–W7 的旗上操作与相邻两个之间隔几次固定点、重挂载后 W1 第 1 次。
fn report_reproduction_probes(points: &Points<'_>, out: &mut Lines) {
    let mut outcome: BTreeMap<&str, (Option<bool>, Option<bool>)> = BTreeMap::new();
    for (probe, name) in [
        ("standard", "files_1000_units_100"),
        ("cold_first", "files_1000_units_100__head__cold_first"),
        ("create_first", "files_1000_units_100__head__create_first"),
        ("young_first", "files_1000_units_100__head__young_first"),
    ] {
        let Some(point) = points.get(name) else {
            out.push(format!("name=q8_probe probe={probe} item=unread"));
            continue;
        };
        let same: Vec<&Operation> = point.operations.iter().filter(|operation| operation.state == "same_mount").collect();
        let first_flag = same.iter().find(|operation| operation.flagged && matches!(operation.operation_name.as_str(), "W1" | "W2" | "W3" | "W4"));
        let extended: Vec<(&Operation, usize)> = same
            .iter()
            .filter(|operation| operation.flagged && matches!(operation.operation_name.as_str(), "W5" | "W6" | "W7"))
            .filter_map(|operation| operation.peak_settle.map(|index| (*operation, index)))
            .collect();
        let gaps: Vec<String> = extended.windows(2).map(|pair| (pair[1].1 - pair[0].1).to_string()).collect();
        let prefix = flagged_at(point, "same_mount", "P1", 1);
        let first_overwrite = flagged_at(point, "same_mount", "W1", 1);
        outcome.insert(probe, (prefix, first_overwrite));
        out.push(format!(
            "name=q8_probe probe={probe} item={} probe_prefix_flagged={} first_file_first_overwrite_flagged={} first_flag_in_the_base_window={} extended_flags={} settles_between_extended_flags={} remounted_first_file_first_overwrite_flagged={}",
            point.item.name,
            prefix.map_or("none".to_string(), |flag| flag.to_string()),
            first_overwrite.map_or("unread".to_string(), |flag| flag.to_string()),
            first_flag.map_or("none".to_string(), |operation| format!("{}#{}", operation.operation_name, operation.operation_index)),
            if extended.is_empty() { "-".to_string() } else { extended.iter().map(|(operation, _)| format!("{}#{}", operation.operation_name, operation.operation_index)).collect::<Vec<_>>().join(",") },
            if gaps.is_empty() { "-".to_string() } else { gaps.join(",") },
            flagged_at(point, "remounted", "W1", 1).map_or("unread".to_string(), |flag| flag.to_string())
        ));
    }
    out.push(format!("name=verdict quantity=q8 outcome={}", probe_outcome(&outcome)));
}

/// Q8 的判读线（第六节末表）：H1、H2、H3 或都不是；三个探针项缺一个记 `unread`。
fn probe_outcome(outcome: &BTreeMap<&str, (Option<bool>, Option<bool>)>) -> &'static str {
    let get = |probe: &str| outcome.get(probe).copied();
    let (Some(cold), Some(create), Some(young)) = (get("cold_first"), get("create_first"), get("young_first")) else {
        return "unread";
    };
    let probes = [cold, create, young];
    if probes.iter().any(|(prefix, first)| prefix.is_none() || first.is_none()) {
        return "unread";
    }
    let flag = |pair: (Option<bool>, Option<bool>)| (pair.0.unwrap_or(false), pair.1.unwrap_or(false));
    let (cold, create, young) = (flag(cold), flag(create), flag(young));
    if cold == (true, false) && create == (false, true) && young == (false, true) {
        "h1"
    } else if [cold, create, young].iter().all(|pair| *pair == (true, false)) {
        "h2"
    } else if [cold, create, young].iter().all(|pair| *pair == (false, true)) {
        "h3"
    } else {
        "none_of_the_three"
    }
}

/// Q9（附带）：`files_then_big_10000` 每个（状态、操作、第几次）的步数 ÷ `files_10000` 同一格的步数，取最大。
fn report_then_big_ratio(points: &Points<'_>, out: &mut Lines) {
    let (Some(then_big), Some(files)) = (points.get("files_then_big_10000"), points.get("files_10000")) else {
        out.push("name=verdict quantity=q9 maximum_ratio=unread exceeds=unread".to_string());
        return;
    };
    let mut best: Option<(f64, String)> = None;
    for operation in &then_big.operations {
        let other = files.operations.iter().find(|candidate| {
            (candidate.state.as_str(), candidate.operation_name.as_str(), candidate.operation_index) == (operation.state.as_str(), operation.operation_name.as_str(), operation.operation_index)
        });
        if let (Some(steps), Some(Some(base))) = (operation.steps, other.map(|other| other.steps)) {
            if base > 0 {
                let ratio = steps as f64 / base as f64;
                if best.as_ref().is_none_or(|(value, _)| ratio > *value) {
                    best = Some((ratio, format!("{}/{}#{}", operation.state, operation.operation_name, operation.operation_index)));
                }
            }
        }
    }
    match best {
        Some((ratio, at)) => out.push(format!("name=verdict quantity=q9 maximum_ratio={ratio:.3} at={at} exceeds={}", ratio > THEN_BIG_RATIO_AT_MOST)),
        None => out.push("name=verdict quantity=q9 maximum_ratio=unread exceeds=unread".to_string()),
    }
}

// ---- 作废条款、停机条款与失败条款里能从产物核的几样 ----
/// 作废条款第 5 条：有 item_done、声明行数等于量窗操作数（E207 产物）、没被拒；F4：没走满、没撞上界、没有条款违例。
/// 第七节 ① 第 2、3 条（F4）：重写集只增不减；定下那一轮没有新弄脏、有记录的节点对得上。照产物的逐轮读数另算一遍（产物里实现那一侧另打
/// `clause_violation` 行，两边各报）；交回 (重写集比上一轮小的轮数, 定下那一轮不干净的轮数)。
fn fixed_point_premise_breaks(settles: &[Settle]) -> (u64, u64) {
    let (mut shrank, mut settled_unclean) = (0, 0);
    for settle in settles {
        let mut size_before = 0;
        for round in &settle.detail {
            if round.rewrite_set_size < size_before {
                shrank += 1;
            }
            size_before = round.rewrite_set_size;
            if round.exit == "settled" && (round.release_dirty_new + round.placement_dirty_new != 0 || round.holding_differs) {
                settled_unclean += 1;
            }
        }
    }
    (shrank, settled_unclean)
}

/// 停机条款第 2 条：轮功 `W`（settle 行的 `round_work`）与逐轮重写角色数之和 `Σ_r A_r`（round 行的 `roles_rewritten`）对不上的固定点个数；
/// 没有逐轮读数的（历史那一段只有第 1 组）不比。
fn round_work_mismatches(settles: &[Settle]) -> u64 {
    let mismatched = settles
        .iter()
        .filter(|settle| !settle.detail.is_empty() && settle.detail.iter().map(|round| round.roles_rewritten).sum::<u64>() != settle.round_work)
        .count();
    u64::try_from(mismatched).unwrap_or(u64::MAX)
}

/// 第五节「量窗」：一个状态里每种操作该有几次。W1 32、W2 32、W3 8、W4 8；探针项同一次挂载多一次 P1；带扩展窗口的点同一次挂载另有
/// W5 200、W6 110（10 批，每批建一次、写十次）、W7 11（第一个文件 1 次、冷文件 5 个、新文件 5 个）；对账点两种状态末尾各一次阴性对照。
const WINDOW_OPERATIONS: [(&str, u64); 4] = [("W1", 32), ("W2", 32), ("W3", 8), ("W4", 8)];
const EXTENDED_WINDOW_OPERATIONS: [(&str, u64); 3] = [("W5", 200), ("W6", 110), ("W7", 11)];
const EXTENDED_WINDOW_POINTS: [&str; 2] = ["files_1000_units_100", "files_30000"];
const NEGATIVE_CONTROL_POINTS: [&str; 3] = ["base", "files_1000", "files_10000"];

fn expected_window(point: &str, probe: &str, state: &str) -> BTreeMap<&'static str, u64> {
    let mut expected: BTreeMap<&'static str, u64> = WINDOW_OPERATIONS.into_iter().collect();
    if state == "same_mount" && probe != "standard" {
        expected.insert("P1", 1);
    }
    if state == "same_mount" && EXTENDED_WINDOW_POINTS.contains(&point) {
        expected.extend(EXTENDED_WINDOW_OPERATIONS);
    }
    if NEGATIVE_CONTROL_POINTS.contains(&point) {
        expected.insert("negative_control", 1);
    }
    expected
}

/// 作废条款第 5 条的一半（读不到不算读到 0）：每个状态里每种操作实际有几次（不同的第几次）与该有的比，交回对不上的 `<状态>/<操作>:<有>≠<该有>`。
fn window_shape_mismatches(point: &PointReadings<'_>) -> Vec<String> {
    let mut mismatches = Vec::new();
    for state in ["same_mount", "remounted"] {
        let mut seen: BTreeMap<&str, u64> = BTreeMap::new();
        for operation in point.operations.iter().filter(|operation| operation.state == state) {
            *seen.entry(operation.operation_name.as_str()).or_default() += 1;
        }
        let expected = expected_window(&point.item.point, &point.item.probe, state);
        let names: BTreeSet<&str> = seen.keys().copied().chain(expected.keys().copied()).collect();
        for name in names {
            let (have, want) = (seen.get(name).copied().unwrap_or(0), expected.get(name).copied().unwrap_or(0));
            if have != want {
                mismatches.push(format!("{state}/{name}:{have}≠{want}"));
            }
        }
    }
    mismatches
}

fn report_item_checks(point: &PointReadings<'_>, out: &mut Lines) {
    let item = point.item;
    let (rewrite_set_shrank, settled_round_unclean) = fixed_point_premise_breaks(&item.window);
    let window_shape = if item.arm == "plain" { Vec::new() } else { window_shape_mismatches(point) };
    let declared_count = item.declared.len();
    let operations = point.operations.len();
    let did_not_settle = item.window.iter().filter(|settle| settle.did_not_settle()).count();
    let at_the_limit = item.window.iter().filter(|settle| settle.rounds >= SETTLING_ROUNDS_LIMIT).count();
    let constants_match = number(&item.constants, "leaf_slots") == Some(LEAF_SLOTS)
        && number(&item.constants, "internal_fanout") == Some(INTERNAL_FANOUT)
        && number(&item.constants, "settling_rounds_at_most") == Some(SETTLING_ROUNDS_LIMIT)
        && number(&item.constants, "disks") == Some(DISKS);
    let negative_rounds = item.window.iter().filter(|settle| settle.operation_name == "negative_control").map(|settle| settle.rounds).max();
    out.push(format!(
        "name=verdict quantity=item_checks item={} item_done={} refused={} declared_lines={declared_count} window_operations={operations} declared_lines_match={} constants_match={} did_not_settle={did_not_settle} rounds_at_the_limit={at_the_limit} clause_violations={} rewrite_set_shrank={rewrite_set_shrank} settled_round_unclean={settled_round_unclean} round_work_mismatches={} window_shape_ok={} window_shape_mismatches={} negative_control_rounds={} negative_control_ok={}",
        item.name,
        item.done,
        item.refused.as_deref().unwrap_or("none"),
        declared_count == operations,
        constants_match || item.arm == "plain",
        item.clause_violations.len(),
        round_work_mismatches(&item.window),
        window_shape.is_empty(),
        if window_shape.is_empty() { "-".to_string() } else { window_shape.join(",") },
        negative_rounds.map_or("not_in_this_point".to_string(), |rounds| rounds.to_string()),
        negative_rounds.is_none_or(|rounds| rounds <= NEGATIVE_CONTROL_ROUNDS_AT_MOST)
    ));
}

/// 第七节 ② 第 3 条：尖峰复现钉。
fn report_spike_pin(points: &Points<'_>, out: &mut Lines) {
    let Some(point) = points.get("files_1000_units_100") else {
        out.push("name=verdict quantity=spike_pin matches=not_run".to_string());
        return;
    };
    let settle_of = |operation_index: u64| point.item.window.iter().find(|settle| settle.state == "same_mount" && settle.operation_name == "W1" && settle.operation_index == operation_index);
    let value = |settle: &Settle, key: &str| match key {
        "rounds" => settle.rounds,
        "final_roles" => settle.final_roles,
        "final_nodes" => settle.final_nodes,
        "round_work" => settle.round_work,
        _ => settle.closure_nodes_added,
    };
    let mut mismatches = Vec::new();
    for (operation_index, pins) in [(1, &SPIKE_PIN_FIRST[..]), (2, &SPIKE_PIN_SECOND[..])] {
        match settle_of(operation_index) {
            Some(settle) => mismatches.extend(
                pins.iter().filter(|(key, pinned)| value(settle, key) != *pinned).map(|(key, pinned)| format!("W1#{operation_index}.{key}={}≠{pinned}", value(settle, key))),
            ),
            None => mismatches.push(format!("W1#{operation_index}=unread")),
        }
    }
    let state = point.item.judgements.get("same_mount/W1/1/").map(|judgement| judgement.state.clone()).unwrap_or_default();
    mismatches.extend(
        SPIKE_PIN_STATE.iter().filter(|(key, pinned)| number(&state, key) != Some(*pinned)).map(|(key, pinned)| format!("state.{key}={:?}≠{pinned}", number(&state, key))),
    );
    out.push(format!(
        "name=verdict quantity=spike_pin matches={} mismatches={}",
        mismatches.is_empty(),
        if mismatches.is_empty() { "-".to_string() } else { mismatches.join(",") }
    ));
}

/// 第七节 ② 第 4 条与 F8：新点每个状态里第一次碰、仍是 K 个单元的 W2 操作，声明的单元差与 `mapped_units` 差之比落在 [0.5, 2]；这种操作至少 5 个。
fn report_declared_against_mapped(points: &Points<'_>, out: &mut Lines) {
    for (name, point) in points.iter().filter(|(_, point)| point.item.units_per_small_file >= LARGE_CHANGE_UNITS_AT_LEAST && point.item.arm == "head") {
        for state in ["same_mount", "remounted"] {
            let in_state: Vec<&Operation> = point.operations.iter().filter(|operation| operation.state == state).collect();
            let mut first_touches = 0;
            let mut outside = Vec::new();
            for (position, operation) in in_state.iter().enumerate() {
                if operation.operation_name != "W2" || operation.old_units != point.item.units_per_small_file {
                    continue;
                }
                first_touches += 1;
                let after = in_state.get(position + 1).and_then(|next| next.mapped_units);
                let ratio = operation.mapped_units.zip(after).and_then(|(before, after)| {
                    (before > after).then(|| (operation.old_units - operation.new_units) as f64 / (before - after) as f64)
                });
                if !ratio.is_some_and(|ratio| (DECLARED_TO_MAPPED_AT_LEAST..=DECLARED_TO_MAPPED_AT_MOST).contains(&ratio)) {
                    outside.push(format!("W2#{}:{}", operation.operation_index, ratio.map_or("unread".to_string(), |ratio| format!("{ratio:.3}"))));
                }
            }
            out.push(format!(
                "name=verdict quantity=declared_against_mapped point={name} state={state} first_touches={first_touches} enough_first_touches={} ratios_outside={} outside={}",
                first_touches >= FIRST_TOUCHES_AT_LEAST,
                outside.len(),
                if outside.is_empty() { "-".to_string() } else { outside.join(",") }
            ));
        }
    }
}

/// 作废条款第 3 条：`plain` 与带计数的构建历史末 `image_sha256` 相同。
fn report_image_pins(points: &Points<'_>, out: &mut Lines) {
    for name in ["base", "files_1000"] {
        let head = points.get(name).and_then(|point| point.item.image_sha256_at_the_history_end.clone());
        let plain = points.get(&format!("{name}__plain")).and_then(|point| point.item.image_sha256_at_the_history_end.clone());
        let verdict = match (&head, &plain) {
            (Some(head), Some(plain)) => (head == plain).to_string(),
            _ => "not_run".to_string(),
        };
        out.push(format!("name=verdict quantity=plain_image point={name} same_image={verdict}"));
    }
}

/// 第七节 ② 第 6 条（第四段）：历史末（量窗第一次操作之前）的 `mapped_units` 落在区间里；另报每点的 user_units ÷ mapped_units（第八节）。
fn report_unit_counts(points: &Points<'_>, out: &mut Lines) {
    for (name, point) in points.iter().filter(|(_, point)| point.item.arm == "head") {
        let history_end = point.operations.iter().find(|operation| operation.state == "same_mount").and_then(|operation| operation.mapped_units);
        let share = history_end.map(|mapped| point.item.units_per_small_file as f64 / mapped as f64);
        out.push(format!(
            "name=occupancy point={name} units_per_small_file={} mapped_units_at_the_history_end={} change_share={} far_below_the_total={}",
            point.item.units_per_small_file,
            history_end.map_or("unread".to_string(), |value| value.to_string()),
            share.map_or("unread".to_string(), |share| format!("{share:.4}")),
            share.map_or("unread".to_string(), |share| (share <= FAR_BELOW_THE_TOTAL_AT_MOST).to_string())
        ));
        if let Some((_, low, high)) = BIG_POINT_MAPPED_UNITS.iter().find(|(big, _, _)| *big == name.as_str()) {
            out.push(format!(
                "name=verdict quantity=big_point_units point={name} mapped_units={} inside={}",
                history_end.map_or("unread".to_string(), |value| value.to_string()),
                history_end.map_or("not_run".to_string(), |value| (*low..=*high).contains(&value).to_string())
            ));
        }
    }
}

/// 第七节 ② 第 2 条：今天的树对 E206 的 p2a 产物，三个对账点 W1–W4 每个固定点（按状态、操作、第几次、第几次判对上）的五项逐个相同。
fn report_reconciliation(points: &Points<'_>, retro: &Points<'_>, out: &mut Lines) {
    for name in ["base", "files_1000", "files_10000"] {
        let (Some(today), Some(before)) = (points.get(name), retro.get(name)) else {
            out.push(format!("name=verdict quantity=reconciliation_with_e206 point={name} matches=not_run"));
            continue;
        };
        let in_window = |settle: &&Settle| matches!(settle.operation_name.as_str(), "W1" | "W2" | "W3" | "W4");
        let keyed = |item: &PointArmReadings| -> BTreeMap<String, (u64, u64, u64, u64, String)> {
            item.window
                .iter()
                .filter(in_window)
                .map(|settle| (settle.publish_key(), (settle.rounds, settle.final_roles, settle.final_nodes, settle.round_work, settle.final_set_hash.clone())))
                .collect()
        };
        let (today_settles, before_settles) = (keyed(today.item), keyed(before.item));
        let keys: BTreeSet<&String> = today_settles.keys().chain(before_settles.keys()).collect();
        let differing: Vec<&String> = keys.into_iter().filter(|key| today_settles.get(*key) != before_settles.get(*key)).collect();
        out.push(format!(
            "name=verdict quantity=reconciliation_with_e206 point={name} settles_today={} settles_e206={} differing={} matches={} first_differing={}",
            today_settles.len(),
            before_settles.len(),
            differing.len(),
            differing.is_empty() && !today_settles.is_empty(),
            differing.first().map_or("-", |key| key.as_str())
        ));
    }
}

/// 回扫（Q4）的另一份：`data_units` 轴 W2 第 1 次（同一次挂载）步数 ÷ user_units，标 `looser=yes`，同时报原步数的 α。
fn report_retro_data_units_first_cold(retro: &Points<'_>, out: &mut Lines) {
    let ends: Vec<Option<End>> = RETRO_SETS[1]
        .points
        .iter()
        .map(|name| {
            let point = retro.get(*name)?;
            let operation = point.operations.iter().find(|operation| operation.state == "same_mount" && operation.operation_name == "W2" && operation.operation_index == 1)?;
            Some((operation.steps?, denominator(operation)?, measured_total(operation)?))
        })
        .collect();
    let reading = ends[0].zip(ends[3]).map(|(low, high)| two_point_reading(low, high));
    out.push(format!(
        "name=q4_data_units_first_cold_overwrite ends={} looser={} alpha={} band={} raw_alpha={} raw_band={}",
        ends.iter().map(|end| end.map_or("unread".to_string(), |end| format!("{}/{}/{}", end.0, end.1, end.2))).collect::<Vec<_>>().join(","),
        reading.map_or("unread", |reading| if reading.looser { "yes" } else { "no" }),
        alpha_text(reading.and_then(|reading| reading.alpha)),
        band_text(reading.and_then(|reading| reading.alpha)),
        alpha_text(reading.and_then(|reading| reading.raw_alpha)),
        band_text(reading.and_then(|reading| reading.raw_alpha))
    ));
}

/// 第七节 ② 第 1 条：照旧判据（旋钮 x、原步数）重算两行，与登记抄下的逐字符比；交回两行都相同没有。
/// 旧判据的一行（E206 合并报告 `q8_*` 行的写法，轴位置 10、100、1000、10000）：四个点的量，读不到的写 `unread`。
fn old_criterion_line(name: &str, axis: &str, values: &[Option<f64>; 4]) -> String {
    let slopes: Vec<String> = (0..3)
        .map(|index| match (values[index], values[index + 1]) {
            (Some(low), Some(high)) if low > 0.0 && high > 0.0 => format!("{:.3}", alpha(low, high, AXIS_POSITIONS[index], AXIS_POSITIONS[index + 1])),
            _ => "unread".to_string(),
        })
        .collect();
    let growth = match (values[0], values[3]) {
        (Some(low), Some(high)) if low > 0.0 && high > 0.0 => Some(alpha_on_the_axis(low, high)),
        _ => None,
    };
    format!(
        "name={name} arm=p2a axis={axis} low_end={} high_end={} alpha={} band={} local_slopes={}",
        values[0].map_or("unread".to_string(), |value| format!("{value:.0}")),
        values[3].map_or("unread".to_string(), |value| format!("{value:.0}")),
        growth.map_or("unread".to_string(), |value| format!("{value:.3}")),
        growth.map_or("unread", |value| band(value).tag()),
        slopes.join(",")
    )
}

/// 第七节 ② 第 1 条：照旧判据（旋钮 x、原步数）重算两行，与登记抄下的逐字符比；交回两行都相同没有。
fn old_criterion_pins(retro_items: &[PointArmReadings], out: &mut Lines) -> bool {
    let items: ReadingsByPointAndArm<'_> = retro_items.iter().map(|item| ((item.point.clone(), item.arm.clone()), item)).collect();
    let values_of = |axis: &str, quantity: &dyn Fn(&PointArmReadings) -> Option<f64>| -> [Option<f64>; 4] {
        let points = AXES.iter().find(|(candidate, _)| *candidate == axis).map(|(_, points)| *points).expect("轴在表里");
        let mut values = [None; 4];
        for (value, point) in values.iter_mut().zip(points.iter()) {
            *value = items.get(&((*point).to_string(), "p2a".to_string())).and_then(|item| quantity(item));
        }
        values
    };
    let first = old_criterion_line(
        "q8_first_publish",
        "files",
        &values_of("files", &|item| window_settle(item, "W1", 1).and_then(|settle| by_delta_of_settle(item, settle)).map(|steps| steps as f64)),
    );
    let peak = old_criterion_line(
        "q8_peak",
        "data_units",
        &values_of("data_units", &|item| item.window.iter().filter_map(|settle| by_delta_of_settle(item, settle)).max().map(|steps| steps as f64)),
    );
    let (first_matches, peak_matches) = (first == OLD_CRITERION_FIRST_PUBLISH_FILES, peak == OLD_CRITERION_PEAK_DATA_UNITS);
    out.push(first);
    out.push(peak);
    out.push(format!("name=verdict quantity=old_criterion_pins first_publish_files_matches={first_matches} peak_data_units_matches={peak_matches}"));
    first_matches && peak_matches
}

// ---- 复量段 hundred_units_per_file_repeat（登记设计员修订 2）----
/// 复量序列的六个点，总量由低到高；高端 `files_1000_units_100` 读第一段的产物，别的五点（含 `files_100_units_100` 再跑一次）读复量段的产物。
const HUNDRED_UNITS_PER_FILE_SERIES: [&str; 6] =
    ["files_56_units_100", "files_100_units_100", "files_178_units_100", "files_316_units_100", "files_562_units_100", "files_1000_units_100"];
/// 公共窗：六点都有的量窗操作（高端的扩展窗口 W5–W7 低端没有，不进公共窗）。
const COMMON_WINDOW_OPERATIONS: [&str; 4] = ["W1", "W2", "W3", "W4"];
/// 序列至少读到几个点才判；少于它记 `not_enough_points`。
const SERIES_POINTS_AT_LEAST: usize = 3;
/// 稳的判语里「随」与「不随」要几段相邻段（五段里）。
const SERIES_ADJACENT_SEGMENTS_AT_LEAST: usize = 3;
/// 阳性对照 `C1`、`C3` 在公共窗的 β 至少多少（读法 4 的「随」）。
const SERIES_CONTROL_BETA_AT_LEAST: f64 = BAND_GROWS_AT_LEAST;

/// 普通最小二乘斜率：ln Q 对 ln x，各点同权；点是 (Q, x)。少于两点或有不是正数的交 `None`。
fn least_squares_slope(points: &[(f64, f64)]) -> Option<f64> {
    if points.len() < 2 || points.iter().any(|(quantity, position)| *quantity <= 0.0 || *position <= 0.0) {
        return None;
    }
    let count = points.len() as f64;
    let mean_position = points.iter().map(|(_, position)| position.ln()).sum::<f64>() / count;
    let mean_quantity = points.iter().map(|(quantity, _)| quantity.ln()).sum::<f64>() / count;
    let covariance: f64 = points.iter().map(|(quantity, position)| (position.ln() - mean_position) * (quantity.ln() - mean_quantity)).sum();
    let variance: f64 = points.iter().map(|(_, position)| (position.ln() - mean_position).powi(2)).sum();
    // 各点的 Q 相同时，均值的舍入误差会让协方差落在 ±1e-16 量级而不是恰为 0；比打印精度（三位小数）小得多的斜率
    // 统一写正零，否则打出来是 -0.000。
    (variance > 0.0).then(|| covariance / variance).map(|slope| if slope.abs() < 1e-9 { 0.0 } else { slope })
}

/// 一条序列的读数：读到的点数、相邻段的 α、β、随 / 介于 / 不随各几段（`unread` 是算不出 α 的段数）与判语。
#[derive(Clone, Debug, PartialEq)]
struct SeriesReading {
    points: usize,
    alphas: Vec<Option<f64>>,
    beta: Option<f64>,
    grows: usize,
    between: usize,
    does_not_grow: usize,
    unread: usize,
    verdict: &'static str,
}

/// 判语（登记修订 2 的四行加 `not_enough_points`）：`g` = 随的段数、`d` = 不随的段数。
fn series_verdict(points: usize, beta: Option<f64>, grows: usize, does_not_grow: usize) -> &'static str {
    let Some(beta) = beta.filter(|_| points >= SERIES_POINTS_AT_LEAST) else {
        return "not_enough_points";
    };
    if beta >= BAND_GROWS_AT_LEAST && grows >= SERIES_ADJACENT_SEGMENTS_AT_LEAST && does_not_grow == 0 {
        "stable_grows"
    } else if beta <= BAND_DOES_NOT_GROW_AT_MOST && does_not_grow >= SERIES_ADJACENT_SEGMENTS_AT_LEAST && grows == 0 {
        "stable_does_not_grow"
    } else if beta > BAND_DOES_NOT_GROW_AT_MOST && beta < BAND_GROWS_AT_LEAST && grows == 0 && does_not_grow == 0 {
        "stable_between"
    } else {
        "unstable"
    }
}

/// 一列 (Q, x)（x 由低到高）的序列读数。
fn series_reading(points: &[(f64, f64)]) -> SeriesReading {
    let alphas = local_slopes(points);
    let beta = least_squares_slope(points);
    let segments_in = |wanted: Band| alphas.iter().flatten().filter(|value| band(**value) == wanted).count();
    let (grows, between, does_not_grow) = (segments_in(Band::Grows), segments_in(Band::Between), segments_in(Band::DoesNotGrow));
    SeriesReading {
        points: points.len(),
        verdict: series_verdict(points.len(), beta, grows, does_not_grow),
        unread: alphas.iter().filter(|value| value.is_none()).count(),
        alphas,
        beta,
        grows,
        between,
        does_not_grow,
    }
}

/// 阳性对照：`C1`、`C3` 在公共窗的 β 都要 ≥ 0.5，不是（或读不到）主格的判语作废。
fn controls_are_red(scan_beta: Option<f64>, linear_beta: Option<f64>) -> bool {
    scan_beta.zip(linear_beta).is_some_and(|(scan, linear)| scan >= SERIES_CONTROL_BETA_AT_LEAST && linear >= SERIES_CONTROL_BETA_AT_LEAST)
}

/// 一个点一个状态一档一种取法一条臂取出来的那一端，只在 `common` 为真时丢掉公共窗之外的操作（W5–W7、P1、阴性对照）；读不到交 `None`。
fn end_among(point: &PointReadings<'_>, state: &str, tier: &str, pick: Pick, steady: bool, arm: Arm, common: bool) -> Option<(End, Option<f64>)> {
    let in_state: Vec<&Operation> = point
        .operations
        .iter()
        .filter(|operation| operation.state == state)
        .filter(|operation| !common || COMMON_WINDOW_OPERATIONS.contains(&operation.operation_name.as_str()))
        .collect();
    let linear_row = largest_lg_row(&in_state);
    let wanted = |operation: &Operation| in_the_tier(operation, tier, point.item.units_per_small_file) && kept_for_the_filter(operation, steady);
    let series: Vec<(&Operation, u64)> = in_state
        .iter()
        .filter(|operation| wanted(operation))
        .filter_map(|operation| arm_steps(operation, arm, linear_row).map(|steps| (*operation, steps)))
        .collect();
    let (operation, steps) = pick_from(&series, pick)?;
    let user_units = denominator(operation)?;
    Some(((steps, user_units, measured_total(operation)?), measured_total_by_mapped_units(operation)))
}

/// 序列里这个点这个状态用不用：做完了（有 `item_done`），且第一次碰、仍是 K 个单元的 W2 操作至少 5 个（F8）。
fn point_is_usable_in_the_series(point: &PointReadings<'_>, state: &str) -> bool {
    let first_touches = point
        .operations
        .iter()
        .filter(|operation| operation.state == state && operation.operation_name == "W2" && operation.old_units == point.item.units_per_small_file)
        .count();
    point.item.done && first_touches >= FIRST_TOUCHES_AT_LEAST
}

/// 一格的序列（六点里读得到的那几个）：每点的一端（读不到的 `None`）、序列读数、`mapped_units` 作 x 的 β。
fn series_of_the_cell(points: &Points<'_>, state: &str, tier: &str, pick: Pick, steady: bool, arm: Arm, common: bool) -> (Vec<Option<End>>, SeriesReading, Option<f64>) {
    let ends: Vec<Option<(End, Option<f64>)>> = HUNDRED_UNITS_PER_FILE_SERIES
        .iter()
        .map(|name| points.get(*name).filter(|point| point_is_usable_in_the_series(point, state)).and_then(|point| end_among(point, state, tier, pick, steady, arm, common)))
        .collect();
    let read: Vec<(End, Option<f64>)> = ends.iter().flatten().copied().collect();
    let looser = read.iter().map(|(end, _)| end.1).collect::<BTreeSet<u64>>().len() > 1;
    let quantity = |end: End| if looser { end.0 as f64 / end.1 as f64 } else { end.0 as f64 };
    let by_total: Vec<(f64, f64)> = read.iter().map(|(end, _)| (quantity(*end), end.2)).collect();
    let by_mapped: Vec<(f64, f64)> = read.iter().filter_map(|(end, mapped)| mapped.map(|position| (quantity(*end), position))).collect();
    let mapped_beta = (by_mapped.len() == read.len()).then(|| least_squares_slope(&by_mapped)).flatten();
    (ends.iter().map(|end| end.map(|(end, _)| end)).collect(), series_reading(&by_total), mapped_beta)
}

/// 序列读数里相邻段 α 的文字：读不到的写 `unread`。
fn alphas_text(reading: &SeriesReading) -> String {
    if reading.alphas.is_empty() {
        return "-".to_string();
    }
    reading.alphas.iter().map(|value| alpha_text(*value)).collect::<Vec<_>>().join(",")
}

/// 去向（登记修订 2 的去向表）：主格的判语、`series_void` 与读到的点数对岔路单第 10 行意味着什么。
fn series_route(verdict: &str, series_void: &str) -> &'static str {
    if series_void != "-" || verdict == "not_enough_points" {
        return "series_not_made";
    }
    match verdict {
        "stable_grows" => "supports_the_flip_on_the_common_window",
        "stable_between" | "stable_does_not_grow" => "does_not_support_the_flip",
        _ => "unstable_the_cell_stays_open",
    }
}

/// 按操作取出两份产物的每一次固定点的六样（`rounds`、`final_roles`、`final_nodes`、`round_work`、`final_set_hash`、`previous_records`），只取 W1–W4；
/// 键是 `<状态>/<操作>/<第几次>/<第几次判>`。
type SettleReadings = (u64, u64, u64, u64, String, u64);

fn window_settle_readings(item: &PointArmReadings) -> Vec<(String, SettleReadings)> {
    item.window
        .iter()
        .filter(|settle| COMMON_WINDOW_OPERATIONS.contains(&settle.operation_name.as_str()))
        .map(|settle| {
            (
                settle.publish_key(),
                (settle.rounds, settle.final_roles, settle.final_nodes, settle.round_work, settle.final_set_hash.clone(), settle.previous_records),
            )
        })
        .collect()
}

/// 这一份产物的 W1–W4 固定点对另一份同名的：(这一份有几次, 不同的键，次序照这一份、另一份独有的接在后面)。
fn settles_against(today: &PointArmReadings, before: &PointArmReadings) -> (usize, Vec<String>) {
    let (today_settles, before_settles) = (window_settle_readings(today), window_settle_readings(before));
    let before_by_key: BTreeMap<&String, &SettleReadings> = before_settles.iter().map(|(key, readings)| (key, readings)).collect();
    let today_keys: BTreeSet<&String> = today_settles.iter().map(|(key, _)| key).collect();
    let mut differing: Vec<String> = today_settles
        .iter()
        .filter(|(key, readings)| before_by_key.get(key) != Some(&readings))
        .map(|(key, _)| key.clone())
        .collect();
    differing.extend(before_settles.iter().map(|(key, _)| key).filter(|key| !today_keys.contains(key)).cloned());
    (today_settles.len(), differing)
}

fn first_or_dash(keys: &[String]) -> &str {
    keys.first().map_or("-", String::as_str)
}

/// 钉（失败条款 ②）：复量段再跑的 `files_100_units_100` 对第一段同名产物，W1–W4 每个固定点六样逐个相同。
fn report_hundred_units_per_file_pin(points: &Points<'_>, pin_against: &[PointArmReadings], out: &mut Lines) -> bool {
    let (Some(rerun), Some(first)) = (points.get("files_100_units_100"), pin_against.iter().find(|item| item.point == "files_100_units_100" && item.arm == "head")) else {
        out.push("name=verdict quantity=hundred_units_per_file_pin matches=not_run".to_string());
        return true;
    };
    let (settles, differing) = settles_against(rerun.item, first);
    out.push(format!(
        "name=verdict quantity=hundred_units_per_file_pin settles={settles} differing={} matches={} first_differing={}",
        differing.len(),
        differing.is_empty() && settles > 0,
        first_or_dash(&differing)
    ));
    differing.is_empty() && settles > 0
}

/// 序列用到的产物（复量段的、第一段高端的、钉的）头里三份被测核心文件的 sha256 逐个相同（作废条款 ①）；一份产物缺这三行记 `not_run`。
fn report_sources_agree(items: &[&PointArmReadings], out: &mut Lines) -> bool {
    let missing = items.iter().filter(|item| item.source_sha256.len() < 3).count();
    let mut values: BTreeMap<&String, BTreeSet<&String>> = BTreeMap::new();
    for item in items {
        for (file, sha) in &item.source_sha256 {
            values.entry(file).or_default().insert(sha);
        }
    }
    let agree = missing == 0 && !values.is_empty() && values.values().all(|shas| shas.len() == 1);
    out.push(format!(
        "name=verdict quantity=sources_agree products={} products_without_the_three_lines={missing} files={} files_with_more_than_one_value={} agree={}",
        items.len(),
        values.len(),
        values.values().filter(|shas| shas.len() > 1).count(),
        if missing > 0 { "not_run".to_string() } else { agree.to_string() }
    ));
    agree
}

/// 复量段（修订 2）的每一格、阳性对照、判语与去向；交回钉与来源是不是都过了。
fn report_hundred_units_per_file_series(points: &Points<'_>, pin_against: &[PointArmReadings], out: &mut Lines) -> bool {
    let pin_matches = report_hundred_units_per_file_pin(points, pin_against, out);
    let used: Vec<&PointArmReadings> = HUNDRED_UNITS_PER_FILE_SERIES.iter().filter_map(|name| points.get(*name).map(|point| point.item)).chain(pin_against.iter()).collect();
    let sources_agree = report_sources_agree(&used, out);
    let mut primary: BTreeMap<&'static str, SeriesReading> = BTreeMap::new();
    for state in ["same_mount", "remounted"] {
        for tier in ["one_user_unit", "user_units_equal_declared"] {
            for pick in PICKS {
                for steady in [false, true] {
                    for (window, common) in [("common_window", true), ("own_window", false)] {
                        for arm in ARMS {
                            let (ends, reading, mapped_beta) = series_of_the_cell(points, state, tier, pick, steady, arm, common);
                            let is_primary = common && state == "same_mount" && tier == "one_user_unit" && pick == Pick::LaterPeak && steady;
                            if is_primary {
                                primary.insert(arm.tag(), reading.clone());
                            }
                            out.push(format!(
                                "name=series_cell state={state} tier={tier} pick={} filter={} window={window} arm={} primary={} points={} ends={} alphas={} beta={} beta_band={} grows={} between={} does_not_grow={} unread_segments={} beta_by_mapped_units={} verdict={}",
                                pick.tag(),
                                if steady { "steady" } else { "all" },
                                arm.tag(),
                                if is_primary && arm == Arm::Tested { "yes" } else { "no" },
                                reading.points,
                                ends.iter().map(|end| end.map_or("unread".to_string(), |end| format!("{}/{}/{}", end.0, end.1, end.2))).collect::<Vec<_>>().join(","),
                                alphas_text(&reading),
                                alpha_text(reading.beta),
                                band_text(reading.beta),
                                reading.grows,
                                reading.between,
                                reading.does_not_grow,
                                reading.unread,
                                alpha_text(mapped_beta),
                                reading.verdict
                            ));
                        }
                    }
                }
            }
        }
    }
    let tested = primary.get(Arm::Tested.tag()).cloned();
    let controls_red = controls_are_red(
        primary.get(Arm::ScanEveryPublish.tag()).and_then(|reading| reading.beta),
        primary.get(Arm::LargestLgRowLinear.tag()).and_then(|reading| reading.beta),
    );
    let series_void = if !pin_matches {
        "pin_differs"
    } else if !sources_agree {
        if used.iter().any(|item| item.source_sha256.len() < 3) { "sources_unread" } else { "sources_differ" }
    } else if !controls_red {
        "control_not_red"
    } else {
        "-"
    };
    let verdict = tested.as_ref().map_or("not_enough_points", |reading| reading.verdict);
    out.push(format!(
        "name=verdict quantity=hundred_units_per_file_series outcome={verdict} series_void={series_void} points={} route={}",
        tested.as_ref().map_or(0, |reading| reading.points),
        series_route(verdict, series_void)
    ));
    pin_matches && sources_agree
}

// ---- 对账分辨段 reconciliation_split（登记设计员修订 3）----
/// 逐操作取镜像与对账的两个点（`head` 与 `plain` 两个构建都跑）；`base` 是阴性对照。
const SPLIT_POINTS: [&str; 2] = ["base", "files_1000"];
/// 池自己的 `name=state` 行对 E206 的三个点（`files_10000` 只做这一行，离线，不重跑）。
const STATE_ROW_POINTS: [&str; 3] = ["base", "files_1000", "files_10000"];
/// `state_rows_against_e206` 比的四个池字段。
const POOL_STATE_FIELDS: [&str; 4] = ["allocation_records", "cluster_segments", "mapped_units", "units"];

fn operation_label(state: &str, operation_name: &str, operation_index: u64) -> String {
    format!("{state}/{operation_name}/{operation_index}")
}

/// `head_against_plain`：逐操作比两个构建的镜像哈希，重挂载前后各是否相同。
#[derive(Clone, Debug, PartialEq)]
struct HeadAgainstPlain {
    operations: usize,
    differing: usize,
    first_differing: String,
    before_the_crash_same: bool,
    after_the_mount_same: bool,
}

fn head_against_plain(head: &PointArmReadings, plain: &PointArmReadings) -> Option<HeadAgainstPlain> {
    if head.images_after_the_operations.is_empty() || plain.images_after_the_operations.is_empty() {
        return None;
    }
    let plain_by_operation: BTreeMap<&(String, String, u64), &String> = plain.images_after_the_operations.iter().map(|(key, sha)| (key, sha)).collect();
    let head_operations: BTreeSet<&(String, String, u64)> = head.images_after_the_operations.iter().map(|(key, _)| key).collect();
    let mut differing: Vec<String> = head
        .images_after_the_operations
        .iter()
        .filter(|(key, sha)| plain_by_operation.get(key) != Some(&sha))
        .map(|((state, operation_name, operation_index), _)| operation_label(state, operation_name, *operation_index))
        .collect();
    differing.extend(
        plain
            .images_after_the_operations
            .iter()
            .filter(|(key, _)| !head_operations.contains(key))
            .map(|((state, operation_name, operation_index), _)| operation_label(state, operation_name, *operation_index)),
    );
    let same_at = |when: &str| matches!((head.images_at_the_remount.get(when), plain.images_at_the_remount.get(when)), (Some(left), Some(right)) if left == right);
    Some(HeadAgainstPlain {
        operations: head.images_after_the_operations.len(),
        first_differing: first_or_dash(&differing).to_string(),
        differing: differing.len(),
        before_the_crash_same: same_at("before_the_crash"),
        after_the_mount_same: same_at("after_the_mount"),
    })
}

/// `placement_gap`：每个操作「计数记的落点槽数 − 盘上变了的单元槽数」的符号分布。
#[derive(Clone, Debug, PartialEq)]
struct PlacementGap {
    operations: usize,
    negative: usize,
    zero: usize,
    positive: usize,
    first_negative: String,
}

fn placement_gap(head: &PointArmReadings) -> Option<PlacementGap> {
    if head.reconciliation_rows.is_empty() {
        return None;
    }
    let rows = &head.reconciliation_rows;
    let first_negative = rows
        .iter()
        .find(|(_, (changed, placed))| placed < changed)
        .map_or("-".to_string(), |((state, operation_name, operation_index), _)| operation_label(state, operation_name, *operation_index));
    Some(PlacementGap {
        operations: rows.len(),
        negative: rows.iter().filter(|(_, (changed, placed))| placed < changed).count(),
        zero: rows.iter().filter(|(_, (changed, placed))| placed == changed).count(),
        positive: rows.iter().filter(|(_, (changed, placed))| placed > changed).count(),
        first_negative,
    })
}

/// 五项（`rounds`、`final_roles`、`final_nodes`、`round_work`、`final_set_hash`）在两份产物里逐个固定点不同的键，按键排序（同 `report_reconciliation` 的 `first_differing`）。
fn first_differing_publish(today: &PointArmReadings, before: &PointArmReadings) -> Option<String> {
    let keyed = |item: &PointArmReadings| -> BTreeMap<String, (u64, u64, u64, u64, String)> {
        window_settle_readings(item).into_iter().map(|(key, readings)| (key, (readings.0, readings.1, readings.2, readings.3, readings.4))).collect()
    };
    let (today_settles, before_settles) = (keyed(today), keyed(before));
    let keys: BTreeSet<String> = today_settles.keys().chain(before_settles.keys()).cloned().collect();
    keys.into_iter().find(|key| today_settles.get(key) != before_settles.get(key))
}

/// 一份产物里键是 `key`（`<状态>/<操作>/<第几次>/<第几次判>`）的固定点。
fn settle_named<'items>(item: &'items PointArmReadings, key: &str) -> Option<&'items Settle> {
    item.window.iter().find(|settle| settle.publish_key() == key)
}

/// 这个键所在那次操作之前的池状态行（键去掉最后一段「第几次判」）。
fn state_of_the_publish(item: &PointArmReadings, key: &str) -> Option<Fields> {
    let (operation_prefix, _) = key.rsplit_once('/')?;
    item.judgements.get(&format!("{operation_prefix}/")).map(|judgement| judgement.state.clone())
}

/// `state_rows_against_e206`：池自己的 `name=state` 行（四个字段）今天对 E206 逐个量窗操作。
#[derive(Clone, Debug, PartialEq)]
struct StateRows {
    operations: usize,
    differing_operations: usize,
    by_field: [usize; 4],
    same_mount_differing_operations: usize,
    first_differing: String,
    /// 五项里第一处不同的发布上，`previous_records`、`previous_nodes`、`tree_height` 与池的 `name=state` 行在两边是否相同；没有这样的发布是 `None`。
    inputs_equal_at_the_first_differing_publish: Option<bool>,
}

fn state_rows_against_e206(today: &PointReadings<'_>, before: &PointReadings<'_>) -> Option<StateRows> {
    let state_of = |item: &PointArmReadings, operation: &Operation| -> Option<Fields> {
        item.judgements.get(&format!("{}/{}/{}/", operation.state, operation.operation_name, operation.operation_index)).map(|judgement| judgement.state.clone())
    };
    let compared: Vec<(&Operation, Fields, Fields)> = today
        .operations
        .iter()
        .filter_map(|operation| Some((operation, state_of(today.item, operation).filter(|fields| !fields.is_empty())?, state_of(before.item, operation).filter(|fields| !fields.is_empty())?)))
        .collect();
    if compared.is_empty() || compared.iter().any(|(_, _, old)| number(old, "cluster_segments").is_none() || number(old, "allocation_records").is_none()) {
        return None;
    }
    let differs_in = |new: &Fields, old: &Fields, field: &str| number(new, field) != number(old, field);
    let differing: Vec<&(&Operation, Fields, Fields)> = compared.iter().filter(|(_, new, old)| POOL_STATE_FIELDS.iter().any(|field| differs_in(new, old, field))).collect();
    let mut by_field = [0; 4];
    for (slot, field) in by_field.iter_mut().zip(POOL_STATE_FIELDS) {
        *slot = compared.iter().filter(|(_, new, old)| differs_in(new, old, field)).count();
    }
    let inputs_equal = first_differing_publish(today.item, before.item).map(|key| {
        let same_settle = settle_named(today.item, &key)
            .zip(settle_named(before.item, &key))
            .is_some_and(|(new, old)| (new.previous_records, new.previous_nodes, new.tree_height) == (old.previous_records, old.previous_nodes, old.tree_height));
        let same_state = state_of_the_publish(today.item, &key)
            .zip(state_of_the_publish(before.item, &key))
            .is_some_and(|(new, old)| POOL_STATE_FIELDS.iter().all(|field| !differs_in(&new, &old, field)));
        same_settle && same_state
    });
    Some(StateRows {
        operations: compared.len(),
        differing_operations: differing.len(),
        by_field,
        same_mount_differing_operations: differing.iter().filter(|(operation, _, _)| operation.state == "same_mount").count(),
        first_differing: differing.first().map_or("-".to_string(), |(operation, _, _)| operation_label(&operation.state, &operation.operation_name, operation.operation_index)),
        inputs_equal_at_the_first_differing_publish: inputs_equal,
    })
}

/// 总判要的四个谓词与五项还有没有不同（修订 3「谓词与结局」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SplitFacts {
    /// `c_head`：`head_against_plain` 的 `differing > 0` 或镜像重挂载前后任一不同。
    head_differs: bool,
    /// `c_gap`：`placement_gap` 的 `negative > 0`。
    gap_negative: bool,
    /// `c_rerun`：`rerun_against_the_first_segment` 的 `differing > 0`。
    rerun_differs: bool,
    /// `t_sign`：池结构的状态行有不同、同一次挂载里一处不差、`base` 一处不差。
    signature_of_a_changed_tree: bool,
    /// 五项（对 E206 的对账）仍有不同。
    five_items_differ: bool,
}

/// 结局表（次序就是判的次序）；读不到的交 `not_run`。表里六行之外的一种：没有任何谓词为真、五项也不再有不同，记 `nothing_left_to_split`。
fn reconciliation_split_outcome(facts: Option<SplitFacts>) -> &'static str {
    let Some(facts) = facts else { return "not_run" };
    let counting = facts.head_differs || facts.gap_negative;
    if counting && !facts.signature_of_a_changed_tree {
        "counting_wrong"
    } else if counting && facts.signature_of_a_changed_tree {
        "both"
    } else if !facts.rerun_differs && facts.signature_of_a_changed_tree {
        "tree_changed"
    } else if !facts.rerun_differs && facts.five_items_differ {
        "neither_reading_differs"
    } else if facts.rerun_differs {
        "neither_not_reproducible"
    } else {
        "nothing_left_to_split"
    }
}

/// 一个点的四行读数（交回：谓词与五项有没有不同；读不全交 `None`）。
struct PointSplitReadings {
    head_differs: Option<bool>,
    gap_negative: Option<bool>,
    rerun_differs: Option<bool>,
    state_rows: Option<StateRows>,
    five_items_differ: Option<bool>,
}

fn report_head_against_plain(point: &str, reading: &Option<HeadAgainstPlain>, out: &mut Lines) {
    match reading {
        Some(reading) => out.push(format!(
            "name=verdict quantity=head_against_plain point={point} operations={} differing={} first_differing={} remount_before_the_crash_same={} remount_after_the_mount_same={}",
            reading.operations, reading.differing, reading.first_differing, reading.before_the_crash_same, reading.after_the_mount_same
        )),
        None => out.push(format!("name=verdict quantity=head_against_plain point={point} operations=not_run")),
    }
}

fn report_placement_gap(point: &str, reading: &Option<PlacementGap>, out: &mut Lines) {
    match reading {
        Some(reading) => out.push(format!(
            "name=verdict quantity=placement_gap point={point} operations={} negative={} first_negative={} zero={} positive={}",
            reading.operations, reading.negative, reading.first_negative, reading.zero, reading.positive
        )),
        None => out.push(format!("name=verdict quantity=placement_gap point={point} operations=not_run")),
    }
}

fn report_rerun(point: &str, reading: &Option<(usize, Vec<String>)>, out: &mut Lines) {
    match reading {
        Some((settles, differing)) => out.push(format!(
            "name=verdict quantity=rerun_against_the_first_segment point={point} settles={settles} differing={} first_differing={}",
            differing.len(),
            first_or_dash(differing)
        )),
        None => out.push(format!("name=verdict quantity=rerun_against_the_first_segment point={point} settles=not_run")),
    }
}

fn report_state_rows(point: &str, reading: &Option<StateRows>, out: &mut Lines) {
    match reading {
        Some(rows) => out.push(format!(
            "name=verdict quantity=state_rows_against_e206 point={point} operations={} differing_operations={} by_field={} same_mount_differing_operations={} first_differing={} inputs_equal_at_the_first_differing_publish={}",
            rows.operations,
            rows.differing_operations,
            POOL_STATE_FIELDS.iter().zip(rows.by_field).map(|(field, count)| format!("{field}:{count}")).collect::<Vec<_>>().join(","),
            rows.same_mount_differing_operations,
            rows.first_differing,
            rows.inputs_equal_at_the_first_differing_publish.map_or("-".to_string(), |equal| equal.to_string())
        )),
        None => out.push(format!("name=verdict quantity=state_rows_against_e206 point={point} operations=not_run")),
    }
}

/// 对账分辨段（修订 3）：每点四行读数、`base` 阴性对照与 `files_1000` 的总判。交回阴性对照有没有过（读不到不算没过）。
fn report_reconciliation_split(points: &Points<'_>, previous: &Points<'_>, retro: &Points<'_>, out: &mut Lines) -> bool {
    let mut by_point: BTreeMap<&str, PointSplitReadings> = BTreeMap::new();
    for point in SPLIT_POINTS {
        let head = points.get(point).filter(|reading| reading.item.done);
        let plain = points.get(&format!("{point}__plain")).filter(|reading| reading.item.done);
        let against_plain = head.zip(plain).and_then(|(head, plain)| head_against_plain(head.item, plain.item));
        let gap = head.and_then(|head| placement_gap(head.item));
        let rerun = head.zip(previous.get(point)).map(|(head, before)| settles_against(head.item, before.item));
        report_head_against_plain(point, &against_plain, out);
        report_placement_gap(point, &gap, out);
        report_rerun(point, &rerun, out);
        by_point.insert(
            point,
            PointSplitReadings {
                head_differs: against_plain.as_ref().map(|reading| reading.differing > 0 || !reading.before_the_crash_same || !reading.after_the_mount_same),
                gap_negative: gap.as_ref().map(|reading| reading.negative > 0),
                rerun_differs: rerun.as_ref().map(|(_, differing)| !differing.is_empty()),
                state_rows: None,
                five_items_differ: head.zip(retro.get(point)).map(|(today, before)| first_differing_publish(today.item, before.item).is_some()),
            },
        );
    }
    for point in STATE_ROW_POINTS {
        let today = if point == "files_10000" { previous.get(point) } else { points.get(point).filter(|reading| reading.item.done) };
        let rows = today.zip(retro.get(point)).and_then(|(today, before)| state_rows_against_e206(today, before));
        report_state_rows(point, &rows, out);
        if let Some(readings) = by_point.get_mut(point) {
            readings.state_rows = rows;
        }
    }
    let base_rows_differ = by_point.get("base").and_then(|readings| readings.state_rows.as_ref()).map(|rows| rows.differing_operations > 0);
    let mut negative_control_ok = true;
    for point in SPLIT_POINTS {
        let Some(readings) = by_point.get(point) else { continue };
        let signature = readings
            .state_rows
            .as_ref()
            .zip(base_rows_differ)
            .map(|(rows, base_differs)| rows.differing_operations > 0 && rows.same_mount_differing_operations == 0 && !base_differs);
        let facts = readings
            .head_differs
            .zip(readings.gap_negative)
            .zip(readings.rerun_differs)
            .zip(signature)
            .zip(readings.five_items_differ)
            .map(|((((head_differs, gap_negative), rerun_differs), signature_of_a_changed_tree), five_items_differ)| SplitFacts {
                head_differs,
                gap_negative,
                rerun_differs,
                signature_of_a_changed_tree,
                five_items_differ,
            });
        if point == "base" {
            // 阴性对照：四个谓词（状态行有没有不同算第四个）全假
            let named_true: Vec<&str> = [
                ("head_differs", readings.head_differs),
                ("gap_negative", readings.gap_negative),
                ("rerun_differs", readings.rerun_differs),
                ("state_rows_differ", base_rows_differ),
            ]
            .into_iter()
            .filter(|(_, value)| *value == Some(true))
            .map(|(name, _)| name)
            .collect();
            let unread = [readings.head_differs, readings.gap_negative, readings.rerun_differs, base_rows_differ].iter().any(Option::is_none);
            let outcome = if unread { "not_run" } else if named_true.is_empty() { "negative_control_clean" } else { "negative_control_red" };
            negative_control_ok &= outcome != "negative_control_red";
            out.push(format!(
                "name=verdict quantity=reconciliation_split point=base outcome={outcome} reason={}",
                if named_true.is_empty() { "-".to_string() } else { named_true.join(",") }
            ));
        } else {
            out.push(format!(
                "name=verdict quantity=reconciliation_split point={point} outcome={} reason={}",
                reconciliation_split_outcome(facts),
                facts.map_or("not_run".to_string(), |facts| format!(
                    "head_differs={},gap_negative={},rerun_differs={},changed_tree_signature={},five_items_differ={}",
                    facts.head_differs, facts.gap_negative, facts.rerun_differs, facts.signature_of_a_changed_tree, facts.five_items_differ
                ))
            ));
        }
    }
    negative_control_ok
}

// ---- 报告 ----
/// 点表里的键：今天的实现（`head`）标准项用点名，探针项与 `plain` 用项名；E206 的 p2a 用点名。
fn point_key(item: &PointArmReadings) -> String {
    if (item.arm == "head" && item.probe == "standard") || item.arm == "p2a" {
        item.point.clone()
    } else {
        item.name.clone()
    }
}

fn points_of(items: &[PointArmReadings]) -> Points<'_> {
    items.iter().map(|item| (point_key(item), PointReadings { item, operations: operations_of(item) })).collect()
}

/// `quantity` 是 (读的是什么, 登记的量号)。
fn report_quantities(
    quantity: (&str, &str),
    segment: &str,
    tier: &str,
    cells: &[JudgedCell],
    sets: &[&str],
    tallies: &BTreeMap<(&'static str, &'static str), ControlTally>,
    out: &mut Lines,
) {
    let chosen: Vec<&JudgedCell> = cells.iter().filter(|cell| cell.tier == tier && sets.contains(&cell.set)).collect();
    report_row_reading(quantity, segment, tier, &chosen, tallies, out);
}

fn report_void(tallies: &BTreeMap<(&'static str, &'static str), ControlTally>, out: &mut Lines) {
    for ((segment, tier), tally) in tallies {
        out.push(format!(
            "name=verdict quantity=void_clauses segment={segment} tier={tier} applicable_cells={} scan_control_red={} linear_control_red={} scan_control_void={} linear_control_void={}",
            tally.applicable,
            tally.scan_red,
            tally.linear_red,
            tally.scan_void(),
            tally.linear_void()
        ));
    }
}

/// 读好的产物（这一段的与回扫的）上逐量报，行推进 `out`；交回第七节 ② 第 1 条的两行都相同没有（没给回扫交 true）。
fn report_items(segment: &str, items: &[PointArmReadings], retro_items: &[PointArmReadings], out: &mut Lines) -> bool {
    let mut points = points_of(items);
    let retro = points_of(retro_items);
    out.push(format!(
        "name=model segment={segment} products={} retro_products={} leaf_slots={LEAF_SLOTS} internal_fanout={INTERNAL_FANOUT} settling_rounds_at_most={SETTLING_ROUNDS_LIMIT} disks={DISKS}",
        items.len(),
        retro_items.len()
    ));
    for point in points.values() {
        report_item_checks(point, out);
    }
    report_operations(&points, out);
    let voided = points_with_a_frozen_publish(&points, out);
    points.retain(|name, _| !voided.contains(name));
    report_missing_the_pooled_fallback_row(&points, out);
    let mut cells = Vec::new();
    match segment {
        "first" => {
            cells = judge_sets(&points, &NEW_POINT_SETS, out);
            report_similar_total_set(&points, out);
            report_spike_pin(&points, out);
            report_image_pins(&points, out);
            report_then_big_ratio(&points, out);
            report_reconciliation(&points, &retro, out);
        }
        "fourth" => cells = judge_sets(&points, &BIG_POINT_SETS, out),
        _ => {}
    }
    report_declared_against_mapped(&points, out);
    report_unit_counts(&points, out);
    report_spike_causes(&points, out);
    if segment == "second" {
        report_reproduction_probes(&points, out);
    }
    let mut pins_match = true;
    if !retro_items.is_empty() {
        cells.extend(judge_sets(&retro, &RETRO_SETS, out));
        report_retro_data_units_first_cold(&retro, out);
        pins_match = old_criterion_pins(retro_items, out);
    }
    report_looser_instances(&cells, out);
    let tallies = control_tallies(&cells);
    report_void(&tallies, out);
    if segment == "first" {
        for (reading, quantity, tier, sets) in NEW_POINT_QUANTITIES {
            report_quantities((reading, quantity), "new", tier, &cells, sets, &tallies, out);
        }
    }
    if segment == "fourth" {
        for tier in ["one_user_unit", "user_units_equal_declared"] {
            report_quantities((BIG_POINT_READING, "q10"), "big", tier, &cells, &["K1000L"], &tallies, out);
        }
    }
    if !retro_items.is_empty() {
        report_quantities((RETRO_READING, "q4"), "retro", "one_user_unit", &cells, &["axis_files", "axis_data_units", "axis_unlinks", "axis_unit_area"], &tallies, out);
    }
    pins_match
}

/// 复量段与对账分辨段在通用的逐点逐量读数（`report_items`）之外多读的东西；交回钉、来源与阴性对照有没有过。
fn report_new_segment(segment: &str, items: &[PointArmReadings], previous: &[PointArmReadings], pin_against: &[PointArmReadings], retro: &[PointArmReadings], out: &mut Lines) -> bool {
    let points = points_of(items);
    match segment {
        "hundred_units_per_file_repeat" => report_hundred_units_per_file_series(&points, pin_against, out),
        _ => {
            let (previous_points, retro_points) = (points_of(previous), points_of(retro));
            let used: Vec<&PointArmReadings> = items.iter().chain(previous.iter()).collect();
            report_sources_agree(&used, out);
            report_reconciliation_split(&points, &previous_points, &retro_points, out)
        }
    }
}

/// 读一串产物路径。
fn read_items(paths: &[String]) -> Result<Vec<PointArmReadings>, String> {
    paths.iter().map(|path| read_item(path)).collect()
}

fn report(segment: &str, products: &[String], retro_products: &[String], previous_products: &[String], pin_products: &[String]) -> Result<bool, String> {
    let items = read_items(products)?;
    let retro_items = read_items(retro_products)?;
    let mut out = Lines::new();
    let pins_match = if matches!(segment, "hundred_units_per_file_repeat" | "reconciliation_split") {
        // 这两段的回扫（E206 的三个 p2a 产物）只给对账分辨用，不进通用的轴上回扫与旧判据钉（那要十六份）
        report_items(segment, &items, &[], &mut out);
        report_new_segment(segment, &items, &read_items(previous_products)?, &read_items(pin_products)?, &retro_items, &mut out)
    } else {
        report_items(segment, &items, &retro_items, &mut out)
    };
    for line in &out {
        println!("{line}");
    }
    Ok(pins_match)
}

fn usage() -> ! {
    eprintln!("用法：e207-strict-scan-steps report --segment first|second|third|fourth <E207 产物>… [--retro <E206 p2a 产物>…]");
    eprintln!("      e207-strict-scan-steps report --segment hundred_units_per_file_repeat <复量段五项产物> <第一段 files_1000_units_100 产物> --pin-against <第一段 files_100_units_100 产物>");
    eprintln!("      e207-strict-scan-steps report --segment reconciliation_split <四项新产物> --previous <第一段 base、files_1000、files_10000 的 head 产物> --retro <E206 的 base、files_1000、files_10000 p2a 产物>");
    eprintln!("      e207-strict-scan-steps old-criterion-pins <E206 p2a 产物>…");
    std::process::exit(2);
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match arguments.first().map(String::as_str) {
        Some("report") => {
            let (Some("--segment"), Some(segment)) = (arguments.get(1).map(String::as_str), arguments.get(2)) else { usage() };
            if !["first", "second", "third", "fourth", "hundred_units_per_file_repeat", "reconciliation_split"].contains(&segment.as_str()) {
                usage();
            }
            // 位置参数是产物；`--retro`、`--previous`、`--pin-against` 各自收后面跟着的路径
            let mut lists: BTreeMap<&str, Vec<String>> = BTreeMap::new();
            let mut current = "products";
            for argument in &arguments[3..] {
                match argument.as_str() {
                    "--retro" => current = "retro",
                    "--previous" => current = "previous",
                    "--pin-against" => current = "pin_against",
                    path => lists.entry(current).or_default().push(path.to_string()),
                }
            }
            let list = |name: &str| lists.get(name).cloned().unwrap_or_default();
            report(segment, &list("products"), &list("retro"), &list("previous"), &list("pin_against"))
        }
        Some("old-criterion-pins") => arguments[1..]
            .iter()
            .map(|path| read_item(path))
            .collect::<Result<Vec<_>, _>>()
            .map(|items| {
                let mut out = Lines::new();
                let same = old_criterion_pins(&items, &mut out);
                for line in &out {
                    println!("{line}");
                }
                same
            }),
        _ => usage(),
    };
    match outcome {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(problem) => {
            eprintln!("{problem}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn operation(steps: u64, user_units: u64, allocation_records: u64) -> Operation {
        Operation { steps: Some(steps), user_units: Some(user_units), allocation_records: Some(allocation_records), ..Operation::default() }
    }

    fn three_decimals(value: Option<f64>) -> String {
        alpha_text(value)
    }

    #[test]
    fn band_edges_sit_where_the_registration_put_them() {
        assert_eq!(three_decimals(positive_alpha(1.0, 1.778, 1.0, 10.0)), "0.250");
        assert_eq!(band(0.25), Band::DoesNotGrow);
        assert_eq!(three_decimals(positive_alpha(1.0, 3.162, 1.0, 10.0)), "0.500");
        assert_eq!(band(0.5), Band::Grows);
        assert_eq!(band(0.4999), Band::Between);
    }

    #[test]
    fn the_spike_reading_of_e206_gives_0_789() {
        assert_eq!(three_decimals(positive_alpha(2102.0, 168_373.0, 922.0, 238_324.0)), "0.789");
    }

    #[test]
    fn constant_steps_do_not_grow() {
        let reading = two_point_reading((500, 1, 1000.0), (500, 1, 10_000.0));
        assert_eq!(reading.alpha, Some(0.0));
        assert_eq!(band(reading.alpha.expect("读得到")), Band::DoesNotGrow);
    }

    #[test]
    fn steps_proportional_to_the_user_units_read_flat_per_unit_and_one_against_the_units() {
        let (low, high) = ((100.0, 10.0), (100_000.0, 10_000.0));
        assert_eq!(three_decimals(positive_alpha(low.0 / low.1, high.0 / high.1, low.1, high.1)), "0.000");
        assert_eq!(three_decimals(positive_alpha(low.0, high.0, low.1, high.1)), "1.000");
    }

    #[test]
    fn equal_user_units_read_raw_steps_and_different_ones_read_per_unit_and_are_looser() {
        let same = two_point_reading((5, 1, 10.0), (50, 1, 100.0));
        assert!(!same.looser);
        assert_eq!(three_decimals(same.alpha), "1.000");
        let different = two_point_reading((5, 1, 10.0), (500, 10, 100.0));
        assert!(different.looser);
        assert_eq!(three_decimals(different.alpha), "1.000");
        assert_eq!(three_decimals(different.raw_alpha), "2.000");
    }

    #[test]
    fn the_scan_control_on_the_one_unit_construction_grows_while_the_tested_arm_does_not() {
        let (low, high) = (operation(4500, 1, 23_923), operation(5500, 1, 239_228));
        let end = |operation: &Operation, arm: Arm| (arm_steps(operation, arm, None).expect("读得到"), 1, measured_total(operation).expect("读得到"));
        assert_eq!(three_decimals(two_point_reading(end(&low, Arm::ScanEveryPublish), end(&high, Arm::ScanEveryPublish)).alpha), "0.935");
        assert_eq!(three_decimals(two_point_reading(end(&low, Arm::Tested), end(&high, Arm::Tested)).alpha), "0.087");
    }

    #[test]
    fn the_scan_control_on_the_many_units_construction_stays_between_and_four_times_grows() {
        let (low, high) = (operation(178_400, 1000, 23_923), operation(178_400, 1000, 239_228));
        let end = |operation: &Operation, arm: Arm| (arm_steps(operation, arm, None).expect("读得到"), 1000, measured_total(operation).expect("读得到"));
        assert_eq!(three_decimals(two_point_reading(end(&low, Arm::ScanEveryPublish), end(&high, Arm::ScanEveryPublish)).alpha), "0.315");
        assert_eq!(
            three_decimals(two_point_reading(end(&low, Arm::ScanEveryPublishFourTimes), end(&high, Arm::ScanEveryPublishFourTimes)).alpha),
            "0.617"
        );
    }

    #[test]
    fn the_linear_control_replaces_lg_by_the_quantity_itself() {
        let mut low = operation(4500, 1, 23_923);
        let mut high = operation(5500, 1, 239_228);
        low.lg_rows.insert("allocator_release_linear_find", (100 * lg(23_923), 100 * 23_923, 100 * lg(23_923)));
        high.lg_rows.insert("allocator_release_linear_find", (128 * lg(239_228), 128 * 239_228, 128 * lg(239_228)));
        let row = Some("allocator_release_linear_find");
        let end = |operation: &Operation| (arm_steps(operation, Arm::LargestLgRowLinear, row).expect("读得到"), 1, measured_total(operation).expect("读得到"));
        assert_eq!(arm_steps(&high, Arm::LargestLgRowLinear, row), Some(5500 - 128 * 18 + 128 * 239_228));
        assert_eq!(three_decimals(two_point_reading(end(&low), end(&high)).alpha), "1.107");
    }

    #[test]
    fn the_linear_control_takes_the_lg_row_with_the_most_steps() {
        let mut one = operation(10, 1, 1);
        one.lg_rows.insert("allocator_release_linear_find", (5, 50, 5));
        one.lg_rows.insert("mapped_units_whole_table", (2, 20, 40));
        let mut other = operation(10, 1, 1);
        other.lg_rows.insert("allocator_release_linear_find", (5, 50, 5));
        assert_eq!(largest_lg_row(&[&one, &other]), Some("mapped_units_whole_table"));
    }

    #[test]
    fn an_lg_term_is_the_lg_part_of_its_row() {
        let input = ByDeltaInput {
            allocation_records: 1000,
            calls: BTreeMap::from([("allocator_release_linear_find".to_string(), 3)]),
            ..ByDeltaInput::default()
        };
        let rows = by_delta_rows(&input);
        let row = rows.iter().find(|(name, _)| *name == "allocator_release_linear_find").expect("行在表里").1;
        let term = lg_terms(&input).into_iter().find(|(name, _, _)| *name == "allocator_release_linear_find").expect("行在表里");
        assert_eq!(row, 30);
        assert_eq!(term.1 * lg(term.2), row);
    }

    #[test]
    fn the_spike_line_takes_the_floor_and_four_times_the_lower_median() {
        assert_eq!(spike_line(&[7, 7, 7, 433, 7]), 28);
        assert!(433 >= spike_line(&[7, 7, 7, 433, 7]) && 7 < spike_line(&[7, 7, 7, 433, 7]));
        assert_eq!(spike_line(&[3, 3, 3, 3, 20]), 28, "四倍中位数只有 12，旗线仍是下限 28，20 不在旗上");
        let with_an_outlier = [7, 7, 7, 7, 7, 7, 7, 433, 60];
        assert_eq!(lower_median(&with_an_outlier), 7);
        assert!(60 >= spike_line(&with_an_outlier), "中位数 7 ⇒ 旗线 28，60 在旗上；均值 60.2 ⇒ 旗线 240，60 不在");
    }

    #[test]
    fn the_first_pick_is_the_first_in_order_and_later_ones_skip_it() {
        let (one, two, three) = (operation(5, 1, 1), operation(50, 1, 1), operation(7, 1, 1));
        let series = [(&one, 5), (&two, 50), (&three, 7)];
        assert_eq!(pick_from(&series, Pick::First).map(|(_, steps)| steps), Some(5));
        assert_eq!(pick_from(&series, Pick::LaterPeak).map(|(_, steps)| steps), Some(50));
        assert_eq!(pick_from(&series, Pick::LaterMedian).map(|(_, steps)| steps), Some(7));
    }

    #[test]
    fn steady_drops_the_flagged_operations_and_all_keeps_them() {
        let flagged = Operation { flagged: true, ..operation(1, 1, 1) };
        let quiet = operation(1, 1, 1);
        assert!(!kept_for_the_filter(&flagged, true));
        assert!(kept_for_the_filter(&flagged, false));
        assert!(kept_for_the_filter(&quiet, true));
    }

    #[test]
    fn the_release_chain_cause_needs_the_release_site_eighty_percent_pulled_and_old_nodes() {
        assert_eq!(release_chain_cause_holds("release_not_covered", 85, 100, 200), (true, Vec::new()));
        assert_eq!(release_chain_cause_holds("placement_not_covered", 85, 100, 200).1, vec!["site"]);
        assert_eq!(release_chain_cause_holds("release_not_covered", 79, 100, 200).1, vec!["pulled_share"]);
        assert_eq!(release_chain_cause_holds("release_not_covered", 85, 100, 99).1, vec!["age_median"]);
    }

    #[test]
    fn three_points_report_adjacent_slopes_not_the_end_to_end_one() {
        let slopes = local_slopes(&[(1.0, 1.0), (2.0, 10.0), (100.0, 100.0)]);
        assert_eq!(slopes.into_iter().map(three_decimals).collect::<Vec<_>>(), vec!["0.301", "1.699"]);
    }

    #[test]
    fn the_measured_total_is_the_allocation_records_and_the_denominator_is_the_declared_units() {
        let operation = Operation { knob: 10, releases: 2000, mapped_units: Some(500), ..operation(1, 1000, 1000) };
        assert_eq!(measured_total(&operation), Some(1000.0));
        assert_eq!(measured_total_by_mapped_units(&operation), Some(500.0));
        assert_eq!(denominator(&operation), Some(1000));
    }

    #[test]
    fn the_factor_table_reads_zero_everywhere_when_nothing_grows() {
        let flat_per_unit = |units: u64| (7 * units, units);
        for (low, high) in [((1, 1), (1, 1)), ((100, 100), (100, 100)), ((100, 1000), (1000, 1000)), ((1000, 10_000), (10_000, 10_000))] {
            let (low_steps, low_units) = flat_per_unit(low.0);
            let (high_steps, high_units) = flat_per_unit(high.0);
            let reading = two_point_reading((low_steps, low_units, low.1 as f64), (high_steps, high_units, 10.0 * high.1 as f64));
            assert_eq!(three_decimals(reading.alpha), "0.000");
        }
    }

    fn judged(segment: &'static str, tier: &'static str, tested: f64, scan: f64) -> JudgedCell {
        let mut outcome = CellOutcome::default();
        outcome.alpha.insert(Arm::Tested.tag(), Some(tested));
        outcome.alpha.insert(Arm::ScanEveryPublish.tag(), Some(scan));
        outcome.alpha.insert(Arm::LargestLgRowLinear.tag(), Some(1.0));
        JudgedCell { segment, set: "constructed", state: "same_mount", tier, pick: Pick::LaterPeak, steady: true, outcome }
    }

    #[test]
    fn the_void_clause_judges_the_big_points_apart_from_the_new_points() {
        let cells = [judged("new", "user_units_equal_declared", 0.3, 0.32), judged("big", "user_units_equal_declared", 0.3, 0.6)];
        let tallies = control_tallies(&cells);
        let new = tallies.get(&("new", "user_units_equal_declared")).copied().expect("新点那一段有账");
        let big = tallies.get(&("big", "user_units_equal_declared")).copied().expect("大点那一段有自己的账");
        assert!(new.scan_void());
        assert!(!big.scan_void());
        assert_eq!(big, ControlTally { applicable: 1, scan_red: 1, linear_red: 1 });
    }

    #[test]
    fn one_later_steady_cell_that_grows_with_both_controls_flips_row_two() {
        let cell = judged("new", "one_user_unit", 0.6, 0.9);
        let reading = row_reading(&[&cell]);
        assert!(reading.row_two_flips);
        assert!(!reading.row_one_holds);
    }

    #[test]
    fn the_e206_points_take_the_units_from_the_name() {
        assert_eq!(units_by_the_point_name("files_1000_units_100"), 100);
        assert_eq!(units_by_the_point_name("files_100_units_1000_unit_area_2_pow_21"), 1000);
        assert_eq!(units_by_the_point_name("files_1000"), 1);
    }

    #[test]
    fn local_constants_match_the_implementation_lines() {
        assert_eq!(LEAF_SLOTS + INTERNAL_FANOUT + SETTLING_ROUNDS_LIMIT + DISKS, 812 + 169 + 64 + 2);
    }

    #[test]
    fn the_probe_outcomes_sort_into_the_three_hypotheses() {
        let outcome = |cold: (bool, bool), create: (bool, bool), young: (bool, bool)| {
            let wrap = |pair: (bool, bool)| (Some(pair.0), Some(pair.1));
            probe_outcome(&BTreeMap::from([("cold_first", wrap(cold)), ("create_first", wrap(create)), ("young_first", wrap(young))]))
        };
        assert_eq!(outcome((true, false), (false, true), (false, true)), "h1");
        assert_eq!(outcome((true, false), (true, false), (true, false)), "h2");
        assert_eq!(outcome((false, true), (false, true), (false, true)), "h3");
        assert_eq!(outcome((true, true), (false, true), (false, true)), "none_of_the_three");
        assert_eq!(probe_outcome(&BTreeMap::new()), "unread");
    }

    #[test]
    fn the_e206_cold_overwrite_declares_the_built_units_once_and_then_one() {
        let item = PointArmReadings { files: 10, data_units: 1000, units_per_small_file: 1, ..PointArmReadings::default() };
        let mut touched = BTreeSet::new();
        assert_eq!(constructed_user_units(&item, "W2", 1, &mut touched), Some((1000, 1000, 1)));
        assert_eq!(constructed_user_units(&item, "W2", 2, &mut touched), Some((1, 1, 1)));
        assert_eq!(constructed_user_units(&item, "W2", 33, &mut touched), Some((1, 1, 1)), "第 33 次又落回大文件，已覆盖写过，记 1");
        assert_eq!(constructed_user_units(&item, "W3", 1, &mut touched), Some((3, 0, 0)));
        assert_eq!(constructed_user_units(&item, "destroy_snapshot", 1, &mut touched), None);
    }

    #[test]
    fn the_tiers_take_one_unit_and_the_declared_count_from_one_hundred_up() {
        let one = operation(1, 1, 1);
        let large = operation(1, 1000, 1);
        assert!(in_the_tier(&one, "one_user_unit", 1000) && !in_the_tier(&one, "user_units_equal_declared", 1000));
        assert!(in_the_tier(&large, "user_units_equal_declared", 1000) && !in_the_tier(&large, "user_units_equal_declared", 100));
        assert!(!in_the_tier(&operation(1, 10, 1), "user_units_equal_declared", 10), "K 不到 100 的点没有 user_units_equal_declared 档");
    }

    #[test]
    fn only_the_first_growing_is_told_apart_from_a_later_one() {
        let mut first_all = judged("new", "one_user_unit", 0.7, 0.9);
        first_all.pick = Pick::First;
        first_all.steady = false;
        let later = judged("new", "one_user_unit", 0.1, 0.9);
        assert!(row_reading(&[&first_all, &later]).only_the_first_grows);
        let later_grows = judged("new", "one_user_unit", 0.7, 0.9);
        assert!(!row_reading(&[&first_all, &later_grows]).only_the_first_grows);
    }

    // ---- 报告层：假的 PointArmReadings 与操作喂已知答案 ----
    fn item_named(point: &str, arm: &str, units_per_small_file: u64) -> PointArmReadings {
        PointArmReadings {
            name: format!("{point}__{arm}"),
            point: point.to_string(),
            arm: arm.to_string(),
            probe: "standard".to_string(),
            units_per_small_file,
            ..PointArmReadings::default()
        }
    }

    /// 一次操作：状态、操作、第几次、步数、user_units、操作之前的记录数。
    fn operation_at(state: &str, operation_name: &str, operation_index: u64, steps: u64, user_units: u64, records: u64) -> Operation {
        Operation { state: state.to_string(), operation_name: operation_name.to_string(), operation_index, ..operation(steps, user_units, records) }
    }

    fn points_with<'items>(entries: Vec<(&str, &'items PointArmReadings, Vec<Operation>)>) -> Points<'items> {
        entries.into_iter().map(|(key, item, operations)| (key.to_string(), PointReadings { item, operations })).collect()
    }

    fn lines_starting<'lines>(out: &'lines [String], start: &str) -> Vec<&'lines str> {
        out.iter().map(String::as_str).filter(|line| line.starts_with(start)).collect()
    }

    fn settle_at(state: &str, operation_name: &str, operation_index: u64, judgement: u64) -> Settle {
        Settle { state: state.to_string(), operation_name: operation_name.to_string(), operation_index, judgement, ..Settle::default() }
    }

    /// 一次判的状态读数：B8 式子要的十一样都给（记录数取 `records`）。
    fn state_fields(records: u64) -> Fields {
        fields_of(&format!(
            "E207RESULT name=state inode_leaf_containers=1 other_files=0 allocation_records={records} mapped_units=10 deadlist_entries=0 extent_upper_nodes=0 extent_upper_levels=0 accounting_tree_levels=1 deadlist_tree_levels=1 allocation_tree_levels=1 central_mapping_levels=1"
        ))
    }

    /// 一次判、固定点 0 轮，两道之间那一段 `allocator_release_linear_find` 调 `calls` 次。
    fn judgement_calling_the_release_find(calls: u64) -> JudgementReadings {
        let mut judgement = JudgementReadings { fixed_point_rounds: Some(0), ..JudgementReadings::default() };
        judgement.rows.insert(("between".to_string(), "allocator_release_linear_find".to_string()), (calls, 0));
        judgement
    }

    #[test]
    fn one_operation_sums_the_admission_scan_steps_of_its_settles() {
        let mut item = item_named("constructed", "head", 1);
        item.judgements.insert("same_mount/W1/1/".to_string(), JudgementReadings { state: state_fields(1000), ..JudgementReadings::default() });
        item.judgements.insert("same_mount/W1/1/1".to_string(), judgement_calling_the_release_find(1));
        item.judgements.insert("same_mount/W1/1/2".to_string(), judgement_calling_the_release_find(4));
        item.window = vec![settle_at("same_mount", "W1", 1, 1), settle_at("same_mount", "W1", 1, 2)];
        let one = by_delta_of_settle(&item, &item.window[0]).expect("读得到");
        let two = by_delta_of_settle(&item, &item.window[1]).expect("读得到");
        assert_eq!(two, one + 30, "多 3 次 allocator_release_linear_find、记录 1000 条：3 × lg(1000) = 30 步");
        let operations = operations_of(&item);
        assert_eq!(operations.len(), 1, "同一个（状态、操作、第几次）的两次固定点是一次操作");
        assert_eq!(operations[0].steps, Some(one + two));
        assert_eq!(operations[0].allocation_records, Some(1000), "x 取操作之前那一行状态");
    }

    /// 一次判、固定点 0 轮，两道之间那一段 s37 调 `calls` 次、共 `steps` 步，s29 另有 `per_disk_fallback_steps` 步（都不进 B8 合计）。
    fn judgement_calling_the_pooled_fallback(calls: u64, steps: u64, per_disk_fallback_steps: u64) -> JudgementReadings {
        let mut judgement = JudgementReadings { fixed_point_rounds: Some(0), ..JudgementReadings::default() };
        judgement.rows.insert(("between".to_string(), ROW_WITHOUT_A_BY_DELTA_FORMULA.to_string()), (calls, 0));
        judgement.row_steps.insert(("between".to_string(), ROW_WITHOUT_A_BY_DELTA_FORMULA.to_string()), steps);
        judgement.row_steps.insert(("between".to_string(), "fallback_slot_linear_scan".to_string()), per_disk_fallback_steps);
        judgement
    }

    #[test]
    fn the_pooled_fallback_adds_no_by_delta_steps_and_marks_the_operations_that_call_it() {
        let mut item = item_named("constructed", "head", 1);
        item.judgements.insert("same_mount/W1/1/".to_string(), JudgementReadings { state: state_fields(1000), ..JudgementReadings::default() });
        item.judgements.insert("same_mount/W1/1/1".to_string(), judgement_calling_the_pooled_fallback(0, 0, 0));
        item.judgements.insert("same_mount/W1/2/".to_string(), JudgementReadings { state: state_fields(1000), ..JudgementReadings::default() });
        item.judgements.insert("same_mount/W1/2/1".to_string(), judgement_calling_the_pooled_fallback(3, 500, 7));
        item.window = vec![settle_at("same_mount", "W1", 1, 1), settle_at("same_mount", "W1", 2, 1)];
        let (none, called) = (
            by_delta_of_settle(&item, &item.window[0]).expect("读得到"),
            by_delta_of_settle(&item, &item.window[1]).expect("读得到"),
        );
        assert_eq!(none, called, "s37 没有 B8 式子：调 3 次、500 步也不进 B8 合计");
        let operations = operations_of(&item);
        assert_eq!(operations.len(), 2);
        assert_eq!((operations[0].pooled_fallback_calls, operations[0].pooled_fallback_steps, operations[0].fallback_scan_steps), (0, 0, 0));
        assert_eq!((operations[1].pooled_fallback_calls, operations[1].pooled_fallback_steps, operations[1].fallback_scan_steps), (3, 500, 7));
        let mut out = Lines::new();
        let points: Points<'_> = BTreeMap::from([("constructed".to_string(), PointReadings { item: &item, operations })]);
        report_operations(&points, &mut out);
        report_missing_the_pooled_fallback_row(&points, &mut out);
        assert!(out.contains(&"name=op_pooled_fallback item=constructed__head state=same_mount operation=W1 operation_index=1 pooled_fallback_calls=0 pooled_fallback_steps=0 per_disk_fallback_steps=0 by_delta_arms_total=complete".to_string()), "{out:?}");
        assert!(out.contains(&"name=op_pooled_fallback item=constructed__head state=same_mount operation=W1 operation_index=2 pooled_fallback_calls=3 pooled_fallback_steps=500 per_disk_fallback_steps=7 by_delta_arms_total=missing_pooled_fallback_row_only_for_the_listed_steps".to_string()), "{out:?}");
        assert!(out.contains(&"name=verdict quantity=missing_pooled_fallback operations=2 operations_with_pooled_fallback_calls=1 points_marked=1 marked_points=constructed".to_string()), "{out:?}");
    }

    #[test]
    fn a_point_with_a_frozen_publish_is_voided_whole_and_the_unread_field_is_counted() {
        let state_with = |frozen: Option<u64>| {
            let mut state = state_fields(1000);
            if let Some(value) = frozen {
                state.insert("frozen_publish_set".to_string(), value.to_string());
            }
            state
        };
        let point_with = |name: &str, frozen: [Option<u64>; 2]| {
            let mut item = item_named(name, "head", 1);
            for (index, value) in frozen.iter().enumerate() {
                let operation_index = index as u64 + 1;
                item.judgements.insert(format!("same_mount/W1/{operation_index}/"), JudgementReadings { state: state_with(*value), ..JudgementReadings::default() });
                item.judgements.insert(format!("same_mount/W1/{operation_index}/1"), judgement_calling_the_release_find(1));
            }
            item.window = vec![settle_at("same_mount", "W1", 1, 1), settle_at("same_mount", "W1", 2, 1)];
            item
        };
        let (clean, frozen, old) = (point_with("clean", [Some(0), Some(0)]), point_with("frozen", [Some(0), Some(1)]), point_with("old", [None, None]));
        let items = [clean, frozen, old];
        let mut points = points_of(&items);
        let mut out = Lines::new();
        let voided = points_with_a_frozen_publish(&points, &mut out);
        points.retain(|name, _| !voided.contains(name));
        assert_eq!(out, vec![
            "name=void_frozen_publish point=frozen result=point_voided".to_string(),
            "name=verdict quantity=void_frozen_publish operations=6 operations_without_the_reading=2 points_voided=1 voided_points=frozen".to_string(),
        ]);
        assert_eq!(points.len(), 2, "冻结着的那个点整点拿掉，别的点照常");
    }

    #[test]
    fn the_report_reads_no_voided_point_in_any_quantity() {
        let state_with = |frozen: u64| {
            let mut state = state_fields(1000);
            state.insert("frozen_publish_set".to_string(), frozen.to_string());
            state
        };
        let point_with = |name: &str, frozen: u64| {
            let mut item = item_named(name, "head", 1);
            item.judgements.insert("same_mount/W1/1/".to_string(), JudgementReadings { state: state_with(frozen), ..JudgementReadings::default() });
            item.judgements.insert("same_mount/W1/1/1".to_string(), judgement_calling_the_release_find(1));
            item.window = vec![settle_at("same_mount", "W1", 1, 1)];
            item
        };
        let items = [point_with("clean", 0), point_with("frozen", 1)];
        let mut out = Lines::new();
        report_items("third", &items, &[], &mut out);
        assert!(out.iter().any(|line| line.starts_with("name=verdict quantity=void_frozen_publish") && line.contains("voided_points=frozen")), "{out:?}");
        // 作废的点不进后面各量：缺池化回退行那一量只数没作废的那个点的一次操作（变异：作废的点留在各量里时红）
        assert!(out.iter().any(|line| line.starts_with("name=verdict quantity=missing_pooled_fallback operations=1 ")), "{out:?}");
    }

    #[test]
    fn operations_take_the_declared_units_and_not_the_constructed_ones() {
        let mut item = item_named("constructed", "head", 1000);
        item.window = vec![settle_at("same_mount", "W1", 1, 1)];
        item.declared = vec![(("same_mount".to_string(), "W1".to_string(), 1), (7, 7, 1))];
        let operations = operations_of(&item);
        assert_eq!((operations[0].user_units, operations[0].old_units, operations[0].new_units), (Some(7), 7, 1));
        item.declared.clear();
        assert_eq!(operations_of(&item)[0].user_units, Some(1), "没有声明行（E206 产物）才按构造补：W1 覆盖写第一个文件记 1");
    }

    #[test]
    fn the_comparison_sets_are_the_registered_points() {
        let shape = |sets: &[ComparisonSet]| -> Vec<(&'static str, &'static str, Vec<&'static str>, Vec<&'static str>)> {
            sets.iter().map(|set| (set.segment, set.name, set.points.to_vec(), set.tiers.to_vec())).collect()
        };
        assert_eq!(
            shape(&NEW_POINT_SETS[..]),
            vec![
                ("new", "K100", vec!["files_100_units_100", "files_1000_units_100"], vec!["one_user_unit", "user_units_equal_declared"]),
                ("new", "K1000", vec!["files_10_units_1000", "files_100_units_1000"], vec!["one_user_unit", "user_units_equal_declared"]),
                ("new", "FTB", vec!["files_then_big_1000", "files_then_big_10000"], vec!["one_user_unit"]),
                ("new", "D100", vec!["files_100_units_100", "files_100_units_1000"], vec!["user_units_equal_declared"]),
                ("new", "D10", vec!["files_10_units_1000", "files_10_units_10000"], vec!["user_units_equal_declared"]),
                ("new", "K1000_other_unit_area", vec!["files_10_units_1000_unit_area_2_pow_17", "files_100_units_1000_unit_area_2_pow_20"], vec!["one_user_unit", "user_units_equal_declared"]),
            ]
        );
        assert_eq!(
            shape(&BIG_POINT_SETS[..]),
            vec![(
                "big",
                "K1000L",
                vec!["files_10_units_1000_unit_area_2_pow_21", "files_100_units_1000_unit_area_2_pow_21", "files_1000_units_1000"],
                vec!["one_user_unit", "user_units_equal_declared"]
            )]
        );
        assert_eq!(shape(&RETRO_SETS[..]).iter().map(|set| set.2.len()).collect::<Vec<_>>(), vec![4, 4, 4, 4], "回扫四根轴各四点");
    }

    #[test]
    fn the_quantities_take_the_registered_sets() {
        let table: Vec<(&str, &str, &str, Vec<&str>)> =
            NEW_POINT_QUANTITIES.iter().map(|(reading, quantity, tier, sets)| (*reading, *quantity, *tier, sets.to_vec())).collect();
        assert_eq!(
            table,
            vec![
                ("one_unit_change_against_the_total", "q1", "one_user_unit", vec!["K100", "K1000", "FTB"]),
                ("many_unit_change_against_the_total", "q2", "user_units_equal_declared", vec!["K100", "K1000"]),
                ("per_unit_steps_as_the_change_grows", "q3", "user_units_equal_declared", vec!["D100", "D10"]),
                ("many_unit_change_against_the_total_on_the_other_unit_area", "q2_other_unit_area", "user_units_equal_declared", vec!["K1000_other_unit_area"]),
                ("one_unit_change_against_the_total_on_the_other_unit_area", "q1_other_unit_area", "one_user_unit", vec!["K1000_other_unit_area"]),
            ]
        );
    }

    #[test]
    fn the_controls_read_the_same_operation_as_the_tested_arm() {
        let (low_item, high_item) = (item_named("low", "head", 1), item_named("high", "head", 1));
        let spiked = |operation_index: u64, steps: u64, records: u64, flagged: bool| Operation { flagged, ..operation_at("same_mount", "W1", operation_index, steps, 1, records) };
        let points = points_with(vec![
            ("low", &low_item, vec![spiked(1, 1000, 100, true), spiked(2, 10, 100, false)]),
            ("high", &high_item, vec![spiked(1, 1000, 1000, true), spiked(2, 10, 1000, false)]),
        ]);
        let set = ComparisonSet { segment: "new", name: "constructed", points: &["low", "high"], tiers: &["one_user_unit"] };
        let mut out = Lines::new();
        let outcome = report_cell(&points, &set, "same_mount", "one_user_unit", Pick::First, true, &mut out);
        assert_eq!(lines_starting(&out, "name=cell ").len(), 4, "每条臂一行");
        assert_eq!(three_decimals(outcome.alpha["tested"]), "0.000", "steady 剔掉旗上的 W1#1，两端都取 W1#2：10 步对 10 步");
        assert_eq!(three_decimals(outcome.alpha["c1"]), "0.963", "C1 同样只取 W1#2：(10 + 100) 对 (10 + 1000)");
        assert_eq!(outcome.alpha["c3"], None, "这个点没有含 lg 的行，C3 读不到");
    }

    #[test]
    fn one_negative_control_over_four_rounds_fails_the_item_check() {
        let check = |rounds: u64| -> String {
            let mut item = item_named("base", "head", 1);
            item.window = vec![Settle { rounds, ..settle_at("same_mount", "negative_control", 1, 1) }];
            let point = PointReadings { item: &item, operations: Vec::new() };
            let mut out = Lines::new();
            report_item_checks(&point, &mut out);
            out.concat()
        };
        assert!(check(4).contains(" negative_control_rounds=4 negative_control_ok=true"));
        assert!(check(5).contains(" negative_control_rounds=5 negative_control_ok=false"));
    }

    fn judged_with(tested: f64, scan: f64, linear: f64) -> JudgedCell {
        let mut cell = judged("new", "one_user_unit", tested, scan);
        cell.outcome.alpha.insert(Arm::LargestLgRowLinear.tag(), Some(linear));
        cell
    }

    #[test]
    fn one_growing_later_cell_needs_both_controls_red_to_flip_row_two() {
        assert!(row_reading(&[&judged_with(0.6, 0.9, 1.0)]).row_two_flips);
        assert!(!row_reading(&[&judged_with(0.6, 0.9, 0.1)]).row_two_flips, "C3 没红");
        assert!(!row_reading(&[&judged_with(0.6, 0.1, 1.0)]).row_two_flips, "C1 没红");
        assert!(row_reading(&[&judged_with(0.1, 0.9, 1.0)]).row_one_holds);
        assert!(!row_reading(&[&judged_with(0.1, 0.9, 0.1)]).row_one_holds, "行一同样要两条对照都红");
    }

    #[test]
    fn the_similar_total_set_reads_the_growth_against_the_user_units() {
        let (low, high) = (item_named("files_100_units_1000", "head", 1000), item_named("files_10_units_10000", "head", 10_000));
        let points = points_with(vec![
            ("files_100_units_1000", &low, vec![operation_at("same_mount", "W2", 1, 100_000, 1000, 1000)]),
            ("files_10_units_10000", &high, vec![operation_at("same_mount", "W2", 1, 1_000_000, 10_000, 1100)]),
        ]);
        let mut out = Lines::new();
        report_similar_total_set(&points, &mut out);
        assert_eq!(out.len(), 12, "两种状态 × 三种取法 × 剔不剔旗");
        assert!(out.contains(
            &"name=t1e5 state=same_mount pick=first filter=all alpha_against_the_user_units_per_unit=0.000 band_per_unit=does_not_grow alpha_against_the_user_units_raw=1.000 band_raw=grows".to_string()
        ));
    }

    #[test]
    fn the_retro_first_cold_overwrite_reads_per_unit_and_is_looser() {
        let (base, top) = (item_named("base", "p2a", 1), item_named("data_units_10000", "p2a", 1));
        let retro = points_with(vec![
            ("base", &base, vec![operation_at("same_mount", "W2", 1, 1000, 10, 100)]),
            ("data_units_10000", &top, vec![operation_at("same_mount", "W2", 1, 1_000_000, 10_000, 1000)]),
        ]);
        let mut out = Lines::new();
        report_retro_data_units_first_cold(&retro, &mut out);
        assert_eq!(
            out,
            vec!["name=q4_data_units_first_cold_overwrite ends=1000/10/100,unread,unread,1000000/10000/1000 looser=yes alpha=0.000 band=does_not_grow raw_alpha=3.000 raw_band=grows".to_string()]
        );
    }

    #[test]
    fn one_cell_where_the_tested_arm_grows_is_not_applicable_to_the_controls() {
        let tallies = control_tallies(&[judged("new", "one_user_unit", 0.6, 0.1)]);
        assert_eq!(tallies.get(&("new", "one_user_unit")).copied(), Some(ControlTally::default()));
        assert!(!tallies[&("new", "one_user_unit")].scan_void());
    }

    #[test]
    fn the_spike_flags_line_lists_the_flagged_operations_and_the_peak() {
        let item = item_named("constructed", "head", 1);
        let nodes = |operation_index: u64, steps: u64, final_nodes: u64, flagged: bool| Operation { final_nodes, flagged, ..operation_at("same_mount", "W1", operation_index, steps, 1, 100) };
        let points = points_with(vec![("constructed", &item, vec![nodes(1, 5, 7, false), nodes(2, 900, 433, true), nodes(3, 6, 7, false), nodes(4, 4, 7, false)])]);
        let mut out = Lines::new();
        report_operations(&points, &mut out);
        assert_eq!(lines_starting(&out, "name=op_row ").len(), 4, "每次操作一行");
        assert_eq!(
            lines_starting(&out, "name=q6_spike_flags "),
            vec![
                "name=q6_spike_flags point=constructed state=same_mount operations=4 median_final_nodes=7 line=28 flagged=1 flagged_operations=W1#2:433 peak_steps=900 peak_at=W1#2 final_steps=4",
                "name=q6_spike_flags point=constructed state=remounted operations=0 median_final_nodes=0 line=28 flagged=0 flagged_operations=- peak_steps=unread peak_at=unread final_steps=unread",
            ]
        );
    }

    #[test]
    fn operations_flag_the_spike_against_the_line_of_their_own_state() {
        let mut item = item_named("constructed", "head", 1);
        item.window = vec![
            Settle { final_nodes: 7, ..settle_at("same_mount", "W1", 1, 1) },
            Settle { final_nodes: 7, ..settle_at("same_mount", "W1", 2, 1) },
            Settle { final_nodes: 7, ..settle_at("same_mount", "W1", 3, 1) },
            Settle { final_nodes: 28, ..settle_at("same_mount", "W1", 4, 1) },
            Settle { final_nodes: 7, ..settle_at("same_mount", "W1", 5, 1) },
            Settle { final_nodes: 433, ..settle_at("remounted", "W1", 1, 1) },
        ];
        let flagged: Vec<(String, u64)> = operations_of(&item).into_iter().filter(|operation| operation.flagged).map(|operation| (operation.state, operation.operation_index)).collect();
        assert_eq!(flagged, vec![("same_mount".to_string(), 4)], "28 正在线上算旗上；433 只比它自己那一状态的线（4 × 433）");
    }

    #[test]
    fn the_then_big_ratio_takes_the_largest_ratio_over_the_same_operations() {
        let (then_big, files) = (item_named("files_then_big_10000", "head", 1), item_named("files_10000", "head", 1));
        let points = points_with(vec![
            ("files_then_big_10000", &then_big, vec![operation_at("same_mount", "W1", 1, 300, 1, 1), operation_at("same_mount", "W1", 2, 500, 1, 1)]),
            ("files_10000", &files, vec![operation_at("same_mount", "W1", 1, 200, 1, 1), operation_at("same_mount", "W1", 2, 200, 1, 1), operation_at("same_mount", "W2", 1, 1, 1, 1)]),
        ]);
        let mut out = Lines::new();
        report_then_big_ratio(&points, &mut out);
        assert_eq!(out, vec!["name=verdict quantity=q9 maximum_ratio=2.500 at=same_mount/W1#2 exceeds=true".to_string()]);
    }

    #[test]
    fn the_declared_line_is_read_back_into_the_item() {
        let mut item = PointArmReadings::default();
        take_line(
            &mut item,
            &fields_of("E207RESULT name=declared item=files_10_units_1000__head phase=window state=same_mount operation=W2 operation_index=1 user_units=1000 old_units=1000 new_units=1"),
            E207_OPERATION_KEYS,
        );
        assert_eq!(item.declared, vec![(("same_mount".to_string(), "W2".to_string(), 1), (1000, 1000, 1))]);
    }

    #[test]
    fn each_line_is_read_with_the_operation_keys_of_its_own_experiment() {
        // E206 留存产物逐字的那一对键，钉在这里；E206 那一行照这一对拼。
        assert_eq!(E206_OPERATION_KEYS, OperationKeys { name: "op", index: "op_index" });
        let lines = [
            format!(
                "E206RESULT name=settle item=files_1000__p2a phase=window state=same_mount {}=W1 {}=2 judgement=1 rounds=3",
                E206_OPERATION_KEYS.name, E206_OPERATION_KEYS.index
            ),
            "E207RESULT name=settle item=files_1000__head phase=window state=remounted operation=W2 operation_index=4 judgement=1 rounds=2".to_string(),
        ];
        let mut item = PointArmReadings::default();
        for line in &lines {
            take_line(&mut item, &fields_of(line), operation_keys_of(line));
        }
        let read: Vec<(String, String, u64, u64)> =
            item.window.iter().map(|settle| (settle.state.clone(), settle.operation_name.clone(), settle.operation_index, settle.rounds)).collect();
        assert_eq!(
            read,
            vec![
                ("same_mount".to_string(), "W1".to_string(), 2, 3),
                ("remounted".to_string(), "W2".to_string(), 4, 2),
            ]
        );
    }

    #[test]
    fn the_item_check_compares_the_product_constants_with_the_local_ones() {
        let check = |leaf_slots: u64| -> String {
            let mut item = item_named("base", "head", 1);
            item.constants = fields_of(&format!("E207RESULT name=constants leaf_slots={leaf_slots} internal_fanout=169 settling_rounds_at_most=64 disks=2"));
            let mut out = Lines::new();
            report_item_checks(&PointReadings { item: &item, operations: Vec::new() }, &mut out);
            out.concat()
        };
        assert!(check(812).contains(" constants_match=true "));
        assert!(check(813).contains(" constants_match=false "));
    }

    fn round_with(rewrite_set_size: u64, exit: &str, release_dirty_new: u64, holding_differs: bool) -> Round {
        Round { rewrite_set_size, exit: exit.to_string(), release_dirty_new, holding_differs, ..Round::default() }
    }

    #[test]
    fn the_fixed_point_premises_are_checked_from_the_round_readings() {
        let clean = Settle { detail: vec![round_with(3, "release_not_covered", 0, false), round_with(5, "settled", 0, false)], ..Settle::default() };
        assert_eq!(fixed_point_premise_breaks(&[clean]), (0, 0));
        let shrinking = Settle {
            detail: vec![round_with(3, "release_not_covered", 0, false), round_with(5, "placement_not_covered", 0, false), round_with(4, "settled", 0, false)],
            ..Settle::default()
        };
        assert_eq!(fixed_point_premise_breaks(&[shrinking]), (1, 0), "5 → 4 变小了一次");
        let dirty = Settle { detail: vec![round_with(3, "settled", 1, false)], ..Settle::default() };
        let holding = Settle { detail: vec![round_with(3, "settled", 0, true)], ..Settle::default() };
        assert_eq!(fixed_point_premise_breaks(&[dirty, holding]), (0, 2), "定下那一轮还有新弄脏、或有记录的节点对不上");
    }

    #[test]
    fn the_round_work_is_held_against_the_sum_of_the_rewritten_roles() {
        let rounds = |roles: &[u64]| -> Vec<Round> { roles.iter().map(|roles_rewritten| Round { roles_rewritten: *roles_rewritten, ..Round::default() }).collect() };
        let settled = Settle { round_work: 8, detail: rounds(&[3, 5][..]), ..Settle::default() };
        let off = Settle { round_work: 9, detail: rounds(&[3, 5][..]), ..Settle::default() };
        let history = Settle { round_work: 9, ..Settle::default() };
        assert_eq!(round_work_mismatches(&[settled, off, history]), 1, "W = 3 + 5 = 8 的对得上；9 的对不上；历史那一次没有逐轮读数，不比");
    }

    #[test]
    fn the_old_criterion_line_is_written_like_the_e206_report() {
        let first = old_criterion_line("q8_first_publish", "files", &[Some(2102.0), Some(2331.0), Some(4598.0), Some(15_440.0)]);
        assert_eq!(first, OLD_CRITERION_FIRST_PUBLISH_FILES);
        let peak = old_criterion_line("q8_peak", "data_units", &[Some(3763.0), Some(17_200.0), Some(179_690.0), Some(3_231_038.0)]);
        assert_eq!(peak, OLD_CRITERION_PEAK_DATA_UNITS);
        assert_eq!(
            old_criterion_line("q8_peak", "data_units", &[Some(3763.0), None, Some(179_690.0), None]),
            "name=q8_peak arm=p2a axis=data_units low_end=3763 high_end=unread alpha=unread band=unread local_slopes=unread,unread,unread"
        );
        let mut out = Lines::new();
        assert!(!old_criterion_pins(&[], &mut out), "没有回扫产物两行都读不到，不当相同");
        assert_eq!(out[2], "name=verdict quantity=old_criterion_pins first_publish_files_matches=false peak_data_units_matches=false");
    }

    #[test]
    fn the_reconciliation_compares_every_window_settle_with_e206() {
        let window = |hash: &str| -> Vec<Settle> {
            vec![Settle { rounds: 2, final_roles: 20, final_nodes: 7, round_work: 32, final_set_hash: hash.to_string(), ..settle_at("same_mount", "W1", 1, 1) }]
        };
        let (mut today_base, mut before_base) = (item_named("base", "head", 1), item_named("base", "p2a", 1));
        today_base.window = window("aa");
        before_base.window = window("bb");
        let (mut today_files, mut before_files) = (item_named("files_1000", "head", 1), item_named("files_1000", "p2a", 1));
        today_files.window = window("cc");
        before_files.window = window("cc");
        let today = points_with(vec![("base", &today_base, Vec::new()), ("files_1000", &today_files, Vec::new())]);
        let before = points_with(vec![("base", &before_base, Vec::new()), ("files_1000", &before_files, Vec::new())]);
        let mut out = Lines::new();
        report_reconciliation(&today, &before, &mut out);
        assert_eq!(
            out,
            vec![
                "name=verdict quantity=reconciliation_with_e206 point=base settles_today=1 settles_e206=1 differing=1 matches=false first_differing=same_mount/W1/1/1",
                "name=verdict quantity=reconciliation_with_e206 point=files_1000 settles_today=1 settles_e206=1 differing=0 matches=true first_differing=-",
                "name=verdict quantity=reconciliation_with_e206 point=files_10000 matches=not_run",
            ]
        );
    }

    fn spike_point(final_nodes: u64) -> PointArmReadings {
        let mut item = item_named("files_1000_units_100", "head", 100);
        item.window = vec![
            Settle { rounds: 3, final_roles: 452, final_nodes, round_work: 917, closure_nodes_added: 410, ..settle_at("same_mount", "W1", 1, 1) },
            Settle { rounds: 2, final_roles: 20, final_nodes: 7, round_work: 32, ..settle_at("same_mount", "W1", 2, 1) },
        ];
        let state = fields_of("E207RESULT name=state mapped_units=101581 allocation_records=238324");
        item.judgements.insert("same_mount/W1/1/".to_string(), JudgementReadings { state, ..JudgementReadings::default() });
        item
    }

    #[test]
    fn the_spike_pin_holds_the_registered_numbers() {
        for (final_nodes, expected) in [
            (433, "name=verdict quantity=spike_pin matches=true mismatches=-"),
            (432, "name=verdict quantity=spike_pin matches=false mismatches=W1#1.final_nodes=432≠433"),
        ] {
            let item = spike_point(final_nodes);
            let points = points_with(vec![("files_1000_units_100", &item, Vec::new())]);
            let mut out = Lines::new();
            report_spike_pin(&points, &mut out);
            assert_eq!(out, vec![expected]);
        }
        let mut out = Lines::new();
        report_spike_pin(&Points::new(), &mut out);
        assert_eq!(out, vec!["name=verdict quantity=spike_pin matches=not_run"]);
    }

    #[test]
    fn the_declared_units_are_held_against_the_mapped_units_on_first_touches() {
        let item = item_named("files_10_units_1000", "head", 1000);
        let touch = |operation_index: u64, mapped: u64| Operation { old_units: 1000, new_units: 1, mapped_units: Some(mapped), ..operation_at("same_mount", "W2", operation_index, 1, 1000, 1) };
        let mut operations = vec![touch(1, 10_000), touch(2, 9001), touch(3, 8002), touch(4, 7003), touch(5, 6004)];
        operations.push(Operation { mapped_units: Some(5005), ..operation_at("same_mount", "W3", 1, 1, 3, 1) });
        let points = points_with(vec![("files_10_units_1000", &item, operations.clone())]);
        let mut out = Lines::new();
        report_declared_against_mapped(&points, &mut out);
        assert_eq!(
            out,
            vec![
                "name=verdict quantity=declared_against_mapped point=files_10_units_1000 state=same_mount first_touches=5 enough_first_touches=true ratios_outside=0 outside=-",
                "name=verdict quantity=declared_against_mapped point=files_10_units_1000 state=remounted first_touches=0 enough_first_touches=false ratios_outside=0 outside=-",
            ],
            "每次换下 999 个单元、mapped_units 也少 999：比值 1，落在 [0.5, 2]"
        );
        let four = points_with(vec![("files_10_units_1000", &item, operations[1..].to_vec())]);
        let mut out = Lines::new();
        report_declared_against_mapped(&four, &mut out);
        assert!(out[0].contains(" first_touches=4 enough_first_touches=false "), "F8：第一次碰的少于 5 个，这一状态的 user_units_equal_declared 档样本不足");
    }

    #[test]
    fn the_plain_build_is_held_against_the_counted_build_on_the_image() {
        let with_image = |point: &str, arm: &str, sha: &str| PointArmReadings { image_sha256_at_the_history_end: Some(sha.to_string()), ..item_named(point, arm, 1) };
        let (base, base_plain) = (with_image("base", "head", "aa"), with_image("base", "plain", "aa"));
        let (files, files_plain) = (with_image("files_1000", "head", "aa"), with_image("files_1000", "plain", "bb"));
        let points = points_with(vec![
            ("base", &base, Vec::new()),
            ("base__plain", &base_plain, Vec::new()),
            ("files_1000", &files, Vec::new()),
            ("files_1000__plain", &files_plain, Vec::new()),
        ]);
        let mut out = Lines::new();
        report_image_pins(&points, &mut out);
        assert_eq!(out, vec!["name=verdict quantity=plain_image point=base same_image=true", "name=verdict quantity=plain_image point=files_1000 same_image=false"]);
        let mut out = Lines::new();
        report_image_pins(&points_with(vec![("base", &base, Vec::new())]), &mut out);
        assert_eq!(out[0], "name=verdict quantity=plain_image point=base same_image=not_run");
    }

    #[test]
    fn the_big_point_units_are_held_inside_the_registered_range() {
        let item = item_named("files_1000_units_1000", "head", 1000);
        let check = |mapped: u64| -> Lines {
            let points = points_with(vec![("files_1000_units_1000", &item, vec![Operation { mapped_units: Some(mapped), ..operation_at("same_mount", "W1", 1, 1, 1, 1) }])]);
            let mut out = Lines::new();
            report_unit_counts(&points, &mut out);
            out
        };
        assert_eq!(
            check(1_020_000),
            vec![
                "name=occupancy point=files_1000_units_1000 units_per_small_file=1000 mapped_units_at_the_history_end=1020000 change_share=0.0010 far_below_the_total=true",
                "name=verdict quantity=big_point_units point=files_1000_units_1000 mapped_units=1020000 inside=true",
            ]
        );
        assert_eq!(check(999_999)[1], "name=verdict quantity=big_point_units point=files_1000_units_1000 mapped_units=999999 inside=false");
    }

    #[test]
    fn looser_instances_count_the_cells_where_only_the_per_unit_reading_stays_flat() {
        let cell = |raw: f64, per_unit: f64| {
            let mut cell = judged("new", "user_units_equal_declared", per_unit, 0.9);
            cell.outcome.raw_alpha.insert(Arm::Tested.tag(), Some(raw));
            cell
        };
        let mut out = Lines::new();
        report_looser_instances(&[cell(0.6, 0.2), cell(0.4, 0.2)], &mut out);
        assert_eq!(
            out,
            vec![
                "name=f3_looser_instance set=constructed state=same_mount tier=user_units_equal_declared pick=later_peak filter=steady raw_alpha=0.600 per_unit_alpha=0.200",
                "name=f3_looser_instances count=1",
            ]
        );
    }

    #[test]
    fn item_checks_count_the_fixed_points_that_did_not_settle_or_hit_the_limit() {
        let mut item = item_named("base", "head", 1);
        item.window = vec![
            Settle { outcome: "did_not_settle".to_string(), rounds: 3, ..settle_at("same_mount", "W1", 1, 1) },
            Settle { outcome: "settled".to_string(), rounds: 64, ..settle_at("same_mount", "W1", 2, 1) },
            Settle { outcome: "settled".to_string(), rounds: 63, ..settle_at("same_mount", "W1", 3, 1) },
        ];
        let mut out = Lines::new();
        report_item_checks(&PointReadings { item: &item, operations: Vec::new() }, &mut out);
        assert!(out[0].contains(" did_not_settle=1 rounds_at_the_limit=1 "), "{}", out[0]);
    }

    #[test]
    fn an_item_with_missing_declared_lines_or_without_done_is_reported() {
        let mut item = item_named("files_10_units_1000", "head", 1000);
        item.declared = vec![(("same_mount".to_string(), "W1".to_string(), 1), (1, 1, 1))];
        let operations = vec![operation_at("same_mount", "W1", 1, 1, 1, 1), operation_at("same_mount", "W1", 2, 1, 1, 1)];
        let mut out = Lines::new();
        report_item_checks(&PointReadings { item: &item, operations: operations.clone() }, &mut out);
        assert!(out[0].contains(" item_done=false "), "{}", out[0]);
        assert!(out[0].contains(" declared_lines=1 window_operations=2 declared_lines_match=false "), "{}", out[0]);
        let complete = PointArmReadings { done: true, declared: vec![item.declared[0].clone(), (("same_mount".to_string(), "W1".to_string(), 2), (1, 1, 1))], ..item.clone() };
        let mut out = Lines::new();
        report_item_checks(&PointReadings { item: &complete, operations }, &mut out);
        assert!(out[0].contains(" item_done=true ") && out[0].contains(" declared_lines_match=true "), "{}", out[0]);
    }

    #[test]
    fn the_three_point_set_without_its_high_end_is_not_read_as_three_points() {
        let (low, middle) = (item_named("low", "head", 1), item_named("middle", "head", 1));
        let points = points_with(vec![
            ("low", &low, vec![operation_at("same_mount", "W1", 1, 10, 1, 100)]),
            ("middle", &middle, vec![operation_at("same_mount", "W1", 1, 100, 1, 1000)]),
        ]);
        let set = ComparisonSet { segment: "big", name: "constructed", points: &["low", "middle", "high"], tiers: &["one_user_unit"] };
        let mut out = Lines::new();
        let outcome = report_cell(&points, &set, "same_mount", "one_user_unit", Pick::First, false, &mut out);
        assert_eq!(outcome.alpha["tested"], None, "高端缺了，α 不拿低端与中点冒充");
        let tested = lines_starting(&out, "name=cell ")[0];
        assert!(tested.contains(" alpha=unread ") && tested.contains(" local_slopes=high_end_missing lower_pair_alpha=1.000 "), "{tested}");
    }

    fn window_of(state: &str, operations: &[(&str, u64)]) -> Vec<Operation> {
        operations.iter().flat_map(|(operation_name, count)| (1..=*count).map(move |operation_index| operation_at(state, operation_name, operation_index, 1, 1, 1))).collect()
    }

    #[test]
    fn the_window_shape_check_names_a_missing_probe_prefix_and_extended_window() {
        let item = PointArmReadings { probe: "cold_first".to_string(), ..item_named("files_1000_units_100", "head", 100) };
        let mut operations = window_of("same_mount", &WINDOW_OPERATIONS);
        operations.extend(window_of("remounted", &WINDOW_OPERATIONS));
        assert_eq!(
            window_shape_mismatches(&PointReadings { item: &item, operations: operations.clone() }),
            vec!["same_mount/P1:0≠1", "same_mount/W5:0≠200", "same_mount/W6:0≠110", "same_mount/W7:0≠11"]
        );
        operations.extend(window_of("same_mount", &EXTENDED_WINDOW_OPERATIONS));
        operations.push(operation_at("same_mount", "P1", 1, 1, 1, 1));
        assert!(window_shape_mismatches(&PointReadings { item: &item, operations }).is_empty());
        let base = item_named("base", "head", 1);
        let mut base_operations = window_of("same_mount", &WINDOW_OPERATIONS);
        base_operations.extend(window_of("remounted", &WINDOW_OPERATIONS));
        base_operations.push(operation_at("remounted", "negative_control", 1, 1, 1, 1));
        assert_eq!(window_shape_mismatches(&PointReadings { item: &base, operations: base_operations }), vec!["same_mount/negative_control:0≠1"]);
    }

    // ---- 复量段 hundred_units_per_file_repeat（修订 2）----
    /// 六点 x 每步乘 10 的 0.25 次幂，相邻五段的指数 `exponents`：Q = 上一点的 Q × (x 之比)^指数。返回 (Q, x)。
    fn series_from_exponents(exponents: [f64; 5]) -> Vec<(f64, f64)> {
        let mut points = vec![(1.0, 1.0)];
        for (index, exponent) in exponents.iter().enumerate() {
            let position = 10_f64.powf(0.25 * (index as f64 + 1.0));
            let (previous_quantity, previous_position) = *points.last().expect("非空");
            points.push((previous_quantity * (position / previous_position).powf(*exponent), position));
        }
        points
    }

    #[test]
    fn the_series_verdicts_match_the_five_registered_constructions() {
        let read = |exponents: [f64; 5]| series_reading(&series_from_exponents(exponents));
        let all_grow = read([0.6; 5]);
        assert_eq!((alpha_text(all_grow.beta), all_grow.grows, all_grow.verdict), ("0.600".to_string(), 5, "stable_grows"));
        assert_eq!(alphas_text(&all_grow), "0.600,0.600,0.600,0.600,0.600");
        let between = read([0.4; 5]);
        assert_eq!((alpha_text(between.beta), between.between, between.verdict), ("0.400".to_string(), 5, "stable_between"));
        let flat = read([0.1; 5]);
        assert_eq!((alpha_text(flat.beta), flat.does_not_grow, flat.verdict), ("0.100".to_string(), 5, "stable_does_not_grow"));
        let falling = read([0.6, 0.6, 0.6, 0.1, 0.1]);
        assert_eq!((alpha_text(falling.beta), falling.grows, falling.does_not_grow, falling.verdict), ("0.414".to_string(), 3, 2, "unstable"));
        let late_rise = read([0.1, 0.1, 0.1, 0.1, 0.9]);
        assert_eq!((alpha_text(late_rise.beta), late_rise.grows, late_rise.does_not_grow, late_rise.verdict), ("0.214".to_string(), 1, 4, "unstable"));
        assert_eq!(band_text(late_rise.beta), "does_not_grow", "只在最高端翘起：β 读不随；首尾两点的 α 是 0.260（介于），不是 β");
    }

    #[test]
    fn one_growing_segment_among_between_ones_is_unstable_and_two_points_are_not_enough() {
        let one_grows = series_reading(&series_from_exponents([0.5, 0.4, 0.4, 0.4, 0.4]));
        assert_eq!((alpha_text(one_grows.beta), one_grows.grows, one_grows.verdict), ("0.414".to_string(), 1, "unstable"));
        assert_eq!(series_reading(&series_from_exponents([0.6; 5])[..2]).verdict, "not_enough_points", "读到的点少于三个不判");
        assert_eq!(series_reading(&series_from_exponents([0.6; 5])[..3]).verdict, "unstable", "三个点两段：g = 2 < 3，不满足稳");
    }

    #[test]
    fn the_controls_must_both_read_grows_or_the_primary_cell_is_void() {
        assert!(!controls_are_red(Some(0.3), Some(0.9)), "C1 的 β 是 0.3：不红");
        assert!(!controls_are_red(Some(0.9), Some(0.49)), "C3 的 β 不到 0.5：不红");
        assert!(!controls_are_red(Some(0.9), None), "读不到不算红");
        assert!(controls_are_red(Some(0.5), Some(0.9)));
        assert_eq!(series_route("stable_does_not_grow", "-"), "does_not_support_the_flip");
        assert_eq!(series_route("stable_grows", "control_not_red"), "series_not_made");
        assert_eq!(series_route("unstable", "-"), "unstable_the_cell_stays_open");
        assert_eq!(series_route("not_enough_points", "-"), "series_not_made");
    }

    /// 一个序列点的产物：状态里有 5 个第一次碰 K = 100 个单元的 W2、3 个 W1，步数固定、记录数照 `records`；`with_lg_row` 给 `C3` 一行含 lg 的行；扩展窗口的一个 W7 操作 `late_spike` 步。
    fn series_point(name: &str, records: u64, with_lg_row: bool, late_spike: Option<u64>) -> (PointArmReadings, Vec<Operation>) {
        let mut item = PointArmReadings { done: true, ..item_named(name, "head", 100) };
        for file in ["crates/singlefs-core/src/transaction.rs", "crates/singlefs-core/src/allocation_record_tree.rs", "crates/singlefs-core/src/allocator.rs"] {
            item.source_sha256.insert(file.to_string(), "00".to_string());
        }
        item.window = vec![Settle { rounds: 2, final_roles: 3, final_nodes: 4, round_work: 5, final_set_hash: "ab".to_string(), previous_records: records, ..settle_at("same_mount", "W1", 1, 0) }];
        let mut operations = Vec::new();
        for state in ["same_mount", "remounted"] {
            for index in 1..=3 {
                operations.push(operation_at(state, "W1", index, 100, 1, records));
            }
            for index in 1..=5 {
                operations.push(Operation { old_units: 100, ..operation_at(state, "W2", index, 100, 100, records) });
            }
        }
        if let Some(steps) = late_spike {
            operations.push(operation_at("same_mount", "W7", 1, steps, 1, records));
        }
        if with_lg_row {
            for operation in &mut operations {
                operation.lg_rows.insert("allocator_release_linear_find", (10, 10 * records, 10));
            }
        }
        (item, operations)
    }

    fn series_report_tweaked(with_lg_row: bool, late_spike: Option<u64>, tweak: impl FnOnce(&mut Vec<(PointArmReadings, Vec<Operation>)>, &mut PointArmReadings)) -> Lines {
        let totals = [14_000, 25_000, 45_000, 80_000, 140_000, 250_000];
        let mut built: Vec<(PointArmReadings, Vec<Operation>)> = HUNDRED_UNITS_PER_FILE_SERIES
            .iter()
            .zip(totals)
            .map(|(name, records)| series_point(name, records, with_lg_row, if *name == "files_1000_units_100" { late_spike } else { None }))
            .collect();
        let mut pin = built[1].0.clone();
        tweak(&mut built, &mut pin);
        let points = points_with(HUNDRED_UNITS_PER_FILE_SERIES.iter().zip(&built).map(|(name, (item, operations))| (*name, item, operations.clone())).collect());
        let mut out = Lines::new();
        report_hundred_units_per_file_series(&points, &[pin], &mut out);
        out
    }

    fn series_report(with_lg_row: bool, late_spike: Option<u64>) -> Lines {
        series_report_tweaked(with_lg_row, late_spike, |_, _| {})
    }

    #[test]
    fn the_hundred_units_per_file_series_reports_the_primary_cell_with_the_controls_red() {
        let out = series_report(true, None);
        let primary = lines_starting(&out, "name=series_cell ").into_iter().filter(|line| line.contains(" primary=yes ")).collect::<Vec<_>>();
        assert_eq!(primary.len(), 1, "主格只有一格：{primary:?}");
        assert!(primary[0].contains("state=same_mount tier=one_user_unit pick=later_peak filter=steady window=common_window arm=tested "), "{}", primary[0]);
        assert!(primary[0].contains(" points=6 ") && primary[0].contains(" beta=0.000 ") && primary[0].contains(" verdict=stable_does_not_grow"), "{}", primary[0]);
        assert_eq!(out.iter().filter(|line| line.starts_with("name=series_cell ")).count(), 192, "2 状态 × 2 档 × 3 取法 × 2 剔旗 × 2 窗 × 4 臂");
        assert!(out.iter().any(|line| line == "name=verdict quantity=hundred_units_per_file_pin settles=1 differing=0 matches=true first_differing=-"), "{out:?}");
        assert!(out.iter().any(|line| line == "name=verdict quantity=hundred_units_per_file_series outcome=stable_does_not_grow series_void=- points=6 route=does_not_support_the_flip"), "{out:?}");
    }

    #[test]
    fn the_hundred_units_per_file_series_is_void_when_the_linear_control_cannot_be_read_and_the_common_window_hides_the_extended_peak() {
        let out = series_report(false, Some(9_999_999));
        assert!(out.iter().any(|line| line.starts_with("name=verdict quantity=hundred_units_per_file_series ") && line.contains(" series_void=control_not_red ") && line.ends_with(" route=series_not_made")), "{out:?}");
        // 公共窗只看 W1–W4：高端的 W7 大操作不进公共窗的主格，进自己的窗
        let cell = |window: &str| {
            out.iter().find(|line| line.starts_with("name=series_cell state=same_mount tier=one_user_unit pick=later_peak filter=steady ") && line.contains(&format!(" window={window} arm=tested "))).expect("有这一格").clone()
        };
        assert!(cell("common_window").contains(" beta=0.000 "), "{}", cell("common_window"));
        assert!(!cell("own_window").contains(" beta=0.000 "), "自己的窗里高端有 W7 的大操作，峰与 β 要变：{}", cell("own_window"));
    }

    #[test]
    fn a_differing_pin_or_disagreeing_sources_void_the_series_and_an_unfinished_point_drops_out() {
        let pin_differs = series_report_tweaked(true, None, |_, pin| pin.window[0].final_set_hash = "zz".to_string());
        assert!(pin_differs.iter().any(|line| line == "name=verdict quantity=hundred_units_per_file_pin settles=1 differing=1 matches=false first_differing=same_mount/W1/1/0"), "{pin_differs:?}");
        assert!(pin_differs.iter().any(|line| line.starts_with("name=verdict quantity=hundred_units_per_file_series ") && line.contains(" series_void=pin_differs ")), "{pin_differs:?}");
        let sources_differ = series_report_tweaked(true, None, |built, _| {
            built[0].0.source_sha256.insert("crates/singlefs-core/src/allocator.rs".to_string(), "11".to_string());
        });
        assert!(sources_differ.iter().any(|line| line.starts_with("name=verdict quantity=sources_agree ") && line.ends_with(" files_with_more_than_one_value=1 agree=false")), "{sources_differ:?}");
        assert!(sources_differ.iter().any(|line| line.starts_with("name=verdict quantity=hundred_units_per_file_series ") && line.contains(" series_void=sources_differ ")), "{sources_differ:?}");
        let unfinished = series_report_tweaked(true, None, |built, _| built[2].0.done = false);
        assert!(unfinished.iter().any(|line| line.starts_with("name=verdict quantity=hundred_units_per_file_series ") && line.contains(" points=5 ")), "没做完的点不进序列：{unfinished:?}");
    }

    // ---- 对账分辨段 reconciliation_split（修订 3）----
    fn operation_key(state: &str, operation_name: &str, operation_index: u64) -> (String, String, u64) {
        (state.to_string(), operation_name.to_string(), operation_index)
    }

    /// 一份带镜像哈希与对账行的产物：同样的三个操作（同一次挂载 W1 第 1、2 次与重挂载 W1 第 1 次），`hash` 给每个操作的哈希、`rows` 给对账行（盘上变了, 计数记）。
    fn split_product(point: &str, arm: &str, hashes: [&str; 3], rows: [(u64, u64); 3], remount: (&str, &str)) -> PointArmReadings {
        let keys = [operation_key("same_mount", "W1", 1), operation_key("same_mount", "W1", 2), operation_key("remounted", "W1", 1)];
        let mut item = PointArmReadings { done: true, ..item_named(point, arm, 1) };
        item.images_after_the_operations = keys.iter().cloned().zip(hashes.iter().map(|hash| hash.to_string())).collect();
        if arm == "head" {
            item.reconciliation_rows = keys.iter().cloned().zip(rows).collect();
        }
        item.images_at_the_remount = BTreeMap::from([("before_the_crash".to_string(), remount.0.to_string()), ("after_the_mount".to_string(), remount.1.to_string())]);
        item
    }

    #[test]
    fn head_against_plain_names_the_first_differing_operation_and_the_remount_images() {
        let plain = split_product("files_1000", "plain", ["a", "b", "c"], [(0, 0); 3], ("image_before_the_crash", "image_after_the_mount"));
        let same = split_product("files_1000", "head", ["a", "b", "c"], [(0, 0); 3], ("image_before_the_crash", "image_after_the_mount"));
        let reading = head_against_plain(&same, &plain).expect("读得到");
        assert_eq!((reading.operations, reading.differing, reading.first_differing.as_str(), reading.before_the_crash_same, reading.after_the_mount_same), (3, 0, "-", true, true));
        let different = split_product("files_1000", "head", ["a", "b", "x"], [(0, 0); 3], ("image_before_the_crash", "other"));
        let reading = head_against_plain(&different, &plain).expect("读得到");
        assert_eq!((reading.differing, reading.first_differing.as_str(), reading.before_the_crash_same, reading.after_the_mount_same), (1, "remounted/W1/1", true, false));
        let without_images = PointArmReadings { done: true, ..item_named("files_1000", "plain", 1) };
        assert_eq!(head_against_plain(&same, &without_images), None, "缺新行记读不到");
    }

    #[test]
    fn the_placement_gap_counts_the_three_signs_and_names_the_first_undercount() {
        let item = split_product("files_1000", "head", ["a", "b", "c"], [(32, 32), (30, 32), (32, 31)], ("same_image", "same_image"));
        let reading = placement_gap(&item).expect("读得到");
        assert_eq!((reading.operations, reading.negative, reading.zero, reading.positive, reading.first_negative.as_str()), (3, 1, 1, 1, "remounted/W1/1"));
        let clean = split_product("base", "head", ["a", "b", "c"], [(32, 32), (32, 32), (4, 5)], ("same_image", "same_image"));
        let reading = placement_gap(&clean).expect("读得到");
        assert_eq!((reading.negative, reading.positive, reading.first_negative.as_str()), (0, 1, "-"), "放下去、盘上内容没变（正小）不算少记");
    }

    #[test]
    fn the_rerun_compares_the_hash_too_and_names_the_first_differing_publish() {
        let settle_with = |hash: &str, previous_records: u64| Settle { rounds: 2, final_roles: 20, final_nodes: 7, round_work: 32, final_set_hash: hash.to_string(), previous_records, ..settle_at("same_mount", "W1", 1, 0) };
        let mut today = item_named("base", "head", 1);
        let mut before = item_named("base", "head", 1);
        today.window = vec![settle_with("aa", 10), Settle { operation_index: 2, ..settle_with("bb", 10) }];
        before.window = vec![settle_with("aa", 10), Settle { operation_index: 2, ..settle_with("bb", 10) }];
        assert_eq!(settles_against(&today, &before), (2, vec![]));
        before.window[1].final_set_hash = "bc".to_string();
        assert_eq!(settles_against(&today, &before), (2, vec!["same_mount/W1/2/0".to_string()]), "只差哈希的一对也算不同");
        before.window[1].final_set_hash = "bb".to_string();
        before.window[0].previous_records = 11;
        assert_eq!(settles_against(&today, &before).1, vec!["same_mount/W1/1/0".to_string()], "previous_records 也比");
    }

    fn state_row_item(point: &str, arm: &str, rows: &[(&str, u64, u64)]) -> PointArmReadings {
        let mut item = item_named(point, arm, 1);
        for (state, index, cluster_segments) in rows {
            let fields = fields_of(&format!("E207RESULT name=state allocation_records=100 cluster_segments={cluster_segments} mapped_units=10 units=12"));
            item.judgements.insert(format!("{state}/W1/{index}/"), JudgementReadings { state: fields, ..JudgementReadings::default() });
        }
        item
    }

    fn state_row_points<'items>(today: &'items PointArmReadings, before: &'items PointArmReadings) -> (Points<'items>, Points<'items>) {
        let operations = |state_rows: &[(&str, u64)]| -> Vec<Operation> { state_rows.iter().map(|(state, index)| operation_at(state, "W1", *index, 1, 1, 1)).collect() };
        let keys = [("same_mount", 1), ("remounted", 1), ("remounted", 2)];
        (points_with(vec![("files_1000", today, operations(&keys))]), points_with(vec![("files_1000", before, operations(&keys))]))
    }

    #[test]
    fn the_state_rows_count_the_fields_and_the_same_mount_ones() {
        let before = state_row_item("files_1000", "p2a", &[("same_mount", 1, 2), ("remounted", 1, 2), ("remounted", 2, 2)]);
        let today = state_row_item("files_1000", "head", &[("same_mount", 1, 2), ("remounted", 1, 1), ("remounted", 2, 2)]);
        let (today_points, before_points) = state_row_points(&today, &before);
        let rows = state_rows_against_e206(&today_points["files_1000"], &before_points["files_1000"]).expect("读得到");
        assert_eq!((rows.operations, rows.differing_operations, rows.by_field, rows.same_mount_differing_operations, rows.first_differing.as_str()), (3, 1, [0, 1, 0, 0], 0, "remounted/W1/1"));
        let same_mount_differs = state_row_item("files_1000", "head", &[("same_mount", 1, 1), ("remounted", 1, 2), ("remounted", 2, 2)]);
        let (today_points, before_points) = state_row_points(&same_mount_differs, &before);
        assert_eq!(state_rows_against_e206(&today_points["files_1000"], &before_points["files_1000"]).expect("读得到").same_mount_differing_operations, 1);
        let mut without_the_field = before.clone();
        for judgement in without_the_field.judgements.values_mut() {
            judgement.state.remove("cluster_segments");
        }
        let (today_points, before_points) = state_row_points(&today, &without_the_field);
        assert_eq!(state_rows_against_e206(&today_points["files_1000"], &before_points["files_1000"]), None, "E206 的状态行缺 cluster_segments：记读不到，不拿别的字段顶替");
    }

    #[test]
    fn the_split_outcomes_follow_the_registered_order() {
        let facts = |head_differs: bool, gap_negative: bool, rerun_differs: bool, signature_of_a_changed_tree: bool, five_items_differ: bool| {
            Some(SplitFacts { head_differs, gap_negative, rerun_differs, signature_of_a_changed_tree, five_items_differ })
        };
        assert_eq!(reconciliation_split_outcome(facts(true, false, false, false, true)), "counting_wrong");
        assert_eq!(reconciliation_split_outcome(facts(false, true, false, false, true)), "counting_wrong");
        assert_eq!(reconciliation_split_outcome(facts(false, true, false, true, true)), "both", "c_gap 且 t_sign 是 both：次序上 both 不能被 counting_wrong 吃掉");
        assert_eq!(reconciliation_split_outcome(facts(false, false, false, true, true)), "tree_changed");
        assert_eq!(reconciliation_split_outcome(facts(false, false, false, false, true)), "neither_reading_differs");
        assert_eq!(reconciliation_split_outcome(facts(false, false, true, false, true)), "neither_not_reproducible");
        assert_eq!(reconciliation_split_outcome(facts(false, false, true, true, true)), "neither_not_reproducible", "没有 c_head、c_gap 而 c_rerun：树变了的记号也压不过");
        assert_eq!(reconciliation_split_outcome(facts(false, false, false, false, false)), "nothing_left_to_split");
        assert_eq!(reconciliation_split_outcome(None), "not_run");
    }

    /// 对账分辨段的整段读数：base 一处不差，files_1000 的 `same_mount_state_differs` 为真时同一次挂载里也有一处状态行不同，`remount_state_differs` 为真时重挂载里有一处不同。
    fn split_report(same_mount_state_differs: bool, remount_state_differs: bool) -> Lines {
        let hashes = ["a", "b", "c"];
        let build = |point: &str, arm: &str, rows: [(u64, u64); 3]| split_product(point, arm, hashes, rows, ("image_before_the_crash", "image_after_the_mount"));
        let mut items = vec![build("base", "head", [(32, 32); 3]), build("base", "plain", [(0, 0); 3]), build("files_1000", "head", [(32, 32); 3]), build("files_1000", "plain", [(0, 0); 3])];
        let window = |hash: &str| vec![Settle { rounds: 2, final_roles: 3, final_nodes: 4, round_work: 5, final_set_hash: hash.to_string(), ..settle_at("same_mount", "W1", 1, 0) }];
        for item in &mut items {
            item.window = window("h");
        }
        let previous: Vec<PointArmReadings> = items.iter().filter(|item| item.arm == "head").cloned().collect();
        let state_rows = |differs_in_same_mount: bool, differs_in_remount: bool| {
            [("same_mount", 1, if differs_in_same_mount { 1 } else { 2 }), ("remounted", 1, if differs_in_remount { 1 } else { 2 }), ("remounted", 2, 2)]
        };
        let today_state = |point: &str, same_mount: bool, remount: bool| state_row_item(point, "head", &state_rows(same_mount, remount));
        let retro_state = |point: &str| {
            let mut item = state_row_item(point, "p2a", &state_rows(false, false));
            item.window = window("h");
            item
        };
        for item in &mut items {
            if item.arm == "head" {
                let (same_mount, remount) = if item.point == "files_1000" { (same_mount_state_differs, remount_state_differs) } else { (false, false) };
                item.judgements = today_state(&item.point, same_mount, remount).judgements;
            }
        }
        let retro_items = [retro_state("base"), retro_state("files_1000")];
        let operations = |_item: &PointArmReadings| -> Vec<Operation> { [("same_mount", 1), ("remounted", 1), ("remounted", 2)].iter().map(|(state, index)| operation_at(state, "W1", *index, 1, 1, 1)).collect() };
        let points = points_with(items.iter().map(|item| (if item.arm == "plain" { if item.point == "base" { "base__plain" } else { "files_1000__plain" } } else { item.point.as_str() }, item, operations(item))).collect());
        let previous_points = points_with(previous.iter().map(|item| (item.point.as_str(), item, operations(item))).collect());
        let retro_points = points_with(retro_items.iter().map(|item| (item.point.as_str(), item, operations(item))).collect());
        let mut out = Lines::new();
        report_reconciliation_split(&points, &previous_points, &retro_points, &mut out);
        out
    }

    fn split_outcome_line(out: &[String], point: &str) -> String {
        out.iter().find(|line| line.starts_with(&format!("name=verdict quantity=reconciliation_split point={point} "))).expect("有总判行").clone()
    }

    #[test]
    fn the_split_report_reads_a_changed_tree_only_when_the_same_mount_stays_clean() {
        let out = split_report(false, true);
        assert!(split_outcome_line(&out, "base").contains(" outcome=negative_control_clean reason=-"), "{out:?}");
        assert!(split_outcome_line(&out, "files_1000").contains(" outcome=tree_changed "), "{}", split_outcome_line(&out, "files_1000"));
        assert!(out.iter().any(|line| line.contains("quantity=state_rows_against_e206 point=files_1000 operations=3 differing_operations=1 ")), "{out:?}");
        let with_the_control_differing = split_report(true, true);
        assert!(
            split_outcome_line(&with_the_control_differing, "files_1000").contains(" outcome=nothing_left_to_split "),
            "同一次挂载里状态行也不同：不是「只在重挂载之后」的树变了记号：{}",
            split_outcome_line(&with_the_control_differing, "files_1000")
        );
    }
}
