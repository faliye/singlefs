//! 按一次发布的种类算出下一条根。

use super::*;

impl IdealModel {
    /// 接在 `previous` 后面的一次发布写出的根（D16（发布语义） 已定项 6：txg + 1；jsn 接着全池计数器）。
    #[allow(
        clippy::too_many_arguments,
        reason = "一条根的每个身份字段各是一个参数，收成结构体只会多一层没人验的名字"
    )]
    pub(super) fn next_root(
        &self,
        previous: &ModelRoot,
        instance: ModelInstanceGeneration,
        checkpoint_txg: ModelCheckpointTxg,
        journal_counter: ModelJournalCounter,
        kind: ModelPublishKind,
        rollback_floor: ModelCheckpointTxg,
        new_file_content: Option<&[u8]>,
        instance_table_rows: Rc<Vec<ModelInstanceRow>>,
    ) -> ModelRoot {
        self.next_root_with_the_user_visible_trees(
            previous,
            instance,
            checkpoint_txg,
            journal_counter,
            kind,
            rollback_floor,
            new_file_content,
            instance_table_rows,
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem,
            InodesOtherThanTheFirstChangeOfThePublish::Carried,
        )
    }

    /// 同 [`IdealModel::next_root`]，另说这一版的用户可见树从哪来：管理员回退那次发布取回退目标那一版的。
    /// 中央映射树装的是这一版的条目，占槽上界要按换过之后的这一版算，所以换在算占槽之前做。
    #[allow(
        clippy::too_many_arguments,
        reason = "一条根的每个身份字段各是一个参数，收成结构体只会多一层没人验的名字"
    )]
    pub(super) fn next_root_with_the_user_visible_trees(
        &self,
        previous: &ModelRoot,
        instance: ModelInstanceGeneration,
        checkpoint_txg: ModelCheckpointTxg,
        journal_counter: ModelJournalCounter,
        kind: ModelPublishKind,
        rollback_floor: ModelCheckpointTxg,
        new_file_content: Option<&[u8]>,
        instance_table_rows: Rc<Vec<ModelInstanceRow>>,
        user_visible_trees: UserVisibleTreesOfTheNewVersion<'_>,
        inodes_other_than_the_first_change: InodesOtherThanTheFirstChangeOfThePublish<'_>,
    ) -> ModelRoot {
        let key = ModelRootKey {
            checkpoint_txg,
            instance,
        };
        let creates_inodes = matches!(
            kind,
            ModelPublishKind::NewInode
                | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
        );
        let writes_an_inode_other_than_the_first = matches!(
            kind,
            ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
        );
        let writes_an_inode_other_than_the_first_at_an_offset = matches!(
            kind,
            ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
        );
        assert_eq!(
            (
                creates_inodes,
                writes_an_inode_other_than_the_first,
                writes_an_inode_other_than_the_first_at_an_offset
            ),
            match inodes_other_than_the_first_change {
                InodesOtherThanTheFirstChangeOfThePublish::Carried => (false, false, false),
                InodesOtherThanTheFirstChangeOfThePublish::Created { .. } => (true, false, false),
                InodesOtherThanTheFirstChangeOfThePublish::WholeWrite { .. } => (false, true, false),
                InodesOtherThanTheFirstChangeOfThePublish::PartialWrite { .. } => {
                    (false, false, true)
                }
            },
            "建 inode 的两种恰好带建了几个、按 inode 号写那一种与按偏移写别的文件那一种恰好带写了谁、写成什么，别的发布照抄：{kind:?}"
        );
        // 这次之后第一个文件之外的 inode 与水位（回退那次下面再换成回退目标那一版的 inode）。
        let mut inodes_other_than_the_first = previous.inodes_other_than_the_first.as_ref().clone();
        let mut inode_number_watermark = match kind {
            // 第一个文件那次发掉 inode 1（它接在树表 0 条的一版后面，`answer_publish_first_file` 判过有树的一版必拒）。
            ModelPublishKind::FirstFileVersion => FIRST_INODE_NUMBER + 1,
            ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
            | ModelPublishKind::FinalDeletionBatch { .. }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => {
                previous.inode_number_watermark
            }
        };
        // 删第一个文件之外的一个号（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1）：意图那次它成了删除中的、
        // 还剩它全部的单元；不是最后一批的批从尾部放 144 个；最后一批删掉它那条记录。删第一个文件的那几步不动这张表。
        match kind {
            ModelPublishKind::UnlinkIntent {
                deleted: ModelDeletedInode::OtherThanTheFirst(inode),
            } => {
                let deleted = inodes_other_than_the_first
                    .get_mut(&inode)
                    .expect("意图点名的号是这一版树里第一个文件之外活着的号（`answer_unlink_intent_of_an_inode` 判过）");
                deleted.liveness = ModelInodeLiveness::BeingDeleted {
                    data_units_remaining: deleted.data_units_present(),
                };
            }
            ModelPublishKind::DeletionBatchKeepingDataUnits {
                deleted: ModelDeletedInode::OtherThanTheFirst(inode),
            } => {
                let deleted = inodes_other_than_the_first
                    .get_mut(&inode)
                    .expect("删除批点名的号是这一版删除中的那个（`answer_deletion_step` 判过）");
                let ModelInodeLiveness::BeingDeleted {
                    data_units_remaining,
                } = deleted.liveness
                else {
                    panic!("删除批点名的号是这一版删除中的那个（`answer_deletion_step` 判过）：{deleted:?}");
                };
                deleted.liveness = ModelInodeLiveness::BeingDeleted {
                    data_units_remaining: data_units_remaining - MODEL_DELETION_BATCH_DATA_UNITS,
                };
            }
            ModelPublishKind::FinalDeletionBatch {
                deleted: ModelDeletedInode::OtherThanTheFirst(inode),
                ..
            } => {
                inodes_other_than_the_first
                    .remove(&inode)
                    .expect("最后一批点名的号是这一版删除中的那个（`answer_deletion_step` 判过）");
            }
            ModelPublishKind::UnlinkIntent {
                deleted: ModelDeletedInode::FirstFile,
            }
            | ModelPublishKind::DeletionBatchKeepingDataUnits {
                deleted: ModelDeletedInode::FirstFile,
            }
            | ModelPublishKind::FinalDeletionBatch {
                deleted: ModelDeletedInode::FirstFile,
                ..
            }
            | ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            // 按偏移写：第一个文件的内容由读回的整份带进来，别的文件的内容由 `inodes_other_than_the_first_change` 带进来（下面）。
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. } => {}
            // 没有数据单元的 inode 改 size（判决 m4-multi-inode-size-r1 第二节 S2、S3）：读回 N 个 0；第一次写上粘住的标签 0 的那一次起它在上段里
            // 有条目；截断意图那次 logged_ops 里那一条截断条目点名它。
            ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits {
                inode,
                new_size_in_bytes,
                change,
                ..
            } => {
                let changed = inodes_other_than_the_first.get_mut(&inode).expect(
                    "改 size 点名的号是这一版树里第一个文件之外活着、没有数据单元的号（`answer_size_change_of_an_inode_other_than_the_first` 判过）",
                );
                changed.content = Rc::from(vec![
                    0u8;
                    usize::try_from(new_size_in_bytes)
                        .expect("答法判过 N 装得进这个进程的地址空间")
                ]);
                match change {
                    ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingWritingTheTagZeroEntry => {
                        changed.keeps_a_tag_zero_upper_leaf_entry = true;
                    }
                    ModelSizeChangeOfAnInodeWithoutDataUnits::TruncationIntent => {
                        changed.truncation_issued_txg = Some(checkpoint_txg);
                    }
                    ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingCarryingTheExtentTree => {}
                }
            }
            ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { inode, .. } => {
                inodes_other_than_the_first
                    .get_mut(&inode)
                    .expect("截断的最后一批点名的号是这一版截断条目点名的那个（`answer_truncation_step` 判过）")
                    .truncation_issued_txg = None;
            }
            // 带数据单元的 inode 改 size（判决 m4-multi-inode-size-r1 第二节 S3、S5）：读回截到 N、或补 0 到 N 的内容；截断意图那次 logged_ops 里那一条
            // 截断条目点名它，它那条记录的截断字段按这次放单元之前存在的单元数写或照抄。单元与下段在下面按单元号改。
            ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                inode,
                new_size_in_bytes,
                direction,
                ..
            } => {
                let changed = inodes_other_than_the_first.get_mut(&inode).expect(
                    "改 size 点名的号是这一版树里第一个文件之外活着、带数据单元的号（`answer_size_change_of_an_inode_other_than_the_first` 判过）",
                );
                let mut content = changed.content.to_vec();
                content.resize(
                    usize::try_from(new_size_in_bytes)
                        .expect("答法判过 N 装得进这个进程的地址空间"),
                    0,
                );
                changed.content = Rc::from(content);
                match direction {
                    ModelDirectionOfASizeChange::TruncationIntent => {
                        changed.truncation_field = ModelTruncationField::after_a_truncation_intent(
                            changed.truncation_field,
                            changed.data_units_present(),
                            new_size_in_bytes,
                            checkpoint_txg,
                        );
                        changed.truncation_issued_txg = Some(checkpoint_txg);
                    }
                    ModelDirectionOfASizeChange::WithoutShrinking => {}
                }
            }
            // 它截断的批不碰记录：内容与截断条目照抄（单元与下段在下面按单元号改）。
            ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. } => {}
            ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { inode, .. } => {
                inodes_other_than_the_first
                    .get_mut(&inode)
                    .expect("截断的最后一批点名的号是这一版截断条目点名的那个（`answer_truncation_step` 判过）")
                    .truncation_issued_txg = None;
            }
        }
        // 这一版父链上 inode 1 的死亡 txg：删第一个文件的意图那次写成这次的；别的照抄上一版的，管理员回退那次下面再换成回退目标那一版的。
        let death_txg_of_the_first_file_unless_rolled_back = match kind {
            ModelPublishKind::UnlinkIntent {
                deleted: ModelDeletedInode::FirstFile,
            } => Some(checkpoint_txg),
            ModelPublishKind::UnlinkIntent {
                deleted: ModelDeletedInode::OtherThanTheFirst(_),
            }
            | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
            | ModelPublishKind::FinalDeletionBatch { .. }
            | ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => {
                previous.death_txg_of_the_first_file
            }
        };
        // 管理员回退那次的父是回退目标（谱系记录 (T_rb, T_old)）：父链上 inode 1 的死亡 txg 取回退目标那一版的，现行那一版在回退目标之后留下的
        // 墓碑落在被抛下的那一段里（β 只认这一版父链上的墓碑，代码轮判决 m3-close-w5-code-r1 第三节第 1 条）。
        let death_txg_of_the_first_file = match user_visible_trees {
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem => {
                death_txg_of_the_first_file_unless_rolled_back
            }
            UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(target_root) => {
                target_root.death_txg_of_the_first_file
            }
        };
        match inodes_other_than_the_first_change {
            InodesOtherThanTheFirstChangeOfThePublish::Carried => {}
            InodesOtherThanTheFirstChangeOfThePublish::Created { count } => {
                for new_inode in inode_number_watermark..inode_number_watermark + count {
                    inodes_other_than_the_first.insert(
                        new_inode,
                        ModelInodeOtherThanTheFirst {
                            content: Rc::from(Vec::new()),
                            data_unit_written_at: Rc::new(Vec::new()),
                            extent_lower_node_written_at: Rc::new(BTreeMap::new()),
                            truncation_field: ModelTruncationField::NotWritten,
                            object_birth: checkpoint_txg,
                            liveness: ModelInodeLiveness::Live,
                            keeps_a_tag_zero_upper_leaf_entry: false,
                            truncation_issued_txg: None,
                        },
                    );
                }
                inode_number_watermark += count;
            }
            InodesOtherThanTheFirstChangeOfThePublish::WholeWrite { inode, content } => {
                let written = inodes_other_than_the_first.get_mut(&inode).expect(
                    "按 inode 号写只点名这一版树里第一个文件之外活着的号（`answer_whole_write_of_an_inode` 判过）",
                );
                written.content = Rc::from(content);
                // 整份写清零（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R2 推到任意 inode：`transaction::inode_record_of_file_version`）；
                // 单元与下段在下面按单元号改（全是这次的）。
                written.truncation_field = ModelTruncationField::NotWritten;
            }
            // 按偏移写（判决 m4-partial-write-r1 第二节 R5）：截断字段照抄（T、N_w 部分写不碰）；单元与下段在下面按单元号改（重写的那一段是这次的）。
            InodesOtherThanTheFirstChangeOfThePublish::PartialWrite {
                inode,
                content_after,
            } => {
                inodes_other_than_the_first
                    .get_mut(&inode)
                    .expect(
                        "按偏移写只点名这一版树里第一个文件之外活着的号（`answer_partial_write` 判过）",
                    )
                    .content = Rc::from(content_after);
            }
        }
        let data_units_written_to_the_file_other_than_the_first =
            match inodes_other_than_the_first_change {
                InodesOtherThanTheFirstChangeOfThePublish::WholeWrite { content, .. } => {
                    data_units_of_content(u64::try_from(content.len()).expect("内容长度"))
                }
                // 按偏移写的单元与下段节点在下面按单元号数占槽（同改 size）。
                InodesOtherThanTheFirstChangeOfThePublish::Carried
                | InodesOtherThanTheFirstChangeOfThePublish::Created { .. }
                | InodesOtherThanTheFirstChangeOfThePublish::PartialWrite { .. } => 0,
            };
        // 第一个文件之外那个文件这次的单元与下段（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3、S5，K4 推到任意 inode）：
        // 整份写、改 size、截断的批、unlink 的批与最后一批各按它自己上一版的单元与下段算（同第一个文件那一份，
        // [`IdealModel::data_units_and_lower_nodes_after_a_data_unit_change`]），换下的各按出生 txg 交给快照那一段判钉不钉。改 size 与截断那几次写的
        // 单元与下段节点在这里数占槽（整份写与 unlink 的批照旧在下面按形状数）。
        let mut replaced_units_besides_the_first_file: Vec<ModelReplacedUnit> = Vec::new();
        let mut slots_written_for_the_size_of_an_inode_other_than_the_first = 0;
        if let Some((changed_inode, change)) =
            Self::data_unit_change_of_an_inode_other_than_the_first(
                previous,
                kind,
                inodes_other_than_the_first_change,
            )
        {
            let before = previous
                .inodes_other_than_the_first
                .get(&changed_inode)
                .expect("这次动的号是上一版树里第一个文件之外的号（答法判过）");
            let (lower_nodes_after, replaced) =
                Self::data_units_and_lower_nodes_after_a_data_unit_change(
                    &before.data_unit_written_at,
                    &before.extent_lower_node_written_at,
                    &change,
                    checkpoint_txg,
                );
            if matches!(
                kind,
                ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
                    | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
                    | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
                    | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            ) {
                let lower_nodes_written = lower_nodes_after
                    .values()
                    .filter(|written_at| **written_at == checkpoint_txg)
                    .count();
                slots_written_for_the_size_of_an_inode_other_than_the_first =
                    (change.rewritten.end - change.rewritten.start)
                        * ModelUnitRole::Data.span_in_slots()
                        + u64::try_from(lower_nodes_written).expect("节点数")
                            * (NODE_BYTES / SLOT_BYTES);
            }
            replaced_units_besides_the_first_file = replaced;
            if let Some(after) = inodes_other_than_the_first.get_mut(&changed_inode) {
                after.data_unit_written_at = Rc::new(
                    change.data_unit_written_at_after(&before.data_unit_written_at, checkpoint_txg),
                );
                after.extent_lower_node_written_at = Rc::new(lower_nodes_after);
            }
        }
        let mut role_written_at = previous.role_written_at.clone();
        for role in kind.removed_roles() {
            role_written_at.remove(role);
        }
        let data_unit_change =
            Self::data_unit_change_of_the_publish(previous, kind, new_file_content);
        // 上一版删除中的那个号（第一个文件或别的文件）还剩几个单元：不是最后一批的删除批截的是它的下段。
        let data_units_remaining_before_this_publish = previous
            .inode_being_deleted()
            .map(|(_, data_units_remaining)| data_units_remaining);
        let rewritten = kind.rewritten_roles();
        // 数据单元按单元号另记（`data_unit_written_at`），不进按角色记的这张表。
        for role in rewritten
            .iter()
            .filter(|role| **role != ModelUnitRole::Data)
        {
            role_written_at.insert(*role, checkpoint_txg);
        }
        let rewrites_the_extent_upper_root = rewritten.contains(&ModelUnitRole::ExtentRoot);
        // 写上段叶条目的发布按这次之后存在的单元数与 size 写标签，不写的照抄（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5）。
        let extent_upper_leaf_entry_label_after =
            |previous_file: Option<&ModelFileVersion>, size_in_bytes: usize| match (
                rewrites_the_extent_upper_root,
                previous_file,
            ) {
                (true, Some(_) | None) => ModelExtentUpperLeafEntryLabel::written_for(
                    data_unit_change.data_units_after,
                    data_units_of_content(u64::try_from(size_in_bytes).expect("内容长度")),
                ),
                (false, Some(previous_file)) => previous_file.extent_upper_leaf_entry_label,
                (false, None) => {
                    panic!("不重写 extent 上段根的发布接在带文件的一版后面，照抄它的标签：{kind:?}")
                }
            };
        let mut contents = match (kind, new_file_content) {
            (
                ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion,
                Some(content),
            ) => ModelVersionContents::LiveFile(ModelFileVersion {
                written_by: key,
                inode_record_written_by: key,
                content: Rc::from(content),
                data_units_present: data_unit_change.data_units_after,
                extent_upper_leaf_entry_label: extent_upper_leaf_entry_label_after(
                    None,
                    content.len(),
                ),
                // 整份写清零（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R2）。
                truncation_field: ModelTruncationField::NotWritten,
            }),
            // 截断意图（K2「先落目标」）：size 写成 N、读回 N 字节，单元一个不放；只在带活文件的一版上发（`answer_truncation` 判过）。
            // 截断字段按这次放单元之前存在的单元数写或照抄。
            (ModelPublishKind::TruncationIntent { .. }, Some(content)) => {
                let ModelVersionContents::LiveFile(previous_file) = &previous.contents else {
                    panic!(
                        "截断意图只接在带活文件的一版后面（`answer_truncation` 判过）：{:?}",
                        previous.contents
                    );
                };
                ModelVersionContents::FileBeingTruncated {
                    file: ModelFileVersion {
                        written_by: key,
                        inode_record_written_by: key,
                        content: Rc::from(content),
                        data_units_present: data_unit_change.data_units_after,
                        extent_upper_leaf_entry_label: extent_upper_leaf_entry_label_after(
                            Some(previous_file),
                            content.len(),
                        ),
                        truncation_field: ModelTruncationField::after_a_truncation_intent(
                            previous_file.truncation_field,
                            previous_file.data_units_present,
                            u64::try_from(content.len()).expect("内容长度"),
                            checkpoint_txg,
                        ),
                    },
                    truncation_issued_txg: checkpoint_txg,
                }
            }
            // 截断的批（K2）：不碰 inode 记录，内容与截断条目照抄，从尾部放 144 个。
            (ModelPublishKind::TruncationBatch, None) => {
                let ModelVersionContents::FileBeingTruncated {
                    file: previous_file,
                    truncation_issued_txg,
                } = &previous.contents
                else {
                    panic!(
                        "截断的批只接在截断途中的那一版后面（`answer_truncation_step` 判过）：{:?}",
                        previous.contents
                    );
                };
                ModelVersionContents::FileBeingTruncated {
                    file: ModelFileVersion {
                        written_by: key,
                        inode_record_written_by: previous_file.inode_record_written_by,
                        content: Rc::clone(&previous_file.content),
                        data_units_present: data_unit_change.data_units_after,
                        extent_upper_leaf_entry_label: extent_upper_leaf_entry_label_after(
                            Some(previous_file),
                            previous_file.content.len(),
                        ),
                        truncation_field: previous_file.truncation_field,
                    },
                    truncation_issued_txg: *truncation_issued_txg,
                }
            }
            // 截断的最后一批（K2、K11）：放完 size 之外的单元、删截断条目，之后是带活文件的一版；放 0 个时 extent 树不动、用户可见树的根指针照抄。
            (ModelPublishKind::FinalTruncationBatch { release, .. }, None) => {
                let ModelVersionContents::FileBeingTruncated {
                    file: previous_file,
                    ..
                } = &previous.contents
                else {
                    panic!(
                        "截断的最后一批只接在截断途中的那一版后面（`answer_truncation_step` 判过）：{:?}",
                        previous.contents
                    );
                };
                ModelVersionContents::LiveFile(ModelFileVersion {
                    written_by: match release {
                        ModelFinalTruncationBatchRelease::AtLeastOneDataUnit => key,
                        ModelFinalTruncationBatchRelease::NoDataUnit => previous_file.written_by,
                    },
                    inode_record_written_by: previous_file.inode_record_written_by,
                    content: Rc::clone(&previous_file.content),
                    data_units_present: data_unit_change.data_units_after,
                    extent_upper_leaf_entry_label: extent_upper_leaf_entry_label_after(
                        Some(previous_file),
                        previous_file.content.len(),
                    ),
                    truncation_field: previous_file.truncation_field,
                })
            }
            // 不缩短的截断（K5、K12）：size 写成 N、读回补 0 到 N 的内容，单元一个不放、不新增。
            (ModelPublishKind::SizeChangeWithoutShrinking { .. }, Some(content)) => {
                let ModelVersionContents::LiveFile(previous_file) = &previous.contents else {
                    panic!(
                        "不缩短的截断只接在带活文件的一版后面（`answer_truncation` 判过）：{:?}",
                        previous.contents
                    );
                };
                ModelVersionContents::LiveFile(ModelFileVersion {
                    written_by: key,
                    inode_record_written_by: key,
                    content: Rc::from(content),
                    data_units_present: data_unit_change.data_units_after,
                    extent_upper_leaf_entry_label: extent_upper_leaf_entry_label_after(
                        Some(previous_file),
                        content.len(),
                    ),
                    // 变长与 N = S 照抄（判决 m4-trunc-r2 第三节 R2）。
                    truncation_field: previous_file.truncation_field,
                })
            }
            // 按偏移写第一个文件（判决 m4-partial-write-r1 第二节 R4、R5）：读回按偏移盖过、空隙读 0 的整份，size 是写之后的，单元数是写之后的，
            // 标签按写之后的单元数与 size 换（重写了上段根），截断字段照抄；只接在带活文件的一版后面（`answer_partial_write` 判过）。
            (ModelPublishKind::PartialWriteOfTheFirstFile { .. }, Some(content)) => {
                let ModelVersionContents::LiveFile(previous_file) = &previous.contents else {
                    panic!(
                        "按偏移写第一个文件只接在带活文件的一版后面（`answer_partial_write` 判过）：{:?}",
                        previous.contents
                    );
                };
                ModelVersionContents::LiveFile(ModelFileVersion {
                    written_by: key,
                    inode_record_written_by: key,
                    content: Rc::from(content),
                    data_units_present: data_unit_change.data_units_after,
                    extent_upper_leaf_entry_label: extent_upper_leaf_entry_label_after(
                        Some(previous_file),
                        content.len(),
                    ),
                    truncation_field: previous_file.truncation_field,
                })
            }
            // unlink 的意图与各批照抄截断字段（判决 m4-trunc-r2 第三节 R2）：意图只接在带活文件的一版后面（`answer_unlink_intent` 判过），
            // 各批只接在删除中的那一版后面（`answer_deletion_step` 判过）。
            (
                ModelPublishKind::UnlinkIntent {
                    deleted: ModelDeletedInode::FirstFile,
                },
                None,
            ) => {
                let truncation_field_of_the_unlinked_record = previous
                    .contents
                    .live_file()
                    .expect("unlink 的意图只接在带活文件的一版后面（`answer_unlink_intent` 判过）")
                    .truncation_field;
                ModelVersionContents::FileBeingDeleted {
                    trees_written_by: key,
                    data_units_remaining: data_unit_change.data_units_after,
                    truncation_field: truncation_field_of_the_unlinked_record,
                }
            }
            (
                ModelPublishKind::DeletionBatchKeepingDataUnits {
                    deleted: ModelDeletedInode::FirstFile,
                },
                None,
            ) => {
                let ModelVersionContents::FileBeingDeleted {
                    truncation_field, ..
                } = previous.contents
                else {
                    panic!(
                        "不是最后一批的删除批只接在删除中的那一版后面（`answer_deletion_step` 判过）：{:?}",
                        previous.contents
                    );
                };
                ModelVersionContents::FileBeingDeleted {
                    trees_written_by: key,
                    data_units_remaining: data_units_remaining_before_this_publish
                        .expect("不是最后一批的删除批只接在删除中的那一版后面（`answer_deletion_step` 判过）")
                        - MODEL_DELETION_BATCH_DATA_UNITS,
                    truncation_field,
                }
            }
            // 删第一个文件的最后一批：树里还有别的 inode 时是带文件的一版（D8（核心索引结构） 已定项 16 第 5 条：不管 inode 1 在不在），
            // 没有了才是有树而文件集合为空的一版。
            (
                ModelPublishKind::FinalDeletionBatch {
                    deleted: ModelDeletedInode::FirstFile,
                    ..
                },
                None,
            ) => Self::contents_without_the_first_file(
                key,
                u64::try_from(inodes_other_than_the_first.len()).expect("条数"),
            ),
            // 删第一个文件之外的号的意图、批与最后一批：第一个文件（活着或已删完）照旧，inode 树换了一版（这次写的），批与最后一批连 extent 树。
            (
                ModelPublishKind::UnlinkIntent {
                    deleted: ModelDeletedInode::OtherThanTheFirst(_),
                }
                | ModelPublishKind::DeletionBatchKeepingDataUnits {
                    deleted: ModelDeletedInode::OtherThanTheFirst(_),
                }
                | ModelPublishKind::FinalDeletionBatch {
                    deleted: ModelDeletedInode::OtherThanTheFirst(_),
                    ..
                },
                None,
            ) => Self::contents_whose_trees_a_change_of_another_inode_rewrote(
                &previous.contents,
                key,
                u64::try_from(inodes_other_than_the_first.len()).expect("条数"),
                kind,
            ),
            // 建 inode：第一个文件活着时文件照旧，inode 树换了一版（这次写的），用户可见树的写者换成这次；删完之后建的数着树里几条。
            (
                ModelPublishKind::NewInode
                | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_),
                None,
            ) => match &previous.contents {
                ModelVersionContents::LiveFile(previous_file) => {
                    ModelVersionContents::LiveFile(Self::file_version_whose_trees_this_publish_rewrote(
                        previous_file,
                        key,
                    ))
                }
                ModelVersionContents::WithoutTrees
                | ModelVersionContents::FileBeingDeleted { .. }
                | ModelVersionContents::TreesAndNoFile { .. }
                | ModelVersionContents::InodesWithoutTheFirstFile { .. }
                | ModelVersionContents::FileBeingTruncated { .. } => {
                    let created = match inodes_other_than_the_first_change {
                        InodesOtherThanTheFirstChangeOfThePublish::Created { count } => count,
                        InodesOtherThanTheFirstChangeOfThePublish::Carried
                        | InodesOtherThanTheFirstChangeOfThePublish::WholeWrite { .. }
                        | InodesOtherThanTheFirstChangeOfThePublish::PartialWrite { .. } => {
                            unreachable!("建 inode 的两种恰好带建了几个（上面断言过）")
                        }
                    };
                    ModelVersionContents::InodesWithoutTheFirstFile {
                        trees_written_by: key,
                        inode_tree_written_by: key,
                        inode_records_in_the_tree: previous
                            .contents
                            .inode_records_created_after_the_first_file_was_deleted()
                            .expect("建 inode 只接在第一个文件活着或已删完的一版后面（`answer_create_inodes` 判过）")
                            + created,
                    }
                }
            },
            // 按 inode 号写第一个文件之外的文件：第一个文件的内容、单元、标签照旧，inode 树与 extent 树换了一版（这次写的）。
            // 第一个文件已删完时只写删之前就有的 inode（K-U5 β，`answer_whole_write_of_an_inode` 判过）。
            (ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }, None)
            // 按偏移写第一个文件之外的文件：同按 inode 号写，第一个文件照旧，inode 树与 extent 树换了一版（这次写的）。
            | (ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }, None)
            // 没有数据单元的 inode 改 size（判决 m4-multi-inode-size-r1 第二节 S3）：同按 inode 号写，第一个文件照旧，inode 树换了一版（这次写的）。
            | (ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }, None)
            // 带数据单元的那种改 size：inode 树换了一版（记录的 size），尾单元或标签变了时连 extent 树。
            | (ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }, None) => {
                Self::contents_whose_trees_a_change_of_another_inode_rewrote(
                    &previous.contents,
                    key,
                    u64::try_from(inodes_other_than_the_first.len()).expect("条数"),
                    kind,
                )
            }
            // 带数据单元的那种 inode 截断的批与放了单元的最后一批只写 extent 树（K2「批不碰 inode 记录」）：用户可见树里只有 extent 树的根指针换成这次写的。
            (
                ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
                | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                    release: ModelFinalTruncationBatchRelease::AtLeastOneDataUnit,
                    ..
                },
                None,
            ) => Self::contents_whose_extent_tree_a_truncation_batch_of_another_inode_rewrote(
                &previous.contents,
                key,
                kind,
            ),
            // 放 0 个单元的最后一批不碰 inode 树、不碰 extent 树。
            (
                ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                    release: ModelFinalTruncationBatchRelease::NoDataUnit,
                    ..
                },
                None,
            ) => previous.contents.clone(),
            // 它截断的最后一批不碰 inode 树、不碰 extent 树：用户可见树的根指针照抄。
            (ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. }, None) => {
                previous.contents.clone()
            }
            (
                ModelPublishKind::RowsOnFileVersion
                | ModelPublishKind::EmptyOnFileVersion
                | ModelPublishKind::RowsOnVersionWithoutFile
                | ModelPublishKind::ZeroUnit,
                None,
            ) => previous.contents.clone(),
            // 建快照与删快照的几次不动文件内容、extent 树与 inode 树（判决 S2、S4）。
            (
                ModelPublishKind::SnapshotCreation
                | ModelPublishKind::SnapshotDestructionIntent { .. }
                | ModelPublishKind::SnapshotDestructionBatch { .. }
                | ModelPublishKind::SnapshotDestructionFinalBatch { .. }

                | ModelPublishKind::CompactionBatch { .. }
                | ModelPublishKind::CompactionIntentIssue
                | ModelPublishKind::CompactionIntentDeletion { .. },
                None,
            ) => previous.contents.clone(),
            (
                ModelPublishKind::FirstFileVersion
                | ModelPublishKind::OverwriteFileVersion
                | ModelPublishKind::TruncationIntent { .. }
                | ModelPublishKind::SizeChangeWithoutShrinking { .. }
                | ModelPublishKind::PartialWriteOfTheFirstFile { .. },
                None,
            )
            | (
                ModelPublishKind::TruncationBatch | ModelPublishKind::FinalTruncationBatch { .. },
                Some(_),
            )
            | (
                ModelPublishKind::RowsOnFileVersion
                | ModelPublishKind::EmptyOnFileVersion
                | ModelPublishKind::RowsOnVersionWithoutFile
                | ModelPublishKind::ZeroUnit
                | ModelPublishKind::UnlinkIntent { .. }
                | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
                | ModelPublishKind::FinalDeletionBatch { .. }
                | ModelPublishKind::NewInode
                | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
                | ModelPublishKind::SnapshotCreation
                | ModelPublishKind::SnapshotDestructionIntent { .. }
                | ModelPublishKind::SnapshotDestructionBatch { .. }
                | ModelPublishKind::SnapshotDestructionFinalBatch { .. }

                | ModelPublishKind::CompactionBatch { .. }
                | ModelPublishKind::CompactionIntentIssue
                | ModelPublishKind::CompactionIntentDeletion { .. }
                // 按 inode 号写的内容走 `InodesOtherThanTheFirstChangeOfThePublish::WholeWrite`，这个参数说的是第一个文件。
                | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
                // 没有数据单元的 inode 改成的 size 在种类里带着，这个参数说的是第一个文件。
                | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
                | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
                | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
                | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
                | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. }
                // 按偏移写别的文件的内容走 `InodesOtherThanTheFirstChangeOfThePublish::PartialWrite`，这个参数说的是第一个文件。
                | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. },
                Some(_),
            ) => panic!(
                "带内容的只有写文件内容的两种、截断意图、不缩短的截断与按偏移写第一个文件（读回的整份），别的都不带：{kind:?}"
            ),
        };
        // extent 树上段（规范形，判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）：按这次之后有条目的号集合与
        // 这次条目变了的号，模型自己的计划（[`extent_upper_node_written_at_after`]）定每个节点这次重写还是照抄；根与根之下的每个节点各是一个角色。
        // 种类定死的角色里有没有上段根，与计划说根这次重写没有，要一致。树表 0 条的一版没有上段。回退那次下面整组换成目标那一版的。
        let inodes_with_an_entry_after =
            inodes_with_an_extent_upper_leaf_entry(&contents, &inodes_other_than_the_first);
        let inodes_whose_entry_changed =
            Self::inodes_whose_extent_upper_leaf_entry_the_publish_changes(
                kind,
                inodes_other_than_the_first_change,
                &previous.inodes_with_an_extent_upper_leaf_entry(),
            );
        role_written_at.retain(|role, _| !role.is_a_node_of_the_extent_upper_segment());
        if contents.has_trees() {
            let root_level_after = extent_upper_segment_root_level(&inodes_with_an_entry_after);
            for (position, written_at) in extent_upper_node_written_at_after(
                &previous.extent_upper_node_written_at(),
                &inodes_with_an_entry_after,
                &inodes_whose_entry_changed,
                checkpoint_txg,
            ) {
                let role = if position.level == root_level_after {
                    ModelUnitRole::ExtentRoot
                } else {
                    ModelUnitRole::ExtentUpperNodeBelowTheRoot(position)
                };
                role_written_at.insert(role, written_at);
            }
            assert_eq!(
                role_written_at.get(&ModelUnitRole::ExtentRoot) == Some(&checkpoint_txg),
                rewrites_the_extent_upper_root,
                "种类定死的角色里有没有 extent 上段根，与规范形计划说根这次重写没有，要一致：{kind:?}，有条目的号 {inodes_with_an_entry_after:?}，变了的 {inodes_whose_entry_changed:?}"
            );
        }
        // extent 上段换下的节点按位置认（根与根之下的一样，判决 m4-snapshot-r1 S3：上段节点问快照生死判定）：上一版的一个位置，这一版没有它、或它这次
        // 重写了。长高那次旧根照抄成根之下的节点、降层那次根之下的节点照抄成根，都不算换下。
        let extent_upper_nodes_after = ModelRoot::extent_upper_node_written_at_of(
            &role_written_at,
            &inodes_with_an_entry_after,
        );
        let extent_upper_nodes_replaced: Vec<ModelReplacedUnit> = previous
            .extent_upper_node_written_at()
            .into_iter()
            .filter(|(position, written_at)| {
                extent_upper_nodes_after.get(position) != Some(written_at)
            })
            .map(|(_, written_at)| ModelReplacedUnit {
                mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_INDEX_NODE,
                mapping_key_birth_tree: TREE_IDENTIFIER_EXTENT,
                birth_txg: written_at,
            })
            .collect();
        let mut data_unit_written_at = data_unit_change
            .data_unit_written_at_after(&previous.data_unit_written_at, checkpoint_txg);
        let mut rollback_rewrites_the_logged_operations_root = false;
        match user_visible_trees {
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem => {}
            UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(target_root) => {
                contents.clone_from(&target_root.contents);
                data_unit_written_at.clone_from(&target_root.data_unit_written_at);
                // 第一个文件之外的 inode 随 inode 树、extent 树取目标那一版的（记录、数据单元、下段都是那一版的）；水位不取目标的（照现行那一版）。
                inodes_other_than_the_first
                    .clone_from(target_root.inodes_other_than_the_first.as_ref());
                // 回退时 logged_ops 按种类分来源（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1）：截断条目取回退目标那一版的
                // （随上面换过的 `contents`），整理意图随现行那一版。截断条目与现行那一版的不同时 logged_ops 这次重写，删空了取根指针置零。
                let truncation_entry_of_the_target = target_root.logged_truncation_issued_txg();
                if truncation_entry_of_the_target != previous.logged_truncation_issued_txg() {
                    if truncation_entry_of_the_target.is_some()
                        || previous.logged_compaction_intent.is_some()
                    {
                        role_written_at.insert(ModelUnitRole::LoggedOperationsRoot, checkpoint_txg);
                        rollback_rewrites_the_logged_operations_root = true;
                    } else {
                        role_written_at.remove(&ModelUnitRole::LoggedOperationsRoot);
                    }
                }
                // 新根的 inode 树、extent 树与 `deleted_inodes` 取回退目标那一版的（D23（journal 的角色与格式） 已定项 14）：
                // 目标那一版有的角色取它的分配代，没有的（删完的一版没有叶容器与 `deleted_inodes` 根）这一版也没有；数据单元整组取目标那一版的
                // （上面）；墓碑树与退役树随现行那一版。
                // extent 上段的根与根之下的节点同样整组取目标那一版的。
                role_written_at.retain(|role, _| !role.is_a_node_of_the_extent_upper_segment());
                for (role, written_at) in target_root
                    .role_written_at
                    .iter()
                    .filter(|(role, _)| role.is_a_node_of_the_extent_upper_segment())
                {
                    role_written_at.insert(*role, *written_at);
                }
                for role in [
                    ModelUnitRole::InodeLeaf,
                    ModelUnitRole::InodeRoot,
                    ModelUnitRole::DeletedInodesRoot,
                ] {
                    match target_root.role_written_at.get(&role) {
                        Some(written_at) => {
                            role_written_at.insert(role, *written_at);
                        }
                        None => {
                            role_written_at.remove(&role);
                        }
                    }
                }
                // 最左那片之外的叶容器同样整组取目标那一版的：现行那一版有、目标那一版没有的那几片这一版也没有。
                role_written_at.retain(|role, _| {
                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))
                });
                role_written_at.extend(
                    target_root
                        .role_written_at
                        .iter()
                        .filter(|(role, _)| {
                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))
                        })
                        .map(|(role, written_at)| (*role, *written_at)),
                );
            }
        }
        assert_eq!(
            u64::try_from(data_unit_written_at.len()).expect("单元数"),
            contents.data_units_present(),
            "按单元号记的分配代与这一版存在的单元数一一对应：{kind:?}"
        );
        if let Some(file) = contents.live_file() {
            assert!(
                file.extent_upper_leaf_entry_label.admits(
                    file.data_units_present,
                    file.data_units_within_the_size(),
                    contents.logged_truncation_issued_txg().is_some(),
                ),
                "上段叶条目的标签 {:?} 与存在的 {} 个单元、size {} 对得上（判决第三节 K5、K7 的 T-3）：{kind:?}",
                file.extent_upper_leaf_entry_label,
                file.data_units_present,
                file.content.len()
            );
        }
        let snapshot_bookkeeping = Self::snapshot_bookkeeping_of_the_publish(
            previous,
            kind,
            checkpoint_txg,
            user_visible_trees,
            &data_unit_change,
            extent_upper_nodes_replaced,
            replaced_units_besides_the_first_file,
        );
        // 记账行：写记账树的发布照抄上一版的台阶行（回退那次的 `previous` 是现行那一版，台阶从它带），水位变了的再留一级；
        // 不写记账树的发布（树表 0 条的一版上的零单元发布）这一版没有记账树。记账树节点数的上界按这次写的行数从上一版的上界往下带
        // （回退那次实现从现行那一版的形状规划，`previous` 就是它）。
        let (inode_number_watermark_steps, accounting_rows, accounting_tree_nodes) =
            if rewritten.contains(&ModelUnitRole::AccountingTree) {
                let accounting_rows = self.accounting_rows_of_a_publish_on(previous);
                (
                    previous.inode_number_watermark_steps
                        + u64::from(kind.leaves_an_inode_number_watermark_step()),
                    accounting_rows,
                    accounting_tree_nodes_upper_bound(
                        previous.accounting_tree_nodes_upper_bound.max(1),
                        accounting_rows,
                    ),
                )
            } else {
                (
                    previous.inode_number_watermark_steps,
                    previous.accounting_rows,
                    previous.accounting_tree_nodes_upper_bound,
                )
            };
        let mut slots_written = 0;
        let mut rewrites_the_allocation_record_tree = false;
        for role in &rewritten {
            match role {
                ModelUnitRole::AllocationTree => rewrites_the_allocation_record_tree = true,
                // 这次写的数据单元：写内容的是内容的每一个，截断与不缩短的截断是那一个尾单元。
                ModelUnitRole::Data => {
                    slots_written += role.span_in_slots()
                        * (data_unit_change.rewritten.end - data_unit_change.rewritten.start);
                }
                // 记账树一次至多重写这一版的全部节点（根与根之下的，每个节点每盘占一个槽）：根之下的节点不对应角色，按上界算。
                ModelUnitRole::AccountingTree => {
                    slots_written += role.span_in_slots() * accounting_tree_nodes;
                }
                // 中央映射树在这里只记它的根：根之下的节点数按上界另算（下面）。extent 上段根之下的节点不在种类定死的角色里，
                // 按这一版的形状另算（下面）。
                ModelUnitRole::ExtentRoot
                | ModelUnitRole::ExtentUpperNodeBelowTheRoot(_)
                | ModelUnitRole::InodeLeaf
                | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
                | ModelUnitRole::InodeRoot
                | ModelUnitRole::MappingTree
                | ModelUnitRole::TreeTable
                | ModelUnitRole::InstanceTable
                | ModelUnitRole::DeletedInodesRoot
                | ModelUnitRole::LoggedOperationsRoot
                | ModelUnitRole::TombstoneLeaf
                | ModelUnitRole::TombstoneRoot
                | ModelUnitRole::RetirementLeaf
                | ModelUnitRole::RetirementRoot => slots_written += role.span_in_slots(),
            }
        }
        // 回退那次截断条目取回退目标那一版的、与现行那一版不同时 logged_ops 根这次重写（K1）。
        if rollback_rewrites_the_logged_operations_root {
            slots_written += ModelUnitRole::LoggedOperationsRoot.span_in_slots();
        }
        // extent 树下段的节点不对应角色（同分配记录树根之下的节点：分配代与位置归池级 checker 判），占槽按条款现算：
        // 写内容的两种重写整个下段；不是最后一批的删除批只重写被截掉那一片下段叶与它的祖先，每层至多一个；截断的批与放了单元的最后一批同
        // （按截之前的层数取上界）；重写尾单元的截断意图与不缩短的截断重写罩着它的那一片下段叶到根，每层一个（判决
        // `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K3）。
        let extent_lower_segment_nodes_written = match kind {
            ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion => {
                extent_lower_segment_nodes(data_unit_change.data_units_after)
            }
            ModelPublishKind::DeletionBatchKeepingDataUnits { .. } => {
                extent_lower_segment_levels(data_units_remaining_before_this_publish.expect(
                    "不是最后一批的删除批只接在删除中的那一版后面（`answer_deletion_step` 判过）",
                ))
            }
            ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch {
                release: ModelFinalTruncationBatchRelease::AtLeastOneDataUnit,
                ..
            } => extent_lower_segment_levels(data_unit_change.data_units_before),
            ModelPublishKind::TruncationIntent {
                extent_tree: ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
            }
            | ModelPublishKind::SizeChangeWithoutShrinking {
                extent_tree: ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
            } => extent_lower_segment_levels(data_unit_change.data_units_after),
            // 按偏移写第一个文件（判决 m4-partial-write-r1 第二节 R4）：罩着重写那一段的下段节点与新长出来的（上面按「内容变了才写」算好的那一份里
            // 分配代是这次的那几个）。
            ModelPublishKind::PartialWriteOfTheFirstFile { .. } => u64::try_from(
                snapshot_bookkeeping
                    .extent_lower_node_written_at
                    .values()
                    .filter(|written_at| **written_at == checkpoint_txg)
                    .count(),
            )
            .expect("节点数"),
            ModelPublishKind::TruncationIntent {
                extent_tree:
                    ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
                    | ModelExtentTreeChangeOfASizeChange::Untouched,
            }
            | ModelPublishKind::FinalTruncationBatch {
                release: ModelFinalTruncationBatchRelease::NoDataUnit,
                ..
            }
            | ModelPublishKind::SizeChangeWithoutShrinking {
                extent_tree:
                    ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
                    | ModelExtentTreeChangeOfASizeChange::Untouched,
            }
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::FinalDeletionBatch { .. }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            // 第一个文件的下段一个节点都不写；写的那个文件的单元与下段在下面另算。
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => 0,
        };
        slots_written += extent_lower_segment_nodes_written * (NODE_BYTES / SLOT_BYTES);
        // 按 inode 号写第一个文件之外的文件：那个文件的每个数据单元与它的整个下段这次写（同第一个文件的整份写，D8（核心索引结构） 已定项 14）。
        slots_written += data_units_written_to_the_file_other_than_the_first
            * ModelUnitRole::Data.span_in_slots()
            + extent_lower_segment_nodes(data_units_written_to_the_file_other_than_the_first)
                * (NODE_BYTES / SLOT_BYTES);
        // 第一个文件之外那个文件改 size、截断的批与最后一批写的尾单元与下段节点（上面按单元号算出来的）。
        slots_written += slots_written_for_the_size_of_an_inode_other_than_the_first;
        // extent 树上段根之下的节点（上段长过层之后才有）各是一个角色，不在种类定死的角色里：这次重写的（规范形计划给它们这次的 txg）各占一个节点的槽。
        // 只有第一个文件的历史里一个都没有，那几组的占槽不变。
        let extent_upper_nodes_below_the_root_rewritten = role_written_at
            .iter()
            .filter(|(role, written_at)| {
                matches!(role, ModelUnitRole::ExtentUpperNodeBelowTheRoot(_))
                    && **written_at == checkpoint_txg
            })
            .count();
        slots_written += u64::try_from(extent_upper_nodes_below_the_root_rewritten)
            .expect("节点数")
            * (NODE_BYTES / SLOT_BYTES);
        // 整理的一批写目的地：至多 `units_at_most` 个单元，每个至多占一个数据单元那么宽（32 KiB 两槽，节点一槽）。
        let relocation_destination_slots_upper_bound = match kind {
            ModelPublishKind::CompactionBatch { units_at_most, .. } => {
                units_at_most * ModelUnitRole::Data.span_in_slots()
            }
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
            | ModelPublishKind::FinalDeletionBatch { .. }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => 0,
        };
        slots_written += relocation_destination_slots_upper_bound;
        // deadlist 树的节点（码 2 节点，每盘占一个槽）：这一版或上一版有条目时按这一版节点数的上界算它这次全写一遍。
        slots_written +=
            snapshot_bookkeeping.deadlist_tree_nodes_written * (NODE_BYTES / SLOT_BYTES);
        // 实例表链多于一片时第 1 片起每一片也是一个重写的单元：`rewritten_roles` 里实例表只记一个角色，多出来的几片在这里补上。
        let instance_table_pages_after_the_first = if kind.rewrites_the_instance_table() {
            instance_table_pages_for_rows(instance_table_rows.len()) - 1
        } else {
            0
        };
        slots_written +=
            instance_table_pages_after_the_first * ModelUnitRole::InstanceTable.span_in_slots();
        // 有树的一版每次发布都重写中央映射树（根恒在 `rewritten_roles` 里）与分配记录树；一次至多重写这一版的全部节点。
        // 树表 0 条的一版没有映射树，分配记录树照写行那次重写不重写算。
        let central_mapping_tree_nodes_upper_bound = if contents.has_trees() {
            assert!(
                rewritten.contains(&ModelUnitRole::MappingTree)
                    && rewrites_the_allocation_record_tree,
                "有树的一版每次发布都重写映射树与分配记录树：{kind:?}"
            );
            let (mapping_nodes, allocation_record_tree_nodes) = self
                .central_mapping_and_allocation_record_tree_nodes_upper_bounds(
                    previous.central_mapping_tree_nodes_upper_bound.max(1),
                    central_mapping_entries_other_than_the_allocation_record_tree(
                        &contents,
                        &role_written_at,
                        accounting_tree_nodes,
                        checkpoint_txg,
                    ) + central_mapping_entries_of_the_files_other_than_the_first(
                        &inodes_other_than_the_first,
                    ) + snapshot_bookkeeping.pinned_mapping_entries
                        + snapshot_bookkeeping.deadlist_tree_nodes_upper_bound,
                    previous.occupied_slots_upper_bound_per_device + slots_written,
                );
            slots_written += (mapping_nodes - 1) * ModelUnitRole::MappingTree.span_in_slots()
                + allocation_record_tree_nodes * ModelUnitRole::AllocationTree.span_in_slots();
            mapping_nodes
        } else {
            if rewrites_the_allocation_record_tree {
                slots_written += self.allocation_record_tree_nodes_rewritten_upper_bound(
                    previous.occupied_slots_upper_bound_per_device + slots_written,
                ) * ModelUnitRole::AllocationTree.span_in_slots();
            }
            0
        };
        let highest_transaction_number_before_this_publish = if previous.key.instance == instance {
            previous.highest_transaction_number_in_its_instance
        } else {
            0
        };
        let transactions_of_this_publish = kind.transactions_carried(
            data_unit_change.rewritten.end - data_unit_change.rewritten.start
                + data_units_written_to_the_file_other_than_the_first,
        );
        let highest_transaction_number_of_its_publish = if transactions_of_this_publish > 0 {
            highest_transaction_number_before_this_publish + transactions_of_this_publish
        } else {
            0
        };
        // 一个事务一条记录（C310（事务切分纪律与记录数口径打架）「一条记录装一个事务连同它的元数据」），不承载事务的发布也写一条；
        // jsn 一条记录加一（D23（journal 的角色与格式） 已定项 7），根带的是这次发布末条记录的 jsn。`journal_counter` 是这次第一条记录的。
        // 末条再跨记录（D23 已定项 17）只在最后一个事务要点名的项多于一条记录装得下的 64 项时才有，模型罩的发布点名的项远不到。
        let records_of_this_publish = transactions_of_this_publish.max(1);
        // logged_ops 照抄上一版（回退那次的 `previous` 是现行那一版：随现行那一版），删意图的几种删空；签发那一条由
        // [`IdealModel::answer_compaction_publish`] 装上。
        let logged_compaction_intent = if kind
            .removed_roles()
            .contains(&ModelUnitRole::LoggedOperationsRoot)
        {
            None
        } else {
            previous.logged_compaction_intent
        };
        // 搬过的根单元：照抄上一版的，只留这一版还引用着的；这一批搬的由 [`IdealModel::answer_compaction_publish`] 记上。
        // 回退那次只取目标那一版的（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H1）：实现回退时树表里用户可见树
        // 那几条根指针整条取目标那一版的字节，目标那一版之后才搬的根单元在新根里仍指搬之前的位置，现行那一版「最后一次搬它」的记录不并进来。
        let mut relocations_carried = match user_visible_trees {
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem => {
                previous.user_visible_root_relocations.as_ref().clone()
            }
            UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(target_root) => {
                target_root.user_visible_root_relocations.as_ref().clone()
            }
        };
        let mut root = ModelRoot {
            key,
            journal_counter: ModelJournalCounter(journal_counter.0 + records_of_this_publish - 1),
            rollback_floor,
            contents,
            instance_table_rows,
            role_written_at,
            data_unit_written_at: Rc::new(data_unit_written_at),
            occupied_slots_upper_bound_per_device: previous.occupied_slots_upper_bound_per_device
                + slots_written,
            central_mapping_tree_nodes_upper_bound,
            inode_number_watermark_steps,
            accounting_rows,
            accounting_tree_nodes_upper_bound: accounting_tree_nodes,
            highest_transaction_number_of_its_publish,
            highest_transaction_number_in_its_instance:
                highest_transaction_number_before_this_publish
                    .max(highest_transaction_number_of_its_publish),
            snapshots: snapshot_bookkeeping.snapshots,
            previous_snapshot_txg_of_the_live_head: snapshot_bookkeeping
                .previous_snapshot_txg_of_the_live_head,
            deadlist: snapshot_bookkeeping.deadlist,
            pinned_mapping_entries: snapshot_bookkeeping.pinned_mapping_entries,
            deadlist_tree_nodes_upper_bound: snapshot_bookkeeping.deadlist_tree_nodes_upper_bound,
            extent_lower_node_written_at: snapshot_bookkeeping.extent_lower_node_written_at,
            logged_compaction_intent,
            user_visible_root_relocations: Rc::new(BTreeMap::new()),
            inodes_other_than_the_first: Rc::new(inodes_other_than_the_first),
            inode_number_watermark,
            death_txg_of_the_first_file,
        };
        let referenced: BTreeSet<ModelUserVisibleRootUnit> =
            root.user_visible_root_units().into_iter().collect();
        relocations_carried.retain(|unit, _| referenced.contains(unit));
        root.user_visible_root_relocations = Rc::new(relocations_carried);
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 这一版第一个文件那条记录的截断字段（模型记的），没有那条记录时 panic。
    fn truncation_field_of(root: &ModelRoot) -> ModelTruncationField {
        root.contents
            .first_file_record_truncation_field()
            .expect("这一版有第一个文件那条记录")
    }

    fn truncation_field_written_at(txg: u64, size_in_bytes: usize) -> ModelTruncationField {
        ModelTruncationField::WrittenByATruncationIntent {
            truncation_txg: ModelCheckpointTxg(txg),
            size_in_bytes: u64::try_from(size_in_bytes).expect("长度"),
        }
    }

    /// 放了单元的截断意图写字段（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第六节「放了单元才写、整个替换」）：
    /// 三个单元的文件（顺序写 txg 10，字段没写过）截到 1 个净荷 + 5 字节，这次放单元之前存在 3 个、多于截后 size 罩的 2 个，意图那次（txg 11）
    /// 写成（11, P + 5）；之后再截到 7 字节（存在 2 个、多于罩的 1 个），整个替换成（13, 7），不取 min、不留上一次的。
    #[test]
    fn truncation_intent_releasing_data_units_writes_its_txg_and_the_new_size_replacing_the_previous_field(
    ) {
        let mut model = model_with_a_file_of(3 * payload());
        assert_eq!(
            truncation_field_of(&current_version(&model)),
            ModelTruncationField::NotWritten,
            "顺序写之后字段没写过"
        );
        let intent = truncate_to(&mut model, payload() + 5);
        assert_eq!(intent.key.checkpoint_txg, ModelCheckpointTxg(11));
        assert_eq!(
            truncation_field_of(&intent),
            truncation_field_written_at(11, payload() + 5)
        );
        assert_eq!(
            truncation_field_of(&intent).recorded_values(),
            ObservedTruncationField {
                truncation_txg: ModelCheckpointTxg(11),
                size_in_bytes: u64::try_from(payload() + 5).expect("长度"),
            },
            "盘上两段：T 是意图那次的 txg，N_w 是截到的 size"
        );
        walk_one_step(&mut model);
        let second_intent = truncate_to(&mut model, 7);
        assert_eq!(second_intent.key.checkpoint_txg, ModelCheckpointTxg(13));
        assert_eq!(
            truncation_field_of(&second_intent),
            truncation_field_written_at(13, 7),
            "第二次放了单元的截断整个替换"
        );
    }

    /// 截到洞里、选择子落在洞里更靠后（攻方 `research/prompts/m4-trunc-r2-opus-output.md`「打中一」那一形）：五个单元的文件（txg 10）截到 1 个净荷
    /// （txg 11 意图写（11, P），txg 12 最后一批放单元 1–4）→ 变长到 5 个净荷（txg 13，照抄）→ 截到 3 个净荷（txg 14：存在 1 个、不多于罩的 3 个，
    /// 一个不放，照抄（11, P），不换成「最近一次」的（14, 3P））→ 最后一批不放单元（照抄）→ 再变长（照抄）。
    #[test]
    fn truncation_intent_into_the_hole_releasing_no_data_unit_copies_the_field_of_the_earlier_truncation(
    ) {
        let mut model = model_with_a_file_of(5 * payload());
        truncate_to(&mut model, payload());
        let final_batch = walk_one_step(&mut model);
        assert_eq!(file_of(&final_batch).data_units_present, 1);
        let grown = truncate_to(&mut model, 5 * payload());
        assert_eq!(
            truncation_field_of(&grown),
            truncation_field_written_at(11, payload())
        );
        let intent_into_the_hole = truncate_to(&mut model, 3 * payload());
        assert_eq!(
            intent_into_the_hole.key.checkpoint_txg,
            ModelCheckpointTxg(14)
        );
        assert_eq!(
            truncation_field_of(&intent_into_the_hole),
            truncation_field_written_at(11, payload()),
            "截到洞里一个不放：照抄第一次截断写的"
        );
        let final_batch_releasing_nothing = walk_one_step(&mut model);
        assert_eq!(
            truncation_field_of(&final_batch_releasing_nothing),
            truncation_field_written_at(11, payload())
        );
        let grown_again = truncate_to(&mut model, 5 * payload());
        assert_eq!(
            truncation_field_of(&grown_again),
            truncation_field_written_at(11, payload())
        );
    }

    /// 截到 0 而文件只有一个单元（攻方「打中三」那条历史的前三步）：半个净荷的文件截到 0，存在 1 个、不多于 ⌈0 ÷ 净荷⌉.max(1) = 1 个，一个不放，
    /// 照抄（没写过）；之后变长到 1 个净荷 + 1 字节、截到 1 个净荷，都照抄，字段始终没写过。
    #[test]
    fn truncation_to_zero_of_a_one_unit_file_releases_nothing_and_copies_the_field() {
        let mut model = model_with_a_file_of(payload() / 2);
        let intent = truncate_to(&mut model, 0);
        assert_eq!(
            truncation_field_of(&intent),
            ModelTruncationField::NotWritten
        );
        walk_one_step(&mut model);
        let grown = truncate_to(&mut model, payload() + 1);
        assert_eq!(
            truncation_field_of(&grown),
            ModelTruncationField::NotWritten
        );
        let cut_to_one_payload = truncate_to(&mut model, payload());
        assert_eq!(
            truncation_field_of(&cut_to_one_payload),
            ModelTruncationField::NotWritten
        );
    }

    /// 截断的各批与最后一批不碰记录、照抄字段（判决 m4-trunc-r2 第三节 R2）：300 个单元截到 0（txg 11 意图写（11, 0)），两批（txg 12、13）与
    /// 最后一批（txg 14）都是（11, 0）。
    #[test]
    fn the_truncation_batches_and_the_final_batch_copy_the_field_the_intent_wrote() {
        let mut model = model_with_a_file_of(300 * payload());
        let intent = truncate_to(&mut model, 0);
        assert_eq!(
            truncation_field_of(&intent),
            truncation_field_written_at(11, 0)
        );
        for expected_txg in [12, 13] {
            let batch = walk_one_step(&mut model);
            assert_eq!(batch.key.checkpoint_txg, ModelCheckpointTxg(expected_txg));
            assert!(model.current_version_has_a_file_being_truncated());
            assert_eq!(
                truncation_field_of(&batch),
                truncation_field_written_at(11, 0)
            );
        }
        let final_batch = walk_one_step(&mut model);
        assert_eq!(final_batch.key.checkpoint_txg, ModelCheckpointTxg(14));
        assert!(matches!(
            final_batch.contents,
            ModelVersionContents::LiveFile(_)
        ));
        assert_eq!(
            truncation_field_of(&final_batch),
            truncation_field_written_at(11, 0)
        );
    }

    /// 变长、N = S 与只重写固定点单元的发布（建快照）照抄字段（判决 m4-trunc-r2 第三节 R2）：两个单元截到 7 字节（txg 11 写（11, 7)，txg 12
    /// 最后一批）→ 跨单元变长（txg 13）→ N = S（txg 14）→ 建快照（txg 15），都是（11, 7）。
    #[test]
    fn growing_the_same_size_and_a_snapshot_creation_copy_the_field() {
        let mut model = model_with_a_file_of(2 * payload());
        truncate_to(&mut model, 7);
        walk_one_step(&mut model);
        let grown = truncate_to(&mut model, 3 * payload() + 1);
        assert_eq!(grown.key.checkpoint_txg, ModelCheckpointTxg(13));
        assert_eq!(
            truncation_field_of(&grown),
            truncation_field_written_at(11, 7)
        );
        let same_size = truncate_to(&mut model, 3 * payload() + 1);
        assert_eq!(same_size.key.checkpoint_txg, ModelCheckpointTxg(14));
        assert_eq!(
            truncation_field_of(&same_size),
            truncation_field_written_at(11, 7)
        );
        let snapshot = model.answer_create_snapshot().expect("会话开着、有树");
        succeed(&mut model, &snapshot);
        let after_the_snapshot = current_version(&model);
        assert_eq!(
            after_the_snapshot.key.checkpoint_txg,
            ModelCheckpointTxg(15)
        );
        assert_eq!(
            truncation_field_of(&after_the_snapshot),
            truncation_field_written_at(11, 7)
        );
    }

    /// 整份写清零（判决 m4-trunc-r2 第三节 R2，取清零）：截过的文件（字段（11, 7)）覆盖写一个单元以内（txg 13）清零；再截一次写上
    /// （txg 14、15），顺序写跨几个单元（txg 16）也清零。
    #[test]
    fn an_overwrite_and_a_sequential_write_clear_the_field() {
        let mut model = model_with_a_file_of(2 * payload());
        truncate_to(&mut model, 7);
        walk_one_step(&mut model);
        let overwrite = model
            .answer_publish_overwrite(&content_without_zeros(100))
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &overwrite);
        let overwritten = current_version(&model);
        assert_eq!(overwritten.key.checkpoint_txg, ModelCheckpointTxg(13));
        assert_eq!(
            truncation_field_of(&overwritten),
            ModelTruncationField::NotWritten
        );
        let sequential = model
            .answer_publish_sequential_write(&content_without_zeros(3 * payload()))
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &sequential);
        let intent = truncate_to(&mut model, 9);
        assert_eq!(
            truncation_field_of(&intent),
            truncation_field_written_at(15, 9)
        );
        walk_one_step(&mut model);
        let sequential_again = model
            .answer_publish_sequential_write(&content_without_zeros(2 * payload()))
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &sequential_again);
        let rewritten = current_version(&model);
        assert_eq!(rewritten.key.checkpoint_txg, ModelCheckpointTxg(17));
        assert_eq!(
            truncation_field_of(&rewritten),
            ModelTruncationField::NotWritten
        );
    }

    /// unlink 的意图与各批照抄字段（判决 m4-trunc-r2 第三节 R2）：300 个单元截到 150 个净荷（txg 11 写（11, 150 P），一批放 144 个、最后一批放 6 个）
    /// → unlink 意图（txg 14，删除中、150 个单元）→ 不是最后一批的删除批（txg 15）都是（11, 150 P）；最后一批之后没有第一个文件那条记录。
    #[test]
    fn the_unlink_intent_and_the_deletion_batches_copy_the_field_until_the_record_is_deleted() {
        let mut model = model_with_a_file_of(300 * payload());
        truncate_to(&mut model, 150 * payload());
        walk_one_step(&mut model);
        walk_one_step(&mut model);
        let unlink = model
            .answer_unlink_intent()
            .expect("会话开着、现行版本带活文件");
        succeed(&mut model, &unlink);
        let unlink_intent = current_version(&model);
        assert_eq!(unlink_intent.key.checkpoint_txg, ModelCheckpointTxg(14));
        assert!(matches!(
            unlink_intent.contents,
            ModelVersionContents::FileBeingDeleted {
                data_units_remaining: 150,
                ..
            }
        ));
        assert_eq!(
            truncation_field_of(&unlink_intent),
            truncation_field_written_at(11, 150 * payload())
        );
        let deletion_batch = walk_one_step(&mut model);
        assert!(matches!(
            deletion_batch.contents,
            ModelVersionContents::FileBeingDeleted {
                data_units_remaining: 6,
                ..
            }
        ));
        assert_eq!(
            truncation_field_of(&deletion_batch),
            truncation_field_written_at(11, 150 * payload())
        );
        let final_deletion_batch = walk_one_step(&mut model);
        assert_eq!(
            final_deletion_batch
                .contents
                .first_file_record_truncation_field(),
            None,
            "最后一批删掉第一个文件那条记录"
        );
    }

    /// 回退随回退目标那一版（判决 m4-trunc-r2 第三节 R5）：三个单元的文件（txg 10，字段没写过）截到 7 字节（txg 11 写（11, 7)、txg 12 最后一批）
    /// → 回退到写字段之前那一版（txg 10，txg 13 那次回退）：字段没写过，不把现行那一版的（11, 7）并回去；之后变长（txg 14）照抄，仍没写过。
    /// 再回退到意图那一版（txg 11）取回（11, 7）。
    #[test]
    fn rollback_takes_the_field_of_its_target_and_the_growth_after_it_copies_that() {
        let mut model = model_with_a_file_of(3 * payload());
        let before_the_truncation = current_version(&model);
        let intent = truncate_to(&mut model, 7);
        walk_one_step(&mut model);
        let rollback = model
            .answer_rollback_while_mounted(before_the_truncation.key)
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &rollback);
        let rolled_back = current_version(&model);
        assert_eq!(rolled_back.key.checkpoint_txg, ModelCheckpointTxg(13));
        assert_eq!(
            truncation_field_of(&rolled_back),
            ModelTruncationField::NotWritten,
            "回退到写字段之前那一版：字段随它"
        );
        let grown = truncate_to(&mut model, 4 * payload());
        assert_eq!(grown.key.checkpoint_txg, ModelCheckpointTxg(14));
        assert_eq!(
            truncation_field_of(&grown),
            ModelTruncationField::NotWritten
        );
        let rollback_to_the_intent = model
            .answer_rollback_while_mounted(intent.key)
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &rollback_to_the_intent);
        assert_eq!(
            truncation_field_of(&current_version(&model)),
            truncation_field_written_at(11, 7),
            "回退到意图那一版取回它的字段"
        );
    }
}
