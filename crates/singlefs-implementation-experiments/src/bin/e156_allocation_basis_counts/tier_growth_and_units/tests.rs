//! 第 7 次重跑：洞数各档的峰值增长、阳性对照与各档单元的判定的单测。

use super::{judge_tier_peaks, tier_sequence_positive_control_cases, TierGrowth};

/// PC-档序列（第 7 次重跑登记 5.3、E24）：W28 的判定函数在五个合成序列上逐条判对。M50（最后一个相邻差「> 0」写成「≥ 0」）在 (1, 2, 2) 上红。
#[test]
fn tier_sequence_judgement_matches_the_five_synthetic_sequences() {
    assert_eq!(judge_tier_peaks(&[1, 2, 3]), TierGrowth::StillRising);
    assert_eq!(
        judge_tier_peaks(&[1, 2, 2]),
        TierGrowth::FlatAtTheLastTiers { upper_bound: 2 }
    );
    assert_eq!(judge_tier_peaks(&[1, 3, 2]), TierGrowth::NotMonotonic);
    assert_eq!(
        judge_tier_peaks(&[5, 5, 5]),
        TierGrowth::FlatAtTheLastTiers { upper_bound: 5 }
    );
    assert_eq!(judge_tier_peaks(&[1, 2]), TierGrowth::FewerThanThreeTiers);
    for (case, actual, expected) in tier_sequence_positive_control_cases() {
        assert_eq!(actual, expected, "PC-档序列用例 {case}");
    }
}
