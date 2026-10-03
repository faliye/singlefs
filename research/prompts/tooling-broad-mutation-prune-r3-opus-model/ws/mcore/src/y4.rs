//! Y4：被改函数用 y4_macros.rs 里的宏。
pub fn bump(x: u32) -> u32 {
    let s = step!(x);
    s - 1
}
