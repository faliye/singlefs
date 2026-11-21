use super::*;

// ── 第七节「出自被测条款本身的」（加法写，不写减法） ──

#[test]
fn unit_and_node_bytes_are_the_clause_values() {
    assert_eq!(UNIT_BYTES, 32768);
    assert_eq!(NODE_BYTES, 16384);
}

#[test]
fn code_two_fanout_fills_a_node_with_53_bytes_left() {
    assert_eq!(CODE_TWO_HEADER_BYTES + 2 * CODE_TWO_KEY_BYTES + CODE_TWO_FANOUT * CODE_TWO_ENTRY_BYTES + 53, 16384);
    assert_eq!(CODE_TWO_FANOUT, 135);
}

#[test]
fn code_three_containers_fill_a_unit() {
    assert_eq!(136 + TOMBSTONE_CONTAINER_RECORDS * TOMBSTONE_RECORD_DISK_BYTES + 32, 32768);
    assert_eq!(136 + LINEAGE_CONTAINER_RECORDS * LINEAGE_RECORD_DISK_BYTES + 16, 32768);
    assert_eq!(136 + RETIREMENT_CONTAINER_RECORDS * RETIREMENT_RECORD_DISK_BYTES + 26, 32768);
    assert_eq!(136 + INODE_LEAF_RECORDS * INODE_RECORD_DISK_BYTES + 12, 32768);
    assert_eq!((TOMBSTONE_CONTAINER_RECORDS, LINEAGE_CONTAINER_RECORDS, RETIREMENT_CONTAINER_RECORDS, INODE_LEAF_RECORDS), (815, 1359, 959, 233));
    assert_eq!(CODE_THREE_RECORD_AREA_START, 136);
}

// ── 第七节「取实现值的模型常量」 ──

#[test]
fn allocation_record_tree_fanout_and_leaf_slots_are_the_implementation_values() {
    assert_eq!(ALLOCATION_RECORD_TREE_INTERNAL_HEADER_BYTES + ALLOCATION_RECORD_TREE_INTERNAL_FANOUT * ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES + 25, 16384);
    assert_eq!(ALLOCATION_RECORD_TREE_LEAF_SLOTS, 812);
    assert_eq!(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT, 169);
}

#[test]
fn multi_level_tree_capacities_are_the_implementation_values() {
    assert_eq!((ACCOUNTING_TREE.leaf_entries, ACCOUNTING_TREE.internal_entries), (477, 150));
    assert_eq!((DEADLIST_TREE.leaf_entries, DEADLIST_TREE.internal_entries), (376, 125));
    assert_eq!((CENTRAL_MAPPING_TREE.leaf_entries, CENTRAL_MAPPING_TREE.internal_entries), (294, 143));
    assert_eq!((CENTRAL_MAPPING_TREE.key_bytes, ACCOUNTING_TREE.key_bytes, DEADLIST_TREE.key_bytes), (27, 22, 43));
    assert_eq!((CENTRAL_MAPPING_TREE.key_fields, ACCOUNTING_TREE.key_fields, DEADLIST_TREE.key_fields), (5, 4, 7));
    assert_eq!(MAPPING_ENTRY_BYTES, 55);
}

#[test]
fn accounting_entries_are_pool_rows_device_rows_and_the_step_row() {
    assert_eq!(3 + 2 * 6 + 1, ACCOUNTING_ENTRIES);
    assert_eq!(ACCOUNTING_ENTRIES, 16);
}

#[test]
fn data_pointer_is_header_two_positions_and_write_order() {
    assert_eq!(50 + 14 * 2 + 10, DATA_POINTER_BYTES);
    assert_eq!(DATA_POINTER_HEADER_BYTES + POSITION_ENTRY_BYTES * 2 + DATA_POINTER_WRITE_ORDER_BYTES, 88);
    assert_eq!(DATA_POINTER_MEMORY_BYTES, 88);
}

#[test]
fn carried_data_unit_is_unit_bytes_and_a_vector_header() {
    assert_eq!(32768 + 24, CARRIED_DATA_UNIT_BYTES);
}

#[test]
fn memory_widths_are_the_registered_values() {
    assert_eq!(
        [TOMBSTONE_RECORD_MEMORY_BYTES, LINEAGE_RECORD_MEMORY_BYTES, RETIREMENT_RECORD_MEMORY_BYTES, INODE_RECORD_MEMORY_BYTES, ALLOCATION_RECORD_MEMORY_BYTES],
        [40, 24, 40, 64, 40]
    );
    assert_eq!([CONTAINER_SHELL_BYTES, UNIT_SHELL_BYTES, SHAPE_NODE_BYTES, SHAPE_LEVEL_BYTES], [56, 48, 8, 24]);
    assert_eq!(
        [key_structure_bytes(CENTRAL_MAPPING_TREE), key_structure_bytes(ACCOUNTING_TREE), key_structure_bytes(DEADLIST_TREE)],
        [115, 102, 147]
    );
    assert_eq!(
        [MAPPED_UNIT_ENTRY_BYTES, MAPPED_UNIT_WITH_LOCATIONS_BYTES, MAPPING_ENTRY_TABLE_BYTES, MAPPING_KEY_BYTES_ENTRY_BYTES, MAPPING_LEAF_ENTRY_BYTES],
        [67, 95, 194, 51, 79]
    );
    assert_eq!([MULTI_LEVEL_SHAPE_NODE_BYTES, ALLOCATION_TREE_NODE_IDENTITY_BYTES, DELETED_INODE_NUMBER_BYTES], [32, 16, 32]);
    assert_eq!(REFERENCE_COUNTED_POINTER_BYTES, 8);
    assert_eq!((CONTAINER_UNIT_MEMORY_BYTES, NODE_UNIT_MEMORY_BYTES), (32816, 16432));
    assert_eq!(mapping_unit_recompute_bytes(), 407);
}
