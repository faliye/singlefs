use super::*;

use singlefs_core::address::{DeviceIdentity, InstanceGeneration, TreeIdentifier};
use singlefs_core::allocator::AllocationRecordQuarantineMark;
use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
use singlefs_core::records::{mapping_key_for_data, mapping_key_for_node};
use singlefs_core::region_scan::scan_region;
use singlefs_core::unit::{
    build_data_unit, build_index_node, build_packed_unit, DataUnitIdentity, PackedIdentity,
    WriteOrder, PACKED_TYPE_INODE,
};

const REGION_DEVICE: DeviceIdentity = DeviceIdentity(0);
const MIRROR_DEVICE: DeviceIdentity = DeviceIdentity(1);
const BIRTH_TXG: CheckpointTxg = CheckpointTxg(7);
const WRITING_INSTANCE: InstanceGeneration = InstanceGeneration(2);
const FILESYSTEM_IDENTIFIER: [u8; 16] = [7; 16];
/// 测试自己定的两棵树号（不读格式里登记的树号）。
const TREE_OF_THE_TEST_DATA: u64 = 11;
const TREE_OF_THE_TEST_NODES: u64 = 12;

fn region_under_test() -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(REGION_DEVICE, SlotNumber(64 * 900))
        .expect("64 的倍数")
}

fn slot_in_region(offset_in_slots: u64) -> SlotNumber {
    SlotNumber(region_under_test().start_slot().0 + offset_in_slots)
}

fn unreleased_record(slot: SlotNumber, span_slots: u16) -> AllocationRecord {
    AllocationRecord {
        device: REGION_DEVICE,
        slot,
        span_slots,
        generation: BIRTH_TXG,
        is_released: false,
        quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
    }
}

fn locations_at(slot: SlotNumber) -> [LocationEntry; 2] {
    [REGION_DEVICE, MIRROR_DEVICE].map(|device| LocationEntry {
        device,
        slot,
        unit_checksum: 0,
    })
}

fn central_mapping_entry(key: Vec<u8>, locations: [LocationEntry; 2]) -> CentralMappingEntry {
    CentralMappingEntry { key, locations }
}

fn data_unit_and_key(transaction: u64) -> (Vec<u8>, Vec<u8>) {
    let write_order = WriteOrder {
        instance: WRITING_INSTANCE,
        transaction,
    };
    let bytes = build_data_unit(
        DataUnitIdentity {
            tree: TreeIdentifier(TREE_OF_THE_TEST_DATA),
            object: 1,
            object_birth: CheckpointTxg(3),
            anchor_offset: 0,
        },
        BIRTH_TXG,
        &FILESYSTEM_IDENTIFIER,
        write_order,
        &[0x5A; 100],
    );
    let key = mapping_key_for_data(
        PointerHead {
            birth_tree: TreeIdentifier(TREE_OF_THE_TEST_DATA),
            birth_txg: BIRTH_TXG,
        },
        write_order,
    );
    (bytes, key)
}

/// 载荷两万字节、字节各不相同的数据单元：头一片之后的字节不全是 0（全 0 的那几字节从 0 起算出的寄存器也是 0，寄存器接没接上看不出来）。
fn data_unit_with_a_long_payload_and_key(transaction: u64) -> (Vec<u8>, Vec<u8>) {
    let write_order = WriteOrder {
        instance: WRITING_INSTANCE,
        transaction,
    };
    let payload: Vec<u8> = (0..20_000_u32)
        .map(|position| {
            u8::try_from((position + u32::try_from(transaction).expect("小数")) % 251)
                .expect("小于 251")
        })
        .collect();
    let bytes = build_data_unit(
        DataUnitIdentity {
            tree: TreeIdentifier(TREE_OF_THE_TEST_DATA),
            object: 1,
            object_birth: CheckpointTxg(3),
            anchor_offset: 0,
        },
        BIRTH_TXG,
        &FILESYSTEM_IDENTIFIER,
        write_order,
        &payload,
    );
    let key = mapping_key_for_data(
        PointerHead {
            birth_tree: TreeIdentifier(TREE_OF_THE_TEST_DATA),
            birth_txg: BIRTH_TXG,
        },
        write_order,
    );
    (bytes, key)
}

fn node_pointer(tree: u64, birth_sequence: u32) -> NodePointer {
    NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(tree),
            birth_txg: BIRTH_TXG,
        },
        locations: NodePointer::empty_root().locations,
        instance: WRITING_INSTANCE,
        birth_sequence: BirthSequence(birth_sequence),
    }
}

fn index_node_and_key(birth_sequence: u32) -> (Vec<u8>, Vec<u8>) {
    let bytes = build_index_node(
        TreeIdentifier(TREE_OF_THE_TEST_NODES),
        0,
        24,
        &[0; 24],
        &[0; 24],
        BIRTH_TXG,
        &FILESYSTEM_IDENTIFIER,
        WRITING_INSTANCE,
        BirthSequence(birth_sequence),
        112,
        &[],
    );
    let key = mapping_key_for_node(
        UNIT_CLASS_INDEX_NODE,
        node_pointer(TREE_OF_THE_TEST_NODES, birth_sequence),
    );
    (bytes, key)
}

