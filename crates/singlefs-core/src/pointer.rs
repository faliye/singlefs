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

/// 出生序号：同一棵树在同一个 checkpoint 里每写出一个码 2 / 码 3 单元加 1（D19（块指针的结构与宽度预算） 已定项 9）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BirthSequence(pub u32);

/// 指针头部 50 字节：第一版 MAC / nonce 全 0、算法类型 0、压缩算法码 0、压后长度 0、extent 偏移 0。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerHead {
    pub birth_tree: TreeIdentifier,
    pub birth_txg: CheckpointTxg,
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
    pub fn read_from(reader: &mut ByteReader<'_>) -> Self {
        reader.skip(16 + 12 + 1 + 1 + 2 + 2);
        Self {
            birth_tree: TreeIdentifier(reader.get_u64()),
            birth_txg: CheckpointTxg(reader.get_u64()),
        }
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
    pub fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        self.head.write_to(writer);
        assert!(
            self.locations[0].device <= self.locations[1].device,
            "位置条目按设备身份升序（I-2.5）"
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
            "位置条目按设备身份升序（I-2.5）"
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
