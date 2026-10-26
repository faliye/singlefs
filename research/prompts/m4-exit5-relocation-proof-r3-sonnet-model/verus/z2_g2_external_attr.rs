// Z2 补：#[verifier::external] 与 admit / assume 在 --no-cheating 下的反应（对照）。
use vstd::prelude::*;
verus! {
#[verifier::external]
pub fn hidden() -> u64 { 0 }
pub fn calls_hidden() -> (r: u64)
    ensures r == 7,
{
    hidden()
}
} // verus!
