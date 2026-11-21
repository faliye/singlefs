//! 逐盘扫 journal 环：自证记录、提交标记、回链，与恢复的读者眼里的环。

use super::*;

/// journal 记录的 jsn 计数器从 1 起、全池接着走（D23（journal 的角色与格式） 已定项 9 / 已定项 19 注 3：
/// 新实例从前缀末 + 1 接着写、不归零）。
pub(super) const FIRST_JOURNAL_COUNTER: u64 = 1;

/// 记录头里提交标记那 1 字节（D23（journal 的角色与格式） 已定项 7，事务号之后那 1 字节）解出来的样子：
/// 0 不带、1 带，别的值是盘上读到的未知编码——做成显式成员，不并进「不带」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CommitMarker {
    Absent,
    Present,
    Unrecognized(u8),
}

impl CommitMarker {
    pub(super) const fn of(byte: u8) -> Self {
        match byte {
            0 => Self::Absent,
            1 => Self::Present,
            unrecognized => Self::Unrecognized(unrecognized),
        }
    }
}

/// 扫描方向读到的一条 journal 记录：它的实例代号、事务号、落在哪个环槽、反向链字段，以及「下一条记录的反向链该等于的值」。
#[derive(Clone)]
pub(super) struct ScannedJournalRecord {
    pub(super) instance: u32,
    /// 记录头里的 checkpoint_txg：这条记录施加出来的是哪一版（「由记录施加出来、根槽从没落盘的那一版」按它认）。
    pub(super) checkpoint_txg: u64,
    /// 记录头里的新根段 373 字节原样（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8 + 实例表指针 86 + 分配记录树根指针 86 + 卸载记号 4 + 回收门槛 t 8 + 钉槽 p 1，
    /// D23（journal 的角色与格式） 已定项 4 / 已定项 15）：
    /// 这条记录施加出来的那一版从哪个树表、哪个映射根走下去。
    pub(super) new_root_segment: Vec<u8>,
    /// 记录头里的事务号（D23（journal 的角色与格式） 已定项 7）：I-8.7（实例内事务号不重号） 判的就是它。
    pub(super) transaction: u64,
    /// 记录头里的提交标记：I-8.8（前缀里的事务不被切开） 判的就是它。
    pub(super) commit_marker: CommitMarker,
    /// 记录头里的本次发布内序号（D23（journal 的角色与格式） 已定项 4）原样：I-8.9（一次发布的记录序号连续且只有末条带标志） 判它。
    pub(super) ordinal_within_publish: u32,
    /// 记录标志那 1 字节原样（D23（journal 的角色与格式） 已定项 4 / 已定项 17）：I-8.9 判它的位 0 与其余位。
    pub(super) record_flags_byte: u8,
    pub(super) slot: u64,
    pub(super) back_chain: u32,
    /// CRC32C(这一条的 496 字节头，`header_csum` 那 32 字节按零参与)——I-8.6（反向链算法） 的算式。
    pub(super) chain_value_the_next_record_must_carry: u32,
    /// 每个点名项的两条位置条目（D23（journal 的角色与格式） 已定项 17）：I-2.5（位置条目按设备身份升序） 逐项判。
    pub(super) named_entry_locations: Vec<[PointerLocation; 2]>,
    /// 点名项解出来的样子（类标签、出生树、诞生代号、key 尾段与两条位置条目）：I-8.11（点名项与被指单元头相同） 逐项判。
    pub(super) named_entries: Vec<NamedEntryView>,
}

/// 一块盘的 journal 环里一条自证过的记录的键：(实例代号, 计数器)，与恢复扫环建表同一个键（`crates/singlefs-core/src/recovery.rs`
/// 的 `scan_journal`，留哪一份在 `singlefs_replay_prefix::first_copy_of_each_record_key`）。字段次序计数器在前：派生的次序先按计数器、
/// 同一计数器再按实例代号，逐盘「按 jsn 计数器从小到大」走的那几判（I-8.7、I-8.8、I-8.9、I-8.12）照旧按计数器走。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct JournalRecordKeyOnADevice {
    pub(super) counter: u64,
    pub(super) instance: u32,
}

impl JournalRecordKeyOnADevice {
    /// 计数器 `counter` 上次序最小的键：按计数器取一段时当下界。
    const fn first_at_counter(counter: u64) -> Self {
        Self {
            counter,
            instance: u32::MIN,
        }
    }

    /// 计数器 `counter` 上次序最大的键：按计数器取一段时当上界。
    const fn last_at_counter(counter: u64) -> Self {
        Self {
            counter,
            instance: u32::MAX,
        }
    }
}

