// 攻方腿 m3-verus-land-code-r1-opus 的对拍：改前（fcc4d340）的 replay_journal 与改后（调 singlefs_replay_prefix::replay_core）
// 在同一份随机造出的记录表与同一个假盘上各跑一次，比结局（Ok 的报告与根、Err 的成员与内容、panic 的消息）与读盘序列（盘、偏移、长度，逐次）。
// setup-differential.sh 把改前的两个函数（改名加 _before）拼在这份前面，整份挂成 recovery 的子模块（只在 cfg(test) 编）。
use super::*;
use crate::address::TreeIdentifier;
use crate::journal::{JournalRecordOrdinalWithinPublish, NamedUnit};
use std::cell::RefCell;
use std::panic::{catch_unwind, AssertUnwindSafe};

const FILESYSTEM_IDENTIFIER_BYTES: [u8; 16] = [7; 16];

fn filesystem_identifier() -> u64 {
    unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER_BYTES)
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
}

struct FakePool {
    storage: BTreeMap<(u32, u64), Vec<u8>>,
    log: RefCell<Vec<(u32, u64, usize)>>,
}

impl PoolReader for FakePool {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        vec![DeviceIdentity(0), DeviceIdentity(1)]
    }
    fn device_size_in_bytes(&self, _device: DeviceIdentity) -> Option<u64> {
        Some(1 << 40)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.log.borrow_mut().push((device.0, offset.0, length));
        self.storage
            .get(&(device.0, offset.0))
            .filter(|bytes| bytes.len() >= length)
            .map(|bytes| bytes[..length].to_vec())
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

struct Case {
    pool: FakePool,
    records: BTreeMap<(InstanceGeneration, u64), JournalRecord>,
    root: RootRecord,
    ring_bytes: u64,
    verify_named_units: bool,
}

fn named_units(rng: &mut Rng, noise: u64, storage: &mut BTreeMap<(u32, u64), Vec<u8>>) -> Vec<NamedUnit> {
    (0..rng.below(3))
        .map(|_| {
            let unit_class = [UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED, 99]
                [usize::try_from(rng.below(4)).expect("4")];
            let length = unit_bytes_for_class(unit_class).unwrap_or(4096);
            let locations = [0u32, 1].map(|device| {
                let slot = SlotNumber(60_000 + rng.below(6));
                let bytes: Vec<u8> = (0..length).map(|at| (at as u64 ^ slot.0) as u8).collect();
                let checksum = crc32_castagnoli(&bytes);
                if rng.chance(100 - noise / 2) {
                    storage.insert((device, slot.to_device_offset().0), bytes);
                }
                LocationEntry {
                    device: DeviceIdentity(device),
                    slot,
                    unit_checksum: if rng.chance(100 - noise / 3) { checksum } else { checksum ^ 1 },
                }
            });
            NamedUnit {
                locations,
                unit_class,
                birth_tree: TreeIdentifier(0),
                birth_txg: CheckpointTxg(0),
                key_tail: [0; 10],
            }
        })
        .collect()
}

fn generate(rng: &mut Rng) -> Case {
    let ring_slots = [3u64, 6, 9, 12, 30, 63][usize::try_from(rng.below(6)).expect("6")];
    let ring_bytes = ring_slots * JOURNAL_RECORD_BYTES;
    // 每个输入取一档噪声：0（健康历史）、6、25（百分比），扰动、反向链写错、单元缺失与校验和错都按它取。
    let noise = [0u64, 6, 25][usize::try_from(rng.below(3)).expect("3")];
    let mut storage = BTreeMap::new();
    let mut written: Vec<JournalRecord> = Vec::new();
    let mut publish_txgs: Vec<(u32, u64)> = Vec::new();
    for instance in [0u32, 1, 2] {
        if instance != 1 && !rng.chance(40) {
            continue;
        }
        let mut counter = 1 + rng.below(3);
        let mut txg = 1 + rng.below(4);
        let mut transaction = 1 + rng.below(3);
        for _publish in 0..1 + rng.below(5) {
            let record_count = 1 + rng.below(3);
            let split = if record_count >= 2 && rng.chance(40) {
                1 + rng.below(record_count - 1)
            } else {
                record_count
            };
            publish_txgs.push((instance, txg));
            for position in 0..record_count {
                let last = position + 1 == record_count;
                written.push(JournalRecord {
                    instance: InstanceGeneration(instance),
                    counter,
                    checkpoint_txg: CheckpointTxg(txg),
                    transaction: if position < split { transaction } else { transaction + 1 },
                    is_commit: position + 1 == split || last,
                    ordinal_within_publish: JournalRecordOrdinalWithinPublish(
                        u32::try_from(position + 1).expect("3"),
                    ),
                    place_in_publish: if last {
                        JournalRecordPlaceInPublish::LastRecordOfThePublish
                    } else {
                        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
                    },
                    back_chain: 0,
                    filesystem_identifier: filesystem_identifier(),
                    new_tree_table: NodePointer::empty_root(),
                    new_mapping_root: NodePointer::empty_root(),
                    new_tree_identifier_watermark: counter * 10 + u64::from(instance),
                    new_rollback_floor: CheckpointTxg(0),
                    named: named_units(rng, noise, &mut storage),
                });
                counter += 1;
            }
            transaction += if split < record_count { 2 } else { 1 };
            txg += if rng.chance(5) { 2 } else { 1 };
        }
    }
    for record in &mut written {
        if !rng.chance(noise) {
            continue;
        }
        match rng.below(7) {
            0 => {
                record.ordinal_within_publish =
                    JournalRecordOrdinalWithinPublish(1 + u32::try_from(rng.below(4)).expect("4"));
            }
            1 => {
                record.place_in_publish = match record.place_in_publish {
                    JournalRecordPlaceInPublish::LastRecordOfThePublish => {
                        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
                    }
                    JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => {
                        JournalRecordPlaceInPublish::LastRecordOfThePublish
                    }
                };
            }
            2 => record.is_commit = !record.is_commit,
            3 => {
                record.checkpoint_txg = CheckpointTxg(
                    (record.checkpoint_txg.0 + rng.below(3)).saturating_sub(1).max(1),
                );
            }
            4 => record.transaction += 1,
            5 => record.instance = InstanceGeneration(u32::try_from(rng.below(3)).expect("3")),
            _ => record.counter += 1,
        }
    }
    written.sort_by_key(|record| (record.instance, record.counter));
    // 反向链按升序算：前一条（同实例、计数器小 1）照它最终的字节算链值；8% 写错。
    let mut bytes_by_key: BTreeMap<(InstanceGeneration, u64), Vec<u8>> = BTreeMap::new();
    for record in &mut written {
        if let Some(previous) = record
            .counter
            .checked_sub(1)
            .and_then(|previous_counter| bytes_by_key.get(&(record.instance, previous_counter)))
        {
            record.back_chain = back_chain_of(previous);
        }
        if rng.chance(noise / 3) {
            record.back_chain = rng.next() as u32;
        }
        bytes_by_key.insert((record.instance, record.counter), record.to_bytes());
    }
    for record in &written {
        let bytes = record.to_bytes();
        let offset = record_offset(record.counter, ring_bytes).0;
        if rng.chance(100 - noise / 2) {
            storage.insert((0, offset), bytes.clone());
        }
        if rng.chance(50) {
            storage.insert((1, offset), bytes.clone());
        } else if rng.chance(15) {
            let mut other = record.clone();
            other.back_chain ^= 1;
            storage.insert((1, offset), other.to_bytes());
        }
    }
    // 扫环：与 scan_journal 同一个收法（盘按次序、槽按次序，两份任一份自证过即算在，先到先得）。
    let mut records = BTreeMap::new();
    for device in [0u32, 1] {
        for slot in 0..ring_slots {
            let offset = JOURNAL_RING_START_SLOT * SLOT_BYTES + slot * JOURNAL_RECORD_BYTES;
            if let Some(record) = storage
                .get(&(device, offset))
                .and_then(|bytes| JournalRecord::parse(bytes, filesystem_identifier()))
            {
                records.entry((record.instance, record.counter)).or_insert(record);
            }
        }
    }
    let (root_instance, root_txg) = if rng.chance(3) {
        (1, u64::MAX - rng.below(2))
    } else if rng.chance(10) || publish_txgs.is_empty() {
        (u32::try_from(rng.below(3)).expect("3"), rng.below(8))
    } else {
        publish_txgs[usize::try_from(rng.below(publish_txgs.len() as u64)).expect("下标")]
    };
    let root = RootRecord {
        filesystem_identifier: FILESYSTEM_IDENTIFIER_BYTES,
        unmount_marker: if rng.chance(50) {
            UnmountMarker::WrittenByTheUnmountSequence
        } else {
            UnmountMarker::NotWrittenByTheUnmountSequence
        },
        instance: InstanceGeneration(root_instance),
        checkpoint_txg: CheckpointTxg(root_txg),
        tree_table: NodePointer::empty_root(),
        tree_identifier_watermark: 77,
        rollback_floor: CheckpointTxg(0),
        instance_table: NodePointer::empty_root(),
        mapping_root: NodePointer::empty_root(),
        allocation_record_tree_root: NodePointer::empty_root(),
    };
    Case {
        pool: FakePool { storage, log: RefCell::new(Vec::new()) },
        records,
        root,
        ring_bytes,
        verify_named_units: rng.chance(70),
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "非字符串 panic".to_string())
}

type Replay = fn(
    &dyn PoolReader,
    &RootRecord,
    u64,
    &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
    bool,
) -> Result<(JournalScanReport, RootRecord), RecoveryFailure>;

fn run_one(case: &Case, replay: Replay) -> (String, Vec<(u32, u64, usize)>) {
    case.pool.log.borrow_mut().clear();
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        replay(&case.pool, &case.root, case.ring_bytes, &case.records, case.verify_named_units)
    }));
    let text = match outcome {
        Ok(result) => format!("{result:?}"),
        Err(payload) => format!("panic: {}", panic_message(payload)),
    };
    (text, case.pool.log.take())
}

