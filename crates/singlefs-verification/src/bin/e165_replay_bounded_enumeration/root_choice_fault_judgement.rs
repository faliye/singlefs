//! 判一个最后一次挂载走完的历史：5 种被判时故障各判一次（登记 5.3 第 11 条）。

use super::*;

// ── 判一个最后一次挂载走完的历史：5 种被判时故障各判一次（登记 5.3 第 11 条）

pub(super) struct FaultJudgement {
    pub(super) domain: FaultDomain,
    /// 这个格的 M（M = 2 的格按挂载 2 开头那项故障另报）。
    pub(super) mounts: usize,
    pub(super) system_configuration: SystemConfiguration,
    pub(super) in_flight_limit: usize,
    /// 关掉时不调三步（单测里为省时；产物里恒开）。
    pub(super) run_the_three_steps: bool,
}

/// 一段历史上与被判时故障无关的真相：每个被判态共用。
struct FaultHistoryFacts {
    is_inside_the_single_fault_promise: bool,
    last_confirmed: Option<RootVersion>,
    parent_of_the_last_confirmed: Option<RootVersion>,
    /// 根环真相里不是最后一次已确认那一版、也不是它后代的根里键最大的那一条（KF4 的落点）。
    kf4_landing: Option<RootVersion>,
    has_same_txg_different_instance_roots: bool,
    has_higher_instance_lower_txg_roots: bool,
    has_a_torn_system_slot: bool,
}

fn facts_of_a_fault_history(world: &FaultWorld, ring_slots: u64) -> FaultHistoryFacts {
    let versions_in_the_true_ring: Vec<RootVersion> = world
        .root_slots
        .iter()
        .flatten()
        .map(|material| material.version)
        .collect();
    let last_confirmed = world.confirmed_user_publishes.iter().copied().max();
    if let Some(last) = last_confirmed {
        for confirmed in &world.confirmed_user_publishes {
            assert!(
                world.is_the_version_or_a_descendant_of(last, *confirmed),
                "停机条款 TH9：已确认的几次发布不在一条父链上（{}、{}）",
                last.describe(),
                confirmed.describe()
            );
        }
    }
    let largest_counter = world
        .landed_records
        .iter()
        .map(|landed| landed.counter)
        .max()
        .unwrap_or(0);
    assert!(
        largest_counter <= ring_slots,
        "停机条款 TH9：界内最大计数器 {largest_counter} 超过环槽数 {ring_slots}，journal 环回绕了"
    );
    let mut has_same_txg_different_instance_roots = false;
    let mut has_higher_instance_lower_txg_roots = false;
    for earlier in &versions_in_the_true_ring {
        for later in &versions_in_the_true_ring {
            has_same_txg_different_instance_roots |= earlier.checkpoint_txg == later.checkpoint_txg
                && earlier.instance != later.instance;
            has_higher_instance_lower_txg_roots |=
                earlier.instance > later.instance && earlier.checkpoint_txg < later.checkpoint_txg;
        }
    }
    FaultHistoryFacts {
        is_inside_the_single_fault_promise: world
            .mount_log
            .iter()
            .all(|entry| entry.fault.is_inside_the_single_fault_promise()),
        last_confirmed,
        parent_of_the_last_confirmed: last_confirmed.and_then(|last| world.parent_of(last)),
        kf4_landing: last_confirmed.and_then(|last| {
            versions_in_the_true_ring
                .iter()
                .copied()
                .filter(|version| !world.is_the_version_or_a_descendant_of(*version, last))
                .max()
        }),
        has_same_txg_different_instance_roots,
        has_higher_instance_lower_txg_roots,
        has_a_torn_system_slot: world
            .system_slots
            .iter()
            .flatten()
            .any(|content| *content == SystemSlotContent::Torn),
    }
}

