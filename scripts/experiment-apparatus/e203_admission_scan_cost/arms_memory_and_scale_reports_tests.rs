use super::*;

/// 两道之间那一段一轮的挂钟：段挂钟 1000，没改的行 100，改过的行 200（B1 步数 10、B8 步数 2）。
#[test]
fn the_second_segment_nanos_take_the_improved_rows_at_the_per_step_price() {
    let at = "point=base state=same_mount kind=overwrite round=1";
    let timing = read_product(&format!(
        "E203RESULT name=row {at} segment=between row=fallback_slot_linear_scan exclusive_nanos=100\n\
             E203RESULT name=row {at} segment=between row=root_header_linear_find exclusive_nanos=200\n\
             E203RESULT name=segment_nanos {at} segment=between nanos=1000\n"
    ));
    let mut today: StepsByRow = BTreeMap::new();
    let mut by_delta: StepsByRow = BTreeMap::new();
    today.insert(("between", "root_header_linear_find"), 10);
    by_delta.insert(("between", "root_header_linear_find"), 2);
    let called: BTreeSet<&'static str> = ["root_header_linear_find"].into_iter().collect();
    let at_key = round_key("base", "same_mount", "overwrite", 1);
    assert_eq!(
        narrowed_between_nanos(&timing, &at_key, &by_delta, &today, &called),
        Some(700.0 + 100.0 + 40.0),
        "残差 700 + 没改的行 100 + 改过的行 2 步 × 每步 20 纳秒"
    );
    let broken = read_product(&format!(
        "E203RESULT name=row {at} segment=between row=fallback_slot_linear_scan exclusive_nanos=2000\n\
             E203RESULT name=segment_nanos {at} segment=between nanos=1000\n"
    ));
    assert_eq!(
        narrowed_between_nanos(&broken, &at_key, &by_delta, &today, &called),
        None,
        "Σ 独占计时大于段挂钟的一轮不能用"
    );
}

fn timing_text_of_the_second_segment(rounds: u64) -> String {
    let mut text = String::new();
    for point in ["base", "unit_area_2_pow_14", "unit_area_2_pow_24"] {
        for round in 1..=rounds {
            let at = format!("point={point} state=same_mount kind=overwrite round={round}");
            push_line(&mut text, &format!("name=row {at} segment=between row=fallback_slot_linear_scan exclusive_nanos=100"));
            push_line(&mut text, &format!("name=row {at} segment=between row=root_header_linear_find exclusive_nanos=200"));
            push_line(
                &mut text,
                &format!("name=segment_nanos {at} segment=between nanos=1000"),
            );
        }
    }
    text
}

#[test]
fn the_second_segment_nanos_at_the_user_scale_need_five_valid_rounds() {
    let counts = read_product(&synthetic_counts_text(&user_scale_points()));
    let scale = user_scale(268_435_456.0);
    let five = narrowed_between_nanos_at_user_scale(
        &counts,
        &read_product(&timing_text_of_the_second_segment(5)),
        "same_mount",
        &scale,
    );
    assert_eq!(
        five,
        "nanos_of_each_round=800,800,800,800,800 axes_not_extrapolated=u,D,F"
    );
    let four = narrowed_between_nanos_at_user_scale(
        &counts,
        &read_product(&timing_text_of_the_second_segment(4)),
        "same_mount",
        &scale,
    );
    assert_eq!(four, "fewer_than_five_valid_rounds");
}
