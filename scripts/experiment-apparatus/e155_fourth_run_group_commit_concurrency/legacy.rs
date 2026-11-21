//! 逐字节照抄第一次装置 `e155_fsync_write_volume.rs` 657–938 行的分配 / 映射树计算
//! （R1–R4 之前的样子），只用于 Q6.11「第三节 3.2 四处改动前后甲的差」。
//! extent / inode 两树的公式两次装置相同，这里直接复用外层的 `single_group_dirty_nodes_per_layer` /
//! `tree_layers` 等，不重复定义。
use e7_index_bench::truncate_f64_to_u64;
use super::*;

fn legacy_class_dirty_nodes_this_layer(region_entries: f64, layer_coverage: f64, insert_count: f64, delete_count: f64, placement: Placement, policy: PositionPolicy, previous_touch_fraction: f64) -> f64 {
    if insert_count <= 0.0 && delete_count <= 0.0 {
        return 0.0;
    }
    let node_count_at_layer_for_class = truncate_f64_to_u64((region_entries / layer_coverage).ceil().max(1.0));
    if node_count_at_layer_for_class <= 1 {
        return 1.0;
    }
    let far_fraction: f64 = match (policy, placement) {
        (PositionPolicy::Near, _) => 0.0,
        (PositionPolicy::Far, _) => 1.0,
        (PositionPolicy::Balanced, Placement::Sequential) => (insert_count / layer_coverage).min(1.0),
        // legacy：远的比例直接取 q（第三节 3.2 ②，未修订）。
        (PositionPolicy::Balanced, Placement::Random) => previous_touch_fraction.min(1.0),
    };
    let near_count = delete_count * (1.0 - far_fraction);
    let far_count = delete_count * far_fraction;
    let tail_group = insert_count + near_count;
    let continuous_touch = if far_fraction > 0.0 && placement == Placement::Sequential {
        let gap = (region_entries - tail_group - far_count).max(0.0);
        continuous_groups_touch(node_count_at_layer_for_class, layer_coverage, &[far_count, tail_group], &[gap])
    } else {
        continuous_groups_touch(node_count_at_layer_for_class, layer_coverage, &[tail_group], &[])
    };
    let scattered_touch = if far_fraction > 0.0 && placement == Placement::Random { scattered_group_touch(far_count, region_entries, layer_coverage) } else { 0.0 };
    let scattered_touches: &[f64] = if scattered_touch > 0.0 { &[scattered_touch] } else { &[] };
    combine_touches(node_count_at_layer_for_class, continuous_touch, scattered_touches)
}

struct LegacyClass {
    region_entries: f64,
    insert_count: f64,
    delete_count: f64,
}

fn legacy_multiclass(layers: &[u64], leaf_capacity: u64, internal_capacity: u64, classes: &[LegacyClass], placement: Placement, policy: PositionPolicy, previous_touch_fractions: &[f64]) -> Vec<f64> {
    let mut dirty_per_layer = Vec::with_capacity(layers.len());
    let mut cov = leaf_capacity as f64;
    for (layer_index, &node_count_at_layer) in layers.iter().enumerate() {
        let mut layer_sum = 0.0;
        for class in classes {
            layer_sum += legacy_class_dirty_nodes_this_layer(class.region_entries, cov, class.insert_count, class.delete_count, placement, policy, previous_touch_fractions[layer_index]);
        }
        dirty_per_layer.push(layer_sum.min(node_count_at_layer as f64));
        cov *= internal_capacity as f64;
    }
    dirty_per_layer
}

