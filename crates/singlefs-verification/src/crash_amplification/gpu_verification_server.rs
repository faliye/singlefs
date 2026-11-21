//! 经 GPU 批处理服务判：写待判行、等它判完、等的时候的进度行，判段的临时库登记（[`temporary_library_of_the_judging_stage_registered_with_the_real_service`]，
//! 不问服务收不收），连同派活计划那一行（[`placement_line_of_the_run`]）。

use super::*;

// ------------------------------ 待判行：判段写待判行、等 GPU 批处理服务判完（协议「CPU 一侧」「待判行」「强制现判」） ------------------------------
//
// 配了 GPU 的判段不在自己进程里持卡判：照排出的计划把这一趟要判的块（[`pending_blocks`]，复用口径）写成待判行（不在才写，
// `VerdictStore::put_the_gpu_pending_row_unless_present_or_raise_it_to_forced`：行已在而不强制、这一块要强制现判的改成强制），
// 等这些块在库上现算为判完，再照判段原来的读法读结论、核哨兵。登记、写行与等的时候都不问服务收不收：待判行不带判器的版本，
// 服务只照行的数据判（判法改了对不对靠对拍与复用哨兵抓）；服务读不了这一份就点名跳过，判段等不到的由统一任务服务照卡住停。
// 一块判完：它的待判行不在了（服务写结论的那一帧里删掉），且照续点口径判过（[`ConclusionsStoredUnder::is_judged_with_the_slots`]：结论连同
// 判器交回的逐盘自证槽数都在；红块也算判过，红不红由读结论那一步报）。待判行不在而块还没判过的（行被删了却没写结论）照不在才写补写一次。
// 强制现判那一块（复用哨兵抽中的，`VerdictStore::conclusion_block_is_forced_to_be_judged_again`）写成强制现判的待判行；它判完之后
// 照哨兵的判据核强制现判凭据（[`problem_of_the_forced_rejudging_by_the_gpu_verification_server`]）：凭据在、结论摘要与库里那一份相同，
// 才记成这一趟交来存过（`VerdictStore::note_the_conclusion_blocks_the_gpu_verification_server_judged_for_this_run`），之后照原来的
// [`ReuseSentinelsOfTheRun::problems_after_judging`] 比开跑前那一份；缺凭据、摘要不同都判红。

/// 判段等 GPU 批处理服务时两次在库上现算判完之间隔多久。
#[cfg(feature = "gpu")]
pub(super) const GPU_VERIFICATION_SERVER_RECKONING_INTERVAL: std::time::Duration =
    std::time::Duration::from_secs(2);

/// 判段等 GPU 批处理服务时进度行至少隔多久（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」的进度间隔）；
/// 还剩的块数没变的不打（[`gpu_verification_server_waiting_line_is_due`]）。
#[cfg(feature = "gpu")]
pub(super) const GPU_VERIFICATION_SERVER_WAITING_PROGRESS_INTERVAL: std::time::Duration =
    std::time::Duration::from_secs(5 * 60);

/// 判段等 GPU 批处理服务时的进度行：`<前缀> blocks_remaining=<还剩几块> states_remaining=<还剩几个状态> blocks_written=<这一趟写成待判行的块数> seconds_waited=<等了几秒>`。
pub const GPU_VERIFICATION_SERVER_WAITING_LINE_PREFIX: &str =
    "CRASH_AMPLIFICATION_WAITING_FOR_THE_GPU_VERIFICATION_SERVER";

/// 判段写的是哪一族待判行：只当临时库登记的标签（`crate::crash_library::register_the_temporary_library_with_the_real_service` 的 `label`，
/// 服务的日志里点名是哪一处登记的），不进任何键。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingRowFamilyOfTheGpuVerificationServer {
    /// 第 ① 段判器写的待判行（镜像判定）。
    ImageJudgement,
    /// 第 ② 段核对写的核对待判行。
    FactsVerifier,
    /// 给定状态表与观察者交回的盘没交卡的判段写的判表待判行（`crate::verdict_store::PackedTablesPendingRow`）。
    PackedTables,
}

