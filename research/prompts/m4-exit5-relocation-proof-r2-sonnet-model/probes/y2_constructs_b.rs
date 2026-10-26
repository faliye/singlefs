// 探针 y2-b：a 批里「没写后置条件所以过」的那几种，加上后置条件再试；再加几种 core 里常见的形状。
use vstd::prelude::*;
verus! {

pub fn r01_map_collect_len(v: &Vec<u64>) -> (r: Vec<u64>)
    ensures r@.len() == v@.len()
{
    v.iter().map(|x: &u64| *x).collect::<Vec<u64>>()
}

pub fn r02_any_spec(v: &Vec<u64>) -> (r: bool)
    ensures r == exists|i: int| 0 <= i < v@.len() && v@[i] > 3
{
    v.iter().any(|x: &u64| *x > 3)
}

pub fn r03_filter_count(v: &Vec<u64>) -> (r: usize)
    ensures r <= v@.len()
{
    v.iter().filter(|x: &&u64| **x > 3).count()
}

pub fn r04_find_map(v: &Vec<u64>) -> (r: Option<u64>)
{
    v.iter().find_map(|x: &u64| if *x > 3 { Some(*x) } else { None })
}

pub fn r05_question_with_from(a: Result<u64, u8>) -> (r: Result<u64, u32>)
{
    let x = a?;
    Ok(x)
}

pub fn r06_for_over_vec_by_value(v: Vec<u64>) -> (c: usize)
{
    let mut c = 0usize;
    for x in v
        invariant c <= 1,
    {
        if c < 1 { c = c + 1; }
    }
    c
}

pub enum Verdict { Accepted(u64), Refused(u8) }
pub fn r07_match_enum(v: &Verdict) -> (r: u64)
    ensures r == match v { Verdict::Accepted(x) => *x, Verdict::Refused(_) => 0 }
{
    match v {
        Verdict::Accepted(x) => *x,
        Verdict::Refused(_) => 0,
    }
}

pub fn r08_let_else(o: Option<u64>) -> (r: u64)
    ensures r == match o { Some(x) => x, None => 0 }
{
    let Some(x) = o else { return 0; };
    x
}

pub fn r09_slice_len_and_split(v: &[u8]) -> (r: usize)
    requires v@.len() >= 4
    ensures r == v@.len() - 4
{
    let (_head, tail) = v.split_at(4);
    tail.len()
}

pub fn r10_closure_param<F: Fn(u64) -> bool>(f: F, x: u64) -> (r: bool)
    requires f.requires((x,))
    ensures r == f.ensures((x,), r)
{
    f(x)
}

pub trait Device {
    fn read(&self, offset: u64, length: usize) -> (r: Option<Vec<u8>>);
}
pub fn r11_generic_device<D: Device>(d: &D) -> (r: usize)
{
    match d.read(0, 4) {
        Some(v) => v.len(),
        None => 0,
    }
}

pub fn r12_dyn_trait(d: &dyn Device) -> (r: usize)
{
    match d.read(0, 4) {
        Some(v) => v.len(),
        None => 0,
    }
}

pub fn r13_slice_copy(dst: &mut [u8], src: &[u8])
    requires old(dst)@.len() == src@.len()
{
    dst.copy_from_slice(src);
}

pub fn r14_sort(v: &mut Vec<u64>)
{
    v.sort();
}

pub fn r15_btree_insert_iter(m: &mut std::collections::BTreeMap<u64, Vec<u8>>, k: u64)
{
    m.insert(k, Vec::new());
    for (kk, vv) in m.iter() {
        let _ = (kk, vv);
    }
}

pub fn r16_string_format(x: u64) -> (r: String)
{
    format!("x={}", x)
}

pub fn r17_u8_array_index(a: &[u8; 16], i: usize) -> (r: u8)
    requires i < 16
{
    a[i]
}

pub fn r18_wrapping_ops(a: u32, b: u32) -> (r: u32)
{
    a.wrapping_add(b)
}

pub fn r19_overflow_arith(a: u64, b: u64) -> (r: u64)
{
    a + b
}

pub fn r20_cast(a: u64) -> (r: u32)
{
    a as u32
}

}
fn main() {}
