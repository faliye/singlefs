//! 注入一次、跑到池级 checker 之前：重跑历史判 panic、判那一步的结局、重开判恢复到的那一版，
//! 再与给定状态引擎判的池级 checker 结论合并成这一次注入的判定。

use super::*;

/// 「重开走读失败」是不是这一次注入的正确答案：盘上本来就没有一版立得住。两种：
///
/// - 注入的是说谎的设备（[`InjectedFault::the_device_lies`]）：那一份内容无声地丢了，
///   单元头里的校验和是旧那一份的，读出来发现对不上、报错正是要的结果。
/// - 注入落在起点段而 mkfs 自己就报了错：盘上没有一个做完了的文件系统，模型一版都没提交过，
///   重开走到任何一版才是缺口。
///
/// 别的一律照判：报错的设备不改盘上已有的字节，重开该走到之前某一条根上，走读失败就是缺口。
fn failed_reopen_is_the_right_answer(
    fault: InjectedFault,
    ending_observation: Option<&FailureObservation>,
) -> bool {
    fault.the_device_lies() || make_filesystem_left_no_finished_filesystem(ending_observation)
}

/// 历史停在起点而 mkfs 自己报了错、盘上没有一个做完了的文件系统（[`failed_reopen_is_the_right_answer`] 的后一种）。
fn make_filesystem_left_no_finished_filesystem(
    ending_observation: Option<&FailureObservation>,
) -> bool {
    matches!(
        ending_observation.and_then(|observation| observation.harness_judgement.as_ref()),
        Some(HarnessJudgement::TheStartingPointReturnedAnError { step, .. })
            if !step.finished_filesystem_is_on_the_devices()
    )
}

