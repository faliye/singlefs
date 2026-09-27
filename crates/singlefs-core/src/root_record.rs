//! 根记录（D22（单元原子性怎么合成） 已定项 7）：457 字节住一个判定宽度（physical_block_size）的根槽，
//! 自证校验和罩整槽含补齐、自身按 0 参与。行序即盘上顺序，偏移按行累加。

use singlefs_format::{NODE_POINTER_BYTES, ROOT_RECORD_BYTES, WIDE_CHECKSUM_BYTES};

use crate::address::{CheckpointTxg, InstanceGeneration};
use crate::bytes::{ByteReader, ByteWriter};
use crate::checksum::{wide_checksum_field_holds, wide_checksum_with_field_zeroed};
use crate::pointer::NodePointer;

pub const ROOT_MAGIC: [u8; 4] = *b"SFSR";
/// magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8 + 树表指针 86 + 树 ID 水位 8 + 回退下界 F 8 = 138。
pub const ROOT_CHECKSUM_OFFSET: usize = 4 + 16 + 4 + 4 + 8 + 86 + 8 + 8;
/// flags 位 0 = 卸载记号：正常卸载那一串写的根置 1，其余根写 0（D22（单元原子性怎么合成） 已定项 7，用户 2026-09-26 定）；
/// 位 n 住第 n div 8 个字节的第 n mod 8 低位（已定项 13）。其余位第一版恒 0、非 0 拒收（已定项 17）。
/// 内存里是 [`RootRecord::unmount_marker`]（[`UnmountMarker`]），只有 `mount::unmount` 那一串写 1。
pub const ROOT_RECORD_FLAG_UNMOUNT_MARKER: u32 = 1 << 0;

/// 根记录 flags 位 0 的卸载记号（D22（单元原子性怎么合成） 已定项 7）：这条根是不是正常卸载那一串空发布写的
/// （D16（发布语义） 已定项 1「正常卸载」）。池级 checker 按它给 I-7.9（回退下界 F 不高于抬 F 的上限） 分两支判；
/// 「记号只由卸载入口打」盘上没有别的东西核它，归层 0 录制流（C557（卸载记号只由卸载入口打没有检查））。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnmountMarker {
    /// flags 位 0 = 1：正常卸载那一串写的根（`mount::unmount`）。
    WrittenByTheUnmountSequence,
    /// flags 位 0 = 0：别的每一条根——mkfs 的第 0 代、暖机、写行、发布、准入抬 F、回退，以及由记录施加出来的那一版。
    NotWrittenByTheUnmountSequence,
}

impl UnmountMarker {
    /// 写进 flags 那 4 字节的值：记号那一位，其余位恒 0。
    #[must_use]
    pub const fn flags_bits(self) -> u32 {
        match self {
            UnmountMarker::WrittenByTheUnmountSequence => ROOT_RECORD_FLAG_UNMOUNT_MARKER,
            UnmountMarker::NotWrittenByTheUnmountSequence => 0,
        }
    }

