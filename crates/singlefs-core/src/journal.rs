//! journal 记录 4096（D23（journal 的角色与格式） 已定项 4 / 已定项 7 / 已定项 8 / 已定项 13 / 已定项 15 / 已定项 17 / 已定项 18 / 已定项 19）：
//! 头 307 + 点名项 56 × N；`header_csum` 罩整条 [0, 4096) 含补齐、自身按 0 参与；`payload_csum` 罩点名项数组；
//! 反向链 = CRC-32C(本实例内逻辑前一条记录的 307 字节头，`header_csum` 按 0 参与)，本实例第一条恒 0。

use singlefs_format::{
    JOURNAL_HEADER_BYTES, JOURNAL_NAMED_ENTRY_BYTES, JOURNAL_NEW_ROOT_SEGMENT_BYTES,
    JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, MAPPING_KEY_BYTES, SLOT_BYTES,
    WIDE_CHECKSUM_BYTES,
};

use crate::address::{CheckpointTxg, DeviceOffsetInBytes, InstanceGeneration, TreeIdentifier};
use crate::bytes::{ByteReader, ByteWriter};
use crate::checksum::{
    crc32_castagnoli, wide_checksum_field_holds, wide_checksum_with_field_zeroed,
};
use crate::pointer::{LocationEntry, NodePointer};

pub const JOURNAL_MAGIC: [u8; 4] = *b"SFSJ";
/// 记录类型登记表（D23（journal 的角色与格式） 已定项 1）：0 无效、1 普通记录。
pub const JOURNAL_RECORD_TYPE_ORDINARY: u16 = 1;
/// magic 4 + 类型 2 + 算法类型 1 + 填充 1 + 记录长度 4 + 点名项数 4 + jsn 10 + checkpoint_txg 8 + nonce 12。
pub const JOURNAL_HEADER_CHECKSUM_OFFSET: usize = 4 + 2 + 1 + 1 + 4 + 4 + 10 + 8 + 12;

/// 点名项 56（D23（journal 的角色与格式） 已定项 17）：位置条目 14 × 2 + 类标签 1 + 出生树 8 + 出生 txg 8 + key 尾段 10 + flags 1。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NamedUnit {
    pub locations: [LocationEntry; 2],
    pub unit_class: u8,
    pub birth_tree: TreeIdentifier,
    pub birth_txg: CheckpointTxg,
    pub key_tail: [u8; 10],
}

impl NamedUnit {
    pub fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        for location in &self.locations {
            location.write_to(writer);
        }
        writer.put_u8(self.unit_class);
        writer.put_u64(self.birth_tree.0);
        writer.put_u64(self.birth_txg.0);
        writer.put(&self.key_tail);
        writer.put_u8(0);
        assert_eq!(
            writer.position() - start,
            usize::try_from(JOURNAL_NAMED_ENTRY_BYTES).expect("56")
        );
    }
    pub fn read_from(reader: &mut ByteReader<'_>) -> Self {
        let locations = [
            LocationEntry::read_from(reader),
            LocationEntry::read_from(reader),
        ];
        let unit_class = reader.get_u8();
        let birth_tree = TreeIdentifier(reader.get_u64());
        let birth_txg = CheckpointTxg(reader.get_u64());
        let key_tail: [u8; 10] = reader.take(10).try_into().expect("切了 10 字节");
        reader.skip(1);
        Self {
            locations,
            unit_class,
            birth_tree,
            birth_txg,
            key_tail,
        }
    }
    /// 重放不查单元就能凑出这一项的中央映射 key（D23（journal 的角色与格式） 已定项 17 末句）。
    #[must_use]
    pub fn mapping_key(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize::try_from(MAPPING_KEY_BYTES).expect("27"));
        writer.put_u8(self.unit_class);
        writer.put_u64(self.birth_tree.0);
        writer.put_u64(self.birth_txg.0);
        writer.put(&self.key_tail);
        writer.into_bytes()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalRecord {
    pub instance: InstanceGeneration,
    pub counter: u64,
    pub checkpoint_txg: CheckpointTxg,
    pub transaction: u64,
    pub is_commit: bool,
    pub back_chain: u32,
    pub filesystem_identifier: u64,
    pub new_tree_table: NodePointer,
    pub new_mapping_root: NodePointer,
    pub new_tree_identifier_watermark: u64,
    pub new_rollback_floor: CheckpointTxg,
    pub named: Vec<NamedUnit>,
}

