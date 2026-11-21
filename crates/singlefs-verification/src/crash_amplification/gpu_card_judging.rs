//! 在卡上判（只有直接调 [`judge_my_blocks_on_gpu_cards`] 的对拍用例走）：判器交回的判定字与恢复字怎么解、几张卡怎么领块、
//! 显存不够怎么停一张卡、交回的块怎么照块序入库，连同 GPU 那一趟的墙钟行与逐卡的过数行。

use super::*;

/// GPU 判器判这一趟时等着收的那条线程隔一个进度间隔打的那一行：写进库几块（共几块）、几块判完了在等前面的块、几张卡收工了、开跑多久。
#[must_use]
pub fn gpu_judge_wall_clock_line(
    blocks_stored: usize,
    pending_blocks: usize,
    blocks_waiting_for_an_earlier_block: usize,
    cards_finished: usize,
    elapsed: std::time::Duration,
) -> String {
    format!(
        "CRASH_AMPLIFICATION_BLOCK by=wall_clock judge=gpu blocks_stored={blocks_stored}/{pending_blocks} blocks_waiting_for_an_earlier_block={blocks_waiting_for_an_earlier_block} cards_finished={cards_finished} elapsed_seconds={:.1}",
        elapsed.as_secs_f64()
    )
}

// ============================== 第 ① 段的 GPU 形态：GPU 判器判块，卡按优先级领活 ==============================

/// [`RecoveryRootAndOutcomesOfState::words`] 摊出几个字：落到的根（[`LANDED_ROOT_WORDS`] 个字）与两遍的走读结局码各一个字。
pub const RECOVERY_ROOT_AND_OUTCOMES_WORDS: usize = LANDED_ROOT_WORDS + 2;

/// 一个状态上 CPU 与 GPU 两侧逐状态比的恢复摘要：看 journal 那一遍落到的根（[`LandedRootWords`]：落没落到根的标记、实例代号、
/// checkpoint_txg 低 32 位与高 32 位 checkpoint_txg「存完整 u64」，`records/2026-10-24-singlefs-checker-tier-审计复核.md` 的 T01），
/// 两遍的走读结局码（`RECOVERY_OUTCOME_CODE_*`：没有文件、读到文件、失败）。CPU 一侧由 [`recovery_words_of`]、GPU 一侧由
/// `gpu_recovery_words`（只开 `gpu` 时编）换出来，两份逐字段比。
/// 红项相同而落到的根不同也是判错（判器读错了一个落点的版本、两个版本都判绿时就只露在这里），对拍要连它一起比。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecoveryRootAndOutcomesOfState {
    pub consulted_pass_landed_root: LandedRootWords,
    pub consulted_pass_outcome: u32,
    pub ignored_pass_outcome: u32,
}

/// GPU 一个状态的结论字换成恢复摘要（与 CPU 一侧 [`recovery_words_of`] 同一个类型）：看 journal 那一遍落到的根取标记、实例代号、
/// checkpoint_txg 低 32 位与高 32 位那四个字（高 32 位与标记是接在结论末尾的那两个字），两遍的走读结局码各取各的字。字号在判器一侧
/// （`crate::crash_judge_gpu` 的 `CONSULTED_PASS_*`、`IGNORED_PASS_*`），换法写在流水线这一侧（见本文件 `gpu_verdict_items` 转出那一处的说明）。
#[cfg(feature = "gpu")]
#[must_use]
pub fn gpu_recovery_words(
    words: &[u32; crate::crash_judge_gpu::VERDICT_WORDS],
) -> RecoveryRootAndOutcomesOfState {
    let consulted_pass_landed_root = LandedRootWords {
        landed_marker: words[CONSULTED_PASS_LANDED_MARKER_WORD],
        instance: words[CONSULTED_PASS_LANDED_INSTANCE_WORD],
        checkpoint_txg_low: words[CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD],
        checkpoint_txg_high: words[CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD],
    };
    let consulted_pass_outcome = words[CONSULTED_PASS_OUTCOME_WORD];
    let ignored_pass_outcome = words[IGNORED_PASS_OUTCOME_WORD];
    RecoveryRootAndOutcomesOfState {
        consulted_pass_landed_root,
        consulted_pass_outcome,
        ignored_pass_outcome,
    }
}

impl RecoveryRootAndOutcomesOfState {
    /// 摊成字：落到的根那四个字（[`LandedRootWords::words`] 的次序）、看 journal 那一遍的结局码、不看 journal 那一遍的结局码
    /// （对拍用例把整份比较结果存进判定库时照这个次序写）。
    #[must_use]
    pub fn words(self) -> [u32; RECOVERY_ROOT_AND_OUTCOMES_WORDS] {
        let landed_root = self.consulted_pass_landed_root.words();
        std::array::from_fn(|position| {
            if position < LANDED_ROOT_WORDS {
                landed_root[position]
            } else if position == LANDED_ROOT_WORDS {
                self.consulted_pass_outcome
            } else {
                self.ignored_pass_outcome
            }
        })
    }

    /// [`Self::words`] 的逆。
    #[must_use]
    pub fn of_the_words(words: [u32; RECOVERY_ROOT_AND_OUTCOMES_WORDS]) -> Self {
        Self {
            consulted_pass_landed_root: LandedRootWords::of_the_words(std::array::from_fn(
                |position| words[position],
            )),
            consulted_pass_outcome: words[LANDED_ROOT_WORDS],
            ignored_pass_outcome: words[LANDED_ROOT_WORDS + 1],
        }
    }
}

/// CPU 两遍恢复换成恢复摘要（[`RecoveryRootAndOutcomesOfState`]）：落到的根取 `effective_root`（施加 journal 之后实际走的那条根），
/// 换法与对拍字同一份（[`LandedRootWords::of_the_landed_root`]，没落到根是 [`LandedRootWords::NOT_LANDED`]）。
#[must_use]
pub fn recovery_words_of(
    judgement: &crate::crash::CrashStateJudgement,
) -> RecoveryRootAndOutcomesOfState {
    let outcome_code = |outcome: &RecoveryOutcome| match outcome {
        RecoveryOutcome::NoFile { .. } => RECOVERY_OUTCOME_CODE_NO_FILE,
        RecoveryOutcome::FileRead { .. } => RECOVERY_OUTCOME_CODE_FILE_READ,
        RecoveryOutcome::Failed { .. } => RECOVERY_OUTCOME_CODE_FAILED,
    };
    RecoveryRootAndOutcomesOfState {
        consulted_pass_landed_root: judgement.consulted.effective_root.map_or(
            LandedRootWords::NOT_LANDED,
            |(instance, checkpoint_txg)| {
                LandedRootWords::of_the_landed_root(instance, checkpoint_txg)
            },
        ),
        consulted_pass_outcome: outcome_code(&judgement.consulted.outcome),
        ignored_pass_outcome: outcome_code(&judgement.ignored.outcome),
    }
}

/// 一块的结论字里第一个 GPU 判不了的状态（块内序号）：恢复内核带「判不了」位、或 checker 内核的「判不了」字不为 0。没有就是 `None`。
#[cfg(feature = "gpu")]
#[must_use]
pub fn first_state_the_gpu_could_not_judge(
    words: &[[u32; crate::crash_judge_gpu::VERDICT_WORDS]],
) -> Option<usize> {
    use crate::crash_judge_gpu::{verdict_bits, POOL_CHECKER_UNDECIDABLE_WORD};
    words.iter().position(|state_words| {
        state_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
            || state_words[POOL_CHECKER_UNDECIDABLE_WORD] != 0
    })
}

/// 一张卡在一趟里的账：判了多少、多快、一次派活带几个状态、占多少显存、挂钟花在哪。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GpuCardOutcome {
    pub name: String,
    /// `nvidia-smi` 给这张卡的序号。
    pub index: u32,
    pub memory_budget_mebibytes: u64,
    pub states_per_dispatch: u64,
    pub scratch_mebibytes: u64,
    pub blocks_judged: u64,
    pub states_judged: u64,
    /// 表上卡与编内核的挂钟。
    pub build_seconds: f64,
    /// 这张卡从开始判到它领的最后一块判完的挂钟。
    pub judge_seconds: f64,
    pub dispatches: u64,
    /// 判的那一段里：备缓冲与提交（CPU）、等卡算完、回读（CPU）、在读回的结论字里找判不了的状态（CPU，卡线程上只剩这一遍；
    /// 结论字拆成红项在写库那条线程上，记在 [`GpuJudgeOutcome::seconds_decoding`]）。
    pub seconds_preparing: f64,
    pub seconds_waiting_for_the_card: f64,
    pub seconds_reading_back: f64,
    pub seconds_finding_states_the_gpu_could_not_judge: f64,
    /// 带得最多的一次派活几个状态、卡算得最久的一次派活多久（一次派活带几个按用时调，不一定到 `states_per_dispatch`）。
    pub states_in_the_widest_dispatch: u64,
    pub seconds_of_the_longest_dispatch: f64,
    /// 这张卡的判器各趟的提交账（每趟用时汇总行照它算，[`gpu_pass_lines_of_the_cards`]）。
    #[cfg(feature = "gpu")]
    pub submission_waits_of_the_passes: crate::crash_judge_dispatch::SubmissionWaitsOfThePasses,
}

