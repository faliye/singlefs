//! 常驻内存与每次发布的维护、撤销日志、按段收窄的步数、阳性与阴性对照与浮点转换。

use super::*;

// ---- B8 的常驻内存与每次发布的维护（登记第五节两段）----
#[derive(Clone, Debug)]
pub(super) struct MemoryInput {
    pub(super) allocation_records: u64,
    pub(super) segments_per_disk: u64,
    pub(super) units: u64,
    pub(super) mapped_units: u64,
    /// 按号索引里的文件数（别的文件加第一个文件，登记第七节 ③4 取 1001）。
    pub(super) files_in_the_index: u64,
    pub(super) extent_upper_nodes: u64,
    pub(super) deadlist_entries: u64,
    pub(super) accounting_rows_of_the_generation: u64,
    pub(super) unit_bytes: u64,
    pub(super) key_bytes: u64,
    pub(super) accounting_row_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MemoryReport {
    pub(super) record_index: u64,
    pub(super) segment_summaries: u64,
    pub(super) units_by_role: u64,
    pub(super) mapped_units_by_role: u64,
    files_by_inode: u64,
    inodes_with_an_extent_entry: u64,
    pub(super) extent_upper_nodes: u64,
    pub(super) deadlist_entries: u64,
    root_head_positions: u64,
    pub(super) accounting_rows_of_the_generation: u64,
}

impl MemoryReport {
    pub(super) fn payload(&self) -> u64 {
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
pub(super) fn by_delta_memory(memory_input: &MemoryInput) -> MemoryReport {
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
pub(super) struct MaintenanceInput {
    pub(super) records_changed: u64,
    pub(super) segments_flipped: u64,
    pub(super) units_delta: u64,
    pub(super) mapped_delta: u64,
    pub(super) files_delta: u64,
    pub(super) upper_delta: u64,
    pub(super) deadlist_delta: u64,
}

/// 每次发布装单元那一步的维护步数（在准入之后）。
pub(super) fn by_delta_maintenance(memory_input: &MemoryInput, maintenance: &MaintenanceInput) -> u64 {
    maintenance.records_changed * lg(memory_input.allocation_records)
        + maintenance.segments_flipped * levels_64(memory_input.segments_per_disk)
        + maintenance.units_delta * lg(memory_input.units)
        + maintenance.mapped_delta * lg(memory_input.mapped_units)
        + maintenance.files_delta * lg(memory_input.files_in_the_index)
        + maintenance.upper_delta * lg(memory_input.extent_upper_nodes)
        + maintenance.deadlist_delta * lg(memory_input.deadlist_entries)
        + memory_input.accounting_rows_of_the_generation
}

pub(super) fn undo_log_bytes(changed_allocation_records: u64) -> u64 {
    changed_allocation_records * 24
}

// ---- B4（改射程）：登记第五节「B4 的式子」----
/// 一行在一段里的步数：键是 (段标记, 行号)。
pub(super) type StepsByRow = BTreeMap<(&'static str, &'static str), u64>;

/// B4 在 P0、S1、S2 三段里的步数。P0、S1 两段每行取 B8 一列；S2 里在 P0 或 S1 里调到过的行也取 B8 一列（改的是函数，不是调用点），
/// 其余 S2 行取 B1 量的步数。
pub(super) fn narrowed_steps_by_segment(
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
pub(super) fn degraded_by_delta_total(
    by_delta_other_rows: u64,
    today_whole_copy_before: u64,
    today_whole_copy_in_a_round: u64,
) -> u64 {
    let retired_round_copy = today_whole_copy_in_a_round;
    by_delta_other_rows + today_whole_copy_before + retired_round_copy
}

/// 阳性对照判过：退化后合计的 α(U) 不小于门槛（B8 取 0.9）。
pub(super) fn positive_control_passes(
    total_at_the_low_end: u64,
    total_at_the_high_end: u64,
    threshold: f64,
) -> bool {
    let low = to_f64(total_at_the_low_end);
    let high = to_f64(total_at_the_high_end);
    alpha(low, high, to_f64(1 << 14), to_f64(1 << 24)) >= threshold
}

/// 阴性对照：s33、s12 在 U 轴两端 |α| ≤ 0.1。
pub(super) fn negative_control_passes(steps_at_the_low_end: u64, steps_at_the_high_end: u64) -> bool {
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
pub(super) fn to_f64(value: u64) -> f64 {
    value as f64
}

#[cfg(test)]
#[path = "memory_and_maintenance_models_tests.rs"]
mod tests;