/// 一块盘的 journal 环里自证过的记录，按 (实例代号, 计数器) 索引（[`JournalRecordKeyOnADevice`]）。
pub(super) type JournalRecordsOfADevice = BTreeMap<JournalRecordKeyOnADevice, ScannedJournalRecord>;

/// 一块盘上计数器 `counter` 上自证过的记录（合法写者每个计数器至多一条；冲突布局上可以有几个实例的各一条）。
pub(super) fn records_at_counter(
    records: &JournalRecordsOfADevice,
    counter: u64,
) -> impl Iterator<Item = (&JournalRecordKeyOnADevice, &ScannedJournalRecord)> {
    records.range(
        JournalRecordKeyOnADevice::first_at_counter(counter)
            ..=JournalRecordKeyOnADevice::last_at_counter(counter),
    )
}

/// 一块盘上计数器在 [`from_counter`, `to_counter`) 里的记录，按计数器从小到大。
pub(super) fn records_with_counters_between(
    records: &JournalRecordsOfADevice,
    from_counter: u64,
    to_counter: u64,
) -> impl Iterator<Item = (&JournalRecordKeyOnADevice, &ScannedJournalRecord)> {
    records.range(
        JournalRecordKeyOnADevice::first_at_counter(from_counter)
            ..JournalRecordKeyOnADevice::first_at_counter(to_counter),
    )
}

/// 一块盘上计数器不小于 `from_counter` 的记录，按计数器从小到大。
pub(super) fn records_from_counter(
    records: &JournalRecordsOfADevice,
    from_counter: u64,
) -> impl Iterator<Item = (&JournalRecordKeyOnADevice, &ScannedJournalRecord)> {
    records.range(JournalRecordKeyOnADevice::first_at_counter(from_counter)..)
}

/// 一块盘的 journal 环里一条自证过的记录（magic、两道校验和、记录类型与长度、fsid 是本池的、卸载记号只有位 0 都过：池级 checker 的记录读者
/// `check_journal_record` 收下的，加上只因点名项 flags 字节非 0 被它拒的，[`self_verified_journal_record_read_past_the_named_entry_flags`]）
/// 坐在哪个环槽、计数器是几：I-8.13 逐条判，不经按键去重的表（同盘同键的两条都要判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SelfVerifiedJournalRecordInARingSlot {
    ring_slot: u64,
    pub(super) counter: u64,
}

/// 一块盘的 journal 环扫一遍得到的三份（[`scanned_journal_ring_of_device`]）。
struct ScannedJournalRingOfADevice {
    /// 池级 checker 的记录读者收得下的记录，键 (实例代号, 计数器)、同键先到先留：I-8.6 至 I-8.12 判它。恢复的读者不收的那几种
    /// （[`journal_record_is_read_by_the_recovery_reader`]）照旧收在这里，由已有的那几判（I-8.8 的提交标记字节、I-8.9 的序号与记录标志）判红。
    /// 只因点名项 flags 字节非 0 被池级 checker 的记录读者拒的不收（与收录口径对齐之前相同）。
    pub(super) records: JournalRecordsOfADevice,
    /// 只在恢复的记录读者也收得下的记录上建、同键先到先留：恢复眼里的环（[`records_as_the_recovery_reads_them`]）读它，
    /// 与 core 的 `scan_journal`（扫环时读不出的记录根本收不进来）同一个选法（`.claude/kb/checks-owed.md` C897 那一行后半）。
    /// 只因点名项 flags 字节非 0 被池级 checker 的记录读者拒的，恢复的读者跳过那一字节照收，这里也收（同 C02 的验收：
    /// 池级 checker 认的恢复视图与 core 扫出的逐条相同）。
    records_the_recovery_reader_reads: JournalRecordsOfADevice,
    /// 每条自证过的记录坐的环槽与它的计数器，按扫环的次序：I-8.13。
    self_verified_records: Vec<SelfVerifiedJournalRecordInARingSlot>,
    /// 读不出的环槽（读不到，或不全零而不自证），按扫环的次序：I-8.14 有它就报部分没判（全零的槽可以是从没写过的，不算）。
    slots_that_do_not_read_back: Vec<u64>,
}

/// 逐盘扫 journal 环得到的三份，按 `reader.devices()` 的次序逐盘一份（[`scanned_journal_rings_by_device`]）。
pub(super) struct ScannedJournalRings {
    /// 判 journal 那几条不变量读的表（[`ScannedJournalRingOfADevice::records`]）。
    pub(super) records_by_device: Vec<(u32, JournalRecordsOfADevice)>,
    /// 恢复视图读的表（[`ScannedJournalRingOfADevice::records_the_recovery_reader_reads`]）。
    pub(super) records_the_recovery_reader_reads_by_device: Vec<(u32, JournalRecordsOfADevice)>,
    /// I-8.13 逐条判的落点（[`ScannedJournalRingOfADevice::self_verified_records`]）。
    pub(super) self_verified_records_by_device:
        Vec<(u32, Vec<SelfVerifiedJournalRecordInARingSlot>)>,
    /// I-8.14 看的读不出的环槽（[`ScannedJournalRingOfADevice::slots_that_do_not_read_back`]）。
    pub(super) slots_that_do_not_read_back_by_device: Vec<(u32, Vec<u64>)>,
}

