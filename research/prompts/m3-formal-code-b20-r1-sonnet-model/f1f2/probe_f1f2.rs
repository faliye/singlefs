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

// ======================================================================================================
// 以下是本轮腿（m3-formal-code-b20-r1 正推）加的探针：不在仓里，只在腿自己的副本上编、经内存包装跑。
// 只用 harness 与 core 的公开入口，把 checker 档第二截、第三截调用点接线那几处在 harness 档复做一遍，
// 每一种接错各打一行，看计数落在哪。
// ======================================================================================================

use singlefs_harness::RecordingBlockDevice;

/// 在 `image` 的副本上起一次可写挂载（读不报错）：做成交回 Ok，被拒交回错误；都交回挂载之后的盘面。
fn try_writable_mount_on(image: &MemoryPool) -> (Result<(), MountError>, MemoryPool) {
    let parameters = SMALL.parameters();
    let physical_block_size = PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size);
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(image.device_size_in_bytes, physical_block_size);
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect();
    let result = mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .map(|_| ());
    let after = MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.image))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    };
    (result, after)
}

/// F1：第二截调用点三种接法在同一份「只落了覆盖写那次轮换」的盘面上，判官列出的没满足的条件。
/// 第二截测试里的两条条件断言：`contains("WitnessCriterionTrueOnACrashImageWithoutReadFaults")` 与
/// `!contains("CriterionFalseWhenRecomputedOnTheCrashImage")`。
#[test]
fn probe_f1_conditions_listed_by_the_judge_under_each_wiring_of_the_second_cut() {
    let (healthy, crash_image) = crash_image_where_only_the_rotation_of_an_overwrite_landed();
    let (error, after) = refused_writable_mount_on(&crash_image);
    let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
    let read_back = read_only_read_back_against_the_first_file(&healthy, &crash_image);
    let disk = disk_after(&crash_image, &after);
    println!("PROBE F1 effective_root = {:?}", fields.report.effective_root);
    for (label, selected, faults) in [
        ("接对（所选那一版 = effective_root，读故障 = NoneInjected）", fields.report.effective_root, ReadFaultsInTheCrashState::NoneInjected),
        ("所选那一版交成 None（mutations.tsv 2362）", None, ReadFaultsInTheCrashState::NoneInjected),
        ("读故障恒交 Injected（旧变异行）", fields.report.effective_root, ReadFaultsInTheCrashState::Injected),
    ] {
        let verdict = judge_writable_mount_refusal_of_a_crash_state(&crash_image, selected, &error, disk, read_back, faults);
        let (legal, illegal, conditions) = match &verdict {
            WritableMountRefusalVerdict::Legal(_) => (1, 0, String::new()),
            WritableMountRefusalVerdict::Illegal { conditions_not_met, .. } => (0, 1, format!("{conditions_not_met:?}")),
            WritableMountRefusalVerdict::ReasonNotOnTheD23List => (0, 0, String::new()),
        };
        let has_c670 = conditions.contains("WitnessCriterionTrueOnACrashImageWithoutReadFaults");
        let has_false = conditions.contains("CriterionFalseWhenRecomputedOnTheCrashImage");
        println!("PROBE F1 [{label}] (合法,不合法)=({legal},{illegal}) 条件文本={conditions:?}");
        println!("PROBE F1 [{label}] 断言①含 C670 = {has_c670}；断言②不含「复算为假」 = {}", !has_false);
    }
}

