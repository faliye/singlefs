//! 一份历史的整趟、回退过缺口、写路径的整趟与成对统计。

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingChange {
    RewriteTheSelectedObject { data_units: u64 },
    UnlinkTheSmallestLiveObject,
}

/// 「按升序删」带 F10 的覆盖写规则：第 j 次 unlink 之前，j 是 10 的倍数时覆盖写、是 25 的倍数时顺序写成 5 个单元（读法 6：删号最小的活对象）。
#[derive(Clone, Debug, Default)]
struct AscendingDeletionStream {
    unlinks_started: u64,
    pending: std::collections::VecDeque<PendingChange>,
}

impl AscendingDeletionStream {
    fn next_change(&mut self, simulation: &Simulation) -> Option<UserChange> {
        loop {
            if self.pending.is_empty() {
                self.unlinks_started += 1;
                let unlink_number = self.unlinks_started;
                if unlink_number.is_multiple_of(MANY_OBJECTS_OVERWRITE_EVERY_UNLINKS) {
                    self.pending.push_back(PendingChange::RewriteTheSelectedObject { data_units: 1 });
                }
                if unlink_number.is_multiple_of(MANY_OBJECTS_SEQUENTIAL_WRITE_EVERY_UNLINKS) {
                    self.pending.push_back(PendingChange::RewriteTheSelectedObject { data_units: MANY_OBJECTS_SEQUENTIAL_WRITE_DATA_UNITS });
                }
                self.pending.push_back(PendingChange::UnlinkTheSmallestLiveObject);
            }
            match self.pending.pop_front()? {
                PendingChange::RewriteTheSelectedObject { data_units } => {
                    if let Some(object) = simulation.rewrite_target() {
                        return Some(UserChange::WriteFile { object, data_units });
                    }
                }
                PendingChange::UnlinkTheSmallestLiveObject => {
                    return simulation
                        .version
                        .live_inodes()
                        .into_iter()
                        .find(|inode| *inode > FIRST_FILE_INODE)
                        .map(|object| UserChange::Unlink { object });
                }
            }
        }
    }
}

/// F11 一段的结局（登记 5.2、5.5 PC-B2）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RollbackOutcome {
    pub(super) event_found: bool,
    pub(super) distance_in_publishes: u64,
    pub(super) too_far: bool,
    pub(super) rollback_publish_sequence: u64,
    pub(super) retired_in_the_gap_and_present_in_the_old_version: u64,
    pub(super) retirement_records_after_the_rollback: u64,
}

fn run_rollback_over_a_gap(simulation: &mut Simulation, event: GapEvent) -> RollbackOutcome {
    let mut outcome = RollbackOutcome::default();
    build_and_write_many_objects(simulation, ROLLBACK_OVER_A_GAP_INODE_COUNT);
    simulation.gap_watch = Some(event);
    let mut stream = AscendingDeletionStream::default();
    while simulation.gap_start.is_none() && !simulation.has_stopped() {
        let Some(change) = stream.next_change(simulation) else {
            return outcome;
        };
        simulation.apply(change);
    }
    let Some(gap_start) = simulation.gap_start.clone() else {
        return outcome;
    };
    outcome.event_found = true;
    for _ in 0..ROLLBACK_OVER_A_GAP_CHANGES_BEFORE_ROLLBACK {
        if let Some(change) = stream.next_change(simulation) {
            simulation.apply(change);
        }
    }
    outcome.distance_in_publishes = simulation.version.txg - gap_start.version_before.txg;
    outcome.too_far = outcome.distance_in_publishes > ROLLBACK_DISTANCE_LIMIT_PUBLISHES;
    let identities_in_the_old_version: HashSet<ContainerIdentity> =
        gap_start.version_before.leaves.iter().map(|leaf| leaf.identity).collect();
    let retired_in_the_gap: BTreeSet<ContainerIdentity> = simulation
        .publish_records
        .iter()
        .filter(|record| record.publish_sequence >= gap_start.event_publish_sequence)
        .flat_map(|record| record.retired_identities.iter().copied())
        .filter(|identity| identities_in_the_old_version.contains(identity))
        .collect();
    outcome.retired_in_the_gap_and_present_in_the_old_version = length_as_u64(retired_in_the_gap.len());
    simulation.roll_back_to(gap_start.version_before);
    outcome.rollback_publish_sequence = simulation.publish_sequence;
    for _ in 0..ROLLBACK_OVER_A_GAP_CHANGES_AFTER_ROLLBACK {
        if let Some(change) = stream.next_change(simulation) {
            simulation.apply(change);
        }
    }
    outcome.retirement_records_after_the_rollback = simulation
        .publish_records
        .iter()
        .filter(|record| record.publish_sequence > outcome.rollback_publish_sequence)
        .map(|record| record.written.retirement_records)
        .sum();
    outcome
}

