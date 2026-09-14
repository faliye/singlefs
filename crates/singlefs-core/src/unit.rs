//! 单元的头：共同前缀 42（D18（块里携带什么信息） 已定项 7），码 2 索引节点头（已定项 18），码 3 打包记录单元头（已定项 11）。

use singlefs_format::{
    index_node_header_bytes, DATA_UNIT_BYTES, DATA_UNIT_HEADER_BYTES, NODE_BYTES,
    NONCE_MAC_ALGORITHM_RESERVED_BYTES, PACKED_UNIT_HEADER_BYTES, UNIT_COMMON_PREFIX_BYTES,
    WIDE_CHECKSUM_BYTES,
};

use crate::address::{CheckpointTxg, InstanceGeneration, TreeIdentifier};
use crate::bytes::{ByteReader, ByteWriter};
use crate::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use crate::pointer::BirthSequence;

pub const UNIT_MAGIC: [u8; 4] = *b"SFSU";
pub const FORMAT_VERSION: u16 = 1;
/// 单元类型标签（D18（块里携带什么信息） 已定项 11 的登记表）。
pub const UNIT_CLASS_DATA: u8 = 1;
pub const UNIT_CLASS_INDEX_NODE: u8 = 2;
pub const UNIT_CLASS_PACKED: u8 = 3;
/// 打包记录类型（第二级登记表）：2 = inode 记录，4 = 实例表。
pub const PACKED_TYPE_INODE: u16 = 2;
pub const PACKED_TYPE_INSTANCE_TABLE: u16 = 4;
/// 头校验和住共同前缀偏移 10。
pub const HEADER_CHECKSUM_OFFSET: usize = 10;

/// 写序：实例代号 4 + 事务号低 48 位（D18（块里携带什么信息） 已定项 7）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteOrder {
    pub instance: InstanceGeneration,
    pub transaction: u64,
}

fn reserved_bytes() -> usize {
    usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29")
}

/// 单元头里的 fsid 是 8 字节：超级块 fsid 的低 8 字节（D18（块里携带什么信息） 已定项 7）。
#[must_use]
pub fn unit_filesystem_identifier(filesystem_identifier: &[u8; 16]) -> u64 {
    u64::from_le_bytes(filesystem_identifier[..8].try_into().expect("切了 8 字节"))
}

pub fn write_common_prefix(writer: &mut ByteWriter, unit_class: u8, declared_length: u16) {
    writer.put(&UNIT_MAGIC);
    writer.put_u16(FORMAT_VERSION);
    writer.put_u8(unit_class);
    writer.put_u8(0);
    writer.put_u16(declared_length);
    writer.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
    writer.assert_position(UNIT_COMMON_PREFIX_BYTES, "共同前缀");
}

/// 头校验和覆盖 [0, 明文头末尾)，自身按 0 参与（I-2.4）。
pub fn seal_header_checksum(bytes: &mut [u8], header_end: usize) {
    let digest = wide_checksum_with_field_zeroed(bytes, header_end, HEADER_CHECKSUM_OFFSET);
    bytes[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
}

/// 码 3 打包记录单元的四元组身份（出生树、打包记录类型、容器号、容器出生代）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedIdentity {
    pub birth_tree: TreeIdentifier,
    pub record_type: u16,
    pub container: u64,
    pub container_birth: CheckpointTxg,
}

/// 码 3 打包记录单元 32768：类身份段 107 + 预留位 29，记录区从 136 起；载荷 CRC 罩 [107, 32768)。
#[must_use]
pub fn build_packed_unit(
    identity: PackedIdentity,
    record_width: u16,
    records: &[Vec<u8>],
    birth_txg: CheckpointTxg,
    filesystem_identifier: &[u8; 16],
    write_order: WriteOrder,
    birth_sequence: BirthSequence,
) -> Vec<u8> {
    let header_end = usize::try_from(PACKED_UNIT_HEADER_BYTES).expect("107");
    let declared_length =
        u16::try_from(records.len() * usize::from(record_width)).expect("声明长度 2 字节");
    let mut writer = ByteWriter::new(usize::try_from(DATA_UNIT_BYTES).expect("32768"));
    write_common_prefix(&mut writer, UNIT_CLASS_PACKED, declared_length);
    writer.put_u8(UNIT_CLASS_PACKED);
    writer.put_u64(identity.birth_tree.0);
    writer.put_u16(identity.record_type);
    writer.put_u64(identity.container);
    writer.put_u64(identity.container_birth.0);
    writer.assert_position(69, "码 3 记录数");
    writer.put_u16(u16::try_from(records.len()).expect("记录数 2 字节"));
    writer.put_u16(record_width);
    writer.assert_position(73, "码 3 诞生代号");
    writer.put_u64(birth_txg.0);
    writer.put_u64(unit_filesystem_identifier(filesystem_identifier));
    writer.assert_position(89, "码 3 载荷校验和");
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.assert_position(93, "码 3 写序");
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction);
    writer.assert_position(103, "码 3 出生序号");
    writer.put_u32(birth_sequence.0);
    writer.assert_position(PACKED_UNIT_HEADER_BYTES, "码 3 头");
    writer.skip(reserved_bytes());
    for record in records {
        assert_eq!(
            record.len(),
            usize::from(record_width),
            "记录宽与头里写的不一致"
        );
        writer.put(record);
    }
    let mut bytes = writer.into_bytes();
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    seal_header_checksum(&mut bytes, header_end);
    bytes
}

