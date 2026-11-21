//! 丁的压实与放行后照抄、装入与现算重建的字节数、一版的大小与各部分。

use super::*;

/// 丁的压实方式：整份新基（照抄 |V'|）与就地折叠（照抄 0）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Compaction {
    WholeNewBase,
    FoldInPlace,
}

impl Compaction {
    pub(super) fn label(self) -> &'static str {
        match self {
            Compaction::WholeNewBase => "whole_new_base",
            Compaction::FoldInPlace => "fold_in_place",
        }
    }
}

/// 丁准入后：非压实那次 0；压实那次整份新基 |V'|、就地折叠 0；准入前 0。
pub(super) fn ding_copied_after_admission(geometry: &Geometry, compacts_now: bool, compaction: Compaction) -> u64 {
    if compacts_now && compaction == Compaction::WholeNewBase {
        version_bytes_after(geometry)
    } else {
        0
    }
}

// ── 新装、新装第二份、现算重建、一版大小（四条臂相同的那几样；登记第五节） ──

/// 新装字节（四条臂相同）：准入前、准入后。
pub(super) fn installed_bytes(geometry: &Geometry) -> (u64, u64) {
    let knobs = geometry.knobs;
    let appended_records: u64 = geometry.trees_with_record_widths().iter().map(|(tree, width)| tree.added * width).sum();
    let before = (knobs.settle_rounds + 1) * (geometry.tombstones.added * TOMBSTONE_RECORD_MEMORY_BYTES + geometry.retirements.added * RETIREMENT_RECORD_MEMORY_BYTES)
        + knobs.settle_rounds
            * (geometry.lineage.added * LINEAGE_RECORD_MEMORY_BYTES
                + geometry.mapping_new_entries * MAPPED_UNIT_ENTRY_BYTES
                + ACCOUNTING_ENTRIES * key_structure_bytes(ACCOUNTING_TREE));
    let after = appended_records + installed_unit_bytes(geometry) + 2 * ALLOCATION_RECORD_MEMORY_BYTES * geometry.new_units;
    (before, after)
}

/// 准入后新装里的单元字节那几项：容器、索引节点、inode 叶、固定点、映射树与分配记录树重写的节点。
pub(super) fn installed_unit_bytes(geometry: &Geometry) -> u64 {
    let mut unit_bytes = 0;
    for tree in geometry.container_trees() {
        unit_bytes += (tree.open_container_rewritten + tree.new_containers) * CONTAINER_UNIT_MEMORY_BYTES
            + (tree.nodes_after - tree.nodes + tree.rewritten_index_nodes) * NODE_UNIT_MEMORY_BYTES;
    }
    unit_bytes
        + CONTAINER_UNIT_MEMORY_BYTES
        + FIXED_POINT_NODES * NODE_UNIT_MEMORY_BYTES
        + (geometry.mapping_rewritten_nodes + geometry.allocation_rewritten_nodes) * NODE_UNIT_MEMORY_BYTES
}

/// 现算重建字节（甲-B2后、甲-今天、乙1；丁压实那次的准入后同此）：准入前、准入后。
pub(super) fn recomputed_in_full(geometry: &Geometry) -> (u64, u64) {
    let knobs = geometry.knobs;
    let deadlist_entries_after = 0;
    let before = knobs.settle_rounds
        * (geometry.mapping_entries_after * key_structure_bytes(CENTRAL_MAPPING_TREE)
            + deadlist_entries_after * key_structure_bytes(DEADLIST_TREE)
            + geometry.allocation_nodes_after * ALLOCATION_TREE_NODE_IDENTITY_BYTES
            + 3 * geometry.allocation_records_after * ALLOCATION_RECORD_MEMORY_BYTES);
    let after = geometry.mapping_entries_after * mapping_unit_recompute_bytes()
        + rewritten_mapping_leaf_entries(geometry, geometry.mapping_entries_after) * MAPPING_LEAF_ENTRY_BYTES
        + deadlist_entries_after * MAPPED_UNIT_ENTRY_BYTES
        + geometry.deleted_inodes_after * DELETED_INODE_NUMBER_BYTES
        + 3 * geometry.allocation_records_after * ALLOCATION_RECORD_MEMORY_BYTES
        + 2 * geometry.allocation_nodes_after * ALLOCATION_TREE_NODE_IDENTITY_BYTES;
    (before, after)
}

/// 准入后每条映射现算的四样：带位置项 95 + `mapped_units` 67 + 映射条目表 194 + key 字节 51 = 407。
pub(super) fn mapping_unit_recompute_bytes() -> u64 {
    MAPPED_UNIT_WITH_LOCATIONS_BYTES + MAPPED_UNIT_ENTRY_BYTES + MAPPING_ENTRY_TABLE_BYTES + MAPPING_KEY_BYTES_ENTRY_BYTES
}

