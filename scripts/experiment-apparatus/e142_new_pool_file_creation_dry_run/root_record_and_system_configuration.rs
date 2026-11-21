//! 根记录、系统配置的字段与偏移与各种读写、可挂载的不兼容位、环的区域与槽偏移、具名单元。

use e7_index_bench::{u32_from_u64, u8_from_u64, usize_from_u64, usize_from_u64_in_const};
use super::*;

// ───────────────────────── 根记录、系统配置、journal 记录 ─────────────────────────

/// D22（单元原子性怎么合成） 已定项 7 的字段表，457 字节，字段序照那张表（gap G4：字节表七的行序与它不同，两处都没写偏移）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct RootRecord {
    pub(super) fsid: [u8; 16],
    /// D22（单元原子性怎么合成） 已定项 7 flags 位 0：正常卸载那一串写的根置 1，
    /// 其余根写 0；其余位第一版恒 0、非 0 拒收（已定项 17）。这条流里没有正常卸载，恒为 `false`。
    pub(super) unmount_marker: bool,
    pub(super) instance: InstanceGeneration,
    pub(super) checkpoint_txg: CheckpointTxg,
    pub(super) tree_table: NodePointer,
    pub(super) tree_identifier_watermark: u64,
    pub(super) rollback_floor: CheckpointTxg,
    pub(super) instance_table: NodePointer,
    /// D19（块指针的结构与宽度预算） 已定项 11：中央映射树的根住根记录。
    pub(super) mapping_root: NodePointer,
    /// C512（树表 0 条的一版上被换下的实例表记在哪没有条款）：只有「树表 0 条、写过行」
    /// 的一版指着写行那次发布写的那一片分配记录节点；这个装置从不建那一格（E142（新池新建文件的干跑） 只有
    /// mkfs、暖机与带文件的一版，milestone 二「第三次可写挂载」不在射程内），恒为 `NodePointer::empty_root()`。
    pub(super) allocation_record_tree_root: NodePointer,
}

pub(super) const ROOT_CHECKSUM_OFFSET: usize = 4 + 16 + 4 + 4 + 8 + usize_from_u64_in_const(NODE_POINTER_BYTES) + 8 + 8;

impl RootRecord {
    /// 写成一个判定宽度的槽：记录 457 字节，其余补 0；
    /// 自证校验和覆盖**整个 512 槽含补齐**、自身按 0 参与（D18（块里携带什么信息） 已定项 17）。
    pub(super) fn to_slot(self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize_from_u64(PHYSICAL_BLOCK_BYTES));
        writer.put(&ROOT_MAGIC);
        writer.put(&self.fsid);
        writer.put_u32(if self.unmount_marker { 0x0000_0001 } else { 0 });
        writer.put_u32(self.instance.0);
        writer.put_u64(self.checkpoint_txg.0);
        self.tree_table.write_to(&mut writer);
        writer.put_u64(self.tree_identifier_watermark);
        writer.put_u64(self.rollback_floor.0);
        writer.assert_position(ROOT_CHECKSUM_OFFSET as u64, "根记录自证校验和");
        writer.skip(usize_from_u64(WIDE_CHECKSUM_BYTES));
        self.instance_table.write_to(&mut writer);
        self.mapping_root.write_to(&mut writer);
        // C512（树表 0 条的一版上被换下的实例表记在哪没有条款）：分配记录树根指针紧跟映射根指针之后。
        writer.assert_position(ROOT_CHECKSUM_OFFSET as u64 + WIDE_CHECKSUM_BYTES + 2 * NODE_POINTER_BYTES, "根记录：分配记录树根指针起点");
        self.allocation_record_tree_root.write_to(&mut writer);
        // D22 已定项 7 的表尾三段：算法类型 1 + nonce 12 + MAC 16，第一版全 0 留位。
        writer.put_u8(0);
        writer.skip(12 + 16);
        writer.assert_position(ROOT_RECORD_BYTES, "根记录");
        let mut bytes = writer.bytes;
        let digest = wide_checksum_with_field_zeroed(&bytes, usize_from_u64(PHYSICAL_BLOCK_BYTES), ROOT_CHECKSUM_OFFSET);
        bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        bytes
    }

    pub(super) fn parse_slot(bytes: &[u8], expected_fsid: &[u8; 16]) -> Option<Self> {
        if bytes[..4] != ROOT_MAGIC {
            return None;
        }
        if wide_checksum_with_field_zeroed(bytes, usize_from_u64(PHYSICAL_BLOCK_BYTES), ROOT_CHECKSUM_OFFSET) != bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32] {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 4);
        let fsid: [u8; 16] = reader.take(16).try_into().expect("切了 16 字节");
        if &fsid != expected_fsid {
            return None;
        }
        let flags = reader.get_u32();
        if flags & !0x0000_0001 != 0 {
            // D22（单元原子性怎么合成） 已定项 17：flags 除位 0（卸载记号）外恒 0、非 0 拒收。
            return None;
        }
        let unmount_marker = flags & 0x0000_0001 != 0;
        let instance = InstanceGeneration(reader.get_u32());
        let checkpoint_txg = CheckpointTxg(reader.get_u64());
        let tree_table = NodePointer::read_from(&mut reader);
        let tree_identifier_watermark = reader.get_u64();
        let rollback_floor = CheckpointTxg(reader.get_u64());
        reader.skip(usize_from_u64(WIDE_CHECKSUM_BYTES));
        let instance_table = NodePointer::read_from(&mut reader);
        let mapping_root = NodePointer::read_from(&mut reader);
        let allocation_record_tree_root = NodePointer::read_from(&mut reader);
        Some(Self { fsid, unmount_marker, instance, checkpoint_txg, tree_table, tree_identifier_watermark, rollback_floor, instance_table, mapping_root, allocation_record_tree_root })
    }
}

