//! 副本装置（m2-presumed-clauses-r1 云端攻方，K4 / P6）：一次发布 N 条记录（并行线一：N 个事务 N 条记录，共享的提交内生块只在最后一条点名；
//! 同一次发布的记录共享同一个 checkpoint_txg，D23（journal 的角色与格式） 已定项 21）时，`replay_journal` 的链首接法。
//! 今天的 `crates/` 写不出这种环：这里在 mkfs 出来的真镜像上手造——每条记录用 `JournalRecord::to_bytes` 落到两份镜像的
//! `record_offset(jsn)` 上，点名的单元真写进单元区、校验和真算——再走 `scan_journal` → `replay_journal`（点名单元验证开着）。
//! 所选根 (1, 10)；发布 P0（txg 10，n0 条）、P1（txg 11，n1 条）、P2（txg 12，n2 条；n2 = 0 表示崩在 P1 的根槽之前、P2 没开始）。
//! 读不出的记录集合 U 取遍全部子集。每格打印一行；不断言，整张表交给报告判。
mod common;

use common::{parameters, IMAGE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::journal::{back_chain_of, record_offset, JournalRecord, NamedUnit};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::pointer::LocationEntry;
use singlefs_core::recovery::{choose_system_configuration, replay_journal, scan_journal};
use singlefs_core::root_record::RootRecord;
use singlefs_core::unit::{unit_filesystem_identifier, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};

const ROOT_TXG: u64 = 10;
const FIRST_COUNTER: u64 = 100;
const SHARED_UNITS_PER_PUBLISH: usize = 2;

#[derive(Clone, Copy, Debug)]
struct PlannedRecord {
    publish: usize,
    is_last_in_publish: bool,
    counter: u64,
    txg: u64,
    transaction: u64,
}

fn plan(record_counts: [usize; 3]) -> Vec<PlannedRecord> {
    let mut planned = Vec::new();
    let mut counter = FIRST_COUNTER;
    let mut transaction = 1;
    for (publish, count) in record_counts.iter().enumerate() {
        for index_in_publish in 0..*count {
            planned.push(PlannedRecord {
                publish,
                is_last_in_publish: index_in_publish + 1 == *count,
                counter,
                txg: ROOT_TXG + u64::try_from(publish).expect("小"),
                transaction,
            });
            counter += 1;
            transaction += 1;
        }
    }
    planned
}

fn genesis_image() -> (MemoryPool, RootRecord) {
    let parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let image = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    };
    (image, genesis.root)
}

/// 把一个单元的字节写到两块盘同一个槽上，交回点名项。
fn named_unit(image: &mut MemoryPool, slot: u64, unit_class: u8, bytes_len: usize, seed: u64, birth_txg: u64) -> NamedUnit {
    let bytes: Vec<u8> = (0..bytes_len)
        .map(|index| u8::try_from((index as u64 * 31 + seed * 17) % 251).expect("小"))
        .collect();
    let checksum = crc32_castagnoli(&bytes);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image
            .devices
            .get_mut(&device)
            .expect("盘")
            .write(SlotNumber(slot).to_device_offset(), &bytes);
    }
    NamedUnit {
        locations: [
            LocationEntry { device: DeviceIdentity(0), slot: SlotNumber(slot), unit_checksum: checksum },
            LocationEntry { device: DeviceIdentity(1), slot: SlotNumber(slot), unit_checksum: checksum },
        ],
        unit_class,
        birth_tree: TreeIdentifier(11),
        birth_txg: CheckpointTxg(birth_txg),
        key_tail: [0; 10],
    }
}

/// 造出一张盘：planned 里除了 unreadable 之外的记录都落两份镜像。交回 (镜像, 所选根)。
fn build_image(planned: &[PlannedRecord], unreadable_mask: u64) -> (MemoryPool, RootRecord) {
    build_image_corrupting(planned, unreadable_mask, None)
}

