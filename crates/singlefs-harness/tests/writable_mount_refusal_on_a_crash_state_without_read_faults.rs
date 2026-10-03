//! C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）：崩溃注入第二截判拒可写合不合法时，崩溃态没注入读故障、拒的理由是
//! D23（journal 的角色与格式） 已定项 14 的判据 N-配置，而判据在崩溃镜像上复算（撕裂的系统配置槽跳过）仍为真 ⇒ 拒得不合法
//! （`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state`）。注入过读故障的崩溃态不判这一条
//! （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）；撕裂系统配置那一支照 U6 判合法（C664、C331）。
//! 小池是单元区 384 槽的两块内存盘，第一个文件之后的盘面；崩溃镜像是内存盘面本身，读不报错。
//! 见证判据为真的那份盘面取法照 E171（三oracle合谋测试） 第 242 行（系统配置轮换先于根槽 FUA 落盘）：一次覆盖写那次发布只落了系统配置轮换，
//! 它的单元、记录与根都没落。

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    mount_writable_with_space_admission, MountError, StillUnreadableAfterOneReread,
    WitnessedCounterComparison,
};
use singlefs_core::recovery::PoolReader;
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields, ComparedVersion,
    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage,
    RefusalConditionNotMet, RefusalReasonOnTheD23List, WitnessedCounterBranchThatHolds,
    WritableMountRefusalVerdict,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RetainedOperation, SharedStream};

const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
/// 撕裂一个系统配置槽时改的那个字节在槽里的位置：槽中间，落在整槽校验和罩着的范围里。
const TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT: usize = 2048;

/// 从第一个文件起、再跑 `operations` 的录制流（内容保留）。
fn recorded_stream_after_the_first_file_then(
    operations: Vec<HistoryOperation>,
) -> Vec<RetainedOperation> {
    let stream = SharedStream::retaining_contents();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: SMALL,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &stream,
        &mut |_observation| {},
    );
    assert_eq!(
        run.outcomes.len(),
        history.operations.len(),
        "前提：历史每一步都跑完：{:?}",
        run.ending
    );
    stream.retained_operations()
}

fn pool_of(operations: &[RetainedOperation]) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        SMALL.device_bytes(),
    );
    pool.apply(operations);
    pool
}

/// 第一个文件之后的盘面上，只落了一次覆盖写那次发布的系统配置轮换（每块盘一槽，journal tail 写的是这次发布末条记录的计数器），
/// 它的单元、记录与根都没落：系统配置见证过一次比盘上任何一版都新的发布。崩溃镜像读不报错。
fn crash_image_where_only_the_rotation_of_an_overwrite_landed() -> (MemoryPool, MemoryPool) {
    let first_file = recorded_stream_after_the_first_file_then(Vec::new());
    let overwrite =
        recorded_stream_after_the_first_file_then(vec![HistoryOperation::PublishOverwrite(
            ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 0 },
                fill_seed: 1,
            },
        )]);
    assert_eq!(
        overwrite[..first_file.len()],
        first_file[..],
        "前提：同一个起点跑出同一段录制流"
    );
    let parameters = SMALL.parameters();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let (writes_of_the_overwrite, _segments, _stream_indexes) =
        writes_and_segments_with_stream_indexes(&overwrite[first_file.len()..], &geometry);
    let rotation: Vec<RetainedWrite> = writes_of_the_overwrite
        .iter()
        .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
        .cloned()
        .collect();
    assert_eq!(
        rotation.len(),
        2,
        "前提：覆盖写那次发布每块盘轮换一个系统配置槽"
    );
    assert!(
        writes_of_the_overwrite
            .iter()
            .any(|write| write.kind == StepKind::RootRecordFua)
            && writes_of_the_overwrite
                .iter()
                .any(|write| write.kind == StepKind::JournalRecord),
        "前提：覆盖写那次发布写了记录与根（这份盘面上都不落）"
    );
    let healthy = pool_of(&first_file);
    let mut crash_image = healthy.clone();
    crash_image.apply_writes(&rotation);
    (healthy, crash_image)
}

/// 第一个文件之后的盘面上，盘 0 两个系统配置槽各改一个字节（读得出、自证不过，设备不报错）：撕裂的系统配置，盘 0 一份自证过的都没有。
fn crash_image_where_both_system_configuration_slots_of_device_zero_are_torn(
) -> (MemoryPool, MemoryPool) {
    let healthy = pool_of(&recorded_stream_after_the_first_file_then(Vec::new()));
    let slot_spacing = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut crash_image = healthy.clone();
    for slot in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        let offset = DeviceOffsetInBytes(slot * slot_spacing);
        let mut bytes = healthy
            .read(DeviceIdentity(0), offset, slot_bytes)
            .expect("内存盘上系统配置槽读得出");
        bytes[TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT] ^= 0xFF;
        crash_image
            .devices
            .get_mut(&DeviceIdentity(0))
            .expect("有盘 0")
            .write(offset, &bytes);
    }
    (healthy, crash_image)
}

