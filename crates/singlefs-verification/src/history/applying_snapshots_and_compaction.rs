//! 施加建、删、读快照，挂着整理与零单元发布。

use super::*;

/// 现行那一版树表里的只读快照（码 14 条目），按（诞生 txg, 树号）升序——建的先后，与模型的快照列表同一个次序：树号与带不带「销毁中」记号。
fn snapshots_in_the_version(version: &TransactionOutput) -> Vec<(TreeIdentifier, bool)> {
    let mut snapshot_inode_roots: Vec<_> = version
        .tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .collect();
    snapshot_inode_roots.sort_by_key(|entry| (entry.birth_txg, entry.tree));
    snapshot_inode_roots
        .into_iter()
        .map(|entry| {
            (
                entry.tree,
                version.snapshots_being_destroyed.contains(&entry.tree),
            )
        })
        .collect()
}

/// 经会话发一次快照改动（建快照、删快照）：会话借走这个进程的分配器、现行版本与认下的池，发完原样还回来（同 unlink），
/// 交回会话的结局与改动之前的分配记录、录制流长度。
fn publish_a_snapshot_change_through_the_session(
    devices: &mut Vec<(DeviceIdentity, HistoryDevice)>,
    session_slot: &mut Option<WritableSession>,
    stream: &SharedStream,
    change: UserChange<'_>,
) -> (
    Result<UserChangePublished, UserChangeRefused>,
    Vec<AllocationRecord>,
    usize,
) {
    let WritableSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        publishes_in_this_mount,
        carried_to_zero_unit_publishes: _,
        compaction,
    } = session_slot.take().expect("调用方判过这个进程的会话开着");
    let records_before = allocator.records().to_vec();
    let stream_length_before = stream.operation_count();
    let mut mounted_session = MountedSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        compaction,
    };
    let change_result = mounted_session.publish_user_change(devices, change);
    *session_slot = Some(WritableSession {
        allocator: mounted_session.allocator,
        current: mounted_session.current,
        instance: mounted_session.instance,
        shadow_ledger: mounted_session.shadow_ledger,
        parameters_and_device_table: mounted_session.parameters_and_device_table,
        compaction: mounted_session.compaction,
        publishes_in_this_mount,
        // 零单元发布只接在树表 0 条的一版后面，快照改动只接在有树的一版后面。
        carried_to_zero_unit_publishes: None,
    });
    (change_result, records_before, stream_length_before)
}

/// 快照改动经会话做成时把「发成」那一种结局换成带快照树号的那一种（RefFS 按树号点名快照）；别的结局原样交回。
fn snapshot_change_applied(
    applied: AppliedStep,
    change: SnapshotChange,
    snapshot: u64,
) -> AppliedStep {
    let outcome = match applied.outcome {
        StepOutcome::Applied(AppliedEffect::Published {
            reuse,
            floor_raises_pushed_by_the_session,
            roots_carrying_the_change: _,
        }) => StepOutcome::Applied(AppliedEffect::SnapshotPublished {
            change,
            snapshot,
            reuse,
            floor_raises_pushed_by_the_session,
        }),
        StepOutcome::Applied(
            effect @ (AppliedEffect::Mounted { .. }
            | AppliedEffect::RaisedFloor { .. }
            | AppliedEffect::RolledBack { .. }
            | AppliedEffect::Recovered { .. }
            | AppliedEffect::SnapshotPublished { .. }
            | AppliedEffect::SnapshotRead { .. }
            | AppliedEffect::Compacted { .. }),
        ) => StepOutcome::Applied(effect),
        refused_or_not_applicable @ (StepOutcome::Refused { .. }
        | StepOutcome::NotApplicable(_)) => refused_or_not_applicable,
    };
    AppliedStep {
        outcome,
        harness_judgement: applied.harness_judgement,
        model_verdict: applied.model_verdict,
    }
}