fn packed_unit_and_key(birth_sequence: u32) -> (Vec<u8>, Vec<u8>) {
    let bytes = build_packed_unit(
        PackedIdentity {
            birth_tree: TreeIdentifier(TREE_OF_THE_TEST_NODES),
            record_type: PACKED_TYPE_INODE,
            container: 1,
            container_birth: BIRTH_TXG,
        },
        140,
        &[vec![1; 140]],
        BIRTH_TXG,
        &FILESYSTEM_IDENTIFIER,
        WriteOrder {
            instance: WRITING_INSTANCE,
            transaction: 9,
        },
        BirthSequence(birth_sequence),
    );
    let key = mapping_key_for_node(
        UNIT_CLASS_PACKED,
        node_pointer(TREE_OF_THE_TEST_NODES, birth_sequence),
    );
    (bytes, key)
}

#[test]
fn pieces_are_packed_end_to_end_with_their_location_span_read_flag_kind_and_registers() {
    let unit_bytes_of_the_first = [0xA1_u8, 0xA2, 0xA3];
    let unit_bytes_of_the_third = [0xC1_u8, 0xC2];
    let piece_after_the_head = [0xD1_u8];
    let wide_slot = SlotNumber((5_u64 << 32) | 9);
    let whole_records = [
        RegionRecordToClassify {
            entry_key: 0,
            record: unreleased_record(slot_in_region(0), 2),
            unit_bytes: Some(unit_bytes_of_the_first.as_slice()),
        },
        RegionRecordToClassify {
            entry_key: 1,
            record: unreleased_record(slot_in_region(2), 1),
            unit_bytes: None,
        },
    ];
    let packed = packed_region_record_chunk(&[
        RegionRecordPieceForTheKernels::of_a_whole_record(&whole_records[0]),
        RegionRecordPieceForTheKernels::of_a_whole_record(&whole_records[1]),
        RegionRecordPieceForTheKernels::UnitFromItsStart {
            record: unreleased_record(wide_slot, 3),
            bytes: Some(&unit_bytes_of_the_third),
            bytes_of_the_unit_after_this_piece: 700,
            registers_of_the_bytes_after_this_piece: CrcRegistersOfTheTwoKernels {
                walking_kernel: 0x1111,
                searching_kernel: 0x2222,
            },
        },
        RegionRecordPieceForTheKernels::PieceAfterTheHead {
            record: unreleased_record(slot_in_region(4), 2),
            bytes: &piece_after_the_head,
            registers_before_this_piece: CrcRegistersOfTheTwoKernels {
                walking_kernel: 0x3333,
                searching_kernel: 0x4444,
            },
        },
    ]);
    assert_eq!(
        packed.input_bytes,
        vec![0xA1, 0xA2, 0xA3, 0xC1, 0xC2, 0xD1, 0, 0],
        "读不到的那一条占 0 字节，末尾补到 8 字节"
    );
    let region_start = u32::try_from(slot_in_region(0).0).expect("小槽号");
    assert_eq!(
            packed.descriptor_words,
            vec![
                0,
                3,
                2,
                1,
                0,
                region_start,
                0,
                0,
                0,
                0,
                0, //
                3,
                0,
                1,
                0,
                0,
                region_start + 2,
                0,
                0,
                0,
                0,
                0, //
                3,
                2,
                3,
                1,
                0,
                9,
                5,
                0,
                700,
                0x1111,
                0x2222, //
                5,
                1,
                2,
                1,
                0,
                region_start + 4,
                0,
                1,
                0,
                0x3333,
                0x4444,
            ],
            "起点、字节数、跨度、读到没有、盘、槽号低 32 位、高 32 位、哪一种片、单元在这一片之后的字节数、两份核各自的寄存器"
        );
    assert_eq!(packed.record_count(), 4);
}

#[test]
fn mapping_tables_list_each_entry_with_its_key_length_and_both_locations() {
    let tables = MappingTablesForTheKernels::of_the_mapping_entries(&[central_mapping_entry(
        vec![1, 2, 3, 4, 5],
        [
            LocationEntry {
                device: DeviceIdentity(3),
                slot: SlotNumber((1_u64 << 32) | 2),
                unit_checksum: 0,
            },
            LocationEntry {
                device: DeviceIdentity(4),
                slot: SlotNumber(6),
                unit_checksum: 0,
            },
        ],
    )])
    .expect("5 字节的 key 装得下");
    assert_eq!(
        tables.entry_words,
        vec![5, 0x0403_0201, 5, 0, 0, 0, 0, 0, 3, 2, 1, 4, 6, 0],
        "key 的字节数、key 的七个字、两条位置项各三个字"
    );
    assert_eq!((tables.entry_count, tables.location_item_count), (1, 2));
}

#[test]
fn the_location_index_is_sorted_by_device_and_slot_and_lists_an_entry_once_per_distinct_location() {
    let same_location_twice = [LocationEntry {
        device: DeviceIdentity(0),
        slot: SlotNumber(70),
        unit_checksum: 0,
    }; 2];
    let tables = MappingTablesForTheKernels::of_the_mapping_entries(&[
        central_mapping_entry(vec![1], locations_at(SlotNumber(90))),
        central_mapping_entry(vec![2], same_location_twice),
        central_mapping_entry(vec![3], locations_at(SlotNumber(70))),
    ])
    .expect("短 key 装得下");
    assert_eq!(
        tables.location_item_words,
        vec![
            0, 70, 0, 1, //
            0, 70, 0, 2, //
            0, 90, 0, 0, //
            1, 70, 0, 2, //
            1, 90, 0, 0,
        ],
        "按（盘、槽号、序号）升序；两条位置项指同一落点的那条条目只记一项"
    );
    assert_eq!(tables.location_item_count, 5);
}