/// 逐字节照抄第一次装置的 `solve_fixed_point_core`（R1–R4 之前）。
#[allow(clippy::too_many_arguments, reason = "固定点核心的入参逐项对应登记公式里的变量，拆结构体不会让哪一步更好验")]
fn legacy_solve_fixed_point_core(
    data_unit_count: u64,
    file_count: u64,
    extent_touch: TouchSpecification,
    inode_touch: TouchSpecification,
    distinct_data_units_touched: f64,
    placement: Placement,
    policy: PositionPolicy,
    geometry: Geometry,
    tree_table_entry_count: u64,
) -> FixedPointCoreResult {
    let device_count = DEVICE_COUNT as f64;

    let extent_leaf_cap = geometry.leaf_capacity(node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES));
    let extent_internal_cap = geometry.internal_capacity(EXTENT_KEY_BYTES);
    let extent_layers = tree_layers(data_unit_count, extent_leaf_cap, extent_internal_cap, 1);
    let extent_dirty = touch_dirty_nodes_per_layer(&extent_layers, extent_leaf_cap, extent_internal_cap, data_unit_count as f64, extent_touch);
    let extent_dirty_total = sum_f64(&extent_dirty);

    let inode_leaf_cap = geometry.leaf_capacity(inode_container_capacity());
    let inode_internal_cap = geometry.leaf_capacity(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES));
    let inode_layers = tree_layers(file_count, inode_leaf_cap, inode_internal_cap, 2);
    let inode_dirty = touch_dirty_nodes_per_layer(&inode_layers, inode_leaf_cap, inode_internal_cap, file_count as f64, inode_touch);
    let inode_container_dirty = inode_dirty[0];
    let inode_root_dirty_total: f64 = inode_dirty[1..].iter().sum();
    let inode_dirty_total = inode_container_dirty + inode_root_dirty_total;

    let accounting_entries = ACCOUNTING_POOL_WIDE_ENTRIES + ACCOUNTING_ENTRIES_PER_DEVICE * DEVICE_COUNT;
    let accounting_leaf_cap = geometry.leaf_capacity(node_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_LEAF_ENTRY_BYTES));
    let accounting_internal_cap = geometry.internal_capacity(ACCOUNTING_KEY_BYTES);
    let accounting_layers = tree_layers(accounting_entries, accounting_leaf_cap, accounting_internal_cap, 1);
    assert_eq!(accounting_layers, vec![1]);
    let accounting_dirty_total = 1.0_f64;

    let tree_table_cap = geometry.leaf_capacity(node_capacity(TREE_TABLE_KEY_BYTES, TREE_TABLE_ENTRY_BYTES));
    let tree_table_layers = tree_layers(tree_table_entry_count, tree_table_cap, tree_table_cap, 1);
    let tree_table_dirty = touch_dirty_nodes_per_layer(&tree_table_layers, tree_table_cap, tree_table_cap, tree_table_entry_count as f64, TouchSpecification::Continuous(TREE_TABLE_TOUCHED_ROOTS as f64));
    let tree_table_total_nodes: u64 = tree_table_layers.iter().sum();
    let tree_table_dirty_total = sum_f64(&tree_table_dirty);

    let allocation_leaf_cap = geometry.leaf_capacity(node_capacity(ALLOCATION_KEY_BYTES, ALLOCATION_LEAF_ENTRY_BYTES));
    let allocation_internal_cap = geometry.internal_capacity(ALLOCATION_KEY_BYTES);
    let mapping_leaf_cap = geometry.leaf_capacity(node_capacity(MAPPING_KEY_BYTES, MAPPING_LEAF_ENTRY_BYTES));
    let mapping_internal_cap = geometry.internal_capacity(MAPPING_KEY_BYTES);

    let mut allocation_layers: Vec<u64> = vec![1];
    let mut mapping_layers: Vec<u64> = vec![1];
    let mut allocation_dirty: Vec<f64> = vec![0.0];
    let mut mapping_dirty: Vec<f64> = vec![0.0];
    let mut previous_touch_allocation: Vec<f64> = vec![0.0];
    let mut previous_touch_mapping: Vec<f64> = vec![0.0];
    let mut previous_written_units = extent_dirty_total + inode_dirty_total + accounting_dirty_total + tree_table_dirty_total;
    let mut previous_allocation_entries = 0.0_f64;
    let mut previous_mapping_entries = 0.0_f64;
    let mut iterations = 0u64;

    loop {
        iterations += 1;
        let allocation_total_nodes: u64 = allocation_layers.iter().sum();
        let mapping_total_nodes: u64 = mapping_layers.iter().sum();
        let extent_total_nodes: u64 = extent_layers.iter().sum();
        let inode_total_nodes: u64 = inode_layers.iter().sum();

        let live_units = data_unit_count as f64 + extent_total_nodes as f64 + inode_total_nodes as f64 + allocation_total_nodes as f64 + 1.0 + mapping_total_nodes as f64 + tree_table_total_nodes as f64 + 1.0;

        let allocation_entries = device_count * (live_units + geometry.backlog_generations as f64 * previous_written_units);
        let allocation_layers_now = tree_layers(truncate_f64_to_u64(allocation_entries.round()), allocation_leaf_cap, allocation_internal_cap, 1);
        let allocation_touch = if allocation_layers_now.len() == previous_touch_allocation.len() { previous_touch_allocation.clone() } else { vec![0.0; allocation_layers_now.len()] };
        let per_device_allocation_entries = allocation_entries / device_count;
        let allocation_classes = [
            LegacyClass { region_entries: per_device_allocation_entries, insert_count: previous_written_units, delete_count: previous_written_units },
            LegacyClass { region_entries: per_device_allocation_entries, insert_count: previous_written_units, delete_count: previous_written_units },
        ];
        let allocation_dirty_now = legacy_multiclass(&allocation_layers_now, allocation_leaf_cap, allocation_internal_cap, &allocation_classes, placement, policy, &allocation_touch);
        let allocation_dirty_total_now = sum_f64(&allocation_dirty_now);
        let allocation_total_nodes_now: u64 = allocation_layers_now.iter().sum();

        let mapping_entries = data_unit_count as f64 + extent_total_nodes as f64 + inode_total_nodes as f64 + allocation_total_nodes_now as f64 + 1.0;
        let mapping_layers_now = tree_layers(truncate_f64_to_u64(mapping_entries.round()), mapping_leaf_cap, mapping_internal_cap, 1);
        let mapping_touch = if mapping_layers_now.len() == previous_touch_mapping.len() { previous_touch_mapping.clone() } else { vec![0.0; mapping_layers_now.len()] };
        let mapping_classes = [
            LegacyClass { region_entries: data_unit_count as f64, insert_count: distinct_data_units_touched, delete_count: distinct_data_units_touched },
            LegacyClass { region_entries: extent_total_nodes as f64, insert_count: extent_dirty_total, delete_count: extent_dirty_total },
            LegacyClass { region_entries: (inode_total_nodes as f64 - inode_layers[0] as f64).max(inode_root_dirty_total), insert_count: inode_root_dirty_total, delete_count: inode_root_dirty_total },
            LegacyClass { region_entries: allocation_total_nodes_now as f64, insert_count: allocation_dirty_total_now, delete_count: allocation_dirty_total_now },
            LegacyClass { region_entries: 1.0, insert_count: accounting_dirty_total, delete_count: accounting_dirty_total },
            LegacyClass { region_entries: inode_layers[0] as f64, insert_count: inode_container_dirty, delete_count: inode_container_dirty },
        ];
        let mapping_dirty_now = legacy_multiclass(&mapping_layers_now, mapping_leaf_cap, mapping_internal_cap, &mapping_classes, placement, policy, &mapping_touch);
        let mapping_dirty_total_now = sum_f64(&mapping_dirty_now);

        let written_units_now = extent_dirty_total + inode_dirty_total + allocation_dirty_total_now + accounting_dirty_total + mapping_dirty_total_now + tree_table_dirty_total;

        let allocation_entries_delta = (allocation_entries - previous_allocation_entries).abs();
        let mapping_entries_delta = (mapping_entries - previous_mapping_entries).abs();
        let written_units_delta = (written_units_now - previous_written_units).abs();
        let allocation_dirty_delta = (allocation_dirty_total_now - sum_f64(&allocation_dirty)).abs();
        let mapping_dirty_delta = (mapping_dirty_total_now - sum_f64(&mapping_dirty)).abs();

        previous_touch_allocation = allocation_layers_now.iter().zip(allocation_dirty_now.iter()).map(|(&node_count, &dirty)| if node_count == 0 { 0.0 } else { dirty / node_count as f64 }).collect();
        previous_touch_mapping = mapping_layers_now.iter().zip(mapping_dirty_now.iter()).map(|(&node_count, &dirty)| if node_count == 0 { 0.0 } else { dirty / node_count as f64 }).collect();
        allocation_layers = allocation_layers_now;
        mapping_layers = mapping_layers_now;
        allocation_dirty = allocation_dirty_now;
        mapping_dirty = mapping_dirty_now;
        previous_allocation_entries = allocation_entries;
        previous_mapping_entries = mapping_entries;
        previous_written_units = written_units_now;

        let converged = allocation_entries_delta < CONVERGENCE_EPSILON && mapping_entries_delta < CONVERGENCE_EPSILON && written_units_delta < CONVERGENCE_EPSILON && allocation_dirty_delta < CONVERGENCE_EPSILON && mapping_dirty_delta < CONVERGENCE_EPSILON;
        if converged || iterations >= MAXIMUM_FIXED_POINT_ITERATIONS {
            break;
        }
    }

    FixedPointCoreResult { extent_layers, extent_dirty, inode_layers, inode_dirty, accounting_dirty_total, tree_table_dirty_total, allocation_layers, allocation_dirty, mapping_layers, mapping_dirty, written_units: previous_written_units, iterations }
}

