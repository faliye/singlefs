//! 家族与几何、池形状、甲的发布结果、不动点核的求解、批发布与每次 fsync 的摊销和间隔内的批数。

use e7_index_bench::truncate_f64_to_u64;
use super::*;

// ============================================================
// 四、族与几何：一格的输入（与第一次逐字相同，加 K10 开关的实际接线）
// ============================================================

/// 族（5.2）：F1 每个文件 1 个数据单元；F8A 一次 fsync 改 8 个相邻数据单元。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Family {
    OneDataUnitPerFile,
    EightAdjacentDataUnitsPerFsync,
}

impl Family {
    pub(super) fn tag(self) -> &'static str {
        match self {
            Family::OneDataUnitPerFile => "F1",
            Family::EightAdjacentDataUnitsPerFsync => "F8A",
        }
    }

    pub(super) fn data_unit_count(self, file_count: u64) -> u64 {
        match self {
            Family::OneDataUnitPerFile => file_count,
            Family::EightAdjacentDataUnitsPerFsync => file_count * 8,
        }
    }

    pub(super) fn data_units_per_fsync(self) -> u64 {
        match self {
            Family::OneDataUnitPerFile => 1,
            Family::EightAdjacentDataUnitsPerFsync => 8,
        }
    }
}

/// 一格的几何输入（5.6）：主几何 φ=1、K3、g=24、pbs=512；反向取样点各自单独翻一个。
/// `intermediate_version_policy`（K9 / K10）不放在这里——它只对 WAL checkpoint 有意义，
/// 是 `solve_write_ahead_log_checkpoint` 的独立参数（见 `IntermediateVersionPolicy`）。
#[derive(Clone, Copy, Debug)]
pub(super) struct Geometry {
    pub(super) fill_ratio: FillRatio,
    pub(super) identity_reference: bool, // K3（false）/ K11（true）
    pub(super) backlog_generations: u64, // g：主 24，反向 0
    pub(super) physical_block_size_bytes: u64,
}

impl Geometry {
    pub(super) fn main() -> Geometry {
        Geometry { fill_ratio: FillRatio::Full, identity_reference: false, backlog_generations: 24, physical_block_size_bytes: PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES }
    }

    pub(super) fn leaf_capacity(self, raw: u64) -> u64 {
        self.fill_ratio.effective_capacity(raw)
    }

    pub(super) fn internal_capacity(self, key_bytes: u64) -> u64 {
        self.fill_ratio.effective_capacity(node_capacity(key_bytes, internal_entry_bytes(key_bytes, self.identity_reference)))
    }
}

/// 一格的输入：池大小、族、落点方式、位置策略、几何。
#[derive(Clone, Copy, Debug)]
pub(super) struct PoolShape {
    pub(super) file_count: u64,
    pub(super) family: Family,
    pub(super) placement: Placement,
    pub(super) policy: PositionPolicy,
    pub(super) geometry: Geometry,
}

/// K9 / K10（岔路单第 6 行，登记 5.1 R7）：间隔内被换掉的中间版要不要各自补一条已释放的分配记录、
/// 进 K6 积压。只对 WAL checkpoint 有意义——甲每次 fsync 就是一次发布，没有中间版。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum IntermediateVersionPolicy {
    /// K9（第一次登记）：中间版不写分配记录、不进映射、不进积压。
    NotTracked,
    /// K10（这一次登记）：每个中间版每盘写一条已释放的分配记录，(a) 并进分配记录树区间尾部、
    /// (b) 进 K6 积压（每个 checkpoint 的释放数加 I）。
    ReleasedAndBacklogged,
}

// ============================================================
// 五、5.3 不动点：甲一次 fsync（= 一次发布）写出的东西
// ============================================================

