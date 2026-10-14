//! harness 测试文件 `crates/singlefs-harness/tests/fault_injection_fast_tier.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::recovery::{
    choose_system_configuration, readable_roots, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, resend_the_frozen_publish, warm_up,
    FirstFile, PoolVersion, PoolWriter, PublishError,
};
use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
use singlefs_harness::fault_injection::{
    inject_one_fault_into_the_segment_handing_out_the_images, FailureAfterAFault, FaultCounting,
    FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement, FaultSchedule,
    FaultedSegment, HistoryFaultInjection, ImagesAfterAFault, InjectedFault,
    RecordedStreamOfAnImage, SharedFaultPlan,
};
use singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, ContentChoice, ContentLength,
    GeneratedHistory, GenerationWeights, HistoryEnding, HistoryExecution, HistoryOperation,
    HistoryOperationKind, HistorySeed, HistoryStartingPoint, StepPosition,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::collections::BTreeMap;

/// 写死的那几条用例里两块内存盘各多大（与各步用例、随机历史的 4 GiB 档相同）。
pub const IMAGE_BYTES: u64 = 4 << 30;

pub const FAST_TIER_OPERATIONS_PER_HISTORY: usize = 20;

/// 注入之后这段历史怎么跑：harness 不跑池级 checker，带着注入跑的那一遍每一步的镜像与重开的盘经钩子交出去，两块 4 GiB 的盘。
/// 注入之后盘面坏没坏只有池级 checker 看得见，那一遍归 checker 档两段（用户 2026-10-13 定：镜像作为给定状态写进判定库、判在 GPU，
/// `crates/singlefs-checker-tier/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs`）；
/// 这里写死的几条用例在交出去的盘上就地判，核的是归类那一截（[`classify_the_pool_checker_verdicts_of_the_images_after_a_fault`]）。
pub const HANDING_EVERY_STEP_IMAGE_OUT: HistoryExecution =
    HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;

/// 写死的那一段历史（第一个文件之后覆盖写五次，txg 4–8）在第 3 步覆盖写（txg 7）那次的根槽 FUA 写上吞掉一次写：harness 跑一遍注入
/// （不跑池级 checker），交回注入的结局与这个注入点交出去的盘（池级 checker 那一遍与归类在 checker 档同名用例）。
pub fn the_fixed_history_with_its_root_slot_write_swallowed(
) -> (HistoryFaultInjection, Vec<ImagesAfterAFault>) {
    const STEP_OF_THE_SWALLOWED_ROOT_SLOT_WRITE: usize = 3;
    let history = GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: (1..=5).map(overwrite_with_fill_seed).collect(),
    };
    let mut handed_out: Vec<ImagesAfterAFault> = Vec::new();
    let injection = inject_one_fault_into_the_segment_handing_out_the_images(
        &history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        InjectedFault::WriteIsSwallowed,
        FaultedSegment::Operation {
            step_index: STEP_OF_THE_SWALLOWED_ROOT_SLOT_WRITE,
            operation_kind: HistoryOperationKind::PublishOverwrite,
        },
        ordinal_of_the_root_slot_write_within_the_step(
            &history,
            STEP_OF_THE_SWALLOWED_ROOT_SLOT_WRITE,
        ),
        &mut |images| handed_out.push(images),
    );
    (injection, handed_out)
}

/// 快档在 2026-10-03 落进这一形的那一段：种子基 + 7（7463871032432355120），快档的规模与比重。
pub const SEED_WHERE_MAKE_FILESYSTEM_FAILS_BEFORE_ANY_SYSTEM_CONFIGURATION_SLOT_LANDS: u64 =
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 7;

/// [`SEED_WHERE_MAKE_FILESYSTEM_FAILS_BEFORE_ANY_SYSTEM_CONFIGURATION_SLOT_LANDS`] 那一段在整池第 1 次 barrier（mkfs 第一道屏障）上注入报错：
/// harness 跑一遍注入（不跑池级 checker），交回注入的结局与这个注入点交出去的盘（池级 checker 那一遍与归类在 checker 档同名用例）。
pub fn the_history_whose_make_filesystem_fails_at_its_first_barrier(
) -> (HistoryFaultInjection, Vec<ImagesAfterAFault>) {
    let history = generate_history_with_weights(
        HistorySeed(SEED_WHERE_MAKE_FILESYSTEM_FAILS_BEFORE_ANY_SYSTEM_CONFIGURATION_SLOT_LANDS),
        FAST_TIER_OPERATIONS_PER_HISTORY,
        &GenerationWeights::BROAD,
    );
    let mut handed_out: Vec<ImagesAfterAFault> = Vec::new();
    let injection = inject_one_fault_into_the_segment_handing_out_the_images(
        &history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        InjectedFault::BarrierFails,
        FaultedSegment::TheStartingPoint,
        1,
        &mut |images| handed_out.push(images),
    );
    (injection, handed_out)
}

