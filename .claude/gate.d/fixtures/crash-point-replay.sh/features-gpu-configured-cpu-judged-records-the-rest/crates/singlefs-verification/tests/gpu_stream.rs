//! 验证包模块：无（样本：整份挂 gpu 特性的测试文件；判段判在 CPU 的一行打在这一段起头、哪一条都还没起跑时，记号归不到哪一条，整个二进制判绿的都不记）
#![cfg(feature = "gpu")]

#[test]
fn gpu_stream_first() {}

#[test]
fn gpu_stream_second() {}
