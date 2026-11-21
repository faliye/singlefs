//! 触及的叶数、旋钮、容器树与它们的层数与改写节点数、一个几何的全部派生量。

use super::*;

// ── 几何（登记第五节「几何与派生量」；第一段与第二段同一套） ──

/// 映射树与分配记录树被碰到的叶取哪一档（读法第 6 条）：分散 = 改动的 key 依次落在不同的叶，集中 = 连号落在最右那几片。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum TouchedLeaves {
    Spread,
    Focused,
}

impl TouchedLeaves {
    pub(super) fn label(self) -> &'static str {
        match self {
            TouchedLeaves::Spread => "spread",
            TouchedLeaves::Focused => "focused",
        }
    }
}

/// 一组旋钮 G = (u, a_T, R_X, a_X, R_L, a_L, I, D, s, φ, 碰叶档)，另带乙2 的 b、g。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct Knobs {
    pub(super) tombstone_records: u64,
    pub(super) tombstones_added: u64,
    pub(super) retirement_records: u64,
    pub(super) retirements_added: u64,
    pub(super) lineage_records: u64,
    pub(super) lineage_records_added: u64,
    pub(super) live_inodes: u64,
    pub(super) file_data_units: u64,
    pub(super) settle_rounds: u64,
    pub(super) mapping_leaves_half_full: bool,
    pub(super) touched_leaves: TouchedLeaves,
    pub(super) persistent_fanout: u64,
    pub(super) persistent_leaf_block: u64,
}

impl Knobs {
    /// G0：u = 10⁶、a_T = 1、R_X = 10³、a_X = 0、R_L = 16、a_L = 0、I = 10³、D = 0、s = 4、φ = 1、分散档；乙2 b = 32、g = 64。
    pub(super) fn main_geometry() -> Knobs {
        Knobs {
            tombstone_records: 1_000_000,
            tombstones_added: 1,
            retirement_records: 1000,
            retirements_added: 0,
            lineage_records: 16,
            lineage_records_added: 0,
            live_inodes: 1000,
            file_data_units: 0,
            settle_rounds: 4,
            mapping_leaves_half_full: false,
            touched_leaves: TouchedLeaves::Spread,
            persistent_fanout: 32,
            persistent_leaf_block: 64,
        }
    }

    /// G-small（方向相反的点）：u = 10、R_X = 0、R_L = 0、I = 1、s = 1，其余取 G0。
    pub(super) fn small_geometry() -> Knobs {
        Knobs { tombstone_records: 10, retirement_records: 0, lineage_records: 0, live_inodes: 1, settle_rounds: 1, ..Knobs::main_geometry() }
    }
}

/// 一棵容器树（墓碑、退役、谱系）这一版与下一版的形状：左满右开，只有最右那条路不满。
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct ContainerTree {
    pub(super) records: u64,
    pub(super) added: u64,
    pub(super) containers: u64,
    pub(super) levels: Vec<u64>,
    pub(super) nodes: u64,
    pub(super) height: u64,
    pub(super) fill: u64,
    pub(super) open_container_rewritten: u64,
    pub(super) new_containers: u64,
    pub(super) rewritten_index_nodes: u64,
    pub(super) appends: bool,
    pub(super) records_after: u64,
    pub(super) containers_after: u64,
    pub(super) levels_after: Vec<u64>,
    pub(super) nodes_after: u64,
    pub(super) height_after: u64,
}

/// 按扇出往上建的每层节点数，从下往上；count = 0 时没有层。
fn levels_by_fanout(count: u64, bottom_capacity: u64, fanout: u64) -> Vec<u64> {
    if count == 0 {
        return Vec::new();
    }
    let mut levels = vec![count.div_ceil(bottom_capacity)];
    while let Some(&top) = levels.last() {
        if top <= 1 {
            break;
        }
        levels.push(top.div_ceil(fanout));
    }
    levels
}

/// 按碰到的叶数往上数重写的节点：x_1 = 碰到的叶，x_ℓ = min(n_ℓ, x_{ℓ−1})。
fn rewritten_nodes_by_level(levels: &[u64], touched_leaves: u64) -> u64 {
    let mut reaching = touched_leaves;
    let mut rewritten = 0;
    for &nodes_at_level in levels {
        reaching = reaching.min(nodes_at_level);
        rewritten += reaching;
    }
    rewritten
}

