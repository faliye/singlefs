//! checker 档模块：无（样本：三条测试同时跑，假 cargo 让 test_c 跑完之前打一行 NOT_RUN_：只有它不记库）

#[test]
fn test_a() {}

#[test]
fn test_b() {}

#[test]
fn test_c() {}
