//! 快照下的释放分流、deadlist、映射条目留存与删快照（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3、S4、S8
//! 与第七节岔路 2、3、4；D5（快照 / 空间记账机制） 已定项 12、13、14、16、19、20）。
//!
//! 释放分流：这次换下的用户可见单元（数据单元、extent 树、inode 叶容器与根）按映射 key 里的出生 txg 与活头的 `previous_snapshot_txg` 判，
//! birth ≤ p 的进 deadlist（头 = 活头 inode 树、纪元 = p、映射 key 照上一版）、不释放、映射条目留存；其余照旧进 defer。
//! 池级元数据（分配记录树、记账树、中央映射树、树表……）有快照时也直接进 defer。
//! 删快照：意图写在码 14 条目 flags 位 0、前驱 P 传给下一侧；之后按批放纪元落在 [lo, hi) 里、birth > P 的条目，其余键不动、
//! 落进下一侧那一段；最后一批同一次发布删掉那一对条目。崩在意图与最后一批之间，下一次可写挂载续做。
//! 期望值按条款从被测之前那一版现算（它的映射 key、它的单元落点、建快照那次的 txg），读回一律从盘上重建。
mod common;

use std::collections::BTreeSet;

use common::{
    build_pool, disk_snapshot, file_content, parameters, publish_overwrite_in_process, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration, SlotNumber, TreeIdentifier,
};
use singlefs_core::mount::{
    mount_writable, roll_back_by_a_forward_publish, RollbackError, RollbackTarget,
    VersionOfTheRollback,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{
    DeadlistEntry, TREE_KIND_DEADLIST, TREE_KIND_INODE, TREE_KIND_SNAPSHOT_EXTENT_ROOT,
    TREE_KIND_SNAPSHOT_INODE_ROOT,
};
use singlefs_core::recovery::{
    read_the_first_file_of_a_snapshot, rebuild_version_with_its_snapshot_entries, RebuiltVersion,
    SnapshotFileReading,
};
use singlefs_core::transaction::{
    asks_the_snapshot_lifetime_judgement, central_mapping_locations_in_the_version,
    publish_create_snapshot, publish_snapshot_destruction_intent, publish_unlink_intent,
    walk_the_deletion_queue_until_it_is_empty, FirstFile, MappingLookup, PoolVersion, PoolWriter,
    PublishError, SnapshotCreationRefusal, SnapshotDestructionRefusal, TransactionOutput,
    TransactionUnit,
};
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};

const INSTANCE: InstanceGeneration = InstanceGeneration(1);

/// 覆盖写的内容：与第一个文件（`common::file_content`）不同的一份。
fn overwrite_content(seed: u32) -> Vec<u8> {
    (0..2700_u32)
        .map(|index| u8::try_from((index * 13 + seed) % 239).expect("小于 256"))
        .collect()
}

/// 同一个进程里接着 `pool.output` 建一个快照，池接到它那一版上。
fn create_snapshot_and_advance(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let created = publish_create_snapshot(&mut writer, &mut pool.allocator, &previous, INSTANCE)
        .expect("建快照");
    pool.output = created.clone();
    created
}

/// 同一个进程里接着 `pool.output` 覆盖写一次，池接到它那一版上。
fn overwrite_and_advance(pool: &mut BuiltPool, seed: u32) -> TransactionOutput {
    let previous = pool.output.clone();
    let overwritten = publish_overwrite_in_process(
        pool,
        &previous,
        &overwrite_content(seed),
        FIXED_WRITE_TIME_SECONDS + 60,
        INSTANCE,
    )
    .expect("覆盖写");
    pool.output = overwritten.clone();
    overwritten
}

/// 删快照的意图那次发布，池接到它那一版上。
fn publish_the_destruction_intent_and_advance(
    pool: &mut BuiltPool,
    snapshot: TreeIdentifier,
) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let intent = publish_snapshot_destruction_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        snapshot,
        INSTANCE,
    )?;
    pool.output = intent.clone();
    Ok(intent)
}

