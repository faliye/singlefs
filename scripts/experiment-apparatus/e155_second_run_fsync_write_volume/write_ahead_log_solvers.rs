//! 预写日志臂与它的 fsync 结果、逐批与逐间隔的求解、中间版计数、检查点结果与收敛饱和判定。

use e7_index_bench::truncate_f64_to_u64;
use super::*;

// ============================================================
// 六、5.4 wal_full 与乙-M（K10，第 6 行三臂）
// ============================================================

/// 岔路单第 6 行的 WAL 两臂。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum WriteAheadLogArm {
    WriteAheadLogFull,
    WriteAheadLogLeaf,
}

/// WAL 两臂一次 fsync 写出的东西（与第一次逐字相同：不发根、不写固定点、不写树表）。
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
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    pub(super) record_count: u64,
}

impl WriteAheadLogFsyncOutcome {
    pub(super) fn total_bytes(&self) -> u64 {
        self.data_bytes + self.extent_bytes + self.inode_container_bytes + self.inode_root_bytes + self.record_bytes
    }
}

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
            (extent_total, inode_container, inode_root, 2u64)
        }
        WriteAheadLogArm::WriteAheadLogLeaf => {
            let extent_leaf = extent_dirty[0];
            let inode_leaf = inode_dirty[0];
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

/// R8（第 7 行组提交）：WAL 两臂把 `concurrent_fsync_count` 个并发 fsync 并成一批 = 一次「单元 → 屏障 → 记录 → 屏障」；
/// wal_full 恒点名两棵新树根（2 项，不随 concurrent_fsync_count 变），乙-M 点名这一批的全部脏叶。
pub(super) fn solve_write_ahead_log_batch_fsync(shape: PoolShape, arm: WriteAheadLogArm, concurrent_fsync_count: u64, shared: bool) -> WriteAheadLogFsyncOutcome {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let batch_data_unit_count = concurrent_fsync_count * fsync_data_unit_count;

    let extent_leaf_cap = geometry.leaf_capacity(node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES));
    let extent_internal_cap = geometry.internal_capacity(EXTENT_KEY_BYTES);
    let extent_layers = tree_layers(data_unit_count, extent_leaf_cap, extent_internal_cap, 1);
    let extent_dirty = if shared {
        single_group_dirty_nodes_per_layer(&extent_layers, extent_leaf_cap, extent_internal_cap, batch_data_unit_count)
    } else {
        touch_dirty_nodes_per_layer(&extent_layers, extent_leaf_cap, extent_internal_cap, data_unit_count as f64, TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: fsync_data_unit_count as f64 })
    };

    let inode_leaf_cap = geometry.leaf_capacity(inode_container_capacity());
    let inode_internal_cap = geometry.leaf_capacity(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES));
    let inode_layers = tree_layers(shape.file_count, inode_leaf_cap, inode_internal_cap, 2);
    let inode_dirty = if shared {
        single_group_dirty_nodes_per_layer(&inode_layers, inode_leaf_cap, inode_internal_cap, concurrent_fsync_count)
    } else {
        touch_dirty_nodes_per_layer(&inode_layers, inode_leaf_cap, inode_internal_cap, shape.file_count as f64, TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: 1.0 })
    };

    let (extent_component, inode_container_component, inode_root_component, named_items) = match arm {
        WriteAheadLogArm::WriteAheadLogFull => {
            let extent_total = sum_f64(&extent_dirty);
            let inode_container = inode_dirty[0];
            let inode_root: f64 = inode_dirty[1..].iter().sum();
            (extent_total, inode_container, inode_root, 2u64) // 恒 2 项，不随 concurrent_fsync_count 变（第一节「读法写死」）。
        }
        WriteAheadLogArm::WriteAheadLogLeaf => {
            let extent_leaf = extent_dirty[0];
            let inode_leaf = inode_dirty[0];
            let dirty_leaves = truncate_f64_to_u64((extent_leaf + inode_leaf).round());
            (extent_leaf, inode_leaf, 0.0, dirty_leaves)
        }
    };

    let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);
    let unit_count = truncate_f64_to_u64((batch_data_unit_count as f64 + extent_component + inode_container_component + inode_root_component).round());
    let write_calls = unit_count * DEVICE_COUNT + record_count * DEVICE_COUNT;

    WriteAheadLogFsyncOutcome {
        data_bytes: batch_data_unit_count * UNIT_BYTES * DEVICE_COUNT,
        extent_bytes: truncate_f64_to_u64((extent_component * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        inode_container_bytes: truncate_f64_to_u64((inode_container_component * UNIT_BYTES as f64).round()) * DEVICE_COUNT,
        inode_root_bytes: truncate_f64_to_u64((inode_root_component * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,
        write_calls,
        barriers: 2 * DEVICE_COUNT,
        record_count,
    }
}

/// WAL 两臂一次 checkpoint 写出的东西：四样固定点的间隔净改动 + 乙额外补的用户树祖先追赶
/// + K10（R7）中间版释放记录的支撑量（Q6.6/Q6.9/B11）。
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
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    pub(super) write_calls: u64,
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    pub(super) barriers: u64,
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    pub(super) fua_count: u64,
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    pub(super) record_count: u64,
    pub(super) allocation_layers: Vec<u64>,
    pub(super) intermediate_version_total: f64,
    // F2（第十节）判定要的支撑量：这个 checkpoint 用的不动点核心，逐树第 0 层的节点数与脏节点数、
    // 与它跑了几轮迭代（第一半：`fixed_point_iterations >= MAXIMUM_FIXED_POINT_ITERATIONS` 未收敛；
    // 第二半：某棵树第 0 层节点数 ≥ 64 且脏节点占比 ≥ 50%）。
    pub(super) extent_layers: Vec<u64>,
    pub(super) extent_dirty: Vec<f64>,
    pub(super) inode_layers: Vec<u64>,
    pub(super) inode_dirty: Vec<f64>,
    pub(super) allocation_dirty: Vec<f64>,
    pub(super) mapping_layers: Vec<u64>,
    pub(super) mapping_dirty: Vec<f64>,
    pub(super) fixed_point_iterations: u64,
}

/// F2 第二半：某棵树第 0 层节点数 ≥ 64 且第 0 层脏节点数占这层节点数 ≥ 50%（登记第十节 F2）。
pub(super) fn layer_zero_is_mostly_dirty(layers: &[u64], dirty: &[f64]) -> bool {
    match (layers.first(), dirty.first()) {
        (Some(&node_count), Some(&dirty_count)) if node_count >= 64 => dirty_count / node_count as f64 >= 0.5,
        _ => false,
    }
}

/// F2 两半的判定：`.0` = 第一半（未收敛），`.1` = 第二半触发的树名单（空 = 没触发）。
#[allow(clippy::too_many_arguments, reason = "入参是迭代次数加四棵树各自的层数与脏节点数，拆结构体不会少传哪一项")]
pub(super) fn convergence_and_saturation_status(iterations: u64, extent_layers: &[u64], extent_dirty: &[f64], inode_layers: &[u64], inode_dirty: &[f64], allocation_layers: &[u64], allocation_dirty: &[f64], mapping_layers: &[u64], mapping_dirty: &[f64]) -> (bool, Vec<&'static str>) {
    let half1 = iterations >= MAXIMUM_FIXED_POINT_ITERATIONS;
    let mut triggered_trees = Vec::new();
    if layer_zero_is_mostly_dirty(extent_layers, extent_dirty) {
        triggered_trees.push("extent");
    }
    if layer_zero_is_mostly_dirty(inode_layers, inode_dirty) {
        triggered_trees.push("inode");
    }
    if layer_zero_is_mostly_dirty(allocation_layers, allocation_dirty) {
        triggered_trees.push("allocation");
    }
    if layer_zero_is_mostly_dirty(mapping_layers, mapping_dirty) {
        triggered_trees.push("mapping");
    }
    (half1, triggered_trees)
}

/// JIA 结果上的 F2 判定（第 5、6 行都要用）。
pub(super) fn jia_convergence_and_saturation_status(outcome: &JiaPublishOutcome) -> (bool, Vec<&'static str>) {
    convergence_and_saturation_status(
        outcome.fixed_point_iterations,
        &outcome.extent_layers,
        &outcome.extent_dirty,
        &outcome.inode_layers,
        &outcome.inode_dirty,
        &outcome.allocation_layers,
        &outcome.allocation_dirty,
        &outcome.mapping_layers,
        &outcome.mapping_dirty,
    )
}

/// WAL checkpoint 结果上的 F2 判定（第 6 行 wal_full / 乙-M 两臂，K9 与 K10 都要看）。
pub(super) fn checkpoint_convergence_and_saturation_status(outcome: &WriteAheadLogCheckpointOutcome) -> (bool, Vec<&'static str>) {
    convergence_and_saturation_status(
        outcome.fixed_point_iterations,
        &outcome.extent_layers,
        &outcome.extent_dirty,
        &outcome.inode_layers,
        &outcome.inode_dirty,
        &outcome.allocation_layers,
        &outcome.allocation_dirty,
        &outcome.mapping_layers,
        &outcome.mapping_dirty,
    )
}

impl WriteAheadLogCheckpointOutcome {
    pub(super) fn total_bytes(&self) -> u64 {
        self.extent_catch_up_bytes + self.inode_catch_up_bytes + self.allocation_bytes + self.accounting_bytes + self.mapping_bytes + self.tree_table_bytes + self.record_bytes + self.root_slot_bytes + self.system_configuration_bytes
    }
}

/// R7：一个间隔 N_b 批里的中间版计数（数据单元、extent 各层、inode 各层分开）。
/// `dedup_*` 是间隔并集去重之后的脏节点向量（`solve_fixed_point_core` 已经算过的那份）；
/// 乙-M 的祖先只在 checkpoint 写一次，不产生中间版，所以只数叶层（inode 容器 = 第 0 层）。
#[allow(clippy::too_many_arguments, reason = "中间版计数的输入逐项摊开（形状、臂、间隔、两棵树的层数与去重脏节点、数据单元数），拆结构体不会让哪一步更好验")]
fn intermediate_version_counts(
    shape: PoolShape,
    arm: WriteAheadLogArm,
    interval_fsyncs: u64,
    extent_layers: &[u64],
    dedup_extent_dirty: &[f64],
    inode_layers: &[u64],
    dedup_inode_dirty: &[f64],
    dedup_data_units_touched: f64,
) -> (f64, f64, f64, f64) {
    let geometry = shape.geometry;
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let batch_count_in_interval = interval_fsyncs as f64;

    let extent_leaf_cap = geometry.leaf_capacity(node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES));
    let extent_internal_cap = geometry.internal_capacity(EXTENT_KEY_BYTES);
    let per_batch_extent = single_group_dirty_nodes_per_layer(extent_layers, extent_leaf_cap, extent_internal_cap, fsync_data_unit_count);

    let inode_leaf_cap = geometry.leaf_capacity(inode_container_capacity());
    let inode_internal_cap = geometry.leaf_capacity(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES));
    let per_batch_inode = single_group_dirty_nodes_per_layer(inode_layers, inode_leaf_cap, inode_internal_cap, 1);

    let data_intermediate = (batch_count_in_interval * fsync_data_unit_count as f64 - dedup_data_units_touched).max(0.0);

    let extent_intermediate_total: f64 = per_batch_extent.iter().zip(dedup_extent_dirty.iter()).map(|(&per_batch, &dedup)| (batch_count_in_interval * per_batch - dedup).max(0.0)).sum();

    let inode_intermediate_total: f64 = match arm {
        WriteAheadLogArm::WriteAheadLogFull => per_batch_inode.iter().zip(dedup_inode_dirty.iter()).map(|(&per_batch, &dedup)| (batch_count_in_interval * per_batch - dedup).max(0.0)).sum(),
        WriteAheadLogArm::WriteAheadLogLeaf => {
            if per_batch_inode.is_empty() {
                0.0
            } else {
                (batch_count_in_interval * per_batch_inode[0] - dedup_inode_dirty[0]).max(0.0)
            }
        }
    };

    let total = data_intermediate + extent_intermediate_total + inode_intermediate_total;
    (data_intermediate, extent_intermediate_total, inode_intermediate_total, total)
}

