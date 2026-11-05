#[allow(unsafe_code, reason = "样本：造一个全零的 u8")]
pub fn zeroed_byte() -> u8 {
    // SAFETY: zeroed 造的是 u8，全零就是一个合法的 u8，不读写调用方的内存。
    unsafe { std::mem::zeroed::<u8>() }
}