/// 甲一次 fsync（= 一次发布）写出的东西，按角色分列，字节数已经乘过两块盘（除非另外注明）。
#[derive(Clone, Debug, Default)]
pub(super) struct JiaPublishOutcome {
    pub(super) data_bytes: u64,
    pub(super) extent_bytes: u64,
    pub(super) inode_container_bytes: u64,
    pub(super) inode_root_bytes: u64,
    pub(super) allocation_bytes: u64,
    pub(super) accounting_bytes: u64,
    pub(super) mapping_bytes: u64,
    pub(super) tree_table_bytes: u64,
    pub(super) record_bytes: u64,
    pub(super) root_slot_bytes: u64,
    pub(super) system_configuration_bytes: u64,
    pub(super) write_calls: u64,
    pub(super) barriers: u64,
    pub(super) fua_count: u64,
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    pub(super) named_items: u64,
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    pub(super) record_count: u64,
    pub(super) extent_layers: Vec<u64>,
    pub(super) extent_dirty: Vec<f64>,
    pub(super) inode_layers: Vec<u64>,
    pub(super) inode_dirty: Vec<f64>,
    pub(super) allocation_layers: Vec<u64>,
    pub(super) allocation_dirty: Vec<f64>,
    pub(super) mapping_layers: Vec<u64>,
    pub(super) mapping_dirty: Vec<f64>,
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    pub(super) accounting_dirty: f64,
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    pub(super) written_units: f64,
    pub(super) fixed_point_iterations: u64,
}

impl JiaPublishOutcome {
    pub(super) fn total_bytes(&self) -> u64 {
        self.data_bytes
            + self.extent_bytes
            + self.inode_container_bytes
            + self.inode_root_bytes
            + self.allocation_bytes
            + self.accounting_bytes
            + self.mapping_bytes
            + self.tree_table_bytes
            + self.record_bytes
            + self.root_slot_bytes
            + self.system_configuration_bytes
    }

    #[cfg(test)]
    pub(super) fn block_layer_write_requests(&self) -> u64 {
        self.write_calls + self.barriers + self.fua_count
    }
}

pub(super) fn sum_f64(values: &[f64]) -> f64 {
    values.iter().sum()
}

pub(super) const CONVERGENCE_EPSILON: f64 = 1e-9;
pub(super) const MAXIMUM_FIXED_POINT_ITERATIONS: u64 = 1000;

#[derive(Clone, Copy, Debug)]
pub(super) enum TouchSpecification {
    Continuous(f64),
    /// R6：散连续组——`group_count` 组、每组 `group_size` 条相邻。
    ScatteredContinuousGroups { group_count: f64, group_size: f64 },
    /// R8（第 7 行组提交，第一节「读法写死」）：`path_count` 条互不共享的路径，每条覆盖 `group_size`
    /// 条相邻记录，路径间空隙都 ≥ 这一层的覆盖（`unshared_paths_touch` 的公式）。
    UnsharedPaths { path_count: f64, group_size: f64 },
}

/// R8：concurrent_fsync_count 条互不共享的路径，每条覆盖 `group_size` 条相邻记录，组间空隙 ≥ `layer_coverage`
/// （第一节「读法写死」：第 l 层脏节点 = min(n_l, concurrent_fsync_count × (1 + (group_size − 1)/cov_l))）。
fn unshared_paths_touch(node_count_at_layer: u64, layer_coverage: f64, path_count: f64, group_size: f64) -> f64 {
    (path_count * (1.0 + (group_size - 1.0) / layer_coverage)).min(node_count_at_layer as f64)
}

pub(super) fn touch_dirty_nodes_per_layer(layers: &[u64], leaf_capacity: u64, internal_capacity: u64, region_entries: f64, touch_specification: TouchSpecification) -> Vec<f64> {
    match touch_specification {
        TouchSpecification::Continuous(touch_count) if touch_count <= 0.0 => vec![0.0; layers.len()],
        TouchSpecification::Continuous(touch_count) => single_group_dirty_nodes_per_layer(layers, leaf_capacity, internal_capacity, truncate_f64_to_u64(touch_count.round())),
        TouchSpecification::ScatteredContinuousGroups { group_count, .. } if group_count <= 0.0 => vec![0.0; layers.len()],
        TouchSpecification::ScatteredContinuousGroups { group_count, group_size } => {
            let mut result = Vec::with_capacity(layers.len());
            let mut cov = leaf_capacity as f64;
            for &node_count_at_layer in layers {
                result.push(scattered_continuous_groups_touch(node_count_at_layer, cov, group_count, group_size, region_entries));
                cov *= internal_capacity as f64;
            }
            result
        }
        TouchSpecification::UnsharedPaths { path_count, .. } if path_count <= 0.0 => vec![0.0; layers.len()],
        TouchSpecification::UnsharedPaths { path_count, group_size } => {
            let mut result = Vec::with_capacity(layers.len());
            let mut cov = leaf_capacity as f64;
            for &node_count_at_layer in layers {
                result.push(unshared_paths_touch(node_count_at_layer, cov, path_count, group_size));
                cov *= internal_capacity as f64;
            }
            result
        }
    }
}