impl GpuCardOutcome {
    #[must_use]
    pub fn states_per_second(&self) -> f64 {
        if self.judge_seconds > 0.0 {
            self.states_judged as f64 / self.judge_seconds
        } else {
            0.0
        }
    }
}

/// GPU 判器判这一趟的合计：判了多少、花了多久（编内核的时间另计）。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GpuJudgeOutcome {
    pub cards_used: usize,
    pub blocks_judged: u64,
    pub states_judged: u64,
    pub red_states: u64,
    /// 表上卡与编内核的挂钟，几张卡加起来（卡按优先级一张一张备，不同时编）；驱动的盘上缓存命中时几秒，冷编按内核大小几分钟到二十分钟。
    pub compile_seconds: f64,
    /// 判块的挂钟：从第一张卡开始备到最后一块写进库，减去编内核的那一段。
    pub judge_seconds: f64,
    /// 写库的挂钟（CPU，单线程，按块序）。
    pub seconds_storing: f64,
    /// 写库那条线程把卡交来的结论字拆成红项、恢复字、逐盘槽数与计数的挂钟（[`decoded_gpu_block`]，在写库之前、不挡卡）。
    pub seconds_decoding: f64,
    /// 用到的卡每收工一次一行，按优先级（一张卡在收尾再起的那一轮里又收工一次就有两行；用到几张卡看 `cards_used`）。
    pub cards: Vec<GpuCardOutcome>,
}

impl GpuJudgeOutcome {
    #[must_use]
    pub fn states_per_second(&self) -> f64 {
        if self.judge_seconds > 0.0 {
            self.states_judged as f64 / self.judge_seconds
        } else {
            0.0
        }
    }
}

/// 一块在 GPU 上判出来、卡线程原样交给写库那条线程的结论字（判不了的状态卡线程上已经拒过）：写库那条线程照块序先解
/// （[`decoded_gpu_block`]）再写，卡线程交完就去判下一块。
#[cfg(feature = "gpu")]
pub(super) struct GpuVerdictWordsOfABlock {
    pub(super) pending_index: usize,
    pub(super) words_of_each_state: Vec<[u32; crate::crash_judge_gpu::VERDICT_WORDS]>,
    /// 判这一块的卡：序号进 `CRASH_AMPLIFICATION_BLOCK` 那一行，卡型与驱动版本进块属性。
    pub(super) card_index: u32,
    pub(super) card_name: String,
    pub(super) card_driver_version: String,
}

/// 一块的结论字拆成写库要的几样：逐状态红项、恢复字、逐盘自证槽数（盘按 `devices_of_the_pool`）与不变量计数，按块内次序逐状态解、
/// 与卡上判不判无关（纯函数：同一份结论字在哪条线程上解都逐字相同）。
///
/// # Panics
/// 有判不了的状态：卡线程交来之前照 [`first_state_the_gpu_could_not_judge`] 拒过，到这里就是那一步被跳过了。
#[cfg(feature = "gpu")]
pub(super) fn decoded_gpu_block(
    words_of_the_block: GpuVerdictWordsOfABlock,
    devices_of_the_pool: &[u32],
) -> GpuJudgedBlock {
    let GpuVerdictWordsOfABlock {
        pending_index,
        words_of_each_state,
        card_index: _,
        card_name,
        card_driver_version,
    } = words_of_the_block;
    let mut items_of_each_state: Vec<Vec<String>> = Vec::with_capacity(words_of_each_state.len());
    let mut recovery_words_of_each_state: Vec<RecoveryWordsOfState> =
        Vec::with_capacity(words_of_each_state.len());
    let mut state_counts = StateCountsOfABlock::default();
    let mut self_verified_system_configuration_slots_of_the_states =
        SelfVerifiedSystemConfigurationSlotsOfTheStates::default();
    for state_words in &words_of_each_state {
        items_of_each_state
            .push(gpu_verdict_items(state_words).expect("判不了的状态卡线程交来之前已经拒过"));
        recovery_words_of_each_state.push(recovery_words_of_the_gpu_verdict(state_words));
        self_verified_system_configuration_slots_of_the_states.push_a_state(
            self_verified_system_configuration_slots_of_each_device_of_the_gpu_verdict(
                state_words,
                devices_of_the_pool,
            ),
        );
        state_counts.count_the_gpu_bits(
            pool_checker_judged_bits(state_words),
            pool_checker_violated_bits(state_words),
        );
    }
    GpuJudgedBlock {
        pending_index,
        items_of_each_state,
        recovery_words_of_each_state,
        state_counts,
        self_verified_system_configuration_slots_of_the_states,
        card_name,
        card_driver_version,
    }
}

/// 一块在 GPU 上判出来、解好了、还没写库的结果。
#[cfg(feature = "gpu")]
#[derive(Debug, PartialEq, Eq)]
pub(super) struct GpuJudgedBlock {
    pending_index: usize,
    pub(super) items_of_each_state: Vec<Vec<String>>,
    /// 逐状态的恢复字（`crate::crash_judge_gpu::recovery_words_of_the_gpu_verdict`）。
    recovery_words_of_each_state: Vec<RecoveryWordsOfState>,
    pub(super) state_counts: StateCountsOfABlock,
    /// 逐状态的逐盘自证过的系统配置槽数（走读内核第 25 字，`crate::crash_judge_gpu::self_verified_system_configuration_slots_of_each_device_of_the_gpu_verdict`）。
    self_verified_system_configuration_slots_of_the_states:
        SelfVerifiedSystemConfigurationSlotsOfTheStates,
    /// 判这一块的卡（块属性里的卡型与驱动版本，`crate::gpu_unit_checks::GpuCard` 起卡时从 wgpu 的 adapter info 读的）。
    pub(super) card_name: String,
    pub(super) card_driver_version: String,
}

/// 一块 GPU 判出、解好的结论照判段存块的写法备好的几样（[`gpu_judged_block_ready_to_store`]）。
#[cfg(feature = "gpu")]
pub(super) struct GpuJudgedBlockReadyToStore {
    pub(super) verdict_block: VerdictBlock,
    pub(super) violating: Vec<StateOffsetInBlock>,
    pub(super) along: StoredAlongTheVerdicts,
}

/// 一块 GPU 判出、解好的结论备成存库的样子（判段存块 [`store_judged_block`] 与 GPU 批处理服务编结论帧
/// [`conclusion_frame_of_a_gpu_pending_block`] 共用这一处）：逐状态红项登记成判定向量编号（判定向量表只追加，登记即写进 `store`）、
/// 红的状态、计数向量、恢复字、在哪判的（这张卡，判器那一把是这个进程编进去的那一把）、判器交回的逐盘自证槽数连同基镜像那一份。
///
/// # Errors
/// 登记判定向量不成（[`VerdictStore::register_verdict_vector`]）。
#[cfg(feature = "gpu")]
pub(super) fn gpu_judged_block_ready_to_store(
    store: &mut VerdictStore,
    local: &Range<u64>,
    judged: GpuJudgedBlock,
    slots_of_each_device_on_the_base_image: &[(u32, u32)],
) -> Result<GpuJudgedBlockReadyToStore, VerdictStoreError> {
    let numbers =
        verdict_vector_numbers_of_the_red_items_of_each_state(store, &judged.items_of_each_state)?;
    let violating = judged
        .items_of_each_state
        .iter()
        .enumerate()
        .filter(|(_offset, items)| !items.is_empty())
        .map(|(offset, _items)| {
            StateOffsetInBlock(u64::try_from(offset).expect("块内序号装得进 u64"))
        })
        .collect();
    Ok(GpuJudgedBlockReadyToStore {
        verdict_block: VerdictBlock::new(numbers).expect("一块至少一个状态"),
        violating,
        along: StoredAlongTheVerdicts {
            count_vector: judged
                .state_counts
                .into_count_vector(local.end - local.start),
            recovery_words: Some(judged.recovery_words_of_each_state),
            judged_on: JudgingDevice::Gpu {
                card_type: judged.card_name,
                driver_version: judged.card_driver_version,
            },
            self_verified_system_configuration_slots: judged
                .self_verified_system_configuration_slots_of_the_states
                .handed_back_for_the_block(
                    local.end - local.start,
                    slots_of_each_device_on_the_base_image,
                ),
        },
    })
}

