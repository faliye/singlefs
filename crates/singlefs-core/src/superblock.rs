//! 超级块（D22（单元原子性怎么合成） 已定项 9 / 已定项 15 / 已定项 16）：481 字节住 4096 字节的槽，每盘两槽轮换，
//! 整槽校验和罩 4096 含补齐、自身按 0 参与。字段顺序照 `layout/01-first-txn.md` 一那一节的字段表。

use singlefs_format::{
    journal_in_flight_record_limit, DATA_UNIT_BYTES, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
    JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, JOURNAL_SAFETY_FACTOR, LOC_ENTRY, NODE_BYTES,
    ROOT_RING_BASE_SLOT, ROOT_RING_CHUNK_BYTES, ROOT_RING_PRIME_STEP, ROOT_RING_REGIONS,
    ROOT_RING_SLOTS_PER_REGION, SLOT_BYTES, SUPERBLOCK_BYTES, SUPERBLOCK_SLOT_BYTES,
    UNIT_AREA_START_SLOT, WIDE_CHECKSUM_BYTES,
};

use crate::address::{DeviceIdentity, InstanceGeneration};
use crate::bytes::{ByteReader, ByteWriter};
use crate::checksum::{wide_checksum_field_holds, wide_checksum_with_field_zeroed};

pub const SUPERBLOCK_MAGIC: [u8; 4] = *b"SFSB";
pub const FORMAT_VERSION: u16 = 1;
/// incompat 位 0 = 第一条纯 SSD 布局线（D15（格式冻结政策） 已定项 4）；位 n 住第 n div 8 个字节的第 n mod 8 低位。
pub const INCOMPAT_FIRST_SSD_LINE_BIT: u8 = 0x01;
pub const SUPPORTED_INCOMPAT_BITS: u8 = INCOMPAT_FIRST_SSD_LINE_BIT;
const FEATURE_BITS_OFFSET: usize = 4 + 2;
const FEATURE_BITMAP_BYTES: usize = 32;
/// 写入者身份：实现标识 16 字节 ASCII 零补齐 + 版本 4（D17（实现分层与第三方管道） 债 3、I-1.5）。
pub const WRITER_IDENTITY_NAME: &[u8] = b"singlefs-rs";
pub const WRITER_IDENTITY_VERSION: u32 = 1;
/// 校验和算法标识登记表：0 无效、1 CRC-32C。
pub const CHECKSUM_ALGORITHM_CRC32_CASTAGNOLI: u8 = 1;
/// 自举头：magic 4 + 格式版本 2 + feature bits 96 + fsid 16 + 写入者身份 20 + 校验和算法标识 1 + 本盘设备号 4 + 设备数 4 + 槽世代号 8。
pub const SUPERBLOCK_CHECKSUM_OFFSET: usize = 4 + 2 + 96 + 16 + 20 + 1 + 4 + 4 + 8;
const FSID_OFFSET: usize = 4 + 2 + 96;
const REGION_DEVICES_OFFSET: u64 = 379;
const TAIL_OFFSET: u64 = 469;
/// D2（RAID 条带策略） 已定项 18：第一版 w_max 与 g 都写 4。
const MAXIMUM_STRIPE_WIDTH: u8 = 4;
const GROUP_SIZE: u8 = 4;
/// D16（发布语义） 已定项 5 的两个可调值。
const TIME_THRESHOLD_SECONDS: u32 = 5;
const DIRTY_THRESHOLD_BYTES: u64 = 2 << 30;

/// mkfs 时定下、之后只读的几何参数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormatTimeGeometry {
    /// mkfs 时探测到的 physical_block_size（2026-09-14 用户定案的字段，给挂载与 checker 比对用）。
    pub physical_block_size: u32,
    /// mkfs 时探测的 io_min。
    pub minimum_input_output_bytes: u32,
    /// 固定结构槽距 = 4096 向上取整到 io_min 的整数倍（D2（RAID 条带策略） 已定项 19）。
    pub fixed_structure_slot_spacing: u32,
    /// journal 环长（mkfs 参数，默认 768 MiB，环 ≤ 容量 / 4，D23（journal 的角色与格式） 已定项 19）。
    pub journal_ring_bytes: u64,
}

impl FormatTimeGeometry {
    /// 槽距 = 4096 向上取整到 io_min 的整数倍。
    #[must_use]
    pub fn slot_spacing_for(minimum_input_output_bytes: u32) -> u32 {
        let floor = u32::try_from(FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES).expect("4096");
        if minimum_input_output_bytes == 0 {
            return floor;
        }
        floor.div_ceil(minimum_input_output_bytes) * minimum_input_output_bytes
    }
}

/// 超级块里每次写都会变的那几个字段；几何与常量段在 [`Superblock::to_slot`] 里照字段表写死。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Superblock {
    pub filesystem_identifier: [u8; 16],
    pub this_device: DeviceIdentity,
    pub device_count: u32,
    pub slot_generation: u64,
    pub region_devices: [DeviceIdentity; 3],
    pub geometry: FormatTimeGeometry,
    pub journal_tail: u64,
    pub journal_instance: InstanceGeneration,
}

