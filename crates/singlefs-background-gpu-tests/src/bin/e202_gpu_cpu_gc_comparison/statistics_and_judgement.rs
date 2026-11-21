//! 统计与判法：中位、离散、成对差的符号、量 T 的走向、阳性对照。

use super::*;

// ---- 统计与判法（登记第六节）：纯函数，同包测试喂合成输入 ----

/// 纳秒数换成浮点算统计：十亿量级以内的整数在 f64 里是精确的（一次跑的纳秒数远小于 2⁵³）。
#[allow(
    clippy::cast_precision_loss,
    reason = "统计口径的纳秒数远小于 2^53，f64 里精确"
)]
pub(super) fn nanoseconds_as_float(duration: Duration) -> f64 {
    duration.as_nanos() as f64
}

#[allow(clippy::cast_precision_loss, reason = "计数远小于 2^53，f64 里精确")]
pub(super) fn count_as_float(count: u64) -> f64 {
    count as f64
}

pub(super) fn median_of(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    match sorted.len() {
        0 => f64::NAN,
        length if length % 2 == 1 => sorted[length / 2],
        length => (sorted[length / 2 - 1] + sorted[length / 2]) / 2.0,
    }
}

pub(super) fn minimum_of(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::INFINITY, f64::min)
}

pub(super) fn maximum_of(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

fn median_absolute_deviation_of(values: &[f64]) -> f64 {
    let median = median_of(values);
    let deviations: Vec<f64> = values.iter().map(|value| (value - median).abs()).collect();
    median_of(&deviations)
}

/// 中位、最小、最大、中位绝对偏差（登记第一节「离散度」）。
pub(super) fn spread_fields(name: &str, values: &[f64]) -> String {
    format!(
        "{name}_median={:.0} {name}_minimum={:.0} {name}_maximum={:.0} {name}_median_absolute_deviation={:.0}",
        median_of(values),
        minimum_of(values),
        maximum_of(values),
        median_absolute_deviation_of(values)
    )
}

/// 成对差的正负（登记第一节「离散度」）：都 > 0 记「正」，都 < 0 记「负」，有正有负（或有 0）记「不稳定」。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SignOfThePairedDifferences {
    EveryOnePositive,
    EveryOneNegative,
    Unstable,
}

impl SignOfThePairedDifferences {
    pub(super) const fn name(self) -> &'static str {
        match self {
            SignOfThePairedDifferences::EveryOnePositive => "positive",
            SignOfThePairedDifferences::EveryOneNegative => "negative",
            SignOfThePairedDifferences::Unstable => "unstable",
        }
    }
}

/// 成对差的正负，门槛 `threshold`（判据用 0；第八节判别力自证把它挪到两个取样点的中位之间）。
pub(super) fn sign_of_the_paired_differences(
    differences: &[f64],
    threshold: f64,
) -> SignOfThePairedDifferences {
    if !differences.is_empty() && differences.iter().all(|difference| *difference > threshold) {
        SignOfThePairedDifferences::EveryOnePositive
    } else if !differences.is_empty()
        && differences.iter().all(|difference| *difference < threshold)
    {
        SignOfThePairedDifferences::EveryOneNegative
    } else {
        SignOfThePairedDifferences::Unstable
    }
}

/// 同一档同一次重复成对相减：基线那一臂减被比那一臂（量 S 是 CPU 臂减 GPU 臂；第九节变异「成对差方向反了」改的是这里）。
pub(super) fn paired_differences(baseline: &[f64], compared: &[f64]) -> Vec<f64> {
    baseline
        .iter()
        .zip(compared)
        .map(|(baseline_value, compared_value)| baseline_value - compared_value)
        .collect()
}

/// 量 T 相邻两档的一步（登记第六节量 T）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StepBetweenTwoTiers {
    Rises,
    Falls,
    Overlaps,
}

fn step_between_two_tiers(smaller_tier: &[f64], larger_tier: &[f64]) -> StepBetweenTwoTiers {
    if minimum_of(larger_tier) > maximum_of(smaller_tier) {
        StepBetweenTwoTiers::Rises
    } else if maximum_of(larger_tier) < minimum_of(smaller_tier) {
        StepBetweenTwoTiers::Falls
    } else {
        StepBetweenTwoTiers::Overlaps
    }
}

/// 量 T 的分类。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TrendOverTheTiers {
    Rising,
    Flat,
    Falling,
    CannotTell,
}

impl TrendOverTheTiers {
    pub(super) const fn name(self) -> &'static str {
        match self {
            TrendOverTheTiers::Rising => "rising",
            TrendOverTheTiers::Flat => "flat",
            TrendOverTheTiers::Falling => "falling",
            TrendOverTheTiers::CannotTell => "cannot_tell",
        }
    }
}

