//! 回退改形态的格式那一半（2026-09-26 用户定案）落到盘上：
//! 系统配置字段表末尾加回退下界 F 8 字节（D22（单元原子性怎么合成） 已定项 9，偏移 481 起），
//! incompat 布局身份从位 0 换到位 1、位 0 退役（D15（格式冻结政策） 已定项 4），
//! 根记录 flags 位 0 当卸载记号（D22（单元原子性怎么合成） 已定项 7）。
//!
//! 这一份只钉格式与读写：mkfs 写出什么、读者交出什么、后来的系统配置写带不带着 F、旧镜像挂不挂得上、checker 认不认。
//! F 读出来之后怎么用（恢复时生效值取两处最大值、抬 F 先写系统配置）不在这里。

mod common;

use common::{format_pool, parameters, E142_FILESYSTEM_IDENTIFIER};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::recovery::{
    choose_system_configuration, verified_system_configuration_slots, PoolReader, RecoveryFailure,
};
use singlefs_core::root_record::{ROOT_CHECKSUM_OFFSET, ROOT_RECORD_FLAG_UNMOUNT_MARKER};
use singlefs_core::root_ring::{slot_offset, RootRingSlot};
use singlefs_core::system_configuration::{
    SystemConfiguration, INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT,
    INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT,
    SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_core::transaction::{acquire_instance, PoolWriter};
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;

type Recorded = common::Recorded;

/// 系统配置槽里 incompat 位图第一个字节的偏移：magic 4 + 格式版本 2。
const FIRST_INCOMPAT_BYTE_OFFSET: usize = 6;
/// 系统配置槽里回退下界 F 那 8 字节：字段表末尾（D22（单元原子性怎么合成） 已定项 9）。
const ROLLBACK_FLOOR_BYTES_IN_THE_SLOT: std::ops::Range<usize> = 481..489;
/// 根记录 flags 那 4 字节：magic 4 + fsid 16 之后。
const ROOT_FLAGS_BYTES: std::ops::Range<usize> = 20..24;

fn slot_bytes() -> usize {
    usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")
}

/// 一块盘上两个系统配置槽的盘内偏移：槽 0 在 0，槽 1 在槽距处（D22（单元原子性怎么合成） 已定项 16）。
fn system_configuration_slot_offsets() -> [DeviceOffsetInBytes; 2] {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    [DeviceOffsetInBytes(0), DeviceOffsetInBytes(spacing)]
}

fn read_slot(
    devices: &[(DeviceIdentity, Recorded)],
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
) -> Vec<u8> {
    PoolReader::read(devices, device, offset, slot_bytes()).expect("系统配置槽读得到")
}

/// 整槽校验和重封：改过字节的槽 magic 与校验和两关照样过，拒不拒只看被改的那一格。
fn reseal_system_configuration_slot(slot: &mut [u8]) {
    let digest =
        wide_checksum_with_field_zeroed(slot, slot_bytes(), SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
        .copy_from_slice(&digest);
}

/// 把一个系统配置槽里的 F 换成 `rollback_floor` 写回（经读者解开、改字段、写者重写，整槽校验和随之重封）；
/// 不经抬 F 的路径，只为造出「盘上系统配置里已经有一个非 0 的 F」这个状态。
fn plant_rollback_floor(
    devices: &mut [(DeviceIdentity, Recorded)],
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    rollback_floor: CheckpointTxg,
) {
    let mut system_configuration =
        SystemConfiguration::parse_slot(&read_slot(devices, device, offset))
            .expect("mkfs 写的槽自证得过");
    system_configuration.quantities.rollback_floor = rollback_floor;
    let (_, target) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("有这块盘");
    target
        .write_at(
            offset,
            &system_configuration.to_slot(),
            WriteDurability::Plain,
        )
        .expect("写回系统配置槽");
}

/// mkfs 写出的四个系统配置槽（两盘各两槽）：incompat 只带位 1、不带退役的位 0，F 那 8 字节是 0，
/// 核心层的读者交回 F = 0，独立实现的 checker 也认这一槽。
#[test]
fn make_filesystem_writes_rollback_floor_zero_and_only_the_new_layout_identity_bit_into_every_system_configuration_slot(
) {
    let mut pool = format_pool("rbf-mkfs-slots");
    let devices = pool.devices.take().expect("mkfs 之后设备还开着");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for offset in system_configuration_slot_offsets() {
            let slot = read_slot(&devices, device, offset);
            assert_eq!(
                slot[FIRST_INCOMPAT_BYTE_OFFSET],
                INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT,
                "盘 {} 偏移 {}：incompat 第一个字节只有位 1",
                device.0,
                offset.0
            );
            assert_eq!(
                slot[FIRST_INCOMPAT_BYTE_OFFSET]
                    & INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT,
                0,
                "退役的位 0 不置"
            );
            assert!(
                slot[ROLLBACK_FLOOR_BYTES_IN_THE_SLOT]
                    .iter()
                    .all(|byte| *byte == 0),
                "盘 {} 偏移 {}：F 那 8 字节是 0",
                device.0,
                offset.0
            );
            let system_configuration =
                SystemConfiguration::parse_slot(&slot).expect("核心层的读者收");
            assert_eq!(
                system_configuration.quantities.rollback_floor,
                CheckpointTxg(0),
                "读者交回 mkfs 写的 F = 0"
            );
            singlefs_checker::check_system_configuration_slot(&slot)
                .expect("checker 也认这一槽的 incompat 位");
        }
    }
}

/// 读系统配置时每个自证过的槽里的 F 各自读出来交出去（D16（发布语义） 已定项 1「生效」：系统配置里读得出的取每块幸存盘
/// 两槽里全部自证过的槽）：盘 0 两槽各种一个不同的 F，`verified_system_configuration_slots` 按槽序交回两份、各带各的 F；
/// 盘 0 槽 1 的世代号抬高之后 `choose_system_configuration` 择到它，交回的也是它那一份 F。取最大值那一步不在这里。
#[test]
fn every_self_verified_system_configuration_slot_hands_out_its_own_rollback_floor() {
    let mut pool = format_pool("rbf-hand-out");
    let mut devices = pool.devices.take().expect("mkfs 之后设备还开着");
    let [slot_zero, slot_one] = system_configuration_slot_offsets();
    plant_rollback_floor(&mut devices, DeviceIdentity(0), slot_zero, CheckpointTxg(5));
    let mut newer =
        SystemConfiguration::parse_slot(&read_slot(&devices, DeviceIdentity(0), slot_one))
            .expect("mkfs 写的槽自证得过");
    newer.quantities.rollback_floor = CheckpointTxg(9);
    newer.quantities.slot_generation = 2;
    devices[0]
        .1
        .write_at(slot_one, &newer.to_slot(), WriteDurability::Plain)
        .expect("写回槽 1");

    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let handed_out: Vec<CheckpointTxg> = verified_system_configuration_slots(
        &devices,
        DeviceIdentity(0),
        spacing,
        &E142_FILESYSTEM_IDENTIFIER,
    )
    .iter()
    .map(|system_configuration| system_configuration.quantities.rollback_floor)
    .collect();
    assert_eq!(
        handed_out,
        [CheckpointTxg(5), CheckpointTxg(9)],
        "盘 0 两槽都自证过：按槽序交回两份，各带各的 F"
    );
    let chosen = choose_system_configuration(&devices).expect("择系统配置");
    assert_eq!(
        chosen.quantities.slot_generation, 2,
        "择到盘 0 世代号最大的槽 1"
    );
    assert_eq!(
        chosen.quantities.rollback_floor,
        CheckpointTxg(9),
        "择到的那一槽交回它自己的 F"
    );
}

/// mkfs 之后的系统配置写（这里是取号）带着这块盘上系统配置里已有的 F 写，不写回 0（D16（发布语义） 已定项 1
/// 「抬 F 那一串」：已写进系统配置的新 F 留着、不回卷）。四个槽种同一个 F = 7：本盘两槽取大、择到的那一槽、
/// 整池的生效值三种读法在这里给出同一个数，这一条只钉「照带不回卷」，不钉取哪一种（D16 已定项 1 没写非抬 F 的写带哪一份）。
#[test]
fn system_configuration_write_after_make_filesystem_carries_the_rollback_floor_already_on_the_device(
) {
    let mut pool = format_pool("rbf-carry");
    let mut devices = pool.devices.take().expect("mkfs 之后设备还开着");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for offset in system_configuration_slot_offsets() {
            plant_rollback_floor(&mut devices, device, offset, CheckpointTxg(7));
        }
    }
    let make_filesystem_parameters = parameters();
    {
        let mut writer = PoolWriter::new(&make_filesystem_parameters, &mut devices);
        acquire_instance(&mut writer).expect("取号");
    }
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let written_by_the_acquisition = SystemConfiguration::parse_slot(&read_slot(
            &devices,
            device,
            system_configuration_slot_offsets()[0],
        ))
        .expect("取号写的槽自证得过");
        assert_eq!(
            written_by_the_acquisition.quantities.slot_generation, 2,
            "盘 {}：取号写的是世代号 2、落槽 0",
            device.0
        );
        assert_eq!(
            written_by_the_acquisition.quantities.rollback_floor,
            CheckpointTxg(7),
            "盘 {}：取号那一写照带系统配置里已有的 F，不回卷成 0",
            device.0
        );
    }
}

