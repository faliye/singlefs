//! 等价配对与块内 panic 的对照。

use super::*;

/// 「等价类对拍」（修订 2 第 2.1 节候选二「用例钉住等价关系」，每一跑也做一遍）：末段与全持久态全枚举、逐态照原路径判，再按等价类推，
/// 逐态比恢复、两个崩溃子列与 RefFS 一票。
pub(super) struct EquivalencePairing {
    pub(super) states: u64,
    pub(super) product_of_the_choices: u64,
    /// 各类态数之和（A10：必须等于末段每个写的取法数之积）。
    pub(super) class_states: u64,
    pub(super) classes: usize,
    pub(super) mismatching_states: usize,
    pub(super) classification: ConfirmedLossClassification,
    pub(super) checks: EquivalenceCheckCounts,
    pub(super) observations: Vec<StateObservation>,
}

impl EquivalencePairing {
    /// 对拍有一态不同、各类态数之和不等于组合数，或类核对不过：停机 S13。
    pub(super) fn stops_at_the_class_check(&self) -> bool {
        self.mismatching_states > 0
            || self.class_states != self.product_of_the_choices
            || equivalence_stops_at_the_class_check(&self.classification, &self.checks)
    }
}

/// 在一条 Q5 那一形的流上做「等价类对拍」。跑法里 Q5 那一处自修订 6 起改调 [`equivalence_pairing_with_injected_piece_panics`]
/// （注入为空时同义），这个只剩内联用例在用。
#[cfg(test)]
pub(super) fn equivalence_pairing_of(stream: &PreparedStream) -> EquivalencePairing {
    equivalence_pairing_with_injected_piece_panics(stream, None)
}

/// 同 [`equivalence_pairing_of`]，`piece_panic_injection` 给了的，判态时照它注入（阳性对照「判态路 panic 单列」，修订 5 第 5.1 节）。
pub(super) fn equivalence_pairing_with_injected_piece_panics(
    stream: &PreparedStream,
    piece_panic_injection: Option<&PiecePanicInjection>,
) -> EquivalencePairing {
    let observation_tables = ObservationTables::of(stream);
    let mut context = observation_tables.context(stream);
    context.piece_panic_injection = piece_panic_injection;
    let context = context;
    let digits = LastSegmentDigits::of(stream, &observation_tables.tables);
    let product_of_the_choices =
        digits.product_of_the_radixes(&(0..digits.write_indexes.len()).collect::<Vec<usize>>());
    let classification = classify_the_last_segment(
        &context,
        &digits,
        CONFIRMED_LOSS_IN_FLIGHT_CALL,
        CONFIRMED_LOSS_STATE_LIMIT,
    );
    let every_state: BTreeSet<u64> =
        (digits.first_ordinal..observation_tables.tables.state_count).collect();
    let observations = observe_sampled_states(
        &context,
        &SampledOrdinals {
            batches: [every_state, BTreeSet::new(), BTreeSet::new()],
            torn_scans: Vec::new(),
        },
        RunSwitches::default(),
        ConsultedReports::Kept,
    );
    if classification.stopped_at_the_state_limit {
        return EquivalencePairing {
            states: u64::try_from(observations.len()).expect("态数"),
            product_of_the_choices,
            class_states: 0,
            classes: 0,
            mismatching_states: 0,
            classification,
            checks: EquivalenceCheckCounts::default(),
            observations,
        };
    }
    let evaluation = evaluate_the_classes(
        &context,
        &digits,
        &classification,
        &observations,
        CONFIRMED_LOSS_IN_FLIGHT_CALL,
    );
    let mismatching_states = observations
        .iter()
        .filter(|observation| {
            let key = classification
                .partition
                .class_of_ordinal(&digits, observation.ordinal);
            evaluation.classes.get(&key).map(|class| &class.columns)
                != ClassColumns::of_the_observation(observation).as_ref()
        })
        .count();
    EquivalencePairing {
        states: u64::try_from(observations.len()).expect("态数"),
        product_of_the_choices,
        class_states: evaluation.classes.values().map(|class| class.states).sum(),
        classes: evaluation.classes.len(),
        mismatching_states,
        classification,
        checks: evaluation.checks,
        observations,
    }
}

