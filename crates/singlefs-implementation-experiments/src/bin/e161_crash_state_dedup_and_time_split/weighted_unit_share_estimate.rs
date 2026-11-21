//! G3 的加权估计（登记 5.5 第一段）。

use super::*;

// ───────────────────────────── G3 的加权估计（5.5 第一段） ─────────────────────────────

/// 单元级检查占每状态总时长的上下界估计（Q3i）：各段取样状态的平均时长 × 该段状态数，再加总。
#[derive(Clone, Copy, Debug)]
pub(super) struct UnitShareEstimate {
    pub(super) lower: f64,
    pub(super) upper: f64,
}

/// G3 的判定（第六节 Q3i）：s_hi < 0.5 翻；s_lo ≥ 0.5 不翻；其余不够判。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShareVerdict {
    FlipsToWalkAndJudgeOnTheDeviceToo,
    DoesNotFlip,
    NotEnoughToJudge,
}

impl ShareVerdict {
    pub(super) fn of(estimate: UnitShareEstimate, threshold: f64) -> Self {
        if estimate.upper < threshold {
            Self::FlipsToWalkAndJudgeOnTheDeviceToo
        } else if estimate.lower >= threshold {
            Self::DoesNotFlip
        } else {
            Self::NotEnoughToJudge
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::FlipsToWalkAndJudgeOnTheDeviceToo => "flips_walk_and_judge_on_the_device_too",
            Self::DoesNotFlip => "does_not_flip",
            Self::NotEnoughToJudge => "not_enough_to_judge",
        }
    }
}

pub(super) fn threshold_half() -> f64 {
    f64::from(u32::try_from(RATIO_THRESHOLD_NUMERATOR).expect("门槛分子"))
        / f64::from(u32::try_from(RATIO_THRESHOLD_DENOMINATOR).expect("门槛分母"))
}

#[allow(
    clippy::cast_precision_loss,
    reason = "计时的纳秒和装得进 f64 的 53 位尾数之内（D1 上最多几千秒 ≈ 10¹³ 纳秒），这里只算占比"
)]
fn nanoseconds_as_float(nanoseconds: u128) -> f64 {
    nanoseconds as f64
}

#[allow(
    clippy::cast_precision_loss,
    reason = "段的状态数至多 3² · 2²⁴，装得进 f64 的尾数"
)]
fn count_as_float(count: u64) -> f64 {
    count as f64
}

/// 加权估计：`timing_by_segment` 是取样状态的各段累计，`plan` 给各段在全域里的状态数（全部持久那一个算一段、1 个）。
pub(super) fn weighted_unit_share(
    timing_by_segment: &BTreeMap<usize, TimingSums>,
    plan: &LocalPlan<'_>,
) -> UnitShareEstimate {
    let mut state = 0f64;
    let mut positions = 0f64;
    let mut events = 0f64;
    for (segment, sums) in timing_by_segment {
        if sums.states == 0 {
            continue;
        }
        let states_in_the_full_domain = plan.count_by_segment.get(*segment).copied().unwrap_or(1);
        let weight = count_as_float(states_in_the_full_domain) / count_as_float(sums.states);
        state += nanoseconds_as_float(sums.state_nanoseconds) * weight;
        positions += nanoseconds_as_float(sums.zone_nanoseconds[5]) * weight;
        events += nanoseconds_as_float(sums.unit_events_replay_nanoseconds) * weight;
    }
    UnitShareEstimate {
        lower: positions / state,
        upper: events / state,
    }
}

/// 未加权的上下界（D1-quick 与线程数 1 那一格：完整枚举或同一段里的取样，不用加权）。
pub(super) fn unweighted_unit_share(summary: &CellSummary) -> UnitShareEstimate {
    UnitShareEstimate {
        lower: nanoseconds_as_float(summary.zone_nanoseconds[5])
            / nanoseconds_as_float(summary.state_nanoseconds),
        upper: nanoseconds_as_float(summary.unit_events_replay_nanoseconds)
            / nanoseconds_as_float(summary.state_nanoseconds),
    }
}

/// 第一段 G3 够不够判：两个数在 0.5 同一侧且离 0.5 至少 0.1（5.5）。
pub(super) fn share_is_enough_to_judge(estimate: UnitShareEstimate) -> bool {
    let threshold = threshold_half();
    (estimate.upper < threshold - SHARE_MARGIN_FOR_ENOUGH)
        || (estimate.lower >= threshold + SHARE_MARGIN_FOR_ENOUGH)
}
