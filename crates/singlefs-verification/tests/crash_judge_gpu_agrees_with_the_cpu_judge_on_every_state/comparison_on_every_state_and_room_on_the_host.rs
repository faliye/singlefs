//! 一条流逐状态对拍的入口（`Comparison` 与 `compare_on_every_state` 一族），对拍判器每次派发的状态数与主机上同时攥着的状态数。

use super::*;

/// 一条流（或它的一块）的对拍结论：只有计数、集合与带说明的前几个状态，不带逐状态的结论（整条流几亿个状态时装不下，
/// 一次约 300 GB 的分配会 SIGABRT）。要看逐状态结论的用例交 [`StateExamination`]，对拍时每个状态上看、按块累加进
/// [`Self::examined`]。
pub(super) struct Comparison {
    pub(super) states: u64,
    pub(super) cpu_red: u64,
    pub(super) gpu_red: u64,
    pub(super) cpu_checker_red: u64,
    pub(super) gpu_checker_red: u64,
    /// CPU 池级 checker 在这条流的哪几条不变量上红过。
    pub(super) cpu_checker_red_invariants: std::collections::BTreeSet<&'static str>,
    /// CPU 池级 checker 在这条流的哪几条不变量上判过而成立过（至少一个状态上不是报不适用）。
    pub(super) cpu_checker_held_invariants: std::collections::BTreeSet<&'static str>,
    /// CPU 在序号最大那个状态（全部写都落了）上的恢复摘要（[`recovery_words_of`]）；一个状态都没有是 `None`。
    pub(super) cpu_recovery_words_of_the_last_state: Option<RecoveryRootAndOutcomesOfState>,
    /// 看 journal 那一遍落到的根或两遍的结局类别与 CPU 不同的状态数（算进不一致）。
    pub(super) recovery_mismatches: u64,
    /// 对拍字（`CrossCheckWords`：红项、两遍各自落到的根、结局、失败点、池级 checker 判到位、判不了）与 CPU 不同的状态数；
    /// 不一致（`mismatches`）照它判（失败点这一版就比）。
    pub(super) cross_check_mismatches: u64,
    pub(super) mismatches: StatesDescribed,
    /// 只看池级 checker 的不一致：违例位与 CPU 不同、或判不了的状态。
    pub(super) checker_mismatches: StatesDescribed,
    /// 调用方交的每一样考察各一份结果，次序同交进来的。
    pub(super) examined: Vec<ExaminationTally>,
}

/// [`Comparison`] 写进对拍块结论的字节（小端；集合与表先写条数，串先写字节数）：逐字段按定义的次序。
impl Comparison {
    /// 这一块两边比下来一致不一致（对拍块结论照它记红绿，记着不一致的那一块下一趟现判）：不一致的状态（判定、池级 checker、恢复、对拍字）
    /// 一个都没有才算一致。
    pub(super) fn agreement_of_the_two_judges(&self) -> PairingAgreement {
        if self.mismatches.is_empty()
            && self.checker_mismatches.is_empty()
            && self.recovery_mismatches == 0
            && self.cross_check_mismatches == 0
        {
            PairingAgreement::BothSidesAgree
        } else {
            PairingAgreement::SidesDisagree
        }
    }

    /// 一个状态都还没收的总账：`examinations` 样考察各一份空结果。
    pub(super) fn of_no_state(examinations: usize) -> Self {
        Self {
            states: 0,
            cpu_red: 0,
            gpu_red: 0,
            cpu_checker_red: 0,
            gpu_checker_red: 0,
            cpu_checker_red_invariants: std::collections::BTreeSet::new(),
            cpu_checker_held_invariants: std::collections::BTreeSet::new(),
            cpu_recovery_words_of_the_last_state: None,
            recovery_mismatches: 0,
            cross_check_mismatches: 0,
            mismatches: StatesDescribed::none(),
            checker_mismatches: StatesDescribed::none(),
            examined: (0..examinations)
                .map(|_| ExaminationTally::none())
                .collect(),
        }
    }