/// 卡上的线程发给写库那条线程的话。
#[cfg(feature = "gpu")]
enum CardMessage {
    Block(GpuVerdictWordsOfABlock),
    /// 收工的卡的账：装箱，账比别的几种话大好几倍，不装箱时每一条话都照它的大小占地方。
    Finished(Box<GpuCardOutcome>),
    /// 这张卡判不了领到的块里的某个状态（表越过了内核的容量）：整趟停下、报错，不退回 CPU 判器。
    Refused(String),
    /// 这张卡（`index` 是 `nvidia-smi` 给它的序号）建判器或判一块时 wgpu 报了 Out of Memory（文件头第 5 条）：只停这一张，判一块时报的那一块已经交回队里；
    /// 带那一句为什么。
    OutOfMemory {
        index: u32,
        reason: String,
    },
}

/// 写库那条线程收完一轮卡上的线程发来的话之后手里的：收工的卡各自的账（按收到的次序）、卡拒判的理由、报了 Out of Memory 的卡（序号与那一句，
/// 按收到的次序）。判完的块按块序写进库几块、写库的错在几轮共用的那一份里（[`BlocksStoredInOrder`]）。
#[cfg(feature = "gpu")]
struct CardsMessagesReceived {
    finished_cards: Vec<GpuCardOutcome>,
    refusal: Option<String>,
    cards_out_of_memory: Vec<(u32, String)>,
}

/// 写库那条线程收卡上的线程发来的话，到发送端全丢掉为止：判完的块（还没解的结论字）交 `stored_in_order` 按块序写库（`store_block` 解一块、写一块；后面的块先到就等前面那一块，
/// 到了一起写；写库失败之后到的块不再写），收工的卡记下它的账，拒判的留第一条理由，报了 Out of Memory 的卡都记下；上一次收到之后 `clock` 走过整个进度间隔
/// 还没有新的话，就交 `write_progress_line` 一行 `CRASH_AMPLIFICATION_BLOCK by=wall_clock judge=gpu …`（C611：卡上判一块、编内核比进度间隔久时
/// 照样隔一个间隔报一行，[`gpu_judge_wall_clock_line`]）。`pending_blocks` 是这一趟要判的块数，只进那一行。
#[cfg(feature = "gpu")]
fn receive_the_cards_messages_and_store_the_blocks_in_order(
    receiver: &std::sync::mpsc::Receiver<CardMessage>,
    wall_clock_lines: &mut WallClockProgressLines,
    clock: &dyn Fn() -> std::time::Duration,
    pending_blocks: usize,
    stored_in_order: &mut BlocksStoredInOrder<GpuVerdictWordsOfABlock>,
    store_block: &mut dyn FnMut(GpuVerdictWordsOfABlock) -> Result<(), String>,
    write_progress_line: &mut dyn FnMut(String),
) -> CardsMessagesReceived {
    let mut received = CardsMessagesReceived {
        finished_cards: Vec::new(),
        refusal: None,
        cards_out_of_memory: Vec::new(),
    };
    receive_until_the_senders_are_gone_with_wall_clock_lines(
        receiver,
        wall_clock_lines,
        clock,
        &mut |event| match event {
            ReceivedOrWallClockLineDue::WallClockLineDue { elapsed } => {
                write_progress_line(gpu_judge_wall_clock_line(
                    stored_in_order.blocks_stored(),
                    pending_blocks,
                    stored_in_order.blocks_waiting(),
                    received.finished_cards.len(),
                    elapsed,
                ));
            }
            ReceivedOrWallClockLineDue::Received(CardMessage::Finished(card_outcome)) => {
                received.finished_cards.push(*card_outcome);
            }
            ReceivedOrWallClockLineDue::Received(CardMessage::Refused(reason)) => {
                received.refusal.get_or_insert(reason);
            }
            ReceivedOrWallClockLineDue::Received(CardMessage::OutOfMemory { index, reason }) => {
                received.cards_out_of_memory.push((index, reason));
            }
            ReceivedOrWallClockLineDue::Received(CardMessage::Block(arrived_block)) => {
                stored_in_order.arrive(arrived_block.pending_index, arrived_block, store_block);
            }
        },
    );
    received
}

/// 几张卡一起干的那份活：要判的块排成一队，备好的卡从队头领。
#[cfg(feature = "gpu")]
struct CardsAtWork<'work> {
    plan: &'work CrashPlan,
    pending: &'work [(usize, Range<u64>)],
    tables: &'work PackedJudgeTablesOfTheRun,
    cards: &'work [std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
    /// 每张卡按优先级装到的状态数（一次派活带几个）；0 的这一趟不用。
    states_placed: &'work [u64],
    blocks: BlocksForTheCards,
    /// 写库失败、有卡拒判之后置上：卡上的线程判完手里那一块就停、一块都不再领。一张卡报 Out of Memory 不置（只停那一张）。
    stop: std::sync::atomic::AtomicBool,
    /// 调用方要停（每领一块之前问一次）：不再领新的块，只领判完的块在等的那几块，手里那一块判完照旧交去写库。
    /// 一趟里见过一次就记住，与一轮收尾问的是同一个。
    stop_request: &'work StopRequestRememberedInThisRun<'work>,
}

#[cfg(feature = "gpu")]
impl CardsAtWork<'_> {
    /// 卡上的线程这一刻照什么停（[`judge_blocks_on_one_card`]、[`start_the_next_card_with_states`] 问）：判红了一块都不再领，
    /// 调用方叫停了只领判完的块在等的，都没有照常领。
    fn stop_of_the_cards(&self) -> StopOfTheCards {
        if self.stop.load(std::sync::atomic::Ordering::SeqCst) {
            StopOfTheCards::TheRunFailed
        } else if self.stop_request.is_requested() {
            StopOfTheCards::RequestedByTheCaller
        } else {
            StopOfTheCards::NotRequested
        }
    }
}

/// 卡上一段活做完之后的结局（[`decide_the_work_on_the_card`]）。
#[cfg(feature = "gpu")]
enum WorkOnTheCard<Outcome> {
    Done(Outcome),
    /// 显存不够：`detail` 说 wgpu 报的那一句（这条线程上的错误范围接住的，或别的线程上没接住、记在设备上的）。
    OutOfMemory {
        detail: String,
    },
    /// 活 panic 了，又说不成显存不够：照样往外抛。
    Panicked(Box<dyn std::any::Any + Send>),
}

/// 卡上一段活（`outcome`，panic 了是 `Err`）怎么收：这条线程上的 Out of Memory 范围接到了（`out_of_memory_in_the_scope`）就是显存不够，
/// 活成没成都一样；没接到而活 panic 了，看设备上从活开始起记的（`out_of_memory_on_the_device_since_the_work_began`，别的线程上没接住的
/// Out of Memory、或因它丢了设备）：有就是显存不够，没有照样抛（判器内核在另起的线程上编，那条线程上的
/// Out of Memory 让卡线程在 `join` 那里 panic，范围接不到）。活没 panic 就交它的结果。
#[cfg(feature = "gpu")]
fn decide_the_work_on_the_card<Outcome>(
    outcome: std::thread::Result<Outcome>,
    out_of_memory_in_the_scope: Option<String>,
    out_of_memory_on_the_device_since_the_work_began: Option<String>,
) -> WorkOnTheCard<Outcome> {
    match (
        outcome,
        out_of_memory_in_the_scope,
        out_of_memory_on_the_device_since_the_work_began,
    ) {
        (Ok(_) | Err(_), Some(detail), _) => WorkOnTheCard::OutOfMemory { detail },
        (Ok(outcome), None, _) => WorkOnTheCard::Done(outcome),
        (Err(_), None, Some(detail)) => WorkOnTheCard::OutOfMemory { detail },
        (Err(panic), None, None) => WorkOnTheCard::Panicked(panic),
    }
}

