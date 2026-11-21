//! 默认环在飞上限、反事实各份额、部分重写的字节与节省、行 2 到 4 的阈值二分与高度增长阈值。

#[cfg(test)]
use e7_index_bench::truncate_f64_to_u64;
use super::*;

/// 默认环的在飞记录上限（768 MiB ÷ 4096 ÷ 3）。
pub(super) fn default_ring_in_flight_limit() -> u64 {
    RING_DEFAULT_BYTES / RECORD_BYTES / RING_SAFETY_FACTOR_MAIN
}

// ============================================================
// 六之二：第一次登记第 2–5 行的反事实上界函数与单测，原样带过来（不进这一次任何判据、main() 不再
// 输出它们的产物行），只为让第一次的变异表（M15/M17/M18/M22 目标在这几个函数里）能在这个新二进制上
// 照跑一遍（登记第九节：「第一次的变异表在新二进制上照跑一遍」）。
// ============================================================

/// 数据单元的头 + 尾总开销（B8/Q4.1 取整用）：kb 登记的 `DATA_UNIT_HEADER_BYTES = 105`
/// （`.claude/kb/decisions/18-块里携带什么信息.md:592`）只是头；这里的 134 = 105 + 29，
/// 29 是数据单元载荷上限公式 `UNIT_BYTES − 105 − 29 = 32634`（第七节 B1）里另算的那一段，
/// 两个不是同一个量，故意不与 kb 那个名字撞名。
#[cfg(test)]
pub(super) const DATA_UNIT_HEADER_AND_TRAILER_BYTES: u64 = 134;

impl JiaPublishOutcome {
    /// Q1.6 分子（第一次登记）：extent、inode 两树的**非叶**节点 + 分配记录、记账、映射、树表四棵的**全部**
    /// 脏节点 + 根槽。这一次不进任何判据（Q1.6 判据已撤），只为让第一次变异表 M15 有目标。
    #[cfg(test)]
    pub(super) fn ancestor_and_fixed_point_share_numerator_bytes(&self) -> u64 {
        let extent_non_leaf_dirty: f64 = self.extent_dirty[1..].iter().sum();
        truncate_f64_to_u64((extent_non_leaf_dirty * NODE_BYTES as f64).round()) * DEVICE_COUNT + self.inode_root_bytes + self.allocation_bytes + self.accounting_bytes + self.mapping_bytes + self.tree_table_bytes + self.root_slot_bytes
    }
}

/// Q2.1–Q2.4：四样固定点各自的反事实上界 `U_X = 1 − B(甲|X 不写) / B(甲)`。
#[cfg(test)]
pub(super) fn fixed_point_counterfactual_share(shape: PoolShape, suppressed: SuppressedFixedPoints) -> f64 {
    let baseline = solve_jia_publish(shape).total_bytes();
    let counterfactual = solve_jia_publish_with_suppressed_fixed_points(shape, suppressed).total_bytes();
    1.0 - (counterfactual as f64 / baseline as f64)
}

/// Q2.5：树表随树的棵数 T 的反事实上界，同一个 `U_X` 公式，X = 树表、T 可变。
#[cfg(test)]
pub(super) fn tree_table_counterfactual_share(shape: PoolShape, tree_table_entry_count: u64) -> f64 {
    let baseline = solve_jia_publish_with_suppressed_fixed_points_and_tree_count(shape, SuppressedFixedPoints::default(), tree_table_entry_count).total_bytes();
    let suppressed = SuppressedFixedPoints { tree_table: true, ..SuppressedFixedPoints::default() };
    let counterfactual = solve_jia_publish_with_suppressed_fixed_points_and_tree_count(shape, suppressed, tree_table_entry_count).total_bytes();
    1.0 - (counterfactual as f64 / baseline as f64)
}

/// Q3.1 的取整：`⌈(H + c×w) / pbs⌉ × pbs`，不超过整单元宽度。
#[cfg(test)]
fn partial_rewrite_rounded_bytes(header: u64, conditions_per_node: f64, entry_width: u64, pbs: u64, full_width: u64) -> f64 {
    let rounded = ((header as f64 + conditions_per_node * entry_width as f64) / pbs as f64).ceil() * pbs as f64;
    rounded.min(full_width as f64)
}

