//! 正推腿 m3-close-b19-code-r3-sonnet 的探针（V1，剔少）：窄盘镜像上只把一块盘的一个系统配置槽整槽写 0（撕裂的单槽，单故障承诺之内，
//! 另一槽自证过，这块盘不是补写盘），取号之前只进内存那一趟照样读这一槽、读者报读坏；叠上树表每一次读都读坏时，看拒因点名谁。
//! 对照：盘 1 两槽整槽写 0（补写盘，第二轮修补剔掉它的读坏）。打印 `PROBE name=<名> result=<一行>`。

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
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::memory_pool::MemoryPool;

fn zero_system_configuration_slots(
    image: &MemoryPool,
    geometry: &PoolImageGeometry,
    device: DeviceIdentity,
    slot_indices: &[u64],
) -> MemoryPool {
    let spacing = u64::from(geometry.parameters.geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut zeroed = image.clone();
    let device_image = zeroed.devices.get_mut(&device).expect("池里有这块盘");
    for slot_index in slot_indices {
        device_image.write(DeviceOffsetInBytes(slot_index * spacing), &vec![0u8; slot_bytes]);
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

/// 一份镜像上：不带读故障的可写挂载（前提）与树表每一次读都读回坏字节的可写挂载，各打一行。
fn probe_one_image(name: &str, image: &MemoryPool, geometry: &PoolImageGeometry) -> String {
    let fault_free = mount_writable_without_faults(image, geometry);
    println!("PROBE name={name}_fault_free result={}", cause_text(&fault_free.result));
    let (window, offset_of_the_copies) =
        ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(
            &fault_free.reads_reaching_the_disk,
        );
    let faulted = mount_writable_on_recording_devices(
        image,
        geometry,
        ReadFault::ReadsBackCorruptedBytes,
        FaultedReads::EveryReadOfTheRangeFromTheOrdinal {
            from_ordinal: *window.first().expect("窗口两次读"),
            offset: offset_of_the_copies,
            length_in_bytes: BYTES_OF_A_TREE_TABLE,
        },
    );
    let text = cause_text(&faulted.result);
    println!(
        "PROBE name={name}_tree_table_garbage_every_time result={text} writes_reaching_the_disk={}",
        faulted.writes_reaching_the_disk.len()
    );
    println!("PROBE name={name}_tree_table_offset result={offset_of_the_copies:?}");
    text
}

#[test]
fn v1_torn_single_slots_of_a_device_that_is_not_rewritten_against_the_control_of_a_rewritten_device() {
    let (image, geometry, _) = narrow_image();
    let control = zero_system_configuration_slots(&image, &geometry, DeviceIdentity(1), &[0, 1]);
    let control_text = probe_one_image("control_device_one_both_slots_zeroed", &control, &geometry);
    let mut texts = Vec::new();
    for (name, device, slots) in [
        ("device_one_slot_zero_zeroed", DeviceIdentity(1), [0u64]),
        ("device_one_slot_one_zeroed", DeviceIdentity(1), [1u64]),
        ("device_zero_slot_zero_zeroed", DeviceIdentity(0), [0u64]),
        ("device_zero_slot_one_zeroed", DeviceIdentity(0), [1u64]),
    ] {
        let torn = zero_system_configuration_slots(&image, &geometry, device, &slots);
        texts.push((name, probe_one_image(name, &torn, &geometry)));
    }
    println!("PROBE name=summary control={control_text}");
    for (name, text) in &texts {
        println!("PROBE name=summary {name}={text}");
    }
}
