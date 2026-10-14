//! 攻方腿 m3-admission-r1-opus，A6（C568、C582）。只在冻结副本的拷贝上跑，不进仓库的 crates/。
//! C568：io_min 取几档、每区槽数 S 取几档，真跑 mkfs 看拒不拒、拒在哪一对；再按「只比同一块盘上真背根的区域」那一种放宽，
//! 逐盘列出仍然真重叠的固定结构（同一块盘上真会写的两段字节相交）。
//! C582：把盘截到「装得下单元区起点、装不下 mkfs 第 0 版写的东西或装不下现行账里的落点」那一截，可写挂载与只读挂载各走到哪一步。
mod common_admission;
#[path = "opus_probe/common.rs"]
mod probe_common;

use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::root_ring::{region_length_in_bytes, region_start, RootRingSlotsPerRegion};
use singlefs_format::{ROOT_RING_REGIONS, SLOT_BYTES};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::e142_parameters;

#[derive(Debug, Clone, Copy)]
struct Extent {
    name: &'static str,
    index: u64,
    start: u64,
    end: u64,
}

#[test]
fn opus_probe_c568_mkfs_with_large_io_min_and_the_relaxed_per_device_overlap() {
    let journal_ring_bytes: u64 = 6 << 20;
    for io_min in [4096u32, 65536, 131072, 262144, 524288, 786432, 1048576] {
        for slots_per_region in [4u64, 6, 8, 16] {
            let mut parameters = e142_parameters(512, io_min);
            parameters.geometry.journal_ring_bytes = journal_ring_bytes;
            parameters.geometry.root_ring_slots_per_region =
                RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region).expect("4..16");
            let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
            let device_bytes: u64 = 64 << 20;
            let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [0u32, 1]
                .into_iter()
                .map(|i| (DeviceIdentity(i), SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512))))
                .collect();
            let outcome = match make_filesystem(&parameters, &mut devices) {
                Ok(_) => "Ok".to_string(),
                Err(error) => format!("{error:?}").chars().take(260).collect(),
            };
            // 逐盘：系统配置两槽、journal 环在每块盘上都真写；区域 r 只在 region_devices[r] 那块盘上真写根（放宽的判据只比这些）。
            let mut per_device = Vec::new();
            for device in [0u32, 1] {
                let mut extents = vec![
                    Extent { name: "sysconf", index: 0, start: 0, end: 4096 },
                    Extent { name: "sysconf", index: 1, start: spacing, end: spacing + 4096 },
                    Extent { name: "journal", index: 0, start: 1024 * SLOT_BYTES, end: 1024 * SLOT_BYTES + journal_ring_bytes },
                ];
                for region in 0..ROOT_RING_REGIONS {
                    if parameters.region_devices[usize::try_from(region).expect("区域")].0 == device {
                        let start = region_start(region).0;
                        extents.push(Extent {
                            name: "region",
                            index: region,
                            start,
                            end: start + region_length_in_bytes(parameters.geometry.fixed_structure_slot_spacing, parameters.geometry.root_ring_slots_per_region),
                        });
                    }
                }
                let mut overlaps = Vec::new();
                for (i, a) in extents.iter().enumerate() {
                    for b in &extents[i + 1..] {
                        if a.start < b.end && b.start < a.end {
                            overlaps.push(format!("{}{}[{}..{})x{}{}[{}..{})", a.name, a.index, a.start, a.end, b.name, b.index, b.start, b.end));
                        }
                    }
                }
                per_device.push(format!("d{device}:{}", if overlaps.is_empty() { "none".to_string() } else { overlaps.join(";") }));
            }
            println!("GEO io_min={io_min} S={slots_per_region} spacing={spacing} mkfs={outcome} relaxed_true_overlaps {}", per_device.join(" "));
        }
    }
}

