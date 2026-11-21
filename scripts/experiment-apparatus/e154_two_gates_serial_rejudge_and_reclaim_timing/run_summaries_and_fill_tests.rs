use super::*;

/// 阳性对照 PC1 在装置里真的会改变 Q1（不是摆设）：H1 在真实基线上有推空发生的那一格，
/// 关掉抬 F 之后重试循环只能靠环转一圈，K 应变得更大或直接不达标。
#[test]
fn positive_control_disabling_floor_raise_changes_delete_then_rewrite_outcome() {
    let geometry = Geometry {
        capacity_slots_per_disk: 4000,
        object_units: 16,
        fixpoint_cost: 4,
        slots_per_region: 8,
        padding_deletes: 2,
    };
    let arm = real_baseline_arm();
    let baseline = run_delete_then_rewrite_history(arm, geometry);
    let damaged = run_delete_then_rewrite_history_with_damage(arm, geometry, Damage { disable_floor_raise: true, ..Damage::default() });
    assert_ne!(
        (baseline.delete_then_rewrite_outcome_label.clone(), baseline.delete_then_rewrite_publish_count),
        (damaged.delete_then_rewrite_outcome_label, damaged.delete_then_rewrite_publish_count),
        "PC1 应当能改变 Q1：baseline={:?}/{} damaged 应不同",
        baseline.delete_then_rewrite_outcome_label, baseline.delete_then_rewrite_publish_count
    );
}
