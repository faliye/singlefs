use vstd::prelude::*;
use crate::relocation_interval_proofs::*;
verus! {
pub open spec fn storage_before() -> spec_fn(int) -> Seq<u8> {
    |slot: int| if slot == 10 { seq![42u8] } else if slot == 20 { seq![99u8] }
        else if slot == 21 { seq![88u8] } else { seq![0u8] }
}
// 必红：断言搬完之后回退目标 key4 两槽读不变（即假装改后协议护住了它）。它应当红。
pub proof fn mustred_a2_pretend_rollback_read_unchanged()
    ensures
        ({
            let sources = seq![SlotRun { start: 10, span: 1 }];
            let destinations = seq![SlotRun { start: 21, span: 1 }];
            let rollback_run = SlotRun { start: 20, span: 2 };
            let moved = storage_after_relocating_runs(storage_before(), sources, destinations, 1);
            unit_read(moved, rollback_run) == unit_read(storage_before(), rollback_run)
        }),
{
}
}
