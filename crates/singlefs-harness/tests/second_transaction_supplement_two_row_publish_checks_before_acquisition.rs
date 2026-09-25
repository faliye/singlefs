//! D18（块里携带什么信息） 已定项 11「可写挂载的顺序」第五个合取（用户 2026-09-24 定）：这次挂载要发的写行与暖机在分配器的副本上预演
//! 取得到全部落点——**预演里写行那次的释放核验报错，同样判这次不能可写**；写行那次发布的释放核验（含树表 0 条那一版分配记录树节点的释放核）
//! 挪到取号之前，报错就在任何写之前拒绝、不烧实例号。
//!
//! 1. 带文件的一版：上一版映射里分配记录树那一条的一条位置项指到池外的盘 7（坏盘）——写行那次换下分配记录树节点要经映射核它
//!    （D19（块指针的结构与宽度预算） 已定项 5：位置项指池外的盘当映射条目损坏）。改之前取号写完、写行那次发布才报出来，号烧掉。
//! 2. 树表 0 条的一版（`research/prompts/m2-final-code-r1-opus-output.md` Z4-1，C544）：实例表长到 66 片。原先写行那次发布的分配记录
//!    条数准入几次之后就过不了（分配记录树只有一个节点那一族，收口表第 39、57 行），池从此挂不上可写；分配记录树按绝对槽号按位置寻址之后
//!    （D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K1）那道墙拆了，这一臂改钉「一路挂得上可写」。

mod common;

use common::{build_pool, disk_snapshot, memory_pool_of_sparse_devices, parameters, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::allocation_record_tree::AllocationRecordTreeNode;
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::records::{build_mapping_entry, parse_mapping_entry};
use singlefs_core::recovery::allocation_records_of_version_without_file;
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    acquire_instance, instance_generation_to_acquire, PoolVersion, PoolWriter, PublishError,
    TransactionUnit,
};
use singlefs_core::unit::{build_index_node, parse_index_node};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 验收（带文件的一臂，坏盘上写行的释放核验报错）：第一个事务（A，txg 3）之后把盘上映射根节点里分配记录树那一条的第二条位置项改指盘 7
/// （节点重封、根记录里映射根指针的整单元校验和跟着改、根槽重封：盘上自洽，只是那一条映射条目坏了）。重开可写挂载：
/// 取号之前的预演核到它，返回 `RowPublishAdmissionRefusedBeforeAcquisition`（原因 `MappingEntryLocationOnADeviceOutsideThePool`），
/// 要取的号不变、盘上逐字节不变（两盘四个系统配置槽、根环、录制流步数）。改之前预演把它交给发布路径、照样取号，号烧掉。
#[test]
fn a_row_publish_release_check_that_fails_on_a_damaged_mapping_entry_refuses_the_mount_before_acquisition(
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

type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

fn sparse_devices(stream: &SharedStream) -> Devices {
    DISKS
        .iter()
        .map(|identity| {
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(
                    *identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

fn instance_to_acquire_now(devices: &mut Devices) -> InstanceGeneration {
    let publish_parameters = parameters();
    instance_generation_to_acquire(&PoolWriter::new(
        &publish_parameters,
        devices.as_mut_slice(),
    ))
}

/// 验收（树表 0 条的一臂，Z4-1 的形状，C544）：mkfs → 可写挂载一次（实例 1，零单元发布）→「取号之后崩溃」65 × 369 次（每次取号写完、
/// 写行那次发布之前掉电：一段合法历史）→ 下一次挂载一次写 23986 行、实例表 66 片；之后一路可写挂载。树表 0 条的一版上每次写行重写整条链
/// 与这一版自己那棵分配记录树里内容变了的节点，被换下的进 defer、根环没转过一条都回收不了，分配记录每次涨一百多条——原先几次之后就
/// 装不下一个节点，在取号之前拒、池从此挂不上可写（收口表第 39、57 行那一族）。分配记录树按绝对槽号按位置寻址之后
/// （D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K1）十二次都做成：每次取到的号是挂载之前算的那一个；最后那一版的分配记录多于 812 条、
/// 每块盘上装着记录的叶不止一片；池级 checker 全绿。
#[test]
fn row_publishes_on_a_version_without_file_with_a_sixty_six_page_instance_table_keep_mounting_writable_past_812_allocation_records(
) {
    let stream = SharedStream::new();
    let mut devices = sparse_devices(&stream);
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    mount_writable(&parameters(), &mut devices).expect("mkfs 之后第一次可写挂载");
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        for _ in 0..65 * 369 {
            acquire_instance(&mut writer).expect("取号之后崩溃");
        }
    }
    let mut last_row_publish_root = None;
    for mount_number in 0..12 {
        let instance_before = instance_to_acquire_now(&mut devices);
        let mounted = mount_writable(&parameters(), &mut devices).unwrap_or_else(|error| {
            panic!("第 {mount_number} 次挂载：树表 0 条的一版上写 66 片实例表要做成：{error:?}")
        });
        assert_eq!(
            mounted.output.instance, instance_before,
            "第 {mount_number} 次挂载取到的号"
        );
        let PoolVersion::WithoutFile(row_publish) = &mounted.output.row_publish else {
            panic!("树表 0 条的一版上写行，写出来的仍是没有文件版本的一版")
        };
        last_row_publish_root = Some(row_publish.root);
    }
    let image = memory_pool_of_sparse_devices(&devices);
    let tree = allocation_records_of_version_without_file(
        &image,
        &last_row_publish_root.expect("挂载了十二次"),
    )
    .expect("分配记录树读得出")
    .expect("写过行的一版有自己的分配记录树");
    assert!(
        tree.records.len() > 812,
        "十二次写 66 片之后 {} 条分配记录",
        tree.records.len()
    );
    for device in DISKS {
        let leaves_holding_records = tree
            .version
            .nodes
            .iter()
            .filter(|(node, _)| {
                matches!(
                    node,
                    AllocationRecordTreeNode::BelowTheRoot(position)
                        if position.level == 0 && position.device == device
                )
            })
            .count();
        assert!(
            leaves_holding_records > 1,
            "盘 {} 上装着记录的叶 {leaves_holding_records} 片",
            device.0
        );
    }
    let violated: Vec<(&str, String)> = check_pool_image(&image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect();
    assert_eq!(violated, Vec::new(), "十二次挂载之后池级 checker 全绿");
}
