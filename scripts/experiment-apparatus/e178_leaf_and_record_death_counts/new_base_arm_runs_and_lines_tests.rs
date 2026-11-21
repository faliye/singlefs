use super::*;

/// A10、A11 与第九节修订 2 的 M20、M21：H51 F1 n = 300 升序、容量 16、θ = 0，B-stub 写 18 个桩、期末 18 个桩、叶片总数 19；
/// 第一个桩在第 71 次发布，有桩的版本 537 个（`anchors3.out`、`anchors4.out`）。
#[test]
fn stub_write_path_on_emptied_inodes_matches_the_closed_forms() {
    let history = History::BuildAndDeleteEmptyInodes { inode_count: 300, order: DeletionOrder::Ascending };
    let (simulation, _) = stub_path_simulation(history, write_path_cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 0), false);
    let stubs_written: u64 = simulation.publish_records.iter().map(|record| record.stubs_written).sum();
    assert_eq!((stubs_written, simulation.version.stub_count()), (17 + 1, 17 + 1));
    let last = simulation.publish_records.last().expect("有发布");
    assert_eq!((last.leaves_in_the_leaf_order, last.stubs_in_the_leaf_order), (18 + 1, 17 + 1));
    let versions_with_an_empty_leaf = simulation.publish_records.iter().filter(|record| record.holds_an_empty_inode_leaf).count();
    assert_eq!(versions_with_an_empty_leaf, 536 + 1);
    let first_stub = simulation.publish_records.iter().position(|record| record.stubs_in_the_leaf_order > 0).map(|position| position + 1);
    assert_eq!(first_stub, Some(70 + 1));
    assert_eq!(first_stub_publish_and_versions_with_a_stub(300, TEST_LEAF_CAPACITY), (70 + 1, 536 + 1));
    assert_eq!(first_stub_publish_and_versions_with_a_stub(900, INODE_LEAF_RECORDS), (950 + 1, 868 + 1));
    assert_eq!(simulation.total_written().retirement_records, 0);
    let wide = History::BuildAndDeleteEmptyInodes { inode_count: 900, order: DeletionOrder::EvenThenOdd };
    let (wide_simulation, _) = stub_path_simulation(wide, write_path_cell(BorrowArm::RaiseSeparator, INODE_LEAF_RECORDS, 0), false);
    assert_eq!((wide_simulation.version.stub_count(), length_as_u64(wide_simulation.version.leaves.len())), (2 + 1, 3 + 1));
}

/// V-Bs1 与第九节修订 2 的 M22：B-stub 写路径与原写路径逐版的活叶、数据单元、txg 与扫描相同（桩不当邻片）。
#[test]
fn stub_write_path_keeps_every_live_leaf_of_the_original_write_path() {
    for (history, write_path) in [
        (History::ManyObjectsWithData { inode_count: 300, order: DeletionOrder::Ascending, batch: 144 }, write_path_cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 8)),
        (History::ManyObjectsWithData { inode_count: 300, order: DeletionOrder::Random(2), batch: 1 }, write_path_cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 4)),
        (History::MixedRandom { seed: 3 }, write_path_cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 8)),
        (History::RollBackOverAGap { event: GapEvent::Absorb }, write_path_cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 8)),
    ] {
        let original = original_path_simulation(history, write_path, false);
        let (stub, _) = stub_path_simulation(history, write_path, false);
        assert!(pairing_holds(&original, &stub), "{}", history.label());
        assert!(stub.version.stub_count() > 0, "{} 一个桩都没有", history.label());
        assert!(stub.publish_records.iter().any(|record| matches!(record.merge, MergeAction::AbsorbedTheRightNeighbour | MergeAction::AbsorbedIntoTheLeftNeighbour)));
    }
}

/// PC-Bs2、PC-Bt3（修订 2 之六）与第九节 M15 补的那一格：构造期末 B-stub 0 / 0、B-stub-lin-off 1 / 1 / 0；B-tsz 不按谱系滤时 inode m + 1。
#[test]
fn the_rollback_construction_separates_the_lineage_of_stubs_and_tombstones() {
    let (stub_holds, stub_detail) = stub_lineage_positive_control();
    assert!(stub_holds, "{stub_detail}");
    let (tombstone_holds, tombstone_detail) = tombstone_lineage_positive_control();
    assert!(tombstone_holds, "{tombstone_detail}");
}

/// 第四段一个工作项（F10、N1、容量 16、θ = 4）：两条写路径配对、读到的桩数对得上、阳性对照两路一致且期望成立、A12 不红。
#[test]
fn segment_four_item_holds_its_pairing_controls_and_anchors() {
    let lines = segment_four_history_item_lines(
        History::ManyObjectsWithData { inode_count: 300, order: DeletionOrder::Ascending, batch: 144 },
        write_path_cell(BorrowArm::RaiseSeparator, TEST_LEAF_CAPACITY, 4),
    );
    let stub_simulation = lines.iter().find(|line| line.starts_with("name=simulation segment=four") && line.contains("path=stub")).expect("有 B-stub 写路径那一行");
    assert!(stub_simulation.contains("pairing_holds=true") && stub_simulation.contains("stub_read_mismatch_scans=0"), "{stub_simulation}");
    let controls: Vec<&String> = lines.iter().filter(|line| line.starts_with("name=positive_control")).collect();
    for expected in ["id=PC-Bs1", "id=PC-Bt2", "id=PC-Z", "id=PC-EL ", "id=PC-M-leaf"] {
        assert!(controls.iter().any(|line| line.contains(expected)), "{expected} 没做");
    }
    for line in &controls {
        assert!(line.contains("paths_agree=true") && line.contains("closed_form_holds=true"), "{line}");
    }
    for line in lines.iter().filter(|line| line.starts_with("name=item_assertion")) {
        assert!(line.ends_with("violated=0"), "{line}");
    }
    assert!(lines.iter().any(|line| line.starts_with("name=difference") && line.contains("base=B-stub")));
}
