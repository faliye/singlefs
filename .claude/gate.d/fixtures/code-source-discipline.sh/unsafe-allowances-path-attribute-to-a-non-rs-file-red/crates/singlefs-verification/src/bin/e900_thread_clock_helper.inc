pub fn zeroed_byte() -> u8 {
    unsafe { std::mem::zeroed::<u8>() }
}
