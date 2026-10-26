use vstd::prelude::*;
use crate::gc_ring_proofs::*;
verus! {
// 必红：断言根环版拒绝往第二槽 21 写（即假装 gap 不存在）。它应当红。
pub proof fn mustred_a1_pretend_second_slot_is_protected()
    ensures
        ({
            let b7 = seq![42u8];
            let b4 = seq![99u8];
            let head = RootView { mapping: Map::<u64, u64>::empty().insert(7u64, 10u64),
                logical: Map::<u64, Seq<u8>>::empty().insert(7u64, b7) };
            let candidate = RootView { mapping: Map::<u64, u64>::empty().insert(4u64, 20u64),
                logical: Map::<u64, Seq<u8>>::empty().insert(4u64, b4) };
            let s = GcRingState {
                disk: Map::<u64, Seq<u8>>::empty().insert(10u64, b7).insert(20u64, b4).insert(21u64, seq![7u8]),
                roots: seq![candidate], floor: 0, head, readers: Seq::empty(), window: 2 };
            !ring_step_enabled(s, RingStep::Write { slot: 21u64, bytes: seq![0u8] })
        }),
{
}
}
