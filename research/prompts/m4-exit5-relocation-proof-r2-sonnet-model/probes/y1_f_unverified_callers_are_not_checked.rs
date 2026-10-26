// 探针 y1-f：verus! 外面的代码不被 Verus 看见——绕过类型不变量构造一个值、不满足 requires 就调被验函数，Verus 都不拦。
use vstd::prelude::*;

verus! {

pub struct Region {
    start: u64,
}

impl Region {
    #[verifier::type_invariant]
    spec fn well_formed(self) -> bool {
        self.start < 100
    }

    pub closed spec fn spec_start(self) -> u64 {
        self.start
    }

    pub fn make(start: u64) -> (r: Option<Region>) {
        if start < 100 { Some(Region { start }) } else { None }
    }

    pub fn doubled(self) -> (r: u64)
        ensures r == self.spec_start() * 2,
    {
        proof { use_type_invariant(&self); }
        self.start * 2
    }
}

pub fn needs_a_small_argument(x: u64) -> (r: u64)
    requires x < 100,
    ensures r == x + 1,
{
    x + 1
}

} // verus!

// verus! 外面：Verus 不看。
pub fn forge_a_region() -> Region {
    Region { start: u64::MAX }
}

pub fn call_without_the_precondition() -> u64 {
    needs_a_small_argument(u64::MAX)
}

fn main() {}
