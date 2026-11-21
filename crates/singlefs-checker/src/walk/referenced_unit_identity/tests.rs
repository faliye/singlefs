use super::*;

/// 指向盘 0、盘 1 槽 1 的指针（`pointer_bytes` 是 86 的节点指针或 88 的数据指针），出生 txg 写 `birth_txg`，两条位置条目的整单元校验和
/// 按读 `unit_bytes` 那么长算。
fn pointer_to_slot_one(
    image: &ImageHoldingOneUnitAtSlotOne,
    pointer_bytes: usize,
    unit_bytes: usize,
    birth_txg: u64,
) -> Vec<u8> {
    let checksum = crc32_castagnoli_table(&image.unit_padded_to_a_data_unit[..unit_bytes]);
    let mut pointer = vec![0u8; pointer_bytes];
    pointer[42..50].copy_from_slice(&birth_txg.to_le_bytes());
    for (device, location_offset) in [(0u32, 50usize), (1u32, 64usize)] {
        pointer[location_offset..location_offset + 4].copy_from_slice(&device.to_le_bytes());
        pointer[location_offset + 4..location_offset + 10]
            .copy_from_slice(&1u64.to_le_bytes()[..6]);
        pointer[location_offset + 10..location_offset + 14]
            .copy_from_slice(&checksum.to_le_bytes());
    }
    pointer
}

/// 按 inode 树内部节点的读法读那个节点（树 `tree` 的引用，指针自己的位置为准）。
fn read_the_node_as_a_node_of_tree(
    walk: &mut Walk<'_>,
    pointer: &[u8],
    tree: u64,
) -> IndexNodeReading {
    walk.read_index_node(
        pointer,
        tree,
        crate::KEY_SCHEMA_INODE,
        "手造的节点",
        KeyRangeReading::SubtreeCoverageOfTheInodeTreeJudgedByTheCaller,
        EntryFieldTableRegistration::Registered,
        PositionAuthority::ThePointerItself,
    )
}

/// C01 树 ID 那一格（审计局部对照树 ID 7 / 8 那一形）：树 7 的节点先被一版按树 7 走过，下一版按树 8 引用它——命中走过的单元、不往下展开，
/// 这一条引用要的树照判，I-1.3 红。
#[test]
fn node_an_earlier_version_walked_reddens_the_tree_identifier_invariant_when_referenced_from_another_tree(
) {
    let image = ImageHoldingOneUnitAtSlotOne::of(&index_node_of_tree(7));
    let pointer = pointer_to_slot_one(&image, node_pointer_bytes(), node_bytes(), 0);
    let mut walk = Walk::starting_with(&image, Judgements::default(), 0, 0, 10);
    assert!(matches!(
        read_the_node_as_a_node_of_tree(&mut walk, &pointer, 7),
        IndexNodeReading::WalkedFirstByThisVersion { .. }
    ));
    assert_eq!(
        walk.judgements.violation_count("I-1.3"),
        0,
        "按树 7 第一次走：成立"
    );
    walk.begin_walking_a_version();
    assert!(matches!(
        read_the_node_as_a_node_of_tree(&mut walk, &pointer, 8),
        IndexNodeReading::WalkedByAnEarlierVersion { .. }
    ));
    assert_eq!(
        walk.judgements.violation_count("I-1.3"),
        1,
        "下一版按树 8 引用别的版本先走过的树 7 的节点：I-1.3 红"
    );
}

