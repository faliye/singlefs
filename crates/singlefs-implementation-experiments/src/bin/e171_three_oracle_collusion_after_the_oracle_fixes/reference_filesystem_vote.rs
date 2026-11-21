//! O1-RefFS 子列：参照文件系统的事件流与它对一个崩溃状态的投票。

use super::*;

// ───── O1-RefFS 子列（读法 1、停机 S9）：照 `reference_filesystem_translation.rs` 文件头第 1–18 行与 `events_of` 那几臂交事件 ─────

/// RefFS 翻这一条流停在哪一步、为什么（名字与 `reference_filesystem_translation::TranslationStopReason::name` 交出的一字不差，
/// 比对器按名字认弃权种类）。
#[allow(
    clippy::enum_variant_names,
    reason = "变体名逐个照抄 harness `TranslationStopReason` 的同名臂（注释与 `events_of` 按这几个名字对照），不另起一套"
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ReferenceFileSystemStop {
    /// `events_of` 的 `RollBackWhileMounted` 那一臂：回退没返回 Ok。
    TheRollbackDidNotReportItsTarget,
    /// 同一臂：回退报的目标根在 m 表里查不到。
    TheRollbackTargetRootHasNoVersion,
    /// `events_of` 的 `Unlink` 那一臂：意图那次或走空的一步写了盘（返回 Ok、报错、没返回都算），RefFS 第一版没有删除。
    TheReferenceFileSystemHasNoUnlinkInTheFirstVersion,
    /// `apply_event` 里 `submit` 拒了：RefFS 不收这一步。
    TheReferenceFileSystemDoesNotTakeTheOperation,
}

impl ReferenceFileSystemStop {
    pub(super) fn name(&self) -> &'static str {
        match self {
            Self::TheRollbackDidNotReportItsTarget => "回退没返回 Ok、没报目标根",
            Self::TheRollbackTargetRootHasNoVersion => "回退目标根在 m 表里查不到",
            Self::TheReferenceFileSystemHasNoUnlinkInTheFirstVersion => "RefFS 第一版没有删除",
            Self::TheReferenceFileSystemDoesNotTakeTheOperation => "RefFS 不收这一步",
        }
    }
}

/// 一次调用在 RefFS 里交的事件（`events_of` 的 `Submit`；本装置的流里没有冷启动恢复与抛弃最新根的那次挂载）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ReferenceFileSystemStep {
    pub(super) position: StepPosition,
    events: Vec<(
        reference_filesystem::Operation,
        reference_filesystem::OperationOutcome,
    )>,
    /// 这一步跑完时录制流（mkfs 之后）里有几步：调用的返回点；停下的那一步取 `usize::MAX`（同 `TranslatedStep::without_events`）。
    stream_length_after_the_step: usize,
}

/// 一条流翻成 RefFS 的输入：每一步的事件、翻译停在哪一步、每一段由哪一道屏障关上（m 表只在翻译时要，查回退目标用，不留）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReferenceFileSystemStream {
    steps: Vec<ReferenceFileSystemStep>,
    pub(super) stopped: Option<(StepPosition, ReferenceFileSystemStop)>,
    pub(super) segment_closing_stream_indexes: Vec<Option<usize>>,
}

/// 一次调用翻成的 RefFS 事件，或这一步翻不出（[`ReferenceFileSystemStop`]）。臂与 `events_of` 一一对应（停机 S9，逐臂写在注释里）。
/// 交回值的第二样：这一步出了新版本时，这一步写的根里是不是只有最后一条映到新版本（发布与回退是；`NewVersionWrittenBy`）。
fn reference_filesystem_events_of_a_call(
    call: &RecordedCall,
    versions_of_the_roots: &BTreeMap<ModelRootKey, reference_filesystem::VersionNumber>,
) -> Result<
    (
        Vec<(
            reference_filesystem::Operation,
            reference_filesystem::OperationOutcome,
        )>,
        bool,
    ),
    ReferenceFileSystemStop,
