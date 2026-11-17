//! 区域扫描里单元头解析与映射反查那两份 WGSL 核在真显卡上与 CPU 参照（核心层的区域扫描，`cpu_reference_classification`）对拍、
//! 两份核彼此对拍（D26（后台整理与放置回收） 已定项 13「GPU 路径与 CPU 路径是两份实现……两份实现在测试里对拍」）。
//! 只开服务经 `GPU_CARDS_BY_PRIORITY` 交给这个进程、属于这一台的那几张卡（`common_discrete_gpus_of_this_host`）：服务没交这一台的卡时每条打一行
//! `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」、返回成功；交了卡却对不上本机的独显、或开不了判红。源头没声明张数，服务分一张。
//! 只在开了特性 `tests-on-a-real-gpu` 时编；用 GPU，经 `task-server/task submit` 交（卡与清场由服务定），带 `--test-threads 1`。
//! 开卡时每张参与的卡的显存上限给 1 GiB（`common_discrete_gpus_of_this_host`），专门测小上限切片的几条照用更小的。
//!
//! 一个 R 里放齐各类：三类可搬（码 1、码 2、码 3）、已死、类标签不符（含指着它的条目 key 为空）、key 与位置不符（两种）、读不到、跨度不对、
//! 字节数不对（排在前面，让后面的单元起点不在字边界上），以及核心层三个 parse 的每一种拒法里造得出的：magic、版本、flags、类标签、
//! 头校验和（值错与字段尾巴非 0 两种）、载荷校验和、预留位、结构字段（码 1 两种、码 2 两种、码 3 两种）。
//!
//! 对拍分两段、经判定库交接（[`region_concluded_on_the_cards_through_the_pairing_library`]）：每条用例先把这个 R 的记录、读到的单元字节与映射条目编成字节过库，
//! 只录那一趟到此返回、不开卡；判段的对拍结论按键入库复用（读回输入的摘要、参照一侧与这条用例 GPU 一侧的判法单元键），库里有两边一致的不开卡；
//! 要现判的从库里读回，交给 GPU 的与比的 CPU 参照都是读回的那一份。每条用例的持卡判入口是 `judged_on_the_cards_that_<用例名>`
//! （登记在 `src/gpu_side_units_entries.rs`）。
mod common_discrete_gpus_of_this_host;
mod common_pairing_input_through_the_library;

use std::collections::BTreeMap;

use common_discrete_gpus_of_this_host::{
    background_gpu_on_the_handed_over_gpus_under_one_gibibyte_each_or_report_not_run,
    handed_over_gpus_or_report_not_run, opened_background_gpu, selection_over_the_handed_over_gpus,
    HandedOverGpusSeenThroughTheInstance,
};

use common_pairing_input_through_the_library::{
    append_count, append_length_prefixed_bytes, append_number,
    judgement_of_the_assertions_on_the_cards,
    pairing_concluded_on_the_cards_through_the_pairing_library, ComparisonOnTheCards,
    PairingInputBeingRead,
};
use singlefs_background_gpu::gpus_of_this_host::bytes_of_the_smallest_piece_every_participating_gpu_holds;
use singlefs_background_gpu::memory_limit_and_chunks::{
    EntryCountOfAChunk, InputLengthOnTheHostInBytes,
};
use singlefs_background_gpu::region_record_kernels::{
    cpu_reference_classification, region_record_chunk_footprint_of_mapping_table_bytes,
    MappingTablesForTheKernels, RegionRecordToClassify, UNIT_HEAD_PIECE_INPUT_BYTES,
};
use singlefs_background_gpu::two_kernel_reconciliation::{
    ConsecutiveUncertaintyCounts, EntryDecision,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
};
use singlefs_core::allocator::{AllocationRecord, AllocationRecordQuarantineMark};
use singlefs_core::background_compute::GpuMemoryInBytes;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
use singlefs_core::records::{mapping_key_for_data, mapping_key_for_node};
use singlefs_core::region_scan::{
    AllocationInRegion, CentralMappingEntry, CentralMappingIdentityIndex,
};
use singlefs_core::unit::{
    build_data_unit, build_index_node, build_packed_unit, seal_header_checksum, DataUnitIdentity,
    PackedIdentity, WriteOrder, PACKED_TYPE_INODE, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
};
use singlefs_verification::crash_library::PairingJudged;

/// 等 GPU 的上限：这几批在健康的卡上远用不了这么久。
const WAIT_LIMIT_OF_THE_GPU_TESTS: std::time::Duration = std::time::Duration::from_secs(60);
const REGION_DEVICE: DeviceIdentity = DeviceIdentity(0);
const MIRROR_DEVICE: DeviceIdentity = DeviceIdentity(1);
const BIRTH_TXG: CheckpointTxg = CheckpointTxg((3 << 32) | 7);
const WRITING_INSTANCE: InstanceGeneration = InstanceGeneration(2);
const FILESYSTEM_IDENTIFIER: [u8; 16] = [7; 16];
const TREE_OF_THE_TEST_DATA: u64 = 11;
const TREE_OF_THE_TEST_NODES: u64 = 12;
/// 测试造的码 2 节点 key 宽 24：头宽 86 + 2 × 24。
const KEY_WIDTH_OF_THE_TEST_NODES: usize = 24;
const HEADER_END_OF_THE_TEST_NODES: usize = 86 + 2 * KEY_WIDTH_OF_THE_TEST_NODES;
/// 码 1 与码 3 的头宽与载荷 CRC 的位置（字节表；造坏单元之后重封校验和用）。
const DATA_UNIT_HEADER_END: usize = 105;
const DATA_UNIT_PAYLOAD_CRC_OFFSET: usize = 101;
const PACKED_UNIT_HEADER_END: usize = 107;

