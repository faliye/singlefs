//! 验证包模块：crash、crash_amplification
//! 崩溃枚举用例里挂载观察者共用的那一判（`singlefs_verification::mount_transparency::image_after_the_refusal_into_the_undecidable_witness`）：
//! 可写挂载被拒时只认乙入判不出那一拒（D22（单元原子性怎么合成） 已定项 8 射程，C971（多处读坏时可写挂载写出倒挂的新实例根）），
//! 拒因、盘面、只读挂载、盘上没被写四样缺一样都判红；认下的态跳过流接在挂载后面的步，跳过的态数记在计数里。
//! 盘面是手摆的：第一个文件之后的池（或只做过 mkfs 的池）上每块盘系统配置槽 1 读回全零、槽 0 自证过，没有一块盘两槽都自证过。
//! 判别力在 `crates/mutations.tsv`（名字以「乙入判不出观察者」起头的几行）。

mod common;
mod common_crash_point_observers;
mod common_crash_points;

use common::{build_pool, crash_state_devices, format_pool, geometry, parameters};
use common_crash_point_observers::{
    mount_continuation_inputs_of_the_unlink_stream, observe_the_mount_continuation,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    mount_writable, MountError, NewerPublishWitness, RollbackTarget,
    SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread, WitnessedCounterComparison,
};
use singlefs_core::recovery::{
    recover, JournalPolicy, JournalScanReport, PoolReader, RecoveryFailure, RecoveryOutcome,
    RecoveryReport,
};
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_verification::crash::CrashPointReplayObserverCounts;
use singlefs_verification::crash_amplification::ImageAfterTheObserver;
use singlefs_verification::memory_pool::{CrashImage, MemoryPool, RetainedWrite, WrittenContents};
use singlefs_verification::mount_transparency::{
    continuation_skipped_on_the_refusal_counter,
    image_after_the_refusal_into_the_undecidable_witness, WritableMountRefusedOnTheCrashDisks,
    READ_ONLY_MOUNT_FOUND_NO_FILE_VERSION_AFTER_THE_REFUSAL_COUNTER,
    READ_ONLY_MOUNT_OPENED_AFTER_THE_REFUSAL_COUNTER,
    WRITABLE_MOUNT_REFUSED_INTO_THE_UNDECIDABLE_WITNESS_COUNTER,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::SharedStream;

/// 判段判干净时加一的计数名（这些用例不走判段，只核它原样交回去）。
const COUNTER_WHEN_THE_POOL_CHECKER_IS_CLEAN: &str = "pool_checker_clean_in_these_cases";

const THE_TWO_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 一块盘系统配置槽 `slot_index` 的设备内偏移：槽 i 在 i × 固定结构槽距（D22（单元原子性怎么合成） 已定项 16）。
fn system_configuration_slot_offset(slot_index: u64) -> DeviceOffsetInBytes {
    DeviceOffsetInBytes(slot_index * u64::from(geometry().fixed_structure_slot_spacing))
}

fn write_on_the_image(
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    contents: WrittenContents,
) -> RetainedWrite {
    RetainedWrite {
        device,
        kind: StepKind::SystemConfigurationSlot,
        is_force_unit_access: false,
        offset,
        contents,
    }
}

/// 每块盘系统配置槽 1 整槽清零：每块盘只剩槽 0 自证过，设备不报读错。
fn with_slot_one_zeroed_on_every_device(mut image: MemoryPool) -> MemoryPool {
    let writes = THE_TWO_DEVICES.map(|device| {
        write_on_the_image(
            device,
            system_configuration_slot_offset(1),
            WrittenContents::Zeros {
                length: SYSTEM_CONFIGURATION_SLOT_BYTES,
            },
        )
    });
    image.apply_writes(&writes);
    image
}

fn slot_zero_bytes_of(image: &MemoryPool, device: DeviceIdentity) -> Vec<u8> {
    PoolReader::read(
        image,
        device,
        system_configuration_slot_offset(0),
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节装得进 usize"),
    )
    .expect("内存盘上的系统配置槽读得出")
}

/// `image` 上真起一次可写挂载，要它被拒，交回拒因与挂载用过的录制设备。
fn writable_mount_refused_on(image: &MemoryPool) -> WritableMountRefusedOnTheCrashDisks {
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices = crash_state_devices(image, &[], &[], &stream);
    match mount_writable(&parameters(), &mut devices) {
        Ok(_) => panic!("前提：每块盘只剩一槽自证过，可写挂载按乙入判不出拒"),
        Err(refusal) => WritableMountRefusedOnTheCrashDisks {
            refusal,
            devices,
            stream,
        },
    }
}

/// `image` 上没起挂载，手给一个拒因：判拒因、盘面、只读挂载与写的那几条用。
fn claimed_refusal_on(
    image: &MemoryPool,
    refusal: MountError,
) -> WritableMountRefusedOnTheCrashDisks {
    let stream = SharedStream::retaining_contents_and_reads();
    let devices = crash_state_devices(image, &[], &[], &stream);
    WritableMountRefusedOnTheCrashDisks {
        refusal,
        devices,
        stream,
    }
}

fn reading_with_the_comparison(
    comparison: WitnessedCounterComparison,
) -> SelectedVersionAgainstTheWitness {
    SelectedVersionAgainstTheWitness {
        selected_version: RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(1),
        },
        witness: NewerPublishWitness {
            witnessed_journal_counter: 0,
            comparison,
        },
    }
}