/// 建一个只读快照（`HistoryOperation::CreateSnapshot`）：经挂着的会话发（`UserChange::CreateSnapshot`，先走空再发一次独立发布）。
/// 前提：现行那一版有树（有树而文件集合为空的一版照调：会话照常发布「放开」，判决
/// `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F5）。做成之后新快照的树号取现行那一版树表里最新的那条码 14。
pub(super) fn apply_create_snapshot(pool: &mut HistoryPool) -> AppliedStep {
    let HistoryPool {
        devices,
        session: session_slot,
        model,
        stream,
        copies_quarantined_in_this_step,
        ..
    } = pool;
    let Some(open_session) = session_slot.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    if open_session.current.version_of_trees().is_none() {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
    }
    let (change_result, records_before, stream_length_before) =
        publish_a_snapshot_change_through_the_session(
            devices,
            session_slot,
            stream,
            UserChange::CreateSnapshot,
        );
    copies_quarantined_in_this_step.extend(copies_quarantined_by_a_user_change(&change_result));
    let session_after_the_change = session_slot.as_mut().expect("会话发完原样还回来");
    let applied = settle_user_change_comparing_every_version_published(
        session_after_the_change,
        model,
        &IdealModel::answer_create_snapshot,
        &records_before,
        &change_result,
        stream,
        stream_length_before,
    );
    let created = match &applied.outcome {
        StepOutcome::Applied(_) => session_after_the_change
            .current
            .version_of_trees()
            .and_then(|version| snapshots_in_the_version(version).last().copied())
            .map(|(tree, _)| tree.0)
            .expect("建快照做成之后现行那一版树表里有它那条码 14（建的最晚，排在最后）"),
        StepOutcome::Refused { .. } | StepOutcome::NotApplicable(_) => return applied,
    };
    snapshot_change_applied(applied, SnapshotChange::Created, created)
}

/// 删一个只读快照（`HistoryOperation::DestroySnapshot`）：经挂着的会话发（`UserChange::DestroySnapshot`，先走空 + 意图 + 全部批）。
/// 点名的是现行那一版里不在销毁中的快照的第 `index mod 个数` 个；会话先走空会把销毁中的那几个走完，所以它在模型那一侧（走空之后）的位置
/// 就是它在不在销毁中的那几个里的位置。前提：现行那一版有树、有这样的快照。
pub(super) fn apply_destroy_snapshot(
    pool: &mut HistoryPool,
    choice: SnapshotChoice,
) -> AppliedStep {
    let HistoryPool {
        devices,
        session: session_slot,
        model,
        stream,
        copies_quarantined_in_this_step,
        ..
    } = pool;
    let Some(open_session) = session_slot.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let Some(version) = open_session.current.version_of_trees() else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    let candidates: Vec<TreeIdentifier> = snapshots_in_the_version(version)
        .into_iter()
        .filter(|(_, is_being_destroyed)| !is_being_destroyed)
        .map(|(tree, _)| tree)
        .collect();
    let Some(position) = choose_among(candidates.len(), choice) else {
        return AppliedStep::not_applicable(
            MissingPrecondition::NoSnapshotOutsideTheDestructionInTheCurrentVersion,
        );
    };
    let snapshot = candidates[position];
    let (change_result, records_before, stream_length_before) =
        publish_a_snapshot_change_through_the_session(
            devices,
            session_slot,
            stream,
            UserChange::DestroySnapshot { snapshot },
        );
    copies_quarantined_in_this_step.extend(copies_quarantined_by_a_user_change(&change_result));
    let session_after_the_change = session_slot.as_mut().expect("会话发完原样还回来");
    let applied = settle_user_change_comparing_every_version_published(
        session_after_the_change,
        model,
        &|ideal_model: &IdealModel| ideal_model.answer_destroy_snapshot(position),
        &records_before,
        &change_result,
        stream,
        stream_length_before,
    );
    snapshot_change_applied(applied, SnapshotChange::Destroyed, snapshot.0)
}

/// 在 `count` 个里按选择子取第几个；一个都没有是 None。
fn choose_among(count: usize, choice: SnapshotChoice) -> Option<usize> {
    let count_as_u64 = u64::try_from(count).expect("快照至多十个");
    (count_as_u64 > 0).then(|| {
        usize::try_from(choice.index_among_the_snapshots % count_as_u64).expect("小于个数")
    })
}

/// 读一个只读快照（`HistoryOperation::ReadSnapshot`）：经现行那一版的根读（`recovery::read_the_first_file_of_a_snapshot`，读不写盘），
/// 读出的交模型比（建它那一刻读得出的内容）。点名的是不在销毁中的快照的第 `index mod 个数` 个，它在模型那一侧的位置是它在全部快照里的位置。
pub(super) fn apply_read_snapshot(pool: &mut HistoryPool, choice: SnapshotChoice) -> AppliedStep {
    let HistoryPool {
        devices,
        session,
        model,
        ..
    } = pool;
    let Some(open_session) = session.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let Some(version) = open_session.current.version_of_trees() else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    let snapshots = snapshots_in_the_version(version);
    let positions_outside_the_destruction: Vec<usize> = snapshots
        .iter()
        .enumerate()
        .filter(|(_, (_, is_being_destroyed))| !is_being_destroyed)
        .map(|(position, _)| position)
        .collect();
    let Some(chosen) = choose_among(positions_outside_the_destruction.len(), choice) else {
        return AppliedStep::not_applicable(
            MissingPrecondition::NoSnapshotOutsideTheDestructionInTheCurrentVersion,
        );
    };
    let position = positions_outside_the_destruction[chosen];
    let (snapshot, _) = snapshots[position];
    let read_back = observed_snapshot_read_back(&read_the_first_file_of_a_snapshot(
        &*devices,
        open_session.current.root(),
        snapshot,
    ));
    let verdict = match model.judge_snapshot_read_back(position, &read_back) {
        Ok(counts) => ModelVerdict {
            disagreement: None,
            counts,
        },
        Err(disagreement) => ModelVerdict {
            disagreement: Some(disagreement),
            counts: ModelJudgementCounts::default(),
        },
    };
    AppliedStep::judged_by_outcome_and_model(
        StepOutcome::Applied(AppliedEffect::SnapshotRead {
            snapshot: snapshot.0,
            read_back,
        }),
        verdict,
    )
}

