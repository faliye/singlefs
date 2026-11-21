//! 经库走一条流：录入段之后照键领块、补录观察者的结论、交判段，收回块账。

use super::*;

/// 两段过库，键由调用方给（单测造拒算用）：在长期库里录、判（录入键命中就不再录）；观察者那几组判法单元哪几组拒算，
/// 那几组照判不入库（开库之后登记它们的占位版本，键里带着它们的行只留在这个进程里），别的照键入库同一份库，块全部计入 `refused=`
/// （4-2a 报告第四节第 7 条后半句）。`gpu_judging_inputs` 是入口照起步那一刻的环境读的、判段走 GPU 那一路要的显卡来路，原样交给判段。
/// 交回判段的结局与这一趟的块账（只录那一趟是 `None`）。
///
/// # Errors
/// 开不了库、写不进、读不回；判段的错。
pub(super) fn enumerate_the_flow_through_the_library_with_the_keys(
    item: CrashPointReplayItem<'_>,
    flow_to_record: &FlowToRecord<'_>,
    observers_on_the_nodes: &[ObserverOnTheNodes<'_>],
    route: CrashPointReplayJudgeRoute,
    keys: KeysOfTheItem,
    gpu_judging_inputs: GpuJudgingInputsOfTheEntry,
) -> Result<(StageOutcome<CrashPointReplayTally>, Option<CrashCaseBlocks>), String> {
    let item_name = item_name_of(item);
    let recorded_bytes = flow_to_record.recorded_bytes();
    let gpu_judges = route == CrashPointReplayJudgeRoute::Gpu;
    let inputs = FlowToRecord::recording_inputs_digest(&recorded_bytes, gpu_judges);
    for sentence in &keys.refusal_sentences {
        println!(
            "CRASH_CASE_BLOCKS_KEY_REFUSED item={} {sentence}（这一组照判、不入库，别的照键入库同一份库）",
            item.name
        );
    }
    let library = CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration)?;
    library
        .lock()
        .keep_the_rows_of_these_versions_out_of_the_library(
            keys.versions_kept_out_of_the_library.iter().copied(),
        );
    let stage = RunStage::of_this_process();
    let recording_key_of_this_run =
        recording_key_of_the_scenario_and_the_inputs(item_name.as_str(), &inputs);
    let observer_versions = &keys.observer_versions;
    let blocks_of_the_run: std::cell::Cell<Option<CrashCaseBlocks>> = std::cell::Cell::new(None);
    let outcome = two_stages_under_the_recording_key_topping_up_a_reused_recording(
        &library,
        stage,
        &item_name,
        &recording_key_of_this_run,
        |library| {
            let manifest = flow_to_record.record_into(library, gpu_judges, &recorded_bytes)?;
            if !observers_on_the_nodes.is_empty() {
                // 观察者的录入段（「观察者改走两段」）：这台机器领的块逐状态看、结果写进库，判段从库里读
                flow_to_record.with_the_flow(|flow| {
                    let plan = plan_crash_points(flow);
                    let is_mine = blocks_of_this_host(&plan)?;
                    record_these_observers_and_print_the_line(
                        library,
                        flow,
                        &plan,
                        &|block| is_mine.contains(&block),
                        item,
                        observers_on_the_nodes,
                        observer_versions,
                    )
                })?;
            }
            Ok(manifest)
        },
        |library| {
            record_the_observers_missing_from_a_reused_recording(
                library,
                flow_to_record,
                item,
                observers_on_the_nodes,
                observer_versions,
            )
        },
        |library, manifest| {
            // 配了 GPU 而这份二进制没编进 gpu 特性：判段不判、报本次未跑，交只录（录入段上面照录了），带特性那一趟从库里读回在卡上判
            if route == CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature {
                println!("{}", crash_point_replay_judge_not_run_line(item.name));
                return Ok(StageOutcome::RecordedOnly);
            }
            judge_the_recorded_item(
                library,
                manifest,
                item,
                observers_on_the_nodes,
                observer_versions,
                route,
                &JudgingUnitsOfTheRun {
                    are_refused: keys.are_refused(),
                    blocks_of_the_run: &blocks_of_the_run,
                },
                gpu_judging_inputs,
            )
        },
    )?;
    drop(library);
    Ok((outcome, blocks_of_the_run.get()))
}

