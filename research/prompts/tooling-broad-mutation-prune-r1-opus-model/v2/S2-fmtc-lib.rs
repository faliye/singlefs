//! 只放常量的 crate，照 crates/singlefs-format/src/lib.rs（那份文件里非测试的部分只有常量）。
/// 测试取样上沿（S1：只被测试文件引用，被改文件不引用）。
pub const SAMPLE_LIMIT: u64 = 25;
/// mkfs 时每个区域几个槽。第二版取 16（照 singlefs-format 那条「4..16 之间取值」的可调常量）。
pub const SLOTS_AT_MKFS: u64 = 16;