/// 码 2 索引节点 16384（D18（块里携带什么信息） 已定项 18 的偏移表，key 宽住 51）：
/// 42 树 ID / 50 层级 / 51 key 宽 / 52 key 区间 2k / 52+2k 诞生代号 / 60+2k fsid / 68+2k 写序 4 / 72+2k 出生序号 /
/// 76+2k 载荷 CRC / 80+2k 预留 2 / 82+2k 条目数 / 84+2k 条目宽 / 86+2k 预留位 29 / 115+2k 条目区。
/// 头校验和罩 [0, 86+2k)，载荷 CRC 罩 [86+2k, 16384)；条目定宽、key 打头；声明长度 = 条目数 × 条目宽。
#[allow(
    clippy::too_many_arguments,
    reason = "字段表就是这么多段，收成结构体只会多一层没人验的名字"
)]
#[must_use]
pub fn build_index_node(
    tree: TreeIdentifier,
    level: u8,
    key_width: usize,
    smallest_key: &[u8],
    largest_key: &[u8],
    birth_txg: CheckpointTxg,
    filesystem_identifier: &[u8; 16],
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
    entry_width: u16,
    entries: &[Vec<u8>],
) -> Vec<u8> {
    assert_eq!(smallest_key.len(), key_width);
    assert_eq!(largest_key.len(), key_width);
    let header_end = usize::try_from(
        index_node_header_bytes(u64::try_from(key_width).expect("key 宽"))
            - NONCE_MAC_ALGORITHM_RESERVED_BYTES,
    )
    .expect("头宽");
    let entries_start = header_end + reserved_bytes();
    let declared_length = entries.len() * usize::from(entry_width);
    assert!(
        entries_start + declared_length <= usize::try_from(NODE_BYTES).expect("16384"),
        "条目装不进一个节点"
    );
    let mut writer = ByteWriter::new(usize::try_from(NODE_BYTES).expect("16384"));
    write_common_prefix(
        &mut writer,
        UNIT_CLASS_INDEX_NODE,
        u16::try_from(declared_length).expect("声明长度 2 字节"),
    );
    writer.put_u64(tree.0);
    writer.put_u8(level);
    writer.assert_position(51, "码 2 的 key 宽字段");
    writer.put_u8(u8::try_from(key_width).expect("key 宽 1 字节"));
    writer.put(smallest_key);
    writer.put(largest_key);
    writer.put_u64(birth_txg.0);
    writer.put_u64(unit_filesystem_identifier(filesystem_identifier));
    writer.put_u32(instance.0);
    writer.put_u32(birth_sequence.0);
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.skip(2);
    writer.put_u16(u16::try_from(entries.len()).expect("条目数 2 字节"));
    writer.put_u16(entry_width);
    writer.assert_position(u64::try_from(header_end).expect("头末尾"), "码 2 头");
    writer.skip(reserved_bytes());
    for entry in entries {
        assert_eq!(
            entry.len(),
            usize::from(entry_width),
            "条目宽与头里写的不一致"
        );
        assert!(
            entry.len() >= key_width,
            "条目里 key 一律是前 key 宽 个字节"
        );
        writer.put(entry);
    }
    let mut bytes = writer.into_bytes();
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    seal_header_checksum(&mut bytes, header_end);
    bytes
}

/// 码 1 数据单元的五元组身份（D18（块里携带什么信息） 已定项 3：类标签副本 1 + 树 ID 8 + 对象 ID 8 + 对象出生代 8 + 锚点偏移 8）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataUnitIdentity {
    pub tree: TreeIdentifier,
    pub object: u64,
    pub object_birth: CheckpointTxg,
    pub anchor_offset: u64,
}