impl FaultJudgement {
    pub(super) fn new(domain: FaultDomain, mounts: usize, run_the_three_steps: bool) -> Self {
        Self {
            domain,
            mounts,
            system_configuration: system_configuration_for_the_root_choice(domain.geometry()),
            in_flight_limit: domain.geometry().ring().local_in_flight_limit(),
            run_the_three_steps,
        }
    }

    /// 一段历史：先记与被判时故障无关的计数，再对 5 种被判时故障各判一个状态。
    pub(super) fn judge_a_history(
        &self,
        world: &FaultWorld,
        is_the_designated_history: bool,
        tally: &mut FaultTally,
    ) {
        let history = facts_of_a_fault_history(world, self.domain.geometry().ring().ring_slots);
        tally.histories += 1;
        tally.refused_mounts += world.refused_mounts;
        for (count, world_count) in tally
            .refusals_by_reason
            .iter_mut()
            .zip(&world.refusals_by_reason)
        {
            *count += world_count;
        }
        tally.reread_mounts += world.reread_mounts;
        tally.largest_counter = tally.largest_counter.max(
            world
                .landed_records
                .iter()
                .map(|landed| landed.counter)
                .max()
                .unwrap_or(0),
        );
        tally.confirmed_publishes_maximum = tally
            .confirmed_publishes_maximum
            .max(world.confirmed_user_publishes.len());
        if !history.is_inside_the_single_fault_promise {
            tally.histories_with_an_uncovered_fault += 1;
        }
        if self.mounts == 2 {
            assert_eq!(world.mount_log.len(), 2, "M = 2 的历史恰好开始过两次挂载");
            let count = &mut tally.mount_two[world.mount_log[1].fault.menu_index];
            count.histories += 1;
            count.refused += world.refused_mounts;
            count.reread += world.reread_mounts;
        }
        let truth = fault_landing_of_a_view(world, FaultLocations::NOWHERE);
        let disk = RootChoiceDisk::of_the_landing(self.domain.geometry(), &truth);
        for judged_fault in JudgedFault::ALL {
            let is_the_designated_state =
                is_the_designated_history && *judged_fault == JudgedFault::Nothing;
            self.judge_a_state(
                world,
                &history,
                &disk,
                (*judged_fault, is_the_designated_state),
                tally,
            );
        }
    }
}