fn field_of(text: &str, name: &str) -> u64 {
    text.split(name)
        .nth(1)
        .and_then(|rest| rest.split([',', ' ', '}']).next())
        .and_then(|number| number.parse().ok())
        .unwrap_or(u64::MAX)
}

fn kind_of(text: &str) -> String {
    if let Some(message) = text.strip_prefix("panic: ") {
        return format!("panic {}", message.chars().take(40).collect::<String>());
    }
    if text.starts_with("Err") {
        return text.split(['(', ' ', '{']).nth(1).unwrap_or("Err").to_string();
    }
    format!(
        "Ok applied={} verification_failed={} verification_passed>0={}",
        field_of(text, "prefix_applied: ").min(4),
        field_of(text, "verification_failed: "),
        field_of(text, "verification_passed: ") > 0
    )
}

#[test]
fn opus_before_and_after_the_core_agree_on_random_record_tables() {
    let cases: u64 = std::env::var("OPUS_DIFF_CASES")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(20_000);
    let seed: u64 = std::env::var("OPUS_DIFF_SEED")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(1);
    std::panic::set_hook(Box::new(|_| {}));
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let mut tally: BTreeMap<String, u64> = BTreeMap::new();
    let mut disagreements = 0u64;
    let mut first_disagreement = None;
    let mut reads_total = 0u64;
    for case_number in 0..cases {
        let case = generate(&mut rng);
        let before = run_one(&case, replay_journal_before);
        let after = run_one(&case, replay_journal);
        reads_total += before.1.len() as u64;
        *tally.entry(kind_of(&before.0)).or_default() += 1;
        if before != after {
            disagreements += 1;
            if first_disagreement.is_none() {
                first_disagreement = Some((case_number, before.0.clone(), after.0.clone()));
            }
        }
    }
    let _ = std::panic::take_hook();
    println!("opus-diff seed={seed} cases={cases} disagreements={disagreements} reads_total={reads_total}");
    for (kind, count) in &tally {
        println!("opus-diff tally {kind}: {count}");
    }
    if let Some(first) = &first_disagreement {
        println!("opus-diff first disagreement: {first:?}");
    }
    assert_eq!(disagreements, 0, "改前改后在 {disagreements} 个输入上结局或读盘序列不同");
}

