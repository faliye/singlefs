//! E203（准入之前扫类的步代价）的计数模型：纯算术，不 `use` 任何 `crates/` 里的东西（跑前登记
//! `research/prompts/e203-preregistration.md` 第五节「装置写在哪」第 2 条）。
//!
//! 做的事：B8（逐项按差量）与 B4（改射程）两条臂的步数式子、B8 的常驻内存与每次发布的维护、增长阶 α 与三档判法、
//! 用户关心的规模上的外推（Q4）与外推核对（Q5）、阳性与阴性对照、计时包含自检，以及读量的那一路（插了计数的副本）
//! 打出的 `E203RESULT` 行，逐段逐行算 B1 的数并与站点读数对账（登记第七节 ③2）。
//!
//! 报告另做的事：Q6、Q8、Q9 在用户关心规模上的值（与 Q4 同一个外推法，`user_scale` 行）；第七节 ③2 的对账（五行式子，别的行点名「没接」）；
//! 作废条款 8（补满五轮仍不满的覆盖写格整格拿掉并点名）；停机条款 3、4 与失败条款 F1–F6 的判读行。
//!
//! 用法：
//!   `e203-admission-scan-cost report <计数产物>… [--timing <计时产物>…]`  读产物、打 `name=verdict` 与各量的行
//!   `e203-admission-scan-cost anchors`                                  打第七节锚点（模型自己算的，与登记原样比）
//! 单测钉第七节锚点的绝对值：`cargo test --release --bin e203-admission-scan-cost`。

#![allow(dead_code, reason = "模型的库函数一部分只有单测与后续接线的报告用")]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::process::ExitCode;

// ---- 本地常量（出处是冻结副本的 文件:行，产物头里实现报的值与这里逐个比；对不上走停机条款第 2 条）----
/// 一槽 16 KiB（`crates/singlefs-format/src/lib.rs:14`）。
const SLOT_BYTES: u64 = 16384;
/// 一段 64 槽（`allocator.rs` 的 `CLUSTER_SEGMENT_SLOTS`）。
const SEGMENT_SLOTS: u64 = 64;
/// 6 MiB 环下单元区起点槽（`UnitAreaStart::following_the_journal_ring(6 << 20)`）。
const UNIT_AREA_START_SLOT: u64 = 1408;
/// 第一版固定两块盘。
const DISKS: u64 = 2;
/// inode 叶每片至多 233 条、至多 135 片（`transaction.rs:4205` 那一族的封顶）。
const INODE_LEAF_RECORDS: u64 = 233;
const INODE_LEAVES_AT_MOST: u64 = 135;
/// 墓碑每片至多 815 条、退役 959 条（第四节第 5 条）。
const TOMBSTONE_CONTAINER_RECORDS: u64 = 815;
/// 分配记录树固定点至多 64 轮（`transaction.rs:13141`）。
const SETTLING_ROUNDS_AT_MOST: u64 = 64;
/// 被量的覆盖写把第一个文件换成这么多字节（`common_admission_pool/mod.rs:42`）。
const OVERWRITE_BYTES: u64 = 2999;
/// 每盘 16 TiB 的盘字节数 2⁴⁴ 对应的单元区槽数 2³⁰ − 1408（登记读法第 5 条）。
const USER_SCALE_DEVICE_SLOTS: u64 = 1 << 30;

// ---- 判档（登记第六节）----
/// 增长阶三档：α ≥ 0.5 随这根轴涨，α ≤ 0.25 不随（至多对数级），之间不判。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Band {
    GrowsWithTheAxis,
    DoesNotGrow,
    Between,
}

fn alpha(low_quantity: f64, high_quantity: f64, low_position: f64, high_position: f64) -> f64 {
    (high_quantity / low_quantity).ln() / (high_position / low_position).ln()
}

fn band(alpha: f64) -> Band {
    if alpha >= 0.5 {
        Band::GrowsWithTheAxis
    } else if alpha <= 0.25 {
        Band::DoesNotGrow
    } else {
        Band::Between
    }
}

/// 五轮每轮一个 α，五轮落在同一档才记那一档，否则 `None`（不稳定）。
fn band_of_the_rounds(alphas: &[f64]) -> Option<Band> {
    let first = band(*alphas.first()?);
    alphas
        .iter()
        .all(|growth| band(*growth) == first)
        .then_some(first)
}

// ---- 整数函数（第七节 ③5）----
/// ⌈log₂(n + 1)⌉，lg(0) = 0。
fn lg(value: u64) -> u64 {
    u64::from(u64::BITS - value.leading_zeros())
}

/// 每层 64 叉的分层位图要几层：最小的 l ≥ 1 使 64ˡ ≥ n。
fn levels_64(value: u64) -> u64 {
    let mut level = 1_u32;
    while 64_u64.pow(level) < value {
        level += 1;
    }
    u64::from(level)
}

/// 分层位图各层字数之和，到只剩 1 字为止。
fn summary_words(value: u64) -> u64 {
    let mut words = 0;
    let mut level_size = value;
    loop {
        level_size = level_size.div_ceil(64);
        words += level_size;
        if level_size == 1 {
            return words;
        }
    }
}

fn segments_of(unit_area_slots: u64) -> u64 {
    unit_area_slots.div_ceil(SEGMENT_SLOTS)
}

/// 一块盘那八张表的元素数：三张 U 位、五张按段表各 ⌈U/64⌉。
fn elements_per_disk(unit_area_slots: u64) -> u64 {
    3 * unit_area_slots + 5 * segments_of(unit_area_slots)
}

/// 两块盘的整份拷贝里只由 U 定的那一部分的元素数（第七节 ③1）。
fn allocator_copy_elements_from_the_unit_area(unit_area_slots: u64) -> u64 {
    DISKS * elements_per_disk(unit_area_slots)
}

fn bytes_per_disk(unit_area_slots: u64) -> u64 {
    3 * unit_area_slots + 40 * segments_of(unit_area_slots)
}

fn device_bytes(unit_area_slots: u64) -> u64 {
    (UNIT_AREA_START_SLOT + unit_area_slots) * SLOT_BYTES
}

/// 用户关心的规模：每盘 16 TiB 的单元区槽数。
fn user_scale_unit_area_slots() -> u64 {
    USER_SCALE_DEVICE_SLOTS - UNIT_AREA_START_SLOT
}

fn tombstone_containers(unlinks: u64) -> u64 {
    unlinks.div_ceil(TOMBSTONE_CONTAINER_RECORDS)
}

// ---- B8（逐项按差量）：登记第五节「B8 每一行的做法与步数式子」----
/// B8 式子吃的输入：这一次发布的差量与调用次数（量的那一路读到的）。字段名与登记第五节、第七节 ③3 一致。
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
    /// 每轮内容变了的叶数 λ_ar（一轮一项）。
    changed_leaves_per_round: Vec<u64>,
    allocation_tree_levels: u64,
    allocation_records: u64,
    segments_per_disk: u64,
    units: u64,
    mapped_units: u64,
    deadlist_entries: u64,
    /// 各行在这一段里被调了几次（`c4`、`c5`……）。
    calls: BTreeMap<&'static str, u64>,
    intersecting_records: u64,
    dirty_per_call: Vec<u64>,
    /// 全部轮里每棵多层码 2 树进出的 key 数与树高：(δ, h)，一棵树一轮一项。
    code_two_trees_over_all_rounds: Vec<(u64, u64)>,
    /// 全部轮里三棵容器树重写与新开的容器、节点数，一棵树一轮一项。
    container_rewrites_over_all_rounds: Vec<u64>,
    moved_entries: u64,
    central_mapping_levels: u64,
    /// 按身份建的索引上 deadlist 判生死那一圈的查询次数乘各自的 ⌈log₂(索引项数 + 1)⌉ 之和。
    deadlist_query_levels: u64,
    deadlist_changed_entries: u64,
    /// 释放判定那一圈的同一个和。
    release_query_levels: u64,
    changed_slots_and_records_per_round: Vec<u64>,
    /// 全部轮里进出 `mapped_units` 的条目数之和。
    mapped_units_changed_over_all_rounds: u64,
    /// 整表那一步的按角色查询的同一个和。
    mapped_units_query_levels: u64,
}

impl ByDeltaInput {
    fn calls_in_the_row(&self, row: &str) -> u64 {
        self.calls.get(row).copied().unwrap_or(0)
    }
}

/// B8 每一行一次发布准入之前的步数，次序同登记第五节。
fn by_delta_rows(input: &ByDeltaInput) -> Vec<(&'static str, u64)> {
    let settling_rounds = input.settling_rounds;
    let per_round_trees: u64 = input
        .code_two_trees_over_all_rounds
        .iter()
        .map(|(delta, height)| delta * height)
        .sum();
    let container_rewrites: u64 = input.container_rewrites_over_all_rounds.iter().sum();
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
            input.changed_leaves_per_round.iter().sum::<u64>() * input.allocation_tree_levels,
        ),
        (
            "allocation_record_tree_plan",
            input.changed_leaves_per_round.iter().sum::<u64>() * input.allocation_tree_levels,
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
        ("mapped_units_of_container_trees", container_rewrites),
        (
            "moved_entries_path_tail_filter",
            settling_rounds * input.moved_entries * input.central_mapping_levels,
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
            input.mapped_units_changed_over_all_rounds + input.mapped_units_query_levels,
        ),
    ]
}

fn by_delta_total(input: &ByDeltaInput) -> u64 {
    by_delta_rows(input).iter().map(|(_, steps)| steps).sum()
}

/// 登记第七节 ③3 的合成输入 Δ0。
fn delta_zero() -> ByDeltaInput {
    let calls = [
        ("inode_record_linear_find", 0),
        ("files_other_than_the_first_plan", 2),
        ("file_other_than_the_first_linear_find", 2),
        ("inodes_with_an_extent_entry_set", 2),
        ("extent_upper_segment_plan", 2),
        ("extent_roles_rewritten_scan", 2),
        ("root_header_linear_find", 8),
        ("checkpoint_cost_accounting_tree_plan", 2),
        ("cluster_segment_sum", 4),
        ("inode_number_watermark_steps", 4),
        ("allocator_release_linear_find", 10),
        ("user_data_slot_linear_scan", 2),
        ("record_intersection_filter", 40),
        ("empty_segment_linear_scan", 4),
        ("fallback_slot_linear_scan", 0),
        ("accounting_entries_of_this_publish", 2),
        ("instance_table_chain", 0),
    ];
    ByDeltaInput {
        inode_leaf_containers: 8,
        other_files: 1000,
        touched_files: 1,
        lower_nodes_of_touched_files: 1,
        extent_upper_levels: 2,
        extent_upper_nodes: 9,
        accounting_rows_of_the_generation: 15,
        accounting_tree_levels: 2,
        settling_rounds: 2,
        changed_leaves_per_round: vec![4, 4],
        allocation_tree_levels: 3,
        allocation_records: 2000,
        segments_per_disk: 262_144,
        units: 3000,
        mapped_units: 1500,
        deadlist_entries: 0,
        calls: calls.into_iter().collect(),
        intersecting_records: 40,
        dirty_per_call: vec![40, 40, 40, 40],
        code_two_trees_over_all_rounds: vec![(15, 2), (0, 1), (2, 2), (15, 2), (0, 1), (2, 2)],
        container_rewrites_over_all_rounds: vec![0, 0],
        moved_entries: 0,
        central_mapping_levels: 2,
        deadlist_query_levels: 0,
        deadlist_changed_entries: 0,
        release_query_levels: 6 * lg(3000) + 6 * lg(1500),
        changed_slots_and_records_per_round: vec![30, 30],
        mapped_units_changed_over_all_rounds: 4,
        mapped_units_query_levels: 0,
    }
}

// ---- B8 的常驻内存与每次发布的维护（登记第五节两段）----
#[derive(Clone, Debug)]
struct MemoryInput {
    allocation_records: u64,
    segments_per_disk: u64,
    units: u64,
    mapped_units: u64,
    /// 按号索引里的文件数（别的文件加第一个文件，登记第七节 ③4 取 1001）。
    files_in_the_index: u64,
    extent_upper_nodes: u64,
    deadlist_entries: u64,
    accounting_rows_of_the_generation: u64,
    unit_bytes: u64,
    key_bytes: u64,
    accounting_row_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MemoryReport {
    record_index: u64,
    segment_summaries: u64,
    units_by_role: u64,
    mapped_units_by_role: u64,
    files_by_inode: u64,
    inodes_with_an_extent_entry: u64,
    extent_upper_nodes: u64,
    deadlist_entries: u64,
    root_head_positions: u64,
    accounting_rows_of_the_generation: u64,
}

impl MemoryReport {
    fn payload(&self) -> u64 {
        self.record_index
            + self.segment_summaries
            + self.units_by_role
            + self.mapped_units_by_role
            + self.files_by_inode
            + self.inodes_with_an_extent_entry
            + self.extent_upper_nodes
            + self.deadlist_entries
            + self.root_head_positions
            + self.accounting_rows_of_the_generation
    }
}

/// 一条分配记录的索引项 24 字节；其余按登记第五节「B8 的常驻内存」。
fn by_delta_memory(memory_input: &MemoryInput) -> MemoryReport {
    MemoryReport {
        record_index: memory_input.allocation_records * 24,
        segment_summaries: 3 * summary_words(memory_input.segments_per_disk) * 8 * DISKS,
        units_by_role: memory_input.units * (memory_input.unit_bytes + 8),
        mapped_units_by_role: memory_input.mapped_units * (memory_input.unit_bytes + 8),
        files_by_inode: memory_input.files_in_the_index * 16,
        inodes_with_an_extent_entry: memory_input.files_in_the_index * 8,
        extent_upper_nodes: memory_input.extent_upper_nodes * 8,
        deadlist_entries: memory_input.deadlist_entries * (memory_input.key_bytes + 8),
        root_head_positions: 4 * 8,
        accounting_rows_of_the_generation: memory_input.accounting_rows_of_the_generation
            * memory_input.accounting_row_bytes,
    }
}

#[derive(Clone, Debug, Default)]
struct MaintenanceInput {
    records_changed: u64,
    segments_flipped: u64,
    units_delta: u64,
    mapped_delta: u64,
    files_delta: u64,
    upper_delta: u64,
    deadlist_delta: u64,
}

/// 每次发布装单元那一步的维护步数（在准入之后）。
fn by_delta_maintenance(memory_input: &MemoryInput, maintenance: &MaintenanceInput) -> u64 {
    maintenance.records_changed * lg(memory_input.allocation_records)
        + maintenance.segments_flipped * levels_64(memory_input.segments_per_disk)
        + maintenance.units_delta * lg(memory_input.units)
        + maintenance.mapped_delta * lg(memory_input.mapped_units)
        + maintenance.files_delta * lg(memory_input.files_in_the_index)
        + maintenance.upper_delta * lg(memory_input.extent_upper_nodes)
        + maintenance.deadlist_delta * lg(memory_input.deadlist_entries)
        + memory_input.accounting_rows_of_the_generation
}

fn undo_log_bytes(changed_allocation_records: u64) -> u64 {
    changed_allocation_records * 24
}

// ---- B4（改射程）：登记第五节「B4 的式子」----
/// 一行在一段里的步数：键是 (段标记, 行号)。
type StepsByRow = BTreeMap<(&'static str, &'static str), u64>;

/// B4 在 P0、S1、S2 三段里的步数。P0、S1 两段每行取 B8 一列；S2 里在 P0 或 S1 里调到过的行也取 B8 一列（改的是函数，不是调用点），
/// 其余 S2 行取 B1 量的步数。
fn narrowed_steps_by_segment(
    today: &StepsByRow,
    by_delta: &StepsByRow,
    called_before_the_between_segment: &BTreeSet<&'static str>,
) -> [u64; 3] {
    let mut totals = [0_u64; 3];
    for (index, segment) in ["caller", "before_landing", "between"]
        .into_iter()
        .enumerate()
    {
        let rows: BTreeSet<&'static str> = today
            .keys()
            .chain(by_delta.keys())
            .filter(|(segment_of_the_cell, _)| *segment_of_the_cell == segment)
            .map(|(_, row)| *row)
            .collect();
        for row in rows {
            let uses_by_delta =
                segment != "between" || called_before_the_between_segment.contains(row);
            let source = if uses_by_delta { by_delta } else { today };
            totals[index] += source.get(&(segment, row)).copied().unwrap_or(0);
        }
    }
    totals
}

// ---- 阳性对照与阴性对照（登记第五节）----
/// B8 阳性对照：s1、s24 两行退回 B1 量的步数，别的行照 B8，得到退化后的合计。
fn degraded_by_delta_total(
    by_delta_other_rows: u64,
    today_whole_copy_before: u64,
    today_whole_copy_in_a_round: u64,
) -> u64 {
    let retired_round_copy = today_whole_copy_in_a_round;
    by_delta_other_rows + today_whole_copy_before + retired_round_copy
}

/// 阳性对照判过：退化后合计的 α(U) 不小于门槛（B8 取 0.9）。
fn positive_control_passes(
    total_at_the_low_end: u64,
    total_at_the_high_end: u64,
    threshold: f64,
) -> bool {
    let low = to_f64(total_at_the_low_end);
    let high = to_f64(total_at_the_high_end);
    alpha(low, high, to_f64(1 << 14), to_f64(1 << 24)) >= threshold
}

/// 阴性对照：s33、s12 在 U 轴两端 |α| ≤ 0.1。
fn negative_control_passes(steps_at_the_low_end: u64, steps_at_the_high_end: u64) -> bool {
    alpha(
        to_f64(steps_at_the_low_end),
        to_f64(steps_at_the_high_end),
        to_f64(1 << 14),
        to_f64(1 << 24),
    )
    .abs()
        <= 0.1
}

#[allow(
    clippy::cast_precision_loss,
    reason = "步数与纳秒远小于 2⁵³，转浮点不丢位"
)]
fn to_f64(value: u64) -> f64 {
    value as f64
}

// ---- 计时的包含自检与 Q4、Q5（登记第六节）----
/// 一轮里某一段 Σ 独占计时不大于段挂钟才有效（作废条款第 5 条）。
fn round_is_valid(exclusive_sum_nanos: u64, segment_nanos: u64) -> bool {
    exclusive_sum_nanos <= segment_nanos
}

/// τ = 一行的独占计时 ÷ 步数，取那一行步数最大的那个量过的点。points 是 (步数, 纳秒)。
fn tau_nanos_per_step(points: &[(u64, u64)]) -> f64 {
    let (steps, nanos) = points
        .iter()
        .max_by_key(|(steps, _)| *steps)
        .copied()
        .unwrap_or((1, 0));
    to_f64(nanos) / to_f64(steps.max(1))
}

