//! 正推腿探针（T4）：第 2 批代码轮 T2 打中的那几个读树表的读者，对一张带码 14 / 15（快照）、码 8（deadlist 非空）、码 12（logged_ops 非空）的树表
//! 各放行还是拒。状态：建快照 → 覆盖写（旧单元钉进 deadlist）→ 签发整理意图（logged_ops 一条）。
mod common;

use common::{build_pool, parameters, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::logged_operations::{CompactionIntent, CompactionRegion};
use singlefs_core::mounted_read::{mount_read_only, open_pool_for_read};
use singlefs_core::records::{TREE_KIND_DEADLIST, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_SNAPSHOT_EXTENT_ROOT, TREE_KIND_SNAPSHOT_INODE_ROOT};
use singlefs_core::recovery::{
    allocation_records_under_root, rebuild_version, rebuild_version_with_its_snapshot_entries, user_visible_tree_root_pointers,
    walk_to_file,
};
use singlefs_core::transaction::{
    publish_compaction_step, publish_create_snapshot, publish_overwrite, CompactionIntentStep, FirstFile, PoolWriter,
};

const INSTANCE: InstanceGeneration = InstanceGeneration(1);

#[test]
fn probe_forward_the_tree_table_readers_on_a_table_with_snapshot_deadlist_and_logged_ops_entries() {
    let mut pool = build_pool("probe-forward-readers");
    let publish_parameters = parameters();
    let created = {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().unwrap();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_create_snapshot(&mut writer, &mut pool.allocator, &previous, INSTANCE).unwrap()
    };
    pool.output = created;
    let overwritten = {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().unwrap();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile { content: &[7u8; 2000], write_time_seconds: FIXED_WRITE_TIME_SECONDS + 5 },
            INSTANCE,
        )
        .unwrap()
    };
    pool.output = overwritten.clone();
    let device_map = &pool.allocator.devices[0];
    let intent = CompactionIntent {
        region: CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(50240)).unwrap(),
        allocation_generation_at_issue: overwritten.root.checkpoint_txg,
        empty_cluster_segments_at_issue: device_map.empty_segments(),
    };
    pool.allocator.compaction_mut().issue(intent);
    let issued = {
        let devices = pool.devices.as_mut().unwrap();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_compaction_step(&mut writer, &mut pool.allocator, &overwritten, &[], CompactionIntentStep::Issue(intent), INSTANCE).unwrap()
    };
    let kinds: Vec<u16> = issued.tree_table_entries.iter().map(|entry| entry.kind).collect();
    println!(
        "name=probe_forward_readers_table kinds_present: snapshot_inode={} snapshot_extent={} deadlist_root_nonzero={} logged_ops_root_nonzero={}",
        kinds.contains(&TREE_KIND_SNAPSHOT_INODE_ROOT),
        kinds.contains(&TREE_KIND_SNAPSHOT_EXTENT_ROOT),
        issued.tree_table_entries.iter().any(|entry| entry.kind == TREE_KIND_DEADLIST && entry.root != singlefs_core::pointer::NodePointer::empty_root()),
        issued.tree_table_entries.iter().any(|entry| entry.kind == TREE_KIND_LOGGED_OPERATIONS && entry.root != singlefs_core::pointer::NodePointer::empty_root()),
    );
    let image = pool.memory_pool();
    let root = issued.root;
    let mut fallbacks = 0;
    println!("name=probe_forward_readers walk_to_file={:?}", walk_to_file(&image, &root, &mut fallbacks).map(|c| c.map(|v| v.len())));
    println!("name=probe_forward_readers user_visible_tree_root_pointers={:?}", user_visible_tree_root_pointers(&image, &root).map(|p| p.snapshot_tree_roots.len()));
    println!("name=probe_forward_readers rebuild_version={:?}", rebuild_version(&image, &root, Some(issued.record.clone())).map(|_| "Ok"));
    println!("name=probe_forward_readers rebuild_version_with_its_snapshot_entries={:?}", rebuild_version_with_its_snapshot_entries(&image, &root, Some(issued.record.clone())).map(|v| v.snapshot_entries.len()));
    println!("name=probe_forward_readers allocation_records_under_root={:?}", allocation_records_under_root(&image, &root).map(|r| r.len()));
    println!("name=probe_forward_readers open_pool_for_read={:?}", open_pool_for_read(&image, &root).map(|_| "Ok"));
    println!("name=probe_forward_readers mount_read_only={:?}", mount_read_only(&image).map(|_| "Ok"));
}