> {
    use reference_filesystem::{FileContent, Operation, OperationOutcome};
    // 发布调用的结局：返回 Ok 是「成」；交回错误、panic、在途都按「写了之后报错」交——`events_of` 的 `publish_with_content`：
    // ReturnedAnErrorAfterItMayHaveWritten 与 NeverReturned 都交 FailedAfterWriting（本装置的流停在出错的那一步，同
    // `note_the_step_the_history_stopped_in` 把入口报错按「写了之后报错」补上那一条）。
    let publish_outcome = match call.outcome {
        CallOutcome::Succeeded => OperationOutcome::Succeeded,
        CallOutcome::ReturnedError(_)
        | CallOutcome::Panicked(_)
        | CallOutcome::InFlightAtTheTruncation => OperationOutcome::FailedAfterWriting,
    };
    match call.label {
        // `events_of`：起点 AfterMakeFilesystem 交 nothing（取号、暖机是 mkfs 那个进程里的起点）；CloseAndMountWritable、
        // RaiseRollbackFloor、CreateInode 交 nothing（不改内容，写的根映到现行版本）；卸载与抬 F 同一类（文件头第 9 行「抬 F、卸载」）。
        "acquire" | "warm_up" | "mount_writable" | "raise_floor" | UNMOUNT_CALL_LABEL
        | "create_inode" => Ok((Vec::new(), false)),
        // `events_of`：PublishFirstFile / PublishOverwrite → 内容改动（成）+ 发布调用（照结局）。
        "publish" => {
            let change = match &call.published_content {
                Some(PublishedContent::FirstFile(content)) => {
                    Operation::CreateTheFirstFile(FileContent(content.clone()))
                }
                Some(PublishedContent::Overwrite(content)) => {
                    Operation::OverwriteTheFile(FileContent(content.clone()))
                }
                None => panic!("发布调用都记了交给实现的内容：{call:?}"),
            };
            Ok((
                vec![
                    (change, OperationOutcome::Succeeded),
                    (Operation::ExplicitPublish, publish_outcome),
                ],
                true,
            ))
        }
        // `events_of`：RollBackWhileMounted 返回 Ok → 回退到 m（成），m 在 m 表里查；没返回 Ok → TheRollbackDidNotReportItsTarget。
        // 本装置的回退目标写死是 A 的根（实例 1、txg 3，`step_roll_back`）。
        "roll_back" => match call.outcome {
            CallOutcome::Succeeded => {
                let target_root = model_root_key(InstanceGeneration(1), CheckpointTxg(3));
                let version = versions_of_the_roots
                    .get(&target_root)
                    .copied()
                    .ok_or(ReferenceFileSystemStop::TheRollbackTargetRootHasNoVersion)?;
                Ok((
                    vec![(
                        Operation::RollBackToVersion(version),
                        OperationOutcome::Succeeded,
                    )],
                    true,
                ))
            }
            CallOutcome::ReturnedError(_)
            | CallOutcome::Panicked(_)
            | CallOutcome::InFlightAtTheTruncation => {
                Err(ReferenceFileSystemStop::TheRollbackDidNotReportItsTarget)
            }
        },
        // `events_of`：Unlink 写了盘（返回 Ok、写了之后报错、没返回）→ TheReferenceFileSystemHasNoUnlinkInTheFirstVersion。
        // 本装置的 unlink 调用要么返回 Ok、要么在出错或 panic 那一步停（按写了之后报错算），没有「在任何写之前拒」那一臂。
        "unlink_intent" | "unlink_step" => {
            Err(ReferenceFileSystemStop::TheReferenceFileSystemHasNoUnlinkInTheFirstVersion)
        }
        unknown => panic!("RefFS 映射里没有这种调用：{unknown}"),
    }
}

