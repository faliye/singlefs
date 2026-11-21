//! 确认丢失那一形的末段分类、四个输入的崩溃列与各类的判。

use super::*;

/// 分类并细分（类核对 ③）：起点是键 = 末段里不属于 `in_flight_call` 的写、类里 = 属于它的写；每一遍在每种取法的代表上经只转发、只记的读者
/// 跑看 journal 的恢复，读段与类里某个写相交就把那个写移进键、重做，直到一遍里一个都不相交（不动点）。移进键之后要判的态数超过
/// `state_limit` 就停（停机 S13）。迭代上界：每一遍不是到不动点就是至少移走一个写，最多 C 写数 + 1 遍。
pub(super) fn classify_the_last_segment(
    context: &ObservationContext<'_>,
    digits: &LastSegmentDigits,
    in_flight_call: usize,
    state_limit: u64,
) -> ConfirmedLossClassification {
    let stream = context.stream;
    let partition = ConfirmedLossPartition::before_any_refinement(
        &context.call_of_write,
        digits,
        in_flight_call,
    );
    let mut classification = ConfirmedLossClassification {
        in_flight_writes: partition.class_positions.len(),
        partition,
        refinements: 0,
        moved_positions: Vec::new(),
        representative_recoveries: BTreeMap::new(),
        overlapping_positions_left: 0,
        stopped_at_the_state_limit: false,
        judged_states_planned: 0,
    };
    loop {
        classification.judged_states_planned = classification.partition.judged_state_count(digits);
        if classification.judged_states_planned > state_limit {
            classification.stopped_at_the_state_limit = true;
            return classification;
        }
        let mut overlapping: BTreeSet<usize> = BTreeSet::new();
        let mut recoveries = BTreeMap::new();
        for keyed_assignment in 0..classification.partition.keyed_assignments(digits) {
            let representative = classification
                .partition
                .representative_ordinal(digits, keyed_assignment);
            let image = CrashImage {
                base: &stream.base,
                writes: &context.tables.enumerated_writes,
                persisted: context.facts.persisted_writes_of_state(representative),
            };
            let (recovery, read_ranges) = consulted_recovery_recording_the_reads(&image);
            for position in &classification.partition.class_positions {
                let write = &stream.writes[digits.write_indexes[*position]];
                if recorded_reads_overlap_the_write(&read_ranges, write) {
                    overlapping.insert(*position);
                }
            }
            recoveries.insert(representative, recovery);
        }
        classification.representative_recoveries = recoveries;
        if overlapping.is_empty() {
            return classification;
        }
        let mut refined = classification.partition.clone();
        refined
            .class_positions
            .retain(|position| !overlapping.contains(position));
        refined.keyed_positions.extend(overlapping.iter().copied());
        refined.keyed_positions.sort_unstable();
        if refined.judged_state_count(digits) > state_limit {
            classification.judged_states_planned = refined.judged_state_count(digits);
            classification.overlapping_positions_left = overlapping.len();
            classification.stopped_at_the_state_limit = true;
            return classification;
        }
        classification.partition = refined;
        classification.moved_positions.extend(overlapping);
        classification.moved_positions.sort_unstable();
        classification.refinements += 1;
    }
}

