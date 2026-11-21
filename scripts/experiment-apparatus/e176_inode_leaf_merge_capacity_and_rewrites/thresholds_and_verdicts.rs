//! 有理数、判定行与重写记法、阈值与点的判定、总判定与移动阈值的判定。

use super::*;

/// 精确分数（分母恒正），判定一律按交叉相乘比，不经浮点。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rational {
    numerator: i128,
    denominator: i128,
}

impl Rational {
    #[must_use]
    pub fn new(numerator: i128, denominator: i128) -> Self {
        assert!(denominator != 0, "分母不为 0");
        if denominator < 0 {
            Self { numerator: -numerator, denominator: -denominator }
        } else {
            Self { numerator, denominator }
        }
    }

    #[must_use]
    pub fn compare(self, other: Self) -> std::cmp::Ordering {
        (self.numerator * other.denominator).cmp(&(other.numerator * self.denominator))
    }

    #[must_use]
    pub fn midpoint(self, other: Self) -> Self {
        Self::new(self.numerator * other.denominator + other.numerator * self.denominator, 2 * self.denominator * other.denominator)
    }

    /// 四舍五入（远离 0）到 `places` 位小数的十进制写法。
    #[must_use]
    pub fn decimal(self, places: u32) -> String {
        let scale = 10_i128.pow(places);
        let magnitude = self.numerator.abs() * scale;
        let rounded = (2 * magnitude + self.denominator) / (2 * self.denominator);
        let sign = if self.numerator < 0 && rounded != 0 { "-" } else { "" };
        let width = usize::try_from(places).expect("位数很小");
        format!("{sign}{}.{:0width$}", rounded / scale, rounded % scale)
    }
}

/// Q1r 的门槛（第六节）：R12 ≥ 0.90 判「不合并的容量代价小」，R12 < 0.50 判「合并买到成倍容量」。
pub const CAPACITY_RATIO_SMALL_COST_AT_LEAST: Rational = Rational { numerator: 90, denominator: 100 };
pub const CAPACITY_RATIO_DOUBLED_BELOW: Rational = Rational { numerator: 50, denominator: 100 };
/// Q2、Q3 的门槛（第六节）：E ≤ 0.10 判「可忽略」，E ≥ 1.00 判「显著」。
pub const WRITE_COST_NEGLIGIBLE_AT_MOST: Rational = Rational { numerator: 10, denominator: 100 };
pub const WRITE_COST_SIGNIFICANT_AT_LEAST: Rational = Rational { numerator: 100, denominator: 100 };

/// 登记的判定行（第六节「量与判据」那张表）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JudgedRow {
    /// Q1r：L_M1 ÷ L_M2。
    CapacityRatio,
    /// Q2p / Q2c / Q3：两条臂（窗口里的槽位）每次 unlink 平均重写片数之差，`minuend_slot` − `subtrahend_slot`。
    WriteCost { name: &'static str, minuend_slot: usize, subtrahend_slot: usize, notation: RewriteNotation, significant_side_counts: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewriteNotation {
    Position,
    Content,
}

impl JudgedRow {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            JudgedRow::CapacityRatio => "Q1r",
            JudgedRow::WriteCost { name, .. } => name,
        }
    }
}

/// 第六节的七行，次序就是产物里的次序。Q2c 的 Δ2c 那一行「≥ 1.00」一侧由臂的定义限死，不算判据（第四节）。
pub const JUDGED_ROWS: [JudgedRow; 7] = [
    JudgedRow::CapacityRatio,
    JudgedRow::WriteCost { name: "Q2p_delta2", minuend_slot: 1, subtrahend_slot: 0, notation: RewriteNotation::Position, significant_side_counts: true },
    JudgedRow::WriteCost { name: "Q2p_delta3", minuend_slot: 2, subtrahend_slot: 0, notation: RewriteNotation::Position, significant_side_counts: true },
    JudgedRow::WriteCost { name: "Q2c_delta2", minuend_slot: 1, subtrahend_slot: 0, notation: RewriteNotation::Content, significant_side_counts: false },
    JudgedRow::WriteCost { name: "Q2c_delta3", minuend_slot: 2, subtrahend_slot: 0, notation: RewriteNotation::Content, significant_side_counts: true },
    JudgedRow::WriteCost { name: "Q3p_delta32", minuend_slot: 2, subtrahend_slot: 1, notation: RewriteNotation::Position, significant_side_counts: true },
    JudgedRow::WriteCost { name: "Q3c_delta32", minuend_slot: 2, subtrahend_slot: 1, notation: RewriteNotation::Content, significant_side_counts: true },
];