/// 5.3 不动点算完的六棵树状态。
pub(super) struct FixedPointCoreResult {
    pub(super) extent_layers: Vec<u64>,
    pub(super) extent_dirty: Vec<f64>,
    pub(super) inode_layers: Vec<u64>,
    pub(super) inode_dirty: Vec<f64>,
    pub(super) accounting_dirty_total: f64,
    pub(super) tree_table_dirty_total: f64,
    pub(super) allocation_layers: Vec<u64>,
    pub(super) allocation_dirty: Vec<f64>,
    pub(super) mapping_layers: Vec<u64>,
    pub(super) mapping_dirty: Vec<f64>,
    pub(super) written_units: f64,
    pub(super) iterations: u64,
}

/// Q2：四样固定点各自的反事实——「X 不写」= X 的节点不写、不分配、不进映射、不点名。
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SuppressedFixedPoints {
    pub(super) accounting: bool,
    pub(super) allocation: bool,
    pub(super) mapping: bool,
    pub(super) tree_table: bool,
}

/// 5.3 不动点核心（R1/R3/R4/R7 在这一版改过；R2 在 `class_dirty_nodes_this_layer` 里已经改）：
/// extent / inode 用 `extent_touch` / `inode_touch` 指定这次持久化怎么碰它们；
/// `extra_allocation_release_insertions`：K10（R7）每个中间版每盘写一条已释放的分配记录，
/// 只并进分配记录树尾部插入、进 K6 积压，K9 与甲恒传 0。
#[allow(clippy::too_many_arguments, reason = "固定点核心的入参逐项对应登记公式里的变量，拆结构体不会让哪一步更好验")]
pub(super) fn solve_fixed_point_core(
    data_unit_count: u64,
    file_count: u64,
    extent_touch: TouchSpecification,
    inode_touch: TouchSpecification,
    distinct_data_units_touched: f64,
    placement: Placement,
    policy: PositionPolicy,
    geometry: Geometry,
    suppressed: SuppressedFixedPoints,
    tree_table_entry_count: u64,
    extra_allocation_release_insertions: f64,
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
    assert_eq!(accounting_layers, vec![1], "15 行远小于 477 的叶容量，记账树恒 1 节点（K5）");
    let accounting_dirty_total = if suppressed.accounting { 0.0_f64 } else { 1.0_f64 };

    let tree_table_cap = geometry.leaf_capacity(node_capacity(TREE_TABLE_KEY_BYTES, TREE_TABLE_ENTRY_BYTES));
    let tree_table_layers = tree_layers(tree_table_entry_count, tree_table_cap, tree_table_cap, 1);
    let tree_table_dirty = touch_dirty_nodes_per_layer(
        &tree_table_layers,
        tree_table_cap,
        tree_table_cap,
        tree_table_entry_count as f64,
        TouchSpecification::Continuous(TREE_TABLE_TOUCHED_ROOTS as f64),
    );
    let tree_table_total_nodes: u64 = tree_table_layers.iter().sum();
    let tree_table_dirty_total = if suppressed.tree_table { 0.0_f64 } else { sum_f64(&tree_table_dirty) };

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
    // `previous_written_units`：不含数据单元（与第一次相同的口径），只用于「节点写」子类与外部 `written_units` 字段。
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

        let live_units = data_unit_count as f64
            + extent_total_nodes as f64
            + inode_total_nodes as f64
            + allocation_total_nodes as f64
            + 1.0
            + mapping_total_nodes as f64
            + tree_table_total_nodes as f64
            + 1.0;

        // R1（第三节 3.2 ①）：K6 积压的「每次持久化释放的单元数」要含数据单元；K10（R7(b)）再加中间版 I。
        let release_backlog_units = previous_written_units + distinct_data_units_touched + extra_allocation_release_insertions;
        let allocation_entries = device_count * (live_units + geometry.backlog_generations as f64 * release_backlog_units);
        let allocation_layers_now = tree_layers(truncate_f64_to_u64(allocation_entries.round()), allocation_leaf_cap, allocation_internal_cap, 1);
        let allocation_touch = if allocation_layers_now.len() == previous_touch_allocation.len() {
            previous_touch_allocation.clone()
        } else {
            vec![0.0; allocation_layers_now.len()]
        };
        let per_device_allocation_entries = allocation_entries / device_count;
        let allocation_dirty_now: Vec<f64> = {
            let mut result = Vec::with_capacity(allocation_layers_now.len());
            let mut cov = allocation_leaf_cap as f64;
            for (layer_index, &node_count) in allocation_layers_now.iter().enumerate() {
                let per_device_dirty = allocation_tree_dirty_this_layer(
                    per_device_allocation_entries,
                    cov,
                    previous_written_units,
                    distinct_data_units_touched,
                    data_unit_count as f64,
                    extra_allocation_release_insertions,
                    placement,
                    policy,
                    allocation_touch[layer_index],
                );
                result.push((per_device_dirty * device_count).min(node_count as f64));
                cov *= allocation_internal_cap as f64;
            }
            result
        };
        let allocation_dirty_now: Vec<f64> = if suppressed.allocation { vec![0.0; allocation_dirty_now.len()] } else { allocation_dirty_now };
        let allocation_dirty_total_now = sum_f64(&allocation_dirty_now);
        let allocation_total_nodes_now: u64 = allocation_layers_now.iter().sum();

        let mapping_entries = data_unit_count as f64 + extent_total_nodes as f64 + inode_total_nodes as f64 + allocation_total_nodes_now as f64 + 1.0;
        let mapping_layers_now = tree_layers(truncate_f64_to_u64(mapping_entries.round()), mapping_leaf_cap, mapping_internal_cap, 1);
        let mapping_touch = if mapping_layers_now.len() == previous_touch_mapping.len() {
            previous_touch_mapping.clone()
        } else {
            vec![0.0; mapping_layers_now.len()]
        };
        let mapping_classes = [
            TreeClassActivity {
                region_entries: data_unit_count as f64,
                insert_count: distinct_data_units_touched,
                delete_count: distinct_data_units_touched,
                // R3：映射树「数据单元」类，远近只看总量 vs 触达量。
                data_unit_totals: Some((data_unit_count as f64, distinct_data_units_touched)),
            },
            TreeClassActivity { region_entries: extent_total_nodes as f64, insert_count: extent_dirty_total, delete_count: extent_dirty_total, data_unit_totals: None },
            TreeClassActivity {
                region_entries: (inode_total_nodes as f64 - inode_layers[0] as f64).max(inode_root_dirty_total),
                insert_count: inode_root_dirty_total,
                delete_count: inode_root_dirty_total,
                data_unit_totals: None,
            },
            TreeClassActivity {
                region_entries: allocation_total_nodes_now as f64,
                insert_count: allocation_dirty_total_now,
                delete_count: allocation_dirty_total_now,
                data_unit_totals: None,
            },
            TreeClassActivity { region_entries: 1.0, insert_count: accounting_dirty_total, delete_count: accounting_dirty_total, data_unit_totals: None },
            TreeClassActivity { region_entries: inode_layers[0] as f64, insert_count: inode_container_dirty, delete_count: inode_container_dirty, data_unit_totals: None },
        ];
        let mapping_dirty_now = multiclass_tree_dirty_nodes_per_layer(&mapping_layers_now, mapping_leaf_cap, mapping_internal_cap, &mapping_classes, placement, policy, &mapping_touch);
        let mapping_dirty_now: Vec<f64> = if suppressed.mapping { vec![0.0; mapping_dirty_now.len()] } else { mapping_dirty_now };
        let mapping_dirty_total_now = sum_f64(&mapping_dirty_now);

        let written_units_now = extent_dirty_total + inode_dirty_total + allocation_dirty_total_now + accounting_dirty_total + mapping_dirty_total_now + tree_table_dirty_total;

        // 相对误差，不是绝对误差：P 从 1 到 1 亿，`allocation_entries` 等量的量级也跟着从个位涨到
        // 千万位，绝对 1e-9 在大 P 上超出 f64 chained 算术的可分辨精度（实测 P=10^6–10^8、F8A、rand，
        // 相邻两轮在第 15 位小数打转，永远到不了绝对 1e-9，触发的不是 F2 说的『重写一大半』，是浮点噪声）；
        // 相对误差在小 P（值本身是个位数）与大 P（值是千万级）上要求的是同一件事：约 9 位有效数字一致。
        let allocation_entries_delta = (allocation_entries - previous_allocation_entries).abs() / allocation_entries.max(1.0);
        let mapping_entries_delta = (mapping_entries - previous_mapping_entries).abs() / mapping_entries.max(1.0);
        let written_units_delta = (written_units_now - previous_written_units).abs() / written_units_now.max(1.0);
        let allocation_dirty_delta = (allocation_dirty_total_now - sum_f64(&allocation_dirty)).abs() / allocation_dirty_total_now.max(1.0);
        let mapping_dirty_delta = (mapping_dirty_total_now - sum_f64(&mapping_dirty)).abs() / mapping_dirty_total_now.max(1.0);

        // R2 的「1 − q」是个倒转映射，原样把上一轮的读数喂给下一轮在大树上会撞出 2-周期震荡
        // （不动点从来到不了，`written_units` 在两个值之间来回跳，实测 P=100 万、F8A、rand、
        // Balanced 一格）；按 0.5 阻尼（successive under-relaxation）与上一轮的值取平均再喂下去——
        // 到达不动点时阻尼前后同值，不改变收敛到的答案，只去掉震荡。未阻尼的行为不是判据，
        // 单测只钉收敛到的绝对值。
        const DAMPING_ALPHA: f64 = 0.15;
        let raw_touch_allocation: Vec<f64> = allocation_layers_now.iter().zip(allocation_dirty_now.iter()).map(|(&node_count, &dirty)| if node_count == 0 { 0.0 } else { dirty / node_count as f64 }).collect();
        previous_touch_allocation = if raw_touch_allocation.len() == previous_touch_allocation.len() {
            raw_touch_allocation.iter().zip(previous_touch_allocation.iter()).map(|(&raw, &previous)| DAMPING_ALPHA * raw + (1.0 - DAMPING_ALPHA) * previous).collect()
        } else {
            raw_touch_allocation
        };
        let raw_touch_mapping: Vec<f64> = mapping_layers_now.iter().zip(mapping_dirty_now.iter()).map(|(&node_count, &dirty)| if node_count == 0 { 0.0 } else { dirty / node_count as f64 }).collect();
        previous_touch_mapping = if raw_touch_mapping.len() == previous_touch_mapping.len() {
            raw_touch_mapping.iter().zip(previous_touch_mapping.iter()).map(|(&raw, &previous)| DAMPING_ALPHA * raw + (1.0 - DAMPING_ALPHA) * previous).collect()
        } else {
            raw_touch_mapping
        };
        allocation_layers = allocation_layers_now;
        mapping_layers = mapping_layers_now;
        allocation_dirty = allocation_dirty_now;
        mapping_dirty = mapping_dirty_now;
        previous_allocation_entries = allocation_entries;
        previous_mapping_entries = mapping_entries;
        // 阻尼同样套在 `written_units`（喂 K6 积压与下一轮 `live_units` 的量）：`tree_layers` 的
        // `ceil` 在条目数正好卡在容量边界时会让节点数在两个整数间来回跳，级联出与 R2 无关的震荡
        // （实测 F8A、rand、Lfar、P=100 万–1 亿，与 policy 无关，Far 分支不读 q）；到达不动点时
        // 阻尼前后同值，不改变收敛到的答案。
        previous_written_units = DAMPING_ALPHA * written_units_now + (1.0 - DAMPING_ALPHA) * previous_written_units;

        let converged =
            allocation_entries_delta < CONVERGENCE_EPSILON && mapping_entries_delta < CONVERGENCE_EPSILON && written_units_delta < CONVERGENCE_EPSILON && allocation_dirty_delta < CONVERGENCE_EPSILON && mapping_dirty_delta < CONVERGENCE_EPSILON;
        if converged || iterations >= MAXIMUM_FIXED_POINT_ITERATIONS {
            break;
        }
    }

    FixedPointCoreResult { extent_layers, extent_dirty, inode_layers, inode_dirty, accounting_dirty_total, tree_table_dirty_total, allocation_layers, allocation_dirty, mapping_layers, mapping_dirty, written_units: previous_written_units, iterations }
}

