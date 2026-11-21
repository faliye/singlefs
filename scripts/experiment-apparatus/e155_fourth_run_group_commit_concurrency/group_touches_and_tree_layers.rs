//! 填充比例、树的层数、摆放与位置策略、成组触及的节点数公式、各层脏节点数与分配树脏节点数。

// ============================================================
// 二、K2：树高与各层节点数（与第一次逐字相同）
// ============================================================

/// 填充率（K4）：主几何 φ=1；反向取样点 φ=0.5。
use e7_index_bench::truncate_f64_to_u64;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FillRatio {
    Full,
    Half,
}

impl FillRatio {
    pub(super) fn effective_capacity(self, raw_capacity: u64) -> u64 {
        match self {
            FillRatio::Full => raw_capacity,
            FillRatio::Half => (raw_capacity / 2).max(1),
        }
    }
}

/// 从叶到根每一层的节点数（K2）；`minimum_layers` 强制树至少这么高（inode 树恒 ≥ 2，根恒码 2）。
pub(super) fn tree_layers(entry_count: u64, leaf_capacity: u64, internal_capacity: u64, minimum_layers: usize) -> Vec<u64> {
    assert!(leaf_capacity >= 1, "叶容量必须 ≥ 1，否则树高不收敛");
    assert!(internal_capacity >= 1, "内部容量必须 ≥ 1，否则树高不收敛");
    let mut layers = Vec::new();
    let mut count = entry_count.div_ceil(leaf_capacity).max(1);
    layers.push(count);
    while count > 1 {
        count = count.div_ceil(internal_capacity);
        layers.push(count);
        assert!(layers.len() <= 64, "树高超过 64 层，多半是容量参数错了");
    }
    while layers.len() < minimum_layers {
        layers.push(1);
    }
    layers
}

// ============================================================
// 三、5.3：组的期望覆盖节点数（R2/R3/R4/R6 在这一节加新函数、改 class_dirty_nodes_this_layer）
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

/// 若干连续组合起来，在某一层期望覆盖的节点数（5.3 第 1 步，与第一次逐字相同）。
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

/// 一个散组在某一层期望覆盖的节点数（5.3 第 2 步，与第一次逐字相同）。
pub(super) fn scattered_group_touch(touch_count: f64, region_entries: f64, layer_coverage: f64) -> f64 {
    if touch_count <= 0.0 {
        return 0.0;
    }
    let expected_saturated_node_count = (region_entries / layer_coverage).max(1.0);
    expected_saturated_node_count * (1.0 - (1.0 - 1.0 / expected_saturated_node_count).powf(touch_count))
}

/// R6：散连续组——`group_count` 组、每组 `group_size` 条相邻，在长 `region_entries` 条的区间里
/// 均匀独立地落。`group_size = 1` 时退化成 5.3 第 2 步（登记 5.1 R6 原文）。
pub(super) fn scattered_continuous_groups_touch(node_count_at_layer: u64, layer_coverage: f64, group_count: f64, group_size: f64, region_entries: f64) -> f64 {
    if group_count <= 0.0 || group_size <= 0.0 {
        return 0.0;
    }
    let saturated_node_count = (region_entries / layer_coverage).max(1.0);
    let nodes_touched_per_group = saturated_node_count.min(1.0 + (group_size - 1.0) / layer_coverage);
    (saturated_node_count * (1.0 - (1.0 - nodes_touched_per_group / saturated_node_count).powf(group_count))).min(node_count_at_layer as f64)
}

/// R6：散连续组去重之后的条目数——`region_entries × (1 − (1 − group_size/region_entries)^group_count)`。
pub(super) fn scattered_continuous_groups_distinct_entries(region_entries: f64, group_count: f64, group_size: f64) -> f64 {
    if group_count <= 0.0 || group_size <= 0.0 || region_entries <= 0.0 {
        return 0.0;
    }
    region_entries * (1.0 - (1.0 - group_size / region_entries).powf(group_count))
}

/// 若干组合起来，在某一层期望覆盖的节点数（5.3 第 3 步，与第一次逐字相同）。
pub(super) fn combine_touches(node_count_at_layer: u64, continuous_touch: f64, scattered_touches: &[f64]) -> f64 {
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

/// 单一连续组的树（extent、inode）逐层期望脏节点数（与第一次逐字相同）。
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

/// 一个带旧版删改的类（映射树的六个类之一）在某一层的期望脏节点数（K8 三种位置策略）。
/// `data_unit_totals`：`Some((总量, 这次持久化触达量))` 时按 R3 判远近（与 policy 无关，只看这两个数）；
/// `None` 时按 K8 policy + R2（Balanced+Random 取 `1 − q`，不是 `q`）判远近。
#[allow(clippy::too_many_arguments, reason = "K8 的公式本身要这么多输入，拆结构体不会让哪一步更好验")]
pub(super) fn class_dirty_nodes_this_layer(
    region_entries: f64,
    layer_coverage: f64,
    insert_count: f64,
    delete_count: f64,
    placement: Placement,
    policy: PositionPolicy,
    previous_touch_fraction: f64,
    data_unit_totals: Option<(f64, f64)>,
) -> f64 {
    if insert_count <= 0.0 && delete_count <= 0.0 {
        return 0.0;
    }
    let node_count_at_layer_for_class = truncate_f64_to_u64((region_entries / layer_coverage).ceil().max(1.0));
    if node_count_at_layer_for_class <= 1 {
        // 只有一个节点：插入与删改（不管远近）都落在它上面。
        return 1.0;
    }
    let far_fraction: f64 = if let Some((total_units, touched_units)) = data_unit_totals {
        // R3（修订）：数据单元旧版远近只看总量 vs 这次持久化触达量，与 K8 策略无关。
        if total_units > touched_units {
            1.0
        } else {
            0.0
        }
    } else {
        match (policy, placement) {
            (PositionPolicy::Near, _) => 0.0,
            (PositionPolicy::Far, _) => 1.0,
            (PositionPolicy::Balanced, Placement::Sequential) => (insert_count / layer_coverage).min(1.0),
            // R2（修订）：远的比例 = 1 − q，不是 q 本身（第三节 3.2 ②）。
            (PositionPolicy::Balanced, Placement::Random) => (1.0 - previous_touch_fraction).clamp(0.0, 1.0),
        }
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

/// 一个类的当前状态：条目数、这次持久化插入 / 删改的条目数、是不是数据单元类（R3）。
#[derive(Clone, Copy, Debug)]
pub(super) struct TreeClassActivity {
    pub(super) region_entries: f64,
    pub(super) insert_count: f64,
    pub(super) delete_count: f64,
    /// R3：`Some((总量, 触达量))` 表示这是数据单元类；`None` 表示按 K8 policy 判的普通类。
    pub(super) data_unit_totals: Option<(f64, f64)>,
}

/// 多个类共享一棵树（映射树的六个类）逐层期望脏节点数，K8 三种位置策略。
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
                class.data_unit_totals,
            );
        }
        dirty_per_layer.push(layer_sum.min(node_count_at_layer as f64));
        cov *= internal_capacity as f64;
    }
    dirty_per_layer
}

