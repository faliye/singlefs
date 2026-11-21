use super::*;

#[test]
fn the_saving_is_the_cpu_arm_minus_the_gpu_arm() {
    assert_eq!(
        paired_differences(&[10.0, 20.0], &[4.0, 25.0]),
        vec![6.0, -5.0]
    );
}

#[test]
fn paired_differences_all_positive_all_negative_or_mixed_are_told_apart() {
    assert_eq!(
        sign_of_the_paired_differences(&[1.0, 2.0, 3.0, 4.0, 5.0], 0.0),
        SignOfThePairedDifferences::EveryOnePositive
    );
    assert_eq!(
        sign_of_the_paired_differences(&[-1.0, -2.0, -3.0, -4.0, -5.0], 0.0),
        SignOfThePairedDifferences::EveryOneNegative
    );
    assert_eq!(
        sign_of_the_paired_differences(&[1.0, -2.0, 3.0, 4.0, 5.0], 0.0),
        SignOfThePairedDifferences::Unstable
    );
    assert_eq!(
        sign_of_the_paired_differences(&[1.0, 0.0, 3.0, 4.0, 5.0], 0.0),
        SignOfThePairedDifferences::Unstable
    );
    assert_eq!(
        sign_of_the_paired_differences(&[], 0.0),
        SignOfThePairedDifferences::Unstable
    );
}

/// 登记第八节判别力自证：把量 S 的判定门槛从 0 挪到两个取样点的中位之间，主取样点与相反取样点一绿一红。
#[test]
fn moving_the_saving_threshold_between_the_two_sampling_points_turns_one_green_and_one_red() {
    let main_sampling_point = [5.0, 6.0, 7.0, 6.0, 5.5];
    let opposite_sampling_point = [1.0, 1.2, 0.8, 1.1, 0.9];
    assert_eq!(
        sign_of_the_paired_differences(&main_sampling_point, 0.0),
        SignOfThePairedDifferences::EveryOnePositive
    );
    assert_eq!(
        sign_of_the_paired_differences(&opposite_sampling_point, 0.0),
        SignOfThePairedDifferences::EveryOnePositive
    );
    let between = f64::midpoint(
        median_of(&main_sampling_point),
        median_of(&opposite_sampling_point),
    );
    assert_eq!(
        sign_of_the_paired_differences(&main_sampling_point, between),
        SignOfThePairedDifferences::EveryOnePositive
    );
    assert_eq!(
        sign_of_the_paired_differences(&opposite_sampling_point, between),
        SignOfThePairedDifferences::EveryOneNegative
    );
}

#[test]
fn median_minimum_maximum_and_median_absolute_deviation_of_five_values() {
    let values = [3.0, 1.0, 4.0, 1.0, 5.0];
    assert_eq!(median_of(&values), 3.0);
    assert_eq!(minimum_of(&values), 1.0);
    assert_eq!(maximum_of(&values), 5.0);
    assert_eq!(median_absolute_deviation_of(&values), 2.0);
    assert_eq!(median_of(&[1.0, 2.0, 3.0, 10.0]), 2.5);
}

fn five_around(center: f64) -> Vec<f64> {
    vec![
        center - 2.0,
        center - 1.0,
        center,
        center + 1.0,
        center + 2.0,
    ]
}

#[test]
fn the_trend_rises_or_falls_when_both_steps_go_the_same_way() {
    assert_eq!(
        trend_over_the_tiers(&[five_around(10.0), five_around(20.0), five_around(30.0)]),
        TrendOverTheTiers::Rising
    );
    assert_eq!(
        trend_over_the_tiers(&[five_around(30.0), five_around(20.0), five_around(10.0)]),
        TrendOverTheTiers::Falling
    );
    assert_eq!(
        trend_over_the_tiers(&[five_around(10.0), five_around(20.0), five_around(10.0)]),
        TrendOverTheTiers::CannotTell
    );
}

#[test]
fn overlapping_tiers_within_ten_percent_are_flat() {
    assert_eq!(
        trend_over_the_tiers(&[five_around(100.0), five_around(102.0), five_around(105.0)]),
        TrendOverTheTiers::Flat
    );
}

/// 第九节变异「平的宽度 10% 改成 50%」在这里红：顶档与最小档中位差 30% 而三档都交叠，是「说不出」不是「平」。
#[test]
fn thirty_percent_between_the_end_tiers_is_not_flat() {
    let wide = |center: f64| {
        vec![
            center - 40.0,
            center - 20.0,
            center,
            center + 20.0,
            center + 40.0,
        ]
    };
    assert_eq!(
        trend_over_the_tiers(&[wide(100.0), wide(115.0), wide(130.0)]),
        TrendOverTheTiers::CannotTell
    );
}

#[test]
fn positive_control_holds_when_the_delta_matches_the_inner_timing() {
    let plain = [100.0, 110.0, 105.0, 95.0, 100.0];
    let rejudged = [150.0, 160.0, 154.0, 146.0, 150.0];
    let inner = [50.0, 50.0, 50.0, 50.0, 50.0];
    let cpu_arm = [200.0, 210.0, 205.0, 195.0, 200.0];
    let control = positive_control_of_an_arm(&plain, &rejudged, &inner, Some(&cpu_arm));
    assert!(control.every_delta_positive);
    assert_eq!(control.median_delta, 50.0);
    assert!(control.median_delta_within_the_band_of_the_inner_timing);
    assert_eq!(
        control.saving_with_rejudging_below_the_saving_in_every_pair,
        Some(true)
    );
    assert!(control.holds());
}

#[test]
fn positive_control_fails_when_the_rejudging_adds_nothing_or_far_more_than_the_inner_timing() {
    let plain = [100.0, 110.0, 105.0, 95.0, 100.0];
    let inner = [50.0, 50.0, 50.0, 50.0, 50.0];
    let adding_nothing = positive_control_of_an_arm(&plain, &plain, &inner, None);
    assert!(!adding_nothing.every_delta_positive);
    assert!(!adding_nothing.holds());
    let far_more = [300.0, 310.0, 305.0, 295.0, 300.0];
    let adding_far_more = positive_control_of_an_arm(&plain, &far_more, &inner, None);
    assert!(adding_far_more.every_delta_positive);
    assert!(!adding_far_more.median_delta_within_the_band_of_the_inner_timing);
    assert!(!adding_far_more.holds());
}
