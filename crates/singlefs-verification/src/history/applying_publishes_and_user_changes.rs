//! 施加发布第一个文件与覆盖写，按会话交回的抬 F 与版本判一次用户改动。

use super::*;

pub(super) fn apply_publish_first_file(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    content_choice: &ContentChoice,
    write_time_seconds: u64,
) -> AppliedStep {
    let HistoryPool {
        devices,
        session,
        model,
        stream,
        copies_quarantined_in_this_step,
        ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let content = content_choice.bytes();
    let root_to_carry_instance_table_from = *session.current.root();
    let previous_record_bytes = session.current.record_bytes().to_vec();
    let records_before = session.allocator.records().to_vec();
    let answer = model.answer_publish_first_file(&content);
    let stream_length_before = stream.operation_count();
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    match publish_first_file(
        &mut writer,
        &mut session.allocator,
        &root_to_carry_instance_table_from,
        FirstFile {
            content: &content,
            write_time_seconds,
        },
        session.instance,
        &previous_record_bytes,
    ) {
        Ok(output) => {
            copies_quarantined_in_this_step.extend(copies_quarantined_by_the_versions([&output]));
            settle_file_publish(session, model, answer, &records_before, output)
        }
        Err(error) => {
            settle_refused_file_publish(model, answer, &error, stream, stream_length_before)
        }
    }
}

/// 第一个文件、覆盖写被拒：成员映射成理由，交模型比。
fn settle_refused_file_publish(
    model: &mut IdealModel,
    answer: Result<ModelAnswer, ModelDisagreement>,
    error: &PublishError,
    stream: &SharedStream,
    stream_length_before: usize,
) -> AppliedStep {
    let member = publish_error_member(error);
    let reason = refusal_reason_of_publish_error(error);
    let verdict = judge_by_model(
        model,
        answer,
        &observed_refusal(
            member.clone(),
            reason,
            0,
            stream_length_before,
            stream,
            None,
            None,
        ),
    );
    AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
}

/// 第一个文件、覆盖写做成之后：数复用、判分配代（第 1 件）、拿输出与模型比（第 2 件）、把现行版本换成这一版。
fn settle_file_publish(
    session: &mut WritableSession,
    model: &mut IdealModel,
    answer: Result<ModelAnswer, ModelDisagreement>,
    records_before: &[AllocationRecord],
    output: singlefs_core::transaction::TransactionOutput,
) -> AppliedStep {
    let reuse = RecordReuse::between(records_before, session.allocator.records());
    let publish_txg = output.root.checkpoint_txg;
    let harness_judgement = allocation_generation_judgement(
        records_before,
        session.allocator.records(),
        publish_txg,
        publish_txg,
        &output.quarantined_after_release_checksum_mismatch,
    );
    let verdict = judge_by_model(
        model,
        answer,
        &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
            roots: vec![observed_root_of_file_version(&output)],
            reported_ceiling: None,
        }),
    );
    session.current = PoolVersion::WithFile(Arc::new(output));
    session.carried_to_zero_unit_publishes = None;
    session.publishes_in_this_mount += 1;
    AppliedStep {
        outcome: StepOutcome::Applied(AppliedEffect::Published {
            reuse,
            floor_raises_pushed_by_the_session: 0,
            roots_carrying_the_change: 1,
        }),
        harness_judgement,
        model_verdict: Some(verdict),
    }
}

