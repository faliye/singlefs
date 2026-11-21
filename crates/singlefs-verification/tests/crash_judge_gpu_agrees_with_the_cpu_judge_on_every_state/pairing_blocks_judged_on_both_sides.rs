//! 对拍块在 CPU 判器与 GPU 判器上各判一遍、逐状态比对拍字，进度与驻留内存的报告。

use super::*;

/// [`compare_on_every_state_of_these_segments_against_the_version_table`] 的本体：`knobs` 是 [`ContainerCapacityKnobs::PRODUCT`] 时编判法摘要罩住的那三份内核、
/// CPU 池级 checker 取判器那一份（产品入口）；否则见 [`compare_on_every_state_given_the_container_capacity_knobs`]。
/// CPU 那一侧（带旋钮值重判池级 checker 的那一处在内）判的是从崩溃点重放枚举那一类的库里读回的流叠出来的镜像。
/// 对拍按块（`crash_library::pairing_conclusions_of_the_blocks_reused_or_judged`）：整条流按
/// [`STATES_PER_PAIRING_BLOCK`] 切块，块键是这一次对拍的输入摘要（[`pairing_inputs_digest`]，连同 `examinations`）拼上块的序号段，
/// 进库里配对结论那一张表（两边判器与比法的代码不进键）；库里有的块复用、一个状态都不再判；没有的攒成一段
/// （至多 [`PAIRING_BLOCKS_JUDGED_TOGETHER`] 块）两边都判、逐块比、逐块存进库（[`Comparison::to_bytes`]），比完就丢，逐块折进整条流的总账。
/// 宿主内存只随一段的块数涨，不随整条流的状态数涨。
/// 卡池借不到卡、卡上显存不够、只录那一趟（判那一侧报「本次未跑」交 `None` 的几种）：那一段一块都不存，照旧交 `None`。
/// `states` 是 [`StatesOfThePairing::TheStatesWithOrdinalsIn`] 时只走那一段序号的组（`crash_library::groups_of_pairing_blocks_holding_the_states`，
/// 区间两头要落在组的边界上，终点另可是整条流的状态数），输入摘要与块键都与整条流那一趟相同、块照样复用；交回的总账只是那一段的状态。
#[allow(
    clippy::too_many_arguments,
    reason = "对拍的输入就是这几样：卡池、流、项名、版本表另带的没有文件的几版、切段、展开、旋钮、逐状态另看的几样、判哪一段序号；收成一个类型只为让签名变短"
)]
// cpu-reference-for-gpu-comparison
pub(super) fn compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
    cards: GpuCardPoolStartedAtTheFirstLease,
    flow: &RecordedFlow,
    item_name: &str,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    segments: &[Vec<usize>],
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
    knobs: ContainerCapacityKnobs,
    examinations: &[StateExamination],
    states: &StatesOfThePairing,
) -> Option<Comparison> {
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion,
    };
    let tables_started = Instant::now();
    let tables = build_judge_tables_against_the_version_table(&judge_flow, versions_without_a_file)
        .expect("建得出表");
    let state_count =
        crash_point_replay_plan_state_count(&flow.base, &flow.writes, segments, expansion);
    assert_eq!(tables.state_count, state_count);
    let words = tables.words();
    eprintln!(
        "CRASH_JUDGE_TABLES flow={} states={state_count} locations={} version_bytes_words={} built_in={:?}",
        flow.name,
        tables.locations.len(),
        words.version_bytes.len(),
        tables_started.elapsed()
    );
    // 输入摘要只由这条流的输入数据与要比的那几项定：两边判器与比法的代码都不进键
    let mut inputs = pairing_inputs_digest(
        flow,
        versions_without_a_file,
        segments,
        expansion,
        knobs,
        &tables,
    )
    .to_vec();
    inputs.extend_from_slice(format!("examinations={examinations:?}\n").as_bytes());
    let inputs_digest = sha256_digest(&inputs);
    let library =
        CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("对拍那一类的库开得了");
    // 判表过库只在头一段要现判的块之前做一次（全部块都复用时不录、不读，整条流复用时同；头一段判完之前只有它一段）；只录那一趟它交 `None`，
    // 那一段照旧交给 CPU 那一侧把流也录上再收手（录入段里两类库都要写到）
    let packed: std::sync::OnceLock<Option<PackedJudgeTablesReadBack>> = std::sync::OnceLock::new();
    let mut comparison = Comparison::of_no_state(examinations.len());
    let started = Instant::now();
    let share_of_this_process =
        PairingBlocksShare::of_this_process().unwrap_or_else(|reason| panic!("{reason}"));
    let (share, states_judged_here) = match states {
        StatesOfThePairing::EveryStateOfTheFlow => (share_of_this_process, state_count),
        StatesOfThePairing::TheStatesWithOrdinalsIn(ordinals) => {
            let groups = groups_of_pairing_blocks_holding_the_states(
                ordinals,
                state_count,
                std::num::NonZeroU64::new(STATES_PER_PAIRING_BLOCK).expect("块宽不是 0"),
                std::num::NonZeroUsize::new(PAIRING_BLOCKS_JUDGED_TOGETHER)
                    .expect("一次判的块数不是 0"),
            )
            .unwrap_or_else(|refusal| panic!("流 {}：{refusal}", flow.name));
            eprintln!(
                "CRASH_JUDGE_GPU_ORDINALS flow={} states={state_count} ordinals={ordinals:?} groups={groups:?}",
                flow.name
            );
            (
                share_of_this_process.only_the_groups_of_pairing_blocks_in(groups),
                ordinals.end - ordinals.start,
            )
        }
    };
    let blocks_of_the_flow = pairing_conclusions_of_the_blocks_reused_or_judged(
        &library,
        &inputs_digest,
        state_count,
        std::num::NonZeroU64::new(STATES_PER_PAIRING_BLOCK).expect("块宽不是 0"),
        std::num::NonZeroUsize::new(PAIRING_BLOCKS_JUDGED_TOGETHER).expect("一次判的块数不是 0"),
        stretches_of_a_flow_judged_at_once_on_these_cards(pairing_cards_of_the_pool(cards)),
        &share,
        &|blocks: &[std::ops::Range<u64>]| {
            let packed =
                packed.get_or_init(|| packed_tables_through_the_library(item_name, &tables));
            Some(
                judge_the_blocks_on_the_cpu_and_the_gpu_and_compare(
                    cards,
                    flow,
                    item_name,
                    versions_without_a_file,
                    segments,
                    expansion,
                    knobs,
                    &tables,
                    packed.as_ref(),
                    examinations,
                    blocks,
                )?
                .iter()
                .map(|block_comparison| PairingJudged {
                    conclusion: block_comparison.to_bytes(),
                    agreement: block_comparison.agreement_of_the_two_judges(),
                })
                .collect(),
            )
        },
        &mut |_ordinals, conclusion| {
            comparison.take_the_block(
                Comparison::from_bytes(conclusion).expect("库里的对拍块结论是这份测试写的，认得回"),
            );
        },
    )
    .unwrap_or_else(|reason| panic!("流 {} 的对拍块结论读写库失败：{reason}", flow.name))?;
    if blocks_of_the_flow.blocks_left_to_other_shares > 0 {
        // 几台分着判的一份：自己领的块判完入库了，整条流的总账与用例钉的数归合并那一趟（整份，每块复用）
        eprintln!(
            "CRASH_JUDGE_GPU_MY_SHARE_ONLY flow={} states={state_count} blocks_reused={} blocks_judged_now={} blocks_left_to_other_shares={} elapsed={:?}",
            flow.name,
            blocks_of_the_flow.blocks_reused,
            blocks_of_the_flow.blocks_judged_now,
            blocks_of_the_flow.blocks_left_to_other_shares,
            started.elapsed()
        );
        return None;
    }
    assert_eq!(
        comparison.states, states_judged_here,
        "流 {}：各块的状态数合起来是这一趟判的状态数（整条流的，或点名那一段序号的）",
        flow.name
    );
    for line in comparison.mismatches.iter().take(3) {
        let shown: String = line.chars().take(700).collect();
        eprintln!("CRASH_JUDGE_GPU_MISMATCH flow={} {shown}", flow.name);
    }
    eprintln!(
        "CRASH_JUDGE_GPU flow={} states={} blocks_reused={} blocks_judged_now={} most_block_conclusions_held_at_once={} cpu_red={} gpu_red={} cpu_checker_red={} gpu_checker_red={} recovery_mismatches={} cross_check_mismatches={} mismatches={} checker_mismatches={} elapsed={:?}",
        flow.name,
        comparison.states,
        blocks_of_the_flow.blocks_reused,
        blocks_of_the_flow.blocks_judged_now,
        blocks_of_the_flow.most_block_conclusions_held_at_once,
        comparison.cpu_red,
        comparison.gpu_red,
        comparison.cpu_checker_red,
        comparison.gpu_checker_red,
        comparison.recovery_mismatches,
        comparison.cross_check_mismatches,
        comparison.mismatches.state_count(),
        comparison.checker_mismatches.state_count(),
        started.elapsed()
    );
    Some(comparison)
}

