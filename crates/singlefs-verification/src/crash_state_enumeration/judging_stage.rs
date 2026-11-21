//! 判段：从库里读回块、照路线在 CPU 或经 GPU 批处理服务判、结论存库，复用的块抽哨兵现判。

use super::*;

/// 判段：凭清单读回流，排计划，这台机器领的、库里还没有这一把版本结论的块交 GPU 批处理服务判或由 CPU 判、存进库；观察者照样按块看、存；
/// 对账、核计数向量，从库里读回每一块的结论重建崩溃点重放的计数。
#[allow(
    clippy::too_many_lines,
    reason = "读回、选路、版本、分工、判、看、对账、重建计数、打行，是判段的全部步骤，拆开要把十几个量来回传"
)]
#[allow(
    clippy::too_many_arguments,
    reason = "库、清单、项、观察者与它们的版本、选路、仓根、这一趟判器那一半键的结局、入口读的 GPU 那一路的两样，各是各的东西"
)]
pub(super) fn judge_the_recorded_item(
    library: &CrashLibrary,
    manifest: &ItemManifest,
    item: CrashPointReplayItem<'_>,
    observers_on_the_nodes: &[ObserverOnTheNodes<'_>],
    observer_versions: &[ObserverVersions],
    route: CrashPointReplayJudgeRoute,
    judging_units_of_the_run: &JudgingUnitsOfTheRun<'_>,
    gpu_judging_inputs: GpuJudgingInputsOfTheEntry,
) -> Result<StageOutcome<CrashPointReplayTally>, String> {
    let started = Instant::now();
    let read_back = RecordedFlowReadBack::read_outside_the_library_lock(
        library,
        &manifest.digest_named(MANIFEST_FLOW)?,
    )?;
    let flow_of_the_library = read_back.flow();
    let flow = &flow_of_the_library;
    let plan = plan_crash_points(flow);
    // 镜像判定的版本由结论种类与判器引到的被检测产品代码定（判器的代码不进键）：调用方不另取判法单元（那一组拒算时是这个进程的占位，开库时登成照判不入库）
    let conclusion_versions = ConclusionVersionsOfThePlan::of_the_plan(&plan);
    let observer_conclusion_versions: Vec<ConclusionVersion> = observer_versions
        .iter()
        .map(|versions| versions.conclusion)
        .collect();
    let judged_by = route.judged_by();
    let judged_conclusions = conclusion_versions.stored_under(judged_by);
    // 两台分片与崩溃放量同一种读法：这台机器领哪几块按权重平滑轮转
    let (share, host_weights) =
        crate::crash_amplification::host_share_and_weights_from_the_environment()?;
    let host_shares = WeightedShares::new(host_weights);
    let total_blocks: u64 = plan
        .points
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("块数装得进 u64"))
        .sum();
    let selected: HashSet<u64> = (0..total_blocks)
        .filter(|block| host_shares.owner_of_block(*block) == share.index)
        .collect();
    let is_mine = |block: u64| selected.contains(&block);
    let (worker_threads, worker_threads_source) = crash_point_replay_worker_threads();
    // 判段每一次读写库现取现放：判块、看观察者、排计划时手里不握锁，同一个进程里别的用例照样读写这条连接
    let library_at_each_read_or_write = library.taken_at_each_read_or_write();
    // 块账（CRASH_CASE_BLOCKS）点这一份领的块（两台各领一份时，别的机器判的块直接写进服务那一份，不分片的那一趟照库复用、罩整条流）；
    // 判之前照库数一遍罩住的，没罩住的就是这一趟要判的
    let is_counted = |block: u64| is_mine(block);
    let observers_in_the_account = observers_in_the_block_account(
        &plan,
        observers_on_the_nodes,
        &observer_conclusion_versions,
    );
    let blocks_before_judging = blocks_in_the_library(
        &library_at_each_read_or_write.take(),
        &plan,
        &judged_conclusions,
        &observers_in_the_account,
        &is_counted,
    )?;
    let pending = pending_blocks(
        &library_at_each_read_or_write.take(),
        &plan,
        &judged_conclusions,
        &is_mine,
    )
    .map_err(|error| format!("读判定块失败：{error:?}"))?;
    let blocks_of_this_share = u64::try_from(selected.len()).expect("块数装得进 u64");
    let blocks_to_judge = u64::try_from(pending.len()).expect("块数装得进 u64");
    let blocks_reused = blocks_of_this_share - blocks_to_judge;
    let state_count = plan
        .points
        .last()
        .map_or(0, |end_of_flow| end_of_flow.ordinals.end);
    let share_fields = if share.count > 1 {
        format!(
            " shard={}/{} all_blocks={total_blocks}",
            share.index, share.count
        )
    } else {
        String::new()
    };
    println!(
        "CRASH_POINT_REPLAY_PARALLEL_START states={state_count} slices={blocks_of_this_share} worker_threads={worker_threads} configured_worker_threads={worker_threads} worker_threads_source={worker_threads_source} resumed_slices={blocks_reused} freshly_run_slices={blocks_to_judge} judge={} library={} item={}{share_fields}",
        route.name(),
        library.library_name(),
        item.name
    );
    let stop_request = StopRequestRememberedInThisRun::new(&stop_is_requested_by_the_environment);
    let stop_is_requested = || stop_request.is_requested();
    // 复用哨兵（与崩溃放量那一路同一套：`.claude/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」每一趟真跑插哨兵与强制现判）：
    // 开跑前从这一份里键相同、库里有不红结论的块里抽几块当哨兵、再抽一块强制现判，种子打进日志；判完照它判（哨兵被现判、强制现判仍复用、
    // 强制现判出的与库里那一份不同，这条用例红）。块账与 resumed / freshly_run 在这之前数（强制现判那一块按复用算）
    let reuse_sentinel_count = crate::crash_amplification::reuse_sentinel_count_of_this_process();
    let observers_for_the_reuse_sentinels_of_the_item =
        crate::crash_amplification::observers_for_the_reuse_sentinels(
            &plan,
            observers_on_the_nodes,
            observer_versions,
        );
    let reuse_sentinels = crate::crash_amplification::plan_the_reuse_sentinels(
        &mut library_at_each_read_or_write.take(),
        &plan,
        &judged_conclusions,
        &is_mine,
        reuse_sentinel_count,
        crate::crash_amplification::reuse_sentinel_seed_of_this_process(),
        &observers_for_the_reuse_sentinels_of_the_item,
    )
    .map_err(|error| format!("抽复用哨兵读库失败：{error:?}"))?;
    if let Some(sentinels) = &reuse_sentinels {
        println!("{}", sentinels.line(item.name, reuse_sentinel_count));
        // 强制现判那一块上观察者要读的录入段那一行不在库里：回录入段只给它重录（判段要现拼它上面的观察者结论）
        if crate::crash_amplification::record_the_observers_of_the_forced_block_again(
            &library_at_each_read_or_write,
            flow,
            &plan,
            observers_on_the_nodes,
            observer_versions,
            sentinels,
            &stop_is_requested,
        )? {
            println!(
                "CRASH_POINT_REPLAY_REUSE_SENTINEL_FORCED_BLOCK_RECORDED_AGAIN item={}",
                item.name
            );
        }
    }
    match route {
        CrashPointReplayJudgeRoute::Cpu { why_not_the_gpu } => {
            println!(
                "CRASH_POINT_REPLAY_JUDGE route=cpu why_not_the_gpu={why_not_the_gpu} threads={worker_threads}"
            );
            check_blocks_where_until_with_threads(
                &library_at_each_read_or_write,
                flow,
                &plan,
                &is_mine,
                &judged_conclusions,
                &stop_is_requested,
                worker_threads,
            )
            .map_err(|error| format!("CPU 判块失败：{error:?}"))?;
        }
        CrashPointReplayJudgeRoute::Gpu => judge_the_pending_blocks_through_the_gpu_verification_server(
            &library_at_each_read_or_write,
            &plan,
            manifest,
            &is_mine,
            &judged_conclusions,
            &stop_is_requested,
        )?,
        CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature => unreachable!(
            "判段不跑的那一趟在 enumerate_crash_states_through_the_library 的判段闭包里先交只录，不进判段"
        ),
    }
    // 观察者的判段：从库里读录入段那几块（照判不入库的观察者先在这个进程里补录），观察者交回的盘上的池级 checker 照第 ① 段的选路判
    // （配了 GPU 上卡，头一张要判的盘那一刻才取卡）。
    // 叫停了不判观察者：录入段可能停在块边上，没录的块判段读不到，交回叫停那一句，重跑同一条用例接着录、接着判
    // （哨兵与强制现判那一块可能没跑到，照「没跑到不判」只撤掉这一趟的强制现判）
    if !observers_on_the_nodes.is_empty() && stop_is_requested() {
        if let Some(sentinels) = &reuse_sentinels {
            sentinels.stop_forcing_the_forced_block(&mut library_at_each_read_or_write.take());
        }
        return Err(
            "叫停了（SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE）：观察者这一趟没判完，录过、判过的在库里，重跑同一条用例接着录、接着判"
                .to_string(),
        );
    }
    let pool_checker =
        pool_checker_of_the_images_on_the_route(route, gpu_judging_inputs.card_sources)?;
    let observers_judging_started = Instant::now();
    // 判器与观察者两段都判完才判哨兵（观察者结论也要核）：次序在两处判段共用的那一处
    let (observing, reuse_sentinel_problems, observers_kept_out_recorded_again) =
        crate::crash_amplification::judge_the_observers_then_the_reuse_sentinels(
            &library_at_each_read_or_write,
            flow,
            &plan,
            Some((observers_on_the_nodes, observer_versions, &pool_checker)),
            &is_mine,
            reuse_sentinels.as_ref(),
            &stop_is_requested,
        )?;
    if observers_kept_out_recorded_again.blocks_recorded_here > 0 {
        println!(
            "CRASH_POINT_REPLAY_OBSERVERS_KEPT_OUT_RECORDED_AGAIN item={} blocks_recorded_here={} states_recorded_here={}",
            item.name,
            observers_kept_out_recorded_again.blocks_recorded_here,
            observers_kept_out_recorded_again.states_recorded_here
        );
    }
    let observing = observing.expect("交了观察者那一格，观察者判段的结果就在");
    if !observers_on_the_nodes.is_empty() {
        // 观察者判段的「现判 N、复用 M」：这一趟在观察者交回的盘上判池级 checker 的张数（其中在卡上的）、库里这一把判器版本早先判过的张数
        println!(
            "CRASH_POINT_REPLAY_OBSERVERS_JUDGED item={} images_judged_fresh={} images_judged_on_the_gpu_here={} image_judgements_reused={} blocks_observed_here={} states_observed_here={} seconds={:.1}",
            item.name,
            observing.images_judged_here,
            observing.images_judged_on_the_gpu_here,
            observing.image_judgements_reused,
            observing.blocks_observed_here,
            observing.states_observed_here,
            observers_judging_started.elapsed().as_secs_f64()
        );
    }
    // 被叫停的那一趟：哨兵与强制现判那一块可能没跑到，照「没跑到不判」只清掉强制现判表（problems_after_judging 已清）
    if let Some(message) =
        reuse_sentinel_red_message(item.name, stop_is_requested(), &reuse_sentinel_problems)
    {
        return Err(message);
    }
    // 这一份自己领的块上的观察者红当场红（排在双机那一份收工之前：分片那一份也报自己那一半；别的机器领的块上存着的红由不分片的那一趟
    // 从整条流的观察者结论里读，见下面重建计数那一步）
    if let Some(message) = red_message_of_the_observers_on_the_blocks_of_this_share(&observing) {
        panic!("{message}");
    }
    // 对账：这条流的块里还没有这一把版本结论的
    let unjudged = unjudged_blocks_of_the_flow(
        &library_at_each_read_or_write.take(),
        &plan,
        &judged_conclusions,
    )
    .map_err(|error| format!("对账失败：{error:?}"))?;
    if !unjudged.is_empty() && stop_is_requested() {
        return Err(format!(
            "叫停了（SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE）：{} 块还没判，判过的在库里，重跑同一条用例接着判",
            unjudged.len()
        ));
    }
    // 观察者判段只判这台机器领的块：别的机器领的块这一把版本的观察者结论由那台机器写进服务那一份；只挂点名段的观察者在它不挂的点上不要结论
    let without_every_conclusion = blocks_of_the_flow_without_every_conclusion(
        &library_at_each_read_or_write.take(),
        &plan,
        &judged_conclusions,
        &observers_in_the_account,
    )?;
    if this_share_leaves_the_tally_to_the_whole_flow_run(
        share.count,
        unjudged.len(),
        without_every_conclusion.missing_an_observer_conclusion,
    ) {
        judging_units_of_the_run
            .blocks_of_the_run
            .set(Some(crash_case_blocks_of_the_share(
                &library_at_each_read_or_write.take(),
                &plan,
                &judged_conclusions,
                &observers_in_the_account,
                &is_counted,
                blocks_before_judging,
                judging_units_of_the_run.are_refused,
            )?));
        let blocks_left: u64 =
            without_every_conclusion.missing_the_judgement_or_an_observer_conclusion;
        println!(
                "CRASH_POINT_REPLAY_PARALLEL_FINISHED states={state_count} slices={blocks_of_this_share} worker_threads={worker_threads} configured_worker_threads={worker_threads} worker_threads_source={worker_threads_source} resumed_slices={blocks_reused} freshly_run_slices={blocks_to_judge} progress_file_after_completion=none elapsed_seconds={:.1} judge={}{share_fields} blocks_left_to_other_hosts={blocks_left}",
                started.elapsed().as_secs_f64(),
                route.name()
            );
        return Ok(StageOutcome::JudgedMyShareOnly {
            blocks_judged_here: blocks_to_judge,
            blocks_left_to_other_hosts: blocks_left,
        });
    }
    if !unjudged.is_empty() {
        return Err(format!(
            "判段判完库里还有 {} 块没有这一把版本的结论（第一块：{}）：判器那一路没写库",
            unjudged.len(),
            unjudged[0].0.short_text()
        ));
    }
    // 计数向量每一趟都判（J7）
    let disagreements = count_vector_disagreements_of_the_flow(
        &library_at_each_read_or_write.take(),
        flow,
        &plan,
        &judged_conclusions,
        &|_| true,
    )
    .map_err(|error| format!("读计数向量失败：{error:?}"))?;
    if !disagreements.is_empty() {
        return Err(format!(
            "计数向量与计划对不上 {} 处：{}",
            disagreements.len(),
            disagreements.join("；")
        ));
    }
    judging_units_of_the_run
        .blocks_of_the_run
        .set(Some(crash_case_blocks_of_the_share(
            &library_at_each_read_or_write.take(),
            &plan,
            &judged_conclusions,
            &observers_in_the_account,
            &is_counted,
            blocks_before_judging,
            judging_units_of_the_run.are_refused,
        )?));
    let (tally, observer_red_of_the_flow) = crash_point_replay_tally_from_the_library(
        &library_at_each_read_or_write.take(),
        flow,
        &plan,
        judged_by.conclusion_kind(),
        conclusion_versions.versions_of_each_point_when_judged_by(judged_by),
        &observers_in_the_account,
    )?;
    // 整条流的块上观察者结论里存着的红（别的机器写进服务的、库里早先存着的、这一趟看的块都读）：别的机器领的块上的观察者红在这里报
    if let Some(message) = red_message_of_the_observers_on_the_whole_flow(
        &observer_red_of_the_flow,
        observers_on_the_nodes,
    ) {
        panic!("{message}");
    }
    report_the_findings(&tally, item, state_count);
    println!(
        "CRASH_POINT_REPLAY_PARALLEL_FINISHED states={state_count} slices={blocks_of_this_share} worker_threads={worker_threads} configured_worker_threads={worker_threads} worker_threads_source={worker_threads_source} resumed_slices={blocks_reused} freshly_run_slices={blocks_to_judge} progress_file_after_completion=none elapsed_seconds={:.1} judge={}{share_fields}",
        started.elapsed().as_secs_f64(),
        route.name()
    );
    Ok(StageOutcome::Judged(tally))
}