fn slot_bytes() -> usize {
    usize::try_from(SUPERBLOCK_SLOT_BYTES).expect("4096")
}

impl Superblock {
    #[must_use]
    pub fn to_slot(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(slot_bytes());
        writer.put(&SUPERBLOCK_MAGIC);
        writer.put_u16(FORMAT_VERSION);
        writer.put_u8(INCOMPAT_FIRST_SSD_LINE_BIT);
        writer.skip(95);
        writer.assert_position(
            u64::try_from(FSID_OFFSET).expect("fsid 偏移"),
            "超级块 fsid",
        );
        writer.put(&self.filesystem_identifier);
        let mut writer_identity = [0u8; 16];
        writer_identity[..WRITER_IDENTITY_NAME.len()].copy_from_slice(WRITER_IDENTITY_NAME);
        writer.put(&writer_identity);
        writer.put_u32(WRITER_IDENTITY_VERSION);
        writer.put_u8(CHECKSUM_ALGORITHM_CRC32_CASTAGNOLI);
        writer.put_u32(self.this_device.0);
        writer.put_u32(self.device_count);
        writer.put_u64(self.slot_generation);
        writer.assert_position(
            u64::try_from(SUPERBLOCK_CHECKSUM_OFFSET).expect("校验和偏移"),
            "超级块整槽校验和",
        );
        writer.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
        writer.skip(16 + 12 + 4); // 超级块 MAC、nonce 水位、KDF 标识：第一版全 0
        writer.put_u8(0); // 加密类型：关
        writer.put_u8(16); // MAC 长度声明
        writer.skip(80); // 主密钥槽：内联进槽，加密关时全 0
        writer.put_u32(u32::try_from(NODE_BYTES).expect("节点大小"));
        writer.put_u32(u32::try_from(DATA_UNIT_BYTES).expect("单元大小"));
        writer.put_u32(u32::try_from(SLOT_BYTES).expect("落点粒度"));
        writer.put_u32(u32::try_from(LOC_ENTRY).expect("位置条目宽度"));
        writer.put_u32(self.geometry.physical_block_size);
        writer.put_u32(0); // 扩展点声明值 N：第一版 0（D21（权威态与派生态的分界） 已定项 8）
        writer.put_u64(JOURNAL_RING_START_SLOT);
        writer.put_u64(self.geometry.journal_ring_bytes);
        writer.put_u32(u32::try_from(JOURNAL_RECORD_BYTES).expect("记录尺寸"));
        let in_flight_limit = journal_in_flight_record_limit(self.geometry.journal_ring_bytes);
        writer.put_u32(u32::try_from(in_flight_limit).expect("在飞上限 4 字节"));
        writer.put_u64(in_flight_limit * JOURNAL_RECORD_BYTES);
        writer.put_u32(u32::try_from(JOURNAL_SAFETY_FACTOR).expect("安全系数"));
        writer.put_u8(u8::try_from(ROOT_RING_REGIONS).expect("R"));
        writer.put_u8(u8::try_from(ROOT_RING_SLOTS_PER_REGION).expect("S"));
        writer.put_u32(u32::try_from(ROOT_RING_PRIME_STEP).expect("P"));
        writer.put_u32(u32::try_from(ROOT_RING_CHUNK_BYTES).expect("chunk"));
        writer.put_u64(ROOT_RING_BASE_SLOT);
        writer.assert_position(REGION_DEVICES_OFFSET, "根环逐区域设备身份");
        for region_device in &self.region_devices {
            writer.put_u32(region_device.0);
        }
        writer.put_u8(MAXIMUM_STRIPE_WIDTH);
        writer.put_u8(GROUP_SIZE);
        writer.skip(24); // 映射来源：第一版全 0
        writer.put_u64(UNIT_AREA_START_SLOT);
        writer.put_u32(self.geometry.minimum_input_output_bytes);
        writer.put_u32(self.geometry.fixed_structure_slot_spacing);
        writer.put_u32(TIME_THRESHOLD_SECONDS);
        writer.put_u64(DIRTY_THRESHOLD_BYTES);
        writer.skip(24); // 整理三条水位：第一版恒 0 = 内置默认
        writer.assert_position(TAIL_OFFSET, "journal tail");
        writer.put_u64(self.journal_tail);
        writer.put_u32(self.journal_instance.0);
        writer.assert_position(SUPERBLOCK_BYTES, "超级块");
        let mut bytes = writer.into_bytes();
        let digest =
            wide_checksum_with_field_zeroed(&bytes, slot_bytes(), SUPERBLOCK_CHECKSUM_OFFSET);
        bytes[SUPERBLOCK_CHECKSUM_OFFSET..SUPERBLOCK_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        bytes
    }

    /// 读者：magic、整槽校验和、incompat 位三关都过才解；任一关不过返回 None（这一槽不可择）。
    #[must_use]
    pub fn parse_slot(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < slot_bytes() || bytes[..4] != SUPERBLOCK_MAGIC {
            return None;
        }
        if !wide_checksum_field_holds(bytes, slot_bytes(), SUPERBLOCK_CHECKSUM_OFFSET) {
            return None;
        }
        if !incompat_bits_are_mountable(bytes) {
            return None;
        }
        let mut reader = ByteReader::at(bytes, FSID_OFFSET);
        let filesystem_identifier: [u8; 16] = reader.take(16).try_into().expect("切了 16 字节");
        reader.skip(16 + 4 + 1);
        let this_device = DeviceIdentity(reader.get_u32());
        let device_count = reader.get_u32();
        let slot_generation = reader.get_u64();
        reader.skip(32 + 16 + 12 + 4 + 1 + 1 + 80 + 4 + 4 + 4 + 4);
        let physical_block_size = reader.get_u32();
        reader.skip(4 + 8);
        let journal_ring_bytes = reader.get_u64();
        let mut region_reader =
            ByteReader::at(bytes, usize::try_from(REGION_DEVICES_OFFSET).expect("379"));
        let region_devices = [
            DeviceIdentity(region_reader.get_u32()),
            DeviceIdentity(region_reader.get_u32()),
            DeviceIdentity(region_reader.get_u32()),
        ];
        region_reader.skip(1 + 1 + 24 + 8);
        let minimum_input_output_bytes = region_reader.get_u32();
        let fixed_structure_slot_spacing = region_reader.get_u32();
        let mut tail_reader = ByteReader::at(bytes, usize::try_from(TAIL_OFFSET).expect("469"));
        let journal_tail = tail_reader.get_u64();
        let journal_instance = InstanceGeneration(tail_reader.get_u32());
        Some(Self {
            filesystem_identifier,
            this_device,
            device_count,
            slot_generation,
            region_devices,
            geometry: FormatTimeGeometry {
                physical_block_size,
                minimum_input_output_bytes,
                fixed_structure_slot_spacing,
                journal_ring_bytes,
            },
            journal_tail,
            journal_instance,
        })
    }
}

/// incompat 位图里不认识的位 ⇒ 挂不上（`.claude/rules/fs-design.md`）；且必须带布局身份位。
#[must_use]
pub fn incompat_bits_are_mountable(slot: &[u8]) -> bool {
    let incompat = &slot[FEATURE_BITS_OFFSET..FEATURE_BITS_OFFSET + FEATURE_BITMAP_BYTES];
    let unknown_in_first_byte = incompat[0] & !SUPPORTED_INCOMPAT_BITS;
    let unknown_in_rest = incompat[1..].iter().any(|byte| *byte != 0);
    let has_layout_identity = incompat[0] & INCOMPAT_FIRST_SSD_LINE_BIT != 0;
    unknown_in_first_byte == 0 && !unknown_in_rest && has_layout_identity
}

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;

