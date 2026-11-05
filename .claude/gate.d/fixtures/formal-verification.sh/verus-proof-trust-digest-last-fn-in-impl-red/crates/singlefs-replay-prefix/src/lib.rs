//! 门禁样本：impl 里唯一（也是最后）的 fn 带 external_body，信任清单照改前的取法登的是空串的 sha256（e3b0c442…）。
//! 规格原文取签名到函数体的 { 之前，现算的不是空串的，判红；改前的取法取到 impl 的 {、规格原文是空串，这一行对得上、判绿。
use vstd::prelude::*;

verus! {

pub struct Shown;

impl Shown {
    #[verifier::external_body]
    pub fn show(&self, x: u64) -> (shown: u64)
        ensures
            shown == x,
    {
        x
    }
}

pub fn shown_once(x: u64) -> (r: u64)
    ensures
        r == x,
{
    let s = Shown;
    s.show(x)
}

} // verus!
