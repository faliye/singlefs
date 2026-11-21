//! 几份测试共用的辅助：拼合成的产物文本与计数点、网格的形状与点、调用处读数的文本与逐行对账的断言。

use super::*;

/// 整份拷那一行（s1）在 U 轴三个点、两种挂载状态上的产物行，步数与 scale 各由一个式子给，可以取不同的值。
/// 第一段真产物的形态是步数恒为 0、元素数只在 scale 里（计数构建只插了 scale）。
pub(super) fn whole_copy_rows_text(
    steps_of_the_copy: fn(u64) -> u64,
    scale_of_the_copy: fn(u64) -> u64,
) -> String {
    let mut text = String::new();
    for (point, unit_area_slots) in [
        ("unit_area_2_pow_14", 1_u64 << 14),
        ("base", 1 << 17),
        ("unit_area_2_pow_24", 1 << 24),
    ] {
        let _ = writeln!(text, "E203RESULT name=point point={point} unit_area_slots={unit_area_slots} unlinks=10 data_units=10 files=10");
        for state in STATES {
            let _ = writeln!(
                text,
                "E203RESULT name=row point={point} state={state} kind=overwrite round=1 segment=caller row=whole_allocator_copy_before_the_judgement calls=1 steps={} scale={}",
                steps_of_the_copy(unit_area_slots),
                scale_of_the_copy(unit_area_slots)
            );
        }
    }
    text
}

pub(super) fn push_line(text: &mut String, rest: &str) {
    let _ = writeln!(text, "E203RESULT {rest}");
}

/// 合成的一个点：Δ0 的每个读数，只让两样随这个点变——每轮内容变了的叶数（两道之间那一段读）与根头线性找的调用次数（落点判之前读）。
pub(super) struct SyntheticPoint {
    name: &'static str,
    pub(super) unit_area_slots: u64,
    pub(super) unlinks: u64,
    pub(super) data_units: u64,
    pub(super) files: u64,
    changed_leaves: u64,
    root_header_calls: u64,
}