/// 沿一根轴用最后两点的斜率线性外推。
fn linear_extrapolation(
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
enum FormulaCell {
    AtLeastOneMillisecond,
    BelowOneMillisecond,
    Unstable,
}

/// Q4 一格：五个 T 都 ≥ 2 毫秒记「≥ 1 毫秒」，都 ≤ 0.5 毫秒记「< 1 毫秒」，否则不稳定。
fn one_millisecond_cell(nanos_of_each_round: &[f64]) -> FormulaCell {
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
fn extrapolation_is_checked(
    steps_extrapolated: f64,
    steps_measured: f64,
    nanos_ratio: f64,
) -> bool {
    (steps_extrapolated - steps_measured).abs() <= 0.10 * steps_measured
        && (0.5..=2.0).contains(&nanos_ratio)
}

// ---- 读量的那一路打出的 E203RESULT 行 ----
type Fields = BTreeMap<String, String>;

fn parse_line(line: &str) -> Option<Fields> {
    let rest = line.strip_prefix("E203RESULT ")?;
    let fields: Fields = rest
        .split_whitespace()
        .filter_map(|pair| pair.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    fields.contains_key("name").then_some(fields)
}

fn number(fields: &Fields, key: &str) -> Option<u64> {
    fields.get(key)?.parse().ok()
}

/// 一个点：四根轴的取值（登记第五节取样点）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PointAxes {
    unit_area_slots: u64,
    unlinks: u64,
    data_units: u64,
    files: u64,
}

/// 某个点某个状态某种发布某一轮某一段某一行的读数。
#[derive(Clone, Debug, Default)]
struct RowCell {
    calls: u64,
    steps: u64,
    scale: u64,
    exclusive_nanos: u64,
    queries: u64,
    query_levels: u64,
}

/// 点、状态、发布种类、轮、段、行。
type CellKey = (String, String, String, u64, String, String);
/// 点、状态、发布种类、轮。
type RoundKey = (String, String, String, u64);

#[derive(Clone, Default)]
struct Product {
    constants: Fields,
    sizes: Fields,
    header: Fields,
    points: BTreeMap<String, PointAxes>,
    rows: BTreeMap<CellKey, RowCell>,
    states: BTreeMap<RoundKey, Fields>,
    publishes: BTreeMap<RoundKey, Fields>,
    deltas: Vec<Fields>,
    segment_nanos: BTreeMap<(String, String, String, u64, String), u64>,
    items_done: BTreeSet<String>,
    /// 被量的 unlink 每一轮判了几次：(点, 状态, 轮) → 次数。
    unlink_publishes: BTreeMap<(String, String, u64), u64>,
    /// 历史第 4 步 u 次 unlink 每次判了几次：点 → (次数, 最少, 最多)。
    history_unlinks: BTreeMap<String, (u64, u64, u64)>,
    /// 被量的覆盖写补满五轮用了几次尝试：(点, 状态) → (取到, 尝试)。
    rounds_taken: BTreeMap<(String, String), (u64, u64)>,
}

fn read_product(text: &str) -> Product {
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
enum RowKind {
    Scan,
    Copy,
    NegativeControl,
}

const ROW_TABLE: [(&str, RowKind); 35] = [
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
];

fn all_rows() -> Vec<(&'static str, RowKind)> {
    ROW_TABLE.to_vec()
}

fn is_a_scan_row(name: &str) -> bool {
    ROW_TABLE
        .iter()
        .any(|(row, kind)| *row == name && *kind == RowKind::Scan)
}

// ---- 读数的取法 ----
/// 清单外归口的一行（插计数时没挂归口守卫的步记在这里，不丢）。
const UNLISTED_ROW: &str = "unlisted_index_or_plan_work";
/// 准入之前的三段（登记读法第 1 条）：调用方段、落点判之前、两道之间。
const SEGMENTS_BEFORE_ADMISSION: [&str; 3] = ["caller", "before_landing", "between"];
/// 落点判之前的两段（B4 臂的射程）。
const SEGMENTS_BEFORE_THE_SECOND_JUDGEMENT: [&str; 2] = ["caller", "before_landing"];
const STATES: [&str; 2] = ["same_mount", "remounted"];
const KINDS: [&str; 3] = ["overwrite", "unlink_intent", "unlink_final"];
const AXES: [&str; 4] = ["U", "u", "D", "F"];
/// 计时每个点量几轮。
const ROUNDS: u64 = 5;

impl Product {
    fn row_cell(
        &self,
        point: &str,
        state: &str,
        kind: &str,
        round: u64,
        segment: &str,
        row: &str,
    ) -> RowCell {
        self.rows
            .get(&(
                point.to_string(),
                state.to_string(),
                kind.to_string(),
                round,
                segment.to_string(),
                row.to_string(),
            ))
            .cloned()
            .unwrap_or_default()
    }

    fn steps_of(&self, at: &RoundKey, segments: &[&str], row: &str) -> u64 {
        segments
            .iter()
            .map(|segment| self.row_cell(&at.0, &at.1, &at.2, at.3, segment, row).steps)
            .sum()
    }

    fn calls_of(&self, at: &RoundKey, segments: &[&str], row: &str) -> u64 {
        segments
            .iter()
            .map(|segment| self.row_cell(&at.0, &at.1, &at.2, at.3, segment, row).calls)
            .sum()
    }

    fn nanos_of(&self, at: &RoundKey, segments: &[&str], row: &str) -> u64 {
        segments
            .iter()
            .map(|segment| {
                self.row_cell(&at.0, &at.1, &at.2, at.3, segment, row)
                    .exclusive_nanos
            })
            .sum()
    }

    fn query_levels_of(&self, at: &RoundKey, segments: &[&str], row: &str) -> u64 {
        segments
            .iter()
            .map(|segment| {
                self.row_cell(&at.0, &at.1, &at.2, at.3, segment, row)
                    .query_levels
            })
            .sum()
    }

    fn wall_of(&self, at: &RoundKey, segment: &str) -> Option<u64> {
        self.segment_nanos
            .get(&(
                at.0.clone(),
                at.1.clone(),
                at.2.clone(),
                at.3,
                segment.to_string(),
            ))
            .copied()
    }

    fn state_value(&self, at: &RoundKey, key: &str) -> Option<u64> {
        number(self.states.get(at)?, key)
    }

    /// 这一轮这一发布里 `key` 的差量读数：(固定点第几轮, 值)，只取点名的几段。
    fn deltas_of(&self, at: &RoundKey, key: &str, segments: &[&str]) -> Vec<(u64, u64)> {
        self.deltas
            .iter()
            .filter(|fields| {
                fields.get("point") == Some(&at.0)
                    && fields.get("state") == Some(&at.1)
                    && fields.get("kind") == Some(&at.2)
                    && number(fields, "round") == Some(at.3)
                    && fields.get("key").map(String::as_str) == Some(key)
                    && fields
                        .get("segment")
                        .is_some_and(|segment| segments.contains(&segment.as_str()))
            })
            .map(|fields| {
                (
                    number(fields, "fixed_point_round").unwrap_or(0),
                    number(fields, "value").unwrap_or(0),
                )
            })
            .collect()
    }

    /// 这一轮这一发布有没有进过 `caller` 段（经调用方进门）。
    fn had_caller_segment(&self, at: &RoundKey) -> Option<bool> {
        Some(self.publishes.get(at)?.get("had_caller_segment")? == "true")
    }

    fn rows_exist(&self, at: &RoundKey) -> bool {
        self.rows
            .keys()
            .any(|key| key.0 == at.0 && key.1 == at.1 && key.2 == at.2 && key.3 == at.3)
    }
}

/// 作废条款 8：被量的覆盖写判了不止一次、补满之后取到的轮数仍不足五轮，那个点那种状态的覆盖写那一格量不到。
/// 交回 (点, 状态, 取到几轮, 尝试几次)。
fn cells_not_measurable(product: &Product) -> Vec<(String, String, u64, u64)> {
    product
        .rounds_taken
        .iter()
        .filter(|(_, (taken, _))| *taken < ROUNDS)
        .map(|((point, state), (taken, attempts))| {
            (point.clone(), state.clone(), *taken, *attempts)
        })
        .collect()
}

/// 把量不到的那几格的覆盖写读数整格拿掉（它们不当 0 进任何一个量，别的格不动）：规模读数、各行读数、差量、段挂钟一起拿。
fn without_the_cells_not_measurable(
    product: &Product,
    cells: &[(String, String, u64, u64)],
) -> Product {
    let mut kept = product.clone();
    let is_dropped = |point: &str, state: &str, kind: &str| {
        kind == "overwrite"
            && cells
                .iter()
                .any(|(cell_point, cell_state, _, _)| cell_point == point && cell_state == state)
    };
    kept.rows
        .retain(|key, _| !is_dropped(&key.0, &key.1, &key.2));
    kept.states
        .retain(|key, _| !is_dropped(&key.0, &key.1, &key.2));
    kept.publishes
        .retain(|key, _| !is_dropped(&key.0, &key.1, &key.2));
    kept.segment_nanos
        .retain(|key, _| !is_dropped(&key.0, &key.1, &key.2));
    kept.deltas.retain(|fields| {
        let text_of = |key: &str| fields.get(key).map_or("", String::as_str).to_string();
        !is_dropped(&text_of("point"), &text_of("state"), &text_of("kind"))
    });
    kept
}

fn round_key(point: &str, state: &str, kind: &str, round: u64) -> RoundKey {
    (
        point.to_string(),
        state.to_string(),
        kind.to_string(),
        round,
    )
}

fn scan_rows() -> Vec<&'static str> {
    ROW_TABLE
        .iter()
        .filter(|(_, kind)| *kind == RowKind::Scan)
        .map(|(row, _)| *row)
        .collect()
}

/// 一个点某状态某发布某轮里扫类各行在点名几段里的步数之和。
fn scan_steps_at(product: &Product, at: &RoundKey, segments: &[&str]) -> u64 {
    scan_rows()
        .iter()
        .map(|row| product.steps_of(at, segments, row))
        .sum()
}

/// 同上，取第 1 轮（计数是确定的）。
fn scan_steps(product: &Product, point: &str, state: &str, kind: &str, segments: &[&str]) -> u64 {
    scan_steps_at(product, &round_key(point, state, kind, 1), segments)
}

// ---- 一根轴的两端 ----
/// 增长阶的低端：U 轴取 2¹⁴；u、D、F 三轴从 x = 10 起算（登记读法第 6 条，x = 1 那一点只报数）。
fn axis_lowest_position(axis: &str) -> u64 {
    if axis == "U" {
        1 << 14
    } else {
        10
    }
}

/// 增长阶的高端：U 轴 2²⁴，别的三轴 10⁴；更大的点（第三段、第二段的 X1 等）不进 α。
fn axis_highest_position(axis: &str) -> u64 {
    if axis == "U" {
        1 << 24
    } else {
        10_000
    }
}

/// 这根轴上在 α 范围里的点，按轴值升序。
fn axis_points_for_alpha<'borrowed>(
    product: &'borrowed Product,
    axis: &str,
) -> Vec<(&'borrowed String, &'borrowed PointAxes)> {
    axis_points(product, axis)
        .into_iter()
        .filter(|(_, point_axes)| {
            (axis_lowest_position(axis)..=axis_highest_position(axis))
                .contains(&axis_value(axis, point_axes))
        })
        .collect()
}

/// 一个量在一根轴上的读数。
#[derive(Clone, Copy, Debug, PartialEq)]
enum AxisReading {
    /// 这根轴上没有两个点。
    NoEnds,
    /// 有一端是 0：不算 α，写「一端为 0」。
    OneEndIsZero(u64, u64),
    Alpha(u64, u64, f64),
}

fn read_axis(product: &Product, axis: &str, quantity: &dyn Fn(&str) -> u64) -> AxisReading {
    let points = axis_points_for_alpha(product, axis);
    let (Some(low), Some(high)) = (points.first(), points.last()) else {
        return AxisReading::NoEnds;
    };
    if low.0 == high.0 {
        return AxisReading::NoEnds;
    }
    let (low_value, high_value) = (quantity(low.0), quantity(high.0));
    if low_value == 0 || high_value == 0 {
        return AxisReading::OneEndIsZero(low_value, high_value);
    }
    AxisReading::Alpha(
        low_value,
        high_value,
        alpha(
            to_f64(low_value),
            to_f64(high_value),
            to_f64(axis_value(axis, low.1)),
            to_f64(axis_value(axis, high.1)),
        ),
    )
}

fn reading_text(reading: &AxisReading) -> String {
    match reading {
        AxisReading::NoEnds => "low_end=NA high_end=NA alpha=no_two_points".to_string(),
        AxisReading::OneEndIsZero(low, high) => {
            format!("low_end={low} high_end={high} alpha=one_end_is_zero")
        }
        AxisReading::Alpha(low, high, growth) => {
            format!(
                "low_end={low} high_end={high} alpha={growth:.6} band={:?}",
                band(*growth)
            )
        }
    }
}

fn reading_band(reading: &AxisReading) -> Option<Band> {
    match reading {
        AxisReading::Alpha(_, _, growth) => Some(band(*growth)),
        _ => None,
    }
}

// ---- 报告 ----
/// B0 = (2¹⁷, 10, 10, 10)。
const BASE_POINT: PointAxes = PointAxes {
    unit_area_slots: 1 << 17,
    unlinks: 10,
    data_units: 10,
    files: 10,
};

fn axis_points<'borrowed>(
    product: &'borrowed Product,
    axis: &str,
) -> Vec<(&'borrowed String, &'borrowed PointAxes)> {
    let mut points: Vec<(&String, &PointAxes)> = product
        .points
        .iter()
        .filter(|(_, point_axes)| match axis {
            "U" => {
                (point_axes.unlinks, point_axes.data_units, point_axes.files)
                    == (BASE_POINT.unlinks, BASE_POINT.data_units, BASE_POINT.files)
            }
            "u" => {
                (
                    point_axes.unit_area_slots,
                    point_axes.data_units,
                    point_axes.files,
                ) == (
                    BASE_POINT.unit_area_slots,
                    BASE_POINT.data_units,
                    BASE_POINT.files,
                )
            }
            "D" => {
                (
                    point_axes.unit_area_slots,
                    point_axes.unlinks,
                    point_axes.files,
                ) == (
                    BASE_POINT.unit_area_slots,
                    BASE_POINT.unlinks,
                    BASE_POINT.files,
                )
            }
            _ => {
                (
                    point_axes.unit_area_slots,
                    point_axes.unlinks,
                    point_axes.data_units,
                ) == (
                    BASE_POINT.unit_area_slots,
                    BASE_POINT.unlinks,
                    BASE_POINT.data_units,
                )
            }
        })
        .collect();
    points.sort_by_key(|(_, point_axes)| match axis {
        "U" => point_axes.unit_area_slots,
        "u" => point_axes.unlinks,
        "D" => point_axes.data_units,
        _ => point_axes.files,
    });
    points
}

fn axis_value(axis: &str, point_axes: &PointAxes) -> u64 {
    match axis {
        "U" => point_axes.unit_area_slots,
        "u" => point_axes.unlinks,
        "D" => point_axes.data_units,
        _ => point_axes.files,
    }
}

fn constants_report(product: &Product, out: &mut String) {
    let local: [(&str, u64); 9] = [
        ("slot_bytes", SLOT_BYTES),
        ("segment_slots", SEGMENT_SLOTS),
        ("unit_area_start_slot", UNIT_AREA_START_SLOT),
        ("disks", DISKS),
        ("inode_leaf_records", INODE_LEAF_RECORDS),
        ("inode_leaves_at_most", INODE_LEAVES_AT_MOST),
        ("tombstone_container_records", TOMBSTONE_CONTAINER_RECORDS),
        ("settling_rounds_at_most", SETTLING_ROUNDS_AT_MOST),
        ("overwrite_bytes", OVERWRITE_BYTES),
    ];
    for (key, value) in local {
        let measured = number(&product.constants, key);
        let _ = writeln!(
            out,
            "name=constant_check key={key} model={value} implementation={} match={}",
            measured.map_or("NOT_READ".to_string(), |maintenance| maintenance
                .to_string()),
            measured == Some(value)
        );
    }
}

/// Q1、Q2 与第七节 ③1：计数产物里每个点每个状态每种发布的扫类步数，每根轴的 α。
fn counts_report(product: &Product, out: &mut String) {
    constants_report(product, out);
    for state in STATES {
        for kind in KINDS {
            for axis in AXES {
                let quantity = |point: &str| {
                    scan_steps(product, point, state, kind, &SEGMENTS_BEFORE_ADMISSION)
                };
                let reading = read_axis(product, axis, &quantity);
                if reading != AxisReading::NoEnds {
                    let _ = writeln!(
                        out,
                        "name=q2 axis={axis} state={state} kind={kind} {}",
                        reading_text(&reading)
                    );
                }
                for (row, _) in all_rows() {
                    let by_row = |point: &str| {
                        product.steps_of(
                            &round_key(point, state, kind, 1),
                            &SEGMENTS_BEFORE_ADMISSION,
                            row,
                        )
                    };
                    let reading = read_axis(product, axis, &by_row);
                    if matches!(reading, AxisReading::Alpha(..)) {
                        let _ = writeln!(
                            out,
                            "name=q1 axis={axis} state={state} kind={kind} row={row} {}",
                            reading_text(&reading)
                        );
                    }
                }
            }
        }
    }
    // 第七节 ③1：整份拷贝每份的元素数里只由 U 定的那一部分，与站点读到的整份元素数比（站点读数 ≥ U 那一部分，多出的是记录等小表）。
    let mut disagreements = 0_u64;
    for ((point, state, kind, round, segment, row), cell) in &product.rows {
        let copy_row = row == "whole_allocator_copy_before_the_judgement"
            || row == "whole_allocator_copy_in_a_settling_round";
        let (true, Some(axes)) = (copy_row && cell.calls > 0, product.points.get(point)) else {
            continue;
        };
        let per_copy = cell.scale / cell.calls;
        let unit_area_part = allocator_copy_elements_from_the_unit_area(axes.unit_area_slots);
        let _ = writeln!(
            out,
            "name=copy_elements point={point} state={state} kind={kind} round={round} segment={segment} row={row} calls={} elements_per_copy={per_copy} from_the_unit_area={unit_area_part} other_tables={}",
            cell.calls,
            per_copy.saturating_sub(unit_area_part)
        );
        if per_copy < unit_area_part || cell.scale % cell.calls != 0 {
            disagreements += 1;
        }
    }
    let _ = writeln!(
        out,
        "name=verdict quantity=three_one_copy_elements disagreements={disagreements}"
    );
    let unlisted: u64 = product
        .rows
        .iter()
        .filter(|(key, _)| {
            key.5 == UNLISTED_ROW && SEGMENTS_BEFORE_ADMISSION.contains(&key.4.as_str())
        })
        .map(|(_, cell)| cell.steps)
        .sum();
    let _ = writeln!(out, "name=unlisted_steps total_over_the_product={unlisted}");
}

// ---- 第七节 ③2：B1 式子与量的逐格对账 ----
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Relation {
    /// 站点读到的步数恰好等于式子值。
    Equals,
    /// 站点读到的步数不大于式子值（线性找走到找到的那一个为止，至多走完整张表）。
    AtMost,
}

struct ReconciledRow {
    row: &'static str,
    relation: Relation,
    /// 对账的发布种类：规模读数取的是被量的发布做之前那一版，最后一批取的是整个 unlink 做完的一版，所以最后一批多数行不对账。
    kinds: &'static [&'static str],
    segments: &'static [&'static str],
}

/// 站点的步数能直接由规模读数写成式子的几行（读码判过：式子里的表在这几种发布里不随被量的发布变长）。
/// 别的行（步数吃这一版之外的表长、答案位置、每轮的差量）写不出只吃规模读数的式子，在 `three_two_rows_not_reconciled` 里点名，不拿 0 补。
const RECONCILED_ROWS: [ReconciledRow; 5] = [
    ReconciledRow {
        row: "files_other_than_the_first_plan",
        relation: Relation::Equals,
        kinds: &["overwrite", "unlink_intent"],
        segments: &["caller", "before_landing", "between"],
    },
    ReconciledRow {
        row: "inodes_with_an_extent_entry_set",
        relation: Relation::Equals,
        kinds: &["overwrite"],
        segments: &["caller", "before_landing", "between"],
    },
    ReconciledRow {
        row: "file_other_than_the_first_linear_find",
        relation: Relation::AtMost,
        kinds: &["overwrite", "unlink_intent"],
        segments: &["caller", "before_landing", "between"],
    },
    ReconciledRow {
        row: "cluster_segment_sum",
        relation: Relation::AtMost,
        kinds: &["overwrite", "unlink_intent"],
        segments: &["before_landing"],
    },
    ReconciledRow {
        row: "empty_segment_linear_scan",
        relation: Relation::AtMost,
        kinds: &["overwrite", "unlink_intent", "unlink_final"],
        segments: &["caller", "before_landing", "between"],
    },
];

