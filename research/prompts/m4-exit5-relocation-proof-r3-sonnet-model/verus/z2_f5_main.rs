use vstd::prelude::*;
macro_rules! plus_span {
    ($start:expr, $span:expr) => { if cfg!(verus_keep_ghost) { $start + $span } else { $start + 1 } };
}
verus! {
pub fn pick(start: u64, span: u64) -> (r: u64)
    requires start + span <= u64::MAX,
    ensures r == start + span,
{
    plus_span!(start, span)
}
} // verus!
fn main() {
    let r = pick(10, 2);
    println!("name=verus-compiled-run pick={r}");
}