/// Q3.1：逐层展开一棵树省下的字节（单盘口径，调用处再乘 `DEVICE_COUNT`）。
#[allow(clippy::too_many_arguments, reason = "登记 Q3.1 的公式本身要这么多输入，拆结构体不会让哪一步更好验")]
#[cfg(test)]
fn partial_rewrite_tree_savings(
    dirty_per_layer: &[f64],
    leaf_conditions: f64,
    leaf_header: u64,
    leaf_entry_width: u64,
    leaf_full_width: u64,
    internal_header: u64,
    internal_entry_width: u64,
    internal_full_width: u64,
    pbs: u64,
) -> f64 {
    let mut savings = 0.0;
    for (layer_index, &dirty) in dirty_per_layer.iter().enumerate() {
        if dirty <= 0.0 {
            continue;
        }
        let (header, entry_width, full_width, conditions) = if layer_index == 0 {
            (leaf_header, leaf_entry_width, leaf_full_width, leaf_conditions)
        } else {
            (internal_header, internal_entry_width, internal_full_width, dirty_per_layer[layer_index - 1])
        };
        let conditions_per_node = conditions / dirty;
        let rounded = partial_rewrite_rounded_bytes(header, conditions_per_node, entry_width, pbs, full_width);
        savings += dirty * (full_width as f64 - rounded);
    }
    savings
}

/// Q3.1：改一条记录也整单元重写的反事实上界，在甲的这一格上算。
#[cfg(test)]
pub(super) fn whole_unit_rewrite_counterfactual_share(shape: PoolShape, pbs: u64) -> f64 {
    let shape = PoolShape { geometry: Geometry { physical_block_size_bytes: pbs, ..shape.geometry }, ..shape };
    let outcome = solve_jia_publish(shape);
    let fsync_data_unit_count = shape.family.data_units_per_fsync() as f64;

    let mut savings = 0.0;

    savings += partial_rewrite_tree_savings(&outcome.extent_dirty, fsync_data_unit_count, node_header_bytes(EXTENT_KEY_BYTES), EXTENT_LEAF_ENTRY_BYTES, NODE_BYTES, node_header_bytes(EXTENT_KEY_BYTES), internal_entry_bytes(EXTENT_KEY_BYTES, false), NODE_BYTES, pbs);

    savings += partial_rewrite_tree_savings(&outcome.inode_dirty, 1.0, INODE_CONTAINER_HEADER_BYTES, INODE_CONTAINER_ENTRY_BYTES, UNIT_BYTES, node_header_bytes(INODE_KEY_BYTES), INODE_INTERNAL_ENTRY_BYTES, NODE_BYTES, pbs);

    savings += partial_rewrite_tree_savings(&outcome.allocation_dirty, 4.0 * outcome.written_units, node_header_bytes(ALLOCATION_KEY_BYTES), ALLOCATION_LEAF_ENTRY_BYTES, NODE_BYTES, node_header_bytes(ALLOCATION_KEY_BYTES), internal_entry_bytes(ALLOCATION_KEY_BYTES, false), NODE_BYTES, pbs);

    let extent_dirty_total = sum_f64(&outcome.extent_dirty);
    let inode_container_dirty = outcome.inode_dirty[0];
    let inode_root_dirty_total: f64 = outcome.inode_dirty[1..].iter().sum();
    let allocation_dirty_total = sum_f64(&outcome.allocation_dirty);
    let mapping_leaf_conditions = 2.0 * (fsync_data_unit_count + extent_dirty_total + inode_container_dirty + inode_root_dirty_total + allocation_dirty_total + outcome.accounting_dirty);
    savings += partial_rewrite_tree_savings(&outcome.mapping_dirty, mapping_leaf_conditions, node_header_bytes(MAPPING_KEY_BYTES), MAPPING_LEAF_ENTRY_BYTES, NODE_BYTES, node_header_bytes(MAPPING_KEY_BYTES), internal_entry_bytes(MAPPING_KEY_BYTES, false), NODE_BYTES, pbs);

    if outcome.accounting_dirty > 0.0 {
        let conditions_per_node = 15.0 / outcome.accounting_dirty;
        let rounded = partial_rewrite_rounded_bytes(node_header_bytes(ACCOUNTING_KEY_BYTES), conditions_per_node, ACCOUNTING_LEAF_ENTRY_BYTES, pbs, NODE_BYTES);
        savings += outcome.accounting_dirty * (NODE_BYTES as f64 - rounded);
    }

    let tree_table_dirty = outcome.tree_table_bytes as f64 / NODE_BYTES as f64 / DEVICE_COUNT as f64;
    if tree_table_dirty > 0.0 {
        let conditions_per_node = TREE_TABLE_TOUCHED_ROOTS as f64 / tree_table_dirty;
        let rounded = partial_rewrite_rounded_bytes(node_header_bytes(TREE_TABLE_KEY_BYTES), conditions_per_node, TREE_TABLE_ENTRY_BYTES, pbs, NODE_BYTES);
        savings += tree_table_dirty * (NODE_BYTES as f64 - rounded);
    }

    (savings * DEVICE_COUNT as f64) / outcome.total_bytes() as f64
}