/// O1 两个崩溃子列与 RefFS 一票，只由修订 2 第 2.1 节「为什么这样分得开」列的四样输入算：看 journal 那一遍的恢复（读回由它出）、
/// 盘上已持久的最新根、触达的最后一次调用、段号（RefFS 的崩溃时刻）。等价类按它推；类核对 ① 拿它在判过的每一态上重算，
/// 与 [`observation_of_a_judged_state`] 那一路比（那一路多吃一样输入就对不上）。恢复 panic 的两列都记恢复 panic（读法 5）。
fn crash_columns_of_the_four_inputs(
    context: &ObservationContext<'_>,
    recovery: &ConsultedRecovery,
    newest_persisted_root_key: Option<ModelRootKey>,
    touched_call: Option<usize>,
    segment_index: usize,
) -> (CrashJudgeVerdict, ReferenceFileSystemVoteOfAState) {
    match recovery {
        ConsultedRecovery::Panicked(_) => (
            CrashJudgeVerdict::RecoveryPanicked,
            ReferenceFileSystemVoteOfAState::RecoveryPanicked,
        ),
        ConsultedRecovery::Reported(report) => {
            // 判官自己 panic 那一列记 `judge_panic`（修订 5 第 5.1 节第 3 条；与逐件重做那一路同一读法），读回连同判官一起包。
            let crash_judge =
                if crash_judge_is_applicable(touched_call, context.stream.model_stopped_at_call) {
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        crash_judge_verdict(
                            &context.stream.committed_versions,
                            newest_persisted_root_key,
                            &observed_read_back_after_a_crash(report),
                        )
                    }))
                    .unwrap_or(CrashJudgeVerdict::JudgePanicked)
                } else {
                    CrashJudgeVerdict::NotApplicable
                };
            let reference_filesystem =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    reference_filesystem_vote_of_a_state(
                        context.reference_filesystem,
                        context.reference_filesystem_moment_of(segment_index),
                        &observed_read_back_after_a_crash(report),
                    )
                }))
                .unwrap_or(ReferenceFileSystemVoteOfAState::JudgePanicked);
            (crash_judge, reference_filesystem)
        }
    }
}

/// 一态上「盘上已持久的最新根」（[`crash_judge_of_the_state`] 同一种数法：只看写表里的原写）。
fn newest_persisted_root_key_of_the_state(
    context: &ObservationContext<'_>,
    ordinal: u64,
) -> Option<ModelRootKey> {
    let stream = context.stream;
    let persisted = context.facts.persisted_writes_of_state(ordinal);
    newest_persisted_root(&stream.writes, &persisted[..stream.writes.len()])
        .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg))
}

/// 一类的触达调用：「C 一个没落」那一类是代表自己的；「C 至少落一个」那一类再算上类里那几个写的调用。
fn touched_call_of_the_class(
    representative_touched_call: Option<usize>,
    in_flight_writes_landed: InFlightWritesLanded,
    call_of_the_class_writes: Option<usize>,
) -> Option<usize> {
    match in_flight_writes_landed {
        InFlightWritesLanded::NoneLanded => representative_touched_call,
        InFlightWritesLanded::AtLeastOneLanded => {
            representative_touched_call.max(call_of_the_class_writes)
        }
    }
}

/// 一类的恢复与两个崩溃子列（「C 一个没落」那一类就是代表照原路径判出的；「C 至少落一个」那一类由代表推出）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ClassColumns {
    pub(super) recovery: ConsultedRecovery,
    pub(super) crash_judge: CrashJudgeVerdict,
    pub(super) reference_filesystem: ReferenceFileSystemVoteOfAState,
}

impl ClassColumns {
    pub(super) fn of_the_observation(observation: &StateObservation) -> Option<Self> {
        ConsultedRecovery::of_the_observation(observation).map(|recovery| Self {
            recovery,
            crash_judge: observation.crash_judge.clone(),
            reference_filesystem: observation.reference_filesystem.clone(),
        })
    }
}

/// 一类（`name=q5_class` 一行，Q5 计数的一行）。
pub(super) struct ConfirmedLossClass {
    pub(super) key: ConfirmedLossClassKey,
    /// 键里那几个写在代表上的落地串（字符同 `states.tsv`：0 没落、1 撕裂、2 持久），按末段里的位置排。
    pub(super) keyed_landing: String,
    pub(super) states: u64,
    pub(super) states_by_writes_landed_in_the_last_segment: BTreeMap<usize, u64>,
    pub(super) columns: ClassColumns,
    pub(super) consulted_root: Option<(InstanceGeneration, CheckpointTxg)>,
}