/// 一行在一格里的式子值：调用次数乘一次调用走的步数（规模读数给的表长）。读不到规模时交回 `None`。
fn expected_steps_by_the_formula(
    row: &str,
    calls: u64,
    state: &Fields,
    axes: &PointAxes,
) -> Option<u64> {
    match row {
        "files_other_than_the_first_plan" | "file_other_than_the_first_linear_find" => {
            Some(calls * number(state, "other_files")?)
        }
        "inodes_with_an_extent_entry_set" => Some(calls * (1 + number(state, "other_files")?)),
        "cluster_segment_sum" => Some(calls * number(state, "cluster_segments")?),
        "empty_segment_linear_scan" => Some(calls * segments_of(axes.unit_area_slots)),
        _ => None,
    }
}

/// 对账：每个量过的点、状态、种类、轮上，`RECONCILED_ROWS` 每行的站点读数与式子值比。交回 (对了几格, 不符几格, 读不到规模几格)。
fn reconciliation_report(product: &Product, out: &mut String) -> (u64, u64, u64) {
    let (mut cells, mut disagreements, mut unread) = (0_u64, 0_u64, 0_u64);
    for entry in &RECONCILED_ROWS {
        let (mut row_cells, mut row_disagreements) = (0_u64, 0_u64);
        for (at, state) in &product.states {
            if !entry.kinds.contains(&at.2.as_str()) {
                continue;
            }
            let Some(axes) = product.points.get(&at.0) else {
                continue;
            };
            let calls = product.calls_of(at, entry.segments, entry.row);
            let steps = product.steps_of(at, entry.segments, entry.row);
            let Some(expected) = expected_steps_by_the_formula(entry.row, calls, state, axes)
            else {
                unread += 1;
                continue;
            };
            row_cells += 1;
            let agrees = match entry.relation {
                Relation::Equals => steps == expected,
                Relation::AtMost => steps <= expected,
            };
            if !agrees {
                row_disagreements += 1;
                let _ = writeln!(
                    out,
                    "name=three_two_disagreement point={} state={} kind={} round={} row={} relation={:?} steps={steps} formula={expected} calls={calls}",
                    at.0, at.1, at.2, at.3, entry.row, entry.relation
                );
            }
        }
        cells += row_cells;
        disagreements += row_disagreements;
        let _ = writeln!(
            out,
            "name=three_two_row row={} relation={:?} cells_checked={row_cells} disagreements={row_disagreements}",
            entry.row, entry.relation
        );
    }
    let not_reconciled: Vec<&str> = scan_rows()
        .into_iter()
        .filter(|row| {
            !RECONCILED_ROWS.iter().any(|entry| entry.row == *row)
                && *row != "whole_allocator_copy_before_the_judgement"
                && *row != "whole_allocator_copy_in_a_settling_round"
        })
        .collect();
    let _ = writeln!(
        out,
        "name=three_two_rows_not_reconciled count={} rows={}",
        not_reconciled.len(),
        not_reconciled.join(",")
    );
    let _ = writeln!(
        out,
        "name=verdict quantity=three_two cells_checked={cells} disagreements={disagreements} cells_with_the_scale_unread={unread} rows_reconciled={} rows_not_reconciled={}",
        RECONCILED_ROWS.len(),
        not_reconciled.len()
    );
    (cells, disagreements, unread)
}

// ---- 计时：Q3、Q10 与包含自检 ----
const BASE_POINT_NAME: &str = "base";

/// 一轮里某一段所有行（含清单外那一行）的独占计时之和。
fn exclusive_nanos_in_the_segment(timing: &Product, at: &RoundKey, segment: &str) -> u64 {
    let listed: u64 = all_rows()
        .iter()
        .map(|(row, _)| timing.nanos_of(at, &[segment], row))
        .sum();
    listed + timing.nanos_of(at, &[segment], UNLISTED_ROW)
}

/// 包含自检（作废条款第 5 条）：这一轮三段里每段 Σ 独占计时都不大于段挂钟；一段没有挂钟时它的独占计时也得是 0。
fn round_is_usable(timing: &Product, at: &RoundKey) -> bool {
    if !timing.rows_exist(at) {
        return false;
    }
    SEGMENTS_BEFORE_ADMISSION.iter().all(|segment| {
        let exclusive = exclusive_nanos_in_the_segment(timing, at, segment);
        match timing.wall_of(at, segment) {
            Some(wall) => round_is_valid(exclusive, wall),
            None => exclusive == 0,
        }
    })
}

fn usable_rounds(timing: &Product, point: &str, state: &str, kind: &str) -> Vec<u64> {
    (1..=ROUNDS)
        .filter(|round| round_is_usable(timing, &round_key(point, state, kind, *round)))
        .collect()
}

/// 一轮里扫类各行在准入之前三段的独占计时之和。
fn scan_nanos(timing: &Product, at: &RoundKey) -> u64 {
    scan_rows()
        .iter()
        .map(|row| timing.nanos_of(at, &SEGMENTS_BEFORE_ADMISSION, row))
        .sum()
}

fn join_numbers<Item: std::fmt::Display>(values: &[Item]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

/// 一根轴上五轮每轮一个 α：两端都有效的轮才算；不足五轮记 `fewer_than_five_valid_rounds`，五轮不同档记 `unstable`。
fn rounds_band(
    timing: &Product,
    axis: &str,
    state: &str,
    kind: &str,
    quantity: &dyn Fn(&RoundKey) -> u64,
) -> Option<(Vec<f64>, String)> {
    let points = axis_points_for_alpha(timing, axis);
    let (low, high) = (points.first()?, points.last()?);
    if low.0 == high.0 {
        return None;
    }
    let mut alphas = Vec::new();
    let mut valid = 0;
    for round in 1..=ROUNDS {
        let (low_at, high_at) = (
            round_key(low.0, state, kind, round),
            round_key(high.0, state, kind, round),
        );
        if !(round_is_usable(timing, &low_at) && round_is_usable(timing, &high_at)) {
            continue;
        }
        valid += 1;
        let (low_value, high_value) = (quantity(&low_at), quantity(&high_at));
        if low_value > 0 && high_value > 0 {
            alphas.push(alpha(
                to_f64(low_value),
                to_f64(high_value),
                to_f64(axis_value(axis, low.1)),
                to_f64(axis_value(axis, high.1)),
            ));
        }
    }
    let verdict = if valid < ROUNDS || alphas.len() < valid as usize {
        if valid < ROUNDS {
            "fewer_than_five_valid_rounds".to_string()
        } else {
            "one_end_is_zero".to_string()
        }
    } else {
        band_of_the_rounds(&alphas)
            .map_or("unstable".to_string(), |one_band| format!("{one_band:?}"))
    };
    Some((alphas, verdict))
}

/// Q3 与 Q10。交回 F3 是否触发：某一段的残差在某根轴上五轮都 α ≥ 0.5。
fn timing_report(timing: &Product, out: &mut String) -> bool {
    let mut residual_grows = false;
    for state in STATES {
        for kind in KINDS {
            for axis in AXES {
                let scan = |at: &RoundKey| scan_nanos(timing, at);
                if let Some((alphas, verdict)) = rounds_band(timing, axis, state, kind, &scan) {
                    let _ = writeln!(
                        out,
                        "name=q3 axis={axis} state={state} kind={kind} alphas={} band={verdict}",
                        join_numbers(
                            &alphas
                                .iter()
                                .map(|growth| format!("{growth:.6}"))
                                .collect::<Vec<_>>()
                        )
                    );
                }
                for segment in SEGMENTS_BEFORE_ADMISSION {
                    let residual = |at: &RoundKey| {
                        let wall = timing.wall_of(at, segment).unwrap_or(0);
                        wall.saturating_sub(exclusive_nanos_in_the_segment(timing, at, segment))
                    };
                    if let Some((alphas, verdict)) =
                        rounds_band(timing, axis, state, kind, &residual)
                    {
                        if verdict == "GrowsWithTheAxis" {
                            residual_grows = true;
                        }
                        let _ = writeln!(out, "name=q10 axis={axis} state={state} kind={kind} segment={segment} alphas={} band={verdict}", join_numbers(&alphas.iter().map(|growth| format!("{growth:.6}")).collect::<Vec<_>>()));
                    }
                }
            }
        }
    }
    // 残差占段挂钟的比（高端那一点第 1 轮，每段）
    for state in STATES {
        for point in timing.points.keys() {
            let at = round_key(point, state, "overwrite", 1);
            for segment in SEGMENTS_BEFORE_ADMISSION {
                if let Some(wall) = timing.wall_of(&at, segment) {
                    let residual =
                        wall.saturating_sub(exclusive_nanos_in_the_segment(timing, &at, segment));
                    let _ = writeln!(out, "name=q10_share point={point} state={state} segment={segment} residual_nanos={residual} segment_nanos={wall}");
                }
            }
        }
    }
    residual_grows
}

// ---- Q4、Q5：用户关心的规模上的外推 ----
/// 用户关心的规模 G-user：每盘 16 TiB 的单元区，10⁶ 次 unlink，F = 1000，D 两档。
struct UserScale {
    unit_area_slots: f64,
    unlinks: f64,
    data_units: f64,
    files: f64,
}

fn user_scale(data_units: f64) -> UserScale {
    UserScale {
        unit_area_slots: to_f64(user_scale_unit_area_slots()),
        unlinks: 1.0e6,
        data_units,
        files: 1000.0,
    }
}

fn axis_target(axis: &str, scale: &UserScale) -> f64 {
    match axis {
        "U" => scale.unit_area_slots,
        "u" => scale.unlinks,
        "D" => scale.data_units,
        _ => scale.files,
    }
}

/// 沿一根轴的 (x, y) 点列取 x 处的值：有这个 x 的点就取它；落在两点之间线性内插；大于最后一点时用最后两点的斜率线性外推。
fn value_on_axis(points: &[(f64, f64)], at_position: f64) -> Option<f64> {
    if let Some((_, value_at)) = points
        .iter()
        .find(|(px, _)| (*px - at_position).abs() < 0.5)
    {
        return Some(*value_at);
    }
    let [.., (earlier_position, earlier_value), (last_position, last_value)] = points else {
        return None;
    };
    if at_position > *last_position {
        return Some(linear_extrapolation(
            *earlier_position,
            *earlier_value,
            *last_position,
            *last_value,
            at_position,
        ));
    }
    let upper = points.iter().position(|(px, _)| *px > at_position)?;
    let (low, high) = (points.get(upper.checked_sub(1)?)?, points.get(upper)?);
    Some(low.1 + (high.1 - low.1) * (at_position - low.0) / (high.0 - low.0))
}

/// 一个量在 G-user 上的值：B0 上的值，加上每根轴上（一次只动一根轴）从 B0 到 G-user 那一轴取值的增量。
/// 交回值与没有两个点可外推的轴（那根轴的增量记 0，调用方标「未外推」）。
fn user_scale_value(
    product: &Product,
    state: &str,
    kind: &str,
    round: u64,
    quantity: &dyn Fn(&RoundKey) -> f64,
    scale: &UserScale,
) -> (f64, Vec<&'static str>) {
    let base = quantity(&round_key(BASE_POINT_NAME, state, kind, round));
    let mut total = base;
    let mut missing = Vec::new();
    for axis in AXES {
        let points: Vec<(f64, f64)> = axis_points_for_alpha(product, axis)
            .iter()
            .map(|(point, axes)| {
                (
                    to_f64(axis_value(axis, axes)),
                    quantity(&round_key(point, state, kind, round)),
                )
            })
            .collect();
        match value_on_axis(&points, axis_target(axis, scale)) {
            Some(value_at) => total += value_at - base,
            None => missing.push(axis),
        }
    }
    (total, missing)
}

/// τ = 一行的独占计时 ÷ 步数，取那一行步数最大的那个量过的点（登记第六节 Q4）。
fn tau_of_the_row(
    counts: &Product,
    timing: &Product,
    state: &str,
    kind: &str,
    round: u64,
    row: &str,
) -> f64 {
    let points: Vec<(u64, u64)> = counts
        .points
        .keys()
        .map(|point| {
            (
                counts.steps_of(
                    &round_key(point, state, kind, 1),
                    &SEGMENTS_BEFORE_ADMISSION,
                    row,
                ),
                timing.nanos_of(
                    &round_key(point, state, kind, round),
                    &SEGMENTS_BEFORE_ADMISSION,
                    row,
                ),
            )
        })
        .collect();
    tau_nanos_per_step(&points)
}

/// 一轮里一次覆盖写准入之前扫类挂钟在 G-user 上的推值 T，与没能外推的轴。
fn user_scale_nanos(
    counts: &Product,
    timing: &Product,
    state: &str,
    round: u64,
    scale: &UserScale,
) -> (f64, Vec<&'static str>) {
    let mut total = 0.0;
    let mut missing = Vec::new();
    for row in scan_rows() {
        let steps = |at: &RoundKey| {
            to_f64(counts.steps_of(
                &round_key(&at.0, &at.1, &at.2, 1),
                &SEGMENTS_BEFORE_ADMISSION,
                row,
            ))
        };
        let (mut steps_at_user_scale, row_missing) =
            user_scale_value(counts, state, "overwrite", 1, &steps, scale);
        if row == "cluster_segment_sum" && state == "same_mount" {
            // 开过的聚簇段数封顶每盘段数 S（登记第六节 Q4）：这一行每次调用看的段数不会超过 S。
            let calls = to_f64(counts.calls_of(
                &round_key(BASE_POINT_NAME, state, "overwrite", 1),
                &SEGMENTS_BEFORE_ADMISSION,
                row,
            ));
            steps_at_user_scale =
                steps_at_user_scale.min(calls * to_f64(segments_of(user_scale_unit_area_slots())));
        }
        for axis in row_missing {
            if !missing.contains(&axis) {
                missing.push(axis);
            }
        }
        total +=
            steps_at_user_scale * tau_of_the_row(counts, timing, state, "overwrite", round, row);
    }
    (total, missing)
}

/// Q4 四格（两档 D_user × 两种挂载状态）；交回四格的档（只含判得出的），与有没有哪一格的 D、F 轴没外推。
fn one_millisecond_report(
    counts: &Product,
    timing: &Product,
    out: &mut String,
) -> (Vec<FormulaCell>, bool) {
    let mut cells = Vec::new();
    let mut reference_only = false;
    for state in STATES {
        for data_units in [10.0, 268_435_456.0] {
            let scale = user_scale(data_units);
            let mut nanos = Vec::new();
            let mut missing_axes = Vec::new();
            let usable = usable_rounds(timing, BASE_POINT_NAME, state, "overwrite");
            for round in &usable {
                let (value, missing) = user_scale_nanos(counts, timing, state, *round, &scale);
                nanos.push(value);
                missing_axes = missing;
            }
            let cell = (usable.len() as u64 == ROUNDS).then(|| one_millisecond_cell(&nanos));
            let missing_data_or_files =
                missing_axes.iter().any(|axis| *axis == "D" || *axis == "F");
            reference_only |= missing_data_or_files;
            let _ = writeln!(
                out,
                "name=q4 state={state} data_units_user={data_units:.0} valid_rounds={} cell={} nanos_of_each_round={} axes_not_extrapolated={}",
                usable.len(),
                cell.map_or("fewer_than_five_valid_rounds".to_string(), |one_cell| format!("{one_cell:?}")),
                join_numbers(&nanos.iter().map(|value| format!("{value:.0}")).collect::<Vec<_>>()),
                missing_axes.join(",")
            );
            if let Some(cell) = cell {
                cells.push(cell);
            }
        }
    }
    (cells, reference_only)
}

/// Q5 外推核对：从更小的点推最大的那个点，步数相对差 ≤ 10%、挂钟比在 [0.5, 2]。交回每格是否核过。
fn extrapolation_check_report(counts: &Product, timing: &Product, out: &mut String) -> Vec<bool> {
    let mut checks = Vec::new();
    for state in STATES {
        for axis in ["U", "u"] {
            let points = axis_points_for_alpha(counts, axis);
            let Some((target, smaller)) = points.split_last() else {
                continue;
            };
            if smaller.len() < 2 {
                continue;
            }
            let steps_series: Vec<(f64, f64)> = smaller
                .iter()
                .map(|(point, axes)| {
                    (
                        to_f64(axis_value(axis, axes)),
                        to_f64(scan_steps(
                            counts,
                            point,
                            state,
                            "overwrite",
                            &SEGMENTS_BEFORE_ADMISSION,
                        )),
                    )
                })
                .collect();
            let predicted_steps =
                value_on_axis(&steps_series, to_f64(axis_value(axis, target.1))).unwrap_or(0.0);
            let measured_steps = to_f64(scan_steps(
                counts,
                target.0,
                state,
                "overwrite",
                &SEGMENTS_BEFORE_ADMISSION,
            ));
            let mut ratios = Vec::new();
            for round in 1..=ROUNDS {
                let target_at = round_key(target.0, state, "overwrite", round);
                let nanos_series: Vec<(f64, f64)> = smaller
                    .iter()
                    .map(|(point, axes)| {
                        (
                            to_f64(axis_value(axis, axes)),
                            to_f64(scan_nanos(
                                timing,
                                &round_key(point, state, "overwrite", round),
                            )),
                        )
                    })
                    .collect();
                let predicted =
                    value_on_axis(&nanos_series, to_f64(axis_value(axis, target.1))).unwrap_or(0.0);
                let measured = to_f64(scan_nanos(timing, &target_at));
                if measured > 0.0 {
                    ratios.push(predicted / measured);
                }
            }
            let checked = ratios.len() as u64 == ROUNDS
                && ratios
                    .iter()
                    .all(|ratio| extrapolation_is_checked(predicted_steps, measured_steps, *ratio));
            let _ = writeln!(
                out,
                "name=q5 axis={axis} state={state} predicted_steps={predicted_steps:.0} measured_steps={measured_steps:.0} nanos_ratios={} checked={checked}",
                join_numbers(&ratios.iter().map(|ratio| format!("{ratio:.3}")).collect::<Vec<_>>())
            );
            checks.push(checked);
        }
    }
    checks
}

// ---- B8、B4 的输入：从计数产物里读（读不到的列进缺口，不当 0）----
/// B8 式子里 `c_x` 那几行（登记第五节表里写调用次数的行）。
const CALLED_ROWS: [&str; 17] = [
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
];

/// 固定点每一轮的某个差量读数（一轮取读到的最大那个）；某一轮一条都没有记缺口。
fn per_round_maxima(
    product: &Product,
    at: &RoundKey,
    key: &str,
    rounds: u64,
    gaps: &mut Vec<String>,
) -> Vec<u64> {
    let readings = product.deltas_of(at, key, &["between"]);
    (1..=rounds)
        .map(|round| {
            let best = readings
                .iter()
                .filter(|(fixed_point_round, _)| *fixed_point_round == round)
                .map(|(_, value)| *value)
                .max();
            best.unwrap_or_else(|| {
                gaps.push(format!("{key}@round{round}"));
                0
            })
        })
        .collect()
}

/// 一个点某状态某发布（第 1 轮）在点名几段里的 B8 输入。缺口列出读不到的读数名。
/// 与登记第七节 ③3 的 Δ0 同一套字段；每个字段怎么从产物里读，见本函数里逐行的注释。
fn by_delta_input_from_the_product(
    product: &Product,
    point: &str,
    state: &str,
    kind: &str,
    segments: &[&str],
) -> Result<ByDeltaInput, Vec<String>> {
    let at = round_key(point, state, kind, 1);
    let Some(axes) = product.points.get(point) else {
        return Err(vec![format!("point@{point}")]);
    };
    let mut gaps: Vec<String> = Vec::new();
    let mut state_field = |key: &str| -> u64 {
        product.state_value(&at, key).unwrap_or_else(|| {
            gaps.push(format!("state.{key}"));
            0
        })
    };
    let inode_leaf_containers = state_field("inode_leaf_containers");
    let other_files = state_field("other_files");
    let allocation_records = state_field("allocation_records");
    let units = state_field("units");
    let mapped_units = state_field("mapped_units");
    let deadlist_entries = state_field("deadlist_entries");
    let extent_upper_nodes = state_field("extent_upper_nodes");
    let extent_upper_levels = state_field("extent_upper_levels");
    let accounting_tree_levels = state_field("accounting_tree_levels");
    let deadlist_tree_levels = state_field("deadlist_tree_levels");
    let allocation_tree_levels = state_field("allocation_tree_levels");
    let central_mapping_levels = state_field("central_mapping_levels");
    let between = segments.contains(&"between");
    let rounds = if between {
        product
            .publishes
            .get(&at)
            .and_then(|fields| number(fields, "fixed_point_rounds"))
            .unwrap_or_else(|| {
                gaps.push("publish.fixed_point_rounds".to_string());
                0
            })
    } else {
        0
    };
    let calls: BTreeMap<&'static str, u64> = CALLED_ROWS
        .iter()
        .map(|row| (*row, product.calls_of(&at, segments, row)))
        .collect();
    let called = |row: &str| calls.get(row).copied().unwrap_or(0) > 0;
    // 这一次发布本身的量（不分段）：取三段里读到的最大那个；该有而没有的记缺口。
    let publish_level = |key: &str, needed: bool, gaps: &mut Vec<String>| -> u64 {
        let best = product
            .deltas_of(&at, key, &SEGMENTS_BEFORE_ADMISSION)
            .into_iter()
            .map(|(_, value)| value)
            .max();
        best.unwrap_or_else(|| {
            if needed {
                gaps.push(format!("delta.{key}"));
            }
            0
        })
    };
    let touched_files = publish_level(
        "touched_files",
        called("files_other_than_the_first_plan"),
        &mut gaps,
    );
    let lower_nodes_of_touched_files = publish_level(
        "lower_nodes_of_touched_files",
        called("files_other_than_the_first_plan"),
        &mut gaps,
    );
    let accounting_rows = publish_level(
        "accounting_rows_written_by_the_built_on_publish",
        called("checkpoint_cost_accounting_tree_plan")
            || called("accounting_entries_of_this_publish"),
        &mut gaps,
    );
    let deadlist_changed_entries = publish_level("deadlist_entries_changed", between, &mut gaps);
    let moved_entries = publish_level("moved_entries", between, &mut gaps);
    let intersecting_records: u64 = product
        .deltas_of(&at, "intersecting_records", segments)
        .iter()
        .map(|(_, value)| value)
        .sum();
    let changed_leaves_per_round =
        per_round_maxima(product, &at, "changed_leaves", rounds, &mut gaps);
    let released_per_round =
        per_round_maxima(product, &at, "round_placements_released", rounds, &mut gaps);
    let rewritten_per_round =
        per_round_maxima(product, &at, "round_roles_rewritten", rounds, &mut gaps);
    let changed_slots_and_records_per_round: Vec<u64> = released_per_round
        .iter()
        .zip(&rewritten_per_round)
        .map(|(released, rewritten)| released + rewritten)
        .collect();
    let dirty_per_call: Vec<u64> = product
        .deltas_of(&at, "dirty_records", segments)
        .into_iter()
        .map(|(_, value)| value)
        .collect();
    let keys_changed = product.deltas_of(&at, "code_two_keys_changed", segments);
    if keys_changed.len() as u64 != 3 * rounds {
        gaps.push(format!(
            "delta.code_two_keys_changed:{}_for_{}_rounds",
            keys_changed.len(),
            rounds
        ));
    }
    let heights = [
        accounting_tree_levels,
        deadlist_tree_levels,
        central_mapping_levels,
    ];
    let code_two_trees_over_all_rounds: Vec<(u64, u64)> = keys_changed
        .iter()
        .enumerate()
        .map(|(index, (_, value))| (*value, heights[index % 3]))
        .collect();
    let container_rewrites_over_all_rounds: Vec<u64> = product
        .deltas_of(&at, "container_units_rewritten", segments)
        .into_iter()
        .map(|(_, value)| value)
        .collect();
    let release_query_levels =
        product.query_levels_of(&at, segments, "release_judgement_version_indexes");
    let deadlist_query_levels = product.query_levels_of(&at, segments, "deadlist_changes_indexes");
    let mapped_units_query_levels =
        product.query_levels_of(&at, segments, "mapped_units_whole_table");
    if !gaps.is_empty() {
        return Err(gaps);
    }
    Ok(ByDeltaInput {
        inode_leaf_containers,
        other_files,
        touched_files,
        lower_nodes_of_touched_files,
        extent_upper_levels,
        extent_upper_nodes,
        accounting_rows_of_the_generation: accounting_rows,
        accounting_tree_levels,
        settling_rounds: rounds,
        changed_leaves_per_round,
        allocation_tree_levels,
        allocation_records,
        segments_per_disk: segments_of(axes.unit_area_slots),
        units,
        mapped_units,
        deadlist_entries,
        calls,
        intersecting_records,
        dirty_per_call,
        code_two_trees_over_all_rounds,
        container_rewrites_over_all_rounds,
        moved_entries,
        central_mapping_levels,
        deadlist_query_levels,
        deadlist_changed_entries,
        release_query_levels,
        changed_slots_and_records_per_round,
        mapped_units_changed_over_all_rounds: 2 * rewritten_per_round.iter().sum::<u64>(),
        mapped_units_query_levels,
    })
}

// ---- Q6、Q7、Q8、Q9：B8、B4 的式子在量过的点上 ----
/// 一个点某状态某发布：三段各自的 B8 每行步数。读不到输入的交回缺口。
fn by_delta_rows_by_segment(
    counts: &Product,
    point: &str,
    state: &str,
    kind: &str,
) -> Result<StepsByRow, Vec<String>> {
    let mut by_row = StepsByRow::new();
    for segment in SEGMENTS_BEFORE_ADMISSION {
        let input = by_delta_input_from_the_product(counts, point, state, kind, &[segment])?;
        for (row, steps) in by_delta_rows(&input) {
            by_row.insert((segment, row), steps);
        }
    }
    Ok(by_row)
}

fn today_rows_by_segment(counts: &Product, point: &str, state: &str, kind: &str) -> StepsByRow {
    let at = round_key(point, state, kind, 1);
    let mut by_row = StepsByRow::new();
    for segment in SEGMENTS_BEFORE_ADMISSION {
        for row in scan_rows() {
            by_row.insert((segment, row), counts.steps_of(&at, &[segment], row));
        }
    }
    by_row
}

fn total_of(by_row: &StepsByRow, segments: &[&str]) -> u64 {
    by_row
        .iter()
        .filter(|((segment, _), _)| segments.contains(segment))
        .map(|(_, steps)| steps)
        .sum()
}

/// 在 P0 或 S1 里调到过的行（B4：改的是函数，两道之间再调到也按 B8）。
fn rows_called_before_the_second_judgement(
    counts: &Product,
    point: &str,
    state: &str,
    kind: &str,
) -> BTreeSet<&'static str> {
    let at = round_key(point, state, kind, 1);
    scan_rows()
        .into_iter()
        .filter(|row| counts.calls_of(&at, &SEGMENTS_BEFORE_THE_SECOND_JUDGEMENT, row) > 0)
        .collect()
}

