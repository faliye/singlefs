// 样本：include_str! 经 use 改名成 grab! 再用。
use std::include_str as grab;

const KERNEL: &str = grab!("../../provider/src/k.wgsl");

#[test]
fn kernel_is_not_empty() {
    assert!(!KERNEL.is_empty());
}
