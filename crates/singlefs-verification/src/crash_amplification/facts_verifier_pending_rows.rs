//! 核对待判行：崩溃放量第 ② 段写核对待判行、等服务核完（`verify_my_blocks_through_the_facts_verifier_pending_rows`），
//! 待核块的材料与核对帧（[`facts_verifier_frame_of_a_pending_block`]）。

use super::*;

// ------------------------------ 第 ② 段配了 GPU：判段写核对待判行、等 GPU 批处理服务核完 ------------------------------
//
// 配了 GPU 的第 ② 段不在自己进程里持卡核：这台机器领的块里在这一把核对键（[`facts_verifier_row_key`]：读回的事实表摘要）下
// 还没核过的（[`blocks_not_yet_verified_under_the_verifier_row_key`]）写成核对待判行（不在才写，`VerdictStore::put_the_facts_verifier_pending_row_unless_present`；
// 不问服务收不收：核对待判行不带核对器的版本，服务只照行的数据核），等它们在库上现算为核完
// （核对待判行不在了、核对表里这一块在这把核对键下有那一行），再照核对键读回核对行记账；第 ③ 段照同一把核对键比。
// 核对待判行不在而核对行也不在的（行被删了却没写核对行）照不在才写补写一次。等的那一圈与第 ① 段同一套
// （[`WaitingForTheGpuVerificationServer`]：进度行只在还剩的块数变了才打，被叫停就不再等）。
// 服务那一侧照调的三样：[`material_of_a_facts_verifier_pending_block`] 读回核对材料，[`facts_verifier_pending_block_is_verified_in_the_library`]
// 在库上现算一块核没核完，[`facts_verifier_frame_of_a_pending_block`] 把核对结果编成写回的那一帧。

/// 核对待判行的流键（`crate::verdict_store::FactsVerifierPendingStreamKey`）：录下的流程的摘要与事实表摘要带标签接起来取 SHA-256，
/// 都是核的输入数据，核对器的代码不进。块号是这条流程排出的计划里的块序，流程进键，块号才只指一块。
#[must_use]
pub fn facts_verifier_pending_stream_key(
    recorded_flow_digest: &[u8; DIGEST_BYTES],
    facts_digest: &[u8; DIGEST_BYTES],
) -> crate::verdict_store::FactsVerifierPendingStreamKey {
    let mut message = b"singlefs facts-verifier-pending stream key\n".to_vec();
    message.extend_from_slice(recorded_flow_digest);
    message.extend_from_slice(facts_digest);
    crate::verdict_store::FactsVerifierPendingStreamKey(sha256_digest(&message))
}

/// 判段等 GPU 批处理服务核完时的进度行：`<前缀> blocks_remaining=<还剩几块> states_remaining=<还剩几个状态> blocks_written=<这一趟写成核对待判行的块数> seconds_waited=<等了几秒>`。
pub const FACTS_VERIFIER_WAITING_LINE_PREFIX: &str =
    "CRASH_AMPLIFICATION_WAITING_FOR_THE_FACTS_VERIFIER_ON_THE_GPU_VERIFICATION_SERVER";

/// 判段等 GPU 批处理服务核完时的进度行（[`FACTS_VERIFIER_WAITING_LINE_PREFIX`]）。
fn facts_verifier_waiting_line(
    blocks_remaining: usize,
    states_remaining: u64,
    blocks_written: usize,
    seconds_waited: f64,
) -> String {
    format!(
        "{FACTS_VERIFIER_WAITING_LINE_PREFIX} blocks_remaining={blocks_remaining} states_remaining={states_remaining} blocks_written={blocks_written} seconds_waited={seconds_waited:.0}"
    )
}

/// 写成核对待判行、等服务核完的一块：块号与库里的那一行（写下的，或别的趟写的）。
#[derive(Clone, Debug)]
struct BlockVerifiedOnTheServer {
    block_number: u64,
    row: crate::verdict_store::FactsVerifierPendingRow,
}

/// 第 ② 段这台机器领的一块在这一把核对键下还没核过（要写核对待判行）：块号（这条流程排出的计划里的块序）、它的点与点内区间。
#[derive(Clone, Debug)]
pub(super) struct BlockNotYetVerified {
    block_number: u64,
    point_index: usize,
    local: Range<u64>,
}

