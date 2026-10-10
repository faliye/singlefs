//! 正推腿探针（第 19 批代码轮第一轮，K1）：只做过 mkfs 的池，接连可写挂载 k 次（k = 1..=K_MAX），
//! 在最后一次挂载里推 z 次零单元发布，再写第一个文件，扫 (k, z) 全格：数据单元落点相对 U 的偏移、I-5.5 判定、池级 checker 判红的清单。
//! 全是合法历史（实现自己写出来的）：I-5.5 判红就是误红。
mod common;

use common::{file_content, memory_pool_of_sparse_devices, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::{
    InvariantVerdict, FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::transaction::{
    publish_first_file, publish_without_units, FirstFile, PoolVersion, PoolWriter, ZeroUnitPublishPlan,
};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type MemoryDevice = RecordingBlockDevice<SparseBlockDevice>;

fn memory_devices(stream: &SharedStream) -> Vec<(DeviceIdentity, MemoryDevice)> {
    (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(number),
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

fn history(k: usize, z: u64) -> String {
    let stream = SharedStream::retaining_contents();
    let mut devices = memory_devices(&stream);
    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let u = genesis.unit_area_start.slot().0;
    let mut mounted: Mounted = mount_writable(&parameters(), &mut devices).expect("mount");
    for _ in 1..k {
        mounted = mount_writable(&parameters(), &mut devices).expect("mount again");
    }
    let PoolVersion::WithoutFile(version) = &mounted.current else {
        panic!("without file")
    };
    let mut current = version.clone();
    for _ in 0..z {
        let p = parameters();
        let mut writer = PoolWriter::new(&p, devices.as_mut_slice());
        current = publish_without_units(
            &mut writer,
            &current.root,
            ZeroUnitPublishPlan {
                txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                counter: current.record.counter + 1,
                instance: mounted.output.instance,
                back_chain: back_chain_of(&current.record_bytes),
                rollback_floor: current.root.rollback_floor,
                tree_identifier_watermark: current.root.tree_identifier_watermark,
            },
        )
        .expect("zero unit publish");
    }
    let mut allocator = mounted.allocator.clone();
    let p = parameters();
    let first_file = {
        let mut writer = PoolWriter::new(&p, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &current.root,
            FirstFile { content: &file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            mounted.output.instance,
            &current.record_bytes,
        )
        .expect("first file")
    };
    let mut slots: Vec<(u32, u64)> = first_file
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
        .map(|location| (location.device.0, location.slot.0 - u))
        .collect();
    slots.sort_unstable();
    let image = memory_pool_of_sparse_devices(&devices);
    let report = check_pool_image(&image);
    let i55 = report
        .iter()
        .find(|(id, _)| *id == FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK)
        .map(|(_, v)| match v {
            InvariantVerdict::Holds => "Holds".to_string(),
            InvariantVerdict::NotApplicable(r) => format!("NotApplicable({r})"),
            InvariantVerdict::Violated(d) => format!("VIOLATED({d})"),
        })
        .unwrap();
    let reds: Vec<&str> = report
        .iter()
        .filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_)))
        .map(|(id, _)| *id)
        .collect();
    format!(
        "k={k} z={z} txg={} instance={:?} unit_offsets_from_U={slots:?} I-5.5={i55} reds={reds:?}",
        first_file.root.checkpoint_txg.0, mounted.output.instance
    )
}

#[test]
fn scan_k_and_z() {
    let ks: Vec<usize> = std::env::var("B19_KS").unwrap_or("1,2,3,4,5,6,7,8,9,10,11,12,13,14".into()).split(',').map(|v| v.parse().unwrap()).collect();
    let zs: Vec<u64> = std::env::var("B19_ZS").unwrap_or("0,10,21,22,23,30".into()).split(',').map(|v| v.parse().unwrap()).collect();
    let cells: Vec<(usize, u64)> = ks.iter().flat_map(|k| zs.iter().map(move |z| (*k, *z))).collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let lines = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..7 {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if i >= cells.len() {
                    break;
                }
                let line = history(cells[i].0, cells[i].1);
                println!("{line}");
                lines.lock().unwrap().push((cells[i], line));
            });
        }
    });
    let mut lines = lines.into_inner().unwrap();
    lines.sort();
    let mut bad = 0;
    for (_, line) in &lines {
        println!("{line}");
        if line.contains("VIOLATED") || !line.contains("reds=[]") {
            bad += 1;
        }
    }
    println!("SCAN_DONE cells={} bad={bad}", cells.len());
}

/// 变体：第一个文件版本的根槽没落盘（清零），它的 journal 记录在——这一版只由记录施加出来（`versions_applied_only_by_records` 那一路）。
fn record_only_history(k: usize) -> String {
    use singlefs_core::block_device::{BlockDevice, WriteDurability};
    let stream = SharedStream::retaining_contents();
    let mut devices = memory_devices(&stream);
    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let u = genesis.unit_area_start.slot().0;
    let mut mounted: Mounted = mount_writable(&parameters(), &mut devices).expect("mount");
    for _ in 1..k {
        mounted = mount_writable(&parameters(), &mut devices).expect("mount again");
    }
    let PoolVersion::WithoutFile(version) = &mounted.current else { panic!("without file") };
    let mut allocator = mounted.allocator.clone();
    let p = parameters();
    let first_file = {
        let mut writer = PoolWriter::new(&p, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile { content: &file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            mounted.output.instance,
            &version.record_bytes,
        )
        .expect("first file")
    };
    let target = singlefs_core::root_ring::target_for_publish(
        first_file.root.checkpoint_txg,
        p.geometry.root_ring_slots_per_region,
    );
    let root_device = p.region_devices[usize::try_from(target.region).unwrap()];
    let root_offset = singlefs_core::root_ring::slot_offset(target, p.geometry.fixed_structure_slot_spacing);
    let bytes = usize::try_from(p.geometry.physical_block_size).unwrap();
    let index = devices.iter().position(|(id, _)| *id == root_device).unwrap();
    devices[index].1.write_at(root_offset, &vec![0u8; bytes], WriteDurability::Plain).expect("zero the root slot");
    let image = memory_pool_of_sparse_devices(&devices);
    let report = check_pool_image(&image);
    let i55 = report
        .iter()
        .find(|(id, _)| *id == FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK)
        .map(|(_, v)| match v {
            InvariantVerdict::Holds => "Holds".to_string(),
            InvariantVerdict::NotApplicable(r) => format!("NotApplicable({r})"),
            InvariantVerdict::Violated(d) => format!("VIOLATED({d})"),
        })
        .unwrap();
    let reds: Vec<&str> = report.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(id, _)| *id).collect();
    let _ = u;
    format!("record-only k={k} txg={} I-5.5={i55} reds={reds:?}", first_file.root.checkpoint_txg.0)
}

#[test]
fn scan_record_only() {
    for k in [1usize, 2, 9, 10, 11] {
        println!("{}", record_only_history(k));
    }
}