/// R8：一个间隔 `batches_in_interval` 批（每批 concurrent_fsync_count 个 fsync）里的中间版计数，批级版本的 `intermediate_version_counts`。
#[allow(clippy::too_many_arguments, reason = "批提交那一行公式的输入逐项摊开，拆结构体不会让哪一步更好验")]
fn intermediate_version_counts_batch(
    shape: PoolShape,
    arm: WriteAheadLogArm,
    concurrent_fsync_count: u64,
    shared: bool,
    batches_in_interval: u64,
    extent_layers: &[u64],
    dedup_extent_dirty: &[f64],
    inode_layers: &[u64],
    dedup_inode_dirty: &[f64],
    dedup_data_units_touched: f64,
) -> (f64, f64, f64, f64) {
    let geometry = shape.geometry;
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let batch_data_unit_count = concurrent_fsync_count * fsync_data_unit_count;
    let batches_in_interval_count = batches_in_interval as f64;

    let extent_leaf_cap = geometry.leaf_capacity(node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES));
    let extent_internal_cap = geometry.internal_capacity(EXTENT_KEY_BYTES);
    let per_batch_extent = if shared {
        single_group_dirty_nodes_per_layer(extent_layers, extent_leaf_cap, extent_internal_cap, batch_data_unit_count)
    } else {
        touch_dirty_nodes_per_layer(extent_layers, extent_leaf_cap, extent_internal_cap, 0.0, TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: fsync_data_unit_count as f64 })
    };

    let inode_leaf_cap = geometry.leaf_capacity(inode_container_capacity());
    let inode_internal_cap = geometry.leaf_capacity(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES));
    let per_batch_inode = if shared {
        single_group_dirty_nodes_per_layer(inode_layers, inode_leaf_cap, inode_internal_cap, concurrent_fsync_count)
    } else {
        touch_dirty_nodes_per_layer(inode_layers, inode_leaf_cap, inode_internal_cap, 0.0, TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: 1.0 })
    };

    let data_intermediate = (batches_in_interval_count * batch_data_unit_count as f64 - dedup_data_units_touched).max(0.0);

    let extent_intermediate_total: f64 = per_batch_extent.iter().zip(dedup_extent_dirty.iter()).map(|(&per_batch, &dedup)| (batches_in_interval_count * per_batch - dedup).max(0.0)).sum();

    let inode_intermediate_total: f64 = match arm {
        WriteAheadLogArm::WriteAheadLogFull => per_batch_inode.iter().zip(dedup_inode_dirty.iter()).map(|(&per_batch, &dedup)| (batches_in_interval_count * per_batch - dedup).max(0.0)).sum(),
        WriteAheadLogArm::WriteAheadLogLeaf => {
            if per_batch_inode.is_empty() {
                0.0
            } else {
                (batches_in_interval_count * per_batch_inode[0] - dedup_inode_dirty[0]).max(0.0)
            }
        }
    };

    let total = data_intermediate + extent_intermediate_total + inode_intermediate_total;
    (data_intermediate, extent_intermediate_total, inode_intermediate_total, total)
}