fn delta_zero_row_calls() -> [(&'static str, u64); 14] {
    [
        ("files_other_than_the_first_plan", 2),
        ("file_other_than_the_first_linear_find", 2),
        ("inodes_with_an_extent_entry_set", 2),
        ("extent_upper_segment_plan", 2),
        ("extent_roles_rewritten_scan", 2),
        ("root_header_linear_find", 8),
        ("checkpoint_cost_accounting_tree_plan", 2),
        ("cluster_segment_sum", 4),
        ("inode_number_watermark_steps", 4),
        ("allocator_release_linear_find", 10),
        ("user_data_slot_linear_scan", 2),
        ("record_intersection_filter", 40),
        ("empty_segment_linear_scan", 4),
        ("accounting_entries_of_this_publish", 2),
    ]
}

/// 每个点、两种挂载状态、三种发布都写同一套 Δ0 读数（登记第七节 ③3），B8 的式子从这段文字里读得出输入。
pub(super) fn synthetic_counts_text(points: &[SyntheticPoint]) -> String {
    let mut text = String::new();
    for point in points {
        push_line(
            &mut text,
            &format!(
                "name=point point={} unit_area_slots={} unlinks={} data_units={} files={}",
                point.name, point.unit_area_slots, point.unlinks, point.data_units, point.files
            ),
        );
        for state in STATES {
            for kind in KINDS {
                let at = format!("point={} state={state} kind={kind} round=1", point.name);
                push_line(&mut text, &format!("name=state {at} units=3000 mapped_units=1500 allocation_records=2000 other_files=1000 inode_leaf_containers=8 deadlist_entries=0 extent_upper_nodes=9 extent_upper_levels=2 accounting_tree_levels=2 deadlist_tree_levels=1 allocation_tree_levels=3 central_mapping_levels=2"));
                push_line(
                    &mut text,
                    &format!("name=publish {at} had_caller_segment=true fixed_point_rounds=2"),
                );
                for (row, calls) in delta_zero_row_calls() {
                    let calls = if row == "root_header_linear_find" {
                        point.root_header_calls
                    } else {
                        calls
                    };
                    push_line(&mut text, &format!("name=row {at} segment=before_landing row={row} calls={calls} steps=1"));
                }
                push_line(&mut text, &format!("name=row {at} segment=between row=release_judgement_version_indexes calls=1 query_levels=138"));
                let mut deltas: Vec<(&str, u64, &str, u64)> = vec![
                    ("before_landing", 0, "touched_files", 1),
                    ("before_landing", 0, "lower_nodes_of_touched_files", 1),
                    (
                        "before_landing",
                        0,
                        "accounting_rows_written_by_the_built_on_publish",
                        15,
                    ),
                    ("between", 0, "deadlist_entries_changed", 0),
                    ("between", 0, "moved_entries", 0),
                    ("between", 0, "intersecting_records", 40),
                ];
                for round in [1, 2] {
                    deltas.push(("between", round, "changed_leaves", point.changed_leaves));
                    deltas.push(("between", round, "round_placements_released", 29));
                    deltas.push(("between", round, "round_roles_rewritten", 1));
                    deltas.push(("between", round, "container_units_rewritten", 0));
                    for keys_changed in [15, 0, 2] {
                        deltas.push(("between", round, "code_two_keys_changed", keys_changed));
                    }
                    for _ in 0..2 {
                        deltas.push(("between", round, "dirty_records", 40));
                    }
                }
                for (segment, fixed_point_round, key, value) in deltas {
                    push_line(&mut text, &format!("name=delta {at} segment={segment} fixed_point_round={fixed_point_round} key={key} value={value}"));
                }
            }
        }
    }
    text
}

/// 单元区轴上的两个点：低端 2¹⁴、高端 2²⁴，高端的两样随点变的读数由调用方给。
pub(super) fn synthetic_unit_area_points(
    changed_leaves_at_the_high_end: u64,
    root_header_calls_at_the_high_end: u64,
) -> Vec<SyntheticPoint> {
    vec![
        SyntheticPoint {
            name: "unit_area_2_pow_14",
            unit_area_slots: 1 << 14,
            unlinks: 10,
            data_units: 10,
            files: 10,
            changed_leaves: 4,
            root_header_calls: 8,
        },
        SyntheticPoint {
            name: "unit_area_2_pow_24",
            unit_area_slots: 1 << 24,
            unlinks: 10,
            data_units: 10,
            files: 10,
            changed_leaves: changed_leaves_at_the_high_end,
            root_header_calls: root_header_calls_at_the_high_end,
        },
    ]
}

pub(super) fn report_of_counts(text: &str) -> String {
    let mut out = String::new();
    full_report(&read_product(text), None, &mut out);
    out
}

/// 合成的一张格子：只有整份拷分配器那一行有步数与计时，十个点覆盖 U、u、D、F 四根轴（Q4 与 Q5 要每根轴至少三个点）。
pub(super) struct GridShape {
    /// 每一步的纳秒（整数）。
    pub(super) nanos_per_step: u64,
    /// 单元区轴高端那一点（2²⁴）的独占计时乘几倍。
    pub(super) high_end_factor: u64,
    /// 单元区轴高端那一点的段挂钟比独占计时多多少纳秒（别的点多 5 纳秒）。
    pub(super) high_end_residual: u64,
    /// 重挂载之后那一种状态里这一行的步数取 0。
    pub(super) remounted_has_no_steps: bool,
}

/// (点名, U, u, D, F)
pub(super) fn grid_points() -> Vec<(&'static str, u64, u64, u64, u64)> {
    vec![
        ("base", 1 << 17, 10, 10, 10),
        ("unit_area_2_pow_14", 1 << 14, 10, 10, 10),
        ("unit_area_2_pow_20", 1 << 20, 10, 10, 10),
        ("unit_area_2_pow_24", 1 << 24, 10, 10, 10),
        ("unlinks_100", 1 << 17, 100, 10, 10),
        ("unlinks_1000", 1 << 17, 1000, 10, 10),
        ("data_units_100", 1 << 17, 10, 100, 10),
        ("data_units_1000", 1 << 17, 10, 1000, 10),
        ("files_100", 1 << 17, 10, 10, 100),
        ("files_1000", 1 << 17, 10, 10, 1000),
    ]
}

pub(super) fn grid_text(shape: &GridShape, points: &[(&'static str, u64, u64, u64, u64)]) -> String {
    let mut text = String::new();
    for (name, unit_area_slots, unlinks, data_units, files) in points {
        push_line(
            &mut text,
            &format!("name=point point={name} unit_area_slots={unit_area_slots} unlinks={unlinks} data_units={data_units} files={files}"),
        );
        for state in STATES {
            for round in 1..=ROUNDS {
                let at = format!("point={name} state={state} kind=overwrite round={round}");
                let steps = if state == "remounted" && shape.remounted_has_no_steps {
                    0
                } else {
                    2 * elements_per_disk(*unit_area_slots)
                };
                let at_the_high_end = *name == "unit_area_2_pow_24";
                let factor = if at_the_high_end {
                    shape.high_end_factor
                } else {
                    1
                };
                let exclusive = steps * shape.nanos_per_step * factor;
                let residual = if at_the_high_end {
                    shape.high_end_residual
                } else {
                    5
                };
                push_line(&mut text, &format!("name=row {at} segment=caller row=whole_allocator_copy_before_the_judgement calls=1 steps={steps} scale={steps} exclusive_nanos={exclusive}"));
                push_line(
                    &mut text,
                    &format!(
                        "name=segment_nanos {at} segment=caller nanos={}",
                        exclusive + residual
                    ),
                );
            }
        }
    }
    text
}

/// 修订 4 那十行对账用的一格（base、同一次挂载、第 1 轮）：这一行两道之间那一段的四个读数。
fn call_site_text(
    kind: &str,
    row: &str,
    steps: u64,
    steps_to_the_hit: u64,
    whole_table_lengths: u64,
    lengths_to_the_hit: u64,
) -> String {
    let at = format!("point=base state=same_mount kind={kind} round=1");
    let mut text = String::from("E203RESULT name=point point=base unit_area_slots=131072 unlinks=10 data_units=10 files=10\n");
    push_line(&mut text, &format!("name=state {at} devices=2"));
    push_line(
        &mut text,
        &format!("name=row {at} segment=between row={row} calls=2 steps={steps} scale={whole_table_lengths} steps_to_the_hit={steps_to_the_hit} scale_to_the_hit={lengths_to_the_hit}"),
    );
    text
}

/// 对账之后这一行那一条 `name=three_two_row` 的「核了几格、不符几格、超上界几格」。
fn row_totals_of(text: &str, row: &str) -> (u64, u64, u64) {
    let mut out = String::new();
    reconciliation_report(&read_product(text), &mut out);
    let line = out
        .lines()
        .find(|line| line.starts_with(&format!("name=three_two_row row={row} ")))
        .unwrap_or_else(|| panic!("没有 {row} 那一行：{out}"));
    let fields = parse_line(&format!("E203RESULT {line}")).expect("这一行是 name= 起头的");
    let read = |key: &str| number(&fields, key).unwrap_or_else(|| panic!("{key}：{line}"));
    (
        read("cells_checked"),
        read("disagreements"),
        read("over_the_bound"),
    )
}

/// 修订 4 的一行：全表扫那一部分等于各次调用处表长之和时核过、差一（多一、少一）时不符；走到命中为止那一部分恰好等于上界时核过、
/// 在上界以内核过、超一时不符并记超上界；这一行没有的那一部分读到了表长就不符（读数插错了地方）。
pub(super) fn assert_the_call_site_row_reconciles(
    kind: &str,
    row: &str,
    whole_table_lengths: u64,
    lengths_to_the_hit: u64,
) {
    let (whole, bound) = (whole_table_lengths, lengths_to_the_hit);
    let cell = |steps: u64, to_the_hit: u64, whole_read: u64, bound_read: u64| {
        row_totals_of(
            &call_site_text(kind, row, steps, to_the_hit, whole_read, bound_read),
            row,
        )
    };
    assert_eq!(
        cell(whole + bound, bound, whole, bound),
        (1, 0, 0),
        "{row}：恰好等于表长与上界该核过"
    );
    assert_eq!(
        cell(whole + bound + 1, bound, whole, bound),
        (1, 1, 0),
        "{row}：全表扫那一部分多一步该记不符"
    );
    if whole > 0 {
        assert_eq!(
            cell(whole + bound - 1, bound, whole, bound),
            (1, 1, 0),
            "{row}：全表扫那一部分少一步该记不符"
        );
    }
    assert_eq!(
        cell(whole + bound + 1, bound + 1, whole, bound),
        (1, 1, 1),
        "{row}：超上界一步该记不符、记超上界"
    );
    if bound > 0 {
        assert_eq!(
            cell(whole + bound - 1, bound - 1, whole, bound),
            (1, 0, 0),
            "{row}：上界以内该核过"
        );
    } else {
        assert_eq!(
            cell(whole, 0, whole, 1),
            (1, 1, 0),
            "{row}：没有走到命中为止那一部分，读到它的表长该记不符"
        );
    }
    if whole == 0 {
        assert_eq!(
            cell(bound + 1, bound, 1, bound),
            (1, 1, 0),
            "{row}：没有全表扫那一部分，读到它的表长该记不符"
        );
    }
}

/// 单元区轴上三个点：低端、B0、高端；别的三根轴只有 B0 一个点。
pub(super) fn user_scale_points() -> Vec<SyntheticPoint> {
    let mut points = synthetic_unit_area_points(4, 8);
    points.push(SyntheticPoint {
        name: "base",
        unit_area_slots: 1 << 17,
        unlinks: 10,
        data_units: 10,
        files: 10,
        changed_leaves: 4,
        root_header_calls: 8,
    });
    points
}