/// 现行那一版里整理能强制选的段（[`CompactionChoice::region_selector`] 在这里面挑），按段起点升序：0 号盘上装着这一版至少一条未释放分配记录的整段
/// （起点取 64 的倍数，`CompactionRegion::of_one_cluster_segment`），去掉这次挂载开着的聚簇段（`PoolAllocator::cluster_segments`：签发时那一判
/// 不合格）。别的资格（钉住的单元、隔离槽、被区域扫描拒过）照整理自己的判，不合格就停在「没有合格段」，一个写都不发。
fn segments_the_compaction_may_be_forced_onto(
    version: &TransactionOutput,
    allocator: &PoolAllocator,
) -> Vec<CompactionRegion> {
    let device_of_the_region = DeviceIdentity(0);
    let segment_starts: BTreeSet<u64> = version
        .allocation_records
        .iter()
        .filter(|record| record.device == device_of_the_region && !record.is_released)
        .map(|record| record.slot.0 - record.slot.0 % CLUSTER_SEGMENT_SLOTS)
        .filter(|segment_start| {
            !allocator
                .cluster_segments()
                .contains(&SlotNumber(*segment_start))
        })
        .collect();
    segment_starts
        .into_iter()
        .filter_map(|segment_start| {
            CompactionRegion::of_one_cluster_segment(
                device_of_the_region,
                SlotNumber(segment_start),
            )
            .ok()
        })
        .collect()
}

