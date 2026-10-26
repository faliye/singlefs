//! 第三轮辩方腿：第二轮正推的区间版 P4（relocation_interval_proofs）单独看，放不放过变体一；
//! 提升到区间的根环协议（gc_ring_runs）放不放过。两个模块里各有一个 SlotRun，按全名区分。
use vstd::prelude::*;

verus! {

// 现行映射只有 key 7 在 [10,11)；搬迁计划把它搬到 [20,21)，而 [20,21) 是回退候选里 key 4 的槽。
pub proof fn the_interval_premise_alone_does_not_see_the_rollback_candidate()
    ensures
        ({
            let mapping = Map::<u64, crate::relocation_interval_proofs::SlotRun>::empty()
                .insert(7u64, crate::relocation_interval_proofs::SlotRun { start: 10, span: 1 });
            crate::relocation_interval_proofs::relocation_plan_is_well_formed(
                mapping, seq![7u64],
                seq![crate::relocation_interval_proofs::SlotRun { start: 10, span: 1 }],
                seq![crate::relocation_interval_proofs::SlotRun { start: 20, span: 1 }], 1)
        }),
{
    use crate::relocation_interval_proofs::*;
    let mapping = Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 });
    let keys = seq![7u64];
    let sources = seq![SlotRun { start: 10, span: 1 }];
    let destinations = seq![SlotRun { start: 20, span: 1 }];
    assert(mapping.contains_key(7u64) && mapping[7u64] == sources[0]);
    assert forall|key: u64| mapping.contains_key(key) implies #[trigger] mapping[key].span >= 1 by { assert(key == 7u64); }
    assert forall|index: int, key: u64| 0 <= index < 1 && #[trigger] mapping.contains_key(key) implies !runs_overlap(
        #[trigger] destinations[index], mapping[key]) by { assert(key == 7u64); assert(index == 0); }
    assert(destination_runs_are_disjoint_and_free(mapping, destinations, 1));
    assert(relocation_plan_is_well_formed(mapping, keys, sources, destinations, 1));
}

} // verus!
