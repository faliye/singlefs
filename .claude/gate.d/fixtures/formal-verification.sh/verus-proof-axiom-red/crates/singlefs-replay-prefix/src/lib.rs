//! 门禁样本：axiom fn（第二轮攻方模型 verus/axiom_plain.rs）。不带 --no-cheating 之后 Verus 不拒它，门禁一律判红。
use vstd::prelude::*;

verus! {

// 不带 broadcast 的 axiom fn：没有函数体、不验，直接当真。
pub axiom fn a_lemma_that_proves_false()
    ensures
        false;

pub fn a_verified_function_that_returns_seven() -> (r: u64)
    ensures
        r == 0,
{
    proof {
        a_lemma_that_proves_false();
    }
    7
}

} // verus!
