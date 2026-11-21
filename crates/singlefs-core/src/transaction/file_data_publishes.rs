//! 写文件数据的发布：第一个文件、覆盖写、顺序写、新 inode 与一起发布的几处数据变化。

use super::*;

/// 整理的一批换了位置的映射条目（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「映射条目换位置」）：`plan` 里装着
/// `moved_keys` 每一把的那片叶、连同从根下来那条路上的每个节点，都改成这次重写（D8（核心索引结构） 已定项 11 ①「路径上的节点整条 COW」）。
/// [`plan_the_tree_after_this_publish`] 只按 key 进出定哪些节点重写；搬迁的 key 不变、条目的位置变了，不在这里标，装着它的叶就照抄上一版、
/// 盘上的条目留在已经释放的源落点，重建按映射覆盖提示时把单元指回源落点。形状不变，只改节点的来处；上一版换下的节点按新的来处重算。
/// `moved_keys` 里每一把都在这一版的 key 里（调用方按这一版的 key 集合筛过）。
pub(super) fn plan_rewriting_the_paths_to_the_moved_entries(
    mut plan: CodeTwoTreePlan,
    previous_shape: &CodeTwoTreeShape,
    moved_keys: impl Iterator<Item = CodeTwoTreeKey>,
) -> CodeTwoTreePlan {
    for moved_key in moved_keys {
        let mut position = plan
            .shape
            .root()
            .expect("这一版的中央映射树装着搬走的这把 key：有根")
            .position;
        // 迭代上界：每一轮下降一层，至多树高那么多轮；走到叶就停。路由同 `leaf_position_routed_to`。
        loop {
            let index = plan
                .shape
                .nodes()
                .binary_search_by_key(&position, |node| node.position)
                .expect("位置取自同一份形状里的子引用");
            plan.origins[index] = CodeTwoTreeNodeOrigin::RewrittenThisPublish;
            match &plan.shape.nodes()[index].contents {
                CodeTwoTreeNodeContents::Leaf { keys } => {
                    assert!(
                        keys.contains(&moved_key),
                        "按分隔 key 走到的叶装着这把 key：调用方只交这一版里有的 key，规划出来的形状按分隔 key 路由"
                    );
                    break;
                }
                CodeTwoTreeNodeContents::Internal { children } => {
                    let child_index = children
                        .iter()
                        .rposition(|child| child.separator_key <= moved_key)
                        .unwrap_or(0);
                    position = children[child_index].child;
                }
            }
        }
    }
    let carried: BTreeSet<CodeTwoTreeNodePosition> = plan
        .origins
        .iter()
        .filter_map(|origin| match origin {
            CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => Some(*previous_position),
            CodeTwoTreeNodeOrigin::RewrittenThisPublish => None,
        })
        .collect();
    plan.replaced_previous_nodes = previous_shape
        .nodes()
        .iter()
        .map(|node| node.position)
        .filter(|position| !carried.contains(position))
        .collect();
    plan
}

/// 截断的三步与变长、只改记录那一次改 size 的那个文件（[`DeletionStepOfThePublish::inode_whose_size_changes`]，任意活 inode：判决
/// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3）改 size 之前与之后的样子：之前取上一版这个文件的单元数与那条记录的 size，
/// 之后取这一步留下的单元数（意图、变长、只改记录不放单元，两种批留下 `data_units_kept` 个）与这一步之后的 size（意图与变长、只改记录写进
/// 记录的那一个；各批不碰记录）。不是这几步、或上一版里没有那条记录，交回 `None`：extent 树整棵照抄。
pub(super) fn size_change_of_a_file(
    step: DeletionStepOfThePublish,
    previous: Option<&TransactionOutput>,
) -> Option<SizeChangeOfAFileOfThePublish> {
    let previous_version = previous?;
    let inode = step.inode_whose_size_changes()?;
    let record_before = previous_version.inode_record_of(inode)?;
    let data_units_before = data_units_of_the_file_in_the_version(previous_version, inode);
    let (size_after_in_bytes, data_units_after) = match step {
        DeletionStepOfThePublish::TruncationIntent {
            record_after_the_intent: record,
            ..
        }
        | DeletionStepOfThePublish::SizeChangeWithoutAnIntent {
            record_after_the_change: record,
            ..
        } => (record.size, data_units_before),
        // 部分写：写之后的单元数（写之前的与写到的最后一个单元号加一里大的），size 取写之后那条记录的（判决 m4-partial-write-r1 第二节 R5）。
        DeletionStepOfThePublish::PartialWrite {
            record_after_the_write: record,
            rewritten_data_units,
        } => (
            record.size,
            rewritten_data_units.end_exclusive.max(data_units_before),
        ),
        DeletionStepOfThePublish::TruncationBatch {
            data_units_kept, ..
        }
        | DeletionStepOfThePublish::TruncationFinalBatch {
            data_units_kept, ..
        } => (record_before.size, data_units_kept),
        DeletionStepOfThePublish::NotADeletion
        | DeletionStepOfThePublish::Intent { .. }
        | DeletionStepOfThePublish::Batch { .. }
        | DeletionStepOfThePublish::FinalBatch { .. } => return None,
    };
    Some(SizeChangeOfAFileOfThePublish {
        inode,
        change: SizeChangeOfTheFile {
            data_units_before,
            data_units_after,
            rewritten_data_units: step
                .data_units_rewritten_from_their_payloads()
                .map_or(DataUnitsRewrittenInOnePublish::NONE, |rewritten| {
                    rewritten.units
                }),
            tail_before: TailOfAFile::of_data_units_and_size(data_units_before, record_before.size),
            tail_after: TailOfAFile::of_data_units_and_size(data_units_after, size_after_in_bytes),
        },
        size_after_in_bytes,
    })
}