#[test]
fn the_key_index_is_sorted_by_key_length_and_words_with_ties_in_table_order() {
    let tables = MappingTablesForTheKernels::of_the_mapping_entries(&[
        central_mapping_entry(vec![9, 9], locations_at(SlotNumber(1))),
        central_mapping_entry(vec![1, 2, 3], locations_at(SlotNumber(2))),
        central_mapping_entry(vec![5], locations_at(SlotNumber(3))),
        central_mapping_entry(vec![9, 9], locations_at(SlotNumber(4))),
    ])
    .expect("短 key 装得下");
    assert_eq!(
        tables.key_sorted_entry_positions,
        vec![2, 0, 3, 1],
        "先比字节数、再比字；相同的 key 按条目表的次序"
    );
}

#[test]
fn one_mapping_key_longer_than_the_kernels_hold_refuses_the_tables() {
    assert_eq!(
        MappingTablesForTheKernels::of_the_mapping_entries(&[
            central_mapping_entry(vec![1; 28], locations_at(SlotNumber(1))),
            central_mapping_entry(vec![1; 29], locations_at(SlotNumber(2))),
        ]),
        Err(MappingKeyLongerThanTheKernelsHold {
            entry_position: 1,
            key_bytes: 29
        }),
        "28 字节正好装满七个字，29 字节装不下"
    );
}

#[test]
fn empty_mapping_tables_still_have_one_item_in_every_buffer() {
    let tables = MappingTablesForTheKernels::of_the_mapping_entries(&[]).expect("空映射建得成");
    assert_eq!(tables.entry_words, vec![0; MAPPING_ENTRY_WORDS]);
    assert_eq!(tables.location_item_words, vec![0; LOCATION_ITEM_WORDS]);
    assert_eq!(tables.key_sorted_entry_positions, vec![0]);
    assert_eq!((tables.entry_count, tables.location_item_count), (0, 0));
    assert_eq!(
        tables.bytes_on_the_gpu(),
        GpuMemoryInBytes(
            4 * u64::try_from(MAPPING_ENTRY_WORDS + LOCATION_ITEM_WORDS + 1).expect("小数")
        )
    );
}

fn region_record_outcome_bytes_of_the_words(
    outcome_words: [u32; REGION_RECORD_OUTCOME_WORDS],
) -> Vec<u8> {
    little_endian_bytes_of_the_words(&outcome_words)
}

#[test]
fn every_outcome_kind_decodes_into_its_region_scan_class() {
    let (_bytes, data_key) = data_unit_and_key(4);
    let mut movable_key_words = [0_u32; KEY_WORDS];
    let mut padded_key = data_key.clone();
    padded_key.resize(KEY_BYTES_THE_KERNELS_HOLD, 0);
    for (word, word_bytes) in movable_key_words
        .iter_mut()
        .zip(padded_key.as_chunks::<4>().0)
    {
        *word = u32::from_le_bytes(*word_bytes);
    }
    let key_length = u32::try_from(data_key.len()).expect("27");
    let mut readback = Vec::new();
    let [key_word_0, key_word_1, key_word_2, key_word_3, key_word_4, key_word_5, key_word_6] =
        movable_key_words;
    readback.extend(region_record_outcome_bytes_of_the_words([
        OUTCOME_KIND_MOVABLE,
        u32::from(UNIT_CLASS_DATA),
        0,
        7,
        1,
        key_length,
        key_word_0,
        key_word_1,
        key_word_2,
        key_word_3,
        key_word_4,
        key_word_5,
        key_word_6,
    ]));
    readback.extend(region_record_outcome_bytes_of_the_words([
        OUTCOME_KIND_DEAD,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]));
    readback.extend(region_record_outcome_bytes_of_the_words([
        OUTCOME_KIND_UNIT_CLASS_DISAGREES_WITH_THE_MAPPING_KEY,
        u32::from(UNIT_CLASS_PACKED),
        u32::from(UNIT_CLASS_INDEX_NODE),
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]));
    readback.extend(region_record_outcome_bytes_of_the_words([
        OUTCOME_KIND_HEADER_KEY_DISAGREES_WITH_THE_MAPPING_LOCATION,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]));
    readback.extend(region_record_outcome_bytes_of_the_words([
        OUTCOME_KIND_DEVICE_READ_FAILED,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]));
    readback.extend(region_record_outcome_bytes_of_the_words([
        OUTCOME_KIND_SPAN_IS_NOT_A_UNIT_FOOTPRINT,
        3,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]));
    readback.extend(region_record_outcome_bytes_of_the_words([
        OUTCOME_KIND_REFUSED_BY_THE_PARSER,
        UNIT_ERROR_WRONG_UNIT_CLASS,
        u32::from(UNIT_CLASS_DATA) | (2 << 8),
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]));
    readback.extend(region_record_outcome_bytes_of_the_words([
        OUTCOME_KIND_REFUSED_BY_THE_PARSER,
        UNIT_ERROR_STRUCTURE,
        7,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]));
    let unreadable = |unreadable| {
        AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(unreadable))
    };
    assert_eq!(
        decoded_region_record_outcomes(&readback),
        Ok(vec![
            AllocationInRegion::Movable(MovableUnit {
                mapping_key: data_key,
                unit_class: MappedUnitClass::Data,
                birth_txg: CheckpointTxg((1 << 32) | 7),
            }),
            AllocationInRegion::Dead,
            AllocationInRegion::Inconsistent(
                RegionScanInconsistency::UnitClassDisagreesWithTheMappingKey {
                    header_unit_class: MappedUnitClass::Packed,
                    mapping_key_class_tag: UNIT_CLASS_INDEX_NODE,
                }
            ),
            AllocationInRegion::Inconsistent(
                RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation
            ),
            unreadable(UnitHeaderUnreadable::DeviceReadFailed),
            unreadable(UnitHeaderUnreadable::SpanIsNotAUnitFootprint { span_slots: 3 }),
            unreadable(UnitHeaderUnreadable::RefusedByTheParser(
                UnitError::WrongUnitClass {
                    expected: UNIT_CLASS_DATA,
                    found: 2
                }
            )),
            unreadable(UnitHeaderUnreadable::RefusedByTheParser(
                UnitError::Structure(UnitStructureDefect::PackedUnitRecordAreaPastTheUnitEnd)
            )),
        ])
    );
}

