//! logged_ops 树的整理意图条目（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1）：
//! 每个字段的偏移与宽度逐字节钉住、读回逐字段相等；key 按逐字段无符号整数比；一节点条数按 core 的节点容量式子现算；
//! 读者的拒收各一条用例（不对齐的 R、槽数不是 64、补零非零、保留的种类码、条目宽不是 44）。
//! 期望的字节按判决 G1 的字段表手写，不从被测的写者反推。

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::logged_operations::{
    CompactionIntent, CompactionRegion, CompactionRegionRefusal, LoggedOperation,
    LoggedOperationEntry, LoggedOperationEntryRefusal, LoggedOperationKey, LoggedOperationKind,
    LOGGED_OPERATION_KIND_COMPACTION,
};
use singlefs_core::unit::index_node_entry_capacity;
use singlefs_format::{
    LOGGED_OPERATIONS_ENTRIES_PER_NODE, LOGGED_OPERATIONS_ENTRY_BYTES, LOGGED_OPERATIONS_KEY_BYTES,
};

/// R 取单元区里起点 50304 的聚簇段：50176 + 128 = 50304 = 786 × 64。
const REGION_START_SLOT: u64 = 50304;

fn compaction_entry() -> LoggedOperationEntry {
    LoggedOperationEntry {
        issued_txg: CheckpointTxg(0x0102_0304_0506_0708),
        operation: LoggedOperation::Compaction(CompactionIntent {
            region: CompactionRegion::of_one_cluster_segment(
                DeviceIdentity(0x0A0B_0C0D),
                SlotNumber(REGION_START_SLOT),
            )
            .expect("50304 是 64 的倍数"),
            allocation_generation_at_issue: CheckpointTxg(0x1112_1314_1516_1718),
            empty_cluster_segments_at_issue: 0x2122_2324_2526_2728,
        }),
    }
}

/// 判决 G1 的字段表逐段：种类 2 | 签发 txg 8 | R 的设备 4 | R 的起始槽 6 | R 的槽数 2 | c0 8 | segs0 8 | 补零 6，小端，共 44。
#[test]
fn every_field_of_a_compaction_intent_sits_at_its_offset_and_reads_back() {
    let entry = compaction_entry();
    let bytes = entry.to_bytes();
    let start_slot_bytes = REGION_START_SLOT.to_le_bytes();
    let expected = [
        vec![0x01, 0x00],
        vec![0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01],
        vec![0x0D, 0x0C, 0x0B, 0x0A],
        start_slot_bytes[..6].to_vec(),
        vec![64, 0],
        vec![0x18, 0x17, 0x16, 0x15, 0x14, 0x13, 0x12, 0x11],
        vec![0x28, 0x27, 0x26, 0x25, 0x24, 0x23, 0x22, 0x21],
        vec![0; 6],
    ]
    .concat();
    assert_eq!(
        bytes, expected,
        "种类 0 / 签发 txg 2 / 设备 10 / 起始槽 14 / 槽数 20 / c0 22 / segs0 30 / 补零 38"
    );
    assert_eq!(
        u64::try_from(bytes.len()).expect("44"),
        LOGGED_OPERATIONS_ENTRY_BYTES
    );
    assert_eq!(
        LoggedOperationEntry::parse(&bytes),
        Ok(entry),
        "读回来逐字段相等"
    );
    assert_eq!(
        entry.key(),
        LoggedOperationKey {
            kind: LoggedOperationKind::Compaction,
            issued_txg: CheckpointTxg(0x0102_0304_0506_0708),
        },
        "key = 前 10 字节的两段"
    );
    assert_eq!(
        u64::try_from(bytes[..10].len()).expect("10"),
        LOGGED_OPERATIONS_KEY_BYTES
    );
}

/// key 的全序按逐字段无符号整数比（D8（核心索引结构） 已定项 11）：签发 txg 1 在 256 之前，而两条条目的 key 字节按 memcmp 比是反的。
#[test]
fn logged_operation_keys_order_by_field_not_by_bytes() {
    let entry_issued_at = |issued_txg: u64| LoggedOperationEntry {
        issued_txg: CheckpointTxg(issued_txg),
        operation: compaction_entry().operation,
    };
    let earlier = entry_issued_at(1);
    let later = entry_issued_at(256);
    assert!(
        earlier.key().sort_key() < later.key().sort_key(),
        "1 < 256 按整数比"
    );
    assert!(
        earlier.to_bytes()[..10] > later.to_bytes()[..10],
        "小端字节按 memcmp 比是反的，所以不能用它"
    );
}