/// 一次改 size 的发布改的那个文件（[`size_change_of_a_file`] 的结果）：号、改之前与之后的样子、这一步之后那条记录的 size。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SizeChangeOfAFileOfThePublish {
    pub(super) inode: InodeNumber,
    pub(super) change: SizeChangeOfTheFile,
    pub(super) size_after_in_bytes: u64,
}

/// 第一个文件之外 `size_change.inode` 那个文件改 size 那次发布之后的 extent 树（第一个文件那一份 [`ExtentTreePlan`]）：第一个文件的下段整段照抄；
/// 上段走统一计划（[`extent_tree_plan_with_the_unified_upper_segment`]）——这次之后有条目的是 `files_other_than_the_first`（这次之后那一份计划，
/// 它在不在里面已按粘住的标签 0 定好）连同第一个文件，条目变了的是它（它那一条这次变了：上一版没有、或
/// [`SizeChangeOfTheFile::rewrites_the_upper_leaf_entry`]）。那个文件自己的下段在 `files_other_than_the_first` 里（判决 m4-multi-inode-size-r1 第二节 S3）。
pub(super) fn plan_the_extent_tree_after_the_size_change_of_a_file_other_than_the_first(
    previous: &TransactionOutput,
    files_other_than_the_first: &FilesOtherThanTheFirstPlan,
    size_change: SizeChangeOfAFileOfThePublish,
) -> ExtentTreePlan {
    let first_file_has_an_entry = previous
        .inodes_with_an_extent_upper_leaf_entry()
        .contains(&FIRST_INODE_NUMBER);
    let inodes_with_an_entry_after_this_publish: BTreeSet<u64> = first_file_has_an_entry
        .then_some(FIRST_INODE_NUMBER)
        .into_iter()
        .chain(
            files_other_than_the_first
                .files
                .iter()
                .map(|file| file.inode.0),
        )
        .collect();
    let had_an_entry = previous
        .file_other_than_the_first(size_change.inode)
        .is_some();
    let has_an_entry = inodes_with_an_entry_after_this_publish.contains(&size_change.inode.0);
    let entry_changed =
        had_an_entry != has_an_entry || size_change.change.rewrites_the_upper_leaf_entry();
    let inodes_whose_entry_changed: BTreeSet<u64> = entry_changed
        .then_some(size_change.inode.0)
        .into_iter()
        .collect();
    extent_tree_plan_with_the_unified_upper_segment(
        Some(&previous.extent_tree),
        previous
            .extent_tree
            .lower_nodes
            .iter()
            .map(|(position, _)| {
                (
                    *position,
                    ExtentTreeNodeOrigin::CarriedFromThePreviousVersion,
                )
            })
            .collect(),
        Vec::new(),
        &inodes_with_an_entry_after_this_publish,
        &inodes_whose_entry_changed,
    )
}

/// 文件版本那条 inode 记录（D8（核心索引结构） 已定项 6 的字段表）：size = 这次写的内容长度；
/// blocks 由 size 现算（⌈size ÷ 512⌉，[`InodeRecord::to_bytes`]），不按这个对象分到几个数据单元填。
pub(super) fn inode_record_of_file_version(file: &FileVersionPlan<'_>) -> InodeRecord {
    InodeRecord {
        inode: file.inode.0,
        object_birth: file.inode_object_birth,
        nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
        size: u64::try_from(file.content.len()).expect("文件长度"),
        change_count: file.change_count,
        write_time_seconds: file.write_time_seconds,
        // 整份写（顺序写、覆盖写）清零（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R2、第六节用户定案）：
        // 这一版从偏移 0 重写了全部单元，没有比它早的单元要靠截断字段判死。
        truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
    }
}

