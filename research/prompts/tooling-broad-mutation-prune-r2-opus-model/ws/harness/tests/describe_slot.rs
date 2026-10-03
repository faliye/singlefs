use model_core::slots::describe;

#[test]
fn describe_names_the_slot() {
    assert_eq!(describe(3 * 4096), "slot-3");
}
