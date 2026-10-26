// Z4 危险二：ghost 变量参与 exec 分支。
use vstd::prelude::*;
verus! {
pub fn branch_on_ghost(x: u64) -> (r: u64)
{
    let ghost g = x > 5;
    if g { 1 } else { 2 }
}
} // verus!
