//! 各臂（df 字面、可用、载荷、写需求与总是多报的对照）的读数与按臂的判定的单测。

use super::*;

#[test]
fn the_non_data_demand_bound_adds_the_tree_nodes_the_fixed_slots_and_the_checkpoint_cost() {
    // n ≤ 1：下段那一项取 0，inode 树的内部层数这里也是 0，只剩上段叶 1、inode 叶容器 2 与检查点 10。
    assert_eq!(non_data_demand_upper_bound_slots(1, 0, 10), 1 + 2 + 10);
    // n = 144：下段叶 1 个，没有更高层。
    assert_eq!(
        non_data_demand_upper_bound_slots(144, 0, 10),
        1 + 1 + 2 + 10
    );
    // n = 145：下段叶 2 个，内部常规层 个。
    assert_eq!(
        non_data_demand_upper_bound_slots(145, 0, 10),
        (2 + 1) + 1 + 2 + 10
    );
    // n = 144 × 147 + 1：叶 148 个，内部大负载层 个，再 1 个根。
    assert_eq!(
        non_data_demand_upper_bound_slots(144 * 147 + 1, 1, 0),
        (148 + 2 + 1) + 1 + 2 + 1
    );
}

#[test]
fn the_write_demand_search_returns_the_largest_unit_count_whose_demand_fits() {
    let available = 1000;
    let largest_units = largest_units_satisfying_the_write_demand(available, 0, 10);
    assert!(
        2 * largest_units + non_data_demand_upper_bound_slots(largest_units, 0, 10) <= available
    );
    assert!(
        2 * (largest_units + 1) + non_data_demand_upper_bound_slots(largest_units + 1, 0, 10)
            > available
    );
    assert_eq!(
        largest_units_satisfying_the_write_demand(5, 0, 10),
        0,
        "连固定部分都装不下"
    );
}

#[test]
fn the_judgement_marks_an_over_report_by_the_units_asked_for() {
    let exact = WritableUnits::Exact {
        units: 10,
        non_monotone_probes_seen: false,
    };
    assert_eq!(
        judge(10 * DATA_UNIT_PAYLOAD_BYTES, &exact),
        Judgement::DoesNotOverReport { under_bytes: 0 }
    );
    assert_eq!(
        judge(10 * DATA_UNIT_PAYLOAD_BYTES + 1, &exact),
        Judgement::OverReports { over_bytes: 1 }
    );
    assert_eq!(
        judge(0, &exact),
        Judgement::DoesNotOverReport {
            under_bytes: 10 * DATA_UNIT_PAYLOAD_BYTES
        }
    );
    let censored = WritableUnits::AtLeastBecauseOfTheJournalCap {
        lower_bound_units: 10,
        lower_bound_was_tightened: true,
    };
    assert_eq!(
        judge(10 * DATA_UNIT_PAYLOAD_BYTES, &censored),
        Judgement::CertainlyDoesNotOverReportBelowTheLowerBound
    );
    assert_eq!(
        judge(10 * DATA_UNIT_PAYLOAD_BYTES + 1, &censored),
        Judgement::Undetermined
    );
    assert_eq!(
        judge(1, &WritableUnits::Unreadable("x".to_string())),
        Judgement::Undetermined
    );
}
