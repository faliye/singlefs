//! 在 CPU 上核：几条线程领块、逐块判、按块序写回（[`check_blocks_where_until_with_threads`]），叫停文件与核对线程数的旋钮。

use super::*;

/// 核对一趟的结果。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CheckingTally {
    pub blocks_checked: u64,
    pub states_checked: u64,
    pub red_states: u64,
    /// 每个红状态的违例正文（点名、块内序号、正文），给门禁与人看；只收前 [`RED_TEXTS_KEPT`] 条。
    pub red_texts: Vec<(String, u64, String)>,
}

/// 核对线程数的环境变量。
pub const CHECKING_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_THREADS";

/// 核对线程数：环境变量 [`CHECKING_THREADS_ENVIRONMENT_VARIABLE`]，没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取
/// `.claude/heavy-thread-default` 的登记值；`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」）。
///
/// # Panics
/// 同 [`checking_threads_from_the_variable_value`]；值不是 UTF-8 也 panic。
#[must_use]
pub fn checking_threads() -> usize {
    checking_threads_from_the_variable_value(
        process_environment_variable(CHECKING_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
    )
}

/// 核对线程数变量的值（`None` 是没设）换成线程数：设了照它，没设取每台默认重活线程数（`thread_cap_value` 是 `SINGLEFS_THREAD_CAP`
/// 的值，判法在 `singlefs_verification::heavy_threads`）；解析经数值环境变量共用的那一份，线程数取 `NonZeroUsize`，与仓里别的线程数变量
/// 同一口径（C606：写错、写 0 都停下，不悄悄退回默认）。
///
/// # Panics
/// 设了而不是正整数（含 0、空串）：消息带变量名与原值；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
#[must_use]
pub fn checking_threads_from_the_variable_value(
    value: Option<&str>,
    thread_cap_value: Option<&str>,
) -> usize {
    own_variable_or_default_heavy_threads(
        CHECKING_THREADS_ENVIRONMENT_VARIABLE,
        value,
        thread_cap_value,
    )
    .get()
}

/// 核对录入表里还没有判定的块：按分工只核 `owner` 那一方分到的；一块切成几段多线程判（[`checking_threads`]），
/// 每块在 CPU 上逐状态判（单元两道校验和也在 CPU 上，由池级 checker 判，这一段不对拍），
/// 判完一块就按块的次序单线程写回判定块与违例——判定向量的编号按块序登记，与线程数无关，输出与单线程逐字相同。
/// 同一块两方不会同时领：块的归属由权重表与块序号定死。
///
/// # Errors
/// 库读写失败；判定向量超过 2^32 种。
///
/// # Panics
/// 某个核对线程 panic（判状态的代码自己的断言）。
pub fn check_recorded_blocks(
    store: &mut VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    shares: &WeightedShares,
    owner: usize,
    version: &JudgingUnitsFingerprint,
) -> Result<CheckingTally, VerdictStoreError> {
    check_blocks_where(
        store,
        flow,
        plan,
        &|block_sequence| shares.owner_of_block(block_sequence) == owner,
        version,
    )
}

/// 核对 `is_mine` 判为自己的那些块（块序号 = 全部点的块按次序编号）；多线程与写库的做法见 [`check_recorded_blocks`]。
///
/// # Errors
/// 库读写失败；判定向量超过 2^32 种。
pub fn check_blocks_where(
    store: &mut VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    is_mine: &(dyn Fn(u64) -> bool + Sync),
    version: &JudgingUnitsFingerprint,
) -> Result<CheckingTally, VerdictStoreError> {
    check_blocks_where_until(
        store,
        flow,
        plan,
        is_mine,
        &ConclusionsStoredUnder::TheJudgingUnitsFingerprint(version),
        &|| false,
    )
}

/// 停下的请求：环境变量 `SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE` 指的文件在，就不再领新的块（手里那一块判完、写进库），这一趟提早收尾。
/// 给「本机先判着、等别的机器准备好再重新分工」用：判过的块在库里，下一趟接着判。
#[must_use]
pub fn stop_is_requested_by_the_environment() -> bool {
    stop_file_of_the_environment().is_some_and(|path| path.exists())
}

/// 环境变量 `SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE` 指的叫停文件：读这个变量的只有这一处。
pub(super) fn stop_file_of_the_environment() -> Option<PathBuf> {
    std::env::var_os("SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE").map(PathBuf::from)
}

/// 同 [`check_blocks_where`]，每领一块之前先问 `stop_is_requested`：要停就不再领，已经判完的块都写进了库。
///
/// # Errors
/// 同 [`check_blocks_where`]。
pub fn check_blocks_where_until(
    store: &mut VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    is_mine: &(dyn Fn(u64) -> bool + Sync),
    conclusions: &ConclusionsStoredUnder<'_>,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<CheckingTally, VerdictStoreError> {
    check_blocks_where_until_with_threads(
        &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(store),
        flow,
        plan,
        is_mine,
        conclusions,
        stop_is_requested,
        checking_threads(),
    )
}

/// 同 [`check_blocks_where_until`]，核对线程数由调用方给（崩溃枚举用例的判段照 `SINGLEFS_CRASH_POINT_REPLAY_THREADS`）；库每一次读写现取现放，
/// 判一段时手里不握锁（[`check_blocks_where_until_judging_each_piece_with`]）。
///
/// # Errors
/// 库读写失败；判定向量超过 2^32 种。
#[allow(
    clippy::too_many_arguments,
    reason = "与 check_blocks_where_until 同一组参数，多一个线程数"
)]
pub fn check_blocks_where_until_with_threads(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    is_mine: &(dyn Fn(u64) -> bool + Sync),
    conclusions: &ConclusionsStoredUnder<'_>,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
    threads: usize,
) -> Result<CheckingTally, VerdictStoreError> {
    check_blocks_where_until_judging_each_piece_with(
        library,
        flow,
        plan,
        is_mine,
        conclusions,
        stop_is_requested,
        threads,
        &|crash_states_of_the_flow, point_index, local| {
            // 判一段的那个入口在判器一侧、登在判镜像的定义那一组里（它改了，两条路线判过的块一起换版本）；日志留在这里，不进那一组的键
            let point = &plan.points[point_index];
            let judged = judge_a_piece_of_a_block_on_the_cpu(
                crash_states_of_the_flow,
                flow.judged_root_index,
                flow.version_table(),
                point.ordinals.clone(),
                local,
            );
            eprintln!(
                "CRASH_AMPLIFICATION_BLOCK point={} block_start={} states={} red={}",
                point.name,
                judged.local.start,
                judged.local.end - judged.local.start,
                judged
                    .items_of_each_state
                    .iter()
                    .filter(|items| !items.is_empty())
                    .count()
            );
            judged
        },
    )
}