/// 系统配置只带退役的位 0（带回退见证、系统配置不带 F 的旧镜像），或位 0 与位 1 都带：
/// 挂载在任何写之前拒成「系统配置的 incompat 位图不认识」（带着盘 0 与那一槽的位图，与「池里没有一份可用的系统配置」那种数据坏了分开），
/// 录制流不多一步、四个槽一个字节不变；checker 判不认识的 incompat 位。
#[test]
fn mounting_a_pool_whose_system_configuration_carries_the_retired_layout_bit_is_refused_before_any_write_when_it_carries_only_the_retired_bit_zero(
) {
    mounting_a_pool_whose_system_configuration_carries_the_retired_layout_bit_is_refused_before_any_write(INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT, "只带退役的位 0");
}

#[test]
fn mounting_a_pool_whose_system_configuration_carries_the_retired_layout_bit_is_refused_before_any_write_when_it_carries_bit_zero_and_bit_one(
) {
    mounting_a_pool_whose_system_configuration_carries_the_retired_layout_bit_is_refused_before_any_write(
        INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT
            | INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT,
        "位 0 与位 1 都带",
    );
}

fn mounting_a_pool_whose_system_configuration_carries_the_retired_layout_bit_is_refused_before_any_write(
    first_incompat_byte: u8,
    what: &str,
) {
    let mut pool = format_pool("rbf-retired-bit");
    let mut devices = pool.devices.take().expect("mkfs 之后设备还开着");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for offset in system_configuration_slot_offsets() {
            let mut slot = read_slot(&devices, device, offset);
            slot[FIRST_INCOMPAT_BYTE_OFFSET] = first_incompat_byte;
            reseal_system_configuration_slot(&mut slot);
            assert_eq!(
                singlefs_checker::check_system_configuration_slot(&slot),
                Err(singlefs_checker::Verdict::UnknownIncompatBit),
                "{what}：checker 判不认识的 incompat 位"
            );
            let (_, target) = devices
                .iter_mut()
                .find(|(identity, _)| *identity == device)
                .expect("有这块盘");
            target
                .write_at(offset, &slot, WriteDurability::Plain)
                .expect("写回系统配置槽");
        }
    }
    let slots_before: Vec<Vec<u8>> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .flat_map(|device| {
            system_configuration_slot_offsets().map(|offset| read_slot(&devices, device, offset))
        })
        .collect();
    let recorded_operations_before = pool.stream.operations().len();
    let refusal = mount_writable(&parameters(), &mut devices);
    let Err(MountError::Recovery(RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized {
        first_device_carrying_them,
        incompat_bitmap,
    })) = refusal
    else {
        panic!(
            "{what}：挂载拒成系统配置的 incompat 位图不认识，实际 {:?}",
            refusal.as_ref().err()
        )
    };
    assert_eq!(
        (first_device_carrying_them, incompat_bitmap.0[0]),
        (DeviceIdentity(0), first_incompat_byte),
        "{what}：点名池里第一块盘、带着它槽里 incompat 位图的第一个字节"
    );
    assert!(
        incompat_bitmap.0[1..].iter().all(|byte| *byte == 0),
        "{what}：位图其余字节照 mkfs 写的全 0 带出来"
    );
    assert_eq!(
        pool.stream.operations().len(),
        recorded_operations_before,
        "{what}：拒之前一个写、一道屏障都没发"
    );
    let slots_after: Vec<Vec<u8>> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .flat_map(|device| {
            system_configuration_slot_offsets().map(|offset| read_slot(&devices, device, offset))
        })
        .collect();
    assert_eq!(
        slots_after, slots_before,
        "{what}：四个系统配置槽逐字节不变"
    );
}

