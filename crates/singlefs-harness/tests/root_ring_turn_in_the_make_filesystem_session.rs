//! mkfs 同一个进程那条会话（`common::build_pool`：取号、暖机、第一个文件，之后一次挂载都不做）里连着覆盖写、转过根环：
//! 分配器装着 mkfs 刚写下的根环（C551（`build_pool` 的分配器没有根环表）；用户 2026-10-05 定 core 在没装根环表而记根时断言），
//! 环转过之后按可再分配谓词回收（D16（发布语义） 已定项 1：已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)），池级 checker 一条不红——
//! I-3.1（已分配统计对得上）是分配器不装根环表、一次不回收时红的那一条（E171 流三与 C551 的连续覆盖写都是这一形）。

mod common;

use common::{build_pool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};

/// 覆盖写到这一条根为止：根环 3 × 8 = 24 槽，txg 26 那条根盖掉 txg 2 之后环里最旧有效根是 3，第一个文件那次（txg 3）换下的
/// mkfs 树表从这里起可再分配；再往后写几条，让回收在好几条根上都发生过。
const TXG_OF_THE_LAST_OVERWRITE: CheckpointTxg = CheckpointTxg(30);

/// txg 30 那条根盖掉 txg 6 之后环里最旧的有效根（30 − 24 + 1）。
const OLDEST_VALID_ROOT_AFTER_THE_LAST_OVERWRITE: CheckpointTxg = CheckpointTxg(7);

/// 第 `overwrite_index` 次覆盖写的内容：每次长度与字节都不同。
fn content_of_overwrite(overwrite_index: usize) -> Vec<u8> {
    (0..3000 + overwrite_index * 37)
        .map(|index| u8::try_from((index * 11 + overwrite_index * 5 + 1) % 251).expect("小于 256"))
        .collect()
}

/// 不装根环表时（C551 那一形）：第一个文件那次释放的 mkfs 树表（释放代 3）起，被盖出根环的根引用过的落点一直留在 defer 里，
/// txg 26 起 I-3.1 红；装了之后环每转一格就按谓词回收，到 txg 30 盘上的账与遍历全部有效根得到的对得上。
#[test]
fn overwrites_in_the_make_filesystem_session_past_one_turn_of_the_root_ring_reclaim_and_keep_every_invariant(
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
        pool.output = common::publish_overwrite_in_process(
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
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        verdicts
            .iter()
            .find(|(name, _)| *name == "I-3.1")
            .map(|(_, verdict)| verdict),
        Some(&InvariantVerdict::Holds),
        "转过根环之后 I-3.1 判过且成立"
    );
    let violated: Vec<_> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .collect();
    assert!(
        violated.is_empty(),
        "转过根环之后池级 checker 一条不红：{violated:?}"
    );
}
