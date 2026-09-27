//! 块指针（D19（块指针的结构与宽度预算） 已定项 7 / 已定项 8 / 已定项 11）：
//! 头部 50（MAC 16 / nonce 12 / 算法类型 1 / 压缩算法码 1 / 压后长度 2 / extent 偏移 2 / 出生树 8 / 出生 txg 8）
//! + 位置条目 14 × 2（按设备身份升序，I-2.5）+ key 尾段：指向码 2 / 码 3 的带实例代号 4 + 出生序号 4（86）。

use singlefs_format::{DATA_POINTER_BYTES, LOC_ENTRY, NODE_POINTER_BYTES, POINTER_HEAD_BYTES};

use crate::address::{
    CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
};
use crate::bytes::{ByteReader, ByteWriter};
use crate::unit::WriteOrder;

/// 位置条目：设备身份 4 + 16 KiB 槽号 6 + 密文校验和 4（加密关时是整单元 CRC-32C）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationEntry {
    pub device: DeviceIdentity,
    pub slot: SlotNumber,
    pub unit_checksum: u32,
}

impl LocationEntry {
    pub fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        writer.put_u32(self.device.0);
        writer.put_six_byte_unsigned(self.slot.0);
        writer.put_u32(self.unit_checksum);
        assert_eq!(
            writer.position() - start,
            usize::try_from(LOC_ENTRY).expect("14")
        );
    }
    pub fn read_from(reader: &mut ByteReader<'_>) -> Self {
        Self {
            device: DeviceIdentity(reader.get_u32()),
            slot: SlotNumber(reader.get_six_byte_unsigned()),
            unit_checksum: reader.get_u32(),
        }
    }
}

/// 一条盘上指针里两条位置条目的槽号不等。**这是一份坏镜像，不是我们的不变量被破坏**：位置条目里的槽号是盘上的
/// 6 字节，可以是任何值；D2（RAID 条带策略） 已定项 10 要的是「每个副本一条位置条目」，而「两条都落在同一个槽」
/// 是第一版的布局约定（一个单元整个落在一列上、两盘各一份）——盘上写着两个不同的槽，说明这一版不是这个实现写出来的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationEntriesOnDifferentSlots {
    pub devices: [DeviceIdentity; 2],
    pub slots: [SlotNumber; 2],
}

/// 一条盘上指针的两条位置条目共用的那个槽号（第一版两盘同槽）：不等就交回错误，由调用方在动任何状态之前拒绝。
/// 读路径上每一处「拿盘上指针当落点用」都走这一处判，不各写各的（`code-discipline.md`「重复要生成，不许手抄」）。
///
/// # Errors
/// 两条位置条目的槽号不等 ⇒ [`LocationEntriesOnDifferentSlots`]。改之前这一判是 `mount.rs` 与 `transaction.rs` 里
/// 各一句 `assert_eq!`：一份两条位置条目不同槽的镜像把可写挂载打 panic（panic 面普查 R5）。
pub fn slot_shared_by_both_location_entries(
    locations: &[LocationEntry; 2],
) -> Result<SlotNumber, LocationEntriesOnDifferentSlots> {
    if locations[0].slot == locations[1].slot {
        Ok(locations[0].slot)
    } else {
        Err(LocationEntriesOnDifferentSlots {
            devices: [locations[0].device, locations[1].device],
            slots: [locations[0].slot, locations[1].slot],
        })
    }
}

/// 一条盘上指针的两条位置条目不按设备身份严格升序（I-2.5（位置条目按设备身份升序）：相同或逆序一律判损坏）。
/// **这是一份坏镜像，不是我们的不变量被破坏**：写者（[`NodePointer::write_to`] / [`DataPointer::write_to`]）按升序写、在那里断言，
/// 而照抄上一版的发布会把盘上读来的指针原样写回去（树表条目、多层码 2 树的内部条目、根记录）——不在读进来的地方判，
/// 坏镜像就走到写者的断言上（C476 普查「缺口」那张表 `pointer.rs` 那一行，实审 A3a 实测坐实）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationEntriesNotAscendingByDevice {
    pub devices: [DeviceIdentity; 2],
}

/// 两条位置条目按设备身份严格升序（I-2.5）。
///
/// # Errors
/// 相同或逆序 ⇒ [`LocationEntriesNotAscendingByDevice`]。
pub fn location_entries_ascend_by_device(
    locations: &[LocationEntry; 2],
) -> Result<(), LocationEntriesNotAscendingByDevice> {
    if locations[0].device < locations[1].device {
        Ok(())
    } else {
        Err(LocationEntriesNotAscendingByDevice {
            devices: [locations[0].device, locations[1].device],
        })
    }
}

