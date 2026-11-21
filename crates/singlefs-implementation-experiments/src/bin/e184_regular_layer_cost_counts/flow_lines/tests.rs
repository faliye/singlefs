use super::*;

#[test]
fn the_group_of_a_label_drops_only_the_seed_suffix() {
    assert_eq!(
        group_of_the_label("random_history-broad-FourGibibytes-steps0100-seed07"),
        "random_history-broad-FourGibibytes-steps0100"
    );
    assert_eq!(
        group_of_the_label("overwrite_chain-04_units_per_overwrite"),
        "overwrite_chain-04_units_per_overwrite"
    );
    assert_eq!(
        group_of_the_label("smoke-snapshot_creation_stream"),
        "smoke-snapshot_creation_stream"
    );
    assert_eq!(group_of_the_label("odd-seedx"), "odd-seedx");
}
