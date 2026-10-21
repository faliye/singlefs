//! C663（系统配置槽越界与不收之值两读者分类不同）core 一侧（用户 2026-10-20 定「S 照条款整池拒，R 读并整池拒」与「格式版本先」）：
//! 根环区域数 R（偏移 361 那 1 字节）读出来，不等第一版常量 3 整池拒，与根环起点、journal 环起点同一类；同一槽里格式版本不是 1、
//! 又有不认得的 incompat 位时，先按格式版本整池拒（不再当 incompat 不认得的槽跳过、择别的槽）。单元区 384 槽那一档的两块内存小盘。
mod common_admission_pool;

use common_admission_pool::{
    device_bytes_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices,
    plain_device, plain_devices_on, PoolUnderTest,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
use singlefs_core::recovery::{
    choose_system_configuration, RecoveryFailure,
    SystemConfigurationValueOutsideWhatThisReaderAccepts,
};
use singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET;
use singlefs_format::{ROOT_RING_REGIONS, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

/// 系统配置槽里根环区域数 R 那一字节的偏移（字段表 `layout/01-first-txn.md` 一）。
const ROOT_RING_REGION_COUNT_OFFSET: usize = 361;
/// 系统配置槽里格式版本那 2 字节的偏移：紧跟 4 字节 magic。
const FORMAT_VERSION_OFFSET: usize = 4;
/// incompat 位图第二个字节的偏移：magic 4 + 格式版本 2 之后是位图，位 8 住第二个字节的最低位（这个读者不认得的位）。
const SECOND_INCOMPAT_BYTE_OFFSET: usize = 7;

/// 第一个文件之后的盘面，盘 0 槽 0 照 `change` 改几个字节、整槽校验和重封（读得出、自证得过）。
fn image_with_slot_zero_of_device_zero_changed(change: impl Fn(&mut [u8])) -> MemoryPool {
    let mut image = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        plain_device,
    )
    .image();
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let device_zero = image
        .devices
        .get_mut(&DeviceIdentity(0))
        .expect("池里有盘 0");
    let mut slot = device_zero.read(DeviceOffsetInBytes(0), slot_bytes);
    change(&mut slot);
    let digest =
        wide_checksum_with_field_zeroed(&slot, slot_bytes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
        .copy_from_slice(&digest);
    device_zero.write(DeviceOffsetInBytes(0), &slot);
    image
}

/// 可写挂载在这份盘面上拒成 `Recovery(expected)`、盘上逐字节不变。
fn assert_the_writable_mount_is_refused_before_any_write(
    image: &MemoryPool,
    expected: &RecoveryFailure,
) {
    let mut devices = plain_devices_on(image);
    let refusal = mount_writable_with_space_admission(
        &parameters_of_the_384_slot_small_devices(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect_err("系统配置槽带这个读者不收的值：整池拒");
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if failure == expected),
        "可写挂载拒成 Recovery({expected:?})，实际 {refusal:?}"
    );
    let after = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    };
    assert_eq!(&after, image, "拒在任何写之前：两块盘逐字节不变");
}

/// 盘 0 槽 0 的 R 写成 4：择系统配置整池拒，报 `RootRingRegionCountNotTheFirstVersionValue`（记的是 4、第一版是 3），可写挂载同拒、盘上不变。
/// 判别力：删掉 R 那一判（变异「C663 R 不读（区域数写成 4 的槽照常可择）」）时盘 0 槽 0 照常可择、挂载做成，这一条红。
#[test]
fn a_slot_recording_four_root_ring_regions_is_refused_by_the_whole_pool() {
    let image = image_with_slot_zero_of_device_zero_changed(|slot| {
        slot[ROOT_RING_REGION_COUNT_OFFSET] = 4;
    });
    let expected = RecoveryFailure::SystemConfigurationValueRefused {
        device: DeviceIdentity(0),
        value: SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingRegionCountNotTheFirstVersionValue {
            recorded_root_ring_region_count: 4,
            first_version_root_ring_region_count: ROOT_RING_REGIONS,
        },
    };
    assert_eq!(choose_system_configuration(&image), Err(expected.clone()));
    assert_the_writable_mount_is_refused_before_any_write(&image, &expected);
}

/// 盘 0 槽 0 的格式版本写成 2、incompat 位图里置一个这个读者不认得的位（位 8）：先按格式版本整池拒（报 `FormatVersionNotRecognized`），
/// 不当 incompat 不认得的槽跳过、去择盘 0 槽 1。判别力：删掉「格式版本先」那一判（变异「C663 格式版本不先于 incompat 判（格式版本 2 又带
/// 不认得的位的槽被跳过）」）时这一槽被跳过、择系统配置做成，这一条红。
#[test]
fn a_slot_with_another_format_version_and_unrecognized_incompat_bits_is_refused_by_the_format_version_first(
) {
    let image = image_with_slot_zero_of_device_zero_changed(|slot| {
        slot[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2].copy_from_slice(&2u16.to_le_bytes());
        slot[SECOND_INCOMPAT_BYTE_OFFSET] |= 0x01;
    });
    let expected = RecoveryFailure::SystemConfigurationValueRefused {
        device: DeviceIdentity(0),
        value: SystemConfigurationValueOutsideWhatThisReaderAccepts::FormatVersionNotRecognized {
            format_version: 2,
        },
    };
    assert_eq!(choose_system_configuration(&image), Err(expected.clone()));
    assert_the_writable_mount_is_refused_before_any_write(&image, &expected);
}
