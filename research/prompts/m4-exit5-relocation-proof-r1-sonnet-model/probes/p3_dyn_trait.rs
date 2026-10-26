use vstd::prelude::*;
verus! {
pub trait PoolReader { fn read(&self, device: u64, offset: u64, length: usize) -> Option<Vec<u8>>; }
// relocation_checked_against_the_version 的 reader: &Reader（Reader: PoolReader + ?Sized），recovery.rs:762 是 &dyn PoolReader
pub fn read_it(reader: &dyn PoolReader, device: u64) -> (r: Option<Vec<u8>>) {
    reader.read(device, 0, 8)
}
}
fn main() {}
