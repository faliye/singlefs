//! harness 测试文件 `crates/singlefs-harness/tests/root_ring_turn_in_the_make_filesystem_session.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{build_pool, FIXED_WRITE_TIME_SECONDS};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};

/// 覆盖写到这一条根为止：根环 3 × 8 = 24 槽，txg 26 那条根盖掉 txg 2 之后环里最旧有效根是 3，第一个文件那次（txg 3）换下的
/// mkfs 树表从这里起可再分配；再往后写几条，让回收在好几条根上都发生过。
pub const TXG_OF_THE_LAST_OVERWRITE: CheckpointTxg = CheckpointTxg(30);

/// txg 30 那条根盖掉 txg 6 之后环里最旧的有效根（30 − 24 + 1）。
pub const OLDEST_VALID_ROOT_AFTER_THE_LAST_OVERWRITE: CheckpointTxg = CheckpointTxg(7);

/// 第 `overwrite_index` 次覆盖写的内容：每次长度与字节都不同。
pub fn content_of_overwrite(overwrite_index: usize) -> Vec<u8> {
    (0..3000 + overwrite_index * 37)
        .map(|index| u8::try_from((index * 11 + overwrite_index * 5 + 1) % 251).expect("小于 256"))
        .collect()
}

/// 不装根环表时（C551 那一形）：第一个文件那次释放的 mkfs 树表（释放代 3）起，被盖出根环的根引用过的落点一直留在 defer 里，
/// txg 26 起 I-3.1 红；装了之后环每转一格就按谓词回收，到 txg 30 盘上的账与遍历全部有效根得到的对得上。
pub fn overwrites_in_the_make_filesystem_session_past_one_turn_of_the_root_ring_reclaim_and_keep_every_invariant(
    images: &mut PoolCheckerImages,
) {
    let mut pool = build_pool("root-ring-turn-in-the-make-filesystem-session");
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(3),
        "前提：build_pool 交回第一个文件那一版（txg 3）"
    );
    let mut overwrite_index = 0usize;
    // 迭代上界：每次覆盖写 txg 加一，从 3 到 30 共 27 次；跨轮携带的是现行那一版。
    while pool.output.root.checkpoint_txg < TXG_OF_THE_LAST_OVERWRITE {
        let previous = pool.output.clone();
        pool.output = crate::common::publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content_of_overwrite(overwrite_index),
            FIXED_WRITE_TIME_SECONDS + 60,
            InstanceGeneration(1),
        )
        .unwrap_or_else(|error| panic!("第 {} 次覆盖写：{error:?}", overwrite_index + 1));
        overwrite_index += 1;
    }
    assert_eq!(
        pool.output.root.checkpoint_txg, TXG_OF_THE_LAST_OVERWRITE,
        "覆盖写到 txg 30，根环转过了一圈"
    );
    // 记了 txg 30 那条根之后环里是 txg 7–30（它盖掉 txg 6），最旧有效根是 7、F 是 0：释放代 ≤ 7 的都该回收了，
    // defer 里只剩释放代比 7 新的。不装根环表时一个都不回收，释放代 3–7 的也还在 defer 里。
    for device_map in &pool.allocator.devices {
        let released_after_the_oldest_valid_root: u64 = pool
            .allocator
            .records()
            .iter()
            .filter(|record| {
                record.device == device_map.device
                    && record.is_released
                    && record.generation > OLDEST_VALID_ROOT_AFTER_THE_LAST_OVERWRITE
            })
            .map(|record| u64::from(record.span_slots))
            .sum();
        assert_eq!(
            device_map.deferred_slots(),
            released_after_the_oldest_valid_root,
            "盘 {:?}：分配器自己的账上 defer 里只剩释放代比环里最旧有效根（txg 7）新的槽",
            device_map.device
        );
    }
    // 转过根环之后 I-3.1 判过且成立，池级 checker 一条不红
    images.hand_out(
        "mkfs 会话里覆盖写转过根环之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated().and_holds("I-3.1"),
    );
}
