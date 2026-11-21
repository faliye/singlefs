use e7_index_bench::{low_byte_of_usize, usize_from_u64};
use super::*;

/// D23 已定项 13：`header_csum` 覆盖**整条记录 [0, 4096) 含补齐**。
/// 三处各翻一个字节，三处都要被抓——尤其最后那处：点名项之后的补齐区，只罩头的那个读法抓不到它。
#[test]
fn the_journal_header_checksum_covers_the_whole_record_including_the_padding() {
    let BuiltPool { output, .. } = built_pool();
    assert!(JournalRecord::parse(&output.record_bytes).is_some());
    // B8（第七节 7.2）：头 307 布局是 46+32+8+1+4=91；加本次发布内序号 4 字节 ⇒ 95（载荷校验和偏移不随新根段变宽而动）。
    assert_eq!(JOURNAL_HEADER_CHECKSUM_OFFSET + usize_from_u64(WIDE_CHECKSUM_BYTES) + 8 + 1 + 4, 91, "头 307 布局的载荷校验和偏移（B8，仅作历史对照，不是这条记录的真实偏移）");
    let payload_checksum_offset = JOURNAL_HEADER_CHECKSUM_OFFSET + usize_from_u64(WIDE_CHECKSUM_BYTES) + 8 + 1 + 4 + 4;
    assert_eq!(payload_checksum_offset, 95, "载荷校验和字段的偏移（487 字节头，已定项 4）");
    for (offset, what) in [
        (payload_checksum_offset, "载荷校验和字段"),
        (usize_from_u64(JOURNAL_HEADER_BYTES) + 3, "点名项数组"),
        (usize_from_u64(JOURNAL_RECORD_BYTES) - 1, "点名项之后的补齐区最后一字节"),
    ] {
        let mut record = output.record_bytes.clone();
        record[offset] ^= 0x01;
        assert!(JournalRecord::parse(&record).is_none(), "{what}落在 header_csum 覆盖内");
    }
    // 反向：只罩头 [0, 487) 的那个读法对补齐区那一字节说不出话——这一行证明上面第三条不是白抓的。
    let mut padded = output.record_bytes.clone();
    padded[usize_from_u64(JOURNAL_RECORD_BYTES) - 1] ^= 0x01;
    let head_only_before = wide_checksum_with_field_zeroed(&output.record_bytes, usize_from_u64(JOURNAL_HEADER_BYTES), JOURNAL_HEADER_CHECKSUM_OFFSET);
    let head_only_after = wide_checksum_with_field_zeroed(&padded, usize_from_u64(JOURNAL_HEADER_BYTES), JOURNAL_HEADER_CHECKSUM_OFFSET);
    assert_eq!(head_only_before, head_only_after, "只罩头的那个读法分不出这一改，罩整条的才分得出");
}

/// A7（第七节）的会红检查（M86）：反向链 = CRC32C(前一条记录的头 [0,487)，头校验和 32 字节按零参与)。
/// 487 在这里写成字面量、逐字节手拼，不调用 `journal_back_chain`——两条路径独立算，
/// 才分得出「算法本身错了」与「验证用的是同一个（可能同样错的）函数」。
/// 另把 t13 与 w4 两条的反向链钉在第二十一次跑登记 7.2 B1、B1b（`back_chain_487.py` 在 N19R 产物的输入上独立推出的绝对值，
/// 不是任何一次产物的输出，也不是 `crates/` 的钉值）。
#[test]
fn back_chain_matches_an_independently_computed_crc_over_487_bytes() {
    let BuiltPool { recording, output, .. } = built_pool();
    let records = scan_journal(&recording.pool, unit_fsid(&FIXED_FSID));
    let warm_up_record_at_counter_one_bytes = records[&(InstanceGeneration(1), JournalCounter(1))].to_bytes();
    let warm_up_record_at_counter_two = &records[&(InstanceGeneration(1), JournalCounter(2))];
    let warm_up_record_at_counter_two_bytes = warm_up_record_at_counter_two.to_bytes();
    let mut header_487 = warm_up_record_at_counter_two_bytes[..487].to_vec();
    assert_eq!(header_487.len(), 487, "A1：头 487 字节");
    header_487[46..46 + 32].fill(0); // 头校验和字段按零参与
    let independently_computed = castagnoli_crc32(&header_487);
    assert_eq!(output.record.back_chain, independently_computed, "t13 的反向链 = CRC32C(w4 的 487 字节头，头校验和按零参与)");
    let mut warm_up_record_at_counter_one_header_487 = warm_up_record_at_counter_one_bytes[..487].to_vec();
    warm_up_record_at_counter_one_header_487[46..46 + 32].fill(0);
    assert_eq!(warm_up_record_at_counter_two.back_chain, castagnoli_crc32(&warm_up_record_at_counter_one_header_487), "w4 的反向链 = CRC32C(w1 的 487 字节头，头校验和按零参与)");
    assert_eq!(output.record.back_chain, ANCHOR_BACK_CHAIN_OF_THE_TRANSACTION_JOURNAL_RECORD, "7.2 B1：t13 的反向链");
    assert_eq!(warm_up_record_at_counter_two.back_chain, ANCHOR_BACK_CHAIN_OF_THE_WARM_UP_RECORD_AT_COUNTER_TWO, "7.2 B1b：w4 的反向链");
    assert_eq!(ANCHOR_BACK_CHAIN_OF_THE_TRANSACTION_JOURNAL_RECORD, 2_660_362_816);
    assert_eq!(ANCHOR_BACK_CHAIN_OF_THE_WARM_UP_RECORD_AT_COUNTER_TWO, 750_278_362);
    assert_eq!(records[&(InstanceGeneration(1), JournalCounter(1))].back_chain, 0, "w1 是本实例第一条，反向链恒 0");
}

