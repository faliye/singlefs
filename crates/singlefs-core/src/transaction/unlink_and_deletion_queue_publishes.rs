//! 删文件的发布：unlink 意图与各步、截断那一步与删除队列的逐步走完。

use super::*;

/// unlink 的意图那次发布（D8（核心索引结构） 已定项 16 第 1 条）：`inode` 那条记录写成 nlink = 0、size 不变，`deleted_inodes` 加 key = `inode`，
/// 墓碑树开放容器追加一条整对象墓碑（死亡 txg = 这次发布的 checkpoint_txg，D18（块里携带什么信息） 已定项 24 第 2 条）；不立 logged_ops。
/// 数据单元与 extent 树照抄。一个事务（不写用户数据，不切）。改动计数设成这次发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88
/// 「最后一次改动所在发布的 checkpoint_txg」：用户 2026-10-03 定 Q-F「算改动，改实现」，意图那次与每一批都算一次改动）；
/// 写入时间取调用方给的 `write_time_seconds`（用户 2026-10-03 定 Q-F「乙 意图带时间、三格同改」：偏移 64 起 atime、mtime、ctime 三个秒字段
/// 共写这一个值，`records::InodeRecord::to_bytes`），各批照抄意图那次写进记录的值、不另取时间（[`publish_unlink_step`]），
/// 同一份崩溃镜像各自续做的两份盘因此逐字节相同（D8（核心索引结构） 已定项 16 第 7 条 ①b）。
///
/// # Errors
/// `inode` 不是树里一条活着的记录（没有、或已经 nlink = 0）⇒ `UnlinkOfAnInodeThatIsNotALiveRecord`；其余同 `publish_version`。
/// 都在任何落盘动作之前返回（墓碑树根满时长高，`crate::container_tree`）。
///
/// # Panics
/// 上一版的 `deleted_inodes` 不空：任何一版至多一条 key（第 6 条），调用方（会话、挂载续做）先把队列走空再发意图。
pub fn publish_unlink_intent<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    inode: InodeNumber,
    write_time_seconds: u64,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    assert!(
        previous.deleted_inodes.is_empty(),
        "发意图之前队列已走空：任何一版的 deleted_inodes 至多一条 key（D8 已定项 16 第 6 条）"
    );
    let record = previous
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .find(|record| record.inode == inode.0 && record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)
        .copied()
        .ok_or(PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { inode: inode.0 })?;
    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    // 改动计数取成局部量、不写 `change_count: txg.0,`：`crates/mutations.tsv` 按那串字面锚在 `publish_overwrite` 上（同 `publish_new_inodes`）。
    let change_count_of_the_intent = txg.0;
    let deletion_and_lineage = DeletionAndLineageOfThePublish {
        step: DeletionStepOfThePublish::Intent {
            record_with_nlink_zero: InodeRecord {
                inode: record.inode,
                object_birth: record.object_birth,
                nlink: NLINK_OF_AN_INODE_BEING_DELETED,
                size: record.size,
                change_count: change_count_of_the_intent,
                write_time_seconds,
                // unlink 的意图照抄截断字段（判决 m4-trunc-r2 R2）。
                truncation_field: record.truncation_field,
            },
            tombstone: TombstoneRecord::of_the_whole_object(inode, record.object_birth, txg),
        },
        container_tree_capacities: pool.container_tree_capacities(),
        lineage: LineageStepOfThePublish::NoLineageRecord,
        snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
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

/// unlink 意图之后的下一步（D8（核心索引结构） 已定项 16 第 2 条）：现行那一版这个文件还剩的数据单元多于一批就释放尾部一批
/// （只重写被截掉那一片下段叶与它的祖先，size 改成剩下单元数 × 净荷容量，nlink 仍 0），不多于一批就是最后一批
/// （释放剩下的全部单元、删 inode 记录、删 `deleted_inodes` 那条 key，同一次发布）。下一步只由现行那一版定（`unlink::next_unlink_step`），
/// 续做与一路做下来同一个计划。
///
/// # Errors
/// `inode` 不在现行那一版的 `deleted_inodes` 里、或它的记录不是 nlink = 0 ⇒ `UnlinkOfAnInodeThatIsNotALiveRecord`
/// （不是删除中的号）；其余同 `publish_version`。
pub fn publish_unlink_step<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    inode: InodeNumber,
    batch_size: UnlinkBatchSize,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let (txg, deletion_and_lineage) = next_unlink_step_after(
        previous,
        inode,
        batch_size,
        pool.container_tree_capacities(),
    )?;
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

/// `inode` 接在 `previous` 后面的下一步：这一步的 txg 与交给发布路径的删除那一半。[`publish_unlink_step`] 真发与可写挂载取号之前
/// 预演整个续做（[`rehearse_every_step_of_the_deletion_queue_walk`]）共用这一份，两处算的是同一个计划。
///
/// # Errors
/// 同 [`publish_unlink_step`] 的头两样：`UnlinkOfAnInodeThatIsNotALiveRecord`、`NextCheckpointTxgPastTheTopOfItsRange`。
fn next_unlink_step_after(
    previous: &TransactionOutput,
    inode: InodeNumber,
    batch_size: UnlinkBatchSize,
    container_tree_capacities: ContainerTreeCapacities,
) -> Result<(CheckpointTxg, DeletionAndLineageOfThePublish), PublishError> {
    let record = previous
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .find(|record| record.inode == inode.0 && record.nlink == NLINK_OF_AN_INODE_BEING_DELETED)
        .copied()
        .filter(|_| previous.deleted_inodes.contains(&inode))
        .ok_or(PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { inode: inode.0 })?;
    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    // 改动计数取成局部量，理由同 [`publish_unlink_intent`] 那一处。
    let change_count_of_the_batch = txg.0;
    // 剩下几个单元取 `inode` 自己的（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1，D8（核心索引结构）
    // 已定项 16 第 2 条对任一 inode 同一条）：第一个文件取它那一棵，别的文件取 `files_other_than_the_first` 那一项，没有的（size 0 的 inode）
    // 取 0、只有最后一批。
    let data_units_remaining = data_units_of_the_file_in_the_version(previous, inode);
    let step = match next_unlink_step(data_units_remaining, batch_size) {
        UnlinkStep::Batch {
            data_units_kept,
            size_after_the_batch_in_bytes,
        } => DeletionStepOfThePublish::Batch {
            inode,
            record_after_the_batch: InodeRecord {
                inode: record.inode,
                object_birth: record.object_birth,
                nlink: record.nlink,
                size: size_after_the_batch_in_bytes,
                // 每一批算一次改动（用户 2026-10-03 定 Q-F，同意图那次）；写入时间照抄上一版这条记录的值——意图那次写进去的那一个
                // （用户 2026-10-03 定 Q-F 乙「各批照抄」）：批不带时间参数，挂载续做里的几批没有调用方给时间。
                change_count: change_count_of_the_batch,
                write_time_seconds: record.write_time_seconds,
                // unlink 的各批照抄截断字段（判决 m4-trunc-r2 R2）。
                truncation_field: record.truncation_field,
            },
            data_units_kept,
        },
        UnlinkStep::FinalBatch { .. } => DeletionStepOfThePublish::FinalBatch { inode },
    };
    Ok((
        txg,
        DeletionAndLineageOfThePublish {
            step,
            container_tree_capacities,
            lineage: LineageStepOfThePublish::NoLineageRecord,
            snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
            logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
        },
    ))
}

/// `version` 里 `inode` 那个文件存在几个数据单元：第一个文件取它那一棵，别的文件取 `files_other_than_the_first` 那一项（标签 0 的是 0），
/// 上段里没有条目的取 0（unlink 的批、截断三步与变长共用这一份，判决 m3-multi-file-unlink-r1 第三节 M1、m4-multi-inode-size-r1 第二节 S3）。
pub(super) fn data_units_of_the_file_in_the_version(
    version: &TransactionOutput,
    inode: InodeNumber,
) -> u64 {
    let data_units = if inode.0 == FIRST_INODE_NUMBER {
        version.data_pointers.len()
    } else {
        version
            .file_other_than_the_first(inode)
            .map_or(0, |file| file.data_pointers.len())
    };
    u64::try_from(data_units).expect("单元数")
}

/// 截断意图之后的下一步（判决 m4-trunc-r1 K2、K11）：现行那一版的截断条目点名一个 inode、目标 N；size 之外还挂着的单元多于一批就从尾部
/// 放一批（不碰 inode 记录），不多于一批就是最后一批（放完剩下的、可以 0 个，同一次发布删截断条目）。下一步只由现行那一版定
/// （`truncate::next_truncation_step`），续做与一路做下来同一个计划。
///
/// # Errors
/// 现行那一版没有截断条目 ⇒ `TruncationRefused(NoTruncationIntentToContinue)`；条目点名的号在现行那一版里不是一条活记录（没有、nlink = 0、
/// 对象出生代或 size 对不上）⇒
/// `TruncationRefused(TruncationIntentDoesNotNameTheLiveRecord)`；其余同 `publish_version`。都在任何落盘动作之前。
pub fn publish_truncation_step<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    batch_size: UnlinkBatchSize,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let (issued_txg, intent) = live_truncation_intent_of_the_entries(&previous.logged_operations)
        .ok_or(PublishError::TruncationRefused(
        TruncationRefusal::NoTruncationIntentToContinue,
    ))?;
    // 按条目点名的号认领（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3、S4：续做按号认领是 K1 能留的前提）。
    let names_the_live_record = previous
        .inode_record_of(intent.inode)
        .is_some_and(|record| {
            record.object_birth == intent.object_birth
                && record.nlink != NLINK_OF_AN_INODE_BEING_DELETED
                && record.size == intent.target_size_in_bytes
        });
    if !names_the_live_record {
        return Err(PublishError::TruncationRefused(
            TruncationRefusal::TruncationIntentDoesNotNameTheLiveRecord { intent },
        ));
    }
    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    let data_units_existing = data_units_of_the_file_in_the_version(previous, intent.inode);
    let inode = intent.inode;
    let (step, logged_operations) =
        match next_truncation_step(data_units_existing, intent.target_size_in_bytes, batch_size) {
            TruncationStep::Batch { data_units_kept } => (
                DeletionStepOfThePublish::TruncationBatch {
                    inode,
                    data_units_kept,
                },
                LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
            ),
            TruncationStep::FinalBatch { data_units_kept } => (
                DeletionStepOfThePublish::TruncationFinalBatch {
                    inode,
                    data_units_kept,
                },
                LoggedOperationsStepOfThePublish::DeleteTheTruncationIntent { issued_txg },
            ),
        };
    let deletion_and_lineage = DeletionAndLineageOfThePublish {
        step,
        container_tree_capacities: pool.container_tree_capacities(),
        lineage: LineageStepOfThePublish::NoLineageRecord,
        snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
        logged_operations,
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

/// 把删除队列从 `current` 走到空的每一步都只算不写（可写挂载在取号之前预演整个挂载续做，岔路单
/// `research/prompts/m3-b44-code-r1-forks.md` 那一行）：每一步与真发（[`walk_the_deletion_queue_until_it_is_empty`]）同一张计划
/// （[`next_unlink_step_after`]、[`deletion_step_plan`]，树号取上一步那一版的），走发布路径落盘之前的那一段（[`prepare_the_version_publish`]，
/// 不读盘核被换下的、照常释放），接在上一步预演出来的那一版上。落点取在 `allocator_copy` 上：调用方交的是分配器的拷贝，真分配器不动，
/// 一个写都不发。队列空（常态）时什么都不算，交 `Ok`。
///
/// 迭代上界同 [`walk_the_deletion_queue_until_it_is_empty`]：每一步要么让队首那个文件少至少一个单元、要么删掉队首那个号。
/// 跨轮携带的是预演到的那一版与 `allocator_copy`。
///
/// # Errors
/// 走到的第一步报的错，原样（`NextContainerNumberPastTheTopOfItsRange` 在内）：那一步之后的各步接不上一版，不再预演。
pub(crate) fn rehearse_every_step_of_the_deletion_queue_walk<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator_copy: &mut PoolAllocator,
    current: &TransactionOutput,
    batch_size: UnlinkBatchSize,
    container_tree_capacities: ContainerTreeCapacities,
    instance: InstanceGeneration,
) -> Result<(), PublishError> {
    let mut version_rehearsed = current.clone();
    loop {
        let Some(inode) = version_rehearsed.deleted_inodes.first().copied() else {
            return Ok(());
        };
        let (txg, deletion_and_lineage) = next_unlink_step_after(
            &version_rehearsed,
            inode,
            batch_size,
            container_tree_capacities,
        )?;
        let (_, version_after_the_rehearsed_step) = prepare_the_version_publish(
            pool,
            allocator_copy,
            &deletion_step_plan(&version_rehearsed, txg, instance),
            Some(&version_rehearsed),
            version_rehearsed.tree_identifiers,
            ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
            UnmountMarker::NotWrittenByTheUnmountSequence,
            &deletion_and_lineage,
        )?;
        version_rehearsed = version_after_the_rehearsed_step;
    }
}

/// 把现行那一版的 `deleted_inodes` 队列走空（D8（核心索引结构） 已定项 16 第 6 条：会话里每一次用户改动先走空、unlink 一次调用走完全部批，
/// 与挂载续做共用这一个走空）：队首的号逐步 [`publish_unlink_step`] 到它的最后一批，每发成一步 `current` 就换成那一版。
/// 交回发了几次（队列本来就空时 0、一个写都不发）。发成的每一步要交回的走 [`walk_the_deletion_queue_until_it_is_empty`]。
///
/// # Errors
/// 同 [`publish_unlink_step`]；失败时已经发成的那几步留在盘上、`current` 停在最后发成的那一版上，交回那一步的错。
pub fn publish_unlink_steps_until_the_queue_is_empty<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    current: &mut Arc<TransactionOutput>,
    batch_size: UnlinkBatchSize,
    instance: InstanceGeneration,
) -> Result<usize, PublishError> {
    let walk =
        walk_the_deletion_queue_until_it_is_empty(pool, allocator, current, batch_size, instance);
    walk.stopped_by.map(|()| walk.steps_published.len())
}

