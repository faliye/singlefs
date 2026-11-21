//! 发一版：发布前不读不写的判定、释放核对隔离的副本、搬迁单元的发布与搬迁落点的分配。

use super::*;

/// 写入口拒删除中的 inode 与没有记录的 inode（D8（核心索引结构） 已定项 16 第 4 条）：覆盖写与顺序写现造第一个文件那条记录
/// （`inode_record_of_file_version` 照 nlink = 1 写），上一版那条记录 nlink = 0、或已经删完不在树里时不许走到那一步；
/// 活着的那条交回（覆盖写要接着它的对象出生代）。
///
/// # Errors
/// 上一版第一个文件那条记录 nlink = 0 ⇒ `WriteToAnInodeBeingDeleted`；已经删完 ⇒ `WriteToAnInodeThatIsNotInTheTree`。
pub(super) fn live_record_of_the_first_file(
    previous: &TransactionOutput,
) -> Result<InodeRecord, PublishError> {
    let Some(record) = previous.inode_record else {
        return Err(PublishError::WriteToAnInodeThatIsNotInTheTree {
            inode: FIRST_INODE_NUMBER,
        });
    };
    if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
        return Err(PublishError::WriteToAnInodeBeingDeleted {
            inode: record.inode,
        });
    }
    Ok(record)
}

/// `publish_first_file` 与 `publish_overwrite` 的契约是一个数据单元（多个数据单元的内容走 `publish_sequential_write`）：
/// 内容装不进一个数据单元是调用方能恢复的失败，不是不变量被破坏——报错，在动分配器、动盘之前返回，不走到 `build_data_unit` 的断言。
///
/// # Errors
/// `ContentExceedsDataUnit`。
fn refuse_a_file_that_does_not_fit_one_data_unit(
    plan: &PublishPlan<'_>,
) -> Result<(), PublishError> {
    if let Some(file) = &plan.file {
        let data_unit_capacity = data_unit_payload_capacity();
        if file.content.len() > data_unit_capacity {
            return Err(PublishError::ContentExceedsDataUnit {
                bytes: file.content.len(),
                capacity: data_unit_capacity,
            });
        }
    }
    Ok(())
}

/// `publish_overwrite` 那一路：先按一个数据单元的契约判内容，再走 `publish_version`。
pub(super) fn publish_version_holding_one_data_unit<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
) -> Result<TransactionOutput, PublishError> {
    refuse_a_file_that_does_not_fit_one_data_unit(&plan)?;
    publish_version(pool, allocator, plan, previous)
}

/// `publish_first_file` 那一路：先按一个数据单元的契约判内容，再走 `publish_version_of_trees`（十二棵树的号由它给）。
pub(super) fn publish_version_of_trees_holding_one_data_unit<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    trees: FileVersionTreeIdentifiers,
) -> Result<TransactionOutput, PublishError> {
    refuse_a_file_that_does_not_fit_one_data_unit(&plan)?;
    publish_version_of_trees(
        pool,
        allocator,
        plan,
        previous,
        trees,
        UnmountMarker::NotWrittenByTheUnmountSequence,
        &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
    )
}

/// 发布一版：先做空间准入（这次的单元落不落得下，再是 D28（挂载期承诺量） 已定项 1 的式子那一判，[`prepare_the_version_publish`]），
/// 再释放上一版被换下的角色的落点、分配、装单元、
/// 按 D16（发布语义） 已定项 7 的持久顺序落盘。准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立，
/// 释放与分配都不算数（第二轮攻方腿：落点被拒（当时叫 `NoSpaceFor`）在释放之后、分配到一半返回，留下半新的池，拿同一个上一版重试撞断言）；
/// 落盘那几步里失败的，这次已记的写进写入口的失败账（增补 2 第 20b 行，`PoolWriter::writes_of_failed_publishes`）。
///
/// 这一版那十二棵树的号：接在上一版之后照抄上一版的（树 ID 只在第一个文件版本那次发）。没有上一版的只有第一个文件版本，
/// 这里按 mkfs 那条流的水位 65536 发（D8（核心索引结构） 已定项 11 登记的那几个常量）；从别的水位发号的第一个文件版本
/// （回退到树表 0 条的一版之后）走 `publish_first_file`，它按要建在上面的那一版的水位发。
///
/// # Errors
/// 冻结着一次没重发的发布 ⇒ `PublishFrozenAfterAWriteFailureIsNotResentYet`；`InodeTreeWriteRefused`、`DeletedInodesQueueRefused`、
/// `SpaceAdmissionRefused`、`MultiLevelCodeTwoTreeRefused`、`AllocationRecordTreeRewriteSetDidNotSettle`、
/// `JournalRecordsOfThePublishExceedTheLimit`、释放判定路径与释放之前读盘核的几种错（`Release` 与 `MappingEntry` 起头的那几个成员）、
/// `PlacementRefused`、块设备错。内容装不进一个数据单元（`ContentExceedsDataUnit`）不由它报：按一个数据单元写的
/// [`publish_first_file`]、[`publish_overwrite`] 在走发布本体之前自己判。
///
/// # Panics
/// 没有上一版、而计划写的新水位不是 mkfs 那条流第一次发布之后的 65549：调用方把一个水位早已推高的池当成了 mkfs 那条流，
/// 按 65536 发号会重发已经发过的号（D8（核心索引结构） 已定项 8 ②）。
pub fn publish_version<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
) -> Result<TransactionOutput, PublishError> {
    publish_version_with_unmount_marker(
        pool,
        allocator,
        plan,
        previous,
        UnmountMarker::NotWrittenByTheUnmountSequence,
    )
}

