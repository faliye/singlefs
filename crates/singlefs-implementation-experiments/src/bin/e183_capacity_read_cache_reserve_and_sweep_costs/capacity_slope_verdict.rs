use super::*;

// ============================================================================================
// E37 里 H2 的判定（纯函数）：X10 的斜率、斜率的类、5 轮同类才写类、判别力自证 ①。
// ============================================================================================

/// X10：(log₂ x, log₂ y) 的最小二乘斜率；y 为 0 的点按 1 取对数，另记「有 0 点」。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SlopeReading {
    pub(super) slope: f64,
    pub(super) has_zero_point: bool,
    pub(super) points: usize,
}

/// 点数不足两个、或 x 全相同（斜率没定义）交回 `None`。
pub(super) fn log_log_least_squares_slope(points: &[(u64, u64)]) -> Option<SlopeReading> {
    let has_zero_point = points.iter().any(|(_, ordinate)| *ordinate == 0);
    let logarithms: Vec<(f64, f64)> = points
        .iter()
        .map(|(abscissa, ordinate)| {
            (
                log2_of_a_count(*abscissa),
                log2_of_a_count((*ordinate).max(1)),
            )
        })
        .collect();
    let count = logarithms.len();
    if count < 2 {
        return None;
    }
    let count_as_float = f64::from(u32::try_from(count).expect("点数装得进 u32"));
    let mean_abscissa =
        logarithms.iter().map(|(abscissa, _)| abscissa).sum::<f64>() / count_as_float;
    let mean_ordinate =
        logarithms.iter().map(|(_, ordinate)| ordinate).sum::<f64>() / count_as_float;
    let abscissa_spread: f64 = logarithms
        .iter()
        .map(|(abscissa, _)| (abscissa - mean_abscissa) * (abscissa - mean_abscissa))
        .sum();
    if abscissa_spread == 0.0 {
        return None;
    }
    let covariance: f64 = logarithms
        .iter()
        .map(|(abscissa, ordinate)| (abscissa - mean_abscissa) * (ordinate - mean_ordinate))
        .sum();
    Some(SlopeReading {
        slope: covariance / abscissa_spread,
        has_zero_point,
        points: count,
    })
}

/// 计数取以 2 为底的对数（计数都在 2⁵³ 以内，转成浮点不丢值的那一段由断言守着）。
fn log2_of_a_count(count: u64) -> f64 {
    assert!(count > 0, "取对数的计数要大于 0：0 点在调用方按 1 取");
    assert!(count < (1u64 << 53), "计数装得进双精度的整数段");
    #[allow(
        clippy::cast_precision_loss,
        reason = "上一行断言了计数小于 2 的 53 次方，转成双精度不丢值"
    )]
    let as_float = count as f64;
    as_float.log2()
}

/// 斜率的类（登记第六节开头）：b ≥ 0.8 随 X 线性，0.2 ≤ b < 0.8 次线性，b < 0.2 与 X 基本无关。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum SlopeClass {
    BasicallyUnrelated,
    Sublinear,
    Linear,
}

impl SlopeClass {
    pub(super) fn name(self) -> &'static str {
        match self {
            SlopeClass::BasicallyUnrelated => "basically_unrelated",
            SlopeClass::Sublinear => "sublinear",
            SlopeClass::Linear => "linear",
        }
    }
}

pub(super) const LINEAR_SLOPE_BOUNDARY: f64 = 0.8;
pub(super) const UNRELATED_SLOPE_BOUNDARY: f64 = 0.2;

fn slope_class(slope: f64) -> SlopeClass {
    slope_class_with_linear_boundary(slope, LINEAR_SLOPE_BOUNDARY)
}

fn slope_class_with_linear_boundary(slope: f64, linear_boundary: f64) -> SlopeClass {
    if slope >= linear_boundary {
        SlopeClass::Linear
    } else if slope >= UNRELATED_SLOPE_BOUNDARY {
        SlopeClass::Sublinear
    } else {
        SlopeClass::BasicallyUnrelated
    }
}