fn alpha_text_for_points(
    counts: &Product,
    axis: &str,
    quantity: &dyn Fn(&str) -> Option<u64>,
) -> AxisReading {
    let points = axis_points_for_alpha(counts, axis);
    let (Some(low), Some(high)) = (points.first(), points.last()) else {
        return AxisReading::NoEnds;
    };
    if low.0 == high.0 {
        return AxisReading::NoEnds;
    }
    let (Some(low_value), Some(high_value)) = (quantity(low.0), quantity(high.0)) else {
        return AxisReading::NoEnds;
    };
    if low_value == 0 || high_value == 0 {
        return AxisReading::OneEndIsZero(low_value, high_value);
    }
    AxisReading::Alpha(
        low_value,
        high_value,
        alpha(
            to_f64(low_value),
            to_f64(high_value),
            to_f64(axis_value(axis, low.1)),
            to_f64(axis_value(axis, high.1)),
        ),
    )
}

/// 三档里最坏的一档：任一格「随」就是「随」，全部「不随」才是「不随」，否则「介于」；有格读不到记「读不到」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Summary {
    GrowsOnSomeAxis,
    DoesNotGrowOnAnyAxis,
    BetweenOnSomeAxis,
    SomeInputUnread,
}

fn summarize(bands: &[Option<Band>], any_unread: bool) -> Summary {
    if bands
        .iter()
        .any(|one_band| *one_band == Some(Band::GrowsWithTheAxis))
    {
        Summary::GrowsOnSomeAxis
    } else if any_unread || bands.is_empty() {
        Summary::SomeInputUnread
    } else if bands
        .iter()
        .all(|one_band| *one_band == Some(Band::DoesNotGrow))
    {
        Summary::DoesNotGrowOnAnyAxis
    } else {
        Summary::BetweenOnSomeAxis
    }
}

/// Q6、Q8（B4 臂）与 Q7：交回 (Q6 总结, Q8 的 B4 臂总结)。
fn arms_report(counts: &Product, out: &mut String) -> (Summary, Summary) {
    let mut by_delta_bands = Vec::new();
    let mut narrowed_bands = Vec::new();
    let mut by_delta_unread = false;
    let mut narrowed_unread = false;
    for state in STATES {
        for kind in KINDS {
            let mut inputs: BTreeMap<String, Result<StepsByRow, Vec<String>>> = BTreeMap::new();
            for point in counts.points.keys() {
                inputs.insert(
                    point.clone(),
                    by_delta_rows_by_segment(counts, point, state, kind),
                );
            }
            for (point, input) in &inputs {
                if let Err(gaps) = input {
                    by_delta_unread = true;
                    narrowed_unread = true;
                    let _ = writeln!(
                        out,
                        "name=b8_input_gap point={point} state={state} kind={kind} unread={}",
                        gaps.join(",")
                    );
                }
            }
            for axis in AXES {
                let by_delta_total = |point: &str| match inputs.get(point) {
                    Some(Ok(rows)) => Some(total_of(rows, &SEGMENTS_BEFORE_ADMISSION)),
                    _ => None,
                };
                let reading = alpha_text_for_points(counts, axis, &by_delta_total);
                if reading != AxisReading::NoEnds {
                    let _ = writeln!(
                        out,
                        "name=q6 axis={axis} state={state} kind={kind} {}",
                        reading_text(&reading)
                    );
                    by_delta_bands.push(reading_band(&reading));
                }
                let narrowed_before_the_second_judgement = |point: &str| match inputs.get(point) {
                    Some(Ok(rows)) => Some(total_of(rows, &SEGMENTS_BEFORE_THE_SECOND_JUDGEMENT)),
                    _ => None,
                };
                let reading =
                    alpha_text_for_points(counts, axis, &narrowed_before_the_second_judgement);
                if reading != AxisReading::NoEnds {
                    let _ = writeln!(
                        out,
                        "name=q8 arm=b4 axis={axis} state={state} kind={kind} {}",
                        reading_text(&reading)
                    );
                    narrowed_bands.push(reading_band(&reading));
                }
                let today_before_the_second_judgement = |point: &str| {
                    Some(total_of(
                        &today_rows_by_segment(counts, point, state, kind),
                        &SEGMENTS_BEFORE_THE_SECOND_JUDGEMENT,
                    ))
                };
                let reading =
                    alpha_text_for_points(counts, axis, &today_before_the_second_judgement);
                if reading != AxisReading::NoEnds {
                    let _ = writeln!(
                        out,
                        "name=q8 arm=b1 axis={axis} state={state} kind={kind} {}",
                        reading_text(&reading)
                    );
                }
            }
            // 贡献最大的三行（高端点）：B8 在三段合计里步数最大的三行，与它们吃的差量输入名。
            if let Some((point, Ok(rows))) = counts
                .points
                .keys()
                .filter_map(|point_name| inputs.get(point_name).map(|result| (point_name, result)))
                .max_by_key(|(point_name, _)| {
                    counts.points[*point_name].unit_area_slots
                        + counts.points[*point_name].unlinks
                        + counts.points[*point_name].data_units
                        + counts.points[*point_name].files
                })
            {
                let mut by_row: BTreeMap<&str, u64> = BTreeMap::new();
                for ((_, row), steps) in rows {
                    *by_row.entry(*row).or_default() += steps;
                }
                let mut ranked: Vec<(&str, u64)> = by_row.into_iter().collect();
                ranked.sort_by_key(|(_, steps)| std::cmp::Reverse(*steps));
                let top: Vec<String> = ranked
                    .iter()
                    .take(3)
                    .map(|(row, steps)| format!("{row}:{steps}"))
                    .collect();
                let _ = writeln!(
                    out,
                    "name=q6_top_rows point={point} state={state} kind={kind} rows={}",
                    top.join(",")
                );
            }
        }
    }
    (
        summarize(&by_delta_bands, by_delta_unread),
        summarize(&narrowed_bands, narrowed_unread),
    )
}

/// Q7：B8 常驻内存与维护（每个点，第 1 轮，覆盖写那一种）。
fn memory_report(counts: &Product, out: &mut String) {
    for state in STATES {
        let mut maintenance_by_point: BTreeMap<String, u64> = BTreeMap::new();
        for (point, axes) in &counts.points {
            let at = round_key(point, state, "overwrite", 1);
            let unit_bytes = number(&counts.sizes, "unit_bytes");
            let key_bytes = number(&counts.sizes, "key_bytes");
            let row_bytes = number(&counts.sizes, "accounting_row_bytes");
            let (Some(unit_bytes), Some(key_bytes), Some(accounting_row_bytes)) =
                (unit_bytes, key_bytes, row_bytes)
            else {
                let _ = writeln!(out, "name=q7_gap point={point} state={state} unread=sizes");
                continue;
            };
            let Ok(input) = by_delta_input_from_the_product(
                counts,
                point,
                state,
                "overwrite",
                &SEGMENTS_BEFORE_ADMISSION,
            ) else {
                let _ = writeln!(
                    out,
                    "name=q7_gap point={point} state={state} unread=b8_input"
                );
                continue;
            };
            let memory_input = MemoryInput {
                allocation_records: input.allocation_records,
                segments_per_disk: segments_of(axes.unit_area_slots),
                units: input.units,
                mapped_units: input.mapped_units,
                files_in_the_index: input.other_files + 1,
                extent_upper_nodes: input.extent_upper_nodes,
                deadlist_entries: input.deadlist_entries,
                accounting_rows_of_the_generation: input.accounting_rows_of_the_generation,
                unit_bytes,
                key_bytes,
                accounting_row_bytes,
            };
            let memory = by_delta_memory(&memory_input);
            let last_round = input.settling_rounds;
            let deltas = |key: &str| {
                counts
                    .deltas_of(&at, key, &["between"])
                    .into_iter()
                    .filter(|(round, _)| *round == last_round)
                    .map(|(_, value)| value)
                    .max()
                    .unwrap_or(0)
            };
            let records_changed =
                deltas("round_placements_released") + deltas("round_roles_rewritten");
            let rewritten_roles = deltas("round_roles_rewritten");
            let upper_rewritten = counts
                .deltas_of(
                    &at,
                    "extent_upper_nodes_rewritten",
                    &SEGMENTS_BEFORE_ADMISSION,
                )
                .into_iter()
                .map(|(_, value)| value)
                .max()
                .unwrap_or(0);
            let maintenance = MaintenanceInput {
                records_changed,
                segments_flipped: records_changed,
                units_delta: rewritten_roles,
                mapped_delta: rewritten_roles,
                files_delta: input.touched_files,
                upper_delta: upper_rewritten,
                deadlist_delta: input.deadlist_changed_entries,
            };
            let steps = by_delta_maintenance(&memory_input, &maintenance);
            maintenance_by_point.insert(point.clone(), steps);
            let peak_round = input
                .changed_slots_and_records_per_round
                .iter()
                .copied()
                .max()
                .unwrap_or(0);
            let _ = writeln!(
                out,
                "name=q7 point={point} state={state} payload_bytes={} payload_bytes_times_two={} segment_summaries={} units_by_role={} mapped_units_by_role={} record_index={} maintenance_steps={steps} undo_log_bytes={} delta_view_peak_bytes={}",
                memory.payload(),
                2 * memory.payload(),
                memory.segment_summaries,
                memory.units_by_role,
                memory.mapped_units_by_role,
                memory.record_index,
                undo_log_bytes(records_changed),
                undo_log_bytes(peak_round)
            );
        }
        for axis in AXES {
            let reading = alpha_text_for_points(counts, axis, &|point: &str| {
                maintenance_by_point.get(point).copied()
            });
            if reading != AxisReading::NoEnds {
                let _ = writeln!(
                    out,
                    "name=q7_maintenance_alpha axis={axis} state={state} {}",
                    reading_text(&reading)
                );
            }
        }
    }
}

/// B4 臂两道之间那一段（S2）一轮的挂钟（纳秒）：S2 里没改的行取独占计时，改过的行取 B8 步数乘那一行 B1 的每步纳秒，再加 S2 残差。
/// 这一轮过不了包含自检时交回 `None`。
fn narrowed_between_nanos(
    timing: &Product,
    at: &RoundKey,
    by_delta: &StepsByRow,
    today: &StepsByRow,
    called: &BTreeSet<&'static str>,
) -> Option<f64> {
    if !round_is_usable(timing, at) {
        return None;
    }
    let wall = timing.wall_of(at, "between").unwrap_or(0);
    let exclusive = exclusive_nanos_in_the_segment(timing, at, "between");
    let mut nanos = to_f64(wall.saturating_sub(exclusive));
    for row in scan_rows() {
        let own = to_f64(timing.nanos_of(at, &["between"], row));
        if called.contains(row) {
            let today_steps = to_f64(today.get(&("between", row)).copied().unwrap_or(0));
            let per_step = if today_steps > 0.0 {
                own / today_steps
            } else {
                0.0
            };
            nanos += to_f64(by_delta.get(&("between", row)).copied().unwrap_or(0)) * per_step;
        } else {
            nanos += own;
        }
    }
    Some(nanos)
}

