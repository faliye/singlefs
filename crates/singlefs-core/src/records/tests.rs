use super::*;
use crate::address::SlotNumber;
use singlefs_format::{
    TREE_IDENTIFIER_ACCOUNTING, TREE_IDENTIFIER_ALLOCATION_RECORDS,
    TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST, TREE_IDENTIFIER_DELETED_INODES,
    TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LINEAGE,
    TREE_IDENTIFIER_LIVELIST, TREE_IDENTIFIER_LOGGED_OPERATIONS, TREE_IDENTIFIER_RETIREMENT,
    TREE_IDENTIFIER_SPARSE_SIDE_TABLE, TREE_IDENTIFIER_TOMBSTONE,
    TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, TREE_IDENTIFIER_WATERMARK_AT_MKFS,
};

/// 树的种类码与树 ID 的值域不相交（D8（核心索引结构） 已定项 9 射程、已定项 11）：
/// 核心层的每个种类码都小于 65536（树的种类那 2 字节取不到的最小值），每个树 ID 与两个水位都不小于它。
#[test]
fn every_tree_kind_code_is_below_every_tree_identifier() {
    let first_value_the_kind_field_cannot_hold = u64::from(u16::MAX) + 1;
    for kind in [
        TREE_KIND_EXTENT,
        TREE_KIND_INODE,
        TREE_KIND_ALLOCATION,
        TREE_KIND_ACCOUNTING,
        TREE_KIND_LIVELIST,
        TREE_KIND_SPARSE_SIDE_TABLE,
        TREE_KIND_DEADLIST,
        TREE_KIND_DELETED_INODES,
        TREE_KIND_TOMBSTONE,
        TREE_KIND_LINEAGE,
        TREE_KIND_LOGGED_OPERATIONS,
        TREE_KIND_RETIREMENT,
    ] {
        assert!(
            u64::from(kind) < first_value_the_kind_field_cannot_hold,
            "种类码 {kind} 小于 65536"
        );
    }
    for identifier in [
        TREE_IDENTIFIER_EXTENT,
        TREE_IDENTIFIER_INODE,
        TREE_IDENTIFIER_ALLOCATION_RECORDS,
        TREE_IDENTIFIER_ACCOUNTING,
        TREE_IDENTIFIER_CENTRAL_MAPPING,
        TREE_IDENTIFIER_LIVELIST,
        TREE_IDENTIFIER_SPARSE_SIDE_TABLE,
        TREE_IDENTIFIER_DEADLIST,
        TREE_IDENTIFIER_DELETED_INODES,
        TREE_IDENTIFIER_TOMBSTONE,
        TREE_IDENTIFIER_LINEAGE,
        TREE_IDENTIFIER_LOGGED_OPERATIONS,
        TREE_IDENTIFIER_RETIREMENT,
        TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
    ] {
        assert!(
            identifier >= first_value_the_kind_field_cannot_hold,
            "树 ID（或水位）{identifier} 不小于 65536：与种类码的值域不相交"
        );
    }
}

