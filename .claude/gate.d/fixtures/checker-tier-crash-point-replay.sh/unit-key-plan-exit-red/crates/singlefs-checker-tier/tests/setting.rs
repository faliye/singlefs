//! checker 档模块：无（样本：一份小设点文件，三条测试；.unit-verdict-sample/plan-failure 让 test-plan 装失灵）

fn observe(state: u32) -> u32 {
    state + 1
}

#[test]
fn test_t() {
    assert_eq!(observe(1), 2);
}

#[test]
fn test_u() {
    let word = "observed";
    assert_eq!(word.len(), 8);
}

#[test]
fn test_v() {
    let word = "unchanged";
    assert_eq!(word.len(), 9);
}
