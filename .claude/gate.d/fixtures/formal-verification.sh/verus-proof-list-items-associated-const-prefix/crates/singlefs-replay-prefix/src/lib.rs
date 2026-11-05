//! 门禁样本：关联 const Ordinal::FIRST 是自由 const FIRST_COUNTER 的前一段，自由 const LAST 是关联 const Ordinal::LAST_SLOT 的前一段
//! （core 的 JournalRecordOrdinalWithinPublish::FIRST 与 FIRST_JOURNAL_COUNTER 同在 journal 那个模块的样子）。
//! 关联 const 交给 --verify-function 的是 *::<名>、自由 const 是 <名>，在可点名函数表里各恰好命中一项，列项退 0；
//! 改前关联 const 交 <名>，FIRST 在表里命中 FIRST_COUNTER 与 impl&%?::FIRST 两项，列项退 1。
use vstd::prelude::*;

verus! {

pub struct Ordinal(pub u32);

impl Ordinal {
    pub const FIRST: Self = Self(1);

    pub const LAST_SLOT: u64 = 9;
}

pub const FIRST_COUNTER: u64 = 1;

pub const LAST: u64 = 2;

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
