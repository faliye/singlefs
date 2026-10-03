use model_core::route::route;

#[test]
fn small_values_take_the_small_path() {
    assert_eq!(route(5), 10);
}