fn region_under_test() -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(REGION_DEVICE, SlotNumber((1 << 33) + 64 * 900))
        .expect("64 的倍数，装得进 6 字节")
}

fn locations_at(slot: SlotNumber) -> [LocationEntry; 2] {
    [REGION_DEVICE, MIRROR_DEVICE].map(|device| LocationEntry {
        device,
        slot,
        unit_checksum: 0,
    })
}

/// 码 1 单元的载荷多长。封闭集合。
#[derive(Clone, Copy)]
enum PayloadOfTheDataUnits {
    /// 100 字节：单元里不是 0 的字节全落在头一片（`UNIT_HEAD_PIECE_INPUT_BYTES`）里，切开之后头一片之后全是 0 字节，从 0 起算的寄存器也是 0，
    /// 两份核接没接上头一片之后的寄存器、合没合进头一片都看不出来。
    WithinTheHeadPiece,
    /// 两万字节、字节各不相同：头一片之后的寄存器不是 0，接错、合错的那一份核把可搬的码 1 单元判成载荷校验和不对。
    RunningPastTheHeadPiece,
}

fn data_unit_and_key(
    transaction: u64,
    payload_of_the_data_units: PayloadOfTheDataUnits,
) -> (Vec<u8>, Vec<u8>) {
    let write_order = WriteOrder {
        instance: WRITING_INSTANCE,
        transaction,
    };
    let payload: Vec<u8> = match payload_of_the_data_units {
        PayloadOfTheDataUnits::WithinTheHeadPiece => vec![0x5A; 100],
        PayloadOfTheDataUnits::RunningPastTheHeadPiece => (0..20_000_u64)
            .map(|position| u8::try_from((position + transaction) % 251).expect("小于 251"))
            .collect(),
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
        KEY_WIDTH_OF_THE_TEST_NODES,
        &[0; KEY_WIDTH_OF_THE_TEST_NODES],
        &[0; KEY_WIDTH_OF_THE_TEST_NODES],
        BIRTH_TXG,
        &FILESYSTEM_IDENTIFIER,
        WRITING_INSTANCE,
        BirthSequence(birth_sequence),
        112,
        &[vec![3; 112], vec![4; 112]],
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
        &[vec![1; 140], vec![2; 140]],
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

/// 改了头里的字节之后重封头校验和，让解析走到后面那一关。
fn with_the_header_resealed(mut unit_bytes: Vec<u8>, header_end: usize) -> Vec<u8> {
    seal_header_checksum(&mut unit_bytes, header_end);
    unit_bytes
}

/// 改了载荷 CRC 罩住的字节之后重算载荷 CRC（在码 1 的头里），再重封头校验和。
fn data_unit_with_the_payload_resealed(mut unit_bytes: Vec<u8>) -> Vec<u8> {
    let payload_crc = crc32_castagnoli(&unit_bytes[DATA_UNIT_HEADER_END..]);
    unit_bytes[DATA_UNIT_PAYLOAD_CRC_OFFSET..DATA_UNIT_PAYLOAD_CRC_OFFSET + 4]
        .copy_from_slice(&payload_crc.to_le_bytes());
    with_the_header_resealed(unit_bytes, DATA_UNIT_HEADER_END)
}

fn with_bytes_set(mut unit_bytes: Vec<u8>, offset: usize, replacement: &[u8]) -> Vec<u8> {
    unit_bytes[offset..offset + replacement.len()].copy_from_slice(replacement);
    unit_bytes
}

/// 一个 R 里的记录、各自读到的字节与这一版的中央映射。
struct RegionUnderTest {
    records_and_bytes: Vec<(AllocationRecord, Option<Vec<u8>>)>,
    mapping_entries: Vec<CentralMappingEntry>,
    /// CPU 参照在只录那一段分好类、写进库的每条记录的分类（`{:?}` 的文字）；现造的那一份（还没过库）是空的。
    reference_classification_texts: Vec<String>,
}

/// 一批分类各自 `{:?}` 的文字：GPU 那一侧的分类换成它，与库里读回的参照逐条比。
fn debug_texts_of(classifications: &[AllocationInRegion]) -> Vec<String> {
    classifications
        .iter()
        .map(|classification| format!("{classification:?}"))
        .collect()
}

impl RegionUnderTest {
    fn records_to_classify(&self) -> Vec<RegionRecordToClassify<'_, usize>> {
        self.records_and_bytes
            .iter()
            .enumerate()
            .map(|(entry_key, (record, bytes))| RegionRecordToClassify {
                entry_key,
                record: *record,
                unit_bytes: bytes.as_deref(),
            })
            .collect()
    }

    fn cpu_reference_classifications(&self) -> Vec<AllocationInRegion> {
        let mapping_index =
            CentralMappingIdentityIndex::of_the_mapping_entries(&self.mapping_entries);
        self.records_and_bytes
            .iter()
            .map(|(record, bytes)| {
                cpu_reference_classification(
                    region_under_test(),
                    *record,
                    &mapping_index,
                    bytes.as_deref(),
                )
            })
            .collect()
    }
}

impl RegionUnderTest {
    /// 这个 R 编成对拍输入的字节：R 的起点槽，记录条数，逐条记录是盘号、槽号、跨几个槽、代、释放没有（0 或 1）、读没读到（0 或 1）、
    /// 读到的带长度的单元字节；映射条目数，逐条是带长度的 key 与两个落点（盘号、槽号、单元校验和）。数都是 8 字节小端。
    fn pairing_input_bytes(&self) -> Vec<u8> {
        let mut input_bytes = Vec::new();
        append_number(&mut input_bytes, region_under_test().start_slot().0);
        append_count(&mut input_bytes, self.records_and_bytes.len());
        for (record, bytes) in &self.records_and_bytes {
            assert_eq!(
                record.quarantine_mark,
                AllocationRecordQuarantineMark::NotQuarantined,
                "这个 R 的记录都没隔离：隔离记号不进对拍输入"
            );
            append_number(&mut input_bytes, u64::from(record.device.0));
            append_number(&mut input_bytes, record.slot.0);
            append_number(&mut input_bytes, u64::from(record.span_slots));
            append_number(&mut input_bytes, record.generation.0);
            append_number(&mut input_bytes, u64::from(record.is_released));
            match bytes {
                None => append_number(&mut input_bytes, 0),
                Some(unit_bytes) => {
                    append_number(&mut input_bytes, 1);
                    append_length_prefixed_bytes(&mut input_bytes, unit_bytes);
                }
            }
        }
        append_count(&mut input_bytes, self.mapping_entries.len());
        for mapping_entry in &self.mapping_entries {
            append_length_prefixed_bytes(&mut input_bytes, &mapping_entry.key);
            for location in mapping_entry.locations {
                append_number(&mut input_bytes, u64::from(location.device.0));
                append_number(&mut input_bytes, location.slot.0);
                append_number(&mut input_bytes, u64::from(location.unit_checksum));
            }
        }
        input_bytes
    }

    /// [`RegionUnderTest::pairing_input_bytes`] 的反向。
    fn of_the_pairing_input_bytes(input_bytes: &[u8]) -> Self {
        fn flag(number: u64) -> bool {
            match number {
                0 => false,
                1 => true,
                other => panic!("只写 0 或 1 的那一格读回 {other}"),
            }
        }
        fn device(number: u64) -> DeviceIdentity {
            DeviceIdentity(u32::try_from(number).expect("写下的盘号装得进 u32"))
        }
        let mut input = PairingInputBeingRead::from_the_start_of(input_bytes);
        assert_eq!(
            input.take_number(),
            region_under_test().start_slot().0,
            "读回的 R 起点与这份测试的 R 相同"
        );
        let record_count = input.take_count();
        let records_and_bytes = (0..record_count)
            .map(|_record| {
                let record = AllocationRecord {
                    device: device(input.take_number()),
                    slot: SlotNumber(input.take_number()),
                    span_slots: u16::try_from(input.take_number()).expect("写下的跨槽数装得进 u16"),
                    generation: CheckpointTxg(input.take_number()),
                    is_released: flag(input.take_number()),
                    quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
                };
                let bytes =
                    flag(input.take_number()).then(|| input.take_length_prefixed_bytes().to_vec());
                (record, bytes)
            })
            .collect();
        let mapping_entry_count = input.take_count();
        let mapping_entries = (0..mapping_entry_count)
            .map(|_mapping_entry| {
                let key = input.take_length_prefixed_bytes().to_vec();
                let locations = [(); 2].map(|()| LocationEntry {
                    device: device(input.take_number()),
                    slot: SlotNumber(input.take_number()),
                    unit_checksum: u32::try_from(input.take_number())
                        .expect("写下的单元校验和装得进 u32"),
                });
                CentralMappingEntry { key, locations }
            })
            .collect();
        input.finish();
        Self {
            records_and_bytes,
            mapping_entries,
            reference_classification_texts: Vec::new(),
        }
    }
}

/// 一条区域记录真卡对拍用例走这一条：放齐各类的那个 R 过判定库（项名用用例名），参照一侧是登记表里区域记录核 CPU 参照那一组，
/// 判段的对拍结论按键入库复用（`common_pairing_input_through_the_library::pairing_concluded_on_the_cards_through_the_pairing_library`）。
/// 要现判的才调 `judged_on_the_cards`（这条用例的持卡判入口）。
fn region_concluded_on_the_cards_through_the_pairing_library(
    test_name: &str,
    payload_of_the_data_units: PayloadOfTheDataUnits,
    judged_on_the_cards: impl FnOnce(&str, &[u8], &[u8]) -> PairingJudged,
) {
    pairing_concluded_on_the_cards_through_the_pairing_library(
        test_name,
        "cpu_reference_of_the_region_record_kernels",
        || {
            region_with_every_class_and_every_parser_refusal(payload_of_the_data_units)
                .pairing_input_bytes()
        },
        reference_bytes_of_the_region_pairing_input,
        judged_on_the_cards,
    );
}

/// 持卡判那一段解读回的输入与参照：那个 R，带 CPU 参照在只录那一段分好的每条记录的分类文字。
fn region_read_back(input_bytes: &[u8], reference_bytes: &[u8]) -> RegionUnderTest {
    let mut reference = PairingInputBeingRead::from_the_start_of(reference_bytes);
    let classification_count = reference.take_count();
    let reference_classification_texts = (0..classification_count)
        .map(|_classification| {
            String::from_utf8(reference.take_length_prefixed_bytes().to_vec())
                .expect("写下的分类文字是 UTF-8")
        })
        .collect();
    reference.finish();
    let RegionUnderTest {
        records_and_bytes,
        mapping_entries,
        reference_classification_texts: _not_yet_through_the_library,
    } = RegionUnderTest::of_the_pairing_input_bytes(input_bytes);
    RegionUnderTest {
        records_and_bytes,
        mapping_entries,
        reference_classification_texts,
    }
}

/// 区域记录对拍参照一侧的入口（登记在 `src/reference_units_entries.rs`）：对拍输入里那个 R 每条记录的 CPU 参照分类，编成字节
/// （条数，逐条是带长度的 `{:?}` 文字）。只录那一段调它。
fn reference_bytes_of_the_region_pairing_input(input_bytes: &[u8]) -> Vec<u8> {
    let classifications =
        RegionUnderTest::of_the_pairing_input_bytes(input_bytes).cpu_reference_classifications();
    let mut reference_bytes = Vec::new();
    append_count(&mut reference_bytes, classifications.len());
    for text in debug_texts_of(&classifications) {
        append_length_prefixed_bytes(&mut reference_bytes, text.as_bytes());
    }
    reference_bytes
}

fn region_with_every_class_and_every_parser_refusal(
    payload_of_the_data_units: PayloadOfTheDataUnits,
) -> RegionUnderTest {
    let mut next_offset_in_slots = 0_u64;
    let mut records_and_bytes = Vec::new();
    let mut slot_of = BTreeMap::new();
    let mut place = |name: &'static str, span_slots: u16, bytes: Option<Vec<u8>>| {
        let slot = SlotNumber(region_under_test().start_slot().0 + next_offset_in_slots);
        next_offset_in_slots += u64::from(span_slots);
        records_and_bytes.push((
            AllocationRecord {
                device: REGION_DEVICE,
                slot,
                span_slots,
                generation: BIRTH_TXG,
                is_released: false,
                quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
            },
            bytes,
        ));
        slot_of.insert(name, slot);
    };
    let (data_bytes, data_key) = data_unit_and_key(4, payload_of_the_data_units);
    let (node_bytes, node_key) = index_node_and_key(5);
    let (packed_bytes, packed_key) = packed_unit_and_key(6);
    // 字节数不对的排在最前：后面每个单元在输入里的起点都差 3 个字节，不在字边界上
    place(
        "字节数不对",
        2,
        Some(data_unit_and_key(1, payload_of_the_data_units).0[..3].to_vec()),
    );
    place("可搬码 1", 2, Some(data_bytes.clone()));
    place("可搬码 2", 1, Some(node_bytes.clone()));
    place("可搬码 3", 2, Some(packed_bytes.clone()));
    place(
        "已死",
        2,
        Some(data_unit_and_key(40, payload_of_the_data_units).0),
    );
    place("指着它的是别的 key", 1, Some(index_node_and_key(50).0));
    place(
        "两条指着它、一条带别的 key",
        1,
        Some(index_node_and_key(51).0),
    );
    place(
        "头里的 key 在映射里、指着它的没有",
        2,
        Some(data_unit_and_key(52, payload_of_the_data_units).0),
    );
    place(
        "类标签与条目 key 不符",
        2,
        Some(data_unit_and_key(60, payload_of_the_data_units).0),
    );
    place(
        "指着它的条目 key 为空",
        2,
        Some(data_unit_and_key(61, payload_of_the_data_units).0),
    );
    place("读不到", 2, None);
    place(
        "跨度不对",
        3,
        Some(data_unit_and_key(62, payload_of_the_data_units).0),
    );
    place(
        "magic 坏",
        2,
        Some(with_bytes_set(
            data_unit_and_key(63, payload_of_the_data_units).0,
            0,
            b"XFSU",
        )),
    );
    place(
        "版本不认得",
        2,
        Some(with_the_header_resealed(
            with_bytes_set(
                data_unit_and_key(64, payload_of_the_data_units).0,
                4,
                &[2, 0],
            ),
            DATA_UNIT_HEADER_END,
        )),
    );
    place(
        "类标签不对",
        2,
        Some(with_bytes_set(
            data_unit_and_key(65, payload_of_the_data_units).0,
            6,
            &[UNIT_CLASS_INDEX_NODE],
        )),
    );
    place(
        "flags 非 0",
        2,
        Some(with_the_header_resealed(
            with_bytes_set(data_unit_and_key(66, payload_of_the_data_units).0, 7, &[1]),
            DATA_UNIT_HEADER_END,
        )),
    );
    // 改的是树 ID 里的一个字节（头校验和罩住它），不重封：存的值与现算的对不上
    place(
        "头校验和的值错",
        2,
        Some(with_bytes_set(
            data_unit_and_key(67, payload_of_the_data_units).0,
            50,
            &[0xEE],
        )),
    );
    place(
        "头校验和字段尾巴非 0",
        2,
        Some(with_bytes_set(
            data_unit_and_key(68, payload_of_the_data_units).0,
            30,
            &[1],
        )),
    );
    place(
        "载荷校验和错",
        2,
        Some(with_bytes_set(
            data_unit_and_key(69, payload_of_the_data_units).0,
            300,
            &[0xEE],
        )),
    );
    place(
        "预留位非 0",
        2,
        Some(data_unit_with_the_payload_resealed(with_bytes_set(
            data_unit_and_key(70, payload_of_the_data_units).0,
            DATA_UNIT_HEADER_END + 3,
            &[1],
        ))),
    );
    place(
        "码 1 类标签副本不对",
        2,
        Some(with_the_header_resealed(
            with_bytes_set(data_unit_and_key(71, payload_of_the_data_units).0, 42, &[9]),
            DATA_UNIT_HEADER_END,
        )),
    );
    place(
        "码 1 声明长度越过单元末尾",
        2,
        Some(with_the_header_resealed(
            with_bytes_set(
                data_unit_and_key(72, payload_of_the_data_units).0,
                8,
                &[0xFF, 0xFF],
            ),
            DATA_UNIT_HEADER_END,
        )),
    );
    place(
        "码 2 声明长度不等于条目数乘条目宽",
        1,
        Some(with_the_header_resealed(
            with_bytes_set(index_node_and_key(73).0, 8, &[1, 0]),
            HEADER_END_OF_THE_TEST_NODES,
        )),
    );
    // 条目数 0、声明长度 0 过得了头一判；条目宽 10 小于 key 宽 24
    place(
        "码 2 条目宽小于 key 宽",
        1,
        Some(with_the_header_resealed(
            with_bytes_set(
                with_bytes_set(
                    with_bytes_set(index_node_and_key(74).0, 8, &[0, 0]),
                    HEADER_END_OF_THE_TEST_NODES - 4,
                    &[0, 0],
                ),
                HEADER_END_OF_THE_TEST_NODES - 2,
                &[10, 0],
            ),
            HEADER_END_OF_THE_TEST_NODES,
        )),
    );
    place(
        "码 3 类标签副本不对",
        2,
        Some(with_the_header_resealed(
            with_bytes_set(packed_unit_and_key(75).0, 42, &[9]),
            PACKED_UNIT_HEADER_END,
        )),
    );
    place(
        "码 3 声明长度不等于记录数乘记录宽",
        2,
        Some(with_the_header_resealed(
            with_bytes_set(packed_unit_and_key(76).0, 8, &[1, 0]),
            PACKED_UNIT_HEADER_END,
        )),
    );
    assert!(
        next_offset_in_slots <= region_under_test().slot_count(),
        "这些记录摆得进一个 R"
    );
    let mapping_entries = vec![
        CentralMappingEntry {
            key: data_key,
            locations: locations_at(slot_of["可搬码 1"]),
        },
        CentralMappingEntry {
            key: node_key.clone(),
            locations: locations_at(slot_of["可搬码 2"]),
        },
        CentralMappingEntry {
            key: packed_key,
            locations: locations_at(slot_of["可搬码 3"]),
        },
        CentralMappingEntry {
            key: node_key.clone(),
            locations: locations_at(slot_of["指着它的是别的 key"]),
        },
        CentralMappingEntry {
            key: index_node_and_key(51).1,
            locations: locations_at(slot_of["两条指着它、一条带别的 key"]),
        },
        CentralMappingEntry {
            key: node_key,
            locations: locations_at(slot_of["两条指着它、一条带别的 key"]),
        },
        CentralMappingEntry {
            key: data_unit_and_key(52, payload_of_the_data_units).1,
            locations: locations_at(SlotNumber(region_under_test().end_slot().0 + 10)),
        },
        CentralMappingEntry {
            key: index_node_and_key(77).1,
            locations: locations_at(slot_of["类标签与条目 key 不符"]),
        },
        CentralMappingEntry {
            key: Vec::new(),
            locations: locations_at(slot_of["指着它的条目 key 为空"]),
        },
    ];
    RegionUnderTest {
        records_and_bytes,
        mapping_entries,
        reference_classification_texts: Vec::new(),
    }
}

