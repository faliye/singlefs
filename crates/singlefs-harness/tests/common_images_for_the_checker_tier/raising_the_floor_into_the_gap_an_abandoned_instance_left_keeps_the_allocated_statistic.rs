//! harness 测试文件 `crates/singlefs-harness/tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    abandon_the_third_version_by_a_crash_before_its_rotation, build_pool, parameters,
    publish_overwrite_in_process, system_configuration_slots_of_every_device, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{mount_writable, raise_rollback_floor, ShadowLedger};
use singlefs_core::transaction::TransactionOutput;

pub fn content_seeded_by(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

pub fn overwrite(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let previous = pool.output.clone();
    let output = publish_overwrite_in_process(
        pool,
        &previous,
        content,
        FIXED_WRITE_TIME_SECONDS + 60,
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// A、B、C（实例 1）→ 崩溃恢复抛弃 C → 再可写挂载一次 → 覆盖写四次：抬 F 之前全绿的池，与再挂载那一次的实例代号。
pub fn pool_before_the_raise(tag: &str, images: &mut PoolCheckerImages) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite(
        &mut pool,
        &content_seeded_by(4100, 3),
        InstanceGeneration(1),
    );
    let system_configuration_before_the_third = system_configuration_slots_of_every_device(&pool);
    let third = overwrite(
        &mut pool,
        &content_seeded_by(2500, 11),
        InstanceGeneration(1),
    );
    assert_eq!(third.root.checkpoint_txg, CheckpointTxg(5), "C 是 txg 5");
    abandon_the_third_version_by_a_crash_before_its_rotation(
        &mut pool,
        &third,
        &system_configuration_before_the_third,
    );
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("再可写挂载：影子账看见 C");
    let instance = mounted.output.instance;
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator.clone();
    pool.output = mounted.current.file_version().expect("带文件").clone();
    for seed in [17usize, 19, 23, 29] {
        overwrite(&mut pool, &content_seeded_by(3000 + seed, seed), instance);
    }
    images.hand_out(
        "抬 F 之前",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    pool
}

/// 抬 F 之前的池（那一刻的镜像交出去）抬到 `floor`，交回抬过的池。
pub fn raise_the_floor(tag: &str, floor: u64, images: &mut PoolCheckerImages) -> BuiltPool {
    let mut pool = pool_before_the_raise(tag, images);
    raise_rollback_floor(
        &parameters(),
        pool.devices.as_mut().expect("镜像开着"),
        &mut pool.allocator,
        &mut pool.output,
        CheckpointTxg(floor),
        ShadowLedger::On,
    )
    .expect("抬 F");
    pool
}

/// F 抬到 C 那个 txg（5）：F 落进被抛弃实例留下的空档，B（txg 4）掉到 F 之下、C 被抛弃，只被它俩引用、释放代 6 的 10 槽还在 defer 里。
pub fn raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_every_invariant_green(
    images: &mut PoolCheckerImages,
) {
    let pool = raise_the_floor("row43-gap-floor-5", 5, images);
    // F 之下仍在 defer 的槽并进遍历：I-3.1 不红
    images.hand_out(
        "F 抬到 5 之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 对照：同一段历史只换 F（6：释放代 6 的那 10 槽已可回收），改之前改之后都全绿。
pub fn raising_the_floor_past_the_gap_keeps_every_invariant_green(images: &mut PoolCheckerImages) {
    let pool = raise_the_floor("row43-gap-floor-6", 6, images);
    // F 抬过空档：全绿
    images.hand_out(
        "F 抬到 6 之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
