//! 可写挂载时的预隔离（D23（journal 的角色与格式） 已定项 14「可写挂载时的预隔离」）只豁免现行账里仍分配着的槽（D28（挂载期承诺量） 已定项 1
//! 第九项：「只除去现行账里仍分配着的槽……这次挂载里释放、回收时分配器照补隔离」）：这样的槽在这次挂载里被释放、过了回收门槛，
//! 而那条被抛弃根的根环槽还没被这个进程写掉时，分配器回收它的那一刻照还没解开的那一份补隔离（`PoolAllocator::record_root_written_by_this_process`
//! 里回收之后补隔离看的集合并进水位之上没施加的记录点名的落点），它在那一槽写掉之前不被取到。
//!
//! 造法（`tests/common/mod.rs`，4 GiB 两块盘，准入宽、不推抬 F：回收只走根环的回收门槛那一条）：第一个文件 A（txg 3）之后同一个进程里覆盖写
//! B（4）、C（5），崩溃恢复抛弃 C（`abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：C 的根槽与数据单元在那次挂载里读不出、
//! 系统配置没见证 C），择根落到 B，实例 2 写行（txg 6）、暖机。C 的记录读得出、不施加：C 的记录新根段与 C 的账里有 B 的现行账里仍分配着的落点，
//! 挂载那一刻按窄读豁免、不隔离；写行那次发布换掉实例表那一类单元、释放它们。之后实例 2 照常覆盖写：C 的根槽成了环里最小的那个时进程内重读读出 C、
//! 按实例表判被抛弃、当没有根，回收门槛越过写行那一版——那几个槽在 C 的根环槽被写掉的前一次发布里回收，这时 C 的记录点名的那一份还没解开。
mod common;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, parameters,
    publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{AllocationRecord, DeviceFreeMap, PlacementOnDevice, PoolAllocator};
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::transaction::TransactionOutput;

fn content_seeded_by(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 同一个进程里对建好的池覆盖写一次，`pool.output` 换成这一版。
fn overwrite_the_built_pool(
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
    .expect("4 GiB 两块盘上这一串覆盖写都落得下");
    pool.output = output.clone();
    output
}

/// 分配器记账里罩住 `placement` 起点那一槽的那条记录。
fn record_covering<'allocator>(
    allocator: &'allocator PoolAllocator,
    placement: &PlacementOnDevice,
) -> Option<&'allocator AllocationRecord> {
    allocator.records().iter().find(|record| {
        record.device == placement.device
            && record.slot.0 <= placement.slot.0
            && placement.slot.0 < record.slot.0 + u64::from(record.span_slots)
    })
}

fn device_map_of<'allocator>(
    allocator: &'allocator PoolAllocator,
    placement: &PlacementOnDevice,
) -> &'allocator DeviceFreeMap {
    allocator
        .devices
        .iter()
        .find(|device_map| device_map.device == placement.device)
        .expect("落点的盘在池里")
}

/// `placement` 跨的每一槽在分配器上都不空闲（已分配、隔离着或被抬 F 扣着）。
fn every_slot_of_the_placement_is_held(
    allocator: &PoolAllocator,
    placement: &PlacementOnDevice,
) -> bool {
    let device_map = device_map_of(allocator, placement);
    (placement.slot.0..placement.slot.0 + placement.span)
        .all(|slot| !device_map.is_free(SlotNumber(slot)))
}

/// `placement` 跨的每一槽都已回收（分配位清了、没被抬 F 扣着）：在空闲图的一份拷贝上清掉隔离位之后它就空闲。
fn every_slot_of_the_placement_is_reclaimed(
    allocator: &PoolAllocator,
    placement: &PlacementOnDevice,
) -> bool {
    let mut without_the_isolation = device_map_of(allocator, placement).clone();
    (placement.slot.0..placement.slot.0 + placement.span).all(|slot| {
        without_the_isolation.clear_isolation_of_slot(SlotNumber(slot));
        without_the_isolation.is_free(SlotNumber(slot))
    })
}

