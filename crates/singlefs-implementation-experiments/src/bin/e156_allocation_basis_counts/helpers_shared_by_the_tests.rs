//! 几份单测共用的帮手。

use super::{parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_verification::memory_pool::SparseBlockDevice;

pub(super) fn new_pool_file_creation_state() -> (
    PoolAllocator,
    TransactionOutput,
    Vec<(DeviceIdentity, SparseBlockDevice)>,
) {
    let parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(
        devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
    );
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    // mkfs 同一个进程里接着写的会话装上 mkfs 刚写下的根环（C551：没装的分配器记根就断言）。用它的单测最多记到 txg 15，
    // 环转不过一格，一个落点都不回收，钉的数与不装时相同。
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    let instance = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("暖机")
    };
    let first = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &super::overwrite_content(0),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("新池新建文件")
    };
    (allocator, first, devices)
}
