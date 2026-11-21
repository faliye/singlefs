//! 登记补充的各项、乙 2 持久结构与乙 1 每片引用计数、丙的放行后照抄。

use super::*;

/// 第三份补进的四项逐项（甲-B2后，每次发布）：`8640`、`8965`、`14640`、`14643`。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct ItemsAddedByTheThirdRegistration {
    pub(super) tombstone_shape_when_not_appending: u64,
    pub(super) lineage_shape_when_not_appending: u64,
    pub(super) carried_file_data_units: u64,
    pub(super) carried_file_data_pointers: u64,
}

pub(super) fn items_added_by_the_third_registration(geometry: &Geometry) -> ItemsAddedByTheThirdRegistration {
    let settle_rounds = geometry.knobs.settle_rounds;
    let tombstones = &geometry.tombstones;
    let lineage = &geometry.lineage;
    ItemsAddedByTheThirdRegistration {
        tombstone_shape_when_not_appending: (settle_rounds + 1) * shape_copied_when_not_appending(tombstones),
        lineage_shape_when_not_appending: settle_rounds * shape_copied_when_not_appending(lineage),
        carried_file_data_units: CARRIED_DATA_UNIT_BYTES * geometry.knobs.file_data_units,
        carried_file_data_pointers: DATA_POINTER_MEMORY_BYTES * geometry.knobs.file_data_units,
    }
}

// ── 乙2、丙、丁的照抄字节（登记第五节；第三份新补几样上三臂多付 0） ──

/// depth(n) = 使 b^d ≥ n 的最小 d（n ≤ 1 时 0）。
pub(super) fn persistent_depth(count: u64, fanout: u64) -> u64 {
    let mut depth = 0;
    let mut reach = 1u64;
    while reach < count {
        reach = reach.saturating_mul(fanout);
        depth += 1;
    }
    depth
}

/// 乙2 的持久集合照抄 PV(n, w, x, 块) = T·(min(块, n)·w + depth(⌈n ÷ 块⌉)·P)，T = min(⌈n ÷ 块⌉, x)；n = 0 或 x = 0 时 0。
fn persistent_vector_copy(count: u64, width: u64, touched: u64, block: u64, fanout: u64) -> u64 {
    if count == 0 || touched == 0 {
        return 0;
    }
    let blocks = count.div_ceil(block);
    blocks.min(touched) * (block.min(count) * width + persistent_depth(blocks, fanout) * persistent_internal_node_bytes(fanout))
}

/// 乙2 一个内部节点 P = 8b + 16。
fn persistent_internal_node_bytes(fanout: u64) -> u64 {
    8 * fanout + 16
}

/// 乙2 一棵容器树一遍：开放容器的末块、容器表末块、最右路径形状（第一份第五节乙2 那一式里逐树那一段）。
fn yi2_container_tree_part(tree: &ContainerTree, record_width: u64, knobs: Knobs) -> u64 {
    let fanout = knobs.persistent_fanout;
    let block = knobs.persistent_leaf_block;
    let node = persistent_internal_node_bytes(fanout);
    let mut part = tree.open_container_rewritten * ((tree.fill % block) * record_width + persistent_depth(tree.fill.div_ceil(block), fanout) * node);
    if tree.containers > 0 {
        part += (((tree.containers - 1) % fanout) + 1) * CONTAINER_SHELL_BYTES + persistent_depth(tree.containers.div_ceil(fanout), fanout) * node;
    }
    part + SHAPE_NODE_BYTES * tree.height + SHAPE_LEVEL_BYTES
}