/// 同 [`publish_version`]，这一版根记录 flags 的卸载记号由调用方给（D22（单元原子性怎么合成） 已定项 7）：
/// 只有正常卸载那一串（`mount::unmount`）给 [`UnmountMarker::WrittenByTheUnmountSequence`]，别的调用方一律走 [`publish_version`]。
///
/// # Errors
/// 同 [`publish_version`]。
///
/// # Panics
/// 同 [`publish_version`]。
pub fn publish_version_with_unmount_marker<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    unmount_marker: UnmountMarker,
) -> Result<TransactionOutput, PublishError> {
    let trees = match previous {
        Some(previous_version) => previous_version.tree_identifiers,
        None => {
            assert_eq!(
                plan.tree_identifier_watermark, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
                "没有上一版的发布在这里按 mkfs 那条流的水位 65536 发号：计划写的新水位必须就是那条流的 65549，别的水位走 publish_first_file"
            );
            FileVersionTreeIdentifiers::issued_from_watermark(TREE_IDENTIFIER_WATERMARK_AT_MKFS)
                .expect("mkfs 水位 65536 加十二个号装得下")
                .0
        }
    };
    publish_version_of_trees(
        pool,
        allocator,
        plan,
        previous,
        trees,
        unmount_marker,
        &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
    )
}

/// 管理员回退那次发布（`mount::roll_back_by_a_forward_publish`）：同 [`publish_version`]，另往谱系树开放容器追加 `lineage_record`
/// （D18（块里携带什么信息） 已定项 24 第 5 条、D23（journal 的角色与格式） 已定项 14「回退那次发布追加一条谱系记录」）。
/// 谱系容器与谱系树根归固定点，这次发布照旧没有普通分配、不判式子（D28（挂载期承诺量） 已定项 1 接线）。
/// 这次的 inode 号水位写 `inode_number_watermark_after_the_rollback`（回退那一判算好的 max），记账里的台阶行照抄 `previous`（它的记账行原样是 cur 的）。
/// `truncation_entry_of_the_rollback_target` 是回退目标 R_old 那一版 logged_ops 里的截断条目（判决 m4-trunc-r1 K1：截断条目取 R_old 的，
/// 整理条目随现行那一版）。
/// `mapping_keys_the_rollback_moves_back_to_the_target_locations` 是条目要换回目标那一版位置的映射 key（`previous` 的指针已是目标那一版的位置，
/// 映射树的节点却是现行那一版的）：装着它们的叶连同到根的路这次重写，同整理的一批换位置的 key（判决
/// `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F1）。
///
/// # Errors
/// 同 [`publish_version`]（谱系树根满时长高，`crate::container_tree`）。
///
/// # Panics
/// `plan` 里有新 inode 记录：回退那次发布不建 inode。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：写入口、分配器、计划、上一版、谱系记录、水位、换回目标位置的映射 key、目标那一版的截断条目"
)]
pub fn publish_version_appending_a_lineage_record<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: &TransactionOutput,
    lineage_record: LineageRecord,
    inode_number_watermark_after_the_rollback: u64,
    mapping_keys_the_rollback_moves_back_to_the_target_locations: &[Vec<u8>],
    truncation_entry_of_the_rollback_target: Option<LoggedOperationEntry>,
) -> Result<TransactionOutput, PublishError> {
    publish_version_of_trees_relocating_units_after_a_check(
        pool,
        allocator,
        plan,
        Some(previous),
        previous.tree_identifiers,
        UnmountMarker::NotWrittenByTheUnmountSequence,
        &DeletionAndLineageOfThePublish {
            step: DeletionStepOfThePublish::NotADeletion,
            container_tree_capacities: pool.container_tree_capacities(),
            lineage: LineageStepOfThePublish::AdministratorRollback {
                record: lineage_record,
                inode_number_watermark_after_the_rollback,
            },
            snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
            // logged_ops 按种类分来源（判决 m4-trunc-r1 K1）：整理条目照抄 `previous`（现行那一版的），截断条目换成目标那一版的。
            // 两样合起来与现行那一版不同时这次重写 logged_ops 根（普通分配）。
            logged_operations:
                LoggedOperationsStepOfThePublish::TakeTheTruncationEntryOfTheRollbackTarget(
                    truncation_entry_of_the_rollback_target,
                ),
        },
        &[],
        mapping_keys_the_rollback_moves_back_to_the_target_locations,
        &[],
        |_version_after_this_publish| Ok::<(), PublishError>(()),
    )
}

/// `publish_version` 的本体，这一版那十二棵树的号由调用方给：`publish_first_file` 给它这次从水位发出来的，
/// `publish_version` 给上一版的（或 mkfs 那条流的）；根记录的卸载记号同样由调用方给。
pub(super) fn publish_version_of_trees<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    trees: FileVersionTreeIdentifiers,
    unmount_marker: UnmountMarker,
    deletion_and_lineage: &DeletionAndLineageOfThePublish,
) -> Result<TransactionOutput, PublishError> {
    publish_version_of_trees_relocating_units(
        pool,
        allocator,
        plan,
        previous,
        trees,
        unmount_marker,
        deletion_and_lineage,
        &[],
    )
}

