use super::*;

/// 第二段一个工作项：两遍逐次扫描相同；PC-Z、PC-T、PC-EL、PC-ELr、PC-M 两路对得上、登记写的期望成立；A4、K4 不红。
#[test]
fn positive_controls_on_a_many_objects_history_agree_on_both_paths() {
    let item = SegmentTwoItem {
        history: History::ManyObjectsWithData { inode_count: 300, order: DeletionOrder::Ascending, batch: 144 },
        cell: write_path_cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 4),
    };
    let result = run_segment_two_item(item);
    assert!(result.two_runs_identical && result.last_scan_paths_agree);
    let identifiers: BTreeSet<&str> = result.positive_controls.iter().map(|outcome| outcome.control_identifier).collect();
    for expected in ["PC-Z", "PC-T", "PC-EL", "PC-ELr", "PC-M-data", "PC-M-leaf"] {
        assert!(identifiers.contains(expected), "{expected} 没做：{identifiers:?}");
    }
    for outcome in &result.positive_controls {
        assert!(outcome.paths_agree && outcome.closed_form_holds, "{}", outcome.line(""));
    }
    for name in ["A4_extra_writes_are_one_and_one", "K4_one_whole_object_tombstone_per_unlink"] {
        assert!(result.assertions.judged_of(name) > 0 && result.assertions.violations_of(name) == 0, "{name}");
    }
}