/// 类核对的各项计数（`name=q5_equivalence` 一行；任一项不是 0 走停机 S13）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct EquivalenceCheckCounts {
    /// ①：判过的态上，两个崩溃子列与 RefFS 一票照原路径算的，与只由四样输入重算的对不上的态数。
    pub(super) input_list_mismatches: usize,
    /// ②：C 的写经 `geometry.classify` 是根槽写、系统配置槽写或 journal 记录写的个数。
    pub(super) in_flight_writes_of_a_forbidden_kind: usize,
    /// ④ 前一半：代表上经读者跑的那一遍恢复，与崩溃点重放判同一态交的 `judgement.consulted` 不同的代表数。
    pub(super) recording_report_mismatches: usize,
    /// ④ 后一半：「C 全落」那一态照原路径判的恢复、两个崩溃子列与 RefFS 一票，与它所在的类推出的不同的态数。
    pub(super) derived_class_mismatches: usize,
    /// 代表与「C 全落」那一态的落地串不是分法说的那样（代表上类里的写不全是 0、「C 全落」上不全是持久，或代表不在它那一类）的态数。
    pub(super) class_landing_mismatches: usize,
    /// C 的写经 `geometry.classify` 各是哪一种（逐个列，按种类数）。
    pub(super) in_flight_write_kinds: BTreeMap<&'static str, usize>,
}

/// 各类与类核对 ①、②、④ 的计数（③ 在 [`classify_the_last_segment`] 里）。
pub(super) struct ConfirmedLossClassEvaluation {
    pub(super) classes: BTreeMap<ConfirmedLossClassKey, ConfirmedLossClass>,
    pub(super) checks: EquivalenceCheckCounts,
}