/// 同 [`publish_version_of_trees_relocating_units`]，算完账、落盘之前先交 `check_before_persisting` 核这一版（整理删意图那一支核 R 按记录全空，
/// [`publish_compaction_step`]）：核不过就把分配器换回进来时那一份、一个写都不发，原样交回核出来的拒绝。
///
/// # Errors
/// `check_before_persisting` 的拒绝；其余同 [`publish_version_of_trees_relocating_units`]（经 `From<PublishError>` 换成 `Refusal`）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：写入口、分配器、计划、上一版、树号、卸载记号、删除这一步、这次要搬的单元、不搬而换了位置的映射 key、\
              读回旧单元改好的新净荷、落盘之前的核"
)]
pub(super) fn publish_version_of_trees_relocating_units_after_a_check<
    Device: BlockDevice,
    Refusal,
    Check,
>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    trees: FileVersionTreeIdentifiers,
    unmount_marker: UnmountMarker,
    deletion_and_lineage: &DeletionAndLineageOfThePublish,
    relocations: &[RelocationOfThePublish],
    mapping_keys_whose_entries_change_without_a_relocation: &[Vec<u8>],
    payloads_of_the_rewritten_data_units: &[RewrittenDataUnitPayload],
    check_before_persisting: Check,
) -> Result<TransactionOutput, Refusal>
where
    Refusal: From<PublishError>,
    Check: FnOnce(&TransactionOutput) -> Result<(), Refusal>,
{
    // 第一道：冻结着一次没重发的发布就在任何读写之前拒（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）。
    refuse_while_a_publish_is_frozen(allocator)?;
    // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
    let allocator_before_this_publish = allocator.clone();
    let built = prepare_the_version_publish_relocating_units(
        pool,
        allocator,
        &plan,
        previous,
        trees,
        ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt,
        unmount_marker,
        deletion_and_lineage,
        relocations,
        mapping_keys_whose_entries_change_without_a_relocation,
        payloads_of_the_rewritten_data_units,
    );
    let (writes, output) = match built {
        Ok(built) => built,
        Err(refusal) => {
            *allocator = allocator_before_this_publish;
            return Err(Refusal::from(refusal));
        }
    };
    if let Err(refused) = check_before_persisting(&output) {
        *allocator = allocator_before_this_publish;
        return Err(refused);
    }
    // 落盘那几步里失败：分配器换回去、这次发布冻结在它上面等原样重发（`FrozenPublish`）。
    persist_the_publish_or_freeze_it(
        pool,
        allocator,
        allocator_before_this_publish,
        writes,
        PoolVersion::of_a_version_of_trees(output),
    )
    .map(|version| {
        version
            .into_version_of_trees()
            .expect("交给落盘的是有树的一版，交回的就是它（`PoolVersion::with_writes` 不换成员）")
    })
    .map_err(Refusal::from)
}

/// 同 [`publish_version_of_trees`]，另带这次要搬的单元（[`prepare_the_version_publish_relocating_units`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：写入口、分配器、计划、上一版、树号、卸载记号、删除这一步、这次要搬的单元"
)]
fn publish_version_of_trees_relocating_units<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    trees: FileVersionTreeIdentifiers,
    unmount_marker: UnmountMarker,
    deletion_and_lineage: &DeletionAndLineageOfThePublish,
    relocations: &[RelocationOfThePublish],
) -> Result<TransactionOutput, PublishError> {
    publish_version_of_trees_relocating_units_after_a_check(
        pool,
        allocator,
        plan,
        previous,
        trees,
        unmount_marker,
        deletion_and_lineage,
        relocations,
        &[],
        &[],
        |_version_after_this_publish| Ok(()),
    )
}

/// 释放之前读不读盘核被换下的每一份的校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseChecksumCheck {
    /// 发布路径：读盘核，核出对不上（读不出也算）的单元在每块盘上的分配记录留在已分配。
    ReadEveryReplacedCopyBeforeReleasingIt,
    /// 可写挂载取号之前的预演（`mount`）：不读盘、照常释放——预演只要这一串取落点的次序与结局；
    /// 两边只在「有一份核出对不上、那一槽又在这一串里被回收」时分叉，那一格见 `mount::establish_instance` 的注释。
    SkippedByTheDryRunBeforeAcquisition,
}

/// 带文件的一版的一次发布，落盘之前的全部：算这次之后每棵树的样子（分配记录树重写哪几个节点走到固定点，
/// [`settle_the_allocation_record_tree`]）、释放判定、释放之前读盘核（`release_checksum_check` 说核不核）、释放、取落点、装单元，
/// 交回装好、还没落盘的字节与这一版（写账是空的）。只动 `allocator`：失败时它停在中途，由调用方换回去——发布路径换回进来时那一份，
/// 可写挂载取号之前的预演拿的本来就是拷贝。两处走的是同一段代码，预演取到的落点因此就是真发时取到的（`mount::establish_instance` 断言）。
///
/// 根记录的卸载记号照 `unmount_marker` 写（D22（单元原子性怎么合成） 已定项 7）：预演与真发给同一个。
///
/// # Errors
/// `InodeTreeWriteRefused`、`SpaceAdmissionRefused`（这次的单元落不下，排在走固定点之前；或式子那一判不够）、
/// `MultiLevelCodeTwoTreeRefused`、释放判定路径的几种错、`PlacementRefused`。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：写入口、分配器、计划、上一版、树号、读不读盘核、卸载记号、删除这一步"
)]
pub fn prepare_the_version_publish<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: &PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    trees: FileVersionTreeIdentifiers,
    release_checksum_check: ReleaseChecksumCheck,
    unmount_marker: UnmountMarker,
    deletion_and_lineage: &DeletionAndLineageOfThePublish,
) -> Result<(PublishWrites, TransactionOutput), PublishError> {
    prepare_the_version_publish_relocating_units(
        pool,
        allocator,
        plan,
        previous,
        trees,
        release_checksum_check,
        unmount_marker,
        deletion_and_lineage,
        &[],
        &[],
        &[],
    )
}

