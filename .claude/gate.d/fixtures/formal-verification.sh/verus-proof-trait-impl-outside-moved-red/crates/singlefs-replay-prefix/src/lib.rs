//! 门禁样本：放过表登记的 impl Device for BadDevice 已经挪进 verus! 里（照 trait 的 ensures 证了），表里那一行该删，判红。
use vstd::prelude::*;

verus! {

pub trait Device {
    fn read_slot(&self, slot: u64) -> (r: u64)
        ensures
            r <= 10;
}

pub fn read_checked<D: Device>(d: &D, slot: u64) -> (r: u64)
    ensures
        r <= 10,
{
    d.read_slot(slot)
}

pub struct BadDevice;

impl Device for BadDevice {
    fn read_slot(&self, _slot: u64) -> (r: u64) {
        3
    }
}

} // verus!