/// journal 记录头 487（D23 已定项 15 的新根段 364 七项 + fsid 8 与 MAC 16 +
/// 记录标志 1 与本次发布内序号 4）；点名项 56 的 key 尾段凑得出中央映射的 6 条 key（已定项 17 末句），一律 27 字节。
/// 新根段后三项（实例表指针、分配记录树根指针、卸载记号）按绝对偏移钉（登记 7.2 B6、B10、J18–J21；
/// M210、M211、M212、M213、M218 的会红检查落在这里）。
#[test]
fn journal_record_carries_the_new_root_segment_and_the_named_entries_rebuild_the_mapping_keys() {
    let BuiltPool { output, .. } = built_pool();
    let parsed = JournalRecord::parse(&output.record_bytes).expect("记录自检要过");
    assert_eq!(parsed, output.record);
    assert_eq!(parsed.new_tree_table.locations[0].slot, SlotNumber(SLOT_TREE_TABLE_FIRST_PUBLISH));
    assert_eq!(parsed.new_mapping_root.locations[0].slot, SlotNumber(SLOT_MAPPING_ROOT));
    assert_eq!(parsed.new_tree_identifier_watermark, 65549, "十三棵 day-1 注册（最后一棵退役树）⇒ 水位 65549（7.2 A-T4）");
    assert_eq!(parsed.new_rollback_floor, CheckpointTxg(0));
    assert_eq!(parsed.transaction, TransactionNumber(1));
    assert_eq!(parsed.fsid, unit_fsid(&FIXED_FSID), "记录头的 fsid 与单元头同口径（D23 已定项 4）");
    // A4 / A5（第七节）：臂 F 三条记录偏移 7 是 0x01，偏移 87–90 是 01 00 00 00（本次发布内序号 1）。
    assert_eq!(parsed.record_flags, 0x01, "臂 F：本次发布末条标志（已定项 17）");
    assert_eq!(parsed.ordinal_within_publish, 1, "这次发布只有这一条（已定项 4）");
    assert_eq!(output.record_bytes[JOURNAL_RECORD_FLAGS_OFFSET], 0x01);
    assert_eq!(&output.record_bytes[87..91], &1u32.to_le_bytes(), "本次发布内序号小端 01 00 00 00");
    assert!(output.record_bytes[471..487].iter().all(|byte| *byte == 0), "记录头末尾 MAC 16 第一版全 0");
    // 新根段后三项（D23 已定项 15）：实例表指针 [287, 373) 与这次发布写的根记录 [170, 256) 逐字节相同（B10），
    // 两条位置条目的槽号 50176（B10：记录内 341、355 处的六字节槽号，设备 0 与 1）；分配记录树根指针 [373, 459) 全零；卸载记号 [459, 463) 为 0；
    // fsid 在 [463, 471)，与 t1 数据单元头 [83, 91) 同口径。
    let root_slot = output.root.to_slot();
    assert_eq!(&output.record_bytes[287..373], &root_slot[170..256], "实例表指针与那次发布写的根记录里的逐字节相同");
    assert_eq!(&output.record_bytes[341..347], &[0x00, 0xC4, 0x00, 0x00, 0x00, 0x00], "实例表指针第 0 条位置条目槽号 50176 = 0xC400（小端六字节）");
    assert_eq!(&output.record_bytes[337..341], &0u32.to_le_bytes(), "第 0 条位置条目设备 0");
    assert_eq!(&output.record_bytes[355..361], &[0x00, 0xC4, 0x00, 0x00, 0x00, 0x00], "实例表指针第 1 条位置条目槽号 50176");
    assert_eq!(&output.record_bytes[351..355], &1u32.to_le_bytes(), "第 1 条位置条目设备 1");
    assert!(output.record_bytes[373..459].iter().all(|byte| *byte == 0), "分配记录树根指针新池新建文件里恒全零");
    assert_eq!(&output.record_bytes[459..463], &[0, 0, 0, 0], "卸载记号 0（新池新建文件里没有卸载）");
    assert_eq!(&output.record_bytes[463..471], &unit_fsid(&FIXED_FSID).to_le_bytes(), "fsid 在 463");
    assert_eq!(parsed.new_instance_table, output.root.instance_table);
    assert_eq!(parsed.new_instance_table.locations[0].slot, SlotNumber(SLOT_INSTANCE_TABLE));
    assert_eq!(parsed.new_allocation_record_tree_root, NodePointer::empty_root());
    assert!(!parsed.new_unmount_marker);
    // 点名项数组从 487 起：第 0 项的第 0 条位置条目是 t1（槽 50180，设备 0）。
    assert_eq!(&output.record_bytes[487..491], &0u32.to_le_bytes());
    assert_eq!(u64::from_le_bytes([output.record_bytes[491], output.record_bytes[492], output.record_bytes[493], output.record_bytes[494], output.record_bytes[495], output.record_bytes[496], 0, 0]), 50180);
    let rebuilt: Vec<Vec<u8>> = parsed.named.iter().map(NamedUnit::mapping_key).collect();
    assert_eq!(rebuilt.len(), 12, "t1..t12（第七节 B6「named=12」）");
    assert_eq!(rebuilt.iter().filter(|key| key.len() == 27).count(), 12, "一律 27（D19 已定项 10，2026-09-14 用户定案）");
    for key in &output.mapping_keys {
        assert!(rebuilt.contains(key), "映射树里的 key 都凑得出来：{key:?}");
    }
}

