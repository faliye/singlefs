//! S2 只有常量的模块：被改文件 ring.rs 引用这里的常量，这里的常量再引用 fmtc 的常量（一跳之外）。
pub const REGION_SLOTS: u64 = model_fmtc::SLOTS_AT_MKFS * 3;