#[test]
fn region_record_outcome_the_kernels_never_wrote_is_an_unrecognized_verdict_word() {
    let mut readback = region_record_outcome_bytes_of_the_words([
        OUTCOME_KIND_DEAD,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]);
    readback.extend(region_record_outcome_buffer_before_the_kernel_writes(1));
    assert_eq!(
        decoded_region_record_outcomes(&readback),
        Err(UnrecognizedVerdictWord {
            entry_position: 1,
            verdict_word: REGION_RECORD_OUTCOME_NOT_WRITTEN_FILL_WORD
        }),
        "位置 1 的那一条还是填充字：核没写到它"
    );
}

/// 核心层整个 R 扫一遍的分类，与 CPU 参照逐条单独判的分类逐条相同：可搬的三类、已死、类标签不符、key 与位置不符、读不到、跨度不对、头校验和坏。
#[test]
fn the_cpu_reference_classifies_each_record_as_the_whole_region_scan_does() {
    let (data_bytes, data_key) = data_unit_and_key(4);
    let (node_bytes, node_key) = index_node_and_key(5);
    let (packed_bytes, packed_key) = packed_unit_and_key(6);
    let (dead_bytes, _dead_key) = data_unit_and_key(40);
    let (other_node_bytes, _other_node_key) = index_node_and_key(50);
    let (mislabelled_bytes, _mislabelled_key) = data_unit_and_key(60);
    let mut header_checksum_broken = data_unit_and_key(70).0;
    header_checksum_broken[20] ^= 1;
    let records_and_bytes: Vec<(AllocationRecord, Option<Vec<u8>>)> = vec![
        (unreleased_record(slot_in_region(0), 2), Some(data_bytes)),
        (unreleased_record(slot_in_region(2), 1), Some(node_bytes)),
        (unreleased_record(slot_in_region(4), 2), Some(packed_bytes)),
        (unreleased_record(slot_in_region(6), 2), Some(dead_bytes)),
        (
            unreleased_record(slot_in_region(8), 1),
            Some(other_node_bytes),
        ),
        (
            unreleased_record(slot_in_region(10), 2),
            Some(mislabelled_bytes),
        ),
        (unreleased_record(slot_in_region(12), 2), None),
        (unreleased_record(slot_in_region(14), 3), None),
        (
            unreleased_record(slot_in_region(18), 2),
            Some(header_checksum_broken),
        ),
    ];
    let mapping_entries = vec![
        central_mapping_entry(data_key, locations_at(slot_in_region(0))),
        central_mapping_entry(node_key.clone(), locations_at(slot_in_region(2))),
        central_mapping_entry(packed_key, locations_at(slot_in_region(4))),
        // 槽 8 上那个节点的 key 不在映射里，指着槽 8 的是槽 2 那个节点的 key
        central_mapping_entry(node_key, locations_at(slot_in_region(8))),
        // 槽 10 上是数据单元，指着它的条目 key 打头是节点的类标签
        central_mapping_entry(index_node_and_key(61).1, locations_at(slot_in_region(10))),
    ];
    let records: Vec<AllocationRecord> = records_and_bytes
        .iter()
        .map(|(record, _bytes)| *record)
        .collect();
    let whole_region_scan = scan_region(
        region_under_test(),
        &records,
        &mapping_entries,
        &BTreeSet::new(),
        &BTreeSet::new(),
        |_device, slot, _unit_bytes_requested| {
            records_and_bytes
                .iter()
                .find(|(record, _bytes)| record.slot == slot)
                .and_then(|(_record, bytes)| bytes.clone())
        },
    )
    .expect("这个 R 扫得成");
    let mapping_index = CentralMappingIdentityIndex::of_the_mapping_entries(&mapping_entries);
    let one_by_one: Vec<AllocationInRegion> = records_and_bytes
        .iter()
        .map(|(record, bytes)| {
            cpu_reference_classification(
                region_under_test(),
                *record,
                &mapping_index,
                bytes.as_deref(),
            )
        })
        .collect();
    let whole: Vec<AllocationInRegion> = whole_region_scan
        .allocations
        .into_iter()
        .map(|allocation| allocation.class)
        .collect();
    assert_eq!(one_by_one, whole);
    assert!(
        matches!(one_by_one[0], AllocationInRegion::Movable(_))
            && matches!(one_by_one[1], AllocationInRegion::Movable(_))
            && matches!(one_by_one[2], AllocationInRegion::Movable(_))
            && one_by_one[3] == AllocationInRegion::Dead,
        "三类可搬与已死都在：{one_by_one:?}"
    );
}