/// CPU 判器每个状态交回的那几样（红位、池级 checker 违例位、红项、恢复报告、恢复摘要、判过而成立的位、看 journal 那一遍报的根、对拍字、
/// 对拍比的那几条红项），按状态序号排好交给 [`compare_the_judgements_state_by_state`]。
pub(super) type JudgedOnCpu = (
    u32,
    u128,
    String,
    String,
    RecoveryRootAndOutcomesOfState,
    u128,
    Option<(u32, u64)>,
    CrossCheckWords,
    String,
);

/// CPU 那一侧在一张崩溃镜像上现算的逐盘自证过的系统配置槽数（有一样考察要它时才算，[`StateExamination::needs_the_self_verified_slots_on_the_cpu`]）。
pub(super) type SelfVerifiedSlotsOnTheCpu = Option<Vec<(u32, u32)>>;

/// 逐状态比一块（序号段 `ordinals`）里 CPU 与 GPU 的判定（对拍的比法）：红位、池级 checker 违例位与判到位、两遍恢复落到的根与结局、对拍字；
/// 不一致的状态逐个打一行、前 [`DESCRIBED_STATES`] 个带说明；每个状态上另看 `examinations` 那几样（`devices` 是判表的盘号）。
/// 交回这一块的结论，不带逐状态的结论。比法的代码不进对拍块的键（键只取输入摘要）：比法改了，库里早先的块结论照旧复用，
/// 比法对不对靠这条对拍自己与复用哨兵抓。
///
/// # Panics
/// CPU、GPU 两边的状态数与序号段的长度不同。
pub(super) fn compare_the_judgements_state_by_state(
    flow_name: &str,
    ordinals: std::ops::Range<u64>,
    cpu: &[(JudgedOnCpu, SelfVerifiedSlotsOnTheCpu)],
    gpu: &[[u32; VERDICT_WORDS]],
    examinations: &[StateExamination],
    devices: &[u32],
) -> Comparison {
    let states = ordinals.end - ordinals.start;
    assert_eq!(
        (
            u64::try_from(cpu.len()).expect("装得进 u64"),
            u64::try_from(gpu.len()).expect("装得进 u64")
        ),
        (states, states),
        "流 {flow_name} 序号段 {ordinals:?}：两边各判了这一段的每个状态"
    );
    let mut comparison = Comparison::of_no_state(examinations.len());
    comparison.states = states;
    comparison.cpu_recovery_words_of_the_last_state = cpu
        .last()
        .map(|((_, _, _, _, cpu_recovery, ..), _)| *cpu_recovery);
    for ((ordinal, (judged_on_cpu, cpu_self_verified_slots)), gpu_words) in
        ordinals.clone().zip(cpu).zip(gpu)
    {
        let (
            cpu_bits,
            cpu_checker_bits,
            cpu_items,
            cpu_report,
            cpu_recovery,
            cpu_checker_held_bits,
            _cpu_consulted_reported_root,
            cpu_cross_check_words,
            _cpu_items_compared_by_the_cross_check,
        ) = judged_on_cpu;
        comparison
            .cpu_checker_held_invariants
            .extend(invariant_names(*cpu_checker_held_bits));
        let gpu_bits = gpu_words[0] & !(1 << verdict_bits::UNDECIDABLE);
        let undecidable = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
            || gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] != 0;
        let gpu_checker_bits = pool_checker_violated_bits(gpu_words);
        if *cpu_bits != 0 {
            comparison.cpu_red += 1;
        }
        if gpu_bits != 0 {
            comparison.gpu_red += 1;
        }
        if *cpu_checker_bits != 0 {
            comparison.cpu_checker_red += 1;
            comparison
                .cpu_checker_red_invariants
                .extend(invariant_names(*cpu_checker_bits));
        }
        if gpu_checker_bits != 0 {
            comparison.gpu_checker_red += 1;
        }
        // 落到的根与两遍的结局也要相同：只比红位看不出「两遍恢复落到同一条根」这类漏洞（重放没扫环那一次就是这样漏的）
        let gpu_recovery = gpu_recovery_words(gpu_words);
        if gpu_recovery != *cpu_recovery {
            comparison.recovery_mismatches += 1;
        }
        let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
        // 内核不置位的那几条（第一版是 I-1.13）对拍不比：比较处两边都去掉再比，CPU 一侧照判定置位
        let cross_check_words_agree = cross_check_words_agree_in_the_cross_check(
            &gpu_cross_check_words,
            cpu_cross_check_words,
        );
        let checker_bits_agree =
            pool_checker_bits_agree_in_the_cross_check(gpu_checker_bits, *cpu_checker_bits);
        if !cross_check_words_agree {
            comparison.cross_check_mismatches += 1;
        }
        if undecidable
            || !checker_bits_agree
            || !pool_checker_bits_agree_in_the_cross_check(
                gpu_cross_check_words.pool_checker_judged_bits,
                cpu_cross_check_words.pool_checker_judged_bits,
            )
        {
            comparison.checker_mismatches.note(|| {
                format!(
                    "状态 {ordinal}：CPU checker={:?}；GPU checker={:?} 判不了的原因 {:#x}",
                    invariant_names(*cpu_checker_bits),
                    invariant_names(gpu_checker_bits),
                    gpu_words[POOL_CHECKER_UNDECIDABLE_WORD]
                )
            });
        }
        if undecidable
            || gpu_bits != *cpu_bits
            || !checker_bits_agree
            || gpu_recovery != *cpu_recovery
            || !cross_check_words_agree
        {
            // 每个不一致的状态打一行、不截（下面的说明只留前几个，CPU 那一半带读出的文件内容的字节数与摘要、GPU 的字排在后面）：GPU 结论字整份
            // （第 1 字是恢复内核判不了的原因、第 13 字是池级 checker 判不了的原因）、两侧恢复的四个字与对拍字，不带文件内容；只打、不进判定
            eprintln!(
                "CRASH_JUDGE_GPU_STATE flow={flow_name} state={ordinal} gpu_words={gpu_words:?} gpu_recovery={gpu_recovery:?} cpu_recovery={cpu_recovery:?} cpu_bits={cpu_bits:#x} gpu={gpu_cross_check_words:?} cpu={cpu_cross_check_words:?}"
            );
            comparison.mismatches.note(|| {
                format!(
                    "状态 {ordinal}：CPU {cpu_bits:#x} {cpu_items} {cpu_report} checker={:?} 判到={:?} 对拍字={cpu_cross_check_words:?}；GPU {gpu_words:?} checker={:?} 判到={:?} 对拍字={gpu_cross_check_words:?}",
                    invariant_names(*cpu_checker_bits),
                    invariant_names(cpu_cross_check_words.pool_checker_judged_bits),
                    invariant_names(gpu_checker_bits),
                    invariant_names(gpu_cross_check_words.pool_checker_judged_bits)
                )
            });
        }
        let state = StateOfThePairing {
            ordinal,
            cpu: judged_on_cpu,
            cpu_self_verified_slots: cpu_self_verified_slots.as_deref(),
            gpu_words,
        };
        for (examination, tally) in examinations.iter().zip(&mut comparison.examined) {
            examine_the_state(examination, &state, devices, tally);
        }
    }
    comparison
}

