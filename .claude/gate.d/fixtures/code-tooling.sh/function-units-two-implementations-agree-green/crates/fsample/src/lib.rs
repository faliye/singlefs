#![allow(dead_code)]
pub mod nested {
    pub const LIMIT: u32 = 3;
    pub fn inside() -> u32 {
        LIMIT + 1
    }
}
pub const WIDTH: u32 = 4;
pub fn entry() -> u32 {
    nested::inside() + WIDTH
}
pub fn table() -> usize {
    include_str!("table.txt").len()
}
pub trait Weigh {
    fn weigh(&self) -> u32;
}
impl<T> Weigh for (T, u32) {
    fn weigh(&self) -> u32 {
        self.1
    }
}
impl<T> Weigh for [T] {
    fn weigh(&self) -> u32 {
        self.len() as u32
    }
}
impl<T, const N: usize> Weigh for [T; N] {
    fn weigh(&self) -> u32 {
        N as u32
    }
}
impl<T: Weigh> Weigh for Box<T> {
    fn weigh(&self) -> u32 {
        (**self).weigh() + 1
    }
}
impl<T: Weigh> Weigh for Option<Box<T>> {
    fn weigh(&self) -> u32 {
        match self {
            Some(inner) => inner.weigh() + 2,
            None => 0,
        }
    }
}
pub fn heaviest(pair: &(u8, u32)) -> u32 {
    pair.weigh()
}
pub const KERNEL_SOURCE: &str = include_str!("kernel.wgsl");
pub const SHARED_SOURCE: &str = include_str!("shared.wgsl");
pub fn kernel_text() -> &'static str {
    KERNEL_SOURCE
}
