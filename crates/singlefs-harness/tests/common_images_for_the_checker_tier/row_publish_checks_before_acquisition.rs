//! harness 测试文件 `crates/singlefs-harness/tests/row_publish_checks_before_acquisition.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{memory_pool_of_sparse_devices, parameters, IMAGE_BYTES};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::allocation_record_tree::AllocationRecordTreeNode;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::allocation_records_of_version_without_file;
use singlefs_core::transaction::{
    acquire_instance, instance_generation_to_acquire, PoolVersion, PoolWriter,
};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

pub const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

pub type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

pub fn sparse_devices(stream: &SharedStream) -> Devices {
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

pub fn instance_to_acquire_now(devices: &mut Devices) -> InstanceGeneration {
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
pub fn row_publishes_on_a_version_without_file_with_a_sixty_six_page_instance_table_keep_mounting_writable_past_812_allocation_records(
    images: &mut PoolCheckerImages,
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
    images.hand_out(
        "十二次挂载之后",
        || image,
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