pub(super) fn apply_publish_overwrite(
    pool: &mut HistoryPool,
    content_choice: &ContentChoice,
    write_time_seconds: u64,
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
    let Some(current_file_version) = open_session.current.file_version() else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    if current_file_version.inode_record.is_none() {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutTheFirstFile);
    }
    // 现行那一版有删除中的文件（unlink 的一批被拒之后、回退到删除中的一版之后）：照调，会话先走空、把它走成有树而文件集合为空的一版、
    // 再拒成 `UserChangeRefused::NoFileVersionToChange`，那个成员带着走空的那几步与推的抬 F（代码轮 U10），逐版交模型比（代码轮 U11）。
    // 有快照在销毁中（挂载续做被空间拒之后，判决 S4：删快照照 unlink 的过程做）、截断走到半路（回退到截断半路的一版之后）同样先走空，
    // 那几批同样逐版交模型比。判的是实现判「队列空」那一个判断。
    let queue_is_walked_first = !current_file_version.queue_of_committed_intents_is_empty();
    let content = content_choice.bytes();
    let records_before = open_session.allocator.records().to_vec();
    let stream_length_before = stream.operation_count();
    // 覆盖写是用户的一次改动，经挂着的会话发布（`MountedSession::publish_user_change`，实七）：发布路径报空间不够时会话先推一串抬 F、
    // 再重判（D16（发布语义） 已定项 1「准入」那一行，C283）。会话借走这个进程的分配器、现行版本与认下的池，发完原样还回来；
    // 会话只按它认下的那份参数建写入口，执行器手里的参数不交进去（实审 A1b Q2）。
    let WritableSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        publishes_in_this_mount,
        carried_to_zero_unit_publishes,
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
    let file = FirstFile {
        content: &content,
        write_time_seconds,
    };
    // 内容跨几个数据单元的走顺序写入口（代码轮 U13），别的走覆盖写入口；模型那一侧各按各的契约答。
    let entry = content_choice.length.whole_file_write_entry();
    let change_result = mounted_session.publish_user_change(
        devices,
        match entry {
            WholeFileWriteEntry::Overwrite => UserChange::Overwrite(file),
            WholeFileWriteEntry::SequentialWrite => UserChange::SequentialWrite(file),
        },
    );
    let answer_of_the_change = |ideal_model: &IdealModel| match entry {
        WholeFileWriteEntry::Overwrite => ideal_model.answer_publish_overwrite(&content),
        WholeFileWriteEntry::SequentialWrite => {
            ideal_model.answer_publish_sequential_write(&content)
        }
    };
    copies_quarantined_in_this_step.extend(copies_quarantined_by_a_user_change(&change_result));
    let session_after_the_change = session_slot.insert(WritableSession {
        allocator: mounted_session.allocator,
        current: mounted_session.current,
        instance: mounted_session.instance,
        shadow_ledger: mounted_session.shadow_ledger,
        parameters_and_device_table: mounted_session.parameters_and_device_table,
        compaction: mounted_session.compaction,
        publishes_in_this_mount,
        carried_to_zero_unit_publishes,
    });
    if queue_is_walked_first {
        return settle_user_change_comparing_every_version_published(
            session_after_the_change,
            model,
            &answer_of_the_change,
            &records_before,
            &change_result,
            stream,
            stream_length_before,
        );
    }
    settle_user_change(
        session_after_the_change,
        model,
        &answer_of_the_change,
        &records_before,
        change_result,
        stream,
        stream_length_before,
    )
}

/// 会话推的那几串抬 F（D16（发布语义） 已定项 1「准入」那一行）：逐串照模型的抬 F 比（新 F 就是那一串的根带的、上限是实现报的），
/// 与挂载那一处（`settle_mount`）同一个比法；前面一处对不上就不再往下比。交回合起来的判定与这几串一共写出几条根。
fn judge_floor_raises_pushed_by_the_session(
    model: &mut IdealModel,
    floor_raises: &[singlefs_core::mount::RaisedFloor],
) -> (ModelVerdict, usize) {
    let mut verdict = ModelVerdict::default();
    let mut publishes = 0;
    for raised in floor_raises {
        if verdict.disagreement.is_some() {
            break;
        }
        let raise_verdict = judge_one_floor_raise_pushed_by_the_session(model, raised);
        verdict.counts.add(&raise_verdict.counts);
        verdict.disagreement = raise_verdict.disagreement;
        publishes += raised.publishes.len();
    }
    (verdict, publishes)
}