/// 这一趟整理装在会话分配器上的开关：批大小与强制选段取 `choice` 与执行器挑的段，别的取产品值（`CompactionSwitches::PRODUCT`）。
fn compaction_switches_of_the_choice(
    batch_units: CompactionBatchUnits,
    region: CompactionRegion,
) -> CompactionSwitches {
    let CompactionSwitches {
        batch_size: _,
        forced_region: _,
        run_forcing,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    CompactionSwitches {
        batch_size: CompactionBatchSize::ForcedForTests(batch_units),
        forced_region: Some(region),
        run_forcing,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    }
}

/// 整理的一批改写或新增的分配记录（[`records_changed_with_a_generation_outside_the_publish_txgs`] 的判法）：代要是这一批的 txg，
/// 只有搬迁的目的地例外——目的地那一条已分配记录的分配代写单元的诞生代（映射 key 的出生 txg 段，D26（后台整理与放置回收） 已定项 5 第 2 条）。
pub(super) fn allocation_generation_judgement_of_a_relocation_batch(
    before: &[AllocationRecord],
    batch: &TransactionOutput,
) -> Option<HarnessJudgement> {
    let publish_txg = batch.root.checkpoint_txg;
    let records = records_a_relocation_batch_changed_with_a_generation_outside_its_txg(
        before,
        &batch.allocation_records,
        publish_txg,
        &batch.units_relocated_by_this_publish,
        &batch.quarantined_after_release_checksum_mismatch,
    );
    (!records.is_empty()).then_some(HarnessJudgement::AllocationGenerationIsNotThePublishTxg {
        records,
        first_publish_txg: publish_txg,
        last_publish_txg: publish_txg,
    })
}

/// [`allocation_generation_judgement_of_a_relocation_batch`] 挑出的那几条：整理的一批（txg `publish_txg`）前后的分配记录里，代不是这一批的 txg、
/// 也不是搬迁目的地写的诞生代的；只改隔离记号的，置记号的只放过这一批实现交回的隔离（`copies_quarantined_by_the_batch`），清记号的照挑。
pub(super) fn records_a_relocation_batch_changed_with_a_generation_outside_its_txg(
    before: &[AllocationRecord],
    after: &[AllocationRecord],
    publish_txg: CheckpointTxg,
    units_relocated_by_the_batch: &[UnitRelocatedByThisPublish],
    copies_quarantined_by_the_batch: &[CopyQuarantinedAfterReleaseChecksumMismatch],
) -> Vec<AllocationRecord> {
    let destinations_at_their_birth: BTreeSet<(DeviceIdentity, u64, CheckpointTxg)> =
        units_relocated_by_the_batch
            .iter()
            .filter_map(|relocated| {
                let birth_txg = birth_txg_of_a_mapping_key(&relocated.mapping_key)?;
                Some(
                    relocated
                        .destination
                        .iter()
                        .map(move |location| (location.device, location.slot.0, birth_txg)),
                )
            })
            .flatten()
            .collect();
    records_changed_with_a_generation_outside_the_publish_txgs(
        before,
        after,
        publish_txg,
        publish_txg,
        QuarantineMarkChangesVouchedFor {
            copies_quarantined_by_the_publishes: copies_quarantined_by_the_batch,
            clearing: QuarantineMarkClearing::NotDoneByThisPublish,
        },
    )
    .into_iter()
    .filter(|record| {
        record.is_released
            || !destinations_at_their_birth.contains(&(
                record.device,
                record.slot.0,
                record.generation,
            ))
    })
    .collect()
}

/// 整理那一趟里实现交回的一版，模型此刻怎么答：现行那一版还有要走空的就是走空的一步；否则是整理的一次发布，是哪一种（签发、搬一批、
/// 随最后一批删意图、单独删意图或丢意图）由模型按它自己记的活意图与这一版看得见的几样判（[`IdealModel::answer_compaction_publish`]）：
/// 搬了几个单元、搬的里有没有用户可见树的根、发布之后 logged_ops 里还有没有活意图、模型那条活意图的 R 在这一版里按记录全空没有
/// （按实现交回的这一版的分配记录数，`compaction_plan::region_is_empty_by_the_records`）。签发的 R 取执行器强制的那一段。
fn model_answer_of_the_next_publish_of_the_compaction(
    model: &IdealModel,
    batch_units: u64,
    version: &TransactionOutput,
    region_forced_for_the_issue: CompactionRegion,
) -> Result<ModelAnswer, ModelDisagreement> {
    if model.current_version_has_a_step_to_walk() {
        return model.answer_next_step_of_the_walk();
    }
    let is_the_region_of_the_live_intent_empty_by_the_records_after_the_publish = model
        .live_compaction_intent_of_the_current_version()
        .and_then(|intent| compaction_region_of_the_model(intent.region))
        .is_some_and(|region| region_is_empty_by_the_records(&version.allocation_records, region));
    model.answer_compaction_publish(&ModelCompactionPublishSeen {
        units_at_most: batch_units,
        units_relocated: u64::try_from(version.units_relocated_by_this_publish.len())
            .expect("一批的单元数装得进 u64"),
        user_visible_root_units_relocated: user_visible_root_units_relocated_by_this_publish(
            version,
        ),
        is_an_intent_logged_after_the_publish: observed_logged_compaction_intent(version).is_some(),
        is_the_region_of_the_live_intent_empty_by_the_records_after_the_publish,
        region_forced_for_the_issue: model_compaction_region(region_forced_for_the_issue),
    })
}

/// 整理那一趟被拒（入口拒了、一次发布没发成）时模型此刻的下一次发布：现行那一版还有要走空的就是走空的一步；否则模型没有活意图时是签发
/// （R 取执行器强制的那一段），有活意图时是不删意图的一批（至多 `batch_units` 个）。这两种在模型里都该成，只有容量墙区间允许拒，
/// 判的就是那个拒绝的理由。
fn model_answer_of_a_refused_publish_of_the_compaction(
    model: &IdealModel,
    batch_units: u64,
    region_forced_for_the_issue: CompactionRegion,
) -> Result<ModelAnswer, ModelDisagreement> {
    if model.current_version_has_a_step_to_walk() {
        return model.answer_next_step_of_the_walk();
    }
    let is_an_intent_live = model
        .live_compaction_intent_of_the_current_version()
        .is_some();
    model.answer_compaction_publish(&ModelCompactionPublishSeen {
        units_at_most: batch_units,
        units_relocated: if is_an_intent_live { batch_units } else { 0 },
        user_visible_root_units_relocated: Vec::new(),
        is_an_intent_logged_after_the_publish: true,
        is_the_region_of_the_live_intent_empty_by_the_records_after_the_publish: false,
        region_forced_for_the_issue: model_compaction_region(region_forced_for_the_issue),
    })
}

/// 整理一趟（经会话的显式入口）返回之后：发成的每一版按次序交模型比——走空的一步照「走空的一步」、整理的每一次发布（签发、搬一批、
/// 随最后一批删意图、单独删意图或丢意图）照 [`IdealModel::answer_compaction_publish`]（判决 G7「只比成功操作的内容」；判决 G1 的意图三态）；
/// 分配代逐版判（搬了单元的一版按 [`allocation_generation_judgement_of_a_relocation_batch`]）。停在判据上算做成；入口拒了、一次发布没发成，
/// 照模型此刻的下一次发布比那个拒绝（[`model_answer_of_a_refused_publish_of_the_compaction`]：整理与走空在模型里都该成，只有容量墙区间允许拒）。
#[allow(
    clippy::too_many_arguments,
    reason = "会话、模型、批大小、强制的段、改动之前的记录、整理的结局、停下时意图活不活着、录制流与它的起点各是一样，收成结构体只会多一层没人验的名字"
)]
fn settle_compaction_run(
    session: &mut WritableSession,
    model: &mut IdealModel,
    batch_units: u64,
    region_forced_for_the_issue: CompactionRegion,
    records_before: &[AllocationRecord],
    run_result: &Result<CompactionRun, CompactionRunRefused>,
    is_the_intent_live_when_the_run_stopped: bool,
    stream: &SharedStream,
    stream_length_before: usize,
) -> AppliedStep {
    let mut verdict = ModelVerdict::default();
    let mut harness_judgement = None;
    let versions_published: &[TransactionOutput] = match run_result {
        Ok(run) => &run.versions_published,
        Err(_) => &[],
    };
    let mut records_of_the_previous_publish: &[AllocationRecord] = records_before;
    let mut units_relocated = 0_u64;
    // 迭代上界是这一趟发成的版数；跨轮带的是上一版之后的分配记录与合起来的判定；模型对不上之后不再问模型（分配代照判）。
    for version in versions_published {
        let is_a_relocation_batch = !version.units_relocated_by_this_publish.is_empty();
        if verdict.disagreement.is_none() {
            let answer = model_answer_of_the_next_publish_of_the_compaction(
                model,
                batch_units,
                version,
                region_forced_for_the_issue,
            );
            let version_verdict = judge_by_model(
                model,
                answer,
                &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                    roots: vec![observed_root_of_file_version(version)],
                    reported_ceiling: None,
                }),
            );
            verdict.counts.add(&version_verdict.counts);
            verdict.disagreement = version_verdict.disagreement;
        }
        harness_judgement = harness_judgement.or_else(|| {
            if is_a_relocation_batch {
                allocation_generation_judgement_of_a_relocation_batch(
                    records_of_the_previous_publish,
                    version,
                )
            } else {
                allocation_generation_judgement(
                    records_of_the_previous_publish,
                    &version.allocation_records,
                    version.root.checkpoint_txg,
                    version.root.checkpoint_txg,
                    &version.quarantined_after_release_checksum_mismatch,
                )
            }
        });
        units_relocated +=
            u64::try_from(version.units_relocated_by_this_publish.len()).expect("一批的单元数");
        records_of_the_previous_publish = &version.allocation_records;
        session.publishes_in_this_mount += 1;
    }
    let (member, reason) = match run_result {
        Ok(CompactionRun {
            batches_published,
            outcome: CompactionRunOutcome::Stopped(stop),
            ..
        }) => {
            return AppliedStep {
                outcome: StepOutcome::Applied(AppliedEffect::Compacted {
                    batches_published: *batches_published,
                    units_relocated,
                    stop: stop.name(),
                    is_the_intent_live_when_the_run_stopped,
                    crashed_then_mounted: false,
                }),
                harness_judgement,
                model_verdict: Some(verdict),
            };
        }
        Ok(CompactionRun {
            outcome: CompactionRunOutcome::Failed(CompactionRunFailure::DeletionContinuation(cause)),
            ..
        }) => (
            format!(
                "CompactionRunFailure::DeletionContinuation({})",
                publish_error_member(cause)
            ),
            refusal_reason_of_publish_error(cause),
        ),
        Ok(CompactionRun {
            outcome: CompactionRunOutcome::Failed(CompactionRunFailure::Batch(refused)),
            ..
        }) => match refused {
            RelocationBatchRefused::Publish(cause) => (
                format!(
                    "CompactionRunFailure::Batch(RelocationBatchRefused::Publish({}))",
                    publish_error_member(cause)
                ),
                refusal_reason_of_publish_error(cause),
            ),
            // 计划对不上上一版（这一批之前刚对 R 重扫过）、或单元的字节不在手里：两块内存盘、不注入故障的历史里走不到，
            // 走到了就按说不出理由的拒绝判。
            RelocationBatchRefused::PlannedUnitNotInTheVersion { .. } => (
                "CompactionRunFailure::Batch(RelocationBatchRefused::PlannedUnitNotInTheVersion)"
                    .to_string(),
                ObservedRefusalReason::Unexplained,
            ),
            RelocationBatchRefused::PinnedUnitInThePlan { .. } => (
                "CompactionRunFailure::Batch(RelocationBatchRefused::PinnedUnitInThePlan)"
                    .to_string(),
                ObservedRefusalReason::Unexplained,
            ),
            RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition { .. } => (
                "CompactionRunFailure::Batch(RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition)"
                    .to_string(),
                ObservedRefusalReason::Unexplained,
            ),
            RelocationBatchRefused::InstanceTableOfTheVersionUnreadable(_) => (
                "CompactionRunFailure::Batch(RelocationBatchRefused::InstanceTableOfTheVersionUnreadable)"
                    .to_string(),
                ObservedRefusalReason::Unexplained,
            ),
            RelocationBatchRefused::UnitBytesNotAvailable { .. } => (
                "CompactionRunFailure::Batch(RelocationBatchRefused::UnitBytesNotAvailable)"
                    .to_string(),
                ObservedRefusalReason::Unexplained,
            ),
            // 意图三态那几种是会话推法自己的错（签发前会话判过没有活意图、删之前判过有、R 排他开着时最后一批之后 R 按记录全空）：
            // 执行器只用产品的 R 排他，走不到；走到了就按说不出理由的拒绝判。
            RelocationBatchRefused::CompactionIntentAlreadyLive { .. } => (
                "CompactionRunFailure::Batch(RelocationBatchRefused::CompactionIntentAlreadyLive)"
                    .to_string(),
                ObservedRefusalReason::Unexplained,
            ),
            RelocationBatchRefused::NoLiveCompactionIntent => (
                "CompactionRunFailure::Batch(RelocationBatchRefused::NoLiveCompactionIntent)"
                    .to_string(),
                ObservedRefusalReason::Unexplained,
            ),
            RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
                ..
            } => (
                "CompactionRunFailure::Batch(RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted)"
                    .to_string(),
                ObservedRefusalReason::Unexplained,
            ),
            // 会话照同一批删意图再发，不交上来（判决 m4-segment-code-r1 第三节 F2）：走到了就按说不出理由的拒绝判。
            RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried {
                ..
            } => (
                "CompactionRunFailure::Batch(RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried)"
                    .to_string(),
                ObservedRefusalReason::Unexplained,
            ),
        },
        // 执行器交整池那两块盘、只在有树的一版上调：走不到，走到了就按说不出理由的拒绝判。
        Err(CompactionRunRefused::DeviceTableOtherThanTheOneOfTheMount { .. }) => (
            "CompactionRunRefused::DeviceTableOtherThanTheOneOfTheMount".to_string(),
            ObservedRefusalReason::Unexplained,
        ),
        Err(CompactionRunRefused::NoVersionOfTrees) => (
            "CompactionRunRefused::NoVersionOfTrees".to_string(),
            ObservedRefusalReason::Unexplained,
        ),
    };
    if verdict.disagreement.is_none() {
        let answer = model_answer_of_a_refused_publish_of_the_compaction(
            model,
            batch_units,
            region_forced_for_the_issue,
        );
        let writes_of_the_versions_published: u64 = versions_published
            .iter()
            .map(|version| version.writes.total().write_calls)
            .sum();
        let wrote_anything_besides_what_was_published = if versions_published.is_empty() {
            stream.operation_count() != stream_length_before
        } else {
            writes_in_the_stream_since(stream, stream_length_before)
                > writes_of_the_versions_published
        };
        let refusal_verdict = judge_by_model(
            model,
            answer,
            &ObservedOutcome::Refused {
                member: member.clone(),
                reason,
                publishes_completed: 0,
                wrote_anything: wrote_anything_besides_what_was_published,
                reported_ceiling: None,
                reported_root_ring_slot_still_bad_after_one_reread: None,
            },
        );
        verdict.counts.add(&refusal_verdict.counts);
        verdict.disagreement = refusal_verdict.disagreement;
    }
    AppliedStep {
        outcome: StepOutcome::Refused { member },
        harness_judgement,
        model_verdict: Some(verdict),
    }
}