/// 一节点 369 条：core 的 `index_node_entry_capacity` 在 key 宽 10、条目宽 44 上现算，与格式常量一个数不差。
#[test]
fn a_logged_operations_node_holds_the_entries_the_node_capacity_formula_gives() {
    let computed = index_node_entry_capacity(
        usize::try_from(LOGGED_OPERATIONS_KEY_BYTES).expect("10"),
        usize::try_from(LOGGED_OPERATIONS_ENTRY_BYTES).expect("44"),
    );
    assert_eq!(
        u64::try_from(computed).expect("条数"),
        LOGGED_OPERATIONS_ENTRIES_PER_NODE
    );
    assert_eq!(computed, 369, "(16384 − 135) ÷ 44 的整数部分");
}

fn bytes_with(offset: usize, replacement: &[u8]) -> Vec<u8> {
    let mut bytes = compaction_entry().to_bytes();
    bytes[offset..offset + replacement.len()].copy_from_slice(replacement);
    bytes
}

/// R 的起始槽不是 64 的倍数：读者拒收（判决 G1「R 的解码拒不对齐的值」），写者那一侧也造不出这样的 R。
#[test]
fn a_compaction_region_whose_start_slot_is_off_a_cluster_segment_boundary_is_refused() {
    let unaligned_start_slot = REGION_START_SLOT + 2;
    assert_eq!(
        LoggedOperationEntry::parse(&bytes_with(14, &unaligned_start_slot.to_le_bytes()[..6])),
        Err(LoggedOperationEntryRefusal::CompactionRegionRefused(
            CompactionRegionRefusal::StartOffAClusterSegmentBoundary {
                start_slot: SlotNumber(unaligned_start_slot),
            }
        ))
    );
    assert_eq!(
        CompactionRegion::of_one_cluster_segment(
            DeviceIdentity(0),
            SlotNumber(unaligned_start_slot)
        ),
        Err(CompactionRegionRefusal::StartOffAClusterSegmentBoundary {
            start_slot: SlotNumber(unaligned_start_slot),
        })
    );
}

/// R 的槽数不是 64：读者拒收。
#[test]
fn a_compaction_region_whose_slot_count_is_not_one_cluster_segment_is_refused() {
    assert_eq!(
        LoggedOperationEntry::parse(&bytes_with(20, &128u16.to_le_bytes())),
        Err(
            LoggedOperationEntryRefusal::CompactionRegionSlotCountIsNotOneClusterSegment {
                slot_count: 128
            }
        )
    );
}

/// 补零那 6 字节里有非零字节：读者拒收（照树表条目预留位「非零判不可用」的政策）。
#[test]
fn a_compaction_intent_with_a_nonzero_padding_byte_is_refused() {
    assert_eq!(
        LoggedOperationEntry::parse(&bytes_with(43, &[1])),
        Err(LoggedOperationEntryRefusal::CompactionPaddingNotZero)
    );
}

/// 种类码不是 1（整理）也不是 2（截断，判决 m4-trunc-r1 K1）：0 与 3 都是保留码，读者拒收（照树表 flags「未知位一律拒收」）。
#[test]
fn a_logged_operation_of_a_reserved_kind_is_refused() {
    assert_eq!(LOGGED_OPERATION_KIND_COMPACTION, 1);
    for reserved_kind_code in [0u16, 3] {
        assert_eq!(
            LoggedOperationEntry::parse(&bytes_with(0, &reserved_kind_code.to_le_bytes())),
            Err(LoggedOperationEntryRefusal::UnknownKind {
                kind_code: reserved_kind_code
            }),
            "种类码 {reserved_kind_code} 是保留码"
        );
    }
}

/// 条目宽（码 2 节点头里的盘上字段）不是 44：窄一字节、宽一字节都拒收，不按固定偏移切下去。
#[test]
fn a_logged_operation_entry_whose_width_is_not_the_field_table_is_refused() {
    let bytes = compaction_entry().to_bytes();
    assert_eq!(
        LoggedOperationEntry::parse(&bytes[..43]),
        Err(LoggedOperationEntryRefusal::WidthIsNotTheFieldTable { entry_bytes: 43 })
    );
    assert_eq!(
        LoggedOperationEntry::parse(&[bytes.as_slice(), &[0]].concat()),
        Err(LoggedOperationEntryRefusal::WidthIsNotTheFieldTable { entry_bytes: 45 })
    );
}
