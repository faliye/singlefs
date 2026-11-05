#![allow(dead_code)]
pub fn entry() -> u32 {
    let mut total = 1;
    #[cfg(feature = "gpu")]
    {
        total += gpu_boost();
    }
    total
}
#[cfg(feature = "gpu")]
fn gpu_boost() -> u32 {
    7
}
