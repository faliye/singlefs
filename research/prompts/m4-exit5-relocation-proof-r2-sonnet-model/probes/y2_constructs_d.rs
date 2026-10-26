// 探针 y2-d：按名字配不上 vstd 规格的高频键，逐个写成小函数，看 Verus 怎么说。
use vstd::prelude::*;
verus! {

pub fn d01_extend(v: &mut Vec<u8>, w: Vec<u8>) { v.extend(w); }
pub fn d02_enumerate(v: &Vec<u8>) -> (r: usize) { let mut c = 0usize; for (i, _x) in v.iter().enumerate() { c = i; } c }
pub fn d03_iter_copied(v: &Vec<u8>) -> (r: Vec<u8>) { v.iter().copied().collect::<Vec<u8>>() }
pub fn d04_contains(v: &[u8], x: u8) -> (r: bool) { v.contains(&x) }
pub fn d05_chain(v: &Vec<u8>, w: &Vec<u8>) -> (r: Vec<u8>) { v.iter().chain(w.iter()).copied().collect::<Vec<u8>>() }
pub fn d06_filter_map(v: &Vec<u8>) -> (r: Vec<u8>) { v.iter().filter_map(|x: &u8| if *x > 3 { Some(*x) } else { None }).collect::<Vec<u8>>() }
pub fn d07_map_or_else(o: Option<u64>) -> (r: u64) { o.map_or_else(|| 0u64, |x: u64| x) }
pub fn d08_then_some(b: bool) -> (r: Option<u8>) { b.then_some(1u8) }
pub fn d09_option_filter(o: Option<u64>) -> (r: Option<u64>) { o.filter(|x: &u64| *x > 3) }
pub fn d10_iter_max(v: &Vec<u64>) -> (r: Option<u64>) { v.iter().copied().max() }
pub fn d11_sum(v: &Vec<u64>) -> (r: u64) { v.iter().copied().sum::<u64>() }
pub fn d12_windows(v: &[u8]) -> (r: usize) { v.windows(2).count() }
pub fn d13_chunks(v: &[u8]) -> (r: usize) { let mut n = 0usize; for _c in v.chunks(4) { n = n + 1; } n }
pub fn d14_retain(v: &mut Vec<u8>) { v.retain(|x: &u8| *x > 3); }
pub fn d15_is_none_or(o: Option<u64>) -> (r: bool) { o.is_none_or(|x: u64| x > 3) }
pub fn d16_btree_entry(m: &mut std::collections::BTreeMap<u64, u64>, k: u64) { *m.entry(k).or_insert(0) = 1; }
pub fn d17_btree_remove(m: &mut std::collections::BTreeMap<u64, u64>, k: u64) -> (r: Option<u64>) { m.remove(&k) }
pub fn d18_btreeset(m: &mut std::collections::BTreeSet<u64>, k: u64) -> (r: bool) { m.insert(k) }
pub fn d19_vec_truncate(v: &mut Vec<u8>) { v.truncate(2); }
pub fn d20_vec_resize(v: &mut Vec<u8>) { v.resize(8, 0u8); }
pub fn d21_vec_from_iter_range(n: u64) -> (r: Vec<u64>) { (0..n).collect::<Vec<u64>>() }
pub fn d22_vec_concat_clone(v: &Vec<u8>) -> (r: Vec<u8>) { let mut w = v.clone(); w.extend_from_slice(&[1u8, 2u8]); w }
pub fn d23_slice_eq(a: &[u8], b: &[u8]) -> (r: bool) ensures r == (a@ == b@) { a == b }
pub fn d24_u64_pow_shift(a: u64) -> (r: u64) requires a < 1000 { (a << 3) | 1 }
pub fn d25_usize_as(a: u64) -> (r: usize) { a as usize }
pub fn d26_box_dyn_fn(f: Box<dyn Fn(u64) -> u64>, x: u64) -> (r: u64) { f(x) }
pub fn d27_swap_mem(a: &mut u64, b: &mut u64) { core::mem::swap(a, b); }
pub fn d28_mem_take(a: &mut Vec<u8>) -> (r: Vec<u8>) { core::mem::take(a) }

}
fn main() {}