impl ContainerTree {
    pub(super) fn new(records: u64, added: u64, capacity: u64, appends_every_publish: bool) -> ContainerTree {
        let containers = records.div_ceil(capacity);
        let levels = levels_by_fanout(containers, CODE_TWO_FANOUT, CODE_TWO_FANOUT);
        let fill = if containers > 0 { records - (containers - 1) * capacity } else { 0 };
        let room = if containers > 0 { capacity - fill } else { 0 };
        let open_container_rewritten = u64::from(added > 0 && containers > 0 && room > 0);
        let new_containers = if added > 0 && added > room { (added - room).div_ceil(capacity) } else { 0 };
        let leftmost_touched_container = if open_container_rewritten == 1 { containers - 1 } else { containers };
        let mut rewritten_index_nodes = 0;
        if added > 0 {
            let mut span = CODE_TWO_FANOUT;
            for &nodes_at_level in &levels {
                rewritten_index_nodes += nodes_at_level.saturating_sub(leftmost_touched_container / span);
                span = span.saturating_mul(CODE_TWO_FANOUT);
            }
        }
        let records_after = records + added;
        let containers_after = records_after.div_ceil(capacity);
        let levels_after = levels_by_fanout(containers_after, CODE_TWO_FANOUT, CODE_TWO_FANOUT);
        ContainerTree {
            records,
            added,
            containers,
            nodes: levels.iter().sum(),
            height: levels.len() as u64,
            levels,
            fill,
            open_container_rewritten,
            new_containers,
            rewritten_index_nodes,
            appends: appends_every_publish || added > 0,
            records_after,
            containers_after,
            nodes_after: levels_after.iter().sum(),
            height_after: levels_after.len() as u64,
            levels_after,
        }
    }

    pub(super) fn shape_bytes(&self) -> u64 {
        shape_bytes(self.nodes, self.height)
    }
}

/// 容器树形状 shape(N, h) = 8N + 24h。
pub(super) fn shape_bytes(nodes: u64, height: u64) -> u64 {
    SHAPE_NODE_BYTES * nodes + SHAPE_LEVEL_BYTES * height
}

/// 多层码 2 树形状 shape_ml(E, N) = 32N + E·W_ck + (N − 1)(W_ck + 8)；N = 0 时 0。
pub(super) fn multi_level_shape_bytes(entries: u64, nodes: u64, kind: MultiLevelTreeKind) -> u64 {
    if nodes == 0 {
        return 0;
    }
    let key_structure = key_structure_bytes(kind);
    MULTI_LEVEL_SHAPE_NODE_BYTES * nodes + entries * key_structure + (nodes - 1) * (key_structure + MULTI_LEVEL_CHILD_POSITION_BYTES)
}

/// 一组旋钮上这一版与下一版的全部派生量（登记第五节「几何与派生量」逐条；「_after」= 这一版之后）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct Geometry {
    pub(super) knobs: Knobs,
    pub(super) tombstones: ContainerTree,
    pub(super) retirements: ContainerTree,
    pub(super) lineage: ContainerTree,
    pub(super) inode_leaves: u64,
    pub(super) core_rewritten_units: u64,
    pub(super) core_new_units: u64,
    pub(super) slots_per_device: u64,
    pub(super) allocation_levels: Vec<u64>,
    pub(super) allocation_nodes: u64,
    pub(super) allocation_levels_after: Vec<u64>,
    pub(super) allocation_nodes_after: u64,
    pub(super) allocation_changed_records: u64,
    pub(super) allocation_touched_leaves: u64,
    pub(super) allocation_rewritten_nodes: u64,
    pub(super) allocation_carried_nodes: u64,
    pub(super) mapping_leaf_entries: u64,
    pub(super) mapping_entries: u64,
    pub(super) mapping_entries_after: u64,
    pub(super) mapping_rewritten_units: u64,
    pub(super) mapping_new_entries: u64,
    pub(super) mapping_changed_entries: u64,
    pub(super) mapping_levels: Vec<u64>,
    pub(super) mapping_nodes: u64,
    pub(super) mapping_levels_after: Vec<u64>,
    pub(super) mapping_nodes_after: u64,
    pub(super) mapping_touched_leaves: u64,
    pub(super) mapping_rewritten_nodes: u64,
    pub(super) mapping_carried_nodes: u64,
    pub(super) units: u64,
    pub(super) units_after: u64,
    pub(super) rewritten_units: u64,
    pub(super) new_units: u64,
    pub(super) allocation_records: u64,
    pub(super) allocation_records_after: u64,
    pub(super) deleted_inodes_after: u64,
}

