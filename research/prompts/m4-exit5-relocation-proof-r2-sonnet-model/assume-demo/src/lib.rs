//! 登记式 assume_specification 的演示：规格谓词只写一次（泛型版进 ensures），
//! 同一条谓词的可执行孪生（Verus 证它与规格谓词相等），差分测试在真实 std 上逐值核。
use vstd::prelude::*;

verus! {

// 规格谓词（泛型）：输入是 Option<&T> 解引用后的值，输出是 copied 的结果。
pub open spec fn copied_post<T>(input: Option<T>, output: Option<T>) -> bool {
    match input {
        Some(x) => output == Some(x),
        None => output == None::<T>,
    }
}

// 规格谓词的可执行孪生：Verus 证它与规格谓词相等，差分测试拿它去核真实 std。
// （试过 exec_spec_verified! 自动编这一个：编出的函数在 Some(0) 这一格上返回假，与规格谓词对不上，原因没追；手写的孪生由 Verus 证相等，不靠宏。）
pub fn copied_post_exec(input: Option<u64>, output: Option<u64>) -> (b: bool)
    ensures
        b == copied_post::<u64>(input, output),
{
    match input {
        Some(x) => match output {
            Some(y) => x == y,
            None => false,
        },
        None => output.is_none(),
    }
}

// 被登记的规格：对 std 的 Option::copied（签名必须与 std 完全一致，Verus 自己核这一点）。
pub assume_specification<T: Copy>[ Option::<&T>::copied ](self_: Option<&T>) -> (r: Option<T>)
    ensures
        copied_post(
            match self_ {
                Some(x) => Some(*x),
                None => None,
            },
            r,
        ),
;

// 用上这条规格的一个函数：Verus 靠它证出 ensures。
pub fn first_or_none(v: &Vec<u64>) -> (r: Option<u64>)
    ensures
        r == if v@.len() > 0 { Some(v@[0]) } else { None::<u64> },
{
    v.first().copied()
}

// CLAIMS-MARKER

} // verus!

#[cfg(test)]
mod differential {
    use super::*;

    // 差分：真实 std 的 copied 在有界域上逐值过 copied_post_exec（Verus 证了它等于规格谓词）。
    #[test]
    fn the_registered_spec_agrees_with_std_on_the_bounded_domain() {
        let domain: Vec<Option<u64>> = vec![None, Some(0), Some(1), Some(u64::MAX), Some(1 << 40)];
        for input in &domain {
            let reference: Option<&u64> = input.as_ref();
            let output: Option<u64> = reference.copied();
            assert!(copied_post_exec(*input, output), "规格与 std 不符：输入 {input:?} 输出 {output:?}");
        }
    }
}