/// 挂着时整理一趟（`HistoryOperation::CompactWhileMounted`）：会话的分配器装上这一趟的开关（批大小、强制选段），经会话的显式入口推
/// （`MountedSession::run_compaction`），推完换回产品值、交模型逐版比（[`settle_compaction_run`]）。会话这一侧的整理状态（`B_gc` 数的发布数等）
/// 跨步带着（[`WritableSession::compaction`]）。批间崩溃那一种先把这一趟的发布预算压到 `batches` 批：盘上没有活意图时再加签发那一次
/// （签发单独一次发布、也算进 `B_gc`，丁 3 交回的读法第 1、4 条），推完之后这个进程崩掉、照 [`HistoryOperation::CloseAndMountWritable`]
/// 可写挂载一次（两段的判定合起来；挂载被拒时结局是那个拒绝）。意图在 logged_ops 里，崩溃之后的那一版还带着它，下一趟整理接着推它。
/// 前提：现行那一版有树、有能强制选的段（[`segments_the_compaction_may_be_forced_onto`]）。
pub(super) fn apply_compact_while_mounted(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    choice: CompactionChoice,
) -> AppliedStep {
    let run_step = {
        let HistoryPool {
            devices,
            session: session_slot,
            model,
            stream,
            copies_quarantined_in_this_step,
            ..
        } = &mut *pool;
        let Some(open_session) = session_slot.as_ref() else {
            return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
        };
        let Some(version) = open_session.current.version_of_trees() else {
            return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
        };
        let candidates =
            segments_the_compaction_may_be_forced_onto(version, &open_session.allocator);
        let candidate_count = u64::try_from(candidates.len()).expect("段数装得进 u64");
        if candidate_count == 0 {
            return AppliedStep::not_applicable(
                MissingPrecondition::NoSegmentForTheCompactionInTheCurrentVersion,
            );
        }
        let region = candidates
            [usize::try_from(choice.region_selector % candidate_count).expect("小于段数")];
        let batch_units = CompactionBatchUnits::new(
            usize::try_from(choice.batch_units).expect("一批的单元数装得进 usize"),
        )
        .expect("生成时一批至少 1 个单元");
        let records_before = open_session.allocator.records().to_vec();
        let stream_length_before = stream.operation_count();
        let issue_publishes_ahead = u64::from(observed_logged_compaction_intent(version).is_none());
        let WritableSession {
            allocator,
            current,
            instance,
            shadow_ledger,
            parameters_and_device_table,
            publishes_in_this_mount,
            carried_to_zero_unit_publishes: _,
            compaction,
        } = session_slot.take().expect("上面判过这个进程的会话开着");
        let mut mounted_session = MountedSession {
            allocator,
            current,
            instance,
            shadow_ledger,
            parameters_and_device_table,
            compaction,
        };
        mounted_session
            .allocator
            .set_compaction_switches(compaction_switches_of_the_choice(batch_units, region));
        match choice.ending {
            CompactionRunEnding::RunsUntilItStops => {}
            CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches } => {
                // 预算 = max(1, 3S − 4 − 已经过去的发布数)（判决 G8 `B_gc`，签发、搬一批、删意图各算一次）：已经过去的抬到
                // 3S − 4 − (batches + 签发那一次)，这一趟至多发 batches 批（盘上没有活意图时先签发一次）。
                let slots_per_region = mounted_session
                    .parameters_and_device_table
                    .parameters_on_disk()
                    .geometry
                    .root_ring_slots_per_region
                    .count();
                let passed_for_the_budget = (3 * slots_per_region)
                    .saturating_sub(4)
                    .saturating_sub(batches + issue_publishes_ahead);
                let compaction_of_this_mount = &mut mounted_session.compaction;
                compaction_of_this_mount.publishes_since_the_last_user_change =
                    compaction_of_this_mount
                        .publishes_since_the_last_user_change
                        .max(passed_for_the_budget);
            }
        }
        let run_result = mounted_session.run_compaction(devices);
        if let Ok(run) = &run_result {
            copies_quarantined_in_this_step
                .extend(copies_quarantined_by_the_versions(&run.versions_published));
        }
        let is_the_intent_live_when_the_run_stopped = mounted_session
            .allocator
            .compaction()
            .live_intent()
            .is_some();
        mounted_session
            .allocator
            .set_compaction_switches(CompactionSwitches::PRODUCT);
        let session_after_the_run = session_slot.insert(WritableSession {
            allocator: mounted_session.allocator,
            current: mounted_session.current,
            instance: mounted_session.instance,
            shadow_ledger: mounted_session.shadow_ledger,
            parameters_and_device_table: mounted_session.parameters_and_device_table,
            compaction: mounted_session.compaction,
            publishes_in_this_mount,
            // 零单元发布只接在树表 0 条的一版后面，整理只接在有树的一版后面。
            carried_to_zero_unit_publishes: None,
        });
        settle_compaction_run(
            session_after_the_run,
            model,
            choice.batch_units,
            region,
            &records_before,
            &run_result,
            is_the_intent_live_when_the_run_stopped,
            stream,
            stream_length_before,
        )
    };
    let crashes_after_the_run = match choice.ending {
        CompactionRunEnding::RunsUntilItStops => false,
        CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { .. } => true,
    };
    let run_was_judged_without_failure = run_step.harness_judgement.is_none()
        && run_step
            .model_verdict
            .as_ref()
            .is_some_and(|verdict| verdict.disagreement.is_none());
    if !crashes_after_the_run || !run_was_judged_without_failure {
        return run_step;
    }
    let AppliedStep {
        outcome: run_outcome,
        harness_judgement: _,
        model_verdict: run_verdict,
    } = run_step;
    let mount_step = apply_mount_writable(pool, parameters);
    let mut verdict = run_verdict.unwrap_or_default();
    if let Some(mount_verdict) = &mount_step.model_verdict {
        verdict.counts.add(&mount_verdict.counts);
        verdict.disagreement = mount_verdict.disagreement.clone();
    }
    let outcome = match (run_outcome, mount_step.outcome) {
        (
            StepOutcome::Applied(AppliedEffect::Compacted {
                batches_published,
                units_relocated,
                stop,
                is_the_intent_live_when_the_run_stopped,
                crashed_then_mounted: _,
            }),
            StepOutcome::Applied(AppliedEffect::Mounted { .. }),
        ) => StepOutcome::Applied(AppliedEffect::Compacted {
            batches_published,
            units_relocated,
            stop,
            is_the_intent_live_when_the_run_stopped,
            crashed_then_mounted: true,
        }),
        // 整理那一段被拒（模型判过、允许）时，崩溃与挂载照样做；这一步的结局取整理那一段的拒绝。
        (refused @ StepOutcome::Refused { .. }, _) => refused,
        // 挂载被拒或交回别的：这一步的结局取挂载那一段的。
        (_, mount_outcome) => mount_outcome,
    };
    AppliedStep {
        outcome,
        harness_judgement: mount_step.harness_judgement,
        model_verdict: Some(verdict),
    }
}