/// [`check_blocks_where_until_with_threads`] 的那一圈，判一段交 `judge_a_piece_on_the_shared_states`（单测给一个睡一会儿的判器，量判的时候
/// 别的线程取不取得到同一条连接的锁）：问库里哪几块要判、每判完一块写进库，各取一次库、取完就放；判一段的时候这条线程与各核对线程都不握锁。
///
/// # Errors
/// 库读写失败；判定向量超过 2^32 种。
#[allow(
    clippy::too_many_arguments,
    reason = "与 check_blocks_where_until_with_threads 同一组参数，多一个判一段的入口"
)]
fn check_blocks_where_until_judging_each_piece_with(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    is_mine: &(dyn Fn(u64) -> bool + Sync),
    conclusions: &ConclusionsStoredUnder<'_>,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
    threads: usize,
    judge_a_piece_on_the_shared_states: &JudgeAPieceOnTheSharedStates<'_>,
) -> Result<CheckingTally, VerdictStoreError> {
    let pending = pending_blocks(&library.take(), plan, conclusions, is_mine)?;
    if pending.is_empty() {
        // 没有要判的块：不建枚举的表（建一份要把原地覆写罩住的范围整段摆出来）
        return Ok(CheckingTally::default());
    }
    // 基镜像那一份逐盘自证槽数（CPU 参照；基镜像不是崩溃状态）：随每一块判器交回的那一份存，两槽全废盘面观察者在判段拿它比
    let slots_of_each_device_on_the_base_image =
        self_verified_system_configuration_slots_of_each_device(flow.base);
    check_pending_blocks_sharing_one_enumeration_table_of_the_flow(
        flow,
        plan,
        &pending,
        threads,
        stop_is_requested,
        judge_a_piece_on_the_shared_states,
        &mut |point_index, local, checked| {
            let block_key = plan.points[point_index].block_key(local);
            let recovery_words = (!checked.recovery_words_of_each_state.is_empty())
                .then_some(checked.recovery_words_of_each_state);
            let mut store = library.take();
            let numbers = verdict_vector_numbers_of_the_red_items_of_each_state(
                &mut store,
                &checked.items_of_each_state,
            )?;
            let verdict_block = VerdictBlock::new(numbers).expect("一块至少一个状态");
            conclusions.store_block(
                &mut store,
                point_index,
                &block_key,
                verdict_block,
                checked.violating,
                StoredAlongTheVerdicts {
                    count_vector: checked
                        .state_counts
                        .into_count_vector(local.end - local.start),
                    recovery_words,
                    judged_on: JudgingDevice::Cpu,
                    self_verified_system_configuration_slots: checked
                        .self_verified_system_configuration_slots_of_the_states
                        .handed_back_for_the_block(
                            local.end - local.start,
                            &slots_of_each_device_on_the_base_image,
                        ),
                },
            )
        },
    )
}

