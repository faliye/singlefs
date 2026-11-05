//! 验证包模块：无（样本：整份挂 gpu 特性的测试文件；带 --test-threads 1 那一趟里 gpu_pair_cpu_judged 起跑之后打了一行判段判在 CPU，记号归得到它）
#![cfg(feature = "gpu")]

#[test]
fn gpu_pair_on_the_gpu() {}

#[test]
fn gpu_pair_cpu_judged() {}
