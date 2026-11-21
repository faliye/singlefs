//! 一个几何格的产物行：头、锚点、各臂、残差、轨迹、族与格的汇总、判定、敏感性与独立算术的单测。

use super::*;

#[test]
fn the_threshold_between_two_readings_flips_the_judgement_only_when_they_differ() {
    assert_eq!(threshold_between_flips_the_judgement(0, 5), Some(true));
    assert_eq!(threshold_between_flips_the_judgement(5, 0), Some(true));
    assert_eq!(threshold_between_flips_the_judgement(3, 3), None);
}
