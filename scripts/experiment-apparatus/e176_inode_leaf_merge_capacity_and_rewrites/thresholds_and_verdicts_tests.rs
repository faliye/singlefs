use super::*;

#[test]
fn verdict_thresholds_are_inclusive_where_the_registration_says_so() {
    let point = |numerator: i128, denominator: i128| PointValue { value: Some(Rational::new(numerator, denominator)), note: "" };
    let capacity = registered_thresholds(JudgedRow::CapacityRatio);
    assert_eq!(point_verdict(JudgedRow::CapacityRatio, point(90, 100), capacity), "no_merge_capacity_cost_small");
    assert_eq!(point_verdict(JudgedRow::CapacityRatio, point(89, 100), capacity), "no_flip");
    assert_eq!(point_verdict(JudgedRow::CapacityRatio, point(50, 100), capacity), "no_flip");
    assert_eq!(point_verdict(JudgedRow::CapacityRatio, point(49, 100), capacity), "merge_buys_double_capacity");
    let delta2p = JUDGED_ROWS[1];
    let delta2c = JUDGED_ROWS[3];
    let write_cost = registered_thresholds(delta2p);
    assert_eq!(point_verdict(delta2p, point(10, 100), write_cost), "negligible");
    assert_eq!(point_verdict(delta2p, point(-3, 1), write_cost), "negligible");
    assert_eq!(point_verdict(delta2p, point(11, 100), write_cost), "no_flip");
    assert_eq!(point_verdict(delta2p, point(1, 1), write_cost), "significant");
    assert_eq!(point_verdict(delta2c, point(1, 1), write_cost), "no_flip", "Δ2c 的「≥ 1.00」一侧不算判据");
    let thin = PointValue { value: Some(Rational::new(5, 1)), note: "insufficient_sample" };
    assert_eq!(point_verdict(delta2p, thin, write_cost), "insufficient_sample");
}

#[test]
fn moving_a_threshold_between_two_same_verdict_points_splits_them() {
    let point = |numerator: i128, denominator: i128| PointValue { value: Some(Rational::new(numerator, denominator)), note: "" };
    let cases = [
        (JudgedRow::CapacityRatio, point(95, 100), point(99, 100)),
        (JudgedRow::CapacityRatio, point(60, 100), point(70, 100)),
        (JudgedRow::CapacityRatio, point(10, 100), point(30, 100)),
        (JUDGED_ROWS[1], point(1, 100), point(5, 100)),
        (JUDGED_ROWS[1], point(30, 100), point(60, 100)),
        (JUDGED_ROWS[1], point(2, 1), point(3, 1)),
        (JUDGED_ROWS[3], point(2, 1), point(3, 1)),
    ];
    for (row, main, opposite) in cases {
        let registered = registered_thresholds(row);
        assert_eq!(point_verdict(row, main, registered), point_verdict(row, opposite, registered), "取样的两点在登记门槛下同判");
        let (_, moved_main, moved_opposite) = moved_threshold_verdicts(row, main, opposite).expect("两点值不同");
        assert_ne!(moved_main, moved_opposite, "{} 挪门槛之后两点要分开", row.name());
    }
    assert!(moved_threshold_verdicts(JUDGED_ROWS[1], point(1, 10), point(1, 10)).is_none(), "两点值相等时挪不开");
}

#[test]
fn rational_decimal_rounds_half_away_from_zero() {
    assert_eq!(Rational::new(4292 * 100, 430 * 100).decimal(2), "9.98");
    assert_eq!(Rational::new(1, 8).decimal(2), "0.13");
    assert_eq!(Rational::new(-1, 8).decimal(2), "-0.13");
    assert_eq!(Rational::new(0, 7).decimal(4), "0.0000");
    assert_eq!(Rational::new(367, 31_455).decimal(4), "0.0117");
}