/// 走空（`deleted_inodes` 与销毁中的快照）到底，交回发成的每一版；池接到最后一版上。
fn walk_the_queues_and_advance(
    pool: &mut BuiltPool,
    batch_size: UnlinkBatchSize,
) -> Vec<TransactionOutput> {
    let publish_parameters = parameters();
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let walk = walk_the_deletion_queue_until_it_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        batch_size,
        INSTANCE,
    );
    walk.stopped_by.expect("走空走到底");
    pool.output = current;
    walk.steps_published
}

/// 删快照：意图 + 全部批（产品批大小），交回意图那一版与之后的每一批。
fn destroy_the_snapshot_and_advance(
    pool: &mut BuiltPool,
    snapshot: TreeIdentifier,
) -> (TransactionOutput, Vec<TransactionOutput>) {
    let intent = publish_the_destruction_intent_and_advance(pool, snapshot).expect("删快照的意图");
    let batches = walk_the_queues_and_advance(pool, UnlinkBatchSize::OneExtentLowerLeaf);
    (intent, batches)
}

/// 从盘上（录制流施加到内存盘上）按 `version` 的根重建那一版。
fn rebuilt_from_disk(pool: &BuiltPool, version: &TransactionOutput) -> TransactionOutput {
    let rebuilt = rebuild_version_with_its_snapshot_entries(
        &pool.memory_pool(),
        &version.root,
        Some(version.record.clone()),
    )
    .expect("盘上这一版重建得出");
    match rebuilt.version {
        RebuiltVersion::WithFile(output) | RebuiltVersion::WithTreesAndNoFile(output) => output,
        RebuiltVersion::WithoutFile => panic!("这些用例的每一版都有树"),
    }
}

/// 一版里的码 14 条目的树号（快照按它点名）。
fn snapshot_trees(version: &TransactionOutput) -> Vec<TreeIdentifier> {
    version
        .tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .map(|entry| entry.tree)
        .collect()
}

/// 一版里第 `kind` 种树那条条目。
fn entry_of_kind(version: &TransactionOutput, kind: u16) -> singlefs_core::records::TreeTableEntry {
    *version
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == kind)
        .expect("这种树在树表里恰好一条")
}

/// 一版里六类用户可见角色各自的映射 key（`mapped_units` 里按角色取）。
fn user_visible_mapping_keys(version: &TransactionOutput) -> Vec<(TransactionUnit, Vec<u8>)> {
    version
        .mapped_units
        .iter()
        .filter(|(identity, _)| asks_the_snapshot_lifetime_judgement(*identity))
        .cloned()
        .collect()
}

/// 一组 deadlist 条目按盘上字节排成的集合（比较用：条目没有全序）。
fn deadlist_keys(entries: &[DeadlistEntry]) -> BTreeSet<Vec<u8>> {
    entries.iter().map(DeadlistEntry::key_bytes).collect()
}

/// 这一槽在池里每块盘上都是在册、没释放的。
fn allocated_on_every_device(pool: &BuiltPool, slot: SlotNumber) -> bool {
    [DeviceIdentity(0), DeviceIdentity(1)].iter().all(|device| {
        pool.allocator
            .record_for(*device, slot)
            .is_some_and(|record| !record.is_released)
    })
}

/// 这一版释放的落点的槽号。
fn released_slots(version: &TransactionOutput) -> BTreeSet<SlotNumber> {
    version
        .released
        .iter()
        .map(|placement| placement.slot)
        .collect()
}

