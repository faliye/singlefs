// 探针 y1-b：不用 verus! 宏，用 #[verus_verify] / #[verus_spec] 属性（Rust 语法模式），被验函数仍是普通 Rust 写法。
use vstd::prelude::*;
use verus_builtin_macros::*;

#[verus_verify]
#[derive(Clone, Copy)]
pub struct Region { pub start: u64, pub count: u64 }

#[verus_verify]
impl Region {
    #[verus_spec(r => ensures r == (slot >= self.start && slot - self.start < self.count))]
    pub fn contains_slot(&self, slot: u64) -> bool { slot >= self.start && slot - self.start < self.count }
}
fn main() {}
