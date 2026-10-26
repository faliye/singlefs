//! harness 测试文件 `crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::parameters;
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::admission::{
    admission_reading_of_a_writable_mount, AdmissionRefusedOnSomeDevices,
    AvailableBytesOnOneDevice, BytesOnOneDevice, DeviceShortOfDemand,
    JudgementThatFoundTheDeviceShort,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{mount_writable_with_space_admission, MountSpaceAdmission};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
    PerStepChecker, StepOutcome,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 记账标签 6（墓碑与退役预付，D3（空间分配） 已定项 2；退役那一份是判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条）
/// 摊到每块盘扣的字节：一个活对象、一条记录、墓碑树与退役树零根时两份副本 × ((2 + 0 + 1 + 2 + 0 + 1) × 16384 + 41 × 1 + 35 × 1) = 196760，
/// ÷ 2 = 98380（第 50 批之前 49193）。准入读数从发布路径接上它起每块盘多扣这么多。小盘的环窗口补足为退役那一份多补了 3 槽
/// （`crates/singlefs-harness/src/history.rs` 的 `SMALLER_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS`），下面两格「可用 X 槽」各比第 50 批之前多 3。
pub const TAG_SIX_SHARE_OF_ONE_DEVICE_IN_BYTES: i128 = (6 * 16384 + 41 + 35) * 2 / 2;

/// 造小盘盘面的那几次覆盖写（随机历史的覆盖写操作）：长度选择子与填充种子都固定，同一段历史逐字节复现；选择子 2999 ⇒ 3000 字节。
pub const SMALL_POOL_OVERWRITE_LENGTH_SELECTOR: u64 = 2999;

pub const SMALL_POOL_OVERWRITE_FILL_SEED: u64 = 1;

/// 一块小盘上的盘面：mkfs 同一个进程里发完第一个文件，再可写挂载一次、覆盖写 `overwrites` 次（内容见上面两个常量），
/// 空间准入关掉（只供测试的开关）——这几次都照写，造出的是一份合法的、占得很满的盘面；准入判不判，由下面的用例在这份盘面上各自判。
/// 交回最后一步之后的整份镜像与这段历史的盘宽。
pub fn small_pool_image_after_overwrites(
    device_width: HistoryDeviceWidth,
    overwrites: usize,
) -> MemoryPool {
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit {
            selector: SMALL_POOL_OVERWRITE_LENGTH_SELECTOR,
        },
        fill_seed: SMALL_POOL_OVERWRITE_FILL_SEED,
    });
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(std::iter::repeat_n(overwrite, overwrites))
            .collect(),
    };
    let mut last_image = None;
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
        },
        &SharedStream::new(),
        &mut |observation| last_image = Some(observation.image.clone()),
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        run.outcomes
            .iter()
            .all(|outcome| matches!(outcome, StepOutcome::Applied(_))),
        "挂载与 {overwrites} 次覆盖写都做成（每一步的池级 checker 归 checker 档两段）：{:?}",
        run.outcomes
    );
    last_image.expect("每一步之后都有一份镜像")
}

/// 一份镜像交给两块录着的内存盘（录进 `stream`）。
pub fn devices_on_the_image(
    image: &MemoryPool,
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device =
                SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect()
}

/// 两块内存盘此刻的整份镜像。
pub fn image_of_the_devices(
    devices: &[(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)],
    device_size_in_bytes: u64,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes,
    }
}

