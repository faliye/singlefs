//! harness 测试文件 `crates/singlefs-harness/tests/random_histories.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
    publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::transaction::TransactionOutput;

pub fn content_seeded_by(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 同一个进程里对建好的池覆盖写一次，`pool.output` 换成这一版。
pub fn overwrite_the_built_pool(
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

/// 建池（第一个文件 A，txg 3）→ 同一个进程里覆盖写 B（txg 4）、C（txg 5）→ 崩溃恢复抛弃 C：`tests/common` 的
/// `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`——C 的根槽与数据单元暂时读不出、见证 C 的系统配置槽坏掉
/// （系统配置没见证到 C：C554 乙罩不到、照旧抛弃的那一形），择根落到 B、实例 2 写行 txg 6、暖机 txg 7，再把 C 写回，C 按实例 2 的表判被抛弃。
/// 随机历史生成器的「崩溃恢复抛弃根」一步藏的是系统配置见证过的最新根，C554 乙之后一律被拒、造不出被抛弃的根，要它的用例从这里造。
/// 交回池（现行版本是实例 2 暖机那一版）与 C 那一版。
pub fn pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version(
    tag: &str,
) -> (BuiltPool, TransactionOutput) {
    let mut pool = build_pool(tag);
    overwrite_the_built_pool(
        &mut pool,
        &content_seeded_by(4100, 3),
        InstanceGeneration(1),
    );
    let third = overwrite_the_built_pool(
        &mut pool,
        &content_seeded_by(2500, 11),
        InstanceGeneration(1),
    );
    let abandoning =
        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third);
    assert_eq!(
        (
            third.root.checkpoint_txg,
            abandoning.output.effective_root.checkpoint_txg,
            abandoning.output.instance,
            abandoning.output.row_publish.root().checkpoint_txg,
            pool.output.root.checkpoint_txg
        ),
        (
            CheckpointTxg(5),
            CheckpointTxg(4),
            InstanceGeneration(2),
            CheckpointTxg(6),
            CheckpointTxg(7)
        ),
        "C 是 txg 5；恢复落到 B（txg 4），实例 2 写行 txg 6、暖机 txg 7"
    );
    (pool, third)
}

/// 双故障形（C583（双故障形：每盘一槽系统配置坏加最新根一时读不出））：第一个文件 A（txg 3）之后覆盖写 B（4）、C（5），
/// 崩溃恢复抛弃 C——择根落到 B，实例 2 写行 6、暖机 7；再覆盖写四次（8–11）。那次挂载时 C 的根槽读不出、系统配置没见证 C（乙罩不到的那一格，
/// `research/prompts/m2-impl-c554-yi-implementer-report.md` 第六节 Q1），影子账看不见 C；可写挂载时的预隔离（D23（journal 的角色与格式） 已定项 14
/// 「可写挂载时的预隔离」）把水位之上没施加的 C 的记录点名的落点与 C 的账里的落点隔离到这个进程写掉 txg 5 那一槽为止，四次覆盖写发不出 C 的单元，
/// 抬 F 之前池级 checker 一条都不红。被抛弃的 C 由 [`pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version`] 造
/// （系统配置没见证到 C 的那一形；生成器的那一步 C554 乙之后造不出）。
/// 判别力：挂载时不隔离那一份（变异「第37批 可写挂载时的预隔离不隔离水位之上没施加的记录点名的落点」）时 C 引用的单元又被发出去，
/// I-7.4（近 K 代块未被复用）红、点名实例 1、txg 5 那条被抛弃的根，这一条红。
pub fn crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, abandoned) =
        pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version("random-history-row-43");
    for seed in [17usize, 19, 23, 29] {
        overwrite_the_built_pool(
            &mut pool,
            &content_seeded_by(3000 + seed, seed),
            InstanceGeneration(2),
        );
    }
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(11));
    assert_eq!(
        (abandoned.root.instance, abandoned.root.checkpoint_txg),
        (InstanceGeneration(1), CheckpointTxg(5)),
        "被抛弃的是 C：实例 1、txg 5"
    );
    // 抬 F 之前池级 checker 一条都不红（C 的单元被预隔离护着）
    images.hand_out(
        "实例 2 覆盖写四次之后、抬 F 之前",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
