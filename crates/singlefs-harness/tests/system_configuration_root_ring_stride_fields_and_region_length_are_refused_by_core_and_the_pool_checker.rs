//! 根环区域间距那三判（D22（单元原子性怎么合成） 已定项 16 第 1 句、I-7.13，m3-close-c506 三轮判决第三节，用户 2026-10-22 定
//! 「四件照写回并实现」）：系统配置里的区域步长 P（偏移 363）与 chunk（偏移 367）先判等于第一版常量（3、1 MiB），再判
//! S × 固定结构槽距 ≤ P × chunk（写 ≤ 收）。越界整池拒：core 择系统配置与可写挂载都拒、盘上逐字节不变，池级 checker 的 `geometry_of`
//! 拒同一槽、报的成员与 core 一一对得上。三盘各一条用例：
//! 一、第一轮攻方那一盘（`research/prompts/m3-close-c506-r1-opus-model/` 的造法）：mkfs 按 S = 4、槽距 512 KiB 建池（4 × 512 KiB = 2 MiB，收），
//!     再把四个系统配置槽里的 S 改成 16（区间里），16 × 512 KiB = 8 MiB 越过区域间距 3 MiB；
//! 二、P 改成 0；三、chunk 改成 4 MiB。
//! 偏移与常量用例自己写死，不从实现里取：取了的话，实现把字段写到别处去这几条也跟着走。

use singlefs_checker::image::geometry_of;
use singlefs_checker::{check_system_configuration_slot, Verdict};
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
use singlefs_harness::memory_pool::SparseBlockDevice;

const IMAGE_BYTES: u64 = 4 << 30;
const PHYSICAL_BLOCK_SIZE: u32 = 512;
/// 第一轮攻方那一盘的 io_min：槽距 = 4096 向上取整到它的整数倍 = 512 KiB。
const FIVE_HUNDRED_TWELVE_KIBIBYTES: u32 = 512 << 10;
/// 区域间距 P × chunk = 3 × 1 MiB（第一版常量，用例自己写一份）。
const REGION_STRIDE_BYTES: u64 = 3 << 20;
/// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）：S 在 362、P 在 363（4 字节）、chunk 在 367（4 字节）。
const SLOTS_PER_REGION_OFFSET_IN_THE_SLOT: usize = 362;
const PRIME_STEP_OFFSET_IN_THE_SLOT: usize = 363;
const CHUNK_BYTES_OFFSET_IN_THE_SLOT: usize = 367;

fn parameters_with(
    slots_per_region: u64,
    minimum_input_output_bytes: u32,
) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"singlefs-stride!",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_SIZE,
            minimum_input_output_bytes,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(
                minimum_input_output_bytes,
            ),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                slots_per_region,
            )
            .expect("用例给 mkfs 的都是区间里的值"),
        },
    }
}

