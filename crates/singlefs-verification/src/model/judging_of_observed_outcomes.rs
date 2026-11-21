//! 拿实现交回的结局判这一步（效果、容量墙、根、截断字段、别的 inode、文件内容、实例表、快照），判完推进模型；连同把回答与读回写成报告文字的几个函数。

use super::*;

impl IdealModel {
    /// 容量墙的区间（收口表第 39 行那种：条款把答案留给实现取上界，模型答允许拒绝的区间）。分配记录树按位置寻址之后
    /// （D8（核心索引结构） 已定项 14）没有「一个节点装不下」那道墙，剩单元区这一道：
    /// - 单元区：真条数那一头不判；上界那一头是占槽上界 × 一个聚簇段的槽数超过单元区（每个落点最坏独占一段：已分配 + defer ≤ 2 × 上界，
    ///   保留池 ckpt_cost（这次发布要接在后面的那一版现算，D16（发布语义） 已定项 1「准入」「`df`」两行）与切换预留
    ///   （D28（挂载期承诺量） 已定项 3）在 64 倍里）。预想：D28 已定项 1 的准入式子
    ///   第一版没实现，各项没有现值。
    fn capacity_wall_is_permitted(
        &self,
        reason: ModelRefusalReason,
        answer: &ModelAnswer,
        publishes_completed: usize,
    ) -> bool {
        // 一串判完的（可写挂载、回退、抬 F）看计划里的每一次；逐次判的（发布）只看拒的那一次。
        let judged_publishes: &[PlannedPublishUpperBounds] =
            if answer.walls_are_judged_before_the_first_write {
                &answer.planned_publish_upper_bounds
            } else {
                answer
                    .planned_publish_upper_bounds
                    .get(publishes_completed..=publishes_completed)
                    .unwrap_or(&[])
            };
        match reason {
            ModelRefusalReason::UnitAreaWall => judged_publishes.iter().any(|planned| {
                planned.occupied_slots_per_device * CLUSTER_SEGMENT_SLOTS
                    > self.unit_area_slots_per_device()
            }),
            ModelRefusalReason::ContentExceedsDataUnitPayload
            | ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn
            | ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile
            | ModelRefusalReason::RollbackTargetNotInRing
            | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
            | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
            | ModelRefusalReason::RollbackTargetWithoutFile
            | ModelRefusalReason::FloorAboveCeiling
            | ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread
            | ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile
            | ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses
            | ModelRefusalReason::MoreSnapshotsThanTheFirstVersionAllows
            | ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow
            | ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted
            | ModelRefusalReason::OldDataUnitToRewriteNotReadBack => false,
        }
    }

