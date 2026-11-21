use super::*;

#[test]
fn layer_zero_below_sixty_four_nodes_never_triggers_second_half() {
    assert!(!layer_zero_is_mostly_dirty(&[63], &[63.0]), "节点数 63 < 64，占比 100% 也不该触发");
    assert!(!layer_zero_is_mostly_dirty(&[], &[]), "空向量不该触发");
}

#[test]
fn layer_zero_at_or_above_sixty_four_nodes_triggers_at_the_fifty_percent_boundary() {
    assert!(!layer_zero_is_mostly_dirty(&[64], &[31.99]), "占比刚好低于 50% 不该触发");
    assert!(layer_zero_is_mostly_dirty(&[64], &[32.0]), "占比恰好 50% 应该触发");
    assert!(layer_zero_is_mostly_dirty(&[100], &[99.0]), "占比 99% 应该触发");
}

#[test]
fn convergence_and_saturation_status_reports_both_conditions_independently() {
    // 第一半触发、第二半不触发。
    let (half1, half2) = convergence_and_saturation_status(MAXIMUM_FIXED_POINT_ITERATIONS, &[1], &[1.0], &[1, 1], &[1.0, 1.0], &[1], &[1.0], &[1], &[1.0]);
    assert!(half1);
    assert!(half2.is_empty());
    // 第一半不触发、第二半在 allocation 与 mapping 两棵树上触发。
    let (convergence_when_not_triggered, saturated_trees_when_two_trees_trigger) = convergence_and_saturation_status(1, &[1], &[1.0], &[1, 1], &[1.0, 1.0], &[100], &[60.0], &[100], &[60.0]);
    assert!(!convergence_when_not_triggered);
    assert_eq!(saturated_trees_when_two_trees_trigger, vec!["allocation", "mapping"]);
}
