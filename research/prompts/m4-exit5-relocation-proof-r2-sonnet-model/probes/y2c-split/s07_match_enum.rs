use vstd::prelude::*;
verus! {
pub enum Verdict { Accepted(u64), Refused(u8) }
pub fn s07_match_enum(v: &Verdict) -> (r: u64)
    ensures r == match v { Verdict::Accepted(x) => *x, Verdict::Refused(_) => 0 }
{
    match v {
        Verdict::Accepted(x) => *x,
        Verdict::Refused(_) => 0,
    }
}
}
fn main() {}
