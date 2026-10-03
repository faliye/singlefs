//! RefFS 的调用方翻译层（`singlefs_harness::reference_filesystem_translation`）在真历史上的判定：r6 攻方第一节那三类
//! （`research/prompts/m3-formal-r6-opus-output.md`「一、X1」：回退之后又崩溃、回退在持久之前就返回的坏实现、恢复之后再回退到挂载写的根），
//! 合法的判绿、坏实现的判红；崩溃时刻按切开当前段的那道屏障由哪一步发出归属；故障注入真落了的每一档 RefFS 都弃权（带故障的状态在 U1 射程外），
//! 注入点没走到的照判，停下那一步 panic 的按在途判（代码轮第三轮判决 `research/prompts/m3-formal-code-r3-main-verification.md` 第三节第 1 条）。
//! 这里不枚举崩溃状态：历史照常跑完，读回由用例按「合法实现会读出什么」「坏实现会读出什么」交入（坏实现在这一层造不出，
//! 它读出的东西造得出——攻方副本上量过的那 9 态读回回退之前的内容）。

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::fault_injection::{FaultOutcome, FlippedBit, InjectedFault};
use singlefs_harness::history::{
    execute_history_with, AppliedEffect, CapturedPanic, ContentChoice, ContentLength,
    FailureObservation, FailureSignature, GeneratedHistory, HistoryDeviceWidth, HistoryEnding,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
    RollbackTargetChoice, StepOutcome, StepPosition,
};
use singlefs_harness::memory_pool::RecordCheck;
use singlefs_harness::model::{ModelRootKey, ObservedReadBack};
use singlefs_harness::model_comparison::model_root_key;
use singlefs_harness::reference_filesystem_translation::{
    crash_moment_of_a_segment_closed_at, judge_a_second_crash_after_the_restart,
    segment_closing_stream_indexes, ReferenceFileSystemAbstention, ReferenceFileSystemCrashMoment,
    ReferenceFileSystemTranslation, ReferenceFileSystemVote, TranslatedHistory,
    WhatTheReopenMustRecover,
};
use singlefs_harness::scenario::first_file_content;
use singlefs_harness::{RecordedOperation, RecordedOperationKind, RetainedOperation, SharedStream};
use singlefs_reference_filesystem::VersionNumber;

/// 一次覆盖写的内容：一个数据单元装得下的 201 字节（攻方副本 `small` 的同一个选择子）。
fn overwrite_content(fill_seed: u64) -> ContentChoice {
    ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 200 },
        fill_seed,
    }
}

/// 根环里次新的那条根：覆盖写之后是起点那次第一个文件的根。
const RING_ROOT_BEFORE_THE_NEWEST: RollbackTargetChoice = RollbackTargetChoice::RingRoot {
    index_from_newest: 1,
};

fn history_after_the_first_file(operations: Vec<HistoryOperation>) -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    }
}

