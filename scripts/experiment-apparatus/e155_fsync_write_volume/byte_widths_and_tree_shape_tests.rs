use super::*;

/// B1（第七节 7.2）：容量锚点，`anchors.py` / `anchors2.py` 手抄常量算出的独立值。
#[test]
fn capacities_match_the_preregistration_anchors() {
    assert_eq!(node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES), 144, "extent 叶容量");
    assert_eq!(
        node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, false)),
        147,
        "extent 内部容量（K3）"
    );
    assert_eq!(inode_container_capacity(), 233, "inode 容器容量");
    assert_eq!(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES), 135, "inode 内部容量");
    assert_eq!(node_capacity(ALLOCATION_KEY_BYTES, ALLOCATION_LEAF_ENTRY_BYTES), 812, "分配记录叶容量");
    assert_eq!(
        node_capacity(ALLOCATION_KEY_BYTES, internal_entry_bytes(ALLOCATION_KEY_BYTES, false)),
        169,
        "分配记录内部容量（K3）"
    );
    assert_eq!(node_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_LEAF_ENTRY_BYTES), 477, "记账叶容量");
    assert_eq!(node_capacity(MAPPING_KEY_BYTES, MAPPING_LEAF_ENTRY_BYTES), 294, "映射叶容量");
    assert_eq!(
        node_capacity(MAPPING_KEY_BYTES, internal_entry_bytes(MAPPING_KEY_BYTES, false)),
        143,
        "映射内部容量（K3）"
    );
    assert_eq!(node_capacity(TREE_TABLE_KEY_BYTES, TREE_TABLE_ENTRY_BYTES), 81, "树表容量");
    assert_eq!(NAMED_ITEMS_PER_RECORD_MAIN, 67, "一条记录装的点名项数（A1）");
    assert_eq!(UNIT_BYTES - 105 - 29, 32634, "数据单元载荷上限");
}

/// B2：头 311 时的余数（D23 已定项 4「4096 上余 3785」，⌊3785/56⌋=67 余 33）——
/// 全部单测里唯一分得开 307 与 311 的一条（头 307 时余数是 37，同样是 67 项）。
#[test]
fn named_items_per_record_remainder_matches_head_311() {
    assert_eq!(RECORD_HEADER_BYTES + NAMED_ITEMS_PER_RECORD_MAIN * NAMED_ITEM_BYTES + 33, RECORD_BYTES);
}

/// B1 补充（`anchors2.py`）：记账内部扇出、K11 反向取样点的四棵树内部容量。
#[test]
fn accounting_internal_capacity_and_identity_reference_variant_match_the_anchors() {
    assert_eq!(node_capacity(ACCOUNTING_KEY_BYTES, internal_entry_bytes(ACCOUNTING_KEY_BYTES, false)), 150);
    assert_eq!(node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, true)), 119, "K11 extent");
    assert_eq!(node_capacity(ALLOCATION_KEY_BYTES, internal_entry_bytes(ALLOCATION_KEY_BYTES, true)), 133, "K11 分配记录");
    assert_eq!(node_capacity(ACCOUNTING_KEY_BYTES, internal_entry_bytes(ACCOUNTING_KEY_BYTES, true)), 121, "K11 记账");
    assert_eq!(node_capacity(MAPPING_KEY_BYTES, internal_entry_bytes(MAPPING_KEY_BYTES, true)), 116, "K11 映射");
}

/// K2：inode 树恒 ≥ 2 层，即便条目数只有 1（D8 已定项 6，根恒码 2）。
#[test]
fn inode_tree_is_at_least_two_layers_even_with_one_entry() {
    let layers = tree_layers(1, inode_container_capacity(), node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES), 2);
    assert_eq!(layers, vec![1, 1], "1 个容器 + 1 个恒码 2 的根");
}

/// K2：extent 树在 P = 145 时跨过叶容量 144，长到 2 层（第七节 B7）。
#[test]
fn extent_tree_reaches_two_layers_at_file_count_145() {
    let layers = tree_layers(145, node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES), node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, false)), 1);
    assert_eq!(layers, vec![2, 1], "145 条分两叶、加一个根");
}
