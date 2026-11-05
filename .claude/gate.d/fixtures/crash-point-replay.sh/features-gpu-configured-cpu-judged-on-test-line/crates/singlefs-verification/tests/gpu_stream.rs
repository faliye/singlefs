//! 验证包模块：无（样本：整份挂 gpu 特性的测试文件；带 --test-threads 1 那一趟里 gpu_stream_second 的 CRASH_POINT_REPLAY_PARALLEL_START … judge=cpu 接在它的「test <名> ... 」同一行后面，记号归得到它）
#![cfg(feature = "gpu")]

#[test]
fn gpu_stream_first() {}

#[test]
fn gpu_stream_second() {}
