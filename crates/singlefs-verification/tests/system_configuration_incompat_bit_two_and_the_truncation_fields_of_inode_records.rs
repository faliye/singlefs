//! 系统配置 incompat 位 2「inode 记录带截断字段」（新开一位，`research/prompts/m4-trunc-r2-main-verification.md` 第六节）
//! 是这一版布局身份的一部分：mkfs 与之后每一次系统配置写都带位 1 与位 2；只带位 1 的池
//! （第一个字节 0x02）不是这一版布局，只读挂载与恢复都在读系统配置那一步拒成「系统配置的 incompat 位图不认识」
//! （可写挂载那一条在 `system_configuration_rollback_floor_and_layout_identity.rs`）。
mod common;

use common::{build_pool, parameters};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::mounted_read::{mount_read_only, MountReadOnlyFailure};
use singlefs_core::recovery::{
    choose_system_configuration, recover, JournalPolicy, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::system_configuration::{
    IncompatBitmap, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET, SYSTEM_CONFIGURATION_MAGIC,
};
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_verification::memory_pool::MemoryPool;

/// incompat 位图第一个字节在系统配置槽里的偏移（magic 4 + 格式版本 2）。
const FIRST_INCOMPAT_BYTE_OFFSET: usize = 6;
/// 只带布局身份位 1、不带位 2 的第一个字节。
const FIRST_INCOMPAT_BYTE_WITHOUT_BIT_TWO: u8 = 0x02;
/// 位 1 与位 2 都带的第一个字节（mkfs 写的那一个）。
const FIRST_INCOMPAT_BYTE_WITH_BIT_TWO: u8 = 0x06;

fn slot_bytes() -> usize {
    usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")
}

/// 每块盘两个系统配置槽（magic 对得上的那几个）的 incompat 第一个字节，按 (盘, 槽号) 排。
fn first_incompat_bytes_of_every_system_configuration_slot(
    image: &MemoryPool,
) -> Vec<(u32, u64, u8)> {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let mut bytes = Vec::new();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for slot in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
            let read =
                image.devices[&device].read(DeviceOffsetInBytes(slot * spacing), slot_bytes());
            if read[..4] == SYSTEM_CONFIGURATION_MAGIC {
                bytes.push((device.0, slot, read[FIRST_INCOMPAT_BYTE_OFFSET]));
            }
        }
    }
    bytes
}

/// 每块盘两个系统配置槽（magic 对得上的那几个）的 incompat 第一个字节改成 `first_incompat_byte`，整槽校验和重封：别的字节一个不动。
fn reseal_every_system_configuration_slot_with_the_first_incompat_byte(
    image: &mut MemoryPool,
    first_incompat_byte: u8,
) {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for slot in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
            let offset = DeviceOffsetInBytes(slot * spacing);
            let mut bytes = image.devices[&device].read(offset, slot_bytes());
            if bytes[..4] != SYSTEM_CONFIGURATION_MAGIC {
                continue;
            }
            bytes[FIRST_INCOMPAT_BYTE_OFFSET] = first_incompat_byte;
            let digest = wide_checksum_with_field_zeroed(
                &bytes,
                slot_bytes(),
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            bytes[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            image
                .devices
                .get_mut(&device)
                .expect("有这块盘")
                .write(offset, &bytes);
        }
    }
}

/// 新池新建文件之后的池，每个系统配置槽重封成只带位 1、不带位 2（第一个字节 0x02），别的字节一个不动。
fn image_whose_system_configuration_lacks_bit_two(tag: &str) -> MemoryPool {
    let mut image = build_pool(tag).memory_pool();
    reseal_every_system_configuration_slot_with_the_first_incompat_byte(
        &mut image,
        FIRST_INCOMPAT_BYTE_WITHOUT_BIT_TWO,
    );
    image
}

/// 「系统配置的 incompat 位图不认识」点名池里第一块盘（盘 0）、带着它槽里的位图：第一个字节 0x02，其余照 mkfs 写的全 0。
fn assert_the_refusal_names_device_zero_and_a_bitmap_without_bit_two(
    failure: &RecoveryFailure,
    what: &str,
) {
    let mut expected_bitmap = [0u8; 32];
    expected_bitmap[0] = FIRST_INCOMPAT_BYTE_WITHOUT_BIT_TWO;
    assert_eq!(
        failure,
        &RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized {
            first_device_carrying_them: DeviceIdentity(0),
            incompat_bitmap: IncompatBitmap(expected_bitmap),
        },
        "{what}：拒成系统配置的 incompat 位图不认识"
    );
}

/// mkfs 写下的每个系统配置槽（两块盘、各两槽里 magic 对得上的那几个）第一个字节都是 0x06，读者择得到；
/// 新池新建文件之后写者轮换写的槽也是 0x06。
#[test]
fn make_filesystem_and_every_later_system_configuration_write_carry_incompat_bit_two() {
    let pool = build_pool("incompat-bit-two-at-mkfs");
    let after_mkfs = pool.memory_pool_after_mkfs();
    let at_mkfs = first_incompat_bytes_of_every_system_configuration_slot(&after_mkfs);
    assert!(!at_mkfs.is_empty(), "前提：mkfs 写了系统配置槽");
    assert!(
        at_mkfs
            .iter()
            .all(|(_, _, byte)| *byte == FIRST_INCOMPAT_BYTE_WITH_BIT_TWO),
        "mkfs 写的每一槽都带位 1 与位 2：{at_mkfs:?}"
    );
    assert!(
        choose_system_configuration(&after_mkfs).is_ok(),
        "mkfs 写的系统配置择得到"
    );
    let after_the_first_file =
        first_incompat_bytes_of_every_system_configuration_slot(&pool.memory_pool());
    assert!(
        after_the_first_file
            .iter()
            .all(|(_, _, byte)| *byte == FIRST_INCOMPAT_BYTE_WITH_BIT_TWO),
        "取号、暖机、新池新建文件写的槽也带位 2：{after_the_first_file:?}"
    );
}

/// 只带位 1 的池：只读挂载在读系统配置那一步拒成 `Recovery(SystemConfigurationIncompatBitsNotRecognized)`。
#[test]
fn mounting_read_only_a_pool_without_bit_two_is_refused_as_an_unrecognized_layout() {
    let image = image_whose_system_configuration_lacks_bit_two("incompat-bit-two-read-only");
    match mount_read_only(&image) {
        Err(MountReadOnlyFailure::Recovery(failure)) => {
            assert_the_refusal_names_device_zero_and_a_bitmap_without_bit_two(&failure, "只读挂载");
        }
        Err(other) => panic!("只读挂载该在读系统配置那一步拒，实际 {other:?}"),
        Ok(_mounted) => panic!("只带位 1 的池被只读挂载收下了"),
    }
}

/// 同一形池，冷启动恢复在读系统配置那一步失败、报同一个错，不择根。
#[test]
fn recovering_a_pool_without_bit_two_fails_as_an_unrecognized_layout() {
    let image = image_whose_system_configuration_lacks_bit_two("incompat-bit-two-recovery");
    match recover(&image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::Failed { root, failure } => {
            assert_eq!(root, None, "读系统配置那一步就失败，没有择根");
            assert_the_refusal_names_device_zero_and_a_bitmap_without_bit_two(&failure, "恢复");
        }
        RecoveryOutcome::FileRead { .. } | RecoveryOutcome::NoFile { .. } => {
            panic!("只带位 1 的池被恢复收下了")
        }
    }
}
