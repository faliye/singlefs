//! 扫环胶水的几臂：直接调 `scan_journal`、收齐之后不释放、逐盘合并与逐盘不释放，记录表的比较与摘要。
use super::*;

// ============================================================================================
// 胶水的各臂（登记第五节）。基线臂直接调 `scan_journal`；其余三份胶水照 `crates/singlefs-core/src/recovery.rs` 的 `scan_journal`
// 函数体（S1 钉了它的 sha256）抄进来，只改登记写明的那一处；建表的核一律是不改的 `first_copy_of_each_record_key`。
// ============================================================================================

pub(super) type RecordTable = BTreeMap<(InstanceGeneration, u64), JournalRecord>;

/// `scan_journal` 开头从系统配置里取的三样。
struct ScanSetup {
    expected_filesystem_identifier: u64,
    ring_start: DeviceOffsetInBytes,
    ring_bytes: u64,
}

fn scan_setup(system_configuration: &SystemConfiguration) -> ScanSetup {
    ScanSetup {
        expected_filesystem_identifier: unit_filesystem_identifier(
            &system_configuration.immutable.filesystem_identifier,
        ),
        ring_start: DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES),
        ring_bytes: system_configuration.immutable.sizes.journal_ring_bytes,
    }
}

fn key_of(record: &JournalRecord) -> (u32, u64) {
    (record.instance.0, record.counter)
}

/// `scan_journal` 里「对一块盘逐槽列偏移、逐槽读、解得过的记录压进 Vec」那一截，逐句照抄（`recovery.rs` 4436–4467 行）。
fn read_the_records_of_one_device(
    reader: &dyn PoolReader,
    device: DeviceIdentity,
    setup: &ScanSetup,
    destination: &mut Vec<JournalRecord>,
) {
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let ring_bytes_inside_the_device =
        reader
            .device_size_in_bytes(device)
            .map_or(0, |device_bytes| {
                setup
                    .ring_bytes
                    .min(device_bytes.saturating_sub(setup.ring_start.0))
            });
    let offsets = reader
        .journal_record_offsets_hint(device, setup.ring_start, setup.ring_bytes)
        .unwrap_or_else(|| {
            (0..ring_bytes_inside_the_device / JOURNAL_RECORD_BYTES)
                .map(|record_index| {
                    DeviceOffsetInBytes(setup.ring_start.0 + record_index * JOURNAL_RECORD_BYTES)
                })
                .collect()
        });
    for offset in offsets {
        let Some(bytes) = reader.read(device, offset, record_bytes) else {
            continue;
        };
        if let Some(record) = JournalRecord::parse(&bytes, setup.expected_filesystem_identifier) {
            destination.push(record);
        }
    }
}

/// baseline_pc：与 `scan_journal` 同一份胶水，只改最后一步——不 `into_iter` move，按引用遍历、把首份 `clone()` 进表；读到的 Vec 留到交回之前才放。
pub(super) fn scan_journal_keeping_the_scan_until_return(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
) -> RecordTable {
    let setup = scan_setup(system_configuration);
    let mut scanned_records_in_read_order: Vec<JournalRecord> = Vec::new();
    for device in reader.device_identities() {
        read_the_records_of_one_device(reader, device, &setup, &mut scanned_records_in_read_order);
    }
    let scanned_record_keys: Vec<(u32, u64)> =
        scanned_records_in_read_order.iter().map(key_of).collect();
    let first_copies = first_copy_of_each_record_key(&scanned_record_keys);
    let mut records = BTreeMap::new();
    for (position, record) in scanned_records_in_read_order.iter().enumerate() {
        if first_copies.get(&(record.instance.0, record.counter)) == Some(&position) {
            records.insert((record.instance, record.counter), record.clone());
        }
    }
    std::hint::black_box(&scanned_records_in_read_order);
    records
}