/// 有随机性的量每轮各算一个 b，5 轮落在同一类才写那一类，否则记「不稳定」；有一轮算不出 b（点不够）记「判不出」。
#[derive(Clone, Debug, PartialEq)]
pub(super) enum ClassAcrossRounds {
    Stable(SlopeClass),
    Unstable,
    NotJudged {
        rounds_with_a_slope: usize,
    },
    /// 登记的 X5（覆盖写第 0 个数据单元以内的 4096 字节）在这一组格上造不出：不是 0、不是「基本无关」，这一行不出类（登记第十二节修订）。
    RegisteredPublishCannotBeMade,
}

impl ClassAcrossRounds {
    pub(super) fn text(&self) -> String {
        match self {
            ClassAcrossRounds::Stable(class) => class.name().to_string(),
            ClassAcrossRounds::Unstable => "unstable".to_string(),
            ClassAcrossRounds::RegisteredPublishCannotBeMade => {
                "x5_as_registered_cannot_be_made".to_string()
            }
            ClassAcrossRounds::NotJudged {
                rounds_with_a_slope,
            } => {
                format!("not_judged_rounds_with_a_slope_{rounds_with_a_slope}")
            }
        }
    }
}

pub(super) fn class_across_rounds(
    slopes: &[Option<f64>],
    rounds_required: usize,
) -> ClassAcrossRounds {
    let known: Vec<f64> = slopes.iter().flatten().copied().collect();
    if known.len() < rounds_required {
        return ClassAcrossRounds::NotJudged {
            rounds_with_a_slope: known.len(),
        };
    }
    let classes: BTreeSet<SlopeClass> = known.iter().map(|slope| slope_class(*slope)).collect();
    match classes.iter().next() {
        Some(class) if classes.len() == 1 => ClassAcrossRounds::Stable(*class),
        Some(_) | None => ClassAcrossRounds::Unstable,
    }
}

/// 斜率写进产物：千分之一精度的定点数，免得浮点的末位随平台变。
pub(super) fn slope_text(slope: Option<f64>) -> String {
    slope.map_or_else(|| "none".to_string(), |value| format!("{value:.3}"))
}

/// 判别力自证 ①（登记第八节）：把 0.8 那条分界挪到真产物里两个相邻的 b 之间，至少一个 b 的类必须转。
/// 挑的是「挪过去之后至少转一个」的相邻两点里中点离 0.8 最近的那一对；一对都没有（b 只有一个值，或只跨着 0.8 的那一对）交回 `None`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MovedBoundary {
    pub(super) boundary: f64,
    pub(super) flipped: usize,
}

pub(super) fn linear_boundary_moved_between_real_slopes(slopes: &[f64]) -> Option<MovedBoundary> {
    let mut sorted: Vec<f64> = slopes.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted.dedup();
    sorted
        .windows(2)
        .map(|pair| (pair[0] + pair[1]) / 2.0)
        .map(|boundary| MovedBoundary {
            boundary,
            flipped: slopes
                .iter()
                .filter(|slope| {
                    slope_class(**slope) != slope_class_with_linear_boundary(**slope, boundary)
                })
                .count(),
        })
        .filter(|moved| moved.flipped > 0)
        .min_by(|left, right| {
            (left.boundary - LINEAR_SLOPE_BOUNDARY)
                .abs()
                .total_cmp(&(right.boundary - LINEAR_SLOPE_BOUNDARY).abs())
        })
}

/// 真产物的 b 挪不出一次转面时的代用（登记第八节「喂合成的 (b = 0.5, 0.9) 必须判出两类」）。
const SYNTHETIC_SLOPES_FOR_THE_SELF_PROOF: [f64; 2] = [0.5, 0.9];

pub(super) fn synthetic_slopes_fall_in_two_classes() -> bool {
    slope_class(SYNTHETIC_SLOPES_FOR_THE_SELF_PROOF[0])
        != slope_class(SYNTHETIC_SLOPES_FOR_THE_SELF_PROOF[1])
}

/// 中位数（偶数个时取靠下的那一个，量都是整数，不取平均）。空表交回 `None`。
pub(super) fn median_of(values: &[u64]) -> Option<u64> {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted.get(sorted.len().saturating_sub(1) / 2).copied()
}

#[cfg(test)]
mod tests;