/// 第 ② 段这台机器领的块在这一把核对键下核到了哪一步（[`blocks_not_yet_verified_under_the_verifier_row_key`]）。
pub(super) struct BlocksOfThisHostUnderTheVerifierRowKey {
    /// 还没核过的，按块号次序：配了 GPU 的第 ② 段给它们写核对待判行，只排派活计划那一行数它们。
    pub(super) not_yet_verified: Vec<BlockNotYetVerified>,
    /// 库里已有核对行的块数。
    already_verified: u64,
}

impl BlocksOfThisHostUnderTheVerifierRowKey {
    /// 还没核过的块的状态数。
    pub(super) fn states_not_yet_verified(&self) -> u64 {
        self.not_yet_verified
            .iter()
            .map(|block| block.local.end - block.local.start)
            .sum()
    }
}

/// 第 ② 段这台机器领的块（`is_mine`，按计划里的块序编号）在核对键 `verifier_row_key` 下核没核过：核对表里这一块在这把键下有那一行就是核过。
/// 配了 GPU 的第 ② 段决定给哪些块写核对待判行（[`verify_my_blocks_through_the_facts_verifier_pending_rows`]）与只排派活计划那一行数
/// 第 ② 段还要核多少（[`placement_line_of_the_pending_blocks`]）都照这一个判断。库每读一块现取现放。
///
/// # Errors
/// 库读失败。
pub(super) fn blocks_not_yet_verified_under_the_verifier_row_key(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    plan: &CrashPlan,
    verifier_row_key: &[u8; DIGEST_BYTES],
    is_mine: &dyn Fn(u64) -> bool,
) -> Result<BlocksOfThisHostUnderTheVerifierRowKey, VerdictStoreError> {
    let mut blocks = BlocksOfThisHostUnderTheVerifierRowKey {
        not_yet_verified: Vec::new(),
        already_verified: 0,
    };
    let mut block_number = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let number_of_this_block = block_number;
            block_number += 1;
            if !is_mine(number_of_this_block) {
                continue;
            }
            if library
                .take()
                .read_verifier_block(&point.block_key(&local), verifier_row_key)?
                .is_some()
            {
                blocks.already_verified += 1;
                continue;
            }
            blocks.not_yet_verified.push(BlockNotYetVerified {
                block_number: number_of_this_block,
                point_index,
                local,
            });
        }
    }
    Ok(blocks)
}

