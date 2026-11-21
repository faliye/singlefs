//! 一格的累计与多线程跑。

use super::*;

// ───────────────────────────── 一格的累计与多线程跑 ─────────────────────────────

/// 一段上的计时累计（G3 的加权估计按段取平均）。
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct TimingSums {
    pub(super) states: u64,
    pub(super) state_nanoseconds: u128,
    pub(super) zone_nanoseconds: [u128; 6],
    pub(super) unit_events_replay_nanoseconds: u128,
    pub(super) walk_checker_nanoseconds: u128,
    pub(super) states_shorter_than_their_segments: u64,
    pub(super) states_replaying_longer_than_the_checker: u64,
}

impl TimingSums {
    pub(super) fn absorb_state(&mut self, timings: &StateTimings) {
        self.states += 1;
        self.state_nanoseconds += timings.state_nanoseconds;
        for (slot, zone) in TIMING_ZONES.iter().enumerate() {
            self.zone_nanoseconds[slot] += timings.zone_nanoseconds(*zone);
        }
        self.unit_events_replay_nanoseconds += timings.unit_events_replay_nanoseconds;
        self.walk_checker_nanoseconds += timings.walk_checker_nanoseconds;
        if timings.state_nanoseconds < timings.five_segments_nanoseconds() {
            self.states_shorter_than_their_segments += 1;
        }
        if timings.unit_positions_replay_nanoseconds > timings.pool_checker_nanoseconds {
            self.states_replaying_longer_than_the_checker += 1;
        }
    }

    pub(super) fn absorb(&mut self, other: &TimingSums) {
        self.states += other.states;
        self.state_nanoseconds += other.state_nanoseconds;
        for (mine, theirs) in self.zone_nanoseconds.iter_mut().zip(other.zone_nanoseconds) {
            *mine += theirs;
        }
        self.unit_events_replay_nanoseconds += other.unit_events_replay_nanoseconds;
        self.walk_checker_nanoseconds += other.walk_checker_nanoseconds;
        self.states_shorter_than_their_segments += other.states_shorter_than_their_segments;
        self.states_replaying_longer_than_the_checker +=
            other.states_replaying_longer_than_the_checker;
    }
}

/// 把 `from` 的 S4 那几项加进 `into`。
pub(super) fn add_tally(into: &mut CrashPointReplayTally, from: &CrashPointReplayTally) {
    into.states += from.states;
    into.violations += from.violations;
    into.ignored_violations += from.ignored_violations;
    into.root_persisted_states += from.root_persisted_states;
    into.no_file_states += from.no_file_states;
    into.file_read_states += from.file_read_states;
    into.failed_states += from.failed_states;
    into.journal_differing_states += from.journal_differing_states;
    into.record_root_without_record += from.record_root_without_record;
    into.record_claimed_state_missing_unit += from.record_claimed_state_missing_unit;
    for (invariant, count) in &from.checker_evaluated_states {
        *into.checker_evaluated_states.entry(invariant).or_insert(0) += count;
    }
    for (invariant, count) in &from.checker_violated_states {
        *into.checker_violated_states.entry(invariant).or_insert(0) += count;
    }
    for (invariant, count) in &from.checker_not_applicable_states {
        *into
            .checker_not_applicable_states
            .entry(invariant)
            .or_insert(0) += count;
    }
}

/// S4 比的那几项拼成一段文本（两边各拼一份逐字比）。
pub(super) fn tally_text(tally: &CrashPointReplayTally) -> String {
    format!(
        "states={} violations={} ignored_violations={} root_persisted={} no_file={} file_read={} failed={} journal_differing={} record_root_without_record={} record_claimed_state_missing_unit={} checker=[{}]",
        tally.states,
        tally.violations,
        tally.ignored_violations,
        tally.root_persisted_states,
        tally.no_file_states,
        tally.file_read_states,
        tally.failed_states,
        tally.journal_differing_states,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        tally.checker_counts_by_invariant()
    )
}