/// C01 出生身份那一格：节点先被一版按出生 txg 0 的指针走过，下一版的指针记出生 txg 9——命中走过的单元，I-1.2 照判红；
/// 「判过几个被引用单元」只在第一次走时加（它数的是单元），命中不翻倍。
#[test]
fn node_an_earlier_version_walked_reddens_the_birth_identity_invariant_when_this_pointer_names_another_birth_txg(
) {
    let image = ImageHoldingOneUnitAtSlotOne::of(&index_node_of_tree(7));
    let pointer_of_the_earlier_version =
        pointer_to_slot_one(&image, node_pointer_bytes(), node_bytes(), 0);
    let pointer_naming_another_birth_txg =
        pointer_to_slot_one(&image, node_pointer_bytes(), node_bytes(), 9);
    let mut walk = Walk::starting_with(&image, Judgements::default(), 0, 0, 10);
    read_the_node_as_a_node_of_tree(&mut walk, &pointer_of_the_earlier_version, 7);
    assert_eq!(
        walk.judgements.violation_count("I-1.2"),
        0,
        "出生 txg 0 对得上：成立"
    );
    walk.begin_walking_a_version();
    assert!(matches!(
        read_the_node_as_a_node_of_tree(&mut walk, &pointer_naming_another_birth_txg, 7),
        IndexNodeReading::WalkedByAnEarlierVersion { .. }
    ));
    assert_eq!(
        walk.judgements.violation_count("I-1.2"),
        1,
        "下一版的指针记出生 txg 9、节点头里是 0：I-1.2 红"
    );
    assert_eq!(
        walk.referenced_units_judged_against_their_pointer, 1,
        "判过出生身份的被引用单元只有这一个：命中不再加"
    );
}

/// C01 类标签那一格：槽 1 上的码 2 节点先被一版按节点走过，下一版的 extent 记录拿一条数据指针指着它（整单元校验和按数据单元宽对得上）——
/// 命中走过的单元，这一条引用要码 1、头里是码 2，I-1.6 红。
#[test]
fn node_an_earlier_version_walked_reddens_the_class_label_invariant_when_an_extent_record_points_at_it(
) {
    let image = ImageHoldingOneUnitAtSlotOne::of(&index_node_of_tree(7));
    let node_pointer = pointer_to_slot_one(&image, node_pointer_bytes(), node_bytes(), 0);
    let data_pointer = pointer_to_slot_one(
        &image,
        data_pointer_bytes_in_the_tests(),
        data_unit_bytes(),
        0,
    );
    let mut walk = Walk::starting_with(&image, Judgements::default(), 0, 0, 10);
    read_the_node_as_a_node_of_tree(&mut walk, &node_pointer, 7);
    assert_eq!(
        walk.judgements.violation_count("I-1.6"),
        0,
        "按节点第一次走：成立"
    );
    walk.begin_walking_a_version();
    walk.walk_extent_data_pointer(&data_pointer, 5, 0, 7);
    assert_eq!(
        walk.judgements.violation_count("I-1.6"),
        1,
        "extent 记录指着别的版本先走过的码 2 节点：这一条引用要码 1，I-1.6 红"
    );
}

/// 数据指针宽（`singlefs_format::DATA_POINTER_BYTES`）：手造 extent 记录的数据指针用。
fn data_pointer_bytes_in_the_tests() -> usize {
    usize::try_from(singlefs_format::DATA_POINTER_BYTES).expect("88")
}