/// 一个注入点交出去的盘重建出来的镜像（池级 checker 那一遍归 checker 档两段；这里只造镜像）：带着注入跑的那一遍每一张每步镜像、
/// 重开的盘、被吞的写插回去之后的盘（有被吞的写才有）。
pub struct PoolImagesAfterAFault {
    pub step_images: Vec<MemoryPool>,
    pub after_the_fault: MemoryPool,
    pub with_the_swallowed_writes_landed: Option<MemoryPool>,
}

pub fn pool_images_after_a_fault(images: &ImagesAfterAFault) -> PoolImagesAfterAFault {
    let image_of = |stream: &RecordedStreamOfAnImage, operations_recorded: usize| {
        let mut image = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            images.device_width.device_bytes(),
        );
        image.apply(&stream.operations[..operations_recorded]);
        image
    };
    PoolImagesAfterAFault {
        step_images: images
            .step_images_of_the_run_with_the_fault
            .iter()
            .map(|step_image| {
                image_of(
                    &images.image_after_the_fault,
                    step_image.operations_recorded,
                )
            })
            .collect(),
        after_the_fault: image_of(
            &images.image_after_the_fault,
            images.image_after_the_fault.operations.len(),
        ),
        with_the_swallowed_writes_landed: images
            .image_with_the_swallowed_writes_landed
            .as_ref()
            .map(|landed| image_of(landed, landed.operations.len())),
    }
}

/// 归好类的失败一条一句（用例比的是这一串）。
pub fn rendered_failures_after_a_fault(failures: &[FailureAfterAFault]) -> Vec<String> {
    failures
        .iter()
        .map(|failure| match failure {
            FailureAfterAFault::LeftByTheLyingDevice { signature } => format!("说谎 {signature:?}"),
            FailureAfterAFault::KnownRed(form) => format!("已知红 {}", form.render()),
            FailureAfterAFault::NewFinding { signature, .. } => format!("新发现 {signature:?}"),
        })
        .collect()
}

/// 增补 2 收口表第 40 行（C381（根已落盘之后发布失败，分配器仍退回））那一格，按 D23（journal 的角色与格式） 已定项 14
/// 「这一版的失败处置」（用户 2026-09-24 定：发布不接受失败，失败的那次冻结，下一次发布之前逐字节原样重发它）：
/// 发布 B 在最后一步（系统配置槽）失败时根已 FUA 落盘、分配器退回——改之前同一个写入口拿同一个上一版再发一次 C，就把那条根指着的单元
/// 原地盖掉，那一次再断电，盘上留下「最新的那条根指着一份内容已经换掉的单元」（池级 checker 判 I-7.4 / I-7.2 红、冷启动走不到那条根）。
/// 现在 B 冻结在分配器上：再发 C 在任何写之前被拒；原样重发 B 之后那条根指着的单元就是它自己写的那一份，checker 不红、冷启动读回 B；
/// 之后 C 接在 B 上照常发布。
pub fn publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root(
    images: &mut PoolCheckerImages,
) {
    let parameters = e142_parameters(512, 512);
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let stream = SharedStream::retaining_contents();
    let plan = SharedFaultPlan::unarmed(geometry);
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<_>)> = (0..2u32)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                FaultInjectingBlockDevice::new(
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                    plan.clone(),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    // mkfs 同一个进程里接着写的分配器照 mkfs 按环长现算的单元区起点建（实审 A3b 报告 Q8：环境变量选小盘那一档时环不是默认的 768 MiB，
    // 起点不是编译期常量 50176），并装上 mkfs 刚写下的根环。
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let first = publish_first_file(
        &mut writer,
        &mut allocator,
        warmed.roots.last().expect("暖机两代根"),
        FirstFile {
            content: &first_file_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warmed.last_record_bytes,
    )
    .expect("新池新建文件");
    assert_eq!(first.root.checkpoint_txg, CheckpointTxg(3));

    // 一、发布 B：系统配置槽那两次写全报错。根槽 FUA 写在它们之前，已经落盘。
    plan.arm(FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::EveryDevice,
        placement: FaultPlacement::OffsetBelow(
            SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
                * u64::from(parameters.geometry.fixed_structure_slot_spacing),
        ),
        counting: FaultCounting::AcrossThePool,
        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
    });
    let refused = publish_overwrite(
        &mut writer,
        &mut allocator,
        &first,
        FirstFile {
            content: &second_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect_err("系统配置槽写报错，这次发布必须失败");
    assert!(
        matches!(refused, PublishError::BlockDevice(_)),
        "报的应当是块设备错：{refused:?}"
    );
    let fired = plan.fired();
    assert_eq!(fired.len(), 1, "系统配置槽第一次写就报错、发布到此为止");
    assert_eq!(
        fired[0].written_structure,
        Some(StepKind::SystemConfigurationSlot),
        "注入的那一次要落在系统配置槽上"
    );
    let image_after_the_failed_publish = image_from(&stream);
    let roots_after: Vec<u64> = readable_root_txgs(&image_after_the_failed_publish);
    assert!(
        roots_after.contains(&4),
        "C381 的前提：这次发布报了失败，txg 4 那条根却已经 FUA 落盘、在环里读得出来；读到的是 {roots_after:?}"
    );

    // 二、同一个写入口、同一个上一版再发一次：B 冻结着，第一道就拒，一个写都不发。
    let fired_before_the_refused_publish = plan.fired_count();
    let operations_before_the_refused_publish = stream.operations().len();
    let refused_again = publish_overwrite(
        &mut writer,
        &mut allocator,
        &first,
        FirstFile {
            content: &third_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
        },
        instance,
    )
    .expect_err("B 冻结着，另建一次发布必须被拒");
    assert!(
        matches!(
            refused_again,
            PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
                checkpoint_txg: CheckpointTxg(4),
                ..
            }
        ),
        "报的应当是「B 冻结着、还没重发」：{refused_again:?}"
    );
    assert_eq!(
        (plan.fired_count(), stream.operations().len()),
        (
            fired_before_the_refused_publish,
            operations_before_the_refused_publish
        ),
        "被拒的那一次一个写、一道屏障都没发"
    );

    // 三、盘好了，原样重发 B；之后 C 接在 B 上照常发布。
    plan.disarm();
    let second = resend_the_frozen_publish(&mut writer, &mut allocator)
        .expect("盘好了，重发这一遍不再报错")
        .and_then(PoolVersion::into_file_version)
        .expect("冻结着的是带文件的发布 B");
    assert_eq!(second.root.checkpoint_txg, CheckpointTxg(4));
    let image_after_the_resend = image_from(&stream);
    // 重发之后最新那条根（B 的 txg 4）指着的单元就是它自己写的那一份：checker 不红
    images.hand_out(
        "原样重发 B 之后",
        || image_after_the_resend.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    let recovery = recover(&image_after_the_resend, JournalPolicy::Consult);
    assert!(
        matches!(
            &recovery.outcome,
            RecoveryOutcome::FileRead { root, content }
                if *root == (instance, CheckpointTxg(4)) && *content == second_content()
        ),
        "冷启动读回 B：{:?}",
        recovery.outcome
    );
    let third = publish_overwrite(
        &mut writer,
        &mut allocator,
        &second,
        FirstFile {
            content: &third_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
        },
        instance,
    )
    .expect("重发之后 C 照常发布");
    assert_eq!(third.root.checkpoint_txg, CheckpointTxg(5));
}

/// 发布 B 的内容（4100 字节，与虚机二进制 `file-overwrite` 模式相同）。
pub fn second_content() -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 发布 C 的内容：与发布 B 不同，单元字节才会真的变。
pub fn third_content() -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 11 + 5) % 251).expect("小于 256"))
        .collect()
}