/// R8（第 7 行组提交）：一个间隔 `batches_in_interval` 批（每批 concurrent_fsync_count 个 fsync）之后的 WAL checkpoint；
/// `batches_in_interval` 由调用处按第八节 8.2 的读法算好再传进来（主：`interval_fsyncs / concurrent_fsync_count`；反向「N 按批计」：`interval_fsyncs`）。
pub(super) fn solve_write_ahead_log_batch_checkpoint(shape: PoolShape, arm: WriteAheadLogArm, concurrent_fsync_count: u64, shared: bool, batches_in_interval: u64, intermediate_version_policy: IntermediateVersionPolicy) -> WriteAheadLogCheckpointOutcome {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let batch_data_unit_count = concurrent_fsync_count * fsync_data_unit_count;
    let batches_in_interval_count = batches_in_interval as f64;
    let interval_data_unit_touch_count = batches_in_interval_count * batch_data_unit_count as f64;

    // R8 + R6：共享是「N_b 组、每组 concurrent_fsync_count·d 条」（seq 连续、rand 散连续组）；不共享 seq 是「concurrent_fsync_count 段、每段
    // N_b·d 条，段间空隙 ≥ cov_l」（`UnsharedPaths`），rand 是「N_b·concurrent_fsync_count 组、每组 d 条」（R6）。
    let extent_touch = match (shape.placement, shared) {
        (Placement::Sequential, true) => TouchSpecification::Continuous(interval_data_unit_touch_count),
        (Placement::Sequential, false) => TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: batches_in_interval_count * fsync_data_unit_count as f64 },
        (Placement::Random, true) => TouchSpecification::ScatteredContinuousGroups { group_count: batches_in_interval_count, group_size: batch_data_unit_count as f64 },
        (Placement::Random, false) => TouchSpecification::ScatteredContinuousGroups { group_count: batches_in_interval_count * concurrent_fsync_count as f64, group_size: fsync_data_unit_count as f64 },
    };
    let inode_touch = match (shape.placement, shared) {
        (Placement::Sequential, true) => TouchSpecification::Continuous(batches_in_interval_count * concurrent_fsync_count as f64),
        (Placement::Sequential, false) => TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: batches_in_interval_count },
        (Placement::Random, true) => TouchSpecification::ScatteredContinuousGroups { group_count: batches_in_interval_count, group_size: concurrent_fsync_count as f64 },
        (Placement::Random, false) => TouchSpecification::ScatteredContinuousGroups { group_count: batches_in_interval_count * concurrent_fsync_count as f64, group_size: 1.0 },
    };
    let distinct_data_units_touched = match shape.placement {
        Placement::Sequential => continuous_groups_touch(data_unit_count, 1.0, &[interval_data_unit_touch_count], &[]),
        Placement::Random => scattered_continuous_groups_distinct_entries(data_unit_count as f64, batches_in_interval_count * concurrent_fsync_count as f64, fsync_data_unit_count as f64),
    };

    let core_zero = solve_fixed_point_core(
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
        0.0,
    );

    let (_, _, _, intermediate_total) = match intermediate_version_policy {
        IntermediateVersionPolicy::NotTracked => (0.0, 0.0, 0.0, 0.0),
        IntermediateVersionPolicy::ReleasedAndBacklogged => {
            intermediate_version_counts_batch(shape, arm, concurrent_fsync_count, shared, batches_in_interval, &core_zero.extent_layers, &core_zero.extent_dirty, &core_zero.inode_layers, &core_zero.inode_dirty, distinct_data_units_touched)
        }
    };

    let core = if intermediate_total > 0.0 {
        solve_fixed_point_core(
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
            intermediate_total,
        )
    } else {
        core_zero
    };

    let allocation_dirty_total = sum_f64(&core.allocation_dirty);
    let mapping_dirty_total = sum_f64(&core.mapping_dirty);

    let (extent_catch_up, _inode_container_catch_up, inode_root_catch_up, extra_named_items) = match arm {
        WriteAheadLogArm::WriteAheadLogFull => (0.0, 0.0, 0.0, 0u64),
        WriteAheadLogArm::WriteAheadLogLeaf => {
            let extent_ancestors: f64 = core.extent_dirty[1..].iter().sum();
            let inode_ancestors: f64 = core.inode_dirty[1..].iter().sum();
            let items = truncate_f64_to_u64((extent_ancestors + inode_ancestors).round());
            (extent_ancestors, 0.0, inode_ancestors, items)
        }
    };

    let fixed_point_units = allocation_dirty_total + core.accounting_dirty_total + mapping_dirty_total + core.tree_table_dirty_total;
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
        barriers: 2 * DEVICE_COUNT,
        fua_count: 1,
        record_count,
        allocation_layers: core.allocation_layers,
        intermediate_version_total: intermediate_total,
        extent_layers: core.extent_layers,
        extent_dirty: core.extent_dirty,
        inode_layers: core.inode_layers,
        inode_dirty: core.inode_dirty,
        allocation_dirty: core.allocation_dirty,
        mapping_layers: core.mapping_layers,
        mapping_dirty: core.mapping_dirty,
        fixed_point_iterations: core.iterations,
    }
}

