//! 门禁样本：std 函数的 assume_specification（第二轮攻方模型 verus/assume_spec.rs 的写法，binary_search 的规格写成「第一个下标」）。
use vstd::prelude::*;

verus! {

pub assume_specification<T: Ord>[ <[T]>::binary_search ](s: &[T], x: &T) -> (r: Result<usize, usize>)
    ensures
        r matches Ok(i) ==> i < s.len() && s[i as int] == *x && forall|j: int| 0 <= j < i ==> s[j] != *x;

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
