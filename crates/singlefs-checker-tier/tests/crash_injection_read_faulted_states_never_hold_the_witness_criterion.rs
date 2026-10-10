//! checker 档模块：无（自己逐个造崩溃状态，只用 harness 档的随机历史、内存池、可写挂载与挂载透明判官）
//! C696（崩溃注入的读故障一支在随机历史里从没触发）判别力自证里「第三截调用点写死成 NoneInjected」那一种接错，在随机历史上是等价变异
//! （第 26 批 C696 设计调查报告第三节）：写死之后只在三样都成立的态上改判定——第一次崩溃那一态注入过读故障、
//! 它上面那次可写挂载途中的二次崩溃镜像上再挂以判据 N-配置 拒、那个二次崩溃镜像上判据 W（`judge_writable_mount_refusal_of_a_crash_state`
//! 没注入读故障时报的 C670 那一条）为真。这里照 `.claude/singlefs-ai-sop/rules/test-discipline.md`「变异测试证明的是断言会红，不是覆盖」
//! 第 2 点把等价性写成一条测试留档：在「每块盘最新那一槽系统配置读回全 0 的可写挂载」那一组比重的历史上，那一步发过写之后的每个崩溃态
//! （段内前缀）与它们上面可写挂载的每个写前缀（第三截的二次崩溃镜像），W 一个都不为真。真会改行为的那一种接错由
//! `crash_injection::tests::a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount_is_not_judged_by_c670`
//! 那条靶向用例钉（W 为真的阳性对照）。
//! 推翻「等价」的观测：这里任一个态上 W 为真。量的范围：一组比重、一档盘宽（单元区 384 槽）、种子基起前 4 段那一步发过写的历史、
//! 段内只摆前缀、第三截只摆挂载写的前缀；更宽的范围没量。

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    mount_writable_with_space_admission, MountError, NewerPublishWitness, RollbackTarget,
    SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread, WitnessedCounterComparison,
};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryExecution, HistoryOperationKind, HistorySeed, PerStepChecker, StepPosition,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields, ComparedVersion,
    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage,
    RefusalConditionNotMet, WritableMountRefusalVerdict,
};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
const HISTORIES: usize = 4;
const SEEDS_SEARCHED: u64 = 64;
const OPERATIONS_PER_HISTORY: usize = 24;

/// 判据 N-配置 为真、重读仍为真的一拒（判官自己在镜像上复算判据，不看这里填的读数）。
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

/// W：这个镜像当作没注入读故障的崩溃态，判官对一次判据 N-配置 的拒报不报 C670 那一条（判据在镜像上复算、撕裂的系统配置槽跳过仍为真）。
/// 只读恢复落不到一版时判据无从复算，交回假。
fn witness_criterion_holds_without_read_faults(image: &MemoryPool) -> bool {
    let fields = read_recovery_fields(image, ComparedVersion::TheRecoveredVersion);
    let Some(selected_version) = fields.report.effective_root else {
        return false;
    };
    match judge_writable_mount_refusal_of_a_crash_state(
        image,
        fields.report.effective_root,
        &witness_refusal_claimed_on(selected_version),
        DiskAfterTheRefusal::ByteForByteUnchanged,
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
        ReadFaultsInTheCrashState::NoneInjected,
    ) {
        WritableMountRefusalVerdict::Illegal {
            conditions_not_met, ..
        } => conditions_not_met.iter().any(|condition| {
            matches!(
                condition,
                RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(_)
            )
        }),
        WritableMountRefusalVerdict::Legal(_)
        | WritableMountRefusalVerdict::ReasonNotOnTheD23List => false,
    }
}

/// 崩溃镜像上起一次可写挂载（与崩溃注入第二截同一种做法），交回挂载写表（做成与否都交，被拒时是空的）。
fn writes_of_a_writable_mount_on(image: &MemoryPool) -> Vec<RetainedWrite> {
    let parameters = WIDTH.parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                image.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let _mounted = mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    );
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &WIDTH.fixed_geometry(),
    );
    writes
}