/// 照历史驱动一段模拟；顺带判只在某个族上能判的断言（A1、A8）。F11 交回回退的结局。
pub(super) fn run_history(history: History, simulation: &mut Simulation) -> Option<RollbackOutcome> {
    match history {
        History::BuildAndDeleteEmptyInodes { inode_count, order } => {
            let mut created = 1;
            while created < inode_count {
                let count = NEW_INODES_PER_STEP.min(inode_count - created);
                simulation.apply(UserChange::NewInodes { count });
                created += count;
            }
            for object in deletion_order_of(inode_count, order) {
                simulation.apply(UserChange::Unlink { object });
            }
            let expected_publishes = 3 + (inode_count - 1).div_ceil(NEW_INODES_PER_STEP) + 2 * (inode_count - 1);
            simulation.assertions.judge("A1_history_publishes", simulation.publishes == expected_publishes);
            simulation.assertions.judge("A1_history_refused_steps", simulation.refused_steps == 0);
            if simulation.cell.merge_threshold == 0 && simulation.container_death == ContainerDeathWritePath::RetirementRecords {
                let expected_retirements = inode_count.div_ceil(simulation.cell.leaf_capacity) - 1;
                let retirements = simulation.total_written().retirement_records;
                simulation.assertions.judge("A8_retirements_equal_identity_disappearances", retirements == expected_retirements);
            }
        }
        History::LargeFileRewriteAndDelete { data_units, rewrites, overwrite_before_unlink, batch } => {
            for _ in 0..=rewrites {
                simulation.apply(UserChange::WriteFile { object: FIRST_FILE_INODE, data_units });
            }
            if overwrite_before_unlink {
                simulation.apply(UserChange::WriteFile { object: FIRST_FILE_INODE, data_units: 1 });
            }
            simulation.apply(UserChange::Unlink { object: FIRST_FILE_INODE });
            let data_units_at_the_intent = if overwrite_before_unlink { 1 } else { data_units };
            let expected_publishes =
                3 + (rewrites + 1) + u64::from(overwrite_before_unlink) + 1 + data_units_at_the_intent.div_ceil(batch).max(1);
            simulation.assertions.judge("A1_history_publishes", simulation.publishes == expected_publishes);
            simulation.assertions.judge("A1_history_refused_steps", simulation.refused_steps == 0);
            if simulation.cell.merge_threshold == 0 && simulation.container_death == ContainerDeathWritePath::RetirementRecords {
                let retirements = simulation.total_written().retirement_records;
                simulation.assertions.judge("A8_retirements_equal_identity_disappearances", retirements == 1);
            }
        }
        History::MixedRandom { seed } => {
            let mut random = SeededRandom::new(seed);
            for _ in 0..MIXED_RANDOM_STEPS {
                let draw = random.below(10);
                let change = if draw < 3 {
                    UserChange::NewInodes { count: 1 + random.below(20) }
                } else if draw < 5 {
                    UserChange::WriteFile { object: FIRST_FILE_INODE, data_units: 1 }
                } else if draw < 6 {
                    UserChange::WriteFile { object: FIRST_FILE_INODE, data_units: 1 + random.below(200) }
                } else {
                    let live = simulation.version.live_inodes();
                    if live.is_empty() {
                        simulation.refused_steps += 1;
                        continue;
                    }
                    UserChange::Unlink { object: live[u64_as_index(random.below(length_as_u64(live.len())))] }
                };
                simulation.apply(change);
            }
            let expected = E177_MIXED_RANDOM_PUBLISHES_AND_REFUSALS.iter().find(|(expected_seed, _, _)| *expected_seed == seed);
            let holds = expected.is_some_and(|(_, publishes, refused)| {
                simulation.publishes == *publishes && simulation.refused_steps == *refused
            });
            simulation.assertions.judge("A1_history_publishes", holds);
            simulation.assertions.judge("A1_history_refused_steps", holds);
        }
        History::ManyObjectsWithData { inode_count, order, .. } => {
            build_and_write_many_objects(simulation, inode_count);
            for (position, victim) in deletion_order_of(inode_count, order).into_iter().enumerate() {
                let unlink_number = length_as_u64(position) + 1;
                if unlink_number.is_multiple_of(MANY_OBJECTS_OVERWRITE_EVERY_UNLINKS) {
                    if let Some(object) = simulation.rewrite_target() {
                        simulation.apply(UserChange::WriteFile { object, data_units: 1 });
                    }
                }
                if unlink_number.is_multiple_of(MANY_OBJECTS_SEQUENTIAL_WRITE_EVERY_UNLINKS) {
                    if let Some(object) = simulation.rewrite_target() {
                        simulation.apply(UserChange::WriteFile { object, data_units: MANY_OBJECTS_SEQUENTIAL_WRITE_DATA_UNITS });
                    }
                }
                simulation.apply(UserChange::Unlink { object: victim });
            }
        }
        History::RollBackOverAGap { event } => return Some(run_rollback_over_a_gap(simulation, event)),
    }
    None
}

