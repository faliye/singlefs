use super::*;

#[test]
fn the_ring_anchor_reclaims_the_genesis_tree_table_at_txg_26_and_reuses_it_at_txg_27() {
    let anchor = run_ring_anchor();
    for line in &anchor.lines {
        assert!(line.ends_with("holds=true"), "{line}");
    }
    assert!(anchor.every_anchor_holds);
    assert_eq!(anchor.lines.len(), 8);
}

fn cell_with_peak(resurrected_peak: u64) -> CellSummary {
    CellSummary {
        geometry: Geometry::Wide,
        arm: Arm::WithoutTombstones,
        reader: Reader::SelectCurrent,
        tree: ReportedTree::Inode,
        family: "F1",
        is_overwrite_before_unlink: false,
        trajectory: Trajectory {
            resurrected_peak,
            deaths_peak: 5,
            ..Trajectory::default()
        },
        tree_has_units: true,
        is_excluded: false,
    }
}

#[test]
fn moving_the_threshold_between_zero_and_the_peak_flips_the_status() {
    let cell = cell_with_peak(1);
    let tally = PositiveControlTally {
        matched: 1,
        mismatched: 0,
        not_plantable: 0,
    };
    assert_eq!(status_of(&[&cell], tally, RESURRECTION_THRESHOLD), TreeStatus::Resurrected);
    assert_eq!(status_of(&[&cell], tally, 1), TreeStatus::MeasuredZero);
    let quiet = cell_with_peak(0);
    assert_eq!(status_of(&[&quiet], tally, RESURRECTION_THRESHOLD), TreeStatus::MeasuredZero);
    assert_eq!(status_of(&[&quiet], PositiveControlTally::default(), RESURRECTION_THRESHOLD), TreeStatus::ZeroWithoutPositiveControl);
}
