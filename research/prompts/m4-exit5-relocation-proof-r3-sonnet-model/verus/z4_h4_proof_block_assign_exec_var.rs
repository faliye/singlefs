// Z4 危险四：proof 块里给 exec 变量赋值。
use vstd::prelude::*;
verus! {
pub fn assigns_in_proof(x: u64) -> (r: u64)
{
    let mut y: u64 = x;
    proof {
        y = 7;
    }
    y
}
} // verus!
