//! journal 记录 4096（D23（journal 的角色与格式） 已定项 4 / 已定项 7 / 已定项 8 / 已定项 13 / 已定项 15 / 已定项 17 / 已定项 18 / 已定项 19）：
//! 头 311 + 点名项 56 × N；`header_csum` 罩整条 [0, 4096) 含补齐、自身按 0 参与；`payload_csum` 罩点名项数组；
//! 反向链 = CRC-32C(本实例内逻辑前一条记录的 311 字节头，`header_csum` 按 0 参与)，本实例第一条恒 0。

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
/// magic 4 + 类型 2 + 算法类型 1 + 记录标志 1 + 记录长度 4 + 点名项数 4 + jsn 10 + checkpoint_txg 8 + nonce 12。
pub const JOURNAL_HEADER_CHECKSUM_OFFSET: usize = 4 + 2 + 1 + 1 + 4 + 4 + 10 + 8 + 12;
/// 记录标志 1 字节在 magic 4 + 类型 2 + 算法类型 1 之后（D23（journal 的角色与格式） 已定项 4，原「填充 1」那个字节）。
pub const JOURNAL_RECORD_FLAGS_OFFSET: usize = 4 + 2 + 1;
/// 记录标志位 0：本次发布末条（D23（journal 的角色与格式） 已定项 17）。其余位写 0，读到非 0 当损坏（已定项 4）。
pub const JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH: u8 = 0b0000_0001;

/// 这条记录在它那次发布里是不是末条：记录标志位 0（D23（journal 的角色与格式） 已定项 4 / 已定项 17）。
/// 一次发布的末条之外的记录写 0；每次只有一条记录的发布（含空发布记录）那一条也写 1。
/// 恢复按它认一次发布的边界（已定项 14 第六条），所选根覆盖的最后一条也按它认（已定项 14 注 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JournalRecordPlaceInPublish {
    /// 位 0 = 1：本次发布末条。
    LastRecordOfThePublish,
    /// 位 0 = 0：这次发布在它之后还有记录。
    MoreRecordsOfThePublishFollow,
}

impl JournalRecordPlaceInPublish {
    /// 写进记录标志那 1 字节的值：只用位 0，其余位恒 0。
    #[must_use]
    pub const fn record_flags_byte(self) -> u8 {
        match self {
            JournalRecordPlaceInPublish::LastRecordOfThePublish => {
                JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH
            }
            JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => 0,
        }
    }

    /// 从盘上读来的记录标志那 1 字节：位 0 之外有位为 1 ⇒ `None`（已定项 4「其余位写 0，读到非 0 当损坏」）。
    #[must_use]
    pub const fn from_record_flags_byte(record_flags_byte: u8) -> Option<Self> {
        match record_flags_byte {
            0 => Some(JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow),
            JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH => {
                Some(JournalRecordPlaceInPublish::LastRecordOfThePublish)
            }
            _other_bits_set => None,
        }
    }
}
/// 本次发布内序号紧跟事务号 8 与提交标记 1（D23（journal 的角色与格式） 已定项 4）：头部校验和 32 之后。
pub const JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET: usize =
    JOURNAL_HEADER_CHECKSUM_OFFSET + 32 + 8 + 1;

/// 本次发布内序号（D23（journal 的角色与格式） 已定项 4，无符号 32 位，从 1 起）：一次发布切成 N 条记录时依次是 1..N，
/// 只有一条时是 1，空发布记录也写 1。所选根覆盖的最后一条读不出时，恢复靠它认出下一次发布的第一条（已定项 14 注 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JournalRecordOrdinalWithinPublish(pub u32);

impl JournalRecordOrdinalWithinPublish {
    /// 一次发布的第一条；只有一条记录的发布（空发布、写行、只有一个数据单元的写）那一条就是它。
    pub const FIRST: Self = Self(1);

