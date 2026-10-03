//! checker 档模块：无（样本：整份挂两个特性、cfg 折成几行的测试文件；这一份样本里 cargo 输出没有它那一段）
#![cfg(all(
    feature = "verdict-store",
    feature = "gpu"
))]

#[test]
fn gpu_pair_agrees_with_the_cpu() {}