#[test]
fn two_data_units_in_one_transaction_share_a_mapping_key() {
    let BuiltPool { output, .. } = built_pool();
    let first = mapping_key_for_data(output.data_pointer.head, output.data_pointer.write_order);
    let second = mapping_key_for_data(PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT), birth_txg: CheckpointTxg(NEW_POOL_FILE_CREATION_TXG) }, WriteOrder { instance: InstanceGeneration(1), transaction: TransactionNumber(1) });
    assert_eq!(first, second);
    assert_eq!(first.len(), 27);
    let distinct: std::collections::BTreeSet<Vec<u8>> = output.mapping_keys.iter().cloned().collect();
    assert_eq!(distinct.len(), 10, "第七节 B7「mapping_entries=10」");
}

/// 树表单元第 1 版 12 条（D8 已定项 8 / 已定项 9 / 已定项 11，D18 已定项 24 第 6 条；第 25 次登记 7.1 K2、K3，7.2 A-T3、A-T4）：
/// 条目按树 ID 升序、树 ID 打头当 key、头 ID 按 D8 已定项 8「头 ID 8 字节的取值」、预留 76 恒 0。绝对值取条款原文，
/// 不取任何一次产物的输出（M204、M205、M207、M234、M236、M237 的会红检查落在这里）。
#[test]
fn the_tree_table_holds_twelve_entries_keyed_by_tree_identifier() {
    let BuiltPool { output, .. } = built_pool();
    let node = parse_index_node(&output.units_by_slot[11].2).expect("树表单元");
    assert_eq!(node.entries.len(), 12);
    assert_eq!(node.key_width, TREE_TABLE_KEY_WIDTH);
    let parsed: Vec<TreeTableEntry> = node.entries.iter().map(|bytes| TreeTableEntry::parse(bytes).expect("树表条目")).collect();
    assert_eq!(parsed.iter().map(|entry| entry.tree.0).collect::<Vec<_>>(), vec![65536, 65537, 65538, 65539, 65541, 65542, 65543, 65544, 65545, 65546, 65547, 65548]); // 末一条退役树
    assert_eq!(parsed.iter().map(|entry| entry.kind).collect::<Vec<_>>(), vec![1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13]);
    assert_eq!(parsed.iter().map(|entry| entry.head_identifier).collect::<Vec<_>>(), vec![65537, 65537, 0, 0, 0, 0, 0, 65537, 65537, 0, 0, 65537]);
    assert!(parsed.iter().all(|entry| entry.birth_txg == CheckpointTxg(3)), "十二条诞生 txg 都是 3");
    assert_eq!(parsed.iter().filter(|entry| entry.root.is_empty_root()).count(), 8, "livelist / 稀疏旁表 / deadlist / deleted_inodes / 墓碑树 / 谱系树 / logged_ops / 退役树八条根指针为零");
    assert!(parsed.windows(2).all(|pair| pair[0].tree < pair[1].tree), "条目按树 ID 严格升序（D8 已定项 8）");
    // 7.1 K2：第 11 条（0 起数）是退役树：树 ID、种类码 13、零根、头 ID 65537、诞生 txg 3（`previous_snapshot_txg` 0 由字段表 L16 那六行核）。
    let retirement_entry = &parsed[11];
    assert_eq!((retirement_entry.tree, retirement_entry.kind, retirement_entry.head_identifier, retirement_entry.birth_txg), (TreeIdentifier(TREE_IDENTIFIER_RETIREMENT), 13, 65537, CheckpointTxg(3)), "退役树条目");
    assert!(retirement_entry.root.is_empty_root(), "退役树零根");
    // 条目里 key 一律打头（D8 已定项 11）：区间就是首末两条的前 8 字节。
    assert_eq!(&node.entries[0][..8], &65536u64.to_le_bytes()[..]);
    assert_eq!(&node.entries[11][..8], &TREE_IDENTIFIER_RETIREMENT.to_le_bytes()[..]);
    assert_eq!(node.smallest_key, 65536u64.to_le_bytes().to_vec());
    assert_eq!(node.largest_key, TREE_IDENTIFIER_RETIREMENT.to_le_bytes().to_vec());
    assert_eq!(TREE_IDENTIFIER_RETIREMENT, 65548, "退役树的树 ID（7.2 A-T4b，D8 已定项 11）");
    let declared_length = u16::from_le_bytes(output.units_by_slot[11].2[8..10].try_into().expect("切了 2 字节"));
    assert_eq!(declared_length, 2400, "12 × 200（7.2 A-T3）");
    assert!(std::hint::black_box(131 + 2400 <= NODE_BYTES), "条目区末尾 131 + 2400 = 2531 ≤ 16384，一个单元装得下（7.2 A-T3）");
    assert_eq!((NODE_BYTES - 131) / TREE_TABLE_ENTRY_BYTES, 81, "每层装 ⌊(16384 − 131) / 200⌋ = 81 棵（7.2 B2）");
    // 树 ID 与水位的盘上字节（小端 8 字节）：65536 = 00 00 01 00 …；十二条依次 00..0c 打头（跳过 0x04）、其后 00 01 00 00 00 00 00。
    for (index, entry) in node.entries.iter().enumerate() {
        let mut expected_identifier = [0u8, 0, 1, 0, 0, 0, 0, 0];
        expected_identifier[0] = if index < 4 { low_byte_of_usize(index) } else { low_byte_of_usize(index) + 1 }; // 中央映射树 65540 住根记录、不在树表里：第 4 条起低字节比下标大 1
        assert_eq!(&entry[..8], &expected_identifier[..], "第 {index} 条的树 ID 小端字节");
    }
    assert_eq!(output.root.tree_identifier_watermark.to_le_bytes(), [0x0d, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00], "65549 的盘上字节（7.2 A-T4）");
    assert_eq!(65536u64.to_le_bytes(), [0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00], "65536 的盘上字节（7.2 B3）");
    // 预留 76 里塞一个非零字节：条目解析当场拒绝。
    let mut tampered = node.entries[0].clone();
    tampered[usize_from_u64(TREE_TABLE_ENTRY_BYTES) - 1] = 1;
    assert_eq!(TreeTableEntry::parse(&tampered).unwrap_err(), UnitError::Structure("树表条目预留 76 字节非零"));
}