/// 「等价类对拍」那一形的三样（修订 5 第 5.2 节 S1 新加三条、U9）：末段里属于第 3 次调用的写（c1…c_k，按发出次序，写表下标）、
/// 末段各写取法数之积、c1…c_k 按发出次序两两成对（同偏移、同长度、分属两块盘）且各对偏移互不相同的对数。
pub(super) struct EquivalencePairingShapeFacts {
    pub(super) in_flight_writes: Vec<usize>,
    pub(super) states: u64,
    pub(super) mirrored_pairs: usize,
}

pub(super) fn equivalence_pairing_shape_facts(
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
) -> EquivalencePairingShapeFacts {
    let call_of_write = call_index_of_each_write(stream);
    let in_flight_writes: Vec<usize> = stream
        .segments
        .last()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|write_index| call_of_write[*write_index] == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL))
        .collect();
    let (_, _, states) = last_segment_product(stream, tables);
    let mut pair_offsets: BTreeSet<u64> = BTreeSet::new();
    let mut mirrored_pairs = 0usize;
    for pair in in_flight_writes.chunks(2) {
        if let [first, second] = pair {
            let first_write = &stream.writes[*first];
            let second_write = &stream.writes[*second];
            if first_write.offset == second_write.offset
                && first_write.length_in_bytes() == second_write.length_in_bytes()
                && first_write.device != second_write.device
                && pair_offsets.insert(first_write.offset.0)
            {
                mirrored_pairs += 1;
            }
        }
    }
    EquivalencePairingShapeFacts {
        in_flight_writes,
        states,
        mirrored_pairs,
    }
}

/// 阳性对照「判态路 panic 单列」的注入（修订 5 第 5.1 节）：c1…c6 是对拍流末段里第 3 次调用的前 6 个写（三对镜像）；
/// `pool_checker`：c3 落、c4 没落；`ignored_recovery`：c3 没落、c4 落；`record_checker`：c5 落、c6 没落，且 (c3,c4) 同落或同不落。
/// 三条选中的态互不相交。C 的写不够 6 个交 None（阳性对照不成立，走 F9）。
fn piece_panic_control_injection(in_flight_writes: &[usize]) -> Option<PiecePanicInjection> {
    let [_, _, third, fourth, fifth, sixth] = in_flight_writes.get(..6)? else {
        return None;
    };
    Some(PiecePanicInjection {
        rules: vec![
            PiecePanicInjectionRule {
                piece: Some(JudgingPiece::PoolChecker),
                conditions: vec![
                    InjectedLandingCondition::Landed(*third),
                    InjectedLandingCondition::NotLanded(*fourth),
                ],
            },
            PiecePanicInjectionRule {
                piece: Some(JudgingPiece::IgnoredRecovery),
                conditions: vec![
                    InjectedLandingCondition::NotLanded(*third),
                    InjectedLandingCondition::Landed(*fourth),
                ],
            },
            PiecePanicInjectionRule {
                piece: Some(JudgingPiece::RecordChecker),
                conditions: vec![
                    InjectedLandingCondition::Landed(*fifth),
                    InjectedLandingCondition::NotLanded(*sixth),
                    InjectedLandingCondition::BothOrNeitherLanded(*third, *fourth),
                ],
            },
        ],
    })
}

/// 一态上 O1 两个崩溃子列与看 journal 那遍恢复的结局（阳性对照「判态路 panic 单列」逐态比的三样）。
fn crash_columns_and_consulted_recovery(
    observation: &StateObservation,
) -> (
    &CrashJudgeVerdict,
    &ReferenceFileSystemVoteOfAState,
    Option<ConsultedRecovery>,
) {
    (
        &observation.crash_judge,
        &observation.reference_filesystem,
        ConsultedRecovery::of_the_observation(observation),
    )
}

/// 注入那一遍对不注入那一遍，两个崩溃子列或看 journal 那遍恢复的结局不同的态数（按崩溃状态序号配对；不注入那一遍里没有的序号也算不同）。
fn crash_judge_columns_differing(
    plain: &[StateObservation],
    injected: &[StateObservation],
) -> usize {
    let plain_of_ordinal: BTreeMap<u64, &StateObservation> = plain
        .iter()
        .map(|observation| (observation.ordinal, observation))
        .collect();
    injected
        .iter()
        .filter(|observation| {
            plain_of_ordinal
                .get(&observation.ordinal)
                .is_none_or(|plain_observation| {
                    crash_columns_and_consulted_recovery(plain_observation)
                        != crash_columns_and_consulted_recovery(observation)
                })
        })
        .count()
}