fn refusal_with_the_comparisons(
    first_read: WitnessedCounterComparison,
    reread: WitnessedCounterComparison,
) -> MountError {
    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
            first_read: reading_with_the_comparison(first_read),
            reread: reading_with_the_comparison(reread),
        },
    ))
}

fn the_refusal_into_the_undecidable_witness() -> MountError {
    refusal_with_the_comparisons(
        WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots,
        WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots,
    )
}

fn recovery_report_with(
    outcome: RecoveryOutcome,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
) -> RecoveryReport {
    RecoveryReport {
        outcome,
        effective_root,
        journal: JournalScanReport::default(),
        mapping_fallbacks: 0,
        devices_without_a_mountable_system_configuration_slot: Vec::new(),
    }
}

fn judge_the_refusal(
    refused: &WritableMountRefusedOnTheCrashDisks,
    crash_disks: &MemoryPool,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    image_after_the_refusal_into_the_undecidable_witness(
        refused,
        crash_disks,
        report,
        &parameters().filesystem_identifier,
        &geometry(),
        COUNTER_WHEN_THE_POOL_CHECKER_IS_CLEAN,
        counts,
    )
}

/// 第一个文件之后的池上每块盘只剩一槽自证过：真起的可写挂载按乙入判不出拒，这一拒认下，只读挂载打开恢复落到的那一版，没改过的盘交回判段。
#[test]
fn the_refusal_on_a_pool_with_a_file_is_taken_and_the_read_only_mount_opens_the_landed_version() {
    let image = with_slot_one_zeroed_on_every_device(
        build_pool("refusal-into-the-undecidable-witness-with-a-file").memory_pool(),
    );
    let refused = writable_mount_refused_on(&image);
    let report = recover(&image, JournalPolicy::Consult);
    let mut counts = CrashPointReplayObserverCounts::default();
    let after = judge_the_refusal(&refused, &image, &report, &mut counts);
    assert_eq!(
        counts.count_named(WRITABLE_MOUNT_REFUSED_INTO_THE_UNDECIDABLE_WITNESS_COUNTER),
        1,
        "认下的这一态记进跳过的态数"
    );
    assert_eq!(
        counts.count_named(READ_ONLY_MOUNT_OPENED_AFTER_THE_REFUSAL_COUNTER),
        1,
        "带文件的一版上只读挂载打开了它"
    );
    assert_eq!(
        counts.count_named(READ_ONLY_MOUNT_FOUND_NO_FILE_VERSION_AFTER_THE_REFUSAL_COUNTER),
        0,
        "带文件的一版不算还没有文件版本"
    );
    assert!(
        after.writes_on_the_crash_image.is_empty(),
        "被拒的挂载没有写交回"
    );
    assert!(
        after.disks_after_the_observer == image,
        "交回的是没改过的盘"
    );
    assert!(
        !after.ranges_read_on_the_crash_image.is_empty(),
        "挂载与只读挂载读过的段进读集见证"
    );
    assert_eq!(
        after.counter_when_the_pool_checker_is_clean,
        COUNTER_WHEN_THE_POOL_CHECKER_IS_CLEAN
    );
}

