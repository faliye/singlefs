//! 第 21 批 K1 的扫描：不同环几何 S、不同环长、不同只暖机挂载次数 k 上，第一个文件的 txg 与数据单元落点（相对 U）、I-5.5 与池级红条。
mod common;
use common::*;
use singlefs_checker::image::FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker::image::InvariantVerdict;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::mount_writable;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type MemoryDevice = RecordingBlockDevice<SparseBlockDevice>;
fn memory_devices(stream: &SharedStream) -> Vec<(DeviceIdentity, MemoryDevice)> {
    (0..2u32).map(|n| (DeviceIdentity(n), RecordingBlockDevice::with_shared_stream(DeviceIdentity(n),
        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)), stream.clone()))).collect()
}

#[test]
fn scan() {
    let slack = std::env::var("B21_SLACK").unwrap_or("0".into());
    let journals: Vec<u64> = std::env::var("B21_J").unwrap_or("default".into()).split(',').map(|x| if x=="default" {JOURNAL_RING_DEFAULT_BYTES_LOCAL} else {128u64<<20}).collect();
    let ss: Vec<u64> = std::env::var("B21_S").unwrap().split(',').map(|x| x.parse().unwrap()).collect();
    for journal in journals {
        for &s in &ss {
            let mut p = parameters();
            p.geometry.journal_ring_bytes = journal;
            p.geometry.root_ring_slots_per_region = match RootRingSlotsPerRegion::from_system_configuration_field(s) { Ok(v) => v, Err(_) => continue };
            for k in (s as usize)..=(s as usize + 3) {
                let stream = SharedStream::retaining_contents();
                let mut devices = memory_devices(&stream);
                let genesis = make_filesystem(&p, &mut devices).expect("mkfs");
                let u = genesis.unit_area_start.slot().0;
                let mut mounted = mount_writable(&p, &mut devices).expect("m");
                for _ in 1..k { mounted = mount_writable(&p, &mut devices).expect("m"); }
                let PoolVersion::WithoutFile(version) = &mounted.current else { panic!() };
                let mut allocator = mounted.allocator.clone();
                let out = {
                    let mut writer = PoolWriter::new(&p, devices.as_mut_slice());
                    publish_first_file(&mut writer, &mut allocator, &version.root,
                        FirstFile { content: &file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
                        mounted.output.instance, &version.record_bytes).expect("first file")
                };
                let mut slots: Vec<(u32, u64)> = out.data_pointers.iter().flat_map(|x| x.locations).map(|l| (l.device.0, l.slot.0 - u)).collect();
                slots.sort_unstable();
                let image = memory_pool_of_sparse_devices(&devices);
                let verdicts = check_pool_image(&image);
                let i55 = verdicts.iter().find(|(i, _)| *i == FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK).map(|x| match &x.1 { InvariantVerdict::Holds => "Holds".to_string(), InvariantVerdict::Violated(d) => format!("Violated({d})"), InvariantVerdict::NotApplicable(_) => "NA".into(), InvariantVerdict::PartiallyJudged(_) => "Partial".into() }).unwrap();
                let red: Vec<&str> = verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(i, _)| *i).collect();
                println!("ROW slack={slack} journal={journal} S={s} k={k} txg={} data_off_U={:?} I55={i55} red={red:?}", out.root.checkpoint_txg.0, slots);
            }
        }
    }
}
const JOURNAL_RING_DEFAULT_BYTES_LOCAL: u64 = singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
