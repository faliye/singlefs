use super::*;

/// 一份格子文字里，点名那一点的 `name=row` 行步数翻倍（别的字段不动）。
fn grid_text_with_the_steps_of_one_point_doubled(text: &str, point: &str) -> String {
    let marker = format!(" point={point} ");
    let mut changed = String::new();
    for line in text.lines() {
        if line.starts_with("E203RESULT name=row ") && line.contains(&marker) {
            let words: Vec<String> = line
                .split(' ')
                .map(|word| match word.strip_prefix("steps=") {
                    Some(value) => format!(
                        "steps={}",
                        2 * value.parse::<u64>().expect("合成行的步数是整数")
                    ),
                    None => word.to_string(),
                })
                .collect();
            push_line(
                &mut changed,
                words
                    .join(" ")
                    .strip_prefix("E203RESULT ")
                    .expect("合成行以 E203RESULT 起头"),
            );
        } else {
            changed.push_str(line);
            changed.push('\n');
        }
    }
    changed
}

#[test]
fn the_extrapolation_check_covers_every_axis_with_three_points_and_the_diagonal() {
    let shape = GridShape {
        nanos_per_step: 1,
        high_end_factor: 1,
        high_end_residual: 5,
        remounted_has_no_steps: false,
    };
    let mut points = grid_points();
    points.push(("diagonal", 1 << 20, 10_000, 10_000, 1000));
    let text = grid_text(&shape, &points);
    let mut matching = String::new();
    full_report(
        &read_product(&text),
        Some(&read_product(&text)),
        &mut matching,
    );
    for axis in ["U", "u", "D", "F"] {
        assert!(
            matching.contains(&format!("name=q5 axis={axis} state=same_mount ")),
            "轴 {axis} 上有三个点：要有一格（第一段只核 U、u 时 D、F 两格漏掉，变异 m34）{matching}"
        );
    }
    let diagonal_steps = 2 * elements_per_disk(1 << 20);
    assert!(
        matching.contains(&format!("name=q5 axis=diagonal state=same_mount predicted_steps={diagonal_steps} measured_steps={diagonal_steps} nanos_ratios=1.000,1.000,1.000,1.000,1.000 passed=true")),
        "B0 加各轴的增量推 X1，只有 U 那一轴有增量：推的等于量的（变异 m35）{matching}"
    );
    assert!(
        matching.contains("name=verdict quantity=Q5 cells_passed=10 cells_failed=0"),
        "四根轴两种状态八格、对角点两格 {matching}"
    );
    let doubled = grid_text_with_the_steps_of_one_point_doubled(&text, "diagonal");
    let mut missed = String::new();
    full_report(
        &read_product(&doubled),
        Some(&read_product(&text)),
        &mut missed,
    );
    assert!(
        missed.contains(&format!("name=q5 axis=diagonal state=same_mount predicted_steps={diagonal_steps} measured_steps={} ", 2 * diagonal_steps)),
        "{missed}"
    );
    assert!(
        missed.contains("name=verdict quantity=Q5 cells_passed=8 cells_failed=2"),
        "X1 量的步数是推的两倍：对角点两格核不过 {missed}"
    );
}

#[test]
fn the_user_scale_value_takes_the_measured_unit_area_at_the_user_scale() {
    let shape = GridShape {
        nanos_per_step: 1,
        high_end_factor: 1,
        high_end_residual: 5,
        remounted_has_no_steps: false,
    };
    let mut points = grid_points();
    points.push((
        "user_scale_unit_area",
        user_scale_unit_area_slots(),
        10,
        10,
        10,
    ));
    let text = grid_text(&shape, &points);
    let measured_at_the_user_scale = 2 * elements_per_disk(user_scale_unit_area_slots());
    // X2 量的步数改成一个外推推不出来的数：取到的就是它
    let marked = text.replace(&format!("steps={measured_at_the_user_scale} "), "steps=7 ");
    let product = read_product(&marked);
    let steps = |at: &RoundKey| {
        to_f64(product.steps_of(
            at,
            &SEGMENTS_BEFORE_ADMISSION,
            "whole_allocator_copy_before_the_judgement",
        ))
    };
    let (value, missing) = user_scale_value(
        &product,
        "same_mount",
        "overwrite",
        1,
        &steps,
        &user_scale(10.0),
    );
    assert!(missing.is_empty(), "四根轴都有两个点：{missing:?}");
    assert!(
        (value - 7.0).abs() < 1e-6,
        "X2 量过：U 那一部分取 X2 的量，记录数三根轴的增量是 0（变异 m37：照旧只拿 ≤ 2²⁴ 的点外推时得 {}）；得 {value}",
        measured_at_the_user_scale
    );
}

