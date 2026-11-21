use super::*;

#[test]
fn log_log_slope_of_a_doubling_series_is_exactly_one_and_of_a_constant_is_zero() {
    let doubling: Vec<(u64, u64)> = (0..5).map(|power| (1u64 << power, 1u64 << power)).collect();
    let slope = log_log_least_squares_slope(&doubling).expect("五个点");
    assert!((slope.slope - 1.0).abs() < 1e-12, "{slope:?}");
    assert_eq!(slope.points, 5);
    assert!(!slope.has_zero_point);
    let squares: Vec<(u64, u64)> = (1..6)
        .map(|power| (1u64 << power, 1u64 << (2 * power)))
        .collect();
    assert!((log_log_least_squares_slope(&squares).expect("五个点").slope - 2.0).abs() < 1e-12);
    let constant: Vec<(u64, u64)> = (1..6).map(|power| (1u64 << (10 * power), 4096)).collect();
    assert!(
        log_log_least_squares_slope(&constant)
            .expect("五个点")
            .slope
            .abs()
            < 1e-12
    );
}

#[test]
fn zero_reading_is_taken_as_one_byte_and_flagged() {
    let slope = log_log_least_squares_slope(&[(2, 0), (4, 4)]).expect("两个点");
    assert!(slope.has_zero_point);
    assert!(
        (slope.slope - 2.0).abs() < 1e-12,
        "0 按 1 取：log₂ 1 = 0 到 log₂ 4 = 2，跨 1 个翻倍"
    );
    assert!(
        log_log_least_squares_slope(&[(2, 5)]).is_none(),
        "一个点没有斜率"
    );
    assert!(
        log_log_least_squares_slope(&[(2, 5), (2, 9)]).is_none(),
        "x 相同没有斜率"
    );
}

#[test]
fn slope_class_puts_exactly_0_8_in_linear_and_exactly_0_2_in_sublinear() {
    assert_eq!(slope_class(0.8), SlopeClass::Linear);
    assert_eq!(slope_class(0.799), SlopeClass::Sublinear);
    assert_eq!(slope_class(0.2), SlopeClass::Sublinear);
    assert_eq!(slope_class(0.199), SlopeClass::BasicallyUnrelated);
    assert_eq!(slope_class(-0.3), SlopeClass::BasicallyUnrelated);
}

#[test]
fn five_rounds_give_a_class_only_when_every_round_falls_in_it() {
    let stable = [Some(0.9), Some(1.1), Some(0.85), Some(1.0), Some(0.95)];
    assert_eq!(
        class_across_rounds(&stable, 5),
        ClassAcrossRounds::Stable(SlopeClass::Linear)
    );
    let crossing = [Some(0.9), Some(0.7), Some(0.85), Some(1.0), Some(0.95)];
    assert_eq!(
        class_across_rounds(&crossing, 5),
        ClassAcrossRounds::Unstable
    );
    let short = [Some(0.9), None, Some(0.85), Some(1.0), Some(0.95)];
    assert_eq!(
        class_across_rounds(&short, 5),
        ClassAcrossRounds::NotJudged {
            rounds_with_a_slope: 4
        }
    );
}

#[test]
fn moving_the_linear_boundary_between_two_real_slopes_flips_one_class() {
    let moved = linear_boundary_moved_between_real_slopes(&[0.05, 0.95, 1.02]).expect("挪得出");
    assert!((moved.boundary - 0.985).abs() < 1e-12, "{moved:?}");
    assert_eq!(moved.flipped, 1, "0.95 从线性转成次线性");
    assert!(
        linear_boundary_moved_between_real_slopes(&[0.1, 0.9]).is_none(),
        "只有跨着 0.8 的那一对：挪到中点 0.5 一个类都不转"
    );
    assert!(synthetic_slopes_fall_in_two_classes());
}

#[test]
fn median_takes_the_middle_value_and_the_lower_one_of_an_even_count() {
    assert_eq!(median_of(&[9, 1, 5, 7, 3]), Some(5));
    assert_eq!(median_of(&[4, 2, 8, 6]), Some(4));
    assert_eq!(median_of(&[]), None);
}
