use vstd::prelude::*;
verus! {
// transaction.rs:10243 的形状：usize::try_from(planned.source.span * SLOT_BYTES).expect(..)
pub fn unit_bytes(span: u64) -> (r: usize) {
    usize::try_from(span * 4096).expect("两槽以内")
}
}
fn main() {}
