// Z2 写法三：trait 的 ensures 只在 verus! 里的 impl 上核；verus! 外面的 impl 没人核，泛型函数照 trait 的 ensures 证过。
use vstd::prelude::*;
verus! {
pub trait Device {
    fn read_slot(&self, slot: u64) -> (r: u64)
        ensures r <= 10;
}
pub fn read_checked<D: Device>(d: &D, slot: u64) -> (r: u64)
    ensures r <= 10,
{
    d.read_slot(slot)
}
} // verus!
pub struct BadDevice;
impl Device for BadDevice {
    fn read_slot(&self, _slot: u64) -> u64 { 99 }
}