/// 分配记录树一块盘一层的期望脏节点数（R1/R3/R4 合并处理；K10 的中间版释放记录 R7(a) 只进尾部插入）。
/// 节点写（新落点 + 旧版按 K8/R2 策略）与数据单元写（新落点 + 旧版按 R3/R4）的「远」贡献合并成
/// **一个**连续组或散组，不是两个独立类各自算完再相加——登记第七节 B17 手算格是这样验证的
/// （`allocation_tree_dirty_this_layer` 名字与手算格逐字节相同的读法：一块盘的分配记录区间共享同一层，
/// 节点与数据单元的「远」量合并成一次 `scattered_group_touch` / `continuous_groups_touch` 调用）。
#[allow(clippy::too_many_arguments, reason = "分配树每层脏节点数的公式本身要这么多输入，拆结构体不会让哪一步更好验")]
pub(super) fn allocation_tree_dirty_this_layer(
    region_entries: f64,
    layer_coverage: f64,
    node_units: f64,
    data_units: f64,
    data_total_units: f64,
    extra_release_only_insertions: f64,
    placement: Placement,
    policy: PositionPolicy,
    previous_touch_fraction: f64,
) -> f64 {
    if node_units <= 0.0 && data_units <= 0.0 && extra_release_only_insertions <= 0.0 {
        return 0.0;
    }
    let node_count_at_layer = truncate_f64_to_u64((region_entries / layer_coverage).ceil().max(1.0));
    if node_count_at_layer <= 1 {
        return 1.0;
    }
    let node_far_fraction: f64 = match (policy, placement) {
        (PositionPolicy::Near, _) => 0.0,
        (PositionPolicy::Far, _) => 1.0,
        (PositionPolicy::Balanced, Placement::Sequential) => (node_units / layer_coverage).min(1.0),
        (PositionPolicy::Balanced, Placement::Random) => (1.0 - previous_touch_fraction).clamp(0.0, 1.0),
    };
    // R3：数据单元旧版远近只看总量 vs 这次持久化触达量，与 K8 策略无关。
    let data_is_far = data_total_units > data_units;
    // R4：仅 L远 + rand 时，数据单元新落点也散组，不并进尾部（登记 5.1 R4，第七节 B17）。
    let data_new_placement_scattered = policy == PositionPolicy::Far && placement == Placement::Random && data_is_far;

    let node_near = node_units * (1.0 - node_far_fraction);
    let node_far = node_units * node_far_fraction;
    let data_near = if data_is_far { 0.0 } else { data_units };
    let data_far = if data_is_far { data_units } else { 0.0 };
    let data_new_in_tail = if data_new_placement_scattered { 0.0 } else { data_units };
    let data_new_scattered = if data_new_placement_scattered { data_units } else { 0.0 };

    // 尾部：节点新落点（恒在尾部）+ 节点旧版的近份 + 数据新落点（除非 R4 把它挪去散组）+ 数据旧版的近份
    // + K10 中间版释放记录（R7(a)，恒在尾部，不参与远近判定）。
    let tail_group = node_units + node_near + data_new_in_tail + data_near + extra_release_only_insertions;
    // 远：节点旧版的远份 + 数据旧版的远份 + （R4 触发时）数据新落点，三者合并成一个连续组或散组。
    let far_total = node_far + data_far + data_new_scattered;

    if far_total > 0.0 && placement == Placement::Sequential {
        let gap = (region_entries - tail_group - far_total).max(0.0);
        continuous_groups_touch(node_count_at_layer, layer_coverage, &[far_total, tail_group], &[gap])
    } else if far_total > 0.0 && placement == Placement::Random {
        let continuous = continuous_groups_touch(node_count_at_layer, layer_coverage, &[tail_group], &[]);
        let scattered = scattered_group_touch(far_total, region_entries, layer_coverage);
        combine_touches(node_count_at_layer, continuous, &[scattered])
    } else {
        continuous_groups_touch(node_count_at_layer, layer_coverage, &[tail_group], &[])
    }
}