impl PendingRowFamilyOfTheGpuVerificationServer {
    /// 临时库登记的标签。
    #[must_use]
    pub fn label_of_the_temporary_library_registration(self) -> &'static str {
        match self {
            Self::ImageJudgement => "image-judgement",
            Self::FactsVerifier => "facts-verifier",
            Self::PackedTables => "packed-tables",
        }
    }
}

/// 判段的库在不在临时 prune-server 里、在的话登记了没有（[`temporary_library_registered_when_the_library_is_in_a_temporary_service`]）。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "没编进 gpu 特性时判段不走 GPU 那一路，只有单测造它"
    )
)]
pub(crate) enum TemporaryLibraryOfTheJudgingStage {
    /// 库不在临时服务里（真库）：照常写待判行，不登记。
    NotInATemporaryService,
    /// 库在临时服务里：已在真服务里登记，守卫放手时放掉登记。装箱：登记守卫比不登记的那一支大得多（clippy `large_enum_variant`）。
    Registered(
        #[allow(
            dead_code,
            reason = "判段只持有它到等完：丢掉时放掉登记，产品这一路不读它"
        )]
        Box<crate::crash_library::TemporaryLibraryRegisteredWithTheRealService>,
    ),
}

/// 判段的库在临时 prune-server 里（`temporary_service`，`VerdictStore::temporary_service_of_the_library` 交的）时，写待判行之前照 `register`
/// 在真服务里登记那份临时库，不问 GPU 批处理服务收不收：服务读不了就点名跳过，判段等不到的由统一任务服务照卡住停。
/// 库不在临时服务里的不登记。
///
/// # Errors
/// 登记不成（`register` 交回的错：判段这一侧先拒的、连不上真服务、认领不成）。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "没编进 gpu 特性时判段不走 GPU 那一路，只有单测照替身真服务调它"
    )
)]
pub(crate) fn temporary_library_registered_when_the_library_is_in_a_temporary_service(
    temporary_service: Option<crate::crash_library::TemporaryServiceOfTheLibrary>,
    register: &dyn Fn(
        &crate::crash_library::TemporaryServiceOfTheLibrary,
    ) -> Result<
        crate::crash_library::TemporaryLibraryRegisteredWithTheRealService,
        String,
    >,
) -> Result<TemporaryLibraryOfTheJudgingStage, String> {
    let Some(temporary_service) = temporary_service else {
        return Ok(TemporaryLibraryOfTheJudgingStage::NotInATemporaryService);
    };
    register(&temporary_service)
        .map(|registered| TemporaryLibraryOfTheJudgingStage::Registered(Box::new(registered)))
}

/// 待判行的流键（`crate::verdict_store::GpuPendingStreamKey`）：录下的流程的摘要、打包判表的摘要与结论版本键带标签接起来取 SHA-256，
/// 都是判的输入数据，判器的代码不进。块号是这条流程排出的计划里的块序，流程进键，块号才只指一块。
#[must_use]
pub fn gpu_pending_stream_key(
    recorded_flow_digest: &[u8; DIGEST_BYTES],
    packed_judge_tables_digest: &[u8; DIGEST_BYTES],
    conclusion_version: &ConclusionVersion,
) -> crate::verdict_store::GpuPendingStreamKey {
    let mut message = b"singlefs gpu-pending stream key\n".to_vec();
    message.extend_from_slice(recorded_flow_digest);
    message.extend_from_slice(packed_judge_tables_digest);
    message.extend_from_slice(&conclusion_version.0);
    crate::verdict_store::GpuPendingStreamKey(sha256_digest(&message))
}

/// 判段怎么等 GPU 批处理服务：进度行至少隔多久，两次现算之间做什么（产品睡 [`GPU_VERIFICATION_SERVER_RECKONING_INTERVAL`]；
/// 单测在这里替服务把待判行判掉写回）。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "没编进 gpu 特性时判段不走 GPU 那一路，只有单测造它"
    )
)]
pub(crate) struct WaitingForTheGpuVerificationServer<'wait> {
    pub(crate) progress_interval: std::time::Duration,
    pub(crate) between_two_reckonings: &'wait dyn Fn(&LibraryTakenAtEachReadOrWrite<'_>),
}