#[test]
fn widths_of_every_record_match_the_byte_table() {
    let record = InodeRecord {
        inode: 1,
        object_birth: CheckpointTxg(3),
        nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
        size: 3000,
        // 新池新建文件那条记录的取值：改动计数 = 这次发布的 checkpoint_txg 3（增补 2 第 11 行）。
        change_count: 3,
        write_time_seconds: 1_788_000_000,
        truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
    };
    let bytes = record.to_bytes();
    assert_eq!(bytes.len(), 140);
    assert_eq!(
        u32::from_le_bytes(bytes[24..28].try_into().expect("4")),
        0o100_644,
        "mode 在偏移 24"
    );
    assert_eq!(
        u64::from_le_bytes(bytes[40..48].try_into().expect("8")),
        3000,
        "size 在偏移 40"
    );
    assert_eq!(
        u64::from_le_bytes(bytes[48..56].try_into().expect("8")),
        6,
        "blocks 在偏移 48：3000 字节 ⇒ ⌈3000 ÷ 512⌉ = 6（D8（核心索引结构） 已定项 6）"
    );
    assert_eq!(InodeRecord::parse(&bytes), Some(record));
    let mut tampered = bytes.clone();
    tampered[139] = 1;
    assert_eq!(InodeRecord::parse(&tampered), None, "预留段非零拒收");
    let pointer = NodePointer::empty_root();
    let identity = PackedIdentity {
        birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
        record_type: 2,
        container: 1,
        container_birth: CheckpointTxg(3),
    };
    assert_eq!(build_inode_internal_entry(1, identity, pointer).len(), 120);
    let data_pointer = DataPointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            birth_txg: CheckpointTxg(3),
        },
        locations: [
            LocationEntry {
                device: DeviceIdentity(0),
                slot: SlotNumber(50180),
                unit_checksum: 1,
            },
            LocationEntry {
                device: DeviceIdentity(1),
                slot: SlotNumber(50180),
                unit_checksum: 1,
            },
        ],
        write_order: WriteOrder {
            instance: InstanceGeneration(1),
            transaction: 1,
        },
    };
    assert_eq!(build_extent_record(1, 0, data_pointer).len(), 112);
    assert_eq!(
        parse_extent_record(&build_extent_record(1, 0, data_pointer))
            .expect("满宽的 112 字节记录解得开")
            .1,
        data_pointer
    );
    assert_eq!(
        parse_inode_internal_entry(&build_inode_internal_entry(1, identity, pointer)),
        Some((1, identity, pointer))
    );
    let entry = AccountingEntry {
        statistic: 1,
        tree: TreeIdentifier(0),
        device: DeviceIdentity(0),
        generation: CheckpointTxg(3),
        value: 212_992,
        sequence: 1,
    };
    assert_eq!(entry.to_bytes().len(), 34);
    assert_eq!(AccountingEntry::parse(&entry.to_bytes()), Some(entry));
    let key = mapping_key_for_data(data_pointer.head, data_pointer.write_order);
    assert_eq!(key.len(), 27);
    assert_eq!(build_mapping_entry(&key, data_pointer.locations).len(), 55);
    assert_eq!(
        parse_mapping_entry(&build_mapping_entry(&key, data_pointer.locations)),
        Some((key.clone(), data_pointer.locations))
    );
    assert_eq!(
        &key[17..27],
        &data_key_tail(data_pointer.write_order),
        "映射 key 的尾段就是点名项的尾段"
    );
    let node_key = mapping_key_for_node(2, pointer);
    assert_eq!(&node_key[25..], &[0, 0], "码 2 的 key 末尾补零到 27");
    let table_entry = TreeTableEntry {
        kind: TREE_KIND_EXTENT,
        tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
        root: pointer,
        previous_snapshot_txg: CheckpointTxg(0),
        birth_txg: CheckpointTxg(3),
        head_identifier: TREE_IDENTIFIER_INODE,
    };
    assert_eq!(table_entry.to_bytes().len(), 200);
    assert_eq!(
        TreeTableEntry::parse(&table_entry.to_bytes()),
        Some(table_entry)
    );
}

/// D5（快照 / 空间记账机制） 已定项 9：`previous_snapshot_txg` 写在条目偏移 100 那 8 字节（树 ID 8 + 条目长度 2 + 种类 2 + flags 2 +
/// 根指针 86 之后、诞生 txg 之前），`to_bytes` 照结构里的值写、`parse` 照盘上读回来；两个 txg 字段各自独立，换一个不动另一个。
#[test]
fn previous_snapshot_txg_is_written_at_offset_one_hundred_and_read_back_apart_from_the_birth_txg() {
    let entry = TreeTableEntry {
        kind: TREE_KIND_INODE,
        tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
        root: NodePointer::empty_root(),
        previous_snapshot_txg: CheckpointTxg(0x0102_0304),
        birth_txg: CheckpointTxg(0x0506),
        head_identifier: TREE_IDENTIFIER_INODE,
    };
    let bytes = entry.to_bytes();
    let field_at =
        |offset: usize| u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"));
    assert_eq!(
        field_at(100),
        0x0102_0304,
        "偏移 100：previous_snapshot_txg"
    );
    assert_eq!(field_at(108), 0x0506, "偏移 108：诞生 txg");
    assert_eq!(
        TreeTableEntry::parse(&bytes).map(|parsed| parsed.previous_snapshot_txg),
        Some(CheckpointTxg(0x0102_0304)),
        "读回来是写进去的值，不是 0"
    );
    assert_eq!(TreeTableEntry::parse(&bytes), Some(entry), "逐字段读回相等");
}

