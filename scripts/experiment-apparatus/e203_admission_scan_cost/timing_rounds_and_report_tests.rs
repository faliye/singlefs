use super::*;

fn timing_product_text(wall_of_round_three: u64) -> String {
    let mut text = String::new();
    for (point, unit_area_slots, nanos) in [
        ("unit_area_2_pow_14", 1_u64 << 14, 1000_u64),
        ("unit_area_2_pow_24", 1 << 24, 1_000_000),
    ] {
        let _ = writeln!(text, "E203RESULT name=point point={point} unit_area_slots={unit_area_slots} unlinks=10 data_units=10 files=10");
        for round in 1..=5 {
            let at = format!("point={point} state=same_mount kind=overwrite round={round}");
            let _ = writeln!(text, "E203RESULT name=row {at} segment=caller row=whole_allocator_copy_before_the_judgement exclusive_nanos={nanos}");
            let wall = if round == 3 && nanos > 1000 {
                wall_of_round_three
            } else {
                nanos + 5
            };
            let _ = writeln!(
                text,
                "E203RESULT name=segment_nanos {at} segment=caller nanos={wall}"
            );
        }
    }
    text
}

#[test]
fn the_timing_report_bands_five_rounds_and_drops_a_round_that_breaks_the_inclusion_check() {
    let mut out = String::new();
    let timing = read_product(&timing_product_text(1_000_005));
    let residual_grows = timing_report(&timing, &[], &mut out);
    assert_eq!(residual_grows, Some(false), "残差只有 5 纳秒，不随轴涨");
    let line = out
        .lines()
        .find(|line| line.starts_with("name=q3 axis=U state=same_mount kind=overwrite"))
        .expect("U 轴一行");
    assert!(line.ends_with("band=GrowsWithTheAxis"), "{line}");
    let mut broken = String::new();
    let _ = timing_report(&read_product(&timing_product_text(10)), &[], &mut broken);
    let broken_line = broken
        .lines()
        .find(|broken_line| broken_line.starts_with("name=q3 axis=U state=same_mount kind=overwrite"))
        .expect("U 轴一行");
    assert!(
        broken_line.ends_with("band=fewer_than_five_valid_rounds"),
        "第 3 轮 Σ 独占计时大于段挂钟：作废，不足五轮 {broken_line}"
    );
    let mut controls = String::new();
    let counts = read_product(&timing_product_text(1_000_005));
    controls_report(&counts, Some(&timing), &mut controls);
    assert!(
        controls.contains("control=b1_timing_whole_copy_alpha_at_least_0_7_in_every_round state=same_mount passed=true"),
        "{controls}"
    );
}
