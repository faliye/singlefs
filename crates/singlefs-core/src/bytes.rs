//! 定长字段的小端读写（D22（单元原子性怎么合成） 已定项 13：盘上定长字段一律小端）。

/// 往一段定长缓冲里顺序写字段；每一段的偏移用 [`ByteWriter::assert_position`] 钉在字段表上。
pub struct ByteWriter {
    bytes: Vec<u8>,
    cursor: usize,
}

impl ByteWriter {
    #[must_use]
    pub fn new(total_bytes: usize) -> Self {
        Self {
            bytes: vec![0u8; total_bytes],
            cursor: 0,
        }
    }
    pub fn put(&mut self, slice: &[u8]) {
        self.bytes[self.cursor..self.cursor + slice.len()].copy_from_slice(slice);
        self.cursor += slice.len();
    }
    pub fn put_u8(&mut self, value: u8) {
        self.put(&[value]);
    }
    pub fn put_u16(&mut self, value: u16) {
        self.put(&value.to_le_bytes());
    }
    pub fn put_u32(&mut self, value: u32) {
        self.put(&value.to_le_bytes());
    }
    /// 6 字节字段（16 KiB 槽号、jsn 计数器、事务号低 48 位）。
    pub fn put_six_byte_unsigned(&mut self, value: u64) {
        assert!(value < (1u64 << 48), "48 位字段装不下 {value}");
        self.put(&value.to_le_bytes()[..6]);
    }
    pub fn put_u64(&mut self, value: u64) {
        self.put(&value.to_le_bytes());
    }
    /// 留位、补齐：跳过的字节保持 0。
    pub fn skip(&mut self, byte_count: usize) {
        self.cursor += byte_count;
    }
    #[must_use]
    pub fn position(&self) -> usize {
        self.cursor
    }
    /// 字段表上的偏移与写到的位置不符就是写错了字段表，当场断言。
    pub fn assert_position(&self, expected: u64, what: &str) {
        let cursor = u64::try_from(self.cursor).expect("偏移装得进 u64");
        assert_eq!(cursor, expected, "{what} 的偏移与字段表不符");
    }
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// 从一段字节里顺序读字段。
pub struct ByteReader<'bytes> {
    bytes: &'bytes [u8],
    cursor: usize,
}

impl<'bytes> ByteReader<'bytes> {
    #[must_use]
    pub fn at(bytes: &'bytes [u8], offset: usize) -> Self {
        Self {
            bytes,
            cursor: offset,
        }
    }
    pub fn take(&mut self, byte_count: usize) -> &'bytes [u8] {
        let slice = &self.bytes[self.cursor..self.cursor + byte_count];
        self.cursor += byte_count;
        slice
    }
    pub fn get_u8(&mut self) -> u8 {
        self.take(1)[0]
    }
    pub fn get_u16(&mut self) -> u16 {
        u16::from_le_bytes(self.take(2).try_into().expect("切了 2 字节"))
    }
    pub fn get_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().expect("切了 4 字节"))
    }
    pub fn get_six_byte_unsigned(&mut self) -> u64 {
        let mut buffer = [0u8; 8];
        buffer[..6].copy_from_slice(self.take(6));
        u64::from_le_bytes(buffer)
    }
    pub fn get_u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take(8).try_into().expect("切了 8 字节"))
    }
    pub fn skip(&mut self, byte_count: usize) {
        self.cursor += byte_count;
    }
    #[must_use]
    pub fn position(&self) -> usize {
        self.cursor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_round_trip_in_little_endian_order() {
        let mut writer = ByteWriter::new(24);
        writer.put_u16(0x0102);
        writer.put_u32(0x0304_0506);
        writer.put_six_byte_unsigned(0x0708_090A_0B0C);
        writer.put_u64(0x0D0E_0F10_1112_1314);
        writer.skip(4);
        writer.assert_position(24, "测试");
        let bytes = writer.into_bytes();
        assert_eq!(&bytes[..2], &[0x02, 0x01], "小端");
        let mut reader = ByteReader::at(&bytes, 0);
        assert_eq!(reader.get_u16(), 0x0102);
        assert_eq!(reader.get_u32(), 0x0304_0506);
        assert_eq!(reader.get_six_byte_unsigned(), 0x0708_090A_0B0C);
        assert_eq!(reader.get_u64(), 0x0D0E_0F10_1112_1314);
        assert_eq!(reader.position(), 20);
    }
}
