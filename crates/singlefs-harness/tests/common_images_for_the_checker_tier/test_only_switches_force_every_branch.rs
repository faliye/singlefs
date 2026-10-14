//! harness 测试文件 `crates/singlefs-harness/tests/test_only_switches_force_every_branch.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::ReuseWindow;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};

pub fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

pub fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
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
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 一次跑的可观测读数：开关自己报的分支名、它在 `release` 里当场回收掉的落点数、两次覆盖写的数据落点、
/// 第一个文件那一版（txg 3，F = 0 时仍是候选根）的数据落点、两次覆盖写之后的池（池级 checker 那一遍交 checker 档）。
pub struct ReuseWindowRun {
    pub branch_name: &'static str,
    pub reclaimed_on_release: u64,
    pub first_file_data_slot: SlotNumber,
    pub overwrite_data_slots: [SlotNumber; 2],
    pub pool_after_the_two_overwrites: BuiltPool,
}

/// 同一条固定脚本跑一遍：mkfs → 新池新建文件（txg 3）→ 覆盖写两次（txg 4、5），复用窗口按入参装。
pub fn run_two_overwrites_with(reuse_window: ReuseWindow, tag: &str) -> ReuseWindowRun {
    let mut pool = build_pool(tag);
    pool.allocator.set_reuse_window(reuse_window);
    let first_file_data_slot = pool.output.data_pointers[0].locations[0].slot;
    let fourth = overwrite_in_process(&mut pool, &content_of(3100, 3), InstanceGeneration(1));
    let fifth = overwrite_in_process(&mut pool, &content_of(2900, 7), InstanceGeneration(1));
    assert_eq!(fourth.root.checkpoint_txg, CheckpointTxg(4));
    assert_eq!(fifth.root.checkpoint_txg, CheckpointTxg(5));
    ReuseWindowRun {
        branch_name: pool.allocator.reuse_window().name(),
        reclaimed_on_release: pool
            .allocator
            .placements_reclaimed_on_release_by_the_forced_zero_reuse_window(),
        first_file_data_slot,
        overwrite_data_slots: [
            fourth.data_pointers[0].locations[0].slot,
            fifth.data_pointers[0].locations[0].slot,
        ],
        pool_after_the_two_overwrites: pool,
    }
}

/// 必红八条第四条（复用窗口置 0，C22（刚释放的块立即重分配））：**走正常的发布路径**把复用造出来——
/// 开关装在分配器上，释放与再分配都由 `publish_overwrite` 自己做，用例一次都不调回收。
/// F = 0 ⇒ txg 3 那条根还在回退候选集里，而它引用的数据单元已经被 txg 5 那一版盖掉 ⇒
/// checker 从那条根走进去，校验和对不上（I-2.1）、近 K 代根不自洽（I-4.8）。
/// 关着时同一条脚本全绿：判别力自证就是这两遍的差。
pub fn reuse_window_forced_to_zero_lets_a_publish_overwrite_a_unit_a_candidate_root_still_references_and_the_checker_goes_red(
    images: &mut PoolCheckerImages,
) {
    let gated = run_two_overwrites_with(
        ReuseWindow::GatedByTheRollbackFloor,
        "must-red-reuse-window-gated",
    );
    // 关着时 checker 全绿
    images.hand_out(
        "复用窗口关着、覆盖写两次之后",
        || gated.pool_after_the_two_overwrites.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );

    let forced = run_two_overwrites_with(ReuseWindow::ForcedToZero, "must-red-reuse-window-forced");
    // 被盖掉的那片数据的校验和与父指针对不上：I-2.1 红；从候选根出发的遍历要红（C22（刚释放的块立即重分配） 第 ② 条点名 I-4.8）
    images.hand_out(
        "复用窗口置 0、覆盖写两次之后",
        || forced.pool_after_the_two_overwrites.memory_pool(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated("I-2.1")
            .and_violated("I-4.8"),
    );
}
