//! 门禁样本：halve_within 的函数体里有一处 assert(…) by (nonlinear_arith) 接 requires … ;（core 的 bytes.rs、unit.rs、journal.rs 里的写法）。
//! 这一处 requires 是那一处 assert 要先证出来的前提，不是 halve_within 的前提，不是空真插入点：列项退 0，空真只有 halve_within 入口那一处，
//! proof 那一行报「跳过 assert-by requires 1 处」并点名 lib.rs 第 18 行；改前当插入点、往下找不到函数体的 {，列项退 1。
use vstd::prelude::*;

verus! {

pub fn halve_within(value: u64, bound: u64) -> (result: u64)
    requires
        bound > 0,
        value < 2 * bound,
    ensures
        result < bound,
{
    let result = value / 2;
    proof {
        assert(result < bound) by (nonlinear_arith)
            requires
                value < 2 * bound,
                result == value / 2,
        ;
    }
    result
}

} // verus!
