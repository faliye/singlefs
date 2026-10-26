use vstd::prelude::*;
use std::collections::BTreeMap;
verus! {
// transaction.rs 的 destination_of_key: BTreeMap<Vec<u8>, [LocationEntry;2]> 与 .get(key).copied() 的形状
pub fn lookup(m: &BTreeMap<u64, u64>, k: u64) -> (r: Option<u64>) {
    m.get(&k).copied()
}
}
fn main() {}
