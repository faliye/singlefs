//! 攻方探针（m3-close-b19-code-r1 T4，副本，不进仓）：窄盘镜像（`common_reads_shared_by_the_rehearsal_and_the_real_run::narrow_image`：
//! 续做要推抬 F，取号之前在只进内存的盘上把取号连同续做预演一遍）上把盘 1 两个系统配置槽整槽写 0（两槽全废，丙' 补写那一形）。
//! 预演那一趟按空的补写盘表走（`mount.rs` `PublishesPlannedAfterAcquisition` 字段注释「预演那一趟按空表走」），只进内存那一趟取号读盘 1 两槽、
//! 读者报读坏、取号交「一份自证过的都没有」、那一趟停在续做之前；推的结局：重走到上限，在取号之前交读坏那一拒，走不到逐盘核与补写。
//! 另两格：同一份镜像关掉空间准入（看拒是不是空间准入式子带出来的）；先挂一次让续做走到空、再写 0 两槽挂第二次（不走只进内存那一趟）。

#![allow(dead_code, reason = "几份 common 模块各只用到一部分")]

mod common;
mod common_admission;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_reads_shared_by_the_rehearsal_and_the_real_run;

use common::PoolImageGeometry;
use common_admission::plain_devices_on;
use common_reads_shared_by_the_rehearsal_and_the_real_run::{
    images_of, mount_writable_without_faults, narrow_image, switches,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, MountError, ShadowLedger,
};
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

const DEVICE_LOSING_BOTH_SLOTS: DeviceIdentity = DeviceIdentity(1);

fn with_both_system_configuration_slots_zeroed(
    image: &MemoryPool,
    geometry: &PoolImageGeometry,
    device: DeviceIdentity,
) -> MemoryPool {
    let mut zeroed = image.clone();
    let spacing = u64::from(geometry.parameters.geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let device_image = zeroed.devices.get_mut(&device).expect("池里有这块盘");
    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        device_image.write(
            DeviceOffsetInBytes(slot_index * spacing),
            &vec![0u8; slot_bytes],
        );
    }
    zeroed
}

fn describe(result: &Result<singlefs_core::mount::Mounted, MountError>) -> String {
    match result {
        Ok(mounted) => format!(
            "mounted instance={:?} devices_rewritten_by_the_acquisition={:?} continuation_steps={}",
            mounted.output.instance,
            mounted.output.devices_rewritten_by_the_acquisition,
            mounted.output.deletion_queue_continuation.len()
        ),
        Err(MountError::Publish(failed)) => format!(
            "refused Publish cause={:?} persisted={} failed={}",
            failed.cause,
            failed.writes_of_persisted_publishes.len(),
            failed.writes_of_failed_publishes.len()
        ),
        Err(other) => format!("refused {other:?}"),
    }
}

#[test]
fn attack_probe_narrow_image_with_a_device_without_any_self_verified_system_configuration() {
    let (image, geometry, _) = narrow_image();
    let unchanged = mount_writable_without_faults(&image, &geometry);
    println!("PROBE unchanged_image {}", describe(&unchanged.result));
    let zeroed = with_both_system_configuration_slots_zeroed(&image, &geometry, DEVICE_LOSING_BOTH_SLOTS);
    let judged = mount_writable_without_faults(&zeroed, &geometry);
    println!("PROBE zeroed_image_space_admission_judged {}", describe(&judged.result));
    println!(
        "PROBE zeroed_image_space_admission_judged devices_unchanged={}",
        judged.images_after == images_of(&zeroed)
    );
    let mut devices = plain_devices_on(&zeroed);
    let skipped = mount_writable_with_test_only_switches_and_deletion_switches(
        &geometry.parameters,
        &mut devices,
        SpaceAdmission::SkippedByTheTestOnlySwitch,
        ShadowLedger::On,
        switches(),
    );
    println!("PROBE zeroed_image_space_admission_skipped {}", describe(&skipped));
    // 对照：先在没改坏的镜像上挂一次（续做走到空、删除队列清空，挂着不卸），再把盘 1 两槽写 0 挂第二次：续做不缺空间，不走只进内存那一趟。
    let mut after_the_continuation = image.clone();
    after_the_continuation.devices = unchanged.images_after.clone();
    let zeroed_after_the_continuation = with_both_system_configuration_slots_zeroed(
        &after_the_continuation,
        &geometry,
        DEVICE_LOSING_BOTH_SLOTS,
    );
    let second = mount_writable_without_faults(&zeroed_after_the_continuation, &geometry);
    println!("PROBE zeroed_after_the_continuation {}", describe(&second.result));
}