/// 只做过 mkfs 的池：恢复落到第 0 代空池、只读挂载答树表 0 条，这是那一版的常规答复，这一拒照认，分开记数。
#[test]
fn the_refusal_on_a_formatted_pool_is_taken_when_the_read_only_mount_finds_no_file_version() {
    let image = with_slot_one_zeroed_on_every_device(
        format_pool("refusal-into-the-undecidable-witness-formatted").memory_pool_after_mkfs(),
    );
    let refused = writable_mount_refused_on(&image);
    let report = recover(&image, JournalPolicy::Consult);
    let mut counts = CrashPointReplayObserverCounts::default();
    judge_the_refusal(&refused, &image, &report, &mut counts);
    assert_eq!(
        counts.count_named(READ_ONLY_MOUNT_FOUND_NO_FILE_VERSION_AFTER_THE_REFUSAL_COUNTER),
        1,
        "还没有文件版本的一版记在这个数上"
    );
    assert_eq!(
        counts.count_named(READ_ONLY_MOUNT_OPENED_AFTER_THE_REFUSAL_COUNTER),
        0,
        "第 0 代空池上只读挂载打不开文件版本"
    );
    assert_eq!(
        counts.count_named(WRITABLE_MOUNT_REFUSED_INTO_THE_UNDECIDABLE_WITNESS_COUNTER),
        1,
        "认下的这一态记进跳过的态数"
    );
}

/// 比法是 R2 本身（有槽设备报读错），不是乙入判不出：别的拒不认。
#[test]
#[should_panic(expected = "拒因不是乙入判不出")]
fn a_refusal_comparing_a_slot_reporting_a_read_error_is_not_taken() {
    let image = with_slot_one_zeroed_on_every_device(
        build_pool("refusal-of-the-read-error-comparison").memory_pool(),
    );
    let refused = claimed_refusal_on(
        &image,
        refusal_with_the_comparisons(
            WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots,
            WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots,
        ),
    );
    let report = recover(&image, JournalPolicy::Consult);
    judge_the_refusal(
        &refused,
        &image,
        &report,
        &mut CrashPointReplayObserverCounts::default(),
    );
}

/// 只有重读那一遍是乙入判不出、读阶段头一遍是别的比法：两遍都得是这一形才认。
#[test]
#[should_panic(expected = "拒因不是乙入判不出")]
fn a_refusal_whose_earlier_read_compared_otherwise_is_not_taken() {
    let image = with_slot_one_zeroed_on_every_device(
        build_pool("refusal-whose-earlier-read-compared-otherwise").memory_pool(),
    );
    let refused = claimed_refusal_on(
        &image,
        refusal_with_the_comparisons(
            WitnessedCounterComparison::Undecidable,
            WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots,
        ),
    );
    let report = recover(&image, JournalPolicy::Consult);
    judge_the_refusal(
        &refused,
        &image,
        &report,
        &mut CrashPointReplayObserverCounts::default(),
    );
}