/// 乙2（b、g 取旋钮）：准入前 (s + 1)·O₂ + s·Rl₂ + Once₂，准入后一次。
pub(super) fn yi2_copied(geometry: &Geometry) -> (u64, u64) {
    let knobs = geometry.knobs;
    let fanout = knobs.persistent_fanout;
    let block = knobs.persistent_leaf_block;
    let node = persistent_internal_node_bytes(fanout);
    let inode_part = INODE_LEAF_RECORDS.min(block) * INODE_RECORD_MEMORY_BYTES
        + persistent_depth(INODE_LEAF_RECORDS.div_ceil(block), fanout) * node
        + fanout.min(geometry.inode_leaves) * CONTAINER_SHELL_BYTES
        + persistent_depth(geometry.inode_leaves.div_ceil(fanout), fanout) * node;
    let mut ordinary_allocation_pass = inode_part;
    for (tree, record_width) in geometry.tombstone_and_retirement_trees() {
        if tree.added > 0 {
            ordinary_allocation_pass += yi2_container_tree_part(tree, record_width, knobs);
        }
    }
    let changed = |kind: MultiLevelTreeKind| -> u64 {
        if kind == CENTRAL_MAPPING_TREE {
            geometry.mapping_changed_entries
        } else if kind == ACCOUNTING_TREE {
            ACCOUNTING_ENTRIES
        } else {
            0
        }
    };
    let mut settle_round = 0;
    if geometry.lineage.added > 0 {
        settle_round += yi2_container_tree_part(&geometry.lineage, LINEAGE_RECORD_MEMORY_BYTES, knobs);
    }
    settle_round += persistent_vector_copy(geometry.mapping_entries, MAPPED_UNIT_ENTRY_BYTES, geometry.mapping_changed_entries, block, fanout);
    for (entries, _, kind) in multi_level_trees(geometry) {
        settle_round += persistent_vector_copy(entries, key_structure_bytes(kind), changed(kind), block, fanout);
    }
    settle_round += 2 * persistent_vector_copy(geometry.allocation_records, ALLOCATION_RECORD_MEMORY_BYTES, geometry.allocation_changed_records, block, fanout);
    let outside_the_rounds =
        persistent_vector_copy(geometry.allocation_nodes, ALLOCATION_TREE_NODE_IDENTITY_BYTES, geometry.allocation_touched_leaves, fanout, fanout);
    let before = CopiedBeforeAdmission { ordinary_allocation_pass, settle_round, outside_the_rounds }.total(knobs.settle_rounds);
    let touched_unit_segments = geometry.container_trees().iter().filter(|tree| tree.added > 0).count() as u64 + 4;
    let mut after = touched_unit_segments
        * (fanout.min(geometry.units) * UNIT_SHELL_BYTES + persistent_depth(geometry.units.div_ceil(fanout), fanout) * node)
        + 2 * geometry.rewritten_units
            * (fanout.min(geometry.allocation_records) * ALLOCATION_RECORD_MEMORY_BYTES
                + persistent_depth(geometry.allocation_records.div_ceil(fanout), fanout) * node)
        + persistent_vector_copy(knobs.live_inodes, INODE_RECORD_MEMORY_BYTES, 1, block, fanout)
        + persistent_vector_copy(geometry.inode_leaves, CONTAINER_SHELL_BYTES, 1, fanout, fanout)
        + persistent_vector_copy(geometry.allocation_records, ALLOCATION_RECORD_MEMORY_BYTES, geometry.allocation_changed_records, block, fanout);
    for (entries, _, kind) in multi_level_trees_after(geometry) {
        after += persistent_vector_copy(entries, key_structure_bytes(kind), changed(kind), block, fanout);
    }
    (before, after)
}

// ── 第四份写死的乙2、乙1 两样（第四份第五节「阳性对照」「乙1 的照抄字节」） ──

/// 一棵容器树整条：记录、容器外壳与形状 R_t·W_t + 56·C_t + shape(N_t, h_t)（乙2 共享关掉那一读里每条集合整条复制）。
pub(super) fn whole_container_tree_bytes(tree: &ContainerTree, record_width: u64) -> u64 {
    tree.records * record_width + CONTAINER_SHELL_BYTES * tree.containers + tree.shape_bytes()
}