/// 出生序号：同一棵树在同一个 checkpoint 里每写出一个码 2 / 码 3 单元加 1（D19（块指针的结构与宽度预算） 已定项 9）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BirthSequence(pub u32);

/// 指针头部 50 字节：第一版 MAC / nonce 全 0、算法类型 0、压缩算法码 0、压后长度 0、extent 偏移 0。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerHead {
    pub birth_tree: TreeIdentifier,
    pub birth_txg: CheckpointTxg,
}

/// 指针头部里为加密与压缩留的那几段（D19（块指针的结构与宽度预算） 已定项 3 / 已定项 7）读到的不是第一版写的取值。
/// 这几段是**盘上读来的字节**、不是我们的不变量：读者判到就交回它，由调用方把那条指针所在的结构当损坏处置，不拿这条指针去解引用。
/// 第一版的取值：加密不进第一个可运行版本（D9（加密） 已定项 10），写者把 MAC 16、nonce 12、算法类型 1 写 0；
/// 压缩算法码登记表第一版只有码 0「不压缩」，码 0 时压后长度恒 0（D9（加密） 已定项 14）。extent 偏移那 2 字节不在这里判：
/// 读到非 0 怎么处置没有条款（第一版写 0）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerHeadFieldOutsideTheFirstVersion {
    /// MAC 16 或 nonce 12 里有非 0 字节。D19（块指针的结构与宽度预算） 已定项 3 只写了「不加密的卷把它们留成空位」，
    /// 读到非 0 怎么处置条款没写；这里按 I-2.4（头校验和覆盖范围） 给单元头那 29 字节定的「恒 0、读者遇到非 0 一律判该结构损坏」同一个读法。
    MacOrNonceNotZero,
    /// 算法类型不是 0（未加密）：1、2 是登记过的加密算法（D22（单元原子性怎么合成） 已定项 17 的登记表），第一版不支持加密
    /// （D9（加密） 已定项 10），别的码不认识——都判这条指针无法解引用（D9（加密） 已定项 14 末句：与压缩算法码同一条政策）。
    AlgorithmTypeNotUnencrypted { algorithm_type: u8 },
    /// 压缩算法码不是 0：登记表第一版只有码 0（D9（加密） 已定项 14），不认识的码判这条指针无法解引用。
    CompressionAlgorithmCodeNotRecognized { compression_algorithm_code: u8 },
    /// 压缩算法码是 0 而压后长度非 0：判这条指针损坏（D9（加密） 已定项 14「码 0 时它恒 0，非 0 判该指针损坏」）。
    CompressedLengthNotZeroWithoutCompression { compressed_length: u16 },
}

/// 指针头部按字段表读出来的样子，加密与压缩那几段还没判：[`PointerHead::read_from`] 与
/// [`PointerHead::read_judging_the_encryption_and_compression_fields_from`] 共用这一份读法，字段表的偏移只写一处。
struct PointerHeadAsRead {
    is_every_mac_and_nonce_byte_zero: bool,
    algorithm_type: u8,
    compression_algorithm_code: u8,
    compressed_length: u16,
    head: PointerHead,
}

impl PointerHeadAsRead {
    fn read_from(reader: &mut ByteReader<'_>) -> Self {
        let is_every_mac_and_nonce_byte_zero = reader.take(16 + 12).iter().all(|byte| *byte == 0);
        let algorithm_type = reader.get_u8();
        let compression_algorithm_code = reader.get_u8();
        let compressed_length = reader.get_u16();
        reader.skip(2); // extent 偏移：第一版写 0，读到非 0 怎么处置没有条款，不判
        Self {
            is_every_mac_and_nonce_byte_zero,
            algorithm_type,
            compression_algorithm_code,
            compressed_length,
            head: PointerHead {
                birth_tree: TreeIdentifier(reader.get_u64()),
                birth_txg: CheckpointTxg(reader.get_u64()),
            },
        }
    }