/// 可写挂载那一处（C363 (b) 判决第四节第 2 条接进来；D16（发布语义） 已定项 1「准入」那一行定了不够时怎么办，用户 2026-09-26 定）：
/// 两块单元区 256 槽的小盘，第一个文件之后可写挂载、覆盖写 12 次的盘面上再可写挂载——取号之前按这次挂载的 rows0 算
/// （D28（挂载期承诺量） 已定项 3），扣掉实例切换预留、checkpoint 保留池（已定项 4，按最坏情况计，用户 2026-09-27 定）与已分配之后，
/// 两块盘的可用(d) 都是 −36 槽 < 0（第 50 批之前 −39；A3b 把小盘改成 6 MiB 环、单元区从 1408 起之前是 −47 槽）：
/// 实例切换的预留拿不到。写行那次发布照走切换预留、写完行与暖机之后推抬 F 的空发布、再判：这里推一串（F 抬到上限 14，3 次空发布），
/// 再判就够了，挂载做成（取号 3），交回 `MountSpaceAdmission::AdmittedAfterTheFloorRaises`，带着取号之前那一判的拒绝原样。
/// 推的那一串的根都带新 F = 14、接在暖机后面，现行那一版就是最后一次；挂载之后的镜像池级 checker 0 违例；按可写挂载的读数再判，每块盘可用 ≥ 0。
/// 判别力：同一份盘面上只供测试的开关关掉准入，同一次挂载一次都不推（`NotJudgedByTheTestOnlySwitch`）——推是式子判出来的，不是落点逼的。
pub fn writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_the_row_publish_and_is_admitted(
    images: &mut PoolCheckerImages,
) {
    let device_width = HistoryDeviceWidth::UnitAreaOf256Slots;
    let image = small_pool_image_after_overwrites(device_width, 12);

    let stream = SharedStream::new();
    let mut devices = devices_on_the_image(&image, &stream);
    let mounted = mount_writable_with_space_admission(
        &device_width.parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("取号之前不够的挂载写行之后推抬 F、再判够了，挂载做成");
    assert_eq!(mounted.output.instance, InstanceGeneration(3));
    let MountSpaceAdmission::AdmittedAfterTheFloorRaises {
        refusal_before_acquisition,
        floor_raises,
    } = &mounted.output.space_admission
    else {
        panic!(
            "取号之前不够、写行之后推了再判够了：{:?}",
            mounted.output.space_admission
        );
    };
    assert_eq!(
        refusal_before_acquisition,
        &AdmissionRefusedOnSomeDevices {
            short_devices: [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(|device| DeviceShortOfDemand {
                    device,
                    judged_short_by:
                        JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand,
                    available: AvailableBytesOnOneDevice(
                        -36 * i128::from(SLOT_BYTES) - TAG_SIX_SHARE_OF_ONE_DEVICE_IN_BYTES,
                    ),
                    demand: BytesOnOneDevice::ZERO,
                })
                .collect(),
        },
        "取号之前两块盘都短：可用 −36 槽再减标签 6 每盘那一份，挂载这一刻不另要需求"
    );
    assert_eq!(floor_raises.len(), 1, "推一串就够");
    let raised = &floor_raises[0];
    assert_eq!(raised.ceiling, CheckpointTxg(14), "F 抬到这一刻的上限");
    let last_warm_up_txg = mounted
        .output
        .warm_up_publishes
        .last()
        .expect("写行之后暖机到本实例的根覆盖每块盘")
        .root()
        .checkpoint_txg;
    assert_eq!(
        raised
            .publishes
            .iter()
            .map(|publish| (publish.root.checkpoint_txg, publish.root.rollback_floor))
            .collect::<Vec<_>>(),
        (1..=3)
            .map(|offset| (
                CheckpointTxg(last_warm_up_txg.0 + offset),
                CheckpointTxg(14)
            ))
            .collect::<Vec<_>>(),
        "三次空发布接在暖机后面、根都带新 F"
    );
    assert_eq!(
        mounted.current.root().checkpoint_txg,
        CheckpointTxg(last_warm_up_txg.0 + 3),
        "现行那一版是推的最后一次"
    );
    for (device, available) in admission_reading_of_a_writable_mount(
        &mounted.allocator,
        mounted.current.file_version(),
        parameters().geometry.root_ring_slots_per_region,
    )
    .available_on_each_device()
    {
        assert!(
            available.0 >= 0,
            "盘 {device:?}：推过之后按可写挂载的读数再判够了：{available:?}"
        );
    }
    let after = image_of_the_devices(&devices, image.device_size_in_bytes);
    images.hand_out(
        "挂载之后的镜像",
        || after.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );

    let not_judged = mount_writable_with_space_admission(
        &device_width.parameters(),
        &mut devices_on_the_image(&image, &SharedStream::new()),
        SpaceAdmission::SkippedByTheTestOnlySwitch,
    )
    .expect("同一份盘面上关掉准入，同一次挂载做成");
    assert!(
        matches!(
            not_judged.output.space_admission,
            MountSpaceAdmission::NotJudgedByTheTestOnlySwitch
        ),
        "关掉准入一次都不推：{:?}",
        not_judged.output.space_admission
    );
}
