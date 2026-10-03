//! X3：运行时的弱测试与常量上下文里的强测试。
#[test]
fn square_of_one_is_positive() {
    assert!(mcore::mixed::square(1) >= 1);
}

const NINE: u32 = mcore::mixed::square(3);

#[test]
fn square_of_three_is_nine_at_compile_time() {
    assert_eq!(NINE, 9);
}