/// 一条流翻成 RefFS 的输入（`ReferenceFileSystemTranslation::translate` 的同一套走法：每一步交事件、攒 m 表、记每段由哪一道屏障关上）。
/// `operations` 是 mkfs 之后的录制流，与调用的返回点同一个原点；根槽写取写表里种类是根槽的那几次（与 `translate` 按几何认根槽写同一组）。
pub(super) fn reference_filesystem_stream_of(
    calls: &[RecordedCall],
    operations: &[RetainedOperation],
    writes: &[RetainedWrite],
    stream_indexes: &[usize],
) -> ReferenceFileSystemStream {
    let root_writes: Vec<(usize, ModelRootKey)> = writes
        .iter()
        .zip(stream_indexes)
        .filter(|(write, _)| write.kind == StepKind::RootRecordFua)
        .map(|(write, stream_index)| {
            let (instance, checkpoint_txg) =
                singlefs_verification::memory_pool::root_identity_of_write(write);
            (*stream_index, model_root_key(instance, checkpoint_txg))
        })
        .collect();
    let mut model = reference_filesystem::ReferenceFileSystem::on_a_freshly_formatted_pool();
    let mut versions_of_the_roots = BTreeMap::new();
    let mut steps = Vec::new();
    let mut stopped = None;
    let mut stream_length_before_the_step = 0usize;
    for (call_index, call) in calls.iter().enumerate() {
        let position = StepPosition::Operation(call_index);
        let version_before_the_step = model.newest_version();
        let applied = reference_filesystem_events_of_a_call(call, &versions_of_the_roots).and_then(
            |(events, only_the_last_root_is_new)| {
                for (operation, outcome) in &events {
                    model.submit(operation.clone(), *outcome).map_err(|_| {
                        ReferenceFileSystemStop::TheReferenceFileSystemDoesNotTakeTheOperation
                    })?;
                }
                Ok((events, only_the_last_root_is_new))
            },
        );
        let (events, only_the_last_root_is_new) = match applied {
            Ok(applied) => applied,
            Err(stop) => {
                stopped = Some((position, stop));
                steps.push(ReferenceFileSystemStep {
                    position,
                    events: Vec::new(),
                    stream_length_after_the_step: usize::MAX,
                });
                break;
            }
        };
        let version_after_the_step = model.newest_version();
        let roots_of_the_step: Vec<ModelRootKey> = root_writes
            .iter()
            .filter(|(stream_index, _)| {
                (stream_length_before_the_step..call.return_point).contains(stream_index)
            })
            .map(|(_, root)| *root)
            .collect();
        for (index_in_the_step, root) in roots_of_the_step.iter().enumerate() {
            let is_the_last_root = index_in_the_step + 1 == roots_of_the_step.len();
            let version = if only_the_last_root_is_new && !is_the_last_root {
                version_before_the_step
            } else {
                version_after_the_step
            };
            versions_of_the_roots.insert(*root, version);
        }
        stream_length_before_the_step = call.return_point;
        steps.push(ReferenceFileSystemStep {
            position,
            events,
            stream_length_after_the_step: call.return_point,
        });
    }
    ReferenceFileSystemStream {
        steps,
        stopped,
        segment_closing_stream_indexes: segment_closing_stream_indexes(operations),
    }
}

/// RefFS 对一个崩溃态的票（读法 1：弃权不算红、也不算绿，单列种类）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ReferenceFileSystemVoteOfAState {
    Green {
        allowed: String,
    },
    Red {
        reason: String,
        allowed: String,
    },
    Abstained {
        kind: &'static str,
    },
    /// 这一态上恢复 panic：读回没有，RefFS 不判（同 O1-模型崩溃判官，读法 5）。
    RecoveryPanicked,
    /// 这一态上 RefFS 自己 panic（修订 5 第 5.1 节第 3 条）：不红不绿，与弃权分开计。
    JudgePanicked,
    /// 关一条（Q3-核）关掉了 O1：RefFS 子列一起不算。
    Off,
}

impl ReferenceFileSystemVoteOfAState {
    pub(super) fn word(&self) -> &'static str {
        match self {
            Self::Green { .. } => "green",
            Self::Red { .. } => "red",
            Self::Abstained { .. } => "abstained",
            Self::RecoveryPanicked => "recovery_panic",
            Self::JudgePanicked => "judge_panic",
            Self::Off => "off",
        }
    }
}

/// 允许集合的下标写成 `S_0,S_2`。
fn allowed_state_indexes_text(allowed: &reference_filesystem::AllowedSet) -> String {
    allowed
        .allowed
        .iter()
        .map(|state| format!("S_{}", state.index.0))
        .collect::<Vec<String>>()
        .join(",")
}

/// 崩在第 `segment_index` 段里（序号越过最后一段的那个全持久态交 `None`）的崩溃落在哪一步：关上这一段的那道屏障由哪一步发出
/// （`crash_moment_of_a_segment_closed_at`，H1-收严的归属；段没关上的、全持久态，算在全部步都返回之后）。
pub(super) fn reference_filesystem_crash_moment(
    stream: &ReferenceFileSystemStream,
    segment_index: Option<usize>,
) -> ReferenceFileSystemCrashMoment {
    let closing_stream_index = segment_index
        .and_then(|segment| stream.segment_closing_stream_indexes.get(segment).copied())
        .flatten();
    let stream_length_after_each_step: Vec<(StepPosition, usize)> = stream
        .steps
        .iter()
        .map(|step| (step.position, step.stream_length_after_the_step))
        .collect();
    crash_moment_of_a_segment_closed_at(closing_stream_index, &stream_length_after_each_step)
}

