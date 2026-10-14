//! harness 测试文件 `crates/singlefs-harness/tests/root_ring_turn_in_one_mount.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, DataUnitIndexInFile, InstanceGeneration};
use singlefs_core::make_filesystem::TREE_TABLE_GENESIS_SLOT;
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, TransactionOutput, TransactionUnit,
};

pub fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

pub fn overwrite_in_process(
    pool: &mut BuiltPool,
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let seed = usize::try_from(previous.root.checkpoint_txg.0).expect("txg 很小");
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content: &content_of(2000 + seed, seed),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 交 checker 档：这一刻的镜像上池级 checker 一条都不红（判不适用的不算）。
pub fn hand_out_expecting_no_violated_invariant(
    images: &mut PoolCheckerImages,
    label: &str,
    pool: &BuiltPool,
) {
    images.hand_out(
        label,
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// C518 的验收：新池新建文件（txg 3）之后可写挂载一次（实例 2：写行 txg 4、暖机 txg 5），同一次挂载里连着覆盖写（txg 6 起）。
/// 根环 R × S = 24 槽：txg 24、25、26 依次盖掉第 0 代根与两条暖机根。txg 2 的暖机根是最后一条引用 mkfs 那片第 0 版树表
/// （50178，1 槽，第一个文件版本在 txg 3 换下）的根；txg 26 盖掉它之后环里最旧有效根是 3，那一槽按谓词回收——
/// 同一次发布的记账行就按回收之后的数写，checker 的 I-3.1（已分配统计对得上） 在这一版上成立（修之前这一步判红）；
/// txg 27 的数据单元（用户数据取最低的两槽空位）落回 50178。
pub fn turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it(
    images: &mut PoolCheckerImages,
) {
    let mut pool = build_pool("root-ring-turn-in-one-mount");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("第一个文件之后重开，现行那一版带文件");
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(5),
        "写行 txg 4、暖机 txg 5"
    );
    while pool.output.root.checkpoint_txg < CheckpointTxg(25) {
        overwrite_in_process(&mut pool, InstanceGeneration(2));
    }
    for device_map in &pool.allocator.devices {
        assert!(
            !device_map.is_free(TREE_TABLE_GENESIS_SLOT),
            "盘 {:?}：txg 25 之后 txg 2 的暖机根还在环里、还引用着 mkfs 那片树表",
            device_map.device
        );
    }
    let deferred_before_txg_26: Vec<u64> = pool
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.deferred_slots())
        .collect();

    let txg_26 = overwrite_in_process(&mut pool, InstanceGeneration(2));
    assert_eq!(txg_26.root.checkpoint_txg, CheckpointTxg(26));
    let released_by_txg_26: u64 = txg_26.released.iter().map(|placement| placement.span).sum();
    for (device_map, deferred_before) in pool.allocator.devices.iter().zip(&deferred_before_txg_26)
    {
        assert!(
            device_map.is_free(TREE_TABLE_GENESIS_SLOT),
            "盘 {:?}：txg 26 盖掉 txg 2 之后 mkfs 那片树表按谓词回收，同一次挂载里就回到空闲",
            device_map.device
        );
        assert_eq!(
            device_map.deferred_slots() + 1,
            deferred_before + released_by_txg_26,
            "盘 {:?}：defer 队列加上 txg 26 换下的，减去回收的那 1 槽",
            device_map.device
        );
    }
    // txg 26 这一版的记账按回收之后的数写：checker 一条都不红
    hand_out_expecting_no_violated_invariant(
        images,
        "txg 26 盖掉最后一条引用第 0 版树表的根之后",
        &pool,
    );

    let txg_27 = overwrite_in_process(&mut pool, InstanceGeneration(2));
    assert_eq!(
        txg_27
            .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
            .slot,
        TREE_TABLE_GENESIS_SLOT,
        "回收回来的那一槽与旁边从没分配过的 50179 是全池最低的两槽空位：txg 27 的数据单元落在那里"
    );
    hand_out_expecting_no_violated_invariant(images, "txg 27 的数据单元落回 50178 之后", &pool);
}