/// C 的记录与账点名的、B 的现行账里仍分配着的槽，写行那次发布释放了它们；C 的根环槽被这个进程写掉之前它们被回收过（过了回收门槛，
/// 抬 F 一次都没推），而每一次覆盖写之后它们都不空闲、也没被重新分配出去；写掉 C 那一槽的那次发布同样没取到它们。
/// 判别力：回收之后补隔离不看水位之上没施加的记录点名的落点（变异「第37批 回收时补隔离不看水位之上没施加的记录点名的落点」，删掉
/// `placements_the_isolation_still_protects` 里那一句 `protected.extend`）时，回收出来的那几个槽当场空闲，「不空闲」那一条红。
#[test]
fn a_slot_of_the_current_account_named_by_an_unapplied_record_stays_isolated_after_its_reclaim_until_the_root_ring_slot_of_that_record_is_overwritten(
) {
    let mut pool = build_pool("released-slot-named-by-an-unapplied-record");
    overwrite_the_built_pool(
        &mut pool,
        &content_seeded_by(4100, 3),
        InstanceGeneration(1),
    );
    let abandoned = overwrite_the_built_pool(
        &mut pool,
        &content_seeded_by(2500, 11),
        InstanceGeneration(1),
    );
    let mounted =
        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &abandoned);
    assert_eq!(
        (
            abandoned.root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.instance
        ),
        (CheckpointTxg(5), CheckpointTxg(4), InstanceGeneration(2)),
        "C 是 txg 5；恢复落到 B（txg 4），实例 2"
    );
    let row_publish_txg = mounted.output.row_publish.root().checkpoint_txg;
    let floor_at_the_mount = pool.output.root.rollback_floor;
    let released_by_the_row_publish: Vec<PlacementOnDevice> = pool
        .allocator
        .root_ring_occupancy()
        .expect("可写挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records()
        .into_iter()
        .filter(|placement| {
            record_covering(&pool.allocator, placement)
                .is_some_and(|record| record.is_released && record.generation == row_publish_txg)
        })
        .collect();
    assert!(
        !released_by_the_row_publish.is_empty(),
        "C 的记录与账点名的落点里有 B 的现行账里仍分配着、写行那次发布（txg {row_publish_txg:?}）释放了的"
    );
    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
    let ring_slot_of_c = target_for_publish(abandoned.root.checkpoint_txg, slots_per_region);
    let overwrites_limit = 4 * slots_per_region.count();
    let mut overwrites: u64 = 0;
    let mut reclaimed_before_the_ring_slot_of_c_was_overwritten = false;
    // 迭代上界 overwrites_limit（根环三区、每区 S 槽，一圈 3S 次发布之内必写到 C 那一槽）；跨轮携带「回收过」一个布尔；
    // 出口是写掉 C 那一槽的那一次发布。
    loop {
        overwrites += 1;
        let current_txg = overwrite_the_built_pool(
            &mut pool,
            &content_seeded_by(3000, usize::try_from(overwrites).expect("轮数装得进 usize")),
            InstanceGeneration(2),
        )
        .root
        .checkpoint_txg;
        assert_eq!(
            pool.output.root.rollback_floor, floor_at_the_mount,
            "第 {overwrites} 次覆盖写（txg {current_txg:?}）：没推抬 F（回收只走根环的回收门槛）"
        );
        for placement in &released_by_the_row_publish {
            assert!(
                !record_covering(&pool.allocator, placement).is_some_and(|record| {
                    !record.is_released && record.generation > row_publish_txg
                }),
                "第 {overwrites} 次覆盖写（txg {current_txg:?}）：C 的根环槽写掉之前（含写掉它的那一次发布）{placement:?} 没被重新分配"
            );
        }
        if target_for_publish(current_txg, slots_per_region) == ring_slot_of_c {
            break;
        }
        for placement in &released_by_the_row_publish {
            if every_slot_of_the_placement_is_reclaimed(&pool.allocator, placement) {
                reclaimed_before_the_ring_slot_of_c_was_overwritten = true;
            }
            assert!(
                every_slot_of_the_placement_is_held(&pool.allocator, placement),
                "第 {overwrites} 次覆盖写（txg {current_txg:?}）之后 C 那一槽还没被盖掉：{placement:?} 不空闲（回收了也照补隔离）"
            );
        }
        assert!(
            overwrites < overwrites_limit,
            "{overwrites_limit} 次覆盖写之内这个进程写到 C 那一槽"
        );
    }
    assert!(
        reclaimed_before_the_ring_slot_of_c_was_overwritten,
        "C 的根环槽写掉之前写行那次发布释放的槽过了回收门槛、被回收过（这一形才走到回收之后补隔离那一步）"
    );
}
