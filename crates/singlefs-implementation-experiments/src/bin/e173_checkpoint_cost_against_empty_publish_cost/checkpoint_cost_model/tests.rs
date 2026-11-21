//! `checkpoint_cost_model` 的单测。
use super::*;

#[test]
fn allocation_record_tree_positions_are_529_on_two_four_gibibyte_unit_areas_and_5_on_the_narrow_ones(
) {
    let four_gibibytes = (50176, 262_144);
    assert_eq!(
        local_allocation_record_tree_positions(2, &[four_gibibytes, four_gibibytes]),
        529,
        "B10：两块 4 GiB 盘、单元区 [50176, 262144)、根层级 2 是 529（k1_positions.py 的 two 4GiB 那一行）"
    );
    for width in [240, 256, 384] {
        let narrow = (50176, 50176 + width);
        assert_eq!(
            local_allocation_record_tree_positions(1, &[narrow, narrow]),
            5,
            "B10：单元区 {width} 槽的窄盘、根层级 1 是 5"
        );
    }
}

#[test]
fn allocation_record_tree_positions_are_341_on_two_unit_areas_from_slot_zero_to_137001_at_root_level_two(
) {
    let from_slot_zero = (0, 137_001);
    assert_eq!(
        local_allocation_record_tree_positions(2, &[from_slot_zero, from_slot_zero]),
        341,
        "两块盘 [0, 137001)、根层级 2：层级 1 的跨度 812 × 169 = 137228 只罩一格（扇出取 168 时跨度 136416、罩两格，变成 343）"
    );
}

#[test]
fn the_local_root_level_is_two_on_four_gibibyte_devices_and_one_on_one_gibibyte_devices() {
    let slots_of = |device_bytes: u64| device_bytes / SLOT_BYTES_LOCAL;
    assert_eq!(
        local_root_level_of_the_pool_geometry(&[slots_of(4 * GIBIBYTE), slots_of(4 * GIBIBYTE)]),
        2
    );
    assert_eq!(
        local_root_level_of_the_pool_geometry(&[slots_of(GIBIBYTE), slots_of(GIBIBYTE)]),
        1,
        "1 GiB 盘 65536 槽，每块盘 ⌈65536 ÷ 812⌉ = 81 格，两块 162 ≤ 169"
    );
}
