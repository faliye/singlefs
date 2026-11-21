use super::*;

fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
    PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
}

#[test]
fn tree_bytes_at_file_count_1_matches_the_hand_calculation() {
    let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let (total_bytes, tree_bytes) = total_bytes_and_tree_bytes(shape);
    assert_eq!(total_bytes, 344_576);
    assert_eq!(tree_bytes, 16384 * 2 + 32768 * 2 + 16384 * 2 + 16384 * 2 + 16384 * 2 + 16384 * 2 + 16384 * 2);
}

#[test]
fn path_ratio_readings_do_not_exceed_one_at_file_count_1() {
    let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let (path_ratio_by_actual_height, path_ratio_by_maximum_height_node_width, path_ratio_by_maximum_height_unit_width) = rho_readings(shape);
    assert!((path_ratio_by_actual_height - 1.0).abs() < 1e-9, "path_ratio_by_actual_height={path_ratio_by_actual_height}");
    assert!(path_ratio_by_maximum_height_node_width <= 1.0 + 1e-9, "path_ratio_by_maximum_height_node_width={path_ratio_by_maximum_height_node_width}");
    assert!(path_ratio_by_maximum_height_unit_width <= 1.0 + 1e-9, "path_ratio_by_maximum_height_unit_width={path_ratio_by_maximum_height_unit_width}");
}

#[test]
fn extent_height_growth_threshold_matches_the_anchor() {
    let height_at = |file_count: u64| {
        let shape = shape_at(file_count, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        solve_jia_publish(shape).extent_layers.len()
    };
    let thresholds = height_growth_thresholds(1_000_000, height_at);
    assert!(!thresholds.is_empty(), "extent 树在 1..=1000000 之间应该至少长高一次");
    assert_eq!(thresholds[0], 145, "extent 叶容量 144，第 145 个条目跨过边界");
}

/// 门槛搜索：分配记录树的门槛这一次是 181（R1 之后），不是第一次登记的 205（legacy 门槛见 `jia_anchor_tests`）。
#[test]
fn allocation_height_growth_threshold_matches_the_value_after_counting_data_units_in_backlog() {
    let height_at = |file_count: u64| {
        let shape = shape_at(file_count, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        solve_jia_publish(shape).allocation_layers.len()
    };
    let thresholds = height_growth_thresholds(1_000_000, height_at);
    assert!(!thresholds.is_empty());
    assert_eq!(thresholds[0], 181, "R1 之后门槛从 205 移到 181（第七节 B10）");
}

#[test]
fn bytes_per_layer_growth_ratio_at_the_extent_threshold_is_close_to_one_layer_one_path() {
    let shape_before = shape_at(144, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let shape_at_threshold = shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let (_, tree_bytes_before_threshold) = total_bytes_and_tree_bytes(shape_before);
    let (_, tree_bytes_at_threshold) = total_bytes_and_tree_bytes(shape_at_threshold);
    let bytes_per_layer_growth_ratio = (tree_bytes_at_threshold as f64 - tree_bytes_before_threshold as f64) / (NODE_BYTES as f64 * DEVICE_COUNT as f64);
    assert!((bytes_per_layer_growth_ratio - 1.0).abs() < 1e-9, "bytes_per_layer_growth_ratio={bytes_per_layer_growth_ratio}：extent 在这个门槛上长高一层应该恰好多写一条路径的份额");
    assert!(bytes_per_layer_growth_ratio <= 1.05, "bytes_per_layer_growth_ratio={bytes_per_layer_growth_ratio}：extent 长高一层不该比『一层一条路径』贵太多");
}