/// 在 `card` 上做 `work`，同时接住 wgpu 报的 Out of Memory（文件头第 5 条）：没报就交 `work` 的结果；报了交 `Err(那一句)`
/// （`what_ran_out` 说是哪一段，`work` 里因为缓冲没建成而 panic 的一并接住、不再往外抛）；没报而 `work` panic 了照样往外抛。
/// wgpu 的错误范围按线程分：这条线程上的接在范围里；`work` 另起的线程上没接住的记在设备上（`crate::gpu_unit_checks::DeviceConditionOfACard`），
/// 照 [`decide_the_work_on_the_card`] 一并认。设备上记的按这段活认（`singlefs_verification::gpu_memory_ledger::WorkOnTheCardOfThisThread`，
/// 同一个设备上别的线程那段活的 Out of Memory 说不成这段活的显存不够，这段活真正的 panic 照样往外抛）。
#[cfg(feature = "gpu")]
fn on_the_card_catching_out_of_memory<Outcome>(
    card: &crate::gpu_unit_checks::GpuCard,
    what_ran_out: &str,
    work: impl FnOnce() -> Outcome,
) -> Result<Outcome, String> {
    let alone = WorksEndingTogetherOnADevice::new();
    on_the_card_catching_out_of_memory_ending_together(card, what_ran_out, alone.join(), work)
}

/// 同 [`on_the_card_catching_out_of_memory`]，这段活与同一个设备上别的几段一起开（`ending` 是它的报到，单元校验和对拍时一块切成的几段）：
/// 活交回那一刻记下设备丢没丢，这条线程上的错误范围接到的 Out of Memory 记进设备之后报到，等那几段都报到了再认设备上记的
/// （`singlefs_verification::gpu_memory_ledger::DeviceConditionOfACard::end_the_work_and_explain_its_failure`：一段因设备丢了先失败、
/// 同块另一段范围里接到的还没记进设备时，先认就认不出、判红：设备因显存不够丢的，不在同一组也认得出；
/// 交回那一刻设备还没丢的，之后别的段丢了设备不把它说成显存不够）。
#[cfg(feature = "gpu")]
fn on_the_card_catching_out_of_memory_ending_together<Outcome>(
    card: &crate::gpu_unit_checks::GpuCard,
    what_ran_out: &str,
    ending: WorkEndingTogether<'_>,
    work: impl FnOnce() -> Outcome,
) -> Result<Outcome, String> {
    let work_on_the_card = crate::gpu_memory_ledger::WorkOnTheCardOfThisThread::begin();
    let out_of_memory_reports_before = card.device_condition().out_of_memory_reports();
    let out_of_memory_scope = card
        .device()
        .push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    // 活交回的这一刻设备丢没丢：之后等同组几段时别的段丢了设备，不把这段活真正的 panic 说成显存不够（
    // 接住与记设备抽在 `gpu_memory_ledger`）
    let (outcome, device_when_the_work_returned) =
        work_caught_noting_the_device_when_it_returned(card.device_condition(), work);
    let out_of_memory_in_the_scope =
        pollster::block_on(out_of_memory_scope.pop()).map(|error| error.to_string());
    let out_of_memory_on_the_device = card
        .device_condition()
        .end_the_work_and_explain_its_failure(
            &work_on_the_card,
            out_of_memory_reports_before,
            device_when_the_work_returned,
            out_of_memory_in_the_scope
                .as_ref()
                .map(|caught| format!("序号 {} 的卡{what_ran_out}报了 {caught}", card.index)),
            ending,
        );
    match decide_the_work_on_the_card(
        outcome,
        out_of_memory_in_the_scope,
        out_of_memory_on_the_device,
    ) {
        WorkOnTheCard::Done(outcome) => Ok(outcome),
        WorkOnTheCard::OutOfMemory { detail } => Err(reason_for_running_out_of_gpu_memory(
            &format!("序号 {} 的卡{what_ran_out}（{detail}）", card.index),
        )),
        WorkOnTheCard::Panicked(panic) => std::panic::resume_unwind(panic),
    }
}

/// 一张卡的线程：表上卡、编内核；备好之后，队里还有活、下一张卡也装了状态，就起下一张卡的线程（卡一张一张备，不同时编内核）；
/// 然后从队头一块一块领、判、把结论字拆成红项、交给写库的线程。有一个状态判不了（结论字带「判不了」），
/// 就叫所有卡停下、发一条 [`CardMessage::Refused`]，不退回 CPU 判器（配了 GPU 就一定要 GPU 判，做不到就报错）。
/// 建判器或判一块时 wgpu 报 Out of Memory（[`on_the_card_catching_out_of_memory`]），只停这一张卡、发一条 [`CardMessage::OutOfMemory`]：
/// 判一块时报的，那一块交回队里给别的卡（[`BlocksForTheCards::give_back`]），不交去写库；建判器时报的，下一张卡照起
/// （P2：一张卡 Out of Memory 不停整趟）。
#[cfg(feature = "gpu")]
fn judge_on_one_card_and_start_the_next<'scope, 'work>(
    scope: &'scope std::thread::Scope<'scope, 'work>,
    work: &'work CardsAtWork<'work>,
    position: usize,
    sender: std::sync::mpsc::Sender<CardMessage>,
) {
    use crate::crash_judge_dispatch::GpuCrashJudge;
    use std::sync::atomic::Ordering;
    let card: &crate::gpu_unit_checks::GpuCard = &work.cards[position];
    let build_started = Instant::now();
    let judge =
        match on_the_card_catching_out_of_memory(card, "建判器（表与草稿区）时", || {
            GpuCrashJudge::on_card_with_packed_tables(
                card,
                work.tables,
                usize::try_from(work.states_placed[position])
                    .expect("一次派活的状态数装得进 usize"),
            )
        }) {
            Ok(judge) => judge,
            Err(reason) => {
                // 写库的线程已经走了（写库失败）就没人收，那一边的错先报
                let _ = sender.send(CardMessage::OutOfMemory {
                    index: card.index,
                    reason,
                });
                start_the_next_card_with_states(scope, work, position, &sender);
                return;
            }
        };
    let build_seconds = build_started.elapsed().as_secs_f64();
    eprintln!(
        "CRASH_AMPLIFICATION_GPU_JUDGE card={:?} index={} memory_budget_mebibytes={} states_per_dispatch={} scratch_mebibytes={} build_seconds={build_seconds:.1}",
        card.name,
        card.index,
        card.memory_budget_mebibytes(),
        judge.states_per_dispatch(),
        judge.scratch_mebibytes()
    );
    start_the_next_card_with_states(scope, work, position, &sender);
    let judging_started = Instant::now();
    let mut outcome = GpuCardOutcome {
        name: card.name.clone(),
        index: card.index,
        memory_budget_mebibytes: card.memory_budget_mebibytes(),
        states_per_dispatch: u64::try_from(judge.states_per_dispatch()).expect("装得进 u64"),
        scratch_mebibytes: u64::try_from(judge.scratch_mebibytes()).expect("装得进 u64"),
        build_seconds,
        ..GpuCardOutcome::default()
    };
    let card_left = judge_blocks_on_one_card(
        &work.blocks,
        &|| work.stop_of_the_cards(),
        &mut |pending_index| {
            let (point_index, local) = &work.pending[pending_index];
            let point = &work.plan.points[*point_index];
            let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
            let words = match on_the_card_catching_out_of_memory(card, "判一块时", || {
                judge.judge_states_unless_stuck_at_the_indivisible_floor(ordinals.clone())
            }) {
                Ok(Ok(words)) => words,
                // 宽度停在拆不开的下限上：同判不了的状态，所有卡停下、整趟报错，不再一个一个派下去
                Ok(Err(stuck)) => {
                    return BlockJudgedOnACard::Refused {
                        reason: stuck.line(None),
                    }
                }
                // 只停这一张卡：这一块交回给别的卡
                Err(reason) => return BlockJudgedOnACard::OutOfMemory { reason },
            };
            // 卡线程上只找判不了的状态（找到就整趟停）；结论字原样交给写库那条线程去解，这条线程接着领下一块
            let finding_started = Instant::now();
            let first_undecidable_state = first_state_the_gpu_could_not_judge(&words);
            outcome.seconds_finding_states_the_gpu_could_not_judge +=
                finding_started.elapsed().as_secs_f64();
            if let Some(offset) = first_undecidable_state {
                let state_words = &words[offset];
                return BlockJudgedOnACard::Refused {
                    reason: format!(
                        "卡 {} 判不了点 {} 块内第 {} 个状态（恢复内核第 1 字 {}、checker 内核第 {} 字 {}：表越过了内核的容量）。\
                         配置开了 GPU，不退回 CPU 判器：照这两个字查是哪张表满了、改内核的容量再跑",
                        card.index,
                        point.name,
                        local.start + u64::try_from(offset).expect("块内序号"),
                        state_words[1],
                        crate::crash_judge_gpu::POOL_CHECKER_UNDECIDABLE_WORD,
                        state_words[crate::crash_judge_gpu::POOL_CHECKER_UNDECIDABLE_WORD]
                    ),
                };
            }
            outcome.blocks_judged += 1;
            outcome.states_judged += local.end - local.start;
            BlockJudgedOnACard::Judged(words)
        },
        &mut |pending_index, words_of_each_state| match sender.send(CardMessage::Block(
            GpuVerdictWordsOfABlock {
                pending_index,
                words_of_each_state,
                card_index: card.index,
                card_name: card.name.clone(),
                card_driver_version: card.driver_version.clone(),
            },
        )) {
            Ok(()) => BlockHandedToTheStore::Handed,
            Err(_) => BlockHandedToTheStore::TheStoreIsGone,
        },
    );
    match card_left {
        CardLeftTheBlocks::NothingLeftOrStopped | CardLeftTheBlocks::TheStoreIsGone => {}
        CardLeftTheBlocks::OutOfMemory { reason } => {
            // 写库的线程已经走了（写库失败）就没人收，那一边的错先报
            let _ = sender.send(CardMessage::OutOfMemory {
                index: card.index,
                reason,
            });
            return;
        }
        CardLeftTheBlocks::Refused { reason } => {
            work.stop.store(true, Ordering::SeqCst);
            // 写库的线程已经走了（写库失败）就没人收，那一边的错先报
            let _ = sender.send(CardMessage::Refused(reason));
            return;
        }
    }
    outcome.judge_seconds = judging_started.elapsed().as_secs_f64();
    let spent = judge.time_spent();
    outcome.dispatches = spent.dispatches;
    outcome.seconds_preparing = spent.seconds_preparing;
    outcome.seconds_waiting_for_the_card = spent.seconds_waiting_for_the_card;
    outcome.seconds_reading_back = spent.seconds_reading_back;
    outcome.states_in_the_widest_dispatch = spent.states_in_the_widest_dispatch;
    outcome.seconds_of_the_longest_dispatch = spent.seconds_of_the_longest_dispatch;
    outcome.submission_waits_of_the_passes = judge.submission_waits_of_the_passes();
    // 每张卡一行：规划显卡要的数（这张卡判了多少、多快、一次派活带几个状态、草稿区占多少显存、挂钟花在哪）
    eprintln!(
        "CRASH_AMPLIFICATION_GPU_CARD card={:?} index={} memory_budget_mebibytes={} blocks={} states={} judge_seconds={:.1} states_per_second={:.0} states_per_dispatch={} scratch_mebibytes={} dispatches={} seconds_preparing={:.1} seconds_waiting_for_the_card={:.1} seconds_reading_back={:.1} seconds_finding_states_the_gpu_could_not_judge={:.1} states_in_the_widest_dispatch={} seconds_of_the_longest_dispatch={:.3}",
        outcome.name,
        outcome.index,
        outcome.memory_budget_mebibytes,
        outcome.blocks_judged,
        outcome.states_judged,
        outcome.judge_seconds,
        outcome.states_per_second(),
        outcome.states_per_dispatch,
        outcome.scratch_mebibytes,
        outcome.dispatches,
        outcome.seconds_preparing,
        outcome.seconds_waiting_for_the_card,
        outcome.seconds_reading_back,
        outcome.seconds_finding_states_the_gpu_could_not_judge,
        outcome.states_in_the_widest_dispatch,
        outcome.seconds_of_the_longest_dispatch
    );
    // 写库的线程已经走了（写库失败）就没人收，丢了也无妨
    let _ = sender.send(CardMessage::Finished(Box::new(outcome)));
}