#[test]
fn the_walking_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal() {
    region_concluded_on_the_cards_through_the_pairing_library(
        "the_walking_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal",
        PayloadOfTheDataUnits::WithinTheHeadPiece,
        judged_on_the_cards_that_the_walking_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal,
    );
}

/// 持卡判入口（登记在 `src/gpu_side_units_entries.rs`）：开交来的卡，逐条走映射那一份核逐条与读回的 CPU 参照比。
fn judged_on_the_cards_that_the_walking_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal(
    test_name: &str,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let region = region_read_back(input_bytes, reference_bytes);
        let Some(mut background_gpu) =
            background_gpu_on_the_handed_over_gpus_under_one_gibibyte_each_or_report_not_run(
                test_name,
                WAIT_LIMIT_OF_THE_GPU_TESTS,
            )
        else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
        };
        let classifications = background_gpu
            .both_kernel_classifications_of_the_region_records(
                region_under_test(),
                &region.records_to_classify(),
                &region.mapping_entries,
            )
            .expect("健康的卡上一批交得成");
        let walking: Vec<AllocationInRegion> = classifications
            .into_iter()
            .map(|(walking, _searching)| walking)
            .collect();
        assert_eq!(
            debug_texts_of(&walking),
            region.reference_classification_texts,
            "逐条走映射那一份核与 CPU 参照逐条相同"
        );
        ComparisonOnTheCards::EveryComparisonHeld
    })
}

