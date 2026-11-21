use e7_index_bench::usize_from_u64;
use super::*;

#[test]
/// D8（核心索引结构） 已定项 11 + D18（块里携带什么信息） 已定项 16：头 86 + 2k，含 29 字节预留位 115 + 2k。
/// 六个数逐个抄自字节表（inode 树根 131、extent 163、分配记录 135、记账 159、映射 169、树表 131）。
fn index_node_header_is_86_plus_twice_the_key_width() {
    let with_reserved = |key_width: usize| index_node_header_bytes(key_width) + usize_from_u64(NONCE_MAC_RESERVED_BYTES);
    assert_eq!(index_node_header_bytes(8), 102);
    assert_eq!(with_reserved(8), 131, "inode 树与树表");
    assert_eq!(with_reserved(ALLOCATION_KEY_BYTES), 135, "分配记录树（key 宽 10）");
    assert_eq!(with_reserved(22), 159, "记账树");
    assert_eq!(with_reserved(24), 163, "extent 树");
    assert_eq!(with_reserved(usize_from_u64(MAPPING_KEY_BYTES)), 169, "中央映射树（key 宽 27）");
    assert_eq!(index_node_header_bytes(0), usize_from_u64(INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE));
    // 树表一层装几条：分母是 16384 减含预留位的码 2 头，不减别的数（D8 已定项 8 写死的口径）。
    assert_eq!((usize_from_u64(NODE_BYTES) - with_reserved(TREE_TABLE_KEY_WIDTH)) / usize_from_u64(TREE_TABLE_ENTRY_BYTES), 81);
    // inode 树内部节点一层装几条：头 131、条目 120。
    assert_eq!((usize_from_u64(NODE_BYTES) - with_reserved(8)) / usize_from_u64(INODE_INTERNAL_ENTRY), 135);
}

/// key 宽住偏移 51（D18 已定项 18）：解析器不用先知道这是哪棵树就找得到头末端。
/// 这是 G2 收口那一格的会红检查——把 key 宽写进别的偏移，解析当场失败。
#[test]
fn the_key_width_field_sits_at_offset_51_so_the_header_locates_itself() {
    let BuiltPool { output, .. } = built_pool();
    // E142 第十五次跑：索引 0..12 是 t1..t12（第七节 B2）；分配记录树五个节点（索引 4..9）
    // 一律 key 宽 10（D3 已定项 11：位置 key 与分配记录 key 同构，不分叶或内部）。
    for (index, expected_key_width) in [(1usize, 24usize), (3, 8), (4, ALLOCATION_KEY_BYTES), (5, ALLOCATION_KEY_BYTES), (6, ALLOCATION_KEY_BYTES), (7, ALLOCATION_KEY_BYTES), (8, ALLOCATION_KEY_BYTES), (9, 22), (10, usize_from_u64(MAPPING_KEY_BYTES)), (11, TREE_TABLE_KEY_WIDTH)] {
        let unit = &output.units_by_slot[index].2;
        assert_eq!(usize::from(unit[INDEX_NODE_KEY_WIDTH_OFFSET]), expected_key_width, "偏移 51 那一字节就是 key 宽（索引 {index}）");
        let node = parse_index_node(unit).expect("自描述的 key 宽足以定出头末端");
        assert_eq!(node.key_width, expected_key_width);
    }
    assert_eq!(INDEX_NODE_KEY_WIDTH_OFFSET, 51);
    // 把 key 宽改成别的值：头末端跟着移，头校验和当场不过（索引 10 是映射树）。
    let mut tampered = output.units_by_slot[10].2.clone();
    tampered[INDEX_NODE_KEY_WIDTH_OFFSET] = 25;
    assert_eq!(parse_index_node(&tampered).unwrap_err(), UnitError::HeaderChecksum);
    // 自述一个大到让头越出这段字节的 key 宽：判结构错，不许下标越界。
    let mut absurd = output.units_by_slot[10].2.clone();
    absurd[INDEX_NODE_KEY_WIDTH_OFFSET] = 255;
    absurd.truncate(200);
    assert_eq!(parse_index_node(&absurd).unwrap_err(), UnitError::Structure("自述的 key 宽让头越出单元"));
}

#[test]
fn flipping_the_last_header_byte_is_caught_by_the_header_checksum() {
    let BuiltPool { output, .. } = built_pool();
    let mut unit = output.units_by_slot[0].2.clone();
    unit[usize_from_u64(DATA_UNIT_HEADER_BYTES) - 1] ^= 0x01;
    assert_eq!(parse_data_unit(&unit).unwrap_err(), UnitError::HeaderChecksum);
    let mut payload_flipped = output.units_by_slot[0].2.clone();
    payload_flipped[usize_from_u64(DATA_UNIT_HEADER_BYTES) + usize_from_u64(NONCE_MAC_RESERVED_BYTES) + 10] ^= 0x01;
    assert_eq!(parse_data_unit(&payload_flipped).unwrap_err(), UnitError::PayloadChecksum);
    let mut reserved_flipped = output.units_by_slot[0].2.clone();
    reserved_flipped[usize_from_u64(DATA_UNIT_HEADER_BYTES) + 3] ^= 0x01;
    assert_eq!(parse_data_unit(&reserved_flipped).unwrap_err(), UnitError::PayloadChecksum, "预留位在载荷 CRC 覆盖内");
}