/// 乙2 阳性对照的退化形态（第四份第五节写死）：乙2 自己的持久分块向量把共享关掉，这一版每一条集合每次造版都整条复制，
/// 碰到的与没碰到的都复制、每一片容器的记录都复制、a_t = 0 的树也复制；遍数照乙2 准入前（O 部分 s + 1 遍、Rl 部分 s 遍、节点集一次）。
/// 只计元素字节，分块向量的内部节点与末块凑整不计（第二段的计数分配器照量）。
pub(super) fn yi2_sharing_turned_off_copied_before_admission(geometry: &Geometry) -> CopiedBeforeAdmission {
    let knobs = geometry.knobs;
    let mut sharing_off_pass = 0;
    for (tree, record_width) in geometry.tombstone_and_retirement_trees() {
        // 没碰到的树（a_t = 0，G0 的退役树）也整条复制。
        sharing_off_pass += whole_container_tree_bytes(tree, record_width);
    }
    sharing_off_pass += INODE_RECORD_MEMORY_BYTES * knobs.live_inodes + CONTAINER_SHELL_BYTES * geometry.inode_leaves;
    let mut sharing_off_round = whole_container_tree_bytes(&geometry.lineage, LINEAGE_RECORD_MEMORY_BYTES);
    sharing_off_round += MAPPED_UNIT_ENTRY_BYTES * geometry.mapping_entries;
    for (entries, _, kind) in multi_level_trees(geometry) {
        sharing_off_round += key_structure_bytes(kind) * entries;
    }
    sharing_off_round += 2 * ALLOCATION_RECORD_MEMORY_BYTES * geometry.allocation_records;
    CopiedBeforeAdmission {
        ordinary_allocation_pass: sharing_off_pass,
        settle_round: sharing_off_round,
        outside_the_rounds: ALLOCATION_TREE_NODE_IDENTITY_BYTES * geometry.allocation_nodes,
    }
}

/// 附带（第四份第六节另加那一行，报数、不判、不触发作废条款第 3 条）：执行员头一种读法，只把乙2 式子里碰到的集合整条复制，
/// 墓碑树只复制开放那一片的记录、没碰到的树照旧共享。
pub(super) fn yi2_touched_collections_whole_copied_before_admission(geometry: &Geometry) -> CopiedBeforeAdmission {
    let knobs = geometry.knobs;
    let open_container_and_table = |tree: &ContainerTree, record_width: u64| -> u64 {
        if tree.added > 0 { tree.fill * record_width + CONTAINER_SHELL_BYTES * tree.containers + tree.shape_bytes() } else { 0 }
    };
    let mut touched_pass = 0;
    for (tree, record_width) in geometry.tombstone_and_retirement_trees() {
        touched_pass += open_container_and_table(tree, record_width);
    }
    touched_pass += INODE_LEAF_RECORDS.min(knobs.live_inodes) * INODE_RECORD_MEMORY_BYTES + CONTAINER_SHELL_BYTES * geometry.inode_leaves;
    let mut touched_round = open_container_and_table(&geometry.lineage, LINEAGE_RECORD_MEMORY_BYTES);
    touched_round += MAPPED_UNIT_ENTRY_BYTES * geometry.mapping_entries;
    for (entries, _, kind) in multi_level_trees(geometry) {
        touched_round += key_structure_bytes(kind) * entries;
    }
    touched_round += 2 * ALLOCATION_RECORD_MEMORY_BYTES * geometry.allocation_records;
    CopiedBeforeAdmission {
        ordinary_allocation_pass: touched_pass,
        settle_round: touched_round,
        outside_the_rounds: ALLOCATION_TREE_NODE_IDENTITY_BYTES * geometry.allocation_nodes,
    }
}

/// 乙1 的引用计数照常共享，或阳性对照那一读把共享关掉（克隆内容、不克隆指针；第四份第五节「乙1：字面推得出」：
/// 乙1 的每一项是在甲-B2后那一项上换出来的，关掉共享就逐项换回甲-B2后那一项）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ReferenceSharing {
    Shared,
    TurnedOff,
}

