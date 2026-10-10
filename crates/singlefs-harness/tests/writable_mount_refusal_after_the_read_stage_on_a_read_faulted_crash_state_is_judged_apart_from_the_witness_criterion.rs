//! C664（挂载拒绝合法复算把两种拒因报成一个，`.claude/kb/checks-owed.md` 那一行）的还清条件里那一句判别力自证：带读故障的档里造一个
//! 取号那一刻瞬时读错的态，合成一个拒因时判不合法、分开后判合法。
//! 造法：第一个文件之后的健康盘面（单元区 384 槽的两块内存盘，mkfs → 取号 → 暖机 → 第一个文件），崩溃镜像就是这份盘面本身——两遍读之间的
//! 瞬时读错过后，盘上什么都看不出来。拒是用例自己拼的（判法自己在崩溃镜像上复算判据，不看拒里填的读数）：
//! - 分开报的那一拒：`MountError::SystemConfigurationUnreadableAfterTheReadStage`，取号那一刻读见证值判不出（R2）；
//! - 合成一个拒因时实现报的那一拒（C664 之前 `mount.rs` 的 `refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage`
//!   拼的）：判据 N-配置 为真、重读仍为真，前一遍是读阶段往下走用的那一遍读数、后一遍同一个见证值判 R2 判不出。
//!
//! 被拒的那次挂载自己在读故障下跑时（判官入参 `ReadFaultsUnderTheRefusedMount`，判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md`
//! 第二节 J2 C3 / J2-a 那一行、改法 G3），两遍读之间的瞬时读错在崩溃镜像上复算不出，「读阶段判完之后系统配置又读不出」那一拒的判据复算为假
//! 不算拒得不合法；判据 N-配置 那一拒照旧复算（读阶段本来就判过它为假）。被拒的那次挂载不带故障层时（崩溃注入第二、三截的调法：
//! 盘是崩溃镜像本身，历史里注入过读故障也不会在这次挂载里出瞬时读错），同一拒复算为假就是拒错了，崩溃态注没注入过读故障都一样。

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    MountError, NewerPublishWitness, RollbackTarget, SelectedVersionAgainstTheWitness,
    StillUnreadableAfterOneReread, SystemConfigurationUnreadableAfterTheReadStage,
    SystemConfigurationUnreadableWhen, WitnessedCounterComparison,
};
use singlefs_harness::history::{
    execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryExecution, HistorySeed,
    HistoryStartingPoint, PerStepChecker,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount,
    read_recovery_fields, ComparedVersion, DiskAfterTheRefusal, ReadFaultsInTheCrashState,
    ReadFaultsUnderTheRefusedMount, ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet,
    RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
};
use singlefs_harness::SharedStream;

const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

fn pool_after_the_first_file() -> MemoryPool {
    let stream = SharedStream::retaining_contents();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: Vec::new(),
    };
    execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: SMALL,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &stream,
        &mut |_observation| {},
    );
    let mut pool = MemoryPool::with_devices(&DEVICES, SMALL.device_bytes());
    pool.apply(&stream.retained_operations());
    pool
}

/// 读阶段往下走用的那一遍读数：所选那一版、判据 N-配置 为假（c_见证 不比所选那一版末条的计数器大）。
fn read_stage_settled_on(
    selected_version: (InstanceGeneration, CheckpointTxg),
) -> SelectedVersionAgainstTheWitness {
    SelectedVersionAgainstTheWitness {
        selected_version: RollbackTarget {
            instance: selected_version.0,
            checkpoint_txg: selected_version.1,
        },
        witness: NewerPublishWitness {
            witnessed_journal_counter: 1,
            comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                selected_version_last_record_counter: 1,
            },
        },
    }
}

