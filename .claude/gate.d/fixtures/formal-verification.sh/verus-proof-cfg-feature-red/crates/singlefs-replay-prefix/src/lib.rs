//! 门禁样本：按 feature 原子分叉的函数体（第三轮辩方模型 verus/z2_f1_feature_fork.rs）。Verus 按这一趟的 cfg 集合验，产品按别的 cfg 集合编。
use vstd::prelude::*;

verus! {

#[cfg(feature = "wide")]
pub fn pick(start: u64, span: u64) -> (r: u64)
    requires
        start + span <= u64::MAX,
    ensures
        r == start + span,
{
    start + 1
}

#[cfg(not(feature = "wide"))]
pub fn pick(start: u64, span: u64) -> (r: u64)
    requires
        start + span <= u64::MAX,
    ensures
        r == start + span,
{
    start + span
}

} // verus!
