//! 待判行与打包判表：旋钮的摘要、打包判表经服务判完的判定字、待判流的材料与待判块的结论帧
//! （[`material_of_a_gpu_pending_stream`]、[`conclusion_frame_of_a_gpu_pending_block`]）。

use super::*;

// ------------------------------ 判表待判行：没交卡的判段把一张打包判表交 GPU 批处理服务判、读回结论字 ------------------------------
//
// 给定状态表（[`GivenStatesEngine`] 判一张表）与观察者交回的盘（[`PoolCheckerOfTheImagesAfterTheObservers`]）配了 GPU、调用方又没交卡
// （取卡交错而环境里没有服务交的卡串，[`way_after_the_cards_are_not_taken`]；交了卡串而取不成的报错）时，判段不在自己进程里持卡判：
// 判表在判段现建、写进库（表 `11`）再读回，
// 读回的那一份交 [`judge_packed_tables_through_the_gpu_verification_server`]：先读判表结论字（表 `21`），有而不强制现判就用；没有就写判表待判行
// （表 `20`，不问服务收不收：键只由判表与旋钮定，服务只照行的数据判），等服务写回表 `21`，
// 读回的结论字在判段照本进程上卡那一支同一套解。判完的口径与强制现判的凭据写在 `crate::verdict_store` 文件头「判表待判行」那一段。
// 临时库登记与进度行的判法都是镜像判定那一族的那几个（[`temporary_library_of_the_judging_stage_registered_with_the_real_service`]、
// [`gpu_verification_server_waiting_line_is_due`]），不另写一份。
// 内核源码照行里的旋钮栏取（`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`），规矩只在这一处：观察者交回的盘那一种恒写
// 产品那一份旋钮（`ContainerCapacityKnobs::PRODUCT`），取出的就是它在本进程上卡那一支编的 `JudgeKernelSources::of_the_judge()`
// （`of_the_judge` 就是照产品旋钮取的那三份）；给定状态表照每张表那一栏旋钮写，与它在本进程上卡那一支编的同一份。

/// 三棵容器树的旋钮的摘要：照判表待判行那三栏的写法编（[`crate::verdict_store::container_capacity_knob_columns_bytes`]）、带标签取 SHA-256。
/// 判表待判行与判表结论字的键里旋钮那一段（[`crate::verdict_store::PackedTablesKey`]）、GPU 批处理服务里一条流的键都是它（服务调这一份，不另写）：
/// 旋钮是判的输入数据（同一张判表、不同的旋钮判出的结论字不同），内核源码与判器的代码不进键。
#[must_use]
pub fn container_capacity_knobs_digest(knobs: ContainerCapacityKnobs) -> [u8; DIGEST_BYTES] {
    let mut message = b"singlefs container capacity knobs of the packed tables\n".to_vec();
    message.extend_from_slice(&crate::verdict_store::container_capacity_knob_columns_bytes(knobs));
    sha256_digest(&message)
}

/// 判段取卡没成之后走哪一支（[`way_after_the_cards_are_not_taken`]；给定状态引擎与观察者交回的盘两种调用方共用）。
#[cfg(feature = "gpu")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum WayAfterTheCardsAreNotTaken {
    /// 调用方没交卡：判表交 GPU 批处理服务判。
    ThroughTheGpuVerificationServer,
    /// 调用方交了卡串，却取不成（写法不对、地址不在 `HOSTS` 里、卡池起不来或与它对不上）：照取卡报的那一句报错，不交服务判。
    CardStringHandedOverButNotTaken { reason: String },
}

/// 判段取卡没成（`reason` 是取卡报的那一句）之后走哪一支：环境里没有服务交的卡串（`GPU_CARDS_BY_PRIORITY` 没设、是空串或只有空白，
/// [`crate::gpu_card_configuration::card_string_handed_over_by_the_service`]）是调用方没交卡，判表交 GPU 批处理服务判；交了卡串而取不成的，
/// 是调用方交的那一份写错了或与这个进程的卡池对不上，报那一句、不交服务判（交了服务判得出结论，那一句配置错就报不出来了）。
#[cfg(feature = "gpu")]
pub(super) fn way_after_the_cards_are_not_taken(
    card_string_handed_over_by_the_caller: Option<&str>,
    reason: String,
) -> WayAfterTheCardsAreNotTaken {
    match crate::gpu_card_configuration::card_string_handed_over_by_the_service(
        card_string_handed_over_by_the_caller,
    ) {
        None => WayAfterTheCardsAreNotTaken::ThroughTheGpuVerificationServer,
        Some(_) => WayAfterTheCardsAreNotTaken::CardStringHandedOverButNotTaken { reason },
    }
}