/// 合取 7（`全空聚簇段数 ≥ N₀`）管不管这一次发布（[`EmptyClusterSegmentsConjunctReach`]，D3（空间分配） 已定项 7 登记表第 7 行）：
/// 写文件内容、建 inode、建快照的发布是前台写入类；管理员回退、删除类（unlink 与删快照）、整理批（`relocations` 非空）与
/// 别的不写文件内容也不建 inode 的发布（写行、暖机、抬 F 的空发布）免。
fn reach_of_the_empty_cluster_segments_conjunct(
    plan: &PublishPlan<'_>,
    deletion_and_lineage: &DeletionAndLineageOfThePublish,
    relocations: &[RelocationOfThePublish],
) -> EmptyClusterSegmentsConjunctReach {
    let is_a_rollback = match deletion_and_lineage.lineage {
        LineageStepOfThePublish::NoLineageRecord => false,
        LineageStepOfThePublish::AdministratorRollback { .. } => true,
    };
    let creates_a_snapshot = match deletion_and_lineage.snapshot {
        SnapshotStepOfThePublish::CreateSnapshot { .. } => true,
        SnapshotStepOfThePublish::NoNewSnapshot
        | SnapshotStepOfThePublish::DestructionIntent { .. }
        | SnapshotStepOfThePublish::DestructionBatch { .. }
        | SnapshotStepOfThePublish::DestructionFinalBatch { .. } => false,
    };
    // 变长与只改记录（判决 m4-trunc-r1 K10「变长当普通写」）也是前台写入类；截断的意图与各批是删除类、免。
    let changes_the_size_without_an_intent = match deletion_and_lineage.step {
        DeletionStepOfThePublish::SizeChangeWithoutAnIntent { .. }
        | DeletionStepOfThePublish::PartialWrite { .. } => true,
        DeletionStepOfThePublish::NotADeletion
        | DeletionStepOfThePublish::Intent { .. }
        | DeletionStepOfThePublish::Batch { .. }
        | DeletionStepOfThePublish::FinalBatch { .. }
        | DeletionStepOfThePublish::TruncationIntent { .. }
        | DeletionStepOfThePublish::TruncationBatch { .. }
        | DeletionStepOfThePublish::TruncationFinalBatch { .. } => false,
    };
    let writes_in_the_foreground = plan.file.is_some()
        || !plan.new_inode_records.is_empty()
        || creates_a_snapshot
        || changes_the_size_without_an_intent;
    if writes_in_the_foreground && !is_a_rollback && relocations.is_empty() {
        EmptyClusterSegmentsConjunctReach::ForegroundWrite
    } else {
        EmptyClusterSegmentsConjunctReach::Exempt
    }
}

