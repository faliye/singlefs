//! 可写挂载预演与真做共用的那一份里，读成而读的一方核出自证不过的字节（读坏）记账、不留（岔路单 `research/prompts/m3-b44-code-r3-forks.md`
//! 岔路一「两样都做」那一行，用户定）：读的一方核自证、核校验和不过时报一声，共用的那一层记进读坏账、丢掉那几块，下一次读穿到盘上；
//! 取号之前预演没走到续做的结局时照读错账与读坏账两本判，没走完就从头重走、重走到上限仍是这样才在取号之前拒（岔路二「没走完就重走」那一行）。
//! 代码轮 `research/prompts/m3-b44-code-r3-main-verification.md` 第二节 L1 ① 与 L2 那两形：攻方原型
//! `research/prompts/m3-b44-code-r3-opus-model/attack_shared_reads_probe.rs` 的 `w:303:2:garbage`（盘 0 两个系统配置槽头一次读进共用的那一份时读回坏字节）
//! 与 `p:<偏移>:306:1:garbage`（只进内存那一趟头一次读的一份树表两份都读回坏字节）。
//! 镜像是 `common_reads_shared_by_the_rehearsal_and_the_real_run` 的窄盘两份（没改坏的、退役树那片号在 u64::MAX 的）。

#![allow(dead_code, reason = "几份 common 模块各只用到一部分")]

mod common;
mod common_admission;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_reads_shared_by_the_rehearsal_and_the_real_run;

use std::collections::BTreeSet;