    /// 拿实现的结局与模型的答案比；对得上就按模型自己算的结果往前走（不按实现交回的），对不上交回哪一格。
    ///
    /// # Errors
    /// 对不上的第一格。
    pub fn judge_and_advance(
        &mut self,
        answer: &ModelAnswer,
        observed: &ObservedOutcome,
    ) -> Result<ModelJudgementCounts, ModelDisagreement> {
        let mut counts = ModelJudgementCounts::default();
        match observed {
            ObservedOutcome::Refused {
                member,
                reason,
                publishes_completed,
                wrote_anything,
                reported_ceiling,
                reported_root_ring_slot_still_bad_after_one_reread,
            } => {
                self.judge_reported_ceiling(answer, *reported_ceiling, &mut counts)?;
                let accepted = match reason {
                    ObservedRefusalReason::Explained(candidate) => {
                        Some(*candidate).filter(|candidate| {
                            answer.required_refusals.contains(candidate)
                                || (candidate.is_capacity_wall_with_an_interval()
                                    && self.capacity_wall_is_permitted(
                                        *candidate,
                                        answer,
                                        *publishes_completed,
                                    ))
                        })
                    }
                    ObservedRefusalReason::Unexplained => None,
                };
                let Some(accepted_reason) = accepted else {
                    let aspect = if answer.required_refusals.is_empty() {
                        ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
                    } else {
                        ModelDisagreementAspect::RefusalReason
                    };
                    // 实现报的读坏的根环槽只在「拒之前一个写都没发、一次发布都没做」时带出去（mount.rs 那个成员的契约：
                    // 在回收、影子账与任何写之前返回）；拒之前写了盘的，故障注入不许拿这个槽去认。
                    let refused_before_any_write = !*wrote_anything && *publishes_completed == 0;
                    let mut disagreement = ModelDisagreement::new(
                        aspect,
                        describe_answer(answer),
                        format!("拒了：{member}"),
                    );
                    let root_ring_slot_reported_before_any_write =
                        reported_root_ring_slot_still_bad_after_one_reread
                            .filter(|_| refused_before_any_write);
                    disagreement.implementation_reported_unreadable_place =
                        root_ring_slot_reported_before_any_write
                            .map(ModelReportedUnreadablePlace::RootRingSlotStillBadAfterOneReread);
                    return Err(disagreement);
                };
                let partial_publishes_allowed = !answer.walls_are_judged_before_the_first_write
                    && accepted_reason.is_capacity_wall_with_an_interval();
                if *publishes_completed > 0 && !partial_publishes_allowed {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::PublishCount,
                        format!("{}：拒之前一次发布都不做", accepted_reason.name()),
                        format!("拒之前做了 {publishes_completed} 次发布（{member}）"),
                    ));
                }
                if *wrote_anything && *publishes_completed == 0 {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::WroteBeforeRefusing,
                        format!("{}：拒绝之前盘上一个字节都不动", accepted_reason.name()),
                        format!("拒了（{member}），录制流里多了写或屏障"),
                    ));
                }
                let Some(completed) = answer.expected_roots.get(..*publishes_completed) else {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::PublishCount,
                        format!("至多 {} 次发布", answer.expected_roots.len()),
                        format!("拒之前做了 {publishes_completed} 次发布（{member}）"),
                    ));
                };
                if answer.required_refusals.contains(&accepted_reason) {
                    counts.required_refusals_matched += 1;
                } else {
                    counts.permitted_refusals_taken += 1;
                }
                if accepted_reason == ModelRefusalReason::UnitAreaWall {
                    counts.unit_area_wall_refusals_in_the_interval += 1;
                }
                if accepted_reason
                    == ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted
                {
                    counts.data_writes_refused_after_the_first_file_was_deleted_matched += 1;
                }
                for root in completed {
                    self.write_root(root.clone());
                }
                if let (Some(session), Some(last)) = (self.session.as_mut(), completed.last()) {
                    session.current = last.clone();
                }
                Ok(counts)
            }
            ObservedOutcome::Succeeded(effect) => {
                if !answer.required_refusals.is_empty() {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::SucceededWhenModelRequiresRefusal,
                        describe_answer(answer),
                        "做成了".to_string(),
                    ));
                }
                self.judge_effect(answer, effect, &mut counts)?;
                counts.successes_matched += 1;
                if answer.operation == ModelOperationKind::PublishDeletionStep {
                    if answer
                        .expected_roots
                        .iter()
                        .any(|root| root.inode_being_deleted().is_some())
                    {
                        counts.deletion_batches_keeping_data_units_matched += 1;
                    } else if let (Some(after), Some(session)) =
                        (answer.expected_roots.first(), self.session.as_ref())
                    {
                        Self::count_the_shape_of_a_final_deletion_batch(
                            &session.current,
                            after,
                            &mut counts,
                        );
                    }
                }
                if answer.operation == ModelOperationKind::PublishSnapshotDestructionStep
                    && answer.expected_roots.iter().any(|root| {
                        root.snapshots
                            .iter()
                            .any(|snapshot| snapshot.is_being_destroyed)
                    })
                {
                    counts.snapshot_destruction_batches_keeping_the_snapshot_matched += 1;
                }
                match answer.operation {
                    ModelOperationKind::PublishCompactionBatch => {
                        counts.compaction_batches_matched += 1;
                        if answer
                            .expected_roots
                            .iter()
                            .any(|root| root.logged_compaction_intent.is_none())
                        {
                            counts.compaction_intents_deleted_with_the_last_batch_matched += 1;
                        }
                    }
                    ModelOperationKind::PublishRelocationBatchAfterAUserChange => {
                        counts.relocation_batches_after_a_user_change_matched += 1;
                    }
                    ModelOperationKind::PublishCompactionIntentIssue => {
                        counts.compaction_intents_issued_matched += 1;
                    }
                    ModelOperationKind::PublishCompactionIntentCompletion => {
                        counts.compaction_intents_deleted_without_a_batch_matched += 1;
                    }
                    ModelOperationKind::PublishCompactionIntentDrop => {
                        counts.compaction_intents_dropped_matched += 1;
                    }
                    ModelOperationKind::PublishFirstFile
                    | ModelOperationKind::PublishOverwrite
                    | ModelOperationKind::PublishSequentialWrite
                    | ModelOperationKind::PublishWithoutUnits
                    | ModelOperationKind::MountWritable
                    | ModelOperationKind::RollbackWhileMounted
                    | ModelOperationKind::RaiseRollbackFloor
                    | ModelOperationKind::Unmount
                    | ModelOperationKind::ColdStartRecover
                    | ModelOperationKind::PublishUnlinkIntent
                    | ModelOperationKind::PublishDeletionStep
                    | ModelOperationKind::PublishNewInode
                    | ModelOperationKind::PublishSnapshotCreation
                    | ModelOperationKind::PublishSnapshotDestructionIntent
                    | ModelOperationKind::PublishSnapshotDestructionStep => {}
                    // 按 inode 号写第一个文件之外的文件：数这次长没长高、写的号落没落在上段第 0 片叶之外（这一组取样点要罩的两形）。
                    ModelOperationKind::PublishWholeWriteOfAnInodeOtherThanTheFirst => {
                        counts.writes_of_an_inode_other_than_the_first_matched += 1;
                        if let (Some(written_version), Some(session)) =
                            (answer.expected_roots.first(), self.session.as_ref())
                        {
                            let level_before = extent_upper_segment_root_level(
                                &session.current.inodes_with_an_extent_upper_leaf_entry(),
                            );
                            let level_after = extent_upper_segment_root_level(
                                &written_version.inodes_with_an_extent_upper_leaf_entry(),
                            );
                            if level_after > level_before {
                                counts.writes_growing_the_extent_upper_segment_matched += 1;
                            }
                            let written_inode = written_version
                                .inodes_other_than_the_first
                                .iter()
                                .find(|(_, inode)| {
                                    inode
                                        .data_unit_written_at
                                        .contains(&written_version.key.checkpoint_txg)
                                })
                                .map(|(number, _)| *number);
                            if written_inode
                                .is_some_and(|number| number >= EXTENT_TREE_UPPER_LEAF_INODES)
                            {
                                counts.writes_past_the_first_extent_upper_leaf_matched += 1;
                            }
                        }
                    }
                    // 按偏移写：第一个文件的单元这次变了的是写第一个文件，没变的是写别的文件。
                    ModelOperationKind::PublishPartialWrite => {
                        if let (Some(written_version), Some(session)) =
                            (answer.expected_roots.first(), self.session.as_ref())
                        {
                            if written_version.data_unit_written_at
                                == session.current.data_unit_written_at
                            {
                                counts.partial_writes_of_an_inode_other_than_the_first_matched += 1;
                            } else {
                                counts.partial_writes_of_the_first_file_matched += 1;
                            }
                        }
                    }
                    // 截断（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2、K5）：缩短发的意图之后这一版在截断途中，
                    // 不缩短的那一次之后是带活文件的一版。
                    ModelOperationKind::PublishTruncation => {
                        if answer
                            .expected_roots
                            .iter()
                            .any(|root| root.logged_truncation_issued_txg().is_some())
                        {
                            counts.truncation_intents_matched += 1;
                        } else {
                            counts.size_changes_without_shrinking_matched += 1;
                        }
                    }
                    // 截断的一批之后仍在截断途中的是不是最后一批的那一批，之后是带活文件的一版的是最后一批。
                    ModelOperationKind::PublishTruncationStep => {
                        if answer
                            .expected_roots
                            .iter()
                            .any(|root| root.logged_truncation_issued_txg().is_some())
                        {
                            counts.truncation_batches_before_the_final_batch_matched += 1;
                        } else {
                            counts.final_truncation_batches_matched += 1;
                        }
                    }
                }
                if answer.expected_roots.iter().any(|root| {
                    root.user_visible_root_relocations
                        .values()
                        .any(|relocated_at| *relocated_at == root.key.checkpoint_txg)
                }) {
                    counts.compaction_publishes_relocating_a_user_visible_root_matched += 1;
                }
                self.count_the_changes_beside_files_other_than_the_first(answer, &mut counts);
                self.advance_by_success(answer);
                Ok(counts)
            }
        }
    }

    /// 一次比过、对得上的改动在第一个文件之外有文件带数据单元的一版上是哪几格（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md`
    /// 第二节 S5 的验收：别的 inode 截断、变长、快照下截断、整理时截断），逐格计数：改 size 那一次与截断的各批改的是带数据单元的别的 inode（它的内容
    /// 或单元这次变了），其中这一版有快照的、logged_ops 里有活整理意图的另数；别的文件带数据单元时建快照、整理发布各数一次。
    fn count_the_changes_beside_files_other_than_the_first(
        &self,
        answer: &ModelAnswer,
        counts: &mut ModelJudgementCounts,
    ) {
        let (Some(session), Some(after)) = (self.session.as_ref(), answer.expected_roots.first())
        else {
            return;
        };
        let before = &session.current;
        let an_inode_holding_data_units_changed =
            before
                .inodes_other_than_the_first
                .iter()
                .any(|(number, inode_before)| {
                    after
                        .inodes_other_than_the_first
                        .get(number)
                        .is_some_and(|inode_after| {
                            (inode_before.data_units_present() > 0
                                || inode_after.data_units_present() > 0)
                                && (inode_after.content != inode_before.content
                                    || inode_after.data_unit_written_at
                                        != inode_before.data_unit_written_at)
                        })
                });
        match answer.operation {
            ModelOperationKind::PublishTruncation | ModelOperationKind::PublishTruncationStep
                if an_inode_holding_data_units_changed =>
            {
                if answer.operation == ModelOperationKind::PublishTruncation {
                    counts
                        .size_changes_of_inodes_other_than_the_first_holding_data_units_matched +=
                        1;
                } else {
                    counts
                        .truncation_steps_of_inodes_other_than_the_first_holding_data_units_matched +=
                        1;
                }
                if !before.snapshots.is_empty() {
                    counts.size_changes_of_inodes_other_than_the_first_under_a_snapshot_matched +=
                        1;
                }
                if before.logged_compaction_intent.is_some() {
                    counts
                        .size_changes_of_inodes_other_than_the_first_beside_a_live_compaction_intent_matched +=
                        1;
                }
            }
            ModelOperationKind::PublishSnapshotCreation
                if before.has_a_file_other_than_the_first_holding_data_units() =>
            {
                counts.snapshots_created_beside_files_other_than_the_first_holding_data_units_matched +=
                    1;
            }
            ModelOperationKind::PublishCompactionBatch
            | ModelOperationKind::PublishCompactionIntentIssue
                if before.has_a_file_other_than_the_first_holding_data_units() =>
            {
                counts
                    .compaction_publishes_beside_files_other_than_the_first_holding_data_units_matched +=
                    1;
            }
            ModelOperationKind::PublishFirstFile
            | ModelOperationKind::PublishOverwrite
            | ModelOperationKind::PublishSequentialWrite
            | ModelOperationKind::PublishWithoutUnits
            | ModelOperationKind::MountWritable
            | ModelOperationKind::RollbackWhileMounted
            | ModelOperationKind::RaiseRollbackFloor
            | ModelOperationKind::Unmount
            | ModelOperationKind::ColdStartRecover
            | ModelOperationKind::PublishUnlinkIntent
            | ModelOperationKind::PublishDeletionStep
            | ModelOperationKind::PublishNewInode
            | ModelOperationKind::PublishSnapshotCreation
            | ModelOperationKind::PublishSnapshotDestructionIntent
            | ModelOperationKind::PublishSnapshotDestructionStep
            | ModelOperationKind::PublishCompactionBatch
            | ModelOperationKind::PublishRelocationBatchAfterAUserChange
            | ModelOperationKind::PublishCompactionIntentIssue
            | ModelOperationKind::PublishCompactionIntentCompletion
            | ModelOperationKind::PublishCompactionIntentDrop
            | ModelOperationKind::PublishTruncation
            | ModelOperationKind::PublishTruncationStep
            | ModelOperationKind::PublishWholeWriteOfAnInodeOtherThanTheFirst
            | ModelOperationKind::PublishPartialWrite => {}
        }
    }

    /// 一次比过、对得上的删除最后一批（`before` 是它接在后面的那一版，`after` 是它写出的那一版）落在 那四形里的哪几形，逐形计数：
    /// 删的是第一个文件之外的号、删第一个文件而别的 inode 还在、删到文件集合为空、extent 上段的根降了层。
    fn count_the_shape_of_a_final_deletion_batch(
        before: &ModelRoot,
        after: &ModelRoot,
        counts: &mut ModelJudgementCounts,
    ) {
        let Some((deleted, _)) = before.inode_being_deleted() else {
            return;
        };
        match deleted {
            ModelDeletedInode::OtherThanTheFirst(_) => {
                counts.deletions_of_an_inode_other_than_the_first_matched += 1;
                if matches!(
                    before.contents,
                    ModelVersionContents::InodesWithoutTheFirstFile { .. }
                ) {
                    counts
                        .deletions_of_an_inode_other_than_the_first_after_the_first_file_was_deleted_matched +=
                        1;
                }
            }
            ModelDeletedInode::FirstFile => {
                if !after.inodes_other_than_the_first.is_empty() {
                    counts.deletions_of_the_first_file_beside_other_inodes_matched += 1;
                }
            }
        }
        if matches!(after.contents, ModelVersionContents::TreesAndNoFile { .. }) {
            counts.final_deletion_batches_emptying_the_version_matched += 1;
        }
        if extent_upper_segment_root_level(&after.inodes_with_an_extent_upper_leaf_entry())
            < extent_upper_segment_root_level(&before.inodes_with_an_extent_upper_leaf_entry())
        {
            counts.deletions_lowering_the_extent_upper_segment_matched += 1;
        }
    }

    fn judge_reported_ceiling(
        &self,
        answer: &ModelAnswer,
        reported_ceiling: Option<ModelCheckpointTxg>,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        let Some(reported) = reported_ceiling else {
            return Ok(());
        };
        counts.ceilings_compared += 1;
        if answer.rollback_floor_ceiling == Some(reported) {
            Ok(())
        } else {
            Err(ModelDisagreement {
                aspect: ModelDisagreementAspect::RollbackFloorCeiling,
                model_answer: format!(
                    "上限 {:?}（D16（发布语义） 已定项 1）",
                    answer.rollback_floor_ceiling.map(|ceiling| ceiling.0)
                ),
                implementation_answer: format!("实现报上限 {}", reported.0),
                implementation_reported_ceiling: Some(reported),
                implementation_reported_unreadable_place: None,
                implementation_rows_written: None,
            })
        }
    }

    /// 这一种操作做成时胶水交的是不是这一种结局。兼容表照各个 `answer_*` 答的种类与 `history.rs`、`model_comparison.rs` 的胶水现在交的那一种：
    /// 冷启动（`answer_cold_start_recover`）只收冷启动读回；可写挂载（`answer_mount_writable` 一族）只收挂载；
    /// 别的种类——各种发布、挂着时回退（`answer_rollback_while_mounted`）、抬 F（`answer_raise_rollback_floor`）、
    /// 正常卸载（`answer_unmount`，胶水 `model_comparison::observed_unmount` 交带新 F 的空发布）——只收发布。
    /// 新加一种操作不在这里两头都写上就编译不过（不写通配臂）。
    fn operation_hands_in_this_kind_of_effect(
        operation: ModelOperationKind,
        effect: &ObservedEffect,
    ) -> bool {
        match (operation, effect) {
            (ModelOperationKind::ColdStartRecover, ObservedEffect::ColdStart { .. })
            | (ModelOperationKind::MountWritable, ObservedEffect::Mount { .. }) => true,
            (
                ModelOperationKind::ColdStartRecover,
                ObservedEffect::Publishes { .. } | ObservedEffect::Mount { .. },
            )
            | (
                ModelOperationKind::MountWritable,
                ObservedEffect::Publishes { .. } | ObservedEffect::ColdStart { .. },
            ) => false,
            (
                ModelOperationKind::PublishFirstFile
                | ModelOperationKind::PublishOverwrite
                | ModelOperationKind::PublishSequentialWrite
                | ModelOperationKind::PublishWithoutUnits
                | ModelOperationKind::RollbackWhileMounted
                | ModelOperationKind::RaiseRollbackFloor
                | ModelOperationKind::Unmount
                | ModelOperationKind::PublishUnlinkIntent
                | ModelOperationKind::PublishDeletionStep
                | ModelOperationKind::PublishNewInode
                | ModelOperationKind::PublishSnapshotCreation
                | ModelOperationKind::PublishSnapshotDestructionIntent
                | ModelOperationKind::PublishSnapshotDestructionStep
                | ModelOperationKind::PublishCompactionBatch
                | ModelOperationKind::PublishRelocationBatchAfterAUserChange
                | ModelOperationKind::PublishCompactionIntentIssue
                | ModelOperationKind::PublishCompactionIntentCompletion
                | ModelOperationKind::PublishCompactionIntentDrop
                | ModelOperationKind::PublishTruncation
                | ModelOperationKind::PublishTruncationStep
                | ModelOperationKind::PublishWholeWriteOfAnInodeOtherThanTheFirst
                | ModelOperationKind::PublishPartialWrite,
                ObservedEffect::Publishes { .. },
            ) => true,
            (
                ModelOperationKind::PublishFirstFile
                | ModelOperationKind::PublishOverwrite
                | ModelOperationKind::PublishSequentialWrite
                | ModelOperationKind::PublishWithoutUnits
                | ModelOperationKind::RollbackWhileMounted
                | ModelOperationKind::RaiseRollbackFloor
                | ModelOperationKind::Unmount
                | ModelOperationKind::PublishUnlinkIntent
                | ModelOperationKind::PublishDeletionStep
                | ModelOperationKind::PublishNewInode
                | ModelOperationKind::PublishSnapshotCreation
                | ModelOperationKind::PublishSnapshotDestructionIntent
                | ModelOperationKind::PublishSnapshotDestructionStep
                | ModelOperationKind::PublishCompactionBatch
                | ModelOperationKind::PublishRelocationBatchAfterAUserChange
                | ModelOperationKind::PublishCompactionIntentIssue
                | ModelOperationKind::PublishCompactionIntentCompletion
                | ModelOperationKind::PublishCompactionIntentDrop
                | ModelOperationKind::PublishTruncation
                | ModelOperationKind::PublishTruncationStep
                | ModelOperationKind::PublishWholeWriteOfAnInodeOtherThanTheFirst
                | ModelOperationKind::PublishPartialWrite,
                ObservedEffect::Mount { .. } | ObservedEffect::ColdStart { .. },
            ) => false,
        }
    }

    fn judge_effect(
        &self,
        answer: &ModelAnswer,
        effect: &ObservedEffect,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        if !Self::operation_hands_in_this_kind_of_effect(answer.operation, effect) {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::EffectKindOfTheOperation,
                format!("{:?} 做成时胶水交的不是这一种结局", answer.operation),
                format!("胶水交来的是{}", describe_observed_effect_kind(effect)),
            ));
        }
        match effect {
            ObservedEffect::Publishes {
                roots,
                reported_ceiling,
            } => {
                self.judge_reported_ceiling(answer, *reported_ceiling, counts)?;
                if answer.operation == ModelOperationKind::RaiseRollbackFloor
                    && reported_ceiling.is_none()
                {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::RollbackFloorCeiling,
                        "抬 F 做成要报上限".to_string(),
                        "胶水没交上限".to_string(),
                    ));
                }
                if let Some(target) = answer.rollback_target {
                    let floor = self.effective_rollback_floor();
                    if floor.0 > 0 && target.checkpoint_txg == floor {
                        counts.rollbacks_accepted_at_the_effective_floor += 1;
                    }
                }
                self.judge_roots(&answer.expected_roots, roots, counts)
            }
            ObservedEffect::Mount {
                instance,
                rows_written,
                roots,
            } => {
                let Some((expected_instance, expected_rows)) = &answer.expected_mount else {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::NotModeled,
                        describe_answer(answer),
                        "实现交回的是挂载".to_string(),
                    ));
                };
                if instance != expected_instance {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::InstanceGeneration,
                        format!("取号 {}", expected_instance.0),
                        format!("取号 {}", instance.0),
                    ));
                }
                if rows_written != expected_rows {
                    let mut disagreement = ModelDisagreement::new(
                        ModelDisagreementAspect::InstanceRows,
                        format!("写行 {expected_rows:?}"),
                        format!("写行 {rows_written:?}"),
                    );
                    disagreement.implementation_rows_written = Some(rows_written.clone());
                    return Err(disagreement);
                }
                self.judge_roots(&answer.expected_roots, roots, counts)
            }
            ObservedEffect::ColdStart { read_back } => {
                let Some(expected) = &answer.expected_read_back else {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::NotModeled,
                        describe_answer(answer),
                        "实现交回的是冷启动读回".to_string(),
                    ));
                };
                let matches = match (expected, read_back) {
                    (
                        ModelReadBack::NoFile {
                            root: expected_root,
                        },
                        ObservedReadBack::NoFile { root },
                    ) => expected_root == root,
                    (
                        ModelReadBack::FileRead {
                            root: expected_root,
                            content: expected_content,
                        },
                        ObservedReadBack::FileRead { root, content },
                    ) => {
                        counts.cold_start_contents_compared += 1;
                        expected_root == root && expected_content.as_ref() == content.as_slice()
                    }
                    (
                        ModelReadBack::NoFile { .. } | ModelReadBack::FileRead { .. },
                        ObservedReadBack::NoFile { .. }
                        | ObservedReadBack::FileRead { .. }
                        | ObservedReadBack::Failed { .. },
                    ) => false,
                };
                if matches {
                    Ok(())
                } else {
                    Err(ModelDisagreement::new(
                        ModelDisagreementAspect::ColdStartReadBack,
                        describe_model_read_back(expected),
                        describe_observed_read_back(read_back),
                    ))
                }
            }
        }
    }

    pub(super) fn judge_roots(
        &self,
        expected_roots: &[ModelRoot],
        observed_roots: &[ObservedRoot],
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        if expected_roots.len() != observed_roots.len() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::PublishCount,
                format!(
                    "写出 {} 条根：{:?}",
                    expected_roots.len(),
                    expected_roots
                        .iter()
                        .map(|root| root.key)
                        .collect::<Vec<_>>()
                ),
                format!(
                    "写出 {} 条根：{:?}",
                    observed_roots.len(),
                    observed_roots
                        .iter()
                        .map(|root| root.key)
                        .collect::<Vec<_>>()
                ),
            ));
        }
        for (expected, observed) in expected_roots.iter().zip(observed_roots) {
            counts.roots_compared += 1;
            if expected.key != observed.key {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::RootIdentity,
                    format!("{:?}", expected.key),
                    format!("{:?}", observed.key),
                ));
            }
            if expected.journal_counter != observed.journal_counter {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::JournalCounter,
                    format!("{:?} 的 jsn {}", expected.key, expected.journal_counter.0),
                    format!("jsn {}", observed.journal_counter.0),
                ));
            }
            if expected.rollback_floor != observed.rollback_floor {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::RollbackFloor,
                    format!("{:?} 带 F {}", expected.key, expected.rollback_floor.0),
                    format!("带 F {}", observed.rollback_floor.0),
                ));
            }
            if expected.contents.live_file().is_some() != observed.file.has_file() {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::FilePresence,
                    format!("{:?} 里有什么：{:?}", expected.key, expected.contents),
                    format!("有文件：{}", observed.file.has_file()),
                ));
            }
            Self::judge_file_content(expected, &observed.file, counts)?;
            Self::judge_instance_table(expected, &observed.instance_table, counts)?;
            self.judge_allocation_generations(expected, observed, counts)?;
            Self::judge_snapshot_state(expected, &observed.snapshot_state, counts)?;
            Self::judge_logged_compaction_intent(expected, observed, counts)?;
            Self::judge_first_file_record_truncation_field(expected, observed, counts)?;
            Self::judge_inodes_other_than_the_first(expected, observed, counts)?;
            Self::judge_truncation_fields_of_the_inodes_other_than_the_first(
                expected, observed, counts,
            )?;
        }
        Ok(())
    }

    /// 这一版第一个文件之外每条记录里的截断字段与模型记的相同（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5「截断字段
    /// 按 inode 比」，m4-trunc-r2 第三节 R1、R2 推到任意 inode）：号的集合一样（活着的与删除中的记录都在树里），每条的 T、N_w 两段一样。
    fn judge_truncation_fields_of_the_inodes_other_than_the_first(
        expected: &ModelRoot,
        observed: &ObservedRoot,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        let expected_fields: BTreeMap<u64, ObservedTruncationField> = expected
            .inodes_other_than_the_first
            .iter()
            .map(|(number, inode)| (*number, inode.truncation_field.recorded_values()))
            .collect();
        if expected_fields != observed.truncation_fields_of_the_inodes_other_than_the_first {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::InodesOtherThanTheFirst,
                format!(
                    "{:?} 第一个文件之外每条记录的截断字段 {expected_fields:?}",
                    expected.key
                ),
                format!(
                    "{:?}",
                    observed.truncation_fields_of_the_inodes_other_than_the_first
                ),
            ));
        }
        for inode in expected.inodes_other_than_the_first.values() {
            counts.truncation_fields_of_the_inodes_other_than_the_first_compared += 1;
            match inode.truncation_field {
                ModelTruncationField::WrittenByATruncationIntent { .. } => {
                    counts.written_truncation_fields_of_the_inodes_other_than_the_first_compared +=
                        1;
                }
                ModelTruncationField::NotWritten => {}
            }
        }
        Ok(())
    }

    /// 这一版第一个文件之外的 inode 与模型记的相同（里程碑「删除与多次 COW」一·1）：号的集合一样，每个读回的内容逐字节一样
    /// （没写过数据的读回空）。写错了 inode（写成别的号、写成第一个文件）这里与第一个文件的内容那一格必有一格对不上。
    fn judge_inodes_other_than_the_first(
        expected: &ModelRoot,
        observed: &ObservedRoot,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        // 删除中的那一个（nlink = 0）读不出，实现那一侧也不交（`model_comparison::observed_inodes_other_than_the_first_of_file_version`），
        // 它还剩的单元按分配代另比。
        let expected_numbers: Vec<u64> = expected
            .inodes_other_than_the_first
            .iter()
            .filter(|(_, inode)| inode.is_live())
            .map(|(number, _)| *number)
            .collect();
        let observed_numbers: Vec<u64> = observed
            .inodes_other_than_the_first
            .keys()
            .copied()
            .collect();
        if expected_numbers != observed_numbers {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::InodesOtherThanTheFirst,
                format!(
                    "{:?} 第一个文件之外的 inode {expected_numbers:?}（水位 {}）",
                    expected.key, expected.inode_number_watermark
                ),
                format!("{observed_numbers:?}"),
            ));
        }
        for (number, inode) in expected
            .inodes_other_than_the_first
            .iter()
            .filter(|(_, inode)| inode.is_live())
        {
            let observed_content = observed
                .inodes_other_than_the_first
                .get(number)
                .expect("上面比过两边的号一样");
            match observed_content {
                ObservedFile::Decoded(content) if content.as_slice() == inode.content.as_ref() => {}
                ObservedFile::Decoded(_)
                | ObservedFile::NoFile
                | ObservedFile::Undecodable { .. } => {
                    let described = match observed_content {
                        ObservedFile::Decoded(content) => {
                            format!("{} 字节（末字节 {:?}）", content.len(), content.last())
                        }
                        ObservedFile::NoFile => "读不出这个文件".to_string(),
                        ObservedFile::Undecodable { what } => format!("解不出内容：{what}"),
                    };
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::InodesOtherThanTheFirst,
                        format!(
                            "{:?} 的 inode {number}（各单元的分配代 {:?}）读回 {} 字节（末字节 {:?}）",
                            expected.key,
                            inode.data_unit_written_at,
                            inode.content.len(),
                            inode.content.last()
                        ),
                        described,
                    ));
                }
            }
            counts.inodes_other_than_the_first_compared += 1;
            if !inode.data_unit_written_at.is_empty() {
                counts.contents_of_files_other_than_the_first_compared += 1;
            }
        }
        Ok(())
    }

    /// 这一版第一个文件那条 inode 记录里的截断字段与模型记的相同（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R1、R2、R5，
    /// 第六节）：有没有那条记录、T、N_w 两段的值。
    fn judge_first_file_record_truncation_field(
        expected: &ModelRoot,
        observed: &ObservedRoot,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        let expected_values = expected
            .contents
            .first_file_record_truncation_field()
            .map(ModelTruncationField::recorded_values);
        if expected_values != observed.first_file_record_truncation_field {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::FirstFileRecordTruncationField,
                format!(
                    "{:?} 第一个文件那条记录的截断字段 {:?}（{:?}）",
                    expected.key,
                    expected_values,
                    expected.contents.first_file_record_truncation_field()
                ),
                format!("{:?}", observed.first_file_record_truncation_field),
            ));
        }
        match expected.contents.first_file_record_truncation_field() {
            Some(ModelTruncationField::WrittenByATruncationIntent { .. }) => {
                counts.truncation_fields_compared += 1;
                counts.written_truncation_fields_compared += 1;
            }
            Some(ModelTruncationField::NotWritten) => counts.truncation_fields_compared += 1,
            None => {}
        }
        Ok(())
    }

    /// 这一版 logged_ops 里的整理意图与模型记的相同：有没有、签发 txg、R、`c0`（判决 G1 的字段表；`segs0` 不比）。
    fn judge_logged_compaction_intent(
        expected: &ModelRoot,
        observed: &ObservedRoot,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        if expected.logged_compaction_intent != observed.logged_compaction_intent {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::LoggedCompactionIntent,
                format!(
                    "{:?} 的 logged_ops 整理意图 {:?}",
                    expected.key, expected.logged_compaction_intent
                ),
                format!("{:?}", observed.logged_compaction_intent),
            ));
        }
        if expected.logged_compaction_intent.is_some() {
            counts.logged_compaction_intents_compared += 1;
        }
        Ok(())
    }

    /// 这一版的快照那几样与模型记的逐项相同（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S7）：活头前驱（树表 0 条的一版
    /// 没有那条条目）、快照列表（个数、每个的诞生 txg、前驱、销毁中记号、两条根指针抄的是谁）、deadlist 条目数、被钉映射条目数，各是一格。
    fn judge_snapshot_state(
        expected: &ModelRoot,
        observed: &ObservedSnapshotState,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        let expected_live_head_previous_snapshot_txg = expected
            .contents
            .has_trees()
            .then_some(expected.previous_snapshot_txg_of_the_live_head);
        if expected_live_head_previous_snapshot_txg != observed.live_head_previous_snapshot_txg {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::LiveHeadPreviousSnapshotTxg,
                format!(
                    "{:?} 的活头 previous_snapshot_txg {:?}",
                    expected.key,
                    expected_live_head_previous_snapshot_txg.map(|txg| txg.0)
                ),
                format!(
                    "{:?}",
                    observed.live_head_previous_snapshot_txg.map(|txg| txg.0)
                ),
            ));
        }
        let expected_snapshots: Vec<ObservedSnapshot> = expected
            .snapshots
            .iter()
            .map(|snapshot| ObservedSnapshot {
                birth_txg: snapshot.birth_txg,
                previous_snapshot_txg: snapshot.previous_snapshot_txg,
                is_being_destroyed: snapshot.is_being_destroyed,
                inode_tree_written_by: snapshot.inode_tree_written_by,
                extent_root_written_at: snapshot.extent_root_written_at,
            })
            .collect();
        if expected_snapshots != observed.snapshots {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::Snapshots,
                format!("{:?} 的快照 {expected_snapshots:?}", expected.key),
                format!("{:?}", observed.snapshots),
            ));
        }
        let mut expected_deadlist_entries = expected.deadlist.as_ref().clone();
        expected_deadlist_entries.sort_unstable();
        let mut observed_deadlist_entries = observed.deadlist_entries.clone();
        observed_deadlist_entries.sort_unstable();
        if expected_deadlist_entries != observed_deadlist_entries {
            let first_difference = expected_deadlist_entries
                .iter()
                .zip(&observed_deadlist_entries)
                .position(|(expected_entry, observed_entry)| expected_entry != observed_entry)
                .unwrap_or_else(|| {
                    expected_deadlist_entries
                        .len()
                        .min(observed_deadlist_entries.len())
                });
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::DeadlistEntries,
                format!(
                    "{:?} 的 deadlist {} 条，按 key 排第 {first_difference} 条起：{:?}",
                    expected.key,
                    expected_deadlist_entries.len(),
                    &expected_deadlist_entries[first_difference..]
                ),
                format!(
                    "{} 条，第 {first_difference} 条起：{:?}",
                    observed_deadlist_entries.len(),
                    &observed_deadlist_entries[first_difference..]
                ),
            ));
        }
        if expected.pinned_mapping_entries != observed.pinned_mapping_entries {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::PinnedMappingEntries,
                format!(
                    "{:?} 被钉住的映射条目 {} 条",
                    expected.key, expected.pinned_mapping_entries
                ),
                format!("{} 条", observed.pinned_mapping_entries),
            ));
        }
        counts.snapshot_states_compared += 1;
        counts.snapshots_compared += u64::try_from(expected_snapshots.len()).expect("个数");
        if !expected_deadlist_entries.is_empty() {
            counts.versions_with_deadlist_entries_compared += 1;
        }
        Ok(())
    }

    /// 这一版的文件内容与模型记的逐字节相同（有没有文件已经比过）。
    fn judge_file_content(
        expected: &ModelRoot,
        observed: &ObservedFile,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        match (expected.contents.live_file(), observed) {
            (None, ObservedFile::NoFile) => Ok(()),
            (Some(file), ObservedFile::Decoded(content)) => {
                counts.file_contents_compared += 1;
                if file.content.as_ref() == content.as_slice() {
                    Ok(())
                } else {
                    Err(ModelDisagreement::new(
                        ModelDisagreementAspect::FileContent,
                        format!(
                            "{:?} 的文件是 {:?} 写的 {} 字节（末字节 {:?}）",
                            expected.key,
                            file.written_by,
                            file.content.len(),
                            file.content.last()
                        ),
                        format!("{} 字节（末字节 {:?}）", content.len(), content.last()),
                    ))
                }
            }
            (Some(file), ObservedFile::Undecodable { what }) => Err(ModelDisagreement::new(
                ModelDisagreementAspect::FileContent,
                format!(
                    "{:?} 的文件是 {:?} 写的 {} 字节",
                    expected.key,
                    file.written_by,
                    file.content.len()
                ),
                format!("交回的单元里解不出内容：{what}"),
            )),
            (None, ObservedFile::Decoded(_) | ObservedFile::Undecodable { .. })
            | (Some(_), ObservedFile::NoFile) => Err(ModelDisagreement::new(
                ModelDisagreementAspect::FilePresence,
                format!("{:?} 里有什么：{:?}", expected.key, expected.contents),
                format!("有文件：{}", observed.has_file()),
            )),
        }
    }

    /// 这一版的整张实例表与模型记的逐行相同；实现交回的东西里没有这一版整条链的字节时只计数——只许出现在这次没重写实例表的那几版
    /// （mkfs 之后第一次重写之前、从盘上重建且多于一片、零单元发布）。这次重写了实例表链的那一版（写行：带文件的与树表 0 条的）
    /// 实现写出了整条链，比不了就是对不上（代码审阅第 12 条：每一版比实例表）。
    fn judge_instance_table(
        expected: &ModelRoot,
        observed: &ObservedInstanceTable,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        match observed {
            ObservedInstanceTable::Rows(rows) => {
                counts.instance_tables_compared += 1;
                if expected.instance_table_rows.as_slice() == rows.as_slice() {
                    Ok(())
                } else {
                    Err(ModelDisagreement::new(
                        ModelDisagreementAspect::InstanceTableOfTheVersion,
                        format!(
                            "{:?} 的实例表 {:?}",
                            expected.key,
                            expected.instance_table_rows.as_slice()
                        ),
                        format!("{rows:?}"),
                    ))
                }
            }
            ObservedInstanceTable::NotInTheOutput { why } => {
                let rewritten_by_this_publish =
                    expected.role_written_at.get(&ModelUnitRole::InstanceTable)
                        == Some(&expected.key.checkpoint_txg);
                if rewritten_by_this_publish {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::InstanceTableOfTheVersion,
                        format!(
                            "{:?} 这次重写了实例表链（{:?}）：实现要交回它的字节",
                            expected.key,
                            expected.instance_table_rows.as_slice()
                        ),
                        format!("比不了：{why}"),
                    ));
                }
                counts.instance_tables_not_in_the_output += 1;
                Ok(())
            }
            ObservedInstanceTable::Undecodable { what } => Err(ModelDisagreement::new(
                ModelDisagreementAspect::InstanceTableOfTheVersion,
                format!(
                    "{:?} 的实例表 {:?}",
                    expected.key,
                    expected.instance_table_rows.as_slice()
                ),
                format!("交回的实例表单元解不开：{what}"),
            )),
        }
    }

    pub(super) fn write_root(&mut self, root: ModelRoot) {
        self.roots_written_since_make_filesystem += 1;
        self.highest_published_txg = self.highest_published_txg.max(root.key.checkpoint_txg);
        self.highest_journal_counter = self.highest_journal_counter.max(root.journal_counter);
        self.ring.insert(
            ring_position_of(root.key.checkpoint_txg, self.slots_per_region),
            root,
        );
    }

    pub(super) fn advance_by_success(&mut self, answer: &ModelAnswer) {
        for root in &answer.expected_roots {
            self.write_root(root.clone());
        }
        match answer.session_after_success {
            ModelSessionAfterSuccess::StaysOpen => {
                if let (Some(session), Some(last)) =
                    (self.session.as_mut(), answer.expected_roots.last())
                {
                    session.current = last.clone();
                }
            }
            ModelSessionAfterSuccess::OpenedAs(instance) => {
                self.highest_acquired_instance = instance;
                let current = answer
                    .expected_roots
                    .last()
                    .expect("挂载至少写出写行那一次")
                    .clone();
                self.session = Some(ModelSession { instance, current });
            }
            ModelSessionAfterSuccess::Closed => self.session = None,
        }
    }
}