/// split（乙二）：核不改、胶水逐盘合并。每读完一盘：这一盘的键接到全局键表后面；先放掉上一盘调核交回的表，再拿到此为止的全部键调核；
/// 这一盘的 Vec 里位置等于表里那一份的 move 进 `records`，别的丢掉；这一盘的 Vec 放掉，再读下一盘。
pub(super) fn scan_journal_merging_device_by_device(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
) -> RecordTable {
    let setup = scan_setup(system_configuration);
    let mut all_keys_so_far: Vec<(u32, u64)> = Vec::new();
    let mut records = BTreeMap::new();
    let mut records_before_this_device = 0usize;
    let mut previous_first_copies: Option<BTreeMap<(u32, u64), usize>> = None;
    for device in reader.device_identities() {
        let mut this_device: Vec<JournalRecord> = Vec::new();
        read_the_records_of_one_device(reader, device, &setup, &mut this_device);
        all_keys_so_far.extend(this_device.iter().map(key_of));
        drop(previous_first_copies.take());
        // 核看到的键：从全局键表的第 0 个起，即到此为止的全部键（登记第九节 M7 把它改成只从这一盘的第一个键起）。
        let first_key_given_to_the_core = 0usize;
        let first_copies =
            first_copy_of_each_record_key(&all_keys_so_far[first_key_given_to_the_core..]);
        for (index, record) in this_device.drain(..).enumerate() {
            let position = records_before_this_device + index - first_key_given_to_the_core;
            if first_copies.get(&(record.instance.0, record.counter)) == Some(&position) {
                records.insert((record.instance, record.counter), record);
            }
        }
        drop(this_device);
        records_before_this_device = all_keys_so_far.len();
        previous_first_copies = Some(first_copies);
    }
    records
}

/// split_pc：与 split 同，只改非首份的去处——不丢，push 进一个留到交回之前的 Vec（每盘的非首份连同点名项堆一直留着）。
pub(super) fn scan_journal_merging_device_by_device_keeping_the_duplicates(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
) -> RecordTable {
    let setup = scan_setup(system_configuration);
    let mut all_keys_so_far: Vec<(u32, u64)> = Vec::new();
    let mut records = BTreeMap::new();
    let mut duplicates_kept_until_return: Vec<JournalRecord> = Vec::new();
    let mut records_before_this_device = 0usize;
    let mut previous_first_copies: Option<BTreeMap<(u32, u64), usize>> = None;
    for device in reader.device_identities() {
        let mut this_device: Vec<JournalRecord> = Vec::new();
        read_the_records_of_one_device(reader, device, &setup, &mut this_device);
        all_keys_so_far.extend(this_device.iter().map(key_of));
        drop(previous_first_copies.take());
        let first_copies = first_copy_of_each_record_key(&all_keys_so_far);
        for (index, record) in this_device.drain(..).enumerate() {
            let position = records_before_this_device + index;
            if first_copies.get(&(record.instance.0, record.counter)) == Some(&position) {
                records.insert((record.instance, record.counter), record);
            } else {
                duplicates_kept_until_return.push(record);
            }
        }
        records_before_this_device = all_keys_so_far.len();
        previous_first_copies = Some(first_copies);
    }
    std::hint::black_box(&duplicates_kept_until_return);
    records
}

/// 一张表的摘要（登记第六节 S2 在生产格上的比法）：按键序把 (实例代号, 计数器, 整条记录 4096 字节编码的 CRC-32C) 折进一个 64 位数。
/// 生产格里不在同一个进程里再跑一遍 `scan_journal`（那会把峰值翻倍），各臂的摘要在出表时与基线臂同格的摘要比；差分集 D 用 `==` 逐键逐值比。
pub(super) fn table_digest(table: &RecordTable) -> String {
    let mut state: u64 = 0xcbf2_9ce4_8422_2325;
    for ((instance, counter), record) in table {
        let encoding_checksum = crc32_castagnoli(&record.to_bytes());
        for word in [
            u64::from(instance.0),
            *counter,
            u64::from(encoding_checksum),
        ] {
            state = (state ^ word)
                .wrapping_mul(0x0000_0100_0000_01b3)
                .rotate_left(31);
        }
    }
    let length = u64::try_from(table.len()).expect("表长装得进 u64");
    format!("{:016x}", state ^ length)
}

/// 两张表逐键逐值相同（`JournalRecord` 派生了 `PartialEq`）。
pub(super) fn tables_are_equal(left: &RecordTable, right: &RecordTable) -> bool {
    left == right
}

#[cfg(test)]
mod tests;
