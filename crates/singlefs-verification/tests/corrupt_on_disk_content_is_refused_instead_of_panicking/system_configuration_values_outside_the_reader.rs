//! 读者不收的系统配置取值：格式版本、加密类型、物理块大小、固定结构槽间距、日志环长度。

use super::*;

// ─── 代码审阅第 29、38 条：系统配置里的池级不可变字段读出来先判 ───

/// 系统配置槽里几样字段的偏移（`SystemConfiguration::to_slot` 的写法）：格式版本紧跟 magic；加密类型在整槽校验和 32、MAC 16、nonce 水位 12、
/// KDF 标识 4 之后，紧跟着的是 MAC 长度声明 16；`physical_block_size`、journal 环长、固定结构槽距三个用例改完之后按 `parse_slot` 读回核。
const FORMAT_VERSION_OFFSET: usize = 4;

const ENCRYPTION_TYPE_OFFSET: usize = SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32 + 16 + 12 + 4;

const PHYSICAL_BLOCK_SIZE_OFFSET: usize = 317;

const JOURNAL_RING_BYTES_OFFSET: usize = 333;

const FIXED_STRUCTURE_SLOT_SPACING_OFFSET: usize = 429;

/// 两块盘各两个系统配置槽（偏移 0 与槽距 4096）逐个改一下、整槽校验和重封：池级字段两盘四槽同值，改就四槽一起改。
fn rewrite_every_system_configuration_slot(devices: &mut Devices, change: impl Fn(&mut [u8])) {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for offset in [0, spacing] {
            let mut slot = read_bytes(devices, device, DeviceOffsetInBytes(offset), slot_bytes);
            assert_eq!(
                (
                    &slot[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2],
                    slot[ENCRYPTION_TYPE_OFFSET],
                    slot[ENCRYPTION_TYPE_OFFSET + 1]
                ),
                (&[1u8, 0][..], 0, 16),
                "偏移对得上字段表：格式版本 1、加密类型 0、其后 MAC 长度声明 16"
            );
            change(&mut slot);
            let digest = wide_checksum_with_field_zeroed(
                &slot,
                slot_bytes,
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            device_mut(devices, device)
                .image
                .write(DeviceOffsetInBytes(offset), &slot);
        }
    }
}

/// 同一份镜像先给只读恢复（不带调用方参数，只看盘上择到的那份系统配置），再给可写挂载：两处都在读系统配置那一步拒、报同一个成员，
/// 可写挂载在任何写之前拒。交回被拒的那个值。
fn system_configuration_value_refused(
    devices: &mut Devices,
) -> SystemConfigurationValueOutsideWhatThisReaderAccepts {
    let report = recover(&*devices, JournalPolicy::Consult);
    let RecoveryOutcome::Failed {
        failure:
            RecoveryFailure::SystemConfigurationValueRefused {
                device: DeviceIdentity(0),
                value,
            },
        ..
    } = report.outcome
    else {
        panic!(
            "只读恢复要在读系统配置那一步拒（盘 0 先读到的那一槽）：{:?}",
            report.outcome
        );
    };
    let refusal = writable_mount_refused_before_any_write(devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::SystemConfigurationValueRefused {
                device: DeviceIdentity(0),
                value: refused_by_the_mount,
            }) if refused_by_the_mount == value
        ),
        "可写挂载与只读恢复报同一个成员：{refusal:?}"
    );
    value
}

fn write_u32_at(slot: &mut [u8], offset: usize, value: u32) {
    slot[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// 场景：四个系统配置槽的格式版本写成 2，整槽校验和重封。预期：可写挂载在读系统配置那一步整池拒（格式版本不认识），盘上逐字节不变。
/// 读者若不看这 2 字节：只读恢复按第 1 版的字段表读下去，可写挂载照常挂上。
#[test]
fn a_system_configuration_of_an_unrecognized_format_version_is_refused() {
    let mut devices = formatted_pool();
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        slot[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2].copy_from_slice(&2u16.to_le_bytes());
    });
    assert_eq!(
        system_configuration_value_refused(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::FormatVersionNotRecognized {
            format_version: 2
        }
    );
}

/// 场景：四个系统配置槽的加密类型写成 1（登记表里的 ChaCha20-Poly1305），整槽校验和重封。
/// 预期：整池拒（加密不进第一个可运行版本，D9（加密） 已定项 10），盘上逐字节不变。读者若不看这 1 字节：只读恢复按明文读下去，可写挂载照常挂上。
#[test]
fn a_system_configuration_whose_encryption_type_is_not_off_is_refused() {
    let mut devices = formatted_pool();
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        slot[ENCRYPTION_TYPE_OFFSET] = 1;
    });
    assert_eq!(
        system_configuration_value_refused(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::EncryptionTypeNotOff {
            encryption_type: 1
        }
    );
}