/// 删快照的意图只占码 14 条目 flags 位 0：写进偏移 12 那 2 字节、
/// 读回来记号相同、别的字段不动；码 14 上别的位、别的种类上的位 0 读者拒收。
#[test]
fn the_destruction_mark_sits_in_flags_bit_zero_of_a_snapshot_inode_root_entry_only() {
    let snapshot_inode_root = TreeTableEntry {
        kind: TREE_KIND_SNAPSHOT_INODE_ROOT,
        tree: TreeIdentifier(65549),
        root: NodePointer::empty_root(),
        previous_snapshot_txg: CheckpointTxg(4),
        birth_txg: CheckpointTxg(9),
        head_identifier: TREE_IDENTIFIER_INODE,
    };
    let marked =
        snapshot_inode_root.to_bytes_with_destruction_mark(SnapshotDestructionMark::BeingDestroyed);
    assert_eq!(
        u16::from_le_bytes([marked[12], marked[13]]),
        TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED,
        "偏移 12：flags 位 0"
    );
    assert_eq!(
        TreeTableEntry::parse_with_destruction_mark(&marked),
        Some((snapshot_inode_root, SnapshotDestructionMark::BeingDestroyed)),
        "读回来带「销毁中」，别的字段与写进去的相同"
    );
    assert_eq!(
        TreeTableEntry::parse_with_destruction_mark(&snapshot_inode_root.to_bytes()),
        Some((
            snapshot_inode_root,
            SnapshotDestructionMark::NotBeingDestroyed
        )),
        "不带记号的那一份读回来不带"
    );
    let mut another_bit = marked.clone();
    another_bit[12] = 2;
    assert_eq!(
        TreeTableEntry::parse(&another_bit),
        None,
        "码 14 上 flags 别的位：拒收"
    );
    let snapshot_extent_root = TreeTableEntry {
        kind: TREE_KIND_SNAPSHOT_EXTENT_ROOT,
        tree: TreeIdentifier(65550),
        root: NodePointer::empty_root(),
        previous_snapshot_txg: CheckpointTxg(0),
        birth_txg: CheckpointTxg(9),
        head_identifier: 65549,
    };
    let mut bit_zero_on_another_kind = snapshot_extent_root.to_bytes();
    bit_zero_on_another_kind[12] = 1;
    assert_eq!(
        TreeTableEntry::parse(&bit_zero_on_another_kind),
        None,
        "码 15 上 flags 位 0：拒收"
    );
}

/// deadlist 条目 43 字节：头的树 ID 8 + 纪元 8 + 中央映射 key 27（D5（快照 / 空间记账机制） 已定项 12），读回来逐字段相等；
/// 出生 txg 取映射 key 偏移 9 那 8 字节，不取纪元。
#[test]
fn deadlist_entry_is_the_head_the_epoch_and_the_mapping_key_and_reads_back() {
    let mapping_key = mapping_key_for_node(
        2,
        NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
                birth_txg: CheckpointTxg(7),
            },
            locations: NodePointer::empty_root().locations,
            instance: InstanceGeneration(1),
            birth_sequence: BirthSequence(3),
        },
    );
    let entry = DeadlistEntry {
        head: TreeIdentifier(TREE_IDENTIFIER_INODE),
        epoch: CheckpointTxg(11),
        mapping_key: mapping_key.clone(),
    };
    let bytes = entry.key_bytes();
    assert_eq!(bytes.len(), DEADLIST_ENTRY_KEY_BYTES, "43 字节");
    assert_eq!(
        &bytes[..8],
        &TREE_IDENTIFIER_INODE.to_le_bytes(),
        "偏移 0：头的树 ID"
    );
    assert_eq!(&bytes[8..16], &11u64.to_le_bytes(), "偏移 8：纪元");
    assert_eq!(
        &bytes[16..],
        mapping_key.as_slice(),
        "偏移 16：中央映射 key"
    );
    assert_eq!(
        DeadlistEntry::parse(&bytes),
        Some(entry.clone()),
        "读回来逐字段相等"
    );
    assert_eq!(
        entry.birth_txg(),
        CheckpointTxg(7),
        "出生 txg 取映射 key 那一段"
    );
    assert_eq!(DeadlistEntry::parse(&bytes[..42]), None, "窄于 43：解不开");
}