/// 一个池里的第一个文件版本（字节表零 t1..t12 + 六 + 七）：分配十二个落点、装十二个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。
/// 建在树表 0 条的那一版上（`version_to_build_on`）：照抄它的实例表指针、换下它指着的那片 mkfs 树表。
///
/// txg 与 jsn 都从它接着算——txg = 那一版的 txg + 1、jsn = 上一条记录的 jsn + 1，**不写死 3**：
/// `NEW_POOL_FILE_CREATION_TXG` 只管 mkfs 同一个进程里那条流（mkfs → 取号 → 暖机两次 → 新池新建文件），不管任何池的第一个文件版本
/// （2026-09-23 用户定案）。只做过 mkfs 的池重开一次可写挂载之后再写文件时，那一版已经推到 txg 4，这里接着写 txg 5。
///
/// 这一版的十二棵树从 `version_to_build_on` 的树 ID 水位起连号发（[`FileVersionTreeIdentifiers::issued_from_watermark`]），
/// 新水位 = max(那一版的水位, 发出的最高号 + 1)（D8（核心索引结构） 已定项 8 ②）。那一版的水位就是根环里全部根记录的 max：
/// 它是这个会话里的现行那一版，挂载时本实例的第一次发布取过环里的 max（`mount` 的 `tree_identifier_watermark_of_the_ring`），
/// 之后每次发布只照抄或推高它，而环里后来写进去的根都是这个会话自己写的。mkfs 同一个进程里那条流上环里只有 mkfs 的根与暖机根，
/// 都带 mkfs 种下的 65536。
///
/// # Errors
/// `version_to_build_on` 那一版的树表读不出、解不开 ⇒ `TreeTableOfTheVersionToBuildOnUnreadable`；
/// 那一版的树表不是 0 条（已经有过文件版本）⇒ `FirstFileVersionOnAVersionThatAlreadyHasAFile`（同一个文件再写一版走
/// `publish_overwrite`）。`previous_record_bytes` 解不出本池的记录，或它的 checkpoint_txg 与 `version_to_build_on` 那条根的不同
/// （两者要说同一版，新根才恒落在它上面一格）⇒ `FirstFileVersionDoesNotFollowTheVersionItBuildsOn`
/// （m2-emptypool-nonempty-r1 云端攻方腿 Z3-A：拿 txg 4 的暖机根配 txg 2 的记录，新根盖在已有的根上、冷恢复读不到）。
/// 那一版的水位离 `u64::MAX` 不到十二个号 ⇒ `TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees`。
/// 这几样都在任何写之前返回，一个字节都不写。其余同 `publish_version`。
pub fn publish_first_file<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    version_to_build_on: &RootRecord,
    file: FirstFile<'_>,
    instance: InstanceGeneration,
    previous_record_bytes: &[u8],
) -> Result<TransactionOutput, PublishError> {
    // 下面补记零单元根会动分配器：冻结着一次没重发的发布时连它也不许做，第一道就拒（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）。
    refuse_while_a_publish_is_frozen(allocator)?;
    let first_file_version_txg =
        checkpoint_txg_of_the_publish_after(version_to_build_on.checkpoint_txg)?;
    // 「树还没建起来」看那一版的树表有几条（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步），不看水位：
    // 回退到树表 0 条的一版时水位带着根环里的 max（D8（核心索引结构） 已定项 8 ②），那一版没有树、水位却早已不是 mkfs 的 65536。
    let tree_table_entries = tree_table_entry_count(&*pool.devices, version_to_build_on)
        .map_err(|failure| PublishError::TreeTableOfTheVersionToBuildOnUnreadable { failure })?;
    if tree_table_entries != 0 {
        return Err(
            PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { tree_table_entries },
        );
    }
    let previous_record = JournalRecord::parse(
        previous_record_bytes,
        unit_filesystem_identifier(&pool.parameters.filesystem_identifier),
    )
    .map(|record| (record.checkpoint_txg, record.counter));
    let follows_directly = previous_record
        .is_some_and(|(previous_txg, _)| previous_txg == version_to_build_on.checkpoint_txg);
    if !follows_directly {
        return Err(
            PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn {
                version_to_build_on: version_to_build_on.checkpoint_txg,
                previous_record,
            },
        );
    }
    let (_, previous_counter) =
        previous_record.expect("follows_directly 为真时上一条记录解得出（is_some_and）");
    // 十二棵树从那一版的水位（下一个可用号）起连号发：水位是 mkfs 的 65536 时就是 D8（核心索引结构） 已定项 11 那几个常量，
    // 回退到树表 0 条的一版之后水位带着环里的 max，号高于此前发过的每一个，不重发。
    let (tree_identifiers, next_available_after_this_issue) =
        FileVersionTreeIdentifiers::issued_from_watermark(
            version_to_build_on.tree_identifier_watermark,
        )
        .map_err(PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees)?;
    // 那一版之前调用方直接发的零单元发布没经分配器：先把它们写下的根补记进分配器那张根环表，这次的根才接得上
    // （`PoolAllocator::record_zero_unit_roots_leading_to`；补记的根都已在盘上，这次发布失败也不用退）。
    allocator.record_zero_unit_roots_leading_to(version_to_build_on.checkpoint_txg);
    publish_version_of_trees_holding_one_data_unit(
        pool,
        allocator,
        PublishPlan {
            txg: first_file_version_txg,
            counter: previous_counter + 1,
            transaction: NEW_POOL_FILE_CREATION_NUMBER,
            // 这个实例的新池新建文件：在它之前只有暖机与写行那几次发布，事务号都是 0。
            highest_transaction_number_before_this_publish: 0,
            instance,
            back_chain: back_chain_of(previous_record_bytes),
            file: Some(FileVersionPlan {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                content: file.content,
                write_time_seconds: file.write_time_seconds,
                inode_object_birth: first_file_version_txg,
                // 改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88 的字段表定义）：
                // 就是这次发布的 txg（mkfs 同一个进程里那条流上是 3）。写 1 是 2026-09-18 之前的老样子
                // （暖机把新池新建文件从 txg 1 推到 3 时这一格没跟着改，增补 2 第 11 行）。
                // 这里不写 `txg.0`：`crates/mutations.tsv`「步 1 变异：改动计数留 1」那一行按那串字面锚在 `publish_overwrite` 上，同一份文件里出现两次它就腐化。
                change_count: first_file_version_txg.0,
            }),
            // 新池新建文件只建第一个文件那一个 inode：树是空的，这条记录建第一片容器（D8（核心索引结构） 已定项 6）。
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(version_to_build_on.instance_table),
            tree_birth_txg: first_file_version_txg,
            // D8（核心索引结构） 已定项 8 ②：max(根环里全部根记录的该字段, 本次发出的最高树 ID + 1)；
            // 前一项就是那一版的水位（见文档注释），后一项恒更大。
            tree_identifier_watermark: version_to_build_on
                .tree_identifier_watermark
                .max(next_available_after_this_issue),
            // F 照抄建在上面的那一版（D16（发布语义） 已定项 1：F 只升不降）。mkfs 同一个进程里那条流上它恒是 0，
            // 重开之后写行那次发布带的是恢复后生效的 F，这里接着带它。
            rollback_floor: version_to_build_on.rollback_floor,
        },
        None,
        tree_identifiers,
    )
}