/// 观察者的录入段（「观察者改走两段」）：给的这几个观察者在这台机器领的块上逐状态看、结果写进库（库里这一把观察者版本
/// 录过的块不再录，[`record_the_observers_of_the_flow`]），打 `CRASH_POINT_REPLAY_OBSERVERS_RECORDED` 一行。
///
/// # Errors
/// 库读写失败。
fn record_these_observers_and_print_the_line(
    library: &CrashLibrary,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    is_mine: &dyn Fn(u64) -> bool,
    item: CrashPointReplayItem<'_>,
    observers: &[ObserverOnTheNodes<'_>],
    versions: &[ObserverVersions],
) -> Result<(), String> {
    let started = Instant::now();
    let stop_request = StopRequestRememberedInThisRun::new(&stop_is_requested_by_the_environment);
    let recording = record_the_observers_of_the_flow(
        &library.taken_at_each_read_or_write(),
        flow,
        plan,
        observers,
        versions,
        is_mine,
        &|| stop_request.is_requested(),
    )?;
    println!(
        "CRASH_POINT_REPLAY_OBSERVERS_RECORDED item={} observers={} blocks_recorded_here={} states_recorded_here={} blocks_already_recorded={} states_counted_here={} images_observed_fresh={} states_reusing_an_observed_image={} buckets={} witness_misses={} stopped_early={} seconds={:.1}",
        item.name,
        observers
            .iter()
            .map(|observer| observer.observer_name.as_str())
            .collect::<Vec<&str>>()
            .join(","),
        recording.blocks_recorded_here,
        recording.states_recorded_here,
        recording.blocks_already_recorded,
        recording.states_counted_here,
        recording.images_observed_here,
        recording.states_reusing_an_observed_image,
        recording.buckets,
        recording.witness_misses,
        recording.stopped_early,
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

/// 录制复用时补录观察者：流的录入键只由流定（[`FlowToRecord::recording_inputs_digest`]），挂着的观察者的版本不进键，命中之后录入那一段
/// 整段不跑；观察者换了版本（代码改了、另读的输入改了、换了一个观察者）而流没变，判段就读不到它录入段的块。这里在录入段里逐个观察者核
/// 它录入段的块都在库里（[`observers_whose_recorded_blocks_are_missing`]），缺块的那几个照录（只录缺的观察者，流与判表已在库里、键只由内容定，
/// 不整段重录），打 `CRASH_POINT_REPLAY_OBSERVERS_CHECKED_ON_A_REUSED_RECORDING` 一行点名缺块的观察者。
///
/// # Errors
/// 读写库失败；分片与权重写法错。
fn record_the_observers_missing_from_a_reused_recording(
    library: &CrashLibrary,
    flow_to_record: &FlowToRecord<'_>,
    item: CrashPointReplayItem<'_>,
    observers: &[ObserverOnTheNodes<'_>],
    versions: &[ObserverVersions],
) -> Result<(), String> {
    if observers.is_empty() {
        return Ok(());
    }
    flow_to_record.with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        let blocks_this_host_takes = blocks_of_this_host(&plan)?;
        let is_mine = |block: u64| blocks_this_host_takes.contains(&block);
        let missing = observers_whose_recorded_blocks_are_missing(
            &library.lock(),
            &plan,
            observers,
            versions,
            &is_mine,
        )?;
        println!(
            "CRASH_POINT_REPLAY_OBSERVERS_CHECKED_ON_A_REUSED_RECORDING item={} observers={} observers_missing_blocks={}",
            item.name,
            observers.len(),
            missing
                .iter()
                .map(|position| observers[*position].observer_name.as_str())
                .collect::<Vec<&str>>()
                .join(",")
        );
        // 迭代上界：缺块的观察者个数（不多于观察者个数）
        for position in missing {
            record_these_observers_and_print_the_line(
                library,
                flow,
                &plan,
                &is_mine,
                item,
                std::slice::from_ref(&observers[position]),
                std::slice::from_ref(&versions[position]),
            )?;
        }
        Ok(())
    })
}
