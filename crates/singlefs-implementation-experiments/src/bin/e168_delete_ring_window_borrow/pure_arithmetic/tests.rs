//! 纯算术（第七节 B 类：只用格式常量与条款式子，不调 `crates/` 的函数；与 `crates/` 的读数只在停机条款 S3、S4 里逐格比）的单测。

use super::*;

#[test]
fn ring_length_is_three_regions_times_the_slots_per_region() {
    assert_eq!(ring_length_in_roots(4), 12, "S = 4 时环长 12（第七节 A3）");
    assert_eq!(ring_length_in_roots(8), 24, "S = 8 时环长 24");
    assert_eq!(ring_length_in_roots(16), 48, "S = 16 时环长 48");
}

#[test]
fn allocation_record_tree_root_level_steps_up_right_past_the_registered_boundaries() {
    assert_eq!(
        allocation_record_tree_root_level(&[68208, 68208]),
        1,
        "两块盘各 68208 槽：2 × 84 = 168 ≤ 169，根层级 1（第七节 B2）"
    );
    assert_eq!(
        allocation_record_tree_root_level(&[68209, 68209]),
        2,
        "各 68209 槽：2 × 85 = 170 > 169，根层级 2"
    );
    assert_eq!(
        allocation_record_tree_root_level(&[11_527_152, 11_527_152]),
        2,
        "各 11527152 槽：根层级 2"
    );
    assert_eq!(
        allocation_record_tree_root_level(&[11_527_153, 11_527_153]),
        3,
        "各 11527153 槽：根层级 3"
    );
}

#[test]
fn allocation_record_tree_positions_match_the_registered_anchors() {
    let two_devices = |start_slot: u64, end_slot_exclusive: u64| {
        vec![
            UnitAreaOnOneDevice {
                start_slot,
                end_slot_exclusive
            };
            2
        ]
    };
    assert_eq!(
        allocation_record_tree_positions_by_the_clause(&two_devices(50176, 262_144)),
        (2, 529),
        "4 GiB 默认环（D28 已定项 4、第七节 A1 / B1）"
    );
    assert_eq!(
        allocation_record_tree_positions_by_the_clause(&two_devices(1408, 1408 + 240)),
        (1, 5),
        "单元区 240 槽、环 6 MiB"
    );
    assert_eq!(
        allocation_record_tree_positions_by_the_clause(&two_devices(1408, 1408 + 384)),
        (1, 5),
        "单元区 384 槽、环 6 MiB（P2）"
    );
    assert_eq!(
        allocation_record_tree_positions_by_the_clause(&two_devices(1088, 65536)),
        (1, 161),
        "1 GiB、环 1 MiB"
    );
    assert_eq!(
        allocation_record_tree_positions_by_the_clause(&two_devices(50176, 16_777_216)),
        (3, 41451),
        "256 GiB 默认环"
    );
}

#[test]
fn make_filesystem_smallest_device_matches_the_registered_anchors() {
    assert_eq!(
        make_filesystem_smallest_device_bytes(1 << 20),
        17_874_944,
        "环 1 MiB：(1088 + 3) × 16384（第七节 B3）"
    );
    assert_eq!(
        make_filesystem_smallest_device_bytes(1_032_193),
        17_874_944,
        "环 1032193 字节的单元区起点同为槽 1088"
    );
    assert_eq!(
        make_filesystem_smallest_device_bytes(805_306_368),
        3_221_225_472,
        "默认环：环 ≤ 盘 ÷ 4 先卡住"
    );
    assert_eq!(
        unit_area_start_slot(805_306_368),
        50176,
        "默认环的单元区起点"
    );
    assert_eq!(
        device_bytes_with_unit_area(6 << 20, 384),
        (1408 + 384) * 16384,
        "P2：环 6 MiB、单元区 384 槽"
    );
}

#[test]
fn reserve_by_the_clauses_is_eight_plus_thirteen_checkpoint_costs() {
    for checkpoint_cost in [1, 9, 161, 529] {
        assert_eq!(
            reserve_slots_by_the_clauses(checkpoint_cost),
            8 + 13 * checkpoint_cost,
            "c = {checkpoint_cost} 时每盘保留量 8 + 13c（第七节 A2）"
        );
    }
}

