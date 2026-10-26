//! 门禁样本：external_trait_specification 给 std 的 FromStr 写假规格（第三轮辩方模型 verus/z2_g1_trait_spec.rs）：要登记进信任清单。
use vstd::prelude::*;

verus! {

#[verifier::external_trait_specification]
pub trait ExFromStr: Sized {
    type ExternalTraitSpecificationFor: core::str::FromStr;

    type Err;

    fn from_str(s: &str) -> (r: Result<Self, Self::Err>)
        ensures
            r is Err;
}

pub fn parse_always_fails(s: &str) -> (r: bool)
    ensures
        r,
{
    match <u32 as core::str::FromStr>::from_str(s) {
        Ok(_) => false,
        Err(_) => true,
    }
}

} // verus!