/// 在 `crash_image` 的副本上起可写挂载（读不报错）：拒了交回错误与拒之后的盘面，做成了 panic。
fn refused_writable_mount_on(crash_image: &MemoryPool) -> (MountError, MemoryPool) {
    let parameters = SMALL.parameters();
    let physical_block_size = PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size);
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = crash_image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device =
                SparseBlockDevice::new(crash_image.device_size_in_bytes, physical_block_size);
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect();
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
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.image))
            .collect(),
        device_size_in_bytes: crash_image.device_size_in_bytes,
    };
    (error, after)
}

/// 拒因是判据 N-配置 为真、重读仍为真时，重读那一遍拿 c_见证 跟什么比的（别的拒因交 `None`）。
fn witnessed_counter_comparison_on_the_reread(
    error: &MountError,
) -> Option<WitnessedCounterComparison> {
    let MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) = error else {
        return None;
    };
    let StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
        reread,
        ..
    } = still_unreadable.as_ref()
    else {
        return None;
    };
    Some(reread.witness.comparison)
}

fn disk_after(crash_image: &MemoryPool, after: &MemoryPool) -> DiskAfterTheRefusal {
    if crash_image == after {
        DiskAfterTheRefusal::ByteForByteUnchanged
    } else {
        DiskAfterTheRefusal::Changed
    }
}

/// 崩溃镜像上只读恢复读出的与第一个文件那一版逐字节相同：在允许集合里（那次覆盖写没确认过，第一个文件那一版是允许的）。
fn read_only_read_back_against_the_first_file(
    healthy: &MemoryPool,
    crash_image: &MemoryPool,
) -> ReadOnlyReadBackOfTheCrashImage {
    let content_of = |pool: &MemoryPool| {
        read_recovery_fields(pool, ComparedVersion::TheRecoveredVersion)
            .compared_version
            .map(|version| version.content)
    };
    if content_of(crash_image) == content_of(healthy) {
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet
    } else {
        ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet
    }
}

/// 只落了覆盖写那次轮换的盘面，没注入读故障：所选那一版是第一个文件那一版，c_见证 大于它末条记录的计数器，正确实现在这份盘面上永远拒可写——
/// 拒的理由在单子上、判据复算为真、盘没变、读出在允许集合里，U6 四条都满足，C670 那一条判不合法。
#[test]
fn a_witness_refusal_on_a_crash_image_without_read_faults_where_only_a_rotation_landed_is_illegal()
{
    let (healthy, crash_image) = crash_image_where_only_the_rotation_of_an_overwrite_landed();
    let (error, after) = refused_writable_mount_on(&crash_image);
    assert!(
        matches!(
            witnessed_counter_comparison_on_the_reread(&error),
            Some(WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord { .. })
        ),
        "前提：判据 N-配置 拿 c_见证 与所选那一版的末条比为真、重读仍真，拒：{error:?}"
    );
    let read_only_read_back = read_only_read_back_against_the_first_file(&healthy, &crash_image);
    assert_eq!(
        read_only_read_back,
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
        "前提：只读恢复读出第一个文件那一版"
    );
    let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state(
            &crash_image,
            fields.report.effective_root,
            &error,
            disk_after(&crash_image, &after),
            read_only_read_back,
            ReadFaultsInTheCrashState::NoneInjected,
        ),
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
            conditions_not_met: vec![
                RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(
                    WitnessedCounterBranchThatHolds::WitnessedCounterAboveTheLastRecordOfTheSelectedVersion
                ),
            ],
        }
    );
}

/// 同一份只落了轮换的盘面，调用方说这个崩溃态注入过读故障：C670 那一条不判，只判 U6 四条，合法。
#[test]
fn the_same_witness_refusal_in_a_crash_state_with_read_faults_injected_is_legal() {
    let (healthy, crash_image) = crash_image_where_only_the_rotation_of_an_overwrite_landed();
    let (error, after) = refused_writable_mount_on(&crash_image);
    let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state(
            &crash_image,
            fields.report.effective_root,
            &error,
            disk_after(&crash_image, &after),
            read_only_read_back_against_the_first_file(&healthy, &crash_image),
            ReadFaultsInTheCrashState::Injected,
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
        )
    );
}

/// 撕裂系统配置那一支：盘 0 两槽都撕裂、没注入读故障，实现在取号之前以「这块盘一份自证过的系统配置都没有」拒（报成判据 N-配置 的拒因，C664）；
/// 跳过撕裂的槽复算判据为假（c_见证 等于第一个文件那一版末条的计数器），C670 那一条不报，U6 照甲把撕裂判成按真：合法。
#[test]
fn a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system_configuration_slots_torn_is_legal(
) {
    let (healthy, crash_image) =
        crash_image_where_both_system_configuration_slots_of_device_zero_are_torn();
    let (error, after) = refused_writable_mount_on(&crash_image);
    assert_eq!(
        witnessed_counter_comparison_on_the_reread(&error),
        Some(WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot),
        "前提：盘 0 一份自证过的系统配置都没有，照判据 N-配置 的拒因拒：{error:?}"
    );
    let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state(
            &crash_image,
            fields.report.effective_root,
            &error,
            disk_after(&crash_image, &after),
            read_only_read_back_against_the_first_file(&healthy, &crash_image),
            ReadFaultsInTheCrashState::NoneInjected,
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
        )
    );
}
