//! 区域扫描里单元头解析与映射反查那两份 WGSL 核在真显卡上与 CPU 参照（核心层的区域扫描，`cpu_reference_classification`）对拍、
//! 两份核彼此对拍（D26（后台整理与放置回收） 已定项 13「GPU 路径与 CPU 路径是两份实现……两份实现在测试里对拍」）。
//! 要本机至少一张独立显卡：一张都没有时每条打一行 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」、返回成功；
//! 有卡而开不了判红。只在开了特性 `tests-on-a-real-gpu` 时编；用 GPU，经 `scripts/crash/gpu-quiesce-run.sh` 起，带 `--test-threads 1`。
//!
//! 一个 R 里放齐各类：三类可搬（码 1、码 2、码 3）、已死、类标签不符（含指着它的条目 key 为空）、key 与位置不符（两种）、读不到、跨度不对、
//! 字节数不对（排在前面，让后面的单元起点不在字边界上），以及核心层三个 parse 的每一种拒法里造得出的：magic、版本、flags、类标签、
//! 头校验和（值错与字段尾巴非 0 两种）、载荷校验和、预留位、结构字段（码 1 两种、码 2 两种、码 3 两种）。

use std::collections::BTreeMap;

use singlefs_background_gpu::adapter_inventory::BackgroundGpuSelection;
use singlefs_background_gpu::background_gpu::{
    open_background_gpu, BackgroundGpu, BackgroundGpuOpenRefused,
};
use singlefs_background_gpu::region_record_kernels::{
    cpu_reference_classification, RegionRecordToClassify,
};
use singlefs_background_gpu::two_kernel_reconciliation::{
    ConsecutiveUncertaintyCounts, EntryDecision,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
};
use singlefs_core::allocator::{AllocationRecord, AllocationRecordQuarantineMark};
use singlefs_core::background_compute::{BackgroundComputeSettingsRefused, GpuMemoryInBytes};
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

fn background_gpu_or_report_not_run(
    test_name: &str,
    selection: &BackgroundGpuSelection,
) -> Option<BackgroundGpu> {
    match open_background_gpu(selection) {
        Ok(background_gpu) => Some(background_gpu),
        Err(BackgroundGpuOpenRefused::SettingsRefused(
            BackgroundComputeSettingsRefused::GpuNotPresent,
        )) => {
            println!(
                "NOT_RUN_NO_GPU_ADAPTER test={test_name} reason=本机的 Vulkan 适配器里一张独立显卡都没有"
            );
            None
        }
        Err(refused) => panic!("本机有独立显卡而开不了后台 GPU：{refused:?}"),
    }
}

fn every_discrete_gpu_without_a_memory_limit() -> BackgroundGpuSelection {
    BackgroundGpuSelection {
        background_gpu_devices: None,
        background_gpu_memory_limit: None,
        background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
    }
}

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