/// 阳性对照「判态路 panic 单列」那一遍（`name=q5_piece_panic_control` 一行）。
pub(super) struct PiecePanicControl {
    pub(super) states: u64,
    pub(super) piece_panic_states: [usize; JUDGING_PIECE_COUNT],
    pub(super) consulted_recovery_panics_of_the_plain_pairing: usize,
    pub(super) crash_judge_columns_differing_from_the_plain_pairing: usize,
    pub(super) mismatching_states: usize,
    pub(super) stops_at_the_class_check: bool,
    pub(super) observations: Vec<StateObservation>,
}

impl PiecePanicControl {
    /// 注入的三件与看 journal 那遍恢复之外的件 panic 的态数之和。
    pub(super) fn other_piece_panic_states(&self) -> usize {
        JudgingPiece::ALL
            .iter()
            .zip(self.piece_panic_states)
            .filter(|(piece, _)| {
                !matches!(
                    piece,
                    JudgingPiece::ConsultedRecovery
                        | JudgingPiece::PoolChecker
                        | JudgingPiece::IgnoredRecovery
                        | JudgingPiece::RecordChecker
                )
            })
            .map(|(_, states)| states)
            .sum()
    }

    /// 恰为修订 5 第 5.1 节的精确数（不符走 F9）。
    pub(super) fn holds(&self) -> bool {
        let states_of = |piece: JudgingPiece| {
            u64::try_from(self.piece_panic_states[piece.position()]).expect("态数装得进 u64")
        };
        self.states == LOCAL_EQUIVALENCE_PAIRING_STATES
            && states_of(JudgingPiece::PoolChecker) == LOCAL_PIECE_PANIC_CONTROL_POOL_CHECKER_STATES
            && states_of(JudgingPiece::IgnoredRecovery)
                == LOCAL_PIECE_PANIC_CONTROL_IGNORED_RECOVERY_STATES
            && states_of(JudgingPiece::RecordChecker)
                == LOCAL_PIECE_PANIC_CONTROL_RECORD_CHECKER_STATES
            && self.other_piece_panic_states() == 0
            && self.piece_panic_states[JudgingPiece::ConsultedRecovery.position()]
                == self.consulted_recovery_panics_of_the_plain_pairing
            && self.crash_judge_columns_differing_from_the_plain_pairing == 0
            && self.mismatching_states == 0
            && !self.stops_at_the_class_check
    }
}

/// 在对拍那一条流上照 [`piece_panic_control_injection`] 注入、再做一遍对拍，与不注入那一遍（`plain`）逐态比。
/// `replacing_the_built_in_rules` 不空时用它替换内置的那三条规则（登记修订 6 第 6.2 节第 3 条，只有内联用例 U14 交）。
pub(super) fn piece_panic_control_of(
    stream: &PreparedStream,
    plain: &EquivalencePairing,
    replacing_the_built_in_rules: Option<&PiecePanicInjection>,
) -> PiecePanicControl {
    let tables = crash_state_enumeration_tables(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let facts = equivalence_pairing_shape_facts(stream, &tables);
    let consulted_recovery_panics_of_the_plain_pairing =
        piece_panic_states_of(&plain.observations)[JudgingPiece::ConsultedRecovery.position()];
    let built_in_injection = piece_panic_control_injection(&facts.in_flight_writes);
    let Some(injection) = replacing_the_built_in_rules.or(built_in_injection.as_ref()) else {
        return PiecePanicControl {
            states: 0,
            piece_panic_states: [0; JUDGING_PIECE_COUNT],
            consulted_recovery_panics_of_the_plain_pairing,
            crash_judge_columns_differing_from_the_plain_pairing: 0,
            mismatching_states: 0,
            stops_at_the_class_check: false,
            observations: Vec::new(),
        };
    };
    let injected = equivalence_pairing_with_injected_piece_panics(stream, Some(injection));
    PiecePanicControl {
        states: injected.states,
        piece_panic_states: piece_panic_states_of(&injected.observations),
        consulted_recovery_panics_of_the_plain_pairing,
        crash_judge_columns_differing_from_the_plain_pairing: crash_judge_columns_differing(
            &plain.observations,
            &injected.observations,
        ),
        mismatching_states: injected.mismatching_states,
        stops_at_the_class_check: injected.stops_at_the_class_check(),
        observations: injected.observations,
    }
}

#[cfg(test)]
mod tests;
