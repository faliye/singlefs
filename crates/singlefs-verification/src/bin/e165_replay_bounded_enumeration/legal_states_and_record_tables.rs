//! 合法历史一族的一个状态（记录在盘上的拷贝、两盘镜像、直接建表）与扫盘建表（登记 r2 第三节「由 scan_journal 建表」、5.4 的 T-表与 T-行），连同判据读的「完整在盘上」与最远版本。

use super::*;

// ── 合法历史一族的一个状态

/// 0 号盘上一条记录在不在；在的话它点名的单元是五态里哪一态（不点名单元的域恒取两份都好）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecordPresence {
    NotOnDisk,
    OnDisk(UnitState),
}

/// 每条记录在 1 号盘上有没有一份（登记 r2 5.3 多盘域；D22（单元原子性怎么合成） 已定项 8 第 2 条「journal 环在两块盘上互为镜像」）。
#[derive(Clone, Copy)]
pub(super) enum JournalMirroring<'state> {
    /// 单盘：生成的记录只写 0 号盘（原登记 5.1；R 域的残留另在 1 号盘）。
    FirstDeviceOnly,
    /// 两盘镜像：第 r 条记录在 1 号盘同一落点那一份落没落；0 号盘那一份看 `presence`。
    BothDevices {
        second_device_copy_is_on_disk: &'state [bool],
    },
}

pub(super) struct LegalState<'state> {
    pub(super) history: &'state History,
    pub(super) presence: &'state [RecordPresence],
    pub(super) mirroring: JournalMirroring<'state>,
    pub(super) residue: Option<&'state Residue>,
    pub(super) chosen_version: usize,
    pub(super) verify_named_units: bool,
}

pub(super) type RecordTable = BTreeMap<(InstanceGeneration, u64), JournalRecord>;

/// 表里一条记录在盘上那一份的真相：S 的 (d) 要它的头算出的链值，(k) 要它点名的单元两份是不是都读得出、校验和都对。
#[derive(Clone, Copy, Debug)]
pub(super) struct RecordTruthOnDisk {
    pub(super) instance: u32,
    pub(super) counter: u64,
    pub(super) chain_value_of_header: u32,
    pub(super) named_units_all_verified: bool,
}

/// 直接建表读盘的次序（登记 5.1「按盘号升序」；r2 5.3「先 0 号盘的全部落了的记录（按记录次序），再 1 号盘的」）。
const DEVICE_ORDER_OF_THE_DIRECT_TABLE: [u32; 2] = [0, 1];

/// 扫盘建表喂给 `scan_journal` 的系统配置里，`scan_journal` 不读的那些字段取的值（它只读 fsid 与 journal 环长，`recovery.rs` 里
/// `scan_journal` 的函数体）：值随便取、只求造得出一份结构体；不借 mkfs 与写路径（登记 r2 第三节「盘上字节由谁造」、停机条款 SH9）。
const SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ: u32 = 4096;
pub(super) const SCAN_CONFIGURATION_DEVICE_COUNT: u32 = 2;
pub(super) const SCAN_CONFIGURATION_ROOT_RING_BASE_SLOT_THE_SCAN_DOES_NOT_READ: u64 = 0;
pub(super) const SCAN_CONFIGURATION_SLOT_GENERATION_THE_SCAN_DOES_NOT_READ: u64 = 1;
pub(super) const SCAN_CONFIGURATION_JOURNAL_TAIL_THE_SCAN_DOES_NOT_READ: u64 = 0;
pub(super) const SCAN_CONFIGURATION_ROLLBACK_FLOOR_THE_SCAN_DOES_NOT_READ: u64 = 0;

/// 一条记录在一块盘上的一份：槽（字节、`parse` 的结果、头算出的链值）、它点名的单元两份摆在哪（跟着 0 号盘那一份摆）、
/// 点名单元是不是两份都好（S 的 (k) 要它）。
struct RecordCopyOnADevice<'state> {
    slot: &'state GeneratedSlot,
    named_unit_copies: Option<(u64, StoredBytes<'state>, StoredBytes<'state>)>,
    named_units_all_verified: bool,
}

