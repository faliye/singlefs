use super::*;

#[test]
fn the_ring_window_reads_a_unit_replaced_twenty_three_publishes_ago() {
    assert!(ReadableWindow::RingWindow.allows(22 + 1));
    assert!(!ReadableWindow::RingWindow.allows(23 + 1));
    assert!(ReadableWindow::Forever.allows(u64::MAX));
}

#[test]
fn the_parent_chain_skips_the_segment_a_rollback_abandoned() {
    let chain = ParentChain::of(30, &[(20, 10)]);
    assert!(chain.contains(30) && chain.contains(20) && chain.contains(10) && chain.contains(1));
    assert!(!chain.contains(15) && !chain.contains(19) && !chain.contains(11) && !chain.contains(31));
    assert!(ParentChain::of(9, &[(20, 10)]).contains(9));
}

/// A3 与第九节 M1、M2 的取样点：F8 n_u = 3、删前覆盖写，覆盖写之后那次扫描 Z、T 的 extent r 都是 0，G0 是 n_u − 1 = 2。
#[test]
fn after_an_overwrite_shrinks_a_file_only_the_object_gate_resurrects_its_tail() {
    let history = History::LargeFileRewriteAndDelete { data_units: 3, rewrites: 0, overwrite_before_unlink: true, batch: 144 };
    let (simulation, _) = simulate(history, write_path_cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0), true);
    let overwrite_scan = simulation
        .publish_records
        .iter()
        .zip(&simulation.scans)
        .find(|(record, _)| record.shrink.is_some())
        .map(|(_, scan)| scan.clone())
        .expect("覆盖写那次变短");
    for (reader, counts) in every_reader().iter().zip(&overwrite_scan.counts) {
        let expected = if reader.shrink == ShrinkArm::ObjectGateOnly { 2 } else { 0 };
        assert_eq!(counts.extent_resurrected, expected, "{}", reader.label());
        assert_eq!(counts.extent_missed, 0, "{}", reader.label());
    }
}

/// A7 与第九节 M8 的取样点：F8 n_u = 300、b = 144，E-I 下 unlink 途中三次扫描 inode m = 1、extent m = 300、156、12。
#[test]
fn the_intent_effect_misses_the_record_and_every_remaining_unit_in_the_window() {
    let history = History::LargeFileRewriteAndDelete { data_units: 300, rewrites: 0, overwrite_before_unlink: false, batch: 144 };
    let (simulation, _) = simulate(history, write_path_cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0), true);
    let intent_effect = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever);
    let deferred_read = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtLastBatchRead, ReadableWindow::Forever);
    let in_window: Vec<&ScanRecord> = simulation.scans.iter().filter(|scan| scan.inside_an_unlink).collect();
    let extent_missed: Vec<u64> = in_window.iter().map(|scan| scan.counts[intent_effect].extent_missed).collect();
    assert_eq!(extent_missed, vec![300, 156, 12]);
    assert!(in_window.iter().all(|scan| scan.counts[intent_effect].inode_missed == 1));
    assert!(in_window.iter().all(|scan| scan.counts[deferred_read].inode_missed == 0 && scan.counts[deferred_read].extent_missed == 0));
}

/// 第九节 M9 的取样点：H51 F1 n = 300，E-L-r 的途中 inode m 恒 0，E-I 的是 1。
#[test]
fn the_deferred_read_effect_keeps_the_record_while_it_is_still_in_the_tree() {
    let history = History::BuildAndDeleteEmptyInodes { inode_count: 300, order: DeletionOrder::Ascending };
    let (simulation, _) = simulate(history, write_path_cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0), true);
    let deferred_read = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtLastBatchRead, ReadableWindow::Forever);
    let intent_effect = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever);
    let trajectories = trajectories_of(&simulation.scans, every_reader().len());
    assert_eq!(trajectories[deferred_read].by_scope[SCOPE_INSIDE_UNLINK][QUANTITY_INODE_MISSED].peak, 0);
    assert_eq!(trajectories[intent_effect].by_scope[SCOPE_INSIDE_UNLINK][QUANTITY_INODE_MISSED].peak, 1);
    assert_eq!(trajectories[intent_effect].by_scope[SCOPE_INSIDE_UNLINK][QUANTITY_INODE_MISSED].positive_scans, 299);
}

/// 第九节 M13：同号多条时取写序最大的那个容器里的那条。植入一片写序最大、装 (1, b, size 0) 的叶容器，Z 判 inode 1 的单元死（extent m = 1）。
#[test]
fn duplicate_records_are_taken_from_the_container_with_the_newest_write_order() {
    let simulation = Simulation::start(write_path_cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0), PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    let record = simulation.version.record_of(FIRST_FILE_INODE).expect("新池新建文件");
    let planted = InodeRecord { inode: FIRST_FILE_INODE, object_birth: record.object_birth, nlink: 1, size_bytes: 0, change_counter: 0 };
    let overlay = Overlay::planting(vec![Simulation::planted_leaf(planted, simulation.newest_write_order(), simulation.publish_sequence)]);
    for path in [SelectionPath::GroupIndex, SelectionPath::FullScan] {
        let counts = simulation.read_every_reader(&overlay, simulation.publish_sequence, path);
        let record_size = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever);
        let object_gate = position_of(BaseArm::RetirementOnLineage, ShrinkArm::ObjectGateOnly, UnlinkEffectArm::AtIntent, ReadableWindow::Forever);
        assert_eq!(counts[record_size].extent_missed, 1, "{path:?}");
        assert_eq!(counts[object_gate].extent_missed, 0, "{path:?}");
        assert_eq!(counts[record_size].container_resurrected, 1, "{path:?}");
    }
}

/// 第十一节 V3：主路径（按组索引）与参照路径（整扫）在多对象带数据、合并与回退的历史末尾逐读者逐量相同。
#[test]
fn the_group_index_reads_the_same_as_a_full_scan() {
    for (history, write_path) in [
        (History::ManyObjectsWithData { inode_count: 300, order: DeletionOrder::Random(3), batch: 1 }, write_path_cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 4)),
        (History::RollBackOverAGap { event: GapEvent::Drop }, write_path_cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 0)),
        (History::MixedRandom { seed: 3 }, write_path_cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 8)),
    ] {
        let (simulation, _) = simulate(history, write_path, false);
        assert!(simulation.group_index_equals_the_full_scan(simulation.publish_sequence), "{}", history.label());
    }
}