/// 等 GPU 批处理服务的这一圈打不打进度行：离上一行（没打过时离开始等）满 `progress_interval`，且还剩的块数与上一行报的
/// （没打过时是写成待判行的块数）不同。块数不动就不打：服务不在跑、或认领之后却一直不判（配不出这条流）时日志不涨，
/// 交统一任务服务照卡住停（`task-server/task_server_lib.py` 异常表 stalled 一行）。
pub(super) fn gpu_verification_server_waiting_line_is_due(
    since_the_last_line: std::time::Duration,
    progress_interval: std::time::Duration,
    blocks_remaining_now: usize,
    blocks_remaining_at_the_last_line: usize,
) -> bool {
    since_the_last_line >= progress_interval
        && blocks_remaining_now != blocks_remaining_at_the_last_line
}

/// 待判行（判器那一族与核对那一族）的入池序号：此刻自 UNIX 纪元的纳秒数，服务按它排先后。
pub(super) fn pool_entry_number_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| {
            u64::try_from(since.as_nanos()).expect("自 UNIX 纪元的纳秒数装得进 u64：五百多年")
        })
}

/// 写成待判行、等服务判完的一块。
#[derive(Clone, Debug)]
struct BlockWaitedForOnTheServer {
    point_index: usize,
    local: Range<u64>,
    block_number: u64,
    stream: crate::verdict_store::GpuPendingStreamKey,
    /// 写下（或库里已在）的那一行：强制现判凭据的键要它的入池序号。
    row: crate::verdict_store::GpuPendingRow,
}

/// 强制现判那一块判完之后核强制现判凭据（协议「强制现判」：「CPU 一侧照今天的哨兵判据，核这一行在、结论摘要与库里那一份相同」）：
/// 凭据不在、库里没有这一块的结论、或凭据里的结论摘要与库里那一份（[`ConclusionSpelledOutInVerdictVectors::conclusion_digest`]）不同，
/// 交回那一句红（点名块号与块键）；都对交 `None`。
///
/// # Errors
/// 库读失败；凭据那一行不合布局。
fn problem_of_the_forced_rejudging_by_the_gpu_verification_server(
    store: &VerdictStore,
    waited: &BlockWaitedForOnTheServer,
    kind: &ConclusionKind,
) -> Result<Option<String>, VerdictStoreError> {
    let block_key_text = block_key_named_in_the_sentinel_report(&waited.row.recorded_block);
    let Some(rejudged) = store.read_gpu_rejudged_row(
        waited.row.pool_entry_number,
        &waited.stream,
        waited.block_number,
    )?
    else {
        return Ok(Some(format!(
            "复用哨兵：强制现判的块 {}（块键 {block_key_text}）判完了，库里却没有它的强制现判凭据（入池序号 {}）：服务没现判它，或现判了没留凭据",
            waited.block_number, waited.row.pool_entry_number
        )));
    };
    let Some(stored) = store.read_conclusion_block(
        &waited.row.recorded_block,
        kind,
        &waited.row.conclusion_version,
    )?
    else {
        return Ok(Some(format!(
            "复用哨兵：强制现判的块 {}（块键 {block_key_text}）有强制现判凭据，库里却没有它的结论",
            waited.block_number
        )));
    };
    let digest_in_the_library =
        ConclusionSpelledOutInVerdictVectors::of_the_stored_block(store, &stored)
            .conclusion_digest();
    if rejudged.conclusion_digest != digest_in_the_library {
        return Ok(Some(format!(
            "复用哨兵：强制现判的块 {}（块键 {block_key_text}）强制现判凭据里的结论摘要 {} 与库里那一份 {} 不同",
            waited.block_number,
            hexadecimal_text(&rejudged.conclusion_digest[..8]),
            hexadecimal_text(&digest_in_the_library[..8])
        )));
    }
    Ok(None)
}