/// 快照下覆盖写：换下的四个用户可见单元（数据单元、extent 根、inode 叶容器、inode 根，都生在建快照之前）全进 deadlist——
/// 头 = 活头 inode 树、纪元 = S.txg、映射 key 照建快照那一版；不进这次的释放、两块盘上的记录仍在册未释放；映射条目照原来的位置项留存。
/// 池级元数据（树表、记账树、中央映射树的根）有快照时也直接释放。盘上读回的 deadlist 与留存的映射条目与发布交回的相同，树表 deadlist 那一条不再是零根。
#[test]
fn an_overwrite_under_a_snapshot_keeps_every_replaced_user_visible_unit_on_the_deadlist_and_releases_the_pool_metadata(
) {
    let mut pool = build_pool("snapshot-release-overwrite-keeps");
    let created = create_snapshot_and_advance(&mut pool);
    let snapshot_txg = created.root.checkpoint_txg;
    let overwritten = overwrite_and_advance(&mut pool, 3);
    let referenced_by_the_snapshot = user_visible_mapping_keys(&created);
    assert_eq!(
        referenced_by_the_snapshot.len(),
        4,
        "第一个文件一个数据单元：数据单元、extent 根、inode 叶容器、inode 根"
    );
    let expected_entries: Vec<DeadlistEntry> = referenced_by_the_snapshot
        .iter()
        .map(|(_, key)| DeadlistEntry {
            head: created.tree_identifiers.inode,
            epoch: snapshot_txg,
            mapping_key: key.clone(),
        })
        .collect();
    assert_eq!(
        deadlist_keys(&overwritten.deadlist_entries),
        deadlist_keys(&expected_entries),
        "四个被快照引用的单元进 deadlist：头 = 活头 inode 树、纪元 = S.txg、映射 key 照建快照那一版"
    );
    let released = released_slots(&overwritten);
    for (identity, key) in &referenced_by_the_snapshot {
        let slot = created.unit(*identity).slot;
        assert!(
            !released.contains(&slot),
            "{identity:?}：被快照引用，不进这次的释放"
        );
        assert!(
            allocated_on_every_device(&pool, slot),
            "{identity:?}：两块盘上的记录仍在册未释放"
        );
        let MappingLookup::Found(locations) =
            central_mapping_locations_in_the_version(&created, key)
        else {
            panic!("{identity:?} 在建快照那一版的映射里查得到");
        };
        assert!(
            overwritten
                .mapping_entries_kept_for_the_snapshots
                .contains(&(key.clone(), locations)),
            "{identity:?}：映射条目照原来的两条位置项留存"
        );
    }
    assert_eq!(
        overwritten.mapping_entries_kept_for_the_snapshots.len(),
        4,
        "留存的映射条目一条 deadlist 条目一条"
    );
    for pool_metadata in [
        TransactionUnit::TreeTable,
        TransactionUnit::AccountingTree,
        TransactionUnit::MappingTree,
    ] {
        assert!(
            released.contains(&created.unit(pool_metadata).slot),
            "{pool_metadata:?}：池级元数据有快照时也直接释放"
        );
    }
    let on_disk = rebuilt_from_disk(&pool, &overwritten);
    assert_eq!(
        deadlist_keys(&on_disk.deadlist_entries),
        deadlist_keys(&overwritten.deadlist_entries),
        "盘上读回的 deadlist 与发布交回的相同"
    );
    assert_eq!(
        on_disk.mapping_entries_kept_for_the_snapshots,
        overwritten.mapping_entries_kept_for_the_snapshots,
        "盘上读回的留存映射条目与发布交回的相同"
    );
    assert_ne!(
        entry_of_kind(&on_disk, TREE_KIND_DEADLIST).root,
        NodePointer::empty_root(),
        "deadlist 树有了根"
    );
}

/// 没有快照时覆盖写：换下的四个用户可见单元照旧释放，deadlist 一条不写、树表 deadlist 那一条仍是零根，没有留存的映射条目。
#[test]
fn an_overwrite_without_any_snapshot_releases_every_replaced_unit_and_leaves_the_deadlist_at_a_zero_root(
) {
    let mut pool = build_pool("snapshot-release-without-a-snapshot");
    let first = pool.output.clone();
    let overwritten = overwrite_and_advance(&mut pool, 5);
    let released = released_slots(&overwritten);
    for (identity, _) in user_visible_mapping_keys(&first) {
        assert!(
            released.contains(&first.unit(identity).slot),
            "{identity:?}：没有快照引用，照旧释放"
        );
    }
    assert!(overwritten.deadlist_entries.is_empty(), "deadlist 一条不写");
    assert!(
        overwritten
            .mapping_entries_kept_for_the_snapshots
            .is_empty(),
        "没有留存的映射条目"
    );
    let on_disk = rebuilt_from_disk(&pool, &overwritten);
    assert_eq!(
        entry_of_kind(&on_disk, TREE_KIND_DEADLIST).root,
        NodePointer::empty_root(),
        "deadlist 树仍是零根"
    );
}

