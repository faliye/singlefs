//! harness 测试文件 `crates/singlefs-harness/tests/unlink_single_file_publishes.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use crate::common_deletion_in_progress::{
    pool_with_a_three_unit_file, publish_the_next_step_in_batches_of_one, FIRST_FILE,
    INSTANCE_OF_THE_PUBLISHES,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::transaction::{
    publish_unlink_intent, PoolWriter, PublishError, TransactionOutput,
};

/// 意图那次被拒要交回错（已经删完的那一格），`crate::common_deletion_in_progress::publish_the_intent` 被拒就 panic：这一份照旧留在这里。
pub fn publish_the_intent(pool: &mut BuiltPool) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    // 第一个文件那条记录现有的时间（已经删完的那一格没有记录，意图在任何写之前被拒，给的时间不落盘）。
    let write_time_seconds = previous
        .inode_record
        .map_or(FIXED_WRITE_TIME_SECONDS, |record| record.write_time_seconds);
    let output = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FIRST_FILE,
        write_time_seconds,
        INSTANCE_OF_THE_PUBLISHES,
    )?;
    pool.output = output.clone();
    Ok(output)
}

pub fn the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_with_a_three_unit_file("unlink-checker-between-batches");
    publish_the_intent(&mut pool).expect("意图");
    // 每一版池级 checker 全绿
    images.hand_out(
        "意图",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    for batch in 1..=3 {
        publish_the_next_step_in_batches_of_one(&mut pool);
        images.hand_out(
            &format!("第 {batch} 批"),
            || pool.memory_pool(),
            PoolCheckerExpectation::no_invariant_violated(),
        );
    }
    assert!(pool.output.deleted_inodes.is_empty(), "三批之后删完");
}