/// 场景：那一组比重、种子基起前 4 段「每块盘最新那一槽系统配置读回全 0 的可写挂载」发过写的历史，每段 24 步。崩溃态取那一步发的第一个写
/// 之后结束的每一段的每个前缀（崩溃窗口末尾在那一步之后，崩溃注入认成注入过读故障的那一类）；每段最长那个前缀上再起一次可写挂载，
/// 挂载写的每个前缀是一个第三截的二次崩溃镜像。
/// 预期：这些镜像上 W 一个都不为真（写死 NoneInjected 与正确接线在这批态上判出一样的东西）；崩溃态与二次崩溃镜像各至少一个（不是空判）。
#[test]
#[ignore = "checker 档：自己逐个摆崩溃态复算判据，release 下跑"]
fn read_faulted_crash_states_and_their_second_crashes_never_hold_the_witness_criterion() {
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: WIDTH,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let mut histories_taken = 0usize;
    let mut read_faulted_crash_states = 0u64;
    let mut second_crash_images = 0u64;
    let mut images_holding_the_witness_criterion: Vec<String> = Vec::new();
    // 迭代上界是 SEEDS_SEARCHED 个种子；跨轮携带三样计数与 W 为真的态的描述；取够 HISTORIES 段就停。
    for seed_offset in 0..SEEDS_SEARCHED {
        if histories_taken == HISTORIES {
            break;
        }
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + seed_offset;
        let history = generate_history_with_weights(
            HistorySeed(seed),
            OPERATIONS_PER_HISTORY,
            &GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS,
        );
        let stream = SharedStream::retaining_contents();
        let mut operations_after_each_step: Vec<(StepPosition, usize)> = Vec::new();
        execute_history_with(&history, execution, &stream, &mut |observation| {
            operations_after_each_step.push((observation.position, stream.operation_count()));
        });
        let mut operations_before_the_step = 0usize;
        let mut first_stream_index_of_the_step = None;
        for (position, operations_after_the_step) in &operations_after_each_step {
            if let StepPosition::Operation(step_index) = position {
                if first_stream_index_of_the_step.is_none()
                    && history.operations[*step_index].kind()
                        == HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
                    && operations_before_the_step < *operations_after_the_step
                {
                    first_stream_index_of_the_step = Some(operations_before_the_step);
                }
            }
            operations_before_the_step = *operations_after_the_step;
        }
        let Some(first_stream_index_of_the_step) = first_stream_index_of_the_step else {
            continue;
        };
        let operations_after_the_last_step = operations_before_the_step;
        let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
            &stream.retained_operations(),
            &WIDTH.fixed_geometry(),
        );
        let mut base = MemoryPool::with_devices(&DEVICES, WIDTH.device_bytes());
        let mut writes_applied_to_the_base = 0usize;
        for segment in &segments {
            let first_write = segment[0];
            let last_stream_index = stream_indexes[*segment.last().expect("段非空")];
            if last_stream_index >= operations_after_the_last_step {
                break;
            }
            base.apply_writes(&writes[writes_applied_to_the_base..first_write]);
            writes_applied_to_the_base = first_write;
            if last_stream_index < first_stream_index_of_the_step {
                continue;
            }
            let mut longest_prefix = base.clone();
            for persisted_writes in 0..segment.len() {
                let mut image = base.clone();
                image.apply_writes(&writes[first_write..first_write + persisted_writes]);
                read_faulted_crash_states += 1;
                if witness_criterion_holds_without_read_faults(&image) {
                    images_holding_the_witness_criterion.push(format!(
                        "种子 {seed} 第一次崩溃：段首写 {first_write}、落了 {persisted_writes} 个"
                    ));
                }
                longest_prefix = image;
            }
            let mount_writes = writes_of_a_writable_mount_on(&longest_prefix);
            for persisted_mount_writes in 0..mount_writes.len() {
                let mut second_crash_image = longest_prefix.clone();
                second_crash_image.apply_writes(&mount_writes[..persisted_mount_writes]);
                second_crash_images += 1;
                if witness_criterion_holds_without_read_faults(&second_crash_image) {
                    images_holding_the_witness_criterion.push(format!(
                        "种子 {seed} 第三截：段首写 {first_write}、挂载写落了 {persisted_mount_writes} 个"
                    ));
                }
            }
        }
        histories_taken += 1;
    }
    assert_eq!(
        histories_taken, HISTORIES,
        "前提：种子基起 {SEEDS_SEARCHED} 段里有 {HISTORIES} 段那一步发过写"
    );
    assert!(
        read_faulted_crash_states >= 1 && second_crash_images >= 1,
        "前提：摆到了注入过读故障的崩溃态 {read_faulted_crash_states} 个、第三截二次崩溃镜像 {second_crash_images} 个"
    );
    assert_eq!(
        images_holding_the_witness_criterion,
        Vec::<String>::new(),
        "注入过读故障的崩溃态 {read_faulted_crash_states} 个、二次崩溃镜像 {second_crash_images} 个上 W 都不为真（写死 NoneInjected 是等价变异）"
    );
}