#[test]
fn the_searching_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal() {
    region_concluded_on_the_cards_through_the_pairing_library(
        "the_searching_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal",
        PayloadOfTheDataUnits::WithinTheHeadPiece,
        judged_on_the_cards_that_the_searching_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal,
    );
}

/// 持卡判入口（登记在 `src/gpu_side_units_entries.rs`）：开交来的卡，在索引上二分查的那一份核逐条与读回的 CPU 参照比。
fn judged_on_the_cards_that_the_searching_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal(
    test_name: &str,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let region = region_read_back(input_bytes, reference_bytes);
        let Some(mut background_gpu) =
            background_gpu_on_the_handed_over_gpus_under_one_gibibyte_each_or_report_not_run(
                test_name,
                WAIT_LIMIT_OF_THE_GPU_TESTS,
            )
        else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
        };
        let classifications = background_gpu
            .both_kernel_classifications_of_the_region_records(
                region_under_test(),
                &region.records_to_classify(),
                &region.mapping_entries,
            )
            .expect("健康的卡上一批交得成");
        let searching: Vec<AllocationInRegion> = classifications
            .into_iter()
            .map(|(_walking, searching)| searching)
            .collect();
        assert_eq!(
            debug_texts_of(&searching),
            region.reference_classification_texts,
            "在索引上二分查的那一份核与 CPU 参照逐条相同"
        );
        ComparisonOnTheCards::EveryComparisonHeld
    })
}

