//! 验证包模块：无（样本：放开 --test-threads 1 的 GPU 对拍测试目标，整份挂两个特性）
#![cfg(all(feature = "sample-store", feature = "gpu"))]

#[test]
fn crash_judge_gpu_agrees_on_the_first_sample_state() {}

#[test]
fn crash_judge_gpu_agrees_on_the_second_sample_state() {}