/// 一张读回的打包判表照这一组旋钮交服务判时的键与待判行：键是判表摘要（与表 `11` 存它的键同一个）与旋钮的摘要
/// （[`container_capacity_knobs_digest`]）；行记旋钮、判表的态数、强制现判与此刻的入池序号。
#[cfg(feature = "gpu")]
fn packed_tables_key_and_pending_row(
    packed: &crate::crash_judge_dispatch::PackedJudgeTables,
    container_capacity_knobs: ContainerCapacityKnobs,
    forced: crate::verdict_store::ForcedRejudging,
) -> (
    crate::verdict_store::PackedTablesKey,
    crate::verdict_store::PackedTablesPendingRow,
) {
    let key = crate::verdict_store::PackedTablesKey {
        packed_judge_tables_digest: sha256_digest(&packed.to_bytes()),
        container_capacity_knobs_digest: container_capacity_knobs_digest(container_capacity_knobs),
    };
    let row = crate::verdict_store::PackedTablesPendingRow {
        container_capacity_knobs,
        state_count: packed.state_count(),
        forced,
        pool_entry_number: pool_entry_number_now(),
    };
    (key, row)
}

/// 库里这一把键的判表结论字（表 `21`），态数与判表的态数（`state_count`）核过；不在是 `None`。
///
/// # Errors
/// 库读失败、那一行不合布局；那一行的态数与判表的不等（库里那一行坏了，点名判表摘要与两个数）。
#[cfg(feature = "gpu")]
fn verdict_words_of_the_packed_tables_in_the_library(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    key: &crate::verdict_store::PackedTablesKey,
    state_count: u64,
) -> Result<Option<crate::verdict_store::PackedTablesVerdictWordsRow>, String> {
    let words = library
        .take()
        .read_packed_tables_verdict_words(key)
        .map_err(|error| format!("判表结论字：读判定库失败：{error:?}"))?;
    if let Some(present) = &words {
        let states_present =
            u64::try_from(present.verdict_words.len()).expect("一张判表的态数装得进 u64");
        if states_present != state_count {
            return Err(format!(
                "判表 {} 的判表结论字有 {states_present} 个状态，判表有 {state_count} 个：库里那一行坏了",
                hexadecimal_text(&key.packed_judge_tables_digest[..8])
            ));
        }
    }
    Ok(words)
}

/// 写判表待判行、等服务写回判表结论字那一路怎么收的（[`wait_for_the_verdict_words_of_the_packed_tables`]）。
#[cfg(feature = "gpu")]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PackedTablesThroughTheGpuVerificationServerEnded {
    /// 服务写回了（或库里早有、不强制现判就直接用）这张判表全部状态的结论字。
    Judged(crate::verdict_store::PackedTablesVerdictWordsRow),
    /// 被叫停：写下的判表待判行留在库里，没等到结论字。
    Stopped,
}

/// 等一张判表的进度行（[`GPU_VERIFICATION_SERVER_WAITING_LINE_PREFIX`] 那一行的写法，带族名）：
/// `<前缀> family=packed-tables blocks_remaining=<还剩几张判表> states_remaining=<还剩几个状态> blocks_written=<这一趟写成待判行的判表数> seconds_waited=<等了几秒>`。
#[cfg(feature = "gpu")]
fn packed_tables_waiting_line(
    tables_remaining: usize,
    states_remaining: u64,
    tables_written: usize,
    seconds_waited: f64,
) -> String {
    format!(
        "{GPU_VERIFICATION_SERVER_WAITING_LINE_PREFIX} family={} blocks_remaining={tables_remaining} states_remaining={states_remaining} blocks_written={tables_written} seconds_waited={seconds_waited:.0}",
        PendingRowFamilyOfTheGpuVerificationServer::PackedTables
            .label_of_the_temporary_library_registration()
    )
}

