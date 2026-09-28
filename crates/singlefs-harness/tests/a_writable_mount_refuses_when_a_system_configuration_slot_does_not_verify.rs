//! C331（择根倒挂压过已确认的写） 取甲（用户 2026-09-28 定，被攻过两轮；`research/prompts/unreadable-at-mount-r2-main-verification.md` L2）：
//! 可写挂载判 N-配置 的见证读里，只要有一个系统配置槽读不出或自证不过，就判不出、按判据为真：重读一次，仍缺那一槽就在取号之前拒可写，
//! 盘上逐字节不变；只读挂载照常（`mounted_read::mount_read_only`）。改之前读不出的槽不进 max、全读不出取 0 判「没见证」，
//! 新实例会从较旧的根可写挂载，确认过的写之后被较旧实例的根压过（第一轮判决 V1、V2）。
//! 这一形的代价（撕裂的系统配置轮换之后只剩只读挂载）是定案的一部分，见同一判决 L1。
mod common;

use common::{build_pool, parameters, BuiltPool};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    mount_writable, MountError, StillUnreadableAfterOneReread, WitnessedCounterComparison,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};

/// 新池新建文件之后每块盘两槽：世代 4、世代 5（新池新建文件那次轮换）。把 `damaged_device` 世代 5 那一槽清零（撕裂那一形），另一槽照样自证得过。
fn pool_with_one_system_configuration_slot_zeroed(
    tag: &str,
    damaged_device: DeviceIdentity,
) -> BuiltPool {
    let mut pool = build_pool(tag);
    let slot_spacing_in_bytes = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let devices = pool.devices.as_mut().expect("新池新建文件写完，盘还开着");
    let (_, device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == damaged_device)
        .expect("池里有这块盘");
    device
        .write_at(
            DeviceOffsetInBytes((5 % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * slot_spacing_in_bytes),
            &vec![0u8; usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")],
            WriteDurability::Plain,
        )
        .expect("世代 5 那一槽清零");
    pool
}

fn a_writable_mount_refuses_before_acquisition_when_one_system_configuration_slot_does_not_verify(
    tag: &str,
    damaged_device: DeviceIdentity,
) {
    let mut pool = pool_with_one_system_configuration_slot_zeroed(tag, damaged_device);
    let image_before = pool.memory_pool();
    let mut devices = pool.reopen_recorded();
    let refusal = mount_writable(&parameters(), &mut devices)
        .expect_err("有一槽系统配置自证不过：判不出，重读一次仍缺，拒可写");
    pool.devices = Some(devices);
    let MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) = &refusal else {
        panic!("拒因是「重读一次仍读不出更新的状态」：{refusal:?}");
    };
    let StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
        first_read,
        reread,
    } = still_unreadable.as_ref()
    else {
        panic!("拒在判据 N-配置 那一格：{still_unreadable:?}");
    };
    for reading in [first_read, reread] {
        assert_eq!(
            reading.witness.comparison,
            WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified,
            "两遍读都判「有一槽系统配置读不出或自证不过」：{reading:?}"
        );
    }
    assert!(
        pool.memory_pool() == image_before,
        "拒可写在取号之前：盘上逐字节不变"
    );
    mount_read_only(&pool.memory_pool()).expect("只读挂载照常");
}

#[test]
fn a_writable_mount_refuses_before_acquisition_when_the_first_devices_newest_system_configuration_slot_does_not_verify(
) {
    a_writable_mount_refuses_before_acquisition_when_one_system_configuration_slot_does_not_verify(
        "c331-jia-mount-device-zero",
        DeviceIdentity(0),
    );
}

#[test]
fn a_writable_mount_refuses_before_acquisition_when_the_second_devices_newest_system_configuration_slot_does_not_verify(
) {
    a_writable_mount_refuses_before_acquisition_when_one_system_configuration_slot_does_not_verify(
        "c331-jia-mount-device-one",
        DeviceIdentity(1),
    );
}

/// 对照：两槽都自证得过的同一个池照常可写挂载（判据 N-配置 为假，不重读）。
#[test]
fn a_writable_mount_with_every_system_configuration_slot_verified_goes_ahead() {
    let mut pool = build_pool("c331-jia-mount-control");
    let mut devices = pool.reopen_recorded();
    mount_writable(&parameters(), &mut devices).expect("两槽都自证得过：照常可写挂载");
    pool.devices = Some(devices);
}