    /// 一次发布里从 0 数第 `record_offset_in_this_publish` 条记录的序号：偏移 + 1。
    ///
    /// # Panics
    /// 偏移 + 1 装不进 32 位：一次发布的记录条数 = 数据单元数（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案），
    /// 写者在任何落盘动作之前按一片 extent 叶的容量拒掉装不下的写，条数远在 2³² 之下。
    #[must_use]
    pub fn of_record_at_offset(record_offset_in_this_publish: usize) -> Self {
        Self(
            u32::try_from(record_offset_in_this_publish + 1)
                .expect("一次发布的记录条数在落盘之前按 extent 叶容量截过，序号装得进 32 位"),
        )
    }
}

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
    pub ordinal_within_publish: JournalRecordOrdinalWithinPublish,
    /// 记录标志位 0（D23（journal 的角色与格式） 已定项 4 / 已定项 17）。
    pub place_in_publish: JournalRecordPlaceInPublish,
    pub back_chain: u32,
    pub filesystem_identifier: u64,
    pub new_tree_table: NodePointer,
    pub new_mapping_root: NodePointer,
    pub new_tree_identifier_watermark: u64,
    pub new_rollback_floor: CheckpointTxg,
    pub named: Vec<NamedUnit>,
}

/// 反向链：前一条记录的整个记录头（[`JOURNAL_HEADER_BYTES`] = 311 字节），`header_csum` 那 32 字节按 0 参与。
#[must_use]
pub fn back_chain_of(previous_record_bytes: &[u8]) -> u32 {
    let mut header =
        previous_record_bytes[..usize::try_from(JOURNAL_HEADER_BYTES).expect("311")].to_vec();
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
        writer.assert_position(
            u64::try_from(JOURNAL_RECORD_FLAGS_OFFSET).expect("偏移"),
            "记录标志",
        );
        writer.put_u8(self.place_in_publish.record_flags_byte());
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
        writer.assert_position(
            u64::try_from(JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET).expect("偏移"),
            "本次发布内序号",
        );
        writer.put_u32(self.ordinal_within_publish.0);
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
            &bytes[usize::try_from(JOURNAL_HEADER_BYTES).expect("311")..payload_end],
        );
        bytes[payload_checksum_offset..payload_checksum_offset + 4]
            .copy_from_slice(&payload_checksum.to_le_bytes());
        let digest =
            wide_checksum_with_field_zeroed(&bytes, record_bytes, JOURNAL_HEADER_CHECKSUM_OFFSET);
        bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        bytes
    }

    /// 读者：magic、类型、整条校验和、fsid、载荷校验和四关，外加三条当损坏的（D23（journal 的角色与格式） 已定项 4 / 已定项 7）：
    /// 记录标志位 0 之外有位为 1、本次发布内序号为 0、提交标记不是 0 也不是 1。当损坏就是与校验和不过同一个结局——这条记录不算在，前缀在它之前断
    /// （已定项 22 断号即止）。不查反向链，也不查一次发布之内跳不跳号：那两样要看别的记录，是前缀取法的事。
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
        reader.skip(1); // 算法类型
        let place_in_publish =
            JournalRecordPlaceInPublish::from_record_flags_byte(reader.get_u8())?;
        if u64::from(reader.get_u32()) != JOURNAL_RECORD_BYTES {
            return None;
        }
        let named_count = usize::try_from(reader.get_u32()).expect("点名项数");
        let instance = InstanceGeneration(reader.get_u32());
        let counter = reader.get_six_byte_unsigned();
        let checkpoint_txg = CheckpointTxg(reader.get_u64());
        reader.skip(12 + 32);
        let transaction = reader.get_u64();
        // 提交标记只有 0（不带）与 1（带）两个取值（D23（journal 的角色与格式） 已定项 7）：读到 2..=255 当这条记录损坏、断链即止，
        // 与已定项 4 读者规则那几格同一处置（主 agent 2026-09-24 定：不许把它静默读成「不带」——checker 的 I-8.8 判它违例，两边说的要是一件事）。
        let is_commit = match reader.get_u8() {
            0 => false,
            1 => true,
            2..=u8::MAX => return None,
        };
        let ordinal_within_publish = JournalRecordOrdinalWithinPublish(reader.get_u32());
        // 序号从 1 起（已定项 4）：读到 0 当这条记录损坏。
        if ordinal_within_publish.0 == 0 {
            return None;
        }
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
            usize::try_from(JOURNAL_HEADER_BYTES).expect("311")
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
            ordinal_within_publish,
            place_in_publish,
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
            ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,
            place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
            back_chain: 0,
            filesystem_identifier: 7,
            new_tree_table: NodePointer::empty_root(),
            new_mapping_root: NodePointer::empty_root(),
            new_tree_identifier_watermark: 11,
            new_rollback_floor: CheckpointTxg(0),
            named: Vec::new(),
        }
    }

    /// 本次发布内序号 4 字节紧跟事务号 8 与提交标记 1（D23（journal 的角色与格式） 已定项 4）：落在 [87, 91)、小端、读得回；
    /// 它之后的反向链、载荷校验和、新根段、fsid 依次后挪 4 字节，MAC 16 占记录头最后的 [295, 311)。偏移按字段表手算、写成字面量，
    /// 不从写者用的常量推（已定项 26 第 1 条：覆盖范围是显式的盘上常量）。
    #[test]
    fn the_ordinal_within_publish_sits_right_after_the_commit_marker_and_round_trips() {
        let record = JournalRecord {
            transaction: 5,
            ordinal_within_publish: JournalRecordOrdinalWithinPublish(3),
            back_chain: 0x0102_0304,
            ..empty_record(9)
        };
        let bytes = record.to_bytes();
        assert_eq!(&bytes[78..86], &5u64.to_le_bytes(), "事务号在 [78, 86)");
        assert_eq!(bytes[86], 1, "提交标记在 86");
        assert_eq!(
            &bytes[87..91],
            &3u32.to_le_bytes(),
            "本次发布内序号紧跟提交标记，在 [87, 91)"
        );
        assert_eq!(
            &bytes[91..95],
            &0x0102_0304u32.to_le_bytes(),
            "反向链后挪到 [91, 95)"
        );
        assert_eq!(
            &bytes[95..99],
            &crc32_castagnoli(&[]).to_le_bytes(),
            "载荷校验和后挪到 [95, 99)：没有点名项时罩的是空串"
        );
        assert_eq!(
            &bytes[99 + 172..99 + 180],
            &11u64.to_le_bytes(),
            "新根段从 99 起：两条指针 86 × 2 之后是树 ID 水位"
        );
        assert_eq!(
            &bytes[287..295],
            &7u64.to_le_bytes(),
            "fsid 后挪到 [287, 295)"
        );
        assert!(
            bytes[295..311].iter().all(|byte| *byte == 0),
            "MAC 16 占 [295, 311)，第一版全 0"
        );
        assert_eq!(JournalRecord::parse(&bytes, 7), Some(record));
    }

    /// 改过记录头某几个字节之后重封头部校验和（罩整条 4096、自身按 0 参与，D23（journal 的角色与格式） 已定项 13）：
    /// 拦得住那条记录的只剩被改的字段本身。
    fn reseal_header_checksum(bytes: &mut [u8]) {
        let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
        let digest =
            wide_checksum_with_field_zeroed(bytes, record_bytes, JOURNAL_HEADER_CHECKSUM_OFFSET);
        bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
    }

    /// 记录标志 1 字节在偏移 7（magic 4 + 类型 2 + 算法类型 1，原「填充 1」那个字节，D23（journal 的角色与格式） 已定项 4）：
    /// 本次发布末条写 1、不是末条写 0，两种都读得回。偏移按字段表手算、写成字面量，不从写者用的常量推（已定项 26 第 1 条）。
    #[test]
    fn the_record_flags_byte_at_offset_seven_carries_bit_zero_for_the_last_record_of_the_publish_and_round_trips(
    ) {
        let last = empty_record(3);
        let last_bytes = last.to_bytes();
        assert_eq!(last_bytes[7], 1, "本次发布末条：记录标志位 0 = 1，其余位 0");
        assert_eq!(JournalRecord::parse(&last_bytes, 7), Some(last));
        let not_last = JournalRecord {
            place_in_publish: JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
            ..empty_record(3)
        };
        let not_last_bytes = not_last.to_bytes();
        assert_eq!(not_last_bytes[7], 0, "不是末条：记录标志整字节 0");
        assert_eq!(JournalRecord::parse(&not_last_bytes, 7), Some(not_last));
        let mut differing: Vec<usize> = (0..last_bytes.len())
            .filter(|offset| last_bytes[*offset] != not_last_bytes[*offset])
            .collect();
        differing.retain(|offset| !(46..78).contains(offset));
        assert_eq!(
            differing,
            vec![7],
            "两条记录只差记录标志那 1 字节（与头部校验和）"
        );
    }

    /// 记录标志位 0 之外有位为 1 的记录，读者当损坏（D23（journal 的角色与格式） 已定项 4「其余位写 0，读到非 0 当损坏」）：
    /// 位 0 本身是不是 1 都一样拒。头部校验和按改过的字节重封过，拦住它的只有记录标志那一判。
    #[test]
    fn a_record_whose_flags_byte_sets_any_bit_other_than_bit_zero_is_refused_by_the_parser() {
        for record_flags_byte in [0b0000_0010u8, 0b0000_0011, 0b1000_0001, 0xff] {
            let mut bytes = empty_record(4).to_bytes();
            bytes[7] = record_flags_byte;
            reseal_header_checksum(&mut bytes);
            assert_eq!(
                JournalRecord::parse(&bytes, 7),
                None,
                "记录标志 {record_flags_byte:#010b}：位 0 之外有位为 1，当损坏"
            );
        }
    }

    /// 本次发布内序号为 0 的记录，读者当损坏（D23（journal 的角色与格式） 已定项 4：序号从 1 起，读到 0 当那条记录损坏、断链即止）。
    /// 头部校验和按改过的字节重封过，拦住它的只有序号那一判。
    #[test]
    fn a_record_whose_ordinal_within_publish_is_zero_is_refused_by_the_parser() {
        let mut bytes = empty_record(5).to_bytes();
        bytes[87..91].copy_from_slice(&0u32.to_le_bytes());
        reseal_header_checksum(&mut bytes);
        assert_eq!(
            JournalRecord::parse(&bytes, 7),
            None,
            "序号 0：当这条记录损坏"
        );
    }

    /// 一次发布里从 0 数第 k 条记录的序号是 k + 1：第一条是 1（D23（journal 的角色与格式） 已定项 4「从 1 起」）。
    #[test]
    fn the_ordinal_of_the_record_at_offset_k_of_a_publish_is_k_plus_one() {
        assert_eq!(
            JournalRecordOrdinalWithinPublish::of_record_at_offset(0),
            JournalRecordOrdinalWithinPublish::FIRST
        );
        assert_eq!(JournalRecordOrdinalWithinPublish::FIRST.0, 1);
        assert_eq!(
            JournalRecordOrdinalWithinPublish::of_record_at_offset(143),
            JournalRecordOrdinalWithinPublish(144)
        );
    }

    /// 一条 4096 字节记录装 67 个点名项（(4096 − 311) ÷ 56 = 3785 ÷ 56 取整）：67 项的记录写得出、读得回；头里自述 68 项的记录
    /// （点名项区越过记录末尾）读者拒收，不按自述的项数切到 4096 之外（里程碑「第二个事务」并行线一验收第 4 条第三个变异：
    /// 点名项超过 67 仍塞进一条记录 ⇒ 记录解析拒收）。头校验和按改过的字节重算过，拦住它的只有项数那一判。
    #[test]
    fn a_record_whose_header_claims_more_named_units_than_one_record_holds_is_refused_by_the_parser(
    ) {
        use singlefs_format::JOURNAL_NAMED_ENTRIES_PER_RECORD;
        let named_unit = NamedUnit {
            locations: [
                LocationEntry {
                    device: crate::address::DeviceIdentity(0),
                    slot: crate::address::SlotNumber(50_176),
                    unit_checksum: 1,
                },
                LocationEntry {
                    device: crate::address::DeviceIdentity(1),
                    slot: crate::address::SlotNumber(50_176),
                    unit_checksum: 1,
                },
            ],
            unit_class: 1,
            birth_tree: TreeIdentifier(11),
            birth_txg: CheckpointTxg(3),
            key_tail: [0; 10],
        };
        let capacity = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("67");
        let full = JournalRecord {
            named: vec![named_unit; capacity],
            ..empty_record(1)
        };
        let full_bytes = full.to_bytes();
        assert_eq!(
            JournalRecord::parse(&full_bytes, 7),
            Some(full),
            "67 项正好装满一条记录"
        );
        let mut claims_one_more = full_bytes;
        let named_count_offset = 4 + 2 + 1 + 1 + 4;
        claims_one_more[named_count_offset..named_count_offset + 4]
            .copy_from_slice(&u32::try_from(capacity + 1).expect("68").to_le_bytes());
        let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
        let digest = wide_checksum_with_field_zeroed(
            &claims_one_more,
            record_bytes,
            JOURNAL_HEADER_CHECKSUM_OFFSET,
        );
        claims_one_more[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        assert_eq!(
            JournalRecord::parse(&claims_one_more, 7),
            None,
            "自述 68 个点名项的记录：点名项区越过 4096，读者拒收"
        );
    }

    #[test]
    fn record_is_4096_with_a_311_byte_header_and_round_trips() {
        let record = empty_record(1);
        let bytes = record.to_bytes();
        assert_eq!(bytes.len(), 4096);
        assert!(
            bytes[311..].iter().all(|byte| *byte == 0),
            "空记录 311 之后全 0"
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
