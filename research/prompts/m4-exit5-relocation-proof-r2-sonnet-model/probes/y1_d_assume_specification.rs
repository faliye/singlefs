// 探针 y1-d：assume_specification 给 std 的 Option::copied 写规格，--no-cheating 下过不过。
use vstd::prelude::*;
verus! {
pub assume_specification<T: Copy>[ Option::<&T>::copied ](self_: Option<&T>) -> (r: Option<T>)
    ensures r == match self_ { Some(x) => Some(*x), None => None };
pub fn first_copied(v: &Option<&u64>) -> (r: Option<u64>)
    ensures r == match *v { Some(x) => Some(*x), None => None }
{ (*v).copied() }
}
fn main() {}
