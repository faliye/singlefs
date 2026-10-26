//! 两份独立 GPU 核的比对与连续不确定计数（D26（后台整理与放置回收） 已定项 13 第 8 条；用户 2026-10-26 定「N=2，计数只放内存」）。
//! 与哪一段无关：扫描里的校验和核对、区域扫描的单元头解析与映射反查、选段打分三段都交「每条两份核的结果」进来，结果是什么类型由那一段定。
//!
//! 逐条比两份的结果：相同算确认；不同算不确定，这一批不回收、不搬，留到下一轮重算；同一条目连续
//! [`CONSECUTIVE_UNCERTAIN_ROUNDS_BEFORE_THE_CPU_REFERENCE_JUDGES`] 轮都不确定，交 CPU 那一份实现单独判一次，按 CPU 的判定处理。
//! 计数只住在 [`ConsecutiveUncertaintyCounts`] 这个对象里：丢掉它（重挂）就清零，不写盘。
//! 一轮里确认或交 CPU 判过的条目，计数清零：「连续」从下一次不确定重新数起。没出现在这一批里的条目，计数原样留着。

use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

/// 第 8 条的 N：同一条目连续这么多轮不确定，交 CPU 判。
pub const CONSECUTIVE_UNCERTAIN_ROUNDS_BEFORE_THE_CPU_REFERENCE_JUDGES: u32 = 2;

/// 一条条目两份核各算出的结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TwoKernelOutcomesOfAnEntry<EntryKey, KernelOutcome> {
    pub entry_key: EntryKey,
    pub outcome_by_one_kernel: KernelOutcome,
    pub outcome_by_the_independently_written_kernel: KernelOutcome,
}

/// 一条条目这一轮的判定。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryDecision<KernelOutcome> {
    /// 两份核相同：照这个结果交 CPU 走发布路径。
    ConfirmedByBothKernels(KernelOutcome),
    /// 两份核不同：这一批不回收、不搬，下一轮重算。
    UncertainThisRound,
    /// 连续 N 轮不确定，CPU 那一份实现单独判出的结果：按它处理。
    JudgedByTheCpuReference(KernelOutcome),
}

/// 一批的判定（按交进来的次序，一条一个）与三个计数；三个计数加起来等于条目数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReconciledBatch<KernelOutcome> {
    pub decisions: Vec<EntryDecision<KernelOutcome>>,
    pub confirmed_entries: usize,
    pub uncertain_entries: usize,
    pub entries_judged_by_the_cpu_reference: usize,
}

/// 每个条目已经连续几轮不确定：只在内存里，随这个对象活。
#[derive(Debug)]
pub struct ConsecutiveUncertaintyCounts<EntryKey> {
    uncertain_rounds_by_entry: HashMap<EntryKey, u32>,
}

impl<EntryKey: Eq + Hash + Clone> ConsecutiveUncertaintyCounts<EntryKey> {
    /// 挂载那一刻：没有一个条目不确定过。
    #[must_use]
    pub fn with_no_entry_uncertain() -> Self {
        Self {
            uncertain_rounds_by_entry: HashMap::new(),
        }
    }

    /// 这个条目此刻已经连续几轮不确定。
    #[must_use]
    pub fn consecutive_uncertain_rounds_of(&self, entry_key: &EntryKey) -> u32 {
        self.uncertain_rounds_by_entry
            .get(entry_key)
            .copied()
            .unwrap_or(0)
    }

    /// 一批交 GPU 的结局进来：两份核都算完了就照 [`Self::reconcile_the_two_kernel_outcomes`] 比、更新计数；
    /// 这一批出错（GPU 出错、等 GPU 超过上限、这一批交不成）就原样交回出错，计数一条都不动、CPU 参照一次都不调（第 4 条：这一批放弃，不算一轮）。
    /// 三段（校验和核对、区域扫描的单元头解析与映射反查、选段打分）交回判定都走这一处。
    ///
    /// # Errors
    /// `both_kernel_outcomes` 是 `Err` 时交回它。
    ///
    /// # Panics
    /// 同 [`Self::reconcile_the_two_kernel_outcomes`]。
    pub fn reconcile_unless_the_batch_failed<KernelOutcome: PartialEq, BatchFailure>(
        &mut self,
        both_kernel_outcomes: Result<
            Vec<TwoKernelOutcomesOfAnEntry<EntryKey, KernelOutcome>>,
            BatchFailure,
        >,
        judge_with_the_cpu_reference: impl FnMut(usize) -> KernelOutcome,
    ) -> Result<ReconciledBatch<KernelOutcome>, BatchFailure> {
        match both_kernel_outcomes {
            Ok(entries) => {
                Ok(self.reconcile_the_two_kernel_outcomes(entries, judge_with_the_cpu_reference))
            }
            Err(batch_failure) => Err(batch_failure),
        }
    }

