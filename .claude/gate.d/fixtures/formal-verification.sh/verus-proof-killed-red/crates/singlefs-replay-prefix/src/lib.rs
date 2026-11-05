//! 门禁样本用的小核（照 verus-proof-green）：证明都过，这份样本红在替身 Verus 起来就被 kill -9 上，不在证明上。
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
    if first < second {
        pick(first)
    } else {
        pick(second)
    }
}

} // verus!
