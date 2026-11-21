//! 位置条目、指针头、节点指针与数据指针、数据单元的身份、公共前缀与头校验和、数据单元的拼装。

use e7_index_bench::usize_from_u64;
use super::*;

// ───────────────────────── 盘上结构：位置条目、指针、单元头 ─────────────────────────

/// D19（块指针的结构与宽度预算） 已定项 4：设备 4 + 16 KiB 槽号 6 + 密文校验和 4（无加密时是整单元 CRC32C，字节表三预想）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct LocationEntry {
    pub(super) device: DeviceIdentity,
    pub(super) slot: SlotNumber,
    pub(super) unit_checksum: u32,
}

impl LocationEntry {
    pub(super) fn write_to(&self, writer: &mut ByteWriter) {
        writer.put_u32(self.device.0);
        writer.put_six_byte_unsigned(self.slot.0);
        writer.put_u32(self.unit_checksum);
    }
    pub(super) fn read_from(reader: &mut ByteReader) -> Self {
        Self { device: DeviceIdentity(reader.get_u32()), slot: SlotNumber(reader.get_six_byte_unsigned()), unit_checksum: reader.get_u32() }
    }
}

/// 指针头部 50（D19 已定项 7 / 已定项 11 的偏移表）：MAC 16（0）+ nonce 12（16）+ 算法类型 1（28）
/// + 压缩算法码 1（29）+ 压后长度 2（30）+ extent 距起点偏移 2（32）+ 出生树 8（34）+ 出生 txg 8（42）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct PointerHead {
    pub(super) birth_tree: TreeIdentifier,
    pub(super) birth_txg: CheckpointTxg,
}

impl PointerHead {
    pub(super) fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        writer.skip(16 + 12); // MAC 与 nonce：第一版留位、全 0（D9（加密） 已定项 2 / 已定项 4）
        writer.put_u8(0); // 算法类型：未加密
        writer.put_u8(POINTER_COMPRESSION_CODE_NONE);
        writer.put_u16(POINTER_COMPRESSED_LENGTH_NONE);
        writer.put_u16(0); // extent 距起点偏移：第一版恒 0，切分是预留特性
        assert_eq!(writer.position() - start, 34, "出生树要落在指针头部偏移 34");
        writer.put_u64(self.birth_tree.0);
        writer.put_u64(self.birth_txg.0);
        assert_eq!(writer.position() - start, usize_from_u64(POINTER_HEAD_BYTES));
    }
    pub(super) fn read_from(reader: &mut ByteReader) -> Self {
        reader.skip(16 + 12 + 1 + 1 + 2 + 2);
        Self { birth_tree: TreeIdentifier(reader.get_u64()), birth_txg: CheckpointTxg(reader.get_u64()) }
    }
}

/// 指向码 2 / 码 3 的指针 86（D19 已定项 8）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct NodePointer {
    pub(super) head: PointerHead,
    pub(super) locations: [LocationEntry; 2],
    pub(super) instance: InstanceGeneration,
    pub(super) birth_sequence: BirthSequence,
}

impl NodePointer {
    /// 空树的根：位置条目、实例代号、出生序号全零（D6 已定项 2 / D5 已定项 6 的 day-1 注册树在新池新建文件里就是这样）。
    pub(super) fn empty_root() -> NodePointer {
        NodePointer {
            head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE), birth_txg: CheckpointTxg(0) },
            locations: [
                LocationEntry { device: DeviceIdentity(0), slot: SlotNumber(0), unit_checksum: 0 },
                LocationEntry { device: DeviceIdentity(0), slot: SlotNumber(0), unit_checksum: 0 },
            ],
            instance: InstanceGeneration(0),
            birth_sequence: BirthSequence(0),
        }
    }
    pub(super) fn is_empty_root(&self) -> bool {
        self.locations.iter().all(|location| location.slot.0 == 0 && location.unit_checksum == 0) && self.birth_sequence.0 == 0
    }
    pub(super) fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        self.head.write_to(writer);
        for location in &self.locations {
            location.write_to(writer);
        }
        writer.put_u32(self.instance.0);
        writer.put_u32(self.birth_sequence.0);
        assert_eq!(writer.position() - start, usize_from_u64(NODE_POINTER_BYTES));
    }
    pub(super) fn read_from(reader: &mut ByteReader) -> Self {
        let head = PointerHead::read_from(reader);
        let locations = [LocationEntry::read_from(reader), LocationEntry::read_from(reader)];
        Self { head, locations, instance: InstanceGeneration(reader.get_u32()), birth_sequence: BirthSequence(reader.get_u32()) }
    }
}

