pub mod walk;
mod rebuild;

pub(crate) const HEADER_MAGIC: &[u8] = b"SFSU";

pub(crate) fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

pub trait ImageReader {
    fn read(&self, offset: u64) -> Option<Vec<u8>>;
}