/// 墓碑记录五个字段落在 `.claude/kb/layout/03-third-txn.md`「二、墓碑记录」那张表的偏移上，读回来逐字段相等。
#[test]
fn tombstone_record_fields_sit_at_the_offsets_of_the_layout_table_and_read_back() {
    let record =
        TombstoneRecord::of_the_whole_object(InodeNumber(7), CheckpointTxg(5), CheckpointTxg(9));
    let bytes = record.to_bytes();
    assert_eq!(bytes.len(), 40, "墓碑记录 40 字节（D18 已定项 24 第 2 条）");
    let field_at =
        |offset: usize| u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"));
    assert_eq!(field_at(0), 7, "偏移 0：对象 ID = 被删的 inode 号");
    assert_eq!(field_at(8), 5, "偏移 8：对象出生代");
    assert_eq!(field_at(16), 0, "偏移 16：区间起 0（整对象）");
    assert_eq!(field_at(24), u64::MAX, "偏移 24：区间止 2⁶⁴ − 1（整对象）");
    assert_eq!(field_at(32), 9, "偏移 32：死亡 txg = 意图那次发布的 txg");
    assert_eq!(
        TombstoneRecord::parse(&bytes),
        Some(record),
        "读回来逐字段相等"
    );
    assert_eq!(
        TombstoneRecord::parse(&bytes[..39]),
        None,
        "窄于 40 字节不解"
    );
}

/// 退役记录 34 字节：出生树 0、打包记录类型 8、容器号 10、容器出生代 18、死亡 txg 26，小端、无填充；读回逐字段相等，
/// 长度不是 34 读不出（短一字节、长一字节都不收）。身份四段与 inode 内部条目里身份引用那 26 字节逐字节相同。
#[test]
fn retirement_record_is_thirty_four_bytes_identity_in_entry_order_then_the_death_txg() {
    let record = RetirementRecord {
        retired: PackedIdentity {
            birth_tree: TreeIdentifier(0x0102),
            record_type: 2,
            container: 0x0304,
            container_birth: CheckpointTxg(0x0506),
        },
        death_txg: CheckpointTxg(0x0708),
    };
    let bytes = record.to_bytes();
    assert_eq!(
        bytes,
        [
            vec![0x02, 0x01, 0, 0, 0, 0, 0, 0],
            vec![2, 0],
            vec![0x04, 0x03, 0, 0, 0, 0, 0, 0],
            vec![0x06, 0x05, 0, 0, 0, 0, 0, 0],
            vec![0x08, 0x07, 0, 0, 0, 0, 0, 0],
        ]
        .concat(),
        "出生树 0、类型 8、容器号 10、出生代 18、死亡 txg 26，小端"
    );
    let entry = build_inode_internal_entry(9, record.retired, NodePointer::empty_root());
    assert_eq!(
        &bytes[..26],
        &entry[8..34],
        "身份四段与 inode 内部条目里的身份引用同序同宽"
    );
    assert_eq!(RetirementRecord::parse(&bytes), Some(record));
    assert_eq!(RetirementRecord::parse(&bytes[..33]), None);
    assert_eq!(
        RetirementRecord::parse(&[bytes.as_slice(), &[0]].concat()),
        None
    );
}

fn record_with_the_truncation_field(truncation_field: InodeRecordTruncationField) -> InodeRecord {
    InodeRecord {
        inode: 1,
        object_birth: CheckpointTxg(3),
        nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
        size: 100,
        change_count: 9,
        write_time_seconds: 1_788_000_000,
        truncation_field,
    }
}

/// T 写在偏移 120 起 8 字节、N_w 写在 128 起 8 字节，都是小端，读者从同一处读回来；两段各取一个每个字节都不同的值，
/// 写错位置、读错位置、读成 0、两段对调都对不上。之前 108–119、之后 136–139 两段仍是 0。
#[test]
fn truncation_field_is_written_little_endian_at_offsets_120_and_128_and_read_back() {
    let record = record_with_the_truncation_field(
        InodeRecordTruncationField::written_by_the_truncation_intent_of(
            CheckpointTxg(0x0807_0605_0403_0201),
            0x1817_1615_1413_1211,
        ),
    );
    let bytes = record.to_bytes();
    assert_eq!(bytes.len(), 140);
    assert_eq!(
        bytes[120..128],
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        "T 在偏移 120 起 8 字节、小端"
    );
    assert_eq!(
        bytes[128..136],
        [0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18],
        "N_w 在偏移 128 起 8 字节、小端"
    );
    assert!(
        bytes[108..120]
            .iter()
            .chain(&bytes[136..])
            .all(|byte| *byte == 0),
        "108–119 与 136–139 仍恒 0"
    );
    assert_eq!(InodeRecord::parse(&bytes), Some(record), "读回来逐字段相同");
    let not_written = record_with_the_truncation_field(InodeRecordTruncationField::NOT_WRITTEN);
    assert!(
        not_written.to_bytes()[108..].iter().all(|byte| *byte == 0),
        "没写过的字段：偏移 108 起整段都是 0"
    );
}

