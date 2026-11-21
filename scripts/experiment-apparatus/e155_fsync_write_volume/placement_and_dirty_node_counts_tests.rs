use super::*;

#[test]
fn single_continuous_group_never_exceeds_node_count() {
    assert_eq!(continuous_groups_touch(1, 100.0, &[8.0], &[]), 1.0);
    let touched = continuous_groups_touch(1000, 100.0, &[8.0], &[]);
    assert!(touched < 1.1, "8 条记录不该碰到超过一个节点太多：{touched}");
}

#[test]
fn scattered_group_saturates_to_region_node_count_as_touch_count_grows() {
    let small_touch_count = scattered_group_touch(1.0, 1000.0, 100.0);
    let large_touch_count = scattered_group_touch(1000.0, 1000.0, 100.0);
    assert!(small_touch_count < large_touch_count, "touch_count 越大期望碰到的节点数不该变少");
    assert!(large_touch_count <= 10.0 + 1e-6, "散组不能碰到超过区间自己的节点数上界");
    // 钉绝对值（M12）：touch_count=1000、region=1000、layer_coverage=100 时 m=10，0.9^1000 趋于 0，
    // 期望值应几乎恰好饱和到 10（不是 M12 那种「不饱和」的原始 touch_count=1000）。
    assert!((large_touch_count - 10.0).abs() < 1e-6, "large_touch_count={large_touch_count}");
}

#[test]
fn combine_touches_never_exceeds_node_count() {
    let combined = combine_touches(10, 5.0, &[5.0, 5.0]);
    assert!(combined <= 10.0, "U_l ≤ n_l（V2）：{combined}");
}

/// K8：只有一个节点时，插入与删改都落在它上面，不管远近策略。
#[test]
fn single_node_class_is_always_fully_dirty_when_active() {
    for policy in [PositionPolicy::Near, PositionPolicy::Far, PositionPolicy::Balanced] {
        for placement in [Placement::Sequential, Placement::Random] {
            let touched = class_dirty_nodes_this_layer(6.0, 294.0, 6.0, 6.0, placement, policy, 0.5);
            assert_eq!(touched, 1.0, "policy={policy:?} placement={placement:?}");
        }
    }
}

/// K8 L近：旧版恒近，只有尾部一组，不产生额外的远组。
#[test]
fn near_policy_never_creates_a_far_group() {
    let touched = class_dirty_nodes_this_layer(100_000.0, 100.0, 8.0, 8.0, Placement::Random, PositionPolicy::Near, 0.9);
    // 尾部组 k=16（插入 8 + 近删改 8），期望碰到节点数应接近 1，不该被「远」的散组拉高。
    assert!(touched < 1.5, "L近不该有远组：{touched}");
}

/// K8 L远：多节点类里旧版恒远，脏节点数应明显高于 L近同一格。
#[test]
fn far_policy_touches_more_nodes_than_near_policy_when_multi_node() {
    let near = class_dirty_nodes_this_layer(100_000.0, 100.0, 8.0, 8.0, Placement::Sequential, PositionPolicy::Near, 0.0);
    let far = class_dirty_nodes_this_layer(100_000.0, 100.0, 8.0, 8.0, Placement::Sequential, PositionPolicy::Far, 0.0);
    assert!(far > near, "L远 应该比 L近 碰更多节点：near={near} far={far}");
}

fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
    PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
}

