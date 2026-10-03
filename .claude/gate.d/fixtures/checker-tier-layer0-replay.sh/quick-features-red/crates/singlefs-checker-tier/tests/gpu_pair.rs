//! checker 档模块：无（样本：整份挂两个特性、cfg 折成几行的测试文件；这一份样本里它在没卡的机器上 panic）
#![cfg(all(
    feature = "verdict-store",
    feature = "gpu"
))]

#[test]
fn gpu_pair_agrees_with_the_cpu() {}