/// 由判过的态（每种取法的代表与「C 全落」那一态，都照原路径判、留了整份恢复报告）推出各类，并做类核对 ①、②、④。
///
/// # Panics
/// 判过的态里缺某种取法的代表或「C 全落」那一态（调用方没照 [`ConfirmedLossClassification::judged_ordinals`] 判）。
pub(super) fn evaluate_the_classes(
    context: &ObservationContext<'_>,
    digits: &LastSegmentDigits,
    classification: &ConfirmedLossClassification,
    observations: &[StateObservation],
    in_flight_call: usize,
) -> ConfirmedLossClassEvaluation {
    let stream = context.stream;
    let partition = &classification.partition;
    let observation_of_ordinal: BTreeMap<u64, &StateObservation> = observations
        .iter()
        .map(|observation| (observation.ordinal, observation))
        .collect();
    let mut checks = EquivalenceCheckCounts::default();
    let geometry = experiment_pool::geometry();
    for write_index in digits.write_indexes.iter().copied() {
        if context.call_of_write[write_index] != Some(in_flight_call) {
            continue;
        }
        let kind =
            geometry.classify(&stream.operations[stream.stream_indexes[write_index]].operation);
        *checks.in_flight_write_kinds.entry(kind.name()).or_insert(0) += 1;
        match kind {
            StepKind::RootRecordFua
            | StepKind::SystemConfigurationSlot
            | StepKind::JournalRecord => {
                checks.in_flight_writes_of_a_forbidden_kind += 1;
            }
            StepKind::ZeroFill | StepKind::Discard | StepKind::UnitWrite | StepKind::Barrier => {}
        }
    }
    for observation in observations {
        let Some(recovery) = ConsultedRecovery::of_the_observation(observation) else {
            continue;
        };
        let recomputed = crash_columns_of_the_four_inputs(
            context,
            &recovery,
            newest_persisted_root_key_of_the_state(context, observation.ordinal),
            observation.touched_call,
            observation.segment_index,
        );
        if recomputed
            != (
                observation.crash_judge.clone(),
                observation.reference_filesystem.clone(),
            )
        {
            checks.input_list_mismatches += 1;
        }
    }
    let call_of_the_class_writes = partition
        .class_positions
        .iter()
        .filter_map(|position| context.call_of_write[digits.write_indexes[*position]])
        .max();
    let keyed_landed_and_landing = |observation: &StateObservation| -> (usize, String) {
        let keyed_landing: String = partition
            .keyed_positions
            .iter()
            .map(|position| {
                char::from(observation.landing_vector.as_bytes()[digits.write_indexes[*position]])
            })
            .collect();
        let landed = keyed_landing
            .chars()
            .filter(|landing| *landing != '0')
            .count();
        (landed, keyed_landing)
    };
    let class_landings_are = |observation: &StateObservation, expected: char| -> bool {
        partition.class_positions.iter().all(|position| {
            char::from(observation.landing_vector.as_bytes()[digits.write_indexes[*position]])
                == expected
        })
    };
    let mut classes = BTreeMap::new();
    for keyed_assignment in 0..partition.keyed_assignments(digits) {
        let representative_ordinal = partition.representative_ordinal(digits, keyed_assignment);
        let representative = observation_of_ordinal
            .get(&representative_ordinal)
            .copied()
            .expect("每种取法的代表都照原路径判过（judged_ordinals）");
        let none_key = ConfirmedLossClassKey {
            keyed_assignment,
            in_flight_writes_landed: InFlightWritesLanded::NoneLanded,
        };
        if partition.class_of_ordinal(digits, representative_ordinal) != none_key
            || !class_landings_are(representative, '0')
        {
            checks.class_landing_mismatches += 1;
        }
        let recording_recovery = classification
            .representative_recoveries
            .get(&representative_ordinal)
            .expect("类核对 ③ 的最后一遍在每个代表上都跑过");
        let representative_columns = ClassColumns::of_the_observation(representative)
            .expect("按等价类判的那一趟留了整份恢复报告");
        if *recording_recovery != representative_columns.recovery {
            checks.recording_report_mismatches += 1;
        }
        let (keyed_landed, keyed_landing) = keyed_landed_and_landing(representative);
        classes.insert(
            none_key,
            ConfirmedLossClass {
                key: none_key,
                keyed_landing: keyed_landing.clone(),
                states: 1,
                states_by_writes_landed_in_the_last_segment: BTreeMap::from([(keyed_landed, 1)]),
                consulted_root: representative.consulted_root,
                columns: representative_columns,
            },
        );
        if partition.class_positions.is_empty() {
            continue;
        }
        let in_flight_writes_landed_in_the_class = InFlightWritesLanded::AtLeastOneLanded;
        let derived_touched_call = touched_call_of_the_class(
            representative.touched_call,
            in_flight_writes_landed_in_the_class,
            call_of_the_class_writes,
        );
        let (crash_judge, reference_filesystem) = crash_columns_of_the_four_inputs(
            context,
            recording_recovery,
            newest_persisted_root_key_of_the_state(context, representative_ordinal),
            derived_touched_call,
            representative.segment_index,
        );
        let derived = ClassColumns {
            recovery: recording_recovery.clone(),
            crash_judge,
            reference_filesystem,
        };
        let fully_landed_ordinal = partition.fully_landed_ordinal(digits, keyed_assignment);
        let fully_landed = observation_of_ordinal
            .get(&fully_landed_ordinal)
            .copied()
            .expect("每种取法的「C 全落」那一态都照原路径判过（judged_ordinals）");
        let some_key = ConfirmedLossClassKey {
            keyed_assignment,
            in_flight_writes_landed: InFlightWritesLanded::AtLeastOneLanded,
        };
        let highest_digit_landing = '2';
        if partition.class_of_ordinal(digits, fully_landed_ordinal) != some_key
            || !class_landings_are(fully_landed, highest_digit_landing)
        {
            checks.class_landing_mismatches += 1;
        }
        if ClassColumns::of_the_observation(fully_landed).as_ref() != Some(&derived) {
            checks.derived_class_mismatches += 1;
        }
        let states_by_writes_landed_in_the_last_segment = partition
            .states_by_in_flight_writes_landed(digits)
            .into_iter()
            .enumerate()
            .filter(|(_, states)| *states > 0)
            .map(|(in_flight_landed, states)| (keyed_landed + in_flight_landed, states))
            .collect();
        classes.insert(
            some_key,
            ConfirmedLossClass {
                key: some_key,
                keyed_landing,
                states: partition.states_with_the_in_flight_writes_landed(digits),
                states_by_writes_landed_in_the_last_segment,
                consulted_root: derived.recovery.effective_root(),
                columns: derived,
            },
        );
    }
    ConfirmedLossClassEvaluation { classes, checks }
}