/// 判段等 GPU 批处理服务时的进度行（[`GPU_VERIFICATION_SERVER_WAITING_LINE_PREFIX`]）。
fn gpu_verification_server_waiting_line(
    blocks_remaining: usize,
    states_remaining: u64,
    blocks_written: usize,
    seconds_waited: f64,
) -> String {
    format!(
        "{GPU_VERIFICATION_SERVER_WAITING_LINE_PREFIX} blocks_remaining={blocks_remaining} states_remaining={states_remaining} blocks_written={blocks_written} seconds_waited={seconds_waited:.0}"
    )
}

/// 第 ① 段配了 GPU 的那一路（文件头这一节上面「待判行」那一段）：这台机器领的块里要判的（[`pending_blocks`]，复用口径）排成待判行、写下，
/// 等它们在库上现算为判完（待判行不在了、续点口径判过）；写之前与等的时候都不问服务收不收（待判行不带判器的版本，服务只照行的数据判）；
/// 进度行照 [`gpu_verification_server_waiting_line_is_due`] 打；被叫停就不再等，交回判完的那几块。
/// 交回的账照服务替这一趟判完的块记（判了几块、几个状态、几个红状态与前几条红的正文，红的照库里读回的违例），强制现判那一块也在里面
/// （调用方照原来的样子从现判的账里扣掉它）。
///
/// # Errors
/// 结论不是三种结论那一套（判法单元指纹那一套是流水线各段单测直接调的，不经服务）；要写待判行的块的结论版本照判不入库
/// （服务写不进这个进程里的行）；这两样都在一行待判行都没写时交回；库读写失败；强制现判那一块缺凭据或结论摘要不同。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "没编进 gpu 特性时判段不走 GPU 那一路，只有单测替服务判掉待判行时调它"
    )
)]
#[allow(
    clippy::too_many_arguments,
    reason = "库、计划、录下的流程摘要、判表摘要、分工、结论记在哪、怎么等、叫停，各是各的东西"
)]
#[allow(
    clippy::too_many_lines,
    reason = "排待判行、写待判行、等判完、核强制现判凭据、记账，是这一路的全部步骤，拆开要把那几块来回传"
)]
pub(crate) fn judge_my_blocks_through_the_pending_rows(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    plan: &CrashPlan,
    recorded_flow_digest: &[u8; DIGEST_BYTES],
    packed_judge_tables_digest: &[u8; DIGEST_BYTES],
    is_mine: &dyn Fn(u64) -> bool,
    conclusions: &ConclusionsStoredUnder<'_>,
    waiting: &WaitingForTheGpuVerificationServer<'_>,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<CheckingTally, String> {
    let ConclusionsStoredUnder::TheConclusionKind {
        kind,
        version_of_each_point,
    } = conclusions
    else {
        return Err(
            "待判行只收三种结论那一套（判法单元指纹那一套是流水线各段单测直接调的，不经 GPU 批处理服务）"
                .to_string(),
        );
    };
    let read_error = |error: VerdictStoreError| format!("待判行：读写判定库失败：{error:?}");
    let pending =
        pending_blocks(&library.take(), plan, conclusions, is_mine).map_err(read_error)?;
    let mut block_number_of = std::collections::HashMap::new();
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            block_number_of.insert((point_index, local.start), block_sequence);
            block_sequence += 1;
        }
    }
    let versions_kept_out = library.take().versions_kept_out_of_the_library();
    // 先排后写：每一块的待判行排好（照判不入库的在这里交错），排齐了才写第一行
    let mut rows_to_write = Vec::with_capacity(pending.len());
    for (point_index, local) in pending {
        let point = &plan.points[point_index];
        let key = point.block_key(&local);
        let conclusion_version = ConclusionsStoredUnder::conclusion_version_of_the_point(
            version_of_each_point,
            point_index,
        );
        if versions_kept_out.contains(&conclusion_version.0) {
            return Err(format!(
                "点 {point_index} 块 {local:?} 的结论版本照判不入库：GPU 批处理服务写不进这个进程里的行，不写待判行"
            ));
        }
        let stream = gpu_pending_stream_key(
            recorded_flow_digest,
            packed_judge_tables_digest,
            &conclusion_version,
        );
        let block_number = *block_number_of
            .get(&(point_index, local.start))
            .expect("要判的块都出自这份计划：上面照同一份计划给每一块编了号");
        let forced = if conclusions.is_forced_to_be_judged_again(&library.take(), point_index, &key)
        {
            crate::verdict_store::ForcedRejudging::Forced
        } else {
            crate::verdict_store::ForcedRejudging::NotForced
        };
        let row = crate::verdict_store::GpuPendingRow {
            recorded_block: key,
            recorded_flow_digest: *recorded_flow_digest,
            packed_judge_tables_digest: *packed_judge_tables_digest,
            state_ordinals: point.ordinals.start + local.start..point.ordinals.start + local.end,
            conclusion_version,
            forced,
            pool_entry_number: pool_entry_number_now(),
        };
        rows_to_write.push((point_index, local, block_number, stream, row));
    }
    let mut waited_for = Vec::with_capacity(rows_to_write.len());
    for (point_index, local, block_number, stream, row) in rows_to_write {
        let row_in_the_library = match library
            .take()
            .put_the_gpu_pending_row_unless_present_or_raise_it_to_forced(
                &stream,
                block_number,
                &row,
            )
            .map_err(read_error)?
        {
            crate::verdict_store::GpuPendingRowPut::Written
            | crate::verdict_store::GpuPendingRowPut::RaisedToForced { replaced: _ } => row,
            crate::verdict_store::GpuPendingRowPut::AlreadyPresent(present) => present,
        };
        waited_for.push(BlockWaitedForOnTheServer {
            point_index,
            local,
            block_number,
            stream,
            row: row_in_the_library,
        });
    }
    let blocks_written = waited_for.len();
    let waiting_started = std::time::Instant::now();
    let mut last_progress_line = waiting_started;
    let mut blocks_remaining_at_the_last_line = blocks_written;
    let mut remaining = waited_for.clone();
    let mut judged = Vec::with_capacity(waited_for.len());
    // 迭代上界：服务判完一块就少一块；被叫停就出圈。服务一直不判就一直等，块数不动不打进度行（日志不涨，统一任务服务照卡住停）
    while !remaining.is_empty() {
        let rows_now = library
            .take()
            .read_gpu_pending_rows(
                &remaining
                    .iter()
                    .map(|waited| (waited.stream, waited.block_number))
                    .collect::<Vec<_>>(),
            )
            .map_err(read_error)?;
        let mut still_remaining = Vec::with_capacity(remaining.len());
        for (waited, row_now) in remaining.into_iter().zip(rows_now) {
            if row_now.is_some() {
                still_remaining.push(waited);
                continue;
            }
            let store = library.take();
            if conclusions
                .is_judged_with_the_slots(&store, waited.point_index, &waited.row.recorded_block)
                .map_err(read_error)?
            {
                judged.push(waited);
                continue;
            }
            // 待判行不在了却还没判过：照不在才写补写，接着等
            store
                .put_the_gpu_pending_row_unless_present_or_raise_it_to_forced(
                    &waited.stream,
                    waited.block_number,
                    &waited.row,
                )
                .map_err(read_error)?;
            still_remaining.push(waited);
        }
        remaining = still_remaining;
        if remaining.is_empty() || stop_is_requested() {
            break;
        }
        if gpu_verification_server_waiting_line_is_due(
            last_progress_line.elapsed(),
            waiting.progress_interval,
            remaining.len(),
            blocks_remaining_at_the_last_line,
        ) {
            println!(
                "{}",
                gpu_verification_server_waiting_line(
                    remaining.len(),
                    remaining
                        .iter()
                        .map(|waited| waited.local.end - waited.local.start)
                        .sum(),
                    blocks_written,
                    waiting_started.elapsed().as_secs_f64()
                )
            );
            last_progress_line = std::time::Instant::now();
            blocks_remaining_at_the_last_line = remaining.len();
        }
        (waiting.between_two_reckonings)(library);
    }
    let mut tally = CheckingTally::default();
    let mut forced_rejudging_problems = Vec::new();
    let mut judged_for_this_run = Vec::with_capacity(judged.len());
    for waited in &judged {
        let store = library.take();
        let key = (
            waited.row.recorded_block.clone(),
            kind.clone(),
            waited.row.conclusion_version,
        );
        match waited.row.forced {
            crate::verdict_store::ForcedRejudging::Forced => {
                match problem_of_the_forced_rejudging_by_the_gpu_verification_server(
                    &store, waited, kind,
                )
                .map_err(read_error)?
                {
                    Some(problem) => forced_rejudging_problems.push(problem),
                    None => judged_for_this_run.push(key),
                }
            }
            crate::verdict_store::ForcedRejudging::NotForced => judged_for_this_run.push(key),
        }
        let point = &plan.points[waited.point_index];
        let violations = conclusions
            .violations_in_block(&store, waited.point_index, &waited.row.recorded_block)
            .map_err(read_error)?;
        tally.blocks_checked += 1;
        tally.states_checked += waited.local.end - waited.local.start;
        tally.red_states += u64::try_from(violations.len()).expect("一块的违例数装得进 u64");
        for red in red_states_of_a_block(&store, point, &waited.local, &violations) {
            if tally.red_texts.len() < RED_TEXTS_KEPT {
                tally
                    .red_texts
                    .push((red.path, red.state_in_point, red.items.join("；")));
            }
        }
    }
    library
        .take()
        .note_the_conclusion_blocks_the_gpu_verification_server_judged_for_this_run(
            judged_for_this_run,
        );
    if !forced_rejudging_problems.is_empty() {
        return Err(forced_rejudging_problems.join("；"));
    }
    Ok(tally)
}