#[test]
fn instance_table_row_is_88_bytes_and_the_inode_leaf_holds_one_140_byte_record() {
    let BuiltPool { genesis, output, .. } = built_pool();
    let instance_table = parse_packed_unit(&genesis.instance_table_unit).expect("实例表");
    assert_eq!(instance_table.record_width as u64, INSTANCE_ROW_BYTES);
    assert_eq!(instance_table.records.len(), 1);
    assert_eq!(instance_table.identity.record_type, PACKED_TYPE_INSTANCE_TABLE);
    let leaf = parse_packed_unit(&output.units_by_slot[2].2).expect("inode 叶");
    assert_eq!(output.units_by_slot[2].1, TransactionUnit::InodeLeaf, "t3 就是 units 里第三个");
    assert_eq!(leaf.record_width as u64, INODE_RECORD_BYTES);
    assert_eq!(leaf.records.len(), 1);
    assert_eq!(InodeRecord::parse(&leaf.records[0]).expect("记录").size, 3000);
    // C480（inode 记录的 blocks 怎么算全仓没有条款）：blocks = ⌈文件字节数 ÷ 512⌉ = ⌈3000 ÷ 512⌉ = 6
    // （逻辑长度的 512 字节块数，不表示分到的空间）；blocks 不在 InodeRecord 结构体里（它是写时算的一个派生量，
    // 不是从盘上读回来再参与判定的那种字段），这里直接按偏移 48 核对写出来的原始字节。
    assert_eq!(u64::from_le_bytes(leaf.records[0][48..56].try_into().expect("切了 8 字节")), 6, "blocks");
    let tree_table = parse_index_node(&genesis.tree_table_genesis_unit).expect("树表第 0 版");
    assert_eq!(tree_table.entries.len(), 0);
}