use common::PoolImageGeometry;
use common_reads_shared_by_the_rehearsal_and_the_real_run::{
    assert_the_faulted_reads_are_both_copies_of_one_unit,
    assert_the_mount_is_made_and_the_continuation_walks_to_empty,
    cause_of_the_refusal_before_acquisition, images_of, mount_writable_on_recording_devices,
    mount_writable_without_faults, narrow_image,
    narrow_image_with_the_retirement_container_at_the_top_of_its_range,
    ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy,
    FaultedReads, ReadFault, WritableMountOnRecordingDevices, BYTES_OF_A_TREE_TABLE,
    DEVICE_OF_THE_COPY_READ_FIRST, READ_OF_SLOT_ONE_OF_DEVICE_ZERO_INTO_THE_SHARED_READS,
    READ_OF_SLOT_ZERO_OF_DEVICE_ZERO_INTO_THE_SHARED_READS,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::transaction::PublishError;
use singlefs_harness::memory_pool::MemoryPool;

/// 整池第 307、308 次读（盘 0 两个系统配置槽头一次读进共用的那一份）读回坏字节各一次：先核不带故障那一次这两次读的确是它们。
fn mount_with_both_slots_of_device_zero_read_back_garbage_once_into_the_shared_reads(
    image: &MemoryPool,
    geometry: &PoolImageGeometry,
    fault_free: &WritableMountOnRecordingDevices,
) -> WritableMountOnRecordingDevices {
    let spacing = u64::from(geometry.parameters.geometry.fixed_structure_slot_spacing);
    let ordinals = [
        READ_OF_SLOT_ZERO_OF_DEVICE_ZERO_INTO_THE_SHARED_READS,
        READ_OF_SLOT_ONE_OF_DEVICE_ZERO_INTO_THE_SHARED_READS,
    ];
    let slots_read_at_the_ordinals: Vec<(DeviceIdentity, DeviceOffsetInBytes, bool)> = ordinals
        .iter()
        .map(|ordinal| {
            let read = fault_free.reads_reaching_the_disk
                [usize::try_from(ordinal - 1).expect("读序数装得进 usize")];
            (
                read.device,
                read.offset,
                read.reads_a_system_configuration_slot(geometry)
                    && read.is_before_the_first_write_of_the_mount(),
            )
        })
        .collect();
    assert_eq!(
        slots_read_at_the_ordinals,
        vec![
            (DeviceIdentity(0), DeviceOffsetInBytes(0), true),
            (DeviceIdentity(0), DeviceOffsetInBytes(spacing), true),
        ],
        "前提：整池第 307、308 次读是第一个写之前读盘 0 两个系统配置槽"
    );
    let faulted = mount_writable_on_recording_devices(
        image,
        geometry,
        ReadFault::ReadsBackCorruptedBytes,
        FaultedReads::AtTheOrdinals(ordinals.into_iter().collect::<BTreeSet<u64>>()),
    );
    assert_eq!(
        faulted
            .reads_that_carried_the_fault
            .iter()
            .map(|read| (read.device, read.offset))
            .collect::<Vec<_>>(),
        vec![
            (DeviceIdentity(0), DeviceOffsetInBytes(0)),
            (DeviceIdentity(0), DeviceOffsetInBytes(spacing)),
        ],
        "盘 0 两槽各恰读回坏字节一次"
    );
    faulted
}

/// 场景：改坏的窄盘镜像（续做最后一批报 C841）上可写挂载，盘 0 两个系统配置槽头一次读进共用的那一份时（判号那一读）读回坏字节各一次，盘上是好的。
/// 预期：判号那一读核出两槽自证不过、从共用的那一份里丢掉；只进内存那一趟取号再读它们穿到盘上、读到好的，续做走到最后一批交 C841，
/// 取号之前拒（`NextContainerNumberPastTheTopOfItsRange`、三份写账全空），两块盘逐字节不变。
/// 判别力：坏字节留在共用的那一份（`crates/mutations.tsv`「4-4 修红 W17 读坏：坏字节留在共用的那一份」）时只进内存那一趟取号读到的仍是坏字节、
/// 盘 0 一份自证过的都没有，那一趟停在取号，重走一趟照样停、到上限交读坏那一拒，不是 C841。
#[test]
fn both_system_configuration_slots_of_device_zero_read_back_garbage_once_into_the_shared_reads_still_refuse_the_corrupted_image_on_the_container_number_before_acquisition(
) {
    let (image, geometry) = narrow_image_with_the_retirement_container_at_the_top_of_its_range();
    let fault_free = mount_writable_without_faults(&image, &geometry);
    assert!(
        matches!(
            cause_of_the_refusal_before_acquisition(&fault_free.result),
            PublishError::NextContainerNumberPastTheTopOfItsRange(_)
        ),
        "前提：不带故障时改坏的那一份在取号之前交 C841"
    );
    let faulted = mount_with_both_slots_of_device_zero_read_back_garbage_once_into_the_shared_reads(
        &image,
        &geometry,
        &fault_free,
    );
    let cause = cause_of_the_refusal_before_acquisition(&faulted.result);
    assert!(
        matches!(cause, PublishError::NextContainerNumberPastTheTopOfItsRange(_)),
        "两槽读坏的字节不留在共用的那一份：预演取号读到好的、走到 C841，在取号之前交它；交的是：{cause:?}"
    );
    assert!(
        faulted.images_after == images_of(&image),
        "取号之前拒：两块盘逐字节不变"
    );
}

/// 场景：没改坏的窄盘镜像上，盘 0 两个系统配置槽头一次读进共用的那一份时读回坏字节各一次（同上一条的窗口），盘上是好的。
/// 预期：挂载做成、续做 16 步走到空，两块盘与不带故障那一次挂载之后逐字节相同。
/// 判别力：坏字节留在共用的那一份（「4-4 修红 W17 读坏：坏字节留在共用的那一份」）时只进内存那一趟停在取号、两趟都是这样，在取号之前交读坏那一拒。
#[test]
fn both_system_configuration_slots_of_device_zero_read_back_garbage_once_into_the_shared_reads_leave_the_unchanged_image_mounted_as_without_the_fault(
) {
    let (image, geometry, _) = narrow_image();
    let fault_free = mount_writable_without_faults(&image, &geometry);
    assert_the_mount_is_made_and_the_continuation_walks_to_empty(
        &fault_free.result,
        "前提：不带故障时没改坏的那一份",
    );
    let faulted = mount_with_both_slots_of_device_zero_read_back_garbage_once_into_the_shared_reads(
        &image,
        &geometry,
        &fault_free,
    );
    assert_the_mount_is_made_and_the_continuation_walks_to_empty(
        &faulted.result,
        "盘 0 两槽各一次读回坏字节、盘上是好的",
    );
    assert!(
        faulted.images_after == fault_free.images_after,
        "两块盘与不带故障那一次挂载之后逐字节相同"
    );
}

/// 场景：没改坏的窄盘镜像上，只进内存那一趟头一次读的一份树表（续做推的抬 F 算上限读一条有效根的树表，取法见
/// `ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy`）两份各读回坏字节一次，盘上是好的
/// （判决 L2 那一形；攻方量的落盘之后交 `FloorRaiseFailedAfterTheMountsPublishes`、之后再挂取号之前拒）。
/// 预期：那一趟读坏两份、记进读坏账、丢掉，抬 F 读不出、那一趟没走到续做的结局；从头重走一趟，那份树表穿到盘上读到好的，走到底；
/// 挂载做成、续做走到空，两块盘与不带故障那一次挂载之后逐字节相同。
/// 判别力：坏字节留在共用的那一份（「4-4 修红 W17 读坏：坏字节留在共用的那一份」）时重走那一趟取的仍是记下的坏字节、又停在抬 F，
/// 到上限在取号之前交读坏那一拒，挂载没做成。
#[test]
fn both_copies_of_a_tree_table_read_back_garbage_once_by_the_in_memory_rehearsal_are_read_from_the_disk_again_by_the_next_rehearsal_and_the_mount_is_made(
) {
    let (image, geometry, _) = narrow_image();
    let fault_free = mount_writable_without_faults(&image, &geometry);
    assert_the_mount_is_made_and_the_continuation_walks_to_empty(
        &fault_free.result,
        "前提：不带故障时没改坏的那一份",
    );
    let (window, offset_of_the_copies) =
        ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(
            &fault_free.reads_reaching_the_disk,
        );
    let faulted = mount_writable_on_recording_devices(
        &image,
        &geometry,
        ReadFault::ReadsBackCorruptedBytes,
        FaultedReads::AtTheOrdinals(window),
    );
    assert_the_faulted_reads_are_both_copies_of_one_unit(&faulted, offset_of_the_copies);
    assert_the_mount_is_made_and_the_continuation_walks_to_empty(
        &faulted.result,
        "那份树表两份各一次读回坏字节、盘上是好的",
    );
    assert!(
        faulted.images_after == fault_free.images_after,
        "两块盘与不带故障那一次挂载之后逐字节相同"
    );
}

/// 场景：没改坏的窄盘镜像上，同一份树表从只进内存那一趟头一次读它起、两块盘上每一次读都读回坏字节（读坏不是一次就过去的）。
/// 预期：每一趟预演都读坏两份、停在抬 F，重走到上限仍是这样：在取号之前拒，交最后那一趟最先的那一次读坏（盘 0 那一份），
/// 三份写账全空、两块盘逐字节不变；之后不带故障再挂一次，挂载做成、续做走到空。
/// 判别力：读坏不进账（「4-4 修红 W17 读坏：读坏不进账」）时取号之前那一判两本账都空、照旧留给续做，真做取号、写行、暖机落盘之后
/// 抬 F 读那份树表仍读坏，挂载报的不是取号之前那一拒、盘也变了。
#[test]
fn both_copies_of_a_tree_table_reading_back_garbage_on_every_read_refuse_the_mount_before_acquisition_on_the_read_failing_self_verification_then_a_plain_remount_is_made(
) {
    let (image, geometry, _) = narrow_image();
    let fault_free = mount_writable_without_faults(&image, &geometry);
    let (window, offset_of_the_copies) =
        ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(
            &fault_free.reads_reaching_the_disk,
        );
    let from_ordinal = *window.first().expect("窗口有两次读");
    let faulted = mount_writable_on_recording_devices(
        &image,
        &geometry,
        ReadFault::ReadsBackCorruptedBytes,
        FaultedReads::EveryReadOfTheRangeFromTheOrdinal {
            from_ordinal,
            offset: offset_of_the_copies,
            length_in_bytes: BYTES_OF_A_TREE_TABLE,
        },
    );
    let devices_read_back_corrupted: BTreeSet<DeviceIdentity> = faulted
        .reads_that_carried_the_fault
        .iter()
        .map(|read| read.device)
        .collect();
    assert!(
        devices_read_back_corrupted
            == [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .collect::<BTreeSet<DeviceIdentity>>()
            && faulted.reads_that_carried_the_fault.len() >= 4,
        "两块盘上那一份每一趟都读回坏字节（至少两趟各两份）：{:?}",
        faulted.reads_that_carried_the_fault
    );
    let cause = cause_of_the_refusal_before_acquisition(&faulted.result);
    let PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(
        read,
    ) = cause
    else {
        panic!("每一趟预演都读坏那份树表：重走到上限在取号之前交读坏那一拒，交的是：{cause:?}");
    };
    assert_eq!(
        (read.device, read.offset, read.length_in_bytes),
        (
            DEVICE_OF_THE_COPY_READ_FIRST,
            offset_of_the_copies,
            BYTES_OF_A_TREE_TABLE
        ),
        "交的是最后那一趟最先的那一次读坏：盘 0 那一份树表"
    );
    assert!(
        faulted.images_after == images_of(&image),
        "取号之前拒：两块盘逐字节不变"
    );
    let remount = mount_writable_without_faults(&image, &geometry);
    assert_the_mount_is_made_and_the_continuation_walks_to_empty(
        &remount.result,
        "不带故障再挂一次",
    );
}