/// CPU 判段一段（一条核对线程）怎么判：状态取自各线程共用的那一份（`crash_states_of_the_flow`），点在计划里的次序、点内序号段。
type JudgeAPieceOnTheSharedStates<'judge> = dyn Fn(&crate::crash::CrashStatesOfTheReadBackFlow<'_>, usize, Range<u64>) -> CpuJudgedBlock
    + Sync
    + 'judge;

/// CPU 判段（[`check_blocks_where_until_with_threads`]）那一圈：枚举的表这一趟只建一份，`pending` 里的块经 [`check_pending_blocks_in_pieces`]
/// 切段、各核对线程交 `judge_a_piece_on_the_shared_states` 借同一份走自己那一段（批 14 的共享表）。不逐段各建一份：一份表要把原地覆写罩住的
/// 范围整段摆出来，用过的盘上重做 mkfs 那一类流一份逾 1.5 GiB，线程数那么多份同时在就撞内存上限
/// （单测 `the_cpu_judging_threads_of_one_block_share_one_enumeration_table_of_the_flow` 钉这一处，观察者录入段那一份另有同形的一条）。
///
/// # Errors
/// `store_a_block` 的错。
///
/// # Panics
/// 判一段的线程 panic。
fn check_pending_blocks_sharing_one_enumeration_table_of_the_flow(
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    pending: &[(usize, Range<u64>)],
    threads: usize,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
    judge_a_piece_on_the_shared_states: &JudgeAPieceOnTheSharedStates<'_>,
    store_a_block: &mut StoreACheckedBlock<'_>,
) -> Result<CheckingTally, VerdictStoreError> {
    let points: Vec<NamesOfACrashPoint> = plan
        .points
        .iter()
        .map(|point| NamesOfACrashPoint {
            name: point.name.clone(),
            path_number: point.path_number.clone(),
        })
        .collect();
    let tables_of_the_flow = crate::crash::TornImageTablesOfTheReadBackFlow::of_the_read_back_flow(
        flow.base,
        flow.writes,
        flow.segments,
    );
    let crash_states_of_the_flow = tables_of_the_flow.crash_states_planned_by(
        flow.base,
        flow.writes,
        flow.segments,
        &|segment_index, segment| plan.segment_expansion(flow, segment_index, segment),
        plan.segment_slices(),
    );
    check_pending_blocks_in_pieces(
        pending,
        &points,
        threads,
        stop_is_requested,
        &|point_index, local| {
            judge_a_piece_on_the_shared_states(&crash_states_of_the_flow, point_index, local)
        },
        store_a_block,
    )
}

