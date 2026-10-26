// Z4 危险一：proof 块里调一个有副作用的 exec 函数。
use vstd::prelude::*;
verus! {
pub fn bump(counter: &mut u64)
    requires *old(counter) < 100,
    ensures *final(counter) == *old(counter) + 1,
{
    *counter = *counter + 1;
}
pub fn uses_it(counter: &mut u64)
    requires *old(counter) < 100,
{
    proof {
        bump(counter);
    }
}
} // verus!