/// 恢复的记录读者（`crates/singlefs-core/src/journal.rs` 的 `JournalRecord::parse`）收不收一条池级 checker 的记录读者已经收下的记录。
/// 恢复的读者另当损坏、读不出的四种（D23（journal 的角色与格式） 已定项 4 / 已定项 7 / 已定项 17，D19（块指针的结构与宽度预算） 已定项 3），
/// 有一种就不收：① 提交标记字节不是 0 / 1；② 记录标志位 0 之外有位；③ 本次发布内序号是 0；④ 新根段里四条指针（树表、映射根、实例表、
/// 分配记录树根）头部 MAC / nonce、算法类型、压缩算法码、压后长度、extent 偏移有一段不是第一版的取值
/// （[`pointer_head_fields_take_the_first_version_values`]）。池级 checker 不依赖 core（D13（验证路线） 已定项 5），这一份按条款另写，
/// core 的读者改了收录口径这里跟着改。
fn journal_record_is_read_by_the_recovery_reader(record: &crate::JournalRecordView) -> bool {
    let commit_marker_recognized = match CommitMarker::of(record.commit_marker_byte) {
        CommitMarker::Absent | CommitMarker::Present => true,
        CommitMarker::Unrecognized(_) => false,
    };
    commit_marker_recognized
        && record.record_flags_byte & !RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH == 0
        && record.ordinal_within_publish != 0
        && [
            NEW_ROOT_SEGMENT_TREE_TABLE_POINTER,
            NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER,
            NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER,
            NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER,
        ]
        .into_iter()
        .all(|pointer_in_the_new_root_segment| {
            pointer_head_fields_take_the_first_version_values(&parse_node_pointer(
                &record.new_root_segment[pointer_in_the_new_root_segment],
            ))
        })
}

/// 点名项 56 字节里 flags 那 1 字节的偏移（D23（journal 的角色与格式） 已定项 17：位置条目 14 × 2 + 类标签 1 + 出生树 8 + 出生 txg 8 +
/// key 尾段 10 之后）。
const JOURNAL_NAMED_ENTRY_FLAGS_OFFSET: usize = 55;

/// 池级 checker 的记录读者（`check_journal_record`）怎么判一条自证过的记录
/// （[`self_verified_journal_record_read_past_the_named_entry_flags`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PoolCheckerReaderVerdictOnAJournalRecord {
    /// 收下。
    Reads,
    /// 只因点名项 flags 字节非 0 拒（`Verdict::NonZeroFlags`）：magic、两道校验和、记录类型与长度、fsid、卸载记号那几关都过了。
    RefusesOnlyForNonZeroNamedEntryFlags,
}

