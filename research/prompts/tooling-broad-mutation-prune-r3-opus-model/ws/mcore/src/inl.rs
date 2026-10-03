//! X4：跨 crate 的 #[inline(always)]。
#[inline(always)]
pub fn mix(a: u32, b: u32) -> u32 {
    a * 3 + b
}