    /// 接上紧挨着的后一块（块按序号次序接）：计数相加、集合并起来、带说明的接到满为止、最后一个状态的恢复摘要取后一块的。
    ///
    /// # Panics
    /// 两份的考察样数不同。
    pub(super) fn take_the_block(&mut self, block: Self) {
        assert_eq!(
            self.examined.len(),
            block.examined.len(),
            "同一条流的每一块考察的样数相同"
        );
        self.states += block.states;
        self.cpu_red += block.cpu_red;
        self.gpu_red += block.gpu_red;
        self.cpu_checker_red += block.cpu_checker_red;
        self.gpu_checker_red += block.gpu_checker_red;
        self.cpu_checker_red_invariants
            .extend(block.cpu_checker_red_invariants);
        self.cpu_checker_held_invariants
            .extend(block.cpu_checker_held_invariants);
        if block.cpu_recovery_words_of_the_last_state.is_some() {
            self.cpu_recovery_words_of_the_last_state = block.cpu_recovery_words_of_the_last_state;
        }
        self.recovery_mismatches += block.recovery_mismatches;
        self.cross_check_mismatches += block.cross_check_mismatches;
        self.mismatches.take_the_block(block.mismatches);
        self.checker_mismatches
            .take_the_block(block.checker_mismatches);
        for (tally, block_tally) in self.examined.iter_mut().zip(block.examined) {
            tally.take_the_block(block_tally);
        }
    }

