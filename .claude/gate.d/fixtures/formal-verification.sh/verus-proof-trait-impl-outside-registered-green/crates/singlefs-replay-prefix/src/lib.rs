//! 门禁样本：verus! 外面的 impl Device for BadDevice 登记在放过表里（.claude/gate.d/verus-trait-impls-outside-lag.tsv），这一格放过、
//! 成功句点名它「契约没证、靠信任」；核与 verus-proof-trait-impl-outside-red 相同。cfg(test) 下的 impl 不进产品构建，不判。
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

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDevice;

    impl Device for MockDevice {
        fn read_slot(&self, _slot: u64) -> u64 {
            1
        }
    }
}