/// Q4.1：数据单元按内容取整的反事实上界。
#[cfg(test)]
pub(super) fn data_unit_rounding_counterfactual_share(shape: PoolShape, content_bytes: u64, pbs: u64) -> f64 {
    let shape = PoolShape { geometry: Geometry { physical_block_size_bytes: pbs, ..shape.geometry }, ..shape };
    let baseline = solve_jia_publish(shape).total_bytes();
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let rounded = (DATA_UNIT_HEADER_AND_TRAILER_BYTES + content_bytes).div_ceil(pbs) * pbs;
    let saving_per_unit_per_device = UNIT_BYTES.saturating_sub(rounded.min(UNIT_BYTES));
    (fsync_data_unit_count * DEVICE_COUNT * saving_per_unit_per_device) as f64 / baseline as f64
}

/// 六棵会长高的 / 恒定的树各自的树高（extent、inode、分配记录、记账、映射、树表）。
#[cfg(test)]
fn tree_heights(outcome: &JiaPublishOutcome) -> [usize; 6] {
    [outcome.extent_layers.len(), outcome.inode_layers.len(), outcome.allocation_layers.len(), 1, outcome.mapping_layers.len(), 1]
}

/// 一棵树高 `height` 的树，从叶到根走一条路径要写多少字节：inode 叶是容器（32768），其余每层 16384。
#[cfg(test)]
fn one_path_bytes(height: usize, leaf_is_inode_container: bool) -> u64 {
    assert!(height >= 1, "树高至少 1（根即叶）");
    if leaf_is_inode_container {
        UNIT_BYTES + NODE_BYTES * (height as u64 - 1)
    } else {
        NODE_BYTES * height as u64
    }
}

/// 甲这一格的 `A(P)`（`total_bytes`）与 `A_tree(P)`（六棵树的字节之和）。
#[cfg(test)]
pub(super) fn total_bytes_and_tree_bytes(shape: PoolShape) -> (u64, u64) {
    let outcome = solve_jia_publish(shape);
    let tree_bytes = outcome.extent_bytes + outcome.inode_container_bytes + outcome.inode_root_bytes + outcome.allocation_bytes + outcome.accounting_bytes + outcome.mapping_bytes + outcome.tree_table_bytes;
    (outcome.total_bytes(), tree_bytes)
}

/// 三种「树高 × 棵数」界的读法。
#[cfg(test)]
pub(super) fn rho_readings(shape: PoolShape) -> (f64, f64, f64) {
    let outcome = solve_jia_publish(shape);
    let tree_bytes = (outcome.extent_bytes + outcome.inode_container_bytes + outcome.inode_root_bytes + outcome.allocation_bytes + outcome.accounting_bytes + outcome.mapping_bytes + outcome.tree_table_bytes) as f64;
    let heights = tree_heights(&outcome);
    let one_path_sum: u64 =
        one_path_bytes(heights[0], false) + one_path_bytes(heights[1], true) + one_path_bytes(heights[2], false) + one_path_bytes(heights[3], false) + one_path_bytes(heights[4], false) + one_path_bytes(heights[5], false);
    let maximum_tree_height = *heights.iter().max().expect("六个高度都有值") as u64;
    let path_ratio_by_actual_height = tree_bytes / (DEVICE_COUNT as f64 * one_path_sum as f64);
    let path_ratio_by_maximum_height_node_width = tree_bytes / (maximum_tree_height as f64 * 6.0 * NODE_BYTES as f64 * DEVICE_COUNT as f64);
    let path_ratio_by_maximum_height_unit_width = tree_bytes / (maximum_tree_height as f64 * 6.0 * UNIT_BYTES as f64 * DEVICE_COUNT as f64);
    (path_ratio_by_actual_height, path_ratio_by_maximum_height_node_width, path_ratio_by_maximum_height_unit_width)
}

/// 在 `[low, high]` 上二分，找「`height_at` 第一次超过 `base_height`」的最小 P。
#[cfg(test)]
fn bisect_first_threshold(low: u64, high: u64, base_height: usize, height_at: impl Fn(u64) -> usize) -> Option<u64> {
    if height_at(high) <= base_height {
        return None;
    }
    let mut low = low;
    let mut high = high;
    while low < high {
        let mid = low + (high - low) / 2;
        if height_at(mid) > base_height {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    Some(low)
}

/// 每棵树每次长高一层的门槛：从 P = 1 的高度开始，逐次二分找下一个跨越点，直到搜索上限。
#[cfg(test)]
pub(super) fn height_growth_thresholds(search_upper_bound: u64, height_at: impl Fn(u64) -> usize) -> Vec<u64> {
    let mut thresholds = Vec::new();
    let mut base_height = height_at(1);
    let mut floor = 1u64;
    while let Some(threshold) = bisect_first_threshold(floor, search_upper_bound, base_height, &height_at) {
        thresholds.push(threshold);
        base_height = height_at(threshold);
        floor = threshold;
        if thresholds.len() > 16 {
            break;
        }
    }
    thresholds
}
