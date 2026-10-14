//! checker 档模块：无（样本：整份挂一个特性、唯一的用例标了 ignore 的测试文件：编进来了，快档跑到 0 条）
#![cfg(feature = "verdict-store")]

#[test]
#[ignore]
fn store_only_quick() {}