/// 一个取样点上一行的读数：值（读不到或不判时是 None）与旁注。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointValue {
    pub value: Option<Rational>,
    pub note: &'static str,
}

/// 一行的门槛（自证判别力时会把其中一个挪到两个取样点的值之间）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Thresholds {
    pub lower: Rational,
    pub upper: Rational,
}

#[must_use]
pub fn registered_thresholds(row: JudgedRow) -> Thresholds {
    match row {
        JudgedRow::CapacityRatio => Thresholds { lower: CAPACITY_RATIO_DOUBLED_BELOW, upper: CAPACITY_RATIO_SMALL_COST_AT_LEAST },
        JudgedRow::WriteCost { .. } => Thresholds { lower: WRITE_COST_NEGLIGIBLE_AT_MOST, upper: WRITE_COST_SIGNIFICANT_AT_LEAST },
    }
}

/// 一个取样点上一行的判定（第六节门槛，按点各判一次）。
#[must_use]
pub fn point_verdict(row: JudgedRow, point: PointValue, thresholds: Thresholds) -> &'static str {
    let Some(value) = point.value else { return point.note };
    if point.blocks_judgement() {
        return point.note;
    }
    match row {
        JudgedRow::CapacityRatio => {
            if value.compare(thresholds.upper).is_ge() {
                "no_merge_capacity_cost_small"
            } else if value.compare(thresholds.lower).is_lt() {
                "merge_buys_double_capacity"
            } else {
                "no_flip"
            }
        }
        JudgedRow::WriteCost { significant_side_counts, .. } => {
            if value.compare(thresholds.lower).is_le() {
                "negligible"
            } else if significant_side_counts && value.compare(thresholds.upper).is_ge() {
                "significant"
            } else {
                "no_flip"
            }
        }
    }
}

impl PointValue {
    /// 「样本不足」（公共窗口里最后一批不足 100 次）与「M1 未撞到」（F6）报数不判；「上界」（M2 吃满上限没被拒）照判、带着旁注。
    #[must_use]
    pub fn blocks_judgement(self) -> bool {
        matches!(self.note, "insufficient_sample" | "m1_not_reached")
    }
}

/// 一条序列上一行的读数。`window_index` 0 是主窗口（M3 主），1 是 f = 59 那个窗口（第 2 槽换成 M3 f = 59）。
#[must_use]
pub fn point_value(result: &SequenceResult, row: JudgedRow, window_index: usize) -> PointValue {
    match row {
        JudgedRow::CapacityRatio => {
            let drop_empty_only_trajectory = &result.trajectories[0];
            let absorb_right_trajectory = &result.trajectories[1];
            if drop_empty_only_trajectory.stop == ArmStop::ReachedOperationCap {
                return PointValue { value: None, note: "m1_not_reached" };
            }
            let note = if absorb_right_trajectory.stop == ArmStop::ReachedOperationCap { "upper_bound" } else { "" };
            PointValue { value: Some(Rational::new(i128::from(drop_empty_only_trajectory.live_inodes_at_stop), i128::from(absorb_right_trajectory.live_inodes_at_stop))), note }
        }
        JudgedRow::WriteCost { minuend_slot, subtrahend_slot, notation, .. } => {
            let window = &result.windows[window_index];
            if window.unlinks == 0 {
                return PointValue { value: None, note: "insufficient_sample" };
            }
            let sum_of = |slot: usize| -> i128 {
                match notation {
                    RewriteNotation::Position => i128::from(window.rewrites[slot].position_sum),
                    RewriteNotation::Content => i128::from(window.rewrites[slot].content_sum),
                }
            };
            let value = Rational::new(sum_of(minuend_slot) - sum_of(subtrahend_slot), i128::from(window.unlinks));
            let note = if window.unlinks < UNLINKS_IN_THE_WINDOW_FOR_A_JUDGEMENT_AT_LEAST { "insufficient_sample" } else { "" };
            PointValue { value: Some(value), note }
        }
    }
}

