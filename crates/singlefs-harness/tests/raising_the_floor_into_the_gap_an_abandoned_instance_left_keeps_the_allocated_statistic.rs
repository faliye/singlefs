//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 43 行取丁-defer（用户 2026-09-28 定，被攻过一轮；
//! `research/prompts/abandoned-floor-r1-main-verification.md` M1）：I-3.1（已分配统计对得上） 的「实际遍历」并上 F_生效 之下、没被抛弃的根
//! 引用着、候选集那一遍没走到、最新根的账里是已释放且释放代高于 F_生效 的槽（还在 defer 里）。
//!
//! 历史只靠崩溃恢复造被抛弃实例（调查员报告 `research/prompts/closeout-recheck-2026-09-28/row43-investigator-report.md` 的最小复现）：
//! A（txg 3）→ 覆盖写 B（4）、C（5）→ C 的系统配置轮换没落盘就崩（轮换写全部退回崩溃前的字节，两槽照样自证得过）→ 可写挂载时 C 的根槽与
//! 数据单元一时读不出，落到 B、实例 2 → 再可写挂载一次（影子账看见 C）→ 覆盖写四次 → 抬 F 到 5。每块盘差的 10 槽释放代 6、
//! 只被 B（txg 4，F 之下）与被抛弃的 C 引用、还在 defer 里：改之前只有 F = 5 那一格 I-3.1 红（记账多于遍历），并上之后全绿。
mod common;

use common::{
    abandon_the_third_version_by_a_crash_before_its_rotation, build_pool, parameters,
    publish_overwrite_in_process, system_configuration_slots_of_every_device, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{mount_writable, raise_rollback_floor, ShadowLedger};
use singlefs_core::transaction::TransactionOutput;
use singlefs_harness::memory_pool::MemoryPool;

fn content_seeded_by(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn overwrite(pool: &mut BuiltPool, content: &[u8], instance: InstanceGeneration) -> TransactionOutput {
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

fn violations_on(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

/// A、B、C（实例 1）→ 崩溃恢复抛弃 C → 再可写挂载一次 → 覆盖写四次：抬 F 之前全绿的池，与再挂载那一次的实例代号。
fn pool_before_the_raise(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite(&mut pool, &content_seeded_by(4100, 3), InstanceGeneration(1));
    let system_configuration_before_the_third = system_configuration_slots_of_every_device(&pool);
    let third = overwrite(&mut pool, &content_seeded_by(2500, 11), InstanceGeneration(1));
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
    assert_eq!(
        violations_on(&pool.memory_pool()),
        Vec::<(&'static str, String)>::new(),
        "抬 F 之前池级 checker 全绿"
    );
    pool
}

fn raise_the_floor_and_judge(tag: &str, floor: u64) -> Vec<(&'static str, String)> {
    let mut pool = pool_before_the_raise(tag);
    raise_rollback_floor(
        &parameters(),
        pool.devices.as_mut().expect("镜像开着"),
        &mut pool.allocator,
        &mut pool.output,
        CheckpointTxg(floor),
        ShadowLedger::On,
    )
    .expect("抬 F");
    violations_on(&pool.memory_pool())
}

/// F 抬到 C 那个 txg（5）：F 落进被抛弃实例留下的空档，B（txg 4）掉到 F 之下、C 被抛弃，只被它俩引用、释放代 6 的 10 槽还在 defer 里。
#[test]
fn raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_every_invariant_green() {
    assert_eq!(
        raise_the_floor_and_judge("row43-gap-floor-5", 5),
        Vec::<(&'static str, String)>::new(),
        "F 之下仍在 defer 的槽并进遍历：I-3.1 不红"
    );
}

/// 对照：同一段历史只换 F（6：释放代 6 的那 10 槽已可回收），改之前改之后都全绿。
#[test]
fn raising_the_floor_past_the_gap_keeps_every_invariant_green() {
    assert_eq!(
        raise_the_floor_and_judge("row43-gap-floor-6", 6),
        Vec::<(&'static str, String)>::new(),
        "F 抬过空档：全绿"
    );
}
