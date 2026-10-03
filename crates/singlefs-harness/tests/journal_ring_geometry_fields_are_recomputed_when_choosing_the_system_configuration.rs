//! I-8.1（环几何够大） 的三条等式由 core 在择系统配置时复算（判决 `research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 4 行
//! 「core 同判并进实现簇」）：安全系数 F 等于格式常量（第一版 3；F ≥ 2 那一半落在格式常量的编译期断言上）、
//! 在飞记录数上限 = ⌊环长 ÷ 4096 ÷ F⌋、最坏占用 = 在飞上限 × 4096。三个字段各改一个、整槽校验和重封，
//! 只读恢复与可写挂载都在读系统配置那一步整池拒、报同一个成员，盘上逐字节不变。改之前读者不读这三个字段，两个入口照常挂。
//!
//! 盘：两块 4 GiB 内存稀疏盘上只做过 mkfs 的池（默认环 768 MiB：在飞上限 65536、最坏占用 256 MiB）。

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::recovery::{
    recover, JournalPolicy, RecoveryFailure, RecoveryOutcome,
    SystemConfigurationValueOutsideWhatThisReaderAccepts,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{
    SystemImmutableSizes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_SAFETY_FACTOR,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};

const DEVICE_BYTES: u64 = 4 << 30;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-m3-i8-1";
/// 系统配置槽里 journal 在飞记录数上限（4 字节）、最坏占用（8 字节）、安全系数 F（4 字节）的偏移（字段表 `layout/01-first-txn.md` 一）。
/// 每条用例改之前先按 mkfs 写下的值核一遍偏移。
const IN_FLIGHT_RECORD_LIMIT_OFFSET: usize = 345;
const WORST_CASE_OCCUPANCY_OFFSET: usize = 349;
const SAFETY_FACTOR_OFFSET: usize = 357;
/// 默认环 768 MiB：⌊768 MiB ÷ 4096 ÷ 3⌋ = 65536。
const DEFAULT_RING_IN_FLIGHT_RECORD_LIMIT: u64 =
    JOURNAL_RING_DEFAULT_BYTES / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
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

fn formatted_pool() -> Devices {
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    make_filesystem(&parameters(), &mut devices).expect("默认几何上 mkfs");
    devices
}

fn images_of(devices: &Devices) -> Vec<(DeviceIdentity, SparseDevice)> {
    devices
        .iter()
        .map(|(identity, device)| (*identity, device.image.clone()))
        .collect()
}

/// 两块盘各两个系统配置槽（偏移 0 与槽距）：先核 mkfs 写下的三个字段就是环长现算的值，再改一下、整槽校验和重封。
fn rewrite_every_system_configuration_slot(devices: &mut Devices, change: impl Fn(&mut [u8])) {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    for (identity, device) in devices.iter_mut() {
        for offset in [0, spacing] {
            let mut slot = device.image.read(DeviceOffsetInBytes(offset), slot_bytes);
            assert_eq!(
                (
                    u32::from_le_bytes(
                        slot[IN_FLIGHT_RECORD_LIMIT_OFFSET..IN_FLIGHT_RECORD_LIMIT_OFFSET + 4]
                            .try_into()
                            .expect("4 字节")
                    ),
                    u64::from_le_bytes(
                        slot[WORST_CASE_OCCUPANCY_OFFSET..WORST_CASE_OCCUPANCY_OFFSET + 8]
                            .try_into()
                            .expect("8 字节")
                    ),
                    u32::from_le_bytes(
                        slot[SAFETY_FACTOR_OFFSET..SAFETY_FACTOR_OFFSET + 4]
                            .try_into()
                            .expect("4 字节")
                    ),
                ),
                (
                    u32::try_from(DEFAULT_RING_IN_FLIGHT_RECORD_LIMIT).expect("65536"),
                    DEFAULT_RING_IN_FLIGHT_RECORD_LIMIT * JOURNAL_RECORD_BYTES,
                    u32::try_from(JOURNAL_SAFETY_FACTOR).expect("3"),
                ),
                "盘 {identity:?} 偏移 {offset} 的槽：345、349、357 就是在飞上限、最坏占用、F"
            );
            change(&mut slot);
            let digest = wide_checksum_with_field_zeroed(
                &slot,
                slot_bytes,
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            device.image.write(DeviceOffsetInBytes(offset), &slot);
        }
    }
}

/// 只读恢复与可写挂载都在读系统配置那一步拒、报同一个成员（盘 0 先读到的那一槽），可写挂载在任何写之前拒。交回被拒的那个值。
fn refused_by_both_mounts_before_any_write(
    devices: &mut Devices,
) -> SystemConfigurationValueOutsideWhatThisReaderAccepts {
    let images_before = images_of(devices);
    let RecoveryOutcome::Failed {
        failure:
            RecoveryFailure::SystemConfigurationValueRefused {
                device: DeviceIdentity(0),
                value,
            },
        ..
    } = recover(&*devices, JournalPolicy::Consult).outcome
    else {
        panic!("只读恢复要在读系统配置那一步拒（盘 0 先读到的那一槽）");
    };
    let writable = mount_writable(&parameters(), devices);
    assert!(
        matches!(
            &writable,
            Err(MountError::Recovery(RecoveryFailure::SystemConfigurationValueRefused {
                device: DeviceIdentity(0),
                value: refused_by_the_mount,
            })) if *refused_by_the_mount == value
        ),
        "可写挂载与只读恢复报同一个成员：{:?}",
        writable.as_ref().err()
    );
    assert_eq!(images_of(devices), images_before, "两块盘逐字节不变");
    value
}

/// 场景：四个系统配置槽的安全系数 F 从 3 改成 2、整槽重封（在飞上限与最坏占用不动）。
/// 预期：两个入口都整池拒，报 F 不是这个读者写的那个（记着 2、第一版 3）。改之前读者不读 F，两个入口照常挂。
#[test]
fn a_safety_factor_other_than_the_first_version_value_is_refused_by_both_mounts() {
    let mut devices = formatted_pool();
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        slot[SAFETY_FACTOR_OFFSET..SAFETY_FACTOR_OFFSET + 4].copy_from_slice(&2u32.to_le_bytes());
    });
    assert_eq!(
        refused_by_both_mounts_before_any_write(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalSafetyFactorNotTheFirstVersionValue {
            recorded_safety_factor: 2,
            first_version_safety_factor: JOURNAL_SAFETY_FACTOR,
        }
    );
}

/// 场景：四个系统配置槽的在飞记录数上限从 65536 改成 65537、整槽重封（F 与最坏占用不动）。
/// 预期：两个入口都整池拒，报在飞上限不等于按环长现算的 65536。改之前读者不读这 4 字节，两个入口照常挂。
#[test]
fn an_in_flight_record_limit_other_than_the_one_recomputed_from_the_ring_length_is_refused_by_both_mounts(
) {
    let mut devices = formatted_pool();
    let recorded = u32::try_from(DEFAULT_RING_IN_FLIGHT_RECORD_LIMIT + 1).expect("65537");
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        slot[IN_FLIGHT_RECORD_LIMIT_OFFSET..IN_FLIGHT_RECORD_LIMIT_OFFSET + 4]
            .copy_from_slice(&recorded.to_le_bytes());
    });
    assert_eq!(
        refused_by_both_mounts_before_any_write(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalInFlightRecordLimitNotRecomputedFromTheRingLength {
            recorded_in_flight_record_limit: recorded,
            recomputed_in_flight_record_limit: DEFAULT_RING_IN_FLIGHT_RECORD_LIMIT,
        }
    );
}

/// 场景：四个系统配置槽的最坏占用从 65536 × 4096 改成多一条记录（+ 4096）、整槽重封（F 与在飞上限不动）。
/// 预期：两个入口都整池拒，报最坏占用不等于现算的在飞上限 × 4096。改之前读者不读这 8 字节，两个入口照常挂。
#[test]
fn a_worst_case_occupancy_other_than_the_in_flight_limit_times_the_record_bytes_is_refused_by_both_mounts(
) {
    let mut devices = formatted_pool();
    let recomputed = DEFAULT_RING_IN_FLIGHT_RECORD_LIMIT * JOURNAL_RECORD_BYTES;
    let recorded = recomputed + JOURNAL_RECORD_BYTES;
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        slot[WORST_CASE_OCCUPANCY_OFFSET..WORST_CASE_OCCUPANCY_OFFSET + 8]
            .copy_from_slice(&recorded.to_le_bytes());
    });
    assert_eq!(
        refused_by_both_mounts_before_any_write(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalWorstCaseOccupancyNotTheInFlightLimitTimesTheRecordBytes {
            recorded_worst_case_occupancy_bytes: recorded,
            recomputed_worst_case_occupancy_bytes: recomputed,
        }
    );
}
