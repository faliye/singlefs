//! 模型对截断与改大小、截断后续批的回答。

use super::*;

impl IdealModel {
    /// 截断这次用户改动（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2、K3、K5、K11、K12；`mounted_session::UserChange::Truncate`
    /// 接线归下一批）：会话先走空（执行器把走空的那几步按「走空的一步」比过之后才问这一次），再接在现行那一版后面发：
    /// - N 按整单元向上取整溢出 u64：在任何写之前必须拒（[`ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow`]，K12）；
    /// - 现行那一版有树而文件集合为空：同覆盖写必须拒（[`ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile`]）；
    /// - N < S：发截断意图（[`ModelPublishKind::TruncationIntent`]），之后的各批是走空的一步（[`IdealModel::answer_truncation_step`]）；
    ///   N 落在存在的单元 k 中间、单元 k 的声明长度不等于 N − k × 净荷时重写它（K3）；不重写而上段叶条目的标签要变（缩短把尾部的洞整个截掉）
    ///   时意图那次只为换标签重写上段根；
    /// - N ≥ S：一次只改记录的发布（[`ModelPublishKind::SizeChangeWithoutShrinking`]；N = S 照 K12 发、不拒）：最后一个存在的单元声明长度小于
    ///   min(N − 起点, 净荷) 时补零重写它，不重写而洞标签要变时只重写 extent 上段根（K5）。
    ///
    /// 模型照字节记内容，N 要装得进这个进程的内存：执行器按单元的类取 N（缩到 0、单元边界、单元中间、单元内变长、跨单元变长、变长后截回洞里）。
    /// 第一个文件之外的 inode 改 size 由 [`IdealModel::answer_size_change_of_an_inode_other_than_the_first`] 答。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本没有带活文件的 inode 1（树表 0 条、删除中、截断途中、删完之后只建过 inode：执行器只在实现报带文件、队列空时调）：
    /// 会话状态与实现对不上；N 装不进这个进程的地址空间：[`ModelDisagreementAspect::NotModeled`]。
    pub fn answer_truncation(
        &self,
        new_size_in_bytes: u64,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let payload = data_unit_payload_capacity_in_bytes();
        if new_size_in_bytes
            .div_ceil(payload)
            .checked_mul(payload)
            .is_none()
        {
            return Ok(self.answer_for_publishes(
                ModelOperationKind::PublishTruncation,
                BTreeSet::from([ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow]),
                Vec::new(),
            ));
        }
        if let Some(refused) = self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(
            ModelOperationKind::PublishTruncation,
        ) {
            return Ok(refused);
        }
        let ModelVersionContents::LiveFile(file) = &current.contents else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有带活文件、队列空的 inode 1：{:?}",
                    current.key, current.contents
                ),
                "实现的现行版本带文件、队列空（执行器调了截断）".to_string(),
            ));
        };
        let size_before = u64::try_from(file.content.len()).expect("内容长度");
        let new_size = usize::try_from(new_size_in_bytes).map_err(|_| {
            ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!("模型照字节记内容，{new_size_in_bytes} 字节装不进这个进程的地址空间"),
                "执行器调了截断".to_string(),
            )
        })?;
        let checkpoint_txg = ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1);
        let journal_counter = ModelJournalCounter(self.highest_journal_counter.0 + 1);
        let root = if new_size_in_bytes < size_before {
            let data_units_within_the_new_size = data_units_of_content(new_size_in_bytes);
            let tail_data_unit = data_units_within_the_new_size - 1;
            let tail_data_unit_declared_length = new_size_in_bytes - tail_data_unit * payload;
            // 意图那次把目标态的 size、尾单元、标签一起写下（K2「先落目标」）：尾单元要重写就连上段根一起写；不重写而标签与
            // 目标态对不上（缩短把尾部的洞整个截掉，3、4 换回 1、2）就只为换标签重写上段根。size 之外还挂着的单元不算洞（S1），标签按
            // 存在的单元数与 N 罩的单元数取（[`ModelExtentUpperLeafEntryLabel::written_for`]）。
            let extent_tree = if tail_data_unit < file.data_units_present
                && file.declared_length_of_the_data_unit(tail_data_unit)
                    != tail_data_unit_declared_length
            {
                ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten
            } else if ModelExtentUpperLeafEntryLabel::written_for(
                file.data_units_present,
                data_units_within_the_new_size,
            ) != file.extent_upper_leaf_entry_label
            {
                ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
            } else {
                ModelExtentTreeChangeOfASizeChange::Untouched
            };
            self.next_root(
                current,
                session.instance,
                checkpoint_txg,
                journal_counter,
                ModelPublishKind::TruncationIntent { extent_tree },
                current.rollback_floor,
                Some(&file.content[..new_size]),
                Rc::clone(&current.instance_table_rows),
            )
        } else {
            let mut content = file.content.to_vec();
            content.resize(new_size, 0);
            let last_data_unit = file.data_units_present - 1;
            let last_data_unit_length_after =
                (new_size_in_bytes - last_data_unit * payload).min(payload);
            let extent_tree = if file.declared_length_of_the_data_unit(last_data_unit)
                < last_data_unit_length_after
            {
                ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten
            } else if ModelExtentUpperLeafEntryLabel::written_for(
                file.data_units_present,
                data_units_of_content(new_size_in_bytes),
            ) != file.extent_upper_leaf_entry_label
            {
                ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
            } else {
                ModelExtentTreeChangeOfASizeChange::Untouched
            };
            self.next_root(
                current,
                session.instance,
                checkpoint_txg,
                journal_counter,
                ModelPublishKind::SizeChangeWithoutShrinking { extent_tree },
                current.rollback_floor,
                Some(&content),
                Rc::clone(&current.instance_table_rows),
            )
        };
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishTruncation,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 第一个文件之外的一个 inode 改 size（`UserChange::Truncate` 点名的不是 inode 1；判决
    /// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2、S3、S6：任意活 inode 都能缩短与变长，删完第一个文件之后才建的
    /// inode 也照这一条办）：先判 N 按整单元向上取整溢不溢出（K12，与第一个文件同一个次序），再接在现行那一版后面发：
    /// - N < S：截断意图（[`ModelSizeChangeOfAnInodeWithoutDataUnits::TruncationIntent`]），之后的最后一批是走空的一步
    ///   （[`IdealModel::answer_truncation_step`]，放 0 个单元）；
    /// - N ≥ S：一次只改记录的发布；上段里还没有它那一条而 N > 0 时这次起写粘住的标签 0
    ///   （[`ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingWritingTheTagZeroEntry`]），否则 extent 树整棵照抄。
    ///
    /// 没有数据单元的那种读回 N 个 0；带数据单元的照第一个文件那一套（[`ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits`]：N < S 走截断意图、
    /// 落在单元 k 中间而声明长度不等于 N − k × 净荷时重写单元 k，N ≥ S 一次发布、最后一个存在的单元声明长度小于 min(N − 起点, 净荷) 时补零重写，
    /// 不重写而洞标签要变时只重写它那一串上段，判决 m4-multi-inode-size-r1 第二节 S3、S5）。
    ///
    /// # Errors
    /// 模型里没有可写会话、现行版本不是第一个文件活着或已删完而树里还有别的 inode 的一版、`inode` 不是这一版树里第一个文件之外活着的号：
    /// 会话状态与实现对不上；N 装不进这个进程的地址空间：[`ModelDisagreementAspect::NotModeled`]。
    pub fn answer_size_change_of_an_inode_other_than_the_first(
        &self,
        inode: u64,
        new_size_in_bytes: u64,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let payload = data_unit_payload_capacity_in_bytes();
        if new_size_in_bytes
            .div_ceil(payload)
            .checked_mul(payload)
            .is_none()
        {
            return Ok(self.answer_for_publishes(
                ModelOperationKind::PublishTruncation,
                BTreeSet::from([ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow]),
                Vec::new(),
            ));
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
                    format!("实现的现行版本树里有 inode {inode}（执行器点名它改 size）"),
                ));
            }
        };
        // 记录按号排，第 r 条在左起第 ⌊r ÷ 233⌋ 片叶容器里（同按 inode 号写，[`IdealModel::answer_whole_write_of_an_inode`]）。
        let Some((position_among_the_inodes_other_than_the_first, changed_inode)) = current
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
                format!("实现的现行版本树里有 inode {inode}（执行器点名它改 size）"),
            ));
        };
        if usize::try_from(new_size_in_bytes).is_err() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!("模型照字节记内容，{new_size_in_bytes} 字节装不进这个进程的地址空间"),
                format!("执行器点名 inode {inode} 改 size"),
            ));
        }
        let record_position_in_the_tree = first_file_records_in_the_tree
            + u64::try_from(position_among_the_inodes_other_than_the_first).expect("条数");
        let inode_leaf_container = ModelUnitRole::inode_leaf_container_at(
            record_position_in_the_tree / INODE_LEAF_RECORDS,
        );
        let size_before = u64::try_from(changed_inode.content.len()).expect("内容长度");
        if changed_inode.data_units_present() > 0 {
            let direction = if new_size_in_bytes < size_before {
                ModelDirectionOfASizeChange::TruncationIntent
            } else {
                ModelDirectionOfASizeChange::WithoutShrinking
            };
            let root = self.next_root(
                current,
                session.instance,
                ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
                ModelJournalCounter(self.highest_journal_counter.0 + 1),
                ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                    inode,
                    inode_leaf_container,
                    new_size_in_bytes,
                    direction,
                    extent_tree:
                        Self::extent_tree_change_of_a_size_change_of_an_inode_holding_data_units(
                            changed_inode,
                            new_size_in_bytes,
                            direction,
                        ),
                },
                current.rollback_floor,
                None,
                Rc::clone(&current.instance_table_rows),
            );
            return Ok(self.answer_for_publishes(
                ModelOperationKind::PublishTruncation,
                BTreeSet::new(),
                vec![root],
            ));
        }
        let change = if new_size_in_bytes < size_before {
            ModelSizeChangeOfAnInodeWithoutDataUnits::TruncationIntent
        } else if new_size_in_bytes > 0 && !changed_inode.keeps_a_tag_zero_upper_leaf_entry {
            ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingWritingTheTagZeroEntry
        } else {
            ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingCarryingTheExtentTree
        };
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits {
                inode,
                inode_leaf_container,
                new_size_in_bytes,
                change,
            },
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishTruncation,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 带数据单元的别的 inode 改 size 那一次对它的尾单元与上段叶条目做了什么（同第一个文件，[`IdealModel::answer_truncation`]；判决
    /// m4-multi-inode-size-r1 第二节 S3）：截断意图落在存在的单元 k 中间、单元 k 的声明长度不等于 N − k × 净荷时重写单元 k（K3），不缩短的那一次最后一个
    /// 存在的单元声明长度小于 min(N − 起点, 净荷) 时补零重写它（K5）；不重写而尾部有没有洞变了（标签要换，S1、K5）时只重写它那一串上段。
    /// 它那一条上段叶条目的标签恒是按存在的单元数与 size 写的那一个（[`ModelExtentUpperLeafEntryLabel::written_for`]：每一次改了单元数、尾单元或 size
    /// 罩的单元数的发布都重写它那一条），所以按发布前与发布后的 size 各取一个比。
    fn extent_tree_change_of_a_size_change_of_an_inode_holding_data_units(
        changed_inode: &ModelInodeOtherThanTheFirst,
        new_size_in_bytes: u64,
        direction: ModelDirectionOfASizeChange,
    ) -> ModelExtentTreeChangeOfASizeChange {
        let payload = data_unit_payload_capacity_in_bytes();
        let data_units_present = changed_inode.data_units_present();
        let data_units_within_the_new_size = data_units_of_content(new_size_in_bytes);
        let tail_data_unit_is_rewritten = match direction {
            ModelDirectionOfASizeChange::TruncationIntent => {
                let tail_data_unit = data_units_within_the_new_size - 1;
                tail_data_unit < data_units_present
                    && changed_inode.declared_length_of_the_data_unit(tail_data_unit)
                        != new_size_in_bytes - tail_data_unit * payload
            }
            ModelDirectionOfASizeChange::WithoutShrinking => {
                let last_data_unit = data_units_present - 1;
                changed_inode.declared_length_of_the_data_unit(last_data_unit)
                    < (new_size_in_bytes - last_data_unit * payload).min(payload)
            }
        };
        if tail_data_unit_is_rewritten {
            ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten
        } else if ModelExtentUpperLeafEntryLabel::written_for(
            data_units_present,
            data_units_within_the_new_size,
        ) != ModelExtentUpperLeafEntryLabel::written_for(
            data_units_present,
            changed_inode.data_units_within_the_size(),
        ) {
            ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
        } else {
            ModelExtentTreeChangeOfASizeChange::Untouched
        }
    }

    /// 走空截断的一步（会话里截断意图之后、每次用户改动之前，挂载续做）：size 之外还挂着的单元多于批大小 144（与 unlink
    /// 共用）就是不是最后一批的一批，从尾部放 144 个；不多于就是最后一批，放完剩下的（可以 0 个）、同一次发布删截断条目。下一步只由现行那一版定。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本不在截断途中：会话状态与实现对不上。
    pub fn answer_truncation_step(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        // 截断条目点名第一个文件之外的 inode（判决 m4-multi-inode-size-r1 第二节 S3）：没有数据单元的 size 之外一个单元都没有，下一步就是放 0 个的最后一批；
        // 带数据单元的同第一个文件，size 之外还挂着的多于批大小 144 就从尾部放 144 个，不多于就是最后一批（可以 0 个）。
        if let Some((inode, truncated_inode)) = current
            .inodes_other_than_the_first
            .iter()
            .find(|(_, other_inode)| other_inode.truncation_issued_txg.is_some())
            .map(|(number, other_inode)| (*number, other_inode))
        {
            let logged_operations_after = if current.logged_compaction_intent.is_some() {
                ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::StillHoldsTheCompactionIntent
            } else {
                ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::Emptied
            };
            let data_units_beyond_the_size = truncated_inode
                .data_units_present()
                .saturating_sub(truncated_inode.data_units_within_the_size());
            let step = if truncated_inode.data_units_present() == 0 {
                ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits {
                    inode,
                    logged_operations_after,
                }
            } else if data_units_beyond_the_size > MODEL_DELETION_BATCH_DATA_UNITS {
                ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { inode }
            } else {
                ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                    inode,
                    release: if data_units_beyond_the_size > 0 {
                        ModelFinalTruncationBatchRelease::AtLeastOneDataUnit
                    } else {
                        ModelFinalTruncationBatchRelease::NoDataUnit
                    },
                    logged_operations_after,
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
            return Ok(self.answer_for_publishes(
                ModelOperationKind::PublishTruncationStep,
                BTreeSet::new(),
                vec![root],
            ));
        }
        let ModelVersionContents::FileBeingTruncated { file, .. } = &current.contents else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 不在截断途中：{:?}",
                    current.key, current.contents
                ),
                "实现交回了走空截断的一步".to_string(),
            ));
        };
        // 截回洞里的（存在的单元不多于 size 罩的）size 之外一个都没有。
        let data_units_beyond_the_size = file
            .data_units_present
            .saturating_sub(file.data_units_within_the_size());
        let step = if data_units_beyond_the_size > MODEL_DELETION_BATCH_DATA_UNITS {
            ModelPublishKind::TruncationBatch
        } else {
            ModelPublishKind::FinalTruncationBatch {
                release: if data_units_beyond_the_size > 0 {
                    ModelFinalTruncationBatchRelease::AtLeastOneDataUnit
                } else {
                    ModelFinalTruncationBatchRelease::NoDataUnit
                },
                logged_operations_after: if current.logged_compaction_intent.is_some() {
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::StillHoldsTheCompactionIntent
                } else {
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::Emptied
                },
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
            ModelOperationKind::PublishTruncationStep,
            BTreeSet::new(),
            vec![root],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 这一版里分配代等于它自己那次发布的 txg 的角色（这次重写的），数据单元有一个是这次写的就算上 `Data`。
    fn roles_rewritten_by_the_publish(root: &ModelRoot) -> BTreeSet<ModelUnitRole> {
        root.role_written_at
            .iter()
            .filter(|(_, written_at)| **written_at == root.key.checkpoint_txg)
            .map(|(role, _)| *role)
            .chain(
                root.data_unit_written_at
                    .contains(&root.key.checkpoint_txg)
                    .then_some(ModelUnitRole::Data),
            )
            .collect()
    }

    /// 截到 0：300 个单元的文件，意图那次 size 写 0、单元 0 重写成声明长度 0、一个都不放；之后 299 个挂在 size 之外，
    /// 两批各从尾部放 144 个（不碰 inode 记录），最后一批放剩下的 11 个、删截断条目（logged_ops 删空），留一个单元、内联。
    #[test]
    fn truncating_to_zero_issues_the_intent_then_two_batches_of_144_then_the_final_batch_leaving_one_inline_unit(
    ) {
        let mut model = model_with_a_file_of(300 * payload() - 5);
        let intent = truncate_to(&mut model, 0);
        let ModelVersionContents::FileBeingTruncated {
            truncation_issued_txg,
            ..
        } = &intent.contents
        else {
            panic!("意图之后在截断途中：{:?}", intent.contents);
        };
        assert_eq!(*truncation_issued_txg, intent.key.checkpoint_txg);
        assert_eq!(file_of(&intent).content.len(), 0, "先落目标：读回 0 字节");
        assert_eq!(file_of(&intent).data_units_present, 300);
        assert_eq!(
            file_of(&intent).extent_upper_leaf_entry_label,
            ModelExtentUpperLeafEntryLabel::LowerSegmentRoot
        );
        assert_eq!(
            roles_rewritten_by_the_publish(&intent),
            BTreeSet::from([
                ModelUnitRole::Data,
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ])
        );
        assert_eq!(
            intent.data_unit_written_at[0], intent.key.checkpoint_txg,
            "单元 0 重写成声明长度 0"
        );
        assert_eq!(intent.data_unit_written_at[1], ModelCheckpointTxg(10));
        assert!(
            model.current_version_has_a_step_to_walk(),
            "截断进共用走空（K1）：下一次用户改动之前先把各批走完"
        );
        let batch_rewritten = BTreeSet::from([
            ModelUnitRole::ExtentRoot,
            ModelUnitRole::AllocationTree,
            ModelUnitRole::AccountingTree,
            ModelUnitRole::MappingTree,
            ModelUnitRole::TreeTable,
        ]);
        let first_batch = walk_one_step(&mut model);
        assert_eq!(file_of(&first_batch).data_units_present, 156);
        assert_eq!(
            roles_rewritten_by_the_publish(&first_batch),
            batch_rewritten
        );
        assert!(model.current_version_has_a_file_being_truncated());
        let second_batch = walk_one_step(&mut model);
        assert_eq!(file_of(&second_batch).data_units_present, 12);
        let final_batch = walk_one_step(&mut model);
        let ModelVersionContents::LiveFile(file) = &final_batch.contents else {
            panic!("最后一批之后是带活文件的一版：{:?}", final_batch.contents);
        };
        assert_eq!((file.content.len(), file.data_units_present), (0, 1));
        assert_eq!(
            file.extent_upper_leaf_entry_label,
            ModelExtentUpperLeafEntryLabel::InlineDataUnit
        );
        assert_eq!(
            roles_rewritten_by_the_publish(&final_batch),
            batch_rewritten
        );
        assert!(
            !final_batch
                .role_written_at
                .contains_key(&ModelUnitRole::LoggedOperationsRoot),
            "截断条目删掉、logged_ops 删空"
        );
        assert!(
            final_batch.extent_lower_node_written_at.is_empty(),
            "一个单元不建下段"
        );
        assert_eq!(
            final_batch.data_unit_written_at.as_slice(),
            [intent.key.checkpoint_txg]
        );
        assert!(!model.current_version_has_a_step_to_walk());
    }

    /// 截到单元中间：三个单元的文件截到 1.5 个净荷，意图那次重写单元 1（声明长度半个净荷）、不放单元；最后一批放单元 2、
    /// 不碰 inode 记录。之后建快照，码 14 抄的 inode 树根是意图那次写的（批只写 extent 树），不是最后一批。
    #[test]
    fn truncating_into_the_middle_of_a_unit_rewrites_that_unit_at_the_intent_and_the_final_batch_leaves_the_inode_tree_of_the_intent(
    ) {
        let mut model = model_with_a_file_of(2 * payload() + payload() / 2);
        let new_size = payload() + payload() / 2;
        let intent = truncate_to(&mut model, new_size);
        assert_eq!(
            file_of(&intent).content.as_ref(),
            &content_without_zeros(new_size)[..],
            "先落目标：读回截后那 N 字节"
        );
        assert_eq!(
            intent.data_unit_written_at.as_slice(),
            [
                ModelCheckpointTxg(10),
                intent.key.checkpoint_txg,
                ModelCheckpointTxg(10)
            ],
            "只重写单元 1，单元 2 还挂在 size 之外"
        );
        let final_batch = walk_one_step(&mut model);
        let file = file_of(&final_batch);
        assert_eq!(file.data_units_present, 2);
        assert_eq!(
            file.extent_upper_leaf_entry_label,
            ModelExtentUpperLeafEntryLabel::LowerSegmentRoot
        );
        assert_eq!(file.written_by, final_batch.key, "最后一批写了 extent 树");
        assert_eq!(
            file.inode_record_written_by, intent.key,
            "最后一批不碰 inode 记录"
        );
        assert_eq!(
            final_batch.role_written_at.get(&ModelUnitRole::InodeRoot),
            Some(&intent.key.checkpoint_txg)
        );
        assert!(!final_batch
            .role_written_at
            .contains_key(&ModelUnitRole::LoggedOperationsRoot));
        let snapshot = model.answer_create_snapshot().expect("会话开着、有树");
        succeed(&mut model, &snapshot);
        let snapshots = current_snapshots(&model);
        assert_eq!(
            snapshots[0].inode_tree_written_by, intent.key,
            "码 14 抄的 inode 树根是意图那次写的"
        );
    }

    /// 截到留下 144 个单元（「把罩着被重写单元 k 的那片下段叶到根恒并进重写集」，攻方 W3 打中 2 那一形）：150 个单元截到单元 143 中间，
    /// 意图那次重写单元 143 与罩着它的叶 0、下段根；最后一批放单元 144–149，留下 144 个、下段只剩叶 0，它的分配代是意图那次的（罩着重写的单元 143）。
    #[test]
    fn truncating_to_leave_144_units_keeps_the_leaf_the_intent_rewrote_for_the_tail_unit() {
        let mut model = model_with_a_file_of(150 * payload());
        let new_size = 143 * payload() + 7;
        let intent = truncate_to(&mut model, new_size);
        let leaf = |index_in_level| ModelExtentLowerNodePosition {
            level: 0,
            index_in_level,
        };
        let lower_root = ModelExtentLowerNodePosition {
            level: 1,
            index_in_level: 0,
        };
        assert_eq!(
            intent.extent_lower_node_written_at.as_ref(),
            &BTreeMap::from([
                (leaf(0), intent.key.checkpoint_txg),
                (leaf(1), ModelCheckpointTxg(10)),
                (lower_root, intent.key.checkpoint_txg),
            ]),
            "意图那次重写罩着单元 143 的叶 0 与下段根"
        );
        let final_batch = walk_one_step(&mut model);
        assert_eq!(file_of(&final_batch).data_units_present, 144);
        assert_eq!(
            final_batch.extent_lower_node_written_at.as_ref(),
            &BTreeMap::from([(leaf(0), intent.key.checkpoint_txg)]),
            "留下的叶 0 照抄意图那次写的"
        );
        assert_eq!(
            final_batch.data_unit_written_at[143],
            intent.key.checkpoint_txg
        );
        assert_eq!(
            final_batch.data_unit_written_at[142],
            ModelCheckpointTxg(10)
        );
    }

    /// 变长到单元内：100 字节的文件变长到 200 字节，一次发布做完：单元 0 补零重写到 200、标签照旧内联。
    #[test]
    fn growing_within_the_unit_rewrites_it_zero_padded_in_one_publish() {
        let mut model = model_with_a_file_of(100);
        let grown = truncate_to(&mut model, 200);
        let ModelVersionContents::LiveFile(file) = &grown.contents else {
            panic!("不缩短的截断一次发布做完：{:?}", grown.contents);
        };
        let mut expected = content_without_zeros(100);
        expected.resize(200, 0);
        assert_eq!(file.content.as_ref(), &expected[..]);
        assert_eq!(
            (file.data_units_present, file.extent_upper_leaf_entry_label),
            (1, ModelExtentUpperLeafEntryLabel::InlineDataUnit)
        );
        assert_eq!(
            roles_rewritten_by_the_publish(&grown),
            BTreeSet::from([
                ModelUnitRole::Data,
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ])
        );
        assert!(!model.current_version_has_a_step_to_walk(), "变长不立意图");
    }

    /// 变长跨单元：100 字节的文件变长到两个半净荷：单元 0 补零重写到满、之后到 size 是洞（标签 4），不写新单元。
    #[test]
    fn growing_across_units_fills_the_last_unit_and_declares_the_tail_hole() {
        let mut model = model_with_a_file_of(100);
        let new_size = 2 * payload() + 5;
        let grown = truncate_to(&mut model, new_size);
        let file = file_of(&grown);
        assert_eq!(file.content.len(), new_size);
        assert!(file.content[100..].iter().all(|byte| *byte == 0), "洞读 0");
        assert_eq!(
            (file.data_units_present, file.extent_upper_leaf_entry_label),
            (
                1,
                ModelExtentUpperLeafEntryLabel::InlineDataUnitThenTailHole
            )
        );
        assert_eq!(
            grown.data_unit_written_at.as_slice(),
            [grown.key.checkpoint_txg]
        );
        assert!(roles_rewritten_by_the_publish(&grown).contains(&ModelUnitRole::ExtentRoot));
    }

    /// 最后一个单元已满的文件变长：两个满单元变长一个字节，单元不动，只把上段叶条目的标签 1 换成 3（只重写 extent 上段根）。
    #[test]
    fn growing_a_file_whose_last_unit_is_full_only_relabels_the_upper_leaf_entry() {
        let mut model = model_with_a_file_of(2 * payload());
        let grown = truncate_to(&mut model, 2 * payload() + 1);
        assert_eq!(
            file_of(&grown).extent_upper_leaf_entry_label,
            ModelExtentUpperLeafEntryLabel::LowerSegmentRootThenTailHole
        );
        assert_eq!(
            roles_rewritten_by_the_publish(&grown),
            BTreeSet::from([
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ])
        );
        assert_eq!(
            grown.data_unit_written_at.as_slice(),
            [ModelCheckpointTxg(10), ModelCheckpointTxg(10)]
        );
    }

    /// 变长后截回洞里：100 字节变长到 5 个净荷（单元 0 补满、之后是洞），再截到 3 个净荷多 7 字节——落在洞里：
    /// 意图那次不重写单元、extent 树不动；最后一批没有单元可放（照走单一路径），也不动 extent 树，只删截断条目；标签仍是 4。
    #[test]
    fn truncating_back_into_the_hole_after_growing_issues_the_intent_and_a_final_batch_releasing_nothing(
    ) {
        let mut model = model_with_a_file_of(100);
        let grown = truncate_to(&mut model, 5 * payload());
        let new_size = 3 * payload() + 7;
        let intent = truncate_to(&mut model, new_size);
        assert_eq!(
            roles_rewritten_by_the_publish(&intent),
            BTreeSet::from([
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ])
        );
        assert!(
            model.current_version_has_a_file_being_truncated(),
            "缩短一律立意图"
        );
        let final_batch = walk_one_step(&mut model);
        assert_eq!(
            roles_rewritten_by_the_publish(&final_batch),
            BTreeSet::from([
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ])
        );
        let file = file_of(&final_batch);
        assert_eq!(file.content.as_ref(), &file_of(&grown).content[..new_size]);
        assert_eq!(
            (file.data_units_present, file.extent_upper_leaf_entry_label),
            (
                1,
                ModelExtentUpperLeafEntryLabel::InlineDataUnitThenTailHole
            )
        );
        assert_eq!(file.written_by, intent.key, "最后一批没写用户可见树");
    }

    /// 缩短把洞整个截掉、而尾单元不用重写、最后一批没有单元可放（变长到三个净荷之后截到恰好一个净荷）：换标签写在意图那次——
    /// 意图那次只为换标签重写 extent 上段根（单元不写），标签 4 换回 2；最后一批不动 extent 树、只删截断条目，标签照抄 2。
    #[test]
    fn cutting_the_whole_hole_at_the_end_of_the_last_unit_relabels_the_upper_leaf_entry_at_the_intent(
    ) {
        let mut model = model_with_a_file_of(100);
        let grown = truncate_to(&mut model, 3 * payload());
        assert_eq!(
            file_of(&grown).extent_upper_leaf_entry_label,
            ModelExtentUpperLeafEntryLabel::InlineDataUnitThenTailHole
        );
        let intent = truncate_to(&mut model, payload());
        assert_eq!(
            roles_rewritten_by_the_publish(&intent),
            BTreeSet::from([
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ]),
            "意图那次为换标签重写上段根、不写数据单元"
        );
        assert_eq!(
            file_of(&intent).extent_upper_leaf_entry_label,
            ModelExtentUpperLeafEntryLabel::InlineDataUnit,
            "标签在意图那次换回 2"
        );
        let final_batch = walk_one_step(&mut model);
        assert!(!roles_rewritten_by_the_publish(&final_batch).contains(&ModelUnitRole::ExtentRoot));
        assert_eq!(
            (
                file_of(&final_batch).data_units_present,
                file_of(&final_batch).extent_upper_leaf_entry_label
            ),
            (1, ModelExtentUpperLeafEntryLabel::InlineDataUnit)
        );
    }

    /// 三个满单元的文件变长到五个净荷（标签 3），再截到恰好一个净荷：尾单元 0 不用重写，size 之外挂着单元 1、2，洞整个截掉——
    /// 意图那次换标签（3 → 1：size 之外挂着的不算洞单元），之后最后一批放掉两个单元、单元只剩一个，标签照写成 2。
    #[test]
    fn cutting_the_whole_hole_and_two_data_units_relabels_at_the_intent_and_the_final_batch_inlines_the_last_unit(
    ) {
        let mut model = model_with_a_file_of(3 * payload());
        truncate_to(&mut model, 5 * payload());
        let intent = truncate_to(&mut model, payload());
        assert!(
            roles_rewritten_by_the_publish(&intent).contains(&ModelUnitRole::ExtentRoot)
                && !roles_rewritten_by_the_publish(&intent).contains(&ModelUnitRole::Data),
            "意图那次只为换标签重写上段根：{:?}",
            roles_rewritten_by_the_publish(&intent)
        );
        assert_eq!(
            (
                file_of(&intent).data_units_present,
                file_of(&intent).extent_upper_leaf_entry_label
            ),
            (3, ModelExtentUpperLeafEntryLabel::LowerSegmentRoot)
        );
        let final_batch = walk_one_step(&mut model);
        assert_eq!(
            (
                file_of(&final_batch).data_units_present,
                file_of(&final_batch).extent_upper_leaf_entry_label
            ),
            (1, ModelExtentUpperLeafEntryLabel::InlineDataUnit)
        );
    }

    /// 第一个文件删完之后建的 inode 改 size（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2、S3、S6：
    /// 删了「第一个文件之外改 size」那一拒）：N 按整单元向上取整溢出 u64 的先按溢出拒、一条根都不写；变长到 size > 0 发一次只改记录的发布、
    /// 这次起写粘住的标签 0（extent 上段根重写、上段有条目的号是 {2}）、读回 N 个 0；再变长只改记录、上段整棵照抄；截短发截断意图（logged_ops 根
    /// 重写、这一版在截断途中），走空的下一步是放 0 个单元的最后一批（删掉截断条目），标签 0 那一条照旧留着。
    #[test]
    fn size_change_of_an_inode_created_after_the_first_file_was_deleted_keeps_a_tag_zero_entry_and_truncates_through_an_intent(
    ) {
        let mut model = model_with_a_file_of(100);
        let unlink = model
            .answer_unlink_intent()
            .expect("会话开着、现行版本带活文件");
        succeed(&mut model, &unlink);
        let final_batch = model.answer_next_step_of_the_walk().expect("删除中");
        succeed(&mut model, &final_batch);
        let create = model.answer_create_inode().expect("删完之后建 inode");
        succeed(&mut model, &create);
        let overflow = model
            .answer_size_change_of_an_inode_other_than_the_first(2, u64::MAX)
            .expect("现行版本是删完之后建过 inode 的一版");
        assert_eq!(
            overflow.required_refusals,
            BTreeSet::from([ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow])
        );
        assert!(overflow.expected_roots.is_empty());
        let extension = model
            .answer_size_change_of_an_inode_other_than_the_first(2, 7)
            .expect("现行版本是删完之后建过 inode 的一版");
        succeed(&mut model, &extension);
        let extended = current_version(&model);
        assert!(
            roles_rewritten_by_the_publish(&extended).contains(&ModelUnitRole::ExtentRoot),
            "第一次写上标签 0：extent 上段根重写：{:?}",
            roles_rewritten_by_the_publish(&extended)
        );
        assert_eq!(
            extended.inodes_with_an_extent_upper_leaf_entry(),
            BTreeSet::from([2])
        );
        assert_eq!(
            extended.inodes_other_than_the_first[&2].content.as_ref(),
            &[0u8; 7][..]
        );
        let again = model
            .answer_size_change_of_an_inode_other_than_the_first(2, 9)
            .expect("现行版本是删完之后建过 inode 的一版");
        succeed(&mut model, &again);
        let extended_again = current_version(&model);
        assert!(
            !roles_rewritten_by_the_publish(&extended_again).contains(&ModelUnitRole::ExtentRoot),
            "标签 0 已经在：再变长只改记录、上段整棵照抄：{:?}",
            roles_rewritten_by_the_publish(&extended_again)
        );
        let intent = model
            .answer_size_change_of_an_inode_other_than_the_first(2, 3)
            .expect("现行版本是删完之后建过 inode 的一版");
        succeed(&mut model, &intent);
        let truncated = current_version(&model);
        assert!(
            roles_rewritten_by_the_publish(&truncated)
                .contains(&ModelUnitRole::LoggedOperationsRoot)
                && truncated.logged_truncation_issued_txg() == Some(truncated.key.checkpoint_txg),
            "截短发截断意图：logged_ops 根重写、截断条目点名这一版的 txg：{:?}",
            roles_rewritten_by_the_publish(&truncated)
        );
        assert!(model.current_version_has_a_file_being_truncated());
        let last_batch = walk_one_step(&mut model);
        assert_eq!(last_batch.logged_truncation_issued_txg(), None);
        assert!(!roles_rewritten_by_the_publish(&last_batch).contains(&ModelUnitRole::ExtentRoot));
        assert_eq!(
            (
                last_batch.inodes_with_an_extent_upper_leaf_entry(),
                last_batch.inodes_other_than_the_first[&2].content.len()
            ),
            (BTreeSet::from([2]), 3),
            "最后一批放 0 个单元、标签 0 那一条照旧留着，读回 3 个 0"
        );
    }

    /// N = S：发一次只改记录的发布，不拒、不动 extent 树。
    #[test]
    fn truncating_to_the_same_size_publishes_one_record_only_change() {
        let mut model = model_with_a_file_of(payload() + 3);
        let same = truncate_to(&mut model, payload() + 3);
        assert_eq!(
            roles_rewritten_by_the_publish(&same),
            BTreeSet::from([
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ])
        );
    }

    /// N 按整单元向上取整溢出 u64：在任何写之前必须拒，一条根都不写。
    #[test]
    fn truncation_size_whose_whole_data_units_overflow_is_refused_before_any_write() {
        let model = model_with_a_file_of(100);
        let answer = model.answer_truncation(u64::MAX).expect("会话开着");
        assert_eq!(
            answer.required_refusals,
            BTreeSet::from([ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow])
        );
        assert!(answer.expected_roots.is_empty());
    }

    /// 回退到截断意图之后、最后一批之前的那一版（logged_ops 里截断条目取回退目标那一版的）：现行那一版截断已做完、logged_ops 删空，
    /// 回退那次把目标的截断条目带回来、logged_ops 根这次重写；之后走空接着做截断的最后一批，再删空。
    #[test]
    fn rollback_to_a_version_in_the_middle_of_a_truncation_takes_its_truncation_entry_and_resumes_the_walk(
    ) {
        let mut model = model_with_a_file_of(2 * payload() + 9);
        let intent = truncate_to(&mut model, payload() + 1);
        walk_one_step(&mut model);
        let rollback = model
            .answer_rollback_while_mounted(intent.key)
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &rollback);
        let rolled_back = current_version(&model);
        assert_eq!(
            rolled_back.contents.logged_truncation_issued_txg(),
            Some(intent.key.checkpoint_txg)
        );
        assert_eq!(
            rolled_back
                .role_written_at
                .get(&ModelUnitRole::LoggedOperationsRoot),
            Some(&rolled_back.key.checkpoint_txg),
            "logged_ops 换成带截断条目的那一份"
        );
        assert_eq!(
            rolled_back.data_unit_written_at,
            intent.data_unit_written_at
        );
        let final_batch = walk_one_step(&mut model);
        assert_eq!(file_of(&final_batch).data_units_present, 2);
        assert!(!final_batch
            .role_written_at
            .contains_key(&ModelUnitRole::LoggedOperationsRoot));
    }

    /// 快照下截断再整份覆盖写（那段历史）：顺序写三个单元（txg 10）→ 建快照（11）→ 截到 1.5 个净荷，单元 1 重写（12）
    /// → 最后一批放单元 2（13）→ 快照下整份覆盖写（14）。覆盖写换下的单元 0 出生 10、不晚于快照，进 deadlist；单元 1 出生 12、晚于快照，直接释放。
    #[test]
    fn an_overwrite_under_a_snapshot_after_a_truncation_pins_unit_zero_and_releases_the_unit_the_truncation_rewrote(
    ) {
        let mut model = model_with_a_file_of(2 * payload() + payload() / 2);
        let snapshot = model.answer_create_snapshot().expect("会话开着、有树");
        succeed(&mut model, &snapshot);
        let intent = truncate_to(&mut model, payload() + payload() / 2);
        assert_eq!(intent.key.checkpoint_txg, ModelCheckpointTxg(12));
        let final_batch = walk_one_step(&mut model);
        let overwrite = model
            .answer_publish_sequential_write(&content_without_zeros(2 * payload()))
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &overwrite);
        let after_the_overwrite = current_version(&model);
        let data_unit_births = |root: &ModelRoot| {
            let mut births: Vec<ModelCheckpointTxg> = root
                .deadlist
                .iter()
                .filter(|entry| entry.mapping_key_unit_class == MODEL_MAPPING_KEY_UNIT_CLASS_DATA)
                .map(|entry| entry.birth_txg)
                .collect();
            births.sort_unstable();
            births
        };
        assert_eq!(
            data_unit_births(&final_batch),
            vec![ModelCheckpointTxg(10), ModelCheckpointTxg(10)],
            "截断换下的旧单元 1 与放掉的单元 2 都出生在快照之前"
        );
        assert_eq!(
            data_unit_births(&after_the_overwrite),
            vec![
                ModelCheckpointTxg(10),
                ModelCheckpointTxg(10),
                ModelCheckpointTxg(10)
            ],
            "覆盖写只把单元 0 记进 deadlist，单元 1（出生 12）直接释放"
        );
    }

    /// 带数据单元的别的 inode 截短（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3、S5）：第一个文件旁边建 inode 2、
    /// 按号写三个满单元，截到 100 字节：意图那次单元 0 重写（分配代是这次的）、三个单元都还挂着、截断字段写成（这次 txg, 100）、截断条目点名它；
    /// 走空的下一步是放掉单元 1、2 的最后一批（删掉截断条目，剩一个单元）。逐版比时别的 inode 的截断字段与模型记的不同就报对不上
    /// （截断字段按 inode 比）。判别力：走空截断的一步把别的 inode 当没有单元（`crates/mutations.tsv` 一行）时最后一批照放 0 个、单元数断言红。
    #[test]
    fn truncating_another_inode_holding_data_units_rewrites_its_tail_unit_writes_its_truncation_field_and_releases_the_units_beyond(
    ) {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model.answer_publish_first_file(&[3]).expect("会话开着");
        succeed(&mut model, &first);
        let created = model
            .answer_create_inodes(1)
            .expect("第一个文件旁边建 inode 2");
        succeed(&mut model, &created);
        let payload = data_unit_payload_capacity_in_bytes();
        let content = vec![5_u8; usize::try_from(3 * payload).expect("长度")];
        let written = model
            .answer_sequential_write_of_an_inode(2, &content)
            .expect("按号写 inode 2 三个满单元");
        succeed(&mut model, &written);
        let intent = model
            .answer_size_change_of_an_inode_other_than_the_first(2, 100)
            .expect("带数据单元的别的 inode 截到 100");
        let intent_root = intent.expected_roots[0].clone();
        let truncated = &intent_root.inodes_other_than_the_first[&2];
        assert_eq!(
            (
                truncated.content.len(),
                truncated.data_unit_written_at.as_slice(),
                truncated.truncation_field,
                truncated.truncation_issued_txg,
            ),
            (
                100,
                [
                    intent_root.key.checkpoint_txg,
                    written.expected_roots[0].key.checkpoint_txg,
                    written.expected_roots[0].key.checkpoint_txg,
                ]
                .as_slice(),
                ModelTruncationField::WrittenByATruncationIntent {
                    truncation_txg: intent_root.key.checkpoint_txg,
                    size_in_bytes: 100,
                },
                Some(intent_root.key.checkpoint_txg),
            ),
            "意图那次：读回 100 字节、单元 0 重写、三个单元都挂着、截断字段写成（这次 txg, 100）、截断条目点名它"
        );
        let mut counts = ModelJudgementCounts::default();
        let mut observed = observed_exactly_as_the_model_wrote(&intent_root);
        assert_eq!(
            model.judge_roots(
                std::slice::from_ref(&intent_root),
                std::slice::from_ref(&observed),
                &mut counts
            ),
            Ok(())
        );
        assert_eq!(
            (
                counts.truncation_fields_of_the_inodes_other_than_the_first_compared,
                counts.written_truncation_fields_of_the_inodes_other_than_the_first_compared,
            ),
            (1, 1),
            "inode 2 的截断字段比过、写过"
        );
        observed
            .truncation_fields_of_the_inodes_other_than_the_first
            .insert(
                2,
                ObservedTruncationField {
                    truncation_txg: ModelCheckpointTxg(0),
                    size_in_bytes: 0,
                },
            );
        let wrong = model
            .judge_roots(
                std::slice::from_ref(&intent_root),
                std::slice::from_ref(&observed),
                &mut counts,
            )
            .expect_err("inode 2 的截断字段交成没写过");
        assert_eq!(
            wrong.aspect,
            ModelDisagreementAspect::InodesOtherThanTheFirst
        );
        succeed(&mut model, &intent);
        assert!(model.current_version_has_a_file_being_truncated());
        let final_batch = model
            .answer_next_step_of_the_walk()
            .expect("走空截断的下一步");
        let after = &final_batch.expected_roots[0];
        assert_eq!(
            (
                after.inodes_other_than_the_first[&2].data_units_present(),
                after.logged_truncation_issued_txg(),
                after.role_written_at.get(&ModelUnitRole::ExtentRoot),
            ),
            (1, None, Some(&after.key.checkpoint_txg)),
            "最后一批放掉单元 1、2（剩一个，内联）、删掉截断条目、上段它那一串重写"
        );
    }
}
