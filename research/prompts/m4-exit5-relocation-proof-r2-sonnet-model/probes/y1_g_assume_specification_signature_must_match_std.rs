// 探针 y1-g：assume_specification 的签名必须与 std 的完全一致（泛型也要）；把 T 钉成 u64 写一条，Verus 自己拒。
use vstd::prelude::*;
verus! {
pub assume_specification[ Option::<&u64>::copied ](self_: Option<&u64>) -> (r: Option<u64>)
    ensures
        r.is_none(),
;
}
fn main() {}
