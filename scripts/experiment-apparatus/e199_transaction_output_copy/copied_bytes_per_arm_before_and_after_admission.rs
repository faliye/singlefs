//! 甲两种与今天对照在放行前后照抄的字节数。

use super::*;

// ── 甲：每次发布的照抄字节（登记第五节「每次发布的照抄字节」；第三份的式子） ──

/// 甲-B2后准入前的三部分：O（`ordinary_allocation_part` 一遍，跑 s + 1 遍）、Rl（`resolve` 一轮，跑 s 遍）、Once（循环外一次）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct CopiedBeforeAdmission {
    pub(super) ordinary_allocation_pass: u64,
    pub(super) settle_round: u64,
    pub(super) outside_the_rounds: u64,
}

impl CopiedBeforeAdmission {
    pub(super) fn total(&self, settle_rounds: u64) -> u64 {
        (settle_rounds + 1) * self.ordinary_allocation_pass + settle_rounds * self.settle_round + self.outside_the_rounds
    }
}

/// 三棵多层码 2 树（记账、deadlist、中央映射）这一版的（条目数、节点数、种类）。记账树 16 条一片叶；deadlist 零根（没有快照）。
pub(super) fn multi_level_trees(geometry: &Geometry) -> [(u64, u64, MultiLevelTreeKind); 3] {
    [(ACCOUNTING_ENTRIES, 1, ACCOUNTING_TREE), (0, 0, DEADLIST_TREE), (geometry.mapping_entries, geometry.mapping_nodes, CENTRAL_MAPPING_TREE)]
}

/// 同上，这一版之后。
pub(super) fn multi_level_trees_after(geometry: &Geometry) -> [(u64, u64, MultiLevelTreeKind); 3] {
    [(ACCOUNTING_ENTRIES, 1, ACCOUNTING_TREE), (0, 0, DEADLIST_TREE), (geometry.mapping_entries_after, geometry.mapping_nodes_after, CENTRAL_MAPPING_TREE)]
}

/// 不追加那一支每遍由调用方抄的那一份上一版形状（`transaction.rs:8640` 墓碑树、`8965` 谱系树；退役树每次都追加，这一项是 0）。
pub(super) fn shape_copied_when_not_appending(tree: &ContainerTree) -> u64 {
    if tree.appends { 0 } else { tree.shape_bytes() }
}

pub(super) fn jia_after_patch_copied_before_admission(geometry: &Geometry) -> CopiedBeforeAdmission {
    let knobs = geometry.knobs;
    let mut ordinary_allocation_pass = 0;
    for (tree, record_width) in geometry.tombstone_and_retirement_trees() {
        ordinary_allocation_pass += tree.records * record_width + CONTAINER_SHELL_BYTES * tree.containers;
        if tree.appends {
            ordinary_allocation_pass += tree.records * record_width + CONTAINER_SHELL_BYTES * tree.containers + tree.shape_bytes();
        }
        // `transaction.rs:8640`：墓碑树不追加那一支照抄上一版形状（第三份补进）。
        ordinary_allocation_pass += shape_copied_when_not_appending(tree);
    }
    // inode 叶照抄与它的第二份（`transaction.rs:4208`、`inode_tree.rs:1093` / `1970`）。
    ordinary_allocation_pass += 2 * (INODE_RECORD_MEMORY_BYTES * knobs.live_inodes + CONTAINER_SHELL_BYTES * geometry.inode_leaves);
    let lineage = &geometry.lineage;
    let mut settle_round = lineage.records * LINEAGE_RECORD_MEMORY_BYTES + CONTAINER_SHELL_BYTES * lineage.containers;
    if lineage.appends {
        settle_round += lineage.records * LINEAGE_RECORD_MEMORY_BYTES + CONTAINER_SHELL_BYTES * lineage.containers + lineage.shape_bytes();
    }
    // `transaction.rs:8965`：谱系树不追加那一支照抄上一版形状（第三份补进）。
    settle_round += shape_copied_when_not_appending(lineage);
    settle_round += (geometry.mapping_entries - geometry.mapping_rewritten_units) * MAPPED_UNIT_ENTRY_BYTES;
    for (entries, nodes, kind) in multi_level_trees(geometry) {
        settle_round += entries * key_structure_bytes(kind);
        if nodes >= 1 {
            settle_round += (entries + nodes - 1) * key_structure_bytes(kind);
        }
    }
    settle_round += 2 * geometry.allocation_records * ALLOCATION_RECORD_MEMORY_BYTES;
    CopiedBeforeAdmission {
        ordinary_allocation_pass,
        settle_round,
        outside_the_rounds: 2 * geometry.allocation_nodes * ALLOCATION_TREE_NODE_IDENTITY_BYTES,
    }
}