/// 走空一趟的结局（[`walk_the_deletion_queue_until_it_is_empty`]）：发成的每一步（按先后，最后一个就是 `current`，两处指着同一份），
/// 与这一趟停在哪：走空了是 `Ok`，一步没发成是那一步的错（已经发成的那几步留在盘上）。
#[derive(Debug)]
pub struct DeletionQueueWalk {
    pub steps_published: Vec<Arc<TransactionOutput>>,
    pub stopped_by: Result<(), PublishError>,
}

/// 选删除队列的下一步、发它：次序只在这里：`deleted_inodes` 先走空，再走销毁中的快照（树号升序），再走截断；整理是提示类，不在这里走。
/// 下一步只由 `current` 这一版定：走空一趟（[`walk_the_deletion_queue_until_it_is_empty`]）与放锁的后台入口一次只走一步都调它，步与步之间插进别的发布，走到的还是同一串步。
/// 队列空着交回 `None`，一个写都不发。
pub fn publish_the_next_step_of_the_deletion_queue<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    current: &TransactionOutput,
    batch_size: UnlinkBatchSize,
    instance: InstanceGeneration,
) -> Option<Result<TransactionOutput, PublishError>> {
    match (
        current.deleted_inodes.first().copied(),
        current.snapshots_being_destroyed.first().copied(),
        live_truncation_intent_of_the_entries(&current.logged_operations),
    ) {
        (Some(inode), _, _) => Some(publish_unlink_step(
            pool, allocator, current, inode, batch_size, instance,
        )),
        (None, Some(snapshot), _) => Some(publish_snapshot_destruction_step(
            pool, allocator, current, snapshot, batch_size, instance,
        )),
        (None, None, Some(_truncation)) => Some(publish_truncation_step(
            pool, allocator, current, batch_size, instance,
        )),
        (None, None, None) => None,
    }
}