/// 建快照之后生的单元不被快照引用：快照之后覆盖写两次，第二次换下的是第一次写的四个单元（birth > S.txg），全部释放，
/// deadlist 仍是第一次覆盖写留下的那四条。
#[test]
fn a_unit_born_after_the_snapshot_is_released_when_it_is_replaced() {
    let mut pool = build_pool("snapshot-release-born-after");
    create_snapshot_and_advance(&mut pool);
    let first_overwrite = overwrite_and_advance(&mut pool, 7);
    let second_overwrite = overwrite_and_advance(&mut pool, 9);
    let released = released_slots(&second_overwrite);
    for (identity, _) in user_visible_mapping_keys(&first_overwrite) {
        assert!(
            released.contains(&first_overwrite.unit(identity).slot),
            "{identity:?}：生在快照之后，换下就释放"
        );
    }
    assert_eq!(
        deadlist_keys(&second_overwrite.deadlist_entries),
        deadlist_keys(&first_overwrite.deadlist_entries),
        "deadlist 不多也不少：仍是第一次覆盖写留下的那几条"
    );
}

/// 快照下覆盖写之后读快照：读出的是建快照那一刻的第一个文件；活头读出的是覆盖写的新内容。
#[test]
fn the_snapshot_reads_the_content_at_its_creation_after_an_overwrite() {
    let mut pool = build_pool("snapshot-release-read-after-overwrite");
    let created = create_snapshot_and_advance(&mut pool);
    let snapshot = snapshot_trees(&created)[0];
    let overwritten = overwrite_and_advance(&mut pool, 11);
    let image = pool.memory_pool();
    assert_eq!(
        read_the_first_file_of_a_snapshot(&image, &overwritten.root, snapshot),
        Ok(SnapshotFileReading::File(file_content())),
        "快照读出建它那一刻的内容"
    );
    let mut mapping_fallbacks = 0;
    assert_eq!(
        singlefs_core::recovery::walk_to_file(&image, &overwritten.root, &mut mapping_fallbacks),
        Ok(Some(overwrite_content(11))),
        "活头读出覆盖写的内容"
    );
}

/// 快照下删文件（unlink 一次调用走完全部批）之后读快照：仍读出建快照那一刻的内容，活头没有文件；
/// 被删文件的四个单元都进了 deadlist、没释放。
#[test]
fn the_snapshot_reads_the_content_at_its_creation_after_the_file_is_unlinked() {
    let mut pool = build_pool("snapshot-release-read-after-unlink");
    let created = create_snapshot_and_advance(&mut pool);
    let snapshot = snapshot_trees(&created)[0];
    {
        let publish_parameters = parameters();
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_unlink_intent(
            &mut writer,
            &mut pool.allocator,
            &previous,
            InodeNumber(1),
            FIXED_WRITE_TIME_SECONDS + 60,
            INSTANCE,
        )
        .expect("unlink 的意图");
    }
    walk_the_queues_and_advance(&mut pool, UnlinkBatchSize::OneExtentLowerLeaf);
    let emptied = pool.output.clone();
    for (identity, _) in user_visible_mapping_keys(&created) {
        assert!(
            allocated_on_every_device(&pool, created.unit(identity).slot),
            "{identity:?}：快照引用的单元删文件之后仍在册未释放"
        );
    }
    let image = pool.memory_pool();
    assert_eq!(
        read_the_first_file_of_a_snapshot(&image, &emptied.root, snapshot),
        Ok(SnapshotFileReading::File(file_content())),
        "删文件之后快照仍读出建它那一刻的内容"
    );
    let mut mapping_fallbacks = 0;
    assert_eq!(
        singlefs_core::recovery::walk_to_file(&image, &emptied.root, &mut mapping_fallbacks),
        Ok(None),
        "活头没有文件"
    );
}