/// legacy 甲：与第一次装置的 `solve_jia_publish` 逐字节相同，只为 Q6.11 对比用。
pub(super) fn legacy_solve_jia_publish(shape: PoolShape) -> JiaPublishOutcome {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();

    let core = legacy_solve_fixed_point_core(
        data_unit_count,
        shape.file_count,
        TouchSpecification::Continuous(fsync_data_unit_count as f64),
        TouchSpecification::Continuous(1.0),
        fsync_data_unit_count as f64,
        shape.placement,
        shape.policy,
        geometry,
        TREE_TABLE_ENTRY_COUNT,
    );

    let written_units = fsync_data_unit_count as f64 + core.written_units;
    let named_items = truncate_f64_to_u64(written_units.round());
    let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);
    let pbs = geometry.physical_block_size_bytes;
    let unit_write_calls = truncate_f64_to_u64(written_units.round()) * DEVICE_COUNT;
    let record_calls = record_count * DEVICE_COUNT;
    let total_calls = unit_write_calls + record_calls + 1 + DEVICE_COUNT;

    let extent_dirty_total = sum_f64(&core.extent_dirty);
    let inode_container_dirty = core.inode_dirty[0];
    let inode_root_dirty_total: f64 = core.inode_dirty[1..].iter().sum();
    let allocation_dirty_total = sum_f64(&core.allocation_dirty);
    let mapping_dirty_total = sum_f64(&core.mapping_dirty);

    JiaPublishOutcome {
        data_bytes: fsync_data_unit_count * UNIT_BYTES * DEVICE_COUNT,
        extent_bytes: truncate_f64_to_u64((extent_dirty_total * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        inode_container_bytes: truncate_f64_to_u64((inode_container_dirty * UNIT_BYTES as f64).round()) * DEVICE_COUNT,
        inode_root_bytes: truncate_f64_to_u64((inode_root_dirty_total * NODE_BYTES as f64).round()) * DEVICE_COUNT,
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
        named_items,
        record_count,
        extent_layers: core.extent_layers,
        extent_dirty: core.extent_dirty,
        inode_layers: core.inode_layers,
        inode_dirty: core.inode_dirty,
        allocation_layers: core.allocation_layers,
        allocation_dirty: core.allocation_dirty,
        mapping_layers: core.mapping_layers,
        mapping_dirty: core.mapping_dirty,
        accounting_dirty: core.accounting_dirty_total,
        written_units,
        fixed_point_iterations: core.iterations,
    }
}
