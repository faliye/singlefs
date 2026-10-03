use model_core::nested::outer;

#[test]
fn outer_of_three_is_eight() {
    assert_eq!(outer(3), 8);
}