/// 甲-B2后准入后照抄的分项：(a) 新版 inode 叶内容再抄一次；(b) 中央映射树照抄节点（两份各一）；(c) 三棵多层码 2 树形状；
/// (d) 上一版分配记录树节点集两份（在准入前，Once）；其余照式子合在 `rest` 里。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct CopiedAfterAdmission {
    pub(super) inode_leaves_again: u64,
    pub(super) mapping_carried_nodes_each_copy: u64,
    pub(super) multi_level_shapes: u64,
    pub(super) carried_file_data_units: u64,
    pub(super) carried_file_data_pointers: u64,
    rest: u64,
}

impl CopiedAfterAdmission {
    pub(super) fn total(&self) -> u64 {
        self.inode_leaves_again
            + 2 * self.mapping_carried_nodes_each_copy
            + self.multi_level_shapes
            + self.carried_file_data_units
            + self.carried_file_data_pointers
            + self.rest
    }
}

pub(super) fn jia_after_patch_copied_after_admission(geometry: &Geometry) -> CopiedAfterAdmission {
    let knobs = geometry.knobs;
    let mut rest = 0;
    for (tree, record_width) in geometry.trees_with_record_widths() {
        rest += tree.records_after * record_width
            + CONTAINER_SHELL_BYTES * tree.containers_after
            + shape_bytes(tree.nodes_after, tree.height_after)
            + (tree.containers - tree.open_container_rewritten) * CONTAINER_UNIT_MEMORY_BYTES
            + (tree.nodes - tree.rewritten_index_nodes) * NODE_UNIT_MEMORY_BYTES;
    }
    // 第一个文件之外的照抄单元（inode 叶 L_I − 1 片与数据单元 D 个）走 `carried_unit`。
    rest += (geometry.inode_leaves - 1 + knobs.file_data_units) * CONTAINER_UNIT_MEMORY_BYTES;
    rest += ALLOCATION_RECORD_MEMORY_BYTES * (geometry.allocation_records_after - 2 * geometry.new_units);
    rest += geometry.allocation_carried_nodes * NODE_UNIT_MEMORY_BYTES;
    rest += INSTANCE_TABLE_UNITS * CONTAINER_UNIT_MEMORY_BYTES;
    rest += geometry.allocation_records * ALLOCATION_RECORD_MEMORY_BYTES;
    CopiedAfterAdmission {
        inode_leaves_again: INODE_RECORD_MEMORY_BYTES * knobs.live_inodes + CONTAINER_SHELL_BYTES * geometry.inode_leaves,
        mapping_carried_nodes_each_copy: geometry.mapping_carried_nodes * NODE_UNIT_MEMORY_BYTES,
        multi_level_shapes: multi_level_trees_after(geometry)
            .iter()
            .map(|&(entries, nodes, kind)| multi_level_shape_bytes(entries, nodes, kind))
            .sum(),
        // `transaction.rs:14640`、`14643`（第三份全扫补进）。
        carried_file_data_units: CARRIED_DATA_UNIT_BYTES * knobs.file_data_units,
        carried_file_data_pointers: DATA_POINTER_MEMORY_BYTES * knobs.file_data_units,
        rest,
    }
}

/// 甲-今天准入前 = 甲-B2后 + (s + 1)·Σ_{T,X} (2 + k_new_t − [t 不追加])·shape + s·(2 + k_new_L − [a_L = 0])·shape_L。
pub(super) fn jia_today_copied_before_admission(geometry: &Geometry) -> u64 {
    let settle_rounds = geometry.knobs.settle_rounds;
    let shapes_dropped_by_the_patch = |tree: &ContainerTree| (2 + tree.new_containers - u64::from(!tree.appends)) * tree.shape_bytes();
    let mut today = jia_after_patch_copied_before_admission(geometry).total(settle_rounds);
    for (tree, _) in geometry.tombstone_and_retirement_trees() {
        today += (settle_rounds + 1) * shapes_dropped_by_the_patch(tree);
    }
    today + settle_rounds * shapes_dropped_by_the_patch(&geometry.lineage)
}

#[cfg(test)]
#[path = "copied_bytes_per_arm_before_and_after_admission_tests.rs"]
mod tests;