/// 一块盘上落了的记录，按记录次序：0 号盘是 `presence` 里在盘上的那几条；1 号盘是镜像那几份（多盘域），之后是 R 域的残留。
fn record_copies_on_the_device<'state>(
    state: &LegalState<'state>,
    device: u32,
) -> Vec<RecordCopyOnADevice<'state>> {
    let mut copies = Vec::new();
    if device == 0 {
        for (record_index, slot) in state.history.slots.iter().enumerate() {
            let RecordPresence::OnDisk(unit_state) = state.presence[record_index] else {
                continue;
            };
            let named_unit_copies = state.history.units[record_index].as_ref().map(|unit| {
                let (first_device_copy, second_device_copy) = match unit_state {
                    UnitState::BothCopiesGood => (
                        StoredBytes::Readable(&unit.good_bytes),
                        StoredBytes::Readable(&unit.good_bytes),
                    ),
                    UnitState::FirstDeviceCopyUnreadable => (
                        StoredBytes::Unreadable,
                        StoredBytes::Readable(&unit.good_bytes),
                    ),
                    UnitState::SecondDeviceCopyUnreadable => (
                        StoredBytes::Readable(&unit.good_bytes),
                        StoredBytes::Unreadable,
                    ),
                    UnitState::FirstDeviceCopyChecksumWrong => (
                        StoredBytes::Readable(&unit.bytes_with_wrong_checksum),
                        StoredBytes::Readable(&unit.good_bytes),
                    ),
                    UnitState::SecondDeviceCopyChecksumWrong => (
                        StoredBytes::Readable(&unit.good_bytes),
                        StoredBytes::Readable(&unit.bytes_with_wrong_checksum),
                    ),
                };
                (unit.offset_in_bytes, first_device_copy, second_device_copy)
            });
            copies.push(RecordCopyOnADevice {
                slot,
                named_unit_copies,
                named_units_all_verified: unit_state == UnitState::BothCopiesGood,
            });
        }
        return copies;
    }
    if let JournalMirroring::BothDevices {
        second_device_copy_is_on_disk,
    } = state.mirroring
    {
        for (record_index, slot) in state.history.slots.iter().enumerate() {
            if !second_device_copy_is_on_disk[record_index] {
                continue;
            }
            assert!(
                state.history.units[record_index].is_none(),
                "多盘域不点名单元（登记 r2 5.3）：1 号盘那一份没有自己的点名单元摆法"
            );
            copies.push(RecordCopyOnADevice {
                slot,
                named_unit_copies: None,
                named_units_all_verified: true,
            });
        }
    }
    if let Some(residue) = state.residue {
        copies.push(RecordCopyOnADevice {
            slot: &residue.slot,
            named_unit_copies: None,
            named_units_all_verified: true,
        });
    }
    copies
}

/// 把一个状态摆上内存盘，并照登记 5.1 建 `records` 表：按 [`DEVICE_ORDER_OF_THE_DIRECT_TABLE`] 的盘次序、逐槽取 `parse` 的结果，
/// 读得出的按 (实例, 计数器) 先到先得；`truths` 记先到的那一份。
pub(super) fn record_table_and_disk_of<'state>(
    state: &LegalState<'state>,
) -> (RecordTable, MemoryDisk<'state>, Vec<RecordTruthOnDisk>) {
    record_table_and_disk_of_in_device_order(state, DEVICE_ORDER_OF_THE_DIRECT_TABLE)
}

