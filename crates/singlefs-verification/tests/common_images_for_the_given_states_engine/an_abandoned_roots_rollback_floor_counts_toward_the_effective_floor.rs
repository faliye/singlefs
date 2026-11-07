//! harness 测试文件 `crates/singlefs-verification/tests/an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_given_states_engine/mod.rs`）。那份 harness 测试文件与给定状态那一份
//! `crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "引它的测试目标各只调其中一部分")]

use crate::common::{
    abandon_the_third_version_by_a_crash_before_its_rotation, build_pool, parameters,
    publish_overwrite_in_process, system_configuration_slots_of_every_device, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use crate::common_images_for_the_given_states_engine::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_shared_by_harness_scenarios::content_seeded_by;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::TransactionOutput;

/// 改写之后 C 带的 F：高于 0（各盘有效根与系统配置里的 F），不高于 C 自己的 txg 5。
pub const FLOOR_CARRIED_BY_THE_ABANDONED_ROOT: CheckpointTxg = CheckpointTxg(4);

pub fn overwrite_as_instance_one(pool: &mut BuiltPool, content: &[u8]) -> TransactionOutput {
    let previous = pool.output.clone();
    let output = publish_overwrite_in_process(
        pool,
        &previous,
        content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// A、B、C（实例 1）→ 崩溃恢复抛弃 C（落到 B、实例 2 写行与暖机）→ C 的根记录改带 F = 4。
pub fn pool_whose_abandoned_root_carries_a_higher_floor(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_as_instance_one(&mut pool, &content_seeded_by(4100, 3));
    let system_configuration_before_the_third = system_configuration_slots_of_every_device(&pool);
    let third = overwrite_as_instance_one(&mut pool, &content_seeded_by(2500, 11));
    abandon_the_third_version_by_a_crash_before_its_rotation(
        &mut pool,
        &third,
        &system_configuration_before_the_third,
    );
    let publish_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(
        third.root.checkpoint_txg,
        publish_parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot_device =
        publish_parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_slot_offset = singlefs_core::root_ring::slot_offset(
        target,
        publish_parameters.geometry.fixed_structure_slot_spacing,
    );
    let root_slot_bytes =
        usize::try_from(publish_parameters.geometry.physical_block_size).expect("根槽宽");
    let devices = pool.devices.as_mut().expect("镜像开着");
    let (_, device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == root_slot_device)
        .expect("池里有这块盘");
    let mut slot = vec![0u8; root_slot_bytes];
    device
        .read_at(root_slot_offset, &mut slot)
        .expect("读 C 的根槽");
    let mut abandoned_root =
        RootRecord::parse_slot(&slot, &publish_parameters.filesystem_identifier)
            .expect("C 的根自证得过");
    assert_eq!(
        abandoned_root.checkpoint_txg,
        CheckpointTxg(5),
        "根槽里住的是 C"
    );
    assert_eq!(abandoned_root.rollback_floor, CheckpointTxg(0));
    abandoned_root.rollback_floor = FLOOR_CARRIED_BY_THE_ABANDONED_ROOT;
    device
        .write_at(
            root_slot_offset,
            &abandoned_root.to_slot(root_slot_bytes),
            WriteDurability::Plain,
        )
        .expect("C 的根改带 F = 4 写回");
    pool
}

/// 池级 checker 一侧：F_生效 取到 4，高于最新根带的 F（0）——I-3.16（碎片度与全空段统计对得上） 那一格按「最新根带的 F 低于 F 生效值」报不适用
/// （`crates/singlefs-checker/invariants/05-空间记账.md` I-3.16 那一行「最新根带的 F 低于 F 生效值……时不判」）。
/// F_生效 = 0 = 最新根带的 F 时，I-3.16 照常判。I-3.1（已分配统计对得上） 按 RCW 判，没有这一道例外
/// （判决 `research/prompts/m3-close-b19-design-r2-main-verification.md` 第二节 Q5「换成 RCW 后『F_生效 高于根上 F 时不判』那条例外拿掉」），
/// 这一张上判到而成立。
pub fn run_the_scenario_where_the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries(
    images: &mut PoolCheckerImages,
) {
    let pool = pool_whose_abandoned_root_carries_a_higher_floor("abandoned-floor-counts-checker");
    // checker 的 F_生效 取到被抛弃根带的 4、高于最新根带的 0：I-3.16 按「最新根带的 F 低于 F 生效值」报不适用（没判到；不适用的原因串不进对拍字）；
    // I-3.1 照 RCW 判到而成立
    images.hand_out(
        "被抛弃的根改带 F = 4 之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_not_judged("I-3.16")
            .and_holds("I-3.1"),
    );
}