#[test]
fn the_two_region_record_kernels_confirm_every_record_on_a_healthy_gpu() {
    region_concluded_on_the_cards_through_the_pairing_library(
        "the_two_region_record_kernels_confirm_every_record_on_a_healthy_gpu",
        PayloadOfTheDataUnits::WithinTheHeadPiece,
        judged_on_the_cards_that_the_two_region_record_kernels_confirm_every_record_on_a_healthy_gpu,
    );
}

/// 持卡判入口（登记在 `src/gpu_side_units_entries.rs`）：开交来的卡，两份核对账逐条全部确认、确认的分类与读回的 CPU 参照比。
fn judged_on_the_cards_that_the_two_region_record_kernels_confirm_every_record_on_a_healthy_gpu(
    test_name: &str,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let region = region_read_back(input_bytes, reference_bytes);
        let Some(mut background_gpu) =
            background_gpu_on_the_handed_over_gpus_under_one_gibibyte_each_or_report_not_run(
                test_name,
                WAIT_LIMIT_OF_THE_GPU_TESTS,
            )
        else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
        };
        let mut uncertainty = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
        let reconciled = background_gpu
            .classify_region_records(
                region_under_test(),
                &region.records_to_classify(),
                &region.mapping_entries,
                &mut uncertainty,
            )
            .expect("健康的卡上一批交得成");
        assert_eq!(
            (
                reconciled.confirmed_entries,
                reconciled.uncertain_entries,
                reconciled.entries_judged_by_the_cpu_reference
            ),
            (region.records_and_bytes.len(), 0, 0),
            "两份核彼此逐条相同：全部确认"
        );
        let confirmed: Vec<AllocationInRegion> = reconciled
            .decisions
            .into_iter()
            .map(|decision| match decision {
                EntryDecision::ConfirmedByBothKernels(classification) => classification,
                EntryDecision::UncertainThisRound | EntryDecision::JudgedByTheCpuReference(_) => {
                    panic!("健康的卡上不该有不确定的记录：{decision:?}")
                }
            })
            .collect();
        assert_eq!(
            debug_texts_of(&confirmed),
            region.reference_classification_texts
        );
        ComparisonOnTheCards::EveryComparisonHeld
    })
}

