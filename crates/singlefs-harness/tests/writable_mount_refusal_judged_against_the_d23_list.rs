//! 可写挂载被拒时判拒绝本身（`singlefs_harness::mount_transparency::judge_writable_mount_refusal`，r5 判决第二节 U6）：拒绝合法 ⇔
//! 拒的理由在 D23（journal 的角色与格式） 已定项 14 那张单子上、那一条判据在崩溃镜像上复算为真、盘上逐字节不变、只读读出在允许集合里。
//! 判据由判法在镜像上自己复算，不经 O1 判官、不经实现判拒那一段。小池是单元区 384 槽的两块内存盘，第一个文件之后的盘面。
//! 要拒的盘面取系统配置见证读的 R2（D22（单元原子性怎么合成） 已定项 8 射程）判不出那一形：两块盘第一个文件那次轮换写的那一槽都报读错，
//! 有槽设备报读错、没有一块盘两槽都自证过；崩溃镜像是同一份盘面包上同一组读错（判法读到的与可写挂载读到的一样）。
//! 撕裂的轮换（读得出、自证不过、设备没报错）R2 下照常可写挂载，不是要拒的盘面。
mod common;

use common::{
    with_unreadable_ranges, without_unreadable_ranges, DeviceWithUnreadableRanges,
    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    mount_writable_with_space_admission, MountError, NewerPublishWitness, RollbackTarget,
    SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread, WitnessedCounterComparison,
};
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::history::{
    execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryExecution, HistorySeed,
    HistoryStartingPoint, PerStepChecker,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal, read_recovery_fields, ComparedVersion, DiskAfterTheRefusal,
    ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet, RefusalReasonOnTheD23List,
    WritableMountRefusalVerdict,
};
use singlefs_harness::SharedStream;

const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
/// 第一个文件那次系统配置轮换写的世代号（mkfs → 取号 → 暖机 → 第一个文件，同 `build_pool` 那一段）：每块盘世代 5 那一槽（槽号 5 mod 2 = 1）。
const GENERATION_OF_THE_FIRST_FILE_ROTATION: u64 = 5;

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
    let mut pool = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        SMALL.device_bytes(),
    );
    pool.apply(&stream.retained_operations());
    pool
}

/// 两块盘第一个文件那次轮换写的那一槽（世代 5）每次读都报读错：R2 判不出那一形（有槽设备报读错、没有一块盘两槽都自证过）。
fn read_errors_on_the_newest_system_configuration_slot_of_every_device() -> SharedUnreadableRanges {
    let offset_of_the_newest_slot = (GENERATION_OF_THE_FIRST_FILE_ROTATION
        % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
        * u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
    SharedUnreadableRanges::new(
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|device| UnreadableRange {
                device,
                offset_in_bytes: offset_of_the_newest_slot,
                length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
                failing_reads: FailingReadsOfARange::Every,
            })
            .collect(),
        UnreadableRangeReadBack::DeviceError,
    )
}

/// `pool` 的盘面拷进内存稀疏盘、逐块包上 `read_errors`：既是可写挂载挂的盘，也是判法复算判据读的崩溃镜像。
fn devices_with_read_errors(
    pool: &MemoryPool,
    read_errors: &SharedUnreadableRanges,
) -> Vec<(
    DeviceIdentity,
    DeviceWithUnreadableRanges<SparseBlockDevice>,
)> {
    let physical_block_size =
        PhysicalBlockSizeInBytes(SMALL.parameters().geometry.physical_block_size);
    let devices = pool
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(pool.device_size_in_bytes, physical_block_size);
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect();
    with_unreadable_ranges(devices, read_errors)
}

/// 在 `pool` 盘面的副本上包上 `read_errors` 起可写挂载：拒了交回错误与拒之后的盘面（读错拆掉），做成了 panic。
fn refused_mount_on(
    pool: &MemoryPool,
    read_errors: &SharedUnreadableRanges,
) -> (MountError, MemoryPool) {
    let parameters = SMALL.parameters();
    let mut devices = devices_with_read_errors(pool, read_errors);
    let error = match mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    ) {
        Err(error) => error,
        Ok(mounted) => panic!(
            "前提：这份盘面上可写挂载要拒，却挂上了实例 {:?}",
            mounted.output.instance
        ),
    };
    let after = MemoryPool {
        devices: without_unreadable_ranges(devices)
            .into_iter()
            .map(|(identity, device)| (identity, device.image))
            .collect(),
        device_size_in_bytes: pool.device_size_in_bytes,
    };
    (error, after)
}

/// 拒因是 R2 的判不出：读阶段判据 N-配置 为真、重读仍为真，两遍读数的比较都是「有槽设备报读错、没有一块盘两槽都自证过」。
fn refused_as_undecidable_under_r2_on_both_reads(error: &MountError) -> bool {
    let MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) = error else {
        return false;
    };
    let StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
        first_read,
        reread,
    } = still_unreadable.as_ref()
    else {
        return false;
    };
    [first_read, reread].iter().all(|reading| {
        reading.witness.comparison
            == WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots
    })
}

fn disk_after(crash_image: &MemoryPool, after: &MemoryPool) -> DiskAfterTheRefusal {
    if crash_image == after {
        DiskAfterTheRefusal::ByteForByteUnchanged
    } else {
        DiskAfterTheRefusal::Changed
    }
}

