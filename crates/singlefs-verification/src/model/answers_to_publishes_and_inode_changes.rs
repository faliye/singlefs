//! 模型对开会话、发布第一个文件、覆盖写、顺序写、按偏移写、建 inode、删文件与零单元发布的回答。

use super::*;

impl IdealModel {
    pub(super) fn open_session(&self) -> Result<&ModelSession, ModelDisagreement> {
        self.session.as_ref().ok_or_else(|| {
            ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                "模型里这个进程没有可写会话".to_string(),
                "实现的会话开着（执行器调了要会话的入口）".to_string(),
            )
        })
    }

    /// 发布第一个文件版本（`transaction::publish_first_file`）。
    ///
    /// # Errors
    /// 模型里没有可写会话：会话状态与实现对不上。
    pub fn answer_publish_first_file(
        &self,
        content: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let mut required_refusals = BTreeSet::new();
        // 第一个文件版本接在现行那一版后面：txg 与 jsn 各加一，不取 `NEW_POOL_FILE_CREATION_TXG`——那个常量只管 mkfs
        // 同一个进程里那条流。执行器恒拿现行那一版的根与记录，所以
        // `FirstFileVersionDoesNotFollowTheVersionItBuildsOn` 在随机历史里走不到、模型一次都不要求它。
        // 现行那一版已经有过文件版本：这条路径按「树还没建起来」写，再走一次会重新建树、重新发对象出生代，
        // 与环里那些旧根记的对不上（I-9.14、I-9.10）——再写一版走覆盖写。
        if current.contents.has_trees() {
            required_refusals
                .insert(ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile);
        }
        if u64::try_from(content.len()).expect("内容长度") > data_unit_payload_capacity_in_bytes()
        {
            required_refusals.insert(ModelRefusalReason::ContentExceedsDataUnitPayload);
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(current.journal_counter.0 + 1),
            ModelPublishKind::FirstFileVersion,
            current.rollback_floor,
            Some(content),
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishFirstFile,
            required_refusals,
            vec![root],
        ))
    }

    /// 覆盖写（`transaction::publish_overwrite`）：接在现行的带文件的一版后面。现行那一版有树而文件集合为空时（执行器在删除中的那一版上调，
    /// 会话先走空、把它走成这一形）必须拒（[`ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile`]）。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本没有文件、也不是有树而文件集合为空（执行器只在带文件或删除中时调）：会话状态与实现对不上。
    pub fn answer_publish_overwrite(
        &self,
        content: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        self.answer_rewriting_the_whole_file(
            content,
            ModelOperationKind::PublishOverwrite,
            WholeFileWriteContract::OneDataUnit,
        )
    }

    /// 顺序写（`transaction::publish_sequential_write`，经会话的 `UserChange::SequentialWrite`）：同覆盖写接在现行的带文件的一版后面、
    /// 整份重写文件内容，不按单元数拒——内容切成几个数据单元就写几个（一个单元一个事务），多于一个时连同 extent 树下段一起写
    /// （D8（核心索引结构） 已定项 14）。现行那一版有树而文件集合为空时同覆盖写必须拒。
    ///
    /// # Errors
    /// 同 [`IdealModel::answer_publish_overwrite`]。
    pub fn answer_publish_sequential_write(
        &self,
        content: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        self.answer_rewriting_the_whole_file(
            content,
            ModelOperationKind::PublishSequentialWrite,
            WholeFileWriteContract::AnyNumberOfDataUnits,
        )
    }

    /// 覆盖写与顺序写共用的答法：两者只差入口的契约（一个数据单元以内，还是不按单元数拒）。
    fn answer_rewriting_the_whole_file(
        &self,
        content: &[u8],
        operation: ModelOperationKind,
        contract: WholeFileWriteContract,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if let Some(refused) =
            self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(operation)
        {
            return Ok(refused);
        }
        // 截断途中的那一版读得出文件，但会话先走空（截断的各批），走到这里现行版本只会是带活文件的一版。
        if !matches!(current.contents, ModelVersionContents::LiveFile(_)) {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有读得出的文件：{:?}",
                    current.key, current.contents
                ),
                "实现的现行版本带文件、队列空（执行器调了覆盖写或顺序写）".to_string(),
            ));
        }
        let mut required_refusals = BTreeSet::new();
        match contract {
            WholeFileWriteContract::OneDataUnit => {
                if u64::try_from(content.len()).expect("内容长度")
                    > data_unit_payload_capacity_in_bytes()
                {
                    required_refusals.insert(ModelRefusalReason::ContentExceedsDataUnitPayload);
                }
            }
            WholeFileWriteContract::AnyNumberOfDataUnits => {}
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::OverwriteFileVersion,
            current.rollback_floor,
            Some(content),
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(operation, required_refusals, vec![root]))
    }

    /// 零单元发布（`transaction::publish_without_units`）：接在现行的树表 0 条的一版后面。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本带文件（执行器只在树表 0 条时调）：会话状态与实现对不上。
    pub fn answer_publish_without_units(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if current.contents.has_trees() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!("模型里现行版本 {:?} 有树", current.key),
                "实现的现行版本树表 0 条（执行器调了零单元发布）".to_string(),
            ));
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::ZeroUnit,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishWithoutUnits,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// unlink 第一个文件的意图那次（`mounted_session::UserChange::Unlink`，inode 1）：同 [`IdealModel::answer_unlink_intent_of_an_inode`]。
    ///
    /// # Errors
    /// 同 [`IdealModel::answer_unlink_intent_of_an_inode`]。
    pub fn answer_unlink_intent(&self) -> Result<ModelAnswer, ModelDisagreement> {
        self.answer_unlink_intent_of_an_inode(FIRST_INODE_NUMBER)
    }

    /// unlink `inode` 的意图那次（`mounted_session::UserChange::Unlink`，D8（核心索引结构） 已定项 16 第 1 条；判决
    /// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1、M3：对任一号同一条）：接在现行那一版后面，那条记录 nlink 写 0、
    /// `deleted_inodes` 加 key、墓碑树追加一条；inode 1 的这一版读回「没有文件」，别的号这一版读不出它、第一个文件照旧。会话接着走空（每一批由
    /// [`IdealModel::answer_deletion_step`] 答）；两次之间会话为空间推的抬 F 由执行器按 txg 次序插进来比。逐次发布判墙。
    /// 有树而文件集合为空的一版上必拒（[`ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile`]）。
    ///
    /// # Errors
    /// 模型里没有可写会话，或 `inode` 不是现行那一版树里活着的号（执行器只点名实现现行那一版里活着的号，会话先走空）：会话状态与实现对不上；
    /// 现行那一版 inode 树多于一片叶容器（删一条要吸收或借，模型的删除那一臂只答一片的，执行器在那一形上不调）：[`ModelDisagreementAspect::NotModeled`]。
    pub fn answer_unlink_intent_of_an_inode(
        &self,
        inode: u64,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if let Some(refused) = self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(
            ModelOperationKind::PublishUnlinkIntent,
        ) {
            return Ok(refused);
        }
        if current.inode_records_in_the_tree() > INODE_LEAF_RECORDS {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!(
                    "模型里现行版本 {:?} 的 inode 树有 {} 条记录、多于一片叶容器：删除那一臂只答一片的（删一条不吸收、不借）",
                    current.key,
                    current.inode_records_in_the_tree()
                ),
                format!("执行器调了 unlink inode {inode}"),
            ));
        }
        let deleted = ModelDeletedInode::of_the_inode_number(inode);
        let is_a_live_record = match deleted {
            ModelDeletedInode::FirstFile => {
                matches!(current.contents, ModelVersionContents::LiveFile(_))
            }
            ModelDeletedInode::OtherThanTheFirst(other) => {
                matches!(
                    current.contents,
                    ModelVersionContents::LiveFile(_)
                        | ModelVersionContents::InodesWithoutTheFirstFile { .. }
                ) && current
                    .inodes_other_than_the_first
                    .get(&other)
                    .is_some_and(ModelInodeOtherThanTheFirst::is_live)
            }
        };
        if !is_a_live_record {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有活着的 inode {inode}：{:?}，第一个文件之外的 {:?}",
                    current.key,
                    current.contents,
                    current
                        .inodes_other_than_the_first
                        .keys()
                        .collect::<Vec<_>>()
                ),
                format!("实现的现行版本带文件、队列空（执行器调了 unlink inode {inode}）"),
            ));
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::UnlinkIntent { deleted },
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishUnlinkIntent,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 走空 `deleted_inodes` 的一步（D8（核心索引结构） 已定项 16 第 2、6 条：会话里 unlink 之后、每次用户改动之前，挂载续做；判决
    /// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1：对任一号同一条，剩几个单元取那个号自己的）：删除中的那个号还剩的
    /// 数据单元多于产品批大小 144 就是一个不是最后一批的删除批（从尾部释放 144 个，之后仍是删除中的那一版），不多于就是最后一批——删掉那条记录：
    /// 树里还有别的记录时是带文件的一版（删第一个文件而别的文件还在也是，第三节 M3），一条都不剩才是有树而文件集合为空的一版；extent 上段按规范形
    /// 收缩（[`extent_upper_node_written_at_after`]：降层那次根照抄）。下一步只由现行那一版定，挂载续做与一路做下来同一个答法。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本没有删除中的号（执行器只在实现交回了一步删除时问）：会话状态与实现对不上；inode 树多于一片叶容器：
    /// [`ModelDisagreementAspect::NotModeled`]（同 [`IdealModel::answer_unlink_intent_of_an_inode`]）。
    pub fn answer_deletion_step(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let Some((deleted, data_units_remaining)) = current.inode_being_deleted() else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有删除中的号：{:?}",
                    current.key, current.contents
                ),
                "实现交回了走空 deleted_inodes 的一步".to_string(),
            ));
        };
        if current.inode_records_in_the_tree() > INODE_LEAF_RECORDS {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!(
                    "模型里现行版本 {:?} 的 inode 树有 {} 条记录、多于一片叶容器：删除那一臂只答一片的",
                    current.key,
                    current.inode_records_in_the_tree()
                ),
                "实现交回了走空 deleted_inodes 的一步".to_string(),
            ));
        }
        let step = if data_units_remaining > MODEL_DELETION_BATCH_DATA_UNITS {
            ModelPublishKind::DeletionBatchKeepingDataUnits { deleted }
        } else {
            let inode_tree_after = if current.inode_records_in_the_tree() == 1 {
                ModelInodeTreeAfterTheFinalDeletionBatch::Emptied
            } else {
                ModelInodeTreeAfterTheFinalDeletionBatch::StillHoldsRecords
            };
            // 上段的根这次重写还是照抄：按规范形计划先算一遍（`next_root_with_the_user_visible_trees` 里同一份计划再算一遍，断言种类与它一致）。
            let txg_of_the_batch = ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1);
            let inodes_with_an_entry_before = current.inodes_with_an_extent_upper_leaf_entry();
            let mut inodes_with_an_entry_after = inodes_with_an_entry_before.clone();
            let inodes_whose_entry_changed =
                if inodes_with_an_entry_after.remove(&deleted.inode_number()) {
                    BTreeSet::from([deleted.inode_number()])
                } else {
                    BTreeSet::new()
                };
            let root_after = ModelExtentUpperNodePosition {
                level: extent_upper_segment_root_level(&inodes_with_an_entry_after),
                index_in_level: 0,
            };
            let root_is_rewritten = extent_upper_node_written_at_after(
                &current.extent_upper_node_written_at(),
                &inodes_with_an_entry_after,
                &inodes_whose_entry_changed,
                txg_of_the_batch,
            )
            .get(&root_after)
                == Some(&txg_of_the_batch);
            let extent_upper_root = if root_is_rewritten {
                ModelExtentUpperRootOfTheFinalDeletionBatch::Rewritten
            } else {
                ModelExtentUpperRootOfTheFinalDeletionBatch::CarriedFromThePreviousVersion
            };
            ModelPublishKind::FinalDeletionBatch {
                deleted,
                inode_tree_after,
                extent_upper_root,
            }
        };
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            step,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishDeletionStep,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 建一个 size 0 的 inode（`mounted_session::UserChange::NewInodes`，count 1）：接在第一个文件已删完的一版后面
    /// （有树而文件集合为空、或删完之后建过 inode 的），这一版仍读回「没有文件」。会话先走空（队列空，一个写都不发）。逐次发布判墙。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本里第一个文件还在、树表 0 条（执行器只在第一个文件已删完时调）：会话状态与实现对不上。
    pub fn answer_create_inode(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        match current.contents {
            ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => {}
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::SessionState,
                    format!(
                        "模型里现行版本 {:?} 不是第一个文件已删完的一版：{:?}",
                        current.key, current.contents
                    ),
                    "实现的现行版本第一个文件已删完（执行器调了建 inode）".to_string(),
                ));
            }
        }
        self.answer_create_inodes(1)
    }

    /// 建 `count` 个 size 0 的 inode（`mounted_session::UserChange::NewInodes`）：接在第一个文件活着的一版（在第一个文件旁边建：按 inode 号写数据
    /// 先要有别的 inode，里程碑「删除与多次 COW」一·1）或第一个文件已删完的一版后面。号从这一版的水位起连着发（[`ModelRoot::inode_number_watermark`]），
    /// 新记录追加在最右那片叶容器、满 233 条在末尾分裂（D8（核心索引结构） 已定项 6），一次一个事务、留一级水位台阶。会话先走空（队列空，一个写都不发）。
    /// 逐次发布判墙。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本树表 0 条、删除中、截断途中（会话先走空，执行器在走空之后才问这一次）：会话状态与实现对不上；`count` 是 0、
    /// 或这几条新记录跨了叶容器（一次要重写两片，模型只答一片：执行器把 `count` 压到最右那片装得下的条数）：[`ModelDisagreementAspect::NotModeled`]。
    pub fn answer_create_inodes(&self, count: u64) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        match current.contents {
            ModelVersionContents::LiveFile(_)
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => {}
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::SessionState,
                    format!(
                        "模型里现行版本 {:?} 不是第一个文件活着或已删完、队列空的一版：{:?}",
                        current.key, current.contents
                    ),
                    "实现的现行版本有树、队列空（执行器调了建 inode）".to_string(),
                ));
            }
        }
        let inode_records_before = current.inode_records_in_the_tree();
        // 新记录落在哪一片叶容器定这次重写哪一片（D8（核心索引结构） 已定项 6）：最左那片之外的那几片各是一个角色。
        let position_of_the_first_new_record =
            inode_leaf_container_position_of_a_new_record(inode_records_before);
        let fits_in_one_leaf_container = count > 0
            && inode_leaf_container_position_of_a_new_record(inode_records_before + count - 1)
                == position_of_the_first_new_record;
        if !fits_in_one_leaf_container {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!(
                    "一次建 {count} 个 inode、树里原有 {inode_records_before} 条：新记录不落在同一片叶容器里，模型只答一次重写一片的"
                ),
                "执行器调了建 inode".to_string(),
            ));
        }
        let kind = match NonZeroU64::new(position_of_the_first_new_record) {
            None => ModelPublishKind::NewInode,
            Some(position_from_the_leftmost) => {
                ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(
                    ModelInodeLeafContainerRightOfTheLeftmost {
                        position_from_the_leftmost,
                    },
                )
            }
        };
        let root = self.next_root_with_the_user_visible_trees(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            kind,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem,
            InodesOtherThanTheFirstChangeOfThePublish::Created { count },
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishNewInode,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 按 inode 号覆盖写（`mounted_session::UserChange::OverwriteOfAnInode`，`transaction::publish_overwrite_of_an_inode`）：契约是一个数据单元，
    /// 装不下必须拒。答法见 [`IdealModel::answer_whole_write_of_an_inode`]。
    ///
    /// # Errors
    /// 同 [`IdealModel::answer_whole_write_of_an_inode`]。
    pub fn answer_overwrite_of_an_inode(
        &self,
        inode: u64,
        content: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        self.answer_whole_write_of_an_inode(inode, content, WholeFileWriteContract::OneDataUnit)
    }

    /// 按 inode 号顺序写（`mounted_session::UserChange::SequentialWriteOfAnInode`，`transaction::publish_sequential_write_of_an_inode`）：
    /// 不按单元数拒。答法见 [`IdealModel::answer_whole_write_of_an_inode`]。
    ///
    /// # Errors
    /// 同 [`IdealModel::answer_whole_write_of_an_inode`]。
    pub fn answer_sequential_write_of_an_inode(
        &self,
        inode: u64,
        content: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        self.answer_whole_write_of_an_inode(
            inode,
            content,
            WholeFileWriteContract::AnyNumberOfDataUnits,
        )
    }

    /// 按 inode 号整份写 `inode` 那个文件（里程碑「删除与多次 COW」一·1）：
    /// - `inode` 是第一个文件：同覆盖写与顺序写（[`IdealModel::answer_publish_overwrite`]，两个入口在实现里就是那两条路）；
    /// - 现行那一版有树而文件集合为空：必拒（[`ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile`]，会话拒成「没有文件版本可改」）；
    /// - 第一个文件已删完、目标是删完之后才建的：必拒（[`ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted`]，D8（核心索引结构）
    ///   已定项 16 射程「删完之后不再建带数据的文件（K-U5，记欠账）」「β 时序级」：对象出生代晚于 inode 1 墓碑死亡 txg 的才拒）；
    /// - 第一个文件活着，或已删完而目标是删之前就有的：那个文件整份换成 `content`（[`ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst`]），
    ///   一个数据单元的契约装不下时必拒。
    ///
    /// # Errors
    /// 模型里没有可写会话；现行版本树表 0 条、删除中、截断途中，或 `inode` 不是这一版树里第一个文件之外活着的号（执行器只点名实现
    /// 现行那一版树里活着的号）：会话状态与实现对不上。
    fn answer_whole_write_of_an_inode(
        &self,
        inode: u64,
        content: &[u8],
        contract: WholeFileWriteContract,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        if inode == FIRST_INODE_NUMBER {
            let operation = match contract {
                WholeFileWriteContract::OneDataUnit => ModelOperationKind::PublishOverwrite,
                WholeFileWriteContract::AnyNumberOfDataUnits => {
                    ModelOperationKind::PublishSequentialWrite
                }
            };
            return self.answer_rewriting_the_whole_file(content, operation, contract);
        }
        let operation = ModelOperationKind::PublishWholeWriteOfAnInodeOtherThanTheFirst;
        let session = self.open_session()?;
        let current = &session.current;
        if let Some(refused) =
            self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(operation)
        {
            return Ok(refused);
        }
        let first_file_records_in_the_tree = match current.contents {
            ModelVersionContents::LiveFile(_) => 1,
            ModelVersionContents::InodesWithoutTheFirstFile { .. } => 0,
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::SessionState,
                    format!(
                        "模型里现行版本 {:?} 不是第一个文件活着、或已删完而树里还有别的 inode 的一版：{:?}",
                        current.key, current.contents
                    ),
                    format!("实现的现行版本树里有 inode {inode}（执行器按 inode 号写它）"),
                ));
            }
        };
        // 记录按号排：第一个文件活着时 inode 1 在最前，第一个文件之外的按号升序接在后面（删除中的那一条还在树里，也算）；第 r 条（从 0 数）在左起第
        // ⌊r ÷ 233⌋ 片叶容器里（建 inode 只往右追加、满了在末尾分裂，[`inode_leaf_container_position_of_a_new_record`]；删除只在树里只有一片叶容器时做，
        // 删完仍是一片，这条数法不破）。
        let Some((position_among_the_inodes_other_than_the_first, written_inode)) = current
            .inodes_other_than_the_first
            .iter()
            .enumerate()
            .find(|(_, (number, other_inode))| **number == inode && other_inode.is_live())
            .map(|(position, (_, other_inode))| (position, other_inode))
        else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 第一个文件之外的 inode 是 {:?}，没有活着的 {inode}",
                    current.key,
                    current
                        .inodes_other_than_the_first
                        .keys()
                        .collect::<Vec<_>>()
                ),
                format!("实现的现行版本树里有 inode {inode}（执行器按 inode 号写它）"),
            ));
        };
        let record_position_in_the_tree = first_file_records_in_the_tree
            + u64::try_from(position_among_the_inodes_other_than_the_first).expect("条数");
        let inode_leaf_container = ModelUnitRole::inode_leaf_container_at(
            record_position_in_the_tree / INODE_LEAF_RECORDS,
        );
        let mut required_refusals = BTreeSet::new();
        match contract {
            WholeFileWriteContract::OneDataUnit => {
                if u64::try_from(content.len()).expect("内容长度")
                    > data_unit_payload_capacity_in_bytes()
                {
                    required_refusals.insert(ModelRefusalReason::ContentExceedsDataUnitPayload);
                }
            }
            WholeFileWriteContract::AnyNumberOfDataUnits => {}
        }
        // K-U5 的 β 时序级读法：第一个文件那条记录不在这一版里、墓碑树里有 inode 1 的墓碑、目标的对象出生代晚于它的死亡 txg
        // 才拒，在任何写之前；删之前就有的别的文件照常能写。
        let is_born_after_the_first_file_was_deleted = first_file_records_in_the_tree == 0
            && current
                .death_txg_of_the_first_file
                .is_some_and(|death_txg| written_inode.object_birth > death_txg);
        if is_born_after_the_first_file_was_deleted {
            required_refusals
                .insert(ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted);
            return Ok(self.answer_for_publishes(operation, required_refusals, Vec::new()));
        }
        let root = self.next_root_with_the_user_visible_trees(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst {
                inode_leaf_container,
            },
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem,
            InodesOtherThanTheFirstChangeOfThePublish::WholeWrite { inode, content },
        );
        Ok(self.answer_for_publishes(operation, required_refusals, vec![root]))
    }

    /// 按偏移写 `inode` 那个文件（`mounted_session::UserChange::WriteAtOffset`，`transaction::publish_partial_write`；判决
    /// `research/prompts/m4-partial-write-r1-main-verification.md` 第二节）：会话先走空（执行器把走空的那几步按「走空的一步」比过之后才问这一次），
    /// 再接在现行那一版后面发一次，逐次发布判墙：
    /// - 现行那一版有树而文件集合为空：必拒（[`ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile`]，会话拒成「没有文件版本可改」）；
    /// - `inode` 不是第一个文件、第一个文件已删完而它是删完之后才建的：必拒（[`ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted`]，
    ///   K-U5 的 β 时序级读法，同按 inode 号整份写）；
    /// - 写的终点越过 u64、或 ⌈写之后的 size ÷ 净荷⌉ × 净荷越过 u64：必拒（[`ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow`]）；
    /// - 别的：读回的内容在偏移处盖上这次的字节，size = max(写之前, 终点)，写之前的末尾到偏移之间读 0、洞读 0（R5）；重写哪一段照
    ///   [`model_partial_write_rewritten_data_units`]（R4）；截断字段照抄（R5）。
    ///
    /// 不答「一次发布切出来的记录多于在飞上限」那一拒：模型不记 journal 环的槽数，执行器取的偏移离末尾至多几个净荷，重写的单元数远在上限之下。
    ///
    /// # Errors
    /// 模型里没有可写会话；现行版本树表 0 条、删除中、截断途中，或 `inode` 不是这一版树里活着的号（执行器只点名实现现行那一版树里活着的号）：
    /// 会话状态与实现对不上。这次的字节是空的（长度 0 的写条款没写，实现在类型上写不出来）、写的终点装不进这个进程的地址空间：
    /// [`ModelDisagreementAspect::NotModeled`]。
    pub fn answer_partial_write(
        &self,
        inode: u64,
        offset_in_bytes: u64,
        bytes: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let operation = ModelOperationKind::PublishPartialWrite;
        let session = self.open_session()?;
        let current = &session.current;
        if let Some(refused) =
            self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(operation)
        {
            return Ok(refused);
        }
        if bytes.is_empty() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                "长度 0 的按偏移写条款没写（判决 m4-partial-write-r1 没判），模型不答".to_string(),
                format!("执行器按偏移写 inode {inode}、0 字节"),
            ));
        }
        let written = if inode == FIRST_INODE_NUMBER {
            // 截断途中的那一版读得出文件，但会话先走空（截断的各批），走到这里现行版本只会是带活文件的一版。
            let ModelVersionContents::LiveFile(file) = &current.contents else {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::SessionState,
                    format!(
                        "模型里现行版本 {:?} 没有带活文件、队列空的 inode 1：{:?}",
                        current.key, current.contents
                    ),
                    "实现的现行版本带文件、队列空（执行器按偏移写 inode 1）".to_string(),
                ));
            };
            ModelFileWrittenAtAnOffset::TheFirstFile(file)
        } else {
            let first_file_records_in_the_tree = match current.contents {
                ModelVersionContents::LiveFile(_) => 1,
                ModelVersionContents::InodesWithoutTheFirstFile { .. } => 0,
                ModelVersionContents::WithoutTrees
                | ModelVersionContents::TreesAndNoFile { .. }
                | ModelVersionContents::FileBeingDeleted { .. }
                | ModelVersionContents::FileBeingTruncated { .. } => {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::SessionState,
                        format!(
                            "模型里现行版本 {:?} 不是第一个文件活着、或已删完而树里还有别的 inode 的一版：{:?}",
                            current.key, current.contents
                        ),
                        format!("实现的现行版本树里有 inode {inode}（执行器按偏移写它）"),
                    ));
                }
            };
            // 记录按号排，第 r 条在左起第 ⌊r ÷ 233⌋ 片叶容器里（同按 inode 号整份写，[`IdealModel::answer_whole_write_of_an_inode`]）。
            let Some((position_among_the_inodes_other_than_the_first, written_inode)) = current
                .inodes_other_than_the_first
                .iter()
                .enumerate()
                .find(|(_, (number, other_inode))| **number == inode && other_inode.is_live())
                .map(|(position, (_, other_inode))| (position, other_inode))
            else {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::SessionState,
                    format!(
                        "模型里现行版本 {:?} 第一个文件之外的 inode 是 {:?}，没有活着的 {inode}",
                        current.key,
                        current
                            .inodes_other_than_the_first
                            .keys()
                            .collect::<Vec<_>>()
                    ),
                    format!("实现的现行版本树里有 inode {inode}（执行器按偏移写它）"),
                ));
            };
            // K-U5 的 β 时序级读法：同按 inode 号整份写，在任何写之前拒。
            let is_born_after_the_first_file_was_deleted = first_file_records_in_the_tree == 0
                && current
                    .death_txg_of_the_first_file
                    .is_some_and(|death_txg| death_txg < written_inode.object_birth);
            if is_born_after_the_first_file_was_deleted {
                return Ok(self.answer_for_publishes(
                    operation,
                    BTreeSet::from([
                        ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted,
                    ]),
                    Vec::new(),
                ));
            }
            let record_position_in_the_tree = first_file_records_in_the_tree
                + u64::try_from(position_among_the_inodes_other_than_the_first).expect("条数");
            ModelFileWrittenAtAnOffset::OtherThanTheFirst {
                inode_leaf_container: ModelUnitRole::inode_leaf_container_at(
                    record_position_in_the_tree / INODE_LEAF_RECORDS,
                ),
                file: written_inode,
            }
        };
        let (content_before, data_units_before, declared_length_of_the_last_existing_unit) =
            match written {
                ModelFileWrittenAtAnOffset::TheFirstFile(file) => (
                    &file.content,
                    file.data_units_present,
                    file.data_units_present
                        .checked_sub(1)
                        .map(|last| file.declared_length_of_the_data_unit(last)),
                ),
                ModelFileWrittenAtAnOffset::OtherThanTheFirst { file, .. } => (
                    &file.content,
                    file.data_units_present(),
                    file.data_units_present()
                        .checked_sub(1)
                        .map(|last| file.declared_length_of_the_data_unit(last)),
                ),
            };
        let size_before = u64::try_from(content_before.len()).expect("内容长度");
        let length_in_bytes = u64::try_from(bytes.len()).expect("一段内存里的字节数装得进 u64");
        let payload = data_unit_payload_capacity_in_bytes();
        let end_exclusive = offset_in_bytes.checked_add(length_in_bytes);
        let size_after_rounds_up_within_the_range = end_exclusive.is_some_and(|end| {
            end.max(size_before)
                .div_ceil(payload)
                .checked_mul(payload)
                .is_some()
        });
        let Some(end_exclusive) = end_exclusive.filter(|_| size_after_rounds_up_within_the_range)
        else {
            return Ok(self.answer_for_publishes(
                operation,
                BTreeSet::from([ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow]),
                Vec::new(),
            ));
        };
        let (Ok(offset_in_memory), Ok(end_in_memory)) = (
            usize::try_from(offset_in_bytes),
            usize::try_from(end_exclusive),
        ) else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!("模型照字节记内容，写到 {end_exclusive} 字节装不进这个进程的地址空间"),
                format!("执行器按偏移写 inode {inode}"),
            ));
        };
        let mut content_after = content_before.to_vec();
        if content_after.len() < end_in_memory {
            content_after.resize(end_in_memory, 0);
        }
        content_after[offset_in_memory..end_in_memory].copy_from_slice(bytes);
        let (first_rewritten_data_unit, data_units_after) =
            model_partial_write_rewritten_data_units(
                data_units_before,
                declared_length_of_the_last_existing_unit,
                offset_in_bytes,
                end_exclusive,
            );
        let checkpoint_txg = ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1);
        let journal_counter = ModelJournalCounter(self.highest_journal_counter.0 + 1);
        let root = match written {
            ModelFileWrittenAtAnOffset::TheFirstFile(_) => self.next_root(
                current,
                session.instance,
                checkpoint_txg,
                journal_counter,
                ModelPublishKind::PartialWriteOfTheFirstFile {
                    first_rewritten_data_unit,
                    data_units_after,
                },
                current.rollback_floor,
                Some(&content_after),
                Rc::clone(&current.instance_table_rows),
            ),
            ModelFileWrittenAtAnOffset::OtherThanTheFirst {
                inode_leaf_container,
                ..
            } => self.next_root_with_the_user_visible_trees(
                current,
                session.instance,
                checkpoint_txg,
                journal_counter,
                ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst {
                    inode,
                    inode_leaf_container,
                    first_rewritten_data_unit,
                    data_units_after,
                },
                current.rollback_floor,
                None,
                Rc::clone(&current.instance_table_rows),
                UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem,
                InodesOtherThanTheFirstChangeOfThePublish::PartialWrite {
                    inode,
                    content_after: &content_after,
                },
            ),
        };
        Ok(self.answer_for_publishes(operation, BTreeSet::new(), vec![root]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 内容装不装得下按 32768 含头与预留位算（D4（校验和位置） 已定项 5）：正好装满不拒，多一个字节才拒。
    #[test]
    fn content_of_exactly_the_payload_capacity_is_accepted_and_one_byte_more_is_refused() {
        assert_eq!(data_unit_payload_capacity_in_bytes(), 32768 - 105 - 29);
        let model = model_after_four_overwrites_in_a_second_instance();
        let capacity = usize::try_from(data_unit_payload_capacity_in_bytes()).expect("三万多");
        let fits = model
            .answer_publish_overwrite(&vec![7; capacity])
            .expect("会话开着");
        assert!(fits.required_refusals.is_empty(), "{fits:?}");
        let exceeds = model
            .answer_publish_overwrite(&vec![7; capacity + 1])
            .expect("会话开着");
        assert_eq!(
            exceeds.required_refusals,
            BTreeSet::from([ModelRefusalReason::ContentExceedsDataUnitPayload])
        );
    }
}