/// 一格（一条流上的一个取样域）跑完的全部累计。各项都是可交换的加法或取最小，与线程数、片的完成次序无关。
pub(super) struct StreamDomainTotals {
    pub(super) tally: CrashPointReplayTally,
    pub(super) tables: ContentTables,
    pub(super) timing_by_segment: BTreeMap<usize, TimingSums>,
    pub(super) arm_stores: Vec<ArmRecordStore>,
    pub(super) whole_unit_positions: u64,
    pub(super) whole_unit_events: u64,
    pub(super) header_scan_positions: u64,
    pub(super) header_scan_events: u64,
    pub(super) ring_record_positions: u64,
    pub(super) ring_record_events: u64,
    pub(super) states_without_whole_unit_reads: u64,
    pub(super) consult_report_differs: u64,
    pub(super) ignore_report_differs: u64,
    pub(super) verdicts_differ: u64,
    pub(super) walk_reads_outside_normal_reads: u64,
    pub(super) control: Vec<ControlStateData>,
}

impl StreamDomainTotals {
    pub(super) fn new() -> Self {
        Self {
            tally: CrashPointReplayTally::default(),
            tables: ContentTables::default(),
            timing_by_segment: BTreeMap::new(),
            arm_stores: ARMS
                .iter()
                .map(|_arm| ArmRecordStore::new(ARM_RECORDS_IN_MEMORY_BEFORE_SPILL))
                .collect(),
            whole_unit_positions: 0,
            whole_unit_events: 0,
            header_scan_positions: 0,
            header_scan_events: 0,
            ring_record_positions: 0,
            ring_record_events: 0,
            states_without_whole_unit_reads: 0,
            consult_report_differs: 0,
            ignore_report_differs: 0,
            verdicts_differ: 0,
            walk_reads_outside_normal_reads: 0,
            control: Vec::new(),
        }
    }

    pub(super) fn absorb_state(&mut self, result: StateResult) {
        self.timing_by_segment
            .entry(result.segment)
            .or_default()
            .absorb_state(&result.timings);
        for (store, record) in self.arm_stores.iter_mut().zip(result.arm_records) {
            store.push(record);
        }
        self.whole_unit_positions += result.whole_unit_positions;
        self.whole_unit_events += result.whole_unit_events;
        self.header_scan_positions += result.header_scan_positions;
        self.header_scan_events += result.header_scan_events;
        self.ring_record_positions += result.ring_record_positions;
        self.ring_record_events += result.ring_record_events;
        if result.whole_unit_positions == 0 {
            self.states_without_whole_unit_reads += 1;
        }
        self.consult_report_differs += u64::from(result.consult_report_differs);
        self.ignore_report_differs += u64::from(result.ignore_report_differs);
        self.verdicts_differ += u64::from(result.verdicts_differ);
        self.walk_reads_outside_normal_reads += u64::from(result.walk_reads_outside_normal_reads);
        if let Some(control) = result.control {
            self.control.push(control);
        }
    }

    pub(super) fn absorb(&mut self, other: StreamDomainTotals) {
        add_tally(&mut self.tally, &other.tally);
        self.tables.absorb(other.tables);
        for (segment, sums) in &other.timing_by_segment {
            self.timing_by_segment
                .entry(*segment)
                .or_default()
                .absorb(sums);
        }
        for (mine, theirs) in self.arm_stores.iter_mut().zip(other.arm_stores) {
            mine.absorb(theirs);
        }
        self.whole_unit_positions += other.whole_unit_positions;
        self.whole_unit_events += other.whole_unit_events;
        self.header_scan_positions += other.header_scan_positions;
        self.header_scan_events += other.header_scan_events;
        self.ring_record_positions += other.ring_record_positions;
        self.ring_record_events += other.ring_record_events;
        self.states_without_whole_unit_reads += other.states_without_whole_unit_reads;
        self.consult_report_differs += other.consult_report_differs;
        self.ignore_report_differs += other.ignore_report_differs;
        self.verdicts_differ += other.verdicts_differ;
        self.walk_reads_outside_normal_reads += other.walk_reads_outside_normal_reads;
        self.control.extend(other.control);
        self.control.sort_by_key(|control| control.ordinal);
    }

    pub(super) fn states(&self) -> u64 {
        self.tally.states
    }
}