// ============================== 取样那一趟的 CPU 结论入库（「能走两段的都走两段」） ==============================

/// 取样（`crate::crash_state_enumeration::sample_crash_states_on_the_cpu` 一族：实验装置从库里读回流、逐状态取样）那一趟在 CPU 上判过的状态里，
/// 整块都判到的块照 CPU 判器那一处（[`ConclusionsStoredUnder`]，`JudgedBy::CpuJudge`）存进库；块有一部分在取样序号段之外的不存（一块是库的
/// 原子单位，半块不存），库里这一处已经存着的块不再存。装置要的恢复报告字段库里的结论不装，判照旧逐状态现判、交回装置。
pub(crate) struct CpuJudgementsOfSampledBlocks<'plan> {
    plan: &'plan CrashPlan,
    conclusions: ConclusionsStoredUnder<'plan>,
    blocks: Vec<SampledBlock>,
    pub(crate) blocks_stored_here: u64,
    pub(crate) blocks_already_stored: u64,
}

/// 取样序号段整块罩住、还没存的一块：每个状态判到了就记下它的红项与恢复字。
struct SampledBlock {
    point_index: usize,
    local: Range<u64>,
    first_ordinal: u64,
    items_of_each_state: Vec<Option<Vec<String>>>,
    recovery_words_of_each_state: Vec<Option<RecoveryWordsOfState>>,
    state_counts: StateCountsOfABlock,
    states_noted: u64,
}

impl<'plan> CpuJudgementsOfSampledBlocks<'plan> {
    /// 计划里整块落在 `ordinals` 里、库里这一处还没存的块。逐块挑选与算块键在锁外，只有问库那一下取锁。
    ///
    /// # Errors
    /// 库读失败。
    pub(crate) fn for_the_ordinals(
        library: &crate::crash_library::CrashLibrary,
        plan: &'plan CrashPlan,
        conclusions: ConclusionsStoredUnder<'plan>,
        ordinals: &Range<u64>,
    ) -> Result<Self, VerdictStoreError> {
        let mut blocks = Vec::new();
        let mut blocks_already_stored = 0u64;
        for (point_index, point) in plan.points.iter().enumerate() {
            crate::crash_library::assert_this_thread_holds_no_library_lock("取样挑块、算块键");
            for local in point.blocks() {
                let first_ordinal = point.ordinals.start + local.start;
                let end_ordinal = point.ordinals.start + local.end;
                if first_ordinal == end_ordinal
                    || first_ordinal < ordinals.start
                    || end_ordinal > ordinals.end
                {
                    continue;
                }
                let block_key = point.block_key(&local);
                if conclusions.is_stored_for_reuse(&library.lock(), point_index, &block_key)? {
                    blocks_already_stored += 1;
                    continue;
                }
                let state_count =
                    usize::try_from(local.end - local.start).expect("一块装得进 usize");
                blocks.push(SampledBlock {
                    point_index,
                    local,
                    first_ordinal,
                    items_of_each_state: vec![None; state_count],
                    recovery_words_of_each_state: vec![None; state_count],
                    state_counts: StateCountsOfABlock::default(),
                    states_noted: 0,
                });
            }
        }
        Ok(Self {
            plan,
            conclusions,
            blocks,
            blocks_stored_here: 0,
            blocks_already_stored,
        })
    }

