//! 门禁样本：放过表只登记了 verus! 外面的 impl Device for BadDevice，同一个 trait 在外面另有一个 impl Device for WorseDevice 没登记，判红。
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

} // verus!

pub struct BadDevice;

impl Device for BadDevice {
    fn read_slot(&self, _slot: u64) -> u64 {
        99
    }
}

pub struct WorseDevice;

impl Device for WorseDevice {
    fn read_slot(&self, _slot: u64) -> u64 {
        98
    }
}