#[test]
#[should_panic(expected = "交进来的记录要在 R 那块盘上")]
fn the_cpu_reference_refuses_a_record_outside_the_region() {
    let mapping_index = CentralMappingIdentityIndex::of_the_mapping_entries(&[]);
    let _classification = cpu_reference_classification(
        region_under_test(),
        unreleased_record(SlotNumber(region_under_test().end_slot().0), 1),
        &mapping_index,
        None,
    );
}

/// 两份核过得了 wgpu 自带的着色器校验（naga 的 WGSL 前端与校验器，起管线时 wgpu 走的同一步），入口点在、是计算着色器。不要 GPU。
#[test]
fn both_region_record_kernels_pass_the_shader_validator() {
    for (kernel_source, entry_point) in [
        (WALKING_KERNEL_SOURCE, WALKING_KERNEL_ENTRY_POINT),
        (SEARCHING_KERNEL_SOURCE, SEARCHING_KERNEL_ENTRY_POINT),
    ] {
        let module = wgpu::naga::front::wgsl::parse_str(kernel_source).unwrap_or_else(|error| {
            panic!(
                "{entry_point} 那一份解析不成：{}",
                error.emit_to_string(kernel_source)
            )
        });
        wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap_or_else(|error| {
            panic!(
                "{entry_point} 那一份过不了校验：{}",
                error.emit_to_string(kernel_source)
            )
        });
        assert!(
            module.entry_points.iter().any(|module_entry_point| {
                module_entry_point.name == entry_point
                    && module_entry_point.stage == wgpu::naga::ShaderStage::Compute
            }),
            "{entry_point} 是这一份里的计算入口"
        );
    }
}

#[test]
fn the_walking_and_searching_region_record_kernels_are_separate_sources_with_their_own_entry_points(
) {
    assert!(WALKING_KERNEL_SOURCE.contains(&format!("fn {WALKING_KERNEL_ENTRY_POINT}(")));
    assert!(SEARCHING_KERNEL_SOURCE.contains(&format!("fn {SEARCHING_KERNEL_ENTRY_POINT}(")));
    assert!(
        !SEARCHING_KERNEL_SOURCE.contains(WALKING_KERNEL_ENTRY_POINT)
            && !WALKING_KERNEL_SOURCE.contains(SEARCHING_KERNEL_ENTRY_POINT),
        "两份核不互相引用"
    );
}