/// 一次对拍的输入摘要：流（基线镜像按扇区终值、每次写的盘、种类、偏移、FUA 与内容摘要、切段与每段的展开方式、被判的根、版本表与没有文件的那几版）、
/// 三棵容器树的旋钮值与判表的摘要。只按内容取，流名、项名不进。
fn pairing_inputs_digest(
    flow: &RecordedFlow,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    segments: &[Vec<usize>],
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
    knobs: ContainerCapacityKnobs,
    tables: &JudgeTables,
) -> [u8; DIGEST_BYTES] {
    let mut text = format!(
        "crash judge gpu and cpu pairing inputs\nbase={}\n",
        hexadecimal_text(&SectorImage::of_base(&flow.base).digest())
    );
    for write in &flow.writes {
        let contents = match &write.contents {
            WrittenContents::Bytes(bytes) => hexadecimal_text(&sha256_digest(bytes)),
            WrittenContents::Zeros { length } => format!("zeros:{length}"),
        };
        text.push_str(&format!(
            "write device={} kind={:?} offset={} force_unit_access={} contents={contents}\n",
            write.device.0, write.kind, write.offset.0, write.is_force_unit_access
        ));
    }
    for (segment_index, segment) in segments.iter().enumerate() {
        text.push_str(&format!(
            "segment {segment:?} expansion={}\n",
            expansion(segment_index, segment).name()
        ));
    }
    text.push_str(&format!(
        "judged_root_index={}\nversions={:?}\nversions_without_a_file={versions_without_a_file:?}\nknobs={knobs:?}\ntables={}\n",
        flow.judged_root_index,
        flow.versions,
        hexadecimal_text(&tables.digest())
    ));
    sha256_digest(text.as_bytes())
}

