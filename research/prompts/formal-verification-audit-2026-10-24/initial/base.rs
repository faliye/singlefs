use vstd::prelude::*;
verus! {
#[verifier::opaque]
spec fn impossible() -> bool { false }
proof fn vacuous()
    requires
        impossible(),
    ensures
        false,
{
    reveal(impossible);
}
}
