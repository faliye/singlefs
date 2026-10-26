// Z2 写法四：verus! 里的 unsafe。
use vstd::prelude::*;
verus! {
pub fn one() -> (r: u64)
    ensures r == 1,
{
    unsafe { 1 }
}
} // verus!