/// 第 3 条「按上限自适应切成几片」：上限只装得下最大那条 32 KiB 的记录连同一条条目的映射，映射按落点切成一片一个落点
/// （两块盘上各六个），每条记录在每片之下各分一次类（两份核在 CPU 上的替身照核心层对这一片判），两份各按自己的合法合成，
/// 与 CPU 参照对整份映射的分类逐条相同。槽 20 上那个单元头里的 key 只有一条条目带着、它指着槽 40（别的片）：本片判成已死，合成之后是
/// 「key 与位置不符」。
#[test]
fn the_mapping_cut_into_slices_under_a_small_memory_limit_classifies_every_record_as_the_whole_mapping(
) {
    let (data_bytes, data_key) = data_unit_and_key(4);
    let (node_bytes, node_key) = index_node_and_key(5);
    let (packed_bytes, packed_key) = packed_unit_and_key(6);
    let (dead_bytes, _dead_key) = data_unit_and_key(40);
    let (other_node_bytes, _other_node_key) = index_node_and_key(50);
    let (mislabelled_bytes, _mislabelled_key) = data_unit_and_key(60);
    let (key_mapped_elsewhere_bytes, key_mapped_elsewhere) = data_unit_and_key(80);
    let records_and_bytes: Vec<(AllocationRecord, Option<Vec<u8>>)> = vec![
        (unreleased_record(slot_in_region(0), 2), Some(data_bytes)),
        (unreleased_record(slot_in_region(2), 1), Some(node_bytes)),
        (unreleased_record(slot_in_region(4), 2), Some(packed_bytes)),
        (unreleased_record(slot_in_region(6), 2), Some(dead_bytes)),
        (
            unreleased_record(slot_in_region(8), 1),
            Some(other_node_bytes),
        ),
        (
            unreleased_record(slot_in_region(10), 2),
            Some(mislabelled_bytes),
        ),
        (unreleased_record(slot_in_region(12), 2), None),
        (unreleased_record(slot_in_region(14), 3), None),
        (
            unreleased_record(slot_in_region(20), 2),
            Some(key_mapped_elsewhere_bytes),
        ),
    ];
    let mapping_entries = vec![
        central_mapping_entry(data_key, locations_at(slot_in_region(0))),
        central_mapping_entry(node_key.clone(), locations_at(slot_in_region(2))),
        central_mapping_entry(packed_key, locations_at(slot_in_region(4))),
        central_mapping_entry(node_key, locations_at(slot_in_region(8))),
        central_mapping_entry(index_node_and_key(61).1, locations_at(slot_in_region(10))),
        central_mapping_entry(key_mapped_elsewhere, locations_at(slot_in_region(40))),
    ];
    let records: Vec<RegionRecordToClassify<'_, usize>> = records_and_bytes
        .iter()
        .enumerate()
        .map(|(entry_key, (record, bytes))| RegionRecordToClassify {
            entry_key,
            record: *record,
            unit_bytes: bytes.as_deref(),
        })
        .collect();
    let mapping_tables = MappingTablesForTheKernels::of_the_mapping_entries(&mapping_entries)
        .expect("key 都是 27 字节");
    let one_entry_with_one_location_bytes =
        mapping_table_bytes_of(MappingEntryCount(1), MappingLocationItemCount(1)).0;
    let memory_limit =
        region_record_chunk_footprint_of_mapping_table_bytes(one_entry_with_one_location_bytes)
            .bytes_held_by_a_chunk(InputLengthOnTheHostInBytes(32768), EntryCountOfAChunk(1));
    let room = ChunkBoundsOfAGpu {
        memory_limit: Some(memory_limit),
        largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
        largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
    };
    let plan = region_record_plan_on_a_gpu(&records, &mapping_entries, &mapping_tables, room)
        .expect("最大那条记录连同一个落点上的一条条目装得下");
    assert_eq!(
        plan.mapping_slices.len(),
        12,
        "两块盘上各六个落点，一片一个"
    );
    assert!(
        plan.chunks_of_the_pieces_after_the_heads.is_empty(),
        "整个单元装得下：一个单元都不切"
    );
    let classifications =
        region_record_classifications_of_the_plan(&records, &plan, |pieces, mapping_slice| {
            Ok::<_, ()>(stand_in_for_both_kernels(
                &records_and_bytes,
                pieces,
                mapping_slice,
            ))
        })
        .expect("替身不出错");
    let whole_index = CentralMappingIdentityIndex::of_the_mapping_entries(&mapping_entries);
    let reference: Vec<AllocationInRegion> = records
        .iter()
        .map(|to_classify| {
            cpu_reference_classification(
                region_under_test(),
                to_classify.record,
                &whole_index,
                to_classify.unit_bytes,
            )
        })
        .collect();
    assert_eq!(
        reference[8],
        header_key_disagrees_with_the_mapping_location(),
        "槽 20 那一条按整份映射是 key 与位置不符"
    );
    assert_eq!(
        reference[3],
        AllocationInRegion::Dead,
        "槽 6 那一条按整份映射是已死"
    );
    assert_eq!(
        classifications,
        reference
            .into_iter()
            .map(|classification| (classification.clone(), classification))
            .collect::<Vec<_>>(),
        "映射切成几片、各片合成，两份照样逐条与 CPU 参照对整份映射的分类相同"
    );
}

/// CRC-32C 的寄存器（不取反、不带全 1 初值）逐位移喂 `bytes` 之后的值：替身算头一片之后那几片用它。
fn crc_register_after_bytes(register_before: u32, bytes: &[u8]) -> u32 {
    let mut register = register_before;
    for byte in bytes {
        register ^= u32::from(*byte);
        for _bit in 0..8 {
            register = if register & 1 == 1 {
                (register >> 1) ^ 0x82F6_3B78
            } else {
                register >> 1
            };
        }
    }
    register
}

