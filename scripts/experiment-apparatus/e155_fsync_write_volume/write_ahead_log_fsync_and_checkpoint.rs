//! 预写日志两条臂的一次 fsync 与检查点的字节数与调用数。

use e7_index_bench::truncate_f64_to_u64;
use super::*;

// ============================================================
// 六、5.4 write_ahead_log_full 与乙（第二段）
// ============================================================

/// 岔路单第 1 行的 WAL 两臂。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum WriteAheadLogArm {
    WriteAheadLogFull,
    WriteAheadLogLeaf,
}

/// WAL 两臂一次 fsync 写出的东西（5.4）：不发根、不写固定点、不写树表，次序同甲但只有一道 barrier
/// 之前的单元与一道之后的记录。
#[derive(Clone, Debug)]
pub(super) struct WriteAheadLogFsyncOutcome {
    pub(super) data_bytes: u64,
    pub(super) extent_bytes: u64,
    pub(super) inode_container_bytes: u64,
    pub(super) inode_root_bytes: u64,
    pub(super) record_bytes: u64,
    pub(super) write_calls: u64,
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    pub(super) barriers: u64,
    pub(super) record_count: u64,
}

impl WriteAheadLogFsyncOutcome {
    pub(super) fn total_bytes(&self) -> u64 {
        self.data_bytes + self.extent_bytes + self.inode_container_bytes + self.inode_root_bytes + self.record_bytes
    }
}

