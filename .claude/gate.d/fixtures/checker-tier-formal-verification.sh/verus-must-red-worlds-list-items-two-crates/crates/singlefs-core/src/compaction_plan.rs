//! 门禁样本：singlefs-core 里的一个被验区域（cargo verus focus 验；impl 里的方法照全路径成项）。
use vstd::prelude::*;

verus! {

pub struct CompactionRegion {
    pub start_slot: u64,
}

impl CompactionRegion {
    pub fn contains_slot(&self, slot: u64) -> (r: bool)
        requires
            self.start_slot < 1000,
        ensures
            r == (self.start_slot <= slot && slot < self.start_slot + 64),
    {
        self.start_slot <= slot && slot < self.start_slot + 64
    }
}

pub const fn holes_admit_the_issue(holes: u64, needed: u64) -> (r: bool)
    ensures
        r == (needed <= holes),
{
    needed <= holes
}

} // verus!