/// 两份核在 CPU 上的替身对一段的交法（见本模块文件头「一个单元切成几片」）。头一片之后的一片：两份各自从带进来的寄存器接着算这几个字节，
/// 交回寄存器。从单元头起的一片：照记录在 `records_and_bytes` 里找回整个单元，核对这一片就是单元的开头、之后的字节数对、两份各自带进来的寄存器
/// 就是之后那些字节从 0 起算出的；对得上就照核心层对整个单元、这一片映射分类，对不上那一份交「载荷校验和不对」（与整个单元的分类不同，用例比对时红）。
fn stand_in_for_both_kernels(
    records_and_bytes: &[(AllocationRecord, Option<Vec<u8>>)],
    pieces: &[RegionRecordPieceForTheKernels<'_>],
    mapping_slice: Option<&MappingSliceWithItsChunks>,
) -> BothKernelOutcomesOfARegionRecordChunk {
    let whole_unit_of = |record: AllocationRecord| {
        records_and_bytes
            .iter()
            .find(|(whole_record, _bytes)| *whole_record == record)
            .expect("片是用例交进去的记录的")
            .1
            .clone()
    };
    let Some(mapping_slice) = mapping_slice else {
        return BothKernelOutcomesOfARegionRecordChunk::RegistersAfterThePieces(
            pieces
                .iter()
                .map(|piece| {
                    let RegionRecordPieceForTheKernels::PieceAfterTheHead {
                        bytes,
                        registers_before_this_piece,
                        ..
                    } = piece
                    else {
                        panic!("不交映射片的段全是头一片之后的片：{piece:?}");
                    };
                    CrcRegistersOfTheTwoKernels {
                        walking_kernel: crc_register_after_bytes(
                            registers_before_this_piece.walking_kernel,
                            bytes,
                        ),
                        searching_kernel: crc_register_after_bytes(
                            registers_before_this_piece.searching_kernel,
                            bytes,
                        ),
                    }
                })
                .collect(),
        );
    };
    let slice_index = CentralMappingIdentityIndex::of_the_mapping_entries(&mapping_slice.entries);
    let payload_checksum_mismatch =
        AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
            UnitHeaderUnreadable::RefusedByTheParser(UnitError::PayloadChecksumMismatch),
        ));
    BothKernelOutcomesOfARegionRecordChunk::Classifications(
        pieces
            .iter()
            .map(|piece| {
                let RegionRecordPieceForTheKernels::UnitFromItsStart {
                    record,
                    bytes,
                    bytes_of_the_unit_after_this_piece,
                    registers_of_the_bytes_after_this_piece,
                } = piece
                else {
                    panic!("交映射片的段全是从单元头起的片：{piece:?}");
                };
                let whole_unit = whole_unit_of(*record);
                let classification = cpu_reference_classification(
                    region_under_test(),
                    *record,
                    &slice_index,
                    whole_unit.as_deref(),
                );
                let piece_is_the_start_of_the_unit = match (bytes, &whole_unit) {
                    (None, None) => *bytes_of_the_unit_after_this_piece == 0,
                    (Some(bytes), Some(whole_unit)) => {
                        whole_unit.starts_with(bytes)
                            && usize::try_from(*bytes_of_the_unit_after_this_piece).expect("小数")
                                == whole_unit.len() - bytes.len()
                    }
                    (None, Some(_)) | (Some(_), None) => false,
                };
                let register_of_the_rest = whole_unit.as_ref().map_or(0, |whole_unit| {
                    crc_register_after_bytes(0, &whole_unit[bytes.map_or(0, <[u8]>::len)..])
                });
                let classification_given = |register_carried_in: u32| {
                    if piece_is_the_start_of_the_unit && register_carried_in == register_of_the_rest
                    {
                        classification.clone()
                    } else {
                        payload_checksum_mismatch.clone()
                    }
                };
                (
                    classification_given(registers_of_the_bytes_after_this_piece.walking_kernel),
                    classification_given(registers_of_the_bytes_after_this_piece.searching_kernel),
                )
            })
            .collect(),
    )
}

/// 第 3 条「装不下不拒绝」「按上限自适应切成几片、缩小逐次交」：上限 2 KiB，连一个 16 KiB 的索引节点都装不下
/// （数据单元的载荷两万字节、字节各不相同，头一片之后的寄存器不是 0）：
/// 比头一片长的单元都切成头一片与之后的几片（之后的几片按上限切、同一个单元的各片在前后不同的段里），两份核在 CPU 上的替身先逐片接着算寄存器、
/// 再照头一片分类（替身核对头一片、之后的字节数与两份各自带进来的寄存器，对不上就交「载荷校验和不对」）；每条记录两份的分类与 CPU 参照对整个单元、
/// 整份映射的分类逐条相同。三类单元、已死、读不到、跨度不对、key 与位置不符都在里面。
#[test]
fn units_cut_into_pieces_under_a_memory_limit_smaller_than_a_unit_classify_every_record_as_the_whole_unit(
) {
    let (data_bytes, data_key) = data_unit_with_a_long_payload_and_key(4);
    let (node_bytes, node_key) = index_node_and_key(5);
    let (packed_bytes, packed_key) = packed_unit_and_key(6);
    let (dead_bytes, _dead_key) = data_unit_with_a_long_payload_and_key(40);
    let (key_mapped_elsewhere_bytes, key_mapped_elsewhere) =
        data_unit_with_a_long_payload_and_key(80);
    let records_and_bytes: Vec<(AllocationRecord, Option<Vec<u8>>)> = vec![
        (unreleased_record(slot_in_region(0), 2), Some(data_bytes)),
        (unreleased_record(slot_in_region(2), 1), Some(node_bytes)),
        (unreleased_record(slot_in_region(4), 2), Some(packed_bytes)),
        (unreleased_record(slot_in_region(6), 2), Some(dead_bytes)),
        (unreleased_record(slot_in_region(12), 2), None),
        (unreleased_record(slot_in_region(14), 3), None),
        (
            unreleased_record(slot_in_region(20), 2),
            Some(key_mapped_elsewhere_bytes),
        ),
    ];
    let mapping_entries = vec![
        central_mapping_entry(data_key, locations_at(slot_in_region(0))),
        central_mapping_entry(node_key, locations_at(slot_in_region(2))),
        central_mapping_entry(packed_key, locations_at(slot_in_region(4))),
        central_mapping_entry(key_mapped_elsewhere, locations_at(slot_in_region(40))),
    ];
    let records: Vec<RegionRecordToClassify<'_, usize>> = records_and_bytes
        .iter()
        .enumerate()
        .map(|(entry_key, (record, bytes))| RegionRecordToClassify {
            entry_key,
            record: *record,
            unit_bytes: bytes.as_deref(),
        })
        .collect();
    let mapping_tables = MappingTablesForTheKernels::of_the_mapping_entries(&mapping_entries)
        .expect("key 都是 27 字节");
    let room = ChunkBoundsOfAGpu {
        memory_limit: Some(GpuMemoryInBytes(2048)),
        largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
        largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
    };
    let plan = region_record_plan_on_a_gpu(&records, &mapping_entries, &mapping_tables, room)
        .expect("切开之后头一片连同一个落点上的条目、之后的每一片都装得下 2 KiB");
    assert_eq!(
        plan.bytes_from_the_start_of_each_record,
        vec![
            UNIT_HEAD_PIECE_INPUT_BYTES,
            UNIT_HEAD_PIECE_INPUT_BYTES,
            UNIT_HEAD_PIECE_INPUT_BYTES,
            UNIT_HEAD_PIECE_INPUT_BYTES,
            InputLengthOnTheHostInBytes(0),
            InputLengthOnTheHostInBytes(0),
            UNIT_HEAD_PIECE_INPUT_BYTES,
        ],
        "读到的单元都切成头一片，读不到的交 0 字节"
    );
    let pieces_after_the_head_of_the_index_node: InputLengthOnTheHostInBytes = plan
        .chunks_of_the_pieces_after_the_heads
        .iter()
        .flatten()
        .filter(|piece| piece.entry_position == 1)
        .map(PieceOfAnEntry::input_byte_count)
        .sum();
    assert_eq!(
        pieces_after_the_head_of_the_index_node,
        InputLengthOnTheHostInBytes(16384).minus(UNIT_HEAD_PIECE_INPUT_BYTES),
        "索引节点头一片之后的几片一个字节不漏"
    );
    assert!(
        plan.chunks_of_the_pieces_after_the_heads.len() > 1,
        "头一片之后的字节切成了好几段"
    );
    let classifications =
        region_record_classifications_of_the_plan(&records, &plan, |pieces, mapping_slice| {
            Ok::<_, ()>(stand_in_for_both_kernels(
                &records_and_bytes,
                pieces,
                mapping_slice,
            ))
        })
        .expect("替身不出错");
    let whole_index = CentralMappingIdentityIndex::of_the_mapping_entries(&mapping_entries);
    let reference: Vec<AllocationInRegion> = records
        .iter()
        .map(|to_classify| {
            cpu_reference_classification(
                region_under_test(),
                to_classify.record,
                &whole_index,
                to_classify.unit_bytes,
            )
        })
        .collect();
    assert!(
        matches!(reference[0], AllocationInRegion::Movable(_))
            && matches!(reference[1], AllocationInRegion::Movable(_))
            && matches!(reference[2], AllocationInRegion::Movable(_))
            && reference[3] == AllocationInRegion::Dead
            && reference[6] == header_key_disagrees_with_the_mapping_location(),
        "三类可搬、已死、key 与位置不符都在里面：{reference:?}"
    );
    assert_eq!(
        classifications,
        reference
            .into_iter()
            .map(|classification| (classification.clone(), classification))
            .collect::<Vec<_>>(),
        "单元切成几片交，两份照样逐条与 CPU 参照对整个单元的分类相同"
    );
}

