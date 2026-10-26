//! C145 第三条「放块时要删的那条 deadlist 条目必须还在」（用户 2026-10-26 定「加断言并留档」）的等价性留档
//! （`.claude/singlefs-ai-sop/rules/test-discipline.md`「等价变异不算盲区，但要分开记」）：删快照每一批放掉的条目由
//! `transaction::deadlist_entries_the_destruction_of_a_snapshot_releases` 从上一版的 deadlist 里挑，放块那一处的 `entries_after` 也从同一份 deadlist 建，
//! 所以那条断言在所有走得到的输入上都成立，删掉它的变异是等价变异、没有用例抓得到。这里把「挑出来的每一条都在它挑的那一份 deadlist 里、
//! 一条不重」钉成一条用例，并核这一条真的走到了（挑出来的不空、删快照照常做完、deadlist 一条不剩）。改变行为的那一条变异
//! （放块那一处不删条目）由 `crates/mutations.tsv`「C145 删快照放块时不删 deadlist 条目」那一行钉在既有用例上。
mod common;

use std::collections::BTreeSet;

use common::{
    build_pool, parameters, publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration, TreeIdentifier};
use singlefs_core::records::{DeadlistEntry, TREE_KIND_SNAPSHOT_INODE_ROOT};
use singlefs_core::transaction::{
    deadlist_entries_the_destruction_of_a_snapshot_releases, publish_create_snapshot,
    publish_snapshot_destruction_intent, walk_the_deletion_queue_until_it_is_empty, PoolWriter,
    TransactionOutput,
};
use singlefs_core::unlink::UnlinkBatchSize;

const INSTANCE: InstanceGeneration = InstanceGeneration(1);

/// 同一个进程里接着 `pool.output` 建一个快照，交回它码 14 条目的树号；池接到那一版上。
fn create_snapshot_and_advance(pool: &mut BuiltPool) -> TreeIdentifier {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let created = publish_create_snapshot(&mut writer, &mut pool.allocator, &previous, INSTANCE)
        .expect("建快照");
    pool.output = created.clone();
    created
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("建快照那一版有一条码 14")
        .tree
}

/// 一组 deadlist 条目按盘上字节排成的集合（条目没有全序）。
fn deadlist_keys(entries: &[DeadlistEntry]) -> BTreeSet<Vec<u8>> {
    entries.iter().map(DeadlistEntry::key_bytes).collect()
}

/// 快照下覆盖写两次（被快照引用的单元进 deadlist），发删快照的意图：在意图那一版上按下一批自己的 txg 挑要放的条目，挑出来的不空、
/// 每一条都在意图那一版的 deadlist 里、键一条不重——放块那一处要删的条目恒在；之后走空把快照删完，deadlist 一条不剩（那条断言一次都没触发）。
#[test]
fn every_deadlist_entry_a_snapshot_destruction_releases_is_in_the_deadlist_the_release_starts_from()
{
    let mut pool = build_pool("snapshot-destruction-releases-entries-of-its-deadlist");
    let snapshot = create_snapshot_and_advance(&mut pool);
    for seed in [3_u32, 5] {
        let previous = pool.output.clone();
        let content: Vec<u8> = (0..2700_u32)
            .map(|index| u8::try_from((index * 13 + seed) % 239).expect("小于 256"))
            .collect();
        pool.output = publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content,
            FIXED_WRITE_TIME_SECONDS + 60,
            INSTANCE,
        )
        .expect("快照下覆盖写");
    }
    let intent: TransactionOutput = {
        let publish_parameters = parameters();
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_snapshot_destruction_intent(
            &mut writer,
            &mut pool.allocator,
            &previous,
            snapshot,
            INSTANCE,
        )
        .expect("删快照的意图")
    };
    pool.output = intent.clone();
    let released = deadlist_entries_the_destruction_of_a_snapshot_releases(
        &intent,
        snapshot,
        CheckpointTxg(intent.root.checkpoint_txg.0 + 1),
    );
    assert!(
        !released.is_empty(),
        "前提：这一批真要放条目（快照下覆盖写换下的单元进了 deadlist）"
    );
    let deadlist_before_the_release = deadlist_keys(&intent.deadlist_entries);
    let released_keys = deadlist_keys(&released);
    assert_eq!(
        released_keys.len(),
        released.len(),
        "挑出来的条目键一条不重"
    );
    assert!(
        released_keys.is_subset(&deadlist_before_the_release),
        "挑出来的每一条都在它挑的那一份 deadlist 里：多出 {:?}",
        released_keys
            .difference(&deadlist_before_the_release)
            .collect::<Vec<_>>()
    );
    let publish_parameters = parameters();
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let walk = walk_the_deletion_queue_until_it_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        INSTANCE,
    );
    walk.stopped_by.expect("删快照走到底");
    assert!(
        current.deadlist_entries.is_empty() && current.snapshots_being_destroyed.is_empty(),
        "删快照做完：deadlist 一条不剩"
    );
}
