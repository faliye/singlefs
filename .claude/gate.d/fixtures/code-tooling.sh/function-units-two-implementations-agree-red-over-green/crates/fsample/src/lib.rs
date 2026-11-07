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
