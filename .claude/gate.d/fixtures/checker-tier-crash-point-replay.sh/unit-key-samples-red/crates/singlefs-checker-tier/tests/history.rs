//! checker 档模块：无（样本：H2 那一处改动连 HEAD 一起提交、H3 只做一次空提交，键不看 HEAD 与提交）

fn h2_value() -> u32 {
    22
}

fn h3_value() -> u32 {
    33
}

#[test]
fn h2_test() {
    assert_eq!(h2_value(), 22);
}

#[test]
fn h3_test() {
    assert_eq!(h3_value(), 33);
}
