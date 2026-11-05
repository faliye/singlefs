//! 验证包模块：无（样本：整份挂 gpu 特性的测试文件；带 --test-threads 1 那一趟里 gpu_pair_cpu_judged 的 CRASH_POINT_REPLAY_JUDGE route=cpu 接在它的「test <名> ... 」同一行后面，记号归得到它）
#![cfg(feature = "gpu")]

#[test]
fn gpu_pair_on_the_gpu() {}

#[test]
fn gpu_pair_cpu_judged() {}