#[test]
fn the_memory_limit_smaller_than_the_region_batch_cuts_it_into_chunks_with_the_same_classifications(
) {
    region_concluded_on_the_cards_through_the_pairing_library(
        "the_memory_limit_smaller_than_the_region_batch_cuts_it_into_chunks_with_the_same_classifications",
        PayloadOfTheDataUnits::WithinTheHeadPiece,
        judged_on_the_cards_that_the_memory_limit_smaller_than_the_region_batch_cuts_it_into_chunks_with_the_same_classifications,
    );
}

/// 持卡判入口（登记在 `src/gpu_side_units_entries.rs`）：每张交来的卡给 96 KiB 的上限开卡，切成几段交的两份核逐条与读回的 CPU 参照比。
fn judged_on_the_cards_that_the_memory_limit_smaller_than_the_region_batch_cuts_it_into_chunks_with_the_same_classifications(
    test_name: &str,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let region = region_read_back(input_bytes, reference_bytes);
        let Some(HandedOverGpusSeenThroughTheInstance {
            vulkan_instance,
            gpus: handed_over_gpus,
        }) = handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)
        else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
        };
        // 96 KiB：一条 32 KiB 的单元连同整份映射、描述、结果与映射着建的那几份暂存装得下，两条装不下，整批要切成一条一段
        let small_memory_limit = GpuMemoryInBytes(96 * 1024);
        let mut limited = opened_background_gpu(
            &vulkan_instance,
            &selection_over_the_handed_over_gpus(
                &handed_over_gpus,
                small_memory_limit,
                WAIT_LIMIT_OF_THE_GPU_TESTS,
            ),
        );
        let classifications = limited
            .both_kernel_classifications_of_the_region_records(
                region_under_test(),
                &region.records_to_classify(),
                &region.mapping_entries,
            )
            .expect("每条记录单独装得进 96 KiB");
        assert_eq!(
            classifications
                .iter()
                .map(|(walking, searching)| (format!("{walking:?}"), format!("{searching:?}")))
                .collect::<Vec<_>>(),
            region
                .reference_classification_texts
                .iter()
                .map(|text| (text.clone(), text.clone()))
                .collect::<Vec<_>>(),
            "切成几段交，两份核的分类照样逐条与 CPU 参照相同"
        );
        ComparisonOnTheCards::EveryComparisonHeld
    })
}