impl FaultJudgement {
    /// 一个被判态：读视图 = 真相的盘减去被判时故障的落点。先记与实现无关的计数与分类，再算本地择根序与 KF2、KF4 的假结局读数，最后调三步判四条判据。
    fn judge_a_state(
        &self,
        world: &FaultWorld,
        history: &FaultHistoryFacts,
        disk: &RootChoiceDisk<'_>,
        (judged_fault, is_the_designated_state): (JudgedFault, bool),
        tally: &mut FaultTally,
    ) {
        let locations = judged_fault.locations(world);
        let view = fault_landing_of_a_view(world, locations);
        let describe_state = || {
            format!(
                "judged_fault={};history_inside_the_promise={};{}",
                judged_fault.label(),
                history.is_inside_the_single_fault_promise,
                describe_a_fault_history(world)
            )
        };
        tally.judged_states += 1;
        tally.states_with_a_torn_system_slot += u64::from(history.has_a_torn_system_slot);
        tally.states_with_same_txg_different_instance_roots +=
            u64::from(history.has_same_txg_different_instance_roots);
        tally.states_with_higher_instance_lower_txg_roots +=
            u64::from(history.has_higher_instance_lower_txg_roots);
        let newest_root = root_choice_newest_root(&view);
        match newest_root {
            None => tally.states_without_a_readable_root += 1,
            Some(newest) => {
                let above_newest_root = view
                    .records
                    .iter()
                    .filter(|record| {
                        record.material.version.instance == newest.version.instance
                            && record.material.version.checkpoint_txg
                                > newest.version.checkpoint_txg
                    })
                    .count();
                tally.above_newest_root_maximum =
                    tally.above_newest_root_maximum.max(above_newest_root);
            }
        }
        let ideal = newest_root.map(|newest| {
            root_choice_ideal_recovery_from(newest, &view.records, self.in_flight_limit)
        });
        let facts = FaultStateFacts {
            last_confirmed: history.last_confirmed,
            newest_readable: newest_root.map(|newest| newest.version),
            ideal_fields: ideal.as_ref().map(|recovery| recovery.landed_fields),
        };
        let cell_index =
            fault_cell_index_of(judged_fault, history.is_inside_the_single_fault_promise);
        tally.cells[cell_index].states += 1;
        let is_cut_by_the_limit = self.classify_the_ideal_recovery(
            world,
            history,
            &view,
            newest_root,
            &ideal,
            judged_fault,
            cell_index,
            tally,
        );
        if is_cut_by_the_limit {
            tally.cells[cell_index].cut_states += 1;
        }
        let is_inside_the_promise = history.is_inside_the_single_fault_promise
            && judged_fault.is_inside_the_single_fault_promise()
            && !is_cut_by_the_limit;
        if is_inside_the_promise {
            tally.promised_states += 1;
        }
        self.read_the_local_rules_and_the_fake_outcomes(
            world,
            history,
            &view,
            &facts,
            judged_fault,
            tally,
        );
        if !self.run_the_three_steps {
            return;
        }
        if is_the_designated_state {
            let mut scratch = FaultTally::new();
            self.count_the_outcome(
                world,
                &facts,
                &RootChoiceOutcome::Panicked {
                    step: "designated_state",
                },
                (cell_index, false, false),
                &describe_state,
                &mut scratch,
            );
            tally.kj0_control_violations += scratch.violations_of_a_criterion(0);
        }
        let faulted_disk = FaultedRootChoiceDisk {
            inner: disk,
            locations,
            slots_per_region: world.slots_per_region(),
        };
        check_the_fault_slot_readings_against_the_truth(
            &faulted_disk,
            &self.system_configuration,
            world,
            tally,
            &describe_state,
        );
        let outcome =
            outcome_of_the_three_steps_on_a_faulted_disk(&faulted_disk, &self.system_configuration);
        let violated = self.count_the_outcome(
            world,
            &facts,
            &outcome,
            (cell_index, is_cut_by_the_limit, newest_root.is_some()),
            &describe_state,
            tally,
        );
        if is_inside_the_promise {
            for (position, criterion_index) in
                [KJ1_INDEX, DESCENDS_FROM_THE_LAST_CONFIRMED_PUBLISH_INDEX]
                    .into_iter()
                    .enumerate()
            {
                if violated[criterion_index] {
                    tally.promised_violations[position] += 1;
                    if tally.promised_witnesses[position].is_none() {
                        tally.promised_witnesses[position] = Some(describe_state());
                    }
                }
            }
        }
    }
}