/// 码 1 数据单元 32768：类身份段 105（偏移 42 标签副本 / 43 树 ID / 51 对象 ID / 59 对象出生代 / 67 锚点 / 75 诞生代号 / 83 fsid / 91 写序 / 101 载荷 CRC）
/// + 预留位 29，载荷从 134 起、声明长度之后补 0；载荷 CRC 罩 [105, 32768)，头校验和罩 [0, 105)。
#[must_use]
pub fn build_data_unit(
    identity: DataUnitIdentity,
    birth_txg: CheckpointTxg,
    filesystem_identifier: &[u8; 16],
    write_order: WriteOrder,
    payload: &[u8],
) -> Vec<u8> {
    let header_end = usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105");
    let payload_start = header_end + reserved_bytes();
    let unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    assert!(
        payload.len() <= unit_bytes - payload_start,
        "载荷装不进一个单元"
    );
    let mut writer = ByteWriter::new(unit_bytes);
    write_common_prefix(
        &mut writer,
        UNIT_CLASS_DATA,
        u16::try_from(payload.len()).expect("声明长度 2 字节"),
    );
    writer.put_u8(UNIT_CLASS_DATA);
    writer.put_u64(identity.tree.0);
    writer.put_u64(identity.object);
    writer.put_u64(identity.object_birth.0);
    writer.put_u64(identity.anchor_offset);
    writer.assert_position(75, "数据单元诞生代号");
    writer.put_u64(birth_txg.0);
    writer.put_u64(unit_filesystem_identifier(filesystem_identifier));
    writer.assert_position(91, "数据单元写序");
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction);
    writer.assert_position(101, "数据单元载荷 CRC");
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.assert_position(DATA_UNIT_HEADER_BYTES, "数据单元头");
    writer.skip(reserved_bytes());
    writer.put(payload);
    let mut bytes = writer.into_bytes();
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    seal_header_checksum(&mut bytes, header_end);
    bytes
}

/// 数据单元的声明长度：共同前缀偏移 8 的那 2 字节。
#[must_use]
pub fn declared_length(unit: &[u8]) -> u16 {
    u16::from_le_bytes([unit[8], unit[9]])
}

/// 读者判单元时能出的错，按调用方要做的决定分：读不到 / 头坏了 / 载荷坏了 / 结构字段自相矛盾。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitError {
    TooShort,
    BadMagic,
    UnknownFormatVersion,
    NonZeroFlags,
    WrongUnitClass { expected: u8, found: u8 },
    HeaderChecksumMismatch,
    PayloadChecksumMismatch,
    Structure(&'static str),
}

/// 共同前缀四关（magic、版本、类标签、flags）+ 头校验和；返回声明长度。
fn check_common_prefix(
    bytes: &[u8],
    expected_class: u8,
    header_end: usize,
) -> Result<u16, UnitError> {
    if bytes.len() < header_end {
        return Err(UnitError::TooShort);
    }
    if bytes[..4] != UNIT_MAGIC {
        return Err(UnitError::BadMagic);
    }
    if u16::from_le_bytes([bytes[4], bytes[5]]) != FORMAT_VERSION {
        return Err(UnitError::UnknownFormatVersion);
    }
    if bytes[6] != expected_class {
        return Err(UnitError::WrongUnitClass {
            expected: expected_class,
            found: bytes[6],
        });
    }
    if bytes[7] != 0 {
        return Err(UnitError::NonZeroFlags);
    }
    if !crate::checksum::wide_checksum_field_holds(bytes, header_end, HEADER_CHECKSUM_OFFSET) {
        return Err(UnitError::HeaderChecksumMismatch);
    }
    Ok(declared_length(bytes))
}

fn check_payload_checksum(
    bytes: &[u8],
    header_end: usize,
    payload_crc_offset: usize,
) -> Result<(), UnitError> {
    let mut reader = ByteReader::at(bytes, payload_crc_offset);
    if reader.get_u32() != crc32_castagnoli(&bytes[header_end..]) {
        return Err(UnitError::PayloadChecksumMismatch);
    }
    Ok(())
}