/// 判完之后复用哨兵判出的红（[`crate::crash_amplification::ReuseSentinelsOfTheRun::problems_after_judging`]，一句一处）连成这条用例的那一句；
/// 没有红、或这一趟被叫停（哨兵与强制现判那一块可能没跑到，照「没跑到不判」）是 `None`。
fn reuse_sentinel_red_message(
    item_name: &str,
    stop_was_requested: bool,
    problems: &[String],
) -> Option<String> {
    (!stop_was_requested && !problems.is_empty()).then(|| {
        format!(
            "崩溃枚举用例 {item_name} 复用哨兵判红：{}",
            problems.join("；")
        )
    })
}

/// 这一份自己领的块上观察者判红的那一句（这一趟看的与库里复用的块都算，`crate::crash_amplification::judge_the_observers_of_the_flow` 只走这一份的块）；没有红是 `None`。
fn red_message_of_the_observers_on_the_blocks_of_this_share(
    observing: &ObservingOfTheRun,
) -> Option<String> {
    if let Some(red) = observing.red_texts.first() {
        return Some(format!(
            "观察者在 {} 个状态上 panic（第一个：路径 {} 点内第 {} 个状态）：{}",
            observing.red_states_of_each_observer.values().sum::<u64>(),
            red.0,
            red.1,
            red.2
        ));
    }
    observing
        .red_states_of_each_observer
        .iter()
        .find(|(_, red_states)| **red_states > 0)
        .map(|(observer_name, red_states)| {
            format!(
                "观察者 {observer_name} 在 {red_states} 个状态上 panic（这一趟复用的块里存着红项，这一趟没看：改了观察者代码它的版本就换、会重看）"
            )
        })
}