impl FaultJudgement {
    /// 有已确认的被判态：数一笔，再分类条款的理想恢复（读视图里版本最大的根起）丢没丢最后一次已确认的发布：
    /// 去掉在飞上限就不丢 ⇒ 上限截掉；否则被判时与历史都罩着 ⇒ 罩着的其余；否则不罩。交回「是上限截掉」。
    #[allow(
        clippy::too_many_arguments,
        reason = "一个被判态的分类要同时看历史、读视图、理想恢复与两个标签，拆开只是把同一组量再传一遍"
    )]
    fn classify_the_ideal_recovery(
        &self,
        world: &FaultWorld,
        history: &FaultHistoryFacts,
        view: &RootChoiceLanding<'_>,
        newest_root: Option<&RootChoicePublishMaterial>,
        ideal: &Option<RootChoiceIdealRecovery>,
        judged_fault: JudgedFault,
        cell_index: usize,
        tally: &mut FaultTally,
    ) -> bool {
        let Some(last) = history.last_confirmed else {
            return false;
        };
        tally.confirmed_states += 1;
        tally.cells[cell_index].confirmed_states += 1;
        let (Some(newest), Some(ideal_recovery)) = (newest_root, ideal) else {
            return false;
        };
        let ideal_version = ideal_recovery.landed_version;
        let ideal_is_earlier = ideal_version < last;
        let ideal_is_off_the_chain = !world.is_the_version_or_a_descendant_of(ideal_version, last);
        tally.cells[cell_index].ideal_violations[0] += u64::from(ideal_is_earlier);
        tally.cells[cell_index].ideal_violations[1] += u64::from(ideal_is_off_the_chain);
        if !(ideal_is_earlier || ideal_is_off_the_chain) {
            return false;
        }
        let without_the_limit =
            root_choice_ideal_recovery_from(newest, &view.records, FAULT_IN_FLIGHT_LIMIT_REMOVED)
                .landed_version;
        let is_cut_by_the_limit = without_the_limit >= last
            && world.is_the_version_or_a_descendant_of(without_the_limit, last);
        if is_cut_by_the_limit {
            tally.ideal_loss_by_limit += 1;
        } else if history.is_inside_the_single_fault_promise
            && judged_fault.is_inside_the_single_fault_promise()
        {
            tally.ideal_loss_covered_other += 1;
        } else {
            tally.ideal_loss_uncovered += 1;
        }
        is_cut_by_the_limit
    }

    /// 本地择根序加理想重放的七种读数（登记第七节「本地择根序」那张表；下标 6 是 KF1），KF2（落在最后一次已确认那一版的父上）与 KF4（落在根环真相里
    /// 不在已确认那条链上的根中键最大的那条）的假结局读数。全部走 [`fault_criteria_of_a_landing`]，与真实现的结局读同一段判据。
    fn read_the_local_rules_and_the_fake_outcomes(
        &self,
        world: &FaultWorld,
        history: &FaultHistoryFacts,
        view: &RootChoiceLanding<'_>,
        facts: &FaultStateFacts,
        judged_fault: JudgedFault,
        tally: &mut FaultTally,
    ) {
        for (rule_index, rule) in FaultLocalRule::ALL.iter().enumerate() {
            let Some(chosen) = rule.chooses_among(&view.roots_in_ring_order) else {
                continue;
            };
            let landed_version =
                root_choice_ideal_recovery_from(chosen, &view.records, self.in_flight_limit)
                    .landed_version;
            let reading = fault_criteria_of_a_landing(world, facts, landed_version, None);
            for (position, verdict) in [reading.kj1, reading.kj1t, reading.kj2]
                .into_iter()
                .enumerate()
            {
                tally.local_rules[rule_index][position] += u64::from(verdict == Some(true));
            }
            tally.local_kj1t_by_judged_fault[rule_index][judged_fault.index()] +=
                u64::from(reading.kj1t == Some(true));
        }
        if history.last_confirmed.is_none() {
            return;
        }
        if let Some(parent) = history.parent_of_the_last_confirmed {
            let reading = fault_criteria_of_a_landing(
                world,
                facts,
                parent,
                world.fields_of_a_version(parent),
            );
            tally.kf2[0] += u64::from(reading.kj1 == Some(true));
            tally.kf2[1] += u64::from(reading.kj1t == Some(true));
            tally.kf2[2] += u64::from(reading.kj2 == Some(true));
            // 读视图里一条根都没有时没有理想恢复可比：假结局落在一版上，照 KJ3 的字面算违例。
            tally.kf2[3] += u64::from(reading.kj3.unwrap_or(facts.ideal_fields.is_none()));
        }
        match history.kf4_landing {
            Some(landing) => {
                let reading = fault_criteria_of_a_landing(world, facts, landing, None);
                tally.kf4[0] += u64::from(reading.kj1 == Some(true));
                tally.kf4[1] += u64::from(reading.kj1t == Some(true));
            }
            None => tally.kf4_states_without_a_root_off_the_confirmed_chain += 1,
        }
    }
}

