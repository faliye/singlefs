//! 对拍的期望位（红项、池级 checker 违例位与判到位）与逐状态另看的几样（`StateExamination`、`StateCondition`）。

use super::*;

/// GPU 判器与 CPU 判器在这条流的每个状态上红位与违例位逐位相同、落到同一条根，CPU 池级 checker 在这条流上红 `invariant`。
/// 本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
pub(super) fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
    test_name: &str,
    flow: &RecordedFlow,
    invariant: &'static str,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains(invariant),
        "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        comparison
            .mismatches
            .iter()
            .filter(|line| !line.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 结局表（[`SCAN_REBUILD_OUTCOMES`]）里名字是 `constant_name` 的那一种的码。
fn scan_rebuild_outcome_code_named(constant_name: &str) -> u32 {
    SCAN_REBUILD_OUTCOMES
        .iter()
        .find(|outcome| outcome.constant_name == constant_name)
        .unwrap_or_else(|| panic!("结局表里有 {constant_name}"))
        .code
}

/// 数「GPU 交回的 I-1.13 结局码（结论的 [`SCAN_REBUILD_OUTCOME_WORD`]）是 `constant_name` 那一种」的状态（对拍时逐状态看，[`StateExamination`]）。
/// 对拍逐状态整份比（对拍字带这一字），没有不一致时 CPU 在同几个状态上换出的也是这一种。
pub(super) fn states_whose_scan_rebuild_ends_with(constant_name: &str) -> StateExamination {
    StateExamination::CountStatesWhere(StateCondition::GpuScanRebuildEndsWith(
        scan_rebuild_outcome_code_named(constant_name),
    ))
}

/// 同 [`assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening`]，再要这条流上至少一个状态两边判 I-1.13 都是池级 checker
/// 走到扫描重建之前整池交回的 `outcome_constant_name` 那一种（码 26 到 29：走读内核在那几处早退写的码，对拍逐位比它）。
pub(super) fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_and_ending_the_scan_rebuild_with(
    test_name: &str,
    flow: &RecordedFlow,
    invariant: &'static str,
    outcome_constant_name: &str,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        flow,
        expand_up_to(),
        &[states_whose_scan_rebuild_ends_with(outcome_constant_name)],
    ) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains(invariant),
        "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
    assert!(
        comparison.examined[0].states_counted > 0,
        "流 {} 上至少一个状态两边判 I-1.13 都是 {outcome_constant_name}",
        flow.name
    );
}

/// 一遍恢复的结局写成一串，读出的文件内容只写字节数与 SHA-256（[`JudgedOnCpu`] 里那一串只进不一致时的说明）。CPU 那一侧手里压着一段块
/// 每个状态一份：内容照 `{:?}` 写出来每个字节四五个字符，两遍各 3000 字节的内容一个状态约 27 KB，一段 8 块 52 万多个状态就是十几 GB（推的，
/// 按 `FILE_BYTES` 与块宽算）。没有文件与失败两种原样写。
pub(super) fn recovery_outcome_without_the_file_content(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::FileRead { root, content } => format!(
            "FileRead {{ root: {root:?}, content_bytes: {}, content_sha256: {} }}",
            content.len(),
            hexadecimal_text(&sha256_digest(content))
        ),
        RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. } => format!("{outcome:?}"),
    }
}

/// 一次恢复报出的那条根（(实例代号, checkpoint_txg)）：读出、没有文件两种结局带的那一条，失败时带的那一条（择不到根时没有）。
pub(super) fn root_reported_by_the_recovery(outcome: &RecoveryOutcome) -> Option<(u32, u64)> {
    match outcome {
        RecoveryOutcome::FileRead { root, .. } | RecoveryOutcome::NoFile { root } => {
            Some((root.0 .0, root.1 .0))
        }
        RecoveryOutcome::Failed { root, .. } => root.map(|(instance, txg)| (instance.0, txg.0)),
    }
}

/// 恢复摘要（[`recovery_words_of`]）里两遍的结局码：没有文件、读出、失败。
pub(super) const RECOVERY_WORD_CLASS_NO_FILE: u32 = 0;