/// 删唯一的快照：意图那次码 14 带「销毁中」（盘上读得回）、活头的前驱换成它的前驱 0、deadlist 不动；最后一批放掉全部四条
/// （落点进这次的释放、记录改成已释放），同一次发布删掉那一对条目，deadlist 回到零根、没有留存的映射条目。
#[test]
fn destroying_the_only_snapshot_releases_every_entry_and_deletes_its_pair_of_entries() {
    let mut pool = build_pool("snapshot-destroy-the-only-one");
    let created = create_snapshot_and_advance(&mut pool);
    let snapshot = snapshot_trees(&created)[0];
    let overwritten = overwrite_and_advance(&mut pool, 13);
    let (intent, batches) = destroy_the_snapshot_and_advance(&mut pool, snapshot);
    assert_eq!(
        intent.snapshots_being_destroyed,
        vec![snapshot],
        "意图那次带上记号"
    );
    assert_eq!(
        rebuilt_from_disk(&pool, &intent).snapshots_being_destroyed,
        vec![snapshot],
        "记号写在盘上码 14 的 flags 位 0、读得回"
    );
    assert_eq!(
        entry_of_kind(&intent, TREE_KIND_INODE).previous_snapshot_txg,
        CheckpointTxg(0),
        "下一侧是活头：活头的前驱换成被删快照的前驱 0"
    );
    assert_eq!(
        deadlist_keys(&intent.deadlist_entries),
        deadlist_keys(&overwritten.deadlist_entries),
        "意图那次一条 deadlist 条目都不动"
    );
    assert_eq!(batches.len(), 1, "四条不多于一批：意图之后只有最后一批");
    let final_batch = &batches[0];
    let released = released_slots(final_batch);
    for (identity, _) in user_visible_mapping_keys(&created) {
        let slot = created.unit(identity).slot;
        assert!(released.contains(&slot), "{identity:?}：最后一批放掉它");
        assert!(
            !allocated_on_every_device(&pool, slot),
            "{identity:?}：放掉之后记录改成已释放"
        );
    }
    let on_disk = rebuilt_from_disk(&pool, final_batch);
    assert!(snapshot_trees(&on_disk).is_empty(), "那一对条目删掉了");
    assert!(
        !on_disk
            .tree_table_entries
            .iter()
            .any(|entry| entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT),
        "码 15 也删掉了"
    );
    assert!(on_disk.deadlist_entries.is_empty(), "deadlist 一条不剩");
    assert!(
        on_disk.mapping_entries_kept_for_the_snapshots.is_empty(),
        "留存的映射条目一条不剩"
    );
    assert!(
        on_disk.snapshots_being_destroyed.is_empty(),
        "记号随条目一起没了"
    );
    assert_eq!(
        entry_of_kind(&on_disk, TREE_KIND_DEADLIST).root,
        NodePointer::empty_root(),
        "deadlist 树回到零根"
    );
}

/// 两个快照（中间不改）之后覆盖写：四条条目纪元都是 S2.txg、birth 不晚于 S1。删 S2（P = S1.txg）：一条都不放（S1 还引用它们），
/// 键不动、落进活头那一段，活头的前驱换成 S1.txg，S1 仍读出建它那一刻的内容；再删 S1（P = 0，下一侧是活头）：四条全放。
#[test]
fn destroying_the_newer_snapshot_keeps_what_the_older_one_references_and_destroying_the_older_one_releases_it(
) {
    let mut pool = build_pool("snapshot-destroy-newer-then-older");
    let first_snapshot_version = create_snapshot_and_advance(&mut pool);
    let older = snapshot_trees(&first_snapshot_version)[0];
    let second_snapshot_version = create_snapshot_and_advance(&mut pool);
    let newer = *snapshot_trees(&second_snapshot_version)
        .last()
        .expect("两个快照");
    let overwritten = overwrite_and_advance(&mut pool, 17);
    assert!(
        overwritten
            .deadlist_entries
            .iter()
            .all(|entry| entry.epoch == second_snapshot_version.root.checkpoint_txg),
        "杀块那一刻活头的前驱是 S2.txg"
    );
    let (_, batches_of_the_newer) = destroy_the_snapshot_and_advance(&mut pool, newer);
    let after_the_newer = batches_of_the_newer.last().expect("最后一批").clone();
    assert_eq!(
        deadlist_keys(&after_the_newer.deadlist_entries),
        deadlist_keys(&overwritten.deadlist_entries),
        "删 S2：S1 还引用它们，一条都不放、键不动"
    );
    for (identity, _) in user_visible_mapping_keys(&first_snapshot_version) {
        assert!(
            allocated_on_every_device(&pool, first_snapshot_version.unit(identity).slot),
            "{identity:?}：删 S2 之后仍在册未释放"
        );
    }
    assert_eq!(
        entry_of_kind(&after_the_newer, TREE_KIND_INODE).previous_snapshot_txg,
        first_snapshot_version.root.checkpoint_txg,
        "活头的前驱换成 S2 的前驱 S1.txg"
    );
    assert_eq!(
        read_the_first_file_of_a_snapshot(&pool.memory_pool(), &after_the_newer.root, older),
        Ok(SnapshotFileReading::File(file_content())),
        "S1 仍读出建它那一刻的内容"
    );
    let (_, batches_of_the_older) = destroy_the_snapshot_and_advance(&mut pool, older);
    let after_the_older = batches_of_the_older.last().expect("最后一批").clone();
    assert!(
        after_the_older.deadlist_entries.is_empty(),
        "再删 S1：落在它那一段（纪元 S2.txg ∈ [S1.txg, hi)）、birth > 0 的四条全放"
    );
    for (identity, _) in user_visible_mapping_keys(&first_snapshot_version) {
        assert!(
            !allocated_on_every_device(&pool, first_snapshot_version.unit(identity).slot),
            "{identity:?}：删 S1 之后释放"
        );
    }
}

