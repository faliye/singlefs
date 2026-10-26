//! 攻方腿（m4-publish-lookup-followup-code-r2）T2 见证：恢复重建一版那一侧调 take_the_mapping_positions_over_the_location_hints 时，
//! 「把映射位置换进 units」那一处（LookupSite::UnitsOfTheMappedRolesTakingTheirMappingPositions）进了几次、建了几份索引、查了几次，
//! 连同重挂一次的挂钟。池：第一个文件之外 K 个八单元的文件（K 取 ATTACK_REMOUNT_FILES，逗号隔开，默认 16,64,256），不卸载直接重挂。
mod common;

use std::time::Instant;

use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::CompactionSwitches;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::mount_writable_with_compaction_switches;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::transaction::lookup_work::{lookup_work_while, LookupSite};
use singlefs_core::transaction::{acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::SparseBlockDevice;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn formatted_and_mounted() -> (Devices, MountedSession) {
    let parameters = common::parameters();
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| (identity, SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512))))
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(&parameters, &genesis));
    {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_file = common::file_content();
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile { content: &first_file, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    let mounted = mount_writable_with_compaction_switches(&parameters, &mut devices, CompactionSwitches::PRODUCT).expect("可写挂载");
    (devices, MountedSession::of_the_mount(mounted).1)
}

fn remount_on_a_pool_of(files: u64) {
    let (mut devices, mut session) = formatted_and_mounted();
    let first_inode = session.current.version_of_trees().map_or(0, TransactionOutput::inode_number_watermark);
    let mut write_time = common::FIXED_WRITE_TIME_SECONDS + 600;
    session.publish_user_change(&mut devices, UserChange::NewInodes { count: files, write_time_seconds: write_time }).expect("建 inode");
    for inode in first_inode..first_inode + files {
        write_time += 1;
        let bytes: Vec<u8> = (0..8 * data_unit_payload_capacity()).map(|at| u8::try_from((at as u64 + inode) & 0xff).expect("低 8 位")).collect();
        session
            .publish_user_change(&mut devices, UserChange::SequentialWriteOfAnInode { inode: InodeNumber(inode), content: &bytes, write_time_seconds: write_time })
            .expect("写一个文件");
    }
    let version = session.current.version_of_trees().expect("有树");
    let (units, mapped) = (version.units.len(), version.mapped_units.len());
    drop(session);
    let parameters = common::parameters();
    let started = Instant::now();
    let (mounted, lookup_work) = lookup_work_while(|| mount_writable_with_compaction_switches(&parameters, &mut devices, CompactionSwitches::PRODUCT));
    let elapsed = started.elapsed();
    let site = lookup_work.of_the_site(LookupSite::UnitsOfTheMappedRolesTakingTheirMappingPositions);
    println!(
        "name=remount files={files} units={units} mapped={mapped} mount={} times_entered={} index_builds={} index_lookups={} mount_micros={}",
        if mounted.is_ok() { "ok" } else { "err" },
        site.times_entered,
        site.index_builds,
        site.index_lookups,
        elapsed.as_micros()
    );
}

#[test]
fn attack_r2_remount_witness() {
    let files: Vec<u64> = std::env::var("ATTACK_REMOUNT_FILES")
        .unwrap_or_else(|_| "16,64,256".to_string())
        .split(',')
        .map(|text| text.parse().expect("文件数"))
        .collect();
    std::thread::scope(|scope| {
        for count in files {
            scope.spawn(move || remount_on_a_pool_of(count));
        }
    });
}
