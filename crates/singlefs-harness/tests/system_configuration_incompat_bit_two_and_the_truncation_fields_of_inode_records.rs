//! 系统配置 incompat 位 2「inode 记录带截断字段」（用户 2026-10-17 定新开一位，判决 `research/prompts/m4-trunc-r2-main-verification.md` 第六节；
//! 主 agent 定 mkfs 写这一位）：mkfs 起就置上；读者收第一个字节 0x02 与 0x06 两种；不带位 2 的池里读到截断字段非零的 inode 记录判不可用，
//! 只读挂载、恢复、可写挂载三处都拒成类型化的错，可写挂载在任何写之前拒。不带位 2 而记录全零的池照常挂上，写者再写系统配置时写回位 2。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_truncation;

use common::{build_pool, parameters};
use common_deletion_in_progress::pool_with_a_three_unit_file;
use common_pointer_head_corruption::{
    recovery_failure, writable_devices_of, writable_mount_refused_before_any_write,
};
use common_truncation::change_the_size_of_the_first_file_and_advance;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::mounted_read::{mount_read_only, MountReadOnlyFailure, OpenPoolForReadFailure};
use singlefs_core::recovery::{
    choose_system_configuration, recover, JournalPolicy, RecoveryFailure, RecoveryOutcome,
    INODE_RECORD_TRUNCATION_FIELD_IN_A_POOL_WITHOUT_INCOMPAT_BIT_TWO,
};
use singlefs_core::system_configuration::{
    InodeRecordTruncationFieldsOfThePool, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
    SYSTEM_CONFIGURATION_MAGIC,
};
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

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

/// 三个单元截到 100 的意图那一版（第一个文件那条记录的截断字段是（5, 100）），每个系统配置槽重封成不带位 2。
fn image_without_bit_two_whose_first_file_record_carries_a_truncation_field(
    tag: &str,
) -> MemoryPool {
    let mut pool = pool_with_a_three_unit_file(tag);
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, 100);
    assert!(
        intent
            .inode_record
            .expect("第一个文件那条记录在")
            .truncation_field
            .is_written(),
        "前提：意图那次写了截断字段"
    );
    let mut image = pool.memory_pool();
    reseal_every_system_configuration_slot_with_the_first_incompat_byte(
        &mut image,
        FIRST_INCOMPAT_BYTE_WITHOUT_BIT_TWO,
    );
    assert_eq!(
        choose_system_configuration(&image)
            .map(|chosen| chosen.immutable.inode_record_truncation_fields),
        Ok(InodeRecordTruncationFieldsOfThePool::AllZero),
        "前提：择到的系统配置不带位 2（读者收 0x02）"
    );
    image
}

/// mkfs 写下的每个系统配置槽（两块盘、各两槽里 magic 对得上的那几个）第一个字节都是 0x06，择到的那一份读成带截断字段；
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
    assert_eq!(
        choose_system_configuration(&after_mkfs)
            .map(|chosen| chosen.immutable.inode_record_truncation_fields),
        Ok(InodeRecordTruncationFieldsOfThePool::Carried)
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

/// 不带位 2 的池里第一个文件那条记录带截断字段：只读挂载拒成
/// `Open(InodeRecordCarriesATruncationFieldInAPoolWithoutIncompatBitTwo { inode: 1 })`。
#[test]
fn mounting_read_only_a_pool_without_bit_two_whose_record_carries_a_truncation_field_is_refused() {
    let image = image_without_bit_two_whose_first_file_record_carries_a_truncation_field(
        "incompat-bit-two-read-only",
    );
    match mount_read_only(&image) {
        Err(refusal) => assert_eq!(
            refusal,
            MountReadOnlyFailure::Open(
                OpenPoolForReadFailure::InodeRecordCarriesATruncationFieldInAPoolWithoutIncompatBitTwo {
                    inode: FIRST_INODE_NUMBER
                }
            )
        ),
        Ok(_mounted) => panic!("不带位 2 而记录带截断字段的池被只读挂载收下了"),
    }
}

/// 同一份镜像，冷启动恢复走读拒成 I-9.7、带那一句说明。
#[test]
fn recovering_a_pool_without_bit_two_whose_record_carries_a_truncation_field_fails_on_that_record()
{
    let image = image_without_bit_two_whose_first_file_record_carries_a_truncation_field(
        "incompat-bit-two-recovery",
    );
    assert_eq!(
        recovery_failure(&image),
        RecoveryFailure::InvariantViolated {
            invariant: "I-9.7",
            detail: INODE_RECORD_TRUNCATION_FIELD_IN_A_POOL_WITHOUT_INCOMPAT_BIT_TWO,
        }
    );
}

/// 同一份镜像，可写挂载在任何写之前拒（盘上逐字节不变），拒成建上一版那一步的 I-9.7。
#[test]
fn mounting_writable_a_pool_without_bit_two_whose_record_carries_a_truncation_field_is_refused_before_any_write(
) {
    let image = image_without_bit_two_whose_first_file_record_carries_a_truncation_field(
        "incompat-bit-two-writable",
    );
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-9.7",
                detail,
            }) if detail == INODE_RECORD_TRUNCATION_FIELD_IN_A_POOL_WITHOUT_INCOMPAT_BIT_TWO
        ),
        "拒成建上一版那一步的 I-9.7，实际 {refusal:?}"
    );
}

/// 对照：不带位 2、记录的截断字段全零（只做过新池新建文件）的池照常挂上——只读挂载与恢复读回第一个文件；可写挂载也挂得上，
/// 它取号那一写起写的系统配置带回位 2（写者每次写系统配置都写位 2，`InodeRecordTruncationFieldsOfThePool::WRITTEN_BY_THIS_WRITER`）。
#[test]
fn a_pool_without_bit_two_whose_records_carry_no_truncation_field_mounts_and_the_writable_mount_writes_bit_two_back(
) {
    let pool = build_pool("incompat-bit-two-control");
    let mut image = pool.memory_pool();
    reseal_every_system_configuration_slot_with_the_first_incompat_byte(
        &mut image,
        FIRST_INCOMPAT_BYTE_WITHOUT_BIT_TWO,
    );
    assert!(mount_read_only(&image).is_ok(), "只读挂载照常挂上");
    assert!(
        matches!(
            recover(&image, JournalPolicy::Consult).outcome,
            RecoveryOutcome::FileRead { .. }
        ),
        "恢复照常读回第一个文件"
    );
    let mut devices = writable_devices_of(&image);
    mount_writable(&parameters(), &mut devices).expect("可写挂载照常挂上");
    let mut after_the_mount = image.clone();
    for (identity, device) in &devices {
        *after_the_mount.devices.get_mut(identity).expect("有这块盘") = device.image.clone();
    }
    assert_eq!(
        choose_system_configuration(&after_the_mount)
            .map(|chosen| chosen.immutable.inode_record_truncation_fields),
        Ok(InodeRecordTruncationFieldsOfThePool::Carried),
        "可写挂载之后择到的系统配置带位 2"
    );
}
