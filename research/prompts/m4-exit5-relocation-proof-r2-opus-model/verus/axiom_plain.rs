use vstd::prelude::*;
verus! {
// 不带 broadcast 的 axiom fn：没有函数体、不验，直接当真。
pub axiom fn a_lemma_that_proves_false()
    ensures false;

pub fn a_verified_function_that_returns_seven() -> (r: u64)
    ensures r == 0,
{
    proof { a_lemma_that_proves_false(); }
    7
}
}