/// 装不下自适应分片：上限装得下最大那条 32 KiB 的记录连同两条条目的映射、装不下整份映射，映射按落点切成几片逐片交、
/// 每条记录的分类两份各按自己的合法合成，照样逐条与 CPU 参照（整份映射）相同。
#[test]
fn the_memory_limit_smaller_than_the_mapping_cuts_it_into_slices_with_the_same_classifications() {
    region_concluded_on_the_cards_through_the_pairing_library(
        "the_memory_limit_smaller_than_the_mapping_cuts_it_into_slices_with_the_same_classifications",
        PayloadOfTheDataUnits::WithinTheHeadPiece,
        judged_on_the_cards_that_the_memory_limit_smaller_than_the_mapping_cuts_it_into_slices_with_the_same_classifications,
    );
}

/// 持卡判入口（登记在 `src/gpu_side_units_entries.rs`）：上限装得下最大那条记录连同两条条目的映射开卡，映射切成几片交的两份核逐条与读回的 CPU 参照比。
fn judged_on_the_cards_that_the_memory_limit_smaller_than_the_mapping_cuts_it_into_slices_with_the_same_classifications(
    test_name: &str,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let region = region_read_back(input_bytes, reference_bytes);
        let Some(HandedOverGpusSeenThroughTheInstance {
            vulkan_instance,
            gpus: handed_over_gpus,
        }) = handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)
        else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
        };
        let largest_record_bytes = region
            .records_and_bytes
            .iter()
            .filter_map(|(_record, bytes)| bytes.as_ref().map(Vec::len))
            .max()
            .expect("有读到的记录");
        // 两条条目、各两个落点的映射：14 字 × 2、4 字 × 4、1 字 × 2，合 46 字
        let two_entries_of_the_mapping_bytes = 4 * (14 * 2 + 4 * 4 + 2);
        let memory_limit = region_record_chunk_footprint_of_mapping_table_bytes(GpuMemoryInBytes(
            two_entries_of_the_mapping_bytes,
        ))
        .bytes_held_by_a_chunk(
            InputLengthOnTheHostInBytes::of_this_many_bytes(largest_record_bytes),
            EntryCountOfAChunk(1),
        );
        let whole_mapping_bytes =
            MappingTablesForTheKernels::of_the_mapping_entries(&region.mapping_entries)
                .expect("这份映射的 key 都是 27 字节")
                .bytes_on_the_gpu()
                .0;
        assert!(
            whole_mapping_bytes > two_entries_of_the_mapping_bytes,
            "整份映射（{whole_mapping_bytes} 字节）比两条条目的大：要切"
        );
        let mut limited = opened_background_gpu(
            &vulkan_instance,
            &selection_over_the_handed_over_gpus(
                &handed_over_gpus,
                memory_limit,
                WAIT_LIMIT_OF_THE_GPU_TESTS,
            ),
        );
        let classifications = limited
            .both_kernel_classifications_of_the_region_records(
                region_under_test(),
                &region.records_to_classify(),
                &region.mapping_entries,
            )
            .expect("最大那条记录连同一个落点上的条目装得下");
        assert_eq!(
            classifications
                .iter()
                .map(|(walking, searching)| (format!("{walking:?}"), format!("{searching:?}")))
                .collect::<Vec<_>>(),
            region
                .reference_classification_texts
                .iter()
                .map(|text| (text.clone(), text.clone()))
                .collect::<Vec<_>>(),
            "映射切成几片交，两份核的分类照样逐条与 CPU 参照相同"
        );
        ComparisonOnTheCards::EveryComparisonHeld
    })
}