/// 码 2 索引节点解出来的头与条目。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexNodeHeader {
    pub tree: TreeIdentifier,
    pub level: u8,
    pub key_width: usize,
    pub smallest_key: Vec<u8>,
    pub largest_key: Vec<u8>,
    pub birth_txg: CheckpointTxg,
    pub filesystem_identifier: u64,
    pub instance: InstanceGeneration,
    pub birth_sequence: BirthSequence,
    pub entry_width: usize,
    pub entries: Vec<Vec<u8>>,
}

/// 解一个码 2 节点：key 宽从偏移 51 自述、头校验和罩 [0, 86+2k)、载荷 CRC 罩 [86+2k, 16384)、声明长度 = 条目数 × 条目宽。
pub fn parse_index_node(bytes: &[u8]) -> Result<IndexNodeHeader, UnitError> {
    if bytes.len() != usize::try_from(NODE_BYTES).expect("16384") {
        return Err(UnitError::TooShort);
    }
    let key_width = usize::from(bytes[51]);
    let header_end = usize::try_from(
        index_node_header_bytes(u64::try_from(key_width).expect("key 宽"))
            - NONCE_MAC_ALGORITHM_RESERVED_BYTES,
    )
    .expect("头宽");
    let declared_length = usize::from(check_common_prefix(
        bytes,
        UNIT_CLASS_INDEX_NODE,
        header_end,
    )?);
    check_payload_checksum(bytes, header_end, header_end - 10)?;
    let mut reader = ByteReader::at(bytes, 42);
    let tree = TreeIdentifier(reader.get_u64());
    let level = reader.get_u8();
    reader.skip(1);
    let smallest_key = reader.take(key_width).to_vec();
    let largest_key = reader.take(key_width).to_vec();
    let birth_txg = CheckpointTxg(reader.get_u64());
    let filesystem_identifier = reader.get_u64();
    let instance = InstanceGeneration(reader.get_u32());
    let birth_sequence = BirthSequence(reader.get_u32());
    reader.skip(4 + 2);
    let entry_count = usize::from(reader.get_u16());
    let entry_width = usize::from(reader.get_u16());
    assert_eq!(reader.position(), header_end);
    if entry_count * entry_width != declared_length {
        return Err(UnitError::Structure("声明长度 ≠ 条目数 × 条目宽"));
    }
    if entry_width < key_width {
        return Err(UnitError::Structure("条目宽小于 key 宽"));
    }
    let entries_start = header_end + reserved_bytes();
    if entries_start + declared_length > bytes.len() {
        return Err(UnitError::Structure("条目区越过节点末尾"));
    }
    let entries = bytes[entries_start..entries_start + declared_length]
        .chunks(entry_width.max(1))
        .take(entry_count)
        .map(<[u8]>::to_vec)
        .collect();
    Ok(IndexNodeHeader {
        tree,
        level,
        key_width,
        smallest_key,
        largest_key,
        birth_txg,
        filesystem_identifier,
        instance,
        birth_sequence,
        entry_width,
        entries,
    })
}

/// 码 3 打包记录单元解出来的头与记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedUnitHeader {
    pub identity: PackedIdentity,
    pub birth_txg: CheckpointTxg,
    pub filesystem_identifier: u64,
    pub write_order: WriteOrder,
    pub birth_sequence: BirthSequence,
    pub record_width: usize,
    pub records: Vec<Vec<u8>>,
}

pub fn parse_packed_unit(bytes: &[u8]) -> Result<PackedUnitHeader, UnitError> {
    if bytes.len() != usize::try_from(DATA_UNIT_BYTES).expect("32768") {
        return Err(UnitError::TooShort);
    }
    let header_end = usize::try_from(PACKED_UNIT_HEADER_BYTES).expect("107");
    let declared_length = usize::from(check_common_prefix(bytes, UNIT_CLASS_PACKED, header_end)?);
    check_payload_checksum(bytes, header_end, 89)?;
    let mut reader = ByteReader::at(bytes, 42);
    if reader.get_u8() != UNIT_CLASS_PACKED {
        return Err(UnitError::Structure("类标签副本不是 3"));
    }
    let identity = PackedIdentity {
        birth_tree: TreeIdentifier(reader.get_u64()),
        record_type: reader.get_u16(),
        container: reader.get_u64(),
        container_birth: CheckpointTxg(reader.get_u64()),
    };
    let record_count = usize::from(reader.get_u16());
    let record_width = usize::from(reader.get_u16());
    let birth_txg = CheckpointTxg(reader.get_u64());
    let filesystem_identifier = reader.get_u64();
    reader.skip(4);
    let write_order = WriteOrder {
        instance: InstanceGeneration(reader.get_u32()),
        transaction: reader.get_six_byte_unsigned(),
    };
    let birth_sequence = BirthSequence(reader.get_u32());
    assert_eq!(reader.position(), header_end);
    if record_count * record_width != declared_length {
        return Err(UnitError::Structure("声明长度 ≠ 记录数 × 记录宽"));
    }
    let records_start = header_end + reserved_bytes();
    if records_start + declared_length > bytes.len() {
        return Err(UnitError::Structure("记录区越过单元末尾"));
    }
    let records = bytes[records_start..records_start + declared_length]
        .chunks(record_width.max(1))
        .take(record_count)
        .map(<[u8]>::to_vec)
        .collect();
    Ok(PackedUnitHeader {
        identity,
        birth_txg,
        filesystem_identifier,
        write_order,
        birth_sequence,
        record_width,
        records,
    })
}

