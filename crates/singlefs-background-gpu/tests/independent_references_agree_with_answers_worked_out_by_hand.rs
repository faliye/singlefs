//! 三段对拍的独立参照（`common_independent_references`）在已知输入上钉绝对值：答案不出自产品与核心层，是另算出来的
//! （CRC-32C 取公开的校验值，活槽数与映射 key 按定义手算），三方比那一侧本身先站得住（`.claude/rules/test-discipline.md`
//! 「只让多条臂互相比，测不出「所有臂一起错」」）。单元的字节由核心层的写者造（造输入用，不是判对错的那一方）。
//! 不要卡、不挂特性，`cargo test -p singlefs-background-gpu --test <本目标>` 就能跑。
mod common_independent_references;

use common_independent_references::{
    crc32c_by_the_most_significant_bit_first_table,
    live_slots_per_full_segment_by_coverage_changes, region_record_class_worked_out_independently,
    AllocationRecordWords, MappingEntryWords, RegionRecordClassWorkedOutIndependently,
    UnitAreaWords,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration, TreeIdentifier};
use singlefs_core::pointer::BirthSequence;
use singlefs_core::unit::{
    build_data_unit, build_index_node, build_packed_unit, DataUnitIdentity, PackedIdentity,
    WriteOrder, PACKED_TYPE_INODE,
};

/// 场景：CRC-32C 的公开校验值——ASCII「123456789」，与 RFC 3720（iSCSI）附录 B.4 的四段 32 字节（全 0、全 0xFF、0 到 31 递增、31 到 0 递减），
/// 再加空串。
/// 预期：依次是 0xE3069283、0x8A9136AA、0x62A8AB43、0x46DD794E、0x113FDB5C、0。
/// 判别力（`crates/mutations.tsv`）：「独立参照 CRC-32C 多项式低位写错」那一行让这一条红。
#[test]
fn the_independent_crc32c_gives_the_published_check_values() {
    let ascending: Vec<u8> = (0..32).collect();
    let descending: Vec<u8> = (0..32).rev().collect();
    let inputs: [&[u8]; 6] = [
        b"123456789",
        &[0_u8; 32],
        &[0xFF_u8; 32],
        &ascending,
        &descending,
        &[],
    ];
    let worked_out: Vec<u32> = inputs
        .iter()
        .copied()
        .map(crc32c_by_the_most_significant_bit_first_table)
        .collect();
    assert_eq!(
        worked_out,
        vec![
            0xE306_9283,
            0x8A91_36AA,
            0x62A8_AB43,
            0x46DD_794E,
            0x113F_DB5C,
            0
        ],
        "公开的校验值"
    );
}

/// 场景：单元区起点槽 128、三个整段加末尾 7 个槽（段宽 64）。盘 0 上的记录：起点 129 跨 2（头一段）、起点 195 跨 1（中间一段）、
/// 起点 255 跨 2（中间一段的末一槽与末一段的头一槽）、起点 125 跨 5（单元区之前起、罩进头一段 2 个槽）、起点 300 跨 20
/// （末一段 [256, 320) 罩 20 个槽）、起点 318 跨 5（末一段罩 2 个，末尾凑不满一段的 3 个不算）、起点 140 跨 4 已释放；
/// 盘 1 上起点 130 跨 3。
/// 预期：每段的活槽数手算是 2 + 2、1 + 1、1 + 20 + 2，即 [4, 2, 23]。
/// 判别力（`crates/mutations.tsv`）：「独立参照每段活槽数也数释放了的记录」那一行让这一条红。
#[test]
fn the_independent_live_slot_count_gives_the_counts_worked_out_by_hand() {
    let record =
        |device: u32, first_slot: u64, span_slots: u64, is_released: bool| AllocationRecordWords {
            device,
            first_slot,
            span_slots,
            is_released,
        };
    let records = [
        record(0, 129, 2, false),
        record(0, 195, 1, false),
        record(0, 255, 2, false),
        record(0, 125, 5, false),
        record(0, 300, 20, false),
        record(0, 318, 5, false),
        record(0, 140, 4, true),
        record(1, 130, 3, false),
    ];
    assert_eq!(
        live_slots_per_full_segment_by_coverage_changes(
            &records,
            UnitAreaWords {
                device: 0,
                first_slot: 128,
                slot_count: 3 * 64 + 7,
            }
        ),
        vec![4, 2, 23],
        "按段手算的活槽数"
    );
}

