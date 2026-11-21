use super::*;

#[test]
fn the_units_kept_arm_has_nine_prefixed_histories_all_at_sixty_four_units_next_to_the_eighteen_registered_ones(
) {
    let kept = units_kept_history_specifications();
    assert_eq!(kept.len(), 9, "(默认 2 个起点 + 窄盘 1 个起点) × 三族");
    assert_eq!(
        main_history_specifications().len(),
        18,
        "第一段的 18 条不动"
    );
    for specification in &kept {
        assert_eq!(
            specification.first_file_units, 64,
            "s = 1 时两种形状同是一个单元，不另建"
        );
        assert_eq!(
            specification.overwrite_shape,
            OverwriteShape::KeepingTheFirstFileUnits
        );
        assert!(history_name(specification).starts_with("units-kept."));
    }
    let kept_names: BTreeSet<String> = kept.iter().map(history_name).collect();
    let main_names: BTreeSet<String> = main_history_specifications()
        .iter()
        .map(history_name)
        .collect();
    assert_eq!(kept_names.len(), 9, "补臂的历史名互不相同");
    assert!(
        kept_names.is_disjoint(&main_names),
        "补臂的历史名与第一段的 18 条不相交（片结果库的键含历史名）"
    );
    assert!(
        kept_names.contains(&arm_witness_history_name(
            OverwriteShape::KeepingTheFirstFileUnits
        )),
        "甲一、甲二判法对照挑状态的那条历史在补臂里"
    );
}
