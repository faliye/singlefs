//! checker 档模块：无（样本：三条测试各调一态的函数，setup.sh 造的上一版只有中间那一态的函数不同）

fn first_state(value: u32) -> u32 {
    value + 10
}

fn middle_state(value: u32) -> u32 {
    value * 2
}

fn last_state(value: u32) -> u32 {
    value - 1
}

#[test]
fn f1_first() {
    assert_eq!(first_state(1), 11);
}

#[test]
fn f1_middle() {
    assert_eq!(middle_state(2), 4);
}

#[test]
fn f1_last() {
    assert_eq!(last_state(3), 2);
}