/// 一次发布落盘之前、读盘与动分配器之前的那几判（[`prepare_the_version_publish_relocating_units`] 开头那一段，发布路径与
/// [`journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write`] 共用这一份，判定与之后的写读同一份输入）：
/// 算这次之后每棵树的样子与这次重写的角色（分配记录树的固定点在分配器的拷贝上走）、这次的单元落不落得下、切成几条记录、合取 7、
/// 空间准入的式子。分配器只读、一个盘都不读、一个写都不发。
///
/// # Errors
/// 同 [`prepare_the_version_publish`] 里排在读盘核之前的那几种：`InodeTreeWriteRefused`、`SpaceAdmissionRefused`、`MultiLevelCodeTwoTreeRefused`、
/// 释放判定路径的错、`PlacementRefused`、`AllocationRecordTreeRewriteSetDidNotSettle`、`JournalRecordsOfThePublishExceedTheLimit`、
/// `EmptyClusterSegmentsBelowTheFloor`。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：写入口、分配器、计划、上一版、树号、删除这一步、这次要搬的单元、不搬而换了位置的映射 key"
)]
pub(super) fn judge_the_publish_before_any_read_or_write<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &PoolAllocator,
    plan: &PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    trees: &FileVersionTreeIdentifiers,
    deletion_and_lineage: &DeletionAndLineageOfThePublish,
    relocations: &[RelocationOfThePublish],
    mapping_keys_whose_entries_change_without_a_relocation: &[Vec<u8>],
) -> Result<PublishJudgedBeforeAnyReadOrWrite, PublishError> {
    // 先算这次之后每棵树是什么样、这次重写哪些角色、换下哪些：只读（固定点在分配器的拷贝上走），条款没写的那几格与多层码 2 树的拒绝、
    // 释放判定的错、取不到落点都在这里交回，盘上逐字节不变。记账行按分配器里的那几块盘展开（装记账行时读的是同一批）。
    let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator
        .devices
        .iter()
        .map(|device_map| device_map.device)
        .collect();
    // 准入先判这次的单元落不落得下（C545（空间准入罩不住分裂与聚簇段层），用户 2026-09-27 定准入先拒）：下面走固定点时就在分配器的拷贝上
    // 取落点，取不到交回 `PlacementRefused`，排在式子那一判之前——落得下不下要在它之前判。判不判与式子那一判同一条：有普通分配、
    // 准入判着。数据单元要段外成对的空槽、提交内生块要没挡的槽（普通分配那一半照计划数，固定点按 ckpt_cost 的最坏数），
    // 读的是分配器增量维护的两个数；不够交回 `SpaceAdmissionRefused`（会话照旧推抬 F 再判），在动分配器与任何写之前，盘上逐字节不变。
    let ordinary_allocation_roles = plan
        .ordinary_allocation_part(previous, trees, deletion_and_lineage)?
        .rewritten_roles();
    let units_landing_is_judged = match allocator.space_admission() {
        SpaceAdmission::JudgedByTheFormula => {
            publish_has_an_ordinary_allocation(&ordinary_allocation_roles)
        }
        SpaceAdmission::SkippedByTheTestOnlySwitch => {
            // 关掉准入的那一档这一判与式子那一判都不判，留给落点那一道在任何写之前拒。
            false
        }
    };
    if units_landing_is_judged {
        admit_the_units_landing_on_every_device(
            &PlacementRoomOnOneDevice::of_every_device_of_the_allocator(allocator),
            units_of_a_publish_to_land(
                &ordinary_allocation_roles,
                checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
                    previous,
                    allocator,
                    pool.code_two_tree_node_capacities(),
                ),
            ),
        )
        .map_err(PublishError::SpaceAdmissionRefused)?;
    }
    let settled = settle_the_allocation_record_tree(
        plan,
        previous,
        trees,
        &device_identities_of_the_accounting_rows,
        pool.code_two_tree_node_capacities(),
        allocator,
        deletion_and_lineage,
        relocations,
        mapping_keys_whose_entries_change_without_a_relocation,
        pool.snapshot_lifetime_judgement(),
    )?;
    pool.rounds_of_the_last_allocation_record_tree_settling
        .set(settled.rounds_walked);
    // 这次发布切出来的记录条数（代码审阅第 26 条）：角色定下来就数得出，在准入、读盘核与动分配器之前判。
    let journal_records_of_the_publish = records_of_a_publish_naming_the_relocated_copies(
        &settled.resolved.rewritten_roles,
        settled.relocations_in_effect.len(),
        pool.journal_record_named_entry_capacity(),
    );
    refuse_a_publish_of_more_journal_records_than_the_limit(
        journal_records_of_the_publish,
        pool.parameters.geometry.journal_ring_bytes,
    )?;
    // 合取 7（D3（空间分配） 已定项 7 登记表第 7 行：`全空聚簇段数 ≥ N₀`，只拦前台写入类发布）：判不判与式子那一判同一个开关、
    // 同样只判带普通分配的发布；在式子那一判、读盘核与动分配器之前拒，盘上逐字节不变。`N₀` 取分配器上整理开关里的那一档（产品路径是第一版的 1）。
    let empty_cluster_segments_conjunct_is_judged = match (
        allocator.space_admission(),
        reach_of_the_empty_cluster_segments_conjunct(plan, deletion_and_lineage, relocations),
    ) {
        (
            SpaceAdmission::JudgedByTheFormula,
            EmptyClusterSegmentsConjunctReach::ForegroundWrite,
        ) => publish_has_an_ordinary_allocation(&settled.resolved.rewritten_roles),
        (SpaceAdmission::JudgedByTheFormula, EmptyClusterSegmentsConjunctReach::Exempt)
        | (SpaceAdmission::SkippedByTheTestOnlySwitch, _) => false,
    };
    if empty_cluster_segments_conjunct_is_judged {
        let empty_cluster_segments_on_each_device: Vec<(DeviceIdentity, u64)> = allocator
            .devices
            .iter()
            .map(|device_map| (device_map.device, device_map.empty_segments()))
            .collect();
        admit_the_empty_cluster_segments_floor_on_every_device(
            &empty_cluster_segments_on_each_device,
            allocator
                .compaction()
                .switches
                .empty_cluster_segments_floor
                .segments(),
        )
        .map_err(PublishError::EmptyClusterSegmentsBelowTheFloor)?;
    }
    // 空间准入（D28（挂载期承诺量） 已定项 1 的式子逐设备合取）：读数取分配器此刻的计数——这次的释放与取落点都还没做——，
    // 实例切换的预留按下一次可写挂载的 rows0（已定项 3「发布路径用 rows0 + 1」）；需求 = 这次新写的全部角色（普通分配加固定点，
    // 不加换下的槽）；一个普通分配都没有的发布（空发布、写行、暖机、抬 F）不判，它们的空间在保留池与切换预留里
    // （`admission` 的模块文档）。在读盘核与动分配器之前拒，盘上逐字节不变。
    // 只供测试的开关关掉准入时（`SpaceAdmission::SkippedByTheTestOnlySwitch`）不判，留给落点那一道在任何写之前拒。
    let demand = demand_of_the_roles_on_each_device(
        &settled.resolved.rewritten_roles,
        &device_identities_of_the_accounting_rows,
    );
    // 删除类发布（意图、各批、最后一批）不判式子那一判、落点那一判照判（D28（挂载期承诺量） 已定项 1：它们的空间罩在环窗口里）。
    // 删快照的三种发布与 unlink 同准入（判决第七节岔路 4「删快照照 unlink 的过程做、同等准入」）：式子那一判同样不判。
    // 整理的各次发布（`publish_compaction_step`）走 `NotADeletion` 那一臂。整理不是释放路径、走常规准入（D26（后台整理与放置回收）
    // 已定项 3 第 2 句；C906「照条款：整理批走常规准入」）：搬了单元的一批照判式子那一判，哪怕重写的角色只有固定点，
    // 批次最坏占用（计划里每一项都真搬，[`worst_case_occupancy_of_a_compaction_batch`]）加进这一次读数的已承诺预留；签发重写 logged_ops 根
    // （普通分配），照判。可用压到需求之下就拒（`SpaceAdmissionRefused`），会话照暂停办：单发一次只删意图的发布丢掉意图。
    // 只删整理意图、不搬单元的发布（意图达成而 R 已空、丢意图、暂停与出错停下时删意图）归删除类（D28（挂载期承诺量） 已定项 1 的不判名单）：
    // 式子那一判不判，落点那一判在上面照判（logged_ops 里另有截断意图、重写 logged_ops 根时）。
    // 截断的意图、各批、最后一批同样当删除类（判决 m4-trunc-r1 K10）；变长与只改记录当普通写，照判。
    let demand_is_judged = match (allocator.space_admission(), deletion_and_lineage.step) {
        (SpaceAdmission::JudgedByTheFormula, step) => {
            !step.is_a_deletion_class_publish()
                && !deletion_and_lineage.snapshot.is_a_snapshot_destruction()
                && !deletes_only_the_compaction_intent(deletion_and_lineage, relocations)
                && (publish_has_an_ordinary_allocation(&settled.resolved.rewritten_roles)
                    || !relocations.is_empty())
        }
        (SpaceAdmission::SkippedByTheTestOnlySwitch, _) => false,
    };
    if demand_is_judged {
        let replicas = ReplicaCount::of_every_device_in_the_pool(allocator);
        let worst_case_occupancy_of_the_batch = worst_case_occupancy_of_a_compaction_batch(
            relocations
                .iter()
                .map(|relocation| relocation.source.span)
                .sum(),
            replicas,
        );
        admit_on_every_device(
            &admission_reading_before_a_publish_on_a_ring_of(
                allocator,
                previous,
                committed_reservation_for_tombstones_and_retirement_records_after_this_publish(
                    &settled.resolved,
                    previous,
                    replicas,
                )
                .plus(worst_case_occupancy_of_the_batch),
                pool.parameters.geometry.root_ring_slots_per_region,
            ),
            &demand,
        )
        .map_err(PublishError::SpaceAdmissionRefused)?;
    }
    Ok(PublishJudgedBeforeAnyReadOrWrite {
        settled,
        device_identities_of_the_accounting_rows,
        journal_records_of_the_publish,
    })
}