/// 录制流里真正落到盘上的那些写重建出来的镜像。
pub fn image_from(stream: &SharedStream) -> MemoryPool {
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    image.apply(&stream.retained_operations());
    image
}

/// 第一个文件（txg 3）之后覆盖写，每次换一份内容。
pub fn overwrite_with_fill_seed(fill_seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed,
    })
}

/// 不注入跑一遍这段历史，数第 `step_index` 步发出的写里落在根槽上的那一次是这一步的第几次写（1 起）：
/// `inject_one_fault_into_the_segment` 按「这一段发出的第几次写」摆注入点。
pub fn ordinal_of_the_root_slot_write_within_the_step(
    history: &GeneratedHistory,
    step_index: usize,
) -> u64 {
    let stream = SharedStream::new();
    let mut stream_length_after: BTreeMap<StepPosition, usize> = BTreeMap::new();
    let run = execute_history_with(
        history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        &stream,
        &mut |observation| {
            stream_length_after.insert(observation.position, stream.operation_count());
        },
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "不注入时这段历史跑完");
    let step_start = match step_index.checked_sub(1) {
        None => stream_length_after[&StepPosition::StartingPoint],
        Some(previous) => stream_length_after[&StepPosition::Operation(previous)],
    };
    let step_end = stream_length_after[&StepPosition::Operation(step_index)];
    let geometry = HANDING_EVERY_STEP_IMAGE_OUT.device_width.fixed_geometry();
    let writes_of_the_step: Vec<_> = stream.operations()[step_start..step_end]
        .iter()
        .filter(|operation| operation.kind != singlefs_harness::RecordedOperationKind::Barrier)
        .cloned()
        .collect();
    let index = writes_of_the_step
        .iter()
        .position(|operation| geometry.classify(operation) == StepKind::RootRecordFua)
        .expect("一次发布恰好写一次根槽");
    u64::try_from(index + 1).expect("一步至多几十次写")
}

/// 环里自证过的根的 txg，从小到大。
pub fn readable_root_txgs(image: &MemoryPool) -> Vec<u64> {
    let system_configuration = choose_system_configuration(image).expect("择系统配置");
    let mut txgs: Vec<u64> = readable_roots(
        image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|root| root.checkpoint_txg.0)
    .collect();
    txgs.sort_unstable();
    txgs
}
