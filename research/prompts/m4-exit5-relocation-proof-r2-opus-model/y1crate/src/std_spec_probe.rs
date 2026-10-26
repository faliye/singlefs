// 攻方原型（m4-exit5-relocation-proof-r2）Y2：给 std 的函数写一条看着对、与 std 文档不符的规格。
// assume_specification 被 --no-cheating 拒，这一份只能不带 --no-cheating 验。
use vstd::prelude::*;

verus! {

// Y2：给 std 的 binary_search 写一条看着对、其实与 std 文档不符的规格（std 说有几个相等元素时返回其中任一个）。
pub assume_specification<T: Ord>[ <[T]>::binary_search ](s: &[T], x: &T) -> (r: Result<usize, usize>)
    ensures
        r matches Ok(i) ==> i < s.len() && s[i as int] == *x
            && forall|j: int| 0 <= j < i ==> s[j] != *x;

pub fn first_index_of(s: &[u64], x: u64) -> (r: Option<usize>)
    ensures
        r matches Some(i) ==> i < s.len() && s[i as int] == x && forall|j: int| 0 <= j < i ==> s[j] != x,
{
    match s.binary_search(&x) {
        Ok(i) => Some(i),
        Err(_) => None,
    }
}

} // verus!