/// 分配记录树每块盘用到的槽 Sl = 2·(Σ C_t + L_I + D + P_IT) + (Σ N_t + F)；叶 n_1 = 2·⌈Sl ÷ 812⌉，往上扇出 169。
fn allocation_record_tree_levels(containers: u64, index_nodes: u64, inode_leaves: u64, file_data_units: u64) -> (u64, Vec<u64>) {
    let slots = 2 * (containers + inode_leaves + file_data_units + INSTANCE_TABLE_UNITS) + (index_nodes + FIXED_POINT_NODES);
    let leaves = DEVICE_COUNT * slots.div_ceil(ALLOCATION_RECORD_TREE_LEAF_SLOTS);
    let mut levels = vec![leaves];
    while let Some(&top) = levels.last() {
        if top <= 1 {
            break;
        }
        levels.push(top.div_ceil(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT));
    }
    (slots, levels)
}

impl Geometry {
    pub(super) fn new(knobs: Knobs) -> Geometry {
        let tombstones = ContainerTree::new(knobs.tombstone_records, knobs.tombstones_added, TOMBSTONE_CONTAINER_RECORDS, false);
        let retirements = ContainerTree::new(knobs.retirement_records, knobs.retirements_added, RETIREMENT_CONTAINER_RECORDS, true);
        let lineage = ContainerTree::new(knobs.lineage_records, knobs.lineage_records_added, LINEAGE_CONTAINER_RECORDS, false);
        let trees = [&tombstones, &retirements, &lineage];
        let inode_leaves = knobs.live_inodes.div_ceil(INODE_LEAF_RECORDS).max(1);
        let containers: u64 = trees.iter().map(|tree| tree.containers).sum();
        let index_nodes: u64 = trees.iter().map(|tree| tree.nodes).sum();
        let containers_after: u64 = trees.iter().map(|tree| tree.containers_after).sum();
        let index_nodes_after: u64 = trees.iter().map(|tree| tree.nodes_after).sum();
        let core_units = containers + index_nodes + inode_leaves + knobs.file_data_units + FIXED_POINT_NODES;
        let core_units_after = containers_after + index_nodes_after + inode_leaves + knobs.file_data_units + FIXED_POINT_NODES;
        let core_rewritten_units =
            trees.iter().map(|tree| tree.open_container_rewritten + tree.rewritten_index_nodes).sum::<u64>() + 1 + FIXED_POINT_NODES;
        let core_new_units = core_units_after - (core_units - core_rewritten_units);
        let (slots_per_device, allocation_levels) = allocation_record_tree_levels(containers, index_nodes, inode_leaves, knobs.file_data_units);
        let (_, allocation_levels_after) = allocation_record_tree_levels(containers_after, index_nodes_after, inode_leaves, knobs.file_data_units);
        let allocation_nodes: u64 = allocation_levels.iter().sum();
        let allocation_nodes_after: u64 = allocation_levels_after.iter().sum();
        let allocation_changed_records = 2 * (core_rewritten_units + core_new_units);
        let allocation_touched_leaves = match knobs.touched_leaves {
            TouchedLeaves::Spread => allocation_levels_after[0].min(allocation_changed_records),
            TouchedLeaves::Focused => allocation_levels_after[0].min(2),
        };
        let allocation_rewritten_nodes = rewritten_nodes_by_level(&allocation_levels_after, allocation_touched_leaves);
        let (mapping_leaf_entries, mapping_internal_entries) = if knobs.mapping_leaves_half_full {
            (CENTRAL_MAPPING_TREE.leaf_entries / 2, CENTRAL_MAPPING_TREE.internal_entries / 2)
        } else {
            (CENTRAL_MAPPING_TREE.leaf_entries, CENTRAL_MAPPING_TREE.internal_entries)
        };
        // 映射条目：树表、映射树、实例表豁免，分配记录树每个节点一条（`transaction.rs:15226-15227`）。
        let mapping_entries = core_units - 1 + allocation_nodes;
        let mapping_entries_after = core_units_after - 1 + allocation_nodes_after;
        let mapping_rewritten_units = core_rewritten_units - 1 + allocation_rewritten_nodes;
        let mapping_new_entries = mapping_entries_after - (mapping_entries - mapping_rewritten_units);
        let mapping_changed_entries = mapping_rewritten_units + mapping_new_entries;
        let mapping_levels = levels_by_fanout(mapping_entries, mapping_leaf_entries, mapping_internal_entries);
        let mapping_levels_after = levels_by_fanout(mapping_entries_after, mapping_leaf_entries, mapping_internal_entries);
        let mapping_nodes: u64 = mapping_levels.iter().sum();
        let mapping_nodes_after: u64 = mapping_levels_after.iter().sum();
        let mapping_touched_leaves = match knobs.touched_leaves {
            TouchedLeaves::Spread => mapping_levels_after[0].min(mapping_changed_entries),
            TouchedLeaves::Focused => mapping_levels_after[0].min(mapping_changed_entries.div_ceil(mapping_leaf_entries)),
        };
        let mapping_rewritten_nodes = rewritten_nodes_by_level(&mapping_levels_after, mapping_touched_leaves);
        let units = core_units + allocation_nodes + mapping_nodes + INSTANCE_TABLE_UNITS;
        let units_after = core_units_after + allocation_nodes_after + mapping_nodes_after + INSTANCE_TABLE_UNITS;
        let rewritten_units = core_rewritten_units + allocation_rewritten_nodes + mapping_rewritten_nodes;
        let new_units = units_after - (units - rewritten_units);
        Geometry {
            knobs,
            inode_leaves,
            core_rewritten_units,
            core_new_units,
            slots_per_device,
            allocation_carried_nodes: allocation_nodes_after.saturating_sub(allocation_rewritten_nodes),
            allocation_levels,
            allocation_nodes,
            allocation_levels_after,
            allocation_nodes_after,
            allocation_changed_records,
            allocation_touched_leaves,
            allocation_rewritten_nodes,
            mapping_leaf_entries,
            mapping_entries,
            mapping_entries_after,
            mapping_rewritten_units,
            mapping_new_entries,
            mapping_changed_entries,
            mapping_carried_nodes: mapping_nodes_after.saturating_sub(mapping_rewritten_nodes),
            mapping_levels,
            mapping_nodes,
            mapping_levels_after,
            mapping_nodes_after,
            mapping_touched_leaves,
            mapping_rewritten_nodes,
            units,
            units_after,
            rewritten_units,
            new_units,
            allocation_records: 2 * units,
            allocation_records_after: 2 * units_after,
            deleted_inodes_after: u64::from(knobs.tombstones_added > 0),
            tombstones,
            retirements,
            lineage,
        }
    }

    pub(super) fn container_trees(&self) -> [&ContainerTree; 3] {
        [&self.tombstones, &self.retirements, &self.lineage]
    }

    /// 墓碑树与退役树（O 里逐树的那几项只管这两棵）。
    pub(super) fn tombstone_and_retirement_trees(&self) -> [(&ContainerTree, u64); 2] {
        [(&self.tombstones, TOMBSTONE_RECORD_MEMORY_BYTES), (&self.retirements, RETIREMENT_RECORD_MEMORY_BYTES)]
    }

    pub(super) fn trees_with_record_widths(&self) -> [(&ContainerTree, u64); 3] {
        [
            (&self.tombstones, TOMBSTONE_RECORD_MEMORY_BYTES),
            (&self.retirements, RETIREMENT_RECORD_MEMORY_BYTES),
            (&self.lineage, LINEAGE_RECORD_MEMORY_BYTES),
        ]
    }
}

#[cfg(test)]
#[path = "knobs_trees_and_geometry_tests.rs"]
mod tests;