/// 准入推的一串空发布（会话、挂载写行之后与挂载续做里推的都走它）交给模型：按 [`singlefs_core::mount::RaisedFloor::floor_of_the_publishes`] 分两种。
/// 抬到上限的那一串照模型的抬 F 比（新 F 就是那一串的根带的、上限是实现报的）。
/// 上限卡在挂载那一刻读坏或读回全零的槽上推的那一次不抬 F 的空发布（D16（发布语义） 已定项 1「抬 F 的上限」那一行的推空）
/// 模型不答：模型不建挂载时读坏、读回全零的根槽，判不了上限卡没卡在它们上，当抬 F 比就拿实现带现行 F 的那一次去对模型推到每块盘的一串，
/// 报的是假的对不上（判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q4 第 4 问）。这一份交回
/// [`ModelDisagreementAspect::EmptyPublishKeepingTheFloorWhileTheCeilingIsHeldIsNotModeled`]（带实现报的上限），模型不往前走；
/// 故障注入照被吞的根槽写认它（`fault_injection::left_by_the_lying_device`）。
#[must_use]
pub fn judge_one_floor_raise_pushed_by_the_session(
    model: &mut IdealModel,
    raised: &singlefs_core::mount::RaisedFloor,
) -> ModelVerdict {
    match raised.floor_of_the_publishes {
        FloorOfThePushedPublishes::RaisedToTheCeiling => {
            judge_one_floor_raise_raised_to_the_ceiling(model, raised)
        }
        FloorOfThePushedPublishes::KeptWhileTheCeilingIsHeldByRingSlotsBadSinceTheMount {
            ceiling_from_the_roots_read,
        } => ModelVerdict {
            disagreement: Some(ModelDisagreement {
                aspect:
                    ModelDisagreementAspect::EmptyPublishKeepingTheFloorWhileTheCeilingIsHeldIsNotModeled,
                model_answer: "模型不建挂载那一刻读坏、读回全零的根槽，答不了上限卡没卡在它们上（D16（发布语义） 已定项 1「抬 F 的上限」那一行的推空）"
                    .to_string(),
                implementation_answer: format!(
                    "实现报上限 {} 卡在挂载时读坏或读回全零的槽上（不并它们时 {}），推了 {} 次不抬 F 的空发布",
                    raised.ceiling.0,
                    ceiling_from_the_roots_read.0,
                    raised.publishes.len()
                ),
                implementation_reported_ceiling: Some(ModelCheckpointTxg(raised.ceiling.0)),
                implementation_reported_unreadable_place: None,
                implementation_rows_written: None,
            }),
            counts: ModelJudgementCounts::default(),
        },
    }
}

/// 抬到上限的一串抬 F 照模型的抬 F 比：新 F 就是那一串的根带的、上限是实现报的。
fn judge_one_floor_raise_raised_to_the_ceiling(
    model: &mut IdealModel,
    raised: &singlefs_core::mount::RaisedFloor,
) -> ModelVerdict {
    let raised_floor = raised
        .publishes
        .first()
        .expect("抬 F 做成至少推一次空发布：推到每块盘上都有一条带新 F 的根")
        .root
        .rollback_floor;
    let raise_answer = model.answer_raise_rollback_floor(ModelCheckpointTxg(raised_floor.0));
    judge_by_model(
        model,
        raise_answer,
        &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
            roots: raised
                .publishes
                .iter()
                .map(observed_root_of_file_version)
                .collect(),
            reported_ceiling: Some(ModelCheckpointTxg(raised.ceiling.0)),
        }),
    )
}

