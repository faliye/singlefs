//! 文件族、几何与池形态、甲一次发布的结果与不动点核心的求解。

use e7_index_bench::truncate_f64_to_u64;
use super::*;

// ============================================================
// 四、族与几何：一格的输入
// ============================================================

/// 族（5.2）：F1 每个文件 1 个数据单元；F8A 一次 fsync 改 8 个相邻数据单元（D25 已定，问法 F8A，第十二节修订①）。
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

    /// 数据单元总数（extent 树条目数）：F1 每文件 1 个、F8A 每文件 8 个。
    pub(super) fn data_unit_count(self, file_count: u64) -> u64 {
        match self {
            Family::OneDataUnitPerFile => file_count,
            Family::EightAdjacentDataUnitsPerFsync => file_count * 8,
        }
    }

    /// 一次 fsync 改的数据单元数 `d`。
    pub(super) fn data_units_per_fsync(self) -> u64 {
        match self {
            Family::OneDataUnitPerFile => 1,
            Family::EightAdjacentDataUnitsPerFsync => 8,
        }
    }
}

/// 一格的几何输入（5.6）：主几何 φ=1、K3、g=24、pbs=512；反向取样点各自单独翻一个。
#[derive(Clone, Copy, Debug)]
pub(super) struct Geometry {
    pub(super) fill_ratio: FillRatio,
    pub(super) identity_reference: bool, // K3（false）/ K11（true）
    pub(super) backlog_generations: u64, // g：主 24，反向 0
    pub(super) physical_block_size_bytes: u64,
}