    /// 加密与压缩那几段按第一版的取值判（[`PointerHeadFieldOutsideTheFirstVersion`] 各成员的出处）。
    fn judged(self) -> Result<PointerHead, PointerHeadFieldOutsideTheFirstVersion> {
        if !self.is_every_mac_and_nonce_byte_zero {
            return Err(PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero);
        }
        if self.algorithm_type != 0 {
            return Err(
                PointerHeadFieldOutsideTheFirstVersion::AlgorithmTypeNotUnencrypted {
                    algorithm_type: self.algorithm_type,
                },
            );
        }
        if self.compression_algorithm_code != 0 {
            return Err(
                PointerHeadFieldOutsideTheFirstVersion::CompressionAlgorithmCodeNotRecognized {
                    compression_algorithm_code: self.compression_algorithm_code,
                },
            );
        }
        if self.compressed_length != 0 {
            return Err(
                PointerHeadFieldOutsideTheFirstVersion::CompressedLengthNotZeroWithoutCompression {
                    compressed_length: self.compressed_length,
                },
            );
        }
        Ok(self.head)
    }
}

impl PointerHead {
    pub fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        writer.skip(16 + 12);
        writer.put_u8(0);
        writer.put_u8(0);
        writer.put_u16(0);
        writer.put_u16(0);
        assert_eq!(writer.position() - start, 34, "出生树落在指针头部偏移 34");
        writer.put_u64(self.birth_tree.0);
        writer.put_u64(self.birth_txg.0);
        assert_eq!(
            writer.position() - start,
            usize::try_from(POINTER_HEAD_BYTES).expect("50")
        );
    }
    /// 读头部，加密与压缩那几段不判（还没换成判的那几个读者用：`root_record`、`records`、`instance_table`、`extent_tree`、
    /// `allocation_record_tree` 里的指针，实审 A3a 的报告列了它们）。
    pub fn read_from(reader: &mut ByteReader<'_>) -> Self {
        PointerHeadAsRead::read_from(reader).head
    }

    /// 读头部，加密与压缩那几段按第一版的取值判。
    ///
    /// # Errors
    /// 那几段有一段不是第一版的取值 ⇒ [`PointerHeadFieldOutsideTheFirstVersion`]。
    pub fn read_judging_the_encryption_and_compression_fields_from(
        reader: &mut ByteReader<'_>,
    ) -> Result<Self, PointerHeadFieldOutsideTheFirstVersion> {
        PointerHeadAsRead::read_from(reader).judged()
    }
}

/// 指向码 2 / 码 3 的指针 86。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodePointer {
    pub head: PointerHead,
    pub locations: [LocationEntry; 2],
    pub instance: InstanceGeneration,
    pub birth_sequence: BirthSequence,
}

impl NodePointer {
    /// 没有根的树：整条指针全 0（I-2.5 对全零指针整体豁免升序判定）。
    #[must_use]
    pub const fn empty_root() -> Self {
        let zero = LocationEntry {
            device: DeviceIdentity(0),
            slot: SlotNumber(0),
            unit_checksum: 0,
        };
        Self {
            head: PointerHead {
                birth_tree: TreeIdentifier(0),
                birth_txg: CheckpointTxg(0),
            },
            locations: [zero, zero],
            instance: InstanceGeneration(0),
            birth_sequence: BirthSequence(0),
        }
    }
    /// 两条位置条目按设备身份严格升序（I-2.5）；整条全零的指针（「注册了但还没有根」）整条豁免。
    ///
    /// # Errors
    /// 不是全零、而两条位置条目相同或逆序 ⇒ [`LocationEntriesNotAscendingByDevice`]。
    pub fn location_entries_ascend_by_device(
        &self,
    ) -> Result<(), LocationEntriesNotAscendingByDevice> {
        if *self == Self::empty_root() {
            return Ok(());
        }
        location_entries_ascend_by_device(&self.locations)
    }

