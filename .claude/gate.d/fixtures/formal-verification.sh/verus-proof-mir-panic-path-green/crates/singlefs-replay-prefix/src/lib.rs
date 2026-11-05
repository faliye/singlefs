//! 门禁样本：verus-proof-green 那份小核，verus! 外面另有一个带格式参数的 panic!：产品构建把那一处调用印成 std::rt::panic_fmt，
//! Verus 构建（有 verus! 的展开，经 ::core 用了 core）印成 core::panicking::panic_fmt，是同一个函数；照规整表改写之后 MIR 逐条目相同。
use vstd::prelude::*;

/// 不在 verus! 里的产品函数，只为让 MIR 里有一处 panic_fmt。
pub fn refuse_seven(value: u64) -> u64 {
    if value == 7 {
        panic!("value {} is not allowed", value);
        // 产品构建把上一行印成 std 再导出的路径：core::panicking::panic_fmt(SELFTEST_MIR_RT_PATH)
    }
    value
}

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
