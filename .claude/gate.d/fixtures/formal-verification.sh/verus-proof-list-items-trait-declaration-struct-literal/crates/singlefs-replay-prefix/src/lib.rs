//! 门禁样本：trait 里没有函数体的声明 Store::put，ensures 里有结构体字面量（Request { slot: slot }）。按切出的单元认（以 ; 收尾）不成项，
//! 成项的是 Shelf::put 与 clamp_to_limit；按「单元里有没有 {」认的改前判法会把 Store::put 也列成一项。
use vstd::prelude::*;

verus! {

pub struct Request {
    pub slot: u64,
}

pub open spec fn issued(request: Request) -> bool {
    request.slot < 10
}

pub trait Store {
    fn put(&mut self, slot: u64) -> (r: bool)
        ensures
            r == issued(Request { slot: slot }),
    ;
}

pub struct Shelf;

impl Store for Shelf {
    fn put(&mut self, slot: u64) -> (r: bool) {
        slot < 10
    }
}

pub fn clamp_to_limit(value: u64, limit: u64) -> (result: u64)
    requires
        limit > 0,
    ensures
        result <= limit,
{
    if value > limit {
        limit
    } else {
        value
    }
}

} // verus!
