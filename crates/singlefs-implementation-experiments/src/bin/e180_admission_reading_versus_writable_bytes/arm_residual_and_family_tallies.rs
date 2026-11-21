//! 累计：各臂、残差、轨迹、产物行与一族的合计。

use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ArmTally {
    pub(super) over_states: u64,
    pub(super) not_over_states: u64,
    pub(super) not_over_below_lower_bound_states: u64,
    pub(super) undetermined_states: u64,
    pub(super) maximum_over_bytes: u64,
    pub(super) maximum_over_at: String,
    pub(super) maximum_under_bytes: u64,
    pub(super) maximum_under_at: String,
    pub(super) sum_under_bytes: u128,
    pub(super) under_states: u64,
}

impl ArmTally {
    pub(super) fn count(&mut self, judgement: &Judgement, state_name: &str) {
        match judgement {
            Judgement::OverReports { over_bytes } => {
                self.over_states += 1;
                if *over_bytes > self.maximum_over_bytes {
                    self.maximum_over_bytes = *over_bytes;
                    self.maximum_over_at = state_name.to_string();
                }
            }
            Judgement::DoesNotOverReport { under_bytes } => {
                self.not_over_states += 1;
                if *under_bytes > 0 {
                    self.under_states += 1;
                    self.sum_under_bytes += u128::from(*under_bytes);
                }
                if *under_bytes > self.maximum_under_bytes {
                    self.maximum_under_bytes = *under_bytes;
                    self.maximum_under_at = state_name.to_string();
                }
            }
            Judgement::CertainlyDoesNotOverReportBelowTheLowerBound => {
                self.not_over_below_lower_bound_states += 1;
            }
            Judgement::Undetermined => self.undetermined_states += 1,
        }
    }

    pub(super) fn merge(&mut self, other: &ArmTally) {
        self.over_states += other.over_states;
        self.not_over_states += other.not_over_states;
        self.not_over_below_lower_bound_states += other.not_over_below_lower_bound_states;
        self.undetermined_states += other.undetermined_states;
        if other.maximum_over_bytes > self.maximum_over_bytes {
            self.maximum_over_bytes = other.maximum_over_bytes;
            self.maximum_over_at.clone_from(&other.maximum_over_at);
        }
        if other.maximum_under_bytes > self.maximum_under_bytes {
            self.maximum_under_bytes = other.maximum_under_bytes;
            self.maximum_under_at.clone_from(&other.maximum_under_at);
        }
        self.sum_under_bytes += other.sum_under_bytes;
        self.under_states += other.under_states;
    }
}

/// 一对「残差」的计数（登记 5.2、第六节 Q5–Q7），一块盘一份。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ResidualTally {
    pub(super) states: u64,
    pub(super) rewritten_check_residual_nonzero_states: u64,
    pub(super) original_formula_residual_nonzero_states: u64,
    pub(super) maximum_absolute_rewritten_check_residual: i128,
    pub(super) maximum_absolute_original_formula_residual: i128,
    pub(super) states_with_overlap: u64,
    pub(super) states_with_held_ledger: u64,
    /// 阳性对照：改写检查残差去掉重叠项之后在重叠 > 0 的状态上必须 ≠ 0；这里数它没 ≠ 0 的状态。
    pub(super) control_without_overlap_term_stayed_zero: u64,
    pub(super) first_state_with_a_nonzero_rewritten_check_residual: String,
    pub(super) first_state_with_a_nonzero_original_formula_residual: String,
}

impl ResidualTally {
    pub(super) fn merge(&mut self, other: &ResidualTally) {
        self.states += other.states;
        self.rewritten_check_residual_nonzero_states +=
            other.rewritten_check_residual_nonzero_states;
        self.original_formula_residual_nonzero_states +=
            other.original_formula_residual_nonzero_states;
        self.maximum_absolute_rewritten_check_residual = self
            .maximum_absolute_rewritten_check_residual
            .max(other.maximum_absolute_rewritten_check_residual);
        self.maximum_absolute_original_formula_residual = self
            .maximum_absolute_original_formula_residual
            .max(other.maximum_absolute_original_formula_residual);
        self.states_with_overlap += other.states_with_overlap;
        self.states_with_held_ledger += other.states_with_held_ledger;
        self.control_without_overlap_term_stayed_zero +=
            other.control_without_overlap_term_stayed_zero;
        if self
            .first_state_with_a_nonzero_rewritten_check_residual
            .is_empty()
        {
            self.first_state_with_a_nonzero_rewritten_check_residual
                .clone_from(&other.first_state_with_a_nonzero_rewritten_check_residual);
        }
        if self
            .first_state_with_a_nonzero_original_formula_residual
            .is_empty()
        {
            self.first_state_with_a_nonzero_original_formula_residual
                .clone_from(&other.first_state_with_a_nonzero_original_formula_residual);
        }
    }
}

