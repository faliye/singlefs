//! S5：宏展开出来的代码。
#[test]
fn the_macro_wraps_once() {
    assert_eq!(model_core::macro_user::wrapped(30), 6);
    assert_eq!(model_core::macro_user::wrapped(3), 3);
}
