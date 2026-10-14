//! 草稿探针（腿自己的副本，不入库）：盘容量落在 [单元区起点, mkfs 下界) 之间时可写挂载与只读挂载走到哪一步。
//! mkfs 下界 = 单元区起点 + 3 槽（实例表 2 槽 + 树表 1 槽）；挂载一侧下界 = 完整槽数 >= 单元区起点。
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, UNIT_AREA_START_SLOT};
use singlefs_harness::memory_pool::SparseBlockDevice;

const SLOT: u64 = 16384;
fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"c582-probe------",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}
fn formatted(big: u64) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    let mut devices: Vec<_> = [0u32, 1]
        .into_iter()
        .map(|n| (DeviceIdentity(n), SparseBlockDevice::new(big, PhysicalBlockSizeInBytes(512))))
        .collect();
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    devices
}
fn shrink(devices: &mut Vec<(DeviceIdentity, SparseBlockDevice)>, which: &[u32], bytes: u64) {
    for (id, device) in devices.iter_mut() {
        if which.contains(&id.0) {
            let mut short = SparseBlockDevice::new(bytes, PhysicalBlockSizeInBytes(512));
            short.image = device.image.clone();
            *device = short;
        }
    }
}
#[test]
fn probe() {
    for which in [vec![1u32], vec![0u32, 1]] {
        for extra in 0..=40u64 {
            let bytes = (UNIT_AREA_START_SLOT + extra) * SLOT;
            let mut devices = formatted(4 << 30);
            shrink(&mut devices, &which, bytes);
            let w = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                match mount_writable(&parameters(), &mut devices) {
                    Ok(_) => "Ok(mounted)".to_string(),
                    Err(e) => format!("Err({e:?})"),
                }
            }))
            .unwrap_or_else(|p| format!("PANIC({})", p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default()));
            let mut devices2 = formatted(4 << 30);
            shrink(&mut devices2, &which, bytes);
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                match mount_read_only(&devices2) {
                    Ok(_) => "Ok(mounted)".to_string(),
                    Err(e) => format!("Err({e:?})"),
                }
            }))
            .unwrap_or_else(|p| format!("PANIC({})", p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default()));
            let w: String = w.chars().take(300).collect();
            let r: String = r.chars().take(300).collect();
            println!("name=c582-probe shrunk_devices={which:?} slots_above_unit_area_start={extra} device_bytes={bytes} writable={w} read_only={r}");
        }
    }
}
