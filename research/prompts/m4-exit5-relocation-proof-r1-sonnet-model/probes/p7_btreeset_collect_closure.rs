use vstd::prelude::*;
use std::collections::BTreeSet;
verus! {
// transaction.rs:10074 的形状：iter().map(|r| r.mapping_key.as_slice()).collect::<BTreeSet<&[u8]>>()
pub fn keys(v: &Vec<Vec<u8>>) -> BTreeSet<&[u8]> {
    v.iter().map(|x| x.as_slice()).collect()
}
}
fn main() {}