/// 7.3 手算格 1：分配记录树第 2 层刚出现的一格（P=205，F1、seq、主几何；用一段临时的搜索脚本
/// 逐个 P 扫过，204 仍 1 层、205 起 2 层，搜索代码与输出见运行记录，不留在正式产出里）。
#[test]
fn allocation_tree_reaches_two_layers_at_file_count_205() {
    let at_204 = solve_jia_publish(shape_at(204, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
    let at_205 = solve_jia_publish(shape_at(205, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
    assert_eq!(at_204.allocation_layers.len(), 1, "P=204 分配记录树仍是 1 层");
    assert_eq!(at_205.allocation_layers.len(), 2, "P=205 分配记录树长到 2 层");
}

/// 7.3 手算格 2：映射树第 2 层刚出现的一格（P=285，F1、seq、主几何，同上搜索方式）。
#[test]
fn mapping_tree_reaches_two_layers_at_file_count_285() {
    let at_284 = solve_jia_publish(shape_at(284, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
    let at_285 = solve_jia_publish(shape_at(285, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
    assert_eq!(at_284.mapping_layers.len(), 1, "P=284 映射树仍是 1 层");
    assert_eq!(at_285.mapping_layers.len(), 2, "P=285 映射树长到 2 层");
}

/// 7.3 手算格 3：rand 下 L远 与 L均 不同的一格（P=1000，F1、rand）。P=145 这一格 alloc / 映射仍是
/// 单节点，远近无差别，用不了（搜索时先试过，正好验证「差别只从多节点树才开始出现」这句话）。
#[test]
fn far_and_balanced_policies_differ_under_random_placement_at_file_count_1000() {
    let far = solve_jia_publish(shape_at(1000, Family::OneDataUnitPerFile, Placement::Random, PositionPolicy::Far));
    let balanced = solve_jia_publish(shape_at(1000, Family::OneDataUnitPerFile, Placement::Random, PositionPolicy::Balanced));
    assert_ne!(far.total_bytes(), balanced.total_bytes(), "L远 与 L均 在这一格应该给出不同的字节数");
}

/// 7.3 手算格 4：点名项超过 67 的一格（P=10⁴、F8A、rand、L远——第 1 行主网格自带的取样点，
/// 不必构造网格之外的格）。
#[test]
fn named_items_exceed_one_record_at_file_count_10000_eight_adjacent_family_random_far() {
    let outcome = solve_jia_publish(shape_at(10_000, Family::EightAdjacentDataUnitsPerFsync, Placement::Random, PositionPolicy::Far));
    assert!(outcome.named_items > NAMED_ITEMS_PER_RECORD_MAIN, "named_items={}", outcome.named_items);
    assert_eq!(outcome.record_count, outcome.named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN));
    assert!(outcome.record_count >= 2, "record_count={}", outcome.record_count);
}

/// 7.3 手算格 5：Q3.1（改一条记录也整单元重写的反事实上界）在 P=1 的一格。
/// `H = 115 + 2×key宽`（码 2）或 136（码 3）；`c` = 这一层每个脏节点平均改了几条；`w` 按 K1 / K3。
/// P=1 时甲写出的七个非数据单元逐个手算（取整到 pbs=512）：
///
/// | 单元 | H | c | w | H+cw | 取整到 512 | 单盘省下 |
/// |---|---|---|---|---|---|---|
/// | extent（此格是叶，key24） | 163 | 1（改 1 条 extent 记录 ÷ 1 脏节点） | 112 | 275 | 512 | 15872 |
/// | inode 容器（码 3） | 136 | 1（改 1 条 inode 记录 ÷ 1 脏节点） | 140 | 276 | 512 | 32256 |
/// | inode 根（码 2，key8） | 131 | 1（1 个脏子节点 ÷ 1 脏节点） | 120 | 251 | 512 | 15872 |
/// | 分配记录（码 2，key10） | 135 | 32（2 盘 ×（插 8 + 释放改写 8）÷ 1 脏节点） | 20 | 775 | 1024 | 15360 |
/// | 记账（码 2，key22） | 159 | 15（全部行重写 ÷ 1 脏节点） | 34 | 669 | 1024 | 15360 |
/// | 映射（码 2，key27） | 169 | 12（6 类 ×（插 1 + 删 1）÷ 1 脏节点） | 55 | 829 | 1024 | 15360 |
/// | 树表（码 2，key8） | 131 | 4（改了根的条目数） | 200 | 931 | 1024 | 15360 |
///
/// 单盘合计省 125440 字节，两盘 250880；`U_3` = 250880 / 344576 ≈ 72.81%（≥ 10%：按 Q2.1 同一门槛，
/// 这一行「值得设计具体候选」；上界不是省下的量，F9）。
#[test]
fn counterfactual_upper_bound_of_rewriting_whole_unit_at_file_count_1_matches_the_hand_calculation() {
    fn rounded_unit_bytes(header: u64, condition_count: u64, entry_width: u64, pbs: u64, full_width: u64) -> u64 {
        ((header + condition_count * entry_width).div_ceil(pbs) * pbs).min(full_width)
    }
    let pbs = PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES;
    let savings_per_device: u64 = [
        NODE_BYTES - rounded_unit_bytes(node_header_bytes(EXTENT_KEY_BYTES), 1, EXTENT_LEAF_ENTRY_BYTES, pbs, NODE_BYTES),
        UNIT_BYTES - rounded_unit_bytes(INODE_CONTAINER_HEADER_BYTES, 1, INODE_CONTAINER_ENTRY_BYTES, pbs, UNIT_BYTES),
        NODE_BYTES - rounded_unit_bytes(node_header_bytes(INODE_KEY_BYTES), 1, INODE_INTERNAL_ENTRY_BYTES, pbs, NODE_BYTES),
        NODE_BYTES - rounded_unit_bytes(node_header_bytes(ALLOCATION_KEY_BYTES), 32, ALLOCATION_LEAF_ENTRY_BYTES, pbs, NODE_BYTES),
        NODE_BYTES - rounded_unit_bytes(node_header_bytes(ACCOUNTING_KEY_BYTES), 15, ACCOUNTING_LEAF_ENTRY_BYTES, pbs, NODE_BYTES),
        NODE_BYTES - rounded_unit_bytes(node_header_bytes(MAPPING_KEY_BYTES), 12, MAPPING_LEAF_ENTRY_BYTES, pbs, NODE_BYTES),
        NODE_BYTES - rounded_unit_bytes(node_header_bytes(TREE_TABLE_KEY_BYTES), 4, TREE_TABLE_ENTRY_BYTES, pbs, NODE_BYTES),
    ]
    .iter()
    .sum();
    assert_eq!(savings_per_device, 125_440, "单盘合计省下的字节，见方法注释的逐行手算");
    let savings_total = savings_per_device * DEVICE_COUNT;
    assert_eq!(savings_total, 250_880);
    let anchor_bytes = solve_jia_publish(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced)).total_bytes();
    let upper_bound = savings_total as f64 / anchor_bytes as f64;
    assert!((upper_bound - 0.728_08).abs() < 1e-3, "U_3={upper_bound}");
    assert!(upper_bound >= 0.10, "Q2.1 同门槛：≥10% 记『上界够进候选』");
}
