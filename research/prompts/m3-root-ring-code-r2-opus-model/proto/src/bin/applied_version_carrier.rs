//! K4 看池级 checker 的 CPU 一侧与 GPU 走读内核对「由记录施加出来、根槽从没落盘的那一版」取实例表与分配记录树的地方是否同一个：
//! CPU（`walk.rs` 的 `walk_version_applied_only_by_records`）从那条记录的新根段取，GPU（`crash_judge_checker.wgsl` 的 `w_walk_one`，
//! WALK_KIND_VERSION）从它施加在其上的那条根（同实例、txg 比它小、最大的那条）取。这里只造盘面、列出两边会取的指针是否相同，不跑 GPU。
//! 造法：mkfs → 反复可写挂载，直到某次挂载带暖机；那次挂载切在第一次暖机的根槽写之前（记录已持久、根槽没持久）→ 再可写挂载一次（`mode`=mount）
//! 或不挂（`mode`=none）。用法：applied_version_carrier <S> <mount|none>
use e2_attack_proto::{devices_of, geometry, parameters, pool_of, recording_of, IMAGE_BYTES};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{choose_system_configuration, instance_table_of_root, readable_roots_with_ring_slots, scan_journal};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn report(image: &MemoryPool, tag: &str) {
    let c = choose_system_configuration(image).expect("系统配置");
    let i = &c.immutable;
    let roots: Vec<_> = readable_roots_with_ring_slots(image, &i.region_devices, &i.sizes, &i.filesystem_identifier).into_iter().map(|(_, r)| r).collect();
    let newest = *roots.iter().max_by_key(|r| (r.checkpoint_txg, r.instance)).expect("有根");
    let table = instance_table_of_root(image, &newest).expect("实例表");
    println!("{tag} newest=(实例 {}, txg {}) rows={:?}", newest.instance.0, newest.checkpoint_txg.0, table.rows);
    for ((instance, counter), record) in scan_journal(image, &c) {
        let in_ring = roots.iter().any(|r| r.instance == instance && r.checkpoint_txg == record.checkpoint_txg);
        let row = table.rows.iter().find(|row| row.instance == instance);
        let applied = row.is_some_and(|row| record.checkpoint_txg <= row.selected_root_txg);
        if in_ring || !record.is_commit || !applied {
            continue;
        }
        let applied_on = roots.iter().filter(|r| r.instance == instance && r.checkpoint_txg < record.checkpoint_txg).max_by_key(|r| r.checkpoint_txg);
        let same_table = applied_on.map(|r| r.instance_table == record.new_instance_table);
        let same_allocation_root = applied_on.map(|r| r.allocation_record_tree_root == record.new_allocation_record_tree_root);
        println!(
            "{tag} APPLIED_ONLY_BY_RECORDS (实例 {}, 计数 {counter}, txg {}) applied_on_txg={:?} record_instance_table_all_zero={} record_allocation_root_all_zero={} same_instance_table_as_applied_on={same_table:?} same_allocation_root_as_applied_on={same_allocation_root:?}",
            instance.0,
            record.checkpoint_txg.0,
            applied_on.map(|r| r.checkpoint_txg.0),
            record.new_instance_table.locations.iter().all(|l| l.slot.0 == 0),
            record.new_allocation_record_tree_root.locations.iter().all(|l| l.slot.0 == 0),
        );
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let mode = a[2].clone();
    let parameters = parameters(s);
    let geo = geometry(&parameters);
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32).map(|n| (DeviceIdentity(n), SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)))).collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let crashed = loop {
        let before = pool_of(&devices);
        let stream = SharedStream::retaining_contents();
        let mut recording = recording_of(&before, &stream);
        let mounted = mount_writable(&parameters, &mut recording).expect("可写挂载");
        drop(recording);
        let (writes, _) = writes_and_segments(&stream.retained_operations(), &geo);
        if mounted.output.warm_up_publishes.is_empty() {
            let mut image = before;
            image.apply_writes(&writes);
            devices = devices_of(&image);
            continue;
        }
        let cut = writes.iter().enumerate().filter(|(_, w)| w.kind == StepKind::RootRecordFua).nth(1).map(|(k, _)| k).expect("暖机的根槽写");
        println!("CUT instance={} row_txg={:?} warm_up_txgs={:?}", mounted.output.instance.0, mounted.output.row_publish.root().checkpoint_txg.0, mounted.output.warm_up_publishes.iter().map(|v| v.root().checkpoint_txg.0).collect::<Vec<_>>());
        let mut image = before;
        image.apply_writes(&writes[..cut]);
        break image;
    };
    report(&crashed, "CRASHED");
    if mode == "mount" {
        let mut devices = devices_of(&crashed);
        let mounted = mount_writable(&parameters, &mut devices).expect("再挂");
        println!("REMOUNT instance={} row_txg={}", mounted.output.instance.0, mounted.output.row_publish.root().checkpoint_txg.0);
        report(&pool_of(&devices), "REMOUNTED");
    }
}
