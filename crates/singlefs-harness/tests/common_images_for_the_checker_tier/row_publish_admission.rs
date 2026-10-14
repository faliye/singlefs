//! harness 测试文件 `crates/singlefs-harness/tests/row_publish_admission.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocation_record_tree::AllocationRecordTreeNode;
use singlefs_core::journal::back_chain_of;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{
    publish_overwrite, publish_version, FirstFile, InstanceTablePlan, PoolVersion, PoolWriter,
    PublishError, PublishPlan, TransactionOutput,
};

pub const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 原先一个节点装得下几条分配记录：(16384 − 头 135) ÷ 20。
pub const ONE_LEAF_OF_ALLOCATION_RECORDS: usize = 812;

pub fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 同一个进程里接着现行版本覆盖写一次，错误原样交回。
pub fn try_overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    )?;
    pool.output = output.clone();
    Ok(output)
}

/// 同一个进程里接着现行版本推一次空发布（不写文件、照抄实例表），错误原样交回。
pub fn try_empty_publish_in_process(
    pool: &mut BuiltPool,
) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_version(
        &mut writer,
        &mut pool.allocator,
        PublishPlan {
            txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
            counter: previous.record.counter + 1,
            transaction: 0,
            highest_transaction_number_before_this_publish: previous
                .highest_transaction_number_in_this_instance,
            instance: InstanceGeneration(1),
            back_chain: back_chain_of(&previous.record_bytes),
            file: None,
            // 这次空发布不碰 inode 树。
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(&previous),
    )?;
    pool.output = output.clone();
    Ok(output)
}

/// 覆盖写 `overwrites` 次、再推 `empty_publishes` 次空发布，每一次都必须做成；交回最后一次覆盖写的内容。
pub fn publish_overwrites_then_empty_publishes(
    pool: &mut BuiltPool,
    overwrites: usize,
    empty_publishes: usize,
) -> Vec<u8> {
    let mut last_content = Vec::new();
    for index in 0..overwrites {
        last_content = content_of(3000 + index % 97, index);
        try_overwrite_in_process(pool, &last_content)
            .unwrap_or_else(|error| panic!("第 {index} 次覆盖写要做成：{error:?}"));
    }
    for index in 0..empty_publishes {
        try_empty_publish_in_process(pool)
            .unwrap_or_else(|error| panic!("第 {index} 次空发布要做成：{error:?}"));
    }
    last_content
}

/// 原先在取号之前被分配记录墙拒的那个池（`overwrites` 次覆盖写 + `empty_publishes` 次空发布）上可写挂载：做成，取到实例 2，
/// 挂载之后那一版多于 812 条分配记录、每块盘上装着记录的叶不止一片；池级 checker 全绿；冷启动读回挂载之前最后一次覆盖写的内容。
pub fn writable_mount_on_a_pool_past_the_former_allocation_record_wall_succeeds(
    images: &mut PoolCheckerImages,
    tag: &str,
    overwrites: usize,
    empty_publishes: usize,
) {
    let mut pool = build_pool(tag);
    let last_content =
        publish_overwrites_then_empty_publishes(&mut pool, overwrites, empty_publishes);
    assert!(
        pool.allocator.records().len() > ONE_LEAF_OF_ALLOCATION_RECORDS,
        "挂载之前已经多于 812 条：{}",
        pool.allocator.records().len()
    );

    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).unwrap_or_else(|error| {
        panic!("原先在取号之前被拒的池，拆墙之后可写挂载要做成：{error:?}")
    });
    pool.devices = Some(devices);
    assert_eq!(mounted.output.instance, InstanceGeneration(2));
    let current = mounted
        .current
        .into_file_version()
        .expect("带文件的一版上写行、暖机，现行那一版仍带文件");
    assert!(
        current.allocation_records.len() > ONE_LEAF_OF_ALLOCATION_RECORDS,
        "挂载之后那一版 {} 条分配记录",
        current.allocation_records.len()
    );
    for device in DISKS {
        let leaves_holding_records = current
            .allocation_record_tree
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
    assert!(
        matches!(mounted.output.row_publish, PoolVersion::WithFile(_)),
        "写行那次发布在带文件的一版上"
    );

    let image = pool.memory_pool();
    images.hand_out(
        "越过原先分配记录那堵墙之后可写挂载",
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    let report = recover(&image, JournalPolicy::Consult);
    let RecoveryOutcome::FileRead { content, .. } = report.outcome else {
        panic!("冷启动要读回文件，实际 {:?}", report.outcome);
    };
    assert_eq!(
        content, last_content,
        "冷启动读回挂载之前最后一次覆盖写的内容"
    );
}

/// 原先「写行那次装不下」的池（49 次覆盖写 + 1 次空发布，那时正好 812 条）。
pub fn writable_mount_on_the_pool_whose_row_publish_used_to_be_refused_before_acquisition_succeeds(
    images: &mut PoolCheckerImages,
) {
    writable_mount_on_a_pool_past_the_former_allocation_record_wall_succeeds(
        images,
        "supplement-two-row-publish-past-the-former-wall",
        49,
        1,
    );
}

/// 原先「写行装得下、暖机第 2 次装不下」的池（47 次覆盖写 + 2 次空发布）：这次挂载计划推两次暖机，两次都发出去。
pub fn writable_mount_on_the_pool_whose_second_warm_up_used_to_be_refused_before_acquisition_succeeds(
    images: &mut PoolCheckerImages,
) {
    writable_mount_on_a_pool_past_the_former_allocation_record_wall_succeeds(
        images,
        "supplement-two-warm-up-past-the-former-wall",
        47,
        2,
    );
}
