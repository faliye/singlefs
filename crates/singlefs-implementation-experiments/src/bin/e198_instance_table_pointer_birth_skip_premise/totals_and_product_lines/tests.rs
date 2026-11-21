use super::*;

/// 多线程与单线程的对拍（`.claude/rules/test-script-discipline.md`「编译与算法都要快」）：同两段历史用 1 个与 2 个线程各判一遍，
/// 交回的次序与每一行产物逐字相同。
#[test]
fn histories_judged_with_one_or_two_threads_come_back_in_the_same_order_with_the_same_lines() {
    let specifications = vec![
        skipped_row_pin_specification(),
        newest_row_bad_pin_specification(),
    ];
    let lines_of = |results: &[HistoryResult]| -> Vec<String> {
        results
            .iter()
            .flat_map(|result| {
                let mut lines = history_lines(result);
                lines.extend(result.images.iter().map(|image| image_line(result, image)));
                lines
            })
            .collect()
    };
    let one_thread = lines_of(&run_the_histories(&specifications, 1));
    let two_threads = lines_of(&run_the_histories(&specifications, 2));
    assert!(!one_thread.is_empty(), "两段历史都有产物行");
    assert_eq!(one_thread, two_threads, "线程数不改产物");
}
