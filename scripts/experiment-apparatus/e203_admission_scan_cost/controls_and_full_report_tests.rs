use super::*;

#[test]
fn the_failure_clause_four_fires_on_a_publish_count_other_than_two() {
    let text = "E203RESULT name=point point=base unit_area_slots=131072 unlinks=10 data_units=10 files=10\n\
                    E203RESULT name=unlink_publishes point=base state=same_mount round=1 publishes=3\n";
    let mut out = String::new();
    full_report(&read_product(text), None, &mut out);
    assert!(out.contains("name=verdict clause=F4 fired=true"), "{out}");
}

#[test]
fn a_point_with_a_frozen_publish_is_voided_whole_and_named() {
    let flat = synthetic_counts_text(&synthetic_unit_area_points(4, 8));
    let clean = report_of_counts(&flat);
    assert!(
        clean.contains("name=verdict clause=void_frozen_publish points_voided_in_counts=0 points_voided_in_timing=0 cells_without_the_reading_in_counts=12 cells_without_the_reading_in_timing=0"),
        "{clean}"
    );
    let frozen = flat.replace(
        "name=state point=unit_area_2_pow_24 state=same_mount kind=overwrite round=1 ",
        "name=state point=unit_area_2_pow_24 state=same_mount kind=overwrite round=1 frozen_publish_set=1 ",
    );
    assert_ne!(frozen, flat, "合成产物里要有这一格的状态行");
    let voided = report_of_counts(&frozen);
    assert!(
        voided.contains("name=void_frozen_publish build=counts point=unit_area_2_pow_24 result=point_voided"),
        "{voided}"
    );
    assert!(
        voided.contains("name=verdict clause=void_frozen_publish points_voided_in_counts=1 points_voided_in_timing=0 cells_without_the_reading_in_counts=11"),
        "{voided}"
    );
    // 整点作废：这个点的读数不进任何量（别的点照常）
    assert!(!voided.contains("name=by_delta_pooled_fallback point=unit_area_2_pow_24 "), "{voided}");
    assert!(voided.contains("name=by_delta_pooled_fallback point=unit_area_2_pow_14 "), "{voided}");
    // 作废的点也不留在点轴里：点轴上的点数 2 变 1（变异：作废的点留在轴里时红）
    assert!(clean.contains("name=verdict clause=void_6 points=2 points_with_the_done_marker="), "{clean}");
    assert!(voided.contains("name=verdict clause=void_6 points=1 points_with_the_done_marker="), "{voided}");
}

#[test]
fn failure_clauses_two_and_five_each_read_their_own_arm() {
    let flat = report_of_counts(&synthetic_counts_text(&synthetic_unit_area_points(4, 8)));
    assert!(
        flat.contains("name=verdict quantity=Q6 result=DoesNotGrowOnAnyAxis"),
        "{flat}"
    );
    assert!(
        flat.contains("name=verdict quantity=Q8_b4_arm result=DoesNotGrowOnAnyAxis"),
        "{flat}"
    );
    assert!(
        flat.contains("name=verdict clause=F2 fired=false"),
        "{flat}"
    );
    assert!(
        flat.contains("name=verdict clause=F5 fired=false"),
        "{flat}"
    );
    let grows_between = report_of_counts(&synthetic_counts_text(&synthetic_unit_area_points(
        40_960, 8,
    )));
    assert!(
        grows_between.contains("name=verdict clause=F2 fired=true"),
        "B8 的步数随单元区涨（两道之间那一段）：F2 触发 {grows_between}"
    );
    assert!(
        grows_between.contains("name=verdict clause=F5 fired=false"),
        "落点判之前剩下的步数不涨：F5 不触发（变异：F5 读 B8 的总结时红）{grows_between}"
    );
    let grows_before_landing = report_of_counts(&synthetic_counts_text(
        &synthetic_unit_area_points(4, 81_920),
    ));
    assert!(
        grows_before_landing.contains("name=verdict clause=F2 fired=true"),
        "{grows_before_landing}"
    );
    assert!(
        grows_before_landing.contains("name=verdict clause=F5 fired=true"),
        "落点判之前的根头线性找随单元区涨：F5 触发 {grows_before_landing}"
    );
    assert!(
        grows_before_landing
            .contains("name=q6_user_scale state=same_mount kind=overwrite data_units_user=10 "),
        "{grows_before_landing}"
    );
}

