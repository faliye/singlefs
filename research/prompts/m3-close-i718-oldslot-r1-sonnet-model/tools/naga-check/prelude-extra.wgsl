// 草稿里的假前言：走读内核的前言由 Rust 一侧生成（crash_judge_gpu.rs 的 kernel_prelude_wgsl_constants 与入口声明），这里只给校验函数闭包要的两样
const TABLE_COUNT: u32 = 8u;
@group(0) @binding(0) var<storage, read_write> tables: array<u32>;
@group(0) @binding(1) var<storage, read_write> scratch: array<u32>;
