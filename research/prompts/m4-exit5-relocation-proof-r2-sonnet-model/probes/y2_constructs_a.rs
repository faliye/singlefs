// 探针 y2-a：core 里最常见的 std 写法，各取一个小函数，直接起 verus --no-cheating。出错的函数 = vstd 罩不到或 Verus 不支持。
use vstd::prelude::*;
verus! {

// 01 ? 运算符（Result，错误类型相同）
pub fn q01_question_mark(a: Result<u64, u8>) -> (r: Result<u64, u8>)
    ensures r == a
{
    let x = a?;
    Ok(x)
}

// 02 Vec 下标与长度
pub fn q02_vec_index(v: &Vec<u8>, i: usize) -> (r: u8)
    requires i < v.len()
    ensures r == v@[i as int]
{
    v[i]
}

// 03 切片 for 循环
pub fn q03_for_over_slice_iter(v: &[u64]) -> (s: bool)
{
    let mut seen_big = false;
    for x in iter: v.iter()
        invariant true,
    {
        if *x > 3 { seen_big = true; }
    }
    seen_big
}

// 04 Range for 循环
pub fn q04_for_range(n: u64) -> (s: u64)
    requires n < 1000
{
    let mut c: u64 = 0;
    for i in 0..n
        invariant c == i, n < 1000,
    {
        c = c + 1;
    }
    c
}

// 05 Vec push / new / len / is_empty / with_capacity
pub fn q05_vec_push() -> (v: Vec<u8>)
    ensures v@.len() == 2
{
    let mut v: Vec<u8> = Vec::with_capacity(2);
    v.push(1);
    v.push(2);
    v
}

// 06 Option 组合子
pub fn q06_option_map(o: Option<u64>) -> (r: Option<u64>)
    ensures r == match o { Some(x) => if x < 100 { Some((x + 1) as u64) } else { Some(x) }, None => None }
{
    o.map(|x: u64| if x < 100 { x + 1 } else { x })
}

pub fn q06_option_ok_or(o: Option<u64>) -> (r: Result<u64, u8>)
    ensures r == match o { Some(x) => Ok::<u64, u8>(x), None => Err::<u64, u8>(7u8) }
{
    o.ok_or(7u8)
}

pub fn q06_option_copied(o: Option<&u64>) -> (r: Option<u64>)
    ensures r == match o { Some(x) => Some(*x), None => None }
{
    o.copied()
}

pub fn q06_option_is_some_and(o: Option<u64>) -> (r: bool)
{
    o.is_some_and(|x: u64| x > 3)
}

pub fn q06_option_map_or(o: Option<u64>) -> (r: u64)
{
    o.map_or(0, |x: u64| x)
}

pub fn q06_option_expect(o: Option<u64>) -> (r: u64)
    requires o.is_some()
    ensures r == o.unwrap()
{
    o.expect("有值")
}

// 07 iter().map().collect()
pub fn q07_map_collect(v: &Vec<u64>) -> (r: Vec<u64>)
{
    v.iter().map(|x: &u64| *x).collect::<Vec<u64>>()
}

// 08 iter().any / all / find
pub fn q08_any(v: &Vec<u64>) -> (r: bool)
{
    v.iter().any(|x: &u64| *x > 3)
}

// 09 整数转换与受检算术
pub fn q09_try_from(x: u64) -> (r: Result<usize, core::num::TryFromIntError>)
{
    usize::try_from(x)
}

pub fn q09_checked_add(a: u64, b: u64) -> (r: Option<u64>)
{
    a.checked_add(b)
}

pub fn q09_div_ceil(a: u64, b: u64) -> (r: u64)
    requires b > 0
{
    a.div_ceil(b)
}

pub fn q09_from(a: u32) -> (r: u64)
    ensures r == a as u64
{
    u64::from(a)
}

// 10 BTreeMap
pub fn q10_btree_get(m: &std::collections::BTreeMap<u64, u64>, k: u64) -> (r: Option<&u64>)
{
    m.get(&k)
}

// 11 panic! / assert! 带消息
pub fn q11_assert_msg(x: u64)
    requires x > 3
{
    assert!(x > 3, "x 必须大于 3");
}

// 12 Vec<u8> 克隆与相等
pub fn q12_vec_clone_eq(v: &Vec<u8>) -> (r: bool)
    ensures r
{
    let w = v.clone();
    w == *v
}

// 13 字节编解码
pub fn q13_le_bytes(x: u64) -> (r: [u8; 8])
{
    x.to_le_bytes()
}

pub fn q13_from_le_bytes(b: [u8; 8]) -> (r: u64)
{
    u64::from_le_bytes(b)
}

// 14 extend_from_slice / to_vec
pub fn q14_extend(v: &mut Vec<u8>, w: &[u8])
{
    v.extend_from_slice(w);
}

pub fn q14_to_vec(w: &[u8]) -> (r: Vec<u8>)
    ensures r@ == w@
{
    w.to_vec()
}

}
fn main() {}
