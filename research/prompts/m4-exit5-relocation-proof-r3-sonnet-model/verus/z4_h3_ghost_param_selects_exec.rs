// Z4 危险三：Ghost<T> 参数的值带进 exec 返回值。
use vstd::prelude::*;
verus! {
pub fn leaks_ghost(x: u64, g: Ghost<u64>) -> (r: u64)
{
    let y: u64 = g@;
    y + x
}
} // verus!
