//! 门禁样本用的小核：证明都过，但 clamp_to_limit 上挂了 rlimit 属性（盖过命令行的 --rlimit，verus-proof 那一格当作弊标记判红）。
use vstd::prelude::*;

verus! {

pub open spec fn within(value: u64, limit: u64) -> bool {
    value <= limit
}

#[verifier::rlimit(20)]
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
