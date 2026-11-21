//! 跑一段历史并读出三样读法的判定，几段历史分给几个线程跑。
use super::*;

/// 执行器的结局写成一个不带空格的词：跑完、已知红第几条、新发现（第几步、什么签名）。
fn ending_label(ending: &HistoryEnding) -> String {
    match ending {
        HistoryEnding::Completed => "completed".to_string(),
        HistoryEnding::KnownRed { form, observation } => {
            format!("known_red:{form}:{:?}", observation.position)
        }
        HistoryEnding::NewFinding {
            signature,
            observation,
        } => {
            format!("new_finding:{:?}:{signature:?}", observation.position)
        }
    }
    .chars()
    .map(|character| {
        if character.is_whitespace() {
            '_'
        } else {
            character
        }
    })
    .collect()
}

/// 登记第五节「每一步怎么跑、怎么看」：4 GiB 的盘、判空间准入、执行器自己的 checker 不跑（装置在观察者里自己调、判红不停）。
pub(super) const EXECUTION: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 登记第八节 G-small：同上，只把盘换成单元区 384 槽的小盘。
pub(super) const EXECUTION_ON_SMALL_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

fn evaluate_history(item: &HistoryItem, breakage: Breakage) -> HistoryEvaluation {
    let started = Instant::now();
    let mut account = HistoryAccount::new(breakage);
    let execution = item.family.execution();
    let run = execute_history_with(
        &item.history,
        execution,
        &SharedStream::new(),
        &mut |observation| {
            account.observe(observation);
        },
    );
    HistoryEvaluation {
        family: item.family,
        index: item.index,
        label: item.label.clone(),
        operations: item.history.operations.clone(),
        ending: ending_label(&run.ending),
        account,
        elapsed: started.elapsed(),
    }
}

impl HistoryEvaluation {
    fn same_timeline_reissues(&self) -> u64 {
        u64::try_from(
            self.account
                .events
                .iter()
                .filter(|event| event.same_timeline)
                .count(),
        )
        .expect("事件数装得进 u64")
    }

    fn reissues_across_abandoned_timelines(&self) -> u64 {
        u64::try_from(
            self.account
                .events
                .iter()
                .filter(|event| !event.same_timeline)
                .count(),
        )
        .expect("事件数装得进 u64")
    }

    fn same_timeline_reissues_with_candidacy(
        &self,
        class: OlderBirthStillInTheCandidateSet,
    ) -> u64 {
        u64::try_from(
            self.account
                .events
                .iter()
                .filter(|event| event.same_timeline && event.older_birth_candidacy == class)
                .count(),
        )
        .expect("事件数装得进 u64")
    }

    /// 结果行（一段历史一行，末尾 `line_end=ok`；D 族不带挂钟，它的产物要逐字节复跑）。
    pub(super) fn history_line(&self, arm: Arm) -> String {
        let counts = &self.account.counts;
        let operations: Vec<String> = self.operations.iter().map(operation_code).collect();
        let timing = match self.family {
            Family::Directed => String::new(),
            Family::DirectedExhaustive
            | Family::RandomWithRollbacks
            | Family::ExistingRandom(_)
            | Family::Geometry(_) => {
                format!(" elapsed_ns={}", self.elapsed.as_nanos())
            }
        };
        format!(
            "E7RESULT name=history family={} arm={} index={} label={} ending={} steps_observed={} issuances={} issuances_at_operations={} \
q1={} q3b={} q9_still={} q9_gone={} q2_one_birth_violated={} {} {} \
q3_steps={} q3_peak={} q3_final={} watermark_peak={} watermark_final={} no_watermark_steps={} \
q4={} q5={} q6={} q6_judged={} q7={} q10_issuances_at_operations={} q11={} \
rollbacks_applied={} rollback_targets_not_in_previous_ring={} creations_applied={} mounts_or_recoveries_applied={} \
s1_root_observations={} s1_unreadable={} s1_rebuilds={} s1_identity_conflicts={} s3_disagreements={} s3_table_unreadable_steps={} \
f3_still_but_holds={} f3_still_but_partial={} f3_violated_without_two_births={} system_configuration_unchosen_steps={} \
first_violation={} first_s3_disagreement={} first_q11={} operations=[{}]{} line_end=ok",
            self.family.label(),
            arm.label(),
            self.index,
            self.label,
            self.ending,
            counts.steps_observed,
            counts.issuances,
            counts.issuances_at_operations,
            self.same_timeline_reissues(),
            self.reissues_across_abandoned_timelines(),
            self.same_timeline_reissues_with_candidacy(OlderBirthStillInTheCandidateSet::Still),
            self.same_timeline_reissues_with_candidacy(OlderBirthStillInTheCandidateSet::Gone),
            u64::from(counts.one_birth_per_number.violated > 0),
            counts.one_birth_per_number.render("i9_29"),
            counts.watermark_above_both_highest_numbers.render("i9_6"),
            counts.watermark_regression_steps,
            counts.watermark_regression_peak,
            counts.watermark_regression_final,
            counts.watermark_peak,
            counts.watermark_final,
            counts.steps_whose_current_version_has_no_watermark,
            counts.rollbacks_across_issued_numbers,
            counts.windows_with_a_creation_after_such_a_rollback,
            counts.rollbacks_whose_ring_maximum_is_below_the_current,
            counts.rollbacks_judged_against_the_ring_maximum,
            counts.mounts_landing_on_a_lower_watermark,
            counts.issuances_at_operations,
            counts.creations_not_issuing_the_previous_watermark,
            counts.rollbacks_applied,
            counts.rollback_targets_not_in_the_previous_ring,
            counts.creations_applied,
            counts.mounts_or_recoveries_applied,
            counts.root_observations,
            counts.unreadable_root_observations,
            counts.version_rebuilds,
            counts.same_identity_with_different_facts,
            counts.abandonment_path_disagreements,
            counts.instance_table_unreadable_steps,
            counts.still_candidate_but_one_birth_holds,
            counts.still_candidate_but_one_birth_partially_judged,
            counts.one_birth_violated_without_two_births_in_the_ledger,
            counts.system_configuration_unchosen_steps,
            self.account.first_violation.as_deref().unwrap_or("none"),
            self.account.first_abandonment_path_disagreement.as_deref().unwrap_or("none"),
            self.account.first_creation_not_issuing_the_previous_watermark.as_deref().unwrap_or("none"),
            operations.join(","),
            timing,
        )
    }