/// 覆盖写（里程碑「覆盖写、释放、回退与复用」步 1 / 步 2）：同一个实例里接在上一次发布之后再发布一版同一个文件——txg、jsn、事务号各加一，
/// 对象出生代与容器身份不改，改动计数取这次的 txg；上一版的十二个落点经映射释放（进 defer 队列）。
/// 契约是一个数据单元：多个数据单元的内容走 `publish_sequential_write`。
///
/// # Errors
/// 内容装不进一个数据单元 ⇒ `ContentExceedsDataUnit`（在动分配器、动盘之前）；
/// 释放判定路径查不到上一版的某个单元（`ReleaseNotInMapping` 一族）、落点被拒（`PlacementRefused`，带分配器的原因）、装不下、
/// 块设备报错，都原样交回。
pub fn publish_overwrite<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    file: FirstFile<'_>,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let first_file_record = live_record_of_the_first_file(previous)?;
    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    publish_version_holding_one_data_unit(
        pool,
        allocator,
        PublishPlan {
            txg,
            counter: previous.record.counter + 1,
            transaction: previous.highest_transaction_number_in_this_instance + 1,
            highest_transaction_number_before_this_publish: previous
                .highest_transaction_number_in_this_instance,
            instance,
            back_chain: back_chain_of(&previous.record_bytes),
            file: Some(FileVersionPlan {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                content: file.content,
                write_time_seconds: file.write_time_seconds,
                inode_object_birth: first_file_record.object_birth,
                change_count: txg.0,
            }),
            // 覆盖写不建新 inode：只把第一个文件那条记录换一版，它落在原来那一片叶里（容器身份不变）。
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            // 这次一个树 ID 都不发：上一版的水位就是根环里全部根记录的 max（本会话每次发布只照抄或推高它，见 `publish_first_file`）。
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
    )
}

/// 顺序写一次写请求（里程碑「覆盖写、释放、回退与复用」并行线一）：从文件偏移 0 写整份内容，按切分纪律切成 N 个一单元事务
/// （`crate::write_request_split`：D16（发布语义） 已定项 5 末段 + D23（journal 的角色与格式） 已定项 7 +
/// C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案），N 个事务在同一次发布里写出 N 个数据单元、N 条记录：
/// 前 N − 1 条各只点名自己那个数据单元，这次发布共享的提交内生块只在最后一条点名（D23（journal 的角色与格式） 已定项 17，
/// C491（多条记录时共享内生块在哪条点名没定） 2026-09-23 定）；extent 叶记录 key 的 offset 段是文件字节偏移
/// （D8（核心索引结构） 已定项 3，C490（extent 叶 key 的 offset 段没定单位） 2026-09-23 定）。
///
/// 走的是 `publish_version` 那一条发布路径（`.claude/rules/fs-design.md`「一个事务层，所有结构共用」：这里不另开一条）：
/// 只要一个数据单元的内容，写出的字节与同内容的 `publish_overwrite` 一个不差。上一版比这一版多出来的数据单元
/// 在同一次发布里释放（文件变短的那一格，`placements_to_release_via_mapping`）。
///
/// 切出的单元多于一片下段叶装得下的 144 个时不拒：这个文件的 extent 树下段按单元号按位置寻址长成两层及以上（叶罩 144 个单元、
/// 内部扇出 147，D8（核心索引结构） 已定项 14），这一版下段的节点在同一次发布里重写、上一版的在同一次发布里释放；只有一个单元时
/// 不建下段，那个数据指针内联在上段叶条目里。
///
/// # Errors
/// 与 `publish_overwrite` 相同（不报 `ContentExceedsDataUnit`：多个数据单元正是这条路径要写的）。这条路径不按单元数拒：extent 树
/// 没有装不下这一格；最后一个事务要点名的项多于一条记录装得下的，末条再跨记录（D23（journal 的角色与格式） 已定项 17），也不拒。
pub fn publish_sequential_write<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    file: FirstFile<'_>,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let first_file_record = live_record_of_the_first_file(previous)?;
    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    // 改动计数与新池新建文件号先取成局部量、字段也换了次序：`crates/mutations.tsv` 有几条变异按 `publish_overwrite` 里
    // `change_count: txg.0,`、`transaction: previous.highest_transaction_number_in_this_instance + 1,` 与那几行字段的原样
    // 锚着，同一份文件里再出现一次它们就腐化（门禁 crates-mutation-replay 判锚点恰好命中一次；`publish_new_inodes` 里那句注释是同一件事）。
    let change_count = txg.0;
    let new_pool_file_creation_of_this_request =
        previous.highest_transaction_number_in_this_instance + 1;
    publish_version(
        pool,
        allocator,
        PublishPlan {
            txg,
            counter: previous.record.counter + 1,
            transaction: new_pool_file_creation_of_this_request,
            highest_transaction_number_before_this_publish: previous
                .highest_transaction_number_in_this_instance,
            instance,
            file: Some(FileVersionPlan {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                content: file.content,
                // 对象出生代不改（D8（核心索引结构） 已定项 6；I-9.10（对象出生代与 inode 记录相符））。
                inode_object_birth: first_file_record.object_birth,
                write_time_seconds: file.write_time_seconds,
                change_count,
            }),
            back_chain: back_chain_of(&previous.record_bytes),
            // 顺序写不建新 inode：只把第一个文件那条记录换一版，它落在原来那一片叶里（容器身份不变）。
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            // 这次一个树 ID 都不发：上一版的水位就是根环里全部根记录的 max。
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
    )
}