/// 第 ① 段借草稿区之前这一步怎么走（[`scratch_step_before_booking`]）。
#[cfg(feature = "gpu")]
#[derive(Clone, Debug, PartialEq, Eq)]
enum ScratchStepBeforeBooking {
    /// 至少一张卡此刻带得了一个状态：去借。
    BookOnTheCards,
    /// 每张卡此刻都带不了一个状态、至少一张空着时带得了：打这一行，报本次未跑（不借、不判红）。
    NotRunNoCardHoldsAStateNow { not_run_line: String },
    /// 空着时也带不了：往下走到没有一张卡装得下那一句，判红。
    RedNoCardHoldsAStateEvenWhenEmpty,
}

/// 照借之前的判定（[`decide_scratch_before_booking`]）定这一步：此刻借不到的那一行照对拍用例那一路写（同一个记号、原因写显存不够），
/// 带每张卡此刻能借的（额度减驱动那一份）与一个状态要借的 `need_of_one_state_mebibytes`。
#[cfg(feature = "gpu")]
fn scratch_step_before_booking(
    decision: ScratchBeforeBooking,
    run_cards: &[CardCapacityOfThisRun],
    need_of_one_state_mebibytes: u64,
) -> ScratchStepBeforeBooking {
    match decision {
        ScratchBeforeBooking::BookOnTheCardsThatHoldAStateNow => {
            ScratchStepBeforeBooking::BookOnTheCards
        }
        ScratchBeforeBooking::NoCardHoldsAStateNow => {
            let card_rooms_mebibytes: Vec<u64> = run_cards
                .iter()
                .map(|card| {
                    card.memory_budget_mebibytes
                        .saturating_sub(crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER)
                })
                .collect();
            ScratchStepBeforeBooking::NotRunNoCardHoldsAStateNow {
                not_run_line: gpu_not_run_line(
                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                    &reason_for_a_booking_without_room_now(
                        need_of_one_state_mebibytes,
                        &card_rooms_mebibytes,
                    ),
                ),
            }
        }
        ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty => {
            ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty
        }
    }
}

/// 第 `position` 张卡备好了（或建判器时报了 Out of Memory、不判）：后面还有装了状态的卡、队里照此刻的叫停还有领得的块
/// （[`CardsAtWork::stop_of_the_cards`]，只在后面有装了状态的卡时问一次），就起下一张的线程（卡一张一张备，不同时编内核）。
#[cfg(feature = "gpu")]
fn start_the_next_card_with_states<'scope, 'work>(
    scope: &'scope std::thread::Scope<'scope, 'work>,
    work: &'work CardsAtWork<'work>,
    position: usize,
    sender: &std::sync::mpsc::Sender<CardMessage>,
) {
    // 起哪一张、看不看叫停抽在 `gpu_memory_ledger`；叫停交成闭包，没有下一张卡不问
    if let Some(next_position) =
        next_card_to_start(work.states_placed, position, &work.blocks, || {
            work.stop_of_the_cards()
        })
    {
        let sender = sender.clone();
        scope.spawn(move || {
            judge_on_one_card_and_start_the_next(scope, work, next_position, sender);
        });
    }
}

/// 一张卡照额度 `memory_budget_mebibytes`（MiB）一次派活带得了几个状态（表上卡之后剩下的显存装草稿区）：表比存储缓冲绑定上限
/// `maximum_storage_buffer_binding_size` 大、或额度装不下表与驱动那一份，是 0。
#[cfg(feature = "gpu")]
fn states_in_flight_within_the_budget(
    maximum_storage_buffer_binding_size: u64,
    memory_budget_mebibytes: u64,
    tables: &PackedJudgeTablesOfTheRun,
) -> u64 {
    if tables.mebibytes() * 1024 * 1024 > maximum_storage_buffer_binding_size
        || memory_budget_mebibytes
            < tables.mebibytes() + crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER
    {
        return 0;
    }
    u64::try_from(crate::crash_judge_dispatch::states_in_flight_within(
        maximum_storage_buffer_binding_size,
        memory_budget_mebibytes,
        tables,
    ))
    .expect("装得进 u64")
}

/// 这一趟本机的一张卡（按优先级）：序号、名字、给判器的额度（MiB），与照额度一次派活带得了几个状态
/// （存储缓冲绑定上限只在算状态数时用，不存）。卡是调用方这一趟自己起的，额度照卡的额度（起设备那一刻的空闲算的）。
#[cfg(feature = "gpu")]
struct CardCapacityOfThisRun {
    index: u32,
    name: String,
    memory_budget_mebibytes: u64,
    states_in_flight_capacity: u64,
    /// 这张卡空着时一次派活带得了几个状态（照空卡上能用的算的额度，[`states_in_flight_now_and_when_empty_of_a_card_started_for_this_run`]）：
    /// 每张卡此刻都带不了一个状态时照它分此刻借不到与放不下。
    states_in_flight_capacity_when_the_card_is_empty: u64,
}

