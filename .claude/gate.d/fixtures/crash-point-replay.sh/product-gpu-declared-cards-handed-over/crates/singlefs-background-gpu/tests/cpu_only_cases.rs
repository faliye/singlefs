// 样本：CPU 的集成测试，feature = "tests-on-a-real-gpu" 只在注释里出现，不挂特性
use singlefs_background_gpu::spare_function;

#[test]
fn a_cpu_only_case() {
    assert_eq!(spare_function(), 7);
}