/// 第 ② 段配了 GPU 的那一路（文件头这一节上面「第 ② 段配了 GPU」那一段）：这台机器领的块（`is_mine`）里在这一把核对键下还没核过的
/// （[`blocks_not_yet_verified_under_the_verifier_row_key`]）写成核对待判行，等它们在库上现算为核完（不问服务收不收）；
/// 进度行照 [`gpu_verification_server_waiting_line_is_due`] 打；被叫停就不再等，交回核完的那几块。
/// 交回的账：开写之前就核过的块记 `blocks_already_verified`；服务替这一趟核完的块读回核对行，记核了几块、几个状态、几个核对红的状态。
///
/// # Errors
/// 这一把核对键照判不入库（服务写不进这个进程里的行），在一行核对待判行都没写时交回；库读写失败；
/// 核完的块读回的核对行不在、或结论位的条数与这一块的状态数不等。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "没编进 gpu 特性时第 ② 段不走 GPU 那一路，只有单测替服务核掉核对待判行时调它"
    )
)]
#[allow(
    clippy::too_many_arguments,
    reason = "库、计划、流程摘要、事实表摘要、分工、怎么等、叫停，各是各的东西"
)]
#[allow(
    clippy::too_many_lines,
    reason = "写核对待判行、等核完、读回核对行记账，是这一路的全部步骤，拆开要把那几块来回传"
)]
pub(crate) fn verify_my_blocks_through_the_facts_verifier_pending_rows(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    plan: &CrashPlan,
    recorded_flow_digest: &[u8; DIGEST_BYTES],
    facts_digest: &[u8; DIGEST_BYTES],
    is_mine: &dyn Fn(u64) -> bool,
    waiting: &WaitingForTheGpuVerificationServer<'_>,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<VerificationTally, String> {
    let verifier_row_key = facts_verifier_row_key(facts_digest);
    if library
        .take()
        .versions_kept_out_of_the_library()
        .contains(&verifier_row_key)
    {
        return Err(
            "这一把核对键照判不入库：GPU 批处理服务写不进这个进程里的行，不写核对待判行"
                .to_string(),
        );
    }
    let read_error = |error: VerdictStoreError| format!("核对待判行：读写判定库失败：{error:?}");
    let stream = facts_verifier_pending_stream_key(recorded_flow_digest, facts_digest);
    let blocks = blocks_not_yet_verified_under_the_verifier_row_key(
        library,
        plan,
        &verifier_row_key,
        is_mine,
    )
    .map_err(read_error)?;
    let mut tally = VerificationTally {
        blocks_verified: 0,
        blocks_already_verified: blocks.already_verified,
        states_verified: 0,
        verifier_red_states: 0,
    };
    let mut waited_for = Vec::with_capacity(blocks.not_yet_verified.len());
    for block in blocks.not_yet_verified {
        let point = &plan.points[block.point_index];
        let row = crate::verdict_store::FactsVerifierPendingRow {
            recorded_block: point.block_key(&block.local),
            recorded_flow_digest: *recorded_flow_digest,
            facts_digest: *facts_digest,
            state_ordinals: point.ordinals.start + block.local.start
                ..point.ordinals.start + block.local.end,
            pool_entry_number: pool_entry_number_now(),
        };
        let row_in_the_library = match library
            .take()
            .put_the_facts_verifier_pending_row_unless_present(&stream, block.block_number, &row)
            .map_err(read_error)?
        {
            crate::verdict_store::FactsVerifierPendingRowPut::Written => row,
            crate::verdict_store::FactsVerifierPendingRowPut::AlreadyPresent(present) => present,
        };
        waited_for.push(BlockVerifiedOnTheServer {
            block_number: block.block_number,
            row: row_in_the_library,
        });
    }
    let blocks_written = waited_for.len();
    let waiting_started = std::time::Instant::now();
    let mut last_progress_line = waiting_started;
    let mut blocks_remaining_at_the_last_line = blocks_written;
    let mut blocks_not_verified_yet = waited_for;
    let mut verified = Vec::with_capacity(blocks_not_verified_yet.len());
    // 迭代上界：服务核完一块就少一块；被叫停就出圈。服务一直不核就一直等，块数不动不打进度行（日志不涨，统一任务服务照卡住停）
    while !blocks_not_verified_yet.is_empty() {
        let rows_now = library
            .take()
            .read_facts_verifier_pending_rows(
                &blocks_not_verified_yet
                    .iter()
                    .map(|waited| (stream, waited.block_number))
                    .collect::<Vec<_>>(),
            )
            .map_err(read_error)?;
        let mut blocks_still_pending = Vec::with_capacity(blocks_not_verified_yet.len());
        for (waited, row_now) in blocks_not_verified_yet.into_iter().zip(rows_now) {
            if row_now.is_some() {
                blocks_still_pending.push(waited);
                continue;
            }
            let store = library.take();
            if store
                .read_verifier_block(&waited.row.recorded_block, &verifier_row_key)
                .map_err(read_error)?
                .is_some()
            {
                verified.push(waited);
                continue;
            }
            // 核对待判行不在了却还没核过：照不在才写补写，接着等
            store
                .put_the_facts_verifier_pending_row_unless_present(
                    &stream,
                    waited.block_number,
                    &waited.row,
                )
                .map_err(read_error)?;
            blocks_still_pending.push(waited);
        }
        blocks_not_verified_yet = blocks_still_pending;
        if blocks_not_verified_yet.is_empty() || stop_is_requested() {
            break;
        }
        if gpu_verification_server_waiting_line_is_due(
            last_progress_line.elapsed(),
            waiting.progress_interval,
            blocks_not_verified_yet.len(),
            blocks_remaining_at_the_last_line,
        ) {
            println!(
                "{}",
                facts_verifier_waiting_line(
                    blocks_not_verified_yet.len(),
                    blocks_not_verified_yet
                        .iter()
                        .map(|waited| waited.row.state_ordinals.end
                            - waited.row.state_ordinals.start)
                        .sum(),
                    blocks_written,
                    waiting_started.elapsed().as_secs_f64()
                )
            );
            last_progress_line = std::time::Instant::now();
            blocks_remaining_at_the_last_line = blocks_not_verified_yet.len();
        }
        (waiting.between_two_reckonings)(library);
    }
    for waited in &verified {
        let verifier_row = library
            .take()
            .read_verifier_block(&waited.row.recorded_block, &verifier_row_key)
            .map_err(read_error)?
            .ok_or_else(|| {
                format!(
                    "块 {} 核完了，读回时核对行却不在（块键 {}）",
                    waited.block_number,
                    block_key_named_in_the_sentinel_report(&waited.row.recorded_block)
                )
            })?;
        let states_of_the_block = waited.row.state_ordinals.end - waited.row.state_ordinals.start;
        if u64::try_from(verifier_row.bits.len()).ok() != Some(states_of_the_block) {
            return Err(format!(
                "块 {} 读回的核对行有 {} 个结论位，这一块有 {states_of_the_block} 个状态（块键 {}）：服务写回的核对行不是这一块的",
                waited.block_number,
                verifier_row.bits.len(),
                block_key_named_in_the_sentinel_report(&waited.row.recorded_block)
            ));
        }
        tally.blocks_verified += 1;
        tally.states_verified += states_of_the_block;
        tally.verifier_red_states +=
            u64::try_from(verifier_row.bits.iter().filter(|bits| **bits != 0).count())
                .expect("一块的状态数装得进 u64");
    }
    Ok(tally)
}