/// 判段的库在不在临时 prune-server 里（[`crate::verdict_store::VerdictStore::temporary_service_of_the_library`]），在就在真服务里登记
/// （[`temporary_library_registered_when_the_library_is_in_a_temporary_service`]，登记照
/// [`crate::crash_library::register_the_temporary_library_with_the_real_service`]，标签是 `family` 那一族待判行的标签）。
///
/// # Errors
/// 同 [`temporary_library_registered_when_the_library_is_in_a_temporary_service`]。
#[cfg(feature = "gpu")]
pub(super) fn temporary_library_of_the_judging_stage_registered_with_the_real_service(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    family: PendingRowFamilyOfTheGpuVerificationServer,
) -> Result<TemporaryLibraryOfTheJudgingStage, String> {
    let temporary_service = library.take().temporary_service_of_the_library();
    temporary_library_registered_when_the_library_is_in_a_temporary_service(
        temporary_service,
        &|service| {
            crate::crash_library::register_the_temporary_library_with_the_real_service(
                service,
                family.label_of_the_temporary_library_registration(),
            )
        },
    )
}

/// 第 ① 段配了 GPU 的入口（崩溃放量的判段与崩溃枚举用例的判段都经它）：判段的库在临时 prune-server 里的，先在真服务里登记那份临时库
/// （[`temporary_library_of_the_judging_stage_registered_with_the_real_service`]，等完放掉登记，不问服务收不收）；
/// 再把这台机器领的要判的块写成待判行、等服务判完（[`judge_my_blocks_through_the_pending_rows`]；两次现算之间睡
/// [`GPU_VERIFICATION_SERVER_RECKONING_INTERVAL`]，进度行至少隔 [`GPU_VERIFICATION_SERVER_WAITING_PROGRESS_INTERVAL`]、块数不动不打）。
/// 这个进程不持卡判、不要卡：交回的账里用了几张卡是 0，判了多少照服务替这一趟判完的块记，判的挂钟是等的挂钟。
/// `recorded_flow_digest` 是判段读回的那一份流在库里的摘要（清单里的那一个），写进待判行，服务照它读回流。
///
/// # Errors
/// 登记不成（判段这一侧先拒的：指到真服务、前缀不合、连不上；连不上真服务、认领不成的照判定库客户端的退出码退出进程）；
/// 同 [`judge_my_blocks_through_the_pending_rows`]。
#[cfg(feature = "gpu")]
#[allow(
    clippy::too_many_arguments,
    reason = "库、录下的流程摘要、计划、判表、分工、结论记在哪、叫停，各是各的东西"
)]
pub(crate) fn judge_my_blocks_on_gpu(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    recorded_flow_digest: &[u8; DIGEST_BYTES],
    plan: &CrashPlan,
    tables: &PackedJudgeTablesOfTheRun,
    is_mine: &dyn Fn(u64) -> bool,
    conclusions: &ConclusionsStoredUnder<'_>,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<(CheckingTally, GpuJudgeOutcome), String> {
    let temporary_library =
        temporary_library_of_the_judging_stage_registered_with_the_real_service(
            library,
            PendingRowFamilyOfTheGpuVerificationServer::ImageJudgement,
        )?;
    let waiting_started = Instant::now();
    let tally = judge_my_blocks_through_the_pending_rows(
        library,
        plan,
        recorded_flow_digest,
        &sha256_digest(&tables.to_bytes()),
        is_mine,
        conclusions,
        &WaitingForTheGpuVerificationServer {
            progress_interval: GPU_VERIFICATION_SERVER_WAITING_PROGRESS_INTERVAL,
            between_two_reckonings: &|_library| {
                std::thread::sleep(GPU_VERIFICATION_SERVER_RECKONING_INTERVAL);
            },
        },
        stop_is_requested,
    )?;
    drop(temporary_library);
    let outcome = GpuJudgeOutcome {
        cards_used: 0,
        blocks_judged: tally.blocks_checked,
        states_judged: tally.states_checked,
        red_states: tally.red_states,
        compile_seconds: 0.0,
        judge_seconds: waiting_started.elapsed().as_secs_f64(),
        seconds_storing: 0.0,
        seconds_decoding: 0.0,
        cards: Vec::new(),
    };
    Ok((tally, outcome))
}

