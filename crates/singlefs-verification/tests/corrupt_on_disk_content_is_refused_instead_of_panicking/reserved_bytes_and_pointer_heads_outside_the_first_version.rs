//! 第一版之外的单元与指针字段：加密保留字节不为零的索引节点、打包单元、数据单元与账务根，带加密或压缩的指针头，
//! 声明条目或记录宽度为零的单元。

use super::*;

// ─── 代码审阅第 29 条：单元头那 29 字节预留位、指针头部的加密与压缩那几段 ───

/// 29 字节预留位（nonce 12 + MAC 16 + 算法类型 1）里挑的那一字节：MAC 的第一个字节（预留位起点 + 12）。
const MAC_BYTE_IN_THE_RESERVED_BYTES: usize = 12;

fn sample_index_node() -> Vec<u8> {
    build_index_node(
        TreeIdentifier(singlefs_format::TREE_IDENTIFIER_ACCOUNTING),
        0,
        22,
        &[1u8; 22],
        &[3u8; 22],
        CheckpointTxg(3),
        &FILESYSTEM_IDENTIFIER,
        InstanceGeneration(1),
        BirthSequence(0),
        34,
        &[vec![1u8; 34], vec![2u8; 34], vec![3u8; 34]],
    )
}

/// 场景：一个码 2 节点（key 宽 22）预留位里 MAC 的第一个字节写成 1，载荷 CRC 与头校验和重算（预留位在载荷 CRC 覆盖内、不在头校验和覆盖内）。
/// 预期：`parse_index_node` 报预留位非 0（I-2.4「恒 0、读者遇到非 0 一律判该结构损坏」）。读者若跳过这 29 字节，就照常解开。
#[test]
fn an_index_node_with_a_non_zero_encryption_reserved_byte_is_refused() {
    let mut node = sample_index_node();
    assert!(parse_index_node(&node).is_ok(), "改之前这个节点解得开");
    node[86 + 2 * 22 + MAC_BYTE_IN_THE_RESERVED_BYTES] = 1;
    reseal_index_node(&mut node);
    assert_eq!(
        parse_index_node(&node),
        Err(UnitError::EncryptionReservedBytesNotZero)
    );
}

/// 场景：一个码 3 inode 叶容器（记录宽 140、一条记录）预留位里 MAC 的第一个字节写成 1，两道校验和重算。
/// 预期：`parse_packed_unit` 报预留位非 0；读者不跳过这 29 字节。
#[test]
fn a_packed_unit_with_a_non_zero_encryption_reserved_byte_is_refused() {
    let mut unit = build_packed_unit(
        PackedIdentity {
            birth_tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_INODE),
            record_type: PACKED_TYPE_INODE,
            container: 1,
            container_birth: CheckpointTxg(3),
        },
        140,
        &[vec![7u8; 140]],
        CheckpointTxg(3),
        &FILESYSTEM_IDENTIFIER,
        WriteOrder {
            instance: InstanceGeneration(1),
            transaction: 1,
        },
        BirthSequence(0),
    );
    assert!(parse_packed_unit(&unit).is_ok(), "改之前这个单元解得开");
    unit[107 + MAC_BYTE_IN_THE_RESERVED_BYTES] = 1;
    reseal_packed_unit(&mut unit);
    assert_eq!(
        parse_packed_unit(&unit),
        Err(UnitError::EncryptionReservedBytesNotZero)
    );
}

/// 场景：一个码 1 数据单元（3000 字节载荷）预留位里 MAC 的第一个字节写成 1，两道校验和重算。
/// 预期：`parse_data_unit` 报预留位非 0；读者不跳过这 29 字节。
#[test]
fn a_data_unit_with_a_non_zero_encryption_reserved_byte_is_refused() {
    let mut unit = build_data_unit(
        DataUnitIdentity {
            tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_EXTENT),
            object: 1,
            object_birth: CheckpointTxg(3),
            anchor_offset: 0,
        },
        CheckpointTxg(3),
        &FILESYSTEM_IDENTIFIER,
        WriteOrder {
            instance: InstanceGeneration(1),
            transaction: 1,
        },
        &[5u8; 3000],
    );
    assert!(parse_data_unit(&unit).is_ok(), "改之前这个单元解得开");
    unit[105 + MAC_BYTE_IN_THE_RESERVED_BYTES] = 1;
    reseal_data_unit(&mut unit);
    assert_eq!(
        parse_data_unit(&unit),
        Err(UnitError::EncryptionReservedBytesNotZero)
    );
}

