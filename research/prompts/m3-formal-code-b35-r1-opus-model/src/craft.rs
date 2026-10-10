//! 造坏镜像的几样工具（照 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的重封与沿引用链补校验和的做法，改成按镜像现扫单元表、
//! 按池几何取根槽与 journal 环；那份文件名不带 layer0）。
use singlefs_checker::image::{chosen_system_configurations, root_slot_positions, ImageReader};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::journal::{back_chain_of, record_offset, JournalRecord};
use singlefs_core::recovery::PoolReader;
use singlefs_harness::memory_pool::MemoryPool;

pub const SLOT: u64 = 16384;
pub const DEVICES: [u32; 2] = [0, 1];

pub fn read(pool: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
    PoolReader::read(pool, DeviceIdentity(device), DeviceOffsetInBytes(offset), length).expect("读")
}

pub fn write(pool: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
    pool.devices
        .get_mut(&DeviceIdentity(device))
        .expect("盘")
        .write(DeviceOffsetInBytes(offset), bytes);
}

/// 盘 0 单元区里写过的槽上，头 magic 对、类 1 / 2 / 3 的单元：(起点槽, 整单元字节数)。
pub fn units(pool: &MemoryPool) -> Vec<(u64, usize)> {
    let mut out = Vec::new();
    for slot in pool.candidate_unit_slots(0).unwrap_or_default() {
        let head = read(pool, 0, slot * SLOT, 512);
        if &head[..4] != b"SFSU" {
            continue;
        }
        let length = match head[6] {
            1 | 3 => 32768,
            2 => 16384,
            _ => continue,
        };
        out.push((slot, length));
    }
    out
}

/// 按类重封：先算载荷 CRC，再算头校验和。
pub fn reseal_unit(bytes: &mut [u8]) {
    let (header_end, payload_crc_offset) = match bytes[6] {
        1 => (105usize, 101usize),
        3 => (107, 89),
        _ => (86 + 2 * usize::from(bytes[51]), 76 + 2 * usize::from(bytes[51])),
    };
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    singlefs_core::unit::seal_header_checksum(bytes, header_end);
}

pub fn location_pattern(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
    [device.to_le_bytes().to_vec(), slot.to_le_bytes()[..6].to_vec(), checksum.to_le_bytes().to_vec()].concat()
}

pub fn replace_all(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) -> bool {
    let mut changed = false;
    let mut index = 0;
    while index + needle.len() <= haystack.len() {
        if haystack[index..index + needle.len()] == *needle {
            haystack[index..index + needle.len()].copy_from_slice(replacement);
            changed = true;
            index += needle.len();
        } else {
            index += 1;
        }
    }
    changed
}

/// 池的根槽：(盘, 偏移)。
pub fn root_positions(pool: &MemoryPool) -> Vec<(u32, u64)> {
    let geometry = chosen_system_configurations(pool)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
        .expect("系统配置择得出");
    root_slot_positions(&geometry).into_iter().map(|(_, _, device, offset)| (device, offset)).collect()
}

pub fn reseal_root(bytes: &mut [u8]) {
    let digest = wide_checksum_with_field_zeroed(bytes, 512, 138);
    bytes[138..170].copy_from_slice(&digest);
}

/// 环里从计数器 1 起读得出的记录（每块盘）：(计数器, 偏移, 字节)。到第一条解不出的为止。
pub fn records(pool: &MemoryPool, device: u32, ring_bytes: u64) -> Vec<(u64, u64, Vec<u8>)> {
    let mut out = Vec::new();
    for counter in 1..=(ring_bytes / 4096) {
        let offset = record_offset(counter, ring_bytes).0;
        let bytes = read(pool, device, offset, 4096);
        let fsid = u64::from_le_bytes(bytes[463..471].try_into().expect("8"));
        if JournalRecord::parse(&bytes, fsid).is_none() {
            break;
        }
        out.push((counter, offset, bytes));
    }
    out
}

/// 记录改过字节之后重封：载荷校验和（95 起 4 字节，罩 487 起的点名项）与记录头校验和（罩整条 4096，字段在 46）。
pub fn reseal_record(record: &mut [u8]) {
    let named = usize::try_from(u32::from_le_bytes(record[12..16].try_into().expect("4"))).expect("点名项数");
    let payload = crc32_castagnoli(&record[487..487 + 56 * named]);
    record[95..99].copy_from_slice(&payload.to_le_bytes());
    let digest = wide_checksum_with_field_zeroed(record, 4096, 46);
    record[46..78].copy_from_slice(&digest);
}

