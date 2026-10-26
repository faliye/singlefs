//! 不认隔离记号的旧读者（从 HEAD 导出的代码，没有 0x4000 的读法）读新写者倒出来的两份镜像：一份带记号，一份是没隔离的对照。
//! 只打 PROBE 行。读者：只读挂载、可写挂载、冷走读、分配记录树读回、池级 checker。
mod common;

use common::parameters;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{allocation_records_under_root, choose_root, choose_system_configuration, walk_to_file};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use std::collections::BTreeMap;

fn load(path: &std::path::Path) -> MemoryPool {
    let bytes = std::fs::read(path).expect("读倒出来的镜像");
    let device_size_in_bytes = u64::from_le_bytes(bytes[0..8].try_into().expect("8"));
    let mut devices: BTreeMap<DeviceIdentity, SparseDevice> = BTreeMap::new();
    let mut at = 8;
    while at < bytes.len() {
        let device = u32::from_le_bytes(bytes[at..at + 4].try_into().expect("4"));
        let sector = u64::from_le_bytes(bytes[at + 4..at + 12].try_into().expect("8"));
        let content = &bytes[at + 12..at + 12 + 512];
        devices
            .entry(DeviceIdentity(device.try_into().expect("设备号")))
            .or_default()
            .write(DeviceOffsetInBytes(sector * 512), content);
        at += 12 + 512;
    }
    MemoryPool { devices, device_size_in_bytes }
}

fn short<T: std::fmt::Debug>(value: &T) -> String {
    format!("{value:?}").chars().take(260).collect()
}

fn read_with_the_old_code(label: &str, image: &MemoryPool) {
    let read_only = match mount_read_only(image) {
        Ok(_) => "ok".to_string(),
        Err(refusal) => format!("REFUSED {}", short(&refusal)),
    };
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect();
    let writable = match mount_writable(&parameters(), &mut devices) {
        Ok(_) => "ok".to_string(),
        Err(refusal) => format!("REFUSED {}", short(&refusal)),
    };
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let root = choose_root(image, &system_configuration).expect("根");
    let mut fallbacks = 0;
    let cold = match walk_to_file(image, &root, &mut fallbacks) {
        Ok(found) => format!("ok(file_bytes={:?})", found.map(|bytes| bytes.len())),
        Err(failure) => format!("REFUSED {}", short(&failure)),
    };
    let records = match allocation_records_under_root(image, &root) {
        Ok(records) => format!("ok({} records)", records.len()),
        Err(failure) => format!("REFUSED {}", short(&failure)),
    };
    let verdicts = check_pool_image(image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect();
    eprintln!(
        "PROBE name=old_reader image={label} mount_read_only={read_only} mount_writable={writable} cold_walk={cold} allocation_record_tree={records} checker_checked={} checker_violated={violated:?}",
        verdicts.len()
    );
}

#[test]
fn probe_old_reader_on_the_marked_image_and_the_control() {
    let Ok(directory) = std::env::var("M4R2_DUMP_DIR") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    for label in ["control", "marked"] {
        read_with_the_old_code(label, &load(&directory.join(format!("{label}.bin"))));
    }
}