    pub fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        self.head.write_to(writer);
        assert!(
            self.locations[0].device <= self.locations[1].device,
            "位置条目按设备身份升序（I-2.5）：盘上读来、要照抄写回的指针在 recovery::rebuild_version 判过"
        );
        for location in &self.locations {
            location.write_to(writer);
        }
        writer.put_u32(self.instance.0);
        writer.put_u32(self.birth_sequence.0);
        assert_eq!(
            writer.position() - start,
            usize::try_from(NODE_POINTER_BYTES).expect("86")
        );
    }
    pub fn read_from(reader: &mut ByteReader<'_>) -> Self {
        let head = PointerHead::read_from(reader);
        Self::read_after_the_head(head, reader)
    }

    /// 同 [`Self::read_from`]，头部里加密与压缩那几段按第一版的取值判（[`PointerHead::read_judging_the_encryption_and_compression_fields_from`]）。
    ///
    /// # Errors
    /// 那几段有一段不是第一版的取值 ⇒ [`PointerHeadFieldOutsideTheFirstVersion`]。
    pub fn read_judging_the_encryption_and_compression_fields_from(
        reader: &mut ByteReader<'_>,
    ) -> Result<Self, PointerHeadFieldOutsideTheFirstVersion> {
        let head = PointerHead::read_judging_the_encryption_and_compression_fields_from(reader)?;
        Ok(Self::read_after_the_head(head, reader))
    }

    fn read_after_the_head(head: PointerHead, reader: &mut ByteReader<'_>) -> Self {
        let locations = [
            LocationEntry::read_from(reader),
            LocationEntry::read_from(reader),
        ];
        Self {
            head,
            locations,
            instance: InstanceGeneration(reader.get_u32()),
            birth_sequence: BirthSequence(reader.get_u32()),
        }
    }
}

/// 指向码 1 数据单元的指针 88：头部 50 + 位置条目 14 × 2 + 写序 10（D19（块指针的结构与宽度预算） 已定项 8）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataPointer {
    pub head: PointerHead,
    pub locations: [LocationEntry; 2],
    pub write_order: WriteOrder,
}

impl DataPointer {
    pub fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        self.head.write_to(writer);
        assert!(
            self.locations[0].device <= self.locations[1].device,
            "位置条目按设备身份升序（I-2.5）：盘上读来、要照抄写回的指针在 recovery::rebuild_version 判过"
        );
        for location in &self.locations {
            location.write_to(writer);
        }
        writer.put_u32(self.write_order.instance.0);
        writer.put_six_byte_unsigned(self.write_order.transaction);
        assert_eq!(
            writer.position() - start,
            usize::try_from(DATA_POINTER_BYTES).expect("88")
        );
    }
    pub fn read_from(reader: &mut ByteReader<'_>) -> Self {
        let head = PointerHead::read_from(reader);
        Self::read_after_the_head(head, reader)
    }

    /// 同 [`Self::read_from`]，头部里加密与压缩那几段按第一版的取值判（[`PointerHead::read_judging_the_encryption_and_compression_fields_from`]）。
    ///
    /// # Errors
    /// 那几段有一段不是第一版的取值 ⇒ [`PointerHeadFieldOutsideTheFirstVersion`]。
    pub fn read_judging_the_encryption_and_compression_fields_from(
        reader: &mut ByteReader<'_>,
    ) -> Result<Self, PointerHeadFieldOutsideTheFirstVersion> {
        let head = PointerHead::read_judging_the_encryption_and_compression_fields_from(reader)?;
        Ok(Self::read_after_the_head(head, reader))
    }

    fn read_after_the_head(head: PointerHead, reader: &mut ByteReader<'_>) -> Self {
        let locations = [
            LocationEntry::read_from(reader),
            LocationEntry::read_from(reader),
        ];
        let instance = InstanceGeneration(reader.get_u32());
        let transaction = reader.get_six_byte_unsigned();
        Self {
            head,
            locations,
            write_order: WriteOrder {
                instance,
                transaction,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_pointer_is_86_bytes_and_round_trips() {
        let pointer = NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(0),
                birth_txg: CheckpointTxg(0),
            },
            locations: [
                LocationEntry {
                    device: DeviceIdentity(0),
                    slot: SlotNumber(50178),
                    unit_checksum: 0xAB,
                },
                LocationEntry {
                    device: DeviceIdentity(1),
                    slot: SlotNumber(50178),
                    unit_checksum: 0xAB,
                },
            ],
            instance: InstanceGeneration(0),
            birth_sequence: BirthSequence(1),
        };
        let mut writer = ByteWriter::new(86);
        pointer.write_to(&mut writer);
        let bytes = writer.into_bytes();
        assert_eq!(bytes[34], 0, "出生树 0");
        assert_eq!(
            &bytes[50..54],
            &0u32.to_le_bytes(),
            "第一条位置条目的设备身份"
        );
        assert_eq!(
            &bytes[54..60],
            &50178u64.to_le_bytes()[..6],
            "槽号 6 字节小端"
        );
        assert_eq!(
            NodePointer::read_from(&mut ByteReader::at(&bytes, 0)),
            pointer
        );
        assert_eq!(NodePointer::empty_root().locations[1].slot, SlotNumber(0));
    }
}