/// 场景：第一个文件那一版的记账树根预留位里 MAC 的第一个字节写成 1，节点与指着它的每一道校验和重算。
/// 预期：可写挂载在任何写之前报这个节点解不开（`UnitMalformed`「树根节点」）。读者若跳过这 29 字节，就照常挂上。
#[test]
fn an_accounting_root_with_a_non_zero_encryption_reserved_byte_is_refused_by_the_writable_mount() {
    let (mut devices, _first) = pool_after_the_first_file();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        TREE_KIND_ACCOUNTING,
        |node| {
            let reserved_start = 86 + 2 * usize::from(node[51]);
            node[reserved_start + MAC_BYTE_IN_THE_RESERVED_BYTES] = 1;
        },
    );
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::UnitMalformed {
                what: "树根节点"
            })
        ),
        "预留位非 0 的记账树根：{refusal:?}"
    );
}

fn sample_node_pointer() -> NodePointer {
    NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_ACCOUNTING),
            birth_txg: CheckpointTxg(3),
        },
        locations: [
            LocationEntry {
                device: DeviceIdentity(0),
                slot: SlotNumber(UNIT_AREA_START_SLOT + 8),
                unit_checksum: 0x1234,
            },
            LocationEntry {
                device: DeviceIdentity(1),
                slot: SlotNumber(UNIT_AREA_START_SLOT + 8),
                unit_checksum: 0x1234,
            },
        ],
        instance: InstanceGeneration(1),
        birth_sequence: BirthSequence(2),
    }
}

fn node_pointer_bytes(pointer: &NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(86);
    pointer.write_to(&mut writer);
    writer.into_bytes()
}

/// 场景：一条节点指针头部（MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2 依次在偏移 0、16、28、29、30）逐段改一个非 0 值。
/// 预期：判的读法逐段报它是哪一段（D19（块指针的结构与宽度预算） 已定项 3 / 已定项 7，D9（加密） 已定项 10 / 已定项 14）；
/// 五段都是 0 时照常读出同一条指针；读者不跳过这 36 字节拿这条指针解引用。
#[test]
fn a_node_pointer_head_carrying_encryption_or_compression_is_refused_field_by_field() {
    let pointer = sample_node_pointer();
    let clean = node_pointer_bytes(&pointer);
    assert_eq!(
        NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
            &mut ByteReader::at(&clean, 0)
        ),
        Ok(pointer),
        "第一版的取值照常读出"
    );
    let cases: [(usize, u8, PointerHeadFieldOutsideTheFirstVersion); 5] = [
        (
            0,
            1,
            PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero,
        ),
        (
            16,
            1,
            PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero,
        ),
        (
            28,
            2,
            PointerHeadFieldOutsideTheFirstVersion::AlgorithmTypeNotUnencrypted {
                algorithm_type: 2,
            },
        ),
        (
            29,
            1,
            PointerHeadFieldOutsideTheFirstVersion::CompressionAlgorithmCodeNotRecognized {
                compression_algorithm_code: 1,
            },
        ),
        (
            30,
            5,
            PointerHeadFieldOutsideTheFirstVersion::CompressedLengthNotZeroWithoutCompression {
                compressed_length: 5,
            },
        ),
    ];
    for (offset, value, expected) in cases {
        let mut damaged = clean.clone();
        damaged[offset] = value;
        assert_eq!(
            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                &mut ByteReader::at(&damaged, 0)
            ),
            Err(expected),
            "指针头部偏移 {offset} 写成 {value}"
        );
    }
}

/// 场景：一条数据指针（88 字节）头部的算法类型写成 1。预期：判的读法报算法类型不是「未加密」。
#[test]
fn a_data_pointer_head_carrying_an_encryption_algorithm_is_refused() {
    let node_pointer = sample_node_pointer();
    let data_pointer = DataPointer {
        head: node_pointer.head,
        locations: node_pointer.locations,
        write_order: WriteOrder {
            instance: InstanceGeneration(1),
            transaction: 1,
        },
    };
    let mut writer = ByteWriter::new(88);
    data_pointer.write_to(&mut writer);
    let mut bytes = writer.into_bytes();
    assert_eq!(
        DataPointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
            &mut ByteReader::at(&bytes, 0)
        ),
        Ok(data_pointer)
    );
    bytes[28] = 1;
    assert_eq!(
        DataPointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
            &mut ByteReader::at(&bytes, 0)
        ),
        Err(
            PointerHeadFieldOutsideTheFirstVersion::AlgorithmTypeNotUnencrypted {
                algorithm_type: 1
            }
        )
    );
}

/// 场景：多层记账树的一条内部条目（key 22 + 子指针 86）里子指针头部的 nonce 第一个字节写成 1。
/// 预期：`parse_internal_entry` 报子指针头部不是第一版的取值（读多层码 2 树时报 `UnitMalformed`），不照常解出子指针。
#[test]
fn a_code_two_internal_entry_whose_child_pointer_head_is_not_the_first_version_is_refused() {
    let key = [9u8; 22];
    let mut entry = build_internal_entry(&key, sample_node_pointer());
    assert_eq!(
        parse_internal_entry(&entry, 22),
        Ok((key.to_vec(), sample_node_pointer()))
    );
    entry[22 + 16] = 1;
    assert_eq!(
        parse_internal_entry(&entry, 22),
        Err(
            InternalEntryRefusal::ChildPointerHeadOutsideTheFirstVersion(
                PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero
            )
        )
    );
}

