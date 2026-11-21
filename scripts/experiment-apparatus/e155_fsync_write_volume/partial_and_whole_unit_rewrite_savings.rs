//! 部分重写与整单元重写、数据单元取整的反事实节省。

use super::*;

/// 数据单元的头 + 尾总开销（B8/Q4.1 取整用）：kb 登记的 `DATA_UNIT_HEADER_BYTES = 105`
/// （`.claude/kb/decisions/18-块里携带什么信息.md:592`）只是头；这里的 134 = 105 + 29，
/// 29 是数据单元载荷上限公式 `UNIT_BYTES − 105 − 29 = 32634`（第七节 B1）里另算的那一段，
/// 两个不是同一个量，故意不与 kb 那个名字撞名。
const DATA_UNIT_HEADER_AND_TRAILER_BYTES: u64 = 134;

// ============================================================
// 七、第 2–4 行：四样固定点、整单元重写、数据单元取整的反事实上界（第三段）
// ============================================================

/// Q2.1–Q2.4：四样固定点各自的反事实上界 `U_X = 1 − B(甲|X 不写) / B(甲)`（登记 6.Q2.1）。
pub(super) fn fixed_point_counterfactual_share(shape: PoolShape, suppressed: SuppressedFixedPoints) -> f64 {
    let baseline = solve_jia_publish(shape).total_bytes();
    let counterfactual = solve_jia_publish_with_suppressed_fixed_points(shape, suppressed).total_bytes();
    1.0 - (counterfactual as f64 / baseline as f64)
}

/// Q2.5：树表随树的棵数 T 的反事实上界，同一个 `U_X` 公式，X = 树表、T 可变（登记 6.Q2.5）。
pub(super) fn tree_table_counterfactual_share(shape: PoolShape, tree_table_entry_count: u64) -> f64 {
    let baseline = solve_jia_publish_with_suppressed_fixed_points_and_tree_count(shape, SuppressedFixedPoints::default(), tree_table_entry_count).total_bytes();
    let suppressed = SuppressedFixedPoints { tree_table: true, ..SuppressedFixedPoints::default() };
    let counterfactual = solve_jia_publish_with_suppressed_fixed_points_and_tree_count(shape, suppressed, tree_table_entry_count).total_bytes();
    1.0 - (counterfactual as f64 / baseline as f64)
}

/// Q3.1 的取整：`⌈(H + c×w) / pbs⌉ × pbs`，不超过整单元宽度。
fn partial_rewrite_rounded_bytes(header: u64, conditions_per_node: f64, entry_width: u64, pbs: u64, full_width: u64) -> f64 {
    let rounded = ((header as f64 + conditions_per_node * entry_width as f64) / pbs as f64).ceil() * pbs as f64;
    rounded.min(full_width as f64)
}

/// Q3.1：逐层展开一棵树省下的字节（单盘口径，调用处再乘 `DEVICE_COUNT`）。`leaf_conditions` 是叶层自己这次持久化的
/// 插 / 删 / 改写条数；内部层的条件数取「上一层的脏节点数」（登记 6.Q3.1：「内部：脏子节点数 ÷ 脏节点数」）。
#[allow(clippy::too_many_arguments, reason = "登记 Q3.1 的公式本身要这么多输入，拆结构体不会让哪一步更好验")]
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

