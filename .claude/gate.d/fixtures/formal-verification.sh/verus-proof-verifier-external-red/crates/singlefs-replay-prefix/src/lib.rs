//! 门禁样本：#[verifier::external] 让条目对 Verus 不可见（第三轮辩方模型 verus/z2_g2_external_attr.rs）：核里一律不许。
use vstd::prelude::*;

verus! {

#[verifier::external]
pub fn hidden() -> u64 {
    0
}

pub fn calls_hidden() -> (r: u64)
    ensures
        r == 7,
{
    7
}

} // verus!