/// 乙1 准入前（第四份第五节「乙1 的照抄字节」三条规则与逐项对照表）：规则 1 第一份乙1 写法，规则 3 其余照甲。
pub(super) fn yi1_copied_before_admission(geometry: &Geometry, sharing: ReferenceSharing) -> CopiedBeforeAdmission {
    let knobs = geometry.knobs;
    let is_shared = sharing == ReferenceSharing::Shared;
    // 容器表：共享时顶层只抄指针 8·C_t（规则 1），关掉时照甲抄记录与外壳 R_t·W_t + 56·C_t。
    let container_table = |tree: &ContainerTree, record_width: u64| -> u64 {
        if is_shared { REFERENCE_COUNTED_POINTER_BYTES * tree.containers } else { tree.records * record_width + CONTAINER_SHELL_BYTES * tree.containers }
    };
    // 被改那片写时复制 fill_t·W_t（a_t > 0、k_old_t = 1；规则 1）；甲没有单列这一项，含在整条记录里，关掉时是 0。
    let copy_on_write_of_the_rewritten_container = |tree: &ContainerTree, record_width: u64| -> u64 {
        if is_shared { tree.open_container_rewritten * tree.fill * record_width } else { 0 }
    };
    let mut yi1_ordinary_allocation_pass = 0;
    for (tree, record_width) in geometry.tombstone_and_retirement_trees() {
        yi1_ordinary_allocation_pass += container_table(tree, record_width);
        if tree.appends {
            yi1_ordinary_allocation_pass += container_table(tree, record_width) + tree.shape_bytes();
        }
        yi1_ordinary_allocation_pass += copy_on_write_of_the_rewritten_container(tree, record_width);
        // `8640`：不追加那一支的形状照甲（规则 3）。
        yi1_ordinary_allocation_pass += shape_copied_when_not_appending(tree);
    }
    // inode 叶第一份：共享时照第一份乙1 写法 8·L_I + min(I, 233)·64（规则 1）。
    let inode_leaves_first_copy = if is_shared {
        REFERENCE_COUNTED_POINTER_BYTES * geometry.inode_leaves + INODE_LEAF_RECORDS.min(knobs.live_inodes) * INODE_RECORD_MEMORY_BYTES
    } else {
        INODE_RECORD_MEMORY_BYTES * knobs.live_inodes + CONTAINER_SHELL_BYTES * geometry.inode_leaves
    };
    // inode 叶第二份是第二份登记新补的项，照甲（规则 3），共享开不开都一样。
    let inode_leaves_second_copy = INODE_RECORD_MEMORY_BYTES * knobs.live_inodes + CONTAINER_SHELL_BYTES * geometry.inode_leaves;
    yi1_ordinary_allocation_pass += inode_leaves_first_copy + inode_leaves_second_copy;
    let lineage = &geometry.lineage;
    let mut yi1_settle_round = container_table(lineage, LINEAGE_RECORD_MEMORY_BYTES);
    if lineage.appends {
        yi1_settle_round += container_table(lineage, LINEAGE_RECORD_MEMORY_BYTES) + lineage.shape_bytes();
    }
    yi1_settle_round += copy_on_write_of_the_rewritten_container(lineage, LINEAGE_RECORD_MEMORY_BYTES);
    // `8965`：谱系树不追加那一支的形状照甲（规则 3）。
    yi1_settle_round += shape_copied_when_not_appending(lineage);
    // 映射 key、多层码 2 树的 key 与规划、分配记录按叶分组照甲（规则 3）。
    yi1_settle_round += MAPPED_UNIT_ENTRY_BYTES * (geometry.mapping_entries - geometry.mapping_rewritten_units);
    for (entries, nodes, kind) in multi_level_trees(geometry) {
        yi1_settle_round += key_structure_bytes(kind) * entries;
        if nodes >= 1 {
            yi1_settle_round += key_structure_bytes(kind) * (entries + nodes - 1);
        }
    }
    yi1_settle_round += 2 * ALLOCATION_RECORD_MEMORY_BYTES * geometry.allocation_records;
    CopiedBeforeAdmission {
        ordinary_allocation_pass: yi1_ordinary_allocation_pass,
        settle_round: yi1_settle_round,
        outside_the_rounds: 2 * ALLOCATION_TREE_NODE_IDENTITY_BYTES * geometry.allocation_nodes,
    }
}