/// 测试造单元用的几个字段。
const TREE_OF_THE_UNITS: u64 = 11;
const BIRTH_TXG_OF_THE_UNITS: u64 = (3 << 32) | 7;
const WRITING_INSTANCE_OF_THE_UNITS: u32 = 2;
const FILESYSTEM_IDENTIFIER_OF_THE_UNITS: [u8; 16] = [7; 16];
/// 记录落在盘 0 的这个槽上；中央映射里指着它的条目两个落点是盘 0 与盘 1 的同一个槽。
const SLOT_OF_THE_RECORD: u64 = (1 << 33) + 64 * 900;

/// 码 1 数据单元：事务号 4、载荷 100 个 0x5A。
fn data_unit_of_transaction_four() -> Vec<u8> {
    build_data_unit(
        DataUnitIdentity {
            tree: TreeIdentifier(TREE_OF_THE_UNITS),
            object: 1,
            object_birth: CheckpointTxg(3),
            anchor_offset: 0,
        },
        CheckpointTxg(BIRTH_TXG_OF_THE_UNITS),
        &FILESYSTEM_IDENTIFIER_OF_THE_UNITS,
        WriteOrder {
            instance: InstanceGeneration(WRITING_INSTANCE_OF_THE_UNITS),
            transaction: 4,
        },
        &[0x5A; 100],
    )
}

/// 手写的映射 key：类标签、树号 8、诞生代号 8（都是小端）接 `tail`。
fn mapping_key_written_by_hand(unit_class_tag: u8, tail: &[u8]) -> Vec<u8> {
    let mut key = vec![unit_class_tag];
    key.extend_from_slice(&TREE_OF_THE_UNITS.to_le_bytes());
    key.extend_from_slice(&BIRTH_TXG_OF_THE_UNITS.to_le_bytes());
    key.extend_from_slice(tail);
    key
}

/// 一条指着记录那个槽（盘 0 与盘 1）的条目。
fn entry_pointing_at_the_record(key: &[u8]) -> MappingEntryWords<'_> {
    MappingEntryWords {
        key,
        locations: [(0, SLOT_OF_THE_RECORD), (1, SLOT_OF_THE_RECORD)],
    }
}