/// 这一趟本机每张卡（`cards`，调用方这一趟自己起的，按优先级）：此刻照起设备那一刻的额度、空着时照空卡上能用的算的额度
/// （[`states_in_flight_now_and_when_empty_of_a_card_started_for_this_run`]，[`states_in_flight_within_the_budget`]）。
#[cfg(feature = "gpu")]
fn card_capacities_of_this_run(
    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
    tables: &PackedJudgeTablesOfTheRun,
) -> Vec<CardCapacityOfThisRun> {
    cards
        .iter()
        .map(|card| {
            let maximum_storage_buffer_binding_size =
                card.device().limits().max_storage_buffer_binding_size;
            let (states_in_flight_capacity, states_in_flight_capacity_when_the_card_is_empty) =
                states_in_flight_now_and_when_empty_of_a_card_started_for_this_run(
                    card.memory_budget_mebibytes(),
                    card.memory_budget_when_the_card_is_empty_mebibytes(),
                    &|memory_budget_mebibytes| {
                        states_in_flight_within_the_budget(
                            maximum_storage_buffer_binding_size,
                            memory_budget_mebibytes,
                            tables,
                        )
                    },
                );
            CardCapacityOfThisRun {
                index: card.index,
                name: card.name.clone(),
                memory_budget_mebibytes: card.memory_budget_mebibytes(),
                states_in_flight_capacity,
                states_in_flight_capacity_when_the_card_is_empty,
            }
        })
        .collect()
}

/// 这一趟自己起的一张卡此刻与空着时一次派活各带得了几个状态（`states_in_flight_for_a_budget` 照给判器的额度 MiB 算）：此刻照起设备那一刻的额度
/// （`memory_budget_now_mebibytes`，[`crate::gpu_unit_checks::GpuCard::memory_budget_mebibytes`]），空着时照空卡上能用的算的额度
/// （`memory_budget_when_the_card_is_empty_mebibytes`，[`crate::gpu_unit_checks::GpuCard::memory_budget_when_the_card_is_empty_mebibytes`]）。
/// 每张卡此刻都带不了一个状态而空着时带得了，是此刻借不到、报本次未跑，不是放不下、判红（[`decide_scratch_before_booking`]）。
#[cfg(feature = "gpu")]
fn states_in_flight_now_and_when_empty_of_a_card_started_for_this_run(
    memory_budget_now_mebibytes: u64,
    memory_budget_when_the_card_is_empty_mebibytes: u64,
    states_in_flight_for_a_budget: &dyn Fn(u64) -> u64,
) -> (u64, u64) {
    (
        states_in_flight_for_a_budget(memory_budget_now_mebibytes),
        states_in_flight_for_a_budget(memory_budget_when_the_card_is_empty_mebibytes),
    )
}

/// 第 ① 段在卡上。卡按 `cards` 的次序（优先级）用：要判的状态先装第一张卡，装到它的显存额度带得了的状态数为止，
/// 装不下的才给下一张（[`place_states_on_cards_by_priority`]）；用到的卡一张一张备（表上卡、编内核），备好就从同一队里领块。
/// 判完的块按块序一块一块写库（后判完的先到就等前面的）：中途被停，写进库的块下一次不再判；
/// 判定向量的编号按块序登记，与用了几张卡无关。配置开了 GPU 就一定由 GPU 判（不接受默认降级）：
/// 建不出表（有清零写、写与落点对不上）、有状态判不了（表越过内核容量），都报错，不退回 CPU 判器；报错之前判完的块照旧在库里。
/// 每领一块之前先问 `stop_is_requested`：要停就不再领新的，已经领的块判完、写进库再收尾（库里不留空洞）；有卡报了 Out of Memory 交回的块、
/// 判完的块在等它的，叫停了也照领照判（交回的块落到收尾再起的那一轮里也一样），判完的块照块序都入库、没判的留给下一趟。
///
/// # Errors
/// 库读写失败；建不出表；没有一张卡装得下这条流的表与一个状态的草稿区；有状态 GPU 判不了；每张卡此刻都带不了一个状态而空着时带得了
/// （「本次未跑」那一行已经打了，一块都没判）；卡上 wgpu 报了 Out of Memory
/// 而别的卡没把剩下的块判完（「本次未跑」那一行已经打了，报错之前判完的块在库里；只停报了的那一张卡）。
///
/// # Panics
/// 卡上的线程 panic（内核没把状态判完、编内核之前量得内核超了预算）。
#[cfg(feature = "gpu")]
pub fn judge_my_blocks_on_gpu_cards(
    store: &mut VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    is_mine: &dyn Fn(u64) -> bool,
    version: &JudgingUnitsFingerprint,
    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<(CheckingTally, GpuJudgeOutcome), String> {
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    let tables = crate::crash_library::record_the_judge_tables_and_read_them_back(
        store,
        &packed_judge_tables_of_the_plan(flow, plan)?,
    )?;
    judge_my_blocks_on_the_cards_started_for_this_run(
        &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(store),
        flow,
        plan,
        &tables,
        is_mine,
        &ConclusionsStoredUnder::TheJudgingUnitsFingerprint(version),
        cards,
        stop_is_requested,
    )
}

/// 「本次未跑」那一行里产品路径的名字（GPU 用例写用例名的那一格）。
#[cfg(feature = "gpu")]
pub(super) const PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE: &str =
    "crash_amplification::run_from_environment";

/// 这一趟的卡判成照这个进程的卡池用（[`gpu_cards_in_use`] 交 [`GpuCardsSource::TheCardPoolOfThisProcess`]）时的那个池。
///
/// # Panics
/// 这个进程没起过卡池（调用方把卡的来路判错了）。
#[cfg(feature = "gpu")]
pub(super) fn the_card_pool_this_run_takes_its_cards_from(
) -> &'static crate::gpu_unit_checks::gpu_test_start::GpuCardPool {
    crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process().expect(
        "这一趟的卡判成照这个进程的卡池用时池已经起好（gpu_cards_in_use 看过），起好的池在进程里不会再变回没起",
    )
}

/// 给定状态表的判器向卡池借不到时报错的那一句（不打印）：此刻借不到、借到的卡上起设备时显存不够，照运行条件不满足报
/// [`GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX`] 那一行（原因照「本次未跑」那一行的原因写，行里不带「本次未跑」的记号：入口照这一行退 78，
/// 与流水线一侧「配了 GPU 卡用不上退 78、不降级」同；不退回 CPU 判器）；要的比每张卡空着时能借的都大、卡上的显存读不出照旧那一句（判红）。
#[cfg(feature = "gpu")]
pub(super) fn given_states_lease_refusal_as_a_run_condition_not_met(
    table_name: &str,
    refusal: &crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal,
) -> String {
    match not_run_line_or_refusal_of_a_lease("给定状态表的判器", refusal) {
        Ok(not_run_line) => {
            let reason = not_run_line
                .split_once(" reason=")
                .map_or(not_run_line.as_str(), |(_, reason)| reason);
            format!(
                "{GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX} table={table_name} reason={reason}：配了 GPU，给定状态表的判器借不到显存，运行条件不满足（退 78），不换成 CPU 判器"
            )
        }
        Err(red) => red,
    }
}

/// 起过卡池的产品路径向卡池借不到时那一行「本次未跑」或判红的那一句，不打印（造那一行与打印分开，不要卡的用例只调这一半，
/// 测试段的标准输出里不出现门禁会收成本次未跑的那一行）：此刻借不到、借到的卡上起设备时显存不够交 `Ok(那一行)`。
///
/// # Errors
/// 要的比每张卡空着时能借的都大（说法与没有一张卡装得下这条流的表那一句同一类，分项驱动照它认成设备用不上）、卡上的显存读不出。
#[cfg(feature = "gpu")]
fn not_run_line_or_refusal_of_a_lease(
    what_was_booked: &str,
    refusal: &crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal,
) -> Result<String, String> {
    use crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal;
    let refusal = match refusal {
        CardPoolLeaseRefusal::Booking(refusal) => refusal,
        CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice { reason } => {
            return Ok(gpu_not_run_line(
                PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                reason,
            ));
        }
    };
    if let BookingRefusal::NeedLargerThanEveryCard(NeedLargerThanEveryCard {
        need_mebibytes,
        largest_ceiling_mebibytes,
    }) = refusal
    {
        return Err(format!(
            "没有一张卡装得下这条流的表与{what_was_booked}：要借 {need_mebibytes} MiB，池里最大的一张卡空着时也只借得出 {largest_ceiling_mebibytes} MiB。\
             看配置里每张卡的显存额度与卡上此刻的空闲显存"
        ));
    }
    match scratch_booking_refusal_outcome(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, 0, refusal) {
        ScratchBookingRefusalOutcome::NotRun { not_run_line } => Ok(not_run_line),
        ScratchBookingRefusalOutcome::Red { reason } => Err(format!("{what_was_booked}：{reason}")),
    }
}