/// 两个快照、每个之后各覆盖写一次：S1 那一段是第一个文件的四条（纪元 S1.txg），S2 那一段是第一次覆盖写的四条（纪元 S2.txg）。
/// 删 S1（P = 0，下一侧是 S2，hi = S2.txg）：意图那次把 0 传给 S2 的前驱；放掉 S1 那一段的四条（S2 不引用它们），S2 那一段一条不动；
/// S2 仍读出第一次覆盖写的内容。
#[test]
fn destroying_the_older_snapshot_releases_its_own_range_and_passes_its_predecessor_to_the_newer_one(
) {
    let mut pool = build_pool("snapshot-destroy-older-first");
    let first_snapshot_version = create_snapshot_and_advance(&mut pool);
    let older = snapshot_trees(&first_snapshot_version)[0];
    let first_overwrite = overwrite_and_advance(&mut pool, 19);
    let second_snapshot_version = create_snapshot_and_advance(&mut pool);
    let newer = *snapshot_trees(&second_snapshot_version)
        .last()
        .expect("两个快照");
    let second_overwrite = overwrite_and_advance(&mut pool, 23);
    assert_eq!(second_overwrite.deadlist_entries.len(), 8, "两段各四条");
    let (intent, batches) = destroy_the_snapshot_and_advance(&mut pool, older);
    let newer_entry_after_the_intent = intent
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT && entry.tree == newer)
        .expect("S2 那条码 14");
    assert_eq!(
        newer_entry_after_the_intent.previous_snapshot_txg,
        CheckpointTxg(0),
        "下一侧是 S2：它的前驱换成 S1 的前驱 0"
    );
    let after = batches.last().expect("最后一批").clone();
    let first_overwrite_entries: BTreeSet<Vec<u8>> = second_overwrite
        .deadlist_entries
        .iter()
        .filter(|entry| entry.epoch == second_snapshot_version.root.checkpoint_txg)
        .map(DeadlistEntry::key_bytes)
        .collect();
    assert_eq!(
        deadlist_keys(&after.deadlist_entries),
        first_overwrite_entries,
        "S1 那一段四条全放、S2 那一段四条不动"
    );
    for (identity, _) in user_visible_mapping_keys(&first_snapshot_version) {
        assert!(
            !allocated_on_every_device(&pool, first_snapshot_version.unit(identity).slot),
            "{identity:?}：第一个文件的单元只有 S1 引用，删 S1 时释放"
        );
    }
    assert_eq!(
        read_the_first_file_of_a_snapshot(&pool.memory_pool(), &after.root, newer),
        Ok(SnapshotFileReading::File(overwrite_content(19))),
        "S2 仍读出第一次覆盖写的内容"
    );
    assert_eq!(
        first_overwrite.deadlist_entries.len(),
        4,
        "第一次覆盖写杀了第一个文件的四个单元"
    );
}

