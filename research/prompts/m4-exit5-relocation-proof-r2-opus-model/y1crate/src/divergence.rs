// 攻方原型（m4-exit5-relocation-proof-r2）Y1：Verus 验的与产品 cargo build 编的不是同一份函数体。
// 两种写法都过 --no-cheating，也都不含 verus-proof-check.py 的 CHEATING_MARKER 里的任何一个词。
use vstd::prelude::*;

verus! {

// 写法一：同名两份，cfg 分开。Verus 编的时候 verus_keep_ghost 为真，产品构建为假。
#[cfg(verus_keep_ghost)]
pub fn destination_after(start: u64, span: u64) -> (r: u64)
    requires start + span <= u64::MAX,
    ensures r == start + span,
{
    start + span
}

#[cfg(not(verus_keep_ghost))]
pub fn destination_after(start: u64, span: u64) -> (r: u64)
    requires start + span <= u64::MAX,
    ensures r == start + span,
{
    start + 1
}

// 写法二：同一份函数体里用 cfg! 分支。
pub fn destination_after_by_the_cfg_macro(start: u64, span: u64) -> (r: u64)
    requires start + span <= u64::MAX,
    ensures r == start + span,
{
    if cfg!(verus_keep_ghost) { start + span } else { start + 1 }
}

} // verus!