/// 系统配置字段表（D22 已定项 9 + 已定项 15）：**481 字节**，512 槽内余 31。
/// 没有「间接目录单元指针 59」（第一版显式留白，不是留位），几何段有三个字段：
/// 单元区起始槽号 8（D3 已定项 10 ④）、mkfs 时的 io_min 4 与固定结构槽距 4（D2 已定项 19）；
/// 另有四个：自举头的写入者身份 20 与校验和算法标识 1、
/// 几何段的 mkfs 时 physical_block_size 4 与扩展点声明值 N 4。
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct SystemConfiguration {
    pub(super) fsid: [u8; 16],
    pub(super) this_device: DeviceIdentity,
    pub(super) device_count: u32,
    pub(super) slot_generation: u64,
    pub(super) region_devices: [DeviceIdentity; 3],
    pub(super) journal_tail: u64,
    pub(super) journal_instance: InstanceGeneration,
    /// D22（单元原子性怎么合成） 已定项 9 字段表末行：回退下界 F 另记一份，
    /// 住 journal 实例代号之后 [481, 489)；这条流里没有抬 F，恒为 `CheckpointTxg(0)`（D16 已定项 1）。
    pub(super) rollback_floor: CheckpointTxg,
}

/// magic 4 + 格式版本 2 + feature bits 96 + fsid 16 + 写入者身份 20 + 校验和算法标识 1 + 本盘设备号 4 + 设备数 4 + 槽世代号 8。
pub(super) const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 4 + 2 + 96 + 16 + 20 + 1 + 4 + 4 + 8;
/// fsid 住自举头里 magic 4 + 格式版本 2 + feature bits 96 之后。
pub(super) const SYSTEM_CONFIGURATION_FSID_OFFSET: usize = 4 + 2 + 96;
const SYSTEM_CONFIGURATION_REGION_DEVICES_OFFSET: usize = 379;
pub(super) const SYSTEM_CONFIGURATION_TAIL_OFFSET: usize = 469;
/// D22（单元原子性怎么合成） 已定项 9 字段表末行：journal 实例代号（477，宽 4）之后
/// 是回退下界 F，按行累加得偏移 481（`.claude/kb/layout/01-first-txn.md`「一」字段表）。
pub(super) const SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET: usize = 481;
/// D2（RAID 条带策略） 已定项 19：固定结构槽距 = max(4096, mkfs 时探测的 io_min)，两个数各占系统配置一个 4 字节字段。
const FIXED_STRUCTURE_SLOT_SPACING: u32 = 4096;
const MKFS_MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
/// D2（RAID 条带策略） 已定项 18：第一版系统配置里 w_max 与 g 都写 4；g 挂载时按可写设备数夹取。
const SYSTEM_CONFIGURATION_MAXIMUM_WIDTH: u8 = 4;
const SYSTEM_CONFIGURATION_GROUP_SIZE: u8 = 4;

