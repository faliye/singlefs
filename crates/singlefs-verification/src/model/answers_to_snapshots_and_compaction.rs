//! 模型对建快照、删快照、读快照、整理发布、用户改动之后的搬迁批与走空的一步的回答。

use super::*;

impl IdealModel {
    /// 建一个只读快照（`mounted_session::UserChange::CreateSnapshot`，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S2）：
    /// 会话先走空（执行器把走空的那几步按「走空的一步」比过之后才问这一次），再接在现行那一版后面发一次独立发布。现行那一版有树而文件集合为空时
    /// 照建，快照读出「没有文件」（「放开」，判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F5）；
    /// 已经有 10 个快照时必须拒（第七节岔路 1）。逐次发布判墙。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本树表 0 条（执行器只在有树的一版上调）：会话状态与实现对不上。
    pub fn answer_create_snapshot(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if !current.contents.has_trees() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!("模型里现行版本 {:?} 树表 0 条", current.key),
                "实现的现行版本有树（执行器调了建快照）".to_string(),
            ));
        }
        let mut required_refusals = BTreeSet::new();
        if current.snapshots.len() >= MODEL_SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {
            required_refusals.insert(ModelRefusalReason::MoreSnapshotsThanTheFirstVersionAllows);
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::SnapshotCreation,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishSnapshotCreation,
            required_refusals,
            vec![root],
        ))
    }

    /// 删按建的先后第 `position` 个快照的意图那次（`mounted_session::UserChange::DestroySnapshot` 与第七节岔路 2、4）：
    /// 会话先走空，再接在现行那一版后面发意图；之后的每一批是走空的一步（[`IdealModel::answer_snapshot_destruction_step`]）。
    /// 现行那一版有树而文件集合为空时照样删（删快照不依赖活头里有没有文件，D8（核心索引结构） 已定项 16 第 5 条那一句管的是写文件的改动；
    /// 丙 2 交判第 5 条主 agent 定是缺陷，会话放行），与建快照相同（判决 `research/prompts/m4-segment-code-r1-main-verification.md`
    /// 第三节 F5）。逐次发布判墙。
    ///
    /// # Errors
    /// 模型里没有可写会话、现行版本树表 0 条、没有第 `position` 个快照、或现行那一版已有快照在销毁中（会话先把它走完，走到这里说明模型与实现
    /// 对不上）：会话状态与实现对不上。
    pub fn answer_destroy_snapshot(
        &self,
        position: usize,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if !current.contents.has_trees()
            || position >= current.snapshots.len()
            || current
                .snapshots
                .iter()
                .any(|snapshot| snapshot.is_being_destroyed)
        {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有能删的第 {position} 个快照：{:?}",
                    current.key, current.snapshots
                ),
                "实现的现行版本里有这个快照、没有销毁中的（执行器调了删快照）".to_string(),
            ));
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::SnapshotDestructionIntent { position },
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishSnapshotDestructionIntent,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 走空销毁中那个快照的一步（判决 S4：会话里删快照的意图之后、每次用户改动之前，挂载续做）：它那一段这一批能放的条目多于一批（144 条）
    /// 就是不是最后一批的一批，放 144 条；不多于就是最后一批，放剩下的、删掉它。下一步只由现行那一版定。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本没有销毁中的快照：会话状态与实现对不上。
    pub fn answer_snapshot_destruction_step(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let Some(position) = current
            .snapshots
            .iter()
            .position(|snapshot| snapshot.is_being_destroyed)
        else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有销毁中的快照：{:?}",
                    current.key, current.snapshots
                ),
                "实现交回了走空销毁中快照的一步".to_string(),
            ));
        };
        let checkpoint_txg = ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1);
        let releasable = u64::try_from(
            Self::deadlist_entries_the_destruction_releases(current, position, checkpoint_txg)
                .len(),
        )
        .expect("条数");
        let step = if releasable > MODEL_SNAPSHOT_DESTRUCTION_BATCH_ENTRIES {
            ModelPublishKind::SnapshotDestructionBatch {
                position,
                entries_released: MODEL_SNAPSHOT_DESTRUCTION_BATCH_ENTRIES,
            }
        } else {
            ModelPublishKind::SnapshotDestructionFinalBatch { position }
        };
        let root = self.next_root(
            current,
            session.instance,
            checkpoint_txg,
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            step,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishSnapshotDestructionStep,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 走空的下一步（`transaction::walk_the_deletion_queue_until_it_is_empty`：`deleted_inodes` 先走空，再走销毁中的快照，再走截断，判决
    /// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1 的次序）：现行那一版有删除中的文件就是删除的一步
    /// （[`IdealModel::answer_deletion_step`]），否则有销毁中的快照就是删快照的一批（[`IdealModel::answer_snapshot_destruction_step`]），
    /// 否则在截断途中就是截断的一批（[`IdealModel::answer_truncation_step`]）。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本三样都没有：会话状态与实现对不上。
    pub fn answer_next_step_of_the_walk(&self) -> Result<ModelAnswer, ModelDisagreement> {
        if self.current_version_has_a_file_being_deleted() {
            self.answer_deletion_step()
        } else if self.current_version_has_a_snapshot_being_destroyed()
            || !self.current_version_has_a_file_being_truncated()
        {
            self.answer_snapshot_destruction_step()
        } else {
            self.answer_truncation_step()
        }
    }

    /// 整理的一次发布（`mounted_session::MountedSession::run_compaction` 发成的一版，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节
    /// G1、G3、G4、G7；丁 3 交回的读法第 1–3 条）：接在现行那一版后面，txg 与 jsn 各加一、一个事务、F 照抄，只重写固定点单元，用户可见的内容、
    /// 快照、deadlist、每个搬走的单元的分配代都与上一版相同。是哪一种由模型记的活意图与执行器交来的这一版看得见的几样（`seen`）定：
    /// - 模型这一版没有活意图：只能是签发（不搬单元、发布之后 logged_ops 里有一条）——R 取执行器强制的那一段、`c0` 取上一版的 txg、
    ///   签发 txg 取这一次的（[`ModelPublishKind::CompactionIntentIssue`]）；别的样子都对不上（全池至多一条，先签发才搬）。
    /// - 有活意图、搬了单元：发布之后意图还在就是照抄意图的一批，这时 R 按记录还不能全空（搬空了 R 就要同一次发布删意图，D8（核心索引结构）
    ///   已定项 5）；意图没了就是最后一批同发布删意图，这时 R 要按记录全空（判决 G4 停机谓词「删意图那一刻 R 按记录全空」）。
    /// - 有活意图、不搬单元：意图要没了（单独删：R 按记录全空是达成，不空是丢），意图还在就对不上（这一次什么都没做）。照抄意图的一批至少搬一个
    ///   R 里真要搬的单元（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H4「规划时补足」）：计划跳过这次
    ///   发布本来就重写的节点，一个都没搬而照抄意图的那一次不许发；R 里只剩这次发布本来就重写的节点时那一次不搬单元、同一次发布删意图，发布之后
    ///   R 按记录全空，就是意图达成那一种。
    ///
    /// 搬的里有用户可见树的根单元（`seen.user_visible_root_units_relocated`）时把这一次记进那几个单元的搬迁 txg，这一版算一个新状态
    /// （[`ModelRoot::user_visible_root_relocations`]）；它们要是这一版引用着的。模型不答推几次、停在哪（判决 G7「只比成功操作的内容」）：
    /// 执行器按实现交回的每一版各问一次。先走空的删除续做由 [`IdealModel::answer_next_step_of_the_walk`] 答。逐次发布判墙。
    ///
    /// # Errors
    /// 模型里没有可写会话、现行版本树表 0 条、现行那一版还有要走空的：会话状态对不上；这一版对意图做的事与上面那几条对不上：
    /// [`ModelDisagreementAspect::CompactionIntentOfThePublish`]；搬的根单元模型这一版不引用：[`ModelDisagreementAspect::NotModeled`]。
    pub fn answer_compaction_publish(
        &self,
        seen: &ModelCompactionPublishSeen,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if !current.contents.has_trees() || self.current_version_has_a_step_to_walk() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 不是能整理的一版（树表 0 条或还有要走空的）：{:?}",
                    current.key, current.contents
                ),
                "实现交回了整理的一次发布".to_string(),
            ));
        }
        let checkpoint_txg = ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1);
        let does_the_publish_relocate_units = seen.units_relocated > 0;
        let refused_step = |model_answer: String| {
            Err(ModelDisagreement::new(
                ModelDisagreementAspect::CompactionIntentOfThePublish,
                format!("{:?} 之后的这一次整理发布：{model_answer}", current.key),
                describe_compaction_publish_seen(seen),
            ))
        };
        let (kind, operation, issued) = match current.logged_compaction_intent {
            None => {
                if does_the_publish_relocate_units || !seen.is_an_intent_logged_after_the_publish {
                    return refused_step(
                        "这一版没有活整理意图：整理只能先签发（只把意图写进 logged_ops、不搬单元，判决 G1）".to_string(),
                    );
                }
                (
                    ModelPublishKind::CompactionIntentIssue,
                    ModelOperationKind::PublishCompactionIntentIssue,
                    Some(ModelCompactionIntent {
                        issued_txg: checkpoint_txg,
                        region: seen.region_forced_for_the_issue,
                        allocation_generation_at_issue: current.key.checkpoint_txg,
                    }),
                )
            }
            Some(live) => {
                let is_the_region_empty =
                    seen.is_the_region_of_the_live_intent_empty_by_the_records_after_the_publish;
                match (
                    does_the_publish_relocate_units,
                    seen.is_an_intent_logged_after_the_publish,
                    is_the_region_empty,
                ) {
                    (true, true, false) => (
                        ModelPublishKind::CompactionBatch {
                            units_at_most: seen.units_at_most,
                            intent_after_the_batch: ModelCompactionIntentAfterTheBatch::CarriedOn,
                            instance_table: ModelInstanceTableOfTheBatch::Carried,
                        },
                        ModelOperationKind::PublishCompactionBatch,
                        None,
                    ),
                    (true, true, true) => {
                        return refused_step(format!(
                            "这一批搬空了活意图的 R {:?}（按记录）：意图要与最后一批同一次发布删（D8（核心索引结构） 已定项 5）",
                            live.region
                        ));
                    }
                    (true, false, true) => (
                        ModelPublishKind::CompactionBatch {
                            units_at_most: seen.units_at_most,
                            intent_after_the_batch:
                                ModelCompactionIntentAfterTheBatch::DeletedWithTheLastBatch,
                            instance_table: ModelInstanceTableOfTheBatch::Carried,
                        },
                        ModelOperationKind::PublishCompactionBatch,
                        None,
                    ),
                    (true, false, false) => {
                        return refused_step(format!(
                            "删意图那一刻活意图的 R {:?} 要按记录全空（判决 G4 停机谓词）",
                            live.region
                        ));
                    }
                    (false, false, true) => (
                        ModelPublishKind::CompactionIntentDeletion {
                            ending:
                                ModelCompactionIntentEnding::CompletedWithTheRegionEmptyByTheRecords,
                        },
                        ModelOperationKind::PublishCompactionIntentCompletion,
                        None,
                    ),
                    (false, false, false) => (
                        ModelPublishKind::CompactionIntentDeletion {
                            ending: ModelCompactionIntentEnding::Dropped,
                        },
                        ModelOperationKind::PublishCompactionIntentDrop,
                        None,
                    ),
                    (false, true, true | false) => {
                        return refused_step(format!(
                            "活意图 {live:?} 在：一次整理发布要么搬一批（至少搬一个 R 里真要搬的单元，判决 H4）、要么删掉意图"
                        ));
                    }
                }
            }
        };
        self.answer_of_a_publish_relocating_units(
            kind,
            operation,
            issued,
            &seen.user_visible_root_units_relocated,
        )
    }

    /// 用户改动发成之后另发的那一批（`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated`，
    /// D4（校验和位置） 已定项 4「可写挂载里读到的，再经已有的搬迁路径把这个单元整个搬到新槽、两份新写」）：接在现行那一版后面的一次搬迁发布，
    /// 意图照抄（`transaction::publish_the_relocations_and_the_repair_of_the_exempt_units`：意图的签发与删不归这一批），所以不问活意图、也不问还有没有要走空的，
    /// 与 [`Self::answer_compaction_publish`] 分开答。这一批不是整理：现行那一版没有活意图时一样发。只重写固定点单元，文件内容逐项照抄；
    /// 换实例表时行不变。模型不答这一批发不发（模型不记盘上读到哪一份坏了），执行器按实现交回的那一版问。
    ///
    /// # Errors
    /// 模型里没有可写会话：会话状态对不上；搬的根单元模型这一版不引用：[`ModelDisagreementAspect::NotModeled`]。
    pub fn answer_relocation_batch_after_a_user_change(
        &self,
        seen: &ModelRelocationBatchAfterAUserChangeSeen,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        self.answer_of_a_publish_relocating_units(
            ModelPublishKind::CompactionBatch {
                units_at_most: seen.units_relocated,
                intent_after_the_batch: ModelCompactionIntentAfterTheBatch::CarriedOn,
                instance_table: seen.instance_table,
            },
            ModelOperationKind::PublishRelocationBatchAfterAUserChange,
            None,
            &seen.user_visible_root_units_relocated,
        )
    }

    /// 接在现行那一版后面的一次搬迁发布（整理的那几种与另发的那一批）：搬的根单元要是这一版引用着的；照 `kind` 算下一版，`issued` 是这次签发的意图，
    /// 搬的根单元记进搬迁 txg（[`ModelRoot::user_visible_root_relocations`]）。
    fn answer_of_a_publish_relocating_units(
        &self,
        kind: ModelPublishKind,
        operation: ModelOperationKind,
        issued: Option<ModelCompactionIntent>,
        user_visible_root_units_relocated: &[ModelUserVisibleRootUnit],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let checkpoint_txg = ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1);
        let referenced: BTreeSet<ModelUserVisibleRootUnit> =
            current.user_visible_root_units().into_iter().collect();
        if let Some(unreferenced) = user_visible_root_units_relocated
            .iter()
            .find(|unit| !referenced.contains(unit))
        {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!("{:?} 引用着的用户可见树根单元 {referenced:?}", current.key),
                format!("实现搬了根单元 {unreferenced:?}"),
            ));
        }
        let mut root = self.next_root(
            current,
            session.instance,
            checkpoint_txg,
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            kind,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        if let Some(intent) = issued {
            root.logged_compaction_intent = Some(intent);
        }
        if !user_visible_root_units_relocated.is_empty() {
            let mut relocations = root.user_visible_root_relocations.as_ref().clone();
            for unit in user_visible_root_units_relocated {
                relocations.insert(*unit, checkpoint_txg);
            }
            root.user_visible_root_relocations = Rc::new(relocations);
        }
        Ok(self.answer_for_publishes(operation, BTreeSet::new(), vec![root]))
    }

    /// 可写会话现行那一版 logged_ops 里那一条活整理意图（没有可写会话或没有活意图时 None）：执行器拿它的 R 判实现交回的那一版 R 按记录全空没有。
    #[must_use]
    pub fn live_compaction_intent_of_the_current_version(&self) -> Option<ModelCompactionIntent> {
        self.session
            .as_ref()
            .and_then(|session| session.current.logged_compaction_intent)
    }

    /// 模型的现行版本还有没有要走空的（删除中的文件、销毁中的快照，或截断途中的文件，判决
    /// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1「截断进共用走空」）：执行器拿它分一次会话交回的那几版里哪一版是走空的一步。
    #[must_use]
    pub fn current_version_has_a_step_to_walk(&self) -> bool {
        self.current_version_has_a_file_being_deleted()
            || self.current_version_has_a_snapshot_being_destroyed()
            || self.current_version_has_a_file_being_truncated()
    }

    /// 模型的现行版本有没有销毁中的快照。
    fn current_version_has_a_snapshot_being_destroyed(&self) -> bool {
        self.session.as_ref().is_some_and(|session| {
            session
                .current
                .snapshots
                .iter()
                .any(|snapshot| snapshot.is_being_destroyed)
        })
    }

    /// 模型的现行版本是不是截断途中（logged_ops 里有一条截断条目）。
    #[must_use]
    pub fn current_version_has_a_file_being_truncated(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.current.logged_truncation_issued_txg().is_some())
    }

    /// 读按建的先后第 `position` 个快照（`recovery::read_the_first_file_of_a_snapshot` 经现行那一版的根「读快照」）：
    /// 读出的要是建它那一刻读得出的文件内容（那一刻没有活文件就是「没有文件」）。读不写盘，模型不动。
    ///
    /// # Errors
    /// 模型里没有可写会话、没有第 `position` 个快照：会话状态与实现对不上；读出的与模型记的不同：读快照那一格对不上。
    pub fn judge_snapshot_read_back(
        &self,
        position: usize,
        observed: &ObservedSnapshotReadBack,
    ) -> Result<ModelJudgementCounts, ModelDisagreement> {
        let session = self.open_session()?;
        let Some(snapshot) = session.current.snapshots.get(position) else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有第 {position} 个快照：{:?}",
                    session.current.key, session.current.snapshots
                ),
                "实现的现行版本里有这个快照（执行器读了它）".to_string(),
            ));
        };
        let expected = match &snapshot.content_at_creation {
            Some(content) => ModelSnapshotReadBack::File(Rc::clone(content)),
            None => ModelSnapshotReadBack::NoFile,
        };
        let matches = match (&expected, observed) {
            (ModelSnapshotReadBack::NoFile, ObservedSnapshotReadBack::NoFile) => true,
            (
                ModelSnapshotReadBack::File(expected_content),
                ObservedSnapshotReadBack::File(content),
            ) => expected_content.as_ref() == content.as_slice(),
            (
                ModelSnapshotReadBack::NoFile | ModelSnapshotReadBack::File(_),
                ObservedSnapshotReadBack::NoFile
                | ObservedSnapshotReadBack::File(_)
                | ObservedSnapshotReadBack::NoSnapshotWithThisTree
                | ObservedSnapshotReadBack::Failed { .. },
            ) => false,
        };
        if !matches {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SnapshotReadBack,
                format!(
                    "第 {position} 个快照（诞生 txg {}）读出 {}",
                    snapshot.birth_txg.0,
                    describe_snapshot_read_back(&expected)
                ),
                describe_observed_snapshot_read_back(observed),
            ));
        }
        let mut counts = ModelJudgementCounts::default();
        counts.snapshot_read_backs_compared += 1;
        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 建快照进根环（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S2、S7）：会话现行那一版带文件（内容 [9]，txg 9），
    /// 连建十个，每个是一次独立发布（txg 10 到 19）、各占一个事务；第 k 个的前驱是第 k − 1 个的诞生 txg（第一个是 0），活头前驱是最后一个的；
    /// 第十一个要求拒成 `MoreSnapshotsThanTheFirstVersionAllows`（第七节岔路 1 的「10」）。之后覆盖写成 [10]，记下的快照内容不跟着变。
    #[test]
    fn the_model_publishes_ten_snapshots_into_the_ring_and_requires_the_eleventh_to_be_refused() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        for snapshot_index in 0..MODEL_SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {
            let creation = model
                .answer_create_snapshot()
                .expect("会话开着、现行那一版带文件");
            assert!(
                creation.required_refusals.is_empty(),
                "第 {snapshot_index} 个照建：{:?}",
                creation.required_refusals
            );
            succeed(&mut model, &creation);
        }
        let eleventh = model.answer_create_snapshot().expect("会话开着");
        assert_eq!(
            eleventh.required_refusals,
            BTreeSet::from([ModelRefusalReason::MoreSnapshotsThanTheFirstVersionAllows]),
            "第十一个拒"
        );
        let snapshots = current_snapshots(&model);
        assert_eq!(
            snapshots.len(),
            MODEL_SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION
        );
        for (snapshot, birth_txg) in snapshots.iter().zip(10_u64..) {
            assert_eq!(
                snapshot.birth_txg,
                ModelCheckpointTxg(birth_txg),
                "S.txg 是建它那次发布的 txg"
            );
            assert_eq!(
                snapshot.previous_snapshot_txg,
                ModelCheckpointTxg(if birth_txg == 10 { 0 } else { birth_txg - 1 }),
                "前驱是排在它前面那一个的诞生 txg"
            );
            assert_eq!(
                snapshot.inode_tree_written_by,
                key(9, 2),
                "两条根指针抄的是 txg 9 那一版的"
            );
        }
        let current = model.current_version_in_the_session().expect("会话开着");
        assert_eq!(
            current.previous_snapshot_txg_of_the_live_head,
            ModelCheckpointTxg(19),
            "活头前驱是最后一个快照的诞生 txg"
        );
        assert_eq!(
            current.highest_transaction_number_in_its_instance,
            4 + 10,
            "四次覆盖写各一个事务，十次建快照各一个事务"
        );
        let overwrite = model
            .answer_publish_overwrite(&[10])
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &overwrite);
        assert_eq!(
            current_snapshots(&model)
                .iter()
                .map(|snapshot| snapshot.content_at_creation.clone())
                .collect::<Vec<_>>(),
            vec![Some(Rc::from(vec![9_u8])); MODEL_SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION],
            "十个快照记的都是建时的内容 [9]，之后的覆盖写不改它们"
        );
    }

    /// 快照下覆盖写钉住被换下的用户可见单元、删快照放掉它们：快照建在 txg 10（内容 [9] 是 txg 9 写的），txg 11 覆盖写换下
    /// 数据单元、extent 根、inode 叶与根四个（都生在 txg 9 ≤ 10），四条 deadlist 条目、四条被钉的映射条目，纪元 10；读快照仍读出 [9]。
    /// 删它：意图那次（txg 12）带上销毁中、把前驱 0 传给活头，一条都不放；下一批（txg 13）能放的四条不多于一批，是最后一批，放掉四条、删掉快照。
    #[test]
    fn the_model_pins_the_units_a_snapshot_references_and_releases_them_when_the_snapshot_is_destroyed(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let creation = model.answer_create_snapshot().expect("建快照");
        succeed(&mut model, &creation);
        let overwrite = model.answer_publish_overwrite(&[10]).expect("覆盖写");
        succeed(&mut model, &overwrite);
        let after_the_overwrite = model
            .current_version_in_the_session()
            .expect("会话开着")
            .clone();
        assert_eq!(
            after_the_overwrite.deadlist.as_ref(),
            &[
                ModelDeadlistEntry {
                    epoch: ModelCheckpointTxg(10),
                    mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_DATA,
                    mapping_key_birth_tree: TREE_IDENTIFIER_EXTENT,
                    birth_txg: ModelCheckpointTxg(9),
                },
                ModelDeadlistEntry {
                    epoch: ModelCheckpointTxg(10),
                    mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_INDEX_NODE,
                    mapping_key_birth_tree: TREE_IDENTIFIER_EXTENT,
                    birth_txg: ModelCheckpointTxg(9),
                },
                ModelDeadlistEntry {
                    epoch: ModelCheckpointTxg(10),
                    mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_PACKED_RECORDS,
                    mapping_key_birth_tree: TREE_IDENTIFIER_INODE,
                    birth_txg: ModelCheckpointTxg(9),
                },
                ModelDeadlistEntry {
                    epoch: ModelCheckpointTxg(10),
                    mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_INDEX_NODE,
                    mapping_key_birth_tree: TREE_IDENTIFIER_INODE,
                    birth_txg: ModelCheckpointTxg(9),
                },
            ],
            "数据单元、extent 根、inode 叶与根各一条，纪元是活头前驱 10（数据单元先进，叶容器与根按角色次序）"
        );
        assert_eq!(
            after_the_overwrite.pinned_mapping_entries, 4,
            "四条映射条目留着"
        );
        assert_eq!(
            model
                .judge_snapshot_read_back(0, &ObservedSnapshotReadBack::File(vec![9]))
                .map(|counts| counts.snapshot_read_backs_compared),
            Ok(1),
            "覆盖写之后快照仍读出建时的 [9]"
        );
        assert_eq!(
            model
                .judge_snapshot_read_back(0, &ObservedSnapshotReadBack::File(vec![10]))
                .map_err(|disagreement| disagreement.aspect),
            Err(ModelDisagreementAspect::SnapshotReadBack),
            "读出活头的 [10] 是读快照那一格对不上"
        );
        let intent = model.answer_destroy_snapshot(0).expect("删快照的意图");
        succeed(&mut model, &intent);
        let after_the_intent = model
            .current_version_in_the_session()
            .expect("会话开着")
            .clone();
        assert!(
            after_the_intent.snapshots[0].is_being_destroyed,
            "带上销毁中"
        );
        assert_eq!(
            after_the_intent.previous_snapshot_txg_of_the_live_head,
            ModelCheckpointTxg(0),
            "被删那一个的前驱 0 传给活头"
        );
        assert_eq!(after_the_intent.deadlist.len(), 4, "意图那次一条都不放");
        assert!(
            model.current_version_has_a_step_to_walk(),
            "销毁中的快照要走空"
        );
        let final_batch = model.answer_next_step_of_the_walk().expect("走空的一步");
        assert_eq!(
            final_batch.operation,
            ModelOperationKind::PublishSnapshotDestructionStep
        );
        succeed(&mut model, &final_batch);
        let after_the_final_batch = model.current_version_in_the_session().expect("会话开着");
        assert!(
            after_the_final_batch.snapshots.is_empty(),
            "最后一批删掉那一对条目"
        );
        assert!(after_the_final_batch.deadlist.is_empty(), "四条都放掉");
        assert_eq!(
            after_the_final_batch.pinned_mapping_entries, 0,
            "留存的映射条目跟着删"
        );
        assert!(!model.current_version_has_a_step_to_walk(), "走空走完");
    }

    /// 整理意图的三态（判决 G1、G4；D8（核心索引结构） 已定项 5）在模型里：没有活意图时只认签发（不搬、之后 logged_ops 里有一条：签发 txg、R、`c0`
    /// 是上一版的 txg、logged_ops 根的分配代 = 签发那次），签发之后再签发、搬空 R 而不删意图、删意图而 R 不空、不搬也不删都对不上；
    /// 照抄意图的一批、随最后一批删意图、单独删意图（R 已空：达成）、丢意图（R 不空）四种各认一次，删了之后这一版没有 logged_ops 根。
    #[test]
    fn the_model_takes_the_issue_first_and_lets_the_intent_leave_logged_operations_only_in_the_ways_the_clauses_allow(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        assert_eq!(
            aspect_of_the_compaction_publish(
                &model,
                &compaction_publish_seen(2, true, false, Vec::new())
            ),
            Err(ModelDisagreementAspect::CompactionIntentOfThePublish),
            "没有活意图：先签发才搬"
        );
        assert_eq!(
            aspect_of_the_compaction_publish(
                &model,
                &compaction_publish_seen(0, false, false, Vec::new())
            ),
            Err(ModelDisagreementAspect::CompactionIntentOfThePublish),
            "没有活意图：签发之后 logged_ops 里要有它"
        );
        let issue = compaction_publish_seen(0, true, false, Vec::new());
        assert_eq!(
            aspect_of_the_compaction_publish(&model, &issue),
            Ok(ModelOperationKind::PublishCompactionIntentIssue)
        );
        let issued = succeed_with_the_compaction_publish(&mut model, &issue);
        let intent = ModelCompactionIntent {
            issued_txg: ModelCheckpointTxg(10),
            region: issue.region_forced_for_the_issue,
            allocation_generation_at_issue: ModelCheckpointTxg(9),
        };
        assert_eq!(
            issued.logged_compaction_intent,
            Some(intent),
            "签发 txg 10、c0 9"
        );
        assert_eq!(
            issued
                .role_written_at
                .get(&ModelUnitRole::LoggedOperationsRoot),
            Some(&ModelCheckpointTxg(10)),
            "logged_ops 根是签发那次写的"
        );
        assert_eq!(
            aspect_of_the_compaction_publish(&model, &issue),
            Err(ModelDisagreementAspect::CompactionIntentOfThePublish),
            "活意图在：不再签发（全池至多一条）"
        );
        assert_eq!(
            aspect_of_the_compaction_publish(
                &model,
                &compaction_publish_seen(2, true, true, Vec::new())
            ),
            Err(ModelDisagreementAspect::CompactionIntentOfThePublish),
            "搬空了 R 而意图照抄"
        );
        assert_eq!(
            aspect_of_the_compaction_publish(
                &model,
                &compaction_publish_seen(2, false, false, Vec::new())
            ),
            Err(ModelDisagreementAspect::CompactionIntentOfThePublish),
            "删意图而 R 不空"
        );
        let carried = succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(2, true, false, Vec::new()),
        );
        assert_eq!(
            carried.logged_compaction_intent,
            Some(intent),
            "照抄意图的一批"
        );
        assert_eq!(
            carried
                .role_written_at
                .get(&ModelUnitRole::LoggedOperationsRoot),
            Some(&ModelCheckpointTxg(10)),
            "logged_ops 根照抄"
        );
        let last_batch = succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(1, false, true, Vec::new()),
        );
        assert_eq!(
            last_batch.logged_compaction_intent, None,
            "最后一批同发布删意图"
        );
        assert!(
            !last_batch
                .role_written_at
                .contains_key(&ModelUnitRole::LoggedOperationsRoot),
            "删空之后没有 logged_ops 根"
        );
        succeed_with_the_compaction_publish(&mut model, &issue);
        assert_eq!(
            aspect_of_the_compaction_publish(
                &model,
                &compaction_publish_seen(0, true, false, Vec::new())
            ),
            Err(ModelDisagreementAspect::CompactionIntentOfThePublish),
            "活意图在：不搬也不删"
        );
        let completion = compaction_publish_seen(0, false, true, Vec::new());
        assert_eq!(
            aspect_of_the_compaction_publish(&model, &completion),
            Ok(ModelOperationKind::PublishCompactionIntentCompletion)
        );
        let completed = succeed_with_the_compaction_publish(&mut model, &completion);
        assert_eq!(
            completed.logged_compaction_intent, None,
            "R 已空：单独删意图"
        );
        succeed_with_the_compaction_publish(&mut model, &issue);
        let drop = compaction_publish_seen(0, false, false, Vec::new());
        assert_eq!(
            aspect_of_the_compaction_publish(&model, &drop),
            Ok(ModelOperationKind::PublishCompactionIntentDrop)
        );
        let dropped = succeed_with_the_compaction_publish(&mut model, &drop);
        assert_eq!(dropped.logged_compaction_intent, None, "R 不空：丢意图");
        assert!(
            !dropped
                .role_written_at
                .contains_key(&ModelUnitRole::LoggedOperationsRoot),
            "丢了之后没有 logged_ops 根"
        );
        assert_eq!(
            dropped.key.checkpoint_txg,
            ModelCheckpointTxg(16),
            "七次整理发布各一次 txg"
        );
    }

    /// 一批至少搬一个（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H4「规划时补足」）：签发之后，
    /// 一个没搬、意图照抄、R 不空的那一次（计划里只有这次发布本来就重写的节点）对不上、模型不动；搬了一个的照抄意图的一批认；R 里只剩这次发布
    /// 本来就重写的节点时那一次不搬单元、同一次发布删意图、发布之后 R 按记录全空，认成意图达成：这一版没有 logged_ops 根，用户可见状态不变。
    #[test]
    fn compaction_publish_carrying_the_intent_relocates_at_least_one_unit_and_one_relocating_none_while_the_region_empties_deletes_the_intent(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let before = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(0, true, false, Vec::new()),
        );
        let after_the_issue = model.clone();
        assert_eq!(
            aspect_of_the_compaction_publish(
                &model,
                &compaction_publish_seen(0, true, false, Vec::new())
            ),
            Err(ModelDisagreementAspect::CompactionIntentOfThePublish),
            "一个没搬、意图照抄、R 不空：照抄意图的一批至少搬一个"
        );
        assert_eq!(model, after_the_issue, "判不合时模型不动");
        let carried = succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(1, true, false, Vec::new()),
        );
        assert_eq!(carried.key, key(11, 2), "搬了一个的照抄意图的一批");
        let only_nodes_the_publish_rewrites = compaction_publish_seen(0, false, true, Vec::new());
        assert_eq!(
            aspect_of_the_compaction_publish(&model, &only_nodes_the_publish_rewrites),
            Ok(ModelOperationKind::PublishCompactionIntentCompletion),
            "R 里只剩这次发布本来就重写的节点：不搬单元、同一次发布删意图，R 按记录全空是达成"
        );
        let completed =
            succeed_with_the_compaction_publish(&mut model, &only_nodes_the_publish_rewrites);
        assert_eq!(completed.key, key(12, 2));
        assert_eq!(completed.logged_compaction_intent, None, "意图删了");
        assert!(
            !completed
                .role_written_at
                .contains_key(&ModelUnitRole::LoggedOperationsRoot),
            "删空之后没有 logged_ops 根"
        );
        assert_eq!(
            completed.user_visible_state(),
            before,
            "整理不改用户可见状态"
        );
    }

    /// 没有可写会话时建快照答不了（会话状态对不上）；会话开着而现行那一版树表 0 条（mkfs 之后取号暖机、还没发第一个文件）同样答不了：
    /// 执行器只在有树的一版上调。
    #[test]
    fn the_model_does_not_answer_a_snapshot_creation_without_a_writable_session_or_on_a_version_without_trees(
    ) {
        let mut model = two_device_model();
        assert_eq!(
            model
                .answer_create_snapshot()
                .map_err(|disagreement| disagreement.aspect),
            Err(ModelDisagreementAspect::SessionState)
        );
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        assert_eq!(
            model
                .answer_create_snapshot()
                .map_err(|disagreement| disagreement.aspect),
            Err(ModelDisagreementAspect::SessionState),
            "树表 0 条的一版"
        );
    }

    /// 文件集合为空时建快照照建（「放开」，判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F5）：
    /// 第一个文件（txg 3）、unlink 意图与最后一批之后那一版有树而文件集合为空，建快照不要求拒、记的内容是「没有文件」；之后建一个 inode，
    /// 读快照读出「没有文件」，读出一个文件对不上。
    #[test]
    fn snapshot_created_on_a_version_with_trees_and_no_file_reads_no_file_after_an_inode_is_created(
    ) {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        succeed(&mut model, &first);
        let intent = model
            .answer_unlink_intent()
            .expect("会话开着、现行版本带活文件");
        succeed(&mut model, &intent);
        let final_batch = model
            .answer_deletion_step()
            .expect("会话开着、现行版本删除中");
        succeed(&mut model, &final_batch);
        let creation = model
            .answer_create_snapshot()
            .expect("会话开着、现行那一版有树");
        assert!(
            creation.required_refusals.is_empty(),
            "有树而文件集合为空时照建：{:?}",
            creation.required_refusals
        );
        succeed(&mut model, &creation);
        assert_eq!(
            current_snapshots(&model)
                .iter()
                .map(|snapshot| snapshot.content_at_creation.clone())
                .collect::<Vec<_>>(),
            vec![None],
            "快照记的是建它那一刻的「没有文件」"
        );
        let created = model
            .answer_create_inode()
            .expect("会话开着、第一个文件已删完");
        succeed(&mut model, &created);
        assert_eq!(
            model
                .judge_snapshot_read_back(0, &ObservedSnapshotReadBack::NoFile)
                .map(|counts| counts.snapshot_read_backs_compared)
                .map_err(|disagreement| disagreement.aspect),
            Ok(1),
            "读快照读出「没有文件」"
        );
        assert_eq!(
            model
                .judge_snapshot_read_back(0, &ObservedSnapshotReadBack::File(vec![3]))
                .map_err(|disagreement| disagreement.aspect),
            Err(ModelDisagreementAspect::SnapshotReadBack),
            "读出一个文件对不上"
        );
    }
}
