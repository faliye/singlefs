//! 判定（登记第六节 6.2、第八节 8.2）：峰值对门槛、几何敏感性、随洞增长、阳性对照与按组的统计。

use super::*;

// ============================================================================================
// 判定（登记第六节 6.2、第八节 8.2；PC-判定器在单测 U18 与 [`main`] 里各跑一遍）。
// ============================================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum PeakJudgement {
    AtMostTheThreshold,
    AboveTheThreshold,
    NothingObserved,
}

impl PeakJudgement {
    pub(super) fn label(self) -> &'static str {
        match self {
            PeakJudgement::AtMostTheThreshold => "at_most",
            PeakJudgement::AboveTheThreshold => "above",
            PeakJudgement::NothingObserved => "nothing_observed",
        }
    }
}

/// 全部分组键上的峰值里有一个越过门槛 ⇒ 越过；一个都没有 ⇒ 没越过；一个观测都没有 ⇒ 没有观测。
fn judge_peak_against_threshold(peaks: &BTreeMap<u64, i64>, threshold_slots: i64) -> PeakJudgement {
    if peaks.is_empty() {
        return PeakJudgement::NothingObserved;
    }
    if peaks.values().any(|peak| *peak > threshold_slots) {
        PeakJudgement::AboveTheThreshold
    } else {
        PeakJudgement::AtMostTheThreshold
    }
}

