//! 门禁样本用的小核：两个带前置条件的地方（一个函数、一个闭包），证明都过。
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

// 一条会让 Z3 一直实例化下去的证明：量词的触发项每用一次就生出两个新的触发项，目标又不在推得出的方向上。
#[verifier::rlimit(infinity)]
pub proof fn reaching_backwards_never_terminates(reached: spec_fn(int) -> bool, start: int)
    requires
        reached(start),
        forall|step: int| #[trigger] reached(step) ==> reached(step + 1) && reached(step + 2),
    ensures
        reached(start - 1),
{
}

} // verus!