/// 合法性只判「T = 0 ⇒ N_w = 0」。T = 0 而 N_w ≠ 0 拒；截到 0 是 T ≠ 0、N_w = 0，收。
#[test]
fn parse_refuses_a_truncation_field_whose_txg_is_zero_while_its_size_is_not() {
    let mut size_without_txg =
        record_with_the_truncation_field(InodeRecordTruncationField::NOT_WRITTEN).to_bytes();
    size_without_txg[128] = 5;
    assert_eq!(
        InodeRecord::parse(&size_without_txg),
        None,
        "T = 0、N_w = 5：不合法"
    );
    let truncated_to_zero = record_with_the_truncation_field(
        InodeRecordTruncationField::written_by_the_truncation_intent_of(CheckpointTxg(7), 0),
    );
    assert_eq!(
        InodeRecord::parse(&truncated_to_zero.to_bytes()).map(|parsed| parsed.truncation_field),
        Some(InodeRecordTruncationField::written_by_the_truncation_intent_of(CheckpointTxg(7), 0)),
        "截到 0：T = 7、N_w = 0 合法，读回来 T 仍是 7"
    );
    assert_eq!(
        InodeRecordTruncationField::from_the_two_fields_on_disk(0, 5),
        None
    );
}

/// 截断字段两边那两段恒零区（108–119、136–139）任一字节非零都拒，截断字段那 16 字节里的非零（T 非零）收。
#[test]
fn parse_refuses_a_nonzero_byte_in_either_always_zero_area_around_the_truncation_field() {
    let bytes =
        record_with_the_truncation_field(InodeRecordTruncationField::NOT_WRITTEN).to_bytes();
    for offset in (108..120).chain(136..140) {
        let mut tampered = bytes.clone();
        tampered[offset] = 1;
        assert_eq!(
            InodeRecord::parse(&tampered),
            None,
            "偏移 {offset} 在恒零区里，非零拒收"
        );
    }
    for offset in 120..128 {
        let mut txg_byte_set = bytes.clone();
        txg_byte_set[offset] = 1;
        assert!(
            InodeRecord::parse(&txg_byte_set).is_some(),
            "偏移 {offset} 是 T 的一个字节：T 非零、N_w = 0 合法"
        );
    }
}

/// D8（核心索引结构） 已定项 16 第 4 条：nlink 由 `to_bytes` 照结构里的值写进偏移 36、由 `parse` 读回来——
/// 删除中的记录（nlink = 0，size 不变）写出去读回来还是 0，读者与写入口才认得出它；活着的写 1。
#[test]
fn nlink_is_written_from_the_record_and_read_back_at_offset_thirty_six() {
    let record_of_nlink = |nlink: u32| InodeRecord {
        inode: 7,
        object_birth: CheckpointTxg(5),
        nlink,
        size: 40_000,
        change_count: 6,
        write_time_seconds: 1_788_000_000,
        truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
    };
    for nlink in [
        NLINK_OF_AN_INODE_BEING_DELETED,
        NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
    ] {
        let bytes = record_of_nlink(nlink).to_bytes();
        assert_eq!(
            u32::from_le_bytes(bytes[36..40].try_into().expect("偏移 36 起 4 字节")),
            nlink,
            "nlink 写在偏移 36，写的是结构里的值 {nlink}"
        );
        assert_eq!(
            InodeRecord::parse(&bytes).map(|parsed| parsed.nlink),
            Some(nlink),
            "nlink {nlink} 读回来不变"
        );
        assert_eq!(
            InodeRecord::parse(&bytes).map(|parsed| parsed.size),
            Some(40_000),
            "size 不随 nlink 变（意图那次 size 不变）"
        );
    }
}