/// 拒的理由、崩溃态注没注入读故障、被拒的那次挂载自己在不在读故障下跑，判一次（所选那一版取只读恢复落到的那一版，盘上逐字节不变、
/// 只读读出在允许集合里）。
fn verdict_on(
    crash_image: &MemoryPool,
    refusal_on: fn(SelectedVersionAgainstTheWitness) -> MountError,
    read_faults: ReadFaultsInTheCrashState,
    read_faults_under_the_refused_mount: ReadFaultsUnderTheRefusedMount,
) -> WritableMountRefusalVerdict {
    let fields = read_recovery_fields(crash_image, ComparedVersion::TheRecoveredVersion);
    let selected_version = fields
        .report
        .effective_root
        .expect("健康盘面上只读恢复落到第一个文件那一版");
    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
        crash_image,
        fields.report.effective_root,
        &refusal_on(read_stage_settled_on(selected_version)),
        DiskAfterTheRefusal::ByteForByteUnchanged,
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
        read_faults,
        read_faults_under_the_refused_mount,
    )
}

/// 分开报：取号那一刻读见证值判不出（盘 1 报读错）。
fn refused_after_the_read_stage(read_stage: SelectedVersionAgainstTheWitness) -> MountError {
    MountError::SystemConfigurationUnreadableAfterTheReadStage(Box::new(
        SystemConfigurationUnreadableAfterTheReadStage {
            read_stage,
            unreadable: SystemConfigurationUnreadableWhen::AcquisitionReadsAnUndecidableWitness {
                first_device_reporting_a_read_error: DEVICES[1],
            },
        },
    ))
}

/// 合成一个拒因：照判据 N-配置 为真、重读仍为真报，后一遍同一个见证值判 R2 判不出。
fn refused_as_the_witness_criterion(read_stage: SelectedVersionAgainstTheWitness) -> MountError {
    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
            first_read: read_stage,
            reread: SelectedVersionAgainstTheWitness {
                selected_version: read_stage.selected_version,
                witness: NewerPublishWitness {
                    witnessed_journal_counter: read_stage.witness.witnessed_journal_counter,
                    comparison:
                        WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots,
                },
            },
        },
    ))
}

/// 场景：第一个文件之后的健康盘面当崩溃镜像，崩溃态注入过读故障，可写挂载在取号那一刻因瞬时读错拒（盘上看不出来）。
/// 预期：被拒的那次挂载在读故障下跑时，分开报的那一拒合法（判据复算为假不算不合法，盘上不变、只读读出在允许集合里）；合成一个拒因时实现报的
/// 判据 N-配置 那一拒复算为假、判不合法；被拒的那次挂载不带故障层时（崩溃注入第二、三截的调法），同一拒复算为假、判不合法——
/// 崩溃态注入过读故障不放过它（攻方 zeros 那组比重上的形状：注入态 + 新拒因复算为假）。
/// 判别力：入参不看、一律按「挂载不带故障层」（`crates/mutations.tsv` 一行）时第一条红；一律按「挂载在读故障下跑」（一行）时第三条红；
/// 那一成员映射回判据 N-配置 那一条理由（一行）时第一条红。
#[test]
fn a_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_legal_where_the_merged_witness_refusal_is_not(
) {
    let crash_image = pool_after_the_first_file();
    assert_eq!(
        verdict_on(
            &crash_image,
            refused_after_the_read_stage,
            ReadFaultsInTheCrashState::Injected,
            ReadFaultsUnderTheRefusedMount::TheRefusedMountReadThroughInjectedFaults
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
        ),
        "被拒的那次挂载在读故障下跑：读阶段之后系统配置又读不出那一拒合法（瞬时读错在崩溃镜像上复算不出）"
    );
    assert_eq!(
        verdict_on(
            &crash_image,
            refused_as_the_witness_criterion,
            ReadFaultsInTheCrashState::Injected,
            ReadFaultsUnderTheRefusedMount::TheRefusedMountReadThroughInjectedFaults
        ),
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
            conditions_not_met: vec![
                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            ],
        },
        "合成一个拒因（报成判据 N-配置）：判据复算为假，判不合法"
    );
    assert_eq!(
        verdict_on(
            &crash_image,
            refused_after_the_read_stage,
            ReadFaultsInTheCrashState::Injected,
            ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults
        ),
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
            conditions_not_met: vec![
                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            ],
        },
        "崩溃态注入过读故障、被拒的那次挂载却不带故障层：盘上两槽都读得出、都自证过，那一拒的判据复算为假，拒错了"
    );
}