    pub(super) fn to_bytes(&self) -> Vec<u8> {
        fn put_u64(bytes: &mut Vec<u8>, value: u64) {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        fn put_length(bytes: &mut Vec<u8>, length: usize) {
            put_u64(bytes, u64::try_from(length).expect("条数装得进 u64"));
        }
        fn put_text(bytes: &mut Vec<u8>, text: &str) {
            put_length(bytes, text.len());
            bytes.extend_from_slice(text.as_bytes());
        }
        fn put_states_described(bytes: &mut Vec<u8>, states: &StatesDescribed) {
            put_u64(bytes, states.count);
            put_length(bytes, states.described.len());
            for text in &states.described {
                put_text(bytes, text);
            }
        }
        let mut bytes = Vec::new();
        for count in [
            self.states,
            self.cpu_red,
            self.gpu_red,
            self.cpu_checker_red,
            self.gpu_checker_red,
            self.recovery_mismatches,
            self.cross_check_mismatches,
        ] {
            put_u64(&mut bytes, count);
        }
        for invariants in [
            &self.cpu_checker_red_invariants,
            &self.cpu_checker_held_invariants,
        ] {
            put_length(&mut bytes, invariants.len());
            for invariant in invariants {
                put_text(&mut bytes, invariant);
            }
        }
        match self.cpu_recovery_words_of_the_last_state {
            Some(summary) => {
                bytes.push(1);
                for word in summary.words() {
                    bytes.extend_from_slice(&word.to_le_bytes());
                }
            }
            None => bytes.push(0),
        }
        put_states_described(&mut bytes, &self.mismatches);
        put_states_described(&mut bytes, &self.checker_mismatches);
        put_length(&mut bytes, self.examined.len());
        for tally in &self.examined {
            put_u64(&mut bytes, tally.states_counted);
            put_length(&mut bytes, tally.first_ordinals_counted.len());
            for ordinal in &tally.first_ordinals_counted {
                put_u64(&mut bytes, *ordinal);
            }
            put_u64(&mut bytes, tally.states_left_to_the_cpu);
            put_u64(
                &mut bytes,
                tally.states_left_to_the_cpu_and_red_on_the_invariant,
            );
            put_states_described(&mut bytes, &tally.disagreements);
        }
        bytes
    }

    /// [`Self::to_bytes`] 的逆；字节不够、不变量名不在清单里、串不是 UTF-8、后面还剩字节都交 `None`。
    pub(super) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        struct Reader<'bytes> {
            rest: &'bytes [u8],
        }
        impl<'bytes> Reader<'bytes> {
            fn take(&mut self, count: usize) -> Option<&'bytes [u8]> {
                if self.rest.len() < count {
                    return None;
                }
                let (taken, rest) = self.rest.split_at(count);
                self.rest = rest;
                Some(taken)
            }
            fn u32(&mut self) -> Option<u32> {
                Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
            }
            fn u64(&mut self) -> Option<u64> {
                Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
            }
            fn length(&mut self) -> Option<usize> {
                usize::try_from(self.u64()?).ok()
            }
            fn text(&mut self) -> Option<String> {
                let length = self.length()?;
                String::from_utf8(self.take(length)?.to_vec()).ok()
            }
            fn invariants(&mut self) -> Option<std::collections::BTreeSet<&'static str>> {
                (0..self.length()?)
                    .map(|_| {
                        let name = self.text()?;
                        IMPLEMENTED_INVARIANTS
                            .iter()
                            .find(|invariant| **invariant == name)
                            .copied()
                    })
                    .collect()
            }
            fn states_described(&mut self) -> Option<StatesDescribed> {
                let count = self.u64()?;
                let described = (0..self.length()?)
                    .map(|_| self.text())
                    .collect::<Option<Vec<String>>>()?;
                Some(StatesDescribed { count, described })
            }
        }
        let mut reader = Reader { rest: bytes };
        let states = reader.u64()?;
        let cpu_red = reader.u64()?;
        let gpu_red = reader.u64()?;
        let cpu_checker_red = reader.u64()?;
        let gpu_checker_red = reader.u64()?;
        let recovery_mismatches = reader.u64()?;
        let cross_check_mismatches = reader.u64()?;
        let cpu_checker_red_invariants = reader.invariants()?;
        let cpu_checker_held_invariants = reader.invariants()?;
        let cpu_recovery_words_of_the_last_state = match reader.take(1)?[0] {
            0 => None,
            1 => {
                let mut words = [0u32; RECOVERY_ROOT_AND_OUTCOMES_WORDS];
                for word in &mut words {
                    *word = reader.u32()?;
                }
                Some(RecoveryRootAndOutcomesOfState::of_the_words(words))
            }
            _ => return None,
        };
        let mismatches = reader.states_described()?;
        let checker_mismatches = reader.states_described()?;
        let examined = (0..reader.length()?)
            .map(|_| {
                let states_counted = reader.u64()?;
                let first_ordinals_counted = (0..reader.length()?)
                    .map(|_| reader.u64())
                    .collect::<Option<Vec<u64>>>()?;
                let states_left_to_the_cpu = reader.u64()?;
                let states_left_to_the_cpu_and_red_on_the_invariant = reader.u64()?;
                let disagreements = reader.states_described()?;
                Some(ExaminationTally {
                    states_counted,
                    first_ordinals_counted,
                    states_left_to_the_cpu,
                    states_left_to_the_cpu_and_red_on_the_invariant,
                    disagreements,
                })
            })
            .collect::<Option<Vec<ExaminationTally>>>()?;
        if !reader.rest.is_empty() {
            return None;
        }
        Some(Self {
            states,
            cpu_red,
            gpu_red,
            cpu_checker_red,
            gpu_checker_red,
            cpu_checker_red_invariants,
            cpu_checker_held_invariants,
            cpu_recovery_words_of_the_last_state,
            recovery_mismatches,
            cross_check_mismatches,
            mismatches,
            checker_mismatches,
            examined,
        })
    }
}

/// 每段写数不超过 `limit` 的段全量展开（[`CrashPointReplaySegmentExpansion::EveryProperSubset`]），别的不展开。
pub(super) fn expansion_up_to(
    limit: usize,
) -> impl Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync {
    move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    }
}

pub(super) fn compare_on_every_state(
    cards: GpuCardPoolStartedAtTheFirstLease,
    flow: &RecordedFlow,
    limit: usize,
) -> Option<Comparison> {
    compare_on_every_state_examining(cards, flow, limit, &[])
}

/// 一趟对拍判一条流的哪些状态。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum StatesOfThePairing {
    EveryStateOfTheFlow,
    /// 状态序号落在这个半开区间里的（一条流状态多到一条用例判不完时，按序号区间拆成几条用例，每条判一段）。
    TheStatesWithOrdinalsIn(std::ops::Range<u64>),
}