/// Q9：B4 臂两道之间那一段（S2）的步数与挂钟。挂钟 = S2 里没改的行的独占计时 + S2 残差 + 改过的行的 B8 步数 × 那一行 B1 的每步纳秒。
fn between_segment_report(counts: &Product, timing: Option<&Product>, out: &mut String) {
    for state in STATES {
        for kind in KINDS {
            let mut steps_by_point: BTreeMap<String, u64> = BTreeMap::new();
            for point in counts.points.keys() {
                let Ok(by_delta) = by_delta_rows_by_segment(counts, point, state, kind) else {
                    continue;
                };
                let today = today_rows_by_segment(counts, point, state, kind);
                let called = rows_called_before_the_second_judgement(counts, point, state, kind);
                let [_, _, between] = narrowed_steps_by_segment(&today, &by_delta, &called);
                steps_by_point.insert(point.clone(), between);
                let mut nanos_text = String::new();
                if let Some(timing) = timing {
                    let mut per_round = Vec::new();
                    for round in 1..=ROUNDS {
                        let at = round_key(point, state, kind, round);
                        if let Some(nanos) =
                            narrowed_between_nanos(timing, &at, &by_delta, &today, &called)
                        {
                            per_round.push(nanos);
                        }
                    }
                    nanos_text = format!(
                        " nanos_of_each_valid_round={}",
                        join_numbers(
                            &per_round
                                .iter()
                                .map(|value| format!("{value:.0}"))
                                .collect::<Vec<_>>()
                        )
                    );
                }
                let _ = writeln!(out, "name=q9 point={point} state={state} kind={kind} steps_between={between}{nanos_text}");
            }
            for axis in AXES {
                let reading = alpha_text_for_points(counts, axis, &|point: &str| {
                    steps_by_point.get(point).copied()
                });
                if reading != AxisReading::NoEnds {
                    let _ = writeln!(
                        out,
                        "name=q9_alpha axis={axis} state={state} kind={kind} {}",
                        reading_text(&reading)
                    );
                }
            }
        }
    }
}

// ---- Q6、Q8、Q9 在用户关心的规模上（推的，与 Q4 同一个外推法）----
/// 一个按点算出的量在 G-user 上的值：先算出每个点的值（任何一个点读不到，整格读不到，不拿 0 补），
/// 再按 Q4 的外推法取 B0 上的值加每根轴从 B0 到 G-user 那一轴取值的增量。交回值与没能外推的轴。
fn quantity_at_user_scale(
    counts: &Product,
    state: &str,
    kind: &str,
    scale: &UserScale,
    per_point: &dyn Fn(&str) -> Result<u64, Vec<String>>,
) -> Result<(f64, Vec<&'static str>), Vec<String>> {
    if !counts.points.contains_key(BASE_POINT_NAME) {
        return Err(vec!["point@base".to_string()]);
    }
    let mut values: BTreeMap<String, f64> = BTreeMap::new();
    let mut gaps: Vec<String> = Vec::new();
    for point in counts.points.keys() {
        match per_point(point) {
            Ok(value) => {
                values.insert(point.clone(), to_f64(value));
            }
            Err(point_gaps) => gaps.push(format!("{point}:{}", point_gaps.join("|"))),
        }
    }
    if !gaps.is_empty() {
        return Err(gaps);
    }
    let quantity = |at: &RoundKey| values.get(&at.0).copied().unwrap_or(0.0);
    Ok(user_scale_value(counts, state, kind, 1, &quantity, scale))
}

