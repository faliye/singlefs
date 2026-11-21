//! 放置与位置策略、连续与散布两种触及的节点数、按层按类的脏节点数。

use e7_index_bench::truncate_f64_to_u64;
#[cfg(test)]
use super::*;

// ============================================================
// 三、5.3：组的期望覆盖节点数
// ============================================================

/// 落点方式（5.2）：只管 fsync 与 fsync 之间的位置怎么推进。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Placement {
    Sequential,
    Random,
}

/// K8 位置策略：一个旧版落在哪。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum PositionPolicy {
    Near,
    Far,
    Balanced,
}

impl Placement {
    pub(super) fn tag(self) -> &'static str {
        match self {
            Placement::Sequential => "seq",
            Placement::Random => "rand",
        }
    }
}

impl PositionPolicy {
    pub(super) fn tag(self) -> &'static str {
        match self {
            PositionPolicy::Near => "Lnear",
            PositionPolicy::Far => "Lfar",
            PositionPolicy::Balanced => "Lbalanced",
        }
    }
}

/// 若干连续组合起来，在某一层期望覆盖的节点数（5.3 第 1 步）。
/// `group_sizes` 按位置顺序排好；`gaps_between` 长度 = `group_sizes.len() - 1`（相邻两组之间的空隙，条目数）。
pub(super) fn continuous_groups_touch(node_count_at_layer: u64, layer_coverage: f64, group_sizes: &[f64], gaps_between: &[f64]) -> f64 {
    if node_count_at_layer == 0 || group_sizes.is_empty() {
        return 0.0;
    }
    assert!(
        gaps_between.len() + 1 == group_sizes.len(),
        "空隙数必须比组数少 1：{} 组、{} 个空隙",
        group_sizes.len(),
        gaps_between.len()
    );
    let mut touched = 1.0;
    for &group_size in group_sizes {
        touched += (group_size - 1.0) / layer_coverage;
    }
    for &gap in gaps_between {
        touched += (gap / layer_coverage).min(1.0);
    }
    touched.min(node_count_at_layer as f64)
}

/// 一个散组在某一层期望覆盖的节点数（5.3 第 2 步）。`region_entries` 是这个组落在的区间条目数。
pub(super) fn scattered_group_touch(touch_count: f64, region_entries: f64, layer_coverage: f64) -> f64 {
    if touch_count <= 0.0 {
        return 0.0;
    }
    let expected_saturated_node_count = (region_entries / layer_coverage).max(1.0);
    expected_saturated_node_count * (1.0 - (1.0 - 1.0 / expected_saturated_node_count).powf(touch_count))
}

/// 若干组合起来，在某一层期望覆盖的节点数（5.3 第 3 步）：`continuous_touch` 是全部连续组合并之后的值，
/// `scattered_touches` 是各个散组各自的值。
fn combine_touches(node_count_at_layer: u64, continuous_touch: f64, scattered_touches: &[f64]) -> f64 {
    let node_count = node_count_at_layer as f64;
    if node_count <= 0.0 {
        return 0.0;
    }
    let mut surviving_fraction = 1.0 - (continuous_touch / node_count).min(1.0);
    for &scattered_touch in scattered_touches {
        surviving_fraction *= 1.0 - (scattered_touch / node_count).min(1.0);
    }
    node_count * (1.0 - surviving_fraction)
}

/// 单一连续组的树（extent、inode）逐层期望脏节点数：这次 fsync 改的 `group_size` 条记录，一路带着祖先往上算。
/// 没有旧版删改分量（extent / inode 树按逻辑位置索引，不因为物理落点变化而在树里挪动，5.3「各树的组从哪来」）。
pub(super) fn single_group_dirty_nodes_per_layer(layers: &[u64], leaf_capacity: u64, internal_capacity: u64, group_size: u64) -> Vec<f64> {
    if group_size == 0 {
        return vec![0.0; layers.len()];
    }
    let mut result = Vec::with_capacity(layers.len());
    let mut cov = leaf_capacity as f64;
    for &node_count_at_layer in layers {
        result.push(continuous_groups_touch(node_count_at_layer, cov, &[group_size as f64], &[]));
        cov *= internal_capacity as f64;
    }
    result
}

