//! 原型（m2-witness-r1 云端攻方腿，只在副本上）：C332 的表形态见证。
//!
//! 一条见证 = (新实例 N, 回退目标实例 r, 回退目标 txg T)，16 字节。它说：按 (实例, txg) 字典序落在
//! (r, T) 与 (N, 0) 之间（两端都不含）的根与记录，都在被这次回退抛弃的时间线上。
//! 三个放处由环境变量 `SINGLEFS_WITNESS_PLACEMENT` 选（none / a / b / c），写序由
//! `SINGLEFS_WITNESS_ORDER` 选（pre / post / late）。一个进程只跑一臂（OnceLock）。
use std::collections::BTreeSet;
use std::sync::OnceLock;

use crate::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use crate::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use crate::recovery::PoolReader;
use crate::system_configuration::SystemConfiguration;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WitnessPlacement {
    None,
    /// (a) 系统配置槽里 481 + 8（C331 乙）之后，被整槽校验和罩着，越过 512。
    SystemConfigurationSlot,
    /// (b) 独立自证单元：每盘两槽轮换，自带 fsid、盘号、世代号、CRC。
    IndependentUnit,
    /// (c) 系统配置槽第 2 个 512 扇区里一片自证片；系统配置自己的整槽校验和把这一扇区按 0 算。
    SectorPiece,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WitnessOrder {
    /// 取号之后、第一个新根之前（(a)(c) 就是取号那次系统配置写本身）。
    BeforeFirstRoot,
    /// 写行那次发布的根 FUA 之后（(a)(c) 是那次发布末尾的系统配置轮换；(b) 是那次发布之后另写）。
    AfterFirstRoot,
    /// 暖机全部做完之后另写一次。
    AfterWarmUp,
}

pub fn placement() -> WitnessPlacement {
    static CELL: OnceLock<WitnessPlacement> = OnceLock::new();
    *CELL.get_or_init(|| match std::env::var("SINGLEFS_WITNESS_PLACEMENT").as_deref() {
        Ok("a") => WitnessPlacement::SystemConfigurationSlot,
        Ok("b") => WitnessPlacement::IndependentUnit,
        Ok("c") => WitnessPlacement::SectorPiece,
        Ok("none") | Err(_) => WitnessPlacement::None,
        Ok(other) => panic!("SINGLEFS_WITNESS_PLACEMENT={other} 不认识"),
    })
}

pub fn order() -> WitnessOrder {
    static CELL: OnceLock<WitnessOrder> = OnceLock::new();
    *CELL.get_or_init(|| match std::env::var("SINGLEFS_WITNESS_ORDER").as_deref() {
        Ok("pre") | Err(_) => WitnessOrder::BeforeFirstRoot,
        Ok("post") => WitnessOrder::AfterFirstRoot,
        Ok("late") => WitnessOrder::AfterWarmUp,
        Ok(other) => panic!("SINGLEFS_WITNESS_ORDER={other} 不认识"),
    })
}

/// 读见证时要不要核 fsid（攻「换盘重格式化之后旧片还在」那一格用；缺省核）。
pub fn checks_filesystem_identifier() -> bool {
    static CELL: OnceLock<bool> = OnceLock::new();
    *CELL.get_or_init(|| std::env::var("SINGLEFS_WITNESS_SKIP_FSID").is_err())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WitnessEntry {
    pub new_instance: u32,
    pub target_instance: u32,
    pub target_txg: u64,
}

pub const WITNESS_CAPACITY: usize = 4;
pub const WITNESS_ENTRY_BYTES: usize = 16;
pub const WITNESS_TABLE_BYTES: usize = 1 + WITNESS_ENTRY_BYTES * WITNESS_CAPACITY;
/// (a)：系统配置 481 字节 + C331 乙 8 字节之后。
pub const SYSTEM_CONFIGURATION_WITNESS_OFFSET: usize = 481 + 8;
/// (c)：系统配置槽第 2 个扇区。
pub const SECTOR_PIECE_OFFSET_IN_SLOT: usize = 512;
pub const SECTOR_PIECE_BYTES: usize = 512;
/// (b)：独立单元两槽的盘上偏移（系统配置两槽之后、根环 1 MiB 之前那段空着的地方）。
pub const INDEPENDENT_UNIT_OFFSETS: [u64; 2] = [512 * 1024, 512 * 1024 + 4096];
const PIECE_MAGIC_UNIT: [u8; 4] = *b"SFWU";
const PIECE_MAGIC_SECTOR: [u8; 4] = *b"SFWP";
/// 片头：magic 4 + fsid 16 + 盘号 4 + 世代号 8，之后是表，之后 CRC 4。
const PIECE_HEADER_BYTES: usize = 4 + 16 + 4 + 8;
pub const PIECE_CONTENT_BYTES: usize = PIECE_HEADER_BYTES + WITNESS_TABLE_BYTES + 4;

pub fn encode_table(entries: &BTreeSet<WitnessEntry>) -> ([u8; WITNESS_TABLE_BYTES], usize) {
    let mut out = [0u8; WITNESS_TABLE_BYTES];
    // 满了丢最老的（new_instance 最小的），并报丢了几条。
    let kept: Vec<&WitnessEntry> = entries.iter().rev().take(WITNESS_CAPACITY).collect();
    let dropped = entries.len().saturating_sub(WITNESS_CAPACITY);
    out[0] = u8::try_from(kept.len()).expect("≤4");
    for (index, entry) in kept.iter().rev().enumerate() {
        let base = 1 + index * WITNESS_ENTRY_BYTES;
        out[base..base + 4].copy_from_slice(&entry.new_instance.to_le_bytes());
        out[base + 4..base + 8].copy_from_slice(&entry.target_instance.to_le_bytes());
        out[base + 8..base + 16].copy_from_slice(&entry.target_txg.to_le_bytes());
    }
    (out, dropped)
}

pub fn decode_table(bytes: &[u8]) -> Vec<WitnessEntry> {
    let count = usize::from(bytes[0]).min(WITNESS_CAPACITY);
    (0..count)
        .map(|index| {
            let base = 1 + index * WITNESS_ENTRY_BYTES;
            WitnessEntry {
                new_instance: u32::from_le_bytes(bytes[base..base + 4].try_into().expect("4")),
                target_instance: u32::from_le_bytes(bytes[base + 4..base + 8].try_into().expect("4")),
                target_txg: u64::from_le_bytes(bytes[base + 8..base + 16].try_into().expect("8")),
            }
        })
        .collect()
}

/// (实例, txg) 落在某一条见证抛弃的区间里。
pub fn abandons(entries: &BTreeSet<WitnessEntry>, instance: InstanceGeneration, txg: CheckpointTxg) -> bool {
    entries.iter().any(|entry| {
        (entry.target_instance, entry.target_txg) < (instance.0, txg.0)
            && instance.0 < entry.new_instance
    })
}

pub fn build_piece(
    magic: [u8; 4],
    filesystem_identifier: &[u8; 16],
    device: DeviceIdentity,
    generation: u64,
    entries: &BTreeSet<WitnessEntry>,
    piece_bytes: usize,
) -> Vec<u8> {
    let mut out = vec![0u8; piece_bytes];
    out[0..4].copy_from_slice(&magic);
    out[4..20].copy_from_slice(filesystem_identifier);
    out[20..24].copy_from_slice(&device.0.to_le_bytes());
    out[24..32].copy_from_slice(&generation.to_le_bytes());
    let (table, _) = encode_table(entries);
    out[PIECE_HEADER_BYTES..PIECE_HEADER_BYTES + WITNESS_TABLE_BYTES].copy_from_slice(&table);
    let crc_at = PIECE_HEADER_BYTES + WITNESS_TABLE_BYTES;
    let crc = crc32_castagnoli(&out[..crc_at]);
    out[crc_at..crc_at + 4].copy_from_slice(&crc.to_le_bytes());
    out
}

/// 片自证过 ⇒ (世代号, 表)。
pub fn parse_piece(
    bytes: &[u8],
    magic: [u8; 4],
    filesystem_identifier: &[u8; 16],
) -> Option<(u64, Vec<WitnessEntry>)> {
    if bytes.len() < PIECE_CONTENT_BYTES || bytes[0..4] != magic {
        return None;
    }
    let crc_at = PIECE_HEADER_BYTES + WITNESS_TABLE_BYTES;
    let crc = u32::from_le_bytes(bytes[crc_at..crc_at + 4].try_into().expect("4"));
    if crc32_castagnoli(&bytes[..crc_at]) != crc {
        return None;
    }
    if checks_filesystem_identifier() && bytes[4..20] != filesystem_identifier[..] {
        return None;
    }
    let generation = u64::from_le_bytes(bytes[24..32].try_into().expect("8"));
    Some((generation, decode_table(&bytes[PIECE_HEADER_BYTES..PIECE_HEADER_BYTES + WITNESS_TABLE_BYTES])))
}

/// (c) 用：系统配置整槽校验和把第 2 扇区按 0 算。
pub fn system_configuration_checksum_view(bytes: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    if placement() == WitnessPlacement::SectorPiece && bytes.len() >= SECTOR_PIECE_OFFSET_IN_SLOT + SECTOR_PIECE_BYTES {
        let mut owned = bytes.to_vec();
        owned[SECTOR_PIECE_OFFSET_IN_SLOT..SECTOR_PIECE_OFFSET_IN_SLOT + SECTOR_PIECE_BYTES].fill(0);
        std::borrow::Cow::Owned(owned)
    } else {
        std::borrow::Cow::Borrowed(bytes)
    }
}

/// 一个见证副本在盘上的位置（故障注入按它点名）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WitnessCopyLocation {
    pub device: DeviceIdentity,
    pub offset: u64,
    pub length: u64,
}

pub fn witness_copy_locations(devices: &[DeviceIdentity], spacing: u64) -> Vec<WitnessCopyLocation> {
    let mut out = Vec::new();
    for &device in devices {
        match placement() {
            WitnessPlacement::None => {}
            WitnessPlacement::SystemConfigurationSlot => {
                for slot_offset in [0, spacing] {
                    out.push(WitnessCopyLocation { device, offset: slot_offset, length: SYSTEM_CONFIGURATION_SLOT_BYTES });
                }
            }
            WitnessPlacement::SectorPiece => {
                for slot_offset in [0, spacing] {
                    out.push(WitnessCopyLocation {
                        device,
                        offset: slot_offset + SECTOR_PIECE_OFFSET_IN_SLOT as u64,
                        length: SECTOR_PIECE_BYTES as u64,
                    });
                }
            }
            WitnessPlacement::IndependentUnit => {
                for unit_offset in INDEPENDENT_UNIT_OFFSETS {
                    out.push(WitnessCopyLocation { device, offset: unit_offset, length: 512 });
                }
            }
        }
    }
    out
}

/// 盘上全部自证过的见证副本的并集。
pub fn read_witness<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    filesystem_identifier: &[u8; 16],
    spacing: u64,
    physical_block_size: u64,
) -> BTreeSet<WitnessEntry> {
    let mut entries = BTreeSet::new();
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in reader.device_identities() {
        match placement() {
            WitnessPlacement::None => {}
            WitnessPlacement::SystemConfigurationSlot => {
                for slot_offset in [0, spacing] {
                    let Some(bytes) = reader.read(device, DeviceOffsetInBytes(slot_offset), slot_bytes) else { continue };
                    let Ok(sc) = SystemConfiguration::parse_slot(&bytes) else { continue };
                    if checks_filesystem_identifier() && sc.immutable.filesystem_identifier != *filesystem_identifier {
                        continue;
                    }
                    entries.extend(decode_table(&bytes[SYSTEM_CONFIGURATION_WITNESS_OFFSET..SYSTEM_CONFIGURATION_WITNESS_OFFSET + WITNESS_TABLE_BYTES]));
                }
            }
            WitnessPlacement::SectorPiece => {
                for slot_offset in [0, spacing] {
                    let offset = slot_offset + SECTOR_PIECE_OFFSET_IN_SLOT as u64;
                    let Some(bytes) = reader.read(device, DeviceOffsetInBytes(offset), SECTOR_PIECE_BYTES) else { continue };
                    if let Some((_, table)) = parse_piece(&bytes, PIECE_MAGIC_SECTOR, filesystem_identifier) {
                        entries.extend(table);
                    }
                }
            }
            WitnessPlacement::IndependentUnit => {
                let unit_bytes = usize::try_from(physical_block_size).expect("pbs");
                for unit_offset in INDEPENDENT_UNIT_OFFSETS {
                    let Some(bytes) = reader.read(device, DeviceOffsetInBytes(unit_offset), unit_bytes) else { continue };
                    if let Some((_, table)) = parse_piece(&bytes, PIECE_MAGIC_UNIT, filesystem_identifier) {
                        entries.extend(table);
                    }
                }
            }
        }
    }
    entries
}