#[test]
fn filling_lands_the_available_bytes_within_one_slot_at_or_below_zero() {
    assert!(
        available_is_within_one_slot_at_or_below_zero(0),
        "0 在区间里"
    );
    assert!(
        available_is_within_one_slot_at_or_below_zero(-16383),
        "−16383 在区间里"
    );
    assert!(
        !available_is_within_one_slot_at_or_below_zero(-16384),
        "−16384 不在区间里"
    );
    assert!(
        !available_is_within_one_slot_at_or_below_zero(1),
        "1 不在区间里"
    );
    assert_eq!(
        slots_to_mark_so_available_is_within_one_slot_at_or_below_zero(-5),
        0,
        "已经 ≤ 0 不标"
    );
    for available in [1i128, 16383, 16384, 16385, 3 * 16384 + 7] {
        let marked =
            i128::from(slots_to_mark_so_available_is_within_one_slot_at_or_below_zero(available));
        assert!(
            available_is_within_one_slot_at_or_below_zero(available - marked * 16384),
            "可用 {available} 标 {marked} 槽之后落进 (−16384, 0]"
        );
    }
}

#[test]
fn an_apex_equal_to_the_reserve_counts_as_covered() {
    assert!(
        reserve_covers_the_apex(125, 125),
        "顶点 = 保留量判罩得住（M1 的门槛是 ≤ 0）"
    );
    assert!(
        !reserve_covers_the_apex(126, 125),
        "顶点比保留量多 1 槽判越过"
    );
}

#[test]
fn apex_is_not_saturated_when_the_second_last_point_rises_and_the_last_falls_back() {
    let rises_then_falls_back = [(14, 10), (15, 12), (16, 10)];
    assert_eq!(
        apex_saturation(&rises_then_falls_back, 4),
        ApexSaturation::GrowingWithoutSaturation,
        "最后两点里一点比之前那一点高，不算饱和"
    );
    assert_eq!(
        apex_saturation(&[(14, 12), (15, 12), (16, 11)], 4),
        ApexSaturation::Saturated,
        "最后两点都不高于之前那一点"
    );
    assert_eq!(
        apex_saturation(&[(13, 50), (14, 12), (15, 12)], 4),
        ApexSaturation::FewerThanThreePointsPastTheRing,
        "批数 ≥ 3S + 2 = 14 的点只有两个"
    );
}

#[test]
fn largest_new_slots_grow_with_file_size_only_on_three_strictly_rising_last_points() {
    assert!(
        largest_new_slots_grow_with_file_size(&[5, 1, 2, 3]),
        "最后三点 1 < 2 < 3"
    );
    assert!(
        !largest_new_slots_grow_with_file_size(&[1, 2, 2]),
        "最后三点有一对相等"
    );
    assert!(!largest_new_slots_grow_with_file_size(&[2, 3]), "不到三点");
}

#[test]
fn smallest_unbuilt_delta_counts_the_ring_window_per_slot() {
    assert_eq!(
        smallest_unbuilt_delta_that_flips_the_cover(100, 125, 4),
        Some(2),
        "S = 4：100 + 13 × 2 = 126 > 125"
    );
    assert_eq!(
        smallest_unbuilt_delta_that_flips_the_cover(126, 125, 4),
        Some(0),
        "已经越过"
    );
    assert_eq!(
        smallest_unbuilt_delta_that_flips_the_cover(0, 125, 4),
        None,
        "δ ≤ 3 翻不了"
    );
    assert_eq!(ring_window_bound_slots(4, 10), 130, "(12 + 1) × 10");
}

#[test]
fn ladder_of_file_sizes_crosses_the_ring_for_both_batch_sizes() {
    assert_eq!(
        ladder_of_file_sizes(144, 4),
        vec![144, 288, 576, 1152, 1584, 1728, 1872, 2016, 2448, 3744, 6912],
        "K = 144、S = 4：N = 144 × {{1, 2, 4, 8, 11, 12, 13, 14, 17, 26, 48}}"
    );
    assert_eq!(
        ladder_of_file_sizes(1, 4),
        vec![1, 2, 4, 8, 11, 12, 13, 14, 17, 26, 48, 96],
        "K = 1、S = 4 另加 8 × 3S"
    );
    assert_eq!(delete_publish_count(144, 144), 2, "意图那次加一批");
    assert_eq!(delete_publish_count(145, 144), 3, "多一个单元多一批");
    assert_eq!(delete_publish_count(1, 1), 2, "一个单元、K = 1");
}