/// 环里一个槽读出来的字节当一条自证过的 journal 记录读（magic、两道校验和、记录类型与长度、fsid 是本池的、卸载记号只有位 0），
/// 点名项 flags 那 1 字节不看：恢复的记录读者（`crates/singlefs-core/src/journal.rs` 的 `NamedUnit::read_from`）跳过它照收，
/// 池级 checker 的记录读者（`check_journal_record`）判 `Verdict::NonZeroFlags` 拒（`.claude/kb/checks-owed.md` C897 那一行后半，
/// 收录口径反方向的那一处）。被它这样拒的，照恢复的读法收下：每个点名项的 flags 清零、两道校验和照池级 checker 自己的算法重封，
/// 再交给同一个 `check_journal_record` 读——视图里的字段与盘上逐字节相同（视图不带 flags），反向链要的头 CRC 调用方照旧从盘上那一份算。
/// 交回视图与池级 checker 的记录读者怎么判它；别的理由拒的交回 `None`（magic、校验和、类型、长度、fsid、卸载记号那几关两个读者判法相同，
/// 两个都不收）。
fn self_verified_journal_record_read_past_the_named_entry_flags(
    bytes: &[u8],
    filesystem_identifier_low: u64,
) -> Option<(
    crate::JournalRecordView,
    PoolCheckerReaderVerdictOnAJournalRecord,
)> {
    let verdict = match check_journal_record(bytes, filesystem_identifier_low) {
        Ok(record) => return Some((record, PoolCheckerReaderVerdictOnAJournalRecord::Reads)),
        Err(verdict) => verdict,
    };
    if !matches!(verdict, Verdict::NonZeroFlags) {
        return None;
    }
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let header_bytes = usize::try_from(singlefs_format::JOURNAL_HEADER_BYTES).expect("496");
    let entry_bytes = usize::try_from(singlefs_format::JOURNAL_NAMED_ENTRY_BYTES).expect("56");
    let named_count = usize::try_from(read_u32(bytes, 12)).expect("点名项数");
    let payload_end = header_bytes + named_count * entry_bytes;
    let mut named_entry_flags_cleared = bytes[..record_bytes].to_vec();
    for index in 0..named_count {
        named_entry_flags_cleared
            [header_bytes + index * entry_bytes + JOURNAL_NAMED_ENTRY_FLAGS_OFFSET] = 0;
    }
    let payload_checksum =
        crc32_castagnoli_table(&named_entry_flags_cleared[header_bytes..payload_end]);
    named_entry_flags_cleared
        [crate::JOURNAL_PAYLOAD_CHECKSUM_OFFSET..crate::JOURNAL_PAYLOAD_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    let header_checksum_field = crate::JOURNAL_HEADER_CHECKSUM_OFFSET
        ..crate::JOURNAL_HEADER_CHECKSUM_OFFSET
            + usize::try_from(singlefs_format::WIDE_CHECKSUM_BYTES).expect("32");
    named_entry_flags_cleared[header_checksum_field].fill(0);
    let header_checksum = crc32_castagnoli_table(&named_entry_flags_cleared);
    named_entry_flags_cleared
        [crate::JOURNAL_HEADER_CHECKSUM_OFFSET..crate::JOURNAL_HEADER_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&header_checksum.to_le_bytes());
    let record = check_journal_record(&named_entry_flags_cleared, filesystem_identifier_low).expect(
        "check_journal_record 判 NonZeroFlags 之前 magic、两道校验和、类型、长度、fsid、卸载记号与点名项不越过记录末尾都过了；清掉 flags、照同一种算法重封两道校验和，那几关原样过，flags 那一关也过",
    );
    Some((
        record,
        PoolCheckerReaderVerdictOnAJournalRecord::RefusesOnlyForNonZeroNamedEntryFlags,
    ))
}

/// 逐盘扫一遍 journal 环：键是 (实例代号, 计数器)，同一个键按扫环的次序（`candidate_journal_slots` 交回的次序，没有就环槽号升序）
/// 先到先留——与恢复扫环同一个选法（`crates/singlefs-core/src/recovery.rs` 的 `scan_journal` 按盘次序、环内偏移次序收，
/// 同一 (实例代号, 计数器) 第一份读到的留下；`records/2026-10-24-singlefs-checker-审计复核.md` 的 C02：只按计数器建键、后到覆盖先到时，
/// 同一计数器两条记录 checker 留后到的、恢复留先到的，同一计数器两个实例的记录 checker 丢掉一条）。
/// 恢复视图那一份只在恢复的读者也收得下的记录上先到先留：同盘同键先到的是恢复的读者不收的、后到的是好记录时，恢复留后到的那条，
/// 判不变量那一份照旧留先到的那条（它的提交标记字节、序号与记录标志照判）。点名项 flags 字节非 0 的记录反过来：恢复的读者收、
/// 池级 checker 的记录读者拒，恢复视图那一份与 I-8.13 的落点收它，判不变量那一份不收。
/// 合法写者的计数器与环槽一一对应（记录 n 落 `(计数器 − 1) mod 槽数 × 4096`，D23（journal 的角色与格式） 已定项 18），同一块盘上不会有两条同键；
/// 不在该落的槽上的记录由 I-8.13 判红（[`judge_journal_records_live_in_the_ring_slot_their_counter_names`]）。
fn scanned_journal_ring_of_device(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    device: u32,
    filesystem_identifier_low: u64,
) -> ScannedJournalRingOfADevice {
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let ring_slot_count = geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
    // 读者交的候选宁多不少（`ImageReader::candidate_journal_slots`）：盘上系统配置槽 0 自证不过时 harness 的读者从环起点罩到盘尾，
    // 候选里有环之外的槽（单元区里写过的那几槽）。它们不是 journal 环的槽：照恢复扫环与 GPU 判表的记录落点（都只在 [环起点, 环起点 + 环长) 里）只扫环里的，
    // 环外那几槽不收记录，也不当 I-8.14 眼里读不出的环槽。
    let slots: Vec<u64> = reader
        .candidate_journal_slots(device)
        .unwrap_or_else(|| (0..ring_slot_count).collect())
        .into_iter()
        .filter(|slot| *slot < ring_slot_count)
        .collect();
    let mut ring = ScannedJournalRingOfADevice {
        records: JournalRecordsOfADevice::new(),
        records_the_recovery_reader_reads: JournalRecordsOfADevice::new(),
        self_verified_records: Vec::new(),
        slots_that_do_not_read_back: Vec::new(),
    };
    for slot in slots {
        let offset = geometry.journal_ring_start_slot * SLOT_BYTES + slot * JOURNAL_RECORD_BYTES;
        let Some(bytes) = reader.read(device, offset, record_bytes) else {
            ring.slots_that_do_not_read_back.push(slot);
            continue;
        };
        let Some((record, pool_checker_reader_verdict)) =
            self_verified_journal_record_read_past_the_named_entry_flags(
                &bytes,
                filesystem_identifier_low,
            )
        else {
            if bytes.iter().any(|byte| *byte != 0) {
                ring.slots_that_do_not_read_back.push(slot);
            }
            continue;
        };
        ring.self_verified_records
            .push(SelfVerifiedJournalRecordInARingSlot {
                ring_slot: slot,
                counter: record.counter,
            });
        let is_read_by_the_recovery_reader = journal_record_is_read_by_the_recovery_reader(&record);
        let key = JournalRecordKeyOnADevice {
            counter: record.counter,
            instance: record.instance,
        };
        let scanned = ScannedJournalRecord {
            instance: record.instance,
            checkpoint_txg: record.checkpoint_txg,
            new_root_segment: record.new_root_segment,
            transaction: record.transaction,
            commit_marker: CommitMarker::of(record.commit_marker_byte),
            ordinal_within_publish: record.ordinal_within_publish,
            record_flags_byte: record.record_flags_byte,
            slot,
            back_chain: record.back_chain,
            chain_value_the_next_record_must_carry: back_chain_of_record_header(&bytes),
            named_entry_locations: record
                .named
                .iter()
                .map(|named| {
                    named
                        .locations
                        .map(|(named_device, named_slot, checksum)| PointerLocation {
                            device: named_device,
                            slot: named_slot,
                            checksum,
                        })
                })
                .collect(),
            named_entries: record.named,
        };
        if is_read_by_the_recovery_reader {
            ring.records_the_recovery_reader_reads
                .entry(key)
                .or_insert_with(|| scanned.clone());
        }
        match pool_checker_reader_verdict {
            PoolCheckerReaderVerdictOnAJournalRecord::Reads => {
                ring.records.entry(key).or_insert(scanned);
            }
            // 判不变量那张表照旧只收池级 checker 的记录读者收下的：这种记录同盘同键先到时不挤掉后到的那条，后到那条的各判一格不少。
            PoolCheckerReaderVerdictOnAJournalRecord::RefusesOnlyForNonZeroNamedEntryFlags => {}
        }
    }
    ring
}

/// I-2.5（位置条目按设备身份升序） 的「journal 点名项里的位置条目数组」那一半（代码审阅 6c 第 9 条：点名项解出来就丢了）：
/// 环里每条自证过的记录（两块盘各一份，逐盘判：一块盘上的写错了另一块不该跟着错）的每个点名项判一次。
pub(super) fn judge_location_order_of_journal_named_entries(
    records_by_device: &[(u32, JournalRecordsOfADevice)],
    judgements: &mut Judgements,
) {
    for (device, records) in records_by_device {
        for (key, record) in records {
            for (index, locations) in record.named_entry_locations.iter().enumerate() {
                judge_location_entries_order(
                    judgements,
                    locations,
                    &format!(
                        "盘 {device} journal 环槽 {} 的记录（计数器 {}）第 {index} 个点名项",
                        record.slot, key.counter
                    ),
                );
            }
        }
    }
}

/// I-8.6（反向链算法） 对一条记录（一块盘上的那一份）该有的反向链：按「本实例内逻辑前一条」= 同一实例、计数器小 1 的那一条找。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpectedBackChain {
    /// 计数器 1（全池第一条，必是本实例第一条）、或计数器 − 1 那一条是别的实例的（这一条是本实例第一条）⇒ 0；
    /// 计数器 − 1 那一条是同一实例的 ⇒ 它的头算出来的链值。
    Value(u32),
    /// 计数器 0：计数器从 1 起（D23（journal 的角色与格式） 已定项 18「记录 n 落在 (计数器 − 1) mod 槽数」），这一格没有逻辑前一条，
    /// 链无从定义——判违例，记在 I-8.6 下。
    CounterZeroHasNoLogicalPredecessor,
    /// 计数器 − 1 那一条读不出、自证不过、或环转过一圈之后那一格坐着上一圈的记录：本实例内逻辑前一条不在这块盘上，判不了。
    PredecessorNotOnThisDevice,
}

