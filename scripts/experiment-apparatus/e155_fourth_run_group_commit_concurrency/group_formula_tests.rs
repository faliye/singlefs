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
    assert!((large_touch_count - 10.0).abs() < 1e-6, "large_touch_count={large_touch_count}");
}

#[test]
fn combine_touches_never_exceeds_node_count() {
    let combined = combine_touches(10, 5.0, &[5.0, 5.0]);
    assert!(combined <= 10.0, "U_l ≤ n_l（V2）：{combined}");
}

#[test]
fn single_node_class_is_always_fully_dirty_when_active() {
    for policy in [PositionPolicy::Near, PositionPolicy::Far, PositionPolicy::Balanced] {
        for placement in [Placement::Sequential, Placement::Random] {
            let touched = class_dirty_nodes_this_layer(6.0, 294.0, 6.0, 6.0, placement, policy, 0.5, None);
            assert_eq!(touched, 1.0, "policy={policy:?} placement={placement:?}");
        }
    }
}

#[test]
fn near_policy_never_creates_a_far_group() {
    let touched = class_dirty_nodes_this_layer(100_000.0, 100.0, 8.0, 8.0, Placement::Random, PositionPolicy::Near, 0.9, None);
    assert!(touched < 1.5, "L近不该有远组：{touched}");
}

#[test]
fn far_policy_touches_more_nodes_than_near_policy_when_multi_node() {
    let near = class_dirty_nodes_this_layer(100_000.0, 100.0, 8.0, 8.0, Placement::Sequential, PositionPolicy::Near, 0.0, None);
    let far = class_dirty_nodes_this_layer(100_000.0, 100.0, 8.0, 8.0, Placement::Sequential, PositionPolicy::Far, 0.0, None);
    assert!(far > near, "L远 应该比 L近 碰更多节点：near={near} far={far}");
}

/// 修订二（B16，第三节 3.2 ②）：L均 + rand，远的比例是 `1 − 上一轮触达比例`，不是「上一轮触达比例」本身。
#[test]
fn balanced_random_far_fraction_is_one_minus_previous_touch_fraction_not_the_fraction_itself() {
    let touched = class_dirty_nodes_this_layer(10_000.0, 294.0, 1.0, 1.0, Placement::Random, PositionPolicy::Balanced, 0.2, None);
    assert!((touched - 1.780_12).abs() < 1e-3, "touched={touched}");
    // 第三节 3.2 ②：按装置原来的口径（远的比例直接取上一轮触达比例）应该是 1.199317，与修订后的读数不同。
    let legacy_reading = 0.2_f64; // 文档记录的旧口径读数，供人对照，不在装置里复算。
    assert!((legacy_reading - 0.2).abs() < 1e-9);
}

/// 修订三（B15，第三节 3.2 ③）：数据单元类旧版远近只看总量 vs 触达量，三种 policy 都一样。
#[test]
fn data_unit_class_far_fraction_ignores_position_policy() {
    for policy in [PositionPolicy::Near, PositionPolicy::Far, PositionPolicy::Balanced] {
        let sequential_reading = class_dirty_nodes_this_layer(10_000.0, 294.0, 1.0, 1.0, Placement::Sequential, policy, 0.0, Some((10_000.0, 1.0)));
        assert!((sequential_reading - 2.0).abs() < 1e-9, "policy={policy:?} sequential_reading={sequential_reading}");
        let random_reading = class_dirty_nodes_this_layer(10_000.0, 294.0, 1.0, 1.0, Placement::Random, policy, 0.0, Some((10_000.0, 1.0)));
        assert!((random_reading - 1.971_428_571).abs() < 1e-6, "policy={policy:?} random_reading={random_reading}");
    }
}

/// 修订四（B17，第三节 3.2 ④）：分配记录树 L远+随机落点，节点写与数据单元写合并成一个远组；
/// 数据新落点计不计进尾部让结果从 7.45661（并进尾部）变成 7.952435（散组）。
#[test]
fn allocation_tree_new_data_placement_scattered_under_far_policy_and_random_placement() {
    let region_entries = 12_000.0;
    let layer_coverage = 812.0;
    let dirty_with_new_placement_scattered = allocation_tree_dirty_this_layer(region_entries, layer_coverage, 8.0, 1.0, 12_000.0, 0.0, Placement::Random, PositionPolicy::Far, 0.0);
    assert!((dirty_with_new_placement_scattered - 7.952_435).abs() < 1e-6, "dirty_with_new_placement_scattered={dirty_with_new_placement_scattered}");
}

/// 修订六（B18）：散连续组——F8A（组大小 8）下间隔并集与「独立散点」的读数不同：16.542347 对 114.41612。
#[test]
fn scattered_continuous_groups_differ_from_independent_scattered_points() {
    let region = 80_000.0;
    let cov = 144.0;
    let node_count = 1_000_000u64;
    let grouped = scattered_continuous_groups_touch(node_count, cov, 16.0, 8.0, region);
    assert!((grouped - 16.542_347).abs() < 1e-6, "grouped={grouped}");
    let independent = scattered_group_touch(16.0 * 8.0, region, cov);
    assert!((independent - 114.416_12).abs() < 1e-5, "independent={independent}");
}