/// 同 [`compare_on_every_state`]，只判状态序号落在 `ordinals` 里的那一段（[`StatesOfThePairing::TheStatesWithOrdinalsIn`]）：输入摘要与块键
/// 都与整条流那一趟相同，整条流那一趟与别的段判过的块照样复用；交回的总账只是这一段的状态。
pub(super) fn compare_on_every_state_with_ordinals_in(
    cards: GpuCardPoolStartedAtTheFirstLease,
    flow: &RecordedFlow,
    limit: usize,
    ordinals: std::ops::Range<u64>,
) -> Option<Comparison> {
    compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
        cards,
        flow,
        flow.name,
        &[],
        &flow.segments,
        &expansion_up_to(limit),
        ContainerCapacityKnobs::PRODUCT,
        &[],
        &StatesOfThePairing::TheStatesWithOrdinalsIn(ordinals),
    )
}

/// 同 [`compare_on_every_state`]，每个状态上另看 `examinations` 那几样（结果在 [`Comparison::examined`]，次序同交进来的）。
pub(super) fn compare_on_every_state_examining(
    cards: GpuCardPoolStartedAtTheFirstLease,
    flow: &RecordedFlow,
    limit: usize,
    examinations: &[StateExamination],
) -> Option<Comparison> {
    compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
        cards,
        flow,
        flow.name,
        &[],
        &flow.segments,
        &expansion_up_to(limit),
        ContainerCapacityKnobs::PRODUCT,
        examinations,
        &StatesOfThePairing::EveryStateOfTheFlow,
    )
}

/// 同 [`compare_on_every_state`]，切段与每段怎么展开由调用方给（`segments` 与 `flow.segments` 罩同一组写、只是分法不同时，
/// 状态仍是这条流全量枚举里的状态）。
/// 表建好之后、要现判的那一段块判之前才向卡池借卡，借的量是这条流的判器实际要的（[`mebibytes_a_judge_needs`]，一次派活的宽度照 [`states_per_dispatch_of_a_pairing_judge`]）；
/// GPU 判完那一段就还，CPU 那一侧判的时候不占着卡。建判器、判的时候卡上 wgpu 报了 Out of Memory：打「本次未跑」那一行（用例名那一格写流名）、
/// 交 `None`，调用的用例当场返回（[`on_the_card_or_report_not_run_when_out_of_memory`]）；这一族几个函数同此。
pub(super) fn compare_on_every_state_of_these_segments(
    cards: GpuCardPoolStartedAtTheFirstLease,
    flow: &RecordedFlow,
    segments: &[Vec<usize>],
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
) -> Option<Comparison> {
    compare_on_every_state_of_these_segments_against_the_version_table(
        cards,
        flow,
        &[],
        segments,
        expansion,
    )
}

/// 同 [`compare_on_every_state_of_these_segments`]，两类库里的项名由调用方给：同一条流按几种切法各比一次时每种切法一个项名——
/// 一个项名下的清单只认一份，撞了名判段会读到别的切法那一份（`crash_library` 的两段协议当场拒）。
pub(super) fn compare_on_every_state_of_these_segments_as_the_item(
    cards: GpuCardPoolStartedAtTheFirstLease,
    item_name: &str,
    flow: &RecordedFlow,
    segments: &[Vec<usize>],
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
) -> Option<Comparison> {
    compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
        cards,
        flow,
        item_name,
        &[],
        segments,
        expansion,
        ContainerCapacityKnobs::PRODUCT,
        &[],
        &StatesOfThePairing::EveryStateOfTheFlow,
    )
}

/// 同 [`compare_on_every_state_examining`]，版本表另带文件从那一版起没有了的那几版（删除流的意图那一版）：GPU 的期望版本表
/// （`crash_judge_tables::build_judge_tables_against_the_version_table`）与 CPU 的 oracle（`crash::CrashPointReplayVersionTable`）都认它们。
pub(super) fn compare_on_every_state_against_the_version_table(
    cards: GpuCardPoolStartedAtTheFirstLease,
    flow: &RecordedFlow,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    limit: usize,
    examinations: &[StateExamination],
) -> Option<Comparison> {
    compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
        cards,
        flow,
        flow.name,
        versions_without_a_file,
        &flow.segments,
        &expansion_up_to(limit),
        ContainerCapacityKnobs::PRODUCT,
        examinations,
        &StatesOfThePairing::EveryStateOfTheFlow,
    )
}

