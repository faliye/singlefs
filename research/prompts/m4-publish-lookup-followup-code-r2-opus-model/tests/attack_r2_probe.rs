//! 攻方腿（m4-publish-lookup-followup-code-r2）T3 探针：`publish_admitted` 装第一个文件的数据单元时，照抄的每个单元在这次重写的角色清单
//! `rewritten` 上逐个 `contains`（第 11 批与七·6 前几批都没碰）。探针变异 PROBE 把那一处换成数比较次数的同义写法，记进
//! `singlefs_core::transaction::ATTACK_R2_FIRST_FILE_CONTAINS_COMPARISONS`。第一个文件长到 F 个数据单元，另有四个文件；
//! 整份重写其中一个文件（N 个数据单元）的那一次发布，量比较次数与挂钟。
mod common;

use std::sync::atomic::Ordering;
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
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
    ATTACK_R2_FIRST_FILE_CONTAINS_COMPARISONS,
};
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

fn content(data_units: usize, seed: u64) -> Vec<u8> {
    let mut state = seed | 1;
    (0..data_units * data_unit_payload_capacity())
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).expect("低 8 位")
        })
        .collect()
}

fn probe(first_file_units: usize, rewrite_units: usize) {
    let (mut devices, mut session) = formatted_and_mounted();
    let mut write_time = common::FIXED_WRITE_TIME_SECONDS + 600;
    // 第一个文件一次写一段（每次至多 64 个单元，整份换成更长的内容），长到 F 个数据单元。
    let mut units = 0;
    while units < first_file_units {
        units = (units + 64).min(first_file_units);
        write_time += 1;
        let bytes = content(units, 77 + units as u64);
        if let Err(refusal) = session.publish_user_change(&mut devices, UserChange::SequentialWrite(FirstFile { content: &bytes, write_time_seconds: write_time })) {
            println!("name=probe first_file_units={first_file_units} rewrite_units={rewrite_units} refused_growing_the_first_file_at={units} refusal={refusal:?}");
            return;
        }
    }
    let first_inode = session.current.version_of_trees().map_or(0, TransactionOutput::inode_number_watermark);
    write_time += 1;
    session.publish_user_change(&mut devices, UserChange::NewInodes { count: 4, write_time_seconds: write_time }).expect("建四个 inode");
    for inode in first_inode..first_inode + 4 {
        write_time += 1;
        let bytes = content(rewrite_units, inode);
        session
            .publish_user_change(&mut devices, UserChange::SequentialWriteOfAnInode { inode: InodeNumber(inode), content: &bytes, write_time_seconds: write_time })
            .expect("写一个文件");
    }
    let version = session.current.version_of_trees().expect("有树");
    let units_in_the_version = version.units.len();
    let first_file_data_units = version.data_pointers.len();
    ATTACK_R2_FIRST_FILE_CONTAINS_COMPARISONS.store(0, Ordering::Relaxed);
    write_time += 1;
    let bytes = content(rewrite_units, 991);
    let started = Instant::now();
    let published = session.publish_user_change(&mut devices, UserChange::SequentialWriteOfAnInode { inode: InodeNumber(first_inode + 1), content: &bytes, write_time_seconds: write_time });
    let elapsed = started.elapsed();
    let comparisons = ATTACK_R2_FIRST_FILE_CONTAINS_COMPARISONS.load(Ordering::Relaxed);
    let rewritten = session.current.version_of_trees().map_or(0, |version| version.rewritten.len());
    println!(
        "name=probe first_file_units={first_file_units} first_file_data_pointers={first_file_data_units} rewrite_units={rewrite_units} units_in_the_version={units_in_the_version} rewritten_roles={rewritten} outcome={} first_file_contains_comparisons={comparisons} publish_micros={}",
        if published.is_ok() { "ok" } else { "err" },
        elapsed.as_micros()
    );
}

#[test]
fn attack_r2_probe_of_the_first_file_contains() {
    let points: Vec<(usize, usize)> = std::env::var("ATTACK_PROBE_POINTS")
        .unwrap_or_else(|_| "64:32,256:32,1024:32,1024:8,1024:128".to_string())
        .split(',')
        .map(|point| {
            let (first, rewrite) = point.split_once(':').expect("F:N");
            (first.parse().expect("F"), rewrite.parse().expect("N"))
        })
        .collect();
    std::thread::scope(|scope| {
        let handles: Vec<_> = points.iter().map(|(first, rewrite)| scope.spawn(move || probe(*first, *rewrite))).collect();
        for handle in handles {
            handle.join().expect("探针一个点没 panic 出来");
        }
    });
}
