// 样本：include_str! 用方括号定界。
const KERNEL: &str = include_str!["../../provider/src/k.wgsl"];

#[test]
fn kernel_is_not_empty() {
    assert!(!KERNEL.is_empty());
}