/// 反向链：前一条记录的 307 字节头，`header_csum` 那 32 字节按 0 参与。
#[must_use]
pub fn back_chain_of(previous_record_bytes: &[u8]) -> u32 {
    let mut header =
        previous_record_bytes[..usize::try_from(JOURNAL_HEADER_BYTES).expect("307")].to_vec();
    header[JOURNAL_HEADER_CHECKSUM_OFFSET
        ..JOURNAL_HEADER_CHECKSUM_OFFSET + usize::try_from(WIDE_CHECKSUM_BYTES).expect("32")]
        .fill(0);
    crc32_castagnoli(&header)
}

/// 记录 n 落在环内偏移 `(计数器 − 1) mod 槽数 × 4096`（D23（journal 的角色与格式） 已定项 18）。
#[must_use]
pub fn record_offset(counter: u64, ring_bytes: u64) -> DeviceOffsetInBytes {
    let ring_slots = ring_bytes / JOURNAL_RECORD_BYTES;
    DeviceOffsetInBytes(
        JOURNAL_RING_START_SLOT * SLOT_BYTES + ((counter - 1) % ring_slots) * JOURNAL_RECORD_BYTES,
    )
}

impl JournalRecord {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
        let mut writer = ByteWriter::new(record_bytes);
        writer.put(&JOURNAL_MAGIC);
        writer.put_u16(JOURNAL_RECORD_TYPE_ORDINARY);
        writer.put_u8(0); // 算法类型
        writer.put_u8(0); // 对齐填充
        writer.put_u32(u32::try_from(JOURNAL_RECORD_BYTES).expect("记录长度"));
        writer.put_u32(u32::try_from(self.named.len()).expect("点名项数 4 字节"));
        writer.put_u32(self.instance.0);
        writer.put_six_byte_unsigned(self.counter);
        writer.put_u64(self.checkpoint_txg.0);
        writer.skip(12); // nonce
        writer.assert_position(
            u64::try_from(JOURNAL_HEADER_CHECKSUM_OFFSET).expect("偏移"),
            "记录头校验和",
        );
        writer.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
        writer.put_u64(self.transaction);
        writer.put_u8(u8::from(self.is_commit));
        writer.put_u32(self.back_chain);
        let payload_checksum_offset = writer.position();
        writer.skip(4);
        let new_root_segment_start = writer.position();
        self.new_tree_table.write_to(&mut writer);
        self.new_mapping_root.write_to(&mut writer);
        writer.put_u64(self.new_tree_identifier_watermark);
        writer.put_u64(self.new_rollback_floor.0);
        assert_eq!(
            writer.position() - new_root_segment_start,
            usize::try_from(JOURNAL_NEW_ROOT_SEGMENT_BYTES).expect("188")
        );
        writer.put_u64(self.filesystem_identifier);
        writer.skip(16); // MAC：第一版留位全 0
        writer.assert_position(JOURNAL_HEADER_BYTES, "记录头");
        for named in &self.named {
            named.write_to(&mut writer);
        }
        let payload_end = writer.position();
        let mut bytes = writer.into_bytes();
        let payload_checksum = crc32_castagnoli(
            &bytes[usize::try_from(JOURNAL_HEADER_BYTES).expect("307")..payload_end],
        );
        bytes[payload_checksum_offset..payload_checksum_offset + 4]
            .copy_from_slice(&payload_checksum.to_le_bytes());
        let digest =
            wide_checksum_with_field_zeroed(&bytes, record_bytes, JOURNAL_HEADER_CHECKSUM_OFFSET);
        bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        bytes
    }

    /// 读者：magic、类型、整条校验和、fsid、载荷校验和四关；不查反向链（那是前缀取法的事）。
    #[must_use]
    pub fn parse(bytes: &[u8], expected_filesystem_identifier: u64) -> Option<Self> {
        let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
        if bytes.len() < record_bytes || bytes[..4] != JOURNAL_MAGIC {
            return None;
        }
        if !wide_checksum_field_holds(bytes, record_bytes, JOURNAL_HEADER_CHECKSUM_OFFSET) {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 4);
        if reader.get_u16() != JOURNAL_RECORD_TYPE_ORDINARY {
            return None;
        }
        reader.skip(1 + 1);
        if u64::from(reader.get_u32()) != JOURNAL_RECORD_BYTES {
            return None;
        }
        let named_count = usize::try_from(reader.get_u32()).expect("点名项数");
        let instance = InstanceGeneration(reader.get_u32());
        let counter = reader.get_six_byte_unsigned();
        let checkpoint_txg = CheckpointTxg(reader.get_u64());
        reader.skip(12 + 32);
        let transaction = reader.get_u64();
        let is_commit = reader.get_u8() == 1;
        let back_chain = reader.get_u32();
        let payload_checksum = reader.get_u32();
        let new_tree_table = NodePointer::read_from(&mut reader);
        let new_mapping_root = NodePointer::read_from(&mut reader);
        let new_tree_identifier_watermark = reader.get_u64();
        let new_rollback_floor = CheckpointTxg(reader.get_u64());
        let filesystem_identifier = reader.get_u64();
        if filesystem_identifier != expected_filesystem_identifier {
            return None;
        }
        reader.skip(16);
        assert_eq!(
            reader.position(),
            usize::try_from(JOURNAL_HEADER_BYTES).expect("307")
        );
        let payload_end = reader.position()
            + named_count * usize::try_from(JOURNAL_NAMED_ENTRY_BYTES).expect("56");
        if payload_end > record_bytes
            || crc32_castagnoli(&bytes[reader.position()..payload_end]) != payload_checksum
        {
            return None;
        }
        let named = (0..named_count)
            .map(|_index| NamedUnit::read_from(&mut reader))
            .collect();
        Some(Self {
            instance,
            counter,
            checkpoint_txg,
            transaction,
            is_commit,
            back_chain,
            filesystem_identifier,
            new_tree_table,
            new_mapping_root,
            new_tree_identifier_watermark,
            new_rollback_floor,
            named,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;

    fn empty_record(counter: u64) -> JournalRecord {
        JournalRecord {
            instance: InstanceGeneration(1),
            counter,
            checkpoint_txg: CheckpointTxg(counter),
            transaction: 0,
            is_commit: true,
            back_chain: 0,
            filesystem_identifier: 7,
            new_tree_table: NodePointer::empty_root(),
            new_mapping_root: NodePointer::empty_root(),
            new_tree_identifier_watermark: 11,
            new_rollback_floor: CheckpointTxg(0),
            named: Vec::new(),
        }
    }

    #[test]
    fn record_is_4096_with_a_307_byte_header_and_round_trips() {
        let record = empty_record(1);
        let bytes = record.to_bytes();
        assert_eq!(bytes.len(), 4096);
        assert!(
            bytes[307..].iter().all(|byte| *byte == 0),
            "空记录 307 之后全 0"
        );
        assert_eq!(JournalRecord::parse(&bytes, 7), Some(record.clone()));
        assert_eq!(
            JournalRecord::parse(&bytes, 8),
            None,
            "fsid 不符的记录不进前缀"
        );
        let mut torn = bytes.clone();
        torn[4000] ^= 1;
        assert_eq!(
            JournalRecord::parse(&torn, 7),
            None,
            "header_csum 罩整条 4096"
        );
        let second = JournalRecord {
            back_chain: back_chain_of(&bytes),
            ..empty_record(2)
        };
        assert_ne!(second.back_chain, 0);
        assert_eq!(
            record_offset(1, JOURNAL_RING_DEFAULT_BYTES),
            DeviceOffsetInBytes(1024 * 16384)
        );
        assert_eq!(
            record_offset(3, JOURNAL_RING_DEFAULT_BYTES),
            DeviceOffsetInBytes(1024 * 16384 + 8192)
        );
        assert_eq!(
            record_offset(196_609, JOURNAL_RING_DEFAULT_BYTES),
            DeviceOffsetInBytes(1024 * 16384),
            "绕回环首"
        );
    }
}
