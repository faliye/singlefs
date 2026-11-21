use super::*;

/// A5：θ = 0 时 N1、N2 两条写流逐发布相同（叶容器摘要与写的单元数），一次借都没有。
#[test]
fn without_merges_both_borrow_arms_write_the_same_stream() {
    let history = History::BuildAndDeleteEmptyInodes { inode_count: 300, order: DeletionOrder::Random(1) };
    let raise_separator = run_write_path(history, write_path_cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 0));
    let renew_identity = run_write_path(history, write_path_cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 0));
    let pair = pair_statistics(&raise_separator, &renew_identity);
    assert!(pair.streams_identical && pair.user_changes_identical && pair.publish_counts_identical);
    assert_eq!(pair.borrow_publishes, 0);
}

/// A8：θ = 0 时退役记录数 = 身份消失次数，F1 n = 300 容量 16 是 ⌈300 ÷ 16⌉ − 1 = 18（第九节 M4 在这里红）。
#[test]
fn at_zero_threshold_every_emptied_leaf_writes_one_retirement_record() {
    let history = History::BuildAndDeleteEmptyInodes { inode_count: 300, order: DeletionOrder::Ascending };
    let run = run_write_path(history, write_path_cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 0));
    assert_eq!(run.written.retirement_records, 17 + 1);
    assert_eq!(run.drops, 17 + 1);
    assert_eq!(run.assertions.violations_of("A8_retirements_equal_identity_disappearances"), 0);
}