/// 建 N 个 inode（里程碑「覆盖写、释放、回退与复用」并行线三：多个文件、元数据按「个」量、没有目录）：一次发布把 N 条 inode 记录
/// 写进 inode 树——号从记账里的 inode 号水位起连着发（D5（快照 / 空间记账机制） 已定项 4 第 12 项），
/// 落在最右那片叶、满 233 条就在末尾分裂（D8（核心索引结构） 已定项 6，`crate::inode_tree`），
/// 这次发布之后水位加 N；数据单元与 extent 树不动（新建的 inode 长度 0，`blocks` = ⌈0 ÷ 512⌉ = 0）。
///
/// 这次发布是**一个事务**：D16（发布语义） 已定项 5 的切分纪律逐字「一个事务最多写一个单元的用户数据」，
/// 建 inode 一个用户数据单元都不写 ⇒ 不切；一个事务一条 journal 记录（D23（journal 的角色与格式） 已定项 7）。
/// 「N 个创建要不要各算一个事务」仓里没有条款，这一版按「不切」走（报告里交主 agent 定）。
///
/// # Errors
/// 记录落法要的条款没定（中间插入、容器数超过一个根装得下的）⇒ `InodeTreeWriteRefused`；
/// 其余与 `publish_overwrite` 相同。两样都在任何落盘动作之前返回，盘上逐字节不变。
/// 这次要点名的单元超过一条 journal 记录装得下的 64 项时不拒：这个事务的记录再跨几条，只有真正的最后一条带
/// 「本次发布末条」标志与提交标记（D23（journal 的角色与格式） 已定项 17 / 已定项 7）。
pub fn publish_new_inodes<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    new_inode_count: u64,
    write_time_seconds: u64,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    let first_new_inode_number = previous.inode_number_watermark();
    // 只供测试的开关装着时从压回去的那个号起发（C479）；记账里写的水位在 `publish_admitted` 里读同一个开关、同一份上一版。
    let first_new_inode_number = pool
        .inode_number_watermark_source()
        .watermark_this_publish_starts_from(first_new_inode_number);
    // 改动计数与事务号都先取成局部量：`crates/mutations.tsv` 有几条变异按 `change_count: txg.0,` 与
    // `transaction: previous.highest_transaction_number_in_this_instance + 1,` 这两串字面锚在 `publish_overwrite` 上，
    // 同一份文件里再出现一次它们就腐化（门禁 crates-mutation-replay 判锚点恰好命中一次；`publish_sequential_write` 里那句注释是同一件事）。
    // 改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88）；建出来那次就是这次发布。
    let change_count = txg.0;
    let transaction_number = previous.highest_transaction_number_in_this_instance + 1;
    let new_inode_records: Vec<InodeRecord> = (0..new_inode_count)
        .map(|offset_from_the_watermark| InodeRecord {
            inode: first_new_inode_number + offset_from_the_watermark,
            // 对象出生代 = 创建那次发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 8）。
            object_birth: txg,
            nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
            // 建出来还没写过数据：长度 0，blocks 跟着是 0。
            size: 0,
            change_count,
            write_time_seconds,
            // 没被截断过：截断字段是 0。
            truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
        })
        .collect();
    publish_version(
        pool,
        allocator,
        PublishPlan {
            txg,
            counter: previous.record.counter + 1,
            transaction: transaction_number,
            highest_transaction_number_before_this_publish: previous
                .highest_transaction_number_in_this_instance,
            instance,
            back_chain: back_chain_of(&previous.record_bytes),
            // 文件内容不动：数据单元与 extent 树根照抄上一版。
            file: None,
            new_inode_records: &new_inode_records,
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
    )
}

/// 按 inode 号覆盖写（里程碑「删除与多次 COW」一·1，用户 2026-10-18 定「先给 core 加多 inode 写数据」）：接在上一次发布之后把 `inode`
/// 那个文件整份换成 `content`，契约是一个数据单元——同 [`publish_overwrite`]，多个数据单元的内容走 [`publish_sequential_write_of_an_inode`]。
/// `inode` 是第一个文件（inode 1）时就是 [`publish_overwrite`]：同一条路，写出的字节一个不差。别的 inode 走
/// [`publish_sequential_write_of_an_inode`] 那一条，只多一道「装得进一个数据单元」。
///
/// # Errors
/// 同 [`publish_sequential_write_of_an_inode`]；内容装不进一个数据单元 ⇒ `ContentExceedsDataUnit`（在记录那两判之后、动分配器与盘之前，
/// 次序同 [`publish_overwrite`]）。
///
/// # Panics
/// 同 [`publish_sequential_write_of_an_inode`]。
pub fn publish_overwrite_of_an_inode<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    inode: InodeNumber,
    content: &[u8],
    write_time_seconds: u64,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    if inode.0 == FIRST_INODE_NUMBER {
        return publish_overwrite(
            pool,
            allocator,
            previous,
            FirstFile {
                content,
                write_time_seconds,
            },
            instance,
        );
    }
    let plan = plan_of_a_whole_write_of_a_file_other_than_the_first(
        previous,
        inode,
        content,
        write_time_seconds,
        instance,
    )?;
    publish_version_holding_one_data_unit(pool, allocator, plan, Some(previous))
}

