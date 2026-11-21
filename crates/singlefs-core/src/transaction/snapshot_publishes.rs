//! 建快照与销毁快照的发布。

use super::*;

/// 建一个只读快照那次发布（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1、S2）：**独立发布**，不与别的用户改动合在一个事务里，
/// 接在 `previous` 后面；文件内容与 inode 树都不动（`file` 空、不建 inode），只重写固定点角色与树表。S.txg := 这次发布的 checkpoint_txg。
/// 树号从上一版的树 ID 水位起连发两个（码 14、码 15，水位 + 2，D8（核心索引结构） 已定项 8 ②），树表在续上的快照条目之后追加这一对，
/// 活头 inode 树条目的 `previous_snapshot_txg` 换成 S.txg（字段取值见 [`SnapshotStepOfThePublish::CreateSnapshot`]）。deadlist 一条不动。
/// 一个事务（不写用户数据，不切；事务号接着实例内计数，同建 inode）。
///
/// 被快照引用的块这一批不改释放：之后的覆盖写照旧经映射释放被换下的角色（释放分流、deadlist、映射条目留存与删快照归下一批）。
///
/// # Errors
/// 上一版已有 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 个快照 ⇒ `SnapshotCreationRefused(MoreSnapshotsThanTheFirstVersionAllows)`；
/// 水位加二越过 `u64::MAX` ⇒ `SnapshotCreationRefused(TreeIdentifierWatermarkLeavesNoRoomForTheSnapshotTrees)`；两样都在任何读写之前返回，
/// 盘上逐字节不变。其余同 [`publish_version`]。
pub fn publish_create_snapshot<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let snapshots_before_this_publish = previous
        .tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .count();
    if let Some(being_destroyed) = previous.snapshots_being_destroyed.first().copied() {
        return Err(PublishError::SnapshotCreationRefused(
            SnapshotCreationRefusal::SnapshotBeingDestroyed { being_destroyed },
        ));
    }
    if snapshots_before_this_publish >= SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {
        return Err(PublishError::SnapshotCreationRefused(
            SnapshotCreationRefusal::MoreSnapshotsThanTheFirstVersionAllows {
                snapshots: snapshots_before_this_publish,
            },
        ));
    }
    let watermark_before_this_publish = previous.root.tree_identifier_watermark;
    let watermark_after_this_publish = watermark_before_this_publish.checked_add(2).ok_or(
        PublishError::SnapshotCreationRefused(
            SnapshotCreationRefusal::TreeIdentifierWatermarkLeavesNoRoomForTheSnapshotTrees {
                tree_identifier_watermark: watermark_before_this_publish,
            },
        ),
    )?;
    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    let transaction_of_the_snapshot = previous.highest_transaction_number_in_this_instance + 1;
    let snapshot = SnapshotStepOfThePublish::CreateSnapshot {
        snapshot_inode_root_tree: TreeIdentifier(watermark_before_this_publish),
        snapshot_extent_root_tree: TreeIdentifier(watermark_before_this_publish + 1),
    };
    publish_version_of_trees(
        pool,
        allocator,
        PublishPlan {
            txg,
            counter: previous.record.counter + 1,
            transaction: transaction_of_the_snapshot,
            highest_transaction_number_before_this_publish: previous
                .highest_transaction_number_in_this_instance,
            instance,
            back_chain: back_chain_of(&previous.record_bytes),
            // 文件内容不动：数据单元、extent 树与 inode 树照抄上一版。
            file: None,
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: watermark_after_this_publish,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
        previous.tree_identifiers,
        UnmountMarker::NotWrittenByTheUnmountSequence,
        &DeletionAndLineageOfThePublish {
            step: DeletionStepOfThePublish::NotADeletion,
            container_tree_capacities: pool.container_tree_capacities(),
            lineage: LineageStepOfThePublish::NoLineageRecord,
            snapshot,
            logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
        },
    )
}

