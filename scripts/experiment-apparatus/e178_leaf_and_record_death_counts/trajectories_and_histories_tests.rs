use super::*;

/// A1：H51 逐段重放的发布数与被拒步数（F1 式子 607；F8 式子；F3 五段与 E177 产物那两段相同）（第九节 M14 在这里红）。
#[test]
fn the_replayed_histories_match_the_registered_publish_counts() {
    let first = run_write_path(History::BuildAndDeleteEmptyInodes { inode_count: 300, order: DeletionOrder::EvenThenOdd }, write_path_cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0));
    assert_eq!(first.publishes, 606 + 1);
    let large = run_write_path(
        History::LargeFileRewriteAndDelete { data_units: 300, rewrites: 5, overwrite_before_unlink: false, batch: 144 },
        write_path_cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0),
    );
    assert_eq!(large.publishes, 12 + 1);
    for (seed, publishes, refused) in E177_MIXED_RANDOM_PUBLISHES_AND_REFUSALS {
        let mixed = run_write_path(History::MixedRandom { seed }, write_path_cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 8));
        assert_eq!((mixed.publishes, mixed.refused_steps), (publishes, refused), "seed {seed}");
        assert_eq!(mixed.assertions.violations_of("A1_history_publishes"), 0);
    }
}

/// V7：F10 在 θ > 0 下，N2 的每一版按今天的 I-9.4 与 I-9.12 全绿，N1 的每一版按 N1 的 I-9.4 与 I-9.12 全绿；借、吸收、
/// 被挪过的对象变短与被 unlink 都跑到（第九节 M5、M6、M7 在这里红）。
#[test]
fn merging_many_objects_keeps_every_published_tree_legal() {
    let history = History::ManyObjectsWithData { inode_count: 300, order: DeletionOrder::Ascending, batch: 144 };
    for threshold in [4, 8] {
        for borrow_arm in BORROW_ARMS {
            let run = run_write_path(history, write_path_cell(borrow_arm, TEST_LEAF_CAPACITY, threshold));
            assert!(self_check_holds(borrow_arm, threshold, run.self_check), "{borrow_arm:?} θ {threshold}: {:?}", run.self_check);
            assert!(path_was_covered(&run), "{borrow_arm:?} θ {threshold}: borrows {} shortened {} unlinked {}", run.borrows, run.moved_then_shortened, run.moved_then_unlinked);
            assert!(run.absorbs > 0);
            assert_eq!(run.assertions.violations_of("K6_split_right_half_numbered_after_the_record_left_half_unchanged"), 0);
            if borrow_arm == BorrowArm::RaiseSeparator {
                assert!(run.self_check.today_container_numbers_red > 0, "N1 借之后今天的 I-9.4 红是定义的推论");
            }
        }
    }
}

/// Q-X4-1、Q-X4-2 的定义推论：每次借 N2 − N1 = 码 3 + 1、码 2 + 1，N2 退役记录 1 条、N1 0 条，别的发布逐单元相同。
#[test]
fn every_borrow_costs_renewed_identity_one_container_and_one_root() {
    let history = History::ManyObjectsWithData { inode_count: 300, order: DeletionOrder::Random(2), batch: 1 };
    let raise_separator = run_write_path(history, write_path_cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 8));
    let renew_identity = run_write_path(history, write_path_cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 8));
    let pair = pair_statistics(&raise_separator, &renew_identity);
    assert!(pair.borrow_publishes > 0);
    assert_eq!((pair.extra_code_three_per_borrow.smallest, pair.extra_code_three_per_borrow.largest), (Some(1), Some(1)));
    assert_eq!((pair.extra_code_two_per_borrow.smallest, pair.extra_code_two_per_borrow.largest), (Some(1), Some(1)));
    assert_eq!(pair.retirement_records_per_borrow_renew_identity.largest, Some(1));
    assert_eq!(pair.retirement_records_per_borrow_raise_separator.largest, Some(0));
    assert_eq!(pair.publishes_outside_borrows_with_a_difference + pair.borrow_kinds_disagree, 0);
    assert!(pair.user_changes_identical);
}

#[test]
fn trajectory_reports_peak_positive_scans_and_last() {
    let mut trajectory = QuantityTrajectory::default();
    for (value, publish) in [(0, 1), (3, 2), (1, 3), (0, 4)] {
        trajectory.observe(value, publish);
    }
    assert_eq!((trajectory.peak, trajectory.peak_at_publish, trajectory.positive_scans, trajectory.last, trajectory.scans), (3, 2, 2, 0, 4));
}