/// 类核对 ①–④ 任一不过，或细分到上限：停机 S13（修订 2 第 2.1 节）。
pub(super) fn equivalence_stops_at_the_class_check(
    classification: &ConfirmedLossClassification,
    checks: &EquivalenceCheckCounts,
) -> bool {
    classification.stopped_at_the_state_limit
        || classification.overlapping_positions_left > 0
        || checks.input_list_mismatches > 0
        || checks.in_flight_writes_of_a_forbidden_kind > 0
        || checks.recording_report_mismatches > 0
        || checks.derived_class_mismatches > 0
        || checks.class_landing_mismatches > 0
}

/// `name=q5_equivalence` 那一行（与对拍那一行）的类核对计数：①–④ 各自的计数与 ③ 停下时还相交的写数（修订 5 第 5.5 节第 2 条：
/// 这两种行只放类核对的计数与判定）。`s13=` 由调用方接在后面：主取样点那一行是 [`equivalence_stops_at_the_class_check`]，对拍那一行是
/// [`EquivalencePairing::stops_at_the_class_check`]（再并上逐态不同与各类态数之和不等于组合数，第 5.5 节第 3 条）。
pub(super) fn equivalence_check_fields(
    classification: &ConfirmedLossClassification,
    checks: &EquivalenceCheckCounts,
) -> String {
    format!(
        "input_list_mismatches={} in_flight_writes_of_a_forbidden_kind={} read_overlaps_left={} recording_report_mismatches={} derived_class_mismatches={} class_landing_mismatches={}",
        checks.input_list_mismatches,
        checks.in_flight_writes_of_a_forbidden_kind,
        classification.overlapping_positions_left,
        checks.recording_report_mismatches,
        checks.derived_class_mismatches,
        checks.class_landing_mismatches,
    )
}

/// `name=q5_partition` 那一行（主取样点与对拍各一行）的分法字段：C 的写数与经 `geometry.classify` 逐个列的种类（类核对 ②）、键里与类里的写
/// （末段里的位置，A10 比对器按它现算各类态数）、要照原路径判的态数与上限，加修订 5 第 5.5 节第 2 条从类核对那一行挪来的细分次数、
/// 移进键的写与停没停在上限（`stopped_at_the_state_limit` 经 `s13=` 进 S13，这一行的字段不直接当触发观测）。
pub(super) fn partition_fields(
    classification: &ConfirmedLossClassification,
    checks: &EquivalenceCheckCounts,
) -> String {
    let kinds: Vec<String> = checks
        .in_flight_write_kinds
        .iter()
        .map(|(kind, count)| format!("{kind}:{count}"))
        .collect();
    format!(
        "in_flight_writes={} in_flight_write_kinds={} keyed_last_segment_positions={} class_last_segment_positions={} judged_states_planned={} state_limit={CONFIRMED_LOSS_STATE_LIMIT} refinements={} moved_last_segment_positions={} stopped_at_the_state_limit={}",
        classification.in_flight_writes,
        if kinds.is_empty() { "-".to_string() } else { kinds.join(",") },
        field_text(&format!("{:?}", classification.partition.keyed_positions)),
        field_text(&format!("{:?}", classification.partition.class_positions)),
        classification.judged_states_planned,
        classification.refinements,
        field_text(&format!("{:?}", classification.moved_positions)),
        classification.stopped_at_the_state_limit
    )
}

#[cfg(test)]
mod tests;
