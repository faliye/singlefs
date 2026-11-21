pub fn clamp_plain(value: u64) -> u64 {
    if value > 10 { 10 } else { value }
}