/// 码 1 数据单元解出来的头。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataUnitHeader {
    pub identity: DataUnitIdentity,
    pub birth_txg: CheckpointTxg,
    pub filesystem_identifier: u64,
    pub write_order: WriteOrder,
    pub declared_length: u16,
}

pub fn parse_data_unit(bytes: &[u8]) -> Result<DataUnitHeader, UnitError> {
    if bytes.len() != usize::try_from(DATA_UNIT_BYTES).expect("32768") {
        return Err(UnitError::TooShort);
    }
    let header_end = usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105");
    let declared_length = check_common_prefix(bytes, UNIT_CLASS_DATA, header_end)?;
    check_payload_checksum(bytes, header_end, 101)?;
    let mut reader = ByteReader::at(bytes, 42);
    if reader.get_u8() != UNIT_CLASS_DATA {
        return Err(UnitError::Structure("类标签副本不是 1"));
    }
    let identity = DataUnitIdentity {
        tree: TreeIdentifier(reader.get_u64()),
        object: reader.get_u64(),
        object_birth: CheckpointTxg(reader.get_u64()),
        anchor_offset: reader.get_u64(),
    };
    let birth_txg = CheckpointTxg(reader.get_u64());
    let filesystem_identifier = reader.get_u64();
    let write_order = WriteOrder {
        instance: InstanceGeneration(reader.get_u32()),
        transaction: reader.get_six_byte_unsigned(),
    };
    let payload_start = header_end + reserved_bytes();
    if payload_start + usize::from(declared_length) > bytes.len() {
        return Err(UnitError::Structure("声明长度越过单元末尾"));
    }
    Ok(DataUnitHeader {
        identity,
        birth_txg,
        filesystem_identifier,
        write_order,
        declared_length,
    })
}