/// [`compare_on_every_state_of_these_segments`] 的本体，版本表照 `versions_without_a_file` 另带「没有文件的那几版」；三棵容器树都没装旋钮。
fn compare_on_every_state_of_these_segments_against_the_version_table(
    cards: GpuCardPoolStartedAtTheFirstLease,
    flow: &RecordedFlow,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    segments: &[Vec<usize>],
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
) -> Option<Comparison> {
    compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
        cards,
        flow,
        flow.name,
        versions_without_a_file,
        segments,
        expansion,
        ContainerCapacityKnobs::PRODUCT,
        &[],
        &StatesOfThePairing::EveryStateOfTheFlow,
    )
}

/// 同 [`compare_on_every_state`]，三棵容器树的旋钮值 `knobs` 编进 GPU 扫描内核（`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`，
/// G6），CPU 那一侧的池级 checker 违例位改按带同一组旋钮值的新入口（`check_pool_image_given_the_container_capacity_knobs`）在同一张崩溃镜像上判；
/// 红位与恢复字照旧按 CPU 判器（`judge_crash_image`，池级 checker 走产品入口的那一份不进比较）。
pub(super) fn compare_on_every_state_given_the_container_capacity_knobs(
    cards: GpuCardPoolStartedAtTheFirstLease,
    flow: &RecordedFlow,
    knobs: ContainerCapacityKnobs,
) -> Option<Comparison> {
    compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
        cards,
        flow,
        flow.name,
        &[],
        &flow.segments,
        &expansion_up_to(expand_up_to()),
        knobs,
        &[],
        &StatesOfThePairing::EveryStateOfTheFlow,
    )
}