/// 批大小压成 1：四条条目走成意图 + 三批 + 最后一批，每一批放一条（deadlist 少一条），最后一批放最后一条并删掉那一对条目。
#[test]
fn a_destruction_batch_capped_at_one_entry_releases_one_entry_per_publish_until_the_final_batch() {
    let mut pool = build_pool("snapshot-destroy-in-batches-of-one");
    let created = create_snapshot_and_advance(&mut pool);
    let snapshot = snapshot_trees(&created)[0];
    overwrite_and_advance(&mut pool, 29);
    publish_the_destruction_intent_and_advance(&mut pool, snapshot).expect("删快照的意图");
    let batches = walk_the_queues_and_advance(
        &mut pool,
        UnlinkBatchSize::CappedForTests(
            UnlinkBatchDataUnitsCappedForTests::new(1).expect("压成 1"),
        ),
    );
    assert_eq!(batches.len(), 4, "四条：三批各放一条，最后一批放最后一条");
    let entries_after_each_batch: Vec<usize> = batches
        .iter()
        .map(|batch| batch.deadlist_entries.len())
        .collect();
    assert_eq!(entries_after_each_batch, vec![3, 2, 1, 0], "每一批放一条");
    for batch in &batches[..3] {
        assert_eq!(
            snapshot_trees(batch),
            vec![snapshot],
            "不是最后一批：那一对条目还在"
        );
    }
    assert!(
        snapshot_trees(&batches[3]).is_empty(),
        "最后一批删掉那一对条目"
    );
}

/// 崩在意图与最后一批之间：意图那次落盘之后进程退出，下一次可写挂载续做——挂载交回的现行那一版没有快照条目、deadlist 一条不剩，
/// 快照引用过的四个单元在挂载的分配器里已释放。
#[test]
fn a_writable_mount_after_the_destruction_intent_continues_the_batches_to_the_end() {
    let mut pool = build_pool("snapshot-destroy-continued-by-the-mount");
    let created = create_snapshot_and_advance(&mut pool);
    let snapshot = snapshot_trees(&created)[0];
    overwrite_and_advance(&mut pool, 31);
    publish_the_destruction_intent_and_advance(&mut pool, snapshot).expect("删快照的意图");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    let current = mounted
        .current
        .version_of_trees()
        .expect("有树的一版")
        .clone();
    assert!(snapshot_trees(&current).is_empty(), "续做删完了那一对条目");
    assert!(current.deadlist_entries.is_empty(), "续做放完了全部条目");
    assert!(current.snapshots_being_destroyed.is_empty(), "记号没了");
    pool.allocator = mounted.allocator;
    for (identity, _) in user_visible_mapping_keys(&created) {
        assert!(
            !allocated_on_every_device(&pool, created.unit(identity).slot),
            "{identity:?}：续做之后已释放"
        );
    }
}

/// 经挂着的会话删快照（`UserChange::DestroySnapshot`）：一次调用发意图与全部批，发成的是意图那一版与最后一批；之后现行那一版没有快照、
/// deadlist 一条不剩。
#[test]
fn destroying_a_snapshot_through_the_mounted_session_publishes_the_intent_and_every_batch_in_one_call(
) {
    let mut pool = build_pool("snapshot-destroy-through-the-session");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    session
        .publish_user_change(&mut devices, UserChange::CreateSnapshot)
        .expect("经会话建快照");
    let snapshot = snapshot_trees(session.current.version_of_trees().expect("有树"))[0];
    let content = overwrite_content(37);
    session
        .publish_user_change(
            &mut devices,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            }),
        )
        .expect("快照下覆盖写");
    let published = session
        .publish_user_change(&mut devices, UserChange::DestroySnapshot { snapshot })
        .expect("经会话删快照");
    pool.devices = Some(devices);
    assert_eq!(
        published.versions_published.len(),
        2,
        "意图那一版与最后一批（四条不多于一批）"
    );
    assert_eq!(
        published.versions_published[0].snapshots_being_destroyed,
        vec![snapshot],
        "先发意图"
    );
    let PoolVersion::WithFile(current) = &session.current else {
        panic!("删快照不动文件：现行那一版带文件");
    };
    assert!(snapshot_trees(current).is_empty(), "那一对条目删掉了");
    assert!(current.deadlist_entries.is_empty(), "deadlist 一条不剩");
}