/// 照 `parameters` 建池（显式给 S 的入口），再把两块盘四个系统配置槽里 `offset_in_the_slot` 起的几个字节改成 `bytes`、
/// 整槽校验和重封：改出来的槽 magic、校验和、incompat 三关照样过，拒它的只能是读者收不收那几判。
fn pool_with_every_system_configuration_slot_changed(
    parameters: &MakeFilesystemParameters,
    offset_in_the_slot: usize,
    bytes: &[u8],
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE)),
            )
        })
        .collect();
    make_filesystem(parameters, &mut devices).expect("mkfs 收这一份几何");
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    for (_, device) in &mut devices {
        for offset in [0, spacing] {
            let mut slot = vec![0u8; slot_bytes];
            device
                .read_at(DeviceOffsetInBytes(offset), &mut slot)
                .expect("系统配置槽在盘内");
            slot[offset_in_the_slot..offset_in_the_slot + bytes.len()].copy_from_slice(bytes);
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

/// core 两条读路（择系统配置、可写挂载）都整池拒成 `expected_by_core`、可写挂载之后盘上逐字节不变；池级 checker 读盘 0 槽 0 拒成
/// `expected_by_the_pool_checker`。两边各一份判法、不共用代码（D13（验证路线） 已定项 5），这里核它们在同一份盘上报对得上的成员。
fn assert_core_and_the_pool_checker_refuse_naming(
    parameters: &MakeFilesystemParameters,
    mut devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    expected_by_core: SystemConfigurationValueOutsideWhatThisReaderAccepts,
    expected_by_the_pool_checker: Verdict,
) {
    let chosen = choose_system_configuration(&devices);
    assert!(
        matches!(
            &chosen,
            Err(RecoveryFailure::SystemConfigurationValueRefused { value, .. }) if *value == expected_by_core
        ),
        "core 择系统配置要整池拒成 {expected_by_core:?}，实际是 {chosen:?}"
    );
    let before = front_of_every_device(&devices);
    let mounted = mount_writable(parameters, &mut devices);
    assert!(
        matches!(
            &mounted,
            Err(MountError::Recovery(RecoveryFailure::SystemConfigurationValueRefused { value, .. })) if *value == expected_by_core
        ),
        "可写挂载要整池拒成 {expected_by_core:?}，实际是 {:?}",
        mounted.err()
    );
    assert_eq!(
        front_of_every_device(&devices),
        before,
        "拒绝挂载在任何写之前：盘上逐字节不变"
    );
    let mut slot = vec![0u8; usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")];
    devices[0]
        .1
        .read_at(DeviceOffsetInBytes(0), &mut slot)
        .expect("盘 0 槽 0");
    let view = check_system_configuration_slot(&slot).expect("改完的槽自证得过");
    assert_eq!(
        geometry_of(&slot, &view).err(),
        Some(expected_by_the_pool_checker),
        "池级 checker 拒同一槽、报与 core 对得上的那一格"
    );
}

/// 一、第一轮那一盘：S = 4、槽距 512 KiB 的池（mkfs 收）改成自述 S = 16。S 在区间里、P 与 chunk 都是第一版常量，
/// 拒它的只有 16 × 512 KiB = 8 MiB 大于 3 MiB 那一判。
#[test]
fn sixteen_slots_at_a_512_kibibyte_spacing_overrun_the_region_stride_and_are_refused_by_core_and_the_pool_checker(
) {
    let parameters = parameters_with(4, FIVE_HUNDRED_TWELVE_KIBIBYTES);
    assert_eq!(
        parameters.geometry.fixed_structure_slot_spacing, FIVE_HUNDRED_TWELVE_KIBIBYTES,
        "前提：io_min 512 KiB 的槽距就是 512 KiB"
    );
    let devices = pool_with_every_system_configuration_slot_changed(
        &parameters,
        SLOTS_PER_REGION_OFFSET_IN_THE_SLOT,
        &[16],
    );
    assert_core_and_the_pool_checker_refuse_naming(
        &parameters,
        devices,
        SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingRegionLongerThanTheRegionStride {
            slots_per_region: 16,
            fixed_structure_slot_spacing: FIVE_HUNDRED_TWELVE_KIBIBYTES,
            region_stride_bytes: REGION_STRIDE_BYTES,
        },
        Verdict::RootRingRegionLongerThanTheRegionStride,
    );
}

/// 二、P 改成 0（S = 8、槽距 4096 的池）：P 不等于第一版常量 3，整池拒。
#[test]
fn a_root_ring_prime_step_of_zero_is_refused_by_core_and_the_pool_checker() {
    let parameters = parameters_with(8, 512);
    let devices = pool_with_every_system_configuration_slot_changed(
        &parameters,
        PRIME_STEP_OFFSET_IN_THE_SLOT,
        &0u32.to_le_bytes(),
    );
    assert_core_and_the_pool_checker_refuse_naming(
        &parameters,
        devices,
        SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingPrimeStepNotTheFirstVersionValue {
            recorded_root_ring_prime_step: 0,
            first_version_root_ring_prime_step: 3,
        },
        Verdict::RootRingPrimeStepNotTheFirstVersionConstant,
    );
}

/// 三、chunk 改成 4 MiB（S = 8、槽距 4096 的池）：chunk 不等于第一版常量 1 MiB，整池拒。
#[test]
fn a_root_ring_chunk_of_four_mebibytes_is_refused_by_core_and_the_pool_checker() {
    let parameters = parameters_with(8, 512);
    let devices = pool_with_every_system_configuration_slot_changed(
        &parameters,
        CHUNK_BYTES_OFFSET_IN_THE_SLOT,
        &(4u32 << 20).to_le_bytes(),
    );
    assert_core_and_the_pool_checker_refuse_naming(
        &parameters,
        devices,
        SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingChunkBytesNotTheFirstVersionValue {
            recorded_root_ring_chunk_bytes: 4 << 20,
            first_version_root_ring_chunk_bytes: 1 << 20,
        },
        Verdict::RootRingChunkBytesNotTheFirstVersionConstant,
    );
}
