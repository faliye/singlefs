pub const SAMPLE_BYTES: u64 = 16384;
pub const fn pick(is_wide: bool) -> u64 { if is_wide { 32768 } else { SAMPLE_BYTES } }