/// 对拍里 CPU 那一侧：崩溃点重放的 CPU 判器先把流入库（崩溃点重放枚举那一类的库，项名 `item_name`）、再从库里读回判序号段 `ordinals` 的状态，
/// 按线程分段判（`SINGLEFS_CRASH_JUDGE_CPU_THREADS`，没设取每台默认重活线程数），各线程按序号各报各的，合并按序号排：交回每个状态判出的那几样
/// （[`JudgedOnCpu`]，`knobs` 不是 [`ContainerCapacityKnobs::PRODUCT`] 时池级 checker 那几位按带同一组旋钮值的入口在同一张崩溃镜像上重判）与
/// `also_of_the_state` 从同一张崩溃镜像、同一份判定里另取的那一样。手里只压着这一段的逐态结论。只录那一趟交 `None`。
///
/// # Panics
/// 判了的状态数不是序号段的长度；整条流的状态数不是 `state_count`。
#[allow(
    clippy::too_many_arguments,
    reason = "CPU 那一侧的输入就是这几样：流、项名、版本表另带的没有文件的几版、切段、展开、旋钮、整条流的状态数、序号段，加另取的那一样；收成一个类型只为让签名变短"
)]
// cpu-reference-for-gpu-comparison
pub(super) fn every_state_judged_on_the_cpu<AlsoOfTheState: Send>(
    flow: &RecordedFlow,
    item_name: &str,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    segments: &[Vec<usize>],
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
    knobs: ContainerCapacityKnobs,
    state_count: u64,
    ordinals: std::ops::Range<u64>,
    also_of_the_state: &(dyn Fn(&CrashImage<'_>, &CrashStateJudgement) -> AlsoOfTheState + Sync),
) -> Option<Vec<(JudgedOnCpu, AlsoOfTheState)>> {
    let threads = cpu_threads_from_the_value_of_the_variable(
        process_environment_variable(CPU_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
    );
    let states_of_the_range = ordinals.end - ordinals.start;
    let judged_by_ordinal: std::sync::Mutex<Vec<(u64, JudgedOnCpu, AlsoOfTheState)>> =
        std::sync::Mutex::new(Vec::with_capacity(
            usize::try_from(states_of_the_range).expect("一段的状态数装得进 usize"),
        ));
    let cpu_judged = sample_crash_states_on_the_cpu_with_threads_against_the_version_table(
        CrashPointReplayItem {
            name: item_name,
            setting_file: PATH_OF_THIS_TEST_FILE,
        },
        &flow.base,
        &flow.writes,
        segments,
        flow.judged_root_index,
        CrashPointReplayVersionTable::new(&flow.versions, versions_without_a_file),
        expansion,
        ordinals,
        threads,
        &|ordinal, image, judgement| {
            let mut cross_check_words = cross_check_words_of_cpu_judgement(judgement);
            let (pool_checker_bits, pool_checker_held_bits) =
                if knobs == ContainerCapacityKnobs::PRODUCT {
                    (
                        expected_pool_checker_bits(judgement),
                        expected_pool_checker_held_bits(judgement),
                    )
                } else {
                    // 旋钮值编进了 GPU 扫描内核：CPU 那一侧池级 checker 的违例位与判到位都按带同一组旋钮值的入口重判
                    let verdicts_given_the_knobs =
                        check_pool_image_given_the_container_capacity_knobs(image, knobs);
                    let (violated_given_the_knobs, judged_given_the_knobs) =
                        pool_checker_violated_and_judged_bits(&verdicts_given_the_knobs);
                    cross_check_words.pool_checker_violated_bits = violated_given_the_knobs;
                    cross_check_words.pool_checker_judged_bits = judged_given_the_knobs;
                    violated_and_held_bits_of(&verdicts_given_the_knobs)
                };
            judged_by_ordinal.lock().expect("判定线程不 panic").push((
                ordinal,
                (
                    expected_bits(judgement),
                    pool_checker_bits,
                    format!("{:?}", judgement.red_items()),
                    format!(
                        "consulted={} effective={:?} ignored={}",
                        recovery_outcome_without_the_file_content(&judgement.consulted.outcome),
                        judgement.consulted.effective_root,
                        recovery_outcome_without_the_file_content(&judgement.ignored.outcome)
                    ),
                    recovery_words_of(judgement),
                    pool_checker_held_bits,
                    root_reported_by_the_recovery(&judgement.consulted.outcome),
                    cross_check_words,
                    red_items_compared_by_the_cross_check(&judgement.red_items()),
                ),
                also_of_the_state(image, judgement),
            ));
        },
    );
    let states_of_the_whole_flow = cpu_judged.judged()?;
    assert_eq!(states_of_the_whole_flow, state_count);
    let mut judged_by_ordinal = judged_by_ordinal.into_inner().expect("判定线程不 panic");
    assert_eq!(
        u64::try_from(judged_by_ordinal.len()).expect("装得进 u64"),
        states_of_the_range,
        "这一段的每个状态各判了一次"
    );
    judged_by_ordinal.sort_by_key(|(ordinal, _, _)| *ordinal);
    Some(
        judged_by_ordinal
            .into_iter()
            .map(|(_, judged, also)| (judged, also))
            .collect(),
    )
}

/// 对拍一段连着的块：CPU 判器从库里读回的流把这一段的状态逐个判、GPU 判器从库里读回的判表（`packed`）把同一段判，逐块逐状态比，
/// 一块交回一份结论（[`compare_the_judgements_state_by_state`]）。这一段判完就还卡、逐态结论随返回丢掉。判那一侧报「本次未跑」时交 `None`：
/// 只录那一趟判表过库交回的是 `None`（`packed` 没有），CPU 那一侧照样先走一遍把流录上，再收手。
#[allow(
    clippy::too_many_arguments,
    reason = "对拍一段块的输入就是这几样：卡池、流、项名、版本表另带的没有文件的几版、切段、展开、旋钮、判表与读回的打包判表、逐状态另看的几样、这一段的块；收成一个类型只为让签名变短"
)]
fn judge_the_blocks_on_the_cpu_and_the_gpu_and_compare(
    cards: GpuCardPoolStartedAtTheFirstLease,
    flow: &RecordedFlow,
    item_name: &str,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    segments: &[Vec<usize>],
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
    knobs: ContainerCapacityKnobs,
    tables: &JudgeTables,
    packed: Option<&PackedJudgeTablesReadBack>,
    examinations: &[StateExamination],
    blocks: &[std::ops::Range<u64>],
) -> Option<Vec<Comparison>> {
    let (Some(first_block), Some(last_block)) = (blocks.first(), blocks.last()) else {
        return Some(Vec::new());
    };
    let span = first_block.start..last_block.end;
    let progress = PairingProgressOfTheSpan {
        cpu_states_judged: AtomicU64::new(0),
        gpu_states_judged: AtomicU64::new(0),
    };
    let (stop_the_progress_lines, progress_lines_stopped) = mpsc::channel::<()>();
    let started = Instant::now();
    let progress_of_the_span = &progress;
    let span_reported = &span;
    let flow_name = flow.name;
    let block_count = blocks.len();
    std::thread::scope(|scope| {
        let progress_lines = scope.spawn(move || {
            report_the_pairing_progress_until_stopped(
                flow_name,
                span_reported,
                block_count,
                progress_of_the_span,
                &progress_lines_stopped,
                started,
            );
        });
        let comparisons = judge_the_blocks_on_the_cpu_and_the_gpu_and_compare_counting_into(
            cards,
            flow,
            item_name,
            versions_without_a_file,
            segments,
            expansion,
            knobs,
            tables,
            packed,
            examinations,
            blocks,
            progress_of_the_span,
        );
        // 这一段判完（交回或报了本次未跑）：放下发的一头，进度那条线程收到断开就收手；这一段中途 panic 时它随这个闭包一起丢掉，同样放下
        drop(stop_the_progress_lines);
        progress_lines.join().expect("进度那条线程不 panic");
        comparisons
    })
}

/// 对拍一段块时两边各判了几个状态：CPU 那一侧每判一个状态加一，GPU 那一侧每判完一块加这一块的状态数（[`report_the_pairing_progress_until_stopped`] 读它）。
struct PairingProgressOfTheSpan {
    cpu_states_judged: AtomicU64,
    gpu_states_judged: AtomicU64,
}

/// 对拍一段块的进度行（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」：单项跑得超过 5 分钟的，每过
/// [`PROGRESS_INTERVAL_OF_THE_SLICES`] 至少打一行，报区间与已判的数）：这一段的序号区间与块数、CPU 那一侧与 GPU 那一侧各判了几个状态、
/// 用了多久、这个进程此刻与到此刻为止的常驻内存（量内存花在哪一段用）。`stopped` 那一头放下（或发了一个）就收手。
fn report_the_pairing_progress_until_stopped(
    flow_name: &str,
    span: &std::ops::Range<u64>,
    blocks: usize,
    progress: &PairingProgressOfTheSpan,
    stopped: &mpsc::Receiver<()>,
    started: Instant,
) {
    // 迭代上界：这一段判完为止，每过一个间隔一轮；跨轮不带状态，每轮现读两边的数
    loop {
        match stopped.recv_timeout(PROGRESS_INTERVAL_OF_THE_SLICES) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        let (resident_mebibytes, peak_resident_mebibytes) =
            resident_and_peak_mebibytes_of_this_process();
        eprintln!(
            "CRASH_JUDGE_GPU_PAIRING_PROGRESS flow={flow_name} states=[{},{}) blocks={blocks} cpu_judged={} gpu_judged={} elapsed_seconds={:.1} resident_mebibytes={resident_mebibytes} peak_resident_mebibytes={peak_resident_mebibytes}",
            span.start,
            span.end,
            progress.cpu_states_judged.load(Ordering::Relaxed),
            progress.gpu_states_judged.load(Ordering::Relaxed),
            started.elapsed().as_secs_f64()
        );
    }
}

/// 这个进程此刻与到此刻为止的常驻内存（MiB）：`/proc/self/status` 的 VmRSS 与 VmHWM。只进进度行、不进判定；读不出的那一样写「读不出」。
fn resident_and_peak_mebibytes_of_this_process() -> (String, String) {
    let unreadable = || "读不出".to_string();
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return (unreadable(), unreadable());
    };
    let mebibytes_of = |field: &str| {
        status
            .lines()
            .find_map(|line| line.strip_prefix(field))
            .and_then(|rest| rest.trim().strip_suffix("kB"))
            .and_then(|kibibytes| kibibytes.trim().parse::<u64>().ok())
            .map_or_else(unreadable, |kibibytes| (kibibytes / 1024).to_string())
    };
    (mebibytes_of("VmRSS:"), mebibytes_of("VmHWM:"))
}