/// 头一片之后那几片的结果读回来是每片的寄存器；核没写到的那一片（还是填充字）认不得，按 GPU 出错交回。
#[test]
fn registers_after_the_pieces_decode_only_from_the_kind_the_kernels_write_for_them() {
    let mut outcome_words = [0_u32; REGION_RECORD_OUTCOME_WORDS];
    outcome_words[0] = OUTCOME_KIND_REGISTER_AFTER_THE_PIECE;
    outcome_words[1] = 0xDEAD_BEEF;
    let mut readback = region_record_outcome_bytes_of_the_words(outcome_words);
    readback.extend(region_record_outcome_buffer_before_the_kernel_writes(1));
    assert_eq!(
        decoded_registers_after_the_pieces(&readback),
        Err(UnrecognizedVerdictWord {
            entry_position: 1,
            verdict_word: REGION_RECORD_OUTCOME_NOT_WRITTEN_FILL_WORD,
        }),
        "核没写到的那一片认不得"
    );
    assert_eq!(
        decoded_registers_after_the_pieces(&readback[..52]),
        Ok(vec![0xDEAD_BEEF]),
        "写了的那一片读出寄存器"
    );
}

/// 映射三张表的字节数按条数与落点项数现算，与真建出来的相同（一项都没有时各放一项）。
#[test]
fn mapping_table_bytes_counted_from_the_entries_match_the_tables_built() {
    let (data_bytes_unused, data_key) = data_unit_and_key(4);
    drop(data_bytes_unused);
    for (entries, expected_location_items) in [
        (Vec::new(), 0_u64),
        (
            vec![central_mapping_entry(
                data_key.clone(),
                locations_at(slot_in_region(0)),
            )],
            2,
        ),
        (
            vec![
                central_mapping_entry(data_key.clone(), locations_at(slot_in_region(0))),
                central_mapping_entry(
                    data_key,
                    [
                        locations_at(slot_in_region(3))[0],
                        locations_at(slot_in_region(3))[0],
                    ],
                ),
            ],
            3,
        ),
    ] {
        let tables =
            MappingTablesForTheKernels::of_the_mapping_entries(&entries).expect("key 都是 27 字节");
        assert_eq!(
            mapping_table_bytes_of(
                MappingEntryCount(u64::try_from(entries.len()).expect("小数")),
                MappingLocationItemCount(expected_location_items)
            ),
            (tables.bytes_on_the_gpu(), tables.largest_buffer_bytes()),
            "{} 条条目",
            entries.len()
        );
    }
}