pub(super) fn apply_publish_without_units(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
) -> AppliedStep {
    let HistoryPool {
        devices,
        session,
        model,
        stream,
        ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    // 前提：只在树表 0 条的一版上调（出处见 `MissingPrecondition::CurrentVersionWithFile` 的注释）。
    let PoolVersion::WithoutFile(previous) = &session.current else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithFile);
    };
    let answer = model.answer_publish_without_units();
    let stream_length_before = stream.operation_count();
    let plan = ZeroUnitPublishPlan {
        txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
        counter: previous.record.counter + 1,
        instance: session.instance,
        back_chain: back_chain_of(&previous.record_bytes),
        rollback_floor: previous.root.rollback_floor,
        // 会话里接着现行那一版：它的水位已经是根环里的 max（挂载时本实例第一次发布取过，之后只照抄或推高）。
        tree_identifier_watermark: previous.root.tree_identifier_watermark,
    };
    let previous_root = previous.root;
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    match publish_without_units(&mut writer, &previous_root, plan) {
        Ok(output) => {
            // 零单元发布从这次挂载里前一版交回的那一份往下带着比（实审 B3c-4）；这次挂载里没有可带的照旧计数。
            let (observed, carried) = observed_root_of_version_without_file_carrying(
                &output,
                session.carried_to_zero_unit_publishes.as_ref(),
            );
            let verdict = judge_by_model(
                model,
                answer,
                &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                    roots: vec![observed],
                    reported_ceiling: None,
                }),
            );
            session.carried_to_zero_unit_publishes = carried;
            session.current = PoolVersion::WithoutFile(output);
            session.publishes_in_this_mount += 1;
            AppliedStep::judged_by_outcome_and_model(
                StepOutcome::Applied(AppliedEffect::Published {
                    reuse: RecordReuse::default(),
                    floor_raises_pushed_by_the_session: 0,
                    roots_carrying_the_change: 1,
                }),
                verdict,
            )
        }
        Err(error) => {
            let member = format!("publish_without_units({})", publish_error_member(&error));
            let verdict = judge_by_model(
                model,
                answer,
                &observed_refusal(
                    member.clone(),
                    refusal_reason_of_publish_error(&error),
                    0,
                    stream_length_before,
                    stream,
                    None,
                    None,
                ),
            );
            AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
        }
    }
}