/// 删一个树表里没有的快照：在任何写之前拒成 `NoSnapshotWithThisTree`，盘上逐项不变。
#[test]
fn destroying_a_snapshot_that_is_not_in_the_tree_table_is_refused_before_any_write() {
    let mut pool = build_pool("snapshot-destroy-missing");
    let created = create_snapshot_and_advance(&mut pool);
    let missing = TreeIdentifier(created.root.tree_identifier_watermark + 7);
    let disk_before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refused = publish_the_destruction_intent_and_advance(&mut pool, missing);
    assert!(
        matches!(
            refused,
            Err(PublishError::SnapshotDestructionRefused(
                SnapshotDestructionRefusal::NoSnapshotWithThisTree { snapshot }
            )) if snapshot == missing
        ),
        "拒成 NoSnapshotWithThisTree：{refused:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        disk_before,
        "在任何写之前拒：盘上逐项不变"
    );
}

/// 有快照在销毁中（意图发了、批还没走）时建快照：在任何写之前拒成 `SnapshotBeingDestroyed`（删快照串行，岔路 2），盘上逐项不变。
#[test]
fn creating_a_snapshot_while_another_is_being_destroyed_is_refused_before_any_write() {
    let mut pool = build_pool("snapshot-create-while-destroying");
    let created = create_snapshot_and_advance(&mut pool);
    let snapshot = snapshot_trees(&created)[0];
    publish_the_destruction_intent_and_advance(&mut pool, snapshot).expect("删快照的意图");
    let disk_before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let refused = publish_create_snapshot(&mut writer, &mut pool.allocator, &previous, INSTANCE);
    assert!(
        matches!(
            refused,
            Err(PublishError::SnapshotCreationRefused(
                SnapshotCreationRefusal::SnapshotBeingDestroyed { being_destroyed }
            )) if being_destroyed == snapshot
        ),
        "拒成 SnapshotBeingDestroyed：{refused:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        disk_before,
        "在任何写之前拒：盘上逐项不变"
    );
}

/// 现行那一版带着快照时回退到第一个文件那一版：拒成 `SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses { Current }`，盘上逐项不变。
#[test]
fn a_rollback_from_a_version_carrying_a_snapshot_is_refused_before_any_write() {
    let mut pool = build_pool("snapshot-rollback-from-a-snapshot");
    let first = pool.output.clone();
    create_snapshot_and_advance(&mut pool);
    overwrite_and_advance(&mut pool, 41);
    let disk_before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refused = roll_back(&mut pool, &first);
    assert!(
        matches!(
            refused,
            Err(
                RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses {
                    version: VersionOfTheRollback::Current,
                    snapshots: 1,
                }
            )
        ),
        "现行那一版带着一个快照：拒（{refused:?}）"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        disk_before,
        "在任何写之前拒：盘上逐项不变"
    );
}

/// 快照删完之后回退到建快照那一版（它的树表带着那一对条目）：拒成 `SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses { Target }`，
/// 盘上逐项不变。
#[test]
fn a_rollback_to_a_version_carrying_a_snapshot_is_refused_before_any_write() {
    let mut pool = build_pool("snapshot-rollback-to-a-snapshot");
    let created = create_snapshot_and_advance(&mut pool);
    let snapshot = snapshot_trees(&created)[0];
    overwrite_and_advance(&mut pool, 43);
    destroy_the_snapshot_and_advance(&mut pool, snapshot);
    let disk_before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refused = roll_back(&mut pool, &created);
    assert!(
        matches!(
            refused,
            Err(
                RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses {
                    version: VersionOfTheRollback::Target,
                    snapshots: 1,
                }
            )
        ),
        "回退目标那一版带着一个快照：拒（{refused:?}）"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        disk_before,
        "在任何写之前拒：盘上逐项不变"
    );
}

/// 挂着的时候回退到 `target` 那一版的根。
fn roll_back(
    pool: &mut BuiltPool,
    target: &TransactionOutput,
) -> Result<singlefs_core::mount::RolledBack, RollbackError> {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: target.root.instance,
            checkpoint_txg: target.root.checkpoint_txg,
        },
    )
}