/// 一个带旧版删改的类（映射树的六个类之一、分配记录树的一块盘）在某一层的期望脏节点数（K8 三种位置策略）。
/// `region_entries`：这个类今天的条目数；`layer_coverage`：这一层节点罩的条目数；`insert_count` / `delete_count`：这次持久化插入 /
/// 删改的条目数（本模型里覆盖写恒 `insert_count == delete_count`）；`previous_touch_fraction`：L均 + rand 用上一轮迭代的 `U_l / n_l`（q）。
#[allow(clippy::too_many_arguments, reason = "K8 的公式本身要这么多输入，拆结构体不会让哪一步更好验")]
fn class_dirty_nodes_this_layer(
    region_entries: f64,
    layer_coverage: f64,
    insert_count: f64,
    delete_count: f64,
    placement: Placement,
    policy: PositionPolicy,
    previous_touch_fraction: f64,
) -> f64 {
    if insert_count <= 0.0 && delete_count <= 0.0 {
        return 0.0;
    }
    let node_count_at_layer_for_class = truncate_f64_to_u64((region_entries / layer_coverage).ceil().max(1.0));
    if node_count_at_layer_for_class <= 1 {
        // 只有一个节点：插入与删改（不管远近）都落在它上面。
        return 1.0;
    }
    let far_fraction: f64 = match (policy, placement) {
        (PositionPolicy::Near, _) => 0.0,
        (PositionPolicy::Far, _) => 1.0,
        (PositionPolicy::Balanced, Placement::Sequential) => (insert_count / layer_coverage).min(1.0),
        (PositionPolicy::Balanced, Placement::Random) => previous_touch_fraction.min(1.0),
    };
    let near_count = delete_count * (1.0 - far_fraction);
    let far_count = delete_count * far_fraction;
    let tail_group = insert_count + near_count;
    let continuous_touch = if far_fraction > 0.0 && placement == Placement::Sequential {
        // 远（seq）：连续组，落在区间起点；与尾部的插入 + 近删改分成两组，中间的空隙是区间剩下的部分。
        let gap = (region_entries - tail_group - far_count).max(0.0);
        continuous_groups_touch(node_count_at_layer_for_class, layer_coverage, &[far_count, tail_group], &[gap])
    } else {
        continuous_groups_touch(node_count_at_layer_for_class, layer_coverage, &[tail_group], &[])
    };
    let scattered_touch = if far_fraction > 0.0 && placement == Placement::Random {
        scattered_group_touch(far_count, region_entries, layer_coverage)
    } else {
        0.0
    };
    let scattered_touches: &[f64] = if scattered_touch > 0.0 { &[scattered_touch] } else { &[] };
    combine_touches(node_count_at_layer_for_class, continuous_touch, scattered_touches)
}

/// 一个类的当前状态：条目数、这次持久化插入 / 删改的条目数。
#[derive(Clone, Copy, Debug)]
pub(super) struct TreeClassActivity {
    pub(super) region_entries: f64,
    pub(super) insert_count: f64,
    pub(super) delete_count: f64,
}

/// 多个类共享一棵树（映射树的六个类、分配记录树的两块盘）逐层期望脏节点数，K8 三种位置策略；
/// `previous_touch_fractions` 长度必须等于 `layers.len()`（上一轮迭代每层的 `U_l / n_l`）。
pub(super) fn multiclass_tree_dirty_nodes_per_layer(
    layers: &[u64],
    leaf_capacity: u64,
    internal_capacity: u64,
    classes: &[TreeClassActivity],
    placement: Placement,
    policy: PositionPolicy,
    previous_touch_fractions: &[f64],
) -> Vec<f64> {
    assert_eq!(previous_touch_fractions.len(), layers.len(), "每层都要有上一轮的 q");
    let mut dirty_per_layer = Vec::with_capacity(layers.len());
    let mut cov = leaf_capacity as f64;
    for (layer_index, &node_count_at_layer) in layers.iter().enumerate() {
        let mut layer_sum = 0.0;
        for class in classes {
            layer_sum += class_dirty_nodes_this_layer(
                class.region_entries,
                cov,
                class.insert_count,
                class.delete_count,
                placement,
                policy,
                previous_touch_fractions[layer_index],
            );
        }
        dirty_per_layer.push(layer_sum.min(node_count_at_layer as f64));
        cov *= internal_capacity as f64;
    }
    dirty_per_layer
}

#[cfg(test)]
#[path = "placement_and_dirty_node_counts_tests.rs"]
mod tests;
