//! 系统配置几何段的单元几何四格（D21（权威态与派生态的分界） 已定项 16「单元几何在挂载时求值一次」、已定项 8「N 住系统配置，第一版 N = 0」；C1041）：
//! 单元大小（偏移 305）、落点粒度（309）、位置条目宽（313）各等于格式常量，扩展点声明值 N（321）等于 0。任一格改一个值、整槽校验和重封，
//! core 择系统配置与可写挂载都整池拒成同一个成员、盘上逐字节不变；池级 checker 把每块盘每个槽读成「带这一版读者不收的值」、拒在同一格，
//! I-7.13（系统配置池级字段在读者收的范围里） 判红。四格各一条用例。
//! 偏移、常量与改成的值用例自己写死，不从实现里取：取了的话，实现把字段写到别处去这几条也跟着走。

use singlefs_checker::image::{
    judge_system_configuration_values_the_reader_accepts, system_configuration_slot_readings,
    ImageReader, Judgements, RefusedSystemConfigurationValue, SystemConfigurationSlotReading,
    UnitGeometryFieldThatDisagrees, SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::recovery::{
    choose_system_configuration, RecoveryFailure,
    SystemConfigurationValueOutsideWhatThisReaderAccepts,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{
    SystemImmutableSizes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_verification::memory_pool::SparseBlockDevice;

const IMAGE_BYTES: u64 = 4 << 30;
const PHYSICAL_BLOCK_SIZE: u32 = 512;
/// 系统配置槽 1 的偏移：io_min 不声明时固定结构槽距是 4096。
const SLOT_ONE_OFFSET: u64 = 4096;
/// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）：单元大小 305、落点粒度 309、位置条目宽 313、扩展点声明值 N 321，各 4 字节。
const UNIT_BYTES_OFFSET_IN_THE_SLOT: usize = 305;
const SLOT_GRANULARITY_BYTES_OFFSET_IN_THE_SLOT: usize = 309;
const LOCATION_ENTRY_BYTES_OFFSET_IN_THE_SLOT: usize = 313;
const EXTENSION_POINT_DECLARED_BYTES_OFFSET_IN_THE_SLOT: usize = 321;

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"singlefs-unitgeo",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_SIZE,
            minimum_input_output_bytes: 0,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(0),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// mkfs 之后把两块盘四个系统配置槽里 `offset_in_the_slot` 起的 4 字节改成 `rewritten_value`（小端）、整槽校验和重封：
/// 改出来的槽 magic、校验和、incompat 三关照样过，拒它的只能是读者收不收那几判。
fn pool_with_every_system_configuration_slot_carrying(
    offset_in_the_slot: usize,
    rewritten_value: u32,
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE)),
            )
        })
        .collect();
    make_filesystem(&parameters(), &mut devices).expect("mkfs 收这一份几何");
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for (_, device) in &mut devices {
        for offset in [0, SLOT_ONE_OFFSET] {
            let mut slot = vec![0u8; slot_bytes];
            device
                .read_at(DeviceOffsetInBytes(offset), &mut slot)
                .expect("系统配置槽在盘内");
            slot[offset_in_the_slot..offset_in_the_slot + 4]
                .copy_from_slice(&rewritten_value.to_le_bytes());
            let digest = wide_checksum_with_field_zeroed(
                &slot,
                slot_bytes,
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            device
                .write_at(DeviceOffsetInBytes(offset), &slot, WriteDurability::Plain)
                .expect("改完的槽写得回去");
        }
    }
    devices
}

/// 池级 checker 按字节读这几块盘：只给系统配置那几判用，扫描方向的候选一个都不给。
struct DevicesReadByThePoolChecker<'pool>(&'pool [(DeviceIdentity, SparseBlockDevice)]);

impl ImageReader for DevicesReadByThePoolChecker<'_> {
    fn devices(&self) -> Vec<u32> {
        self.0.iter().map(|(identity, _)| identity.0).collect()
    }
    fn device_bytes(&self, device: u32) -> Option<u64> {
        self.0
            .iter()
            .find(|(identity, _)| identity.0 == device)
            .map(|(_, sparse)| sparse.size_in_bytes())
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        let (_, sparse) = self.0.iter().find(|(identity, _)| identity.0 == device)?;
        let mut bytes = vec![0u8; length];
        sparse
            .read_at(DeviceOffsetInBytes(offset), &mut bytes)
            .ok()?;
        Some(bytes)
    }
    fn candidate_unit_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(Vec::new())
    }
    fn candidate_journal_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(Vec::new())
    }
}

