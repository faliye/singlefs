//! 阳性对照 PC5、PC3：设备内存与计时对照的单测。

use super::*;

#[test]
fn device_memory_parts_grow_by_the_synthetic_amounts() {
    let (stream, mut totals, small) =
        evaluate_the_first_stream_small_domain(2, TimingControl::WALL_CLOCK_WITHOUT_BUSY_WAIT);
    let table = TornWriteTable::of(&stream);
    let plan = LocalPlan::new(
        &stream,
        &table,
        domain_expansions(&stream, LocalExpansion::EveryCombination),
    );
    let intervals = TrajectoryIntervals::of(&plan, &small, &BTreeSet::new());
    let stores = std::mem::take(&mut totals.arm_stores);
    let mut output = ResultLines::new();
    let summary = report_cell(
        &mut output,
        "test-first-small",
        &totals,
        stores,
        &intervals,
        None,
    );
    assert!(
        report_device_memory_control(
            &mut output,
            "test-first-small",
            &mut totals.tables,
            &summary,
            table.mask_width()
        ),
        "PC5：P1 加 32768、P3 加 7 × 21、P2 加 ⌈W/8⌉、P4 加 48"
    );
    assert_eq!(mask_bytes_per_state(53), 7, "⌈53 / 8⌉");
}
