//! S6：照 crates/singlefs-checker-tier/tests/*.rs 的 `#[path = "../../singlefs-harness/tests/common/mod.rs"]`。
#[path = "../../harness/tests/common/mod.rs"]
mod common;

#[test]
fn the_shared_module_is_reached_through_a_path_with_dot_dot() {
    assert_eq!(common::sample_lengths(3), vec![0, 1, 2]);
    assert_eq!(model_core::split::units(25, 10), 3);
}