/// 重写的映射树叶的条目数 min(M', τ_M·c)。
fn rewritten_mapping_leaf_entries(geometry: &Geometry, mapping_entries_after: u64) -> u64 {
    mapping_entries_after.min(geometry.mapping_touched_leaves * geometry.mapping_leaf_entries)
}

/// 残余式（乙2、丙、丁非压实那次）：x = min(d_M, M')、y = min(d_AR, A')。
pub(super) fn recomputed_residual(geometry: &Geometry) -> (u64, u64) {
    let changed_mappings = geometry.mapping_changed_entries.min(geometry.mapping_entries_after);
    let changed_allocations = geometry.allocation_changed_records.min(geometry.allocation_records_after);
    let before = geometry.knobs.settle_rounds
        * (changed_mappings * key_structure_bytes(CENTRAL_MAPPING_TREE) + 3 * changed_allocations * ALLOCATION_RECORD_MEMORY_BYTES);
    let after = changed_mappings * mapping_unit_recompute_bytes()
        + rewritten_mapping_leaf_entries(geometry, geometry.mapping_entries_after) * MAPPING_LEAF_ENTRY_BYTES
        + geometry.deleted_inodes_after * DELETED_INODE_NUMBER_BYTES
        + 3 * changed_allocations * ALLOCATION_RECORD_MEMORY_BYTES;
    (before, after)
}

/// 一版大小的各项：三棵容器树、inode、单元（含固定点、实例表、映射树与分配记录树节点）、分配记录、映射、多层码 2 树形状、数据指针。
fn version_size_parts(
    trees: [(u64, u64, u64, u64, u64); 3],
    geometry: &Geometry,
    mapping_entries: u64,
    mapping_nodes: u64,
    allocation_nodes: u64,
    allocation_records: u64,
) -> (u64, u64) {
    let knobs = geometry.knobs;
    let mut without_pointers = 0;
    for (records, record_width, containers, nodes, height) in trees {
        without_pointers += records * record_width
            + CONTAINER_SHELL_BYTES * containers
            + shape_bytes(nodes, height)
            + containers * CONTAINER_UNIT_MEMORY_BYTES
            + nodes * NODE_UNIT_MEMORY_BYTES;
    }
    without_pointers += INODE_RECORD_MEMORY_BYTES * knobs.live_inodes
        + CONTAINER_SHELL_BYTES * geometry.inode_leaves
        + (geometry.inode_leaves + knobs.file_data_units) * CONTAINER_UNIT_MEMORY_BYTES
        + FIXED_POINT_NODES * NODE_UNIT_MEMORY_BYTES
        + INSTANCE_TABLE_UNITS * CONTAINER_UNIT_MEMORY_BYTES
        + (mapping_nodes + allocation_nodes) * NODE_UNIT_MEMORY_BYTES
        + ALLOCATION_RECORD_MEMORY_BYTES * allocation_records
        + MAPPED_UNIT_ENTRY_BYTES * mapping_entries
        + ALLOCATION_TREE_NODE_IDENTITY_BYTES * allocation_nodes
        + multi_level_shape_bytes(ACCOUNTING_ENTRIES, 1, ACCOUNTING_TREE)
        + multi_level_shape_bytes(mapping_entries, mapping_nodes, CENTRAL_MAPPING_TREE);
    (without_pointers, DATA_POINTER_MEMORY_BYTES * knobs.file_data_units)
}

/// |V|：这一版的大小（第三份另加这一版的 `data_pointers` 88·D）。
pub(super) fn version_bytes(geometry: &Geometry) -> u64 {
    let trees = geometry
        .trees_with_record_widths()
        .map(|(tree, width)| (tree.records, width, tree.containers, tree.nodes, tree.height));
    let (without_pointers, pointers) =
        version_size_parts(trees, geometry, geometry.mapping_entries, geometry.mapping_nodes, geometry.allocation_nodes, geometry.allocation_records);
    without_pointers + pointers
}

/// |V'|：这一版之后的大小，另加 `deleted_inodes` 的号 32·Q'。
pub(super) fn version_bytes_after(geometry: &Geometry) -> u64 {
    let trees = geometry
        .trees_with_record_widths()
        .map(|(tree, width)| (tree.records_after, width, tree.containers_after, tree.nodes_after, tree.height_after));
    let (without_pointers, pointers) = version_size_parts(
        trees,
        geometry,
        geometry.mapping_entries_after,
        geometry.mapping_nodes_after,
        geometry.allocation_nodes_after,
        geometry.allocation_records_after,
    );
    without_pointers + pointers + DELETED_INODE_NUMBER_BYTES * geometry.deleted_inodes_after
}

#[cfg(test)]
#[path = "compaction_and_version_sizes_tests.rs"]
mod tests;