/// 同上，另把 jsn = `corrupt_data_unit_of` 那条记录点名的数据单元两份都改坏一个字节（再多一个介质故障）。
fn build_image_corrupting(planned: &[PlannedRecord], unreadable_mask: u64, corrupt_data_unit_of: Option<u64>) -> (MemoryPool, RootRecord) {
    let (mut image, genesis_root) = genesis_image();
    let parameters = parameters();
    let filesystem_identifier = unit_filesystem_identifier(&parameters.filesystem_identifier);
    let ring_bytes = parameters.geometry.journal_ring_bytes;
    let mut previous_bytes: Option<Vec<u8>> = None;
    let mut next_slot = 60_000u64;
    for (position, record_plan) in planned.iter().enumerate() {
        let mut named = vec![named_unit(&mut image, next_slot, UNIT_CLASS_DATA, 32768, record_plan.counter, record_plan.txg)];
        if corrupt_data_unit_of == Some(record_plan.counter) {
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                image.flip_byte(device, SlotNumber(next_slot).to_device_offset(), 100);
            }
        }
        next_slot += 2;
        if record_plan.is_last_in_publish {
            for shared in 0..SHARED_UNITS_PER_PUBLISH {
                named.push(named_unit(
                    &mut image,
                    next_slot,
                    UNIT_CLASS_INDEX_NODE,
                    16384,
                    record_plan.counter * 10 + u64::try_from(shared).expect("小"),
                    record_plan.txg,
                ));
                next_slot += 1;
            }
        }
        let record = JournalRecord {
            instance: InstanceGeneration(1),
            counter: record_plan.counter,
            checkpoint_txg: CheckpointTxg(record_plan.txg),
            transaction: record_plan.transaction,
            is_commit: true,
            back_chain: previous_bytes.as_deref().map_or(0, back_chain_of),
            filesystem_identifier,
            new_tree_table: genesis_root.tree_table,
            new_mapping_root: genesis_root.mapping_root,
            // 用树 ID 水位记下「这条记录的 jsn」：重建出来的根带着最后施加的那条的 jsn。
            new_tree_identifier_watermark: record_plan.counter,
            new_rollback_floor: CheckpointTxg(0),
            named,
        };
        let bytes = record.to_bytes();
        let parsed = JournalRecord::parse(&bytes, filesystem_identifier).expect("手造的记录自证得过");
        assert_eq!(parsed, record, "to_bytes / parse 往返逐字段相等");
        previous_bytes = Some(bytes.clone());
        if unreadable_mask & (1 << position) != 0 {
            continue;
        }
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            image
                .devices
                .get_mut(&device)
                .expect("盘")
                .write(record_offset(record_plan.counter, ring_bytes), &bytes);
        }
    }
    let mut root = genesis_root;
    root.instance = InstanceGeneration(1);
    root.checkpoint_txg = CheckpointTxg(ROOT_TXG);
    (image, root)
}

/// 条款口径下「链首知道锚点时」该接到哪：锚点 = P0 的最后一条；从它的下一条起 jsn 严格连续，碰到读不出的就停。
fn clause_chain_with_known_anchor(planned: &[PlannedRecord], unreadable_mask: u64) -> Vec<u64> {
    let anchor = planned
        .iter()
        .filter(|record| record.publish == 0)
        .map(|record| record.counter)
        .max()
        .expect("P0 至少一条");
    let mut chain = Vec::new();
    for (position, record) in planned.iter().enumerate() {
        if record.counter <= anchor {
            continue;
        }
        if unreadable_mask & (1 << position) != 0 {
            break;
        }
        chain.push(record.counter);
    }
    chain
}