fn describe_answer(answer: &ModelAnswer) -> String {
    if answer.required_refusals.is_empty() {
        format!(
            "{:?} 该成（写出 {:?}；允许拒的只有容量墙区间）",
            answer.operation,
            answer
                .expected_roots
                .iter()
                .map(|root| (root.key.checkpoint_txg.0, root.key.instance.0))
                .collect::<Vec<_>>()
        )
    } else {
        let required: Vec<&'static str> = answer
            .required_refusals
            .iter()
            .map(|reason| reason.name())
            .collect();
        format!("{:?} 该拒：{required:?}", answer.operation)
    }
}

pub(super) fn describe_snapshot_read_back(read_back: &ModelSnapshotReadBack) -> String {
    match read_back {
        ModelSnapshotReadBack::NoFile => "没有文件".to_string(),
        ModelSnapshotReadBack::File(content) => {
            format!("{} 字节（末字节 {:?}）", content.len(), content.last())
        }
    }
}

pub(super) fn describe_observed_snapshot_read_back(read_back: &ObservedSnapshotReadBack) -> String {
    match read_back {
        ObservedSnapshotReadBack::NoFile => "读出没有文件".to_string(),
        ObservedSnapshotReadBack::File(content) => {
            format!("读出 {} 字节（末字节 {:?}）", content.len(), content.last())
        }
        ObservedSnapshotReadBack::NoSnapshotWithThisTree => "树表里没有这个快照".to_string(),
        ObservedSnapshotReadBack::Failed { what } => format!("读失败：{what}"),
    }
}

