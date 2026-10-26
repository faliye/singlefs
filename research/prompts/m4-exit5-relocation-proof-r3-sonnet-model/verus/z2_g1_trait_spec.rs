// Z2 补：external_trait_specification 给一个 std 特质方法写假规格（from_str 恒返回 Err），--no-cheating 拦不拦。
use vstd::prelude::*;
verus! {
#[verifier::external_trait_specification]
pub trait ExFromStr: Sized {
    type ExternalTraitSpecificationFor: core::str::FromStr;
    type Err;
    fn from_str(s: &str) -> (r: Result<Self, Self::Err>)
        ensures r is Err;
}
pub fn parse_always_fails(s: &str) -> (r: bool)
    ensures r,
{
    match <u32 as core::str::FromStr>::from_str(s) {
        Ok(_) => false,
        Err(_) => true,
    }
}
} // verus!
