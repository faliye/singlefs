//! 根记录（D22（单元原子性怎么合成） 已定项 7）：371 字节住一个判定宽度（physical_block_size）的根槽，
//! 自证校验和罩整槽含补齐、自身按 0 参与。行序即盘上顺序，偏移按行累加。

use singlefs_format::{NODE_POINTER_BYTES, ROOT_RECORD_BYTES, WIDE_CHECKSUM_BYTES};

use crate::address::{CheckpointTxg, InstanceGeneration};
use crate::bytes::{ByteReader, ByteWriter};
use crate::checksum::{wide_checksum_field_holds, wide_checksum_with_field_zeroed};
use crate::pointer::NodePointer;

pub const ROOT_MAGIC: [u8; 4] = *b"SFSR";
/// magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8 + 树表指针 86 + 树 ID 水位 8 + 回退下界 F 8 = 138。
pub const ROOT_CHECKSUM_OFFSET: usize = 4 + 16 + 4 + 4 + 8 + 86 + 8 + 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootRecord {
    pub filesystem_identifier: [u8; 16],
    pub instance: InstanceGeneration,
    pub checkpoint_txg: CheckpointTxg,
    pub tree_table: NodePointer,
    pub tree_identifier_watermark: u64,
    pub rollback_floor: CheckpointTxg,
    pub instance_table: NodePointer,
    /// 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
    pub mapping_root: NodePointer,
}

impl RootRecord {
    /// 写成一个 `slot_bytes` 宽的槽；记录 371 字节，其余补 0。
    #[must_use]
    pub fn to_slot(&self, slot_bytes: usize) -> Vec<u8> {
        assert!(
            slot_bytes >= usize::try_from(ROOT_RECORD_BYTES).expect("371"),
            "根槽装不下根记录"
        );
        let mut writer = ByteWriter::new(slot_bytes);
        writer.put(&ROOT_MAGIC);
        writer.put(&self.filesystem_identifier);
        writer.put_u32(0); // flags：第一版恒 0、非 0 拒收
        writer.put_u32(self.instance.0);
        writer.put_u64(self.checkpoint_txg.0);
        self.tree_table.write_to(&mut writer);
        writer.put_u64(self.tree_identifier_watermark);
        writer.put_u64(self.rollback_floor.0);
        writer.assert_position(
            u64::try_from(ROOT_CHECKSUM_OFFSET).expect("138"),
            "根记录自证校验和",
        );
        writer.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
        self.instance_table.write_to(&mut writer);
        self.mapping_root.write_to(&mut writer);
        writer.put_u8(0); // 算法类型：未加密
        writer.skip(12 + 16); // nonce、MAC：第一版留位全 0
        writer.assert_position(ROOT_RECORD_BYTES, "根记录");
        let mut bytes = writer.into_bytes();
        let digest = wide_checksum_with_field_zeroed(&bytes, slot_bytes, ROOT_CHECKSUM_OFFSET);
        bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        bytes
    }

    /// 读者：magic、整槽校验和、fsid、flags 四关。
    #[must_use]
    pub fn parse_slot(bytes: &[u8], expected_filesystem_identifier: &[u8; 16]) -> Option<Self> {
        if bytes.len() < usize::try_from(ROOT_RECORD_BYTES).expect("371")
            || bytes[..4] != ROOT_MAGIC
        {
            return None;
        }
        if !wide_checksum_field_holds(bytes, bytes.len(), ROOT_CHECKSUM_OFFSET) {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 4);
        let filesystem_identifier: [u8; 16] = reader.take(16).try_into().expect("切了 16 字节");
        if &filesystem_identifier != expected_filesystem_identifier {
            return None;
        }
        if reader.get_u32() != 0 {
            return None;
        }
        let instance = InstanceGeneration(reader.get_u32());
        let checkpoint_txg = CheckpointTxg(reader.get_u64());
        let tree_table = NodePointer::read_from(&mut reader);
        let tree_identifier_watermark = reader.get_u64();
        let rollback_floor = CheckpointTxg(reader.get_u64());
        reader.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
        let instance_table = NodePointer::read_from(&mut reader);
        let mapping_root = NodePointer::read_from(&mut reader);
        assert_eq!(
            reader.position(),
            ROOT_CHECKSUM_OFFSET + 32 + 2 * usize::try_from(NODE_POINTER_BYTES).expect("86")
        );
        Some(Self {
            filesystem_identifier,
            instance,
            checkpoint_txg,
            tree_table,
            tree_identifier_watermark,
            rollback_floor,
            instance_table,
            mapping_root,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RootRecord {
        RootRecord {
            filesystem_identifier: [5u8; 16],
            instance: InstanceGeneration(0),
            checkpoint_txg: CheckpointTxg(0),
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark: 11,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
        }
    }

    #[test]
    fn root_record_is_371_bytes_with_the_checksum_at_138_and_round_trips() {
        let slot = sample().to_slot(512);
        assert_eq!(slot.len(), 512);
        assert!(slot[371..].iter().all(|byte| *byte == 0));
        assert_eq!(&slot[..4], b"SFSR");
        assert_eq!(RootRecord::parse_slot(&slot, &[5u8; 16]), Some(sample()));
        assert_eq!(
            RootRecord::parse_slot(&slot, &[6u8; 16]),
            None,
            "fsid 不符不可择"
        );
        let mut torn = slot.clone();
        torn[500] ^= 1;
        assert_eq!(
            RootRecord::parse_slot(&torn, &[5u8; 16]),
            None,
            "补齐区改一字节：自证校验和罩整槽"
        );
    }
}