    /// 从已经判过「记号之外的位全是 0」的 flags 读回记号。
    #[must_use]
    const fn of_flags_bits_carrying_only_the_marker(flags: u32) -> Self {
        if flags & ROOT_RECORD_FLAG_UNMOUNT_MARKER == 0 {
            UnmountMarker::NotWrittenByTheUnmountSequence
        } else {
            UnmountMarker::WrittenByTheUnmountSequence
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootRecord {
    pub filesystem_identifier: [u8; 16],
    /// flags 位 0（D22（单元原子性怎么合成） 已定项 7）；flags 其余位第一版恒 0，不进内存。
    pub unmount_marker: UnmountMarker,
    pub instance: InstanceGeneration,
    pub checkpoint_txg: CheckpointTxg,
    pub tree_table: NodePointer,
    pub tree_identifier_watermark: u64,
    pub rollback_floor: CheckpointTxg,
    pub instance_table: NodePointer,
    /// 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
    pub mapping_root: NodePointer,
    /// 树表 0 条的那一版的分配记录树的根（2026-09-23 用户定案随 C512（树表 0 条的一版上被换下的单元记在哪） 加）。
    ///
    /// **只有树表 0 条的那一版用这一项**：它没有树表条目可放（往树表里写一条，`tree_table_has_no_entries` 当场翻面，
    /// 按 `PreviousVersion::WithoutFile` / `WithFile` 分流的每一处跟着变）。带文件的一版恒 [`NodePointer::empty_root`]
    /// ——那一版的分配记录树住树表条目（D8（核心索引结构） 已定项 8），两处都写就成了同一个量的两份手抄。
    /// mkfs 的第 0 代也恒全零：那一版的账由 `mount::format_time_allocator` 从实例表与树表两条指针直接算。
    pub allocation_record_tree_root: NodePointer,
}

impl RootRecord {
    /// 写成一个 `slot_bytes` 宽的槽；记录 457 字节，其余补 0。
    #[must_use]
    pub fn to_slot(&self, slot_bytes: usize) -> Vec<u8> {
        assert!(
            slot_bytes >= usize::try_from(ROOT_RECORD_BYTES).expect("457"),
            "根槽装不下根记录"
        );
        let mut writer = ByteWriter::new(slot_bytes);
        writer.put(&ROOT_MAGIC);
        writer.put(&self.filesystem_identifier);
        writer.put_u32(self.unmount_marker.flags_bits());
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
        self.allocation_record_tree_root.write_to(&mut writer);
        writer.put_u8(0); // 算法类型：未加密
        writer.skip(12 + 16); // nonce、MAC：第一版留位全 0
        writer.assert_position(ROOT_RECORD_BYTES, "根记录");
        let mut bytes = writer.into_bytes();
        let digest = wide_checksum_with_field_zeroed(&bytes, slot_bytes, ROOT_CHECKSUM_OFFSET);
        bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        bytes
    }

    /// 读者：magic、整槽校验和、fsid、flags 四关。flags 只收卸载记号那一位（[`ROOT_RECORD_FLAG_UNMOUNT_MARKER`]），
    /// 别的位非 0 拒收；记号照实读进 [`RootRecord::unmount_marker`]（挂载与恢复不按它分支，按它分支的是池级 checker 的 I-7.9）。
    #[must_use]
    pub fn parse_slot(bytes: &[u8], expected_filesystem_identifier: &[u8; 16]) -> Option<Self> {
        if bytes.len() < usize::try_from(ROOT_RECORD_BYTES).expect("457")
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
        let flags = reader.get_u32();
        if flags & !ROOT_RECORD_FLAG_UNMOUNT_MARKER != 0 {
            return None;
        }
        let unmount_marker = UnmountMarker::of_flags_bits_carrying_only_the_marker(flags);
        let instance = InstanceGeneration(reader.get_u32());
        let checkpoint_txg = CheckpointTxg(reader.get_u64());
        let tree_table = NodePointer::read_from(&mut reader);
        let tree_identifier_watermark = reader.get_u64();
        let rollback_floor = CheckpointTxg(reader.get_u64());
        reader.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
        let instance_table = NodePointer::read_from(&mut reader);
        let mapping_root = NodePointer::read_from(&mut reader);
        let allocation_record_tree_root = NodePointer::read_from(&mut reader);
        assert_eq!(
            reader.position(),
            ROOT_CHECKSUM_OFFSET + 32 + 3 * usize::try_from(NODE_POINTER_BYTES).expect("86")
        );
        Some(Self {
            filesystem_identifier,
            unmount_marker,
            instance,
            checkpoint_txg,
            tree_table,
            tree_identifier_watermark,
            rollback_floor,
            instance_table,
            mapping_root,
            allocation_record_tree_root,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RootRecord {
        RootRecord {
            filesystem_identifier: [5u8; 16],
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: InstanceGeneration(0),
            checkpoint_txg: CheckpointTxg(0),
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark: 11,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
        }
    }

    #[test]
    fn root_record_is_457_bytes_with_the_checksum_at_138_and_round_trips() {
        let slot = sample().to_slot(512);
        assert_eq!(slot.len(), 512);
        assert!(slot[457..].iter().all(|byte| *byte == 0));
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

    /// 把槽里 flags 那 4 字节（偏移 20）改成 `flags`，自证校验和重封：magic、校验和、fsid 三关照样过，拒不拒只看 flags 判。
    fn slot_with_flags(flags: u32) -> Vec<u8> {
        let mut slot = sample().to_slot(512);
        slot[20..24].copy_from_slice(&flags.to_le_bytes());
        let digest = wide_checksum_with_field_zeroed(&slot, 512, ROOT_CHECKSUM_OFFSET);
        slot[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        slot
    }

    /// flags 位 0 是卸载记号（D22（单元原子性怎么合成） 已定项 7）：带它的根照收、记号照实读进 `unmount_marker`、其余字段原样读回；
    /// 别的位（位 1、位 31、带着记号又多一位）非 0 照旧拒收。
    #[test]
    fn root_carrying_the_unmount_marker_is_accepted_and_any_other_flag_bit_is_refused() {
        assert_eq!(
            ROOT_RECORD_FLAG_UNMOUNT_MARKER, 0x0000_0001,
            "卸载记号是位 0"
        );
        let marked = slot_with_flags(ROOT_RECORD_FLAG_UNMOUNT_MARKER);
        assert_eq!(
            marked[20], 0x01,
            "位 0 住 flags 第一个字节的最低位（已定项 13）"
        );
        assert_eq!(
            RootRecord::parse_slot(&marked, &[5u8; 16]),
            Some(RootRecord {
                unmount_marker: UnmountMarker::WrittenByTheUnmountSequence,
                ..sample()
            }),
            "带卸载记号的根照收，记号读成卸载那一串写的，别的字段一个不差"
        );
        for refused_flags in [
            1u32 << 1,
            1u32 << 31,
            ROOT_RECORD_FLAG_UNMOUNT_MARKER | (1 << 1),
        ] {
            assert_eq!(
                RootRecord::parse_slot(&slot_with_flags(refused_flags), &[5u8; 16]),
                None,
                "flags {refused_flags:#010x}：卸载记号之外的位非 0 拒收"
            );
        }
    }

    /// 写者照 `unmount_marker` 写 flags：卸载那一串写的根 flags 是 1（只有位 0），别的根 flags 是 0；两样都原样读回。
    #[test]
    fn the_writer_puts_the_unmount_marker_into_flag_bit_zero_and_leaves_it_clear_on_every_other_root(
    ) {
        let marked_root = RootRecord {
            unmount_marker: UnmountMarker::WrittenByTheUnmountSequence,
            ..sample()
        };
        let marked_slot = marked_root.to_slot(512);
        assert_eq!(
            marked_slot[20..24],
            [0x01, 0x00, 0x00, 0x00],
            "卸载那一串写的根：flags 只有位 0"
        );
        assert_eq!(
            RootRecord::parse_slot(&marked_slot, &[5u8; 16]),
            Some(marked_root),
            "带记号写出去的根带记号读回来"
        );
        let unmarked_slot = sample().to_slot(512);
        assert_eq!(
            unmarked_slot[20..24],
            [0x00, 0x00, 0x00, 0x00],
            "别的根：flags 全 0"
        );
        assert_eq!(
            RootRecord::parse_slot(&unmarked_slot, &[5u8; 16]),
            Some(sample()),
            "不带记号写出去的根不带记号读回来"
        );
    }
}