/// G10：`pub fn replay_journal` 的调用方直接造记录表（不经 scan_journal 的 6 字节计数器）时，锚点计数器取 u64 最大值。
/// 核的 requires（计数器不是最大值）在这里不成立；看改前改后各怎么收场。
#[test]
fn opus_counter_at_u64_max_before_and_after() {
    std::panic::set_hook(Box::new(|_| {}));
    let anchor = JournalRecord {
        instance: InstanceGeneration(1),
        counter: u64::MAX,
        checkpoint_txg: CheckpointTxg(3),
        transaction: 1,
        is_commit: true,
        ordinal_within_publish: JournalRecordOrdinalWithinPublish(1),
        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
        back_chain: 0,
        filesystem_identifier: filesystem_identifier(),
        new_tree_table: NodePointer::empty_root(),
        new_mapping_root: NodePointer::empty_root(),
        new_tree_identifier_watermark: 0,
        new_rollback_floor: CheckpointTxg(0),
        named: Vec::new(),
    };
    let records: BTreeMap<_, _> = [((anchor.instance, anchor.counter), anchor)]
        .into_iter()
        .collect();
    let root = RootRecord {
        filesystem_identifier: FILESYSTEM_IDENTIFIER_BYTES,
        unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
        tree_table: NodePointer::empty_root(),
        tree_identifier_watermark: 0,
        rollback_floor: CheckpointTxg(0),
        instance_table: NodePointer::empty_root(),
        mapping_root: NodePointer::empty_root(),
        allocation_record_tree_root: NodePointer::empty_root(),
    };
    let case = Case {
        pool: FakePool { storage: BTreeMap::new(), log: RefCell::new(Vec::new()) },
        records,
        root,
        ring_bytes: 3 * JOURNAL_RECORD_BYTES,
        verify_named_units: true,
    };
    let before = run_one(&case, replay_journal_before);
    let after = run_one(&case, replay_journal);
    let _ = std::panic::take_hook();
    println!("opus-u64max before={:?}", before.0);
    println!("opus-u64max after={:?}", after.0);
    assert_eq!(before, after);
}