    /// 取样判过的一个状态记进来；它那一块的状态都判到了就存进库。不在任何一块里（块有一部分在序号段外、或库里已存）的状态不记。
    /// 只有存块那几下取库锁：逐状态记账不碰它。
    ///
    /// # Errors
    /// 库写失败；判定向量超过 2^32 种。
    pub(crate) fn note_a_state(
        &mut self,
        library: &crate::crash_library::CrashLibrary,
        ordinal: u64,
        publish: CrashPointReplayPublishOfState,
        judgement: &crate::crash::CrashStateJudgement,
    ) -> Result<(), VerdictStoreError> {
        let position = self.blocks.partition_point(|block| {
            block.first_ordinal + (block.local.end - block.local.start) <= ordinal
        });
        let Some(block) = self.blocks.get_mut(position) else {
            return Ok(());
        };
        if ordinal < block.first_ordinal {
            return Ok(());
        }
        let offset = usize::try_from(ordinal - block.first_ordinal).expect("块内序号装得进 usize");
        if block.items_of_each_state[offset].is_some() {
            return Ok(());
        }
        block.items_of_each_state[offset] = Some(judgement.red_items());
        block.recovery_words_of_each_state[offset] = Some(judgement.recovery_words());
        block.state_counts.count_the_cpu_judgement(judgement);
        block.state_counts.count_the_publish_of_a_state(publish);
        block.states_noted += 1;
        if block.states_noted < block.local.end - block.local.start {
            return Ok(());
        }
        let items_of_each_state: Vec<Vec<String>> = std::mem::take(&mut block.items_of_each_state)
            .into_iter()
            .map(|items| items.expect("这一块每个状态都记过"))
            .collect();
        let recovery_words: Vec<RecoveryWordsOfState> =
            std::mem::take(&mut block.recovery_words_of_each_state)
                .into_iter()
                .map(|words| words.expect("这一块每个状态都记过"))
                .collect();
        crate::crash_library::assert_this_thread_holds_no_library_lock("取样编判定向量、算块键");
        let mut encoded_verdicts = Vec::with_capacity(items_of_each_state.len());
        let mut violating = Vec::new();
        for (offset_in_the_block, items) in items_of_each_state.iter().enumerate() {
            if !items.is_empty() {
                violating.push(StateOffsetInBlock(
                    u64::try_from(offset_in_the_block).expect("块内序号装得进 u64"),
                ));
            }
            encoded_verdicts.push(encoded_verdict_of_the_red_items(items));
        }
        let point_index = block.point_index;
        let local = block.local.clone();
        let state_counts = std::mem::take(&mut block.state_counts);
        let block_key = self.plan.points[point_index].block_key(&local);
        let mut store = library.lock();
        let mut numbers = Vec::with_capacity(encoded_verdicts.len());
        for encoded_verdict in &encoded_verdicts {
            numbers.push(store.register_verdict_vector(encoded_verdict)?);
        }
        self.conclusions.store_block(
            &mut store,
            point_index,
            &block_key,
            VerdictBlock::new(numbers).expect("一块至少一个状态"),
            violating,
            StoredAlongTheVerdicts {
                count_vector: state_counts.into_count_vector(local.end - local.start),
                recovery_words: Some(recovery_words),
                judged_on: JudgingDevice::Cpu,
                // 取样交来的是判定、不带镜像，逐盘自证槽数交不回：这一块不算复用得上（`is_stored_for_reuse`），判段照判一遍补上
                self_verified_system_configuration_slots: None,
            },
        )?;
        self.blocks_stored_here += 1;
        Ok(())
    }
}

/// [`check_pending_blocks_in_pieces`] 写一块的那一份：点在 `points` 里的次序、块内序号、拼好的一块。
type StoreACheckedBlock<'store> =
    dyn FnMut(usize, &Range<u64>, CheckedBlock) -> Result<(), VerdictStoreError> + 'store;

/// 各段拼好的一块：逐状态红项、红状态的块内序号、逐状态恢复字（没算是空的）、逐条不变量判到与违例的状态数、逐状态的逐盘自证槽数（没算是空的）。
struct CheckedBlock {
    items_of_each_state: Vec<Vec<String>>,
    violating: Vec<StateOffsetInBlock>,
    recovery_words_of_each_state: Vec<RecoveryWordsOfState>,
    state_counts: StateCountsOfABlock,
    self_verified_system_configuration_slots_of_the_states:
        SelfVerifiedSystemConfigurationSlotsOfTheStates,
}

/// 一个崩溃点的名字与路径编号（核对时打进度行、记红状态的正文用）。
struct NamesOfACrashPoint {
    name: String,
    path_number: String,
}

