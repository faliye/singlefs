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