/// 整条流的块上观察者结论里存着红的那一句（[`ObserverRedOfTheFlow`]，`observers` 与重建计数时给的观察者同序）；没有红是 `None`。
///
/// # Panics
/// 有观察者的红状态数不为 0 而没记下头一块存着红的（数的那一步漏记，是这里的 bug）。
fn red_message_of_the_observers_on_the_whole_flow(
    observer_red: &ObserverRedOfTheFlow,
    observers: &[ObserverOnTheNodes<'_>],
) -> Option<String> {
    let (observer_position, red_states) = observer_red
        .red_states_of_each_observer
        .iter()
        .enumerate()
        .find(|(_, red_states)| **red_states > 0)?;
    let first_red_block = observer_red
        .first_red_block
        .as_ref()
        .expect("有观察者的红状态数不为 0，数的时候就记下了头一块存着红的");
    Some(format!(
        "观察者 {} 在整条流的块上存着 {red_states} 个红状态（头一块存着红的：观察者 {} 点 {} 块 {}；别的机器写进服务的、库里早先存着的与这一趟看的块都读）",
        observers[observer_position].observer_name.as_str(),
        observers[first_red_block.observer_position]
            .observer_name
            .as_str(),
        first_red_block.point_name,
        first_red_block.block_start
    ))
}

/// 双机的一份（`share_count` 大于 1）：判器结论或观察者结论有一种对这条流不齐（别的机器领的块那台还没判完、没写进服务那一份），
/// 就只交自己那一份（`StageOutcome::JudgedMyShareOnly`），整条流的计数留给两份都判完之后不分片的那一趟（它照服务里那一份复用两台判过的块）。
/// 判器结论全复用而观察者版本这一趟变了时，缺的只有观察者结论，也在这里收工（双机四条用例两份都 panic 在重建计数读不到对方那几块的
/// 观察者结论）。整份自己跑的不收工：缺结论照旧交错。
fn this_share_leaves_the_tally_to_the_whole_flow_run(
    share_count: usize,
    blocks_without_the_judgement: usize,
    blocks_missing_an_observer_conclusion: usize,
) -> bool {
    share_count > 1
        && (blocks_without_the_judgement > 0 || blocks_missing_an_observer_conclusion > 0)
}

/// 逐块问判器结论（`judged_conclusions` 那一处）与挂在这一块那个点上的每个观察者结论（[`observers_holding_the_point`]）在不在库里，
/// 只问在不在、不读内容。
///
/// # Errors
/// 库读失败。
pub(super) fn blocks_of_the_flow_without_every_conclusion(
    store: &VerdictStore,
    plan: &CrashPlan,
    judged_conclusions: &crate::crash_amplification::ConclusionsStoredUnder<'_>,
    observers: &[ObserverInTheBlockAccount],
) -> Result<BlocksOfTheFlowWithoutEveryConclusion, String> {
    let mut missing_an_observer_conclusion = 0usize;
    let mut missing_the_judgement_or_an_observer_conclusion = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        let observer_kinds_and_versions: Vec<(ConclusionKind, ConclusionVersion)> =
            observers_holding_the_point(observers, point_index)
                .into_iter()
                .map(|(_position, kind_and_version)| kind_and_version)
                .collect();
        for local in point.blocks() {
            let key = point.block_key(&local);
            let is_missing_an_observer_conclusion = !store
                .conclusions_missing_for_block(&key, &observer_kinds_and_versions)
                .map_err(|error| format!("读观察者的结论在不在失败：{error:?}"))?
                .is_empty();
            let is_missing_the_judgement = !judged_conclusions
                .is_stored(store, point_index, &key)
                .map_err(|error| format!("读判器的结论在不在失败：{error:?}"))?;
            if is_missing_an_observer_conclusion {
                missing_an_observer_conclusion += 1;
            }
            if is_missing_an_observer_conclusion || is_missing_the_judgement {
                missing_the_judgement_or_an_observer_conclusion += 1;
            }
        }
    }
    Ok(BlocksOfTheFlowWithoutEveryConclusion {
        missing_an_observer_conclusion,
        missing_the_judgement_or_an_observer_conclusion,
    })
}

/// 这一趟的块账：点到的块（`is_counted`：这一份领的块，不分片的那一趟是整条流）、复用（判之前库里罩住的，
/// `blocks_before_judging`）、现判（判之前没罩住的）、判红（判完之后判器结论或观察者结论里有违例的块，[`blocks_in_the_library`]）；
/// 判器那一半哪一组拒算时点到的块全部计入拒算（判了，拒算那几组照判不入库，这条用例的块在库里罩不全）。
///
/// # Errors
/// 库读失败。
fn crash_case_blocks_of_the_share(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &crate::crash_amplification::ConclusionsStoredUnder<'_>,
    observers: &[ObserverInTheBlockAccount],
    is_counted: &dyn Fn(u64) -> bool,
    blocks_before_judging: BlocksInTheLibrary,
    judging_units_are_refused: bool,
) -> Result<CrashCaseBlocks, String> {
    let blocks_after_judging =
        blocks_in_the_library(store, plan, conclusions, observers, is_counted)?;
    // 点到的块（这一份领的块；不分片的那一趟是整条流）与判之前没罩住的块
    let blocks_of_this_share = blocks_before_judging.blocks;
    let blocks_judged_here = blocks_before_judging.blocks - blocks_before_judging.covered;
    let (reused, fresh, refused) = if judging_units_are_refused {
        (0, 0, blocks_of_this_share)
    } else {
        (
            blocks_of_this_share - blocks_judged_here,
            blocks_judged_here,
            0,
        )
    };
    Ok(CrashCaseBlocks {
        blocks: blocks_of_this_share,
        reused,
        fresh,
        red: blocks_after_judging.red,
        refused,
        states: blocks_before_judging.states,
        expand_up_to: None,
    })
}

/// 观察者交回的盘上的池级 checker 照第 ① 段的选路：CPU 那一路由 CPU 判；GPU 那一路在卡上判，卡在头一张要判的盘那一刻才取
/// （[`pool_checker_of_the_images_on_the_cards`]）：判段等完 GPU 批处理服务之后，观察者一张盘都不交回的用例一张卡都不取、不读显卡配置。
/// `card_sources_of_the_entry` 是判段入口照起步那一刻的环境记下的显卡来路，CPU 那一路用不上。
///
/// # Errors
/// 选路走 GPU 而这份二进制没编进 gpu 特性。
fn pool_checker_of_the_images_on_the_route(
    route: CrashPointReplayJudgeRoute,
    card_sources_of_the_entry: CardSourcesOfTheEngineStart,
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    match route {
        CrashPointReplayJudgeRoute::Cpu { .. } => {
            Ok(PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu)
        }
        CrashPointReplayJudgeRoute::Gpu => {
            pool_checker_of_the_images_on_the_cards(card_sources_of_the_entry)
        }
        CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature => unreachable!(
            "判段不跑的那一趟在 enumerate_crash_states_through_the_library 的判段闭包里先交只录，不判观察者交回的盘"
        ),
    }
}

