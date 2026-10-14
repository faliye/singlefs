//! D18（块里携带什么信息） 已定项 11「可写挂载的顺序」第五个合取：这次挂载要发的写行与暖机在分配器的副本上预演
//! 取得到全部落点——**预演里写行那次的释放核验报错，同样判这次不能可写**；写行那次发布的释放核验（含树表 0 条那一版分配记录树节点的释放核）
//! 挪到取号之前，报错就在任何写之前拒绝、不烧实例号。
//!
//! 1. 带文件的一版：上一版映射里分配记录树那一条的一条位置项指到池外的盘 7（坏盘）——写行那次换下分配记录树节点要经映射核它
//!    （D19（块指针的结构与宽度预算） 已定项 5：位置项指池外的盘当映射条目损坏）。改之前取号写完、写行那次发布才报出来，号烧掉。
//! 2. 树表 0 条的一版（`research/prompts/m2-final-code-r1-opus-output.md` Z4-1，C544）：实例表长到 66 片。原先写行那次发布的分配记录
//!    条数准入几次之后就过不了（分配记录树只有一个节点那一族，收口表第 39、57 行），池从此挂不上可写；分配记录树按绝对槽号按位置寻址之后
//!    （D8（核心索引结构） 已定项 14 的 K1）那道墙拆了，这一臂改钉「一路挂得上可写」。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/row_publish_checks_before_acquisition.rs"]
mod scenarios_of_row_publish_checks_before_acquisition;

use common::{build_pool, disk_snapshot, parameters};
use common_images_for_the_checker_tier::PoolCheckerImages;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::records::{build_mapping_entry, parse_mapping_entry};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    instance_generation_to_acquire, PoolWriter, PublishError, TransactionUnit,
};
use singlefs_core::unit::{build_index_node, parse_index_node};

/// 验收（带文件的一臂，坏盘上写行的释放核验报错）：新池新建文件（A，txg 3）之后把盘上映射根节点里分配记录树那一条的第二条位置项改指盘 7
/// （节点重封、根记录里映射根指针的整单元校验和跟着改、根槽重封：盘上自洽，只是那一条映射条目坏了）。重开可写挂载：
/// 取号之前的预演核到它，返回 `RowPublishAdmissionRefusedBeforeAcquisition`（原因 `MappingEntryLocationOnADeviceOutsideThePool`），
/// 要取的号不变、盘上逐字节不变（两盘四个系统配置槽、根环、录制流步数）。改之前预演把它交给发布路径、照样取号，号烧掉。
#[test]
fn row_publish_release_check_that_fails_on_the_damaged_mapping_entry_refuses_the_mount_before_acquisition(
) {
    let mut pool = build_pool("row-publish-release-check-before-acquisition");
    let output = pool.output.clone();
    let mapping_unit = output.unit(TransactionUnit::MappingTree).clone();
    let allocation_key = output
        .mapped_units
        .iter()
        .find(|(role, _)| *role == TransactionUnit::AllocationTree)
        .expect("分配记录树一把映射 key")
        .1
        .clone();
    let node = parse_index_node(&mapping_unit.bytes).expect("映射根节点解得开");
    let entries: Vec<Vec<u8>> = node
        .entries
        .iter()
        .map(|entry| {
            let (key, mut locations) = parse_mapping_entry(entry).expect("映射条目宽 55");
            if key == allocation_key {
                locations[1].device = DeviceIdentity(7);
                build_mapping_entry(&key, locations)
            } else {
                entry.clone()
            }
        })
        .collect();
    let damaged_node = build_index_node(
        node.tree,
        node.level,
        node.key_width,
        &entries[0][..node.key_width],
        &entries[entries.len() - 1][..node.key_width],
        node.birth_txg,
        &parameters().filesystem_identifier,
        node.instance,
        node.birth_sequence,
        u16::try_from(node.entry_width).expect("条目宽"),
        &entries,
    );
    let mut root: RootRecord = output.root;
    for location in &mut root.mapping_root.locations {
        location.unit_checksum = crc32_castagnoli(&damaged_node);
    }
    let root_target = target_for_publish(
        root.checkpoint_txg,
        parameters().geometry.root_ring_slots_per_region,
    );
    let root_device =
        parameters().region_devices[usize::try_from(root_target.region).expect("区域号")];
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        for (identity, device) in devices.iter_mut() {
            device
                .write_at(
                    mapping_unit.slot.to_device_offset(),
                    &damaged_node,
                    WriteDurability::Plain,
                )
                .expect("写回改过的映射根节点");
            if *identity == root_device {
                device
                    .write_at(
                        slot_offset(
                            root_target,
                            parameters().geometry.fixed_structure_slot_spacing,
                        ),
                        &root.to_slot(512),
                        WriteDurability::ForceUnitAccess,
                    )
                    .expect("写回重封的根槽");
            }
        }
    }
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let mut devices = pool.reopen_recorded();
    let instance_before = {
        let publish_parameters = parameters();
        instance_generation_to_acquire(&PoolWriter::new(
            &publish_parameters,
            devices.as_mut_slice(),
        ))
    };
    let refused = mount_writable(&parameters(), &mut devices);
    let instance_after = {
        let publish_parameters = parameters();
        instance_generation_to_acquire(&PoolWriter::new(
            &publish_parameters,
            devices.as_mut_slice(),
        ))
    };
    pool.devices = Some(devices);
    assert!(
        matches!(
            &refused,
            Err(MountError::RowPublishAdmissionRefusedBeforeAcquisition {
                cause: PublishError::MappingEntryLocationOnADeviceOutsideThePool {
                    unit: TransactionUnit::AllocationTree,
                    device: DeviceIdentity(7),
                    ..
                },
                ..
            })
        ),
        "预演里写行那次的释放核验报错 ⇒ 取号之前拒：{refused:?}"
    );
    assert_eq!(instance_after, instance_before, "实例代号不变：号没烧");
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "盘上逐字节不变：系统配置槽、根环、录制流步数"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_row_publish_checks_before_acquisition::row_publishes_on_a_version_without_file_with_a_sixty_six_page_instance_table_keep_mounting_writable_past_812_allocation_records`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn row_publishes_on_a_version_without_file_with_a_sixty_six_page_instance_table_keep_mounting_writable_past_812_allocation_records(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_row_publish_checks_before_acquisition::row_publishes_on_a_version_without_file_with_a_sixty_six_page_instance_table_keep_mounting_writable_past_812_allocation_records,
    );
}
