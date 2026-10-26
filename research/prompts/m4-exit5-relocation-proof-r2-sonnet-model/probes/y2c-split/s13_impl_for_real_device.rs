// 文件后端/内存盘这种「真」设备要实现 trait：实现体必须满足 trait 里写的 ensures，读盘本身没法在 Verus 里证
use vstd::prelude::*;
verus! {
pub trait Device {
    fn read(&self, offset: u64, length: usize) -> (r: Option<Vec<u8>>)
        ensures r.is_some() ==> r.unwrap()@.len() == length;
}
pub struct MemoryDisk { pub bytes: Vec<u8> }
impl Device for MemoryDisk {
    fn read(&self, offset: u64, length: usize) -> (r: Option<Vec<u8>>) {
        let start = offset as usize;
        if start > self.bytes.len() || length > self.bytes.len() - start { return None; }
        let mut out: Vec<u8> = Vec::new();
        let mut i = 0usize;
        while i < length
            invariant i <= length, out@.len() == i, start + length <= self.bytes@.len(),
            decreases length - i,
        {
            out.push(self.bytes[start + i]);
            i += 1;
        }
        Some(out)
    }
}
}
fn main() {}