/// 场景：一条 journal 记录新根段里树表指针头部的压缩算法码写成 1（新根段从 99 起，指针头部偏移 29），整条 `header_csum` 重算。
/// 预期：`JournalRecord::parse` 当这条记录损坏（交回 `None`，与校验和不过同一个结局），不照常解出、重放时不拿这条指针当新根的树表。
#[test]
fn a_journal_record_whose_new_tree_table_pointer_carries_compression_is_not_parsed() {
    let record = JournalRecord {
        instance: InstanceGeneration(1),
        counter: 3,
        checkpoint_txg: CheckpointTxg(3),
        transaction: 1,
        is_commit: true,
        ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,
        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
        back_chain: 0,
        filesystem_identifier: 7,
        new_tree_table: sample_node_pointer(),
        new_mapping_root: sample_node_pointer(),
        new_tree_identifier_watermark:
            singlefs_format::TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
        new_rollback_floor: CheckpointTxg(0),
        new_instance_table: NodePointer::empty_root(),
        new_allocation_record_tree_root: NodePointer::empty_root(),
        new_unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
        named: Vec::new(),
        new_reclaim_threshold: singlefs_core::address::CheckpointTxg(0),
        new_pinned_ring_slot: singlefs_core::root_record::PinnedRingSlot::NO_SLOT_PINNED,
    };
    let mut bytes = record.to_bytes();
    assert_eq!(
        JournalRecord::parse(&bytes, 7),
        Some(record),
        "改之前解得开"
    );
    bytes[99 + 29] = 1;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let digest =
        wide_checksum_with_field_zeroed(&bytes, record_bytes, JOURNAL_HEADER_CHECKSUM_OFFSET);
    bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
        .copy_from_slice(&digest);
    assert_eq!(JournalRecord::parse(&bytes, 7), None);
}

// ─── 代码审阅第 38 条：条目宽 0 而条目数非 0 ───

/// 场景：一个 key 宽 0、没有条目的码 2 节点，头里的条目数改成 3、条目宽改成 0（声明长度 0 = 3 × 0 照样对得上），头校验和重算。
/// 预期：`parse_index_node` 报结构错「条目宽为 0 而条目数不为 0」，不切出 0 条条目、头里却说 3 条还照常交出去。
#[test]
fn an_index_node_declaring_entries_of_width_zero_is_refused() {
    let mut node = build_index_node(
        TreeIdentifier(singlefs_format::TREE_IDENTIFIER_ACCOUNTING),
        0,
        0,
        &[],
        &[],
        CheckpointTxg(3),
        &FILESYSTEM_IDENTIFIER,
        InstanceGeneration(1),
        BirthSequence(0),
        1,
        &[],
    );
    // key 宽 0 时条目数住 82、条目宽住 84。
    node[82..84].copy_from_slice(&3u16.to_le_bytes());
    node[84..86].copy_from_slice(&0u16.to_le_bytes());
    reseal_index_node(&mut node);
    assert_eq!(
        parse_index_node(&node),
        Err(UnitError::Structure(
            UnitStructureDefect::IndexNodeEntryWidthZeroWithEntries
        ))
    );
}

/// 场景：一个没有记录的码 3 单元，头里的记录数改成 3、记录宽改成 0（住 69、71），头校验和重算。
/// 预期：`parse_packed_unit` 报结构错「记录宽为 0 而记录数不为 0」，不切出 0 条记录照常交出去。
#[test]
fn a_packed_unit_declaring_records_of_width_zero_is_refused() {
    let mut unit = build_packed_unit(
        PackedIdentity {
            birth_tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_INODE),
            record_type: PACKED_TYPE_INODE,
            container: 1,
            container_birth: CheckpointTxg(3),
        },
        140,
        &[],
        CheckpointTxg(3),
        &FILESYSTEM_IDENTIFIER,
        WriteOrder {
            instance: InstanceGeneration(1),
            transaction: 1,
        },
        BirthSequence(0),
    );
    unit[69..71].copy_from_slice(&3u16.to_le_bytes());
    unit[71..73].copy_from_slice(&0u16.to_le_bytes());
    reseal_packed_unit(&mut unit);
    assert_eq!(
        parse_packed_unit(&unit),
        Err(UnitError::Structure(
            UnitStructureDefect::PackedUnitRecordWidthZeroWithRecords
        ))
    );
}