/// 池级 checker 一份判定的违例位与判过而成立的位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）：经 `pool_checker_violated_and_judged_bits` 取。
pub(super) fn violated_and_held_bits_of(
    verdicts: &[(&'static str, InvariantVerdict)],
) -> (u128, u128) {
    let (violated, judged) = pool_checker_violated_and_judged_bits(verdicts);
    (violated, judged & !violated)
}

/// 一次交出去现判的至多几块（每块 [`STATES_PER_PAIRING_BLOCK`] 个状态）：CPU 那一侧一趟取样、GPU 那一侧一次判，判完逐块比完就丢。
/// 手里同一时刻压着的逐态结论只有这几块（CPU 那一侧每个状态一份 [`JudgedOnCpu`] 连同它的串、GPU 每个状态 [`VERDICT_WORDS`] 个字），
/// 与整条流有几块无关；取样一趟要从库里读回流、建一份撕裂表，几块攒一趟摊开它。
pub(super) const PAIRING_BLOCKS_JUDGED_TOGETHER: usize = 8;

/// 一张卡上同时放几个对拍判器：草稿区按卡的额度除以它开（[`states_per_dispatch_of_a_pairing_judge`]），借卡照账，这张卡放满才借下一张。
/// 不是 1：一个判器读回结论的时候卡上还有另一个判器的活，卡不空转（推的，`gpu_judge_throughput_by_states_per_dispatch` 量宽度，同时几个判器与吞吐的关系没量）；
/// 不是更大：判器多了提交互相排队，单次提交的用时撞 `LONGEST_DISPATCH_SECONDS` 就减半派活宽度（量的：六个判器挤一张卡时派活宽度只有 256 与 267 两档）。
pub(super) const PAIRING_JUDGES_SHARING_A_CARD: usize = 2;

/// 卡上每个判器的位子备几段块的状态压在宿主内存里（一段在卡上、一段 CPU 判好或正在判）：卡判完一段，下一段已经备好，卡不等 CPU；
/// CPU 判一个状态比卡快（量的：判 CPU 那一段约 14 千态每秒，一张卡约 7 千），不必再多备。
const PAIRING_SPANS_READY_FOR_A_JUDGE: usize = 2;

/// 一条流至多几段块一起判：卡上放得下几个判器就几段（一条流独占整个进程时照样用满分到的每张卡）。
pub(super) fn stretches_of_a_flow_judged_at_once_on_these_cards(
    card_count: usize,
) -> std::num::NonZeroUsize {
    std::num::NonZeroUsize::new(card_count * PAIRING_JUDGES_SHARING_A_CARD)
        .expect("池里至少一张卡：没有卡的机器上起不出池")
}

/// 这个进程至多几个状态的 CPU 结论同时压在宿主内存里（判 CPU 那一段之前排队，[`PairingStatesHeldOnTheHost`]）：每张卡上每个判器的位子备
/// [`PAIRING_SPANS_READY_FOR_A_JUDGE`] 段满 [`PAIRING_BLOCKS_JUDGED_TOGETHER`] 块的段。内存的上界是它乘每个状态的结论大小，加这些段各自读回的流与撕裂表。
pub(super) fn most_pairing_states_held_on_the_host_of(card_count: usize) -> u64 {
    let spans = card_count * PAIRING_JUDGES_SHARING_A_CARD * PAIRING_SPANS_READY_FOR_A_JUDGE;
    u64::try_from(spans * PAIRING_BLOCKS_JUDGED_TOGETHER).expect("装得进 u64")
        * STATES_PER_PAIRING_BLOCK
}

/// 宿主内存里压着几个状态的 CPU 结论这一本账：判一段块之前按它的状态数排队，判完比完（票丢掉）才还，放得下才判。
/// 按状态数不按段数：只有几百个状态的小流不占满一段的份额。这本账只等票回来，借卡排队另算（票在手里等卡时，卡上那几段都拿着票，判完就还，等得到头）。
pub(super) struct PairingStatesHeldOnTheHost {
    most_states: u64,
    pub(super) counts: std::sync::Mutex<PairingStatesHeldCounts>,
    a_ticket_came_back: std::sync::Condvar,
}

pub(super) struct PairingStatesHeldCounts {
    pub(super) states_held_now: u64,
    pub(super) most_states_held_so_far: u64,
}

impl PairingStatesHeldOnTheHost {
    pub(super) fn with_room_for_these_many_states(most_states: u64) -> Self {
        Self {
            most_states,
            counts: std::sync::Mutex::new(PairingStatesHeldCounts {
                states_held_now: 0,
                most_states_held_so_far: 0,
            }),
            a_ticket_came_back: std::sync::Condvar::new(),
        }
    }

    pub(super) fn pairing_states_counts_locked(
        &self,
    ) -> std::sync::MutexGuard<'_, PairingStatesHeldCounts> {
        // 持锁只为加减几个数，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
        self.counts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// 等到放得下 `states` 个状态、记上账，交回的票丢掉时还。账上一个都没有时比上限大的一段也放行（否则永远等不到）。
    pub(super) fn wait_for_room_and_hold_the_states_of_a_span(
        &self,
        states: u64,
    ) -> PairingStatesHeldTicket<'_> {
        let mut counts = self.pairing_states_counts_locked();
        // 迭代上界：每一轮等到一张票回来；票丢掉时都还（判的那一段 panic 了也还），等得到头
        while counts.states_held_now > 0 && counts.states_held_now + states > self.most_states {
            counts = self
                .a_ticket_came_back
                .wait(counts)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        counts.states_held_now += states;
        counts.most_states_held_so_far = counts.most_states_held_so_far.max(counts.states_held_now);
        PairingStatesHeldTicket {
            held_on: self,
            states,
        }
    }
}

/// 记在 [`PairingStatesHeldOnTheHost`] 账上的一段块的状态：丢掉时还、叫醒排队的。
pub(super) struct PairingStatesHeldTicket<'host> {
    held_on: &'host PairingStatesHeldOnTheHost,
    pub(super) states: u64,
}

impl Drop for PairingStatesHeldTicket<'_> {
    fn drop(&mut self) {
        self.held_on.pairing_states_counts_locked().states_held_now -= self.states;
        self.held_on.a_ticket_came_back.notify_all();
    }
}