/// 被条款当谓词输入的量的轨迹（登记第八节）：初始供给（第一个状态）、之后的峰值、为正的状态数、期末值。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct TrajectoryTally {
    pub(super) states: u64,
    pub(super) first_value: i128,
    pub(super) peak_after_the_first: Option<i128>,
    pub(super) positive_states_after_the_first: u64,
    pub(super) last_value: i128,
}

impl TrajectoryTally {
    pub(super) fn record(&mut self, value: i128) {
        if self.states == 0 {
            self.first_value = value;
        } else {
            self.peak_after_the_first = Some(
                self.peak_after_the_first
                    .map_or(value, |peak| peak.max(value)),
            );
            if value > 0 {
                self.positive_states_after_the_first += 1;
            }
        }
        self.last_value = value;
        self.states += 1;
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// 一族历史上的累计与产物行
// ---------------------------------------------------------------------------------------------------------------------

pub(super) struct ProductLine {
    pub(super) text: String,
}

impl ProductLine {
    pub(super) fn new(name: &str) -> Self {
        ProductLine {
            text: format!("E name={name}"),
        }
    }

    pub(super) fn field(mut self, key: &str, value: impl std::fmt::Display) -> Self {
        self.text.push(' ');
        self.text.push_str(key);
        self.text.push('=');
        self.text.push_str(&value.to_string());
        self
    }

    pub(super) fn finish(self) -> String {
        self.text
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct FamilyTally {
    pub(super) states: u64,
    pub(super) exact_states: u64,
    pub(super) censored_states: u64,
    pub(super) unreadable_states: u64,
    pub(super) arms: BTreeMap<Arm, ArmTally>,
    pub(super) residuals: Vec<ResidualTally>,
    pub(super) closed_minus_ring_window: Vec<TrajectoryTally>,
    pub(super) closed_alone: Vec<TrajectoryTally>,
    pub(super) closed_minus_all_common_commitments: Vec<TrajectoryTally>,
    pub(super) control_failures: Vec<String>,
    pub(super) shortcut_pairs_checked: u64,
    pub(super) shortcut_differences: Vec<String>,
    pub(super) records_disagree_with_the_disk_states: u64,
    pub(super) decomposition_failure_states: u64,
    pub(super) not_whole_slot_available_states: u64,
    pub(super) lag_unattributed_states: u64,
    pub(super) non_monotone_states: u64,
    pub(super) probes_run: u64,
    pub(super) stop_clause_held_ledger_mismatch_states: u64,
    pub(super) unmount_pushes_checked: u64,
    pub(super) unmount_pushes_differing: Vec<String>,
    pub(super) panics: Vec<String>,
}

impl FamilyTally {
    pub(super) fn new(device_count: usize) -> Self {
        FamilyTally {
            residuals: vec![ResidualTally::default(); device_count],
            closed_minus_ring_window: vec![TrajectoryTally::default(); device_count],
            closed_alone: vec![TrajectoryTally::default(); device_count],
            closed_minus_all_common_commitments: vec![TrajectoryTally::default(); device_count],
            ..FamilyTally::default()
        }
    }

    /// 同一几何格里几族的累计并起来（轨迹不并：轨迹按族报）。
    pub(super) fn merge_counts_of(&mut self, other: &FamilyTally) {
        self.states += other.states;
        self.exact_states += other.exact_states;
        self.censored_states += other.censored_states;
        self.unreadable_states += other.unreadable_states;
        for (arm, tally) in &other.arms {
            self.arms.entry(*arm).or_default().merge(tally);
        }
        if self.residuals.len() < other.residuals.len() {
            self.residuals = vec![ResidualTally::default(); other.residuals.len()];
        }
        for (mine, theirs) in self.residuals.iter_mut().zip(&other.residuals) {
            mine.merge(theirs);
        }
        self.control_failures
            .extend(other.control_failures.iter().cloned());
        self.shortcut_pairs_checked += other.shortcut_pairs_checked;
        self.shortcut_differences
            .extend(other.shortcut_differences.iter().cloned());
        self.records_disagree_with_the_disk_states += other.records_disagree_with_the_disk_states;
        self.decomposition_failure_states += other.decomposition_failure_states;
        self.not_whole_slot_available_states += other.not_whole_slot_available_states;
        self.lag_unattributed_states += other.lag_unattributed_states;
        self.non_monotone_states += other.non_monotone_states;
        self.probes_run += other.probes_run;
        self.stop_clause_held_ledger_mismatch_states +=
            other.stop_clause_held_ledger_mismatch_states;
        self.unmount_pushes_checked += other.unmount_pushes_checked;
        self.unmount_pushes_differing
            .extend(other.unmount_pushes_differing.iter().cloned());
        self.panics.extend(other.panics.iter().cloned());
    }
}

#[cfg(test)]
mod tests;