/// C480（inode 记录的 blocks 怎么算全仓没有条款）（D8（核心索引结构） 已定项 6）：`blocks` = ⌈size ÷ 512⌉，
/// 逻辑长度的块数，不表示分到的空间。期望值按条款手算，不从被测代码反推。
/// 每个取样点旁边写着另外两种读法会给的数：「按分到几个 32 KiB 数据单元 × 64 填」（净荷 32634 一个单元）
/// 与「向下取整」——除了 0 之外，每个点至少把其中一种分开。
#[test]
fn blocks_is_the_logical_length_in_512_byte_blocks_rounded_up_not_the_allocated_units() {
    let blocks_written_for_size = |size: u64| {
        let bytes = InodeRecord {
            inode: 1,
            object_birth: CheckpointTxg(3),
            nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
            size,
            change_count: 3,
            write_time_seconds: 1_788_000_000,
            truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
        }
        .to_bytes();
        u64::from_le_bytes(bytes[48..56].try_into().expect("偏移 48 起 8 字节"))
    };
    // (size, 条款给的 blocks, 按分到的单元填, 向下取整)
    let expected: [(u64, u64, u64, u64); 7] = [
        (0, 0, 0, 0),
        (1, 1, 64, 0),
        (512, 1, 64, 1),
        (513, 2, 64, 1),
        (3000, 6, 64, 5),
        (32_635, 64, 128, 63),
        (40_000, 79, 128, 78),
    ];
    for (size, blocks, by_allocated_units, rounded_down) in expected {
        assert_eq!(
                blocks_written_for_size(size),
                blocks,
                "size {size} 字节 ⇒ blocks = ⌈{size} ÷ 512⌉ = {blocks}（按分到的单元填是 {by_allocated_units}，向下取整是 {rounded_down}）"
            );
    }
}

/// 条目宽是索引节点头里的一个**盘上字段**（`parse_index_node` 只判了它 ≥ key 宽）：
/// 窄到刚好等于 key 宽的条目，四个读者都要交回 `None`，不许按字段表的固定偏移切下去。
/// key 宽：extent 24、inode 8、记账 22、映射 27（panic 面普查 R1 / R3 / R4 / R2 的坏法就是把条目宽缩到这四个数）。
#[test]
fn entry_readers_refuse_an_entry_narrower_than_its_field_table() {
    let entry_narrowed_to_the_key_width = |key_width: usize| vec![0u8; key_width];
    assert_eq!(
        parse_extent_record(&entry_narrowed_to_the_key_width(24)),
        Err(PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable),
        "extent 叶记录该有 112 字节，条目宽缩到 key 宽 24 时不解"
    );
    assert_eq!(
        parse_extent_record(&[0u8; 111]),
        Err(PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable),
        "差一个字节也不解"
    );
    assert!(
        parse_extent_record(&[0u8; 112]).is_ok(),
        "刚好 112 要解得开"
    );
    assert_eq!(
        parse_inode_internal_entry(&entry_narrowed_to_the_key_width(8)),
        None,
        "inode 内部条目该有 120 字节，条目宽缩到 key 宽 8 时不解"
    );
    assert!(
        parse_inode_internal_entry(&[0u8; 120]).is_some(),
        "刚好 120 要解得开"
    );
    assert_eq!(
        AccountingEntry::parse(&entry_narrowed_to_the_key_width(22)),
        None,
        "记账条目该有 34 字节，条目宽缩到 key 宽 22 时不解"
    );
    assert!(
        AccountingEntry::parse(&[0u8; 34]).is_some(),
        "刚好 34 要解得开"
    );
    assert_eq!(
        parse_mapping_entry(&entry_narrowed_to_the_key_width(27)),
        None,
        "映射条目该有 55 字节，条目宽缩到 key 宽 27 时不解（普查 R2）"
    );
    assert_eq!(parse_mapping_entry(&[0u8; 54]), None, "差一个字节也不解");
    assert!(
        parse_mapping_entry(&[0u8; 55]).is_some(),
        "刚好 55 要解得开"
    );
}

#[test]
fn mapping_keys_order_by_field_not_by_bytes() {
    let pointer_with_sequence = |sequence: u32| NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
            birth_txg: CheckpointTxg(3),
        },
        locations: NodePointer::empty_root().locations,
        instance: InstanceGeneration(1),
        birth_sequence: BirthSequence(sequence),
    };
    let earlier = mapping_key_for_node(2, pointer_with_sequence(1));
    let later = mapping_key_for_node(2, pointer_with_sequence(256));
    assert!(
        mapping_key_sort_key(&earlier) < mapping_key_sort_key(&later),
        "1 < 256 按整数比，小端字节序会比反"
    );
    assert!(earlier > later, "memcmp 序确实是反的，所以不能用它");
}