/// F2：第二截那次可写挂载（起在覆盖写一个写都没落的盘面上）写出的写表、写行那次发布的「根槽没落、两槽轮换落了」二次崩溃镜像，
/// 以及第三截调用点接对、四种接错下再挂的结局与计数（判过 / 不合法）。
#[test]
fn probe_f2_third_cut_wiring_on_the_row_publish_rotation_only_second_crash() {
    let first_file = recorded_stream_after_the_first_file_then(Vec::new());
    let first_crash_image = pool_of(&first_file);
    let parameters = SMALL.parameters();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    // 与 mount_and_publish_once_on_the_crash_image 同一种录法：挂载那一段录进共享流。
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = first_crash_image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                first_crash_image.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()))
        })
        .collect();
    let mounted = mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula)
        .expect("起点那一版上的可写挂载做成");
    drop(mounted);
    let (mount_writes, mount_segments, _) =
        writes_and_segments_with_stream_indexes(&stream.retained_operations(), &geometry);
    println!("PROBE F2 挂载写表 {} 个写，{} 段", mount_writes.len(), mount_segments.len());
    for (segment_index, segment) in mount_segments.iter().enumerate() {
        let kinds: Vec<String> = segment.iter().map(|w| format!("{}:{:?}", w, mount_writes[*w].kind)).collect();
        println!("PROBE F2 段 {segment_index}: {}", kinds.join(" "));
    }
    let root_of_the_row_publish = (0..mount_writes.len())
        .find(|w| mount_writes[*w].kind == StepKind::RootRecordFua)
        .expect("写行那次发布写了根槽");
    let rotation_after_the_root: Vec<usize> = (root_of_the_row_publish + 1..mount_writes.len())
        .take_while(|w| mount_writes[*w].kind == StepKind::SystemConfigurationSlot)
        .collect();
    println!("PROBE F2 写行根槽 = 写 {root_of_the_row_publish}，其后紧跟的轮换写 = {rotation_after_the_root:?}");
    let segment_of_the_root = mount_segments.iter().position(|s| s.contains(&root_of_the_row_publish)).expect("段");
    let first_of_segment = mount_segments[segment_of_the_root][0];
    let last_rotation = *rotation_after_the_root.last().expect("有轮换");
    let window = last_rotation + 1 - first_of_segment;
    let withheld = root_of_the_row_publish - first_of_segment;
    println!("PROBE F2 靶向二次崩溃：段 {segment_of_the_root}（段内写 {:?}），窗口 {window} 个写，扣下窗口里第 {withheld} 个", mount_segments[segment_of_the_root]);
    // 二次崩溃镜像：段首之前整段持久，窗口里除了扣下的都持久，窗口之后一个都没持久。
    let mut second_base = first_crash_image.clone();
    second_base.apply_writes(&mount_writes[..first_of_segment]);
    let mut second_image = second_base.clone();
    for within in 0..window {
        if within != withheld {
            second_image.apply_writes(std::slice::from_ref(&mount_writes[first_of_segment + within]));
        }
    }
    let effective_second = read_recovery_fields(&second_image, ComparedVersion::TheRecoveredVersion).report.effective_root;
    let effective_first = read_recovery_fields(&first_crash_image, ComparedVersion::TheRecoveredVersion).report.effective_root;
    println!("PROBE F2 只读恢复：第一次崩溃镜像 {effective_first:?}；二次崩溃镜像 {effective_second:?}；两者相同 = {}", effective_first == effective_second);
    // 第三截判再挂被拒：与 conditions_not_met_by_a_remount_refusal_with_the_witness_criterion_true 同一个计数口径。
    let judge = |label: &str, target: &MemoryPool, selected, faults| {
        let (result, after) = try_writable_mount_on(target);
        match result {
            Ok(()) => println!("PROBE F2 [{label}] 再挂做成（没被拒）→ 判过 0、不合法 0"),
            Err(error) => {
                let disk = if after == *target { DiskAfterTheRefusal::ByteForByteUnchanged } else { DiskAfterTheRefusal::Changed };
                let verdict = judge_writable_mount_refusal_of_a_crash_state(
                    target, selected, &error, disk, ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet, faults,
                );
                let (judged, c670) = match &verdict {
                    WritableMountRefusalVerdict::ReasonNotOnTheD23List
                    | WritableMountRefusalVerdict::Legal(RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable)
                    | WritableMountRefusalVerdict::Illegal { reason: RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable, .. } => (0, 0),
                    WritableMountRefusalVerdict::Legal(RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration) => (1, 0),
                    WritableMountRefusalVerdict::Illegal { reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration, conditions_not_met } => (
                        1,
                        i32::from(conditions_not_met.iter().any(|c| matches!(c, RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(_)))),
                    ),
                };
                println!("PROBE F2 [{label}] 再挂被拒 → 判过 {judged}、不合法 {c670}（verdict={verdict:?}）");
            }
        }
    };
    judge("接对", &second_image, effective_second, ReadFaultsInTheCrashState::NoneInjected);
    judge("读故障恒交 Injected（tsv 2363）", &second_image, effective_second, ReadFaultsInTheCrashState::Injected);
    judge("再挂的盘 = 第一次崩溃的镜像（tsv 2364）", &first_crash_image, effective_second, ReadFaultsInTheCrashState::NoneInjected);
    judge("再挂的盘 = 没叠这一段的基线 second_base（tsv 2365）", &second_base, effective_second, ReadFaultsInTheCrashState::NoneInjected);
    judge("所选那一版交成 None（没有变异行）", &second_image, None, ReadFaultsInTheCrashState::NoneInjected);
    judge("所选那一版交成第一次崩溃那一个（没有变异行）", &second_image, effective_first, ReadFaultsInTheCrashState::NoneInjected);
}