    fn sample() -> Superblock {
        Superblock {
            filesystem_identifier: [3u8; 16],
            this_device: DeviceIdentity(1),
            device_count: 2,
            slot_generation: 1,
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            geometry: FormatTimeGeometry {
                physical_block_size: 512,
                minimum_input_output_bytes: 512,
                fixed_structure_slot_spacing: 4096,
                journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            },
            journal_tail: 0,
            journal_instance: InstanceGeneration(0),
        }
    }

    #[test]
    fn superblock_is_481_bytes_in_a_4096_slot_and_round_trips() {
        let slot = sample().to_slot();
        assert_eq!(slot.len(), 4096);
        assert!(
            slot[481..].iter().all(|byte| *byte == 0),
            "481 之后全是补齐 0"
        );
        assert_eq!(slot[6], 0x01, "incompat 位 0");
        assert_eq!(
            &slot[FSID_OFFSET + 16..FSID_OFFSET + 16 + 11],
            b"singlefs-rs"
        );
        assert_eq!(Superblock::parse_slot(&slot), Some(sample()));
        assert!(wide_checksum_field_holds(
            &slot,
            4096,
            SUPERBLOCK_CHECKSUM_OFFSET
        ));
    }

    #[test]
    fn corrupted_slot_and_unknown_incompat_bit_are_both_unmountable() {
        let mut slot = sample().to_slot();
        slot[4000] ^= 1;
        assert_eq!(
            Superblock::parse_slot(&slot),
            None,
            "补齐区改一字节也失配：整槽校验和罩 4096"
        );
        let mut unknown = sample().to_slot();
        unknown[6] |= 0x02;
        assert!(!incompat_bits_are_mountable(&unknown));
    }

    #[test]
    fn slot_spacing_rounds_4096_up_to_a_multiple_of_minimum_input_output_size() {
        assert_eq!(FormatTimeGeometry::slot_spacing_for(512), 4096);
        assert_eq!(FormatTimeGeometry::slot_spacing_for(4096), 4096);
        assert_eq!(
            FormatTimeGeometry::slot_spacing_for(3072),
            6144,
            "io_min = 3072 时槽距 6144，两槽不共享映射单元"
        );
        assert_eq!(FormatTimeGeometry::slot_spacing_for(65536), 65536);
    }
}
