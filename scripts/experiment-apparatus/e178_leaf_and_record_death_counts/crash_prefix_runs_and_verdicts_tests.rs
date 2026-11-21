use super::*;

#[test]
fn stop_check_intent_matches_the_implementation_rules() {
    let (holds, detail) = stop_check_intent_matches_the_implementation();
    assert!(holds, "{detail}");
}

#[test]
fn stop_check_tombstone_tree_matches_the_implementation_rules() {
    let (holds, detail) = stop_check_tombstone_tree_matches_the_implementation();
    assert!(holds, "{detail}");
}

/// 第九节 M10 的取样点：F12 P-1、u = 145、b = 1，意图那一版之后崩溃（被抛弃的是第一批），恢复之后写行那一版的扫描：
/// txg 已重发、被抛弃那一批的叶容器落在父链上，只有已发布谓词挡它；E-L-w × Z 的 extent m 是 0。
#[test]
fn the_published_predicate_hides_the_batch_the_crash_abandoned() {
    let (prefix, object) = build_crash_prefix(CrashPrefix::FirstFileOnly, write_path_cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0)).expect("P-1 前缀");
    let mut simulation = prefix;
    simulation.batch_data_units = 1;
    simulation.apply(UserChange::WriteFile { object, data_units: 145 });
    assert!(simulation.publish(&Operation::UnlinkIntent { object }));
    let abandoned_txg = simulation.version.txg + 1;
    assert!(simulation.crash_with_the_next_unlink_step_written().is_some());
    simulation.scanning = true;
    simulation.recover_after_the_crash();
    let row_scan = &simulation.scans[0];
    assert_eq!(row_scan.txg, abandoned_txg, "写行那次重发了被抛弃的 txg");
    for window in READABLE_WINDOWS {
        let last_batch_written = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtLastBatchWritten, window);
        assert_eq!(row_scan.counts[last_batch_written].extent_missed, 0);
        assert_eq!(row_scan.counts[last_batch_written].inode_missed, 0);
    }
}

/// 第九节 M3 的敏感取样点与 PC-B2 的方向（修订 1 的直接构造）：缺口里删空 L3（退役记录死亡 txg 在缺口里），回退之后 L3 回来、
/// 版本还是缺口之前那一版；再删空最右的 L4（不左移）让退役容器在新父链上重写。B-on 的容器 m 是 0，B-off-lin 是 1。
#[test]
fn retirement_from_the_abandoned_segment_applies_only_when_lineage_is_ignored() {
    let (holds, detail) = direct_rollback_positive_control_holds();
    assert!(holds, "{detail}");
}

/// 第三段：P-1、u = 3、b = 1 上三个崩溃点；两路对得上，PC-O 两项与 PC-M 的期望成立；E-I 下崩溃点 extent m = 这一版剩下的单元数。
#[test]
fn crash_branches_count_what_the_rebuilt_view_misses() {
    let (prefix, object) = build_crash_prefix(CrashPrefix::FirstFileOnly, write_path_cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0)).expect("P-1 前缀");
    let outcomes = run_crash_branches(&prefix, object, 3, 1);
    assert_eq!(outcomes.len(), 3);
    let intent_effect = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever);
    let deferred_read = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtLastBatchRead, ReadableWindow::Forever);
    let missed: Vec<u64> = outcomes.iter().map(|outcome| outcome.crash_counts[intent_effect].extent_missed).collect();
    assert_eq!(missed, vec![3, 2, 1]);
    for outcome in &outcomes {
        assert!(outcome.crash_paths_agree && outcome.recovery_paths_agree);
        assert_eq!(outcome.crash_counts[deferred_read].records_whose_intent_is_published, 1);
        assert_eq!(outcome.crash_counts[intent_effect].records_whose_intent_is_published, 0);
        for control in &outcome.positive_controls {
            assert!(control.paths_agree && control.closed_form_holds, "{}", control.line(""));
        }
    }
}