/// 按 (实例代号, 计数器 − 1) 找本实例内逻辑前一条（[`JournalRecordKeyOnADevice`]）：在 ⇒ 它的链值；不在而这块盘上计数器 − 1 坐着
/// 别的实例的记录 ⇒ 这一条是本实例第一条、链值 0；都没有 ⇒ 判不了。合法写者每个计数器至多一条，与只按计数器找同一个结果；
/// 计数器 − 1 上同时坐着本实例与别的实例的记录（冲突布局）时取本实例那一条（I-8.6 那一行「本实例内逻辑前一条」）。
fn expected_back_chain_of(
    records: &JournalRecordsOfADevice,
    key: JournalRecordKeyOnADevice,
) -> ExpectedBackChain {
    match key.counter {
        0 => ExpectedBackChain::CounterZeroHasNoLogicalPredecessor,
        FIRST_JOURNAL_COUNTER => ExpectedBackChain::Value(0),
        later_counter => {
            let previous_counter = later_counter
                .checked_sub(1)
                .expect("计数器 0 与 1 两支上面已分走：这一支的计数器至少是 2");
            let previous_of_the_same_instance = records.get(&JournalRecordKeyOnADevice {
                counter: previous_counter,
                instance: key.instance,
            });
            match previous_of_the_same_instance {
                Some(previous) => {
                    ExpectedBackChain::Value(previous.chain_value_the_next_record_must_carry)
                }
                None => {
                    if records_at_counter(records, previous_counter)
                        .next()
                        .is_some()
                    {
                        ExpectedBackChain::Value(0)
                    } else {
                        ExpectedBackChain::PredecessorNotOnThisDevice
                    }
                }
            }
        }
    }
}

