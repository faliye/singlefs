//! 门禁样本：trait 的 ensures 只在 verus! 里的 impl 上核，verus! 外面的 impl 没人核（第三轮辩方模型 verus/z2_f2_trait_impl_outside.rs；
//! 产品构建 read_checked(&BadDevice, 0) 返回 99）。cfg(test) 下的 impl 不进产品构建，不判。
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