/// 同 [`record_table_and_disk_of`]，直接建表读盘的次序由 `device_order` 给：登记 r2 第八节盘次序那一点上，直接建表分两栏各比一遍，
/// 「盘号升序」栏给 [`DEVICE_ORDER_OF_THE_DIRECT_TABLE`]、「照 `device_identities()` 次序」栏给内存盘报的次序。
pub(super) fn record_table_and_disk_of_in_device_order<'state>(
    state: &LegalState<'state>,
    device_order: [u32; 2],
) -> (RecordTable, MemoryDisk<'state>, Vec<RecordTruthOnDisk>) {
    let mut records = RecordTable::new();
    let mut truths: Vec<RecordTruthOnDisk> = Vec::new();
    let mut contents: Vec<(u32, u64, StoredBytes<'state>)> = Vec::new();
    for device in device_order {
        for copy in record_copies_on_the_device(state, device) {
            contents.push((
                device,
                copy.slot.offset_in_bytes,
                StoredBytes::Readable(&copy.slot.bytes),
            ));
            if let Some((unit_offset_in_bytes, first_device_copy, second_device_copy)) =
                copy.named_unit_copies
            {
                contents.push((0, unit_offset_in_bytes, first_device_copy));
                contents.push((1, unit_offset_in_bytes, second_device_copy));
            }
            if let Some(parsed) = copy.slot.parsed.as_ref() {
                if let btree_map::Entry::Vacant(vacant_place) =
                    records.entry((parsed.instance, parsed.counter))
                {
                    vacant_place.insert(parsed.clone());
                    truths.push(RecordTruthOnDisk {
                        instance: parsed.instance.0,
                        counter: parsed.counter,
                        chain_value_of_header: copy.slot.chain_value_of_header,
                        named_units_all_verified: copy.named_units_all_verified,
                    });
                }
            }
        }
    }
    (
        records,
        MemoryDisk {
            ring: state.history.ring,
            reported_disk_order: state.history.run_profile.reported_disk_order,
            contents,
        },
        truths,
    )
}

// ── 扫盘建表（登记 r2 第三节「由 scan_journal 建表」、5.4 的 T-表与 T-行）

/// 喂给 `scan_journal` 的系统配置：fsid 取时间线根的那个 16 字节常量（低 8 字节就是记录头里那 8 字节），journal 环长取这个环几何的字节数，
/// 其余字段 `scan_journal` 不读（[`SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ`] 一组常量）。
pub(super) fn system_configuration_for_the_scan(ring: RingGeometry) -> SystemConfiguration {
    SystemConfiguration {
        immutable: SystemImmutableConfiguration {
            filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,
            this_device: DeviceIdentity(0),
            device_count: SCAN_CONFIGURATION_DEVICE_COUNT,
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            sizes: SystemImmutableSizes {
                physical_block_size: SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ,
                minimum_input_output_bytes: SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ,
                fixed_structure_slot_spacing: SCAN_CONFIGURATION_BLOCK_BYTES_THE_SCAN_DOES_NOT_READ,
                journal_ring_bytes: ring.ring_bytes(),
                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
            },
            journal_ring_start_slot: SlotNumber(LOCAL_JOURNAL_RING_START_SLOT),
            root_ring_base_slot: SlotNumber(
                SCAN_CONFIGURATION_ROOT_RING_BASE_SLOT_THE_SCAN_DOES_NOT_READ,
            ),
        },
        mutable: SystemMutableConfiguration,
        runtime: SystemRuntimeConfiguration,
        quantities: SystemRuntimeQuantities {
            slot_generation: SCAN_CONFIGURATION_SLOT_GENERATION_THE_SCAN_DOES_NOT_READ,
            journal_tail: SCAN_CONFIGURATION_JOURNAL_TAIL_THE_SCAN_DOES_NOT_READ,
            journal_instance: InstanceGeneration(CHOSEN_INSTANCE),
            rollback_floor: CheckpointTxg(SCAN_CONFIGURATION_ROLLBACK_FLOOR_THE_SCAN_DOES_NOT_READ),
        },
    }
}

/// 扫盘建表：对摆好的内存盘调 `scan_journal`。`implementation` 是 T-表那条对照时，建完把键最大的那条记录删掉（假结局：扫盘建的表少一条）。
pub(super) fn scanned_record_table_of(
    disk: &MemoryDisk<'_>,
    implementation: Implementation,
) -> RecordTable {
    let mut table = scan_journal(disk, &system_configuration_for_the_scan(disk.ring));
    if implementation == Implementation::Control(ControlKind::ScanTableLosesItsLargestKeyRecord) {
        table.pop_last();
    }
    table
}

/// T-表：同一个状态上两张表逐键逐值比，有一处不同即不同（登记 r2 5.4）。
fn the_two_record_tables_differ(direct_table: &RecordTable, scanned_table: &RecordTable) -> bool {
    direct_table != scanned_table
}

/// 两张表差在哪：只在直接建的表里的键、只在扫盘建的表里的键、两边都有而记录不同的键（键写成 `实例:计数器`）。
fn describe_how_the_two_record_tables_differ(
    direct_table: &RecordTable,
    scanned_table: &RecordTable,
) -> String {
    let key_text = |key: &(InstanceGeneration, u64)| format!("{}:{}", key.0 .0, key.1);
    let keys_of = |keys: Vec<String>| {
        if keys.is_empty() {
            "none".to_string()
        } else {
            keys.join("+")
        }
    };
    let only_in_the_direct_table = keys_of(
        direct_table
            .keys()
            .filter(|key| !scanned_table.contains_key(*key))
            .map(key_text)
            .collect(),
    );
    let only_in_the_scanned_table = keys_of(
        scanned_table
            .keys()
            .filter(|key| !direct_table.contains_key(*key))
            .map(key_text)
            .collect(),
    );
    let with_different_records = keys_of(
        direct_table
            .iter()
            .filter(|(key, record)| {
                scanned_table
                    .get(*key)
                    .is_some_and(|scanned_record| scanned_record != *record)
            })
            .map(|(key, _record)| key_text(key))
            .collect(),
    );
    format!(
        "direct_table_entries={};scanned_table_entries={};only_in_the_direct_table={only_in_the_direct_table};only_in_the_scanned_table={only_in_the_scanned_table};same_key_different_record={with_different_records}",
        direct_table.len(),
        scanned_table.len()
    )
}

/// 一个状态在一种建表法下判完的小结（登记 r2 5.4 T-行要比的两样）：第一次调用的结局，这个状态让哪几个计数器加了一。
pub(super) struct JudgedRowSummary {
    pub(super) first_outcome: ReplayOutcome,
    pub(super) counter_labels: String,
}

/// T-行：两遍的实现结局（`describe_outcome` 那一串）与这个状态加了一的计数器标签集合，有一项不同即不同（登记 r2 5.4）。
fn the_two_judged_rows_differ(direct: &JudgedRowSummary, scanned: &JudgedRowSummary) -> bool {
    describe_outcome(&direct.first_outcome) != describe_outcome(&scanned.first_outcome)
        || direct.counter_labels != scanned.counter_labels
}

/// 两张表各判一遍之后逐态比，比出来不同的记进 `tally` 的 `tables_differ_counter`、`rows_differ_counter`，各记第一个见证（状态的写法 + 差在哪）。
pub(super) fn add_the_comparison_of_two_judged_tables(
    tally: &mut Tally,
    (tables_differ_counter, rows_differ_counter): (Counter, Counter),
    direct: (&RecordTable, &JudgedRowSummary),
    scanned: (&RecordTable, &JudgedRowSummary),
    describe_state: &dyn Fn() -> String,
) {
    let (direct_table, direct_summary) = direct;
    let (scanned_table, scanned_summary) = scanned;
    if the_two_record_tables_differ(direct_table, scanned_table) {
        tally.add_violation(tables_differ_counter, &|| {
            format!(
                "{};{}",
                describe_state(),
                describe_how_the_two_record_tables_differ(direct_table, scanned_table)
            )
        });
    }
    if the_two_judged_rows_differ(direct_summary, scanned_summary) {
        tally.add_violation(rows_differ_counter, &|| {
            format!(
                "{};direct_outcome={};scanned_outcome={};direct_counters={};scanned_counters={}",
                describe_state(),
                describe_outcome(&direct_summary.first_outcome),
                describe_outcome(&scanned_summary.first_outcome),
                direct_summary.counter_labels,
                scanned_summary.counter_labels
            )
        });
    }
}

/// 扫盘那一遍用哪种实现、算不算指定状态：T-行那条对照把扫盘那一遍的结局换成 panic（表不动），别的情形与直接建表那一遍相同。
pub(super) fn implementation_and_designation_of_the_scan_pass(
    implementation: Implementation,
    is_the_designated_state: bool,
) -> (Implementation, bool) {
    match implementation {
        Implementation::Control(ControlKind::ScanPassOutcomeBecomesAPanic) => (
            Implementation::Control(ControlKind::PanicsOnTheDesignatedState),
            true,
        ),
        Implementation::TodayReplayJournal | Implementation::Control(_) => {
            (implementation, is_the_designated_state)
        }
    }
}

/// 一个状态判完之后「这个状态让哪几个计数器加了一」：对着判之前的计数。
pub(super) fn counter_labels_a_state_added_to(tally: &Tally, counts_before: &[u64]) -> String {
    labels_of_the_counters_a_state_added(counts_before, &tally.counts)
}

/// 每条记录在每块盘上都落了（登记 r2 第七节甲类「全部落盘那一态」）：0 号盘上都在，多盘域 1 号盘上的镜像也都在。
fn every_record_is_landed_on_every_device(state: &LegalState<'_>) -> bool {
    state
        .presence
        .iter()
        .all(|record_presence| *record_presence != RecordPresence::NotOnDisk)
        && (0..state.presence.len()).all(|record_index| match state.mirroring {
            JournalMirroring::FirstDeviceOnly => true,
            JournalMirroring::BothDevices {
                second_device_copy_is_on_disk,
            } => second_device_copy_is_on_disk[record_index],
        })
}

/// 登记 r2 第七节甲类：每条记录在每块盘上都落了的那一态，扫盘表的条数等于这一态生成的记录条数（R 域的残留与 0 号盘那一份同键，后到不进表）；
/// 不符走 SF9（停下，两边都查）。只在今天的实现上断言：T-表那条对照故意删一条。
pub(super) fn assert_the_scan_table_of_a_fully_landed_state(
    state: &LegalState<'_>,
    scanned_table: &RecordTable,
    implementation: Implementation,
    tally: &mut Tally,
) {
    if implementation != Implementation::TodayReplayJournal
        || !every_record_is_landed_on_every_device(state)
    {
        return;
    }
    tally.add(Counter::FullyLandedScanTableSizeChecked);
    assert_eq!(
        scanned_table.len(),
        state.history.slots.len(),
        "停机条款 SF9：每条记录在每块盘上都落了的那一态，扫盘表的条数不等于生成的记录条数（形状 {}）",
        state.history.shape_description
    );
}

/// 一个状态判完的结果：返回给 C、MC 域状态对要用的施加区间，加上第一次调用的结局与它落到的最远版本（逐态行的写法要）。
pub(super) struct LegalJudgement {
    pub(super) interval: Option<AppliedInterval>,
    pub(super) first_outcome: ReplayOutcome,
    pub(super) farthest_version: usize,
}

/// 一条记录「完整在盘上」：在 0 号盘上读得出，且（U 域）它点名的单元两份都读得出、校验和都对（登记 5.3）；
/// 多盘域任一份在即在（登记 r2 5.3，D23（journal 的角色与格式） 已定项 14「两份镜像何时算「记录在」：任一份自证校验和过即在」）。
fn record_is_completely_on_disk(state: &LegalState<'_>, record_index: usize) -> bool {
    let first_device_copy_is_complete = match state.presence[record_index] {
        RecordPresence::NotOnDisk => false,
        RecordPresence::OnDisk(unit_state) => unit_state == UnitState::BothCopiesGood,
    };
    first_device_copy_is_complete || second_device_copy_is_on_disk(state, record_index)
}

/// 1 号盘上那一份落没落；单盘域恒为假。
pub(super) fn second_device_copy_is_on_disk(state: &LegalState<'_>, record_index: usize) -> bool {
    match state.mirroring {
        JournalMirroring::FirstDeviceOnly => false,
        JournalMirroring::BothDevices {
            second_device_copy_is_on_disk,
        } => second_device_copy_is_on_disk[record_index],
    }
}

/// 这条记录有没有一份落了盘（不看点名单元）。
pub(super) fn record_has_a_copy_on_disk(state: &LegalState<'_>, record_index: usize) -> bool {
    state.presence[record_index] != RecordPresence::NotOnDisk
        || second_device_copy_is_on_disk(state, record_index)
}

pub(super) fn publish_is_completely_on_disk(state: &LegalState<'_>, publish_index: usize) -> bool {
    state
        .history
        .record_indexes_of_publish(publish_index)
        .all(|record_index| record_is_completely_on_disk(state, record_index))
}

pub(super) fn publish_has_a_record_on_disk(state: &LegalState<'_>, publish_index: usize) -> bool {
    state
        .history
        .record_indexes_of_publish(publish_index)
        .any(|record_index| record_has_a_copy_on_disk(state, record_index))
}

/// j\*（登记 5.3 的 J4）：最大的 j ≥ c，使 (c, j] 里每次发布都完整在盘上、且这些发布的记录条数之和不超过在飞上限；
/// 只数所选实例的发布。`in_flight_limit` 为 `None` 时不看上限（只给阳性对照「不看上限」用）。
pub(super) fn farthest_version_of(state: &LegalState<'_>, in_flight_limit: Option<usize>) -> usize {
    let mut farthest_version = state.chosen_version;
    let mut record_count_so_far = 0usize;
    for version in state.chosen_version + 1..state.history.chosen_instance_publish_count {
        if !publish_is_completely_on_disk(state, version) {
            break;
        }
        record_count_so_far += state.history.publishes[version].record_count;
        if in_flight_limit.is_some_and(|limit| record_count_so_far > limit) {
            break;
        }
        farthest_version = version;
    }
    farthest_version
}

/// (chosen_version, version] 里各次发布的记录条数之和。
pub(super) fn record_count_of_versions_after_the_chosen_one(
    state: &LegalState<'_>,
    version: usize,
) -> usize {
    (state.chosen_version + 1..=version)
        .map(|applied_version| state.history.publishes[applied_version].record_count)
        .sum()
}

#[cfg(test)]
mod tests;