/// 恢复眼里的 journal 环，也是重放前缀读的那一份环：键是 (实例代号, 计数器)，每个键取按盘的次序（`reader.devices()`，与恢复扫环按
/// `device_identities()` 的次序相同）第一块盘上恢复的记录读者收得下的那一份（「两份镜像任一份自证校验和过即在」，D23（journal 的角色与格式）
/// 已定项 14）——与实现扫环建表同一个键、同一个选法（`crates/singlefs-core/src/recovery.rs` 的 `scan_journal` 按 (实例代号, 计数器) 先到先得，
/// 读不出的记录收不进来）。`records_the_recovery_reader_reads_by_device` 是 [`ScannedJournalRings::records_the_recovery_reader_reads_by_device`]：
/// 恢复的读者不收的记录（[`journal_record_is_read_by_the_recovery_reader`]）不在里面，同盘同键先到的是这种、后到的是好记录时留后到的那条，
/// 与 core 相同（`.claude/kb/checks-owed.md` C897 那一行后半）。两块盘同一计数器坐着不同实例的记录时两条都留，不拿先读到的那块盘上别的实例那一条
/// 挤掉所选根那个实例的（I-4.1 那一行；`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 8 行）。
/// I-8.6（反向链算法） 那一判逐盘判两份（一块盘上的链坏了另一块不该跟着坏）；认「恢复施加过 / 下一次挂载会施加」哪几条记录时照恢复的读法只看这一份
/// （实审 A2a 报告「checker 要跟着改什么」第 1 条，`research/prompts/m2-rev-a2a-implementer-report.md`）。
pub(super) fn records_as_the_recovery_reads_them(
    records_the_recovery_reader_reads_by_device: &[(u32, JournalRecordsOfADevice)],
) -> BTreeMap<(u32, u64), &ScannedJournalRecord> {
    let mut first_copies: BTreeMap<(u32, u64), &ScannedJournalRecord> = BTreeMap::new();
    for (_, records) in records_the_recovery_reader_reads_by_device {
        for (key, record) in records {
            first_copies
                .entry((record.instance, key.counter))
                .or_insert(record);
        }
    }
    first_copies
}

/// 恢复眼里的一条 journal 记录：键 (实例代号, 计数器) 与它的 `checkpoint_txg`。给验证包比池级 checker 与 core 扫环选的是不是同一批记录
/// （[`journal_records_as_the_recovery_reads_them`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JournalRecordTheRecoveryReads {
    pub instance: u32,
    pub counter: u64,
    pub checkpoint_txg: u64,
}