/// 线程数：`E161_THREADS`，没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值；
/// 登记 5.6 的「没设取 `available_parallelism`」按每台默认改；线程数不进产物）。
pub(super) fn threads_from_environment() -> usize {
    threads_of_the_variable_values(
        std::env::var(THREADS_ENVIRONMENT_VARIABLE).ok().as_deref(),
        singlefs_verification::numeric_environment_variable::process_environment_variable(
            singlefs_verification::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE,
        )
        .as_deref(),
    )
}

/// [`threads_from_environment`] 的判法，两个变量的值由调用方给（`None` 是没设）。
///
/// # Panics
/// `E161_THREADS` 设了而不是正整数；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
fn threads_of_the_variable_values(
    e161_threads_value: Option<&str>,
    thread_cap_value: Option<&str>,
) -> usize {
    match e161_threads_value {
        Some(text) => text
            .parse::<usize>()
            .ok()
            .filter(|threads| *threads > 0)
            .unwrap_or_else(|| panic!("{THREADS_ENVIRONMENT_VARIABLE} 要是正整数，读到 {text:?}")),
        None => singlefs_verification::heavy_threads::default_heavy_threads_of_the_cap_value(
            thread_cap_value,
        )
        .get(),
    }
}

/// 一格要跑的状态：序号（按计划的次序）与要不要留对照数据。
pub(super) struct StreamDomainRun<'run> {
    pub(super) label: String,
    pub(super) context: &'run EvaluationContext<'run>,
    pub(super) plan: &'run LocalPlan<'run>,
    pub(super) ordinals: &'run [u64],
    pub(super) control_ordinals: &'run BTreeSet<u64>,
    pub(super) timing: TimingControl,
    pub(super) threads: usize,
    pub(super) depth: EvaluationDepth,
}

/// 一格里每个状态做到哪一步：停机条款 S4 只要照跑那一遍的计数；产物要 5.1 的四件事。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EvaluationDepth {
    BaselineOnly,
    FourSteps,
}

/// 多线程跑一格：按片领活，每个线程一份累计，最后逐份并起来（并法可交换，结果与线程数无关）。
pub(super) fn evaluate_stream_domain(run: &StreamDomainRun<'_>) -> StreamDomainTotals {
    let next_slice = AtomicUsize::new(0);
    let slices: Vec<&[u64]> = run.ordinals.chunks(STATES_PER_SLICE).collect();
    let finished_slices = AtomicUsize::new(0);
    let started = Instant::now();
    let merged = Mutex::new(StreamDomainTotals::new());
    std::thread::scope(|scope| {
        for _worker in 0..run.threads {
            scope.spawn(|| {
                let tables = RefCell::new(ContentTables::default());
                let mut totals = StreamDomainTotals::new();
                loop {
                    let slice_index = next_slice.fetch_add(1, Ordering::Relaxed);
                    let Some(slice) = slices.get(slice_index) else {
                        break;
                    };
                    for ordinal in *slice {
                        if run.depth == EvaluationDepth::BaselineOnly {
                            let image = CrashImage {
                                base: &run.context.stream.base,
                                writes: &run.context.table.writes,
                                persisted: run.plan.persisted_of(*ordinal),
                            };
                            let mut timings = StateTimings::default();
                            let _reports = evaluate_baseline(
                                run.context,
                                &image,
                                TimingControl::WALL_CLOCK_WITHOUT_BUSY_WAIT,
                                &mut timings,
                                &mut totals.tally,
                            );
                            continue;
                        }
                        let result = evaluate_state(
                            run.context,
                            *ordinal,
                            run.plan.segment_of(*ordinal),
                            run.plan.persisted_of(*ordinal),
                            run.timing,
                            run.control_ordinals.contains(ordinal),
                            &tables,
                            &mut totals.tally,
                        );
                        totals.absorb_state(result);
                    }
                    let finished = finished_slices.fetch_add(1, Ordering::Relaxed) + 1;
                    if finished.is_multiple_of(256) || finished == slices.len() {
                        eprintln!(
                            "progress cell={} slices={finished}/{} elapsed_seconds={}",
                            run.label,
                            slices.len(),
                            started.elapsed().as_secs()
                        );
                    }
                }
                totals.tables = tables.into_inner();
                merged.lock().expect("并累计的锁").absorb(totals);
            });
        }
    });
    merged.into_inner().expect("并累计的锁")
}

#[cfg(test)]
mod tests;