/// 从选中的系统配置取几何读见证（择根与重放用）。
pub fn read_witness_with<Reader: PoolReader + ?Sized>(reader: &Reader, system_configuration: &SystemConfiguration) -> BTreeSet<WitnessEntry> {
    if placement() == WitnessPlacement::None {
        return BTreeSet::new();
    }
    read_witness(
        reader,
        &system_configuration.immutable.filesystem_identifier,
        u64::from(system_configuration.immutable.sizes.fixed_structure_slot_spacing),
        u64::from(system_configuration.immutable.sizes.physical_block_size),
    )
}

/// (a)：把表嵌进系统配置槽、重算整槽校验和。
pub fn embed_into_system_configuration_slot(slot: &mut [u8], entries: &BTreeSet<WitnessEntry>) {
    let (table, _) = encode_table(entries);
    slot[SYSTEM_CONFIGURATION_WITNESS_OFFSET..SYSTEM_CONFIGURATION_WITNESS_OFFSET + WITNESS_TABLE_BYTES].copy_from_slice(&table);
    let checksum_offset = singlefs_format_checksum_offset();
    let digest = wide_checksum_with_field_zeroed(slot, slot.len(), checksum_offset);
    slot[checksum_offset..checksum_offset + 32].copy_from_slice(&digest);
}