/// 按 inode 号顺序写一次写请求（里程碑「删除与多次 COW」一·1，用户 2026-10-18 定）：从 `inode` 那个文件的偏移 0 写整份 `content`，
/// 按切分纪律切成 N 个一单元事务（`crate::write_request_split`，D16（发布语义） 已定项 5 末段），N 个事务在同一次发布里写出 N 个数据单元、
/// N 条记录——与 [`publish_sequential_write`] 同一张计划、同一条发布路径（`.claude/rules/fs-design.md`「一个事务层，所有结构共用」），
/// 只是写的对象换成 `inode`：数据单元头的五元组、extent 叶记录 key、inode 记录都写它（对象出生代照它那条记录、不改，改动计数取这次的 txg，
/// 截断字段清零——整份写从偏移 0 重写了全部单元，同第一个文件）。`inode` 是第一个文件（inode 1）时就是 [`publish_sequential_write`]。
///
/// 别的 inode：它上一版的每个数据单元与它下段的每个节点整份换下、经映射释放（被活快照引用的进 deadlist），第一个文件与别的文件的数据单元、
/// 下段照抄；extent 树上段按这次之后有条目的 inode 号定形状（D8（核心索引结构） 已定项 14），它那片叶到根那一串重写，那片叶里别的文件的条目
/// 照这一版内存里的指针重装（[`plan_the_extent_tree_after_writing_a_file_other_than_the_first`]）。
///
/// # Errors
/// 都在任何读写之前返回，盘上逐字节不变：`inode` 不在上一版的 inode 树里 ⇒ `WriteToAnInodeThatIsNotInTheTree`；上一版里第一个文件已经删完、
/// `inode` 的对象出生代晚于 inode 1 墓碑的死亡 txg ⇒ `DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion`（K-U5 的 β 读法）；
/// 它那条记录 nlink = 0（删除中，D8（核心索引结构） 已定项 16 第 4 条「写入口拒 nlink = 0 的 inode」）⇒ `WriteToAnInodeBeingDeleted`。
/// 其余同 [`publish_sequential_write`]（准入、落点、释放判定、块设备错）。
///
/// # Panics
/// `inode` 不是第一个文件而上一版的承诺类意图队列不空（`deleted_inodes`、销毁中的快照、截断条目）：会话里每一次用户改动先把队列走空再做它自己
/// （D8（核心索引结构） 已定项 16 第 6 条），挂载续做走空之后才交回；直接调这个函数的调用方同样先走空（同 [`publish_unlink_intent`]、
/// [`publish_size_change`] 的截断那一条）。
pub fn publish_sequential_write_of_an_inode<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    inode: InodeNumber,
    content: &[u8],
    write_time_seconds: u64,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    if inode.0 == FIRST_INODE_NUMBER {
        return publish_sequential_write(
            pool,
            allocator,
            previous,
            FirstFile {
                content,
                write_time_seconds,
            },
            instance,
        );
    }
    let plan = plan_of_a_whole_write_of_a_file_other_than_the_first(
        previous,
        inode,
        content,
        write_time_seconds,
        instance,
    )?;
    publish_version(pool, allocator, plan, Some(previous))
}

/// 按 inode 号整份写第一个文件之外 `inode` 那个文件的那一次发布的计划（[`publish_sequential_write_of_an_inode`] 与
/// [`publish_overwrite_of_an_inode`] 共用）：先判那条记录在不在树里、是不是删除中，再拼计划。只读上一版的内存态。
///
/// # Errors
/// `WriteToAnInodeThatIsNotInTheTree`、`DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion`、`WriteToAnInodeBeingDeleted`、
/// `NextCheckpointTxgPastTheTopOfItsRange`。
///
/// # Panics
/// 上一版的承诺类意图队列不空（见 [`publish_sequential_write_of_an_inode`]）。
fn plan_of_a_whole_write_of_a_file_other_than_the_first<'content>(
    previous: &TransactionOutput,
    inode: InodeNumber,
    content: &'content [u8],
    write_time_seconds: u64,
    instance: InstanceGeneration,
) -> Result<PublishPlan<'content>, PublishError> {
    let record = previous
        .inode_record_of(inode)
        .ok_or(PublishError::WriteToAnInodeThatIsNotInTheTree { inode: inode.0 })?;
    // K-U5 的 β 时序级读法（用户 2026-10-19 定）：第一个文件已经删完的一版上，目标的对象出生代晚于 inode 1 墓碑的死亡 txg 才拒——
    // 删完之后才建的 inode 不写数据，删之前就有的别的文件照常能写。读墓碑树（墓碑第一版不回收），只认这一版父链上的那一条（谱系树）。
    if let Some(death_txg_of_the_first_file) = previous.death_txg_of_the_deleted_first_file() {
        if record.object_birth > death_txg_of_the_first_file {
            return Err(
                PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion {
                    inode: inode.0,
                    object_birth: record.object_birth,
                    death_txg_of_the_first_file,
                },
            );
        }
    }
    let is_being_deleted = record.nlink == NLINK_OF_AN_INODE_BEING_DELETED;
    if is_being_deleted {
        return Err(PublishError::WriteToAnInodeBeingDeleted { inode: inode.0 });
    }
    assert!(
        previous.queue_of_committed_intents_is_empty(),
        "按 inode 号写之前承诺类意图队列已走空（D8（核心索引结构） 已定项 16 第 6 条：会话与挂载续做都先走空）：inode {}",
        inode.0
    );
    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    // 改动计数与这次请求的新池新建文件号先取成局部量：`crates/mutations.tsv` 有几条变异按 `publish_overwrite` 里那几串字面锚着，
    // 同一份文件里再出现一次它们就腐化（同 `publish_sequential_write` 那一句）。
    let change_count_of_this_write = txg.0;
    let new_pool_file_creation_of_this_request =
        previous.highest_transaction_number_in_this_instance + 1;
    Ok(PublishPlan {
        txg,
        counter: previous.record.counter + 1,
        transaction: new_pool_file_creation_of_this_request,
        highest_transaction_number_before_this_publish: previous
            .highest_transaction_number_in_this_instance,
        instance,
        back_chain: back_chain_of(&previous.record_bytes),
        file: Some(FileVersionPlan {
            inode,
            content,
            write_time_seconds,
            // 对象出生代不改（D8（核心索引结构） 已定项 6；I-9.10（对象出生代与 inode 记录相符））：建 inode 那次发布写下的。
            inode_object_birth: record.object_birth,
            change_count: change_count_of_this_write,
        }),
        // 写文件不建新 inode：只把 `inode` 那条记录换一版，它落在原来那一片叶里（容器身份不变）。
        new_inode_records: &[],
        instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
        tree_birth_txg: previous.tree_birth_txg(),
        // 这次一个树 ID 都不发：上一版的水位就是根环里全部根记录的 max。
        tree_identifier_watermark: previous.root.tree_identifier_watermark,
        rollback_floor: previous.root.rollback_floor,
    })
}