/// 同一个「重读一次仍读不出」的错误成员，读不出的是别的一样（重建分配器读实例表）：不认。
#[test]
#[should_panic(expected = "拒因不是乙入判不出")]
fn a_refusal_for_another_unreadable_thing_is_not_taken() {
    let image = with_slot_one_zeroed_on_every_device(
        build_pool("refusal-for-another-unreadable-thing").memory_pool(),
    );
    let report = recover(&image, JournalPolicy::Consult);
    let (instance, checkpoint_txg) = report
        .effective_root
        .expect("槽 1 全零、槽 0 自证过的盘面上恢复落到一版");
    let refused = claimed_refusal_on(
        &image,
        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
            StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger {
                root_chosen_by_the_read_stage: RollbackTarget {
                    instance,
                    checkpoint_txg,
                },
            },
        )),
    );
    judge_the_refusal(
        &refused,
        &image,
        &report,
        &mut CrashPointReplayObserverCounts::default(),
    );
}

/// 四槽都自证过的盘上报乙入判不出：盘面不是这一形，实现判错了，不认。
#[test]
#[should_panic(expected = "盘面不是这一形")]
fn the_refusal_on_disks_whose_four_slots_self_verify_is_not_taken() {
    let image = build_pool("refusal-on-four-self-verified-slots").memory_pool();
    let refused = claimed_refusal_on(&image, the_refusal_into_the_undecidable_witness());
    let report = recover(&image, JournalPolicy::Consult);
    judge_the_refusal(
        &refused,
        &image,
        &report,
        &mut CrashPointReplayObserverCounts::default(),
    );
}

/// 盘面是这一形，根环与 journal 一个字节都没有：只读挂载择不到根，做不成，这一拒不认（拒了却连只读都给不了）。
#[test]
#[should_panic(expected = "只读挂载做不成")]
fn the_refusal_on_disks_the_read_only_mount_cannot_open_is_not_taken() {
    let pool = build_pool("refusal-where-the-read-only-mount-fails").memory_pool();
    let mut image = MemoryPool::with_devices(&THE_TWO_DEVICES, pool.device_size_in_bytes);
    let writes = THE_TWO_DEVICES.map(|device| {
        write_on_the_image(
            device,
            system_configuration_slot_offset(0),
            WrittenContents::Bytes(slot_zero_bytes_of(&pool, device)),
        )
    });
    image.apply_writes(&writes);
    let refused = claimed_refusal_on(&image, the_refusal_into_the_undecidable_witness());
    let report = recovery_report_with(
        RecoveryOutcome::Failed {
            root: None,
            failure: RecoveryFailure::NoValidRoot,
        },
        None,
    );
    judge_the_refusal(
        &refused,
        &image,
        &report,
        &mut CrashPointReplayObserverCounts::default(),
    );
}

/// 只读挂载答树表 0 条，恢复却读出了文件：树表 0 条只在恢复也说没有文件的时候才算常规答复。
#[test]
#[should_panic(expected = "只读挂载做不成")]
fn no_file_version_from_the_read_only_mount_is_not_taken_when_recovery_read_a_file() {
    let image = with_slot_one_zeroed_on_every_device(
        format_pool("refusal-where-recovery-read-a-file").memory_pool_after_mkfs(),
    );
    let refused = claimed_refusal_on(&image, the_refusal_into_the_undecidable_witness());
    let landed = (InstanceGeneration(0), CheckpointTxg(0));
    let report = recovery_report_with(
        RecoveryOutcome::FileRead {
            root: landed,
            content: Vec::new(),
        },
        Some(landed),
    );
    judge_the_refusal(
        &refused,
        &image,
        &report,
        &mut CrashPointReplayObserverCounts::default(),
    );
}

/// 只读挂载打开的不是恢复落到的那一版：不认。
#[test]
#[should_panic(expected = "只读挂载落到的就是恢复落到的那一版")]
fn a_read_only_mount_landing_elsewhere_than_recovery_is_not_taken() {
    let image = with_slot_one_zeroed_on_every_device(
        build_pool("refusal-where-the-read-only-mount-lands-elsewhere").memory_pool(),
    );
    let refused = claimed_refusal_on(&image, the_refusal_into_the_undecidable_witness());
    let landed_elsewhere = (InstanceGeneration(99), CheckpointTxg(99));
    let report = recovery_report_with(
        RecoveryOutcome::FileRead {
            root: landed_elsewhere,
            content: Vec::new(),
        },
        Some(landed_elsewhere),
    );
    judge_the_refusal(
        &refused,
        &image,
        &report,
        &mut CrashPointReplayObserverCounts::default(),
    );
}