#[test]
fn k4_chain_head_with_several_records_per_publish() {
    let arm = std::env::var("PRESUMED_R1_CHAIN_HEAD").unwrap_or_else(|_| "today".into());
    let parameters = parameters();
    let ring_bytes = parameters.geometry.journal_ring_bytes;
    let mut cells = 0;
    let mut tally: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for n0 in 1..=3usize {
        for n1 in 1..=3usize {
            for n2 in 0..=2usize {
                let planned = plan([n0, n1, n2]);
                for unreadable_mask in 0..(1u64 << planned.len()) {
                    cells += 1;
                    let (image, root) = build_image(&planned, unreadable_mask);
                    let system_configuration = choose_system_configuration(&image).expect("系统配置");
                    let records = scan_journal(&image, &system_configuration);
                    assert_eq!(
                        records.len(),
                        planned.len() - usize::try_from(unreadable_mask.count_ones()).expect("小"),
                        "扫描读到的就是没被撕掉的那几条"
                    );
                    let (report, rebuilt) = replay_journal(&image, &root, ring_bytes, &records, true, None);
                    let applied = report.prefix_applied;
                    let applied_counters: Vec<u64> = if applied == 0 {
                        Vec::new()
                    } else {
                        let last = rebuilt.tree_identifier_watermark;
                        (last + 1 - u64::try_from(applied).expect("小")..=last).collect()
                    };
                    let clause = clause_chain_with_known_anchor(&planned, unreadable_mask);
                    let anchor_readable = planned.iter().enumerate().any(|(position, record)| {
                        record.publish == 0 && record.is_last_in_publish && unreadable_mask & (1 << position) == 0
                    });
                    let any_root_publish_record_readable = planned.iter().enumerate().any(|(position, record)| {
                        record.publish == 0 && unreadable_mask & (1 << position) == 0
                    });
                    let branch = if anchor_readable {
                        "anchor-readable"
                    } else if any_root_publish_record_readable {
                        "anchor-torn-other-P0-readable"
                    } else {
                        "all-P0-torn(no-anchor)"
                    };
                    let today_head = applied_counters.first().copied();
                    let clause_head = clause.first().copied();
                    let verdict = match (today_head, clause_head) {
                        (None, None) => "match-none",
                        (Some(today), Some(clause)) if today == clause => "match",
                        (Some(_), _) => "OVER(接上不该接的)",
                        (None, Some(_)) => "UNDER(少施加)",
                    };
                    // 最少要几个故障：P0 的记录读不出每条 2 个（两份镜像）；n2 = 0 时 P1 缺的是崩溃造成的（0 个）、根 11 从没写过；
                    // n2 > 0 时根 11 写过、要读不出（1 个），P1 缺的每条 2 个，P2 缺的是崩溃造成的。
                    let mut faults = 0u64;
                    for (position, record) in planned.iter().enumerate() {
                        if unreadable_mask & (1 << position) == 0 {
                            continue;
                        }
                        faults += match (record.publish, n2) {
                            (0, _) => 2,
                            (1, 0) => 0,
                            (1, _) => 2,
                            _ => 0,
                        };
                    }
                    if n2 > 0 {
                        faults += 1;
                    }
                    let torn: Vec<u64> = planned
                        .iter()
                        .enumerate()
                        .filter(|(position, _)| unreadable_mask & (1 << position) != 0)
                        .map(|(_, record)| record.counter)
                        .collect();
                    *tally.entry(format!("{branch} {verdict}")).or_default() += 1;
                    println!(
                        "K4 [{arm}] n=({n0},{n1},{n2}) torn={torn:?} branch={branch} applied={applied_counters:?} clause_known_anchor={clause:?} verdict={verdict} min_faults={faults} verification_failed={} effective_txg={}",
                        report.verification_failed,
                        rebuilt.checkpoint_txg.0
                    );
                }
            }
        }
    }
    for (key, count) in &tally {
        println!("K4-TALLY [{arm}] {key} = {count}");
    }
    println!("K4-SUMMARY [{arm}] cells={cells}");
}

/// 多接那一格的代价：P0 那条读不出、P1 第一条（jsn 101）崩溃时没落盘、它点名的数据单元两份都坏。
/// 今天的接法从 jsn 102 接起、施加 P1，jsn 101 点名的那个坏单元一次都没被验；锚点读得出时同一段历史在 101 那个断号上停住。
#[test]
fn k4_over_connect_skips_the_verification_of_the_unit_named_only_by_the_missing_record() {
    let arm = std::env::var("PRESUMED_R1_CHAIN_HEAD").unwrap_or_else(|_| "today".into());
    let ring_bytes = parameters().geometry.journal_ring_bytes;
    let planned = plan([1, 2, 0]);
    for (label, unreadable_mask) in [("anchor-torn", 0b011u64), ("anchor-readable", 0b010u64)] {
        let (image, root) = build_image_corrupting(&planned, unreadable_mask, Some(101));
        let system_configuration = choose_system_configuration(&image).expect("系统配置");
        let records = scan_journal(&image, &system_configuration);
        let (report, rebuilt) = replay_journal(&image, &root, ring_bytes, &records, true, None);
        println!(
            "K4-UNIT [{arm}] {label} torn_mask={unreadable_mask:#b} corrupt_unit_named_by=101 applied={} verification_passed={} verification_failed={} effective_txg={} last_applied_jsn={}",
            report.prefix_applied,
            report.verification_passed,
            report.verification_failed,
            rebuilt.checkpoint_txg.0,
            if report.prefix_applied == 0 { 0 } else { rebuilt.tree_identifier_watermark }
        );
    }
}