/// 经会话的覆盖写返回之后：先把会话推的那几串抬 F 交模型逐串比，再比这次覆盖写——做成的比根与分配代、数复用
/// （比的是改动那一版，不是会话的现行那一版：读到只坏一份时改动之后另发一批，现行那一版是那一批，
/// [`settle_what_the_session_published_for_the_relocation_batch`] 接着比）；
/// 被拒的比理由，「拒之前写没写盘」只看那几串抬 F 之外多出来的写（推了抬 F 的时候那几串的写不算这次拒绝写的）。
/// 分配代逐次比：盘上的上一版（会话开始时分配器里的记录）→ 每一次空发布的记录 → 覆盖写之后分配器里的记录，各按那一次的 txg 判。
/// 会话推抬 F 半路报错（`UserChangeRefused::FloorRaiseFailedWhilePushingForSpace`）时，那一串已落盘的那几次空发布不在 `floor_raises` 里：模型那一侧
/// 比不到它们，接下来的历史里模型与实现的现行版本对不上，由下一步的比较报出来（与挂载那一处推满途中报错同一个现状）。
#[allow(
    clippy::too_many_arguments,
    reason = "会话、模型、内容、改动之前的记录、会话的结局、录制流与它的起点各是一样，收成结构体只会多一层没人验的名字"
)]
fn settle_user_change(
    session: &mut WritableSession,
    model: &mut IdealModel,
    answer_of_the_change: &dyn Fn(&IdealModel) -> Result<ModelAnswer, ModelDisagreement>,
    records_before: &[AllocationRecord],
    change_result: Result<UserChangePublished, UserChangeRefused>,
    stream: &SharedStream,
    stream_length_before: usize,
) -> AppliedStep {
    let floor_raises: &[singlefs_core::mount::RaisedFloor] = match &change_result {
        Ok(published) => &published.floor_raises,
        Err(
            UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
            | UserChangeRefused::PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration { .. }
            | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
            | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
            | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. },
        ) => &[],
        Err(
            UserChangeRefused::Publish { floor_raises, .. }
            | UserChangeRefused::NoFileVersionToChange { floor_raises, .. },
        ) => floor_raises,
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused)) => &refused.floor_raises,
        Err(UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(refused)) => {
            &refused.floor_raises
        }
        Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(refused)) => {
            &refused.floor_raises
        }
    };
    let (mut verdict, publishes_of_the_floor_raises) =
        judge_floor_raises_pushed_by_the_session(model, floor_raises);
    session.publishes_in_this_mount += publishes_of_the_floor_raises;
    // 分配代：每一次空发布按它自己的 txg，上一版是前一次空发布（第一次是会话开始时分配器里的记录）。
    let mut records_of_the_previous_publish: &[AllocationRecord] = records_before;
    let mut harness_judgement = None;
    for publish in floor_raises
        .iter()
        .flat_map(|raised| raised.publishes.iter())
    {
        let publish_txg = publish.root.checkpoint_txg;
        harness_judgement = harness_judgement.or_else(|| {
            allocation_generation_judgement(
                records_of_the_previous_publish,
                &publish.allocation_records,
                publish_txg,
                publish_txg,
                &publish.quarantined_after_release_checksum_mismatch,
            )
        });
        records_of_the_previous_publish = &publish.allocation_records;
    }
    let refusal = match &change_result {
        Ok(published) => {
            let answer = answer_of_the_change(model);
            let output: &TransactionOutput = versions_of_the_change_without_the_relocation_batch(published)
                .last()
                .expect("会话发成一次改动：这次调用发成的那几版里有改动自己那一版，这里只接没有要走空的一版，它就是最后一版");
            let publish_txg = output.root.checkpoint_txg;
            harness_judgement = harness_judgement.or_else(|| {
                allocation_generation_judgement(
                    records_of_the_previous_publish,
                    &output.allocation_records,
                    publish_txg,
                    publish_txg,
                    &output.quarantined_after_release_checksum_mismatch,
                )
            });
            if verdict.disagreement.is_none() {
                let overwrite_verdict = judge_by_model(
                    model,
                    answer,
                    &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                        roots: vec![observed_root_of_file_version(output)],
                        reported_ceiling: None,
                    }),
                );
                verdict.counts.add(&overwrite_verdict.counts);
                verdict.disagreement = overwrite_verdict.disagreement;
            }
            session.publishes_in_this_mount += 1;
            let roots_of_the_relocation_batch =
                settle_what_the_session_published_for_the_relocation_batch(
                    session,
                    model,
                    published,
                    &output.allocation_records,
                    &mut verdict,
                    &mut harness_judgement,
                );
            return AppliedStep {
                outcome: StepOutcome::Applied(AppliedEffect::Published {
                    reuse: RecordReuse::between(records_before, session.allocator.records()),
                    floor_raises_pushed_by_the_session: floor_raises.len(),
                    roots_carrying_the_change: 1 + roots_of_the_relocation_batch,
                }),
                harness_judgement,
                model_verdict: Some(verdict),
            };
        }
        Err(refused) => refused,
    };
    let (member, reason) = member_and_reason_of_a_refused_user_change(refusal);
    if verdict.disagreement.is_none() {
        let answer = answer_of_the_change(model);
        let wrote_anything_besides_the_floor_raises = if floor_raises.is_empty() {
            stream.operation_count() != stream_length_before
        } else {
            writes_in_the_stream_since(stream, stream_length_before)
                > writes_of_the_floor_raises(floor_raises)
        };
        let refusal_verdict = judge_by_model(
            model,
            answer,
            &ObservedOutcome::Refused {
                member: member.clone(),
                reason,
                publishes_completed: 0,
                wrote_anything: wrote_anything_besides_the_floor_raises,
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

/// 一次用户改动被拒：成员的名字与拒绝的理由。
pub(super) fn member_and_reason_of_a_refused_user_change(
    refusal: &UserChangeRefused,
) -> (String, ObservedRefusalReason) {
    match refusal {
        // 执行器每次都交整池两块盘、与会话认下的那一份同次序：走不到，走到了就按说不出理由的拒绝判。
        UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. } => (
            "UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount".to_string(),
            ObservedRefusalReason::Unexplained,
        ),
        // 有树而文件集合为空的一版上不是建 inode 的改动（会话先走空、走成这一形之后拒）：D8（核心索引结构） 已定项 16 第 5 条那一条理由。
        UserChangeRefused::NoFileVersionToChange { .. } => (
            "UserChangeRefused::NoFileVersionToChange".to_string(),
            ObservedRefusalReason::Explained(
                ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile,
            ),
        ),
        // 执行器交的是整池那两块盘、没换过盘（物理块大小那一判）：走不到，走到了就按说不出理由的拒绝判。
        UserChangeRefused::PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration {
            ..
        } => (
            "UserChangeRefused::PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration"
                .to_string(),
            ObservedRefusalReason::Unexplained,
        ),
        // 执行器交的是整池那两块盘、没换过盘（Z3-A 乙那一判）：走不到，走到了就按说不出理由的拒绝判。
        UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. } => (
            "UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration".to_string(),
            ObservedRefusalReason::Unexplained,
        ),
        // 执行器交的盘体各在自己的身份上、没换过旧快照（代码三方 m2-closeout-code-r2 那两判）：走不到，走到了就按说不出理由的拒绝判。
        UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. } => (
            "UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable".to_string(),
            ObservedRefusalReason::Unexplained,
        ),
        UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. } => (
            "UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot"
                .to_string(),
            ObservedRefusalReason::Unexplained,
        ),
        UserChangeRefused::Publish { cause, .. } => (
            publish_error_member(cause),
            refusal_reason_of_publish_error(cause),
        ),
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused) => (
            format!(
                "UserChangeRefused::NoSpaceAfterRaisingTheFloor({})",
                publish_error_member(&refused.last_refusal)
            ),
            refusal_reason_of_publish_error(&refused.last_refusal),
        ),
        UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(refused) => (
            format!(
                "UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease({})",
                publish_error_member(&refused.last_refusal)
            ),
            refusal_reason_of_publish_error(&refused.last_refusal),
        ),
        UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(refused) => (
            format!(
                "UserChangeRefused::FloorRaiseFailedWhilePushingForSpace({})",
                mount_error_member(&refused.cause)
            ),
            refusal_reason_of_mount_error(&refused.cause),
        ),
    }
}