/// 乙1 准入后一次（第四份第五节合起来那一式）：容器内容与形状照第一份乙1 写法（规则 1），单元字节项换成 × 8（规则 2），其余照甲（规则 3）。
pub(super) fn yi1_copied_after_admission(geometry: &Geometry, sharing: ReferenceSharing) -> u64 {
    let knobs = geometry.knobs;
    let is_shared = sharing == ReferenceSharing::Shared;
    let carried_container_units = |units: u64| -> u64 { if is_shared { REFERENCE_COUNTED_POINTER_BYTES * units } else { CONTAINER_UNIT_MEMORY_BYTES * units } };
    let carried_node_units = |units: u64| -> u64 { if is_shared { REFERENCE_COUNTED_POINTER_BYTES * units } else { NODE_UNIT_MEMORY_BYTES * units } };
    let mut yi1_after = 0;
    for (tree, record_width) in geometry.trees_with_record_widths() {
        let new_version_contents =
            if is_shared { REFERENCE_COUNTED_POINTER_BYTES * tree.containers_after } else { tree.records_after * record_width + CONTAINER_SHELL_BYTES * tree.containers_after };
        yi1_after += new_version_contents
            + shape_bytes(tree.nodes_after, tree.height_after)
            + carried_container_units(tree.containers - tree.open_container_rewritten)
            + carried_node_units(tree.nodes - tree.rewritten_index_nodes);
    }
    yi1_after += carried_container_units(geometry.inode_leaves - 1 + knobs.file_data_units);
    yi1_after += ALLOCATION_RECORD_MEMORY_BYTES * (geometry.allocation_records_after - 2 * geometry.new_units);
    // (a) 新版 inode 叶内容再抄一次（规则 3）。
    yi1_after += INODE_RECORD_MEMORY_BYTES * knobs.live_inodes + CONTAINER_SHELL_BYTES * geometry.inode_leaves;
    // (b) 中央映射树照抄节点与它的第二份、分配记录树照抄节点（规则 2）。
    yi1_after += 2 * carried_node_units(geometry.mapping_carried_nodes) + carried_node_units(geometry.allocation_carried_nodes);
    // (c) 三棵多层码 2 树形状（规则 3）。
    yi1_after += multi_level_trees_after(geometry).iter().map(|&(entries, nodes, kind)| multi_level_shape_bytes(entries, nodes, kind)).sum::<u64>();
    yi1_after += carried_container_units(INSTANCE_TABLE_UNITS);
    // 分配记录按叶分组 A·40（规则 3）。
    yi1_after += ALLOCATION_RECORD_MEMORY_BYTES * geometry.allocation_records;
    // `14640` 第一个文件照抄的数据单元换成指针（规则 2）；`14643` 数据指针照甲（规则 3）。
    let yi1_carried_file_data_units = if is_shared { REFERENCE_COUNTED_POINTER_BYTES * knobs.file_data_units } else { CARRIED_DATA_UNIT_BYTES * knobs.file_data_units };
    let yi1_carried_file_data_pointers = knobs.file_data_units * DATA_POINTER_MEMORY_BYTES;
    yi1_after + yi1_carried_file_data_units + yi1_carried_file_data_pointers
}

/// 丙准入后：撤回日志 = 48·r + Σ_{T,X,L,M,AR} (8 + 8·h_t) + 64 + 80·r；准入前 0。
pub(super) fn bing_copied_after_admission(geometry: &Geometry) -> u64 {
    let heights = [
        geometry.tombstones.height,
        geometry.retirements.height,
        geometry.lineage.height,
        geometry.mapping_levels.len() as u64,
        geometry.allocation_levels.len() as u64,
    ];
    let rightmost_paths: u64 = heights.iter().map(|height| 8 + 8 * height).sum();
    UNIT_SHELL_BYTES * geometry.rewritten_units
        + rightmost_paths
        + INODE_RECORD_MEMORY_BYTES
        + 2 * ALLOCATION_RECORD_MEMORY_BYTES * geometry.rewritten_units
}

#[cfg(test)]
#[path = "persistent_and_reference_counted_arms_tests.rs"]
mod tests;