impl Geometry {
    pub(super) fn main() -> Geometry {
        Geometry {
            fill_ratio: FillRatio::Full,
            identity_reference: false,
            backlog_generations: 24,
            physical_block_size_bytes: PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES,
        }
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
    pub(super) named_items: u64,
    pub(super) record_count: u64,
    pub(super) extent_layers: Vec<u64>,
    /// 逐层期望脏节点数（长度 = `extent_layers.len()`），Q1.6 分子要用「非叶」那些层。
    pub(super) extent_dirty: Vec<f64>,
    pub(super) inode_layers: Vec<u64>,
    /// 逐层期望脏节点数；`inode_dirty[0]` 是容器（叶），`inode_dirty[1..]` 是码 2 内部 / 根。Q3.1 要按层拆开算 c。
    pub(super) inode_dirty: Vec<f64>,
    pub(super) allocation_layers: Vec<u64>,
    /// 逐层期望脏节点数（长度 = `allocation_layers.len()`）。Q3.1 要按层拆开算 c。
    pub(super) allocation_dirty: Vec<f64>,
    pub(super) mapping_layers: Vec<u64>,
    /// 逐层期望脏节点数（长度 = `mapping_layers.len()`）。Q3.1 要按层拆开算 c。
    pub(super) mapping_dirty: Vec<f64>,
    /// 记账树这次持久化的脏节点数（0 或 1，取决于是否被 Q2 抑制）。Q3.1 算记账的 c 要用它。
    pub(super) accounting_dirty: f64,
    /// 这次持久化写出的单元总数（不动点收敛值，未取整）。Q3.1 算分配记录、映射两棵树的 c 要用它。
    pub(super) written_units: f64,
    pub(super) fixed_point_iterations: u64,
}

impl JiaPublishOutcome {
    /// Q1.6 分子：extent、inode 两树的**非叶**节点 + 分配记录、记账、映射、树表四棵的**全部**脏节点 + 根槽。
    pub(super) fn ancestor_and_fixed_point_share_numerator_bytes(&self) -> u64 {
        let extent_non_leaf_dirty: f64 = self.extent_dirty[1..].iter().sum();
        truncate_f64_to_u64((extent_non_leaf_dirty * NODE_BYTES as f64).round()) * DEVICE_COUNT
            + self.inode_root_bytes
            + self.allocation_bytes
            + self.accounting_bytes
            + self.mapping_bytes
            + self.tree_table_bytes
            + self.root_slot_bytes
    }

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
    fn block_layer_write_requests(&self) -> u64 {
        self.write_calls + self.barriers + self.fua_count
    }
}

/// 求和一个 `Vec<f64>`。
pub(super) fn sum_f64(values: &[f64]) -> f64 {
    values.iter().sum()
}

/// 5.3 不动点：反复重算分配记录 / 映射两棵树的条目数、层数、脏节点数，直到相邻两轮变化都 < `CONVERGENCE_EPSILON`。
const CONVERGENCE_EPSILON: f64 = 1e-9;
pub(super) const MAXIMUM_FIXED_POINT_ITERATIONS: u64 = 1000;

/// 求一格（P、族、落点、位置策略、几何）甲一次 fsync 的写出（5.3 + 5.4 甲）。
/// 一次持久化对 extent / inode 树的碰法：连续组（fsync 内 / checkpoint 的 seq 间隔并集）
/// 或散组（checkpoint 的 rand 间隔并集，5.4 write_ahead_log_full / 乙 checkpoint 段）。
#[derive(Clone, Copy, Debug)]
pub(super) enum TouchSpecification {
    Continuous(f64),
    Scattered(f64),
}

fn touch_dirty_nodes_per_layer(layers: &[u64], leaf_capacity: u64, internal_capacity: u64, region_entries: f64, touch_specification: TouchSpecification) -> Vec<f64> {
    match touch_specification {
        TouchSpecification::Continuous(touch_count) if touch_count <= 0.0 => vec![0.0; layers.len()],
        TouchSpecification::Continuous(touch_count) => single_group_dirty_nodes_per_layer(layers, leaf_capacity, internal_capacity, truncate_f64_to_u64(touch_count.round())),
        TouchSpecification::Scattered(touch_count) if touch_count <= 0.0 => vec![0.0; layers.len()],
        TouchSpecification::Scattered(touch_count) => {
            let mut result = Vec::with_capacity(layers.len());
            let mut cov = leaf_capacity as f64;
            for &node_count_at_layer in layers {
                result.push(scattered_group_touch(touch_count, region_entries, cov).min(node_count_at_layer as f64));
                cov *= internal_capacity as f64;
            }
            result
        }
    }
}

/// 5.3 不动点算完的六棵树状态：`solve_jia_publish`（甲 fsync）与 checkpoint 段共用这一个核心。
pub(super) struct FixedPointCoreResult {
    pub(super) extent_layers: Vec<u64>,
    /// 逐层期望脏节点数（长度 = `extent_layers.len()`）。
    pub(super) extent_dirty: Vec<f64>,
    pub(super) inode_layers: Vec<u64>,
    /// 逐层期望脏节点数；`inode_dirty[0]` 是容器（叶），`inode_dirty[1..]` 是码 2 内部 / 根。
    pub(super) inode_dirty: Vec<f64>,
    pub(super) accounting_dirty_total: f64,
    pub(super) tree_table_dirty_total: f64,
    pub(super) allocation_layers: Vec<u64>,
    /// 逐层期望脏节点数（长度 = `allocation_layers.len()`）；Q2.1/Q3.1 要按层拆开算 c。
    pub(super) allocation_dirty: Vec<f64>,
    pub(super) mapping_layers: Vec<u64>,
    /// 逐层期望脏节点数（长度 = `mapping_layers.len()`）。
    pub(super) mapping_dirty: Vec<f64>,
    /// 六棵树（extent + inode + 四样固定点）这次的脏节点数之和：喂给下一轮迭代的 W、
    /// K6 积压公式的「每次持久化释放的单元数」，**不是**任何一个臂自己这次持久化实际写出的字节口径
    /// （write_ahead_log_full / 乙的 checkpoint 段不重写 extent / inode，自己的字节口径由调用处另算）。
    pub(super) written_units: f64,
    iterations: u64,
}

/// Q2：四样固定点各自的反事实——「X 不写」= X 的节点不写、不分配、不进映射、不点名，其余照甲重算不动点（登记 5.6/6）。
/// 每个字段独立取 `true`；主表（`SuppressedFixedPoints::default()`）四个都是 `false`，行为与甲完全相同。
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SuppressedFixedPoints {
    pub(super) accounting: bool,
    pub(super) allocation: bool,
    pub(super) mapping: bool,
    pub(super) tree_table: bool,
}

/// 5.3 不动点核心：extent / inode 用 `extent_touch` / `inode_touch` 指定这次持久化怎么碰它们
/// （甲 fsync：连续组 k=d/f；WAL checkpoint：N 次 fsync 并集，seq 连续、rand 散组）；
/// 记账、树表、分配记录、映射四样固定点每次都按甲的写法全部重算（K9 的净改动近似见调用处）。
#[allow(clippy::too_many_arguments, reason = "固定点核心的入参逐项对应登记公式里的变量，拆结构体不会让哪一步更好验")]
pub(super) fn solve_fixed_point_core(
    data_unit_count: u64,
    file_count: u64,
    extent_touch: TouchSpecification,
    inode_touch: TouchSpecification,
    // 这次持久化里**去重之后**碰到的数据单元数（映射树「数据单元」类的插 / 删条目数）：
    // 甲一次 fsync 恰好是 `d`（这 d 个单元互不相同）；WAL checkpoint 是 N 次 fsync 并集去重之后的数
    // （5.4：seq `min(Nd, 数据单元数)`、rand 用生日公式，等价于 `continuous_groups_touch` /
    // `scattered_group_touch` 在 `cov=1` 时的读数，调用处按 5.4 原文算好再传进来）。
    distinct_data_units_touched: f64,
    placement: Placement,
    policy: PositionPolicy,
    geometry: Geometry,
    suppressed: SuppressedFixedPoints,
    // 树表条目数（Q2.5：主几何 7，反事实取样点 81/82/6561/6562，D8 已定项 8「树表按每层 81 棵嵌套」，
    // 用与其它树相同的 `tree_layers` 公式算树表自己的高度）。
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
    assert_eq!(accounting_layers, vec![1], "15 行远小于 477 的叶容量，记账树恒 1 节点（K5）");
    // Q2：记账「不写」时这次持久化的记账脏节点数为 0（树仍在，只是这次不重写、不进映射、不点名）。
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
    // Q2：树表「不写」——树仍在（贡献 `live_units` 的节点数不变），这次持久化脏节点数强制为 0。
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
            + 1.0 // 记账树 1 节点
            + mapping_total_nodes as f64
            + tree_table_total_nodes as f64
            + 1.0; // 实例表 1

        let allocation_entries = device_count * (live_units + geometry.backlog_generations as f64 * previous_written_units);
        let allocation_layers_now = tree_layers(truncate_f64_to_u64(allocation_entries.round()), allocation_leaf_cap, allocation_internal_cap, 1);
        let allocation_touch = if allocation_layers_now.len() == previous_touch_allocation.len() {
            previous_touch_allocation.clone()
        } else {
            vec![0.0; allocation_layers_now.len()]
        };
        let per_device_allocation_entries = allocation_entries / device_count;
        let allocation_classes = [
            TreeClassActivity { region_entries: per_device_allocation_entries, insert_count: previous_written_units, delete_count: previous_written_units },
            TreeClassActivity { region_entries: per_device_allocation_entries, insert_count: previous_written_units, delete_count: previous_written_units },
        ];
        let allocation_dirty_now = multiclass_tree_dirty_nodes_per_layer(
            &allocation_layers_now,
            allocation_leaf_cap,
            allocation_internal_cap,
            &allocation_classes,
            placement,
            policy,
            &allocation_touch,
        );
        // Q2：分配记录「不写」——树仍在（entries/layers 正常算，供别处的「活单元」计数用），
        // 这次持久化的脏节点数强制为 0，不进 W、不作为映射里「分配记录树节点」类的插删来源。
        let allocation_dirty_now: Vec<f64> = if suppressed.allocation { vec![0.0; allocation_dirty_now.len()] } else { allocation_dirty_now };
        let allocation_dirty_total_now = sum_f64(&allocation_dirty_now);
        let allocation_total_nodes_now: u64 = allocation_layers_now.iter().sum();

        let mapping_entries = data_unit_count as f64
            + extent_total_nodes as f64
            + inode_total_nodes as f64
            + allocation_total_nodes_now as f64
            + 1.0; // 记账树 1 节点
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
            },
            TreeClassActivity { region_entries: extent_total_nodes as f64, insert_count: extent_dirty_total, delete_count: extent_dirty_total },
            TreeClassActivity {
                region_entries: (inode_total_nodes as f64 - inode_layers[0] as f64).max(inode_root_dirty_total),
                insert_count: inode_root_dirty_total,
                delete_count: inode_root_dirty_total,
            },
            TreeClassActivity { region_entries: allocation_total_nodes_now as f64, insert_count: allocation_dirty_total_now, delete_count: allocation_dirty_total_now },
            TreeClassActivity { region_entries: 1.0, insert_count: accounting_dirty_total, delete_count: accounting_dirty_total },
            TreeClassActivity { region_entries: inode_layers[0] as f64, insert_count: inode_container_dirty, delete_count: inode_container_dirty },
        ];
        let mapping_dirty_now = multiclass_tree_dirty_nodes_per_layer(
            &mapping_layers_now,
            mapping_leaf_cap,
            mapping_internal_cap,
            &mapping_classes,
            placement,
            policy,
            &mapping_touch,
        );
        // Q2：映射「不写」——同上，树仍在，这次持久化脏节点数强制为 0。
        let mapping_dirty_now: Vec<f64> = if suppressed.mapping { vec![0.0; mapping_dirty_now.len()] } else { mapping_dirty_now };
        let mapping_dirty_total_now = sum_f64(&mapping_dirty_now);

        let written_units_now = extent_dirty_total + inode_dirty_total + allocation_dirty_total_now + accounting_dirty_total + mapping_dirty_total_now + tree_table_dirty_total;

        let allocation_entries_delta = (allocation_entries - previous_allocation_entries).abs();
        let mapping_entries_delta = (mapping_entries - previous_mapping_entries).abs();
        let written_units_delta = (written_units_now - previous_written_units).abs();
        let allocation_dirty_delta = (allocation_dirty_total_now - sum_f64(&allocation_dirty)).abs();
        let mapping_dirty_delta = (mapping_dirty_total_now - sum_f64(&mapping_dirty)).abs();

        previous_touch_allocation = allocation_layers_now
            .iter()
            .zip(allocation_dirty_now.iter())
            .map(|(&node_count, &dirty)| if node_count == 0 { 0.0 } else { dirty / node_count as f64 })
            .collect();
        previous_touch_mapping = mapping_layers_now
            .iter()
            .zip(mapping_dirty_now.iter())
            .map(|(&node_count, &dirty)| if node_count == 0 { 0.0 } else { dirty / node_count as f64 })
            .collect();
        allocation_layers = allocation_layers_now;
        mapping_layers = mapping_layers_now;
        allocation_dirty = allocation_dirty_now;
        mapping_dirty = mapping_dirty_now;
        previous_allocation_entries = allocation_entries;
        previous_mapping_entries = mapping_entries;
        previous_written_units = written_units_now;

        let converged = allocation_entries_delta < CONVERGENCE_EPSILON
            && mapping_entries_delta < CONVERGENCE_EPSILON
            && written_units_delta < CONVERGENCE_EPSILON
            && allocation_dirty_delta < CONVERGENCE_EPSILON
            && mapping_dirty_delta < CONVERGENCE_EPSILON;
        if converged || iterations >= MAXIMUM_FIXED_POINT_ITERATIONS {
            break;
        }
    }

    FixedPointCoreResult {
        extent_layers,
        extent_dirty,
        inode_layers,
        inode_dirty,
        accounting_dirty_total,
        tree_table_dirty_total,
        allocation_layers,
        allocation_dirty,
        mapping_layers,
        mapping_dirty,
        written_units: previous_written_units,
        iterations,
    }
}