pub(super) const RECOVERY_WORD_CLASS_FILE_READ: u32 = 1;

pub(super) const RECOVERY_WORD_CLASS_FAILED: u32 = 2;

fn oracle_index(kind: CrashPointReplayOracleViolationKind) -> u32 {
    u32::try_from(
        CrashPointReplayOracleViolationKind::EVERY_KIND
            .iter()
            .position(|candidate| *candidate == kind)
            .expect("七类之一"),
    )
    .expect("装得进 u32")
}

/// CPU 判定 → GPU 那一套红位。
pub(super) fn expected_bits(judgement: &CrashStateJudgement) -> u32 {
    let mut bits = 0u32;
    if let Some(violation) = &judgement.consulted_oracle {
        bits |= 1 << (verdict_bits::CONSULTED_ORACLE_FIRST + oracle_index(violation.kind));
    }
    if let Some(violation) = &judgement.ignored_oracle {
        bits |= 1 << (verdict_bits::IGNORED_ORACLE_FIRST + oracle_index(violation.kind));
    }
    if judgement.records.root_without_record {
        bits |= 1 << verdict_bits::ROOT_WITHOUT_RECORD;
    }
    if judgement.records.claimed_state_missing_unit {
        bits |= 1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT;
    }
    bits
}

/// CPU 池级 checker 的违例位：位 i ↔ `IMPLEMENTED_INVARIANTS[i]`；经 `pool_checker_violated_and_judged_bits` 取（与 CPU 一侧的对拍字同一个函数），
/// 内核不置位的那几条照判定置位，比较处再去掉。
pub(super) fn expected_pool_checker_bits(judgement: &CrashStateJudgement) -> u128 {
    let (violated, _judged) = pool_checker_violated_and_judged_bits(&judgement.pool_checker);
    violated
}

/// CPU 池级 checker 在这个状态上判过而成立的不变量（不是报不适用）：一位一条，与违例位同一个下标；经同一个函数取，判到而没违例的那几位。
pub(super) fn expected_pool_checker_held_bits(judgement: &CrashStateJudgement) -> u128 {
    let (violated, judged) = pool_checker_violated_and_judged_bits(&judgement.pool_checker);
    judged & !violated
}

pub(super) fn invariant_names(bits: u128) -> Vec<&'static str> {
    IMPLEMENTED_INVARIANTS
        .iter()
        .enumerate()
        .filter(|(index, _)| bits & (1u128 << index) != 0)
        .map(|(_, name)| *name)
        .collect()
}

/// 一份红项（与 `CrashStateJudgement::red_items` 同名、同序）在对拍里比的那一份，排成 `{:?}`：去掉内核不置位的那几条池级 checker 红项
/// （`POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS`，第一版只有 I-1.13，C821），别的照原序留着。
/// 与违例位比较处的 `pool_checker_bits_agree_in_the_cross_check` 同一张表：照流水线那样判不了就回落 CPU 的用例两边都经它再比，
/// CPU 红 I-1.13 的状态上 GPU 交回的红项里没有它，不算不一致。
pub(super) fn red_items_compared_by_the_cross_check(items: &[String]) -> String {
    let left_out: Vec<String> = POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS
        .iter()
        .map(|index| {
            format!(
                "{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}:{}",
                IMPLEMENTED_INVARIANTS[*index]
            )
        })
        .collect();
    let compared: Vec<&String> = items
        .iter()
        .filter(|item| !left_out.contains(item))
        .collect();
    format!("{compared:?}")
}

/// 不一致与判不了的状态各留几个带说明的：再多只计数、不留串（整条流几亿个状态时，每个状态留一个空串也装不下）。
const DESCRIBED_STATES: usize = 12;

/// 一份状态清单：几个状态，前 [`DESCRIBED_STATES`] 个带说明。按块累加（[`Self::take_the_block`]），整条流只留这么多。
/// 解引用成带说明的那几行：`iter()`、`&[String]` 照用；`state_count()`、`is_empty()` 是状态数，不是带说明的行数。
pub(super) struct StatesDescribed {
    pub(super) count: u64,
    pub(super) described: Vec<String>,
}