/// GPU 那一路：建的时候不取卡（[`crate::crash_amplification::CardsTakenAtTheFirstImageToJudge`]）；头一张要判的盘那一刻照判段入口记下的多机配置与
/// 服务交的卡取（`card_sources_of_the_entry`，[`crate::crash_amplification::gpu_cards_of_this_run_from_the_multi_host_configuration`]），
/// 取不到（调用方没交卡）的判表交 GPU 批处理服务判
/// （[`crate::crash_amplification::WayToTheGpuVerificationServer::TheRealService`]），不退回 CPU 判。
#[cfg(feature = "gpu")]
#[allow(
    clippy::unnecessary_wraps,
    reason = "没编进 gpu 特性的那一份报错，两份同一个签名"
)]
fn pool_checker_of_the_images_on_the_cards(
    card_sources_of_the_entry: CardSourcesOfTheEngineStart,
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    Ok(
        PoolCheckerOfTheImagesAfterTheObservers::TakingTheGpuCardsAtTheFirstImageToJudge(
            crate::crash_amplification::CardsTakenAtTheFirstImageToJudge::new(
                card_sources_of_the_entry,
                crate::crash_amplification::WayToTheGpuVerificationServer::TheRealService,
            ),
        ),
    )
}

/// 没编进 `gpu` 特性选路不会走 GPU（[`crash_point_replay_judge_route`]）。
#[cfg(not(feature = "gpu"))]
fn pool_checker_of_the_images_on_the_cards(
    _card_sources_of_the_entry: CardSourcesOfTheEngineStart,
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    Err("这份二进制没编进 gpu 特性，观察者交回的盘上不了卡（选路不该要它）".to_string())
}

/// 配了 GPU：读回清单里的判表，这台机器领的要判的块写成待判行、等 GPU 批处理服务判完（`crash_amplification::judge_my_blocks_on_gpu`：
/// 判段的库在临时 prune-server 里的先在真服务里登记那份临时库，不问服务收不收；不退回 CPU、不在这个进程里借卡判）。
#[cfg(feature = "gpu")]
fn judge_the_pending_blocks_through_the_gpu_verification_server(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    plan: &CrashPlan,
    manifest: &ItemManifest,
    is_mine: &dyn Fn(u64) -> bool,
    judged_conclusions: &crate::crash_amplification::ConclusionsStoredUnder<'_>,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<(), String> {
    use crate::crash_amplification::{judge_my_blocks_on_gpu, packed_judge_tables_read_back};
    // 判表读回：库只在读字节那一次往返里取，核摘要与解码在锁外
    let tables =
        packed_judge_tables_read_back(library, &manifest.digest_named(MANIFEST_JUDGE_TABLES)?)?;
    let (tally, outcome) = judge_my_blocks_on_gpu(
        library,
        &manifest.digest_named(MANIFEST_FLOW)?,
        plan,
        &tables,
        is_mine,
        judged_conclusions,
        stop_is_requested,
    )?;
    println!(
        "CRASH_POINT_REPLAY_JUDGE route=gpu cards_used={} blocks={} states={} red_states={} compile_seconds={:.1} judge_seconds={:.1} states_per_second={:.0}",
        outcome.cards_used,
        outcome.blocks_judged,
        outcome.states_judged,
        tally.red_states,
        outcome.compile_seconds,
        outcome.judge_seconds,
        outcome.states_per_second()
    );
    Ok(())
}

