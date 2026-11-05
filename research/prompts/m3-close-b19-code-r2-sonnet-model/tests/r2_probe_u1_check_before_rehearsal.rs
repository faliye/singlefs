//! 辩方腿 m3-close-b19-code-r2-sonnet 的探针（U1）：逐盘核挪到续做预演之前之后，两样同时成立时报哪一个。
//! 每条探针打印 `PROBE name=<名> result=<一行>`，断言写的是新次序（逐盘核在前）下的结局，旧次序下会红。

#![allow(dead_code, reason = "几份 common 模块各只用到一部分")]

mod common;
mod common_admission;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_reads_shared_by_the_rehearsal_and_the_real_run;

use common::PoolImageGeometry;
use common_reads_shared_by_the_rehearsal_and_the_real_run::{
    mount_writable_without_faults, narrow_image,
    narrow_image_with_the_retirement_container_at_the_top_of_its_range,
    WritableMountOnRecordingDevices,
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

fn blank_the_whole_device(image: &MemoryPool) -> MemoryPool {
    let mut blank = image.clone();
    let size = blank.device_size_in_bytes;
    blank
        .devices
        .get_mut(&DEVICE_ONE)
        .expect("盘 1")
        .zero_fill(DeviceOffsetInBytes(0), size);
    blank
}

fn describe(mounted: &WritableMountOnRecordingDevices) -> String {
    match &mounted.result {
        Ok(mounted) => format!(
            "MountMade devices_rewritten={:?}",
            mounted.output.devices_rewritten_by_the_acquisition
        ),
        Err(MountError::Publish(failed)) => format!(
            "Publish cause={} writes_before_any_publish_empty={}",
            match &failed.cause {
                PublishError::NextContainerNumberPastTheTopOfItsRange(_) => "C841".to_string(),
                other => format!("{other:?}").chars().take(160).collect::<String>(),
            },
            failed.writes_of_persisted_publishes.is_empty() && failed.writes_of_failed_publishes.is_empty()
        ),
        Err(other) => format!("{other:?}").chars().take(240).collect::<String>(),
    }
}

fn is_c841(mounted: &WritableMountOnRecordingDevices) -> bool {
    matches!(
        &mounted.result,
        Err(MountError::Publish(failed))
            if matches!(failed.cause, PublishError::NextContainerNumberPastTheTopOfItsRange(_))
    )
}

#[test]
fn u1_c841_image_unmodified_returns_c841() {
    let (image, geometry) = narrow_image_with_the_retirement_container_at_the_top_of_its_range();
    let mounted = mount_writable_without_faults(&image, &geometry);
    println!("PROBE name=c841_unmodified result={}", describe(&mounted));
    assert!(is_c841(&mounted));
}

#[test]
fn u1_c841_image_with_both_slots_of_device_one_zeroed_returns_c841_when_the_check_comes_first() {
    let (image, geometry) = narrow_image_with_the_retirement_container_at_the_top_of_its_range();
    let zeroed = zero_both_system_configuration_slots(&image, &geometry);
    let mounted = mount_writable_without_faults(&zeroed, &geometry);
    println!("PROBE name=c841_with_zeroed_slots result={}", describe(&mounted));
    assert!(is_c841(&mounted), "新次序：逐盘核先把盘 1 交进补写盘表，预演走到续做最后一批交 C841");
}

#[test]
fn u1_c841_image_with_device_one_blank_reports_the_check_not_c841_when_the_check_comes_first() {
    let (image, geometry) = narrow_image_with_the_retirement_container_at_the_top_of_its_range();
    let blank = blank_the_whole_device(&image);
    let mounted_on_the_c841_image = mount_writable_without_faults(&blank, &geometry);
    println!("PROBE name=c841_with_blank_device_one result={}", describe(&mounted_on_the_c841_image));

    let (plain_image, plain_geometry, _) = narrow_image();
    let plain_blank = blank_the_whole_device(&plain_image);
    let mounted_on_the_plain_image = mount_writable_without_faults(&plain_blank, &plain_geometry);
    println!("PROBE name=plain_with_blank_device_one result={}", describe(&mounted_on_the_plain_image));
    assert!(mounted_on_the_plain_image.result.is_err(), "前提：空盘 1 的窄盘镜像可写挂载被拒");
    assert!(
        !is_c841(&mounted_on_the_c841_image),
        "新次序：两样同时成立时报逐盘核那一拒，不报 C841"
    );
}
