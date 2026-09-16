//! C245 第二轮攻方腿的探针：拿提交 fbae43e 的 `recovery::replay_journal` 原样跑几种盘面，记录按真字节编码再解析回来。
//! 每条记录的 `new_tree_identifier_watermark` 当标签用（1000 × 实例 + 计数器），重建出的根带着最后施加的那条的标签。
//! 不验点名单元（verify_named_units = false）：这几个形状判的是链首与链尾，单元验证单独不参与。

use std::collections::BTreeMap;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::address::{SlotNumber, TreeIdentifier};
use singlefs_core::journal::{back_chain_of, JournalRecord, NamedUnit};
use singlefs_core::pointer::{LocationEntry, NodePointer};
use singlefs_core::recovery::{replay_journal, PoolReader};
use singlefs_core::root_record::RootRecord;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;

struct NoDevices;

impl PoolReader for NoDevices {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        Vec::new()
    }
    fn read(&self, _device: DeviceIdentity, _offset: DeviceOffsetInBytes, _length: usize) -> Option<Vec<u8>> {
        None
    }
    fn journal_record_offsets_hint(
        &self,
        _device: DeviceIdentity,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        None
    }
}

const FILESYSTEM_IDENTIFIER_LOW: u64 = 7;

/// (实例, 计数器, txg, 事务号, 提交标记, 反向链取谁：None = 本实例第一条 0，Some(计数器) = 同实例那条记录的头)
type RecordSpec = (u32, u64, u64, u64, bool, Option<u64>);

fn record_from(spec: RecordSpec, written: &BTreeMap<(u32, u64), Vec<u8>>) -> Vec<u8> {
    let (instance, counter, txg, transaction, is_commit, back_of) = spec;
    let back_chain = back_of.map_or(0, |previous| back_chain_of(&written[&(instance, previous)]));
    JournalRecord {
        instance: InstanceGeneration(instance),
        counter,
        checkpoint_txg: CheckpointTxg(txg),
        transaction,
        is_commit,
        back_chain,
        filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW,
        new_tree_table: NodePointer::empty_root(),
        new_mapping_root: NodePointer::empty_root(),
        new_tree_identifier_watermark: 1000 * u64::from(instance) + counter,
        new_rollback_floor: CheckpointTxg(0),
        named: Vec::new(),
    }
    .to_bytes()
}

fn root(instance: u32, txg: u64) -> RootRecord {
    RootRecord {
        filesystem_identifier: [0; 16],
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(txg),
        tree_table: NodePointer::empty_root(),
        tree_identifier_watermark: 1000 * u64::from(instance),
        rollback_floor: CheckpointTxg(0),
        instance_table: NodePointer::empty_root(),
        mapping_root: NodePointer::empty_root(),
    }
}

/// written：盘上曾写出的全部记录（算反向链用）；on_disk：崩溃后读得出的那几条的 (实例, 计数器)。
fn probe(shape: &str, truth: &str, chosen: (u32, u64), written_specs: &[RecordSpec], on_disk: &[(u32, u64)]) {
    let mut written = BTreeMap::new();
    for spec in written_specs {
        let bytes = record_from(*spec, &written);
        written.insert((spec.0, spec.1), bytes);
    }
    let mut records = BTreeMap::new();
    for key in on_disk {
        let parsed = JournalRecord::parse(&written[key], FILESYSTEM_IDENTIFIER_LOW).expect("自己写的记录解析得回来");
        records.insert((parsed.instance, parsed.counter), parsed);
    }
    let chosen_root = root(chosen.0, chosen.1);
    let (report, rebuilt) = replay_journal(&NoDevices, &chosen_root, JOURNAL_RING_DEFAULT_BYTES, &records, false);
    let tag = rebuilt.tree_identifier_watermark;
    let applied_last = if report.prefix_applied == 0 {
        "none".to_string()
    } else {
        format!("({},{})", tag / 1000, tag % 1000)
    };
    println!(
        "C245R2-IMPL shape={shape} chosen_root=({},{}) above_water={} prefix_applied={} last_applied={applied_last} \
         effective_root=({},{}) truth={truth}",
        chosen.0, chosen.1, report.above_water, report.prefix_applied, rebuilt.instance.0, rebuilt.checkpoint_txg.0
    );
}

