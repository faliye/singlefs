//! 每个状态上做的四件事（登记 5.1）：基线、各臂的键与判定、checker 读的计数与重放。

use super::*;

// ───────────────────────────── 每个状态上做的四件事（5.1） ─────────────────────────────

/// G2 的两条复用臂与成对键、G4 的三条复用臂（第五节 5.2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Arm {
    RecoveryConsult,
    RecoveryIgnore,
    RecoveryPair,
    WalkUnits,
    AllUnits,
    FullReadSet,
}

pub(super) const ARMS: [Arm; 6] = [
    Arm::RecoveryConsult,
    Arm::RecoveryIgnore,
    Arm::RecoveryPair,
    Arm::WalkUnits,
    Arm::AllUnits,
    Arm::FullReadSet,
];

impl Arm {
    pub(super) fn slot(self) -> usize {
        match self {
            Self::RecoveryConsult => 0,
            Self::RecoveryIgnore => 1,
            Self::RecoveryPair => 2,
            Self::WalkUnits => 3,
            Self::AllUnits => 4,
            Self::FullReadSet => 5,
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RecoveryConsult => "k2_consult",
            Self::RecoveryIgnore => "k2_ignore",
            Self::RecoveryPair => "k2_pair",
            Self::WalkUnits => "k4_walk",
            Self::AllUnits => "k4_units",
            Self::FullReadSet => "k4_full",
        }
    }

    /// G5 P3 的每单元字节：读集按调用 21，集合按元素 20（第六节 G5）。
    pub(super) fn bytes_per_element(self) -> u64 {
        match self {
            Self::RecoveryConsult
            | Self::RecoveryIgnore
            | Self::RecoveryPair
            | Self::FullReadSet => READ_SET_BYTES_PER_CALL,
            Self::WalkUnits | Self::AllUnits => SET_BYTES_PER_ELEMENT,
        }
    }
}

/// 一条臂上一个状态落的定长记录（5.1 第 4 步）：键、序号、结局指纹、读集或集合的大小。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ArmRecord {
    pub(super) key: Digest128,
    pub(super) ordinal: u64,
    pub(super) outcome: Digest128,
    pub(super) elements: u32,
}

/// 对照样本上一个状态的全部读集（PC2c、PC4c 按位置去掉再算键）。
pub(super) struct ControlStateData {
    pub(super) ordinal: u64,
    pub(super) consult_calls: Vec<LoggedCall>,
    pub(super) ignore_calls: Vec<LoggedCall>,
    pub(super) full_calls: Vec<LoggedCall>,
    pub(super) root: Digest128,
    pub(super) walk_elements: SetElements,
    pub(super) unit_elements: SetElements,
    pub(super) consult_outcome: Digest128,
    pub(super) ignore_outcome: Digest128,
    pub(super) verdict_outcome: Digest128,
}

/// 一个状态评完交回的东西（计数记进调用方给的 `CrashPointReplayTally` 与内容表，其余在这里）。
pub(super) struct StateResult {
    pub(super) segment: usize,
    pub(super) timings: StateTimings,
    pub(super) arm_records: [ArmRecord; 6],
    pub(super) consult_report_differs: bool,
    pub(super) ignore_report_differs: bool,
    pub(super) verdicts_differ: bool,
    pub(super) whole_unit_positions: u64,
    pub(super) whole_unit_events: u64,
    pub(super) header_scan_positions: u64,
    pub(super) header_scan_events: u64,
    pub(super) ring_record_positions: u64,
    pub(super) ring_record_events: u64,
    pub(super) walk_reads_outside_normal_reads: bool,
    pub(super) control: Option<ControlStateData>,
}

/// 评状态要的只读输入。
pub(super) struct EvaluationContext<'context> {
    pub(super) stream: &'context PreparedStream,
    pub(super) table: &'context TornWriteTable,
    filesystem_identifier_low: u64,
}

impl EvaluationContext<'_> {
    pub(super) fn new<'context>(
        stream: &'context PreparedStream,
        table: &'context TornWriteTable,
    ) -> EvaluationContext<'context> {
        EvaluationContext {
            stream,
            table,
            filesystem_identifier_low: u64::from_le_bytes(
                experiment_pool::E142_FILESYSTEM_IDENTIFIER[..8]
                    .try_into()
                    .expect("8 字节"),
            ),
        }
    }
}