/// 布局不认识与数据坏了同时在池里：盘 0 两槽都坏（补齐区改一字节、不重封），盘 1 两槽自证得过、只带退役的位 0。
/// 一份可择的都没有，而盘 1 那两槽是完整的系统配置 ⇒ 报「incompat 位图不认识」、点名盘 1（不报盘 0 的坏）。
/// 对照：盘 1 两槽也改坏 ⇒ 一槽自证得过的都没有，才报「池里没有一份可用的系统配置」、点名盘 0。
#[test]
fn unknown_layout_on_one_device_with_damaged_slots_on_the_other_reports_the_layout_and_damage_everywhere_reports_no_valid_system_configuration(
) {
    let mut pool = format_pool("rbf-layout-and-damage");
    let mut devices = pool.devices.take().expect("mkfs 之后设备还开着");
    let rewrite_slot = |open_devices: &mut Vec<(DeviceIdentity, Recorded)>,
                        device: DeviceIdentity,
                        change: &dyn Fn(&mut Vec<u8>)| {
        for offset in system_configuration_slot_offsets() {
            let mut slot = read_slot(open_devices, device, offset);
            change(&mut slot);
            let (_, target) = open_devices
                .iter_mut()
                .find(|(identity, _)| *identity == device)
                .expect("有这块盘");
            target
                .write_at(offset, &slot, WriteDurability::Plain)
                .expect("写回系统配置槽");
        }
    };
    let damage = |slot: &mut Vec<u8>| slot[4000] ^= 1;
    rewrite_slot(&mut devices, DeviceIdentity(0), &damage);
    rewrite_slot(&mut devices, DeviceIdentity(1), &|slot: &mut Vec<u8>| {
        slot[FIRST_INCOMPAT_BYTE_OFFSET] =
            INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT;
        reseal_system_configuration_slot(slot);
    });
    let Err(RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized {
        first_device_carrying_them,
        incompat_bitmap,
    }) = choose_system_configuration(&devices)
    else {
        panic!(
            "盘 0 坏、盘 1 布局不认识：该报 incompat 位图不认识，实际 {:?}",
            choose_system_configuration(&devices).err()
        )
    };
    assert_eq!(
        (first_device_carrying_them, incompat_bitmap.0[0]),
        (
            DeviceIdentity(1),
            INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT
        ),
        "点名带着完整系统配置、布局不认识的那块盘（盘 1）与它的位图"
    );

    rewrite_slot(&mut devices, DeviceIdentity(1), &damage);
    assert_eq!(
        choose_system_configuration(&devices).err(),
        Some(RecoveryFailure::NoValidSystemConfiguration {
            first_device_with_no_valid_system_configuration_slot: DeviceIdentity(0),
        }),
        "两块盘四槽都自证不过：数据坏了，报池里没有一份可用的系统配置"
    );
}