/// 两块盘前 16 MiB（系统配置两槽 + 整条根环 + 一段余量）的字节：拒绝挂载那一格拿它比「盘上逐字节不变」。
fn front_of_every_device(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> Vec<Vec<u8>> {
    devices
        .iter()
        .map(|(_, device)| {
            let mut front = vec![0u8; 16 << 20];
            device
                .read_at(DeviceOffsetInBytes(0), &mut front)
                .expect("前 16 MiB 在盘内");
            front
        })
        .collect()
}

/// 四个槽的某一格改成 `rewritten_value` 之后：core 择系统配置与可写挂载都整池拒成 `expected_by_core`、可写挂载之后盘上逐字节不变；
/// 池级 checker 把两块盘四个槽都读成拒在 `expected_by_the_pool_checker` 那一格的不收之值，I-7.13 判红。
fn assert_core_and_the_pool_checker_refuse_the_field_rewritten_at(
    offset_in_the_slot: usize,
    rewritten_value: u32,
    expected_by_core: SystemConfigurationValueOutsideWhatThisReaderAccepts,
    expected_by_the_pool_checker: UnitGeometryFieldThatDisagrees,
) {
    let mut devices =
        pool_with_every_system_configuration_slot_carrying(offset_in_the_slot, rewritten_value);
    let chosen = choose_system_configuration(&devices);
    assert!(
        matches!(
            &chosen,
            Err(RecoveryFailure::SystemConfigurationValueRefused { value: refused_value, .. }) if *refused_value == expected_by_core
        ),
        "core 择系统配置要整池拒成 {expected_by_core:?}，实际是 {chosen:?}"
    );
    let before = front_of_every_device(&devices);
    let mounted = mount_writable(&parameters(), &mut devices);
    assert!(
        matches!(
            &mounted,
            Err(MountError::Recovery(RecoveryFailure::SystemConfigurationValueRefused { value: refused_value, .. })) if *refused_value == expected_by_core
        ),
        "可写挂载要整池拒成 {expected_by_core:?}，实际是 {:?}",
        mounted.err()
    );
    assert_eq!(
        front_of_every_device(&devices),
        before,
        "拒绝挂载在任何写之前：盘上逐字节不变"
    );
    let readings = system_configuration_slot_readings(&DevicesReadByThePoolChecker(&devices));
    assert_eq!(readings.len(), 2, "两块盘各两槽");
    for (device, slots) in &readings {
        for (slot_index, slot) in slots.iter().enumerate() {
            assert_eq!(
                slot,
                &SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
                    offset_in_bytes: if slot_index == 0 { 0 } else { SLOT_ONE_OFFSET },
                    refused: RefusedSystemConfigurationValue::UnitGeometryFieldDisagrees(
                        expected_by_the_pool_checker
                    ),
                },
                "池级 checker：盘 {device} 槽 {slot_index} 拒在与 core 对得上的那一格"
            );
        }
    }
    let mut judgements = Judgements::default();
    assert!(
        judge_system_configuration_values_the_reader_accepts(&readings, &mut judgements),
        "池级 checker 交回「有带越界值的槽」：整池不作保"
    );
    assert!(
        judgements
            .first_violation_of(SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS)
            .is_some(),
        "I-7.13 判红"
    );
}

/// 单元大小写成 65536（格式常量 32768 的两倍）。
#[test]
fn a_unit_size_other_than_the_format_constant_is_refused_by_core_and_the_pool_checker() {
    assert_core_and_the_pool_checker_refuse_the_field_rewritten_at(
        UNIT_BYTES_OFFSET_IN_THE_SLOT,
        65536,
        SystemConfigurationValueOutsideWhatThisReaderAccepts::UnitBytesNotTheFormatConstant {
            recorded_unit_bytes: 65536,
            format_unit_bytes: 32768,
        },
        UnitGeometryFieldThatDisagrees::UnitBytesNotTheFormatConstant,
    );
}

/// 落点粒度写成 4096（格式常量 16384 之外）。
#[test]
fn a_slot_granularity_other_than_the_format_constant_is_refused_by_core_and_the_pool_checker() {
    assert_core_and_the_pool_checker_refuse_the_field_rewritten_at(
        SLOT_GRANULARITY_BYTES_OFFSET_IN_THE_SLOT,
        4096,
        SystemConfigurationValueOutsideWhatThisReaderAccepts::SlotGranularityBytesNotTheFormatConstant {
            recorded_slot_granularity_bytes: 4096,
            format_slot_granularity_bytes: 16384,
        },
        UnitGeometryFieldThatDisagrees::SlotGranularityBytesNotTheFormatConstant,
    );
}

/// 位置条目宽写成 15（格式常量 14 加一）。
#[test]
fn a_location_entry_width_other_than_the_format_constant_is_refused_by_core_and_the_pool_checker() {
    assert_core_and_the_pool_checker_refuse_the_field_rewritten_at(
        LOCATION_ENTRY_BYTES_OFFSET_IN_THE_SLOT,
        15,
        SystemConfigurationValueOutsideWhatThisReaderAccepts::LocationEntryBytesNotTheFormatConstant {
            recorded_location_entry_bytes: 15,
            format_location_entry_bytes: 14,
        },
        UnitGeometryFieldThatDisagrees::LocationEntryBytesNotTheFormatConstant,
    );
}

/// 扩展点声明值 N 写成 1（第一版只收 0）。
#[test]
fn a_non_zero_extension_point_size_is_refused_by_core_and_the_pool_checker() {
    assert_core_and_the_pool_checker_refuse_the_field_rewritten_at(
        EXTENSION_POINT_DECLARED_BYTES_OFFSET_IN_THE_SLOT,
        1,
        SystemConfigurationValueOutsideWhatThisReaderAccepts::ExtensionPointDeclaredBytesNotTheFirstVersionValue {
            recorded_extension_point_declared_bytes: 1,
            first_version_extension_point_declared_bytes: 0,
        },
        UnitGeometryFieldThatDisagrees::ExtensionPointDeclaredBytesNotTheFirstVersionValue,
    );
}