/// 同 [`publish_unlink_steps_until_the_queue_is_empty`]，连同发成的每一步交回（会话与挂载续做要把它们交给调用方，
/// 模型对拍逐次比它们）。每发成一步，`current` 与交回的那一步指着同一份（不拷一整版）。
pub fn walk_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    current: &mut Arc<TransactionOutput>,
    batch_size: UnlinkBatchSize,
    instance: InstanceGeneration,
) -> DeletionQueueWalk {
    let mut steps_published = Vec::new();
    // 迭代上界：每一步要么让队首那个文件少至少一个单元、要么删掉队首那个号（最后一批），要么放掉销毁中那个快照至少一条 deadlist 条目、
    // 要么删掉它那一对条目（最后一批），要么让截断的文件少至少一个单元、要么删掉截断条目（最后一批）：队列、单元数与条目数都有限。
    loop {
        let Some(next_step) = publish_the_next_step_of_the_deletion_queue(
            pool, allocator, current, batch_size, instance,
        ) else {
            return DeletionQueueWalk {
                steps_published,
                stopped_by: Ok(()),
            };
        };
        match next_step {
            Ok(next) => {
                *current = Arc::new(next);
                steps_published.push(Arc::clone(current));
            }
            Err(refusal) => {
                return DeletionQueueWalk {
                    steps_published,
                    stopped_by: Err(refusal),
                }
            }
        }
    }
}

