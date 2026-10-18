//! 系统配置槽里节点大小那 4 字节（偏移 301，几何段开头）不等于格式常量 `NODE_BYTES` 的槽当自证不过、不可择
//! （C840，用户 2026-10-15 弹窗定「当自证不过」）：别的槽照旧可择；全池每一槽都这样时择系统配置与只读挂载都拒；
//! 池级 checker 另判一份（不引实现），把这一槽同样当自证不过；全池每一槽都这样时池级 checker 判哪几条，在
//! `pool_checker_reddens_only_the_tail_slot_invariant_when_every_system_configuration_slot_has_a_foreign_node_size.rs`。
//!
//! 坏镜像的造法在 `common_foreign_node_size_planted_in_system_configuration_slots`：只改那 4 字节、整槽校验和重封，拒它的只能是节点大小那一判。
//! 盘：两个文件镜像上 mkfs → 取号 → 暖机 → 新池新建文件（`common::build_pool`），整条录制流施加到内存镜像上再改。

mod common;
mod common_foreign_node_size_planted_in_system_configuration_slots;

use common::build_pool;
use common_foreign_node_size_planted_in_system_configuration_slots::{
    plant_a_foreign_node_size, plant_a_foreign_node_size_into_every_slot, read_slot,
    system_configuration_slot_offsets, FOREIGN_NODE_BYTES,
};
use singlefs_checker::image::{
    system_configuration_redundancy_report, SystemConfigurationRedundancyReport,
};
use singlefs_checker::{check_system_configuration_slot, Verdict};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::mounted_read::{mount_read_only, MountReadOnlyFailure};
use singlefs_core::recovery::{choose_system_configuration, RecoveryFailure};
use singlefs_core::system_configuration::{SystemConfiguration, SystemConfigurationSlotRefusal};

/// 场景：新池新建文件之后，盘 0 世代号大的那一槽（改之前择系统配置择到的就是它）节点大小换成 8192。
/// 预期：读者拒这一槽、点名节点大小；择系统配置不择它，择到盘 0 的另一槽。
#[test]
fn the_slot_with_a_foreign_node_size_is_not_chosen_and_the_other_slot_of_the_device_is() {
    let mut image = build_pool("foreign-node-size-one-slot").memory_pool();
    let [slot_zero_offset, slot_one_offset] = system_configuration_slot_offsets();
    let slot_zero = SystemConfiguration::parse_slot(&read_slot(&image, 0, slot_zero_offset))
        .expect("盘 0 槽 0 自证得过");
    let slot_one = SystemConfiguration::parse_slot(&read_slot(&image, 0, slot_one_offset))
        .expect("盘 0 槽 1 自证得过");
    assert_ne!(
        slot_zero.quantities.slot_generation, slot_one.quantities.slot_generation,
        "前提：盘 0 两槽世代号不同，择到的是哪一槽分得出"
    );
    let (newer_offset, newer, older) =
        if slot_one.quantities.slot_generation > slot_zero.quantities.slot_generation {
            (slot_one_offset, slot_one, slot_zero)
        } else {
            (slot_zero_offset, slot_zero, slot_one)
        };
    assert_eq!(
        choose_system_configuration(&image),
        Ok(newer),
        "改之前择到盘 0 世代号大的那一槽"
    );

    plant_a_foreign_node_size(&mut image, 0, newer_offset);

    assert_eq!(
        SystemConfiguration::parse_slot(&read_slot(&image, 0, newer_offset)),
        Err(
            SystemConfigurationSlotRefusal::NodeBytesNotTheFormatConstant {
                recorded_node_bytes: FOREIGN_NODE_BYTES,
            }
        ),
        "读者拒这一槽，点名节点大小、带着盘上的 8192"
    );
    assert_eq!(
        choose_system_configuration(&image),
        Ok(older),
        "节点大小不对的那一槽不可择：择到盘 0 的另一槽"
    );
}

/// 场景：两块盘四个系统配置槽的节点大小都换成 8192。
/// 预期：一槽可择的都没有，择系统配置拒成 `NoValidSystemConfiguration`（与整槽校验和全坏同一个成员），只读挂载在择系统配置那一步同样拒。
#[test]
fn every_slot_with_a_foreign_node_size_refuses_choosing_the_system_configuration_and_the_read_only_mount(
) {
    let mut image = build_pool("foreign-node-size-every-slot").memory_pool();
    plant_a_foreign_node_size_into_every_slot(&mut image);
    let no_valid_system_configuration = RecoveryFailure::NoValidSystemConfiguration {
        first_device_with_no_valid_system_configuration_slot: DeviceIdentity(0),
    };
    assert_eq!(
        choose_system_configuration(&image),
        Err(no_valid_system_configuration.clone()),
        "择系统配置：每块盘两槽都当自证不过"
    );
    assert_eq!(
        mount_read_only(&image).err(),
        Some(MountReadOnlyFailure::Recovery(
            no_valid_system_configuration
        )),
        "只读挂载在择系统配置那一步拒，同一个成员"
    );
}

/// 场景：盘 0 槽 0 的节点大小换成 8192，别的三槽不动。
/// 预期：池级 checker 判这一槽点名节点大小；旁路报告把盘 0 记成少一槽自证过（可写挂载会拒），盘 0 槽 1 还自证过、不算两槽全废。
#[test]
fn the_pool_checker_refuses_the_slot_with_a_foreign_node_size_as_not_self_verified() {
    let mut image = build_pool("foreign-node-size-checker-one-slot").memory_pool();
    assert_eq!(
        system_configuration_redundancy_report(&image),
        SystemConfigurationRedundancyReport {
            devices_without_a_self_verified_slot: Vec::new(),
            devices_whose_writable_mount_would_be_refused: Vec::new(),
        },
        "改之前每块盘两槽都自证过"
    );

    plant_a_foreign_node_size(&mut image, 0, 0);

    assert_eq!(
        check_system_configuration_slot(&read_slot(&image, 0, 0)),
        Err(Verdict::NodeBytesNotTheFormatConstant),
        "池级 checker 判这一槽：点名节点大小那一格"
    );
    assert_eq!(
        system_configuration_redundancy_report(&image),
        SystemConfigurationRedundancyReport {
            devices_without_a_self_verified_slot: Vec::new(),
            devices_whose_writable_mount_would_be_refused: vec![0],
        },
        "盘 0 少一槽自证过：节点大小不对的槽当自证不过"
    );
}