fn smallest_key_above_threshold(peaks: &BTreeMap<u64, i64>, threshold_slots: i64) -> Option<u64> {
    peaks
        .iter()
        .find(|(_, peak)| **peak > threshold_slots)
        .map(|(key, _)| *key)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum GeometrySensitivity {
    Consistent { sampling_points: usize },
    Unstable,
    NoSamplingPoint,
}

impl GeometrySensitivity {
    pub(super) fn label(self) -> String {
        match self {
            GeometrySensitivity::Consistent { sampling_points } => {
                format!("consistent_on_{sampling_points}_sampling_points")
            }
            GeometrySensitivity::Unstable => "unstable".to_string(),
            GeometrySensitivity::NoSamplingPoint => "no_sampling_point".to_string(),
        }
    }
}

/// r2「八」字面：逐个取样点的判定值，任意两个不同 ⇒ 不稳定；全部相同 ⇒ N 个取样点一致。没有观测的取样点不算进去。
pub(super) fn judge_geometry_sensitivity(judgements: &[PeakJudgement]) -> GeometrySensitivity {
    let judged: Vec<PeakJudgement> = judgements
        .iter()
        .copied()
        .filter(|judgement| *judgement != PeakJudgement::NothingObserved)
        .collect();
    match judged.first() {
        None => GeometrySensitivity::NoSamplingPoint,
        Some(first) if judged.iter().all(|judgement| judgement == first) => {
            GeometrySensitivity::Consistent {
                sampling_points: judged.len(),
            }
        }
        Some(_) => GeometrySensitivity::Unstable,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum GrowthWithHoles {
    IndependentOfHoles,
    GrowsWithHoles,
    NotMonotonic,
    FewerThanTwoPoints,
}

impl GrowthWithHoles {
    pub(super) fn label(self) -> &'static str {
        match self {
            GrowthWithHoles::IndependentOfHoles => "independent_of_holes",
            GrowthWithHoles::GrowsWithHoles => "grows_with_holes",
            GrowthWithHoles::NotMonotonic => "not_monotonic",
            GrowthWithHoles::FewerThanTwoPoints => "fewer_than_two_points",
        }
    }
}

/// Q1d：按洞的计数升序列峰值，相邻差全 0 ⇒ 与洞无关；全部 ≥ 0 且有一个 > 0 ⇒ 随洞增长；否则不单调。
fn judge_growth_with_holes(peaks: &BTreeMap<u64, i64>) -> GrowthWithHoles {
    let values: Vec<i64> = peaks.values().copied().collect();
    if values.len() < 2 {
        return GrowthWithHoles::FewerThanTwoPoints;
    }
    let differences: Vec<i64> = values.windows(2).map(|pair| pair[1] - pair[0]).collect();
    if differences.iter().all(|difference| *difference == 0) {
        GrowthWithHoles::IndependentOfHoles
    } else if differences.iter().all(|difference| *difference >= 0) {
        GrowthWithHoles::GrowsWithHoles
    } else {
        GrowthWithHoles::NotMonotonic
    }
}

/// PC-判定器的合成用例（登记第五节 5.4、第八节 8.2 判别力自证 ①）：(说明, 实际, 应得)。
pub(super) fn judge_positive_control_cases() -> Vec<(&'static str, String, String)> {
    let peaks = |pairs: &[(u64, i64)]| -> BTreeMap<u64, i64> { pairs.iter().copied().collect() };
    vec![
        (
            "peak_three_against_two",
            judge_peak_against_threshold(&peaks(&[(1, 3)]), E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS)
                .label()
                .to_string(),
            "above".to_string(),
        ),
        (
            "peak_two_against_two",
            judge_peak_against_threshold(&peaks(&[(1, 2)]), E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS)
                .label()
                .to_string(),
            "at_most".to_string(),
        ),
        (
            "peak_one_against_one",
            judge_peak_against_threshold(&peaks(&[(1, 1)]), E156_THRESHOLD_ONE_SPAN_ONE_UNIT_SLOTS)
                .label()
                .to_string(),
            "at_most".to_string(),
        ),
        (
            "peak_two_against_one",
            judge_peak_against_threshold(&peaks(&[(1, 2)]), E156_THRESHOLD_ONE_SPAN_ONE_UNIT_SLOTS)
                .label()
                .to_string(),
            "above".to_string(),
        ),
        (
            "three_at_most_one_above",
            judge_geometry_sensitivity(&[
                PeakJudgement::AtMostTheThreshold,
                PeakJudgement::AtMostTheThreshold,
                PeakJudgement::AtMostTheThreshold,
                PeakJudgement::AboveTheThreshold,
            ])
            .label(),
            "unstable".to_string(),
        ),
        (
            "four_at_most",
            judge_geometry_sensitivity(&[PeakJudgement::AtMostTheThreshold; 4]).label(),
            "consistent_on_4_sampling_points".to_string(),
        ),
        (
            "synthetic_zero_and_five_against_two",
            judge_peak_against_threshold(
                &peaks(&[(1, 0), (2, 5)]),
                E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS,
            )
            .label()
            .to_string(),
            "above".to_string(),
        ),
        (
            "growth_flat",
            judge_growth_with_holes(&peaks(&[(1, 2), (2, 2)]))
                .label()
                .to_string(),
            "independent_of_holes".to_string(),
        ),
        (
            "growth_rising",
            judge_growth_with_holes(&peaks(&[(1, 1), (2, 3), (4, 3)]))
                .label()
                .to_string(),
            "grows_with_holes".to_string(),
        ),
        (
            "growth_falling",
            judge_growth_with_holes(&peaks(&[(1, 3), (2, 1)]))
                .label()
                .to_string(),
            "not_monotonic".to_string(),
        ),
    ]
}

/// 一个 (S, ρ, 回收时点) 分组里的一个 Δ 观测。
#[derive(Clone, Debug)]
pub(super) struct DeltaSample {
    pub(super) shape: HhCellShape,
    pub(super) txg: u64,
    pub(super) workload_publishes: u64,
    pub(super) missing_txgs: u64,
    pub(super) live_holes: u64,
    pub(super) delta: i64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub(super) enum DensityKey {
    EveryPublishOverwrites,
    OneOverwriteInFourPublishes,
}

pub(super) fn density_key(density: WorkloadDensity) -> DensityKey {
    match density {
        WorkloadDensity::EveryPublishOverwrites => DensityKey::EveryPublishOverwrites,
        WorkloadDensity::OneOverwriteInFourPublishes => DensityKey::OneOverwriteInFourPublishes,
    }
}

pub(super) type GroupKey = (u64, DensityKey, ReclaimTiming);

pub(super) fn group_label(key: &GroupKey) -> String {
    let density = match key.1 {
        DensityKey::EveryPublishOverwrites => "1",
        DensityKey::OneOverwriteInFourPublishes => "1/4",
    };
    format!("s={} rho={density} timing={}", key.0, key.2.label())
}

/// 一组 Δ 按分组键（h_缺 或 h_洞 或 k）取峰值、和、条数、为正的条数。
fn group_statistics(
    samples: &[&DeltaSample],
    key_of: impl Fn(&DeltaSample) -> u64,
) -> BTreeMap<u64, (i64, i64, u64, u64)> {
    let mut statistics: BTreeMap<u64, (i64, i64, u64, u64)> = BTreeMap::new();
    for sample in samples {
        let entry = statistics
            .entry(key_of(sample))
            .or_insert((i64::MIN, 0, 0, 0));
        entry.0 = entry.0.max(sample.delta);
        entry.1 += sample.delta;
        entry.2 += 1;
        if sample.delta > 0 {
            entry.3 += 1;
        }
    }
    statistics
}

fn peaks_of(statistics: &BTreeMap<u64, (i64, i64, u64, u64)>) -> BTreeMap<u64, i64> {
    statistics
        .iter()
        .map(|(key, entry)| (*key, entry.0))
        .collect()
}

fn statistics_text(statistics: &BTreeMap<u64, (i64, i64, u64, u64)>) -> String {
    let parts: Vec<String> = statistics
        .iter()
        .map(|(key, (peak, sum, count, positive))| {
            format!("{key}:peak{peak}:sum{sum}:count{count}:positive{positive}")
        })
        .collect();
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join(",")
    }
}

/// 第一次越过门槛的观测（最小的那个越过门槛的分组键里，工作负载发布数、txg 最小的一条）。
fn first_crossing_text(
    samples: &[&DeltaSample],
    key_of: impl Fn(&DeltaSample) -> u64,
    peaks: &BTreeMap<u64, i64>,
    threshold_slots: i64,
) -> String {
    let Some(key) = smallest_key_above_threshold(peaks, threshold_slots) else {
        return "not_crossed_before_the_end".to_string();
    };
    samples
        .iter()
        .filter(|sample| key_of(sample) == key && sample.delta > threshold_slots)
        .min_by_key(|sample| (sample.workload_publishes, sample.txg))
        .map_or_else(
            || "none".to_string(),
            |sample| {
                format!(
                    "key{key}:k{}:position{}:workload_publishes{}:txg{}",
                    sample.shape.hole_count,
                    sample.shape.position_label(),
                    sample.workload_publishes,
                    sample.txg
                )
            },
        )
}

/// 一个分组的四个 Q1c 读数与 Q1d（登记第六节 6.2 表里 Q1c、Q1c-洞、Q1c-前、Q1c-后、Q1d 各一行）。
#[derive(Clone, Copy, Debug)]
pub(super) struct GroupJudgements {
    pub(super) all_points: PeakJudgement,
    pub(super) on_holes: PeakJudgement,
    pub(super) not_on_holes: PeakJudgement,
    pub(super) front_holes: PeakJudgement,
    pub(super) back_holes: PeakJudgement,
    pub(super) front_holes_one_slot: PeakJudgement,
    pub(super) growth: GrowthWithHoles,
    pub(super) front_peak_is_zero_everywhere: bool,
    pub(super) back_peak_is_zero_everywhere: bool,
}

#[allow(
    clippy::too_many_lines,
    reason = "一个分组的 Q1c 四行、Q1d 三张表与判别力自证 ② 连着打，才能与登记第六节那张表逐行对"
)]
pub(super) fn judge_group(
    emitter: &mut Emitter,
    key: &GroupKey,
    samples: &[DeltaSample],
) -> GroupJudgements {
    let label = group_label(key);
    let all: Vec<&DeltaSample> = samples.iter().collect();
    let on_holes: Vec<&DeltaSample> = all
        .iter()
        .copied()
        .filter(|sample| sample.missing_txgs >= 1)
        .collect();
    let not_on_holes: Vec<&DeltaSample> = all
        .iter()
        .copied()
        .filter(|sample| sample.missing_txgs == 0)
        .collect();
    let front: Vec<&DeltaSample> = on_holes
        .iter()
        .copied()
        .filter(|sample| sample.shape.position == Some(HolePosition::Front))
        .collect();
    let back: Vec<&DeltaSample> = on_holes
        .iter()
        .copied()
        .filter(|sample| sample.shape.position == Some(HolePosition::Back))
        .collect();
    let by_missing = |sample: &DeltaSample| sample.missing_txgs;
    let mut judge_quantity = |quantity: &str,
                              subset: &[&DeltaSample]|
     -> (PeakJudgement, PeakJudgement, BTreeMap<u64, i64>) {
        let statistics = group_statistics(subset, by_missing);
        let peaks = peaks_of(&statistics);
        let two = judge_peak_against_threshold(&peaks, E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS);
        let one = judge_peak_against_threshold(&peaks, E156_THRESHOLD_ONE_SPAN_ONE_UNIT_SLOTS);
        emitter.emit(&format!(
            "name=q1c {label} quantity={quantity} samples={} judgement_two_slots={} judgement_one_slot={} smallest_h_que_above_two={} smallest_h_que_above_one={} first_crossing_two={} by_h_que={}",
            subset.len(),
            two.label(),
            one.label(),
            smallest_key_above_threshold(&peaks, E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS)
                .map_or_else(|| "none".to_string(), |smallest_key| smallest_key.to_string()),
            smallest_key_above_threshold(&peaks, E156_THRESHOLD_ONE_SPAN_ONE_UNIT_SLOTS)
                .map_or_else(|| "none".to_string(), |smallest_key| smallest_key.to_string()),
            first_crossing_text(subset, by_missing, &peaks, E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS),
            statistics_text(&statistics)
        ));
        (two, one, peaks)
    };
    let (all_points, _, all_peaks) = judge_quantity("q1c", &all);
    let (on_holes_judgement, _, _) = judge_quantity("q1c_on_holes", &on_holes);
    let (not_on_holes_judgement, _, _) = judge_quantity("q1c_not_on_holes", &not_on_holes);
    let (front_judgement, front_one_slot, front_peaks) = judge_quantity("q1c_front", &front);
    let (back_judgement, _, back_peaks) = judge_quantity("q1c_back", &back);

    let growth = judge_growth_with_holes(&all_peaks);
    let adjacent: Vec<String> = all_peaks
        .iter()
        .collect::<Vec<_>>()
        .windows(2)
        .map(|pair| format!("{}->{}:{}", pair[0].0, pair[1].0, pair[1].1 - pair[0].1))
        .collect();
    let by_live_holes = group_statistics(&all, |sample| sample.live_holes);
    let by_hole_count = group_statistics(&all, |sample| sample.shape.hole_count);
    emitter.emit(&format!(
        "name=q1d {label} growth_by_h_que={} adjacent_differences={} first_crossing_two={} first_crossing_one={} by_h_dong={} by_injected_k={}",
        growth.label(),
        if adjacent.is_empty() { "none".to_string() } else { adjacent.join(",") },
        first_crossing_text(&all, by_missing, &all_peaks, E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS),
        first_crossing_text(&all, by_missing, &all_peaks, E156_THRESHOLD_ONE_SPAN_ONE_UNIT_SLOTS),
        statistics_text(&by_live_holes),
        statistics_text(&by_hole_count),
    ));

    // 判别力自证 ②（第八节 8.2）：把 Q1c-前 的门槛挪到真产物 Δ 峰值两个观测值之间，同一份真产物上的判定必须转一次。
    let distinct_front_peaks: BTreeSet<i64> = front_peaks.values().copied().collect();
    if distinct_front_peaks.len() >= 2 {
        let largest = *distinct_front_peaks.iter().next_back().expect("至少两个");
        let second_largest = *distinct_front_peaks.iter().rev().nth(1).expect("至少两个");
        let between = judge_peak_against_threshold(&front_peaks, second_largest);
        let at_largest = judge_peak_against_threshold(&front_peaks, largest);
        emitter.emit(&format!(
            "name=q1c_front_threshold_moved {label} threshold_between={second_largest} judgement_between={} threshold_at_largest={largest} judgement_at_largest={} flips={}",
            between.label(),
            at_largest.label(),
            between != at_largest
        ));
    } else {
        let synthetic: BTreeMap<u64, i64> = [(1u64, 0i64), (2, 5)].into_iter().collect();
        let synthetic_judgement =
            judge_peak_against_threshold(&synthetic, E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS);
        emitter.emit(&format!(
            "name=q1c_front_threshold_moved {label} real_distinct_peaks={} real_product_has_no_two_points_to_move_between=true synthetic_zero_and_five_against_two={} synthetic_is_red={}",
            distinct_front_peaks.len(),
            synthetic_judgement.label(),
            synthetic_judgement == PeakJudgement::AboveTheThreshold
        ));
    }
    GroupJudgements {
        all_points,
        on_holes: on_holes_judgement,
        not_on_holes: not_on_holes_judgement,
        front_holes: front_judgement,
        back_holes: back_judgement,
        front_holes_one_slot: front_one_slot,
        growth,
        front_peak_is_zero_everywhere: front_peaks.values().all(|peak| *peak == 0),
        back_peak_is_zero_everywhere: back_peaks.values().all(|peak| *peak == 0),
    }
}

#[cfg(test)]
mod tests;