/// 场景：四个系统配置槽的 `physical_block_size` 写成 256（装不下 466 字节的根记录），整槽校验和重封。
/// 预期：整池拒（根槽宽下界），盘上逐字节不变。否则只读恢复按 256 字节读根槽、一条根都自证不过，报「根环里一条根都没有」；
/// 可写挂载因为调用方给的 512 与盘上不一致报 `CallerParametersDisagreeWithTheSelectedSystemConfiguration`——只读那条路没有调用方参数挡着。
#[test]
fn a_physical_block_size_narrower_than_the_root_record_is_refused() {
    let mut devices = formatted_pool();
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        write_u32_at(slot, PHYSICAL_BLOCK_SIZE_OFFSET, 256)
    });
    assert_eq!(
        system_configuration_value_refused(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::PhysicalBlockSizeOutsideTheRootSlotBounds { physical_block_size: 256 }
    );
}

/// 场景：四个系统配置槽的 `physical_block_size` 写成 8192（大过固定结构槽距 4096，一个根槽盖到下一个槽上），整槽校验和重封。
/// 预期：整池拒（根槽宽上界是槽距），盘上逐字节不变。否则只读恢复按 8192 字节开缓冲读根槽（缓冲随这 4 字节开）、一条根都自证不过；可写挂载报调用方参数不一致。
#[test]
fn a_physical_block_size_wider_than_the_fixed_structure_slot_spacing_is_refused() {
    let mut devices = formatted_pool();
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        write_u32_at(slot, PHYSICAL_BLOCK_SIZE_OFFSET, 8192)
    });
    let refused_slot = read_bytes(&devices, DeviceIdentity(0), DeviceOffsetInBytes(0), 4096);
    assert_eq!(
        SystemConfiguration::parse_slot(&refused_slot)
            .expect("整槽校验和重封过")
            .immutable
            .sizes
            .physical_block_size,
        8192,
        "偏移 317 就是 physical_block_size"
    );
    assert_eq!(
        system_configuration_value_refused(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::PhysicalBlockSizeOutsideTheRootSlotBounds { physical_block_size: 8192 }
    );
}

/// 场景：四个系统配置槽的固定结构槽距写成 2 MiB（槽 1 越过根环基址 1 MiB），整槽校验和重封。
/// 预期：整池拒（槽距在格式允许的区间之外），盘上逐字节不变。否则只读恢复按 2 MiB 的槽距找槽 1 与第 1 号起的根槽（落到根环区域里别的地方）；可写挂载报调用方参数不一致。
#[test]
fn a_fixed_structure_slot_spacing_outside_the_format_range_is_refused() {
    let mut devices = formatted_pool();
    let spacing = u32::try_from(2 * MEBIBYTE).expect("2 MiB 装得进 u32");
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        write_u32_at(slot, FIXED_STRUCTURE_SLOT_SPACING_OFFSET, spacing)
    });
    let refused_slot = read_bytes(&devices, DeviceIdentity(0), DeviceOffsetInBytes(0), 4096);
    assert_eq!(
        SystemConfiguration::parse_slot(&refused_slot)
            .expect("整槽校验和重封过")
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
        spacing,
        "偏移 429 就是固定结构槽距"
    );
    assert_eq!(
        system_configuration_value_refused(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::FixedStructureSlotSpacingOutsideTheFormatRange {
            fixed_structure_slot_spacing: spacing,
        }
    );
}

/// 场景：四个系统配置槽的 journal 环长写成 1000 字节（一条 4096 的记录都装不下），整槽校验和重封。
/// 预期：整池拒（环装不下 F 条记录），盘上逐字节不变。否则只读恢复扫 0 个环槽、照常走到第 0 代根；可写挂载报调用方参数不一致。
/// 环槽数 0 时 `journal::record_offset` 取模的那句 expect，读者一侧靠的就是这一道。
#[test]
fn a_journal_ring_too_short_for_the_safety_factor_is_refused_before_any_write() {
    let mut devices = formatted_pool();
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        slot[JOURNAL_RING_BYTES_OFFSET..JOURNAL_RING_BYTES_OFFSET + 8]
            .copy_from_slice(&1000u64.to_le_bytes());
    });
    let refused_slot = read_bytes(&devices, DeviceIdentity(0), DeviceOffsetInBytes(0), 4096);
    assert_eq!(
        SystemConfiguration::parse_slot(&refused_slot)
            .expect("整槽校验和重封过")
            .immutable
            .sizes
            .journal_ring_bytes,
        1000,
        "偏移 333 就是 journal 环长"
    );
    assert_eq!(
        system_configuration_value_refused(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingBytesOutsideTheSupportedRange { journal_ring_bytes: 1000 }
    );
}

/// 场景：四个系统配置槽的 journal 环长写成 2 GiB（末端越过编译期的单元区起点 784 MiB，C475），整槽校验和重封。
/// 预期：整池拒，盘上逐字节不变。否则只读恢复把单元区当环扫（按环长一次列出 2 GiB ÷ 4096 个偏移逐个读）；可写挂载报调用方参数不一致。
#[test]
fn a_journal_ring_reaching_past_the_unit_area_start_is_refused() {
    let mut devices = formatted_pool();
    let ring_bytes: u64 = 2 << 30;
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        slot[JOURNAL_RING_BYTES_OFFSET..JOURNAL_RING_BYTES_OFFSET + 8]
            .copy_from_slice(&ring_bytes.to_le_bytes());
    });
    assert_eq!(
        system_configuration_value_refused(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingBytesOutsideTheSupportedRange {
            journal_ring_bytes: ring_bytes,
        }
    );
}