impl SystemConfiguration {
    pub(super) fn to_slot(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES));
        writer.put(&SYSTEM_CONFIGURATION_MAGIC);
        writer.put_u16(FORMAT_VERSION);
        writer.put_u8(WRITTEN_FIRST_INCOMPAT_BYTE); // feature bits：incompat 位 1 = 布局身份（D15 已定项 4；位 0 空出）、位 2 = inode 记录带截断字段
        writer.skip(95); // 其余 incompat 位与 compat_ro / compat 两张位图全 0
        writer.assert_position(SYSTEM_CONFIGURATION_FSID_OFFSET as u64, "系统配置 fsid");
        writer.put(&self.fsid);
        // 写入者身份（D22 已定项 9）：实现标识 16 字节 ASCII 零补齐 + 版本 4 字节。
        let mut writer_identity = [0u8; WRITER_IDENTITY_NAME_BYTES];
        writer_identity[..WRITER_IDENTITY_NAME.len()].copy_from_slice(WRITER_IDENTITY_NAME);
        writer.put(&writer_identity);
        writer.put_u32(WRITER_IDENTITY_VERSION);
        writer.put_u8(CHECKSUM_ALGORITHM_CRC32_CASTAGNOLI);
        writer.put_u32(self.this_device.0);
        writer.put_u32(self.device_count);
        writer.put_u64(self.slot_generation);
        writer.assert_position(SYSTEM_CONFIGURATION_CHECKSUM_OFFSET as u64, "系统配置整槽校验和");
        writer.skip(usize_from_u64(WIDE_CHECKSUM_BYTES));
        writer.skip(16 + 12 + 4); // 系统配置 MAC、nonce 水位、KDF 标识（4，D22 已定项 9）
        writer.put_u8(0); // 加密类型：关
        writer.put_u8(16); // MAC 长度声明
        writer.skip(80); // 主密钥槽：内联进槽，加密关时全 0（D22 已定项 9）
        writer.put_u32(u32_from_u64(NODE_BYTES));
        writer.put_u32(u32_from_u64(DATA_UNIT_BYTES));
        writer.put_u32(u32_from_u64(SLOT_BYTES));
        writer.put_u32(u32_from_u64(LOC_ENTRY));
        // 两个几何字段，排在位置条目宽度之后（字节表一那一节的行序）。
        writer.put_u32(MKFS_PHYSICAL_BLOCK_BYTES);
        writer.put_u32(EXTENSION_POINT_DECLARED_BYTES);
        writer.put_u64(JOURNAL_START_SLOT);
        writer.put_u64(JOURNAL_RING_BYTES);
        writer.put_u32(u32_from_u64(JOURNAL_RECORD_BYTES));
        // D23（journal 的角色与格式） 已定项 18：在飞记录数上限 = 环槽数 ÷ F。
        writer.put_u32(u32::try_from(JOURNAL_IN_FLIGHT_RECORD_LIMIT).expect("在飞上限 4 字节"));
        // journal 最坏占用 = 在飞上限 × 记录宽（字节表 `layout/01-first-txn.md` 那一行逐字
        // 「268 435 456（65536 条 × 4096）」）。
        writer.put_u64(JOURNAL_WORST_CASE_BYTES);
        writer.put_u32(u32::try_from(JOURNAL_SAFETY_FACTOR).expect("安全系数 4 字节"));
        writer.put_u8(u8_from_u64(RING_REGIONS));
        writer.put_u8(u8_from_u64(RING_SLOTS_PER_REGION));
        writer.put_u32(u32_from_u64(RING_PRIME_STEP));
        writer.put_u32(u32_from_u64(RING_CHUNK_BYTES));
        writer.put_u64(RING_START_OFFSET / SLOT_BYTES);
        writer.assert_position(SYSTEM_CONFIGURATION_REGION_DEVICES_OFFSET as u64, "根环逐区域设备身份");
        for region_device in &self.region_devices {
            writer.put_u32(region_device.0);
        }
        writer.put_u8(SYSTEM_CONFIGURATION_MAXIMUM_WIDTH);
        writer.put_u8(SYSTEM_CONFIGURATION_GROUP_SIZE);
        writer.skip(24); // 映射来源
        // D22 已定项 15 加的三个几何字段，排在「映射来源」之后。
        writer.put_u64(UNIT_AREA_START_SLOT);
        writer.put_u32(MKFS_MINIMUM_INPUT_OUTPUT_BYTES);
        writer.put_u32(FIXED_STRUCTURE_SLOT_SPACING);
        writer.put_u32(5000); // T_time 毫秒
        writer.put_u64(2 << 30); // T_dirty
        writer.skip(24); // 整理三条水位：第一版恒 0 = 内置默认（D22 已定项 15）
        writer.assert_position(SYSTEM_CONFIGURATION_TAIL_OFFSET as u64, "journal tail");
        writer.put_u64(self.journal_tail);
        writer.put_u32(self.journal_instance.0);
        // D22（单元原子性怎么合成） 已定项 9 字段表末行：回退下界 F 8 字节，
        // 住 [481, 489)，journal 实例代号之后。
        writer.assert_position(SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET as u64, "系统配置回退下界 F");
        writer.put_u64(self.rollback_floor.0);
        writer.assert_position(SYSTEM_CONFIGURATION_BYTES, "系统配置");
        let mut bytes = writer.bytes;
        // 「整槽校验和」：覆盖整个 4096 槽含补齐、自身按 0 参与（D18 已定项 17）。
        let digest = wide_checksum_with_field_zeroed(&bytes, usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES), SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
        bytes[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        bytes
    }

    pub(super) fn parse_slot(bytes: &[u8]) -> Option<Self> {
        if bytes[..4] != SYSTEM_CONFIGURATION_MAGIC {
            return None;
        }
        if wide_checksum_with_field_zeroed(bytes, usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES), SYSTEM_CONFIGURATION_CHECKSUM_OFFSET) != bytes[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32] {
            return None;
        }
        if !incompat_bits_are_mountable(bytes) {
            return None;
        }
        let mut reader = ByteReader::at(bytes, SYSTEM_CONFIGURATION_FSID_OFFSET);
        let fsid: [u8; 16] = reader.take(16).try_into().expect("切了 16 字节");
        reader.skip(WRITER_IDENTITY_NAME_BYTES + 4); // 写入者身份：读者不判它，只要在
        if reader.get_u8() != CHECKSUM_ALGORITHM_CRC32_CASTAGNOLI {
            return None; // I-2.2（校验和算法全盘一致）：算法标识不是已登记的 CRC-32C 就不认这个槽
        }
        let this_device = DeviceIdentity(reader.get_u32());
        let device_count = reader.get_u32();
        let slot_generation = reader.get_u64();
        let mut region_reader = ByteReader::at(bytes, SYSTEM_CONFIGURATION_REGION_DEVICES_OFFSET);
        let region_devices = [DeviceIdentity(region_reader.get_u32()), DeviceIdentity(region_reader.get_u32()), DeviceIdentity(region_reader.get_u32())];
        let mut tail_reader = ByteReader::at(bytes, SYSTEM_CONFIGURATION_TAIL_OFFSET);
        let journal_tail = tail_reader.get_u64();
        let journal_instance = InstanceGeneration(tail_reader.get_u32());
        let mut rollback_floor_reader = ByteReader::at(bytes, SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET);
        let rollback_floor = CheckpointTxg(rollback_floor_reader.get_u64());
        Some(Self { fsid, this_device, device_count, slot_generation, region_devices, journal_tail, journal_instance, rollback_floor })
    }
}

