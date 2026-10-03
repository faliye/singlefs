//! 门禁样本用的小核：外层函数的前置条件可满足，它里面闭包 never 的前置条件互相矛盾（空真）；证明都过。
//! 几处一起插 assert(false) 时外层入口那一处先失败，闭包那一处不单报；空真格要一处一份副本才认得出它。
use vstd::prelude::*;

verus! {

pub open spec fn within(value: u64, limit: u64) -> bool {
    value <= limit
}

pub fn clamp_to_limit(value: u64, limit: u64) -> (result: u64)
    requires
        limit > 0,
    ensures
        within(result, limit),
        value <= limit ==> result == value,
{
    if value > limit {
        limit
    } else {
        value
    }
}

pub fn smaller_of_two(first: u64, second: u64) -> (result: u64)
    requires
        first > 0,
    ensures
        result <= first,
        result <= second,
{
    let pick = |candidate: u64| -> (picked: u64)
        requires
            candidate <= first,
            candidate <= second,
        ensures
            picked == candidate,
        { candidate };
    let _never = |candidate: u64| -> (picked: u64)
        requires
            candidate < first,
            candidate > first,
        ensures
            picked == 0,
        { candidate };
    if first < second {
        pick(first)
    } else {
        pick(second)
    }
}

} // verus!