/// 这个进程那一本账（卡池一个进程一份，账跟着它）。
pub(super) fn pairing_states_held_on_this_host(
    cards: GpuCardPoolStartedAtTheFirstLease,
) -> &'static PairingStatesHeldOnTheHost {
    static PAIRING_STATES_HELD: std::sync::OnceLock<PairingStatesHeldOnTheHost> =
        std::sync::OnceLock::new();
    PAIRING_STATES_HELD.get_or_init(|| {
        let most_states = most_pairing_states_held_on_the_host_of(pairing_cards_of_the_pool(cards));
        eprintln!("CRASH_JUDGE_GPU_PAIRING_HOST_QUEUE most_states_held={most_states}");
        PairingStatesHeldOnTheHost::with_room_for_these_many_states(most_states)
    })
}

/// 这个进程分到几张卡（卡池还没起的在这一刻起）。
pub(super) fn pairing_cards_of_the_pool(cards: GpuCardPoolStartedAtTheFirstLease) -> usize {
    cards.card_pool_started_now().card_count()
}

/// 对拍判器一次派活最多带几个状态（草稿区按它开、显存按它借，实际带几个判器在卡上现量）：一张卡空着时借得出的（`smallest_ceiling_mebibytes`，
/// 取池里最小的一张：借之前不知道借到哪张，每张都要放得下）平分给 [`PAIRING_JUDGES_SHARING_A_CARD`] 个判器，一个判器放得下几个状态（`mebibytes_for_states`）；
/// 放不下一条草稿缓冲的也开一条（比卡还大的由借卡那一步判红）；不超过 [`MAXIMUM_SCRATCH_LANES`] 条缓冲，不超过这一段的状态数。
/// 判器量宽度要草稿区至少两条缓冲（`GpuCrashJudge::judge_states` 里 `widest_to_try >= 2 * lane` 那一处），钉死一条缓冲时派活宽度只减不增。
pub(super) fn states_per_dispatch_of_a_pairing_judge(
    smallest_ceiling_mebibytes: u64,
    states_per_lane: usize,
    states_of_the_span: u64,
    mebibytes_for_states: &dyn Fn(usize) -> u64,
) -> usize {
    let judges_sharing_a_card = u64::try_from(PAIRING_JUDGES_SHARING_A_CARD).expect("装得进 u64");
    most_states_whose_need_fits_the_capacity(
        states_per_lane * MAXIMUM_SCRATCH_LANES,
        smallest_ceiling_mebibytes / judges_sharing_a_card,
        mebibytes_for_states,
    )
    .max(states_per_lane)
    .min(usize::try_from(states_of_the_span).unwrap_or(usize::MAX))
    .max(1)
}

/// 池里最小的一张卡空着时借得出多少（MiB）。
///
/// # Panics
/// 卡上的显存读不出（借卡那一步同样判红）。
pub(super) fn smallest_card_ceiling_of_the_pool_mebibytes(
    cards: GpuCardPoolStartedAtTheFirstLease,
) -> u64 {
    let pool = cards.card_pool_started_now();
    let card_indices: Vec<u32> = pool
        .cards_in_priority_order_as_configured()
        .iter()
        .map(|(card_index, _configured_quota_mebibytes)| *card_index)
        .collect();
    pool.card_rooms_now_and_when_empty_of(&card_indices)
        .unwrap_or_else(|reason| panic!("{reason}"))
        .iter()
        .map(|room| room.ceiling_mebibytes)
        .min()
        .expect("池里至少一张卡：没有卡的机器上起不出池")
}

/// 判 `states_of_the_span` 个状态的对拍判器在这个卡池上开多宽（[`states_per_dispatch_of_a_pairing_judge`]）。
pub(super) fn states_per_dispatch_of_a_pairing_judge_on_these_cards(
    cards: GpuCardPoolStartedAtTheFirstLease,
    packed: &PackedJudgeTablesReadBack,
    states_of_the_span: u64,
) -> usize {
    states_per_dispatch_of_a_pairing_judge(
        smallest_card_ceiling_of_the_pool_mebibytes(cards),
        states_of_one_scratch_lane_on_every_card_of(cards),
        states_of_the_span,
        &|states| mebibytes_a_judge_needs(packed, states),
    )
}
