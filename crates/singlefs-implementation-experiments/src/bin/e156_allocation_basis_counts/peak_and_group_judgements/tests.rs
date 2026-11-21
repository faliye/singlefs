//! 判定（登记第六节 6.2、第八节 8.2）：峰值对门槛、几何敏感性、随洞增长、阳性对照与按组的统计的单测。

use super::{
    judge_geometry_sensitivity, judge_growth_with_holes, judge_peak_against_threshold,
    judge_positive_control_cases, GeometrySensitivity, GrowthWithHoles, PeakJudgement,
};
use std::collections::BTreeMap;

/// U18（第 4 次重跑登记第九节，PC-判定器）：判定函数在合成用例上逐条判对。M43（「> 2」写成「≥ 2」）在这里红。
#[test]
fn judgement_of_peak_delta_against_threshold_and_geometry_sensitivity_on_synthetic_cases() {
    let peaks = |pairs: &[(u64, i64)]| -> BTreeMap<u64, i64> { pairs.iter().copied().collect() };
    assert_eq!(
        judge_peak_against_threshold(&peaks(&[(1, 3)]), 2),
        PeakJudgement::AboveTheThreshold
    );
    assert_eq!(
        judge_peak_against_threshold(&peaks(&[(1, 2)]), 2),
        PeakJudgement::AtMostTheThreshold
    );
    assert_eq!(
        judge_peak_against_threshold(&peaks(&[(1, 1)]), 1),
        PeakJudgement::AtMostTheThreshold
    );
    assert_eq!(
        judge_peak_against_threshold(&peaks(&[(1, 2)]), 1),
        PeakJudgement::AboveTheThreshold
    );
    assert_eq!(
        judge_peak_against_threshold(&peaks(&[]), 2),
        PeakJudgement::NothingObserved
    );
    assert_eq!(
        judge_geometry_sensitivity(&[
            PeakJudgement::AtMostTheThreshold,
            PeakJudgement::AtMostTheThreshold,
            PeakJudgement::AtMostTheThreshold,
            PeakJudgement::AboveTheThreshold,
        ]),
        GeometrySensitivity::Unstable
    );
    assert_eq!(
        judge_geometry_sensitivity(&[PeakJudgement::AtMostTheThreshold; 4]),
        GeometrySensitivity::Consistent { sampling_points: 4 }
    );
    assert_eq!(
        judge_growth_with_holes(&peaks(&[(1, 2), (2, 2)])),
        GrowthWithHoles::IndependentOfHoles
    );
    assert_eq!(
        judge_growth_with_holes(&peaks(&[(1, 1), (2, 3)])),
        GrowthWithHoles::GrowsWithHoles
    );
    assert_eq!(
        judge_growth_with_holes(&peaks(&[(1, 3), (2, 1)])),
        GrowthWithHoles::NotMonotonic
    );
    for (case, actual, expected) in judge_positive_control_cases() {
        assert_eq!(actual, expected, "PC-判定器用例 {case}");
    }
}
