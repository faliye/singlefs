//! checker 档模块：无（样本：只有一条用例挂特性的测试文件；下一行注释里的特性名不算）
// #[cfg(feature = "named-only-in-a-comment")]

#[test]
fn accelerator_items_plain() {}

#[cfg(feature = "sample-accelerator")]
#[test]
fn accelerator_items_accelerated() {}
