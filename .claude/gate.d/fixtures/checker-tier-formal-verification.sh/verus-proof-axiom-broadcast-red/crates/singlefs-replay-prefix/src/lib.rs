//! 门禁样本：broadcast axiom（第二轮攻方模型 verus/axiom_broadcast.rs）。
use vstd::prelude::*;

verus! {

pub uninterp spec fn slot_is_free(slot: u64) -> bool;

// broadcast axiom：broadcast use 之后对每个触发点都当真。
pub broadcast axiom fn axiom_every_slot_is_free(slot: u64)
    ensures
        #[trigger] slot_is_free(slot);

pub fn a_slot_that_is_free(slot: u64) -> (r: u64)
    ensures
        slot_is_free(r),
{
    broadcast use axiom_every_slot_is_free;
    slot
}

} // verus!