/// 装不下不拒绝、自适应分片，缩小逐次进行：上限 4 KiB，比一个单元（16 KiB 的索引节点、
/// 32 KiB 的两槽单元）小、装得下参与的卡至少要装得下的那一片：比头一片长的单元都切成头一片与之后的几片，两份核先各自逐片接着算载荷 CRC 的寄存器、
/// 再照头一片解头、查映射、合出整个载荷的 CRC，每条记录两份的分类各自逐条与 CPU 参照（整个单元、整份映射）相同——各类、各种拒法都在里面。
/// 码 1 单元的载荷两万字节、字节各不相同（[`PayloadOfTheDataUnits::RunningPastTheHeadPiece`]）：头一片之后的寄存器不是 0，
/// 哪一份核接错带进来的寄存器、或头一片不把之后那几字节合进来，可搬的码 1 单元就被那一份判成载荷校验和不对。
#[test]
fn the_memory_limit_smaller_than_a_unit_cuts_each_unit_into_pieces_with_the_same_classifications() {
    region_concluded_on_the_cards_through_the_pairing_library(
        "the_memory_limit_smaller_than_a_unit_cuts_each_unit_into_pieces_with_the_same_classifications",
        PayloadOfTheDataUnits::RunningPastTheHeadPiece,
        judged_on_the_cards_that_the_memory_limit_smaller_than_a_unit_cuts_each_unit_into_pieces_with_the_same_classifications,
    );
}

/// 持卡判入口（登记在 `src/gpu_side_units_entries.rs`）：每张交来的卡给 4 KiB 的上限开卡，单元切成几片交的两份核逐条与读回的 CPU 参照比。
fn judged_on_the_cards_that_the_memory_limit_smaller_than_a_unit_cuts_each_unit_into_pieces_with_the_same_classifications(
    test_name: &str,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let region = region_read_back(input_bytes, reference_bytes);
        let Some(HandedOverGpusSeenThroughTheInstance {
            vulkan_instance,
            gpus: handed_over_gpus,
        }) = handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)
        else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
        };
        let smaller_than_a_unit = GpuMemoryInBytes(4 * 1024);
        assert!(
            smaller_than_a_unit >= bytes_of_the_smallest_piece_every_participating_gpu_holds()
                && smaller_than_a_unit.0 < 16 * 1024,
            "上限装得下参与的卡至少要装得下的那一片、装不下一个单元"
        );
        // 4 KiB 装得下最小的一片：不拒挂载
        let mut limited = opened_background_gpu(
            &vulkan_instance,
            &selection_over_the_handed_over_gpus(
                &handed_over_gpus,
                smaller_than_a_unit,
                WAIT_LIMIT_OF_THE_GPU_TESTS,
            ),
        );
        assert_eq!(
            limited.participating_gpus().len(),
            handed_over_gpus.len(),
            "交来的每张卡都参与"
        );
        let reference = region.reference_classification_texts.clone();
        let head_piece_bytes = usize::try_from(UNIT_HEAD_PIECE_INPUT_BYTES.0).expect("628");
        assert!(
            region.records_and_bytes.iter().zip(&reference).any(
                |((_record, bytes), classification)| {
                    classification.starts_with("Movable(")
                        && bytes.as_ref().is_some_and(|bytes| {
                            bytes.len() > head_piece_bytes
                                && bytes[head_piece_bytes..].iter().any(|byte| *byte != 0)
                        })
                }
            ),
            "有一个可搬的单元在头一片之后还有不是 0 的字节：寄存器接没接上、合没合进头一片看得出来"
        );
        let classifications = limited
            .both_kernel_classifications_of_the_region_records(
                region_under_test(),
                &region.records_to_classify(),
                &region.mapping_entries,
            )
            .expect("单元切开之后头一片连同一个落点上的条目、之后的每一片都装得下");
        let (walking, searching): (Vec<AllocationInRegion>, Vec<AllocationInRegion>) =
            classifications.into_iter().unzip();
        assert_eq!(
            debug_texts_of(&walking),
            reference,
            "单元切成几片交，逐条走映射那一份核与 CPU 参照逐条相同"
        );
        assert_eq!(
            debug_texts_of(&searching),
            reference,
            "单元切成几片交，在索引上二分查的那一份核与 CPU 参照逐条相同"
        );
        ComparisonOnTheCards::EveryComparisonHeld
    })
}
