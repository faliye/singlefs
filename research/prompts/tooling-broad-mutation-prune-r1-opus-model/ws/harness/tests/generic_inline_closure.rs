//! S8：泛型在这个测试 crate 里单态化、`#[inline]` 跨 crate、闭包。
#[test]
fn generic_inline_and_closure_code_runs() {
    assert_eq!(model_core::generic::largest(&[3u32, 9, 4]), 9);
    assert_eq!(model_core::generic::halve(10), 5);
    assert_eq!(model_core::generic::apply_twice(1), 3);
}