/// 中央映射树：**10** 条条目（第七节 B7「mapping_entries=10」，分配记录树的五个新节点各占一条）——
/// 一宽 55、key 一律 27（D19 已定项 10）；码 2 / 码 3 的 25 字节 key 末尾补两个零，
/// 补零进 key 本身而不只是区间字段。
#[test]
fn mapping_tree_holds_one_entry_width_and_pads_short_keys_into_the_key_itself() {
    let BuiltPool { output, .. } = built_pool();
    let node = parse_index_node(&output.units_by_slot[10].2).expect("映射树根");
    assert_eq!(node.entries.len(), 10);
    assert!(node.entries.iter().all(|entry| entry.len() == 55), "条目一律 55");
    assert_eq!(node.key_width, 27, "头自述的 key 宽");
    assert_eq!(node.smallest_key.len(), 27);
    assert_eq!(node.largest_key.len(), 27);
    assert_eq!(&node.entries[0][..27], &node.smallest_key[..], "条目里 key 一律打头，区间就是首末两条的前 27 字节");
    assert_eq!(&node.entries[9][..27], &node.largest_key[..]);
    assert_eq!(node.entries[0][0], UNIT_CLASS_DATA, "最小那条是码 1（类标签 1 排在最前）");
    assert_eq!(&node.largest_key[25..], &[0, 0], "码 2 / 码 3 的 key 末尾补的那两个零");
    // 声明长度 = 条目数 × 条目宽，条目区从含预留位的头末尾（169）起。
    let declared_length = u16::from_le_bytes(output.units_by_slot[10].2[8..10].try_into().expect("切了 2 字节"));
    assert_eq!(declared_length, 550, "10 × 55（第七节 B7「mapping_declared=550」）");
    assert_eq!(index_node_header_bytes(27) + usize_from_u64(NONCE_MAC_RESERVED_BYTES), 169);
}

/// 每个码 2 节点里的条目 key 严格递增（D8（核心索引结构） 已定项 11 的全序：逐字段无符号整数、自左向右）。
/// 这是一条**跨结构**的绝对检查：任何一棵树把两条同 key 的条目写进同一个节点都会红，
/// 不必等某一棵树自己那条断言想到这一格。D8 已定项 14 第 395 行（唯一写法）之后分配记录树的
/// 内部 / 根节点也是稀疏表示、不再有空格，与其余码 2 树走同一条检查（M101、M102 的会红检查落在这里）。
#[test]
fn every_index_node_keeps_its_entry_keys_strictly_ascending() {
    let BuiltPool { output, .. } = built_pool();
    let mut checked_nodes = 0;
    for (slot, unit_identity, unit) in &output.units_by_slot {
        let (unit_class, key_width) = unit_identity.class_and_key_width();
        if unit_class != UNIT_CLASS_INDEX_NODE {
            continue;
        }
        let node = parse_index_node(unit).expect("码 2 节点");
        assert_eq!(node.key_width, key_width);
        for pair in node.entries.windows(2) {
            assert!(pair[0][..key_width] < pair[1][..key_width], "{} @ 槽 {} 的条目 key 没有严格递增", unit_identity.tag(), slot.0);
        }
        checked_nodes += 1;
    }
    assert_eq!(checked_nodes, 10, "事务里十个码 2 节点：extent、inode 根、分配记录树五个节点、记账、映射、树表");
}

/// 条目宽字段写坏了，解析器要当场拒绝（D8（核心索引结构） 已定项 11：声明长度 = 条目数 × 条目宽）。
/// 变异 M62 钉着它：这条结构检查整段删掉要有测试红。
#[test]
fn the_parser_refuses_an_entry_width_field_that_contradicts_the_declared_length() {
    let BuiltPool { output, .. } = built_pool();
    let mapping_unit = &output.units_by_slot[10].2;
    let entry_width_offset = index_node_header_bytes(usize_from_u64(MAPPING_KEY_BYTES)) - 2;
    assert_eq!(u16::from_le_bytes(mapping_unit[entry_width_offset..entry_width_offset + 2].try_into().expect("切了 2 字节")), 55);
    // 条目宽与载荷 CRC 都住头里（偏移 76 + 2k 与 84 + 2k），改条目宽只让头校验和失配 ⇒ 重新封一次头校验和再解。
    for (bad_width, what) in [(0u16, "条目宽 0（第六次跑那个变长哨兵）"), (54, "比声明长度算出来的小一"), (27, "刚好等于 key 宽")] {
        let mut tampered = mapping_unit.clone();
        tampered[entry_width_offset..entry_width_offset + 2].copy_from_slice(&bad_width.to_le_bytes());
        seal_header_checksum(&mut tampered, index_node_header_bytes(usize_from_u64(MAPPING_KEY_BYTES)));
        assert_eq!(parse_index_node(&tampered).unwrap_err(), UnitError::Structure("声明长度与条目数 × 条目宽对不上"), "{what}");
    }
    // 反向：原样的 55 解得开——上面三条不是因为「改了字节就一定红」。
    assert!(parse_index_node(mapping_unit).is_ok());
}