/// 删快照 `snapshot` 的意图那次发布（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4 与第七节岔路 2、4）：
/// 它那条码 14 条目的 flags 位 0 写成「销毁中」、预留字节不写；它的前驱 P 传给那一刻的下一侧（[`SnapshotStepOfThePublish::DestructionIntent`]）；
/// 一条 deadlist 条目都不动、一个单元都不释放。照 unlink 的意图那次同一张计划（不写文件内容、不建 inode、实例表照抄、不发树号、一个事务），
/// 同等准入（式子那一判不判、落点那一判照判）。从这一刻起被删的快照不再钉住块（D5（快照 / 空间记账机制） 已定项 20 的例外）；
/// 之后的批由 [`publish_snapshot_destruction_step`] 走，会话与挂载续做的走空（[`walk_the_deletion_queue_until_it_is_empty`]）接着走它们。
///
/// # Errors
/// 已经有快照在销毁中（删快照串行，判决第七节岔路 2）⇒ `SnapshotDestructionRefused(SnapshotAlreadyBeingDestroyed / AnotherSnapshotIsBeingDestroyed)`；
/// 树表里没有这个快照 ⇒ `SnapshotDestructionRefused(NoSnapshotWithThisTree)`；都在任何读写之前。其余同 `publish_version`。
pub fn publish_snapshot_destruction_intent<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    snapshot: TreeIdentifier,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    if let Some(being_destroyed) = previous.snapshots_being_destroyed.first().copied() {
        return Err(PublishError::SnapshotDestructionRefused(
            if being_destroyed == snapshot {
                SnapshotDestructionRefusal::SnapshotAlreadyBeingDestroyed { snapshot }
            } else {
                SnapshotDestructionRefusal::AnotherSnapshotIsBeingDestroyed { being_destroyed }
            },
        ));
    }
    if snapshot_inode_root_entry(&previous.tree_table_entries, snapshot).is_none() {
        return Err(PublishError::SnapshotDestructionRefused(
            SnapshotDestructionRefusal::NoSnapshotWithThisTree { snapshot },
        ));
    }
    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    let deletion_and_lineage = DeletionAndLineageOfThePublish {
        step: DeletionStepOfThePublish::NotADeletion,
        container_tree_capacities: pool.container_tree_capacities(),
        lineage: LineageStepOfThePublish::NoLineageRecord,
        snapshot: SnapshotStepOfThePublish::DestructionIntent { snapshot },
        logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
    };
    publish_deletion_step(
        pool,
        allocator,
        previous,
        txg,
        &deletion_and_lineage,
        &[],
        instance,
    )
}

/// 删快照意图之后的下一批（判决 S4）：这一版 `snapshot` 那一段里能放的条目（[`deadlist_entries_the_destruction_of_a_snapshot_releases`]，
/// 下一侧是活头时 hi 取这一批自己的 txg）多于一批就放按 key 升序的前一批（[`crate::unlink::deadlist_entries_per_snapshot_destruction_batch`]），
/// 不多于一批就是最后一批：放掉剩下的全部、同一次发布删掉那一对码 14、15 条目。每条放之前按它留存的映射条目读盘核、逐盘核落点，
/// 删 deadlist 条目、删留存的映射条目、落点进 defer 队列。下一批只由现行那一版定，续做与一路做下来同一个计划；续做点不存（条目被放掉就是进度）。
///
/// # Errors
/// `snapshot` 不带「销毁中」⇒ `SnapshotDestructionRefused(SnapshotNotBeingDestroyed)`，在任何读写之前；条目放不了 ⇒ `DeadlistEntryNotReleasable`，
/// 在任何落盘动作之前；其余同 `publish_version`。
pub fn publish_snapshot_destruction_step<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    snapshot: TreeIdentifier,
    batch_size: UnlinkBatchSize,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    if !previous.snapshots_being_destroyed.contains(&snapshot) {
        return Err(PublishError::SnapshotDestructionRefused(
            SnapshotDestructionRefusal::SnapshotNotBeingDestroyed { snapshot },
        ));
    }
    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    let releasable =
        deadlist_entries_the_destruction_of_a_snapshot_releases(previous, snapshot, txg).len();
    let entries_per_batch =
        crate::unlink::deadlist_entries_per_snapshot_destruction_batch(batch_size);
    let step = if releasable <= entries_per_batch {
        SnapshotStepOfThePublish::DestructionFinalBatch { snapshot }
    } else {
        SnapshotStepOfThePublish::DestructionBatch {
            snapshot,
            entries_released: entries_per_batch,
        }
    };
    let deletion_and_lineage = DeletionAndLineageOfThePublish {
        step: DeletionStepOfThePublish::NotADeletion,
        container_tree_capacities: pool.container_tree_capacities(),
        lineage: LineageStepOfThePublish::NoLineageRecord,
        snapshot: step,
        logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
    };
    publish_deletion_step(
        pool,
        allocator,
        previous,
        txg,
        &deletion_and_lineage,
        &[],
        instance,
    )
}