    /// 比一批两份核的结果、更新计数、交回判定。连续不确定到了 N 轮的那几条，调 `judge_with_the_cpu_reference`（参数是它在这一批里的位置）
    /// 现判一次；别的条目不调它。GPU 出错、这一批放弃时不调这个函数：计数照旧。
    ///
    /// # Panics
    /// 同一个条目在一批里出现两次：一批里每条一个条目，重复是调用方的 bug。
    pub fn reconcile_the_two_kernel_outcomes<KernelOutcome: PartialEq>(
        &mut self,
        entries: Vec<TwoKernelOutcomesOfAnEntry<EntryKey, KernelOutcome>>,
        mut judge_with_the_cpu_reference: impl FnMut(usize) -> KernelOutcome,
    ) -> ReconciledBatch<KernelOutcome> {
        let mut entries_seen_in_this_batch: HashSet<EntryKey> =
            HashSet::with_capacity(entries.len());
        let mut decisions = Vec::with_capacity(entries.len());
        let mut confirmed_entries = 0;
        let mut uncertain_entries = 0;
        let mut entries_judged_by_the_cpu_reference = 0;
        for (entry_position, entry) in entries.into_iter().enumerate() {
            assert!(
                entries_seen_in_this_batch.insert(entry.entry_key.clone()),
                "同一个条目在一批里出现了两次"
            );
            if entry.outcome_by_one_kernel == entry.outcome_by_the_independently_written_kernel {
                self.uncertain_rounds_by_entry.remove(&entry.entry_key);
                confirmed_entries += 1;
                decisions.push(EntryDecision::ConfirmedByBothKernels(
                    entry.outcome_by_one_kernel,
                ));
                continue;
            }
            let uncertain_rounds = self.consecutive_uncertain_rounds_of(&entry.entry_key) + 1;
            if uncertain_rounds >= CONSECUTIVE_UNCERTAIN_ROUNDS_BEFORE_THE_CPU_REFERENCE_JUDGES {
                self.uncertain_rounds_by_entry.remove(&entry.entry_key);
                entries_judged_by_the_cpu_reference += 1;
                decisions.push(EntryDecision::JudgedByTheCpuReference(
                    judge_with_the_cpu_reference(entry_position),
                ));
                continue;
            }
            self.uncertain_rounds_by_entry
                .insert(entry.entry_key, uncertain_rounds);
            uncertain_entries += 1;
            decisions.push(EntryDecision::UncertainThisRound);
        }
        ReconciledBatch {
            decisions,
            confirmed_entries,
            uncertain_entries,
            entries_judged_by_the_cpu_reference,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 假的条目：键是一个数，结果是一个数（两份核交回的数相同就算相同）。
    fn entry(
        entry_key: u32,
        outcome_by_one_kernel: u32,
        outcome_by_the_independently_written_kernel: u32,
    ) -> TwoKernelOutcomesOfAnEntry<u32, u32> {
        TwoKernelOutcomesOfAnEntry {
            entry_key,
            outcome_by_one_kernel,
            outcome_by_the_independently_written_kernel,
        }
    }

    /// CPU 参照判出的结果固定是 900 加位置，测试认得出是不是它判的。
    fn cpu_reference_outcome(entry_position: usize) -> u32 {
        900 + u32::try_from(entry_position).expect("测试的批不大")
    }

    #[test]
    fn identical_outcomes_are_confirmed() {
        let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
        let batch = counts.reconcile_the_two_kernel_outcomes(
            vec![entry(1, 7, 7), entry(2, 0, 0)],
            |_entry_position| panic!("两份核相同时不交 CPU 判"),
        );
        assert_eq!(
            batch,
            ReconciledBatch {
                decisions: vec![
                    EntryDecision::ConfirmedByBothKernels(7),
                    EntryDecision::ConfirmedByBothKernels(0),
                ],
                confirmed_entries: 2,
                uncertain_entries: 0,
                entries_judged_by_the_cpu_reference: 0,
            }
        );
    }

    #[test]
    fn different_outcomes_are_uncertain_for_one_round_without_asking_the_cpu() {
        let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
        let batch = counts.reconcile_the_two_kernel_outcomes(
            vec![entry(1, 7, 8), entry(2, 5, 5)],
            |_entry_position| panic!("只不确定一轮时不交 CPU 判"),
        );
        assert_eq!(batch.decisions[0], EntryDecision::UncertainThisRound);
        assert_eq!(
            (
                batch.confirmed_entries,
                batch.uncertain_entries,
                batch.entries_judged_by_the_cpu_reference
            ),
            (1, 1, 0)
        );
        assert_eq!(counts.consecutive_uncertain_rounds_of(&1), 1, "记下一轮");
    }

    #[test]
    fn two_consecutive_uncertain_rounds_hand_the_entry_to_the_cpu_reference() {
        let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
        counts.reconcile_the_two_kernel_outcomes(vec![entry(1, 7, 8)], |_entry_position| {
            panic!("只不确定一轮时不交 CPU 判")
        });
        let mut cpu_judged_positions = Vec::new();
        let batch = counts.reconcile_the_two_kernel_outcomes(
            vec![entry(3, 4, 4), entry(1, 9, 2)],
            |entry_position| {
                cpu_judged_positions.push(entry_position);
                cpu_reference_outcome(entry_position)
            },
        );
        assert_eq!(
            cpu_judged_positions,
            vec![1],
            "只有连续两轮不确定的那一条交 CPU 判"
        );
        assert_eq!(
            batch.decisions,
            vec![
                EntryDecision::ConfirmedByBothKernels(4),
                EntryDecision::JudgedByTheCpuReference(cpu_reference_outcome(1)),
            ],
            "按 CPU 的判定处理"
        );
        assert_eq!(
            (
                batch.confirmed_entries,
                batch.uncertain_entries,
                batch.entries_judged_by_the_cpu_reference
            ),
            (1, 0, 1)
        );
        assert_eq!(
            counts.consecutive_uncertain_rounds_of(&1),
            0,
            "CPU 判过就清零"
        );
    }

    #[test]
    fn agreeing_again_after_a_disagreement_restarts_the_consecutive_count() {
        let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
        let never_cpu = |_entry_position: usize| -> u32 {
            panic!("没有连续两轮不确定，不交 CPU 判")
        };
        counts.reconcile_the_two_kernel_outcomes(vec![entry(1, 7, 8)], never_cpu);
        let agreeing = counts.reconcile_the_two_kernel_outcomes(vec![entry(1, 7, 7)], never_cpu);
        assert_eq!(
            agreeing.decisions,
            vec![EntryDecision::ConfirmedByBothKernels(7)]
        );
        assert_eq!(
            counts.consecutive_uncertain_rounds_of(&1),
            0,
            "相同了就清零"
        );
        let disagreeing_again =
            counts.reconcile_the_two_kernel_outcomes(vec![entry(1, 7, 8)], never_cpu);
        assert_eq!(
            disagreeing_again.decisions,
            vec![EntryDecision::UncertainThisRound],
            "不同、相同、再不同：只算连续一轮，不交 CPU"
        );
    }

    #[test]
    fn an_entry_absent_from_a_batch_keeps_its_count() {
        let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
        let never_cpu = |_entry_position: usize| -> u32 { panic!("这一轮不交 CPU 判") };
        counts.reconcile_the_two_kernel_outcomes(vec![entry(1, 7, 8)], never_cpu);
        counts.reconcile_the_two_kernel_outcomes(vec![entry(2, 3, 3)], never_cpu);
        assert_eq!(
            counts.consecutive_uncertain_rounds_of(&1),
            1,
            "没出现的那一轮不清零也不加"
        );
    }

    #[test]
    fn dropping_the_counts_forgets_every_uncertain_round() {
        let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
        counts.reconcile_the_two_kernel_outcomes(vec![entry(1, 7, 8)], |_entry_position| {
            panic!("只不确定一轮时不交 CPU 判")
        });
        drop(counts);
        let mut counts_after_a_remount = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
        let batch = counts_after_a_remount
            .reconcile_the_two_kernel_outcomes(vec![entry(1, 7, 8)], |_entry_position| {
                panic!("重挂之后从零数起")
            });
        assert_eq!(batch.decisions, vec![EntryDecision::UncertainThisRound]);
    }

    #[test]
    #[should_panic(expected = "出现了两次")]
    fn the_same_entry_twice_in_one_batch_panics() {
        let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
        counts.reconcile_the_two_kernel_outcomes(
            vec![entry(1, 7, 7), entry(1, 7, 7)],
            |_entry_position| 0,
        );
    }
}
