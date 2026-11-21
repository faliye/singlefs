//! 计数：复用臂与轨迹（第六节、8.1）。

use super::*;

// ───────────────────────────── 计数：复用臂、轨迹（第六节、8.1） ─────────────────────────────

/// 轨迹的区间（8.1）：每一段的末尾一个检查点，段内每 2²⁰ 个状态一个；D1-stride 的段按取样状态每 2¹² 个一个。
pub(super) struct TrajectoryIntervals {
    /// 每个区间的第一个序号，递增。
    pub(super) starts: Vec<u64>,
}

impl TrajectoryIntervals {
    pub(super) fn of(
        plan: &LocalPlan<'_>,
        sorted_ordinals: &[u64],
        strided_segments: &BTreeSet<usize>,
    ) -> Self {
        let mut starts = Vec::new();
        let mut current_segment = None;
        let mut states_in_interval = 0u64;
        for ordinal in sorted_ordinals {
            let segment = plan.segment_of(*ordinal);
            let per_checkpoint = if strided_segments.contains(&segment) {
                TRAJECTORY_STRIDE_SAMPLES_PER_CHECKPOINT
            } else {
                TRAJECTORY_STATES_PER_CHECKPOINT
            };
            if current_segment != Some(segment) || states_in_interval == per_checkpoint {
                starts.push(*ordinal);
                current_segment = Some(segment);
                states_in_interval = 0;
            }
            states_in_interval += 1;
        }
        Self { starts }
    }

    pub(super) fn interval_of(&self, ordinal: u64) -> usize {
        self.starts
            .partition_point(|start| *start <= ordinal)
            .saturating_sub(1)
    }

    pub(super) fn count(&self) -> usize {
        self.starts.len()
    }
}

/// 一个量的轨迹（8.1）：每个区间的增量。
pub(super) struct Trajectory {
    increments: Vec<u128>,
}

impl Trajectory {
    pub(super) fn new(intervals: &TrajectoryIntervals) -> Self {
        Self {
            increments: vec![0; intervals.count().max(1)],
        }
    }

    pub(super) fn add(&mut self, interval: usize, amount: u128) {
        self.increments[interval] += amount;
    }

    /// `名字_peak_after_cold_start=… 名字_positive_intervals=… 名字_final=… 名字_last_four=[…]`。
    pub(super) fn fields(&self, name: &str) -> String {
        let peak_after_cold_start = self.increments.iter().skip(1).copied().max().unwrap_or(0);
        let positive = self
            .increments
            .iter()
            .filter(|increment| **increment > 0)
            .count();
        let total: u128 = self.increments.iter().sum();
        let last_four: Vec<String> = self
            .increments
            .iter()
            .rev()
            .take(4)
            .rev()
            .map(u128::to_string)
            .collect();
        format!(
            "{name}_peak_after_cold_start={peak_after_cold_start} {name}_positive_intervals={positive}/{} {name}_final={total} {name}_last_four=[{}]",
            self.increments.len(),
            last_four.join(",")
        )
    }
}

/// 一条复用臂在一格上的计数（第六节 Q2a–Q2c、Q4a–Q4c，G5 P3）。
pub(super) struct ArmCount {
    pub(super) states: u64,
    pub(super) distinct_keys: u64,
    pub(super) hits: u64,
    pub(super) inconsistent: u64,
    /// 每个不同键第一条记录的大小之和（G5 P3 的元素或调用数）。
    pub(super) distinct_key_elements: u64,
    pub(super) new_keys: Trajectory,
    pub(super) inconsistencies: Trajectory,
    /// PC2b / PC4b：键换成常数时的不一致数（与序号最小那个状态结局不同的状态数）。
    pub(super) constant_key_inconsistent: u64,
    pub(super) distinct_outcomes: u64,
    /// 结局不同的两个状态（PC4b 找不到时挑一对强行设同键用）：（序号，序号）。
    pub(super) differing_pair: Option<(u64, u64)>,
}