/// 删除的三种发布、改 size 与按偏移部分写共用的计划：不带整份文件内容、不建 inode、实例表照抄、一个树 ID 都不发，接在 `previous` 后面。
/// `payloads_of_the_rewritten_data_units` 是这一步按读回的旧单元（或零）重写的那一段单元的新净荷（入口在任何写之前读好、改好），别的步给空的。
pub(super) fn publish_deletion_step<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    txg: CheckpointTxg,
    deletion_and_lineage: &DeletionAndLineageOfThePublish,
    payloads_of_the_rewritten_data_units: &[RewrittenDataUnitPayload],
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    publish_version_of_trees_relocating_units_after_a_check(
        pool,
        allocator,
        deletion_step_plan(previous, txg, instance),
        Some(previous),
        previous.tree_identifiers,
        UnmountMarker::NotWrittenByTheUnmountSequence,
        deletion_and_lineage,
        &[],
        &[],
        payloads_of_the_rewritten_data_units,
        |_version_after_this_publish| Ok::<(), PublishError>(()),
    )
}

/// 删除的三种发布接在 `previous` 后面那一次的计划（[`publish_deletion_step`] 真发与挂载取号之前预演整个续做共用）。
fn deletion_step_plan(
    previous: &TransactionOutput,
    txg: CheckpointTxg,
    instance: InstanceGeneration,
) -> PublishPlan<'static> {
    let transaction_of_this_deletion = previous.highest_transaction_number_in_this_instance + 1;
    PublishPlan {
        txg,
        counter: previous.record.counter + 1,
        transaction: transaction_of_this_deletion,
        highest_transaction_number_before_this_publish: previous
            .highest_transaction_number_in_this_instance,
        instance,
        back_chain: back_chain_of(&previous.record_bytes),
        file: None,
        new_inode_records: &[],
        instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
        tree_birth_txg: previous.tree_birth_txg(),
        tree_identifier_watermark: previous.root.tree_identifier_watermark,
        rollback_floor: previous.root.rollback_floor,
    }
}