/// D15 已定项 4 与 fs-design「格式层的让非法状态无法表示」：incompat 位图里有读者不认识的位、
/// 或第一条 SSD 线那一位没置（没有布局身份），都拒绝挂载；位 2 没置（inode 记录没有截断字段的布局）同样拒；compat_ro / compat 两张位图不认识随便，读者不看。
pub(super) fn incompat_bits_are_mountable(slot: &[u8]) -> bool {
    let incompat = &slot[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET..SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + FEATURE_BITMAP_BYTES];
    let unknown_in_first_byte = incompat[0] & !SUPPORTED_INCOMPAT_BITS;
    let unknown_in_rest = incompat[1..].iter().any(|byte| *byte != 0);
    let has_layout_identity = incompat[0] & INCOMPAT_LAYOUT_IDENTITY_BIT != 0;
    let has_truncation_fields = incompat[0] & INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT != 0;
    unknown_in_first_byte == 0 && !unknown_in_rest && has_layout_identity && has_truncation_fields
}

/// 根环区域 r 的起点（设备内偏移）：`1 MiB + r × P × chunk`（字节表零，预想按设备内偏移读）。
pub(super) fn ring_region_offset(region: u64) -> DeviceOffset {
    DeviceOffset(RING_START_OFFSET + region * RING_PRIME_STEP * RING_CHUNK_BYTES)
}

