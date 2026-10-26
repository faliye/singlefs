// 探针 y1-a：类型写在 verus! 外面（像 core 里 AllocationRecord 那样的普通 struct），verus! 里的函数用它。
use vstd::prelude::*;

#[derive(Clone, Copy)]
pub struct Region { pub start: u64, pub count: u64 }
impl Region {
    pub fn contains_slot(&self, slot: u64) -> bool { slot >= self.start && slot - self.start < self.count }
}
pub struct AllocationRecord { pub device: u32, pub slot: u64, pub is_released: bool }

verus! {
pub fn region_is_empty_by_the_records(records: &[AllocationRecord], region: Region) -> bool {
    !records.iter().any(|r| !r.is_released && region.contains_slot(r.slot))
}
}
fn main() {}