impl FaultJudgement {
    /// 一个被判态的结局落进判据（各报各的，不合成一个真假）：真实现的结局与阳性对照的假结局走这一段。
    /// `(cell_index, is_cut_by_the_limit, a_root_is_readable)`：这一态落在哪个格、是不是上限截掉、读视图里有没有读得出的根。
    /// 交回每条判据这一态是不是违例（承诺内那一行用）。
    fn count_the_outcome(
        &self,
        world: &FaultWorld,
        facts: &FaultStateFacts,
        outcome: &RootChoiceOutcome,
        (cell_index, is_cut_by_the_limit, root_is_readable): (usize, bool, bool),
        describe_state: &dyn Fn() -> String,
        tally: &mut FaultTally,
    ) -> [bool; FAULT_CRITERION_COUNT] {
        let mut violated = [false; FAULT_CRITERION_COUNT];
        tally.cells[cell_index].judged[0] += 1;
        match outcome {
            RootChoiceOutcome::Panicked { step } => {
                tally.panicked_states += 1;
                self.add_a_kj0_violation(cell_index, is_cut_by_the_limit, tally, &|| {
                    format!("panicked_in={step};{}", describe_state())
                });
                violated[0] = true;
            }
            RootChoiceOutcome::ChoseNothing => {
                if root_is_readable {
                    tally.chose_nothing_with_a_readable_root += 1;
                    self.add_a_kj0_violation(cell_index, is_cut_by_the_limit, tally, &|| {
                        format!(
                            "choose_root_returned_none_with_a_readable_root;{}",
                            describe_state()
                        )
                    });
                    violated[0] = true;
                } else {
                    tally.chose_nothing_without_a_readable_root += 1;
                }
            }
            RootChoiceOutcome::Landed {
                chosen,
                landed_version,
                landed_fields,
                replay_failed,
            } => {
                tally.replay_failed_states += u64::from(*replay_failed);
                tally.replay_applied_states += u64::from(landed_version != chosen);
                let reading = fault_criteria_of_a_landing(
                    world,
                    facts,
                    *landed_version,
                    Some(*landed_fields),
                );
                let counted = count_a_criteria_reading_into_a_cell(
                    reading,
                    is_cut_by_the_limit,
                    &mut tally.cells[cell_index],
                );
                for criterion in FaultCriterion::ALL {
                    let criterion_index = criterion.index();
                    if criterion_index == 0 || !counted[criterion_index] {
                        continue;
                    }
                    violated[criterion_index] = true;
                    let witness =
                        &mut tally.witnesses[cell_index * FAULT_CRITERION_COUNT + criterion_index];
                    if witness.is_none() {
                        *witness = Some(format!(
                            "{};chosen={};landed={};last_confirmed={};newest_readable={};{}",
                            criterion.label(),
                            chosen.describe(),
                            landed_version.describe(),
                            facts
                                .last_confirmed
                                .map_or_else(|| "none".to_string(), RootVersion::describe),
                            facts
                                .newest_readable
                                .map_or_else(|| "none".to_string(), RootVersion::describe),
                            describe_state()
                        ));
                    }
                }
                if reading.kj1.is_some() && reading.kj1 != reading.kj1t {
                    tally.kj1_and_kj1t_differing_states += 1;
                    if tally.kj1_and_kj1t_differing_witness.is_none() {
                        tally.kj1_and_kj1t_differing_witness = Some(format!(
                            "kj1_violated={:?};kj1t_violated={:?};landed={};{}",
                            reading.kj1,
                            reading.kj1t,
                            landed_version.describe(),
                            describe_state()
                        ));
                    }
                }
            }
        }
        violated
    }

    fn add_a_kj0_violation(
        &self,
        cell_index: usize,
        is_cut_by_the_limit: bool,
        tally: &mut FaultTally,
        describe_witness: &dyn Fn() -> String,
    ) {
        tally.add_cell_violation(cell_index, 0, describe_witness);
        if is_cut_by_the_limit {
            tally.cells[cell_index].cut_violations[0] += 1;
        }
    }
}