/// Y1 的单元区加大到 2¹⁷ 那一档：(2¹⁷, 10⁵, 10, 10) 落在 u 轴的筛子里，外推只取登记 u 轴到 10⁴ 的点，
/// 多了它（且它量的步数翻倍）G-user 上的推值不变（变异 m70：记录数那三根轴不封顶时 u 那一部分多出增量）。
#[test]
fn the_many_unlinks_point_on_the_unit_area_of_the_base_stays_out_of_the_unlinks_axis_extrapolation() {
    let shape = GridShape {
        nanos_per_step: 1,
        high_end_factor: 1,
        high_end_residual: 5,
        remounted_has_no_steps: false,
    };
    let without = read_product(&grid_text(&shape, &grid_points()));
    let mut points = grid_points();
    points.push(("many_unlinks_unit_area_2_pow_17", 1 << 17, 100_000, 10, 10));
    let with = read_product(&grid_text_with_the_steps_of_one_point_doubled(
        &grid_text(&shape, &points),
        "many_unlinks_unit_area_2_pow_17",
    ));
    let value_of = |product: &Product| {
        let steps = |at: &RoundKey| {
            to_f64(product.steps_of(
                at,
                &SEGMENTS_BEFORE_ADMISSION,
                "whole_allocator_copy_before_the_judgement",
            ))
        };
        user_scale_value(
            product,
            "same_mount",
            "overwrite",
            1,
            &steps,
            &user_scale(10.0),
        )
    };
    let (value_without, missing_without) = value_of(&without);
    let (value_with, missing_with) = value_of(&with);
    assert_eq!(missing_without, missing_with);
    assert!(
        (value_with - value_without).abs() < 1e-6,
        "多了 Y1 那一点，G-user 上的推值不该变：没有它 {value_without}，有它 {value_with}"
    );
    assert_eq!(
        axis_points_for_extrapolation(&with, "u").len(),
        axis_points_for_extrapolation(&without, "u").len()
    );
    assert_eq!(
        axis_points(&with, "u").len(),
        axis_points(&without, "u").len() + 1,
        "筛子本身照旧收它（α 与外推各自封顶）"
    );
}

#[test]
fn the_user_scale_value_is_the_base_plus_each_axis_increment() {
    let counts = read_product(&synthetic_counts_text(&user_scale_points()));
    let total_of_the_point = |point: &str| -> Result<u64, Vec<String>> {
        let rows = by_delta_rows_by_segment(&counts, point, "same_mount", "overwrite")?;
        Ok(total_of(&rows, &SEGMENTS_BEFORE_ADMISSION))
    };
    let at_the_base = total_of_the_point("base").expect("B0 的输入都读得到");
    let at_the_high_end = total_of_the_point("unit_area_2_pow_24").expect("高端的输入都读得到");
    assert_eq!(
        at_the_high_end - at_the_base,
        6,
        "单元区从 2¹⁷ 到 2²⁴：每盘段数 2048 → 262144，分层位图层数 2 → 3，user_data_slot 2 次调用加 2 步、empty_segment 4 次调用加 4 步"
    );
    let (value, missing) = quantity_at_user_scale(
        &counts,
        "same_mount",
        "overwrite",
        &user_scale(268_435_456.0),
        &total_of_the_point,
    )
    .expect("每个点都读得到");
    let slope_over_the_gap =
        6.0 * (1_073_740_416.0 - 16_777_216.0) / (16_777_216.0 - 131_072.0);
    let expected = to_f64(at_the_high_end) + slope_over_the_gap;
    assert!(
        (value - expected).abs() < 1e-6,
        "G-user 上的值 = B0 的值加单元区轴上最后两点斜率外推的增量：{value} 对 {expected}"
    );
    assert_eq!(missing, ["u", "D", "F"], "只有 B0 一个点的轴外推不出，点名");
    let gaps = quantity_at_user_scale(
        &counts,
        "same_mount",
        "overwrite",
        &user_scale(268_435_456.0),
        &|point: &str| -> Result<u64, Vec<String>> {
            if point == "unit_area_2_pow_14" {
                Err(vec!["state.units".to_string()])
            } else {
                total_of_the_point(point)
            }
        },
    )
    .expect_err("一个点读不到，整格读不到");
    assert_eq!(gaps, ["unit_area_2_pow_14:state.units"]);
}
