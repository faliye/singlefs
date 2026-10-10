//! C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）：崩溃注入第二截判拒可写合不合法时，崩溃态没注入读故障、拒的理由是
//! D23（journal 的角色与格式） 已定项 14 的判据 N-配置，而判据在崩溃镜像上复算（撕裂的系统配置槽跳过）仍为真 ⇒ 拒得不合法
//! （`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state`）。注入过读故障的崩溃态不判这一条
//! （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。
//! 改法 G4（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2「新拒因少一条 C670 同形的判法（J2-b）」那一行）与它同形：
//! 某块盘一份自证过的系统配置都没有时，C664（挂载拒绝合法复算把两种拒因报成一个）之后实现报的是「读阶段判完之后系统配置又读不出」那一拒，
//! 不是判据 N-配置；崩溃态没注入读故障而那一拒的判据在崩溃镜像上复算为真（系统配置见证照 R2 判得出、撕裂不算判不出，判据按「某块盘一份自证过的都没有」为真）
//! ⇒ 没有读故障的历史留不下这份盘面，拒得不合法。注入过读故障的崩溃态不判这一条，那一拒照 U6 判，判据为真即合法。
//! 小池是单元区 384 槽的两块内存盘，第一个文件之后的盘面；崩溃镜像是内存盘面本身，读不报错。
//! 见证判据为真的那份盘面取法照 E171（三oracle合谋测试） 第 242 行（系统配置轮换先于根槽 FUA 落盘）：一次覆盖写那次发布只落了系统配置轮换，
//! 它的单元、记录与根都没落。盘 0 两槽都自证不过的盘面有两份：一份是手工各改一字节（静默损坏，不是崩溃摆出来的）；
//! 一份是攻方坏实现 MUT-B1（`research/prompts/m3-formal-code-b26-r1-opus-model/tools/apply_mut_b.py`：盘 0 轮换写完新槽之后把另一槽清成全 0、
//! 与新槽同在一段里）在 harness 档能摆的最小形状——这次轮换在盘 0 上写到一半（撕裂）、另一槽清零落了。

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    mount_writable_with_space_admission, MountError, StillUnreadableAfterOneReread,
    SystemConfigurationUnreadableWhen, WitnessedCounterComparison,
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
    judge_writable_mount_refusal, judge_writable_mount_refusal_of_a_crash_state,
    read_recovery_fields, ComparedVersion, DiskAfterTheRefusal, ReadFaultsInTheCrashState,
    ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet, RefusalReasonOnTheD23List,
    WitnessedCounterBranchThatHolds, WritableMountRefusalVerdict,
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

/// 第一个文件之后的盘面，与在它上面跑一次覆盖写时那次发布的系统配置轮换（每块盘一槽，journal tail 写的是这次发布末条记录的计数器）。
fn pool_after_the_first_file_and_rotation_of_an_overwrite() -> (MemoryPool, Vec<RetainedWrite>) {
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
        "前提：覆盖写那次发布写了记录与根（摆出来的盘面上都不落）"
    );
    (pool_of(&first_file), rotation)
}

/// 第一个文件之后的盘面上，只落了一次覆盖写那次发布的系统配置轮换，它的单元、记录与根都没落：系统配置见证过一次比盘上任何一版都新的发布。
/// 崩溃镜像读不报错。
fn crash_image_where_only_the_rotation_of_an_overwrite_landed() -> (MemoryPool, MemoryPool) {
    let (healthy, rotation) = pool_after_the_first_file_and_rotation_of_an_overwrite();
    let mut crash_image = healthy.clone();
    crash_image.apply_writes(&rotation);
    (healthy, crash_image)
}

/// 原地覆写写到一半的那份字节：旧字节与新字节不同的那一截，前一半取新、后一半取旧（与层 0 撕裂第三态同一个取法，
/// checker 档 `crash::torn_image_of_in_place_overwrite`；harness 档不依赖 checker 档，这里留一份）。新旧逐字节相同时交回新字节。
fn torn_image_of_an_in_place_overwrite(old: &[u8], new: &[u8]) -> Vec<u8> {
    assert_eq!(old.len(), new.len(), "旧字节与这次写一样长");
    let differs = |index: &usize| old[*index] != new[*index];
    let Some(first_differing_byte) = (0..new.len()).find(differs) else {
        return new.to_vec();
    };
    let last_differing_byte = (0..new.len())
        .rev()
        .find(differs)
        .expect("有第一个不同的字节就有最后一个");
    let differing_span_in_bytes = last_differing_byte - first_differing_byte + 1;
    let first_old_byte = first_differing_byte + differing_span_in_bytes / 2;
    let mut torn = new.to_vec();
    torn[first_old_byte..].copy_from_slice(&old[first_old_byte..]);
    torn
}