/// 一次用户改动里建的 inode：几个、写进它们记录的时间秒。几次建 inode 并进一次发布时个数相加、时间秒逐次保留在各自那几条记录里
/// （D16（发布语义） 已定项 10「建 inode 求和」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NewInodesOfOneChange {
    pub count: u64,
    pub write_time_seconds: u64,
}

/// 按 inode 号整份写一个文件：同一个 inode 的几次写并进一次发布时折叠成最后那一次（D16（发布语义） 已定项 10「同一 inode 的写折叠」）。
/// `inode` 是第一个文件（inode 1）时写的是 [`TransactionUnit::Data`] 那一族，同 [`publish_sequential_write`]；别的号同
/// [`publish_sequential_write_of_an_inode`]。按切分纪律切成几个一单元事务（不按单元数拒）。
#[derive(Clone, Copy, Debug)]
pub struct WholeWriteOfOneInode<'content> {
    pub inode: InodeNumber,
    pub content: &'content [u8],
    pub write_time_seconds: u64,
}

/// 并进一次发布的数据写类改动（D16（发布语义） 已定项 10「数据写类可并：同一 inode 的写折叠、建 inode 求和」；会话的待发排空时交进来，
/// `mounted_session::MountedSessionWithPendingChanges`）：至多一个文件的整份写（折叠过的），与按收下的次序的几次建 inode。
/// 一次发布写一个文件以内是这一版发布计划的形状（[`PublishPlan::file`] 只装一个文件），不同 inode 的写不在同一次发布里——会话在收下时拒
/// 第二个文件的写，不在这里拆。
///
/// 写的那个文件那条记录与新建的那几条一起写进 inode 树：写的那个号在上一版树里，新建的号从记账水位起发、都大于树里每一个 key，号严格升序。
/// 事务照切分纪律：文件内容切成 N 个一单元事务、N 条记录，新建 inode 的记录与这次发布共享的提交内生块在最后一个事务的记录里点名；
/// 没有文件写时整次发布一个事务（同 [`publish_new_inodes`]）。
#[derive(Clone, Copy, Debug)]
pub struct DataChangesPublishedTogether<'content> {
    pub whole_write: Option<WholeWriteOfOneInode<'content>>,
    pub new_inodes: &'content [NewInodesOfOneChange],
}

impl DataChangesPublishedTogether<'_> {
    /// 这一批里建的 inode 一共几个。
    ///
    /// # Panics
    /// 个数之和装不进 u64：每一次的个数都是一次用户改动交进来的，加起来过 u64 说明调用方交的不是一批真改动。
    #[must_use]
    pub fn new_inodes_in_total(&self) -> u64 {
        self.new_inodes
            .iter()
            .try_fold(0_u64, |total, change| total.checked_add(change.count))
            .expect("一批里建的 inode 个数之和装得进 u64")
    }
}

/// 这一批要新建的那几条 inode 记录，按收下的次序：号从这一版的记账水位起连着发（只供测试的水位开关装着时从压回去的那个号起，C479，
/// 与 [`publish_new_inodes`] 读同一个开关、同一份上一版），对象出生代与改动计数都是这次发布的 txg，时间秒是各自那一次改动交进来的。
fn new_inode_records_of_data_changes_published_together<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    previous: &TransactionOutput,
    batch: &DataChangesPublishedTogether<'_>,
    txg_of_the_batch: CheckpointTxg,
) -> Vec<InodeRecord> {
    let mut next_inode_number = pool
        .inode_number_watermark_source()
        .watermark_this_publish_starts_from(previous.inode_number_watermark());
    let mut records_of_the_batch = Vec::new();
    // 迭代上界是这一批的改动数乘各自的个数；跨轮携带的是下一个要发的号（只增）。
    for change in batch.new_inodes {
        for _created_in_this_change in 0..change.count {
            records_of_the_batch.push(InodeRecord {
                inode: next_inode_number,
                object_birth: txg_of_the_batch,
                nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
                size: 0,
                change_count: txg_of_the_batch.0,
                write_time_seconds: change.write_time_seconds,
                truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
            });
            next_inode_number = next_inode_number
                .checked_add(1)
                .expect("inode 号水位装得进 u64：号从记账水位起发，水位本身是这一版记账里读出的");
        }
    }
    records_of_the_batch
}

