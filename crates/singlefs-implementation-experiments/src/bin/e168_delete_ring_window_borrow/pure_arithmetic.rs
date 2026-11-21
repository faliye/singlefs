//! 纯算术（第七节 B 类：只用格式常量与条款式子，不调 `crates/` 的函数；与 `crates/` 的读数只在停机条款 S3、S4 里逐格比）。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// 纯算术（第七节 B 类：只用格式常量与条款式子，不调 `crates/` 的函数；与 `crates/` 的读数只在停机条款 S3、S4 里逐格比）
// ---------------------------------------------------------------------------------------------------------------------

/// 环长（根环里的根数）= 区数 × 每区槽数（第七节 A3）。
fn ring_length_in_roots(slots_per_region: u64) -> u64 {
    ROOT_RING_REGIONS * slots_per_region
}

/// 分配记录树层级 L 上一个位置罩的槽数 S_L = 812 × 169^L。
fn allocation_record_tree_span_in_slots_at_level(level: u32) -> u64 {
    ALLOCATION_RECORD_TREE_LEAF_SLOTS * ALLOCATION_RECORD_TREE_INTERNAL_FANOUT.pow(level)
}

/// 分配记录树的根层级（`allocation_record_tree.rs` 的 `AllocationRecordTreeGeometry::of_devices` 的规则）：最小的 R 使 Σ_盘 ⌈盘绝对槽数 ÷ S_(R−1)⌉ ≤ 169。
fn allocation_record_tree_root_level(absolute_slots_of_each_device: &[u64]) -> u32 {
    let mut root_level = 1;
    loop {
        let span_below_the_root = allocation_record_tree_span_in_slots_at_level(root_level - 1);
        let positions_under_the_root: u64 = absolute_slots_of_each_device
            .iter()
            .map(|absolute_slots| absolute_slots.div_ceil(span_below_the_root))
            .sum();
        if positions_under_the_root <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT {
            return root_level;
        }
        root_level += 1;
    }
}

/// 一块盘的单元区 [起点, 盘末)，按绝对槽号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UnitAreaOnOneDevice {
    pub(super) start_slot: u64,
    pub(super) end_slot_exclusive: u64,
}

/// D28 已定项 4 的 K1：1 + Σ_盘 Σ_{L = 0 … 根层级 − 1}（⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1）。交回 (根层级, K1)。
pub(super) fn allocation_record_tree_positions_by_the_clause(
    unit_areas: &[UnitAreaOnOneDevice],
) -> (u32, u64) {
    let absolute_slots: Vec<u64> = unit_areas
        .iter()
        .map(|area| area.end_slot_exclusive)
        .collect();
    let root_level = allocation_record_tree_root_level(&absolute_slots);
    let positions_below_the_root: u64 = unit_areas
        .iter()
        .map(|area| {
            (0..root_level)
                .map(|level| {
                    let span = allocation_record_tree_span_in_slots_at_level(level);
                    (area.end_slot_exclusive - 1) / span - area.start_slot / span + 1
                })
                .sum::<u64>()
        })
        .sum();
    (root_level, 1 + positions_below_the_root)
}

/// 单元区起点 = journal 环起点 + ⌈环长 ÷ 16384⌉（`journal.rs` 的 `slot_after_the_journal_ring`）。
pub(super) fn unit_area_start_slot(journal_ring_bytes: u64) -> u64 {
    JOURNAL_RING_START_SLOT + journal_ring_bytes.div_ceil(SLOT_BYTES)
}

/// 一次发布的 journal 记录数上限 = ⌊环长 ÷ 4096 ÷ 3⌋（`system_configuration.rs` 的 `journal_in_flight_record_limit`，拒在
/// `transaction.rs` 的 `refuse_a_publish_of_more_journal_records_than_the_limit`）；本地按格式常量算、不调 core，单测与 core 回比。
pub(super) fn journal_record_limit_by_the_format(journal_ring_bytes: u64) -> u64 {
    journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR
}

/// 登记第十二节修订第 15 条：装得下一次发布 `records_of_the_publish` 条 journal 记录的最小合法环。合法 = mkfs 收：环长按
/// [`JOURNAL_RING_STEP_BYTES`]（1 MiB）一档，取在飞上限 ≥ 记录数的最小一档、档的上端；不小于 [`SMALL_JOURNAL_RING_BYTES`]。
pub(super) fn smallest_legal_journal_ring_holding(records_of_the_publish: u64) -> u64 {
    let bytes_holding_the_records =
        records_of_the_publish * JOURNAL_RECORD_BYTES * JOURNAL_SAFETY_FACTOR;
    let steps = bytes_holding_the_records
        .div_ceil(JOURNAL_RING_STEP_BYTES)
        .max(SMALL_JOURNAL_RING_BYTES / JOURNAL_RING_STEP_BYTES);
    steps * JOURNAL_RING_STEP_BYTES
}

/// mkfs 允许的最小盘（第七节 B3）：单元区要装得下 mkfs 写的实例表（2 槽）与树表（1 槽），且环 ≤ 盘 ÷ 4。
pub(super) fn make_filesystem_smallest_device_bytes(journal_ring_bytes: u64) -> u64 {
    let unit_area_holds_the_make_filesystem_units =
        (unit_area_start_slot(journal_ring_bytes) + 3) * SLOT_BYTES;
    unit_area_holds_the_make_filesystem_units.max(4 * journal_ring_bytes)
}

/// 单元区恰好 `unit_area_slots` 槽的盘的字节数。
pub(super) fn device_bytes_with_unit_area(journal_ring_bytes: u64, unit_area_slots: u64) -> u64 {
    (unit_area_start_slot(journal_ring_bytes) + unit_area_slots) * SLOT_BYTES
}