/// 求一格 WAL 两臂一次 checkpoint（间隔 `interval_fsyncs` 次 fsync 之后）的写出。
/// `intermediate_version_policy`：K9（不进积压）或 K10（R7，中间版进积压 + 尾部插入）。
pub(super) fn solve_write_ahead_log_checkpoint(shape: PoolShape, arm: WriteAheadLogArm, interval_fsyncs: u64, intermediate_version_policy: IntermediateVersionPolicy) -> WriteAheadLogCheckpointOutcome {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let interval_data_unit_touch_count = interval_fsyncs as f64 * fsync_data_unit_count as f64;
    let interval_inode_record_touch_count = interval_fsyncs as f64;

    // R6：rand 下用散连续组（N_g 组、每组 s 条），不是把 N×d 次触达当独立散点；seq 不变。
    let extent_touch = match shape.placement {
        Placement::Sequential => TouchSpecification::Continuous(interval_data_unit_touch_count),
        Placement::Random => TouchSpecification::ScatteredContinuousGroups { group_count: interval_fsyncs as f64, group_size: fsync_data_unit_count as f64 },
    };
    let inode_touch = match shape.placement {
        Placement::Sequential => TouchSpecification::Continuous(interval_inode_record_touch_count),
        // inode 每次 fsync 只碰 1 条记录（f=1），组大小恒 1，散连续组退化成散点——两种写法读数相同。
        Placement::Random => TouchSpecification::ScatteredContinuousGroups { group_count: interval_fsyncs as f64, group_size: 1.0 },
    };
    let distinct_data_units_touched = match shape.placement {
        Placement::Sequential => continuous_groups_touch(data_unit_count, 1.0, &[interval_data_unit_touch_count], &[]),
        Placement::Random => scattered_continuous_groups_distinct_entries(data_unit_count as f64, interval_fsyncs as f64, fsync_data_unit_count as f64),
    };

    // 第一遍（extra=0）：算出 extent / inode 的间隔并集去重脏节点数，供 I（中间版）计数用；
    // 这两个向量不受 extra_allocation_release_insertions 影响（它只进分配记录树的计算）。
    let core_zero = solve_fixed_point_core(
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
        0.0,
    );

    let (_, _, _, intermediate_total) = match intermediate_version_policy {
        IntermediateVersionPolicy::NotTracked => (0.0, 0.0, 0.0, 0.0),
        IntermediateVersionPolicy::ReleasedAndBacklogged => intermediate_version_counts(shape, arm, interval_fsyncs, &core_zero.extent_layers, &core_zero.extent_dirty, &core_zero.inode_layers, &core_zero.inode_dirty, distinct_data_units_touched),
    };

    let core = if intermediate_total > 0.0 {
        solve_fixed_point_core(
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
            intermediate_total,
        )
    } else {
        core_zero
    };

    let allocation_dirty_total = sum_f64(&core.allocation_dirty);
    let mapping_dirty_total = sum_f64(&core.mapping_dirty);

    let (extent_catch_up, _inode_container_catch_up, inode_root_catch_up, extra_named_items) = match arm {
        WriteAheadLogArm::WriteAheadLogFull => (0.0, 0.0, 0.0, 0u64),
        WriteAheadLogArm::WriteAheadLogLeaf => {
            let extent_ancestors: f64 = core.extent_dirty[1..].iter().sum();
            let inode_ancestors: f64 = core.inode_dirty[1..].iter().sum();
            let items = truncate_f64_to_u64((extent_ancestors + inode_ancestors).round());
            (extent_ancestors, 0.0, inode_ancestors, items)
        }
    };

    let fixed_point_units = allocation_dirty_total + core.accounting_dirty_total + mapping_dirty_total + core.tree_table_dirty_total;
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
        barriers: 2 * DEVICE_COUNT,
        fua_count: 1,
        record_count,
        allocation_layers: core.allocation_layers,
        intermediate_version_total: intermediate_total,
        extent_layers: core.extent_layers,
        extent_dirty: core.extent_dirty,
        inode_layers: core.inode_layers,
        inode_dirty: core.inode_dirty,
        allocation_dirty: core.allocation_dirty,
        mapping_layers: core.mapping_layers,
        mapping_dirty: core.mapping_dirty,
        fixed_point_iterations: core.iterations,
    }
}

/// 摊销行（Q6.2）：（N 次 fsync 之和 + checkpoint）÷ N。
pub(super) fn amortize(fsync_total_bytes: u64, checkpoint_total_bytes: u64, interval_fsyncs: u64) -> f64 {
    (fsync_total_bytes as f64 * interval_fsyncs as f64 + checkpoint_total_bytes as f64) / interval_fsyncs as f64
}