/// 载荷从 134 起、长声明长度；之后到单元末尾必须全 0（I-2.3），不是就报结构错。
pub fn data_unit_payload(bytes: &[u8], declared_length: u16) -> Result<&[u8], UnitError> {
    let payload_start = usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105") + reserved_bytes();
    let payload_end = payload_start + usize::from(declared_length);
    if bytes[payload_end..].iter().any(|byte| *byte != 0) {
        return Err(UnitError::Structure("补齐字节非零"));
    }
    Ok(&bytes[payload_start..payload_end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checksum::wide_checksum_field_holds;

    #[test]
    fn empty_tree_table_node_has_key_width_at_51_and_a_sealed_header() {
        let filesystem_identifier = [9u8; 16];
        let node = build_index_node(
            TreeIdentifier(0),
            0,
            8,
            &[0u8; 8],
            &[0u8; 8],
            CheckpointTxg(0),
            &filesystem_identifier,
            InstanceGeneration(0),
            BirthSequence(1),
            148,
            &[],
        );
        assert_eq!(node.len(), 16384);
        assert_eq!(&node[..4], b"SFSU");
        assert_eq!(node[6], UNIT_CLASS_INDEX_NODE);
        assert_eq!(node[51], 8, "key 宽住偏移 51");
        assert_eq!(
            u16::from_le_bytes([node[8], node[9]]),
            0,
            "0 条条目 ⇒ 声明长度 0"
        );
        assert_eq!(
            u16::from_le_bytes([node[84 + 16], node[85 + 16]]),
            148,
            "条目宽在 84 + 2k"
        );
        assert!(wide_checksum_field_holds(
            &node,
            86 + 16,
            HEADER_CHECKSUM_OFFSET
        ));
        let payload_crc = u32::from_le_bytes(node[76 + 16..80 + 16].try_into().expect("4"));
        assert_eq!(
            payload_crc,
            crc32_castagnoli(&node[86 + 16..]),
            "载荷 CRC 从明文头末尾起、含预留位"
        );
    }

    #[test]
    fn packed_unit_places_records_at_136_and_seals_both_checksums() {
        let identity = PackedIdentity {
            birth_tree: TreeIdentifier(0),
            record_type: PACKED_TYPE_INSTANCE_TABLE,
            container: 0,
            container_birth: CheckpointTxg(0),
        };
        let record = vec![0x5Au8; 88];
        let unit = build_packed_unit(
            identity,
            88,
            std::slice::from_ref(&record),
            CheckpointTxg(0),
            &[1u8; 16],
            WriteOrder {
                instance: InstanceGeneration(0),
                transaction: 0,
            },
            BirthSequence(0),
        );
        assert_eq!(unit.len(), 32768);
        assert_eq!(&unit[136..136 + 88], &record[..]);
        assert_eq!(u16::from_le_bytes([unit[69], unit[70]]), 1, "记录数");
        assert_eq!(u16::from_le_bytes([unit[71], unit[72]]), 88, "记录宽");
        assert!(wide_checksum_field_holds(
            &unit,
            107,
            HEADER_CHECKSUM_OFFSET
        ));
        assert_eq!(
            u32::from_le_bytes(unit[89..93].try_into().expect("4")),
            crc32_castagnoli(&unit[107..])
        );
    }

    #[test]
    fn the_three_parsers_round_trip_their_builders_and_refuse_a_damaged_header() {
        let filesystem_identifier = [9u8; 16];
        let node = build_index_node(
            TreeIdentifier(11),
            0,
            24,
            &[1u8; 24],
            &[2u8; 24],
            CheckpointTxg(3),
            &filesystem_identifier,
            InstanceGeneration(1),
            BirthSequence(0),
            112,
            &[vec![1u8; 112], vec![2u8; 112]],
        );
        let header = parse_index_node(&node).expect("码 2");
        assert_eq!(
            (header.tree, header.key_width, header.entry_width),
            (TreeIdentifier(11), 24, 112)
        );
        assert_eq!(header.entries.len(), 2);
        assert_eq!(header.smallest_key, vec![1u8; 24]);
        let mut damaged = node.clone();
        damaged[60] ^= 1;
        assert_eq!(
            parse_index_node(&damaged),
            Err(UnitError::HeaderChecksumMismatch)
        );
        let mut wrong_class = node;
        wrong_class[6] = UNIT_CLASS_DATA;
        assert_eq!(
            parse_index_node(&wrong_class),
            Err(UnitError::WrongUnitClass {
                expected: 2,
                found: 1
            })
        );

        let identity = PackedIdentity {
            birth_tree: TreeIdentifier(12),
            record_type: PACKED_TYPE_INODE,
            container: 1,
            container_birth: CheckpointTxg(3),
        };
        let write_order = WriteOrder {
            instance: InstanceGeneration(1),
            transaction: 1,
        };
        let packed = build_packed_unit(
            identity,
            140,
            &[vec![7u8; 140]],
            CheckpointTxg(3),
            &filesystem_identifier,
            write_order,
            BirthSequence(0),
        );
        let packed_header = parse_packed_unit(&packed).expect("码 3");
        assert_eq!(
            (
                packed_header.identity,
                packed_header.record_width,
                packed_header.records.len()
            ),
            (identity, 140, 1)
        );
        assert_eq!(packed_header.write_order, write_order);

        let data = build_data_unit(
            DataUnitIdentity {
                tree: TreeIdentifier(11),
                object: 1,
                object_birth: CheckpointTxg(3),
                anchor_offset: 0,
            },
            CheckpointTxg(3),
            &filesystem_identifier,
            write_order,
            &[5u8; 3000],
        );
        let data_header = parse_data_unit(&data).expect("码 1");
        assert_eq!(data_header.declared_length, 3000);
        assert_eq!(
            data_unit_payload(&data, 3000).expect("载荷"),
            &[5u8; 3000][..]
        );
        let mut padded = data.clone();
        padded[32000] = 1;
        assert_eq!(
            parse_data_unit(&padded),
            Err(UnitError::PayloadChecksumMismatch),
            "补齐区在载荷 CRC 内"
        );
        let mut torn = data;
        torn[20000] ^= 1;
        assert_eq!(
            parse_data_unit(&torn),
            Err(UnitError::PayloadChecksumMismatch)
        );
    }
}
