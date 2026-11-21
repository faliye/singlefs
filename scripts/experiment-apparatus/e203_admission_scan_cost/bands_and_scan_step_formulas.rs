//! 判档（增长阶三档）、小算术、逐项按差量的步数式子与差量为零的行。

use super::*;

// ---- 判档（登记第六节）----
/// 增长阶三档：α ≥ 0.5 随这根轴涨，α ≤ 0.25 不随（至多对数级），之间不判。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Band {
    GrowsWithTheAxis,
    DoesNotGrow,
    Between,
}

pub(super) fn alpha(low_quantity: f64, high_quantity: f64, low_position: f64, high_position: f64) -> f64 {
    (high_quantity / low_quantity).ln() / (high_position / low_position).ln()
}

pub(super) fn band(alpha: f64) -> Band {
    if alpha >= 0.5 {
        Band::GrowsWithTheAxis
    } else if alpha <= 0.25 {
        Band::DoesNotGrow
    } else {
        Band::Between
    }
}

/// 五轮每轮一个 α，五轮落在同一档才记那一档，否则 `None`（不稳定）。
pub(super) fn band_of_the_rounds(alphas: &[f64]) -> Option<Band> {
    let first = band(*alphas.first()?);
    alphas
        .iter()
        .all(|growth| band(*growth) == first)
        .then_some(first)
}

// ---- 整数函数（第七节 ③5）----
/// ⌈log₂(n + 1)⌉，lg(0) = 0。
pub(super) fn lg(value: u64) -> u64 {
    u64::from(u64::BITS - value.leading_zeros())
}

/// 每层 64 叉的分层位图要几层：最小的 l ≥ 1 使 64ˡ ≥ n。
pub(super) fn levels_64(value: u64) -> u64 {
    let mut level = 1_u32;
    while 64_u64.pow(level) < value {
        level += 1;
    }
    u64::from(level)
}

/// 分层位图各层字数之和，到只剩 1 字为止。
pub(super) fn summary_words(value: u64) -> u64 {
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

pub(super) fn segments_of(unit_area_slots: u64) -> u64 {
    unit_area_slots.div_ceil(SEGMENT_SLOTS)
}

/// 一块盘那八张表的元素数：三张 U 位、五张按段表各 ⌈U/64⌉。
pub(super) fn elements_per_disk(unit_area_slots: u64) -> u64 {
    3 * unit_area_slots + 5 * segments_of(unit_area_slots)
}

/// 两块盘的整份拷贝里只由 U 定的那一部分的元素数（第七节 ③1）。
pub(super) fn allocator_copy_elements_from_the_unit_area(unit_area_slots: u64) -> u64 {
    DISKS * elements_per_disk(unit_area_slots)
}

pub(super) fn bytes_per_disk(unit_area_slots: u64) -> u64 {
    3 * unit_area_slots + 40 * segments_of(unit_area_slots)
}

pub(super) fn device_bytes(unit_area_slots: u64) -> u64 {
    (UNIT_AREA_START_SLOT + unit_area_slots) * SLOT_BYTES
}

/// 用户关心的规模：每盘 16 TiB 的单元区槽数。
pub(super) fn user_scale_unit_area_slots() -> u64 {
    USER_SCALE_DEVICE_SLOTS - UNIT_AREA_START_SLOT
}

fn tombstone_containers(unlinks: u64) -> u64 {
    unlinks.div_ceil(TOMBSTONE_CONTAINER_RECORDS)
}

// ---- B8（逐项按差量）：登记第五节「B8 每一行的做法与步数式子」----
/// B8 式子吃的输入：这一次发布的差量与调用次数（量的那一路读到的）。字段名与登记第五节、第七节 ③3 一致。
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
    /// 每轮内容变了的叶数 λ_ar（一轮一项）。
    pub(super) changed_leaves_per_round: Vec<u64>,
    pub(super) allocation_tree_levels: u64,
    pub(super) allocation_records: u64,
    pub(super) segments_per_disk: u64,
    pub(super) units: u64,
    pub(super) mapped_units: u64,
    pub(super) deadlist_entries: u64,
    /// 各行在这一段里被调了几次（`c4`、`c5`……）。
    pub(super) calls: BTreeMap<&'static str, u64>,
    pub(super) intersecting_records: u64,
    pub(super) dirty_per_call: Vec<u64>,
    /// 全部轮里每棵多层码 2 树进出的 key 数与树高：(δ, h)，一棵树一轮一项。
    pub(super) code_two_trees_over_all_rounds: Vec<(u64, u64)>,
    /// 全部轮里三棵容器树重写与新开的容器、节点数，一棵树一轮一项。
    pub(super) container_rewrites_over_all_rounds: Vec<u64>,
    pub(super) moved_entries: u64,
    pub(super) central_mapping_levels: u64,
    /// 按身份建的索引上 deadlist 判生死那一圈的查询次数乘各自的 ⌈log₂(索引项数 + 1)⌉ 之和。
    pub(super) deadlist_query_levels: u64,
    pub(super) deadlist_changed_entries: u64,
    /// 释放判定那一圈的同一个和。
    pub(super) release_query_levels: u64,
    pub(super) changed_slots_and_records_per_round: Vec<u64>,
    /// 全部轮里进出 `mapped_units` 的条目数之和。
    pub(super) mapped_units_changed_over_all_rounds: u64,
    /// s36 按差量要问的角色（全部轮里进出 `mapped_units` 的角色数）各乘 ⌈log₂(M + 1)⌉ 之和。
    pub(super) mapped_units_query_levels: u64,
}

impl ByDeltaInput {
    fn calls_in_the_row(&self, row: &str) -> u64 {
        self.calls.get(row).copied().unwrap_or(0)
    }
}

/// B8 每一行一次发布准入之前的步数，次序同登记第五节。
pub(super) fn by_delta_rows(input: &ByDeltaInput) -> Vec<(&'static str, u64)> {
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

pub(super) fn by_delta_total(input: &ByDeltaInput) -> u64 {
    by_delta_rows(input).iter().map(|(_, steps)| steps).sum()
}

/// 登记第七节 ③3 的合成输入 Δ0。
pub(super) fn delta_zero() -> ByDeltaInput {
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

#[cfg(test)]
#[path = "bands_and_scan_step_formulas_tests.rs"]
mod tests;