/// 场景：码 1、码 2、码 3 各一个单元落在记录那个槽上（跨度 2、1、2），中央映射里一条条目指着它、带手写的那把 key。
/// 预期：三条都收：映射 key 是手写的那一把（码 1 = 类标签、树号、诞生代号、写序 10；码 2 与码 3 = 类标签、树号、诞生代号、实例代号、
/// 出生序号、补两个 0），类标签 1、2、3，诞生代号 (3 << 32) | 7。
/// 判别力（`crates/mutations.tsv`）：「独立参照分类码 1 的 key 漏掉事务号」那一行让这一条红。
#[test]
fn every_unit_class_is_movable_with_the_mapping_key_written_by_hand() {
    let mut data_tail = WRITING_INSTANCE_OF_THE_UNITS.to_le_bytes().to_vec();
    data_tail.extend_from_slice(&[4, 0, 0, 0, 0, 0]);
    let mut node_tail = WRITING_INSTANCE_OF_THE_UNITS.to_le_bytes().to_vec();
    node_tail.extend_from_slice(&[5, 0, 0, 0, 0, 0]);
    let mut packed_tail = WRITING_INSTANCE_OF_THE_UNITS.to_le_bytes().to_vec();
    packed_tail.extend_from_slice(&[6, 0, 0, 0, 0, 0]);
    let index_node = build_index_node(
        TreeIdentifier(TREE_OF_THE_UNITS),
        0,
        24,
        &[0; 24],
        &[0; 24],
        CheckpointTxg(BIRTH_TXG_OF_THE_UNITS),
        &FILESYSTEM_IDENTIFIER_OF_THE_UNITS,
        InstanceGeneration(WRITING_INSTANCE_OF_THE_UNITS),
        BirthSequence(5),
        112,
        &[vec![3; 112], vec![4; 112]],
    );
    let packed_unit = build_packed_unit(
        PackedIdentity {
            birth_tree: TreeIdentifier(TREE_OF_THE_UNITS),
            record_type: PACKED_TYPE_INODE,
            container: 1,
            container_birth: CheckpointTxg(BIRTH_TXG_OF_THE_UNITS),
        },
        140,
        &[vec![1; 140], vec![2; 140]],
        CheckpointTxg(BIRTH_TXG_OF_THE_UNITS),
        &FILESYSTEM_IDENTIFIER_OF_THE_UNITS,
        WriteOrder {
            instance: InstanceGeneration(WRITING_INSTANCE_OF_THE_UNITS),
            transaction: 9,
        },
        BirthSequence(6),
    );
    for (unit_class_tag, span_slots, unit_bytes, tail) in [
        (1_u8, 2_u64, data_unit_of_transaction_four(), data_tail),
        (2, 1, index_node, node_tail),
        (3, 2, packed_unit, packed_tail),
    ] {
        let key = mapping_key_written_by_hand(unit_class_tag, &tail);
        assert_eq!(
            u64::try_from(key.len()).expect("key 的长度装得进 u64"),
            singlefs_format::MAPPING_KEY_BYTES,
            "手写的 key 是映射 key 那么长"
        );
        assert_eq!(
            region_record_class_worked_out_independently(
                0,
                SLOT_OF_THE_RECORD,
                span_slots,
                Some(unit_bytes.as_slice()),
                &[entry_pointing_at_the_record(&key)],
            ),
            RegionRecordClassWorkedOutIndependently::Movable {
                mapping_key: key.clone(),
                unit_class_tag,
                birth_txg: BIRTH_TXG_OF_THE_UNITS,
            },
            "码 {unit_class_tag} 收下，key 是手写的那一把"
        );
    }
}

/// 场景：同一个码 1 单元落在记录那个槽上，三种映射与字节：映射是空的；载荷里改一个字节（不重封载荷校验和）、映射里有条目指着它；
/// 跨度写成 3。
/// 预期：依次已死、读不出头、读不出头。
/// 判别力（`crates/mutations.tsv`）：「独立参照分类不核载荷校验和」那一行让这一条红。
#[test]
fn an_unmapped_unit_is_dead_and_a_damaged_or_mis_spanned_unit_is_unreadable() {
    let unit_bytes = data_unit_of_transaction_four();
    let mut damaged = unit_bytes.clone();
    damaged[300] ^= 0xEE;
    let mut tail = WRITING_INSTANCE_OF_THE_UNITS.to_le_bytes().to_vec();
    tail.extend_from_slice(&[4, 0, 0, 0, 0, 0]);
    let key = mapping_key_written_by_hand(1, &tail);
    let mapping = [entry_pointing_at_the_record(&key)];
    assert_eq!(
        [
            region_record_class_worked_out_independently(
                0,
                SLOT_OF_THE_RECORD,
                2,
                Some(unit_bytes.as_slice()),
                &[]
            ),
            region_record_class_worked_out_independently(
                0,
                SLOT_OF_THE_RECORD,
                2,
                Some(damaged.as_slice()),
                &mapping
            ),
            region_record_class_worked_out_independently(
                0,
                SLOT_OF_THE_RECORD,
                3,
                Some(unit_bytes.as_slice()),
                &mapping
            ),
        ],
        [
            RegionRecordClassWorkedOutIndependently::Dead,
            RegionRecordClassWorkedOutIndependently::HeaderUnreadable,
            RegionRecordClassWorkedOutIndependently::HeaderUnreadable,
        ],
        "映射空的已死；载荷坏了、跨度不对的读不出头"
    );
}
