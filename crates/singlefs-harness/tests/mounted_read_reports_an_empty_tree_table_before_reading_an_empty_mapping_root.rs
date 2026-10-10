//! C695 甲（用户 2026-10-07 定「先修次序，选哪一版归 C683」；判决 `research/prompts/m3-c694-r1-main-verification.md` 第二节 K3 那一行）：
//! 打开挂载态时根记录的中央映射树根是空指针，先判树表 0 条、报 `OpenPoolForReadFailure::TreeTableHasNoEntries`，不先读空映射根
//! （改之前读那条空指针的位置条目，两盘都在也报 `Walk(UnitUnreadable { slot: SlotNumber(0) })`）。
//! 盘面：384 槽小盘 mkfs，可写挂载两次（第一次取号 1、暖机；第二次取号 2、在树表 0 条的一版上写行、暖机），一个文件都不写：
//! 环里读得出的根是 mkfs 的第 0 代根、暖机根与写行根，树表都是 0 条、映射根都是空指针。
mod common_admission_pool;

use common_admission_pool::{
    device_bytes_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices,
};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::{
    mount_read_only, open_pool_for_read, MountReadOnlyFailure, OpenPoolForReadFailure,
};
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{choose_system_configuration, readable_roots};
use singlefs_harness::memory_pool::SparseBlockDevice;

/// mkfs 之后可写挂载两次、一个文件都不写的两块 384 槽小盘。
fn devices_of_a_pool_without_a_file_after_two_writable_mounts(
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
    make_filesystem(&parameters, &mut devices).expect("小盘上 mkfs 做得成");
    mount_writable(&parameters, &mut devices).expect("mkfs 之后第一次可写挂载");
    mount_writable(&parameters, &mut devices).expect("树表 0 条的一版上第二次可写挂载（写行）");
    devices
}

/// 环里每一条读得出的根（mkfs 的第 0 代、暖机、写行）打开挂载态都报树表 0 条；只读挂载同样报它。
/// 判别力：先判树表 0 条那一步拿掉（变异「C695 先读空映射根」）时报的是 `Walk(UnitUnreadable { slot: SlotNumber(0) })`，这一条红。
#[test]
fn every_root_of_a_pool_without_a_file_opens_to_an_empty_tree_table_instead_of_an_unreadable_slot_zero(
) {
    let devices = devices_of_a_pool_without_a_file_after_two_writable_mounts();
    let system_configuration = choose_system_configuration(&devices).expect("择系统配置");
    let roots = readable_roots(
        &devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let instances: Vec<u32> = roots.iter().map(|root| root.instance.0).collect();
    assert!(
        instances.contains(&0) && instances.contains(&1) && instances.contains(&2),
        "环里有 mkfs 的第 0 代根、实例 1 的暖机根与实例 2 的写行根：{instances:?}"
    );
    for root in &roots {
        assert_eq!(
            root.mapping_root,
            NodePointer::empty_root(),
            "树表 0 条的根（实例 {:?} txg {:?}）映射根是空指针",
            root.instance,
            root.checkpoint_txg
        );
        assert_eq!(
            open_pool_for_read(&devices, root).err(),
            Some(OpenPoolForReadFailure::TreeTableHasNoEntries),
            "实例 {:?} txg {:?} 那条根两盘都在：报树表 0 条，不报槽 0 读不出",
            root.instance,
            root.checkpoint_txg
        );
    }
    assert_eq!(
        mount_read_only(&devices).err(),
        Some(MountReadOnlyFailure::Open(
            OpenPoolForReadFailure::TreeTableHasNoEntries
        )),
        "只读挂载择到的那条根同样报树表 0 条（树表 0 条不是走读失败，F2 乙不退）"
    );
}
