//! `reserve_fit_and_summary` 的单测。
use super::*;

#[test]
fn the_reserve_fit_divides_by_the_largest_actual_empty_publish_and_a_floor_raise_chain_needs_three()
{
    let fit = reserve_fit(14, &[3, 5]).expect("取得出两次空发布");
    assert_eq!(
        fit.largest_actual_empty_publish_slots, 5,
        "X3 取 c实 的最大值"
    );
    assert_eq!(
        fit.code_empty_publishes_held, 2,
        "⌊14 ÷ 5⌋ = 2（取最小值 3 会是 4）"
    );
    assert!(
        !fit.code_reserve_holds_the_longest_floor_raise_chain(),
        "2 次装不下最长那一串抬 F 的 3 次（门槛取 2 会判装得下）"
    );
    assert_eq!(fit.clause_reserve, 108, "条款读法 10 + 7 × 14");
    assert!(
        fit.clause_reserve_holds_the_empty_publishes_of_one_admission(),
        "⌊108 ÷ 5⌋ = 21 ≥ 7"
    );
    let at_the_threshold = reserve_fit(15, &[5]).expect("取得出一次空发布");
    assert!(
        at_the_threshold.code_reserve_holds_the_longest_floor_raise_chain(),
        "⌊15 ÷ 5⌋ = 3 正好装得下"
    );
    assert!(
        !at_the_threshold.code_reserve_holds_the_empty_publishes_of_one_admission(),
        "3 < 7"
    );
}

#[test]
fn reserve_fit_has_no_reading_without_an_obtained_empty_publish_that_writes_something() {
    assert_eq!(reserve_fit(14, &[]), None, "一次都取不出：读不到不当 0");
    assert_eq!(reserve_fit(14, &[0]), None, "最大的是 0：除不了");
}

#[test]
fn the_threshold_moved_between_the_observed_sample_points_turns_and_equal_sample_points_do_not() {
    let spread = discrimination_of_a_moved_threshold(&[2, 105], 3).expect("两个取样点");
    assert_eq!(spread.moved_threshold, Some(54), "2 + ⌈103 ÷ 2⌉");
    assert_eq!(spread.holding_at_the_moved_threshold, 1);
    assert_eq!(spread.failing_at_the_moved_threshold, 1);
    assert!(spread.turns());
    let equal = discrimination_of_a_moved_threshold(&[105, 105], 3).expect("两个取样点");
    assert_eq!(equal.moved_threshold, None);
    assert!(!equal.turns(), "取样点同值时翻不动");
    assert_eq!(discrimination_of_a_moved_threshold(&[], 3), None);
}
