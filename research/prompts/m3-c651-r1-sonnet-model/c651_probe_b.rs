//! 正推腿的探针（只在草稿副本里）：全池槽 0 都坏时 core 与 checker 各择到什么。
use singlefs_checker::image::{
    system_configuration_redundancy_report, verified_system_configuration_slots, ImageReader,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::recovery::{choose_system_configuration, recover, JournalPolicy, PoolReader};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableSizes};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};

const DEVICE_BYTES: u64 = 4 << 30;

fn parameters(io_min: u32, fsid: [u8; 16]) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: fsid,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: io_min,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(io_min),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn pool(io_min: u32, fsid: [u8; 16]) -> MemoryPool {
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [0u32, 1]
        .into_iter()
        .map(|d| (DeviceIdentity(d), SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512))))
        .collect();
    make_filesystem(&parameters(io_min, fsid), &mut devices).expect("mkfs");
    MemoryPool {
        devices: devices.iter().map(|(i, d)| (*i, d.image.clone())).collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

fn kill_slot_zero(image: &mut MemoryPool, devices: &[u32]) {
    for d in devices {
        image.flip_byte(DeviceIdentity(*d), DeviceOffsetInBytes(0), 2048);
    }
}

fn report(label: &str, image: &MemoryPool) {
    let core = match choose_system_configuration(image) {
        Ok(c) => format!(
            "Ok(盘{} 槽距{} 世代{} fsid{:?})",
            c.immutable.this_device.0,
            c.immutable.sizes.fixed_structure_slot_spacing,
            c.quantities.slot_generation,
            String::from_utf8_lossy(&c.immutable.filesystem_identifier)
        ),
        Err(e) => format!("Err({e:?})"),
    };
    let verified = verified_system_configuration_slots(image);
    let per_device: Vec<String> = verified
        .iter()
        .map(|(d, slots)| {
            format!(
                "盘{d}:{}",
                slots
                    .iter()
                    .map(|(v, g)| format!("[槽距{} 世代{}]", g.slot_spacing, v.slot_generation))
                    .collect::<String>()
            )
        })
        .collect();
    let redundancy = system_configuration_redundancy_report(image);
    let i85 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        check_pool_image(image)
            .into_iter()
            .find(|(n, _)| *n == "I-8.5")
            .map(|(_, v)| format!("{v:?}"))
    }))
    .map_or_else(|_| "panic".to_string(), |v| v.unwrap_or_default());
    let recovered = format!("{:?}", recover(image, JournalPolicy::Consult).outcome);
    let recovered: String = recovered.chars().take(260).collect();
    println!("C651PROBE {label}\n  core 恢复(recover): {recovered}\n  core: {core}\n  checker 可择槽: {}\n  checker 冗余旁路: {redundancy:?}\n  checker I-8.5: {i85}", per_device.join(" "));
}

const NEW: [u8; 16] = *b"singlefs-new-poo";
const OLD: [u8; 16] = *b"singlefs-old-poo";

#[test]
fn c651_probe_b() {
    for io_min in [512u32, 8192] {
        let spacing = SystemImmutableSizes::slot_spacing_for(io_min);
        let mut p = pool(io_min, NEW);
        report(&format!("槽距{spacing} 完好"), &p);
        kill_slot_zero(&mut p, &[0]);
        report(&format!("槽距{spacing} 只盘0槽0坏"), &p);
        kill_slot_zero(&mut p, &[1]);
        report(&format!("槽距{spacing} 两盘槽0都坏"), &p);
    }
    // 槽距 8192 的池，两盘槽 0 都坏，且 4096 处残留一份旧池（槽距 4096，别的 fsid）的槽 1，世代号 1。
    let mut p = pool(8192, NEW);
    let old = pool(512, OLD);
    for d in [0u32, 1] {
        let stale = PoolReader::read(&old, DeviceIdentity(d), DeviceOffsetInBytes(4096), 4096).expect("旧槽 1");
        assert!(SystemConfiguration::parse_slot(&stale).is_ok());
        p.devices.get_mut(&DeviceIdentity(d)).unwrap().write(DeviceOffsetInBytes(4096), &stale);
    }
    report("槽距8192 槽0未坏 4096 处有旧池残留(世代1)", &p);
    kill_slot_zero(&mut p, &[0, 1]);
    report("槽距8192 两盘槽0坏 4096 处有旧池残留(世代1)", &p);
    // 只有盘 1 带残留，盘 0 的槽 1 好、盘 1 的槽 1 坏：
    let mut q = pool(8192, NEW);
    let stale = PoolReader::read(&old, DeviceIdentity(1), DeviceOffsetInBytes(4096), 4096).unwrap();
    q.devices.get_mut(&DeviceIdentity(1)).unwrap().write(DeviceOffsetInBytes(4096), &stale);
    q.flip_byte(DeviceIdentity(1), DeviceOffsetInBytes(8192), 2048);
    kill_slot_zero(&mut q, &[0, 1]);
    report("槽距8192 两盘槽0坏 盘1槽1坏且盘1的4096处有旧池残留", &q);
    let _ = ImageReader::devices(&q);
    // 反方向：槽距 4096 的新池，8192 处残留旧池（槽距 8192、别的 fsid）世代号 9 的槽 1；两盘槽 0 都坏。
    let mut r = pool(512, NEW);
    let old8192 = pool(8192, OLD);
    for d in [0u32, 1] {
        let bytes = PoolReader::read(&old8192, DeviceIdentity(d), DeviceOffsetInBytes(8192), 4096).unwrap();
        let mut stale = SystemConfiguration::parse_slot(&bytes).unwrap();
        stale.quantities.slot_generation = 9;
        r.devices.get_mut(&DeviceIdentity(d)).unwrap().write(DeviceOffsetInBytes(8192), &stale.to_slot());
    }
    report("槽距4096 两盘槽0未坏 8192 处有旧池残留(世代9)", &r);
    kill_slot_zero(&mut r, &[0, 1]);
    report("槽距4096 两盘槽0坏 8192 处有旧池残留(世代9, 旧池槽距8192)", &r);
}