/// Q3.1：改一条记录也整单元重写的反事实上界，在甲的这一格上算（登记 6.Q3.1）；`pbs` 用登记声明的 512 或反向 4096。
pub(super) fn whole_unit_rewrite_counterfactual_share(shape: PoolShape, pbs: u64) -> f64 {
    let shape = PoolShape { geometry: Geometry { physical_block_size_bytes: pbs, ..shape.geometry }, ..shape };
    let outcome = solve_jia_publish(shape);
    let fsync_data_unit_count = shape.family.data_units_per_fsync() as f64;

    let mut savings = 0.0;

    // extent：码 2，叶 conditions = d（这次 fsync 改的数据单元数，D25 的「1 条共享脊柱」）。
    savings += partial_rewrite_tree_savings(
        &outcome.extent_dirty,
        fsync_data_unit_count,
        node_header_bytes(EXTENT_KEY_BYTES),
        EXTENT_LEAF_ENTRY_BYTES,
        NODE_BYTES,
        node_header_bytes(EXTENT_KEY_BYTES),
        internal_entry_bytes(EXTENT_KEY_BYTES, false),
        NODE_BYTES,
        pbs,
    );

    // inode：叶是码 3 容器，conditions = f = 1；内部层是码 2。
    savings += partial_rewrite_tree_savings(
        &outcome.inode_dirty,
        1.0,
        INODE_CONTAINER_HEADER_BYTES,
        INODE_CONTAINER_ENTRY_BYTES,
        UNIT_BYTES,
        node_header_bytes(INODE_KEY_BYTES),
        INODE_INTERNAL_ENTRY_BYTES,
        NODE_BYTES,
        pbs,
    );

    // 分配记录：叶 conditions = 4W（2 盘 ×（插 + 释放改写各一份 W）），W 用这一格收敛后的 `written_units`。
    savings += partial_rewrite_tree_savings(
        &outcome.allocation_dirty,
        4.0 * outcome.written_units,
        node_header_bytes(ALLOCATION_KEY_BYTES),
        ALLOCATION_LEAF_ENTRY_BYTES,
        NODE_BYTES,
        node_header_bytes(ALLOCATION_KEY_BYTES),
        internal_entry_bytes(ALLOCATION_KEY_BYTES, false),
        NODE_BYTES,
        pbs,
    );

    // 映射：叶 conditions = 6 类各插 + 删 = 2×(d + extent 脏 + inode 容器脏 + inode 非叶脏 + 分配记录脏 + 记账脏)。
    let extent_dirty_total = sum_f64(&outcome.extent_dirty);
    let inode_container_dirty = outcome.inode_dirty[0];
    let inode_root_dirty_total: f64 = outcome.inode_dirty[1..].iter().sum();
    let allocation_dirty_total = sum_f64(&outcome.allocation_dirty);
    let mapping_leaf_conditions =
        2.0 * (fsync_data_unit_count + extent_dirty_total + inode_container_dirty + inode_root_dirty_total + allocation_dirty_total + outcome.accounting_dirty);
    savings += partial_rewrite_tree_savings(
        &outcome.mapping_dirty,
        mapping_leaf_conditions,
        node_header_bytes(MAPPING_KEY_BYTES),
        MAPPING_LEAF_ENTRY_BYTES,
        NODE_BYTES,
        node_header_bytes(MAPPING_KEY_BYTES),
        internal_entry_bytes(MAPPING_KEY_BYTES, false),
        NODE_BYTES,
        pbs,
    );

    // 记账：主几何恒 1 层，conditions = 15（全部行重写）。
    if outcome.accounting_dirty > 0.0 {
        let conditions_per_node = 15.0 / outcome.accounting_dirty;
        let rounded = partial_rewrite_rounded_bytes(node_header_bytes(ACCOUNTING_KEY_BYTES), conditions_per_node, ACCOUNTING_LEAF_ENTRY_BYTES, pbs, NODE_BYTES);
        savings += outcome.accounting_dirty * (NODE_BYTES as f64 - rounded);
    }

    // 树表：主几何恒 1 层，conditions = 改了根的条目数（4）。
    let tree_table_dirty = outcome.tree_table_bytes as f64 / NODE_BYTES as f64 / DEVICE_COUNT as f64;
    if tree_table_dirty > 0.0 {
        let conditions_per_node = TREE_TABLE_TOUCHED_ROOTS as f64 / tree_table_dirty;
        let rounded = partial_rewrite_rounded_bytes(node_header_bytes(TREE_TABLE_KEY_BYTES), conditions_per_node, TREE_TABLE_ENTRY_BYTES, pbs, NODE_BYTES);
        savings += tree_table_dirty * (NODE_BYTES as f64 - rounded);
    }

    (savings * DEVICE_COUNT as f64) / outcome.total_bytes() as f64
}

/// Q4.1：数据单元按内容取整的反事实上界（登记 6.Q4.1）。
pub(super) fn data_unit_rounding_counterfactual_share(shape: PoolShape, content_bytes: u64, pbs: u64) -> f64 {
    let shape = PoolShape { geometry: Geometry { physical_block_size_bytes: pbs, ..shape.geometry }, ..shape };
    let baseline = solve_jia_publish(shape).total_bytes();
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let rounded = (DATA_UNIT_HEADER_AND_TRAILER_BYTES + content_bytes).div_ceil(pbs) * pbs;
    let saving_per_unit_per_device = UNIT_BYTES.saturating_sub(rounded.min(UNIT_BYTES));
    (fsync_data_unit_count * DEVICE_COUNT * saving_per_unit_per_device) as f64 / baseline as f64
}

#[cfg(test)]
#[path = "partial_and_whole_unit_rewrite_savings_tests.rs"]
mod tests;