/// 三档（按单元区从小到大）各自 S 的每批值：两步都「升」判「涨」，都「落」判「降」；两步都「交叠」且顶档与最小档中位之差的绝对值
/// ≤ 两者绝对值大者的 `FLAT_TREND_WIDTH` 判「平」；别的判「说不出」（第九节变异「平的宽度 10% 改成 50%」改的是那个常量）。
pub(super) fn trend_over_the_tiers(saving_per_batch_by_tier: &[Vec<f64>; 3]) -> TrendOverTheTiers {
    let [smallest, middle, top] = saving_per_batch_by_tier;
    let lower_step = step_between_two_tiers(smallest, middle);
    let upper_step = step_between_two_tiers(middle, top);
    match (lower_step, upper_step) {
        (StepBetweenTwoTiers::Rises, StepBetweenTwoTiers::Rises) => TrendOverTheTiers::Rising,
        (StepBetweenTwoTiers::Falls, StepBetweenTwoTiers::Falls) => TrendOverTheTiers::Falling,
        (StepBetweenTwoTiers::Overlaps, StepBetweenTwoTiers::Overlaps) => {
            let smallest_median = median_of(smallest);
            let top_median = median_of(top);
            let width = FLAT_TREND_WIDTH * smallest_median.abs().max(top_median.abs());
            if (top_median - smallest_median).abs() <= width {
                TrendOverTheTiers::Flat
            } else {
                TrendOverTheTiers::CannotTell
            }
        }
        (
            StepBetweenTwoTiers::Rises | StepBetweenTwoTiers::Falls | StepBetweenTwoTiers::Overlaps,
            _,
        ) => TrendOverTheTiers::CannotTell,
    }
}

/// 量 P 一条臂的判定（登记第五节「阳性对照」判法、第六节量 P）：各项分开报，不合成一个真假。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PositiveControlOfAnArm {
    pub(super) every_delta_positive: bool,
    pub(super) median_delta: f64,
    pub(super) median_inner_rejudge_cpu: f64,
    pub(super) median_delta_within_the_band_of_the_inner_timing: bool,
    /// 两条 GPU 臂：5 对里 CPU 臂减 GPU 多判臂都比 CPU 臂减 GPU 臂少；CPU 臂没有这一项。
    pub(super) saving_with_rejudging_below_the_saving_in_every_pair: Option<bool>,
}

impl PositiveControlOfAnArm {
    pub(super) fn holds(self) -> bool {
        self.every_delta_positive
            && self.median_delta_within_the_band_of_the_inner_timing
            && self.saving_with_rejudging_below_the_saving_in_every_pair != Some(false)
    }
}

/// Δ = 多判臂 C − 不多判臂 C（成对）；E = 多判臂的里层计时；GPU 臂另给 CPU 臂的 C 算 S 多判与 S。
pub(super) fn positive_control_of_an_arm(
    process_cpu_not_rejudged: &[f64],
    process_cpu_rejudged: &[f64],
    inner_rejudge_cpu: &[f64],
    process_cpu_of_the_cpu_arm: Option<&[f64]>,
) -> PositiveControlOfAnArm {
    let deltas = paired_differences(process_cpu_rejudged, process_cpu_not_rejudged);
    let median_delta = median_of(&deltas);
    let median_inner_rejudge_cpu = median_of(inner_rejudge_cpu);
    let saving_with_rejudging_below_the_saving_in_every_pair =
        process_cpu_of_the_cpu_arm.map(|cpu_arm| {
            let savings = paired_differences(cpu_arm, process_cpu_not_rejudged);
            let savings_with_rejudging = paired_differences(cpu_arm, process_cpu_rejudged);
            !savings.is_empty()
                && savings
                    .iter()
                    .zip(&savings_with_rejudging)
                    .all(|(saving, saving_with_rejudging)| saving_with_rejudging < saving)
        });
    PositiveControlOfAnArm {
        every_delta_positive: sign_of_the_paired_differences(&deltas, 0.0)
            == SignOfThePairedDifferences::EveryOnePositive,
        median_delta,
        median_inner_rejudge_cpu,
        median_delta_within_the_band_of_the_inner_timing: median_delta
            >= POSITIVE_CONTROL_RATIO_LOWER * median_inner_rejudge_cpu
            && median_delta <= POSITIVE_CONTROL_RATIO_UPPER * median_inner_rejudge_cpu,
        saving_with_rejudging_below_the_saving_in_every_pair,
    }
}

#[cfg(test)]
mod tests;
