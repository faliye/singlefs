//! 本地常量与锚点逐个回比 crates 里的定义与真盘上算出来的值（停机条款 S4）。
use super::*;

/// 停机条款 S4：本地常量与 `crates/` 同名常量逐个回比，一行一个（全等才往下跑）。
pub(super) fn check_local_constants_against_the_crates() -> bool {
    let comparisons: [(&str, u64, u64); 16] = [
        ("slot_bytes", SLOT_BYTES_LOCAL, singlefs_format::SLOT_BYTES),
        (
            "data_unit_bytes",
            DATA_UNIT_BYTES_LOCAL,
            singlefs_format::DATA_UNIT_BYTES,
        ),
        (
            "data_unit_payload_bytes",
            DATA_UNIT_PAYLOAD_BYTES_LOCAL,
            u64::try_from(singlefs_core::unit::data_unit_payload_capacity())
                .expect("载荷字节数装得进 u64"),
        ),
        (
            "allocation_record_tree_leaf_slots",
            ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL,
            singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        ),
        (
            "allocation_record_tree_internal_fanout",
            ALLOCATION_RECORD_TREE_INTERNAL_FANOUT_LOCAL,
            singlefs_format::ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
        ),
        (
            "root_ring_regions",
            ROOT_RING_REGIONS_LOCAL,
            singlefs_format::ROOT_RING_REGIONS,
        ),
        (
            "root_ring_slots_per_region_minimum",
            ROOT_RING_SLOTS_PER_REGION_MINIMUM_LOCAL,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_MINIMUM,
        ),
        (
            "root_ring_slots_per_region_maximum",
            ROOT_RING_SLOTS_PER_REGION_MAXIMUM_LOCAL,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
        ),
        (
            "default_journal_ring_bytes",
            DEFAULT_JOURNAL_RING_BYTES_LOCAL,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        ),
        (
            "unit_area_start_slot_of_the_default_ring",
            UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING_LOCAL,
            singlefs_format::UNIT_AREA_START_SLOT,
        ),
        (
            "publishes_per_admission_at_most",
            PUBLISHES_PER_ADMISSION_AT_MOST_LOCAL,
            u64::try_from(singlefs_core::mount::PUBLISHES_PER_ADMISSION_AT_MOST)
                .expect("次数装得进 u64"),
        ),
        (
            "lineage_container_blocks_of_one_cow",
            LINEAGE_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL,
            singlefs_core::admission::LINEAGE_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE,
        ),
        (
            "lineage_tree_root_blocks_of_one_cow",
            LINEAGE_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL,
            singlefs_core::admission::LINEAGE_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE,
        ),
        (
            "retirement_container_blocks_of_one_cow",
            RETIREMENT_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL,
            singlefs_core::admission::RETIREMENT_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE,
        ),
        (
            "retirement_tree_root_blocks_of_one_cow",
            RETIREMENT_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL,
            singlefs_core::admission::RETIREMENT_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE,
        ),
        (
            "unit_area_start_slot_of_the_default_ring_from_the_ring_start",
            JOURNAL_RING_START_SLOT_LOCAL + DEFAULT_JOURNAL_RING_BYTES_LOCAL / SLOT_BYTES_LOCAL,
            singlefs_format::UNIT_AREA_START_SLOT,
        ),
    ];
    let mut every_constant_equal = true;
    for (name, local, crates) in comparisons {
        let equal = local == crates;
        every_constant_equal &= equal;
        emit(&format!(
            "name=constant item={name} local={local} crates={crates} equal={equal}"
        ));
    }
    every_constant_equal
}

/// 乙类锚点里装置自己算得出的几项（登记第七节 B8 之外：B10 的 529 与 5、A2 的条款读法保留池、B11 钉 `reserve_fit.py` 的两组输出），
/// 一行一个；不等即作废（第十一节 V3）。
pub(super) fn check_the_anchors() -> bool {
    let narrow_unit_area = |width: u64| {
        (
            UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING_LOCAL,
            UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING_LOCAL + width,
        )
    };
    let four_gibibyte_unit_area = (
        UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING_LOCAL,
        4 * GIBIBYTE / SLOT_BYTES_LOCAL,
    );
    let reserve_fit_with_five = reserve_fit(5, &[5]).expect("最大的空发布 5 槽");
    let reserve_fit_with_529 = reserve_fit(529, &[5]).expect("最大的空发布 5 槽");
    let anchors: [(&str, u64, u64); 12] = [
        (
            "b10_k1_two_four_gibibyte_root_level_two",
            local_allocation_record_tree_positions(
                2,
                &[four_gibibyte_unit_area, four_gibibyte_unit_area],
            ),
            529,
        ),
        (
            "b10_k1_two_narrow_240_root_level_one",
            local_allocation_record_tree_positions(
                1,
                &[narrow_unit_area(240), narrow_unit_area(240)],
            ),
            5,
        ),
        (
            "b10_k1_two_narrow_256_root_level_one",
            local_allocation_record_tree_positions(
                1,
                &[narrow_unit_area(256), narrow_unit_area(256)],
            ),
            5,
        ),
        (
            "b10_k1_two_narrow_384_root_level_one",
            local_allocation_record_tree_positions(
                1,
                &[narrow_unit_area(384), narrow_unit_area(384)],
            ),
            5,
        ),
        (
            "a2_clause_reserve_at_kappa_five",
            clause_reading_reserve(5),
            45,
        ),
        (
            "a2_clause_reserve_at_kappa_529",
            clause_reading_reserve(529),
            3713,
        ),
        (
            "b11_code_empties_held_c5_kappa5",
            reserve_fit_with_five.code_empty_publishes_held,
            1,
        ),
        (
            "b11_clause_empties_held_c5_kappa5",
            reserve_fit_with_five.clause_empty_publishes_held,
            9,
        ),
        (
            "b11_code_empties_held_c5_kappa529",
            reserve_fit_with_529.code_empty_publishes_held,
            105,
        ),
        (
            "b11_clause_empties_held_c5_kappa529",
            reserve_fit_with_529.clause_empty_publishes_held,
            742,
        ),
        (
            "a1_publishes_per_admission_at_most",
            PUBLISHES_PER_ADMISSION_AT_MOST_LOCAL,
            8,
        ),
        (
            "local_root_level_of_two_four_gibibyte_devices",
            u64::from(local_root_level_of_the_pool_geometry(&[
                4 * GIBIBYTE / SLOT_BYTES_LOCAL,
                4 * GIBIBYTE / SLOT_BYTES_LOCAL,
            ])),
            2,
        ),
    ];
    let mut every_anchor_equal = true;
    for (name, value, expected) in anchors {
        let equal = value == expected;
        every_anchor_equal &= equal;
        emit(&format!(
            "name=anchor item={name} value={value} expected={expected} equal={equal}"
        ));
    }
    every_anchor_equal
}

#[cfg(test)]
mod tests;
