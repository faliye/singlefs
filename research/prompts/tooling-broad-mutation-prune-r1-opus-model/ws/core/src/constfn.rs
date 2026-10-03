/// S7 只在常量上下文里被叫到的 const fn（照 root_ring.rs 的 at_make_filesystem_checked_at_compile_time），与一个常量初始化式（照 address.rs 的 COUNTER_LIMIT）。
pub const fn twice(value: u64) -> u64 {
    value * 2
}
pub const DOUBLE_BASE: u64 = twice(3);
pub const LIMIT: u64 = 1 << 8;
