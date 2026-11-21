use e7_index_bench::usize_from_u64;
use super::*;

/// E142 第十五次跑（第七节 B1–B3）：分配记录树五个节点的层级、罩的段、格——按位置寻址的落点表。
/// M94（叶宽 812→806）、M95（扇出 169→168）、M97/M98（哪个常量该对哪个槽）、M100（γ乙 半开）、
/// M101（层级 1 格号差一）、M102（α甲 下空格写 key，退回 α丙）的会红检查都落在这里。
#[test]
fn allocation_record_tree_five_nodes_match_the_position_addressed_layout() {
    let BuiltPool { output, .. } = built_pool();
    let leaf0 = parse_index_node(&output.units_by_slot[4].2).expect("罩盘 0 的叶");
    let leaf1 = parse_index_node(&output.units_by_slot[5].2).expect("罩盘 1 的叶");
    let internal0 = parse_index_node(&output.units_by_slot[6].2).expect("罩盘 0 的层级 1 节点");
    let internal1 = parse_index_node(&output.units_by_slot[7].2).expect("罩盘 1 的层级 1 节点");
    let root = parse_index_node(&output.units_by_slot[8].2).expect("根");

    // 叶（层级 0）：第七节 B3「leaves=[61] leaf_range=[(49532,50343)]」。
    for (leaf, device) in [(&leaf0, 0u32), (&leaf1, 1u32)] {
        assert_eq!(leaf.level, 0);
        assert_eq!(leaf.entries.len(), 14, "每盘 14 条记录都落在同一片叶（第七节 B4）");
        assert_eq!(leaf.smallest_key, allocation_position_key(device, 49532));
        assert_eq!(leaf.largest_key, allocation_position_key(device, 50343));
        for entry in &leaf.entries {
            assert_eq!(&entry[..4], &device.to_le_bytes(), "叶里每条记录都是这块盘自己的");
        }
    }

    // 层级 1（第七节 B3「l1_range=(0,137227)」）：D8 已定项 14 第 395 行唯一写法——稀疏，
    // 只写有孩子的格，这个装置的场景每盘只有一个孩子（cell_in_l1=61）。
    for (internal, device, leaf_slot) in [(&internal0, 0u32, SLOT_ALLOCATION_LEAF_DEVICE_0), (&internal1, 1u32, SLOT_ALLOCATION_LEAF_DEVICE_1)] {
        assert_eq!(internal.level, 1);
        assert_eq!(internal.entries.len(), 1, "唯一写法：稀疏，只有一条条目（第七节 B7「level1=96」＝1×96）");
        assert_eq!(internal.smallest_key, allocation_position_key(device, 0));
        assert_eq!(internal.largest_key, allocation_position_key(device, 137_227));
        let cell = &internal.entries[0];
        assert_eq!(&cell[..ALLOCATION_KEY_BYTES], &allocation_position_key(device, 49532)[..], "ι甲：条目 key = 孩子按位置罩的段起点");
        let mut reader = ByteReader::new(cell);
        reader.skip(ALLOCATION_KEY_BYTES);
        let pointer = NodePointer::read_from(&mut reader);
        assert_eq!(pointer.locations[0].slot, SlotNumber(leaf_slot), "唯一那条条目的子指针指向对应盘的叶");
    }

    // 根（层级 2，单一节点，按盘分流）：D8 已定项 14 第 395 行唯一写法——稀疏，两条条目（一盘一条），
    // smallest_key=(0,0)、largest_key=(0xFFFFFFFF,2^48−1)（第七节 A6，与盘数、盘大小无关）。
    assert_eq!(root.level, 2);
    assert_eq!(root.entries.len(), 2, "唯一写法：稀疏，一盘一条（第七节 B7「root=192」＝2×96）");
    assert_eq!(root.smallest_key, allocation_position_key(0, 0));
    assert_eq!(root.largest_key, allocation_position_key(u32::MAX, (1u64 << 48) - 1));
    for (cell_index, device, expected_slot) in [(0usize, 0u32, SLOT_ALLOCATION_INTERNAL_DEVICE_0), (1, 1u32, SLOT_ALLOCATION_INTERNAL_DEVICE_1)] {
        let cell = &root.entries[cell_index];
        assert_eq!(&cell[..ALLOCATION_KEY_BYTES], &allocation_position_key(device, 0)[..], "根条目 {cell_index} 的 key（ι甲：段起点 0）");
        let mut reader = ByteReader::new(cell);
        reader.skip(ALLOCATION_KEY_BYTES);
        let pointer = NodePointer::read_from(&mut reader);
        assert_eq!(pointer.locations[0].slot, SlotNumber(expected_slot), "根条目 {cell_index} 指向对应盘的层级 1 节点");
    }

    // 格式常量（第七节 B1，A1、A3）。
    assert_eq!(ALLOCATION_RECORD_TREE_LEAF_SLOTS, 812);
    assert_eq!(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES, 96);
    assert_eq!(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT, 169);
    assert_eq!(EXTENT_TREE_UPPER_LEAF_INODES, 143);
    assert_eq!(EXTENT_TREE_INTERNAL_ENTRY_BYTES, 110);
    assert_eq!(EXTENT_TREE_INTERNAL_FANOUT, 147);
    assert_eq!(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, 144);
}

/// D8 已定项 14 第 395 行（唯一写法）：`assemble_allocation_cells` 只给有孩子的格建条目，没有孩子的
/// 格不占条目（M127/M128 的会红检查：改回稠密／稠密带 key 会让这条数变成 3）。
#[test]
fn assemble_allocation_cells_only_keeps_positions_with_a_child() {
    let pointer = NodePointer::empty_root();
    let slots = vec![(0u32, 100u64, None), (0u32, 200u64, Some(pointer)), (0u32, 300u64, None)];
    let cells = assemble_allocation_cells(&slots);
    assert_eq!(cells.len(), 1, "唯一写法：稀疏，只写有孩子的格");
    assert_eq!(cells[0], allocation_internal_cell(0, 200, pointer));
}