/// 按（键、序号）顺次走一遍：同键里序号最小那个是参照，其余是命中，结局与参照不同即不一致（Q2a / Q4a）。
pub(super) fn count_arm(
    store: ArmRecordStore,
    intervals: &TrajectoryIntervals,
    copy_into: Option<&mut ArmRecordStore>,
) -> ArmCount {
    let mut count = ArmCount {
        states: 0,
        distinct_keys: 0,
        hits: 0,
        inconsistent: 0,
        distinct_key_elements: 0,
        new_keys: Trajectory::new(intervals),
        inconsistencies: Trajectory::new(intervals),
        constant_key_inconsistent: 0,
        distinct_outcomes: 0,
        differing_pair: None,
    };
    let mut current: Option<(Digest128, Digest128)> = None;
    let mut outcome_counts: HashMap<Digest128, (u64, u64)> = HashMap::new();
    let mut minimum: Option<(u64, Digest128)> = None;
    let mut copy_into = copy_into;
    store.visit_sorted(&mut |record| {
        if let Some(target) = copy_into.as_deref_mut() {
            target.push(record);
        }
        count.states += 1;
        let entry = outcome_counts
            .entry(record.outcome)
            .or_insert((0, record.ordinal));
        entry.0 += 1;
        entry.1 = entry.1.min(record.ordinal);
        if minimum.is_none_or(|(ordinal, _)| record.ordinal < ordinal) {
            minimum = Some((record.ordinal, record.outcome));
        }
        match current {
            Some((key, reference)) if key == record.key => {
                count.hits += 1;
                if record.outcome != reference {
                    count.inconsistent += 1;
                    count
                        .inconsistencies
                        .add(intervals.interval_of(record.ordinal), 1);
                }
            }
            Some(_) | None => {
                current = Some((record.key, record.outcome));
                count.distinct_keys += 1;
                count.distinct_key_elements += u64::from(record.elements);
                count.new_keys.add(intervals.interval_of(record.ordinal), 1);
            }
        }
    });
    if let Some((_ordinal, outcome)) = minimum {
        let same = outcome_counts
            .get(&outcome)
            .map_or(0, |(states, _)| *states);
        count.constant_key_inconsistent = count.states - same;
    }
    count.distinct_outcomes = u64::try_from(outcome_counts.len()).expect("个数");
    let mut firsts: Vec<u64> = outcome_counts.values().map(|(_, first)| *first).collect();
    firsts.sort_unstable();
    if let [first, second, ..] = firsts.as_slice() {
        count.differing_pair = Some((*first, *second));
    }
    count
}

/// 比值的文本：分子 / 分母，六位小数（判定用整数比较，不用这个文本）。
pub(super) fn ratio_text(numerator: u128, denominator: u128) -> String {
    if denominator == 0 {
        return "undefined".to_string();
    }
    let scaled = numerator * 1_000_000 / denominator;
    format!("{}.{:06}", scaled / 1_000_000, scaled % 1_000_000)
}

/// 「比值 ≥ 0.5」的整数判定（登记第一节：接近 1 / 接近状态数 = 比值 ≥ 0.5）。
pub(super) fn reaches_half(numerator: u128, denominator: u128) -> bool {
    ratio_reaches(
        numerator,
        denominator,
        u128::from(RATIO_THRESHOLD_NUMERATOR),
        u128::from(RATIO_THRESHOLD_DENOMINATOR),
    )
}

/// 比值 numerator / denominator ≥ threshold_numerator / threshold_denominator（判别力自证把门槛挪到两点之间时用同一个判定函数）。
pub(super) fn ratio_reaches(
    numerator: u128,
    denominator: u128,
    threshold_numerator: u128,
    threshold_denominator: u128,
) -> bool {
    numerator * threshold_denominator >= denominator * threshold_numerator
}
