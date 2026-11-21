//! 内存池交给池级 checker 的扫描候选按盘上系统配置槽 0 自述的几何划（`memory_pool` 的 `recorded_geometry_in_the_slot_zero_read_through`）：
//! 槽 0 的单元几何四格（单元大小 305、落点粒度 309、位置条目宽 313、扩展点声明值 N 321；D21（权威态与派生态的分界） 已定项 16、
//! 已定项 8，C1041）有一格不是第一版的值时，这一槽与 core 择系统配置、池级 checker 读槽一样不算数，候选退到罩住任何合法几何的那一档。
//! 看得出来的是 journal 扫描候选：认槽 0 时只罩自述的环（mkfs 清过的环里一个写过的扇区都没有，候选为空）；不认时从环起点罩到盘尾，
//! 单元区里 mkfs 写的实例表与树表落进来、候选不空。偏移与改成的值用例自己写死，不从实现里取。

use singlefs_checker::image::ImageReader;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{
    SystemImmutableSizes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};

const DEVICE_BYTES: u64 = 4 << 30;
const PHYSICAL_BLOCK_SIZE: u32 = 512;

/// mkfs 之后两块盘的镜像，按盘放进一个内存池（与崩溃点重放的基线池同一个类型）。
fn memory_pool_after_make_filesystem() -> MemoryPool {
    let parameters = MakeFilesystemParameters {
        filesystem_identifier: *b"singlefs-scangeo",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_SIZE,
            minimum_input_output_bytes: 0,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(0),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    };
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE)),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs 收这一份几何");
    MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.image))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

/// 内存池盘 0 的系统配置槽 0 里 `offset_in_the_slot` 起的 4 字节改成 `rewritten_value`（小端）、整槽校验和重封。
fn rewrite_a_field_of_slot_zero_on_device_zero(
    pool: &mut MemoryPool,
    offset_in_the_slot: usize,
    rewritten_value: u32,
) {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let device_zero = pool
        .devices
        .get_mut(&DeviceIdentity(0))
        .expect("池里有盘 0");
    let mut slot = device_zero.read(DeviceOffsetInBytes(0), slot_bytes);
    slot[offset_in_the_slot..offset_in_the_slot + 4]
        .copy_from_slice(&rewritten_value.to_le_bytes());
    let digest =
        wide_checksum_with_field_zeroed(&slot, slot_bytes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
        .copy_from_slice(&digest);
    device_zero.write(DeviceOffsetInBytes(0), &slot);
}

/// 槽 0 四格都是第一版的值时 journal 候选为空（阳性对照）；四格各改一个值（各在池的一份拷贝上），这块盘的 journal 候选都不空，
/// 拷贝的槽 0 照样过 magic、整槽校验和、incompat 那几关。
#[test]
fn journal_scan_candidates_reach_past_the_recorded_ring_when_slot_zero_carries_a_unit_geometry_field_other_than_the_first_version_value(
) {
    let pool = memory_pool_after_make_filesystem();
    assert_eq!(
        ImageReader::candidate_journal_slots(&pool, 0),
        Some(Vec::new()),
        "槽 0 四格都是第一版的值：journal 候选只罩自述的环，mkfs 清过的环里一个写过的扇区都没有（阳性对照）"
    );
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for (field_offset, rewritten_value, field_named) in [
        (305, 65_536, "单元大小"),
        (309, 4096, "落点粒度"),
        (313, 15, "位置条目宽"),
        (321, 1, "扩展点声明值 N"),
    ] {
        let mut rewritten_pool = pool.clone();
        rewrite_a_field_of_slot_zero_on_device_zero(
            &mut rewritten_pool,
            field_offset,
            rewritten_value,
        );
        let slot =
            rewritten_pool.devices[&DeviceIdentity(0)].read(DeviceOffsetInBytes(0), slot_bytes);
        assert!(
            singlefs_checker::check_system_configuration_slot(&slot).is_ok(),
            "{field_named} 写成 {rewritten_value}、重封之后 magic、整槽校验和、incompat 那几关照样过：不认它的只能是四格那一判"
        );
        let candidates =
            ImageReader::candidate_journal_slots(&rewritten_pool, 0).expect("池里有盘 0");
        assert!(
            !candidates.is_empty(),
            "{field_named} 写成 {rewritten_value} 的槽 0 不算数：journal 候选从环起点罩到盘尾、单元区里 mkfs 写的单元落进来，而候选为空"
        );
    }
}