fn plain_record(counter: u64, checkpoint_txg: u64, ordinal: u32, last: bool) -> JournalRecord {
    JournalRecord {
        instance: InstanceGeneration(1),
        counter,
        checkpoint_txg: CheckpointTxg(checkpoint_txg),
        transaction: counter,
        is_commit: true,
        ordinal_within_publish: JournalRecordOrdinalWithinPublish(ordinal),
        place_in_publish: if last {
            JournalRecordPlaceInPublish::LastRecordOfThePublish
        } else {
            JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
        },
        back_chain: 0,
        filesystem_identifier: filesystem_identifier(),
        new_tree_table: NodePointer::empty_root(),
        new_mapping_root: NodePointer::empty_root(),
        new_tree_identifier_watermark: counter,
        new_rollback_floor: CheckpointTxg(0),
        named: Vec::new(),
    }
}

/// C1 的一格：`pub fn replay_journal` 收任意 `BTreeMap`；键与记录自己的 (实例, 计数器) 不一致时（scan_journal 造不出，类型挡不住），
/// 改前按记录字段排序（`sort_by_key`），改后按键序交给核（核里没有排序）——两边结局不同。
#[test]
fn opus_a_table_whose_keys_disagree_with_the_record_fields_splits_before_and_after() {
    let anchor = plain_record(1, 1, 1, true);
    let first_of_the_next_publish = plain_record(2, 2, 1, false);
    let last_of_the_next_publish = plain_record(3, 2, 2, true);
    let records: BTreeMap<_, _> = [
        ((InstanceGeneration(1), 1), anchor),
        ((InstanceGeneration(1), 2), last_of_the_next_publish),
        ((InstanceGeneration(1), 3), first_of_the_next_publish),
    ]
    .into_iter()
    .collect();
    let root = RootRecord {
        filesystem_identifier: FILESYSTEM_IDENTIFIER_BYTES,
        unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(1),
        tree_table: NodePointer::empty_root(),
        tree_identifier_watermark: 0,
        rollback_floor: CheckpointTxg(0),
        instance_table: NodePointer::empty_root(),
        mapping_root: NodePointer::empty_root(),
        allocation_record_tree_root: NodePointer::empty_root(),
    };
    let case = Case {
        pool: FakePool { storage: BTreeMap::new(), log: RefCell::new(Vec::new()) },
        records,
        root,
        ring_bytes: 30 * JOURNAL_RECORD_BYTES,
        verify_named_units: false,
    };
    let before = run_one(&case, replay_journal_before);
    let after = run_one(&case, replay_journal);
    println!("opus-keys before={}", before.0);
    println!("opus-keys after={}", after.0);
    assert_ne!(before.0, after.0, "键与字段不一致时改前改后结局不同");
}
