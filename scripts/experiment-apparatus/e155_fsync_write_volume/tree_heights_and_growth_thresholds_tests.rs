use super::*;

fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
    PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
}

/// Q5.1：P = 1 一格 `A_tree` 恰好是七个非数据角色里去掉数据单元后的六棵树部分——
/// 用 B2 锚点手算：extent 16384×2 + inode 容器 32768×2 + inode 根 16384×2 + 分配记录 16384×2
/// + 记账 16384×2 + 映射 16384×2 + 树表 16384×2 = 278528（B(甲) 344576 减掉数据单元 65536 与固定开销）。
#[test]
fn tree_bytes_at_file_count_1_matches_the_hand_calculation() {
    let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let (total_bytes, tree_bytes) = total_bytes_and_tree_bytes(shape);
    assert_eq!(total_bytes, 344_576);
    assert_eq!(tree_bytes, 16384 * 2 + 32768 * 2 + 16384 * 2 + 16384 * 2 + 16384 * 2 + 16384 * 2 + 16384 * 2);
}

/// Q5.2a/b/c：P = 1 一格每棵树高度都是能取到的最小值（extent/alloc/mapping/记账/树表 1 层、inode 2 层），
/// 「一条路径」字节应该恰好等于 `A_tree`（一次持久化每棵树只碰一条从叶到根的路径），三种 ρ 都应该 ≤ 1。
#[test]
fn path_ratio_readings_do_not_exceed_one_at_file_count_1() {
    let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let (path_ratio_by_actual_height, path_ratio_by_maximum_height_node_width, path_ratio_by_maximum_height_unit_width) = rho_readings(shape);
    assert!(
        (path_ratio_by_actual_height - 1.0).abs() < 1e-9,
        "P=1 时六棵树各自只有一条路径，path_ratio_by_actual_height 应该恰好是 1：{path_ratio_by_actual_height}"
    );
    assert!(path_ratio_by_maximum_height_node_width <= 1.0 + 1e-9, "path_ratio_by_maximum_height_node_width={path_ratio_by_maximum_height_node_width}");
    assert!(path_ratio_by_maximum_height_unit_width <= 1.0 + 1e-9, "path_ratio_by_maximum_height_unit_width={path_ratio_by_maximum_height_unit_width}");
}

/// 门槛搜索：extent 树在 F1、seq、主几何下第一个跨越点必须恰好是 145（B7 锚点）。
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

/// 门槛搜索：分配记录树的门槛必须与 7.3 手算格（P = 205）相同。
#[test]
fn allocation_height_growth_threshold_matches_the_hand_calculated_cell() {
    let height_at = |file_count: u64| {
        let shape = shape_at(file_count, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        solve_jia_publish(shape).allocation_layers.len()
    };
    let thresholds = height_growth_thresholds(1_000_000, height_at);
    assert!(!thresholds.is_empty());
    assert_eq!(thresholds[0], 205);
}

/// Q5.3：e ≈ 1（一层一条路径）在 extent 树自己的门槛上必须成立——长高一层只多写一条路径的份额，
/// 不是好几条（这是「随对象数线性增长」与「指数恶化」之间最直接的判别）。
#[test]
fn bytes_per_layer_growth_ratio_at_the_extent_threshold_is_close_to_one_layer_one_path() {
    let shape_before = shape_at(144, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let shape_at_threshold = shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let (_, tree_bytes_before_threshold) = total_bytes_and_tree_bytes(shape_before);
    let (_, tree_bytes_at_threshold) = total_bytes_and_tree_bytes(shape_at_threshold);
    let bytes_per_layer_growth_ratio = (tree_bytes_at_threshold as f64 - tree_bytes_before_threshold as f64) / (NODE_BYTES as f64 * DEVICE_COUNT as f64);
    // P=144→145 只多一个脏节点（1 个新内部根，见 B7 锚点「jia = jia + NODE*D」），这个比值应该恰好是 1.0，不只是 ≤ 1.05。
    assert!(
        (bytes_per_layer_growth_ratio - 1.0).abs() < 1e-9,
        "bytes_per_layer_growth_ratio={bytes_per_layer_growth_ratio}：extent 在这个门槛上长高一层应该恰好多写一条路径的份额"
    );
    assert!(bytes_per_layer_growth_ratio <= 1.05, "bytes_per_layer_growth_ratio={bytes_per_layer_growth_ratio}：extent 长高一层不该比『一层一条路径』贵太多");
}
