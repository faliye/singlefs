use super::*;

#[test]
fn the_inclusion_check_the_tau_choice_and_the_cells_hold() {
    assert!(
        !round_is_valid(1001, 1000),
        "Σ 独占计时 > 段挂钟的一轮作废（变异 m8）"
    );
    assert!(round_is_valid(1000, 1000));
    assert_eq!(
        tau_nanos_per_step(&[(10, 100), (1000, 5000)]),
        5.0,
        "τ 取步数最大的那个点（变异 m9）"
    );
    assert_eq!(
        one_millisecond_cell(&[2.0e6, 3.0e6, 2.5e6, 9.0e6, 2.0e6]),
        FormulaCell::AtLeastOneMillisecond
    );
    assert_eq!(
        one_millisecond_cell(&[0.5e6, 0.1e6, 0.2e6, 0.4e6, 0.3e6]),
        FormulaCell::BelowOneMillisecond
    );
    assert_eq!(
        one_millisecond_cell(&[2.0e6, 0.1e6, 2.5e6, 9.0e6, 2.0e6]),
        FormulaCell::Unstable
    );
    assert!(extrapolation_is_checked(105.0, 100.0, 1.9));
    assert!(!extrapolation_is_checked(115.0, 100.0, 1.0));
    assert!(!extrapolation_is_checked(100.0, 100.0, 2.1));
    assert_eq!(
        linear_extrapolation(1000.0, 10.0, 10_000.0, 100.0, 100_000.0),
        1000.0
    );
}

#[test]
fn the_report_reads_the_rows_and_gives_the_growth_order_of_the_scan_sum() {
    let mut text = String::new();
    for (point, unit_area_slots) in [
        ("unit_area_2_pow_14", 1_u64 << 14),
        ("base", 1 << 17),
        ("unit_area_2_pow_24", 1 << 24),
    ] {
        let _ = writeln!(text, "E203RESULT name=point point={point} unit_area_slots={unit_area_slots} unlinks=10 data_units=10 files=10");
        let steps = 2 * elements_per_disk(unit_area_slots);
        let _ = writeln!(text, "E203RESULT name=row point={point} state=same_mount kind=overwrite round=1 segment=caller row=whole_allocator_copy_before_the_judgement calls=1 steps={steps} scale={steps}");
    }
    let mut out = String::new();
    counts_report(&read_product(&text), &mut out);
    let line = out
        .lines()
        .find(|line_text| {
            line_text.starts_with("name=q2 axis=U state=same_mount kind=overwrite")
        })
        .expect("U 轴一行");
    assert!(
        line.contains("alpha=1.000000") && line.contains("GrowsWithTheAxis"),
        "{line}"
    );
    assert!(
        out.contains("name=verdict quantity=three_one_copy_elements disagreements=0"),
        "{out}"
    );
    assert!(out.contains("name=constant_check key=slot_bytes model=16384 implementation=NOT_READ match=false"), "读不到不是 0");
}
