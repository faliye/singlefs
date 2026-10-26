// 探针 y1-c：类型在 verus! 外，用 external_type_specification 把它交给 Verus（字段可见），--no-cheating 下过不过。
use vstd::prelude::*;
pub struct AllocationRecord { pub device: u32, pub slot: u64, pub is_released: bool }
verus! {
#[verifier::external_type_specification]
pub struct ExAllocationRecord(crate::AllocationRecord);
pub fn released_count_zero(r: &AllocationRecord) -> (b: bool)
    ensures b == r.is_released
{ r.is_released }
}
fn main() {}