/// [`check_blocks_where_until`] 的那一圈（不碰库与卡：判一段交 `judge_a_piece`，写一块交 `store_a_block`，单测用假的）：`pending` 里的块按次序，
/// 每领一块之前先问 `stop_is_requested`；一块的序号切成 `threads` 那么多段、一段一个线程交 `judge_a_piece`，
/// 收齐按段序拼回一块、当场交 `store_a_block`（点在 `points` 里的次序、块内序号、逐状态红项、红状态的块内序号）。
///
/// # Errors
/// `store_a_block` 的错。
///
/// # Panics
/// 判一段的线程 panic。
fn check_pending_blocks_in_pieces(
    pending: &[(usize, Range<u64>)],
    points: &[NamesOfACrashPoint],
    threads: usize,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
    judge_a_piece: &(dyn Fn(usize, Range<u64>) -> CpuJudgedBlock + Sync),
    store_a_block: &mut StoreACheckedBlock<'_>,
) -> Result<CheckingTally, VerdictStoreError> {
    // 块是库与 GPU 批的粒度，不改小；一块的序号切成线程数那么多段、一段一个线程（一块 65536 个状态、32 线程时一段 2048 个），
    // 判完按段序拼回一块、当场写库：中途被停，写进库的块下一次不再判。
    // 单测进程里从用例共用的那一份额度拿（[`crate::crash_library::InnerThreadsTaken`]），产品构建里照给：拿到几条就切几段
    let threads = crate::crash_library::InnerThreadsTaken::take(threads);
    let mut tally = CheckingTally::default();
    // 每一段判完打一行 `CRASH_AMPLIFICATION_BLOCK`；一段判得比进度间隔久时，等着收的这条线程隔一个间隔另打一行按挂钟的
    let started = std::time::Instant::now();
    let clock = || started.elapsed();
    let mut wall_clock_lines = WallClockProgressLines::every_progress_line_interval(clock());
    for (pending_index, (point_index, local)) in pending.iter().enumerate() {
        if stop_is_requested() {
            break;
        }
        let point = &points[*point_index];
        let piece_length = (local.end - local.start)
            .div_ceil(u64::try_from(threads.threads()).expect("段数装得进 u64"))
            .max(1);
        let mut pieces: Vec<Range<u64>> = Vec::new();
        let mut start = local.start;
        while start < local.end {
            let end = (start + piece_length).min(local.end);
            pieces.push(start..end);
            start = end;
        }
        let piece_count = pieces.len();
        let mut judged: Vec<CpuJudgedBlock> = std::thread::scope(|scope| {
            let (judged_sender, judged_receiver) = std::sync::mpsc::channel::<CpuJudgedBlock>();
            for range in pieces.iter().cloned() {
                let judged_sender = judged_sender.clone();
                scope.spawn(move || {
                    let piece = judge_a_piece(*point_index, range);
                    // 发不出去说明收的这条线程已经走了（它 panic 了），这一段的结果没人要
                    let _ = judged_sender.send(piece);
                });
            }
            // 只留核对线程手里的发送端：它们都退出之后（判完或 panic），下面的接收才结束；有线程 panic 时 scope 收尾把它往外传
            drop(judged_sender);
            receive_the_pieces_of_one_block(
                &judged_receiver,
                &mut wall_clock_lines,
                &clock,
                &BlockInProgress {
                    point_name: &point.name,
                    block_start: local.start,
                    piece_count,
                    blocks_finished: tally.blocks_checked,
                    pending_blocks: pending.len(),
                    states_finished: tally.states_checked,
                },
                &mut |line| eprintln!("{line}"),
            )
        });
        assert_eq!(
            judged.len(),
            piece_count,
            "块 {pending_index} 的每一段都判完交回了（核对线程 panic 时上面的 scope 已经往外传）"
        );
        judged.sort_by_key(|piece| piece.local.start);
        let mut items_of_each_state: Vec<Vec<String>> =
            Vec::with_capacity(usize::try_from(local.end - local.start).expect("一块装得进 usize"));
        let mut recovery_words_of_each_state: Vec<RecoveryWordsOfState> = Vec::new();
        let mut state_counts = StateCountsOfABlock::default();
        let mut self_verified_system_configuration_slots_of_the_states =
            SelfVerifiedSystemConfigurationSlotsOfTheStates::default();
        let mut next_expected = local.start;
        for piece in judged {
            assert_eq!(
                piece.local.start, next_expected,
                "块 {pending_index} 的段没有接上"
            );
            next_expected = piece.local.end;
            items_of_each_state.extend(piece.items_of_each_state);
            recovery_words_of_each_state.extend(piece.recovery_words_of_each_state);
            state_counts.absorb(piece.state_counts);
            self_verified_system_configuration_slots_of_the_states
                .absorb(piece.self_verified_system_configuration_slots_of_the_states);
            for (state_in_point, text) in piece.red_texts {
                if tally.red_texts.len() < RED_TEXTS_KEPT {
                    tally
                        .red_texts
                        .push((point.path_number.clone(), state_in_point, text));
                }
            }
        }
        assert_eq!(next_expected, local.end, "块 {pending_index} 的段没有拼满");
        let mut violating = Vec::new();
        for (offset, items) in items_of_each_state.iter().enumerate() {
            if !items.is_empty() {
                tally.red_states += 1;
                violating.push(StateOffsetInBlock(
                    u64::try_from(offset).expect("块内序号装得进 u64"),
                ));
            }
        }
        store_a_block(
            *point_index,
            local,
            CheckedBlock {
                items_of_each_state,
                violating,
                recovery_words_of_each_state,
                state_counts,
                self_verified_system_configuration_slots_of_the_states,
            },
        )?;
        tally.blocks_checked += 1;
        tally.states_checked += local.end - local.start;
    }
    Ok(tally)
}