pub(super) fn crash_moment_text(moment: ReferenceFileSystemCrashMoment) -> String {
    match moment {
        ReferenceFileSystemCrashMoment::InsideStep(StepPosition::Operation(call_index)) => {
            format!("call_{call_index}")
        }
        ReferenceFileSystemCrashMoment::InsideStep(StepPosition::StartingPoint) => {
            "starting_point".to_string()
        }
        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep => "after_every_step".to_string(),
    }
}

/// 一个崩溃态上 RefFS 的票（`TranslatedHistory::judge_a_crash` 的同一套走法）：从头交事件到崩溃那一步，hi 取那一步最后进序列的操作号
/// （那一步一个都没进是「都返回过之后」的号）；翻译停在崩溃之前或崩溃落在停下的那一步里弃权；读回失败判红；读回不在允许集合里判红。
// gpu-exempt: reference-filesystem
pub(super) fn reference_filesystem_vote_of_a_state(
    stream: &ReferenceFileSystemStream,
    moment: ReferenceFileSystemCrashMoment,
    read_back: &ObservedReadBack,
) -> ReferenceFileSystemVoteOfAState {
    use reference_filesystem::{CrashNotAccepted, FileContent, VisibleState};
    let mut model = reference_filesystem::ReferenceFileSystem::on_a_freshly_formatted_pool();
    let mut crash_falls_in = None;
    for step in &stream.steps {
        if let Some((_, stop)) = stream
            .stopped
            .as_ref()
            .filter(|(position, _)| *position == step.position)
        {
            return ReferenceFileSystemVoteOfAState::Abstained { kind: stop.name() };
        }
        let mut last_entered = None;
        for (operation, outcome) in &step.events {
            match model
                .submit(operation.clone(), *outcome)
                .expect("建翻译时同一串事件交给同一个起点的 RefFS 收过")
            {
                reference_filesystem::SubmittedOperation::Entered(number) => {
                    last_entered = Some(number);
                }
                reference_filesystem::SubmittedOperation::DidNotHappen => {}
            }
        }
        if moment == ReferenceFileSystemCrashMoment::InsideStep(step.position) {
            crash_falls_in =
                Some(last_entered.unwrap_or_else(|| model.crash_after_every_operation_returned()));
            break;
        }
    }
    let crash_falls_in =
        crash_falls_in.unwrap_or_else(|| model.crash_after_every_operation_returned());
    let recovered = match read_back {
        ObservedReadBack::FileRead { content, .. } => {
            VisibleState::OneFile(FileContent(content.clone()))
        }
        ObservedReadBack::NoFile { .. } => VisibleState::NoFile,
        ObservedReadBack::Failed { what } => {
            return ReferenceFileSystemVoteOfAState::Red {
                reason: format!("恢复失败、没有读出的状态（{what}）"),
                allowed: "-".to_string(),
            };
        }
    };
    match model.crash_and_recover(crash_falls_in, recovered) {
        Ok(allowed) => ReferenceFileSystemVoteOfAState::Green {
            allowed: allowed_state_indexes_text(&allowed),
        },
        Err(CrashNotAccepted::RecoveredStateOutsideTheAllowedSet { allowed, recovered }) => {
            ReferenceFileSystemVoteOfAState::Red {
                reason: format!(
                    "崩溃落在第 {} 个操作里，读出 {}",
                    crash_falls_in.0,
                    match recovered {
                        VisibleState::NoFile => "没有文件".to_string(),
                        VisibleState::OneFile(FileContent(bytes)) =>
                            format!("文件 {} 字节（末字节 {:?}）", bytes.len(), bytes.last()),
                    }
                ),
                allowed: allowed_state_indexes_text(&allowed),
            }
        }
        Err(CrashNotAccepted::OperationAfterAFailureAfterWritingBeforeTheNextRecoveryIsNotInTheFirstVersion { .. }) => {
            ReferenceFileSystemVoteOfAState::Abstained {
                kind: "写后报错之后又起了操作",
            }
        }
        Err(refusal @ CrashNotAccepted::CrashOperationIsOutsideTheOperationsSinceTheLastRecovery { .. }) => {
            panic!("hi 取的是这一步最后进序列的操作号或「都返回过之后」那个号，不会越界：{refusal:?}")
        }
    }
}

#[cfg(test)]
mod tests;
