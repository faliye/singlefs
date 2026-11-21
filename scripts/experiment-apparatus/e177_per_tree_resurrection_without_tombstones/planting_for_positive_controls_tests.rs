use super::*;

#[test]
fn ring_equals_wide_before_the_first_reuse_and_every_positive_control_matches() {
    let result = run_segment(SMALL_REWRITE);
    assert_eq!(result.assertions.violations_of("A16_ring_equals_wide_before_the_first_reuse"), 0);
    assert_eq!(result.assertions.judged.get("A16_ring_equals_wide_before_the_first_reuse").copied(), Some(2));
    assert_eq!(result.assertions.violations_of("positive_control_matches"), 0);
    assert!(result.positive_controls.iter().any(|(_, control)| control.measured.is_some()));
}

#[test]
fn positive_controls_match_on_the_empty_inode_build_and_delete_family() {
    let result = run_segment(HistorySpecification::BuildAndDeleteEmptyInodes {
        inode_count: 300,
        order: DeletionOrder::Ascending,
    });
    let measured: Vec<&PositiveControl> = result
        .positive_controls
        .iter()
        .map(|(_, control)| control)
        .filter(|control| control.measured.is_some())
        .collect();
    assert!(measured.len() > 50);
    for control in &measured {
        assert_eq!(control.measured, Some(control.expected), "{control:?}");
    }
    assert!(measured.iter().any(|control| control.kind == "PC2" && control.expected > 0));
    assert!(measured.iter().any(|control| control.kind == "PC1_tombstoned" && control.measured == Some(0)));
    assert!(measured.iter().any(|control| control.kind == "PC1_without_its_death_record" && control.measured == Some(1)));
}