/// 求一格（P、族、落点、位置策略、几何）甲一次 fsync 的写出（5.3 + 5.4 甲），四样固定点都正常写、树表 7 条（主几何）。
pub(super) fn solve_jia_publish(shape: PoolShape) -> JiaPublishOutcome {
    solve_jia_publish_with_suppressed_fixed_points(shape, SuppressedFixedPoints::default())
}

/// Q2.1–Q2.4 反事实：`suppressed` 里标 `true` 的那样固定点，这次持久化不写、不分配、不进映射、不点名，
/// 其余照甲重算不动点（登记 5.6/六）；树表仍是主几何 7 条。`SuppressedFixedPoints::default()` 时与 `solve_jia_publish` 完全相同。
pub(super) fn solve_jia_publish_with_suppressed_fixed_points(shape: PoolShape, suppressed: SuppressedFixedPoints) -> JiaPublishOutcome {
    solve_jia_publish_with_suppressed_fixed_points_and_tree_count(shape, suppressed, TREE_TABLE_ENTRY_COUNT)
}

/// Q2.5 反事实：树表条目数换成 `tree_table_entry_count`（登记 5.6 第 2 行：T ∈ {7,81,82,6561,6562}），
/// 其余与 `solve_jia_publish_with_suppressed_fixed_points` 相同。
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

#[cfg(test)]
#[path = "family_geometry_and_jia_publish_tests.rs"]
mod tests;
