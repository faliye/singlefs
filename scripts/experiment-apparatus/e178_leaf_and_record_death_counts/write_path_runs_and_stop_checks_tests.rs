use super::*;

#[test]
fn stop_check_split_matches_the_implementation_rules() {
    let (holds, detail) = stop_check_split_matches_the_implementation();
    assert!(holds, "{detail}");
}

#[test]
fn stop_check_removal_matches_the_implementation_rules() {
    let (holds, detail) = stop_check_removal_matches_the_implementation();
    assert!(holds, "{detail}");
}

#[test]
fn stop_check_unlink_steps_match_the_implementation_rules() {
    let (holds, detail) = stop_check_unlink_steps_match_the_implementation();
    assert!(holds, "{detail}");
}

/// A6：一借小历史（第九节 M5、M6 在这里红：N2 不换身份则右号仍是 17，N1 不抬分隔 key 则右分隔 key 仍是 17）。
#[test]
fn single_borrow_history_moves_five_records_and_costs_one_retirement() {
    let (holds, detail) = single_borrow_anchor_holds();
    assert!(holds, "{detail}");
}

/// 第一段的 5.6 表：「今天 I-9.4 红而 N1 的 I-9.4 绿」的名单（判定由谓词算，不是抄的）里有 K-2。
#[test]
fn the_shape_table_lists_the_shapes_the_raised_separator_reading_misses() {
    let (_, verdicts) = shape_table_lines();
    let lines = shape_verdict_lines(&verdicts);
    let dense_split_family_line = lines.iter().find(|line| line.contains("question=Q-X4-4 layout=dense family=K ")).expect("有 K 族那一行");
    assert!(dense_split_family_line.contains("K-2"), "{dense_split_family_line}");
}