/// 这次发布只删整理意图、不搬单元（[`CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords`] 不带一批、[`CompactionIntentStep::Drop`]）：
/// 归删除类，式子那一判不判（D28（挂载期承诺量） 已定项 1 的不判名单，[`judge_the_publish_before_any_read_or_write`]）。
/// logged_ops 那一步之外别的都照抄（整理的发布不写文件、不建 inode、不建也不删快照），只看 logged_ops 那一步与搬不搬单元。
fn deletes_only_the_compaction_intent(
    deletion_and_lineage: &DeletionAndLineageOfThePublish,
    relocations: &[RelocationOfThePublish],
) -> bool {
    let deletes_the_compaction_intent = match deletion_and_lineage.logged_operations {
        LoggedOperationsStepOfThePublish::DeleteTheCompactionIntent { .. } => true,
        LoggedOperationsStepOfThePublish::CarryTheLoggedOperations
        | LoggedOperationsStepOfThePublish::IssueTheCompactionIntent(_)
        | LoggedOperationsStepOfThePublish::IssueTheTruncationIntent(_)
        | LoggedOperationsStepOfThePublish::DeleteTheTruncationIntent { .. }
        | LoggedOperationsStepOfThePublish::TakeTheTruncationEntryOfTheRollbackTarget(_) => false,
    };
    deletes_the_compaction_intent && relocations.is_empty()
}

/// [`judge_the_publish_before_any_read_or_write`] 判过的：走到固定点的这一次发布、记账行的盘、这次切成几条 journal 记录。
pub(super) struct PublishJudgedBeforeAnyReadOrWrite {
    pub(super) settled: SettledPublish,
    pub(super) device_identities_of_the_accounting_rows: Vec<DeviceIdentity>,
    pub(super) journal_records_of_the_publish: usize,
}