const EXECUTION_WITHOUT_THE_PER_STEP_CHECKER: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 照常跑完一段历史（不跑每步的 checker），边跑边攒 RefFS 的输入，再翻成 RefFS 的事件与 m 表。
fn translated_run_of(operations: Vec<HistoryOperation>) -> (TranslatedHistory, Vec<StepOutcome>) {
    let history = history_after_the_first_file(operations);
    let execution = EXECUTION_WITHOUT_THE_PER_STEP_CHECKER;
    let stream = SharedStream::retaining_contents();
    let mut translation =
        ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
    let run = execute_history_with(&history, execution, &stream, &mut |observation| {
        translation.note_an_observed_step(observation, stream.operation_count());
    });
    assert_eq!(run.ending, HistoryEnding::Completed, "这段历史照常跑完");
    let translated = translation.translate(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    (translated, run.outcomes)
}

/// 第 `step_index` 步是一次做成的回退：交回它报的目标根。
fn rollback_target_of(outcomes: &[StepOutcome], step_index: usize) -> ModelRootKey {
    match &outcomes[step_index] {
        StepOutcome::Applied(AppliedEffect::RolledBack { target, .. }) => {
            model_root_key(target.instance, target.checkpoint_txg)
        }
        other @ (StepOutcome::Applied(_)
        | StepOutcome::Refused { .. }
        | StepOutcome::NotApplicable(_)) => {
            panic!("第 {step_index} 步要是一次做成的回退：{other:?}")
        }
    }
}

fn file_read_back(root: ModelRootKey, content: Vec<u8>) -> ObservedReadBack {
    ObservedReadBack::FileRead { root, content }
}

/// 起点（第一个文件）→ 覆盖写 → 回退到根环里次新的那条（第一个文件那一版）。
fn overwrite_then_roll_back_to_the_first_file() -> (TranslatedHistory, ModelRootKey) {
    let (translated, outcomes) = translated_run_of(vec![
        HistoryOperation::PublishOverwrite(overwrite_content(7)),
        HistoryOperation::RollBackWhileMounted(RING_ROOT_BEFORE_THE_NEWEST),
    ]);
    let target = rollback_target_of(&outcomes, 1);
    assert_eq!(
        translated.version_of_the_root(target),
        Some(VersionNumber(2)),
        "第 1 版是空池，第 2 版是起点那次第一个文件"
    );
    (translated, target)
}

#[test]
fn a_crash_after_a_returned_rollback_that_reads_the_rolled_back_content_is_green() {
    let (translated, target) = overwrite_then_roll_back_to_the_first_file();
    let judgement = translated.judge_a_crash(
        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
        &file_read_back(target, first_file_content()),
    );
    assert_eq!(
        judgement.vote,
        ReferenceFileSystemVote::Green,
        "回退返回之后崩溃、读回回退目标：合法"
    );
}

#[test]
fn a_rollback_that_returned_before_its_root_was_durable_reads_back_the_content_before_it_and_is_red(
) {
    let (translated, target) = overwrite_then_roll_back_to_the_first_file();
    let judgement = translated.judge_a_crash(
        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
        &file_read_back(target, overwrite_content(7).bytes()),
    );
    assert!(
        matches!(judgement.vote, ReferenceFileSystemVote::Red { .. }),
        "切开那一段的屏障在回退返回之后才发、读回回退之前的覆盖写：回退抬了 lo，判红：{:?}",
        judgement.vote
    );
}

#[test]
fn a_crash_inside_a_rollback_may_read_back_the_content_before_it() {
    let (translated, target) = overwrite_then_roll_back_to_the_first_file();
    let judgement = translated.judge_a_crash(
        ReferenceFileSystemCrashMoment::InsideStep(StepPosition::Operation(1)),
        &file_read_back(target, overwrite_content(7).bytes()),
    );
    assert_eq!(
        judgement.vote,
        ReferenceFileSystemVote::Green,
        "回退还在途：回退之前的覆盖写合法"
    );
}

#[test]
fn a_rollback_to_a_root_the_mount_wrote_after_a_cold_start_recovery_is_named_by_the_version_that_recovery_restarted_from(
) {
    let (translated, outcomes) = translated_run_of(vec![
        HistoryOperation::PublishOverwrite(overwrite_content(7)),
        HistoryOperation::ColdStartRecover,
        HistoryOperation::CloseAndMountWritable,
        HistoryOperation::PublishOverwrite(overwrite_content(9)),
        HistoryOperation::RollBackWhileMounted(RING_ROOT_BEFORE_THE_NEWEST),
    ]);
    let target = rollback_target_of(&outcomes, 4);
    assert_eq!(
        translated.version_of_the_root(target),
        Some(VersionNumber(4)),
        "空池、第一个文件、第一次覆盖写、冷启动读出重新起头的那一份：挂载写的根映到第 4 版"
    );
    let judgement = translated.judge_a_crash(
        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
        &file_read_back(target, overwrite_content(7).bytes()),
    );
    assert_eq!(
        judgement.vote,
        ReferenceFileSystemVote::Green,
        "回退到挂载写的根，之后读回第一次覆盖写的内容"
    );
}

#[test]
fn a_second_crash_inside_the_mount_that_reads_back_an_older_version_than_the_first_recovery_is_red()
{
    let (translated, outcomes) = translated_run_of(vec![HistoryOperation::PublishOverwrite(
        overwrite_content(7),
    )]);
    assert!(matches!(outcomes[0], StepOutcome::Applied(_)), "覆盖写做成");
    let root = model_root_key(InstanceGeneration(1), CheckpointTxg(4));
    let first_crash = translated.judge_a_crash(
        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
        &file_read_back(root, overwrite_content(7).bytes()),
    );
    assert_eq!(
        first_crash.vote,
        ReferenceFileSystemVote::Green,
        "第一次崩溃读回覆盖写"
    );
    let vote = judge_a_second_crash_after_the_restart(
        &first_crash,
        &file_read_back(root, first_file_content()),
    );
    assert!(
        matches!(vote, ReferenceFileSystemVote::Red { .. }),
        "挂载途中再崩一次，读回比第一次恢复读出的更旧：{vote:?}"
    );
}

fn write_on_device_zero() -> RetainedOperation {
    RetainedOperation {
        operation: RecordedOperation {
            device: DeviceIdentity(0),
            kind: RecordedOperationKind::Write,
            offset: DeviceOffsetInBytes(1 << 20),
            length: 512,
            content_hash: 0,
        },
        contents: Some(vec![0x5a; 512]),
    }
}

fn barrier_on_device_zero() -> RetainedOperation {
    RetainedOperation {
        operation: RecordedOperation {
            device: DeviceIdentity(0),
            kind: RecordedOperationKind::Barrier,
            offset: DeviceOffsetInBytes(0),
            length: 0,
            content_hash: 0,
        },
        contents: None,
    }
}

#[test]
fn the_crash_moment_is_the_step_that_issued_the_barrier_closing_the_segment_not_the_step_that_wrote_into_it(
) {
    // 第 0 步写了一次就返回（没发屏障），第 1 步发的屏障关上那一段：段里第一个被扣下的写在第 0 步里，切段的屏障是第 1 步发的。
    let operations = vec![write_on_device_zero(), barrier_on_device_zero()];
    let stream_length_after_each_step = [
        (StepPosition::Operation(0), 1),
        (StepPosition::Operation(1), 2),
    ];
    let closing = segment_closing_stream_indexes(&operations);
    assert_eq!(
        closing,
        vec![Some(1)],
        "一段，由录制流第 1 步（那道屏障）关上"
    );
    assert_eq!(
        crash_moment_of_a_segment_closed_at(closing[0], &stream_length_after_each_step),
        ReferenceFileSystemCrashMoment::InsideStep(StepPosition::Operation(1)),
        "崩溃时刻归给发那道屏障的第 1 步：第 0 步那时已经返回"
    );
}

/// 覆盖写返回之后注入 `fault`（`fault_outcome` 是注入落没落）、重开读回 `content`：RefFS 照判时读回覆盖写之前那一版（第一个文件）
/// 是红（最后一次发布返回过，lo 抬到它），读回覆盖写是绿；弃权时它不投票。
fn reopen_vote_after_the_last_publish_returned(
    fault: InjectedFault,
    fault_outcome: FaultOutcome,
    content: Vec<u8>,
) -> ReferenceFileSystemVote {
    let (translated, _outcomes) = translated_run_of(vec![HistoryOperation::PublishOverwrite(
        overwrite_content(7),
    )]);
    let root = model_root_key(InstanceGeneration(1), CheckpointTxg(3));
    translated.judge_a_reopen_after_an_injected_fault(
        fault,
        fault_outcome,
        WhatTheReopenMustRecover::AVersionTheHistoryLeft,
        &file_read_back(root, content),
    )
}

/// 注入真落了（那一步返回了错误）之后重开读回覆盖写之前那一版。
fn reopen_vote_after_reading_back_the_version_before_the_last_publish(
    fault: InjectedFault,
) -> ReferenceFileSystemVote {
    reopen_vote_after_the_last_publish_returned(
        fault,
        FaultOutcome::SurfacedAsAnError,
        first_file_content(),
    )
}

fn assert_no_reference_filesystem_vote_because_the_state_carries_the_fault(
    vote: &ReferenceFileSystemVote,
    fault: InjectedFault,
) {
    assert_eq!(
        vote,
        &ReferenceFileSystemVote::Abstained(
            ReferenceFileSystemAbstention::TheStateCarriesAnInjectedFault
        ),
        "{} 之后的状态带故障、在 U1 射程外：RefFS 那一票弃权，O1 判官照旧",
        fault.name()
    );
}

#[test]
fn the_reopen_after_a_swallowed_write_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish(
) {
    let fault = InjectedFault::WriteIsSwallowed;
    let vote = reopen_vote_after_reading_back_the_version_before_the_last_publish(fault);
    assert_no_reference_filesystem_vote_because_the_state_carries_the_fault(&vote, fault);
}

#[test]
fn the_reopen_after_a_failing_write_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish(
) {
    let fault = InjectedFault::WriteFails;
    let vote = reopen_vote_after_reading_back_the_version_before_the_last_publish(fault);
    assert_no_reference_filesystem_vote_because_the_state_carries_the_fault(&vote, fault);
}

#[test]
fn the_reopen_after_a_failing_read_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish(
) {
    let fault = InjectedFault::ReadFails;
    let vote = reopen_vote_after_reading_back_the_version_before_the_last_publish(fault);
    assert_no_reference_filesystem_vote_because_the_state_carries_the_fault(&vote, fault);
}

#[test]
fn the_reopen_after_a_read_returning_a_flipped_bit_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish(
) {
    let fault = InjectedFault::ReadReturnsCorruptedBytes {
        flipped_bit: FlippedBit {
            byte_index: 0,
            bit_index: 0,
        },
    };
    let vote = reopen_vote_after_reading_back_the_version_before_the_last_publish(fault);
    assert_no_reference_filesystem_vote_because_the_state_carries_the_fault(&vote, fault);
}

#[test]
fn the_reopen_after_a_failing_barrier_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish(
) {
    let fault = InjectedFault::BarrierFails;
    let vote = reopen_vote_after_reading_back_the_version_before_the_last_publish(fault);
    assert_no_reference_filesystem_vote_because_the_state_carries_the_fault(&vote, fault);
}

#[test]
fn the_reopen_after_a_swallowed_barrier_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish(
) {
    let fault = InjectedFault::BarrierIsSwallowed;
    let vote = reopen_vote_after_reading_back_the_version_before_the_last_publish(fault);
    assert_no_reference_filesystem_vote_because_the_state_carries_the_fault(&vote, fault);
}

#[test]
fn the_reopen_after_a_fault_that_never_fired_reading_back_the_last_returned_publish_is_green() {
    let vote = reopen_vote_after_the_last_publish_returned(
        InjectedFault::WriteIsSwallowed,
        FaultOutcome::NeverFired,
        overwrite_content(7).bytes(),
    );
    assert_eq!(
        vote,
        ReferenceFileSystemVote::Green,
        "注入点没走到：状态不带故障、RefFS 照判，读回最后一次返回过的覆盖写是合法的"
    );
}

#[test]
fn the_reopen_after_a_fault_that_never_fired_reading_back_the_version_before_the_last_returned_publish_is_red(
) {
    let vote = reopen_vote_after_the_last_publish_returned(
        InjectedFault::WriteIsSwallowed,
        FaultOutcome::NeverFired,
        first_file_content(),
    );
    assert!(
        matches!(vote, ReferenceFileSystemVote::Red { .. }),
        "注入点没走到：状态不带故障、RefFS 照判，覆盖写返回过、lo 抬到它，读回它之前那一版判红：{vote:?}"
    );
}

/// 起点（第一个文件）→ 覆盖写，覆盖写写完全部写之后 panic、没返回（用例造的结局：照常跑完这段历史，覆盖写那一步观察者不记、
/// 交回值改成停在它上面的一次 panic）；注入点没走到。重开读回 `content` 时 RefFS 那一票。
fn reopen_vote_after_the_last_publish_panicked(content: Vec<u8>) -> ReferenceFileSystemVote {
    let history = history_after_the_first_file(vec![HistoryOperation::PublishOverwrite(
        overwrite_content(7),
    )]);
    let execution = EXECUTION_WITHOUT_THE_PER_STEP_CHECKER;
    let panicked_step = StepPosition::Operation(0);
    let stream = SharedStream::retaining_contents();
    let mut translation =
        ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
    let mut run = execute_history_with(&history, execution, &stream, &mut |observation| {
        if observation.position != panicked_step {
            translation.note_an_observed_step(observation, stream.operation_count());
        }
    });
    assert_eq!(run.ending, HistoryEnding::Completed, "这段历史照常跑完");
    run.outcomes.clear();
    run.ending = HistoryEnding::NewFinding {
        signature: FailureSignature::Panic {
            location: "用例造的 panic".to_string(),
        },
        observation: FailureObservation {
            position: panicked_step,
            operation_kind: Some(history.operations[0].kind()),
            violations: Vec::new(),
            panic: Some(CapturedPanic {
                location: "用例造的 panic".to_string(),
                message: "覆盖写写完之后、返回之前 panic".to_string(),
            }),
            newest_ring_root_txg: None,
            root_ring_slot_count: None,
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        },
    };
    translation.note_the_step_the_history_stopped_in(
        &history.operations,
        &run,
        stream.operation_count(),
    );
    let translated = translation.translate(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    translated.judge_a_reopen_after_an_injected_fault(
        InjectedFault::WriteFails,
        FaultOutcome::NeverFired,
        WhatTheReopenMustRecover::AVersionTheHistoryLeft,
        &file_read_back(
            model_root_key(InstanceGeneration(1), CheckpointTxg(3)),
            content,
        ),
    )
}

#[test]
fn a_publish_that_panicked_before_returning_is_in_flight_so_the_reopen_may_read_back_the_version_before_it(
) {
    let vote = reopen_vote_after_the_last_publish_panicked(first_file_content());
    assert_eq!(
        vote,
        ReferenceFileSystemVote::Green,
        "panic 的覆盖写没返回、在途：不抬 lo，读回它之前那一版（第一个文件）合法"
    );
}

#[test]
fn a_publish_that_panicked_before_returning_is_in_flight_so_the_reopen_may_read_back_its_content() {
    let vote = reopen_vote_after_the_last_publish_panicked(overwrite_content(7).bytes());
    assert_eq!(
        vote,
        ReferenceFileSystemVote::Green,
        "panic 的覆盖写在途：它的末尾在允许集合里，读回它的内容合法"
    );
}
