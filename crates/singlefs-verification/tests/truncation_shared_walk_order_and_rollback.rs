//! 截断进共用走空与管理员回退（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1）：走空的次序是 `deleted_inodes` →
//! 销毁中的快照 → 截断；回退时 logged_ops 按种类分来源，截断条目取 R_old 那一版的——回退到截断批间那一版，续做照它那一条截到底；
//! 截到一半回退到截断之前那一版，那一条随回退去掉、不截 R_old 的内容（攻方 H1、H2 两个方向）。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_truncation;

use common::{parameters, BuiltPool};
use common_deletion_in_progress::{
    content_of_three_data_units, pool_with_a_three_unit_file,
    publish_new_inodes_and_advance_the_pool, publish_the_intent_of, roll_back_to,
};
use common_truncation::{
    assert_both_readers_read, batches_of_one, change_the_size_of_the_first_file_and_advance,
    publish_the_next_truncation_step_and_advance, truncation_entry_of, walk_the_queues_and_advance,
    INSTANCE_OF_THE_SIZE_CHANGES,
};
use singlefs_core::address::TreeIdentifier;
use singlefs_core::records::TREE_KIND_SNAPSHOT_INODE_ROOT;
use singlefs_core::transaction::{
    publish_create_snapshot, publish_snapshot_destruction_intent, PoolWriter, TransactionOutput,
};

/// 接着 `pool.output` 建一个快照，交回它码 14 条目的树号。
fn create_a_snapshot_and_advance(pool: &mut BuiltPool) -> TreeIdentifier {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let created = publish_create_snapshot(
        &mut writer,
        &mut pool.allocator,
        &previous,
        INSTANCE_OF_THE_SIZE_CHANGES,
    )
    .expect("建快照");
    pool.output = created.clone();
    created
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("建快照那一版有一条码 14")
        .tree
}

/// 接着 `pool.output` 发删快照的意图。
fn publish_the_snapshot_destruction_intent_and_advance(
    pool: &mut BuiltPool,
    snapshot: TreeIdentifier,
) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_snapshot_destruction_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        snapshot,
        INSTANCE_OF_THE_SIZE_CHANGES,
    )
    .expect("删快照的意图");
}

/// 走空的一步做的是哪一样：`deleted_inodes` 变了是 unlink；第一个文件的单元数或截断条目变了是截断；别的是删快照。
fn kind_of_the_step(previous: &TransactionOutput, step: &TransactionOutput) -> &'static str {
    if previous.deleted_inodes != step.deleted_inodes {
        "deleted_inodes"
    } else if previous.data_pointers.len() != step.data_pointers.len()
        || truncation_entry_of(previous) != truncation_entry_of(step)
    {
        "truncation"
    } else {
        "snapshot being destroyed"
    }
}

/// 三样承诺类队列同时不空（截断意图在 inode 2 的 unlink 意图与删快照的意图之前发：发截断意图时队列是空的）：共用走空先把
/// `deleted_inodes` 走空、再走销毁中的快照、最后走截断（批大小 1：放到两个、最后一批放到一个）。
#[test]
fn the_shared_walk_empties_deleted_inodes_then_the_snapshot_being_destroyed_then_the_truncation() {
    let mut pool = pool_with_a_three_unit_file("truncation-walk-order");
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let snapshot = create_a_snapshot_and_advance(&mut pool);
    change_the_size_of_the_first_file_and_advance(&mut pool, 100);
    publish_the_snapshot_destruction_intent_and_advance(&mut pool, snapshot);
    publish_the_intent_of(&mut pool, 2);
    let before_the_walk = pool.output.clone();
    assert!(
        !before_the_walk.deleted_inodes.is_empty()
            && !before_the_walk.snapshots_being_destroyed.is_empty()
            && truncation_entry_of(&before_the_walk).is_some(),
        "三样队列都不空"
    );
    let steps = walk_the_queues_and_advance(&mut pool, batches_of_one());
    let kinds: Vec<&str> = std::iter::once(&before_the_walk)
        .chain(steps.iter())
        .zip(steps.iter())
        .map(|(previous, step)| kind_of_the_step(previous, step))
        .collect();
    let mut runs: Vec<&str> = kinds.clone();
    runs.dedup();
    assert_eq!(
        runs,
        ["deleted_inodes", "snapshot being destroyed", "truncation"],
        "走空的次序：{kinds:?}"
    );
    assert_eq!(
        kinds.iter().filter(|kind| **kind == "truncation").count(),
        2,
        "截断：放到两个、最后一批放到一个"
    );
    assert!(pool.output.queue_of_committed_intents_is_empty());
}

/// 攻方 H1：截断批间（放到两个、条目还在）那一版之后最后一批做完，再回退到批间那一版：回退那次的截断条目取 R_old 的，
/// 走空续做照它截到底，终态是截后的 100 字节、一个单元。
#[test]
fn a_rollback_to_a_version_between_truncation_batches_takes_its_truncation_entry_and_the_walk_finishes_the_truncation(
) {
    let mut pool = pool_with_a_three_unit_file("truncation-rollback-between-batches");
    let content = content_of_three_data_units();
    change_the_size_of_the_first_file_and_advance(&mut pool, 100);
    let between_the_batches =
        publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
    publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
    assert!(
        truncation_entry_of(&pool.output).is_none(),
        "最后一批删了条目"
    );
    let rolled_back = roll_back_to(&mut pool, &between_the_batches);
    assert_eq!(
        truncation_entry_of(&rolled_back),
        truncation_entry_of(&between_the_batches),
        "回退那次的截断条目取 R_old 的"
    );
    assert_eq!(rolled_back.data_pointers.len(), 2, "单元取 R_old 的：两个");
    let steps = walk_the_queues_and_advance(&mut pool, batches_of_one());
    assert_eq!(steps.len(), 1, "续做一个最后一批");
    assert_eq!(pool.output.data_pointers.len(), 1);
    assert!(truncation_entry_of(&pool.output).is_none());
    assert_both_readers_read(
        &pool.memory_pool(),
        &pool.output.root,
        &content[..100],
        "续做到底",
    );
}

/// 攻方 H2：截到一半（条目还在）时回退到截断之前那一版：R_old 没有截断条目，回退那次去掉它；走空一个写都不发，读回截断之前的整份内容。
#[test]
fn a_rollback_from_the_middle_of_a_truncation_to_a_version_before_it_drops_the_truncation_entry() {
    let mut pool = pool_with_a_three_unit_file("truncation-rollback-before-the-intent");
    let content = content_of_three_data_units();
    let before_the_truncation = pool.output.clone();
    change_the_size_of_the_first_file_and_advance(&mut pool, 100);
    publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
    assert!(truncation_entry_of(&pool.output).is_some(), "截到一半");
    let rolled_back = roll_back_to(&mut pool, &before_the_truncation);
    assert!(
        truncation_entry_of(&rolled_back).is_none(),
        "R_old 没有截断条目：回退那次去掉它"
    );
    assert!(
        walk_the_queues_and_advance(&mut pool, batches_of_one()).is_empty(),
        "走空一个写都不发"
    );
    assert_both_readers_read(
        &pool.memory_pool(),
        &pool.output.root,
        &content,
        "截断之前的整份内容",
    );
}