pub(super) fn simulate(history: History, cell: WritePathCell, scanning: bool) -> (Simulation, Option<RollbackOutcome>) {
    let mut simulation = Simulation::start(cell, history.batch(), scanning);
    let outcome = run_history(history, &mut simulation);
    (simulation, outcome)
}

// ---------------------------------------------------------------------------------------------
// 第一段（登记 5.8）：X4。H51 与 F10 在 N1 / N2 × 容量 × θ 下只跑写路径；5.6 全表；PC-N、PC-K；停机对拍 S1–S3、S6
// ---------------------------------------------------------------------------------------------

/// 一组观测的最小、最大、总数与个数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Extremes {
    pub(super) observations: u64,
    pub(super) smallest: Option<u64>,
    pub(super) largest: Option<u64>,
    total: u64,
}

impl Extremes {
    pub(super) fn observe(&mut self, value: u64) {
        self.observations += 1;
        self.smallest = Some(self.smallest.map_or(value, |smallest| smallest.min(value)));
        self.largest = Some(self.largest.map_or(value, |largest| largest.max(value)));
        self.total += value;
    }

    pub(super) fn absorb(&mut self, other: Extremes) {
        self.observations += other.observations;
        self.total += other.total;
        if let Some(value) = other.smallest {
            self.smallest = Some(self.smallest.map_or(value, |smallest| smallest.min(value)));
        }
        if let Some(value) = other.largest {
            self.largest = Some(self.largest.map_or(value, |largest| largest.max(value)));
        }
    }

    pub(super) fn fields(self, prefix: &str) -> String {
        let shown = |value: Option<u64>| value.map_or_else(|| "none".to_string(), |known| known.to_string());
        format!(
            "{prefix}_count={} {prefix}_smallest={} {prefix}_largest={} {prefix}_total={}",
            self.observations,
            shown(self.smallest),
            shown(self.largest),
            self.total
        )
    }
}

/// 一条写路径跑完留下的账（模拟本身丢掉，免得盘上的单元占着内存）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct WritePathRun {
    pub(super) publishes: u64,
    pub(super) refused_steps: u64,
    pub(super) borrows: u64,
    pub(super) absorbs: u64,
    pub(super) drops: u64,
    pub(super) moved_then_shortened: u64,
    pub(super) moved_then_unlinked: u64,
    pub(super) written: UnitsWritten,
    pub(super) self_check: SelfCheckTally,
    pub(super) does_not_fit_at: Option<u64>,
    pub(super) assertions: AssertionTally,
    pub(super) publish_records: Vec<PublishRecord>,
    pub(super) user_changes: Vec<UserChange>,
}

