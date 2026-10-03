//! S7：常量上下文里算出来的值。
#[test]
fn compile_time_values_are_pinned() {
    assert_eq!(model_core::constfn::DOUBLE_BASE, 6);
    assert_eq!(model_core::constfn::LIMIT, 256);
}
