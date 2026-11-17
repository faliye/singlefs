//! 验证包模块：无（样本：整份挂 gpu 特性的测试文件；带 gpu 特性那一趟里 gpu_stream_second 的 CRASH_POINT_REPLAY_PARALLEL_START … judge=cpu 接在「test <名> ... 」同一行后面，打这一行时只剩它还没出结果，记号归得到它）
#![cfg(feature = "gpu")]

#[test]
fn gpu_stream_first() {}

#[test]
fn gpu_stream_second() {}
