//! X4–X6：跨 crate inline(always)、函数指针、dyn 分派。
use mcore::dynd::Shape;

#[test]
fn mix_inlined_across_crates() {
    assert_eq!(mcore::inl::mix(2, 3), 9);
}

#[test]
fn triple_through_a_function_pointer_table() {
    assert_eq!((mcore::fptr::OPS[1])(5), 15);
}

#[test]
fn area_through_dyn_dispatch() {
    assert_eq!(mcore::dynd::boxed(4).area(), 16);
}
