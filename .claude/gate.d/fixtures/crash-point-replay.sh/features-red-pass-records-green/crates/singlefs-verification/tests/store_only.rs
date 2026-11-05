//! 验证包模块：无（样本：整份挂一个特性的测试文件；带特性那一趟假 cargo 判 store_only_red FAILED、store_only_quick ok）
#![cfg(feature = "sample-store")]

#[test]
fn store_only_quick() {}

#[test]
fn store_only_red() {}
