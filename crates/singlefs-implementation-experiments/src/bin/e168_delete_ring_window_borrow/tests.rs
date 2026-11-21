use super::*;

#[test]
#[ignore = "要补丁表副本：主工作区那份的 delete_path_hooks 是桩，一调就打停机行退 3；在套过补丁表的副本里 `-- --ignored` 跑"]
fn segment_one_on_the_narrow_disk_with_four_slots_per_region_prints_the_same_lines_on_one_thread_and_on_eight_threads(
) {
    let lines_and_stop_line_on = |thread_count: usize| {
        lines_and_stop_line_of(|| {
            run_segment_one(
                Some("P2-S4"),
                SliceScheduling {
                    thread_count,
                    large_slice_thread_count: 1,
                },
            );
        })
    };
    let on_one_thread = lines_and_stop_line_on(1);
    assert!(
        on_one_thread
            .0
            .iter()
            .any(|line| line.starts_with("E name=group point=P2 S=4 ")),
        "P2-S4 的梯子组跑到了组行（不是一开头就停）"
    );
    assert_eq!(
        lines_and_stop_line_on(8),
        on_one_thread,
        "八条线程与一条线程打出的行逐行相同、停机行（有的话）相同（登记第十二节修订第 12 条：只改调度、不改判据）"
    );
}
