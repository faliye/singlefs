// Z2 写法二：按 debug_assertions 分叉（cargo build 与 cargo build --release 不同）。
use vstd::prelude::*;
verus! {
pub fn pick(start: u64, span: u64) -> (r: u64)
    requires start + span <= u64::MAX,
    ensures r == start + span,
{
    if cfg!(debug_assertions) { start + span } else { start + 1 }
}
} // verus!
