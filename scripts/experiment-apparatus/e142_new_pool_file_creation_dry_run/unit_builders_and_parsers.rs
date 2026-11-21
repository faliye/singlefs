//! 打包单元、索引节点的拼装与解析、单元错误、公共前缀核对、数据单元与打包单元的解析。

use e7_index_bench::usize_from_u64;
use super::*;

/// 码 3 打包记录单元的类身份四元组（D18 已定项 11）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct PackedIdentity {
    pub(super) birth_tree: TreeIdentifier,
    pub(super) record_type: u16,
    pub(super) container: u64,
    pub(super) container_birth: CheckpointTxg,
}

/// 码 3 打包记录单元 32768：头 107（偏移照 D18 已定项 11 的表）+ 预留 29 + 记录区从 136 起。
pub(super) fn build_packed_unit(
    identity: PackedIdentity,
    record_width: u16,
    records: &[Vec<u8>],
    birth_txg: CheckpointTxg,
    fsid: &[u8; 16],
    write_order: WriteOrder,
    birth_sequence: BirthSequence,
) -> Vec<u8> {
    let record_area_start = usize_from_u64(PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES);
    let declared_length = u16::try_from(records.len() * record_width as usize).expect("声明长度 2 字节");
    let mut writer = ByteWriter::new(usize_from_u64(DATA_UNIT_BYTES));
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
    writer.put_u64(unit_fsid(fsid));
    writer.assert_position(89, "码 3 载荷校验和");
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.assert_position(93, "码 3 写序");
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction.0);
    writer.assert_position(103, "码 3 出生序号");
    writer.put_u32(birth_sequence.0);
    writer.assert_position(PACKED_UNIT_HEADER_BYTES, "码 3 头");
    writer.skip(usize_from_u64(NONCE_MAC_RESERVED_BYTES));
    for record in records {
        assert_eq!(record.len(), record_width as usize, "记录宽与头里写的不一致");
        writer.put(record);
    }
    let _ = record_area_start;
    let mut bytes = writer.bytes;
    // 载荷校验和覆盖偏移 107 到单元末尾（D18 已定项 11 逐字：nonce / MAC 预留位 + 记录区 + 补齐）。
    let payload_crc = castagnoli_crc32(&bytes[usize_from_u64(PACKED_UNIT_HEADER_BYTES)..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    seal_header_checksum(&mut bytes, usize_from_u64(PACKED_UNIT_HEADER_BYTES));
    bytes
}

/// 码 2 索引节点头的宽度：86 + 2 × key 宽（D8（核心索引结构） 已定项 11）；含 29 字节预留位则是 115 + 2 × key 宽。
pub(super) fn index_node_header_bytes(key_width: usize) -> usize {
    usize_from_u64(INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE) + 2 * key_width
}

/// 码 2 索引节点 16384，偏移照 D18（块里携带什么信息） 已定项 18 的表（key 宽在 51）：
/// 42 树 ID 8 / 50 层级 1 / **51 key 宽 1** / 52 key 区间 2k / 52+2k 诞生代号 8 / 60+2k fsid 8
/// / 68+2k 写序 4（只实例代号）/ 72+2k 出生序号 4 / 76+2k 载荷 CRC 4 / 80+2k 预留 2 / 82+2k 条目数 2
/// / 84+2k 条目宽 2 / 86+2k 预留位 29 / 115+2k 条目区。头校验和罩 0..86+2k，载荷 CRC 从 86+2k 到单元末尾。
/// 条目一律定宽（中央映射的条目一宽 55），所以条目宽是个单值、声明长度 = 条目数 × 条目宽。
#[allow(clippy::too_many_arguments, reason = "字段表就是这么多段，收成结构体只会多一层没人验的名字")]
pub(super) fn build_index_node(
    tree: TreeIdentifier,
    level: u8,
    key_width: usize,
    smallest_key: &[u8],
    largest_key: &[u8],
    birth_txg: CheckpointTxg,
    fsid: &[u8; 16],
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
    entry_width: u16,
    entries: &[Vec<u8>],
) -> Vec<u8> {
    assert_eq!(smallest_key.len(), key_width);
    assert_eq!(largest_key.len(), key_width);
    for entry in entries {
        assert_eq!(entry.len(), entry_width as usize, "条目宽与节点里写的不一致");
        assert!(entry.len() >= key_width, "条目里 key 一律是前 key 宽 个字节（D8 已定项 11，2026-09-14 用户定案）");
    }
    let header_end = index_node_header_bytes(key_width);
    let entries_start = header_end + usize_from_u64(NONCE_MAC_RESERVED_BYTES);
    // D8（核心索引结构） 已定项 11：声明长度 = 条目数 × 条目宽。
    let declared_length: usize = entries.len() * entry_width as usize;
    assert!(entries_start + declared_length <= usize_from_u64(NODE_BYTES), "条目装不进一个节点");
    let mut writer = ByteWriter::new(usize_from_u64(NODE_BYTES));
    write_common_prefix(&mut writer, UNIT_CLASS_INDEX_NODE, u16::try_from(declared_length).expect("声明长度 2 字节"));
    writer.put_u64(tree.0);
    writer.put_u8(level);
    writer.assert_position(INDEX_NODE_KEY_WIDTH_OFFSET as u64, "码 2 的 key 宽字段");
    writer.put_u8(u8::try_from(key_width).expect("key 宽 1 字节"));
    writer.put(smallest_key);
    writer.put(largest_key);
    writer.put_u64(birth_txg.0);
    writer.put_u64(unit_fsid(fsid));
    writer.put_u32(instance.0);
    writer.put_u32(birth_sequence.0);
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.skip(2);
    writer.put_u16(u16::try_from(entries.len()).expect("条目数 2 字节"));
    writer.put_u16(entry_width);
    writer.assert_position(header_end as u64, "码 2 头");
    writer.skip(usize_from_u64(NONCE_MAC_RESERVED_BYTES));
    for entry in entries {
        writer.put(entry);
    }
    let mut bytes = writer.bytes;
    // 载荷 CRC 覆盖从头末尾（含 28 字节预留位）到单元末尾（D18 已定项 7 索引节点类：口径照码 1 与码 3）。
    let payload_crc = castagnoli_crc32(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    seal_header_checksum(&mut bytes, header_end);
    bytes
}

/// 解出来的码 2 节点头。key 宽住**偏移 51**（D18（块里携带什么信息） 已定项 18），
/// 定位它不再需要先知道 k ⇒ 扫描期按 `[0, 51)` 这段定长前缀读出 k、一次定出头末端（gap G2 收口）。
#[derive(Clone, Debug)]
pub(super) struct IndexNodeHeader {
    pub(super) tree: TreeIdentifier,
    pub(super) level: u8,
    pub(super) key_width: usize,
    pub(super) smallest_key: Vec<u8>,
    pub(super) largest_key: Vec<u8>,
    pub(super) birth_txg: CheckpointTxg,
    pub(super) fsid: u64,
    pub(super) instance: InstanceGeneration,
    pub(super) birth_sequence: BirthSequence,
    pub(super) entries: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum UnitError {
    BadMagic,
    BadVersion,
    NonZeroFlags,
    WrongClass { expected: u8, found: u8 },
    HeaderChecksum,
    PayloadChecksum,
    Structure(&'static str),
}

fn check_common_prefix(bytes: &[u8], expected_class: u8) -> Result<u16, UnitError> {
    if bytes[..4] != UNIT_MAGIC {
        return Err(UnitError::BadMagic);
    }
    let mut reader = ByteReader::at(bytes, 4);
    if reader.get_u16() != FORMAT_VERSION {
        return Err(UnitError::BadVersion);
    }
    let found_class = reader.get_u8();
    if reader.get_u8() != 0 {
        return Err(UnitError::NonZeroFlags);
    }
    if found_class != expected_class {
        return Err(UnitError::WrongClass { expected: expected_class, found: found_class });
    }
    Ok(reader.get_u16())
}

/// 解码 2 节点：key 宽住**偏移 51**（D18 已定项 18），它自己的偏移不依赖 k
/// ⇒ 先读 `[0, 51)` 这段定长前缀里的 k、一次定出头末端，再核头校验和。调用方不必先知道这是哪棵树。
pub(super) fn parse_index_node(bytes: &[u8]) -> Result<IndexNodeHeader, UnitError> {
    let declared_length = check_common_prefix(bytes, UNIT_CLASS_INDEX_NODE)?;
    let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
    let header_end = index_node_header_bytes(key_width);
    if header_end + usize_from_u64(NONCE_MAC_RESERVED_BYTES) > bytes.len() {
        return Err(UnitError::Structure("自述的 key 宽让头越出单元"));
    }
    if !header_checksum_holds(bytes, header_end) {
        return Err(UnitError::HeaderChecksum);
    }
    let mut reader = ByteReader::at(bytes, usize_from_u64(COMMON_PREFIX_BYTES));
    let tree = TreeIdentifier(reader.get_u64());
    let level = reader.get_u8();
    let declared_key_width = usize::from(reader.get_u8());
    let smallest_key = reader.take(key_width).to_vec();
    let largest_key = reader.take(key_width).to_vec();
    let birth_txg = CheckpointTxg(reader.get_u64());
    let fsid = reader.get_u64();
    let instance = InstanceGeneration(reader.get_u32());
    let birth_sequence = BirthSequence(reader.get_u32());
    let payload_crc = reader.get_u32();
    reader.skip(2);
    let entry_count = reader.get_u16() as usize;
    let entry_width = reader.get_u16() as usize;
    if castagnoli_crc32(&bytes[header_end..]) != payload_crc {
        return Err(UnitError::PayloadChecksum);
    }
    // D8（核心索引结构） 已定项 11：条目定宽连续排列，声明长度 = 条目数 × 条目宽。
    if entry_width < key_width || entry_count * entry_width != declared_length as usize {
        return Err(UnitError::Structure("声明长度与条目数 × 条目宽对不上"));
    }
    let mut payload = ByteReader::at(bytes, header_end + usize_from_u64(NONCE_MAC_RESERVED_BYTES));
    let mut entries: Vec<Vec<u8>> = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        if payload.cursor + entry_width > bytes.len() {
            return Err(UnitError::Structure("条目区越界"));
        }
        entries.push(payload.take(entry_width).to_vec());
    }
    // D18（块里携带什么信息） 已定项 18：条目区之后的补齐区恒 0 且参与载荷 CRC。
    if bytes[payload.cursor..].iter().any(|&byte| byte != 0) {
        return Err(UnitError::Structure("条目区之后的补齐区非零")); // I-2.3 射程含码 2
    }
    Ok(IndexNodeHeader { tree, level, key_width: declared_key_width, smallest_key, largest_key, birth_txg, fsid, instance, birth_sequence, entries })
}

#[derive(Clone, Debug)]
pub(super) struct PackedUnitHeader {
    pub(super) identity: PackedIdentity,
    pub(super) record_width: u16,
    pub(super) birth_txg: CheckpointTxg,
    pub(super) fsid: u64,
    pub(super) write_order: WriteOrder,
    pub(super) birth_sequence: BirthSequence,
    pub(super) records: Vec<Vec<u8>>,
}

pub(super) fn parse_packed_unit(bytes: &[u8]) -> Result<PackedUnitHeader, UnitError> {
    let declared_length = check_common_prefix(bytes, UNIT_CLASS_PACKED)?;
    if !header_checksum_holds(bytes, usize_from_u64(PACKED_UNIT_HEADER_BYTES)) {
        return Err(UnitError::HeaderChecksum);
    }
    let mut reader = ByteReader::at(bytes, usize_from_u64(COMMON_PREFIX_BYTES));
    if reader.get_u8() != UNIT_CLASS_PACKED {
        return Err(UnitError::Structure("类身份段首字节不等于偏移 6"));
    }
    let identity = PackedIdentity {
        birth_tree: TreeIdentifier(reader.get_u64()),
        record_type: reader.get_u16(),
        container: reader.get_u64(),
        container_birth: CheckpointTxg(reader.get_u64()),
    };
    let record_count = reader.get_u16() as usize;
    let record_width = reader.get_u16();
    let birth_txg = CheckpointTxg(reader.get_u64());
    let fsid = reader.get_u64();
    let payload_crc = reader.get_u32();
    let write_order = WriteOrder { instance: InstanceGeneration(reader.get_u32()), transaction: TransactionNumber(reader.get_six_byte_unsigned()) };
    let birth_sequence = BirthSequence(reader.get_u32());
    // I-1.7 的判定顺序：边界先于载荷校验和。
    if record_count * record_width as usize > declared_length as usize || declared_length as usize > usize_from_u64(DATA_UNIT_BYTES - PACKED_UNIT_HEADER_BYTES) {
        return Err(UnitError::Structure("记录数 × 记录宽 > 声明长度"));
    }
    if castagnoli_crc32(&bytes[usize_from_u64(PACKED_UNIT_HEADER_BYTES)..]) != payload_crc {
        return Err(UnitError::PayloadChecksum);
    }
    let mut record_reader = ByteReader::at(bytes, usize_from_u64(PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES));
    let records = (0..record_count).map(|_| record_reader.take(record_width as usize).to_vec()).collect();
    Ok(PackedUnitHeader { identity, record_width, birth_txg, fsid, write_order, birth_sequence, records })
}

#[derive(Clone, Debug)]
pub(super) struct DataUnitHeader {
    pub(super) identity: DataUnitIdentity,
    pub(super) declared_length: u16,
    pub(super) birth_txg: CheckpointTxg,
    pub(super) fsid: u64,
    pub(super) write_order: WriteOrder,
}

pub(super) fn parse_data_unit(bytes: &[u8]) -> Result<DataUnitHeader, UnitError> {
    let declared_length = check_common_prefix(bytes, UNIT_CLASS_DATA)?;
    if !header_checksum_holds(bytes, usize_from_u64(DATA_UNIT_HEADER_BYTES)) {
        return Err(UnitError::HeaderChecksum);
    }
    let mut reader = ByteReader::at(bytes, usize_from_u64(COMMON_PREFIX_BYTES));
    if reader.get_u8() != UNIT_CLASS_DATA {
        return Err(UnitError::Structure("五元组首字节不等于偏移 6"));
    }
    let identity = DataUnitIdentity {
        tree: TreeIdentifier(reader.get_u64()),
        object: reader.get_u64(),
        object_birth: CheckpointTxg(reader.get_u64()),
        anchor_offset: reader.get_u64(),
    };
    let birth_txg = CheckpointTxg(reader.get_u64());
    let fsid = reader.get_u64();
    let write_order = WriteOrder { instance: InstanceGeneration(reader.get_u32()), transaction: TransactionNumber(reader.get_six_byte_unsigned()) };
    let payload_crc = reader.get_u32();
    if castagnoli_crc32(&bytes[usize_from_u64(DATA_UNIT_HEADER_BYTES)..]) != payload_crc {
        return Err(UnitError::PayloadChecksum);
    }
    Ok(DataUnitHeader { identity, declared_length, birth_txg, fsid, write_order })
}

pub(super) fn data_unit_payload(bytes: &[u8], declared_length: u16) -> &[u8] {
    let start = usize_from_u64(DATA_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES);
    &bytes[start..start + declared_length as usize]
}

#[cfg(test)]
#[path = "unit_builders_and_parsers_tests.rs"]
mod tests;
