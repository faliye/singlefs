//! 门禁样本：trait 里带默认值的 Limited::LIMIT 与 impl 里覆盖它的 Ordinal 的 LIMIT 同在一个模块。impl 里那一项交给 --verify-function 的 *::LIMIT
//! 在可点名函数表里命中 Limited::LIMIT 与 impl&%?::LIMIT 两项（真 Verus 两个一起验），列项当场判红并点名这两项；
//! 改前的判法只比成项的几项谁是谁的一段，放过它、列项退 0。
use vstd::prelude::*;

verus! {

pub struct Ordinal(pub u32);

pub trait Limited {
    const LIMIT: u64 = 3;
}

impl Limited for Ordinal {
    const LIMIT: u64 = 4;
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