/// 一个注入点跑到池级 checker 之前的样子（[`run_one_fault_up_to_the_pool_checker`]）：harness 自己判完的那几样（理想模型、RefFS、执行器）
/// 连同计数，池级 checker 那一遍的结论还没并进来（[`settle_one_fault_with_the_pool_checker_verdicts`] 并）。
pub(super) struct OneFaultBeforeThePoolChecker {
    /// 这一个注入点自己的计数，并进来时整份加进这一段的计数（截短重跑时整段那一遍的计数不加）。
    pub(super) tally: FaultInjectionTally,
    /// C378 那一格（取号之后挂载报错、新号没回卷），没落进去是 None。
    acquired_instance_left: Option<AcquiredInstanceLeft>,
    /// 历史停在别的东西上（不是注入那一步返回的错）时停下那一步的那一条失败。
    failure_where_the_history_stopped: Option<FailureObservation>,
    /// 重开读回的那一版理想模型不允许时对不上的那一格。
    model_disagreement_at_the_reopen: Option<crate::model::ModelDisagreement>,
    /// 重开上 RefFS 判红的那一项（借违例那一栏带着走）。
    reference_filesystem_violation_at_the_reopen: Option<(&'static str, String)>,
    /// 交给给定状态引擎判的盘与归类要的事实。
    pub(super) images: ImagesAfterAFault,
}

/// 照给定状态引擎判回的结论把一个注入点并进来（[`inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine`] 的那三步）：
/// 不判池级 checker 跑一遍、交出去判；每步镜像里有判红的，照那几张的结论截短重跑一遍、再交一次判；并进来的是历史与判红就停那一档跑的那一遍。
pub(super) fn merge_one_fault_with_the_pool_checker_verdicts(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    marks: &[StepMark],
    drawn: &DrawnFault,
    pool_checker_verdicts_of: &mut dyn FnMut(&ImagesAfterAFault) -> PoolCheckerVerdictsAfterAFault,
    accumulator: &mut FaultInjectionAccumulator<'_>,
) {
    let run_through = run_one_fault_up_to_the_pool_checker(history, execution, marks, drawn, None);
    let verdicts = pool_checker_verdicts_of(&run_through.images);
    assert_eq!(
        verdicts.violations_of_the_step_images.len(),
        run_through
            .images
            .step_images_of_the_run_with_the_fault
            .len(),
        "给定状态引擎交回的每步镜像结论与交出去的每步镜像逐张对应（种子 {} {}）",
        history.seed.0,
        drawn.render()
    );
    let Some(first_red_step_image) = verdicts
        .violations_of_the_step_images
        .iter()
        .position(|violations| !violations.is_empty())
    else {
        settle_one_fault_with_the_pool_checker_verdicts(
            &run_through,
            verdicts.violations_after_the_fault,
            verdicts.invariants_still_violated_if_the_swallowed_writes_had_landed,
            accumulator,
        );
        return;
    };
    let stopped = run_one_fault_up_to_the_pool_checker(
        history,
        execution,
        marks,
        drawn,
        Some(&verdicts.violations_of_the_step_images[..=first_red_step_image]),
    );
    assert_eq!(
        stopped
            .images
            .step_images_of_the_run_with_the_fault
            .as_slice(),
        &run_through.images.step_images_of_the_run_with_the_fault[..=first_red_step_image],
        "截短重跑那一遍停在判红的那一张、之前（含）逐张与整段那一遍相同（种子 {} {}）",
        history.seed.0,
        drawn.render()
    );
    let verdicts_of_the_stopped_run = pool_checker_verdicts_of(&stopped.images);
    assert_eq!(
        verdicts_of_the_stopped_run
            .violations_of_the_step_images
            .len(),
        stopped.images.step_images_of_the_run_with_the_fault.len(),
        "给定状态引擎交回的每步镜像结论与截短重跑那一遍交出去的每步镜像逐张对应（种子 {} {}）",
        history.seed.0,
        drawn.render()
    );
    settle_one_fault_with_the_pool_checker_verdicts(
        &stopped,
        verdicts_of_the_stopped_run.violations_after_the_fault,
        verdicts_of_the_stopped_run.invariants_still_violated_if_the_swallowed_writes_had_landed,
        accumulator,
    );
}

/// 一个注入点跑到池级 checker 之前判完的那几样，连同池级 checker 在重开那张盘与被吞的写插回去那张盘上的结论，归好类记进这一段：
/// 停下那一步的失败（历史停在别的东西上时）、重开那张盘上的失败（池级 checker、RefFS 判红或理想模型对不上时），各一条。
pub(super) fn settle_one_fault_with_the_pool_checker_verdicts(
    one_fault: &OneFaultBeforeThePoolChecker,
    pool_checker_violations_after_the_fault: Vec<(&'static str, String)>,
    invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet<&'static str>,
    accumulator: &mut FaultInjectionAccumulator<'_>,
) {
    accumulator.tally.absorb(&one_fault.tally);
    if let Some(left) = one_fault.acquired_instance_left {
        accumulator.acquired_instances_left.push(left);
    }
    let images = &one_fault.images;
    // 三、判：注入那一步返回错误的那一格不算失败；别的失败（panic、checker 判红、重开走到模型或 RefFS 不允许的版本）逐条分类。
    // RefFS 判红的那一项借违例那一栏带着走（注入真落了的每一档 RefFS 都弃权，只有注入点没走到的那几次投票；没走到时说谎的设备
    // 什么也没丢，不会落进 `left_by_the_lying_device` 的豁免）。
    let mut violations_after_the_fault = pool_checker_violations_after_the_fault;
    violations_after_the_fault.extend(
        one_fault
            .reference_filesystem_violation_at_the_reopen
            .clone(),
    );
    let mut failures: Vec<FailureObservation> = Vec::new();
    failures.extend(one_fault.failure_where_the_history_stopped.clone());
    if !violations_after_the_fault.is_empty()
        || one_fault.model_disagreement_at_the_reopen.is_some()
    {
        let template = &images.observation_without_the_pool_checker_violations;
        failures.push(FailureObservation {
            position: template.position,
            operation_kind: template.operation_kind,
            violations: violations_after_the_fault,
            panic: None,
            newest_ring_root_txg: template.newest_ring_root_txg,
            root_ring_slot_count: template.root_ring_slot_count,
            harness_judgement: None,
            model_disagreement: one_fault.model_disagreement_at_the_reopen.clone(),
            raised_floor_lands_only_on_abandoned_roots: template
                .raised_floor_lands_only_on_abandoned_roots,
            record_check: RecordCheck::default(),
        });
    }
    let what_the_swallowed_writes_explain = images
        .what_the_swallowed_writes_explain_without_the_pool_checker
        .clone()
        .map(|explained| WhatTheSwallowedWritesExplain {
            invariants_still_violated_if_the_swallowed_writes_had_landed,
            ceiling_without_the_swallowed_roots: explained.ceiling_without_the_swallowed_roots,
            root_ring_slots_of_the_swallowed_root_slot_writes: explained
                .root_ring_slots_of_the_swallowed_root_slot_writes,
            instance_rows_with_the_swallowed_roots_rebuilt_from_their_records: explained
                .instance_rows_with_the_swallowed_roots_rebuilt_from_their_records,
            byte_ranges_of_the_swallowed_writes: explained.byte_ranges_of_the_swallowed_writes,
        });
    for observation in failures {
        if observation.panic.is_some() {
            accumulator.tally.faults_that_panicked += 1;
        }
        accumulator.settle_a_failure_after_a_fault(
            images.seed,
            images.drawn,
            classify_a_failure_after_a_fault(
                observation,
                &images.facts_after_the_fault,
                images.drawn.fault,
                what_the_swallowed_writes_explain.as_ref(),
            ),
            &images.reopen,
        );
    }
}

/// 注入一次：armed 的计划重跑这段历史，判 panic、判那一段的结局、再重开判恢复到的那一版（与理想模型、RefFS 比），
/// 交回判完的这几样连同交给给定状态引擎判的盘（[`OneFaultBeforeThePoolChecker`]）。harness 不判池级 checker：
/// `pool_checker_violations_of_the_step_images` 是 None 时历史不因池级 checker 停；是 `Some` 时那是给定状态引擎在整段那一遍交出去的每步镜像上
/// 判红的（到第一张判红的为止），历史照判红就停那一档的口径停在第一张判红的那一步（合并入口截短重跑的那一遍）。
///
/// # Panics
/// 给了每步镜像的结论而历史没停在最后那一张上（交出去的镜像比给的结论多：同一个种子、同一个注入点重跑，执行器不确定了）。
// gpu-exempt: comparison-with-the-ideal-model
// gpu-exempt: reference-filesystem
pub(super) fn run_one_fault_up_to_the_pool_checker(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    marks: &[StepMark],
    drawn: &DrawnFault,
    pool_checker_violations_of_the_step_images: Option<&[Vec<(&'static str, String)>]>,
) -> OneFaultBeforeThePoolChecker {
    let mut tally = FaultInjectionTally::default();
    let geometry = execution.device_width.fixed_geometry();
    let plan = SharedFaultPlan::armed(
        geometry,
        FaultSchedule::the_nth_call_across_the_pool(drawn.fault, drawn.call_ordinal),
    );
    let stream = SharedStream::retaining_contents();
    let mut committed_by_the_armed_run: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
    // 最后一次观察那一刻的模型与录制流长度：历史停下的那一步不调观察者，它之前那一步的这两样就是「停下那一步开始时」的。
    let mut model_at_the_last_observation: Option<crate::model::IdealModel> = None;
    let mut stream_length_at_the_last_observation = 0usize;
    // RefFS 的输入（重开的判定 = RefFS ∧ O1 判官全四条）：开着注入跑的那一遍逐步攒，停下的那一步跑完之后补。
    let mut reference_filesystem_translation =
        ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
    // 带着注入跑的那一遍每一步的镜像（池级 checker 本来在那一刻判的那几张）：只记它在录制流里的位置与根环，盘本身随重开那一份交出去
    let mut step_images_of_the_run_with_the_fault: Vec<StepImageInTheRecordedStream> = Vec::new();
    let run = execute_history_stopping_at_the_given_pool_checker_violations(
        history,
        execution,
        &stream,
        &plan,
        &mut |observation| {
            for (key, file) in observation.model.committed_versions() {
                committed_by_the_armed_run.insert(key, file);
            }
            model_at_the_last_observation = Some(observation.model.clone());
            stream_length_at_the_last_observation = stream.operation_count();
            reference_filesystem_translation
                .note_an_observed_step(observation, stream_length_at_the_last_observation);
        },
        &mut |step_image| {
            let (newest_ring_root_txg, root_ring_slot_count) =
                newest_ring_root_and_slot_count(step_image.image);
            step_images_of_the_run_with_the_fault.push(StepImageInTheRecordedStream {
                position: step_image.position,
                operation_kind: step_image.operation_kind,
                operations_recorded: step_image.operations_recorded,
                newest_ring_root_txg,
                root_ring_slot_count,
                swallowed_writes_before_it: plan.swallowed_write_count(),
                copies_quarantined_by_the_implementation_before_it: step_image
                    .copies_quarantined_by_the_implementation_up_to_this_step,
            });
        },
        &mut |step_image_index| {
            match pool_checker_violations_of_the_step_images {
            None => Vec::new(),
            Some(violations_of_the_step_images) => violations_of_the_step_images
                .get(step_image_index)
                .cloned()
                .unwrap_or_else(|| {
                    panic!(
                        "截短重跑那一遍要停在给了结论的最后一张上，交出去了第 {step_image_index} 张（给了 {} 张的结论）",
                        violations_of_the_step_images.len()
                    )
                }),
        }
        },
    );
    reference_filesystem_translation.note_the_step_the_history_stopped_in(
        &history.operations,
        &run,
        stream.operation_count(),
    );
    let fired = plan.fired();
    tally.faults += 1;
    *tally.faults_by_kind.entry(drawn.fault.name()).or_insert(0) += 1;
    if let Some(first) = fired.first() {
        let point = InjectionPoint {
            segment: drawn.segment.kind(),
            call: drawn.fault.call_kind(),
            written_structure: first.written_structure,
        };
        *tally
            .faults_by_injection_point
            .entry(point.render())
            .or_insert(0) += 1;
    }

    let ending_observation = match &run.ending {
        HistoryEnding::Completed => None,
        HistoryEnding::KnownRed { observation, .. }
        | HistoryEnding::NewFinding { observation, .. } => Some(observation),
    };
    let surfaced = ending_observation.is_some_and(|observation| {
        the_fault_surfaced_as_an_error(&run, observation, drawn.segment)
    });
    if surfaced {
        if let Some(StepOutcome::Refused { member }) = run.outcomes.last() {
            *tally.refusal_members.entry(member.clone()).or_insert(0) += 1;
        }
        if let Some(HarnessJudgement::TheStartingPointReturnedAnError { step, .. }) =
            ending_observation.and_then(|observation| observation.harness_judgement.as_ref())
        {
            *tally
                .starting_point_failures
                .entry(step.name())
                .or_insert(0) += 1;
        }
    }
    let outcome = if fired.is_empty() {
        FaultOutcome::NeverFired
    } else if surfaced {
        FaultOutcome::SurfacedAsAnError
    } else {
        match &run.ending {
            HistoryEnding::Completed => FaultOutcome::ToleratedAndTheHistoryFinished,
            HistoryEnding::KnownRed { .. } | HistoryEnding::NewFinding { .. } => {
                FaultOutcome::StoppedOnSomethingOtherThanTheInjectedError
            }
        }
    };
    *tally.faults_by_outcome.entry(outcome.name()).or_insert(0) += 1;

    // 二、重开：拿录制流（只记真正落到盘上的那些写）重建镜像，跑恢复，问模型走到的这一版允不允许。
    let operations = stream.retained_operations();
    let mut image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    image.apply(&operations);
    let (writes, _segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations, &geometry);
    let all_persisted = vec![true; writes.len()];
    let newest_persisted = newest_persisted_root(&writes, &all_persisted)
        .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
    let recovery = recover(&image, JournalPolicy::Consult);
    tally.reopens += 1;
    match &recovery.outcome {
        RecoveryOutcome::FileRead { .. } => tally.reopens_reading_a_file += 1,
        RecoveryOutcome::NoFile { .. } => tally.reopens_without_a_file += 1,
        RecoveryOutcome::Failed { .. } => tally.reopens_that_failed += 1,
    }
    let read_back = observed_read_back_after_a_crash(&recovery);
    // 模型认下来的每一版：这段历史（开着注入跑的那一遍）停下时模型根环里的那几条。观察者攒的那一份漏掉失败那一步——
    // 判出失败的那一步不调观察者，而模型常常已经认下那一版了，两份并起来才完整。
    let mut committed_by_the_model = committed_by_the_armed_run.clone();
    for (key, file) in &run.committed_versions_at_the_end {
        committed_by_the_model.insert(
            *key,
            file.as_ref().map(|content| Rc::from(content.as_slice())),
        );
    }
    // 再放行一样：不注入时失败那一步会写出的那几版（测量跑记的那一份）。
    let mut allowed = committed_by_the_model.clone();
    if let Some(index) = marks
        .iter()
        .position(|mark| mark.position == drawn.segment.position())
    {
        for (key, file) in &marks[index].committed_versions {
            allowed.insert(*key, file.clone());
        }
    }
    // 再放行一样（实七）：历史停下的那一步里实现已经写出的根——最后一次观察之后录制流里落盘的根槽写
    // （说谎的设备让历史停在注入之后的另一步时，那一步写出的根不在上面两份里；实四丙交回 ④ offset 284）。
    // 那条根的内容不另信实现：读回的正是它、而且读回的内容是模型认过的某一版，或停下那一步这次发布自己的内容，才放行。
    let roots_written_by_the_stopping_step: BTreeSet<ModelRootKey> = writes
        .iter()
        .zip(&stream_indexes)
        .filter(|(write, stream_index)| {
            write.kind == StepKind::RootRecordFua
                && **stream_index >= stream_length_at_the_last_observation
        })
        .filter_map(|(write, _)| write.bytes().map(root_identity_written_by))
        .map(|(instance, checkpoint_txg)| model_root_key(instance, checkpoint_txg))
        .collect();
    let content_published_by_the_stopping_step: Option<Vec<u8>> = ending_observation
        .and_then(|observation| match observation.position {
            StepPosition::Operation(step_index) => history.operations.get(step_index),
            StepPosition::StartingPoint => None,
        })
        .and_then(|operation| match operation {
            HistoryOperation::PublishFirstFile(content)
            | HistoryOperation::PublishOverwrite(content) => Some(content.bytes()),
            // 按 inode 号写点名的号执行时才取（`history::WriteOfAnInodeChoice::inode_among`），这里分不出写的是不是第一个文件：照写第一个文件放行
            // 这一步交的内容（写的是别的文件时第一个文件的内容不变，模型认过的那几版里本来就有它）。
            HistoryOperation::WriteOfAnInode(choice) => Some(choice.content.bytes()),
            HistoryOperation::PublishWithoutUnits
            | HistoryOperation::CloseAndMountWritable
            | HistoryOperation::RollBackWhileMounted(_)
            | HistoryOperation::RaiseRollbackFloor(_)
            | HistoryOperation::ColdStartRecover
            | HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
            | HistoryOperation::Unlink
            | HistoryOperation::CreateInode
            | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_)
            | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
            | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros
            // 建快照、读快照、删快照不改文件内容（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S2、S4）。
            | HistoryOperation::CreateSnapshot
            | HistoryOperation::ReadSnapshot(_)
            | HistoryOperation::DestroySnapshot(_)
            // 整理只搬单元的落点，不改文件内容（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3）。
            | HistoryOperation::CompactWhileMounted(_)
            // 截断改到的长度执行时按那一刻的 size 取（`history::TruncationChoice`），这里取不到它的内容：照不改内容的那几种只放行模型认过的
            // 那几版与测量跑记的那几版（截断那一步不注入时写出的截后内容在后一份里）。
            | HistoryOperation::Truncate(_)
            // 建 inode 不改第一个文件的内容。
            | HistoryOperation::CreateInodes(_)
            // 按偏移写的号、偏移与长度执行时才取（`history::PartialWriteChoice`），这里取不到写之后的内容：照截断那样只放行模型认过的那几版与
            // 测量跑记的那几版。
            | HistoryOperation::WriteAtOffset(_) => None,
        });
    let (read_back_root, read_back_content): (Option<ModelRootKey>, Option<&[u8]>) =
        match &read_back {
            ObservedReadBack::FileRead { root, content } => (Some(*root), Some(content.as_slice())),
            ObservedReadBack::NoFile { root } => (Some(*root), None),
            ObservedReadBack::Failed { .. } => (None, None),
        };
    if let Some(root) = read_back_root {
        let content_is_one_the_model_knows = allowed
            .values()
            .any(|file| file.as_deref() == read_back_content)
            || (read_back_content.is_some()
                && read_back_content == content_published_by_the_stopping_step.as_deref());
        if roots_written_by_the_stopping_step.contains(&root)
            && !allowed.contains_key(&root)
            && content_is_one_the_model_knows
        {
            allowed.insert(root, read_back_content.map(Rc::from));
        }
    }
    let nothing_was_left_to_recover = matches!(read_back, ObservedReadBack::Failed { .. })
        && failed_reopen_is_the_right_answer(drawn.fault, ending_observation);
    let disagreement = if nothing_was_left_to_recover {
        None
    } else {
        crash_recovery_disagreement(&allowed, newest_persisted, &read_back)
    };
    // RefFS 那一票：重开 = 停下之后崩、全部步都返回过（停下那一步报错的按「写了之后报错」两可，panic 的按在途）。注入真落了的每一档
    // RefFS 都弃权、O1 判官照旧：U1 射程句「带故障的状态（说谎的设备、丢整块盘 C519）不适用」管全部带故障的状态，括号里是举例
    // （调度记录「12-2 交回」那一条）；注入点没走到的不带故障、照判，说谎的设备没说谎，
    // 盘上有没有一版立得住只看 mkfs 做没做完。弃权按理由计进 `reference_filesystem_abstentions_at_the_reopen`。
    let no_version_stands_for_the_reference_filesystem = match outcome {
        FaultOutcome::NeverFired => make_filesystem_left_no_finished_filesystem(ending_observation),
        FaultOutcome::SurfacedAsAnError
        | FaultOutcome::ToleratedAndTheHistoryFinished
        | FaultOutcome::StoppedOnSomethingOtherThanTheInjectedError => {
            failed_reopen_is_the_right_answer(drawn.fault, ending_observation)
        }
    };
    let reference_filesystem_vote = reference_filesystem_translation
        .translate(&operations, &geometry)
        .judge_a_reopen_after_an_injected_fault(
            drawn.fault,
            outcome,
            if no_version_stands_for_the_reference_filesystem {
                WhatTheReopenMustRecover::NothingBecauseNoVersionStands
            } else {
                WhatTheReopenMustRecover::VersionTheHistoryLeft
            },
            &read_back,
        );
    *tally
        .reopens_by_the_reference_filesystem_and_model_verdicts
        .entry(JointVerdictOfTheReferenceFileSystemAndTheModel::of(
            &reference_filesystem_vote,
            disagreement.as_ref(),
        ))
        .or_insert(0) += 1;
    if let ReferenceFileSystemVote::Abstained(reason) = &reference_filesystem_vote {
        *tally
            .reference_filesystem_abstentions_at_the_reopen
            .entry(reason.name())
            .or_insert(0) += 1;
    }
    let reopened = if nothing_was_left_to_recover {
        tally.reopens_that_correctly_found_nothing_to_recover += 1;
        ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo
    } else if crash_recovery_disagreement(&committed_by_the_model, newest_persisted, &read_back)
        .is_none()
    {
        tally.reopened_into_a_version_the_model_committed += 1;
        ReopenedVersion::CommittedByTheModel
    } else if disagreement.is_none() {
        tally.reopened_into_the_version_the_faulted_step_was_writing += 1;
        ReopenedVersion::TheVersionTheFaultedStepWasWriting
    } else {
        tally.reopens_outside_everything_the_model_allows += 1;
        ReopenedVersion::OutsideEverythingTheModelAllows
    };
    // 池级 checker 那一遍归给定状态引擎两段（重开的盘随交回的那一份交出去），harness 一条不变量都不判
    let reopen_text = format!("{reopened:?}（{}）", describe_read_back(&read_back));
    // C378：注入落在一次挂载里、那一步返回了错误，而重开的盘上系统配置的实例代号比这一步之前大——取号之后的写行或暖机报了错，
    // 新号没回卷。认下的已知行为：记进这一格、在报告里点名，不进新发现，也不挡下面的判定。
    let acquired_instance_left = if surfaced {
        acquired_instance_left_after_a_failed_mount(&image, marks, *drawn)
    } else {
        None
    };
    if let Some(left) = &acquired_instance_left {
        *tally
            .acquired_instances_left_after_a_failed_mount
            .entry(left.kind.name())
            .or_insert(0) += 1;
    }

    // 说谎的设备吞掉的写（只有 `WriteIsSwallowed` 有）要是都落了盘，盘面与抬 F 的上限该是什么样：判「这条失败是不是它留下的」要的两样。
    let swallowed_writes = plan.swallowed_writes();
    let stream_after_the_fault = RecordedStreamOfAnImage {
        operations,
        operations_written_by_make_filesystem: run.operations_written_by_make_filesystem,
        earliest_stream_index_after_each_power_loss: run
            .earliest_stream_index_after_each_power_loss
            .clone(),
    };
    let stream_with_the_swallowed_writes_landed = (!swallowed_writes.is_empty()).then(|| {
        stream_with_the_swallowed_writes_landed(&stream_after_the_fault, &swallowed_writes)
    });
    let what_the_swallowed_writes_explain = (!swallowed_writes.is_empty()).then(|| {
        // 池级 checker 那一项（插回去之后仍红的不变量）在并进来那一刻由给定状态引擎的结论填：那张盘随交回的那一份交出去判
        let invariants_still_violated_if_the_swallowed_writes_had_landed = BTreeSet::new();
        let swallowed_root_slot_writes: Vec<&SwallowedWrite> = swallowed_writes
            .iter()
            .filter(|write| {
                geometry.classify(&write.as_retained_operation().operation)
                    == StepKind::RootRecordFua
            })
            .collect();
        let swallowed_roots: Vec<ModelRootKey> = swallowed_root_slot_writes
            .iter()
            .filter_map(|write| match &write.contents {
                SwallowedContents::Bytes(bytes) => Some(root_identity_written_by(bytes)),
                SwallowedContents::Zeroes { .. } | SwallowedContents::Discard { .. } => None,
            })
            .map(|(instance, checkpoint_txg)| model_root_key(instance, checkpoint_txg))
            .collect();
        let ceiling_without_the_swallowed_roots = if swallowed_roots.is_empty() {
            None
        } else {
            model_at_the_last_observation
                .as_ref()
                .map(|model| model.rollback_floor_ceilings_without_the_roots(&swallowed_roots))
        };
        let parameters = execution.device_width.parameters();
        let every_slot_of_the_ring = RootRingSlotTarget {
            named_slots: NamedRootRingSlots::every_slot_in_the_ring(
                parameters.geometry.root_ring_slots_per_region,
            ),
            region_devices: parameters.region_devices,
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        };
        let root_ring_slots_of_the_swallowed_root_slot_writes = swallowed_root_slot_writes
            .iter()
            .filter_map(|write| every_slot_of_the_ring.slot_covering(write.device, write.offset))
            .map(model_ring_position)
            .collect();
        let instance_rows_with_the_swallowed_roots_rebuilt_from_their_records =
            if swallowed_roots.is_empty() {
                None
            } else {
                model_at_the_last_observation
                    .as_ref()
                    .and_then(|model| {
                        model
                            .answer_mount_writable_with_the_roots_rebuilt_from_their_records(
                                &swallowed_roots,
                            )
                            .ok()
                    })
                    .and_then(|answer| answer.expected_mount)
                    .map(|(_, rows)| rows)
            };
        WhatTheSwallowedWritesExplain {
            invariants_still_violated_if_the_swallowed_writes_had_landed,
            ceiling_without_the_swallowed_roots,
            root_ring_slots_of_the_swallowed_root_slot_writes,
            instance_rows_with_the_swallowed_roots_rebuilt_from_their_records,
            byte_ranges_of_the_swallowed_writes: swallowed_writes
                .iter()
                .map(SwallowedWriteByteRange::of)
                .collect(),
        }
    });

    let failure_where_the_history_stopped = if surfaced {
        None
    } else {
        ending_observation.cloned()
    };
    let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count(&image);
    let raised_floor_lands_only_on_abandoned_roots =
        raised_floor_of_the_newest_root_lands_only_on_abandoned_roots(&image);
    let facts_after_the_fault = FactsAfterTheFault {
        starting_point_step_that_returned_an_error: ending_observation.and_then(|observation| {
            match &observation.harness_judgement {
                Some(HarnessJudgement::TheStartingPointReturnedAnError { step, .. }) => Some(*step),
                Some(_) | None => None,
            }
        }),
        system_configuration_slot_writes_that_landed: writes
            .iter()
            .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
            .count(),
        reopened,
    };

    // 四、交回：判完的这几样，连同交给给定状态引擎判的盘（池级 checker 那一遍归给定状态引擎两段）
    let images = ImagesAfterAFault {
        seed: history.seed,
        drawn: *drawn,
        device_width: execution.device_width,
        image_after_the_fault: stream_after_the_fault,
        step_images_of_the_run_with_the_fault,
        image_with_the_swallowed_writes_landed: stream_with_the_swallowed_writes_landed,
        swallowed_writes,
        copies_quarantined_by_the_implementation: run.copies_quarantined_by_the_implementation,
        observation_without_the_pool_checker_violations: FailureObservation {
            position: drawn.segment.position(),
            operation_kind: drawn.segment.operation_kind(),
            violations: Vec::new(),
            panic: None,
            newest_ring_root_txg,
            root_ring_slot_count,
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots,
            record_check: RecordCheck::default(),
        },
        facts_after_the_fault,
        what_the_swallowed_writes_explain_without_the_pool_checker:
            what_the_swallowed_writes_explain,
        reopen: reopen_text,
    };
    OneFaultBeforeThePoolChecker {
        tally,
        acquired_instance_left,
        failure_where_the_history_stopped,
        model_disagreement_at_the_reopen: disagreement,
        reference_filesystem_violation_at_the_reopen: reference_filesystem_vote.violation(),
        images,
    }
}

/// 重开读回了什么，给人看的一行。
fn describe_read_back(read_back: &ObservedReadBack) -> String {
    match read_back {
        ObservedReadBack::NoFile { root } => {
            format!(
                "走到实例 {} 第 {} 代根、没有文件",
                root.instance.0, root.checkpoint_txg.0
            )
        }
        ObservedReadBack::FileRead { root, content } => format!(
            "走到实例 {} 第 {} 代根、读回 {} 字节",
            root.instance.0,
            root.checkpoint_txg.0,
            content.len()
        ),
        ObservedReadBack::Failed { what } => format!("走读失败：{what}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::StartingPointStep;

    /// 「重开走读失败」什么时候是正确答案：说谎的那两种一律是；报错的那四种只有在 mkfs 自己就报错
    /// （盘上还没有文件系统）时才是。这道豁免放宽一格，报错的设备之后的走读失败就没人看了。
    #[test]
    fn failed_reopen_is_excused_only_by_the_lying_device_or_by_the_failed_make_filesystem() {
        let stopped_at = |step| FailureObservation {
            position: StepPosition::StartingPoint,
            operation_kind: None,
            violations: Vec::new(),
            panic: None,
            newest_ring_root_txg: None,
            root_ring_slot_count: None,
            harness_judgement: Some(HarnessJudgement::TheStartingPointReturnedAnError {
                step,
                error: "注入的写错".to_string(),
            }),
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        };
        let failed_make_filesystem = stopped_at(StartingPointStep::MakeFilesystem);
        let failed_first_file = stopped_at(StartingPointStep::PublishFirstFile);
        for lying in [
            InjectedFault::WriteIsSwallowed,
            InjectedFault::BarrierIsSwallowed,
        ] {
            assert!(
                failed_reopen_is_the_right_answer(lying, None),
                "{} 之后走读失败是对的",
                lying.name()
            );
        }
        assert!(
            !failed_reopen_is_the_right_answer(InjectedFault::WriteFails, None),
            "写报错之后走读失败是缺口，不许豁免"
        );
        assert!(
            failed_reopen_is_the_right_answer(
                InjectedFault::WriteFails,
                Some(&failed_make_filesystem)
            ),
            "mkfs 自己就报错时盘上没有文件系统，走读失败是对的"
        );
        assert!(
            !failed_reopen_is_the_right_answer(InjectedFault::WriteFails, Some(&failed_first_file)),
            "mkfs 做完了，第一个文件那一步报错之后重开该走到创世根上"
        );
    }
}
