//! 门禁样本用的小核：一个带前置条件的函数，一条 assume_specification 指到路径依赖 singlefs-format 里的常量（core 指格式常量的样子）。
use vstd::prelude::*;

verus! {

pub fn clamp_to_limit(value: u64, limit: u64) -> (result: u64)
    requires
        limit > 0,
    ensures
        result <= limit,
{
    if value > limit {
        limit
    } else {
        value
    }
}

#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::SLOT_BYTES ] -> (slot_bytes: u64) returns 4096u64;

} // verus!