/// 每块盘按计数器升序：前一条（同实例、计数器小 1）改过的，这一条的反向链照它重算、重封。`changed` 是这一遍改过字节的 (盘, 计数器)。
pub fn rechain(pool: &mut MemoryPool, ring_bytes: u64, mut changed: std::collections::BTreeSet<(u32, u64)>) {
    for device in DEVICES {
        let list = records(pool, device, ring_bytes);
        for window in 0..list.len() {
            let (counter, offset, _) = list[window];
            if counter == 1 || !changed.contains(&(device, counter - 1)) {
                continue;
            }
            let previous = read(pool, device, record_offset(counter - 1, ring_bytes).0, 4096);
            let bytes = read(pool, device, offset, 4096);
            let fsid = u64::from_le_bytes(bytes[463..471].try_into().expect("8"));
            let previous_fsid = u64::from_le_bytes(previous[463..471].try_into().expect("8"));
            let Some(mut record) = JournalRecord::parse(&bytes, fsid) else { continue };
            let Some(previous_record) = JournalRecord::parse(&previous, previous_fsid) else { continue };
            if previous_record.instance != record.instance {
                continue;
            }
            record.back_chain = back_chain_of(&previous);
            let rewritten = record.to_bytes();
            if rewritten != bytes {
                write(pool, device, offset, &rewritten);
                changed.insert((device, counter));
            }
        }
    }
}

/// 几个单元的整单元校验和变了（(盘, 起点槽, 旧, 新)）：单元、根槽、journal 记录里引用它们的位置条目逐个补，补过的单元再往上补；
/// 记录补完重封、后面的记录反向链重算。
pub fn propagate(pool: &mut MemoryPool, ring_bytes: u64, mut changes: Vec<(u32, u64, u32, u32)>) {
    let unit_list = units(pool);
    let mut records_changed = std::collections::BTreeSet::new();
    while let Some((device, slot, old, new)) = changes.pop() {
        if old == new {
            continue;
        }
        let (needle, replacement) = (location_pattern(device, slot, old), location_pattern(device, slot, new));
        for (container, length) in unit_list.iter().copied() {
            for container_device in DEVICES {
                let mut bytes = read(pool, container_device, container * SLOT, length);
                if !replace_all(&mut bytes, &needle, &replacement) {
                    continue;
                }
                let before = crc32_castagnoli(&read(pool, container_device, container * SLOT, length));
                reseal_unit(&mut bytes);
                write(pool, container_device, container * SLOT, &bytes);
                changes.push((container_device, container, before, crc32_castagnoli(&bytes)));
            }
        }
        for (root_device, offset) in root_positions(pool) {
            let mut bytes = read(pool, root_device, offset, 512);
            if replace_all(&mut bytes, &needle, &replacement) {
                reseal_root(&mut bytes);
                write(pool, root_device, offset, &bytes);
            }
        }
        for record_device in DEVICES {
            for (counter, offset, mut bytes) in records(pool, record_device, ring_bytes) {
                if replace_all(&mut bytes, &needle, &replacement) {
                    reseal_record(&mut bytes);
                    write(pool, record_device, offset, &bytes);
                    records_changed.insert((record_device, counter));
                }
            }
        }
    }
    rechain(pool, ring_bytes, records_changed);
}

/// 新根段在记录里的偏移与它七段各对应根记录的哪一段（D23 已定项 15 的对照，`crates/singlefs-checker/src/walk.rs` 的
/// `NEW_ROOT_SEGMENT_FIELDS_COMPARED_WITH_THE_ROOT` 同一张）。
pub const SEGMENT_OFFSET_IN_RECORD: usize = 99;
pub const SEGMENT_FROM_ROOT: [(std::ops::Range<usize>, std::ops::Range<usize>); 7] = [
    (0..86, 36..122),
    (86..172, 256..342),
    (172..180, 122..130),
    (180..188, 130..138),
    (188..274, 170..256),
    (274..360, 342..428),
    (360..364, 20..24),
];

/// 改一条根（按 txg 认、两盘的根槽里找）：改字节、重封，再把它照抄进同 (实例, txg) 的每条记录的新根段（I-8.10 照旧成立）、重链。
pub fn rewrite_root(pool: &mut MemoryPool, ring_bytes: u64, txg: u64, change: impl Fn(&mut [u8])) -> usize {
    let mut rewritten = 0;
    let mut changed = std::collections::BTreeSet::new();
    for (device, offset) in root_positions(pool) {
        let mut bytes = read(pool, device, offset, 512);
        if &bytes[..4] == [0u8; 4] || u64::from_le_bytes(bytes[28..36].try_into().expect("8")) != txg {
            continue;
        }
        change(&mut bytes);
        reseal_root(&mut bytes);
        write(pool, device, offset, &bytes);
        rewritten += 1;
        let instance = &bytes[24..28];
        for record_device in DEVICES {
            for (counter, record_offset_bytes, mut record) in records(pool, record_device, ring_bytes) {
                if &record[16..20] != instance || u64::from_le_bytes(record[26..34].try_into().expect("8")) != txg {
                    continue;
                }
                for (segment, root) in SEGMENT_FROM_ROOT.iter().cloned() {
                    record[SEGMENT_OFFSET_IN_RECORD + segment.start..SEGMENT_OFFSET_IN_RECORD + segment.end]
                        .copy_from_slice(&bytes[root]);
                }
                reseal_record(&mut record);
                write(pool, record_device, record_offset_bytes, &record);
                changed.insert((record_device, counter));
            }
        }
    }
    rechain(pool, ring_bytes, changed);
    rewritten
}