pub(super) fn ring_slot_offset(region: u64, slot: u64) -> DeviceOffset {
    DeviceOffset(ring_region_offset(region).0 + slot * RING_SLOT_SPACING)
}

/// 第 n 次发布写区域 `n mod R` 的槽 `(n div R) mod S`（字节表一，区域内公式是预想）。
pub(super) fn ring_target_for_publish(checkpoint_txg: CheckpointTxg) -> (u64, u64) {
    (checkpoint_txg.0 % RING_REGIONS, (checkpoint_txg.0 / RING_REGIONS) % RING_SLOTS_PER_REGION)
}

/// 一个单元类占多少字节：码 1 与码 3 都是 32768，码 2 是 16384。点名项不带单元大小，靠类标签定（D23 已定项 17）。
pub(super) fn unit_bytes_for_class(unit_class: u8) -> Option<u64> {
    match unit_class {
        UNIT_CLASS_DATA | UNIT_CLASS_PACKED => Some(DATA_UNIT_BYTES),
        UNIT_CLASS_INDEX_NODE => Some(NODE_BYTES),
        _ => None,
    }
}

/// 点名项 56（D23（journal 的角色与格式） 已定项 17 的字段表）：
/// 位置条目 14 × 2（偏移 0）、单元类型标签 1（28）、出生树 8（29）、出生 txg 8（37）、
/// key 尾段 10（45：码 1 写序；码 2 / 码 3 实例代号 4 + 出生序号 4 + 补零 2）、flags 1（55）。
/// 不另带载荷 CRC——位置条目各带 4 字节校验和；重放从点名项直接凑出映射 key。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct NamedUnit {
    pub(super) locations: [LocationEntry; 2],
    pub(super) unit_class: u8,
    pub(super) birth_tree: TreeIdentifier,
    pub(super) birth_txg: CheckpointTxg,
    pub(super) key_tail: [u8; 10],
}

impl NamedUnit {
    pub(super) fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        for location in &self.locations {
            location.write_to(writer);
        }
        writer.put_u8(self.unit_class);
        writer.put_u64(self.birth_tree.0);
        writer.put_u64(self.birth_txg.0);
        writer.put(&self.key_tail);
        writer.put_u8(0); // flags：第一版恒 0、非 0 拒收（D22 已定项 17）
        assert_eq!(writer.position() - start, usize_from_u64(JOURNAL_NAMED_ENTRY_BYTES));
    }
    pub(super) fn read_from(reader: &mut ByteReader) -> Self {
        let locations = [LocationEntry::read_from(reader), LocationEntry::read_from(reader)];
        let unit_class = reader.get_u8();
        let birth_tree = TreeIdentifier(reader.get_u64());
        let birth_txg = CheckpointTxg(reader.get_u64());
        let key_tail: [u8; 10] = reader.take(10).try_into().expect("切了 10 字节");
        reader.skip(1);
        Self { locations, unit_class, birth_tree, birth_txg, key_tail }
    }
    /// 重放不查单元就能凑出这一项的中央映射 key（D23 已定项 17 末句）：一律 27 字节——
    /// 码 1 的尾段是 10 字节写序，码 2 / 码 3 的尾段是实例代号 4 + 出生序号 4 + 补零 2，两者加起来都正好是 key 的后 10 字节。
    pub(super) fn mapping_key(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize_from_u64(MAPPING_KEY_BYTES));
        writer.put_u8(self.unit_class);
        writer.put_u64(self.birth_tree.0);
        writer.put_u64(self.birth_txg.0);
        writer.put(&self.key_tail);
        writer.assert_position(MAPPING_KEY_BYTES, "点名项凑出来的映射 key");
        writer.bytes
    }
}

#[cfg(test)]
#[path = "root_record_and_system_configuration_tests.rs"]
mod tests;
