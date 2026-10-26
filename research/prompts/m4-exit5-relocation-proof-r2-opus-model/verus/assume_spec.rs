use vstd::prelude::*;
verus! {
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
}