fn region_with_every_class_and_every_parser_refusal() -> RegionUnderTest {
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
    let (data_bytes, data_key) = data_unit_and_key(4);
    let (node_bytes, node_key) = index_node_and_key(5);
    let (packed_bytes, packed_key) = packed_unit_and_key(6);
    // 字节数不对的排在最前：后面每个单元在输入里的起点都差 3 个字节，不在字边界上
    place("字节数不对", 2, Some(data_unit_and_key(1).0[..3].to_vec()));
    place("可搬码 1", 2, Some(data_bytes.clone()));
    place("可搬码 2", 1, Some(node_bytes.clone()));
    place("可搬码 3", 2, Some(packed_bytes.clone()));
    place("已死", 2, Some(data_unit_and_key(40).0));
    place("指着它的是别的 key", 1, Some(index_node_and_key(50).0));
    place(
        "两条指着它、一条带别的 key",
        1,
        Some(index_node_and_key(51).0),
    );
    place(
        "头里的 key 在映射里、指着它的没有",
        2,
        Some(data_unit_and_key(52).0),
    );
    place("类标签与条目 key 不符", 2, Some(data_unit_and_key(60).0));
    place("指着它的条目 key 为空", 2, Some(data_unit_and_key(61).0));
    place("读不到", 2, None);
    place("跨度不对", 3, Some(data_unit_and_key(62).0));
    place(
        "magic 坏",
        2,
        Some(with_bytes_set(data_unit_and_key(63).0, 0, b"XFSU")),
    );
    place(
        "版本不认得",
        2,
        Some(with_the_header_resealed(
            with_bytes_set(data_unit_and_key(64).0, 4, &[2, 0]),
            DATA_UNIT_HEADER_END,
        )),
    );
    place(
        "类标签不对",
        2,
        Some(with_bytes_set(
            data_unit_and_key(65).0,
            6,
            &[UNIT_CLASS_INDEX_NODE],
        )),
    );
    place(
        "flags 非 0",
        2,
        Some(with_the_header_resealed(
            with_bytes_set(data_unit_and_key(66).0, 7, &[1]),
            DATA_UNIT_HEADER_END,
        )),
    );
    // 改的是树 ID 里的一个字节（头校验和罩住它），不重封：存的值与现算的对不上
    place(
        "头校验和的值错",
        2,
        Some(with_bytes_set(data_unit_and_key(67).0, 50, &[0xEE])),
    );
    place(
        "头校验和字段尾巴非 0",
        2,
        Some(with_bytes_set(data_unit_and_key(68).0, 30, &[1])),
    );
    place(
        "载荷校验和错",
        2,
        Some(with_bytes_set(data_unit_and_key(69).0, 300, &[0xEE])),
    );
    place(
        "预留位非 0",
        2,
        Some(data_unit_with_the_payload_resealed(with_bytes_set(
            data_unit_and_key(70).0,
            DATA_UNIT_HEADER_END + 3,
            &[1],
        ))),
    );
    place(
        "码 1 类标签副本不对",
        2,
        Some(with_the_header_resealed(
            with_bytes_set(data_unit_and_key(71).0, 42, &[9]),
            DATA_UNIT_HEADER_END,
        )),
    );
    place(
        "码 1 声明长度越过单元末尾",
        2,
        Some(with_the_header_resealed(
            with_bytes_set(data_unit_and_key(72).0, 8, &[0xFF, 0xFF]),
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
            key: data_unit_and_key(52).1,
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
    }
}

#[test]
fn the_walking_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal() {
    let Some(mut background_gpu) = background_gpu_or_report_not_run(
        "the_walking_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal",
        &every_discrete_gpu_without_a_memory_limit(),
    ) else {
        return;
    };
    let region = region_with_every_class_and_every_parser_refusal();
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
        walking,
        region.cpu_reference_classifications(),
        "逐条走映射那一份核与 CPU 参照逐条相同"
    );
}

#[test]
fn the_searching_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal() {
    let Some(mut background_gpu) = background_gpu_or_report_not_run(
        "the_searching_kernel_matches_the_cpu_reference_on_every_class_and_parser_refusal",
        &every_discrete_gpu_without_a_memory_limit(),
    ) else {
        return;
    };
    let region = region_with_every_class_and_every_parser_refusal();
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
        searching,
        region.cpu_reference_classifications(),
        "在索引上二分查的那一份核与 CPU 参照逐条相同"
    );
}

#[test]
fn the_two_region_record_kernels_confirm_every_record_on_a_healthy_gpu() {
    let Some(mut background_gpu) = background_gpu_or_report_not_run(
        "the_two_region_record_kernels_confirm_every_record_on_a_healthy_gpu",
        &every_discrete_gpu_without_a_memory_limit(),
    ) else {
        return;
    };
    let region = region_with_every_class_and_every_parser_refusal();
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
    assert_eq!(confirmed, region.cpu_reference_classifications());
}

#[test]
fn a_memory_limit_smaller_than_the_region_batch_cuts_it_into_chunks_with_the_same_classifications()
{
    let test_name =
        "a_memory_limit_smaller_than_the_region_batch_cuts_it_into_chunks_with_the_same_classifications";
    let Some(unlimited) =
        background_gpu_or_report_not_run(test_name, &every_discrete_gpu_without_a_memory_limit())
    else {
        return;
    };
    let participating_gpus = unlimited.participating_gpus().len();
    drop(unlimited);
    // 48 KiB：一条 32 KiB 的单元连同整份映射、描述与结果装得下，两条装不下，整批要切成一条一段
    let small_memory_limit = GpuMemoryInBytes(48 * 1024);
    let mut limited = background_gpu_or_report_not_run(
        test_name,
        &BackgroundGpuSelection {
            background_gpu_devices: None,
            background_gpu_memory_limit: Some(vec![small_memory_limit; participating_gpus]),
            background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
        },
    )
    .expect("同一台机器上刚开过一次");
    let region = region_with_every_class_and_every_parser_refusal();
    let classifications = limited
        .both_kernel_classifications_of_the_region_records(
            region_under_test(),
            &region.records_to_classify(),
            &region.mapping_entries,
        )
        .expect("每条记录单独装得进 48 KiB");
    assert_eq!(
        classifications,
        region
            .cpu_reference_classifications()
            .into_iter()
            .map(|classification| (classification.clone(), classification))
            .collect::<Vec<_>>(),
        "切成几段交，两份核的分类照样逐条与 CPU 参照相同"
    );
}