/// 攻方坏实现 MUT-B1 在 harness 档能摆的最小形状：第一个文件之后的盘面上，覆盖写那次发布在盘 0 的轮换写到一半（撕裂，旧槽内容也没了），
/// 坏实现紧跟着把盘 0 另一槽清成全 0 的那次写落了（与轮换同在一段里，段内真子集摆得出）；盘 1 不动。盘 0 一份自证过的系统配置都没有，
/// 设备不报错。
fn crash_image_where_a_rotation_tore_and_the_other_slot_of_device_zero_was_zeroed(
) -> (MemoryPool, MemoryPool) {
    let (healthy, rotation) = pool_after_the_first_file_and_rotation_of_an_overwrite();
    let rotation_on_device_zero = rotation
        .iter()
        .find(|write| write.device == DeviceIdentity(0))
        .expect("前提：覆盖写那次发布在盘 0 轮换一槽");
    let slot_spacing = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    assert_eq!(
        rotation_on_device_zero.length_in_bytes(),
        SYSTEM_CONFIGURATION_SLOT_BYTES,
        "前提：轮换那次写正好一个系统配置槽"
    );
    let rotated_slot_index = rotation_on_device_zero.offset.0 / slot_spacing;
    assert!(
        rotated_slot_index < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
            && rotation_on_device_zero.offset.0 % slot_spacing == 0,
        "前提：轮换写在盘 0 的一个系统配置槽上：{:?}",
        rotation_on_device_zero.offset
    );
    let other_slot_offset = DeviceOffsetInBytes(
        ((rotated_slot_index + 1) % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * slot_spacing,
    );
    let old_bytes = healthy
        .read(
            DeviceIdentity(0),
            rotation_on_device_zero.offset,
            slot_bytes,
        )
        .expect("内存盘上系统配置槽读得出");
    let mut new_bytes = vec![0u8; slot_bytes];
    rotation_on_device_zero
        .contents
        .copy_range_into(0, &mut new_bytes);
    let mut crash_image = healthy.clone();
    let device_zero = crash_image
        .devices
        .get_mut(&DeviceIdentity(0))
        .expect("有盘 0");
    device_zero.write(
        rotation_on_device_zero.offset,
        &torn_image_of_an_in_place_overwrite(&old_bytes, &new_bytes),
    );
    device_zero.write(other_slot_offset, &vec![0u8; slot_bytes]);
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

/// 前提：盘 0 一份自证过的系统配置都没有，取号之前的逐盘核以读阶段判完之后系统配置又读不出拒（C664 之后不报成判据 N-配置 的拒因）。
fn assert_refused_after_the_read_stage_for_device_zero(error: &MountError) {
    assert!(
        matches!(
            error,
            MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable)
                if unreadable.unreadable
                    == SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                        device: DeviceIdentity(0)
                    }
        ),
        "前提：盘 0 一份自证过的系统配置都没有，取号之前的逐盘核以读阶段之后系统配置又读不出拒：{error:?}"
    );
}

/// 手工撕裂的系统配置（静默损坏，不是崩溃摆出来的）：盘 0 两槽各改一字节、没注入读故障，实现以「读阶段判完之后系统配置又读不出」拒。
/// 那一拒的判据在崩溃镜像上复算为真（R2 之下撕裂的槽不算判不出，盘 0 一份自证过的都没有），U6 四条都满足；改法 G4：没注入读故障的崩溃态上
/// 判据为真，正确实现摆不出这份盘面，拒得不合法。
#[test]
fn a_refusal_after_the_read_stage_on_a_crash_image_without_read_faults_where_device_zero_has_both_system_configuration_slots_torn_is_illegal(
) {
    let (healthy, crash_image) =
        crash_image_where_both_system_configuration_slots_of_device_zero_are_torn();
    let (error, after) = refused_writable_mount_on(&crash_image);
    assert_refused_after_the_read_stage_for_device_zero(&error);
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
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
            conditions_not_met: vec![
                RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults,
            ],
        }
    );
}

/// 同一份手工撕裂的盘面，调用方说这个崩溃态注入过读故障：G4 那一条不判，那一拒照 U6 判——判据在崩溃镜像上复算为真、盘没变、
/// 读出在允许集合里，拒本身成立，合法。
#[test]
fn the_same_refusal_after_the_read_stage_in_a_crash_state_with_read_faults_injected_is_legal() {
    let (healthy, crash_image) =
        crash_image_where_both_system_configuration_slots_of_device_zero_are_torn();
    let (error, after) = refused_writable_mount_on(&crash_image);
    assert_refused_after_the_read_stage_for_device_zero(&error);
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
            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
        )
    );
}

/// 攻方坏实现 MUT-B1 的最小形状（盘 0 轮换撕裂、另一槽清零落了），没注入读故障：实现以「读阶段判完之后系统配置又读不出」拒。
/// 只判 U6 四条的判官（攻方量的那一版第二截）判它合法；改法 G4 之后，崩溃态判官判它拒得不合法、只差 G4 那一条。
#[test]
fn a_refusal_after_the_read_stage_without_read_faults_where_a_rotation_tore_and_the_other_slot_of_device_zero_was_zeroed_turns_from_legal_to_illegal(
) {
    let (healthy, crash_image) =
        crash_image_where_a_rotation_tore_and_the_other_slot_of_device_zero_was_zeroed();
    let (error, after) = refused_writable_mount_on(&crash_image);
    assert_refused_after_the_read_stage_for_device_zero(&error);
    let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
    let read_only_read_back = read_only_read_back_against_the_first_file(&healthy, &crash_image);
    assert_eq!(
        judge_writable_mount_refusal(
            &crash_image,
            fields.report.effective_root,
            &error,
            disk_after(&crash_image, &after),
            read_only_read_back,
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
        ),
        "前提：只判 U6 四条时这一拒合法（判据复算为真、盘没变、读出在允许集合里）"
    );
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
            reason: RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
            conditions_not_met: vec![
                RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults,
            ],
        }
    );
}
