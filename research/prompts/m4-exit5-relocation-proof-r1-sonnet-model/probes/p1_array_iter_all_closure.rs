use vstd::prelude::*;
verus! {
pub struct Loc { pub slot: u64, pub checksum: u32 }
// transaction.rs:10222-10227 的形状：locations.iter().all(|location| location.slot == planned.source.slot)
pub fn both_at(locations: [Loc; 2], planned_slot: u64) -> (r: bool)
    ensures r == (locations[0].slot == planned_slot && locations[1].slot == planned_slot),
{
    locations.iter().all(|location| location.slot == planned_slot)
}
}
fn main() {}