fn describe_model_read_back(read_back: &ModelReadBack) -> String {
    match read_back {
        ModelReadBack::NoFile { root } => format!("所选根 {root:?}，没有文件"),
        ModelReadBack::FileRead { root, content } => format!(
            "所选根 {root:?}，读回 {} 字节（末字节 {:?}）",
            content.len(),
            content.last()
        ),
    }
}

pub(super) fn describe_observed_read_back(read_back: &ObservedReadBack) -> String {
    match read_back {
        ObservedReadBack::NoFile { root } => format!("所选根 {root:?}，没有文件"),
        ObservedReadBack::FileRead { root, content } => format!(
            "所选根 {root:?}，读回 {} 字节（末字节 {:?}）",
            content.len(),
            content.last()
        ),
        ObservedReadBack::Failed { what } => format!("读回失败：{what}"),
    }
}

/// 交来的做成结局是哪一种（判兼容那一格的报告用）。
fn describe_observed_effect_kind(effect: &ObservedEffect) -> &'static str {
    match effect {
        ObservedEffect::Publishes { .. } => "发布（Publishes）",
        ObservedEffect::Mount { .. } => "挂载（Mount）",
        ObservedEffect::ColdStart { .. } => "冷启动读回（ColdStart）",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 可写挂载写行那一版（txg 4）：第一个文件（txg 3，内容 [3]）之后关掉、再挂一次。
    fn row_publish_of_a_mount_after_the_first_file() -> (IdealModel, ModelRoot) {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model.answer_publish_first_file(&[3]).expect("会话开着");
        succeed(&mut model, &first);
        model.close_session();
        let mount = model.answer_mount_writable();
        let row_publish = mount.expected_roots[0].clone();
        (model, row_publish)
    }

    /// 分配代对拍两个方向都比（代码审阅第 12 条）：模型这一版有、实现没交回的角色也报——只遍历实现交回的角色会漏判，
    /// 写行那一版九个角色只交两个也判对得上。
    #[test]
    fn role_of_the_version_that_the_implementation_did_not_hand_in_is_reported() {
        let (model, row_publish) = row_publish_of_a_mount_after_the_first_file();
        let mut observed = observed_exactly_as_the_model_wrote(&row_publish);
        let mut counts = ModelJudgementCounts::default();
        assert_eq!(
            model.judge_roots(
                std::slice::from_ref(&row_publish),
                std::slice::from_ref(&observed),
                &mut counts
            ),
            Ok(()),
            "原样交回九个角色对得上"
        );
        let ObservedUnitAllocationRecords::EveryRoleOfTheVersion(records) =
            &mut observed.unit_allocation_records
        else {
            panic!("照模型写的是带文件的那一版的全部角色");
        };
        records
            .roles
            .retain(|(role, _)| *role != ModelUnitRole::InodeRoot);
        let missing = model
            .judge_roots(
                std::slice::from_ref(&row_publish),
                std::slice::from_ref(&observed),
                &mut counts,
            )
            .expect_err("inode 根这一版有，实现没交");
        assert_eq!(
            missing.aspect,
            ModelDisagreementAspect::AllocationGeneration
        );
        assert!(
            missing.model_answer.contains("InodeRoot"),
            "点名漏交的那个角色：{missing:?}"
        );
    }

    /// 数据单元按单元号逐个比（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K4）：漏交一个单元、单元号重了，都报单元的分配代
    /// 对不上，点名交回的单元号。
    #[test]
    fn data_unit_the_implementation_did_not_hand_in_or_handed_in_twice_is_reported() {
        let (model, row_publish) = row_publish_of_a_mount_after_the_first_file();
        let mut observed = observed_exactly_as_the_model_wrote(&row_publish);
        let ObservedUnitAllocationRecords::EveryRoleOfTheVersion(records) =
            &mut observed.unit_allocation_records
        else {
            panic!("照模型写的是带文件的那一版的全部单元");
        };
        let only_data_unit = records.data_units.pop().expect("第一个文件一个数据单元");
        let mut counts = ModelJudgementCounts::default();
        let missing = model
            .judge_allocation_generations(&row_publish, &observed, &mut counts)
            .expect_err("单元 0 这一版有，实现没交");
        assert_eq!(
            missing.aspect,
            ModelDisagreementAspect::AllocationGeneration
        );
        assert!(
            missing.implementation_answer.contains("[]"),
            "点名交回的单元号：{missing:?}"
        );
        let ObservedUnitAllocationRecords::EveryRoleOfTheVersion(records_handed_in_twice) =
            &mut observed.unit_allocation_records
        else {
            panic!("照模型写的是带文件的那一版的全部单元");
        };
        records_handed_in_twice
            .data_units
            .push(only_data_unit.clone());
        records_handed_in_twice.data_units.push(only_data_unit);
        let twice = model
            .judge_allocation_generations(&row_publish, &observed, &mut counts)
            .expect_err("单元 0 交了两次");
        assert!(
            twice.implementation_answer.contains("[0, 0]"),
            "点名交回的单元号：{twice:?}"
        );
    }

    /// 树表 0 条的一版上：零单元发布（暖机）一个字节都不写，输出不带实例表与分配记录，比「这次重写了哪几个角色」（空集），
    /// 多报一个就报；写行那一版（这次重写实例表与这一版自己的分配记录树）写出了整条实例表链与整版分配记录，交回「比不了」的两臂
    /// 就是对不上（代码审阅第 12 条：每一版比实例表、比分配代）。
    #[test]
    fn only_a_zero_unit_publish_on_a_version_without_file_may_hand_in_the_rewritten_roles_only() {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        model.close_session();
        let mount = model.answer_mount_writable();
        let row_publish = mount.expected_roots[0].clone();
        let zero_unit_publish = mount.expected_roots[1].clone();
        assert!(
            !row_publish.contents.has_trees() && !zero_unit_publish.contents.has_trees(),
            "起点那一版树表 0 条"
        );
        let observed_with = |root: &ModelRoot,
                             instance_table: ObservedInstanceTable,
                             roles: &[ModelUnitRole]| ObservedRoot {
            key: root.key,
            journal_counter: root.journal_counter,
            rollback_floor: root.rollback_floor,
            file: ObservedFile::NoFile,
            instance_table,
            unit_allocation_records: ObservedUnitAllocationRecords::RewrittenRolesOnly(
                roles.iter().copied().collect(),
            ),
            snapshot_state: ObservedSnapshotState::OF_A_VERSION_WITHOUT_TREES,
            logged_compaction_intent: None,
            first_file_record_truncation_field: None,
            inodes_other_than_the_first: BTreeMap::new(),
            truncation_fields_of_the_inodes_other_than_the_first: BTreeMap::new(),
        };
        let mut counts = ModelJudgementCounts::default();
        assert_eq!(
            model.judge_roots(
                std::slice::from_ref(&zero_unit_publish),
                &[observed_with(
                    &zero_unit_publish,
                    ObservedInstanceTable::NotInTheOutput { why: "用例" },
                    &[]
                )],
                &mut counts
            ),
            Ok(())
        );
        assert_eq!(counts.rewritten_role_sets_compared, 1);
        assert_eq!(counts.instance_tables_not_in_the_output, 1);
        let one_role_too_many = model
            .judge_roots(
                std::slice::from_ref(&zero_unit_publish),
                &[observed_with(
                    &zero_unit_publish,
                    ObservedInstanceTable::NotInTheOutput { why: "用例" },
                    &[ModelUnitRole::InstanceTable],
                )],
                &mut counts,
            )
            .expect_err("零单元发布多报一个重写的角色");
        assert_eq!(
            one_role_too_many.aspect,
            ModelDisagreementAspect::AllocationGeneration
        );
        let row_publish_without_its_instance_table = model
            .judge_roots(
                std::slice::from_ref(&row_publish),
                &[observed_with(
                    &row_publish,
                    ObservedInstanceTable::NotInTheOutput { why: "用例" },
                    &[ModelUnitRole::InstanceTable, ModelUnitRole::AllocationTree],
                )],
                &mut counts,
            )
            .expect_err("写行那一版交回「实例表比不了」");
        assert_eq!(
            row_publish_without_its_instance_table.aspect,
            ModelDisagreementAspect::InstanceTableOfTheVersion
        );
        let row_publish_with_the_rewritten_roles_only = model
            .judge_roots(
                std::slice::from_ref(&row_publish),
                &[observed_with(
                    &row_publish,
                    ObservedInstanceTable::Rows(row_publish.instance_table_rows.as_ref().clone()),
                    &[ModelUnitRole::InstanceTable, ModelUnitRole::AllocationTree],
                )],
                &mut counts,
            )
            .expect_err("写行那一版只交回重写的角色（角色集合与模型的相同）");
        assert_eq!(
            row_publish_with_the_rewritten_roles_only.aspect,
            ModelDisagreementAspect::AllocationGeneration
        );
        assert_eq!(
            counts.instance_tables_not_in_the_output, 2,
            "写行那一版的「比不了」没有计进比不了的次数"
        );
    }

    /// 每一版都比文件内容（代码审阅第 12 条：内容只在冷启动那一步比会漏判）：写行那一版照抄第一个文件的内容 [3]。
    #[test]
    fn each_version_compares_its_file_content() {
        let (model, row_publish) = row_publish_of_a_mount_after_the_first_file();
        let mut counts = ModelJudgementCounts::default();
        let mut observed = observed_exactly_as_the_model_wrote(&row_publish);
        assert_eq!(
            model.judge_roots(
                std::slice::from_ref(&row_publish),
                std::slice::from_ref(&observed),
                &mut counts
            ),
            Ok(())
        );
        assert_eq!(counts.file_contents_compared, 1);
        for wrong in [
            ObservedFile::Decoded(vec![4]),
            ObservedFile::Undecodable {
                what: "用例".to_string(),
            },
        ] {
            observed.file = wrong.clone();
            let disagreement = model
                .judge_roots(
                    std::slice::from_ref(&row_publish),
                    std::slice::from_ref(&observed),
                    &mut counts,
                )
                .expect_err("内容不是 [3]");
            assert_eq!(
                disagreement.aspect,
                ModelDisagreementAspect::FileContent,
                "{wrong:?}"
            );
        }
    }

    /// 每一版都比整张实例表：写行那一版有实例 1 那一行；少了它报。输出里没有实例表字节的，这次没重写实例表的那一版（暖机）只计数不判，
    /// 这次重写了实例表链的那一版（写行）报「这一版的实例表」（实审 B3c-3：写行那一版写出了整条链，比不了就是对不上）。
    #[test]
    fn each_version_compares_its_instance_table() {
        let (model, row_publish) = row_publish_of_a_mount_after_the_first_file();
        assert_eq!(row_publish.instance_table_rows.len(), 1, "实例 1 那一行");
        let warm_up = model.answer_mount_writable().expected_roots[1].clone();
        assert_eq!(
            warm_up.role_written_at.get(&ModelUnitRole::InstanceTable),
            row_publish
                .role_written_at
                .get(&ModelUnitRole::InstanceTable),
            "暖机那一版照抄写行那一版的实例表，不重写它"
        );
        let mut counts = ModelJudgementCounts::default();
        let mut observed = observed_exactly_as_the_model_wrote(&row_publish);
        assert_eq!(
            model.judge_roots(
                std::slice::from_ref(&row_publish),
                std::slice::from_ref(&observed),
                &mut counts
            ),
            Ok(())
        );
        assert_eq!(counts.instance_tables_compared, 1);
        observed.instance_table = ObservedInstanceTable::Rows(Vec::new());
        let disagreement = model
            .judge_roots(
                std::slice::from_ref(&row_publish),
                std::slice::from_ref(&observed),
                &mut counts,
            )
            .expect_err("实例表丢了那一行");
        assert_eq!(
            disagreement.aspect,
            ModelDisagreementAspect::InstanceTableOfTheVersion
        );
        observed.instance_table = ObservedInstanceTable::NotInTheOutput { why: "用例" };
        let row_publish_not_in_the_output = model
            .judge_roots(
                std::slice::from_ref(&row_publish),
                std::slice::from_ref(&observed),
                &mut counts,
            )
            .expect_err("写行那一版这次重写了实例表链，交回「比不了」");
        assert_eq!(
            row_publish_not_in_the_output.aspect,
            ModelDisagreementAspect::InstanceTableOfTheVersion
        );
        assert_eq!(counts.instance_tables_not_in_the_output, 0);
        let mut warm_up_observed = observed_exactly_as_the_model_wrote(&warm_up);
        warm_up_observed.instance_table = ObservedInstanceTable::NotInTheOutput { why: "用例" };
        assert_eq!(
            model.judge_roots(
                std::slice::from_ref(&warm_up),
                std::slice::from_ref(&warm_up_observed),
                &mut counts
            ),
            Ok(())
        );
        assert_eq!(counts.instance_tables_not_in_the_output, 1);
    }

    /// 单元区墙的区间按单元区起点随环长走算（C475（非默认环长下单元区起点取编译期常量），实审 A3b 第 15 条）：6 MiB 的环，单元区从槽 1408 起；
    /// 盘只比默认环长那一档的单元区起点（编译期常量那个 50176）多 1000 槽。第一个文件之后一次覆盖写的上界乘 64 落在两道墙之间：
    /// 按随环长走的起点，单元区有四万九千多槽，离墙还远，实现报单元区墙就是对不上；按编译期常量取起点，单元区只剩 1000 槽，同一个拒绝落在区间里放行。
    /// 随机历史的单元区墙取样点（两块单元区 256 槽的小盘、准入关掉的那一段）一次单元区墙的拒绝都没有，两个起点在它上面判得一样（取样点不敏感）。
    #[test]
    fn unit_area_wall_refusal_far_below_the_wall_that_follows_the_ring_is_judged_a_disagreement() {
        let journal_ring_bytes = 6 << 20;
        let slots_past_the_unit_area_start_of_the_default_ring = 1000;
        let device_slots = model_unit_area_start_slot(JOURNAL_RING_DEFAULT_BYTES)
            + slots_past_the_unit_area_start_of_the_default_ring;
        let mut model = IdealModel::after_make_filesystem_on_a_journal_ring_of(
            ModelPoolGeometry {
                devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
                device_size_in_bytes: device_slots * SLOT_BYTES,
            },
            journal_ring_bytes,
        );
        assert_eq!(
            model.unit_area_slots_per_device(),
            device_slots - 1408,
            "6 MiB 的环末尾的下一个槽 1024 + 384 = 1408 起"
        );
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        succeed(&mut model, &first);
        let overwrite = model
            .answer_publish_overwrite(&[4])
            .expect("会话开着、现行版本带文件");
        let planned_slots = overwrite.planned_publish_upper_bounds[0].occupied_slots_per_device;
        assert!(
            planned_slots * CLUSTER_SEGMENT_SLOTS
                > slots_past_the_unit_area_start_of_the_default_ring
                && planned_slots * CLUSTER_SEGMENT_SLOTS <= device_slots - 1408,
            "取样点：上界 {planned_slots} 槽乘 64 落在两道墙之间"
        );
        let disagreement = model
            .judge_and_advance(
                &overwrite,
                &ObservedOutcome::Refused {
                    member: "PublishError::PlacementRefused(NoPlacementOnAnyDevice)".to_string(),
                    reason: ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall),
                    publishes_completed: 0,
                    wrote_anything: false,
                    reported_ceiling: None,
                    reported_root_ring_slot_still_bad_after_one_reread: None,
                },
            )
            .expect_err("离墙还远：单元区墙的拒绝不在允许的区间里");
        assert_eq!(
            disagreement.aspect,
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
        );
    }

    /// 快照那几样逐项比（判决 S7）：实现交回的这一版照模型记的原样时对得上；活头前驱、快照列表、deadlist 条目数、被钉映射条目数各改一样，
    /// 各报自己那一格。
    #[test]
    fn each_snapshot_field_of_a_version_that_differs_is_reported_as_its_own_aspect() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let creation = model.answer_create_snapshot().expect("建快照");
        succeed(&mut model, &creation);
        let overwrite = model.answer_publish_overwrite(&[10]).expect("覆盖写");
        succeed(&mut model, &overwrite);
        let version = model
            .current_version_in_the_session()
            .expect("会话开着")
            .clone();
        let aspect_of = |observed: ObservedRoot| {
            model
                .judge_roots(
                    std::slice::from_ref(&version),
                    &[observed],
                    &mut ModelJudgementCounts::default(),
                )
                .map_err(|disagreement| disagreement.aspect)
        };
        assert_eq!(
            aspect_of(observed_exactly_as_the_model_wrote(&version)),
            Ok(())
        );
        let mut wrong_live_head = observed_exactly_as_the_model_wrote(&version);
        wrong_live_head
            .snapshot_state
            .live_head_previous_snapshot_txg = Some(ModelCheckpointTxg(0));
        assert_eq!(
            aspect_of(wrong_live_head),
            Err(ModelDisagreementAspect::LiveHeadPreviousSnapshotTxg)
        );
        let mut wrong_snapshots = observed_exactly_as_the_model_wrote(&version);
        wrong_snapshots.snapshot_state.snapshots[0].is_being_destroyed = true;
        assert_eq!(
            aspect_of(wrong_snapshots),
            Err(ModelDisagreementAspect::Snapshots)
        );
        let mut wrong_deadlist = observed_exactly_as_the_model_wrote(&version);
        wrong_deadlist
            .snapshot_state
            .deadlist_entries
            .push(ModelDeadlistEntry {
                epoch: ModelCheckpointTxg(0),
                mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_DATA,
                mapping_key_birth_tree: TREE_IDENTIFIER_EXTENT,
                birth_txg: ModelCheckpointTxg(0),
            });
        assert_eq!(
            aspect_of(wrong_deadlist),
            Err(ModelDisagreementAspect::DeadlistEntries)
        );
        // 条数相同、一条的类标签不同（一批放错一条就是这个样子）：逐条比照样报 deadlist 那一格。
        let mut wrong_deadlist_entry = observed_exactly_as_the_model_wrote(&version);
        wrong_deadlist_entry.snapshot_state.deadlist_entries[0].mapping_key_unit_class =
            MODEL_MAPPING_KEY_UNIT_CLASS_PACKED_RECORDS;
        assert_eq!(
            aspect_of(wrong_deadlist_entry),
            Err(ModelDisagreementAspect::DeadlistEntries),
            "条数相同、有一条不同也报"
        );
        let mut wrong_pinned_mapping_entries = observed_exactly_as_the_model_wrote(&version);
        wrong_pinned_mapping_entries
            .snapshot_state
            .pinned_mapping_entries += 1;
        assert_eq!(
            aspect_of(wrong_pinned_mapping_entries),
            Err(ModelDisagreementAspect::PinnedMappingEntries)
        );
    }

    /// 对拍比字段（判决 m4-trunc-r2 第三节 R6「理想模型记字段并在对拍里比」）：截断意图那一版实现交回的记录里两段全零（没写字段）时报
    /// 截断字段对不上；照模型写的交回对得上、计数加一；模型这一版有那条记录而实现交回没有，也报。
    #[test]
    fn truncation_field_the_implementation_did_not_write_is_reported() {
        let model = model_with_a_file_of(3 * payload());
        let answer = model
            .answer_truncation(u64::try_from(payload() + 5).expect("长度"))
            .expect("会话开着、现行版本带活文件");
        let intent = answer.expected_roots[0].clone();
        let mut observed = observed_exactly_as_the_model_wrote(&intent);
        let mut counts = ModelJudgementCounts::default();
        assert_eq!(
            IdealModel::judge_first_file_record_truncation_field(&intent, &observed, &mut counts),
            Ok(())
        );
        assert_eq!(
            (
                counts.truncation_fields_compared,
                counts.written_truncation_fields_compared
            ),
            (1, 1)
        );
        observed.first_file_record_truncation_field =
            Some(ModelTruncationField::NotWritten.recorded_values());
        let not_written =
            IdealModel::judge_first_file_record_truncation_field(&intent, &observed, &mut counts)
                .expect_err("实现没写字段");
        assert_eq!(
            not_written.aspect,
            ModelDisagreementAspect::FirstFileRecordTruncationField
        );
        observed.first_file_record_truncation_field = None;
        let missing =
            IdealModel::judge_first_file_record_truncation_field(&intent, &observed, &mut counts)
                .expect_err("模型这一版有那条记录，实现交了没有");
        assert_eq!(
            missing.aspect,
            ModelDisagreementAspect::FirstFileRecordTruncationField
        );
    }
}