/// 第 ② 段配了 GPU 的入口（崩溃放量的判段经它）：判段的库在临时 prune-server 里的，先在真服务里登记那份临时库
/// （[`temporary_library_of_the_judging_stage_registered_with_the_real_service`]，等完放掉登记，不问服务收不收）；再把要核的块写成核对待判行、等服务核完
/// （[`verify_my_blocks_through_the_facts_verifier_pending_rows`]，等法同第 ① 段的 [`judge_my_blocks_on_gpu`]）。这个进程不持卡核、不要卡。
/// `recorded_flow_digest` 是判段读回的那一份流在库里的摘要，`facts_digest` 是读回的事实表的摘要：写进核对待判行，服务照它们读回材料。
///
/// # Errors
/// 登记不成（判段这一侧先拒的：指到真服务、前缀不合、连不上；连不上真服务、认领不成的照判定库客户端的退出码退出进程）；
/// 同 [`verify_my_blocks_through_the_facts_verifier_pending_rows`]。
#[cfg(feature = "gpu")]
pub(super) fn verify_my_blocks_on_gpu(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    recorded_flow_digest: &[u8; DIGEST_BYTES],
    plan: &CrashPlan,
    facts_digest: &[u8; DIGEST_BYTES],
    is_mine: &dyn Fn(u64) -> bool,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<VerificationTally, String> {
    let temporary_library =
        temporary_library_of_the_judging_stage_registered_with_the_real_service(
            library,
            PendingRowFamilyOfTheGpuVerificationServer::FactsVerifier,
        )?;
    let tally = verify_my_blocks_through_the_facts_verifier_pending_rows(
        library,
        plan,
        recorded_flow_digest,
        facts_digest,
        is_mine,
        &WaitingForTheGpuVerificationServer {
            progress_interval: GPU_VERIFICATION_SERVER_WAITING_PROGRESS_INTERVAL,
            between_two_reckonings: &|_library| {
                std::thread::sleep(GPU_VERIFICATION_SERVER_RECKONING_INTERVAL);
            },
        },
        stop_is_requested,
    )?;
    drop(temporary_library);
    Ok(tally)
}

/// 没编进 `gpu` 特性走不到这里（选路先报错：[`computation_routes`]）。
#[cfg(not(feature = "gpu"))]
pub(super) fn verify_my_blocks_on_gpu(
    _library: &LibraryTakenAtEachReadOrWrite<'_>,
    _recorded_flow_digest: &[u8; DIGEST_BYTES],
    _plan: &CrashPlan,
    _facts_digest: &[u8; DIGEST_BYTES],
    _is_mine: &dyn Fn(u64) -> bool,
    _stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<VerificationTally, String> {
    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features gpu）".to_string())
}

// ------------------------------ GPU 批处理服务核核对待判行要的三样（服务照调，不另写一份） ------------------------------
//
// 服务从库里取出一行核对待判行（`VerdictStore::every_facts_verifier_pending_row`，连同它的流键与块号）之后：照
// [`facts_verifier_pending_block_is_verified_in_the_library`] 在库上现算这一块核没核完（核完了只删那一行）；没核完照
// [`material_of_a_facts_verifier_pending_block`] 读回核对材料，在卡上照材料建核对器（`crate::crash_verify_gpu::GpuFactsVerifier::new_by_the_segment_expansions`，
// 收材料里的事实表与段展开）、核材料里那一段状态（`GpuFactsVerifier::verify_states`）；核完照 [`facts_verifier_frame_of_a_pending_block`]
// 把结论位编成一帧（写核对行、删核对待判行，带两个守卫），在自己的写帧里原子地写下。

/// 一行核对待判行指的那一块，服务在卡上核要的材料（[`material_of_a_facts_verifier_pending_block`]）。
pub struct FactsVerifierPendingBlockMaterial {
    /// 读回、核过摘要的事实表（行里的事实表摘要）。
    pub facts: crate::crash_facts::FlowFactsReadBack,
    /// 照录下的流程排的计划，每一段的拆法（核对器照它拆序号，与判段 CPU 参照核的同一份）。
    pub segment_expansions: Vec<SegmentExpansionFact>,
    /// 这一行领的块：要核的状态在事实表里的序号区间（核过与计划里块键相同的那一块对得上）。
    pub state_ordinals: Range<u64>,
    recorded_flow_digest: [u8; DIGEST_BYTES],
    facts_digest: [u8; DIGEST_BYTES],
    recorded_block: VerdictBlockKey,
}

/// 照一行核对待判行读回核对材料：事实表（`row.facts_digest`）、录下的流程（`row.recorded_flow_digest`）照它排的计划的段展开，
/// 与这一行领的块（`row.state_ordinals`，核过它是计划里块键为 `row.recorded_block` 的那一块）。
///
/// # Errors
/// 库读失败；库里没有这份事实表或这份流程，或字节的摘要对不上、解不开（判段读回同一样东西时的那几句）；
/// 行里的态序号区间对不上计划里块键相同的任何一块。
pub fn material_of_a_facts_verifier_pending_block(
    store: &VerdictStore,
    row: &crate::verdict_store::FactsVerifierPendingRow,
) -> Result<FactsVerifierPendingBlockMaterial, String> {
    let facts = crate::crash_library::read_flow_facts_back(store, &row.facts_digest)?;
    let flow_read_back =
        RecordedFlowReadBack::read_from_the_library(store, &row.recorded_flow_digest)?;
    let flow = flow_read_back.flow();
    let plan = plan_crash_points_with_slices(&flow, &flow_read_back.recorded().segment_slices);
    let is_a_block_of_the_plan = plan.points.iter().any(|point| {
        let (Some(start), Some(end)) = (
            row.state_ordinals.start.checked_sub(point.ordinals.start),
            row.state_ordinals.end.checked_sub(point.ordinals.start),
        ) else {
            return false;
        };
        point.blocks().contains(&(start..end))
            && point.block_key(&(start..end)) == row.recorded_block
    });
    if !is_a_block_of_the_plan {
        return Err(format!(
            "核对待判行（块键 {}）的态序号区间 {:?} 对不上这条流计划里块键相同的任何一块",
            block_key_named_in_the_sentinel_report(&row.recorded_block),
            row.state_ordinals
        ));
    }
    Ok(FactsVerifierPendingBlockMaterial {
        segment_expansions: segment_expansions_of_the_plan(&flow, &plan),
        facts,
        state_ordinals: row.state_ordinals.clone(),
        recorded_flow_digest: row.recorded_flow_digest,
        facts_digest: row.facts_digest,
        recorded_block: row.recorded_block.clone(),
    })
}

/// 一行核对待判行的核对键（[`facts_verifier_row_key`]：行里的事实表摘要）。
fn facts_verifier_row_key_of_the_pending_row(
    row: &crate::verdict_store::FactsVerifierPendingRow,
) -> [u8; DIGEST_BYTES] {
    facts_verifier_row_key(&row.facts_digest)
}

/// 在库上现算一行核对待判行指的那一块核没核完：核对表里这一块在行的核对键（[`facts_verifier_row_key`]）下有那一行，判段等的就是它。
///
/// # Errors
/// 库读失败（不当成没核完）。
pub fn facts_verifier_pending_block_is_verified_in_the_library(
    store: &VerdictStore,
    row: &crate::verdict_store::FactsVerifierPendingRow,
) -> Result<bool, VerdictStoreError> {
    Ok(store
        .read_verifier_block(
            &row.recorded_block,
            &facts_verifier_row_key_of_the_pending_row(row),
        )?
        .is_some())
}

/// 把一块的核对结果编成服务写回的那一帧（[`crate::verdict_store::FactsVerifierRowsForOneFrame`]：写核对行、删核对待判行，带两个守卫）。
/// `material` 是照这一行读回的材料（[`material_of_a_facts_verifier_pending_block`]），`stream` 与 `block_number` 是服务取这一行时的流键与块号
/// （删的就是这一把键），`row` 是服务取到的那一行（守卫照它的字节），`bits_of_each_state` 是核对器交回的结论位（按态序号次序，一个状态一个）。
/// 核对行照 `crate::verdict_store::VerdictStore::store_verifier_block` 同一种值编，键是行的核对键（[`facts_verifier_row_key`]）。不写库。
///
/// # Errors
/// 材料不是这一行那条流、那份事实表、那一块的；流键不是照这一行算出的那一把；结论位的条数与这一块的状态数不等；库读失败。
pub fn facts_verifier_frame_of_a_pending_block(
    store: &VerdictStore,
    material: &FactsVerifierPendingBlockMaterial,
    stream: &crate::verdict_store::FactsVerifierPendingStreamKey,
    block_number: u64,
    row: &crate::verdict_store::FactsVerifierPendingRow,
    bits_of_each_state: &[u32],
) -> Result<crate::verdict_store::FactsVerifierRowsForOneFrame, String> {
    let block_key_text = block_key_named_in_the_sentinel_report(&row.recorded_block);
    if (
        material.recorded_flow_digest,
        material.facts_digest,
        &material.recorded_block,
        &material.state_ordinals,
    ) != (
        row.recorded_flow_digest,
        row.facts_digest,
        &row.recorded_block,
        &row.state_ordinals,
    ) {
        return Err(format!(
            "材料是录下的流程 {}、事实表 {}、块键 {} 那一块的，这一行指的是流程 {}、事实表 {}、块键 {block_key_text}",
            hexadecimal_text(&material.recorded_flow_digest[..8]),
            hexadecimal_text(&material.facts_digest[..8]),
            block_key_named_in_the_sentinel_report(&material.recorded_block),
            hexadecimal_text(&row.recorded_flow_digest[..8]),
            hexadecimal_text(&row.facts_digest[..8])
        ));
    }
    if *stream != facts_verifier_pending_stream_key(&row.recorded_flow_digest, &row.facts_digest) {
        return Err(format!(
            "块键 {block_key_text}：交来的流键不是照这一行（流程、事实表）算出的那一把，删的就不是这一行"
        ));
    }
    let states_of_the_block = row.state_ordinals.end - row.state_ordinals.start;
    if u64::try_from(bits_of_each_state.len()).ok() != Some(states_of_the_block) {
        return Err(format!(
            "块键 {block_key_text}：交来 {} 个状态的结论位，这一块有 {states_of_the_block} 个状态",
            bits_of_each_state.len()
        ));
    }
    store
        .facts_verifier_rows_for_one_frame(
            stream,
            block_number,
            row,
            &facts_verifier_row_key_of_the_pending_row(row),
            bits_of_each_state,
        )
        .map_err(|error| format!("块键 {block_key_text}：{error:?}"))
}

#[cfg(test)]
mod tests;