pub(super) fn run_write_path(history: History, cell: WritePathCell) -> WritePathRun {
    let (simulation, _) = simulate(history, cell, false);
    let (borrows, absorbs, drops) = simulation.count_merges();
    WritePathRun {
        publishes: simulation.publishes,
        refused_steps: simulation.refused_steps,
        borrows,
        absorbs,
        drops,
        moved_then_shortened: simulation.moved_then_shortened,
        moved_then_unlinked: simulation.moved_then_unlinked,
        written: simulation.total_written(),
        self_check: simulation.self_check,
        does_not_fit_at: simulation.does_not_fit_at,
        assertions: simulation.assertions.clone(),
        publish_records: simulation.publish_records,
        user_changes: simulation.user_changes,
    }
}

/// 同一段历史、同一个容量与 θ 下 N1 与 N2 两条写流的配对（登记第一节读法 7、Q-X4-1、Q-X4-2、A5）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct PairStatistics {
    pub(super) borrow_publishes: u64,
    pub(super) extra_code_three_per_borrow: Extremes,
    pub(super) extra_code_two_per_borrow: Extremes,
    pub(super) retirement_records_per_borrow_renew_identity: Extremes,
    pub(super) retirement_records_per_borrow_raise_separator: Extremes,
    pub(super) publishes_outside_borrows_with_a_difference: u64,
    pub(super) borrow_kinds_disagree: u64,
    pub(super) renew_identity_counted_units: u64,
    pub(super) raise_separator_counted_units: u64,
    pub(super) user_changes_identical: bool,
    pub(super) publish_counts_identical: bool,
    pub(super) streams_identical: bool,
}

impl PairStatistics {
    /// N2 多写占这段历史全部写单元的比例，按百万分之一计（读法 8）。
    pub(super) fn extra_share_in_parts_per_million(self) -> u64 {
        let extra = self.renew_identity_counted_units.saturating_sub(self.raise_separator_counted_units);
        if self.renew_identity_counted_units == 0 {
            return 0;
        }
        extra * 1_000_000 / self.renew_identity_counted_units
    }
}

pub(super) fn pair_statistics(raise_separator: &WritePathRun, renew_identity: &WritePathRun) -> PairStatistics {
    let mut pair = PairStatistics {
        borrow_publishes: 0,
        extra_code_three_per_borrow: Extremes::default(),
        extra_code_two_per_borrow: Extremes::default(),
        retirement_records_per_borrow_renew_identity: Extremes::default(),
        retirement_records_per_borrow_raise_separator: Extremes::default(),
        publishes_outside_borrows_with_a_difference: 0,
        borrow_kinds_disagree: 0,
        renew_identity_counted_units: 0,
        raise_separator_counted_units: 0,
        user_changes_identical: raise_separator.user_changes == renew_identity.user_changes,
        publish_counts_identical: raise_separator.publish_records.len() == renew_identity.publish_records.len(),
        streams_identical: true,
    };
    for (raised, renewed) in raise_separator.publish_records.iter().zip(&renew_identity.publish_records) {
        pair.renew_identity_counted_units += renewed.written.every_counted_unit();
        pair.raise_separator_counted_units += raised.written.every_counted_unit();
        if raised.written != renewed.written || raised.leaf_digest != renewed.leaf_digest {
            pair.streams_identical = false;
        }
        let renewed_borrowed = matches!(renewed.merge, MergeAction::BorrowedFromTheRightNeighbour { .. });
        let raised_borrowed = matches!(raised.merge, MergeAction::BorrowedFromTheRightNeighbour { .. });
        if renewed_borrowed != raised_borrowed || std::mem::discriminant(&renewed.merge) != std::mem::discriminant(&raised.merge) {
            pair.borrow_kinds_disagree += 1;
        }
        if renewed_borrowed {
            pair.borrow_publishes += 1;
            pair.extra_code_three_per_borrow
                .observe(difference_or_marker(renewed.written.code_three_counted(), raised.written.code_three_counted()));
            pair.extra_code_two_per_borrow
                .observe(difference_or_marker(renewed.written.code_two_counted(), raised.written.code_two_counted()));
            pair.retirement_records_per_borrow_renew_identity.observe(renewed.written.retirement_records);
            pair.retirement_records_per_borrow_raise_separator.observe(raised.written.retirement_records);
        } else if renewed.written.every_counted_unit() != raised.written.every_counted_unit() {
            pair.publishes_outside_borrows_with_a_difference += 1;
        }
    }
    pair
}

#[cfg(test)]
#[path = "history_runs_and_pair_statistics_tests.rs"]
mod tests;
