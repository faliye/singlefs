//! 门禁样本：requires 挂在一个普通的 proof 块后面（前面没有 assert(…) by (…)），往下一行只有 ; 。
//! 它不是 assert-by 的前提，也找不到函数体的 {：列项判红并点名 lib.rs 第 17 行；只凭「下面有一行只有 ;」放行的判法会放过它、列项退 0。
use vstd::prelude::*;

verus! {

pub fn halve_plain(value: u64) -> (result: u64)
    requires
        value > 0,
    ensures
        result <= value,
{
    let result = value / 2;
    proof {
        assert(result <= value);
    }
        requires
            value > 1,
    ;
    result
}

} // verus!