/// 第七节 A2：发布路径上实例表链一片装得下时，每盘保留量 = (N_switch + 1) × (一片 2 槽 + R × c) + c（c = ckpt_cost，
/// ckpt 池 c × 16384 × 副本数按全部副本之和记、摊到每块盘 c 槽）= 8 + 13c。
pub(super) fn reserve_slots_by_the_clauses(checkpoint_cost: u64) -> u64 {
    let one_switch =
        INSTANCE_TABLE_PAGE_SLOTS + WARM_UP_EMPTY_PUBLISHES_PER_SWITCH * checkpoint_cost;
    (INSTANCE_SWITCHES_PER_MOUNT + 1) * one_switch + checkpoint_cost
}

/// 5.3 压满：把式子可用压进 (−16384, 0] 要再标掉的槽数。
pub(super) fn slots_to_mark_so_available_is_within_one_slot_at_or_below_zero(
    available_bytes: i128,
) -> u64 {
    if available_bytes <= 0 {
        return 0;
    }
    let slot_bytes = i128::from(SLOT_BYTES);
    u64::try_from((available_bytes + slot_bytes - 1) / slot_bytes).expect("可用的槽数装得进 u64")
}

/// 压满之后的区间断言 (−16384, 0]（登记 1.2「满池」）。
pub(super) fn available_is_within_one_slot_at_or_below_zero(available_bytes: i128) -> bool {
    available_bytes <= 0 && available_bytes > -i128::from(SLOT_BYTES)
}

/// M1：顶点 − 顶点处的保留量 ≤ 0 判「罩得住（顶点）」。
pub(super) fn reserve_covers_the_apex(apex_slots: i128, reserve_at_the_apex_slots: i128) -> bool {
    apex_slots - reserve_at_the_apex_slots <= 0
}

/// M7：(环长 + 1) × W_max。
pub(super) fn ring_window_bound_slots(
    slots_per_region: u64,
    largest_new_slots_of_a_delete_publish: u64,
) -> u64 {
    (ring_length_in_roots(slots_per_region) + 1) * largest_new_slots_of_a_delete_publish
}

/// 8.3：使 apex + (3S + 1) × δ > reserve_apex 的最小 δ（只算 δ ≤ 3）；罩不住的格（δ = 0 就越过）交回 `Some(0)`。
pub(super) fn smallest_unbuilt_delta_that_flips_the_cover(
    apex_slots: i128,
    reserve_at_the_apex_slots: i128,
    slots_per_region: u64,
) -> Option<u64> {
    let roots_plus_one = i128::from(ring_length_in_roots(slots_per_region) + 1);
    (0..=DELTA_AT_MOST)
        .find(|delta| apex_slots + roots_plus_one * i128::from(*delta) > reserve_at_the_apex_slots)
}

/// M3 的判定。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ApexSaturation {
    /// 批数 ≥ 3S + 2 的梯子点里，最后两点的 apex ≤ 它们之前那一点的 apex。
    Saturated,
    /// 有至少三个那样的点，最后两点里有一点比之前那一点高。
    GrowingWithoutSaturation,
    /// 那样的点不到三个，判不了。
    FewerThanThreePointsPastTheRing,
}

/// M3：`apex_by_delete_publishes` 按梯子次序给 (这条删除的发布次数, apex)。
pub(super) fn apex_saturation(
    apex_by_delete_publishes: &[(u64, i128)],
    slots_per_region: u64,
) -> ApexSaturation {
    let past_the_ring: Vec<i128> = apex_by_delete_publishes
        .iter()
        .filter(|(publishes, _)| *publishes >= ring_length_in_roots(slots_per_region) + 2)
        .map(|(_, apex)| *apex)
        .collect();
    let [.., before_the_last_two, second_last, last] = past_the_ring.as_slice() else {
        return ApexSaturation::FewerThanThreePointsPastTheRing;
    };
    if *second_last <= *before_the_last_two && *last <= *before_the_last_two {
        ApexSaturation::Saturated
    } else {
        ApexSaturation::GrowingWithoutSaturation
    }
}

/// M5：W_max(N) 在梯子最后三点逐点严格增 ⇒ 判「随文件大小涨」。
pub(super) fn largest_new_slots_grow_with_file_size(
    largest_new_slots_by_ladder_point: &[u64],
) -> bool {
    matches!(largest_new_slots_by_ladder_point, [.., first, second, third] if first < second && second < third)
}

/// 删除的发布次数 = 1（意图那次）+ ⌈N ÷ K⌉。
pub(super) fn delete_publish_count(data_units: u64, batch_data_units: u64) -> u64 {
    1 + data_units.div_ceil(batch_data_units)
}

/// 8.4 文件大小梯子（数据单元数 N），从小到大、去重。K = 144：N = 144 × b；K = 1：N 取同一组 b 再加 8 × 3S。
pub(super) fn ladder_of_file_sizes(batch_data_units: u64, slots_per_region: u64) -> Vec<u64> {
    let ring = ring_length_in_roots(slots_per_region);
    let mut batches_per_file = vec![
        1,
        2,
        4,
        8,
        ring - 1,
        ring,
        ring + 1,
        ring + 2,
        ring + 5,
        2 * ring + 2,
        4 * ring,
    ];
    if batch_data_units == SMALLEST_BATCH_DATA_UNITS {
        batches_per_file.push(8 * ring);
    }
    let mut data_units: Vec<u64> = batches_per_file
        .into_iter()
        .map(|batches| batches * batch_data_units)
        .collect();
    data_units.sort_unstable();
    data_units.dedup();
    data_units
}

#[cfg(test)]
mod tests;