/// 一次用户改动交回的推的抬 F 与推的抬 F 之外发成的那几版（被拒的几种里带着的，见 `UserChangeRefused` 各成员）。
pub(super) fn floor_raises_and_versions_published_of_a_user_change(
    change_result: &Result<UserChangePublished, UserChangeRefused>,
) -> (
    &[singlefs_core::mount::RaisedFloor],
    &[Arc<singlefs_core::transaction::TransactionOutput>],
) {
    match change_result {
        Ok(published) => (&published.floor_raises, &published.versions_published),
        Err(UserChangeRefused::NoFileVersionToChange {
            floor_raises,
            versions_published,
            ..
        }) => (floor_raises, versions_published),
        Err(
            UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
            | UserChangeRefused::PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration { .. }
            | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
            | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
            | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. },
        ) => (&[], &[]),
        Err(UserChangeRefused::Publish {
            floor_raises,
            versions_published,
            ..
        }) => (floor_raises, versions_published),
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused)) => {
            (&refused.floor_raises, &refused.versions_published)
        }
        Err(UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(refused)) => {
            (&refused.floor_raises, &refused.versions_published)
        }
        Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(refused)) => {
            (&refused.floor_raises, &refused.versions_published)
        }
    }
}

/// 一次 unlink（或挂载续做）里落了盘的一样：推的一串抬 F，或推的抬 F 之外发成的一版（意图那次、走空的一步、建 inode 那次）。
pub(super) enum FloorRaiseOrVersion<'change> {
    FloorRaise(&'change singlefs_core::mount::RaisedFloor),
    Version(&'change singlefs_core::transaction::TransactionOutput),
}

/// 推的那几串抬 F 与发成的那几版按 txg 排好（一串抬 F 按它第一次空发布的 txg）：两样在会话与挂载续做里按落盘的先后交错。
pub(super) fn floor_raises_and_versions_in_txg_order<'change>(
    floor_raises: &'change [singlefs_core::mount::RaisedFloor],
    versions_published: &'change [Arc<singlefs_core::transaction::TransactionOutput>],
) -> Vec<FloorRaiseOrVersion<'change>> {
    let mut published_in_txg_order: Vec<(CheckpointTxg, FloorRaiseOrVersion<'change>)> =
        floor_raises
            .iter()
            .map(|raised| {
                let first_publish_txg = raised
                    .publishes
                    .first()
                    .expect("抬 F 做成至少推一次空发布")
                    .root
                    .checkpoint_txg;
                (first_publish_txg, FloorRaiseOrVersion::FloorRaise(raised))
            })
            .chain(versions_published.iter().map(|version| {
                (
                    version.root.checkpoint_txg,
                    FloorRaiseOrVersion::Version(Arc::as_ref(version)),
                )
            }))
            .collect();
    published_in_txg_order.sort_by_key(|(txg, _)| *txg);
    published_in_txg_order
        .into_iter()
        .map(|(_, published)| published)
        .collect()
}