impl StatesDescribed {
    pub(super) fn none() -> Self {
        Self {
            count: 0,
            described: Vec::new(),
        }
    }

    /// 记一个状态：前 [`DESCRIBED_STATES`] 个照 `describe` 写说明，之后只计数。
    pub(super) fn note(&mut self, describe: impl FnOnce() -> String) {
        if self.described.len() < DESCRIBED_STATES {
            self.described.push(describe());
        }
        self.count += 1;
    }

    pub(super) fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// 状态数（不是带说明的行数）。
    pub(super) fn state_count(&self) -> usize {
        usize::try_from(self.count).expect("状态数装得进 usize")
    }

    /// 接上后一块的那一份：状态数相加，带说明的接到满 [`DESCRIBED_STATES`] 个为止（块按序号次序接，留下的是整条流最前面那几个）。
    pub(super) fn take_the_block(&mut self, block: Self) {
        self.count += block.count;
        let room = DESCRIBED_STATES.saturating_sub(self.described.len());
        self.described
            .extend(block.described.into_iter().take(room));
    }
}

impl std::ops::Deref for StatesDescribed {
    type Target = [String];

    fn deref(&self) -> &[String] {
        &self.described
    }
}

/// 逐状态另看的一样里「数状态」用的条件（[`StateExamination::CountStatesWhere`]）：每个状态上拿 CPU 那一侧交回的几样与 GPU 的结论字判。
/// 条件本身（连同取值）进对拍块的输入摘要，判法（[`state_condition_holds`]）在比法那一组的闭包里：改了它，库里早先的块不再复用。
#[derive(Clone, Debug)]
pub(super) enum StateCondition {
    /// CPU 池级 checker 红 `red` 而不红 `not_red`。
    CpuCheckerRedOnOneAndNotTheOther {
        red: &'static str,
        not_red: &'static str,
    },
    /// GPU 池级 checker 判得了（判不了的原因字是 0），且在 `invariant` 上置了判到位或违例位。
    GpuCheckerJudgedOrViolated(&'static str),
    /// CPU 判器的红项（`CrashStateJudgement::red_items` 的 `{:?}`）里有这一段。
    CpuRedItemsContain(String),
    /// CPU 看 journal 那一遍报的根（`RecoveryOutcome` 里的那一条；择不到根是 `None`）是它。
    CpuReportedRootIs(Option<(u32, u64)>),
    /// CPU 看 journal 那一遍实际走的那条根是 `version`（实例代号与完整的 checkpoint_txg）、结局码是 `class`。
    CpuConsultedPassLandsOn { version: (u32, u64), class: u32 },
    /// CPU 看 journal 那一遍报的根是实例 `instance` 的，实际走的那条根是另一条（从所选根往上施加了记录）。
    CpuReportedRootOfTheInstanceWhileTheConsultedPassLandsElsewhere { instance: u32 },
    /// GPU 交回的 I-1.13 结局码（结论的 [`SCAN_REBUILD_OUTCOME_WORD`]）是它。
    GpuScanRebuildEndsWith(u32),
    /// GPU 两遍恢复的失败点都是它。
    GpuBothPassesFailAt(u32),
    /// GPU 两遍恢复里至少一遍的失败点是它。
    GpuEitherPassFailsAt(u32),
    /// 几样都成立。
    AllOf(Vec<StateCondition>),
}

/// 逐状态另看的一样（对拍时每个状态上跟逐状态比一起看，按块累加进 [`ExaminationTally`]；要看逐状态结论的用例都经它，
/// 对拍不把整条流的逐态结论留在内存里）。
#[derive(Clone, Debug)]
pub(super) enum StateExamination {
    /// 数条件成立的状态，记前几个的序号。
    CountStatesWhere(StateCondition),
    /// 照流水线那样判不了就回落 CPU：GPU 池级 checker 判了的，违例位与 CPU 逐位相同；判不了的，原因字要在 `reasons_left_to_the_cpu` 里
    /// （交 CPU 的状态，其中原因字在 `reasons_one_state_must_carry` 里的另数、CPU 红 `invariant_red_on_a_state_left_to_the_cpu` 的另数）；
    /// 恢复内核判不了的，原因要在 `recovery_reasons_left_to_the_cpu` 里；回落之后每个状态的红项（两边都去掉内核不置位的那几条）都与 CPU 相同。
    LeftToTheCpuOnlyFor {
        reasons_left_to_the_cpu: Vec<u32>,
        recovery_reasons_left_to_the_cpu: Vec<u32>,
        reasons_one_state_must_carry: Vec<u32>,
        invariant_red_on_a_state_left_to_the_cpu: Option<&'static str>,
    },
    /// 逐盘自证过的系统配置槽数：GPU 走读内核交的与 CPU 参照（池级 checker 的逐盘数，CPU 那一侧在同一张崩溃镜像上现算）逐盘相同，判段那一判
    /// （对 `slots_on_the_base_image` 比）拿两份各判一次结局相同；数判段那一判记盘 `device_newly_dead` 是新出现的死盘的状态。
    SelfVerifiedSystemConfigurationSlotsOfEachDevice {
        slots_on_the_base_image: Vec<(u32, u32)>,
        device_newly_dead: u32,
    },
}

impl StateExamination {
    /// CPU 那一侧要不要在每张崩溃镜像上现算逐盘自证过的系统配置槽数。
    pub(super) fn needs_the_self_verified_slots_on_the_cpu(&self) -> bool {
        match self {
            Self::SelfVerifiedSystemConfigurationSlotsOfEachDevice { .. } => true,
            Self::CountStatesWhere(_) | Self::LeftToTheCpuOnlyFor { .. } => false,
        }
    }
}

/// 一样逐状态另看的结果（[`StateExamination`] 一样一份，按块累加）。
pub(super) struct ExaminationTally {
    /// 数状态那一种：条件成立的状态数；交 CPU 那一种：交 CPU 的状态里原因字在非要不可那几种里的状态数；
    /// 逐盘自证槽数那一种：判段那一判记点名那块盘新出现死盘的状态数。
    pub(super) states_counted: u64,
    /// 数状态那一种：条件成立的前 [`DESCRIBED_STATES`] 个状态的序号。
    pub(super) first_ordinals_counted: Vec<u64>,
    /// 交 CPU 那一种：GPU 池级 checker 因点名的原因判不了、交 CPU 的状态数。
    pub(super) states_left_to_the_cpu: u64,
    /// 交 CPU 那一种：交 CPU 的状态里 CPU 红点名那一条的状态数。
    pub(super) states_left_to_the_cpu_and_red_on_the_invariant: u64,
    /// 交 CPU 那一种与逐盘自证槽数那一种：不一致的状态。
    pub(super) disagreements: StatesDescribed,
}

impl ExaminationTally {
    pub(super) fn none() -> Self {
        Self {
            states_counted: 0,
            first_ordinals_counted: Vec::new(),
            states_left_to_the_cpu: 0,
            states_left_to_the_cpu_and_red_on_the_invariant: 0,
            disagreements: StatesDescribed::none(),
        }
    }

