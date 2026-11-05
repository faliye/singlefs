//! 门禁样本：宏在 verus! 外面定义、展开里带 cfg!（第三轮辩方模型 verus/z2_f5_macro_outside.rs）：被验函数的文本里没有 cfg，
//! 产品构建 pick(10, 2) 返回 11，verus --compile 编出返回 12。
use vstd::prelude::*;

macro_rules! plus_span {
    ($start:expr, $span:expr) => {
        if cfg!(verus_keep_ghost) {
            $start + $span
        } else {
            $start + 1
        }
    };
}

verus! {

pub fn pick(start: u64, span: u64) -> (r: u64)
    requires
        start + span <= u64::MAX,
    ensures
        r == start + span,
{
    plus_span!(start, span)
}

} // verus!