/// 第 ① 段每张卡上的判器用什么（只列装了状态的卡，按这一趟卡的次序）：卡、每张卡装几个状态（一次派活带几个，都大于 0）；
/// 连同这一趟本机每张卡（与 [`CardCapacityOfThisRun`] 的表逐张对上）各装了几个。
#[cfg(feature = "gpu")]
struct StatesPlacedOnTheCards {
    cards_for_the_judges: Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>>,
    states_placed: Vec<u64>,
    states_placed_on_each_card_of_this_run: Vec<u64>,
}

/// 把 `pending_states` 个状态照 [`place_states_on_cards_by_priority`] 装到这一趟自己起的卡上（`cards`，与 `states_each_card_holds` 逐张对上，
/// 每张最多装那么多）；卡照原样用，不借显存账上的草稿区。
#[cfg(feature = "gpu")]
fn states_placed_on_the_cards_started_for_this_run(
    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
    pending_states: u64,
    states_each_card_holds: &[u64],
) -> StatesPlacedOnTheCards {
    let states_placed_on_each_card_of_this_run =
        place_states_on_cards_by_priority(pending_states, states_each_card_holds);
    let placed: Vec<(&std::sync::Arc<crate::gpu_unit_checks::GpuCard>, u64)> = cards
        .iter()
        .zip(&states_placed_on_each_card_of_this_run)
        .filter(|(_, states_placed)| **states_placed > 0)
        .map(|(card, states_placed)| (card, *states_placed))
        .collect();
    StatesPlacedOnTheCards {
        cards_for_the_judges: placed.iter().map(|(card, _)| (*card).clone()).collect(),
        states_placed: placed
            .iter()
            .map(|(_, states_placed)| *states_placed)
            .collect(),
        states_placed_on_each_card_of_this_run,
    }
}