/// 求一格 WAL 两臂一次 fsync 的写出。write_ahead_log_full 写脏叶与全部祖先、记一条按树点名的记录；
/// 乙只写脏叶、记一条按叶点名的记录（5.4）。
pub(super) fn solve_write_ahead_log_fsync(shape: PoolShape, arm: WriteAheadLogArm) -> WriteAheadLogFsyncOutcome {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();

    let extent_leaf_cap = geometry.leaf_capacity(node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES));
    let extent_internal_cap = geometry.internal_capacity(EXTENT_KEY_BYTES);
    let extent_layers = tree_layers(data_unit_count, extent_leaf_cap, extent_internal_cap, 1);
    let extent_dirty = single_group_dirty_nodes_per_layer(&extent_layers, extent_leaf_cap, extent_internal_cap, fsync_data_unit_count);

    let inode_leaf_cap = geometry.leaf_capacity(inode_container_capacity());
    let inode_internal_cap = geometry.leaf_capacity(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES));
    let inode_layers = tree_layers(shape.file_count, inode_leaf_cap, inode_internal_cap, 2);
    let inode_dirty = single_group_dirty_nodes_per_layer(&inode_layers, inode_leaf_cap, inode_internal_cap, 1);

    let (extent_component, inode_container_component, inode_root_component, named_items) = match arm {
        WriteAheadLogArm::WriteAheadLogFull => {
            let extent_total = sum_f64(&extent_dirty);
            let inode_container = inode_dirty[0];
            let inode_root: f64 = inode_dirty[1..].iter().sum();
            // 每棵被碰到的树点名它的新树根一项：这一格 d ≥ 1、f = 1，extent 与 inode 恒各被碰到一次。
            (extent_total, inode_container, inode_root, 2u64)
        }
        WriteAheadLogArm::WriteAheadLogLeaf => {
            let extent_leaf = extent_dirty[0];
            let inode_leaf = inode_dirty[0]; // 容器 = 叶
            let dirty_leaves = truncate_f64_to_u64((extent_leaf + inode_leaf).round());
            (extent_leaf, inode_leaf, 0.0, dirty_leaves)
        }
    };

    let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);
    let unit_count = truncate_f64_to_u64((fsync_data_unit_count as f64 + extent_component + inode_container_component + inode_root_component).round());
    let write_calls = unit_count * DEVICE_COUNT + record_count * DEVICE_COUNT;

    WriteAheadLogFsyncOutcome {
        data_bytes: fsync_data_unit_count * UNIT_BYTES * DEVICE_COUNT,
        extent_bytes: truncate_f64_to_u64((extent_component * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        inode_container_bytes: truncate_f64_to_u64((inode_container_component * UNIT_BYTES as f64).round()) * DEVICE_COUNT,
        inode_root_bytes: truncate_f64_to_u64((inode_root_component * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,
        write_calls,
        barriers: 2 * DEVICE_COUNT,
        record_count,
    }
}

/// WAL 两臂一次 checkpoint 写出的东西（5.4）：四样固定点的 K9 净改动 + 乙额外补的用户树祖先追赶。
#[derive(Clone, Debug)]
pub(super) struct WriteAheadLogCheckpointOutcome {
    pub(super) extent_catch_up_bytes: u64,
    pub(super) inode_catch_up_bytes: u64,
    pub(super) allocation_bytes: u64,
    pub(super) accounting_bytes: u64,
    pub(super) mapping_bytes: u64,
    pub(super) tree_table_bytes: u64,
    pub(super) record_bytes: u64,
    pub(super) root_slot_bytes: u64,
    pub(super) system_configuration_bytes: u64,
    pub(super) write_calls: u64,
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    pub(super) fua_count: u64,
    pub(super) record_count: u64,
}

impl WriteAheadLogCheckpointOutcome {
    pub(super) fn total_bytes(&self) -> u64 {
        self.extent_catch_up_bytes
            + self.inode_catch_up_bytes
            + self.allocation_bytes
            + self.accounting_bytes
            + self.mapping_bytes
            + self.tree_table_bytes
            + self.record_bytes
            + self.root_slot_bytes
            + self.system_configuration_bytes
    }
}

/// 求一格 WAL 两臂一次 checkpoint（间隔 `interval_fsyncs` 次 fsync 之后）的写出。
/// K9（主几何）：净改动按「间隔并集」算，乙额外补上用户树第 1 层起的全部祖先（write_ahead_log_full 每次 fsync 已经写过，不必再补）。
pub(super) fn solve_write_ahead_log_checkpoint(shape: PoolShape, arm: WriteAheadLogArm, interval_fsyncs: u64) -> WriteAheadLogCheckpointOutcome {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let interval_data_unit_touch_count = interval_fsyncs as f64 * fsync_data_unit_count as f64;
    let interval_inode_record_touch_count = interval_fsyncs as f64; // 每次 fsync 恰好碰 1 条 inode 记录（f = 1）。

    let extent_touch = match shape.placement {
        Placement::Sequential => TouchSpecification::Continuous(interval_data_unit_touch_count),
        Placement::Random => TouchSpecification::Scattered(interval_data_unit_touch_count),
    };
    let inode_touch = match shape.placement {
        Placement::Sequential => TouchSpecification::Continuous(interval_inode_record_touch_count),
        Placement::Random => TouchSpecification::Scattered(interval_inode_record_touch_count),
    };
    let distinct_data_units_touched = match shape.placement {
        Placement::Sequential => continuous_groups_touch(data_unit_count, 1.0, &[interval_data_unit_touch_count], &[]),
        Placement::Random => scattered_group_touch(interval_data_unit_touch_count, data_unit_count as f64, 1.0),
    };

    // Q2 的反事实只问甲（登记 5.6：「第 2–4 行：第 1 行主表的...只算甲」），WAL 两臂的 checkpoint 恒不抑制四样固定点。
    let core = solve_fixed_point_core(
        data_unit_count,
        shape.file_count,
        extent_touch,
        inode_touch,
        distinct_data_units_touched,
        shape.placement,
        shape.policy,
        geometry,
        SuppressedFixedPoints::default(),
        TREE_TABLE_ENTRY_COUNT,
    );
    let allocation_dirty_total = sum_f64(&core.allocation_dirty);
    let mapping_dirty_total = sum_f64(&core.mapping_dirty);

    let (extent_catch_up, _inode_container_catch_up, inode_root_catch_up, extra_named_items) = match arm {
        WriteAheadLogArm::WriteAheadLogFull => (0.0, 0.0, 0.0, 0u64),
        WriteAheadLogArm::WriteAheadLogLeaf => {
            // 乙每次 fsync 只写叶：checkpoint 要把第 1 层起的全部祖先追上来；容器（叶）已经在每次 fsync 里写过。
            let extent_ancestors: f64 = core.extent_dirty[1..].iter().sum();
            let inode_ancestors: f64 = core.inode_dirty[1..].iter().sum();
            let items = truncate_f64_to_u64((extent_ancestors + inode_ancestors).round());
            (extent_ancestors, 0.0, inode_ancestors, items)
        }
    };

    let fixed_point_units = allocation_dirty_total + core.accounting_dirty_total + mapping_dirty_total + core.tree_table_dirty_total;
    // 点名项按「固定点单元数（各自四舍五入）+ 乙的祖先追赶数」相加，不对总和再四舍五入一次——
    // 两种算法在小数部分接近 .5 时会差 1（先加后舍入 vs 先舍入后加），这里取跟 `extra_named_items`
    // 同一套舍入口径，两者天然一致，不需要再断言。
    let named_items = truncate_f64_to_u64(fixed_point_units.round()) + extra_named_items;
    let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);

    let pbs = geometry.physical_block_size_bytes;
    let unit_write_calls = named_items * DEVICE_COUNT;
    let record_calls = record_count * DEVICE_COUNT;
    let total_calls = unit_write_calls + record_calls + 1 + DEVICE_COUNT;

    WriteAheadLogCheckpointOutcome {
        extent_catch_up_bytes: truncate_f64_to_u64((extent_catch_up * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        inode_catch_up_bytes: truncate_f64_to_u64((inode_root_catch_up * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        allocation_bytes: truncate_f64_to_u64((allocation_dirty_total * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        accounting_bytes: truncate_f64_to_u64((core.accounting_dirty_total * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        mapping_bytes: truncate_f64_to_u64((mapping_dirty_total * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        tree_table_bytes: truncate_f64_to_u64((core.tree_table_dirty_total * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,
        root_slot_bytes: pbs,
        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT,
        write_calls: total_calls,
        fua_count: 1,
        record_count,
    }
}

/// 摊销行（Q1.2）：（N 次 fsync 之和 + checkpoint）÷ N。
pub(super) fn amortize(fsync_total_bytes: u64, checkpoint_total_bytes: u64, interval_fsyncs: u64) -> f64 {
    (fsync_total_bytes as f64 * interval_fsyncs as f64 + checkpoint_total_bytes as f64) / interval_fsyncs as f64
}

#[cfg(test)]
#[path = "write_ahead_log_fsync_and_checkpoint_tests.rs"]
mod tests;