/// 被拒的挂载发了一次写（字节与盘上相同，盘面逐字节不变）：照样不认，判的是发没发写。
#[test]
#[should_panic(expected = "被拒的可写挂载发了写")]
fn a_refused_mount_that_issued_a_write_is_not_taken() {
    let image = with_slot_one_zeroed_on_every_device(
        build_pool("refusal-whose-mount-issued-a-write").memory_pool(),
    );
    let mut refused = claimed_refusal_on(&image, the_refusal_into_the_undecidable_witness());
    let slot_zero = slot_zero_bytes_of(&image, DeviceIdentity(0));
    refused.devices[0]
        .1
        .write_at(
            system_configuration_slot_offset(0),
            &slot_zero,
            WriteDurability::Plain,
        )
        .expect("内存盘上写得进");
    let report = recover(&image, JournalPolicy::Consult);
    judge_the_refusal(
        &refused,
        &image,
        &report,
        &mut CrashPointReplayObserverCounts::default(),
    );
}

/// 被拒的挂载之后盘上字节与崩溃镜像不同（录制流里没有那一次写）：不认，判的是盘上逐字节。
#[test]
#[should_panic(expected = "盘上逐字节不变")]
fn disks_differing_from_the_crash_image_after_the_refusal_are_not_taken() {
    let on_the_devices = with_slot_one_zeroed_on_every_device(
        build_pool("refusal-whose-disks-differ-from-the-crash-image").memory_pool(),
    );
    let mut crash_image = on_the_devices.clone();
    crash_image.apply_writes(&[write_on_the_image(
        DeviceIdentity(0),
        DeviceOffsetInBytes(1 << 30),
        WrittenContents::Bytes(vec![1; 512]),
    )]);
    let refused = claimed_refusal_on(&on_the_devices, the_refusal_into_the_undecidable_witness());
    let report = recover(&on_the_devices, JournalPolicy::Consult);
    judge_the_refusal(
        &refused,
        &crash_image,
        &report,
        &mut CrashPointReplayObserverCounts::default(),
    );
}

/// 标准前缀上的可写挂载续做观察者走到这一拒：不 panic，续做跳过（一步都没记成做过），这一态上续做本来还差几步另记一个数。
#[test]
fn the_mount_continuation_observer_skips_the_continuation_on_the_refused_state_and_counts_it() {
    let image = with_slot_one_zeroed_on_every_device(
        build_pool("mount-continuation-observer-on-the-refused-state").memory_pool(),
    );
    let report = recover(&image, JournalPolicy::Consult);
    let inputs = mount_continuation_inputs_of_the_unlink_stream(
        "mount-continuation-observer-on-the-refused-state-inputs",
    );
    let crash_image = CrashImage {
        base: &image,
        writes: &[],
        persisted: Vec::new(),
    };
    let mut counts = CrashPointReplayObserverCounts::default();
    let after = observe_the_mount_continuation(&inputs, &crash_image, &report, &mut counts);
    assert_eq!(
        counts.count_named(&continuation_skipped_on_the_refusal_counter(0)),
        1,
        "第一个文件那一版队列空：续做本来就差 0 步"
    );
    assert_eq!(
        counts.count_named("mount_continuation_steps_0"),
        0,
        "续做跳过，不记成做过 0 步"
    );
    assert_eq!(
        counts.count_named(WRITABLE_MOUNT_REFUSED_INTO_THE_UNDECIDABLE_WITNESS_COUNTER),
        1,
        "认下的这一态记进跳过的态数"
    );
    assert!(
        after.disks_after_the_observer == image,
        "交回的是没改过的盘"
    );
}
