//! 辩方腿 m3-close-b19-code-r2-sonnet 的探针（U2）：补写盘（盘 1 两槽整槽写 0）上，预演的读坏账里有盘 1 两槽的读坏，
//! 叠上别的读故障时挂载怎么收尾。打印 `PROBE name=<名> result=<一行>`。

#![allow(dead_code, reason = "几份 common 模块各只用到一部分")]

mod common;
mod common_admission;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_reads_shared_by_the_rehearsal_and_the_real_run;

use common::PoolImageGeometry;
use common_reads_shared_by_the_rehearsal_and_the_real_run::{
    mount_writable_on_recording_devices, mount_writable_without_faults, narrow_image,
    ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy,
    FaultedReads, ReadFault, BYTES_OF_A_TREE_TABLE,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::mount::MountError;
use singlefs_core::transaction::PublishError;
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);

fn zero_both_system_configuration_slots(image: &MemoryPool, geometry: &PoolImageGeometry) -> MemoryPool {
    let spacing = u64::from(geometry.parameters.geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut zeroed = image.clone();
    let device = zeroed.devices.get_mut(&DEVICE_ONE).expect("盘 1");
    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        device.write(DeviceOffsetInBytes(slot_index * spacing), &vec![0u8; slot_bytes]);
    }
    zeroed
}

fn cause_text(result: &Result<singlefs_core::mount::Mounted, MountError>) -> String {
    match result {
        Ok(mounted) => format!(
            "MountMade devices_rewritten={:?}",
            mounted.output.devices_rewritten_by_the_acquisition
        ),
        Err(MountError::Publish(failed)) => match &failed.cause {
            PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(read) => format!(
                "ReadBackFailingSelfVerification device={:?} offset={:?} length={}",
                read.device, read.offset, read.length_in_bytes
            ),
            PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(read) => {
                format!("ReadFailed device={:?} offset={:?} length={}", read.device, read.offset, read.length_in_bytes)
            }
            PublishError::NextContainerNumberPastTheTopOfItsRange(_) => "C841".to_string(),
            other => format!("{other:?}").chars().take(160).collect(),
        },
        Err(other) => format!("{other:?}").chars().take(200).collect(),
    }
}

/// 盘 1 两槽整槽写 0，只进内存那一趟头一次读的树表两份各读回坏字节一次：重走一趟读到好的，走到底，挂载做成。
#[test]
fn u2_zeroed_slots_and_a_tree_table_read_back_garbage_once_is_made() {
    let (image, geometry, _) = narrow_image();
    let zeroed = zero_both_system_configuration_slots(&image, &geometry);
    let fault_free = mount_writable_without_faults(&zeroed, &geometry);
    println!("PROBE name=zeroed_fault_free result={}", cause_text(&fault_free.result));
    let (window, offset_of_the_copies) =
        ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(
            &fault_free.reads_reaching_the_disk,
        );
    let faulted = mount_writable_on_recording_devices(
        &zeroed,
        &geometry,
        ReadFault::ReadsBackCorruptedBytes,
        FaultedReads::AtTheOrdinals(window),
    );
    println!("PROBE name=zeroed_garbage_once result={}", cause_text(&faulted.result));
    assert!(faulted.result.is_ok());
    let _ = offset_of_the_copies;
}

/// 盘 1 两槽整槽写 0，同一份树表每一次读都读回坏字节：重走到上限，在取号之前拒；看交的是哪一次读坏。
#[test]
fn u2_zeroed_slots_and_a_tree_table_reading_back_garbage_every_time_names_which_read() {
    let (image, geometry, _) = narrow_image();
    let zeroed = zero_both_system_configuration_slots(&image, &geometry);
    let fault_free = mount_writable_without_faults(&zeroed, &geometry);
    let (window, offset_of_the_copies) =
        ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(
            &fault_free.reads_reaching_the_disk,
        );
    let faulted = mount_writable_on_recording_devices(
        &zeroed,
        &geometry,
        ReadFault::ReadsBackCorruptedBytes,
        FaultedReads::EveryReadOfTheRangeFromTheOrdinal {
            from_ordinal: *window.first().expect("窗口两次读"),
            offset: offset_of_the_copies,
            length_in_bytes: BYTES_OF_A_TREE_TABLE,
        },
    );
    println!("PROBE name=zeroed_garbage_every_time result={}", cause_text(&faulted.result));
    println!("PROBE name=zeroed_garbage_every_time_tree_table_offset result={offset_of_the_copies:?}");

    let (plain_fault_free, plain_geometry) = (mount_writable_without_faults(&image, &geometry), &geometry);
    let (plain_window, plain_offset) =
        ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(
            &plain_fault_free.reads_reaching_the_disk,
        );
    let plain = mount_writable_on_recording_devices(
        &image,
        plain_geometry,
        ReadFault::ReadsBackCorruptedBytes,
        FaultedReads::EveryReadOfTheRangeFromTheOrdinal {
            from_ordinal: *plain_window.first().expect("窗口两次读"),
            offset: plain_offset,
            length_in_bytes: BYTES_OF_A_TREE_TABLE,
        },
    );
    println!("PROBE name=plain_garbage_every_time result={}", cause_text(&plain.result));
    assert!(faulted.result.is_err());
}