/// 这次发布释放之前读盘核的全部读（[`prepare_the_version_publish_relocating_units`]）：换下的角色、记着只坏一份的豁免映射的单元、删快照这一批放的条目、搬走的源落点，
/// 交回要隔离的那几份。经 `reader` 读，调用方在它返回之后（`Err` 也是）取数到的只坏一份。
///
/// # Errors
/// 映射条目的位置项指池外的盘、两条指同一块盘；删快照这一批放的条目认不出角色；[`copies_failing_the_release_checksum_check`] 那几种。都在动分配器的账与任何写之前。
fn copies_quarantined_by_the_release_checks<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    allocator: &mut PoolAllocator,
    plan: &PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    settled: &SettledPublish,
    device_identities_of_the_accounting_rows: &[DeviceIdentity],
    release_checksum_check: ReleaseChecksumCheck,
) -> Result<Vec<CopyQuarantinedAfterReleaseChecksumMismatch>, PublishError> {
    let kept_for_the_snapshots = &settled
        .resolved
        .deadlist
        .user_visible_roles_kept_for_the_snapshots;
    let mut quarantine = match (previous, release_checksum_check) {
        (Some(previous_version), ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt) => {
            let mut copies_failing = copies_failing_the_release_checksum_check(
                previous_version,
                &settled.previous_roles_replaced,
                kept_for_the_snapshots,
                reader,
                FileContentOfThePublish::of_the_plan(plan),
            )?;
            copies_failing.extend(copies_of_the_exempt_units_read_with_one_copy_failing(
                previous_version,
                &settled.previous_roles_replaced,
                &plan.instance_table,
                &allocator.locations_read_with_one_copy_failing_waiting_for_the_next_user_change(),
                reader,
            ));
            copies_failing
        }
        (Some(previous_version), ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition) => {
            // 预演不读盘，但映射条目的位置项指池外的盘、两条指同一块盘那一判不读盘，照样判（发布路径在读盘核之前判的同一件事）。
            refuse_mapping_entries_that_do_not_name_two_pool_devices(
                previous_version,
                &settled.previous_roles_replaced,
                kept_for_the_snapshots,
                device_identities_of_the_accounting_rows,
                FileContentOfThePublish::of_the_plan(plan),
            )?;
            Vec::new()
        }
        (None, _) => Vec::new(),
    };
    // 删快照这一批放的条目同样先按留存的映射条目读盘核（判决 S4「每条先核还在、读盘核校验和」；条目还在由 `resolve` 从上一版的 deadlist 取保证）。
    for (entry, locations) in &settled
        .resolved
        .deadlist
        .entries_released_by_the_destruction
    {
        let role = role_standing_for_the_unit_of_a_deadlist_entry(entry)?;
        let not_releasable = |cause: PublishError| PublishError::DeadlistEntryNotReleasable {
            mapping_key: entry.mapping_key.clone(),
            refusal: DeadlistEntryReleaseRefusal::PlacementNotReleasable(Box::new(cause)),
        };
        refuse_locations_that_do_not_name_two_pool_devices(
            role,
            locations,
            device_identities_of_the_accounting_rows,
        )
        .map_err(not_releasable)?;
        match release_checksum_check {
            ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt => {
                quarantine.extend(
                    copies_of_a_released_unit_failing_the_release_checksum_check(
                        role, locations, reader,
                    ),
                );
            }
            ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition => {}
        }
    }
    // 搬走的源落点释放之前同样按映射位置项读盘核（判决 G3「源落点写释放代进 defer」走的是同一条释放判定路径）。签发之后强制一次
    // 「释放前核出对不上」那个只供测试的开关（判决 G6）落在这一批第一个源落点上：两份都当核出对不上，隔离（D19 已定项 5）。
    for (position, relocation) in settled.relocations_in_effect.iter().enumerate() {
        refuse_locations_that_do_not_name_two_pool_devices(
            relocation.role_for_the_checks,
            &relocation.source_locations,
            device_identities_of_the_accounting_rows,
        )?;
        match release_checksum_check {
            ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt => {
                let is_forced_to_mismatch = position == 0
                    && allocator
                        .compaction_mut()
                        .take_the_forced_release_checksum_mismatch();
                if is_forced_to_mismatch {
                    quarantine.extend(relocation.source_locations.iter().enumerate().map(
                        |(copy_position, location)| CopyQuarantinedAfterReleaseChecksumMismatch {
                            unit: relocation.role_for_the_checks,
                            device: location.device,
                            placement: relocation.source,
                            reading: if copy_position == 0 {
                                QuarantinedCopyReading::ChecksumMismatch
                            } else {
                                QuarantinedCopyReading::IntactButAnotherCopyFailed
                            },
                        },
                    ));
                } else {
                    quarantine.extend(
                        copies_of_a_released_unit_failing_the_release_checksum_check(
                            relocation.role_for_the_checks,
                            &relocation.source_locations,
                            reader,
                        ),
                    );
                }
            }
            ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition => {}
        }
    }
    Ok(quarantine)
}

/// 同 [`prepare_the_version_publish`]，另带这次要搬的单元（整理的一批，[`publish_relocation_batch`]；别的发布给空的）：
/// 固定点与真发都按「释放（换下的与搬走的源落点）→ 取目的地 → 取这次重写的角色的落点」走同一串，源落点照释放判定路径逐盘核、
/// 读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）；这次重写要换下的角色不另搬（它的新落点本来就不在活 R 里）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：写入口、分配器、计划、上一版、树号、读不读盘核、卸载记号、删除这一步、这次要搬的单元、不搬而换了位置的映射 key、\
              读回旧单元改好的新净荷"
)]
fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: &PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    trees: FileVersionTreeIdentifiers,
    release_checksum_check: ReleaseChecksumCheck,
    unmount_marker: UnmountMarker,
    deletion_and_lineage: &DeletionAndLineageOfThePublish,
    relocations: &[RelocationOfThePublish],
    mapping_keys_whose_entries_change_without_a_relocation: &[Vec<u8>],
    payloads_of_the_rewritten_data_units: &[RewrittenDataUnitPayload],
) -> Result<(PublishWrites, TransactionOutput), PublishError> {
    let PublishJudgedBeforeAnyReadOrWrite {
        settled,
        device_identities_of_the_accounting_rows,
        journal_records_of_the_publish: _,
    } = judge_the_publish_before_any_read_or_write(
        pool,
        allocator,
        plan,
        previous,
        &trees,
        deletion_and_lineage,
        relocations,
        mapping_keys_whose_entries_change_without_a_relocation,
    )?;
    // 经映射核到的那几个单元，释放之前再按位置项读盘核一次校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）：只读，
    // 读不到就在动分配器、发任何一个写之前返回。第一个文件版本换下的 mkfs 那片树表与树表 0 条那一版的分配记录树节点不经映射，不核。
    // 释放之前读盘核读到只坏一份的进这次挂载别的读那一份账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）。
    let devices_counting_unit_reads_with_one_copy_failing =
        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*pool.devices);
    let quarantine_read = copies_quarantined_by_the_release_checks(
        &devices_counting_unit_reads_with_one_copy_failing,
        allocator,
        plan,
        previous,
        &settled,
        &device_identities_of_the_accounting_rows,
        release_checksum_check,
    );
    // 核不过提前返回的那一次，返回之前读到的只坏一份同样进账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）。
    allocator.count_the_other_unit_reads_with_one_copy_failing(
        devices_counting_unit_reads_with_one_copy_failing
            .locations_of_the_units_read_with_one_copy_failing(),
    );
    let quarantine = quarantine_read?;
    publish_admitted(
        pool,
        allocator,
        plan,
        previous,
        &settled.resolved,
        &settled.release,
        &quarantine,
        trees,
        unmount_marker,
        deletion_and_lineage.lineage,
        deletion_and_lineage.snapshot,
        &settled.relocations_in_effect,
        payloads_of_the_rewritten_data_units,
    )
}

