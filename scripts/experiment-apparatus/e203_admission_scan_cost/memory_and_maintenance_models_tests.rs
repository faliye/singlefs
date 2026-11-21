use super::*;

fn delta_zero_memory() -> MemoryInput {
    MemoryInput {
        allocation_records: 2000,
        segments_per_disk: 262_144,
        units: 3000,
        mapped_units: 1500,
        files_in_the_index: 1001,
        extent_upper_nodes: 9,
        deadlist_entries: 0,
        accounting_rows_of_the_generation: 15,
        unit_bytes: 24,
        key_bytes: 24,
        accounting_row_bytes: 48,
    }
}

#[test]
fn by_delta_memory_and_maintenance_on_delta_zero_hit_the_registered_values() {
    let memory = by_delta_memory(&delta_zero_memory());
    assert_eq!(memory.segment_summaries, 199_728);
    assert_eq!(
        (
            memory.units_by_role,
            memory.mapped_units_by_role,
            memory.record_index
        ),
        (96_000, 48_000, 48_000)
    );
    assert_eq!(
        (
            memory.files_by_inode,
            memory.inodes_with_an_extent_entry,
            memory.accounting_rows_of_the_generation
        ),
        (16_016, 8008, 720)
    );
    assert_eq!(
        (memory.extent_upper_nodes, memory.root_head_positions),
        (72, 32)
    );
    assert_eq!(memory.payload(), 416_576);
    assert_eq!(2 * memory.payload(), 833_152);
    let maintenance = MaintenanceInput {
        records_changed: 30,
        segments_flipped: 4,
        units_delta: 20,
        mapped_delta: 2,
        ..MaintenanceInput::default()
    };
    assert_eq!(
        by_delta_maintenance(&delta_zero_memory(), &maintenance),
        619
    );
    assert_eq!((undo_log_bytes(60), undo_log_bytes(30)), (1440, 720));
}

#[test]
fn narrowed_takes_the_cheaper_function_in_the_second_segment_too() {
    let mut today: StepsByRow = BTreeMap::new();
    let mut by_delta: StepsByRow = BTreeMap::new();
    for (row, today_steps, by_delta_steps) in [
        ("root_header_linear_find", 100, 8),
        ("checkpoint_cost_accounting_tree_plan", 200, 60),
        ("inode_number_watermark_steps", 5, 4),
        ("allocator_release_linear_find", 7, 110),
    ] {
        today.insert(("between", row), today_steps);
        by_delta.insert(("between", row), by_delta_steps);
    }
    today.insert(("before_landing", "root_header_linear_find"), 40);
    by_delta.insert(("before_landing", "root_header_linear_find"), 4);
    today.insert(
        ("before_landing", "checkpoint_cost_accounting_tree_plan"),
        50,
    );
    by_delta.insert(
        ("before_landing", "checkpoint_cost_accounting_tree_plan"),
        10,
    );
    today.insert(("before_landing", "inode_number_watermark_steps"), 3);
    by_delta.insert(("before_landing", "inode_number_watermark_steps"), 2);
    let called: BTreeSet<&'static str> = [
        "root_header_linear_find",
        "checkpoint_cost_accounting_tree_plan",
        "inode_number_watermark_steps",
    ]
    .into_iter()
    .collect();
    let [caller_steps, landing_steps, between_steps] =
        narrowed_steps_by_segment(&today, &by_delta, &called);
    assert_eq!(
        (caller_steps, landing_steps),
        (0, 16),
        "第一段与调用方段每行取 B8 一列"
    );
    let today_between_steps: u64 = 100 + 200 + 5 + 7;
    assert_eq!(
        between_steps,
        today_between_steps - (100 - 8) - (200 - 60) - (5 - 4),
        "S2 里在 P0 或 S1 里调到的行也取 B8 一列（变异 m6），s25 只在 S2 调到、照 B1"
    );
}

#[test]
fn the_positive_controls_see_the_whole_copy_and_the_negative_control_sees_the_constant() {
    let low = degraded_by_delta_total(1000, 0, 100_864);
    let high = degraded_by_delta_total(1000, 0, 103_284_736);
    assert!(
        positive_control_passes(low, high, 0.9),
        "退化后 s24 随 U 一次涨：α 不小于 0.9（变异 m12 漏退 s24 时红）"
    );
    assert!(
        !positive_control_passes(1000, 1000, 0.9),
        "不随 U 的合计过不了阳性对照"
    );
    assert!(negative_control_passes(8, 8));
    assert!(!negative_control_passes(8, 80_000));
}