fn user_scale_text(label: &str, result: &Result<(f64, Vec<&'static str>), Vec<String>>) -> String {
    match result {
        Ok((steps, missing)) => format!(
            "{label}_steps={steps:.0} {label}_axes_not_extrapolated={}",
            missing.join(",")
        ),
        Err(gaps) => format!("{label}_unread={}", gaps.join(";")),
    }
}

/// Q6（B8 准入之前扫类步数）、Q8（B4 臂落点判之前剩下的步数）、Q9（B4 臂两道之间的步数与挂钟）在 G-user 上的值，
/// 两档 D_user × 两种挂载状态 × 三种发布；每个量旁边是 B1 今天的同一个量。有计时产物时另算 Q9 的挂钟（覆盖写，五轮各一个）。
fn user_scale_steps_report(counts: &Product, timing: Option<&Product>, out: &mut String) {
    for state in STATES {
        for kind in KINDS {
            for data_units in [10.0, 268_435_456.0] {
                let scale = user_scale(data_units);
                let today_before_admission =
                    quantity_at_user_scale(counts, state, kind, &scale, &|point: &str| -> Result<
                        u64,
                        Vec<String>,
                    > {
                        Ok(scan_steps(
                            counts,
                            point,
                            state,
                            kind,
                            &SEGMENTS_BEFORE_ADMISSION,
                        ))
                    });
                let by_delta_before_admission =
                    quantity_at_user_scale(counts, state, kind, &scale, &|point: &str| -> Result<
                        u64,
                        Vec<String>,
                    > {
                        let rows = by_delta_rows_by_segment(counts, point, state, kind)?;
                        Ok(total_of(&rows, &SEGMENTS_BEFORE_ADMISSION))
                    });
                let today_before_landing_judgement =
                    quantity_at_user_scale(counts, state, kind, &scale, &|point: &str| -> Result<
                        u64,
                        Vec<String>,
                    > {
                        Ok(scan_steps(
                            counts,
                            point,
                            state,
                            kind,
                            &SEGMENTS_BEFORE_THE_SECOND_JUDGEMENT,
                        ))
                    });
                let narrowed_before_landing_judgement =
                    quantity_at_user_scale(counts, state, kind, &scale, &|point: &str| -> Result<
                        u64,
                        Vec<String>,
                    > {
                        let rows = by_delta_rows_by_segment(counts, point, state, kind)?;
                        Ok(total_of(&rows, &SEGMENTS_BEFORE_THE_SECOND_JUDGEMENT))
                    });
                let today_between =
                    quantity_at_user_scale(counts, state, kind, &scale, &|point: &str| -> Result<
                        u64,
                        Vec<String>,
                    > {
                        Ok(scan_steps(counts, point, state, kind, &["between"]))
                    });
                let narrowed_between =
                    quantity_at_user_scale(counts, state, kind, &scale, &|point: &str| -> Result<
                        u64,
                        Vec<String>,
                    > {
                        let by_delta = by_delta_rows_by_segment(counts, point, state, kind)?;
                        let today = today_rows_by_segment(counts, point, state, kind);
                        let called =
                            rows_called_before_the_second_judgement(counts, point, state, kind);
                        let [_, _, between] = narrowed_steps_by_segment(&today, &by_delta, &called);
                        Ok(between)
                    });
                let _ = writeln!(
                    out,
                    "name=q6_user_scale state={state} kind={kind} data_units_user={data_units:.0} {} {}",
                    user_scale_text("by_delta", &by_delta_before_admission),
                    user_scale_text("today", &today_before_admission)
                );
                let _ = writeln!(
                    out,
                    "name=q8_user_scale state={state} kind={kind} data_units_user={data_units:.0} {} {}",
                    user_scale_text("narrowed", &narrowed_before_landing_judgement),
                    user_scale_text("today", &today_before_landing_judgement)
                );
                let _ = writeln!(
                    out,
                    "name=q9_user_scale state={state} kind={kind} data_units_user={data_units:.0} {} {}",
                    user_scale_text("narrowed", &narrowed_between),
                    user_scale_text("today", &today_between)
                );
                if let (true, Some(timing)) = (kind == "overwrite", timing) {
                    let text = narrowed_between_nanos_at_user_scale(counts, timing, state, &scale);
                    let _ = writeln!(
                        out,
                        "name=q9_user_scale_nanos state={state} kind={kind} data_units_user={data_units:.0} {text}"
                    );
                }
            }
        }
    }
}

/// Q9 的挂钟在 G-user 上：每个有效轮一个值（五个有效轮才算），点上读不到、轮不能用的整格记读不到。
fn narrowed_between_nanos_at_user_scale(
    counts: &Product,
    timing: &Product,
    state: &str,
    scale: &UserScale,
) -> String {
    let usable = usable_rounds(timing, BASE_POINT_NAME, state, "overwrite");
    let mut nanos_of_each_round: Vec<f64> = Vec::new();
    let mut missing_axes: Vec<&'static str> = Vec::new();
    let mut unread: Vec<String> = Vec::new();
    for round in &usable {
        let mut values: BTreeMap<String, f64> = BTreeMap::new();
        for point in counts.points.keys() {
            let Ok(by_delta) = by_delta_rows_by_segment(counts, point, state, "overwrite") else {
                unread.push(format!("{point}:b8_input"));
                continue;
            };
            let today = today_rows_by_segment(counts, point, state, "overwrite");
            let called = rows_called_before_the_second_judgement(counts, point, state, "overwrite");
            let at = round_key(point, state, "overwrite", *round);
            match narrowed_between_nanos(timing, &at, &by_delta, &today, &called) {
                Some(nanos) => {
                    values.insert(point.clone(), nanos);
                }
                None => unread.push(format!("{point}@round{round}")),
            }
        }
        let quantity = |at: &RoundKey| values.get(&at.0).copied().unwrap_or(0.0);
        let (value, missing) =
            user_scale_value(counts, state, "overwrite", *round, &quantity, scale);
        nanos_of_each_round.push(value);
        missing_axes = missing;
    }
    unread.sort();
    unread.dedup();
    if !unread.is_empty() {
        return format!("unread={}", unread.join(";"));
    }
    if usable.len() as u64 != ROUNDS {
        return "fewer_than_five_valid_rounds".to_string();
    }
    format!(
        "nanos_of_each_round={} axes_not_extrapolated={}",
        join_numbers(
            &nanos_of_each_round
                .iter()
                .map(|value| format!("{value:.0}"))
                .collect::<Vec<_>>()
        ),
        missing_axes.join(",")
    )
}

/// Q11、Q12、Q13：附带，只报数。
fn side_quantities_report(counts: &Product, out: &mut String) {
    for (point, _) in &counts.points {
        for state in STATES {
            let at = round_key(point, state, "overwrite", 1);
            let after: u64 = scan_rows()
                .iter()
                .map(|row| counts.steps_of(&at, &["after"], row))
                .sum();
            let copies: Vec<String> = all_rows()
                .iter()
                .filter(|(_, kind)| *kind == RowKind::Copy)
                .map(|(row, _)| {
                    format!(
                        "{row}:{}",
                        counts.steps_of(&at, &SEGMENTS_BEFORE_ADMISSION, row)
                    )
                })
                .collect();
            let _ = writeln!(
                out,
                "name=q11 point={point} state={state} scan_steps_after_the_judgements={after}"
            );
            let _ = writeln!(
                out,
                "name=q13 point={point} state={state} copy_rows={}",
                copies.join(",")
            );
        }
        if let Some((count, fewest, most)) = counts.history_unlinks.get(point) {
            let _ = writeln!(out, "name=q12 point={point} history_unlinks={count} publishes_per_unlink_fewest={fewest} publishes_per_unlink_most={most}");
        }
    }
    for ((point, state, round), publishes) in &counts.unlink_publishes {
        let _ = writeln!(out, "name=q12_measured_unlink point={point} state={state} round={round} publishes={publishes}");
    }
    for ((point, state), (taken, attempts)) in &counts.rounds_taken {
        let _ = writeln!(out, "name=q12_overwrite_rounds point={point} state={state} taken={taken} attempts={attempts}");
    }
}

/// 阳性对照与阴性对照（登记第五节）。交回每条是否过。
fn controls_report(counts: &Product, timing: Option<&Product>, out: &mut String) -> bool {
    let mut all_pass = true;
    let mut record = |name: &str, state: &str, passed: bool, detail: String, out: &mut String| {
        all_pass &= passed;
        let _ = writeln!(
            out,
            "name=verdict control={name} state={state} passed={passed} {detail}"
        );
    };
    let copy_row = "whole_allocator_copy_before_the_judgement";
    let settle_copy_row = "whole_allocator_copy_in_a_settling_round";
    for state in STATES {
        let at = |point: &str| round_key(point, state, "overwrite", 1);
        let (Some(low), Some(high)) = (
            axis_points_for_alpha(counts, "U")
                .first()
                .map(|point_axes| point_axes.0.clone()),
            axis_points_for_alpha(counts, "U")
                .last()
                .map(|point_axes| point_axes.0.clone()),
        ) else {
            record("axis_u_has_two_points", state, false, String::new(), out);
            continue;
        };
        let steps =
            |point: &str, segments: &[&str], row: &str| counts.steps_of(&at(point), segments, row);
        // B1 计数：s1 的 α(U) ≥ 0.95
        let (copy_low, copy_high) = (
            steps(&low, &["caller"], copy_row),
            steps(&high, &["caller"], copy_row),
        );
        let copy_alpha = alpha(
            to_f64(copy_low.max(1)),
            to_f64(copy_high.max(1)),
            to_f64(1 << 14),
            to_f64(1 << 24),
        );
        record(
            "b1_counts_whole_copy_alpha_at_least_0_95",
            state,
            copy_alpha >= 0.95,
            format!("alpha={copy_alpha:.6}"),
            out,
        );
        // B8 与 B4：退化后的 α(U) ≥ 0.9
        let rows_at = |point: &str| {
            (
                by_delta_rows_by_segment(counts, point, state, "overwrite"),
                today_rows_by_segment(counts, point, state, "overwrite"),
            )
        };
        let (by_delta_low, today_low) = rows_at(&low);
        let (by_delta_high, today_high) = rows_at(&high);
        if let (Ok(by_delta_low), Ok(by_delta_high)) = (by_delta_low, by_delta_high) {
            let degraded = |by_delta: &StepsByRow, today: &StepsByRow| {
                let by_delta_rest = total_of(by_delta, &SEGMENTS_BEFORE_ADMISSION)
                    - by_delta.get(&("caller", copy_row)).copied().unwrap_or(0)
                    - by_delta
                        .get(&("between", settle_copy_row))
                        .copied()
                        .unwrap_or(0);
                degraded_by_delta_total(
                    by_delta_rest,
                    today.get(&("caller", copy_row)).copied().unwrap_or(0),
                    today
                        .get(&("between", settle_copy_row))
                        .copied()
                        .unwrap_or(0),
                )
            };
            let passed = positive_control_passes(
                degraded(&by_delta_low, &today_low),
                degraded(&by_delta_high, &today_high),
                0.9,
            );
            record(
                "b8_degraded_alpha_at_least_0_9",
                state,
                passed,
                String::new(),
                out,
            );
            let narrowed_degraded = |by_delta: &StepsByRow, today: &StepsByRow| {
                total_of(by_delta, &SEGMENTS_BEFORE_THE_SECOND_JUDGEMENT)
                    - by_delta.get(&("caller", copy_row)).copied().unwrap_or(0)
                    + today.get(&("caller", copy_row)).copied().unwrap_or(0)
            };
            let passed = positive_control_passes(
                narrowed_degraded(&by_delta_low, &today_low),
                narrowed_degraded(&by_delta_high, &today_high),
                0.9,
            );
            record(
                "b4_degraded_alpha_at_least_0_9",
                state,
                passed,
                String::new(),
                out,
            );
            // 阴性对照：s33 的 B1 步数与 B8 的 s12 步数在 U 轴两端 |α| ≤ 0.1
            let negative_today = negative_control_passes(
                today_low
                    .get(&("before_landing", "units_of_a_publish_to_land"))
                    .copied()
                    .unwrap_or(0)
                    .max(1),
                today_high
                    .get(&("before_landing", "units_of_a_publish_to_land"))
                    .copied()
                    .unwrap_or(0)
                    .max(1),
            );
            record(
                "negative_units_to_land_b1",
                state,
                negative_today,
                String::new(),
                out,
            );
            let root_head = |rows: &StepsByRow| {
                rows.iter()
                    .filter(|((_, row), _)| *row == "root_header_linear_find")
                    .map(|(_, steps)| steps)
                    .sum::<u64>()
            };
            let negative_by_delta = negative_control_passes(
                root_head(&by_delta_low).max(1),
                root_head(&by_delta_high).max(1),
            );
            record(
                "negative_root_header_b8",
                state,
                negative_by_delta,
                String::new(),
                out,
            );
        } else {
            record(
                "b8_b4_controls_inputs_read",
                state,
                false,
                "unread=b8_input".to_string(),
                out,
            );
        }
        // B1 计时：s1 的独占计时五轮每轮 α(U) ≥ 0.7
        if let Some(timing) = timing {
            let mut alphas = Vec::new();
            for round in 1..=ROUNDS {
                let (low_at, high_at) = (
                    round_key(&low, state, "overwrite", round),
                    round_key(&high, state, "overwrite", round),
                );
                if !(round_is_usable(timing, &low_at) && round_is_usable(timing, &high_at)) {
                    continue;
                }
                let (low_nanos, high_nanos) = (
                    timing.nanos_of(&low_at, &["caller"], copy_row),
                    timing.nanos_of(&high_at, &["caller"], copy_row),
                );
                if low_nanos > 0 && high_nanos > 0 {
                    alphas.push(alpha(
                        to_f64(low_nanos),
                        to_f64(high_nanos),
                        to_f64(1 << 14),
                        to_f64(1 << 24),
                    ));
                }
            }
            let passed =
                alphas.len() as u64 == ROUNDS && alphas.iter().all(|growth| *growth >= 0.7);
            record(
                "b1_timing_whole_copy_alpha_at_least_0_7_in_every_round",
                state,
                passed,
                format!(
                    "alphas={}",
                    join_numbers(
                        &alphas
                            .iter()
                            .map(|growth| format!("{growth:.6}"))
                            .collect::<Vec<_>>()
                    )
                ),
                out,
            );
        }
    }
    all_pass
}

// ---- 停机条款、作废条款、失败条款的判读 ----
/// 停机条款 3：计数产物与计时产物同一个点、同一轮上的规模读数逐字段相同（除了 `build` 字段）。交回不同的字段个数。
fn state_fields_that_differ(counts: &Product, timing: &Product) -> u64 {
    let mut differing = 0;
    for (key, count_fields) in &counts.states {
        let Some(timing_fields) = timing.states.get(key) else {
            differing += 1;
            continue;
        };
        for (field, value) in count_fields {
            if field != "build" && timing_fields.get(field) != Some(value) {
                differing += 1;
            }
        }
    }
    differing
}

/// 停机条款 4：每个被量的发布都经调用方进门（有 `caller` 段），并且进了落点判之前那一段。交回违例个数。
fn publishes_that_skipped_the_caller_or_the_judge(counts: &Product) -> u64 {
    let mut violations = 0;
    for key in counts.publishes.keys() {
        let had_caller = counts.had_caller_segment(key) == Some(true);
        let entered_the_judge = counts.rows.keys().any(|row| {
            row.0 == key.0
                && row.1 == key.1
                && row.2 == key.2
                && row.3 == key.3
                && row.4 == "before_landing"
        });
        if !had_caller || !entered_the_judge {
            violations += 1;
        }
    }
    violations
}

fn clause_line(out: &mut String, clause: &str, fired: Option<bool>, detail: &str) {
    let fired = fired.map_or("unknown".to_string(), |flag| flag.to_string());
    let _ = writeln!(out, "name=verdict clause={clause} fired={fired} {detail}");
}

/// 一份计数产物（可带一份计时产物）的全部判读。
fn full_report(counts: &Product, timing: Option<&Product>, out: &mut String) {
    // 作废条款 8：量不到的格整格拿掉，不当 0 进任何一个量；每一格另点名。
    let cells_of_counts = cells_not_measurable(counts);
    let cells_of_timing = timing.map(cells_not_measurable).unwrap_or_default();
    let cleaned_counts = without_the_cells_not_measurable(counts, &cells_of_counts);
    let cleaned_timing =
        timing.map(|product| without_the_cells_not_measurable(product, &cells_of_timing));
    let (counts, timing) = (&cleaned_counts, cleaned_timing.as_ref());
    for (build, cells) in [("counts", &cells_of_counts), ("timing", &cells_of_timing)] {
        for (point, state, taken, attempts) in cells {
            let _ = writeln!(
                out,
                "name=void_8_cell build={build} point={point} state={state} kind=overwrite taken={taken} attempts={attempts} result=not_measurable"
            );
        }
    }
    let _ = writeln!(
        out,
        "name=verdict clause=void_8 cells_not_measurable_in_counts={} cells_not_measurable_in_timing={}",
        cells_of_counts.len(),
        cells_of_timing.len()
    );
    counts_report(counts, out);
    let (by_delta_summary, narrowed_summary) = arms_report(counts, out);
    memory_report(counts, out);
    between_segment_report(counts, timing, out);
    user_scale_steps_report(counts, timing, out);
    side_quantities_report(counts, out);
    let _ = writeln!(out, "name=verdict quantity=Q6 result={by_delta_summary:?}");
    let _ = writeln!(
        out,
        "name=verdict quantity=Q8_b4_arm result={narrowed_summary:?}"
    );
    let controls_pass = controls_report(counts, timing, out);
    let _ = writeln!(
        out,
        "name=verdict quantity=controls all_passed={controls_pass}"
    );
    // 停机条款与作废条款
    let constant_mismatches = out
        .lines()
        .filter(|line| line.starts_with("name=constant_check") && line.ends_with("match=false"))
        .count();
    let (_, three_two_disagreements, _) = reconciliation_report(counts, out);
    let _ = writeln!(
        out,
        "name=verdict clause=stop_2 constants_not_matching_or_unread={constant_mismatches} three_two_disagreements={three_two_disagreements}"
    );
    let _ = writeln!(
        out,
        "name=verdict clause=stop_4 publishes_that_skipped_the_caller_or_the_judge={}",
        publishes_that_skipped_the_caller_or_the_judge(counts)
    );
    let _ = writeln!(
        out,
        "name=verdict clause=void_6 points={} points_with_the_done_marker={}",
        counts.points.len(),
        counts.items_done.len()
    );
    // 失败条款 F1–F6
    let mut residual_clause = None;
    let mut flips_today_clause = None;
    let mut extrapolation_clause = None;
    if let Some(timing) = timing {
        let _ = writeln!(
            out,
            "name=verdict clause=stop_3 state_fields_that_differ={}",
            state_fields_that_differ(counts, timing)
        );
        let header_ok = timing.header.get("profile").map(String::as_str) == Some("release")
            && timing.header.get("opt_level").map(String::as_str) == Some("3")
            && timing.header.get("test_threads").map(String::as_str) == Some("1");
        let _ = writeln!(out, "name=verdict clause=void_7 timing_header_is_release_opt_level_3_single_thread={header_ok}");
        residual_clause = Some(timing_report(timing, out));
        let (cells, reference_only) = one_millisecond_report(counts, timing, out);
        let checks = extrapolation_check_report(counts, timing, out);
        // 一格都没核（轴上不足三个点）就是没判，不记「核过」。
        extrapolation_clause = (!checks.is_empty()).then(|| checks.iter().any(|ok| !*ok));
        flips_today_clause = if cells.len() == 4 && !reference_only {
            Some(
                cells
                    .iter()
                    .all(|one_cell| *one_cell == FormulaCell::AtLeastOneMillisecond),
            )
        } else {
            None
        };
        let _ = writeln!(out, "name=verdict quantity=Q4 cells={} reference_only_because_data_or_files_axes_are_missing={reference_only}", cells.iter().map(|one_cell| format!("{one_cell:?}")).collect::<Vec<_>>().join(","));
        let _ = writeln!(
            out,
            "name=verdict quantity=Q5 cells_checked={} cells_not_checked={}",
            checks.iter().filter(|ok| **ok).count(),
            checks.iter().filter(|ok| !**ok).count()
        );
    } else {
        let _ = writeln!(
            out,
            "name=verdict quantity=timing_quantities result=no_timing_product_given"
        );
    }
    clause_line(
        out,
        "F1",
        flips_today_clause,
        "meaning=all_four_Q4_cells_at_least_one_millisecond_flips_B1",
    );
    clause_line(
        out,
        "F2",
        Some(by_delta_summary == Summary::GrowsOnSomeAxis),
        "meaning=Q6_B8_still_grows_on_some_axis",
    );
    clause_line(
        out,
        "F3",
        residual_clause,
        "meaning=residual_grows_on_some_axis_in_all_five_rounds",
    );
    let unlink_not_two = counts
        .unlink_publishes
        .values()
        .filter(|value| **value != 2)
        .count()
        + counts
            .history_unlinks
            .values()
            .filter(|(_, fewest, most)| *fewest != 2 || *most != 2)
            .count();
    clause_line(
        out,
        "F4",
        Some(unlink_not_two > 0),
        &format!("unlinks_not_two_publishes={unlink_not_two}"),
    );
    clause_line(
        out,
        "F5",
        Some(narrowed_summary == Summary::GrowsOnSomeAxis),
        "meaning=B4_arm_before_the_landing_judgement_still_grows",
    );
    clause_line(
        out,
        "F6",
        extrapolation_clause,
        "meaning=some_Q5_cell_not_checked",
    );
}

fn run_report(paths: &[String]) -> ExitCode {
    let split = paths.iter().position(|path| path == "--timing");
    let (count_paths, timing_paths) = match split {
        Some(index) => (&paths[..index], &paths[index + 1..]),
        None => (paths, &paths[paths.len()..]),
    };
    let read_all = |list: &[String]| -> Option<String> {
        let mut text = String::new();
        for path in list {
            let Ok(part) = std::fs::read_to_string(path) else {
                eprintln!("读不了产物 {path}");
                return None;
            };
            text.push_str(&part);
        }
        Some(text)
    };
    let (Some(count_text), Some(timing_text)) = (read_all(count_paths), read_all(timing_paths))
    else {
        return ExitCode::from(2);
    };
    let counts = read_product(&count_text);
    let timing = (!timing_paths.is_empty()).then(|| read_product(&timing_text));
    let mut out = String::new();
    let _ = writeln!(
        out,
        "name=report_of counts_files={} timing_files={}",
        count_paths.len(),
        timing_paths.len()
    );
    full_report(&counts, timing.as_ref(), &mut out);
    print!("{out}");
    ExitCode::SUCCESS
}

fn anchors_lines() -> String {
    let mut out = String::new();
    for (name, unit_area_slots) in [
        ("U=2^14", 1_u64 << 14),
        ("U=2^17", 1 << 17),
        ("U=2^20", 1 << 20),
        ("U=2^24", 1 << 24),
        ("G-user", user_scale_unit_area_slots()),
    ] {
        let _ = writeln!(
            out,
            "name=clone_vectors point={name} unit_area_slots={unit_area_slots} device_bytes={} segments={} elements_two_disks={} bytes_two_disks={} lg_segments={}",
            device_bytes(unit_area_slots),
            segments_of(unit_area_slots),
            allocator_copy_elements_from_the_unit_area(unit_area_slots),
            DISKS * bytes_per_disk(unit_area_slots),
            lg(segments_of(unit_area_slots))
        );
    }
    let total = by_delta_total(&delta_zero());
    let _ = writeln!(out, "name=b8_anchor_delta0 total={total}");
    out
}

fn main() -> ExitCode {
    let command_line: Vec<String> = std::env::args().skip(1).collect();
    match command_line.first().map(String::as_str) {
        Some("report") if command_line.len() >= 2 => run_report(&command_line[1..]),
        Some("anchors") => {
            print!("{}", anchors_lines());
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("用法：e203-admission-scan-cost report <计数产物>… [--timing <计时产物>…] | anchors");
            ExitCode::from(2)
        }
    }
}

/// 线性找走过的步数（登记第七节 ③2 里「找到那一个」也算一步）：答案在第 `position` 个（从 0 起）走 position + 1 步，没找到走完整张表。
fn linear_find_steps(position_of_the_answer: Option<u64>, table_length: u64) -> u64 {
    position_of_the_answer.map_or(table_length, |position| position + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rounded(value: f64) -> String {
        format!("{value:.6}")
    }

    #[test]
    fn levels_and_the_integer_functions_hit_the_registered_values() {
        let lg_values = [
            lg(0),
            lg(1),
            lg(2),
            lg(3),
            lg(233),
            lg(8),
            lg(1000),
            lg(2000),
            lg(262_144),
            lg(50),
            lg(3000),
            lg(1500),
            lg(9),
        ];
        assert_eq!(lg_values, [0, 1, 2, 2, 8, 4, 10, 11, 19, 6, 12, 11, 4]);
        let levels = [
            levels_64(256),
            levels_64(2048),
            levels_64(16384),
            levels_64(262_144),
            levels_64(16_777_194),
        ];
        assert_eq!(levels, [2, 2, 3, 3, 4]);
        assert_eq!(
            [
                summary_words(256),
                summary_words(262_144),
                summary_words(16_777_194)
            ],
            [5, 4161, 266_305]
        );
    }

    #[test]
    fn the_allocator_copy_from_the_unit_area_hits_the_registered_counts() {
        let expected = [
            (1_u64 << 14, 100_864, 118_784),
            (1 << 17, 806_912, 950_272),
            (1 << 20, 6_455_296, 7_602_176),
            (1 << 24, 103_284_736, 121_634_816),
        ];
        for (unit_area, elements, bytes) in expected {
            assert_eq!(
                allocator_copy_elements_from_the_unit_area(unit_area),
                elements
            );
            assert_eq!(DISKS * bytes_per_disk(unit_area), bytes);
        }
        let user_scale = user_scale_unit_area_slots();
        assert_eq!(
            allocator_copy_elements_from_the_unit_area(user_scale),
            6_610_214_436
        );
        assert_eq!(DISKS * bytes_per_disk(user_scale), 7_784_618_016);
        assert_eq!(
            allocator_copy_elements_from_the_unit_area(1 << 30)
                - allocator_copy_elements_from_the_unit_area(user_scale),
            8668,
            "G-user 的 U 减 1408（变异 m10 取 2³⁰ 时差 0）"
        );
    }

    #[test]
    fn the_user_scale_geometry_hits_the_registered_values() {
        let unit_area_slots = user_scale_unit_area_slots();
        assert_eq!(unit_area_slots, 1_073_740_416);
        assert_eq!(device_bytes(unit_area_slots), 17_592_186_044_416);
        assert_eq!(segments_of(unit_area_slots), 16_777_194);
        assert_eq!(
            (
                lg(segments_of(unit_area_slots)),
                levels_64(segments_of(unit_area_slots))
            ),
            (24, 4)
        );
        assert_eq!(tombstone_containers(1_000_000), 1227);
        assert_eq!(
            [
                tombstone_containers(10),
                tombstone_containers(1000),
                tombstone_containers(10_000)
            ],
            [1, 2, 13]
        );
    }

    #[test]
    fn the_growth_order_examples_hit_the_registered_values() {
        let linear = alpha(
            to_f64(elements_per_disk(1 << 14)),
            to_f64(elements_per_disk(1 << 24)),
            to_f64(1 << 14),
            to_f64(1 << 24),
        );
        assert_eq!(rounded(linear), "1.000000");
        let lg_unit_area = alpha(
            to_f64(lg(1 << 14)),
            to_f64(lg(1 << 24)),
            to_f64(1 << 14),
            to_f64(1 << 24),
        );
        assert_eq!(rounded(lg_unit_area), "0.073697");
        let lg_axis = alpha(to_f64(lg(10)), to_f64(lg(10_000)), 10.0, 10_000.0);
        assert_eq!(rounded(lg_axis), "0.181356");
        assert_eq!(
            rounded(alpha(10_f64.sqrt(), 10_000_f64.sqrt(), 10.0, 10_000.0)),
            "0.500000"
        );
        assert_eq!(
            rounded(alpha(to_f64(lg(1)), to_f64(lg(10_000)), 1.0, 10_000.0)),
            "0.286532"
        );
    }

    #[test]
    fn the_two_bands_split_the_growth_orders_and_leave_the_middle_open() {
        assert_eq!(band(0.3), Band::Between, "判档的两条带不能对调（变异 m7）");
        assert_eq!(
            (band(0.5), band(0.25), band(0.073697), band(1.0)),
            (
                Band::GrowsWithTheAxis,
                Band::DoesNotGrow,
                Band::DoesNotGrow,
                Band::GrowsWithTheAxis
            )
        );
        assert_eq!(
            band_of_the_rounds(&[0.9, 0.8, 1.0, 0.7, 0.6]),
            Some(Band::GrowsWithTheAxis)
        );
        assert_eq!(
            band_of_the_rounds(&[0.9, 0.1, 1.0, 0.7, 0.6]),
            None,
            "五轮不同档记不稳定"
        );
        assert_eq!(band_of_the_rounds(&[]), None);
    }

    #[test]
    fn by_delta_on_the_synthetic_delta_zero_hits_the_registered_rows_and_total() {
        let rows: BTreeMap<&str, u64> = by_delta_rows(&delta_zero()).into_iter().collect();
        let expected = [
            ("whole_allocator_copy_before_the_judgement", 0),
            ("inode_record_linear_find", 0),
            ("files_other_than_the_first_plan", 22),
            ("file_other_than_the_first_linear_find", 20),
            ("inodes_with_an_extent_entry_set", 20),
            ("extent_upper_segment_plan", 16),
            ("extent_roles_rewritten_scan", 2),
            ("root_header_linear_find", 8),
            ("checkpoint_cost_accounting_tree_plan", 60),
            ("cluster_segment_sum", 4),
            ("allocation_record_tree_node_set", 24),
            ("allocation_record_tree_plan", 24),
            ("inode_number_watermark_steps", 4),
            ("deadlist_changes_indexes", 0),
            ("multi_level_tree_planning", 68),
            ("mapped_units_of_container_trees", 0),
            ("moved_entries_path_tail_filter", 0),
            ("release_judgement_version_indexes", 138),
            ("whole_allocator_copy_in_a_settling_round", 60),
            ("allocator_release_linear_find", 110),
            ("user_data_slot_linear_scan", 70),
            ("record_intersection_filter", 480),
            ("empty_segment_linear_scan", 12),
            ("fallback_slot_linear_scan", 0),
            ("records_of_each_leaf", 160),
            ("accounting_entries_of_this_publish", 30),
            ("instance_table_chain", 0),
            ("mapped_units_whole_table", 4),
        ];
        for (row, steps) in expected {
            assert_eq!(rows[row], steps, "行 {row}");
        }
        assert_eq!(rows.len(), expected.len());
        assert_eq!(by_delta_total(&delta_zero()), 1336);
    }

    fn delta_zero_memory() -> MemoryInput {
        MemoryInput {
            allocation_records: 2000,
            segments_per_disk: 262_144,
            units: 3000,
            mapped_units: 1500,
            files_in_the_index: 1001,
            extent_upper_nodes: 9,
            deadlist_entries: 0,
            accounting_rows_of_the_generation: 15,
            unit_bytes: 24,
            key_bytes: 24,
            accounting_row_bytes: 48,
        }
    }

    #[test]
    fn by_delta_memory_and_maintenance_on_delta_zero_hit_the_registered_values() {
        let memory = by_delta_memory(&delta_zero_memory());
        assert_eq!(memory.segment_summaries, 199_728);
        assert_eq!(
            (
                memory.units_by_role,
                memory.mapped_units_by_role,
                memory.record_index
            ),
            (96_000, 48_000, 48_000)
        );
        assert_eq!(
            (
                memory.files_by_inode,
                memory.inodes_with_an_extent_entry,
                memory.accounting_rows_of_the_generation
            ),
            (16_016, 8008, 720)
        );
        assert_eq!(
            (memory.extent_upper_nodes, memory.root_head_positions),
            (72, 32)
        );
        assert_eq!(memory.payload(), 416_576);
        assert_eq!(2 * memory.payload(), 833_152);
        let maintenance = MaintenanceInput {
            records_changed: 30,
            segments_flipped: 4,
            units_delta: 20,
            mapped_delta: 2,
            ..MaintenanceInput::default()
        };
        assert_eq!(
            by_delta_maintenance(&delta_zero_memory(), &maintenance),
            619
        );
        assert_eq!((undo_log_bytes(60), undo_log_bytes(30)), (1440, 720));
    }

    #[test]
    fn narrowed_takes_the_cheaper_function_in_the_second_segment_too() {
        let mut today: StepsByRow = BTreeMap::new();
        let mut by_delta: StepsByRow = BTreeMap::new();
        for (row, today_steps, by_delta_steps) in [
            ("root_header_linear_find", 100, 8),
            ("checkpoint_cost_accounting_tree_plan", 200, 60),
            ("inode_number_watermark_steps", 5, 4),
            ("allocator_release_linear_find", 7, 110),
        ] {
            today.insert(("between", row), today_steps);
            by_delta.insert(("between", row), by_delta_steps);
        }
        today.insert(("before_landing", "root_header_linear_find"), 40);
        by_delta.insert(("before_landing", "root_header_linear_find"), 4);
        today.insert(
            ("before_landing", "checkpoint_cost_accounting_tree_plan"),
            50,
        );
        by_delta.insert(
            ("before_landing", "checkpoint_cost_accounting_tree_plan"),
            10,
        );
        today.insert(("before_landing", "inode_number_watermark_steps"), 3);
        by_delta.insert(("before_landing", "inode_number_watermark_steps"), 2);
        let called: BTreeSet<&'static str> = [
            "root_header_linear_find",
            "checkpoint_cost_accounting_tree_plan",
            "inode_number_watermark_steps",
        ]
        .into_iter()
        .collect();
        let [caller_steps, landing_steps, between_steps] =
            narrowed_steps_by_segment(&today, &by_delta, &called);
        assert_eq!(
            (caller_steps, landing_steps),
            (0, 16),
            "第一段与调用方段每行取 B8 一列"
        );
        let today_between_steps: u64 = 100 + 200 + 5 + 7;
        assert_eq!(
            between_steps,
            today_between_steps - (100 - 8) - (200 - 60) - (5 - 4),
            "S2 里在 P0 或 S1 里调到的行也取 B8 一列（变异 m6），s25 只在 S2 调到、照 B1"
        );
    }

    #[test]
    fn the_positive_controls_see_the_whole_copy_and_the_negative_control_sees_the_constant() {
        let low = degraded_by_delta_total(1000, 0, 100_864);
        let high = degraded_by_delta_total(1000, 0, 103_284_736);
        assert!(
            positive_control_passes(low, high, 0.9),
            "退化后 s24 随 U 一次涨：α 不小于 0.9（变异 m12 漏退 s24 时红）"
        );
        assert!(
            !positive_control_passes(1000, 1000, 0.9),
            "不随 U 的合计过不了阳性对照"
        );
        assert!(negative_control_passes(8, 8));
        assert!(!negative_control_passes(8, 80_000));
    }

    #[test]
    fn the_inclusion_check_the_tau_choice_and_the_cells_hold() {
        assert!(
            !round_is_valid(1001, 1000),
            "Σ 独占计时 > 段挂钟的一轮作废（变异 m8）"
        );
        assert!(round_is_valid(1000, 1000));
        assert_eq!(
            tau_nanos_per_step(&[(10, 100), (1000, 5000)]),
            5.0,
            "τ 取步数最大的那个点（变异 m9）"
        );
        assert_eq!(
            one_millisecond_cell(&[2.0e6, 3.0e6, 2.5e6, 9.0e6, 2.0e6]),
            FormulaCell::AtLeastOneMillisecond
        );
        assert_eq!(
            one_millisecond_cell(&[0.5e6, 0.1e6, 0.2e6, 0.4e6, 0.3e6]),
            FormulaCell::BelowOneMillisecond
        );
        assert_eq!(
            one_millisecond_cell(&[2.0e6, 0.1e6, 2.5e6, 9.0e6, 2.0e6]),
            FormulaCell::Unstable
        );
        assert!(extrapolation_is_checked(105.0, 100.0, 1.9));
        assert!(!extrapolation_is_checked(115.0, 100.0, 1.0));
        assert!(!extrapolation_is_checked(100.0, 100.0, 2.1));
        assert_eq!(
            linear_extrapolation(1000.0, 10.0, 10_000.0, 100.0, 100_000.0),
            1000.0
        );
    }

    #[test]
    fn linear_find_counts_the_step_that_finds_the_answer() {
        assert_eq!(
            linear_find_steps(Some(0), 50),
            1,
            "找到第一个也走一步（变异 m2）"
        );
        assert_eq!(linear_find_steps(Some(3), 50), 4);
        assert_eq!(linear_find_steps(None, 50), 50);
    }

    #[test]
    fn the_report_reads_the_rows_and_gives_the_growth_order_of_the_scan_sum() {
        let mut text = String::new();
        for (point, unit_area_slots) in [
            ("unit_area_2_pow_14", 1_u64 << 14),
            ("base", 1 << 17),
            ("unit_area_2_pow_24", 1 << 24),
        ] {
            let _ = writeln!(text, "E203RESULT name=point point={point} unit_area_slots={unit_area_slots} unlinks=10 data_units=10 files=10");
            let steps = 2 * elements_per_disk(unit_area_slots);
            let _ = writeln!(text, "E203RESULT name=row point={point} state=same_mount kind=overwrite round=1 segment=caller row=whole_allocator_copy_before_the_judgement calls=1 steps={steps} scale={steps}");
        }
        let mut out = String::new();
        counts_report(&read_product(&text), &mut out);
        let line = out
            .lines()
            .find(|line_text| {
                line_text.starts_with("name=q2 axis=U state=same_mount kind=overwrite")
            })
            .expect("U 轴一行");
        assert!(
            line.contains("alpha=1.000000") && line.contains("GrowsWithTheAxis"),
            "{line}"
        );
        assert!(
            out.contains("name=verdict quantity=three_one_copy_elements disagreements=0"),
            "{out}"
        );
        assert!(out.contains("name=constant_check key=slot_bytes model=16384 implementation=NOT_READ match=false"), "读不到不是 0");
    }

    /// 登记第七节 ③3 的 Δ0 写成产物行：从产物里读 B8 的输入，再算出来的合计必须还是 1336（读数接线与式子一起钉住）。
    fn delta_zero_product_text() -> String {
        let at = "point=base state=same_mount kind=overwrite round=1";
        let mut text = String::new();
        let mut line = |rest: &str| {
            let _ = writeln!(text, "E203RESULT {rest}");
        };
        line("name=point point=base unit_area_slots=16777216 unlinks=10 data_units=10 files=10");
        line(&format!("name=state {at} units=3000 mapped_units=1500 allocation_records=2000 other_files=1000 inode_leaf_containers=8 deadlist_entries=0 extent_upper_nodes=9 extent_upper_levels=2 accounting_tree_levels=2 deadlist_tree_levels=1 allocation_tree_levels=3 central_mapping_levels=2"));
        line(&format!(
            "name=publish {at} had_caller_segment=true fixed_point_rounds=2"
        ));
        for (row, calls) in [
            ("files_other_than_the_first_plan", 2),
            ("file_other_than_the_first_linear_find", 2),
            ("inodes_with_an_extent_entry_set", 2),
            ("extent_upper_segment_plan", 2),
            ("extent_roles_rewritten_scan", 2),
            ("root_header_linear_find", 8),
            ("checkpoint_cost_accounting_tree_plan", 2),
            ("cluster_segment_sum", 4),
            ("inode_number_watermark_steps", 4),
            ("allocator_release_linear_find", 10),
            ("user_data_slot_linear_scan", 2),
            ("record_intersection_filter", 40),
            ("empty_segment_linear_scan", 4),
            ("accounting_entries_of_this_publish", 2),
        ] {
            line(&format!(
                "name=row {at} segment=before_landing row={row} calls={calls} steps=1"
            ));
        }
        line(&format!("name=row {at} segment=between row=release_judgement_version_indexes calls=1 query_levels=138"));
        let mut delta = |segment: &str, fixed_point_round: u32, key: &str, value: u32| {
            line(&format!("name=delta {at} segment={segment} fixed_point_round={fixed_point_round} key={key} value={value}"));
        };
        delta("before_landing", 0, "touched_files", 1);
        delta("before_landing", 0, "lower_nodes_of_touched_files", 1);
        delta(
            "before_landing",
            0,
            "accounting_rows_written_by_the_built_on_publish",
            15,
        );
        delta("between", 0, "deadlist_entries_changed", 0);
        delta("between", 0, "moved_entries", 0);
        delta("between", 0, "intersecting_records", 40);
        for round in [1, 2] {
            delta("between", round, "changed_leaves", 4);
            delta("between", round, "round_placements_released", 29);
            delta("between", round, "round_roles_rewritten", 1);
            delta("between", round, "container_units_rewritten", 0);
            for keys_changed in [15, 0, 2] {
                delta("between", round, "code_two_keys_changed", keys_changed);
            }
            for _ in 0..2 {
                delta("between", round, "dirty_records", 40);
            }
        }
        text
    }

    #[test]
    fn the_by_delta_inputs_read_from_a_product_reproduce_the_registered_total() {
        let product = read_product(&delta_zero_product_text());
        let input = by_delta_input_from_the_product(
            &product,
            "base",
            "same_mount",
            "overwrite",
            &SEGMENTS_BEFORE_ADMISSION,
        )
        .expect("Δ0 的每个读数都给了");
        assert_eq!(by_delta_total(&input), 1336);
        let by_row: BTreeMap<&str, u64> = by_delta_rows(&input).into_iter().collect();
        assert_eq!(by_row["release_judgement_version_indexes"], 138);
        assert_eq!(by_row["multi_level_tree_planning"], 68);
        assert_eq!(by_row["record_intersection_filter"], 480);
    }

    #[test]
    fn an_unread_value_is_a_gap_and_not_a_zero() {
        let text =
            delta_zero_product_text().replace("key=changed_leaves", "key=changed_leaves_renamed");
        let product = read_product(&text);
        let gaps = by_delta_input_from_the_product(
            &product,
            "base",
            "same_mount",
            "overwrite",
            &SEGMENTS_BEFORE_ADMISSION,
        )
        .expect_err("少了每轮内容变了的叶数");
        assert!(
            gaps.iter()
                .any(|gap| gap.starts_with("changed_leaves@round")),
            "{gaps:?}"
        );
    }

    #[test]
    fn the_extrapolation_and_the_summary_rules_hold() {
        let points = [(10.0, 1.0), (100.0, 2.0), (1000.0, 3.0)];
        assert_eq!(value_on_axis(&points, 100.0), Some(2.0), "量过的点直接取");
        assert_eq!(value_on_axis(&points, 550.0), Some(2.5), "两点之间线性内插");
        assert_eq!(
            value_on_axis(&points, 10_000.0),
            Some(13.0),
            "最后两点的斜率线性外推"
        );
        assert_eq!(value_on_axis(&points[..1], 20.0), None, "一个点外推不出");
        let grows = Some(Band::GrowsWithTheAxis);
        let flat = Some(Band::DoesNotGrow);
        assert_eq!(summarize(&[flat, grows], false), Summary::GrowsOnSomeAxis);
        assert_eq!(
            summarize(&[flat, flat], false),
            Summary::DoesNotGrowOnAnyAxis
        );
        assert_eq!(
            summarize(&[flat, Some(Band::Between)], false),
            Summary::BetweenOnSomeAxis
        );
        assert_eq!(
            summarize(&[flat], true),
            Summary::SomeInputUnread,
            "有格读不到不记「不随」"
        );
        assert_eq!(summarize(&[], false), Summary::SomeInputUnread);
    }

    fn timing_product_text(wall_of_round_three: u64) -> String {
        let mut text = String::new();
        for (point, unit_area_slots, nanos) in [
            ("unit_area_2_pow_14", 1_u64 << 14, 1000_u64),
            ("unit_area_2_pow_24", 1 << 24, 1_000_000),
        ] {
            let _ = writeln!(text, "E203RESULT name=point point={point} unit_area_slots={unit_area_slots} unlinks=10 data_units=10 files=10");
            for round in 1..=5 {
                let at = format!("point={point} state=same_mount kind=overwrite round={round}");
                let _ = writeln!(text, "E203RESULT name=row {at} segment=caller row=whole_allocator_copy_before_the_judgement exclusive_nanos={nanos}");
                let wall = if round == 3 && nanos > 1000 {
                    wall_of_round_three
                } else {
                    nanos + 5
                };
                let _ = writeln!(
                    text,
                    "E203RESULT name=segment_nanos {at} segment=caller nanos={wall}"
                );
            }
        }
        text
    }

    #[test]
    fn the_timing_report_bands_five_rounds_and_drops_a_round_that_breaks_the_inclusion_check() {
        let mut out = String::new();
        let timing = read_product(&timing_product_text(1_000_005));
        let residual_grows = timing_report(&timing, &mut out);
        assert!(!residual_grows, "残差只有 5 纳秒，不随轴涨");
        let line = out
            .lines()
            .find(|line| line.starts_with("name=q3 axis=U state=same_mount kind=overwrite"))
            .expect("U 轴一行");
        assert!(line.ends_with("band=GrowsWithTheAxis"), "{line}");
        let mut broken = String::new();
        timing_report(&read_product(&timing_product_text(10)), &mut broken);
        let line = broken
            .lines()
            .find(|line| line.starts_with("name=q3 axis=U state=same_mount kind=overwrite"))
            .expect("U 轴一行");
        assert!(
            line.ends_with("band=fewer_than_five_valid_rounds"),
            "第 3 轮 Σ 独占计时大于段挂钟：作废，不足五轮 {line}"
        );
        let mut controls = String::new();
        let counts = read_product(&timing_product_text(1_000_005));
        controls_report(&counts, Some(&timing), &mut controls);
        assert!(
            controls.contains("control=b1_timing_whole_copy_alpha_at_least_0_7_in_every_round state=same_mount passed=true"),
            "{controls}"
        );
    }

    #[test]
    fn the_failure_clause_four_fires_on_a_publish_count_other_than_two() {
        let text = "E203RESULT name=point point=base unit_area_slots=131072 unlinks=10 data_units=10 files=10\n\
                    E203RESULT name=unlink_publishes point=base state=same_mount round=1 publishes=3\n";
        let mut out = String::new();
        full_report(&read_product(text), None, &mut out);
        assert!(out.contains("name=verdict clause=F4 fired=true"), "{out}");
    }

    fn push_line(text: &mut String, rest: &str) {
        let _ = writeln!(text, "E203RESULT {rest}");
    }

    /// 合成的一个点：Δ0 的每个读数，只让两样随这个点变——每轮内容变了的叶数（两道之间那一段读）与根头线性找的调用次数（落点判之前读）。
    struct SyntheticPoint {
        name: &'static str,
        unit_area_slots: u64,
        unlinks: u64,
        data_units: u64,
        files: u64,
        changed_leaves: u64,
        root_header_calls: u64,
    }

    fn delta_zero_row_calls() -> [(&'static str, u64); 14] {
        [
            ("files_other_than_the_first_plan", 2),
            ("file_other_than_the_first_linear_find", 2),
            ("inodes_with_an_extent_entry_set", 2),
            ("extent_upper_segment_plan", 2),
            ("extent_roles_rewritten_scan", 2),
            ("root_header_linear_find", 8),
            ("checkpoint_cost_accounting_tree_plan", 2),
            ("cluster_segment_sum", 4),
            ("inode_number_watermark_steps", 4),
            ("allocator_release_linear_find", 10),
            ("user_data_slot_linear_scan", 2),
            ("record_intersection_filter", 40),
            ("empty_segment_linear_scan", 4),
            ("accounting_entries_of_this_publish", 2),
        ]
    }

    /// 每个点、两种挂载状态、三种发布都写同一套 Δ0 读数（登记第七节 ③3），B8 的式子从这段文字里读得出输入。
    fn synthetic_counts_text(points: &[SyntheticPoint]) -> String {
        let mut text = String::new();
        for point in points {
            push_line(
                &mut text,
                &format!(
                    "name=point point={} unit_area_slots={} unlinks={} data_units={} files={}",
                    point.name, point.unit_area_slots, point.unlinks, point.data_units, point.files
                ),
            );
            for state in STATES {
                for kind in KINDS {
                    let at = format!("point={} state={state} kind={kind} round=1", point.name);
                    push_line(&mut text, &format!("name=state {at} units=3000 mapped_units=1500 allocation_records=2000 other_files=1000 inode_leaf_containers=8 deadlist_entries=0 extent_upper_nodes=9 extent_upper_levels=2 accounting_tree_levels=2 deadlist_tree_levels=1 allocation_tree_levels=3 central_mapping_levels=2"));
                    push_line(
                        &mut text,
                        &format!("name=publish {at} had_caller_segment=true fixed_point_rounds=2"),
                    );
                    for (row, calls) in delta_zero_row_calls() {
                        let calls = if row == "root_header_linear_find" {
                            point.root_header_calls
                        } else {
                            calls
                        };
                        push_line(&mut text, &format!("name=row {at} segment=before_landing row={row} calls={calls} steps=1"));
                    }
                    push_line(&mut text, &format!("name=row {at} segment=between row=release_judgement_version_indexes calls=1 query_levels=138"));
                    let mut deltas: Vec<(&str, u64, &str, u64)> = vec![
                        ("before_landing", 0, "touched_files", 1),
                        ("before_landing", 0, "lower_nodes_of_touched_files", 1),
                        (
                            "before_landing",
                            0,
                            "accounting_rows_written_by_the_built_on_publish",
                            15,
                        ),
                        ("between", 0, "deadlist_entries_changed", 0),
                        ("between", 0, "moved_entries", 0),
                        ("between", 0, "intersecting_records", 40),
                    ];
                    for round in [1, 2] {
                        deltas.push(("between", round, "changed_leaves", point.changed_leaves));
                        deltas.push(("between", round, "round_placements_released", 29));
                        deltas.push(("between", round, "round_roles_rewritten", 1));
                        deltas.push(("between", round, "container_units_rewritten", 0));
                        for keys_changed in [15, 0, 2] {
                            deltas.push(("between", round, "code_two_keys_changed", keys_changed));
                        }
                        for _ in 0..2 {
                            deltas.push(("between", round, "dirty_records", 40));
                        }
                    }
                    for (segment, fixed_point_round, key, value) in deltas {
                        push_line(&mut text, &format!("name=delta {at} segment={segment} fixed_point_round={fixed_point_round} key={key} value={value}"));
                    }
                }
            }
        }
        text
    }

    /// 单元区轴上的两个点：低端 2¹⁴、高端 2²⁴，高端的两样随点变的读数由调用方给。
    fn synthetic_unit_area_points(
        changed_leaves_at_the_high_end: u64,
        root_header_calls_at_the_high_end: u64,
    ) -> Vec<SyntheticPoint> {
        vec![
            SyntheticPoint {
                name: "unit_area_2_pow_14",
                unit_area_slots: 1 << 14,
                unlinks: 10,
                data_units: 10,
                files: 10,
                changed_leaves: 4,
                root_header_calls: 8,
            },
            SyntheticPoint {
                name: "unit_area_2_pow_24",
                unit_area_slots: 1 << 24,
                unlinks: 10,
                data_units: 10,
                files: 10,
                changed_leaves: changed_leaves_at_the_high_end,
                root_header_calls: root_header_calls_at_the_high_end,
            },
        ]
    }

    fn report_of_counts(text: &str) -> String {
        let mut out = String::new();
        full_report(&read_product(text), None, &mut out);
        out
    }

    #[test]
    fn failure_clauses_two_and_five_each_read_their_own_arm() {
        let flat = report_of_counts(&synthetic_counts_text(&synthetic_unit_area_points(4, 8)));
        assert!(
            flat.contains("name=verdict quantity=Q6 result=DoesNotGrowOnAnyAxis"),
            "{flat}"
        );
        assert!(
            flat.contains("name=verdict quantity=Q8_b4_arm result=DoesNotGrowOnAnyAxis"),
            "{flat}"
        );
        assert!(
            flat.contains("name=verdict clause=F2 fired=false"),
            "{flat}"
        );
        assert!(
            flat.contains("name=verdict clause=F5 fired=false"),
            "{flat}"
        );
        let grows_between = report_of_counts(&synthetic_counts_text(&synthetic_unit_area_points(
            40_960, 8,
        )));
        assert!(
            grows_between.contains("name=verdict clause=F2 fired=true"),
            "B8 的步数随单元区涨（两道之间那一段）：F2 触发 {grows_between}"
        );
        assert!(
            grows_between.contains("name=verdict clause=F5 fired=false"),
            "落点判之前剩下的步数不涨：F5 不触发（变异：F5 读 B8 的总结时红）{grows_between}"
        );
        let grows_before_landing = report_of_counts(&synthetic_counts_text(
            &synthetic_unit_area_points(4, 81_920),
        ));
        assert!(
            grows_before_landing.contains("name=verdict clause=F2 fired=true"),
            "{grows_before_landing}"
        );
        assert!(
            grows_before_landing.contains("name=verdict clause=F5 fired=true"),
            "落点判之前的根头线性找随单元区涨：F5 触发 {grows_before_landing}"
        );
        assert!(
            grows_before_landing
                .contains("name=q6_user_scale state=same_mount kind=overwrite data_units_user=10 "),
            "{grows_before_landing}"
        );
    }

    /// 合成的一张格子：只有整份拷分配器那一行有步数与计时，十个点覆盖 U、u、D、F 四根轴（Q4 与 Q5 要每根轴至少三个点）。
    struct GridShape {
        /// 每一步的纳秒（整数）。
        nanos_per_step: u64,
        /// 单元区轴高端那一点（2²⁴）的独占计时乘几倍。
        high_end_factor: u64,
        /// 单元区轴高端那一点的段挂钟比独占计时多多少纳秒（别的点多 5 纳秒）。
        high_end_residual: u64,
        /// 重挂载之后那一种状态里这一行的步数取 0。
        remounted_has_no_steps: bool,
    }

    /// (点名, U, u, D, F)
    fn grid_points() -> Vec<(&'static str, u64, u64, u64, u64)> {
        vec![
            ("base", 1 << 17, 10, 10, 10),
            ("unit_area_2_pow_14", 1 << 14, 10, 10, 10),
            ("unit_area_2_pow_20", 1 << 20, 10, 10, 10),
            ("unit_area_2_pow_24", 1 << 24, 10, 10, 10),
            ("unlinks_100", 1 << 17, 100, 10, 10),
            ("unlinks_1000", 1 << 17, 1000, 10, 10),
            ("data_units_100", 1 << 17, 10, 100, 10),
            ("data_units_1000", 1 << 17, 10, 1000, 10),
            ("files_100", 1 << 17, 10, 10, 100),
            ("files_1000", 1 << 17, 10, 10, 1000),
        ]
    }

    fn grid_text(shape: &GridShape, points: &[(&'static str, u64, u64, u64, u64)]) -> String {
        let mut text = String::new();
        for (name, unit_area_slots, unlinks, data_units, files) in points {
            push_line(
                &mut text,
                &format!("name=point point={name} unit_area_slots={unit_area_slots} unlinks={unlinks} data_units={data_units} files={files}"),
            );
            for state in STATES {
                for round in 1..=ROUNDS {
                    let at = format!("point={name} state={state} kind=overwrite round={round}");
                    let steps = if state == "remounted" && shape.remounted_has_no_steps {
                        0
                    } else {
                        2 * elements_per_disk(*unit_area_slots)
                    };
                    let at_the_high_end = *name == "unit_area_2_pow_24";
                    let factor = if at_the_high_end {
                        shape.high_end_factor
                    } else {
                        1
                    };
                    let exclusive = steps * shape.nanos_per_step * factor;
                    let residual = if at_the_high_end {
                        shape.high_end_residual
                    } else {
                        5
                    };
                    push_line(&mut text, &format!("name=row {at} segment=caller row=whole_allocator_copy_before_the_judgement calls=1 steps={steps} scale={steps} exclusive_nanos={exclusive}"));
                    push_line(
                        &mut text,
                        &format!(
                            "name=segment_nanos {at} segment=caller nanos={}",
                            exclusive + residual
                        ),
                    );
                }
            }
        }
        text
    }

    /// 计数产物与计时产物读同一份文字（两种产物的字段不相交，互不干扰）。
    fn report_of_the_grid(
        shape: &GridShape,
        points: &[(&'static str, u64, u64, u64, u64)],
    ) -> String {
        let text = grid_text(shape, points);
        let mut out = String::new();
        full_report(&read_product(&text), Some(&read_product(&text)), &mut out);
        out
    }

    #[test]
    fn failure_clause_one_fires_only_when_all_four_cells_reach_one_millisecond() {
        let all_reach = report_of_the_grid(
            &GridShape {
                nanos_per_step: 1,
                high_end_factor: 1,
                high_end_residual: 5,
                remounted_has_no_steps: false,
            },
            &grid_points(),
        );
        assert!(
            all_reach.contains("name=verdict quantity=Q4 cells=AtLeastOneMillisecond,AtLeastOneMillisecond,AtLeastOneMillisecond,AtLeastOneMillisecond"),
            "{all_reach}"
        );
        assert!(
            all_reach.contains("name=verdict clause=F1 fired=true"),
            "{all_reach}"
        );
        let none_reach = report_of_the_grid(
            &GridShape {
                nanos_per_step: 0,
                high_end_factor: 1,
                high_end_residual: 5,
                remounted_has_no_steps: false,
            },
            &grid_points(),
        );
        assert!(
            none_reach.contains("name=verdict quantity=Q4 cells=BelowOneMillisecond,BelowOneMillisecond,BelowOneMillisecond,BelowOneMillisecond"),
            "{none_reach}"
        );
        assert!(
            none_reach.contains("name=verdict clause=F1 fired=false"),
            "{none_reach}"
        );
        let mixed = report_of_the_grid(
            &GridShape {
                nanos_per_step: 1,
                high_end_factor: 1,
                high_end_residual: 5,
                remounted_has_no_steps: true,
            },
            &grid_points(),
        );
        assert!(
            mixed.contains("name=verdict quantity=Q4 cells=AtLeastOneMillisecond,AtLeastOneMillisecond,BelowOneMillisecond,BelowOneMillisecond"),
            "{mixed}"
        );
        assert!(
            mixed.contains("name=verdict clause=F1 fired=false"),
            "四格里有「< 1 毫秒」又有「≥ 1 毫秒」：几何敏感，F1 不触发（变异：all 改 any 时红）{mixed}"
        );
    }

    #[test]
    fn failure_clause_three_fires_when_the_residual_grows_in_all_five_rounds() {
        let small_residual = report_of_the_grid(
            &GridShape {
                nanos_per_step: 1,
                high_end_factor: 1,
                high_end_residual: 5,
                remounted_has_no_steps: false,
            },
            &grid_points(),
        );
        assert!(
            small_residual.contains("name=verdict clause=F3 fired=false"),
            "{small_residual}"
        );
        let growing_residual = report_of_the_grid(
            &GridShape {
                nanos_per_step: 1,
                high_end_factor: 1,
                high_end_residual: 5_000_000,
                remounted_has_no_steps: false,
            },
            &grid_points(),
        );
        assert!(
            growing_residual.contains("name=verdict clause=F3 fired=true"),
            "{growing_residual}"
        );
    }

    #[test]
    fn failure_clause_six_fires_on_a_cell_the_extrapolation_misses_and_is_unknown_without_cells() {
        let proportional = report_of_the_grid(
            &GridShape {
                nanos_per_step: 1,
                high_end_factor: 1,
                high_end_residual: 5,
                remounted_has_no_steps: false,
            },
            &grid_points(),
        );
        assert!(
            proportional.contains("name=verdict quantity=Q5 cells_checked=4 cells_not_checked=0"),
            "{proportional}"
        );
        assert!(
            proportional.contains("name=verdict clause=F6 fired=false"),
            "{proportional}"
        );
        let high_end_four_times = report_of_the_grid(
            &GridShape {
                nanos_per_step: 1,
                high_end_factor: 4,
                high_end_residual: 5,
                remounted_has_no_steps: false,
            },
            &grid_points(),
        );
        assert!(
            high_end_four_times.contains("name=verdict clause=F6 fired=true"),
            "挂钟比 0.25 出了 [0.5, 2]：外推核不过 {high_end_four_times}"
        );
        let only_the_base = report_of_the_grid(
            &GridShape {
                nanos_per_step: 1,
                high_end_factor: 1,
                high_end_residual: 5,
                remounted_has_no_steps: false,
            },
            &grid_points()[..1],
        );
        assert!(
            only_the_base.contains("name=verdict clause=F6 fired=unknown"),
            "一格都没核：没判，不记核过 {only_the_base}"
        );
    }

    fn state_line(build: &str, units: u64) -> String {
        format!("E203RESULT name=state build={build} point=base state=same_mount kind=overwrite round=1 units={units} mapped_units=5\n")
    }

    #[test]
    fn stop_clause_three_counts_each_state_field_that_differs_between_the_two_builds() {
        let counts = read_product(&state_line("counts", 3000));
        assert_eq!(
            state_fields_that_differ(&counts, &read_product(&state_line("timing", 3000))),
            0,
            "只有 build 字段不同：逐字段相同，不停（变异：比较方向反了时红）"
        );
        assert_eq!(
            state_fields_that_differ(&counts, &read_product(&state_line("timing", 3001))),
            1
        );
        assert_eq!(
            state_fields_that_differ(&counts, &read_product("")),
            1,
            "计时产物里少了这一轮的规模读数：数一格不同，不当相同"
        );
        let mut out = String::new();
        full_report(
            &counts,
            Some(&read_product(&state_line("timing", 3001))),
            &mut out,
        );
        assert!(
            out.contains("name=verdict clause=stop_3 state_fields_that_differ=1"),
            "{out}"
        );
    }

    fn publish_text(had_caller_segment: bool, with_the_row_before_landing: bool) -> String {
        let at = "point=base state=same_mount kind=overwrite round=1";
        let mut text = format!("E203RESULT name=publish {at} had_caller_segment={had_caller_segment} fixed_point_rounds=2\n");
        if with_the_row_before_landing {
            text.push_str(&format!("E203RESULT name=row {at} segment=before_landing row=root_header_linear_find calls=1 steps=1\n"));
        }
        text
    }

    #[test]
    fn stop_clause_four_counts_a_publish_that_skipped_the_caller_or_the_judge() {
        let violations = |had_caller_segment: bool, with_the_row_before_landing: bool| {
            publishes_that_skipped_the_caller_or_the_judge(&read_product(&publish_text(
                had_caller_segment,
                with_the_row_before_landing,
            )))
        };
        assert_eq!(violations(true, true), 0);
        assert_eq!(violations(false, true), 1, "没经调用方进门");
        assert_eq!(violations(true, false), 1, "没进落点判之前那一段");
        assert_eq!(violations(false, false), 1, "两样都缺也只数一次发布");
        let mut out = String::new();
        full_report(&read_product(&publish_text(false, true)), None, &mut out);
        assert!(
            out.contains(
                "name=verdict clause=stop_4 publishes_that_skipped_the_caller_or_the_judge=1"
            ),
            "{out}"
        );
    }

    /// 对账用的一格：规模读数给 other_files = 1000、cluster_segments = 7，单元区 2¹⁷（每盘 2048 段）。
    fn reconciliation_text(plan_steps: u64, cluster_steps: u64, kind: &str) -> String {
        let at = format!("point=base state=same_mount kind={kind} round=1");
        let mut text = String::from("E203RESULT name=point point=base unit_area_slots=131072 unlinks=10 data_units=10 files=10\n");
        push_line(
            &mut text,
            &format!("name=state {at} other_files=1000 cluster_segments=7"),
        );
        for (row, calls, steps) in [
            ("files_other_than_the_first_plan", 2, plan_steps),
            ("inodes_with_an_extent_entry_set", 2, 2002),
            ("file_other_than_the_first_linear_find", 2, 1500),
            ("cluster_segment_sum", 4, cluster_steps),
            ("empty_segment_linear_scan", 4, 100),
        ] {
            push_line(
                &mut text,
                &format!(
                    "name=row {at} segment=before_landing row={row} calls={calls} steps={steps}"
                ),
            );
        }
        text
    }

    #[test]
    fn the_reconciliation_compares_the_rows_it_can_and_names_the_rows_it_cannot() {
        assert_eq!(scan_rows().len(), 28);
        let mut matching = String::new();
        let (cells, disagreements, unread) = reconciliation_report(
            &read_product(&reconciliation_text(2000, 28, "overwrite")),
            &mut matching,
        );
        assert_eq!((cells, disagreements, unread), (5, 0, 0), "{matching}");
        assert!(
            matching.contains("name=verdict quantity=three_two cells_checked=5 disagreements=0 cells_with_the_scale_unread=0 rows_reconciled=5 rows_not_reconciled=21"),
            "{matching}"
        );
        let mut plan_off = String::new();
        let (_, disagreements, _) = reconciliation_report(
            &read_product(&reconciliation_text(2001, 28, "overwrite")),
            &mut plan_off,
        );
        assert_eq!(disagreements, 1, "{plan_off}");
        assert!(
            plan_off.contains("name=three_two_disagreement point=base state=same_mount kind=overwrite round=1 row=files_other_than_the_first_plan relation=Equals steps=2001 formula=2000 calls=2"),
            "{plan_off}"
        );
        let mut over_the_bound = String::new();
        let (_, disagreements, _) = reconciliation_report(
            &read_product(&reconciliation_text(2000, 29, "overwrite")),
            &mut over_the_bound,
        );
        assert_eq!(
            disagreements, 1,
            "上界 4 × 7 = 28，29 步超了：{over_the_bound}"
        );
        let mut last_batch = String::new();
        let (cells, disagreements, _) = reconciliation_report(
            &read_product(&reconciliation_text(9999, 9999, "unlink_final")),
            &mut last_batch,
        );
        assert_eq!(
            (cells, disagreements),
            (1, 0),
            "最后一批的规模读数是整个 unlink 做完的一版，只对不依赖那一版表长的一行对账：{last_batch}"
        );
        let mut unread_scale = String::new();
        let without_scale = reconciliation_text(2000, 28, "overwrite")
            .replace("other_files=1000 cluster_segments=7", "other_files=1000");
        let (_, _, unread) =
            reconciliation_report(&read_product(&without_scale), &mut unread_scale);
        assert_eq!(
            unread, 1,
            "读不到 cluster_segments：记读不到，不拿 0 比 {unread_scale}"
        );
    }

    #[test]
    fn the_cell_that_cannot_be_measured_is_named_and_left_out_of_every_reading() {
        let flat = synthetic_counts_text(&synthetic_unit_area_points(4, 8));
        let full_five = format!(
            "{flat}E203RESULT name=rounds_taken point=unit_area_2_pow_24 state=same_mount taken=5 attempts=5\n"
        );
        let all_measured = report_of_counts(&full_five);
        assert!(
            all_measured.contains("name=verdict clause=void_8 cells_not_measurable_in_counts=0 cells_not_measurable_in_timing=0"),
            "{all_measured}"
        );
        assert!(
            !all_measured.contains("name=b8_input_gap"),
            "{all_measured}"
        );
        let three_of_five = format!(
            "{flat}E203RESULT name=rounds_taken point=unit_area_2_pow_24 state=same_mount taken=3 attempts=10\n"
        );
        let report = report_of_counts(&three_of_five);
        assert!(
            report.contains("name=void_8_cell build=counts point=unit_area_2_pow_24 state=same_mount kind=overwrite taken=3 attempts=10 result=not_measurable"),
            "{report}"
        );
        assert!(
            report.contains("name=verdict clause=void_8 cells_not_measurable_in_counts=1 cells_not_measurable_in_timing=0"),
            "{report}"
        );
        assert!(
            report.contains(
                "name=b8_input_gap point=unit_area_2_pow_24 state=same_mount kind=overwrite"
            ),
            "量不到的格整格拿掉，B8 的输入记读不到：{report}"
        );
        assert!(
            !report.contains("name=b8_input_gap point=unit_area_2_pow_24 state=remounted"),
            "只拿掉量不到的那一种挂载状态：{report}"
        );
        assert!(
            report.contains("name=verdict quantity=Q6 result=SomeInputUnread"),
            "有格读不到不记「不随」：{report}"
        );
    }

    /// 单元区轴上三个点：低端、B0、高端；别的三根轴只有 B0 一个点。
    fn user_scale_points() -> Vec<SyntheticPoint> {
        let mut points = synthetic_unit_area_points(4, 8);
        points.push(SyntheticPoint {
            name: "base",
            unit_area_slots: 1 << 17,
            unlinks: 10,
            data_units: 10,
            files: 10,
            changed_leaves: 4,
            root_header_calls: 8,
        });
        points
    }

    #[test]
    fn the_user_scale_value_is_the_base_plus_each_axis_increment() {
        let counts = read_product(&synthetic_counts_text(&user_scale_points()));
        let total_of_the_point = |point: &str| -> Result<u64, Vec<String>> {
            let rows = by_delta_rows_by_segment(&counts, point, "same_mount", "overwrite")?;
            Ok(total_of(&rows, &SEGMENTS_BEFORE_ADMISSION))
        };
        let at_the_base = total_of_the_point("base").expect("B0 的输入都读得到");
        let at_the_high_end = total_of_the_point("unit_area_2_pow_24").expect("高端的输入都读得到");
        assert_eq!(
            at_the_high_end - at_the_base,
            6,
            "单元区从 2¹⁷ 到 2²⁴：每盘段数 2048 → 262144，分层位图层数 2 → 3，user_data_slot 2 次调用加 2 步、empty_segment 4 次调用加 4 步"
        );
        let (value, missing) = quantity_at_user_scale(
            &counts,
            "same_mount",
            "overwrite",
            &user_scale(268_435_456.0),
            &total_of_the_point,
        )
        .expect("每个点都读得到");
        let slope_over_the_gap =
            6.0 * (1_073_740_416.0 - 16_777_216.0) / (16_777_216.0 - 131_072.0);
        let expected = to_f64(at_the_high_end) + slope_over_the_gap;
        assert!(
            (value - expected).abs() < 1e-6,
            "G-user 上的值 = B0 的值加单元区轴上最后两点斜率外推的增量：{value} 对 {expected}"
        );
        assert_eq!(missing, ["u", "D", "F"], "只有 B0 一个点的轴外推不出，点名");
        let gaps = quantity_at_user_scale(
            &counts,
            "same_mount",
            "overwrite",
            &user_scale(268_435_456.0),
            &|point: &str| -> Result<u64, Vec<String>> {
                if point == "unit_area_2_pow_14" {
                    Err(vec!["state.units".to_string()])
                } else {
                    total_of_the_point(point)
                }
            },
        )
        .expect_err("一个点读不到，整格读不到");
        assert_eq!(gaps, ["unit_area_2_pow_14:state.units"]);
    }

    /// 两道之间那一段一轮的挂钟：段挂钟 1000，没改的行 100，改过的行 200（B1 步数 10、B8 步数 2）。
    #[test]
    fn the_second_segment_nanos_take_the_improved_rows_at_the_per_step_price() {
        let at = "point=base state=same_mount kind=overwrite round=1";
        let timing = read_product(&format!(
            "E203RESULT name=row {at} segment=between row=fallback_slot_linear_scan exclusive_nanos=100\n\
             E203RESULT name=row {at} segment=between row=root_header_linear_find exclusive_nanos=200\n\
             E203RESULT name=segment_nanos {at} segment=between nanos=1000\n"
        ));
        let mut today: StepsByRow = BTreeMap::new();
        let mut by_delta: StepsByRow = BTreeMap::new();
        today.insert(("between", "root_header_linear_find"), 10);
        by_delta.insert(("between", "root_header_linear_find"), 2);
        let called: BTreeSet<&'static str> = ["root_header_linear_find"].into_iter().collect();
        let at_key = round_key("base", "same_mount", "overwrite", 1);
        assert_eq!(
            narrowed_between_nanos(&timing, &at_key, &by_delta, &today, &called),
            Some(700.0 + 100.0 + 40.0),
            "残差 700 + 没改的行 100 + 改过的行 2 步 × 每步 20 纳秒"
        );
        let broken = read_product(&format!(
            "E203RESULT name=row {at} segment=between row=fallback_slot_linear_scan exclusive_nanos=2000\n\
             E203RESULT name=segment_nanos {at} segment=between nanos=1000\n"
        ));
        assert_eq!(
            narrowed_between_nanos(&broken, &at_key, &by_delta, &today, &called),
            None,
            "Σ 独占计时大于段挂钟的一轮不能用"
        );
    }

    fn timing_text_of_the_second_segment(rounds: u64) -> String {
        let mut text = String::new();
        for point in ["base", "unit_area_2_pow_14", "unit_area_2_pow_24"] {
            for round in 1..=rounds {
                let at = format!("point={point} state=same_mount kind=overwrite round={round}");
                push_line(&mut text, &format!("name=row {at} segment=between row=fallback_slot_linear_scan exclusive_nanos=100"));
                push_line(&mut text, &format!("name=row {at} segment=between row=root_header_linear_find exclusive_nanos=200"));
                push_line(
                    &mut text,
                    &format!("name=segment_nanos {at} segment=between nanos=1000"),
                );
            }
        }
        text
    }

    #[test]
    fn the_second_segment_nanos_at_the_user_scale_need_five_valid_rounds() {
        let counts = read_product(&synthetic_counts_text(&user_scale_points()));
        let scale = user_scale(268_435_456.0);
        let five = narrowed_between_nanos_at_user_scale(
            &counts,
            &read_product(&timing_text_of_the_second_segment(5)),
            "same_mount",
            &scale,
        );
        assert_eq!(
            five,
            "nanos_of_each_round=800,800,800,800,800 axes_not_extrapolated=u,D,F"
        );
        let four = narrowed_between_nanos_at_user_scale(
            &counts,
            &read_product(&timing_text_of_the_second_segment(4)),
            "same_mount",
            &scale,
        );
        assert_eq!(four, "fewer_than_five_valid_rounds");
    }
}