/// 一个合法的拒（读阶段判据 N-配置 按真）在一份健康盘面上报出来：判据复算为假。
fn witness_refusal_claimed_on(selected_version: (InstanceGeneration, CheckpointTxg)) -> MountError {
    let reading = SelectedVersionAgainstTheWitness {
        selected_version: RollbackTarget {
            instance: selected_version.0,
            checkpoint_txg: selected_version.1,
        },
        witness: NewerPublishWitness {
            witnessed_journal_counter: 1,
            comparison: WitnessedCounterComparison::Undecidable,
        },
    };
    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
            first_read: reading,
            reread: reading,
        },
    ))
}

/// 两块盘世代 5 那一槽都报读错：R2 判不出，可写挂载在取号之前拒、盘上逐字节不变，判法在包着同一组读错的崩溃镜像上复算判据为真 ⇒ 合法。
#[test]
fn a_refusal_when_the_newest_system_configuration_slot_of_every_device_reports_a_read_error_is_legal(
) {
    let healthy = pool_after_the_first_file();
    let content_of_the_healthy_pool =
        read_recovery_fields(&healthy, ComparedVersion::TheRecoveredVersion);
    let (error, after) = refused_mount_on(
        &healthy,
        &read_errors_on_the_newest_system_configuration_slot_of_every_device(),
    );
    assert!(
        refused_as_undecidable_under_r2_on_both_reads(&error),
        "前提：两块盘世代 5 那一槽都报读错，R2 判不出，读阶段判据 N-配置 按真、重读仍真，拒：{error:?}"
    );
    let crash_image = devices_with_read_errors(
        &healthy,
        &read_errors_on_the_newest_system_configuration_slot_of_every_device(),
    );
    let fields_of_the_crash_image =
        read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
    let read_only_read_back = if fields_of_the_crash_image
        .compared_version
        .as_ref()
        .map(|version| &version.content)
        == content_of_the_healthy_pool
            .compared_version
            .as_ref()
            .map(|version| &version.content)
    {
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet
    } else {
        ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet
    };
    assert_eq!(
        judge_writable_mount_refusal(
            &crash_image,
            fields_of_the_crash_image.report.effective_root,
            &error,
            disk_after(&healthy, &after),
            read_only_read_back,
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
        )
    );
}

#[test]
fn a_witness_refusal_on_a_healthy_pool_is_a_refusal_for_the_wrong_reason() {
    let healthy = pool_after_the_first_file();
    let fields = read_recovery_fields(&healthy, ComparedVersion::TheRecoveredVersion);
    let selected_version = fields
        .report
        .effective_root
        .expect("健康盘面上恢复落到一版");
    assert_eq!(
        judge_writable_mount_refusal(
            &healthy,
            fields.report.effective_root,
            &witness_refusal_claimed_on(selected_version),
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
        ),
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
            conditions_not_met: vec![
                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            ],
        }
    );
}

#[test]
fn an_unreadable_instance_table_refusal_on_a_healthy_pool_is_a_refusal_for_the_wrong_reason() {
    let healthy = pool_after_the_first_file();
    let fields = read_recovery_fields(&healthy, ComparedVersion::TheRecoveredVersion);
    let refusal = MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
        StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger {
            newest_root_on_the_reread: None,
        },
    ));
    assert_eq!(
        judge_writable_mount_refusal(
            &healthy,
            fields.report.effective_root,
            &refusal,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
        ),
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
            conditions_not_met: vec![
                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            ],
        }
    );
}

/// 同一份 R2 判不出的盘面（两块盘世代 5 那一槽都报读错）上的拒，交进判法时报成拒之后盘变了 ⇒ 判非法，只差「盘被改了」那一条。
#[test]
fn a_refusal_on_read_errors_of_every_newest_system_configuration_slot_that_changed_the_disk_is_illegal(
) {
    let healthy = pool_after_the_first_file();
    let (error, _after) = refused_mount_on(
        &healthy,
        &read_errors_on_the_newest_system_configuration_slot_of_every_device(),
    );
    assert!(
        refused_as_undecidable_under_r2_on_both_reads(&error),
        "前提：两块盘世代 5 那一槽都报读错，R2 判不出，拒：{error:?}"
    );
    let crash_image = devices_with_read_errors(
        &healthy,
        &read_errors_on_the_newest_system_configuration_slot_of_every_device(),
    );
    let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
    assert_eq!(
        judge_writable_mount_refusal(
            &crash_image,
            fields.report.effective_root,
            &error,
            DiskAfterTheRefusal::Changed,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
        ),
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
            conditions_not_met: vec![RefusalConditionNotMet::DiskChangedByTheRefusedMount],
        }
    );
}

#[test]
fn a_refusal_whose_reason_is_not_on_the_d23_list_is_left_to_the_campaign() {
    let healthy = pool_after_the_first_file();
    let fields = read_recovery_fields(&healthy, ComparedVersion::TheRecoveredVersion);
    assert_eq!(
        judge_writable_mount_refusal(
            &healthy,
            fields.report.effective_root,
            &MountError::InstanceTableMalformed,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
        ),
        WritableMountRefusalVerdict::ReasonNotOnTheD23List
    );
}