/// 照跑一个状态（5.1 第 1 步，真实基线）：次序与 `crash.rs` 的 `evaluate_state_for_versions` 逐步相同，每一步各用一对
/// 读钟（`StateClock::now`，钟的种类照 `timing`）包住，外面再包一对量整个状态；计数照它记进 `tally`。交回两遍恢复的报告与判定表。
// gpu-exempt: experiment-measures-the-cpu-judge-itself
pub(super) fn evaluate_baseline(
    context: &EvaluationContext<'_>,
    image: &CrashImage<'_>,
    timing: TimingControl,
    timings: &mut StateTimings,
    tally: &mut CrashPointReplayTally,
) -> (
    RecoveryReport,
    RecoveryReport,
    Vec<(&'static str, InvariantVerdict)>,
) {
    let mut clock = StateClock::new(timing.clock);
    let state_started = clock.now();
    let oracle_prefix_started = clock.now();
    let newest = newest_persisted_root(image.writes, &image.persisted);
    let oracle_prefix_nanoseconds = clock.nanoseconds_since(oracle_prefix_started);
    let consult_started = clock.now();
    clock.busy_wait_if(TimingZone::RecoveryConsult, timing.busy_zone);
    let consulted = recover(image, JournalPolicy::Consult);
    timings.recovery_consult_nanoseconds += clock.nanoseconds_since(consult_started);
    let ignore_started = clock.now();
    clock.busy_wait_if(TimingZone::RecoveryIgnore, timing.busy_zone);
    let ignored = recover(image, JournalPolicy::Ignore);
    timings.recovery_ignore_nanoseconds += clock.nanoseconds_since(ignore_started);
    let oracle_started = clock.now();
    clock.busy_wait_if(TimingZone::Oracle, timing.busy_zone);
    let consulted_violation = singlefs_verification::crash::oracle_violation_for_versions(
        &consulted.outcome,
        consulted.effective_root,
        newest,
        &context.stream.versions,
    );
    let ignored_violation = singlefs_verification::crash::oracle_violation_for_versions(
        &ignored.outcome,
        ignored.effective_root,
        newest,
        &context.stream.versions,
    );
    timings.oracle_nanoseconds +=
        clock.nanoseconds_since(oracle_started) + oracle_prefix_nanoseconds;
    let checker_started = clock.now();
    clock.busy_wait_if(TimingZone::PoolChecker, timing.busy_zone);
    let verdicts = check_pool_image(image);
    timings.pool_checker_nanoseconds += clock.nanoseconds_since(checker_started);
    let records_started = clock.now();
    clock.busy_wait_if(TimingZone::RecordChecker, timing.busy_zone);
    let records = check_records(image, consulted.effective_root);
    timings.record_checker_nanoseconds += clock.nanoseconds_since(records_started);
    timings.state_nanoseconds += clock.nanoseconds_since(state_started);
    tally.states += 1;
    if image.persisted[context.stream.judged_root_index] {
        tally.root_persisted_states += 1;
    }
    if recovery_passes_differ(&consulted, &ignored) {
        tally.journal_differing_states += 1;
    }
    match &consulted.outcome {
        RecoveryOutcome::NoFile { .. } => tally.no_file_states += 1,
        RecoveryOutcome::FileRead { .. } => tally.file_read_states += 1,
        RecoveryOutcome::Failed { .. } => tally.failed_states += 1,
    }
    if consulted_violation.is_some() {
        tally.violations += 1;
    }
    if ignored_violation.is_some() {
        tally.ignored_violations += 1;
    }
    for (invariant, verdict) in &verdicts {
        match verdict {
            InvariantVerdict::Holds => {
                *tally.checker_evaluated_states.entry(invariant).or_insert(0) += 1
            }
            InvariantVerdict::Violated(_) => {
                *tally.checker_evaluated_states.entry(invariant).or_insert(0) += 1;
                *tally.checker_violated_states.entry(invariant).or_insert(0) += 1;
            }
            // 部分没判不红、不算评估过：与崩溃点重放那一路（`crash.rs`）同一个口径，计进不适用这一格。
            InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {
                *tally
                    .checker_not_applicable_states
                    .entry(invariant)
                    .or_insert(0) += 1;
            }
        }
    }
    if records.root_without_record {
        tally.record_root_without_record += 1;
    }
    if records.claimed_state_missing_unit {
        tally.record_claimed_state_missing_unit += 1;
    }
    (consulted, ignored, verdicts)
}

/// checker 正常那一遍的读按 G1 读法分出来的计数与重放用的字节。
struct CheckerReadCounts {
    pub(super) whole_unit_positions: u64,
    pub(super) whole_unit_events: u64,
    pub(super) header_scan_positions: u64,
    pub(super) header_scan_events: u64,
    pub(super) ring_record_positions: u64,
    pub(super) ring_record_events: u64,
}

fn count_checker_reads(reads: &[LoggedRead]) -> CheckerReadCounts {
    let mut whole_unit = BTreeSet::new();
    let mut header_scan = BTreeSet::new();
    let mut ring_records = BTreeSet::new();
    let mut counts = CheckerReadCounts {
        whole_unit_positions: 0,
        whole_unit_events: 0,
        header_scan_positions: 0,
        header_scan_events: 0,
        ring_record_positions: 0,
        ring_record_events: 0,
    };
    for read in reads.iter().filter(|read| read.content.is_some()) {
        let position = (read.device, read.offset, read.length);
        match ReadCategory::of(read.offset, read.length) {
            ReadCategory::WholeUnit => {
                counts.whole_unit_events += 1;
                whole_unit.insert(position);
            }
            ReadCategory::UnitHeaderScan => {
                counts.header_scan_events += 1;
                header_scan.insert(position);
            }
            ReadCategory::JournalRing => {
                if u64::try_from(read.length).expect("读长") == LOCAL_JOURNAL_RECORD_BYTES {
                    counts.ring_record_events += 1;
                    ring_records.insert(position);
                }
            }
            ReadCategory::FixedStructure => {}
        }
    }
    counts.whole_unit_positions = u64::try_from(whole_unit.len()).expect("个数");
    counts.header_scan_positions = u64::try_from(header_scan.len()).expect("个数");
    counts.ring_record_positions = u64::try_from(ring_records.len()).expect("个数");
    counts
}

/// Q3f：每个不同单元位置重放一次 U；Q3g：每一次单元读重放 U、每一次头扫描读算一次按位 CRC、每一次环内 4096 字节读判一次记录。
fn replay_checker_reads(
    reads: &[LoggedRead],
    filesystem_identifier_low: u64,
    timing: TimingControl,
    timings: &mut StateTimings,
) {
    let mut clock = StateClock::new(timing.clock);
    let positions_started = clock.now();
    clock.busy_wait_if(TimingZone::UnitCheckReplay, timing.busy_zone);
    let mut replayed_positions = BTreeSet::new();
    for read in reads {
        if let (ReadCategory::WholeUnit, Some((_digest, bytes))) =
            (ReadCategory::of(read.offset, read.length), &read.content)
        {
            if replayed_positions.insert((read.device, read.offset, read.length)) {
                replay_unit_check(bytes);
            }
        }
    }
    timings.unit_positions_replay_nanoseconds += clock.nanoseconds_since(positions_started);
    let events_started = clock.now();
    for read in reads {
        let Some((_digest, bytes)) = &read.content else {
            continue;
        };
        match ReadCategory::of(read.offset, read.length) {
            ReadCategory::WholeUnit => replay_unit_check(bytes),
            ReadCategory::UnitHeaderScan => {
                black_box(singlefs_checker::crc32_castagnoli_bitwise(bytes));
            }
            ReadCategory::JournalRing => {
                if u64::try_from(read.length).expect("读长") == LOCAL_JOURNAL_RECORD_BYTES {
                    black_box(
                        singlefs_checker::check_journal_record(bytes, filesystem_identifier_low)
                            .is_ok(),
                    );
                }
            }
            ReadCategory::FixedStructure => {}
        }
    }
    timings.unit_events_replay_nanoseconds += clock.nanoseconds_since(events_started);
}

/// 一个状态上的四件事（5.1）：照跑计时、记录跑、单元级检查重放、出键。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：输入、序号、段、持久集合、计时控制（忙等区与钟）、要不要留对照数据、内容表、计数"
)]
// gpu-exempt: experiment-measures-the-cpu-judge-itself
pub(super) fn evaluate_state(
    context: &EvaluationContext<'_>,
    ordinal: u64,
    segment: usize,
    persisted: Vec<bool>,
    timing: TimingControl,
    keep_control_data: bool,
    tables: &RefCell<ContentTables>,
    tally: &mut CrashPointReplayTally,
) -> StateResult {
    let image = CrashImage {
        base: &context.stream.base,
        writes: &context.table.writes,
        persisted,
    };
    let mut timings = StateTimings::default();
    let (consulted, ignored, verdicts) =
        evaluate_baseline(context, &image, timing, &mut timings, tally);
    let consult_reader = RecordingReader::new(&image, RecordedPass::Recovery, ordinal, tables);
    let recorded_consult = recover(&consult_reader, JournalPolicy::Consult);
    let ignore_reader = RecordingReader::new(&image, RecordedPass::Recovery, ordinal, tables);
    let recorded_ignore = recover(&ignore_reader, JournalPolicy::Ignore);
    let normal_reader = RecordingReader::new(&image, RecordedPass::CheckerNormal, ordinal, tables);
    let recorded_verdicts = check_pool_image(&normal_reader);
    let walk_reader = RecordingReader::new(&image, RecordedPass::CheckerWalk, ordinal, tables);
    let walk_clock = StateClock::new(timing.clock);
    let walk_started = walk_clock.now();
    black_box(check_pool_image(&walk_reader));
    timings.walk_checker_nanoseconds += walk_clock.nanoseconds_since(walk_started);
    let normal_reads = normal_reader.reads.into_inner();
    replay_checker_reads(
        &normal_reads,
        context.filesystem_identifier_low,
        timing,
        &mut timings,
    );
    let counts = count_checker_reads(&normal_reads);
    let root = chosen_root_digest(&image);
    let walk_elements = unit_area_elements(&walk_reader.reads.into_inner());
    let unit_elements = unit_area_elements(&normal_reads);
    let consult_calls = consult_reader.calls.into_inner();
    let ignore_calls = ignore_reader.calls.into_inner();
    let full_calls = normal_reader.calls.into_inner();
    let consult_outcome = fingerprint_of_debug_text(&consulted);
    let ignore_outcome = fingerprint_of_debug_text(&ignored);
    let verdict_outcome = fingerprint_of_debug_text(&verdicts);
    let consult_key = sequence_key(&consult_calls);
    let ignore_key = sequence_key(&ignore_calls);
    let length_of = |length: usize| u32::try_from(length).expect("读集长度装得进 u32");
    let record = |key: Digest128, outcome: Digest128, elements: usize| ArmRecord {
        key,
        ordinal,
        outcome,
        elements: length_of(elements),
    };
    let arm_records = [
        record(consult_key, consult_outcome, consult_calls.len()),
        record(ignore_key, ignore_outcome, ignore_calls.len()),
        record(
            pair_digest(consult_key, ignore_key),
            pair_digest(consult_outcome, ignore_outcome),
            consult_calls.len() + ignore_calls.len(),
        ),
        record(
            set_key(root, &walk_elements),
            verdict_outcome,
            walk_elements.len(),
        ),
        record(
            set_key(root, &unit_elements),
            verdict_outcome,
            unit_elements.len(),
        ),
        record(sequence_key(&full_calls), verdict_outcome, full_calls.len()),
    ];
    let walk_reads_outside_normal_reads = !walk_elements.is_subset(&unit_elements);
    StateResult {
        segment,
        timings,
        arm_records,
        consult_report_differs: recorded_consult != consulted,
        ignore_report_differs: recorded_ignore != ignored,
        verdicts_differ: recorded_verdicts != verdicts,
        whole_unit_positions: counts.whole_unit_positions,
        whole_unit_events: counts.whole_unit_events,
        header_scan_positions: counts.header_scan_positions,
        header_scan_events: counts.header_scan_events,
        ring_record_positions: counts.ring_record_positions,
        ring_record_events: counts.ring_record_events,
        walk_reads_outside_normal_reads,
        control: keep_control_data.then_some(ControlStateData {
            ordinal,
            consult_calls,
            ignore_calls,
            full_calls,
            root,
            walk_elements,
            unit_elements,
            consult_outcome,
            ignore_outcome,
            verdict_outcome,
        }),
    }
}

#[cfg(test)]
mod tests;