/// 计数产物与计时产物读同一份文字（两种产物的字段不相交，互不干扰）。
fn report_of_the_grid(
    shape: &GridShape,
    points: &[(&'static str, u64, u64, u64, u64)],
) -> String {
    let text = grid_text(shape, points);
    let mut out = String::new();
    full_report(&read_product(&text), Some(&read_product(&text)), &mut out);
    out
}

/// 同 [`report_of_the_grid`]，只是独占计时是 0 的行改成 1 纳秒：Q5 那几格才有挂钟可比、核得过，
/// F1 才判（F6 打中时 F1 不判）。Q4 照旧：步数是 0 的那一种推出来仍是 0，一纳秒一步的那一种仍远小于 0.5 毫秒。
fn report_of_the_grid_timed_at_least_one_nanosecond(
    shape: &GridShape,
    points: &[(&'static str, u64, u64, u64, u64)],
) -> String {
    let text = grid_text(shape, points).replace(" exclusive_nanos=0\n", " exclusive_nanos=1\n");
    let mut out = String::new();
    full_report(&read_product(&text), Some(&read_product(&text)), &mut out);
    assert!(
        out.contains("name=verdict clause=F6 fired=false"),
        "这张格子的外推要核得过 {out}"
    );
    out
}

#[test]
fn failure_clause_one_fires_only_when_all_four_cells_reach_one_millisecond() {
    let all_reach = report_of_the_grid(
        &GridShape {
            nanos_per_step: 1,
            high_end_factor: 1,
            high_end_residual: 5,
            remounted_has_no_steps: false,
        },
        &grid_points(),
    );
    assert!(
        all_reach.contains("name=verdict quantity=Q4 cells=AtLeastOneMillisecond,AtLeastOneMillisecond,AtLeastOneMillisecond,AtLeastOneMillisecond"),
        "{all_reach}"
    );
    assert!(
        all_reach.contains("name=verdict clause=F1 fired=true"),
        "{all_reach}"
    );
    let none_reach = report_of_the_grid_timed_at_least_one_nanosecond(
        &GridShape {
            nanos_per_step: 0,
            high_end_factor: 1,
            high_end_residual: 5,
            remounted_has_no_steps: false,
        },
        &grid_points(),
    );
    assert!(
        none_reach.contains("name=verdict quantity=Q4 cells=BelowOneMillisecond,BelowOneMillisecond,BelowOneMillisecond,BelowOneMillisecond"),
        "{none_reach}"
    );
    assert!(
        none_reach.contains("name=verdict clause=F1 fired=false"),
        "{none_reach}"
    );
    let mixed = report_of_the_grid_timed_at_least_one_nanosecond(
        &GridShape {
            nanos_per_step: 1,
            high_end_factor: 1,
            high_end_residual: 5,
            remounted_has_no_steps: true,
        },
        &grid_points(),
    );
    assert!(
        mixed.contains("name=verdict quantity=Q4 cells=AtLeastOneMillisecond,AtLeastOneMillisecond,BelowOneMillisecond,BelowOneMillisecond"),
        "{mixed}"
    );
    assert!(
        mixed.contains("name=verdict clause=F1 fired=false"),
        "四格里有「< 1 毫秒」又有「≥ 1 毫秒」：几何敏感，F1 不触发（变异：all 改 any 时红）{mixed}"
    );
}

#[test]
fn failure_clause_three_fires_when_the_residual_grows_in_all_five_rounds() {
    let small_residual = report_of_the_grid(
        &GridShape {
            nanos_per_step: 1,
            high_end_factor: 1,
            high_end_residual: 5,
            remounted_has_no_steps: false,
        },
        &grid_points(),
    );
    assert!(
        small_residual.contains("name=verdict clause=F3 fired=false"),
        "{small_residual}"
    );
    let growing_residual = report_of_the_grid(
        &GridShape {
            nanos_per_step: 1,
            high_end_factor: 1,
            high_end_residual: 5_000_000,
            remounted_has_no_steps: false,
        },
        &grid_points(),
    );
    assert!(
        growing_residual.contains("name=verdict clause=F3 fired=true"),
        "{growing_residual}"
    );
}

/// 计时产物的真形态：不打 `name=point` 行，轴值只在计数产物里。
fn without_the_point_lines(text: &str) -> String {
    text.lines()
        .filter(|line| !line.starts_with("E203RESULT name=point "))
        .map(|line| format!("{line}\n"))
        .collect()
}

fn report_of_the_grid_with_the_timing_without_points(
    shape: &GridShape,
    counts_have_the_points: bool,
) -> String {
    let text = grid_text(shape, &grid_points());
    let timing_text = without_the_point_lines(&text);
    let counts_text = if counts_have_the_points {
        text
    } else {
        timing_text.clone()
    };
    let mut out = String::new();
    full_report(
        &read_product(&counts_text),
        Some(&read_product(&timing_text)),
        &mut out,
    );
    out
}

#[test]
fn failure_clause_three_is_unknown_when_no_counts_product_gives_the_points() {
    let shape = GridShape {
        nanos_per_step: 1,
        high_end_factor: 1,
        high_end_residual: 5,
        remounted_has_no_steps: false,
    };
    let out = report_of_the_grid_with_the_timing_without_points(&shape, false);
    assert!(
        out.contains("name=verdict clause=F3 fired=unknown"),
        "计时与计数两边都没有点的轴值：F3 不判，不报没打中 {out}"
    );
    assert!(
        out.contains(
            "name=timing_point_axes points_of_the_timing=10 taken_from_the_counts=0 unread=10"
        ),
        "{out}"
    );
    assert!(
        out.contains("name=q10 point=unit_area_2_pow_24 result=axes_unread"),
        "{out}"
    );
    assert!(!out.contains("name=q10 axis="), "一根轴都算不成 {out}");
}

#[test]
fn failure_clause_three_is_judged_on_the_point_axes_of_the_counts_product() {
    let mut shape = GridShape {
        nanos_per_step: 1,
        high_end_factor: 1,
        high_end_residual: 5,
        remounted_has_no_steps: false,
    };
    let small_residual = report_of_the_grid_with_the_timing_without_points(&shape, true);
    assert!(
        small_residual.contains("name=verdict clause=F3 fired=false"),
        "{small_residual}"
    );
    assert!(
        small_residual.contains(
            "name=timing_point_axes points_of_the_timing=10 taken_from_the_counts=10 unread=0"
        ),
        "{small_residual}"
    );
    assert!(
        small_residual
            .contains("name=q10 axis=U state=same_mount kind=overwrite segment=caller "),
        "Q10 照打 {small_residual}"
    );
    assert!(
        small_residual.contains("name=q3 axis=U state=same_mount kind=overwrite "),
        "Q3 照打 {small_residual}"
    );
    shape.high_end_residual = 5_000_000;
    let growing_residual = report_of_the_grid_with_the_timing_without_points(&shape, true);
    assert!(
        growing_residual.contains("name=verdict clause=F3 fired=true"),
        "{growing_residual}"
    );
}

#[test]
fn failure_clause_six_fires_on_a_cell_the_extrapolation_misses_and_is_unknown_without_cells() {
    let proportional = report_of_the_grid(
        &GridShape {
            nanos_per_step: 1,
            high_end_factor: 1,
            high_end_residual: 5,
            remounted_has_no_steps: false,
        },
        &grid_points(),
    );
    assert!(
        proportional.contains("name=verdict quantity=Q5 cells_passed=8 cells_failed=0"),
        "四根轴各有三个点、两种状态：八格都核过 {proportional}"
    );
    assert!(
        proportional.contains("name=verdict clause=F6 fired=false"),
        "{proportional}"
    );
    let high_end_four_times = report_of_the_grid(
        &GridShape {
            nanos_per_step: 1,
            high_end_factor: 4,
            high_end_residual: 5,
            remounted_has_no_steps: false,
        },
        &grid_points(),
    );
    assert!(
        high_end_four_times.contains("name=verdict clause=F6 fired=true"),
        "挂钟比 0.25 出了 [0.5, 2]：外推核不过 {high_end_four_times}"
    );
    let only_the_base = report_of_the_grid(
        &GridShape {
            nanos_per_step: 1,
            high_end_factor: 1,
            high_end_residual: 5,
            remounted_has_no_steps: false,
        },
        &grid_points()[..1],
    );
    assert!(
        only_the_base.contains("name=verdict clause=F6 fired=unknown"),
        "一格都没核：没判，不记核过 {only_the_base}"
    );
}

#[test]
fn failure_clause_one_is_not_judged_when_the_extrapolation_check_fails() {
    let high_end_four_times = report_of_the_grid(
        &GridShape {
            nanos_per_step: 1,
            high_end_factor: 4,
            high_end_residual: 5,
            remounted_has_no_steps: false,
        },
        &grid_points(),
    );
    assert!(
        high_end_four_times.contains("name=verdict quantity=Q4 cells=AtLeastOneMillisecond,AtLeastOneMillisecond,AtLeastOneMillisecond,AtLeastOneMillisecond reference_only_because_data_or_files_axes_are_missing=false not_judged_because_the_extrapolation_is_not_trusted=true"),
        "{high_end_four_times}"
    );
    assert!(
        high_end_four_times.contains("name=verdict clause=F1 fired=unknown"),
        "Q5 有一格核不过：Q4 外推不可信、不判，四格都到 1 毫秒也不记 F1 打中（变异 m36）{high_end_four_times}"
    );
}

fn state_line(build: &str, units: u64) -> String {
    format!("E203RESULT name=state build={build} point=base state=same_mount kind=overwrite round=1 units={units} mapped_units=5\n")
}

#[test]
fn stop_clause_three_counts_each_state_field_that_differs_between_the_two_builds() {
    let counts = read_product(&state_line("counts", 3000));
    assert_eq!(
        state_fields_that_differ(&counts, &read_product(&state_line("timing", 3000))),
        0,
        "只有 build 字段不同：逐字段相同，不停（变异：比较方向反了时红）"
    );
    assert_eq!(
        state_fields_that_differ(&counts, &read_product(&state_line("timing", 3001))),
        1
    );
    assert_eq!(
        state_fields_that_differ(&counts, &read_product("")),
        1,
        "计时产物里少了这一轮的规模读数：数一格不同，不当相同"
    );
    let mut out = String::new();
    full_report(
        &counts,
        Some(&read_product(&state_line("timing", 3001))),
        &mut out,
    );
    assert!(
        out.contains("name=verdict clause=stop_3 state_fields_that_differ=1"),
        "{out}"
    );
}

fn publish_text(had_caller_segment: bool, with_the_row_before_landing: bool) -> String {
    let at = "point=base state=same_mount kind=overwrite round=1";
    let mut text = format!("E203RESULT name=publish {at} had_caller_segment={had_caller_segment} fixed_point_rounds=2\n");
    if with_the_row_before_landing {
        text.push_str(&format!("E203RESULT name=row {at} segment=before_landing row=root_header_linear_find calls=1 steps=1\n"));
    }
    text
}

#[test]
fn stop_clause_four_counts_a_publish_that_skipped_the_caller_or_the_judge() {
    let violations = |had_caller_segment: bool, with_the_row_before_landing: bool| {
        publishes_that_skipped_the_caller_or_the_judge(&read_product(&publish_text(
            had_caller_segment,
            with_the_row_before_landing,
        )))
    };
    assert_eq!(violations(true, true), 0);
    assert_eq!(violations(false, true), 1, "没经调用方进门");
    assert_eq!(violations(true, false), 1, "没进落点判之前那一段");
    assert_eq!(violations(false, false), 1, "两样都缺也只数一次发布");
    let mut out = String::new();
    full_report(&read_product(&publish_text(false, true)), None, &mut out);
    assert!(
        out.contains(
            "name=verdict clause=stop_4 publishes_that_skipped_the_caller_or_the_judge=1"
        ),
        "{out}"
    );
}
