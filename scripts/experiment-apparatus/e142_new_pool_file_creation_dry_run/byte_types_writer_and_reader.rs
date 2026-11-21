//! 设备、槽、偏移、检查点、实例代、事务号、树与出生序号的数值类型，写序，以及字节写入器与字节读取器。

use super::*;

// ───────────────────────── 地址空间的 newtype：混用编译不过 ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct DeviceIdentity(pub(super) u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct SlotNumber(pub(super) u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct DeviceOffset(pub(super) u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct CheckpointTxg(pub(super) u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct InstanceGeneration(pub(super) u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct TransactionNumber(pub(super) u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct TreeIdentifier(pub(super) u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct BirthSequence(pub(super) u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct JournalCounter(pub(super) u64);

impl SlotNumber {
    pub(super) fn device_offset(self) -> DeviceOffset {
        DeviceOffset(self.0 * SLOT_BYTES)
    }
}

/// C113 选定的 P1 写序：实例代号 4 + 事务号低 48 位。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct WriteOrder {
    pub(super) instance: InstanceGeneration,
    pub(super) transaction: TransactionNumber,
}

// ───────────────────────── 字节读写 ─────────────────────────

pub(super) struct ByteWriter {
    pub(super) bytes: Vec<u8>,
    pub(super) cursor: usize,
}

impl ByteWriter {
    pub(super) fn new(total_bytes: usize) -> Self {
        Self { bytes: vec![0u8; total_bytes], cursor: 0 }
    }
    pub(super) fn put(&mut self, slice: &[u8]) {
        self.bytes[self.cursor..self.cursor + slice.len()].copy_from_slice(slice);
        self.cursor += slice.len();
    }
    pub(super) fn put_u8(&mut self, value: u8) {
        self.put(&[value]);
    }
    pub(super) fn put_u16(&mut self, value: u16) {
        self.put(&value.to_le_bytes());
    }
    pub(super) fn put_u32(&mut self, value: u32) {
        self.put(&value.to_le_bytes());
    }
    pub(super) fn put_six_byte_unsigned(&mut self, value: u64) {
        assert!(value < (1u64 << 48), "48 位字段装不下 {value}");
        self.put(&value.to_le_bytes()[..6]);
    }
    pub(super) fn put_u64(&mut self, value: u64) {
        self.put(&value.to_le_bytes());
    }
    pub(super) fn skip(&mut self, byte_count: usize) {
        self.cursor += byte_count;
    }
    pub(super) fn position(&self) -> usize {
        self.cursor
    }
    pub(super) fn assert_position(&self, expected: u64, what: &str) {
        assert_eq!(self.cursor as u64, expected, "{what} 的偏移与字段表不符");
    }
}

pub(super) struct ByteReader<'bytes> {
    pub(super) bytes: &'bytes [u8],
    pub(super) cursor: usize,
}

impl<'bytes> ByteReader<'bytes> {
    pub(super) fn new(bytes: &'bytes [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }
    pub(super) fn at(bytes: &'bytes [u8], offset: usize) -> Self {
        Self { bytes, cursor: offset }
    }
    pub(super) fn take(&mut self, byte_count: usize) -> &'bytes [u8] {
        let slice = &self.bytes[self.cursor..self.cursor + byte_count];
        self.cursor += byte_count;
        slice
    }
    pub(super) fn get_u8(&mut self) -> u8 {
        self.take(1)[0]
    }
    pub(super) fn get_u16(&mut self) -> u16 {
        u16::from_le_bytes(self.take(2).try_into().expect("切了 2 字节"))
    }
    pub(super) fn get_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().expect("切了 4 字节"))
    }
    pub(super) fn get_six_byte_unsigned(&mut self) -> u64 {
        let mut eight = [0u8; 8];
        eight[..6].copy_from_slice(self.take(6));
        u64::from_le_bytes(eight)
    }
    pub(super) fn get_u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take(8).try_into().expect("切了 8 字节"))
    }
    pub(super) fn skip(&mut self, byte_count: usize) {
        self.cursor += byte_count;
    }
}