/// 池级 checker 认的恢复视图（[`records_as_the_recovery_reads_them`]：逐盘扫环、键 (实例代号, 计数器)、同键按扫环次序先到先留，跨盘按盘次序取第一份），
/// 按键升序交回每条的键与 `checkpoint_txg`，只读不判。系统配置择不出（池里没有一块盘交得出可择的系统配置）时交回 `None`：
/// 环几何与文件系统身份都读不到。几何与文件系统身份取第一块择到系统配置的盘那一份，与 [`check_pool_image`] 走 journal 环同一个取法。
#[must_use]
pub fn journal_records_as_the_recovery_reads_them(
    reader: &dyn ImageReader,
) -> Option<Vec<JournalRecordTheRecoveryReads>> {
    let geometry = chosen_system_configurations(reader)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))?;
    let filesystem_identifier_low = u64::from_le_bytes(
        geometry.filesystem_identifier[..8]
            .try_into()
            .expect("8 字节"),
    );
    let records_the_recovery_reader_reads_by_device =
        scanned_journal_rings_by_device(reader, &geometry, filesystem_identifier_low)
            .records_the_recovery_reader_reads_by_device;
    Some(
        records_as_the_recovery_reads_them(&records_the_recovery_reader_reads_by_device)
            .into_iter()
            .map(
                |((instance, counter), record)| JournalRecordTheRecoveryReads {
                    instance,
                    counter,
                    checkpoint_txg: record.checkpoint_txg,
                },
            )
            .collect(),
    )
}

/// I-8.6「不等的记录不进重放前缀」（代码审阅 6c 第 25 条）：
/// 照恢复的判法（`crates/singlefs-core/src/recovery.rs` 的 `judge_back_chain_against_the_previous_record_on_disk`，实审 A2a）——
/// 本实例内逻辑前一条（同一实例、计数器小 1，在 [`records_as_the_recovery_reads_them`] 里按 (实例代号, 计数器 − 1) 找）在盘上、
/// 它的头算出来的链值与这一条的反向链不等，这一条进不了前缀。
/// 恢复只判这一种：前缀都接在所选根覆盖的那一条之后、同一实例，计数器 1、前一条是别的实例、前一条不在盘上都不判
/// （A2a 报告那一节第 2 条；I-8.6 那一判对前两种照条款判「链恒 0」，前缀这里不跟着判）。计数器 0 进不了前缀
/// （jsn 从所选根覆盖的那一条 + 1 起严格连续，计数器从 1 起，D23（journal 的角色与格式） 已定项 18）。
pub(super) fn record_can_enter_a_replay_prefix(
    records_as_the_recovery_reads_them: &BTreeMap<(u32, u64), &ScannedJournalRecord>,
    counter: u64,
    record: &ScannedJournalRecord,
) -> bool {
    let Some(previous_counter) = counter.checked_sub(1) else {
        return false;
    };
    records_as_the_recovery_reads_them
        .get(&(record.instance, previous_counter))
        .is_none_or(|previous| record.back_chain == previous.chain_value_the_next_record_must_carry)
}

/// 逐盘扫一遍 journal 环，把自证过的记录按盘收齐（[`ScannedJournalRings`]）：判 journal 的那几条不变量（I-8.6（反向链算法）、
/// I-8.7（实例内事务号不重号）、I-8.8（前缀里的事务不被切开）、I-8.9（一次发布的记录序号连续且只有末条带标志）、I-8.13（记录落在它的计数器该落的环槽））
/// 与恢复视图读的是同一遍扫环，扫一次一起判，不各扫一遍。
pub(super) fn scanned_journal_rings_by_device(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    filesystem_identifier_low: u64,
) -> ScannedJournalRings {
    let mut rings = ScannedJournalRings {
        records_by_device: Vec::new(),
        records_the_recovery_reader_reads_by_device: Vec::new(),
        self_verified_records_by_device: Vec::new(),
        slots_that_do_not_read_back_by_device: Vec::new(),
    };
    for device in reader.devices() {
        let ring =
            scanned_journal_ring_of_device(reader, geometry, device, filesystem_identifier_low);
        rings.records_by_device.push((device, ring.records));
        rings
            .records_the_recovery_reader_reads_by_device
            .push((device, ring.records_the_recovery_reader_reads));
        rings
            .self_verified_records_by_device
            .push((device, ring.self_verified_records));
        rings
            .slots_that_do_not_read_back_by_device
            .push((device, ring.slots_that_do_not_read_back));
    }
    rings
}

