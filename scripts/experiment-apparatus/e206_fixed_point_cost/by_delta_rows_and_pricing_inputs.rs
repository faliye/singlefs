//! 逐项按差量的输入与行、轮间计价的输入与单价、产物字段的读取。

use super::*;

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

/// 这几行每轮一份，P3 把它们的输入换成轮间差（第五节 P3 式子）；别的行 P1、P2a、P3 同一份。
pub(super) const ROWS_THAT_BETWEEN_ROUNDS_PRICING_CHANGES: [&str; 14] = [
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

// ---- P3（第五节「P3 的式子」）----
/// 一轮喂给 P3 式子的输入。`P3Input::between_rounds` 取轮间差，阳性对照 `P3Input::cumulative` 照 B8 取相对上一版的累计，同一个式子。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct BetweenRoundsPricingInput {
    pub(super) changed_leaves: u64,
    pub(super) code_two_keys: Vec<(u64, u64)>,
    pub(super) containers: u64,
    pub(super) mapped_changed: u64,
    pub(super) records_in_the_view: u64,
    pub(super) release_finds: u64,
    pub(super) intersection_finds: u64,
    pub(super) intersecting_records: u64,
    pub(super) dirty_records: u64,
    pub(super) query_levels: u64,
    pub(super) user_data_finds: u64,
    pub(super) empty_segment_finds: u64,
    pub(super) fallback_finds: u64,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct BetweenRoundsUnitPrices {
    pub(super) allocation_tree_levels: u64,
    pub(super) allocation_records: u64,
    pub(super) mapped_units: u64,
    pub(super) segments_per_disk: u64,
}

/// 一轮的步：s15、s16 两行，s20、s21，s36，s24、s25、s27、s30，s19、s23（E203 B8 那几行的单价）。
pub(super) fn between_rounds_pricing_round_steps(input: &BetweenRoundsPricingInput, prices: BetweenRoundsUnitPrices) -> u64 {
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

pub(super) fn numbers(text: &str) -> Vec<u64> {
    text.split(',')
        .filter_map(|value| value.parse().ok())
        .collect()
}