/// checker 判根槽的 flags（D22（单元原子性怎么合成） 已定项 7）：卸载记号那一位照收，别的位非 0 判 flags 非 0。
/// 根取 mkfs 种在区域 0 槽 0 的第 0 代根，改 flags、重封自证校验和。
#[test]
fn the_checker_accepts_the_unmount_marker_on_a_root_and_refuses_every_other_flag_bit() {
    let mut pool = format_pool("rbf-root-flags");
    let devices = pool.devices.take().expect("mkfs 之后设备还开着");
    let root_slot_bytes = usize::try_from(parameters().geometry.physical_block_size).expect("512");
    let genesis_root_offset = slot_offset(
        RootRingSlot { region: 0, slot: 0 },
        parameters().geometry.fixed_structure_slot_spacing,
    );
    let genesis_root = PoolReader::read(
        devices.as_slice(),
        DeviceIdentity(0),
        genesis_root_offset,
        root_slot_bytes,
    )
    .expect("第 0 代根读得到");
    let root_with_flags = |flags: u32| {
        let mut slot = genesis_root.clone();
        slot[ROOT_FLAGS_BYTES].copy_from_slice(&flags.to_le_bytes());
        let digest = wide_checksum_with_field_zeroed(&slot, root_slot_bytes, ROOT_CHECKSUM_OFFSET);
        slot[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        slot
    };
    let unmarked = singlefs_checker::check_root_slot(&genesis_root, &E142_FILESYSTEM_IDENTIFIER)
        .expect("mkfs 的根 flags 是 0");
    let marked = singlefs_checker::check_root_slot(
        &root_with_flags(ROOT_RECORD_FLAG_UNMOUNT_MARKER),
        &E142_FILESYSTEM_IDENTIFIER,
    )
    .expect("带卸载记号的根照收");
    assert_eq!(
        (
            marked.instance,
            marked.checkpoint_txg,
            marked.rollback_floor
        ),
        (
            unmarked.instance,
            unmarked.checkpoint_txg,
            unmarked.rollback_floor
        ),
        "带记号的根读出的其余字段与原根相同"
    );
    for refused_flags in [
        1u32 << 1,
        1u32 << 31,
        ROOT_RECORD_FLAG_UNMOUNT_MARKER | (1 << 1),
    ] {
        assert_eq!(
            singlefs_checker::check_root_slot(
                &root_with_flags(refused_flags),
                &E142_FILESYSTEM_IDENTIFIER
            ),
            Err(singlefs_checker::Verdict::NonZeroFlags),
            "flags {refused_flags:#010x}：卸载记号之外的位非 0"
        );
    }
}