fn singlefs_format_checksum_offset() -> usize {
    crate::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET
}

/// (c)：系统配置槽第 2 扇区写一片自证片（整槽校验和已经按这一扇区为 0 算过）。
pub fn place_sector_piece(slot: &mut [u8], filesystem_identifier: &[u8; 16], device: DeviceIdentity, generation: u64, entries: &BTreeSet<WitnessEntry>) {
    // 表空时不写片（第 2 扇区保持 0）：没有回退过的池与今天逐字节相同。
    if entries.is_empty() {
        return;
    }
    let piece = build_piece(PIECE_MAGIC_SECTOR, filesystem_identifier, device, generation, entries, SECTOR_PIECE_BYTES);
    slot[SECTOR_PIECE_OFFSET_IN_SLOT..SECTOR_PIECE_OFFSET_IN_SLOT + SECTOR_PIECE_BYTES].copy_from_slice(&piece);
}

/// (b)：独立单元一片（物理块宽）。
pub fn build_independent_unit(filesystem_identifier: &[u8; 16], device: DeviceIdentity, generation: u64, entries: &BTreeSet<WitnessEntry>, physical_block_size: usize) -> Vec<u8> {
    build_piece(PIECE_MAGIC_UNIT, filesystem_identifier, device, generation, entries, physical_block_size)
}

/// (b)：这块盘两槽里自证过的最大世代号。
pub fn highest_independent_unit_generation<Reader: PoolReader + ?Sized>(reader: &Reader, device: DeviceIdentity, filesystem_identifier: &[u8; 16], physical_block_size: usize) -> u64 {
    INDEPENDENT_UNIT_OFFSETS
        .iter()
        .filter_map(|offset| reader.read(device, DeviceOffsetInBytes(*offset), physical_block_size))
        .filter_map(|bytes| parse_piece(&bytes, PIECE_MAGIC_UNIT, filesystem_identifier).map(|(generation, _)| generation))
        .max()
        .unwrap_or(0)
}