/// 反向链值能不能分辨「缺掉的那条在根里」与「缺掉的那条是这次要重放的第一条」：两条前驱头只差 txg（3 对 4）与点名单元的校验和，
/// 解一个 32 位的单元校验和让两者的 CRC32C 相同，再看后继记录 (1, 5) 的整条字节是不是逐字节相同。
fn back_chain_collision() {
    let named_with = |unit_checksum: u32| NamedUnit {
        locations: [
            LocationEntry { device: DeviceIdentity(1), slot: SlotNumber(50000), unit_checksum },
            LocationEntry { device: DeviceIdentity(2), slot: SlotNumber(50000), unit_checksum },
        ],
        unit_class: 1,
        birth_tree: TreeIdentifier(12),
        birth_txg: CheckpointTxg(0),
        key_tail: [0; 10],
    };
    let predecessor = |txg: u64, unit_checksum: u32| {
        let mut named = named_with(unit_checksum);
        named.birth_txg = CheckpointTxg(txg);
        JournalRecord {
            instance: InstanceGeneration(1),
            counter: 4,
            checkpoint_txg: CheckpointTxg(txg),
            transaction: 2,
            is_commit: true,
            back_chain: 0x1234_5678,
            filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW,
            new_tree_table: NodePointer::empty_root(),
            new_mapping_root: NodePointer::empty_root(),
            new_tree_identifier_watermark: 19,
            new_rollback_floor: CheckpointTxg(0),
            named: vec![named],
        }
        .to_bytes()
    };
    let covered_variant = |unit_checksum: u32| back_chain_of(&predecessor(3, unit_checksum));
    let target = back_chain_of(&predecessor(4, 0xCAFE_F00D));
    let base = covered_variant(0);
    let mut basis: Vec<(u32, u32)> = (0..32).map(|bit| (covered_variant(1 << bit) ^ base, 1u32 << bit)).collect();
    let mut want = target ^ base;
    let mut solution = 0u32;
    for pivot_bit in (0..32).rev() {
        let Some(row) = basis.iter().position(|(value, _)| value >> pivot_bit & 1 == 1) else { continue };
        let (pivot_value, pivot_mask) = basis.remove(row);
        for entry in &mut basis {
            if entry.0 >> pivot_bit & 1 == 1 {
                entry.0 ^= pivot_value;
                entry.1 ^= pivot_mask;
            }
        }
        if want >> pivot_bit & 1 == 1 {
            want ^= pivot_value;
            solution ^= pivot_mask;
        }
    }
    let solved = covered_variant(solution);
    let successor = |back_chain: u32| JournalRecord {
        instance: InstanceGeneration(1),
        counter: 5,
        checkpoint_txg: CheckpointTxg(4),
        transaction: 3,
        is_commit: true,
        back_chain,
        filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW,
        new_tree_table: NodePointer::empty_root(),
        new_mapping_root: NodePointer::empty_root(),
        new_tree_identifier_watermark: 20,
        new_rollback_floor: CheckpointTxg(0),
        named: vec![named_with(0xBEEF_0001)],
    }
    .to_bytes();
    println!(
        "C245R2-IMPL shape=S12_back_chain_collision residual_after_solve={want} unit_checksum_covered_variant={solution:#010x} \
         back_chain_covered_variant={solved:#010x} back_chain_head_variant={target:#010x} successor_bytes_identical={}",
        successor(solved) == successor(target)
    );
}

fn main() {
    back_chain_collision();
    let warm = [(1, 1, 1, 0, true, None), (1, 2, 2, 0, true, Some(1))];
    let with = |extra: &[RecordSpec]| -> Vec<RecordSpec> { warm.iter().copied().chain(extra.iter().copied()).collect() };
    probe("S1_mkfs_root_warmup_record", "none", (0, 0), &with(&[]), &[(1, 1)]);
    probe(
        "S2_H1_rollback_overwrote_covered_record",
        "apply(1,2)",
        (1, 1),
        &[(1, 1, 1, 0, true, None), (1, 2, 2, 0, true, Some(1)), (2, 1, 3, 1, true, None)],
        &[(2, 1), (1, 2)],
    );
    probe(
        "S3_uncommitted_rollback_record_above_water",
        "none",
        (1, 3),
        &with(&[(1, 3, 3, 1, true, Some(2)), (2, 3, 4, 1, true, None)]),
        &[(1, 1), (1, 2), (2, 3)],
    );
    probe("S4_H3_covered_record_unreadable", "apply(1,3)", (1, 2), &with(&[(1, 3, 3, 1, true, Some(2))]), &[(1, 1), (1, 3)]);
    let two = [(1, 3, 3, 1, true, Some(2)), (1, 4, 3, 2, true, Some(3))];
    probe("S5_multi_record_head_torn", "none", (1, 2), &with(&two), &[(1, 1), (1, 2), (1, 4)]);
    probe("S6_multi_record_tail_torn", "none(sixth_rule)", (1, 2), &with(&two), &[(1, 1), (1, 2), (1, 3)]);
    probe(
        "S7_one_transaction_two_records_complete",
        "apply(1,3)(1,4)",
        (1, 2),
        &with(&[(1, 3, 3, 1, false, Some(2)), (1, 4, 3, 1, true, Some(3))]),
        &[(1, 1), (1, 2), (1, 3), (1, 4)],
    );
    probe(
        "S8_torn_head_residual_after_new_instance_record",
        "none",
        (1, 2),
        &with(&[two[0], two[1], (2, 3, 4, 0, true, None)]),
        &[(1, 1), (1, 2), (2, 3), (1, 4)],
    );
    let in_flight = [(1, 3, 3, 1, true, Some(2)), (1, 4, 4, 2, true, Some(3))];
    probe(
        "S9_switch_from_covered_plus_one_overwrote_in_flight",
        "none",
        (1, 3),
        &with(&[in_flight[0], in_flight[1], (2, 4, 5, 2, true, None)]),
        &[(1, 1), (1, 2), (1, 3), (2, 4)],
    );
    probe(
        "S10_switch_from_mount_prefix_end_overwrote_warmup",
        "apply(1,4)",
        (1, 3),
        &with(&[in_flight[0], in_flight[1], (2, 1, 5, 2, true, None)]),
        &[(2, 1), (1, 2), (1, 3), (1, 4)],
    );
    probe(
        "S11_synthetic_back_chain_mismatch",
        "none",
        (1, 2),
        &with(&[(1, 3, 3, 1, true, Some(1))]),
        &[(1, 1), (1, 2), (1, 3)],
    );
}
