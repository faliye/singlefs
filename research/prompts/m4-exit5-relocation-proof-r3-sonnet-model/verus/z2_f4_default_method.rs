// Z2 写法五：trait 默认方法带 ensures，verus! 里的 impl 覆盖它而违 ensures。
use vstd::prelude::*;
verus! {
pub trait Pick {
    fn pick(&self) -> (r: u64)
        ensures r <= 10,
    { 5 }
}
pub struct A;
impl Pick for A {
    fn pick(&self) -> (r: u64) { 99 }
}
} // verus!