/// [`judge_the_blocks_on_the_cpu_and_the_gpu_and_compare`] 的本体：两边各判到几个状态记进 `progress`（GPU 那一侧按块判，判完一块记一块）。
#[allow(
    clippy::too_many_arguments,
    reason = "对拍一段块的输入就是这几样：卡池、流、项名、版本表另带的没有文件的几版、切段、展开、旋钮、判表与读回的打包判表、逐状态另看的几样、这一段的块，加记进度的那一份；收成一个类型只为让签名变短"
)]
// cpu-reference-for-gpu-comparison
fn judge_the_blocks_on_the_cpu_and_the_gpu_and_compare_counting_into(
    cards: GpuCardPoolStartedAtTheFirstLease,
    flow: &RecordedFlow,
    item_name: &str,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    segments: &[Vec<usize>],
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
    knobs: ContainerCapacityKnobs,
    tables: &JudgeTables,
    packed: Option<&PackedJudgeTablesReadBack>,
    examinations: &[StateExamination],
    blocks: &[std::ops::Range<u64>],
    progress: &PairingProgressOfTheSpan,
) -> Option<Vec<Comparison>> {
    let (Some(first_block), Some(last_block)) = (blocks.first(), blocks.last()) else {
        return Some(Vec::new());
    };
    let span = first_block.start..last_block.end;
    // 判 CPU 那一段之前先排队，判完比完才还：CPU 判完的结论压在内存里等卡的不超过这本账的上限
    let _states_held_on_the_host = pairing_states_held_on_this_host(cards)
        .wait_for_room_and_hold_the_states_of_a_span(span.end - span.start);
    let kernel_sources = JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs(knobs);
    let self_verified_slots_are_examined = examinations
        .iter()
        .any(StateExamination::needs_the_self_verified_slots_on_the_cpu);
    let cpu_started = Instant::now();
    let cpu = every_state_judged_on_the_cpu(
        flow,
        item_name,
        versions_without_a_file,
        segments,
        expansion,
        knobs,
        tables.state_count,
        span.clone(),
        &|image, _judgement| {
            progress.cpu_states_judged.fetch_add(1, Ordering::Relaxed);
            self_verified_slots_are_examined
                .then(|| self_verified_system_configuration_slots_of_each_device(image))
        },
    )?;
    let cpu_elapsed = cpu_started.elapsed();
    let packed = packed?;
    let states_per_dispatch =
        states_per_dispatch_of_a_pairing_judge_on_these_cards(cards, packed, span.end - span.start);
    let lease = cards.lease_card_or_report_not_run(
        flow.name,
        mebibytes_a_judge_needs(packed, states_per_dispatch),
    )?;
    let gpu_started = Instant::now();
    let gpu = on_the_card_or_report_not_run_when_out_of_memory(
        flow.name,
        lease.card(),
        "建判器与判这条流的一段块",
        || {
            let judge = GpuCrashJudge::on_card_compiling_these_kernel_sources(
                lease.card(),
                packed,
                states_per_dispatch,
                &kernel_sources,
            );
            // 按块判、判完一块记一块（进度行读它）：块首尾相接罩住这一段，拼起来与整段一次判同序、逐状态同一份结论
            let mut verdicts =
                Vec::with_capacity(usize::try_from(span.end - span.start).expect("装得进"));
            // 迭代上界是这一段的块数；跨轮携带的只有拼起来的结论
            for block in blocks {
                verdicts.extend(judge.judge_states(block.clone()));
                progress
                    .gpu_states_judged
                    .fetch_add(block.end - block.start, Ordering::Relaxed);
            }
            verdicts
        },
    )?;
    let gpu_elapsed = gpu_started.elapsed();
    drop(lease);
    assert_eq!(
        (cpu.len(), gpu.len()),
        (
            usize::try_from(span.end - span.start).expect("装得进"),
            usize::try_from(span.end - span.start).expect("装得进")
        ),
        "流 {}：两边各判了序号段 {span:?} 的每个状态",
        flow.name
    );
    let comparisons: Vec<Comparison> = blocks
        .iter()
        .map(|block| {
            let within_the_span = usize::try_from(block.start - span.start).expect("装得进")
                ..usize::try_from(block.end - span.start).expect("装得进");
            compare_the_judgements_state_by_state(
                flow.name,
                block.clone(),
                &cpu[within_the_span.clone()],
                &gpu[within_the_span],
                examinations,
                &tables.devices,
            )
        })
        .collect();
    eprintln!(
        "CRASH_JUDGE_GPU_BLOCKS_JUDGED flow={} first_state={} states={} blocks={} gpu_elapsed={gpu_elapsed:?} cpu_elapsed={cpu_elapsed:?}",
        flow.name,
        span.start,
        span.end - span.start,
        blocks.len()
    );
    Some(comparisons)
}