/// E142 第十五次跑（第七节 A3）：extent 树上段叶条目 113 = key 24 `(0, inode, 0)` + 标签 2（内联）+ 数据指针 88。
/// M103（标签 2→1）、M104（max_key 的 inode 分量 142→143）、M105（丢标签字节，113→112）的会红检查落在这里。
#[test]
fn extent_upper_leaf_entry_is_113_bytes_with_an_inline_data_pointer_tag() {
    let BuiltPool { output, .. } = built_pool();
    let extent_node = parse_index_node(&output.units_by_slot[1].2).expect("extent 上段叶（根即叶）");
    assert_eq!(extent_node.level, 0, "5.2 ζ：只有一个 inode ⇒ 根就是上段叶");
    assert_eq!(extent_node.entries.len(), 1, "5.2 ε 甲：稀疏，只写有文件的 inode");
    let entry = &extent_node.entries[0];
    assert_eq!(entry.len(), 113, "第七节 A3「extent_tree_upper_leaf_entry=113」");
    let (key, tag, pointer) = parse_extent_upper_leaf_entry(entry);
    assert_eq!(&key[..8], &0u64.to_le_bytes(), "locality 恒 0");
    assert_eq!(&key[8..16], &FIRST_INODE_NUMBER.to_le_bytes());
    assert_eq!(&key[16..24], &0u64.to_le_bytes(), "条目自己的 key 第三分量恒 0（与节点头 largest_key 的第三分量 2^64−1 是两回事）");
    assert_eq!(tag, EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT, "只有一个数据单元的文件 ⇒ 标签 2（内联）");
    assert_eq!(pointer, output.data_pointer);
    // key 区间（D8 已定项 14 第 401 行，唯一写法：按位置罩的那一段，不是记录自己的 key）：
    // inode 1 ⇒ k = 0，闭区间 [0,142]，第三分量恒 2^64 − 1（第七节 A7、B12）。
    let (expected_smallest, expected_largest) = extent_upper_leaf_positional_key_range(0);
    assert_eq!(extent_node.smallest_key, expected_smallest);
    assert_eq!(extent_node.largest_key, expected_largest);
}

/// 第八节 G3「形状，纯函数」（第七节 B13）：分配记录树的根层级、根每盘格数、新池新建文件节点数
/// 在五个几何点上都用纯函数独立算一遍，不跑整条写路，入参一律是「每盘设备字节数」经 `slots_of_device_bytes`
/// （D8 已定项 14 第 408 行，唯一写法）。M96（根层级门槛 ≤169→≤3）在主点（R=2→3）就翻；
/// M142（盘上槽数向上取整）在主点不改输出（4 GiB 整除），只在 half_slot 点上把 R 从 2 顶到 3。
#[test]
fn allocation_record_tree_shape_matches_the_five_geometry_points() {
    assert_eq!(DEVICE_SLOTS, 262_144, "主点：4 GiB ÷ 16 KiB（δ 甲，经 slots_of_device_bytes，第七节 B3「device_slots」）");
    let half_slot_device_bytes: u64 = 84 * allocation_record_tree_span_at_level(1) * SLOT_BYTES + SLOT_BYTES / 2;
    let cases: [(&str, u64, u64, u32, u64, u64); 5] = [
        // (标签, 盘数, 每盘设备字节数, R, root_cells_per_device, new_pool_file_creation_allocation_nodes)——第七节 B13。
        ("main", 2, DEVICE_BYTES, 2, 2, 5),
        ("one_device", 1, DEVICE_BYTES, 2, 2, 3),
        ("8GiB", 2, 8u64 << 30, 2, 4, 5),
        ("256GiB", 2, 256u64 << 30, 3, 1, 7),
        ("half_slot", 2, half_slot_device_bytes, 2, 84, 5),
    ];
    for (label, device_count, device_bytes, expected_root_level, expected_cells_per_device, expected_nodes) in cases {
        let slots = slots_of_device_bytes(device_bytes);
        let device_slots = vec![slots; usize_from_u64(device_count)];
        let root_level = allocation_record_tree_root_level(&device_slots);
        assert_eq!(root_level, expected_root_level, "{label}：R");
        let cells_per_device = allocation_record_tree_cells_per_device(slots, root_level);
        assert_eq!(cells_per_device, expected_cells_per_device, "{label}：root_cells_per_device");
        let nodes = allocation_record_tree_new_pool_file_creation_node_count(device_count, root_level);
        assert_eq!(nodes, expected_nodes, "{label}：new_pool_file_creation_allocation_nodes");
    }
    // M142 的会红检查：half_slot 点上向上取整会把盘上槽数多算一槽，进而把 R 从 2 顶到 3（第七节 B13「R_if_ceil=3」）。
    let half_slot_slots_floor = slots_of_device_bytes(half_slot_device_bytes);
    let half_slot_slots_ceil = half_slot_device_bytes.div_ceil(SLOT_BYTES);
    assert_eq!(half_slot_slots_floor + 1, half_slot_slots_ceil, "half_slot 点：向上取整比向下取整多一槽（第 408 行「零头不算」）");
    assert_eq!(allocation_record_tree_root_level(&[half_slot_slots_floor; 2]), 2, "向下取整：R 仍是 2");
    assert_eq!(allocation_record_tree_root_level(&[half_slot_slots_ceil; 2]), 3, "M142 的会红检查：向上取整会把 R 顶到 3");
}