// ───────── E142 第十一次跑新增：量 1-4/8 与阳性对照 A/B 用到的算术模型的单测 ─────────

#[test]
fn inode_record_change_count_bytes_differ_only_in_the_low_byte_between_1_and_3() {
    let record_1 = InodeRecord { inode: 1, object_birth: CheckpointTxg(3), size: 3000, change_count: 1 }.to_bytes();
    let record_3 = InodeRecord { inode: 1, object_birth: CheckpointTxg(3), size: 3000, change_count: 3 }.to_bytes();
    let differing: Vec<usize> = (0..record_1.len()).filter(|&index| record_1[index] != record_3[index]).collect();
    assert_eq!(differing, vec![88], "改动计数 1→3 的 u64 小端表示只翻低位那一个字节（重跑登记第七节乙类）");
    assert_eq!(record_1[88], 1);
    assert_eq!(record_3[88], 3);
}

/// 读者一侧（M212 的会红检查之一）：卸载记号位 0 之外有位为 1 的记录当损坏（编码照根记录 flags，D22 已定项 7；kb 空白，推的）；
/// 位 0 = 1 的记录读得回来、卸载记号为真。两种都重封头校验和，免得是校验和先拒的。
#[test]
fn journal_record_unmount_marker_bit_zero_round_trips_and_other_bits_are_rejected() {
    let BuiltPool { output, .. } = built_pool();
    let reseal = |bytes: &mut Vec<u8>| {
        let digest = wide_checksum_with_field_zeroed(bytes, usize_from_u64(JOURNAL_RECORD_BYTES), JOURNAL_HEADER_CHECKSUM_OFFSET);
        bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
    };
    assert_eq!(&output.record_bytes[459..463], &[0, 0, 0, 0], "写出的卸载记号是 0");
    let mut marked = output.record_bytes.clone();
    marked[459] = 0x01;
    reseal(&mut marked);
    assert!(JournalRecord::parse(&marked).expect("位 0 = 1 的记录读得回来").new_unmount_marker);
    let mut stray_bit = output.record_bytes.clone();
    stray_bit[459] = 0x02;
    reseal(&mut stray_bit);
    assert!(JournalRecord::parse(&stray_bit).is_none(), "位 1 非 0：当损坏");
    let mut high_byte = output.record_bytes.clone();
    high_byte[462] = 0x80;
    reseal(&mut high_byte);
    assert!(JournalRecord::parse(&high_byte).is_none(), "卸载记号高字节非 0：当损坏");
    let mut untouched = output.record_bytes.clone();
    reseal(&mut untouched);
    assert!(JournalRecord::parse(&untouched).is_some(), "重封校验和本身不弄坏记录");
}