/// 这一批那一次发布的计划：接在 `previous` 后面，txg、jsn、事务号各接着走；写的那个文件照它是不是第一个文件分两路判（第一个文件要活着；
/// 别的号要在树里、不在删除中、不是第一个文件删完之后才建的，[`plan_of_a_whole_write_of_a_file_other_than_the_first`]），新建的记录由调用方
/// 照 [`new_inode_records_of_data_changes_published_together`] 先算好交进来。只读上一版的内存态。
///
/// # Errors
/// `WriteToAnInodeThatIsNotInTheTree`、`WriteToAnInodeBeingDeleted`、`DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion`、
/// `NextCheckpointTxgPastTheTopOfItsRange`，都在任何读写之前。
///
/// # Panics
/// 写的是第一个文件之外的号而上一版的承诺类意图队列不空（同 [`publish_sequential_write_of_an_inode`]：会话先走空）。
fn plan_of_data_changes_published_together<'content>(
    previous: &TransactionOutput,
    batch: &DataChangesPublishedTogether<'content>,
    new_inode_records: &'content [InodeRecord],
    instance: InstanceGeneration,
) -> Result<PublishPlan<'content>, PublishError> {
    let txg_of_the_batch = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    let file_of_the_batch = match batch.whole_write {
        None => None,
        Some(whole_write) if whole_write.inode.0 == FIRST_INODE_NUMBER => {
            let record_of_the_first_file = live_record_of_the_first_file(previous)?;
            Some(FileVersionPlan {
                inode: whole_write.inode,
                content: whole_write.content,
                write_time_seconds: whole_write.write_time_seconds,
                // 对象出生代不改（D8（核心索引结构） 已定项 6）；改动计数取这次发布的 txg。
                inode_object_birth: record_of_the_first_file.object_birth,
                change_count: txg_of_the_batch.0,
            })
        }
        Some(whole_write) => {
            plan_of_a_whole_write_of_a_file_other_than_the_first(
                previous,
                whole_write.inode,
                whole_write.content,
                whole_write.write_time_seconds,
                instance,
            )?
            .file
        }
    };
    let first_transaction_of_the_batch = previous.highest_transaction_number_in_this_instance + 1;
    Ok(PublishPlan {
        txg: txg_of_the_batch,
        counter: previous.record.counter + 1,
        transaction: first_transaction_of_the_batch,
        highest_transaction_number_before_this_publish: previous
            .highest_transaction_number_in_this_instance,
        instance,
        back_chain: back_chain_of(&previous.record_bytes),
        file: file_of_the_batch,
        new_inode_records,
        instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
        tree_birth_txg: previous.tree_birth_txg(),
        // 这次一个树 ID 都不发：上一版的水位就是根环里全部根记录的 max。
        tree_identifier_watermark: previous.root.tree_identifier_watermark,
        rollback_floor: previous.root.rollback_floor,
    })
}

/// 并在一起的数据写类改动发成一次发布（[`DataChangesPublishedTogether`]，会话的待发排空走这一条）：走 [`publish_version`] 那一条发布路径
/// （`.claude/rules/fs-design.md`「一个事务层，所有结构共用」：这里不另开一条）。只有一个文件写、没有建 inode 时写出的字节与同内容的
/// [`publish_sequential_write_of_an_inode`] 一个不差；只有建 inode、各次时间秒相同时与同个数的 [`publish_new_inodes`] 一个不差。
///
/// # Errors
/// 计划那几种（[`plan_of_data_changes_published_together`]），其余同 [`publish_version`]（冻结、准入、落点、释放判定、记录条数、块设备错）。
///
/// # Panics
/// 同 [`plan_of_data_changes_published_together`]。
pub fn publish_data_changes_published_together<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    batch: DataChangesPublishedTogether<'_>,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let txg_of_the_batch = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    let new_inode_records = new_inode_records_of_data_changes_published_together(
        pool,
        previous,
        &batch,
        txg_of_the_batch,
    );
    let plan =
        plan_of_data_changes_published_together(previous, &batch, &new_inode_records, instance)?;
    publish_version(pool, allocator, plan, Some(previous))
}

/// 并在一起的这一批要是此刻接在 `previous` 后面发，过不过得了落盘之前那几判、要写多少 journal 字节（D16（发布语义） 已定项 10「accept 进门先做这一批
/// 合起来的准入与条数预判，过不了就在任何读写之前拒」；已定项 5「脏量按待发改动要写的 journal 字节算」）：冻结着一次没重发的发布就拒，
/// 再照 [`publish_data_changes_published_together`] 拼同一份计划，走发布路径落盘之前、读盘之前那一段（[`judge_the_publish_before_any_read_or_write`]：
/// 这次的单元落不落得下、分配记录树的固定点、切成几条记录与在飞上限、合取 7、空间准入的式子）。分配器只读（固定点在它的拷贝上走，同发布路径），
/// 一个盘都不读、一个写都不发；`pool` 只读它的参数与开关。过了交回这一次要写的 journal 字节（记录条数 × 4096）。
///
/// 判的是「此刻接在 `previous` 后面」：会话里收下与排空之间不再有别的发布（收下与排空都拿着 `&mut` 会话，屏障类先排空），排空那一次与这里判的
/// 是同一份上一版与分配器，除非又一次 `accept` 为空间推了抬 F（那一次在推完之后把待发并上它重判过）；这里只判、不推抬 F（推是写）：被空间拒时
/// `mounted_session::MountedSessionWithPendingChanges::accept` 推抬 F 再调这里，会话排空那一次照旧推抬 F 再判。
///
/// # Errors
/// `PublishFrozenAfterAWriteFailureIsNotResentYet`；计划那几种；`SpaceAdmissionRefused`、`PlacementRefused`、`JournalRecordsOfThePublishExceedTheLimit`、
/// `EmptyClusterSegmentsBelowTheFloor` 与 [`judge_the_publish_before_any_read_or_write`] 别的几种。
///
/// # Panics
/// 同 [`plan_of_data_changes_published_together`]。
pub fn journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write<
    Device: BlockDevice,
>(
    pool: &PoolWriter<'_, Device>,
    allocator: &PoolAllocator,
    previous: &TransactionOutput,
    batch: DataChangesPublishedTogether<'_>,
    instance: InstanceGeneration,
) -> Result<JournalBytesToWrite, PublishError> {
    refuse_while_a_publish_is_frozen(allocator)?;
    let txg_of_the_batch = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    let new_inode_records = new_inode_records_of_data_changes_published_together(
        pool,
        previous,
        &batch,
        txg_of_the_batch,
    );
    let plan =
        plan_of_data_changes_published_together(previous, &batch, &new_inode_records, instance)?;
    let judged = judge_the_publish_before_any_read_or_write(
        pool,
        allocator,
        &plan,
        Some(previous),
        &previous.tree_identifiers,
        &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
        &[],
        &[],
    )?;
    Ok(JournalBytesToWrite::of_records(
        u64::try_from(judged.journal_records_of_the_publish).expect("一次发布的记录条数装得进 u64"),
    ))
}
