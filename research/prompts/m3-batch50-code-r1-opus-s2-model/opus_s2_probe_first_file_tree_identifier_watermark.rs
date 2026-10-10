//! s2 攻方原型 K3-1：照 `crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs` 的流 ①
//! （mkfs → 取号 → 暖机 → 第一个文件版本，同一个进程、内存盘、同一份参数与分配器装法）只做到它那一处断言之前，
//! 打出第一个文件版本那条根的 (txg, 树 ID 水位)：那份 checker 档用例断言它是 (3, 65548)。只在攻方副本上跑。
#[path = "common/mod.rs"]
mod common;

use common::{crash_state_devices, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::make_filesystem::{make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::transaction::{acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

#[test]
fn first_file_version_tree_identifier_watermark_on_stream_one() {
    let stream = SharedStream::retaining_contents();
    let empty = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    let mut devices = crash_state_devices(&empty, &[], &[], &stream);
    let publish_parameters = parameters();
    let genesis = make_filesystem(&publish_parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    allocator.install_root_ring_occupancy(
        singlefs_core::make_filesystem::root_ring_occupancy_after_make_filesystem(&publish_parameters, &genesis),
    );
    let mut writer = PoolWriter::new(&publish_parameters, &mut devices);
    let instance = acquire_instance(&mut writer).expect("取号");
    let warm = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let first_file = publish_first_file(
        &mut writer,
        &mut allocator,
        warm.roots.last().expect("暖机两代根"),
        FirstFile { content: &content_of(3000, 1), write_time_seconds: FIXED_WRITE_TIME_SECONDS },
        instance,
        &warm.last_record_bytes,
    )
    .expect("第一个文件版本");
    println!(
        "name=first-file-watermark txg={} tree_identifier_watermark={} tree_table_entries={} checker_tier_test_expects=(3, 65548)",
        first_file.root.checkpoint_txg.0,
        first_file.root.tree_identifier_watermark,
        first_file.tree_table_entries.len()
    );
}
