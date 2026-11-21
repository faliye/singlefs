//! 各条臂与每臂的字节数、两侧与判定、被判的格。

use super::*;

// ── 各臂每次发布的四样字节、比值与判定（登记第六节） ──

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Arm {
    JiaAfterPatch,
    JiaToday,
    Yi2,
    Bing,
    DingWithoutCompaction,
    DingWholeNewBaseCompaction,
}

impl Arm {
    pub(super) const EVERY_ARM: [Arm; 6] = [Arm::JiaAfterPatch, Arm::JiaToday, Arm::Yi2, Arm::Bing, Arm::DingWithoutCompaction, Arm::DingWholeNewBaseCompaction];

    pub(super) fn label(self) -> &'static str {
        match self {
            Arm::JiaAfterPatch => "jia_after_patch",
            Arm::JiaToday => "jia_today",
            Arm::Yi2 => "yi2_persistent",
            Arm::Bing => "bing_by_value",
            Arm::DingWithoutCompaction => "ding_without_compaction",
            Arm::DingWholeNewBaseCompaction => "ding_whole_new_base_compaction",
        }
    }
}

/// 一条臂一次发布：照抄、现算重建，准入前与准入后各一格（读法第 1 条：两段各报各的）。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(super) struct ArmBytes {
    pub(super) copied_before: u64,
    pub(super) copied_after: u64,
    pub(super) recomputed_before: u64,
    pub(super) recomputed_after: u64,
}

pub(super) fn arm_bytes(geometry: &Geometry, arm: Arm) -> ArmBytes {
    let in_full = recomputed_in_full(geometry);
    let residual = recomputed_residual(geometry);
    let jia_after = jia_after_patch_copied_after_admission(geometry).total();
    let (copied_before, copied_after, recomputed) = match arm {
        Arm::JiaAfterPatch => (jia_after_patch_copied_before_admission(geometry).total(geometry.knobs.settle_rounds), jia_after, in_full),
        Arm::JiaToday => (jia_today_copied_before_admission(geometry), jia_after, in_full),
        Arm::Yi2 => {
            let (before, after) = yi2_copied(geometry);
            (before, after, residual)
        }
        Arm::Bing => (0, bing_copied_after_admission(geometry), residual),
        Arm::DingWithoutCompaction => (0, ding_copied_after_admission(geometry, false, Compaction::WholeNewBase), residual),
        Arm::DingWholeNewBaseCompaction => (0, ding_copied_after_admission(geometry, true, Compaction::WholeNewBase), (residual.0, in_full.1)),
    };
    ArmBytes { copied_before, copied_after, recomputed_before: recomputed.0, recomputed_after: recomputed.1 }
}

/// 一格判定落在门槛的哪一侧；分母 0 判「不低于门槛」（读法第 9 条）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Side {
    BelowThreshold,
    AtOrAboveThreshold,
}

/// 门槛写成分数：分子、分母（判据门槛 2 = (2, 1)）。
pub(super) const JUDGED_THRESHOLD: (u64, u64) = (2, 1);

pub(super) fn side_of(numerator: u64, denominator: u64, threshold: (u64, u64)) -> Side {
    if denominator == 0 || u128::from(numerator) * u128::from(threshold.1) >= u128::from(threshold.0) * u128::from(denominator) {
        Side::AtOrAboveThreshold
    } else {
        Side::BelowThreshold
    }
}

pub(super) fn ratio_text(numerator: u64, denominator: u64) -> String {
    if denominator == 0 {
        "zero_denominator".to_string()
    } else {
        format!("{:.4}", numerator as f64 / denominator as f64)
    }
}

/// 字节比那几行的判定：两种形式都 < 2 倾向留甲、都 ≥ 2 不倾向、分到两侧不稳定（读法第 9 条）。
pub(super) fn verdict_of_sides(sides: &[Side]) -> &'static str {
    if sides.iter().all(|side| *side == Side::BelowThreshold) {
        "leans_keep_jia"
    } else if sides.iter().all(|side| *side == Side::AtOrAboveThreshold) {
        "does_not_lean_keep_jia"
    } else {
        "unstable"
    }
}

/// 比较的三对：甲-B2后 ÷ 乙2、÷ 丙、÷ 丁（丁取非压实那次；读法第 7 条逐对各算）。
pub(super) const COMPARED_ARMS: [Arm; 3] = [Arm::Yi2, Arm::Bing, Arm::DingWithoutCompaction];

/// 判定那几格的一格：哪一对、哪一段、照抄还是现算重建。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct JudgedCell {
    pub(super) compared_arm: Arm,
    pub(super) after_admission: bool,
    pub(super) recomputed: bool,
}

pub(super) fn judged_cells() -> Vec<JudgedCell> {
    let mut judged = Vec::new();
    for recomputed in [false, true] {
        for after_admission in [false, true] {
            for compared_arm in COMPARED_ARMS {
                judged.push(JudgedCell { compared_arm, after_admission, recomputed });
            }
        }
    }
    judged
}

impl JudgedCell {
    pub(super) fn label(&self) -> String {
        let kind = if self.recomputed { "recomputed" } else { "copied" };
        let segment = if self.after_admission { "after_admission" } else { "before_admission" };
        format!("jia_after_patch_over_{}_{kind}_{segment}", self.compared_arm.label())
    }

    /// 这一格的 (分子, 分母) 两种形式：照抄格给形式 A（只照抄）与形式 B（照抄 + 新装）；现算重建格只有一种形式。
    pub(super) fn forms(&self, geometry: &Geometry) -> Vec<(u64, u64)> {
        let jia = arm_bytes(geometry, Arm::JiaAfterPatch);
        let other = arm_bytes(geometry, self.compared_arm);
        let (installed_before, installed_after) = installed_bytes(geometry);
        let pick = |bytes: &ArmBytes| match (self.recomputed, self.after_admission) {
            (false, false) => bytes.copied_before,
            (false, true) => bytes.copied_after,
            (true, false) => bytes.recomputed_before,
            (true, true) => bytes.recomputed_after,
        };
        if self.recomputed {
            vec![(pick(&jia), pick(&other))]
        } else {
            let installed = if self.after_admission { installed_after } else { installed_before };
            vec![(pick(&jia), pick(&other)), (pick(&jia) + installed, pick(&other) + installed)]
        }
    }

    pub(super) fn verdict(&self, geometry: &Geometry, threshold: (u64, u64)) -> &'static str {
        let sides: Vec<Side> = self.forms(geometry).iter().map(|&(numerator, denominator)| side_of(numerator, denominator, threshold)).collect();
        verdict_of_sides(&sides)
    }
}

#[cfg(test)]
#[path = "arms_and_judged_cells_tests.rs"]
mod tests;