/// 同 [`judge_my_blocks_on_gpu_cards`]，判表已读回、库每一次读写现取现放：卡是调用方这一趟自己起的（`cards`），不进显存账，
/// 本机每张卡装几个状态照起设备那一刻的额度算（[`card_capacities_of_this_run`]、[`states_placed_on_the_cards_started_for_this_run`]）。
///
/// # Errors
/// 同 [`judge_my_blocks_on_gpu_cards`]：每张卡此刻都带不了一个状态而空着时带得了（一块都不判）、卡上 wgpu 报了 Out of Memory 而别的卡没把
/// 剩下的块判完（之前判完的块在库里），两样都先打「本次未跑」那一行，再交回它的原因。
///
/// # Panics
/// 同 [`judge_my_blocks_on_gpu_cards`]。
#[cfg(feature = "gpu")]
#[allow(
    clippy::too_many_arguments,
    reason = "库、流、计划、判表、分工、判法单元指纹、卡、叫停，各是各的东西，凑成结构体只为让签名变短"
)]
#[allow(
    clippy::too_many_lines,
    reason = "装卡、几张卡一起判、按块序写库、收尾，是第 ① 段的全部步骤，拆开要把十几个量来回传"
)]
fn judge_my_blocks_on_the_cards_started_for_this_run(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    tables: &PackedJudgeTablesOfTheRun,
    is_mine: &dyn Fn(u64) -> bool,
    conclusions: &ConclusionsStoredUnder<'_>,
    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<(CheckingTally, GpuJudgeOutcome), String> {
    let pending = pending_blocks(&library.take(), plan, conclusions, is_mine)
        .map_err(|error| format!("读判定块失败：{error:?}"))?;
    // 按发布分的态数（J7）：GPU 判器不逐状态造镜像，存块时按段的序号区间现算
    let publishes_of_the_states = PublishesOfTheStates::of_the_plan(flow, plan);
    let mut tally = CheckingTally::default();
    let mut outcome = GpuJudgeOutcome::default();
    if pending.is_empty() {
        return Ok((tally, outcome));
    }
    // 走读内核第 25 字按池里的盘的次序（判表 `devices` 照基镜像的 `PoolReader::device_identities` 建）；基镜像那一份逐盘自证槽数
    // 用 CPU 参照算（基镜像不是崩溃状态、不上卡），随每一块存，两槽全废盘面观察者在判段拿它比
    let devices_of_the_pool: Vec<u32> =
        singlefs_core::recovery::PoolReader::device_identities(flow.base)
            .into_iter()
            .map(|device| device.0)
            .collect();
    let slots_of_each_device_on_the_base_image =
        self_verified_system_configuration_slots_of_each_device(flow.base);
    // 几轮之间带着的（`gpu_memory_ledger`，不要卡就测得到）：按块序写库的那一份（上一轮判完、等着前面那一块的，下一轮那一块判完照块序一起写）、
    // 下一轮的那一队块、报过 Out of Memory 的卡与头一句、收过工的卡
    let mut rounds: RoundsOfTheCards<GpuVerdictWordsOfABlock> =
        RoundsOfTheCards::new(pending.len());
    // 卡线程与一轮收尾问的是同一个叫停，见过一次就记住（调用方 `run_from_environment` 交的已经是记住的那一个）
    let stop_request = StopRequestRememberedInThisRun::new(stop_is_requested);
    // 这一趟本机每张卡的优先级（头一轮那一份，后面几轮只少不多）：收尾时每张卡的账照它排
    let mut card_indices_by_priority: Vec<u32> = Vec::new();
    let started = Instant::now();
    // 每再起一轮至少少一张卡（报过 Out of Memory 的不再用），轮数不超过卡数
    loop {
        let pending_states: u64 = rounds
            .blocks_of_the_next_round()
            .iter()
            .map(|pending_index| {
                let (_, local) = &pending[*pending_index];
                local.end - local.start
            })
            .sum();
        let cards_of_the_round: Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>> = cards
            .iter()
            .filter(|card| !rounds.is_short_of_memory(card.index))
            .cloned()
            .collect();
        let run_cards: Vec<CardCapacityOfThisRun> =
            card_capacities_of_this_run(&cards_of_the_round, tables)
                .into_iter()
                .filter(|card| !rounds.is_short_of_memory(card.index))
                .collect();
        if card_indices_by_priority.is_empty() {
            card_indices_by_priority = run_cards.iter().map(|card| card.index).collect();
        }
        let capacities: Vec<u64> = run_cards
            .iter()
            .map(|card| card.states_in_flight_capacity)
            .collect();
        let capacities_when_the_cards_are_empty: Vec<u64> = run_cards
            .iter()
            .map(|card| card.states_in_flight_capacity_when_the_card_is_empty)
            .collect();
        let scratch = match scratch_step_before_booking(
            decide_scratch_before_booking(&capacities, &capacities_when_the_cards_are_empty),
            &run_cards,
            crate::crash_judge_dispatch::mebibytes_a_judge_needs(tables, 1),
        ) {
            ScratchStepBeforeBooking::BookOnTheCards => {
                Some(states_placed_on_the_cards_started_for_this_run(
                    &cards_of_the_round,
                    pending_states,
                    &capacities,
                ))
            }
            ScratchStepBeforeBooking::NotRunNoCardHoldsAStateNow { not_run_line } => {
                println!("{not_run_line}");
                return Err(not_run_line);
            }
            ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty => None,
        };
        let states_placed_on_each_card_of_this_run = scratch.as_ref().map_or_else(
            || vec![0; run_cards.len()],
            |scratch| scratch.states_placed_on_each_card_of_this_run.clone(),
        );
        for ((card, capacity), placed) in run_cards
            .iter()
            .zip(&capacities)
            .zip(&states_placed_on_each_card_of_this_run)
        {
            eprintln!(
                "CRASH_AMPLIFICATION_GPU_PLACEMENT card={:?} index={} memory_budget_mebibytes={} states_in_flight_capacity={capacity} states_placed={placed} pending_states={pending_states} tables_mebibytes={}",
                card.name,
                card.index,
                card.memory_budget_mebibytes,
                tables.mebibytes()
            );
        }
        let Some(scratch) = scratch.filter(|scratch| !scratch.cards_for_the_judges.is_empty())
        else {
            if let Some(reason) = rounds.first_out_of_memory().map(str::to_string) {
                // 报过 Out of Memory 的卡之外没有一张装得下：没有卡能判剩下的块了
                println!(
                    "{}",
                    gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
                );
                return Err(reason);
            }
            return Err(format!(
                "没有一张卡装得下这条流的表（{} MiB）与一个状态的草稿区：看配置里每张卡的显存额度与卡上此刻的空闲显存",
                tables.mebibytes()
            ));
        };
        let work = CardsAtWork {
            plan,
            pending: &pending,
            tables,
            cards: &scratch.cards_for_the_judges,
            states_placed: &scratch.states_placed,
            blocks: rounds.take_the_blocks_of_the_round(),
            stop: std::sync::atomic::AtomicBool::new(false),
            stop_request: &stop_request,
        };
        let received = std::thread::scope(|scope| {
            let (sender, receiver) = std::sync::mpsc::channel::<CardMessage>();
            let work = &work;
            scope.spawn(move || {
                judge_on_one_card_and_start_the_next(scope, work, 0, sender);
            });
            // 卡上判一块、编内核都可能比进度间隔久：等着收的这条线程隔一个间隔打一行按挂钟的
            let clock = || started.elapsed();
            let mut wall_clock_lines =
                WallClockProgressLines::every_progress_line_interval(clock());
            let received = receive_the_cards_messages_and_store_the_blocks_in_order(
                &receiver,
                &mut wall_clock_lines,
                &clock,
                pending.len(),
                rounds.stored_in_order(),
                &mut |words_of_the_block| {
                    let decoding_started = Instant::now();
                    let card_index = words_of_the_block.card_index;
                    let mut block = decoded_gpu_block(words_of_the_block, &devices_of_the_pool);
                    outcome.seconds_decoding += decoding_started.elapsed().as_secs_f64();
                    let storing_started = Instant::now();
                    let (point_index, local) = &pending[block.pending_index];
                    let point = &plan.points[*point_index];
                    eprintln!(
                        "CRASH_AMPLIFICATION_BLOCK point={} block_start={} states={} red={} judge=gpu card={card_index}",
                        point.name,
                        local.start,
                        local.end - local.start,
                        block
                            .items_of_each_state
                            .iter()
                            .filter(|items| !items.is_empty())
                            .count()
                    );
                    block
                        .state_counts
                        .absorb(publishes_of_the_states.state_counts_of(
                            &(point.ordinals.start + local.start..point.ordinals.start + local.end),
                        ));
                    let stored = store_judged_block(
                        &mut library.take(),
                        *point_index,
                        point,
                        local,
                        conclusions,
                        block,
                        &slots_of_each_device_on_the_base_image,
                        &mut tally,
                    );
                    outcome.seconds_storing += storing_started.elapsed().as_secs_f64();
                    match stored {
                        Ok(()) => {
                            outcome.blocks_judged += 1;
                            outcome.states_judged += local.end - local.start;
                            Ok(())
                        }
                        Err(error) => {
                            work.stop.store(true, std::sync::atomic::Ordering::SeqCst);
                            Err(format!("写判定块失败：{error:?}"))
                        }
                    }
                },
                &mut |line| eprintln!("{line}"),
            );
            received
        });
        if let Some(error) = rounds.stored_in_order().storing_error() {
            return Err(error.to_string());
        }
        if let Some(reason) = received.refusal {
            return Err(reason);
        }
        let finished_card_indices: Vec<u32> = received
            .finished_cards
            .iter()
            .map(|card| card.index)
            .collect();
        outcome.cards.extend(received.finished_cards);
        let stop = stop_request.stop_of_the_run();
        let run_card_indices: Vec<u32> = run_cards.iter().map(|card| card.index).collect();
        // 收工之后的不变量（没有卡报 Out of Memory 时判完的块都写进了库、没叫停时块都领走了）也在这一步里判
        match rounds.after_the_round(
            &work.blocks,
            &run_card_indices,
            finished_card_indices,
            &received.cards_out_of_memory,
            stop,
        ) {
            AfterARoundOfTheCards::EveryBlockStored => {
                if rounds.first_out_of_memory().is_some() {
                    eprintln!(
                        "CRASH_AMPLIFICATION_GPU_OUT_OF_MEMORY_ON_A_CARD 有卡报了 Out of Memory、只停那一张，别的卡判完了全部 {} 块",
                        pending.len()
                    );
                }
                break;
            }
            AfterARoundOfTheCards::StoppedOnRequest => {
                // 叫停加有卡报过 Out of Memory：显存不够的缘由照本次未跑那一行打出来
                if let Some(not_run_line) = rounds.not_run_line_of_a_stop_after_an_out_of_memory(
                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                ) {
                    println!("{not_run_line}");
                }
                break;
            }
            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory => {
                eprintln!(
                    "CRASH_AMPLIFICATION_GPU_ANOTHER_ROUND 报了 Out of Memory 的卡 {:?} 收工之后还有 {} 块没写进库，收尾再起一轮、用别的卡判剩下的 {} 块",
                    rounds.cards_short_of_memory(),
                    pending.len() - rounds.blocks_stored(),
                    rounds.blocks_of_the_next_round().len()
                );
            }
            AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor => {
                eprintln!(
                    "CRASH_AMPLIFICATION_GPU_ANOTHER_ROUND_AFTER_THE_STOP 叫停了；报了 Out of Memory 的卡 {:?} 交回的块前面判完的 {} 块在等，再起一轮、用别的卡只判它们在等的那几块，判完照叫停收尾",
                    rounds.cards_short_of_memory(),
                    rounds.blocks_waiting()
                );
            }
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort => {
                let reason = rounds
                    .first_out_of_memory()
                    .expect("这一轮有卡报了 Out of Memory 才会走到本次未跑这一支")
                    .to_string();
                println!(
                    "{}",
                    gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
                );
                return Err(reason);
            }
        }
    }
    outcome.cards.sort_by_key(|card| {
        card_indices_by_priority
            .iter()
            .position(|index| *index == card.index)
    });
    // 一张卡在收尾再起的那一轮里又收工一次，`outcome.cards` 里有它两行，用到的卡只算一张
    outcome.cards_used = rounds.cards_used();
    outcome.compile_seconds = outcome.cards.iter().map(|card| card.build_seconds).sum();
    outcome.judge_seconds = (started.elapsed().as_secs_f64() - outcome.compile_seconds).max(0.0);
    outcome.red_states = tally.red_states;
    Ok((tally, outcome))
}

/// GPU 判完这一趟打的那一行：判了多少、多快，连这一趟的展开上限与全域与否（[`run_from_environment`] 打，展开上限只有它知道）。
#[must_use]
pub fn gpu_total_line(
    outcome: &GpuJudgeOutcome,
    expand_up_to: ExpandUpToWritesPerSegment,
    exhaustive: bool,
) -> String {
    format!(
        "CRASH_AMPLIFICATION_GPU_TOTAL cards_used={} blocks={} states={} compile_seconds={:.1} judge_seconds={:.1} states_per_second={:.0} seconds_storing={:.1} seconds_decoding={:.1} expand_up_to={} exhaustive={exhaustive}",
        outcome.cards_used,
        outcome.blocks_judged,
        outcome.states_judged,
        outcome.compile_seconds,
        outcome.judge_seconds,
        outcome.states_per_second(),
        outcome.seconds_storing,
        outcome.seconds_decoding,
        expand_up_to.0
    )
}

/// GPU 判完这一趟、紧接着 [`gpu_total_line`] 打的每趟用时汇总行：每张卡每一趟一行（同一张卡收工几次并成一份），
/// 没有卡派过活（整段复用）打一行提交次数 0 的（[`crate::crash_judge_dispatch::gpu_pass_summaries`]、
/// [`crate::crash_judge_dispatch::GpuPassSummary::line`]）。判完格式化一次，不在派活里格式化。
#[cfg(feature = "gpu")]
#[must_use]
pub fn gpu_pass_lines_of_the_cards(flow_name: &str, cards: &[GpuCardOutcome]) -> Vec<String> {
    crate::crash_judge_dispatch::gpu_pass_summaries(cards.iter().map(|card| {
        (
            card.name.as_str(),
            card.index,
            &card.submission_waits_of_the_passes,
        )
    }))
    .iter()
    .map(|summary| summary.line(flow_name))
    .collect()
}

#[cfg(test)]
mod tests;