pub(super) fn solve_jia_publish(shape: PoolShape) -> JiaPublishOutcome {
    solve_jia_publish_with_suppressed_fixed_points(shape, SuppressedFixedPoints::default())
}

pub(super) fn solve_jia_publish_with_suppressed_fixed_points(shape: PoolShape, suppressed: SuppressedFixedPoints) -> JiaPublishOutcome {
    solve_jia_publish_with_suppressed_fixed_points_and_tree_count(shape, suppressed, TREE_TABLE_ENTRY_COUNT)
}

pub(super) fn solve_jia_publish_with_suppressed_fixed_points_and_tree_count(shape: PoolShape, suppressed: SuppressedFixedPoints, tree_table_entry_count: u64) -> JiaPublishOutcome {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();

    let core = solve_fixed_point_core(
        data_unit_count,
        shape.file_count,
        TouchSpecification::Continuous(fsync_data_unit_count as f64),
        TouchSpecification::Continuous(1.0),
        fsync_data_unit_count as f64,
        shape.placement,
        shape.policy,
        geometry,
        suppressed,
        tree_table_entry_count,
        0.0, // 甲没有中间版（K10 只对 WAL 两臂有意义）。
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

/// R8（第 7 行组提交，登记 5.1 R8 + 第一节「读法写死」）：甲把 `concurrent_fsync_count` 个并发 fsync 并成一批 = 一次发布；
/// `shared` = 这 concurrent_fsync_count 个文件是不是在 key 上相邻（true：共享脊柱，一个连续组；false：互不共享，concurrent_fsync_count 条路径）。
pub(super) fn solve_jia_batch_publish(shape: PoolShape, concurrent_fsync_count: u64, shared: bool) -> JiaPublishOutcome {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let batch_data_unit_count = concurrent_fsync_count * fsync_data_unit_count;

    let extent_touch = if shared {
        TouchSpecification::Continuous(batch_data_unit_count as f64)
    } else {
        TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: fsync_data_unit_count as f64 }
    };
    let inode_touch = if shared {
        TouchSpecification::Continuous(concurrent_fsync_count as f64)
    } else {
        TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: 1.0 }
    };

    let core = solve_fixed_point_core(
        data_unit_count,
        shape.file_count,
        extent_touch,
        inode_touch,
        batch_data_unit_count as f64,
        shape.placement,
        shape.policy,
        geometry,
        SuppressedFixedPoints::default(),
        TREE_TABLE_ENTRY_COUNT,
        0.0, // 甲没有中间版。
    );

    let batch_written_units = batch_data_unit_count as f64 + core.written_units;
    let named_items = truncate_f64_to_u64(batch_written_units.round());
    let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);

    let pbs = geometry.physical_block_size_bytes;
    let unit_write_calls = truncate_f64_to_u64(batch_written_units.round()) * DEVICE_COUNT;
    let record_calls = record_count * DEVICE_COUNT;
    let batch_write_calls = unit_write_calls + record_calls + 1 + DEVICE_COUNT;

    let extent_dirty_total = sum_f64(&core.extent_dirty);
    let inode_container_dirty = core.inode_dirty[0];
    let inode_root_dirty_total: f64 = core.inode_dirty[1..].iter().sum();
    let allocation_dirty_total = sum_f64(&core.allocation_dirty);
    let mapping_dirty_total = sum_f64(&core.mapping_dirty);

    JiaPublishOutcome {
        data_bytes: batch_data_unit_count * UNIT_BYTES * DEVICE_COUNT,
        extent_bytes: truncate_f64_to_u64((extent_dirty_total * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        inode_container_bytes: truncate_f64_to_u64((inode_container_dirty * UNIT_BYTES as f64).round()) * DEVICE_COUNT,
        inode_root_bytes: truncate_f64_to_u64((inode_root_dirty_total * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        // R8 一批只写一份固定点、一条记录组、一个根槽、一份系统配置（M9 的反例：每个 fsync 各写一份）。
        allocation_bytes: truncate_f64_to_u64((allocation_dirty_total * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        accounting_bytes: truncate_f64_to_u64((core.accounting_dirty_total * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        mapping_bytes: truncate_f64_to_u64((mapping_dirty_total * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        tree_table_bytes: truncate_f64_to_u64((core.tree_table_dirty_total * NODE_BYTES as f64).round()) * DEVICE_COUNT,
        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,
        root_slot_bytes: pbs,
        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT,
        write_calls: batch_write_calls,
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
        written_units: batch_written_units,
        fixed_point_iterations: core.iterations,
    }
}

/// 第 7 行：每次 fsync 摊到的写字节 / 写请求（甲 = 一批的写 ÷ concurrent_fsync_count；M16 的反例：不除以 concurrent_fsync_count）。
pub(super) fn amortize_batch_per_fsync(batch_value: u64, concurrent_fsync_count: u64) -> f64 {
    batch_value as f64 / concurrent_fsync_count as f64
}

/// 第 7 行：checkpoint 间隔 N 按 fsync 计——一个间隔 N 次 fsync = N/concurrent_fsync_count 批
/// （第一节「读法写死」；M13 的反例：间隔按批计，直接把 N 当批数）。
#[cfg(test)]
pub(super) fn batches_in_interval_by_fsync_count(interval_fsyncs: u64, concurrent_fsync_count: u64) -> u64 {
    interval_fsyncs / concurrent_fsync_count
}

/// R13（第四次跑登记第一节「N_b 在 k > N 时怎么算」）：k ≤ N 时 N_b = N/concurrent_fsync_count；
/// k > N 时一批已经把间隔撑满，N_b = 1（B-r4-9）。M1 的变异目标。
pub(super) fn batches_in_interval_for_concurrent_fsync_count(interval_fsyncs: u64, concurrent_fsync_count: u64) -> u64 {
    if concurrent_fsync_count > interval_fsyncs {
        1
    } else {
        interval_fsyncs / concurrent_fsync_count
    }
}

/// R13：k ≤ N 时间隔实际 fsync 数 = N（摊销分母不变）；k > N 时间隔实际 fsync 数 = concurrent_fsync_count，
/// 摊销分母跟着从 N 换成 concurrent_fsync_count（B-r4-9）。M6 的变异目标。
pub(super) fn effective_fsyncs_in_interval(interval_fsyncs: u64, concurrent_fsync_count: u64) -> u64 {
    if concurrent_fsync_count > interval_fsyncs {
        concurrent_fsync_count
    } else {
        interval_fsyncs
    }
}

/// Q7.16：某一层「不共享脏节点 = n_l」即饱和，返回饱和的层号（从 0 开始，叶层 = 0）。
/// `node_count == 1` 的层（多数树的根）排除在外：单节点层不管碰没碰都必然「脏节点 = n_l」，
/// 这是树结构本身的平凡事实，跟不共享路径数够不够多没有关系，不算这一条要判的「饱和」
/// （实测：P=10⁴、F1、k=1024，共享与不共享的 extent 根层都会命中这个平凡情形，
/// 若不排除，「共享不该饱和」这条阳性对照会被平凡命中一起打红）。
pub(super) fn saturated_layer_indices(layers: &[u64], dirty: &[f64]) -> Vec<usize> {
    layers
        .iter()
        .zip(dirty.iter())
        .enumerate()
        .filter_map(|(layer_index, (&node_count, &dirty_value))| if node_count > 1 && (dirty_value - node_count as f64).abs() < 1e-9 { Some(layer_index) } else { None })
        .collect()
}