/// 指向码 1 的指针 88（D19 已定项 8）：头部 + 位置条目 × 2 + 写序 10。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct DataPointer {
    pub(super) head: PointerHead,
    pub(super) locations: [LocationEntry; 2],
    pub(super) write_order: WriteOrder,
}

impl DataPointer {
    pub(super) fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        self.head.write_to(writer);
        for location in &self.locations {
            location.write_to(writer);
        }
        writer.put_u32(self.write_order.instance.0);
        writer.put_six_byte_unsigned(self.write_order.transaction.0);
        assert_eq!(writer.position() - start, usize_from_u64(DATA_POINTER_BYTES));
    }
    pub(super) fn read_from(reader: &mut ByteReader) -> Self {
        let head = PointerHead::read_from(reader);
        let locations = [LocationEntry::read_from(reader), LocationEntry::read_from(reader)];
        let instance = InstanceGeneration(reader.get_u32());
        let transaction = TransactionNumber(reader.get_six_byte_unsigned());
        Self { head, locations, write_order: WriteOrder { instance, transaction } }
    }
}

/// D18 已定项 3 的五元组（33 字节）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct DataUnitIdentity {
    pub(super) tree: TreeIdentifier,
    pub(super) object: u64,
    pub(super) object_birth: CheckpointTxg,
    pub(super) anchor_offset: u64,
}

/// 共同前缀 42：magic 4 + 版本 2 + 类标签 1 + flags 1 + 声明长度 2 + 头校验和 32。头校验和最后填。
pub(super) fn write_common_prefix(writer: &mut ByteWriter, unit_class: u8, declared_length: u16) {
    writer.put(&UNIT_MAGIC);
    writer.put_u16(FORMAT_VERSION);
    writer.put_u8(unit_class);
    writer.put_u8(0);
    writer.put_u16(declared_length);
    writer.skip(usize_from_u64(WIDE_CHECKSUM_BYTES));
    writer.assert_position(COMMON_PREFIX_BYTES, "共同前缀");
}

const HEADER_CHECKSUM_OFFSET: usize = 10;

/// 头校验和覆盖偏移 0 到该类明文头末尾（I-2.4），自身按 0 参与。
pub(super) fn seal_header_checksum(bytes: &mut [u8], header_end: usize) {
    let digest = wide_checksum_with_field_zeroed(bytes, header_end, HEADER_CHECKSUM_OFFSET);
    bytes[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
}

pub(super) fn header_checksum_holds(bytes: &[u8], header_end: usize) -> bool {
    wide_checksum_with_field_zeroed(bytes, header_end, HEADER_CHECKSUM_OFFSET) == bytes[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 32]
}

/// fsid 在单元头里是 8 字节（D18 已定项 7）；字节表二预想取系统配置 fsid 的低 8 字节。
pub(super) fn unit_fsid(fsid: &[u8; 16]) -> u64 {
    u64::from_le_bytes(fsid[..8].try_into().expect("切了 8 字节"))
}

/// 码 1 数据单元 32768：头 105 + 预留 29 + 载荷从 134 起（声明长度之后补 0，补齐参与 CRC——I-2.3）。
pub(super) fn build_data_unit(identity: DataUnitIdentity, birth_txg: CheckpointTxg, fsid: &[u8; 16], write_order: WriteOrder, payload: &[u8]) -> Vec<u8> {
    let payload_start = usize_from_u64(DATA_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES);
    assert!(payload.len() <= usize_from_u64(DATA_UNIT_BYTES) - payload_start, "载荷装不进一个单元");
    let mut writer = ByteWriter::new(usize_from_u64(DATA_UNIT_BYTES));
    write_common_prefix(&mut writer, UNIT_CLASS_DATA, u16::try_from(payload.len()).expect("声明长度 2 字节"));
    writer.put_u8(UNIT_CLASS_DATA);
    writer.put_u64(identity.tree.0);
    writer.put_u64(identity.object);
    writer.put_u64(identity.object_birth.0);
    writer.put_u64(identity.anchor_offset);
    writer.put_u64(birth_txg.0);
    writer.put_u64(unit_fsid(fsid));
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction.0);
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.assert_position(DATA_UNIT_HEADER_BYTES, "数据单元头");
    writer.skip(usize_from_u64(NONCE_MAC_RESERVED_BYTES));
    writer.put(payload);
    let mut bytes = writer.bytes;
    // 载荷 CRC 覆盖偏移 105 到单元末尾（字节表二：含预留位与补齐）。
    let payload_crc = castagnoli_crc32(&bytes[usize_from_u64(DATA_UNIT_HEADER_BYTES)..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    seal_header_checksum(&mut bytes, usize_from_u64(DATA_UNIT_HEADER_BYTES));
    bytes
}