/// 一张判表交服务判的那一层（文件头这一节上面「判表待判行」那一段）：先读判表结论字，有而不强制现判就交回、不写待判行；
/// 否则写判表待判行（不在才写，已在而这一份强制就抬成强制；不问服务收不收），
/// 每圈读表 `21` 与表 `20`：不强制的表 `21` 有了算完；强制的要表 `20` 这一把键不在、表 `21` 在才算完（凭据：服务只在写表 `21` 那一帧里
/// 删强制的那一行）；表 `20` 不在而表 `21` 也不在的照不在才写补写。进度行照 [`gpu_verification_server_waiting_line_is_due`] 打；
/// 被叫停就不再等，交回 [`PackedTablesThroughTheGpuVerificationServerEnded::Stopped`]。
///
/// # Errors
/// 判表的态数多于一帧装得下的
/// （[`crate::verdict_store::MOST_STATES_OF_A_PACKED_TABLE_IN_ONE_VERDICT_WORDS_FRAME`]：一行判表结论字是一张判表的全部状态、第一版不切行），
/// 在一行都没写时交回；库读写失败；库里那一行判表结论字的态数与判表的不等。
#[cfg(feature = "gpu")]
pub(crate) fn wait_for_the_verdict_words_of_the_packed_tables(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    key: &crate::verdict_store::PackedTablesKey,
    row: &crate::verdict_store::PackedTablesPendingRow,
    waiting: &WaitingForTheGpuVerificationServer<'_>,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<PackedTablesThroughTheGpuVerificationServerEnded, String> {
    use crate::verdict_store::ForcedRejudging;
    if row.state_count
        > crate::verdict_store::MOST_STATES_OF_A_PACKED_TABLE_IN_ONE_VERDICT_WORDS_FRAME
    {
        return Err(format!(
            "判表 {} 有 {} 个状态，一行判表结论字一帧最多装 {} 个：一行是一张判表的全部状态，切行协议没定，第一版不支持（OutsideTheFirstVersion）。\
             一行判表待判行都没写",
            hexadecimal_text(&key.packed_judge_tables_digest[..8]),
            row.state_count,
            crate::verdict_store::MOST_STATES_OF_A_PACKED_TABLE_IN_ONE_VERDICT_WORDS_FRAME
        ));
    }
    let read_error = |error: VerdictStoreError| format!("判表待判行：读写判定库失败：{error:?}");
    match row.forced {
        ForcedRejudging::NotForced => {
            if let Some(words) =
                verdict_words_of_the_packed_tables_in_the_library(library, key, row.state_count)?
            {
                return Ok(PackedTablesThroughTheGpuVerificationServerEnded::Judged(
                    words,
                ));
            }
        }
        ForcedRejudging::Forced => {}
    }
    library
        .take()
        .put_the_packed_tables_pending_row_unless_present_or_raise_it_to_forced(key, row)
        .map_err(read_error)?;
    let tables_written = 1;
    let waiting_started = std::time::Instant::now();
    let mut last_progress_line = waiting_started;
    let mut tables_remaining_at_the_last_line = tables_written;
    // 迭代上界：服务写回这一张判表的结论字就出圈；被叫停也出圈。服务一直不判就一直等，判表数不动不打进度行
    loop {
        let pending_row_now = library
            .take()
            .read_packed_tables_pending_row(key)
            .map_err(read_error)?;
        let words_now =
            verdict_words_of_the_packed_tables_in_the_library(library, key, row.state_count)?;
        match (row.forced, pending_row_now.is_some(), words_now) {
            (ForcedRejudging::NotForced, _, Some(words))
            | (ForcedRejudging::Forced, false, Some(words)) => {
                return Ok(PackedTablesThroughTheGpuVerificationServerEnded::Judged(
                    words,
                ));
            }
            // 强制的：表 21 早就在而这一行还在，是写这一行之前的那一份，接着等服务现判
            (ForcedRejudging::Forced, true, Some(_))
            | (ForcedRejudging::NotForced | ForcedRejudging::Forced, true, None) => {}
            // 待判行不在了却还没有结论字：照不在才写补写，接着等
            (ForcedRejudging::NotForced | ForcedRejudging::Forced, false, None) => {
                library
                    .take()
                    .put_the_packed_tables_pending_row_unless_present_or_raise_it_to_forced(
                        key, row,
                    )
                    .map_err(read_error)?;
            }
        }
        if stop_is_requested() {
            return Ok(PackedTablesThroughTheGpuVerificationServerEnded::Stopped);
        }
        // 这一圈还没判完的就是这一张：判表数在判完之前不变，照镜像判定那一族的判法块数不动不打
        let tables_remaining_now = 1;
        if gpu_verification_server_waiting_line_is_due(
            last_progress_line.elapsed(),
            waiting.progress_interval,
            tables_remaining_now,
            tables_remaining_at_the_last_line,
        ) {
            println!(
                "{}",
                packed_tables_waiting_line(
                    tables_remaining_now,
                    row.state_count,
                    tables_written,
                    waiting_started.elapsed().as_secs_f64()
                )
            );
            last_progress_line = std::time::Instant::now();
            tables_remaining_at_the_last_line = tables_remaining_now;
        }
        (waiting.between_two_reckonings)(library);
    }
}