/// 新池新建文件、固定脚本到回退与它们改坏一处的样子那几条流上 CPU 要显出的红（[`assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of`]）。
pub(super) enum RedTheCpuShowsOnTheFlow {
    /// 合法流：不要求哪一样红。
    NoneRequired,
    /// CPU 判器（两遍 oracle 与记录核对器）在至少一个状态上红。
    TheCpuJudge,
    /// CPU 池级 checker 在至少一个状态上红。
    ThePoolChecker,
    /// CPU 判器与 CPU 池级 checker 都在至少一个状态上红。
    TheCpuJudgeAndThePoolChecker,
    /// CPU 池级 checker 红过这一条。
    ThePoolCheckerOn(&'static str),
}

/// 这几条流（一条流一个用例，按流拆）共用的判法：录下流与它上面 CPU 要显出的红
/// （`record_the_flow`，取卡之后才录，没卡的机器上不录）、全量域每个状态对拍（[`compare_on_every_state`]，展开上限照 [`expand_up_to`]），
/// CPU 显出那样的红，每个状态 GPU 与 CPU 整份相同。本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
pub(super) fn assert_the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of(
    test_name: &str,
    record_the_flow: &dyn Fn() -> (RecordedFlow, RedTheCpuShowsOnTheFlow),
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let (flow, red_the_cpu_shows) = record_the_flow();
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    match red_the_cpu_shows {
        RedTheCpuShowsOnTheFlow::NoneRequired => {}
        RedTheCpuShowsOnTheFlow::TheCpuJudge => {
            assert!(comparison.cpu_red > 0, "流 {} 上 CPU 判器要有红", flow.name);
        }
        RedTheCpuShowsOnTheFlow::ThePoolChecker => {
            assert!(
                comparison.cpu_checker_red > 0,
                "流 {} 上 CPU 池级 checker 要有红",
                flow.name
            );
        }
        RedTheCpuShowsOnTheFlow::TheCpuJudgeAndThePoolChecker => {
            assert!(
                comparison.cpu_checker_red > 0,
                "流 {} 上 CPU 池级 checker 要有红",
                flow.name
            );
            assert!(
                comparison.cpu_red > 0,
                "流 {} 上两遍恢复要报没择到根（CPU 判器要有红）",
                flow.name
            );
        }
        RedTheCpuShowsOnTheFlow::ThePoolCheckerOn(invariant) => {
            assert!(
                comparison.cpu_checker_red_invariants.contains(invariant),
                "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
                flow.name,
                comparison.cpu_checker_red_invariants
            );
        }
    }
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}