/// 一行在全部登记点上的总判定（第六节）：Q1r 任一主取样点 < 0.50 ⇒ 成倍；每一个登记点都 ≥ 0.90 ⇒ 代价小；其余不翻。
/// Q2、Q3 任一主取样点 ≥ 1.00 ⇒ 显著（Δ2c 不算这一侧）；每一个判得了的登记点都 ≤ 0.10 ⇒ 可忽略；其余不翻。样本不足的点不进判定。
#[must_use]
pub fn overall_verdict(results: &[SequenceResult], row: JudgedRow, window_index: usize) -> &'static str {
    let thresholds = registered_thresholds(row);
    let points: Vec<(bool, PointValue)> = results.iter().map(|result| (result.sequence.is_main_sampling_point, point_value(result, row, window_index))).collect();
    match row {
        JudgedRow::CapacityRatio => {
            let doubled = points.iter().any(|(is_main, point)| {
                *is_main && !point.blocks_judgement() && point.value.is_some_and(|value| value.compare(thresholds.lower).is_lt())
            });
            let all_small = points.iter().all(|(_, point)| !point.blocks_judgement() && point.value.is_some_and(|value| value.compare(thresholds.upper).is_ge()));
            if doubled {
                "merge_buys_double_capacity"
            } else if all_small {
                "no_merge_capacity_cost_small"
            } else {
                "no_flip"
            }
        }
        JudgedRow::WriteCost { significant_side_counts, .. } => {
            let judged: Vec<&(bool, PointValue)> = points.iter().filter(|(_, point)| !point.blocks_judgement() && point.value.is_some()).collect();
            let significant = significant_side_counts
                && judged.iter().any(|(is_main, point)| *is_main && point.value.is_some_and(|value| value.compare(thresholds.upper).is_ge()));
            let all_negligible = !judged.is_empty() && judged.iter().all(|(_, point)| point.value.is_some_and(|value| value.compare(thresholds.lower).is_le()));
            if significant {
                "significant"
            } else if all_negligible {
                "negligible"
            } else {
                "no_flip"
            }
        }
    }
}

/// 判别力自证（第八节末）：两点判定相同时，把一个门槛挪到两点的值之间，判定必须转成不同。交回挪后的门槛与两点挪后的判定。
#[must_use]
pub fn moved_threshold_verdicts(row: JudgedRow, main: PointValue, opposite: PointValue) -> Option<(Thresholds, &'static str, &'static str)> {
    let (main_value, opposite_value) = (main.value?, opposite.value?);
    if main_value.compare(opposite_value).is_eq() {
        return None;
    }
    let registered = registered_thresholds(row);
    let between = main_value.midpoint(opposite_value);
    let smaller = if main_value.compare(opposite_value).is_lt() { main_value } else { opposite_value };
    let moved = match row {
        JudgedRow::CapacityRatio => {
            if smaller.compare(registered.lower).is_lt() && main_value.maximum_with(opposite_value).compare(registered.lower).is_lt() {
                Thresholds { lower: between, upper: registered.upper }
            } else {
                Thresholds { lower: registered.lower, upper: between }
            }
        }
        JudgedRow::WriteCost { significant_side_counts, .. } => {
            let larger = main_value.maximum_with(opposite_value);
            if significant_side_counts && smaller.compare(registered.upper).is_ge() && larger.compare(registered.upper).is_ge() {
                Thresholds { lower: registered.lower, upper: between }
            } else {
                Thresholds { lower: between, upper: registered.upper }
            }
        }
    };
    Some((moved, point_verdict(row, main, moved), point_verdict(row, opposite, moved)))
}

impl Rational {
    #[must_use]
    pub fn maximum_with(self, other: Self) -> Self {
        if self.compare(other).is_ge() {
            self
        } else {
            other
        }
    }
}

#[cfg(test)]
#[path = "thresholds_and_verdicts_tests.rs"]
mod tests;