/// 没编进 `gpu` 特性走不到这里（选路走不到 GPU：崩溃放量是 [`computation_routes`]，崩溃枚举用例是它的判段选路）。
#[cfg(not(feature = "gpu"))]
#[allow(
    clippy::too_many_arguments,
    reason = "与编进 gpu 特性的那一份同一个签名"
)]
pub(crate) fn judge_my_blocks_on_gpu(
    _library: &LibraryTakenAtEachReadOrWrite<'_>,
    _recorded_flow_digest: &[u8; DIGEST_BYTES],
    _plan: &CrashPlan,
    _tables: &PackedJudgeTablesOfTheRun,
    _is_mine: &dyn Fn(u64) -> bool,
    _conclusions: &ConclusionsStoredUnder<'_>,
    _stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<(CheckingTally, GpuJudgeOutcome), String> {
    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features gpu）".to_string())
}

// ------------------------------ 只排派活计划：只数两段要判、要核的块与状态 ------------------------------

/// 只排派活计划那一趟打的那一行的头。分项驱动 `scripts/crash/crash-amplification-item.sh` 读它（`decide_where_to_run`）：
/// 跑不跑只看排计划那一趟的日志里有没有这一行（一条用例几条流各一行；一行都没有就不跑、照日志分类退），不看各栏的值；
/// 有这一行就把各流的 `pending_states` 加起来打进 `CRASH_AMPLIFICATION_DECISION` 行，只打印、不拿来定跑不跑。
/// 所以第 ① 段都复用（`pending_states=0`）而第 ② 段还要核的流照样跑。
pub const PLACEMENT_LINE_PREFIX: &str = "CRASH_AMPLIFICATION_PLACEMENT";

/// 只排派活计划那一趟数出来的（[`placement_line_of_the_run`] 照它拼那一行）：两段在这台机器领的块里还要交 GPU 批处理服务的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlacementOfTheRun {
    /// 第 ① 段要写待判行的块数与状态数（[`pending_blocks`]，复用口径）。
    pub pending_blocks: usize,
    pub pending_states: u64,
    /// 录入时库里已判、因此第 ① 段不排的态数。
    pub states_reused: u64,
    /// GPU 判表上卡之后占多少显存（MiB，向上取整）。
    pub tables_mebibytes: u64,
    /// 第 ② 段在这一把核对键下还没核过、要写核对待判行的块数与状态数（[`blocks_not_yet_verified_under_the_verifier_row_key`]）。
    pub verifier_pending_blocks: usize,
    pub verifier_pending_states: u64,
}