/// 没交卡的判段把一张读回的打包判表交 GPU 批处理服务判（给定状态表与观察者交回的盘两种调用方共用）：判段的库在临时 prune-server 里的，
/// 先在真服务里登记那份临时库（[`temporary_library_of_the_judging_stage_registered_with_the_real_service`]，等完放掉登记，不问服务收不收），
/// 照镜像判定那一族的写法（[`judge_my_blocks_on_gpu`]）；再写判表待判行、
/// 等服务写回（[`wait_for_the_verdict_words_of_the_packed_tables`]：两次现算之间睡 [`GPU_VERIFICATION_SERVER_RECKONING_INTERVAL`]，
/// 进度行至少隔 [`GPU_VERIFICATION_SERVER_WAITING_PROGRESS_INTERVAL`]），交回这张判表全部状态的结论字与写它的设备。
/// 不退回 CPU，也不在这个进程里借卡判。
///
/// # Errors
/// 登记不成；被叫停（行留在库里，没等到结论字）；同 [`wait_for_the_verdict_words_of_the_packed_tables`]。
#[cfg(feature = "gpu")]
pub(crate) fn judge_packed_tables_through_the_gpu_verification_server(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    packed: &crate::crash_judge_dispatch::PackedJudgeTablesReadBack,
    container_capacity_knobs: ContainerCapacityKnobs,
    forced: crate::verdict_store::ForcedRejudging,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<crate::verdict_store::PackedTablesVerdictWordsRow, String> {
    let temporary_library =
        temporary_library_of_the_judging_stage_registered_with_the_real_service(
            library,
            PendingRowFamilyOfTheGpuVerificationServer::PackedTables,
        )?;
    let (key, row) = packed_tables_key_and_pending_row(packed, container_capacity_knobs, forced);
    let ended = wait_for_the_verdict_words_of_the_packed_tables(
        library,
        &key,
        &row,
        &WaitingForTheGpuVerificationServer {
            progress_interval: GPU_VERIFICATION_SERVER_WAITING_PROGRESS_INTERVAL,
            between_two_reckonings: &|_library| {
                std::thread::sleep(GPU_VERIFICATION_SERVER_RECKONING_INTERVAL);
            },
        },
        stop_is_requested,
    )?;
    drop(temporary_library);
    match ended {
        PackedTablesThroughTheGpuVerificationServerEnded::Judged(words) => Ok(words),
        PackedTablesThroughTheGpuVerificationServerEnded::Stopped => Err(format!(
            "叫停：判表 {} 写下了判表待判行，没等到 GPU 批处理服务写回结论字就停了；那一行留在库里，下一趟照键接着等",
            hexadecimal_text(&key.packed_judge_tables_digest[..8])
        )),
    }
}

/// 没交卡的判段经哪一条路把一张判表交 GPU 批处理服务判：产品登记临时库、两次现算之间睡
/// （[`judge_packed_tables_through_the_gpu_verification_server`]）；
/// 单测换成替身（两次现算之间替服务读表 `20`、写表 `21`），不登记临时库，其余同产品那一层
/// （[`wait_for_the_verdict_words_of_the_packed_tables`]）。
#[cfg(feature = "gpu")]
pub(crate) enum WayToTheGpuVerificationServer {
    TheRealService,
    #[cfg(test)]
    StandIn(std::sync::Arc<GpuVerificationServerStandIn>),
}

/// 单测里替 GPU 批处理服务的那一份：两次现算之间做的事（照表 `20` 写表 `21`、删表 `20`），交给等候那一层。
#[cfg(all(feature = "gpu", test))]
pub(crate) struct GpuVerificationServerStandIn {
    pub(crate) between_two_reckonings:
        Box<dyn Fn(&LibraryTakenAtEachReadOrWrite<'_>) + Send + Sync>,
}

#[cfg(feature = "gpu")]
impl WayToTheGpuVerificationServer {
    /// 照这条路判一张读回的打包判表，交回全部状态的结论字与写它的设备。两种调用方在等的时候都不收叫停（给定状态表引擎的叫停只管崩溃注入
    /// 不再领新的一片、手里那一片判完，`crate::crash_given_states::GivenStatesJudge::stop_is_requested`；观察者判段不收叫停）。
    ///
    /// # Errors
    /// 产品那一条同 [`judge_packed_tables_through_the_gpu_verification_server`]。替身那一条同
    /// [`wait_for_the_verdict_words_of_the_packed_tables`]。
    pub(super) fn judge_the_packed_tables(
        &self,
        library: &LibraryTakenAtEachReadOrWrite<'_>,
        packed: &crate::crash_judge_dispatch::PackedJudgeTablesReadBack,
        container_capacity_knobs: ContainerCapacityKnobs,
        forced: crate::verdict_store::ForcedRejudging,
    ) -> Result<crate::verdict_store::PackedTablesVerdictWordsRow, String> {
        match self {
            Self::TheRealService => judge_packed_tables_through_the_gpu_verification_server(
                library,
                packed,
                container_capacity_knobs,
                forced,
                &|| false,
            ),
            #[cfg(test)]
            Self::StandIn(stand_in) => {
                let (key, row) =
                    packed_tables_key_and_pending_row(packed, container_capacity_knobs, forced);
                match wait_for_the_verdict_words_of_the_packed_tables(
                    library,
                    &key,
                    &row,
                    &WaitingForTheGpuVerificationServer {
                        progress_interval: GPU_VERIFICATION_SERVER_WAITING_PROGRESS_INTERVAL,
                        between_two_reckonings: &*stand_in.between_two_reckonings,
                    },
                    &|| false,
                )? {
                    PackedTablesThroughTheGpuVerificationServerEnded::Judged(words) => Ok(words),
                    PackedTablesThroughTheGpuVerificationServerEnded::Stopped => {
                        Err("替身服务那一条不收叫停".to_string())
                    }
                }
            }
        }
    }
}

/// 服务取一行判表待判行之后照行里的旋钮栏取内核源码（`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`，文件头这一节
/// 「内核源码照行里的旋钮栏取」那一句的规矩），先核行里旋钮的摘要与键里旋钮那一段相同。
///
/// # Errors
/// 旋钮的摘要与键里那一段不同（这一行的值与它的键对不上，点名两个摘要），服务拒这一行。
#[cfg(feature = "gpu")]
pub fn kernel_sources_of_a_packed_tables_pending_row(
    key: &crate::verdict_store::PackedTablesKey,
    row: &crate::verdict_store::PackedTablesPendingRow,
) -> Result<crate::crash_judge_gpu::JudgeKernelSources, String> {
    let digest = container_capacity_knobs_digest(row.container_capacity_knobs);
    if digest != key.container_capacity_knobs_digest {
        return Err(format!(
            "判表 {} 的判表待判行：行里旋钮栏的摘要是 {}，键里是 {}：这一行的值与它的键对不上",
            hexadecimal_text(&key.packed_judge_tables_digest[..8]),
            hexadecimal_text(&digest[..8]),
            hexadecimal_text(&key.container_capacity_knobs_digest[..8])
        ));
    }
    Ok(
        crate::crash_judge_gpu::JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs(
            row.container_capacity_knobs,
        ),
    )
}

/// 服务照一行判表待判行读回它要判的打包判表（表 `11`，读法是 [`crate::crash_library::read_packed_judge_tables_back`]：读字节、核摘要、解码），
/// 核读回的判表的态数与行里记的相同。
///
/// # Errors
/// 库读失败、库里没有这个摘要、摘要对不上、解不开；态数与行里记的不等（点名判表摘要与两个数），服务拒这一行。
#[cfg(feature = "gpu")]
pub fn read_packed_judge_tables_for_a_pending_row(
    store: &VerdictStore,
    key: &crate::verdict_store::PackedTablesKey,
    row: &crate::verdict_store::PackedTablesPendingRow,
) -> Result<crate::crash_judge_dispatch::PackedJudgeTablesReadBack, String> {
    let packed = crate::crash_library::read_packed_judge_tables_back(
        store,
        &key.packed_judge_tables_digest,
    )?;
    if packed.state_count() != row.state_count {
        return Err(format!(
            "判表 {} 读回有 {} 个状态，判表待判行记的是 {}：拒这一行",
            hexadecimal_text(&key.packed_judge_tables_digest[..8]),
            packed.state_count(),
            row.state_count
        ));
    }
    Ok(packed)
}

/// 一行判表待判行在库上判完了没有（服务认领之前与认领之后重读时问）：不强制现判的，表 `21` 有这一把键、态数与行里记的相同就算判完
/// （服务带守卫删这一行）；强制现判的一律不算（它的凭据只认服务这一次现判写下的那一帧，`crate::verdict_store` 文件头「判表待判行」那一段）。
///
/// # Errors
/// 库读失败；那一行判表结论字不合布局。
#[cfg(feature = "gpu")]
pub fn packed_tables_verdict_words_are_in_the_library(
    store: &VerdictStore,
    key: &crate::verdict_store::PackedTablesKey,
    row: &crate::verdict_store::PackedTablesPendingRow,
) -> Result<bool, VerdictStoreError> {
    match row.forced {
        crate::verdict_store::ForcedRejudging::Forced => Ok(false),
        crate::verdict_store::ForcedRejudging::NotForced => Ok(store
            .read_packed_tables_verdict_words(key)?
            .is_some_and(|words| {
                u64::try_from(words.verdict_words.len()).expect("一张判表的态数装得进 u64")
                    == row.state_count
            })),
    }
}

// ------------------------------ GPU 批处理服务写回要的三样（协议「待判行」「强制现判」；服务照调，不另写一份） ------------------------------
//
// 服务从库里取出一行待判行（`VerdictStore::every_gpu_pending_row`）之后：照 [`material_of_a_gpu_pending_stream`] 读回装判器的判表与内核源码
// （连同编结论行要的那条流）；不是强制现判的块照 [`judging_of_a_gpu_pending_block_in_the_library`] 在库上现算判到了哪一步；
// 判完照 [`conclusion_frame_of_a_gpu_pending_block`] 把一块的结论字编成一帧要写的行与结论摘要。读法与写法就是判段的那一套：
// 读判表是 `crash_library::read_packed_judge_tables_back`，内核源码是判段上卡那一处编的 `JudgeKernelSources::of_the_judge`，
// 计划照判段读回流之后排的那一份（`plan_crash_points_with_slices`，片照录下的那一份），判完照判段那两个口径，结论行照判段存块那一处
// （[`gpu_judged_block_ready_to_store`]、[`ConclusionsStoredUnder::store_block`] 的整份、`VerdictStore` 存结论块的规矩）编。

/// 一行待判行指的那条流里，服务装判器、编结论行要的材料（[`material_of_a_gpu_pending_stream`]）：同一份录下的流程与判表的几行共用一份。
#[cfg(feature = "gpu")]
pub struct GpuPendingStreamMaterial {
    /// 读回、核过摘要的打包判表（GPU 判器只收它）。
    pub packed_judge_tables: PackedJudgeTablesOfTheRun,
    /// 判器编的三个内核（崩溃放量与崩溃枚举用例的流是产品旋钮，判段上卡那一处编的同一份）。
    pub kernel_sources: crate::crash_judge_gpu::JudgeKernelSources,
    recorded_flow_digest: [u8; DIGEST_BYTES],
    packed_judge_tables_digest: [u8; DIGEST_BYTES],
    plan: CrashPlan,
    devices_of_the_pool: Vec<u32>,
    slots_of_each_device_on_the_base_image: Vec<(u32, u32)>,
    publishes_of_the_states: PublishesOfTheStates,
}

/// 照一行待判行读回它那条流的材料：判表（行里的判表摘要）、内核源码，连同录下的流程（行里的流程摘要）、照它排的计划、池里盘的次序、
/// 基镜像的逐盘自证槽数与按发布分的态数（编结论行时解结论字、记计数向量要的，与判段存块那一处同一份算法）。
///
/// # Errors
/// 库读失败；库里没有这份判表或这份流程，或字节的摘要对不上、解不开（判段读回同一样东西时的那几句）。
#[cfg(feature = "gpu")]
pub fn material_of_a_gpu_pending_stream(
    store: &VerdictStore,
    row: &crate::verdict_store::GpuPendingRow,
) -> Result<GpuPendingStreamMaterial, String> {
    let packed_judge_tables = crate::crash_library::read_packed_judge_tables_back(
        store,
        &row.packed_judge_tables_digest,
    )?;
    let flow_read_back =
        RecordedFlowReadBack::read_from_the_library(store, &row.recorded_flow_digest)?;
    let (
        plan,
        devices_of_the_pool,
        slots_of_each_device_on_the_base_image,
        publishes_of_the_states,
    ) = {
        let flow = flow_read_back.flow();
        let plan = plan_crash_points_with_slices(&flow, &flow_read_back.recorded().segment_slices);
        let devices_of_the_pool: Vec<u32> =
            singlefs_core::recovery::PoolReader::device_identities(flow.base)
                .into_iter()
                .map(|device| device.0)
                .collect();
        let slots_of_each_device_on_the_base_image =
            self_verified_system_configuration_slots_of_each_device(flow.base);
        let publishes_of_the_states = PublishesOfTheStates::of_the_plan(&flow, &plan);
        (
            plan,
            devices_of_the_pool,
            slots_of_each_device_on_the_base_image,
            publishes_of_the_states,
        )
    };
    Ok(GpuPendingStreamMaterial {
        packed_judge_tables,
        kernel_sources: crate::crash_judge_gpu::JudgeKernelSources::of_the_judge(),
        recorded_flow_digest: row.recorded_flow_digest,
        packed_judge_tables_digest: row.packed_judge_tables_digest,
        plan,
        devices_of_the_pool,
        slots_of_each_device_on_the_base_image,
        publishes_of_the_states,
    })
}

/// 一行待判行指的那一块在库里判到了哪一步（[`judging_of_a_gpu_pending_block_in_the_library`]），判段那两个口径各从它读。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuPendingBlockInTheLibrary {
    /// 结论不在，或结论在而判器交回的逐盘自证槽数那一行不在：两个口径都答没判完，照判。
    NotJudged,
    /// 结论连同槽数都在、结论红：续点口径答判完（判段等的就是它），复用口径答不复用（记红、下趟现判：判段照旧把它写成待判行）。
    JudgedRed,
    /// 结论连同槽数都在、不红：两个口径都答判完。
    JudgedAndReusable,
}

impl GpuPendingBlockInTheLibrary {
    /// 续点口径（`ConclusionsStoredUnder::is_judged_with_the_slots`）：判段等服务判完看的就是它，红块也算判完。
    #[must_use]
    pub fn is_judged_with_the_slots(self) -> bool {
        match self {
            Self::NotJudged => false,
            Self::JudgedRed | Self::JudgedAndReusable => true,
        }
    }

    /// 复用口径（`ConclusionsStoredUnder::is_stored_for_reuse`，不含这一趟复用哨兵抽中的强制现判：那一块的待判行带着强制现判，服务照行判）：
    /// 判段只把这一口径答不复用的块写成待判行。
    #[must_use]
    pub fn is_stored_for_reuse(self) -> bool {
        match self {
            Self::NotJudged | Self::JudgedRed => false,
            Self::JudgedAndReusable => true,
        }
    }
}

/// 在库上现算一行待判行指的那一块判到了哪一步：结论按 GPU 判器那一种结论、行里期待的结论版本找，判段两个口径同一套读法
/// （[`ConclusionsStoredUnder::is_judged_with_the_slots`]、[`ConclusionsStoredUnder::is_red`]）。
///
/// # Errors
/// 库读失败（不当成没判完）。
pub fn judging_of_a_gpu_pending_block_in_the_library(
    store: &VerdictStore,
    row: &crate::verdict_store::GpuPendingRow,
) -> Result<GpuPendingBlockInTheLibrary, VerdictStoreError> {
    let version_of_the_only_point = [row.conclusion_version];
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: JudgedBy::GpuJudge.conclusion_kind(),
        version_of_each_point: &version_of_the_only_point,
    };
    if !conclusions.is_judged_with_the_slots(store, 0, &row.recorded_block)? {
        return Ok(GpuPendingBlockInTheLibrary::NotJudged);
    }
    if conclusions.is_red(store, 0, &row.recorded_block)? {
        Ok(GpuPendingBlockInTheLibrary::JudgedRed)
    } else {
        Ok(GpuPendingBlockInTheLibrary::JudgedAndReusable)
    }
}