/// 一块正在判的时候按挂钟那一行要报的：哪个点、块从点内第几个状态起、切成几段，这一趟已经判完几块（共几块）与几个状态。
struct BlockInProgress<'point> {
    point_name: &'point str,
    block_start: u64,
    piece_count: usize,
    blocks_finished: u64,
    pending_blocks: usize,
    states_finished: u64,
}

/// 收齐一块各段的判定：收到一段就记下；上一行之后 `clock` 走过整个进度间隔还没有新的一段，就交 `write_progress_line` 一行
/// `CRASH_AMPLIFICATION_BLOCK by=wall_clock …`（C611：一块判得比进度间隔久时照样隔一个间隔报一行）。核对线程都退出（发送端全丢掉）才交回，
/// 交回的是收到的各段（次序是到达的次序）。
fn receive_the_pieces_of_one_block(
    receiver: &std::sync::mpsc::Receiver<CpuJudgedBlock>,
    wall_clock_lines: &mut WallClockProgressLines,
    clock: &dyn Fn() -> std::time::Duration,
    block: &BlockInProgress<'_>,
    write_progress_line: &mut dyn FnMut(String),
) -> Vec<CpuJudgedBlock> {
    let mut arrived: Vec<CpuJudgedBlock> = Vec::with_capacity(block.piece_count);
    receive_until_the_senders_are_gone_with_wall_clock_lines(
        receiver,
        wall_clock_lines,
        clock,
        &mut |event| {
            match event {
            ReceivedOrWallClockLineDue::Received(piece) => arrived.push(piece),
            ReceivedOrWallClockLineDue::WallClockLineDue { elapsed } => write_progress_line(format!(
                "CRASH_AMPLIFICATION_BLOCK by=wall_clock point={} block_start={} pieces_finished={}/{} blocks_finished={}/{} states_finished={} elapsed_seconds={:.1}",
                block.point_name,
                block.block_start,
                arrived.len(),
                block.piece_count,
                block.blocks_finished,
                block.pending_blocks,
                block.states_finished,
                elapsed.as_secs_f64()
            )),
        }
        },
    );
    arrived
}

#[cfg(test)]
mod tests;