    pub(super) fn take_the_block(&mut self, block: Self) {
        self.states_counted += block.states_counted;
        let room = DESCRIBED_STATES.saturating_sub(self.first_ordinals_counted.len());
        self.first_ordinals_counted
            .extend(block.first_ordinals_counted.into_iter().take(room));
        self.states_left_to_the_cpu += block.states_left_to_the_cpu;
        self.states_left_to_the_cpu_and_red_on_the_invariant +=
            block.states_left_to_the_cpu_and_red_on_the_invariant;
        self.disagreements.take_the_block(block.disagreements);
    }
}

/// 一个状态上对拍两边交回的那几样（[`state_condition_holds`]、[`examine_the_state`] 读它）。
pub(super) struct StateOfThePairing<'state> {
    pub(super) ordinal: u64,
    pub(super) cpu: &'state JudgedOnCpu,
    /// CPU 那一侧在这张崩溃镜像上现算的逐盘自证过的系统配置槽数；没有一样考察要它时没算。
    pub(super) cpu_self_verified_slots: Option<&'state [(u32, u32)]>,
    pub(super) gpu_words: &'state [u32; VERDICT_WORDS],
}

/// 条件在这个状态上成不成立。
fn state_condition_holds(condition: &StateCondition, state: &StateOfThePairing) -> bool {
    let (
        _cpu_bits,
        cpu_checker_bits,
        cpu_red_items,
        _cpu_report,
        cpu_recovery,
        _cpu_checker_held_bits,
        cpu_consulted_reported_root,
        _cpu_cross_check_words,
        _cpu_red_items_compared_by_the_cross_check,
    ) = state.cpu;
    let gpu_words = state.gpu_words;
    match condition {
        StateCondition::CpuCheckerRedOnOneAndNotTheOther { red, not_red } => {
            cpu_checker_bits & invariant_bit(red) != 0
                && cpu_checker_bits & invariant_bit(not_red) == 0
        }
        StateCondition::GpuCheckerJudgedOrViolated(invariant) => {
            gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] == 0
                && (pool_checker_judged_bits(gpu_words) | pool_checker_violated_bits(gpu_words))
                    & invariant_bit(invariant)
                    != 0
        }
        StateCondition::CpuRedItemsContain(item) => cpu_red_items.contains(item.as_str()),
        StateCondition::CpuReportedRootIs(root) => cpu_consulted_reported_root == root,
        StateCondition::CpuConsultedPassLandsOn { version, class } => {
            consulted_landed_root_of(*cpu_recovery) == Some(*version)
                && cpu_recovery.consulted_pass_outcome == *class
        }
        StateCondition::CpuReportedRootOfTheInstanceWhileTheConsultedPassLandsElsewhere {
            instance,
        } => {
            let effective_root = consulted_landed_root_of(*cpu_recovery);
            cpu_consulted_reported_root.is_some_and(|(reported_instance, reported_txg)| {
                reported_instance == *instance
                    && effective_root.is_some_and(|effective_root| {
                        effective_root != (reported_instance, reported_txg)
                    })
            })
        }
        StateCondition::GpuScanRebuildEndsWith(code) => {
            gpu_words[SCAN_REBUILD_OUTCOME_WORD] == *code
        }
        StateCondition::GpuBothPassesFailAt(failure_point) => {
            gpu_words[CONSULTED_PASS_FAILURE_POINT_WORD] == *failure_point
                && gpu_words[IGNORED_PASS_FAILURE_POINT_WORD] == *failure_point
        }
        StateCondition::GpuEitherPassFailsAt(failure_point) => {
            gpu_words[CONSULTED_PASS_FAILURE_POINT_WORD] == *failure_point
                || gpu_words[IGNORED_PASS_FAILURE_POINT_WORD] == *failure_point
        }
        StateCondition::AllOf(conditions) => conditions
            .iter()
            .all(|each_condition| state_condition_holds(each_condition, state)),
    }
}

