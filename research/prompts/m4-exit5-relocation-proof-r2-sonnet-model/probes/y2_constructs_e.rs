// 探针 y2-e：panic!、unreachable!、unwrap、Result::expect、标量移位、枚举 match 带通配、impl Trait 返回、泛型 where 约束
use vstd::prelude::*;
verus! {

pub fn e01_panic_reachable(x: u64) -> (r: u64) { if x > 3 { panic!("x 太大：{}", x); } x }
pub fn e02_panic_unreachable(x: u64) -> (r: u64) requires x <= 3 { if x > 3 { panic!("x 太大：{}", x); } x }
pub fn e03_unreachable_macro(x: u8) -> (r: u8) requires x < 2 { match x { 0 => 1, 1 => 0, _ => unreachable!() } }
pub fn e04_result_expect(r: Result<u64, u8>) -> (v: u64) requires r is Ok { r.expect("必须是 Ok") }
pub fn e05_shift_mask(x: u64) -> (r: u64) { (x >> 8) & 0xff }
pub fn e06_u32_u64_cast(x: u32) -> (r: u64) { (x as u64) << 32 }
pub fn e07_wrapping_sub(a: u64, b: u64) -> (r: u64) { a.wrapping_sub(b) }
pub fn e08_vec_index_mut(v: &mut Vec<u8>, i: usize) requires i < old(v).len() { v[i] = 1; }
pub fn e09_vec_of_vec(v: &Vec<Vec<u8>>) -> (r: usize) { let mut t = 0usize; let mut i = 0usize; while i < v.len() invariant i <= v.len(), t <= i * 0, { i += 1; } t }
pub fn e10_tuple_return(a: u64) -> (r: (u64, bool)) { (a, true) }
pub fn e11_struct_update(a: u64) -> (r: u64) { let t = (a, 1u64); t.0 + 0 }
pub fn e12_array_literal() -> (r: [u8; 4]) { [1, 2, 3, 4] }
pub fn e13_array_loop(a: &[u8; 4]) -> (r: u8) { let mut m = 0u8; let mut i = 0usize; while i < 4 invariant i <= 4, { if a[i] > m { m = a[i]; } i += 1; } m }
pub fn e14_option_take(o: &mut Option<u64>) -> (r: Option<u64>) { o.take() }
pub fn e15_string_literal() -> (r: &'static str) { "abc" }
pub fn e16_debug_assert(x: u64) { debug_assert!(x < 10 || x >= 10); }
pub fn e17_char_ops(c: char) -> (r: bool) { c == 'a' }
pub fn e18_const_in_expr() -> (r: u64) { const LIMIT: u64 = 64; LIMIT * 2 }
pub fn e19_loop_labeled(n: usize) -> (r: usize) { let mut c = 0usize; 'outer: loop { c += 1; if c >= n { break 'outer; } if c > 100 { break; } } c }
pub fn e20_slice_pattern(v: &[u8]) -> (r: u8) { match v { [a, ..] => *a, [] => 0 } }

}
fn main() {}