/// 判一块待判块的那张卡：序号只进报错的句子，卡型与驱动版本进结论块属性里的在哪判的那一格（与判段上卡那一处同一样）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardThatJudgedAPendingBlock {
    pub card_index: u32,
    pub card_name: String,
    pub card_driver_version: String,
}

/// 一块待判块的结论编成的一帧（[`conclusion_frame_of_a_gpu_pending_block`]）：要写的行（守卫那一行排最后）、要删的行、守卫与它读到的样子，
/// 连同结论摘要（强制现判凭据里的那一个，[`ConclusionSpelledOutInVerdictVectors::conclusion_digest`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConclusionFrameOfAGpuPendingBlock {
    pub rows: crate::verdict_store::ConclusionBlockRowsForOneFrame,
    pub conclusion_digest: [u8; DIGEST_BYTES],
}

/// 把一块待判块的结论字（按态序号次序，一个状态一份）连同判它的卡编成一帧要写的行与结论摘要，照判段存块那一处同一套：
/// 解结论字（[`decoded_gpu_block`]）、记按发布分的态数、登记判定向量编号（判定向量表只追加，登记即写进 `store`）、拼计数向量与恢复字与逐盘槽数、
/// 照存结论块的规矩比库里那一份（`VerdictStore::conclusion_block_rows_for_one_frame_with_the_slots_the_judge_handed_back`）。结论行本身不写库：
/// 服务把它们连同删待判行（强制现判的再加凭据行）放进自己的写结论那一帧。
///
/// # Errors
/// 材料不是这一行那条流的；行里的态序号区间对不上计划里任何一块、或那一块的块键不是行里的块键；结论字的条数与区间长度不等；
/// 有状态 GPU 判不了；登记判定向量、读库失败；库里同一把键下存着不同而两份都不红的结论（照判段存块那一处拒）。
#[cfg(feature = "gpu")]
pub fn conclusion_frame_of_a_gpu_pending_block(
    store: &mut VerdictStore,
    material: &GpuPendingStreamMaterial,
    row: &crate::verdict_store::GpuPendingRow,
    words_of_each_state: Vec<[u32; crate::crash_judge_gpu::VERDICT_WORDS]>,
    card: &CardThatJudgedAPendingBlock,
) -> Result<ConclusionFrameOfAGpuPendingBlock, String> {
    if (
        material.recorded_flow_digest,
        material.packed_judge_tables_digest,
    ) != (row.recorded_flow_digest, row.packed_judge_tables_digest)
    {
        return Err(format!(
            "材料是录下的流程 {}、判表 {} 那一条流的，这一行指的是流程 {}、判表 {}",
            hexadecimal_text(&material.recorded_flow_digest[..8]),
            hexadecimal_text(&material.packed_judge_tables_digest[..8]),
            hexadecimal_text(&row.recorded_flow_digest[..8]),
            hexadecimal_text(&row.packed_judge_tables_digest[..8])
        ));
    }
    let block_key_text = block_key_named_in_the_sentinel_report(&row.recorded_block);
    let local = material
        .plan
        .points
        .iter()
        .find_map(|point| {
            let local = row.state_ordinals.start.checked_sub(point.ordinals.start)?
                ..row.state_ordinals.end.checked_sub(point.ordinals.start)?;
            (point.blocks().contains(&local) && point.block_key(&local) == row.recorded_block)
                .then_some(local)
        })
        .ok_or_else(|| {
            format!(
                "待判行（块键 {block_key_text}）的态序号区间 {:?} 对不上这条流计划里块键相同的任何一块",
                row.state_ordinals
            )
        })?;
    let state_count = usize::try_from(local.end - local.start).expect("一块的状态数装得进 usize");
    if words_of_each_state.len() != state_count {
        return Err(format!(
            "块键 {block_key_text}：交来 {} 个状态的结论字，这一块有 {state_count} 个状态",
            words_of_each_state.len()
        ));
    }
    if let Some(offset) = first_state_the_gpu_could_not_judge(&words_of_each_state) {
        return Err(format!(
            "卡 {} 判不了块键 {block_key_text} 块内第 {offset} 个状态（结论字带「判不了」：表越过了内核的容量），不编结论行",
            card.card_index
        ));
    }
    let mut judged = decoded_gpu_block(
        GpuVerdictWordsOfABlock {
            pending_index: 0,
            words_of_each_state,
            card_index: card.card_index,
            card_name: card.card_name.clone(),
            card_driver_version: card.card_driver_version.clone(),
        },
        &material.devices_of_the_pool,
    );
    judged.state_counts.absorb(
        material
            .publishes_of_the_states
            .state_counts_of(&row.state_ordinals),
    );
    let library_error = |error: VerdictStoreError| format!("块键 {block_key_text}：{error:?}");
    let ready = gpu_judged_block_ready_to_store(
        store,
        &local,
        judged,
        &material.slots_of_each_device_on_the_base_image,
    )
    .map_err(library_error)?;
    let (record, slots_bytes) = ConclusionsStoredUnder::conclusion_block_record_and_the_slots_bytes(
        ready.verdict_block,
        ready.violating,
        ready.along,
    );
    let rows = store
        .conclusion_block_rows_for_one_frame_with_the_slots_the_judge_handed_back(
            &row.recorded_block,
            &JudgedBy::GpuJudge.conclusion_kind(),
            &row.conclusion_version,
            record,
            slots_bytes,
        )
        .map_err(library_error)?;
    let conclusion_digest = ConclusionSpelledOutInVerdictVectors::of_the_stored_block(
        store,
        &rows.conclusion_as_requested,
    )
    .conclusion_digest();
    Ok(ConclusionFrameOfAGpuPendingBlock {
        rows,
        conclusion_digest,
    })
}

#[cfg(test)]
mod tests;