/// 一样考察在这个状态上看一遍，结果记进 `tally`。`devices` 是判表的盘号（解 GPU 交回的逐盘自证槽数用）。
// cpu-reference-for-gpu-comparison
pub(super) fn examine_the_state(
    examination: &StateExamination,
    state: &StateOfThePairing,
    devices: &[u32],
    tally: &mut ExaminationTally,
) {
    let ordinal = state.ordinal;
    let gpu_words = state.gpu_words;
    let cpu_checker_bits = state.cpu.1;
    let cpu_red_items_compared_by_the_cross_check = &state.cpu.8;
    match examination {
        StateExamination::CountStatesWhere(condition) => {
            if state_condition_holds(condition, state) {
                tally.states_counted += 1;
                if tally.first_ordinals_counted.len() < DESCRIBED_STATES {
                    tally.first_ordinals_counted.push(ordinal);
                }
            }
        }
        StateExamination::LeftToTheCpuOnlyFor {
            reasons_left_to_the_cpu,
            recovery_reasons_left_to_the_cpu,
            reasons_one_state_must_carry,
            invariant_red_on_a_state_left_to_the_cpu,
        } => {
            let reason = gpu_words[POOL_CHECKER_UNDECIDABLE_WORD];
            if reason == 0 {
                let gpu_checker_bits = pool_checker_violated_bits(gpu_words);
                if !pool_checker_bits_agree_in_the_cross_check(gpu_checker_bits, cpu_checker_bits) {
                    tally.disagreements.note(|| {
                        format!(
                            "状态 {ordinal}：GPU 池级 checker 判了，违例 {:?}；CPU 违例 {:?}",
                            invariant_names(gpu_checker_bits),
                            invariant_names(cpu_checker_bits)
                        )
                    });
                }
            } else if reasons_left_to_the_cpu.contains(&reason) {
                tally.states_left_to_the_cpu += 1;
                if reasons_one_state_must_carry.contains(&reason) {
                    tally.states_counted += 1;
                }
                if invariant_red_on_a_state_left_to_the_cpu
                    .is_some_and(|invariant| invariant_names(cpu_checker_bits).contains(&invariant))
                {
                    tally.states_left_to_the_cpu_and_red_on_the_invariant += 1;
                }
            } else {
                tally.disagreements.note(|| {
                    format!(
                        "状态 {ordinal}：GPU 池级 checker 判不了的原因字 {reason:#x} 不在 {reasons_left_to_the_cpu:#x?} 里"
                    )
                });
            }
            if let Some(disagreement) =
                recovery_kernel_left_the_state_to_the_cpu_for_an_unlisted_reason(
                    ordinal,
                    gpu_words,
                    recovery_reasons_left_to_the_cpu,
                )
            {
                tally.disagreements.note(|| disagreement);
            }
            // 内核不置位的那几条（第一版是 I-1.13）两边都去掉再比（`red_items_compared_by_the_cross_check`）
            let concluded = gpu_verdict_items(gpu_words).map_or_else(
                || cpu_red_items_compared_by_the_cross_check.clone(),
                |items| red_items_compared_by_the_cross_check(&items),
            );
            if concluded != *cpu_red_items_compared_by_the_cross_check {
                tally.disagreements.note(|| {
                    format!(
                        "状态 {ordinal}：判不了就回落 CPU 之后的红项 {concluded} 与 CPU 的 {cpu_red_items_compared_by_the_cross_check} 不同"
                    )
                });
            }
        }
        StateExamination::SelfVerifiedSystemConfigurationSlotsOfEachDevice {
            slots_on_the_base_image,
            device_newly_dead,
        } => {
            let on_the_cpu = state
                .cpu_self_verified_slots
                .expect("要逐盘自证槽数的考察在 CPU 那一侧每张崩溃镜像上都现算了它");
            let on_the_gpu =
                self_verified_system_configuration_slots_of_each_device_of_the_gpu_verdict(
                    gpu_words, devices,
                );
            let judged_from_the_gpu_counts =
                devices_without_a_self_verified_system_configuration_slot_against_the_base_image_from_the_slot_counts(
                    &on_the_gpu,
                    slots_on_the_base_image,
                );
            let judged_from_the_cpu_counts =
                devices_without_a_self_verified_system_configuration_slot_against_the_base_image_from_the_slot_counts(
                    on_the_cpu,
                    slots_on_the_base_image,
                );
            if on_the_cpu != on_the_gpu.as_slice()
                || judged_from_the_gpu_counts != judged_from_the_cpu_counts
            {
                tally.disagreements.note(|| {
                    format!(
                        "第 {ordinal} 个状态：CPU 逐盘 {on_the_cpu:?}、GPU 逐盘 {on_the_gpu:?}；判段拿 GPU 那一份 {judged_from_the_gpu_counts:?}、拿 CPU 参照那一份 {judged_from_the_cpu_counts:?}"
                    )
                });
            }
            if judged_from_the_gpu_counts
                == DevicesWithoutASelfVerifiedSystemConfigurationSlotAgainstTheBaseImage::New(vec![
                    *device_newly_dead,
                ])
            {
                tally.states_counted += 1;
            }
        }
    }
}