fn truncated_copy(devices: &[(DeviceIdentity, SparseBlockDevice)], device_to_cut: u32, slots: u64) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    devices
        .iter()
        .map(|(identity, device)| {
            let bytes = if identity.0 == device_to_cut { slots * SLOT_BYTES } else { device.size_in_bytes() };
            let mut copy = SparseBlockDevice::new(bytes, PhysicalBlockSizeInBytes(512));
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}

fn try_mounts(tag: &str, parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters, devices: Vec<(DeviceIdentity, SparseBlockDevice)>) {
    use singlefs_core::admission::SpaceAdmission;
    use singlefs_core::mount::mount_writable_with_space_admission;
    use singlefs_core::mounted_read::mount_read_only;
    let mut writable_devices = devices
        .iter()
        .map(|(identity, device)| {
            let mut copy = SparseBlockDevice::new(device.size_in_bytes(), PhysicalBlockSizeInBytes(512));
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect::<Vec<_>>();
    let read_only = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match mount_read_only(&devices) {
        Ok(_) => "Ok".to_string(),
        Err(error) => format!("{error:?}").chars().take(200).collect(),
    }))
    .unwrap_or_else(|panic| format!("PANIC {}", panic.downcast_ref::<String>().cloned().or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default().chars().take(200).collect::<String>()));
    let writable = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        match mount_writable_with_space_admission(parameters, &mut writable_devices, SpaceAdmission::JudgedByTheFormula) {
            Ok(mounted) => {
                let session = singlefs_core::mounted_session::MountedSession::of_the_mount(mounted).1;
                format!("Ok unit_area_slots={:?}", session.allocator.devices.iter().map(|d| d.unit_area_slots()).collect::<Vec<_>>())
            }
            Err(error) => format!("{error:?}").chars().take(200).collect(),
        }
    }))
    .unwrap_or_else(|panic| format!("PANIC {}", panic.downcast_ref::<String>().cloned().or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default().chars().take(200).collect::<String>()));
    println!("CAP {tag} read_only={read_only} | writable={writable}");
}

#[test]
fn opus_probe_c582_mount_of_a_device_cut_between_the_unit_area_start_and_what_was_written() {
    use common_admission::content_of;
    let journal_ring_bytes: u64 = 6 << 20;
    let unit_area_start = 1408u64;
    // 只做 mkfs 的池。
    {
        let mut parameters = e142_parameters(512, 512);
        parameters.geometry.journal_ring_bytes = journal_ring_bytes;
        let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [0u32, 1]
            .into_iter()
            .map(|i| (DeviceIdentity(i), SparseBlockDevice::new((unit_area_start + 1391) * SLOT_BYTES, PhysicalBlockSizeInBytes(512))))
            .collect();
        make_filesystem(&parameters, &mut devices).expect("mkfs");
        for cut in [0u32, 1] {
            for extra in [0u64, 1, 2, 3] {
                try_mounts(&format!("mkfs_only cut_device={cut} slots=U+{extra}"), &parameters, truncated_copy(&devices, cut, unit_area_start + extra));
            }
        }
    }
    // 第一个文件之后，再顺序写几次（数据落到更高的槽）。
    let mut pool = probe_common::start_pool([1391, 1391], journal_ring_bytes);
    let highest = |pool: &common_admission::PoolUnderTest<SparseBlockDevice>| {
        pool.session().allocator.records().iter().filter(|r| !r.is_released).map(|r| r.slot.0 + u64::from(r.span_slots)).max().unwrap_or(0)
    };
    let mut stage = 0;
    for writes in [0usize, 6] {
        while stage < writes {
            stage += 1;
            let content = content_of(8 * singlefs_core::unit::data_unit_payload_capacity(), stage as u64);
            pool.write_time_seconds += 1;
            let time = pool.write_time_seconds;
            let _ = pool.publish(
                singlefs_core::mounted_session::UserChange::SequentialWrite(singlefs_core::transaction::FirstFile { content: &content, write_time_seconds: time }),
                Some(&content),
            );
        }
        let top = highest(&pool);
        println!("CAP stage writes={writes} highest_allocated_slot_end={top} (U={unit_area_start}, U+{})", top - unit_area_start);
        let devices: Vec<(DeviceIdentity, SparseBlockDevice)> = pool
            .devices
            .iter()
            .map(|(identity, device)| {
                let mut copy = SparseBlockDevice::new(device.size_in_bytes(), PhysicalBlockSizeInBytes(512));
                copy.image = device.image.clone();
                (*identity, copy)
            })
            .collect();
        for cut in [0u32, 1] {
            for extra in [0u64, 1, 2, 3, 4, (top - unit_area_start) / 2, top - unit_area_start - 1, top - unit_area_start] {
                try_mounts(&format!("writes={writes} cut_device={cut} slots=U+{extra}"), &pool.parameters, truncated_copy(&devices, cut, unit_area_start + extra));
            }
        }
    }
}
