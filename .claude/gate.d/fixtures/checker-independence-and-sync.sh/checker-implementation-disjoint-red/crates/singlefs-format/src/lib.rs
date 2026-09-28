pub const SAMPLE_BYTES: u64 = 16384;
pub const fn pick(is_wide: bool) -> u64 { if is_wide { 32768 } else { SAMPLE_BYTES } }
#[cfg(test)]
const TEST_ONLY_BYTES: u64 = 1;
pub const fn clamp(value: u64) -> u64 { match value { 0 => 1, other => other } }