/// 这次发布切出来的记录条数：没有搬迁副本时同 [`roles_named_by_each_record_of_the_publish`] 的项数；整理的一批（不重写数据单元，
/// 一个事务）点名项 = 搬迁副本 + 这次重写的角色，按一条记录装几个点名项切（判决 G3「journal 点名项 = 搬迁副本 + 新写的节点，一个事务号」）。
fn records_of_a_publish_naming_the_relocated_copies(
    rewritten: &[TransactionUnit],
    relocated_copies: usize,
    named_entry_capacity: JournalRecordNamedEntryCapacity,
) -> usize {
    if relocated_copies == 0 {
        return roles_named_by_each_record_of_the_publish(rewritten, named_entry_capacity).len();
    }
    (relocated_copies + rewritten.len()).div_ceil(named_entry_capacity.named_entries_per_record())
}

/// 整理的一批要搬的一个单元，交给发布路径之前验过（[`publish_relocation_batch`]）：映射 key、它归谁、释放判定与写账用哪个角色
/// （留存条目没有角色，按类标签取 [`user_visible_role_standing_for_a_unit_class`]）、源落点（映射位置）、原样的字节、诞生代号。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RelocationOfThePublish {
    pub(super) mapping_key: Vec<u8>,
    pub(super) holder: RelocatedUnitHolder,
    pub(super) role_for_the_checks: TransactionUnit,
    pub(super) source_locations: [LocationEntry; 2],
    pub(super) source: Placement,
    pub(super) bytes: Vec<u8>,
    pub(super) birth_txg: CheckpointTxg,
}

/// 一个搬迁单元的目的落点（判决 G2：由这一批发布时的分配器现取，不缓存）：照开关里的目的地政策取，分配代写单元的诞生代
/// （D26（后台整理与放置回收） 已定项 5 第 2 条、D3（空间分配） 已定项 7「分配代」口径），活 R 由分配器排除。
///
/// # Errors
/// 取不到 ⇒ `PlacementRefused`，带着释放判定用的那个角色。提交内生块的搬迁回落时同 [`allocate_placement_for_role`]，先取 `rewrite_set_leaves` 里的空槽。
pub(super) fn allocate_relocation_destination(
    allocator: &mut PoolAllocator,
    relocation: &RelocationOfThePublish,
    rewrite_set_leaves: &RewriteSetLeaves,
) -> Result<Placement, PublishError> {
    let rule = relocation.role_for_the_checks.placement();
    let placement = match (allocator.compaction().switches.destination_policy, rule) {
        (
            RelocationDestinationPolicy::UserDataPlacementForDataAndOpenSegmentForTheRest,
            PlacementRule::UserData,
        ) => allocator.try_allocate_user_data(relocation.birth_txg),
        (
            RelocationDestinationPolicy::UserDataPlacementForDataAndOpenSegmentForTheRest,
            PlacementRule::CommitGenerated(footprint),
        ) => allocator.try_allocate_commit_generated_preferring_the_rewrite_set(
            footprint,
            relocation.birth_txg,
            rewrite_set_leaves,
        ),
        (
            RelocationDestinationPolicy::LowestUnblockedSlotForEveryUnitForTests,
            PlacementRule::UserData,
        ) => allocator.try_allocate_at_the_lowest_unblocked_slot_for_tests(
            UnitFootprint::TwoSlotsAligned,
            relocation.birth_txg,
        ),
        (
            RelocationDestinationPolicy::LowestUnblockedSlotForEveryUnitForTests,
            PlacementRule::CommitGenerated(footprint),
        ) => allocator
            .try_allocate_at_the_lowest_unblocked_slot_for_tests(footprint, relocation.birth_txg),
    };
    placement.map_err(|refusal| PublishError::PlacementRefused {
        unit: relocation.role_for_the_checks,
        refusal,
    })
}

/// 这次发布算定了的样子：每棵树这次之后的形状与重写的角色、这次换下的上一版角色、要释放的落点（按 bump 次序）。
pub(super) struct SettledPublish {
    pub(super) resolved: ResolvedPublish,
    pub(super) previous_roles_replaced: Vec<TransactionUnit>,
    pub(super) release: Vec<Placement>,
    /// 这次真搬的那几个（要搬的里面去掉这次重写本来就换下的角色），按计划的次序。
    pub(super) relocations_in_effect: Vec<RelocationOfThePublish>,
    /// 固定点走了几轮（定下来的那一轮算在内）。
    pub(super) rounds_walked: usize,
}
