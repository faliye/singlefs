pub fn zeroed_byte() -> u8 {
    // SAFETY: zeroed 造的是 u8，全零就是一个合法的 u8。
    unsafe { std::mem::zeroed::<u8>() }
}