/// I-8.13（[`JOURNAL_RECORDS_LIVE_IN_THE_RING_SLOT_THEIR_COUNTER_NAMES`]）：逐盘，每条自证过的 journal 记录（池级 checker 的记录读者收得下，
/// fsid 是本池的）坐在它的计数器该落的环槽 `(计数器 − 1) mod 槽数`（D23（journal 的角色与格式） 已定项 18「记录 n 落在环内偏移
/// `(计数器 − 1) mod 槽数 × 4096`」），不在判红。槽数是所选系统配置的环长 ÷ 4096（与扫环同一份几何）。
/// 同一块盘上同 (实例代号, 计数器) 的两条至少一条不在该落的槽上，这一格连带判红（`.claude/kb/checks-owed.md` C897）；别的池（fsid 不同）
/// 留下的旧记录不自证、不判。计数器 0 没有该落的槽（落点式从计数器 1 起），不判，归 I-8.6 判红。一条要判的记录都没有时报不适用。
pub(super) fn judge_journal_records_live_in_the_ring_slot_their_counter_names(
    self_verified_records_by_device: &[(u32, Vec<SelfVerifiedJournalRecordInARingSlot>)],
    ring_slots: u64,
    judgements: &mut Judgements,
) {
    for (device, records) in self_verified_records_by_device {
        for record in records {
            let Some(ring_slot_the_counter_names) = record
                .counter
                .checked_sub(FIRST_JOURNAL_COUNTER)
                .and_then(|counter_from_zero| counter_from_zero.checked_rem(ring_slots))
            else {
                continue;
            };
            judgements.judge(
                JOURNAL_RECORDS_LIVE_IN_THE_RING_SLOT_THEIR_COUNTER_NAMES,
                record.ring_slot == ring_slot_the_counter_names,
                || {
                    format!(
                        "盘 {device} journal 环槽 {} 的记录计数器是 {}，它该落在环槽 {ring_slot_the_counter_names}（(计数器 − 1) mod 槽数 {ring_slots}）",
                        record.ring_slot, record.counter
                    )
                },
            );
        }
    }
    judgements.not_applicable(
        JOURNAL_RECORDS_LIVE_IN_THE_RING_SLOT_THEIR_COUNTER_NAMES,
        NO_SELF_VERIFIED_JOURNAL_RECORD_WITH_A_COUNTER_FROM_ONE,
    );
}

/// I-8.6（反向链算法）：任一 journal 记录的反向链 = CRC32C(**本实例内逻辑前一条**记录的 496 字节头，`header_csum` 那 32 字节
/// 按零参与)；**本实例写出的第一条记录反向链恒 0**。逐盘判：环两盘互为镜像，一块盘上的链坏了另一块不该跟着坏。
/// 「本实例内逻辑前一条」= 同一实例、计数器小 1 的那一条，三种情形穷举：
/// ① 计数器 == 1 ⇒ 全池第一条，必是本实例第一条 ⇒ 反向链恒 0；
/// ② 计数器 − 1 那一条在同一块盘上自证过、实例代号相同 ⇒ 反向链 = 它的头算出来的链值；
/// ③ 计数器 − 1 那一条自证过而实例代号不同 ⇒ 这一条是本实例写出的第一条（一个实例的计数器连续、接着上一个实例走，
///    D23（journal 的角色与格式） 已定项 19 注 3）⇒ 反向链恒 0。**跨实例边界的两条记录之间不比链值**——I-8.6 明写不判那一格。
/// 计数器 − 1 那一条读不出、自证不过、或环转过一圈之后那一格坐着上一圈的记录（计数器对不上）⇒ 本实例内逻辑前一条不在盘上，
/// 这一条判不了，跳过。
pub(super) fn judge_journal_back_chain(
    records_by_device: &[(u32, JournalRecordsOfADevice)],
    judgements: &mut Judgements,
) {
    let mut judged_records = 0u64;
    for (device, records) in records_by_device {
        for (key, record) in records {
            let counter = key.counter;
            // 计数器是盘上读来的 6 字节，自证过的记录也可以带 0（代码审阅 6c 第 6 条：减一在 0 上下溢，checker 曾当场 panic）；
            // 实审 B1 把它当判不了跳过判违例（`ExpectedBackChain::CounterZeroHasNoLogicalPredecessor`）。
            let expected_back_chain = match expected_back_chain_of(records, *key) {
                ExpectedBackChain::Value(expected_back_chain) => expected_back_chain,
                ExpectedBackChain::CounterZeroHasNoLogicalPredecessor => {
                    judged_records += 1;
                    judgements.judge("I-8.6", false, || {
                        format!(
                            "盘 {device} journal 环槽 {} 的记录（实例 {}）计数器 0：计数器从 1 起，这一格没有本实例内逻辑前一条，反向链无从定义",
                            record.slot, record.instance
                        )
                    });
                    continue;
                }
                ExpectedBackChain::PredecessorNotOnThisDevice => continue,
            };
            judged_records += 1;
            judgements.judge("I-8.6", record.back_chain == expected_back_chain, || {
                format!(
                    "盘 {device} journal 环槽 {} 的记录（实例 {}、计数器 {counter}）反向链 {:#010x}，本实例内逻辑前一条的头算出来的是 {expected_back_chain:#010x}",
                    record.slot, record.instance, record.back_chain
                )
            });
        }
    }
    if judged_records == 0 {
        judgements.not_applicable(
            "I-8.6",
            "环里没有一条自证过的记录找得到本实例内逻辑前一条：链判不了",
        );
    }
}

#[cfg(test)]
mod tests;