/// 只排派活计划那一行（[`PLACEMENT_LINE_PREFIX`] 起头），字段与次序：
/// `judge=gpu pending_blocks=<n> pending_states=<n> states_reused=<n> tables_mebibytes=<n> verifier_pending_blocks=<n> verifier_pending_states=<n>`。
#[must_use]
pub fn placement_line_of_the_run(placement: &PlacementOfTheRun) -> String {
    format!(
        "{PLACEMENT_LINE_PREFIX} judge=gpu pending_blocks={} pending_states={} states_reused={} tables_mebibytes={} verifier_pending_blocks={} verifier_pending_states={}",
        placement.pending_blocks,
        placement.pending_states,
        placement.states_reused,
        placement.tables_mebibytes,
        placement.verifier_pending_blocks,
        placement.verifier_pending_states
    )
}

/// 只排派活计划那一趟的那一行（[`placement_line_of_the_run`]）：这台机器这一趟第 ① 段要判的块数与状态数（[`pending_blocks`]，复用口径）、
/// 录入时库里已判因此不排的态数（`states_reused`）、GPU 判表多大，与第 ② 段在核对键 `verifier_row_key`（读回的事实表摘要，
/// [`facts_verifier_row_key`]）下还没核过的块数与状态数（[`blocks_not_yet_verified_under_the_verifier_row_key`]，与第 ② 段真跑时决定写哪些核对待判行同一个判断）。
/// 卡归 GPU 批处理服务、任务进程不拿卡：这一趟不建判器、不试判。
///
/// # Errors
/// 库读失败。
pub(crate) fn placement_line_of_the_pending_blocks(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    plan: &CrashPlan,
    tables: &PackedJudgeTablesOfTheRun,
    is_mine: &dyn Fn(u64) -> bool,
    conclusions: &ConclusionsStoredUnder<'_>,
    states_reused: u64,
    verifier_row_key: &[u8; DIGEST_BYTES],
) -> Result<String, String> {
    let read_error = |error: VerdictStoreError| format!("读判定块失败：{error:?}");
    let pending =
        pending_blocks(&library.take(), plan, conclusions, is_mine).map_err(read_error)?;
    let verifier = blocks_not_yet_verified_under_the_verifier_row_key(
        library,
        plan,
        verifier_row_key,
        is_mine,
    )
    .map_err(read_error)?;
    Ok(placement_line_of_the_run(&PlacementOfTheRun {
        pending_blocks: pending.len(),
        pending_states: pending
            .iter()
            .map(|(_, local)| local.end - local.start)
            .sum(),
        states_reused,
        tables_mebibytes: mebibytes_of_the_packed_judge_tables(tables),
        verifier_pending_blocks: verifier.not_yet_verified.len(),
        verifier_pending_states: verifier.states_not_yet_verified(),
    }))
}

/// GPU 判表上卡之后占多少显存（MiB，向上取整）。
#[cfg(feature = "gpu")]
fn mebibytes_of_the_packed_judge_tables(tables: &PackedJudgeTablesOfTheRun) -> u64 {
    tables.mebibytes()
}

/// 没编进 `gpu` 特性造不出 GPU 判表（[`PackedJudgeTablesOfTheRun`] 是空的枚举）。
#[cfg(not(feature = "gpu"))]
fn mebibytes_of_the_packed_judge_tables(tables: &PackedJudgeTablesOfTheRun) -> u64 {
    match *tables {}
}

#[cfg(test)]
mod tests;