/// 没编进 `gpu` 特性选路不会要卡（[`crash_point_replay_judge_route`]）。
#[cfg(not(feature = "gpu"))]
fn judge_the_pending_blocks_through_the_gpu_verification_server(
    _library: &LibraryTakenAtEachReadOrWrite<'_>,
    _plan: &CrashPlan,
    _manifest: &ItemManifest,
    _is_mine: &dyn Fn(u64) -> bool,
    _judged_conclusions: &crate::crash_amplification::ConclusionsStoredUnder<'_>,
    _stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<(), String> {
    Err("这份二进制没编进 gpu 特性，上不了卡（选路不该要它）".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "gpu")]
    use crate::crash_amplification::CrashAmplificationEnvironment;
    #[cfg(feature = "gpu")]
    use crate::verdict_store::PruneServerFailureHandling;

    /// 双机的一份判器结论全复用、别的机器领的块缺这一趟那一把版本的观察者结论（双机四条用例两份都 panic 的那一格）：
    /// 这一份照样只交自己那一份、整条流的计数留给不分片的那一趟；判器结论缺的照旧收工；两种都齐的照常重建计数；整份自己跑的缺观察者结论不收工
    /// （照旧交错）。判别力（`crates/mutations.tsv`）：收工的条件去掉观察者那一半（改回只看判器结论），这条红。
    #[test]
    fn a_share_with_every_judge_conclusion_reused_still_leaves_the_tally_when_an_observer_conclusion_is_missing(
    ) {
        assert!(
            this_share_leaves_the_tally_to_the_whole_flow_run(2, 0, 1),
            "判器结论全齐、别的机器那几块缺观察者结论：只交自己那一份"
        );
        assert!(
            this_share_leaves_the_tally_to_the_whole_flow_run(2, 1, 0),
            "判器结论缺：只交自己那一份（改前就是这样）"
        );
        assert!(
            !this_share_leaves_the_tally_to_the_whole_flow_run(2, 0, 0),
            "两种结论都齐：照常重建计数"
        );
        assert!(
            !this_share_leaves_the_tally_to_the_whole_flow_run(1, 0, 1),
            "整份自己跑：缺观察者结论照旧交错，不收工"
        );
        assert!(
            !this_share_leaves_the_tally_to_the_whole_flow_run(1, 1, 0),
            "整份自己跑：缺判器结论照旧交错，不收工"
        );
    }

    /// 「配 GPU 必用 GPU、做不到报错不降级」：配置开了 GPU 而这份二进制没编进 gpu 特性，判段不跑（不退回 CPU 判），
    /// 结论版本按配置的那一路（GPU）；打的本次未跑那一行是门禁 crash-point-replay 的 `test_not_run_lines` 认的那一族记号、带着项名，
    /// 不是 `cpu_judging_lines` 认的 `route=cpu`。
    #[test]
    fn the_gpu_switched_on_without_the_gpu_feature_routes_the_judge_to_not_run_and_never_to_the_cpu(
    ) {
        assert_eq!(
            crash_point_replay_judge_route_of_the_switches(true, false),
            CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature
        );
        assert_eq!(
            crash_point_replay_judge_route_of_the_switches(true, true),
            CrashPointReplayJudgeRoute::Gpu
        );
        for gpu_feature_is_compiled_in in [false, true] {
            assert!(matches!(
                crash_point_replay_judge_route_of_the_switches(false, gpu_feature_is_compiled_in),
                CrashPointReplayJudgeRoute::Cpu { .. }
            ));
        }
        assert_eq!(
            CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature.judged_by(),
            JudgedBy::GpuJudge
        );
        let line = crash_point_replay_judge_not_run_line("some_crash_enumeration_item");
        assert!(
            line.starts_with(
                "NOT_RUN_NO_GPU_ADAPTER test=crash_state_enumeration::some_crash_enumeration_item reason="
            ) && line.contains("ENABLE_GPU=1"),
            "{line}"
        );
        assert!(
            !line.contains("route=cpu") && !line.contains("judge=cpu"),
            "{line}"
        );
    }

    /// 这几条库单测的判路照这台机器的多机配置取（配 GPU 必用 GPU、做不到报错不降级；门禁 crash-point-replay 的
    /// `cpu_judging_lines` 判配了 GPU 的判段不许落在 CPU）：配了 GPU 的机器只有编进 gpu 特性的那一趟判得了，所以这几条只编进带特性的那一趟，
    /// 「没编进特性不跑判段」那一路在这里到不了。
    #[cfg(feature = "gpu")]
    fn configured_judge_route() -> CrashPointReplayJudgeRoute {
        match crash_point_replay_judge_route(
            &crate::crash_amplification::multi_host_configuration_of_this_process()
                .expect("多机配置读得出"),
        ) {
            route @ (CrashPointReplayJudgeRoute::Gpu | CrashPointReplayJudgeRoute::Cpu { .. }) => {
                route
            }
            CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature => {
                unreachable!("这几条编进了 gpu 特性，判路不会是「没编进特性不跑判段」")
            }
        }
    }

    /// 两块盘 mkfs 一次的池（io_min 512、槽距 4096，没有文件）：GPU 判表要底座读得出系统配置（`crash_judge_tables::JudgeTablesRefusal::BaseSystemConfigurationUnreadable`），
    /// 没格式化的空池它建不了表。与 `crash_judge_tables` 单测里的造法同一种，两处各留一份（测试模块之间不互相导入）。
    #[cfg(feature = "gpu")]
    fn formatted_pool_of_two_devices_for_the_probe() -> MemoryPool {
        use crate::memory_pool::SparseBlockDevice;
        use singlefs_core::block_device::PhysicalBlockSizeInBytes;
        use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
        use singlefs_core::root_ring::RootRingSlotsPerRegion;
        use singlefs_core::system_configuration::SystemImmutableSizes;
        let device_bytes: u64 = 4 << 30;
        let mut devices: Vec<(singlefs_core::address::DeviceIdentity, SparseBlockDevice)> = [
            singlefs_core::address::DeviceIdentity(0),
            singlefs_core::address::DeviceIdentity(1),
        ]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
        make_filesystem(
            &MakeFilesystemParameters {
                filesystem_identifier: *b"singlefs-probe!!",
                region_devices: [
                    singlefs_core::address::DeviceIdentity(0),
                    singlefs_core::address::DeviceIdentity(1),
                    singlefs_core::address::DeviceIdentity(0),
                ],
                geometry: SystemImmutableSizes {
                    physical_block_size: 512,
                    minimum_input_output_bytes: 512,
                    fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
                    journal_ring_bytes: singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                    root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
                },
            },
            &mut devices,
        )
        .expect("两块空盘上 mkfs 做得成");
        MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.image.clone()))
                .collect(),
            device_size_in_bytes: device_bytes,
        }
    }

    /// 探针流两次单元写写什么。两次都落在盘 1 单元区头两个槽的起点，那里坐着 mkfs 在单元区起点写下、底座的根引用的单元
    /// （`make_filesystem` 的 `instance_table_placement_at`），写什么决定单元写落下之后的块红不红。
    #[cfg(feature = "gpu")]
    enum ProbeUnitWriteContents {
        /// 两槽各写满一个物理块的填充字节（7 与 8）：盖掉被引用的单元，池级 checker 判 I-2.1、I-4.8、I-7.4，单元写落下之后的块都红。
        FillerBytesOverReferencedUnits,
        /// 两槽各照底座在那里原有的字节重写一个物理块：单元写落下之后的镜像与底座逐字节相同，那几块与什么都没落下的那一块一样判不红。
        TheBaseBytesWrittenAgain,
    }

    /// 探针流的三次写，每次都落在 GPU 判表认的一个落点的起点上（`crash_judge_tables::JudgeTablesRefusal::WriteNotAlignedToALocation` 拒起点不在落点上的写）：
    /// 两次单元写各在单元区头两个槽的起点（盘 1，内容照 `unit_write_contents`，取底座字节的从 `base` 读），一次根槽写在根环区域 0 的 0 号槽（盘 0，长度一个物理块），
    /// 根身份写在偏移 24（实例 1）与 28（checkpoint_txg 1）。
    #[cfg(feature = "gpu")]
    fn probe_writes_on_the_locations_of(
        base: &MemoryPool,
        unit_write_contents: ProbeUnitWriteContents,
    ) -> Vec<RetainedWrite> {
        use crate::memory_pool::WrittenContents;
        use singlefs_core::journal::slot_after_the_journal_ring;
        /// 每次写的字节数：一个物理块（[`formatted_pool_of_two_devices_for_the_probe`] 的池物理块宽 512）。
        const PROBE_WRITE_BYTES: usize = 512;
        let unit_device = singlefs_core::address::DeviceIdentity(1);
        let unit_area_start =
            slot_after_the_journal_ring(singlefs_format::JOURNAL_RING_DEFAULT_BYTES).0
                * singlefs_format::SLOT_BYTES;
        let unit_write = |slot_in_the_unit_area: u64, filler_byte: u8| {
            let offset = singlefs_core::address::DeviceOffsetInBytes(
                unit_area_start + slot_in_the_unit_area * singlefs_format::SLOT_BYTES,
            );
            let bytes = match unit_write_contents {
                ProbeUnitWriteContents::FillerBytesOverReferencedUnits => {
                    vec![filler_byte; PROBE_WRITE_BYTES]
                }
                ProbeUnitWriteContents::TheBaseBytesWrittenAgain => base
                    .devices
                    .get(&unit_device)
                    .expect("底座是两块盘 mkfs 出的池，盘 1 在里面")
                    .read(offset, PROBE_WRITE_BYTES),
            };
            RetainedWrite {
                device: unit_device,
                kind: StepKind::UnitWrite,
                is_force_unit_access: false,
                offset,
                contents: WrittenContents::Bytes(bytes),
            }
        };
        let mut root_record = vec![0u8; PROBE_WRITE_BYTES];
        root_record[24..28].copy_from_slice(&1u32.to_le_bytes());
        root_record[28..36].copy_from_slice(&1u64.to_le_bytes());
        vec![
            unit_write(0, 7),
            unit_write(1, 8),
            RetainedWrite {
                device: singlefs_core::address::DeviceIdentity(0),
                kind: StepKind::RootRecordFua,
                is_force_unit_access: true,
                offset: singlefs_core::root_ring::region_start(0),
                contents: WrittenContents::Bytes(root_record),
            },
        ]
    }

    /// 一条用例在给定的库上过两段（判路由调用方给，取 [`configured_judge_route`]），交回结局与块账：库是这个进程开的那一份（用例先把一次性库设成它），用例在
    /// [`CrashAmplificationEnvironment`] 里起。
    /// 流是两段单元写加一段根槽写（[`probe_writes_on_the_locations_of`]，单元写的内容照 `unit_write_contents`），被判的那一次写是那条根槽写
    /// （「根槽已持久」只按根槽写两态数，被判的写不是根槽写时判段交错）。
    #[cfg(feature = "gpu")]
    fn run_the_unit_writes_case(
        route: CrashPointReplayJudgeRoute,
        keys: KeysOfTheItem,
        unit_write_contents: ProbeUnitWriteContents,
    ) -> (StageOutcome<CrashPointReplayTally>, Option<CrashCaseBlocks>) {
        run_the_unit_writes_case_with_the_observers(route, keys, unit_write_contents, &[])
    }

    /// 单元写那条用例的项（项名与设点文件）：它自己那个点与末节点都归它（[`nodes_of_the_item`]）。
    #[cfg(feature = "gpu")]
    fn the_unit_writes_case_item() -> CrashPointReplayItem<'static> {
        CrashPointReplayItem {
            name: "crash_case_blocks_probe",
            setting_file: "crates/singlefs-verification/src/crash_state_enumeration.rs",
        }
    }

    /// 同 [`run_the_unit_writes_case`]，另挂 `observers_on_the_nodes`（它们那几把版本在 `keys` 里，同序）。
    #[cfg(feature = "gpu")]
    fn run_the_unit_writes_case_with_the_observers(
        route: CrashPointReplayJudgeRoute,
        keys: KeysOfTheItem,
        unit_write_contents: ProbeUnitWriteContents,
        observers_on_the_nodes: &[ObserverOnTheNodes<'_>],
    ) -> (StageOutcome<CrashPointReplayTally>, Option<CrashCaseBlocks>) {
        let base = formatted_pool_of_two_devices_for_the_probe();
        let writes = probe_writes_on_the_locations_of(&base, unit_write_contents);
        let segments: Vec<Vec<usize>> = (0..writes.len()).map(|index| vec![index]).collect();
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("造的流最后一段是一次根槽写");
        let item = the_unit_writes_case_item();
        let flow_to_record = flow_of_the_inputs(
            item,
            &base,
            &writes,
            &segments,
            judged_root_index,
            CrashPointReplayVersionTable::with_a_file_only(&[]),
            &every_proper_subset,
        );
        // 判段走 GPU 那一路要的两样照入口那样在用例的环境守卫里读（多机配置是这一刻读到的那一份）
        let configuration = crate::crash_amplification::multi_host_configuration_of_this_process()
            .expect("多机配置读得出");
        enumerate_the_flow_through_the_library_with_the_keys(
            item,
            &flow_to_record,
            observers_on_the_nodes,
            route,
            keys,
            GpuJudgingInputsOfTheEntry::read_at_the_entry(configuration.configuration_file),
        )
        .expect("两段过得了库")
    }

    #[cfg(feature = "gpu")]
    fn computed_keys(route: CrashPointReplayJudgeRoute) -> KeysOfTheItem {
        keys_of_the_item(
            route,
            &[],
            ObserverJudgingUnitsKeyTable::OfTheVerificationPackage,
        )
    }

    /// 第二节第 9 条与 `CRASH_CASE_BLOCKS` 接口：头一趟现判、写进库，`blocks = reused + fresh + refused`，那一行五个字段次序照 admission.py。
    /// 判别力（`crates/mutations.tsv`）：那一行少打一个字段，这条红。
    #[cfg(feature = "gpu")]
    #[test]
    fn the_crash_case_blocks_line_counts_the_blocks_of_a_fresh_run() {
        // 一次性库留着：钉头一趟现判的块账行，要从空库起跑
        let disposable = crate::crash_library::DisposableLibrary::new("crash-case-blocks")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        // 照调用方的多机配置判（副本里只经 SINGLEFS_MULTI_HOST_CONFIG 交进来），走 GPU 那一路要卡：照服务交的卡取（守卫清掉了这两个变量）
        environment.set_the_multi_host_configuration_and_the_cards_of_the_caller();
        let route = configured_judge_route();
        let (judged, fresh_run) = run_the_unit_writes_case(
            route,
            computed_keys(route),
            ProbeUnitWriteContents::FillerBytesOverReferencedUnits,
        );
        assert!(judged.judged().is_some(), "头一趟两段先后跑、交判的结果");
        let fresh_run = fresh_run.expect("判段跑了就有块账");
        assert!(fresh_run.blocks > 0);
        assert_eq!(
            (fresh_run.reused, fresh_run.fresh, fresh_run.refused),
            (0, fresh_run.blocks, 0),
            "空库头一趟全部现判"
        );
        assert!(fresh_run.states > 0, "判段那一趟数了块里的状态");
        assert_eq!(
            fresh_run.blocks,
            fresh_run.reused + fresh_run.fresh + fresh_run.refused
        );
        assert_eq!(
            fresh_run.line(),
            format!(
                "CRASH_CASE_BLOCKS blocks={} reused={} fresh={} red={} refused={} states={}",
                fresh_run.blocks,
                fresh_run.reused,
                fresh_run.fresh,
                fresh_run.red,
                fresh_run.refused,
                fresh_run.states
            )
        );
    }

    /// 红批第 5 条与 crates 第 3 条：拒算的只是观察者那一组（判器的代码不进键、不会拒算），判器的结论照键入库
    /// 同一份长期库、一块不少；这条用例有一组拒算，块照旧全部计入 `refused=`。钉「拒算的组一块不入、别的组照入」。
    /// 判别力（`crates/mutations.tsv`）：判段把照判不入库的版本当成镜像判定的版本，判器的结论也照判不入库，这条红。
    #[cfg(feature = "gpu")]
    #[test]
    fn a_case_whose_other_judging_units_are_refused_stores_every_judge_block_in_the_library() {
        // 一次性库留着：钉别的组拒算时判器结论照入库，要数库里的结论
        let disposable =
            crate::crash_library::DisposableLibrary::new("refused-observer-judging-units")
                .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        // 照调用方的多机配置判（副本里只经 SINGLEFS_MULTI_HOST_CONFIG 交进来），走 GPU 那一路要卡：照服务交的卡取（守卫清掉了这两个变量）
        environment.set_the_multi_host_configuration_and_the_cards_of_the_caller();
        let refused_observer_units = crate::crash_identity::JudgingUnits::ObserverCode {
            observer_code_file: "crates/singlefs-verification/src/crash_state_enumeration.rs"
                .to_string(),
            observer_name: "refused_in_the_unit_test".to_string(),
        };
        let (judged, blocks) = run_the_unit_writes_case(
            configured_judge_route(),
            KeysOfTheItem {
                observer_versions: Vec::new(),
                refusal_sentences: vec![
                    "判法单元 observer_code:crates/singlefs-verification/src/crash_state_enumeration.rs:refused_in_the_unit_test 拒算（单测造的）"
                        .to_string(),
                ],
                versions_kept_out_of_the_library: vec![
                    *crate::crash_amplification::placeholder_judging_units_key_of_the_refused(
                        &refused_observer_units,
                    )
                    .fingerprint(),
                ],
            },
            ProbeUnitWriteContents::FillerBytesOverReferencedUnits,
        );
        assert!(
            judged.judged().expect("判段交判的结果").states > 0,
            "状态都判了"
        );
        let blocks = blocks.expect("判段跑了就有块账");
        assert!(blocks.blocks > 0);
        assert_eq!(
            (blocks.reused, blocks.fresh, blocks.refused),
            (0, 0, blocks.blocks),
            "有一组拒算，这条用例的块全部计入 refused"
        );
        let store = disposable
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .expect("录入段照录进同一份库");
        assert_eq!(
            u64::try_from(
                store
                    .stored_conclusion_keys()
                    .expect("列得出库里的结论")
                    .into_iter()
                    .filter(|(_key, kind, _version)| *kind == ConclusionKind::ImageJudgement)
                    .count()
            )
            .expect("块数装得进 u64"),
            blocks.blocks,
            "判器那一组没拒算：它的结论每一块都照键进了长期库"
        );
    }

    /// 场景：同一份库（测试拿着一份开着的库，几趟共用这个进程里同一份 `VerdictStore`）上那条单元写的用例照键复用跑两趟；第二趟设复用哨兵的种子、
    /// 抽 1 块当哨兵。两次单元写照底座原有的字节重写（[`ProbeUnitWriteContents::TheBaseBytesWrittenAgain`]）：哨兵与强制现判只从不红的块里抽，
    /// 根槽写落下之前的几块要判不红；写填充字节的那一种盖掉被引用的单元，只有什么都没落下的那一块不红，凑不出一块哨兵加一块强制现判。
    /// 预期：第二趟照常交判的结果、块账除记着红的块之外全部复用（强制现判那一块按复用算）；这一趟交来存过的块是记着红的那几块加恰好一块
    /// （抽中强制现判的那一块现判、判完照键写回，哨兵与别的不红的块都复用）：崩溃枚举用例那一路接上块级复用哨兵与强制现判（与崩溃放量那一路同一套，`.claude/rules/test-script-discipline.md`
    /// 「判定可复用：按输入存，按前缀剪」每一趟真跑插哨兵）。
    /// 判别力（`crates/mutations.tsv`）：「崩溃枚举用例：判段不抽复用哨兵」这一行让这一条红。
    #[cfg(feature = "gpu")]
    #[test]
    fn a_case_judged_again_on_the_same_library_draws_reuse_sentinels_and_judges_only_the_forced_block_again(
    ) {
        // 一次性库留着：钉头一趟现判、第二趟抽哨兵与强制现判，要从空库起跑
        let disposable = crate::crash_library::DisposableLibrary::new("crash-case-reuse-sentinels")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        // 照调用方的多机配置判（副本里只经 SINGLEFS_MULTI_HOST_CONFIG 交进来），走 GPU 那一路要卡：照服务交的卡取（守卫清掉了这两个变量）
        environment.set_the_multi_host_configuration_and_the_cards_of_the_caller();
        let held_library = CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration)
            .expect("开得了设成这个进程那一份的一次性库");
        let route = configured_judge_route();
        let (judged, fresh_run) = run_the_unit_writes_case(
            route,
            computed_keys(route),
            ProbeUnitWriteContents::TheBaseBytesWrittenAgain,
        );
        assert!(judged.judged().is_some(), "头一趟交判的结果");
        let fresh_run = fresh_run.expect("判段跑了就有块账");
        assert!(
            fresh_run.blocks - fresh_run.red >= 2,
            "前提：至少两块不红（红块不当哨兵的候选；一块当哨兵、一块强制现判）"
        );
        environment.set_variable(
            crate::crash_amplification::REUSE_SENTINEL_SEED_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new("7"),
        );
        environment.set_variable(
            crate::crash_amplification::REUSE_SENTINEL_COUNT_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new("1"),
        );
        let offered_before_the_rerun = held_library.lock().conclusion_blocks_offered_so_far();
        let (rerun, rerun_blocks) = run_the_unit_writes_case(
            route,
            computed_keys(route),
            ProbeUnitWriteContents::TheBaseBytesWrittenAgain,
        );
        assert!(
            rerun.judged().is_some(),
            "第二趟照常交判的结果（哨兵一块都没被现判、强制现判那一块现判出同一个结论）"
        );
        let rerun_blocks = rerun_blocks.expect("判段跑了就有块账");
        assert_eq!(
            (rerun_blocks.reused, rerun_blocks.fresh),
            (rerun_blocks.blocks - fresh_run.red, fresh_run.red),
            "强制现判那一块按复用算：除了记着红的块（下一趟现判），块账全部复用"
        );
        assert_eq!(
            u64::try_from(
                held_library
                    .lock()
                    .conclusion_blocks_offered_since(offered_before_the_rerun)
                    .len()
            )
            .expect("块数装得进 u64"),
            fresh_run.red + 1,
            "第二趟交来存过的块：记着红的那几块，加抽中强制现判的那一块"
        );
        drop(held_library);
    }

    /// 单元写那条用例挂的观察者：只记计数、每个状态都不 panic（[`observe_no_crash_state`]），挂在整条流上（它自己那个点与末节点）。
    #[cfg(feature = "gpu")]
    fn the_unit_writes_case_observer() -> ObserverOnTheNodes<'static> {
        ObserverOnTheNodes {
            observer_name: ObserverName::new("crash_case_reuse_sentinel_probe_observer")
                .expect("名字合规矩"),
            observer_code_file: the_unit_writes_case_item().setting_file.to_string(),
            extra_inputs_digest: [5; DIGEST_BYTES],
            nodes: nodes_of_the_item(the_unit_writes_case_item()),
            observe: ObserverCode::CountsOnly(&observe_no_crash_state),
        }
    }

    /// [`the_unit_writes_case_observer`] 那几把版本：没登记入口的观察者取键拒算（照判、不入库），单测照同形另造摘要；判观察者交回的盘的
    /// 那一路照这一趟的判路（判段断言两边同一路）。
    #[cfg(feature = "gpu")]
    fn versions_of_the_unit_writes_case_observer(
        route: CrashPointReplayJudgeRoute,
    ) -> ObserverVersions {
        let observer = ConclusionVersion(sha256_digest(
            b"observer version of the crash case reuse sentinel probe\n",
        ));
        let pool_checker_version = ConclusionVersion(sha256_digest(
            b"pool checker version of the crash case reuse sentinel probe\n",
        ));
        ObserverVersions {
            observer,
            observer_code: ConclusionVersion(sha256_digest(
                b"observer code version of the crash case reuse sentinel probe\n",
            )),
            pool_checker: route.judged_by(),
            pool_checker_version,
            conclusion: ConclusionVersion(sha256_digest(
                &[observer.0, pool_checker_version.0].concat(),
            )),
        }
    }

    /// 场景：同一份库上那条单元写的用例挂一个只记计数的观察者（整条流），照键复用跑两趟；第二趟设复用哨兵的种子、抽 1 块当哨兵（同上一条）。
    /// 预期：第二趟照常交判的结果——判哨兵在观察者判段之后（两处判段共用的
    /// `crate::crash_amplification::judge_the_observers_then_the_reuse_sentinels`），强制现判那一块上的观察者结论这一趟现拼、交来存过，
    /// 哨兵一处红都不报；第二趟交来存过的观察者结论恰好一份（强制现判那一块上的那一份，别的块上的都复用）。
    /// 判别力（`crates/mutations.tsv`）：「复用哨兵：判哨兵挪到观察者判段之前」这一行让这一条红（判哨兵时强制现判那一块上的观察者结论还没现拼，
    /// 强制现判表又被它清掉，报「仍复用」，第二趟交不回判的结果）。
    #[cfg(feature = "gpu")]
    #[test]
    fn a_case_with_an_observer_judged_again_judges_the_reuse_sentinels_after_the_observers() {
        // 一次性库留着：钉头一趟现判、第二趟抽哨兵与强制现判，要从空库起跑
        let disposable =
            crate::crash_library::DisposableLibrary::new("crash-case-reuse-sentinels-observer")
                .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        // 照调用方的多机配置判（副本里只经 SINGLEFS_MULTI_HOST_CONFIG 交进来），走 GPU 那一路要卡：照服务交的卡取（守卫清掉了这两个变量）
        environment.set_the_multi_host_configuration_and_the_cards_of_the_caller();
        let held_library = CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration)
            .expect("开得了设成这个进程那一份的一次性库");
        let route = configured_judge_route();
        let observer = the_unit_writes_case_observer();
        let keys_with_the_observer = || {
            let mut keys = computed_keys(route);
            keys.observer_versions
                .push(versions_of_the_unit_writes_case_observer(route));
            keys
        };
        let (judged, fresh_run) = run_the_unit_writes_case_with_the_observers(
            route,
            keys_with_the_observer(),
            ProbeUnitWriteContents::TheBaseBytesWrittenAgain,
            std::slice::from_ref(&observer),
        );
        assert!(judged.judged().is_some(), "头一趟交判的结果");
        let fresh_run = fresh_run.expect("判段跑了就有块账");
        assert!(
            fresh_run.blocks - fresh_run.red >= 2,
            "前提：至少两块不红（一块当哨兵、一块强制现判）"
        );
        environment.set_variable(
            crate::crash_amplification::REUSE_SENTINEL_SEED_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new("7"),
        );
        environment.set_variable(
            crate::crash_amplification::REUSE_SENTINEL_COUNT_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new("1"),
        );
        let offered_before_the_rerun = held_library.lock().conclusion_blocks_offered_so_far();
        let (rerun, _rerun_blocks) = run_the_unit_writes_case_with_the_observers(
            route,
            keys_with_the_observer(),
            ProbeUnitWriteContents::TheBaseBytesWrittenAgain,
            std::slice::from_ref(&observer),
        );
        assert!(
            rerun.judged().is_some(),
            "第二趟照常交判的结果（哨兵一处红都没报）"
        );
        assert_eq!(
            held_library
                .lock()
                .conclusion_blocks_offered_since(offered_before_the_rerun)
                .iter()
                .filter(|(_block_key, kind, _version)| *kind == ConclusionKind::Observer)
                .count(),
            1,
            "第二趟交来存过的观察者结论恰好一份：强制现判那一块上的那一份"
        );
        drop(held_library);
    }

    /// 替身 GPU 批处理服务（不上卡）：库里的每一行待判行，照那一行的块键与结论版本存一份干净的判器结论（状态数照行里的态序号区间）与
    /// 逐盘自证槽数，再删掉那一行（判段等服务判完那一路认的「判完」：待判行不在、结论连同逐盘自证槽数都在）。
    #[cfg(feature = "gpu")]
    fn stand_in_service_stores_a_clean_conclusion_of_every_pending_row(
        library: &LibraryTakenAtEachReadOrWrite<'_>,
    ) {
        let rows = library.take().every_gpu_pending_row().expect("扫得了库");
        for (stream, block_number, row) in rows {
            let mut store = library.take();
            store_a_conclusion_block_of_the_unit_test(
                &mut store,
                &row.recorded_block,
                &ConclusionKind::ImageJudgement,
                &row.conclusion_version,
                row.state_ordinals.end - row.state_ordinals.start,
                &[],
                None,
            );
            store
                .store_observer_block(
                    &row.recorded_block,
                    &ConclusionKind::ImageJudgement,
                    &row.conclusion_version,
                    b"self verified slots of the stand-in service",
                )
                .expect("存得下");
            store
                .delete_the_gpu_pending_row(&stream, block_number)
                .expect("删得了");
        }
    }

    /// 场景：配了 GPU（多机配置只写 `ENABLE_GPU=1`）、环境里没有服务交的卡；单元写那条探针流挂一个只记计数的观察者
    /// （[`the_unit_writes_case_observer`]，不交回盘），录入段录进一份空库；判器那一段写待判行、由替身服务判掉写回
    /// （[`stand_in_service_stores_a_clean_conclusion_of_every_pending_row`]）；之后照判段的次序取观察者交回的盘上的池级 checker
    /// （[`pool_checker_of_the_images_on_the_route`]，走 GPU）、判观察者、读回每一块的结论。
    /// 预期：取池级 checker 不取卡、不报错；观察者判段一张盘都不判；读回时判器结论与观察者结论每一块都在。
    /// 判别力（`crates/mutations.tsv`）：取池级 checker 那一刻就取卡（有没有盘要判都取，没经服务给卡就报错），这条红。
    #[cfg(feature = "gpu")]
    #[test]
    fn the_gpu_route_takes_no_card_after_the_service_judged_when_no_observer_hands_back_an_image() {
        // 一次性库留着：录入段与替身服务写的行要从空库起
        let disposable =
            crate::crash_library::DisposableLibrary::new("gpu-route-no-card-without-images")
                .unwrap_or_else(|refusal| panic!("{refusal}"));
        let mut store = disposable
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .expect("开得了一次性库");
        let configuration = disposable.directory().join("multi-host-with-the-gpu.env");
        std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了配置");
        // 先开库、后拿改环境的锁；守卫清掉了服务交的卡（GPU_CARDS_BY_PRIORITY）与领卡通道，只设多机配置
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[(
            crate::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            configuration.as_os_str(),
        )]);
        let base = formatted_pool_of_two_devices_for_the_probe();
        let writes = probe_writes_on_the_locations_of(
            &base,
            ProbeUnitWriteContents::TheBaseBytesWrittenAgain,
        );
        let segments: Vec<Vec<usize>> = (0..writes.len()).map(|index| vec![index]).collect();
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("造的流最后一段是一次根槽写");
        let route = CrashPointReplayJudgeRoute::Gpu;
        let observers = [the_unit_writes_case_observer()];
        let versions = [versions_of_the_unit_writes_case_observer(route)];
        flow_of_the_inputs(
            the_unit_writes_case_item(),
            &base,
            &writes,
            &segments,
            judged_root_index,
            CrashPointReplayVersionTable::with_a_file_only(&[]),
            &every_proper_subset,
        )
        .with_the_flow(|flow| {
            let plan = plan_crash_points(flow);
            let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
            record_the_observers_of_the_flow(
                &library,
                flow,
                &plan,
                &observers,
                &versions,
                &|_| true,
                &|| false,
            )
            .expect("录入段写得进库");
            let judgement_versions = vec![
                ConclusionVersion(sha256_digest(
                    b"judge version of the gpu route taking no card\n"
                ));
                plan.points.len()
            ];
            let judged_conclusions =
                crate::crash_amplification::ConclusionsStoredUnder::TheConclusionKind {
                    kind: ConclusionKind::ImageJudgement,
                    version_of_each_point: &judgement_versions,
                };
            let tally = crate::crash_amplification::judge_my_blocks_through_the_pending_rows(
                &library,
                &plan,
                &[0x2B; DIGEST_BYTES],
                &[0x3C; DIGEST_BYTES],
                &|_| true,
                &judged_conclusions,
                &crate::crash_amplification::WaitingForTheGpuVerificationServer {
                    progress_interval: std::time::Duration::from_secs(3600),
                    between_two_reckonings:
                        &stand_in_service_stores_a_clean_conclusion_of_every_pending_row,
                },
                &|| false,
            )
            .expect("替身服务判完、读得回");
            assert_eq!(
                usize::try_from(tally.blocks_checked).expect("块数装得进 usize"),
                blocks_of_the_plan(&plan).len(),
                "前提：判器那一段每一块都写了待判行、等到替身服务判完"
            );
            // 显卡来路照判段入口那样在守卫里记（配置开着 GPU、没有服务交的卡）
            let pool_checker = pool_checker_of_the_images_on_the_route(
                route,
                CardSourcesOfTheEngineStart::read_from_the_environment_now(configuration.clone()),
            )
            .expect(
                "观察者交回的盘上的池级 checker 不在这一刻取卡：配置开了 GPU、环境里没有服务交的卡也取得到",
            );
            let (observing, reuse_sentinel_problems, _observers_recorded_again) =
                crate::crash_amplification::judge_the_observers_then_the_reuse_sentinels(
                    &library,
                    flow,
                    &plan,
                    Some((observers.as_slice(), versions.as_slice(), &pool_checker)),
                    &|_| true,
                    None,
                    &|| false,
                )
                .expect("观察者一张盘都不交回：不取卡，判得完");
            let observing = observing.expect("交了观察者那一格，观察者判段的结果就在");
            assert!(
                observing.blocks_observed_here > 0,
                "前提：观察者挂着的块这一趟拼了结论"
            );
            assert_eq!(
                (
                    observing.images_judged_here,
                    observing.images_judged_on_the_gpu_here,
                    observing.image_judgements_reused
                ),
                (0, 0, 0),
                "观察者一张盘都不交回：池级 checker 一张盘都不判"
            );
            assert!(
                reuse_sentinel_problems.is_empty(),
                "没抽哨兵：{reuse_sentinel_problems:?}"
            );
            let without_every_conclusion = blocks_of_the_flow_without_every_conclusion(
                &library.take(),
                &plan,
                &judged_conclusions,
                &observers_in_the_block_account(&plan, &observers, &[versions[0].conclusion]),
            )
            .expect("读得了库");
            assert_eq!(
                (
                    without_every_conclusion.missing_an_observer_conclusion,
                    without_every_conclusion.missing_the_judgement_or_an_observer_conclusion
                ),
                (0, 0),
                "读回结论：判器结论与观察者结论每一块都在"
            );
        });
    }

    /// 录入段配了 GPU 判表：建表、打包在锁外，判表进了库、清单里有它的摘要；给定状态表那一路的判表写回读同样只有往返在锁里。这一条不上卡，只建表、写库、读回。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁收窄 录入段打包判表挪回锁里」「判定库锁收窄 判表序列化挪回锁里」「判定库锁收窄 判表解码挪回锁里」这几行让这一条红。
    #[cfg(feature = "gpu")]
    #[test]
    fn the_judge_tables_are_built_written_and_read_back_with_only_the_round_trips_in_the_lock() {
        // 一次性库留着：写判表、读回，要一份空的临时服务
        let disposable =
            crate::crash_library::DisposableLibrary::new("judge-tables-outside-the-lock")
                .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration)
            .expect("开得了设成这个进程那一份的一次性库");
        let base = formatted_pool_of_two_devices_for_the_probe();
        let writes = probe_writes_on_the_locations_of(
            &base,
            ProbeUnitWriteContents::TheBaseBytesWrittenAgain,
        );
        let segments: Vec<Vec<usize>> = (0..writes.len()).map(|index| vec![index]).collect();
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("造的流最后一段是一次根槽写");
        let flow_to_record = flow_of_the_inputs(
            the_unit_writes_case_item(),
            &base,
            &writes,
            &segments,
            judged_root_index,
            CrashPointReplayVersionTable::with_a_file_only(&[]),
            &every_proper_subset,
        );
        let manifest = flow_to_record
            .record_into(&library, true, &flow_to_record.recorded_bytes())
            .expect("录得进库");
        manifest
            .digest_named(MANIFEST_JUDGE_TABLES)
            .expect("配了 GPU 的那一趟清单里有判表");
        let packed = flow_to_record.with_the_flow(|flow| {
            crate::crash_amplification::packed_judge_tables_of_the_plan(
                flow,
                &plan_crash_points(flow),
            )
            .expect("这条流建得出判表")
        });
        crate::crash_library::record_the_judge_tables_and_read_them_back_outside_the_lock(
            library.shared_handle(),
            &packed,
        )
        .expect("写得进、读得回");
    }
}