    /// 事件行：每个 Q1 / Q3b 事件一行，排在它那段历史的结果行前面。
    pub(super) fn event_lines(&self, arm: Arm) -> Vec<String> {
        self.account
            .events
            .iter()
            .map(|event| {
                let older: Vec<String> = event.older_births.iter().map(u64::to_string).collect();
                format!(
                    "E7RESULT name=event family={} arm={} index={} kind={} position={} inode={} new_birth={} older_births=[{}] q9={:?} i9_29_at_step={}",
                    self.family.label(),
                    arm.label(),
                    self.index,
                    if event.same_timeline { "q1_same_timeline" } else { "q3b_across_abandoned" },
                    event.position,
                    event.inode,
                    event.new_birth,
                    older.join(","),
                    event.older_birth_candidacy,
                    event.one_birth_per_number_at_this_step.label(),
                )
            })
            .collect()
    }
}

/// 按区间切片并行跑（登记第五节「跑法」）：工作线程按序号递增领活，到点（`deadline`）就不再领；领了的跑完。
/// 领到的序号恒是一段前缀，合并按序号次序交给 `emit`，与单线程逐字相同（`one_thread_and_three_threads_give_the_same_lines` 对拍）。
pub(super) fn evaluate_items(
    items: &[HistoryItem],
    worker_threads: usize,
    deadline: Option<Instant>,
    breakage: Breakage,
    mut emit: impl FnMut(HistoryEvaluation),
) {
    assert!(worker_threads >= 1, "至少一个工作线程");
    let next_position = AtomicUsize::new(0);
    let (sender, receiver) = mpsc::channel::<(usize, HistoryEvaluation)>();
    std::thread::scope(|scope| {
        for _ in 0..worker_threads {
            let sender = sender.clone();
            let next_position = &next_position;
            scope.spawn(move || loop {
                if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                    break;
                }
                let position = next_position.fetch_add(1, Ordering::SeqCst);
                let Some(item) = items.get(position) else {
                    break;
                };
                let evaluation = evaluate_history(item, breakage);
                if sender.send((position, evaluation)).is_err() {
                    break;
                }
            });
        }
        drop(sender);
        let mut pending: BTreeMap<usize, HistoryEvaluation> = BTreeMap::new();
        let mut next_to_emit = 0_usize;
        for (position, evaluation) in receiver {
            pending.insert(position, evaluation);
            while let Some(evaluation_in_order) = pending.remove(&next_to_emit) {
                emit(evaluation_in_order);
                next_to_emit += 1;
            }
        }
        assert!(
            pending.is_empty(),
            "领到的序号应是一段前缀，合并之后不该剩下：剩 {:?}（某个工作线程 panic 了？）",
            pending.keys().collect::<Vec<_>>()
        );
    });
}

#[cfg(test)]
mod tests;