/// 一个零条目的码 2 节点：树 ID `tree`、层级 `level`、头里的 key 区间 [`smallest_key`, `largest_key`]（key 宽取它们的字节数）、
/// 条目宽 `entry_width`，头里出生身份全 0，头校验和与载荷校验和都封好（fsid 0）。偏移照 `crate::index_node_view` 的字段表：
/// 诞生代号在 52 + 2k、载荷 CRC 在 76 + 2k、条目宽在 84 + 2k，明文头到 86 + 2k。
fn index_node_of_tree_at_level_over_the_key_range(
    tree: u64,
    level: u8,
    smallest_key: &[u8],
    largest_key: &[u8],
    entry_width: u16,
) -> Vec<u8> {
    let key_width = smallest_key.len();
    assert_eq!(largest_key.len(), key_width, "两把 key 同宽");
    let mut unit = vec![0u8; node_bytes()];
    unit[..4].copy_from_slice(b"SFSU");
    unit[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2]
        .copy_from_slice(&UNIT_FORMAT_VERSION_THIS_CHECKER_READS.to_le_bytes());
    unit[6] = 2;
    unit[42..50].copy_from_slice(&tree.to_le_bytes());
    unit[50] = level;
    unit[51] = u8::try_from(key_width).expect("key 宽装得进 1 字节");
    unit[52..52 + key_width].copy_from_slice(smallest_key);
    unit[52 + key_width..52 + 2 * key_width].copy_from_slice(largest_key);
    let header_end = 86 + 2 * key_width;
    unit[header_end - 2..header_end].copy_from_slice(&entry_width.to_le_bytes());
    let payload_checksum = crc32_castagnoli_table(&unit[header_end..]);
    unit[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    let header_checksum = crc32_castagnoli_table(&unit[..header_end]);
    unit[10..14].copy_from_slice(&header_checksum.to_le_bytes());
    let view = index_node_view(&unit).expect("手造的节点解得开");
    assert_eq!(
        (
            view.level,
            view.smallest_key.as_slice(),
            view.largest_key.as_slice()
        ),
        (level, smallest_key, largest_key),
        "手造的节点头里的层级与 key 区间是给定的那几样"
    );
    unit
}

/// C896：分配记录树一个叶（盘 0 第 0 格）先被一版按它自己的位置走过，下一版的父条目把它放在第 1 格——命中走过的单元、不往下走，
/// 这一版的位置规定的那一段照判，头里的区间不是第 1 格罩的那一段，I-1.1 多红一格。
#[test]
fn an_allocation_record_leaf_an_earlier_version_walked_reddens_the_logical_address_invariant_when_this_parent_entry_places_it_in_another_cell(
) {
    let cell_walked_first = AllocationRecordTreeCell::BelowTheRoot {
        level: 0,
        device: 0,
        index: 0,
    };
    let (smallest_key, largest_key) = allocation_record_tree_cell_range(cell_walked_first);
    let image = ImageHoldingOneUnitAtSlotOne::of(&index_node_of_tree_at_level_over_the_key_range(
        7,
        0,
        &smallest_key,
        &largest_key,
        u16::try_from(allocation_record_bytes()).expect("20"),
    ));
    let pointer = pointer_to_slot_one(&image, node_pointer_bytes(), node_bytes(), 0);
    let mut walk = Walk::starting_with(&image, Judgements::default(), 0, 0, 10);
    walk.walk_allocation_record_node(&pointer, cell_walked_first, 0, 7, "分配记录树", &[0, 1]);
    let red_after_the_first_walk = walk.judgements.violation_count("I-1.1");
    walk.begin_walking_a_version();
    walk.walk_allocation_record_node(
        &pointer,
        AllocationRecordTreeCell::BelowTheRoot {
            level: 0,
            device: 0,
            index: 1,
        },
        0,
        7,
        "分配记录树",
        &[0, 1],
    );
    assert_eq!(
        walk.judgements.violation_count("I-1.1"),
        red_after_the_first_walk + 1,
        "下一版把第 0 格的叶放在第 1 格：头里的区间不是第 1 格罩的那一段，I-1.1 红"
    );
}

/// C896：extent 树上段一个叶（层级 0 第 0 格）先被一版走过，下一版的父条目把它放在第 1 格——命中，I-1.1 多红一格。
#[test]
fn an_extent_upper_leaf_an_earlier_version_walked_reddens_the_logical_address_invariant_when_this_parent_entry_places_it_in_another_cell(
) {
    let (smallest_key, largest_key) = extent_upper_cell_range(0, 0);
    let image = ImageHoldingOneUnitAtSlotOne::of(&index_node_of_tree_at_level_over_the_key_range(
        7,
        0,
        &smallest_key,
        &largest_key,
        u16::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113"),
    ));
    let pointer = pointer_to_slot_one(&image, node_pointer_bytes(), node_bytes(), 0);
    let mut walk = Walk::starting_with(&image, Judgements::default(), 0, 0, 10);
    walk.walk_extent_upper_node(&pointer, Some((0, 0)), 7);
    let red_after_the_first_walk = walk.judgements.violation_count("I-1.1");
    walk.begin_walking_a_version();
    walk.walk_extent_upper_node(&pointer, Some((0, 1)), 7);
    assert_eq!(
        walk.judgements.violation_count("I-1.1"),
        red_after_the_first_walk + 1,
        "下一版把第 0 格的上段叶放在第 1 格：头里的区间不是第 1 格罩的那一段，I-1.1 红"
    );
}

/// C896：inode 5 的 extent 下段根先被一版走过，下一版的上段叶条目说它是 inode 6 的下段根——命中，下段根的区间按这一版的 inode 照判，
/// I-1.1 多红一格。
#[test]
fn an_extent_lower_root_an_earlier_version_walked_reddens_the_logical_address_invariant_when_this_upper_entry_names_another_inode(
) {
    let (smallest_key, largest_key) = extent_lower_cell_range(5, 0, 0);
    let image = ImageHoldingOneUnitAtSlotOne::of(&index_node_of_tree_at_level_over_the_key_range(
        7,
        0,
        &smallest_key,
        &largest_key,
        u16::try_from(extent_leaf_record_bytes()).expect("112"),
    ));
    let pointer = pointer_to_slot_one(&image, node_pointer_bytes(), node_bytes(), 0);
    let mut walk = Walk::starting_with(&image, Judgements::default(), 0, 0, 10);
    walk.walk_extent_lower_node(&pointer, None, 5, 7);
    let red_after_the_first_walk = walk.judgements.violation_count("I-1.1");
    walk.begin_walking_a_version();
    walk.walk_extent_lower_node(&pointer, None, 6, 7);
    assert_eq!(
        walk.judgements.violation_count("I-1.1"),
        red_after_the_first_walk + 1,
        "下一版说 inode 5 的下段根是 inode 6 的：头里的区间不是 inode 6 那一段，I-1.1 红"
    );
}

/// 墓碑树、谱系树那种容器树的一个内部节点视图：层级 `level`、key 宽 8、头里的区间 [`smallest_key`, `largest_key`]，
/// 第 0 条条目（类型段 0、身份全 0）的子指针是 `child_pointer`。
fn container_tree_parent_view(
    level: u8,
    smallest_key: &[u8],
    largest_key: &[u8],
    child_pointer: &[u8],
) -> crate::IndexNodeView {
    let mut entry = vec![0u8; 34];
    entry.extend_from_slice(child_pointer);
    crate::IndexNodeView {
        tree_identifier: 7,
        level,
        key_width: 8,
        smallest_key: smallest_key.to_vec(),
        largest_key: largest_key.to_vec(),
        birth_txg: 0,
        filesystem_identifier: 0,
        instance: 0,
        birth_sequence: 0,
        entry_width: inode_internal_entry_bytes(),
        entries: vec![entry],
    }
}

/// C896：墓碑树一个层级 1 的内部节点先被一版层级 2 的父节点走过，下一版层级 3 的父节点又指着它——命中，层级 = 父层级 − 1 照判，
/// I-1.1 多红一格。
#[test]
fn container_tree_internal_node_an_earlier_version_walked_reddens_the_logical_address_invariant_under_a_parent_of_another_level(
) {
    let (smallest_key, largest_key) = (1u64.to_le_bytes(), 4u64.to_le_bytes());
    let image = ImageHoldingOneUnitAtSlotOne::of(&index_node_of_tree_at_level_over_the_key_range(
        7,
        1,
        &smallest_key,
        &largest_key,
        u16::try_from(inode_internal_entry_bytes()).expect("120"),
    ));
    let pointer = pointer_to_slot_one(&image, node_pointer_bytes(), node_bytes(), 0);
    let child = parse_node_pointer(&pointer);
    let mut walk = Walk::starting_with(&image, Judgements::default(), 0, 0, 10);
    walk.walk_the_internal_node_below_a_container_tree_entry(
        &container_tree_parent_view(2, &smallest_key, &largest_key, &pointer),
        0,
        &child,
        7,
        1,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "墓碑树",
    );
    let red_after_the_first_walk = walk.judgements.violation_count("I-1.1");
    walk.begin_walking_a_version();
    walk.walk_the_internal_node_below_a_container_tree_entry(
        &container_tree_parent_view(3, &smallest_key, &largest_key, &pointer),
        0,
        &child,
        7,
        1,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "墓碑树",
    );
    assert_eq!(
        walk.judgements.violation_count("I-1.1"),
        red_after_the_first_walk + 1,
        "下一版层级 3 的父节点指着层级 1 的节点：层级不是父层级减一，I-1.1 红"
    );
}

/// C896：inode 树一个层级 1 的内部节点先被一版层级 2 的父节点走过，下一版层级 3 的父节点又指着它——命中，层级 = 父层级 − 1 照判，
/// I-1.1 多红一格。
#[test]
fn an_inode_internal_node_an_earlier_version_walked_reddens_the_logical_address_invariant_under_a_parent_of_another_level(
) {
    let unit = index_node_of_tree_at_level_over_the_key_range(
        7,
        1,
        &1u64.to_le_bytes(),
        &4u64.to_le_bytes(),
        u16::try_from(inode_internal_entry_bytes()).expect("120"),
    );
    let image = ImageHoldingOneUnitAtSlotOne::of(&unit);
    let child = parse_node_pointer(&pointer_to_slot_one(
        &image,
        node_pointer_bytes(),
        node_bytes(),
        0,
    ));
    let mut walk = Walk::starting_with(&image, Judgements::default(), 0, 0, 10);
    walk.walk_inode_internal_child(&unit, &child, 2, 7, (0, 1));
    let red_after_the_first_walk = walk.judgements.violation_count("I-1.1");
    walk.begin_walking_a_version();
    walk.walk_inode_internal_child(&unit, &child, 3, 7, (0, 1));
    assert_eq!(
        walk.judgements.violation_count("I-1.1"),
        red_after_the_first_walk + 1,
        "下一版层级 3 的父节点指着层级 1 的节点：层级不是父层级减一，I-1.1 红"
    );
}

/// inode 树的一片叶容器（码 3 打包记录类型 2）：出生树 `tree`、容器号 `container`、容器出生代 0，每条 inode 记录只写记录号、别的字段全 0，
/// 头校验和与载荷校验和都封好（fsid 0）。偏移照 `crate::packed_unit_view` 与 `Walk::judge_packed_container` 的字段表：
/// 类身份段 42 起（标签 3、出生树 43、类型 51、容器号 53、容器出生代 61、记录数 69、记录宽 71）、载荷 CRC 89、明文头到 107、记录从 136 起。
fn inode_leaf_container_of_tree_holding(tree: u64, container: u64, inodes: &[u64]) -> Vec<u8> {
    let record_width = registered_record_width(PACKED_TYPE_INODE).expect("inode 记录宽登记过");
    let mut unit = vec![0u8; data_unit_bytes()];
    unit[..4].copy_from_slice(b"SFSU");
    unit[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2]
        .copy_from_slice(&UNIT_FORMAT_VERSION_THIS_CHECKER_READS.to_le_bytes());
    unit[6] = 3;
    let declared_length = u16::try_from(inodes.len() * record_width).expect("装得进一片");
    unit[8..10].copy_from_slice(&declared_length.to_le_bytes());
    unit[42] = 3;
    unit[43..51].copy_from_slice(&tree.to_le_bytes());
    unit[51..53].copy_from_slice(&PACKED_TYPE_INODE.to_le_bytes());
    unit[53..61].copy_from_slice(&container.to_le_bytes());
    unit[69..71].copy_from_slice(&u16::try_from(inodes.len()).expect("条数").to_le_bytes());
    unit[71..73].copy_from_slice(&u16::try_from(record_width).expect("记录宽").to_le_bytes());
    for (index, inode) in inodes.iter().enumerate() {
        let start = 136 + index * record_width;
        unit[start..start + 8].copy_from_slice(&inode.to_le_bytes());
    }
    let payload_checksum = crc32_castagnoli_table(&unit[107..]);
    unit[89..93].copy_from_slice(&payload_checksum.to_le_bytes());
    let header_checksum = crc32_castagnoli_table(&unit[..107]);
    unit[10..14].copy_from_slice(&header_checksum.to_le_bytes());
    assert!(
        packed_unit_view(&unit).is_ok(),
        "手造的叶容器解得开：{:?}",
        packed_unit_view(&unit)
    );
    unit
}

/// inode 树内部节点的一条条目（120 字节，D8（核心索引结构） 已定项 6 的字段表）：分隔 key 8、出生树 8、类型段 2、容器号 8、容器出生代 8、
/// 子指针 86。
fn inode_internal_entry(
    separator: u64,
    birth_tree: u64,
    record_type: u16,
    container: u64,
    child_pointer: &[u8],
) -> Vec<u8> {
    let mut entry = Vec::with_capacity(inode_internal_entry_bytes());
    entry.extend_from_slice(&separator.to_le_bytes());
    entry.extend_from_slice(&birth_tree.to_le_bytes());
    entry.extend_from_slice(&record_type.to_le_bytes());
    entry.extend_from_slice(&container.to_le_bytes());
    entry.extend_from_slice(&0u64.to_le_bytes());
    entry.extend_from_slice(child_pointer);
    assert_eq!(entry.len(), inode_internal_entry_bytes(), "条目宽 120");
    entry
}

/// 带条目的 inode 树码 2 内部节点：树 ID `tree`、层级 `level`、key 宽 8、头里的 key 区间 [`smallest_inode`, `largest_inode`]（子树覆盖区间）、
/// 条目宽 120，头里出生身份全 0，头校验和与载荷校验和都封好（fsid 0）。偏移照 `crate::index_node_view` 的字段表：条目数在 82 + 2k、
/// 条目宽在 84 + 2k、明文头到 86 + 2k，条目区从 115 + 2k 起。
fn inode_internal_node_of_tree(
    tree: u64,
    level: u8,
    smallest_inode: u64,
    largest_inode: u64,
    entries: &[Vec<u8>],
) -> Vec<u8> {
    let key_width = 8usize;
    let entry_width = inode_internal_entry_bytes();
    let mut unit = vec![0u8; node_bytes()];
    unit[..4].copy_from_slice(b"SFSU");
    unit[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2]
        .copy_from_slice(&UNIT_FORMAT_VERSION_THIS_CHECKER_READS.to_le_bytes());
    unit[6] = 2;
    let declared_length = u16::try_from(entries.len() * entry_width).expect("装得进一个节点");
    unit[8..10].copy_from_slice(&declared_length.to_le_bytes());
    unit[42..50].copy_from_slice(&tree.to_le_bytes());
    unit[50] = level;
    unit[51] = u8::try_from(key_width).expect("8");
    unit[52..60].copy_from_slice(&smallest_inode.to_le_bytes());
    unit[60..68].copy_from_slice(&largest_inode.to_le_bytes());
    let header_end = 86 + 2 * key_width;
    unit[header_end - 4..header_end - 2]
        .copy_from_slice(&u16::try_from(entries.len()).expect("条目数").to_le_bytes());
    unit[header_end - 2..header_end]
        .copy_from_slice(&u16::try_from(entry_width).expect("120").to_le_bytes());
    let entries_start =
        header_end + usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29");
    for (index, entry) in entries.iter().enumerate() {
        let start = entries_start + index * entry_width;
        unit[start..start + entry_width].copy_from_slice(entry);
    }
    let payload_checksum = crc32_castagnoli_table(&unit[header_end..]);
    unit[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    let header_checksum = crc32_castagnoli_table(&unit[..header_end]);
    unit[10..14].copy_from_slice(&header_checksum.to_le_bytes());
    let view = index_node_view(&unit).expect("手造的节点解得开");
    assert_eq!(
        (view.level, view.entries.len()),
        (level, entries.len()),
        "手造的节点头里的层级与条目数是给定的那几样"
    );
    unit
}

/// inode 树一个层级 2 的根视图：头里的区间 [`smallest_inode`, `largest_inode`]，一条类型段 0 的条目（三段身份为 0），分隔 key 是
/// `separator`、子指针是 `child_pointer`。
fn inode_root_view_over_one_internal_child(
    separator: u64,
    smallest_inode: u64,
    largest_inode: u64,
    child_pointer: &[u8],
) -> crate::IndexNodeView {
    crate::IndexNodeView {
        tree_identifier: 7,
        level: 2,
        key_width: 8,
        smallest_key: smallest_inode.to_le_bytes().to_vec(),
        largest_key: largest_inode.to_le_bytes().to_vec(),
        birth_txg: 0,
        filesystem_identifier: 0,
        instance: 0,
        birth_sequence: 0,
        entry_width: inode_internal_entry_bytes(),
        entries: vec![inode_internal_entry(separator, 0, 0, 0, child_pointer)],
    }
}

/// C896 的口径扩到 I-9.12（分隔 key 落在孩子区间之外）：inode 树一个层级 1 的内部节点（槽 1，下面一片叶容器装 inode 3 与 9）先被一版的根
/// 以分隔 key 3 走过，下一版的根以分隔 key 5 又指着它——命中走过的节点、不往下走，这个孩子的区间取先走那一遍记下的叶序 [3, 9]，
/// 这一版的分隔 key 5 大于它的最小 key 3，I-9.12 多红一格。
#[test]
fn an_inode_internal_node_an_earlier_version_walked_reddens_the_separator_invariant_when_this_parent_entry_names_a_separator_past_its_smallest_key(
) {
    const INODE_TREE: u64 = 7;
    let leaf_container = inode_leaf_container_of_tree_holding(INODE_TREE, 3, &[3, 9]);
    let image_holding_only_the_leaf =
        ImageHoldingUnitsAtSlots::of(&[(2, leaf_container.as_slice())]);
    let pointer_to_the_leaf =
        image_holding_only_the_leaf.node_pointer_to_slot(2, data_unit_bytes(), INODE_TREE);
    let shared_internal_node = inode_internal_node_of_tree(
        INODE_TREE,
        1,
        3,
        9,
        &[inode_internal_entry(
            3,
            INODE_TREE,
            PACKED_TYPE_INODE,
            3,
            &pointer_to_the_leaf,
        )],
    );
    let image = ImageHoldingUnitsAtSlots::of(&[
        (1, shared_internal_node.as_slice()),
        (2, leaf_container.as_slice()),
    ]);
    let pointer_to_the_shared_node = image.node_pointer_to_slot(1, node_bytes(), 0);
    let mut walk = Walk::starting_with(&image, Judgements::default(), 0, 0, 10);
    walk.walk_inode_internal_node(
        &inode_root_view_over_one_internal_child(3, 3, 9, &pointer_to_the_shared_node),
        INODE_TREE,
        (0, 100),
        "先走那一版的 inode 树根",
        &mut Vec::new(),
    );
    assert_eq!(
        walk.judgements.violation_count("I-9.12"),
        0,
        "先走那一版：根的分隔 key 3、内部节点的分隔 key 3 都不大于叶装的最小 key 3，成立"
    );
    assert!(
        walk.inode_leaf_order_below_a_walked_unit
            .get(&(0, 1))
            .is_some_and(|below| below.leaves_in_leaf_order.len() == 1),
        "先走那一版走到了内部节点下面那片叶，叶序记下了"
    );
    walk.begin_walking_a_version();
    walk.walk_inode_internal_node(
        &inode_root_view_over_one_internal_child(5, 3, 9, &pointer_to_the_shared_node),
        INODE_TREE,
        (0, 101),
        "下一版的 inode 树根",
        &mut Vec::new(),
    );
    assert_eq!(
        walk.judgements.violation_count("I-9.12"),
        1,
        "下一版根的分隔 key 5 大于命中的内部节点下面叶序里的最小 key 3：I-9.12 红"
    );
}
